//! The Clutch moveset (the bounder body), modelled mechanically on a reference character: phase 1, the body's normals. A child module
//! of `moves` (it uses that file's private row builders). Data and sources: the ninth moveset's `REFERENCE_DATA.md` (`docs/movesets/`),
//! checked against ssbwiki.com's raw hitbox tables (13.0.1).

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

/// Points along a reference capsule from `(x0, y0)` to `(x1, y1)` (reference tenths), about `step` apart: the ends and evenly spaced
/// points between [D].
fn capsule(a: (i32, i32), b: (i32, i32), step: i32) -> Vec<(i32, i32)> {
    let span = (b.0 - a.0).abs().max((b.1 - a.1).abs());
    let n = (span / step.max(1)).max(1);
    (0..=n)
        .map(|i| (a.0 + (b.0 - a.0) * i / n, a.1 + (b.1 - a.1) * i / n))
        .collect()
}

/// Rows along a reference capsule: the rest of the row as `ru`.
#[allow(clippy::too_many_arguments)]
fn along_rows(
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
) -> Vec<R> {
    capsule(a, b, 30)
        .into_iter()
        .map(|(x, y)| {
            ru(
                first, last, x, y, radius, damage, angle, bkb, kbg, priority, group,
            )
        })
        .collect()
}

/// Shield damage as a percent of the damage for the reference's additive `sd` (both in tenths of a percent), rounded [D].
const fn shield(damage: i32, sd: i32) -> u16 {
    ((100 * (damage + sd) * 2 + damage) / (2 * damage)) as u16
}

fn set_all(mv: &mut Move, f: impl Fn(&mut Hitbox)) {
    for h in &mut mv.hitboxes {
        f(h);
    }
}

/// The down air's landing hit (an `ext` slot; the claws' placeholder specials use `ext0` to `ext2` until phase 2).
pub const DAIR_LANDING: MoveId = MoveId::Ext3;

/// The Clutch moveset (the bounder body): kicks, a tail, a headbutt and a long tongue. Every normal follows the reference frame for
/// frame (the ninth moveset's `REFERENCE_DATA.md` in `docs/movesets/`, version 13.0.1, read from ssbwiki.com's raw hitbox tables):
/// active frames, FAF, landing lag and autocancel windows, the jab's continuable window and its held rules, smash charge frames;
/// damage, angles and angle types, base and growth knockback, set knockback and set weight, hitlag and SDI multipliers, shield damage,
/// trip chances, grounded-only boxes, hit groups, sweet and sour phases, the down air's twelve kicks and its landing hit, no-clank boxes;
/// the head and foot intangibility windows. Radii and offsets on the reference's root bone (`top`) are its values converted (x 1.25,
/// rounded half up to a tenth) [D]; boxes on the tail, head, leg, foot and tongue bones are placed on our body where that bone is in the
/// move's pose [E]. No normal has the reference's `rebound=f` alone, or a shieldstun multiplier (checked in the raw tables). The specials
/// are still the claws' (phase 2 builds the lobbed shell and the drop, phase 3 the tongue's shell and the rolling shell).
pub fn clutch() -> Weapon {
    let mut w = claws();
    // Kicks, a tail, a head: the body's own limbs (the limb circle reaches toward the hits, and its intangibility acts on it).
    w.limbs = true;
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    w
}

fn jabs(w: &mut Weapon) {
    // Jab 1: a kick on frame 3, 3%, hitlag x1.5, angle type 3 throughout, 7.0 of the reference's units up: 361 at 5.5 ahead (BK 40,
    // KS 30) and 8.5 ahead (BK 35, KS 28), radius 1.8; the far box pulls a fighter in (180 degrees, BK 10, KS 20, radius 2.2, 12.0
    // ahead; the reference's id 2, fighters only; its id 3, the same box for objects, is not modelled) [D]. FAF 18. The continuable
    // window is frames 6-30: attack pressed in it carries on into jab 2 from frame 6 (`next_from`); attack held since jab 1 hit carries
    // on from 6 too, and held without a hit from frame 16 (the script).
    let mut jab1 = ref_move(
        18,
        0,
        0,
        255,
        &[
            ru(3, 3, 55, 70, 18, 30, 361, 40, 30, 0, 0),
            ru(3, 3, 85, 70, 18, 30, 361, 35, 28, 1, 0),
            ru(3, 3, 120, 70, 22, 30, 180, 10, 20, 2, 0),
        ],
    );
    set_all(&mut jab1, |h| {
        h.hitlag = 150;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab1, MoveId::Jab2, 6);
    // ...and past the FAF to frame 30 (owner decision 58): a press there is jab 2, not a new jab 1.
    late_chain(&mut jab1, MoveId::Jab2, 30);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/clutch_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: a second kick on its frames 3-4, 4%, 361 (type 3), BK 56, KS 70, hitlag x2: radius 3.0 at 7.0 of the reference's units up,
    // 5.0 and 8.5 ahead, radius 4.0 at 7.5 up, 13.3 ahead [D]. FAF 20. It ends the chain (no rapid jab, no third hit).
    let mut jab2 = ref_move(
        20,
        0,
        0,
        255,
        &[
            ru(3, 4, 50, 70, 30, 40, 361, 56, 70, 0, 0),
            ru(3, 4, 85, 70, 30, 40, 361, 56, 70, 1, 0),
            ru(3, 4, 133, 75, 40, 40, 361, 56, 70, 2, 0),
        ],
    );
    set_all(&mut jab2, |h| {
        h.hitlag = 200;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::Jab2 as usize] = jab2;
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a tail swipe on frames 5-7, 8%, angle type 3, BK 60, KS 75, the same numbers for each aim [R]: the tail's base 88
    // degrees (radius 4.0), 4.3 along it 85 degrees (3.5), the tail tip 85 degrees (3.0; 3.1 / 4.5 / 3.0 along the next bone up /
    // straight / down) [R]. Where the tail is in each aim is ours [E]: straight out at mid height, 30 degrees up, 20 degrees down. FAF 30.
    let mut ftilt = ref_move(30, 0, 0, 255, &[]);
    for (aim, base, mid, tip) in [
        (AIM_UP, (55, 75), (90, 95), (115, 110)),
        (AIM_STRAIGHT, (60, 55), (100, 50), (135, 50)),
        (AIM_DOWN, (60, 40), (100, 25), (125, 15)),
    ] {
        let rows = [
            ru(5, 7, base.0, base.1, 40, 80, 88, 60, 75, 0, 0),
            ru(5, 7, mid.0, mid.1, 35, 80, 85, 60, 75, 1, 0),
            ru(5, 7, tip.0, tip.1, 30, 80, 85, 60, 75, 2, 0),
        ];
        let mut mv = ref_move(30, 0, 0, 255, &rows);
        set_all(&mut mv, |h| {
            h.aim = aim;
            h.angle_type = ANGLE_FORWARD;
        });
        ftilt.hitboxes.extend(mv.hitboxes);
    }
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: the tail swept from in front up over the head to behind on frames 8-15, 7%, 100 degrees, BK 72, KS 45, angle type 0 (by
    // position): the tail tip (radius 4.0), 1.2 along the second bone (4.5) and the base (2.5) [R]. The tail high in front on 8-10 (it
    // misses most grounded targets in front, as the reference's), overhead on 11-12, high behind on 13-15 [E]. One hit. FAF 32.
    let mut rows = Vec::new();
    for (first, last, dir) in [(8u8, 10u8, 1), (11, 12, 0), (13, 15, -1)] {
        for (reach, y, radius, id) in [(110, 150, 40, 0u8), (80, 140, 45, 1), (40, 115, 25, 2)] {
            let x = if dir == 0 { reach / 4 } else { reach * dir };
            let y = if dir == 0 { y + 40 } else { y };
            rows.push(ru(first, last, x, y, radius, 70, 100, 72, 45, id, 0));
        }
    }
    let mut utilt = ref_move(32, 0, 0, 255, &rows);
    set_all(&mut utilt, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: the tail swept low along the floor in front on frames 8-10 (conflict 1), angle type 0, BK 67, KS 30: the base 5% and
    // the middle 4.5% at 28 degrees with a 30% trip chance, the tip 4% at 361 with 40% (radius 3.5 / 3.5 / 3.0) [R]; low in front [E]. One
    // hit. FAF 24. (At these angles with this base knockback it lifts a grounded target, so it trips only a target taking reduced
    // knockback, as the reference says.)
    let mut dtilt = ref_move(
        24,
        0,
        0,
        255,
        &[
            ru(8, 10, 50, 20, 35, 50, 28, 67, 30, 0, 0),
            ru(8, 10, 95, 15, 35, 45, 28, 67, 30, 1, 0),
            ru(8, 10, 135, 15, 30, 40, 361, 67, 30, 2, 0),
        ],
    );
    set_all(&mut dtilt, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.trip = if h.priority == 2 { 40 } else { 30 };
    });
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a flying side kick. Clean on frames 10-12: 11%, 65 degrees, BK 90, KS 45; late on 13-20: 8%, KS 44; angle type 3.
    // The foot (radius 5.0 clean, 4.0 late; 6.1 along the leg) and the knee (3.5 / 2.5; 2.0 along it) [R], the leg straight out in
    // front at 6.0 of the reference's units up [E]. FAF 41.
    let mut dash = ref_move(
        41,
        0,
        0,
        255,
        &[
            ru(10, 12, 110, 60, 50, 110, 65, 90, 45, 0, 0),
            ru(10, 12, 70, 60, 35, 110, 65, 90, 45, 1, 0),
            ru(13, 20, 110, 60, 40, 80, 65, 90, 44, 0, 0),
            ru(13, 20, 70, 60, 25, 80, 65, 90, 44, 1, 0),
        ],
    );
    set_all(&mut dash, |h| h.angle_type = ANGLE_FORWARD);
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash: a headbutt on frames 14-16, angle 361 (type 3), BK 30: the head 15.5% (KS 102, radius 4.5), which wins where both
    // reach (the lower id), and the nose 14% (KS 97, radius 4.8). Aimed up the head is at 11.0 of the reference's units up, 8.5 ahead
    // and the nose 11.5 up, 12.4 ahead; straight 6.5 up, 9.5 / 14.4 ahead; down 4.2 up, 8.5 / 13.0 ahead [D]. Charges on frames 7-8.
    // The head and nose are intangible on 12-16 (the high circle and the limb circle reaching toward the hit). FAF 53.
    let mut fsmash = ref_move(53, 0, 0, 255, &[]);
    for (aim, head, nose) in [
        (AIM_UP, (85, 110), (124, 115)),
        (AIM_STRAIGHT, (95, 65), (144, 65)),
        (AIM_DOWN, (85, 42), (130, 42)),
    ] {
        let rows = [
            ru(14, 16, head.0, head.1, 45, 155, 361, 30, 102, 0, 0),
            ru(14, 16, nose.0, nose.1, 48, 140, 361, 30, 97, 1, 0),
        ];
        let mut mv = ref_move(53, 0, 0, 255, &rows);
        set_all(&mut mv, |h| {
            h.aim = aim;
            h.angle_type = ANGLE_FORWARD;
        });
        fsmash.hitboxes.extend(mv.hitboxes);
    }
    fsmash.charge_at = Some(6);
    fsmash.part_intangible = vec![PartIntangible {
        start: 11,
        end: 15,
        parts: PART_HIGH | PART_LIMB,
    }];
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: a flipping bicycle kick overhead. Clean on frames 11-13: 14%; late on 14-16: 12%; 75 degrees (type 0), BK 38, KS 99:
    // the foot (radius 5.0) and the leg (3.5) [R], overhead and a little behind [E] (it hits behind). Charges on frames 7-8. The kicking
    // foot is intangible on 11-13 (the limb circle). FAF 47.
    let mut rows = Vec::new();
    for (first, last, damage) in [(11u8, 13u8, 140), (14, 16, 120)] {
        rows.push(ru(first, last, -20, 190, 50, damage, 75, 38, 99, 0, 0));
        rows.push(ru(first, last, -10, 135, 35, damage, 75, 38, 99, 1, 0));
    }
    let mut usmash = ref_move(47, 0, 0, 255, &rows);
    set_all(&mut usmash, |h| h.angle_type = ANGLE_BY_POSITION);
    usmash.charge_at = Some(6);
    usmash.part_intangible = vec![PartIntangible {
        start: 10,
        end: 12,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a tail sweep in front, then behind. Hit 1 on frames 7-8 (type 3): 12% (BK 51, KS 75, radius 3.5, 3.0 of the
    // reference's units up, 7.0 to 10.0 ahead) and 10% (BK 46, KS 63, radius 2.0, 2.0 up, 4.0 to 14.0 ahead) at 26 degrees [D]. Hit 2 on
    // 22-23, sent backward (type 4): 12% (BK 51, KS 83, 3.0 up, 7.0 to 10.0 behind) and 10% (BK 44, KS 75, 2.0 up, 3.0 to 13.0 behind)
    // [D]. Two hits (a target in front takes only the first). Charges on frames 3-4. FAF 50.
    let mut rows = along_rows(7, 8, (70, 30), (100, 30), 35, 120, 26, 51, 75, 0, 0);
    rows.extend(along_rows(
        7,
        8,
        (40, 20),
        (140, 20),
        20,
        100,
        26,
        46,
        63,
        1,
        0,
    ));
    let hit1 = rows.len();
    rows.extend(along_rows(
        22,
        23,
        (-70, 30),
        (-100, 30),
        35,
        120,
        26,
        51,
        83,
        0,
        1,
    ));
    rows.extend(along_rows(
        22,
        23,
        (-30, 20),
        (-130, 20),
        20,
        100,
        26,
        44,
        75,
        1,
        1,
    ));
    let mut dsmash = ref_move(50, 0, 0, 255, &rows);
    for (i, h) in dsmash.hitboxes.iter_mut().enumerate() {
        h.angle_type = if i < hit1 {
            ANGLE_FORWARD
        } else {
            ANGLE_BACKWARD
        };
    }
    dsmash.charge_at = Some(2);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: a held kick, one hit in three phases, 361 degrees (type 0), each a capsule along the leg [D]: clean on frames 3-4
    // 10% (BK 40, KS 86, radius 4.8, 4.3 to 3.0 of the reference's units up, 7.5 ahead to 1.0 behind), mid on 5-11 7% (BK 30, KS 100,
    // 3.6, 4.5 to 3.7 up, 5.0 ahead to 1.5 behind), late on 12-25 5% (BK 30, KS 100, 3.0, 4.8 to 4.0 up, 3.5 ahead to 1.5 behind).
    // Landing lag 7, autocancels on frames 1-2 and from 38, FAF 45.
    let mut rows = along_rows(3, 4, (75, 43), (-10, 30), 48, 100, 361, 40, 86, 0, 0);
    rows.extend(along_rows(
        5,
        11,
        (50, 45),
        (-15, 37),
        36,
        70,
        361,
        30,
        100,
        0,
        0,
    ));
    rows.extend(along_rows(
        12,
        25,
        (35, 48),
        (-15, 40),
        30,
        50,
        361,
        30,
        100,
        0,
        0,
    ));
    let mut nair = ref_move(45, 7, 2, 38, &rows);
    set_all(&mut nair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: a headbutt swung down in front on frames 16-20, angle type 3, BK 0, KS 90: the head 15% at 361 (radius 4.2, 1.5 along
    // the head bone), which wins where both reach, and the tip of the snout a meteor, 14% at 270 (4.0, 7.0 along it), on grounded and
    // aerial targets alike [R]; the head low in front [E]. One hit. Landing lag 12, autocancels on frames 1-3 and from 40, FAF 44. (The
    // reference's "ledges from both sides from frame 68" (2.0.0) is not modelled: an aerial catches ledges after its FAF here.)
    let mut fair = ref_move(
        44,
        12,
        3,
        40,
        &[
            ru(16, 20, 75, 60, 42, 150, 361, 0, 90, 0, 0),
            ru(16, 20, 120, 30, 40, 140, 270, 0, 90, 1, 0),
        ],
    );
    set_all(&mut fair, |h| h.angle_type = ANGLE_FORWARD);
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: a triple tail-kick behind, every box sent backward (type 4). Hits 1 and 2 on frames 11 and 14: 3.5% each at the autolink
    // angle, BK 20, KS 100, set weight, hitlag x0.5, SDI x0.5, radius 5.5 at 10.0 of the reference's units behind, 5.0 to 3.0 up [D].
    // Hit 3 on 18-19: 5.5% at 54 degrees, BK 30, KS 184, hitlag x1.5, radius 8.0 at 5.0 up, 11.0 to 9.0 behind [D]. Three hit groups.
    // Landing lag 11, autocancels on frames 1-5 and from 44, FAF 56.
    let mut rows = Vec::new();
    for (frame, group) in [(11u8, 0u8), (14, 1)] {
        rows.extend(along_rows(
            frame,
            frame,
            (-100, 50),
            (-100, 30),
            55,
            35,
            ANGLE_AUTOLINK,
            20,
            100,
            0,
            group,
        ));
    }
    let links = rows.len();
    rows.extend(along_rows(
        18,
        19,
        (-110, 50),
        (-90, 50),
        80,
        55,
        54,
        30,
        184,
        0,
        2,
    ));
    let mut bair = ref_move(56, 11, 5, 44, &rows);
    for (i, h) in bair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BACKWARD;
        if i < links {
            h.set_weight = true;
            h.hitlag = 50;
            h.sdi = 50;
        } else {
            h.hitlag = 150;
        }
    }
    w.moves[MoveId::BAir as usize] = bair;

    // Up air: a tail flip overhead on frames 5-6, 12%, 90 degrees (type 0), BK 25, KS 100: two boxes of radius 7.5 on the tail (its
    // second bone and 6.0 along it) [R], over the head from in front to behind [E] (it hits behind, and grounded targets there). Landing
    // lag 8, autocancels on frames 1-4 and from 31, FAF 37.
    let mut uair = ref_move(
        37,
        8,
        4,
        31,
        &[
            ru(5, 6, 20, 150, 75, 120, 90, 25, 100, 0, 0),
            ru(5, 6, -40, 130, 75, 120, 90, 25, 100, 1, 0),
        ],
    );
    set_all(&mut uair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: a flutter of kicks below. Twelve one-frame kicks two frames apart on frames 16-38, every one at the autolink angle (type
    // 0), BK 5, hitlag x0.6, SDI x1.8: the middle (radius 5.0, 2.0 of the reference's units ahead) wins over the two sides (3.5, 6.5
    // ahead and 2.5 behind) [D]; kicks 1-6 2.3% / 1.8% at 1.0 / 2.0 up (KS 80), kicks 7-12 1.9% / 1.4% at 0.5 / 1.5 up (KS 100). Then
    // the finisher on 41 (its own group, never re-armed): 2.8%, BK 55, KS 179, hitlag x1.5, the middle at 80 degrees (radius 6.5) and
    // the sides at 40 (4.5). 28% when every middle kick lands. Landing lag 17, autocancels on frames 1-13 and from 50, FAF 57. Landing
    // outside the windows is the landing hit (`DAIR_LANDING`).
    let mut rows = Vec::new();
    for k in 0..12u8 {
        let frame = 16 + 2 * k;
        let (mid, side, kbg, y_mid, y_side) = if k < 6 {
            (23, 18, 80, 10, 20)
        } else {
            (19, 14, 100, 5, 15)
        };
        rows.push(ru(
            frame,
            frame,
            20,
            y_mid,
            50,
            mid,
            ANGLE_AUTOLINK,
            5,
            kbg,
            0,
            0,
        ));
        rows.push(ru(
            frame,
            frame,
            65,
            y_side,
            35,
            side,
            ANGLE_AUTOLINK,
            5,
            kbg,
            1,
            0,
        ));
        rows.push(ru(
            frame,
            frame,
            -25,
            y_side,
            35,
            side,
            ANGLE_AUTOLINK,
            5,
            kbg,
            2,
            0,
        ));
    }
    let kicks = rows.len();
    rows.push(ru(41, 41, 20, 5, 65, 28, 80, 55, 179, 0, 1));
    rows.push(ru(41, 41, 65, 15, 45, 28, 40, 55, 179, 1, 1));
    rows.push(ru(41, 41, -25, 15, 45, 28, 40, 55, 179, 2, 1));
    let mut dair = ref_move(57, 17, 13, 50, &rows);
    for (i, h) in dair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BY_POSITION;
        if i < kicks {
            h.hitlag = 60;
            h.sdi = 180;
        } else {
            h.hitlag = 150;
        }
    }
    // (Reference frame 16 is move frame 15: the kick group is re-armed on 15, 17, ..., so each kick can land.)
    dair.rehit = Some((15, 2));
    dair.rehit_groups = 0b1;
    dair.landing_move = Some(DAIR_LANDING as u8);
    w.moves[MoveId::DAir as usize] = dair;
    // The landing hit: on the landing's frame 1, 1% at 361 (type 0), BK 65, KS 100, radius 5.0 at 2.5 of the reference's units up, 3.0
    // behind to 6.5 ahead [D]. Its landing lag 17 (FAF 18).
    let mut landing = ref_move(
        18,
        0,
        0,
        255,
        &along_rows(1, 1, (-30, 25), (65, 25), 50, 10, 361, 65, 100, 0, 0),
    );
    set_all(&mut landing, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[DAIR_LANDING as usize] = landing;
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: the tongue. Standing on frames 14-21 (FAF 49), dash 16-23 (FAF 57), pivot 17-24 (FAF 52). On the first two frames a box at
    // the mouth (radius 5.5, 6.0 of the reference's units up, 7.5 / 9.0 / 8.0 ahead: the pivot's written in front, since the engine
    // turns a pivot grab round) [D]; throughout, the tongue tip's grounded-target box (radius 2.8; 2.2 from a dash) [R], swept along the
    // tongue's reach (8.0 to 22.0 ahead at 6.0 up; 8.0 to 24.0 from a dash) [E: the tongue's length is not published, gap 3]. (Its
    // aerial-target box is not modelled: grabs catch only fighters on the ground here.)
    let tongue = |first: u8, last: u8, mouth_x: i32, tip: i32, reach: i32| -> Vec<R> {
        let mut rows = vec![ru(first, first + 1, mouth_x, 60, 55, 0, 361, 0, 0, 0, 0)];
        rows.extend(along_rows(
            first,
            last,
            (80, 60),
            (reach, 60),
            tip,
            0,
            361,
            0,
            0,
            1,
            0,
        ));
        rows
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(49, 0, 0, 255, &tongue(14, 21, 75, 28, 220)),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(57, 0, 0, 255, &tongue(16, 23, 90, 22, 240)),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(52, 0, 0, 255, &tongue(17, 24, 80, 28, 220)),
        HIT_GRAB,
    );
    // Pummel: a headbutt, 1.3%, set knockback 30 (KS 100), 361 (type 3), hitlag x2.1, radius 5.7 on the head [E], no clank. The
    // reference hits on frame 1 with FAF 7; ours on frame 2 with FAF 8 (a pummel cannot hit on its first frame) [D].
    let mut pummel = reference_pummel(13, u(60), u(80), u(57));
    set_all(&mut pummel, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
    });
    w.moves[MoveId::Pummel as usize] = pummel;
    // Throws (no pre-hits; the "interrupted" rows and the thrower's invincibility are not modelled: house limits). Forward: on frame 15,
    // 9% at 45 degrees, BK 55, KS 66, FAF 40. Back: turns round and throws behind on 20, the same numbers, FAF 44. Up: on 13, 5% at 90,
    // BK 65, KS 90, FAF 44. Down: 4% at 80, BK 100, KS 25; the reference freezes both fighters for 8 frames of hitlag on its frame 17 and
    // releases on 18 with FAF 44 counted without them, so here (nobody frozen) the release is on 26 and the FAF 52, the same timeline
    // [D] (conflict 7).
    let throw = |faf: u8, frame: u8, x: i32, damage: i32, angle: i16, bkb: i16, kbg: i16| {
        with_kind(
            ref_move(
                faf,
                0,
                0,
                255,
                &[r(frame, frame, x, 11, 10, damage, angle, bkb, kbg, 0, 0)],
            ),
            HIT_THROW,
        )
    };
    w.moves[MoveId::FThrow as usize] = throw(40, 15, 13, 90, 45, 55, 66);
    let mut bthrow = throw(44, 20, -13, 90, 45, 55, 66);
    bthrow.turns_around = true;
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = throw(44, 13, 13, 50, 90, 65, 90);
    w.moves[MoveId::DThrow as usize] = throw(52, 26, 13, 40, 80, 100, 25);
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: frames 20-22, 9%, 45 degrees (type 3: forward on both sides), BK 90, KS 20, shield damage +1 (111 [D]), no clank;
    // intangible through frame 22 (on the first grab: 1-18 and 1-11 on the second and third, the body's decay 82% / 50%), FAF 56. Its
    // swept box (12.5 to 1.0 of the reference's units ahead, 5.0 up, radius 5.0) as circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along_rows(20, 22, (125, 50), (10, 50), 50, 90, 45, 90, 20, 0, 0),
    );
    set_all(&mut ledge, |h| {
        h.shield_damage = shield(90, 10);
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
    });
    ledge.intangible = 22;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack (lying face up, the reference's "back" floor attack): in front on frames 14-15 (14.0 to 3.0 of the reference's units
    // ahead), then behind on 20-21 (15.0 to 4.0 behind), 5.0 up, radius 5.0; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8
    // (214 [D]), no clank; intangible through frame 21, FAF 46. (The face-down and trip get-up attacks have no slot of their own.)
    let mut rows = along_rows(14, 15, (140, 50), (30, 50), 50, 70, 48, 80, 48, 0, 0);
    rows.extend(along_rows(
        20,
        21,
        (-150, 50),
        (-40, 50),
        50,
        70,
        48,
        80,
        48,
        0,
        1,
    ));
    let mut getup = ref_move(46, 0, 0, 255, &rows);
    set_all(&mut getup, |h| {
        h.shield_damage = shield(70, 80);
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
    });
    getup.intangible = 21;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}
