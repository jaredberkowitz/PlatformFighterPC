//! Sim 151 (`docs/MOVEMENT_1TO1.md`): the jump squat brakes the speed carried into it at traction on each of its frames, the frame the
//! jump is pressed included (`Ruleset::jump_squat_brake`, the reference's: its squat is a ground motion that moves nothing of its
//! own); and a grounded hit's slide can brake at traction alone (`Ruleset::ground_hit_slide`, the reference's, off for now).

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP};
use sim_core::state::FighterState as S;
use sim_core::{Fx, GameState};

const CLAWS: u8 = 1;

/// The Claws alone on a long floor, running right at its full run speed.
fn running(squat_brake: u8) -> Sim {
    let mut sim = Sim::with_chars([CLAWS, 0, 0, 0]);
    sim.content.rules.jump_squat_brake = squat_brake;
    let st = &mut sim.content.stage;
    st.platforms[0].left = Fx::from_int(-300);
    st.platforms[0].right = Fx::from_int(300);
    st.platforms.truncate(1);
    st.ledges.clear();
    st.blast_left = Fx::from_int(-400);
    st.blast_right = Fx::from_int(400);
    sim.state = GameState::new_with_active(&sim.content, 1, [CLAWS, 0, 0, 0], 0b0001);
    sim.stand(0, Fx::from_int(-100), 1);
    sim.tick(inp(0, 0, 0));
    sim.ticks(40, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    sim
}

#[test]
fn the_jump_squat_brakes_the_run_at_traction_on_each_of_its_frames() {
    let mut sim = running(1);
    let p = sim.content.fighters[usize::from(CLAWS)];
    let traction = p.ground_friction * Fx::HALF;
    let run = sim.f().vel.x;
    assert_eq!(run, p.run_speed);
    // The press frame is the squat's first: it brakes, and moves at the braked speed.
    let x0 = sim.f().pos.x;
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
    assert_eq!(sim.f().vel.x, run - traction);
    assert_eq!(sim.f().pos.x - x0, run - traction);
    // Then each of its other frames.
    for k in 2..=i32::from(p.jump_squat_frames) {
        sim.tick(inp(0, 0, JUMP));
        assert_eq!(sim.f().state, S::JumpSquat, "squat frame {k}");
        assert_eq!(sim.f().vel.x, run - traction.mul_int(k), "squat frame {k}");
    }
    // The takeoff keeps the body's share of what is left (the Claws' run is under its jump cap, so the braking shows), less the
    // takeoff frame's own air friction (the stick is let go).
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::Airborne);
    let left = run - traction.mul_int(i32::from(p.jump_squat_frames));
    let kept = left * Fx::from_ratio(i32::from(p.jump_speed_x_mul), 100);
    assert!(kept < p.jump_speed_x_max);
    assert_eq!(sim.f().vel.x, kept - p.air_friction);
}

#[test]
fn without_the_squat_brake_the_run_is_kept_whole_and_the_press_frame_stands() {
    let mut sim = running(0);
    let p = sim.content.fighters[usize::from(CLAWS)];
    let run = sim.f().vel.x;
    let x0 = sim.f().pos.x;
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().pos.x, x0, "the press frame stands still");
    for _ in 0..p.jump_squat_frames {
        sim.tick(inp(0, 0, JUMP));
    }
    assert_eq!(sim.f().state, S::Airborne);
    let kept = (run * Fx::from_ratio(i32::from(p.jump_speed_x_mul), 100)).min(p.jump_speed_x_max);
    assert_eq!(sim.f().vel.x, kept - p.air_friction);
}

/// The duelist jabs the other standing just ahead; returns how fast the hit fighter's slide slows on each grounded frame after its
/// hitlag, under `ground_hit_slide`.
fn hit_slide_steps(rule: u8) -> Vec<Fx> {
    let mut sim = Sim::duel(Fx::from_ratio(11, 10));
    sim.content.rules.ground_hit_slide = rule;
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.tick(inp(0, 0, ATTACK));
    let mut steps = Vec::new();
    let mut last: Option<Fx> = None;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        let v = sim.fighter(1);
        if v.state == S::Hitstun && v.hitlag == 0 && v.grounded() && v.kb_vel.x != Fx::ZERO {
            if let Some(l) = last {
                steps.push(l.abs() - v.kb_vel.x.abs());
            }
            last = Some(v.kb_vel.x);
        }
    }
    steps
}

#[test]
fn a_grounded_hit_slides_at_traction_alone_with_the_references_rule() {
    // (The one hit is player 1, the brawler body: its traction.)
    let p = Sim::duel(Fx::ONE).content.fighters[1];
    let traction = p.ground_friction * Fx::HALF;
    // Off (the standard for now): the launch decay and the doubled friction, so each grounded frame takes off more than traction.
    let off = hit_slide_steps(0);
    assert!(!off.is_empty(), "the hit slid");
    assert!(off.iter().all(|s| *s > traction), "{off:?}");
    // On: traction alone (the last step may be cut short by the stop).
    let on = hit_slide_steps(1);
    assert!(
        on.len() > off.len(),
        "it slides longer: {on:?} against {off:?}"
    );
    assert!(
        on.iter().all(|s| *s == traction)
            || (on[..on.len() - 1].iter().all(|s| *s == traction) && on[on.len() - 1] < traction),
        "{on:?}"
    );
    assert_eq!(Sim::new().content.rules.ground_hit_slide, 0);
}
