//! The stage library: the stages a match can be played on. A stage is chosen by index when a match is set up (character select, or the
//! host online); the match content is the base roster with that stage swapped in, so the simulation still sees one stage and every
//! match, replay and handshake agrees on it through the content hash.
//!
//! Stage 0 is the base roster's own stage ("Meadow"); the rest are defined here. Like the recipes, this is code for now: the text
//! format holds one stage, so a bundle's own stage is stage 0. Everything here is original geometry.

use sim_core::content::{Ledge, Platform};
use sim_core::{Content, Fx, Stage, Vec2, MAX_FIGHTERS};

/// How many stages there are, counting the base roster's.
pub const COUNT: u8 = 5;

pub const NAMES: [&str; COUNT as usize] = [
    "Meadow",
    "Triple Tier",
    "Flat Island",
    "Skyline",
    "Treetop Isle",
];

/// One line about each stage, for the picker.
pub const BLURBS: [&str; COUNT as usize] = [
    "A solid stage with a platform on each side.",
    "A wide stage with three platforms stacked like steps: more room to juggle in the air.",
    "No platforms at all, and a long floor. Pure ground game.",
    "A narrow stage with a high roof of platforms hanging out past the edges.",
    "A floating island under a giant old tree: two low platforms and a high one in the middle.",
];

/// The backdrop each stage is drawn with (presentation only; the stage editor can change it).
pub const BACKDROPS: [&str; COUNT as usize] = ["meadow", "sunset", "ocean", "city", "grove"];

/// How stage `index` looks.
pub fn look(index: u8) -> sim_core::content::StageLook {
    sim_core::content::StageLook {
        backdrop: BACKDROPS
            .get(usize::from(index))
            .copied()
            .unwrap_or("meadow")
            .to_string(),
        ..Default::default()
    }
}

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
        4 => {
            // An island that narrows as it goes down, built from stacked solid blocks so the underside you can hit is the shape
            // you see; two low platforms and a high one in the middle.
            let mut platforms = vec![Platform {
                left: int(-11),
                right: int(11),
                y: Fx::ZERO,
                bottom: int(-2),
                pass_through: false,
            }];
            for (half, top, bottom) in [(9, -2, -4), (7, -4, -6), (4, -6, -8)] {
                platforms.push(Platform {
                    left: int(-half),
                    right: int(half),
                    y: int(top),
                    bottom: int(bottom),
                    pass_through: false,
                });
            }
            platforms.push(shelf(frac(-17, 2), frac(-7, 2), frac(17, 5)));
            platforms.push(shelf(frac(7, 2), frac(17, 2), frac(17, 5)));
            platforms.push(shelf(frac(-5, 2), frac(5, 2), frac(34, 5)));
            Some(Stage {
                platforms,
                ledges: edges(-11, 11),
                spawns: spawns(),
                blast_left: int(-28),
                blast_right: int(28),
                blast_bottom: int(-17),
                blast_top: int(26),
            })
        }
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
        out.look = look(index);
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
    fn a_stage_look_is_presentation_only_and_round_trips() {
        let base = Content::placeholder();
        let isle = with_stage(&base, 4).unwrap();
        assert_eq!(isle.look.backdrop, "grove");
        // The look never changes the content hash (two players may see different skies).
        let mut recoloured = isle.clone();
        recoloured.look.sky_top = "ff8800".to_string();
        recoloured.look.backdrop = "night".to_string();
        assert_eq!(recoloured.hash(), isle.hash());
        // It survives the text format.
        let text = crate::to_text(&recoloured, "t", "", "", false);
        let back = crate::load(&text)
            .unwrap_or_else(|e| panic!("{e:?}"))
            .content;
        assert_eq!(back.look, recoloured.look);
        // Validation knows the backdrops and the colour format.
        let mut bad = isle.clone();
        bad.look.backdrop = "castle".to_string();
        assert!(validate(&bad).is_err());
        let mut bad = isle;
        bad.look.sky_bottom = "#12345".to_string();
        assert!(validate(&bad).is_err());
    }

    #[test]
    fn the_treetop_isle_narrows_underneath_with_nothing_to_stand_on_below() {
        let s = preset(4).unwrap();
        // Every solid block under the top one is narrower than the one above it, so its top is covered.
        let solids: Vec<_> = s.platforms.iter().filter(|p| !p.pass_through).collect();
        for pair in solids.windows(2) {
            assert!(pair[1].left > pair[0].left && pair[1].right < pair[0].right);
            assert_eq!(pair[1].y, pair[0].bottom);
        }
        assert_eq!(s.platforms.iter().filter(|p| p.pass_through).count(), 3);
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
