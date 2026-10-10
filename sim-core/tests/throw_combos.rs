//! Throws as in the reference game (sim v37): the claws' forward, up and back throws are a hit and then the throw, and only the
//! throw's own damage sets the launch, so the combo throws lead into follow-ups at low percents and never knock out; the back throw
//! is the kill throw.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::GRAB;
use sim_core::state::FighterState as S;
use sim_core::Fx;

struct Thrown {
    /// The frame the thrower could act, and the frame the thrown fighter could.
    thrower_free: u32,
    victim_free: u32,
    /// Knocked out through a side or the top (not by falling, which a fighter that does nothing always would).
    launched_out: bool,
}

/// Player 0 (`ch`) grabs player 1 standing at `pct` and throws it with `stick`; `at_ledge` puts them at the right ledge with the
/// thrower facing the middle.
fn throw(ch: u8, stick: (i8, i8), pct: i32, at_ledge: bool) -> Thrown {
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.fighters[1].stocks = 3;
    if at_ledge {
        sim.stand(0, Fx::from_int(9), -1);
        sim.stand(1, fx(75, 10), 1);
    } else {
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(15, 10), -1);
    }
    sim.state.fighters[1].percent = Fx::from_int(pct);
    sim.tick(inp(0, 0, GRAB));
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).state, S::Grabbed, "not grabbed");
    let facing = sim.f().facing;
    sim.tick(inp(stick.0 * facing, stick.1, 0));
    let stocks = sim.fighter(1).stocks;
    let mut out = Thrown {
        thrower_free: 0,
        victim_free: 0,
        launched_out: false,
    };
    let mut launched = false;
    let mut last_y = Fx::ZERO;
    for t in 1..400u32 {
        sim.tick(inp(0, 0, 0));
        if out.thrower_free == 0 && sim.f().state != S::Attack {
            out.thrower_free = t;
        }
        let v = sim.fighter(1);
        launched |= v.state == S::Hitstun;
        if launched && out.victim_free == 0 && v.state != S::Hitstun && v.hitlag == 0 {
            out.victim_free = t;
        }
        if v.stocks < stocks {
            out.launched_out = last_y > Fx::from_int(-8);
            break;
        }
        last_y = v.pos.y;
    }
    out
}

const FORWARD: (i8, i8) = (100, 0);
const BACK: (i8, i8) = (-100, 0);
const UP: (i8, i8) = (0, 100);
const DOWN: (i8, i8) = (0, -100);

#[test]
fn the_claws_up_throw_leaves_time_for_a_follow_up_at_low_percent() {
    let t = throw(1, UP, 0, false);
    assert!(
        t.victim_free >= t.thrower_free + 10,
        "the thrower is free at {} and the victim at {}",
        t.thrower_free,
        t.victim_free
    );
}

#[test]
fn the_combo_throws_never_knock_out() {
    for ch in [0, 1] {
        for stick in [FORWARD, UP, DOWN] {
            for pct in [100, 150] {
                assert!(
                    !throw(ch, stick, pct, false).launched_out,
                    "class {ch} throw {stick:?} knocked out from mid-stage at {pct}%"
                );
            }
        }
    }
}

#[test]
fn the_claws_back_throw_is_the_kill_throw() {
    assert!(!throw(1, BACK, 0, true).launched_out, "not at 0%");
    assert!(
        throw(1, BACK, 160, true).launched_out,
        "from the ledge at 160%"
    );
}
