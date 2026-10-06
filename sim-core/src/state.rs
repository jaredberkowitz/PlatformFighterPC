//! Everything that changes during a match lives in [`GameState`], and nothing else does.
//!
//! The state is plain-old-data and `Copy`, so a snapshot is a memcpy: `let saved = state;`
//! and a restore is `state = saved;`.

use crate::collision;
use crate::content::{Content, MAX_LEDGES};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::input::Input;
use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::{MAX_FIGHTERS, MAX_SCRIPT_VARS};

/// Frames of input kept per fighter. Must cover the longest buffer window in `FighterParams`.
pub const HISTORY_LEN: usize = 12;
pub const NONE: i8 = -1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FighterState {
    Idle = 0,
    Walk,
    Run,
    Dash,
    Turn,
    Crouch,
    JumpSquat,
    Airborne,
    AirDodge,
    Landing,
    WaveLand,
    Shield,
    ShieldDrop,
    LedgeHang,
    LedgeGetUp,
    LedgeAttack,
    /// Special fall: no air jumps or air dodge, can still drift and grab ledges.
    Helpless,
    /// Performing a move; `Fighter::move_id` says which and `state_frame` is the move frame.
    Attack,
    /// Launched or flinching from a hit.
    Hitstun,
}

impl StateHash for FighterState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(*self as u8);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fighter {
    /// Feet position (bottom centre of the ECB).
    pub pos: Vec2,
    pub vel: Vec2,
    /// Normalised direction of the current or last air dodge.
    pub dodge_dir: Vec2,
    /// Index into `Content::fighters`.
    pub char_id: u8,
    /// -1 faces left, +1 faces right.
    pub facing: i8,
    pub state: FighterState,
    /// Frames since entering `state`.
    pub state_frame: u16,
    /// Index of the platform being stood on, or [`NONE`].
    pub platform: i8,
    pub air_jumps_left: u8,
    pub air_dodge_used: bool,
    /// Frames of lag for the current `Landing` state (normal vs helpless landing).
    pub lag: u8,
    pub fast_fall: bool,
    /// While non-zero, pass-through platforms are ignored (set by shield drop).
    pub platform_ignore: u8,
    /// Ledge currently held, or [`NONE`].
    pub ledge: i8,
    pub ledge_invuln: u8,
    /// Grabs since last touching stable ground; drives the diminishing invincibility.
    pub ledge_grab_count: u8,
    pub ledge_cooldown: u8,
    // ---- Combat ----
    /// Damage taken, in percent.
    pub percent: Fx,
    pub stocks: u8,
    /// Frames of freeze after hitting or being hit. Nothing about the fighter advances during hitlag.
    pub hitlag: u8,
    /// Frames of hitstun remaining once hitlag ends.
    pub hitstun: u16,
    /// A hit is waiting to launch at the end of hitlag (so DI can bend it).
    pub launch_pending: bool,
    pub launch_kb: Fx,
    /// World angle of the pending launch, in `trig::Angle` units.
    pub launch_angle: u16,
    /// Current launch velocity, decaying each frame.
    pub kb_vel: Vec2,
    pub tumble: bool,
    /// Index of the move being performed, see `moves::MoveId`.
    pub move_id: u8,
    /// Bit `n` set means this move has already hit fighter `n`.
    pub hit_mask: u8,
    /// Invulnerable frames remaining (respawn).
    pub invuln: u8,
    /// `history[0]` is this frame's input, `history[1]` the previous frame's, and so on.
    pub history: [Input; HISTORY_LEN],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameState {
    pub frame: u32,
    pub rng: Rng,
    pub fighters: [Fighter; MAX_FIGHTERS],
    /// Which fighter holds each ledge, or [`NONE`].
    pub ledge_owner: [i8; MAX_LEDGES],
    pub script_vars: [i32; MAX_SCRIPT_VARS],
}

impl Fighter {
    pub fn spawn(pos: Vec2, char_id: u8, facing: i8, platform: i8, air_jumps: u8) -> Fighter {
        Fighter {
            pos,
            vel: Vec2::ZERO,
            dodge_dir: Vec2::ZERO,
            char_id,
            facing,
            state: if platform == NONE {
                FighterState::Airborne
            } else {
                FighterState::Idle
            },
            state_frame: 0,
            platform,
            air_jumps_left: air_jumps,
            air_dodge_used: false,
            lag: 0,
            fast_fall: false,
            platform_ignore: 0,
            ledge: NONE,
            ledge_invuln: 0,
            ledge_grab_count: 0,
            ledge_cooldown: 0,
            percent: Fx::ZERO,
            stocks: 3,
            hitlag: 0,
            hitstun: 0,
            launch_pending: false,
            launch_kb: Fx::ZERO,
            launch_angle: 0,
            kb_vel: Vec2::ZERO,
            tumble: false,
            move_id: 0,
            hit_mask: 0,
            invuln: 0,
            history: [Input::default(); HISTORY_LEN],
        }
    }

    pub fn grounded(&self) -> bool {
        self.platform != NONE
    }
}

impl GameState {
    /// Fresh match: every fighter on its stage spawn point. `char_ids` indexes `content.fighters`.
    pub fn new(content: &Content, seed: u64, char_ids: [u8; MAX_FIGHTERS]) -> GameState {
        let fighters = core::array::from_fn(|i| {
            let pos = content.stage.spawns[i];
            let idx = usize::from(char_ids[i]).min(content.fighters.len().saturating_sub(1));
            Fighter::spawn(
                pos,
                char_ids[i],
                if i % 2 == 0 { 1 } else { -1 },
                collision::standing_on(&content.stage, pos),
                content.fighters[idx].air_jumps,
            )
        });
        GameState {
            frame: 0,
            rng: Rng::new(seed),
            fighters,
            ledge_owner: [NONE; MAX_LEDGES],
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
        self.dodge_dir.hash_into(h);
        h.write_u8(self.char_id);
        h.write_i8(self.facing);
        self.state.hash_into(h);
        h.write_u16(self.state_frame);
        h.write_i8(self.platform);
        h.write_u8(self.air_jumps_left);
        h.write_bool(self.air_dodge_used);
        h.write_u8(self.lag);
        h.write_bool(self.fast_fall);
        h.write_u8(self.platform_ignore);
        h.write_i8(self.ledge);
        h.write_u8(self.ledge_invuln);
        h.write_u8(self.ledge_grab_count);
        h.write_u8(self.ledge_cooldown);
        self.percent.hash_into(h);
        h.write_u8(self.stocks);
        h.write_u8(self.hitlag);
        h.write_u16(self.hitstun);
        h.write_bool(self.launch_pending);
        self.launch_kb.hash_into(h);
        h.write_u16(self.launch_angle);
        self.kb_vel.hash_into(h);
        h.write_bool(self.tumble);
        h.write_u8(self.move_id);
        h.write_u8(self.hit_mask);
        h.write_u8(self.invuln);
        for input in &self.history {
            input.hash_into(h);
        }
    }
}

impl StateHash for GameState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(self.frame);
        self.rng.hash_into(h);
        for f in &self.fighters {
            f.hash_into(h);
        }
        for o in &self.ledge_owner {
            h.write_i8(*o);
        }
        for v in &self.script_vars {
            h.write_i32(*v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fx;

    fn base() -> GameState {
        GameState::new(&Content::placeholder(), 5, [0, 1, 0, 1])
    }

    #[test]
    fn snapshot_is_a_copy() {
        let mut a = base();
        let saved = a;
        a.frame = 99;
        a.fighters[0].pos.x = Fx::from_int(7);
        assert_ne!(a.checksum(), saved.checksum());
        a = saved;
        assert_eq!(a.checksum(), saved.checksum());
    }

    #[test]
    fn fighters_spawn_standing_on_the_main_stage() {
        for f in &base().fighters {
            assert_eq!(f.platform, 0);
            assert_eq!(f.state, FighterState::Idle);
        }
    }

    /// Every field must feed the checksum, or a desync in it would go undetected.
    #[test]
    fn checksum_sees_every_field() {
        let state = base();
        let c = state.checksum();
        let changed: Vec<(&str, GameState)> = vec![
            ("frame", {
                let mut s = state;
                s.frame += 1;
                s
            }),
            ("rng", {
                let mut s = state;
                s.rng = Rng::new(6);
                s
            }),
            ("ledge_owner", {
                let mut s = state;
                s.ledge_owner[MAX_LEDGES - 1] = 0;
                s
            }),
            ("script_vars", {
                let mut s = state;
                s.script_vars[MAX_SCRIPT_VARS - 1] = 1;
                s
            }),
            ("pos", {
                let mut s = state;
                s.fighters[3].pos.y = Fx::from_raw(1);
                s
            }),
            ("vel", {
                let mut s = state;
                s.fighters[2].vel.y = Fx::from_raw(1);
                s
            }),
            ("dodge_dir", {
                let mut s = state;
                s.fighters[1].dodge_dir.x = Fx::from_raw(1);
                s
            }),
            ("char_id", {
                let mut s = state;
                s.fighters[0].char_id = 1;
                s
            }),
            ("facing", {
                let mut s = state;
                s.fighters[0].facing = -1;
                s
            }),
            ("state", {
                let mut s = state;
                s.fighters[0].state = FighterState::Shield;
                s
            }),
            ("state_frame", {
                let mut s = state;
                s.fighters[0].state_frame = 1;
                s
            }),
            ("platform", {
                let mut s = state;
                s.fighters[0].platform = NONE;
                s
            }),
            ("air_jumps_left", {
                let mut s = state;
                s.fighters[0].air_jumps_left = 0;
                s
            }),
            ("air_dodge_used", {
                let mut s = state;
                s.fighters[0].air_dodge_used = true;
                s
            }),
            ("lag", {
                let mut s = state;
                s.fighters[0].lag = 1;
                s
            }),
            ("fast_fall", {
                let mut s = state;
                s.fighters[0].fast_fall = true;
                s
            }),
            ("platform_ignore", {
                let mut s = state;
                s.fighters[0].platform_ignore = 1;
                s
            }),
            ("ledge", {
                let mut s = state;
                s.fighters[0].ledge = 0;
                s
            }),
            ("ledge_invuln", {
                let mut s = state;
                s.fighters[0].ledge_invuln = 1;
                s
            }),
            ("ledge_grab_count", {
                let mut s = state;
                s.fighters[0].ledge_grab_count = 1;
                s
            }),
            ("ledge_cooldown", {
                let mut s = state;
                s.fighters[0].ledge_cooldown = 1;
                s
            }),
            ("percent", {
                let mut s = state;
                s.fighters[0].percent = Fx::from_int(1);
                s
            }),
            ("stocks", {
                let mut s = state;
                s.fighters[0].stocks = 1;
                s
            }),
            ("hitlag", {
                let mut s = state;
                s.fighters[0].hitlag = 1;
                s
            }),
            ("hitstun", {
                let mut s = state;
                s.fighters[0].hitstun = 1;
                s
            }),
            ("launch_pending", {
                let mut s = state;
                s.fighters[0].launch_pending = true;
                s
            }),
            ("launch_kb", {
                let mut s = state;
                s.fighters[0].launch_kb = Fx::from_int(1);
                s
            }),
            ("launch_angle", {
                let mut s = state;
                s.fighters[0].launch_angle = 1;
                s
            }),
            ("kb_vel", {
                let mut s = state;
                s.fighters[0].kb_vel.y = Fx::from_raw(1);
                s
            }),
            ("tumble", {
                let mut s = state;
                s.fighters[0].tumble = true;
                s
            }),
            ("move_id", {
                let mut s = state;
                s.fighters[0].move_id = 1;
                s
            }),
            ("hit_mask", {
                let mut s = state;
                s.fighters[0].hit_mask = 1;
                s
            }),
            ("invuln", {
                let mut s = state;
                s.fighters[0].invuln = 1;
                s
            }),
            ("history", {
                let mut s = state;
                s.fighters[0].history[HISTORY_LEN - 1].buttons = 1;
                s
            }),
        ];
        for (name, s) in changed {
            assert_ne!(c, s.checksum(), "checksum ignores `{name}`");
        }
    }
}
