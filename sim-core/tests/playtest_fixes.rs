//! Sim 144: fixes and changes from the 19 October 2026 playtest (the owner's answers in `docs/PLAYTEST_2026-10-19.md`).

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{Fx, GameState};

const GAUNTLETS: u8 = 4;

/// Two fighters of `chars[0]` and `chars[1]`: player 0 at x = -7 facing right, player 1 `gap` ahead facing back.
fn duel(chars: [u8; 4], gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.state = GameState::new_with_active(&sim.content, 1, chars, 0b0011);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-7) + gap, -1);
    sim
}

/// Player 0 grabs player 1 standing just ahead, and lets go of every button. Returns the sim with player 0 holding.
fn holding() -> Sim {
    let mut sim = duel([0, 0, 0, 0], fx(15, 10));
    sim.tick(inp(0, 0, GRAB));
    for _ in 0..20 {
        if sim.f().state == S::Grabbing {
            break;
        }
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Grabbing);
    sim.tick(inp(0, 0, 0));
    sim
}

#[test]
fn any_button_pummels_but_a_button_held_from_before_does_not() {
    for button in [ATTACK, GRAB, SPECIAL, SHIELD, JUMP] {
        let mut sim = holding();
        sim.tick(inp(0, 0, button));
        assert_eq!(sim.f().move_id, MoveId::Pummel as u8, "button {button}");
        assert_eq!(sim.f().state, S::Attack, "button {button}");
    }
    // The grab button still held from the grab itself does not pummel.
    let mut sim = duel([0, 0, 0, 0], fx(15, 10));
    for _ in 0..20 {
        sim.tick(inp(0, 0, GRAB));
        if sim.f().state == S::Grabbing {
            break;
        }
    }
    sim.ticks(3, inp(0, 0, GRAB));
    assert_eq!(sim.f().state, S::Grabbing, "a held grab button holds");
}

#[test]
fn a_fighter_lying_on_the_ground_cannot_be_grabbed() {
    let mut sim = duel([0, 0, 0, 0], fx(15, 10));
    sim.state.fighters[1].state = S::Knockdown;
    sim.tick(inp(0, 0, GRAB));
    sim.ticks(20, inp(0, 0, 0));
    assert_ne!(sim.fighter(1).state, S::Grabbed);
    assert_eq!(sim.f().grab_with, sim_core::state::NONE);
}

#[test]
fn a_pivot_grab_never_catches_someone_behind_it() {
    // Dashing at a fighter ahead and pivot grabbing (turning away from it): the slide carries the grabber into that fighter, whose
    // body touches the grab box's near end. It used to be caught and pulled round to the front in one frame.
    for gap in [2, 6, 10, 14, 18] {
        let mut sim = duel([GAUNTLETS, 0, 0, 0], Fx::from_ratio(gap, 10));
        sim.ticks(5, inp(127, 0, 0));
        sim.tick(inp(-127, 0, GRAB));
        assert_eq!(sim.f().move_id, MoveId::PivotGrab as u8);
        for _ in 0..20 {
            sim.tick(inp(0, 0, 0));
        }
        assert_ne!(sim.fighter(1).state, S::Grabbed, "gap {gap}");
    }
    // (One ahead of the turned grabber is still caught: `kits.rs`, `a_pivot_grab_turns_around_out_of_a_dash_and_catches_behind`.)
}

/// The frames (1 = the frame of the input) on which fighter 0 is intangible after `first`, then nothing.
fn intangible_frames(mut sim: Sim, first: sim_core::Input) -> Vec<usize> {
    let mut on = Vec::new();
    for frame in 1..=60 {
        sim.tick(if frame == 1 { first } else { inp(0, 0, 0) });
        if is_intangible(sim.f()) {
            on.push(frame);
        }
    }
    on
}

#[test]
fn dodges_are_intangible_on_the_references_frames() {
    // The reference (ultimateframedata.com, 13.0.1) for the duelist's: spot dodge 3-17, neutral air dodge 3-29, rolls 4-15 forward
    // and 5-16 back.
    let shielding = || {
        let mut sim = Sim::with_chars([0, 0, 0, 0]);
        sim.state = GameState::new_with_active(&sim.content, 1, [0, 0, 0, 0], 0b0001);
        sim.stand(0, Fx::ZERO, 1);
        sim.ticks(10, inp(0, 0, SHIELD));
        sim
    };
    let span = |v: Vec<usize>| (v[0], *v.last().unwrap(), v.len());
    assert_eq!(
        span(intangible_frames(shielding(), inp(0, -127, SHIELD))),
        (3, 17, 15)
    );
    assert_eq!(
        span(intangible_frames(shielding(), inp(127, 0, SHIELD))),
        (4, 15, 12)
    );
    assert_eq!(
        span(intangible_frames(shielding(), inp(-127, 0, SHIELD))),
        (5, 16, 12)
    );
    let mut air = Sim::with_chars([0, 0, 0, 0]);
    air.state = GameState::new_with_active(&air.content, 1, [0, 0, 0, 0], 0b0001);
    air.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    air.tick(inp(0, 0, 0));
    assert_eq!(span(intangible_frames(air, inp(0, 0, SHIELD))), (3, 29, 27));
}

/// Forward roll, back roll, spot dodge and neutral air dodge totals, and the back roll's intangible frames.
type Dodges = (u8, u8, u8, u8, (u8, u8));

#[test]
fn every_body_rolls_and_dodges_on_the_references_totals() {
    // ultimateframedata.com, 13.0.1: forward roll, back roll, spot dodge, neutral air dodge totals, and the back roll's
    // intangibility.
    let c = sim_core::Content::placeholder();
    let want: [Dodges; 14] = [
        (29, 34, 25, 52, (5, 16)),
        (29, 34, 25, 44, (5, 16)),
        (26, 32, 23, 44, (4, 14)),
        (29, 34, 25, 52, (5, 16)),
        (30, 35, 26, 42, (5, 16)),
        (29, 34, 25, 49, (5, 16)),
        (29, 34, 25, 47, (5, 16)),
        (30, 35, 26, 48, (5, 16)),
        (30, 35, 26, 58, (5, 16)),
        (30, 35, 26, 49, (5, 16)),
        (29, 34, 25, 57, (5, 16)),
        (28, 33, 24, 44, (4, 15)),
        (29, 34, 25, 47, (5, 16)),
        (29, 34, 25, 44, (5, 16)),
    ];
    for (i, w) in want.iter().enumerate() {
        let p = &c.fighters[i];
        let got = (
            p.roll_frames,
            p.roll_back_frames,
            p.spot_dodge_frames,
            p.air_dodge_frames,
            (p.roll_back_intangible_start, p.roll_back_intangible_end),
        );
        assert_eq!(got, *w, "fighter {i} ({})", c.names.fighters[i]);
    }
}

#[test]
fn a_fighter_just_on_the_ledge_can_be_hit_for_two_frames() {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(-1), Fx::ZERO, fx(-1, 20));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::LedgeHang);
    assert!(sim.f().ledge_invuln > 0);
    let mut seen = Vec::new();
    for _ in 1..=4 {
        sim.tick(inp(0, 0, 0));
        seen.push(is_intangible(sim.f()));
    }
    assert_eq!(seen, vec![false, false, true, true]);
}

/// Player 0's horizontal speed on the first airborne frame of a jump out of a run, with the stick at `stick_x` on the jump.
fn run_jump_speed(stick_x: i8) -> Fx {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state = GameState::new_with_active(&sim.content, 1, [0, 0, 0, 0], 0b0001);
    sim.stand(0, Fx::from_int(-6), 1);
    sim.ticks(25, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    sim.tick(inp(stick_x, 0, JUMP));
    for _ in 0..10 {
        if sim.f().state == S::Airborne {
            break;
        }
        sim.tick(inp(stick_x, 0, JUMP));
    }
    sim.f().vel.x
}

#[test]
fn a_run_jump_carries_further_with_the_stick_forward() {
    // Sim 144 (the owner after a playtest): the run's kept share plus the stick's, within the body's maximum. (The stick back out of a
    // run is a reverse aerial rush, which keeps the run's share: sim 145, below.)
    let (neutral, forward) = (run_jump_speed(0), run_jump_speed(127));
    assert!(neutral <= forward, "{neutral:?} {forward:?}");
    let p = Sim::new().content.fighters[0];
    assert!(forward <= p.jump_speed_x_max);
    // A body whose run's share is under its maximum gains from the stick: the Claws.
    assert!(takeoff_speed(127, 25, 0) < takeoff_speed(127, 25, 127));
}

/// Player 0's horizontal speed on the first airborne frame of a jump made with `jump_stick`, after `before` frames holding `stick`.
fn takeoff_speed(stick: i8, before: usize, jump_stick: i8) -> Fx {
    let mut sim = Sim::with_chars([1, 0, 0, 0]);
    sim.state = GameState::new_with_active(&sim.content, 1, [1, 0, 0, 0], 0b0001);
    sim.stand(0, Fx::from_int(-8), 1);
    sim.ticks(before, inp(stick, 0, 0));
    for _ in 0..10 {
        sim.tick(inp(jump_stick, 0, JUMP));
        if sim.f().state == S::Airborne {
            break;
        }
    }
    sim.f().vel.x
}

#[test]
fn a_reverse_aerial_rush_carries_the_runs_momentum() {
    // Sim 145 (the owner after docs/MOMENTUM.md): the stick back with the jump out of a run turns the fighter round and keeps the run's
    // share whole (it took a standing jump's speed off it in sim 144: the Claws left at 0.23 reference units a frame).
    let (neutral, rar) = (takeoff_speed(127, 25, 0), takeoff_speed(127, 25, -127));
    assert_eq!(
        rar, neutral,
        "the run's share, as a jump with the stick let go"
    );
    assert!(rar > Fx::ZERO);
    // Out of a walk (slower than a walk's top speed), the stick back still jumps backward.
    let walk_back = takeoff_speed(40, 6, -127);
    assert!(
        walk_back < Fx::ZERO,
        "a backward jump out of a walk: {walk_back:?}"
    );
}
