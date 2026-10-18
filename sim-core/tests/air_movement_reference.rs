//! Air movement against the reference (sim 139, owner decision 99: "recoveries and air movement brought to the reference"). Every
//! body's air numbers are the reference character's parameter file (dragdown.wiki `SSBU/<Fighter>/Data`, 13.0.1, cross-checked with
//! ssbwiki.com's attribute tables), and the jumps they make are measured here against the published heights. Values are in the
//! reference's units (1 world unit = 8 of them). Sources and the per-fighter table: `docs/movesets/SHARED_MECHANICS.md` ("Air movement
//! against the reference").

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::JUMP;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// One body's published air numbers, in thousandths of the reference's units (friction in hundred-thousandths), heights in
/// hundredths, the jump's ground share and the midair jump's drift in percent.
struct Ref {
    air_speed: i64,
    accel_add: i64,
    accel_mul: i64,
    friction: i64,
    gravity: i64,
    fall: i64,
    fast_fall: i64,
    full_hop: i64,
    short_hop: i64,
    air_jump: i64,
    jump_x: i64,
    jump_x_mul: u8,
    jump_x_max: i64,
    air_jump_drift: u8,
}

#[rustfmt::skip]
const REFS: [Ref; 14] = [
    // 0 Longsword (the duelist)
    Ref { air_speed: 1071, accel_add: 10, accel_mul: 70, friction: 375, gravity: 75, fall: 1580, fast_fall: 2528, full_hop: 3366, short_hop: 1626, air_jump: 3366, jump_x: 950, jump_x_mul: 80, jump_x_max: 1200, air_jump_drift: 100 },
    // 1 Claws (the brawler)
    Ref { air_speed: 1281, accel_add: 10, accel_mul: 80, friction: 400, gravity: 130, fall: 1800, fast_fall: 2880, full_hop: 3202, short_hop: 1538, air_jump: 3071, jump_x: 1000, jump_x_mul: 80, jump_x_max: 1300, air_jump_drift: 90 },
    // 2 Needles (the rogue)
    Ref { air_speed: 1155, accel_add: 10, accel_mul: 80, friction: 1500, gravity: 150, fall: 1750, fast_fall: 2800, full_hop: 3900, short_hop: 1875, air_jump: 4000, jump_x: 760, jump_x_mul: 80, jump_x_max: 1400, air_jump_drift: 100 },
    // 3 Embers (the tumbler)
    Ref { air_speed: 1208, accel_add: 10, accel_mul: 70, friction: 1500, gravity: 87, fall: 1500, fast_fall: 2400, full_hop: 3633, short_hop: 1754, air_jump: 3633, jump_x: 950, jump_x_mul: 80, jump_x_max: 1300, air_jump_drift: 90 },
    // 4 Gauntlets (the sprinter)
    Ref { air_speed: 1218, accel_add: 10, accel_mul: 65, friction: 750, gravity: 120, fall: 1865, fast_fall: 2984, full_hop: 3731, short_hop: 1798, air_jump: 3731, jump_x: 900, jump_x_mul: 75, jump_x_max: 2100, air_jump_drift: 90 },
    // 5 Staff (the seer)
    Ref { air_speed: 1000, accel_add: 10, accel_mul: 105, friction: 1000, gravity: 120, fall: 1550, fast_fall: 2480, full_hop: 3590, short_hop: 1730, air_jump: 3590, jump_x: 720, jump_x_mul: 70, jump_x_max: 1100, air_jump_drift: 90 },
    // 6 Greatsword (the champion)
    Ref { air_speed: 1155, accel_add: 10, accel_mul: 60, friction: 900, gravity: 98, fall: 1680, fast_fall: 2688, full_hop: 3250, short_hop: 1700, air_jump: 3250, jump_x: 950, jump_x_mul: 90, jump_x_max: 1200, air_jump_drift: 100 },
    // 7 Bracers (the hauler)
    Ref { air_speed: 1208, accel_add: 10, accel_mul: 50, friction: 1500, gravity: 85, fall: 1630, fast_fall: 2608, full_hop: 3400, short_hop: 1730, air_jump: 3550, jump_x: 950, jump_x_mul: 80, jump_x_max: 1600, air_jump_drift: 100 },
    // 8 Clutch (the bounder)
    Ref { air_speed: 1344, accel_add: 30, accel_mul: 68, friction: 500, gravity: 80, fall: 1290, fast_fall: 2064, full_hop: 3609, short_hop: 1443, air_jump: 5156, jump_x: 910, jump_x_mul: 70, jump_x_max: 1500, air_jump_drift: 120 },
    // 9 Buckler (the wanderer)
    Ref { air_speed: 924, accel_add: 10, accel_mul: 40, friction: 380, gravity: 96, fall: 1600, fast_fall: 3040, full_hop: 2780, short_hop: 1338, air_jump: 2900, jump_x: 950, jump_x_mul: 80, jump_x_max: 1200, air_jump_drift: 100 },
    // 10 Pockets (the settler)
    Ref { air_speed: 987, accel_add: 10, accel_mul: 50, friction: 1000, gravity: 78, fall: 1320, fast_fall: 2112, full_hop: 3250, short_hop: 1569, air_jump: 3250, jump_x: 950, jump_x_mul: 80, jump_x_max: 1300, air_jump_drift: 90 },
    // 11 Popgun (the scamp)
    Ref { air_speed: 924, accel_add: 0, accel_mul: 50, friction: 800, gravity: 125, fall: 1750, fast_fall: 2800, full_hop: 4121, short_hop: 1990, air_jump: 4121, jump_x: 950, jump_x_mul: 80, jump_x_max: 1300, air_jump_drift: 90 },
    // 12 Dagger (the phantom)
    Ref { air_speed: 1100, accel_add: 10, accel_mul: 70, friction: 1000, gravity: 127, fall: 1630, fast_fall: 3097, full_hop: 3250, short_hop: 1420, air_jump: 3400, jump_x: 950, jump_x_mul: 75, jump_x_max: 1200, air_jump_drift: 90 },
    // 13 Nodachi (the exile)
    Ref { air_speed: 950, accel_add: 10, accel_mul: 70, friction: 800, gravity: 108, fall: 1840, fast_fall: 2944, full_hop: 3050, short_hop: 1000, air_jump: 3400, jump_x: 900, jump_x_mul: 90, jump_x_max: 1200, air_jump_drift: 100 },
];

/// `v` in the reference's units, scaled by `per_unit` and rounded.
fn units(v: Fx, per_unit: i64) -> i64 {
    let n = i64::from(v.raw()) * 8 * per_unit;
    (n + 32768).div_euclid(65536)
}

fn sim_for(c: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// The peak above the ground of a ground jump with jump held `hold` frames.
fn ground_jump(c: u8, hold: u32) -> Fx {
    let mut sim = sim_for(c);
    sim.stand(0, Fx::ZERO, 1);
    let mut peak = Fx::ZERO;
    for t in 1..=200u32 {
        sim.tick(inp(0, 0, if t <= hold { JUMP } else { 0 }));
        peak = peak.max(sim.f().pos.y);
        if t > 10 && sim.f().grounded() {
            break;
        }
    }
    peak
}

#[test]
fn every_body_has_its_reference_characters_air_numbers() {
    for (c, r) in REFS.iter().enumerate() {
        let p = sim_for(c as u8).content.fighters[c];
        let near = |name: &str, ours: i64, want: i64, tol: i64| {
            assert!(
                (ours - want).abs() <= tol,
                "class {c} {name}: ours {ours}, reference {want}"
            );
        };
        near("air speed", units(p.air_speed, 1000), r.air_speed, 1);
        near("air acceleration", units(p.air_accel, 1000), r.accel_add, 1);
        near(
            "stick acceleration",
            units(p.air_accel_stick, 1000),
            r.accel_mul,
            1,
        );
        // (Friction to the fixed-point step: 1 raw unit is 12 hundred-thousandths of the reference's units.)
        near(
            "air friction",
            units(p.air_friction, 100_000),
            r.friction,
            13,
        );
        near("gravity", units(p.gravity, 1000), r.gravity, 1);
        near("fall speed", units(p.max_fall_speed, 1000), r.fall, 1);
        near("fast fall", units(p.fast_fall_speed, 1000), r.fast_fall, 1);
        near("jump_speed_x", units(p.jump_speed_x, 1000), r.jump_x, 1);
        near(
            "jump_speed_x_max",
            units(p.jump_speed_x_max, 1000),
            r.jump_x_max,
            1,
        );
        assert_eq!(
            p.jump_speed_x_mul, r.jump_x_mul,
            "class {c} jump_speed_x_mul"
        );
        assert_eq!(
            p.air_jump_drift, r.air_jump_drift,
            "class {c} air_jump_drift"
        );
    }
}

#[test]
fn every_bodys_jumps_reach_the_reference_heights() {
    for (c, r) in REFS.iter().enumerate() {
        // Full hop and short hop from the ground, within 0.3 of the reference's units.
        let full = units(ground_jump(c as u8, 60), 100);
        let short = units(ground_jump(c as u8, 1), 100);
        assert!(
            (full - r.full_hop).abs() <= 30,
            "class {c} full hop {full} vs {}",
            r.full_hop
        );
        assert!(
            (short - r.short_hop).abs() <= 30,
            "class {c} short hop {short} vs {}",
            r.short_hop
        );
        // The midair jump from a standstill in the air.
        let mut sim = sim_for(c as u8);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        let y0 = sim.f().pos.y;
        sim.tick(inp(0, 0, JUMP));
        let mut peak = y0;
        for _ in 0..120 {
            sim.tick(inp(0, 0, 0));
            peak = peak.max(sim.f().pos.y);
        }
        let rise = units(peak - y0, 100);
        assert!(
            (rise - r.air_jump).abs() <= 40,
            "class {c} midair jump {rise} vs {}",
            r.air_jump
        );
    }
}

/// The midair jump's sideways speed is the stick times the air speed times the body's `jump_aerial_speed_x_mul`, whatever the speed
/// before it (it replaces the momentum: owner decision 59).
#[test]
fn a_midair_jump_sets_its_drift_by_the_bodys_multiplier() {
    for (c, r) in REFS.iter().enumerate() {
        let mut sim = sim_for(c as u8);
        let p = sim.content.fighters[c];
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), -p.air_speed, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        sim.tick(inp(127, 0, JUMP));
        let want = inp(127, 0, 0).stick_x_fx()
            * p.air_speed
            * Fx::from_ratio(i32::from(r.air_jump_drift), 100);
        // (The jump's own frame then drifts one frame toward the stick's full air speed.)
        let got = sim.f().vel.x;
        assert!(
            got >= want - p.air_friction && got <= want + p.air_accel + p.air_accel_stick,
            "class {c}: {got:?} vs {want:?}"
        );
    }
}

/// Class `c` on a wide floor under `share` (`Ruleset::jump_stick_share`): moves with the stick at `move_x` for `frames` frames
/// (none: from standing), then jumps with the stick at `jump_x`. Returns the ground speed before the jump and the speed on leaving.
fn ground_jump_x(c: u8, share: u8, move_x: i8, frames: usize, jump_x: i8) -> (Fx, Fx) {
    let mut sim = sim_for(c);
    sim.content.rules.jump_stick_share = share;
    sim.content.stage.platforms[0].left = Fx::from_int(-40);
    sim.content.stage.platforms[0].right = Fx::from_int(40);
    sim.stand(1, Fx::from_int(39), -1);
    sim.stand(0, Fx::from_int(-10), 1);
    sim.ticks(frames, inp(move_x, 0, 0));
    let before = sim.f().vel.x;
    sim.tick(inp(jump_x, 0, JUMP));
    let mut n = 0;
    while sim.f().grounded() && n < 10 {
        sim.tick(inp(jump_x, 0, JUMP));
        n += 1;
    }
    assert_eq!(sim.f().state, S::Airborne, "class {c}");
    (before, sim.f().vel.x)
}

/// A ground jump leaves at the body's own rule (owner decision 100, the standard `jump_stick_share` 0): from standing with the stick
/// held, at once at the stick's share of `jump_speed_x`; out of a run, `jump_speed_x_mul` of the run held within `jump_speed_x_max`,
/// whatever the stick (held forward or back).
#[test]
fn a_ground_jump_leaves_at_the_bodys_own_horizontal_rule() {
    for c in 0..14u8 {
        let p = sim_for(c).content.fighters[usize::from(c)];
        let slack = p.air_accel + p.air_accel_stick + p.air_friction;
        let stick = inp(127, 0, 0).stick_x_fx() * p.jump_speed_x;
        // (The takeoff frame then drifts one frame toward the stick's air-speed target.)
        let (_, vx) = ground_jump_x(c, 0, 0, 0, 127);
        assert!(
            vx >= stick.min(p.jump_speed_x_max) && vx <= stick + p.air_accel + p.air_accel_stick,
            "class {c} standing jump: {vx:?} vs {stick:?}"
        );
        for x in [127i8, 0, -127] {
            let (run, vx) = ground_jump_x(c, 0, 127, 30, x);
            let kept =
                (run * Fx::from_ratio(i32::from(p.jump_speed_x_mul), 100)).min(p.jump_speed_x_max);
            assert!(
                (vx - kept).abs() <= slack,
                "class {c} run jump (stick {x}): {vx:?}, run {run:?}, kept {kept:?}"
            );
        }
        // A slow walk into a jump with the stick pushed full: the walk's share, not the stick's (faster) one.
        let (walk, vx) = ground_jump_x(c, 0, 30, 30, 127);
        let kept = walk * Fx::from_ratio(i32::from(p.jump_speed_x_mul), 100);
        assert!(walk > Fx::ZERO, "class {c} walks");
        assert!(
            (vx - kept).abs() <= slack,
            "class {c} walk jump: {vx:?}, walk {walk:?}"
        );
    }
}

/// With `jump_stick_share` 1 (decision 99's reading, kept for playtests) a jump takes the faster of the run's share and the stick's
/// the same way; a stick held back keeps the run's share (a reverse aerial rush).
#[test]
fn with_the_stick_share_on_a_jump_takes_the_faster_of_the_run_and_the_stick() {
    for c in 0..14u8 {
        let p = sim_for(c).content.fighters[usize::from(c)];
        let slack = p.air_accel + p.air_accel_stick + p.air_friction;
        let stick = (inp(127, 0, 0).stick_x_fx() * p.jump_speed_x).min(p.jump_speed_x_max);
        for back in [false, true] {
            let (run, vx) = ground_jump_x(c, 1, 127, 30, if back { -127 } else { 127 });
            let kept =
                (run * Fx::from_ratio(i32::from(p.jump_speed_x_mul), 100)).min(p.jump_speed_x_max);
            let want = kept.max(if back { Fx::ZERO } else { stick });
            assert!(
                (vx - want).abs() <= slack,
                "class {c} run jump (back {back}): {vx:?}, run {run:?}"
            );
        }
        // The slow walk with the stick full: the stick's share, where it is the faster.
        let (walk, vx) = ground_jump_x(c, 1, 30, 30, 127);
        let kept = walk * Fx::from_ratio(i32::from(p.jump_speed_x_mul), 100);
        let want = kept.max(stick);
        assert!(
            (vx - want).abs() <= slack,
            "class {c} walk jump: {vx:?}, walk {walk:?}"
        );
    }
}
