//! The Popgun moveset (the scamp body), modelled mechanically on a reference character: phase 1, the body's normals and the
//! specials. A child module of `moves` (it uses that file's private row builders). Data and sources: the twelfth moveset's
//! `REFERENCE_DATA.md` (`docs/movesets/diddy_kong/`), read from ssbwiki.com's raw hitbox tables (13.0.1), ultimateframedata.com and
//! the parameter file via dragdown.wiki.

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

/// Points along a reference capsule from `a` to `b` (reference tenths), about `step` apart: the ends and evenly spaced points
/// between [D].
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

/// The Popgun moveset (the scamp body): quick slaps and flips, a peanut popgun, a banana peel anyone can pick up, a leaping latch and
/// rocket barrels. Every normal follows the reference frame for frame (the twelfth moveset's `REFERENCE_DATA.md` in
/// `docs/movesets/diddy_kong/`, version 13.0.1, read from ssbwiki.com's raw hitbox tables): active frames, FAF, landing lag and
/// autocancel windows, smash charge frames; damage, angles and angle types, base and growth knockback, set knockback and set weight,
/// hitlag and SDI multipliers, shield damage, trip chances, grounded- and aerial-only boxes, no-clank boxes, hit groups. Radii and
/// offsets on the reference's root bone (`top`) are its values converted (x 1.25, rounded half up to a tenth) [D]; boxes on other
/// bones (hands, arms, knees, legs) are placed on our body where that bone is in the move's pose [E].
pub fn popgun() -> Weapon {
    let mut w = claws();
    // Bare hands and feet: the limb throwing an attack can be hit (the reference's hurtboxes on its arms and legs).
    w.limbs = true;
    // The claws' up special and its endings are replaced: those slots are free again.
    for id in [MoveId::Ext0, MoveId::Ext1, MoveId::Ext2] {
        w.moves[id as usize] = Move::empty();
    }
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    banana(&mut w);
    popgun_special(&mut w);
    rocket_barrels(&mut w);
    flip_special(&mut w);
    w
}

fn jabs(w: &mut Weapon) {
    // Jab 1: a slap on frame 3, 2%, angle type 3, hitlag x1.6, 5.5 of the reference's units up: 361 (BK 25, KS 25, radius 1.8) at 4.0
    // and 7.0 ahead; the far box (10.0 ahead, radius 2.0) pulls a fighter in at 180 degrees, BK 20, KS 15 (the reference's id 2,
    // fighters only; its id 3, the same box at 361 for objects, is not modelled) [R]. FAF 20. The continuable window is frames 6-30:
    // a press from frame 6 carries on into jab 2 (`next_from`), and past the FAF to frame 30 a press is jab 2, not a new jab
    // (owner decision 58). Attack held (no new press) throws jab 1 again from frame 10 (the reference's consecutive jab: the loop
    // marker on frames 6-9, ssbwiki.com) [R: held; D: the frame].
    let mut jab1 = ref_move(
        20,
        0,
        0,
        255,
        &[
            ru(3, 3, 40, 55, 18, 20, 361, 25, 25, 0, 0),
            ru(3, 3, 70, 55, 18, 20, 361, 25, 25, 1, 0),
            ru(3, 3, 100, 55, 20, 20, 180, 20, 15, 2, 0),
        ],
    );
    set_all(&mut jab1, |h| {
        h.hitlag = 160;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab1, MoveId::Jab2, 6);
    late_chain(&mut jab1, MoveId::Jab2, 30);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: the other hand on frame 5, 1.5%, type 3, hitlag x1.2, 5.5 up: 361 (BK 20, KS 25) at 4.0 (radius 2.3) and 7.0 ahead (2.6);
    // the far box (11.3 ahead, radius 2.8) at 180 degrees, BK 20, KS 15 (fighters only) [R]. FAF 21. Continuable 7-30 into jab 3
    // (`next_from`; past the FAF, owner decision 58).
    let mut jab2 = ref_move(
        21,
        0,
        0,
        255,
        &[
            ru(5, 5, 40, 55, 23, 15, 361, 20, 25, 0, 0),
            ru(5, 5, 70, 55, 26, 15, 361, 20, 25, 1, 0),
            ru(5, 5, 113, 55, 28, 15, 180, 20, 15, 2, 0),
        ],
    );
    set_all(&mut jab2, |h| {
        h.hitlag = 120;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab2, MoveId::Jab3, 7);
    late_chain(&mut jab2, MoveId::Jab3, 30);
    w.moves[MoveId::Jab2 as usize] = jab2;

    // Jab 3: a side kick on frame 5, 4%, 361 (type 3), BK 60, KS 80, hitlag x2: radius 2.6 at 4.0 ahead / 6.0 up, 3.0 at 7.5 / 6.3, 3.8
    // at 12.0 / 6.8 [R]. FAF 35.
    let mut jab3 = ref_move(
        35,
        0,
        0,
        255,
        &[
            ru(5, 5, 40, 60, 26, 40, 361, 60, 80, 0, 0),
            ru(5, 5, 75, 63, 30, 40, 361, 60, 80, 1, 0),
            ru(5, 5, 120, 68, 38, 40, 361, 60, 80, 2, 0),
        ],
    );
    set_all(&mut jab3, |h| {
        h.hitlag = 200;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::Jab3 as usize] = jab3;
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a two-handed slap, 35 degrees (type 3), BK 60, KS 60 [R]. Frames 10-11: both hands (radius 3.8) and two boxes on
    // the root, 4.0 at 7.5 ahead / 6.0 up and 3.0 at 1.5 / 6.0, 10%; frame 12 the hands alone, 10%; frames 13-16 the hands, 7% [R].
    // Where the hands are is ours, for each aim [E]: straight out at chest height, raised, or low (the root boxes are the same in
    // every aim, as the reference's). One hit. FAF 37.
    let mut ftilt = ref_move(37, 0, 0, 255, &[]);
    for (aim, right, left) in [
        (AIM_UP, (100, 100), (85, 110)),
        (AIM_STRAIGHT, (110, 60), (90, 75)),
        (AIM_DOWN, (105, 30), (90, 40)),
    ] {
        let mut rows = Vec::new();
        for (first, last, damage) in [(10u8, 11u8, 100), (12, 12, 100), (13, 16, 70)] {
            rows.push(ru(
                first, last, right.0, right.1, 38, damage, 35, 60, 60, 0, 0,
            ));
            rows.push(ru(
                first, last, left.0, left.1, 38, damage, 35, 60, 60, 1, 0,
            ));
        }
        let mut mv = ref_move(37, 0, 0, 255, &rows);
        set_all(&mut mv, |h| h.aim = aim);
        ftilt.hitboxes.extend(mv.hitboxes);
    }
    ftilt.hitboxes.extend(
        ref_move(
            37,
            0,
            0,
            255,
            &[
                ru(10, 11, 75, 60, 40, 100, 35, 60, 60, 2, 0),
                ru(10, 11, 15, 60, 30, 100, 35, 60, 60, 3, 0),
            ],
        )
        .hitboxes,
    );
    set_all(&mut ftilt, |h| h.angle_type = ANGLE_FORWARD);
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: an arm swept up over the head on frames 6-11, 6%, 92 degrees (type 0), BK 28, KS 150, SDI x1.5 [R]: the arm (two boxes,
    // radius 4.0) and the shoulder (3.8) [R radii; E: placed over the head, the hand in front and above]. FAF 30.
    let mut utilt = ref_move(
        30,
        0,
        0,
        255,
        &[
            ru(6, 11, 30, 150, 40, 60, 92, 28, 150, 0, 0),
            ru(6, 11, -10, 165, 40, 60, 92, 28, 150, 1, 0),
            ru(6, 11, 10, 110, 38, 60, 92, 28, 150, 2, 0),
        ],
    );
    set_all(&mut utilt, |h| {
        h.sdi = 150;
        h.angle_type = ANGLE_BY_POSITION;
    });
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a low slap on frames 4-5, 5.5%, BK 15, KS 75, a 20% trip chance, type 0: 60 degrees at 16.0 ahead / 1.8 up (radius
    // 3.0), 50 degrees at 8.5 / 3.0 (3.5) [R]. FAF 19 (ultimateframedata.com and dragdown.wiki; ssbwiki.com's frame strip reads 20:
    // conflict 1). (At these angles it lifts a grounded fighter, so it trips only on a slope, as the reference.)
    let mut dtilt = ref_move(
        19,
        0,
        0,
        255,
        &[
            ru(4, 5, 160, 18, 30, 55, 60, 15, 75, 0, 0),
            ru(4, 5, 85, 30, 35, 55, 50, 15, 75, 1, 0),
        ],
    );
    set_all(&mut dtilt, |h| {
        h.trip = 20;
        h.angle_type = ANGLE_BY_POSITION;
    });
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a three-hit cartwheel, every box type 3, never clanking (`clang=f rebound=f`) [R]. Hit 1 on frames 8-9, 2%, set
    // weight, BK 0, KS 100, hitlag x0.5: against grounded fighters 35 degrees, set knockback 60, against airborne ones 10 degrees, set
    // knockback 45 (radius 3.5 on the root from 8.5 ahead / 11.0 up to 7.5 / 9.5), and 43 degrees, set knockback 60, at 7.5 / 5.5 (radius
    // 3.5, all fighters). Hit 2 on 16-17, 2%, set weight, hitlag x0.8: grounded 25 degrees, set knockback 45, airborne 10 degrees, 45
    // (radius 4.0 from 8.0 / 11.5 to 7.0 / 10.0), and 40 degrees, 70 at 7.0 / 6.5. Hit 3 on 22-23: 3%, 83 degrees, BK 80, KS 90, radius
    // 5.5 from 8.0 ahead / 11.0 up down to 6.0 [R]. FAF 41.
    let mut rows = along_rows(8, 9, (85, 110), (75, 95), 35, 20, 35, 0, 100, 0, 0);
    let air1 = rows.len();
    rows.extend(along_rows(
        8,
        9,
        (85, 110),
        (75, 95),
        35,
        20,
        10,
        0,
        100,
        1,
        0,
    ));
    let all1 = rows.len();
    rows.push(ru(8, 9, 75, 55, 35, 20, 43, 0, 100, 2, 0));
    let ground2 = rows.len();
    rows.extend(along_rows(
        16,
        17,
        (80, 115),
        (70, 100),
        40,
        20,
        25,
        0,
        100,
        0,
        1,
    ));
    let air2 = rows.len();
    rows.extend(along_rows(
        16,
        17,
        (80, 115),
        (70, 100),
        40,
        20,
        10,
        0,
        100,
        1,
        1,
    ));
    let all2 = rows.len();
    rows.push(ru(16, 17, 70, 65, 40, 20, 40, 0, 100, 2, 1));
    let third = rows.len();
    rows.extend(along_rows(
        22,
        23,
        (80, 110),
        (80, 60),
        55,
        30,
        83,
        80,
        90,
        0,
        2,
    ));
    let mut dash = ref_move(41, 0, 0, 255, &rows);
    for (i, h) in dash.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
        if i < third {
            h.set_weight = true;
            h.hitlag = if i < ground2 { 50 } else { 80 };
            h.fixed_kb = match i {
                _ if i < air1 => 60,
                _ if i < all1 => 45,
                _ if i < ground2 => 60,
                _ if i < all2 => 45,
                _ => 70,
            };
            h.targets = match i {
                _ if i < air1 => TARGETS_GROUNDED,
                _ if i < all1 => TARGETS_AERIAL,
                _ if i < ground2 => TARGETS_ALL,
                _ if i < air2 => TARGETS_GROUNDED,
                _ if i < all2 => TARGETS_AERIAL,
                _ => TARGETS_ALL,
            };
        }
    }
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash: two slaps. Charges on frame 4 [R]. Hit 1, every box 5%, set weight, BK 0, KS 100, type 3 [R]: on frame 12 the root
    // box (radius 4.2 at 9.5 ahead / 9.7 up, airborne fighters only, 10 degrees, set knockback 20) and the arm (radius 4.0, 45 degrees,
    // set knockback 35), the arm's grounded-only box (4.0, 80 degrees, 25) and the shoulder (3.5, 22 degrees, 42); on frame 13 the root
    // box moves to 9.2 ahead / 13.5 up at 0 degrees (set knockback 20), the arm boxes stay. Hit 2 on frames 21-23: the other arm, 11%
    // (radius 4.6, 46 degrees; 4.0, 50 degrees) and its shoulder 9% (3.5, 60 degrees), BK 40, KS 116, hitlag x1.2, type 3 [R]. The arms
    // and shoulders are placed reaching forward at chest height [E]. FAF 51.
    let mut rows = Vec::new();
    for (frame, root) in [(12u8, (95, 97, 42, 10)), (13, (92, 135, 45, 0))] {
        rows.push(ru(
            frame, frame, root.0, root.1, root.2, 50, root.3, 0, 100, 0, 0,
        ));
        rows.push(ru(frame, frame, 75, 70, 40, 50, 45, 0, 100, 1, 0));
        rows.push(ru(frame, frame, 75, 45, 40, 50, 80, 0, 100, 2, 0));
        rows.push(ru(frame, frame, 40, 70, 35, 50, 22, 0, 100, 3, 0));
    }
    let hit2 = rows.len();
    rows.push(ru(21, 23, 100, 65, 46, 110, 46, 40, 116, 0, 1));
    rows.push(ru(21, 23, 75, 65, 40, 110, 50, 40, 116, 1, 1));
    rows.push(ru(21, 23, 40, 70, 35, 90, 60, 40, 116, 2, 1));
    let mut fsmash = ref_move(51, 0, 0, 255, &rows);
    for (i, h) in fsmash.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        if i < hit2 {
            h.set_weight = true;
            h.fixed_kb = match (h.priority, h.angle) {
                (0, _) => 20,
                (1, _) => 35,
                (2, _) => 25,
                _ => 42,
            };
            h.targets = match h.priority {
                0 => TARGETS_AERIAL,
                2 => TARGETS_GROUNDED,
                _ => TARGETS_ALL,
            };
        } else {
            h.hitlag = 120;
        }
    }
    fsmash.charge_at = Some(3);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: a three-part flip upward, every box 2.5% set weight (BK 0, KS 100) until the last, type 0 [R]. Charges on frame 1 [R].
    // Hit 1: frame 5 the arm against grounded fighters 110 degrees (set knockback 110), against airborne ones 125 (30) (radius 5.0), the
    // shoulder 95 (50) (3.9); frames 6-8 the arm 130 (25) / 140 (15), the shoulder 95 (50). Hit 2: frames 12-14 the leg 105 (65) (3.9)
    // and the knee 125 (45) (5.0); frame 15 the leg 105 (65), the knee 115 (80). Hit 3 on frames 19-23: 6%, 85 degrees (type 3), BK 55,
    // KS 131, the arm (6.5) and shoulder (4.5) [R]. The limbs are placed over the body as it flips up [E]. FAF 54.
    let mut rows = vec![
        ru(5, 5, 40, 70, 50, 25, 110, 0, 100, 0, 0),
        ru(5, 5, 40, 70, 50, 25, 125, 0, 100, 1, 0),
        ru(5, 5, 20, 80, 39, 25, 95, 0, 100, 2, 0),
        ru(6, 8, 30, 110, 50, 25, 130, 0, 100, 0, 0),
        ru(6, 8, 30, 110, 50, 25, 140, 0, 100, 1, 0),
        ru(6, 8, 20, 80, 39, 25, 95, 0, 100, 2, 0),
        ru(12, 14, 10, 150, 39, 25, 105, 0, 100, 0, 1),
        ru(12, 14, 15, 110, 50, 25, 125, 0, 100, 1, 1),
        ru(15, 15, 10, 150, 39, 25, 105, 0, 100, 0, 1),
        ru(15, 15, 15, 110, 50, 25, 115, 0, 100, 1, 1),
    ];
    let last = rows.len();
    rows.push(ru(19, 23, 0, 186, 65, 60, 85, 55, 131, 0, 2));
    rows.push(ru(19, 23, 0, 130, 45, 60, 85, 55, 131, 1, 2));
    let mut usmash = ref_move(54, 0, 0, 255, &rows);
    for (i, h) in usmash.hitboxes.iter_mut().enumerate() {
        if i < last {
            h.set_weight = true;
            h.angle_type = ANGLE_BY_POSITION;
            h.fixed_kb = match (h.group, h.angle) {
                (0, 110) => 110,
                (0, 125) => 30,
                (0, 95) => 50,
                (0, 130) => 25,
                (0, 140) => 15,
                (1, 105) => 65,
                (1, 125) => 45,
                _ => 80,
            };
            if h.group == 0 {
                h.targets = match h.priority {
                    0 => TARGETS_GROUNDED,
                    1 => TARGETS_AERIAL,
                    _ => TARGETS_ALL,
                };
            }
        } else {
            h.angle_type = ANGLE_FORWARD;
        }
    }
    usmash.charge_at = Some(0);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: slaps in front then behind, on the root, 3.7 of the reference's units up, type 0 [R]. Charges on frame 2. Hit 1 on
    // frames 7-8: 12%, 33 degrees, BK 46, KS 77, radius 3.9 at 13.8 ahead and 3.3 at 8.8. Hit 2 on 12-13: 15%, 35 degrees, BK 38, KS 76,
    // radius 3.9 at 10.5 behind and 3.3 at 5.5 behind [R]. FAF 51 (ultimateframedata.com, dragdown.wiki; the frame strip and the 8.0.0
    // notes read 50: conflict 2).
    let mut dsmash = ref_move(
        51,
        0,
        0,
        255,
        &[
            ru(7, 8, 138, 37, 39, 120, 33, 46, 77, 0, 0),
            ru(7, 8, 88, 37, 33, 120, 33, 46, 77, 1, 0),
            ru(12, 13, -105, 37, 39, 150, 35, 38, 76, 0, 1),
            ru(12, 13, -55, 37, 33, 150, 35, 38, 76, 1, 1),
        ],
    );
    set_all(&mut dsmash, |h| h.angle_type = ANGLE_BY_POSITION);
    dsmash.charge_at = Some(1);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: a spin with arms and knees out, frames 8-20, 6%, 80 degrees (type 0), BK 60, KS 75, radius 3.5 on both arms and both
    // knees [R]; the arms 4.5 ahead and behind at 7.0 up, the knees 3.0 ahead and behind at 2.5 up [E]. Landing lag 8, autocancels on
    // frames 1-3 and from 27, FAF 49.
    let mut nair = ref_move(
        49,
        8,
        3,
        27,
        &[
            ru(8, 20, 45, 70, 35, 60, 80, 60, 75, 0, 0),
            ru(8, 20, -45, 70, 35, 60, 80, 60, 75, 1, 0),
            ru(8, 20, 30, 25, 35, 60, 80, 60, 75, 2, 0),
            ru(8, 20, -30, 25, 35, 60, 80, 60, 75, 3, 0),
        ],
    );
    set_all(&mut nair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: a flipping kick, 361 (type 3), BK 30, KS 86: clean on frames 6-9 10% (the foot and the other knee, radius 5.0), late
    // on 10-16 8% (radius 2.7) [R]; the foot 7.5 ahead / 4.0 up, the knee 4.0 / 5.5 [E]. Landing lag 16, autocancels on frames 1-2 and
    // from 26, FAF 47.
    let mut fair = ref_move(
        47,
        16,
        2,
        26,
        &[
            ru(6, 9, 75, 40, 50, 100, 361, 30, 86, 0, 0),
            ru(6, 9, 40, 55, 50, 100, 361, 30, 86, 1, 0),
            ru(10, 16, 75, 40, 27, 80, 361, 30, 86, 0, 0),
            ru(10, 16, 40, 55, 27, 80, 361, 30, 86, 1, 0),
        ],
    );
    set_all(&mut fair, |h| h.angle_type = ANGLE_FORWARD);
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: a kick behind on frames 5-8, 9%, 361 (type 4, backward), BK 30, KS 102: the knee's two boxes (radius 5.3 and 4.7) and the
    // leg (4.3) [R], reaching 7.5, 4.5 and 2.0 behind at about 5.0 up [E]. Landing lag 7, autocancels on frames 1-2 and from 20, FAF 31.
    let mut bair = ref_move(
        31,
        7,
        2,
        20,
        &[
            ru(5, 8, -75, 50, 53, 90, 361, 30, 102, 0, 0),
            ru(5, 8, -45, 55, 47, 90, 361, 30, 102, 1, 0),
            ru(5, 8, -20, 60, 43, 90, 361, 30, 102, 2, 0),
        ],
    );
    set_all(&mut bair, |h| h.angle_type = ANGLE_BACKWARD);
    w.moves[MoveId::BAir as usize] = bair;

    // Up air: a flip kick overhead on frames 4-7, 7%, 76 degrees (type 0), BK 57, KS 89, the knee's two boxes (radius 5.0 and 4.5) [R],
    // over the head [E]. Landing lag 9, autocancels from frame 27 (none at the start) [R]. FAF 36 (the 3.1.0 notes and ssbwiki.com's
    // frame strip; ultimateframedata.com and dragdown.wiki read 37: conflict 3).
    let mut uair = ref_move(
        36,
        9,
        0,
        27,
        &[
            ru(4, 7, 10, 150, 50, 70, 76, 57, 89, 0, 0),
            ru(4, 7, 5, 110, 45, 70, 76, 57, 89, 1, 0),
        ],
    );
    set_all(&mut uair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: a stomp below, on the root, type 0 [R]: on frame 15 13% (radius 4.5 at 6.5 below the feet, BK 25, KS 110) and 10% (6.7 at
    // 3.0 below, BK 20, KS 100), both 270 degrees; on 16 the 13% box alone. Landing lag 17, autocancels on frames 1-14 and from 35, FAF
    // 47.
    let mut dair = ref_move(
        47,
        17,
        14,
        35,
        &[
            ru(15, 16, 0, -65, 45, 130, 270, 25, 110, 0, 0),
            ru(15, 15, 0, -30, 67, 100, 270, 20, 100, 1, 0),
        ],
    );
    set_all(&mut dair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::DAir as usize] = dair;
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: one hand reaching ahead, the reference's grounded-target box (id 0) swept along its capsule [R]: standing on frames 7-8
    // (radius 3.7, 4.0 to 7.8 ahead, 7.0 up), FAF 37; from a dash on 10-11 (3.0, 4.0 to 9.5, 6.0 up), FAF 45; pivot on 11-12 (3.7, 4.0 to
    // 13.6, 6.0 up; written in front, since the engine turns a pivot grab round), FAF 40. (The aerial-target box is not modelled: grabs
    // catch only fighters on the ground here.)
    let grab = |faf: u8, first: u8, last: u8, y: i32, to: i32, radius: i32| {
        with_kind(
            ref_move(
                faf,
                0,
                0,
                255,
                &along_rows(first, last, (40, y), (to, y), radius, 0, 361, 0, 0, 0, 0),
            ),
            HIT_GRAB,
        )
    };
    w.moves[MoveId::Grab as usize] = grab(37, 7, 8, 70, 78, 37);
    w.moves[MoveId::DashGrab as usize] = grab(45, 10, 11, 60, 95, 30);
    w.moves[MoveId::PivotGrab as usize] = grab(40, 11, 12, 60, 136, 37);

    // Pummel: 1%, set knockback 30 (KS 100), 361 (type 3), hitlag x1.7, radius 5.0 at 9.0 ahead and 8.0 up, never clanks [R]. The
    // reference's hit on frame 1 is frame 2 here (a pummel's hit cannot start on its first frame) with the same 6 frames after it: FAF 8
    // [D] (the reference's 7; its hitlag freezes the move, so it is not in the FAF).
    let mut pummel = with_kind(
        ref_move(8, 0, 0, 255, &[ru(2, 2, 90, 80, 50, 10, 361, 0, 100, 0, 0)]),
        HIT_PUMMEL,
    );
    set_all(&mut pummel, |h| {
        h.fixed_kb = 30;
        h.hitlag = 170;
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
    });
    w.moves[MoveId::Pummel as usize] = pummel;

    // Throws (the thrown fighter's 3% hit on bystanders and the thrower's invincibility are not modelled: house limits). Forward: on
    // frame 21, 9% at 46 degrees, BK 69, KS 65, no hitlag, FAF 32. Back: on 18, 12% at 42 degrees behind, BK 60, KS 55, FAF 52 (the
    // thrower keeps its facing [E: not published]). Up: a 1% hit on the held fighter on frame 17 (the reference's hitbox over the head on
    // 17-18: 75 degrees, BK 50, KS 120, never clanking; here a hit on the held fighter, as the house's pre-hits), then the throw on 19, 4%
    // at 87 degrees, BK 70, KS 72, no hitlag, FAF 27 (the 3.1.0 notes, ultimateframedata.com, dragdown.wiki; the frame strip reads 28).
    // Down: on 22, 7% at 106 degrees, BK 85, KS 59, no hitlag, FAF 32 [R].
    let throw =
        |faf: u8, frame: u8, x: i32, damage: i32, angle: i16, bkb: i16, kbg: i16, lag: u16| {
            let mut m = with_kind(
                ref_move(
                    faf,
                    0,
                    0,
                    255,
                    &[r(frame, frame, x, 11, 10, damage, angle, bkb, kbg, 0, 0)],
                ),
                HIT_THROW,
            );
            m.hitboxes[0].hitlag = lag;
            m
        };
    w.moves[MoveId::FThrow as usize] = throw(32, 21, 13, 90, 46, 69, 65, 0);
    w.moves[MoveId::BThrow as usize] = throw(52, 18, -13, 120, 42, 60, 55, 100);
    let mut uthrow = throw_after_hit(27, (17, 10), r(19, 19, 13, 11, 10, 40, 87, 70, 72, 0, 1));
    uthrow.hitboxes[0].clanks = false;
    uthrow.hitboxes[0].rebounds = false;
    uthrow.hitboxes[1].hitlag = 0;
    w.moves[MoveId::UThrow as usize] = uthrow;
    w.moves[MoveId::DThrow as usize] = throw(32, 22, 13, 70, 106, 85, 59, 0);
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: frames 23-25, 9%, 45 degrees (type 3), BK 90, KS 20, shield damage +1 (111 [D]), never clanks; intangible through
    // frame 25 (on the first grab: 1-20 and 1-12 on the second and third, the shared 80% / 50%), FAF 56. Its swept box (19.0 ahead to
    // 3.5, 5.0 up, radius 4.0) as circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along_rows(23, 25, (190, 50), (35, 50), 40, 90, 45, 90, 20, 0, 0),
    );
    set_all(&mut ledge, |h| {
        h.shield_damage = shield(90, 10);
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
    });
    ledge.intangible = 25;
    w.moves[MoveId::LedgeAttack as usize] = ledge;

    // Get-up attack (lying face up, the reference's "back" floor attack): in front on frames 15-16 (12.0 to 5.0 of the reference's units
    // ahead), then behind on 22-23 (11.0 to 5.0 behind), 5.0 up, radius 5.0; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8
    // (214 [D]), never clanks; intangible through frame 23, FAF 46. (The face-down and trip get-up attacks have no slot of their own.)
    let mut rows = along_rows(15, 16, (120, 50), (50, 50), 50, 70, 48, 80, 48, 0, 0);
    rows.extend(along_rows(
        22,
        23,
        (-110, 50),
        (-50, 50),
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
        h.rebounds = false;
    });
    getup.intangible = 23;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}

// ---- The down special: the banana peel ---------------------------------------------------------------------------------------------

/// The down special's versions (the script `popgun_peel.script` goes on as one on frame 1): with an item already in the hand (it is
/// thrown first), and the toss that makes no peel (one is out, or the wait since the last is not over), with or without an item.
pub const PEEL_WITH_ITEM: MoveId = MoveId::Ext0;
pub const PEEL_FAILS: MoveId = MoveId::Ext1;
pub const PEEL_FAILS_WITH_ITEM: MoveId = MoveId::Ext2;
/// The weapon's items: the banana peel.
pub const BANANA_ITEM: u8 = 0;
/// Its kind (what `owns(kind)` and the `holding` register read).
pub const BANANA_KIND: u8 = 9;
/// The wait before the next peel after one is tossed (the reference's 85 frames since 11.0.0: 12, 61 in 8.0.0, + 24) [R].
pub const PEEL_WAIT: u8 = 85;

/// A reference speed or length (thousandths of its units) in world units: / 8 [D].
fn rs(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

/// The peel's toss (13.0.1, ssbwiki.com `Banana_Peel_(move)`): 105 degrees (up and behind), 2.2 of the reference's units a frame, 3.036
/// for a smash input, times the toss strength (the scamp's 0.966667) [R]; on `frame`, with an item held before the move on `again`.
fn peel_toss(frame: u8, again: u8, wait: u8) -> Toss {
    Toss {
        frame,
        again,
        angle: 105,
        speed: rs(2200),
        smash_speed: rs(3036),
        wait,
    }
}

/// The down special: the banana peel (13.0.1; ssbwiki.com's raw down special page, `Banana_Peel_(move)`, `Banana_Peel`, ultimateframedata.com).
///
/// * The peel appears in the hand on frame 4 and is tossed over the shoulder on frame 20 (`Toss`, E47); FAF 40 [R]. An item already in
///   the hand is tossed first, on frame 14, the same way [R] (`PEEL_WITH_ITEM`: the peel then appears as the other is let go [E: one
///   hand here]).
/// * One peel out at a time, and the next only 85 frames after the last was tossed [R]: otherwise the toss makes no peel
///   (`PEEL_FAILS`; with an item held, it is still tossed: `PEEL_FAILS_WITH_ITEM`) and has more ending lag [R], FAF 50 [E: the
///   animation's 50 frames].
/// * The peel is an item anyone can pick up and throw (E4, see `banana_item`).
/// * In the air the same frames, on the body's own air physics [E]; touching down carries the move on [E].
fn banana(w: &mut Weapon) {
    let mut peel = ref_move(40, 0, 0, 255, &[]);
    peel.pull = Some((3, BANANA_ITEM));
    peel.toss = Some(peel_toss(19, 255, PEEL_WAIT));
    peel.land_continue = true;
    peel.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_peel.script"),
    ));
    w.moves[MoveId::DownSpecial as usize] = peel;

    // With an item in the hand: it is tossed on the special's frame 14 (the child's move frame 12) and the peel made then, the peel
    // tossed on 20 (the child's 18); FAF 40 (the child's 39).
    let mut with_item = ref_move(39, 0, 0, 255, &[]);
    with_item.pull = Some((12, BANANA_ITEM));
    with_item.toss = Some(peel_toss(18, 12, PEEL_WAIT));
    with_item.land_continue = true;
    w.moves[PEEL_WITH_ITEM as usize] = with_item;

    // No peel: the throwing motion and a shrug, FAF 50 [E] (the child's 49); an item held is still tossed on 14.
    let mut fails = ref_move(49, 0, 0, 255, &[]);
    fails.land_continue = true;
    fails.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_shrug.script"),
    ));
    w.moves[PEEL_FAILS as usize] = fails;
    let mut fails_item = ref_move(49, 0, 0, 255, &[]);
    fails_item.toss = Some(peel_toss(12, 255, 0));
    fails_item.land_continue = true;
    w.moves[PEEL_FAILS_WITH_ITEM as usize] = fails_item;

    w.items = vec![banana_item()];
}

/// The banana peel (an item: E4, the reference's, 13.0.1).
///
/// * Thrown (by its move's toss or by anyone's item throw): 0% base, the base damage modifier +1 and 1.7 per unit of throw speed (the
///   reference's `1 + 1.7 x toss speed` for the move's toss), the throws 0.85x as fast [R] (ssbwiki.com `Item_throw`); its hit (radius
///   3.0, 361 degrees, BK 50, KS 0, by travel) trips a fighter on the ground (the slip effect: a hard trip, with hitlag) and hits one in
///   the air [R]; it never clanks (transcendent) [R]. On a fighter it bounces off high [R: "a high arc"]; its numbers [E].
/// * Lying on the ground: a fighter (not its last thrower [R]) whose legs touch it (radius 3.0 [R], at its foot [E]) slips (a soft trip:
///   no hitlag, get-ups from 30, up by itself on 90 [R: ssbwiki.com's trip pages]), whatever it is doing on the ground, shield too
///   [R: `blockable=no`]; the peel flies up and away [E: the numbers].
/// * Two uses: a fighter it trips (lying or thrown) and a fighter's throw each spend one, then it is gone [R: "two slips and/or
///   throws"]; its own move's toss does not count [D]. No time limit [R: none published; the sources describe removal by use only].
/// * Its fall: 2.2 at most [R], gravity 0.1 [E]; it slides to a stop on landing (friction 0.06 [E]); a box 2.5 wide and 1.5 tall, picked
///   up within 4.0 [E].
pub fn banana_item() -> ItemSpec {
    let mut hit = row(&ru(1, 1, 0, 0, 30, 0, 361, 50, 0, 0, 0));
    hit.angle_type = ANGLE_BY_TRAVEL;
    hit.slip = SLIP_HARD;
    hit.clanks = false;
    hit.rebounds = false;
    let mut it = ItemSpec::with_hit(hit);
    it.kind = BANANA_KIND;
    it.gravity = rs(100);
    it.max_fall = rs(2200);
    it.friction = rs(60);
    it.half_width = rs(1250);
    it.half_height = rs(750);
    it.pick_width = rs(4000);
    it.pick_height = rs(4000);
    it.throw_mul = Fx::from_ratio(85, 100);
    it.damage_add = Fx::ONE;
    // (1.7 a reference unit a frame is 13.6 a world unit a frame [D].)
    it.damage_per_speed = Fx::from_ratio(136, 10);
    // (It never bounces off the floor: it lands and slides [E].)
    it.bounce_min = Fx::from_int(8);
    it.fighter_keep = (10, 0);
    it.fighter_lift = rs(2000);
    it.uses = 2;
    let mut trap = row(&ru(1, 1, 0, 0, 30, 0, 361, 50, 0, 0, 0));
    trap.slip = SLIP_SOFT;
    trap.clanks = false;
    trap.rebounds = false;
    it.trap = Some(Trap {
        hit: trap,
        kick_x: rs(800),
        kick_y: rs(1500),
    });
    it
}

// ---- The neutral special: the peanut popgun ----------------------------------------------------------------------------------------

/// The popgun's shot (the peanut flies from the move's first frame, on the release) and its overcharge (the explosion).
pub const POPGUN_FIRE: MoveId = MoveId::Ext3;
pub const POPGUN_MISFIRE: MoveId = MoveId::Ext4;
/// Frames of charge for a full charge: from the reference's frame 16 (the uncharged shot) to its 94 (full) [R: 3.1.0].
pub const POPGUN_FULL: u8 = 78;

/// A projectile's hit from a reference row (radius and damage in tenths; the frames do not matter on a projectile).
fn shot_hit(radius: i32, damage: i32, angle: i16, bkb: i16, kbg: i16, angle_type: u8) -> Hitbox {
    let mut h = row(&ru(1, 1, 0, 0, radius, damage, angle, bkb, kbg, 0, 0));
    h.angle_type = angle_type;
    h
}

/// The neutral special: the peanut popgun (13.0.1; ssbwiki.com's raw neutral special page and `Peanut_Popgun`, the 3.1.0 notes,
/// ultimateframedata.com, dragdown.wiki).
///
/// * Special pressed and let go before frame 16: the peanut leaves on frame 16, uncharged [R]. Held at 16 it charges while held, a full
///   charge on frame 94 [R: 3.1.0, 121 before], and fires on the frame special is let go [R]; still held on frame 97 the gun goes off
///   in the scamp's hands (`POPGUN_MISFIRE`) [R: 3.1.0, 123 before]. The charge is not kept [R]; shield (or an air dodge) cancels it from
///   frame 14 [R]. (Held on frame 96 the misfire is certain: the move goes on as it there, its explosion on 97 [D].)
/// * After the shot, control returns 33 frames after the frame it left (FAF 49 for the uncharged shot) [R: 48 total; D: the charged
///   shots' FAF by the same 33].
/// * The peanut: 4.3% uncharged to 13.8% at a full charge [R: 3.1.0], 70 degrees (type 3), BK 40, KS 70, radius 2.4, hitlag x0.2,
///   shield damage -1 (77% of the uncharged hit [D]; the same percent at every charge [E]), never clanks (transcendent), reflectable
///   [R]. Faster and flatter with the charge [R: qualitative]: 1.6 of the reference's units a frame 35 degrees up uncharged, to 3.6 at 8
///   degrees full [E], falling under 0.08 [E] to 2.0 [R: the peanut's falling speed], gone on the ground [R: its bounce is only for show]
///   or after 80 frames [E]. (The reference's later, taller hit (`-1.1 to 1.9` up) and its melee hit at the muzzle, 2% (3% in the
///   changelist) with no published knockback, are not modelled; a peanut cannot be caught as an item here: gaps.)
/// * The misfire: 23%, 361 (type 0), BK 30, KS 92, radius 8.2 at 7.0 ahead / 8.5 up, hitlag x1.5, never clanks, on its first frame
///   [R]; FAF 105 frames after it [D: ultimateframedata.com's 228 total less its old 123], no damage to the scamp [E].
/// * In the air the same frames on the body's own air physics [E]; touching down carries the move on [E].
fn popgun_special(w: &mut Weapon) {
    let mut draw = ref_move(49, 0, 0, 255, &[]);
    // (The script decides when the hold ends: the move itself lasts past the overcharge.)
    draw.total_frames = 120;
    draw.land_continue = true;
    draw.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_draw.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = draw;

    // The shot: reached on the frame the peanut leaves (`G`), it ends so control returns on the special's frame G + 33 (the child
    // starting on the special's G - 1 in the reference's numbering: FAF 34 in its own) [D].
    let mut fire = ref_move(34, 0, 0, 255, &[]);
    fire.keep_charge = true;
    fire.land_continue = true;
    let mut peanut = shot_hit(24, 43, 70, 40, 70, ANGLE_FORWARD);
    peanut.hitlag = 20;
    peanut.shield_damage = shield(43, -10);
    peanut.clanks = false;
    peanut.rebounds = false;
    fire.projectile = Some(ProjectileSpawn {
        // (Only the draw's script fires it, on the release frame.)
        frame: 255,
        x: rs(6000),
        y: rs(7000),
        speed: rs(1600),
        life: 80,
        hitbox: peanut,
        end_damage: peanut.damage,
        arc: ProjectileArc {
            gravity: rs(80),
            max_fall: rs(2000),
            // (A touch of the floor ends it.)
            lodge: 1,
            ..ProjectileArc::NONE
        },
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: Some(ChargedShot {
            full: POPGUN_FULL,
            snap: 0,
            damage_to: Fx::from_ratio(138, 10),
            // (The script sets the speed and angle by the charge: the spec's speed is not scaled.)
            speed_to: rs(1600),
            life_to: 80,
            size_from: 100,
            size_to: 100,
            floor: false,
            times: 100,
            power_to: 100,
        }),
        tool: Tool::NONE,
    });
    w.moves[POPGUN_FIRE as usize] = fire;

    // The misfire: reached on the special's move frame 95 (the reference's 96, still held), the explosion on its second frame (the
    // special's 97), FAF 105 frames later (the child's FAF 108: 95 + 106 + 2 is the special's 203) [D].
    let mut boom = ref_move(
        108,
        0,
        0,
        255,
        &[ru(2, 2, 70, 85, 82, 230, 361, 30, 92, 0, 0)],
    );
    set_all(&mut boom, |h| {
        h.hitlag = 150;
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
        h.rebounds = false;
    });
    boom.land_continue = true;
    w.moves[POPGUN_MISFIRE as usize] = boom;
}

// ---- The up special: the rocket barrels --------------------------------------------------------------------------------------------

/// The up special's parts: the flight, the crash, and the holder of the barrels that fly off when the scamp is hit (never started by
/// a button).
pub const ROCKET_FLIGHT: MoveId = MoveId::Ext6;
pub const ROCKET_CRASH: MoveId = MoveId::Ext7;
pub const BARRELS: MoveId = MoveId::Ext8;
/// The barrels' tool kind (one set at a time is all there is: the scamp has one pack).
pub const BARRELS_KIND: u8 = 10;
/// A second rocket-barrel charge in one air time, on the body's own fall (owner decision 75).
pub const ROCKET_AGAIN: MoveId = MoveId::Ext5;
/// The air mark the first charge of an air time sets (`set_air_mark(0)`), and the air flag the flip's kick sets (`set_air_flag(1)`).
pub const ROCKET_MARK: u8 = 0;
pub const KICK_FLAG: u8 = 1;

/// The explosion of the rocket barrels (a crash, or the barrels meeting the stage after flying off): 18%, 70 degrees, BK 50, KS 70,
/// radius 20.0 round the root (the feet) [R].
fn barrels_blast() -> Hitbox {
    let mut h = shot_hit(200, 180, 70, 50, 70, ANGLE_BY_POSITION);
    h.clanks = true;
    h
}

/// The up special's charge. In the air the scamp falls slower while charging, on the first charge of an air time only [R: 3.1.0; E:
/// gravity 0.02 and fall 0.5 of the reference's units]: a mark a hit does not clear (owner decision 75) picks `ROCKET_AGAIN` for a
/// later charge in the same air time (after being hit out of the first, for one).
fn rocket_charge() -> Move {
    let mut mv = rocket_charge_again();
    mv.air_gravity = Some(rs(20));
    mv.air_max_fall = Some(rs(500));
    // The first charge of an air time only (owner decision 75): its script marks the air time (`set_air_mark(0)`, a mark a hit does
    // not clear), and a later charge in it is `ROCKET_AGAIN`, on the body's own fall. After the flip's kick (`air_flag(1)`) the up
    // special does not start at all until the scamp lands or catches a ledge (the reference's rule) [R]; a hit clears that flag [E].
    mv.flag_versions = vec![
        FlagVersion {
            flags: 1 << KICK_FLAG,
            marks: 0,
            slot: None,
        },
        FlagVersion {
            flags: 0,
            marks: 1 << ROCKET_MARK,
            slot: Some(ROCKET_AGAIN as u8),
        },
    ];
    mv
}

/// The rocket barrels' charge on the body's own fall (a second charge in one air time: the reference's slower fall is for the first
/// only, owner decision 75).
fn rocket_charge_again() -> Move {
    let mut mv = ref_move(49, 0, 0, 255, &[]);
    // (The script decides when the charge ends: the move lasts past the longest.)
    mv.total_frames = 80;
    mv.land_continue = true;
    // (Started in the air, the scamp's momentum is stopped [E: "disrupted", the reference's barrel jumping note].)
    mv.start_speed = 0;
    mv.start_speed_y = 0;
    mv.shed = Some(BARRELS as u8);
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_rocket.script"),
    ));
    mv
}

/// The up special: the rocket barrels (13.0.1; ssbwiki.com's raw up special page, `Rocketbarrel_Boost`, the SSB4-to-13.0.1 changelist,
/// the 3.1.0 and 8.0.0 notes, ultimateframedata.com).
///
/// * The charge: from frame 7 special held charges it, up to 60 frames [R]; let go (or at 60) the scamp launches on the next frame
///   (frames 8 to 68 [R]). In the air it falls slower [R; E: how much] (see `rocket_charge`). The stick tilts the launch's angle while
///   charging, more the longer the charge [R: qualitative; E: 0.75 degrees a frame of the stick, at most 45 either way].
/// * The flight (`ROCKET_FLIGHT`): launched at 3.1 + 0.019 a frame of charge of the reference's units a frame, x1.05 from the air [R];
///   steered by the stick while flying [R: qualitative; E: 1.5 degrees a frame, at most 80 either way]; its speed falls off to nothing
///   over its 54 frames [E: the published speed is the launch's only]. Hits: frames 1-2 of the flight 10% (361, BK 40, KS 85, radius
///   4.0), 3-9 8% (80 degrees, BK 40, KS 90, radius 3.2), 10-28 6% (70 degrees, BK 40, KS 90, radius 2.8), each a box in front of and
///   behind the waist, never clanking, one hit [R]. Then the special fall, 30 frames of landing lag [R]; ledges from the launch [E].
/// * The crash (`ROCKET_CRASH`): touching a wall, or the ground after the flight's first frames, sets the barrels off [R]: 18%, 70
///   degrees, BK 50, KS 70, radius 20.0 round the scamp on the crash's frames 1-2 [R], FAF 58 [R: 57-59 on level ground]; the scamp
///   takes 5% itself (`Move::self_damage`, sim 119) [R: "about 5%", the article's general text; E: 5, not tied to 13.0.1].
/// * Hit while charging or flying (E48, `shed`): the barrels fly off (`BARRELS`), a flinch ending the move in the special fall [R]; they
///   wobble along, steered a while by the scamp's stick [R; E: how], hitting for 1% (361, BK 10, KS 100) [R], and burst on the stage
///   (the 18% explosion) [R]. They cannot be shielded (`Hitbox::unblockable`, sim 119) [R].
/// * In the air only the first charge of an air time falls slower; a later one in it (`ROCKET_AGAIN`) falls at the body's speed
///   (owner decision 75). After the flip's kick the up special does not start until the scamp lands or catches a ledge.
fn rocket_barrels(w: &mut Weapon) {
    w.moves[MoveId::UpSpecial as usize] = rocket_charge();
    w.moves[ROCKET_AGAIN as usize] = rocket_charge_again();

    // The flight: its frame k is the k-th of the flight (the move is reached on the charge's last frame).
    let mut rows = Vec::new();
    for (first, last, damage, angle, kbg, radius) in [
        (2u8, 3u8, 100, 361i16, 85i16, 40),
        (4, 10, 80, 80, 90, 32),
        (11, 29, 60, 70, 90, 28),
    ] {
        rows.push(ru(
            first, last, 50, 50, radius, damage, angle, 40, kbg, 0, 0,
        ));
        rows.push(ru(
            first, last, -50, 50, radius, damage, angle, 40, kbg, 1, 0,
        ));
    }
    let mut flight = ref_move(56, 0, 0, 255, &rows);
    set_all(&mut flight, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
        h.rebounds = false;
    });
    flight.keep_charge = true;
    flight.helpless_after = true;
    flight.fall_landing_lag = 30;
    flight.land_continue = true;
    flight.ledge_front_from = 1;
    flight.ledge_back_from = 1;
    flight.fast_fall_from = 255;
    flight.shed = Some(BARRELS as u8);
    flight.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_rocket_fly.script"),
    ));
    w.moves[ROCKET_FLIGHT as usize] = flight;

    // The crash: the explosion on its frames 1-2, FAF 58 counted from the crash [R].
    let mut crash = ref_move(58, 0, 0, 255, &[]);
    crash.hitboxes = vec![Hitbox {
        start: 1,
        end: 2,
        ..barrels_blast()
    }];
    crash.helpless_after = true;
    crash.fall_landing_lag = 30;
    // (The scamp hurts itself: "about 5%" when it hits a surface [R: the article's general text, not tied to 13.0.1; E: 5].)
    crash.self_damage = Fx::from_int(5);
    crash.land_continue = true;
    w.moves[ROCKET_CRASH as usize] = crash;

    // The barrels flying off: a projectile fired at the scamp's feet (1.0 up), up and a little forward [E], its script wobbling and
    // steering it [E]; 1% to each fighter once [R], bursting on the stage (or after 180 frames [E]) into the 18% explosion [R].
    let mut barrel = shot_hit(32, 10, 361, 10, 100, ANGLE_BY_POSITION);
    barrel.clanks = false;
    barrel.rebounds = false;
    // (They cannot be shielded [R: the article]; they can be reflected.)
    barrel.unblockable = true;
    let mut holder = ref_move(3, 0, 0, 255, &[]);
    holder.projectile = Some(ProjectileSpawn {
        frame: 255,
        x: Fx::ZERO,
        y: rs(8000),
        speed: rs(1500),
        life: 180,
        hitbox: barrel,
        end_damage: barrel.damage,
        arc: ProjectileArc {
            angle: 75,
            gravity: rs(40),
            max_fall: rs(1500),
            // (Any landing on a platform top bursts it: a bounce allowed only from a fall slower than nothing.)
            bounces: 1,
            bounce_max_fall: Fx::from_raw(1),
            ..ProjectileArc::NONE
        },
        phases: [None; MAX_PHASES],
        pierce: true,
        rehit: 0,
        burst: Some(Burst {
            life: 3,
            rehit: 0,
            on_wall: true,
            y: Fx::ZERO,
            phases: phase_list(&[(1, barrels_blast())]),
            on_ground: true,
            on_expire: true,
            keep_hits: false,
            on_hit: false,
            on_shield: true,
        }),
        charged: None,
        tool: Tool {
            kind: BARRELS_KIND,
            ..Tool::NONE
        },
    });
    holder.projectile_script = Some(script(
        Kind::Projectile,
        include_str!("scripts/popgun_barrels.script"),
    ));
    w.moves[BARRELS as usize] = holder;
}

// ---- The side special: the flip -----------------------------------------------------------------------------------------------

/// The side special's parts (sim 119): the aerial leap, the kick, the latch and its two exits.
pub const FLIP_AIR: MoveId = MoveId::Ext9;
pub const FLIP_KICK: MoveId = MoveId::Ext10;
pub const LATCH: MoveId = MoveId::Ext11;
pub const LATCH_ATTACK: MoveId = MoveId::Ext12;
pub const LATCH_JUMP: MoveId = MoveId::Ext13;

/// Where the scamp rides a fighter it latched onto: on its head, 0.9 of the held fighter's height above its feet, straight over them
/// [E: the reference's pose].
const LATCH_RIDE: (Fx, u8) = (Fx::ZERO, 90);

/// The leap's grab: two command-grab boxes ahead (the changelist's 4.5 and 4.0 of the reference's units ahead), at the hands and the
/// feet as the scamp flips [E: heights and radius 4.0].
fn flip_grab(last: u8) -> Vec<R> {
    vec![
        ru(18, last, 45, 70, 40, 0, 361, 0, 0, 0, 0),
        ru(18, last, 40, 20, 40, 0, 361, 0, 0, 1, 0),
    ]
}

/// The leap (ground or air): the grab on frames 18 to `last`, launched on frame 13 by its script, a second press of special the kick,
/// ledges in front once falling [E], no air control of its own [E].
fn flip_leap(faf: u8, landing_lag: u8, last: u8) -> Move {
    let mut leap = with_kind(
        ref_move(faf, landing_lag, 0, 255, &flip_grab(last)),
        HIT_GRAB,
    );
    set_all(&mut leap, |h| h.command = true);
    leap.catch_move = Some(LATCH as u8);
    leap.air_drift = 0;
    leap.fast_fall_from = 255;
    leap.ledge_front_from = 12;
    leap.ledge_falling_only = true;
    leap.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_flip.script"),
    ));
    leap
}

/// The side special: the flip (13.0.1; ultimateframedata.com, dragdown.wiki, the SSB4-to-13.0.1 changelist, ssbwiki.com
/// `Monkey_Flip`; no 13.0.1 hitbox page exists).
///
/// * The leap: a crouch, then a flip forward launched on frame 13 [E: the frame, 2.2 of the reference's units a frame ahead and 1.7 up,
///   2.8 ahead for a smash input: the distance is not published]; on its frames 18-36 from the ground, 18-29 in the air, it grabs
///   whomever it touches (a command grab: on the ground or in the air, shielding or not) [R]. FAF 62 on level ground (it lands and
///   slides to a stop) and 48 in the air, landing lag 26 [R]; not helpless after it [R]. Started in the air it stops the scamp's
///   momentum [E]. Ledges in front once falling from the launch [E].
/// * The kick (`FLIP_KICK`): special pressed again during the flip (from the launch to the end of the grab) [R: "a second button press";
///   E: the window]: clean on 6-10 14% (the inner leg, radius 3.5) and 12% (the foot, 4.0), late on 11-25 12% / 10%, 361, BK 80, KS 50
///   [R: damage, frames, angle, knockback; radii from the previous version; E: the positions]. FAF 33, landing lag 18 [R]. It bars the
///   up special until the scamp lands or catches a ledge [R] (a hit clears the bar too [E]).
/// * The latch (`LATCH`, E49): the caught fighter is held with the ordinary hold's clock and mash-out (owner decision 74), falling on
///   its own with the scamp riding its head (offstage both fall [R]); a scratch for 1% (361, BK 0, KS 0) on its frames 11 and 29 of
///   every 48 [R: ultimateframedata.com's "11 / 29 / 59..."; E: that reading]; the clock running out lets go as an ordinary hold.
/// * Attack (`LATCH_ATTACK`): a 3% scratch on frame 18 (on the held fighter, and on others near it: 361, BK 60, KS 150, radius 6.0 [E:
///   where]) and the throw on 20: 7%, 361, BK 50, KS 100 [R]; the scamp flips off back and up [E]; FAF 37 [R: ultimateframedata.com's
///   36 total; dragdown.wiki's 21 is the throw frame]. (The table's 5% throw row, BK 0, KS 50, is not used: its role is not published.)
/// * Jump (`LATCH_JUMP`): the scamp jumps off the head on frame 18, kicking the fighter for 2% (361, BK 0, KS 50: a throw) that holds
///   it 20 frames when it does not tumble (`Hitbox::fixed_stun`) [R]; the jump's speed is the scamp's full hop [E]; FAF 43 [R:
///   ultimateframedata.com's 42 total].
fn flip_special(w: &mut Weapon) {
    let mut ground = flip_leap(62, 0, 36);
    ground.land_continue = true;
    ground.air_version = Some(FLIP_AIR as u8);
    w.moves[MoveId::SideSpecial as usize] = ground;

    let mut air = flip_leap(48, 26, 29);
    air.start_speed = 0;
    air.start_speed_y = 0;
    w.moves[FLIP_AIR as usize] = air;

    let mut rows = Vec::new();
    for (first, last, inner, foot) in [(6u8, 10u8, 140, 120), (11, 25, 120, 100)] {
        rows.push(ru(first, last, 40, 45, 35, inner, 361, 80, 50, 0, 0));
        rows.push(ru(first, last, 75, 40, 40, foot, 361, 80, 50, 1, 0));
    }
    let mut kick = ref_move(33, 18, 0, 255, &rows);
    kick.air_drift = 0;
    kick.fast_fall_from = 255;
    kick.ledge_front_from = 25;
    kick.ledge_falling_only = true;
    kick.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_flip_kick.script"),
    ));
    w.moves[FLIP_KICK as usize] = kick;

    let ride = |clock: u8| Latch {
        x: LATCH_RIDE.0,
        height: LATCH_RIDE.1,
        clock_num: clock,
        clock_den: 1,
    };
    let mut latch = with_kind(
        ref_move(
            50,
            0,
            0,
            255,
            &[
                ru(11, 11, 30, 30, 40, 10, 361, 0, 0, 0, 0),
                ru(29, 29, 30, 30, 40, 10, 361, 0, 0, 0, 1),
            ],
        ),
        HIT_PUMMEL,
    );
    latch.latch = Some(ride(1));
    latch.land_continue = true;
    latch.fast_fall_from = 255;
    latch.air_drift = 0;
    latch.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_latch.script"),
    ));
    w.moves[LATCH as usize] = latch;

    let mut attack = ref_move(
        37,
        0,
        0,
        255,
        &[
            ru(18, 18, 30, 30, 40, 30, 361, 60, 150, 0, 0),
            // (In the throw's group: once thrown, the fighter is not hit again by it.)
            ru(18, 20, 20, 0, 60, 30, 361, 60, 150, 1, 2),
            ru(20, 20, 30, 30, 40, 70, 361, 50, 100, 0, 2),
        ],
    );
    attack.hitboxes[0].kind = HIT_PUMMEL;
    attack.hitboxes[2].kind = HIT_THROW;
    attack.latch = Some(ride(0));
    attack.fast_fall_from = 255;
    attack.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_latch_attack.script"),
    ));
    w.moves[LATCH_ATTACK as usize] = attack;

    let mut jump = with_kind(
        ref_move(43, 0, 0, 255, &[ru(18, 18, 0, 0, 40, 20, 361, 0, 50, 0, 0)]),
        HIT_THROW,
    );
    jump.hitboxes[0].fixed_stun = 20;
    jump.latch = Some(ride(0));
    jump.fast_fall_from = 255;
    jump.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/popgun_latch_jump.script"),
    ));
    w.moves[LATCH_JUMP as usize] = jump;
}
