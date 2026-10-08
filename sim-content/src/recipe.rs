//! Character recipes: a handful of stats chosen in the character creator, turned into a full set of fighter parameters.
//!
//! A recipe is a class (which moveset and body archetype it starts from) plus four stats from 1 to 9, where 5 is the
//! archetype as it ships:
//!
//! * **Size**: the body. Bigger means a taller, wider hurtbox, more weight, a slower run, a lower jump and a faster fall;
//!   smaller is the opposite. Size 5 changes nothing.
//! * **Speed**: how fast and how responsive the fighter is on the ground and in the air (walk, run, dash, accelerations).
//!   A little less weight comes with it.
//! * **Jump**: how high the jumps go.
//! * **Weight**: how hard the fighter is to launch. Heavier fighters also fall a little faster.
//!
//! The same recipe always gives the same parameters (integer arithmetic only), so two machines that share a recipe share a
//! fighter. Everything it produces passes [`crate::validate`]; a test checks every recipe there is.

use crate::format::fighter_block;
use crate::tree::Block;
use sim_core::{FighterParams, Fx};

pub const MIN_STAT: u8 = 1;
pub const MAX_STAT: u8 = 9;
/// The stat value that leaves the archetype unchanged.
pub const NEUTRAL: u8 = 5;
/// How many classes there are (index into the roster's movesets: 0 longsword, 1 claws).
pub const CLASSES: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub class: u8,
    pub size: u8,
    pub speed: u8,
    pub jump: u8,
    pub weight: u8,
}

impl Default for Recipe {
    fn default() -> Recipe {
        Recipe {
            class: 0,
            size: NEUTRAL,
            speed: NEUTRAL,
            jump: NEUTRAL,
            weight: NEUTRAL,
        }
    }
}

/// `v` times `percent` / 100.
fn scale(v: Fx, percent: i32) -> Fx {
    v * Fx::from_ratio(percent, 100)
}

impl Recipe {
    /// The recipe with every value brought into range.
    pub fn clamped(self) -> Recipe {
        let c = |s: u8| s.clamp(MIN_STAT, MAX_STAT);
        Recipe {
            class: self.class.min(CLASSES - 1),
            size: c(self.size),
            speed: c(self.speed),
            jump: c(self.jump),
            weight: c(self.weight),
        }
    }

    /// Body size as a percent of the archetype's: 68 at size 1, 100 at size 5, 132 at size 9.
    pub fn size_percent(self) -> i32 {
        60 + 8 * i32::from(self.clamped().size)
    }

    /// The archetype's parameters changed by the stats.
    pub fn params(self) -> FighterParams {
        let r = self.clamped();
        let mut p = if r.class == 0 {
            FighterParams::duelist()
        } else {
            FighterParams::brawler()
        };
        let size = r.size_percent();
        let speed = i32::from(r.speed);
        let jump = i32::from(r.jump);
        let weight = i32::from(r.weight);
        let neutral = i32::from(NEUTRAL);

        // Body: the collision box and where a ledge is held scale with size.
        for v in [
            &mut p.ecb_half_width,
            &mut p.ecb_height,
            &mut p.ecb_side_height,
            &mut p.ledge_reach_x,
            &mut p.ledge_min_drop,
            &mut p.ledge_reach_down,
            &mut p.ledge_hang_dx,
            &mut p.ledge_hang_dy,
        ] {
            *v = scale(*v, size);
        }

        // Ground and air speed: the speed stat, and bigger bodies are slower.
        let bulk_slow = 100 - (size - 100) * 35 / 100;
        let pace = (80 + 4 * speed) * bulk_slow / 100;
        for v in [
            &mut p.walk_speed,
            &mut p.run_speed,
            &mut p.dash_speed,
            &mut p.dash_initial_speed,
            &mut p.air_speed,
            &mut p.air_dodge_speed,
            &mut p.waveland_speed,
        ] {
            *v = scale(*v, pace);
        }
        // Responsiveness follows the speed stat alone.
        let snap = 80 + 4 * speed;
        for v in [
            &mut p.dash_accel,
            &mut p.dash_brake,
            &mut p.ground_accel,
            &mut p.ground_friction,
            &mut p.run_decel,
            &mut p.landing_friction,
            &mut p.air_accel,
            &mut p.air_accel_stick,
        ] {
            *v = scale(*v, snap);
        }

        // Jumps: the jump stat raises them, a big body lowers them.
        let lift = (100 + (jump - neutral) * 3) * (100 - (size - 100) * 20 / 100) / 100;
        for v in [
            &mut p.full_hop_velocity,
            &mut p.hop_burst_velocity,
            &mut p.short_hop_velocity,
            &mut p.air_jump_velocity,
        ] {
            *v = scale(*v, lift);
        }

        // Falling: big and heavy bodies drop faster.
        let pull = 100 + (size - 100) * 20 / 100 + (weight - neutral) * 2;
        p.gravity = scale(p.gravity, pull);
        let fall = 100 + (size - 100) * 15 / 100 + (weight - neutral) * 3;
        p.max_fall_speed = scale(p.max_fall_speed, fall);
        p.fast_fall_speed = scale(p.fast_fall_speed, fall);

        // Weight: size and the weight stat add to it, the speed stat takes a little off.
        let mass = (100 + (size - 100) * 120 / 100) * (100 + (weight - neutral) * 8) / 100
            * (100 - (speed - neutral) * 3)
            / 100;
        p.weight = scale(p.weight, mass);
        p
    }

    /// The fighter section for this recipe: every parameter written out, so it does not depend on any other fighter.
    pub fn section(self, name: &str, weapon_name: &str) -> Block {
        fighter_block(name, &self.params(), weapon_name)
    }
}

/// What a fighter feels like, for the creator's stat bars. Units are world units and frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Readout {
    pub run_speed: Fx,
    pub jump_height: Fx,
    pub weight: Fx,
    pub fall_speed: Fx,
    pub height: Fx,
}

pub fn readout(p: &FighterParams) -> Readout {
    Readout {
        run_speed: p.run_speed,
        jump_height: crate::full_hop_height(p),
        weight: p.weight,
        fall_speed: p.max_fall_speed,
        height: p.ecb_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_neutral_recipe_is_the_archetype() {
        assert_eq!(Recipe::default().params(), FighterParams::duelist());
        let claws = Recipe {
            class: 1,
            ..Recipe::default()
        };
        assert_eq!(claws.params(), FighterParams::brawler());
    }

    #[test]
    fn out_of_range_stats_are_brought_into_range() {
        let wild = Recipe {
            class: 9,
            size: 0,
            speed: 200,
            jump: 0,
            weight: 255,
        };
        let c = wild.clamped();
        assert_eq!(
            (c.class, c.size, c.speed, c.jump, c.weight),
            (1, 1, 9, 1, 9)
        );
        assert_eq!(wild.params(), c.params());
    }
}
