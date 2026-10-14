//! The single-player run (R1, sim 120; rooms R1b, sim 122): the room window (dropping old rooms, the left wall, no backtracking past
//! it), the camera, arenas, the shop, rest stops and the generator's rules; mobs, coins and the combo; and determinism.

mod common;

use common::inp;
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::run::data::{MobKind, RunContent};
use sim_core::run::{
    self, door_here, new_run, step_run, Mob, GATE_INDEX, MAX_MOBS, MODE_ATTACK, MODE_BURST,
    MODE_IDLE, MODE_MOVE, MODE_WINDUP, NOTE_BOUGHT, NOTE_NOT_NEEDED, NOTE_SOLD_OUT, NOTE_TOO_POOR,
    PLAYER, RING, ROOM_ARENA, ROOM_REST, ROOM_SHOP, ROOM_STREET, WALL_INDEX,
};
use sim_core::state::{FighterState as S, DRAW, HISTORY_LEN, NONE};
use sim_core::{collision, Content, Fx, GameState, Input, Rng, Vec2, MAX_FIGHTERS};

fn fx(n: i32) -> Fx {
    Fx::from_int(n)
}

struct RunSim {
    world: Content,
    rc: RunContent,
    state: GameState,
}

impl RunSim {
    fn with_char(rc: RunContent, seed: u64, char_id: u8) -> RunSim {
        let mut world = Content::placeholder();
        let state = new_run(&mut world, &rc, seed, char_id);
        RunSim { world, rc, state }
    }

    fn with(rc: RunContent, seed: u64) -> RunSim {
        RunSim::with_char(rc, seed, 0)
    }

    fn new(seed: u64) -> RunSim {
        RunSim::with(RunContent::placeholder(), seed)
    }

    fn tick(&mut self, p0: Input) {
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = p0;
        step_run(&mut self.state, &mut self.world, &self.rc, &inputs);
    }

    fn idle(&mut self, frames: u32) {
        for _ in 0..frames {
            self.tick(Input::default());
        }
    }

    fn player(&self) -> &sim_core::Fighter {
        &self.state.fighters[PLAYER]
    }

    fn clear_mobs(&mut self) {
        for m in self.state.run.mobs.iter_mut() {
            m.active = false;
        }
    }

    /// Moves the camera to where it would hold a player at `x` (inside the window), as if it had followed.
    fn look_at(&mut self, x: Fx) {
        let r = self.rc.rules;
        let run = &mut self.state.run;
        let hi = (run.right_limit() - r.view_width).max(run.wall_x);
        run.cam_x = (x - r.view_width * r.follow).max(run.wall_x).min(hi);
    }

    /// Puts the player in the air at `at`, still (the camera on it).
    fn place(&mut self, at: Vec2) {
        self.look_at(at.x);
        let f = &mut self.state.fighters[PLAYER];
        f.pos = at;
        f.vel = Vec2::ZERO;
        f.platform = NONE;
        f.state = S::Airborne;
        f.state_frame = 0;
    }

    /// Stands the player on the ground at `at` (the camera on it). Where the ground is not at that height (the run's ground rises
    /// and falls), on the highest ground under `at` lifted by ten.
    fn stand(&mut self, at: Vec2) {
        self.look_at(at.x);
        sim_core::run::build_stage(&self.state.run, &self.rc, &mut self.world.stage);
        let mut at = at;
        if collision::standing_on(&self.world.stage, at) == NONE {
            if let Some((_, d)) =
                collision::surface_below(&self.world.stage, at + Vec2::new(Fx::ZERO, fx(10)), false)
            {
                at.y = at.y + fx(10) - d;
            }
        }
        let f = &mut self.state.fighters[PLAYER];
        f.pos = at;
        f.vel = Vec2::ZERO;
        f.platform = collision::standing_on(&self.world.stage, at);
        assert_ne!(f.platform, NONE, "no ground at {at:?}");
        f.state = S::Idle;
        f.state_frame = 0;
    }

    /// A mob of `kind` standing (or hovering) at `at`, facing the player.
    fn add_mob(&mut self, kind: u8, at: Vec2) -> usize {
        let k = self.rc.mobs[usize::from(kind)];
        let slot = self.state.run.mobs.iter().position(|m| !m.active).unwrap();
        self.state.run.mobs[slot] = Mob {
            active: true,
            kind,
            mode: MODE_MOVE,
            pos: at,
            facing: if at.x > self.player().pos.x { -1 } else { 1 },
            platform: collision::standing_on(&self.world.stage, at),
            stamina: k.stamina,
            room: self.state.run.room,
            ..Mob::default()
        };
        slot
    }

    /// Walks (teleports) the player into the next room, clearing an arena it locks; returns the room's type.
    fn next_room(&mut self) -> u8 {
        let target = self.state.run.room + 1;
        let slot = *self
            .state
            .run
            .slot_of(target)
            .expect("the room ahead is laid");
        let at = Vec2::new(
            slot.x + self.rc.rules.enter_depth + Fx::ONE,
            slot.y + fx(12),
        );
        self.place(at);
        self.state.fighters[PLAYER].invuln = 30;
        self.tick(Input::default());
        assert_eq!(
            self.state.run.room,
            target,
            "entered: player {:?} {:?} slot x {:?} y {:?}",
            self.player().pos,
            self.player().state,
            slot.x,
            slot.y
        );
        let kind = self.rc.rooms[usize::from(slot.room)].kind;
        if self.state.run.gate_closed {
            self.clear_mobs();
            self.tick(Input::default());
            assert!(!self.state.run.gate_closed, "cleared");
        }
        kind
    }
}

/// The built-in rooms with no mobs in the streets, and no arenas (so walking on never locks).
fn quiet() -> RunContent {
    let mut rc = RunContent::placeholder();
    for r in rc.rooms.iter_mut().filter(|r| r.kind == ROOM_STREET) {
        r.spawns.clear();
    }
    rc.rules.arena_weight = 0;
    rc
}

/// The start room, then only the rooms named (the start is the fallback street); every street's mobs gone; the room types' odds
/// `(street, arena, rest)`.
fn only(names: &[&str], weights: (u8, u8, u8)) -> RunContent {
    let mut rc = quiet();
    let start = rc.rooms[0].clone();
    rc.rooms.retain(|r| names.contains(&r.name.as_str()));
    rc.rooms.insert(0, start);
    (
        rc.rules.street_weight,
        rc.rules.arena_weight,
        rc.rules.rest_weight,
    ) = weights;
    rc
}

// ---- The room window ------------------------------------------------------------------------------------------------------------

/// Walking on through many rooms: the window holds the current room, at most `rooms_behind` behind it and `rooms_ahead` ahead, joined
/// up; the left wall is the oldest room's left edge; every platform of every room in play is in the stage (ground behind the wall
/// trimmed to it), and nothing of a dropped room is left.
#[test]
fn the_window_keeps_the_rooms_near_and_drops_old_ones() {
    for seed in 0..6u64 {
        let mut sim = RunSim::with(quiet(), seed);
        let r = sim.rc.rules;
        for _ in 0..40 {
            sim.next_room();
            let run = &sim.state.run;
            let mut live: Vec<_> = run.slots.iter().filter(|s| s.live).collect();
            live.sort_by_key(|s| s.x);
            assert!(live.len() <= RING);
            let first = run.room.saturating_sub(u32::from(r.rooms_behind));
            let want: Vec<u32> = (first..=run.room + u32::from(r.rooms_ahead)).collect();
            let seqs: Vec<u32> = live.iter().map(|s| s.seq).collect();
            assert_eq!(seqs, want, "seed {seed}: the window");
            for w in live.windows(2) {
                assert_eq!(w[0].end, w[1].x, "seed {seed}: rooms do not join");
            }
            assert_eq!(
                run.wall_x, live[0].x,
                "the wall stands at the oldest room kept"
            );
            for s in &live {
                let c = &sim.rc.rooms[usize::from(s.room)];
                let mut laid = Vec::new();
                run::for_each_footing(&sim.rc, s, |placed| laid.push(placed.platform));
                for p in &laid {
                    let (l, rr) = (p.left, p.right);
                    let found = run.platforms.iter().any(|e| {
                        e.live
                            && e.pass_through == p.pass_through
                            && e.rise == p.rise
                            && e.left <= l.max(run.wall_x)
                            && e.right >= rr
                            && (e.y == p.y || l < run.wall_x && e.left == run.wall_x)
                    });
                    assert!(
                        found,
                        "seed {seed}: a platform of room {} is missing",
                        c.name
                    );
                }
            }
            for p in run.platforms.iter().filter(|p| p.live) {
                assert!(
                    p.left >= run.wall_x,
                    "seed {seed}: a platform left of the wall"
                );
            }
            for l in run.ledges.iter().filter(|l| l.live) {
                assert!(l.x > run.wall_x, "seed {seed}: a ledge left of the wall");
            }
        }
        assert!(sim.state.run.placed > 40);
    }
}

/// A room dropped behind takes its mobs with it, and the ground the player stands on (joined across the seams) keeps its pool index,
/// trimmed to the new wall.
#[test]
fn dropping_a_room_trims_the_ground_and_removes_its_mobs() {
    let mut sim = RunSim::with(only(&["shop"], (1, 0, 0)), 1);
    // (Only the flat start room and the shop: one floor all along.)
    sim.idle(5);
    let floor = sim.player().platform;
    assert_ne!(floor, NONE);
    let x0 = sim.player().pos.x;
    let left_mob = sim.add_mob(0, Vec2::new(x0 + fx(4), Fx::ZERO));
    sim.state.run.mobs[left_mob].mode = MODE_IDLE;
    for _ in 0..3 {
        let slot = *sim.state.run.slot_of(sim.state.run.room + 1).unwrap();
        sim.stand(Vec2::new(slot.x + fx(3), Fx::ZERO));
        sim.state.run.mobs[left_mob].pos.x = x0 + fx(4);
        sim.tick(Input::default());
    }
    let run = &sim.state.run;
    assert_eq!(run.room, 3);
    assert!(run.wall_x > fx(0), "the first room is dropped");
    assert_eq!(
        sim.player().platform,
        floor,
        "the same floor, by the same index"
    );
    let p = run.platforms[floor as usize];
    assert!(
        p.live && p.left == run.wall_x,
        "trimmed to the wall: {:?}",
        p.left
    );
    assert!(!run.mobs[left_mob].active, "its mob went with it");
}

/// The window's sizes are rules: one room behind keeps less.
#[test]
fn the_window_sizes_are_rules() {
    let mut rc = quiet();
    rc.rules.rooms_behind = 1;
    let mut sim = RunSim::with(rc, 2);
    for _ in 0..5 {
        sim.next_room();
    }
    let live = sim.state.run.slots.iter().filter(|s| s.live).count();
    assert_eq!(live, 3, "one behind, the current room, one ahead");
}

// ---- The left wall -----------------------------------------------------------------------------------------------------------------

/// The left wall is a solid block of the stage: running into it stops, and a fighter that can wall-jump kicks off it.
#[test]
fn the_gate_wall_is_a_real_wall() {
    // (Class 3 can wall-jump.)
    let mut sim = RunSim::with_char(quiet(), 3, 3);
    for _ in 0..4 {
        sim.next_room();
    }
    let wall = sim.state.run.wall_x;
    let block = sim.world.stage.platforms[WALL_INDEX];
    assert!(!block.pass_through && block.right == wall);
    assert!(block.y > sim.rc.rules.blast_top, "nothing gets over it");
    // Running left into it.
    sim.stand(Vec2::new(wall + fx(4), Fx::ZERO));
    for _ in 0..120 {
        sim.tick(inp(-127, 0, 0));
        assert!(sim.player().pos.x > wall, "through the wall");
    }
    assert!(sim.player().pos.x < wall + fx(1), "stopped at it");
    // Falling beside it, the stick held away and jump: a wall jump.
    let half = sim.world.fighters[3].ecb_half_width;
    sim.place(Vec2::new(wall + half, fx(3)));
    let f = &mut sim.state.fighters[PLAYER];
    f.vel.y = -Fx::from_ratio(1, 10);
    f.facing = -1;
    f.history = [inp(127, 0, 0); HISTORY_LEN];
    sim.tick(inp(127, 0, 0));
    sim.tick(inp(127, 0, 0));
    sim.tick(inp(127, 0, JUMP));
    assert_eq!(sim.player().wall_jumps, 1, "a wall jump off the gate");
    assert!(sim.player().vel.x > Fx::ZERO);
}

/// However the player tries (running, jumping, launched), it never gets behind the wall, and the camera never shows past it.
#[test]
fn no_backtracking_past_the_gate() {
    let mut sim = RunSim::with(quiet(), 5);
    for _ in 0..3 {
        sim.next_room();
    }
    let wall = sim.state.run.wall_x;
    sim.stand(Vec2::new(wall + fx(30), Fx::ZERO));
    for n in 0..900 {
        let jump = if n % 40 == 0 { JUMP } else { 0 };
        sim.tick(inp(-127, 0, jump));
        assert!(sim.player().pos.x > wall, "frame {n}: behind the wall");
        assert!(
            sim.state.run.cam_x >= wall,
            "the camera shows past the wall"
        );
    }
    assert!(sim.player().pos.x < wall + fx(3), "it got to the wall");
    assert_eq!(sim.state.run.room, 3, "going back enters nothing");
    // Launched hard at it.
    sim.place(Vec2::new(wall + fx(5), fx(4)));
    let f = &mut sim.state.fighters[PLAYER];
    f.state = S::Hitstun;
    f.hitstun = 40;
    f.kb_vel = Vec2::new(-Fx::ONE, Fx::from_ratio(1, 4));
    for _ in 0..40 {
        sim.tick(Input::default());
        assert!(sim.player().pos.x > wall - Fx::HALF);
    }
    assert_eq!(
        sim.player().stocks,
        3,
        "the wall stops a launch: no knock-out there"
    );
    // A mob cannot get behind it either.
    let m = sim.add_mob(1, Vec2::new(wall + fx(2), fx(6)));
    sim.state.run.mobs[m].vel = Vec2::new(-fx(2), Fx::ZERO);
    sim.tick(Input::default());
    assert!(sim.state.run.mobs[m].pos.x > wall);
}

// ---- The camera ------------------------------------------------------------------------------------------------------------------

/// The camera follows the player right and back, smoothly, inside the window; with no creep it never moves by itself.
#[test]
fn the_camera_follows_both_ways_inside_the_window() {
    // (Flat rooms: the start and the shop.)
    let mut sim = RunSim::with(only(&["shop"], (1, 0, 0)), 1);
    let start = sim.state.run.cam_x;
    sim.idle(120);
    assert_eq!(sim.state.run.cam_x, start, "no creep");
    for _ in 0..150 {
        let before = sim.state.run.cam_x;
        sim.tick(inp(127, 0, 0));
        assert!(sim.state.run.cam_x - before <= sim.rc.rules.cam_max_speed);
    }
    let far = sim.state.run.cam_x;
    assert!(far > start + fx(10), "it followed: {far:?}");
    for _ in 0..150 {
        sim.tick(inp(-127, 0, 0));
    }
    assert!(sim.state.run.cam_x < far - fx(5), "and came back");
    let run = &sim.state.run;
    assert!(run.cam_x >= run.wall_x);
    assert!(run.cam_x + sim.rc.rules.view_width <= run.right_limit());
}

/// With the creep switched on (an A/B option) the view's left edge creeps on and pushes the fighter, but never crushes it.
#[test]
fn an_optional_creep_pushes_but_never_crushes() {
    let mut rc = quiet();
    rc.rules.creep = Fx::from_ratio(1, 20);
    rc.rules.creep_max = Fx::from_ratio(1, 20);
    let mut sim = RunSim::with(rc, 1);
    let x0 = sim.player().pos.x;
    sim.idle(400);
    assert!(sim.state.run.cam_x > fx(10), "it crept");
    assert!(sim.player().pos.x > x0 + fx(5), "and pushed");
    assert_eq!(sim.player().stocks, 3, "no harm");
}

// ---- Lives -------------------------------------------------------------------------------------------------------------------------

#[test]
fn a_fall_into_a_pit_costs_a_life() {
    let mut sim = RunSim::with(only(&["gap", "shop"], (1, 0, 0)), 1);
    // (The gap room: put the player over its pit.)
    let gap = sim.rc.rooms.iter().position(|r| r.name == "gap").unwrap();
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(usize::from(slot.room), gap, "the only street but the start");
    sim.place(Vec2::new(slot.x + fx(13), fx(-3)));
    for _ in 0..240 {
        sim.tick(inp(0, -127, 0));
        if sim.player().stocks < 3 {
            break;
        }
    }
    assert_eq!(sim.player().stocks, 2);
}

#[test]
fn leaving_the_screen_costs_a_life_and_the_last_life_ends_the_run() {
    let mut sim = RunSim::with(quiet(), 1);
    for lives in (0..3u8).rev() {
        let top = sim.rc.rules.blast_top;
        sim.place(Vec2::new(sim.state.run.cam_x + fx(10), top + fx(1)));
        sim.state.fighters[PLAYER].invuln = 0;
        sim.tick(Input::default());
        assert_eq!(sim.player().stocks, lives);
    }
    assert!(!sim.player().active);
    assert!(sim.state.run.over);
    assert_eq!(sim.state.winner, DRAW);
    let before = sim.state.run;
    sim.idle(10);
    assert_eq!(
        sim.state.run.cam_x, before.cam_x,
        "a finished run stands still"
    );
}

// ---- Arenas ------------------------------------------------------------------------------------------------------------------------

/// Entering an arena shuts its gate (a wall) and wakes all its mobs; the player cannot leave by the right until they are all beaten;
/// then the gate opens and the arena pays its bonus.
#[test]
fn an_arena_locks_until_its_mobs_are_beaten() {
    let mut sim = RunSim::with(only(&["yard", "shop"], (0, 1, 0)), 4);
    let slot = *sim.state.run.slot_of(1).unwrap();
    let arena = &sim.rc.rooms[usize::from(slot.room)];
    assert_eq!(arena.kind, ROOM_ARENA);
    let mobs = arena.spawns.len();
    // Not shut before entering.
    sim.stand(Vec2::new(slot.x - fx(2), Fx::ZERO));
    sim.tick(Input::default());
    assert!(!sim.state.run.gate_closed);
    sim.stand(Vec2::new(slot.x + fx(3), Fx::ZERO));
    sim.state.fighters[PLAYER].invuln = 250;
    sim.tick(Input::default());
    let run = &sim.state.run;
    assert!(run.gate_closed, "shut");
    assert_eq!(run.gate_x, slot.end);
    let theirs: Vec<usize> = (0..MAX_MOBS)
        .filter(|&i| run.mobs[i].active && run.mobs[i].room == 1)
        .collect();
    assert_eq!(theirs.len(), mobs, "every mob of the arena is out");
    assert!(
        theirs.iter().all(|&i| run.mobs[i].mode != MODE_IDLE),
        "and awake"
    );
    let gate = sim.world.stage.platforms[GATE_INDEX];
    assert!(
        !gate.pass_through && gate.left == slot.end,
        "the gate is a wall"
    );
    // The camera stays inside the gate.
    assert!(sim.state.run.cam_x + sim.rc.rules.view_width <= slot.end);
    // Running at the gate: stopped.
    sim.stand(Vec2::new(slot.end - fx(4), Fx::ZERO));
    for _ in 0..90 {
        sim.tick(inp(127, 0, 0));
        assert!(sim.player().pos.x < slot.end, "through the gate");
        // (The mobs kept off the player.)
        for &i in &theirs {
            if sim.state.run.mobs[i].active {
                sim.state.run.mobs[i].pos.x = slot.x + fx(3);
            }
        }
    }
    assert_eq!(sim.state.run.room, 1);
    // All but one beaten: still shut. A mob in its burst counts as beaten.
    let coins = sim.state.run.coins;
    for &i in &theirs[1..] {
        sim.state.run.mobs[i].active = false;
    }
    sim.state.run.mobs[theirs[0]].mode = MODE_MOVE;
    sim.tick(Input::default());
    assert!(sim.state.run.gate_closed);
    let last = &mut sim.state.run.mobs[theirs[0]];
    last.mode = MODE_BURST;
    last.timer = 20;
    sim.tick(Input::default());
    let run = &sim.state.run;
    assert!(!run.gate_closed, "open");
    assert_eq!(run.cleared, 1);
    assert_eq!(run.coins, coins + u32::from(sim.rc.rules.arena_bonus));
    sim.idle(30);
    // On through it into the next room.
    for _ in 0..120 {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(sim.state.run.room, 2, "through the open gate");
}

// ---- The shop and rest stops -----------------------------------------------------------------------------------------------------

/// The door of the (only) shop room laid after the start: its room's slot and the door's feet, in the world.
fn shop_door(sim: &RunSim) -> (run::RoomSlot, Vec2) {
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(sim.rc.rooms[usize::from(slot.room)].kind, ROOM_SHOP);
    (slot, run::door_of(&sim.rc, &slot).unwrap())
}

/// Stands the player inside the shop at interior x `x` (on the floor), the camera untouched.
fn stand_inside(sim: &mut RunSim, x: Fx) {
    assert!(sim.state.run.inside);
    run::build_stage(&sim.state.run, &sim.rc, &mut sim.world.stage);
    let at = Vec2::new(fx(run::INTERIOR_X) + x, Fx::ZERO);
    let f = &mut sim.state.fighters[PLAYER];
    f.pos = at;
    f.vel = Vec2::ZERO;
    f.platform = collision::standing_on(&sim.world.stage, at);
    assert_ne!(f.platform, NONE, "the shop's floor");
    f.state = S::Idle;
    f.state_frame = 0;
}

/// Up, then the stick let go (the shop reads presses).
fn press_up(sim: &mut RunSim) {
    sim.tick(inp(0, 127, 0));
    sim.tick(Input::default());
}

/// Pressing up at a shop's door walks in: the shop's interior becomes the stage, the fighter stands inside its door, the shopkeeper
/// waves, and the world outside stands still while the fighter walks about inside. Away from the door, up does nothing.
#[test]
fn walking_into_a_shop() {
    let mut rc = only(&["shop"], (1, 0, 0));
    rc.rules.shop_min = 1;
    rc.rules.shop_max = 1;
    let mut sim = RunSim::with(rc, 1);
    let (_, door) = shop_door(&sim);
    sim.stand(Vec2::new(door.x - fx(5), door.y));
    sim.tick(Input::default());
    sim.tick(inp(0, 127, 0));
    assert!(!sim.state.run.inside);
    sim.stand(door);
    sim.tick(Input::default());
    assert!(door_here(&sim.state, &sim.rc).is_some());
    let outside = sim.state.run;
    sim.tick(inp(0, 127, 0));
    let run = sim.state.run;
    assert!(run.inside, "in the shop");
    assert_eq!(run.shop_room, 1);
    assert_eq!(run.keeper_mood, run::KEEPER_WAVE, "the shopkeeper waves");
    let shop = sim.rc.interior.clone();
    assert_eq!(
        sim.player().pos,
        Vec2::new(fx(run::INTERIOR_X) + shop.enter, Fx::ZERO),
        "inside its door"
    );
    assert_ne!(sim.player().platform, NONE, "on the shop's floor");
    assert_eq!(
        sim.world.stage.platforms[sim.player().platform as usize].left,
        fx(run::INTERIOR_X),
        "the interior is the stage"
    );
    // Walking about inside; the world outside stands still (its rooms, mobs, camera and coins).
    let frame = sim.state.frame;
    for _ in 0..40 {
        sim.tick(inp(127, 0, 0));
    }
    assert!(
        sim.player().pos.x > fx(run::INTERIOR_X) + shop.enter + fx(2),
        "it walks inside"
    );
    assert!(
        sim.player().pos.x < fx(run::INTERIOR_X) + shop.width,
        "and stays inside"
    );
    assert_eq!(sim.state.frame, frame + 40, "the frame counts on");
    let now = sim.state.run;
    assert_eq!(now.cam_x, outside.cam_x);
    assert_eq!(now.slots, outside.slots);
    assert_eq!(now.mobs, outside.mobs);
    assert_eq!(now.platforms, outside.platforms);
    assert_eq!(now.coins, outside.coins);
    // Waving is over after a while.
    sim.idle(u32::from(shop.wave_frames));
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_IDLE);
}

/// Inside, the player buys by walking up to a ware's pedestal and pressing up: the shopkeeper looks at the ware stood by, cheers a
/// purchase and grumbles at a refusal (coins short, sold out, not needed: nothing paid). One extra life a shop, remembered after
/// leaving and coming back. Up at the door goes back out to the street, at the storefront's door.
#[test]
fn buying_by_walking_up_and_leaving() {
    let mut rc = only(&["shop"], (1, 0, 0));
    rc.rules.shop_min = 1;
    rc.rules.shop_max = 1;
    let mut sim = RunSim::with(rc, 1);
    let (_, door) = shop_door(&sim);
    sim.stand(door);
    sim.tick(Input::default());
    press_up(&mut sim);
    assert!(sim.state.run.inside);
    let shop = sim.rc.interior.clone();
    let heal = sim.rc.items[0].clone();
    let life = sim.rc.items[1].clone();
    // By the heal: the shopkeeper looks at it.
    stand_inside(&mut sim, shop.pedestals[0].x);
    sim.tick(Input::default());
    assert_eq!(
        sim.state.run.keeper_look, 0,
        "it looks at the ware stood by"
    );
    assert_eq!(run::shop_spot(&sim.state, &sim.rc), Some(Some(0)));
    // No coins: refused, grumbled at.
    sim.state.fighters[PLAYER].percent = fx(50);
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, NOTE_TOO_POOR);
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_GRUMBLE);
    assert_eq!(sim.player().percent, fx(50));
    // With coins: healed, paid, cheered.
    sim.state.run.coins = 500;
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, NOTE_BOUGHT);
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_CHEER);
    assert_eq!(sim.player().percent, fx(50) - heal.amount);
    assert_eq!(sim.state.run.coins, 500 - u32::from(heal.price));
    // The extra life, at its own pedestal: bought once, then sold out.
    stand_inside(&mut sim, shop.pedestals[1].x);
    sim.tick(Input::default());
    assert_eq!(sim.state.run.keeper_look, 1);
    let coins = sim.state.run.coins;
    press_up(&mut sim);
    assert_eq!(sim.player().stocks, 4);
    assert_eq!(sim.state.run.coins, coins - u32::from(life.price));
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, NOTE_SOLD_OUT);
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_GRUMBLE);
    assert_eq!(sim.player().stocks, 4);
    // Up at the door: out, at the storefront's door, on the street.
    stand_inside(&mut sim, shop.door);
    sim.tick(Input::default());
    assert_eq!(run::shop_spot(&sim.state, &sim.rc), Some(None));
    press_up(&mut sim);
    assert!(!sim.state.run.inside, "back outside");
    assert_eq!(sim.player().pos, door);
    assert_ne!(sim.player().platform, NONE, "standing on the street");
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_IDLE);
    // Back in: the shop remembers the life it sold.
    sim.idle(10);
    sim.stand(door);
    sim.tick(Input::default());
    press_up(&mut sim);
    assert!(sim.state.run.inside);
    stand_inside(&mut sim, shop.pedestals[1].x);
    sim.tick(Input::default());
    press_up(&mut sim);
    assert_eq!(
        sim.state.run.shop_note, NOTE_SOLD_OUT,
        "still one life a shop"
    );
    // Healing at 0%: not needed, nothing paid.
    stand_inside(&mut sim, shop.pedestals[0].x);
    sim.state.fighters[PLAYER].percent = Fx::ZERO;
    sim.tick(Input::default());
    let coins = sim.state.run.coins;
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, NOTE_NOT_NEEDED);
    assert_eq!(sim.state.run.coins, coins);
    // Away from the pedestals and the door, up does nothing.
    stand_inside(&mut sim, (shop.pedestals[3].x + shop.keeper.x) / fx(2));
    sim.tick(Input::default());
    let acts = sim.state.run.shop_acts;
    press_up(&mut sim);
    assert!(sim.state.run.inside);
    assert_eq!(sim.state.run.shop_acts, acts);
}

/// Hitting the shopkeeper makes it dizzy for a moment ("hey!"), and nothing else: no damage anywhere, no coins, no lives, no
/// hitlag, and it cannot be bonked again while it is seeing stars.
#[test]
fn hitting_the_shopkeeper_only_makes_it_dizzy() {
    let mut rc = only(&["shop"], (1, 0, 0));
    rc.rules.shop_min = 1;
    rc.rules.shop_max = 1;
    let mut sim = RunSim::with(rc, 1);
    let (_, door) = shop_door(&sim);
    sim.stand(door);
    sim.tick(Input::default());
    press_up(&mut sim);
    let shop = sim.rc.interior.clone();
    // Between the counter and the shopkeeper, facing it.
    stand_inside(&mut sim, shop.keeper.x - fx(1));
    sim.state.fighters[PLAYER].facing = 1;
    let before = (
        sim.state.run.coins,
        sim.player().percent,
        sim.player().stocks,
    );
    let mut dizzy_at = None;
    for t in 0..40 {
        sim.tick(if t % 12 == 0 {
            inp(0, 0, ATTACK)
        } else {
            Input::default()
        });
        if dizzy_at.is_none() && sim.state.run.keeper_mood == run::KEEPER_DIZZY {
            dizzy_at = Some(t);
            assert_eq!(sim.player().hitlag, 0, "no hitlag: it is not a hit");
        }
    }
    assert!(dizzy_at.is_some(), "the jab bonked it");
    assert_eq!(sim.state.run.keeper_bonks, 1, "once while dizzy");
    assert_eq!(
        (
            sim.state.run.coins,
            sim.player().percent,
            sim.player().stocks
        ),
        before,
        "nothing harmful happened"
    );
    sim.idle(u32::from(shop.dizzy_frames));
    assert_eq!(
        sim.state.run.keeper_mood,
        run::KEEPER_IDLE,
        "it shakes it off"
    );
    assert!(sim.state.run.inside);
}

#[test]
fn a_rest_stop_heals_on_entering() {
    let mut sim = RunSim::with(only(&["garden", "shop"], (0, 0, 1)), 1);
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(sim.rc.rooms[usize::from(slot.room)].kind, ROOM_REST);
    sim.state.fighters[PLAYER].percent = fx(50);
    sim.tick(Input::default());
    sim.stand(Vec2::new(slot.x + fx(3), Fx::ZERO));
    sim.tick(Input::default());
    assert_eq!(sim.player().percent, fx(50) - sim.rc.rules.rest_heal);
    assert_eq!(sim.state.run.combo, 0);
}

// ---- The generator --------------------------------------------------------------------------------------------------------------

/// The rooms' types over long runs: the first is the start street with no mobs; never two arenas (or two rest stops) in a row; a
/// shop every `shop_min` to `shop_max` rooms; every type comes; and the seed decides it all.
#[test]
fn the_generator_keeps_its_rules() {
    let kinds_of = |seed: u64| {
        let mut sim = RunSim::new(seed);
        let mut kinds =
            vec![sim.rc.rooms[usize::from(sim.state.run.slot_of(0).unwrap().room)].kind];
        let mut rooms = vec![usize::from(sim.state.run.slot_of(0).unwrap().room)];
        for _ in 0..80 {
            kinds.push(sim.next_room());
            rooms.push(usize::from(
                sim.state.run.slot_of(sim.state.run.room).unwrap().room,
            ));
        }
        (kinds, rooms, sim.rc.rules)
    };
    let mut seen = [0u32; 4];
    for seed in 0..12u64 {
        let (kinds, rooms, r) = kinds_of(seed);
        assert_eq!(rooms[0], 0, "a run starts in the start room");
        assert!(
            rooms[1..].iter().all(|&i| i != 0),
            "the start room never comes again"
        );
        assert!(kinds
            .windows(2)
            .all(|w| !(w[0] == ROOM_ARENA && w[1] == ROOM_ARENA)));
        assert!(kinds
            .windows(2)
            .all(|w| !(w[0] == ROOM_REST && w[1] == ROOM_REST)));
        let shops: Vec<usize> = (0..kinds.len())
            .filter(|&i| kinds[i] == ROOM_SHOP)
            .collect();
        assert!(shops.len() >= 10, "seed {seed}: shops {shops:?}");
        let mut last = 0;
        for &s in &shops {
            let gap = s - last;
            assert!(
                gap >= usize::from(r.shop_min) && gap <= usize::from(r.shop_max),
                "seed {seed}: a shop {gap} rooms after the last: {shops:?}"
            );
            last = s;
        }
        for k in kinds {
            seen[usize::from(k)] += 1;
        }
        assert_eq!(kinds_of(seed).0, kinds_of(seed).0);
    }
    assert!(seen.iter().all(|&n| n > 0), "every type comes: {seen:?}");
    assert!(seen[usize::from(ROOM_STREET)] > seen[usize::from(ROOM_ARENA)]);
    assert!(seen[usize::from(ROOM_ARENA)] > seen[usize::from(ROOM_REST)]);
    assert_ne!(kinds_of(1).1, kinds_of(2).1, "another seed, other rooms");
}

// ---- Mobs ---------------------------------------------------------------------------------------------------------------------------

/// The player's attack launches a mob with the versus formulas: more knockback at a higher percent, hitlag on both, hitstun.
#[test]
fn a_mob_takes_knockback_that_grows_with_its_percent() {
    let launch = |percent: i32| {
        let mut sim = RunSim::with(quiet(), 1);
        sim.clear_mobs();
        let x = sim.player().pos.x;
        let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
        sim.state.run.mobs[n].percent = fx(percent);
        sim.state.run.mobs[n].stamina = fx(500);
        let mut hit = false;
        for frame in 0..30 {
            sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
            let m = sim.state.run.mobs[n];
            if m.percent > fx(percent) {
                hit = true;
                assert!(m.hitlag > 0, "the mob freezes");
                assert!(sim.player().hitlag > 0, "and so does the attacker");
                assert!(m.hitstun > 0);
                break;
            }
        }
        assert!(hit, "the jab missed");
        let m = sim.state.run.mobs[n];
        assert_eq!(sim.state.run.combo, 1, "a hit counts in the combo");
        m.kb_vel.length()
    };
    assert!(launch(100) > launch(0));
}

#[test]
fn a_mob_out_of_stamina_bursts_and_pays() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
    sim.state.run.mobs[n].stamina = Fx::from_ratio(1, 2);
    for frame in 0..30 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        if sim.state.run.kills > 0 {
            break;
        }
    }
    assert_eq!(sim.state.run.kills, 1);
    assert_eq!(sim.state.run.mobs[n].mode, MODE_BURST);
    assert_eq!(sim.state.run.combo, 2);
    assert_eq!(sim.state.run.coins, u32::from(sim.rc.mobs[0].coins));
    sim.idle(u32::from(sim.rc.rules.burst_frames) + 40);
    assert!(!sim.state.run.mobs[n].active, "gone after its burst");
}

#[test]
fn a_mob_launched_off_screen_is_a_kill_only_if_the_player_hit_it() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    let top = sim.rc.rules.blast_top;
    let a = sim.add_mob(1, Vec2::new(fx(20), top + fx(1)));
    let b = sim.add_mob(1, Vec2::new(fx(22), top + fx(1)));
    sim.state.run.mobs[a].credit = 30;
    sim.tick(Input::default());
    assert!(!sim.state.run.mobs[a].active && !sim.state.run.mobs[b].active);
    assert_eq!(sim.state.run.kills, 1);
    assert_eq!(sim.state.run.coins, u32::from(sim.rc.mobs[1].coins));
}

#[test]
fn a_walker_rushes_and_hits_the_player_and_breaks_the_combo() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    sim.state.run.combo = 7;
    sim.state.run.best_combo = 7;
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(8), Fx::ZERO));
    let mut hit = false;
    let mut wound_up = false;
    for _ in 0..240 {
        sim.tick(Input::default());
        let m = sim.state.run.mobs[n];
        wound_up |= m.mode == MODE_WINDUP;
        if sim.player().percent > Fx::ZERO {
            hit = true;
            assert_eq!(m.mode, MODE_ATTACK);
            break;
        }
    }
    assert!(wound_up, "it warns before it strikes");
    assert!(hit, "the walker never hit");
    assert!(sim.player().hitlag > 0);
    sim.tick(Input::default());
    assert_eq!(sim.state.run.combo, 0, "being hit breaks the combo");
    assert_eq!(sim.state.run.best_combo, 7);
}

#[test]
fn a_shield_blocks_a_mob() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(8), Fx::ZERO));
    let shield = sim.player().shield_hp;
    let mut blocked = false;
    for _ in 0..240 {
        sim.tick(inp(0, 0, SHIELD));
        if sim.state.run.mobs[n].struck {
            blocked = true;
            break;
        }
    }
    assert!(blocked);
    assert_eq!(sim.player().percent, Fx::ZERO);
    assert!(sim.player().shield_hp < shield);
}

#[test]
fn a_flyer_dives_at_the_player() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    // (Clear of the start room's awnings.)
    sim.stand(Vec2::new(fx(16), Fx::ZERO));
    let p = sim.player().pos;
    let n = sim.add_mob(1, Vec2::new(p.x + fx(10), fx(9)));
    let mut dived = false;
    for _ in 0..400 {
        sim.tick(Input::default());
        dived |= sim.state.run.mobs[n].mode == MODE_ATTACK;
        if sim.player().percent > Fx::ZERO {
            break;
        }
    }
    assert!(dived);
    assert!(sim.player().percent > Fx::ZERO, "the dive never hit");
}

/// One swing hits a mob once per hit group, even though the attacker's hitlag pauses the move (the move is the same swing after it).
#[test]
fn one_swing_hits_a_mob_once() {
    let mut rc = quiet();
    rc.mobs[0].weight = fx(400);
    let mut sim = RunSim::with(rc, 1);
    sim.clear_mobs();
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
    sim.state.run.mobs[n].stamina = fx(500);
    for frame in 0..60 {
        sim.tick(inp(
            if frame < 2 { 60 } else { 0 },
            0,
            if frame == 0 { ATTACK } else { 0 },
        ));
        let m = &mut sim.state.run.mobs[n];
        m.pos.x = x + fx(1);
        m.kb_vel = Vec2::ZERO;
        m.mode = run::MODE_COOLDOWN;
        m.timer = 999;
    }
    let groups = {
        let w = &sim.world.weapons[usize::from(sim.world.fighters[0].weapon)];
        let mv = w.get(sim.player().move_id);
        mv.hitboxes
            .iter()
            .map(|h| h.group)
            .max()
            .map_or(1, |g| u16::from(g) + 1)
    };
    assert!(sim.state.run.combo >= 1, "the swing missed");
    assert!(sim.state.run.combo <= groups);
}

// ---- Coins, the combo and distance ---------------------------------------------------------------------------------------------

#[test]
fn the_combo_multiplies_the_coins() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    let rules = sim.rc.rules;
    let check = |combo: u16, want: u32| {
        let mut run = sim_core::run::Run::INACTIVE;
        run.combo = combo;
        assert_eq!(run.multiplier(&rules), want, "combo {combo}");
    };
    check(0, 1);
    check(4, 1);
    check(5, 2);
    check(14, 3);
    check(1000, u32::from(rules.combo_max));
    let a = sim.add_mob(0, Vec2::new(fx(20), sim.rc.rules.blast_top + fx(1)));
    sim.state.run.mobs[a].credit = 10;
    sim.state.run.combo = 9;
    sim.tick(Input::default());
    assert_eq!(sim.state.run.coins, 15);
    assert_eq!(sim.state.run.combo, 10);
}

#[test]
fn distance_counts_the_furthest_point_reached() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    assert_eq!(sim.state.run.distance(), 0);
    for _ in 0..60 {
        sim.tick(inp(127, 0, 0));
    }
    let d = sim.state.run.distance();
    assert!(d > 5);
    for _ in 0..30 {
        sim.tick(inp(-127, 0, 0));
    }
    assert!(
        sim.state.run.distance() >= d,
        "going back does not count down"
    );
}

/// Ground that carries on across a room seam is one floor: a fighter dashes across without leaving the ground.
#[test]
fn ground_joins_across_a_room_seam() {
    let mut sim = RunSim::with(only(&["shop"], (1, 0, 0)), 3);
    let seam = sim.state.run.slot_of(0).unwrap().end;
    let floor = sim.player().platform;
    let mut crossed = false;
    for frame in 0..200 {
        sim.tick(inp(127, 0, 0));
        let f = sim.player();
        assert!(
            f.grounded(),
            "frame {frame}: left the ground at {:?}",
            f.pos.x
        );
        assert_eq!(f.platform, floor, "the floor is one platform");
        if f.pos.x > seam + fx(2) {
            crossed = true;
            break;
        }
    }
    assert!(crossed);
}

/// The ledges and platforms of a room laid later work as any stage's: a fighter catches the gap's ledge and lands on its stone.
#[test]
fn ledges_and_platforms_work_in_a_later_room() {
    let mut rc = only(&["gap", "shop"], (1, 0, 0));
    for r in rc.rooms.iter_mut() {
        r.mirror = false;
    }
    let mut sim = RunSim::with(rc, 3);
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(sim.rc.rooms[usize::from(slot.room)].name, "gap");
    let ledge_x = slot.x + fx(10);
    let ledge = sim
        .world
        .stage
        .ledges
        .iter()
        .position(|l| l.x == ledge_x && l.side == 1)
        .expect("the gap's ledge is in the stage");
    let p = sim.world.fighters[0];
    sim.place(Vec2::new(
        ledge_x + p.ecb_half_width + Fx::from_ratio(1, 2),
        -p.ledge_min_drop - Fx::ONE,
    ));
    sim.state.fighters[PLAYER].facing = -1;
    let mut caught = false;
    for _ in 0..30 {
        sim.tick(Input::default());
        if sim.player().state == S::LedgeHang {
            caught = true;
            break;
        }
    }
    assert!(caught, "the ledge was not caught: {:?}", sim.player().state);
    assert_eq!(sim.player().ledge, ledge as i8);
    sim.place(Vec2::new(slot.x + fx(13), fx(8)));
    sim.clear_mobs();
    for _ in 0..90 {
        sim.tick(Input::default());
    }
    let f = sim.player();
    assert!(f.grounded());
    let stone = sim.world.stage.platforms[f.platform as usize];
    assert!(stone.pass_through && stone.y == Fx::from_ratio(36, 10));
}

// ---- Determinism ---------------------------------------------------------------------------------------------------------------

fn random_run(seed: u64, inputs: &[[Input; MAX_FIGHTERS]]) -> (GameState, Vec<u64>) {
    let mut sim = RunSim::new(seed);
    let mut sums = vec![];
    for i in inputs {
        step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
        sums.push(sim.state.checksum());
    }
    (sim.state, sums)
}

/// Random play mostly running right (so rooms come, arenas lock and shops are entered by chance).
fn rightward_inputs(seed: u64, frames: usize) -> Vec<[Input; MAX_FIGHTERS]> {
    let mut inputs = random_inputs(&mut Rng::new(seed), frames);
    for (n, i) in inputs.iter_mut().enumerate() {
        if n % 3 != 0 {
            i[0].stick_x = 127;
        }
    }
    inputs
}

#[test]
fn same_seed_and_inputs_give_the_same_run() {
    let inputs = rightward_inputs(5, 3000);
    let (a, sa) = random_run(9, &inputs);
    let (_, sb) = random_run(9, &inputs);
    assert_eq!(sa, sb);
    assert!(a.run.cam_x > fx(20), "it moved on");
    assert!(a.run.room > 0, "rooms were entered");
    let (_, sc) = random_run(10, &inputs);
    assert_ne!(sa.last(), sc.last(), "another seed is another run");
}

/// A run with shopping replays: the shop is in the state, driven by the inputs.
#[test]
fn a_run_with_shopping_replays() {
    let play = || {
        let mut rc = only(&["shop"], (1, 0, 0));
        rc.rules.shop_min = 1;
        rc.rules.shop_max = 1;
        let mut sim = RunSim::with(rc, 6);
        sim.state.run.coins = 400;
        sim.state.fighters[PLAYER].percent = fx(40);
        let (_, door) = shop_door(&sim);
        sim.stand(door);
        let mut sums = vec![];
        // In at the door, run right to the wares, press up a few times, run back to the door and leave.
        let script: Vec<Input> = (0..320)
            .map(|n| match n {
                50 | 120 | 145 | 230 => inp(0, 127, 0),
                60..=84 => inp(127, 0, 0),
                160..=187 => inp(-127, 0, 0),
                _ => Input::default(),
            })
            .collect();
        let mut opened = false;
        for i in script {
            sim.tick(i);
            opened |= sim.state.run.inside;
            sums.push(sim.state.checksum());
        }
        assert!(!sim.state.run.inside, "and came back out");
        (sums, opened, sim.state.run.coins)
    };
    let (a, opened, coins) = play();
    assert!(opened, "the script went into the shop");
    assert!(coins < 400, "and bought something");
    assert_eq!(a, play().0);
}

/// A saved state played on from (a rollback, a replay seek) gives the same run: the stage is rebuilt from the state.
#[test]
fn restoring_a_state_replays_the_same() {
    let inputs = rightward_inputs(8, 1600);
    let mut sim = RunSim::new(4);
    let mut saved = None;
    for (n, i) in inputs.iter().enumerate() {
        if n == 700 {
            saved = Some(sim.state);
        }
        step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
    }
    let straight = sim.state.checksum();
    let mut world = Content::placeholder();
    let mut state = saved.unwrap();
    for i in &inputs[700..] {
        step_run(&mut state, &mut world, &sim.rc, i);
    }
    assert_eq!(state.checksum(), straight);
}

/// Random play with a full pool of mobs: nothing panics, mobs stay inside the walls, and the fighter is never inside a block nor
/// behind the wall.
#[test]
fn random_play_with_many_mobs_keeps_invariants() {
    for seed in 0..8u64 {
        let inputs = rightward_inputs(seed, 1500);
        let mut sim = RunSim::new(seed);
        for (n, i) in inputs.iter().enumerate() {
            if n % 30 == 0 && !sim.state.run.inside {
                let p = sim.player().pos;
                for k in 0..MAX_MOBS {
                    if !sim.state.run.mobs[k].active {
                        let kind = (k % 2) as u8;
                        let at = Vec2::new(
                            p.x + fx((k as i32 % 7) - 3),
                            p.y + fx(if kind == 1 { 6 } else { 1 }),
                        );
                        sim.add_mob(kind, at);
                    }
                }
            }
            step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
            let run = &sim.state.run;
            for m in run.mobs.iter().filter(|m| m.active && m.mode != MODE_BURST) {
                assert!(usize::from(m.kind) < sim.rc.mobs.len());
                assert!(m.pos.x >= run.wall_x, "seed {seed}: a mob behind the wall");
                if let Ok(p) = usize::try_from(m.platform) {
                    assert!(
                        p >= run::POOL_PLATFORMS || run.platforms[p].live,
                        "a mob on a freed platform"
                    );
                }
            }
            let f = sim.player();
            if f.active {
                assert!(f.pos.x > run.wall_x, "seed {seed}: behind the wall");
                if let Ok(p) = usize::try_from(f.platform) {
                    assert!(run.platforms[p].live, "seed {seed}: on a freed platform");
                }
                for b in sim.world.stage.platforms.iter().filter(|b| !b.pass_through) {
                    let inside = f.pos.x > b.left
                        && f.pos.x < b.right
                        && f.pos.y > b.bottom
                        && f.pos.y < b.top_at(f.pos.x);
                    assert!(!inside, "seed {seed}: inside a block at {:?}", f.pos);
                }
            }
        }
    }
}

// ---- The checksum sees everything -------------------------------------------------------------------------------------------------

#[test]
fn checksum_sees_every_run_field() {
    let sim = RunSim::new(3);
    let base = sim.state;
    let c = base.checksum();
    type Edit = fn(&mut run::Run);
    let edits: Vec<(&str, Edit)> = vec![
        ("active", |r| r.active = false),
        ("over", |r| r.over = true),
        ("rng", |r| r.rng = Rng::new(77)),
        ("cam_x", |r| r.cam_x += Fx::from_raw(1)),
        ("floor_x", |r| r.floor_x += Fx::from_raw(1)),
        ("start_x", |r| r.start_x += Fx::from_raw(1)),
        ("best_x", |r| r.best_x += Fx::from_raw(1)),
        ("next_x", |r| r.next_x += Fx::from_raw(1)),
        ("next_y", |r| r.next_y += Fx::from_raw(1)),
        ("placed", |r| r.placed += 1),
        ("last_room", |r| r.last_room ^= 1),
        ("last_kind", |r| r.last_kind ^= 1),
        ("kinds_log", |r| r.kinds_log ^= 1 << 60),
        ("since_shop", |r| r.since_shop += 1),
        ("shop_due", |r| r.shop_due += 1),
        ("room", |r| r.room += 1),
        ("wall_x", |r| r.wall_x += Fx::from_raw(1)),
        ("gate_closed", |r| r.gate_closed ^= true),
        ("gate_x", |r| r.gate_x += Fx::from_raw(1)),
        ("cleared", |r| r.cleared += 1),
        ("slots.live", |r| r.slots[RING - 1].live ^= true),
        ("slots.room", |r| r.slots[0].room ^= 1),
        ("slots.seq", |r| r.slots[0].seq += 7),
        ("slots.x", |r| r.slots[0].x += Fx::from_raw(1)),
        ("slots.end", |r| r.slots[0].end += Fx::from_raw(1)),
        ("slots.spawned", |r| r.slots[0].spawned ^= 1 << 7),
        ("slots.y", |r| r.slots[0].y += Fx::from_raw(1)),
        ("slots.y_end", |r| r.slots[0].y_end += Fx::from_raw(1)),
        ("slots.mirror", |r| r.slots[0].mirror ^= true),
        ("slots.parts", |r| r.slots[0].parts ^= 1 << 3),
        ("slots.pieces", |r| r.slots[0].pieces[2] ^= 1),
        ("slots.coins", |r| r.slots[0].coins ^= 1 << 31),
        ("platforms.live", |r| {
            r.platforms[run::POOL_PLATFORMS - 1].live ^= true
        }),
        ("platforms.left", |r| r.platforms[0].left += Fx::from_raw(1)),
        ("platforms.right", |r| {
            r.platforms[0].right += Fx::from_raw(1)
        }),
        ("platforms.y", |r| r.platforms[0].y += Fx::from_raw(1)),
        ("platforms.bottom", |r| {
            r.platforms[0].bottom += Fx::from_raw(1)
        }),
        ("platforms.pass_through", |r| {
            r.platforms[0].pass_through ^= true
        }),
        ("platforms.rise", |r| r.platforms[0].rise += Fx::from_raw(1)),
        ("ledges.live", |r| {
            r.ledges[run::POOL_LEDGES - 1].live ^= true
        }),
        ("ledges.x", |r| r.ledges[0].x += Fx::from_raw(1)),
        ("ledges.y", |r| r.ledges[0].y += Fx::from_raw(1)),
        ("ledges.side", |r| r.ledges[0].side = 1 - r.ledges[0].side),
        ("layout", |r| r.layout += 1),
        ("coins", |r| r.coins += 1),
        ("coins_picked", |r| r.coins_picked += 1),
        ("kills", |r| r.kills += 1),
        ("combo", |r| r.combo += 1),
        ("best_combo", |r| r.best_combo += 1),
        ("last_percent", |r| r.last_percent += Fx::from_raw(1)),
        ("last_stocks", |r| r.last_stocks += 1),
        ("inside", |r| r.inside ^= true),
        ("shop_room", |r| r.shop_room += 1),
        ("shop_bought", |r| r.shop_bought[3] += 1),
        ("shop_note", |r| r.shop_note += 1),
        ("shop_acts", |r| r.shop_acts += 1),
        ("keeper_mood", |r| r.keeper_mood += 1),
        ("keeper_timer", |r| r.keeper_timer += 1),
        ("keeper_look", |r| r.keeper_look += 1),
        ("keeper_bonks", |r| r.keeper_bonks += 1),
        ("prev_stick_y", |r| r.prev_stick_y += 1),
        ("mob.active", |r| r.mobs[MAX_MOBS - 1].active ^= true),
        ("mob.kind", |r| r.mobs[0].kind ^= 1),
        ("mob.mode", |r| r.mobs[0].mode ^= 1),
        ("mob.timer", |r| r.mobs[0].timer += 1),
        ("mob.pos", |r| r.mobs[0].pos.y += Fx::from_raw(1)),
        ("mob.vel", |r| r.mobs[0].vel.x += Fx::from_raw(1)),
        ("mob.kb_vel", |r| r.mobs[0].kb_vel.y += Fx::from_raw(1)),
        ("mob.facing", |r| r.mobs[0].facing = 2),
        ("mob.platform", |r| r.mobs[0].platform = 5),
        ("mob.percent", |r| r.mobs[0].percent += Fx::from_raw(1)),
        ("mob.stamina", |r| r.mobs[0].stamina += Fx::from_raw(1)),
        ("mob.hitlag", |r| r.mobs[0].hitlag += 1),
        ("mob.hitstun", |r| r.mobs[0].hitstun += 1),
        ("mob.hit_stamp", |r| r.mobs[0].hit_stamp ^= 1),
        ("mob.hit_groups", |r| r.mobs[0].hit_groups ^= 1),
        ("mob.hit_age", |r| r.mobs[0].hit_age += 1),
        ("mob.shot_mask", |r| r.mobs[0].shot_mask ^= 1 << 15),
        ("mob.struck", |r| r.mobs[0].struck ^= true),
        ("mob.credit", |r| r.mobs[0].credit += 1),
        ("mob.target", |r| r.mobs[0].target.x += Fx::from_raw(1)),
        ("mob.room", |r| r.mobs[0].room += 1),
    ];
    for (name, edit) in edits {
        let mut s = base;
        edit(&mut s.run);
        assert_ne!(s.run, base.run, "{name}: the edit changed nothing");
        assert_ne!(s.checksum(), c, "{name} is not in the checksum");
    }
}

/// Versus is untouched: a match never has an active run, and a run's content is hashed with its own numbers (names are labels).
#[test]
fn versus_has_no_run_and_run_content_hashes() {
    let content = Content::placeholder();
    let s = GameState::new(&content, 1, [0, 1, 0, 1]);
    assert!(!s.run.active);
    let rc = RunContent::placeholder();
    let changed: Vec<(&str, RunContent)> = vec![
        ("rules", {
            let mut o = rc.clone();
            o.rules.rooms_behind = 1;
            o
        }),
        ("room kind", {
            let mut o = rc.clone();
            o.rooms[1].kind = ROOM_REST;
            o
        }),
        ("door", {
            let mut o = rc.clone();
            let shop = o.rooms.iter().position(|r| r.kind == ROOM_SHOP).unwrap();
            o.rooms[shop].door.x += Fx::ONE;
            o
        }),
        ("price", {
            let mut o = rc.clone();
            o.items[0].price += 1;
            o
        }),
        ("mob", {
            let mut o = rc.clone();
            o.mobs[0] = MobKind {
                coins: 6,
                ..o.mobs[0]
            };
            o
        }),
    ];
    for (what, o) in changed {
        assert_ne!(rc.hash(), o.hash(), "{what}");
    }
    let mut renamed = rc.clone();
    renamed.rooms[0].name = "elsewhere".to_string();
    renamed.items[0].name = "Soup".to_string();
    assert_eq!(rc.hash(), renamed.hash(), "names are labels only");
}
