//! Attack data: moves, hitboxes and weapons. A character's moveset comes from its **weapon**,
//! so a character is a body type (physics) plus a weapon (this file) plus cosmetics.
//!
//! Conventions:
//! * `Hitbox::start`/`end`, `Move::total_frames` and the autocancel fields count in **move frames**: the frame
//!   the move begins on is move frame 0 (`Fighter::state_frame`). The reference's published frame data
//!   numbers the first frame 1, so the [`R`]-row builders convert (`first_active - 1`, `FAF - 2`).
//! * Positions are offsets from the fighter's feet, with +x in the direction the fighter faces.
//! * Damage and knockback use the reference conventions (percent, base knockback, knockback growth).
//!
//! Moves built from reference frame data are marked below. Everything else is a placeholder to tune.
//! Hitbox *positions and sizes* are not published in the sources used, so those are estimates.

use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::vec2::Vec2;
use sim_script::{Kind, Program};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveId {
    Jab = 0,
    FTilt,
    UTilt,
    DTilt,
    DashAttack,
    FSmash,
    USmash,
    DSmash,
    NAir,
    FAir,
    BAir,
    UAir,
    DAir,
    NSpecial,
    SideSpecial,
    UpSpecial,
    DownSpecial,
    /// Second and third hits of a jab combo (started by the previous jab, never by a button).
    Jab2,
    Jab3,
    /// Standing grab (also what a shield grab does) and the grab from a dash.
    Grab,
    DashGrab,
    /// A hit on the fighter being held, and the four throws.
    Pummel,
    FThrow,
    BThrow,
    UThrow,
    DThrow,
    /// Attack out of a ledge hang, and attack out of a knockdown.
    LedgeAttack,
    GetUpAttack,
    /// Ten free slots for follow-up moves that only a script (`goto`) or a counter reaches: the later hits of a
    /// multi-hit special, the counter-attack, and so on. They are never started by a button.
    Ext0,
    Ext1,
    Ext2,
    Ext3,
    Ext4,
    Ext5,
    Ext6,
    Ext7,
    Ext8,
    Ext9,
    /// A grab out of a dash or run turned around: the grab comes out behind the direction of travel.
    PivotGrab,
}

impl MoveId {
    pub const COUNT: usize = 39;
    /// Index of the first special move.
    pub const FIRST_SPECIAL: u8 = 13;

    pub const fn is_aerial(self) -> bool {
        matches!(
            self,
            MoveId::NAir | MoveId::FAir | MoveId::BAir | MoveId::UAir | MoveId::DAir
        )
    }

    pub const fn is_special(self) -> bool {
        matches!(
            self,
            MoveId::NSpecial | MoveId::SideSpecial | MoveId::UpSpecial | MoveId::DownSpecial
        )
    }

    pub const fn is_ext(self) -> bool {
        (self as u8) >= 28 && (self as u8) <= 37
    }

    /// Slots a weapon may leave empty: specials it does not have, jab hits it does not chain into, and the
    /// script follow-up slots.
    pub const fn may_be_empty(self) -> bool {
        self.is_special()
            || self.is_ext()
            || matches!(self, MoveId::Jab2 | MoveId::Jab3 | MoveId::PivotGrab)
    }

    pub const fn from_index(i: u8) -> MoveId {
        match i {
            0 => MoveId::Jab,
            1 => MoveId::FTilt,
            2 => MoveId::UTilt,
            3 => MoveId::DTilt,
            4 => MoveId::DashAttack,
            5 => MoveId::FSmash,
            6 => MoveId::USmash,
            7 => MoveId::DSmash,
            8 => MoveId::NAir,
            9 => MoveId::FAir,
            10 => MoveId::BAir,
            11 => MoveId::UAir,
            12 => MoveId::DAir,
            13 => MoveId::NSpecial,
            14 => MoveId::SideSpecial,
            15 => MoveId::UpSpecial,
            17 => MoveId::Jab2,
            18 => MoveId::Jab3,
            19 => MoveId::Grab,
            20 => MoveId::DashGrab,
            21 => MoveId::Pummel,
            22 => MoveId::FThrow,
            23 => MoveId::BThrow,
            24 => MoveId::UThrow,
            25 => MoveId::DThrow,
            26 => MoveId::LedgeAttack,
            27 => MoveId::GetUpAttack,
            28 => MoveId::Ext0,
            29 => MoveId::Ext1,
            30 => MoveId::Ext2,
            31 => MoveId::Ext3,
            32 => MoveId::Ext4,
            33 => MoveId::Ext5,
            34 => MoveId::Ext6,
            35 => MoveId::Ext7,
            36 => MoveId::Ext8,
            37 => MoveId::Ext9,
            38 => MoveId::PivotGrab,
            _ => MoveId::DownSpecial,
        }
    }

    /// Every move slot, in index order.
    pub fn all() -> impl Iterator<Item = MoveId> {
        (0..Self::COUNT as u8).map(MoveId::from_index)
    }

    /// The identifier content files use for this slot (`dash_attack`, `up_special`, ...).
    pub const fn key(self) -> &'static str {
        match self {
            MoveId::Jab => "jab",
            MoveId::FTilt => "ftilt",
            MoveId::UTilt => "utilt",
            MoveId::DTilt => "dtilt",
            MoveId::DashAttack => "dash_attack",
            MoveId::FSmash => "fsmash",
            MoveId::USmash => "usmash",
            MoveId::DSmash => "dsmash",
            MoveId::NAir => "nair",
            MoveId::FAir => "fair",
            MoveId::BAir => "bair",
            MoveId::UAir => "uair",
            MoveId::DAir => "dair",
            MoveId::NSpecial => "neutral_special",
            MoveId::SideSpecial => "side_special",
            MoveId::UpSpecial => "up_special",
            MoveId::DownSpecial => "down_special",
            MoveId::Jab2 => "jab2",
            MoveId::Jab3 => "jab3",
            MoveId::Grab => "grab",
            MoveId::DashGrab => "dash_grab",
            MoveId::Pummel => "pummel",
            MoveId::FThrow => "fthrow",
            MoveId::BThrow => "bthrow",
            MoveId::UThrow => "uthrow",
            MoveId::DThrow => "dthrow",
            MoveId::LedgeAttack => "ledge_attack",
            MoveId::GetUpAttack => "get_up_attack",
            MoveId::Ext0 => "ext0",
            MoveId::Ext1 => "ext1",
            MoveId::Ext2 => "ext2",
            MoveId::Ext3 => "ext3",
            MoveId::Ext4 => "ext4",
            MoveId::Ext5 => "ext5",
            MoveId::Ext6 => "ext6",
            MoveId::Ext7 => "ext7",
            MoveId::Ext8 => "ext8",
            MoveId::Ext9 => "ext9",
            MoveId::PivotGrab => "pivot_grab",
        }
    }

    pub fn from_key(key: &str) -> Option<MoveId> {
        MoveId::all().find(|m| m.key() == key)
    }

    pub const fn name(self) -> &'static str {
        match self {
            MoveId::Jab => "jab",
            MoveId::FTilt => "ftilt",
            MoveId::UTilt => "utilt",
            MoveId::DTilt => "dtilt",
            MoveId::DashAttack => "dash attack",
            MoveId::FSmash => "fsmash",
            MoveId::USmash => "usmash",
            MoveId::DSmash => "dsmash",
            MoveId::NAir => "nair",
            MoveId::FAir => "fair",
            MoveId::BAir => "bair",
            MoveId::UAir => "uair",
            MoveId::DAir => "dair",
            MoveId::NSpecial => "neutral special",
            MoveId::SideSpecial => "side special",
            MoveId::UpSpecial => "up special",
            MoveId::DownSpecial => "down special",
            MoveId::Jab2 => "jab 2",
            MoveId::Jab3 => "jab 3",
            MoveId::Grab => "grab",
            MoveId::DashGrab => "dash grab",
            MoveId::Pummel => "pummel",
            MoveId::FThrow => "forward throw",
            MoveId::BThrow => "back throw",
            MoveId::UThrow => "up throw",
            MoveId::DThrow => "down throw",
            MoveId::LedgeAttack => "ledge attack",
            MoveId::GetUpAttack => "get-up attack",
            MoveId::Ext0 => "extra move 0",
            MoveId::Ext1 => "extra move 1",
            MoveId::Ext2 => "extra move 2",
            MoveId::Ext3 => "extra move 3",
            MoveId::Ext4 => "extra move 4",
            MoveId::Ext5 => "extra move 5",
            MoveId::Ext6 => "extra move 6",
            MoveId::Ext7 => "extra move 7",
            MoveId::Ext8 => "extra move 8",
            MoveId::Ext9 => "extra move 9",
            MoveId::PivotGrab => "pivot grab",
        }
    }
}

/// One circular hitbox, active for move frames `start..=end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hitbox {
    pub start: u8,
    pub end: u8,
    pub x: Fx,
    pub y: Fx,
    pub radius: Fx,
    /// Percent of damage dealt.
    pub damage: Fx,
    /// Launch angle in degrees, 0 = forward. 361 is the reference's 361 angle (see `combat::launch_angle`), 367 the autolink angle,
    /// 368 the pull angle.
    pub angle: i16,
    pub base_knockback: i16,
    pub knockback_growth: i16,
    /// When several hitboxes of one move overlap a target in the same frame, the lowest priority number
    /// wins. A sword's tip is priority 0 and its hilt priority 1, which is what makes spacing matter.
    pub priority: u8,
    /// A move can hit the same target once per group, which is how a move gets separate hits
    /// (group 0 and group 1). Hitboxes in one group never hit a target twice.
    pub group: u8,
    /// What the hitbox does: a normal hit, a grab, or (while holding someone) a throw or pummel.
    pub kind: u8,
    /// Percent of its damage this hit does to a shield (100 is normal; a shield-breaking move uses more).
    pub shield_damage: u16,
    /// Percent of the normal hitlag this hit causes (100 is normal; 0 gives none).
    pub hitlag: u16,
    /// What the hit is made of: [`EFFECT_NORMAL`] or [`EFFECT_ELECTRIC`] (more hitlag, and the victim drifts with the held stick).
    pub effect: u8,
    /// Who it can hit: [`TARGETS_ALL`], or only fighters on the ground ([`TARGETS_GROUNDED`]) or in the air ([`TARGETS_AERIAL`]).
    pub targets: u8,
    /// Set knockback (the reference's "fixed knockback value"): when above 0 the knockback is worked out as if the victim's percent
    /// were this, whatever it really is, so the hit sends the same distance at any percent. 0 for normal knockback.
    pub fixed_kb: u16,
    /// Extra hitstun frames this hit adds (the reference's hitstun modifier).
    pub hitstun_add: u8,
    /// Percent of the normal survival-DI distance the victim gets during this hit's hitlag (100 is normal).
    pub sdi: u16,
    /// For the autolink angle: the pull aims this far in front of the hitbox (world units, the way the hit sends), so a carried victim
    /// drifts in front of the attacker instead of staying where the hitbox is. 0 aims at the hitbox itself (the reference).
    pub link_ahead: Fx,
    /// Which way the move must be aimed for this hitbox to be out: [`AIM_ANY`] (always), or only when the move was aimed up
    /// ([`AIM_UP`]), straight ([`AIM_STRAIGHT`]) or down ([`AIM_DOWN`]) as it started (the reference's angled forward tilts and
    /// smashes: see `Fighter::aim`).
    pub aim: u8,
    /// The reference's "set weight": the knockback is worked out as if the victim weighed 100, whatever its weight, so a multi-hit
    /// holds light and heavy fighters alike.
    pub set_weight: bool,
    /// The reference's "reverse" effect: the hit turns the victim round (it ends up facing away from the attacker, its horizontal
    /// speed reversed), as well as launching it.
    pub reverse: bool,
    /// A push (the reference's flinchless, windbox-like hit): the knockback becomes a shove added to the victim's speed (only its
    /// horizontal part on the ground) with no hitlag, no hitstun and no change of state: the victim keeps control. A shield takes it
    /// as pushback.
    pub push: bool,
}

/// `Hitbox::aim` values.
pub const AIM_ANY: u8 = 0;
pub const AIM_UP: u8 = 1;
pub const AIM_STRAIGHT: u8 = 2;
pub const AIM_DOWN: u8 = 3;

/// `Hitbox::targets` values.
pub const TARGETS_ALL: u8 = 0;
pub const TARGETS_GROUNDED: u8 = 1;
pub const TARGETS_AERIAL: u8 = 2;

/// The autolink angle (the reference's 367): an airborne victim is launched at the attacker's velocity plus a pull toward the
/// hitbox (see `combat::autolink`), so a multi-hit carries it along; a grounded one is sent at 80 degrees.
pub const ANGLE_AUTOLINK: i16 = 367;

/// The pull angle (ours, not the reference's): instead of a launch, the victim is drawn to just in front of the attacker (a
/// projectile's owner) while its hitstun lasts, `pull_gap` from it (see `fighter::pull_frame`). Its hitstun comes from the
/// knockback as usual (set knockback makes it the same at any percent).
pub const ANGLE_PULL: i16 = 368;

impl Hitbox {
    /// The fields added after the first release of the format, at their defaults (what older content means).
    pub const fn extras(mut self) -> Hitbox {
        self.targets = TARGETS_ALL;
        self.fixed_kb = 0;
        self.hitstun_add = 0;
        self.sdi = 100;
        self.link_ahead = Fx::ZERO;
        self.aim = AIM_ANY;
        self.set_weight = false;
        self.reverse = false;
        self.push = false;
        self
    }
}

/// `Hitbox::effect` values.
pub const EFFECT_NORMAL: u8 = 0;
pub const EFFECT_ELECTRIC: u8 = 1;

/// `Hitbox::kind` values.
pub const HIT_NORMAL: u8 = 0;
/// Catches a grounded fighter instead of damaging it. Shields do not stop it.
pub const HIT_GRAB: u8 = 1;
/// Releases the fighter being held with this hit's damage and launch. It needs no overlap.
pub const HIT_THROW: u8 = 2;
/// Damages the fighter being held without releasing it.
pub const HIT_PUMMEL: u8 = 3;

/// Scripted movement during a move (for example a rising special). While active it overrides
/// the fighter's velocity; `vx` is relative to facing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Motion {
    pub start: u8,
    pub end: u8,
    pub vx: Fx,
    pub vy: Fx,
}

/// A projectile a move fires on one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileSpawn {
    /// Move frame on which it appears.
    pub frame: u8,
    /// Offset from the feet, relative to facing.
    pub x: Fx,
    pub y: Fx,
    /// Speed along the facing direction, per frame.
    pub speed: Fx,
    /// Frames it exists before disappearing.
    pub life: u8,
    /// Its hit. The damage falls linearly from `hitbox.damage` to `end_damage` over its life.
    pub hitbox: Hitbox,
    pub end_damage: Fx,
    /// Its flight: a launch angle, gravity and bounces off the ground (default: straight on, through the ground's air).
    pub arc: ProjectileArc,
    /// Later hits by age (the reference's clean, late and latest hits of one projectile): from age `.0` on, `.1` replaces the hit
    /// (with no damage falloff). Up to two; `None` for none.
    pub phases: [Option<(u8, Hitbox)>; 2],
}

impl ProjectileSpawn {
    /// The hit a projectile of this kind has at `age`, and whether a phase gave it (a phase has no falloff).
    pub fn hit_at(&self, age: u8) -> (Hitbox, bool) {
        let mut out = (self.hitbox, false);
        for (from, hb) in self.phases.iter().flatten() {
            if age >= *from {
                out = (*hb, true);
            }
        }
        out
    }

    /// Whether it has later hits by age.
    pub fn any_phase(&self) -> bool {
        self.phases.iter().any(Option::is_some)
    }
}

/// How a projectile flies (world units per frame): fired `angle` degrees above its facing (negative is downward), pulled down by
/// `gravity` to at most `max_fall` (0: no limit). With `bounces` above 0 it bounces off the top of any platform up to that many
/// times (the next touch removes it): it comes off with its falling speed held between `bounce_min` and `bounce_max` and keeps
/// `bounce_keep` percent of its horizontal speed. With `bounces` 0 it ignores the ground's top as today (a solid wall removes it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileArc {
    pub angle: i16,
    pub gravity: Fx,
    pub max_fall: Fx,
    pub bounces: u8,
    pub bounce_min: Fx,
    pub bounce_max: Fx,
    pub bounce_keep: u8,
}

impl ProjectileArc {
    /// A straight flight (the original projectiles).
    pub const NONE: ProjectileArc = ProjectileArc {
        angle: 0,
        gravity: Fx::ZERO,
        max_fall: Fx::ZERO,
        bounces: 0,
        bounce_min: Fx::ZERO,
        bounce_max: Fx::ZERO,
        bounce_keep: 100,
    };
}

/// A reflecting field in front of the fighter: projectiles that touch it while it is up turn around and
/// belong to the fighter. Percentages are of the projectile's own damage and speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reflector {
    pub start: u8,
    pub end: u8,
    pub x: Fx,
    pub y: Fx,
    pub radius: Fx,
    pub damage_percent: u8,
    pub speed_percent: u8,
    /// The reflected shot's vertical speed is turned round too (it goes back the way it came), not only its horizontal speed.
    pub reverse_vertical: bool,
    /// Percent of its lifetime a reflected shot gets (its age starts again).
    pub life_percent: u8,
    /// A shot whose damage is above this is not reflected (0: no limit).
    pub max_damage: Fx,
    /// A projectile detector: a shot touching the circle at (`detect_x`, `detect_y`) of `detect_radius` on move frames
    /// `detect_start..=detect_end` skips the move ahead so the reflector is up on the next frame. `detect_end` 255: none.
    pub detect_start: u8,
    pub detect_end: u8,
    pub detect_x: Fx,
    pub detect_y: Fx,
    pub detect_radius: Fx,
}

impl Reflector {
    /// The fields added after the first reflectors, at their defaults (what older content means).
    pub const fn extras(mut self) -> Reflector {
        self.reverse_vertical = false;
        self.life_percent = 100;
        self.max_damage = Fx::ZERO;
        self.detect_start = 0;
        self.detect_end = 255;
        self.detect_x = Fx::ZERO;
        self.detect_y = Fx::ZERO;
        self.detect_radius = Fx::ZERO;
        self
    }
}

/// A counter stance: a hit that lands while it is active is cancelled, and the fighter answers with another move
/// that deals the hit's damage back with interest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counter {
    /// Move frames in which the stance catches hits.
    pub start: u8,
    pub end: u8,
    /// The move the fighter switches to (usually an `ext` slot).
    pub then: u8,
    /// The answer deals this percent of the caught hit's damage, at least `min_damage`.
    pub percent: u8,
    pub min_damage: Fx,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    /// The move ends when the move frame reaches this.
    pub total_frames: u8,
    /// Landing lag when an aerial lands mid-move.
    pub landing_lag: u8,
    /// Landing while the move frame is below this, or at/after `autocancel_after`, autocancels: only the
    /// character's normal landing lag. 0 and 255 mean "no such window".
    pub autocancel_before: u8,
    pub autocancel_after: u8,
    pub hitboxes: Vec<Hitbox>,
    pub motion: Vec<Motion>,
    pub projectile: Option<ProjectileSpawn>,
    /// Intangible for this many frames from the start of the move.
    pub intangible: u8,
    /// A special that leaves the fighter helpless when it ends in the air.
    pub helpless_after: bool,
    /// The fighter ends the move facing the other way.
    pub turns_around: bool,
    /// The fighter can grab a ledge while the move is in progress, even while rising (an up special), from either side and from
    /// its first frame. (The same as `ledge_front_from` and `ledge_back_from` 0; older content's way of saying it.)
    pub grabs_ledge: bool,
    // ---- Per-move recovery rules (the reference sets these per move; every default is the old shared behaviour) ----
    /// From this move frame a ledge in front of the fighter can be caught during the move (rising or not); 255 never.
    pub ledge_front_from: u8,
    /// ...and a ledge behind it (its back to the stage; the body's shorter back reach applies).
    pub ledge_back_from: u8,
    /// The special fall after this move keeps this percent of the body's air speed and acceleration; 0 uses
    /// `Ruleset::helpless_drift`.
    pub fall_drift: u8,
    /// Landing lag of the special fall after this move; 0 uses the body's `helpless_landing_lag`.
    pub fall_landing_lag: u8,
    /// Percent of the motion's vertical speed when the move started on the ground (`rise_ground`) or in the air (`rise_air`): the
    /// reference's grounded and aerial height multipliers on an up special's rise. 100 is the motion as written.
    pub rise_ground: u8,
    pub rise_air: u8,
    /// Started in the air, the fighter keeps this percent of its horizontal speed (the reference's "speed multiplier on startup").
    pub start_speed: u8,
    /// Angling by the stick: on move frame `steer_frame` (255 never) the stick's forward or backward tilt beyond `steer_stick`
    /// percent turns the move's motion by up to `steer_degrees` (toward forward or backward), growing evenly to the rim.
    pub steer_frame: u8,
    pub steer_stick: u8,
    pub steer_degrees: u8,
    /// The move's own air physics while it runs without a motion (`None`: the body's): gravity, top fall speed, air friction,
    /// and the percent of the body's air speed and acceleration it keeps (`air_drift`, 100 = all).
    pub air_gravity: Option<Fx>,
    pub air_max_fall: Option<Fx>,
    pub air_friction: Option<Fx>,
    pub air_drift: u8,
    /// Intangibility (`intangible` frames from the start) begins on this move frame instead of the first, when the move started
    /// on the ground (`intangible_from`) or in the air (`intangible_from_air`, 255 = the same as on the ground).
    pub intangible_from: u8,
    pub intangible_from_air: u8,
    /// Touching down during the move does not end it: the fighter lands and the move carries on along the ground.
    pub land_continue: bool,
    /// A smash attack holds on this move frame while the attack button stays held, up to the
    /// ruleset's charge limit, and then deals more damage the longer it was held.
    pub charge_at: Option<u8>,
    /// A jab: pressing attack within `next_window` frames of the move's end continues into this move.
    pub next: Option<u8>,
    pub next_window: u8,
    /// A jab whose next hit can start before its end (the reference's continuable window): from this move frame on, a press of attack
    /// (within `next_window` frames) carries on into `next` at once. 255 (the default) only continues at the end.
    pub next_from: u8,
    /// A multi-hit move: from move frame `.0`, every `.1` frames the move's hits are allowed to land again.
    pub rehit: Option<(u8, u8)>,
    /// The hit groups the rehit rhythm re-arms (bit `g` for group `g`); others hit once per move (a multi-hit's final hit). All by default.
    pub rehit_groups: u8,
    /// A move (usually an `ext` slot) started when this aerial lands outside its autocancel windows, instead of the plain landing lag:
    /// a landing hit. Its length is the landing lag.
    pub landing_move: Option<u8>,
    pub reflector: Option<Reflector>,
    pub counter: Option<Counter>,
    /// This move's hitboxes deal the damage the fighter's counter stance caught (see [`Counter`]).
    pub counter_strike: bool,
    /// Extra damage at full charge for this move, in percent; 0 uses the ruleset's `charge_bonus_percent`.
    pub charge_bonus: u8,
    /// Runs every frame of the move (movement, branching, extra projectile logic). See `sim-script`.
    pub script: Option<Program>,
    /// Runs every frame of the projectile this move fires.
    pub projectile_script: Option<Program>,
}

impl Move {
    /// A move slot that is not implemented yet; using it does nothing.
    pub fn empty() -> Move {
        Move {
            total_frames: 0,
            landing_lag: 0,
            autocancel_before: 0,
            autocancel_after: 255,
            hitboxes: Vec::new(),
            motion: Vec::new(),
            projectile: None,
            intangible: 0,
            helpless_after: false,
            turns_around: false,
            grabs_ledge: false,
            ledge_front_from: 255,
            ledge_back_from: 255,
            fall_drift: 0,
            fall_landing_lag: 0,
            rise_ground: 100,
            rise_air: 100,
            start_speed: 100,
            steer_frame: 255,
            steer_stick: 0,
            steer_degrees: 0,
            air_gravity: None,
            air_max_fall: None,
            air_friction: None,
            air_drift: 100,
            intangible_from: 0,
            intangible_from_air: 255,
            land_continue: false,
            charge_at: None,
            next: None,
            next_window: 0,
            next_from: 255,
            rehit: None,
            rehit_groups: 0xff,
            landing_move: None,
            reflector: None,
            counter: None,
            counter_strike: false,
            charge_bonus: 0,
            script: None,
            projectile_script: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_frames == 0
    }

    /// The move frame from which a ledge in front (`behind` false) or behind can be caught during the move; 255 never.
    pub fn ledge_from(&self, behind: bool) -> u8 {
        if self.grabs_ledge {
            0
        } else if behind {
            self.ledge_back_from
        } else {
            self.ledge_front_from
        }
    }

    /// Whether the move can catch a ledge at some point.
    pub fn catches_ledges(&self) -> bool {
        self.ledge_from(false) != 255 || self.ledge_from(true) != 255
    }

    /// The move frame its intangibility starts on, for a move started in the air or not.
    pub fn intangible_start(&self, air: bool) -> u8 {
        if air && self.intangible_from_air != 255 {
            self.intangible_from_air
        } else {
            self.intangible_from
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weapon {
    /// Indexed by `MoveId as usize`.
    pub moves: Vec<Move>,
    /// The attacks are the body's own (fists, claws, feet): while a hit is out, the limb throwing it can be hit too (see
    /// `combat::hurtboxes`). A held weapon (a blade, a hammer) reaches out beyond the body and cannot.
    pub limbs: bool,
}

impl Weapon {
    pub fn get(&self, id: u8) -> &Move {
        let i = usize::from(id).min(self.moves.len().saturating_sub(1));
        &self.moves[i]
    }
}

impl StateHash for Hitbox {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.start);
        h.write_u8(self.end);
        self.x.hash_into(h);
        self.y.hash_into(h);
        self.radius.hash_into(h);
        self.damage.hash_into(h);
        h.write_u16(self.angle as u16);
        h.write_u16(self.base_knockback as u16);
        h.write_u16(self.knockback_growth as u16);
        h.write_u8(self.priority);
        h.write_u8(self.group);
        h.write_u8(self.kind);
        h.write_u16(self.shield_damage);
        h.write_u16(self.hitlag);
        h.write_u8(self.effect);
        h.write_u8(self.targets);
        h.write_u16(self.fixed_kb);
        h.write_u8(self.hitstun_add);
        h.write_u16(self.sdi);
        self.link_ahead.hash_into(h);
        h.write_u8(self.aim);
        h.write_bool(self.set_weight);
        h.write_bool(self.reverse);
        h.write_bool(self.push);
    }
}

impl StateHash for Weapon {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(u8::from(self.limbs));
        h.write_u32(self.moves.len() as u32);
        for m in &self.moves {
            h.write_u8(m.total_frames);
            h.write_u8(m.landing_lag);
            h.write_u8(m.autocancel_before);
            h.write_u8(m.autocancel_after);
            h.write_u32(m.hitboxes.len() as u32);
            for hb in &m.hitboxes {
                hb.hash_into(h);
            }
            h.write_u32(m.motion.len() as u32);
            for mo in &m.motion {
                h.write_u8(mo.start);
                h.write_u8(mo.end);
                mo.vx.hash_into(h);
                mo.vy.hash_into(h);
            }
            match &m.projectile {
                Some(p) => {
                    h.write_bool(true);
                    h.write_u8(p.frame);
                    p.x.hash_into(h);
                    p.y.hash_into(h);
                    p.speed.hash_into(h);
                    h.write_u8(p.life);
                    p.hitbox.hash_into(h);
                    p.end_damage.hash_into(h);
                    let a = &p.arc;
                    h.write_u16(a.angle as u16);
                    a.gravity.hash_into(h);
                    a.max_fall.hash_into(h);
                    h.write_u8(a.bounces);
                    a.bounce_min.hash_into(h);
                    a.bounce_max.hash_into(h);
                    h.write_u8(a.bounce_keep);
                    for phase in &p.phases {
                        match phase {
                            Some((from, hb)) => {
                                h.write_bool(true);
                                h.write_u8(*from);
                                hb.hash_into(h);
                            }
                            None => h.write_bool(false),
                        }
                    }
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.intangible);
            h.write_bool(m.helpless_after);
            h.write_bool(m.turns_around);
            h.write_bool(m.grabs_ledge);
            for b in [
                m.ledge_front_from,
                m.ledge_back_from,
                m.fall_drift,
                m.fall_landing_lag,
                m.rise_ground,
                m.rise_air,
                m.start_speed,
                m.steer_frame,
                m.steer_stick,
                m.steer_degrees,
                m.air_drift,
                m.intangible_from,
                m.intangible_from_air,
            ] {
                h.write_u8(b);
            }
            for v in [m.air_gravity, m.air_max_fall, m.air_friction] {
                match v {
                    Some(x) => {
                        h.write_bool(true);
                        x.hash_into(h);
                    }
                    None => h.write_bool(false),
                }
            }
            h.write_bool(m.land_continue);
            match m.charge_at {
                Some(f) => {
                    h.write_bool(true);
                    h.write_u8(f);
                }
                None => h.write_bool(false),
            }
            match m.next {
                Some(n) => {
                    h.write_bool(true);
                    h.write_u8(n);
                    h.write_u8(m.next_window);
                    h.write_u8(m.next_from);
                }
                None => h.write_bool(false),
            }
            match m.rehit {
                Some((start, every)) => {
                    h.write_bool(true);
                    h.write_u8(start);
                    h.write_u8(every);
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.rehit_groups);
            match m.landing_move {
                Some(id) => {
                    h.write_bool(true);
                    h.write_u8(id);
                }
                None => h.write_bool(false),
            }
            match &m.reflector {
                Some(r) => {
                    h.write_bool(true);
                    h.write_u8(r.start);
                    h.write_u8(r.end);
                    r.x.hash_into(h);
                    r.y.hash_into(h);
                    r.radius.hash_into(h);
                    h.write_u8(r.damage_percent);
                    h.write_u8(r.speed_percent);
                    h.write_bool(r.reverse_vertical);
                    h.write_u8(r.life_percent);
                    r.max_damage.hash_into(h);
                    h.write_u8(r.detect_start);
                    h.write_u8(r.detect_end);
                    r.detect_x.hash_into(h);
                    r.detect_y.hash_into(h);
                    r.detect_radius.hash_into(h);
                }
                None => h.write_bool(false),
            }
            match &m.counter {
                Some(c) => {
                    h.write_bool(true);
                    h.write_u8(c.start);
                    h.write_u8(c.end);
                    h.write_u8(c.then);
                    h.write_u8(c.percent);
                    c.min_damage.hash_into(h);
                }
                None => h.write_bool(false),
            }
            h.write_bool(m.counter_strike);
            h.write_u8(m.charge_bonus);
            for script in [&m.script, &m.projectile_script] {
                match script {
                    Some(p) => {
                        h.write_bool(true);
                        h.write_bytes(&p.hash_bytes());
                    }
                    None => h.write_bool(false),
                }
            }
        }
    }
}

// ---- Builders -------------------------------------------------------------------------------------------

/// Compiles a built-in script. A mistake in one is a bug in this file, caught by the first test run.
fn script(kind: Kind, source: &str) -> Program {
    Program::compile(kind, source)
        .unwrap_or_else(|e| panic!("built-in script does not compile: {e}"))
}

/// Placeholder hitbox row: frames, position (tenths of a unit), radius (tenths), damage in half percents,
/// angle, base knockback, growth, priority.
struct B(u8, u8, i32, i32, i32, i32, i16, i16, i16, u8);

fn mk(total: u8, landing_lag: u8, autocancel: (u8, u8), boxes: &[B]) -> Move {
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    Move {
        total_frames: total,
        landing_lag,
        autocancel_before: autocancel.0,
        autocancel_after: autocancel.1,
        hitboxes: boxes
            .iter()
            .map(|b| Hitbox {
                start: b.0,
                end: b.1,
                x: tenths(b.2),
                y: tenths(b.3),
                radius: tenths(b.4),
                damage: Fx::from_ratio(b.5, 2),
                angle: b.6,
                base_knockback: b.7,
                knockback_growth: b.8,
                priority: b.9,
                group: 0,
                kind: HIT_NORMAL,
                shield_damage: 100,
                hitlag: 100,
                effect: EFFECT_NORMAL,
                targets: TARGETS_ALL,
                fixed_kb: 0,
                hitstun_add: 0,
                sdi: 100,
                link_ahead: Fx::ZERO,
                aim: AIM_ANY,
                set_weight: false,
                reverse: false,
                push: false,
            })
            .collect(),
        ..Move::empty()
    }
}

/// A hitbox row in the **reference's frame numbering** (frame 1 is the first frame of the move).
/// Position and radius are in tenths of a world unit, damage in tenths of a percent.
struct R {
    first: u8,
    last: u8,
    x: i32,
    y: i32,
    r: i32,
    damage: i32,
    angle: i16,
    bkb: i16,
    kbg: i16,
    priority: u8,
    group: u8,
}

fn row(r: &R) -> Hitbox {
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    Hitbox {
        start: r.first - 1,
        end: r.last - 1,
        x: tenths(r.x),
        y: tenths(r.y),
        radius: tenths(r.r),
        damage: tenths(r.damage),
        angle: r.angle,
        base_knockback: r.bkb,
        knockback_growth: r.kbg,
        priority: r.priority,
        group: r.group,
        kind: HIT_NORMAL,
        shield_damage: 100,
        hitlag: 100,
        effect: EFFECT_NORMAL,
        targets: TARGETS_ALL,
        fixed_kb: 0,
        hitstun_add: 0,
        sdi: 100,
        link_ahead: Fx::ZERO,
        aim: AIM_ANY,
        set_weight: false,
        reverse: false,
        push: false,
    }
}

/// Marks every hitbox of a move as `kind` (a grab, throw or pummel).
/// A throw in two parts: on frame `hit.0` a hit of `hit.1` tenths of a percent on the held fighter (damage only, like a pummel), then
/// the throw `throw` (its own group, so both land), which lets go and launches.
fn throw_after_hit(faf: u8, hit: (u8, i32), throw: R) -> Move {
    let mut m = ref_move(
        faf,
        0,
        0,
        255,
        &[r(hit.0, hit.0, 13, 11, 10, hit.1, 361, 0, 0, 0, 0), throw],
    );
    m.hitboxes[0].kind = HIT_PUMMEL;
    m.hitboxes[1].kind = HIT_THROW;
    m
}

fn with_kind(mut m: Move, kind: u8) -> Move {
    for hb in &mut m.hitboxes {
        hb.kind = kind;
    }
    m
}

/// A move from reference frame data: `faf` is the first actionable frame, `ac_before` the last frame
/// of an autocancel window at the start (0 = none) and `ac_after` the first frame of the late window
/// (255 = none).
fn ref_move(faf: u8, landing_lag: u8, ac_before: u8, ac_after: u8, rows: &[R]) -> Move {
    Move {
        total_frames: faf.saturating_sub(2),
        landing_lag,
        autocancel_before: ac_before,
        autocancel_after: if ac_after == 255 { 255 } else { ac_after - 1 },
        hitboxes: rows.iter().map(row).collect(),
        ..Move::empty()
    }
}

/// Sets each hitbox's hitlag multiplier (percent), in the order the rows were given.
fn with_hitlag(mut mv: Move, percents: &[u16]) -> Move {
    for (hb, &pct) in mv.hitboxes.iter_mut().zip(percents) {
        hb.hitlag = pct;
    }
    mv
}

#[allow(clippy::too_many_arguments)]
fn r(
    first: u8,
    last: u8,
    x: i32,
    y: i32,
    radius: i32,
    damage: i32,
    angle: i16,
    bkb: i16,
    kbg: i16,
    priority: u8,
    group: u8,
) -> R {
    R {
        first,
        last,
        x,
        y,
        r: radius,
        damage,
        angle,
        bkb,
        kbg,
        priority,
        group,
    }
}

// ---- Weapons --------------------------------------------------------------------------------------------

/// A long, thin blade: the tip (priority 0) hits harder than the body, so spacing is the skill.
///
/// Built from reference frame data (a swordfighter archetype): forward tilt, neutral air, forward air,
/// back air and up special. The other moves are placeholders.
pub fn longsword() -> Weapon {
    let mut w = sword_kit();
    shorten_blade(
        &mut w,
        Vec2::new(Fx::from_ratio(2, 5), Fx::from_ratio(11, 10)),
        Fx::from_ratio(72, 100),
    );
    w.moves[MoveId::UpSpecial as usize] = longsword_up_special();
    tipper_last(&mut w);
    w
}

/// The longsword's up special, a rising slash (the reference swordfighter's up special: ssbwiki.com, 13.0.1).
///
/// Hits [R]: frame 5 the clean hit at the hand (top bone, 0 / 8 / 8 and 0 / 8 / 4 reference units: 11%, 361 and 74 degrees, BK 70,
/// KS 74, radius 4); frame 6 the same plus the blade (7%, 361 and 74, BK 20, KS 90, radius 5, hitlag 0.9), which wins where both
/// reach (its ids are lower); frames 7-11 the blade alone. Intangible on frames 4-5 from the ground, 1-5 in the air. Animation 39
/// frames, then the special fall: air speed x0.4, landing lag 24. Ledges in front from frame 8, never behind during the move. The
/// stick angles it (sensitivity 0.5, up to 20 degrees); started in the air it keeps 0.66 of its horizontal speed and rises 1.1 times
/// as high; once the rise is over its gravity is 0.06. The rise itself is the reference's animation, not a published number: it is
/// kept at the earlier estimate (about 44 reference units from the ground) [E], and the hitbox positions on the blade are [E].
fn longsword_up_special() -> Move {
    let su = |n: i32| Fx::from_ratio(n, 8000);
    // `tipper_last` turns the written priorities 1, 2, 0, 4 into the reference's ids 0, 1, 2, 3.
    let mut up = with_hitlag(
        ref_move(
            40,
            0,
            0,
            255,
            &[
                r(6, 11, 12, 26, 6, 70, 361, 20, 90, 1, 0),
                r(6, 11, 8, 18, 6, 70, 74, 20, 90, 2, 0),
                r(5, 6, 10, 10, 5, 110, 361, 70, 74, 0, 0),
                r(5, 6, 5, 10, 5, 110, 74, 70, 74, 4, 0),
            ],
        ),
        &[90, 90, 100, 100],
    );
    up.motion = vec![
        Motion {
            start: 4,
            end: 12,
            vx: su(1000),
            vy: su(4400),
        },
        Motion {
            start: 13,
            end: 18,
            vx: su(500),
            vy: su(500),
        },
    ];
    up.intangible = 5;
    up.intangible_from = 3;
    up.intangible_from_air = 0;
    up.helpless_after = true;
    up.ledge_front_from = 7;
    up.fall_drift = 40;
    up.fall_landing_lag = 24;
    up.rise_air = 110;
    up.start_speed = 66;
    // The angle is read as the rise begins [E: the frame is not published].
    up.steer_frame = 3;
    up.steer_stick = 50;
    up.steer_degrees = 20;
    up.air_gravity = Some(su(60));
    up
}

/// The reference game's order among a sword's hitboxes: where several overlap a target, the blade's sourspot wins, then the arm, and
/// the tipper only when it is the one that reaches (so the tipper has to be spaced). The kit is written tip first (priority 0), blade
/// (1), arm (2); this turns that into blade 0, arm 1, tip 2.
fn tipper_last(w: &mut Weapon) {
    for mv in &mut w.moves {
        for hb in &mut mv.hitboxes {
            if hb.kind == HIT_NORMAL {
                hb.priority = match hb.priority {
                    0 => 2,
                    p => p - 1,
                };
            }
        }
    }
}

/// Pulls every hit of every move toward `pivot` (the shoulder) by `k`: a shorter blade with the same swings. Grabs and throws
/// (which are the hand, not the blade) are left alone; sizes stay the same.
fn shorten_blade(w: &mut Weapon, pivot: Vec2, k: Fx) {
    for mv in &mut w.moves {
        for hb in &mut mv.hitboxes {
            if hb.kind == HIT_NORMAL {
                hb.x = pivot.x + (hb.x - pivot.x) * k;
                hb.y = pivot.y + (hb.y - pivot.y) * k;
            }
        }
    }
}

/// The sword's moves at full reach: the duelist's longsword shortens them, and the claws start from them.
fn sword_kit() -> Weapon {
    let none = (0, 255);
    let mut moves = vec![
        // Jab (below)
        Move::empty(),
        // Forward tilt (replaced below with reference data)
        Move::empty(),
        // Up tilt
        mk(
            26,
            0,
            none,
            &[
                B(7, 10, 10, 26, 6, 14, 100, 20, 100, 0),
                B(7, 10, 6, 20, 8, 10, 100, 16, 95, 1),
            ],
        ),
        // Down tilt
        mk(
            24,
            0,
            none,
            &[
                B(6, 8, 25, 4, 5, 12, 80, 10, 80, 0),
                B(6, 8, 14, 4, 6, 8, 80, 8, 75, 1),
            ],
        ),
        // Dash attack (below)
        Move::empty(),
        // Forward smash
        mk(
            46,
            0,
            none,
            &[
                B(14, 16, 29, 12, 7, 32, 361, 25, 100, 0),
                B(14, 16, 16, 11, 8, 24, 361, 22, 95, 1),
            ],
        ),
        // Up smash
        mk(
            52,
            0,
            none,
            &[
                B(14, 18, 8, 29, 7, 26, 88, 30, 100, 0),
                B(14, 18, 4, 20, 9, 20, 88, 26, 95, 1),
            ],
        ),
        // Down smash (hits both sides)
        mk(
            50,
            0,
            none,
            &[
                B(12, 14, 27, 4, 6, 24, 30, 25, 85, 0),
                B(12, 14, -20, 4, 6, 22, 30, 22, 80, 0),
            ],
        ),
        // Neutral air, forward air, back air (replaced below with reference data)
        Move::empty(),
        Move::empty(),
        Move::empty(),
        // Up air
        mk(
            34,
            11,
            (6, 28),
            &[
                B(6, 10, 4, 30, 7, 16, 85, 20, 90, 0),
                B(6, 10, 4, 21, 8, 12, 85, 16, 85, 1),
            ],
        ),
        // Down air (below)
        Move::empty(),
        // Specials (the up special is replaced below)
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        // Jab 2, jab 3 and the grabs and throws (below)
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
    ];
    // Ledge attack and get-up attack (below), then the script follow-up slots.
    moves.push(Move::empty());
    moves.push(Move::empty());
    moves.resize_with(MoveId::COUNT, Move::empty);

    // Forward tilt: frames 8-11, 9/12 damage (sour/tip), angle 361, FAF 34. A rising slash: the blade climbs from low in front (frame 8)
    // to high in front (11), and the hits climb with it.
    moves[MoveId::FTilt as usize] = ref_move(
        34,
        0,
        0,
        255,
        &[
            r(8, 8, 29, 0, 7, 120, 361, 55, 85, 0, 0),
            r(9, 9, 32, 11, 7, 120, 361, 55, 85, 0, 0),
            r(10, 10, 29, 22, 7, 120, 361, 55, 85, 0, 0),
            r(11, 11, 22, 32, 7, 120, 361, 55, 85, 0, 0),
            r(8, 8, 18, 5, 8, 90, 361, 30, 70, 1, 0),
            r(9, 9, 19, 11, 8, 90, 361, 30, 70, 1, 0),
            r(10, 10, 18, 17, 8, 90, 361, 30, 70, 1, 0),
            r(11, 11, 14, 22, 8, 90, 361, 30, 70, 1, 0),
        ],
    );
    // Up tilt: one hit in three phases, an overhead arc. Early (frame 6) 6% tip, 5% arm and body; frames 7-8
    // 10% tipper / 6% sourspot; frames 9-12 the same with a lower angle (85) and knockback (52). FAF 34.
    // Angle 100 pulls a victim in front of the sword back toward the fighter.
    moves[MoveId::UTilt as usize] = ref_move(
        34,
        0,
        0,
        255,
        &[
            r(6, 6, 14, 26, 8, 60, 100, 65, 100, 0, 0),
            r(6, 6, 9, 20, 7, 50, 100, 65, 100, 1, 0),
            r(6, 6, 3, 18, 8, 50, 100, 65, 100, 1, 0),
            r(7, 8, 9, 31, 7, 100, 100, 65, 100, 0, 0),
            r(7, 8, 6, 27, 8, 60, 100, 65, 100, 1, 0),
            r(7, 8, 3, 22, 7, 50, 100, 65, 100, 1, 0),
            r(7, 8, 0, 19, 8, 50, 100, 65, 100, 1, 0),
            r(9, 12, -8, 30, 7, 100, 85, 52, 100, 0, 0),
            r(9, 12, -5, 26, 8, 60, 85, 52, 100, 1, 0),
            r(9, 12, -3, 21, 7, 50, 100, 52, 100, 1, 0),
            r(9, 12, 0, 18, 8, 50, 100, 52, 100, 1, 0),
        ],
    );
    // Down tilt: a low stab on frames 7-8, 7% close / 10% at the tip, angle 30, FAF 24. (It can trip in the
    // reference game; tripping is not implemented.)
    moves[MoveId::DTilt as usize] = ref_move(
        24,
        0,
        0,
        255,
        &[
            r(7, 8, 16, 5, 8, 70, 30, 40, 40, 1, 0),
            r(7, 8, 31, 4, 7, 100, 30, 50, 40, 0, 0),
        ],
    );
    // Forward smash: frames 10-13, 13% (18% at the tip), angle 361, FAF 52. Charges on frame 2. A big downward cut: the blade comes
    // over from high in front (frame 10) down through level (12) to just below it (13).
    let mut fsmash = ref_move(
        52,
        0,
        0,
        255,
        &[
            r(10, 10, 15, 38, 7, 180, 361, 80, 80, 0, 0),
            r(11, 11, 29, 28, 7, 180, 361, 80, 80, 0, 0),
            r(12, 12, 34, 11, 7, 180, 361, 80, 80, 0, 0),
            r(13, 13, 31, -1, 7, 180, 361, 80, 80, 0, 0),
            r(10, 10, 11, 27, 8, 130, 361, 48, 75, 1, 0),
            r(11, 11, 19, 21, 8, 130, 361, 48, 75, 1, 0),
            r(12, 12, 22, 11, 8, 130, 361, 48, 75, 1, 0),
            r(13, 13, 20, 4, 8, 130, 361, 48, 75, 1, 0),
            r(10, 13, 5, 12, 8, 130, 361, 48, 75, 1, 0),
        ],
    );
    fsmash.charge_at = Some(1);
    moves[MoveId::FSmash as usize] = fsmash;
    // Up smash: frames 13-17 overhead, 13% (17% at the tip), angle 89, FAF 59. Charges on frame 4. The
    // reference game also has a 3% launcher that pulls grounded targets in; that is not implemented.
    let mut usmash = ref_move(
        59,
        0,
        0,
        255,
        &[
            r(13, 17, 6, 26, 9, 130, 89, 45, 90, 1, 0),
            r(13, 17, 5, 36, 7, 170, 89, 40, 95, 0, 0),
            r(13, 17, -4, 30, 8, 130, 90, 45, 90, 1, 0),
        ],
    );
    usmash.charge_at = Some(3);
    moves[MoveId::USmash as usize] = usmash;
    // Down smash: a front hit on frames 6-7 (8%, 12% at the tip), then a back hit on frames 21-23 (12%, 17%
    // at the tip), angle 361, FAF 56. Charges on frame 4.
    let mut dsmash = ref_move(
        56,
        0,
        0,
        255,
        &[
            r(6, 7, 18, 5, 8, 80, 361, 60, 88, 1, 0),
            r(6, 7, 10, 6, 7, 80, 361, 60, 88, 1, 0),
            r(6, 7, 4, 10, 6, 80, 361, 60, 88, 1, 0),
            r(6, 7, 31, 4, 7, 120, 361, 50, 88, 0, 0),
            r(21, 23, -18, 5, 8, 120, 361, 40, 88, 1, 1),
            r(21, 23, -10, 6, 7, 120, 361, 40, 88, 1, 1),
            r(21, 23, -4, 10, 6, 120, 361, 40, 88, 1, 1),
            r(21, 23, -31, 4, 7, 170, 361, 50, 92, 0, 1),
        ],
    );
    dsmash.charge_at = Some(3);
    moves[MoveId::DSmash as usize] = dsmash;
    // Up air: frames 5-9, 9.5% (13% at the tip), angle 80 (90 at the tip), landing lag 8, autocancels on frames 1-2 and from 38, FAF
    // 46. A crescent overhead with a somersault: the blade goes from in front (frame 5) over the top (7) to behind (9).
    moves[MoveId::UAir as usize] = ref_move(
        46,
        8,
        2,
        38,
        &[
            r(5, 5, 24, 22, 7, 130, 90, 40, 84, 0, 0),
            r(6, 6, 15, 34, 7, 130, 90, 40, 84, 0, 0),
            r(7, 7, 0, 39, 7, 130, 90, 40, 84, 0, 0),
            r(8, 8, -15, 34, 7, 130, 90, 40, 84, 0, 0),
            r(9, 9, -24, 22, 7, 130, 90, 40, 84, 0, 0),
            r(5, 5, 13, 18, 8, 95, 80, 40, 80, 1, 0),
            r(6, 6, 8, 24, 8, 95, 80, 40, 80, 1, 0),
            r(7, 7, 0, 27, 8, 95, 80, 40, 80, 1, 0),
            r(8, 8, -8, 24, 8, 95, 80, 40, 80, 1, 0),
            r(9, 9, -13, 18, 8, 95, 80, 40, 80, 1, 0),
            r(5, 9, 0, 14, 6, 95, 80, 40, 80, 2, 0),
        ],
    );
    // Neutral air (the reference's frames): two hits. Hit 1 on frames 6-7, a quick upward sweep in front (3.5%, 5% at the tip); hit 2 on
    // frames 15-21, the blade carried right round the fighter once, from high in front over the top, behind, underneath and back to the
    // front (7%, 9.5% at the tip). Landing lag 7, autocancels from frame 47, FAF 50.
    moves[MoveId::NAir as usize] = ref_move(
        50,
        7,
        0,
        47,
        &[
            r(6, 6, 23, -2, 7, 50, 90, 35, 50, 0, 0),
            r(7, 7, 23, 24, 7, 50, 90, 35, 50, 0, 0),
            r(6, 6, 13, 4, 8, 35, 75, 45, 50, 1, 0),
            r(7, 7, 13, 18, 8, 35, 75, 45, 50, 1, 0),
            r(15, 15, 14, 34, 7, 95, 361, 60, 100, 0, 1),
            r(16, 16, -9, 36, 7, 95, 361, 60, 100, 0, 1),
            r(17, 17, -25, 20, 7, 95, 361, 60, 100, 0, 1),
            r(18, 18, -23, -3, 7, 95, 361, 60, 100, 0, 1),
            r(19, 19, -5, -16, 7, 95, 361, 60, 100, 0, 1),
            r(20, 20, 17, -10, 7, 95, 361, 60, 100, 0, 1),
            r(21, 21, 27, 6, 7, 95, 361, 60, 100, 0, 1),
            r(15, 15, 8, 24, 8, 70, 361, 50, 90, 1, 1),
            r(16, 16, -5, 25, 8, 70, 361, 50, 90, 1, 1),
            r(17, 17, -14, 16, 8, 70, 361, 50, 90, 1, 1),
            r(18, 18, -13, 3, 8, 70, 361, 50, 90, 1, 1),
            r(19, 19, -3, -4, 8, 70, 361, 50, 90, 1, 1),
            r(20, 20, 10, 0, 8, 70, 361, 50, 90, 1, 1),
            r(21, 21, 15, 8, 8, 70, 361, 50, 90, 1, 1),
        ],
    );
    // Forward air (the reference's, frame for frame): the blade sweeps a crescent in front, from high overhead (frame 6) through straight
    // ahead (7) to low in front (8). The tip is the tipper: 11.5%, angle 361, BKB 40, KBG 80, hitlag x1.25; the rest of the blade
    // and the arm are 8% with hitlag x0.7. Landing lag 10, autocancels from frame 36, FAF 38. The hits follow the blade frame by frame
    // (one hit per target), so the trail and the arm draw the arc where it really hits.
    moves[MoveId::FAir as usize] = with_hitlag(
        ref_move(
            38,
            10,
            0,
            36,
            &[
                // Tipper along the arc (centre about the shoulder, 2.7 out): 70, 15 and -45 degrees.
                r(6, 6, 14, 35, 7, 115, 361, 40, 80, 0, 0),
                r(7, 7, 31, 17, 7, 115, 361, 40, 80, 0, 0),
                r(8, 8, 24, -9, 7, 115, 361, 40, 80, 0, 0),
                // The middle of the blade (1.5 out).
                r(6, 6, 10, 24, 8, 80, 361, 40, 80, 1, 0),
                r(7, 7, 20, 14, 8, 80, 361, 40, 80, 1, 0),
                r(8, 8, 16, -1, 8, 80, 361, 40, 80, 1, 0),
                // The arm.
                r(6, 8, 6, 12, 7, 80, 361, 40, 80, 2, 0),
            ],
        ),
        &[125, 125, 125, 70, 70, 70, 70],
    );
    // Back air: frames 7-11, 9/12.5 damage, angle 361, landing lag 10, autocancels frames 1-2 and from 32, FAF 40.
    // The fighter ends the move facing the other way.
    // The fighter turns and slashes upward behind itself: the blade rises from low behind (frame 7) to high behind (11).
    let mut bair = ref_move(
        40,
        10,
        2,
        32,
        &[
            r(7, 7, -22, -4, 7, 125, 361, 40, 94, 0, 0),
            r(8, 8, -27, 6, 7, 125, 361, 40, 94, 0, 0),
            r(9, 9, -26, 18, 7, 125, 361, 40, 94, 0, 0),
            r(10, 10, -21, 28, 7, 125, 361, 40, 94, 0, 0),
            r(11, 11, -11, 35, 7, 125, 361, 40, 94, 0, 0),
            r(7, 7, -12, 2, 8, 90, 361, 40, 85, 1, 0),
            r(8, 8, -15, 8, 8, 90, 361, 40, 85, 1, 0),
            r(9, 9, -14, 15, 8, 90, 361, 40, 85, 1, 0),
            r(10, 10, -11, 21, 8, 90, 361, 40, 85, 1, 0),
            r(11, 11, -6, 25, 8, 90, 361, 40, 85, 1, 0),
        ],
    );
    bair.turns_around = true;
    moves[MoveId::BAir as usize] = bair;

    // (The up special is the longsword's own, built after the blade is shortened: see `longsword_up_special`.)

    // ---- The rest of the sword kit ----
    // Positions and sizes are estimates throughout; the frames, damage and (where known) knockback follow the
    // reference tables. `ext` slots are only reached by a script or a counter.

    // Jab: two hits. Hit 1 on frames 5-6 (3% close, 5% tip), FAF 26; hit 2 on frames 4-5 (4% / 6%), FAF 29.
    let mut jab1 = ref_move(
        26,
        0,
        0,
        255,
        &[
            // A quick downward slash in front (frame 5 high, frame 6 level).
            r(5, 5, 27, 26, 7, 50, 361, 25, 15, 0, 0),
            r(6, 6, 32, 13, 7, 50, 361, 25, 15, 0, 0),
            r(5, 5, 16, 19, 8, 30, 361, 25, 15, 1, 0),
            r(6, 6, 19, 12, 8, 30, 361, 25, 15, 1, 0),
        ],
    );
    jab1.next = Some(MoveId::Jab2 as u8);
    jab1.next_window = 10;
    moves[MoveId::Jab as usize] = jab1;
    moves[MoveId::Jab2 as usize] = ref_move(
        29,
        0,
        0,
        255,
        &[
            // The backhand coming back up (frame 4 low, frame 5 high).
            r(4, 4, 32, 6, 7, 60, 361, 40, 30, 0, 0),
            r(5, 5, 29, 22, 7, 60, 361, 40, 30, 0, 0),
            r(4, 4, 19, 9, 8, 40, 361, 40, 30, 1, 0),
            r(5, 5, 18, 17, 8, 40, 361, 40, 30, 1, 0),
        ],
    );

    // Dash attack: frames 13-16, 9% / 10% / 13% (closest, close, tip), tip knockback 93 base 58 growth, FAF 50.
    moves[MoveId::DashAttack as usize] = ref_move(
        50,
        0,
        0,
        255,
        &[
            // A rising slash on the run: low in front (frame 13) up to high in front (16).
            r(13, 13, 29, -3, 8, 130, 361, 93, 58, 0, 0),
            r(14, 14, 33, 11, 8, 130, 361, 93, 58, 0, 0),
            r(15, 15, 29, 25, 8, 130, 361, 93, 58, 0, 0),
            r(16, 16, 19, 35, 8, 130, 361, 93, 58, 0, 0),
            r(13, 13, 20, 3, 8, 100, 361, 75, 58, 1, 0),
            r(14, 14, 22, 11, 8, 100, 361, 75, 58, 1, 0),
            r(15, 15, 20, 20, 8, 100, 361, 75, 58, 1, 0),
            r(16, 16, 14, 26, 8, 100, 361, 75, 58, 1, 0),
            r(13, 16, 13, 10, 9, 90, 361, 60, 58, 2, 0),
        ],
    );

    // Down air (the reference's): the blade sweeps a crescent underneath, from in front (frame 9) through straight down (11) to behind
    // (13). The tip is 14% (angle 361, BKB 20, KBG 80, hitlag x1.25) except on frame 11, when a big hitbox straight below is the 15%
    // meteor (angle 270, BKB 20, KBG 80, hitlag x1.3). The middle of the blade is 12% at 80 degrees (BKB 40, KBG 70) and the arm 12%
    // 361 (BKB 30, KBG 70), both hitlag x0.7. Landing lag 14, autocancels on frames 1-2 and from 55, FAF 60.
    moves[MoveId::DAir as usize] = with_hitlag(
        ref_move(
            60,
            14,
            2,
            55,
            &[
                // Tip along the arc (centre at the waist, 2.4 out): -15, -50, (frame 11: the meteor), -130, -165 degrees.
                r(9, 9, 23, 3, 7, 140, 361, 20, 80, 0, 0),
                r(10, 10, 15, -9, 7, 140, 361, 20, 80, 0, 0),
                r(11, 11, 0, -12, 9, 150, 270, 20, 80, 0, 0),
                r(12, 12, -15, -9, 7, 140, 361, 20, 80, 0, 0),
                r(13, 13, -23, 3, 7, 140, 361, 20, 80, 0, 0),
                // The middle of the blade (1.3 out).
                r(9, 9, 13, 6, 8, 120, 80, 40, 70, 1, 0),
                r(10, 10, 8, -1, 8, 120, 80, 40, 70, 1, 0),
                r(11, 11, 0, -4, 8, 120, 80, 40, 70, 1, 0),
                r(12, 12, -8, -1, 8, 120, 80, 40, 70, 1, 0),
                r(13, 13, -13, 6, 8, 120, 80, 40, 70, 1, 0),
                // The arm.
                r(9, 13, 0, 7, 6, 120, 361, 30, 70, 2, 0),
            ],
        ),
        &[125, 125, 130, 125, 125, 70, 70, 70, 70, 70, 70],
    );

    // Grab (hits frames 6-7, FAF 35), dash grab (frames 9-10, FAF 43), pummel 1.3% (frame 2: the reference says 1, but a hit cannot
    // start on a move's first frame; FAF 20) and the four throws: forward 4% (releases on frame 18, FAF 35), back 4% (19, FAF 45),
    // up 5% (13, FAF 45, growth 102), down 4% (20, FAF 47, growth 57). Throw angles, base knockback and the grab's reach are
    // estimates.
    moves[MoveId::Grab as usize] = with_kind(
        ref_move(35, 0, 0, 255, &[r(6, 7, 19, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(43, 0, 0, 255, &[r(9, 10, 21, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    // Pivot grab: a grab out of a dash turned around; frames 10-11, FAF 38.
    moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(38, 0, 0, 255, &[r(10, 11, 19, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    moves[MoveId::Pummel as usize] = reference_pummel(13, 13, 11, 10);
    moves[MoveId::FThrow as usize] = with_kind(
        ref_move(
            35,
            0,
            0,
            255,
            &[r(18, 18, 13, 11, 10, 40, 40, 60, 50, 0, 0)],
        ),
        HIT_THROW,
    );
    moves[MoveId::BThrow as usize] = with_kind(
        ref_move(
            45,
            0,
            0,
            255,
            &[r(19, 19, -13, 11, 10, 40, 45, 65, 55, 0, 0)],
        ),
        HIT_THROW,
    );
    moves[MoveId::UThrow as usize] = with_kind(
        ref_move(
            45,
            0,
            0,
            255,
            &[r(13, 13, 13, 11, 10, 50, 88, 60, 102, 0, 0)],
        ),
        HIT_THROW,
    );
    moves[MoveId::DThrow as usize] = with_kind(
        ref_move(
            47,
            0,
            0,
            255,
            &[r(20, 20, 13, 11, 10, 40, 84, 85, 57, 0, 0)],
        ),
        HIT_THROW,
    );

    // Ledge attack (the reference's swordfighter): 9%, angle 45, base knockback 90, growth 20, hits on frames 23-25,
    // intangible through frame 25, FAF 56.
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &[
            r(23, 25, 31, 6, 9, 90, 45, 90, 20, 0, 0),
            r(23, 25, 19, 6, 10, 90, 45, 90, 20, 1, 0),
        ],
    );
    ledge.intangible = 25;
    moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack from the floor (the reference's swordfighter): 7% in front on frames 16-17, then 7% behind on 22-23, angle 361,
    // base knockback 80, growth 48 (it always sends into tumble but barely grows with damage), intangible through frame 23, FAF 46.
    let mut getup_attack = ref_move(
        46,
        0,
        0,
        255,
        &[
            r(16, 17, 18, 5, 9, 70, 361, 80, 48, 0, 0),
            r(22, 23, -18, 5, 9, 70, 361, 80, 48, 0, 1),
        ],
    );
    getup_attack.intangible = 23;
    moves[MoveId::GetUpAttack as usize] = getup_attack;

    // Neutral special, Shield Breaker: hold special to charge (from frame 19, at most the charge limit), let go to
    // thrust. The thrust hits on frame 27: 8% (9% at the tip) uncharged, up to about 24% fully charged. Release to
    // end is 39 frames. Knockback is an estimate.
    let mut breaker = ref_move(
        58,
        0,
        0,
        255,
        &[
            r(27, 29, 45, 12, 8, 90, 361, 60, 60, 0, 0),
            r(27, 29, 31, 12, 9, 80, 361, 55, 60, 1, 0),
        ],
    );
    breaker.charge_bonus = 170;
    // It is the shield-breaking move: its hits do double damage to a shield (an estimate; charged, it breaks a full one).
    for hb in &mut breaker.hitboxes {
        hb.shield_damage = 200;
    }
    breaker.motion = vec![Motion {
        start: 25,
        end: 30,
        vx: Fx::from_ratio(45, 100),
        vy: Fx::ZERO,
    }];
    breaker.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/shield_breaker.script"),
    ));
    moves[MoveId::NSpecial as usize] = breaker;

    // Side special, Dancing Blade: up to four hits, each started by pressing special again (the stick chooses the
    // rising or low variants). Hit 1 starts on frame 9 (FAF 39 on the ground, ends after 29 in the air). Hits 2-4
    // are `ext` slots: startup / FAF 5/38 (straight) and 4/38 (rising); 4/43, 5/43 and 5/43 for hit 3; 7/55,
    // 6/44 and 7/74 for hit 4. Damage is 2.5% (3% at the tip) for hits 1-2, 3% (4%) for hit 3, and 4% (6%),
    // 5% (7%) and 2% x4 then 4% (5%) for the three finishers. Angles and the early hits' knockback are estimates;
    // the straight finisher's knockback (74/103, tip 85/125) and the rising one's (80/85 base, 40 growth) are not.
    let ext = |n: u8| MoveId::from_index(28 + n) as usize;
    let db_script = |src: &str| Some(script(Kind::Fighter, src));
    let mut db1 = ref_move(
        39,
        0,
        0,
        255,
        &[
            r(9, 10, 31, 11, 7, 30, 361, 25, 0, 0, 0),
            r(9, 10, 18, 11, 8, 25, 361, 25, 0, 1, 0),
        ],
    );
    db1.script = db_script(include_str!("scripts/dancing_blade_1.script"));
    moves[MoveId::SideSpecial as usize] = db1;
    let mut db2n = ref_move(
        38,
        0,
        0,
        255,
        &[
            r(5, 6, 31, 11, 7, 30, 361, 25, 0, 0, 0),
            r(5, 6, 18, 11, 8, 25, 361, 25, 0, 1, 0),
        ],
    );
    db2n.script = db_script(include_str!("scripts/dancing_blade_2.script"));
    moves[ext(1)] = db2n;
    let mut db2u = ref_move(
        38,
        0,
        0,
        255,
        &[
            r(4, 5, 24, 22, 7, 30, 80, 25, 0, 0, 0),
            r(4, 5, 14, 17, 8, 25, 80, 25, 0, 1, 0),
        ],
    );
    db2u.script = db_script(include_str!("scripts/dancing_blade_2.script"));
    moves[ext(2)] = db2u;
    for (slot, startup, y, angle, tip_dmg, body_dmg) in [
        (3u8, 4u8, 11, 361, 40, 30),
        (4, 5, 22, 80, 40, 30),
        (5, 5, 4, 361, 40, 30),
    ] {
        let mut m = ref_move(
            43,
            0,
            0,
            255,
            &[
                r(startup, startup + 1, 31, y, 7, tip_dmg, angle, 25, 0, 0, 0),
                r(startup, startup + 1, 18, y, 8, body_dmg, angle, 25, 0, 1, 0),
            ],
        );
        m.script = db_script(include_str!("scripts/dancing_blade_3.script"));
        moves[ext(slot)] = m;
    }
    // Finishers: straight, rising, and the low multi-hit one.
    let mut finisher_straight = ref_move(
        55,
        0,
        0,
        255,
        &[
            r(7, 8, 35, 11, 8, 60, 361, 85, 125, 0, 0),
            r(7, 8, 20, 11, 9, 40, 361, 74, 103, 1, 0),
        ],
    );
    finisher_straight.script = db_script(include_str!("scripts/dancing_blade_4.script"));
    moves[ext(6)] = finisher_straight;
    let mut finisher_rising = ref_move(
        44,
        0,
        0,
        255,
        &[
            r(6, 7, 22, 26, 8, 70, 80, 85, 40, 0, 0),
            r(6, 7, 12, 20, 9, 50, 80, 80, 40, 1, 0),
        ],
    );
    finisher_rising.script = db_script(include_str!("scripts/dancing_blade_4.script"));
    moves[ext(7)] = finisher_rising;
    let mut low = ref_move(
        74,
        0,
        0,
        255,
        &[
            r(7, 22, 30, 4, 8, 20, 361, 10, 0, 0, 0),
            r(26, 27, 33, 5, 8, 50, 361, 40, 20, 0, 1),
            r(26, 27, 20, 5, 9, 40, 361, 40, 20, 1, 1),
        ],
    );
    low.rehit = Some((6, 4));
    low.script = db_script(include_str!("scripts/dancing_blade_4.script"));
    moves[ext(8)] = low;

    // Down special, Counter: for 22 frames starting on frame 6 a hit is caught instead of taken, and the duelist
    // answers with the counter-attack (`ext0`), which deals 1.2 times the caught damage (at least 8%). Without a
    // hit the stance lasts 64 frames. The counter-attack comes out on frame 4 and lasts to FAF 40; its
    // position, size and knockback are estimates.
    let mut counter = ref_move(64, 0, 0, 255, &[]);
    counter.counter = Some(Counter {
        start: 5,
        end: 26,
        then: MoveId::Ext0 as u8,
        percent: 120,
        min_damage: Fx::from_int(8),
    });
    moves[MoveId::DownSpecial as usize] = counter;
    let mut answer = ref_move(
        40,
        0,
        0,
        255,
        &[
            r(4, 6, 36, 11, 9, 80, 361, 60, 85, 0, 0),
            r(4, 6, 20, 11, 10, 80, 361, 60, 85, 1, 0),
        ],
    );
    answer.counter_strike = true;
    answer.intangible = 8;
    moves[MoveId::Ext0 as usize] = answer;

    Weapon {
        moves,
        limbs: false,
    }
}

/// Sets fields on the hitboxes `rows` (indices, in the order the rows were given) of a move.
fn set_rows(mv: &mut Move, rows: &[usize], f: impl Fn(&mut Hitbox)) {
    for &i in rows {
        f(&mut mv.hitboxes[i]);
    }
}

/// The Needles moveset (the rogue body): quick kicks and chops, modelled mechanically on the reference rogue. The priority
/// moves (forward, up and down tilt; neutral, forward, back, up and down air) follow the reference frame for frame: frames, damage,
/// angles, base and growth knockback, set knockback, hitlag and SDI multipliers, grounded / aerial targets, hitstun modifiers, landing
/// lag and autocancel windows, the up air's looping autolink hits and final hit, the down air's dive and landing hit. Radii are the
/// reference's converted; positions are estimates on our body (the reference attaches its hitboxes to bones of its own skeleton). The
/// data and its sources: `docs/sheik/REFERENCE_DATA.md`. The jabs and rapid jab, smashes, grabs, throws and the needle, vanish and
/// bouncing specials follow the reference too; the dash attack is the earlier game's, and the side special is our own pulling line.
pub fn needles() -> Weapon {
    let mut w = claws();
    // Kicks and the hands: the limb throwing a hit can be hit.
    w.limbs = true;

    // Forward tilt: a rising front kick, frames 5-8, FAF 25. 3% each; angles 84 / 86 / 78 (a near-vertical launch), BK 20, KS 180 /
    // 180 / 188, hitlag x0.5. Thigh, knee, foot (the foot at chest height in front).
    let mut ftilt = ref_move(
        25,
        0,
        0,
        255,
        &[
            r(5, 8, 4, 9, 5, 30, 84, 20, 180, 0, 0),
            r(5, 8, 7, 11, 4, 30, 86, 20, 180, 1, 0),
            r(5, 8, 11, 13, 5, 30, 78, 20, 188, 2, 0),
        ],
    );
    set_rows(&mut ftilt, &[0, 1, 2], |h| h.hitlag = 50);
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: a stretch kick then an axe kick. Hit 1 (frames 5-8, the leg stretched straight up: hard to land on a grounded target) is
    // set knockback (FKV 30 / 28 / 40, KS 80, BK 0) at 0 / 330 / 75 degrees with +2 hitstun and SDI x0.5, which holds the target for
    // hit 2 (frames 16-23, the axe kick coming down in front): 4%, 60 degrees, BK 25, KS 147. FAF 33.
    let mut utilt = ref_move(
        33,
        0,
        0,
        255,
        &[
            r(5, 8, 4, 22, 4, 30, 0, 0, 80, 0, 0),
            r(5, 8, 3, 18, 7, 30, 330, 0, 80, 1, 0),
            r(5, 8, 2, 14, 6, 30, 75, 0, 80, 2, 0),
            r(16, 23, 3, 12, 5, 40, 60, 25, 147, 0, 1),
            r(16, 23, 10, 12, 5, 40, 60, 25, 147, 1, 1),
            r(16, 23, 7, 13, 6, 40, 60, 25, 147, 2, 1),
        ],
    );
    for (i, fkv) in [(0usize, 30u16), (1, 28), (2, 40)] {
        set_rows(&mut utilt, &[i], |h| {
            h.fixed_kb = fkv;
            h.hitstun_add = 2;
            h.sdi = 50;
        });
    }
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a low kick along the floor, frames 5-6, FAF 27. 4.5%, 70 / 70 / 90 degrees (the toe is the 90), BK 37, KS 100.
    w.moves[MoveId::DTilt as usize] = ref_move(
        27,
        0,
        0,
        255,
        &[
            r(5, 6, 3, 2, 4, 45, 70, 37, 100, 0, 0),
            r(5, 6, 7, 2, 4, 45, 70, 37, 100, 1, 0),
            r(5, 6, 11, 2, 5, 45, 90, 37, 100, 2, 0),
        ],
    );

    // Neutral air: a held flying kick. Clean on frames 3-6 (6 / 6 / 5%), late 7-30 (4 / 4 / 3%), angle 361, BK 30, KS 100.
    // Landing lag 6, autocancels on frames 1-2 and from 31, FAF 50. One hit per target (clean and late are one group).
    w.moves[MoveId::NAir as usize] = ref_move(
        50,
        6,
        2,
        31,
        &[
            r(3, 6, 9, 8, 5, 60, 361, 30, 100, 0, 0),
            r(3, 6, 4, 9, 4, 60, 361, 30, 100, 1, 0),
            r(3, 6, 0, 9, 4, 50, 361, 30, 100, 2, 0),
            r(7, 30, 8, 8, 4, 40, 361, 30, 100, 0, 0),
            r(7, 30, 4, 9, 3, 40, 361, 30, 100, 1, 0),
            r(7, 30, 0, 9, 2, 30, 361, 30, 100, 2, 0),
        ],
    );

    // Forward air: a downward chop in front. Early 5-6, late 7: the shoulder 3.8%, the arm 4.5% (BK 44 / 43, KS 127, 50 degrees).
    // Landing lag 5, autocancels on frames 1-4 and from 11, FAF 35.
    w.moves[MoveId::FAir as usize] = ref_move(
        35,
        5,
        4,
        11,
        &[
            r(5, 6, 3, 13, 6, 38, 50, 44, 127, 0, 0),
            r(5, 6, 7, 12, 6, 45, 50, 43, 127, 1, 0),
            r(7, 7, 3, 12, 6, 38, 50, 44, 127, 0, 0),
            r(7, 7, 7, 10, 6, 45, 50, 43, 127, 1, 0),
        ],
    );

    // Back air: a back kick. Clean 4-6: the hip and knee 7.5% (BK 32), the foot 9.5% (BK 30), KS 98; late 7-14: 6%, BK 10, KS 101.
    // angle 361, sent away behind. Landing lag 7, autocancels on frames 1-3 and from 31, FAF 38.
    w.moves[MoveId::BAir as usize] = ref_move(
        38,
        7,
        3,
        31,
        &[
            r(4, 6, -1, 9, 5, 75, 361, 32, 98, 0, 0),
            r(4, 6, -4, 8, 3, 75, 361, 32, 98, 1, 0),
            r(4, 6, -9, 8, 5, 95, 361, 30, 98, 2, 0),
            r(7, 14, -1, 9, 5, 60, 361, 10, 101, 0, 0),
            r(7, 14, -4, 8, 4, 60, 361, 10, 101, 1, 0),
            r(7, 14, -8, 8, 4, 60, 361, 10, 101, 2, 0),
        ],
    );

    // Up air: a flurry of kicks overhead, then a final kick. The loop (frames 4-15, re-armed every 4: hits on 4, 8 and 12) is 1%,
    // hitlag x0.7, SDI x1.2: against a grounded target 80 degrees (BK 20, KS 80), against an airborne one the autolink angle (BK 30,
    // KS 100), which carries the target along with the fighter (a fast fall drags it down). The final hit (frames 23-24) is its own
    // group, never re-armed: 4%, 80 degrees, BK 50, KS 138, hitlag x2. Landing lag 13, autocancels on 1-3 and from 44, FAF 44.
    let mut uair = ref_move(
        44,
        13,
        3,
        44,
        &[
            r(4, 15, 1, 23, 6, 10, 80, 20, 80, 0, 0),
            r(4, 15, 1, 23, 6, 10, ANGLE_AUTOLINK, 30, 100, 1, 0),
            r(23, 24, 1, 21, 9, 40, 80, 50, 138, 0, 1),
        ],
    );
    set_rows(&mut uair, &[0], |h| h.targets = TARGETS_GROUNDED);
    // (Ours: the aerial loop's pull aims 0.6 units in front of the kick, so a dragged-down target lands in front of the fighter, open to
    // a follow-up; the reference pulls toward the kick itself.)
    set_rows(&mut uair, &[1], |h| {
        h.targets = TARGETS_AERIAL;
        h.link_ahead = Fx::from_ratio(6, 10);
    });
    set_rows(&mut uair, &[0, 1], |h| {
        h.hitlag = 70;
        h.sdi = 120;
    });
    set_rows(&mut uair, &[2], |h| h.hitlag = 200);
    // (Reference frame 4 is move frame 3.)
    uair.rehit = Some((3, 4));
    uair.rehit_groups = 0b1;
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: a stall, then a dive kick. Frames 1-14 rise (from 2 a frame, slowing under gravity, no drift), then from frame 15 a
    // dive of 1.2 forward and 6 down a frame. Clean 15-18: 10%, 280 degrees (a meteor), BK 30, KS 90; late 19-33: 10%, 56 degrees,
    // BK 55, KS 78. Landing (outside the autocancel windows, frames 1-2 and from 53) is the landing hit: its own move (`Ext0`, 22
    // frames of landing lag) with a 2% hit at 48 degrees (BK 77, KS 50) on its frames 1-2. FAF 55.
    let mut dair = ref_move(
        55,
        22,
        2,
        53,
        &[
            r(15, 18, 2, 1, 7, 100, 280, 30, 90, 0, 0),
            r(19, 33, 2, 1, 7, 100, 56, 55, 78, 0, 0),
        ],
    );
    let su = |thousandths: i32| Fx::from_ratio(thousandths, 8000);
    for k in 0..14u8 {
        dair.motion.push(Motion {
            start: k,
            end: k,
            vx: Fx::ZERO,
            vy: su(2000 - 150 * i32::from(k)),
        });
    }
    dair.motion.push(Motion {
        start: 14,
        end: 53,
        vx: su(1200),
        vy: -su(6000),
    });
    dair.landing_move = Some(MoveId::Ext0 as u8);
    w.moves[MoveId::DAir as usize] = dair;
    w.moves[MoveId::Ext0 as usize] =
        ref_move(23, 0, 0, 255, &[r(1, 2, 0, 8, 8, 20, 48, 77, 50, 0, 0)]);

    // ---- The rest of the kit (docs/sheik/REFERENCE_DATA.md, "The rest of the kit") ----
    // Frames, damage, angles, knockback, hitlag and SDI multipliers are the reference's; positions are estimates on our body.

    // Jab 1: a hand swipe on frame 2, 2%, FAF 18; hitlag x1.6. The close hits use the 361 angle (BK 35 / 30, KS 30); the tip
    // pulls a fighter in (180 degrees, BK 20, KS 15). Pressing attack again (or holding it through a hit) carries on into jab 2 from
    // frame 4 (a script).
    let mut jab1 = ref_move(
        18,
        0,
        0,
        255,
        &[
            r(2, 2, 3, 11, 2, 20, 361, 35, 30, 0, 0),
            r(2, 2, 6, 11, 2, 20, 361, 30, 30, 1, 0),
            r(2, 2, 10, 11, 3, 20, 180, 20, 15, 2, 0),
        ],
    );
    set_rows(&mut jab1, &[0, 1, 2], |h| h.hitlag = 160);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/needles_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;
    // Jab 2: the backhand on frame 3, 1.6%, FAF 18; hitlag x1.2; angle 361, BK 35 / 35 / 25, KS 25 / 25 / 20. Three presses since
    // jab 1 (or attack held as it hits) go on into the rapid jab from frame 5. There is no jab 3.
    let mut jab2 = ref_move(
        18,
        0,
        0,
        255,
        &[
            r(3, 3, 5, 9, 2, 16, 361, 35, 25, 0, 0),
            r(3, 3, 9, 10, 3, 16, 361, 35, 25, 1, 0),
            r(3, 3, 14, 10, 3, 16, 361, 25, 20, 2, 0),
        ],
    );
    set_rows(&mut jab2, &[0, 1, 2], |h| h.hitlag = 120);
    jab2.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/needles_jab_2.script"),
    ));
    w.moves[MoveId::Jab2 as usize] = jab2;
    w.moves[MoveId::Jab3 as usize] = Move::empty();
    // Rapid jab (ext1): stabs on loop frames 5, 7, 9, 11 and 13, each 0.3% at the 361 angle (BK 12, KS 15) with +2 hitstun,
    // hitlag x0.5 and SDI x0.4, so the target stays caught. The reference's loop is 12 frames; ours restarts the move, 14 frames a
    // loop [D]. Each loop's end goes round again while attack is held or mashed, else into the finisher (a script).
    let mut rapid = ref_move(
        16,
        0,
        0,
        255,
        &[
            r(5, 13, 8, 11, 3, 3, 361, 12, 15, 0, 0),
            r(5, 13, 12, 10, 6, 3, 361, 12, 15, 1, 0),
        ],
    );
    set_rows(&mut rapid, &[0, 1], |h| {
        h.hitlag = 50;
        h.sdi = 40;
        h.hitstun_add = 2;
    });
    rapid.rehit = Some((4, 2));
    rapid.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/needles_rapid_jab.script"),
    ));
    w.moves[MoveId::Ext1 as usize] = rapid;
    // Rapid jab finisher (ext2): an upward palm thrust on frames 5-6, 2%, 40 degrees, BK 60, KS 132, hitlag x2. FAF 36.
    let mut finisher = ref_move(
        36,
        0,
        0,
        255,
        &[
            r(5, 6, 6, 10, 4, 20, 40, 60, 132, 0, 0),
            r(5, 6, 9, 12, 4, 20, 40, 60, 132, 1, 0),
            r(5, 6, 12, 14, 4, 20, 40, 60, 132, 2, 0),
        ],
    );
    set_rows(&mut finisher, &[0, 1, 2], |h| h.hitlag = 200);
    w.moves[MoveId::Ext2 as usize] = finisher;

    // Dash attack: the earlier game's (asked for by name), not the reference game's. Clean on frame 6: 10%, 80 degrees, BK 34, KS 100,
    // a pop-up that starts combos; late on frames 7-12: 7%, angle 361, BK 15, KS 100. FAF 36. Radii from that game's table
    // (3.9 and 3.1 of its units, the same scale as the reference's); positions estimated from its offsets.
    w.moves[MoveId::DashAttack as usize] = ref_move(
        36,
        0,
        0,
        255,
        &[
            r(6, 6, 4, 8, 5, 100, 80, 34, 100, 0, 0),
            r(6, 6, 1, 5, 4, 100, 80, 34, 100, 1, 0),
            r(7, 12, 5, 7, 4, 70, 361, 15, 100, 0, 0),
            r(7, 12, 2, 4, 4, 70, 361, 15, 100, 1, 0),
        ],
    );

    // Forward smash: two kicks. Hit 1 on frame 12, 5%, set knockback (FKV 47 / 47 / 30 at 35 / 45 / 80 degrees, KS 100) with SDI x0.5,
    // holds the target for hit 2 on frames 20-21: 8%, angle 361, BK 60, KS 109. Charges on frame 3, FAF 45.
    let mut fsmash = ref_move(
        45,
        0,
        0,
        255,
        &[
            r(12, 12, 4, 8, 4, 50, 35, 0, 100, 0, 0),
            r(12, 12, 6, 9, 4, 50, 45, 0, 100, 1, 0),
            r(12, 12, 9, 10, 5, 50, 80, 0, 100, 2, 0),
            r(20, 21, 8, 10, 6, 80, 361, 60, 109, 0, 1),
            r(20, 21, 14, 10, 6, 80, 361, 60, 109, 1, 1),
        ],
    );
    for (i, fkv) in [(0usize, 47u16), (1, 47), (2, 30)] {
        set_rows(&mut fsmash, &[i], |h| {
            h.fixed_kb = fkv;
            h.sdi = 50;
        });
    }
    fsmash.charge_at = Some(2);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: both hands thrust overhead on frame 11, 15%, 93 degrees, BK 52, KS 88, hitlag x1.3, SDI x1.5; then a low hit around the
    // feet on frames 14-15, 11%, 50 degrees, BK 38, KS 100. Charges on frame 6, FAF 53. (The reference's arms are intangible on
    // frames 8-15; ours are not [E].)
    let mut usmash = ref_move(
        53,
        0,
        0,
        255,
        &[
            r(11, 11, 1, 22, 8, 150, 93, 52, 88, 0, 0),
            r(14, 15, 8, 7, 4, 110, 50, 38, 100, 0, 1),
            r(14, 15, 0, 7, 4, 110, 50, 38, 100, 1, 1),
            r(14, 15, -8, 7, 4, 110, 50, 38, 100, 2, 1),
        ],
    );
    set_rows(&mut usmash, &[0], |h| {
        h.hitlag = 130;
        h.sdi = 150;
    });
    usmash.charge_at = Some(5);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a spinning kick, in front and behind. Hit 1 on frames 8-9, 4%, set knockback 35 (KS 100): in front 140 degrees (200
    // against an airborne target), behind 165, both drawing the target in; hit 2 on frames 15-16, 6%, 43 degrees, BK 35, KS 165, both
    // sides. Charges on frame 2 (the reference holds on frame 1, before our first moving frame [D]), FAF 49.
    let mut dsmash = ref_move(
        49,
        0,
        0,
        255,
        &[
            r(8, 9, 5, 8, 5, 40, 200, 0, 100, 0, 0),
            r(8, 9, 5, 8, 5, 40, 140, 0, 100, 1, 0),
            r(8, 9, -5, 5, 5, 40, 165, 0, 100, 2, 0),
            r(8, 9, -11, 5, 5, 40, 165, 0, 100, 3, 0),
            r(15, 16, 7, 6, 6, 60, 43, 35, 165, 0, 1),
            r(15, 16, -7, 8, 6, 60, 43, 35, 165, 1, 1),
        ],
    );
    set_rows(&mut dsmash, &[0, 1, 2, 3], |h| h.fixed_kb = 35);
    set_rows(&mut dsmash, &[0], |h| h.targets = TARGETS_AERIAL);
    dsmash.charge_at = Some(1);
    w.moves[MoveId::DSmash as usize] = dsmash;

    // Grabs: standing on frames 6-7 (FAF 37), dash 9-10 (FAF 45), pivot 11-12 (FAF 40). The reach is an estimate.
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(37, 0, 0, 255, &[r(6, 7, 16, 11, 10, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(45, 0, 0, 255, &[r(9, 10, 18, 11, 10, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(40, 0, 0, 255, &[r(11, 12, 16, 11, 10, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    // Pummel: a knee, 1%, set knockback 30 (KS 100), hitlag x1.7. The reference's hit is on frame 1 and its FAF 6 (ssbwiki.com); ours
    // is on frame 2 (a pummel's hit cannot start on its first frame) with the same 5 frames after it, FAF 7 [D]. (Its hitlag freezes
    // the move, so it is not counted in the FAF: the earlier FAF 16 counted it twice.)
    let mut pummel = with_kind(
        ref_move(7, 0, 0, 255, &[r(2, 2, 6, 10, 6, 10, 361, 0, 100, 0, 0)]),
        HIT_PUMMEL,
    );
    pummel.hitboxes[0].hitlag = 170;
    pummel.hitboxes[0].fixed_kb = 30;
    w.moves[MoveId::Pummel as usize] = pummel;
    // Throws: a hit on the held fighter, then the throw. Forward: 5% on frame 20, released on 22 (2%, 72 degrees, BK 80, KS 79), FAF 40.
    // Back: 5% on 15, released on 17 (2%, BK 90, KS 50; the reference's 120 degrees from the front is 60 behind), FAF 36. Up: 3% on 19,
    // released on 21 (3%, 85 degrees, BK 110, KS 62), FAF 50. Down: 3% on 26, released on 30 (3%, 77 degrees, BK 95, KS 60), FAF 54.
    w.moves[MoveId::FThrow as usize] =
        throw_after_hit(40, (20, 50), r(22, 22, 13, 11, 10, 20, 72, 80, 79, 0, 1));
    w.moves[MoveId::BThrow as usize] =
        throw_after_hit(36, (15, 50), r(17, 17, -13, 11, 10, 20, 60, 90, 50, 0, 1));
    w.moves[MoveId::UThrow as usize] =
        throw_after_hit(50, (19, 30), r(21, 21, 13, 11, 10, 30, 85, 110, 62, 0, 1));
    w.moves[MoveId::DThrow as usize] =
        throw_after_hit(54, (26, 30), r(30, 30, 13, 11, 10, 30, 77, 95, 60, 0, 1));
    // The ledge attack (9%) and get-up attack (7% each side) keep the claws' frames: the damage is the reference's, the timing an
    // estimate [E].

    // ---- Specials ----
    // Needle Storm (neutral special), charged and stored as the reference's: a wind-up to frame 7, then charging (ext3), which carries
    // on by itself, a needle on its first frame and another every 15 up to 6; pressing special again throws them (ext4), one every 3
    // frames from frame 11; shield or jump stores them, and a full charge stores itself. A full store is thrown at once next time; a
    // part charge carries on charging from where it was (all scripts). Each needle: 1.5% falling to 0.8% over its 9 frames, 60 degrees, BK 0, KS 167, hitlag x0.8, 10 reference units a frame
    // (straight on the ground, 35 degrees down in the air). Throw FAF 49 (43 in the air), landing lag 12.
    let mut storm = ref_move(10, 0, 0, 255, &[]);
    storm.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/needle_storm_start.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = storm;
    let mut charge = ref_move(18, 0, 0, 255, &[]);
    charge.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/needle_storm_charge.script"),
    ));
    // Landing carries the charge on.
    charge.landing_move = Some(MoveId::Ext3 as u8);
    w.moves[MoveId::Ext3 as usize] = charge;
    let mut throw = ref_move(49, 12, 0, 255, &[]);
    let mut needle = row(&r(1, 1, 0, 0, 2, 15, 60, 0, 167, 0, 0));
    needle.hitlag = 80;
    throw.projectile = Some(ProjectileSpawn {
        // (Only the script fires them.)
        frame: 255,
        x: Fx::ONE,
        y: Fx::from_ratio(11, 10),
        speed: su(10000),
        life: 9,
        hitbox: needle,
        end_damage: Fx::from_ratio(8, 10),
        arc: ProjectileArc::NONE,
        phases: [None; 2],
    });
    throw.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/needle_storm_throw.script"),
    ));
    w.moves[MoveId::Ext4 as usize] = throw;

    // Side special, our own in place of the reference's grenade (asked for): a line thrown on frame 13 that, if it hits, reels the
    // target in to just in front of the fighter (the pull angle). 3%, set knockback 80 (KS 100) so the hitstun is the same at any
    // percent (about 30 frames: time for the pull and a follow-up). The line flies 0.5 units a frame for 14 frames (7 units). FAF 45,
    // landing lag 10. All estimates [E].
    let mut hook = ref_move(45, 10, 0, 255, &[]);
    let mut line = row(&r(1, 1, 0, 0, 5, 30, ANGLE_PULL, 0, 100, 0, 0));
    line.fixed_kb = 80;
    hook.projectile = Some(ProjectileSpawn {
        frame: 12,
        x: Fx::from_ratio(9, 10),
        y: Fx::from_ratio(10, 10),
        speed: Fx::from_ratio(1, 2),
        life: 14,
        hitbox: line,
        end_damage: Fx::from_int(3),
        arc: ProjectileArc::NONE,
        phases: [None; 2],
    });
    w.moves[MoveId::SideSpecial as usize] = hook;

    // Vanish (up special): a small hop [E], the explosion on frames 36-38 (12%, 55 degrees, BK 50, KS 112, radius 13), then 20 frames of
    // teleport the way the stick points on frame 36 (1.5 reference units a frame, up to 2.5 with the stick pushed all the way), and the
    // reappearance hit on frames 55-56 (5%, 45 degrees, set knockback 80, SDI x0). Intangible 14-54 (19-54 in the air). FAF 95, then
    // helpless; landing lag 20. Its 0% wind box on reappearing is not modelled [E].
    let mut vanish = ref_move(
        95,
        20,
        0,
        255,
        &[
            r(36, 38, 0, 6, 16, 120, 55, 50, 112, 0, 0),
            r(55, 56, 0, 12, 11, 50, 45, 0, 100, 0, 1),
        ],
    );
    set_rows(&mut vanish, &[1], |h| {
        h.fixed_kb = 80;
        h.sdi = 0;
    });
    vanish.script = Some(script(Kind::Fighter, include_str!("scripts/vanish.script")));
    vanish.helpless_after = true;
    vanish.grabs_ledge = true;
    w.moves[MoveId::UpSpecial as usize] = vanish;

    // Bouncing Fish (down special): a flip forward into a heel kick (frames 23-29): 11%, angle 361, BK 26, KS 97, hitlag x1.7. A hit
    // bounces the fighter back off the target (ext5; FAF 80, landing lag 20). FAF 51; landing during it, 36 frames (the reference's kick
    // landing; landing before the kick is 22 there [D]). Consecutive kicks after a bounce and wall bounces are not modelled [E].
    let mut fish = ref_move(
        51,
        36,
        0,
        255,
        &[
            r(23, 29, 7, 3, 4, 110, 361, 26, 97, 0, 0),
            r(23, 29, 1, 6, 4, 110, 361, 26, 97, 1, 0),
        ],
    );
    set_rows(&mut fish, &[0, 1], |h| h.hitlag = 170);
    fish.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bouncing_fish.script"),
    ));
    fish.grabs_ledge = true;
    w.moves[MoveId::DownSpecial as usize] = fish;
    // The early kick (ext6): pressing special again before frame 16 carries on into it there, kicking on frames 18-24 (its 3-9), FAF 46.
    let mut early = ref_move(
        31,
        36,
        0,
        255,
        &[
            r(3, 9, 7, 3, 4, 110, 361, 26, 97, 0, 0),
            r(3, 9, 1, 6, 4, 110, 361, 26, 97, 1, 0),
        ],
    );
    set_rows(&mut early, &[0, 1], |h| h.hitlag = 170);
    early.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bouncing_fish_early.script"),
    ));
    early.grabs_ledge = true;
    w.moves[MoveId::Ext6 as usize] = early;
    let mut bounce = ref_move(80, 20, 0, 255, &[]);
    bounce.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bouncing_fish_bounce.script"),
    ));
    bounce.grabs_ledge = true;
    w.moves[MoveId::Ext5 as usize] = bounce;

    w
}

/// The Embers moveset (the tumbler body): quick punches and kicks, modelled mechanically on a reference character. Every normal
/// follows the reference frame for frame (`docs/movesets/mario/REFERENCE_DATA.md`): active frames, FAF, landing lag and autocancel
/// windows; damage, angles, base and growth knockback, set knockback and set weight, hitlag multipliers, shield damage; the jab's
/// continuable windows and the held jab's repeat; the angled forward tilt and smash; the down air's five looping hits, final hit and
/// landing hit. Radii are the reference's converted (x 1.25, rounded to a tenth); positions on the reference's root bone (`top`) are
/// its offsets converted [D], the others are estimates on our body [E]. The specials are still the claws' (phase 2 builds the
/// reference's).
pub fn embers() -> Weapon {
    let mut w = claws();
    // Fists and feet: the limb throwing a hit can be hit.
    w.limbs = true;

    // ---- Jab ----
    // Jab 1: a punch on frames 2-3, 2.2%, hitlag x1.8. The two close boxes use angle 361 (BK 20, KS 30), the tip pulls a
    // fighter in (180 degrees, BK 15, KS 20). FAF 20. Pressing attack again carries on into jab 2 from frame 5; holding attack
    // throws jab 1 again from frame 7 (a script).
    let mut jab1 = ref_move(
        20,
        0,
        0,
        255,
        &[
            r(2, 3, 8, 8, 2, 22, 361, 20, 30, 0, 0),
            r(2, 3, 11, 8, 2, 22, 361, 20, 30, 1, 0),
            r(2, 3, 15, 8, 2, 22, 180, 15, 20, 2, 0),
        ],
    );
    set_rows(&mut jab1, &[0, 1, 2], |h| h.hitlag = 180);
    chain(&mut jab1, MoveId::Jab2, 5);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/embers_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;
    // Jab 2: a straight on frames 2-3, 1.7%, hitlag x1.2; angle 361, BK 20 / 20 / 15, KS 30 / 30 / 15. FAF 22. Attack again carries
    // on into jab 3 from frame 6.
    let mut jab2 = ref_move(
        22,
        0,
        0,
        255,
        &[
            r(2, 3, 4, 9, 4, 17, 361, 20, 30, 0, 0),
            r(2, 3, 1, 9, 2, 17, 361, 20, 30, 1, 0),
            r(2, 3, 8, 9, 5, 17, 361, 15, 15, 2, 0),
        ],
    );
    set_rows(&mut jab2, &[0, 1, 2], |h| h.hitlag = 120);
    chain(&mut jab2, MoveId::Jab3, 6);
    w.moves[MoveId::Jab2 as usize] = jab2;
    // Jab 3: a rising kick on frames 3-4, 4%, angle 361, BK 60, KS 80, hitlag x2. FAF 34. The swept box (the reference's runs from
    // 9.0 / 13.0 down to 6.0 / 11.0 of its units) is two circles at its ends.
    let mut jab3 = ref_move(
        34,
        0,
        0,
        255,
        &[
            r(3, 4, 16, 11, 5, 40, 361, 60, 80, 0, 0),
            r(3, 4, 14, 8, 5, 40, 361, 60, 80, 0, 0),
            r(3, 4, 9, 10, 3, 40, 361, 60, 80, 1, 0),
            r(3, 4, 14, 10, 3, 40, 361, 60, 80, 2, 0),
        ],
    );
    set_rows(&mut jab3, &[0, 1, 2, 3], |h| h.hitlag = 200);
    w.moves[MoveId::Jab3 as usize] = jab3;

    // ---- Tilts ----
    // Forward tilt: a lunging kick on frames 5-7, 7%, angle 361, BK 55, KS 70 (foot, thigh, hip). FAF 26. Aimed up or down (the
    // stick tilted that way) the kick goes up or down; the hit is the same.
    let mut ftilt = ref_move(26, 0, 0, 255, &[]);
    for (aim, foot, thigh, hip) in [
        (AIM_STRAIGHT, (11, 8), (6, 8), (2, 8)),
        (AIM_UP, (10, 13), (6, 10), (2, 9)),
        (AIM_DOWN, (11, 3), (6, 6), (2, 8)),
    ] {
        let mut rows = ref_move(
            26,
            0,
            0,
            255,
            &[
                r(5, 7, foot.0, foot.1, 5, 70, 361, 55, 70, 0, 0),
                r(5, 7, thigh.0, thigh.1, 4, 70, 361, 55, 70, 1, 0),
                r(5, 7, hip.0, hip.1, 3, 70, 361, 55, 70, 2, 0),
            ],
        );
        set_rows(&mut rows, &[0, 1, 2], |h| h.aim = aim);
        ftilt.hitboxes.extend(rows.hitboxes);
    }
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: a spinning uppercut on frames 5-11, 5.5%, 96 degrees, BK 28, KS 130 (head, arm, fist). The fist sweeps from in front, over
    // the head, to behind. FAF 30.
    w.moves[MoveId::UTilt as usize] = ref_move(
        30,
        0,
        0,
        255,
        &[
            r(5, 11, -1, 17, 4, 55, 96, 28, 130, 0, 0),
            r(5, 6, 4, 16, 5, 55, 96, 28, 130, 1, 0),
            r(5, 6, 6, 19, 6, 55, 96, 28, 130, 2, 0),
            r(7, 9, 2, 19, 5, 55, 96, 28, 130, 1, 0),
            r(7, 9, 1, 23, 6, 55, 96, 28, 130, 2, 0),
            r(10, 11, -2, 18, 5, 55, 96, 28, 130, 1, 0),
            r(10, 11, -5, 20, 6, 55, 96, 28, 130, 2, 0),
        ],
    );

    // Down tilt: a leg sweep on frames 5-7: the leg 7%, the toe 5%, 80 degrees, BK 35, KS 80 (its trip chance can never trip: the
    // angle is too high). FAF 28.
    w.moves[MoveId::DTilt as usize] = ref_move(
        28,
        0,
        0,
        255,
        &[
            r(5, 7, 5, 2, 4, 70, 80, 35, 80, 0, 0),
            r(5, 7, 10, 2, 5, 50, 80, 35, 80, 1, 0),
        ],
    );

    // Dash attack: a sliding kick. Clean on frames 6-9: 8%, 50 degrees, BK 100, KS 43; late 10-25: 6%, 48 degrees, BK 100, KS 30; hitlag
    // x1.25, shield damage +1 (as a percent of its damage). FAF 38.
    let mut dash = ref_move(
        38,
        0,
        0,
        255,
        &[
            r(6, 9, 7, 2, 4, 80, 50, 100, 43, 0, 0),
            r(10, 25, 6, 2, 3, 60, 48, 100, 30, 0, 0),
        ],
    );
    set_rows(&mut dash, &[0, 1], |h| h.hitlag = 125);
    set_rows(&mut dash, &[0], |h| h.shield_damage = 113);
    set_rows(&mut dash, &[1], |h| h.shield_damage = 117);
    w.moves[MoveId::DashAttack as usize] = dash;

    // ---- Smashes ----
    // Forward smash: a palm thrust with an explosion, frames 15-17. The arm 14.7% (KS 105) wins over the explosion 17.8% (KS 99 straight,
    // 101 aimed up, 100 aimed down) where both reach, so the explosion has to be spaced; angle 361, BK 25. Charges on frame 5.
    // FAF 48.
    let mut fsmash = ref_move(48, 0, 0, 255, &[]);
    for (aim, ks, arm, fire) in [
        (AIM_STRAIGHT, 99, (5, 9), (13, 9)),
        (AIM_UP, 101, (5, 11), (12, 14)),
        (AIM_DOWN, 100, (5, 7), (12, 4)),
    ] {
        let mut rows = ref_move(
            48,
            0,
            0,
            255,
            &[
                r(15, 17, arm.0, arm.1, 3, 147, 361, 25, 105, 0, 0),
                r(15, 17, fire.0, fire.1, 6, 178, 361, 25, ks, 1, 0),
            ],
        );
        set_rows(&mut rows, &[0, 1], |h| h.aim = aim);
        fsmash.hitboxes.extend(rows.hitboxes);
    }
    fsmash.charge_at = Some(4);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: a headbutt from back to front, frames 9-12, 14%, 83 degrees, BK 32, KS 94 (head and body). Charges on frame 6. FAF 40.
    // (The reference's head is intangible on frames 9-12; partial intangibility is not modelled [E].)
    let mut usmash = ref_move(
        40,
        0,
        0,
        255,
        &[
            r(9, 10, -2, 20, 6, 140, 83, 32, 94, 0, 0),
            r(9, 10, -1, 14, 5, 140, 83, 32, 94, 1, 0),
            r(11, 12, 3, 20, 6, 140, 83, 32, 94, 0, 0),
            r(11, 12, 1, 14, 5, 140, 83, 32, 94, 1, 0),
        ],
    );
    usmash.charge_at = Some(5);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a breakdance kick, in front on frames 5-6 (10%, 32 degrees), then behind on frame 14 (12%, 30 degrees); BK 30, KS 100,
    // both low (semi-spikes). Charges on frame 2. FAF 44.
    let mut dsmash = ref_move(
        44,
        0,
        0,
        255,
        &[
            r(5, 6, 16, 5, 5, 100, 32, 30, 100, 0, 0),
            r(5, 6, 9, 5, 4, 100, 32, 30, 100, 1, 0),
            r(14, 14, -14, 5, 5, 120, 30, 30, 100, 0, 1),
            r(14, 14, -8, 5, 4, 120, 30, 30, 100, 1, 1),
        ],
    );
    dsmash.charge_at = Some(1);
    w.moves[MoveId::DSmash as usize] = dsmash;

    // ---- Aerials ----
    // Neutral air: a held flying kick. Clean on frames 3-5 (8%, BK 20, KS 100), late 6-27 (5%, BK 13, KS 90); angle 361. Landing
    // lag 6, autocancels on frames 1-2 and from 39, FAF 46.
    w.moves[MoveId::NAir as usize] = ref_move(
        46,
        6,
        2,
        39,
        &[
            r(3, 5, 5, 8, 5, 80, 361, 20, 100, 0, 0),
            r(3, 5, 1, 10, 5, 80, 361, 20, 100, 1, 0),
            r(6, 27, 4, 8, 4, 50, 361, 13, 90, 0, 0),
            r(6, 27, 1, 10, 4, 50, 361, 13, 90, 1, 0),
        ],
    );

    // Forward air: an overhand. Early on frame 16 (12%, angle 361, BK 30, KS 80), clean 17-20 (14%, 280 degrees: a meteor, BK 32,
    // KS 78), late 21 (10%, angle 361, BK 20, KS 80). Landing lag 17, autocancels on frames 1-2 and from 43, FAF 60.
    w.moves[MoveId::FAir as usize] = ref_move(
        60,
        17,
        2,
        43,
        &[
            r(16, 16, 7, 15, 4, 120, 361, 30, 80, 0, 0),
            r(17, 20, 3, 12, 5, 140, 280, 32, 78, 0, 0),
            r(17, 20, 8, 8, 6, 140, 280, 32, 78, 1, 0),
            r(21, 21, 3, 11, 5, 100, 361, 20, 80, 0, 0),
            r(21, 21, 6, 3, 6, 100, 361, 20, 80, 1, 0),
        ],
    );

    // Back air: a dropkick behind. Clean 6-7: 10.5%, BK 12, KS 106; late 8-10: 7%, BK 7, KS 90; angle 361, sent away behind.
    // Landing lag 6, autocancels on frames 1-5 and from 19, FAF 34.
    w.moves[MoveId::BAir as usize] = ref_move(
        34,
        6,
        5,
        19,
        &[
            r(6, 7, -11, 8, 6, 105, 361, 12, 106, 0, 0),
            r(6, 7, -5, 8, 7, 105, 361, 12, 106, 1, 0),
            r(8, 10, -11, 8, 6, 70, 361, 7, 90, 0, 0),
            r(8, 10, -5, 8, 7, 70, 361, 7, 90, 1, 0),
        ],
    );

    // Up air: an overhead kick sweeping front to back on frames 4-7, 7%, 75 degrees, BK 10, KS 135 (no sourspot). Landing lag 6,
    // autocancels on frame 1 and from 17, FAF 31.
    w.moves[MoveId::UAir as usize] = ref_move(
        31,
        6,
        1,
        17,
        &[
            r(4, 5, 6, 18, 7, 70, 75, 10, 135, 0, 0),
            r(4, 5, 2, 15, 6, 70, 75, 10, 135, 1, 0),
            r(6, 7, -4, 19, 7, 70, 75, 10, 135, 0, 0),
            r(6, 7, -1, 15, 6, 70, 75, 10, 135, 1, 0),
        ],
    );

    // Down air: a spin. Five looping hits on frames 5, 7, 9, 11 and 13 (re-armed every 2 frames): 1.4%, 94 degrees, BK 50 (low) / 25
    // (high), KS 15, set weight, SDI x0.8; the final hit on frame 23 (its own group): 5.5%, 75 degrees, BK 80, KS 100, SDI x0.8. Landing
    // lag 15, autocancels on frames 1-4 and from 33, FAF 38. Landing outside the autocancel windows is the landing hit (ext0, 15 frames
    // of lag): 2% on its frames 1-2, angle 361, set knockback 60 (KS 100), on both sides.
    let mut dair_rows = Vec::new();
    for f in [5u8, 7, 9, 11, 13] {
        dair_rows.push(r(f, f, 0, -1, 5, 14, 94, 50, 15, 0, 0));
        dair_rows.push(r(f, f, 0, 8, 9, 14, 94, 25, 15, 1, 0));
    }
    dair_rows.push(r(23, 23, 0, 9, 14, 55, 75, 80, 100, 0, 1));
    let mut dair = ref_move(38, 15, 4, 33, &dair_rows);
    let loops: Vec<usize> = (0..10).collect();
    set_rows(&mut dair, &loops, |h| h.set_weight = true);
    let all: Vec<usize> = (0..11).collect();
    set_rows(&mut dair, &all, |h| h.sdi = 80);
    // (Reference frame 5 is move frame 4.)
    dair.rehit = Some((4, 2));
    dair.rehit_groups = 0b1;
    dair.landing_move = Some(MoveId::Ext0 as u8);
    w.moves[MoveId::DAir as usize] = dair;
    let mut dair_landing = ref_move(
        16,
        0,
        0,
        255,
        &[
            r(1, 2, 5, 4, 5, 20, 361, 0, 100, 0, 0),
            r(1, 2, -5, 4, 5, 20, 361, 0, 100, 1, 0),
        ],
    );
    set_rows(&mut dair_landing, &[0, 1], |h| h.fixed_kb = 60);
    w.moves[MoveId::Ext0 as usize] = dair_landing;

    // ---- Grabs and throws ----
    // Grabs: standing on frames 6-7 (FAF 35), dash 9-10 (FAF 43), pivot 10-11 (FAF 37). Each reach is the reference's grab box (a
    // swept circle along the arm at 6.6 of its units up) as circles along it.
    let grab_line = |first: u8, last: u8, radius: i32, xs: &[i32]| -> Vec<R> {
        xs.iter()
            .map(|&x| r(first, last, x, 8, radius, 0, 361, 0, 0, 0, 0))
            .collect()
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(35, 0, 0, 255, &grab_line(6, 7, 4, &[5, 11])),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(43, 0, 0, 255, &grab_line(9, 10, 3, &[5, 9, 13])),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(37, 0, 0, 255, &grab_line(10, 11, 4, &[5, 12, 18])),
        HIT_GRAB,
    );
    // Pummel: a headbutt, 1.3%, set knockback 30 (KS 100), hitlag x2.1. The reference's hit is on frame 1 and its FAF 7; ours is on
    // frame 2 (a pummel's hit cannot start on its first frame) with the same 6 frames after it, FAF 8 [D].
    let mut pummel = with_kind(
        ref_move(8, 0, 0, 255, &[r(2, 2, 13, 13, 6, 13, 361, 0, 100, 0, 0)]),
        HIT_PUMMEL,
    );
    pummel.hitboxes[0].fixed_kb = 30;
    pummel.hitboxes[0].hitlag = 210;
    w.moves[MoveId::Pummel as usize] = pummel;
    // Throws, each a single release: forward on frame 13 (8%, 45 degrees, BK 60, KS 65), FAF 28; back on 44 after three spins (11%,
    // 45 degrees, BK 70, KS 66, behind; the fighter ends facing the way it threw), FAF 60; up on 18 (7%, 90 degrees, BK 70, KS 72), FAF
    // 40; down on 18 (5%, 68 degrees, BK 40, KS 90), FAF 40.
    w.moves[MoveId::FThrow as usize] = with_kind(
        ref_move(
            28,
            0,
            0,
            255,
            &[r(13, 13, 13, 11, 10, 80, 45, 60, 65, 0, 0)],
        ),
        HIT_THROW,
    );
    let mut bthrow = with_kind(
        ref_move(
            60,
            0,
            0,
            255,
            &[r(44, 44, -13, 11, 10, 110, 45, 70, 66, 0, 0)],
        ),
        HIT_THROW,
    );
    // While it spins (frames 13-39) the held fighter hits anyone else close by: 8%, 50 degrees, BK 100, KS 70 (a normal hit, which
    // never touches the fighter being held). Positions [E]: around the thrower at the swung fighter's height.
    bthrow
        .hitboxes
        .push(row(&r(13, 39, -4, 9, 5, 80, 50, 100, 70, 0, 1)));
    bthrow
        .hitboxes
        .push(row(&r(13, 39, 4, 9, 5, 80, 50, 100, 70, 0, 1)));
    bthrow.turns_around = true;
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = with_kind(
        ref_move(
            40,
            0,
            0,
            255,
            &[r(18, 18, 13, 11, 10, 70, 90, 70, 72, 0, 0)],
        ),
        HIT_THROW,
    );
    w.moves[MoveId::DThrow as usize] = with_kind(
        ref_move(
            40,
            0,
            0,
            255,
            &[r(18, 18, 13, 11, 10, 50, 68, 40, 90, 0, 0)],
        ),
        HIT_THROW,
    );

    // ---- Ledge and floor ----
    // Ledge attack: a dropkick climbing up, frames 24-26, 9%, 45 degrees, BK 90, KS 20, shield damage +1; intangible through frame 26,
    // FAF 56. Its swept box (12.0 to 2.0 of the reference's units ahead, 5.0 up) is three circles.
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &[
            r(24, 26, 15, 6, 6, 90, 45, 90, 20, 0, 0),
            r(24, 26, 9, 6, 6, 90, 45, 90, 20, 0, 0),
            r(24, 26, 3, 6, 6, 90, 45, 90, 20, 0, 0),
        ],
    );
    set_rows(&mut ledge, &[0, 1, 2], |h| h.shield_damage = 111);
    ledge.intangible = 26;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack (lying face up): a punch behind on frames 16-17, then in front on 21-22; 7%, 48 degrees, BK 80, KS 48, shield damage
    // +8; intangible through frame 22, FAF 46. (The reference's face-down and trip get-up attacks are not separate here.)
    let mut getup = ref_move(
        46,
        0,
        0,
        255,
        &[
            r(16, 17, -15, 6, 6, 70, 48, 80, 48, 0, 0),
            r(16, 17, -10, 6, 6, 70, 48, 80, 48, 0, 0),
            r(16, 17, -6, 6, 6, 70, 48, 80, 48, 0, 0),
            r(21, 22, 15, 6, 6, 70, 48, 80, 48, 0, 1),
            r(21, 22, 10, 6, 6, 70, 48, 80, 48, 0, 1),
            r(21, 22, 6, 6, 6, 70, 48, 80, 48, 0, 1),
        ],
    );
    let all: Vec<usize> = (0..6).collect();
    set_rows(&mut getup, &all, |h| h.shield_damage = 214);
    getup.intangible = 22;
    w.moves[MoveId::GetUpAttack as usize] = getup;

    embers_specials(&mut w);
    w
}

/// The Embers' special moves (the reference's: ssbwiki.com, 13.0.1; `docs/movesets/mario/REFERENCE_DATA.md`).
fn embers_specials(w: &mut Weapon) {
    // Nothing of the claws' follow-up slots belongs to the Embers (ext0 is its down air's landing hit, set above).
    for id in [
        MoveId::Ext1,
        MoveId::Ext2,
        MoveId::Ext3,
        MoveId::Ext4,
        MoveId::Ext5,
        MoveId::Ext6,
        MoveId::Ext7,
        MoveId::Ext8,
        MoveId::Ext9,
    ] {
        w.moves[id as usize] = Move::empty();
    }
    w.moves[MoveId::UpSpecial as usize] = rising_punch();
    w.moves[MoveId::NSpecial as usize] = fireball();
    w.moves[MoveId::SideSpecial as usize] = cape();
    water_pump(w);
}

/// Down special, a charged water pump (the reference's): a wind-up (the down special), the charge (ext1) and the shot
/// (ext2), scripted in `pump_start`, `pump_charge` and `pump_shot`.
///
/// * Wind-up 19 frames, then charging; 80 frames of charge in all (ssbwiki.com's parameter, counted from the charge's start: with the
///   wind-up that is the 100 frames UFD and dragdown give [D]). Shield or jump keeps the charge (no lag); special shoots; at full
///   charge the stance holds until special or shield (the owner's call, as the reference); a stored charge carries into the next use.
/// * The shot: from its frame 1, the fighter pushed back 0.02 a charge frame; 7 pumps every 5 frames (each out for 12 frames:
///   2-13, 7-18, ... 32-43), FAF 49. A pump: 0% at 55 degrees, set knockback 70, KS 100, radius 4, a push (the reference's
///   flinchless hit: no hitstun), 4.3 to 8.6 a frame with the charge, gravity 0.17, fall 3, size x0.8 to x1.2 and up to 55 more
///   base knockback. Aimed from 12 degrees, turned by the stick 1.9 degrees a frame to +50 / -18. Where the pumps leave the
///   nozzle is [E]. Touching down carries the move on.
fn water_pump(w: &mut Weapon) {
    let su = |n: i32| Fx::from_ratio(n, 8000);
    let mut start = ref_move(21, 0, 0, 255, &[]);
    start.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pump_start.script"),
    ));
    start.land_continue = true;
    w.moves[MoveId::DownSpecial as usize] = start;

    // (Longer than a full charge: the script restarts it while the stance holds.)
    let mut charge = ref_move(100, 0, 0, 255, &[]);
    charge.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pump_charge.script"),
    ));
    charge.land_continue = true;
    w.moves[MoveId::Ext1 as usize] = charge;

    let mut pump = row(&r(1, 1, 0, 0, 5, 0, 55, 0, 100, 0, 0));
    pump.fixed_kb = 70;
    pump.push = true;
    let mut shot = ref_move(50, 0, 0, 255, &[]);
    shot.projectile = Some(ProjectileSpawn {
        // (Only the script fires them; a pump hits from the frame after it leaves, for 12 frames.)
        frame: 255,
        x: Fx::ONE,
        y: Fx::from_ratio(8, 10),
        speed: Fx::ZERO,
        life: 13,
        hitbox: pump,
        end_damage: Fx::ZERO,
        arc: ProjectileArc {
            gravity: su(170),
            max_fall: su(3000),
            ..ProjectileArc::NONE
        },
        phases: [None; 2],
    });
    shot.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pump_shot.script"),
    ));
    shot.land_continue = true;
    w.moves[MoveId::Ext2 as usize] = shot;
}

/// Side special, a cape swing (the reference's).
///
/// * The swing, frames 12-14: 7%, 110 degrees, set knockback 80 (KS 100), shield damage +4 (157 percent, [D]), and the reference's
///   reverse effect: the victim is turned round. Radius 7.5 and 5.0 on the root, converted [D].
/// * The reflector, frames 9-20: centred 8 ahead and 6.5 up, 6.5 across (reference units); 1.5 times the damage, the same speed,
///   0.75 of the lifetime; shots of more than 50% go through. Its projectile detector, frames 6-8 (8 across, along 2.5 to 8 ahead:
///   one circle covering it here, [D]), skips the move to frame 9.
/// * In the air: 1 / 2.5 of the horizontal speed kept, air friction 0.0025, gravity 0.031, fall speed 1; the first use per airtime
///   adds 0.3 of vertical speed (`cape.script`; that it starts from a stopped fall is [E]). Ledges in front from frame 2, behind
///   from 16.
/// * FAF 36. (The reflected shot also turns its vertical speed round: [E], the reference's pages do not say.)
fn cape() -> Move {
    let su = |n: i32| Fx::from_ratio(n, 8000);
    let mut mv = ref_move(
        36,
        0,
        0,
        255,
        &[
            r(12, 14, 12, 8, 9, 70, 110, 0, 100, 0, 0),
            r(12, 14, 6, 8, 6, 70, 110, 0, 100, 1, 0),
        ],
    );
    set_rows(&mut mv, &[0, 1], |h| {
        h.fixed_kb = 80;
        h.reverse = true;
        h.shield_damage = 157;
    });
    mv.reflector = Some(Reflector {
        start: 8,
        end: 19,
        x: Fx::ONE,
        y: Fx::from_ratio(81, 100),
        radius: Fx::from_ratio(81, 100),
        damage_percent: 150,
        speed_percent: 100,
        reverse_vertical: true,
        life_percent: 75,
        max_damage: Fx::from_int(50),
        detect_start: 5,
        detect_end: 7,
        detect_x: Fx::from_ratio(66, 100),
        detect_y: Fx::from_ratio(81, 100),
        detect_radius: Fx::from_ratio(134, 100),
    });
    mv.script = Some(script(Kind::Fighter, include_str!("scripts/cape.script")));
    mv.start_speed = 40;
    mv.air_friction = Some(Fx::from_ratio(25, 80_000));
    mv.air_gravity = Some(su(31));
    mv.air_max_fall = Some(su(1000));
    mv.ledge_front_from = 1;
    mv.ledge_back_from = 15;
    mv
}

/// Neutral special, a bouncing fireball (the reference's). Out on frame 17, FAF 50; in the air the fighter keeps its air
/// control (no motion). The ball [R]: 70 frames of life, 1.4 reference units a frame at 10 degrees down, gravity 0.1; off the ground
/// it bounces back up at 1.4 to 1.6 keeping 0.97 of its speed, 7 times (the next touch removes it); it does not pierce. Its hits by
/// age (frames 17-21, 22-46, 47-86 of the move): 5% BK 35 KS 20 radius 2.4, 5% BK 28 KS 15 radius 2.2, 4% BK 22 KS 10 radius 2.0;
/// 361 degrees, hitlag x0.6, shield damage -2.5 / -2.5 / -2.0 (half its damage on a shield, [D]). Where it leaves the hand is [E].
/// (The reference's walls under 60 degrees also bounce it; this game's walls are upright, so a wall removes it, as before.)
fn fireball() -> Move {
    let su = |n: i32| Fx::from_ratio(n, 8000);
    let hit = |damage: i32, bkb: i16, kbg: i16, radius_thousandths: i32| {
        let mut h = row(&r(1, 1, 0, 0, 1, damage, 361, bkb, kbg, 0, 0));
        h.radius = Fx::from_ratio(radius_thousandths, 1000);
        h.hitlag = 60;
        h.shield_damage = 50;
        h
    };
    let mut mv = ref_move(50, 0, 0, 255, &[]);
    mv.projectile = Some(ProjectileSpawn {
        // (A projectile first hits on the frame after it appears: out on frame 16, it hits from 17 to 86, as the reference.)
        frame: 15,
        x: Fx::from_ratio(8, 10),
        y: Fx::from_ratio(9, 10),
        speed: su(1400),
        life: 71,
        hitbox: hit(50, 35, 20, 300),
        end_damage: Fx::from_int(5),
        arc: ProjectileArc {
            angle: -10,
            gravity: su(100),
            max_fall: Fx::ZERO,
            bounces: 7,
            bounce_min: su(1400),
            bounce_max: su(1600),
            bounce_keep: 97,
        },
        // Its age is 1 on frame 17: the late hit from frame 22, the latest from 47.
        phases: [
            Some((6, hit(50, 28, 15, 275))),
            Some((31, hit(40, 22, 10, 250))),
        ],
    });
    mv
}

/// Up special, a rising multi-hit punch (the reference's). Every hit is on the root (`top`), so the positions are
/// converted directly [D].
///
/// * Hit 1, frames 3-6: 5%, set knockback 160 / 150 / 150 at 60 / 86 / 100 degrees (KS 100, set weight, no SDI); the box at the
///   body (60 degrees) does not hit grounded fighters.
/// * Hits 2-6, frames 7-16, landing again every 2 frames: 0.6%, set knockback 180 / 170 / 110 / 110 at 60 / 92 / 60 / 92 (set weight,
///   no SDI).
/// * Hit 7, frames 17-18: 3%, 60 degrees, BK 50, KS 145, radius 9.
///
/// Intangible on frames 3-6. Animation 37 frames, then the special fall (air speed x0.6, landing lag 30). Ledges from either side
/// from frame 14. Angled by the stick (sensitivity 0.5, up to 12 degrees); x1.1 the rise from the ground, x1.08 from the air; 0.67 of
/// the momentum kept on startup. The rise itself is the reference's animation, not published: [E] about 40 reference units before
/// the multipliers (44 from the ground), a little forward.
fn rising_punch() -> Move {
    let su = |n: i32| Fx::from_ratio(n, 8000);
    let mut up = ref_move(
        38,
        0,
        0,
        255,
        &[
            // Hit 1 (group 0).
            r(3, 6, 0, 8, 3, 50, 60, 0, 100, 0, 0),
            r(3, 6, 6, 8, 5, 50, 86, 0, 100, 1, 0),
            r(3, 6, 12, 8, 5, 50, 100, 0, 100, 2, 0),
            // Hits 2-6 (group 1).
            r(7, 16, 3, 8, 4, 6, 60, 0, 100, 0, 1),
            r(7, 16, 10, 8, 5, 6, 92, 0, 100, 1, 1),
            r(7, 16, 3, 14, 4, 6, 60, 0, 100, 2, 1),
            r(7, 16, 11, 14, 5, 6, 92, 0, 100, 3, 1),
            // Hit 7 (group 2).
            r(17, 18, 10, 14, 11, 30, 60, 50, 145, 0, 2),
            r(17, 18, 3, 14, 11, 30, 60, 50, 145, 1, 2),
        ],
    );
    let set = [160, 150, 150, 180, 170, 110, 110];
    for (hb, fkv) in up.hitboxes.iter_mut().zip(set) {
        hb.fixed_kb = fkv;
        hb.set_weight = true;
        hb.sdi = 0;
    }
    up.hitboxes[0].targets = TARGETS_AERIAL;
    // Hits 2-6 land again every 2 frames from frame 7; hits 1 and 7 once each.
    up.rehit = Some((6, 2));
    up.rehit_groups = 0b10;
    up.motion = vec![
        Motion {
            start: 2,
            end: 7,
            vx: su(800),
            vy: su(3600),
        },
        Motion {
            start: 8,
            end: 13,
            vx: su(600),
            vy: su(2000),
        },
        Motion {
            start: 14,
            end: 19,
            vx: su(400),
            vy: su(1000),
        },
    ];
    up.intangible = 6;
    up.intangible_from = 2;
    up.helpless_after = true;
    up.ledge_front_from = 13;
    up.ledge_back_from = 13;
    up.fall_drift = 60;
    up.fall_landing_lag = 30;
    up.rise_ground = 110;
    up.rise_air = 108;
    up.start_speed = 67;
    // Read on frame 3, the move's turnaround frame [E: the angle's frame is not published].
    up.steer_frame = 2;
    up.steer_stick = 50;
    up.steer_degrees = 12;
    up
}

/// The pummel the longsword and the claws share with the reference's (ssbwiki.com, the reference swordfighter's and brawler's, 13.0.1): 1.3%,
/// set knockback 30 (KS 100), hitlag x2.1, the hit on frame 1 and FAF 7. Ours hits on frame 2 (a pummel's hit cannot start on its
/// first frame) with the same 6 frames after it, FAF 8 [D]; its hitlag freezes the move, so it is not in the FAF.
fn reference_pummel(damage: i32, x: i32, y: i32, radius: i32) -> Move {
    let mut pummel = with_kind(
        ref_move(
            8,
            0,
            0,
            255,
            &[r(2, 2, x, y, radius, damage, 361, 0, 100, 0, 0)],
        ),
        HIT_PUMMEL,
    );
    pummel.hitboxes[0].fixed_kb = 30;
    pummel.hitboxes[0].hitlag = 210;
    pummel
}

/// A jab that carries on into `next` when attack is pressed from reference frame `from` on (its continuable window), with the input
/// buffer's 9 frames for the press.
fn chain(mv: &mut Move, next: MoveId, from: u8) {
    mv.next = Some(next as u8);
    mv.next_window = 9;
    mv.next_from = from - 1;
}

/// The claws' up special, a flaming dash kick (the reference brawler's up special: ssbwiki.com, 13.0.1): the hit
/// data, timing and movement parameters are [R]; the movement is the script `fire_aim.script` built from the parameters, and the
/// hitbox positions on the hip and legs are [E].
///
/// Frames 18-21 the wind-up hit (4%, 93 degrees, BK 50, KS 10, set weight, radius 7). Frames 27-32 the dash's looping hits, every
/// 2 frames (2.5%, hitlag x0.5, no SDI): on grounded targets 40 degrees (BK 60, then 70 from frame 30), on airborne ones the
/// autolink angle (BK 35 / 40). Then the last hit: on the ground frames 36-37 (5%, 361, BK 60, KS 148, hitlag x2.5) and the move ends
/// on frame 67; in the air frames 37-38 (6%, 361, BK 60, KS 136, hitlag x2.5), the animation lasts 71 frames, then the special
/// fall (air speed x0.65, landing lag 36). Ledges behind from frame 26, in front from 27. A steep dash into the ground bounces off
/// (22 frames). (The reference's looping hits angle from the brawler's direction of travel; here they angle from its facing.)
fn claws_up_special(w: &mut Weapon) {
    let set_weight = |mut mv: Move, rows: &[usize]| {
        for &i in rows {
            mv.hitboxes[i].set_weight = true;
        }
        mv
    };
    let loop_hit = |first: u8, last: u8, x: i32, y: i32, radius: i32, angle: i16, bkb: i16| {
        r(first, last, x, y, radius, 25, angle, bkb, 20, 0, 1)
    };
    let mut fire = set_weight(
        ref_move(
            72,
            36,
            0,
            255,
            &[
                r(18, 21, 1, 10, 9, 40, 93, 50, 10, 0, 0),
                // The hip (the reference's ids 0 and 1) and the knee (2 and 3), early then late.
                loop_hit(27, 29, 0, 10, 6, 40, 60),
                loop_hit(27, 29, 0, 10, 8, ANGLE_AUTOLINK, 35),
                loop_hit(27, 29, 5, 8, 4, 40, 60),
                loop_hit(27, 29, 5, 8, 4, ANGLE_AUTOLINK, 40),
                loop_hit(30, 32, 0, 10, 6, 40, 70),
                loop_hit(30, 32, 0, 10, 8, ANGLE_AUTOLINK, 35),
                loop_hit(30, 32, 5, 8, 4, 40, 70),
                loop_hit(30, 32, 5, 8, 4, ANGLE_AUTOLINK, 40),
            ],
        ),
        &[0],
    );
    for (i, hb) in fire.hitboxes.iter_mut().enumerate().skip(1) {
        hb.priority = ((i - 1) % 4) as u8;
        hb.hitlag = 50;
        hb.sdi = 0;
        hb.targets = if hb.angle == ANGLE_AUTOLINK {
            TARGETS_AERIAL
        } else {
            TARGETS_GROUNDED
        };
    }
    fire.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/fire_aim.script"),
    ));
    // The loop (group 1) lands again every 2 frames from frame 27; the wind-up hit does not.
    fire.rehit = Some((26, 2));
    fire.rehit_groups = 0b10;
    fire.helpless_after = true;
    fire.land_continue = true;
    fire.ledge_back_from = 25;
    fire.ledge_front_from = 26;
    fire.fall_drift = 65;
    fire.fall_landing_lag = 36;
    w.moves[MoveId::UpSpecial as usize] = fire;

    // The ending, reached on frame 33 (so its frame 1 is the reference's 35): on the ground its last hit is on frames 36-37 and it
    // ends on 67; in the air the last hit is on 37-38 and the special fall follows the 71-frame animation.
    let ending = |faf: u8, rows: &[R]| {
        let mut m = with_hitlag(ref_move(faf, 36, 0, 255, rows), &[250, 250, 250]);
        m.script = Some(script(
            Kind::Fighter,
            include_str!("scripts/fire_end.script"),
        ));
        m.helpless_after = true;
        m.ledge_front_from = 0;
        m.ledge_back_from = 0;
        m.fall_drift = 65;
        m.fall_landing_lag = 36;
        m
    };
    w.moves[MoveId::Ext0 as usize] = ending(
        34,
        &[
            r(3, 4, 6, 8, 8, 50, 361, 60, 148, 0, 2),
            r(3, 4, 4, 8, 10, 50, 361, 60, 148, 1, 2),
            r(3, 4, 2, 9, 6, 50, 361, 60, 148, 2, 2),
        ],
    );
    w.moves[MoveId::Ext1 as usize] = ending(
        39,
        &[
            r(4, 5, 6, 9, 8, 60, 361, 60, 136, 0, 2),
            r(4, 5, 1, 9, 8, 60, 361, 60, 136, 1, 2),
        ],
    );
    // The bounce off the ground: 22 frames, then the special fall [E: the reference's page gives the bounce's length, not what
    // follows it].
    let mut bounce = ref_move(23, 36, 0, 255, &[]);
    bounce.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/fire_bounce.script"),
    ));
    bounce.helpless_after = true;
    bounce.fall_drift = 65;
    bounce.fall_landing_lag = 36;
    w.moves[MoveId::Ext2 as usize] = bounce;
}

/// Short-reach, quick and heavy hitting: the longsword's moves pulled in close and sped up, with the
/// forward tilt, neutral air, forward air and blaster built from reference frame data (a blaster-wielding
/// brawler archetype).
pub fn claws() -> Weapon {
    let mut w = sword_kit();
    // Claws, fists and feet: the limb throwing a hit can be hit.
    w.limbs = true;
    for (i, m) in w.moves.iter_mut().enumerate() {
        if MoveId::from_index(i as u8) == MoveId::UpSpecial {
            *m = Move::empty();
            continue;
        }
        // Nothing of the sword's follow-up slots belongs to the claws.
        if MoveId::from_index(i as u8).is_ext() {
            *m = Move::empty();
            continue;
        }
        m.script = None;
        m.projectile_script = None;
        let quick = |f: u8| ((u16::from(f) * 85 + 50) / 100) as u8;
        m.total_frames = quick(m.total_frames).max(if m.total_frames == 0 { 0 } else { 6 });
        m.landing_lag = quick(m.landing_lag);
        m.autocancel_before = quick(m.autocancel_before);
        if m.autocancel_after != 255 {
            m.autocancel_after = quick(m.autocancel_after);
        }
        for hb in &mut m.hitboxes {
            hb.start = quick(hb.start);
            hb.end = quick(hb.end).max(hb.start);
            hb.x = hb.x * Fx::from_ratio(3, 5);
            hb.radius = hb.radius * Fx::from_ratio(11, 10);
            hb.damage = hb.damage * Fx::from_ratio(11, 10);
            hb.knockback_growth += 5;
            hb.priority = 0; // no tip and hilt: claws hit the same everywhere
        }
    }

    // Forward tilt: two hits. Hit 1 on frame 8 (5%, angle 60), hit 2 on frames 9-10 (6%, angle 361). FAF 35.
    w.moves[MoveId::FTilt as usize] = ref_move(
        35,
        0,
        0,
        255,
        &[
            r(8, 8, 19, 10, 8, 50, 60, 10, 70, 0, 0),
            r(9, 10, 21, 10, 9, 60, 361, 55, 106, 0, 1),
        ],
    );
    // Up tilt: an overhead kick on frames 7-11, angle 80, base knockback 30, growth 115-120, FAF 36. The foot
    // (10%) is only active on frames 7-8; the 8%, 9% and 10% hitboxes along the leg last through frame 11.
    w.moves[MoveId::UTilt as usize] = ref_move(
        36,
        0,
        0,
        255,
        &[
            r(7, 8, 9, 28, 9, 100, 80, 30, 115, 0, 0),
            r(7, 11, 8, 24, 9, 80, 80, 30, 115, 1, 0),
            r(7, 11, 5, 16, 9, 90, 80, 30, 115, 1, 0),
            r(7, 11, 6, 20, 11, 100, 80, 30, 120, 0, 0),
        ],
    );
    // Down tilt: a low kick on frames 5-6, 6%, angle 361, base knockback 25, growth 100, FAF 28.
    w.moves[MoveId::DTilt as usize] = ref_move(
        28,
        0,
        0,
        255,
        &[
            r(5, 6, 17, 4, 9, 60, 361, 25, 100, 0, 0),
            r(5, 6, 11, 5, 9, 60, 361, 25, 100, 0, 0),
            r(5, 6, 6, 4, 9, 60, 361, 25, 100, 0, 0),
        ],
    );
    // Neutral air (the reference's): a split kick held out, both legs and the hips hitting. Clean on frames 7-9: 12%, angle 361, BKB
    // 30, KBG 75; late on frames 10-26: 8%, BKB 0, KBG 100 (a lingering hit). Landing lag 9, autocancels on frames 1-6 and from 39, FAF 43.
    w.moves[MoveId::NAir as usize] = ref_move(
        43,
        9,
        6,
        39,
        &[
            // The front leg (kicked out ahead), the hips, the back leg (kicked out behind).
            r(7, 9, 8, 8, 7, 120, 361, 30, 75, 0, 0),
            r(7, 9, 0, 9, 7, 120, 361, 30, 75, 1, 0),
            r(7, 9, -7, 8, 6, 120, 361, 30, 75, 2, 0),
            r(10, 26, 8, 8, 7, 80, 361, 0, 100, 0, 0),
            r(10, 26, 0, 9, 7, 80, 361, 0, 100, 1, 0),
            r(10, 26, -7, 8, 6, 80, 361, 0, 100, 2, 0),
        ],
    );
    // Forward air: frames 7-9, 9%, angle 60, landing lag 10, autocancels from 29, FAF 41.
    w.moves[MoveId::FAir as usize] =
        ref_move(41, 10, 0, 29, &[r(7, 9, 23, 10, 9, 90, 60, 45, 85, 0, 0)]);

    // Blaster: a bayonet hit on frames 15-19 (7%, angle 60, strong knockback) if something is in front of the
    // muzzle, otherwise a shot fires on frame 16. The shot does 8% falling to 6% over its range (about two
    // thirds of the stage). The sources do not give its speed or exact range, so those are estimates.
    let su = |n: i32| Fx::from_ratio(n, 8000);
    let mut blaster = ref_move(53, 0, 0, 255, &[r(15, 19, 21, 10, 9, 70, 60, 80, 37, 0, 0)]);
    blaster.projectile = Some(ProjectileSpawn {
        frame: 15,
        x: Fx::from_ratio(20, 10),
        y: Fx::from_ratio(12, 10),
        speed: su(3000),
        life: 35,
        hitbox: Hitbox {
            start: 0,
            end: 0,
            x: Fx::ZERO,
            y: Fx::ZERO,
            radius: Fx::from_ratio(5, 10),
            damage: Fx::from_int(8),
            angle: 361,
            base_knockback: 20,
            knockback_growth: 0,
            priority: 0,
            group: 0,
            kind: HIT_NORMAL,
            shield_damage: 100,
            hitlag: 100,
            effect: EFFECT_NORMAL,
            targets: TARGETS_ALL,
            fixed_kb: 0,
            hitstun_add: 0,
            sdi: 100,
            link_ahead: Fx::ZERO,
            aim: AIM_ANY,
            set_weight: false,
            reverse: false,
            push: false,
        },
        end_damage: Fx::from_int(6),
        arc: ProjectileArc::NONE,
        phases: [None; 2],
    });
    w.moves[MoveId::NSpecial as usize] = blaster;

    // ---- The rest of the brawler kit (frame data from the reference tables; positions are estimates) ----

    // Jab combo: three quick claw hits. Hit 1 and 2: frame 4, 2%, angle 361, FAF 22; hit 3: frame 4, 4%, angle 55,
    // growth 176, FAF 35. Pressing attack in the last 12 frames of a hit continues the combo.
    let mut jab1 = ref_move(
        22,
        0,
        0,
        255,
        &[
            r(4, 5, 11, 11, 9, 20, 361, 20, 15, 1, 0),
            r(4, 5, 19, 11, 9, 20, 361, 25, 25, 0, 0),
        ],
    );
    jab1.next = Some(MoveId::Jab2 as u8);
    jab1.next_window = 12;
    w.moves[MoveId::Jab as usize] = jab1;
    let mut jab2 = ref_move(
        22,
        0,
        0,
        255,
        &[
            r(4, 5, 11, 11, 9, 20, 361, 20, 20, 1, 0),
            r(4, 5, 19, 11, 9, 20, 361, 25, 25, 0, 0),
        ],
    );
    jab2.next = Some(MoveId::Jab3 as u8);
    jab2.next_window = 12;
    w.moves[MoveId::Jab2 as usize] = jab2;
    w.moves[MoveId::Jab3 as usize] = ref_move(
        35,
        0,
        0,
        255,
        &[
            r(4, 5, 12, 11, 10, 40, 55, 40, 176, 1, 0),
            r(4, 5, 21, 11, 10, 40, 55, 40, 176, 0, 0),
        ],
    );

    // Dash attack: a flying kick. Frames 11-14: 11% (hip angle 80, knee angles 50 and 361); frames 15-18: 8% with
    // lower growth. FAF 38.
    w.moves[MoveId::DashAttack as usize] = ref_move(
        38,
        0,
        0,
        255,
        &[
            r(11, 14, 8, 10, 8, 110, 80, 40, 92, 1, 0),
            r(11, 14, 14, 6, 10, 110, 50, 40, 91, 1, 0),
            r(11, 14, 20, 5, 11, 110, 361, 45, 85, 0, 0),
            r(15, 18, 8, 10, 8, 80, 80, 40, 60, 1, 0),
            r(15, 18, 14, 6, 10, 80, 50, 40, 60, 1, 0),
            r(15, 18, 20, 5, 11, 80, 361, 40, 60, 0, 0),
        ],
    );

    // Up air: frames 7-9, 12%, angle 80, base knockback 30, growth 85. Landing lag 10, autocancels on 1-3 and from 31,
    // FAF 39.
    w.moves[MoveId::UAir as usize] = ref_move(
        39,
        10,
        3,
        31,
        &[
            r(7, 9, 4, 26, 10, 120, 80, 30, 85, 0, 0),
            r(7, 9, 3, 21, 9, 120, 80, 30, 85, 1, 0),
            r(7, 9, 1, 16, 9, 120, 80, 30, 85, 1, 0),
        ],
    );
    // Back air: a back kick, frames 13-15, 15% / 13% / 11% from the foot in, angle 361, base knockback 37,
    // growth 96. Landing lag 15, autocancels on 1-7 and from 19, FAF 45. The brawler keeps facing forward.
    w.moves[MoveId::BAir as usize] = ref_move(
        45,
        15,
        7,
        19,
        &[
            r(13, 15, -20, 9, 11, 150, 361, 37, 96, 0, 0),
            r(13, 15, -13, 10, 10, 130, 361, 37, 96, 1, 0),
            r(13, 15, -6, 10, 9, 110, 361, 37, 96, 2, 0),
        ],
    );
    // Down air: a stomp, frames 16-17, 15% at the foot (a spike, angle 270) and 13% around it, base knockback 6,
    // growth 90. Landing lag 19, autocancels on 1-4 and from 36, FAF 54.
    w.moves[MoveId::DAir as usize] = ref_move(
        54,
        19,
        4,
        36,
        &[
            r(16, 17, 4, 2, 11, 150, 270, 6, 90, 0, 0),
            r(16, 17, 5, 7, 13, 130, 270, 6, 90, 1, 0),
        ],
    );

    // Forward smash: frames 20-23, 15%, angle 361, base knockback 30, growth 106, FAF 42. Charges on frame 6.
    let mut fsmash = ref_move(
        42,
        0,
        0,
        255,
        &[
            r(20, 23, 22, 11, 11, 150, 361, 30, 106, 0, 0),
            r(20, 23, 15, 11, 10, 150, 361, 30, 106, 1, 0),
        ],
    );
    fsmash.charge_at = Some(5);
    w.moves[MoveId::FSmash as usize] = fsmash;
    // Up smash: two hits. Frames 13-15: 6% pulling hits (angles 110 and 125, base knockback 70-80, growth 15);
    // frames 20-23: 12% (angle 95, base knockback 85, growth 65). FAF 48. Charges on frame 3.
    let mut usmash = ref_move(
        48,
        0,
        0,
        255,
        &[
            r(13, 15, 8, 18, 10, 60, 110, 70, 15, 1, 0),
            r(13, 15, 4, 22, 9, 60, 125, 80, 15, 1, 0),
            r(13, 15, 0, 12, 9, 60, 125, 80, 15, 1, 0),
            r(20, 23, 5, 27, 11, 120, 95, 85, 65, 0, 1),
            r(20, 23, 1, 24, 10, 120, 95, 85, 65, 1, 1),
        ],
    );
    usmash.charge_at = Some(2);
    w.moves[MoveId::USmash as usize] = usmash;
    // Down smash: a front hit on frames 14-15 (16% at the claw, 14% closer in, angle 30-35) and a back hit on frames
    // 21-22 (14% / 12%), FAF 44. Charges on frame 2.
    let mut dsmash = ref_move(
        44,
        0,
        0,
        255,
        &[
            r(14, 15, 12, 8, 9, 140, 35, 50, 80, 1, 0),
            r(14, 15, 20, 6, 11, 160, 30, 37, 93, 0, 0),
            r(21, 22, -12, 8, 9, 120, 35, 50, 80, 1, 1),
            r(21, 22, -20, 6, 11, 140, 30, 50, 90, 0, 1),
        ],
    );
    dsmash.charge_at = Some(1);
    w.moves[MoveId::DSmash as usize] = dsmash;

    // Flash dash (side special): after a 19 frame wind-up a dash of about 7 world units (3% on the way), ending in a
    // 20% spike with a 15% hit around it. Helpless in the air. The stick at the start of the dash angles it up or
    // down (a script). The distance, the ending hit knockback and the total length are estimates.
    let mut flash = ref_move(
        55,
        0,
        0,
        255,
        &[
            r(19, 28, 12, 11, 11, 30, 361, 20, 0, 0, 0),
            r(29, 32, 14, 3, 11, 200, 270, 70, 90, 0, 1),
            r(29, 32, 15, 9, 13, 150, 290, 60, 90, 1, 1),
        ],
    );
    flash.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/flash_aim.script"),
    ));
    flash.helpless_after = true;
    w.moves[MoveId::SideSpecial as usize] = flash;

    claws_up_special(&mut w);

    // Reflector (down special): a reflecting field in front of the brawler from frame 9 to 21 that turns projectiles
    // around (1.5 times the damage), plus a 4% hit on anyone touching it. The reflector frames, size and
    // reflected speed are estimates. The brawler is intangible on frames 5-8 (a script).
    let mut reflector = ref_move(31, 0, 0, 255, &[r(9, 12, 14, 11, 15, 40, 65, 60, 85, 0, 0)]);
    reflector.reflector = Some(Reflector {
        start: 8,
        end: 20,
        x: Fx::from_ratio(14, 10),
        y: Fx::from_ratio(11, 10),
        radius: Fx::from_ratio(17, 10),
        damage_percent: 150,
        speed_percent: 130,
        reverse_vertical: false,
        life_percent: 100,
        max_damage: Fx::ZERO,
        detect_start: 0,
        detect_end: 255,
        detect_x: Fx::ZERO,
        detect_y: Fx::ZERO,
        detect_radius: Fx::ZERO,
    });
    reflector.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/reflector_guard.script"),
    ));
    w.moves[MoveId::DownSpecial as usize] = reflector;

    // Grabs and throws. Standing grab: frames 6-7, FAF 37; dash grab 8-9, FAF 45; pivot grab 9-10, FAF 40; pummel 1.3% on
    // frame 2 (the reference says 1; a hit cannot start on a move's first frame), FAF 20; throws release on frame 11
    // (forward, 9%), 24 (back, 11%), 27 (up, 7%) and 26 (down, 8.5%) with first actionable frames 33, 48, 46 and 41. Throw
    // angles and the grab reach are estimates; the sources disagree on the back throw's damage (8% or 11%).
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(37, 0, 0, 255, &[r(6, 7, 17, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(40, 0, 0, 255, &[r(9, 10, 17, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(45, 0, 0, 255, &[r(8, 9, 19, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::Pummel as usize] = reference_pummel(13, 13, 11, 10);
    // The throws are the reference game's two parts: a hit that damages the held fighter without launching it, then the throw,
    // whose own (smaller) damage is all the launch is worked out from. So forward, up and down throws lead into combos at low
    // percents and never kill; the back throw is the kill throw (base 40, growth 150).
    w.moves[MoveId::FThrow as usize] =
        throw_after_hit(33, (10, 50), r(11, 11, 13, 11, 10, 40, 45, 55, 57, 0, 1));
    w.moves[MoveId::BThrow as usize] =
        throw_after_hit(48, (23, 60), r(24, 24, -13, 11, 10, 50, 50, 40, 150, 0, 1));
    w.moves[MoveId::UThrow as usize] =
        throw_after_hit(46, (26, 50), r(27, 27, 13, 11, 10, 20, 80, 75, 110, 0, 1));
    // Ledge attack: 9%, angle 45, base knockback 90, growth 20, hits on frames 24-26, intangible through frame 26,
    // FAF 56 (reference numbers). Positions are estimates.
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &[
            r(24, 26, 21, 6, 10, 90, 45, 90, 20, 0, 0),
            r(24, 26, 12, 6, 10, 90, 45, 90, 20, 0, 0),
        ],
    );
    ledge.intangible = 26;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack from the floor (reference numbers): as the swordfighter's, but the second hit on frames 21-22 and intangible
    // through frame 22.
    let mut getup_attack = ref_move(
        46,
        0,
        0,
        255,
        &[
            r(16, 17, 18, 5, 9, 70, 361, 80, 48, 0, 0),
            r(21, 22, -18, 5, 9, 70, 361, 80, 48, 0, 1),
        ],
    );
    getup_attack.intangible = 22;
    w.moves[MoveId::GetUpAttack as usize] = getup_attack;
    w.moves[MoveId::DThrow as usize] = with_kind(
        ref_move(
            41,
            0,
            0,
            255,
            &[r(26, 26, 13, 11, 10, 85, 361, 50, 65, 0, 0)],
        ),
        HIT_THROW,
    );
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_weapon_has_every_move_slot() {
        for w in [longsword(), claws()] {
            assert_eq!(w.moves.len(), MoveId::COUNT);
            for (i, m) in w.moves.iter().enumerate() {
                if m.is_empty() {
                    assert!(
                        MoveId::from_index(i as u8).may_be_empty(),
                        "only specials and jab chain slots may be empty"
                    );
                    continue;
                }
                assert!(
                    !m.hitboxes.is_empty()
                        || m.projectile.is_some()
                        || m.counter.is_some()
                        || m.script.is_some()
                );
                for hb in &m.hitboxes {
                    assert!(
                        hb.start <= hb.end && hb.end <= m.total_frames,
                        "{}",
                        MoveId::from_index(i as u8).name()
                    );
                    assert!((hb.damage > Fx::ZERO || hb.kind == HIT_GRAB) && hb.radius > Fx::ZERO);
                }
            }
        }
    }

    #[test]
    fn the_tip_hits_harder_than_the_body_for_the_sword() {
        let w = longsword();
        for id in [MoveId::FTilt, MoveId::FSmash, MoveId::FAir, MoveId::BAir] {
            let m = &w.moves[id as usize];
            // (The tip is the last in priority, as in the reference: the blade's sourspot wins where both reach.)
            let tip = m.hitboxes.iter().find(|h| h.priority == 2).unwrap();
            let body = m.hitboxes.iter().find(|h| h.priority == 0).unwrap();
            assert!(tip.damage > body.damage, "{}", id.name());
            assert!(
                tip.x.abs() > body.x.abs(),
                "the tip is further out than the body"
            );
        }
    }

    #[test]
    fn move_ids_round_trip() {
        for i in 0..MoveId::COUNT as u8 {
            assert_eq!(MoveId::from_index(i) as u8, i);
        }
    }

    #[test]
    fn aerials_and_specials_are_classified() {
        for i in 0..MoveId::COUNT as u8 {
            let id = MoveId::from_index(i);
            assert_eq!(id.is_aerial(), (8..13).contains(&i));
            assert_eq!(id.is_special(), (13..=16).contains(&i));
        }
    }

    #[test]
    fn reference_rows_convert_to_move_frames() {
        // The reference numbers the first frame 1; move frames count from 0.
        let fair = &longsword().moves[MoveId::FAir as usize];
        // The tipper's first row is frame 6 alone (the swing moves it each frame).
        assert_eq!(fair.hitboxes[0].start, 5);
        assert_eq!(fair.hitboxes[0].end, 5);
        assert_eq!(fair.total_frames, 36);
        assert_eq!(fair.autocancel_after, 35);
    }
}
