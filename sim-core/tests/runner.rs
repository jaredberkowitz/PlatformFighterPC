//! The single-player run (R1, sim 120; rooms R1b, sim 122): the room window (dropping old rooms, the left wall, no backtracking past
//! it), the camera, arenas, the shop, rest stops and the generator's rules; mobs, coins and the combo; and determinism.

mod common;

use common::inp;
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::run::data::{MobKind, RunContent, SPOT_ANY};
use sim_core::run::{
    self, door_here, grunt_slot, new_run, step_run, GATE_INDEX, MAX_MOBS, MODE_ATTACK, MODE_IDLE,
    MODE_MOVE, MODE_WINDUP, NOTE_BOUGHT, NOTE_NOT_NEEDED, NOTE_SOLD_OUT, NOTE_TOO_POOR, PLAYER,
    RING, ROOM_ARENA, ROOM_REST, ROOM_SHOP, ROOM_STREET, WALL_INDEX,
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
        run::clear_grunts(&mut self.state, &self.world);
    }

    /// Grunt `n`'s fighter (R2c: a grunt is one).
    fn g(&self, n: usize) -> &sim_core::Fighter {
        &self.state.fighters[grunt_slot(n)]
    }

    fn g_mut(&mut self, n: usize) -> &mut sim_core::Fighter {
        &mut self.state.fighters[grunt_slot(n)]
    }

    /// Keeps grunt `n` from doing anything of its own (cooling down for good).
    fn hold(&mut self, n: usize) {
        let m = &mut self.state.run.mobs[n];
        m.mode = run::MODE_COOLDOWN;
        m.timer = 9999;
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

    /// A grunt of `kind` (tier 0) awake at `at` (standing there if there is ground), facing the player.
    fn add_mob(&mut self, kind: u8, at: Vec2) -> usize {
        sim_core::run::build_stage(&self.state.run, &self.rc, &mut self.world.stage);
        run::place_grunt(&mut self.state, &self.world, &self.rc, kind, at, 0)
            .expect("a free grunt slot")
    }

    /// Walks (teleports) the player into the next room, clearing an arena it locks; returns the room's type.
    fn next_room(&mut self) -> u8 {
        self.take_road(0)
    }

    /// As `next_room`, but at a branching fork onto road `k` (0 the high road, 1 the low road: R2d).
    fn take_road(&mut self, k: usize) -> u8 {
        let (ahead, n) = self.state.run.ahead();
        assert!(n >= 1, "the room ahead is laid");
        // (At a branching fork, the high road's alley: it is laid first, and the player comes in from above.)
        let k = if self.state.run.branches().is_some() {
            k.min(1)
        } else {
            0
        };
        let slot = ahead[k];
        let target = slot.seq;
        let at = Vec2::new(
            slot.x + self.rc.rules.enter_depth + Fx::ONE,
            slot.y + if k == 1 { fx(2) } else { fx(12) },
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
        // (An arena's grunts come out in waves, the tier's cap at a time and a stagger apart (R2e): cleared again until it opens.)
        for _ in 0..2000 {
            if !self.state.run.gate_closed {
                break;
            }
            self.clear_mobs();
            self.tick(Input::default());
        }
        assert!(!self.state.run.gate_closed, "cleared");
        kind
    }
}

/// No cap on grunts alive and no stagger between doorways (R2e's note A off), for tests about something else.
fn uncap(rc: &mut RunContent) {
    for t in rc.tiers.iter_mut() {
        t.cap = MAX_MOBS as u8;
    }
    rc.rules.emerge_gap = 0;
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
    sim.hold(left_mob);
    for _ in 0..3 {
        let slot = *sim.state.run.slot_of(sim.state.run.room + 1).unwrap();
        sim.stand(Vec2::new(slot.x + fx(3), Fx::ZERO));
        if sim.state.run.mobs[left_mob].active {
            sim.g_mut(left_mob).pos.x = x0 + fx(4);
        }
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
    assert!(!sim.g(left_mob).active, "its fighter too");
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
    // A grunt not launched cannot get behind it either (only a tumbling one goes through: R2c).
    let m = sim.add_mob(1, Vec2::new(wall + fx(2), fx(6)));
    sim.hold(m);
    sim.g_mut(m).vel = Vec2::new(-Fx::HALF, Fx::ZERO);
    for _ in 0..10 {
        sim.tick(Input::default());
        assert!(sim.g(m).pos.x > wall);
    }
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

/// The owner's R2d answer 4 ("a floor below"): a fall into the gap room's gap lands on the lower roof in it, no life lost; a room
/// still marked a true pit (none of the built-in ones is) costs a life.
#[test]
fn a_fall_into_a_gap_lands_on_a_floor_below_and_a_true_pit_costs_a_life() {
    for pit in [false, true] {
        let mut rc = only(&["gap", "shop"], (1, 0, 0));
        let gap = rc.rooms.iter().position(|r| r.name == "gap").unwrap();
        if pit {
            rc.rooms[gap].platforms.truncate(3);
            rc.rooms[gap].pit = true;
        }
        let mut sim = RunSim::with(rc, 1);
        let slot = *sim.state.run.slot_of(1).unwrap();
        assert_eq!(usize::from(slot.room), gap, "the only street but the start");
        sim.place(Vec2::new(slot.x + fx(13), fx(-3)));
        for _ in 0..240 {
            sim.tick(inp(0, -127, 0));
            if sim.player().stocks < 3 {
                break;
            }
        }
        if pit {
            assert_eq!(sim.player().stocks, 2, "a true pit");
        } else {
            assert_eq!(sim.player().stocks, 3, "a floor below");
            assert!(sim.player().grounded());
            assert!(sim.player().pos.y < slot.y - fx(3), "below the roofs");
        }
    }
}

#[test]
fn leaving_the_screen_costs_a_life_and_the_last_life_ends_the_run() {
    let mut sim = RunSim::with(quiet(), 1);
    for lives in (0..3u8).rev() {
        let top = sim.state.run.blast_y(&sim.rc.rules).1;
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
    // (All its grunts out at once: no cap and no stagger here; the waves have their own test.)
    let mut rc = only(&["yard", "shop"], (0, 1, 0));
    uncap(&mut rc);
    let mut sim = RunSim::with(rc, 4);
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
                sim.g_mut(i).pos.x = slot.x + fx(3);
                sim.hold(i);
            }
        }
    }
    assert_eq!(sim.state.run.room, 1);
    // All but one beaten: still shut.
    let coins = sim.state.run.coins;
    for &i in &theirs[1..] {
        run::remove_grunt(&mut sim.state, &sim.world, i);
    }
    sim.tick(Input::default());
    assert!(sim.state.run.gate_closed);
    run::remove_grunt(&mut sim.state, &sim.world, theirs[0]);
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
    // R2c (the owner's rule): not on a timer. Ten seconds later it still sulks.
    sim.idle(600);
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, run::NOTE_REFUSED, "still closed");
    assert_eq!(sim.state.run.keeper_mood, run::KEEPER_HMPH);
    // Out, back and in again without going on: still closed.
    stand_inside(&mut sim, shop.door);
    press_up(&mut sim);
    assert!(!sim.state.run.inside, "out on the street");
    assert_ne!(sim.state.run.keeper_grudge, 0, "the grudge stands");
    // On into a room further right than the shop: it opens again.
    let shop_room = sim.state.run.shop_room;
    while sim.state.run.room <= shop_room {
        sim.next_room();
    }
    assert_eq!(sim.state.run.keeper_grudge, 0, "a new room lifts it");
    let (_, door) = shop_door(&sim);
    sim.stand(door);
    sim.tick(Input::default());
    press_up(&mut sim);
    assert!(sim.state.run.inside, "back in");
    assert_ne!(sim.state.run.keeper_mood, run::KEEPER_HMPH, "over it");
    stand_inside(&mut sim, shop.pedestals[0].x);
    press_up(&mut sim);
    assert_eq!(sim.state.run.shop_note, run::NOTE_BOUGHT, "and sells again");
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

/// The player's attack launches a grunt with the versus formulas: more knockback at a higher percent, hitlag on both, hitstun.
#[test]
fn a_mob_takes_knockback_that_grows_with_its_percent() {
    let launch = |percent: i32| {
        let mut sim = RunSim::with(quiet(), 1);
        sim.clear_mobs();
        let x = sim.player().pos.x;
        let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
        sim.hold(n);
        sim.g_mut(n).percent = fx(percent);
        sim.state.run.mobs[n].last_percent = fx(percent);
        let mut hit = false;
        for frame in 0..30 {
            sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
            let g = *sim.g(n);
            if g.percent > fx(percent) {
                hit = true;
                assert!(g.hitlag > 0, "the grunt freezes");
                assert!(sim.player().hitlag > 0, "and so does the attacker");
                assert!(g.hitstun > 0 || g.launch_pending);
                break;
            }
        }
        assert!(hit, "the jab missed");
        assert_eq!(sim.state.run.combo, 1, "a hit counts in the combo");
        let mut most = Fx::ZERO;
        for _ in 0..10 {
            sim.tick(Input::default());
            if sim.state.run.mobs[n].active {
                most = most.max(sim.g(n).kb_vel.length());
            }
        }
        most
    };
    assert!(launch(100) > launch(0));
}

/// R2c: a grunt is knocked out only by the blast zones, never by damage (no stamina burst by default); an optional stamina, if a
/// kind has one, still beats it where it stands, paid as a kill.
#[test]
fn grunts_have_no_stamina_unless_a_kind_gives_one() {
    let rc = RunContent::placeholder();
    assert!(rc.mobs.iter().all(|k| k.stamina == Fx::ZERO), "no stamina");
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
    sim.hold(n);
    sim.g_mut(n).percent = fx(900);
    sim.state.run.mobs[n].last_percent = fx(900);
    sim.idle(5);
    assert!(sim.state.run.mobs[n].active, "900% and standing");
    // With a stamina of 5: one jab beats it, a kill.
    let mut rc = quiet();
    rc.mobs[0].stamina = fx(5);
    let mut sim = RunSim::with(rc, 1);
    sim.clear_mobs();
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
    sim.hold(n);
    sim.g_mut(n).percent = fx(4);
    sim.state.run.mobs[n].last_percent = fx(4);
    for frame in 0..30 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        if sim.state.run.kills > 0 {
            break;
        }
    }
    assert_eq!(sim.state.run.kills, 1);
    assert!(!sim.state.run.mobs[n].active && !sim.g(n).active);
    assert_eq!(sim.state.run.coins, u32::from(sim.rc.mobs[0].coins));
}

/// A grunt launched past a blast zone is knocked out (the KO counted for the game's effect); it is the player's kill only if they
/// hit it lately. One that merely walks out of view is not knocked out.
#[test]
fn a_mob_launched_off_screen_is_a_kill_only_if_the_player_hit_it() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    sim.idle(2);
    let (_, _, _, top) = run::grunt_zone(&sim.state.run, &sim.rc.rules);
    let a = sim.add_mob(0, Vec2::new(fx(20), top + fx(1)));
    let b = sim.add_mob(0, Vec2::new(fx(22), top + fx(1)));
    for n in [a, b] {
        let g = sim.g_mut(n);
        g.state = S::Hitstun;
        g.hitstun = 40;
        g.vel = Vec2::new(Fx::ZERO, Fx::ONE);
    }
    sim.state.run.mobs[a].credit = 30;
    sim.tick(Input::default());
    assert!(!sim.state.run.mobs[a].active && !sim.state.run.mobs[b].active);
    assert_eq!(sim.state.run.kos, 2, "both knocked out");
    assert_eq!(sim.state.run.kills, 1, "one the player's");
    assert_eq!(sim.state.run.coins, u32::from(sim.rc.mobs[0].coins));
    // Not launched: a grunt standing high above the view is left alone.
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    sim.idle(2);
    let (_, right, _, _) = run::grunt_zone(&sim.state.run, &sim.rc.rules);
    let c = sim.add_mob(0, Vec2::new(right - Fx::HALF, Fx::ZERO));
    sim.hold(c);
    sim.g_mut(c).pos.x = right + fx(1);
    sim.tick(Input::default());
    assert!(sim.state.run.mobs[c].active, "walking out of view is no KO");
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
    for _ in 0..300 {
        sim.tick(Input::default());
        let m = sim.state.run.mobs[n];
        wound_up |= m.mode == MODE_WINDUP;
        if sim.player().percent > Fx::ZERO {
            hit = true;
            assert_eq!(m.mode, MODE_ATTACK);
            assert_eq!(sim.g(n).state, S::Attack, "with its fighter's own attack");
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
    sim.add_mob(0, Vec2::new(x + fx(8), Fx::ZERO));
    let shield = sim.player().shield_hp;
    let mut blocked = false;
    for _ in 0..300 {
        sim.tick(inp(0, 0, SHIELD));
        if sim.player().shield_stun > 0 {
            blocked = true;
            break;
        }
    }
    assert!(blocked, "its hit landed on the shield (shield stun)");
    assert_eq!(sim.player().percent, Fx::ZERO);
    assert!(sim.player().shield_hp < shield);
}

/// A leaper on high ground tells (crouching), leaps at a player below and strikes with an aerial (R2b: the diver's anti-air lesson;
/// R2c: its fighter's own jump and aerial).
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
    assert_ne!(sim.g(n).platform, NONE, "it stands on the awning");
    let (mut told, mut crouched, mut leapt, mut aerial) = (false, false, false, false);
    for _ in 0..400 {
        sim.tick(Input::default());
        let m = sim.state.run.mobs[n];
        let g = *sim.g(n);
        told |= m.mode == MODE_WINDUP;
        crouched |= m.mode == MODE_WINDUP && g.state == S::Crouch;
        leapt |= m.mode == run::MODE_LEAP && !g.grounded();
        aerial |= m.mode == MODE_ATTACK && g.state == S::Attack && !g.grounded();
        if sim.player().percent > Fx::ZERO {
            break;
        }
        // (The player holds still.)
        sim.state.fighters[PLAYER].pos.x = p.x;
    }
    assert!(
        told && crouched && leapt && aerial,
        "told {told}, crouched {crouched}, leapt {leapt}, aerial {aerial}"
    );
    assert!(sim.player().percent > Fx::ZERO, "the leap never hit");
}

/// One swing hits a grunt once per hit group, even though the attacker's hitlag pauses the move (versus's own hit records).
#[test]
fn one_swing_hits_a_mob_once() {
    let mut sim = RunSim::with(quiet(), 1);
    sim.clear_mobs();
    let x = sim.player().pos.x;
    let n = sim.add_mob(0, Vec2::new(x + fx(1), Fx::ZERO));
    sim.hold(n);
    for frame in 0..60 {
        sim.tick(inp(
            if frame < 2 { 60 } else { 0 },
            0,
            if frame == 0 { ATTACK } else { 0 },
        ));
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
    let top = sim.state.run.blast_y(&sim.rc.rules).1;
    let a = sim.add_mob(0, Vec2::new(fx(20), top + fx(1)));
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
                        // (A run's frame rebuilds the stage: the grunt is on the ground under it, or in the air.)
                    }
                }
            }
            step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
            let run = &sim.state.run;
            for (n, m) in run.mobs.iter().enumerate() {
                let g = &sim.state.fighters[grunt_slot(n)];
                assert_eq!(
                    m.active, g.active,
                    "seed {seed}: a grunt and its fighter agree"
                );
                if !m.active {
                    continue;
                }
                assert!(usize::from(m.kind) < sim.rc.mobs.len());
                // (A grunt launched in tumble may go through the gate behind, into the blast zone there.)
                assert!(
                    g.pos.x >= run.wall_x - sim.rc.rules.gate_thickness - sim.rc.rules.blast_margin,
                    "seed {seed}: a mob far behind the wall"
                );
                if let Ok(p) = usize::try_from(g.platform) {
                    assert!(
                        p >= run::POOL_PLATFORMS || run.platforms[p].live,
                        "a mob on a freed platform"
                    );
                }
                if g.grab_with != NONE {
                    let other = &sim.state.fighters[g.grab_with as usize];
                    assert_eq!(
                        other.grab_with as usize,
                        grunt_slot(n),
                        "seed {seed}: a one-sided grab"
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
        ("mob.credit", |r| r.mobs[0].credit += 1),
        ("mob.room", |r| r.mobs[0].room += 1),
        // R2b.
        ("mob.move_slot", |r| r.mobs[0].move_slot += 1),
        ("mob.tier", |r| r.mobs[0].tier += 1),
        // R2c (the grunt itself is a fighter: `state::tests::checksum_sees_every_field` covers it, the grunt slots included).
        ("mob.step", |r| r.mobs[0].step += 1),
        ("mob.last_percent", |r| {
            r.mobs[0].last_percent += Fx::from_raw(1)
        }),
        ("keeper_grudge", |r| r.keeper_grudge += 1),
        ("van.struck", |r| r.van.struck ^= 1 << 11),
        ("cam_y", |r| r.cam_y += Fx::from_raw(1)),
        ("kos", |r| r.kos += 1),
        ("ko_at", |r| r.ko_at.y += Fx::from_raw(1)),
        ("slots.kinds", |r| r.slots[0].kinds[7] ^= 1),
        ("slots.tier", |r| r.slots[0].tier += 1),
        ("platforms.seq", |r| r.platforms[0].seq += 1),
        ("ledges.seq", |r| r.ledges[0].seq += 1),
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
        // R2e.
        ("slots.emerged_at", |r| r.slots[0].emerged_at += 1),
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
        ("tier cap", {
            let mut o = rc.clone();
            o.tiers[0].cap += 1;
            o
        }),
        ("emerge_gap", {
            let mut o = rc.clone();
            o.rules.emerge_gap += 1;
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
    uncap(&mut rc);
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
    let at = sim.g(n).pos;
    for _ in 1..m.timer {
        sim.tick(Input::default());
        assert_eq!(sim.state.run.mobs[n].mode, run::MODE_EMERGE);
        assert_eq!(sim.g(n).pos, at, "it stands in its doorway");
        assert!(sim.g(n).invuln > 0, "untouchable meanwhile");
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
    assert_ne!(sim.g(n).platform, NONE, "on the ramp");
    let mut climbed = false;
    for _ in 0..240 {
        sim.tick(Input::default());
        let m = sim.state.run.mobs[n];
        let g = *sim.g(n);
        if let Ok(i) = usize::try_from(g.platform) {
            let p = sim.world.stage.platforms[i];
            assert_eq!(g.pos.y, p.top_at(g.pos.x), "its feet on its ground");
            climbed |= g.pos.y == ramp.top_max();
        }
        if m.mode == MODE_WINDUP {
            break;
        }
    }
    assert!(climbed, "it walked up onto the high ground");
}

/// The run's state stays small enough to save and restore every frame for rollback. R2c: the twelve grunts are fighters in the
/// state (about 400 bytes each), so the limit is 12 KiB (versus snapshots carry the empty grunt slots too).
#[test]
fn the_game_state_stays_small() {
    let size = std::mem::size_of::<GameState>();
    let run = std::mem::size_of::<run::Run>();
    let fighter = std::mem::size_of::<sim_core::Fighter>();
    println!("GameState: {size} bytes (the run: {run} bytes, a fighter: {fighter} bytes)");
    assert!(size <= 12 * 1024, "GameState is {size} bytes");
}

// ---- R2b: grunts, hazards, the curve, rare events, branching forks (sim 127; grunts as fighters in R2c, sim 131) -------------------

use run::data::{
    KIND_BRAWLER, KIND_BRUISER, KIND_LOBBER, KIND_SHIELD, KIND_SPEAR, PLACE_ALONE, PLACE_FLAT,
    PLACE_HIGH, PROP_KIND_CHEST, PROP_KIND_GAS, PROP_KIND_SIGN,
};
use sim_core::input::buttons::{GRAB, SPECIAL, STRONG};
use sim_core::moves::MoveId;
use sim_core::run::ROOM_EVENT;
use sim_core::MAX_BODIES;

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
    sim_core::run::build_stage(&sim.state.run, &sim.rc, &mut sim.world.stage);
    run::place_grunt(&mut sim.state, &sim.world, &sim.rc, kind, at, tier).expect("a free slot")
}

/// R2c: a grunt **is** a fighter of its body, in the fighter slots after the players, with its kind's and tier's weight and damage.
#[test]
fn a_grunt_is_a_fighter_of_its_body() {
    let mut sim = open_street(quiet(), 0);
    for kind in 0..sim.rc.mobs.len() as u8 {
        sim.clear_mobs();
        let n = grunt(&mut sim, kind, Vec2::new(fx(20), Fx::ZERO), 2);
        let k = sim.rc.mobs[usize::from(kind)];
        let g = *sim.g(n);
        assert_eq!(grunt_slot(n), MAX_FIGHTERS + n);
        assert!(g.active && g.stocks > 0);
        assert_eq!(g.char_id, k.body, "kind {kind}: a fighter of its body");
        assert_eq!(
            run::grunt_fighter(&sim.state, n),
            g,
            "the game draws the fighter itself"
        );
        assert_eq!(g.weight, Some(run::grunt_weight(&sim.rc, kind, 2)));
        assert_eq!(g.damage_scale, u16::from(sim.rc.tiers[2].damage));
        let body = sim.world.fighters[usize::from(k.body)];
        let hurt = sim_core::combat::body_hurtboxes(&sim.world, &g);
        let top = hurt.iter().map(|(c, r)| c.y + *r).max().unwrap();
        assert!(
            top - g.pos.y >= Fx::from_ratio(3, 2),
            "kind {kind}: fighter-tall ({:?})",
            top - g.pos.y
        );
        assert!(body.ecb_height > Fx::from_ratio(12, 10));
    }
    // Every kind's moves are its body's own.
    for k in &sim.rc.mobs {
        let w = &sim.world.weapons[usize::from(sim.world.fighters[usize::from(k.body)].weapon)];
        for &id in k.moves.iter().filter(|&&id| id != run::data::NO_MOVE) {
            assert!(usize::from(id) < w.moves.len());
        }
    }
}

/// Each ground kind tells for its tier's frames (18 in tier 0, a "!"), then makes its body's move with its fighter, and the move hits.
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
        for _ in 0..300 {
            sim.tick(Input::default());
            sim.state.fighters[PLAYER].pos.x = fx(16);
            let m = sim.state.run.mobs[n];
            if m.mode == MODE_WINDUP {
                told += 1;
            }
            let g = sim.g(n);
            if g.state == S::Attack && struck_with.is_none() {
                struck_with = Some(g.move_id);
            }
            if sim.player().percent > Fx::ZERO {
                break;
            }
        }
        assert_eq!(told, 18, "kind {kind}: an 18-frame tell");
        let id = struck_with.expect("it attacked");
        assert!(
            k.moves.contains(&id),
            "kind {kind}: one of its kit's moves ({id})"
        );
        assert!(sim.player().percent > Fx::ZERO, "kind {kind}: the move hit");
    }
}

/// The tell shortens with the tier, and a grunt's damage grows with it (its fighter's `damage_scale`).
#[test]
fn tiers_shorten_tells_and_toughen_grunts() {
    let rc = RunContent::placeholder();
    assert_eq!(rc.tiers[0].tell, 18);
    assert!(rc.tiers.windows(2).all(|w| w[1].tell < w[0].tell
        && w[1].damage >= w[0].damage
        && w[1].budget >= w[0].budget
        && w[1].weight > w[0].weight));
    let hit = |tier: u8| {
        let mut sim = open_street(quiet(), 0);
        let n = grunt(
            &mut sim,
            KIND_BRAWLER,
            Vec2::new(fx(16) + Fx::from_ratio(14, 10), Fx::ZERO),
            tier,
        );
        let mut told = 0;
        for _ in 0..300 {
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
/// and a grunt spawned in a room gets its tier (its weight and damage).
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
    let mut seen = 0;
    for (n, m) in sim
        .state
        .run
        .mobs
        .iter()
        .enumerate()
        .filter(|(_, m)| m.active)
    {
        assert_eq!(m.tier, run::tier_of(&sim.rc, m.room));
        let g = &sim.state.fighters[grunt_slot(n)];
        assert_eq!(g.weight, Some(run::grunt_weight(&sim.rc, m.kind, m.tier)));
        seen += 1;
    }
    assert!(seen > 0);
}

/// R2c: the shieldbearer's shield is a versus shield. A hit on it is blocked (shield stun, no damage, the attacker freezes); a grab
/// beats it; held too long under pressure it breaks, and the grunt is stunned like a fighter (the broken-shield daze).
#[test]
fn a_shieldbearer_shields_like_a_fighter() {
    let shielding = |sim: &mut RunSim| {
        let n = grunt(sim, KIND_SHIELD, Vec2::new(fx(17), Fx::ZERO), 0);
        // (It holds its shield: its tell, for good.)
        let m = &mut sim.state.run.mobs[n];
        m.mode = MODE_WINDUP;
        m.timer = 9999;
        m.move_slot = 0;
        sim.idle(2);
        sim.state.fighters[PLAYER].pos.x = fx(16);
        assert_eq!(sim.g(n).state, S::Shield, "its shield is up");
        n
    };
    // A jab: blocked.
    let mut sim = open_street(quiet(), 0);
    let n = shielding(&mut sim);
    let full = sim.g(n).shield_hp;
    let mut stunned = false;
    for frame in 0..20 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        stunned |= sim.g(n).shield_stun > 0;
    }
    assert!(stunned, "shield stun");
    assert_eq!(sim.g(n).percent, Fx::ZERO, "blocked");
    assert!(sim.g(n).shield_hp < full, "its shield took it");
    assert_eq!(sim.state.run.combo, 0, "a blocked hit is no combo");
    // A grab beats it.
    let mut sim = open_street(quiet(), 0);
    let n = shielding(&mut sim);
    sim.g_mut(n).pos.x = fx(16) + Fx::from_ratio(14, 10);
    let mut caught = false;
    for frame in 0..30 {
        sim.tick(inp(0, 0, if frame == 0 { GRAB } else { 0 }));
        caught |= sim.g(n).state == S::Grabbed;
    }
    assert!(caught, "the grab caught it out of its shield");
    // Worn to nothing: broken.
    let mut sim = open_street(quiet(), 0);
    let n = shielding(&mut sim);
    sim.g_mut(n).shield_hp = Fx::from_ratio(1, 10);
    let mut broke = false;
    for frame in 0..20 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        broke |= sim.g(n).state == S::ShieldBreak;
    }
    assert!(broke, "its shield broke");
}

/// A bruiser's super armor: a hit through its tell or its smash's startup deals damage but does not flinch it; out of it, it
/// flinches.
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
        if sim.g(n).percent > Fx::ZERO {
            break;
        }
    }
    let g = *sim.g(n);
    assert!(g.super_armor, "armored through its tell");
    assert!(g.percent > Fx::ZERO, "it took the damage");
    assert_eq!(g.hitstun, 0, "and did not flinch");
    assert_ne!(g.state, S::Hitstun);
    assert_eq!(sim.state.run.mobs[n].mode, MODE_WINDUP);
    // Cooling down: no armor.
    let mut sim = open_street(quiet(), 0);
    let n = grunt(&mut sim, KIND_BRUISER, Vec2::new(fx(17), Fx::ZERO), 0);
    sim.hold(n);
    for frame in 0..20 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        if sim.g(n).percent > Fx::ZERO {
            break;
        }
    }
    assert!(!sim.g(n).super_armor);
    assert!(
        sim.g(n).hitstun > 0 || sim.g(n).launch_pending,
        "it flinches"
    );
}

/// A lobber on high ground throws its shell (its body's own move and projectile, aimed by the run) in an arc onto the player.
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
    assert_ne!(sim.g(n).platform, NONE);
    let mut thrown = false;
    for _ in 0..300 {
        sim.tick(Input::default());
        sim.state.fighters[PLAYER].pos.x = fx(19);
        thrown |= sim
            .state
            .projectiles
            .iter()
            .any(|p| p.active && usize::from(p.owner) == grunt_slot(n));
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
    sim.hold(target);
    // (The shot's lobber, far off and held.)
    let lobber = grunt(&mut sim, KIND_LOBBER, Vec2::new(fx(27), Fx::ZERO), 0);
    sim.hold(lobber);
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..8 {
        sim.tick(Input::default());
    }
    let shell = sim.rc.mobs[usize::from(KIND_LOBBER)].moves[0];
    let slot = grunt_slot(lobber) as u8;
    sim.state.projectiles[0] = sim_core::Projectile {
        active: true,
        owner: slot,
        origin: slot,
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
        let p = sim.state.projectiles[0];
        reflected |= p.active && usize::from(p.owner) == PLAYER;
        if sim.g(target).percent > Fx::ZERO {
            break;
        }
    }
    assert!(reflected, "the cape turned it round");
    assert_eq!(sim.player().percent, Fx::ZERO, "it did not hit the player");
    assert!(sim.g(target).percent > Fx::ZERO, "it hit the grunt");
    assert!(sim.state.run.combo >= 1, "the player's hit");
}

/// A lobber's own shot never hits another grunt (grunts are one side: `combat::rivals`).
#[test]
fn grunts_never_hit_one_another() {
    let mut sim = open_street(quiet(), 0);
    sim.stand(Vec2::new(fx(4), Fx::ZERO));
    let a = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(16), Fx::ZERO), 0);
    let b = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(17), Fx::ZERO), 0);
    sim.hold(b);
    // `a` jabs right into `b`.
    sim.g_mut(a).facing = 1;
    let mut inputs = [Input::default(); MAX_FIGHTERS];
    inputs[0] = Input::default();
    sim.state.run.mobs[a].mode = MODE_ATTACK;
    sim.state.run.mobs[a].move_slot = 0;
    sim.state.run.mobs[a].step = 0;
    sim.state.run.mobs[a].timer = 30;
    for _ in 0..40 {
        sim.tick(Input::default());
    }
    assert!(
        sim.state.fighters[grunt_slot(a)].move_id == MoveId::Jab as u8,
        "it jabbed"
    );
    assert_eq!(sim.g(b).percent, Fx::ZERO, "its own side is never hit");
    assert!(sim_core::combat::rivals(PLAYER, grunt_slot(a)));
    assert!(!sim_core::combat::rivals(grunt_slot(a), grunt_slot(b)));
    let _ = inputs;
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

/// Hazards hurt grunts (the player's hits) and the player: a gas canister hit burns, then bursts; a loose sign falls on whoever is
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
    sim.hold(n);
    let mut lit = false;
    let mut burnt = false;
    for frame in 0..60 {
        sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
        lit |= sim.state.run.props[gas].mode == run::PROP_LIT;
        burnt |= !sim.state.run.mobs[n].active || sim.g(n).percent > Fx::ZERO;
        if !sim.state.run.props[gas].active {
            break;
        }
    }
    assert!(lit, "the hit lit it");
    assert!(!sim.state.run.props[gas].active, "it burst and is gone");
    assert!(burnt, "the blast caught the grunt");
    assert!(sim.state.run.combo >= 1, "the player's hit");
    assert!(sim.player().percent > Fx::ZERO, "and caught the player too");
    // A loose sign falling onto a grunt.
    // (Clear of the start room's awnings, the player well away.)
    let mut sim = open_street(quiet(), 0);
    sim.stand(Vec2::new(fx(3), Fx::ZERO));
    let n = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(16), Fx::ZERO), 0);
    sim.hold(n);
    let sign = put(&mut sim, PROP_KIND_SIGN, Vec2::new(fx(16), fx(5)));
    sim.state.run.props[sign].mode = run::PROP_FALLING;
    let mut broke = false;
    let mut struck = false;
    for _ in 0..90 {
        sim.tick(Input::default());
        broke |= sim.state.run.props[sign].mode == run::PROP_BROKEN;
        struck |= !sim.state.run.mobs[n].active || sim.g(n).percent > Fx::ZERO;
    }
    assert!(struck, "it fell on the grunt");
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
    // (Within a few rooms: a branching fork's two roads and a shop due come first, R2d.)
    assert!(
        first > u32::from(r.event_from) && first <= u32::from(r.event_from) + 5,
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
/// while riding), runs into the grunts in its way (through the versus hit), and parks at the street's end.
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
    sim.stand(Vec2::new(v.x + fx(4), Fx::ZERO));
    sim.idle(30);
    assert_eq!(sim.state.run.van.mode, run::VAN_WAITING);
    let n = grunt(&mut sim, KIND_BRAWLER, Vec2::new(v.x + fx(14), Fx::ZERO), 0);
    sim.hold(n);
    sim.stand(Vec2::new(v.x, v.y + run::VAN_HEIGHT));
    let start = sim.player().pos.x;
    let mut rode = 0;
    let mut sent = false;
    for _ in 0..400 {
        sim.tick(Input::default());
        let v = sim.state.run.van;
        if v.riding {
            rode += 1;
            assert!(sim.player().invuln > 0, "untouchable while riding");
        }
        sent |= !sim.state.run.mobs[n].active || sim.g(n).percent > Fx::ZERO;
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
    assert!(sent, "the grunt was sent flying");
}

/// A branching fork (the owner's R2a answer): its two roads come next, side by side, the alley the fork's lift up; walking out on the
/// high road keeps the alley (with its chest) and drops the low road's room at once, and the other way round; the map marks the road
/// not taken; the run goes on from the room taken.
#[test]
fn a_branching_fork_leads_to_two_rooms() {
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
        let mut fought = 0;
        for (n, i) in inputs.iter().enumerate() {
            if n % 200 == 0 && !sim.state.run.inside && !sim.state.run.over {
                if sim.state.run.gate_closed {
                    sim.clear_mobs();
                    sim.tick(Input::default());
                }
                if !sim.state.run.gate_closed && sim.state.run.ahead().1 > 0 {
                    sim.next_room();
                }
            }
            step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
            fought += sim.state.run.mobs.iter().filter(|m| m.active).count();
            sums.push(sim.state.checksum());
        }
        (sums, sim.state.run.placed, fought)
    };
    let (a, placed, fought) = play(&rc);
    let (b, _, _) = play(&rc);
    assert_eq!(a, b);
    assert!(placed > 8);
    assert!(fought > 0, "grunts were about");
}

// ---- R2c: combat parity, lighter grunts, the following blast zones (sim 131) ------------------------------------------------------

/// Grabs the grunt (or fighter) standing 1.4 in front of the player, optionally pummels once, then throws with `stick`; returns what
/// the victim was launched with on the first frame of its hitstun after the throw: its percent, hitstun, launch speed and tumble.
fn grab_and_throw(
    tick: &mut dyn FnMut(Input),
    victim: &dyn Fn() -> sim_core::Fighter,
    holder: &dyn Fn() -> sim_core::Fighter,
    pummel: bool,
    stick: (i8, i8),
) -> (Fx, u16, Vec2, bool, Fx) {
    tick(inp(0, 0, GRAB));
    let mut held = false;
    for _ in 0..30 {
        tick(Input::default());
        if holder().state == S::Grabbing {
            held = true;
            break;
        }
    }
    assert!(held, "the grab caught it");
    assert_eq!(victim().state, S::Grabbed, "held");
    if pummel {
        let before = victim().percent;
        tick(inp(0, 0, ATTACK));
        let mut pummelled = false;
        for _ in 0..40 {
            tick(Input::default());
            if victim().percent > before {
                pummelled = true;
            }
            if pummelled && holder().state == S::Grabbing && holder().hitlag == 0 {
                break;
            }
        }
        assert!(pummelled, "the pummel landed");
        assert_eq!(victim().state, S::Grabbed, "still held after the pummel");
    }
    let after_pummel = victim().percent;
    tick(inp(stick.0, stick.1, 0));
    for _ in 0..80 {
        tick(Input::default());
        let v = victim();
        if v.state == S::Hitstun && !v.launch_pending && v.hitlag == 0 {
            return (v.percent, v.hitstun, v.kb_vel, v.tumble, after_pummel);
        }
    }
    panic!("the throw never launched");
}

/// R2c: a grab on a grunt is a versus grab (the hold, a pummel, all four throws), and each throw launches it exactly as it launches a
/// versus fighter of the same body and weight: the same damage, hitstun, launch speed and tumble.
#[test]
fn grabs_pummels_and_every_throw_on_a_grunt_match_versus() {
    for char_id in [0u8, 1, 4] {
        for (stick, pummel) in [
            ((127, 0), true),
            ((-127, 0), false),
            ((0, 127), true),
            ((0, -127), false),
        ] {
            // In the run.
            let mut sim = open_street(quiet(), char_id);
            sim.state.fighters[PLAYER].facing = 1;
            let n = grunt(
                &mut sim,
                KIND_BRAWLER,
                Vec2::new(fx(16) + Fx::from_ratio(14, 10), Fx::ZERO),
                0,
            );
            sim.hold(n);
            let weight = sim.g(n).weight.unwrap();
            let body = sim.g(n).char_id;
            let sim = std::cell::RefCell::new(sim);
            let ran = grab_and_throw(
                &mut |i| sim.borrow_mut().tick(i),
                &|| *sim.borrow().g(n),
                &|| *sim.borrow().player(),
                pummel,
                stick,
            );
            // In versus: the same body with that weight.
            let mut content = Content::placeholder();
            content.fighters[usize::from(body)].weight = weight;
            content.stage.blast_top = fx(5000);
            content.stage.blast_left = fx(-5000);
            content.stage.blast_right = fx(5000);
            let mut state = GameState::new_with_active(&content, 1, [char_id, body, 0, 0], 0b0011);
            let stand = |state: &mut GameState, i: usize, x: Fx, facing: i8| {
                let f = &mut state.fighters[i];
                f.pos = Vec2::new(x, Fx::ZERO);
                f.vel = Vec2::ZERO;
                f.platform = collision::standing_on(&content.stage, f.pos);
                f.state = S::Idle;
                f.facing = facing;
            };
            stand(&mut state, 0, fx(-2), 1);
            stand(&mut state, 1, fx(-2) + Fx::from_ratio(14, 10), -1);
            let state = std::cell::RefCell::new(state);
            let vs = grab_and_throw(
                &mut |i| {
                    let mut inputs = [Input::default(); MAX_FIGHTERS];
                    inputs[0] = i;
                    sim_core::step(&mut state.borrow_mut(), &content, &inputs);
                },
                &|| state.borrow().fighters[1],
                &|| state.borrow().fighters[0],
                pummel,
                stick,
            );
            assert_eq!(
                ran, vs,
                "fighter {char_id}, stick {stick:?}: the grunt is thrown as a versus fighter of its weight"
            );
        }
    }
}

/// R2c: a held grunt breaks free like a fighter: with no mashing it is let go when its hold runs out; at a tier that mashes it gets
/// out sooner.
#[test]
fn a_held_grunt_breaks_free_and_mashes_out_by_its_tier() {
    let held_for = |mash: u8| {
        let mut rc = quiet();
        rc.tiers[0].mash = mash;
        let mut sim = open_street(rc, 1);
        sim.state.fighters[PLAYER].facing = 1;
        let n = grunt(
            &mut sim,
            KIND_BRAWLER,
            Vec2::new(fx(16) + Fx::from_ratio(14, 10), Fx::ZERO),
            0,
        );
        sim.hold(n);
        sim.tick(inp(0, 0, GRAB));
        let mut frames = 0;
        let mut was_held = false;
        for _ in 0..400 {
            sim.tick(Input::default());
            if sim.g(n).state == S::Grabbed {
                was_held = true;
                frames += 1;
            } else if was_held {
                break;
            }
        }
        assert!(was_held);
        assert_ne!(sim.g(n).state, S::Grabbed, "let go in the end");
        frames
    };
    let still = held_for(0);
    let mashing = held_for(100);
    assert!(
        mashing < still,
        "mashing gets it out sooner ({mashing} < {still})"
    );
}

/// R2c: grunts tech and DI at their tier's rates. A tumbling grunt falling onto the ground techs (in place, untouchable) when its tier
/// techs, and lies in a knockdown when it does not; DI bends its launch.
#[test]
fn grunts_tech_and_di_by_their_tier() {
    let land = |tech: u8| {
        let mut rc = quiet();
        rc.tiers[0].tech = tech;
        let mut sim = open_street(rc, 0);
        let n = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(20), Fx::ZERO), 0);
        {
            let g = sim.g_mut(n);
            g.pos.y = fx(3);
            g.platform = NONE;
            g.state = S::Hitstun;
            g.hitstun = 60;
            g.tumble = true;
            g.kb_vel = Vec2::new(Fx::ZERO, -Fx::from_ratio(3, 10));
        }
        let mut after = None;
        for _ in 0..60 {
            sim.tick(Input::default());
            let g = *sim.g(n);
            if g.grounded() {
                after = Some(g.state);
                break;
            }
        }
        after.expect("it landed")
    };
    assert_eq!(land(0), S::Knockdown, "no tech at a tier that never techs");
    assert_ne!(land(100), S::Knockdown, "a tech");
    let launched = |di: u8| {
        let mut rc = quiet();
        rc.tiers[0].di = di;
        let mut sim = open_street(rc, 1);
        sim.state.fighters[PLAYER].facing = 1;
        let n = grunt(
            &mut sim,
            KIND_BRAWLER,
            Vec2::new(fx(16) + Fx::from_ratio(12, 10), Fx::ZERO),
            0,
        );
        sim.hold(n);
        sim.g_mut(n).percent = fx(40);
        sim.state.run.mobs[n].last_percent = fx(40);
        sim.tick(Input {
            stick_x: 127,
            stick_y: 0,
            buttons: ATTACK | STRONG,
        });
        for _ in 0..40 {
            sim.tick(Input::default());
            let g = *sim.g(n);
            if g.state == S::Hitstun && !g.launch_pending && g.hitlag == 0 {
                return g.kb_vel;
            }
        }
        panic!("never launched");
    };
    let plain = launched(0);
    let di = launched(100);
    assert_ne!(plain, di, "DI bent the launch");
    assert!(
        di.y * plain.x > plain.y * di.x,
        "survival DI sends it higher: {di:?} vs {plain:?}"
    );
}

/// R2c: weight by tier. Tier 1's grunts are far lighter than any fighter, every kind gets heavier with each tier, and the bruiser
/// is the heaviest kind in every tier; a forward smash at 0 to 20% knocks a tier-1 grunt out of the view.
#[test]
fn grunts_get_heavier_by_tier_and_tier_one_flies_from_a_smash() {
    let rc = RunContent::placeholder();
    let roster = Content::placeholder();
    let lightest = roster.fighters.iter().map(|f| f.weight).min().unwrap();
    for kind in 0..rc.mobs.len() as u8 {
        assert!(
            run::grunt_weight(&rc, kind, 0) + fx(40) < lightest,
            "kind {kind}: much lighter than any fighter at tier 1"
        );
        for tier in 1..rc.tiers.len() as u8 {
            assert!(run::grunt_weight(&rc, kind, tier) > run::grunt_weight(&rc, kind, tier - 1));
        }
    }
    for tier in 0..rc.tiers.len() as u8 {
        let bruiser = run::grunt_weight(&rc, KIND_BRUISER, tier);
        for kind in (0..rc.mobs.len() as u8).filter(|&k| k != KIND_BRUISER) {
            assert!(bruiser > run::grunt_weight(&rc, kind, tier), "tier {tier}");
        }
    }
    // The gauntlets' sprinter's forward smash at 10%: out of the view, a KO.
    let mut sim = open_street(quiet(), 4);
    // (Further into the room: the camera follows, the right zone 18 + 4 ahead.)
    sim.stand(Vec2::new(fx(26), Fx::ZERO));
    sim.state.fighters[PLAYER].facing = 1;
    sim.state.run.cam_y = sim.player().pos.y + sim.rc.rules.cam_lift;
    let n = grunt(
        &mut sim,
        KIND_BRAWLER,
        Vec2::new(fx(26) + Fx::from_ratio(14, 10), Fx::ZERO),
        0,
    );
    sim.hold(n);
    sim.g_mut(n).percent = fx(10);
    sim.state.run.mobs[n].last_percent = fx(10);
    sim.tick(Input {
        stick_x: 127,
        stick_y: 0,
        buttons: ATTACK | STRONG,
    });
    for _ in 0..200 {
        sim.tick(Input::default());
        if !sim.state.run.mobs[n].active {
            break;
        }
    }
    assert!(!sim.state.run.mobs[n].active, "knocked out");
    assert_eq!(sim.state.run.kos, 1);
    assert_eq!(sim.state.run.kills, 1, "the player's kill");
}

/// R2c: the grunts' blast zones follow the view, `ko_margin` outside it on every side (the view is the camera's, its height followed
/// by the sim too); a launched grunt past any of them is knocked out. A grunt launched backward in tumble goes through the gate (a
/// wall for the player) and is knocked out by the zone behind; the player stays inside.
#[test]
fn the_blast_zones_follow_the_view_on_every_side() {
    let rules = RunContent::placeholder().rules;
    let mut sim = RunSim::with(quiet(), 2);
    sim.idle(2);
    let zone = |sim: &RunSim| run::grunt_zone(&sim.state.run, &sim.rc.rules);
    let (l, r, b, t) = zone(&sim);
    let run = &sim.state.run;
    assert_eq!(l, run.cam_x - rules.ko_margin);
    assert_eq!(r, run.cam_x + rules.view_width + rules.ko_margin);
    assert_eq!(b, run.cam_y - rules.view_height / fx(2) - rules.ko_margin);
    assert_eq!(t, run.cam_y + rules.view_height / fx(2) + rules.ko_margin);
    // It follows the camera: on into the next rooms, up a height.
    for _ in 0..3 {
        sim.next_room();
    }
    sim.idle(120);
    let (l2, _, b2, _) = zone(&sim);
    assert!(l2 > l, "the zone moved on with the view");
    assert_eq!(l2, sim.state.run.cam_x - rules.ko_margin);
    assert_eq!(
        b2,
        sim.state.run.cam_y - rules.view_height / fx(2) - rules.ko_margin
    );
    // A launched grunt just past each side is knocked out; just inside, not.
    let (l, r, b, t) = zone(&sim);
    let mid_x = (l + r) / fx(2);
    let mid_y = (b + t) / fx(2);
    let tiny = Fx::from_ratio(1, 4);
    for (at, out) in [
        (Vec2::new(l - tiny, mid_y), true),
        (Vec2::new(r + tiny, mid_y), true),
        (Vec2::new(mid_x, b - tiny), true),
        (Vec2::new(mid_x, t + tiny), true),
        (Vec2::new(l + fx(2), mid_y), false),
        (Vec2::new(mid_x, t - fx(2)), false),
    ] {
        sim.clear_mobs();
        let n = grunt(&mut sim, KIND_BRAWLER, at, 0);
        {
            let g = sim.g_mut(n);
            g.pos = at;
            g.platform = NONE;
            g.state = S::Hitstun;
            g.hitstun = 50;
            g.vel = Vec2::ZERO;
        }
        sim.state.run.mobs[n].credit = 10;
        let kos = sim.state.run.kos;
        // (The zone is checked after the step; place it where the step will leave it.)
        sim.state.fighters[grunt_slot(n)].kb_vel = Vec2::ZERO;
        sim.tick(Input::default());
        assert_eq!(
            sim.state.run.kos > kos,
            out,
            "at {at:?} (zone {l:?}..{r:?}, {b:?}..{t:?})"
        );
    }
    // Through the gate: the player at the wall throws a grunt backward.
    let mut sim = RunSim::with(quiet(), 2);
    sim.clear_mobs();
    for _ in 0..3 {
        sim.next_room();
    }
    sim.clear_mobs();
    let wall = sim.state.run.wall_x;
    sim.stand(Vec2::new(wall + fx(3), Fx::ZERO));
    sim.state.fighters[PLAYER].facing = 1;
    let at = sim.player().pos + Vec2::new(Fx::from_ratio(14, 10), Fx::ZERO);
    let n = grunt(&mut sim, KIND_BRAWLER, at, 0);
    sim.hold(n);
    sim.g_mut(n).percent = fx(60);
    sim.state.run.mobs[n].last_percent = fx(60);
    sim.tick(inp(0, 0, GRAB));
    let mut passed = false;
    for frame in 0..200 {
        let i = if frame == 12 {
            inp(-127, 0, 0)
        } else {
            Input::default()
        };
        sim.tick(i);
        if sim.state.run.mobs[n].active && sim.g(n).pos.x < wall {
            passed = true;
        }
        assert!(
            sim.player().pos.x > wall,
            "the player stays inside the gate"
        );
        if !sim.state.run.mobs[n].active {
            break;
        }
    }
    assert!(passed, "the grunt went through the gate");
    assert!(
        !sim.state.run.mobs[n].active,
        "and was knocked out behind it"
    );
    assert!(sim.state.run.kos >= 1);
    // The player's own rules stand (R1b): the gate stops them, and their zones are outside the grunts' (they never die first).
    let s = &sim.world.stage;
    let (gl, gr, gb, gt) = zone(&sim);
    assert!(s.blast_left < gl && s.blast_right >= gr && s.blast_bottom <= gb && s.blast_top >= gt);
}

/// R2c: a run's every new piece of state is in the checksum: a grunt's fighter (its slot after the players), its weight, damage
/// scale and super armor.
#[test]
fn the_checksum_sees_the_grunts_fighters() {
    let mut sim = open_street(quiet(), 0);
    let n = grunt(&mut sim, KIND_BRAWLER, Vec2::new(fx(20), Fx::ZERO), 1);
    let base = sim.state;
    let c = base.checksum();
    type Edit = fn(&mut GameState, usize);
    let edits: Vec<(&str, Edit)> = vec![
        ("pos", |s, i| s.fighters[i].pos.x += Fx::from_raw(1)),
        ("percent", |s, i| s.fighters[i].percent += Fx::from_raw(1)),
        ("weight", |s, i| s.fighters[i].weight = Some(fx(3))),
        ("damage_scale", |s, i| s.fighters[i].damage_scale += 1),
        ("super_armor", |s, i| s.fighters[i].super_armor ^= true),
        ("last slot", |s, _| {
            s.fighters[MAX_BODIES - 1].pos.y += Fx::from_raw(1)
        }),
    ];
    for (name, edit) in edits {
        let mut s = base;
        edit(&mut s, grunt_slot(n));
        assert_ne!(s.checksum(), c, "{name} is not in the checksum");
    }
}

/// R2c: bodies push each other on the ground (versus's rule), but in a run a push never shoves anyone through a gate standing on that
/// ground (the arena's, or the wall behind): a grunt crowding the player against a shut gate leaves it inside.
#[test]
fn a_push_never_goes_through_a_gate() {
    let mut sim = RunSim::with(only(&["yard", "shop"], (0, 1, 0)), 4);
    let slot = *sim.state.run.slot_of(1).unwrap();
    sim.stand(Vec2::new(slot.x + fx(3), Fx::ZERO));
    sim.state.fighters[PLAYER].invuln = 30;
    sim.tick(Input::default());
    assert!(sim.state.run.gate_closed);
    sim.clear_mobs();
    let gate = sim.state.run.gate_x;
    sim.stand(Vec2::new(gate - fx(1), Fx::ZERO));
    let at = sim.player().pos - Vec2::new(Fx::HALF, Fx::ZERO);
    let n = sim.add_mob(KIND_BRAWLER, at);
    sim.hold(n);
    for _ in 0..60 {
        sim.g_mut(n).pos.x = sim.player().pos.x - Fx::HALF;
        sim.tick(inp(127, 0, 0));
        assert!(sim.player().pos.x < gate, "pushed through the gate");
    }
}

// ---- R2d: verticality (decks, climbs and drops, the vertical gate, interiors; sim 137) --------------------------------------------

/// The current room's slot.
fn here(sim: &RunSim) -> run::RoomSlot {
    *sim.state.run.slot_of(sim.state.run.room).unwrap()
}

fn name_of(sim: &RunSim, s: &run::RoomSlot) -> String {
    sim.rc.rooms[usize::from(s.room)].name.clone()
}

/// The decks over many seeds: every room laid begins and ends inside a deck's band, its decks recorded in its slot; the deck only
/// changes through a room that climbs or drops, or a deck fork's road; a fork's choice holds for at least a stretch
/// (`deck_rooms_min` rooms); and the run visits all three decks, with climbs, drops, shafts, interiors and deck forks along the way.
#[test]
fn the_deck_plan_over_many_seeds() {
    let rc = quiet();
    let r = rc.rules;
    let mut decks_seen = [0u32; 3];
    let mut names: Vec<String> = Vec::new();
    let mut forks_taken = 0;
    for seed in 0..16u64 {
        let mut sim = RunSim::with(rc.clone(), seed);
        let mut deck = here(&sim).deck_out;
        let mut since_fork: Option<u32> = None;
        for step in 0..60u64 {
            let forked = sim.state.run.branches().is_some();
            sim.take_road(((seed + step) % 2) as usize);
            let s = here(&sim);
            let room = &sim.rc.rooms[usize::from(s.room)];
            names.push(room.name.clone());
            for (y, d) in [(s.y, s.deck_in), (s.y_end, s.deck_out)] {
                // (A treasure alley's high road starts between two decks' bands, on its fork's deck; every other room is inside a
                // deck's band.)
                if room.kind == run::ROOM_ALLEY && y == s.y {
                    assert_eq!(d, s.deck_out, "seed {seed}: {}", room.name);
                    continue;
                }
                assert_eq!(run::deck_at(&sim.rc, y), d, "seed {seed}: {}", room.name);
                assert!(
                    run::in_band(&sim.rc, d, y),
                    "seed {seed}: {} at {y:?} is outside deck {d}'s band",
                    room.name
                );
            }
            decks_seen[usize::from(s.deck_out)] += 1;
            if s.deck_in != deck {
                // (Entered on another deck than the last room left on: only a deck fork's road does that.)
                assert!(forked, "seed {seed}: {} jumped decks", room.name);
            }
            if forked && s.deck_in != deck || forked && s.deck_out != deck {
                forks_taken += 1;
                since_fork = Some(0);
            } else if let Some(n) = since_fork.as_mut() {
                *n += 1;
                if s.deck_out != deck {
                    assert!(
                        *n > u32::from(r.deck_rooms_min),
                        "seed {seed}: a fork's choice lasted only {n} rooms"
                    );
                    since_fork = None;
                }
            }
            if s.vertical() {
                assert_ne!(
                    room.deck_change, 0,
                    "seed {seed}: {} changed deck",
                    room.name
                );
            }
            deck = s.deck_out;
        }
    }
    println!("rooms on each deck: {decks_seen:?}; deck forks taken {forks_taken}");
    assert!(
        decks_seen.iter().all(|&n| n > 50),
        "every deck: {decks_seen:?}"
    );
    assert!(forks_taken > 5, "deck forks taken: {forks_taken}");
    for want in [
        "escape_climb",
        "terrace_climb",
        "great_climb",
        "balcony_drop",
        "awning_drop",
        "light_well",
        "elevator_shaft",
        "office",
        "stairwell",
        "rooftop_bar",
        "catwalks",
        "cranes",
        "water_towers",
        "sky_bridge",
        "under_bridge",
    ] {
        assert!(names.iter().any(|n| n == want), "{want} never came");
    }
}

/// A fall from a deck lands on the one below (never a death, but in a true pit): over a skyline gap the player falls past the
/// catwalk's level onto the roofs a deck down, its lives untouched.
#[test]
fn a_fall_from_a_deck_lands_on_the_one_below() {
    let mut sim = RunSim::with(only(&["catwalks", "shop"], (1, 0, 0)), 3);
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(name_of(&sim, &slot), "catwalks");
    sim.next_room();
    sim.clear_mobs();
    // (Under the first gap's catwalk, where no sill is: 12.5 into the room, or as far from its end when mirrored.)
    let x = if slot.mirror {
        slot.end - Fx::from_ratio(25, 2)
    } else {
        slot.x + Fx::from_ratio(25, 2)
    };
    sim.place(Vec2::new(x, slot.y - fx(1)));
    sim.idle(120);
    let f = sim.player();
    assert_eq!(f.stocks, 3, "a fall, not a death");
    assert!(f.grounded(), "landed");
    assert_eq!(f.pos.y, slot.y - fx(14), "on the roofs below");
    assert_eq!(
        run::deck_at(&sim.rc, f.pos.y) + 1,
        run::deck_at(&sim.rc, slot.y),
        "the deck below"
    );
    // The view and the blast zone reach down to it.
    let (bottom, _) = sim.state.run.blast_y(&sim.rc.rules);
    assert!(bottom < f.pos.y);
}

/// The vertical camera and the vertical gate: in a climb the view's height follows the player up (faster than on the flat); once
/// the player has left the climb into the next room, nothing before the climb is kept (the wall stands at its left edge) and a lid
/// closes over its top, so a drop back over it lands on the lid, not down the building face.
#[test]
fn a_climb_scrolls_the_view_and_is_a_gate_once_left() {
    let mut rc = only(&["escape_climb", "street", "shop"], (1, 0, 0));
    rc.rules.deck_rooms_min = 1;
    rc.rules.deck_rooms_max = 1;
    rc.rules.deck_fork_chance = 0;
    let rules = rc.rules;
    for seed in 0..30u64 {
        let mut sim = RunSim::with(rc.clone(), seed);
        let mut climb = None;
        for _ in 0..12 {
            sim.next_room();
            let s = here(&sim);
            if s.vertical() && s.y_end > s.y {
                climb = Some(s);
                break;
            }
        }
        let Some(climb) = climb else {
            continue;
        };
        sim.clear_mobs();
        // At its foot, then at its top: the view rises most of a deck in 40 frames.
        let foot = if climb.mirror {
            climb.end - fx(5)
        } else {
            climb.x + fx(5)
        };
        sim.stand(Vec2::new(foot, climb.y));
        sim.idle(90);
        let low = sim.state.run.cam_y;
        let top = if climb.mirror {
            climb.x + fx(3)
        } else {
            climb.end - fx(3)
        };
        sim.stand(Vec2::new(top, climb.y_end));
        sim.idle(40);
        let risen = sim.state.run.cam_y - low;
        assert!(risen >= fx(13), "the view rose {risen:?} in 40 frames");
        // (On the flat the same rise is slower: its ease is `cam_y_ease`, not `cam_y_ease_tall`.)
        assert!(rules.cam_y_ease_tall > rules.cam_y_ease);
        // Into the next room: the climb is the gate.
        sim.next_room();
        let run = &sim.state.run;
        let c = *run.slot_of(climb.seq).expect("the climb is kept");
        assert!(c.lid, "a lid over the climb");
        assert_eq!(run.wall_x, climb.x, "nothing before the climb is kept");
        assert!(run.slots.iter().all(|s| !s.live || s.seq >= climb.seq));
        assert!(
            run.platforms.iter().any(|p| p.live
                && !p.pass_through
                && p.left == climb.x
                && p.right == climb.end
                && p.y == climb.y_end),
            "the lid is a solid floor across the climb's top"
        );
        let (bottom, _) = run.blast_y(&rules);
        assert!(
            bottom >= climb.y_end + rules.blast_bottom,
            "the blast line rose with it"
        );
        // Dropped back over the climb, the player lands on the lid.
        let mid = (climb.x + climb.end) / fx(2);
        sim.place(Vec2::new(mid, climb.y_end + fx(3)));
        sim.idle(60);
        assert_eq!(sim.player().pos.y, climb.y_end, "on the lid");
        assert!(sim.player().grounded());
        return;
    }
    panic!("no climb came");
}

/// A drop: the view follows the player down a deck, and the rooms before the drop are gone once it is left (no lid: its back edge
/// is its top, and nothing is left above it to go back to).
#[test]
fn a_drop_scrolls_the_view_down() {
    let mut rc = only(&["balcony_drop", "street", "shop"], (1, 0, 0));
    rc.rules.deck_rooms_min = 1;
    rc.rules.deck_rooms_max = 1;
    rc.rules.deck_fork_chance = 0;
    for seed in 0..30u64 {
        let mut sim = RunSim::with(rc.clone(), seed);
        let mut drop = None;
        for _ in 0..12 {
            sim.next_room();
            let s = here(&sim);
            if s.vertical() && s.y_end < s.y {
                drop = Some(s);
                break;
            }
        }
        let Some(drop) = drop else {
            continue;
        };
        sim.clear_mobs();
        let (top, foot) = if drop.mirror {
            (drop.end - fx(3), drop.x + fx(3))
        } else {
            (drop.x + fx(3), drop.end - fx(3))
        };
        sim.stand(Vec2::new(top, drop.y));
        sim.idle(90);
        let high = sim.state.run.cam_y;
        sim.stand(Vec2::new(foot, drop.y_end));
        sim.idle(40);
        assert!(high - sim.state.run.cam_y >= fx(13), "the view came down");
        sim.next_room();
        let run = &sim.state.run;
        assert_eq!(run.wall_x, drop.x, "nothing before the drop is kept");
        assert!(!run.slot_of(drop.seq).unwrap().lid, "no lid over a drop");
        return;
    }
    panic!("no drop came");
}

/// An interior's ceiling cuts the jumps short: a full hop and a midair jump under the office's ceiling stay under it, while the
/// same jumps outside its window rise well above that height.
#[test]
fn an_interior_ceiling_cuts_the_jumps_short() {
    let mut sim = RunSim::with(only(&["office", "shop"], (1, 0, 0)), 2);
    let slot = *sim.state.run.slot_of(1).unwrap();
    assert_eq!(name_of(&sim, &slot), "office");
    assert!(sim.rc.rooms[usize::from(slot.room)].indoor);
    sim.next_room();
    sim.clear_mobs();
    let at = |local: i32| {
        if slot.mirror {
            slot.end - fx(local)
        } else {
            slot.x + fx(local)
        }
    };
    let peak = |sim: &mut RunSim, x: Fx| {
        sim.stand(Vec2::new(x, slot.y));
        let mut top = slot.y;
        for n in 0..70 {
            let jump = if n < 12 || (20..24).contains(&n) {
                JUMP
            } else {
                0
            };
            sim.tick(inp(0, 0, jump));
            top = top.max(sim.player().pos.y);
        }
        top - slot.y
    };
    let inside = peak(&mut sim, at(13));
    let outside = peak(&mut sim, at(2));
    let ceiling = fx(5);
    assert!(inside < ceiling - fx(1), "under the ceiling: {inside:?}");
    assert!(outside > ceiling, "outside: {outside:?}");
}

/// Every new piece of the run's state is in the checksum, and the run's ledge table (16, more than versus's 8) is too; in versus
/// the extra ledges are never hashed.
#[test]
fn the_checksum_sees_the_decks() {
    let sim = RunSim::new(3);
    let base = sim.state;
    let c = base.checksum();
    type Edit = fn(&mut GameState);
    let edits: Vec<(&str, Edit)> = vec![
        ("slots.lo", |s| s.run.slots[0].lo += Fx::from_raw(1)),
        ("slots.hi", |s| s.run.slots[0].hi += Fx::from_raw(1)),
        ("slots.deck_in", |s| s.run.slots[0].deck_in ^= 1),
        ("slots.deck_out", |s| s.run.slots[0].deck_out ^= 1),
        ("slots.lid", |s| s.run.slots[0].lid ^= true),
        ("plan_deck", |s| s.run.plan_deck ^= 1),
        ("plan_left", |s| s.run.plan_left += 1),
        ("ledge_owner beyond versus's", |s| {
            s.ledge_owner[sim_core::state::LEDGE_SLOTS - 1] = 0
        }),
    ];
    for (name, edit) in edits {
        let mut s = base;
        edit(&mut s);
        assert_ne!(s.checksum(), c, "{name} is not in the checksum");
    }
    // Versus: the run's extra ledges are never touched nor hashed.
    let world = Content::placeholder();
    let mut vs = GameState::new(&world, 5, [0, 1, 0, 1]);
    let sum = vs.checksum();
    vs.ledge_owner[sim_core::state::LEDGE_SLOTS - 1] = 0;
    assert_eq!(vs.checksum(), sum);
    let mut vs = GameState::new(&world, 5, [0, 1, 0, 1]);
    for _ in 0..300 {
        sim_core::step(&mut vs, &world, &[inp(60, -127, JUMP); MAX_FIGHTERS]);
    }
    assert!(vs.ledge_owner[sim_core::content::MAX_LEDGES..]
        .iter()
        .all(|&o| o == NONE));
}

/// Runs through the decks replay exactly: the same seed and inputs give the same checksums, frame by frame, through climbs, drops,
/// forks and lids.
#[test]
fn a_run_through_the_decks_replays() {
    let mut rc = RunContent::placeholder();
    rc.rules.deck_rooms_min = 1;
    rc.rules.deck_rooms_max = 2;
    let inputs = rightward_inputs(5, 2400);
    let play = |rc: &RunContent| {
        let mut sim = RunSim::with(rc.clone(), 4);
        let mut sums = Vec::new();
        let mut vertical = 0;
        for (n, i) in inputs.iter().enumerate() {
            if n % 120 == 0 && !sim.state.run.inside && !sim.state.run.over {
                if sim.state.run.gate_closed {
                    sim.clear_mobs();
                    sim.tick(Input::default());
                }
                if !sim.state.run.gate_closed && sim.state.run.ahead().1 > 0 {
                    sim.next_room();
                    vertical += usize::from(here(&sim).vertical());
                }
            }
            step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
            sums.push(sim.state.checksum());
        }
        (sums, vertical)
    };
    let (a, vertical) = play(&rc);
    let (b, _) = play(&rc);
    assert_eq!(a, b);
    assert!(vertical >= 2, "through {vertical} climbs and drops");
}

// ---- R2e, the owner's note A: a small cap on grunts at once, rising with the tier; a stagger between doorways ------------------

fn alive(sim: &RunSim) -> usize {
    sim.state.run.mobs.iter().filter(|m| m.active).count()
}

/// The cap per tier is data (tier 1 two, tier 2 three, then one more a tier), and over many seeds and rooms the grunts alive never
/// pass the current tier's cap, which is reached.
#[test]
fn the_grunts_alive_never_pass_the_tiers_cap() {
    let rc = RunContent::placeholder();
    let caps: Vec<u8> = rc.tiers.iter().map(|t| t.cap).collect();
    assert_eq!(caps, [2, 3, 4, 5, 6]);
    assert!(caps.iter().all(|&c| usize::from(c) <= MAX_MOBS));
    let mut reached = [false; 5];
    for seed in 0..6u64 {
        let mut sim = RunSim::new(seed);
        for _ in 0..30 {
            for _ in 0..90 {
                sim.state.fighters[PLAYER].invuln = 5;
                sim.tick(Input::default());
                let tier = usize::from(run::tier_of(&sim.rc, sim.state.run.room));
                let cap = usize::from(sim.rc.tiers[tier].cap);
                assert!(
                    alive(&sim) <= cap,
                    "seed {seed}: {} alive, cap {cap}",
                    alive(&sim)
                );
                reached[tier] |= alive(&sim) == cap;
            }
            sim.next_room();
        }
    }
    assert!(reached[0] && reached[1], "the cap is reached: {reached:?}");
}

/// Grunts come out of doors one at a time in a room, `emerge_gap` frames apart at least; and a spot past the cap waits its turn,
/// coming out once a grunt is beaten.
#[test]
fn spots_wait_their_turn_and_doorways_are_staggered() {
    let rc = RunContent::placeholder();
    let gap = u32::from(rc.rules.emerge_gap);
    assert!(gap >= 50, "about a second between two doorways");
    let mut stagger_seen = 0;
    for seed in 0..8u64 {
        let mut sim = RunSim::new(seed);
        let emerge = u32::from(rc.rules.emerge_frames);
        // (Each grunt's own frame of coming out: its emergence's timer tells how long ago.)
        let mut out: Vec<(usize, u32, u32)> = Vec::new();
        for _ in 0..12 {
            for _ in 0..120 {
                sim.state.fighters[PLAYER].invuln = 5;
                sim.tick(Input::default());
                for n in 0..MAX_MOBS {
                    let m = sim.state.run.mobs[n];
                    if !m.active || m.mode != run::MODE_EMERGE {
                        continue;
                    }
                    let at = sim.state.frame - (emerge - u32::from(m.timer));
                    if !out.iter().any(|&(k, _, f)| k == n && f == at) {
                        out.push((n, m.room, at));
                    }
                }
            }
            sim.next_room();
        }
        out.sort_by_key(|&(_, room, at)| (room, at));
        for w in out.windows(2) {
            if w[0].1 == w[1].1 {
                assert!(
                    w[1].2 - w[0].2 >= gap,
                    "seed {seed}: two grunts out of room {}'s doors {} frames apart",
                    w[0].1,
                    w[1].2 - w[0].2
                );
                stagger_seen += 1;
            }
        }
    }
    assert!(stagger_seen > 10, "{stagger_seen} staggered pairs");
    // A room with more spots than the cap: the cap comes out, the rest wait; beat one and the next comes.
    let mut rc = only(&["avenue", "shop"], (1, 0, 0));
    let avenue = rc.rooms.iter().position(|r| r.name == "avenue").unwrap();
    rc.rooms[avenue].min_tier = 0;
    rc.rooms[avenue].spawns = RunContent::placeholder()
        .rooms
        .into_iter()
        .find(|r| r.name == "avenue")
        .unwrap()
        .spawns;
    for s in rc.rooms[avenue].spawns.iter_mut() {
        s.kind = 0;
    }
    let mut sim = RunSim::with(rc, 2);
    sim.next_room();
    let cap = usize::from(sim.rc.tiers[0].cap);
    for _ in 0..400 {
        sim.state.fighters[PLAYER].invuln = 5;
        for n in 0..MAX_MOBS {
            if sim.state.run.mobs[n].active {
                sim.hold(n);
            }
        }
        sim.tick(Input::default());
    }
    let slot = here(&sim);
    assert_eq!(alive(&sim), cap, "the cap is out");
    let waiting =
        (slot.spawned as u32 | !((1u32 << sim.rc.rooms[avenue].spawns.len()) - 1)).count_zeros();
    assert!(waiting > 0, "spots wait");
    let n = (0..MAX_MOBS)
        .find(|&n| sim.state.run.mobs[n].active)
        .unwrap();
    run::remove_grunt(&mut sim.state, &sim.world, n);
    sim.idle(gap + 2);
    assert_eq!(alive(&sim), cap, "the next spot came out");
    assert_eq!(
        here(&sim).spawned.count_ones(),
        slot.spawned.count_ones() + 1
    );
}

/// An arena comes in waves under the cap and its gate opens only when every grunt of it, every wave, is beaten.
#[test]
fn an_arena_clears_in_waves() {
    let mut rc = only(&["yard", "shop"], (0, 1, 0));
    let yard = rc.rooms.iter().position(|r| r.name == "yard").unwrap();
    for s in rc.rooms[yard].spawns.iter_mut() {
        s.kind = 0;
    }
    let spots = rc.rooms[yard].spawns.len();
    let mut sim = RunSim::with(rc, 4);
    let cap = usize::from(sim.rc.tiers[0].cap);
    assert!(spots > cap);
    let slot = *sim.state.run.slot_of(1).unwrap();
    sim.stand(Vec2::new(slot.x + fx(3), slot.y));
    sim.tick(Input::default());
    assert!(sim.state.run.gate_closed);
    let mut beaten = 0;
    let mut waves_max = 0;
    for _ in 0..3000 {
        sim.state.fighters[PLAYER].invuln = 5;
        sim.tick(Input::default());
        assert!(alive(&sim) <= cap);
        waves_max = waves_max.max(alive(&sim));
        if !sim.state.run.gate_closed {
            break;
        }
        // (Beat whoever is out once it is out of its door.)
        for n in 0..MAX_MOBS {
            let m = sim.state.run.mobs[n];
            if m.active && m.mode != run::MODE_EMERGE {
                run::remove_grunt(&mut sim.state, &sim.world, n);
                beaten += 1;
            }
        }
    }
    assert!(!sim.state.run.gate_closed, "the arena opened");
    assert_eq!(beaten, spots, "every wave beaten first");
    assert!(waves_max >= 1 && waves_max <= cap);
    assert_eq!(sim.state.run.cleared, 1);
}
