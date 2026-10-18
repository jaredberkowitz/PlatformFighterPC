//! How high the hauler's and the sprinter's up specials rise (sim 143, owner decision 102: "does not gain any height at all", "should
//! gain more height"). Measured from a standstill in the air and from the ground, in reference units (world units x 8). Sources:
//! `docs/movesets/bracers/REFERENCE_DATA.md` (the spin's published speeds) and `docs/movesets/gauntlets/REFERENCE_DATA.md` ("Fitted
//! distances": the leap fitted to the reference's hitbox visualisation).

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::state::FighterState as S;
use sim_core::Fx;

const HAULER: u8 = 7;
const SPRINTER: u8 = 4;

fn sim_for(c: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    sim.stand(1, Fx::from_int(60), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// Reference units as world units.
fn su(n: i32) -> Fx {
    Fx::from_int(n) / Fx::from_int(8)
}

/// Starts the up special (stick up, special tapped) and returns the height above the start, frame by frame, for `frames` frames.
fn heights(sim: &mut Sim, frames: u32) -> Vec<Fx> {
    let y0 = sim.f().pos.y;
    (1..=frames)
        .map(|t| {
            sim.tick(inp(0, 127, if t == 1 { SPECIAL } else { 0 }));
            sim.f().pos.y - y0
        })
        .collect()
}

fn in_air(c: u8) -> Sim {
    let mut sim = sim_for(c);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim
}

fn on_ground(c: u8) -> Sim {
    let mut sim = sim_for(c);
    sim.stand(0, Fx::ZERO, 1);
    sim
}

fn peak(h: &[Fx]) -> (usize, Fx) {
    h.iter().copied().enumerate().fold(
        (0, Fx::ZERO),
        |b, (i, y)| if y > b.1 { (i + 1, y) } else { b },
    )
}

#[test]
fn the_haulers_aerial_spin_rises_through_the_whole_move() {
    // Initial vertical speed 0.9, slowed by the body's gravity (0.085) times the published multiplier 0.07 [R]: still rising when the
    // spin ends (frame 70; the special fall then tops it out a few frames later), about 48 reference units up [D]. Before sim 143 it peaked about 6 units up on frame 13, then sank.
    let h = heights(&mut in_air(HAULER), 120);
    let (at, top) = peak(&h);
    assert!(top > su(46) && top < su(51), "peak {top:?}");
    assert!((70..=78).contains(&at), "peak on frame {at}");
    assert!(h[12] > su(10), "already well up on frame 13");
}

#[test]
fn the_haulers_grounded_spin_stays_on_the_ground() {
    let mut sim = on_ground(HAULER);
    let h = heights(&mut sim, 100);
    assert!(h.iter().all(|&y| y == Fx::ZERO));
}

#[test]
fn the_sprinters_leap_holds_still_then_rises_to_the_fitted_peak() {
    // In the air: no drop through the startup (frames 1-12), a peak on frame 45 about 53 units up, then the flip down to about 15
    // units above the start as the special fall begins (frame 66) [E: fitted]. Before sim 143 it fell 9 units in the startup and
    // peaked about 25 units up.
    let mut sim = in_air(SPRINTER);
    let h = heights(&mut sim, 66);
    assert!(h[..12].iter().all(|&y| y == Fx::ZERO), "{:?}", &h[..12]);
    let (at, top) = peak(&h);
    assert!(top > su(51) && top < su(55), "air peak {top:?}");
    assert!((44..=47).contains(&at), "air peak on frame {at}");
    assert!(h[65] > su(12) && h[65] < su(18), "air end {:?}", h[65]);
    assert_eq!(sim.f().state, S::Helpless);

    // From the ground the take-off is stronger: about 62 units up.
    let h = heights(&mut on_ground(SPRINTER), 66);
    let (at, top) = peak(&h);
    assert!(top > su(60) && top < su(65), "ground peak {top:?}");
    assert!((44..=47).contains(&at), "ground peak on frame {at}");
}
