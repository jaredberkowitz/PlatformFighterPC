//! The stage library: the stages a match can be played on. A stage is chosen by index when a match is set up (character select, or the
//! host online); the match content is the base roster with that stage swapped in, so the simulation still sees one stage and every
//! match, replay and handshake agrees on it through the content hash.
//!
//! Stage 0 is the base roster's own stage ("Meadow"); the rest are defined here. Like the recipes, this is code for now: the text
//! format holds one stage, so a bundle's own stage is stage 0. Everything here is original geometry.

use sim_core::content::{Ledge, Platform};
use sim_core::{Content, Fx, Stage, Vec2, MAX_FIGHTERS};

/// How many stages there are, counting the base roster's.
pub const COUNT: u8 = 4;

pub const NAMES: [&str; COUNT as usize] = ["Meadow", "Triple Tier", "Flat Island", "Skyline"];

/// One line about each stage, for the picker.
pub const BLURBS: [&str; COUNT as usize] = [
    "A solid stage with a platform on each side.",
    "A wide stage with three platforms stacked like steps: more room to juggle in the air.",
    "No platforms at all, and a long floor. Pure ground game.",
    "A narrow stage with a high roof of platforms hanging out past the edges.",
];

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

fn frac(n: i32, d: i32) -> Fx {
    Fx::from_ratio(n, d)
}

fn ground(left: i32, right: i32) -> Platform {
    Platform {
        left: int(left),
        right: int(right),
        y: Fx::ZERO,
        bottom: int(-8),
        pass_through: false,
    }
}

fn shelf(left: Fx, right: Fx, y: Fx) -> Platform {
    Platform {
        left,
        right,
        y,
        bottom: y,
        pass_through: true,
    }
}

fn edges(left: i32, right: i32) -> Vec<Ledge> {
    vec![
        Ledge {
            x: int(left),
            y: Fx::ZERO,
            side: -1,
        },
        Ledge {
            x: int(right),
            y: Fx::ZERO,
            side: 1,
        },
    ]
}

fn spawns() -> [Vec2; MAX_FIGHTERS] {
    let at = |x: i32| Vec2::new(int(x), Fx::ZERO);
    [at(-4), at(-1), at(1), at(4)]
}

/// The stage with this index, or `None` if there is none. Stage 0 is the base roster's, so it is not built here.
pub fn preset(index: u8) -> Option<Stage> {
    match index {
        0 => Some(Stage::placeholder()),
        1 => Some(Stage {
            platforms: vec![
                ground(-12, 12),
                shelf(int(-9), int(-4), frac(18, 5)),
                shelf(int(4), int(9), frac(18, 5)),
                shelf(frac(-5, 2), frac(5, 2), frac(36, 5)),
            ],
            ledges: edges(-12, 12),
            spawns: spawns(),
            blast_left: int(-30),
            blast_right: int(30),
            blast_bottom: int(-17),
            blast_top: int(26),
        }),
        2 => Some(Stage {
            platforms: vec![ground(-14, 14)],
            ledges: edges(-14, 14),
            spawns: spawns(),
            blast_left: int(-26),
            blast_right: int(26),
            blast_bottom: int(-16),
            blast_top: int(22),
        }),
        3 => Some(Stage {
            platforms: vec![
                ground(-9, 9),
                shelf(int(-14), int(-8), frac(16, 5)),
                shelf(int(8), int(14), frac(16, 5)),
                shelf(int(-3), int(3), frac(34, 5)),
            ],
            ledges: edges(-9, 9),
            spawns: spawns(),
            blast_left: int(-27),
            blast_right: int(27),
            blast_bottom: int(-18),
            blast_top: int(25),
        }),
        _ => None,
    }
}

pub fn name(index: u8) -> &'static str {
    NAMES.get(usize::from(index)).copied().unwrap_or("?")
}

/// The content with stage `index` swapped in. Stage 0 leaves the content as it is (so a base-roster match keeps its own hash).
/// `None` for a stage that does not exist.
pub fn with_stage(content: &Content, index: u8) -> Option<Content> {
    if index >= COUNT {
        return None;
    }
    let mut out = content.clone();
    if index != 0 {
        out.stage = preset(index)?;
        out.names.stage = name(index).to_string();
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate;
    use sim_core::collision;
    use sim_core::fuzz::random_inputs;
    use sim_core::{step, GameState, Rng};

    #[test]
    fn every_stage_is_valid_and_distinct() {
        let base = Content::placeholder();
        let mut hashes = Vec::new();
        for i in 0..COUNT {
            let c = with_stage(&base, i).expect("stage exists");
            assert_eq!(validate(&c), Ok(()), "stage {i} ({}) is valid", name(i));
            hashes.push(c.hash());
        }
        hashes.sort_unstable();
        hashes.dedup();
        assert_eq!(
            hashes.len(),
            usize::from(COUNT),
            "every stage is a different content"
        );
        assert!(preset(COUNT).is_none() && with_stage(&base, COUNT).is_none());
        assert_eq!(
            with_stage(&base, 0).unwrap().hash(),
            base.hash(),
            "stage 0 is the base roster's own"
        );
    }

    #[test]
    fn everyone_spawns_standing_on_every_stage() {
        for i in 0..COUNT {
            let s = preset(i).unwrap();
            for sp in &s.spawns {
                assert!(
                    collision::standing_on(&s, *sp) >= 0,
                    "stage {i}: a spawn is in the air"
                );
            }
            assert_eq!(
                s.ledges.len(),
                2,
                "stage {i}: both ends of the floor are ledges"
            );
        }
    }

    /// Random play on every stage never panics and never leaves a fighter inside a solid block.
    #[test]
    fn random_play_is_safe_on_every_stage() {
        let base = Content::placeholder();
        for i in 0..COUNT {
            let content = with_stage(&base, i).unwrap();
            for seed in 0..6u64 {
                let inputs = random_inputs(&mut Rng::new(seed + 100 * u64::from(i)), 1500);
                let mut s = GameState::new(&content, seed, [0, 1, 0, 1]);
                for frame in &inputs {
                    step(&mut s, &content, frame);
                    for (n, f) in s.fighters.iter().enumerate() {
                        for b in content.stage.platforms.iter().filter(|b| !b.pass_through) {
                            let inside = f.pos.x > b.left
                                && f.pos.x < b.right
                                && f.pos.y > b.bottom
                                && f.pos.y < b.y;
                            assert!(
                                !inside,
                                "stage {i} seed {seed} fighter {n}: feet inside a solid block"
                            );
                        }
                    }
                }
            }
        }
    }
}
