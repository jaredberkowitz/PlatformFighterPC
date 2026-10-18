//! How much speed a fighter keeps along the ground as it lands (sim 143, owner decision 102: "I slid way too far across the stage by
//! doing up special pointed towards the stage into the stage"). The ruleset's `landing_speed_cap` (100: the body's dash speed) caps a
//! landing out of a jump, a move or a special fall; the slide then bleeds off at the body's `landing_friction` as before. The reference
//! publishes no landing rule [E]; `docs/movesets/SHARED_MECHANICS.md` ("Landing speed").

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{JUMP, SPECIAL};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const EXILE: u8 = 13;

fn sim_for(c: u8, cap: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    sim.content.rules.landing_speed_cap = cap;
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// Class `c` in the air near the left edge (3 units up), facing the stage; it starts a special with stick `(sx, sy)` and then holds
/// `aim` until it lands. Returns its speed along the ground on landing, how far it slides from there, and whether it stayed on the stage.
fn into_the_stage(c: u8, cap: u8, (sx, sy): (i8, i8), aim: (i8, i8)) -> Option<(Fx, Fx, bool)> {
    let mut sim = sim_for(c, cap);
    sim.put_airborne(0, Fx::from_int(-10), Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    let mut landed: Option<(Fx, Fx)> = None;
    for t in 1..=300u32 {
        let i = match (t, landed) {
            (1, _) => inp(sx, sy, SPECIAL),
            (_, None) => inp(aim.0, aim.1, 0),
            _ => inp(0, 0, 0),
        };
        sim.tick(i);
        let f = sim.f();
        if landed.is_none() && f.state == S::Landing {
            landed = Some((f.vel.x, f.pos.x));
        }
    }
    let (v, x0) = landed?;
    let f = sim.f();
    Some((v, f.pos.x - x0, f.grounded()))
}

#[test]
fn the_exiles_blade_dash_into_the_stage_keeps_at_most_its_dash_speed() {
    let p = sim_for(EXILE, 100).content.fighters[usize::from(EXILE)];
    let su = |n: i32| Fx::from_int(n) / Fx::from_int(8);
    // Aimed down and forward into the stage (the stick at 45 degrees, then 20 below level).
    for aim in [(127i8, -127i8), (127, -40)] {
        let (v_old, slide_old, kept_old) =
            into_the_stage(EXILE, 0, (90, 90), aim).expect("it lands (no cap)");
        let (v, slide, kept) = into_the_stage(EXILE, 100, (90, 90), aim).expect("it lands");
        // Before sim 143: the dash's 11 units a frame (less a frame's friction) carried on into the landing: it slid off the stage
        // from the steep aim, about 59 reference units from the shallow one.
        assert!(v_old > p.dash_speed, "aim {aim:?}: {v_old:?}");
        assert!(
            !kept_old || slide_old > su(55),
            "aim {aim:?}: {slide_old:?}"
        );
        // Since: at most the dash speed (less a frame of the landing friction), about 37 units of slide, and still on the stage.
        assert!(v <= p.dash_speed && v > Fx::ZERO, "aim {aim:?}: {v:?}");
        assert!(kept, "aim {aim:?}: stayed on the stage");
        assert!(slide < su(40), "aim {aim:?}: slid {slide:?}");
    }
}

/// Every built-in fighter's specials, started near the edge and steered into the stage: none lands with more than its body's dash speed
/// along the ground. The cap changes only the landings that were faster (measured: the exile's blade dash and the popgun's flip).
#[test]
fn no_special_lands_faster_than_its_bodys_dash_speed() {
    for c in 0u8..14 {
        let dash = sim_for(c, 100).content.fighters[usize::from(c)].dash_speed;
        for stick in [(0i8, 0i8), (127, 0), (90, 90), (0, -127)] {
            for aim in [(127i8, -127i8), (127, -40)] {
                if let Some((v, _, _)) = into_the_stage(c, 100, stick, aim) {
                    assert!(
                        v.abs() <= dash,
                        "class {c} stick {stick:?} aim {aim:?}: {v:?}"
                    );
                }
            }
        }
    }
}

/// A landing that was not faster than the dash speed is untouched by the cap: a running full hop landing back on the stage.
#[test]
fn a_running_jumps_landing_is_unchanged_by_the_cap() {
    let run = |cap: u8| {
        let mut sim = sim_for(0, cap);
        sim.stand(0, Fx::from_int(-9), 1);
        sim.ticks(12, inp(127, 0, 0));
        sim.ticks(4, inp(127, 0, JUMP));
        let mut out = Vec::new();
        for _ in 0..80 {
            sim.tick(inp(0, 0, 0));
            out.push((sim.f().pos, sim.f().vel));
        }
        out
    };
    assert_eq!(run(0), run(100));
}
