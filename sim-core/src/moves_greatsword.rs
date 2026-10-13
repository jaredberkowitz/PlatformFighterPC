//! The Greatsword moveset (the champion body), modelled mechanically on a reference character: phase 1, the body's normals. A child
//! module of `moves` (it uses that file's private row builders). Data and sources: the seventh moveset's `REFERENCE_DATA.md`
//! (`docs/movesets/`).

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

/// Points along a reference capsule from `a0` to `a1` (reference tenths), about `step` apart: the ends and evenly spaced points
/// between [D].
fn along(a0: i32, a1: i32, step: i32) -> Vec<i32> {
    let n = ((a1 - a0).abs() / step.max(1)).max(1);
    (0..=n).map(|i| a0 + (a1 - a0) * i / n).collect()
}

/// A point on the sword (the reference's `haver` bone runs along the blade from the grip): the grip at `(gx, gy)` and the blade
/// pointing along `(dx, dy)` (a direction in thousandths), `at` reference tenths from the grip. Where the grip is and which way the
/// blade points in each pose are [E] (the reference's bones are its own skeleton's); the distance along the blade is [R].
const fn blade(gx: i32, gy: i32, dx: i32, dy: i32, at: i32) -> (i32, i32) {
    (gx + dx * at / 1000, gy + dy * at / 1000)
}

/// Rows for boxes along the blade in one pose: `(at along the blade, radius, priority)` each, the rest of the row as `ru`.
#[allow(clippy::too_many_arguments)]
fn on_blade(
    first: u8,
    last: u8,
    pose: (i32, i32, i32, i32),
    boxes: &[(i32, i32, u8)],
    damage: i32,
    angle: i16,
    bkb: i16,
    kbg: i16,
    group: u8,
) -> Vec<R> {
    let (gx, gy, dx, dy) = pose;
    boxes
        .iter()
        .map(|&(at, radius, priority)| {
            let (x, y) = blade(gx, gy, dx, dy, at);
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

/// The Greatsword moveset (the champion body): a huge two-handed sword and kicks. Every normal follows the reference frame for frame
/// (the seventh moveset's `REFERENCE_DATA.md` in `docs/movesets/`, version 13.0.1, read from ssbwiki.com's raw hitbox tables):
/// active frames, FAF, landing lag and autocancel windows, continuable jab windows, smash charge frames; damage, angles and angle types, base and growth
/// knockback, set knockback and set weight, hitlag and SDI multipliers, shield damage, grounded- and aerial-only boxes, hit groups,
/// sweet and sour phases, no-clank boxes. Radii and offsets on the reference's root bone (`top`) are its values converted (x 1.25,
/// rounded half up to a tenth) [D]; boxes on the sword (`haver`) and the legs (`kneer`, `footl`, `footr`) are placed on our body
/// where that bone is in the move's pose [E]. The specials are still the claws' (phases 2 and 3 build the reference's).
pub fn greatsword() -> Weapon {
    let mut w = claws();
    // A held sword: the hits reach out beyond the body, so no limb reaches out to be hit with them.
    w.limbs = false;
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    w
}

fn set_all(mv: &mut Move, f: impl Fn(&mut Hitbox)) {
    for h in &mut mv.hitboxes {
        f(h);
    }
}

fn jabs(w: &mut Weapon) {
    // Jab 1: a kick on frames 4-5, 2.5%, hitlag x1.5, angle type 3 (forward) throughout: two boxes at the 361 angle (BK 35, KS 25,
    // radius 2.0, 10.0 of the reference's units up, 7.0 and 10.5 ahead) and the tip that pulls a fighter in (180 degrees, BK 20,
    // KS 15, radius 2.3, 14.0 ahead; the reference's id 2, fighters only; its id 3, the same box for objects, is not modelled). FAF
    // 30. Pressing attack on frames 8-30 carries on into jab 2 from frame 8 (the continuable window); holding attack throws jab 1
    // again from frame 16 (the reference's "consecutive jab frame", a script).
    let mut jab1 = ref_move(
        30,
        0,
        0,
        255,
        &[
            ru(4, 5, 70, 100, 20, 25, 361, 35, 25, 0, 0),
            ru(4, 5, 105, 100, 20, 25, 361, 35, 25, 1, 0),
            ru(4, 5, 140, 100, 23, 25, 180, 20, 15, 2, 0),
        ],
    );
    set_all(&mut jab1, |h| {
        h.hitlag = 150;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab1, MoveId::Jab2, 8);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/greatsword_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: a second kick on its frames 5-6, 2%, hitlag x1.2, angle 361 (type 3): BK 35 / KS 25 at 3.0 and 7.0 of the reference's
    // units ahead (radius 3.0), BK 25 / KS 20 at 11.0 (radius 3.5), 9.5 up. FAF 33. Attack pressed on its frames 9-30 carries on into
    // jab 3 from its frame 9.
    let mut jab2 = ref_move(
        33,
        0,
        0,
        255,
        &[
            ru(5, 6, 30, 95, 30, 20, 361, 35, 25, 0, 0),
            ru(5, 6, 70, 95, 30, 20, 361, 35, 25, 1, 0),
            ru(5, 6, 110, 95, 35, 20, 361, 25, 20, 2, 0),
        ],
    );
    set_all(&mut jab2, |h| {
        h.hitlag = 120;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab2, MoveId::Jab3, 9);
    w.moves[MoveId::Jab2 as usize] = jab2;

    // Jab 3: the sword on its frames 6-7, 3.5%, 30 degrees (type 3), BK 75, KS 50, hitlag x2, radius 4.5 along 16.0 to 8.0 of the
    // reference's units ahead at 9.0 up [D]. FAF 38.
    let mut jab3 = ref_move(
        38,
        0,
        0,
        255,
        &along(160, 80, 40)
            .into_iter()
            .map(|x| ru(6, 7, x, 90, 45, 35, 30, 75, 50, 0, 0))
            .collect::<Vec<_>>(),
    );
    set_all(&mut jab3, |h| {
        h.hitlag = 200;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::Jab3 as usize] = jab3;
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a horizontal sword sweep on frames 9-10, 11%, angle 361 (type 3), BK 20, KS 100, radius 6.0 along 14.5 to 7.0 of
    // the reference's units ahead at 9.0 up [D]. FAF 36.
    let mut ftilt = ref_move(
        36,
        0,
        0,
        255,
        &along(145, 70, 37)
            .into_iter()
            .map(|x| ru(9, 10, x, 90, 60, 110, 361, 20, 100, 0, 0))
            .collect::<Vec<_>>(),
    );
    set_all(&mut ftilt, |h| h.angle_type = ANGLE_FORWARD);
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: the sword swept from behind over the head. Early on frames 6-8, late on 9, 8%, 95 degrees, BK 46, KS 100, angle type 0
    // (by position: a target behind is sent backward). The blade's three boxes (radius 5.6 / 5.0 / 6.0 at 12.0 / 8.5 / 3.0 along it)
    // point up and back early [E] and up and forward late [E]; early only, the box above the head (radius 4.5, 21.0 to 16.0 up) [D].
    // FAF 33.
    let blade_boxes = [(120, 56, 0u8), (85, 50, 1), (30, 60, 2)];
    let mut rows = on_blade(6, 8, (0, 150, -500, 866), &blade_boxes, 80, 95, 46, 100, 0);
    for y in along(210, 160, 50) {
        rows.push(ru(6, 8, 0, y, 45, 80, 95, 46, 100, 3, 0));
    }
    rows.extend(on_blade(
        9,
        9,
        (20, 150, 500, 866),
        &blade_boxes,
        80,
        95,
        46,
        100,
        0,
    ));
    let mut utilt = ref_move(33, 0, 0, 255, &rows);
    set_all(&mut utilt, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a sliding kick. Early on frames 7-8, late on 9-17: the leg (radius 3.6, low in front [E]: the reference's knee bone)
    // 7%; early only, the body box (radius 2.5, 2.5 of the reference's units up, 2.6 behind [D]) 6%; 80 degrees (type 3), BK 80, KS
    // 75. The reference's 20% trip chance can never happen at that angle (ssbwiki.com), so it is not modelled. FAF 41. The slide
    // forward on frames 6-17 [E: the distance is not published; 12 of the reference's units]. The back leg is intangible on 7-17
    // (recorded: with a held sword the fourth hurtbox circle is the middle one, so it acts on nothing, a listed limit as the Staff's
    // arms; the lower body circle covers the whole lower body, not one leg).
    let leg = |first: u8, last: u8| -> Vec<R> {
        [(90, 20), (60, 25)]
            .into_iter()
            .map(|(x, y)| ru(first, last, x, y, 36, 70, 80, 80, 75, 0, 0))
            .collect()
    };
    let mut rows = leg(7, 8);
    rows.push(ru(7, 8, -26, 25, 25, 60, 80, 80, 75, 1, 0));
    rows.extend(leg(9, 17));
    let mut dtilt = ref_move(41, 0, 0, 255, &rows);
    set_all(&mut dtilt, |h| h.angle_type = ANGLE_FORWARD);
    dtilt.motion = vec![Motion {
        start: 5,
        end: 16,
        vx: Fx::from_ratio(1, 8),
        vy: Fx::ZERO,
    }];
    dtilt.part_intangible = vec![PartIntangible {
        start: 6,
        end: 16,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a lunging slash. Clean on frames 9-13: 11%, 60 degrees, BK 100, KS 70 (7.0.0); late on 14-18: 8%, 65 degrees, BK
    // 100, KS 50; angle type 3; shield damage +1 (109 and 113 percent [D]). Two boxes on the blade (14.0 and 8.0 along it, radius 5.0 /
    // 4.0 clean, 4.0 / 3.3 late), pointing forward and a little up [E], and one on the body (8.0 up, 3.0 ahead, radius 3.0 / 2.8) [D].
    // FAF 46.
    let pose = (50, 90, 940, 342);
    let mut rows = on_blade(
        9,
        13,
        pose,
        &[(140, 50, 0), (80, 40, 1)],
        110,
        60,
        100,
        70,
        0,
    );
    rows.push(ru(9, 13, 30, 80, 30, 110, 60, 100, 70, 2, 0));
    rows.extend(on_blade(
        14,
        18,
        pose,
        &[(140, 40, 0), (80, 33, 1)],
        80,
        65,
        100,
        50,
        0,
    ));
    rows.push(ru(14, 18, 30, 80, 28, 80, 65, 100, 50, 2, 0));
    let mut dash = ref_move(46, 0, 0, 255, &rows);
    set_all(&mut dash, |h| {
        h.angle_type = ANGLE_FORWARD;
        h.shield_damage = if h.damage == Fx::from_int(11) {
            shield(110, 10)
        } else {
            shield(80, 10)
        };
    });
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash: three single-frame hits along the sword in front (radius per hit, along 10.0 / 16.0 to 6.5 of the reference's
    // units ahead at 9.0 up [D]), angle type 3 throughout, each its own hit group. Hit 1 on frame 19: 3%, set knockback with set
    // weight, hitlag x1.5, SDI x0; on grounded targets 60 degrees (FKV 25, BK 30, KS 30) near and 170 (FKV 10) at the tip, on aerial
    // ones 0 (FKV 10) near and 180 (FKV 10) at the tip: they pull the target into the next hits. Hit 2 on 24: set weight, hitlag
    // x1.5, SDI x0.5: 3% at 15 degrees (FKV 80, 7.0.0) at the hilt, 2% at 15 (FKV 50) near, 4% at 170 (FKV 10) at the tip. Hit 3 on
    // 28: 13% near, 12% at the tip, angle 361, BK 42, KS 115, hitlag x1.8. Charges on frame 5. FAF 67.
    let near = along(100, 65, 35);
    let tip = along(160, 65, 35);
    let mut rows = Vec::new();
    let push = |rows: &mut Vec<R>, xs: &[i32], row: &dyn Fn(i32) -> R| {
        for &x in xs {
            rows.push(row(x));
        }
    };
    push(&mut rows, &near, &|x| {
        ru(19, 19, x, 90, 45, 30, 60, 30, 30, 0, 0)
    });
    push(&mut rows, &tip, &|x| {
        ru(19, 19, x, 90, 45, 30, 170, 0, 100, 1, 0)
    });
    push(&mut rows, &near, &|x| {
        ru(19, 19, x, 90, 45, 30, 0, 0, 100, 2, 0)
    });
    push(&mut rows, &tip, &|x| {
        ru(19, 19, x, 90, 45, 30, 180, 0, 100, 3, 0)
    });
    let hit1 = rows.len();
    rows.push(ru(24, 24, 50, 90, 25, 30, 15, 0, 100, 0, 1));
    push(&mut rows, &near, &|x| {
        ru(24, 24, x, 90, 52, 20, 15, 0, 100, 1, 1)
    });
    push(&mut rows, &tip, &|x| {
        ru(24, 24, x, 90, 52, 40, 170, 0, 100, 2, 1)
    });
    let hit2 = rows.len();
    push(&mut rows, &near, &|x| {
        ru(28, 28, x, 90, 55, 130, 361, 42, 115, 0, 2)
    });
    push(&mut rows, &tip, &|x| {
        ru(28, 28, x, 90, 55, 120, 361, 42, 115, 1, 2)
    });
    let mut fsmash = ref_move(67, 0, 0, 255, &rows);
    for (i, h) in fsmash.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        if i < hit1 {
            h.set_weight = true;
            h.hitlag = 150;
            h.sdi = 0;
            h.fixed_kb = if h.priority == 0 { 25 } else { 10 };
            h.targets = if h.priority < 2 {
                TARGETS_GROUNDED
            } else {
                TARGETS_AERIAL
            };
        } else if i < hit2 {
            h.set_weight = true;
            h.hitlag = 150;
            h.sdi = 50;
            h.fixed_kb = [80, 50, 10][usize::from(h.priority)];
        } else {
            h.hitlag = 180;
        }
    }
    fsmash.charge_at = Some(4);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: the sword swung from in front up over the head and behind. Clean on frame 12 (12.0.0 / 7.0.0): 13% on the blade (9.0
    // and 13.0 along it, radius 4.0; KS 105), in front and up [E]. Mid on 13-14: the same blade boxes overhead [E], the hilt (2.5 along,
    // radius 3.3, 12%, KS 96), which wins where both reach (the lower id), and a box behind the head (radius 3.5, 15.0 to 8.0 up, 5.0
    // behind [D], 11%, KS 90). All 83 degrees, BK 40, hitlag x1.2, angle type 3. Late on 15-16: 8% at 60 degrees (type 0), BK 32, KS 100,
    // radius 3.0 at 2.5 / 7.0 / 11.0 along the blade, behind [E]. One hit in all. Charges on frame 6 (ssbwiki.com; conflict 3). FAF 46.
    let mut rows = on_blade(
        12,
        12,
        (60, 110, 707, 707),
        &[(90, 40, 1), (130, 40, 2)],
        130,
        83,
        40,
        105,
        0,
    );
    let overhead = (20, 150, 0, 1000);
    rows.extend(on_blade(
        13,
        14,
        overhead,
        &[(25, 33, 0)],
        120,
        83,
        40,
        96,
        0,
    ));
    rows.extend(on_blade(
        13,
        14,
        overhead,
        &[(90, 40, 1), (130, 40, 2)],
        130,
        83,
        40,
        105,
        0,
    ));
    for y in along(150, 80, 35) {
        rows.push(ru(13, 14, -50, y, 35, 110, 83, 40, 90, 3, 0));
    }
    let late = rows.len();
    rows.extend(on_blade(
        15,
        16,
        (-20, 150, -707, 707),
        &[(25, 30, 0), (70, 30, 1), (110, 30, 2)],
        80,
        60,
        32,
        100,
        0,
    ));
    let mut usmash = ref_move(46, 0, 0, 255, &rows);
    for (i, h) in usmash.hitboxes.iter_mut().enumerate() {
        if i < late {
            h.hitlag = 120;
            h.angle_type = ANGLE_FORWARD;
        } else {
            h.angle_type = ANGLE_BY_POSITION;
        }
    }
    usmash.charge_at = Some(5);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a low sweep in front, then behind. Hit 1 on frames 8-9: 3%, set knockback with set weight (BK 0, KS 97), SDI x0.5,
    // pulling in: on aerial targets 170 degrees (FKV 90) near and 155 (FKV 85, 3.1.0) far (radius 2.5), on grounded ones 160 (FKV 90)
    // and 150 (FKV 85) (radius 3.5); along 8.0 / 13.5 to 5.0 of the reference's units ahead at 4.0 up [D]; type 3. Hit 2 on 21-23: 11%,
    // 35 degrees sent backward (type 4), BK 35, KS 92, hitlag x1.4, radius 4.4 along 14.0 to 1.0 behind at 4.8 up [D]. Charges on
    // frame 2. FAF 50.
    let mut rows = Vec::new();
    for (id, x0, radius, angle) in [
        (0u8, 80, 25, 170i16),
        (1, 135, 25, 155),
        (2, 80, 35, 160),
        (3, 135, 35, 150),
    ] {
        for x in along(x0, 50, 30) {
            rows.push(ru(8, 9, x, 40, radius, 30, angle, 0, 97, id, 0));
        }
    }
    let hit1 = rows.len();
    for x in along(-140, -10, 45) {
        rows.push(ru(21, 23, x, 48, 44, 110, 35, 35, 92, 0, 1));
    }
    let mut dsmash = ref_move(50, 0, 0, 255, &rows);
    for (i, h) in dsmash.hitboxes.iter_mut().enumerate() {
        if i < hit1 {
            // (Ids 0 and 1 hit aerial targets, 2 and 3 grounded ones; the near boxes FKV 90, the far ones 85.)
            h.targets = if h.priority < 2 {
                TARGETS_AERIAL
            } else {
                TARGETS_GROUNDED
            };
            h.fixed_kb = if h.priority % 2 == 0 { 90 } else { 85 };
            h.set_weight = true;
            h.sdi = 50;
            h.angle_type = ANGLE_FORWARD;
        } else {
            h.hitlag = 140;
            h.angle_type = ANGLE_BACKWARD;
        }
    }
    dsmash.charge_at = Some(1);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: the sword swung in a full circle round the body on frames 5-15, 8%, 40 degrees (type 0), BK 25, KS 100, radius 3.8
    // at 12.0 / 8.0 / 4.0 along the blade. One hit; the blade's place on each part of the circle is [E] (above on 5-7, in front on
    // 8-10, below on 11-13, behind on 14-15). Landing lag 9, autocancels on frame 1 and from 31, FAF 39.
    let mut rows = Vec::new();
    for (first, last, dx, dy) in [
        (5u8, 7u8, 0, 1000),
        (8, 10, 1000, 0),
        (11, 13, 0, -1000),
        (14, 15, -1000, 0),
    ] {
        rows.extend(on_blade(
            first,
            last,
            (0, 100, dx, dy),
            &[(120, 38, 0), (80, 38, 1), (40, 38, 2)],
            80,
            40,
            25,
            100,
            0,
        ));
    }
    let mut nair = ref_move(39, 9, 1, 31, &rows);
    set_all(&mut nair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: an overhead chop in front. Clean on frames 18-19 (one hit): at the tip (radius 3.3, 7.5 to 6.5 of the reference's
    // units up, 14.5 to 14.0 ahead [D]) a meteor on aerial targets (14%, 270 degrees, BK 20, KS 90) and 14% at 60 degrees (BK 20, KS
    // 100) on grounded ones; the swing's arc (radius 7.8, 9.5 to 14.5 up, 14.0 to 14.6 ahead [D]) and the blade (radius 4.0, 12.0 to
    // 4.0 along it, pointing forward and down [E]) 13% at 45 (BK 30, KS 83); hitlag x1.4. Late on 20-25: the blade lower down [E],
    // 11%, 45, BK 30, KS 83, radius 3.0. Angle type 3. Landing lag 11, autocancels on frames 1-2 and from 43, FAF 54. (The
    // reference's "ledges from both sides from frame 78" (2.0.0) is not modelled: an aerial catches ledges after its FAF here.)
    let mut rows = vec![
        ru(18, 19, 145, 75, 33, 140, 270, 20, 90, 0, 0),
        ru(18, 19, 140, 65, 33, 140, 270, 20, 90, 0, 0),
        ru(18, 19, 145, 75, 33, 140, 60, 20, 100, 1, 0),
        ru(18, 19, 140, 65, 33, 140, 60, 20, 100, 1, 0),
        ru(18, 19, 140, 95, 78, 130, 45, 30, 83, 2, 0),
        ru(18, 19, 146, 145, 78, 130, 45, 30, 83, 2, 0),
    ];
    rows.extend(on_blade(
        18,
        19,
        (50, 130, 800, -600),
        &[(120, 40, 3), (80, 40, 3), (40, 40, 3)],
        130,
        45,
        30,
        83,
        0,
    ));
    let clean = rows.len();
    rows.extend(on_blade(
        20,
        25,
        (50, 90, 500, -866),
        &[(120, 30, 3), (80, 30, 3), (40, 30, 3)],
        110,
        45,
        30,
        83,
        0,
    ));
    let mut fair = ref_move(54, 11, 2, 43, &rows);
    for (i, h) in fair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        h.hitlag = if i < clean { 140 } else { 100 };
        h.targets = match (i < clean, h.priority) {
            (true, 0) => TARGETS_AERIAL,
            (true, 1) => TARGETS_GROUNDED,
            _ => TARGETS_ALL,
        };
    }
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: a backward slash, 13%, angle 361 sent backward (type 4), BK 20, KS 95, radius 7.0 along 15.0 to 7.0 of the reference's
    // units behind: at 8.5 up on frame 11, 16.0 up on 12 [D]. Landing lag 8, autocancels on frames 1-4 and from 37, FAF 43.
    let mut rows = Vec::new();
    for (frame, y) in [(11u8, 85), (12, 160)] {
        for x in along(-150, -70, 40) {
            rows.push(ru(frame, frame, x, y, 70, 130, 361, 20, 95, 0, 0));
        }
    }
    let mut bair = ref_move(43, 8, 4, 37, &rows);
    set_all(&mut bair, |h| h.angle_type = ANGLE_BACKWARD);
    w.moves[MoveId::BAir as usize] = bair;

    // Up air: a single hit in three phases (13.0.1; not a multi-hit). Clean early on frame 8: the blade flat overhead, three columns
    // (radius 4.2) from 17.5 of the reference's units up to 24.0 / 25.0 / 26.0, at 0 / 4.5 / 9.0 ahead [D]; clean late on 9: on the
    // blade (radius 4.0 at 4.0 / 9.0 / 14.0 along it, pointing up and a little forward [E]); both 11%, hitlag x1.2. Late on 10-23: 8%
    // on the blade (radius 3.0), hitlag x1. All 80 degrees (type 0), BK 42, KS 80. Landing lag 9, autocancels on frame 1 and from 32,
    // FAF 45.
    let mut rows = Vec::new();
    for (id, x, top) in [(0u8, 0, 240), (1, 45, 250), (2, 90, 260)] {
        for y in along(175, top, 35) {
            rows.push(ru(8, 8, x, y, 42, 110, 80, 42, 80, id, 0));
        }
    }
    let pose = (20, 160, 342, 940);
    rows.extend(on_blade(
        9,
        9,
        pose,
        &[(40, 40, 0), (90, 40, 1), (140, 40, 2)],
        110,
        80,
        42,
        80,
        0,
    ));
    let clean = rows.len();
    rows.extend(on_blade(
        10,
        23,
        pose,
        &[(40, 30, 0), (90, 30, 1), (140, 30, 2)],
        80,
        80,
        42,
        80,
        0,
    ));
    let mut uair = ref_move(45, 9, 1, 32, &rows);
    for (i, h) in uair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BY_POSITION;
        if i < clean {
            h.hitlag = 120;
        }
    }
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: the sword driven straight down. Clean on frames 11-13: at the tip (11.0 to 12.5 along the blade, radius 3.6) a meteor
    // on aerial targets only (15%, 270 degrees, BK 10, KS 100, hitlag x1.5); along it (7.0 to 13.0, radius 4.5) 13% at 60 (BK 30, KS
    // 105, hitlag x1.2), which grounded targets and the rest take. Late on 14-39: 8% at 65 (BK 80, KS 55), radius 3.5 at 9.0 to 11.0,
    // hitlag x1.2. Angle type 0. The grip at the chest, the blade pointing down [E]. Landing lag 16, autocancels on frames 1-4 and from
    // 46, FAF 65.
    let down = (0, 90, 0, -1000);
    let mut rows = on_blade(
        11,
        13,
        down,
        &[(110, 36, 0), (125, 36, 0)],
        150,
        270,
        10,
        100,
        0,
    );
    let meteor = rows.len();
    rows.extend(on_blade(
        11,
        13,
        down,
        &[(70, 45, 1), (100, 45, 1), (130, 45, 1)],
        130,
        60,
        30,
        105,
        0,
    ));
    rows.extend(on_blade(
        14,
        39,
        down,
        &[(90, 35, 1), (110, 35, 1)],
        80,
        65,
        80,
        55,
        0,
    ));
    let mut dair = ref_move(65, 16, 4, 46, &rows);
    for (i, h) in dair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BY_POSITION;
        if i < meteor {
            h.targets = TARGETS_AERIAL;
            h.hitlag = 150;
        } else {
            h.hitlag = 120;
        }
    }
    w.moves[MoveId::DAir as usize] = dair;
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: standing on frames 9-10 (FAF 37), dash 12-13 (FAF 45), pivot 13-14 (FAF 40). Each reach is the reference's grounded-target
    // grab box (a swept circle at 6.6 of its units up: 4.0 to 8.7 ahead, radius 3.3; 4.0 to 10.4, 2.6; behind 4.0 to 14.7, 3.3, written
    // in front since the engine turns a pivot grab round) as circles along it [D]. (Its second, thinner box for aerial targets is not
    // modelled: grabs catch only fighters on the ground here.)
    let grab_line = |first: u8, last: u8, radius: i32, x0: i32, x1: i32| -> Vec<R> {
        along(x0, x1, 30)
            .into_iter()
            .map(|x| ru(first, last, x, 66, radius, 0, 361, 0, 0, 0, 0))
            .collect()
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(37, 0, 0, 255, &grab_line(9, 10, 33, 40, 87)),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(45, 0, 0, 255, &grab_line(12, 13, 26, 40, 104)),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(40, 0, 0, 255, &grab_line(13, 14, 33, 40, 147)),
        HIT_GRAB,
    );
    // Pummel: 1.3%, set knockback 30 (KS 100), hitlag x2.1, radius 5.0 at 10.0 of the reference's units up and 8.0 ahead [D], no clank.
    // The reference hits on frame 1 with FAF 7; ours on frame 2 with FAF 8 (a pummel cannot hit on its first frame) [D].
    let mut pummel = reference_pummel(13, u(80), u(100), u(50));
    set_all(&mut pummel, |h| h.clanks = false);
    w.moves[MoveId::Pummel as usize] = pummel;
    // Throws. Forward: a kick on the held fighter on frame 7 (4%, hitlag x1.5), the throw on 9 (3%, 49 degrees, BK 48, KS 125), FAF 36.
    // Back: the kick on 13 (3%, hitlag x1.5), the throw on 15 behind (3%, 38 degrees, BK 48, KS 110), the thrower ending turned round
    // (the reference turns on 15), FAF 32. Up: a kick on 8 (2%) and a slash on 13 (2.5%, hitlag x1.2), the throw on 15 (4%, 80
    // degrees, BK 78, KS 97), FAF 51. (The reference's hits last two frames, 7-8, 13-14, 8-9 and 13-14; each lands once, on its first.)
    // Down: 7% at 122 degrees (sent behind), BK 40, KS 110; the reference freezes both fighters for 10 frames of hitlag on its frame 17
    // and releases on 18 with FAF 44 counted without them, so here (with nobody frozen) the release is on frame 28 and the FAF 54, the
    // same timeline [D]. The hits on the held fighter never clank; the "interrupted" rows and the thrower's invincibility are not
    // modelled (house limits).
    let mut fthrow = throw_after_hit(36, (7, 40), r(9, 9, 13, 11, 10, 30, 49, 48, 125, 0, 1));
    fthrow.hitboxes[0].hitlag = 150;
    let mut bthrow = throw_after_hit(32, (13, 30), r(15, 15, -13, 11, 10, 30, 38, 48, 110, 0, 1));
    bthrow.hitboxes[0].hitlag = 150;
    bthrow.turns_around = true;
    let mut uthrow = ref_move(
        51,
        0,
        0,
        255,
        &[
            r(8, 8, 13, 11, 10, 20, 361, 0, 0, 0, 0),
            r(13, 13, 13, 11, 10, 25, 361, 0, 0, 0, 1),
            r(15, 15, 13, 11, 10, 40, 80, 78, 97, 0, 2),
        ],
    );
    uthrow.hitboxes[0].kind = HIT_PUMMEL;
    uthrow.hitboxes[1].kind = HIT_PUMMEL;
    uthrow.hitboxes[1].hitlag = 120;
    uthrow.hitboxes[2].kind = HIT_THROW;
    for mv in [&mut fthrow, &mut bthrow, &mut uthrow] {
        for h in &mut mv.hitboxes {
            if h.kind == HIT_PUMMEL {
                h.clanks = false;
            }
        }
    }
    w.moves[MoveId::FThrow as usize] = fthrow;
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = uthrow;
    w.moves[MoveId::DThrow as usize] = with_kind(
        ref_move(
            54,
            0,
            0,
            255,
            &[r(28, 28, 13, 11, 10, 70, 122, 40, 110, 0, 0)],
        ),
        HIT_THROW,
    );
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: a sword sweep climbing up, frames 20-22, 10%, 45 degrees (type 3: forward on both sides), BK 90, KS 20, shield
    // damage +1 (110 [D]), no clank; intangible through frame 22 (on the first grab; the shared decay after), FAF 56. Its swept box
    // (17.0 of the reference's units ahead to 1.0 behind, 5.0 up, radius 5.0) as circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along(170, -10, 45)
            .into_iter()
            .map(|x| ru(20, 22, x, 50, 50, 100, 45, 90, 20, 0, 0))
            .collect::<Vec<_>>(),
    );
    set_all(&mut ledge, |h| {
        h.shield_damage = shield(100, 10);
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
    });
    ledge.intangible = 22;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack (lying face up, the reference's "back" floor attack): behind on frames 20-21 (18.5 to 5.5 of the reference's units
    // behind), then in front on 24-25 (18.5 to 6.0 ahead), 4.5 up, radius 4.5; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8
    // (214 [D]), no clank; intangible through frame 25, FAF 46. (The face-down and trip get-up attacks have no slot of their own.)
    let mut rows = Vec::new();
    for x in along(-185, -55, 45) {
        rows.push(ru(20, 21, x, 45, 45, 70, 48, 80, 48, 0, 0));
    }
    for x in along(185, 60, 45) {
        rows.push(ru(24, 25, x, 45, 45, 70, 48, 80, 48, 0, 1));
    }
    let mut getup = ref_move(46, 0, 0, 255, &rows);
    set_all(&mut getup, |h| {
        h.shield_damage = shield(70, 80);
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
    });
    getup.intangible = 25;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}
