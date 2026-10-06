//! Immutable, hash-verified content. It is never part of the snapshot; both peers must hold
//! identical content, which the netplay handshake checks via [`Content::hash`].
//!
//! These are the *types*. Loading and validation live in `sim-content`.

use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::vec2::Vec2;
use crate::{MAX_FIGHTERS, SIM_VERSION};

/// Per-character physics. Every tunable lives here, never in code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FighterParams {
    pub run_speed: Fx,
    pub gravity: Fx,
    pub jump_velocity: Fx,
    pub max_fall_speed: Fx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stage {
    pub floor_y: Fx,
    pub spawns: [Vec2; MAX_FIGHTERS],
    pub blast_left: Fx,
    pub blast_right: Fx,
    pub blast_bottom: Fx,
    pub blast_top: Fx,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub fighters: Vec<FighterParams>,
    pub stage: Stage,
}

impl Content {
    /// Hash of the sim version plus every content field.
    pub fn hash(&self) -> u64 {
        let mut h = StateHasher::new();
        h.write_u16(SIM_VERSION);
        h.write_u32(self.fighters.len() as u32);
        for f in &self.fighters {
            f.run_speed.hash_into(&mut h);
            f.gravity.hash_into(&mut h);
            f.jump_velocity.hash_into(&mut h);
            f.max_fall_speed.hash_into(&mut h);
        }
        let s = &self.stage;
        s.floor_y.hash_into(&mut h);
        for p in &s.spawns {
            p.hash_into(&mut h);
        }
        s.blast_left.hash_into(&mut h);
        s.blast_right.hash_into(&mut h);
        s.blast_bottom.hash_into(&mut h);
        s.blast_top.hash_into(&mut h);
        h.finish()
    }

    /// Two capsule-free placeholder fighters with different physics profiles on a flat stage.
    /// Units: 1.0 = one world unit; speeds are per 60 Hz frame.
    pub fn placeholder() -> Content {
        let r = Fx::from_ratio;
        let spawn = |x: i32| Vec2::new(Fx::from_int(x), Fx::ZERO);
        Content {
            fighters: vec![
                // 0: balanced
                FighterParams {
                    run_speed: r(2, 15),
                    gravity: r(1, 150),
                    jump_velocity: r(11, 50),
                    max_fall_speed: r(1, 5),
                },
                // 1: floaty
                FighterParams {
                    run_speed: r(1, 10),
                    gravity: r(1, 250),
                    jump_velocity: r(9, 50),
                    max_fall_speed: r(1, 8),
                },
            ],
            stage: Stage {
                floor_y: Fx::ZERO,
                spawns: [spawn(-6), spawn(-2), spawn(2), spawn(6)],
                blast_left: Fx::from_int(-40),
                blast_right: Fx::from_int(40),
                blast_bottom: Fx::from_int(-30),
                blast_top: Fx::from_int(30),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_changes_with_any_field() {
        let base = Content::placeholder();
        let h = base.hash();
        assert_eq!(h, Content::placeholder().hash());

        let mut c = base.clone();
        c.fighters[1].gravity += Fx::from_raw(1);
        assert_ne!(h, c.hash());

        let mut c = base.clone();
        c.stage.blast_top += Fx::from_raw(1);
        assert_ne!(h, c.hash());

        let mut c = base;
        c.fighters.pop();
        assert_ne!(h, c.hash());
    }
}
