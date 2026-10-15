//! The single-player run (R1, sim 120; rooms R1b, sim 122): the room window (dropping old rooms, the left wall, no backtracking past
//! it), the camera, arenas, the shop, rest stops and the generator's rules; mobs, coins and the combo; and determinism.

mod common;

use common::inp;
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::run::data::{MobKind, RunContent, SPOT_ANY};
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
            shield: k.shield,
            room: self.state.run.room,
            ..Mob::default()
        };
        slot
    }

    /// Walks (teleports) the player into the next room, clearing an arena it locks; returns the room's type.
    fn next_room(&mut self) -> u8 {
        let (ahead, n) = self.state.run.ahead();
        assert!(n >= 1, "the room ahead is laid");
        // (At a branching fork, the high road's alley: it is laid first, and the player comes in from above.)
        let slot = ahead[0];
        let target = slot.seq;
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

/// The built-in rooms with no mobs in the streets, no arenas (so walking on never locks), no rare events and no props.
fn quiet() -> RunContent {
    let mut rc = RunContent::placeholder();
    for r in rc.rooms.iter_mut() {
        if r.kind == ROOM_STREET {
            r.spawns.clear();
        }
        r.props.clear();
    }
    rc.rules.arena_weight = 0;
    rc.rules.event_base = 0;
    rc.rules.event_step = 0;
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
            live.sort_by_key(|s| (s.x, s.seq));
            assert!(live.len() <= RING);
            // The current room, at most `rooms_behind` behind, and `rooms_ahead` ahead (both roads of a branching fork: two).
            assert!(
                live.iter().any(|s| s.seq == run.room),
                "seed {seed}: current"
            );
            let behind = live.iter().filter(|s| s.seq < run.room).count();
            let ahead = live.iter().filter(|s| s.seq > run.room).count();
            assert!(behind <= usize::from(r.rooms_behind), "seed {seed}: behind");
            let want = if run.branches().is_some() {
                2
            } else {
                usize::from(r.rooms_ahead)
            };
            assert_eq!(ahead, want, "seed {seed}: the window ahead");
            if run.room >= 3 {
                assert!(behind >= 1, "seed {seed}: rooms kept behind");
            }
            for w in live.windows(2) {
                assert!(
                    w[0].end == w[1].x || w[0].x == w[1].x,
                    "seed {seed}: rooms do not join"
                );
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
    // (The spots its budget filled.)
    let mobs = slot.kinds.iter().filter(|&&k| k != SPOT_ANY).count();
    assert!(mobs >= 1);
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
fn hitting_the_shopkeeper_makes_it_dizzy_then_sulk() {
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
    // R2b (owner's R2a answer): then it sulks, arms crossed, and sells nothing.
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_HMPH, "it sulks");
    sim.state.run.coins = 500;
    sim.state.fighters[PLAYER].percent = fx(50);
    sim.state.run.last_percent = fx(50);
    stand_inside(&mut sim, shop.pedestals[0].x);
    press_up(&mut sim);
    assert_eq!(
        sim.state.run.shop_note,
        run::NOTE_REFUSED,
        "it will not sell"
    );
    assert_eq!(sim.state.run.coins, 500);
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_HMPH);
    sim.idle(u32::from(shop.sulk_frames));
    assert_eq!(
        sim.state.run.keeper_mood,
        run::KEEPER_IDLE,
        "it gets over it"
    );
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, run::NOTE_BOUGHT, "and sells again");
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
    let mut seen = [0u32; 6];
    for seed in 0..12u64 {
        let (all_kinds, all_rooms, r) = kinds_of(seed);
        // (A branching fork's roads are laid as its pair, outside the generator's count: only the rooms it drew count here.)
        let rc = RunContent::placeholder();
        let drawn: Vec<usize> = (0..all_rooms.len())
            .filter(|&i| !rc.rooms[all_rooms[i]].paired)
            .collect();
        let kinds: Vec<u8> = drawn.iter().map(|&i| all_kinds[i]).collect();
        let rooms: Vec<usize> = drawn.iter().map(|&i| all_rooms[i]).collect();
        for &k in &all_kinds {
            seen[usize::from(k)] += 1;
        }
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
        if sim.state.run.mobs[n].struck != 0 {
            blocked = true;
            break;
        }
    }
    assert!(blocked);
    assert_eq!(sim.player().percent, Fx::ZERO);
    assert!(sim.player().shield_hp < shield);
}

/// A leaper on high ground tells (crouching), leaps at a player below and strikes with an aerial (R2b: the diver's anti-air lesson).
#[test]
fn a_leaper_leaps_from_high_ground_with_an_aerial() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    sim.stand(Vec2::new(fx(16), Fx::ZERO));
    let p = sim.player().pos;
    // On the start room's second awning (3.6 up), 6 to the player's right.
    let n = sim.add_mob(
        run::data::KIND_LEAPER,
        Vec2::new(fx(22), Fx::from_ratio(36, 10)),
    );
    assert_ne!(
        sim.state.run.mobs[n].platform, NONE,
        "it stands on the awning"
    );
    let (mut told, mut leapt, mut aerial) = (false, false, false);
    for _ in 0..400 {
        sim.tick(Input::default());
        let m = sim.state.run.mobs[n];
        told |= m.mode == MODE_WINDUP;
        leapt |= m.mode == run::MODE_LEAP;
        aerial |= m.mode == MODE_ATTACK && m.platform == NONE;
        if sim.player().percent > Fx::ZERO {
            break;
        }
        // (The player holds still.)
        sim.state.fighters[PLAYER].pos.x = p.x;
    }
    assert!(
        told && leapt && aerial,
        "told {told}, leapt {leapt}, aerial {aerial}"
    );
    assert!(sim.player().percent > Fx::ZERO, "the leap never hit");
}

/// One swing hits a mob once per hit group, even though the attacker's hitlag pauses the move (the move is the same swing after it).
#[test]
fn one_swing_hits_a_mob_once() {
    let mut sim = RunSim::with(quiet(), 1);
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
fn random_play_with_many_grunts_keeps_invariants() {
    for seed in 0..8u64 {
        let inputs = rightward_inputs(seed, 1500);
        let mut sim = RunSim::new(seed);
        for (n, i) in inputs.iter().enumerate() {
            if n % 30 == 0 && !sim.state.run.inside {
                let p = sim.player().pos;
                for k in 0..MAX_MOBS {
                    if !sim.state.run.mobs[k].active {
                        let kind = (k % 6) as u8;
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
        ("mob.struck", |r| r.mobs[0].struck ^= 1 << 7),
        ("mob.credit", |r| r.mobs[0].credit += 1),
        ("mob.target", |r| r.mobs[0].target.x += Fx::from_raw(1)),
        ("mob.room", |r| r.mobs[0].room += 1),
        // R2b.
        ("mob.move_slot", |r| r.mobs[0].move_slot += 1),
        ("mob.frame", |r| r.mobs[0].frame += 1),
        ("mob.shield", |r| r.mobs[0].shield += Fx::from_raw(1)),
        ("mob.aux", |r| r.mobs[0].aux += 1),
        ("mob.tier", |r| r.mobs[0].tier += 1),
        ("slots.kinds", |r| r.slots[0].kinds[7] ^= 1),
        ("slots.tier", |r| r.slots[0].tier += 1),
        ("platforms.seq", |r| r.platforms[0].seq += 1),
        ("ledges.seq", |r| r.ledges[0].seq += 1),
        ("keeper_sulk", |r| r.keeper_sulk += 1),
        ("props.active", |r| {
            r.props[run::MAX_PROPS - 1].active ^= true
        }),
        ("props.kind", |r| r.props[0].kind ^= 1),
        ("props.room", |r| r.props[0].room += 1),
        ("props.mode", |r| r.props[0].mode += 1),
        ("props.timer", |r| r.props[0].timer += 1),
        ("props.pos", |r| r.props[0].pos.x += Fx::from_raw(1)),
        ("props.vel", |r| r.props[0].vel.y += Fx::from_raw(1)),
        ("props.hit_mobs", |r| r.props[0].hit_mobs ^= 1 << 15),
        ("props.hit_player", |r| r.props[0].hit_player ^= true),
        ("chests", |r| r.chests += 1),
        ("event_pity", |r| r.event_pity += 1),
        ("event_city", |r| r.event_city += 1),
        ("van.mode", |r| r.van.mode += 1),
        ("van.room", |r| r.van.room += 1),
        ("van.x", |r| r.van.x += Fx::from_raw(1)),
        ("van.y", |r| r.van.y += Fx::from_raw(1)),
        ("van.speed", |r| r.van.speed += Fx::from_raw(1)),
        ("van.timer", |r| r.van.timer += 1),
        ("van.platform", |r| r.van.platform += 3),
        ("van.riding", |r| r.van.riding ^= true),
        ("van.hits", |r| r.van.hits += 1),
        ("skipped_log", |r| r.skipped_log ^= 1 << 15),
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
        ("piece", {
            let mut o = rc.clone();
            o.pieces[0].platforms[0].platform.y += Fx::ONE;
            o
        }),
        ("part chance", {
            let mut o = rc.clone();
            o.rooms[1].parts[0] += 1;
            o
        }),
        ("exit height", {
            let mut o = rc.clone();
            o.rooms[1].exit_y += Fx::ONE;
            o
        }),
        ("interior", {
            let mut o = rc.clone();
            o.interior.pedestals[0].x += Fx::ONE;
            o
        }),
        ("keeper", {
            let mut o = rc.clone();
            o.interior.dizzy_frames += 1;
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
    renamed.pieces[0].name = "skip".to_string();
    renamed.rooms[1].platforms[0].look = 9;
    assert_eq!(rc.hash(), renamed.hash(), "names are labels only");
}

// ---- R2a: the world (sim 124) -------------------------------------------------------------------------------------------------

/// A room of the content by its name.
fn room_named(rc: &RunContent, name: &str) -> usize {
    rc.rooms.iter().position(|r| r.name == name).unwrap()
}

/// A slot laying room `index` at the origin as this variant.
fn variant(
    rc: &RunContent,
    index: usize,
    mirror: bool,
    parts: u8,
    pieces: [u8; 3],
) -> run::RoomSlot {
    let room = &rc.rooms[index];
    run::RoomSlot {
        live: true,
        room: index as u8,
        seq: 0,
        x: Fx::ZERO,
        end: room.width,
        y: Fx::ZERO,
        y_end: run::room_exit(room, mirror),
        mirror,
        parts,
        pieces,
        spawned: 0,
        coins: 0,
        ..run::RoomSlot::default()
    }
}

fn footings(rc: &RunContent, slot: &run::RoomSlot) -> Vec<sim_core::content::Platform> {
    let mut out = Vec::new();
    run::for_each_footing(rc, slot, |p| out.push(p.platform));
    out
}

/// Over many runs every room is laid as a variant: mirrored and not, optional parts on and off, every piece in some slot, the ground
/// joining room to room at one height and drifting no further than the rules allow; and the pools hold exactly what the variants lay.
#[test]
fn rooms_are_laid_as_variants() {
    let rc = RunContent::placeholder();
    let mut mirrored = [false; 2];
    let mut parts_seen = [0u8; 2];
    let mut pieces_seen = vec![false; rc.pieces.len()];
    let mut heights = std::collections::BTreeSet::new();
    for seed in 0..24u64 {
        let mut sim = RunSim::with(quiet(), seed);
        for _ in 0..40 {
            sim.next_room();
            let run = &sim.state.run;
            let mut live: Vec<_> = run.slots.iter().filter(|s| s.live).copied().collect();
            live.sort_by_key(|s| s.seq);
            for s in &live {
                // (Its ground joins the room before it; a branching fork's alley joins its high road, the fork's lift up.)
                let Some(before) = live.iter().find(|p| p.end == s.x && p.x != s.x) else {
                    continue;
                };
                let lift = if sim.rc.rooms[usize::from(s.room)].kind == run::ROOM_ALLEY {
                    sim.rc.rooms[usize::from(before.room)].lift
                } else {
                    Fx::ZERO
                };
                assert_eq!(
                    before.y_end + lift,
                    s.y,
                    "seed {seed}: the ground joins at one height"
                );
            }
            for s in &live {
                let room = &sim.rc.rooms[usize::from(s.room)];
                assert_eq!(s.y_end - s.y, run::room_exit(room, s.mirror));
                let drift = sim.rc.rules.drift;
                let alley = room.kind == run::ROOM_ALLEY;
                assert!(
                    (alley || s.y.abs() <= drift) && s.y_end.abs() <= drift,
                    "seed {seed}: drifted too far"
                );
                assert!(
                    room.mirror || !s.mirror,
                    "only mirrorable rooms are mirrored"
                );
                heights.insert(s.y_end.raw());
                mirrored[usize::from(s.mirror)] = true;
                if room.name == "street" {
                    parts_seen[usize::from(s.parts & 1)] += 1;
                }
                for p in s.pieces.iter().filter(|&&p| p > 0) {
                    pieces_seen[usize::from(*p) - 1] = true;
                }
            }
            // Every laid footing is in the pool (ground joined across seams is one longer entry, trimmed at the wall).
            for s in &live {
                for p in footings(&sim.rc, s) {
                    let found = run.platforms.iter().any(|e| {
                        e.live
                            && e.pass_through == p.pass_through
                            && e.rise == p.rise
                            && e.left <= p.left.max(run.wall_x)
                            && e.right >= p.right
                            && (e.y == p.y || e.left == run.wall_x)
                    });
                    let name = &sim.rc.rooms[usize::from(s.room)].name;
                    assert!(found, "seed {seed}: a footing of {name} is not in the pool");
                }
            }
        }
    }
    assert_eq!(mirrored, [true, true], "mirrored and not");
    assert!(
        parts_seen[0] > 0 && parts_seen[1] > 0,
        "a part on and off: {parts_seen:?}"
    );
    assert!(
        pieces_seen.iter().all(|&p| p),
        "every piece came: {pieces_seen:?}"
    );
    assert!(heights.len() >= 4, "the ground rose and fell: {heights:?}");
}

/// A mirrored room is its reflection: a rising ramp falls, its platforms' ends swap, its slope turns, its ledges hang the other
/// way, and a shop's door stays at the same distance from the other edge.
#[test]
fn a_mirrored_room_is_its_reflection() {
    let rc = RunContent::placeholder();
    let ramp = room_named(&rc, "ramp");
    let w = rc.rooms[ramp].width;
    let plain = footings(&rc, &variant(&rc, ramp, false, 0, [0; 3]));
    let flipped = footings(&rc, &variant(&rc, ramp, true, 0, [0; 3]));
    assert_eq!(plain.len(), flipped.len());
    let exit = rc.rooms[ramp].exit_y;
    for (a, b) in plain.iter().zip(&flipped) {
        assert_eq!((b.left, b.right), (w - a.right, w - a.left));
        assert_eq!(b.rise, -a.rise, "the slope turns");
        assert_eq!(b.y, a.top_right() - exit, "heights from the new entry");
    }
    let slope = flipped.iter().find(|p| p.sloped()).unwrap();
    assert!(slope.rise < Fx::ZERO, "mirrored, the ramp falls");
    // Ledges.
    let gap = room_named(&rc, "gap");
    let mut sides = [Vec::new(), Vec::new()];
    for (k, m) in [false, true].into_iter().enumerate() {
        run::for_each_ledge(&rc, &variant(&rc, gap, m, 0, [0; 3]), |l| {
            sides[k].push((l.x, l.side))
        });
    }
    let gw = rc.rooms[gap].width;
    assert_eq!(sides[0].len(), 2);
    for ((x, s), (mx, ms)) in sides[0].iter().zip(&sides[1]) {
        assert_eq!((*mx, *ms), (gw - *x, -*s));
    }
    // The shop's door.
    let shop = room_named(&rc, "shop");
    let d0 = run::door_of(&rc, &variant(&rc, shop, false, 0, [0; 3])).unwrap();
    let d1 = run::door_of(&rc, &variant(&rc, shop, true, 0, [0; 3])).unwrap();
    assert_eq!(d1.x, rc.rooms[shop].width - d0.x);
}

/// Optional parts swap what is there: the drop's "no gap" part lays one long roof and no ledges; without it, the gap, its balcony and
/// both ledges. A piece in a slot lays its footings and coins on the slot's ground.
#[test]
fn optional_parts_and_pieces_lay_what_they_say() {
    let rc = RunContent::placeholder();
    let drop = room_named(&rc, "drop");
    let count_ledges = |parts| {
        let mut n = 0;
        run::for_each_ledge(&rc, &variant(&rc, drop, false, parts, [0; 3]), |_| n += 1);
        n
    };
    assert_eq!(count_ledges(0), 2, "the gap has its ledges");
    assert_eq!(count_ledges(1), 0, "no gap, no ledges");
    let width = rc.rooms[drop].width;
    let gapless = footings(&rc, &variant(&rc, drop, false, 1, [0; 3]));
    assert!(gapless
        .iter()
        .any(|p| !p.pass_through && p.left == Fx::ZERO && p.right == width));
    let gapped = footings(&rc, &variant(&rc, drop, false, 0, [0; 3]));
    assert!(gapped
        .iter()
        .all(|p| p.pass_through || p.left > Fx::ZERO || p.right < width));
    // A dumpster in the street's slot: its two blocks stand on the street at the slot, with its coin.
    let street = room_named(&rc, "street");
    let slot = rc.rooms[street].slots[0];
    let dumpster = rc.pieces.iter().position(|p| p.name == "dumpster").unwrap() as u8 + 1;
    let none = footings(&rc, &variant(&rc, street, false, 0, [0; 3]));
    let with = footings(&rc, &variant(&rc, street, false, 0, [dumpster, 0, 0]));
    assert_eq!(with.len(), none.len() + 2);
    let lid = with
        .iter()
        .find(|p| p.left == slot.x - Fx::from_ratio(12, 10))
        .expect("the dumpster");
    assert_eq!(lid.y, slot.y + Fx::from_ratio(12, 10));
    let mut coins = 0;
    run::for_each_coin(
        &rc,
        &variant(&rc, street, false, 0, [dumpster, 0, 0]),
        |_, _| coins += 1,
    );
    assert_eq!(coins, rc.rooms[street].coins.len() + 1, "and its coin");
}

/// The fork: one room, two roads. The high road (fire escape, catwalks and scaffolds) is reached up the stairs a coin trail climbs;
/// falling from it lands on the low road (the street, never a pit); the high road's end comes down to the street and both roads
/// lead into the next room. The map knows it is a fork.
#[test]
fn the_fork_has_two_roads_that_meet_again() {
    let mut rc = only(&["fork", "shop"], (1, 0, 0));
    rc.rules.fork_chance = 100;
    rc.rules.shop_min = 7;
    rc.rules.shop_max = 7;
    let fork = room_named(&rc, "fork");
    assert!(rc.rooms[fork].fork);
    let mut sim = RunSim::with(rc, 2);
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(usize::from(slot.room), fork);
    assert_eq!(sim.state.run.fork_of(1), Some(true), "the map's split icon");
    assert_eq!(sim.state.run.fork_of(0), Some(false));
    // The coin trail climbs the fire escape.
    let mut trail = Vec::new();
    run::for_each_coin(&sim.rc, &slot, |_, at| trail.push(at));
    assert!(
        trail
            .windows(2)
            .take(4)
            .all(|w| w[1].x > w[0].x && w[1].y > w[0].y),
        "{trail:?}"
    );
    // Falling from the high road's gap: onto the street, safe.
    sim.next_room();
    sim.clear_mobs();
    let stocks = sim.player().stocks;
    sim.place(Vec2::new(slot.x + Fx::from_ratio(285, 10), slot.y + fx(7)));
    for _ in 0..90 {
        sim.tick(Input::default());
        if sim.player().grounded() {
            break;
        }
    }
    assert!(sim.player().grounded(), "landed");
    assert_eq!(sim.player().pos.y, slot.y, "on the low road");
    assert_eq!(sim.player().stocks, stocks);
    // Along the high road's last catwalk and down its stairs: into the next room, no life lost.
    sim.place(Vec2::new(slot.x + fx(43), slot.y + fx(7)));
    for _ in 0..400 {
        sim.tick(inp(127, 0, 0));
        sim.clear_mobs();
        if sim.state.run.room == 2 {
            break;
        }
    }
    assert_eq!(sim.state.run.room, 2, "the roads meet in the next room");
    assert_eq!(sim.player().stocks, stocks);
}

/// The fork's two roads and the window: entering an arena after a fork shuts its gate as ever, and the window drops the fork
/// behind like any room.
#[test]
fn a_fork_works_with_the_window_and_the_gate() {
    let mut rc = only(&["fork", "yard", "shop"], (1, 1, 0));
    rc.rules.fork_chance = 100;
    rc.rules.arena_weight = 50;
    let mut sim = RunSim::with(rc, 5);
    let mut forks = 0;
    let mut gated_after_fork = false;
    for _ in 0..30 {
        let was_fork = sim.state.run.fork_of(sim.state.run.room).unwrap_or(false);
        let target = sim.state.run.room + 1;
        let slot = *sim.state.run.slot_of(target).unwrap();
        sim.place(Vec2::new(
            slot.x + sim.rc.rules.enter_depth + Fx::ONE,
            slot.y + fx(12),
        ));
        sim.state.fighters[PLAYER].invuln = 30;
        sim.tick(Input::default());
        assert_eq!(sim.state.run.room, target);
        forks += usize::from(sim.state.run.fork_of(target).unwrap_or(false));
        if sim.state.run.gate_closed {
            gated_after_fork |= was_fork;
            assert_eq!(sim.state.run.gate_x, slot.end);
            sim.clear_mobs();
            sim.tick(Input::default());
            assert!(!sim.state.run.gate_closed);
        }
        assert!(sim.state.run.slots.iter().filter(|s| s.live).count() <= RING);
    }
    assert!(forks > 5, "forks came: {forks}");
    assert!(gated_after_fork, "an arena came right after a fork");
}

/// Coins on the way are picked up by walking (or jumping) through them: each once, paying its value, not counting toward the
/// combo.
#[test]
fn coins_are_picked_up_once() {
    let mut sim = RunSim::with(only(&["street", "shop"], (1, 0, 0)), 3);
    sim.clear_mobs();
    let slot = *sim.state.run.slot_of(1).unwrap();
    let mut first = None;
    run::for_each_coin(&sim.rc, &slot, |n, at| {
        first.get_or_insert((n, at));
    });
    let (n, at) = first.expect("the street has coins");
    let before = sim.state.run.coins;
    // (The body's middle is a unit above the feet.)
    sim.place(at - Vec2::new(Fx::ZERO, Fx::ONE));
    sim.tick(Input::default());
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert!(slot.coins & (1 << n) != 0, "taken");
    let value = u32::from(sim.rc.rules.coin_value);
    assert_eq!(sim.state.run.coins, before + value);
    assert_eq!(sim.state.run.coins_picked, 1);
    assert_eq!(sim.state.run.combo, 0);
    sim.place(at - Vec2::new(Fx::ZERO, Fx::ONE));
    sim.tick(Input::default());
    assert_eq!(sim.state.run.coins, before + value, "only once");
}

/// A mob comes out of its door or stairwell before it acts: for `emerge_frames` it neither moves nor can be hit, then it wakes.
#[test]
fn mobs_come_out_of_their_doors() {
    let mut rc = only(&["street", "shop"], (1, 0, 0));
    let street = room_named(&rc, "street");
    rc.rooms[street].spawns = vec![run::MobSpawn {
        kind: 0,
        x: fx(20),
        y: Fx::ZERO,
        part: 0,
    }];
    let mut sim = RunSim::with(rc, 1);
    let mut n = None;
    for _ in 0..600 {
        sim.tick(inp(60, 0, 0));
        n = sim.state.run.mobs.iter().position(|m| m.active);
        if n.is_some() {
            break;
        }
    }
    let n = n.expect("the mob came");
    let m = sim.state.run.mobs[n];
    assert_eq!(m.mode, run::MODE_EMERGE);
    assert!(m.timer <= u16::from(sim.rc.rules.emerge_frames));
    let at = m.pos;
    for _ in 1..m.timer {
        sim.tick(Input::default());
        assert_eq!(sim.state.run.mobs[n].mode, run::MODE_EMERGE);
        assert_eq!(sim.state.run.mobs[n].pos, at, "it stands in its doorway");
    }
    sim.tick(Input::default());
    assert!(
        matches!(sim.state.run.mobs[n].mode, MODE_IDLE | MODE_MOVE),
        "awake"
    );
    // Emerging is off with the rule at 0: mobs pop in at once.
    let mut rc = quiet();
    rc.rules.emerge_frames = 0;
    let mut sim = RunSim::with(rc, 1);
    sim.next_room();
    assert!(sim
        .state
        .run
        .mobs
        .iter()
        .all(|m| m.mode != run::MODE_EMERGE));
}

/// A walker walks along a slope (its feet on it) up to the high ground.
#[test]
fn walkers_follow_slopes() {
    let rc = only(&["ramp", "shop"], (1, 0, 0));
    let mut sim = RunSim::with(rc, 4);
    sim.clear_mobs();
    let slot = *sim.state.run.slot_of(1).unwrap();
    // The ramp, wherever the room's mirroring put it: the walker stands on its middle and walks to the player on its high side.
    let ramp = footings(&sim.rc, &slot)
        .into_iter()
        .find(|p| p.sloped())
        .unwrap();
    let mid = (ramp.left + ramp.right) / fx(2);
    let high_x = if ramp.rise > Fx::ZERO {
        ramp.right + fx(4)
    } else {
        ramp.left - fx(4)
    };
    sim.stand(Vec2::new(high_x, ramp.top_max()));
    let n = sim.add_mob(0, Vec2::new(mid, ramp.top_at(mid)));
    assert_ne!(sim.state.run.mobs[n].platform, NONE, "on the ramp");
    let mut climbed = false;
    for _ in 0..240 {
        sim.tick(Input::default());
        let m = sim.state.run.mobs[n];
        if let Ok(i) = usize::try_from(m.platform) {
            let p = sim.world.stage.platforms[i];
            assert_eq!(m.pos.y, p.top_at(m.pos.x), "its feet on its ground");
            climbed |= m.pos.y == ramp.top_max();
        }
        if m.mode == MODE_WINDUP {
            break;
        }
    }
    assert!(climbed, "it walked up onto the high ground");
}

/// The run's state stays small enough to save and restore every frame for rollback.
#[test]
fn the_game_state_stays_small() {
    let size = std::mem::size_of::<GameState>();
    let run = std::mem::size_of::<run::Run>();
    println!("GameState: {size} bytes (the run: {run} bytes)");
    assert!(size <= 8 * 1024, "GameState is {size} bytes");
}

// ---- R2b: grunts, hazards, the curve, rare events, branching forks (sim 127) ----------------------------------------------------

use run::data::{
    KIND_BRAWLER, KIND_BRUISER, KIND_LOBBER, KIND_SHIELD, KIND_SPEAR, PLACE_ALONE, PLACE_FLAT,
    PLACE_HIGH, PROP_KIND_CHEST, PROP_KIND_GAS, PROP_KIND_SIGN,
};
use sim_core::input::buttons::{GRAB, SPECIAL};
use sim_core::run::ROOM_EVENT;

/// The player standing at x 16 of the start room (clear of its awnings), no grunts about.
fn open_street(rc: RunContent, char_id: u8) -> RunSim {
    let mut sim = RunSim::with_char(rc, 1, char_id);
    sim.clear_mobs();
    sim.tick(Input::default());
    sim.stand(Vec2::new(fx(16), Fx::ZERO));
    sim
}

/// A grunt of `kind` awake at `at`, facing the player, in tier `tier`.
fn grunt(sim: &mut RunSim, kind: u8, at: Vec2, tier: u8) -> usize {
    let n = sim.add_mob(kind, at);
    sim.state.run.mobs[n].tier = tier;
    n
}

/// A grunt's hurtbox is its body's, as tall as a fighter's (R1's mobs were circles 0.5 to 0.6 across).
#[test]
fn a_grunt_has_a_fighters_hurtbox() {
    let mut sim = open_street(quiet(), 0);
    for kind in 0..sim.rc.mobs.len() as u8 {
        sim.clear_mobs();
        let n = grunt(&mut sim, kind, Vec2::new(fx(20), Fx::ZERO), 0);
        let m = sim.state.run.mobs[n];
        let k = sim.rc.mobs[usize::from(kind)];
        let body = sim.world.fighters[usize::from(k.body)];
        // The circles a fighter of that body standing there would have.
        let mut f = sim_core::Fighter::spawn(m.pos, k.body, m.facing, m.platform, 0, Fx::ZERO);
        f.active = true;
        let want =
            sim_core::combat::hurtboxes(&f, &body, sim_core::combat::weapon_of(&sim.world, &body));
        let g = run::grunt_fighter(&sim.world, &sim.rc, &m);
        let got =
            sim_core::combat::hurtboxes(&g, &body, sim_core::combat::weapon_of(&sim.world, &body));
        assert_eq!(got, want, "kind {kind}: its body's hurtbox");
        let top = got.iter().map(|(c, r)| c.y + *r).max().unwrap();
        assert!(
            top - m.pos.y >= Fx::from_ratio(3, 2),
            "kind {kind}: fighter-tall ({:?})",
            top - m.pos.y
        );
        assert!(
            body.ecb_height > Fx::from_ratio(12, 10),
            "as tall as a fighter"
        );
    }
    // Every kind's moves are its body's own.
    for k in &sim.rc.mobs {
        let w = &sim.world.weapons[usize::from(sim.world.fighters[usize::from(k.body)].weapon)];
        for &id in k.moves.iter().filter(|&&id| id != run::data::NO_MOVE) {
            assert!(usize::from(id) < w.moves.len());
        }
    }
}

/// Each ground kind tells for its tier's frames (18 in tier 0, a flash and a "!"), then makes its body's move, and the move hits.
#[test]
fn each_grunt_tells_then_strikes_with_its_bodys_move() {
    for (kind, dx) in [
        (KIND_BRAWLER, 14),
        (KIND_SPEAR, 28),
        (KIND_SHIELD, 26),
        (KIND_BRUISER, 26),
    ] {
        let mut sim = open_street(quiet(), 0);
        let n = grunt(
            &mut sim,
            kind,
            Vec2::new(fx(16) + Fx::from_ratio(dx, 10), Fx::ZERO),
            0,
        );
        let k = sim.rc.mobs[usize::from(kind)];
        let mut told = 0;
        let mut struck_with = None;
        for _ in 0..240 {
            sim.tick(Input::default());
            sim.state.fighters[PLAYER].pos.x = fx(16);
            let m = sim.state.run.mobs[n];
            if m.mode == MODE_WINDUP {
                told += 1;
            }
            if m.mode == MODE_ATTACK && struck_with.is_none() {
                struck_with = Some(k.moves[usize::from(m.move_slot)]);
            }
            if sim.player().percent > Fx::ZERO {
                break;
            }
        }
        assert_eq!(told, 18, "kind {kind}: an 18-frame tell");
        let id = struck_with.expect("it attacked");
        assert!(k.moves.contains(&id), "kind {kind}: one of its kit's moves");
        assert!(sim.player().percent > Fx::ZERO, "kind {kind}: the move hit");
    }
}

/// The tell shortens with the tier, and a grunt's damage and stamina grow with it.
#[test]
fn tiers_shorten_tells_and_toughen_grunts() {
    let rc = RunContent::placeholder();
    assert_eq!(rc.tiers[0].tell, 18);
    assert!(rc.tiers.windows(2).all(|w| w[1].tell < w[0].tell
        && w[1].damage >= w[0].damage
        && w[1].stamina >= w[0].stamina
        && w[1].budget >= w[0].budget));
    let hit = |tier: u8| {
        let mut sim = open_street(quiet(), 0);
        let n = grunt(
            &mut sim,
            KIND_BRAWLER,
            Vec2::new(fx(16) + Fx::from_ratio(14, 10), Fx::ZERO),
            tier,
        );
        let mut told = 0;
        for _ in 0..240 {
            sim.tick(Input::default());
            sim.state.fighters[PLAYER].pos.x = fx(16);
            if sim.state.run.mobs[n].mode == MODE_WINDUP {
                told += 1;
            }
            if sim.player().percent > Fx::ZERO {
                break;
            }
        }
        (told, sim.player().percent)
    };
    let (tell0, dmg0) = hit(0);
    let last = (rc.tiers.len() - 1) as u8;
    let (tell4, dmg4) = hit(last);
    assert_eq!(tell0, 18);
    assert_eq!(tell4, i32::from(rc.tiers[usize::from(last)].tell));
    assert!(dmg4 > dmg0, "tier {last} hits harder: {dmg4:?} vs {dmg0:?}");
}

/// The tier steps up every `tier_rooms` rooms to the last tier; a room's pool is its tier's (rooms from tier 1 never come in tier 0),
/// and a grunt spawned in a room gets its tier (and its stamina).
#[test]
fn the_tier_steps_up_and_rooms_come_from_their_tiers_pool() {
    let rc = RunContent::placeholder();
    let step = u32::from(rc.rules.tier_rooms);
    assert_eq!(run::tier_of(&rc, 0), 0);
    assert_eq!(run::tier_of(&rc, step - 1), 0);
    assert_eq!(run::tier_of(&rc, step), 1);
    assert_eq!(run::tier_of(&rc, 1000), (rc.tiers.len() - 1) as u8);
    for seed in 0..10u64 {
        let mut sim = RunSim::new(seed);
        for _ in 0..30 {
            sim.next_room();
            for s in sim.state.run.slots.iter().filter(|s| s.live) {
                let room = &sim.rc.rooms[usize::from(s.room)];
                assert_eq!(s.tier, run::tier_of(&sim.rc, s.seq));
                if !room.paired {
                    assert!(
                        room.min_tier <= s.tier && s.tier <= room.max_tier,
                        "seed {seed}: {} in tier {}",
                        room.name,
                        s.tier
                    );
                }
            }
        }
    }
    // A grunt spawned in tier 2 has 120% stamina.
    let mut sim = RunSim::new(3);
    for _ in 0..13 {
        sim.next_room();
    }
    sim.tick(Input::default());
    for _ in 0..200 {
        sim.tick(Input::default());
        if sim.state.run.mobs.iter().any(|m| m.active) {
            break;
        }
    }
    for m in sim
        .state
        .run
        .mobs
        .iter()
        .filter(|m| m.active && m.mode != MODE_BURST)
    {
        let base = sim.rc.mobs[usize::from(m.kind)].stamina;
        let pct = i32::from(sim.rc.tiers[usize::from(m.tier)].stamina);
        assert!(m.stamina <= base * fx(pct) / fx(100));
        assert_eq!(m.tier, run::tier_of(&sim.rc, m.room));
    }
}

/// A shieldbearer's guard blocks a hit from the front (its guard wears down, the attacker freezes, no damage); a cross-up before it
/// turns lands; so does a hit from above; a grab throws it out of its guard; a guard worn to nothing dazes it.
#[test]
fn a_shieldbearer_blocks_from_the_front_only() {
    let jab_at = |sim: &mut RunSim, n: usize| {
        for frame in 0..20 {
            sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
            let m = &mut sim.state.run.mobs[n];
            m.timer = 999;
            if m.percent > Fx::ZERO || sim.player().hitlag > 0 {
                break;
            }
        }
    };
    let guarded = |sim: &mut RunSim, facing: i8| {
        let n = grunt(sim, KIND_SHIELD, Vec2::new(fx(17), Fx::ZERO), 0);
        let m = &mut sim.state.run.mobs[n];
        m.mode = run::MODE_GUARD;
        m.facing = facing;
        n
    };
    // From the front.
    let mut sim = open_street(quiet(), 0);
    let n = guarded(&mut sim, -1);
    let full = sim.state.run.mobs[n].shield;
    jab_at(&mut sim, n);
    let m = sim.state.run.mobs[n];
    assert_eq!(m.percent, Fx::ZERO, "blocked");
    assert!(m.shield < full, "its guard wore down");
    assert!(sim.player().hitlag > 0, "the attacker froze on the shield");
    assert_eq!(sim.state.run.combo, 0, "a blocked hit is no combo");
    // Crossed up: its back to the player (it turns only `guard_turn` frames later).
    let mut sim = open_street(quiet(), 0);
    let n = guarded(&mut sim, 1);
    jab_at(&mut sim, n);
    assert!(sim.state.run.mobs[n].percent > Fx::ZERO, "a cross-up lands");
    // From above: a down air onto its head.
    let mut sim = open_street(quiet(), 0);
    let n = guarded(&mut sim, -1);
    let height =
        sim.world.fighters[usize::from(sim.rc.mobs[usize::from(KIND_SHIELD)].body)].ecb_height;
    sim.place(Vec2::new(fx(17), height + Fx::from_ratio(5, 10)));
    let mut landed = false;
    for frame in 0..40 {
        sim.tick(inp(
            0,
            if frame == 0 { -127 } else { 0 },
            if frame == 0 { ATTACK } else { 0 },
        ));
        sim.state.run.mobs[n].timer = 999;
        if sim.state.run.mobs[n].percent > Fx::ZERO {
            landed = true;
            break;
        }
    }
    assert!(landed, "a hit from above lands");
    // A grab: thrown, guard and all.
    let mut sim = open_street(quiet(), 0);
    let n = guarded(&mut sim, -1);
    sim.state.run.mobs[n].pos.x = fx(16) + Fx::from_ratio(12, 10);
    let mut thrown = false;
    for frame in 0..30 {
        sim.tick(inp(0, 0, if frame == 0 { GRAB } else { 0 }));
        let m = sim.state.run.mobs[n];
        if m.percent > Fx::ZERO {
            thrown = m.hitstun > 0 && m.mode != run::MODE_GUARD;
            break;
        }
    }
    assert!(thrown, "the grab threw it");
    // A guard worn out: dazed.
    let mut sim = open_street(quiet(), 0);
    let n = guarded(&mut sim, -1);
    sim.state.run.mobs[n].shield = Fx::from_ratio(1, 10);
    jab_at(&mut sim, n);
    assert_eq!(
        sim.state.run.mobs[n].mode,
        run::MODE_DAZED,
        "its guard broke"
    );
}

/// A bruiser's armor: a hit through its tell or its smash's startup deals damage but does not flinch it; out of it, it flinches.
#[test]
fn a_bruisers_armor_holds_through_its_startup() {
    let mut sim = open_street(quiet(), 0);
    let n = grunt(&mut sim, KIND_BRUISER, Vec2::new(fx(17), Fx::ZERO), 0);
    {
        let m = &mut sim.state.run.mobs[n];
        m.mode = MODE_WINDUP;
        m.move_slot = 0;
        m.timer = 200;
    }
    for frame in 0..20 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        if sim.state.run.mobs[n].percent > Fx::ZERO {
            break;
        }
    }
    let m = sim.state.run.mobs[n];
    assert!(m.percent > Fx::ZERO, "it took the damage");
    assert_eq!(m.hitstun, 0, "and did not flinch");
    assert_eq!(m.mode, MODE_WINDUP);
    // Cooling down: no armor.
    let mut sim = open_street(quiet(), 0);
    let n = grunt(&mut sim, KIND_BRUISER, Vec2::new(fx(17), Fx::ZERO), 0);
    sim.state.run.mobs[n].mode = run::MODE_COOLDOWN;
    sim.state.run.mobs[n].timer = 200;
    for frame in 0..20 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        if sim.state.run.mobs[n].percent > Fx::ZERO {
            break;
        }
    }
    assert!(sim.state.run.mobs[n].hitstun > 0, "it flinches");
}

/// A lobber on high ground throws its shot (a real projectile, from the slot standing for its body) in an arc onto the player.
#[test]
fn a_lobber_throws_its_shot_in_an_arc() {
    let mut sim = open_street(quiet(), 0);
    sim.stand(Vec2::new(fx(19), Fx::ZERO));
    // On the start room's first awning (3.6 up).
    let n = grunt(
        &mut sim,
        KIND_LOBBER,
        Vec2::new(fx(9), Fx::from_ratio(36, 10)),
        0,
    );
    assert_ne!(sim.state.run.mobs[n].platform, NONE);
    let mut thrown = false;
    for _ in 0..300 {
        sim.tick(Input::default());
        sim.state.fighters[PLAYER].pos.x = fx(19);
        thrown |= sim
            .state
            .projectiles
            .iter()
            .any(|p| p.active && usize::from(p.owner) == run::SHOT_SLOT);
        if sim.player().percent > Fx::ZERO {
            break;
        }
    }
    assert!(thrown, "it threw");
    assert!(
        sim.player().percent > Fx::ZERO,
        "the shot came down on the player"
    );
}

/// A lobber's shot reflected (the tumbler's cape) is the player's: it flies back and hits the grunts.
#[test]
fn a_reflected_shot_hits_the_grunts() {
    const TUMBLER: u8 = 3;
    let mut sim = open_street(quiet(), TUMBLER);
    sim.state.fighters[PLAYER].facing = 1;
    let target = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(22), Fx::ZERO), 0);
    sim.state.run.mobs[target].mode = run::MODE_COOLDOWN;
    sim.state.run.mobs[target].timer = 999;
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..8 {
        sim.tick(Input::default());
        sim.state.run.mobs[target].timer = 999;
    }
    let shell = sim.rc.mobs[usize::from(KIND_LOBBER)].moves[0];
    sim.state.projectiles[0] = sim_core::Projectile {
        active: true,
        owner: run::SHOT_SLOT as u8,
        origin: run::SHOT_SLOT as u8,
        power: 100,
        move_id: shell,
        pos: Vec2::new(fx(16) + Fx::from_ratio(22, 10), Fx::from_ratio(8, 10)),
        vel: Vec2::new(-Fx::from_ratio(18, 100), Fx::ZERO),
        life: 46,
        size: 100,
        facing: -1,
        ..sim_core::Projectile::default()
    };
    let mut reflected = false;
    for _ in 0..60 {
        sim.tick(Input::default());
        sim.state.run.mobs[target].timer = 999;
        let p = sim.state.projectiles[0];
        reflected |= p.active && usize::from(p.owner) == PLAYER;
        if sim.state.run.mobs[target].percent > Fx::ZERO {
            break;
        }
    }
    assert!(reflected, "the cape turned it round");
    assert_eq!(sim.player().percent, Fx::ZERO, "it did not hit the player");
    assert!(
        sim.state.run.mobs[target].percent > Fx::ZERO,
        "it hit the grunt"
    );
    assert!(sim.state.run.combo >= 1, "the player's hit");
}

/// A room's grunts over many seeds: what its tier allows, within its budget (an arena: at least one), lobbers and leapers on high
/// ground, bruisers on flat room, never two shieldbearers on one platform, and grunts still come out of their doors.
#[test]
fn placement_rules_and_budgets_hold_over_many_seeds() {
    let mut kinds_seen = [0u32; 6];
    let mut checked = 0;
    for seed in 0..30u64 {
        let mut sim = RunSim::new(seed);
        let mut seen: Vec<u32> = Vec::new();
        for _ in 0..30 {
            sim.next_room();
            let rc = &sim.rc;
            for s in sim.state.run.slots.iter().filter(|s| s.live) {
                if seen.contains(&s.seq) {
                    continue;
                }
                seen.push(s.seq);
                checked += 1;
                let room = &rc.rooms[usize::from(s.room)];
                let tier = rc.tiers[usize::from(s.tier)];
                let mut budget = (room.width * fx(i32::from(tier.budget)) / fx(16)).floor_int();
                if room.kind == ROOM_ARENA {
                    budget = budget * i32::from(rc.rules.arena_budget) / 100;
                }
                let mut feet = Vec::new();
                run::for_each_footing(rc, s, |p| feet.push(p.platform));
                let lowest = feet
                    .iter()
                    .filter(|p| !p.pass_through)
                    .map(|p| p.y.min(p.top_right()))
                    .min()
                    .unwrap();
                let mut spent = 0;
                let mut count = 0;
                let mut alone_on = Vec::new();
                for (j, &k) in s.kinds.iter().enumerate() {
                    if k == SPOT_ANY {
                        continue;
                    }
                    count += 1;
                    let kind = rc.mobs[usize::from(k)];
                    kinds_seen[usize::from(k)] += 1;
                    spent += i32::from(kind.cost);
                    assert!(
                        tier.kinds & (1 << k) != 0 && kind.min_tier <= s.tier,
                        "seed {seed}: kind {k} in tier {}",
                        s.tier
                    );
                    let spot = room.spawns[j];
                    let at = if s.mirror {
                        Vec2::new(s.x + room.width - spot.x, s.y + spot.y - room.exit_y)
                    } else {
                        Vec2::new(s.x + spot.x, s.y + spot.y)
                    };
                    let support = feet
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            at.x >= p.left
                                && at.x <= p.right
                                && (p.top_at(at.x) - at.y).abs() <= Fx::HALF
                        })
                        .max_by_key(|(_, p)| p.top_at(at.x))
                        .map(|(i, _)| i)
                        .expect("it stands on something");
                    if kind.place & PLACE_HIGH != 0 {
                        assert!(
                            at.y - lowest >= rc.rules.high_ground,
                            "seed {seed}: kind {k} on high ground in {}",
                            room.name
                        );
                    }
                    if kind.place & PLACE_FLAT != 0 {
                        let p = feet[support];
                        assert!(
                            !p.sloped() && p.right - p.left >= rc.rules.flat_room,
                            "seed {seed}: kind {k} on flat room"
                        );
                    }
                    if kind.place & PLACE_ALONE != 0 {
                        assert!(
                            !alone_on.contains(&support),
                            "seed {seed}: two shieldbearers on one platform in {}",
                            room.name
                        );
                        alone_on.push(support);
                    }
                }
                if room.kind == ROOM_ARENA {
                    assert!(count >= 1, "seed {seed}: an arena with nobody in it");
                    assert!(
                        spent <= budget || count == 1,
                        "seed {seed}: over budget in {}",
                        room.name
                    );
                } else {
                    assert!(
                        spent <= budget,
                        "seed {seed}: {spent} over a budget of {budget} in {}",
                        room.name
                    );
                }
                if room.kind == ROOM_SHOP || room.kind == ROOM_REST || room.kind == run::ROOM_ALLEY
                {
                    assert_eq!(count, 0, "seed {seed}: grunts in a {}", room.name);
                }
            }
        }
    }
    assert!(checked > 500, "{checked} rooms");
    assert!(
        kinds_seen.iter().all(|&n| n > 0),
        "every kind comes: {kinds_seen:?}"
    );
}

/// Hazards hurt grunts (the player's kills) and the player: a gas canister hit burns, then bursts; a loose sign falls on whoever is
/// under it and breaks where it lands; a chest hit open pays once.
#[test]
fn hazards_hurt_grunts_and_chests_pay() {
    let put = |sim: &mut RunSim, kind: u8, at: Vec2| {
        let room = sim.state.run.room;
        let i = sim.state.run.props.iter().position(|p| !p.active).unwrap();
        sim.state.run.props[i] = run::Prop {
            active: true,
            kind,
            room,
            pos: at,
            ..run::Prop::default()
        };
        i
    };
    // A gas canister by the player, a grunt beyond it.
    let mut sim = open_street(quiet(), 0);
    let gas = put(&mut sim, PROP_KIND_GAS, Vec2::new(fx(17), Fx::ZERO));
    let n = grunt(
        &mut sim,
        KIND_BRAWLER,
        Vec2::new(fx(18) + Fx::HALF, Fx::ZERO),
        0,
    );
    sim.state.run.mobs[n].stamina = Fx::ONE;
    sim.state.run.mobs[n].mode = run::MODE_COOLDOWN;
    sim.state.run.mobs[n].timer = 999;
    let mut lit = false;
    for frame in 0..60 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        lit |= sim.state.run.props[gas].mode == run::PROP_LIT;
        if !sim.state.run.props[gas].active {
            break;
        }
    }
    assert!(lit, "the hit lit it");
    assert!(!sim.state.run.props[gas].active, "it burst and is gone");
    assert_eq!(
        sim.state.run.kills, 1,
        "the blast beat the grunt: the player's kill"
    );
    assert!(sim.player().percent > Fx::ZERO, "and caught the player too");
    // A loose sign falling onto a grunt.
    // (Clear of the start room's awnings, the player well away.)
    let mut sim = open_street(quiet(), 0);
    sim.stand(Vec2::new(fx(3), Fx::ZERO));
    let n = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(16), Fx::ZERO), 0);
    sim.state.run.mobs[n].mode = run::MODE_COOLDOWN;
    sim.state.run.mobs[n].timer = 999;
    let sign = put(&mut sim, PROP_KIND_SIGN, Vec2::new(fx(16), fx(5)));
    sim.state.run.props[sign].mode = run::PROP_FALLING;
    let mut broke = false;
    for _ in 0..90 {
        sim.tick(Input::default());
        broke |= sim.state.run.props[sign].mode == run::PROP_BROKEN;
        if sim.state.run.mobs[n].active && sim.state.run.mobs[n].hitstun == 0 {
            sim.state.run.mobs[n].timer = 999;
        }
    }
    assert!(
        sim.state.run.mobs[n].percent > Fx::ZERO,
        "it fell on the grunt"
    );
    assert!(broke, "and broke where it landed");
    assert_eq!(sim.player().percent, Fx::ZERO, "the player was clear");
    // A chest.
    let mut sim = open_street(quiet(), 0);
    let chest = put(&mut sim, PROP_KIND_CHEST, Vec2::new(fx(17), Fx::ZERO));
    let coins = sim.state.run.coins;
    for frame in 0..40 {
        sim.tick(inp(0, 0, if frame % 20 == 0 { ATTACK } else { 0 }));
    }
    let pay = u32::from(sim.rc.props[usize::from(PROP_KIND_CHEST)].coins);
    assert_eq!(sim.state.run.props[chest].mode, run::PROP_OPEN);
    assert_eq!(sim.state.run.coins, coins + pay, "it paid once");
    assert_eq!(sim.state.run.chests, 1);
}

/// The first rare event (the runaway van): at most one in each city, never before `event_from`, weighted up by the pity counter for
/// every room without one, so it is rare but never too rare; it is marked on the map.
#[test]
fn rare_events_are_rare_but_come_with_the_pity_counter() {
    let rc = RunContent::placeholder();
    let r = rc.rules;
    let city = u32::from(r.city_rooms);
    let mut cities = 0;
    let mut with = 0;
    for seed in 0..24u64 {
        let mut sim = RunSim::new(seed);
        let mut events: Vec<u32> = Vec::new();
        for _ in 0..96 {
            sim.next_room();
            let s = *sim.state.run.slot_of(sim.state.run.room).unwrap();
            if sim.rc.rooms[usize::from(s.room)].kind == ROOM_EVENT && !events.contains(&s.seq) {
                events.push(s.seq);
                assert_eq!(
                    sim.state.run.kind_of(s.seq),
                    Some(ROOM_EVENT),
                    "the map marks it"
                );
            }
        }
        for e in &events {
            assert!(
                *e >= u32::from(r.event_from),
                "seed {seed}: an event at room {e}"
            );
        }
        let full_cities = sim.state.run.room / city;
        for c in 0..full_cities {
            let n = events.iter().filter(|&&e| e / city == c).count();
            assert!(n <= 1, "seed {seed}: {n} events in city {c}");
            cities += 1;
            with += usize::from(n == 1);
        }
    }
    assert!(
        with * 10 >= cities * 7,
        "{with} of {cities} cities had their event"
    );
    assert!(with < cities, "not certain");
    // The pity counter: with no base chance and a big step, the event comes on the second eligible room; with neither, never.
    let mut sure = RunContent::placeholder();
    sure.rules.event_base = 0;
    sure.rules.event_step = 1000;
    let mut sim = RunSim::with(sure, 2);
    let mut first = None;
    for _ in 0..20 {
        sim.next_room();
        let s = *sim.state.run.slot_of(sim.state.run.room).unwrap();
        if first.is_none() && sim.rc.rooms[usize::from(s.room)].kind == ROOM_EVENT {
            first = Some(s.seq);
        }
    }
    let first = first.expect("it came");
    assert!(
        first > u32::from(r.event_from) && first <= u32::from(r.event_from) + 3,
        "at {first}"
    );
    let mut never = RunContent::placeholder();
    never.rules.event_base = 0;
    never.rules.event_step = 0;
    let mut sim = RunSim::with(never, 2);
    for _ in 0..60 {
        assert_ne!(sim.next_room(), ROOM_EVENT);
    }
}

/// The runaway van end to end: it waits until the player lands on its roof, then drives down the street carrying them (untouchable
/// while riding), runs into the grunts in its way (the player's kills or launches), and parks at the street's end.
#[test]
fn riding_the_runaway_van() {
    let mut rc = quiet();
    rc.rules.event_from = 1;
    rc.rules.event_base = 1000;
    let mut sim = RunSim::with(rc, 5);
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(sim.rc.rooms[usize::from(slot.room)].kind, ROOM_EVENT);
    sim.next_room();
    sim.clear_mobs();
    let v = sim.state.run.van;
    assert_eq!(v.mode, run::VAN_WAITING);
    // Not until someone is on its roof.
    sim.stand(Vec2::new(v.x + fx(4), Fx::ZERO));
    sim.idle(30);
    assert_eq!(sim.state.run.van.mode, run::VAN_WAITING);
    // A grunt down the street in its way.
    let n = grunt(&mut sim, KIND_BRAWLER, Vec2::new(v.x + fx(14), Fx::ZERO), 0);
    sim.state.run.mobs[n].mode = run::MODE_COOLDOWN;
    sim.state.run.mobs[n].timer = 9999;
    sim.stand(Vec2::new(v.x, v.y + run::VAN_HEIGHT));
    let start = sim.player().pos.x;
    let mut rode = 0;
    for _ in 0..400 {
        sim.tick(Input::default());
        let v = sim.state.run.van;
        if v.riding {
            rode += 1;
            assert!(sim.player().invuln > 0, "untouchable while riding");
        }
        if sim.state.run.mobs[n].active {
            sim.state.run.mobs[n].timer = 9999;
        }
        if v.mode == run::VAN_PARKED {
            break;
        }
    }
    let v = sim.state.run.van;
    assert!(rode > 30, "it carried the player ({rode} frames)");
    assert!(
        sim.player().pos.x > start + fx(20),
        "a long way down the street"
    );
    assert_eq!(v.mode, run::VAN_PARKED, "it parked");
    assert!(v.x + run::VAN_HALF <= slot.end, "at its street's end");
    assert!(v.hits >= 1, "it ran into the grunt");
    let m = sim.state.run.mobs[n];
    assert!(
        !m.active || m.percent > Fx::ZERO,
        "the grunt was sent flying"
    );
}

/// A branching fork (the owner's R2a answer): its two roads come next, side by side, the alley the fork's lift up; walking out on the
/// high road keeps the alley (with its chest) and drops the low road's room at once, and the other way round; the map marks the road
/// not taken; the run goes on from the room taken.
#[test]
fn a_branching_fork_leads_to_two_rooms() {
    // (The built-in rooms, props kept, no grunts in the streets, no arenas or events.)
    let mut rc = RunContent::placeholder();
    for r in rc.rooms.iter_mut().filter(|r| r.kind == ROOM_STREET) {
        r.spawns.clear();
    }
    rc.rules.arena_weight = 0;
    rc.rules.event_base = 0;
    rc.rules.event_step = 0;
    rc.rules.fork_chance = 100;
    let fork_up = room_named(&rc, "fork_up");
    let seed = (0..200u64)
        .find(|&s| {
            let sim = RunSim::with(rc.clone(), s);
            sim.state
                .run
                .slot_of(1)
                .is_some_and(|x| usize::from(x.room) == fork_up)
        })
        .expect("a seed where the second room is the branching fork");
    for high in [true, false] {
        let mut sim = RunSim::with(rc.clone(), seed);
        let fork = *sim.state.run.slot_of(1).unwrap();
        assert_eq!(
            sim.state.run.branches(),
            None,
            "nothing past the fork until it is entered"
        );
        sim.next_room();
        let (alley, low) = sim.state.run.branches().expect("both roads laid");
        assert_eq!(sim.rc.rooms[usize::from(alley.room)].kind, run::ROOM_ALLEY);
        assert_eq!(alley.x, fork.end);
        assert_eq!(low.x, fork.end);
        assert_eq!(alley.y, fork.y_end + sim.rc.rooms[fork_up].lift);
        assert_eq!(low.y, fork.y_end);
        assert!(
            sim.state
                .run
                .props
                .iter()
                .any(|p| p.active && p.room == alley.seq),
            "the alley's chest"
        );
        // Out on one road.
        let (take, other) = if high { (alley, low) } else { (low, alley) };
        sim.place(Vec2::new(take.x + fx(3), take.y + Fx::HALF));
        sim.state.fighters[PLAYER].invuln = 30;
        sim.tick(Input::default());
        let run = &sim.state.run;
        assert_eq!(run.room, take.seq, "high {high}: the road taken");
        assert!(run.slot_of(other.seq).is_none(), "the other is dropped");
        assert_eq!(run.skipped(other.seq), Some(true), "and marked on the map");
        assert_eq!(run.skipped(take.seq), Some(false));
        assert!(
            run.platforms.iter().all(|p| !p.live || p.seq != other.seq),
            "nothing of it left"
        );
        assert!(run.props.iter().all(|p| !p.active || p.room != other.seq));
        let (ahead, n) = run.ahead();
        assert_eq!(n, 1, "the run goes on");
        assert_eq!(ahead[0].x, take.end);
        assert_eq!(ahead[0].y, take.y_end);
        assert_eq!(
            run.props.iter().any(|p| p.active && p.room == take.seq),
            high,
            "the chest only on the high road"
        );
        // The street under the alley is still there to land on.
        sim.stand(Vec2::new(take.x + fx(10), fork.y_end));
    }
}

/// The owner's R2a answers: a coin is worth 1.
#[test]
fn a_coin_is_worth_one() {
    assert_eq!(RunContent::placeholder().rules.coin_value, 1);
}

/// Grunts, props, the van and the branching forks replay exactly: the same seed and inputs give the same checksums.
#[test]
fn a_run_with_grunts_props_and_events_replays() {
    let mut rc = RunContent::placeholder();
    rc.rules.event_from = 2;
    rc.rules.event_base = 300;
    rc.rules.fork_chance = 60;
    let inputs = rightward_inputs(11, 2400);
    let play = |rc: &RunContent| {
        let mut sim = RunSim::with(rc.clone(), 9);
        let mut sums = Vec::new();
        for (n, i) in inputs.iter().enumerate() {
            if n % 200 == 0 && !sim.state.run.inside && !sim.state.run.over {
                // (Carried on into the next room now and then, an arena cleared, so the generator's rooms come.)
                if sim.state.run.gate_closed {
                    sim.clear_mobs();
                    sim.tick(Input::default());
                }
                if !sim.state.run.gate_closed && sim.state.run.ahead().1 > 0 {
                    sim.next_room();
                }
            }
            step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
            sums.push(sim.state.checksum());
        }
        (sums, sim.state.run.placed)
    };
    let (a, placed) = play(&rc);
    let (b, _) = play(&rc);
    assert_eq!(a, b);
    assert!(placed > 8);
}
