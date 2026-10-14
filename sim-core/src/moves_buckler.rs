//! The Buckler moveset (the wanderer body), modelled mechanically on a reference character: phase 1, every sword normal and both
//! versions of the up special (the spin). A child module of `moves` (it uses that file's private row builders). Data and sources: the
//! tenth moveset's `REFERENCE_DATA.md` (`docs/movesets/`), read from ssbwiki.com's raw hitbox tables (13.0.1).

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

/// Rows along a reference capsule (`a` to `b`, reference tenths): the rest of the row as `ru`.
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
) -> Vec<R> {
    capsule(a, b, (radius / 2).max(10))
        .into_iter()
        .map(|(x, y)| {
            ru(
                first, last, x, y, radius, damage, angle, bkb, kbg, priority, group,
            )
        })
        .collect()
}

/// Where the sword's `sword2` bone starts, in reference tenths along the blade from the grip [E: the reference's own skeleton; the
/// guard is about 4 units past the hand].
const SWORD2_BASE: i32 = 40;

/// A sword pose: the grip (the hand) at `(gx, gy)` and the blade pointing along `(dx, dy)` (thousandths), reference tenths [E].
type Pose = (i32, i32, i32, i32);

/// The point `x` reference tenths along the `sword2` bone in `pose` (the distance along the bone is the reference's [R], the pose
/// ours [E]).
const fn on_sword(pose: Pose, x: i32) -> (i32, i32) {
    let at = SWORD2_BASE + x;
    (pose.0 + pose.2 * at / 1000, pose.1 + pose.3 * at / 1000)
}

/// A row at `x` tenths along `sword2` in `pose`: the rest as `ru`.
#[allow(clippy::too_many_arguments)]
fn sw(
    first: u8,
    last: u8,
    pose: Pose,
    x: i32,
    radius: i32,
    damage: i32,
    angle: i16,
    bkb: i16,
    kbg: i16,
    priority: u8,
    group: u8,
) -> R {
    let (px, py) = on_sword(pose, x);
    ru(
        first, last, px, py, radius, damage, angle, bkb, kbg, priority, group,
    )
}

/// The sword arm (`armr`) and the hand at the hilt (`colonells`) in a pose: the forearm half way from the shoulder (2 behind the
/// grip toward the chest at 12 up) and the hand at the grip [E].
const fn arm(pose: Pose) -> (i32, i32) {
    (pose.0 / 2, (pose.1 + 120) / 2)
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

/// The up special's aerial version (the multi-hit rising spin).
pub const SPIN_AIR: MoveId = MoveId::Ext0;
/// The forward smash's second hit (its own move, continued by a press).
pub const FSMASH_2: MoveId = MoveId::Ext1;
/// The down air after it bounced off a target, and its last frames (the original move's frames 65 on, all autocancelling).
pub const DAIR_BOUNCE: MoveId = MoveId::Ext2;
pub const DAIR_BOUNCE_END: MoveId = MoveId::Ext3;

/// The Buckler moveset (the wanderer body): a sword, a small shield that stops shots (the body's passive shield, E14) and a pack of
/// tools. Every sword normal follows the reference frame for frame (the tenth moveset's `REFERENCE_DATA.md`, 13.0.1, ssbwiki.com's
/// raw hitbox tables): active frames, FAF, landing lag and autocancel windows, continuable jab windows and their held rules, smash
/// charge frames, the two-part forward smash keeping its charge (E36); damage, angles and angle types, base and growth knockback, set
/// knockback and set weight, hitlag and SDI multipliers, hitstun modifiers, shield damage and shieldstun, trip chances, grounded- and
/// aerial-only boxes, hit groups, tip / blade / arm / hilt boxes and their no-clank flags; the down air's bounce. The up special is the
/// reference's spin, both versions (the grounded charged single hit, the aerial eight hits with position vectors (E34), the rise and
/// its special fall). Offsets on the reference's root bone (`top`) are its values converted [D]; boxes on the sword, arm, hand and leg
/// bones are placed on our body where that bone is in the move's pose [E]. The neutral, side and down specials (the bow, boomerang
/// and bomb) are the claws' placeholders until phases 2 and 3.
pub fn buckler() -> Weapon {
    let mut w = claws();
    // A held sword: the hits reach out beyond the body, so no limb reaches out to be hit with them.
    w.limbs = false;
    // The claws' up special used ext0 to ext2: empty every ext slot first (the placeholders left use none).
    for i in 0..MoveId::COUNT as u8 {
        if MoveId::from_index(i).is_ext() {
            w.moves[usize::from(i)] = Move::empty();
        }
    }
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    spin(&mut w);
    tools(&mut w);
    w
}

fn jabs(w: &mut Weapon) {
    // Jab 1 (13.0.1): frames 7-8, 3%, angle type 3, hitlag x1.6; three capsules at 8.0 up from 7.0 ahead to 10.0 / 14.0 / 18.0
    // (radius 1.8): 361 (BK 25, KS 25) near and middle, the far one 180 (BK 20, KS 15), which pulls a fighter in (the reference's id 2,
    // fighters only; its id 3, the same box for objects, is not modelled). FAF 24. Attack pressed on 10-35 carries on into jab 2 from
    // frame 10 (the continuable window); held since a hit, from 10 too; held without a hit, from 14 (the script).
    let mut rows = along(7, 8, (100, 80), (70, 80), 18, 30, 361, 25, 25, 0, 0);
    rows.extend(along(7, 8, (140, 80), (70, 80), 18, 30, 361, 25, 25, 1, 0));
    rows.extend(along(7, 8, (180, 80), (70, 80), 18, 30, 180, 20, 15, 2, 0));
    let mut jab1 = ref_move(24, 0, 0, 255, &rows);
    set_all(&mut jab1, |h| {
        h.hitlag = 160;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab1, MoveId::Jab2, 10);
    // (The window runs to 35, past the FAF 24: a press after the end still starts jab 2, owner decision 58.)
    late_chain(&mut jab1, MoveId::Jab2, 35);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: its frames 5-6, 3%, hitlag x1.2, type 3; at 8.5 up from 8.5 ahead to 9.5 (radius 2.8) / 14.5 (3.5) / 18.5 (3.5, the 180
    // pull, BK 20, KS 15). FAF 26 (conflict 2: UFD and dragdown). Pressed on 8-35 carries on into jab 3 from frame 8; held since a hit
    // from 8, held without one from 13.
    let mut rows = along(5, 6, (95, 85), (85, 85), 28, 30, 361, 25, 25, 0, 0);
    rows.extend(along(5, 6, (145, 85), (85, 85), 35, 30, 361, 25, 25, 1, 0));
    rows.extend(along(5, 6, (185, 85), (85, 85), 35, 30, 180, 20, 15, 2, 0));
    let mut jab2 = ref_move(26, 0, 0, 255, &rows);
    set_all(&mut jab2, |h| {
        h.hitlag = 120;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab2, MoveId::Jab3, 8);
    late_chain(&mut jab2, MoveId::Jab3, 35);
    jab2.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_jab_2.script"),
    ));
    w.moves[MoveId::Jab2 as usize] = jab2;

    // Jab 3: the thrust, frames 6-7, 4%, 30 degrees (type 3: the semi-spike), BK 70, KS 70, hitlag x2: two root boxes (radius 4.5, 8.0
    // up, 10.0 and 15.0 ahead) and the blade's tip (radius 3.8, `sword1` [E]: 20.0 ahead). FAF 35.
    let mut jab3 = ref_move(
        35,
        0,
        0,
        255,
        &[
            ru(6, 7, 100, 80, 45, 40, 30, 70, 70, 0, 0),
            ru(6, 7, 150, 80, 45, 40, 30, 70, 70, 1, 0),
            ru(6, 7, 200, 80, 38, 40, 30, 70, 70, 2, 0),
        ],
    );
    set_all(&mut jab3, |h| {
        h.hitlag = 200;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::Jab3 as usize] = jab3;
}

/// Four boxes of a sword hit in a pose: the tip and the blade on `sword2` (`(x along it, radius)`), the forearm (`armr`) and the hand
/// at the hilt (`colonells`) with their radii (0 for none). Priorities 0 to 3 as the reference's ids.
#[allow(clippy::too_many_arguments)]
fn sword_hit(
    first: u8,
    last: u8,
    pose: Pose,
    tip: (i32, i32),
    blade: (i32, i32),
    arm_r: i32,
    hand_r: i32,
    damage: [i32; 4],
    angle: [i16; 4],
    bkb: i16,
    kbg: [i16; 4],
    group: u8,
) -> Vec<R> {
    let mut rows = vec![
        sw(
            first, last, pose, tip.0, tip.1, damage[0], angle[0], bkb, kbg[0], 0, group,
        ),
        sw(
            first, last, pose, blade.0, blade.1, damage[1], angle[1], bkb, kbg[1], 1, group,
        ),
    ];
    if arm_r > 0 {
        let (ax, ay) = arm(pose);
        rows.push(ru(
            first, last, ax, ay, arm_r, damage[2], angle[2], bkb, kbg[2], 2, group,
        ));
    }
    if hand_r > 0 {
        rows.push(ru(
            first, last, pose.0, pose.1, hand_r, damage[3], angle[3], bkb, kbg[3], 3, group,
        ));
    }
    rows
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: an overhead chop. 13%, 361 (type 3), BK 55, KS 82, hitlag x1. The blade (`sword2` 1.8, radius 4.0), the forearm
    // (2.9) and the hand (2.1) from frame 15, when the sword is still high (it "hits targets above him"), and the tip (7.8, radius 3.5)
    // with them on 16-19, the sword coming down in front [E: poses]. The arm and hand boxes never clank. FAF 40.
    let high: Pose = (60, 150, 500, 866);
    let front: Pose = (80, 100, 940, -342);
    let mut rows = vec![sw(15, 15, high, 18, 40, 130, 361, 55, 82, 1, 0)];
    let (ax, ay) = arm(high);
    rows.push(ru(15, 15, ax, ay, 29, 130, 361, 55, 82, 2, 0));
    rows.push(ru(15, 15, high.0, high.1, 21, 130, 361, 55, 82, 3, 0));
    rows.extend(sword_hit(
        16,
        19,
        front,
        (78, 35),
        (18, 40),
        29,
        21,
        [130; 4],
        [361; 4],
        55,
        [82; 4],
        0,
    ));
    let mut ftilt = ref_move(40, 0, 0, 255, &rows);
    set_all(&mut ftilt, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.clanks = h.priority < 2;
        h.rebounds = h.priority < 2;
    });
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: the sword swept from behind over the head to the front, frames 8-12, 11%, BK 30, hitlag x0.9, angle type 0 (by
    // position). The tip (`sword2` 8.4, radius 3.4) 95 degrees KS 103, the blade (2.0, radius 4.3) 85 KS 111, the forearm (2.7) 85
    // KS 105, the hand (2.16) 85 KS 104; the arm and hand never clank. The sweep [E]: up and back on 8-9, straight up on 10, up and
    // forward on 11-12. FAF 36.
    let mut rows = Vec::new();
    for (first, last, pose) in [
        (8u8, 9u8, (0, 150, -500, 866)),
        (10, 10, (10, 150, 0, 1000)),
        (11, 12, (20, 150, 500, 866)),
    ] {
        rows.extend(sword_hit(
            first,
            last,
            pose,
            (84, 34),
            (20, 43),
            27,
            22,
            [110; 4],
            [95, 85, 85, 85],
            30,
            [103, 111, 105, 104],
            0,
        ));
    }
    let mut utilt = ref_move(36, 0, 0, 255, &rows);
    set_all(&mut utilt, |h| {
        h.hitlag = 90;
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = h.priority < 2;
        h.rebounds = h.priority < 2;
    });
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a low thrust, frames 10-11, 9%, 85 degrees (13.0.1; type 3), BK 90, KS 30, radius 2.0 along the reference's capsule
    // from (17.5 ahead, 1.3 up) to (5.5, 5.0) [D], never clanks. FAF 31.
    let mut dtilt = ref_move(
        31,
        0,
        0,
        255,
        &along(10, 11, (175, 13), (55, 50), 20, 90, 85, 90, 30, 0, 0),
    );
    set_all(&mut dtilt, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
    });
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a running slash, frames 20-23, type 3, shieldstun x1.4 on every box: the tip (`sword2` 8.5, radius 3.2) 14% at 45,
    // BK 70, KS 85, hitlag x1.2; the blade (3.0, radius 3.5) 13% at 50, BK 85, KS 85; the forearm (radius 3.0) 12% at 55, BK 85, KS
    // 77, never clanks. The sword held forward at the hip [E]. FAF 57.
    let pose: Pose = (80, 80, 985, -174);
    let mut rows = vec![
        sw(20, 23, pose, 85, 32, 140, 45, 70, 85, 0, 0),
        sw(20, 23, pose, 30, 35, 130, 50, 85, 85, 1, 0),
    ];
    let (ax, ay) = arm(pose);
    rows.push(ru(20, 23, ax, ay, 30, 120, 55, 85, 77, 2, 0));
    let mut dash = ref_move(57, 0, 0, 255, &rows);
    set_all(&mut dash, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.shieldstun = 140;
        h.hitlag = if h.priority == 0 { 120 } else { 100 };
        h.clanks = h.priority < 2;
        h.rebounds = h.priority < 2;
    });
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash, hit 1: a downward slash on frames 17-18, type 3: the tip (`sword2` 8.5, radius 3.5) 14% at 361, BK 30, KS 100; the
    // blade (2.7, radius 4.3) 7% at 69, BK 45, KS 12, and the body box (8.5 up, 6.0 to 3.5 ahead, radius 3.2) 7% at 60, BK 45, KS 12:
    // a weak lift into hit 2. Charges on frame 9 (move frame 8). FAF 51. Attack pressed on frames 23-36 (a press from 14 waits in the
    // buffer) starts hit 2 (the script), which keeps hit 1's charge (E36). (The sword wave at 0% is phase 2.)
    let pose: Pose = (90, 100, 940, -342);
    let mut rows = vec![
        sw(17, 18, pose, 27, 43, 70, 69, 45, 12, 0, 0),
        sw(17, 18, pose, 85, 35, 140, 361, 30, 100, 1, 0),
    ];
    rows.extend(along(17, 18, (60, 85), (35, 85), 32, 70, 60, 45, 12, 2, 0));
    let mut fsmash = ref_move(51, 0, 0, 255, &rows);
    set_all(&mut fsmash, |h| h.angle_type = ANGLE_FORWARD);
    fsmash.charge_at = Some(8);
    fsmash.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_fsmash.script"),
    ));
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Forward smash, hit 2 (`ext1`): a rising slash on its frames 10-11, 48 degrees (type 3), BK 85, KS 89, hitlag x1.5: the blade
    // (`sword2` 7.0 and 1.5, radius 4.5) 13%, the forearm (radius 3.5) 12%, the body box (9.0 up, 3.0 ahead, radius 3.5) 12%, which
    // never clanks. Charged by hit 1's charge (E36: `keep_charge`). FAF 61.
    let pose: Pose = (80, 90, 940, 342);
    let (ax, ay) = arm(pose);
    let mut hit2 = ref_move(
        61,
        0,
        0,
        255,
        &[
            sw(10, 11, pose, 70, 45, 130, 48, 85, 89, 0, 0),
            sw(10, 11, pose, 15, 45, 130, 48, 85, 89, 1, 0),
            ru(10, 11, ax, ay, 35, 120, 48, 85, 89, 2, 0),
            ru(10, 11, 30, 90, 35, 120, 48, 85, 89, 3, 0),
        ],
    );
    set_all(&mut hit2, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.hitlag = 150;
        h.clanks = h.priority != 3;
        h.rebounds = h.priority != 3;
    });
    hit2.keep_charge = true;
    w.moves[FSMASH_2 as usize] = hit2;

    // Up smash: three slashes overhead (the sword sweeping above the head [E]), angle type 0, every box +3 hitstun (9.0.0) but the
    // late aerial tip's. Hits 1 and 2 (4% on clean 10-13 / late 14-15, 3% on clean 25-28 / late 29-30): set knockback with set weight
    // (BK 0, KS 100), SDI x0.5: the hand (radius 3.5) 105 degrees FKV 55; the blade (`sword2` 2.5, radius 4.8) and the tip (8.5, radius
    // 3.0) on aerial targets at 115 (FKV 38 / 28), on grounded ones at 107 (FKV 28, 9.0.0); late, the aerial tip at 135, FKV 45 (hit 1)
    // or 40 (hit 2), with no +3. Hit 3 on 41-45 (13.0.1: KS 101), BK 60, never clanks: the tip (7.0, radius 4.3) 11% at 80, the blade
    // (1.0, radius 4.3) 11% at 90, the forearm (3.0) 10% at 90, the body box (11.0 up, 0.5 behind, radius 5.2) 9% at 90. Charges on 5
    // (move frame 4). FAF 78.
    let over: Pose = (0, 170, 0, 1000);
    let mut rows = Vec::new();
    for (hit, damage, clean, late) in [
        (0u8, 40, (10u8, 13u8), (14u8, 15u8)),
        (1, 30, (25, 28), (29, 30)),
    ] {
        for (first, last, is_late) in [(clean.0, clean.1, false), (late.0, late.1, true)] {
            let tip_air_angle = if is_late { 135 } else { 115 };
            rows.push(ru(
                first, last, over.0, over.1, 35, damage, 105, 0, 100, 0, hit,
            ));
            rows.push(sw(first, last, over, 25, 48, damage, 115, 0, 100, 1, hit));
            rows.push(sw(
                first,
                last,
                over,
                85,
                30,
                damage,
                tip_air_angle,
                0,
                100,
                3,
                hit,
            ));
            rows.push(sw(first, last, over, 25, 48, damage, 107, 0, 100, 2, hit));
            rows.push(sw(first, last, over, 85, 30, damage, 107, 0, 100, 4, hit));
        }
    }
    let hit3 = rows.len();
    rows.extend(sword_hit(
        41,
        45,
        over,
        (70, 43),
        (10, 43),
        30,
        0,
        [110, 110, 100, 0],
        [80, 90, 90, 90],
        60,
        [101; 4],
        2,
    ));
    rows.push(ru(41, 45, -5, 110, 52, 90, 90, 60, 101, 3, 2));
    let mut usmash = ref_move(78, 0, 0, 255, &rows);
    for (i, h) in usmash.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BY_POSITION;
        h.hitstun_add = 3;
        if i < hit3 {
            h.set_weight = true;
            h.sdi = 50;
            let late = matches!(h.start + 1, 14 | 29);
            h.fixed_kb = match h.priority {
                0 => 55,
                1 => 38,
                3 if late => {
                    h.hitstun_add = 0;
                    if h.group == 0 {
                        45
                    } else {
                        40
                    }
                }
                _ => 28,
            };
            h.targets = match h.priority {
                1 | 3 => TARGETS_AERIAL,
                2 | 4 => TARGETS_GROUNDED,
                _ => TARGETS_ALL,
            };
        } else {
            h.clanks = false;
            h.rebounds = false;
        }
    }
    usmash.charge_at = Some(4);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a low sweep in front, then behind, angle type 0, trip chance 30% on every box (it never trips at these angles: the
    // reference's too). In front on 12-13 (78 degrees, BK 40, KS 88): the blade (`sword2` 1.5, radius 3.5) 16%, the tip (7.5, radius
    // 3.6) 17%, the forearm (3.4) 16%, the hand (3.0) 14%. Behind on 24-25 (30 degrees: the semi-spike, BK 80, KS 63): the blade and tip
    // (radius 3.5 / 3.6) 12%, the forearm 11%, the hand 10%. Charges on 4 (move frame 3). FAF 57 (conflict 4: UFD and dragdown).
    let front: Pose = (60, 30, 1000, 0);
    let back: Pose = (-60, 30, -1000, 0);
    let mut rows = vec![
        sw(12, 13, front, 15, 35, 160, 78, 40, 88, 0, 0),
        sw(12, 13, front, 75, 36, 170, 78, 40, 88, 1, 0),
    ];
    let (ax, ay) = arm(front);
    rows.push(ru(12, 13, ax, ay, 34, 160, 78, 40, 88, 2, 0));
    rows.push(ru(12, 13, front.0, front.1, 30, 140, 78, 40, 88, 3, 0));
    rows.push(sw(24, 25, back, 15, 35, 120, 30, 80, 63, 0, 1));
    rows.push(sw(24, 25, back, 75, 36, 120, 30, 80, 63, 1, 1));
    let (bx, by) = arm(back);
    rows.push(ru(24, 25, bx, by, 34, 110, 30, 80, 63, 2, 1));
    rows.push(ru(24, 25, back.0, back.1, 30, 100, 30, 80, 63, 3, 1));
    let mut dsmash = ref_move(57, 0, 0, 255, &rows);
    set_all(&mut dsmash, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.trip = 30;
    });
    dsmash.charge_at = Some(3);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: a kick, angle 361 (type 0), KS 100. Clean on 7-8: the foot (radius 5.0, the front knee's bone at 3.5 [E]) 11%, BK
    // 22; the knee (5.0) and the back knee (3.0) 9%. Late on 9-31: all 6%, BK 15 (radius 4.5 / 4.5 / 3.0). Landing lag 6, autocancels
    // on 1-3 and from 36, FAF 39.
    let mut rows = Vec::new();
    for (first, last, foot, other, bkb, r0, r1) in [
        (7u8, 8u8, 110, 90, 22i16, 50, 50),
        (9, 31, 60, 60, 15, 45, 45),
    ] {
        rows.push(ru(first, last, 75, 55, r0, foot, 361, bkb, 100, 0, 0));
        rows.push(ru(first, last, 25, 75, r1, other, 361, bkb, 100, 1, 0));
        rows.push(ru(first, last, -25, 55, 30, other, 361, bkb, 100, 2, 0));
    }
    let mut nair = ref_move(39, 6, 3, 36, &rows);
    set_all(&mut nair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: two slashes in front, angle type 3, root boxes [D] (11.5 / 11.0 up). Hit 1 on 16-17 (8%, set weight, hitlag x0.8):
    // at 18.0 ahead (radius 4.0) 43 degrees (BK 31, KS 28) on grounded targets and the autolink 367 (BK 25, KS 30) on aerial ones; at
    // 11.5 (radius 6.5) 47 (BK 33, KS 30) / 367 (BK 25, KS 30); at 6.5 (radius 7.0) 47 (BK 33, KS 28) / 38 (BK 26, KS 26); the two
    // inner boxes never clank. The 367 boxes drag a falling target down into hit 2 (owner decisions 4 and 57: the reference's own
    // data). Hit 2 on 24-25 (10%, 44 degrees, BK 43, KS 120, hitlag x1.2): at 17.5 (radius 4.5), 11.5 (6.5) and 6.7 (6.8, never
    // clanks). Landing lag 11, autocancels from 53 (after the FAF: never), FAF 52.
    let mut rows = vec![
        ru(16, 17, 180, 115, 40, 80, 43, 31, 28, 0, 0),
        ru(16, 17, 180, 115, 40, 80, ANGLE_AUTOLINK, 25, 30, 1, 0),
        ru(16, 17, 115, 115, 65, 80, 47, 33, 30, 2, 0),
        ru(16, 17, 115, 115, 65, 80, ANGLE_AUTOLINK, 25, 30, 3, 0),
        ru(16, 17, 65, 115, 70, 80, 47, 33, 28, 4, 0),
        ru(16, 17, 65, 115, 70, 80, 38, 26, 26, 5, 0),
    ];
    let hit1 = rows.len();
    rows.push(ru(24, 25, 175, 110, 45, 100, 44, 43, 120, 0, 1));
    rows.push(ru(24, 25, 115, 110, 65, 100, 44, 43, 120, 1, 1));
    rows.push(ru(24, 25, 67, 110, 68, 100, 44, 43, 120, 2, 1));
    let mut fair = ref_move(52, 11, 0, 53, &rows);
    for (i, h) in fair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        if i < hit1 {
            h.set_weight = true;
            h.hitlag = 80;
            h.targets = if h.priority % 2 == 0 {
                TARGETS_GROUNDED
            } else {
                TARGETS_AERIAL
            };
            h.clanks = h.priority < 2;
            h.rebounds = h.priority < 2;
        } else {
            h.hitlag = 120;
            h.clanks = h.priority < 2;
            h.rebounds = h.priority < 2;
        }
    }
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: two kicks behind, angle type 4 (backward). Kick 1 on 6-8 (5%, set knockback with set weight, BK 0, KS 100, hitlag
    // x0.8): the foot (radius 4.0) 73 degrees FKV 50 on grounded targets, 70 FKV 36 on aerial ones; the knee (4.5) and the hip (5.0) 70
    // FKV 50 / 66 FKV 36. Kick 2 on 15-17 (7%, 361, BK 45, KS 98): the other foot (4.5), knee (5.0) and hip (5.5). Leg positions [E].
    // Landing lag 6, autocancels from 29, FAF 31.
    let mut rows = Vec::new();
    for (id, x, y, radius, angle) in [
        (0u8, -95, 70, 40, 73i16),
        (1, -95, 70, 40, 70),
        (2, -40, 75, 45, 70),
        (3, -40, 75, 45, 66),
        (4, -5, 85, 50, 70),
        (5, -5, 85, 50, 66),
    ] {
        rows.push(ru(6, 8, x, y, radius, 50, angle, 0, 100, id, 0));
    }
    let kick1 = rows.len();
    rows.push(ru(15, 17, -95, 70, 45, 70, 361, 45, 98, 0, 1));
    rows.push(ru(15, 17, -40, 75, 50, 70, 361, 45, 98, 1, 1));
    rows.push(ru(15, 17, -5, 85, 55, 70, 361, 45, 98, 2, 1));
    let mut bair = ref_move(31, 6, 0, 29, &rows);
    for (i, h) in bair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BACKWARD;
        if i < kick1 {
            h.set_weight = true;
            h.hitlag = 80;
            let grounded = h.priority % 2 == 0;
            h.targets = if grounded {
                TARGETS_GROUNDED
            } else {
                TARGETS_AERIAL
            };
            h.fixed_kb = if grounded { 50 } else { 36 };
        }
    }
    w.moves[MoveId::BAir as usize] = bair;

    // Up air: one long stab straight up, one box along the blade (`sword2` 6.9 to -1.5, radius 4.5) out from 11 to 40: 15% on 11-13
    // (BK 23, KS 93), 13% on 14-40 (BK 18, KS 88), 85 degrees (type 0), hitlag x1.2. Landing lag 14, autocancels on 1-10 and from 43
    // (2.0.0), FAF 60.
    let up: Pose = (0, 140, 0, 1000);
    let mut rows = Vec::new();
    for (first, last, damage, bkb, kbg) in [(11u8, 13u8, 150, 23i16, 93i16), (14, 40, 130, 18, 88)]
    {
        for x in [69, 42, 15, -15] {
            rows.push(sw(first, last, up, x, 45, damage, 85, bkb, kbg, 0, 0));
        }
    }
    let mut uair = ref_move(60, 14, 10, 43, &rows);
    set_all(&mut uair, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.hitlag = 120;
    });
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: the sword driven down (root boxes at the feet [D]), type 0, hitlag x1.2. Clean on 14-19 (radius 4.0, 1.0 up): 18% at
    // 270 (BK 30, KS 80) on aerial targets (the meteor), 18% at 60 (BK 30, KS 78) on grounded ones. Late on 20-64: 15% at 60 (BK 30,
    // KS 80), radius 4.2 from 1.5 to 4.0 up. Landing lag 19, autocancels on 1-9 and from 65, FAF 80. A hit when not fast falling
    // bounces the fighter up (vertical speed 1.33) and the box comes back 30 frames later as the after-bounce hit (11%, 65 degrees, BK
    // 50, KS 80, radius 4.3, 1.0 up and 0.5 ahead) until the down air's frame 64 [E: its end]; the script and two `ext` slots carry it
    // (`DAIR_BOUNCE`, `DAIR_BOUNCE_END`).
    let mut rows = vec![
        ru(14, 19, 0, 10, 40, 180, 270, 30, 80, 0, 0),
        ru(14, 19, 0, 10, 40, 180, 60, 30, 78, 1, 0),
    ];
    let clean = rows.len();
    rows.extend(along(20, 64, (0, 15), (0, 40), 42, 150, 60, 30, 80, 0, 0));
    let mut dair = ref_move(80, 19, 9, 65, &rows);
    for (i, h) in dair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BY_POSITION;
        h.hitlag = 120;
        if i < clean {
            h.targets = if h.priority == 0 {
                TARGETS_AERIAL
            } else {
                TARGETS_GROUNDED
            };
        }
    }
    dair.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_dair.script"),
    ));
    w.moves[MoveId::DAir as usize] = dair;

    // After the bounce (`ext2`): its frame 0 is the down air's frame after the hit (the frame the script switches on), so its box on
    // its frames 30-50 is 30 frames after the hit; the script ends it into `ext3` at the down air's frame 65. Landing lag 19.
    let mut bounce = ref_move(
        80,
        19,
        0,
        255,
        &[ru(30, 50, 5, 10, 43, 110, 65, 50, 80, 0, 0)],
    );
    set_all(&mut bounce, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.hitlag = 120;
    });
    bounce.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_dair_bounce.script"),
    ));
    w.moves[DAIR_BOUNCE as usize] = bounce;
    // The down air's frames 65 to 79 after a bounce (`ext3`): no box, every landing autocancels, FAF 80 of the down air.
    let mut end = ref_move(16, 0, 0, 255, &[]);
    end.autocancel_after = 0;
    end.script = Some(script(
        Kind::Fighter,
        "// (no logic: the down air's last frames)\n",
    ));
    w.moves[DAIR_BOUNCE_END as usize] = end;
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: a hand grab (no tether). Standing on 6-7 (FAF 35): 8.0 up, 4.0 to 7.7 ahead, radius 3.3; dash on 9-10 (FAF 43): 7.0 up, 4.0
    // to 9.4, radius 2.6; pivot on 10-11 (FAF 38): 7.0 up, 4.0 to 13.2 behind (written in front: the engine turns a pivot grab
    // round), radius 3.3. The grounded-target boxes [D]; the thin aerial-target ones are not modelled (the house limit).
    let grab_line = |first: u8, last: u8, y: i32, radius: i32, x1: i32| -> Vec<R> {
        along(first, last, (40, y), (x1, y), radius, 0, 361, 0, 0, 0, 0)
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(35, 0, 0, 255, &grab_line(6, 7, 80, 33, 77)),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(43, 0, 0, 255, &grab_line(9, 10, 70, 26, 94)),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(38, 0, 0, 255, &grab_line(10, 11, 70, 33, 132)),
        HIT_GRAB,
    );
    // Pummel: 1.3%, set knockback 30 (KS 100), hitlag x2.1, radius 4.8 at 7.0 up and 7.0 ahead [D], no clank. The reference hits on
    // frame 1 with FAF 7; ours on frame 2 with FAF 8 (the house rule) [D].
    let mut pummel = reference_pummel(13, u(70), u(70), u(48));
    set_all(&mut pummel, |h| {
        h.clanks = false;
        h.rebounds = false;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::Pummel as usize] = pummel;
    // Throws, hit then throw (the hit on the held fighter on its first frame; the reference's hits last two frames and land once).
    // Forward: a kick on 12 (3%), the throw on 13 (2.5%, 33 degrees, BK 75, KS 85), FAF 38. Back: the kick on 12 (3%), the throw on 14
    // (2.5%, 130 degrees: up and behind, BK 67, KS 110), FAF 39 (conflict 5). Up: the slash on 26 (5%: the thrown fighter always takes
    // the sweet spot), the throw on 28 (2%, 90 degrees, BK 24, KS 230), FAF 50. Down: the hit on 22 (3%), the throw on 24 (3%, 83
    // degrees, BK 60, KS 125), FAF 49. Thrower invincibility, the interrupted rows and the down throw's edge push are not modelled
    // (house limits).
    let fthrow = throw_after_hit(38, (12, 30), r(13, 13, 13, 11, 10, 25, 33, 75, 85, 0, 1));
    let bthrow = throw_after_hit(39, (12, 30), r(14, 14, 13, 11, 10, 25, 130, 67, 110, 0, 1));
    let uthrow = throw_after_hit(50, (26, 50), r(28, 28, 13, 11, 10, 20, 90, 24, 230, 0, 1));
    let dthrow = throw_after_hit(49, (22, 30), r(24, 24, 13, 11, 10, 30, 83, 60, 125, 0, 1));
    for (id, mut mv) in [
        (MoveId::FThrow, fthrow),
        (MoveId::BThrow, bthrow),
        (MoveId::UThrow, uthrow),
        (MoveId::DThrow, dthrow),
    ] {
        for h in &mut mv.hitboxes {
            if h.kind == HIT_PUMMEL {
                h.clanks = false;
                h.rebounds = false;
            }
        }
        w.moves[id as usize] = mv;
    }
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: frames 20-22, 9%, 45 degrees (type 3), BK 90, KS 20, radius 5.0 along 5.0 up from 15.5 ahead to 4.0 behind [D], no
    // clank; intangible through 22 (the first grab; the shared decay after), FAF 56.
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along(20, 22, (155, 50), (-40, 50), 50, 90, 45, 90, 20, 0, 0),
    );
    set_all(&mut ledge, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
    });
    ledge.intangible = 22;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack: behind on 17-18 (15.0 to 4.0 behind), in front on 21-22 (17.0 to 4.0 ahead), 5.0 up [E: the height], radius 5.0;
    // 7% each, 48 degrees (type 0), BK 80, KS 48, shield damage +8 (214 [D]), no clank; intangible through 22, FAF 46. (One get-up slot:
    // the front and back get-ups share every number but the reach, owner decision 36.)
    let mut rows = along(17, 18, (-150, 50), (-40, 50), 50, 70, 48, 80, 48, 0, 0);
    rows.extend(along(21, 22, (170, 50), (40, 50), 50, 70, 48, 80, 48, 0, 1));
    let mut getup = ref_move(46, 0, 0, 255, &rows);
    set_all(&mut getup, |h| {
        h.shield_damage = shield(70, 80);
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
        h.rebounds = false;
    });
    getup.intangible = 22;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}

// ---- The up special: the spin -------------------------------------------------------------------------------------------------

/// The grounded spin's phases: (first, last reference frame, blade damage, tip damage (x0.8), angle, BK, KS) [R].
pub const GROUND_SPIN_PHASES: [(u8, u8, i32, i32, i16, i16, i16); 4] = [
    (7, 8, 140, 112, 45, 60, 88),
    (9, 13, 120, 96, 43, 52, 88),
    (14, 23, 90, 72, 43, 48, 84),
    (24, 39, 70, 56, 361, 20, 82),
];

/// The aerial spin's hits: (first reference frame, damage in tenths, angle, angle type, set knockback, set weight, front) [R]; each
/// lasts two frames.
pub const AIR_SPIN_HITS: [(u8, i32, i16, u8, u16, bool, bool); 8] = [
    (8, 40, ANGLE_AUTOLINK, ANGLE_FORWARD, 107, true, true),
    (12, 20, 145, ANGLE_BACKWARD, 95, false, false),
    (16, 20, ANGLE_AUTOLINK, ANGLE_FORWARD, 107, true, true),
    (19, 20, 145, ANGLE_BACKWARD, 95, false, false),
    (22, 20, 83, ANGLE_FORWARD, 71, true, true),
    (26, 20, 150, ANGLE_BACKWARD, 95, false, false),
    (31, 20, 69, ANGLE_FORWARD, 62, true, true),
    (38, 20, 160, ANGLE_BACKWARD, 95, false, false),
];

/// The aerial spin's rise: the vertical speed set on its first frame the script runs (the reference's initial 2.4, a frame later
/// [E: gap 4, which frame]) and its gravity (the body's 0.096 x 0.55) [R], world units.
pub const SPIN_RISE_SPEED: Fx = Fx::from_ratio(2400, 8000);
pub const SPIN_GRAVITY: Fx = Fx::from_ratio(528, 80_000);

/// The up special, the reference's spin, both versions (owner decision 2).
///
/// * On the ground (up special): **one hit** for the whole move (one hit group) in four timing phases, the blade (`sword2` -2.0 and
///   2.5, radius 4.0 / 3.5) and the weaker tip (8.0, radius 2.8, x0.8 damage): clean 7-8 (14% / 11.2%, 45 degrees, BK 60, KS 88,
///   hitlag x1.4 on the blade), mid 9-13 (12% / 9.6%, 43, BK 52, KS 88), late 14-23 (9% / 7.2%, 43, BK 48, KS 84), latest 24-39 (7% /
///   5.6%, 361, BK 20, KS 82); no clank but on the latest phase; angle type 0 (by position). Held special holds it on frame 3 (move
///   frame 2: the script's stall), up to 60 frames, and the hit then deals `1 + 0.6 x charge / 60` times its damage (`charge_bonus`
///   60) [R]. FAF 77 (74 from release). The blade's sweep round the body is [E] (gap 3): in front on 7-8, behind on 9-10, in front on
///   11-12 and so on, the blade at hip height (7.0 up). Out of a shield it comes out at once (E37).
/// * In the air (`ext0`, the up special's `air_version`): eight spin hits (`AIR_SPIN_HITS`: front 4% / 2% at 367, 83 and 69 with set
///   weight, back 2% at 145 / 145 / 150 / 160 without it (4.0.0), BK 0, KS 100, SDI x0.3) and a finisher on 47 / 48 / 49 (4%, 361,
///   BK 30, KS 200, hitlag x2), every box never clanking. Each back hit carries its target to 10 ahead and 20 above the fighter's feet
///   over 10 frames (a position vector, E34, owner decision 55), round into the next front hit. The spin hits are one hit group
///   re-armed by the script before each; the finisher is its own. The rise: vertical speed 2.4 set on the first frame the script runs
///   [E: frame], gravity x0.55 throughout [R]; air speed x0.5 and the stick's acceleration 0.22 while rising, from frame 48 air speed
///   0.8 and 0.01 [R, the reading that these replace the body's "additional" (stick) acceleration: gap 4]. Ledges in front from frame
///   15, behind from 34; landing lag 30; animation 91, then the special fall (air speed x0.909, stick acceleration 0.025: E11's
///   `fall_speed` / `fall_accel`; landing lag 30).
fn spin(w: &mut Weapon) {
    // The grounded spin.
    let mut rows = Vec::new();
    for f in 7u8..=39 {
        let front = ((f - 7) / 2) % 2 == 0;
        let side = if front { 1 } else { -1 };
        let pose: Pose = (40 * side, 70, 1000 * side, 0);
        let (_, _, blade, tip, angle, bkb, kbg) = GROUND_SPIN_PHASES
            .iter()
            .copied()
            .find(|p| p.0 <= f && f <= p.1)
            .expect("a phase");
        rows.push(sw(f, f, pose, -20, 40, blade, angle, bkb, kbg, 0, 0));
        rows.push(sw(f, f, pose, 25, 35, blade, angle, bkb, kbg, 1, 0));
        rows.push(sw(f, f, pose, 80, 28, tip, angle, bkb, kbg, 2, 0));
    }
    let mut ground = ref_move(77, 0, 0, 255, &rows);
    set_all(&mut ground, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        // (Move frames: reference frames 7-8 are the clean phase, 24 on the latest.)
        let clean = h.start < 8;
        h.hitlag = if clean && h.priority < 2 { 140 } else { 100 };
        let latest = h.start + 1 >= 24;
        h.clanks = latest;
        h.rebounds = latest;
    });
    ground.charge_bonus = 60;
    ground.air_version = Some(SPIN_AIR as u8);
    ground.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_spin.script"),
    ));
    w.moves[MoveId::UpSpecial as usize] = ground;

    // The aerial spin. Front boxes on the root bone [D]; the back hits' 3 and 4 on the sword behind [E].
    let mut rows = Vec::new();
    for (i, &(first, damage, angle, ..)) in AIR_SPIN_HITS.iter().enumerate() {
        let last = first + 1;
        let (a, b, radius) = match i {
            0 => ((145, 100), (80, 100), 60),
            1 => ((-120, 115), (-90, 110), 50),
            2 => ((150, 60), (80, 75), 60),
            3 => ((-125, 115), (-90, 110), 50),
            4 => ((150, 100), (80, 105), 60),
            6 => ((125, 140), (80, 125), 70),
            _ => ((-100, 100), (-135, 100), 50),
        };
        rows.extend(along(
            first, last, a, b, radius, damage, angle, 0, 100, 0, 0,
        ));
    }
    let spins = rows.len();
    for (frame, a, b) in [
        (47u8, (140, 75), (75, 95)),
        (48, (155, 130), (75, 125)),
        (49, (125, 190), (75, 140)),
    ] {
        rows.extend(along(frame, frame, a, b, 75, 40, 361, 30, 200, 0, 1));
    }
    let mut air = ref_move(92, 30, 0, 255, &rows);
    for (i, h) in air.hitboxes.iter_mut().enumerate() {
        h.clanks = false;
        h.rebounds = false;
        if i < spins {
            let &(_, _, _, angle_type, fkv, set_weight, front) = AIR_SPIN_HITS
                .iter()
                .find(|s| s.0 == h.start + 1)
                .expect("a spin hit");
            h.angle_type = angle_type;
            h.fixed_kb = fkv;
            h.set_weight = set_weight;
            h.sdi = 30;
            if !front {
                h.posvec_x = Fx::from_ratio(10, 8);
                h.posvec_y = Fx::from_ratio(20, 8);
                h.posvec_frames = 10;
            }
        } else {
            h.angle_type = ANGLE_FORWARD;
            h.hitlag = 200;
        }
    }
    let su = |n: i32| Fx::from_ratio(n, 8000);
    air.air_blocks = vec![
        AirBlock {
            from: 0,
            to: 46,
            gravity: Some(SPIN_GRAVITY),
            max_fall: None,
            friction: None,
            drift: 100,
            accel: 100,
            speed: Some(su(462)),
            accel_stick: Some(su(220)),
        },
        AirBlock {
            from: 47,
            to: 255,
            gravity: Some(SPIN_GRAVITY),
            max_fall: None,
            friction: None,
            drift: 100,
            accel: 100,
            speed: Some(su(800)),
            accel_stick: Some(su(10)),
        },
    ];
    air.ledge_front_from = 14;
    air.ledge_back_from = 33;
    air.helpless_after = true;
    air.fall_speed = Some(Fx::from_ratio(839_916, 8_000_000));
    air.fall_accel = Some(su(25));
    air.fall_landing_lag = 30;
    air.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_spin_air.script"),
    ));
    w.moves[SPIN_AIR as usize] = air;
}

/// The bow's shot (the neutral special's release, `ext4`).
pub const BOW_FIRE: MoveId = MoveId::Ext4;
/// The boomerang thrown with a smash input (the side special's `smash_version`).
pub const BOOMERANG_FAR: MoveId = MoveId::Ext5;
/// The throw with nothing thrown, while a boomerang is out.
pub const BOOMERANG_FAILED: MoveId = MoveId::Ext6;
/// Catching the boomerang as it comes back.
pub const BOOMERANG_CATCH: MoveId = MoveId::Ext7;

/// A reference speed (thousandths of its units a frame) in world units a frame (/8) [D].
fn rs(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

/// A projectile's hit from a reference row (radius and damage in tenths; the frames do not matter on a projectile).
fn shot_hit(radius: i32, damage: i32, angle: i16, bkb: i16, kbg: i16, angle_type: u8) -> Hitbox {
    let mut h = row(&ru(1, 1, 0, 0, radius, damage, angle, bkb, kbg, 0, 0));
    h.angle_type = angle_type;
    h
}

/// Phase 2: the bow, the boomerang and the sword wave.
fn tools(w: &mut Weapon) {
    bow(w);
    boomerang(w);
    sword_wave(w);
}

/// The bow (the neutral special, 13.0.1; `REFERENCE_DATA.md` "Bow and Arrows"). Frames 1-15 draw it; from frame 16 it holds while
/// special is held, up to 180 frames (then it fires by itself); the arrow leaves on the frame special is let go (frame 16 for a tap),
/// and the move goes on as the release (`ext4`, FAF 30 from the release: 45 for a tap, 81 after 36 held frames) [R]. The charge is the
/// frames held (`buckler_bow.script`); 36 or more is a full one [R]. In the air the same frames with the body's air physics, and
/// touching down carries the move on along the ground [E: gap 7].
///
/// The arrow (E35, a charge-scaled shot): fired 2.5 degrees up at 2.6 to 10 reference units a frame (0.325 to 1.25 [D]) by the
/// charge, gravity 0.054 (0.00675 [D]), 46 frames; floor(4 + 8 x c / 60)% (4 to 12), angle 361 (type forward), BK 10, KS 71, radius
/// 1.35 → 2 [D], hitlag x0.8, reflectable; it does not pierce [R]. Where it leaves the body (0.8 ahead, 1.2 up) is [E]. It lodges in a
/// platform top or a wall it meets at 9 degrees or more for 180 frames, and is gone if it meets one more shallowly (E3b; the 9-degree
/// reading [E: gap 8]); picking it up waits for items (phase 3).
fn bow(w: &mut Weapon) {
    let mut draw = ref_move(45, 0, 0, 255, &[]);
    // (The script decides when the hold ends: the move itself lasts past the 180-frame cap.)
    draw.total_frames = 200;
    draw.land_continue = true;
    draw.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_bow.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = draw;

    let mut fire = ref_move(30, 0, 0, 255, &[]);
    fire.keep_charge = true;
    fire.land_continue = true;
    let mut arrow = shot_hit(135, 40, 361, 10, 71, ANGLE_FORWARD);
    arrow.hitlag = 80;
    fire.projectile = Some(ProjectileSpawn {
        // (Only the draw's script fires it, on the release frame.)
        frame: 255,
        x: Fx::from_ratio(8, 10),
        y: Fx::from_ratio(12, 10),
        speed: rs(2600),
        life: 46,
        hitbox: arrow,
        end_damage: arrow.damage,
        arc: ProjectileArc {
            gravity: rs(54),
            lodge: 180,
            lodge_angle: 9,
            ..ProjectileArc::NONE
        },
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: Some(ChargedShot {
            full: 60,
            snap: 36,
            damage_to: Fx::from_int(12),
            speed_to: rs(10000),
            life_to: 46,
            size_from: 100,
            size_to: 100,
            floor: true,
        }),
        tool: Tool::NONE,
    });
    w.moves[BOW_FIRE as usize] = fire;
}

/// The boomerang's three hits by age (the reference's clean, late and returning hits [R]), type 1 (the way it travels), shield damage
/// from the reference's additive values (-4, -3, -1.5 [R], conflict 6) [D]; `far` the smash input's x1.2 on the first two [R].
fn boomerang_hits(far: bool) -> (Hitbox, Hitbox, Hitbox) {
    let k = if far { 12 } else { 10 };
    let hit = |radius: i32, damage: i32, sd: i32, angle: i16, bkb: i16| {
        let d = damage * k / 10;
        let mut h = shot_hit(radius, d, angle, bkb, 40, ANGLE_BY_TRAVEL);
        h.shield_damage = shield(d, sd);
        h
    };
    let mut returning = shot_hit(36, 30, 70, 50, 40, ANGLE_BY_TRAVEL);
    returning.shield_damage = shield(30, -15);
    (
        hit(40, 80, -40, 70, 80),
        hit(36, 60, -30, 65, 60),
        returning,
    )
}

/// A boomerang throw: `far` the smash input's (3.1 a frame, 167 frames, returning from age 60 [R]) or the tilt's (2.4, 125, from 45
/// [R]). It appears on frame 27, aimed by the stick up to 26 degrees up or down (the script; the 0.1 dead zone is the reading of the
/// reference's "sensitivity" [E: gap 6]), FAF 46 [R]. One at a time (E3d: tool kind 1, limit 1): with one out, the move goes on as the
/// failed throw on its frame 2 [D]. The flight is its projectile script: it slows by 0.047 a frame to 0.3 (the returning hit starts as
/// it gets there [D]), then turns toward its thrower and speeds up to 3 a frame, reaching that over the "distance spent turning back"
/// (16 / 24) [E: gap 6, the reading], homing at up to 0.75 degrees a frame [R]. It bounces off the ground [E: the reference's 16- and
/// 30-degree rules are not modelled]. Coming back (from that age) it is caught by touching its thrower (`ext7` if free) [R]; a clank
/// with an attack drops it (60 frames, -0.15 / +1.25, gravity 0.09, fall 1.4 [R]). Its spawn point (1.0 ahead, 1.1 up) is [E].
fn boomerang_throw(far: bool) -> Move {
    let mut mv = ref_move(46, 0, 0, 255, &[]);
    mv.land_continue = true;
    mv.script = Some(script(
        Kind::Fighter,
        if far {
            include_str!("scripts/buckler_boomerang_far.script")
        } else {
            include_str!("scripts/buckler_boomerang_throw.script")
        },
    ));
    let (clean, late, returning) = boomerang_hits(far);
    let back_from = if far { 60 } else { 45 };
    mv.projectile = Some(ProjectileSpawn {
        // (Only the script fires it, aimed.)
        frame: 255,
        x: Fx::ONE,
        y: Fx::from_ratio(11, 10),
        speed: rs(if far { 3100 } else { 2400 }),
        life: if far { 167 } else { 125 },
        hitbox: clean,
        end_damage: clean.damage,
        arc: ProjectileArc {
            // (It bounces off any ground it meets, keeping its speed: the script steers it on.)
            bounces: 255,
            bounce_min: Fx::ZERO,
            bounce_max: Fx::from_int(4),
            ..ProjectileArc::NONE
        },
        phases: phase_list(&[(10, late), (back_from, returning)]),
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool {
            kind: 1,
            limit: 1,
            catch_from: back_from,
            catch_move: Some(BOOMERANG_CATCH as u8),
            on_clank: Some(Fall {
                life: 60,
                vx: rs(-150),
                vy: rs(1250),
                gravity: rs(90),
                max_fall: rs(1400),
            }),
        },
    });
    mv.projectile_script = Some(script(
        Kind::Projectile,
        if far {
            include_str!("scripts/buckler_boomerang_far_flight.script")
        } else {
            include_str!("scripts/buckler_boomerang_flight.script")
        },
    ));
    mv
}

/// The boomerang (the side special, 13.0.1; `REFERENCE_DATA.md` "Boomerang"): the tilt throw, the smash-input throw (E3d
/// `smash_version`), the failed throw and the catch.
fn boomerang(w: &mut Weapon) {
    let mut tilt = boomerang_throw(false);
    tilt.smash_version = Some(BOOMERANG_FAR as u8);
    w.moves[MoveId::SideSpecial as usize] = tilt;
    w.moves[BOOMERANG_FAR as usize] = boomerang_throw(true);

    // The failed throw: FAF 60 on the ground, 46 in the air [R]. It starts on the throw's frame 2 (a `goto`), so its own FAF is 59 and
    // the aerial one ends on its frame 43 (the script) [D].
    let mut failed = ref_move(59, 0, 0, 255, &[]);
    failed.land_continue = true;
    failed.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/buckler_boomerang_failed.script"),
    ));
    w.moves[BOOMERANG_FAILED as usize] = failed;

    // The catch: FAF 10, no hits [R]; on the ground or in the air alike [E].
    let mut catch = ref_move(10, 0, 0, 255, &[]);
    catch.land_continue = true;
    w.moves[BOOMERANG_CATCH as usize] = catch;
}

/// The sword wave (the forward smash's first hit, only at exactly 0%; `REFERENCE_DATA.md` "The Sword Beam"): it leaves with hit 1 on
/// frame 17, 11 ahead and 6 up of the reference's units (1.375, 0.75 [D]) (`buckler_fsmash.script`), and scales with the smash's
/// charge (E35): 5 to 13%, 2.4 to 3.4 a frame (0.3 to 0.425 [D]), slowed 0.01 a frame (0.00125 [D], E35 drag), 32 to 40 frames, the
/// size x0.8 to x1.1 [R]. Angle 361 (type forward), BK 45, KS 15, radius 1.8 → 2 [D] on a lying capsule 14 units long (half 7 →
/// 0.875 [D], E38), hitlag x1.0, reflectable; no falloff (gap 9) [R].
fn sword_wave(w: &mut Weapon) {
    let mut wave = shot_hit(18, 50, 361, 45, 15, ANGLE_FORWARD);
    wave.half_width = Fx::from_ratio(7, 8);
    w.moves[MoveId::FSmash as usize].projectile = Some(ProjectileSpawn {
        // (Only the script fires it, at 0%.)
        frame: 255,
        x: Fx::from_ratio(11, 8),
        y: Fx::from_ratio(6, 8),
        speed: rs(2400),
        life: 32,
        hitbox: wave,
        end_damage: wave.damage,
        arc: ProjectileArc {
            drag: rs(10),
            ..ProjectileArc::NONE
        },
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: Some(ChargedShot {
            full: 60,
            snap: 0,
            damage_to: Fx::from_int(13),
            speed_to: rs(3400),
            life_to: 40,
            size_from: 80,
            size_to: 110,
            floor: false,
        }),
        tool: Tool::NONE,
    });
}
