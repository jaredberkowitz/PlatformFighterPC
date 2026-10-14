//! The Nodachi moveset (the exile body), modelled mechanically on a reference character: a very long blade, a wing that comes when
//! the exile is in danger (E57, the body's `content::Wing`), the blade dash and its eight-slash form (up special) and the flare in
//! three charged levels (neutral special) (phase 1, sim 125). A child module of `moves` (it uses that file's private row builders).
//! Data and sources: `docs/movesets/nodachi/REFERENCE_DATA.md`, read from ssbwiki.com's raw hitbox tables (13.0.1),
//! ultimateframedata.com, kuroganehammer.com and dragdown.wiki.
//!
//! **The wing** powers every attack begun while it is out (`Fighter::move_powered`): the three smash attacks' damage armor (20%, from
//! their charge frame to their hit) is out only then (`Move::armor_mode` powered).

use super::*;

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

/// One hitbox in reference tenths: frames, `x` (the reference's z, ahead) and `y` (up), radius, damage in tenths of a percent,
/// angle, BK, KS, priority (the reference's id) and hit group, changed by `f`.
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
    let mut h = row(&r(
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
    ));
    f(&mut h);
    h
}

/// Points along a reference capsule from `a` to `b` (reference tenths), about 3 units apart: the ends and evenly spaced points
/// between [D].
fn capsule(a: (i32, i32), b: (i32, i32)) -> Vec<(i32, i32)> {
    let span = (b.0 - a.0).abs().max((b.1 - a.1).abs());
    let n = (span / 30).max(1);
    (0..=n)
        .map(|i| (a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n))
        .collect()
}

/// Hitboxes along a reference capsule (the rest of the row as `bx`).
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
    capsule(a, b)
        .into_iter()
        .map(|(x, y)| {
            bx(
                first, last, x, y, radius, damage, angle, bkb, kbg, priority, group, &f,
            )
        })
        .collect()
}

/// A point `along` reference tenths down the blade from its hilt at `hilt`, the blade pointing `dir` (a direction in hundredths:
/// (100, 0) straight ahead, (0, 100) straight up). The reference hangs its blade boxes on the sword's own bone; where that bone is in
/// each pose is an estimate [E].
fn blade(hilt: (i32, i32), dir: (i32, i32), along: i32) -> (i32, i32) {
    (hilt.0 + along * dir.0 / 100, hilt.1 + along * dir.1 / 100)
}

/// Shield damage as a percent of the damage for the reference's additive `sd` (both in tenths of a percent), rounded [D].
const fn shield(damage: i32, sd: i32) -> u16 {
    ((100 * (damage + sd) * 2 + damage) / (2 * damage)) as u16
}

/// Angle types (the reference's `af`): 3 forward, 4 backward, unset (0) by position.
fn fwd(h: &mut Hitbox) {
    h.angle_type = ANGLE_FORWARD;
}
fn back(h: &mut Hitbox) {
    h.angle_type = ANGLE_BACKWARD;
}
fn pos(h: &mut Hitbox) {
    h.angle_type = ANGLE_BY_POSITION;
}
/// The reference's `clang=f rebound=f`: the hit never clanks.
fn no_clank(h: &mut Hitbox) {
    h.clanks = false;
    h.rebounds = false;
}

fn moved(faf: u8, landing: u8, ac_before: u8, ac_after: u8, boxes: Vec<Hitbox>) -> Move {
    let mut m = ref_move(faf, landing, ac_before, ac_after, &[]);
    m.hitboxes = boxes;
    m
}

/// The wing's armor on a smash attack (E57): damage armor of 20% on reference frames `first..=last`, only in an attack begun with
/// the wing out [R].
fn wing_armor(m: &mut Move, first: u8, last: u8) {
    m.armor = Some(Armor {
        start: first - 1,
        end: last - 1,
        kind: ARMOR_DAMAGE,
        threshold: Fx::from_int(20),
        subtract: false,
        min_kb: Fx::ZERO,
    });
    m.armor_mode = MODE_POWERED;
}

/// The follow-up slots of the Nodachi.
pub mod slots {
    use super::MoveId;
    /// The down air's landing hit.
    pub const DAIR_LANDING: MoveId = MoveId::Ext0;
}
use slots::*;

/// The Nodachi moveset (the exile body): a very long blade, kicks and dark magic, a wing when in danger. Every normal follows the
/// reference frame for frame (`docs/movesets/nodachi/REFERENCE_DATA.md`, version 13.0.1, read from ssbwiki.com's raw hitbox tables):
/// active frames, FAF, landing lag and autocancel windows, smash charge frames; damage, angles and angle types, base and growth
/// knockback, set knockback, hitlag and hitstun modifiers, shield damage, grounded- and aerial-only boxes, no-clank boxes, hit groups
/// and the reference's box ids as priorities. Radii and offsets on the reference's root bone (`top`) are its values converted (x 1.25,
/// rounded half up to a tenth) [D]; boxes on the blade's bone are placed along the blade where it is in the move's pose [E].
pub fn nodachi() -> Weapon {
    let mut w = claws();
    // A blade held in the hand and bare feet: the limb throwing an attack can be hit (the reference's arm and leg hurtboxes).
    w.limbs = true;
    for m in w.moves.iter_mut() {
        *m = Move::empty();
    }
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    w
}

fn jabs(w: &mut Weapon) {
    // Jab 1: a kick on frames 5-7, 2.5%, type 3, hitlag x1.4, hitstun +2, 8.0 up: 361 (BK 25, KS 10) at 6.0 (radius 2.5) and 9.25
    // (2.7) ahead; the far box at 12.5 (2.7) at 180 degrees, KS 15 (fighters only; its 361 twin for objects is not modelled) [R]. FAF
    // 29; continuable 10-28 on a press or held after a hit, 19 held without a hit [R].
    let j = |h: &mut Hitbox| {
        h.hitlag = 140;
        h.hitstun_add = 2;
        fwd(h);
    };
    let mut jab1 = moved(
        29,
        0,
        0,
        255,
        vec![
            bx(5, 7, 60, 80, 25, 25, 361, 25, 10, 0, 0, j),
            bx(5, 7, 93, 80, 27, 25, 361, 25, 10, 1, 0, j),
            bx(5, 7, 125, 80, 27, 25, 180, 25, 15, 2, 0, j),
        ],
    );
    chain(&mut jab1, MoveId::Jab2, 10);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/nodachi_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: a second kick on frames 5-7, 2%, type 3, hitlag x1.3, hitstun +3, 8.0 up: BK 35, KS 20 at 7.5 (radius 3.2) and 11.0 (3.7);
    // BK 25, KS 15 at 14.5 (3.7) [R]. FAF 34; continuable 10-30 on a press or held after a hit, 19 held without a hit [R].
    let j = |h: &mut Hitbox| {
        h.hitlag = 130;
        h.hitstun_add = 3;
        fwd(h);
    };
    let mut jab2 = moved(
        34,
        0,
        0,
        255,
        vec![
            bx(5, 7, 75, 80, 32, 20, 361, 35, 20, 0, 0, j),
            bx(5, 7, 110, 80, 37, 20, 361, 35, 20, 1, 0, j),
            bx(5, 7, 145, 80, 37, 20, 361, 25, 15, 2, 0, j),
        ],
    );
    chain(&mut jab2, MoveId::Jab3, 10);
    jab2.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/nodachi_jab_2.script"),
    ));
    w.moves[MoveId::Jab2 as usize] = jab2;

    // Jab 3: a pommel thrust on frames 7-9, 4%, 361 (type 3), BK 70, KS 80, hitlag x1.5, 7.5 up: at 5.8 (radius 3.8) and 11.4 (4.8)
    // ahead [R]. FAF 36.
    let j = |h: &mut Hitbox| {
        h.hitlag = 150;
        fwd(h);
    };
    w.moves[MoveId::Jab3 as usize] = moved(
        36,
        0,
        0,
        255,
        vec![
            bx(7, 9, 58, 75, 38, 40, 361, 70, 80, 0, 0, j),
            bx(7, 9, 114, 75, 48, 40, 361, 70, 80, 1, 0, j),
        ],
    );
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a long slash on frames 14-16, aimed up, straight or down by the stick, 361 (type 3), BK 41; the hilt 7.5% (KS 102,
    // hitlag x0.6), the blade 12% (KS 98, x1.2), the tip 9% (KS 102, x0.6), each a capsule from 6.0 ahead (radius 2.7), the reference's
    // ids as priorities (the hilt's wins where they overlap) [R]: straight 9.5 up, to 12.5 / 23.0 / 31.0 ahead; up from 12.5 up to 14.2
    // / 17.5 / 20.0 up at 11.5 / 21.5 / 29.5 ahead; down from 8.5 up to 7.1 / 4.5 / 2.5 [R]. FAF 38.
    let mut b = Vec::new();
    for (aim, y0, ends) in [
        (AIM_STRAIGHT, 95, [(125, 95), (230, 95), (310, 95)]),
        (AIM_UP, 125, [(115, 142), (215, 175), (295, 200)]),
        (AIM_DOWN, 85, [(115, 71), (215, 45), (295, 25)]),
    ] {
        for (id, (end, damage, kbg, lag)) in [
            (ends[0], 75, 102, 60u16),
            (ends[1], 120, 98, 120),
            (ends[2], 90, 102, 60),
        ]
        .into_iter()
        .enumerate()
        {
            b.extend(along(
                14,
                16,
                (60, y0),
                end,
                27,
                damage,
                361,
                41,
                kbg,
                id as u8,
                0,
                |h| {
                    h.aim = aim;
                    h.hitlag = lag;
                    fwd(h);
                },
            ));
        }
    }
    w.moves[MoveId::FTilt as usize] = moved(38, 0, 0, 255, b);

    // Up tilt: a kick that lifts, then an upward thrust of the blade [R]. Hit 1 on frame 10: 1%, BK 0, KS 100, hitlag x0.4, grounded
    // targets only, never clanking, at 5.0 (radius 4.5) and 9.5 to 10.0 ahead, 8.5 up; a position vector carries the target to 2 ahead
    // and 45 up over 5 frames (id 0) or 0 ahead, 46 up over 6 (id 1) [R] (the reference's 368 is its position-vector angle; the launch
    // angle here only sets the hitstun: 90 [E]). Hit 2, the thrust, by position (type 0), stab: clean on 13-15 the tip 14% (90, BK 74,
    // KS 80, radius 3.5, 46.5 to 48.0 up, 1.1 to 1.4 behind, x1.6), the blade 12.5% (91, BK 69, KS 75, 40.0 to 43.7 and 29.0 to 38.0
    // up, x1.2), the hilt 11% (radius 3.7, 21.0 to 25.0 up, x0.7); late on 16-25 10% / 8.5% / 7% at 96, BK 60, KS 73 (radius 2.0 for
    // the tip and blade, x0.6) [R]. The upper body (head, chest and arms) is intangible on 13-15 [R]. FAF 45 (ssbwiki.com and
    // kuroganehammer.com; ultimateframedata.com's 38 disagrees: `REFERENCE_DATA.md`).
    let lift = |vx: i32, vy: i32, frames: u8| {
        move |h: &mut Hitbox| {
            h.hitlag = 40;
            h.targets = TARGETS_GROUNDED;
            no_clank(h);
            pos(h);
            h.posvec_x = Fx::from_ratio(vx, 8);
            h.posvec_y = Fx::from_ratio(vy, 8);
            h.posvec_frames = frames;
        }
    };
    let mut b = vec![bx(10, 10, 50, 85, 45, 10, 90, 0, 100, 0, 0, lift(2, 45, 5))];
    b.extend(along(
        10,
        10,
        (95, 85),
        (100, 85),
        45,
        10,
        90,
        0,
        100,
        1,
        0,
        lift(0, 46, 6),
    ));
    for (
        first,
        last,
        tip,
        blade_dmg,
        hilt,
        angle_tip,
        angle,
        bk_tip,
        bk,
        ks_tip,
        ks,
        r_thin,
        lags,
    ) in [
        (
            13u8,
            15u8,
            140,
            125,
            110,
            90i16,
            91i16,
            74i16,
            69i16,
            80i16,
            75i16,
            35,
            [160u16, 120, 120, 70],
        ),
        (
            16,
            25,
            100,
            85,
            70,
            96,
            96,
            60,
            60,
            73,
            73,
            20,
            [60, 60, 60, 60],
        ),
    ] {
        let st = |lag: u16| {
            move |h: &mut Hitbox| {
                h.hitlag = lag;
                pos(h);
            }
        };
        b.extend(along(
            first,
            last,
            (-11, 465),
            (-14, 480),
            r_thin,
            tip,
            angle_tip,
            bk_tip,
            ks_tip,
            2,
            1,
            st(lags[0]),
        ));
        b.extend(along(
            first,
            last,
            (0, 400),
            (-7, 437),
            r_thin,
            blade_dmg,
            angle,
            bk,
            ks,
            1,
            1,
            st(lags[1]),
        ));
        b.extend(along(
            first,
            last,
            (3, 290),
            (1, 380),
            r_thin,
            blade_dmg,
            angle,
            bk,
            ks,
            3,
            1,
            st(lags[2]),
        ));
        b.extend(along(
            first,
            last,
            (1, 210),
            (1, 250),
            37,
            hilt,
            angle,
            bk,
            ks,
            0,
            1,
            st(lags[3]),
        ));
    }
    let mut utilt = moved(45, 0, 0, 255, b);
    utilt.part_intangible = vec![PartIntangible {
        start: 12,
        end: 14,
        parts: PART_MIDDLE | PART_HIGH | PART_LIMB,
        mode: MODE_ANY,
    }];
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a low sliding kick, type 3; clean on 9-15 8% (85 degrees, BK 84, KS 60): radius 2.0 from 12.0 ahead back to 2.0, 1.5
    // up, and radius 3.5 from 5.5 to 2.0, 2.0 up; late on 16-22 6% (75, BK 79, KS 60) with radii 1.2 and 2.8 [R]. FAF 43.
    let mut b = Vec::new();
    for (first, last, damage, angle, bk, r0, r1) in [
        (9u8, 15u8, 80, 85i16, 84i16, 20, 35),
        (16, 22, 60, 75, 79, 12, 28),
    ] {
        b.extend(along(
            first,
            last,
            (120, 15),
            (20, 15),
            r0,
            damage,
            angle,
            bk,
            60,
            0,
            0,
            fwd,
        ));
        b.extend(along(
            first,
            last,
            (55, 20),
            (20, 20),
            r1,
            damage,
            angle,
            bk,
            60,
            1,
            0,
            fwd,
        ));
    }
    w.moves[MoveId::DTilt as usize] = moved(43, 0, 0, 255, b);

    // Dash attack: a burst of dark magic, type 3, 8.0 up: clean on 14-15 13% (53, BK 67, KS 82, x1.3) radius 3.0 from 4.0 to 8.5 ahead
    // and radius 6.0 at 13.0; late on 16-19 7% (361, BK 66, KS 82) radii 2.5 and 4.0 [R]. FAF 47.
    let mut b = Vec::new();
    for (first, last, damage, angle, bk, lag, r0, r1) in [
        (14u8, 15u8, 130, 53i16, 67i16, 130u16, 30, 60),
        (16, 19, 70, 361, 66, 100, 25, 40),
    ] {
        let f = move |h: &mut Hitbox| {
            h.hitlag = lag;
            fwd(h);
        };
        b.extend(along(
            first,
            last,
            (40, 80),
            (85, 80),
            r0,
            damage,
            angle,
            bk,
            82,
            0,
            0,
            f,
        ));
        b.push(bx(first, last, 130, 80, r1, damage, angle, bk, 82, 1, 0, f));
    }
    w.moves[MoveId::DashAttack as usize] = moved(47, 0, 0, 255, b);
}

fn smashes(w: &mut Weapon) {
    // Forward smash: a great horizontal slash on frames 24-25, 361 (type 3): on the blade (radius 4.0) the hilt 13.5% (BK 54, KS 81,
    // x0.7) at 0, the middle 20% (BK 57, KS 78, x1.3) at 7.0 and 15.0, the tip 15% (BK 57, KS 85, x0.9) at 21.5 along it; on the root
    // 13.5% (radius 5.8, 21.5 ahead, 8.5 up) and 20% (radius 5.5, 16.5 to 21.5 ahead, 9.0 up) [R]; the ids as priorities (the
    // reference's). The blade lies level 9.0 up from a hilt 8.0 ahead [E]. Charges on frame 9 (between 8 and 9); with the wing, armor
    // on 10-23 [R]. FAF 68.
    let hilt = (80, 90);
    let dir = (100, 0);
    let s = |lag: u16| {
        move |h: &mut Hitbox| {
            h.hitlag = lag;
            fwd(h);
        }
    };
    let mut b = Vec::new();
    for (id, at, damage, bk, ks, lag) in [
        (0u8, 0, 135, 54i16, 81i16, 70u16),
        (2, 70, 200, 57, 78, 130),
        (3, 150, 200, 57, 78, 130),
        (4, 215, 150, 57, 85, 90),
    ] {
        let (x, y) = blade(hilt, dir, at);
        b.push(bx(24, 25, x, y, 40, damage, 361, bk, ks, id, 0, s(lag)));
    }
    b.push(bx(24, 25, 215, 85, 58, 135, 361, 54, 81, 1, 0, s(70)));
    b.extend(along(
        24,
        25,
        (165, 90),
        (215, 90),
        55,
        200,
        361,
        57,
        78,
        5,
        0,
        s(130),
    ));
    let mut fsmash = moved(68, 0, 0, 255, b);
    fsmash.charge_at = Some(8);
    wing_armor(&mut fsmash, 10, 23);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: an overhead arc of the blade from in front to behind, on the blade (ids 2 hilt, 0 middle, 1 tip at 5.2, 13.8 and 21.0
    // along it; radius 4.5 / 5.7 / 4.3) [R]. Early on 23 92 degrees (type 3) 10.5% / 15.5% / 11.5% (BK 62 / 67 / 67, KS 79, x0.9 / 1.4
    // / 0.9); on 24 97 (type 3) 12% / 17% / 13%; on 25-28 83 (type 4, behind) with the same damage (x0.8 / 1.4 / 0.8); on 26 also two
    // root boxes 97 (type 3) 17% (radius 5.7, 34.0 up, 5.0 ahead to 5.0 behind) and 13% (radius 4.3, 40.5 up, 8.5 to 8.5) [R]. The
    // hilt overhead 16 up, 3 ahead, the blade turning from 45 degrees ahead to 45 behind [E]. Charges on frame 13 (between 12 and 13);
    // with the wing, armor on 14-22 [R]. FAF 64.
    let hilt = (30, 160);
    let mut b = Vec::new();
    for (first, last, dir, angle, back_side, late) in [
        (23u8, 23u8, (71, 71), 92i16, false, false),
        (24, 24, (34, 94), 97, false, true),
        (25, 25, (0, 100), 83, true, true),
        (26, 26, (-34, 94), 83, true, true),
        (27, 28, (-71, 71), 83, true, true),
    ] {
        for (id, at, radius, early_dmg, late_dmg, bk, early_lag, late_lag) in [
            (2u8, 52, 45, 105, 120, 62i16, 90u16, 80u16),
            (0, 138, 57, 155, 170, 67, 140, 140),
            (1, 210, 43, 115, 130, 67, 90, 80),
        ] {
            let (x, y) = blade(hilt, dir, at);
            let (damage, lag) = if late {
                (late_dmg, late_lag)
            } else {
                (early_dmg, early_lag)
            };
            b.push(bx(
                first,
                last,
                x,
                y,
                radius,
                damage,
                angle,
                bk,
                79,
                id,
                0,
                move |h| {
                    h.hitlag = lag;
                    if back_side {
                        back(h);
                    } else {
                        fwd(h);
                    }
                },
            ));
        }
    }
    let up = |h: &mut Hitbox| {
        h.hitlag = 140;
        fwd(h);
    };
    b.extend(along(
        26,
        26,
        (50, 340),
        (-50, 340),
        57,
        170,
        97,
        67,
        79,
        3,
        0,
        up,
    ));
    b.extend(along(
        26,
        26,
        (85, 405),
        (-85, 405),
        43,
        130,
        97,
        67,
        79,
        4,
        0,
        move |h| {
            h.hitlag = 80;
            fwd(h);
        },
    ));
    let mut usmash = moved(64, 0, 0, 255, b);
    usmash.charge_at = Some(12);
    wing_armor(&mut usmash, 14, 22);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: the blade driven into the ground ahead, which erupts at its tip; type 3; on 21-23 (the ground-contact version) the
    // eruption 16.5% (63 degrees, BK 85, KS 68, radius 4.6, 23.5 ahead, 2.5 up, x1.3, shield damage +15) and its outer part 13.5%
    // (radius 8.8, 6.5 to 8.5 up, x0.7, shield damage +10, never clanks), the blade along the ground 10.5% / 11.5% / 13% (48 degrees,
    // BK 80, KS 68, radius 1.2, from 2.0 behind to 8.0, 10.0 to 19.0 and 20.0 to 24.0 ahead, x0.7 / 0.7 / 1.2) [R]; none rebound (the
    // reference's `rebound=f`) [R]. Charges on frame 9 (between 8 and 9); with the wing, armor on 10-20 [R]. FAF 54. (The version
    // whose blade meets no ground, the blade alone on 21-22, is not modelled: phase 2.)
    let nr = |lag: u16| {
        move |h: &mut Hitbox| {
            h.hitlag = lag;
            h.rebounds = false;
            fwd(h);
        }
    };
    let mut b = vec![bx(21, 23, 235, 25, 46, 165, 63, 85, 68, 0, 0, move |h| {
        nr(130)(h);
        h.shield_damage = shield(165, 150);
    })];
    b.extend(along(
        21,
        23,
        (235, 65),
        (235, 85),
        88,
        135,
        63,
        85,
        68,
        1,
        0,
        move |h| {
            nr(70)(h);
            h.clanks = false;
            h.shield_damage = shield(135, 100);
        },
    ));
    b.extend(along(
        21,
        23,
        (-20, 0),
        (80, 0),
        12,
        105,
        48,
        80,
        68,
        2,
        0,
        nr(70),
    ));
    b.extend(along(
        21,
        23,
        (100, 0),
        (190, 0),
        12,
        115,
        48,
        80,
        68,
        3,
        0,
        nr(70),
    ));
    b.extend(along(
        21,
        23,
        (200, 0),
        (240, 0),
        12,
        130,
        48,
        80,
        68,
        4,
        0,
        nr(120),
    ));
    let mut dsmash = moved(54, 0, 0, 255, b);
    dsmash.charge_at = Some(8);
    wing_armor(&mut dsmash, 10, 20);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: a burst of dark magic around the body on frames 9-10, 8.5%, 60 degrees (by position), BK 64, KS 72, radius 12.5,
    // 10.0 up [R]. FAF 34, landing lag 9, autocancel 1 and 30 on [R].
    w.moves[MoveId::NAir as usize] = moved(
        34,
        9,
        1,
        30,
        vec![bx(9, 10, 0, 100, 125, 85, 60, 64, 72, 0, 0, pos)],
    );

    // Forward air: a long stab ahead, 361 (type 3), BK 39, KS 86: clean on 13-14 the root box 8.5% (radius 4.0, 4.5 to 8.5 ahead, 12.5
    // up, x0.9), on the blade the tip 11% (radius 2.3, 20.0 to 23.5 along it, x1.3), the blade 10% (2.5, 12.5 to 17.0) and the base 10%
    // (2.8, 0 to 9.5); late on 15-18 5.5% / 8% / 7% / 7% with radii 3.5 / 1.8 / 2.0 / 2.3 [R]. The blade level ahead 12.5 up from a
    // hilt 8.5 ahead [E]. FAF 42, landing lag 14, autocancel 1-2 and 39 on [R]. (Sticking to a wall it stabs is not modelled: phase 2.)
    let hilt = (85, 125);
    let dir = (100, 0);
    let mut b = Vec::new();
    for (first, last, dmg, radii) in [
        (13u8, 14u8, [85, 110, 100, 100], [40, 23, 25, 28]),
        (15, 18, [55, 80, 70, 70], [35, 18, 20, 23]),
    ] {
        let s = |lag: u16| {
            move |h: &mut Hitbox| {
                h.hitlag = lag;
                fwd(h);
            }
        };
        b.extend(along(
            first,
            last,
            (45, 125),
            (85, 125),
            radii[0],
            dmg[0],
            361,
            39,
            86,
            0,
            0,
            s(90),
        ));
        for (id, from, to, d, r0, lag) in [
            (2u8, 200, 235, dmg[1], radii[1], 130u16),
            (1, 125, 170, dmg[2], radii[2], 100),
            (3, 0, 95, dmg[3], radii[3], 100),
        ] {
            b.extend(along(
                first,
                last,
                blade(hilt, dir, from),
                blade(hilt, dir, to),
                r0,
                d,
                361,
                39,
                86,
                id,
                0,
                s(lag),
            ));
        }
    }
    w.moves[MoveId::FAir as usize] = moved(42, 14, 2, 39, b);

    // Back air: a slash behind on frames 15-16, 42 degrees (type 4, behind), 12.5 to 13.0 up: the near part 9.5% (BK 35, KS 106, radius
    // 4.8, 6.0 to 11.3 behind, x0.8), the middle 14.5% (BK 47, KS 91, radius 6.0, 16.5 to 22.2, x1.3), the tip 11.5% (BK 39, KS 104,
    // radius 3.7, 28.0 to 30.0, x0.8) [R]. The exile does not turn round. FAF 47, landing lag 16, autocancel 1-4 and 46 on [R].
    let mut b = Vec::new();
    for (id, from, to, y, radius, damage, bk, ks, lag) in [
        (0u8, -60, -113, 125, 48, 95, 35i16, 106i16, 80u16),
        (1, -165, -222, 130, 60, 145, 47, 91, 130),
        (2, -280, -300, 125, 37, 115, 39, 104, 80),
    ] {
        b.extend(along(
            15,
            16,
            (from, y),
            (to, y),
            radius,
            damage,
            42,
            bk,
            ks,
            id,
            0,
            move |h| {
                h.hitlag = lag;
                back(h);
            },
        ));
    }
    w.moves[MoveId::BAir as usize] = moved(47, 16, 4, 46, b);

    // Up air: an overhead arc of the blade from in front to behind, radius 4.0 on the blade: the hilt 7% (x0.8) at 0.5, the middle
    // 11% (x1.1) at 7.0 and 14.0, the tip 8.5% (x0.8) at 21.0 along it; BK 55, KS 96 [R]. On 16 all 106 degrees (type 3); on 17 the
    // hilt 74 (type 4); on 18 the hilt and the first middle box 74; on 19-21 all 74 (type 4) [R]. The hilt 17 up, the blade turning
    // from 45 degrees ahead to 70 behind [E]. FAF 54, landing lag 21, autocancel 1-4 and 52 on [R].
    let hilt = (20, 170);
    let mut b = Vec::new();
    for (first, last, dir, behind_ids) in [
        (16u8, 16u8, (71, 71), 0usize),
        (17, 17, (26, 97), 1),
        (18, 18, (-26, 97), 2),
        (19, 21, (-71, 71), 4),
    ] {
        for (n, (id, at, damage, lag)) in [
            (0u8, 5, 70, 80u16),
            (1, 70, 110, 110),
            (2, 140, 110, 110),
            (3, 210, 85, 80),
        ]
        .into_iter()
        .enumerate()
        {
            let (x, y) = blade(hilt, dir, at);
            let behind = n < behind_ids;
            b.push(bx(
                first,
                last,
                x,
                y,
                40,
                damage,
                if behind { 74 } else { 106 },
                55,
                96,
                id,
                0,
                move |h| {
                    h.hitlag = lag;
                    if behind {
                        back(h);
                    } else {
                        fwd(h);
                    }
                },
            ));
        }
    }
    w.moves[MoveId::UAir as usize] = moved(54, 21, 4, 52, b);

    // Down air: a stall, then a plunge with the blade straight down [R]: the move stops the sideways momentum and sets the fall speed
    // to 1 up on frames 1-18, then 4.2 down from 19 (the stick's sideways control off on 1-46) [R]. Clean on 19-23 the tip (20.0 along
    // the blade, radius 5.5, x1.3) 15%: on grounded targets 58 degrees (BK 75, KS 65), on aerial ones 270 (a meteor, BK 20, KS 90); the
    // blade 10% (48, BK 58, KS 73, radius 6.0, 11.0 to 19.5 along it, x0.8); late on 24-39 the blade 10% (radius 4.0, 9.5 to 21.0) [R];
    // by position (type 0). The blade straight down from a hilt 5 up [E]. FAF 61, landing lag 26, autocancel 1-4 and 55 on [R]. The
    // landing hit is its own slot (below).
    let hilt = (15, 50);
    let dir = (0, -100);
    let (tx, ty) = blade(hilt, dir, 200);
    let tip = |targets: u8| {
        move |h: &mut Hitbox| {
            h.hitlag = 130;
            h.targets = targets;
            pos(h);
        }
    };
    let mut b = vec![
        bx(
            19,
            23,
            tx,
            ty,
            55,
            150,
            58,
            75,
            65,
            0,
            0,
            tip(TARGETS_GROUNDED),
        ),
        bx(
            19,
            23,
            tx,
            ty,
            55,
            150,
            270,
            20,
            90,
            1,
            0,
            tip(TARGETS_AERIAL),
        ),
    ];
    let lag8 = |h: &mut Hitbox| {
        h.hitlag = 80;
        pos(h);
    };
    b.extend(along(
        19,
        23,
        blade(hilt, dir, 110),
        blade(hilt, dir, 195),
        60,
        100,
        48,
        58,
        73,
        2,
        0,
        lag8,
    ));
    b.extend(along(
        24,
        39,
        blade(hilt, dir, 95),
        blade(hilt, dir, 210),
        40,
        100,
        48,
        58,
        73,
        0,
        0,
        lag8,
    ));
    let mut dair = moved(61, 26, 4, 55, b);
    dair.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/nodachi_dair.script"),
    ));
    dair.landing_move = Some(DAIR_LANDING as u8);
    w.moves[MoveId::DAir as usize] = dair;

    // The down air's landing (its 26 frames of landing lag): on frame 1 the shockwave 5% (361, BK 70, KS 70, radius 2.5 from 6.5
    // ahead to 6.5 behind, 2.7 up, by position, never clanks) and the blade 7% (53, BK 58, KS 73, radius 2.0, 3.0 to 24.0 along it,
    // never clanks); on 2 the shockwave alone [R]. The blade stuck straight down into the ground [E: below the stage, so in practice
    // only the shockwave and the blade's top reach anyone]. (The reference's falling and landing hits do not hit the same target:
    // `REFERENCE_DATA.md`.)
    let lhilt = (15, 20);
    let mut b = along(1, 2, (65, 27), (-65, 27), 25, 50, 361, 70, 70, 0, 0, |h| {
        no_clank(h);
        pos(h);
    });
    b.extend(along(
        1,
        1,
        blade(lhilt, dir, 30),
        blade(lhilt, dir, 240),
        20,
        70,
        53,
        58,
        73,
        1,
        0,
        |h| {
            no_clank(h);
            pos(h);
        },
    ));
    w.moves[DAIR_LANDING as usize] = moved(27, 0, 0, 255, b);
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: the reference's grounded-target box (id 0) swept along its capsule [R]: standing on 7-8 (radius 3.3, 4.7 to 8.7 ahead, 8.5
    // up), FAF 37; from a dash on 10-11 (3.0, 4.7 to 12.0, 7.0 up), FAF 45; pivot on 11-12 (3.3, 4.7 to 15.0, 7.0 up; written in front,
    // since the engine turns a pivot grab round), FAF 40 [R]. (The aerial-target box is not modelled.)
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
    w.moves[MoveId::Grab as usize] = grab(37, 7, 8, 85, 47, 87, 33);
    w.moves[MoveId::DashGrab as usize] = grab(45, 10, 11, 70, 47, 120, 30);
    w.moves[MoveId::PivotGrab as usize] = grab(40, 11, 12, 70, 47, 150, 33);

    // Pummel: dark magic, 1.3%, set knockback 30 (KS 100), 361 (type 3), hitlag x2.2, radius 5.0 at 10.0 ahead and 10.0 up, never
    // clanks [R]. The reference's hit on frame 1 is frame 2 here with the same 6 frames after it: FAF 8 [D] (the reference's 7).
    let mut pummel = with_kind(
        moved(
            8,
            0,
            0,
            255,
            vec![bx(2, 2, 100, 100, 50, 13, 361, 0, 100, 0, 0, |h| {
                h.fixed_kb = 30;
                h.hitlag = 220;
                fwd(h);
                no_clank(h);
            })],
        ),
        HIT_PUMMEL,
    );
    pummel.hitboxes[0].kind = HIT_PUMMEL;
    w.moves[MoveId::Pummel as usize] = pummel;

    // Throws: a hit on the held fighter (as a pummel: damage only) on one frame, the throw on the next [R] (no hitlag on the throw, the
    // reference's 0.0; the hit's own launch on bystanders and the thrower's invincibility up to the release are not modelled: house
    // limits). Forward: 3% on 14, the throw 3% at 28 degrees, BK 70, KS 68 on 15; FAF 35. Back: 3% on 18, the throw 3.5% at 58 (sent
    // behind), BK 60, KS 105 on 19 (type 4: sent behind, the reference's fighter teleported behind the exile); FAF 32. Up: 4% on 13,
    // the throw 3% at 98, BK 70, KS 110 on 14; FAF 33. Down: the throw 4% at 56, BK 80, KS 94 on 40; FAF 48 [R].
    let throw = |faf: u8,
                 hit: Option<(u8, i32)>,
                 frame: u8,
                 damage: i32,
                 angle: i16,
                 bkb: i16,
                 kbg: i16| {
        let mut rows = Vec::new();
        if let Some((f, d)) = hit {
            let mut h = bx(f, f, 100, 110, 60, d, 361, 0, 0, 0, 0, |_| {});
            h.kind = HIT_PUMMEL;
            h.hitlag = 0;
            rows.push(h);
        }
        let mut t = bx(
            frame, frame, 100, 110, 60, damage, angle, bkb, kbg, 0, 1, fwd,
        );
        t.kind = HIT_THROW;
        t.hitlag = 0;
        rows.push(t);
        moved(faf, 0, 0, 255, rows)
    };
    w.moves[MoveId::FThrow as usize] = throw(35, Some((14, 30)), 15, 30, 28, 70, 68);
    let mut bthrow = throw(32, Some((18, 30)), 19, 35, 58, 60, 105);
    for h in &mut bthrow.hitboxes {
        back(h);
    }
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = throw(33, Some((13, 40)), 14, 30, 98, 70, 110);
    w.moves[MoveId::DThrow as usize] = throw(48, None, 40, 40, 56, 80, 94);
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: a kick on frames 24-26, 9%, 45 degrees (type 3), BK 90, KS 20, shield damage +1 (111 [D]), never clanks; its swept
    // box (10.5 ahead to 0.5 behind, 5.0 up, radius 5.0) as circles [D]; intangible through frame 26 (on the first grab: 1-21 and 1-13
    // on the second and third, the shared 80% / 50%), FAF 56 [R].
    let mut ledge = moved(
        56,
        0,
        0,
        255,
        along(24, 26, (105, 50), (-5, 50), 50, 90, 45, 90, 20, 0, 0, |h| {
            h.shield_damage = shield(90, 10);
            fwd(h);
            no_clank(h);
        }),
    );
    ledge.intangible = 26;
    w.moves[MoveId::LedgeAttack as usize] = ledge;

    // Get-up attack (the reference's face-up and face-down floor attacks are the same): behind on frames 19-20 (12.5 to 5.0 behind),
    // then in front on 24-25 (12.0 to 5.0 ahead), 5.0 up, radius 5.0; 7%, 48 degrees (by position), BK 80, KS 48, shield damage +8 (214
    // [D]), never clanks; intangible through frame 25, FAF 46 [R]. (The trip get-up attack has no slot of its own.)
    let set = |h: &mut Hitbox| {
        h.shield_damage = shield(70, 80);
        pos(h);
        no_clank(h);
    };
    let mut b = along(19, 20, (-125, 50), (-50, 50), 50, 70, 48, 80, 48, 0, 0, set);
    b.extend(along(
        24,
        25,
        (120, 50),
        (50, 50),
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
