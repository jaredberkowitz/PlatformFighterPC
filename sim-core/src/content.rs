//! Immutable, hash-verified content. It is never part of the snapshot; both peers must hold
//! identical content, which the netplay handshake checks via [`Content::hash`].
//!
//! These are the *types* and the hand-written placeholder fighters. Loading and validation live in
//! `sim-content`. Units: 1.0 = one world unit; speeds are per 60 Hz frame; `*_frames` are frames.

use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::vec2::Vec2;
use crate::{MAX_FIGHTERS, SIM_VERSION};

pub const MAX_PLATFORMS: usize = 8;
pub const MAX_LEDGES: usize = 8;

/// Per-character physics and movement tech. Every tunable lives here, never in code.
///
/// `fx_fields` and `int_fields` are the single source of truth for hashing and validation, so a
/// field added to one of those lists is automatically hashed and range-checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FighterParams {
    // Ground movement
    pub walk_speed: Fx,
    pub run_speed: Fx,
    pub dash_speed: Fx,
    /// Speed on the first frame of a dash; it ramps to `dash_speed` at `dash_accel` per frame.
    pub dash_initial_speed: Fx,
    pub dash_accel: Fx,
    pub dash_frames: u8,
    pub turn_frames: u8,
    pub ground_accel: Fx,
    pub ground_friction: Fx,
    /// How fast speed above `run_speed` (left over from a dash) bleeds off while running.
    pub run_decel: Fx,
    /// Friction while landing; lower than `ground_friction` so landing keeps momentum.
    pub landing_friction: Fx,
    // Air movement
    pub air_speed: Fx,
    /// Base air acceleration; the stick adds `air_accel_stick` scaled by how far it is pushed.
    pub air_accel: Fx,
    pub air_accel_stick: Fx,
    pub air_friction: Fx,
    pub gravity: Fx,
    pub max_fall_speed: Fx,
    pub fast_fall_speed: Fx,
    // Jumping
    pub jump_squat_frames: u8,
    pub full_hop_velocity: Fx,
    pub short_hop_velocity: Fx,
    pub air_jumps: u8,
    pub air_jump_velocity: Fx,
    pub landing_lag: u8,
    // Air dodge and wavedash
    pub air_dodge_frames: u8,
    pub air_dodge_speed: Fx,
    /// Per-frame velocity multiplier during an air dodge (0..=1).
    pub air_dodge_decay: Fx,
    /// A shield press this many frames ago still starts the air dodge on the first airborne frame.
    pub air_dodge_buffer: u8,
    /// A downward air dodge starting this close to a surface counts as ground contact.
    pub ground_assist_dist: Fx,
    /// A dodge direction counts as a wavedash if its normalised y is at or below minus this value.
    /// Smaller = wider cone.
    pub wavedash_min_down: Fx,
    pub waveland_speed: Fx,
    /// Per-frame velocity multiplier while sliding (0..=1).
    pub waveland_friction: Fx,
    pub waveland_lag: u8,
    // Shield drop
    pub shield_drop_buffer: u8,
    pub shield_drop_recovery: u8,
    pub shield_drop_speed: Fx,
    pub platform_ignore_frames: u8,
    // Ledges
    pub ledge_reach_x: Fx,
    /// The fighter must be at least this far below the ledge to grab it (no instant grab on walk-off).
    pub ledge_min_drop: Fx,
    pub ledge_reach_down: Fx,
    pub ledge_hang_dx: Fx,
    pub ledge_hang_dy: Fx,
    pub ledge_hang_max: u16,
    pub ledge_regrab_cooldown: u8,
    pub ledge_invuln_base: u8,
    pub ledge_invuln_decay: u8,
    pub ledge_invuln_floor: u8,
    pub ledge_getup_frames: u8,
    pub ledge_getup_dx: Fx,
    pub ledge_roll_dx: Fx,
    pub ledge_jump_velocity: Fx,
    pub ledge_jump_dx: Fx,
    pub ledge_trump_vx: Fx,
    pub ledge_trump_vy: Fx,
    // Body (feet-origin ECB; used for hurtbox defaults later)
    pub ecb_half_width: Fx,
    pub ecb_height: Fx,
    /// Height above the feet of the ECB's widest points, used for wall contact.
    pub ecb_side_height: Fx,
    // Special states
    pub helpless_landing_lag: u8,
    pub ledge_attack_frames: u8,
    pub ledge_attack_dx: Fx,
}

impl FighterParams {
    /// Every fixed-point field with its name.
    pub fn fx_fields(&self) -> [(&'static str, Fx); 41] {
        [
            ("walk_speed", self.walk_speed),
            ("run_speed", self.run_speed),
            ("dash_speed", self.dash_speed),
            ("dash_initial_speed", self.dash_initial_speed),
            ("dash_accel", self.dash_accel),
            ("ground_accel", self.ground_accel),
            ("ground_friction", self.ground_friction),
            ("run_decel", self.run_decel),
            ("landing_friction", self.landing_friction),
            ("air_speed", self.air_speed),
            ("air_accel", self.air_accel),
            ("air_accel_stick", self.air_accel_stick),
            ("air_friction", self.air_friction),
            ("gravity", self.gravity),
            ("max_fall_speed", self.max_fall_speed),
            ("fast_fall_speed", self.fast_fall_speed),
            ("full_hop_velocity", self.full_hop_velocity),
            ("short_hop_velocity", self.short_hop_velocity),
            ("air_jump_velocity", self.air_jump_velocity),
            ("air_dodge_speed", self.air_dodge_speed),
            ("air_dodge_decay", self.air_dodge_decay),
            ("ground_assist_dist", self.ground_assist_dist),
            ("wavedash_min_down", self.wavedash_min_down),
            ("waveland_speed", self.waveland_speed),
            ("waveland_friction", self.waveland_friction),
            ("shield_drop_speed", self.shield_drop_speed),
            ("ledge_reach_x", self.ledge_reach_x),
            ("ledge_min_drop", self.ledge_min_drop),
            ("ledge_reach_down", self.ledge_reach_down),
            ("ledge_hang_dx", self.ledge_hang_dx),
            ("ledge_hang_dy", self.ledge_hang_dy),
            ("ledge_getup_dx", self.ledge_getup_dx),
            ("ledge_roll_dx", self.ledge_roll_dx),
            ("ledge_jump_velocity", self.ledge_jump_velocity),
            ("ledge_jump_dx", self.ledge_jump_dx),
            ("ledge_trump_vx", self.ledge_trump_vx),
            ("ledge_trump_vy", self.ledge_trump_vy),
            ("ecb_half_width", self.ecb_half_width),
            ("ecb_height", self.ecb_height),
            ("ecb_side_height", self.ecb_side_height),
            ("ledge_attack_dx", self.ledge_attack_dx),
        ]
    }

    /// Every integer (frame-count) field with its name.
    pub fn int_fields(&self) -> [(&'static str, u32); 19] {
        [
            ("jump_squat_frames", u32::from(self.jump_squat_frames)),
            ("air_jumps", u32::from(self.air_jumps)),
            ("landing_lag", u32::from(self.landing_lag)),
            ("air_dodge_frames", u32::from(self.air_dodge_frames)),
            ("air_dodge_buffer", u32::from(self.air_dodge_buffer)),
            ("waveland_lag", u32::from(self.waveland_lag)),
            ("shield_drop_buffer", u32::from(self.shield_drop_buffer)),
            ("shield_drop_recovery", u32::from(self.shield_drop_recovery)),
            (
                "platform_ignore_frames",
                u32::from(self.platform_ignore_frames),
            ),
            ("ledge_hang_max", u32::from(self.ledge_hang_max)),
            (
                "ledge_regrab_cooldown",
                u32::from(self.ledge_regrab_cooldown),
            ),
            ("ledge_invuln_base", u32::from(self.ledge_invuln_base)),
            ("ledge_invuln_decay", u32::from(self.ledge_invuln_decay)),
            ("ledge_invuln_floor", u32::from(self.ledge_invuln_floor)),
            ("ledge_getup_frames", u32::from(self.ledge_getup_frames)),
            ("dash_frames", u32::from(self.dash_frames)),
            ("turn_frames", u32::from(self.turn_frames)),
            ("helpless_landing_lag", u32::from(self.helpless_landing_lag)),
            ("ledge_attack_frames", u32::from(self.ledge_attack_frames)),
        ]
    }

    /// Converts a Smash-Ultimate-style number (given in thousandths of a unit) into world units.
    /// One world unit is 8 of those units, so a ~17 unit tall fighter is about 2.2 world units tall.
    pub const fn su(thousandths: i32) -> Fx {
        Fx::from_ratio(thousandths, 8000)
    }

    /// Initial upward velocity that makes a jump peak `height` above where it started.
    ///
    /// Gravity is applied before moving each frame, so the peak is `v^2/(2g) - v/2`,
    /// which solves to `v = (g + sqrt(g^2 + 8gH)) / 2`.
    pub fn hop_velocity(gravity: Fx, height: Fx) -> Fx {
        (gravity + (gravity * gravity + (gravity * height).mul_int(8)).sqrt()) * Fx::HALF
    }

    /// Values shared by every fighter: tech that is not character specific (air dodge, wavedash,
    /// shield drop, ledges). Character profiles below override the movement numbers.
    fn base() -> FighterParams {
        let r = Fx::from_ratio;
        let su = FighterParams::su;
        FighterParams {
            walk_speed: su(1400),
            run_speed: su(1800),
            dash_speed: su(2200),
            dash_initial_speed: su(900),
            dash_accel: su(300),
            dash_frames: 12,
            turn_frames: 6,
            ground_accel: su(200),
            ground_friction: su(110),
            run_decel: su(60),
            landing_friction: su(40),
            air_speed: su(1100),
            air_accel: su(10),
            air_accel_stick: su(70),
            air_friction: su(8),
            gravity: su(100),
            max_fall_speed: su(1700),
            fast_fall_speed: su(2700),
            jump_squat_frames: 3,
            full_hop_velocity: r(1, 4),
            short_hop_velocity: r(3, 20),
            air_jumps: 1,
            air_jump_velocity: r(1, 4),
            landing_lag: 3,
            air_dodge_frames: 30,
            air_dodge_speed: r(2, 5),
            air_dodge_decay: r(9, 10),
            air_dodge_buffer: 6,
            ground_assist_dist: r(2, 5),
            wavedash_min_down: r(1, 5),
            waveland_speed: r(7, 20),
            waveland_friction: r(9, 10),
            waveland_lag: 10,
            shield_drop_buffer: 4,
            shield_drop_recovery: 6,
            shield_drop_speed: r(3, 50),
            platform_ignore_frames: 8,
            ledge_reach_x: r(3, 2),
            ledge_min_drop: r(1, 4),
            ledge_reach_down: Fx::from_int(2),
            ledge_hang_dx: r(4, 5),
            ledge_hang_dy: r(9, 5),
            ledge_hang_max: 300,
            ledge_regrab_cooldown: 30,
            ledge_invuln_base: 60,
            ledge_invuln_decay: 10,
            ledge_invuln_floor: 10,
            ledge_getup_frames: 30,
            ledge_getup_dx: Fx::ONE,
            ledge_roll_dx: Fx::from_int(3),
            ledge_jump_velocity: r(11, 50),
            ledge_jump_dx: r(3, 50),
            ledge_trump_vx: r(1, 8),
            ledge_trump_vy: r(1, 15),
            ecb_half_width: r(4, 5),
            ecb_height: r(11, 5),
            ecb_side_height: r(11, 10),
            helpless_landing_lag: 20,
            ledge_attack_frames: 40,
            ledge_attack_dx: r(3, 2),
        }
    }

    /// Fast on the ground, light and floaty in the air: the "longsword duelist" body type.
    /// Movement numbers follow a swordfighter-style profile from the reference game, in world units.
    /// Marth-style reference: walk 1.575, run 1.964, dash 2.255, air speed 1.071, gravity 0.075,
    /// fall 1.58 (fast 2.528), full hop 33.66, short hop 16.26, double jump 33.66.
    pub fn duelist() -> FighterParams {
        let su = FighterParams::su;
        let gravity = su(75);
        FighterParams {
            walk_speed: su(1575),
            run_speed: su(1964),
            dash_speed: su(2255),
            air_speed: su(1071),
            air_friction: su(8),
            gravity,
            max_fall_speed: su(1580),
            fast_fall_speed: su(2528),
            ground_friction: su(114),
            full_hop_velocity: Self::hop_velocity(gravity, su(33660)),
            short_hop_velocity: Self::hop_velocity(gravity, su(16260)),
            air_jump_velocity: Self::hop_velocity(gravity, su(33660)),
            ..FighterParams::base()
        }
    }

    /// Heavier, faster in the air, with a high gravity and fast fall: the "blaster brawler" body type.
    /// Wolf-style reference: walk 1.208, run 1.54, dash 2.09, air speed 1.281, gravity 0.13,
    /// fall 1.8 (fast 2.88), full hop 32.02, short hop 15.38, double jump 30.71.
    pub fn brawler() -> FighterParams {
        let su = FighterParams::su;
        let gravity = su(130);
        FighterParams {
            walk_speed: su(1208),
            run_speed: su(1540),
            dash_speed: su(2090),
            dash_initial_speed: su(840),
            air_speed: su(1281),
            air_accel_stick: su(80),
            air_friction: su(4),
            gravity,
            max_fall_speed: su(1800),
            fast_fall_speed: su(2880),
            ground_friction: su(110),
            full_hop_velocity: Self::hop_velocity(gravity, su(32020)),
            short_hop_velocity: Self::hop_velocity(gravity, su(15380)),
            air_jump_velocity: Self::hop_velocity(gravity, su(30710)),
            ..FighterParams::base()
        }
    }
}

impl StateHash for FighterParams {
    fn hash_into(&self, h: &mut StateHasher) {
        for (_, v) in self.fx_fields() {
            v.hash_into(h);
        }
        for (_, v) in self.int_fields() {
            h.write_u32(v);
        }
    }
}

/// A surface you can stand on. Solid ones are blocks spanning `bottom..y` with walls and a ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Platform {
    pub left: Fx,
    pub right: Fx,
    pub y: Fx,
    /// Underside of a solid block (ceiling). Unused for pass-through platforms.
    pub bottom: Fx,
    /// Can be landed on from above and dropped through (platforms), versus solid ground.
    pub pass_through: bool,
}

/// A grabbable stage edge. `side` is -1 for the stage's left edge (the fighter hangs to its left)
/// and +1 for its right edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ledge {
    pub x: Fx,
    pub y: Fx,
    pub side: i8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    pub platforms: Vec<Platform>,
    pub ledges: Vec<Ledge>,
    pub spawns: [Vec2; MAX_FIGHTERS],
    pub blast_left: Fx,
    pub blast_right: Fx,
    pub blast_bottom: Fx,
    pub blast_top: Fx,
}

impl Stage {
    /// A flat main stage with two pass-through platforms.
    pub fn placeholder() -> Stage {
        let int = Fx::from_int;
        let spawn = |x: i32| Vec2::new(int(x), Fx::ZERO);
        Stage {
            platforms: vec![
                Platform {
                    left: int(-11),
                    right: int(11),
                    y: Fx::ZERO,
                    bottom: int(-8),
                    pass_through: false,
                },
                Platform {
                    left: int(-8),
                    right: int(-3),
                    y: Fx::from_ratio(18, 5),
                    bottom: Fx::from_ratio(18, 5),
                    pass_through: true,
                },
                Platform {
                    left: int(3),
                    right: int(8),
                    y: Fx::from_ratio(18, 5),
                    bottom: Fx::from_ratio(18, 5),
                    pass_through: true,
                },
            ],
            ledges: vec![
                Ledge {
                    x: int(-11),
                    y: Fx::ZERO,
                    side: -1,
                },
                Ledge {
                    x: int(11),
                    y: Fx::ZERO,
                    side: 1,
                },
            ],
            spawns: [spawn(-4), spawn(-1), spawn(1), spawn(4)],
            blast_left: int(-28),
            blast_right: int(28),
            blast_bottom: int(-17),
            blast_top: int(24),
        }
    }
}

impl StateHash for Stage {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(self.platforms.len() as u32);
        for p in &self.platforms {
            p.left.hash_into(h);
            p.right.hash_into(h);
            p.y.hash_into(h);
            p.bottom.hash_into(h);
            h.write_bool(p.pass_through);
        }
        h.write_u32(self.ledges.len() as u32);
        for l in &self.ledges {
            l.x.hash_into(h);
            l.y.hash_into(h);
            h.write_i8(l.side);
        }
        for s in &self.spawns {
            s.hash_into(h);
        }
        self.blast_left.hash_into(h);
        self.blast_right.hash_into(h);
        self.blast_bottom.hash_into(h);
        self.blast_top.hash_into(h);
    }
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
            f.hash_into(&mut h);
        }
        self.stage.hash_into(&mut h);
        h.finish()
    }

    /// Two placeholder fighters with different physics profiles on a main stage with platforms.
    pub fn placeholder() -> Content {
        Content {
            fighters: vec![FighterParams::duelist(), FighterParams::brawler()],
            stage: Stage::placeholder(),
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

        let mut c = base.clone();
        c.fighters[0].ledge_hang_max += 1;
        assert_ne!(h, c.hash());

        let mut c = base.clone();
        c.stage.platforms[1].pass_through = false;
        assert_ne!(h, c.hash());

        let mut c = base;
        c.fighters.pop();
        assert_ne!(h, c.hash());
    }

    #[test]
    fn placeholder_profiles_differ() {
        assert_ne!(FighterParams::duelist(), FighterParams::brawler());
    }
}
