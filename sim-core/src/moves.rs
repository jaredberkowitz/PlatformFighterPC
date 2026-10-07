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
}

impl MoveId {
    pub const COUNT: usize = 26;
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

    /// Slots a weapon may leave empty: specials it does not have, and jab hits it does not chain into.
    pub const fn may_be_empty(self) -> bool {
        self.is_special() || matches!(self, MoveId::Jab2 | MoveId::Jab3)
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
            _ => MoveId::DownSpecial,
        }
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
    /// Launch angle in degrees, 0 = forward. 361 is the "Sakurai" angle: horizontal for weak hits on the
    /// ground, 44 degrees otherwise.
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
}

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
    /// The fighter can grab a ledge while the move is in progress, even while rising (an up special).
    pub grabs_ledge: bool,
    /// A smash attack holds on this move frame while the attack button stays held, up to the
    /// ruleset's charge limit, and then deals more damage the longer it was held.
    pub charge_at: Option<u8>,
    /// A jab: pressing attack within `next_window` frames of the move's end continues into this move.
    pub next: Option<u8>,
    pub next_window: u8,
    /// A multi-hit move: from move frame `.0`, every `.1` frames the move's hits are allowed to land again.
    pub rehit: Option<(u8, u8)>,
    pub reflector: Option<Reflector>,
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
            charge_at: None,
            next: None,
            next_window: 0,
            rehit: None,
            reflector: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_frames == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weapon {
    /// Indexed by `MoveId as usize`.
    pub moves: Vec<Move>,
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
    }
}

impl StateHash for Weapon {
    fn hash_into(&self, h: &mut StateHasher) {
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
                }
                None => h.write_bool(false),
            }
            h.write_u8(m.intangible);
            h.write_bool(m.helpless_after);
            h.write_bool(m.turns_around);
            h.write_bool(m.grabs_ledge);
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
                }
                None => h.write_bool(false),
            }
        }
    }
}

// ---- Builders -------------------------------------------------------------------------------------------

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
    }
}

/// Marks every hitbox of a move as `kind` (a grab, throw or pummel).
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
    let none = (0, 255);
    let mut moves = vec![
        // Jab
        mk(
            18,
            0,
            none,
            &[
                B(3, 5, 26, 11, 6, 8, 361, 10, 20, 0),
                B(3, 5, 15, 11, 7, 6, 361, 8, 18, 1),
            ],
        ),
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
        // Dash attack
        mk(
            36,
            0,
            none,
            &[
                B(8, 13, 26, 10, 6, 16, 361, 20, 80, 0),
                B(8, 13, 15, 10, 8, 12, 361, 16, 75, 1),
            ],
        ),
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
        // Down air
        mk(
            48,
            18,
            (10, 40),
            &[
                B(11, 14, 6, 1, 6, 18, 270, 20, 80, 0),
                B(11, 14, 6, 6, 7, 12, 270, 16, 75, 1),
            ],
        ),
        // Specials (the up special is replaced below)
        Move::empty(),
        Move::empty(),
        Move::empty(),
        Move::empty(),
        // Jab 2 and 3 (this weapon's jab does not chain)
        Move::empty(),
        Move::empty(),
        // Grab, dash grab, pummel and throws: placeholders (the published data used here is for the brawler)
        with_kind(
            ref_move(30, 0, 0, 255, &[r(6, 7, 17, 11, 11, 0, 361, 0, 0, 0, 0)]),
            HIT_GRAB,
        ),
        with_kind(
            ref_move(38, 0, 0, 255, &[r(8, 9, 19, 11, 11, 0, 361, 0, 0, 0, 0)]),
            HIT_GRAB,
        ),
        with_kind(
            ref_move(22, 0, 0, 255, &[r(4, 4, 13, 11, 10, 13, 361, 0, 0, 0, 0)]),
            HIT_PUMMEL,
        ),
        with_kind(
            ref_move(
                36,
                0,
                0,
                255,
                &[r(13, 13, 13, 11, 10, 80, 40, 60, 55, 0, 0)],
            ),
            HIT_THROW,
        ),
        with_kind(
            ref_move(
                40,
                0,
                0,
                255,
                &[r(20, 20, -13, 11, 10, 90, 45, 55, 65, 0, 0)],
            ),
            HIT_THROW,
        ),
        with_kind(
            ref_move(
                40,
                0,
                0,
                255,
                &[r(22, 22, 13, 11, 10, 70, 85, 60, 85, 0, 0)],
            ),
            HIT_THROW,
        ),
        with_kind(
            ref_move(
                34,
                0,
                0,
                255,
                &[r(14, 14, 13, 11, 10, 50, 80, 40, 80, 0, 0)],
            ),
            HIT_THROW,
        ),
    ];

    // Forward tilt: first active 8, 9/12 damage (sour/tip), angle 361, FAF 34.
    moves[MoveId::FTilt as usize] = ref_move(
        34,
        0,
        0,
        255,
        &[
            r(8, 11, 19, 11, 8, 90, 361, 30, 70, 1, 0),
            r(8, 11, 34, 11, 7, 120, 361, 55, 85, 0, 0),
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
    // Forward smash: frames 10-13, 13% (18% at the tip), Sakurai angle, FAF 52. Charges on frame 2.
    let mut fsmash = ref_move(
        52,
        0,
        0,
        255,
        &[
            r(10, 13, 20, 12, 8, 130, 361, 48, 75, 1, 0),
            r(10, 13, 12, 11, 8, 130, 361, 48, 75, 1, 0),
            r(10, 13, 5, 12, 8, 130, 361, 48, 75, 1, 0),
            r(10, 13, 34, 12, 7, 180, 361, 80, 80, 0, 0),
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
    // at the tip), Sakurai angle, FAF 56. Charges on frame 4.
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
    // Up air: frames 5-9, 9.5% (13% at the tip), angle 80 (90 at the tip), landing lag 8, autocancels on
    // frames 1-2 and from 38, FAF 46.
    moves[MoveId::UAir as usize] = ref_move(
        46,
        8,
        2,
        38,
        &[
            r(5, 9, 5, 26, 8, 95, 80, 40, 80, 1, 0),
            r(5, 9, 3, 20, 7, 95, 80, 40, 80, 1, 0),
            r(5, 9, 0, 14, 6, 95, 80, 40, 80, 1, 0),
            r(5, 9, 4, 34, 7, 130, 90, 40, 84, 0, 0),
        ],
    );
    // Neutral air: two separate hits (frames 6-7, then 15-21). Landing lag 7, autocancels from frame 47, FAF 50.
    moves[MoveId::NAir as usize] = ref_move(
        50,
        7,
        0,
        47,
        &[
            r(6, 7, 17, 12, 8, 35, 75, 45, 50, 1, 0),
            r(6, 7, 30, 12, 7, 50, 90, 35, 50, 0, 0),
            r(15, 21, 17, 12, 8, 70, 361, 50, 90, 1, 1),
            r(15, 21, 30, 12, 7, 95, 361, 60, 100, 0, 1),
        ],
    );
    // Forward air: frames 6-8, 8/11.5 damage (sour/tip), angle 361, landing lag 10, autocancels from 36, FAF 38.
    moves[MoveId::FAir as usize] = ref_move(
        38,
        10,
        0,
        36,
        &[
            r(6, 8, 19, 12, 8, 80, 361, 40, 80, 1, 0),
            r(6, 8, 34, 11, 7, 115, 361, 40, 80, 0, 0),
        ],
    );
    // Back air: frames 7-11, 9/12.5 damage, angle 361, landing lag 10, autocancels frames 1-2 and from 32, FAF 40.
    // The fighter ends the move facing the other way.
    let mut bair = ref_move(
        40,
        10,
        2,
        32,
        &[
            r(7, 11, -19, 12, 8, 90, 361, 40, 85, 1, 0),
            r(7, 11, -34, 12, 7, 125, 361, 40, 94, 0, 0),
        ],
    );
    bair.turns_around = true;
    moves[MoveId::BAir as usize] = bair;

    // Up special: a rising slash. Intangible for the first 5 frames, hits on frames 5-11 (11% early tip,
    // 7% after), then helpless. The sources do not give its travel distance, so the rise (about 44
    // reference units) is an estimate.
    let su = |n: i32| Fx::from_ratio(n, 8000);
    let mut up_special = ref_move(
        50,
        24,
        0,
        255,
        &[
            r(5, 6, 18, 22, 9, 110, 74, 70, 74, 0, 0),
            r(6, 11, 18, 22, 11, 70, 74, 20, 90, 1, 0),
        ],
    );
    up_special.motion = vec![
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
    up_special.intangible = 5;
    up_special.helpless_after = true;
    up_special.grabs_ledge = true;
    moves[MoveId::UpSpecial as usize] = up_special;

    Weapon { moves }
}

/// Short-reach, quick and heavy hitting: the longsword's moves pulled in close and sped up, with the
/// forward tilt, neutral air, forward air and blaster built from reference frame data (a blaster-wielding
/// brawler archetype).
pub fn claws() -> Weapon {
    let mut w = longsword();
    for (i, m) in w.moves.iter_mut().enumerate() {
        if MoveId::from_index(i as u8) == MoveId::UpSpecial {
            *m = Move::empty();
            continue;
        }
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
    // Down tilt: a low kick on frames 5-6, 6%, Sakurai angle, base knockback 25, growth 100, FAF 28.
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
    // Neutral air: 12% early (frames 7-9), then a long 8% hit (frames 10-26). Landing lag 9, autocancels
    // frames 1-6 and from 38, FAF 43.
    w.moves[MoveId::NAir as usize] = ref_move(
        43,
        9,
        6,
        38,
        &[
            r(7, 9, 13, 11, 11, 120, 361, 30, 75, 0, 0),
            r(10, 26, 13, 11, 12, 80, 361, 0, 100, 1, 0),
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
        },
        end_damage: Fx::from_int(6),
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
    // Back air: a back kick, frames 13-15, 15% / 13% / 11% from the foot in, Sakurai angle, base knockback 37,
    // growth 96. Landing lag 15, autocancels on 1-7 and from 19, FAF 45. Wolf keeps facing forward.
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

    // Forward smash: frames 20-23, 15%, Sakurai angle, base knockback 30, growth 106, FAF 42. Charges on frame 6.
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

    // Wolf Flash (side special): after a 19 frame wind-up a dash of about 7 world units (3% on the way), ending in a
    // 20% spike with a 15% hit around it. Helpless in the air. The distance, the ending hit knockback and the
    // total length are estimates; it cannot be angled yet.
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
    flash.motion = vec![
        Motion {
            start: 18,
            end: 27,
            vx: su(5500),
            vy: Fx::ZERO,
        },
        // The dash stops dead where it ends.
        Motion {
            start: 28,
            end: 28,
            vx: Fx::ZERO,
            vy: Fx::ZERO,
        },
    ];
    flash.helpless_after = true;
    w.moves[MoveId::SideSpecial as usize] = flash;

    // Fire Wolf (up special): after an 18 frame wind-up a rising flame kick that hits five times (4%, 2.5% x 3, then
    // 6%; the last hit launches) and then leaves Wolf helpless. It can grab the ledge mid-move. The travel
    // (about 5.6 up and 2.6 forward world units) and the knockback values are estimates.
    let mut fire = ref_move(
        55,
        0,
        0,
        255,
        &[
            r(18, 18, 9, 13, 13, 40, 65, 100, 0, 0, 0),
            r(19, 28, 9, 13, 13, 25, 65, 100, 0, 0, 0),
            r(29, 31, 9, 13, 14, 60, 45, 60, 136, 0, 0),
        ],
    );
    fire.motion = vec![
        Motion {
            start: 17,
            end: 30,
            vx: su(1500),
            vy: su(3200),
        },
        // The kick ends and Wolf tumbles into the helpless fall with only a little speed left.
        Motion {
            start: 31,
            end: 31,
            vx: su(300),
            vy: su(400),
        },
    ];
    fire.rehit = Some((17, 3));
    fire.helpless_after = true;
    fire.grabs_ledge = true;
    w.moves[MoveId::UpSpecial as usize] = fire;

    // Reflector (down special): a reflecting field in front of Wolf from frame 9 to 21 that turns projectiles
    // around (1.5 times the damage), plus a 4% hit on anyone touching it. The reflector frames, size and
    // reflected speed are estimates; the reference frames 5-8 of intangibility are not implemented.
    let mut reflector = ref_move(31, 0, 0, 255, &[r(9, 12, 14, 11, 15, 40, 65, 60, 85, 0, 0)]);
    reflector.reflector = Some(Reflector {
        start: 8,
        end: 20,
        x: Fx::from_ratio(14, 10),
        y: Fx::from_ratio(11, 10),
        radius: Fx::from_ratio(17, 10),
        damage_percent: 150,
        speed_percent: 130,
    });
    w.moves[MoveId::DownSpecial as usize] = reflector;

    // Grabs and throws. Standing grab: hits on frame 7 (dash grab frame 8); pummel 1.3%; throws release on frame 11
    // (forward, 9%), 24 (back, 11%), 27 (up, 7%) and 26 (down, 8.5%) with first actionable frames 33, 48, 46
    // and 41. Throw angles and the grab reach, pummel speed and the grab and dash grab lengths are estimates;
    // the sources disagree on the back throw's damage (8% or 11%).
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(30, 0, 0, 255, &[r(7, 8, 17, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(38, 0, 0, 255, &[r(8, 9, 19, 11, 11, 0, 361, 0, 0, 0, 0)]),
        HIT_GRAB,
    );
    w.moves[MoveId::Pummel as usize] = with_kind(
        ref_move(22, 0, 0, 255, &[r(4, 4, 13, 11, 10, 13, 361, 0, 0, 0, 0)]),
        HIT_PUMMEL,
    );
    w.moves[MoveId::FThrow as usize] = with_kind(
        ref_move(
            33,
            0,
            0,
            255,
            &[r(11, 11, 13, 11, 10, 90, 45, 55, 57, 0, 0)],
        ),
        HIT_THROW,
    );
    w.moves[MoveId::BThrow as usize] = with_kind(
        ref_move(
            48,
            0,
            0,
            255,
            &[r(24, 24, -13, 11, 10, 110, 50, 40, 150, 0, 0)],
        ),
        HIT_THROW,
    );
    w.moves[MoveId::UThrow as usize] = with_kind(
        ref_move(
            46,
            0,
            0,
            255,
            &[r(27, 27, 13, 11, 10, 70, 80, 75, 110, 0, 0)],
        ),
        HIT_THROW,
    );
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
                assert!(!m.hitboxes.is_empty() || m.projectile.is_some());
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
            let tip = m.hitboxes.iter().find(|h| h.priority == 0).unwrap();
            let body = m.hitboxes.iter().find(|h| h.priority == 1).unwrap();
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
        assert_eq!(fair.hitboxes[0].start, 5);
        assert_eq!(fair.hitboxes[0].end, 7);
        assert_eq!(fair.total_frames, 36);
        assert_eq!(fair.autocancel_after, 35);
    }
}
