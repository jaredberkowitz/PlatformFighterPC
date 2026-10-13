//! Buffered dashes out of landing lag: a dash flicked while the fighter is still in a landing (an aerial's landing lag included) comes
//! out on the first frame it can act, as the reference game's input buffer allows, as long as the stick is still held past the dash
//! threshold. Aerials landing inside their autocancel windows take only the plain landing lag (tested with the move data elsewhere).

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::ATTACK;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// Player 0 dropped from `height` above the ground into an aerial (`stick_y` 0 a neutral air, -127 a down air), stepped until it lands.
/// `None` if it never lands in a landing.
fn land_out_of_aerial(height: Fx, stick_y: i8) -> Option<Sim> {
    let mut sim = Sim::alone();
    let ground = sim.f().pos.y;
    let x = sim.f().pos.x;
    sim.put_airborne(0, x, ground + height, Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, stick_y, ATTACK));
    for _ in 0..120 {
        if sim.f().state == S::Landing {
            return Some(sim);
        }
        sim.tick(inp(0, 0, 0));
    }
    None
}

/// A landing out of an aerial with at least `min_lag` frames of landing lag (found by trying drop heights, neutral then down airs).
fn long_landing(min_lag: u8) -> Sim {
    for stick_y in [0, -127] {
        for tenths in 2..80 {
            if let Some(sim) = land_out_of_aerial(Fx::from_ratio(tenths, 10), stick_y) {
                if sim.f().lag >= min_lag {
                    return sim;
                }
            }
        }
    }
    panic!("no drop height lands an aerial with {min_lag} frames of landing lag");
}

#[test]
fn a_dash_flicked_during_an_aerials_landing_lag_comes_out_when_the_lag_ends() {
    for dir in [1i8, -1] {
        let mut sim = long_landing(6);
        let lag = usize::from(sim.f().lag);
        // Neutral through most of the lag, then a flick a few frames before it ends, held.
        let flick_at = lag.saturating_sub(5).max(1);
        for _ in 0..flick_at {
            sim.tick(inp(0, 0, 0));
        }
        let mut dashed = false;
        for _ in 0..(lag + 3) {
            sim.tick(inp(127 * dir, 0, 0));
            if sim.f().state == S::Dash {
                dashed = true;
                break;
            }
            assert!(
                matches!(sim.f().state, S::Landing | S::Idle),
                "unexpected {:?}",
                sim.f().state
            );
        }
        assert!(dashed, "no dash out of a {lag}-frame landing (flick {dir})");
        assert_eq!(sim.f().facing, dir, "the dash goes the flicked way");
    }
}

#[test]
fn a_dash_flicked_on_the_first_frame_of_a_short_landing_comes_out() {
    // The plain landing lag (an autocancelled aerial lands like this): a flick the moment it touches down dashes as soon as it can.
    let mut sim = Sim::alone();
    let ground = sim.f().pos.y;
    let x = sim.f().pos.x;
    sim.put_airborne(0, x, ground + Fx::from_ratio(1, 2), Fx::ZERO, Fx::ZERO);
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Landing);
    let lag = usize::from(sim.f().lag);
    let mut dashed = false;
    for _ in 0..(lag + 3) {
        sim.tick(inp(127, 0, 0));
        if sim.f().state == S::Dash {
            dashed = true;
            break;
        }
    }
    assert!(dashed, "no dash out of the plain {lag}-frame landing");
}

#[test]
fn a_held_stick_with_no_recent_flick_still_walks() {
    // Outside the buffer, a stick pushed long ago and still held is a walk, not a dash: the window only reaches back the input buffer.
    let mut sim = long_landing(13);
    for _ in 0..usize::from(sim.f().lag) + 3 {
        sim.tick(inp(127, 0, 0));
    }
    assert!(
        matches!(sim.f().state, S::Walk | S::Idle),
        "dashed from an old flick: {:?}",
        sim.f().state
    );
}
