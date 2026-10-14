//! Game feel: walking before running, committed runs, the shape of a full hop, graded air drift,
//! and the strong-attack button.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, STRONG};
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
    // The turnaround stands for a moment (`dash_turn_delay`), then the dash goes the new way.
    sim.ticks(3, inp(-127, 0, 0));
    assert!(sim.f().vel.x < Fx::ZERO);
}

#[test]
fn a_reversed_dash_stands_for_the_turnaround_before_it_moves() {
    let mut sim = Sim::new();
    let delay = usize::from(sim.content.fighters[0].dash_turn_delay);
    assert!(delay >= 1);
    sim.ticks(4, inp(127, 0, 0));
    assert!(sim.f().vel.x > Fx::ZERO);
    sim.tick(inp(-127, 0, 0));
    assert_eq!(sim.f().facing, -1, "the fighter turns at once");
    assert_eq!(sim.f().vel.x, Fx::ZERO, "but does not move yet");
    sim.ticks(delay - 1, inp(-127, 0, 0));
    assert_eq!(sim.f().vel.x, Fx::ZERO);
    sim.tick(inp(-127, 0, 0));
    assert!(
        sim.f().vel.x < Fx::ZERO,
        "the first step after {delay} frames"
    );
}

// ---- Dash turnaround and foxtrot (sim 103: the reference's dash turnaround table and interrupt frame) ------------------------
// A flick back on the first 5 frames of an initial dash turns at once; later in the initial dash it is held and the dash turns as
// it ends; once it is a run, a flick back skids. A dash that ends with the stick let go stays in the dash until frame 15
// (`Ruleset::dash_interrupt_frame`), when the next dash (a foxtrot, or a late dash back) starts.

/// The frame of an initial dash (counted from the dash's first frame) on which a flick back made on `flick_on` turns the fighter, and
/// the state it is in then.
fn turn_after_flick_on(flick_on: usize) -> (usize, S) {
    turn_after_flick_with(flick_on, None)
}

/// [`turn_after_flick_on`] with every body's `dash_reverse_frames` set to `reverse` (their own value if `None`).
fn turn_after_flick_with(flick_on: usize, reverse: Option<u8>) -> (usize, S) {
    let mut sim = Sim::new();
    if let Some(r) = reverse {
        for p in sim.content.fighters.iter_mut() {
            p.dash_reverse_frames = r;
        }
    }
    sim.tick(inp(0, 0, 0));
    sim.ticks(flick_on - 1, inp(127, 0, 0));
    for frame in flick_on..flick_on + 30 {
        sim.tick(inp(-127, 0, 0));
        if sim.f().facing == -1 {
            return (frame, sim.f().state);
        }
    }
    panic!("never turned");
}

#[test]
fn a_flick_back_anywhere_in_the_initial_dash_turns_at_once() {
    // Owner decision 2026-10-14 (sim 106): the reference's held turn (below) felt sluggish.
    let p = Sim::new().content.fighters[0];
    assert_eq!(p.dash_reverse_frames, 255);
    for frame in 2..=usize::from(p.dash_frames) {
        assert_eq!(
            turn_after_flick_on(frame),
            (frame, S::Dash),
            "flick on frame {frame}"
        );
    }
}

#[test]
fn the_reference_held_turn_still_works_with_dash_reverse_frames_5() {
    // The reference's turnaround (sim 103), kept as an option: frames 1-5 turn at once, later flicks are held to the dash's end.
    let dash = usize::from(Sim::new().content.fighters[0].dash_frames);
    for frame in 2..=5 {
        assert_eq!(
            turn_after_flick_with(frame, Some(5)),
            (frame, S::Dash),
            "flick on frame {frame}"
        );
    }
    for frame in 6..=dash {
        assert_eq!(
            turn_after_flick_with(frame, Some(5)),
            (dash + 1, S::Dash),
            "flick on frame {frame}: a dash back once the dash ends"
        );
    }
}

#[test]
fn once_the_dash_is_a_run_a_flick_back_skids() {
    let dash = usize::from(Sim::new().content.fighters[0].dash_frames);
    for frame in [dash + 2, dash + 4, 20, 30] {
        assert_eq!(
            turn_after_flick_on(frame),
            (frame, S::Turn),
            "flick on frame {frame}"
        );
    }
}

#[test]
fn a_dash_back_lasts_its_turnaround_longer_than_a_dash() {
    // The reference's pivot dash is two frames longer than its dash: the turnaround's standing frames do not use up the dash.
    let p = Sim::new().content.fighters[0];
    let mut sim = Sim::new();
    sim.ticks(3, inp(127, 0, 0));
    let mut frames = 0;
    sim.tick(inp(-127, 0, 0));
    while sim.f().state == S::Dash {
        frames += 1;
        sim.tick(inp(-127, 0, 0));
    }
    assert_eq!(sim.f().state, S::Run);
    assert_eq!(
        frames,
        usize::from(p.dash_frames) + usize::from(p.dash_turn_delay)
    );
}

/// Taps a dash on tick 2 (its frame 1), lets go, and from tick `flick_tick` holds the stick at `stick_x`: the tick on which a new dash
/// starts, if one does within 25 ticks.
fn next_dash_tick(flick_tick: usize, stick_x: i8) -> Option<usize> {
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    for tick in 3..flick_tick {
        sim.tick(inp(0, 0, 0));
        assert!(
            sim.f().state != S::Dash || sim.f().state_frame == u16::try_from(tick - 2).unwrap()
        );
    }
    for tick in flick_tick..flick_tick + 25 {
        sim.tick(inp(stick_x, 0, 0));
        if sim.f().state == S::Dash && sim.f().dash_age == 0 {
            return Some(tick);
        }
    }
    None
}

#[test]
fn a_foxtrot_waits_for_the_dash_interrupt_frame() {
    let rules = Sim::new().content.rules;
    assert_eq!(rules.dash_interrupt_frame, 15);
    // The first dash's frame 15 is tick 16: a flick made after the dash ended (frame 11) but before that is held until then, either
    // way; one made on or after it dashes at once.
    for flick_tick in [13, 14, 15, 16] {
        assert_eq!(
            next_dash_tick(flick_tick, 127),
            Some(16),
            "foxtrot flicked on tick {flick_tick}"
        );
        assert_eq!(
            next_dash_tick(flick_tick, -127),
            Some(16),
            "dash back flicked on tick {flick_tick}"
        );
    }
    assert_eq!(next_dash_tick(18, 127), Some(18));
}

#[test]
fn the_stand_after_a_dash_still_tilts_and_crouches_before_the_interrupt_frame() {
    // Only the next dash waits: the fighter that let go of its dash is standing, as before.
    let mut sim = brawler();
    sim.tick(inp(127, 0, 0));
    sim.ticks(10, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().state, S::Crouch);
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

// ---- Dash attacks -------------------------------------------------------------------------------------------

fn brawler() -> Sim {
    let mut sim = Sim::with_chars([1, 0, 0, 0]);
    sim.stand(0, Fx::from_int(-10), 1);
    // Nobody in the way: bodies on the ground do not pass through each other.
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

#[test]
fn a_dash_attack_out_of_a_dash_keeps_sliding_forward_on_traction() {
    // From dash speed (2.09 reference units a frame) at the reference traction (0.11): about 20 reference units, 2.5 world units,
    // well past where a standing attack stops (sim v35; it slid a gentler 3.5 and more before).
    let mut sim = brawler();
    sim.ticks(8, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
    let x0 = sim.f().pos.x;
    sim.tick(inp(127, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::DashAttack as u8);
    sim.ticks(40, inp(127, 0, 0));
    let slid = sim.f().pos.x - x0;
    assert!(slid > fx(22, 10) && slid < fx(30, 10), "slid {slid:?}");
}

/// Lets go of the stick after a few frames of dash and waits for the dash to end by itself.
fn dash_then_let_go(sim: &mut Sim) {
    sim.ticks(8, inp(127, 0, 0));
    for _ in 0..20 {
        if sim.f().state != S::Dash {
            break;
        }
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn attack_just_after_letting_go_of_a_dash_is_still_a_dash_attack_not_a_jab() {
    let mut sim = brawler();
    dash_then_let_go(&mut sim); // the fighter is still sliding fast
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::DashAttack as u8);
}

#[test]
fn attack_once_the_slide_has_stopped_is_a_jab() {
    let mut sim = brawler();
    sim.ticks(8, inp(127, 0, 0));
    sim.ticks(30, inp(0, 0, 0));
    assert_eq!(sim.f().vel.x, Fx::ZERO);
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Jab as u8);
}

#[test]
fn attack_while_sliding_with_a_direction_held_is_still_a_tilt_or_smash() {
    let mut sim = brawler();
    dash_then_let_go(&mut sim);
    sim.tick(inp(0, 70, ATTACK)); // up tilt, not a dash attack
    assert_eq!(sim.f().move_id, MoveId::UTilt as u8);
}

// ---- The initial dash is committed ---------------------------------------------------------------

/// Where the fighter is `frames` frames after a dash started by a one-frame tap (then nothing held), and whether it was still
/// dashing on every one of them.
fn tapped_dash(frames: usize) -> (Fx, bool) {
    let mut sim = Sim::alone();
    let x0 = sim.f().pos.x;
    sim.tick(inp(127, 0, 0));
    let mut dashing = true;
    for _ in 1..frames {
        sim.tick(inp(0, 0, 0));
        dashing &= sim.f().state == S::Dash;
    }
    (sim.f().pos.x - x0, dashing)
}

#[test]
fn a_tap_covers_the_whole_initial_dash() {
    let sim = Sim::alone();
    let dash = usize::from(sim.content.fighters[0].dash_frames);
    let (tapped, dashing) = tapped_dash(dash);
    assert!(dashing, "letting go does not end the dash");
    // The same distance as a dash with the stick held all the way.
    let mut held = Sim::alone();
    let x0 = held.f().pos.x;
    held.ticks(dash, inp(127, 0, 0));
    assert_eq!(tapped, held.f().pos.x - x0, "a tap dashes as far as a hold");
    // The view draws the dash's stride from this, so it must be what the simulation does.
    assert_eq!(tapped, sim.content.fighters[0].initial_dash_distance());
    // Then it ends by itself, braking to a stand.
    let mut sim = Sim::alone();
    sim.tick(inp(127, 0, 0));
    sim.ticks(dash, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    // A second tap the same way does not restart or extend it.
    let mut again = Sim::alone();
    again.tick(inp(127, 0, 0));
    again.ticks(3, inp(0, 0, 0));
    again.tick(inp(127, 0, 0));
    again.ticks(dash - 4, inp(0, 0, 0));
    assert_eq!(again.f().state, S::Idle, "the second tap changed nothing");
}

#[test]
fn a_flick_the_other_way_still_dash_dances_out_of_a_tapped_dash() {
    let mut sim = Sim::new();
    sim.tick(inp(127, 0, 0));
    sim.ticks(3, inp(0, 0, 0));
    sim.tick(inp(-127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
    assert_eq!(sim.f().facing, -1);
}

#[test]
fn no_crouch_or_tilt_during_the_initial_dash() {
    let mut sim = Sim::new();
    sim.tick(inp(127, 0, 0));
    sim.ticks(3, inp(0, -70, 0));
    assert_eq!(
        sim.f().state,
        S::Dash,
        "holding down does not crouch out of it"
    );
    // A down tilt (or any tilt) is the dash attack.
    sim.tick(inp(0, -70, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::DashAttack as u8);
    let mut sim = Sim::new();
    sim.tick(inp(127, 0, 0));
    sim.ticks(3, inp(0, 0, 0));
    sim.tick(inp(0, 70, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::DashAttack as u8, "an up tilt too");
}

#[test]
fn a_smash_attack_can_come_out_of_the_initial_dash() {
    let mut sim = Sim::new();
    sim.tick(inp(127, 0, 0));
    sim.ticks(3, inp(127, 0, 0));
    sim.tick(inp(0, 127, STRONG | ATTACK));
    assert_eq!(sim.f().move_id, MoveId::USmash as u8);
    let mut sim = Sim::new();
    sim.tick(inp(127, 0, 0));
    sim.ticks(3, inp(127, 0, 0));
    sim.tick(inp(127, 0, STRONG | ATTACK));
    assert_eq!(sim.f().move_id, MoveId::FSmash as u8);
    assert_eq!(sim.f().facing, 1);
}

#[test]
fn the_shield_cuts_the_initial_dash_to_about_half() {
    let sim = Sim::alone();
    let p = sim.content.fighters[0];
    let half = usize::from(p.dash_shield_frame);
    assert!(
        half * 2 >= usize::from(p.dash_frames) - 1 && half * 2 <= usize::from(p.dash_frames) + 1
    );
    // Forward held and the shield pressed early: it comes up on `dash_shield_frame`, not before.
    let mut sim = Sim::alone();
    let x0 = sim.f().pos.x;
    sim.tick(inp(127, 0, 0));
    for frame in 1..=half {
        sim.tick(inp(127, 0, SHIELD));
        if frame < half {
            assert_eq!(sim.f().state, S::Dash, "frame {frame}: too early to shield");
        }
    }
    assert_eq!(sim.f().state, S::Shield);
    sim.ticks(30, inp(127, 0, SHIELD));
    let shielded = sim.f().pos.x - x0;
    // The whole dash (and its slide to a stop) goes much further.
    let mut full = Sim::alone();
    full.tick(inp(127, 0, 0));
    full.ticks(30 + half, inp(0, 0, 0));
    let whole = full.f().pos.x - x0;
    assert!(shielded > Fx::ZERO, "it did move");
    assert!(
        shielded < whole * fx(7, 10),
        "shield {shielded:?} against {whole:?}"
    );
}
