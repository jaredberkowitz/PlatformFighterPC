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
    /// A flick the other way within this many frames of an initial dash's start reverses it at once. Later in the initial dash the
    /// flick is held, and the dash turns when it ends (`dash_frames`); once it is a run, a flick back is the run's skid.
    ///
    /// 255 (owner decision 2026-10-14, sim 106): every flick back in the initial dash turns at once; the held turn felt sluggish.
    /// **The reference's value is 5** (its "fast" turnaround on frames 1-5, the rest held to the dash's end: sim 103): set it back to 5
    /// to try that again with more playtesters (`feel.rs::the_reference_held_turn_*` keeps it working).
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
    /// Kicking off a wall: 1 if the body can wall jump (touching a wall in the air, the stick flicked away from it), 0 if not; and
    /// the speeds of that jump and of every body's wall-tech jump, away from the wall and up (the reference's
    /// `passive_wall_jump_x_speed` and `_y_speed`).
    pub wall_jump: u8,
    pub wall_jump_vx: Fx,
    pub wall_jump_vy: Fx,
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
    /// The body's gauge and its powered mode (see [`Gauge`]); [`Gauge::NONE`] for a body without one.
    pub gauge: Gauge,
    /// Armor on the body's midair jump (sim 89, E33; `None` for none). Its window counts the frames of the jump (1 is the jump's own
    /// frame: `Fighter::air_jump_frame`), and it lasts only while the fighter stays in the jump: an attack, a special, an air dodge, a
    /// landing, a ledge or a launch ends it. Read in `combat::apply_hit` as a move's armor is; grabs ignore it. The reference's
    /// double-jump armor is the subtractive knockback kind (`Armor::subtract`).
    pub air_jump_armor: Option<crate::moves::Armor>,
    /// The midair jump turns the fighter round when the stick is held back (past the deadzone) as it jumps: the reference's
    /// double-jump turnaround, a trait of a few bodies. False: the facing stays.
    pub air_jump_turns: bool,
    /// A shield carried in front that stops projectiles while the fighter stands, walks or crouches (E14, the reference's passive
    /// shield; see [`PassiveShield`]). `None` for none.
    pub passive_shield: Option<PassiveShield>,
    /// The midair jump's sideways speed in percent of the stick times the air speed (owner decision 59, sim 95: the reference's per-body
    /// `jump_aerial_speed_x_mul`); 100 (the default) is the shared rule.
    pub air_jump_drift: u8,
    /// The frames on which the body lets go of an item it throws (E4, sim 111; see [`ItemThrows`]).
    pub item_throws: ItemThrows,
    /// Index into `Content::weapons`.
    pub weapon: u8,
}

/// The frame of each item throw on which the item leaves the hand (E4, sim 111; the reference's per-body release frames, ssbwiki.com
/// `Item_throw`, 13.0.1): forward, back, up and down throws (smash throws alike, on the ground or in the air), the dash throw and the
/// grounded drop. Frame 1 is the throw's first. The lengths are every body's alike (the ruleset's `item_*_frames`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemThrows {
    pub forward: u8,
    pub back: u8,
    pub up: u8,
    pub down: u8,
    pub dash: u8,
    pub drop: u8,
}

impl ItemThrows {
    /// The reference's most common release frames (8, 11, 12, 7, 5, 6, shared by the reference characters behind the tumbler, sprinter, champion and brawler [R]): every body
    /// without its own.
    pub const STANDARD: ItemThrows = ItemThrows {
        forward: 8,
        back: 11,
        up: 12,
        down: 7,
        dash: 5,
        drop: 6,
    };

    /// The frames by name, in file order.
    pub fn frames(&self) -> [(&'static str, u8); 6] {
        [
            ("forward", self.forward),
            ("back", self.back),
            ("up", self.up),
            ("down", self.down),
            ("dash", self.dash),
            ("drop", self.drop),
        ]
    }
}

/// A passive shield (E14, the reference's): while the fighter is standing (`Idle`), walking or crouching on the ground, a projectile
/// whose box touches this standing capsule (centred `x` ahead of the feet, the way it faces, and `y` up, or `crouch_y` up while
/// crouching; `half_height` above and below the centre, of `radius`), coming from in front, is blocked: no damage and no launch; the
/// fighter freezes for the shield's hitlag (shield SDI allowed) and is pushed back as a shield is by that hit (`combat::block`'s
/// pushback, with no shield damage or stun); the projectile ends as on a shield (a piercing one goes on). It never breaks and blocks any
/// number of shots. Melee hits are never blocked (the reference blocks only reflectable hitboxes: every projectile here, no melee hit).
/// Scaled by the body's `hitbox_scale`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassiveShield {
    pub x: Fx,
    pub y: Fx,
    pub crouch_y: Fx,
    pub half_height: Fx,
    pub radius: Fx,
}

/// A gauge that fills from damage dealt and taken (and from moves that charge it), and a powered mode while it is full: the
/// reference's gauge-and-break mechanic, built general (owner decision 31). Its working name in the game is in [`GAUGE_NAME`].
///
/// `max` 0 means no gauge (every body but the champion). Gain: `per_dealt` units per percent of damage this fighter's hits add to
/// another fighter's percent, `per_taken` per percent added to its own (or taken away while it is in a `Move::gauge_drain` move, or
/// held out of one). Reaching `max` starts the powered mode for `mode_frames` frames; while it runs nothing more is gained or lost,
/// and the body's numbers are multiplied by the percents below (100 = unchanged; jump launch speeds never change): `walk` (walk
/// speed and both walk accelerations), `dash` (both dash speeds), `run` (run speed and both run accelerations), `traction`
/// (`ground_friction` and `dash_attack_friction`), `air_speed`, `air_accel` (both parts), `air_friction`, `gravity`, `fall` (both fall
/// speeds). The mode ends when its time runs out (the gauge empties) or a special with a `Move::limit_version` spends it. A KO empties
/// the gauge and ends the mode.
///
/// Sim 121 (E53, the reference's second gauge, the partner's): `per_frame` units fill it every frame by themselves (not on a revival
/// platform, not in the mode); the gain from damage taken is also multiplied by `behind[0]` / `behind[1]` per mille when the fighter
/// has one / two or more stocks fewer than the leader (a stock match), and, with three or four fighters in the match, by `crowd`'s row
/// (per mille, by the same stock gap: none, one, two or more); a move's `Move::gauge_gain` multiplies it too. While the mode runs, damage
/// taken cuts `mode_taken[players - 2]` frames a percent off the mode (2, 3 or 4 fighters). A KO leaves `ko_keep[0]` units in the gauge
/// (`ko_keep[1]` two stocks behind the leader, `ko_keep[2]` three or more; times the `crowd` row) instead of emptying it. With
/// `keeps`, a special's powered version (`Move::limit_version`) does not spend the gauge; with `halts`, the mode's clock stands still
/// while the fighter's shield is broken. The defaults (0, 1000, false) leave the champion's gauge as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gauge {
    pub max: Fx,
    pub per_dealt: Fx,
    pub per_taken: Fx,
    pub mode_frames: u16,
    pub per_frame: Fx,
    pub behind: [u16; 2],
    pub crowd: [[u16; 3]; 2],
    pub mode_taken: [Fx; 3],
    pub ko_keep: [Fx; 3],
    pub keeps: bool,
    pub halts: bool,
    pub walk: u16,
    pub dash: u16,
    pub run: u16,
    pub traction: u16,
    pub air_speed: u16,
    pub air_accel: u16,
    pub air_friction: u16,
    pub gravity: u16,
    pub fall: u16,
}

/// The gauge's working name in the game, the one place it is written down in code (owner decision 30: "Boost" for now). The script
/// names that follow it are `boost`, `boosting` and `boost_add` (`sim-script/src/api.rs`, `docs/CONTENT.md`); a rename changes
/// those three and this.
pub const GAUGE_NAME: &str = "Boost";

impl Gauge {
    /// No gauge (every multiplier 100).
    pub const NONE: Gauge = Gauge {
        max: Fx::ZERO,
        per_dealt: Fx::ZERO,
        per_taken: Fx::ZERO,
        mode_frames: 0,
        per_frame: Fx::ZERO,
        behind: [1000; 2],
        crowd: [[1000; 3]; 2],
        mode_taken: [Fx::ZERO; 3],
        ko_keep: [Fx::ZERO; 3],
        keeps: false,
        halts: false,
        walk: 100,
        dash: 100,
        run: 100,
        traction: 100,
        air_speed: 100,
        air_accel: 100,
        air_friction: 100,
        gravity: 100,
        fall: 100,
    };

    /// Whether the body has a gauge.
    pub const fn exists(&self) -> bool {
        self.max.raw() > 0
    }

    /// The multipliers by name, in file order.
    pub fn percents(&self) -> [(&'static str, u16); 9] {
        [
            ("walk", self.walk),
            ("dash", self.dash),
            ("run", self.run),
            ("traction", self.traction),
            ("air_speed", self.air_speed),
            ("air_accel", self.air_accel),
            ("air_friction", self.air_friction),
            ("gravity", self.gravity),
            ("fall", self.fall),
        ]
    }
}

impl StateHash for Gauge {
    fn hash_into(&self, h: &mut StateHasher) {
        self.max.hash_into(h);
        self.per_dealt.hash_into(h);
        self.per_taken.hash_into(h);
        h.write_u16(self.mode_frames);
        for (_, p) in self.percents() {
            h.write_u16(p);
        }
        self.per_frame.hash_into(h);
        for v in self.behind.iter().chain(self.crowd.iter().flatten()) {
            h.write_u16(*v);
        }
        for v in self.mode_taken.iter().chain(self.ko_keep.iter()) {
            v.hash_into(h);
        }
        h.write_bool(self.keeps);
        h.write_bool(self.halts);
    }
}

/// `x` times `percent` / 100, exactly in the fixed-point raw value (rounded toward zero) [D].
fn percent_of(x: Fx, percent: u16) -> Fx {
    if percent == 100 {
        return x;
    }
    Fx::from_raw((i64::from(x.raw()) * i64::from(percent) / 100) as i32)
}

impl FighterParams {
    /// The body as it moves this frame: itself, or, while the fighter's gauge's powered mode runs, a copy with the gauge's
    /// multipliers applied (see [`Gauge`]). Weight, size, jump speeds and every other number stay as they are.
    pub fn powered(&self, powered: bool) -> FighterParams {
        let g = self.gauge;
        if !powered || !g.exists() {
            return *self;
        }
        let mut q = *self;
        q.walk_speed = percent_of(self.walk_speed, g.walk);
        q.walk_accel = percent_of(self.walk_accel, g.walk);
        q.walk_accel_stick = percent_of(self.walk_accel_stick, g.walk);
        q.dash_speed = percent_of(self.dash_speed, g.dash);
        q.dash_initial_speed = percent_of(self.dash_initial_speed, g.dash);
        q.run_speed = percent_of(self.run_speed, g.run);
        q.run_accel = percent_of(self.run_accel, g.run);
        q.run_accel_stick = percent_of(self.run_accel_stick, g.run);
        q.ground_friction = percent_of(self.ground_friction, g.traction);
        q.dash_attack_friction = percent_of(self.dash_attack_friction, g.traction);
        q.air_speed = percent_of(self.air_speed, g.air_speed);
        q.air_accel = percent_of(self.air_accel, g.air_accel);
        q.air_accel_stick = percent_of(self.air_accel_stick, g.air_accel);
        q.air_friction = percent_of(self.air_friction, g.air_friction);
        q.gravity = percent_of(self.gravity, g.gravity);
        q.max_fall_speed = percent_of(self.max_fall_speed, g.fall);
        q.fast_fall_speed = percent_of(self.fast_fall_speed, g.fall);
        q
    }
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
    pub fn fx_fields(&self) -> [(&'static str, Fx); 55] {
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
            ("wall_jump_vx", self.wall_jump_vx),
            ("wall_jump_vy", self.wall_jump_vy),
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
    pub fn int_fields(&self) -> [(&'static str, u32); 59] {
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
            ("wall_jump", u32::from(self.wall_jump)),
            ("ledge_attack_frames", u32::from(self.ledge_attack_frames)),
            ("weapon", u32::from(self.weapon)),
        ]
    }

    /// Converts a reference-style number (given in thousandths of a unit) into world units.
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
            gauge: Gauge::NONE,
            air_jump_armor: None,
            air_jump_turns: false,
            passive_shield: None,
            air_jump_drift: 100,
            item_throws: ItemThrows::STANDARD,
            walk_speed: gu(1400),
            run_speed: gu(1800),
            dash_speed: gu(2200),
            dash_initial_speed: gu(900),
            dash_accel: gu(300),
            dash_brake: gu(520),
            dash_frames: 10,
            dash_reverse_frames: 255,
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
            // The reference's common wall-tech jump (1.3 away, 2.3 up); no wall jump of its own.
            wall_jump: 0,
            wall_jump_vx: Fx::from_ratio(1300, 8000),
            wall_jump_vy: Fx::from_ratio(2300, 8000),
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
    /// The reference duelist: walk 1.575, run 1.964, dash 2.255, air speed 1.071, gravity 0.075,
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
            // No wall jump; the reference's wall-tech jump (dragdown, the parameter file): 1.3 away, 2.4 up.
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2400),
            // Item throws let go on 10, 14, 15, 8, 6, 7 (the reference character's: ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 10,
                back: 14,
                up: 15,
                down: 8,
                dash: 6,
                drop: 7,
            },
            ..FighterParams::base()
        }
    }

    /// Heavier, faster in the air, with a high gravity and fast fall: the "blaster brawler" body type.
    /// The reference brawler: walk 1.208, run 1.54, dash 2.09, air speed 1.281, gravity 0.13,
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
            // The reference brawler's air dodges: neutral 44 frames (intangible 2-26), directional 61 / 73 / 93 (2-20).
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
            // The reference's wall jump and wall-tech jump (dragdown, the parameter file): 1.3 away, 2.3 up.
            wall_jump: 1,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2300),
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// Light, fast on the ground, a high gravity and fast fall: the "rogue" body type of the Needles moveset. The reference rogue's
    /// attributes (ssbwiki.com / ultimateframedata.com, `docs/movesets/needles/REFERENCE_DATA.md`): weight 78, walk 1.47, initial dash 2.178,
    /// run 2.42, traction 0.106, air speed 1.155, air acceleration 0.01 + 0.08, air friction 0.015, gravity 0.15, fall 1.75 (fast
    /// 2.8), full hop 39, short hop 18.75, double jump 40; air dodges 44 (neutral, intangible 2-26) and 63 / 74 / 93 (directional,
    /// intangible 2-19). On the ground: walk acceleration 0.105 + 0.21, run acceleration 0.044 + 0.121, an initial dash of 7 frames
    /// (the others' 10), a heavy landing of 7 (6 of lag) (the parameter file, via dragdown.wiki and ssbwiki.com's dash table). The
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
            // The reference's wall jump and wall-tech jump (dragdown, the parameter file): 1.4 away, 2.8 up.
            wall_jump: 1,
            wall_jump_vx: su(1400),
            wall_jump_vy: su(2800),
            // Item throws let go on 8, 11, 11, 8, 6, 6 (the reference character's (its grounded up and down: 11, 8): ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 8,
                back: 11,
                up: 11,
                down: 8,
                dash: 6,
                drop: 6,
            },
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A short middleweight all-rounder with a fast air speed and high, floaty jumps: the "tumbler" body of the Embers moveset. The
    /// reference character's (ssbwiki.com attributes, the parameter file via dragdown.wiki, ultimateframedata.com,
    /// `docs/movesets/embers/REFERENCE_DATA.md`): weight 98, walk 1.155, initial dash 1.936, run 1.76, traction 0.102, air speed 1.208,
    /// air acceleration 0.01 + 0.07, air friction 0.015, gravity 0.087, fall 1.5 (fast 2.4), full hop 36.33, short hop 17.54, double
    /// jump 36.33, ledge jump 38.165. On the ground: walk acceleration 0.105 + 0.105, run acceleration 0.044 + 0.10978, an initial dash
    /// of 10 frames, a heavy landing of 4 (3 of lag). Air dodges 52 (neutral, intangible 3-29) and 71 / 87 / 116 (directional,
    /// intangible 3-21); rolls 29 forward (intangible 4-15) and 34 back (5-16, not separate here); spot dodge 25 (3-17).
    pub fn tumbler() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(87);
        let (burst, arc) = Self::full_hop(gravity, su(36330));
        FighterParams {
            walk_speed: gu(1155),
            run_speed: gu(1760),
            dash_speed: gu(1936),
            walk_accel: gu(105),
            walk_accel_stick: gu(105),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(10978),
            dash_frames: 10,
            heavy_landing_lag: 3,
            air_speed: su(1208),
            air_accel: su(10),
            air_accel_stick: su(70),
            air_friction: su(15),
            gravity,
            max_fall_speed: su(1500),
            fast_fall_speed: su(2400),
            air_dodge_frames: 52,
            air_dodge_dir_down_frames: 71,
            air_dodge_dir_side_frames: 87,
            air_dodge_dir_up_frames: 116,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 29,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.102 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(204),
            dash_attack_friction: gu(102),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(17540)),
            air_jump_velocity: Self::hop_velocity(gravity, su(36330)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(38165)),
            weight: Fx::from_int(98),
            weapon: 3,
            // Rolls and spot dodge are the shared ones, which are the reference's (forward 29, intangible 4-15; back 34; spot dodge
            // 25, intangible 3-17), apart from the back roll's own window (5-16), which is not separate here [D].
            // The reference's wall jump and wall-tech jump (dragdown, the parameter file): 1.3 away, 2.3 up.
            wall_jump: 1,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2300),
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A tall, heavy and very fast runner with a strong fall: the "sprinter" body of the Gauntlets moveset. The reference
    /// character's attributes (ssbwiki.com, the parameter file via dragdown.wiki, ultimateframedata.com,
    /// `docs/movesets/gauntlets/REFERENCE_DATA.md`): weight 104, walk 0.987, initial dash 1.98, run 2.552, traction 0.105, air
    /// speed 1.218, air acceleration 0.01 + 0.065, air friction 0.0075, gravity 0.12, fall 1.865 (fast 2.984), full hop 37.31, short
    /// hop 17.98, double jump 37.31, ledge jump 38.655. On the ground: walk acceleration 0.105 + 0.158, run acceleration 0.11 +
    /// 0.066, an initial dash of 15 frames, a heavy landing of 4 (3 of lag). Air dodges 42 (neutral, intangible 3-30) and 62 / 72 /
    /// 94 (directional, intangible 3-21); rolls 30 forward and 35 back (intangible 4-15), spot dodge 26 (3-17). A wall jump (1.4
    /// away, 3.1 up). The body is taller than the shared one [E: fitted to the heights its attacks reach].
    pub fn sprinter() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let r = Fx::from_ratio;
        let gravity = su(120);
        let (burst, arc) = Self::full_hop(gravity, su(37310));
        FighterParams {
            walk_speed: gu(987),
            run_speed: gu(2552),
            dash_speed: gu(1980),
            walk_accel: gu(105),
            walk_accel_stick: gu(158),
            run_accel: gu(110),
            run_accel_stick: FighterParams::gu_fine(6600),
            dash_frames: 15,
            // (The shield can cut a dash short from about half way, as for the others [E].)
            dash_shield_frame: 8,
            heavy_landing_lag: 3,
            air_speed: su(1218),
            air_accel: su(10),
            air_accel_stick: su(65),
            air_friction: FighterParams::su_fine(750),
            gravity,
            max_fall_speed: su(1865),
            fast_fall_speed: su(2984),
            air_dodge_frames: 42,
            air_dodge_dir_down_frames: 62,
            air_dodge_dir_side_frames: 72,
            air_dodge_dir_up_frames: 94,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.105 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(210),
            dash_attack_friction: gu(105),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(17980)),
            air_jump_velocity: Self::hop_velocity(gravity, su(37310)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(38655)),
            weight: Fx::from_int(104),
            weapon: 4,
            // Dodges (the reference's): a forward roll of 30 frames, a backward one of 35, intangible 4-15 (the back roll's 5-16 is
            // not separate here [D]); a spot dodge of 26, intangible 3-17 (the shared windows).
            roll_frames: 30,
            roll_back_frames: 35,
            spot_dodge_frames: 26,
            // The reference's wall jump and wall-tech jump (dragdown, the parameter file): 1.4 away, 3.1 up. (Its 0.2 "enable" speed
            // and 0.5 passive wall speed are not modelled for anyone.)
            wall_jump: 1,
            wall_jump_vx: su(1400),
            wall_jump_vy: su(3100),
            // Taller than the shared body: its attacks reach 10-11 reference units up for a punch at the chest and 29 above the head
            // [E].
            ecb_height: r(13, 5),
            ecb_side_height: r(13, 10),
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A tall middleweight caster with a slow air speed and quick acceleration in the air: the "seer" body of the Staff moveset. The
    /// reference character's attributes (ssbwiki.com, the parameter file via dragdown.wiki, ultimateframedata.com,
    /// `docs/movesets/staff/REFERENCE_DATA.md`): weight 91, walk 1.271, initial dash 2.035, run 2.077, traction 0.106, air speed
    /// 1.0, air acceleration 0.01 + 0.105, air friction 0.01, gravity 0.12, fall 1.55 (fast 2.48), full hop 35.9, short hop 17.3,
    /// double jump 35.9, ledge jump 37.95. On the ground: walk acceleration 0.105 + 0.21, run acceleration 0.044 + 0.12991, an initial
    /// dash of 10 frames, a heavy landing of 7 (6 of lag: the parameter file, owner decision 22). Air dodges 49 (neutral, intangible
    /// 3-30) and 70 / 82 / 105 (directional, intangible 3-21); rolls 29 forward (intangible 4-15) and 34 back, spot dodge 25 (3-17):
    /// the shared ones. No wall jump (the parameter file); the wall-tech jump 1.3 away, 2.2 up. The body is taller than the shared
    /// one [E: fitted to the heights its attacks reach].
    pub fn seer() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let r = Fx::from_ratio;
        let gravity = su(120);
        let (burst, arc) = Self::full_hop(gravity, su(35900));
        FighterParams {
            walk_speed: gu(1271),
            run_speed: gu(2077),
            dash_speed: gu(2035),
            walk_accel: gu(105),
            walk_accel_stick: gu(210),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(12991),
            dash_frames: 10,
            heavy_landing_lag: 6,
            air_speed: su(1000),
            air_accel: su(10),
            air_accel_stick: su(105),
            air_friction: su(10),
            gravity,
            max_fall_speed: su(1550),
            fast_fall_speed: su(2480),
            air_dodge_frames: 49,
            air_dodge_dir_down_frames: 70,
            air_dodge_dir_side_frames: 82,
            air_dodge_dir_up_frames: 105,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.106 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(212),
            dash_attack_friction: gu(106),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(17300)),
            air_jump_velocity: Self::hop_velocity(gravity, su(35900)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(37950)),
            weight: Fx::from_int(91),
            weapon: 5,
            // No wall jump; the reference's wall-tech jump (the parameter file): 1.3 away, 2.2 up.
            wall_jump: 0,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2200),
            // Taller than the shared body, as the sprinter: the up tilt's top box is 23 reference units up and the counter's bubble is
            // centred 10 up [E].
            ecb_height: r(13, 5),
            ecb_side_height: r(13, 10),
            // Item throws let go on 7, 9, 11, 6, 4, 5 (the reference character's: ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 7,
                back: 9,
                up: 11,
                down: 6,
                dash: 4,
                drop: 5,
            },
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A middleweight with a huge sword: the "champion" body of the Greatsword moveset. The reference character's attributes
    /// (ssbwiki.com, the parameter file via dragdown.wiki, ultimateframedata.com, the seventh moveset's `REFERENCE_DATA.md` in
    /// `docs/movesets/`): weight 100, walk 1.155, initial dash 2.145, run 2.167, traction 0.106, air speed 1.155, air acceleration 0.01 + 0.06, air friction 0.009,
    /// gravity 0.098, fall 1.68 (fast 2.688), full hop 32.5, short hop 17, double jump 32.5, ledge jump 36.25 (drift 0.6). On the
    /// ground: walk acceleration 0 + 0.1575, run acceleration 0.011 + 0.099, an initial dash of 10 frames, a heavy landing of 4 (3 of
    /// lag: the parameter file's `landing_heavy_frame`). Air dodges 47 (neutral, intangible 3-30) and 64 / 79 / 104 (directional,
    /// intangible 3-21); rolls 29 forward (intangible 4-15) and 34 back, spot dodge 25 (3-17): the shared ones. A wall jump (the
    /// parameter file's `wall_jump_type`), 1.3 away and 2.4 up, the same speeds as its wall-tech jump. The shared body height [E].
    pub fn champion() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(98);
        let (burst, arc) = Self::full_hop(gravity, su(32500));
        FighterParams {
            walk_speed: gu(1155),
            run_speed: gu(2167),
            dash_speed: gu(2145),
            walk_accel: Fx::ZERO,
            walk_accel_stick: FighterParams::gu_fine(15750),
            run_accel: gu(11),
            run_accel_stick: gu(99),
            dash_frames: 10,
            heavy_landing_lag: 3,
            air_speed: su(1155),
            air_accel: su(10),
            air_accel_stick: su(60),
            air_friction: su(9),
            gravity,
            max_fall_speed: su(1680),
            fast_fall_speed: su(2688),
            air_dodge_frames: 47,
            air_dodge_dir_down_frames: 64,
            air_dodge_dir_side_frames: 79,
            air_dodge_dir_up_frames: 104,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.106 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(212),
            dash_attack_friction: gu(106),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(17000)),
            air_jump_velocity: Self::hop_velocity(gravity, su(32500)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(36250)),
            weight: Fx::from_int(100),
            weapon: 6,
            // The reference's wall jump (`passive_wall_jump_x_speed` 1.3, `_y_speed` 2.4). (Its 0.2 "enable" speed and 0.5 passive wall
            // speed are not modelled for anyone.)
            wall_jump: 1,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2400),
            gauge: Self::champion_gauge(),
            ..FighterParams::base()
        }
    }

    /// The champion's gauge (the reference's, ssbwiki.com and ultimateframedata.com, 13.0.1): 100 units, 0.667 a percent of damage
    /// dealt and 1 a percent taken; full, 900 frames of the powered mode with walk x1.15, dash and run x1.1, traction x1.15, air speed
    /// and acceleration x1.2, air friction x1.15, gravity and fall speeds x1.1 (the jumps' launch speeds unchanged, so they are a
    /// little lower) [R].
    pub fn champion_gauge() -> Gauge {
        Gauge {
            max: Fx::from_int(100),
            per_dealt: Fx::from_ratio(667, 1000),
            per_taken: Fx::ONE,
            mode_frames: 900,
            walk: 115,
            dash: 110,
            run: 110,
            traction: 115,
            air_speed: 120,
            air_accel: 120,
            air_friction: 115,
            gravity: 110,
            fall: 110,
            ..Gauge::NONE
        }
    }
}

impl FighterParams {
    /// A huge super-heavyweight with long arms: the "hauler" body of the Bracers moveset. The reference character's attributes
    /// (ssbwiki.com, the parameter file via dragdown.wiki, ultimateframedata.com, the eighth moveset's `REFERENCE_DATA.md` in
    /// `docs/movesets/`): weight 127, walk 1.365, initial dash 2.09, run 1.873, traction 0.123, air speed 1.208, air acceleration
    /// 0.01 + 0.05, air friction 0.015, gravity 0.085, fall 1.63 (fast 2.608), full hop 34, short hop 17.3, double jump 35.5, ledge jump
    /// 37 (drift 0.6, the shared value). On the ground: walk acceleration 0.105 + 0.105, run acceleration 0.044 + 0.07337, an initial
    /// dash of 15 frames, a heavy landing of 4 (3 of lag: the parameter file's `landing_heavy_frame`). Air dodges 48 (neutral,
    /// intangible 4-32) and 66 / 82 / 109 (directional, intangible 4-22); rolls 30 forward (intangible 4-15) and 35 back, spot dodge 26
    /// (3-17); the ledge options' intangibility cut to 80% and 48% on the second and third grab. No wall jump (the parameter file); the
    /// wall-tech jump 1.3 away, 2.7 up. A bigger body than the shared one [E: the reference's "one of the largest hurtboxes", fitted to
    /// the heights and reaches of its attacks].
    pub fn hauler() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let r = Fx::from_ratio;
        let gravity = su(85);
        let (burst, arc) = Self::full_hop(gravity, su(34000));
        FighterParams {
            walk_speed: gu(1365),
            run_speed: gu(1873),
            dash_speed: gu(2090),
            walk_accel: gu(105),
            walk_accel_stick: gu(105),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(7337),
            dash_frames: 15,
            // (The shield can cut a dash short from about half way, as for the others [E].)
            dash_shield_frame: 8,
            heavy_landing_lag: 3,
            air_speed: su(1208),
            air_accel: su(10),
            air_accel_stick: su(50),
            air_friction: su(15),
            gravity,
            max_fall_speed: su(1630),
            fast_fall_speed: su(2608),
            air_dodge_frames: 48,
            air_dodge_dir_down_frames: 66,
            air_dodge_dir_side_frames: 82,
            air_dodge_dir_up_frames: 109,
            air_dodge_intangible_start: 4,
            air_dodge_intangible_end: 32,
            air_dodge_dir_intangible_end: 22,
            // Traction 0.123 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(246),
            dash_attack_friction: gu(123),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(17300)),
            air_jump_velocity: Self::hop_velocity(gravity, su(35500)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(37000)),
            weight: Fx::from_int(127),
            weapon: 7,
            // Dodges (the reference's): a forward roll of 30 frames, a backward one of 35, intangible 4-15 (the back roll's 5-16 is
            // not separate here [D]); a spot dodge of 26, intangible 3-17 (the shared windows).
            roll_frames: 30,
            roll_back_frames: 35,
            spot_dodge_frames: 26,
            ledge_option_decay_3: 48,
            // No wall jump; the reference's wall-tech jump (the parameter file): 1.3 away, 2.7 up.
            wall_jump: 0,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2700),
            // Bigger than the shared body: its up smash reaches 28 reference units up and its forward smash 24 ahead [E].
            ecb_half_width: Fx::ONE,
            ecb_height: r(14, 5),
            ecb_side_height: r(7, 5),
            // (It hangs a little further out from the wall, as its body is wider [E].)
            ledge_hang_dx: r(11, 10),
            // Item throws let go on 10, 14, 15, 8, 6, 7 (the reference character's: ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 10,
                back: 14,
                up: 15,
                down: 8,
                dash: 6,
                drop: 7,
            },
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A floaty heavyweight with a towering second jump: the "bounder" body of the Clutch moveset. The reference character's attributes
    /// (ssbwiki.com, the parameter file via dragdown.wiki, ultimateframedata.com, the ninth moveset's `REFERENCE_DATA.md` in
    /// `docs/movesets/`): weight 104, walk 1.208, initial dash 1.98, run 2.046, traction 0.095, air speed 1.344, air acceleration 0.03 +
    /// 0.068, air friction 0.005, gravity 0.08, fall 1.29 (fast 2.064), full hop 36.09, short hop 14.43, **double jump 51.56**, ledge jump
    /// 38.045 (drift 0.6, the shared value). On the ground: walk acceleration 0.105 + 0.168, run acceleration 0.044 + 0.1056, an initial
    /// dash of 14 frames, a heavy landing of 7 (6 of lag: the parameter file's `landing_heavy_frame`). Air dodges 58 (neutral,
    /// intangible 3-30) and 79 / 107 / 130 (directional down / sideways / up, intangible 3-21); rolls 30 forward (intangible 4-15) and 35
    /// back, spot dodge 26 (3-17); the ledge options' intangibility cut to 82% and 50% on the second and third grab. No wall jump; the
    /// wall-tech jump 1.3 away, 2.3 up (the shared one). The shared body size [E].
    ///
    /// **The double jump** (E33): an ordinary jump to 51.56 (not a delayed one in 13.0.1), turning round with the stick held back, and
    /// **subtractive knockback armor on its frames 1-44**: a hit of at most 120 units of knockback only damages and freezes; a stronger
    /// one launches with its knockback less 120, at least 60 (each hit compared on its own: owner decision 47). An attack, a special or
    /// an air dodge ends it; grabs ignore it.
    pub fn bounder() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(80);
        let (burst, arc) = Self::full_hop(gravity, su(36090));
        FighterParams {
            walk_speed: gu(1208),
            run_speed: gu(2046),
            dash_speed: gu(1980),
            walk_accel: gu(105),
            walk_accel_stick: gu(168),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(10560),
            dash_frames: 14,
            // (The shield can cut a dash short from about half way, as for the others [E].)
            dash_shield_frame: 7,
            heavy_landing_lag: 6,
            air_speed: su(1344),
            air_accel: su(30),
            air_accel_stick: su(68),
            air_friction: su(5),
            gravity,
            max_fall_speed: su(1290),
            fast_fall_speed: su(2064),
            air_dodge_frames: 58,
            air_dodge_dir_down_frames: 79,
            air_dodge_dir_side_frames: 107,
            air_dodge_dir_up_frames: 130,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.095 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(190),
            dash_attack_friction: gu(95),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(14430)),
            air_jump_velocity: Self::hop_velocity(gravity, su(51560)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(38045)),
            weight: Fx::from_int(104),
            weapon: 8,
            // Dodges (the reference's): a forward roll of 30 frames, a backward one of 35, intangible 4-15 (the back roll's 5-16 is
            // not separate here [D]); a spot dodge of 26, intangible 3-17 (the shared windows).
            roll_frames: 30,
            roll_back_frames: 35,
            spot_dodge_frames: 26,
            ledge_option_decay_2: 82,
            // No wall jump; the reference's wall-tech jump (1.3 away, 2.3 up) is the shared one.
            wall_jump: 0,
            air_jump_armor: Some(Self::bounder_air_jump_armor()),
            air_jump_turns: true,
            // The double jump's sideways speed x1.2 (the parameter file's `jump_aerial_speed_x_mul`, dragdown.wiki) [R].
            air_jump_drift: 120,
            // Item throws let go on 8, 7, 8, 7, 7, 6 (the reference character's (its grounded down: 7): ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 8,
                back: 7,
                up: 8,
                down: 7,
                dash: 7,
                drop: 6,
            },
            ..FighterParams::base()
        }
    }

    /// The bounder's double-jump armor (ssbwiki.com's list of armored attacks and the move notes, dragdown.wiki, 13.0.1): frames 1-44
    /// of the jump (move frames 0-43), knockback-based, 120 units, subtractive, a breaking hit launching with at least 60 units [R].
    pub fn bounder_air_jump_armor() -> crate::moves::Armor {
        crate::moves::Armor {
            start: 0,
            end: 43,
            kind: crate::moves::ARMOR_KNOCKBACK,
            threshold: Fx::from_int(120),
            subtract: true,
            min_kb: Fx::from_int(60),
        }
    }
}

impl FighterParams {
    /// A heavy swordsman with low jumps and a shield on his arm: the "wanderer" body of the Buckler moveset. The reference character's
    /// attributes (ssbwiki.com, the parameter file via dragdown.wiki, ultimateframedata.com, the tenth moveset's `REFERENCE_DATA.md` in
    /// `docs/movesets/`; conflict 1 resolved to the parameter file): weight 104, walk 1.247, initial dash 1.98, run 1.534,
    /// traction 0.113, air speed 0.924, air acceleration 0.01 + 0.04, air friction 0.0038, gravity 0.096, fall 1.6 (fast 3.04), full hop
    /// 27.8, short hop 13.38, double jump 29.0, ledge jump 33.9 (drift 0.6, the shared value). On the ground: walk acceleration 0.105 +
    /// 0.21, run acceleration 0.044 + 0.13167, an initial dash of 12 frames, a heavy landing of 4 (3 of lag: the parameter file's
    /// `landing_heavy_frame`). Air dodges 49 (neutral, intangible 3-30) and 69 / 82 / 108 (directional, intangible 3-21); rolls 30
    /// forward (intangible 4-15) and 35 back, spot dodge 26 (3-17). No wall jump (the parameter file); the wall-tech jump 1.3 away, 2.6
    /// up. The shared body size (the reference's medium size, scale 1.0) [E]. The passive shield (E14): see
    /// [`FighterParams::wanderer_shield`].
    pub fn wanderer() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(96);
        let (burst, arc) = Self::full_hop(gravity, su(27800));
        FighterParams {
            walk_speed: gu(1247),
            run_speed: gu(1534),
            dash_speed: gu(1980),
            walk_accel: gu(105),
            walk_accel_stick: gu(210),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(13167),
            dash_frames: 12,
            // (The shield can cut a dash short from about half way, as for the others [E].)
            dash_shield_frame: 6,
            heavy_landing_lag: 3,
            air_speed: su(924),
            air_accel: su(10),
            air_accel_stick: su(40),
            air_friction: FighterParams::su_fine(380),
            gravity,
            max_fall_speed: su(1600),
            fast_fall_speed: su(3040),
            air_dodge_frames: 49,
            air_dodge_dir_down_frames: 69,
            air_dodge_dir_side_frames: 82,
            air_dodge_dir_up_frames: 108,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.113 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(226),
            dash_attack_friction: gu(113),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(13380)),
            air_jump_velocity: Self::hop_velocity(gravity, su(29000)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(33900)),
            weight: Fx::from_int(104),
            weapon: 9,
            roll_frames: 30,
            roll_back_frames: 35,
            spot_dodge_frames: 26,
            // No wall jump; the reference's wall-tech jump (the parameter file): 1.3 away, 2.6 up.
            wall_jump: 0,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2600),
            passive_shield: Some(Self::wanderer_shield()),
            // Item throws let go on 7, 9, 11, 6, 4, 5 (ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 7,
                back: 9,
                up: 11,
                down: 6,
                dash: 4,
                drop: 5,
            },
            ..FighterParams::base()
        }
    }

    /// The wanderer's shield (E14, the reference's passive shield: ssbwiki.com `Passive_shield` and the character's moveset notes,
    /// 13.0.1): it stops projectiles from in front while standing, walking or crouching [R]. Its place and size are not published
    /// (gap 2) [E]: a capsule 0.6 ahead of the feet with a radius of 0.45 (its front reaches as far as the guard held out in front, so
    /// a shot meets it no later than the body), centred 1.0 up standing (from 0.65 to 1.35: the chest and waist, below the head and
    /// above the feet) and 0.4 up crouching (the shield brought down to the legs, so a shot at face height reaches the head).
    pub fn wanderer_shield() -> PassiveShield {
        PassiveShield {
            x: Fx::from_ratio(6, 10),
            y: Fx::ONE,
            crouch_y: Fx::from_ratio(2, 5),
            half_height: Fx::from_ratio(35, 100),
            radius: Fx::from_ratio(45, 100),
        }
    }
}

impl FighterParams {
    /// A small, floaty middleweight with a pocketful of tools: the "settler" body of the Pockets moveset. The reference character's
    /// 13.0.1 attributes (owner decision 68; the parameter file via dragdown.wiki, ssbwiki.com's `Dash` and `Item_throw` tables,
    /// ultimateframedata.com; the eleventh moveset's `REFERENCE_DATA.md` in `docs/movesets/`): weight 92, walk 1.092, initial
    /// dash 1.815 for 13 frames, run 1.397, traction 0.131, air speed 0.987, air acceleration 0.01 + 0.05, air friction 0.01, gravity
    /// 0.078, fall 1.32 (fast 2.112), full hop 32.5, short hop 15.69, double jump 32.5, ledge jump 36.25 (drift 0.6, the shared value).
    /// On the ground: walk acceleration 0.105 + 0.105, run acceleration 0.044 + 0.10978, a heavy landing of 8 (7 of lag: the parameter
    /// file's `landing_heavy_frame`). Air dodges 58 (neutral, intangible 3-30) and 81 / 97 / 129 (directional down / sideways / up,
    /// intangible 3-21); rolls 30 forward (intangible 4-15) and 35 back, spot dodge 26 (3-17). A wall jump (the parameter file's
    /// `wall_jump_type`) at 1.3 away, 2.3 up. Item throws let go on 11, 11, 10, 10, 5, 6. The shared body size [E].
    pub fn settler() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(78);
        let (burst, arc) = Self::full_hop(gravity, su(32500));
        FighterParams {
            walk_speed: gu(1092),
            run_speed: gu(1397),
            dash_speed: gu(1815),
            walk_accel: gu(105),
            walk_accel_stick: gu(105),
            run_accel: gu(44),
            run_accel_stick: FighterParams::gu_fine(10978),
            dash_frames: 13,
            // (The shield can cut a dash short from about half way, as for the others [E].)
            dash_shield_frame: 7,
            heavy_landing_lag: 7,
            air_speed: su(987),
            air_accel: su(10),
            air_accel_stick: su(50),
            air_friction: su(10),
            gravity,
            max_fall_speed: su(1320),
            fast_fall_speed: su(2112),
            air_dodge_frames: 58,
            air_dodge_dir_down_frames: 81,
            air_dodge_dir_side_frames: 97,
            air_dodge_dir_up_frames: 129,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.131 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(262),
            dash_attack_friction: gu(131),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(15690)),
            air_jump_velocity: Self::hop_velocity(gravity, su(32500)),
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(36250)),
            weight: Fx::from_int(92),
            weapon: 10,
            roll_frames: 30,
            roll_back_frames: 35,
            spot_dodge_frames: 26,
            // A wall jump (the parameter file: 1.3 away, 2.3 up).
            wall_jump: 1,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2300),
            // Item throws let go on 11, 11, 10, 10, 5, 6 (ssbwiki.com `Item_throw`, the grounded ones) [R].
            item_throws: ItemThrows {
                forward: 11,
                back: 11,
                up: 10,
                down: 10,
                dash: 5,
                drop: 6,
            },
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A small, quick middleweight that falls fast and drifts little: the "scamp" body of the Popgun moveset. The reference character's
    /// 13.0.1 attributes (the parameter file via dragdown.wiki, ssbwiki.com's `Dash` and `Item_throw` tables, ultimateframedata.com; the
    /// twelfth moveset's `REFERENCE_DATA.md` in `docs/movesets/popgun/`): weight 90, walk 1.313, initial dash 2.09 for 10 frames, run
    /// 2.006, traction 0.093, air speed 0.924, air acceleration 0 + 0.05, air friction 0.008, gravity 0.125, fall 1.75 (fast 2.8), full
    /// hop 41.21, short hop 19.9, double jump 41.21 (its sideways speed x0.9), ledge jump 40.605 (drift 0.6, the shared value). On the
    /// ground: walk acceleration 0.105 + 0.105, run acceleration 0.044 + 0.165, a heavy landing of 7 (6 of lag: the parameter file's
    /// `landing_heavy_frame`). Air dodges 45 (neutral, intangible 2-27) and 63 / 75 / 97 (directional down / sideways / up, intangible
    /// 2-19); rolls 29 forward (intangible 4-14) and 34 back, spot dodge 24 (3-16). A wall jump (the parameter file's `wall_jump_type`)
    /// at 1.3 away, 2.3 up. Item throws let go on 7, 9, 11, 6, 4, 5. The shared body size [E].
    pub fn scamp() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(125);
        let (burst, arc) = Self::full_hop(gravity, su(41210));
        FighterParams {
            walk_speed: gu(1313),
            run_speed: gu(2006),
            dash_speed: gu(2090),
            walk_accel: gu(105),
            walk_accel_stick: gu(105),
            run_accel: gu(44),
            run_accel_stick: gu(165),
            dash_frames: 10,
            heavy_landing_lag: 6,
            air_speed: su(924),
            air_accel: Fx::ZERO,
            air_accel_stick: su(50),
            air_friction: su(8),
            gravity,
            max_fall_speed: su(1750),
            fast_fall_speed: su(2800),
            air_dodge_frames: 45,
            air_dodge_dir_down_frames: 63,
            air_dodge_dir_side_frames: 75,
            air_dodge_dir_up_frames: 97,
            air_dodge_intangible_start: 2,
            air_dodge_intangible_end: 27,
            air_dodge_dir_intangible_end: 19,
            // Traction 0.093 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(186),
            dash_attack_friction: gu(93),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(19900)),
            air_jump_velocity: Self::hop_velocity(gravity, su(41210)),
            air_jump_drift: 90,
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(40605)),
            weight: Fx::from_int(90),
            weapon: 11,
            roll_frames: 29,
            roll_back_frames: 34,
            roll_intangible_start: 4,
            roll_intangible_end: 14,
            spot_dodge_frames: 24,
            spot_intangible_start: 3,
            spot_intangible_end: 16,
            // A wall jump (the parameter file: 1.3 away, 2.3 up).
            wall_jump: 1,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2300),
            // Item throws let go on 7, 9, 11, 6, 4, 5 (ssbwiki.com `Item_throw`) [R].
            item_throws: ItemThrows {
                forward: 7,
                back: 9,
                up: 11,
                down: 6,
                dash: 4,
                drop: 5,
            },
            ..FighterParams::base()
        }
    }
}

impl FighterParams {
    /// A quick, slender middleweight that falls fast, with a partner that comes when its gauge fills: the "phantom" body of the Dagger
    /// moveset. The reference character's 13.0.1 attributes (the parameter file via dragdown.wiki, ssbwiki.com's `Dash` and
    /// `Item_throw` tables, ultimateframedata.com; the thirteenth moveset's `REFERENCE_DATA.md` in `docs/movesets/dagger/`): weight 93,
    /// walk 1.12, initial dash 1.9 for 10 frames, run 2.06, traction 0.13, air speed 1.1, air acceleration 0.01 + 0.07, air friction
    /// 0.01, gravity 0.127, fall 1.63 (fast 3.097), full hop 32.5, short hop 14.2, double jump 34 (its sideways speed x0.9), ledge jump
    /// 36.25 (drift 0.6, the shared value). On the ground: walk acceleration 0.2 + 0.11, run acceleration 0.04 + 0.11, a heavy landing
    /// of 4 (3 of lag: the parameter file's `landing_heavy_frame`). Air dodges 48 (neutral, intangible 3-30) and 68 / 84 / 101
    /// (directional down / sideways / up, intangible 3-21); rolls 30 forward (intangible 4-15) and 35 back, spot dodge 25 (3-17). A
    /// wall jump (the parameter file's `wall_jump_type`) at 1.3 away, 2.7 up. Item throws let go on the common 8, 11, 12, 7, 5, 6. The
    /// shared body size [E]. Its gauge is [`FighterParams::phantom_gauge`].
    pub fn phantom() -> FighterParams {
        let su = FighterParams::su;
        let gu = FighterParams::gu;
        let gravity = su(127);
        let (burst, arc) = Self::full_hop(gravity, su(32500));
        FighterParams {
            walk_speed: gu(1120),
            run_speed: gu(2060),
            dash_speed: gu(1900),
            walk_accel: gu(200),
            walk_accel_stick: gu(110),
            run_accel: gu(40),
            run_accel_stick: gu(110),
            dash_frames: 10,
            heavy_landing_lag: 3,
            air_speed: su(1100),
            air_accel: su(10),
            air_accel_stick: su(70),
            air_friction: su(10),
            gravity,
            max_fall_speed: su(1630),
            fast_fall_speed: su(3097),
            air_dodge_frames: 48,
            air_dodge_dir_down_frames: 68,
            air_dodge_dir_side_frames: 84,
            air_dodge_dir_up_frames: 101,
            air_dodge_intangible_start: 3,
            air_dodge_intangible_end: 30,
            air_dodge_dir_intangible_end: 21,
            // Traction 0.13 (the doubled stopping friction of ground moves, and the dash attack's slide).
            ground_friction: gu(260),
            dash_attack_friction: gu(130),
            full_hop_velocity: arc,
            hop_burst_velocity: burst,
            hop_burst_frames: HOP_BURST_FRAMES,
            short_hop_velocity: Self::hop_velocity(gravity, su(14200)),
            air_jump_velocity: Self::hop_velocity(gravity, su(34000)),
            air_jump_drift: 90,
            ledge_jump_velocity: Self::ledge_jump_velocity_for(gravity, su(36250)),
            weight: Fx::from_int(93),
            weapon: 12,
            roll_frames: 30,
            roll_back_frames: 35,
            roll_intangible_start: 4,
            roll_intangible_end: 15,
            spot_dodge_frames: 25,
            spot_intangible_start: 3,
            spot_intangible_end: 17,
            // A wall jump (the parameter file: 1.3 away, 2.7 up).
            wall_jump: 1,
            wall_jump_vx: su(1300),
            wall_jump_vy: su(2700),
            gauge: Self::phantom_gauge(),
            ..FighterParams::base()
        }
    }

    /// The phantom's gauge, its partner's (the reference's, ssbwiki.com's article on the gauge and ultimateframedata.com, 13.0.1; E53):
    /// 100 units; 0.00925 a frame by itself (not on a revival platform); 1.3 a percent of damage taken (with the one-on-one
    /// multiplier: the damage actually added), x1.25 one stock behind the leader and x1.5 two or more (the reference's 1.625 and 1.95),
    /// x(1.1, 1.1667) with three fighters and x(1.2, 1.3333) with four when behind. Damage dealt (the phantom's and its partner's hits on
    /// fighters, not shields or the curse's ticks) fills it 1.3 a percent too, with no stock-gap or crowd multiplier: a house rule (owner
    /// decision 80, sim 123), not the reference's [E]. Full, the partner comes for
    /// 1800 frames; while it is out nothing fills the gauge, and each percent taken cuts 16 frames (10.2 with four fighters); a stun (a
    /// broken shield) stops the clock. Its specials do not spend it. A KO leaves 20 units (25 two stocks behind, 30 three or more). The
    /// body's own numbers do not change with it (no multipliers) [R]. The guard's gain is its move's (`Move::gauge_gain`).
    pub fn phantom_gauge() -> Gauge {
        Gauge {
            max: Fx::from_int(100),
            per_dealt: Fx::from_ratio(13, 10),
            per_taken: Fx::from_ratio(13, 10),
            mode_frames: 1800,
            per_frame: Fx::from_ratio(925, 100_000),
            behind: [1250, 1500],
            crowd: [[1000, 1100, 1167], [1000, 1200, 1333]],
            mode_taken: [Fx::from_int(16), Fx::from_int(16), Fx::from_ratio(102, 10)],
            ko_keep: [Fx::from_int(20), Fx::from_int(25), Fx::from_int(30)],
            keeps: true,
            halts: true,
            ..Gauge::NONE
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
        self.gauge.hash_into(h);
        match &self.air_jump_armor {
            Some(a) => {
                h.write_bool(true);
                h.write_u8(a.start);
                h.write_u8(a.end);
                h.write_u8(a.kind);
                a.threshold.hash_into(h);
                h.write_bool(a.subtract);
                a.min_kb.hash_into(h);
            }
            None => h.write_bool(false),
        }
        h.write_bool(self.air_jump_turns);
        match &self.passive_shield {
            Some(s) => {
                h.write_bool(true);
                for v in [s.x, s.y, s.crouch_y, s.half_height, s.radius] {
                    v.hash_into(h);
                }
            }
            None => h.write_bool(false),
        }
        h.write_u8(self.air_jump_drift);
        for (_, v) in self.item_throws.frames() {
            h.write_u8(v);
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
    /// A slope (sim 124, R2a): the top's height at `right` minus its height at `left` (`y` is the top at `left`); 0 is flat.
    /// At most half the width (about 26.6 degrees; `sim_content` checks it). A sloped surface carries a grounded fighter along
    /// it at its ground speed, its feet on the surface ([`Platform::top_at`]); ledges hang only from flat corners. Only the run's
    /// rooms use slopes for now: every versus stage is flat, and flat surfaces behave exactly as before.
    pub rise: Fx,
}

impl Platform {
    /// The top's height at `x` (held to the platform's span): `y` on a flat platform.
    pub fn top_at(&self, x: Fx) -> Fx {
        if self.rise == Fx::ZERO {
            return self.y;
        }
        let width = self.right - self.left;
        if width <= Fx::ZERO {
            return self.y;
        }
        let t = x.clamp(self.left, self.right) - self.left;
        self.y + self.rise * t / width
    }

    /// The top's height at the right end.
    pub fn top_right(&self) -> Fx {
        self.y + self.rise
    }

    /// The highest point of the top.
    pub fn top_max(&self) -> Fx {
        self.y.max(self.y + self.rise)
    }

    pub fn sloped(&self) -> bool {
        self.rise != Fx::ZERO
    }
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
                    rise: Fx::ZERO,
                },
                Platform {
                    left: int(-8),
                    right: int(-3),
                    y: Fx::from_ratio(18, 5),
                    bottom: Fx::from_ratio(18, 5),
                    pass_through: true,
                    rise: Fx::ZERO,
                },
                Platform {
                    left: int(3),
                    right: int(8),
                    y: Fx::from_ratio(18, 5),
                    bottom: Fx::from_ratio(18, 5),
                    pass_through: true,
                    rise: Fx::ZERO,
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
            p.rise.hash_into(h);
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
    /// A shield stays up at least this many frames, even if the button is let go at once (the reference's 3).
    pub shield_min_frames: u8,
    /// A grab pressed during shield stun keeps the shield up this many more frames before the grab starts (the reference's 4).
    pub shield_grab_delay: u8,
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
    /// A platform drop (a down flick on a pass-through platform) stays on the platform for this many frames after the flick's
    /// frame before the fighter falls, and a special pressed on any of them, or on the frame it would fall, comes out grounded
    /// instead (the down special, with the stick down). The reference's drop is 2 frames (derived from a fighter of its whose
    /// drop is 11 with 9 extra frames added for command specials; owner decision 63, `docs/movesets/ROSTER_STATUS.md`).
    pub platform_drop_frames: u8,
    /// Whether a down flick on a pass-through platform drops through it (1) or not (0). Owner decision 2026-10-14 (sim 112): 0, the
    /// drop is shield + special instead, so down tilts, down smashes, crouches and spot dodges work on platforms.
    pub stick_platform_drop: u8,
    /// Dodge staling (sim 112, SmashWiki "Dodge staling", 13.0.1): each roll or spot dodge out of a shield and each air dodge makes
    /// dodges one level staler, up to `dodge_stale_max`. A level comes off `dodge_stale_first` frames after the last dodge ended and
    /// another every `dodge_stale_every` frames after that (fresh again from full in about 8 seconds). Per level: spot dodges and
    /// forward rolls last `dodge_stale_spot` / `dodge_stale_roll` percent longer, back rolls `dodge_stale_back_roll` percent; fully
    /// stale, their intangibility starts `dodge_stale_spot_shift` / `dodge_stale_roll_shift` frames later (in between, by level);
    /// a directional air dodge travels `dodge_stale_air_cut` less (as a share) per level.
    pub dodge_stale_max: u8,
    pub dodge_stale_first: u8,
    pub dodge_stale_every: u8,
    pub dodge_stale_spot: u8,
    pub dodge_stale_roll: u8,
    pub dodge_stale_back_roll: u8,
    pub dodge_stale_spot_shift: u8,
    pub dodge_stale_roll_shift: u8,
    pub dodge_stale_air_cut: Fx,
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
    /// Each wall jump since the fighter last stood on the ground or held a ledge keeps this share of the one before (the
    /// reference's 0.96, compounding), so walls cannot be climbed forever.
    pub wall_jump_decay: Fx,
    /// An initial dash's interrupt frame, the same for every fighter (the reference's 15): a dash that ends without becoming a run
    /// (the stick let go) stays in the dash, slowing, until this many frames after it began; only then can a new dash start, either
    /// way (a foxtrot, or a late dash back). A flick made meanwhile and still held comes out on that frame (sim 103).
    pub dash_interrupt_frame: u8,
    pub wall_tech_invuln: u8,
    /// A tumbling fighter that hits a wall, ceiling or (fast enough) the floor without teching bounces off with this share of its speed.
    pub bounce_keep: Fx,
    pub ground_bounce_speed: Fx,
    /// A knocked-out fighter comes back this far above its spawn point, standing on a revival platform for up to `respawn_platform_frames` frames (or until it moves), invincible.
    pub respawn_height: Fx,
    pub respawn_platform_frames: u8,
    /// A trip (a hit's `Hitbox::trip` chance coming up, sim 86): the fighter sits on the ground, intangible for its first
    /// `trip_invuln` frames, can choose a get-up (stand, roll or the get-up attack) from frame `trip_lag`, and stands up by itself
    /// on frame `trip_max` (the reference's "hard trip": options from frame 40, a neutral get-up on frame 100, 5 intangible frames).
    pub trip_lag: u8,
    pub trip_max: u8,
    pub trip_invuln: u8,
    /// The reference's base trip chance (owner decision 50): a hit of more than `trip_base_kb` knockback that would not lift a
    /// grounded fighter trips it `trip_base` percent of the time, and a hit with a trip chance of its own adds this to it (7 and 55).
    /// 0 turns the base chance off.
    pub trip_base: u8,
    pub trip_base_kb: Fx,
    /// A soft trip (E44, sim 117: the reference's slip effect from walking into a lying banana peel, `Hitbox::slip` soft): as a
    /// trip, with no hitlag, get-up options from frame `slip_lag` and a get-up by itself on frame `slip_max` (30 and 90: the
    /// reference's soft trip, options "after 29 frames", the neutral get-up "on the 90th frame", ssbwiki.com's trip pages).
    pub slip_lag: u8,
    pub slip_max: u8,
    /// The bury (E5, a hit's `EFFECT_BURY`; the reference's rules, ssbwiki.com "Buried", 13.0.1): a grounded fighter is buried for
    /// `bury_base + bury_percent x p + bury_kb x k` frames rounded up (p its percent after the hit, k the hit's knockback at weight
    /// 100: 55, 0.5, 1.5), `bury_behind_1` / `bury_behind_2` more when the attacker is one / two or more stocks behind (8, 12) and
    /// `bury_ahead_1` / `bury_ahead_2` fewer when it is ahead (18, 22). Mashing cuts it as a grab's hold. A hit of at least
    /// `bury_break_kb` knockback (90) frees and launches it; a weaker one's knockback is kept for when the bury ends. Out of a bury
    /// with nothing kept, the fighter has `bury_exit_lag` frames (12) of lag. While buried its hurtboxes are `bury_depth` of its
    /// height lower [E].
    pub bury_base: Fx,
    pub bury_percent: Fx,
    pub bury_kb: Fx,
    pub bury_behind_1: u8,
    pub bury_behind_2: u8,
    pub bury_ahead_1: u8,
    pub bury_ahead_2: u8,
    pub bury_break_kb: Fx,
    pub bury_exit_lag: u8,
    pub bury_depth: Fx,
    /// Items (E4, sim 111; the reference's universal item rules, ssbwiki.com `Item_throw`, 13.0.1). Picking an item up from the ground
    /// takes `item_pick_frames` frames, the item in hand from the first [E: not published]. The throws last `item_side_frames` (forward
    /// and back, smash or not, 24), `item_up_frames` (23), `item_down_frames` (19), `item_dash_frames` (the dash throw, 29) and
    /// `item_drop_frames` (the grounded drop, 23) [R: the 13.0.1 rule, every fighter alike], the item leaving on the body's
    /// `FighterParams::item_throws` frame. Speeds (world units a frame, the reference's over 8) and angles (degrees above forward): the
    /// forward and back throws `item_side_speed` (2.6) at `item_side_angle` (17), smash `item_side_smash` (3.64) at
    /// `item_side_smash_angle` (11); up `item_up_speed` / `item_up_smash` (3.1 / 4.1) straight up; down `item_down_speed` /
    /// `item_down_smash` (3.3 / 4.62) straight down; the dash throw `item_dash_speed` (3.4) at `item_dash_angle` (12); the grounded drop
    /// `item_drop_speed` (2.1) at `item_drop_angle` (100: up and a little behind) [R]. Reach to pick up: `item_reach_front` ahead of the
    /// feet (13.5), `item_reach_back` behind (6.79) and `item_reach_up` up (12.75) [R: the wanderer's reference values, used for every
    /// body: E]. A held item sits `item_hand_x` ahead of the feet and `item_hand_y` up [E: presentation only].
    pub item_pick_frames: u8,
    pub item_side_frames: u8,
    pub item_up_frames: u8,
    pub item_down_frames: u8,
    pub item_dash_frames: u8,
    pub item_drop_frames: u8,
    pub item_side_speed: Fx,
    pub item_side_smash: Fx,
    pub item_side_angle: i16,
    pub item_side_smash_angle: i16,
    pub item_up_speed: Fx,
    pub item_up_smash: Fx,
    pub item_down_speed: Fx,
    pub item_down_smash: Fx,
    pub item_dash_speed: Fx,
    pub item_dash_angle: i16,
    pub item_drop_speed: Fx,
    pub item_drop_angle: i16,
    pub item_reach_front: Fx,
    pub item_reach_back: Fx,
    pub item_reach_up: Fx,
    pub item_hand_x: Fx,
    pub item_hand_y: Fx,
    /// The previous reference version's (1.1.x) numbers for a hit with `Hitbox::rules` `RULES_PREVIOUS` (E40, sim 113, owner decisions
    /// 66-67; ssbwiki.com `Hitlag`, `Hitstun`, `Knockback` and `Rage`): it multiplies damage by `prev_damage_mult` (1: that version had no
    /// one-on-one multiplier), its hitlag is `floor(d x prev_hitlag_per_damage + prev_hitlag_base)` times the hit's factors (0.3846154
    /// and 5), a crouch shortening only the victim's; rage reaches `prev_rage_max` (0.15) over the same percents; a tumbling launch
    /// (`tumble_knockback` and more) gets `prev_tumble_hitstun` (1) more frame of hitstun.
    pub prev_damage_mult: Fx,
    pub prev_hitlag_per_damage: Fx,
    pub prev_hitlag_base: Fx,
    pub prev_rage_max: Fx,
    pub prev_tumble_hitstun: u8,
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
            shield_min_frames: 3,
            shield_grab_delay: 4,
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
            platform_drop_frames: 2,
            stick_platform_drop: 0,
            dodge_stale_max: 5,
            dodge_stale_first: 60,
            dodge_stale_every: 120,
            dodge_stale_spot: 6,
            dodge_stale_roll: 6,
            dodge_stale_back_roll: 10,
            dodge_stale_spot_shift: 3,
            dodge_stale_roll_shift: 4,
            dodge_stale_air_cut: Fx::from_ratio(1, 15),
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
            wall_jump_decay: Fx::from_ratio(96, 100),
            dash_interrupt_frame: 15,
            wall_tech_invuln: 14,
            bounce_keep: Fx::from_ratio(95, 100),
            ground_bounce_speed: Fx::from_ratio(1, 4),
            respawn_height: Fx::from_int(7),
            respawn_platform_frames: 150,
            trip_lag: 40,
            trip_max: 100,
            trip_invuln: 5,
            trip_base: 7,
            trip_base_kb: Fx::from_int(55),
            slip_lag: 30,
            slip_max: 90,
            bury_base: Fx::from_int(55),
            bury_percent: Fx::from_ratio(1, 2),
            bury_kb: Fx::from_ratio(3, 2),
            bury_behind_1: 8,
            bury_behind_2: 12,
            bury_ahead_1: 18,
            bury_ahead_2: 22,
            bury_break_kb: Fx::from_int(90),
            bury_exit_lag: 12,
            bury_depth: Fx::from_ratio(1, 2),
            item_pick_frames: 8,
            item_side_frames: 24,
            item_up_frames: 23,
            item_down_frames: 19,
            item_dash_frames: 29,
            item_drop_frames: 23,
            item_side_speed: Fx::from_ratio(26, 80),
            item_side_smash: Fx::from_ratio(364, 800),
            item_side_angle: 17,
            item_side_smash_angle: 11,
            item_up_speed: Fx::from_ratio(31, 80),
            item_up_smash: Fx::from_ratio(41, 80),
            item_down_speed: Fx::from_ratio(33, 80),
            item_down_smash: Fx::from_ratio(462, 800),
            item_dash_speed: Fx::from_ratio(34, 80),
            item_dash_angle: 12,
            item_drop_speed: Fx::from_ratio(21, 80),
            item_drop_angle: 100,
            item_reach_front: Fx::from_ratio(135, 80),
            item_reach_back: Fx::from_ratio(95, 112),
            item_reach_up: Fx::from_ratio(1275, 800),
            item_hand_x: Fx::from_ratio(1, 2),
            item_hand_y: Fx::from_ratio(9, 10),
            prev_damage_mult: Fx::ONE,
            prev_hitlag_per_damage: Fx::from_ratio(3_846_154, 10_000_000),
            prev_hitlag_base: Fx::from_int(5),
            prev_rage_max: Fx::from_ratio(15, 100),
            prev_tumble_hitstun: 1,
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
        h.write_u8(self.shield_min_frames);
        h.write_u8(self.shield_grab_delay);
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
        h.write_u8(self.platform_drop_frames);
        h.write_u8(self.stick_platform_drop);
        h.write_u8(self.dodge_stale_max);
        h.write_u8(self.dodge_stale_first);
        h.write_u8(self.dodge_stale_every);
        h.write_u8(self.dodge_stale_spot);
        h.write_u8(self.dodge_stale_roll);
        h.write_u8(self.dodge_stale_back_roll);
        h.write_u8(self.dodge_stale_spot_shift);
        h.write_u8(self.dodge_stale_roll_shift);
        self.dodge_stale_air_cut.hash_into(h);
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
        self.wall_jump_decay.hash_into(h);
        h.write_u8(self.dash_interrupt_frame);
        h.write_u8(self.wall_tech_invuln);
        self.bounce_keep.hash_into(h);
        self.ground_bounce_speed.hash_into(h);
        self.respawn_height.hash_into(h);
        h.write_u8(self.respawn_platform_frames);
        h.write_u8(self.trip_lag);
        h.write_u8(self.trip_max);
        h.write_u8(self.trip_invuln);
        h.write_u8(self.trip_base);
        self.trip_base_kb.hash_into(h);
        h.write_u8(self.slip_lag);
        h.write_u8(self.slip_max);
        self.bury_base.hash_into(h);
        self.bury_percent.hash_into(h);
        self.bury_kb.hash_into(h);
        h.write_u8(self.bury_behind_1);
        h.write_u8(self.bury_behind_2);
        h.write_u8(self.bury_ahead_1);
        h.write_u8(self.bury_ahead_2);
        self.bury_break_kb.hash_into(h);
        h.write_u8(self.bury_exit_lag);
        self.bury_depth.hash_into(h);
        h.write_u8(self.item_pick_frames);
        h.write_u8(self.item_side_frames);
        h.write_u8(self.item_up_frames);
        h.write_u8(self.item_down_frames);
        h.write_u8(self.item_dash_frames);
        h.write_u8(self.item_drop_frames);
        self.item_side_speed.hash_into(h);
        self.item_side_smash.hash_into(h);
        h.write_u16(self.item_side_angle as u16);
        h.write_u16(self.item_side_smash_angle as u16);
        self.item_up_speed.hash_into(h);
        self.item_up_smash.hash_into(h);
        self.item_down_speed.hash_into(h);
        self.item_down_smash.hash_into(h);
        self.item_dash_speed.hash_into(h);
        h.write_u16(self.item_dash_angle as u16);
        self.item_drop_speed.hash_into(h);
        h.write_u16(self.item_drop_angle as u16);
        self.item_reach_front.hash_into(h);
        self.item_reach_back.hash_into(h);
        self.item_reach_up.hash_into(h);
        self.item_hand_x.hash_into(h);
        self.item_hand_y.hash_into(h);
        self.prev_damage_mult.hash_into(h);
        self.prev_hitlag_per_damage.hash_into(h);
        self.prev_hitlag_base.hash_into(h);
        self.prev_rage_max.hash_into(h);
        h.write_u8(self.prev_tumble_hitstun);
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

    /// The built-in fighters (a swordfighter, a close-range brawler, a quick rogue, an all-round tumbler, a fast, heavy sprinter, a
    /// tall caster, a champion with a greatsword, a huge hauler with bracers, a floaty, high-jumping bounder with a long tongue and a
    /// wanderer with a sword and a buckler, a settler with pockets full of tools, a scamp with a popgun, and a phantom with a dagger) on a main stage with
    /// platforms.
    pub fn placeholder() -> Content {
        Content {
            fighters: vec![
                FighterParams::duelist(),
                FighterParams::brawler(),
                FighterParams::rogue(),
                FighterParams::tumbler(),
                FighterParams::sprinter(),
                FighterParams::seer(),
                FighterParams::champion(),
                FighterParams::hauler(),
                FighterParams::bounder(),
                FighterParams::wanderer(),
                FighterParams::settler(),
                FighterParams::scamp(),
                FighterParams::phantom(),
            ],
            weapons: vec![
                crate::moves::longsword(),
                crate::moves::claws(),
                crate::moves::needles(),
                crate::moves::embers(),
                crate::moves::gauntlets(),
                crate::moves::staff(),
                crate::moves::greatsword(),
                crate::moves::bracers(),
                crate::moves::clutch(),
                crate::moves::buckler(),
                crate::moves::pockets(),
                crate::moves::popgun(),
                crate::moves::dagger(),
            ],
            stage: Stage::placeholder(),
            rules: Ruleset::standard(),
            look: StageLook::default(),
            names: Names {
                fighters: vec![
                    "duelist".to_string(),
                    "brawler".to_string(),
                    "rogue".to_string(),
                    "tumbler".to_string(),
                    "sprinter".to_string(),
                    "seer".to_string(),
                    "champion".to_string(),
                    "hauler".to_string(),
                    "bounder".to_string(),
                    "wanderer".to_string(),
                    "settler".to_string(),
                    "scamp".to_string(),
                    "phantom".to_string(),
                ],
                weapons: vec![
                    "longsword".to_string(),
                    "claws".to_string(),
                    "needles".to_string(),
                    "embers".to_string(),
                    "gauntlets".to_string(),
                    "staff".to_string(),
                    "greatsword".to_string(),
                    "bracers".to_string(),
                    "clutch".to_string(),
                    "buckler".to_string(),
                    "pockets".to_string(),
                    "popgun".to_string(),
                    "dagger".to_string(),
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

        // The air jump's armor and turnaround (E33), every field of the armor.
        let mut c = base.clone();
        c.fighters[0].air_jump_turns = true;
        assert_ne!(h, c.hash());
        let mut c = base.clone();
        c.fighters[0].air_jump_drift = 120;
        assert_ne!(h, c.hash());
        let armor = FighterParams::bounder_air_jump_armor();
        let mut with = base.clone();
        with.fighters[0].air_jump_armor = Some(armor);
        assert_ne!(h, with.hash());
        let hw = with.hash();
        for change in [
            |a: &mut crate::moves::Armor| a.start += 1,
            |a: &mut crate::moves::Armor| a.end += 1,
            |a: &mut crate::moves::Armor| a.kind = crate::moves::ARMOR_SUPER,
            |a: &mut crate::moves::Armor| a.threshold += Fx::from_raw(1),
            |a: &mut crate::moves::Armor| a.subtract = false,
            |a: &mut crate::moves::Armor| a.min_kb += Fx::from_raw(1),
        ] {
            let mut c = with.clone();
            let mut a = armor;
            change(&mut a);
            c.fighters[0].air_jump_armor = Some(a);
            assert_ne!(hw, c.hash());
        }
        // A move armor's new fields too.
        let mut c = with.clone();
        let gauntlet_armor = c.weapons[4]
            .moves
            .iter()
            .position(|m| m.armor.is_some())
            .expect("a move with armor");
        if let Some(a) = c.weapons[4].moves[gauntlet_armor].armor.as_mut() {
            a.min_kb += Fx::from_raw(1);
        }
        assert_ne!(hw, c.hash());

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
