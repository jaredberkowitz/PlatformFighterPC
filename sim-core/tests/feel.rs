//! Game feel: walking before running, committed runs, the shape of a full hop, graded air drift,
//! and the strong-attack button.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, STRONG};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// A stick that rises gradually, the way a keyboard direction ramps in `input_reader.gd`.
fn ramp(sim: &mut Sim, start: i8, step: i8, frames: usize) {
    let mut x = i32::from(start);
    for _ in 0..frames {
        sim.tick(inp(x.min(127) as i8, 0, 0));
        x += i32::from(step);
        assert!(
            !matches!(sim.f().state, S::Dash | S::Run),
            "a gradual press must walk, not dash (stick {x})"
        );
    }
}

// ---- Walking -------------------------------------------------------------------------------------

#[test]
fn a_gradual_press_walks_even_when_it_ends_at_full_tilt() {
    let mut sim = Sim::new();
    ramp(&mut sim, 44, 8, 12);
    sim.ticks(40, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Walk);
    let walk = sim.content.fighters[0].walk_speed;
    assert!(
        (sim.f().vel.x - walk).abs() < fx(1, 100),
        "walks at walk speed"
    );
}

#[test]
fn a_fast_press_from_neutral_dashes() {
    let mut sim = Sim::new();
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
}

#[test]
fn walking_starts_slowly_and_speeds_up_with_the_press() {
    let mut sim = Sim::new();
    sim.tick(inp(44, 0, 0));
    let first = sim.f().vel.x;
    assert!(first > Fx::ZERO);
    assert!(first < sim.content.fighters[0].walk_speed * fx(1, 2));
}

// ---- A run is committed --------------------------------------------------------------------------

#[test]
fn reversing_out_of_a_run_skids_and_turns_instead_of_dash_dancing() {
    let mut sim = Sim::new();
    sim.ticks(40, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    sim.tick(inp(-127, 0, 0));
    assert_eq!(sim.f().state, S::Turn, "a run turns by skidding");
    assert_eq!(sim.f().facing, -1);
    assert!(
        sim.f().vel.x > Fx::ZERO,
        "the run's speed is still carrying the fighter forward"
    );
    // It comes to a stop before going the other way.
    let mut crossed = false;
    for _ in 0..30 {
        sim.tick(inp(-127, 0, 0));
        if sim.f().state == S::Dash {
            break;
        }
        crossed |= sim.f().vel.x < Fx::ZERO && sim.f().state == S::Turn;
    }
    assert!(!crossed, "no instant reversal while skidding");
}

#[test]
fn a_dash_dance_still_works_inside_the_initial_dash() {
    let mut sim = Sim::new();
    sim.ticks(3, inp(127, 0, 0));
    sim.tick(inp(-127, 0, 0));
    assert_eq!((sim.f().state, sim.f().facing), (S::Dash, -1));
    assert!(sim.f().vel.x < Fx::ZERO);
}

// ---- Full hop shape ------------------------------------------------------------------------------

/// Heights above the floor on each frame of a full hop, starting from the first frame of jump squat.
fn full_hop_heights(chars: [u8; 4], frames: usize) -> Vec<Fx> {
    let mut sim = Sim::with_chars(chars);
    (0..frames)
        .map(|_| {
            sim.tick(inp(0, 0, JUMP));
            sim.f().pos.y
        })
        .collect()
}

#[test]
fn a_full_hop_opens_fast_and_covers_most_of_its_rise_early() {
    for chars in [[0, 1, 0, 1], [1, 0, 1, 0]] {
        let heights = full_hop_heights(chars, 120);
        let apex = heights.iter().copied().fold(Fx::ZERO, Fx::max);
        let apex_frame = heights.iter().position(|h| *h == apex).unwrap();
        let p = Sim::with_chars(chars).content.fighters[usize::from(chars[0])];
        let squat = usize::from(p.jump_squat_frames);
        let burst = usize::from(p.hop_burst_frames);
        // When the opening ends the fighter has risen about 55% of the hop.
        // Takeoff happens on tick `squat` (index), and the opening moves on the next `burst` ticks.
        let at_burst = heights[squat + burst];
        assert!(
            at_burst > apex * fx(50, 100) && at_burst < apex * fx(60, 100),
            "after the opening: {at_burst:?} of {apex:?}"
        );
        // Starting from the same height, a plain arc would take far longer to get there.
        let g = p.gravity;
        let plain_arc_frames = (apex * Fx::from_int(2) / g).sqrt();
        let apex_after_takeoff = apex_frame - squat;
        assert!(
            Fx::from_int(apex_after_takeoff as i32) < plain_arc_frames * fx(90, 100),
            "apex after {apex_after_takeoff} frames, a plain arc would take {plain_arc_frames:?}"
        );
    }
}

#[test]
fn a_short_hop_has_no_fast_opening() {
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP));
    let mut prev = Fx::ZERO;
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        let rise = sim.f().pos.y - prev;
        prev = sim.f().pos.y;
        assert!(rise < sim.content.fighters[0].hop_burst_velocity);
    }
}

#[test]
fn an_air_jump_during_the_opening_ends_it() {
    let mut sim = Sim::new();
    sim.ticks(5, inp(0, 0, JUMP)); // squat, then the opening begins
    assert!(sim.f().hop_boost > 0);
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().hop_boost, 0);
    assert_eq!(sim.f().air_jumps_left, 0);
}

// ---- Air control ---------------------------------------------------------------------------------

#[test]
fn a_light_press_in_the_air_drifts_less_than_a_full_one() {
    let drift = |stick: i8| {
        let mut sim = Sim::new();
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.ticks(6, inp(stick, 0, 0));
        sim.f().vel.x
    };
    let (light, full) = (drift(44), drift(127));
    assert!(light > Fx::ZERO);
    assert!(
        light * Fx::from_int(2) < full,
        "light {light:?} vs full {full:?}"
    );
}

#[test]
fn a_short_tap_in_the_air_leaves_a_small_nudge_not_a_full_drift() {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.ticks(4, inp(60, 0, 0));
    sim.ticks(40, inp(0, 0, 0));
    let air_speed = sim.content.fighters[0].air_speed;
    let v = sim.f().vel.x;
    assert!(v > Fx::ZERO && v < air_speed * fx(1, 2), "nudge of {v:?}");
}

// ---- Strong attack button --------------------------------------------------------------------------

fn first_ground_move(x: i8, y: i8, buttons: u16, prior: i8) -> u8 {
    let mut sim = Sim::new();
    // The stick has been resting in place (a tilt) before the attack is pressed.
    sim.ticks(10, inp(prior, y, 0));
    sim.tick(inp(x, y, buttons));
    assert_eq!(sim.f().state, S::Attack);
    sim.f().move_id
}

#[test]
fn the_strong_button_makes_smash_attacks_from_any_held_direction() {
    let smash = |x: i8, y: i8| first_ground_move(x, y, ATTACK | STRONG, x);
    assert_eq!(smash(60, 0), MoveId::FSmash as u8);
    assert_eq!(smash(0, 100), MoveId::USmash as u8);
    assert_eq!(smash(0, -100), MoveId::DSmash as u8);
    // Without a direction there is nothing to smash toward.
    assert_eq!(smash(0, 0), MoveId::Jab as u8);
}

#[test]
fn the_plain_attack_button_with_a_held_direction_is_a_tilt() {
    let tilt = |x: i8, y: i8| first_ground_move(x, y, ATTACK, x);
    assert_eq!(tilt(60, 0), MoveId::FTilt as u8);
    assert_eq!(tilt(0, 100), MoveId::UTilt as u8);
    assert_eq!(tilt(0, -100), MoveId::DTilt as u8);
}

#[test]
fn the_strong_button_does_nothing_special_in_the_air() {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(60, 0, ATTACK | STRONG));
    assert_eq!(sim.f().move_id, MoveId::FAir as u8);
}
