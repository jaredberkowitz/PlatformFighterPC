//! The single-player run (R1, sim 118): the streamed stage, the camera's push, lives, mobs, coins and the combo, and determinism.

mod common;

use common::inp;
use sim_core::content::Platform;
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::{ATTACK, SHIELD};
use sim_core::run::data::{Chunk, MobKind};
use sim_core::run::{
    self, new_run, step_run, Mob, RunContent, MAX_MOBS, MODE_ATTACK, MODE_BURST, MODE_MOVE,
    MODE_WINDUP, PLAYER, RING,
};
use sim_core::state::{FighterState as S, DRAW, NONE};
use sim_core::{Content, Fx, GameState, Input, Rng, Vec2, MAX_FIGHTERS};

fn fx(n: i32) -> Fx {
    Fx::from_int(n)
}

struct RunSim {
    world: Content,
    rc: RunContent,
    state: GameState,
}

impl RunSim {
    fn with(rc: RunContent, seed: u64) -> RunSim {
        let mut world = Content::placeholder();
        let state = new_run(&mut world, &rc, seed, 0);
        RunSim { world, rc, state }
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

    /// Puts the player in the air at `at`, still.
    fn place(&mut self, at: Vec2) {
        let f = &mut self.state.fighters[PLAYER];
        f.pos = at;
        f.vel = Vec2::ZERO;
        f.platform = NONE;
        f.state = S::Airborne;
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
            platform: sim_core::collision::standing_on(&self.world.stage, at),
            stamina: k.stamina,
            ..Mob::default()
        };
        slot
    }
}

fn chunk(
    name: &str,
    width: i32,
    platforms: Vec<Platform>,
    ledges: Vec<sim_core::content::Ledge>,
) -> Chunk {
    Chunk {
        name: name.to_string(),
        width: fx(width),
        platforms,
        ledges,
        spawns: vec![],
    }
}

fn block(left: i32, right: i32, top: i32) -> Platform {
    Platform {
        left: fx(left),
        right: fx(right),
        y: fx(top),
        bottom: fx(-24),
        pass_through: false,
    }
}

/// The built-in chunks without their mobs, in a fixed order: the street then the gap, over and over (two chunks alternate).
fn street_and_gap() -> RunContent {
    let mut rc = RunContent::placeholder();
    rc.chunks.truncate(2);
    for c in rc.chunks.iter_mut() {
        c.spawns.clear();
    }
    rc
}

// ---- The stage streams ------------------------------------------------------------------------------------------------------------

/// Sweeping the camera across a long run: the chunks in play always join up and cover the view and beyond, never more than the
/// ring, and every platform of every chunk in play is in the stage (none lost to a full pool).
#[test]
fn chunks_stream_and_always_cover_the_view() {
    for seed in 0..6u64 {
        let mut sim = RunSim::new(seed);
        let view = sim.rc.rules.view_width;
        for _ in 0..4000 {
            sim.state.run.cam_x += Fx::from_ratio(1, 2);
            // (Keep the fighter in the air above the view, out of everything.)
            let at = Vec2::new(sim.state.run.cam_x + fx(12), fx(15));
            sim.place(at);
            sim.state.fighters[PLAYER].invuln = 10;
            sim.tick(Input::default());
            let run = &sim.state.run;
            let mut live: Vec<_> = run.slots.iter().filter(|s| s.live).collect();
            assert!(live.len() <= RING);
            live.sort_by_key(|s| s.x);
            for w in live.windows(2) {
                assert_eq!(w[0].end, w[1].x, "seed {seed}: chunks do not join");
            }
            assert!(
                live[0].x <= run.cam_x,
                "seed {seed}: nothing under the left edge"
            );
            assert!(
                live.last().unwrap().end >= run.cam_x + view,
                "seed {seed}: the view runs out"
            );
            for s in &live {
                let c = &sim.rc.chunks[usize::from(s.chunk)];
                for p in &c.platforms {
                    let (l, r) = (s.x + p.left, s.x + p.right);
                    let found = run.platforms.iter().any(|e| {
                        e.live
                            && e.y == p.y
                            && e.pass_through == p.pass_through
                            && e.left <= l
                            && e.right >= r
                    });
                    // (A platform wholly behind the screen may already be gone.)
                    assert!(
                        found || r < run.cam_x - sim.rc.rules.blast_margin,
                        "seed {seed}: a platform of chunk {} is missing",
                        c.name
                    );
                }
            }
        }
        assert!(sim.state.run.placed > 50);
    }
}

/// Every chunk comes round once a lap, in an order the seed decides.
#[test]
fn chunks_come_round_in_a_seeded_order() {
    let order = |seed: u64| {
        let mut sim = RunSim::new(seed);
        // (The first chunks are laid as the run starts.)
        let mut first: Vec<_> = sim.state.run.slots.iter().filter(|s| s.live).collect();
        first.sort_by_key(|s| s.x);
        let mut seen: Vec<u8> = first.iter().map(|s| s.chunk).collect();
        let mut last = sim.state.run.placed;
        for _ in 0..3000 {
            sim.state.run.cam_x += Fx::from_ratio(1, 2);
            let at = Vec2::new(sim.state.run.cam_x + fx(12), fx(15));
            sim.place(at);
            sim.state.fighters[PLAYER].invuln = 10;
            sim.tick(Input::default());
            if sim.state.run.placed != last {
                last = sim.state.run.placed;
                seen.push(sim.state.run.last_chunk);
            }
        }
        seen
    };
    let a = order(1);
    assert_eq!(a, order(1));
    assert_eq!(a[0], 0, "a run starts on the first chunk");
    assert!(
        a.windows(2).all(|w| w[0] != w[1]),
        "never the same chunk twice running: {a:?}"
    );
    for c in 0..5u8 {
        assert!(a.contains(&c), "chunk {c} never came: {a:?}");
    }
    assert_ne!(a, order(2), "another seed, another order");
}

/// Ground that carries on across a chunk seam is one floor: a fighter dashes across without leaving the ground.
#[test]
fn ground_joins_across_a_chunk_seam() {
    let mut sim = RunSim::with(street_and_gap(), 3);
    let seam = sim
        .state
        .run
        .slots
        .iter()
        .filter(|s| s.live)
        .map(|s| s.end)
        .min()
        .unwrap();
    assert_eq!(seam, fx(28));
    let floor = sim.player().platform;
    let mut crossed = false;
    for frame in 0..200 {
        sim.tick(inp(127, 0, 0));
        let f = sim.player();
        assert!(
            f.grounded(),
            "frame {frame}: left the ground at x {:?} in {:?}",
            f.pos.x,
            f.state
        );
        assert_eq!(f.platform, floor, "the floor is one platform");
        if f.pos.x > seam + fx(2) {
            crossed = true;
            break;
        }
    }
    assert!(crossed);
}

/// The ledges and platforms of a chunk laid later work as any stage's: a fighter catches the gap's ledge and lands on its stepping
/// stone.
#[test]
fn ledges_and_platforms_work_in_a_later_chunk() {
    let mut sim = RunSim::with(street_and_gap(), 3);
    // The gap chunk starts at 28: its pit is 38 to 44, its ledges at 38 (facing right) and 44, its stepping stone 39.5-42.5 at 4.5.
    let ledge_x = fx(38);
    let ledge = sim
        .world
        .stage
        .ledges
        .iter()
        .position(|l| l.x == ledge_x && l.side == 1)
        .expect("the gap's ledge is in the stage");
    let p = sim.world.fighters[0];
    sim.state.run.cam_x = fx(20);
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
    assert!(
        caught,
        "the ledge was not caught: {:?} at {:?}",
        sim.player().state,
        sim.player().pos
    );
    assert_eq!(sim.player().ledge, ledge as i8);
    assert_eq!(sim.state.ledge_owner[ledge], PLAYER as i8);

    // Dropped above the stepping stone, it lands on it.
    sim.place(Vec2::new(fx(41), fx(8)));
    sim.clear_mobs();
    for _ in 0..90 {
        sim.tick(Input::default());
    }
    let f = sim.player();
    assert!(f.grounded());
    let stone = sim.world.stage.platforms[f.platform as usize];
    assert!(stone.pass_through && stone.y == Fx::from_ratio(45, 10));
}

// ---- The camera, the push and lives -----------------------------------------------------------------------------------------------

#[test]
fn the_camera_creeps_and_follows_but_never_goes_back() {
    let mut sim = RunSim::with(street_and_gap(), 1);
    let start = sim.state.run.cam_x;
    sim.idle(60);
    let crept = sim.state.run.cam_x;
    assert!(crept > start, "it creeps by itself");
    assert_eq!(crept - start, sim.rc.rules.creep.mul_int(60));
    // Running right, it follows, keeping the fighter `follow` across the view.
    for _ in 0..90 {
        sim.tick(inp(127, 0, 0));
    }
    let r = &sim.rc.rules;
    let held = sim.player().pos.x - sim.state.run.cam_x;
    assert!(
        held <= r.view_width * r.follow + Fx::ONE,
        "held at {held:?}"
    );
    // Walking back, it stays.
    let before = sim.state.run.cam_x;
    for _ in 0..40 {
        sim.tick(inp(-127, 0, 0));
        assert!(sim.state.run.cam_x >= before);
    }
}

#[test]
fn the_left_edge_pushes_a_fighter_standing_still() {
    let mut sim = RunSim::with(street_and_gap(), 1);
    let x0 = sim.player().pos.x;
    sim.state.run.cam_x = x0 - fx(1);
    for _ in 0..240 {
        sim.tick(Input::default());
        let limit = sim.state.run.cam_x + sim.rc.rules.push_margin;
        assert!(
            sim.player().pos.x >= limit - sim.rc.rules.push_speed,
            "left behind"
        );
    }
    assert!(sim.player().pos.x > x0 + fx(3), "pushed along");
    assert_eq!(sim.player().stocks, 3, "pushed, not hurt");
    assert!(sim.player().grounded());
}

#[test]
fn pinned_against_a_wall_by_the_edge_costs_a_life() {
    let mut rc = street_and_gap();
    // The first chunk: a floor, then a building too tall to jump.
    rc.chunks[0] = chunk("wall", 30, vec![block(0, 16, 0), block(16, 30, 12)], vec![]);
    let mut sim = RunSim::with(rc, 1);
    // (The first chunk starts at -4: the wall's face is at 12.)
    sim.place(Vec2::new(fx(11), Fx::ZERO));
    sim.idle(30);
    assert!(sim.player().grounded());
    let wall = fx(12);
    sim.state.run.cam_x = wall - fx(2);
    let mut lost = false;
    for _ in 0..200 {
        sim.tick(Input::default());
        if sim.player().stocks == 2 {
            lost = true;
            break;
        }
        assert!(sim.player().pos.x < wall, "through the wall");
    }
    assert!(lost, "never crushed");
    assert_eq!(sim.state.run.crushes, 1);
    // Back on the revival platform, ahead of the edge.
    assert_eq!(sim.player().state, S::Respawn);
    assert!(sim.player().pos.x > sim.state.run.cam_x + fx(5));
}

#[test]
fn a_fall_into_a_pit_costs_a_life() {
    let mut sim = RunSim::with(street_and_gap(), 1);
    sim.state.run.cam_x = fx(25);
    sim.place(Vec2::new(fx(41), fx(-3)));
    // (Holding down declines the ledges.)
    for _ in 0..240 {
        sim.tick(inp(0, -127, 0));
        if sim.player().stocks < 3 {
            break;
        }
    }
    assert_eq!(sim.player().stocks, 2);
    assert_eq!(sim.state.run.crushes, 0);
}

#[test]
fn leaving_the_screen_to_the_left_costs_a_life_and_the_last_life_ends_the_run() {
    let mut sim = RunSim::with(street_and_gap(), 1);
    sim.state.run.cam_x = fx(20);
    for lives in (0..3u8).rev() {
        sim.place(Vec2::new(
            sim.state.run.cam_x - sim.rc.rules.blast_margin - fx(1),
            fx(5),
        ));
        sim.state.fighters[PLAYER].invuln = 0;
        sim.tick(Input::default());
        assert_eq!(sim.player().stocks, lives);
    }
    assert!(!sim.player().active);
    assert!(sim.state.run.over);
    assert_eq!(sim.state.winner, DRAW);
    // A finished run stands still.
    let before = sim.state.run;
    sim.idle(10);
    assert_eq!(sim.state.run.cam_x, before.cam_x);
}

// ---- Mobs ---------------------------------------------------------------------------------------------------------------------------

/// The player's attack launches a mob with the versus formulas: more knockback at a higher percent, hitlag on both, hitstun.
#[test]
fn a_mob_takes_knockback_that_grows_with_its_percent() {
    let launch = |percent: i32| {
        let mut sim = RunSim::with(street_and_gap(), 1);
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
    let mut sim = RunSim::with(street_and_gap(), 1);
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
    // The hit and the kill both count: 2 in the combo, so the walker's 5 coins at 1x.
    assert_eq!(sim.state.run.combo, 2);
    assert_eq!(sim.state.run.coins, u32::from(sim.rc.mobs[0].coins));
    // (Its hitlag first, then the burst.)
    sim.idle(u32::from(sim.rc.rules.burst_frames) + 40);
    assert!(!sim.state.run.mobs[n].active, "gone after its burst");
}

#[test]
fn a_mob_launched_off_screen_is_a_kill_only_if_the_player_hit_it() {
    let mut sim = RunSim::with(street_and_gap(), 1);
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
    let mut sim = RunSim::with(street_and_gap(), 1);
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
    let mut sim = RunSim::with(street_and_gap(), 1);
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
    let mut sim = RunSim::with(street_and_gap(), 1);
    sim.clear_mobs();
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

// ---- Coins and the combo --------------------------------------------------------------------------------------------------------

#[test]
fn the_combo_multiplies_the_coins() {
    let mut sim = RunSim::with(street_and_gap(), 1);
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
    // A kill with 9 in a row before it: the kill makes 10, so the walker's 5 coins at 3x.
    let a = sim.add_mob(0, Vec2::new(fx(20), sim.rc.rules.blast_top + fx(1)));
    sim.state.run.mobs[a].credit = 10;
    sim.state.run.combo = 9;
    sim.tick(Input::default());
    assert_eq!(sim.state.run.coins, 15);
    assert_eq!(sim.state.run.combo, 10);
}

#[test]
fn distance_counts_the_furthest_point_reached() {
    let mut sim = RunSim::with(street_and_gap(), 1);
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
    assert_eq!(sim.state.run.distance(), d.max(sim.state.run.distance()));
    assert!(sim.state.run.distance() >= d);
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

#[test]
fn same_seed_and_inputs_give_the_same_run() {
    let inputs = random_inputs(&mut Rng::new(5), 3000);
    let (a, sa) = random_run(9, &inputs);
    let (_, sb) = random_run(9, &inputs);
    assert_eq!(sa, sb);
    // Something happened: mobs came, the camera moved.
    assert!(a.run.cam_x > fx(20));
    let (_, sc) = random_run(10, &inputs);
    assert_ne!(sa.last(), sc.last(), "another seed is another run");
}

/// A saved state played on from (a rollback, a replay seek) gives the same run: the stage is rebuilt from the state.
#[test]
fn restoring_a_state_replays_the_same() {
    let inputs = random_inputs(&mut Rng::new(8), 1200);
    let mut sim = RunSim::new(4);
    let mut saved = None;
    for (n, i) in inputs.iter().enumerate() {
        if n == 500 {
            saved = Some(sim.state);
        }
        step_run(&mut sim.state, &mut sim.world, &sim.rc, i);
    }
    let straight = sim.state.checksum();
    // (A fresh working copy of the content: nothing carried over but the state.)
    let mut world = Content::placeholder();
    let mut state = saved.unwrap();
    for i in &inputs[500..] {
        step_run(&mut state, &mut world, &sim.rc, i);
    }
    assert_eq!(state.checksum(), straight);
}

/// Random play with a full pool of mobs: nothing panics, mobs stay sane, and the fighter never ends up inside a block.
#[test]
fn random_play_with_many_mobs_keeps_invariants() {
    for seed in 0..8u64 {
        let inputs = random_inputs(&mut Rng::new(seed), 1500);
        let mut sim = RunSim::new(seed);
        for (n, i) in inputs.iter().enumerate() {
            if n % 30 == 0 {
                // Keep the pool topped up around the player.
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
            for m in sim.state.run.mobs.iter().filter(|m| m.active) {
                assert!(usize::from(m.kind) < sim.rc.mobs.len());
                if let Ok(p) = usize::try_from(m.platform) {
                    assert!(sim.state.run.platforms[p].live, "a mob on a freed platform");
                }
            }
            let f = sim.player();
            if f.active {
                if let Ok(p) = usize::try_from(f.platform) {
                    assert!(
                        sim.state.run.platforms[p].live,
                        "seed {seed}: on a freed platform"
                    );
                }
                for b in sim.world.stage.platforms.iter().filter(|b| !b.pass_through) {
                    let inside = f.pos.x > b.left
                        && f.pos.x < b.right
                        && f.pos.y > b.bottom
                        && f.pos.y < b.y;
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
        ("start_x", |r| r.start_x += Fx::from_raw(1)),
        ("best_x", |r| r.best_x += Fx::from_raw(1)),
        ("next_x", |r| r.next_x += Fx::from_raw(1)),
        ("placed", |r| r.placed += 1),
        ("last_chunk", |r| r.last_chunk ^= 1),
        ("lap", |r| r.lap ^= 1 << 7),
        ("slots.live", |r| r.slots[RING - 1].live ^= true),
        ("slots.chunk", |r| r.slots[0].chunk ^= 1),
        ("slots.x", |r| r.slots[0].x += Fx::from_raw(1)),
        ("slots.end", |r| r.slots[0].end += Fx::from_raw(1)),
        ("slots.spawned", |r| r.slots[0].spawned ^= 1 << 7),
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
        ("ledges.live", |r| {
            r.ledges[run::POOL_LEDGES - 1].live ^= true
        }),
        ("ledges.x", |r| r.ledges[0].x += Fx::from_raw(1)),
        ("ledges.y", |r| r.ledges[0].y += Fx::from_raw(1)),
        ("ledges.side", |r| r.ledges[0].side = 1 - r.ledges[0].side),
        ("layout", |r| r.layout += 1),
        ("coins", |r| r.coins += 1),
        ("kills", |r| r.kills += 1),
        ("combo", |r| r.combo += 1),
        ("best_combo", |r| r.best_combo += 1),
        ("last_percent", |r| r.last_percent += Fx::from_raw(1)),
        ("last_stocks", |r| r.last_stocks += 1),
        ("crushes", |r| r.crushes += 1),
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
        ("mob.shot_mask", |r| r.mobs[0].shot_mask ^= 1 << 15),
        ("mob.struck", |r| r.mobs[0].struck ^= true),
        ("mob.credit", |r| r.mobs[0].credit += 1),
        ("mob.target", |r| r.mobs[0].target.x += Fx::from_raw(1)),
    ];
    for (name, edit) in edits {
        let mut s = base;
        edit(&mut s.run);
        assert_ne!(s.run, base.run, "{name}: the edit changed nothing");
        assert_ne!(s.checksum(), c, "{name} is not in the checksum");
    }
}

/// Versus is untouched: a match never has an active run, and a run's content is hashed with its own numbers.
#[test]
fn versus_has_no_run_and_run_content_hashes() {
    let content = Content::placeholder();
    let s = GameState::new(&content, 1, [0, 1, 0, 1]);
    assert!(!s.run.active);
    let rc = RunContent::placeholder();
    let mut other = rc.clone();
    other.rules.creep += Fx::from_raw(1);
    assert_ne!(rc.hash(), other.hash());
    let mut renamed = rc.clone();
    renamed.chunks[0].name = "elsewhere".to_string();
    assert_eq!(rc.hash(), renamed.hash(), "names are labels only");
    let mut kinds = rc.clone();
    kinds.mobs[0] = MobKind {
        coins: 6,
        ..kinds.mobs[0]
    };
    assert_ne!(rc.hash(), kinds.hash());
}
