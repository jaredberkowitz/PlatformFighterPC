//! Bodies in the air: fighters cannot drift or jump through each other unless they air dodge (as on the ground, where they cannot
//! walk or run through each other). Bodies only meet where they overlap in height too, so a fighter well above another's head
//! passes over it.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::SHIELD;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// Player 0 at x -2 and player 1 at x 0, both in the air at heights `y0` and `y1`, the others out of the way.
fn two_in_the_air(y0: i32, y1: i32) -> Sim {
    let mut sim = Sim::new();
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.put_airborne(0, Fx::from_int(-2), Fx::from_int(y0), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, Fx::ZERO, Fx::from_int(y1), Fx::ZERO, Fx::ZERO);
    sim
}

#[test]
fn drifting_into_a_fighter_in_the_air_stops_against_it() {
    let mut sim = two_in_the_air(12, 12);
    for _ in 0..40 {
        sim.tick(inp(127, 0, 0));
        assert!(
            sim.fighter(0).pos.x < sim.fighter(1).pos.x,
            "drifted through at {:?}",
            sim.fighter(0).pos.x
        );
    }
}

#[test]
fn an_air_dodge_passes_through_a_fighter_in_the_air() {
    let mut sim = two_in_the_air(12, 12);
    sim.tick(inp(127, 0, SHIELD));
    assert_eq!(sim.f().state, S::AirDodge);
    for _ in 0..30 {
        sim.tick(inp(127, 0, 0));
    }
    assert!(
        sim.fighter(0).pos.x > sim.fighter(1).pos.x,
        "the air dodge did not pass through"
    );
}

#[test]
fn a_fighter_well_above_anothers_head_drifts_over_it() {
    let mut sim = two_in_the_air(20, 12);
    for _ in 0..30 {
        sim.tick(inp(127, 0, 0));
    }
    assert!(
        sim.fighter(0).pos.x > sim.fighter(1).pos.x,
        "blocked by a body far below"
    );
}

#[test]
fn drifting_into_a_standing_fighter_stops_against_it() {
    let mut sim = Sim::new();
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::ZERO, -1);
    // Player 0 just above the ground beside player 1, drifting into it.
    let ground = sim.fighter(1).pos.y;
    sim.put_airborne(
        0,
        Fx::from_int(-2),
        ground + Fx::from_ratio(1, 2),
        Fx::ZERO,
        Fx::ZERO,
    );
    for _ in 0..30 {
        sim.tick(inp(127, 0, 0));
        assert!(
            sim.fighter(0).pos.x < sim.fighter(1).pos.x,
            "passed through the standing fighter"
        );
    }
}
