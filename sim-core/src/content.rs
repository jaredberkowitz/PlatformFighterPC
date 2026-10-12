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
pub const GROUND_SPEED_PERCENT: i32 = 100;
/// How much of a full hop's height is covered by its fast opening frames (the reference game's
/// "initial height" is about 0.55 of the full hop). The frame count is an estimate.
pub const HOP_BURST_SHARE_PERCENT: i32 = 55;
pub const HOP_BURST_FRAMES: u8 = 4;
/// How far below a ledge a fighter hangs (its feet), the start of a ledge jump.
pub const LEDGE_HANG_DY: Fx = Fx::from_ratio(9, 5);

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
    /// Braking on the frame an initial dash ends with the stick let go. Stronger than `ground_friction`,
    /// so spacing and dash dancing stay tight.
    pub dash_brake: Fx,
    /// The initial dash: this many frames at dash speed, then a run (if the stick is still held). It is committed: a tap covers the
    /// whole distance, and only a dash the other way, a jump, a grab, a special, a dash attack or a smash attack cuts it short.
    pub dash_frames: u8,
    /// The initial dash can be cut short by the shield from this frame on (about half of `dash_frames`); a shield held earlier comes
    /// out then.
    pub dash_shield_frame: u8,
    /// A flick the other way this many frames after a dash started reverses it (a dash dance), even once the run has begun.
    pub dash_reverse_frames: u8,
    /// A reversed dash stands still this many frames before it starts accelerating (the turnaround).
    pub dash_turn_delay: u8,
    pub turn_frames: u8,
    /// How long a run's turnaround lasts (it skids to a stop facing the new way): longer than a turn from standing, as in the
    /// reference game, where the run's turnaround is a visible slide. A jump in it is a reverse aerial rush, so this is that window.
    pub run_turn_frames: u8,
    /// Walking acceleration a frame: `walk_accel` plus `walk_accel_stick` scaled by how far the stick is pushed (the reference
    /// game's `walk_accel_add` and `walk_accel_mul`).
    pub walk_accel: Fx,
    pub walk_accel_stick: Fx,
    /// Running acceleration a frame, the same way (the reference's `run_accel_add` and `run_accel_mul`).
    pub run_accel: Fx,
    pub run_accel_stick: Fx,
    pub ground_friction: Fx,
    /// How fast speed above `run_speed` (left over from a dash) bleeds off while running.
    pub run_decel: Fx,
    /// Friction during a landing's lag, whatever the stick does.
    pub landing_friction: Fx,
    /// Friction while a dash attack slides: the reference game's traction (`ground_friction` is twice it, as traction is
    /// above walking speed).
    pub dash_attack_friction: Fx,
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
    /// Lag of a light landing (from a jump, without an attack).
    pub landing_lag: u8,
    /// Lag of a heavy landing: touching down while falling at full speed (fast falling, or at `max_fall_speed`).
    pub heavy_landing_lag: u8,
    /// Frames before an action becomes possible that a button press for it still counts (the reference game's 9). A press still held when the
    /// action becomes possible also counts (the hold buffer), for every button but shield. Each press starts one action only.
    pub input_buffer: u8,
    // Air dodge and wavedash
    pub air_dodge_frames: u8,
    pub air_dodge_speed: Fx,
    /// Per-frame velocity multiplier during an air dodge (0..=1).
    pub air_dodge_decay: Fx,
    /// A directional air dodge first drifts the opposite way (the slingshot) for this many frames, then goes full speed.
    pub air_dodge_windup: u8,
    /// The slingshot's speed as a fraction of `air_dodge_speed`.
    pub air_dodge_sling: Fx,
    /// Lag when an air dodge that is not a wavedash lands.
    pub air_dodge_landing_lag: u8,
    /// A shield press this many frames ago still starts the air dodge on the first airborne frame.
    pub air_dodge_buffer: u8,
    /// How long a directional air dodge lasts when aimed straight down, sideways and straight up (in between, by the angle). A
    /// neutral one lasts `air_dodge_frames`. The fighter cannot act until it is over.
    pub air_dodge_dir_down_frames: u8,
    pub air_dodge_dir_side_frames: u8,
    pub air_dodge_dir_up_frames: u8,
    /// Intangible frames of a neutral air dodge (`start..=end`); a directional one ends at `air_dodge_dir_intangible_end`.
    pub air_dodge_intangible_start: u8,
    pub air_dodge_intangible_end: u8,
    pub air_dodge_dir_intangible_end: u8,
    /// From this frame a directional air dodge stops carrying the fighter and it falls (gravity, no drift) until the dodge ends.
    pub air_dodge_fall_frame: u8,
    /// An air dodge can grab a ledge from this frame on.
    pub air_dodge_ledge_frame: u8,
    /// Landing lag of a directional air dodge (and a wavedash): `waveland_lag` when it lands as the slingshot ends, one frame less
    /// for every `air_dodge_landing_step` frames later, down to `air_dodge_dir_landing_min`.
    pub air_dodge_dir_landing_min: u8,
    pub air_dodge_landing_step: u8,
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
    /// Letting go of the shield takes this many frames before the fighter can act (actions straight out of the shield skip it).
    pub shield_release_frames: u8,
    // Defensive rolls and spot dodge (out of a shield)
    /// A forward roll's length; a backward roll (away from the way the fighter faces) takes `roll_back_frames`.
    pub roll_frames: u8,
    pub roll_back_frames: u8,
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
    /// Intangibility on the first ledge grab since landing or being hit: `ledge_grab_frames` plus the larger of `ledge_invuln_min`
    /// and `ledge_invuln_airtime * airtime / 300 + ledge_invuln_damage * (1 - percent / 120)` (airtime in frames, capped at 300;
    /// percent capped at 120). A regrab without landing or being hit gets none.
    pub ledge_invuln_airtime: u8,
    pub ledge_invuln_damage: u8,
    pub ledge_invuln_min: u8,
    /// The grab itself: no ledge option is possible for this many frames.
    pub ledge_grab_frames: u8,
    /// Ledge grabs allowed between landings (or hits). After that a ledge cannot be caught.
    pub ledge_grab_limit: u8,
    /// Reach for a ledge behind the fighter (its back to the stage); shorter than `ledge_reach_x` in front.
    pub ledge_reach_back_x: Fx,
    /// Getting up: total frames, and intangible frames from its start.
    pub ledge_getup_frames: u8,
    pub ledge_getup_intangible: u8,
    pub ledge_roll_frames: u8,
    pub ledge_roll_intangible: u8,
    /// A ledge jump cannot act for this many frames, and is intangible for its first `ledge_jump_intangible`.
    pub ledge_jump_frames: u8,
    pub ledge_jump_intangible: u8,
    /// Percent of the ledge options' intangibility on the second and the third grab without landing; from the fourth there is none.
    pub ledge_option_decay_2: u8,
    pub ledge_option_decay_3: u8,
    pub ledge_getup_dx: Fx,
    pub ledge_roll_dx: Fx,
    /// The ledge jump's launch speed. Set (`ledge_jump_velocity_for`) so that, like the reference game's, it peaks a given height above
    /// the stage (its `cliff_jump_y`, a little higher than a full hop): the climb from the hang is part of the jump.
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
    /// Crouching, the hurtboxes come down to this share of their standing heights, so high attacks pass over a crouch.
    pub crouch_height: Fx,
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
    /// How far a whole initial dash from a standstill goes over the frames it is shown as a dash (`dash_frames`, before it becomes a
    /// run or brakes). The view times the dash's stride by it; a test checks the simulation agrees.
    pub fn initial_dash_distance(&self) -> Fx {
        let mut v = self.dash_initial_speed;
        let mut d = Fx::ZERO;
        for _ in 0..self.dash_frames {
            v = if v < self.dash_speed {
                (v + self.dash_accel).min(self.dash_speed)
            } else {
                (v - self.dash_accel).max(self.dash_speed)
            };
            d += v;
        }
        d
    }

    /// Every fixed-point field with its name.
    pub fn fx_fields(&self) -> [(&'static str, Fx); 53] {
        [
            ("walk_speed", self.walk_speed),
            ("run_speed", self.run_speed),
            ("dash_speed", self.dash_speed),
            ("dash_initial_speed", self.dash_initial_speed),
            ("dash_accel", self.dash_accel),
            ("dash_brake", self.dash_brake),
            ("walk_accel", self.walk_accel),
            ("walk_accel_stick", self.walk_accel_stick),
            ("run_accel", self.run_accel),
            ("run_accel_stick", self.run_accel_stick),
            ("ground_friction", self.ground_friction),
            ("run_decel", self.run_decel),
            ("landing_friction", self.landing_friction),
            ("dash_attack_friction", self.dash_attack_friction),
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
            ("air_dodge_sling", self.air_dodge_sling),
            ("ground_assist_dist", self.ground_assist_dist),
            ("wavedash_min_down", self.wavedash_min_down),
            ("waveland_speed", self.waveland_speed),
            ("waveland_friction", self.waveland_friction),
            ("roll_speed", self.roll_speed),
            ("shield_drop_speed", self.shield_drop_speed),
            ("ledge_reach_x", self.ledge_reach_x),
            ("ledge_min_drop", self.ledge_min_drop),
            ("ledge_reach_down", self.ledge_reach_down),
            ("ledge_reach_back_x", self.ledge_reach_back_x),
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
            ("crouch_height", self.crouch_height),
            ("ledge_attack_dx", self.ledge_attack_dx),
            ("weight", self.weight),
        ]
    }

    /// Every integer (frame-count) field with its name.
    pub fn int_fields(&self) -> [(&'static str, u32); 58] {
        [
            ("jump_squat_frames", u32::from(self.jump_squat_frames)),
            ("dash_reverse_frames", u32::from(self.dash_reverse_frames)),
            ("dash_turn_delay", u32::from(self.dash_turn_delay)),
            ("dash_shield_frame", u32::from(self.dash_shield_frame)),
            ("air_dodge_windup", u32::from(self.air_dodge_windup)),
            (
                "air_dodge_landing_lag",
                u32::from(self.air_dodge_landing_lag),
            ),
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
            ("ledge_invuln_airtime", u32::from(self.ledge_invuln_airtime)),
            ("ledge_invuln_damage", u32::from(self.ledge_invuln_damage)),
            ("ledge_invuln_min", u32::from(self.ledge_invuln_min)),
            ("ledge_grab_frames", u32::from(self.ledge_grab_frames)),
            ("ledge_grab_limit", u32::from(self.ledge_grab_limit)),
            ("ledge_getup_frames", u32::from(self.ledge_getup_frames)),
            (
                "ledge_getup_intangible",
                u32::from(self.ledge_getup_intangible),
            ),
            ("ledge_roll_frames", u32::from(self.ledge_roll_frames)),
            (
                "ledge_roll_intangible",
                u32::from(self.ledge_roll_intangible),
            ),
            ("ledge_jump_frames", u32::from(self.ledge_jump_frames)),
            (
                "ledge_jump_intangible",
                u32::from(self.ledge_jump_intangible),
            ),
            ("ledge_option_decay_2", u32::from(self.ledge_option_decay_2)),
            ("ledge_option_decay_3", u32::from(self.ledge_option_decay_3)),
            ("heavy_landing_lag", u32::from(self.heavy_landing_lag)),
            ("input_buffer", u32::from(self.input_buffer)),
            (
                "air_dodge_dir_down_frames",
                u32::from(self.air_dodge_dir_down_frames),
            ),
            (
                "air_dodge_dir_side_frames",
                u32::from(self.air_dodge_dir_side_frames),
            ),
            (
                "air_dodge_dir_up_frames",
                u32::from(self.air_dodge_dir_up_frames),
            ),
            (
                "air_dodge_intangible_start",
                u32::from(self.air_dodge_intangible_start),
            ),
            (
                "air_dodge_intangible_end",
                u32::from(self.air_dodge_intangible_end),
            ),
            (
                "air_dodge_dir_intangible_end",
                u32::from(self.air_dodge_dir_intangible_end),
            ),
            ("air_dodge_fall_frame", u32::from(self.air_dodge_fall_frame)),
            (
                "air_dodge_ledge_frame",
                u32::from(self.air_dodge_ledge_frame),
            ),
            (
                "air_dodge_dir_landing_min",
                u32::from(self.air_dodge_dir_landing_min),
            ),
            (
                "air_dodge_landing_step",
                u32::from(self.air_dodge_landing_step),
            ),
            (
                "shield_release_frames",
                u32::from(self.shield_release_frames),
            ),
            ("roll_back_frames", u32::from(self.roll_back_frames)),
            ("dash_frames", u32::from(self.dash_frames)),
            ("turn_frames", u32::from(self.turn_frames)),
            ("run_turn_frames", u32::from(self.run_turn_frames)),
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

    /// Like `gu`, in hundred-thousandths, for ground values with more digits (run acceleration 0.10978).
    pub const fn gu_fine(hundred_thousandths: i32) -> Fx {
        Fx::from_ratio(hundred_thousandths * GROUND_SPEED_PERCENT, 80_000_000)
    }

    /// Initial upward velocity that makes a jump peak `height` above where it started.
    ///
    /// Gravity is applied before moving each frame, so the peak is `v^2/(2g) - v/2`,
    /// which solves to `v = (g + sqrt(g^2 + 8gH)) / 2`.
    pub fn hop_velocity(gravity: Fx, height: Fx) -> Fx {
        (gravity + (gravity * gravity + (gravity * height).mul_int(8)).sqrt()) * Fx::HALF
    }

    /// The ledge jump's launch speed for a jump peaking `height` above the stage, from a hang `LEDGE_HANG_DY` below it.
    pub fn ledge_jump_velocity_for(gravity: Fx, height: Fx) -> Fx {
        Self::hop_velocity(gravity, height + LEDGE_HANG_DY)
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
            dash_frames: 10,
            dash_reverse_frames: 15,
            dash_turn_delay: 2,
            dash_shield_frame: 5,
            turn_frames: 6,
            run_turn_frames: 12,
            walk_accel: Fx::ZERO,
            walk_accel_stick: gu(189),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(9020),
            ground_friction: gu(220),
            run_decel: gu(60),
            landing_friction: gu(40),
            dash_attack_friction: gu(110),
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
            // Light and heavy landings (the reference game: 2 frames light, heavy ones 2 to 6 by character).
            landing_lag: 2,
            heavy_landing_lag: 4,
            input_buffer: 9,
            // Air dodge numbers for a sword-fighter-style body from the reference game's frame data: a neutral dodge lasts 52
            // frames, intangible on 3-29; a directional one 69 (down) to 85 (sideways) to 116 (up), intangible on 3-21, landing
            // lag 11 to 19 (more the sooner it lands), ledges catchable after frame 24.
            air_dodge_frames: 52,
            air_dodge_dir_down_frames: 69,
            air_dodge_dir_side_frames: 85,
            air_dodge_dir_up_frames: 116,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 29,
            air_dodge_dir_intangible_end: 21,
            air_dodge_fall_frame: 20,
            air_dodge_ledge_frame: 24,
            air_dodge_dir_landing_min: 11,
            air_dodge_landing_step: 4,
            air_dodge_speed: r(2, 5),
            air_dodge_decay: r(9, 10),
            air_dodge_windup: 5,
            air_dodge_sling: r(1, 4),
            air_dodge_landing_lag: 10,
            air_dodge_buffer: 6,
            ground_assist_dist: r(2, 5),
            wavedash_min_down: r(1, 5),
            waveland_speed: r(7, 20),
            waveland_friction: r(9, 10),
            waveland_lag: 19,
            shield_drop_buffer: 4,
            shield_drop_recovery: 6,
            shield_release_frames: 11,
            // The reference game: a forward roll is 29 frames (intangible 4-15), a backward one 34; a spot dodge 25 (intangible
            // 3-17). The distance (about 2.8 world units) is an estimate.
            roll_frames: 29,
            roll_back_frames: 34,
            roll_speed: r(7, 50),
            roll_move_start: 5,
            roll_move_end: 24,
            roll_intangible_start: 4,
            roll_intangible_end: 15,
            spot_dodge_frames: 25,
            spot_intangible_start: 3,
            spot_intangible_end: 17,
            shield_drop_speed: r(3, 50),
            platform_ignore_frames: 8,
            ledge_reach_x: r(11, 5),
            ledge_min_drop: r(9, 10),
            ledge_reach_down: r(13, 5),
            ledge_hang_dx: r(4, 5),
            ledge_hang_dy: LEDGE_HANG_DY,
            // Ledges follow the reference game: 6.5 seconds of hanging, a 19-frame grab, intangibility from airtime and damage on the
            // first grab only, six grabs between landings, a 40% shorter reach behind, and getup options of 34 frames (intangible
            // 1-33), a 45-frame roll (1-26) and a jump that can act on frame 15 (1-12), their intangibility cut to 80% and 50% on the
            // second and third grab and gone after.
            ledge_hang_max: 390,
            ledge_regrab_cooldown: 30,
            ledge_invuln_airtime: 60,
            ledge_invuln_damage: 44,
            ledge_invuln_min: 4,
            ledge_grab_frames: 19,
            ledge_grab_limit: 6,
            ledge_reach_back_x: r(33, 25),
            ledge_getup_frames: 34,
            ledge_getup_intangible: 33,
            ledge_roll_frames: 45,
            ledge_roll_intangible: 26,
            ledge_jump_frames: 14,
            ledge_jump_intangible: 12,
            ledge_option_decay_2: 80,
            ledge_option_decay_3: 50,
            ledge_getup_dx: Fx::ONE,
            ledge_roll_dx: Fx::from_int(3),
            // The reference game: a ledge jump peaks about 36 units above the stage (12% over a full hop) and drifts at 0.6.
            ledge_jump_velocity: Self::ledge_jump_velocity_for(su(100), su(36000)),
            ledge_jump_dx: su(600),
            ledge_trump_vx: r(1, 8),
            ledge_trump_vy: r(1, 15),
            ecb_half_width: r(4, 5),
            ecb_height: r(11, 5),
            ecb_side_height: r(11, 10),
            hitbox_scale: Fx::ONE,
            crouch_height: r(3, 5),
            helpless_landing_lag: 20,
            ledge_attack_frames: 55,
            ledge_attack_dx: r(3, 2),
            weight: Fx::from_int(90),
            weapon: 0,
        }
    }

    /// Fast on the ground, light and floaty in the air: the "longsword duelist" body type.
    /// Movement numbers follow a swordfighter-style profile from the reference game, in world units.
    /// Marth-style reference: walk 1.575, run 1.964, dash 2.255, air speed 1.071, gravity 0.075,
    /// fall 1.58 (fast 2.528), full hop 33.66, short hop 16.26, double jump 33.66, ledge jump 36.83.
    pub fn duelist() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(75);
        // Ground: walk acceleration 0 + 0.189 (by the stick), run acceleration 0.044 + 0.0902, an initial dash of 10 frames, a heavy
        // landing of 4 frames counted from the touchdown (3 of lag).
        let (burst, arc) = Self::full_hop(gravity, su(33660));
        FighterParams {
            walk_speed: gu(1575),
            run_speed: gu(1964),
            dash_speed: gu(2255),
            walk_accel: Fx::ZERO,
            walk_accel_stick: gu(189),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(9020),
            dash_frames: 10,
            heavy_landing_lag: 3,
            air_speed: su(1071),
            air_friction: FighterParams::su_fine(375),
            gravity,
            max_fall_speed: su(1580),
            fast_fall_speed: su(2528),
            ground_friction: gu(228),
            dash_attack_friction: gu(114),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(16260)),
            air_jump_velocity: Self::hop_velocity(gravity, su(33660)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(36830)),
            ..FighterParams::base()
        }
    }

    /// Heavier, faster in the air, with a high gravity and fast fall: the "blaster brawler" body type.
    /// Wolf-style reference: walk 1.208, run 1.54, dash 2.09, air speed 1.281, gravity 0.13,
    /// fall 1.8 (fast 2.88), full hop 32.02, short hop 15.38, double jump 30.71, ledge jump 36.01; on the ground, traction 0.11, walk
    /// acceleration 0.105 + 0.105, run acceleration 0.044 + 0.10978, an initial dash of 10 frames, a heavy landing of 4 (3 of lag).
    pub fn brawler() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(130);
        let (burst, arc) = Self::full_hop(gravity, su(32020));
        FighterParams {
            walk_speed: gu(1208),
            run_speed: gu(1540),
            dash_speed: gu(2090),
            walk_accel: gu(105),
            walk_accel_stick: gu(105),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(10978),
            dash_frames: 10,
            heavy_landing_lag: 3,
            dash_initial_speed: gu(840),
            air_speed: su(1281),
            air_accel_stick: su(80),
            air_friction: su(10),
            gravity,
            max_fall_speed: su(1800),
            fast_fall_speed: su(2880),
            // Wolf-style air dodges: neutral 44 frames (intangible 2-26), directional 61 / 73 / 93 (2-20).
            air_dodge_frames: 44,
            air_dodge_dir_down_frames: 61,
            air_dodge_dir_side_frames: 73,
            air_dodge_dir_up_frames: 93,
            air_dodge_intangible_start: 2,
            air_dodge_intangible_end: 26,
            air_dodge_dir_intangible_end: 20,
            ground_friction: gu(220),
            dash_attack_friction: gu(110),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(15380)),
            air_jump_velocity: Self::hop_velocity(gravity, su(30710)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(36010)),
            weight: Fx::from_int(92),
            weapon: 1,
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// Light, fast on the ground, a high gravity and fast fall: the "rogue" body type of the Needles moveset. The reference game's
    /// Sheik attributes (SmashWiki / ultimateframedata.com, `docs/sheik/REFERENCE_DATA.md`): weight 78, walk 1.47, initial dash 2.178,
    /// run 2.42, traction 0.106, air speed 1.155, air acceleration 0.01 + 0.08, air friction 0.015, gravity 0.15, fall 1.75 (fast
    /// 2.8), full hop 39, short hop 18.75, double jump 40; air dodges 44 (neutral, intangible 2-26) and 63 / 74 / 93 (directional,
    /// intangible 2-19). On the ground: walk acceleration 0.105 + 0.21, run acceleration 0.044 + 0.121, an initial dash of 7 frames
    /// (the others' 10), a heavy landing of 7 (6 of lag) (the parameter file, via dragdown.wiki and SmashWiki's dash table). The
    /// ledge jump height is not published: the duelist's (an estimate).
    pub fn rogue() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(150);
        let (burst, arc) = Self::full_hop(gravity, su(39000));
        FighterParams {
            walk_speed: gu(1470),
            run_speed: gu(2420),
            dash_speed: gu(2178),
            walk_accel: gu(105),
            walk_accel_stick: gu(210),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(12100),
            dash_frames: 7,
            // (The shield can cut a dash short from about half way, as for the others.)
            dash_shield_frame: 4,
            heavy_landing_lag: 6,
            air_speed: su(1155),
            air_accel: su(10),
            air_accel_stick: su(80),
            air_friction: su(15),
            gravity,
            max_fall_speed: su(1750),
            fast_fall_speed: su(2800),
            air_dodge_frames: 44,
            air_dodge_dir_down_frames: 63,
            air_dodge_dir_side_frames: 74,
            air_dodge_dir_up_frames: 93,
            air_dodge_intangible_start: 2,
            air_dodge_intangible_end: 26,
            air_dodge_dir_intangible_end: 19,
            // Traction 0.106 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(212),
            dash_attack_friction: gu(106),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(18750)),
            air_jump_velocity: Self::hop_velocity(gravity, su(40000)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(36830)),
            weight: Fx::from_int(78),
            weapon: 2,
            // Dodges (the reference's): a forward roll of 26 frames, a backward one of 32, intangible on 4-12 (the backward roll's
            // 4-14 is not separate here [D]); a spot dodge of 23, intangible on 3-14. The roll moves on frames 5-21 [E].
            roll_frames: 26,
            roll_back_frames: 32,
            roll_move_end: 21,
            roll_intangible_start: 4,
            roll_intangible_end: 12,
            spot_dodge_frames: 23,
            spot_intangible_start: 3,
            spot_intangible_end: 14,
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
    /// Scales every hitstun duration (1.0 is the reference game: hitstun is `floor(knockback * 0.4 * this) - 1`).
    pub hitstun_mult: Fx,
    /// Launch speed lost per frame (world units per frame squared).
    pub knockback_decay: Fx,
    /// Launch knockback at or above which a hit is a tumble (reference knockback units).
    pub tumble_knockback: Fx,
    /// Distance moved by one stick flick of survival DI during hitlag.
    pub sdi_distance: Fx,
    /// Damage of an aerial made during a short hop, as a fraction of its listed damage (the reference game's 0.85).
    pub short_hop_damage: Fx,
    /// How far the launch angle can be bent by DI, in degrees (the reference game's 0.17 radians, about 10).
    pub di_degrees: u8,
    /// Launch speed influence: a tumbling launch is this many times faster with the stick held fully up, and `lsi_down` times
    /// as fast held fully down (in between in proportion), except within `lsi_vertical` degrees of straight up or down.
    pub lsi_up: Fx,
    pub lsi_down: Fx,
    pub lsi_vertical: u8,
    /// Bodies on the ground push each other apart (see `push`): each is `push_ratio` of its ECB's half-width either side, the inner
    /// `push_hard` of two bodies' combined width is a wall, and the rest eases apart at up to `push_speed` a frame.
    pub push_ratio: Fx,
    pub push_hard: Fx,
    pub push_speed: Fx,
    /// A jump leaves the ground with `jump_momentum` of the ground speed, at most `jump_momentum_max` times the fighter's air speed
    /// (the reference game's jump speed multiplier and cap; the values are estimates).
    pub jump_momentum: Fx,
    pub jump_momentum_max: Fx,
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
    // ---- Hit feel (the reference game's formulas) ----
    /// Hitlag is `floor(damage * hitlag_per_damage + hitlag_base)` frames (times the shield and crouch-cancel factors), at most
    /// `hitlag_cap`. The damage is the hit's own damage, before the one-on-one multiplier.
    pub hitlag_per_damage: Fx,
    pub hitlag_base: Fx,
    pub hitlag_cap: u8,
    /// Hitlag factor when the hit lands on a shield.
    pub shield_hitlag_mult: Fx,
    /// A fighter crouching on the ground when hit takes this share of the knockback, and both fighters this share of the hitlag
    /// (the victim's capped at `crouch_cancel_hitlag_cap`).
    pub crouch_cancel_kb: Fx,
    pub crouch_cancel_hitlag: Fx,
    pub crouch_cancel_hitlag_cap: u8,
    /// Frames between two survival-DI nudges.
    pub sdi_interval: u8,
    /// Rage: the attacker's percent adds knockback, from nothing at `rage_start` to `rage_max` (a fraction) at `rage_full`.
    pub rage_start: Fx,
    pub rage_full: Fx,
    pub rage_max: Fx,
    /// 1 to weaken moves used over and over (stale-move negation), 0 to turn it off.
    pub stale_moves: u8,
    /// A fighter in a long hitstun can air dodge out of it from this frame of the hitstun, and attack from the second.
    pub hitstun_dodge_cancel: u8,
    pub hitstun_attack_cancel: u8,
    /// ...and only once the launch has slowed below these speeds (world units a frame; the reference's 2.5 and 2.0).
    pub hitstun_dodge_cancel_speed: Fx,
    pub hitstun_attack_cancel_speed: Fx,
    /// Gravity boost: a tumbling launch outside the vertical band gains `gravity_boost * (gravity - gravity_boost_from)` of upward
    /// speed, so a fighter that falls faster is launched a little higher (the reference's 5 and 0.075).
    pub gravity_boost: Fx,
    pub gravity_boost_from: Fx,
    /// In the special fall after an up special, the share of its air speed and air acceleration a fighter keeps (a slight drift).
    pub helpless_drift: Fx,
    /// The first frames of an up special in which pushing the stick back turns it around (an estimate).
    pub special_turn_window: u8,
    /// A B-reverse: the stick pushed the other way in the first this many frames of a neutral or side special turns the fighter round and
    /// reverses its momentum in the air (the reference game: within 4 frames of the special input).
    pub special_reverse_window: u8,
    /// The autolink angle's caps (world units a frame): the launch speed at most `autolink_max_speed` (the reference's 3, the speed
    /// of 100 knockback) and the fall no faster than `autolink_max_fall` (the reference's 1).
    pub autolink_max_speed: Fx,
    pub autolink_max_fall: Fx,
    /// The pull angle (ours, see `moves::ANGLE_PULL`): each frame the victim moves `pull_rate` of the way to the point `pull_gap` in
    /// front of the attacker, at most `pull_max_speed` (world units a frame).
    pub pull_rate: Fx,
    pub pull_max_speed: Fx,
    pub pull_gap: Fx,
    /// Hitlag factor of an electric hit (for both fighters).
    pub electric_hitlag_mult: Fx,
    /// How far an electric hit lets the victim drift with the held stick as its hitlag ends (applied twice).
    pub asdi_distance: Fx,
    /// Every this many hits in one combo, survival DI goes `sdi_combo_mult` times further.
    pub sdi_combo_hits: u8,
    pub sdi_combo_mult: Fx,
    /// Launch speed-up of a strong hit: a hitstun ending on frame F (at most `balloon_max_faf`) plays `1 + (F - balloon_min_faf) * balloon_per_frame` knockback frames per frame at first (at most `balloon_max`), easing back to one.
    pub balloon_min_faf: u8,
    pub balloon_max_faf: u8,
    pub balloon_per_frame: Fx,
    pub balloon_max: Fx,
    /// For the first `launch_fall_frames` frames of a launch every fighter falls the same way: this gravity and fall speed.
    pub launch_fall_accel: Fx,
    pub launch_fall: Fx,
    pub launch_fall_frames: u8,
    /// Fall speed through the whole hitstun of a launch between `vertical_launch_from` and `vertical_launch_to` degrees.
    pub vertical_launch_fall: Fx,
    pub vertical_launch_from: u8,
    pub vertical_launch_to: u8,
    /// Two grounded attacks whose hitboxes meet clank: within this much damage both rebound, otherwise only the weaker one does.
    pub clank_range: Fx,
    /// A rebound lasts `floor((d + 4) * 15 / 8)` frames for the stronger hit's damage `d`, at most this.
    pub rebound_cap: u8,
    /// Two fighters grabbing each other on the same frame both let go, take 1%, and rebound this long.
    pub grab_parry_lag: u8,
    /// After a tech press, another press does not count for this many frames (no mashing).
    pub tech_lockout: u8,
    /// A wall or ceiling tech holds the fighter against the surface this long, intangible for `wall_tech_invuln` frames.
    pub wall_tech_frames: u8,
    pub wall_tech_invuln: u8,
    /// A tumbling fighter that hits a wall, ceiling or (fast enough) the floor without teching bounces off with this share of its speed.
    pub bounce_keep: Fx,
    pub ground_bounce_speed: Fx,
    /// A knocked-out fighter comes back this far above its spawn point, standing on a revival platform for up to `respawn_platform_frames` frames (or until it moves), invincible.
    pub respawn_height: Fx,
    pub respawn_platform_frames: u8,
}

impl Ruleset {
    pub fn standard() -> Ruleset {
        Ruleset {
            damage_mult: Fx::from_ratio(12, 10),
            hitstun_mult: Fx::ONE,
            knockback_decay: Fx::from_ratio(51, 8000),
            tumble_knockback: Fx::from_int(80),
            sdi_distance: Fx::from_ratio(1, 4),
            short_hop_damage: Fx::from_ratio(17, 20),
            di_degrees: 10,
            lsi_up: Fx::from_ratio(1095, 1000),
            lsi_down: Fx::from_ratio(92, 100),
            lsi_vertical: 25,
            push_ratio: Fx::from_ratio(3, 5),
            push_hard: Fx::from_ratio(7, 10),
            push_speed: Fx::from_ratio(3, 40),
            jump_momentum: Fx::from_ratio(17, 20),
            jump_momentum_max: Fx::from_ratio(5, 4),
            respawn_invuln: 120,
            tech_window: 11,
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
            hitlag_per_damage: Fx::from_ratio(65, 100),
            hitlag_base: Fx::from_int(6),
            hitlag_cap: 30,
            shield_hitlag_mult: Fx::from_ratio(67, 100),
            crouch_cancel_kb: Fx::from_ratio(85, 100),
            crouch_cancel_hitlag: Fx::from_ratio(67, 100),
            crouch_cancel_hitlag_cap: 20,
            sdi_interval: 4,
            rage_start: Fx::from_int(35),
            rage_full: Fx::from_int(150),
            rage_max: Fx::from_ratio(1, 10),
            stale_moves: 1,
            hitstun_dodge_cancel: 40,
            hitstun_attack_cancel: 45,
            hitstun_dodge_cancel_speed: Fx::from_ratio(25, 80),
            hitstun_attack_cancel_speed: Fx::from_ratio(20, 80),
            gravity_boost: Fx::from_int(5),
            gravity_boost_from: Fx::from_ratio(75, 8000),
            helpless_drift: Fx::from_ratio(1, 2),
            special_turn_window: 3,
            special_reverse_window: 4,
            autolink_max_speed: Fx::from_ratio(3, 8),
            autolink_max_fall: Fx::from_ratio(1, 8),
            pull_rate: Fx::from_ratio(1, 3),
            pull_max_speed: Fx::from_ratio(1, 2),
            pull_gap: Fx::from_ratio(9, 10),
            electric_hitlag_mult: Fx::from_ratio(3, 2),
            asdi_distance: Fx::from_ratio(133, 800),
            sdi_combo_hits: 5,
            sdi_combo_mult: Fx::from_ratio(115, 100),
            balloon_min_faf: 30,
            balloon_max_faf: 80,
            balloon_per_frame: Fx::from_ratio(1, 10),
            balloon_max: Fx::from_int(6),
            launch_fall_accel: Fx::from_ratio(87, 8000),
            launch_fall: Fx::from_ratio(15, 80),
            launch_fall_frames: 0,
            vertical_launch_fall: Fx::from_ratio(18, 80),
            vertical_launch_from: 70,
            vertical_launch_to: 110,
            clank_range: Fx::from_int(9),
            rebound_cap: 58,
            grab_parry_lag: 30,
            tech_lockout: 40,
            wall_tech_frames: 18,
            wall_tech_invuln: 14,
            bounce_keep: Fx::from_ratio(95, 100),
            ground_bounce_speed: Fx::from_ratio(1, 4),
            respawn_height: Fx::from_int(7),
            respawn_platform_frames: 150,
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
        self.short_hop_damage.hash_into(h);
        h.write_u8(self.di_degrees);
        self.lsi_up.hash_into(h);
        self.lsi_down.hash_into(h);
        h.write_u8(self.lsi_vertical);
        self.push_ratio.hash_into(h);
        self.push_hard.hash_into(h);
        self.push_speed.hash_into(h);
        self.jump_momentum.hash_into(h);
        self.jump_momentum_max.hash_into(h);
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
        self.hitlag_per_damage.hash_into(h);
        self.hitlag_base.hash_into(h);
        h.write_u8(self.hitlag_cap);
        self.shield_hitlag_mult.hash_into(h);
        self.crouch_cancel_kb.hash_into(h);
        self.crouch_cancel_hitlag.hash_into(h);
        h.write_u8(self.crouch_cancel_hitlag_cap);
        h.write_u8(self.sdi_interval);
        self.rage_start.hash_into(h);
        self.rage_full.hash_into(h);
        self.rage_max.hash_into(h);
        h.write_u8(self.stale_moves);
        h.write_u8(self.hitstun_dodge_cancel);
        h.write_u8(self.hitstun_attack_cancel);
        self.hitstun_dodge_cancel_speed.hash_into(h);
        self.hitstun_attack_cancel_speed.hash_into(h);
        self.gravity_boost.hash_into(h);
        self.gravity_boost_from.hash_into(h);
        self.helpless_drift.hash_into(h);
        h.write_u8(self.special_turn_window);
        h.write_u8(self.special_reverse_window);
        self.autolink_max_speed.hash_into(h);
        self.autolink_max_fall.hash_into(h);
        self.pull_rate.hash_into(h);
        self.pull_max_speed.hash_into(h);
        self.pull_gap.hash_into(h);
        self.electric_hitlag_mult.hash_into(h);
        self.asdi_distance.hash_into(h);
        h.write_u8(self.sdi_combo_hits);
        self.sdi_combo_mult.hash_into(h);
        h.write_u8(self.balloon_min_faf);
        h.write_u8(self.balloon_max_faf);
        self.balloon_per_frame.hash_into(h);
        self.balloon_max.hash_into(h);
        self.launch_fall_accel.hash_into(h);
        self.launch_fall.hash_into(h);
        h.write_u8(self.launch_fall_frames);
        self.vertical_launch_fall.hash_into(h);
        h.write_u8(self.vertical_launch_from);
        h.write_u8(self.vertical_launch_to);
        self.clank_range.hash_into(h);
        h.write_u8(self.rebound_cap);
        h.write_u8(self.grab_parry_lag);
        h.write_u8(self.tech_lockout);
        h.write_u8(self.wall_tech_frames);
        h.write_u8(self.wall_tech_invuln);
        self.bounce_keep.hash_into(h);
        self.ground_bounce_speed.hash_into(h);
        self.respawn_height.hash_into(h);
        h.write_u8(self.respawn_platform_frames);
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

/// How the stage looks: which backdrop the game draws behind it and the sky's colours (`rrggbb` hex, empty for the backdrop's own).
/// Presentation only: the simulation never reads it, so it is not part of the content hash and two players may see different
/// skies without the match noticing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageLook {
    pub backdrop: String,
    pub sky_top: String,
    pub sky_bottom: String,
}

impl Default for StageLook {
    fn default() -> StageLook {
        StageLook {
            backdrop: "meadow".to_string(),
            sky_top: String::new(),
            sky_bottom: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub fighters: Vec<FighterParams>,
    pub weapons: Vec<Weapon>,
    pub stage: Stage,
    pub rules: Ruleset,
    pub names: Names,
    /// Not hashed (see [`StageLook`]).
    pub look: StageLook,
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

    /// The built-in fighters (a swordfighter, a close-range brawler and a quick rogue) on a main stage with platforms.
    pub fn placeholder() -> Content {
        Content {
            fighters: vec![
                FighterParams::duelist(),
                FighterParams::brawler(),
                FighterParams::rogue(),
            ],
            weapons: vec![
                crate::moves::longsword(),
                crate::moves::claws(),
                crate::moves::needles(),
            ],
            stage: Stage::placeholder(),
            rules: Ruleset::standard(),
            look: StageLook::default(),
            names: Names {
                fighters: vec![
                    "duelist".to_string(),
                    "brawler".to_string(),
                    "rogue".to_string(),
                ],
                weapons: vec![
                    "longsword".to_string(),
                    "claws".to_string(),
                    "needles".to_string(),
                ],
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

        let mut c = base.clone();
        c.fighters[0].crouch_height += Fx::from_raw(1);
        assert_ne!(h, c.hash());

        for change in [
            |r: &mut Ruleset| r.lsi_up += Fx::from_raw(1),
            |r: &mut Ruleset| r.lsi_down += Fx::from_raw(1),
            |r: &mut Ruleset| r.lsi_vertical += 1,
            |r: &mut Ruleset| r.push_ratio += Fx::from_raw(1),
            |r: &mut Ruleset| r.push_hard += Fx::from_raw(1),
            |r: &mut Ruleset| r.push_speed += Fx::from_raw(1),
            |r: &mut Ruleset| r.jump_momentum += Fx::from_raw(1),
            |r: &mut Ruleset| r.jump_momentum_max += Fx::from_raw(1),
        ] {
            let mut c = base.clone();
            change(&mut c.rules);
            assert_ne!(h, c.hash());
        }

        let mut c = base.clone();
        c.weapons[0].limbs = !c.weapons[0].limbs;
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
