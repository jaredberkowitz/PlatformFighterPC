//! Dodge staling (sim 112, the reference's; owner decision 2026-10-14): every roll or spot dodge out of a shield and every air dodge
//! makes the next ones staler, up to five levels. A stale spot dodge or forward roll plays 6% slower per level (a back roll 10%), and
//! fully stale its intangibility starts 3 (spot dodge) or 4 (roll) frames later; a stale directional air dodge travels 1/15 less per
//! level. A level wears off a second after the last dodge ended, then one every two seconds. Get-up and tech rolls are not affected.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{JUMP, SHIELD};
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// Raises the shield and spot dodges; returns how many frames the spot dodge lasted and the frame (1-based) its intangibility began.
fn spot_dodge(sim: &mut Sim) -> (u16, Option<u16>) {
    sim.ticks(2, inp(0, 0, SHIELD));
    sim.tick(inp(0, -127, SHIELD));
    assert_eq!(sim.f().state, S::SpotDodge);
    let mut frames = 1;
    let mut intangible = if sim.f().invuln > 0 { Some(1) } else { None };
    while sim.f().state == S::SpotDodge {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::SpotDodge {
            break;
        }
        frames += 1;
        if intangible.is_none() && sim.f().invuln > 0 {
            intangible = Some(frames);
        }
    }
    (frames, intangible)
}

#[test]
fn spot_dodges_in_a_row_get_longer_and_intangible_later_up_to_five_levels() {
    let mut sim = Sim::alone();
    let rules = sim.content.rules;
    let fresh = sim.content.fighters[0].spot_dodge_frames;
    let (first, first_on) = spot_dodge(&mut sim);
    let mut last = (first, first_on);
    for _ in 0..6 {
        last = spot_dodge(&mut sim);
    }
    assert_eq!(sim.f().dodge_stale, rules.dodge_stale_max);
    // Fully stale: 30% longer and intangible 3 frames later.
    let stale_len = (u16::from(fresh) * 130).div_ceil(100);
    assert!(
        last.0 >= stale_len - 1 && last.0 <= stale_len + 1,
        "{last:?} vs {first}"
    );
    assert!(last.0 > first);
    assert_eq!(last.1.unwrap(), first_on.unwrap() + 3);
}

#[test]
fn staleness_wears_off_a_second_after_the_last_dodge_then_every_two_seconds() {
    let mut sim = Sim::alone();
    for _ in 0..5 {
        spot_dodge(&mut sim);
    }
    assert_eq!(sim.f().dodge_stale, 5);
    sim.ticks(58, inp(0, 0, 0));
    assert_eq!(sim.f().dodge_stale, 5);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.f().dodge_stale, 4, "one level after a second");
    sim.ticks(120, inp(0, 0, 0));
    assert_eq!(sim.f().dodge_stale, 3, "then one every two seconds");
    sim.ticks(400, inp(0, 0, 0));
    assert_eq!(sim.f().dodge_stale, 0, "fresh again");
}

/// How far a directional air dodge to the right carries the fighter, made at staleness `level`.
fn air_dodge_distance(level: u8) -> Fx {
    let mut sim = Sim::alone();
    sim.state.fighters[0].dodge_stale = level;
    sim.tick(inp(0, 0, JUMP));
    sim.ticks(8, inp(0, 0, JUMP));
    assert!(!sim.f().grounded());
    let x0 = sim.f().pos.x;
    sim.tick(inp(127, 0, SHIELD));
    assert_eq!(sim.f().state, S::AirDodge);
    for _ in 0..25 {
        sim.tick(inp(0, 0, 0));
    }
    sim.f().pos.x - x0
}

#[test]
fn a_stale_directional_air_dodge_travels_less() {
    let fresh = air_dodge_distance(0);
    let stale = air_dodge_distance(5);
    assert!(stale < fresh, "{stale:?} vs {fresh:?}");
    // Two thirds of the dodge's speed fully stale (gravity and the slingshot aside, roughly two thirds of the distance).
    let ratio = stale / fresh;
    assert!(
        ratio > Fx::from_ratio(55, 100) && ratio < Fx::from_ratio(80, 100),
        "{ratio:?}"
    );
}

#[test]
fn a_get_up_roll_is_neither_staled_nor_staling() {
    let mut sim = Sim::alone();
    sim.state.fighters[0].dodge_stale = 3;
    sim.state.fighters[0].state = S::Knockdown;
    sim.state.fighters[0].state_frame = 60;
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Roll);
    assert_eq!(sim.f().dodge_level, 0);
    assert_eq!(sim.f().dodge_stale, 3);
}

#[test]
fn every_staling_field_is_hashed() {
    let base = Sim::alone();
    for set in [
        |s: &mut Sim| s.state.fighters[0].dodge_stale = 2,
        |s: &mut Sim| s.state.fighters[0].dodge_level = 2,
        |s: &mut Sim| s.state.fighters[0].dodge_clock = 7,
    ] {
        let mut other = Sim::alone();
        set(&mut other);
        assert_ne!(base.state.checksum(), other.state.checksum());
    }
}
