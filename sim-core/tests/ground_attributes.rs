//! Each fighter's ground movement is its reference character's (sim v54), from the reference game's parameter file (via
//! dragdown.wiki) and ssbwiki.com's dash table: walk, initial dash and run speeds, traction, walk and run acceleration (a base and a
//! share by the stick), the initial dash's length and the heavy landing. Shared numbers are the reference's universal ones.

mod common;

use common::{inp, Sim};
use sim_core::content::FighterParams;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// Reference units (thousandths) on the ground, as the content builders write them.
fn g(thousandths: i32) -> Fx {
    FighterParams::gu(thousandths)
}

fn fine(hundred_thousandths: i32) -> Fx {
    FighterParams::gu_fine(hundred_thousandths)
}

#[test]
fn every_fighter_has_its_reference_characters_ground_numbers() {
    // (walk, initial dash, run, traction, walk accel base + stick, run accel base + stick, dash frames, heavy landing lag)
    let expected = [
        (
            "duelist",
            FighterParams::duelist(),
            [g(1575), g(2255), g(1964), g(114)],
            [Fx::ZERO, g(189), g(44), fine(9020)],
            (10, 3),
        ),
        (
            "brawler",
            FighterParams::brawler(),
            [g(1208), g(2090), g(1540), g(110)],
            [g(105), g(105), g(44), fine(10978)],
            (10, 3),
        ),
        (
            "rogue",
            FighterParams::rogue(),
            [g(1470), g(2178), g(2420), g(106)],
            [g(105), g(210), g(44), fine(12100)],
            (7, 6),
        ),
        (
            "tumbler",
            FighterParams::tumbler(),
            [g(1155), g(1936), g(1760), g(102)],
            [g(105), g(105), g(44), fine(10978)],
            (10, 3),
        ),
    ];
    for (name, p, speeds, accels, (dash, heavy)) in expected {
        assert_eq!(
            [
                p.walk_speed,
                p.dash_speed,
                p.run_speed,
                p.dash_attack_friction
            ],
            speeds,
            "{name}: walk, initial dash, run, traction"
        );
        // (Traction is doubled above walking speed, as `ground_friction`.)
        assert!(
            (p.ground_friction - p.dash_attack_friction.mul_int(2)).abs() <= Fx::from_raw(1),
            "{name}"
        );
        assert_eq!(
            [
                p.walk_accel,
                p.walk_accel_stick,
                p.run_accel,
                p.run_accel_stick
            ],
            accels,
            "{name}: walk and run acceleration"
        );
        assert_eq!(
            (p.dash_frames, p.heavy_landing_lag),
            (dash, heavy),
            "{name}"
        );
        // Universal in the reference: a 3-frame jump squat and a light landing of 3 (2 of lag).
        assert_eq!((p.jump_squat_frames, p.landing_lag), (3, 2), "{name}");
    }
}

#[test]
fn no_two_fighters_move_the_same_on_the_ground() {
    let all = [
        FighterParams::duelist(),
        FighterParams::brawler(),
        FighterParams::rogue(),
        FighterParams::tumbler(),
    ];
    let ground = |p: &FighterParams| {
        (
            p.walk_speed,
            p.dash_speed,
            p.run_speed,
            p.ground_friction,
            p.walk_accel_stick,
            p.run_accel_stick,
        )
    };
    for i in 0..all.len() {
        for j in i + 1..all.len() {
            let (a, b) = (ground(&all[i]), ground(&all[j]));
            // (Reference characters can share an acceleration, as the tumbler and the brawler do, but never their speeds and
            // traction.)
            assert!(
                a.0 != b.0 && a.1 != b.1 && a.2 != b.2 && a.3 != b.3,
                "fighters {i} and {j} share a ground speed or traction: {a:?} {b:?}"
            );
            assert_ne!(a, b, "fighters {i} and {j} move the same on the ground");
        }
    }
}

/// Frames from a dash flick until the fighter is running (stick held).
fn frames_to_run(char_id: u8) -> u32 {
    let mut sim = Sim::with_chars([char_id, 0, 0, 0]);
    sim.stand(1, Fx::from_int(14), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-9), 1);
    sim.tick(inp(0, 0, 0));
    let mut n = 0;
    while sim.f().state != S::Run && n < 40 {
        sim.tick(inp(127, 0, 0));
        n += 1;
    }
    n
}

#[test]
fn the_rogues_initial_dash_is_shorter() {
    let (duelist, brawler, rogue) = (frames_to_run(0), frames_to_run(1), frames_to_run(2));
    assert_eq!(duelist, brawler);
    assert_eq!(duelist - rogue, 3, "10 frames against 7");
}

#[test]
fn a_walk_speeds_up_faster_with_the_stick_pushed_further() {
    // The duelist walks with no base acceleration, all of it by the stick: half a push accelerates half as fast.
    let speed_after = |x: i8| {
        let mut sim = Sim::with_chars([0, 0, 0, 0]);
        sim.stand(1, Fx::from_int(14), -1);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.stand(0, Fx::from_int(-9), 1);
        sim.tick(inp(0, 0, 0));
        // (Rolled up slowly, so it walks rather than dashes.)
        sim.tick(inp(x / 2, 0, 0));
        sim.tick(inp(x, 0, 0));
        assert_eq!(sim.f().state, S::Walk);
        sim.f().vel.x
    };
    let (half, full) = (speed_after(38), speed_after(76));
    assert!(full > half, "{half:?} {full:?}");
}
