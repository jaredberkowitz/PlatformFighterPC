//! Buffered dashes out of landing lag: a dash flicked while the fighter is still in a landing (an aerial's landing lag included) comes
//! out on the first frame it can act, as the reference game's input buffer allows, as long as the stick is still held past the dash
//! threshold. Aerials landing inside their autocancel windows take only the plain landing lag (tested with the move data elsewhere).
//! And a jump out of a run lands back into the run while the stick is still held the run's way.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP};
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
fn a_stick_already_held_in_the_air_walks_out_of_the_landing() {
    // A dash flicked at any point of the landing lag is kept (sim v82, `grab_dash.rs`), but a stick that was pushed before the
    // landing, in the air, and held through it is a walk, not a dash: the flick was not made during the lag.
    for tenths in 2..80 {
        let mut sim = Sim::alone();
        let ground = sim.f().pos.y;
        let x = sim.f().pos.x;
        sim.put_airborne(
            0,
            x,
            ground + Fx::from_ratio(tenths, 10),
            Fx::ZERO,
            Fx::ZERO,
        );
        sim.tick(inp(0, -127, ATTACK));
        // The flick, in the air, then held.
        sim.tick(inp(0, 0, 0));
        let mut n = 0;
        while sim.f().state != S::Landing && n < 120 {
            sim.tick(inp(127, 0, 0));
            n += 1;
        }
        if sim.f().state != S::Landing || sim.f().lag < 13 {
            continue;
        }
        for _ in 0..usize::from(sim.f().lag) + 3 {
            sim.tick(inp(127, 0, 0));
        }
        assert!(
            matches!(sim.f().state, S::Walk | S::Idle),
            "dashed from a flick made in the air: {:?}",
            sim.f().state
        );
        return;
    }
    panic!("no drop height lands a long landing");
}

/// Player 0 where it spawns, as the movement tests start (room to run right).
fn open_stage() -> Sim {
    Sim::new()
}

/// A jump taken `jump_from` (a run when true, standing otherwise), with `stick` held through the air and the landing, then a few frames
/// after: the state the fighter is in once it has landed and its landing has ended.
fn jump_and_land(jump_from_run: bool, stick: i8) -> S {
    let mut sim = open_stage();
    if jump_from_run {
        sim.tick(inp(0, 0, 0));
        for _ in 0..24 {
            sim.tick(inp(127, 0, 0));
        }
        assert_eq!(sim.f().state, S::Run, "running before the jump");
        sim.tick(inp(127, 0, JUMP));
    } else {
        sim.tick(inp(0, 0, JUMP));
    }
    let mut was_airborne = false;
    for _ in 0..120 {
        sim.tick(inp(stick, 0, 0));
        let s = sim.f().state;
        if !sim.f().grounded() {
            was_airborne = true;
        } else if was_airborne && !matches!(s, S::Landing | S::JumpSquat) {
            return s;
        }
    }
    panic!("never landed: {:?}", sim.f().state);
}

#[test]
fn a_running_jump_lands_back_into_the_run_while_the_stick_is_held() {
    assert_eq!(jump_and_land(true, 127), S::Run);
}

#[test]
fn a_running_jump_landed_with_the_stick_let_go_stands() {
    assert_eq!(jump_and_land(true, 0), S::Idle);
}

#[test]
fn a_standing_jump_landed_holding_the_stick_walks_not_runs() {
    assert!(matches!(jump_and_land(false, 127), S::Walk | S::Idle));
}
