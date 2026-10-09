//! Movement rules taken from the reference game: the 9-frame input buffer and the hold buffer, one press per action, the
//! short-hop aerial (and its damage), midair jumps that steer, light and heavy landings, and backward rolls.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::state::{FighterState as S, NONE};
use sim_core::Fx;

/// Raises and drops the shield, leaving the fighter at the start of the 11-frame shield release.
fn into_shield_release(sim: &mut Sim) {
    sim.ticks(5, inp(0, 0, SHIELD));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::ShieldRelease);
}

/// Ticks with no input until the fighter can act again (leaves the shield release); returns how many frames that took.
fn until_free(sim: &mut Sim) -> usize {
    let mut n = 0;
    while sim.f().state == S::ShieldRelease {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    n
}

// ---- Input buffer ---------------------------------------------------------------------------------

#[test]
fn a_press_up_to_nine_frames_early_is_buffered_and_one_ten_frames_early_is_not() {
    for (early, jumps) in [(9usize, true), (10, false)] {
        let mut sim = Sim::new();
        into_shield_release(&mut sim);
        let release = usize::from(sim.content.fighters[0].shield_release_frames);
        // Press jump `early` frames before the fighter can act, then let go.
        sim.ticks(release - early, inp(0, 0, 0));
        sim.tick(inp(0, 0, JUMP));
        until_free(&mut sim);
        sim.tick(inp(0, 0, 0));
        assert_eq!(
            sim.f().state == S::JumpSquat,
            jumps,
            "a press {early} frames early"
        );
    }
}

#[test]
fn holding_a_button_buffers_it_however_early_it_was_pressed() {
    let mut sim = Sim::new();
    into_shield_release(&mut sim);
    // Pressed on the first frame of the release (11 frames early) and held: the hold buffer.
    while sim.f().state == S::ShieldRelease {
        sim.tick(inp(0, 0, JUMP));
    }
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
}

#[test]
fn one_press_starts_one_action() {
    let mut sim = Sim::new();
    into_shield_release(&mut sim);
    sim.ticks(6, inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    until_free(&mut sim);
    let mut attacks = 0;
    let mut was_attacking = false;
    for _ in 0..120 {
        sim.tick(inp(0, 0, 0));
        let attacking = sim.f().state == S::Attack;
        if attacking && !was_attacking {
            attacks += 1;
        }
        was_attacking = attacking;
    }
    assert_eq!(
        attacks, 1,
        "the buffered press gives one jab, not one per frame of the buffer"
    );
}

#[test]
fn a_held_jump_used_for_the_jump_does_not_also_double_jump() {
    let mut sim = Sim::new();
    sim.ticks(40, inp(0, 0, JUMP));
    assert_eq!(
        sim.f().air_jumps_left,
        1,
        "holding jump through a full hop keeps the midair jump"
    );
}

// ---- Short hops -----------------------------------------------------------------------------------

/// Peak height of a ground jump with the given inputs on the first frames.
fn peak(first: &[u16]) -> Fx {
    let mut sim = Sim::new();
    let mut top = Fx::ZERO;
    for t in 0..80 {
        let b = first.get(t).copied().unwrap_or(0);
        sim.tick(inp(0, 0, b));
        top = top.max(sim.f().pos.y);
    }
    top
}

#[test]
fn jump_and_attack_together_is_a_short_hop_aerial_even_with_jump_held() {
    let full = peak(&[JUMP; 10]);
    let short = peak(&[JUMP]);
    let macro_hop = peak(&[JUMP | ATTACK, JUMP, JUMP, JUMP, JUMP, JUMP]);
    assert!(full > short * fx(3, 2));
    assert!(
        macro_hop < short + fx(1, 10),
        "jump and attack together is a short hop: {macro_hop:?} vs {short:?}"
    );
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP | ATTACK));
    for _ in 0..6 {
        sim.tick(inp(0, 0, JUMP));
    }
    assert_eq!(sim.f().state, S::Attack, "the aerial comes out right away");
    assert!(sim.f().short_hop);
}

/// Damage a neutral aerial deals to a fighter right next to it, from a short hop or not.
fn aerial_damage(short_hop: bool) -> Fx {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, fx(1, 2), Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].short_hop = short_hop;
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            break;
        }
    }
    sim.fighter(1).percent
}

#[test]
fn a_short_hop_aerial_deals_85_percent_damage() {
    let full = aerial_damage(false);
    let short = aerial_damage(true);
    assert!(full > Fx::ZERO, "the aerial connects");
    let ratio = short / full;
    assert!(
        (ratio - fx(85, 100)).abs() < fx(1, 100),
        "short hop / full: {ratio:?}"
    );
}

#[test]
fn landing_ends_the_short_hop() {
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP));
    sim.ticks(5, inp(0, 0, 0));
    assert!(sim.f().short_hop);
    sim.ticks(80, inp(0, 0, 0));
    assert_ne!(sim.f().platform, NONE);
    assert!(!sim.f().short_hop);
}

// ---- Midair jumps ---------------------------------------------------------------------------------

#[test]
fn a_midair_jump_takes_its_drift_from_the_stick() {
    let jump_with = |x: i8| {
        let mut sim = Sim::new();
        let air_speed = sim.content.fighters[0].air_speed;
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), air_speed, Fx::ZERO);
        sim.tick(inp(x, 0, JUMP));
        sim.f().vel.x
    };
    assert!(jump_with(-127) < Fx::ZERO, "it reverses the drift at once");
    assert_eq!(
        jump_with(0),
        Fx::ZERO,
        "with the stick neutral it goes straight up"
    );
    assert!(jump_with(127) > Fx::ZERO);
}

// ---- Landings -------------------------------------------------------------------------------------

#[test]
fn a_landing_at_full_fall_speed_is_heavy_and_a_short_hop_landing_is_light() {
    let p = Sim::new().content.fighters[0];
    assert_eq!((p.landing_lag, p.heavy_landing_lag), (2, 4));

    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP));
    let mut lag = None;
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            lag = Some(sim.f().lag);
            break;
        }
    }
    assert_eq!(lag, Some(p.landing_lag), "a short hop lands lightly");

    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    let mut lag = None;
    for _ in 0..300 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            lag = Some(sim.f().lag);
            break;
        }
    }
    assert_eq!(lag, Some(p.heavy_landing_lag), "a long fall lands heavily");
}

// ---- Rolls ----------------------------------------------------------------------------------------

#[test]
fn a_backward_roll_takes_longer_than_a_forward_one() {
    let roll = |x: i8| {
        let mut sim = Sim::new();
        sim.stand(0, Fx::ZERO, 1);
        sim.ticks(5, inp(0, 0, SHIELD));
        sim.tick(inp(x, 0, SHIELD));
        assert_eq!(sim.f().state, S::Roll);
        let mut n = 0;
        while sim.f().state == S::Roll {
            sim.tick(inp(0, 0, 0));
            n += 1;
        }
        n
    };
    let p = Sim::new().content.fighters[0];
    assert_eq!(roll(127), usize::from(p.roll_frames));
    assert_eq!(roll(-127), usize::from(p.roll_back_frames));
}
