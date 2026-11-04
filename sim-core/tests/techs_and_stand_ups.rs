//! The tech in place and the stand-ups from the floor on the reference's frames (sim 150, D8; a public dump of the reference's game
//! data, the same on every body we have ported). A tech in place keeps the fighter on the ground for 26 frames, intangible 1-20;
//! standing up from lying takes 29 frames, intangible 1-22, whether pressed or by itself; standing up from a trip, hard or soft, 21,
//! intangible 1-16. Frame 1 is the landing, or the frame the stand-up begins on. Before, a tech in place could act after 4 frames
//! (18 intangible) and both stand-ups took 26 frames, intangible from the second frame through the sixteenth.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::{JUMP, SHIELD};
use sim_core::state::{FighterState as S, NONE};
use sim_core::{Fx, Vec2};

/// Body `ch` as player 0 in the middle of the stage facing `facing`, with player 1 standing at the far end and the others out.
fn alone(ch: u8, facing: i8) -> Sim {
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.stand(1, fx(10, 1), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, facing);
    sim
}

/// Body `ch` facing `facing` falls tumbling onto the stage with shield pressed just before landing (`then` held after the press):
/// the landing frame, the tech's first.
fn tech_in_place(ch: u8, facing: i8, then: u16) -> Sim {
    let mut sim = alone(ch, facing);
    let f = &mut sim.state.fighters[0];
    f.state = S::Hitstun;
    f.hitstun = 80;
    f.tumble = true;
    f.pos = Vec2::new(Fx::ZERO, fx(5, 10));
    f.vel = Vec2::new(Fx::ZERO, fx(-2, 10));
    f.platform = NONE;
    sim.tick(inp(0, 0, SHIELD));
    for _ in 0..8 {
        sim.tick(inp(0, 0, then));
        if sim.f().state != S::Hitstun {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Landing, "body {ch}: a tech in place");
    sim
}

/// Body `ch` lying in a knockdown (sitting after a trip, with `tripped`; a soft one with `soft`) long enough to get up.
fn lying(ch: u8, tripped: bool, soft: bool) -> Sim {
    let mut sim = alone(ch, 1);
    let f = &mut sim.state.fighters[0];
    f.state = S::Knockdown;
    f.tripped = tripped;
    f.trip_soft = soft;
    f.state_frame = 60;
    sim
}

/// The state begun this frame (its frame 1) played out with `held`: how many frames it lasted and the frames it was intangible on.
fn play(sim: &mut Sim, held: sim_core::Input) -> (u16, Vec<u16>) {
    let state = sim.f().state;
    let mut frame = 1u16;
    let mut intangible = Vec::new();
    while sim.f().state == state {
        if is_intangible(sim.f()) {
            intangible.push(frame);
        }
        sim.tick(held);
        frame += 1;
        assert!(frame < 200, "{state:?} never ended");
    }
    (frame - 1, intangible)
}

fn through(last: u16) -> Vec<u16> {
    (1..=last).collect()
}

fn bodies() -> u8 {
    Sim::new().content.fighters.len() as u8
}

#[test]
fn the_rulesets_tech_in_place_and_stand_ups_are_the_references() {
    let r = Sim::new().content.rules;
    assert_eq!((r.tech_lag, r.tech_invuln), (26, 20));
    assert_eq!((r.getup_frames, r.getup_intangible), (29, 22));
    assert_eq!((r.trip_getup_frames, r.trip_getup_intangible), (21, 16));
}

#[test]
fn every_body_techs_in_place_for_26_frames_intangible_from_the_landing_through_frame_20() {
    for ch in 0..bodies() {
        for facing in [1i8, -1] {
            let mut sim = tech_in_place(ch, facing, 0);
            let (frames, intangible) = play(&mut sim, inp(0, 0, 0));
            let at = format!("body {ch} facing {facing}");
            assert_eq!(frames, 26, "{at}");
            assert_eq!(intangible, through(20), "{at}");
            assert_eq!(sim.f().facing, facing, "{at}");
        }
    }
}

#[test]
fn a_tech_in_place_cannot_be_jumped_out_of_before_it_ends() {
    // Jump held from the landing: no jump until the tech's 26 frames are over (it used to come out after 4).
    for ch in 0..bodies() {
        let mut sim = tech_in_place(ch, 1, JUMP);
        let (frames, _) = play(&mut sim, inp(0, 0, JUMP));
        assert_eq!(frames, 26, "body {ch}");
        sim.tick(inp(0, 0, JUMP));
        assert_eq!(
            sim.f().state,
            S::JumpSquat,
            "body {ch}: the held jump comes out as it ends"
        );
    }
}

#[test]
fn every_body_stands_up_from_lying_in_29_frames_intangible_through_frame_22() {
    for ch in 0..bodies() {
        // Stick up, jump, or by itself.
        for how in 0..3 {
            let mut sim = lying(ch, false, false);
            if how == 2 {
                sim.state.fighters[0].state_frame = u16::from(sim.content.rules.knockdown_max) - 1;
                sim.tick(inp(0, 0, 0));
            } else {
                let held = if how == 0 {
                    inp(0, 100, 0)
                } else {
                    inp(0, 0, JUMP)
                };
                sim.tick(held);
            }
            assert_eq!(sim.f().state, S::GetUp, "body {ch}, way {how}");
            let (frames, intangible) = play(&mut sim, inp(0, 0, 0));
            assert_eq!(frames, 29, "body {ch}, way {how}");
            assert_eq!(intangible, through(22), "body {ch}, way {how}");
            assert_eq!(sim.f().state, S::Idle, "body {ch}, way {how}");
        }
    }
}

#[test]
fn every_body_stands_up_from_a_trip_in_21_frames_intangible_through_frame_16() {
    for ch in 0..bodies() {
        for soft in [false, true] {
            let mut sim = lying(ch, true, soft);
            sim.tick(inp(0, 100, 0));
            assert_eq!(sim.f().state, S::GetUp, "body {ch}, soft {soft}");
            let (frames, intangible) = play(&mut sim, inp(0, 0, 0));
            assert_eq!(frames, 21, "body {ch}, soft {soft}");
            assert_eq!(intangible, through(16), "body {ch}, soft {soft}");
        }
    }
}

#[test]
fn a_trip_left_to_sit_stands_up_by_itself_on_the_trips_own_frames() {
    let mut sim = lying(0, true, false);
    sim.state.fighters[0].state_frame = u16::from(sim.content.rules.trip_max) - 1;
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::GetUp);
    let (frames, intangible) = play(&mut sim, inp(0, 0, 0));
    assert_eq!((frames, intangible), (21, through(16)));
}
