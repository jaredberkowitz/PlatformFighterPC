//! The Dagger moveset (the phantom body), modelled mechanically on a reference character: the body's normals with its partner's
//! boxes, the gun, the curse bolt, the guard and the partner's specials (phase 1), the hook and its tether (phase 2, sim 123). A child module of `moves` (it uses that file's private
//! row builders). Data and sources: the thirteenth moveset's `REFERENCE_DATA.md` (`docs/movesets/dagger/`), read from ssbwiki.com's raw
//! hitbox tables (13.0.1), ultimateframedata.com and dragdown.wiki.
//!
//! **The partner** (E52, E53): the phantom's gauge (`FighterParams::phantom_gauge`) fills by itself, from damage taken and from hits on
//! its guard; full, its partner comes for 1800 frames. A move begun while the partner is out (`Fighter::move_powered`) uses its
//! `MODE_POWERED` boxes instead of its `MODE_PLAIN` ones (the reference's "awakened" boxes: the same damage, other knockback, size and
//! hitlag), and the partner's own boxes (`part` 1) hit alongside the phantom's own, each target once per part (the reference's part 1:
//! "the total damage of a move is the sum of both"). Pummel, throws, ledge and get-up attacks have no partner boxes [R]. Each special
//! has a powered version (`Move::limit_version`) that does not spend the gauge (`Gauge::keeps`).

use super::*;
use crate::content::FighterParams;

/// A reference length in tenths of its units (85 for 8.5) as world tenths: x 1.25, rounded half away from zero to a whole tenth
/// (the house rule) [D].
const fn u(reference_tenths: i32) -> i32 {
    let n = reference_tenths * 125;
    if n >= 0 {
        (n + 500) / 1000
    } else {
        -((-n + 500) / 1000)
    }
}

/// A row in reference units: frames, then `x` (the reference's z, ahead), `y` (up) and the radius in tenths of its units, damage in
/// tenths of a percent, angle, BK, KS, priority (the reference's id) and hit group.
#[allow(clippy::too_many_arguments)]
fn ru(
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
    r(
        first,
        last,
        u(x),
        u(y),
        u(radius),
        damage,
        angle,
        bkb,
        kbg,
        priority,
        group,
    )
}

/// Points along a reference capsule from `a` to `b` (reference tenths), about `step` apart: the ends and evenly spaced points
/// between [D].
fn capsule(a: (i32, i32), b: (i32, i32), step: i32) -> Vec<(i32, i32)> {
    let span = (b.0 - a.0).abs().max((b.1 - a.1).abs());
    let n = (span / step.max(1)).max(1);
    (0..=n)
        .map(|i| (a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n))
        .collect()
}

/// Hitboxes along a reference capsule (the rest of the row as `ru`), each changed by `f`.
#[allow(clippy::too_many_arguments)]
fn along(
    first: u8,
    last: u8,
    a: (i32, i32),
    b: (i32, i32),
    radius: i32,
    damage: i32,
    angle: i16,
    bkb: i16,
    kbg: i16,
    priority: u8,
    group: u8,
    f: impl Fn(&mut Hitbox),
) -> Vec<Hitbox> {
    capsule(a, b, 30)
        .into_iter()
        .map(|(x, y)| {
            let mut h = row(&ru(
                first, last, x, y, radius, damage, angle, bkb, kbg, priority, group,
            ));
            f(&mut h);
            h
        })
        .collect()
}

/// One hitbox (as `ru`) changed by `f`.
#[allow(clippy::too_many_arguments)]
fn bx(
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
    f: impl Fn(&mut Hitbox),
) -> Hitbox {
    let mut h = row(&ru(
        first, last, x, y, radius, damage, angle, bkb, kbg, priority, group,
    ));
    f(&mut h);
    h
}

/// Shield damage as a percent of the damage for the reference's additive `sd` (both in tenths of a percent), rounded [D].
const fn shield(damage: i32, sd: i32) -> u16 {
    ((100 * (damage + sd) * 2 + damage) / (2 * damage)) as u16
}

/// Angle types (the reference's `af`): 3 forward, 4 backward, unset (0) by position.
fn fwd(h: &mut Hitbox) {
    h.angle_type = ANGLE_FORWARD;
}
fn fw(h: &mut Hitbox) {
    fwd(h);
}
fn back(h: &mut Hitbox) {
    h.angle_type = ANGLE_BACKWARD;
}
fn pos(h: &mut Hitbox) {
    h.angle_type = ANGLE_BY_POSITION;
}

/// A box out only in an attack begun without the partner (the reference's "without" tables).
fn plain(h: &mut Hitbox) {
    h.mode = MODE_PLAIN;
}

/// A box of the phantom's own out only in an attack begun with the partner (the reference's "with" tables, its awakened boxes).
fn awake(h: &mut Hitbox) {
    h.mode = MODE_POWERED;
}

/// One of the partner's own boxes (the reference's `part=1`): out only with the partner, its own hit (part 1), never clanking (the
/// partner cannot be hit or stopped: its boxes are transcendent here [R for most; E where the table does not say `clang=f`]).
fn partner(h: &mut Hitbox) {
    h.mode = MODE_POWERED;
    h.part = 1;
    h.clanks = false;
    h.rebounds = false;
}

/// The follow-up slots of the Dagger (all sixteen used).
pub mod slots {
    use super::MoveId;
    /// The hook in the air (with its once-per-air-time boost), the hook's pull and throw, the hook in the air without the boost.
    pub const HOOK_AIR: MoveId = MoveId::Ext0;
    pub const HOOK_PULL: MoveId = MoveId::Ext1;
    pub const HOOK_AIR_SPENT: MoveId = MoveId::Ext2;
    /// The gun's sideways dodges, forward and back.
    pub const GUN_DASH: MoveId = MoveId::Ext3;
    pub const GUN_BACK: MoveId = MoveId::Ext4;
    /// The gun's jump (grounded only).
    pub const GUN_JUMP: MoveId = MoveId::Ext5;
    /// The gun's aerial spiral (stick up) and shots below (stick down).
    pub const GUN_SPIRAL: MoveId = MoveId::Ext6;
    pub const GUN_BELOW: MoveId = MoveId::Ext7;
    /// The curse bolt's powered version.
    pub const BOLT_POWERED: MoveId = MoveId::Ext8;
    /// The guard held, its end lag and its backhand.
    pub const GUARD_HOLD: MoveId = MoveId::Ext9;
    pub const GUARD_END: MoveId = MoveId::Ext10;
    pub const BACKHAND: MoveId = MoveId::Ext11;
    /// The powered down special (the counter and reflector stance), its counterattack and its reflection.
    pub const STANCE: MoveId = MoveId::Ext12;
    pub const COUNTERATTACK: MoveId = MoveId::Ext13;
    pub const REFLECTION: MoveId = MoveId::Ext14;
    /// The powered up special (the wings).
    pub const WINGS: MoveId = MoveId::Ext15;
}
use slots::*;

/// The Dagger moveset (the phantom body): a short knife, kicks, a gun, a curse bolt, a guard that fills its gauge and the partner
/// that comes when the gauge is full. Every normal follows the reference frame for frame (the thirteenth moveset's `REFERENCE_DATA.md`
/// in `docs/movesets/dagger/`, version 13.0.1, read from ssbwiki.com's raw hitbox tables), with and without the partner: active frames,
/// FAF, landing lag and autocancel windows, smash charge frames; damage, angles and angle types, base and growth knockback, set
/// knockback and set weight, hitlag, SDI and shieldstun multipliers, shield damage, grounded- and aerial-only boxes, no-clank boxes, hit
/// groups, the partner's own boxes. Radii and offsets on the reference's root bone (`top`) are its values converted (x 1.25, rounded
/// half up to a tenth) [D]; boxes on other bones (the knife, arms, knees, legs, feet, the partner's hand) are placed on our body where
/// that bone is in the move's pose [E].
pub fn dagger() -> Weapon {
    let mut w = claws();
    // A knife held in the hand, bare feet and knees: the limb throwing an attack can be hit (the reference's arm and leg hurtboxes).
    w.limbs = true;
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    gun(&mut w);
    curse_bolt(&mut w);
    guard(&mut w);
    stance(&mut w);
    hook(&mut w);
    wings(&mut w);
    w
}

fn moved(faf: u8, landing: u8, ac_before: u8, ac_after: u8, boxes: Vec<Hitbox>) -> Move {
    let mut m = ref_move(faf, landing, ac_before, ac_after, &[]);
    m.hitboxes = boxes;
    m
}

fn jabs(w: &mut Weapon) {
    // Jab 1: an inward slash on frames 4-5, 2%, angle type 3, hitlag x1.6, 7.5 of the reference's units up: 361 (BK 32, KS 30, radius
    // 2.0) at 7.0 and 10.0 ahead; the far box (12.5 to 15.2 ahead) pulls a fighter in at 180 degrees, BK 32, KS 20 (fighters only;
    // the same box at 361 for objects is not modelled) [R]. The partner's swipe (0.7%, BK 33, KS 30, radius 2.5, hitlag x1.8, no clank):
    // 361 from 7.5 to 10.8 ahead, 180 from 10.8 to 14.7 [R]. FAF 24; continuable 7-24 into jab 2 [R].
    let fwd = |h: &mut Hitbox| {
        h.hitlag = 160;
        h.angle_type = ANGLE_FORWARD;
    };
    let mut b = vec![
        bx(4, 5, 70, 75, 20, 20, 361, 32, 30, 0, 0, fwd),
        bx(4, 5, 100, 75, 20, 20, 361, 32, 30, 1, 0, fwd),
    ];
    b.extend(along(
        4,
        5,
        (125, 75),
        (152, 75),
        20,
        20,
        180,
        32,
        20,
        2,
        0,
        fwd,
    ));
    let pal = |h: &mut Hitbox| {
        partner(h);
        h.hitlag = 180;
        h.angle_type = ANGLE_FORWARD;
    };
    b.extend(along(
        4,
        5,
        (75, 75),
        (108, 75),
        25,
        7,
        361,
        33,
        30,
        4,
        1,
        pal,
    ));
    b.extend(along(
        4,
        5,
        (108, 75),
        (147, 75),
        25,
        7,
        180,
        33,
        30,
        5,
        1,
        pal,
    ));
    let mut jab1 = moved(24, 0, 0, 255, b);
    chain(&mut jab1, MoveId::Jab2, 7);
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: a backhanded stab on frames 3-4, 1.5%, type 3, hitlag x1.6, 8.0 up: 361 (BK 25, KS 30, radius 2.0) at 10.0 and 13.0 to
    // 14.5 ahead; the far box (radius 2.5, 18.0 ahead) at 180, BK 30, KS 30 (fighters only) [R]. The partner (0.7%, BK 25, KS 30, radius
    // 2.5, hitlag x1.8): 361 from 10.5 to 15.0 ahead, 180 from 15.0 to 18.0 [R]. FAF 24; continuable 11-24 into jab 3 [R].
    let mut b = vec![bx(3, 4, 100, 80, 20, 15, 361, 25, 30, 0, 0, fwd)];
    b.extend(along(
        3,
        4,
        (130, 80),
        (145, 80),
        20,
        15,
        361,
        25,
        30,
        1,
        0,
        fwd,
    ));
    b.push(bx(3, 4, 180, 80, 25, 15, 180, 30, 30, 2, 0, fwd));
    b.extend(along(
        3,
        4,
        (105, 80),
        (150, 80),
        25,
        7,
        361,
        25,
        30,
        4,
        1,
        pal,
    ));
    b.extend(along(
        3,
        4,
        (150, 80),
        (180, 80),
        25,
        7,
        180,
        25,
        30,
        5,
        1,
        pal,
    ));
    let mut jab2 = moved(24, 0, 0, 255, b);
    chain(&mut jab2, MoveId::Jab3, 11);
    w.moves[MoveId::Jab2 as usize] = jab2;

    // Jab 3: an outward slash, 4%, 361 (type 3), BK 60, KS 80, 8.0 up: early on frame 3 the boxes at 10.0 and 13.5 to 14.5 ahead
    // (radius 2.0) and the knife (2.5); late on 4 those and a box at 15.0 ahead (never clanking) [R]; hitlag x1.6 [R]. With the partner
    // the phantom's boxes have hitlag x1.95 and the late box sits at 17.0; the partner's 3.1% (361, BK 65, KS 135, radius 2.5, hitlag x2)
    // from 11.0 to 14.0 ahead on 3-4, and at 17.0 on 4 [R]. The knife at 12.0 ahead, 9.0 up [E: the knife bone]. FAF 35.
    let mut b = Vec::new();
    for (mode, ff, far) in [(MODE_PLAIN, 160u16, 150), (MODE_POWERED, 195, 170)] {
        let m = move |h: &mut Hitbox| {
            h.mode = mode;
            h.hitlag = ff;
            h.angle_type = ANGLE_FORWARD;
        };
        b.push(bx(3, 4, 100, 80, 20, 40, 361, 60, 80, 0, 0, m));
        b.extend(along(
            3,
            4,
            (135, 80),
            (145, 80),
            20,
            40,
            361,
            60,
            80,
            1,
            0,
            m,
        ));
        b.push(bx(3, 4, 120, 90, 25, 40, 361, 60, 80, 2, 0, m));
        b.push(bx(4, 4, far, 80, 20, 40, 361, 60, 80, 3, 0, move |h| {
            m(h);
            h.clanks = false;
            h.rebounds = false;
        }));
    }
    let pal3 = |h: &mut Hitbox| {
        partner(h);
        h.hitlag = 200;
        h.angle_type = ANGLE_FORWARD;
    };
    b.extend(along(
        3,
        4,
        (110, 80),
        (140, 80),
        25,
        31,
        361,
        65,
        135,
        4,
        1,
        pal3,
    ));
    b.push(bx(4, 4, 170, 80, 25, 31, 361, 65, 135, 5, 1, pal3));
    w.moves[MoveId::Jab3 as usize] = moved(35, 0, 0, 255, b);
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a double thrust with the knife, aimed up, straight or down by the stick [R]. Hit 1 on frames 8-9, 3%, set weight,
    // BK 0, KS 100, type 3: the tip (id 1, radius 2.5) and the blade (id 2) with their own angles and set knockback by aim (up 180 / 10,
    // 35 / 25; straight 93 / 20, 40 / 20; down 93 / 10, 40 / 20) [R]. Hit 2 on 13-15, 5%, BK 45, type 3, the blade: up 45 degrees KS
    // 128, straight 35 KS 110, down 25 KS 105 [R]. With the partner hit 2 is on 13-19: the phantom's knife (radius 4.0, BK 60, KS 121 up
    // / 106 straight and down, hitlag x1.7, shieldstun x2) and the partner's hand on the same box (5%, the same knockback, no clank,
    // shieldstun x2) [R]. FAF 35. The knife reaches 10 to 17 of the reference's units ahead at chest height, raised or lowered with the
    // aim [E: the knife bone]; the forward step between the thrusts is not modelled [E: unpublished].
    let mut b = Vec::new();
    // (aim, tip, base, hit 1 tip angle / fkv, hit 1 blade angle / fkv, hit 2 angle, hit 2 KS, powered KS)
    for (aim, tip, base, t1, b1, a2, ks2, pks) in [
        (
            AIM_UP,
            (150, 130),
            (100, 95),
            (180, 10),
            (35, 25),
            45,
            128,
            121,
        ),
        (
            AIM_STRAIGHT,
            (170, 80),
            (100, 80),
            (93, 20),
            (40, 20),
            35,
            110,
            106,
        ),
        (
            AIM_DOWN,
            (160, 40),
            (100, 65),
            (93, 10),
            (40, 20),
            25,
            105,
            106,
        ),
    ] {
        let set = |fkv: u16| {
            move |h: &mut Hitbox| {
                h.aim = aim;
                h.set_weight = true;
                h.fixed_kb = fkv;
                h.angle_type = ANGLE_FORWARD;
            }
        };
        b.push(bx(
            8,
            9,
            tip.0,
            tip.1,
            25,
            30,
            t1.0,
            0,
            100,
            1,
            0,
            set(t1.1),
        ));
        b.extend(along(
            8,
            9,
            base,
            tip,
            25,
            30,
            b1.0,
            0,
            100,
            2,
            0,
            set(b1.1),
        ));
        let a = move |h: &mut Hitbox| {
            h.aim = aim;
            h.angle_type = ANGLE_FORWARD;
        };
        b.extend(along(
            13,
            15,
            base,
            tip,
            25,
            50,
            a2,
            45,
            ks2,
            2,
            1,
            move |h| {
                a(h);
                plain(h);
            },
        ));
        let pw = move |h: &mut Hitbox| {
            a(h);
            h.hitlag = 170;
            h.shieldstun = 200;
        };
        b.extend(along(
            13,
            19,
            base,
            tip,
            40,
            50,
            a2,
            60,
            pks,
            0,
            1,
            move |h| {
                pw(h);
                awake(h);
            },
        ));
        b.extend(along(
            13,
            19,
            base,
            tip,
            40,
            50,
            a2,
            60,
            pks,
            1,
            2,
            move |h| {
                pw(h);
                partner(h);
            },
        ));
    }
    w.moves[MoveId::FTilt as usize] = moved(35, 0, 0, 255, b);

    // Up tilt: the knife tossed spinning overhead, then caught (type 0 unless said) [R]. Frame 8: 4%, set weight, BK 0, KS 100, against
    // grounded fighters the box at the root (4.0 to 8.0 up, 4.0 to 7.5 ahead, radius 2.5) 120 degrees set knockback 150, the arm 105 /
    // 150, the knife 110 / 165 and 123 / 165 (radius 2.0, no clank), against airborne ones the arm 100 / 60, the knife 115 / 60 [R].
    // Frame 9: 4%, set weight, the knife high (radius 3.0, no clank): 130 / 20 from 23.0 to 20.0 up, 367 / 20 at 23.0 up, 133 / 0 from
    // 13.0 to 18.0 up and 7.5 ahead [R]. Frames 10-19: the spinning knife 1%, 367, BK 20, KS 20, set weight, radius 5.0, hitlag x0.3,
    // no clank, every 3 frames after its first hit (one hit group with frames 8-9: hits 2-4) [R]. Final hit on 20-23: 1%, 90 degrees, BK
    // 35, KS 270, radius 5.0, no clank [R]. With the partner: its electric hand (0.7%, 367, BK 50, KS 20, radius 6.0 at 25.0 up, hitlag
    // x0.3) on 11-19 every 3 frames, and the final hit is the knife's 3.9% (90, BK 50, KS 180, hitlag x1.3, electric) [R]. FAF 41. The
    // arm and knife over the head [E: the bones]: the knife at 22.0 up while it spins [E].
    let set = |fkv: u16, targets: u8, clanks: bool| {
        move |h: &mut Hitbox| {
            h.set_weight = true;
            h.fixed_kb = fkv;
            h.targets = targets;
            h.clanks = clanks;
            h.rebounds = clanks;
            h.angle_type = ANGLE_BY_POSITION;
        }
    };
    let mut b = along(
        8,
        8,
        (40, 40),
        (75, 80),
        25,
        40,
        120,
        0,
        100,
        0,
        0,
        set(150, TARGETS_GROUNDED, true),
    );
    b.push(bx(
        8,
        8,
        50,
        110,
        25,
        40,
        105,
        0,
        100,
        1,
        0,
        set(150, TARGETS_GROUNDED, true),
    ));
    b.push(bx(
        8,
        8,
        50,
        110,
        25,
        40,
        100,
        0,
        100,
        2,
        0,
        set(60, TARGETS_AERIAL, true),
    ));
    b.push(bx(
        8,
        8,
        60,
        140,
        20,
        40,
        110,
        0,
        100,
        3,
        0,
        set(165, TARGETS_GROUNDED, false),
    ));
    b.push(bx(
        8,
        8,
        60,
        140,
        20,
        40,
        115,
        0,
        100,
        4,
        0,
        set(60, TARGETS_AERIAL, false),
    ));
    b.push(bx(
        8,
        8,
        65,
        165,
        20,
        40,
        123,
        0,
        100,
        5,
        0,
        set(165, TARGETS_GROUNDED, false),
    ));
    b.push(bx(
        8,
        8,
        65,
        165,
        20,
        40,
        115,
        0,
        100,
        6,
        0,
        set(60, TARGETS_AERIAL, false),
    ));
    b.extend(along(
        9,
        9,
        (10, 230),
        (50, 200),
        30,
        40,
        130,
        0,
        100,
        3,
        0,
        set(20, TARGETS_ALL, false),
    ));
    b.push(bx(
        9,
        9,
        10,
        230,
        30,
        40,
        ANGLE_AUTOLINK,
        0,
        100,
        4,
        0,
        set(20, TARGETS_ALL, false),
    ));
    b.extend(along(
        9,
        9,
        (75, 130),
        (70, 180),
        30,
        40,
        133,
        0,
        100,
        5,
        0,
        set(0, TARGETS_ALL, false),
    ));
    b.push(bx(
        10,
        19,
        10,
        220,
        50,
        10,
        ANGLE_AUTOLINK,
        20,
        20,
        3,
        0,
        |h| {
            set(0, TARGETS_ALL, false)(h);
            h.hitlag = 30;
        },
    ));
    b.push(bx(
        11,
        19,
        0,
        250,
        60,
        7,
        ANGLE_AUTOLINK,
        50,
        20,
        0,
        1,
        |h| {
            partner(h);
            h.hitlag = 30;
            h.effect = EFFECT_ELECTRIC;
            h.angle_type = ANGLE_BY_POSITION;
        },
    ));
    b.push(bx(20, 23, 10, 220, 50, 10, 90, 35, 270, 3, 2, |h| {
        plain(h);
        h.clanks = false;
        h.rebounds = false;
        h.angle_type = ANGLE_BY_POSITION;
    }));
    b.push(bx(20, 23, 10, 220, 50, 39, 90, 50, 180, 3, 2, |h| {
        awake(h);
        h.hitlag = 130;
        h.effect = EFFECT_ELECTRIC;
        h.clanks = false;
        h.rebounds = false;
        h.angle_type = ANGLE_BY_POSITION;
    }));
    let mut utilt = moved(41, 0, 0, 255, b);
    // (The loop: re-armed every 3 frames from frame 11, the hit after one on frame 8; groups 0 and 1, not the final hit's.)
    utilt.rehit = Some((10, 3));
    utilt.rehit_groups = 0b11;
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a baseball slide, 6%, type 3, BK 75, KS 66 on the knee (radius 2.6): early 8-9 at 75 degrees (hitlag x0.75), mid 10-11
    // at 82, late 12-14 at 85 [R]. With the partner the phantom's box is on the root (radius 3.5, 3.2 up, 0 to 8.5 ahead; BK 55, hitlag
    // x1.5) at 75 / 82 / 85, and the partner slides alongside: 7%, 82 degrees, BK 70, KS 95 / 98 / 99 on the same box [R]. FAF 38. The
    // slide forward on frames 6-14 [E: the distance is not published; 14 of the reference's units]; the knee low in front [E].
    let mut b = Vec::new();
    for (first, last, angle, ff, pks) in [
        (8u8, 9u8, 75i16, 75u16, 95i16),
        (10, 11, 82, 100, 98),
        (12, 14, 85, 100, 99),
    ] {
        b.push(bx(
            first,
            last,
            85,
            25,
            26,
            60,
            angle,
            75,
            66,
            0,
            0,
            move |h| {
                plain(h);
                h.hitlag = ff;
                h.angle_type = ANGLE_FORWARD;
            },
        ));
        b.extend(along(
            first,
            last,
            (0, 32),
            (85, 32),
            35,
            60,
            angle,
            55,
            66,
            0,
            0,
            |h| {
                awake(h);
                h.hitlag = 150;
                h.angle_type = ANGLE_FORWARD;
            },
        ));
        b.extend(along(
            first,
            last,
            (0, 32),
            (85, 32),
            35,
            70,
            82,
            70,
            pks,
            1,
            1,
            |h| {
                partner(h);
                h.hitlag = 150;
                h.angle_type = ANGLE_FORWARD;
            },
        ));
    }
    let mut dtilt = moved(38, 0, 0, 255, b);
    dtilt.motion = vec![Motion {
        start: 5,
        end: 13,
        vx: Fx::from_ratio(14, 72),
        vy: Fx::ZERO,
    }];
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a roundhouse then a spinning side kick, type 3 [R]. Hit 1 on frames 6-7, 2%, BK 0, KS 95, set knockback: against
    // grounded fighters the root box (4.0 to 9.0 up, 4.0 to 5.0 ahead, radius 3.0) 35 / 70, the knee 40 / 80, the foot 50 / 50;
    // against airborne ones 25 / 40, 10 / 30, 15 / 25 [R]. Hit 2 on 15-21, 6%, 361, BK 60, KS 85, the leg and foot (radius 2.8) [R]. With
    // the partner hit 2 is the phantom's and the partner's kick on one box (6% each, 361, BK 85, KS 82, radius 5.0, hitlag x1.7): 8.0 up
    // and 7.0 to 12.0 ahead on 15-16, 10.0 up and 7.0 to 13.0 on 17-18, 11.0 up and 7.0 to 11.0 on 19-21 [R]. FAF 46. The knee and
    // foot ahead at hip height [E].
    let h1 = |fkv: u16, targets: u8| {
        move |h: &mut Hitbox| {
            h.fixed_kb = fkv;
            h.targets = targets;
            h.angle_type = ANGLE_FORWARD;
        }
    };
    let mut b = along(
        6,
        7,
        (40, 40),
        (50, 90),
        30,
        20,
        35,
        0,
        95,
        0,
        0,
        h1(70, TARGETS_GROUNDED),
    );
    b.push(bx(
        6,
        7,
        70,
        65,
        30,
        20,
        40,
        0,
        95,
        1,
        0,
        h1(80, TARGETS_GROUNDED),
    ));
    b.push(bx(
        6,
        7,
        100,
        70,
        30,
        20,
        50,
        0,
        95,
        2,
        0,
        h1(50, TARGETS_GROUNDED),
    ));
    b.extend(along(
        6,
        7,
        (40, 40),
        (50, 90),
        30,
        20,
        25,
        0,
        95,
        3,
        0,
        h1(40, TARGETS_AERIAL),
    ));
    b.push(bx(
        6,
        7,
        70,
        65,
        30,
        20,
        10,
        0,
        95,
        4,
        0,
        h1(30, TARGETS_AERIAL),
    ));
    b.push(bx(
        6,
        7,
        100,
        70,
        30,
        20,
        15,
        0,
        95,
        5,
        0,
        h1(25, TARGETS_AERIAL),
    ));
    b.push(bx(15, 21, 70, 70, 28, 60, 361, 60, 85, 0, 1, |h| {
        plain(h);
        h.angle_type = ANGLE_FORWARD;
    }));
    b.push(bx(15, 21, 100, 70, 28, 60, 361, 60, 85, 1, 1, |h| {
        plain(h);
        h.angle_type = ANGLE_FORWARD;
    }));
    for (first, last, y, to) in [
        (15u8, 16u8, 80, 120),
        (17, 18, 100, 130),
        (19, 21, 110, 110),
    ] {
        b.extend(along(
            first,
            last,
            (70, y),
            (to, y),
            50,
            60,
            361,
            85,
            82,
            0,
            1,
            |h| {
                awake(h);
                h.hitlag = 170;
                h.angle_type = ANGLE_FORWARD;
            },
        ));
        b.extend(along(
            first,
            last,
            (70, y),
            (to, y),
            50,
            60,
            361,
            85,
            82,
            1,
            2,
            |h| {
                partner(h);
                h.hitlag = 170;
                h.angle_type = ANGLE_FORWARD;
            },
        ));
    }
    w.moves[MoveId::DashAttack as usize] = moved(46, 0, 0, 255, b);
}

fn smashes(w: &mut Weapon) {
    // Forward smash: a two-handed downward slash, 14%, 361 (type 3), BK 40, KS 98, hitlag x0.8 [R]. Charges on frame 5. Early on frame
    // 16: radius 2.0 at 12.0 ahead from 10.0 down to 6.0 up, radius 4.0 at 9.0 ahead from 9.5 to 8.0; late on 17-18 the same lower
    // (8.0 to 6.0, 9.5 to 6.0) [R]. With the partner (hitlag x1.2): the phantom's boxes at 15.5 ahead (radius 3.0, 14.0 down to 8.0) and
    // 10.0 (radius 5.0, 8.5 to 11.5) on 16, 14.0 to 7.5 and 7.0 to 11.5 on 17-18, the same at BK 0 on 19 (14%); the partner's slash on the
    // same boxes on 16-19, 8%, BK 50, KS 151 [R]. FAF 48. (The step forward is not modelled [E: unpublished].)
    let mut b = Vec::new();
    let pl = |h: &mut Hitbox| {
        plain(h);
        h.hitlag = 80;
        fwd(h);
    };
    b.extend(along(
        16,
        16,
        (120, 100),
        (120, 60),
        20,
        140,
        361,
        40,
        98,
        0,
        0,
        pl,
    ));
    b.extend(along(
        16,
        16,
        (90, 95),
        (90, 80),
        40,
        140,
        361,
        40,
        98,
        1,
        0,
        pl,
    ));
    b.extend(along(
        17,
        18,
        (120, 80),
        (120, 60),
        20,
        140,
        361,
        40,
        98,
        0,
        0,
        pl,
    ));
    b.extend(along(
        17,
        18,
        (90, 95),
        (90, 60),
        40,
        140,
        361,
        40,
        98,
        1,
        0,
        pl,
    ));
    for (first, last, a, b2, bk) in [
        (
            16u8,
            16u8,
            ((155, 140), (155, 80)),
            ((100, 85), (100, 115)),
            40i16,
        ),
        (17, 18, ((155, 140), (155, 75)), ((100, 70), (100, 115)), 40),
        (19, 19, ((155, 140), (155, 75)), ((100, 70), (100, 115)), 0),
    ] {
        let pw = |h: &mut Hitbox| {
            awake(h);
            h.hitlag = 120;
            fwd(h);
        };
        b.extend(along(first, last, a.0, a.1, 30, 140, 361, bk, 98, 0, 0, pw));
        b.extend(along(
            first, last, b2.0, b2.1, 50, 140, 361, bk, 98, 1, 0, pw,
        ));
        let ar = |h: &mut Hitbox| {
            partner(h);
            h.hitlag = 120;
            fwd(h);
        };
        b.extend(along(first, last, a.0, a.1, 30, 80, 361, 50, 151, 2, 1, ar));
        b.extend(along(
            first, last, b2.0, b2.1, 50, 80, 361, 50, 151, 3, 1, ar,
        ));
    }
    let mut fsmash = moved(48, 0, 0, 255, b);
    fsmash.charge_at = Some(4);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: a knife uppercut, 12%, 83 degrees (type 0), BK 43, KS 100 [R]. Charges on frame 4. Early on 10 the knife (radius 3.0),
    // the arm (2.5) and the root box (radius 3.5, 7.0 to 12.0 up, 3.0 to 6.0 ahead); late on 11-14 the knife and arm [R]. With the
    // partner (hitlag x1.5): early 10 the knife (3.5), arm and root box; mid 11 the knife and arm; late 12-13 those and a high box
    // (radius 5.0, 28.0 down to 16.0 up, 1.0 to 9.5 ahead, no clank); latest 14 that box at 28.0 up, 1.0 ahead; the partner's arm on the
    // same boxes on 10-14, 5%, BK 59, KS 181, no clank [R]. FAF 52. The knife and arm overhead [E]; the arm's intangibility with the
    // partner on 10-14 is not modelled (an E19 window cannot follow the partner) [E].
    let mut b = vec![
        bx(10, 14, 40, 180, 30, 120, 83, 43, 100, 0, 0, |h| {
            plain(h);
            pos(h);
        }),
        bx(10, 14, 30, 140, 25, 120, 83, 43, 100, 1, 0, |h| {
            plain(h);
            pos(h);
        }),
    ];
    b.extend(along(
        10,
        10,
        (30, 70),
        (60, 120),
        35,
        120,
        83,
        43,
        100,
        2,
        0,
        |h| {
            plain(h);
            pos(h);
        },
    ));
    let pw = |h: &mut Hitbox| {
        awake(h);
        h.hitlag = 150;
        pos(h);
    };
    let ar = |h: &mut Hitbox| {
        partner(h);
        h.hitlag = 150;
        pos(h);
    };
    for (part, damage, bk, ks, group, priority) in [
        (pw as fn(&mut Hitbox), 120, 43i16, 100i16, 0u8, 0u8),
        (ar, 50, 59, 181, 1, 3),
    ] {
        b.push(bx(
            10, 14, 40, 190, 35, damage, 83, bk, ks, priority, group, part,
        ));
        b.push(bx(
            10,
            14,
            30,
            140,
            25,
            damage,
            83,
            bk,
            ks,
            priority + 1,
            group,
            part,
        ));
        b.extend(along(
            10,
            10,
            (30, 70),
            (60, 120),
            35,
            damage,
            83,
            bk,
            ks,
            priority + 2,
            group,
            part,
        ));
        b.extend(along(
            12,
            13,
            (10, 280),
            (95, 160),
            50,
            damage,
            83,
            bk,
            ks,
            priority + 2,
            group,
            move |h| {
                part(h);
                h.clanks = false;
                h.rebounds = false;
            },
        ));
        b.push(bx(
            14,
            14,
            10,
            280,
            50,
            damage,
            83,
            bk,
            ks,
            priority + 2,
            group,
            move |h| {
                part(h);
                h.clanks = false;
                h.rebounds = false;
            },
        ));
    }
    let mut usmash = moved(52, 0, 0, 255, b);
    usmash.charge_at = Some(3);
    // With the partner the arm is intangible on 10-14 [R] (sim 123: a window for the powered mode only).
    usmash.part_intangible = vec![PartIntangible {
        start: 9,
        end: 13,
        parts: PART_LIMB,
        mode: MODE_POWERED,
    }];
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: the knife swept low in front then behind, type 0 [R]. Charges on frame 5. Hit 1 on 12-13: 12%, 32 degrees, BK 30, KS
    // 93, the knife (two boxes, radius 2.0) and the arm (2.5), hitlag x0.8; hit 2 on 16-17 behind: 12%, 30 degrees, BK 35, KS 93, no
    // clank [R]. With the partner (hitlag x1.3): hit 1 at BK 64, KS 84 with a root box (radius 3.5, 6.5 to 4.0 up, 6.5 to 15.5 ahead, no
    // clank) and the partner's kick on it (6%, 38 degrees, BK 44, KS 158); hit 2 as without, with the root box behind (6.0 to 3.0 up,
    // 8.0 to 15.0 behind) and the partner's kick there (6%, 30, BK 40, KS 158) [R]. FAF 52 (ultimateframedata.com, dragdown.wiki;
    // ssbwiki.com reads 46: conflict 2). The knife and arm low in front and behind [E].
    let mut b = Vec::new();
    for (mode, ff, bk1, ks1) in [
        (MODE_PLAIN, 80u16, 30i16, 93i16),
        (MODE_POWERED, 130, 64, 84),
    ] {
        let h1 = move |h: &mut Hitbox| {
            h.mode = mode;
            h.hitlag = ff;
            pos(h);
        };
        b.push(bx(12, 13, 100, 20, 20, 120, 32, bk1, ks1, 0, 0, h1));
        b.push(bx(12, 13, 125, 25, 20, 120, 32, bk1, ks1, 1, 0, h1));
        b.push(bx(12, 13, 60, 40, 25, 120, 32, bk1, ks1, 2, 0, h1));
        let h2 = move |h: &mut Hitbox| {
            h1(h);
            h.clanks = false;
            h.rebounds = false;
        };
        b.push(bx(16, 17, -100, 20, 20, 120, 30, 35, 93, 0, 1, h2));
        b.push(bx(16, 17, -125, 25, 20, 120, 30, 35, 93, 1, 1, h2));
        b.push(bx(16, 17, -60, 40, 25, 120, 30, 35, 93, 2, 1, h2));
    }
    let pw = |h: &mut Hitbox| {
        awake(h);
        h.hitlag = 130;
        h.clanks = false;
        h.rebounds = false;
        pos(h);
    };
    let ar = |h: &mut Hitbox| {
        partner(h);
        h.hitlag = 130;
        pos(h);
    };
    b.extend(along(
        12,
        13,
        (65, 65),
        (155, 40),
        35,
        120,
        32,
        64,
        84,
        4,
        0,
        pw,
    ));
    b.extend(along(
        12,
        13,
        (65, 65),
        (155, 40),
        35,
        60,
        38,
        44,
        158,
        5,
        2,
        ar,
    ));
    b.extend(along(
        16,
        17,
        (-80, 60),
        (-150, 30),
        35,
        120,
        30,
        35,
        93,
        4,
        1,
        pw,
    ));
    b.extend(along(
        16,
        17,
        (-80, 60),
        (-150, 30),
        35,
        60,
        30,
        40,
        158,
        5,
        3,
        ar,
    ));
    let mut dsmash = moved(52, 0, 0, 255, b);
    dsmash.charge_at = Some(4);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: the knife swung in a circle through two turns on frames 12-27, 7%, 46 degrees (type 0), BK 35, KS 93, radius 2.8
    // (one hit) [R]; with the partner its hand sweeps the other half of the circle, 4%, 46, BK 50, KS 139, radius 2.8 [R]. Landing lag
    // 8, autocancels on frames 1-4 and from 48, FAF 55. The knife's path: a circle 9 of the reference's units round the body's middle (8
    // up), a turn every 8 frames, the partner's hand opposite [E: the bones].
    let mut b = Vec::new();
    // (Eight points round the circle, a frame each, starting in front.)
    const RING: [(i32, i32); 8] = [
        (90, 80),
        (64, 144),
        (0, 170),
        (-64, 144),
        (-90, 80),
        (-64, 16),
        (0, -10),
        (64, 16),
    ];
    for k in 0..16u8 {
        let (x, y) = RING[usize::from(k % 8)];
        let frame = 12 + k;
        b.push(bx(frame, frame, x, y, 28, 70, 46, 35, 93, 0, 0, pos));
        let (px, py) = RING[usize::from((k + 4) % 8)];
        b.push(bx(frame, frame, px, py, 28, 40, 46, 50, 139, 3, 1, |h| {
            partner(h);
            pos(h);
        }));
    }
    w.moves[MoveId::NAir as usize] = moved(55, 8, 4, 48, b);

    // Forward air: a hook kick then a roundhouse, type 3 [R]. Hit 1, 2%, set weight: frame 7 the leg, knee and a root box (8.5 up, 10.0
    // ahead), radius 4.0, against airborne fighters 367 (BK 70, KS 25; the knee KS 5), against grounded ones 76 / 80 (BK 73, KS 50);
    // frame 8 the leg and knee only [R]. Hit 2 on 12-14: 5%, 361, BK 46, KS 125, the knee and leg (radius 3.5) [R]. With the partner hit
    // 2 is the phantom's box at 13.5 up, 5.0 to 7.5 ahead (radius 6.0, 5%, 34 degrees, BK 48, KS 148, hitlag x1.9) and the partner's kick
    // there and higher (radius 6.0 / 7.0 at 18.0 to 17.0 up, 8%, 34, BK 58, KS 98, hitlag x1.6) [R]. Landing lag 12, autocancels on 1-3
    // and from 42, FAF 48. The leg 7.0 ahead at 8.0 up, the knee 4.5 ahead at 9.0 [E].
    let air = |h: &mut Hitbox| {
        h.set_weight = true;
        h.targets = TARGETS_AERIAL;
        fw(h);
    };
    let ground = |h: &mut Hitbox| {
        h.set_weight = true;
        h.targets = TARGETS_GROUNDED;
        fw(h);
    };
    let mut b = vec![
        bx(7, 8, 70, 80, 40, 20, ANGLE_AUTOLINK, 70, 25, 0, 0, air),
        bx(7, 8, 70, 80, 40, 20, 76, 73, 50, 1, 0, ground),
        bx(7, 8, 45, 90, 40, 20, ANGLE_AUTOLINK, 70, 5, 2, 0, air),
        bx(7, 8, 45, 90, 40, 20, 80, 73, 50, 3, 0, ground),
        bx(7, 7, 100, 85, 40, 20, ANGLE_AUTOLINK, 70, 25, 4, 0, air),
        bx(7, 7, 100, 85, 40, 20, 76, 73, 50, 5, 0, ground),
        bx(12, 14, 80, 85, 35, 50, 361, 46, 125, 0, 1, |h| {
            plain(h);
            fw(h);
        }),
        bx(12, 14, 50, 90, 35, 50, 361, 46, 125, 1, 1, |h| {
            plain(h);
            fw(h);
        }),
    ];
    b.extend(along(
        12,
        14,
        (50, 135),
        (75, 135),
        60,
        50,
        34,
        48,
        148,
        0,
        1,
        |h| {
            awake(h);
            h.hitlag = 190;
            fw(h);
        },
    ));
    let ar = |h: &mut Hitbox| {
        partner(h);
        h.hitlag = 160;
        fw(h);
    };
    b.extend(along(
        12,
        14,
        (50, 135),
        (75, 135),
        60,
        80,
        34,
        58,
        98,
        1,
        2,
        ar,
    ));
    b.extend(along(
        12,
        14,
        (50, 180),
        (65, 170),
        70,
        80,
        34,
        58,
        98,
        2,
        2,
        ar,
    ));
    w.moves[MoveId::FAir as usize] = moved(48, 12, 3, 42, b);

    // Back air: an upward swipe behind, 9%, 43 degrees backward (type 4), BK 54, KS 90, hitlag x0.8 [R]. Early on frame 7: radius 2.5
    // from 3.3 to 10.7 up, 6.0 to 8.4 behind, and from 4.7 to 11.1 up, 11.1 to 14.2 behind; radius 3.0 from 4.0 to 8.5 up, 3.0 behind
    // to 5.0 ahead. Late on 8: the knife's two boxes high behind (radius 2.5) and radius 3.5 from 4.5 to 7.5 up, 6.5 behind to 5.5 ahead
    // [R; E: the knife's place]. With the partner on 7-8: the phantom's boxes (9%, 35 degrees, BK 51, KS 102, radius 5.0, hitlag x1.3)
    // from 10.5 to 15.0 up at 12.5 behind and from 7.5 to 5.5 up, 10.0 to 8.5 behind; the partner's chop on those (7%, BK 58, KS 114,
    // hitlag x1.4) and from 16.5 to 20.5 up, 12.0 to 9.0 behind (7%, BK 76, KS 98) [R]. Landing lag 9, autocancels on 1-3 and from 30,
    // FAF 32.
    let pl = |h: &mut Hitbox| {
        plain(h);
        h.hitlag = 80;
        back(h);
    };
    let mut b = along(7, 7, (-60, 33), (-84, 107), 25, 90, 43, 54, 90, 0, 0, pl);
    b.extend(along(
        7,
        7,
        (-111, 47),
        (-142, 111),
        25,
        90,
        43,
        54,
        90,
        1,
        0,
        pl,
    ));
    b.extend(along(
        7,
        7,
        (-30, 40),
        (50, 85),
        30,
        90,
        43,
        54,
        90,
        2,
        0,
        pl,
    ));
    b.extend(along(
        8,
        8,
        (-100, 100),
        (-130, 135),
        25,
        90,
        43,
        54,
        90,
        0,
        0,
        pl,
    ));
    b.extend(along(
        8,
        8,
        (-65, 45),
        (55, 75),
        35,
        90,
        43,
        54,
        90,
        2,
        0,
        pl,
    ));
    let pw = |h: &mut Hitbox| {
        awake(h);
        h.hitlag = 130;
        back(h);
    };
    b.extend(along(
        7,
        8,
        (-125, 105),
        (-125, 150),
        50,
        90,
        35,
        51,
        102,
        0,
        0,
        pw,
    ));
    b.extend(along(
        7,
        8,
        (-100, 75),
        (-85, 55),
        50,
        90,
        35,
        51,
        102,
        1,
        0,
        pw,
    ));
    let ar = |h: &mut Hitbox| {
        partner(h);
        h.hitlag = 140;
        back(h);
    };
    b.extend(along(
        7,
        8,
        (-100, 75),
        (-85, 55),
        50,
        70,
        35,
        58,
        114,
        2,
        1,
        ar,
    ));
    b.extend(along(
        7,
        8,
        (-125, 105),
        (-125, 150),
        50,
        70,
        35,
        58,
        114,
        3,
        1,
        ar,
    ));
    b.extend(along(
        7,
        8,
        (-120, 165),
        (-90, 205),
        50,
        70,
        35,
        76,
        98,
        4,
        1,
        ar,
    ));
    w.moves[MoveId::BAir as usize] = moved(32, 9, 3, 30, b);

    // Up air: a corkscrew kick overhead, type 3 [R]. Frames 5-18, every 4 frames (hits 1-4): the knee (radius 4.0) 0.75%, hitlag x0.7,
    // SDI x1.2: against grounded fighters 80 degrees, BK 20, KS 80; against airborne ones 367, BK 30, KS 100 [R]. Final hit on 20-21: the
    // knee 3%, 80 degrees (type 0), BK 50, KS 155, radius 4.5 [R]; with the partner the knee is radius 5.0 (hitlag x1.7) and the
    // partner's hand swings overhead (7%, 80, BK 50, KS 126, radius 5.0, 20.0 to 24.0 up, 1.5 to 0 ahead, hitlag x1.7) [R]. Landing lag
    // 14, autocancels on 1-4 and from 40, FAF 40. The knee overhead [E: 17 of the reference's units up].
    let multi = |targets: u8| {
        move |h: &mut Hitbox| {
            h.hitlag = 70;
            h.sdi = 120;
            h.targets = targets;
            fw(h);
        }
    };
    let mut b = vec![
        bx(
            5,
            18,
            15,
            170,
            40,
            7,
            80,
            20,
            80,
            0,
            0,
            multi(TARGETS_GROUNDED),
        ),
        bx(
            5,
            18,
            15,
            170,
            40,
            7,
            ANGLE_AUTOLINK,
            30,
            100,
            1,
            0,
            multi(TARGETS_AERIAL),
        ),
        bx(20, 21, 15, 170, 45, 30, 80, 50, 155, 0, 1, |h| {
            plain(h);
            pos(h);
        }),
        bx(20, 21, 15, 170, 50, 30, 80, 50, 155, 0, 1, |h| {
            awake(h);
            h.hitlag = 170;
            pos(h);
        }),
    ];
    // (0.75% is not a whole tenth: the damage is set exactly below.)
    for h in b.iter_mut().take(2) {
        h.damage = Fx::from_ratio(3, 4);
    }
    b.extend(along(
        20,
        21,
        (15, 200),
        (0, 240),
        50,
        70,
        80,
        50,
        126,
        1,
        2,
        |h| {
            partner(h);
            h.hitlag = 170;
            pos(h);
        },
    ));
    let mut uair = moved(40, 14, 4, 40, b);
    uair.rehit = Some((8, 4));
    uair.rehit_groups = 0b1;
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: the knife swung below, 8%, 38 degrees (type 3), BK 40, KS 100, hitlag x0.8, on 13-16 (two boxes, radius 2.5 and 3.0)
    // [R]. With the partner: early on 13-14 the knife at KS 60; clean on 15-16 the knife at BK 50, KS 50, hitlag x1.5, no clank, and the
    // partner's stomp: 8%, 270 degrees (a meteor), BK 35, KS 96, radius 5.0 from 2.0 up to 4.0 below, 1.0 ahead, hitlag x1.5, no clank
    // [R]. Landing lag 11, autocancels on 1-3 and from 40, FAF 47. The knife below and ahead [E].
    let mut b = Vec::new();
    for (first, last, mode, ff, bk, ks, clanks) in [
        (13u8, 16u8, MODE_PLAIN, 80u16, 40i16, 100i16, true),
        (13, 14, MODE_POWERED, 80, 40, 60, true),
        (15, 16, MODE_POWERED, 150, 50, 50, false),
    ] {
        let set = move |h: &mut Hitbox| {
            h.mode = mode;
            h.hitlag = ff;
            h.clanks = clanks;
            h.rebounds = clanks;
            fw(h);
        };
        b.push(bx(first, last, 50, -10, 25, 80, 38, bk, ks, 1, 0, set));
        b.push(bx(first, last, 30, 20, 30, 80, 38, bk, ks, 2, 0, set));
    }
    b.extend(along(
        15,
        16,
        (10, 20),
        (10, -40),
        50,
        80,
        270,
        35,
        96,
        3,
        1,
        |h| {
            partner(h);
            h.hitlag = 150;
            fw(h);
        },
    ));
    w.moves[MoveId::DAir as usize] = moved(47, 11, 3, 40, b);
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: one arm reaching ahead, the reference's grounded-target box (id 0) swept along its capsule [R]: standing on frames 6-7
    // (radius 3.3, 4.5 to 9.0 ahead, 8.1 up), FAF 35; from a dash on 8-9 (2.6, 4.0 to 10.6, 6.6 up), FAF 43; pivot on 9-10 (3.3, 7.0 to
    // 16.7, 6.6 up; written in front, since the engine turns a pivot grab round), FAF 38. (The aerial-target box is not modelled.)
    let grab = |faf: u8, first: u8, last: u8, y: i32, from: i32, to: i32, radius: i32| {
        with_kind(
            moved(
                faf,
                0,
                0,
                255,
                along(
                    first,
                    last,
                    (from, y),
                    (to, y),
                    radius,
                    0,
                    361,
                    0,
                    0,
                    0,
                    0,
                    |_| {},
                ),
            ),
            HIT_GRAB,
        )
    };
    w.moves[MoveId::Grab as usize] = grab(35, 6, 7, 81, 45, 90, 33);
    w.moves[MoveId::DashGrab as usize] = grab(43, 8, 9, 66, 40, 106, 26);
    w.moves[MoveId::PivotGrab as usize] = grab(38, 9, 10, 66, 70, 167, 33);

    // Pummel: a knee, 1.5%, set knockback 30 (KS 100), 361 (type 3), hitlag x2.2, radius 5.0 at 10.0 ahead and 10.0 up, never clanks
    // [R]. The reference's hit on frame 1 is frame 2 here with the same 6 frames after it: FAF 8 [D] (the reference's 7).
    let mut pummel = with_kind(
        moved(
            8,
            0,
            0,
            255,
            vec![bx(2, 2, 100, 100, 50, 15, 361, 0, 100, 0, 0, |h| {
                h.fixed_kb = 30;
                h.hitlag = 220;
                h.angle_type = ANGLE_FORWARD;
                h.clanks = false;
                h.rebounds = false;
            })],
        ),
        HIT_PUMMEL,
    );
    pummel.hitboxes[0].kind = HIT_PUMMEL;
    w.moves[MoveId::Pummel as usize] = pummel;

    // Throws, no hitlag (the reference's 0.0) [R]; the partner does not change them [R] (the thrown fighter's 3% hit on bystanders and
    // the thrower's invincibility on the frames up to the release are not modelled: house limits). Forward: on frame 9, 8% at 45
    // degrees, BK 65, KS 65, FAF 30. Back: on 14, 10% at 131 (sent behind and up), BK 70, KS 66, FAF 40. Up: on 16, 7% at 90, BK 70,
    // KS 72, FAF 38. Down: on 23, 7% at 79, BK 55, KS 95, FAF 41 [R].
    let throw = |faf: u8, frame: u8, damage: i32, angle: i16, bkb: i16, kbg: i16| {
        let mut m = with_kind(
            ref_move(
                faf,
                0,
                0,
                255,
                &[r(frame, frame, 13, 11, 10, damage, angle, bkb, kbg, 0, 0)],
            ),
            HIT_THROW,
        );
        m.hitboxes[0].hitlag = 0;
        m
    };
    w.moves[MoveId::FThrow as usize] = throw(30, 9, 80, 45, 65, 65);
    w.moves[MoveId::BThrow as usize] = throw(40, 14, 100, 131, 70, 66);
    w.moves[MoveId::UThrow as usize] = throw(38, 16, 70, 90, 70, 72);
    w.moves[MoveId::DThrow as usize] = throw(41, 23, 70, 79, 55, 95);
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: frames 24-26, 9%, 45 degrees (type 3), BK 90, KS 20, shield damage +1 (111 [D]), never clanks; its swept box (16.0
    // ahead to 1.0, 5.0 up, radius 5.0) as circles [D]; intangible through frame 26 (on the first grab: 1-21 and 1-13 on the second and
    // third, the shared 80% / 50%), FAF 56 [R].
    let mut ledge = moved(
        56,
        0,
        0,
        255,
        along(24, 26, (160, 50), (10, 50), 50, 90, 45, 90, 20, 0, 0, |h| {
            h.shield_damage = shield(90, 10);
            h.angle_type = ANGLE_FORWARD;
            h.clanks = false;
            h.rebounds = false;
        }),
    );
    ledge.intangible = 26;
    w.moves[MoveId::LedgeAttack as usize] = ledge;

    // Get-up attack (lying face up, the reference's "back" floor attack): behind on frames 15-16 (15.0 to 10.0 behind), then in front
    // on 24-25 (14.0 to 9.0 ahead), 5.0 up, radius 5.0; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8 (214 [D]), never clanks;
    // intangible through frame 25, FAF 46 [R]. (The face-down and trip get-up attacks have no slot of their own.)
    let set = |h: &mut Hitbox| {
        h.shield_damage = shield(70, 80);
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
        h.rebounds = false;
    };
    let mut b = along(
        15,
        16,
        (-150, 50),
        (-100, 50),
        50,
        70,
        48,
        80,
        48,
        0,
        0,
        set,
    );
    b.extend(along(
        24,
        25,
        (140, 50),
        (90, 50),
        50,
        70,
        48,
        80,
        48,
        0,
        1,
        set,
    ));
    let mut getup = moved(46, 0, 0, 255, b);
    getup.intangible = 25;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}

// ---- The gun (neutral special) ----------------------------------------------------------------------------------------------------

/// The shot's bands in the reference's tenths from the muzzle (4.0 ahead): close to 20.0 of its units from the feet, middle 23.0 to
/// 44.0, far 47.0 to 72.0 [E: the reference publishes the
/// three bands' damages and knockbacks but no distances; "roughly half of the stage's length" overall].
const CLOSE: (i32, i32) = (0, 160);
const MID: (i32, i32) = (190, 400);
const FAR: (i32, i32) = (430, 680);
/// The shot's height (chest, 9.5 of the reference's units) and its boxes' radius (2.5) [E].
const SHOT_Y: i32 = 95;
const SHOT_R: i32 = 25;

/// One shot (a bullet: not a projectile, boxes reaching from the muzzle in a line, the reference's indirect hit) on reference frame
/// `frame`, along the direction (`dx`, `dy`, in hundredths: 100 / 0 straight ahead) from (`ox`, `oy`), its three bands (damage in
/// tenths, BK, KS, hitlag percent each), in hit group `group`, with `f` for the rest.
#[allow(clippy::too_many_arguments)]
fn shot(
    frame: u8,
    (ox, oy): (i32, i32),
    (dx, dy): (i32, i32),
    bands: [(i32, i16, i16, u16); 3],
    angle: i16,
    group: u8,
    f: impl Fn(&mut Hitbox) + Copy,
) -> Vec<Hitbox> {
    let mut out = Vec::new();
    for (k, ((from, to), (damage, bk, ks, lag))) in
        [CLOSE, MID, FAR].into_iter().zip(bands).enumerate()
    {
        let a = (ox + dx * from / 100, oy + dy * from / 100);
        let b = (ox + dx * to / 100, oy + dy * to / 100);
        out.extend(along(
            frame,
            frame,
            a,
            b,
            SHOT_R,
            damage,
            angle,
            bk,
            ks,
            k as u8,
            group,
            move |h| {
                h.hitlag = lag;
                h.angle_type = ANGLE_FORWARD;
                f(h);
            },
        ));
    }
    out
}

/// The plain shot's bands (the reference's close / middle / far: 5% BK 5 KS 75, 3% BK 1 KS 50, 1% BK 0 KS 0) and their hitlag (the
/// close one's 6 frames [D: x0.7], none further out) [R].
const PLAIN_BANDS: [(i32, i16, i16, u16); 3] = [(50, 5, 75, 70), (30, 1, 50, 0), (10, 0, 0, 0)];
/// The partner's first bullet (6% BK 60 KS 50, 3%, 1%) and its two follow-ups (2%, 1.5%, 1%; BK 1, KS 0 [E: the follow-ups'
/// knockback is not given apart]) [R].
const POWER_BANDS: [(i32, i16, i16, u16); 3] = [(60, 60, 50, 70), (30, 1, 0, 0), (10, 0, 0, 0)];
const FOLLOW_BANDS: [(i32, i16, i16, u16); 3] = [(20, 1, 0, 0), (15, 1, 0, 0), (10, 0, 0, 0)];

/// The gun's aerial stall (the reference: a shot stalls the phantom in the air): its vertical speed stopped on starting, then gravity
/// 0.03 and a top fall of 0.8 a frame (the reference's units) [E: not published].
fn gun_air(m: &mut Move) {
    m.start_speed_y = 0;
    m.air_gravity = Some(FighterParams::su(30));
    m.air_max_fall = Some(FighterParams::su(800));
}

/// Neutral special, the gun (the reference's, ssbwiki.com and ultimateframedata.com, 13.0.1; `REFERENCE_DATA.md` "Gun").
///
/// * A shot on frame 12, ground or air, FAF 37 [R: ultimateframedata.com's total 36; dragdown.wiki reads FAF 36: conflict 3]. From
///   frame 26 a new press of special fires again (the next shot 25 frames after the last, the move starting over); held, it fires again
///   when the move ends (slower, as the reference's "holding fires, but not as fast") [E: the held rhythm]. From frame 26 the stick
///   pushed sideways dodges while firing (forward or back: at most four in one string), and on the ground jump or the stick up jumps
///   while firing below; in the air from frame 14 the stick down fires below and up spins [R].
/// * The shots are boxes in a line from the muzzle (they cannot be reflected or blocked by a passive shield and go through every
///   fighter in the line) with three bands by distance: close 5% (BK 5, KS 75), middle 3% (BK 1, KS 50), far 1% (no knockback) [R].
/// * With the partner (powered): three bullets a shot on frames 12, 18 and 24 (the first 6% / 3% / 1%, BK 60, KS 50 close; the
///   follow-ups 2% / 1.5% / 1%), FAF 42, the next shot or dodge from frame 34, the aerial ones from 15 [R].
fn gun(w: &mut Weapon) {
    let muzzle = (40, SHOT_Y);
    let mut b = shot(12, muzzle, (100, 0), PLAIN_BANDS, 361, 0, plain);
    b.extend(shot(12, muzzle, (100, 0), POWER_BANDS, 361, 0, awake));
    b.extend(shot(18, muzzle, (100, 0), FOLLOW_BANDS, 361, 1, awake));
    b.extend(shot(24, muzzle, (100, 0), FOLLOW_BANDS, 361, 2, awake));
    let mut nb = moved(42, 0, 0, 255, b);
    gun_air(&mut nb);
    nb.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_gun.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = nb;

    // The dodges (ext3 forward, ext4 back): a roll-like step while firing straight ahead (the reference's "gun dash"), ground or air.
    // Forward: intangible 4-13, the shot on 20 (6% BK 35 KS 67, 4% BK 4 KS 45, 2%), FAF 40; with the partner shots on 20, 26 and 32 (7% BK
    // 50 KS 50 close, then the follow-ups), FAF 46 [R]. Back: intangible 5-14, the shot on 22 (5% BK 5 KS 75, 3%, 1%), FAF 46; with the
    // partner 22, 28, 34, FAF 50 [R]. The step: 1.52 / 1.216 of the reference's units a frame (the parameter file's
    // `shoot_dash_speed_f` / `_b`) on frames 3-16 / 4-17 [R speeds; E frames]. Staled as rolls are [R]: counted, intangible later and
    // played slower (sim 126).
    let dash_bands = [(60, 35, 67, 70), (40, 4, 45, 0), (20, 0, 0, 0)];
    let dash_power = [(70, 50, 50, 70), (40, 4, 0, 0), (15, 0, 0, 0)];
    let mut b = shot(20, muzzle, (100, 0), dash_bands, 361, 0, plain);
    b.extend(shot(20, muzzle, (100, 0), dash_power, 361, 0, awake));
    b.extend(shot(26, muzzle, (100, 0), FOLLOW_BANDS, 361, 1, awake));
    b.extend(shot(32, muzzle, (100, 0), FOLLOW_BANDS, 361, 2, awake));
    let mut dash = moved(46, 0, 0, 255, b);
    dash.motion = vec![Motion {
        start: 2,
        end: 15,
        vx: FighterParams::su(1520),
        vy: Fx::ZERO,
    }];
    dash.intangible_from = 3;
    dash.intangible_from_air = 3;
    dash.intangible = 13;
    // (Staled like a roll: sim 123; and stretched in time as a forward roll is: sim 126, owner decision 85.)
    dash.dodge_stale = true;
    dash.stale_stretch = crate::moves::STRETCH_ROLL;
    dash.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_gun_dash.script"),
    ));
    w.moves[GUN_DASH as usize] = dash;

    let mut b = shot(22, muzzle, (100, 0), PLAIN_BANDS, 361, 0, plain);
    b.extend(shot(22, muzzle, (100, 0), PLAIN_BANDS, 361, 0, awake));
    b.extend(shot(28, muzzle, (100, 0), FOLLOW_BANDS, 361, 1, awake));
    b.extend(shot(34, muzzle, (100, 0), FOLLOW_BANDS, 361, 2, awake));
    let mut back = moved(50, 0, 0, 255, b);
    back.motion = vec![Motion {
        start: 3,
        end: 16,
        vx: -FighterParams::su(1216),
        vy: Fx::ZERO,
    }];
    back.intangible_from = 4;
    back.intangible_from_air = 4;
    back.intangible = 14;
    // (Stretched as a back roll is, the step going backward [E: which of the ruleset's rates].)
    back.dodge_stale = true;
    back.stale_stretch = crate::moves::STRETCH_BACK_ROLL;
    back.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_gun_back.script"),
    ));
    w.moves[GUN_BACK as usize] = back;

    // The jump (ext5, from the ground only): a jump up and forward while firing below on frames 10, 17, 24 and 31 (3% close, 0.8% far;
    // BK 10 / 20, KS 70 / 120), intangible 1-6, FAF 60, landing lag 14 [R]; with the partner shots on 13, 20 and 27 too (1% / 0.5%) [R].
    // The reference's angle for the first three is 365 (sim 123: half the phantom's momentum, `ANGLE_HALF_LINK`), the fourth 90 [E: its
    // several angles]; the partner's three follow-ups 365 too [E]. The jump: 2.2 up a frame for 12 frames, 0.5 forward, then the body's fall [E: not published].
    let below = (0, 0);
    let down = (25, -100);
    let jump_bands = |angle_hit: bool| {
        if angle_hit {
            [(30, 10, 70, 100), (8, 20, 120, 0), (8, 20, 120, 0)]
        } else {
            [(10, 0, 50, 100), (5, 0, 50, 0), (5, 0, 50, 0)]
        }
    };
    let mut b = Vec::new();
    for (k, frame) in [10u8, 17, 24, 31].into_iter().enumerate() {
        let angle = if k == 3 { 90 } else { ANGLE_HALF_LINK };
        b.extend(shot(
            frame,
            below,
            down,
            jump_bands(true),
            angle,
            k as u8,
            |_| {},
        ));
    }
    for (k, frame) in [13u8, 20, 27].into_iter().enumerate() {
        b.extend(shot(
            frame,
            below,
            down,
            jump_bands(false),
            ANGLE_HALF_LINK,
            4 + k as u8,
            awake,
        ));
    }
    let mut jump = moved(60, 14, 0, 255, b);
    jump.intangible = 6;
    jump.motion = vec![Motion {
        start: 0,
        end: 11,
        vx: FighterParams::su(500),
        vy: FighterParams::su(2200),
    }];
    jump.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_gun_jump.script"),
    ));
    w.moves[GUN_JUMP as usize] = jump;

    // The spiral (ext6, air, stick up): twelve shots round the body on frames 12, 15, 18, 24, 27, 30, 36, 39, 42, 48, 51, 54, 3% each
    // (BK 5, KS 75 close; no knockback far), FAF 70, landing lag 20 [R]; with the partner 4.5% each [R]. A shot every 30 degrees from
    // straight ahead round over the top [E: the directions]; the reach 30 of the reference's units (close to 12) [E]. It falls slowly [E].
    let mut b = Vec::new();
    let frames = [12u8, 15, 18, 24, 27, 30, 36, 39, 42, 48, 51, 54];
    // (Every 30 degrees from straight ahead, in hundredths: cos and sin.)
    const DIRS: [(i32, i32); 12] = [
        (100, 0),
        (87, 50),
        (50, 87),
        (0, 100),
        (-50, 87),
        (-87, 50),
        (-100, 0),
        (-87, -50),
        (-50, -87),
        (0, -100),
        (50, -87),
        (87, -50),
    ];
    for (frame, dir) in frames.into_iter().zip(DIRS) {
        for (mode, damage) in [(MODE_PLAIN, 30), (MODE_POWERED, 45)] {
            let set = move |h: &mut Hitbox| {
                h.mode = mode;
                h.angle_type = ANGLE_BY_POSITION;
            };
            let a = (dir.0 * 4 / 100, 80 + dir.1 * 4 / 100);
            let mid = (dir.0 * 120 / 100, 80 + dir.1 * 120 / 100);
            let end = (dir.0 * 300 / 100, 80 + dir.1 * 300 / 100);
            b.extend(along(
                frame, frame, a, mid, SHOT_R, damage, 361, 5, 75, 0, 0, set,
            ));
            b.extend(along(
                frame, frame, mid, end, SHOT_R, damage, 361, 0, 0, 1, 0, set,
            ));
        }
    }
    let mut spiral = moved(70, 20, 0, 255, b);
    // (One hit group re-armed every 3 frames from the first shot: each shot, 3 or 6 frames after the last, can hit again.)
    spiral.rehit = Some((11, 3));
    gun_air(&mut spiral);
    w.moves[GUN_SPIRAL as usize] = spiral;

    // Shots below (ext7, air, stick down): straight down on frames 7, 21, 35 and 44, 4% close (50 degrees, BK 25, KS 80), 1.2% far (45,
    // BK 29, KS 0); FAF 49, landing lag 10; held, it goes on with more shots (the move again) [R]. With the partner, two more on 14 and
    // 28 (1.6% / 0.8%) [R]. The reach 40 of the reference's units straight down, close to 16 [E].
    let mut b = Vec::new();
    let below_bands = |main: bool| {
        if main {
            [(40, 25, 80, 100), (12, 29, 0, 0), (12, 29, 0, 0)]
        } else {
            [(16, 25, 80, 100), (8, 29, 0, 0), (8, 29, 0, 0)]
        }
    };
    for (k, frame) in [7u8, 21, 35, 44].into_iter().enumerate() {
        b.extend(shot(
            frame,
            (0, -10),
            (0, -100),
            below_bands(true),
            50,
            k as u8,
            |_| {},
        ));
    }
    for (k, frame) in [14u8, 28].into_iter().enumerate() {
        b.extend(shot(
            frame,
            (0, -10),
            (0, -100),
            below_bands(false),
            50,
            4 + k as u8,
            awake,
        ));
    }
    let mut below = moved(49, 10, 0, 255, b);
    gun_air(&mut below);
    below.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_gun_below.script"),
    ));
    w.moves[GUN_BELOW as usize] = below;
}

// ---- The curse bolt (side special) ------------------------------------------------------------------------------------------------

/// The bolt and its burst (the reference's, ssbwiki.com 13.0.1; `REFERENCE_DATA.md` "Side special").
///
/// * Plain: a snap of the fingers; the bolt leaves on frame 16 and lasts to 42 (27 frames), FAF 53 (since 11.0.0) [R]. It hits for 1%
///   (361 by its travel, BK 35, KS 20, radius 1.2, hitlag x0.6) and erupts a frame later where it hit: 2%, 75 degrees forward, BK 20,
///   KS 210, radius 8.5 from 1.5 to 3.5 up, hitlag x0.6, for 19 frames; it does not erupt on a shield [R]. Either hit curses the fighter:
///   1% every 45 frames for 361 frames (8 ticks) [R]. Its path: straight ahead on the ground, 30 degrees down from the air, 1.6 of the
///   reference's units a frame [E: not published]. Each use stalls the phantom a little in the air [E].
/// * Powered (ext8): FAF 58; the bolt lasts frames 16-32, faster [E: 3.0 a frame], 1% at 90 degrees (BK 35, KS 20, radius 1.5); it
///   erupts on a fighter or the ground: three hits of 0.5% every 5 frames (367, BK 5, KS 100, radius 4.0 from 1.5 to 8.0 up, hitlag
///   x0.3; sent backward on its first 5 frames, then forward [E: the split]) and a last 2.5% (73 degrees, BK 105, KS 105, hitlag x1) on
///   16-27; the curse is 1.5% every 40 frames for 321 (8 ticks) [R].
fn curse_bolt(w: &mut Weapon) {
    let base = |frame: u8, life: u8, hit: Hitbox, burst: Burst| ProjectileSpawn {
        frame,
        x: Fx::from_ratio(u(60), 10),
        y: Fx::from_ratio(u(70), 10),
        speed: Fx::ZERO,
        life,
        hitbox: hit,
        end_damage: hit.damage,
        arc: ProjectileArc::NONE,
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: Some(burst),
        charged: None,
        tool: Tool::NONE,
    };
    // The plain bolt.
    let bolt = bx(1, 1, 0, 0, 12, 10, 361, 35, 20, 0, 0, |h| {
        h.hitlag = 60;
        h.angle_type = ANGLE_BY_TRAVEL;
    });
    let erupt = bx(1, 1, 0, 25, 85, 20, 75, 20, 210, 0, 0, |h| {
        h.hitlag = 60;
        h.angle_type = ANGLE_FORWARD;
        h.half_height = Fx::from_ratio(u(10), 10);
    });
    let burst = Burst {
        life: 19,
        rehit: 0,
        on_wall: false,
        y: Fx::ZERO,
        phases: phase_list(&[(1, erupt)]),
        on_ground: false,
        on_expire: false,
        keep_hits: false,
        on_hit: true,
        on_shield: false,
    };
    let mut mv = ref_move(53, 0, 0, 255, &[]);
    // (Fired by the script on frame 16, so the spec's own frame never comes.)
    mv.projectile = Some(base(255, 27, bolt, burst));
    mv.curse = Some(Curse {
        damage: Fx::ONE,
        every: 45,
        frames: 361,
    });
    mv.start_speed_y = 0;
    mv.air_gravity = Some(FighterParams::su(40));
    mv.air_max_fall = Some(FighterParams::su(1000));
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_bolt.script"),
    ));
    let mut side = mv;

    // The powered bolt.
    let bolt = bx(1, 1, 0, 0, 15, 10, 90, 35, 20, 0, 0, |h| {
        h.hitlag = 80;
        h.angle_type = ANGLE_BY_TRAVEL;
    });
    let multi = |angle_type: u8| {
        let mut h = bx(1, 1, 0, 48, 40, 5, ANGLE_AUTOLINK, 5, 100, 0, 0, |h| {
            h.hitlag = 30;
            h.half_height = Fx::from_ratio(u(33), 10);
        });
        h.angle_type = angle_type;
        h
    };
    let last = bx(1, 1, 0, 48, 40, 25, 73, 105, 105, 0, 1, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.half_height = Fx::from_ratio(u(33), 10);
    });
    let burst = Burst {
        life: 27,
        rehit: 5,
        on_wall: false,
        y: Fx::ZERO,
        phases: phase_list(&[
            (1, multi(ANGLE_BACKWARD)),
            (6, multi(ANGLE_FORWARD)),
            (16, last),
        ]),
        on_ground: true,
        on_expire: false,
        keep_hits: false,
        on_hit: true,
        on_shield: false,
    };
    let mut mv = ref_move(58, 0, 0, 255, &[]);
    let mut spec = base(255, 17, bolt, burst);
    // (A landing bursts it: one bounce allowed, but any fall is too fast to bounce, so it bursts where it touched.)
    spec.arc = ProjectileArc {
        bounces: 1,
        bounce_max_fall: Fx::from_raw(1),
        ..ProjectileArc::NONE
    };
    mv.projectile = Some(spec);
    mv.curse = Some(Curse {
        damage: Fx::from_ratio(3, 2),
        every: 40,
        frames: 321,
    });
    mv.start_speed_y = 0;
    mv.air_gravity = Some(FighterParams::su(40));
    mv.air_max_fall = Some(FighterParams::su(1000));
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_bolt_powered.script"),
    ));
    w.moves[BOLT_POWERED as usize] = mv;

    side.limit_version = Some(BOLT_POWERED as u8);
    w.moves[MoveId::SideSpecial as usize] = side;
}

// ---- The guard (down special) and the partner's stance ----------------------------------------------------------------------------

/// Down special, the guard (the reference's, ssbwiki.com and ultimateframedata.com, 13.0.1; `REFERENCE_DATA.md` "Down special").
///
/// * Super armor from frame 3 to 19 (E1); held, the guard goes on (ext9) up to frame 120; FAF 53 unheld, and 33 frames of lag after a
///   longer guard (ext10) [R]. A hit on it deals 0.4 of its damage (`Move::armor_damage`, E55), with x1.5 hitlag, and fills the gauge
///   by 1.3 x 6.7368 a percent taken (`Move::gauge_gain`, E53: 28.6% fills it from empty) [R].
/// * Let go within 33 frames of a hit (the `withstood` register), the backhand (ext11): 2.4%, 50 degrees, set knockback 82 (KS 100),
///   radius 12.0 at 10.0 up, 3.0 behind to 3.0 ahead, hitlag x1.25, no SDI, no clank, on frames 8-9; intangible 1-14, FAF 31 [R].
fn guard(w: &mut Weapon) {
    let armor = |end: u8| {
        Some(Armor {
            start: 2,
            end,
            kind: ARMOR_SUPER,
            threshold: Fx::ZERO,
            subtract: false,
            min_kb: Fx::ZERO,
        })
    };
    let rules = |m: &mut Move| {
        m.armor_damage = 400;
        m.armor_hitlag = 150;
        m.gauge_gain = 6737;
    };
    let mut g = ref_move(53, 0, 0, 255, &[]);
    g.armor = armor(18);
    rules(&mut g);
    g.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_guard.script"),
    ));
    g.limit_version = Some(STANCE as u8);
    w.moves[MoveId::DownSpecial as usize] = g;

    // (Held: one frame of armor at a time, the move going round while special is held.)
    let mut hold = ref_move(4, 0, 0, 255, &[]);
    hold.armor = Some(Armor {
        start: 0,
        end: 2,
        kind: ARMOR_SUPER,
        threshold: Fx::ZERO,
        subtract: false,
        min_kb: Fx::ZERO,
    });
    rules(&mut hold);
    hold.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_guard_hold.script"),
    ));
    w.moves[GUARD_HOLD as usize] = hold;

    let mut end = ref_move(34, 0, 0, 255, &[]);
    end.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_guard_end.script"),
    ));
    w.moves[GUARD_END as usize] = end;

    let mut backhand = moved(
        31,
        0,
        0,
        255,
        along(
            8,
            9,
            (-30, 100),
            (30, 100),
            120,
            24,
            50,
            0,
            100,
            0,
            0,
            |h| {
                h.fixed_kb = 82;
                h.hitlag = 125;
                h.sdi = 0;
                h.clanks = false;
                h.rebounds = false;
                h.angle_type = ANGLE_BY_POSITION;
            },
        ),
    );
    backhand.intangible = 14;
    backhand.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/dagger_backhand.script"),
    ));
    w.moves[BACKHAND as usize] = backhand;
}

/// The powered down special: the counter and the reflector (the reference's, ssbwiki.com, ultimateframedata.com and dragdown.wiki,
/// 13.0.1; `REFERENCE_DATA.md` "Down special, powered").
///
/// * The stance (ext12): intangible 3-4; counters melee hits on frames 4-31 (E10c `melee_only`) at 1.6x, at least 12% and at most 50%,
///   caught on its bubble (radius 10.0 at 10.0 up [E: not published]); on the same frames a projectile search box (radius 12.0 at 11.0
///   up [E]: the reference's is "larger, notably higher") starts the reflection (E10c `detect_then`). FAF 57 (since 11.0.0) [R].
/// * The counterattack (ext13): the hit on frames 5-7, the caught damage (361, BK 80, KS 51, radius 20.0 at 10.5 up, 5.0 behind to 10.0
///   ahead, hitlag x0.75, no clank); intangible 1-8, FAF 39 [R].
/// * The reflection (ext14): reflects on frames 1-29 (radius 12.0 at 10.0 up [E]) at 1.6x damage and 1.9x speed, shots up to 50% [R];
///   FAF 42 [R]. (Its arm's intangibility is not modelled.)
fn stance(w: &mut Weapon) {
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    let mut stance = ref_move(57, 0, 0, 255, &[]);
    stance.intangible_from = 2;
    stance.intangible = 4;
    stance.counter = Some(Counter {
        start: 3,
        end: 30,
        then: COUNTERATTACK as u8,
        percent: 160,
        min_damage: Fx::from_int(12),
        max_damage: Fx::from_int(50),
        melee_only: true,
        bubble: Some(CounterBubble {
            x: Fx::ZERO,
            y: tenths(u(100)),
            half_height: Fx::ZERO,
            radius: tenths(u(100)),
        }),
        break_above: Fx::ZERO,
        break_move: 0,
        still: false,
    });
    stance.reflector = Some(Reflector {
        damage_percent: 160,
        speed_percent: 190,
        max_damage: Fx::from_int(50),
        detect_start: 3,
        detect_end: 30,
        detect_x: Fx::ZERO,
        detect_y: tenths(u(110)),
        detect_radius: tenths(u(120)),
        detect_then: Some(REFLECTION as u8),
        ..Reflector::NONE
    });
    w.moves[STANCE as usize] = stance;

    let mut strike = moved(
        39,
        0,
        0,
        255,
        along(
            5,
            7,
            (-50, 105),
            (100, 105),
            200,
            120,
            361,
            80,
            51,
            0,
            0,
            |h| {
                h.hitlag = 75;
                h.clanks = false;
                h.rebounds = false;
                h.angle_type = ANGLE_FORWARD;
            },
        ),
    );
    strike.counter_strike = true;
    strike.intangible = 8;
    w.moves[COUNTERATTACK as usize] = strike;

    let mut reflection = ref_move(42, 0, 0, 255, &[]);
    reflection.reflector = Some(Reflector {
        start: 0,
        end: 28,
        x: Fx::ZERO,
        y: tenths(u(100)),
        radius: tenths(u(120)),
        damage_percent: 160,
        speed_percent: 190,
        max_damage: Fx::from_int(50),
        ..Reflector::NONE
    });
    w.moves[REFLECTION as usize] = reflection;
}

// ---- The hook (the plain up special) -----------------------------------------------------------------------------------------------

/// The hook's line, in the reference's tenths: from 3.2 ahead and 22.0 up to 10.6 ahead and 38.0 up (the aerial box id 1 on `top`, the
/// root at the feet) [R]; the hook's head (the reference's `throw` bone, the far end of its id 0 capsule, 16.4 long) at its far end [E].
const HOOK_NEAR: (i32, i32) = (32, 220);
const HOOK_FAR: (i32, i32) = (106, 380);

/// The plain up special, the hook (the reference's, ssbwiki.com 13.0.1 raw hitbox tables and ultimateframedata.com) [R unless marked]:
///
/// * **On the ground** (the up special): a grab box on frame 20 (the rope, radius 3.0, along the hook's line, and the head, radius 5.5)
///   and 21-26 (the head), FAF 60. It catches fighters in the air or on the ground (a command grab: the reference's "can grab
///   opponents, dragging them towards" the fighter) but not one in hitstun, and a raised shield or a reflector stops it (`blockable`,
///   E56: "can be shielded despite its grab properties"; the move goes on to its end [E: what the stop leaves is not published]). A
///   catch goes on as the pull (ext1): the caught fighter held just in front [E] and thrown on frame 6, 11% at 78 degrees, BK 75, KS
///   40, FAF 30.
/// * **In the air** (ext0, its `air_version`): no grab; a hit on 20 (the head, radius 2.7, and the line, radius 2.7) and 21-25 (the
///   head): 5%, 55 degrees, BK 76, KS 56, shield damage +1, hitlag x1.2, clanks but does not rebound; FAF 45; no special fall ("does
///   not induce special fall"). The first in an air time gives a small rise (the reference's "slight vertical boost": 1.5 of its units
///   a frame on frame 5 [E: not published]) and marks the air time (mark 0); a later one (ext2, by the flag version) has none.
/// * **The tether** (E56, the air versions): from frame 12 ("tethers on frame 12", ultimateframedata.com) to the head's last frame 25
///   [E], a free ledge in reach is caught: up to 40.0 of the reference's units above the feet (the line's far end and its radius) and
///   16.0 in front [E: "a large ledge grab range", "very high vertical reach", "low horizontal range"]; the fighter hangs 8 frames
///   (losing 0.24 of its speed a frame: the reference's hang air resistance, `air_resist_hang` 0.24 since 4.0.0 [R; its reading E]),
///   then is reeled to the ledge in 12 [E]; three catches before landing [R]; the stick down lets go [R].
fn hook(w: &mut Weapon) {
    let line =
        |radius: i32, damage: i32, angle: i16, bkb: i16, kbg: i16, f: &dyn Fn(&mut Hitbox)| {
            along(
                20, 20, HOOK_NEAR, HOOK_FAR, radius, damage, angle, bkb, kbg, 1, 0, f,
            )
        };
    // On the ground: the grab.
    let grab = |h: &mut Hitbox| {
        h.kind = HIT_GRAB;
        h.command = true;
        h.blockable = true;
    };
    let mut b = line(30, 0, 361, 0, 0, &grab);
    b.push(bx(
        20, 26, HOOK_FAR.0, HOOK_FAR.1, 55, 0, 361, 0, 0, 0, 0, grab,
    ));
    let mut ground = moved(60, 0, 0, 255, b);
    ground.catch_move = Some(HOOK_PULL as u8);
    // A shield or a reflector stopping the hook ends the move 20 frames later (sim 126, owner decision 84: the reference "cancels" it
    // on contact [R]; the frames after the stop are not published [E]).
    ground.shield_end = 20;
    ground.air_version = Some(HOOK_AIR as u8);
    ground.limit_version = w.moves[MoveId::UpSpecial as usize].limit_version;
    w.moves[MoveId::UpSpecial as usize] = ground;

    // The pull and throw.
    let mut pull = with_kind(
        ref_move(30, 0, 0, 255, &[r(6, 6, 13, 11, 10, 110, 78, 75, 40, 0, 0)]),
        HIT_THROW,
    );
    pull.hold_at = Some((FighterParams::su(9000), Fx::ZERO));
    w.moves[HOOK_PULL as usize] = pull;

    // In the air: the hit and the tether.
    let hit = |h: &mut Hitbox| {
        h.shield_damage = shield(50, 10);
        h.hitlag = 120;
        h.rebounds = false;
    };
    let air = |boost: bool| {
        let mut b = line(27, 50, 55, 76, 56, &hit);
        b.push(bx(
            20, 25, HOOK_FAR.0, HOOK_FAR.1, 27, 50, 55, 76, 56, 0, 0, hit,
        ));
        // (Every frame lands with the body's own landing lag [E: none published].)
        let mut m = moved(45, 0, 255, 255, b);
        m.tether = Some(crate::moves::Tether {
            from: 11,
            to: 24,
            ahead: FighterParams::su(16_000),
            behind: Fx::ZERO,
            up: FighterParams::su(40_000),
            down: Fx::ZERO,
            hang: 8,
            reel: 12,
            drag: 240,
            limit: 3,
            // (A press of special while hanging reels in at once: owner decision 86, a house addition.)
            reel_press: true,
        });
        if boost {
            m.motion = vec![Motion {
                start: 4,
                end: 4,
                vx: Fx::ZERO,
                vy: FighterParams::su(1500),
            }];
            m.motion_drift = 100;
            m.script = Some(script(
                Kind::Fighter,
                include_str!("scripts/dagger_hook.script"),
            ));
            m.flag_versions = vec![FlagVersion {
                flags: 0,
                marks: 1,
                slot: Some(HOOK_AIR_SPENT as u8),
            }];
        }
        m
    };
    w.moves[HOOK_AIR as usize] = air(true);
    w.moves[HOOK_AIR_SPENT as usize] = air(false);
}

/// The powered up special, the wings (ext15; the reference's, ssbwiki.com 13.0.1): a flight up with no hitbox, intangible 1-25 from the
/// ground and 3-25 from the air, 98 frames, then the special fall with 30 frames of landing lag [R]; the stick angles it a little before
/// it rises [R]. The rise: 2.0 of the reference's units a frame for frames 6-45, gently slowing [E: not published; "higher than most up
/// specials"]; ledges in front from frame 10 [E].
fn wings(w: &mut Weapon) {
    let mut m = ref_move(99, 30, 0, 255, &[]);
    m.intangible = 25;
    m.intangible_from_air = 2;
    m.motion = vec![
        Motion {
            start: 5,
            end: 29,
            vx: Fx::ZERO,
            vy: FighterParams::su(2200),
        },
        Motion {
            start: 30,
            end: 44,
            vx: Fx::ZERO,
            vy: FighterParams::su(1400),
        },
    ];
    m.steer_frame = 5;
    m.steer_degrees = 25;
    m.steer_stick = 30;
    m.helpless_after = true;
    m.fall_landing_lag = 30;
    m.ledge_front_from = 9;
    w.moves[WINGS as usize] = m;
    w.moves[MoveId::UpSpecial as usize].limit_version = Some(WINGS as u8);
}
