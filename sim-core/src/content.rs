//! Immutable, hash-verified content. It is never part of the snapshot; both peers must hold
//! identical content, which the netplay handshake checks via [`Content::hash`].
//!
//! These are the *types* and the hand-written placeholder fighters. Loading and validation live in
//! `sim-content`. Units: 1.0 = one world unit; speeds are per 60 Hz frame; `*_frames` are frames.

use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::moves::Weapon;
use crate::vec2::Vec2;
use crate::{MAX_FIGHTERS, SIM_VERSION};

/// Ground speeds as a percentage of the reference values. Lower this to slow the ground game down.
pub const GROUND_SPEED_PERCENT: i32 = 90;
/// How much of a full hop's height is covered by its fast opening frames (the reference game's
/// "initial height" is about 0.55 of the full hop). The frame count is an estimate.
pub const HOP_BURST_SHARE_PERCENT: i32 = 55;
pub const HOP_BURST_FRAMES: u8 = 4;

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
    /// Friction while a dash is cancelled by releasing the stick. Stronger than `ground_friction`,
    /// so spacing and dash dancing stay tight.
    pub dash_brake: Fx,
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
    /// Upward speed after the fast opening of a full hop (see `hop_burst_frames`).
    pub full_hop_velocity: Fx,
    /// A full hop opens with `hop_burst_frames` frames at this constant speed and no gravity, which
    /// covers about half the hop's height at once (the reference game speeds up a full hop's first
    /// frames this way). Then it continues as a normal arc from `full_hop_velocity`. Zero frames
    /// disables it.
    pub hop_burst_velocity: Fx,
    pub hop_burst_frames: u8,
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
    // Defensive rolls and spot dodge (out of a shield)
    pub roll_frames: u8,
    /// Horizontal speed during `roll_move_start..=roll_move_end` (move frames).
    pub roll_speed: Fx,
    pub roll_move_start: u8,
    pub roll_move_end: u8,
    pub roll_intangible_start: u8,
    pub roll_intangible_end: u8,
    pub spot_dodge_frames: u8,
    pub spot_intangible_start: u8,
    pub spot_intangible_end: u8,
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
    /// Scales the size and position of everything this fighter's attacks do (hitboxes, projectile muzzles and sizes,
    /// reflectors). 1.0 is the moveset as written; a big fighter's attacks are bigger and reach further.
    pub hitbox_scale: Fx,
    // Special states
    pub helpless_landing_lag: u8,
    pub ledge_attack_frames: u8,
    pub ledge_attack_dx: Fx,
    /// Weight in reference units; heavier fighters are launched less.
    pub weight: Fx,
    /// Index into `Content::weapons`.
    pub weapon: u8,
}

impl FighterParams {
    /// Every fixed-point field with its name.
    pub fn fx_fields(&self) -> [(&'static str, Fx); 46] {
        [
            ("walk_speed", self.walk_speed),
            ("run_speed", self.run_speed),
            ("dash_speed", self.dash_speed),
            ("dash_initial_speed", self.dash_initial_speed),
            ("dash_accel", self.dash_accel),
            ("dash_brake", self.dash_brake),
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
            ("hop_burst_velocity", self.hop_burst_velocity),
            ("short_hop_velocity", self.short_hop_velocity),
            ("air_jump_velocity", self.air_jump_velocity),
            ("air_dodge_speed", self.air_dodge_speed),
            ("air_dodge_decay", self.air_dodge_decay),
            ("ground_assist_dist", self.ground_assist_dist),
            ("wavedash_min_down", self.wavedash_min_down),
            ("waveland_speed", self.waveland_speed),
            ("waveland_friction", self.waveland_friction),
            ("roll_speed", self.roll_speed),
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
            ("hitbox_scale", self.hitbox_scale),
            ("ledge_attack_dx", self.ledge_attack_dx),
            ("weight", self.weight),
        ]
    }

    /// Every integer (frame-count) field with its name.
    pub fn int_fields(&self) -> [(&'static str, u32); 29] {
        [
            ("jump_squat_frames", u32::from(self.jump_squat_frames)),
            ("hop_burst_frames", u32::from(self.hop_burst_frames)),
            ("air_jumps", u32::from(self.air_jumps)),
            ("landing_lag", u32::from(self.landing_lag)),
            ("air_dodge_frames", u32::from(self.air_dodge_frames)),
            ("air_dodge_buffer", u32::from(self.air_dodge_buffer)),
            ("waveland_lag", u32::from(self.waveland_lag)),
            ("shield_drop_buffer", u32::from(self.shield_drop_buffer)),
            ("shield_drop_recovery", u32::from(self.shield_drop_recovery)),
            ("roll_frames", u32::from(self.roll_frames)),
            ("roll_move_start", u32::from(self.roll_move_start)),
            ("roll_move_end", u32::from(self.roll_move_end)),
            (
                "roll_intangible_start",
                u32::from(self.roll_intangible_start),
            ),
            ("roll_intangible_end", u32::from(self.roll_intangible_end)),
            ("spot_dodge_frames", u32::from(self.spot_dodge_frames)),
            (
                "spot_intangible_start",
                u32::from(self.spot_intangible_start),
            ),
            ("spot_intangible_end", u32::from(self.spot_intangible_end)),
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
            ("weapon", u32::from(self.weapon)),
        ]
    }

    /// Converts a Smash-Ultimate-style number (given in thousandths of a unit) into world units.
    /// One world unit is 8 of those units, so a ~17 unit tall fighter is about 2.2 world units tall.
    pub const fn su(thousandths: i32) -> Fx {
        Fx::from_ratio(thousandths, 8000)
    }

    /// Like `su`, but for ground movement (speeds, accelerations, friction), scaled by
    /// [`GROUND_SPEED_PERCENT`] so the ground game can be tuned without touching air movement or jumps.
    /// Like `su`, but in hundred-thousandths, for the very small values (air friction 0.00375).
    pub const fn su_fine(hundred_thousandths: i32) -> Fx {
        Fx::from_ratio(hundred_thousandths, 800_000)
    }

    /// Like `su`, but for ground movement (speeds, accelerations, friction), scaled by
    /// [`GROUND_SPEED_PERCENT`] so the ground game can be tuned without touching air movement or jumps.
    pub const fn gu(thousandths: i32) -> Fx {
        Fx::from_ratio(thousandths * GROUND_SPEED_PERCENT, 800_000)
    }

    /// Initial upward velocity that makes a jump peak `height` above where it started.
    ///
    /// Gravity is applied before moving each frame, so the peak is `v^2/(2g) - v/2`,
    /// which solves to `v = (g + sqrt(g^2 + 8gH)) / 2`.
    pub fn hop_velocity(gravity: Fx, height: Fx) -> Fx {
        (gravity + (gravity * gravity + (gravity * height).mul_int(8)).sqrt()) * Fx::HALF
    }

    /// Full hop split into its fast opening and the arc after it: returns
    /// `(burst_velocity, arc_velocity)` so the whole hop peaks `height` above where it started.
    /// The opening covers [`HOP_BURST_SHARE_PERCENT`] of the height in [`HOP_BURST_FRAMES`] frames.
    pub fn full_hop(gravity: Fx, height: Fx) -> (Fx, Fx) {
        let burst_height = height * Fx::from_ratio(HOP_BURST_SHARE_PERCENT, 100);
        let burst = burst_height / Fx::from_int(i32::from(HOP_BURST_FRAMES));
        let arc = Self::hop_velocity(gravity, height - burst_height);
        (burst, arc)
    }

    /// Values shared by every fighter: tech that is not character specific (air dodge, wavedash,
    /// shield drop, ledges). Character profiles below override the movement numbers.
    fn base() -> FighterParams {
        let r = Fx::from_ratio;
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        FighterParams {
            walk_speed: gu(1400),
            run_speed: gu(1800),
            dash_speed: gu(2200),
            dash_initial_speed: gu(900),
            dash_accel: gu(300),
            dash_brake: gu(520),
            dash_frames: 12,
            turn_frames: 6,
            ground_accel: gu(200),
            ground_friction: gu(220),
            run_decel: gu(60),
            landing_friction: gu(40),
            air_speed: su(1100),
            air_accel: su(10),
            air_accel_stick: su(70),
            air_friction: FighterParams::su_fine(375),
            gravity: su(100),
            max_fall_speed: su(1700),
            fast_fall_speed: su(2700),
            jump_squat_frames: 3,
            full_hop_velocity: r(1, 4),
            hop_burst_velocity: Fx::ZERO,
            hop_burst_frames: 0,
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
            // Estimates: a roll is about 31 frames covering roughly 2.8 world units, intangible on 4-19; a spot
            // dodge is about 25 frames, intangible on 3-20.
            roll_frames: 31,
            roll_speed: r(7, 50),
            roll_move_start: 5,
            roll_move_end: 24,
            roll_intangible_start: 4,
            roll_intangible_end: 19,
            spot_dodge_frames: 25,
            spot_intangible_start: 3,
            spot_intangible_end: 20,
            shield_drop_speed: r(3, 50),
            platform_ignore_frames: 8,
            ledge_reach_x: r(11, 5),
            ledge_min_drop: r(9, 10),
            ledge_reach_down: r(13, 5),
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
            hitbox_scale: Fx::ONE,
            helpless_landing_lag: 20,
            ledge_attack_frames: 54,
            ledge_attack_dx: r(3, 2),
            weight: Fx::from_int(90),
            weapon: 0,
        }
    }

    /// Fast on the ground, light and floaty in the air: the "longsword duelist" body type.
    /// Movement numbers follow a swordfighter-style profile from the reference game, in world units.
    /// Marth-style reference: walk 1.575, run 1.964, dash 2.255, air speed 1.071, gravity 0.075,
    /// fall 1.58 (fast 2.528), full hop 33.66, short hop 16.26, double jump 33.66.
    pub fn duelist() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(75);
        let (burst, arc) = Self::full_hop(gravity, su(33660));
        FighterParams {
            walk_speed: gu(1575),
            run_speed: gu(1964),
            dash_speed: gu(2255),
            air_speed: su(1071),
            air_friction: FighterParams::su_fine(375),
            gravity,
            max_fall_speed: su(1580),
            fast_fall_speed: su(2528),
            ground_friction: gu(228),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
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
        let gu = FighterParams::gu;
        let gravity = su(130);
        let (burst, arc) = Self::full_hop(gravity, su(32020));
        FighterParams {
            walk_speed: gu(1208),
            run_speed: gu(1540),
            dash_speed: gu(2090),
            dash_initial_speed: gu(840),
            air_speed: su(1281),
            air_accel_stick: su(80),
            air_friction: su(10),
            gravity,
            max_fall_speed: su(1800),
            fast_fall_speed: su(2880),
            ground_friction: gu(220),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(15380)),
            air_jump_velocity: Self::hop_velocity(gravity, su(30710)),
            weight: Fx::from_int(92),
            weapon: 1,
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

/// Global combat rules, hashed with the content so both peers must agree on them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ruleset {
    /// Multiplies all damage dealt. 1.2 matches the reference game's default one-on-one rules.
    pub damage_mult: Fx,
    /// Scales every hitstun duration. Slightly above 1.0 gives combos a little more room (plan 4.3).
    pub hitstun_mult: Fx,
    /// Launch speed lost per frame (world units per frame squared).
    pub knockback_decay: Fx,
    /// Launch knockback at or above which a hit is a tumble (reference knockback units).
    pub tumble_knockback: Fx,
    /// Distance moved by one stick flick of survival DI during hitlag.
    pub sdi_distance: Fx,
    /// How far the launch angle can be bent by DI, in degrees.
    pub di_degrees: u8,
    pub respawn_invuln: u8,
    /// A shield press this many frames before landing in hitstun is a tech.
    pub tech_window: u8,
    pub tech_lag: u8,
    /// A fighter that lands in hitstun without teching lies down for at least this many frames.
    pub knockdown_lag: u8,
    /// ...and gets up by itself after this many.
    pub knockdown_max: u8,
    /// Frames of a neutral get-up, and how many of them are intangible.
    pub getup_frames: u8,
    pub getup_intangible: u8,
    /// Intangible frames of a tech in place.
    pub tech_invuln: u8,
    /// Frames a smash attack can be held charging.
    pub charge_frames: u8,
    /// Extra damage at full charge, in percent (40 means 1.4 times the damage).
    pub charge_bonus_percent: u8,
    // ---- Shields ----
    /// Full shield health.
    pub shield_max: Fx,
    /// Health lost per frame while the shield is up.
    pub shield_deplete: Fx,
    /// Health regained per frame while it is not.
    pub shield_regen: Fx,
    /// Share of shield health a broken shield comes back with, in percent.
    pub shield_restore_percent: u8,
    /// Stun after a shield break: `shield_break_frames - shield_break_per_percent * percent`, at least
    /// `shield_break_min`.
    pub shield_break_frames: u16,
    pub shield_break_per_percent: u8,
    pub shield_break_min: u16,
    /// Upward speed of the hop a shield break launches the fighter into.
    pub shield_break_hop: Fx,
    /// Frames a button press takes off a shield break stun (mashing out).
    pub shield_mash_frames: u8,
    /// No block can stun for longer than this.
    pub shield_stun_cap: u8,
    /// A hit landing within this many frames of the shield going up is a perfect shield.
    pub perfect_shield_window: u8,
    /// Frames a perfect shield takes off the shield stun.
    pub perfect_shield_stun_cut: u8,
    // ---- Grabs ----
    /// A held fighter breaks free after `grab_base_frames + grab_percent_tenths / 10 * percent` frames (at least
    /// `grab_min_frames`).
    pub grab_base_frames: u16,
    pub grab_percent_tenths: u16,
    pub grab_min_frames: u16,
    /// Frames a button press or a stick flick takes off the time held.
    pub grab_mash_button: u8,
    pub grab_mash_stick: u8,
    /// Lag for the holder when the held fighter breaks free.
    pub grab_release_lag: u8,
    /// Frames the released fighter cannot be grabbed again.
    pub grab_immunity: u8,
    /// How far in front of the holder the held fighter stands.
    pub grab_distance: Fx,
}

impl Ruleset {
    pub fn standard() -> Ruleset {
        Ruleset {
            damage_mult: Fx::from_ratio(12, 10),
            hitstun_mult: Fx::from_ratio(105, 100),
            knockback_decay: Fx::from_ratio(51, 8000),
            tumble_knockback: Fx::from_int(80),
            sdi_distance: Fx::from_ratio(3, 4),
            di_degrees: 18,
            respawn_invuln: 120,
            tech_window: 5,
            tech_lag: 4,
            knockdown_lag: 24,
            knockdown_max: 90,
            getup_frames: 26,
            getup_intangible: 15,
            tech_invuln: 18,
            charge_frames: 60,
            charge_bonus_percent: 40,
            shield_max: Fx::from_int(50),
            shield_deplete: Fx::from_ratio(15, 100),
            shield_regen: Fx::from_ratio(8, 100),
            shield_restore_percent: 75,
            shield_break_frames: 400,
            shield_break_per_percent: 1,
            shield_break_min: 120,
            shield_break_hop: Fx::from_ratio(28, 100),
            shield_mash_frames: 4,
            shield_stun_cap: 60,
            perfect_shield_window: 5,
            perfect_shield_stun_cut: 3,
            grab_base_frames: 90,
            grab_percent_tenths: 17,
            grab_min_frames: 19,
            grab_mash_button: 14,
            grab_mash_stick: 8,
            grab_release_lag: 25,
            grab_immunity: 60,
            grab_distance: Fx::from_ratio(13, 10),
        }
    }
}

impl StateHash for Ruleset {
    fn hash_into(&self, h: &mut StateHasher) {
        self.damage_mult.hash_into(h);
        self.hitstun_mult.hash_into(h);
        self.knockback_decay.hash_into(h);
        self.tumble_knockback.hash_into(h);
        self.sdi_distance.hash_into(h);
        h.write_u8(self.di_degrees);
        h.write_u8(self.respawn_invuln);
        h.write_u8(self.tech_window);
        h.write_u8(self.tech_lag);
        h.write_u8(self.knockdown_lag);
        h.write_u8(self.knockdown_max);
        h.write_u8(self.getup_frames);
        h.write_u8(self.getup_intangible);
        h.write_u8(self.tech_invuln);
        h.write_u8(self.charge_frames);
        h.write_u8(self.charge_bonus_percent);
        self.shield_max.hash_into(h);
        self.shield_deplete.hash_into(h);
        self.shield_regen.hash_into(h);
        h.write_u8(self.shield_restore_percent);
        h.write_u16(self.shield_break_frames);
        h.write_u8(self.shield_break_per_percent);
        h.write_u16(self.shield_break_min);
        self.shield_break_hop.hash_into(h);
        h.write_u8(self.shield_mash_frames);
        h.write_u8(self.shield_stun_cap);
        h.write_u8(self.perfect_shield_window);
        h.write_u8(self.perfect_shield_stun_cut);
        h.write_u16(self.grab_base_frames);
        h.write_u16(self.grab_percent_tenths);
        h.write_u16(self.grab_min_frames);
        h.write_u8(self.grab_mash_button);
        h.write_u8(self.grab_mash_stick);
        h.write_u8(self.grab_release_lag);
        h.write_u8(self.grab_immunity);
        self.grab_distance.hash_into(h);
    }
}

/// Human-readable names for the things in a [`Content`]. Content files refer to fighters and weapons by name;
/// the simulation only ever uses indices, so names are not part of the content hash.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Names {
    pub fighters: Vec<String>,
    pub weapons: Vec<String>,
    pub stage: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub fighters: Vec<FighterParams>,
    pub weapons: Vec<Weapon>,
    pub stage: Stage,
    pub rules: Ruleset,
    pub names: Names,
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
        h.write_u32(self.weapons.len() as u32);
        for w in &self.weapons {
            w.hash_into(&mut h);
        }
        self.stage.hash_into(&mut h);
        self.rules.hash_into(&mut h);
        h.finish()
    }

    /// Two placeholder fighters (a swordfighter and a close-range brawler) on a main stage with platforms.
    pub fn placeholder() -> Content {
        Content {
            fighters: vec![FighterParams::duelist(), FighterParams::brawler()],
            weapons: vec![crate::moves::longsword(), crate::moves::claws()],
            stage: Stage::placeholder(),
            rules: Ruleset::standard(),
            names: Names {
                fighters: vec!["duelist".to_string(), "brawler".to_string()],
                weapons: vec!["longsword".to_string(), "claws".to_string()],
                stage: "proving_grounds".to_string(),
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
