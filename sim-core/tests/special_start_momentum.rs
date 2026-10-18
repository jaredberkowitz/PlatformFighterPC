//! Neutral and down specials (and the up and side specials again) started on the frames right after a jump (sim 143, owner decision
//! 102: "characters gain too much height by inputting jump, and then using specials right after"). Two rules, both in `fighter.rs`
//! `start_special`: a special that sets its own vertical rule, started in a full hop's fast opening, works from the arc's speed (the
//! opening is the jump's own set height, the reference's "initial height", not a speed to carry); and the moves whose reference
//! publishes a vertical rule use it. The per-move table and sources: `docs/movesets/SHARED_MECHANICS.md` ("Starting momentum of
//! specials").

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{JUMP, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

const LONGSWORD: u8 = 0;
const GREATSWORD: u8 = 6;
const CLUTCH: u8 = 8;

fn sim_for(c: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// Reference units as world units.
fn su(n: i32) -> Fx {
    Fx::from_ratio(n, 8)
}

/// The highest the fighter gets over `frames` frames of `input`, stopping at a KO.
fn peak(sim: &mut Sim, frames: u32, input: impl Fn(u32) -> sim_core::input::Input) -> Fx {
    let mut top = sim.f().pos.y;
    let stocks = sim.f().stocks;
    for k in 1..=frames {
        sim.tick(input(k));
        if sim.f().state == S::Respawn || sim.f().stocks != stocks {
            break;
        }
        top = top.max(sim.f().pos.y);
    }
    top
}

/// A full hop from the stage centre (and, with `double`, a midair jump 4 frames later), facing right.
fn jumped(c: u8, double: bool) -> Sim {
    let mut sim = sim_for(c);
    sim.stand(0, Fx::ZERO, 1);
    let mut t = 0;
    while sim.f().state != S::Airborne && t < 30 {
        sim.tick(inp(0, 0, JUMP));
        t += 1;
    }
    if double {
        sim.ticks(3, inp(0, 0, 0));
        sim.tick(inp(0, 0, JUMP));
    }
    sim
}

/// Every built-in special started 1 to 4 frames after a ground jump or a midair jump (a tap and a 40-frame hold): it gets no higher
/// than the higher of the jump alone and the same special from a standstill at that height. The one exception is the clutch's shell,
/// whose reference keeps 0.8 of the vertical speed [R] (a few units over).
#[test]
fn no_special_rises_over_the_jump_it_was_started_from() {
    for c in 0u8..14 {
        for double in [false, true] {
            let jump = peak(&mut jumped(c, double), 150, |_| inp(0, 0, JUMP));
            for stick in [(0i8, 0i8), (0, -127), (0, 127), (127, 0)] {
                for hold in [1u32, 40] {
                    for after in [1u32, 2, 4] {
                        let mut sim = jumped(c, double);
                        sim.ticks(after as usize - 1, inp(0, 0, JUMP));
                        let y = sim.f().pos.y;
                        let press =
                            |k: u32| inp(stick.0, stick.1, if k <= hold { SPECIAL } else { 0 });
                        let from_jump = peak(&mut sim, 150, press);
                        let mut still = sim_for(c);
                        still.put_airborne(0, Fx::ZERO, y, Fx::ZERO, Fx::ZERO);
                        still.state.fighters[0].facing = 1;
                        let from_still = peak(&mut still, 150, press);
                        let slack = if c == CLUTCH && stick == (0, 127) {
                            su(4)
                        } else {
                            Fx::from_raw(64)
                        };
                        assert!(
                            from_jump <= jump.max(from_still) + slack,
                            "class {c} stick {stick:?} hold {hold}, {after} frames after a {} jump: {:?} over",
                            if double { "midair" } else { "ground" },
                            from_jump - jump.max(from_still)
                        );
                    }
                }
            }
        }
    }
}

/// The greatsword's aerial beam keeps 0.65 of the vertical speed and falls at 0.06 [R]; out of a full hop's opening it takes that
/// share of the arc's speed, not of the opening's (before sim 143 it rose 36 to 50 reference units over the jump).
#[test]
fn the_beam_out_of_a_full_hops_opening_keeps_065_of_the_arcs_speed() {
    let p = sim_for(GREATSWORD).content.fighters[usize::from(GREATSWORD)];
    let mut sim = jumped(GREATSWORD, false);
    assert!(sim.f().hop_boost > 0);
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().hop_boost, 0);
    let want = p.full_hop_velocity * fx(65, 100);
    assert!(
        (sim.f().vel.y - want).abs() <= Fx::from_raw(64),
        "{:?} vs {want:?}",
        sim.f().vel.y
    );
}

/// The duelist's counter in the air (ssbwiki.com 13.0.1, the parameters): half the sideways speed, a light pull of 0.04 and a top
/// fall of 1.2 [R]; a jump's rise is stopped on starting [E].
#[test]
fn the_duelists_aerial_counter_stops_a_rise_and_falls_slowly() {
    let mut sim = sim_for(LONGSWORD);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), fx(1, 10), su(2));
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
    // Started: no rise; half the sideways speed. A frame on: the pull 0.04 once.
    assert_eq!(sim.f().vel.y, Fx::ZERO);
    assert!(
        (sim.f().vel.x - fx(1, 20)).abs() <= Fx::from_raw(64),
        "{:?}",
        sim.f().vel.x
    );
    sim.tick(inp(0, 0, 0));
    assert!(
        (sim.f().vel.y + su(40) / Fx::from_int(1000)).abs() <= Fx::from_raw(2),
        "{:?}",
        sim.f().vel.y
    );
    sim.ticks(50, inp(0, 0, 0));
    assert_eq!(sim.f().vel.y, -su(1200) / Fx::from_int(1000));
}

/// The duelist's charged thrust in the air: half the sideways speed on startup [R]; the lunge no longer carries the fighter off the
/// stage (before sim 143 its 3.6 units a frame went on after it: a thrust from the stage's middle fell off the side) [E].
#[test]
fn the_duelists_aerial_thrust_stays_near_where_it_was_used() {
    let mut sim = sim_for(LONGSWORD);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    let x0 = sim.f().pos.x;
    for k in 1..=60u32 {
        sim.tick(inp(0, 0, if k <= 40 { SPECIAL } else { 0 }));
    }
    let travelled = sim.f().pos.x - x0;
    assert!(
        travelled > Fx::ZERO && travelled < Fx::from_int(8),
        "{travelled:?}"
    );
}

/// A neutral special pressed during the jump squat (after the jump's own press) does not cancel it: it comes out in the air on the
/// first airborne frame, as the reference's (only the up special and the up smash cancel a jump squat, ssbwiki.com `Jump_canceling`).
#[test]
fn a_neutral_special_pressed_in_the_jump_squat_comes_out_in_the_air() {
    for c in 0u8..14 {
        let mut sim = sim_for(c);
        sim.stand(0, Fx::ZERO, 1);
        sim.tick(inp(0, 0, JUMP));
        assert_eq!(sim.f().state, S::JumpSquat, "class {c}");
        sim.tick(inp(0, 0, JUMP | SPECIAL));
        // (The first frame it is attacking: in the air. Some aerial versions set a small rise and land soon after: the gauntlets'
        // punch sets 0.2 up [R].)
        let mut first = None;
        for _ in 0..8 {
            if first.is_none() && sim.f().state == S::Attack {
                first = Some(sim.f().grounded());
            }
            sim.tick(inp(0, 0, JUMP));
        }
        assert_eq!(
            first,
            Some(false),
            "class {c}: the special came out, in the air"
        );
    }
}

/// The tumbler's cape and the duelist's blade flurry add their lift to a fall, not to a jump's rise (sim 143 [E: the reading of the
/// reference's "vertical speed added"]): started rising, the cape's first use leaves exactly its 0.3 up, the flurry's first its 1.4,
/// and a later cape stops the rise.
#[test]
fn the_lifts_do_not_keep_a_jumps_rise() {
    for (c, lift) in [(3u8, 300), (LONGSWORD, 1400)] {
        let mut sim = sim_for(c);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, su(2));
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(127, 0, SPECIAL));
        sim.tick(inp(0, 0, 0));
        let p = sim.content.fighters[usize::from(c)];
        let want = su(lift) / Fx::from_int(1000);
        // (Within a frame of the move's or the body's pull.)
        assert!(
            sim.f().vel.y <= want && sim.f().vel.y >= want - p.gravity - Fx::from_raw(4),
            "class {c}: {:?} vs {want:?}",
            sim.f().vel.y
        );
    }
    // A second cape in the same air time, started rising: the rise stops.
    let mut sim = sim_for(3);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, su(2));
    sim.state.fighters[0].air_flags = 1;
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(127, 0, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert!(sim.f().vel.y <= Fx::ZERO, "{:?}", sim.f().vel.y);
}
