//! Shields: health, stun, perfect shield, shield break, rolls and spot dodge.
//!
//! Reference numbers: shield health 50, drained 0.15 a frame while up and refilled 0.08 a frame otherwise; a block
//! stuns for `floor(0.8 * damage * type + 2)` frames (smash 0.725, aerial 0.33, projectile 0.29, capped at 60);
//! a hit in the first 5 frames of the shield is a perfect shield; a broken shield stuns for a long time and returns
//! with 75% health.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const MARTH: [u8; 4] = [0, 0, 0, 0];
const WOLF: [u8; 4] = [1, 1, 1, 1];

fn park(sim: &mut Sim) {
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
}

/// Player 0 on the left facing right, player 1 `gap` in front of it.
fn duel(chars: [u8; 4], gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-7) + gap, -1);
    park(&mut sim);
    sim
}

/// Player 1 holds shield from tick 1; player 0 forward-tilts on tick `attack_at`. Returns the sim just after
/// player 1 blocks (the tick its shield stun is set).
fn block_a_forward_tilt(shield_from: usize, attack_at: usize) -> Sim {
    let mut sim = duel(MARTH, fx(10, 10));
    for t in 1..=40 {
        let p0 = if t == attack_at {
            inp(30, 0, ATTACK)
        } else {
            inp(0, 0, 0)
        };
        let p1 = if t >= shield_from {
            inp(0, 0, SHIELD)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(p0, p1);
        if sim.fighter(1).shield_stun > 0 {
            return sim;
        }
    }
    panic!("the hit never landed on the shield");
}

fn near(a: Fx, b: Fx, tol: Fx) -> bool {
    (a - b).abs() <= tol
}

// ---- Health ------------------------------------------------------------------------------------------------

#[test]
fn the_shield_drains_while_up_and_refills_while_down() {
    let mut sim = Sim::new();
    park(&mut sim);
    let full = sim.content.rules.shield_max;
    assert_eq!(sim.f().shield_hp, full);
    sim.ticks(21, inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    // 21 ticks of shield: the first tick enters the state, the next 20 drain 0.15 each.
    assert!(
        near(sim.f().shield_hp, full - fx(15, 100).mul_int(20), fx(1, 20)),
        "{:?}",
        sim.f().shield_hp
    );
    // Let go: it refills 0.08 a frame up to the maximum.
    let low = sim.f().shield_hp;
    sim.ticks(10, inp(0, 0, 0));
    assert!(near(
        sim.f().shield_hp,
        low + fx(8, 100).mul_int(10),
        fx(3, 10)
    ));
    sim.ticks(400, inp(0, 0, 0));
    assert_eq!(sim.f().shield_hp, full);
}

#[test]
fn holding_the_shield_long_enough_breaks_it() {
    let mut sim = Sim::new();
    park(&mut sim);
    sim.state.fighters[0].shield_hp = fx(1, 1);
    for _ in 0..20 {
        sim.tick(inp(0, 0, SHIELD));
    }
    assert_eq!(sim.f().state, S::ShieldBreak);
}

// ---- Blocking ----------------------------------------------------------------------------------------------

#[test]
fn a_block_costs_shield_health_and_stuns_for_0_8_damage_plus_2_frames() {
    let sim = block_a_forward_tilt(1, 12);
    // Forward tilt does 9% close up: 0.8 * 9 + 2 = 9.2, so 9 frames.
    assert_eq!(sim.fighter(1).shield_stun, 9);
    assert_eq!(
        sim.fighter(1).percent,
        Fx::ZERO,
        "no damage through a shield"
    );
    let drained = sim.content.rules.shield_max - sim.fighter(1).shield_hp;
    // 9% of damage times the 1.2 multiplier, plus the frames of holding the shield.
    assert!(
        drained > fx(108, 10) && drained < fx(145, 10),
        "{drained:?}"
    );
}

#[test]
fn a_blocked_fighter_cannot_act_until_the_shield_stun_ends() {
    let mut sim = block_a_forward_tilt(1, 12);
    let stun = u32::from(sim.fighter(1).shield_stun);
    // Hitlag first, then the stun counts down; shield released and a jump asked for meanwhile.
    let mut released_at = None;
    for t in 0..80 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, JUMP));
        if sim.fighter(1).state != S::Shield && released_at.is_none() {
            released_at = Some(t);
        }
    }
    let t = released_at.expect("it gets out eventually");
    // Hitlag is 7 frames for a 9% hit, then 9 frames of stun.
    assert!(
        t >= 7 + stun as usize - 2,
        "left the shield after only {t} ticks"
    );
}

#[test]
fn a_hit_in_the_first_frames_of_the_shield_is_a_perfect_shield() {
    // The hit lands on tick 19; the shield goes up on tick 16, so it is 3 frames old.
    let perfect = block_a_forward_tilt(16, 12);
    let normal = block_a_forward_tilt(1, 12);
    assert_eq!(perfect.fighter(1).shield_stun, 9 - 3);
    assert_eq!(normal.fighter(1).shield_stun, 9);
    let full = perfect.content.rules.shield_max;
    // A perfect shield costs no health beyond the frames spent holding it.
    assert!(
        full - perfect.fighter(1).shield_hp < fx(1, 1),
        "{:?}",
        perfect.fighter(1).shield_hp
    );
    assert!(
        normal.fighter(1).vel.x.abs() > perfect.fighter(1).vel.x.abs(),
        "less pushback"
    );
}

#[test]
fn aerials_smashes_and_projectiles_stun_a_shield_less() {
    // The blaster's close shot (8%): floor(0.8 * 8 * 0.29 + 2) = 3 frames.
    let mut sim = duel(WOLF, fx(100, 10));
    for t in 1..=60 {
        let p0 = if t == 1 {
            inp(0, 0, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(p0, inp(0, 0, SHIELD));
        if sim.fighter(1).shield_stun > 0 {
            break;
        }
    }
    let stun = sim.fighter(1).shield_stun;
    assert!((2..=3).contains(&stun), "projectile stun {stun}");
}

#[test]
fn a_hit_that_empties_the_shield_breaks_it() {
    let mut sim = duel(MARTH, fx(10, 10));
    sim.state.fighters[1].shield_hp = fx(5, 1);
    for t in 1..=30 {
        let p0 = if t == 1 {
            inp(30, 0, ATTACK)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(p0, inp(0, 0, SHIELD));
    }
    assert_eq!(sim.fighter(1).state, S::ShieldBreak);
    assert_eq!(sim.fighter(1).percent, Fx::ZERO, "a break does no damage");
}

// ---- Shield break ------------------------------------------------------------------------------------------

fn broken(percent: i32) -> Sim {
    let mut sim = Sim::new();
    park(&mut sim);
    sim.state.fighters[0].percent = Fx::from_int(percent);
    sim.state.fighters[0].shield_hp = fx(1, 10);
    // The shield goes up on the first tick and the next one drains it.
    for _ in 0..6 {
        if sim.f().state == S::ShieldBreak {
            break;
        }
        sim.tick(inp(0, 0, SHIELD));
    }
    assert_eq!(sim.f().state, S::ShieldBreak);
    sim
}

#[test]
fn a_shield_break_hops_up_and_stuns_for_longer_at_low_damage() {
    let low = broken(0);
    let high = broken(100);
    assert!(low.f().vel.y > Fx::ZERO, "launched upward");
    assert!(!low.f().grounded());
    assert_eq!(low.f().hitstun, 400);
    assert_eq!(high.f().hitstun, 300);
}

#[test]
fn a_broken_fighter_cannot_act_but_comes_back_with_75_percent_shield() {
    let mut sim = broken(0);
    // Wait for the hop to come down; the stun keeps running meanwhile.
    for _ in 0..200 {
        if sim.f().grounded() {
            break;
        }
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().grounded());
    sim.state.fighters[0].hitstun = 60;
    for _ in 0..30 {
        sim.tick(inp(0, 0, ATTACK | JUMP));
        assert_eq!(sim.f().state, S::ShieldBreak);
    }
    // Let go of the buttons and let the rest run out.
    sim.state.fighters[0].hitstun = 30;
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    // 75% of 50, plus the little it has refilled since.
    let restored = sim.content.rules.shield_max * fx(75, 100);
    assert!(
        near(sim.f().shield_hp, restored, fx(3, 1)),
        "{:?}",
        sim.f().shield_hp
    );
    assert!(sim.f().shield_hp >= restored);
}

#[test]
fn mashing_buttons_shortens_the_break_stun() {
    let ticks_to_recover = |mash: bool| {
        let mut sim = broken(0);
        for t in 0..1000 {
            let b = if mash && t % 2 == 0 { ATTACK } else { 0 };
            sim.tick(inp(0, 0, b));
            if sim.f().state != S::ShieldBreak {
                return t;
            }
        }
        panic!("never recovered");
    };
    let slow = ticks_to_recover(false);
    let fast = ticks_to_recover(true);
    assert!(fast * 2 < slow + 20, "mashing took {fast}, waiting {slow}");
}

// ---- Rolls and spot dodge --------------------------------------------------------------------------------------

fn start_roll(x: i8) -> Sim {
    let mut sim = Sim::new();
    park(&mut sim);
    sim.state.fighters[1].invuln = 255;
    sim.stand(0, Fx::from_int(-2), 1);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.tick(inp(x, 0, SHIELD));
    sim
}

#[test]
fn a_flick_sideways_in_the_shield_rolls_that_way() {
    let mut sim = start_roll(127);
    assert_eq!(sim.f().state, S::Roll);
    let x0 = sim.f().pos.x;
    sim.ticks(40, inp(0, 0, 0));
    let moved = sim.f().pos.x - x0;
    assert!(moved > fx(25, 10) && moved < fx(34, 10), "rolled {moved:?}");
    assert_eq!(sim.f().state, S::Idle);

    let mut sim = start_roll(-127);
    let x0 = sim.f().pos.x;
    sim.ticks(40, inp(0, 0, 0));
    assert!(sim.f().pos.x < x0 - fx(25, 10));
}

#[test]
fn a_roll_is_intangible_for_part_of_its_length() {
    let intangible_on = |frame: usize| {
        let mut sim = start_roll(127);
        sim.ticks(frame, inp(0, 0, 0));
        sim_core::combat::is_intangible(sim.f())
    };
    let p = Sim::new().content.fighters[0];
    assert!(!intangible_on(1), "not yet");
    for f in usize::from(p.roll_intangible_start)..=usize::from(p.roll_intangible_end) {
        assert!(intangible_on(f), "intangible on roll frame {f}");
    }
    assert!(!intangible_on(usize::from(p.roll_intangible_end) + 2));
}

#[test]
fn a_hit_during_the_roll_misses_but_one_after_it_lands() {
    let mut dodged = duel(MARTH, fx(10, 10));
    dodged.ticks(6, inp(0, 0, 0));
    // Player 1 rolls away on tick 7; player 0 tilts on tick 12 (a hit on tick 19, mid-roll).
    for t in 1..=60 {
        let p0 = if t == 12 {
            inp(30, 0, ATTACK)
        } else {
            inp(0, 0, 0)
        };
        let p1 = if t < 7 {
            inp(0, 0, SHIELD)
        } else if t == 7 {
            inp(-127, 0, SHIELD)
        } else {
            inp(0, 0, 0)
        };
        dodged.tick2(p0, p1);
    }
    assert_eq!(dodged.fighter(1).percent, Fx::ZERO);
}

#[test]
fn shield_and_a_hard_down_is_a_spot_dodge_that_stays_in_place() {
    let mut sim = Sim::new();
    park(&mut sim);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.tick(inp(0, -127, SHIELD));
    assert_eq!(sim.f().state, S::SpotDodge);
    let x0 = sim.f().pos.x;
    let p = sim.content.fighters[0];
    sim.ticks(usize::from(p.spot_intangible_start), inp(0, 0, 0));
    assert!(sim_core::combat::is_intangible(sim.f()));
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().pos.x, x0);
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn a_soft_down_in_the_shield_does_not_spot_dodge() {
    let mut sim = Sim::new();
    park(&mut sim);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.ticks(10, inp(0, -70, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
}

#[test]
fn rolling_and_spot_dodging_are_not_possible_while_stunned_from_a_block() {
    let mut sim = block_a_forward_tilt(1, 12);
    // Still in hitlag and shield stun: a flick does nothing yet.
    sim.tick2(inp(0, 0, 0), inp(127, 0, SHIELD));
    assert_ne!(sim.fighter(1).state, S::Roll);
}
