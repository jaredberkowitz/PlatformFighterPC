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
    /// Free slots for follow-up moves (ten here and six more after the pivot grab) that only a script (`goto`) or a counter reaches: the later hits of a
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
    /// Six more follow-up slots (sixteen in all), after the pivot grab so that every older slot keeps its index.
    Ext10,
    Ext11,
    Ext12,
    Ext13,
    Ext14,
    Ext15,
}

impl MoveId {
    pub const COUNT: usize = 45;
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
        let i = self as u8;
        (i >= 28 && i <= 37) || (i >= 39 && i <= 44)
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
            39 => MoveId::Ext10,
            40 => MoveId::Ext11,
            41 => MoveId::Ext12,
            42 => MoveId::Ext13,
            43 => MoveId::Ext14,
            44 => MoveId::Ext15,
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
            MoveId::Ext10 => "ext10",
            MoveId::Ext11 => "ext11",
            MoveId::Ext12 => "ext12",
            MoveId::Ext13 => "ext13",
            MoveId::Ext14 => "ext14",
            MoveId::Ext15 => "ext15",
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
            MoveId::Ext10 => "extra move 10",
            MoveId::Ext11 => "extra move 11",
            MoveId::Ext12 => "extra move 12",
            MoveId::Ext13 => "extra move 13",
            MoveId::Ext14 => "extra move 14",
            MoveId::Ext15 => "extra move 15",
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
    /// What the hit is made of: [`EFFECT_NORMAL`], [`EFFECT_ELECTRIC`] (more hitlag, and the victim drifts with the held stick) or
    /// [`EFFECT_BURY`] (a grounded victim is buried).
    pub effect: u8,
    /// Who it can hit: [`TARGETS_ALL`], or only fighters on the ground ([`TARGETS_GROUNDED`]) or in the air ([`TARGETS_AERIAL`]).
    pub targets: u8,
    /// Set knockback (the reference's "fixed knockback value"): when above 0 the knockback is worked out as if the victim's percent
    /// were this, whatever it really is, so the hit sends the same distance at any percent. 0 for normal knockback.
    pub fixed_kb: u16,
    /// Hitstun frames this hit adds, or takes away when negative (the reference's hitstun modifier). A launched victim keeps at
    /// least one frame of hitstun.
    pub hitstun_add: i16,
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
    /// Percent of the normal shield stun this hit causes on a shield (the reference's per-hitbox shieldstun multiplier; 100 is
    /// normal). It scales the damage part of the stun, before the cap.
    pub shieldstun: u16,
    /// Which way "forward" (0 degrees) is for this hit's launch (the reference's angle type): [`ANGLE_BY_BOX`] (the default: a box
    /// behind the attacker sends backward, any other forward), [`ANGLE_FORWARD`] (always the way the attacker faces),
    /// [`ANGLE_BACKWARD`] (always the other way), [`ANGLE_BY_POSITION`] (away from the attacker, from the side the victim is on) or
    /// [`ANGLE_BY_TRAVEL`] (the way the attacker is moving sideways, its facing when it is not: the reference's speed-based type).
    /// For a projectile the attacker is the projectile: its direction of travel, and its own position.
    pub angle_type: u8,
    /// A command grab (on a [`HIT_GRAB`] box): it catches fighters in the air too, shielding or in hitstun (`grab::catchable`), and
    /// the holder goes on with its move's `Move::catch_move` instead of the ordinary hold.
    pub command: bool,
    /// The hit can clank with other attacks and projectiles (the reference's `clang`). False: it never clanks (the reference's
    /// no-clank hits, and a projectile's transcendent priority): it trades instead.
    pub clanks: bool,
    /// When this hit clanks, its owner rebounds out of the move as usual (the reference's `rebound`). False: the hit is cancelled by
    /// the clank but its owner carries on with the move (the reference's "cannot rebound": `rebound=f` without `clang=f`).
    pub rebounds: bool,
    /// Percent chance (0 to 100) that the hit trips a fighter on the ground instead of launching it, when its launch would not lift
    /// it off the ground (the reference's `trip`). Rolled on the game's own random generator, so it is the same on every machine.
    pub trip: u8,
    /// Half the height of the box: above 0 it is a standing capsule from `y - half_height` to `y + half_height` (a tall wall of a
    /// projectile, the reference's stacked circles) instead of a circle. Scaled like the radius.
    pub half_height: Fx,
    /// A position vector (E34, the reference's): with `posvec_frames` above 0, the hit sends its victim's feet to the point
    /// `posvec_x` ahead of (the attacker's facing) and `posvec_y` above the attacker's feet where it hit, straight there at the speed
    /// that covers the distance in `posvec_frames` frames, with no gravity of its own on the way; there it stops and its hitstun
    /// runs out as a fall (see `fighter::posvec_launch`). The angle and knockback still give the hitstun. Scaled like the position.
    /// 0 frames: an ordinary launch. (On a projectile the attacker is its owner.)
    pub posvec_x: Fx,
    pub posvec_y: Fx,
    pub posvec_frames: u8,
    /// Half the length of a lying capsule (E38, sim 102: the reference's horizontal capsule, a bolt): above 0 the box reaches this
    /// far ahead of and behind its centre as well (with `half_height`, a rounded rectangle). Scaled by the body's `hitbox_scale`
    /// but not by a projectile's own size (the reference scales a shot's radius, not its capsule's ends). 0: as before.
    pub half_width: Fx,
    /// Which version of the reference's hit rules this hit follows (E40, sim 113, owner decisions 66-67): [`RULES_CURRENT`] (the
    /// default: 13.0.1's, as every other hit) or [`RULES_PREVIOUS`] (the previous reference version's, 1.1.x: no one-on-one damage
    /// multiplier, its hitlag formula with a crouch shortening only the victim's, its rage, one more hitstun frame on a tumbling
    /// launch, and its launch physics: no launch speed-up, the gravity boost on every tumbling launch and no shared fall speed for a
    /// vertical one). The numbers are the ruleset's `prev_*`. Only moves the owner names use it (the Settler's jab).
    pub rules: u8,
    /// The reference's slip effect (E44, sim 117): a fighter on the ground (not already lying or sitting) that this hit reaches always
    /// trips, whatever the hit's knockback and angle, instead of being launched: [`SLIP_HARD`] (the hit's damage and hitlag stand,
    /// then the ruleset's trip timings: a thrown banana peel) or [`SLIP_SOFT`] (no hitlag, the ruleset's shorter `slip_*` timings:
    /// walking into a lying peel). An airborne fighter takes the hit as usual. [`SLIP_NONE`] (the default): no slip.
    pub slip: u8,
}

/// `Hitbox::slip` values (E44, sim 117).
pub const SLIP_NONE: u8 = 0;
pub const SLIP_HARD: u8 = 1;
pub const SLIP_SOFT: u8 = 2;

/// `Hitbox::rules` values (E40, sim 113).
pub const RULES_CURRENT: u8 = 0;
pub const RULES_PREVIOUS: u8 = 1;

/// `Hitbox::angle_type` values.
pub const ANGLE_BY_BOX: u8 = 0;
pub const ANGLE_FORWARD: u8 = 1;
pub const ANGLE_BACKWARD: u8 = 2;
pub const ANGLE_BY_POSITION: u8 = 3;
/// The reference's type 1 (sim 73, owner decision 23): 0 degrees is the way the attacker is moving.
pub const ANGLE_BY_TRAVEL: u8 = 4;

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

/// The vortex angle (the reference's 366, E28): the victim is sent the way the attacker was moving when it hit, at the knockback's
/// own launch speed (no cap, no cut, grounded victims too), and for its first [`crate::state::VORTEX_FRAMES`] frames it is also
/// drawn to the hitbox's centre (see `fighter::vortex_pull`). An attacker standing still (a burst) gives the pull alone.
pub const ANGLE_VORTEX: i16 = 366;

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
        self.shieldstun = 100;
        self.angle_type = ANGLE_BY_BOX;
        self.command = false;
        self.clanks = true;
        self.rebounds = true;
        self.trip = 0;
        self.half_height = Fx::ZERO;
        self.posvec_x = Fx::ZERO;
        self.posvec_y = Fx::ZERO;
        self.posvec_frames = 0;
        self.half_width = Fx::ZERO;
        self.rules = RULES_CURRENT;
        self.slip = SLIP_NONE;
        self
    }
}

/// `Hitbox::effect` values.
pub const EFFECT_NORMAL: u8 = 0;
pub const EFFECT_ELECTRIC: u8 = 1;
/// The reference's bury effect (E5, sim 90): a fighter on the ground (not lying down) is buried instead of launched
/// (`FighterState::Buried`, see `combat::bury_frames`); an airborne one takes the hit as usual. Its knockback is worked out at weight
/// 100 and it ignores subtractive knockback armor.
pub const EFFECT_BURY: u8 = 2;
/// The reference's egg (E8, sim 107): on a throw row (`HIT_THROW`) of a move with an [`Encase`] block, the released fighter is shut in
/// an egg (`FighterState::Encased`) instead of launched; see [`Encase`].
pub const EFFECT_ENCASE: u8 = 3;

/// `Hitbox::kind` values.
pub const HIT_NORMAL: u8 = 0;
/// Catches a grounded fighter instead of damaging it. Shields do not stop it.
pub const HIT_GRAB: u8 = 1;
/// Releases the fighter being held with this hit's damage and launch. It needs no overlap.
pub const HIT_THROW: u8 = 2;
/// Damages the fighter being held without releasing it.
pub const HIT_PUMMEL: u8 = 3;
/// A search box (the reference's "search" collision): touching a fighter's hurtbox does nothing to it (no damage, hitlag or clank;
/// shields do not matter), but switches the attacker's move to its `Move::on_search` on the next frame, keeping its speed.
pub const HIT_SEARCH: u8 = 4;

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
    /// Later hits by age (the reference's clean, late and latest hits of one projectile, or a growing explosion): from age `.0` on,
    /// `.1` replaces the hit (with no damage falloff); a hit of radius 0 is none (a gap). Up to [`MAX_PHASES`]; `None` for none.
    pub phases: [Option<(u8, Hitbox)>; MAX_PHASES],
    /// It is not used up by a hit: it goes on (through every fighter it touches) to the end of its life.
    pub pierce: bool,
    /// A fighter it already hit with a hit group can be hit by that group again this many frames after its last hit on that fighter
    /// (the reference's per-target rehit; `Projectile::hit_mask` and `hit_age` remember); 0 never.
    pub rehit: u8,
    /// It bursts where it hits (E3e, the reference's powered wave): the first fighter it touches (shielding or not) takes its own hit,
    /// then it stops there and becomes the burst (`Projectile::bursting`). `None`: it is used up or pierces as above.
    pub burst: Option<Burst>,
    /// A charge-scaled shot (E35, sim 102): its damage, speed, life and size go with the firing fighter's charge (see [`ChargedShot`]).
    /// `None`: the values above, whatever the charge.
    pub charged: Option<ChargedShot>,
    /// A thrown tool's life rules (E3d, sim 102): how many may be out at once, the owner's catch, falling after a clank. See [`Tool`];
    /// [`Tool::NONE`] for none of them.
    pub tool: Tool,
}

/// A charge-scaled shot (E35, sim 102: the reference's charged arrows and sword wave). With `c` the firing fighter's `charge` as the shot
/// appears (a smash attack's charge, or a script's `set_charge` carried to the firing move by `Move::keep_charge`), at most `full`, and
/// `full` at once from `snap` charge frames on (0: no snap): the shot's damage goes from the hit's damage (`ProjectileSpawn::hitbox`)
/// to `damage_to` (rounded down to a whole percent with `floor`), its speed from `ProjectileSpawn::speed` to `speed_to` (a script's
/// `spawn` speed is scaled by the same ratio: the script fires as at no charge), its life from `ProjectileSpawn::life` to `life_to`
/// and its size from `size_from` to `size_to` percent (the radius; a lying capsule's ends do not grow), each in proportion to
/// `c / full`. The damage set at the start has no falloff over the flight; later hits by age (`phases`) keep their own damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChargedShot {
    pub full: u8,
    pub snap: u8,
    pub damage_to: Fx,
    pub speed_to: Fx,
    pub life_to: u8,
    pub size_from: u16,
    pub size_to: u16,
    pub floor: bool,
    /// Sim 111: percent of that damage the shot deals (100: all), applied after the floor and rounded down again with `floor` (the
    /// reference's double arrow: each arrow deals floor(0.8 x the single arrow's damage)).
    pub times: u8,
    /// Sim 113: every hit of the shot, its later hits by age too, deals from 100 percent at no charge to `power_to` percent at `full`
    /// (in proportion: the reference's smash-attack charge carried by a shot a smash attack makes, such as the settler's bowling
    /// ball, 1.2x at full). 100 (the default): no change.
    pub power_to: u16,
}

impl ChargedShot {
    /// The charge the shot is made with, out of `full`: `charge` capped at `full`, and `full` from `snap` on.
    pub fn level(&self, charge: u8) -> u8 {
        if self.snap > 0 && charge >= self.snap {
            self.full
        } else {
            charge.min(self.full)
        }
    }

    /// `from` to `to` in proportion to `level / full` [D: linear].
    pub fn lerp(&self, from: Fx, to: Fx, level: u8) -> Fx {
        if self.full == 0 {
            return to;
        }
        from + (to - from) * Fx::from_ratio(i32::from(level), i32::from(self.full))
    }
}

/// A thrown tool's life rules (E3d, sim 102: the reference's boomerang). All off in [`Tool::NONE`].
///
/// * `kind` (0: none) groups the moves that fire the same tool, and `limit` (0: no limit) is how many of them one fighter may have out
///   at once: a spawn past the limit does not happen, and the `owned` register counts them so the move can branch (a failed throw).
///   With kind 0 the move's own shots are counted.
/// * From age `catch_from` on (255: never), a projectile still its firer's (not reflected) that touches its owner's body is caught:
///   it is removed, and if the owner is free (standing, walking, running, dashing, turning, crouching or airborne: not attacking,
///   defending or hurt) it starts `catch_move` (an `ext` slot). An owner dodging (a roll, spot dodge or air dodge) lets it fly on.
/// * `on_clank`: a clank with an attack does not destroy it, it drops (see [`Fall`]).
/// * `pocket` (E42, sim 115): whether a fighter's pocket can put it away ([`POCKET_YES`], the default: the reference's rule that a
///   projectile that can be reflected can be pocketed), never ([`POCKET_NEVER`]: one that is not a thing of its own, such as a line its
///   thrower holds or a standing explosion), or yes and its maker may make another at once ([`POCKET_FREES`]: the reference's rocket;
///   any other pocketed tool still counts as out for its maker, the reference's "still in play").
/// * `water` (E43, sim 115): each frame it touches a placed piece that grows with water (`Placed::water`), it gives it this much (0:
///   none; the reference's watering can).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tool {
    pub kind: u8,
    pub limit: u8,
    pub catch_from: u8,
    pub catch_move: Option<u8>,
    pub on_clank: Option<Fall>,
    pub pocket: u8,
    pub water: u8,
}

/// `Tool::pocket` values (E42).
pub const POCKET_YES: u8 = 0;
pub const POCKET_NEVER: u8 = 1;
pub const POCKET_FREES: u8 = 2;

impl Tool {
    pub const NONE: Tool = Tool {
        kind: 0,
        limit: 0,
        catch_from: 255,
        catch_move: None,
        on_clank: None,
        pocket: POCKET_YES,
        water: 0,
    };

    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.kind);
        h.write_u8(self.limit);
        h.write_u8(self.catch_from);
        h.write_u8(self.pocket);
        h.write_u8(self.water);
        match self.catch_move {
            Some(id) => {
                h.write_bool(true);
                h.write_u8(id);
            }
            None => h.write_bool(false),
        }
        match &self.on_clank {
            Some(f) => {
                h.write_bool(true);
                h.write_u8(f.life);
                f.vx.hash_into(h);
                f.vy.hash_into(h);
                f.gravity.hash_into(h);
                f.max_fall.hash_into(h);
            }
            None => h.write_bool(false),
        }
    }
}

/// A projectile knocked down by a clank (`Tool::on_clank`): it stops hitting, its age starts again and it lasts `life` frames, moving
/// `vx` the way it was going (negative: back) and `vy` up, pulled down by `gravity` to at most `max_fall`; it comes to rest on a
/// platform top it falls onto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fall {
    pub life: u8,
    pub vx: Fx,
    pub vy: Fx,
    pub gravity: Fx,
    pub max_fall: Fx,
}

/// What a bursting projectile becomes (`ProjectileSpawn::burst`): it stands still `y` above where it stopped, its age starts again
/// at 0 and it lasts `life` frames, piercing, with `rehit` as its per-target rehit, its hits by age from `phases` (as
/// `ProjectileSpawn::phases`: from age `.0` on, `.1`; radius 0 is a gap; before the first phase there is no hit). It hits the
/// fighter that stopped it again (its hits start afresh). With `on_wall`, touching a solid wall bursts it there too (just short of
/// the wall) instead of removing it. A bursting projectile is not reflected and does not clank.
///
/// Sim 95 (the lobbed shell's burst, all off by default): with `on_ground`, a landing it does not bounce from (its bounces used up, or
/// falling faster than `ProjectileArc::bounce_max_fall`) bursts it where it touched, instead of removing it; with `on_expire`, the end
/// of its life bursts it where it is; with `keep_hits`, the burst keeps the projectile's hits so far, so a fighter the projectile itself
/// hit is not hit again by a burst phase of the same hit group (the reference's single hit).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Burst {
    pub life: u8,
    pub rehit: u8,
    pub on_wall: bool,
    pub y: Fx,
    pub phases: [Option<(u8, Hitbox)>; MAX_PHASES],
    pub on_ground: bool,
    pub on_expire: bool,
    pub keep_hits: bool,
    /// Sim 113: touching a fighter bursts it (the default, as before). False: a fighter it touches takes the projectile's own hit and
    /// the projectile is used up there with no burst (the reference's rocket, which bursts only against a wall or when its flight
    /// runs out); `on_wall`, `on_ground` and `on_expire` still burst it.
    pub on_hit: bool,
}

impl Burst {
    /// The burst's hit at `age` (`None` before its first phase or in a radius-0 gap).
    pub fn hit_at(&self, age: u8) -> Option<Hitbox> {
        let mut out = None;
        for (from, hb) in self.phases.iter().flatten() {
            if age >= *from {
                out = Some(*hb);
            }
        }
        out.filter(|hb| hb.radius > Fx::ZERO)
    }

    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.life);
        h.write_u8(self.rehit);
        h.write_bool(self.on_wall);
        h.write_bool(self.on_ground);
        h.write_bool(self.on_expire);
        h.write_bool(self.keep_hits);
        h.write_bool(self.on_hit);
        self.y.hash_into(h);
        for phase in &self.phases {
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
}

/// The most later hits by age a projectile can have (`ProjectileSpawn::phases`).
pub const MAX_PHASES: usize = 8;

/// A projectile's later hits from a list (the rest `None`).
pub fn phase_list(list: &[(u8, Hitbox)]) -> [Option<(u8, Hitbox)>; MAX_PHASES] {
    let mut out = [None; MAX_PHASES];
    for (slot, p) in out.iter_mut().zip(list) {
        *slot = Some(*p);
    }
    out
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
///
/// Sim 95: `bounce_keep_y` (percent, 100 default) multiplies the bounce's vertical speed after the clamp, and `bounce_max_fall` (0
/// default: no limit) is the fastest fall it still bounces from: a landing faster than that does not bounce (it is removed, or bursts
/// with `Burst::on_ground`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileArc {
    pub angle: i16,
    pub gravity: Fx,
    pub max_fall: Fx,
    pub bounces: u8,
    pub bounce_min: Fx,
    pub bounce_max: Fx,
    pub bounce_keep: u8,
    pub bounce_keep_y: u8,
    pub bounce_max_fall: Fx,
    /// Sim 99: `drag` slows it by this much a frame along its flight, down to a stop (E35, the reference's friction on a shot); `lodge`
    /// (frames, 0: none) makes it stick where it meets a platform top or a solid wall (E3b, the reference's arrows), if it meets it at
    /// `lodge_angle` degrees or more (0: any angle; shallower, it is removed): stuck, it stops moving and hitting and lasts `lodge`
    /// more frames.
    pub drag: Fx,
    pub lodge: u8,
    pub lodge_angle: u8,
    /// Sim 111 (E4): a lodged shot becomes this item of its weapon (an index into `Weapon::items`) instead of a lodged projectile: it
    /// lies stuck where it lodged for `lodge` frames, and any fighter can pick it up (the reference's arrows). `None`: it stays a
    /// lodged projectile.
    pub lodge_item: Option<u8>,
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
        bounce_keep_y: 100,
        bounce_max_fall: Fx::ZERO,
        drag: Fx::ZERO,
        lodge: 0,
        lodge_angle: 0,
        lodge_item: None,
    };
}

/// An item a weapon makes (E4, sim 111; the reference's held items: owner decision 64). Items are not projectiles: they can be held,
/// thrown by anyone (owner decision 54), lie on the stage, be knocked about and burst. The numbers are world units (a reference value
/// over 8) and frames.
///
/// * `kind` (1 to 255) is the number the `holding` and `owned` script registers read for it (shared with `Tool::kind`'s numbers).
/// * Flight: `gravity` down to `max_fall`; `air_friction` slows its sideways speed in the air and `friction` on the ground; its
///   sideways speed is held within `max_speed` (0: no limit). It is a box of `half_width` by `half_height` standing on its `pos`.
/// * Bounces (the reference's): off a floor it keeps `floor_keep` (sideways, upward) percent, off a wall `wall_keep` and gains
///   `wall_lift` upward, off a ceiling `ceiling_keep`; it only bounces when its speed into the surface is above `bounce_min` (else it
///   stops against it, or lies on a floor); after a bounce its sideways speed is held within `bounce_x` and its vertical within
///   `bounce_y` (low, high).
/// * Picking up: a fighter's reach (the ruleset's `item_reach_*`) must overlap its pick-up box (`pick_width`, `pick_height`, half
///   lengths around its middle).
/// * Throwing: thrown at the ruleset's speed for the throw times the thrower's toss strength (`1 + (weight - 100) / 300`, the
///   reference's) times `throw_mul`, the sideways part times `throw_mul` once more (the reference's); its `hit` is live from the
///   throw until it lands, hits a fighter or a shield, or `thrown_life` frames pass (0: no limit). The hit deals `hit.damage +
///   damage_per_speed x the throw's speed` (world units a frame, before `throw_mul`; a grounded drop halves the first part).
///   `vanish` (the reference's arrow): it disappears on a fighter, a shield, a surface or the end of `thrown_life`; otherwise it bounces
///   off a fighter or a shield keeping `fighter_keep` (sideways, back the other way; upward) percent and gaining `fighter_lift`.
/// * Life: it lasts `life` frames from its making (0: until used; held or not), then goes, or bursts with a `blast`.
/// * Knocked about: with `hp` above 0 an attack touching its `hurt_radius` circle (around its middle) takes the attack's damage off
///   its `hp` and launches it along the attack's angle at `knock_speed.0` for `knock_damage.0` percent up to `knock_speed.1` for
///   `knock_damage.1` (in proportion, held within), with `knock_hitlag` percent of the hit's hitlag; at 0 hp it bursts (or goes).
/// * Pocketing (E42, sim 115): a fighter's pocket can put it away when `pocketable` (the default; the reference: any item but a few).
/// * Placed (E43, sim 115): see [`Placed`] (`None`: an ordinary item).
/// * Sim 117 (the reference's banana peel; all off by default): `damage_add` is added to the thrown hit's damage after a drop has
///   halved its base (the reference's "base damage modifier", which a drop does not halve); `uses` (0: no limit) is how many uses
///   the item has: a fighter it hits while thrown, a fighter it trips while lying (`trap`) and a throw by a fighter each spend one,
///   and once they are spent it is gone (at once on a hit or a trip; at the end of its flight after a throw); `trap` makes it trip
///   fighters that walk into it while it lies on a platform (see [`Trap`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemSpec {
    pub pocketable: bool,
    pub placed: Option<Placed>,
    pub damage_add: Fx,
    pub uses: u8,
    pub trap: Option<Trap>,
    pub kind: u8,
    pub gravity: Fx,
    pub max_fall: Fx,
    pub air_friction: Fx,
    pub friction: Fx,
    pub max_speed: Fx,
    pub half_width: Fx,
    pub half_height: Fx,
    pub floor_keep: (u8, u8),
    pub wall_keep: (u8, u8),
    pub wall_lift: Fx,
    pub ceiling_keep: (u8, u8),
    pub bounce_min: Fx,
    pub bounce_x: (Fx, Fx),
    pub bounce_y: (Fx, Fx),
    pub pick_width: Fx,
    pub pick_height: Fx,
    pub throw_mul: Fx,
    pub hit: Hitbox,
    pub damage_per_speed: Fx,
    pub thrown_life: u16,
    pub vanish: bool,
    pub fighter_keep: (u8, u8),
    pub fighter_lift: Fx,
    pub life: u16,
    pub hp: Fx,
    pub hurt_radius: Fx,
    pub knock_speed: (Fx, Fx),
    pub knock_damage: (Fx, Fx),
    pub knock_hitlag: u8,
    pub blast: Option<ItemBlast>,
}

/// A move's throw of the item in hand (E47, sim 117: the reference's banana peel, thrown by its move at its own speed and angle, not the
/// universal item throws'). On move frame `frame` (and on `again`, 255 none: an item already held as the move began is thrown first),
/// if the fighter holds an item it lets go of it: `angle` degrees above forward (more than 90 is behind), at `speed` (`smash_speed`
/// when the special was input like a smash attack, the `flick` register) times the fighter's toss strength, with the item's
/// `throw_mul` as the universal throws apply it. Its thrown hit is a drop's (the base halved, the reference's: the toss "considers
/// the throw as dropping the item"). A throw on `frame` also starts the fighter's `pull_wait` at `wait` frames (0: none), the
/// reference's time before the next peel can be made; the `pull_wait` register reads it. A move's toss is not one of the item's
/// `uses`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Toss {
    pub frame: u8,
    pub again: u8,
    pub angle: i16,
    pub speed: Fx,
    pub smash_speed: Fx,
    pub wait: u8,
}

/// A lying item's trap (E45, sim 117: the reference's banana peel lying on the ground). While the item lies (or slides) on a platform,
/// not held, a fighter on the ground whose lowest hurtbox (the legs) touches `hit` (placed `hit.x` ahead of and `hit.y` above the
/// item's foot, by the item's facing) is tripped by it, unless the fighter is the item's owner (its last thrower), already lying or
/// sitting, in hitstun, held or intangible. A shield does not stop it (the reference's `blockable=no`). The trip is the hit's `slip`
/// ([`SLIP_SOFT`] for the peel: no hitlag, the ruleset's `slip_*` timings) with its damage (none for the peel). The trip spends one of
/// the item's `uses`; if any are left the item is kicked up and away from the fighter it tripped at `kick_x` sideways and `kick_y` up
/// (world units a frame) [E: the reference's peel flies a short way].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trap {
    pub hit: Hitbox,
    pub kick_x: Fx,
    pub kick_y: Fx,
}

/// An item's burst (the reference's bomb explosion): it starts `delay` frames after a `detonate()` (or at once at the end of the
/// item's life or at 0 hp), stands where the item was and hits with `hit` (centred on the item's middle) for `frames` frames, each
/// fighter once, credited to the fighter whose weapon made the item; with `hits_owner` that fighter too (the reference's self-damage).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemBlast {
    pub hit: Hitbox,
    pub frames: u8,
    pub delay: u8,
    pub hits_owner: bool,
}

impl ItemSpec {
    /// An item with this thrown hit and every other value at its default (0, the keeps 100, `throw_mul` 1, `knock_hitlag` 100): what
    /// a content file's `item` block starts from.
    pub fn with_hit(hit: Hitbox) -> ItemSpec {
        ItemSpec {
            pocketable: true,
            placed: None,
            damage_add: Fx::ZERO,
            uses: 0,
            trap: None,
            kind: 0,
            gravity: Fx::ZERO,
            max_fall: Fx::ZERO,
            air_friction: Fx::ZERO,
            friction: Fx::ZERO,
            max_speed: Fx::ZERO,
            half_width: Fx::ZERO,
            half_height: Fx::ZERO,
            floor_keep: (100, 100),
            wall_keep: (100, 100),
            wall_lift: Fx::ZERO,
            ceiling_keep: (100, 100),
            bounce_min: Fx::ZERO,
            bounce_x: (Fx::ZERO, Fx::ZERO),
            bounce_y: (Fx::ZERO, Fx::ZERO),
            pick_width: Fx::ZERO,
            pick_height: Fx::ZERO,
            throw_mul: Fx::ONE,
            hit,
            damage_per_speed: Fx::ZERO,
            thrown_life: 0,
            vanish: false,
            fighter_keep: (100, 100),
            fighter_lift: Fx::ZERO,
            life: 0,
            hp: Fx::ZERO,
            hurt_radius: Fx::ZERO,
            knock_speed: (Fx::ZERO, Fx::ZERO),
            knock_damage: (Fx::ZERO, Fx::ZERO),
            knock_hitlag: 100,
            blast: None,
        }
    }

    /// Every fixed-point value with its content name (in file order).
    pub fn fx_fields_mut(&mut self) -> [(&'static str, &mut Fx); 25] {
        [
            ("gravity", &mut self.gravity),
            ("max_fall", &mut self.max_fall),
            ("air_friction", &mut self.air_friction),
            ("friction", &mut self.friction),
            ("max_speed", &mut self.max_speed),
            ("half_width", &mut self.half_width),
            ("half_height", &mut self.half_height),
            ("wall_lift", &mut self.wall_lift),
            ("bounce_min", &mut self.bounce_min),
            ("bounce_x_min", &mut self.bounce_x.0),
            ("bounce_x_max", &mut self.bounce_x.1),
            ("bounce_y_min", &mut self.bounce_y.0),
            ("bounce_y_max", &mut self.bounce_y.1),
            ("pick_width", &mut self.pick_width),
            ("pick_height", &mut self.pick_height),
            ("throw_mul", &mut self.throw_mul),
            ("damage_per_speed", &mut self.damage_per_speed),
            ("fighter_lift", &mut self.fighter_lift),
            ("hp", &mut self.hp),
            ("hurt_radius", &mut self.hurt_radius),
            ("knock_speed_min", &mut self.knock_speed.0),
            ("knock_speed_max", &mut self.knock_speed.1),
            ("knock_damage_min", &mut self.knock_damage.0),
            ("knock_damage_max", &mut self.knock_damage.1),
            ("damage_add", &mut self.damage_add),
        ]
    }

    /// Every percent (and the knock's hitlag percent, and the number of uses) with its content name (in file order).
    pub fn u8_fields_mut(&mut self) -> [(&'static str, &mut u8); 10] {
        [
            ("floor_keep_x", &mut self.floor_keep.0),
            ("floor_keep_y", &mut self.floor_keep.1),
            ("wall_keep_x", &mut self.wall_keep.0),
            ("wall_keep_y", &mut self.wall_keep.1),
            ("ceiling_keep_x", &mut self.ceiling_keep.0),
            ("ceiling_keep_y", &mut self.ceiling_keep.1),
            ("fighter_keep_x", &mut self.fighter_keep.0),
            ("fighter_keep_y", &mut self.fighter_keep.1),
            ("knock_hitlag", &mut self.knock_hitlag),
            ("uses", &mut self.uses),
        ]
    }
}

/// A placed piece (E43, sim 115: the reference's sapling and tree). An item with `placed` is set down on the ground rather than in the
/// hand: a move's `pull` of it puts it `x` ahead of the maker's feet on the platform it stands on, and pulls up any other placed piece of
/// its kind it lands on and the maker's own older one (one each). It is never picked up, pocketed, moved or knocked about.
///
/// * Hits: with `hp` above 0 an attack or a shot touching its hurtbox (a standing capsule of `hurt_radius` around its middle, reaching
///   `hurt_half_height` above and below it) takes the hit's damage off its `hp` (once per attack), and at 0 it is gone (or bursts). A
///   shot that hits it is used up on it (a piercing one passes). An attacker that hits it is frozen for `hitter_lag` percent of the
///   hit's hitlag (no one-on-one multiplier).
/// * Growing (`water` above 0): a shot with `Tool::water` touching it gives it that much each frame (the shot goes on); once it has
///   `water`, `grow_delay` frames later it becomes item `grow_into` of its maker's weapon where it stands, and the projectile of its
///   maker's move `grow_move` (255: none) is fired there, credited to the last fighter whose water touched it.
/// * Felling (`cuts` above 0): a hit by a move with `Move::chops` cuts it on the side the attacker stands; `cuts` cuts on one side fell
///   it (the other side's do not count): it is gone, the projectile of its maker's move `fall_move` (255: none) is fired from its feet
///   the way away from the chopper, credited to the chopper, and with `chip_chance` per mille (drawn from the game's generator) item
///   `chip` (255: none) of its maker's weapon is made there, credited to the chopper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub x: Fx,
    pub hurt_half_height: Fx,
    pub hitter_lag: u8,
    pub water: u8,
    pub grow_delay: u8,
    pub grow_into: u8,
    pub grow_move: u8,
    pub cuts: u8,
    pub fall_move: u8,
    pub chip: u8,
    pub chip_chance: u16,
}

impl Placed {
    /// A piece that only stands there: nothing grows, nothing fells it.
    pub const PLAIN: Placed = Placed {
        x: Fx::ZERO,
        hurt_half_height: Fx::ZERO,
        hitter_lag: 0,
        water: 0,
        grow_delay: 0,
        grow_into: 0,
        grow_move: 255,
        cuts: 0,
        fall_move: 255,
        chip: 255,
        chip_chance: 0,
    };
}

impl StateHash for ItemSpec {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.pocketable);
        match &self.placed {
            Some(p) => {
                h.write_bool(true);
                p.x.hash_into(h);
                p.hurt_half_height.hash_into(h);
                for v in [
                    p.hitter_lag,
                    p.water,
                    p.grow_delay,
                    p.grow_into,
                    p.grow_move,
                    p.cuts,
                    p.fall_move,
                    p.chip,
                ] {
                    h.write_u8(v);
                }
                h.write_u16(p.chip_chance);
            }
            None => h.write_bool(false),
        }
        h.write_u8(self.kind);
        for v in [
            self.gravity,
            self.max_fall,
            self.air_friction,
            self.friction,
            self.max_speed,
            self.half_width,
            self.half_height,
            self.wall_lift,
            self.bounce_min,
            self.bounce_x.0,
            self.bounce_x.1,
            self.bounce_y.0,
            self.bounce_y.1,
            self.pick_width,
            self.pick_height,
            self.throw_mul,
            self.damage_per_speed,
            self.fighter_lift,
            self.hp,
            self.hurt_radius,
            self.knock_speed.0,
            self.knock_speed.1,
            self.knock_damage.0,
            self.knock_damage.1,
        ] {
            v.hash_into(h);
        }
        for b in [
            self.floor_keep.0,
            self.floor_keep.1,
            self.wall_keep.0,
            self.wall_keep.1,
            self.ceiling_keep.0,
            self.ceiling_keep.1,
            self.fighter_keep.0,
            self.fighter_keep.1,
            self.knock_hitlag,
        ] {
            h.write_u8(b);
        }
        self.hit.hash_into(h);
        h.write_u16(self.thrown_life);
        h.write_bool(self.vanish);
        h.write_u16(self.life);
        match &self.blast {
            Some(b) => {
                h.write_bool(true);
                b.hit.hash_into(h);
                h.write_u8(b.frames);
                h.write_u8(b.delay);
                h.write_bool(b.hits_owner);
            }
            None => h.write_bool(false),
        }
        self.damage_add.hash_into(h);
        h.write_u8(self.uses);
        match &self.trap {
            Some(t) => {
                h.write_bool(true);
                t.hit.hash_into(h);
                t.kick_x.hash_into(h);
                t.kick_y.hash_into(h);
            }
            None => h.write_bool(false),
        }
    }
}

/// A pocket (E42, sim 115: the reference's Pocket). On move frames `start..=end` a circle of `radius` at (`x` ahead, `y` up) from the
/// feet (scaled by the fighter's `hitbox_scale`) puts away the first thing that touches it while the fighter's pocket is empty: a
/// flying projectile of another owner whose tool allows it (`Tool::pocket`; a burst never), or an item that is not held and is
/// `pocketable`. If the fighter holds an item on move frame `start`, that item is put away instead (and an item already in the pocket
/// comes into the hand: the reference's swap). The thing waits in its pool, frozen and unseen (`REST_POCKETED` / `ITEM_POCKETED`),
/// still counting as out for its maker, until a [`PocketRelease`] takes it out; a KO loses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PocketCatch {
    pub start: u8,
    pub end: u8,
    pub x: Fx,
    pub y: Fx,
    pub radius: Fx,
}

/// Taking out what was put away (E42, sim 115): on move frame `frame` a pocketed item comes into the empty hand (its clock starting
/// again), or a pocketed projectile flies out from (`x` ahead, `y` up) with the speed it had as it was caught, turned to go the way the
/// fighter faces, its clock and hits starting again, now the fighter's, dealing `power` percent of its own damage (its earlier
/// multipliers dropped: the reference's 1.9x) and counting as this move for stale moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PocketRelease {
    pub frame: u8,
    pub x: Fx,
    pub y: Fx,
    pub power: u16,
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
    /// What the detector does: `None` skips this move ahead (above); `Some(move)` starts that move (usually an `ext` slot: the
    /// reference's separate reflection move) at once, so its own reflector can turn the shot round on the same frame. With it the
    /// detector works on its whole window, whatever this reflector's own frames are.
    pub detect_then: Option<u8>,
    /// Half the height of the field: above 0 it is a vertical capsule from `y - half_height` to `y + half_height` (a standing barrier)
    /// instead of a circle.
    pub half_height: Fx,
    /// A second field with the same numbers on move frames `body_start..=body_end` (the reference's body reflector beside a barrier):
    /// a circle at (`body_x`, `body_y`) of `body_radius`; radius 0 for none.
    pub body_start: u8,
    pub body_end: u8,
    pub body_x: Fx,
    pub body_y: Fx,
    pub body_radius: Fx,
}

impl Reflector {
    /// An empty reflector (no frames, no field): the base the builders fill in, with every later field at its default.
    pub const NONE: Reflector = Reflector {
        start: 255,
        end: 0,
        x: Fx::ZERO,
        y: Fx::ZERO,
        radius: Fx::ZERO,
        damage_percent: 100,
        speed_percent: 100,
        reverse_vertical: false,
        life_percent: 100,
        max_damage: Fx::ZERO,
        detect_start: 0,
        detect_end: 255,
        detect_x: Fx::ZERO,
        detect_y: Fx::ZERO,
        detect_radius: Fx::ZERO,
        detect_then: None,
        half_height: Fx::ZERO,
        body_start: 0,
        body_end: 0,
        body_x: Fx::ZERO,
        body_y: Fx::ZERO,
        body_radius: Fx::ZERO,
    };

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
        self.detect_then = None;
        self.half_height = Fx::ZERO;
        self.body_start = 0;
        self.body_end = 0;
        self.body_x = Fx::ZERO;
        self.body_y = Fx::ZERO;
        self.body_radius = Fx::ZERO;
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
    /// The answer deals this percent of the caught hit's damage, at least `min_damage` and (when above 0) at most `max_damage`.
    pub percent: u8,
    pub min_damage: Fx,
    pub max_damage: Fx,
    /// Only melee hits are caught: a projectile's hit is taken as usual (the reference's counters that leave projectiles to a
    /// reflector).
    pub melee_only: bool,
    /// Where the stance catches (the reference's counter bubble): a hit that touches it while the window is open is caught. `None`:
    /// the body's hurtboxes catch (older content). A push (`Hitbox::push`) is never caught either way.
    pub bubble: Option<CounterBubble>,
}

/// A counter's catching area, relative to the fighter's feet: a circle of `radius` centred `x` ahead (mirrored by facing) and `y` up,
/// or a standing capsule reaching `half_height` above and below that centre. Scaled by the body's `hitbox_scale`, as hitboxes are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterBubble {
    pub x: Fx,
    pub y: Fx,
    pub half_height: Fx,
    pub radius: Fx,
}

/// Intangibility of part of the body (the reference's per-bone intangibility): during move frames `start..=end` the hurtbox
/// circles in `parts` cannot be hit, while the rest of the body still can. See `combat::hurtboxes` for the circles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartIntangible {
    pub start: u8,
    pub end: u8,
    /// Bits: [`PART_LOW`], [`PART_MIDDLE`], [`PART_HIGH`], [`PART_LIMB`].
    pub parts: u8,
}

/// Air physics for move frames `from..=to` of a move that runs without a motion: gravity, top fall speed and air friction
/// (`None`: the move's flat value, else the body's), the percent of the body's air speed and acceleration kept (`drift`), and a further
/// percent on the acceleration alone (`accel`: the reference's horizontal acceleration multiplier, x6 is 600).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirBlock {
    pub from: u8,
    pub to: u8,
    pub gravity: Option<Fx>,
    pub max_fall: Option<Fx>,
    pub friction: Option<Fx>,
    pub drift: u8,
    pub accel: u16,
    /// The air speed in these frames, in world units, replacing the drift's share (the reference's own air speed for a stretch of a
    /// move); `None`: the drift's.
    pub speed: Option<Fx>,
    /// The stick's share of the air acceleration in these frames (world units a frame at a full tilt), replacing the body's
    /// `air_accel_stick` after the drift and `accel` percents (the reference's "additional air acceleration" of a move); the base
    /// acceleration stays the drift's. `None`: the body's, scaled as above.
    pub accel_stick: Option<Fx>,
}

/// Armor during move frames `start..=end` (the reference's): the fighter takes a hit's damage and hitlag but is not knocked out of the
/// move. `kind` says which hits it withstands: [`ARMOR_SUPER`] every hit, [`ARMOR_DAMAGE`] a hit dealing at most `threshold` percent
/// (compared before the one-on-one damage multiplier, as the reference does), [`ARMOR_KNOCKBACK`] a hit whose knockback is at most
/// `threshold` units (the reference's knockback-based "heavy" armor, sim 89: the knockback `combat::apply_hit` works out, rage, staleness
/// and set knockback included, before crouch cancelling; the one-on-one multiplier never touches it). Grabs ignore armor; a push still
/// pushes.
///
/// `subtract` (knockback kind only): a hit that breaks the armor launches with its knockback less `threshold`, and at least `min_kb`
/// (the reference's subtractive armor and its minimum launch); without it a breaking hit takes its whole knockback. The knockback kind
/// is also used outside moves: a body's air-jump armor (`FighterParams::air_jump_armor`), and later a held state's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Armor {
    pub start: u8,
    pub end: u8,
    pub kind: u8,
    pub threshold: Fx,
    pub subtract: bool,
    pub min_kb: Fx,
}

/// `Armor::kind` values.
pub const ARMOR_SUPER: u8 = 0;
pub const ARMOR_DAMAGE: u8 = 1;
pub const ARMOR_KNOCKBACK: u8 = 2;

impl Armor {
    /// Whether this armor, on frame `frame` of its window, withstands a hit dealing `damage` percent (before the damage multiplier)
    /// with `knockback` units of knockback.
    pub fn withstands(&self, frame: u16, damage: Fx, knockback: Fx) -> bool {
        self.covers(frame)
            && match self.kind {
                ARMOR_SUPER => true,
                ARMOR_DAMAGE => damage <= self.threshold,
                ARMOR_KNOCKBACK => knockback <= self.threshold,
                _ => false,
            }
    }

    /// Whether `frame` is inside the window.
    pub fn covers(&self, frame: u16) -> bool {
        u16::from(self.start) <= frame && frame <= u16::from(self.end)
    }

    /// The knockback a hit that broke this armor on `frame` launches with: less `threshold` and at least `min_kb` for a subtractive
    /// knockback armor inside its window, else `knockback` as it is.
    pub fn broken_knockback(&self, frame: u16, knockback: Fx) -> Fx {
        if self.kind == ARMOR_KNOCKBACK && self.subtract && self.covers(frame) {
            (knockback - self.threshold).max(self.min_kb)
        } else {
            knockback
        }
    }
}

/// A move's own soft-platform drop (E30, sim 95; the reference's ground pound): on move frame `from` the fighter passes through soft
/// platforms for `frames` frames (its `platform_ignore`), and with `while_down` also on every frame of the move from `from` on that the
/// stick is held down (past the crouch threshold). Solid ground always stops it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropThrough {
    pub from: u8,
    pub frames: u8,
    pub while_down: bool,
}

/// A carry (E7, sim 98: the reference's cargo hold). A move with this (a throw slot) is the lift: while it runs, the held fighter is
/// pinned at the move's `hold_at`; when it ends with the fighter still held, the holder goes into [`crate::state::FighterState::Carrying`]
/// instead of standing (the move stays its `move_id`, so this block is read from it all through the carry). While carrying:
///
/// * the hold's clock is set when the lift starts to `hold time(percent now) x clock_num / clock_den - g`, `g` the hold time already
///   used (`Fighter::grab_start - grab_timer`, mashing included: the reference's `((90 + 1.7p) x 11 / 6) - g`), at least 1, and runs
///   on with the same mash cuts as an ordinary hold;
/// * the holder walks only, at `walk` percent of its walk speed and both walk accelerations, turning with the stick; it has `jumps`
///   jumps in all (`Fighter::carry_jumps`: the ground's jump squat and jump, or a midair jump once off the ground); it walks off edges
///   and falls with the body's air physics (drift and fast fall), keeping the held fighter; it can catch a ledge only if `ledge`
///   (which lets go of the held fighter);
/// * the held fighter is pinned at `hold_at` (forward, up) from the holder's feet, mirrored by its facing;
/// * attack starts a throw by the stick (the attack stick too): `throws` are the move slots of the forward (also the stick in the
///   middle), back, up and down throws, ordinary moves with a `HIT_THROW` row;
/// * `armor` (usually the knockback kind) covers the carry, its frames counted from the carry's first (255 and on count as 255): a
///   hit it withstands does not end the carry; any other hit does, as for any hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Carry {
    pub walk: u16,
    pub clock_num: u8,
    pub clock_den: u8,
    pub jumps: u8,
    pub hold_at: (Fx, Fx),
    pub armor: Option<Armor>,
    pub throws: [u8; 4],
    pub ledge: bool,
}

impl Carry {
    /// The throw slot for the stick held up, down or back (in that order of precedence), else the forward throw.
    pub fn throw_for(&self, up: bool, down: bool, back: bool) -> u8 {
        if up {
            self.throws[2]
        } else if down {
            self.throws[3]
        } else if back {
            self.throws[1]
        } else {
            self.throws[0]
        }
    }
}

/// An egg (E8, sim 107: the reference's, owner decision 46). A move with this block shuts the fighter its `EFFECT_ENCASE` throw row
/// releases in an egg ([`crate::state::FighterState::Encased`]) instead of launching it. The numbers are read from the throwing move
/// all through the egg (the egged fighter remembers the thrower's body and the move):
///
/// * the egg lasts `base + per_percent x min(p, percent_cap)` frames, `p` the fighter's percent before the throw, less `hit_cut` frames
///   for every percent the throw (and later each hit on the egg) actually adds; it counts down a frame at a time outside hitlag, and
///   `mash_button` more on a frame with a new button press, `mash_stick` more on one with a stick flick (the shared mash reading);
/// * it is laid at `launch` (forward by the thrower's facing, up: world units a frame) and falls with the fighter's own gravity, fall
///   speed and friction; it catches no ledge, techs nothing, and lands still an egg;
/// * on the ground a stick flick sideways adds `shift.0` that way, a flick up hops it at `shift.1`; in the air `air_shift` instead;
/// * a hit on it adds `damage_taken` percent of its damage, keeps its hitlag, and never launches, stuns or changes its state;
///   grabs and command grabs cannot catch it; a push still pushes it;
/// * its hurtbox is one egg, `size` percent of the fighter's height across when grown, growing from `grow_from` percent of that over
///   `grow_frames` frames;
/// * at 0 the fighter breaks out: airborne, vertical speed `exit_vy`, intangible for `exit_intangible` frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Encase {
    pub base: Fx,
    pub per_percent: Fx,
    pub percent_cap: Fx,
    pub mash_stick: Fx,
    pub mash_button: Fx,
    pub hit_cut: Fx,
    pub damage_taken: u8,
    pub launch: (Fx, Fx),
    pub grow_from: u8,
    pub grow_frames: u8,
    pub size: u8,
    pub shift: (Fx, Fx),
    pub air_shift: (Fx, Fx),
    pub exit_vy: Fx,
    pub exit_intangible: u8,
}

impl Encase {
    /// The egg's length in frames (fixed point) for a fighter at `percent` before the throw: `base + per_percent x min(percent, cap)`.
    pub fn frames(&self, percent: Fx) -> Fx {
        self.base + self.per_percent * percent.min(self.percent_cap).max(Fx::ZERO)
    }

    /// The egg's radius for a fighter of body height `height`, `age` frames after it was laid.
    pub fn radius(&self, height: Fx, age: u8) -> Fx {
        let full = height * Fx::from_ratio(i32::from(self.size), 200);
        let span = i32::from(self.grow_frames.max(1));
        let a = i32::from(age).min(span);
        let from = i32::from(self.grow_from);
        full * Fx::from_ratio(from * span + (100 - from) * a, 100 * span)
    }

    fn hash_into(&self, h: &mut StateHasher) {
        for v in [
            self.base,
            self.per_percent,
            self.percent_cap,
            self.mash_stick,
            self.mash_button,
            self.hit_cut,
            self.launch.0,
            self.launch.1,
            self.shift.0,
            self.shift.1,
            self.air_shift.0,
            self.air_shift.1,
            self.exit_vy,
        ] {
            v.hash_into(h);
        }
        for b in [
            self.damage_taken,
            self.grow_from,
            self.grow_frames,
            self.size,
            self.exit_intangible,
        ] {
            h.write_u8(b);
        }
    }
}

/// Balloons (E41, sim 113: the reference's balloon flight, owner decision 68). A move with this block flies on balloons: while the
/// fighter is in a move with one, it carries `count` balloons (one if its helium is below `one_below`), each popped by hits on the
/// balloons' hurt circle that do not also touch the body (a hit that touches the body is an ordinary hit, which ends the flight):
///
/// * the helium (`Fighter::helium_used`, sixteenths of a frame of flight used) is a tank of `fuel` frames: every frame of a balloon
///   move outside hitlag uses one; when it runs out the balloons let go and the fighter falls helpless; out of a balloon move it
///   refills `refill` sixteenths a frame (the reference's slow recharge), and a KO fills it;
/// * the balloons' circle is `radius` around the point `x` ahead of and `y` above the feet (scaled by the body's `hitbox_scale`);
///   each balloon takes `hp` percent of damage (before the one-on-one multiplier) and pops, a hit popping at most one;
/// * when the last balloon pops the fighter falls helpless with `pop_vy` upward speed (the reference's little hop);
/// * a script reads the count as the `balloons` register (its flaps are weaker on one balloon).
///
/// Leaving a balloon move for one without the block (the move's end, `end()`, a hit) lets the balloons go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Balloons {
    pub count: u8,
    pub hp: Fx,
    pub x: Fx,
    pub y: Fx,
    pub radius: Fx,
    pub fuel: u16,
    pub refill: u16,
    pub one_below: u16,
    pub pop_vy: Fx,
}

impl Balloons {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.count);
        self.hp.hash_into(h);
        self.x.hash_into(h);
        self.y.hash_into(h);
        self.radius.hash_into(h);
        h.write_u16(self.fuel);
        h.write_u16(self.refill);
        h.write_u16(self.one_below);
        self.pop_vy.hash_into(h);
    }
}

/// Additive charge damage (the reference's charge-and-release specials, E29): a hit of a move with this deals its damage plus `add`
/// times the share of `full` frames the fighter's `Fighter::charge` reached (at most all of it), rounded down to a whole percent if
/// `floor`, instead of the smash attacks' multiplicative bonus. A script sets the charge with `set_charge(n)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChargeAdd {
    pub full: u8,
    pub add: Fx,
    pub floor: bool,
}

impl ChargeAdd {
    /// The damage a hit of `base` deals at `charge` frames.
    pub fn damage(&self, base: Fx, charge: u8) -> Fx {
        let full = i32::from(self.full.max(1));
        let share = i32::from(charge).min(full);
        let d = base + self.add.mul_int(share) / Fx::from_int(full);
        if self.floor {
            Fx::from_int(d.floor_int())
        } else {
            d
        }
    }
}

/// `PartIntangible::parts` bits, one per hurtbox circle: low on the body (legs), the middle, high (the head), and the limb an
/// attack throws out (the arm or leg reaching toward the move's hitboxes).
pub const PART_LOW: u8 = 1;
pub const PART_MIDDLE: u8 = 2;
pub const PART_HIGH: u8 = 4;
pub const PART_LIMB: u8 = 8;

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
    /// The special fall's own air speed and stick acceleration after this move (world units; E11, the reference's separate air speed
    /// and "additional acceleration" of a special fall): `fall_speed` replaces the air speed `fall_drift` gives, `fall_accel` the
    /// stick's share of the acceleration (the base acceleration is then the body's own). `None`: `fall_drift` scales both.
    pub fall_speed: Option<Fx>,
    pub fall_accel: Option<Fx>,
    /// A follow-up (a move reached by `next` or a script's `goto`: the reference's two-part smash attack) keeps the charge the move
    /// before it was given, so its hits are charged too (E36). False: it starts uncharged, as every move.
    pub keep_charge: bool,
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
    /// While the move runs without a motion in the air, its upward speed is held at or under this (the reference's maximum vertical
    /// speed of a stance); `None` for no limit.
    pub air_max_rise: Option<Fx>,
    /// Intangibility (`intangible` frames from the start) begins on this move frame instead of the first, when the move started
    /// on the ground (`intangible_from`) or in the air (`intangible_from_air`, 255 = the same as on the ground).
    pub intangible_from: u8,
    pub intangible_from_air: u8,
    /// Touching down during the move does not end it: the fighter lands and the move carries on along the ground.
    pub land_continue: bool,
    /// Frame-ranged air physics (the reference's per-frame move parameters): on move frames `from..=to` of a block, its values replace
    /// the flat `air_*` ones above (the first block that covers the frame; empty for none). See [`AirBlock`].
    pub air_blocks: Vec<AirBlock>,
    /// A fast fall is only possible from this move frame on (0: any frame, as before).
    pub fast_fall_from: u8,
    /// The move's own turnaround (the reference's per-move B-reverse frame): on move frame `.0` the stick held backward past `.1`
    /// percent turns the fighter round (in the air its horizontal speed too). It replaces the shared up-special turn window, the
    /// up-and-back start and the B-reverse window for this move. `None`: the shared rules.
    pub turn_at: Option<(u8, u8)>,
    /// Touching down before this move frame does not end the move (it lands and carries on, as `land_continue`); 0: no such window.
    pub lands_from: u8,
    /// Windows in which part of the body cannot be hit (a kicking leg, a headbutting head); empty for none.
    pub part_intangible: Vec<PartIntangible>,
    /// Armor while the move runs (see [`Armor`]); `None` for none.
    pub armor: Option<Armor>,
    /// The move (usually an `ext` slot) a search box of this move switches to when it touches someone (see [`HIT_SEARCH`]): it starts
    /// on the next frame, with the speed the fighter had.
    pub on_search: Option<u8>,
    /// A special started in the air uses this slot instead (its aerial version, with its own frames and hits); `None`: the same move.
    pub air_version: Option<u8>,
    /// A special input like a smash attack (the stick flicked, or the strong button: the `flick` register) uses this slot instead
    /// (E3d, sim 102: the reference's smash-input version, such as a boomerang thrown further), on the move's first frame; checked
    /// after `air_version`. `None`: the same move.
    pub smash_version: Option<u8>,
    /// The move a command grab of this move goes on with when it catches someone (see `Hitbox::command`): it starts on the next frame,
    /// the holder keeping its speed and staying in the air if it is there.
    pub catch_move: Option<u8>,
    /// A ground move that goes over an edge goes on as this move instead (usually an `ext` slot: the reference's version for leaving
    /// the ground, with its own FAF), on the same frame, keeping its hits so far. `None`: it goes on as itself (a motion) or ends
    /// (no motion), as before.
    pub off_edge: Option<u8>,
    /// Where a fighter held during this move is pinned: (forward, up) from the holder's feet. `None`: the ordinary hold, the ruleset's
    /// `grab_distance` in front on the holder's platform.
    pub hold_at: Option<(Fx, Fx)>,
    /// A smash attack holds on this move frame while the attack button stays held, up to the
    /// ruleset's charge limit, and then deals more damage the longer it was held.
    pub charge_at: Option<u8>,
    /// A jab: pressing attack within `next_window` frames of the move's end continues into this move.
    pub next: Option<u8>,
    pub next_window: u8,
    /// A jab whose next hit can start before its end (the reference's continuable window): from this move frame on, a press of attack
    /// (within `next_window` frames) carries on into `next` at once. 255 (the default) only continues at the end.
    pub next_from: u8,
    /// A jab that can be cancelled (sim 115, owner decision 70: the previous reference version's jab cancel, a crouch or shield from
    /// the continuable frame on): from this move frame on, on the ground, any action the fighter could start standing (a jump, a dash,
    /// a crouch, a shield, a grab, another attack or a special; not a walk, and not the jab itself, which `next` carries on) ends the
    /// move and starts on the same frame. 255 (the default): never.
    pub cancel_from: u8,
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
    /// Damage added by the charge instead of the multiplicative bonus (see [`ChargeAdd`]); `None`: the bonus.
    pub charge_add: Option<ChargeAdd>,
    /// A special used while the fighter's gauge's powered mode runs uses this slot instead (its powered version) and spends the gauge
    /// at once (see `content::Gauge`); `None`: the move is the same in the mode and spends nothing.
    pub limit_version: Option<u8>,
    /// Damage taken during this move drains the fighter's gauge instead of filling it, and so does every hit while it is held by an
    /// ordinary grab that caught it in this move (the reference's charge and its cancel).
    pub gauge_drain: bool,
    /// Started in the air, the fighter keeps this percent of its vertical speed (the reference's vertical speed multiplier on startup:
    /// a stall); 100 keeps it all.
    pub start_speed_y: u8,
    /// Ledges during the move only once the fighter has fallen this far below where the move began (`Fighter::move_y0`): `.0` for a
    /// ledge in front, `.1` for one behind (on top of `ledge_front_from` / `ledge_back_from`). `None`: no such rule.
    pub ledge_drop: Option<(Fx, Fx)>,
    /// An aerial move that touches the ground goes on as this move instead (usually its grounded version), on the same frame, keeping
    /// its frame and its hits so far: the mirror of `off_edge`. `None`: the usual landing.
    pub on_land: Option<u8>,
    /// During the move's motion in the air, the fighter steers sideways with its own air control at this percent of the body's air
    /// speed and acceleration, instead of moving at the motion's `vx` (the motion still sets the vertical speed): a rise with air
    /// control. 0 (the default): the motion's `vx`, as before.
    pub motion_drift: u8,
    /// A jab whose continuable window runs past its end (owner decision 58, sim 95: the reference's window past the FAF): once the move
    /// has ended on the ground, a press of attack that would start a jab starts `.0` instead, up to move frame `.1` (counted on past the
    /// move's end; the fighter is idle meanwhile, and any other action ends it). `None`: a press after the end starts a new jab.
    pub late_next: Option<(u8, u8)>,
    /// The move's ledge windows (`ledge_front_from` / `ledge_back_from` / `grabs_ledge`) catch a ledge only while the fighter is not
    /// rising (sim 95: the reference's "as long as it is not moving upward"). False: rising or not, as before.
    pub ledge_falling_only: bool,
    /// The move drops through soft platforms by itself (sim 95, E30): see [`DropThrough`]. `None`: only the usual crouch and shield drops.
    pub drop_through: Option<DropThrough>,
    /// On move frames `push_immune_from..=push_immune_to` a push (`Hitbox::push`, the reference's windbox) does not move the fighter
    /// (the reference's windbox immunity, sim 90). 255 / 0 (the default): never.
    pub push_immune_from: u8,
    pub push_immune_to: u8,
    /// The move is a carry's lift (E7, sim 98): see [`Carry`]. `None` (the default): an ordinary move.
    pub carry: Option<Carry>,
    /// The egg its `EFFECT_ENCASE` throw row lays (E8, sim 107): see [`Encase`]. `None` (the default): no egg.
    pub encase: Option<Encase>,
    /// Sim 111 (E4): on move frame `.0` an item of the weapon's `items[.1]` appears in the fighter's hand, if the hand is empty (the
    /// reference's bomb pull: none while holding an item). `None`: no item.
    pub pull: Option<(u8, u8)>,
    /// Sim 111 (E4): on this move frame the item in the fighter's hand is used up (the double arrow fires the arrow held); 255 never.
    pub spend_item: u8,
    /// Random versions (E39, sim 113: the reference's moves whose version is drawn when they start, such as the settler's turnips): on
    /// the move's first frame the game's random generator picks `(chance, slot)` with `chance` percent each, the move switching to
    /// that slot (an `ext` slot with the version's frames and hits), and the move itself with what is left of 100. Empty: no draw.
    pub versions: Vec<(u8, u8)>,
    /// The move flies on balloons (E41, sim 113): see [`Balloons`]. `None` (the default): no balloons.
    pub balloons: Option<Balloons>,
    /// The move puts away a projectile or an item in front (E42, sim 115: the reference's Pocket): see [`PocketCatch`].
    pub pocket: Option<PocketCatch>,
    /// The move takes out what the fighter put away (E42, sim 115): see [`PocketRelease`].
    pub unpocket: Option<PocketRelease>,
    /// The move's hits cut a placed piece that can be felled (E43, sim 115: the reference's axe; see [`Placed`]).
    pub chops: bool,
    /// The move throws the item in the fighter's hand at its own speed and angle (E47, sim 117: the reference's banana peel toss; see
    /// [`Toss`]). `None` (the default): no throw.
    pub toss: Option<Toss>,
    /// A hit landing on the fighter during this move (not a push, not one its armor withstands) sheds this move's projectile (E48, sim
    /// 117: the reference's rocket barrels flying off when their wearer is hit): the projectile of move `shed` (usually an `ext` slot
    /// that only holds it) is fired at the fighter's feet (the spec's offset, its facing), credited to the fighter; and if the hit leaves
    /// the fighter in its move (a flinch, no hitstun), the move ends: into the special fall in the air, standing on the ground. `None`
    /// (the default): nothing.
    pub shed: Option<u8>,
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
            fall_speed: None,
            fall_accel: None,
            keep_charge: false,
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
            air_max_rise: None,
            intangible_from: 0,
            intangible_from_air: 255,
            land_continue: false,
            air_blocks: Vec::new(),
            fast_fall_from: 0,
            turn_at: None,
            lands_from: 0,
            part_intangible: Vec::new(),
            armor: None,
            on_search: None,
            air_version: None,
            smash_version: None,
            catch_move: None,
            off_edge: None,
            hold_at: None,
            charge_at: None,
            next: None,
            next_window: 0,
            next_from: 255,
            cancel_from: 255,
            rehit: None,
            rehit_groups: 0xff,
            landing_move: None,
            reflector: None,
            counter: None,
            counter_strike: false,
            charge_bonus: 0,
            charge_add: None,
            limit_version: None,
            gauge_drain: false,
            start_speed_y: 100,
            ledge_drop: None,
            on_land: None,
            motion_drift: 0,
            late_next: None,
            ledge_falling_only: false,
            drop_through: None,
            push_immune_from: 255,
            push_immune_to: 0,
            carry: None,
            encase: None,
            pull: None,
            spend_item: 255,
            versions: Vec::new(),
            balloons: None,
            pocket: None,
            unpocket: None,
            chops: false,
            toss: None,
            shed: None,
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
    /// The items the moveset makes (E4, sim 111: see [`ItemSpec`]), indexed by `Move::pull` and `ProjectileArc::lodge_item`; empty
    /// for none.
    pub items: Vec<ItemSpec>,
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
        h.write_u16(self.hitstun_add as u16);
        h.write_u16(self.sdi);
        self.link_ahead.hash_into(h);
        h.write_u8(self.aim);
        h.write_bool(self.set_weight);
        h.write_bool(self.reverse);
        h.write_bool(self.push);
        h.write_u16(self.shieldstun);
        h.write_u8(self.angle_type);
        h.write_bool(self.command);
        h.write_bool(self.clanks);
        h.write_bool(self.rebounds);
        h.write_u8(self.trip);
        self.half_height.hash_into(h);
        self.posvec_x.hash_into(h);
        self.posvec_y.hash_into(h);
        h.write_u8(self.posvec_frames);
        self.half_width.hash_into(h);
        h.write_u8(self.rules);
        h.write_u8(self.slip);
    }
}

impl StateHash for Weapon {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(u8::from(self.limbs));
        h.write_u32(self.items.len() as u32);
        for it in &self.items {
            it.hash_into(h);
        }
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
                    h.write_u8(a.bounce_keep_y);
                    a.bounce_max_fall.hash_into(h);
                    h.write_bool(p.pierce);
                    h.write_u8(p.rehit);
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
                    match &p.burst {
                        Some(b) => {
                            h.write_bool(true);
                            b.hash_into(h);
                        }
                        None => h.write_bool(false),
                    }
                    a.drag.hash_into(h);
                    h.write_u8(a.lodge);
                    h.write_u8(a.lodge_angle);
                    match a.lodge_item {
                        Some(i) => {
                            h.write_bool(true);
                            h.write_u8(i);
                        }
                        None => h.write_bool(false),
                    }
                    match &p.charged {
                        Some(c) => {
                            h.write_bool(true);
                            h.write_u8(c.full);
                            h.write_u8(c.snap);
                            c.damage_to.hash_into(h);
                            c.speed_to.hash_into(h);
                            h.write_u8(c.life_to);
                            h.write_u16(c.size_from);
                            h.write_u16(c.size_to);
                            h.write_bool(c.floor);
                            h.write_u8(c.times);
                            h.write_u16(c.power_to);
                        }
                        None => h.write_bool(false),
                    }
                    p.tool.hash_into(h);
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
            for v in [
                m.air_gravity,
                m.air_max_fall,
                m.air_friction,
                m.air_max_rise,
            ] {
                match v {
                    Some(x) => {
                        h.write_bool(true);
                        x.hash_into(h);
                    }
                    None => h.write_bool(false),
                }
            }
            h.write_bool(m.land_continue);
            h.write_u32(m.air_blocks.len() as u32);
            for b in &m.air_blocks {
                h.write_u8(b.from);
                h.write_u8(b.to);
                for v in [b.gravity, b.max_fall, b.friction] {
                    match v {
                        Some(x) => {
                            h.write_bool(true);
                            x.hash_into(h);
                        }
                        None => h.write_bool(false),
                    }
                }
                h.write_u8(b.drift);
                h.write_u16(b.accel);
                for v in [b.speed, b.accel_stick] {
                    match v {
                        Some(x) => {
                            h.write_bool(true);
                            x.hash_into(h);
                        }
                        None => h.write_bool(false),
                    }
                }
            }
            h.write_u8(m.fast_fall_from);
            match m.turn_at {
                Some((frame, stick)) => {
                    h.write_bool(true);
                    h.write_u8(frame);
                    h.write_u8(stick);
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.lands_from);
            h.write_u32(m.part_intangible.len() as u32);
            for w in &m.part_intangible {
                h.write_u8(w.start);
                h.write_u8(w.end);
                h.write_u8(w.parts);
            }
            match &m.armor {
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
            match m.hold_at {
                Some((x, y)) => {
                    h.write_bool(true);
                    x.hash_into(h);
                    y.hash_into(h);
                }
                None => h.write_bool(false),
            }
            for slot in [
                m.on_search,
                m.air_version,
                m.catch_move,
                m.off_edge,
                m.smash_version,
            ] {
                match slot {
                    Some(id) => {
                        h.write_bool(true);
                        h.write_u8(id);
                    }
                    None => h.write_bool(false),
                }
            }
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
                    match r.detect_then {
                        Some(id) => {
                            h.write_bool(true);
                            h.write_u8(id);
                        }
                        None => h.write_bool(false),
                    }
                    r.half_height.hash_into(h);
                    h.write_u8(r.body_start);
                    h.write_u8(r.body_end);
                    r.body_x.hash_into(h);
                    r.body_y.hash_into(h);
                    r.body_radius.hash_into(h);
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
                    c.max_damage.hash_into(h);
                    h.write_bool(c.melee_only);
                    match &c.bubble {
                        Some(b) => {
                            h.write_bool(true);
                            b.x.hash_into(h);
                            b.y.hash_into(h);
                            b.half_height.hash_into(h);
                            b.radius.hash_into(h);
                        }
                        None => h.write_bool(false),
                    }
                }
                None => h.write_bool(false),
            }
            h.write_bool(m.counter_strike);
            h.write_u8(m.charge_bonus);
            match m.charge_add {
                Some(c) => {
                    h.write_bool(true);
                    h.write_u8(c.full);
                    c.add.hash_into(h);
                    h.write_bool(c.floor);
                }
                None => h.write_bool(false),
            }
            for slot in [m.limit_version, m.on_land] {
                match slot {
                    Some(id) => {
                        h.write_bool(true);
                        h.write_u8(id);
                    }
                    None => h.write_bool(false),
                }
            }
            h.write_bool(m.gauge_drain);
            h.write_u8(m.start_speed_y);
            h.write_u8(m.motion_drift);
            for v in [m.fall_speed, m.fall_accel] {
                match v {
                    Some(x) => {
                        h.write_bool(true);
                        x.hash_into(h);
                    }
                    None => h.write_bool(false),
                }
            }
            h.write_bool(m.keep_charge);
            match m.late_next {
                Some((id, until)) => {
                    h.write_bool(true);
                    h.write_u8(id);
                    h.write_u8(until);
                }
                None => h.write_bool(false),
            }
            h.write_bool(m.ledge_falling_only);
            match m.drop_through {
                Some(d) => {
                    h.write_bool(true);
                    h.write_u8(d.from);
                    h.write_u8(d.frames);
                    h.write_bool(d.while_down);
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.push_immune_from);
            h.write_u8(m.push_immune_to);
            match m.ledge_drop {
                Some((front, back)) => {
                    h.write_bool(true);
                    front.hash_into(h);
                    back.hash_into(h);
                }
                None => h.write_bool(false),
            }
            match &m.carry {
                Some(c) => {
                    h.write_bool(true);
                    h.write_u16(c.walk);
                    h.write_u8(c.clock_num);
                    h.write_u8(c.clock_den);
                    h.write_u8(c.jumps);
                    c.hold_at.0.hash_into(h);
                    c.hold_at.1.hash_into(h);
                    match &c.armor {
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
                    for t in c.throws {
                        h.write_u8(t);
                    }
                    h.write_bool(c.ledge);
                }
                None => h.write_bool(false),
            }
            match &m.encase {
                Some(e) => {
                    h.write_bool(true);
                    e.hash_into(h);
                }
                None => h.write_bool(false),
            }
            match m.pull {
                Some((frame, item)) => {
                    h.write_bool(true);
                    h.write_u8(frame);
                    h.write_u8(item);
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.spend_item);
            h.write_u32(m.versions.len() as u32);
            for (chance, slot) in &m.versions {
                h.write_u8(*chance);
                h.write_u8(*slot);
            }
            match &m.balloons {
                Some(b) => {
                    h.write_bool(true);
                    b.hash_into(h);
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.cancel_from);
            match &m.pocket {
                Some(p) => {
                    h.write_bool(true);
                    h.write_u8(p.start);
                    h.write_u8(p.end);
                    p.x.hash_into(h);
                    p.y.hash_into(h);
                    p.radius.hash_into(h);
                }
                None => h.write_bool(false),
            }
            match &m.unpocket {
                Some(p) => {
                    h.write_bool(true);
                    h.write_u8(p.frame);
                    p.x.hash_into(h);
                    p.y.hash_into(h);
                    h.write_u16(p.power);
                }
                None => h.write_bool(false),
            }
            h.write_bool(m.chops);
            match &m.toss {
                Some(t) => {
                    h.write_bool(true);
                    h.write_u8(t.frame);
                    h.write_u8(t.again);
                    h.write_u16(t.angle as u16);
                    t.speed.hash_into(h);
                    t.smash_speed.hash_into(h);
                    h.write_u8(t.wait);
                }
                None => h.write_bool(false),
            }
            match m.shed {
                Some(s) => {
                    h.write_bool(true);
                    h.write_u8(s);
                }
                None => h.write_bool(false),
            }
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
                shieldstun: 100,
                angle_type: ANGLE_BY_BOX,
                command: false,
                clanks: true,
                rebounds: true,
                trip: 0,
                half_height: Fx::ZERO,
                posvec_x: Fx::ZERO,
                posvec_y: Fx::ZERO,
                posvec_frames: 0,
                half_width: Fx::ZERO,
                rules: RULES_CURRENT,
                slip: SLIP_NONE,
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
        shieldstun: 100,
        angle_type: ANGLE_BY_BOX,
        command: false,
        clanks: true,
        rebounds: true,
        trip: 0,
        half_height: Fx::ZERO,
        posvec_x: Fx::ZERO,
        posvec_y: Fx::ZERO,
        posvec_frames: 0,
        half_width: Fx::ZERO,
        rules: RULES_CURRENT,
        slip: SLIP_NONE,
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
    // The down tilt's 35% trip chance (ssbwiki.com raw table, 13.0.1; E20, sim 86). At its 30-degree angle it lifts a grounded
    // target, so it can only trip on a slope (as the reference says): never on the flat stages here.
    for h in &mut w.moves[MoveId::DTilt as usize].hitboxes {
        h.trip = 35;
    }
    // The reference's angle types (ssbwiki.com hitbox tables, 13.0.1; sim 67), on every move with a box reaching the fighter's
    // centre or behind it, the only ones where the type can change where a hit sends: by position (type 0) for the up tilt, up and
    // down smash, neutral, up and down air and the get-up attack; forward (3) for the forward smash and forward air. The back air
    // is type 3 too, but the reference fighter has turned round when it hits, so from its starting facing it sends backward.
    angle_types(
        &mut w,
        &[
            (MoveId::UTilt, ANGLE_BY_POSITION),
            (MoveId::FSmash, ANGLE_FORWARD),
            (MoveId::USmash, ANGLE_BY_POSITION),
            (MoveId::DSmash, ANGLE_BY_POSITION),
            (MoveId::NAir, ANGLE_BY_POSITION),
            (MoveId::FAir, ANGLE_FORWARD),
            (MoveId::BAir, ANGLE_BACKWARD),
            (MoveId::UAir, ANGLE_BY_POSITION),
            (MoveId::DAir, ANGLE_BY_POSITION),
            (MoveId::GetUpAttack, ANGLE_BY_POSITION),
        ],
    );
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
    // Get-up attack from the floor (the reference's swordfighter): 7% in front on frames 16-17, then 7% behind on 22-23, angle 48
    // (ssbwiki.com 13.0.1; it was 361 before sim 73, owner decision 27), base knockback 80, growth 48 (it always sends into tumble but
    // barely grows with damage), intangible through frame 23, FAF 46.
    let mut getup_attack = ref_move(
        46,
        0,
        0,
        255,
        &[
            r(16, 17, 18, 5, 9, 70, 48, 80, 48, 0, 0),
            r(22, 23, -18, 5, 9, 70, 48, 80, 48, 0, 1),
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
    //
    // It catches on its counter bubble (ssbwiki.com 13.0.1, the parameters): radius 10.5 on the root bone, offset [0, 10, 0], so
    // 10 up and centred on the body [R]; in world units radius 1.3 at 1.3 up [D: x 1/8, to the tenth]. The answer is capped at 50%
    // [R]. A push (a windbox) is never caught.
    let mut counter = ref_move(64, 0, 0, 255, &[]);
    counter.counter = Some(Counter {
        start: 5,
        end: 26,
        then: MoveId::Ext0 as u8,
        percent: 120,
        min_damage: Fx::from_int(8),
        max_damage: Fx::from_int(50),
        melee_only: false,
        bubble: Some(CounterBubble {
            x: Fx::ZERO,
            y: Fx::from_ratio(13, 10),
            half_height: Fx::ZERO,
            radius: Fx::from_ratio(13, 10),
        }),
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
        items: Vec::new(),
    }
}

/// Sets fields on the hitboxes `rows` (indices, in the order the rows were given) of a move.
fn set_rows(mv: &mut Move, rows: &[usize], f: impl Fn(&mut Hitbox)) {
    for &i in rows {
        f(&mut mv.hitboxes[i]);
    }
}

/// Gives every damaging hitbox of each move its reference angle type (`Hitbox::angle_type`): `(move, type)` pairs.
fn angle_types(w: &mut Weapon, moves: &[(MoveId, u8)]) {
    for &(id, t) in moves {
        for hb in &mut w.moves[id as usize].hitboxes {
            if hb.kind == HIT_NORMAL {
                hb.angle_type = t;
            }
        }
    }
}

/// Gives the hitboxes `rows` of move `id` the angle type `t`.
fn angle_type_rows(w: &mut Weapon, id: MoveId, rows: &[usize], t: u8) {
    set_rows(&mut w.moves[id as usize], rows, |h| h.angle_type = t);
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
    // Its 40% trip chance on every box (ssbwiki.com raw table, 13.0.1; owner decision 51): at 70 and 90 degrees it lifts, so it never
    // trips on the flat stages (ssbwiki.com: "can never occur").
    for h in &mut w.moves[MoveId::DTilt as usize].hitboxes {
        h.trip = 40;
    }

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
    // The second hit's 30% trip chance (ssbwiki.com raw table, 13.0.1; owner decision 51): at 43 degrees it lifts, so it never trips
    // on the flat stages.
    set_rows(&mut dsmash, &[4, 5], |h| h.trip = 30);
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
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
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
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        // (A line its thrower holds is not a thing of its own: no pocket can take it, E42.)
        tool: Tool {
            pocket: POCKET_NEVER,
            ..Tool::NONE
        },
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

    // The reference's angle types (ssbwiki.com hitbox tables, 13.0.1; sim 67), on every move with a box reaching the fighter's
    // centre or behind it: forward (type 3) for the forward tilt, the up tilt's first hit, the forward air, the down air's landing
    // hit and both down-special kicks; by position (0) for the up tilt's second hit, the down tilt, up smash, neutral air, down air
    // and the up special; backward (4) for the back air and the down smash's boxes behind (its front ones forward); the up air's
    // grounded loop box forward, its aerial loop box and final hit by position. (The dash attack is the earlier game's, kept at the
    // default; the get-up attack is the claws'.)
    angle_types(
        &mut w,
        &[
            (MoveId::FTilt, ANGLE_FORWARD),
            (MoveId::DTilt, ANGLE_BY_POSITION),
            (MoveId::USmash, ANGLE_BY_POSITION),
            (MoveId::NAir, ANGLE_BY_POSITION),
            (MoveId::FAir, ANGLE_FORWARD),
            (MoveId::BAir, ANGLE_BACKWARD),
            (MoveId::DAir, ANGLE_BY_POSITION),
            (MoveId::Ext0, ANGLE_FORWARD),
            (MoveId::UpSpecial, ANGLE_BY_POSITION),
            (MoveId::DownSpecial, ANGLE_FORWARD),
            (MoveId::Ext6, ANGLE_FORWARD),
        ],
    );
    angle_type_rows(&mut w, MoveId::UTilt, &[0, 1, 2], ANGLE_FORWARD);
    angle_type_rows(&mut w, MoveId::UTilt, &[3, 4, 5], ANGLE_BY_POSITION);
    angle_type_rows(&mut w, MoveId::DSmash, &[0, 1, 4], ANGLE_FORWARD);
    angle_type_rows(&mut w, MoveId::DSmash, &[2, 3, 5], ANGLE_BACKWARD);
    angle_type_rows(&mut w, MoveId::UAir, &[0], ANGLE_FORWARD);
    angle_type_rows(&mut w, MoveId::UAir, &[1, 2], ANGLE_BY_POSITION);
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
    // The window runs to frame 24, past the FAF (owner decision 58).
    late_chain(&mut jab1, MoveId::Jab2, 24);
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
    late_chain(&mut jab2, MoveId::Jab3, 24);
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
    // The 40% trip chance as data (ssbwiki.com raw table, 13.0.1; owner decision 51): at 80 degrees it lifts, so it never trips here.
    for h in &mut w.moves[MoveId::DTilt as usize].hitboxes {
        h.trip = 40;
    }

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

    // The reference's angle types (ssbwiki.com hitbox tables, 13.0.1; sim 67), on every move with a box reaching the fighter's
    // centre or behind it: forward (type 3) for jab 2, the forward tilt, forward air and up special; backward (4) for the back air;
    // by position (0) for the up tilt, up and down smash, neutral, up and down air, the down air's landing hit, the back throw's
    // hits on bystanders, the ledge attack and the get-up attack.
    angle_types(
        &mut w,
        &[
            (MoveId::Jab2, ANGLE_FORWARD),
            (MoveId::FTilt, ANGLE_FORWARD),
            (MoveId::UTilt, ANGLE_BY_POSITION),
            (MoveId::USmash, ANGLE_BY_POSITION),
            (MoveId::DSmash, ANGLE_BY_POSITION),
            (MoveId::NAir, ANGLE_BY_POSITION),
            (MoveId::FAir, ANGLE_FORWARD),
            (MoveId::BAir, ANGLE_BACKWARD),
            (MoveId::UAir, ANGLE_BY_POSITION),
            (MoveId::DAir, ANGLE_BY_POSITION),
            (MoveId::Ext0, ANGLE_BY_POSITION),
            (MoveId::UpSpecial, ANGLE_FORWARD),
            (MoveId::BThrow, ANGLE_BY_POSITION),
            (MoveId::LedgeAttack, ANGLE_BY_POSITION),
            (MoveId::GetUpAttack, ANGLE_BY_POSITION),
        ],
    );
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
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
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
        ..Reflector::NONE
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
            ..ProjectileArc::NONE
        },
        // Its age is 1 on frame 17: the late hit from frame 22, the latest from 47.
        phases: phase_list(&[(6, hit(50, 28, 15, 275)), (31, hit(40, 22, 10, 250))]),
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
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

/// A jab whose continuable window runs past its FAF (owner decision 58): after the move ends, a jab press up to reference frame
/// `last` (the window's last frame, counted on from the move's start) starts `next` instead of a new jab.
fn late_chain(mv: &mut Move, next: MoveId, last: u8) {
    mv.late_next = Some((next as u8, last - 1));
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
    // Its 40% trip chance on every box (ssbwiki.com raw table, 13.0.1; owner decision 51, sim 90): at the 361 angle's low knockback
    // it sends along the ground, so it can trip.
    for h in &mut w.moves[MoveId::DTilt as usize].hitboxes {
        h.trip = 40;
    }
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
    // Forward air: frames 7-9, 9%, angle 60, landing lag 10, autocancels from 30 (UFD ">29", ssbwiki.com), FAF 41.
    w.moves[MoveId::FAir as usize] =
        ref_move(41, 10, 0, 30, &[r(7, 9, 23, 10, 9, 90, 60, 45, 85, 0, 0)]);

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
            shieldstun: 100,
            angle_type: ANGLE_BY_BOX,
            command: false,
            clanks: true,
            rebounds: true,
            trip: 0,
            half_height: Fx::ZERO,
            posvec_x: Fx::ZERO,
            posvec_y: Fx::ZERO,
            posvec_frames: 0,
            half_width: Fx::ZERO,
            rules: RULES_CURRENT,
            slip: SLIP_NONE,
        },
        end_damage: Fx::from_int(6),
        arc: ProjectileArc::NONE,
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
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
    // Every box clanks but cannot rebound (the reference's `rebound=f` without `clang=f`, ssbwiki.com raw table; E3g, sim 86): a clank
    // stops its hit for that frame and the smash carries on.
    for h in &mut usmash.hitboxes {
        h.rebounds = false;
    }
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
    // Its 35% trip chance on every box (ssbwiki.com raw table, 13.0.1; owner decision 51): at 30 and 35 degrees it lifts, so it never
    // trips on the flat stages.
    for h in &mut dsmash.hitboxes {
        h.trip = 35;
    }
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
        ..Reflector::NONE
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
    // Get-up attack from the floor (reference numbers): as the swordfighter's (angle 48 since sim 73), but the second hit on frames
    // 21-22 and intangible through frame 22.
    let mut getup_attack = ref_move(
        46,
        0,
        0,
        255,
        &[
            r(16, 17, 18, 5, 9, 70, 48, 80, 48, 0, 0),
            r(21, 22, -18, 5, 9, 70, 48, 80, 48, 0, 1),
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
    // The reference's angle types (ssbwiki.com hitbox tables, 13.0.1; sim 67), on every move with a box reaching the fighter's
    // centre or behind it: by position (type 0) for the up and down tilt, up and down smash, neutral, up and down air, the up
    // special's wind-up hit, the reflector's hit and the get-up attack; backward (4) for the back air; forward (3) for the up
    // special's last hits; the up special's looping hits are the reference's type 1, from the direction of travel (sim 73, owner
    // decision 23).
    angle_types(
        &mut w,
        &[
            (MoveId::UTilt, ANGLE_BY_POSITION),
            (MoveId::DTilt, ANGLE_BY_POSITION),
            (MoveId::USmash, ANGLE_BY_POSITION),
            (MoveId::DSmash, ANGLE_BY_POSITION),
            (MoveId::NAir, ANGLE_BY_POSITION),
            (MoveId::BAir, ANGLE_BACKWARD),
            (MoveId::UAir, ANGLE_BY_POSITION),
            (MoveId::DAir, ANGLE_BY_POSITION),
            (MoveId::DownSpecial, ANGLE_BY_POSITION),
            (MoveId::Ext0, ANGLE_FORWARD),
            (MoveId::Ext1, ANGLE_FORWARD),
            (MoveId::GetUpAttack, ANGLE_BY_POSITION),
        ],
    );
    angle_type_rows(&mut w, MoveId::UpSpecial, &[0], ANGLE_BY_POSITION);
    angle_type_rows(
        &mut w,
        MoveId::UpSpecial,
        &[1, 2, 3, 4, 5, 6, 7, 8],
        ANGLE_BY_TRAVEL,
    );
    w
}

#[path = "moves_gauntlets.rs"]
mod gauntlets;
pub use gauntlets::gauntlets;

#[path = "moves_staff.rs"]
mod staff;
pub use staff::staff;

#[path = "moves_greatsword.rs"]
mod greatsword;
pub use greatsword::greatsword;
/// The Greatsword's special follow-up slots (which `ext` slot holds which part), for tests and tools.
pub mod greatsword_slots {
    pub use super::greatsword::{
        CHARGE_CANCEL, COMBO_2, COMBO_3, FALL, FALL_LANDING, FINISHER, FINISHER_AIR, POWER_COMBO,
        POWER_RISE, POWER_WAVE, POWER_WAVE_AIR, RISE, WAVE_AIR,
    };
}

#[path = "moves_bracers.rs"]
mod bracers;
pub use bracers::bracers;
/// The Bracers' carry slots (the cargo throws' `ext` slots) and armor threshold, for tests and tools.
pub mod bracers_slots {
    pub use super::bracers::{CARGO_ARMOR_KB, CARGO_BACK, CARGO_DOWN, CARGO_FORWARD, CARGO_UP};
}

#[path = "moves_clutch.rs"]
mod clutch;
pub use clutch::clutch;
/// The Clutch's follow-up slots (which `ext` slot holds which part), for tests and tools.
pub mod clutch_slots {
    pub use super::clutch::{DAIR_LANDING, DROP_AIR, DROP_LANDING, LAY, ROLL_END};
}

#[path = "moves_buckler.rs"]
mod buckler;
pub use buckler::buckler;
/// The Buckler's follow-up slots (which `ext` slot holds which part), for tests and tools.
pub mod buckler_slots {
    pub use super::buckler::{
        AIR_SPIN_HITS, ARROW_ITEM, ARROW_KIND, BOMB_DETONATE, BOMB_ITEM, BOMB_KIND,
        BOOMERANG_CATCH, BOOMERANG_FAILED, BOOMERANG_FAR, BOW_FIRE, DAIR_BOUNCE, DAIR_BOUNCE_END,
        DOUBLE_FIRE, DOUBLE_SECOND, FSMASH_2, GROUND_SPIN_PHASES, SPIN_AIR, SPIN_GRAVITY,
        SPIN_RISE_SPEED,
    };
}

#[path = "moves_pockets.rs"]
mod pockets;
pub use pockets::pockets;
/// The Pockets' follow-up slots (which `ext` slot holds which part), and its balloons, for tests and tools.
pub mod pockets_slots {
    pub use super::pockets::{
        balloon_block, AXE, CHIP_ITEM, CHIP_KIND, DAIR_ONE, DAIR_TWO, FALL_KIND, FLIGHT, PLANT,
        PLANT_FAILS, POCKET_OUT, RIDE, RIDE_BOUNCE, RIDE_CRASH, RIDE_HOP, ROCKET_KIND,
        SAPLING_ITEM, SAPLING_KIND, TREE_FALLS, TREE_GROWS, TREE_ITEM, TREE_KIND, UAIR_ONE,
        UAIR_TWO, WATER,
    };
}

#[path = "moves_popgun.rs"]
mod popgun;
pub use popgun::popgun;
/// The Popgun's follow-up slots (which `ext` slot holds which part) and its banana peel, for tests and tools.
pub mod popgun_slots {
    pub use super::popgun::{
        banana_item, BANANA_ITEM, BANANA_KIND, BARRELS, BARRELS_KIND, PEEL_FAILS,
        PEEL_FAILS_WITH_ITEM, PEEL_WAIT, PEEL_WITH_ITEM, POPGUN_FIRE, POPGUN_FULL, POPGUN_MISFIRE,
        ROCKET_CRASH, ROCKET_FLIGHT,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_weapon_has_every_move_slot() {
        for w in [
            longsword(),
            claws(),
            needles(),
            embers(),
            gauntlets(),
            greatsword(),
            bracers(),
            clutch(),
            buckler(),
            pockets(),
            popgun(),
        ] {
            assert_eq!(w.moves.len(), MoveId::COUNT);
            for (i, m) in w.moves.iter().enumerate() {
                if m.is_empty() {
                    assert!(
                        MoveId::from_index(i as u8).may_be_empty(),
                        "only specials and jab chain slots may be empty"
                    );
                    continue;
                }
                // (A thrown tool's catch move, E3d, is an animation only.)
                let catch = w
                    .moves
                    .iter()
                    .any(|o| o.projectile.and_then(|p| p.tool.catch_move) == Some(i as u8));
                assert!(
                    !m.hitboxes.is_empty()
                        || m.projectile.is_some()
                        || m.counter.is_some()
                        || m.script.is_some()
                        || !m.motion.is_empty()
                        || catch
                        || m.carry.is_some()
                        || m.pull.is_some()
                        || m.pocket.is_some()
                        || m.unpocket.is_some()
                        || m.toss.is_some()
                );
                for hb in &m.hitboxes {
                    assert!(
                        hb.start <= hb.end && hb.end <= m.total_frames,
                        "{}",
                        MoveId::from_index(i as u8).name()
                    );
                    assert!(
                        (hb.damage > Fx::ZERO
                            || hb.push
                            || matches!(hb.kind, HIT_GRAB | HIT_SEARCH))
                            && hb.radius > Fx::ZERO
                    );
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
