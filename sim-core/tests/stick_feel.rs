//! The movement stick (sim v53): a dash takes a fuller flick than before, so a quick push to a brisk walk stays a walk; and while
//! running, the stick's top and bottom halves on either side still count as that side, so rolling it along the bottom (or the top)
//! from one side to the other turns the run, and a jump in that turn is a reverse aerial rush, as in the reference. And a fighter
//! standing or walking can be hit on the guard it holds out in front, where its fists are drawn.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::{hurtboxes, params_of, weapon_of};
use sim_core::input::buttons::JUMP;
use sim_core::state::FighterState as S;
use sim_core::Fx;

fn alone() -> Sim {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.stand(0, Fx::from_int(-9), 1);
    sim.stand(1, Fx::from_int(14), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn run(sim: &mut Sim) {
    sim.tick(inp(0, 0, 0));
    for _ in 0..40 {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(sim.f().state, S::Run);
}

/// The stick on its rim, rolled from the right side to the left one along the bottom (`dir` -1) or the top (1), 30 degrees a
/// frame, then held left.
fn roll(dir: i8) -> [(i8, i8); 7] {
    let d = |y: i8| y * dir;
    [
        (110, d(64)),
        (64, d(110)),
        (0, d(127)),
        (-64, d(110)),
        (-110, d(64)),
        (-127, 0),
        (-127, 0),
    ]
}

#[test]
fn a_quick_push_to_a_brisk_walk_walks() {
    // From neutral to about 0.68 in one frame: before, a dash; now a walk.
    let mut sim = alone();
    sim.tick(inp(0, 0, 0));
    for _ in 0..10 {
        sim.tick(inp(86, 0, 0));
        assert_ne!(sim.f().state, S::Dash);
    }
    assert_eq!(sim.f().state, S::Walk);
    // A full flick still dashes, as does one to about 0.8.
    for x in [127, 102] {
        let mut sim = alone();
        sim.tick(inp(0, 0, 0));
        sim.tick(inp(x, 0, 0));
        assert_eq!(sim.f().state, S::Dash, "flick to {x}");
    }
}

#[test]
fn down_and_forward_keeps_a_run_going() {
    let mut sim = alone();
    run(&mut sim);
    for _ in 0..10 {
        sim.tick(inp(100, -78, 0));
        assert_eq!(sim.f().state, S::Run);
    }
    assert!(sim.f().vel.x > Fx::ZERO);
}

#[test]
fn a_stick_rolled_along_the_bottom_or_top_turns_a_run_like_a_flick() {
    for dir in [-1, 1] {
        let mut sim = alone();
        run(&mut sim);
        let mut states = Vec::new();
        for (x, y) in roll(dir) {
            sim.tick(inp(x, y, 0));
            states.push(sim.f().state);
        }
        assert!(
            !states.contains(&S::Crouch) && !states.contains(&S::Idle),
            "dir {dir}: the run never stops or crouches: {states:?}"
        );
        assert!(states.contains(&S::Turn), "dir {dir}: {states:?}");
        assert_eq!(sim.f().facing, -1, "dir {dir}");
    }
}

#[test]
fn a_jump_out_of_a_rolled_turnaround_is_a_reverse_aerial_rush() {
    for dir in [-1, 1] {
        let mut sim = alone();
        run(&mut sim);
        let path = roll(dir);
        // Jump as the stick reaches the far side's lower (or upper) half.
        for (k, &(x, y)) in path[..5].iter().enumerate() {
            sim.tick(inp(x, y, if k == 4 { JUMP } else { 0 }));
        }
        let mut n = 0;
        while sim.f().grounded() && n < 10 {
            sim.tick(inp(0, 0, 0));
            n += 1;
        }
        assert!(!sim.f().grounded(), "dir {dir}");
        assert_eq!(sim.f().facing, -1, "dir {dir}: facing back");
        assert!(
            // (At least the duelist's jump cap, `jump_speed_x_max` 1.2 of the reference's units: sim 139.)
            sim.f().vel.x >= fx(15, 100),
            "dir {dir}: still carried forward: {:?}",
            sim.f().vel.x
        );
    }
}

#[test]
fn straight_down_out_of_a_run_still_crouches_after_a_short_skid() {
    let mut sim = alone();
    run(&mut sim);
    let mut states = Vec::new();
    for _ in 0..6 {
        sim.tick(inp(0, -127, 0));
        states.push(sim.f().state);
    }
    assert_eq!(sim.f().state, S::Crouch, "{states:?}");
    assert!(
        states[..4].contains(&S::Crouch),
        "within four frames: {states:?}"
    );
}

#[test]
fn letting_go_of_a_run_still_stops_at_once() {
    let mut sim = alone();
    run(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

/// How far the fighter's hurtboxes reach ahead of it and behind it (both positive).
fn reach(sim: &Sim, i: usize) -> (Fx, Fx) {
    let f = sim.fighter(i);
    let p = params_of(&sim.content, f);
    let hb = hurtboxes(f, p, weapon_of(&sim.content, p));
    let ahead = hb
        .iter()
        .map(|(c, r)| (c.x - f.pos.x).mul_int(i32::from(f.facing)) + *r)
        .max()
        .unwrap();
    let behind = hb
        .iter()
        .map(|(c, r)| *r - (c.x - f.pos.x).mul_int(i32::from(f.facing)))
        .max()
        .unwrap();
    (ahead, behind)
}

#[test]
fn the_guard_held_out_in_front_can_be_hit() {
    for c in 0..3u8 {
        for facing in [1, -1] {
            let mut sim = Sim::with_chars([c, 0, 0, 0]);
            sim.stand(0, Fx::ZERO, facing);
            let (ahead, behind) = reach(&sim, 0);
            // The fists, about a unit ahead (drawn at 1.0 to 1.1); the back, the body circles' 0.68.
            assert!(
                ahead > fx(100, 100) && ahead < fx(110, 100),
                "char {c}: {ahead:?}"
            );
            assert!(behind < fx(70, 100), "char {c}: {behind:?}");
            // Walking too; not in a run (the arms pump) or the air.
            sim.state.fighters[0].state = S::Walk;
            assert_eq!(reach(&sim, 0).0, ahead);
            sim.state.fighters[0].state = S::Run;
            assert!(reach(&sim, 0).0 < fx(70, 100));
        }
    }
}
