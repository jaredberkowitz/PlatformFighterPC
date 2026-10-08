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
/// How many classes there are (index into the roster's movesets: 0 longsword, 1 claws, 2 maul).
pub const CLASSES: u8 = 3;

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
        let mut p = match r.class {
            0 => FighterParams::duelist(),
            1 => FighterParams::brawler(),
            _ => FighterParams::bruiser(),
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

        // Attacks are the size of the body: a big fighter's hitboxes are bigger and reach further.
        p.hitbox_scale = scale(p.hitbox_scale, size);

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

// ---- The point budget ------------------------------------------------------------------------------------------------

/// The most points a ranked-legal fighter may spend. Each stat is worth its value, so the neutral recipe (5 + 5 + 5 + 5)
/// spends exactly the budget: to raise one stat, lower another. Casual play has no limit.
pub const BUDGET: u8 = 20;

impl Recipe {
    /// Points spent: the sum of the four stats.
    pub fn points(self) -> u8 {
        let r = self.clamped();
        r.size + r.speed + r.jump + r.weight
    }

    /// Within the budget, so legal under ranked rules.
    pub fn is_legal(self) -> bool {
        self.points() <= BUDGET
    }
}

// ---- Fighters on the wire -------------------------------------------------------------------------------------------------

/// Which fighter a player brings to a match: one of the base roster's, or one made from a recipe. It is a few bytes, so
/// it travels in the network handshake and both sides build the same match from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FighterSpec {
    /// An index into the base roster's fighters.
    Builtin(u8),
    Made(Recipe),
}

impl FighterSpec {
    pub fn encode(self) -> Vec<u8> {
        match self {
            FighterSpec::Builtin(i) => vec![0, i],
            FighterSpec::Made(r) => {
                let r = r.clamped();
                vec![1, r.class, r.size, r.speed, r.jump, r.weight]
            }
        }
    }

    /// Reads a spec strictly: the exact length, a known kind, and every value in range. Anything else is `None`.
    pub fn decode(bytes: &[u8]) -> Option<FighterSpec> {
        match bytes {
            [0, i] => Some(FighterSpec::Builtin(*i)),
            [1, class, size, speed, jump, weight] => {
                let stats = [*size, *speed, *jump, *weight];
                if *class >= CLASSES || stats.iter().any(|s| !(MIN_STAT..=MAX_STAT).contains(s)) {
                    return None;
                }
                Some(FighterSpec::Made(Recipe {
                    class: *class,
                    size: *size,
                    speed: *speed,
                    jump: *jump,
                    weight: *weight,
                }))
            }
            _ => None,
        }
    }
}

/// Why a set of fighters cannot make a match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecError {
    /// Bytes that are not a fighter spec (or missing).
    Undecodable,
    /// A built-in fighter or a class that the base content does not have.
    UnknownFighter,
    /// Ranked rules and a fighter over the point budget.
    OverBudget,
    TooMany,
    /// A stage index that does not exist (see `stages`).
    UnknownStage,
}

/// The fighter index each spec gets in the match content built by [`match_content`]: built-in fighters keep theirs, made
/// ones are appended after the base roster in the order given.
pub fn resolve(
    base_fighters: usize,
    base_weapons: usize,
    specs: &[FighterSpec],
    ranked: bool,
) -> Result<Vec<u8>, SpecError> {
    let mut next = base_fighters;
    let mut out = Vec::with_capacity(specs.len());
    for spec in specs {
        match spec {
            FighterSpec::Builtin(i) => {
                if usize::from(*i) >= base_fighters {
                    return Err(SpecError::UnknownFighter);
                }
                out.push(*i);
            }
            FighterSpec::Made(r) => {
                if usize::from(r.class) >= base_weapons {
                    return Err(SpecError::UnknownFighter);
                }
                if ranked && !r.is_legal() {
                    return Err(SpecError::OverBudget);
                }
                if next > 255 {
                    return Err(SpecError::TooMany);
                }
                out.push(next as u8);
                next += 1;
            }
        }
    }
    Ok(out)
}

/// The content for a match: `base` plus every made fighter in `specs`, and each spec's fighter index.
pub fn match_content(
    base: &sim_core::Content,
    specs: &[FighterSpec],
    ranked: bool,
) -> Result<(sim_core::Content, Vec<u8>), SpecError> {
    let chars = resolve(base.fighters.len(), base.weapons.len(), specs, ranked)?;
    let mut content = base.clone();
    for spec in specs {
        if let FighterSpec::Made(r) = spec {
            content.fighters.push(r.params());
            let n = content.names.fighters.len();
            content.names.fighters.push(format!("made{n}"));
        }
    }
    Ok((content, chars))
}

/// Like [`match_content`], on stage `stage` (0 keeps the base roster's own stage).
pub fn match_content_on(
    base: &sim_core::Content,
    specs: &[FighterSpec],
    ranked: bool,
    stage: u8,
) -> Result<(sim_core::Content, Vec<u8>), SpecError> {
    let (content, chars) = match_content(base, specs, ranked)?;
    let content = crate::stages::with_stage(&content, stage).ok_or(SpecError::UnknownStage)?;
    Ok((content, chars))
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
    fn the_neutral_recipe_spends_exactly_the_budget() {
        assert_eq!(Recipe::default().points(), BUDGET);
        assert!(Recipe::default().is_legal());
        let more = Recipe {
            size: 6,
            ..Recipe::default()
        };
        assert!(!more.is_legal());
        let traded = Recipe {
            size: 4,
            speed: 6,
            ..Recipe::default()
        };
        assert!(traded.is_legal());
    }

    #[test]
    fn specs_round_trip_and_reject_garbage() {
        for spec in [
            FighterSpec::Builtin(0),
            FighterSpec::Builtin(7),
            FighterSpec::Made(Recipe::default()),
            FighterSpec::Made(Recipe {
                class: 1,
                size: 9,
                speed: 1,
                jump: 3,
                weight: 9,
            }),
            FighterSpec::Made(Recipe {
                class: 2,
                ..Recipe::default()
            }),
        ] {
            assert_eq!(FighterSpec::decode(&spec.encode()), Some(spec));
        }
        for bad in [
            vec![],
            vec![0],
            vec![0, 1, 2],
            vec![1, 0, 5, 5, 5],
            vec![1, 0, 5, 5, 5, 5, 5],
            vec![1, 0, 0, 5, 5, 5],
            vec![1, 0, 10, 5, 5, 5],
            vec![1, 3, 5, 5, 5, 5],
            vec![2, 0, 0],
        ] {
            assert_eq!(FighterSpec::decode(&bad), None, "{bad:?}");
        }
        let mut seed = 3u32;
        for _ in 0..5000 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let len = (seed >> 28) as usize;
            let bytes: Vec<u8> = (0..len)
                .map(|_| {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (seed >> 24) as u8 % 12
                })
                .collect();
            let _ = FighterSpec::decode(&bytes);
        }
    }

    #[test]
    fn match_content_appends_made_fighters_in_order() {
        let base = sim_core::Content::placeholder();
        let a = Recipe {
            size: 9,
            ..Recipe::default()
        };
        let b = Recipe {
            class: 1,
            size: 1,
            ..Recipe::default()
        };
        let specs = [
            FighterSpec::Made(a),
            FighterSpec::Builtin(1),
            FighterSpec::Made(b),
        ];
        let (content, chars) = match_content(&base, &specs, false).unwrap();
        let n = base.fighters.len() as u8;
        assert_eq!(chars, vec![n, 1, n + 1]);
        assert_eq!(content.fighters.len(), base.fighters.len() + 2);
        assert_eq!(content.fighters[usize::from(n)], a.params());
        assert_eq!(content.fighters[usize::from(n) + 1], b.params());
        assert_eq!(&content.fighters[..usize::from(n)], &base.fighters[..]);
        assert_eq!(crate::validate(&content), Ok(()));
        // The same specs always make the same content (and the same hash).
        assert_eq!(
            match_content(&base, &specs, false).unwrap().0.hash(),
            content.hash()
        );
    }

    #[test]
    fn match_content_refuses_what_it_cannot_make() {
        let base = sim_core::Content::placeholder();
        assert_eq!(
            match_content(&base, &[FighterSpec::Builtin(9)], false).unwrap_err(),
            SpecError::UnknownFighter
        );
        let big = Recipe {
            size: 9,
            speed: 9,
            jump: 9,
            weight: 9,
            ..Recipe::default()
        };
        assert!(match_content(&base, &[FighterSpec::Made(big)], false).is_ok());
        assert_eq!(
            match_content(&base, &[FighterSpec::Made(big)], true).unwrap_err(),
            SpecError::OverBudget
        );
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
            (CLASSES - 1, 1, 9, 1, 9)
        );
        assert_eq!(wild.params(), c.params());
    }
}
