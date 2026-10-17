//! Up specials and the momentum they start with (sim 130, owner decision 97): started in the air, an up special whose movement is the
//! move's own halts the vertical speed it began with, so a jump's rise is not carried into it (the owner's "super-jump": the exile's
//! blade dash started just after a jump went far higher than from a standstill). The reference's per-move rules and sources are in
//! `docs/movesets/SHARED_MECHANICS.md` ("Starting momentum of specials").

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, C_UP, GRAB, JUMP, SHIELD, SPECIAL, STRONG};
use sim_core::moves::nodachi_slots::{DASH_AIR_END, OCTA};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

const EXILE: u8 = 13;

/// Class `c` as player 0, alone (player 1 far away, the others out), its gauge's powered mode running when `powered`.
fn sim_for(c: u8, powered: bool) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    sim.stand(1, Fx::from_int(60), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    if powered {
        sim.state.fighters[0].gauge_mode = 1500;
    }
    sim
}

/// Starts the up special now (the stick straight up, special pressed and held `hold` frames) and returns how far above its start the
/// fighter rises in the next 240 frames.
fn rise(sim: &mut Sim, hold: u32) -> Fx {
    let y0 = sim.f().pos.y;
    let mut peak = y0;
    for t in 1..=240u32 {
        let b = if t <= hold { SPECIAL } else { 0 };
        sim.tick(inp(0, 127, b));
        peak = peak.max(sim.f().pos.y);
    }
    peak - y0
}

/// The fighter jumps from the stage (a full hop) and is `after` frames into the air: returns it with its height.
fn jumped(c: u8, powered: bool, after: u32) -> (Sim, Fx) {
    let mut sim = sim_for(c, powered);
    sim.stand(0, Fx::ZERO, 1);
    let mut t = 0;
    while sim.f().state != S::Airborne && t < 30 {
        sim.tick(inp(0, 0, JUMP));
        t += 1;
    }
    for _ in 1..after {
        sim.tick(inp(0, 0, JUMP));
    }
    assert!(sim.f().vel.y > Fx::ZERO, "class {c} still rising");
    let y = sim.f().pos.y;
    (sim, y)
}

/// The same fighter still in the air at height `y`, moving at `vy`.
fn in_air(c: u8, powered: bool, y: Fx, vy: Fx) -> Sim {
    let mut sim = sim_for(c, powered);
    sim.put_airborne(0, Fx::ZERO, y, Fx::ZERO, vy);
    sim.state.fighters[0].facing = 1;
    sim
}

#[test]
fn the_exiles_up_special_out_of_a_jump_rises_no_higher_than_from_a_standstill() {
    // The blade dash (a tap) and the eight slashes (special held through frame 18), started 1 to 4 frames after leaving the ground.
    for hold in [1u32, 40] {
        for after in 1..=4u32 {
            let (mut sim, y) = jumped(EXILE, false, after);
            let from_jump = rise(&mut sim, hold);
            let from_still = rise(&mut in_air(EXILE, false, y, Fx::ZERO), hold);
            assert_eq!(
                from_jump, from_still,
                "hold {hold}, {after} frames after the jump"
            );
        }
    }
    // Before sim 130 the dash out of a full hop rose 21.4 world units against 9.4 from a standstill: the bound below keeps it near.
    let (mut sim, y) = jumped(EXILE, false, 4);
    assert!(rise(&mut sim, 1) < Fx::from_int(10));
    let _ = y;
}

#[test]
fn the_exiles_up_special_out_of_a_midair_jump_halts_its_rise_too() {
    let mut sim = in_air(EXILE, false, Fx::from_int(30), Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, JUMP));
    sim.tick(inp(0, 0, 0));
    assert!(sim.f().vel.y > Fx::ZERO);
    let y = sim.f().pos.y;
    let from_jump = rise(&mut sim, 1);
    let from_still = rise(&mut in_air(EXILE, false, y, Fx::ZERO), 1);
    assert_eq!(from_jump, from_still);
}

/// Every built-in up special that halts its starting vertical speed (the reference's motion-driven or speed-setting up specials):
/// out of a jump's rise, from a standstill and out of a fall at the same height, it rises the same.
#[test]
fn every_halting_up_special_rises_the_same_from_a_jump_a_standstill_or_a_fall() {
    // (class, powered: the gauge's version — the Greatsword's powered rise, the Dagger's wings)
    let halting = [
        (0, false),
        (1, false),
        (2, false),
        (3, false),
        (4, false),
        (5, false),
        (6, false),
        (6, true),
        (7, false),
        (9, false),
        (10, false),
        (11, false),
        // The Dagger's aerial hook (sim 139, owner decision 98: it halts a jump's rise like the recoveries).
        (12, false),
        (12, true),
        (13, false),
    ];
    for (c, powered) in halting {
        for hold in [1u32, 40] {
            let (mut sim, y) = jumped(c, powered, 4);
            let from_jump = rise(&mut sim, hold);
            let from_still = rise(&mut in_air(c, powered, y, Fx::ZERO), hold);
            let from_fall = rise(&mut in_air(c, powered, y, -Fx::from_ratio(1, 4)), hold);
            assert_eq!(
                from_jump, from_still,
                "class {c} (powered {powered}, hold {hold}): out of a jump"
            );
            assert_eq!(
                from_fall, from_still,
                "class {c} (powered {powered}, hold {hold}): out of a fall"
            );
        }
    }
}

/// The Clutch's lobbed shell keeps 0.8 of its starting vertical speed by the reference's own parameter ("vertical speed multiplier on
/// startup 0.8", capped at 2.6 of its units: ssbwiki.com), so out of a jump it rises more than from a standstill, but only a little.
#[test]
fn the_lobbed_shell_keeps_its_published_share_of_a_jump() {
    let (mut sim, y) = jumped(8, false, 4);
    let from_jump = rise(&mut sim, 1);
    let from_still = rise(&mut in_air(8, false, y, Fx::ZERO), 1);
    assert!(from_jump > from_still);
    assert!(from_jump - from_still < Fx::ONE);
}

/// After its up special the exile acts again only by landing (or catching a ledge): every button in the special fall does nothing.
#[test]
fn nothing_lets_the_exile_act_after_its_up_special_but_landing() {
    let presses = [
        JUMP,
        ATTACK,
        SPECIAL,
        SHIELD,
        GRAB,
        STRONG,
        C_UP,
        JUMP | SPECIAL,
    ];
    let own = [MoveId::UpSpecial as u8, OCTA as u8, DASH_AIR_END as u8];
    for hold in [1u32, 40] {
        let mut sim = in_air(EXILE, false, Fx::from_int(12), Fx::ZERO);
        let mut helpless = false;
        let mut landed = false;
        for t in 1..=400u32 {
            let b = if t <= hold {
                SPECIAL
            } else if helpless {
                presses[(t as usize) % presses.len()]
            } else {
                0
            };
            let stick = if helpless {
                ((t % 3) as i8 - 1) * 127
            } else {
                0
            };
            sim.tick(inp(stick, 127, b));
            let f = sim.f();
            if f.platform != sim_core::state::NONE || f.state == S::LedgeHang {
                landed = true;
                break;
            }
            match f.state {
                S::Attack => assert!(
                    !helpless && own.contains(&f.move_id),
                    "hold {hold}: frame {t} attacking with move {}",
                    f.move_id
                ),
                S::Helpless => helpless = true,
                other => panic!("hold {hold}: frame {t} {other:?} in the air after the up special"),
            }
        }
        assert!(helpless && landed, "hold {hold}");
    }
}
