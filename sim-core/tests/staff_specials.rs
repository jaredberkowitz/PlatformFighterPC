//! The Staff's specials (class 5, the seer body; phase 2), modelled mechanically on a reference character: frame data, movement
//! and the behaviour of each special against `docs/movesets/palutena/REFERENCE_DATA.md` ("Specials"), and the extensions they
//! brought.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{MoveId, Weapon};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const SEER: u8 = 5;

/// The seer (player 0) and a duelist (player 1, parked at the far right), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SEER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn staff(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SEER)].weapon)]
}

fn su(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

fn near(a: Fx, b: Fx, tolerance_thousandths: i32) -> bool {
    (a - b).abs() <= Fx::from_ratio(tolerance_thousandths, 1000)
}

// ---- Up special: the warp ------------------------------------------------------------------------------------------------

#[test]
fn the_warps_data_is_the_references() {
    let sim = duel();
    let warp = staff(&sim).get(MoveId::UpSpecial as u8);
    assert_eq!(u16::from(warp.total_frames) + 2, 68, "FAF 68");
    assert!(warp.hitboxes.is_empty(), "no hitbox");
    // Intangible on the reference's 16-32 (move frames 15-31).
    assert_eq!((warp.intangible_from, warp.intangible), (15, 32));
    // Startup gravity 0.03 and top fall speed 0.75 (reference units), frames 1-15.
    let startup = warp.air_blocks[0];
    assert_eq!((startup.from, startup.to), (0, 14));
    assert_eq!(
        (startup.gravity, startup.max_fall),
        (Some(su(30)), Some(su(750)))
    );
    assert!(warp.helpless_after);
    assert_eq!((warp.landing_lag, warp.fall_landing_lag), (22, 22));
    assert_eq!(
        warp.lands_from + 1,
        33,
        "touching down before the reappearance carries on"
    );
    assert_eq!(
        (warp.ledge_front_from + 1, warp.ledge_back_from + 1),
        (33, 33)
    );
}

/// Starts the warp in the air at (x, y) with the stick up, then holds `aim` from the next frame on. Returns the sim after the
/// starting frame (move frame 0).
fn air_warp(x: Fx, y: Fx) -> Sim {
    let mut sim = duel();
    sim.put_airborne(0, x, y, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    sim
}

#[test]
fn the_warp_travels_59_5_reference_units_straight_up_intangible_on_16_to_32() {
    let mut sim = air_warp(Fx::ZERO, Fx::from_int(8));
    let mut y = [Fx::ZERO; 67];
    for frame in 1..=40u16 {
        sim.tick(inp(0, 127, 0));
        assert_eq!(sim.f().state_frame, frame);
        y[usize::from(frame)] = sim.f().pos.y;
        // (Reference frame = move frame + 1.)
        let reference = frame + 1;
        assert_eq!(
            is_intangible(sim.f()),
            (16..=32).contains(&reference),
            "intangible on the reference's frames 16-32 only (frame {reference})"
        );
    }
    // The startup falls under gravity 0.03 (14 frames of it after the starting one), never past 0.75.
    assert!(y[14] < Fx::from_int(8));
    // 17 frames at 3.5 reference units a frame: 59.5 units, 7.4375 world units.
    assert_eq!(y[31] - y[14], su(3500).mul_int(17));
    // The reappearance: half the speed once.
    assert_eq!(y[32] - y[31], su(1750));
    assert_eq!(sim.f().pos.x, Fx::ZERO, "straight up");
}

#[test]
fn the_stick_on_frame_16_aims_the_warp_and_later_changes_do_not() {
    // Up and forward at 45 degrees, then the stick swung back down: the warp keeps the first direction.
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(8));
    sim.ticks(14, inp(0, 0, 0));
    let start = sim.f().pos;
    sim.tick(inp(90, 90, 0));
    sim.ticks(16, inp(-127, -127, 0));
    let moved = sim.f().pos - start;
    let leg = su(3500).mul_int(17) * fx(7071, 10000);
    assert!(near(moved.x, leg, 5), "forward {moved:?}");
    assert!(near(moved.y, leg, 5), "up {moved:?}");

    // A neutral stick on frame 16 warps straight up; the stick's tilt does not change the distance.
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(8));
    sim.ticks(14, inp(0, 0, 0));
    let start = sim.f().pos;
    sim.ticks(17, inp(0, 0, 0));
    assert_eq!(sim.f().pos.x, start.x);
    assert_eq!(sim.f().pos.y - start.y, su(3500).mul_int(17));
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(8));
    sim.ticks(14, inp(0, 0, 0));
    let start = sim.f().pos;
    sim.tick(inp(50, 0, 0));
    sim.ticks(16, inp(0, 0, 0));
    assert!(near(sim.f().pos.x - start.x, su(3500).mul_int(17), 2));
}

#[test]
fn an_aerial_warp_ends_helpless_and_lands_with_22_frames() {
    let mut sim = air_warp(Fx::ZERO, Fx::from_int(12));
    sim.ticks(66, inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Helpless,
        "FAF 68: helpless after the reappearance"
    );
    let mut frames = 0;
    while sim.f().state == S::Helpless && frames < 600 {
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 22);
}

#[test]
fn landing_during_the_reappearance_costs_22_frames() {
    // A sideways warp low over the stage: the reappearance falls onto it before frame 68.
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(2));
    sim.ticks(14, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    let mut landed_on = None;
    for _ in 16..=66 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            landed_on = Some(sim.f().state_frame);
            break;
        }
    }
    assert!(landed_on.is_some(), "landed during the reappearance");
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 22);
}

#[test]
fn a_warp_into_the_ground_glides_along_it_and_reappears_standing() {
    // Down and forward from just above the stage: it meets the ground, glides on along it and ends standing (no landing lag).
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(1));
    sim.ticks(14, inp(0, 0, 0));
    let x0 = sim.f().pos.x;
    sim.tick(inp(90, -90, 0));
    sim.ticks(16, inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Attack,
        "still warping after touching down"
    );
    assert!(sim.f().grounded());
    assert_eq!(sim.f().pos.y, Fx::ZERO);
    assert!(sim.f().pos.x > x0 + Fx::from_int(4), "it glided on");
    sim.ticks(35, inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Idle,
        "a grounded reappearance ends standing"
    );
}

#[test]
fn a_grounded_warp_along_the_stage_ends_standing_and_one_sliding_off_an_edge_is_cancelled() {
    // Sideways along the ground: it stays on the ground and ends standing at FAF 68.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-6), 1);
    sim.tick(inp(0, 127, SPECIAL));
    sim.ticks(14, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    sim.ticks(16, inp(0, 0, 0));
    assert!(sim.f().grounded());
    assert!(near(
        sim.f().pos.x,
        Fx::from_int(-6) + su(3500).mul_int(17),
        1
    ));
    sim.ticks(34, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);

    // Near the right edge: the travel stays on the stage, the reappearance slides off it and is cancelled into an ordinary fall.
    let mut sim = duel();
    sim.stand(0, fx(32, 10), 1);
    sim.tick(inp(0, 127, SPECIAL));
    sim.ticks(14, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    sim.ticks(17, inp(0, 0, 0));
    assert!(sim.f().grounded(), "the reappearance starts on the stage");
    let mut fell = false;
    for _ in 33..=66 {
        sim.tick(inp(0, 0, 0));
        assert_ne!(
            sim.f().state,
            S::Helpless,
            "an edge-cancelled warp is not helpless"
        );
        if sim.f().state == S::Airborne {
            fell = true;
            break;
        }
    }
    assert!(fell, "slid off the edge during the reappearance");
}

#[test]
fn the_warp_catches_a_ledge_from_the_reappearance_on() {
    // Below the left ledge, facing the stage: the warp straight up passes into reach on frame 32 but catches only from 33.
    let half_width = duel().content.fighters[usize::from(SEER)].ecb_half_width;
    let mut sim = air_warp(Fx::from_int(-11) - half_width - fx(2, 10), fx(-95, 10));
    let mut caught = None;
    for frame in 1..=40u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            caught = Some(frame);
            break;
        }
    }
    let frame = caught.expect("caught the ledge");
    assert!(
        frame + 1 >= 33,
        "not before the reappearance (caught on {})",
        frame + 1
    );
}
