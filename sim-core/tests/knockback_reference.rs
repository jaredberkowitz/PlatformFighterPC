//! What the knockback formula reads at the moment of a hit (sim 97), checked against the reference's own calculation (rubendal's
//! SSBU-Calculator, `formulas.js` / `global.js`):
//! - its percent is the victim's percent before the hit plus the hit's damage *without* the one-on-one multiplier (that multiplier
//!   only scales the damage taken: it never moves the percent a move KOs at);
//! - its damage term is the move's own damage: a short hop's 0.85 is a damage-dealt multiplier and leaves it alone;
//! - the fixed fall speed of a launch close to straight up belongs to the upward tumble: a launch too weak to tumble falls with the
//!   fighter's own gravity and fall speed.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP};
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// The Embers (the tumbler, class 3) short hop up airs the Longsword (the duelist, weight 90, class 0) standing next to it at
/// `percent`. Returns the sim on the frame the hit lands.
fn short_hop_up_air(percent: i32, damage_mult: Fx) -> Sim {
    let mut sim = Sim::with_chars([3, 0, 0, 1]);
    sim.content.rules.damage_mult = damage_mult;
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-8), 1);
    sim.stand(1, Fx::from_ratio(-68, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(percent);
    // A tap of jump is a short hop; the up air the first frame in the air.
    sim.tick(inp(0, 0, JUMP));
    let mut pressed = false;
    for _ in 0..60 {
        let toward = if sim.fighter(1).pos.x > sim.f().pos.x + Fx::from_ratio(1, 10) {
            127
        } else {
            0
        };
        let input = if !pressed && sim.f().state == S::Airborne {
            pressed = true;
            inp(0, 127, ATTACK)
        } else {
            inp(toward, 0, 0)
        };
        sim.tick(input);
        if sim.fighter(1).percent > Fx::from_int(percent) {
            assert!(sim.f().short_hop, "the up air was made in a short hop");
            return sim;
        }
    }
    panic!("the short-hop up air never hit");
}

#[test]
fn a_short_hop_up_air_at_40_percent_gives_the_reference_hitstun() {
    // Reference: d = 7 (no short-hop factor), p = 40 + 7 x 0.85 = 45.95, weight 90, BK 10, KS 135: KB 75.4, hitstun
    // floor(75.4 x 0.4) - 1 = 29. (With the 0.85 in d as well, as before sim 97: KB 71.6 and 27 frames.)
    let sim = short_hop_up_air(40, Fx::ONE);
    assert_eq!(sim.fighter(1).hitstun, 29);
    // The damage taken does carry the short hop's 0.85.
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(40) + Fx::from_int(7) * Fx::from_ratio(85, 100)
    );
}

#[test]
fn the_one_on_one_multiplier_scales_the_damage_but_not_the_knockback() {
    let plain = short_hop_up_air(40, Fx::ONE);
    let duel = short_hop_up_air(40, Fx::from_ratio(12, 10));
    assert_eq!(duel.fighter(1).hitstun, plain.fighter(1).hitstun);
    assert_eq!(duel.fighter(1).launch_kb, plain.fighter(1).launch_kb);
    assert!(duel.fighter(1).percent > plain.fighter(1).percent);
}

/// The victim's steepest fall while its hitstun lasts, after a short hop up air at `percent`, and whether that launch tumbled.
fn fall_after_up_air(percent: i32) -> (Fx, bool) {
    let mut sim = short_hop_up_air(percent, Fx::ONE);
    let mut fastest = Fx::ZERO;
    let mut tumbled = false;
    let mut lifted = false;
    for _ in 0..200 {
        sim.tick(inp(0, 0, 0));
        let v = sim.fighter(1);
        tumbled |= v.tumble;
        if v.hitlag > 0 {
            continue;
        }
        lifted |= !v.grounded();
        if v.state != S::Hitstun || (lifted && v.grounded()) {
            break;
        }
        fastest = fastest.max(-v.vel.y);
    }
    assert!(lifted, "the up air lifts its victim");
    (fastest, tumbled)
}

#[test]
fn only_a_tumbling_upward_launch_falls_at_the_fixed_launch_speed() {
    let rules = sim_core::Content::placeholder().rules;
    let own_fall = Sim::with_chars([3, 0, 0, 1]).content.fighters[0].max_fall_speed;
    assert!(
        own_fall < rules.vertical_launch_fall,
        "the Longsword falls slower than the fixed launch fall"
    );
    // 40%: the up air (75 degrees, knockback 75) does not tumble, though its hitstun lasts 29 frames; the victim falls no faster
    // than its own fall speed.
    let (weak, tumbled) = fall_after_up_air(40);
    assert!(!tumbled);
    assert!(weak <= own_fall, "a non-tumbling launch fell at {weak:?}");
    // High percent: it tumbles and falls at the fixed launch fall speed.
    let (strong, tumbled) = fall_after_up_air(120);
    assert!(tumbled);
    assert_eq!(strong, rules.vertical_launch_fall);
}
