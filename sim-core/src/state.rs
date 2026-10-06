//! Everything that changes during a match lives in [`GameState`], and nothing else does.
//!
//! The state is plain-old-data and `Copy`, so a snapshot is a memcpy: `let saved = state;`
//! and a restore is `state = saved;`.

use crate::content::Content;
use crate::hash::{StateHash, StateHasher};
use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::{MAX_FIGHTERS, MAX_SCRIPT_VARS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fighter {
    pub pos: Vec2,
    pub vel: Vec2,
    /// Index into `Content::fighters`.
    pub char_id: u8,
    /// -1 faces left, +1 faces right.
    pub facing: i8,
    pub grounded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameState {
    pub frame: u32,
    pub rng: Rng,
    pub fighters: [Fighter; MAX_FIGHTERS],
    pub script_vars: [i32; MAX_SCRIPT_VARS],
}

impl GameState {
    /// Fresh match: every fighter on its stage spawn point. `char_ids` indexes `content.fighters`.
    pub fn new(content: &Content, seed: u64, char_ids: [u8; MAX_FIGHTERS]) -> GameState {
        let fighters = core::array::from_fn(|i| Fighter {
            pos: content.stage.spawns[i],
            vel: Vec2::ZERO,
            char_id: char_ids[i],
            facing: if i % 2 == 0 { 1 } else { -1 },
            grounded: content.stage.spawns[i].y <= content.stage.floor_y,
        });
        GameState {
            frame: 0,
            rng: Rng::new(seed),
            fighters,
            script_vars: [0; MAX_SCRIPT_VARS],
        }
    }

    /// Per-frame checksum over the whole state.
    pub fn checksum(&self) -> u64 {
        let mut h = StateHasher::new();
        self.hash_into(&mut h);
        h.finish()
    }
}

impl StateHash for Fighter {
    fn hash_into(&self, h: &mut StateHasher) {
        self.pos.hash_into(h);
        self.vel.hash_into(h);
        h.write_u8(self.char_id);
        h.write_i8(self.facing);
        h.write_bool(self.grounded);
    }
}

impl StateHash for GameState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(self.frame);
        self.rng.hash_into(h);
        for f in &self.fighters {
            f.hash_into(h);
        }
        for v in &self.script_vars {
            h.write_i32(*v);
        }
    }
}

impl Default for Fighter {
    fn default() -> Self {
        Fighter {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            char_id: 0,
            facing: 1,
            grounded: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fx;

    #[test]
    fn snapshot_is_a_copy() {
        let content = Content::placeholder();
        let mut a = GameState::new(&content, 5, [0, 1, 0, 1]);
        let saved = a;
        a.frame = 99;
        a.fighters[0].pos.x = Fx::from_int(7);
        assert_ne!(a.checksum(), saved.checksum());
        a = saved;
        assert_eq!(a.checksum(), saved.checksum());
    }

    #[test]
    fn checksum_sees_every_field() {
        let content = Content::placeholder();
        let base = GameState::new(&content, 5, [0, 1, 0, 1]);
        let c = base.checksum();

        let mut s = base;
        s.script_vars[MAX_SCRIPT_VARS - 1] = 1;
        assert_ne!(c, s.checksum());

        let mut s = base;
        s.rng = Rng::new(6);
        assert_ne!(c, s.checksum());

        let mut s = base;
        s.fighters[MAX_FIGHTERS - 1].grounded = false;
        assert_ne!(c, s.checksum());

        let mut s = base;
        s.fighters[2].vel.y = Fx::from_raw(1);
        assert_ne!(c, s.checksum());
    }
}
