//! The Gauntlets moveset (the sprinter body), modelled mechanically on a reference character: phase 1, the body's normals. A child
//! module of `moves` (it uses that file's private row builders).

use super::*;

/// The rapid jab's first cycle, its later cycles and its finisher (slots 39, 40 and 41: `goto` numbers in the jab scripts).
const RAPID: MoveId = MoveId::Ext10;
const RAPID_LOOP: MoveId = MoveId::Ext11;
const RAPID_END: MoveId = MoveId::Ext12;

/// The Gauntlets moveset (the sprinter body): punches, kicks and a flying knee. Every normal follows the reference frame for frame
/// (`docs/movesets/captain_falcon/REFERENCE_DATA.md`, version 13.0.1): active frames, FAF, landing lag and autocancel windows; damage,
/// angles, base and growth knockback, set knockback and set weight, hitlag, SDI and shieldstun multipliers, hitstun modifiers, shield
/// damage, grounded- and aerial-only boxes; the jab's continuable windows, the held repeat and the rapid jab's entry rule; the angled
/// forward tilt and smash; the up tilt's and up smash's intangible legs. Radii and offsets on the reference's root bone (`top`) are
/// its values converted (x 1.25, rounded half up to a tenth) [D]; boxes on other bones are placed on our body where that bone is in
/// the move's pose [E]. The specials are still the claws' (phase 2 builds the reference's).
pub fn gauntlets() -> Weapon {
    let mut w = claws();
    // Fists, knees and feet: the limb throwing a hit can be hit.
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
    // Jab 1: a jab on frame 3, 1.5%, hitlag x1.6. The near and middle boxes use the 361 angle (BK 20, KS 25); the far one pulls a
    // fighter in (180 degrees, BK 20, KS 20; the reference's priority since 7.0.0). FAF 18. Pressing attack again carries on into
    // jab 2 from frame 5; holding it throws jab 1 again from frame 7 (a script). (The reference's fourth box is the far box for
    // objects; no objects here.)
    let mut jab1 = ref_move(
        18,
        0,
        0,
        255,
        &[
            r(3, 3, 8, 13, 3, 15, 361, 20, 25, 0, 0),
            r(3, 3, 15, 13, 3, 15, 180, 20, 20, 1, 0),
            r(3, 3, 11, 13, 3, 15, 361, 20, 25, 2, 0),
        ],
    );
    set_rows(&mut jab1, &[0, 1, 2], |h| h.hitlag = 160);
    chain(&mut jab1, MoveId::Jab2, 5);
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: a second jab on frame 5, 1.5%, angle 361, BK 20, KS 25 / 25 / 20, hitlag x1.2, +3 hitstun. FAF 19. On frames 7-9 it goes
    // into the rapid jab once attack has been pressed three times and let go twice since jab 1 began (presses during hitlag count:
    // the `presses` register); otherwise a new press gives jab 3 from frame 8 (a script; the rapid jab wins where both are open).
    let mut jab2 = ref_move(
        19,
        0,
        0,
        255,
        &[
            r(5, 5, 8, 13, 3, 15, 361, 20, 25, 0, 0),
            r(5, 5, 13, 13, 4, 15, 361, 20, 25, 1, 0),
            r(5, 5, 18, 13, 5, 15, 361, 20, 20, 2, 0),
        ],
    );
    set_rows(&mut jab2, &[0, 1, 2], |h| {
        h.hitlag = 120;
        h.hitstun_add = 3;
    });
    jab2.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_jab_2.script"),
    ));
    w.moves[MoveId::Jab2 as usize] = jab2;

    // Jab 3: a knee on frames 6-7, 5%, angle 361, BK 70, KS 100, hitlag x2. FAF 33.
    let mut jab3 = ref_move(
        33,
        0,
        0,
        255,
        &[
            r(6, 7, 15, 14, 7, 50, 361, 70, 100, 0, 0),
            r(6, 7, 8, 13, 5, 50, 361, 70, 100, 1, 0),
        ],
    );
    set_rows(&mut jab3, &[0, 1], |h| h.hitlag = 200);
    w.moves[MoveId::Jab3 as usize] = jab3;

    // The rapid jab: hits on its frames 5, 7, ... 19 (eight a cycle), each 0.6% at the 361 angle (BK 8, KS 15), hitlag x0.5, SDI x0.4,
    // +2 hitstun and shieldstun x9 (on both boxes: the reference's scripts set them on the arm's box only, but a known engine quirk
    // applies them to the big box too). The cycle loops from its frame 20 back to frame 3, so the first cycle (ext10) runs frames
    // 1-20 and the later ones (ext11) frames 3-20 [D: the loop points 2-3 and 19-20]. After one full cycle, letting go of attack ends
    // it in the finisher (ext12) [E: "let go" is not held and not pressed for 8 frames, so mashing keeps it going].
    let rapid_rows = |frames: &[u8]| -> Vec<R> {
        let mut rows = Vec::new();
        for &f in frames {
            rows.push(r(f, f, 10, 13, 3, 6, 361, 8, 15, 0, 0));
            rows.push(r(f, f, 19, 10, 8, 6, 361, 8, 15, 1, 0));
            rows.push(r(f, f, 15, 10, 8, 6, 361, 8, 15, 1, 0));
        }
        rows
    };
    let rapid_hit = |mv: &mut Move| {
        for h in &mut mv.hitboxes {
            h.hitlag = 50;
            h.sdi = 40;
            h.hitstun_add = 2;
            h.shieldstun = 900;
        }
    };
    let mut rapid = ref_move(22, 0, 0, 255, &rapid_rows(&[5, 7, 9, 11, 13, 15, 17, 19]));
    rapid_hit(&mut rapid);
    // (Reference frame 5 is move frame 4; the hits land again every 2 frames.)
    rapid.rehit = Some((4, 2));
    rapid.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_rapid_jab.script"),
    ));
    w.moves[RAPID as usize] = rapid;
    // The later cycles: move frame k is the cycle's frame k + 3, so the hits are on move frames 2, 4, ... 16 (rows given as frames
    // 3, 5, ... 17 in the reference numbering of this move) and the loop point is move frame 17.
    let mut rapid_loop = ref_move(20, 0, 0, 255, &rapid_rows(&[3, 5, 7, 9, 11, 13, 15, 17]));
    rapid_hit(&mut rapid_loop);
    rapid_loop.rehit = Some((2, 2));
    rapid_loop.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_rapid_loop.script"),
    ));
    w.moves[RAPID_LOOP as usize] = rapid_loop;
    // The finisher: a rising backhand on frames 6-7, 3%, 45 degrees, BK 110, KS 55, hitlag x2. FAF 50. The reference's swept boxes
    // (13.0 down to 5.5 of its units up, 7.0 and 13.5 ahead) are circles at the sweep's ends.
    let mut finisher = ref_move(
        50,
        0,
        0,
        255,
        &[
            r(6, 7, 9, 16, 6, 30, 45, 110, 55, 0, 0),
            r(6, 7, 9, 7, 6, 30, 45, 110, 55, 0, 0),
            r(6, 7, 17, 16, 6, 30, 45, 110, 55, 1, 0),
            r(6, 7, 17, 7, 6, 30, 45, 110, 55, 1, 0),
        ],
    );
    set_rows(&mut finisher, &[0, 1, 2, 3], |h| h.hitlag = 200);
    w.moves[RAPID_END as usize] = finisher;
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a side kick on frames 7-10, angle 361, BK 60, KS 58; the shin and hip 8%, the foot 9% (the shin wins where both
    // reach). FAF 30. Aimed up or down the kick goes up or down and every box does 1% more.
    let mut ftilt = ref_move(30, 0, 0, 255, &[]);
    for (aim, extra, shin, hip, foot) in [
        (AIM_STRAIGHT, 0, (10, 12), (3, 12), (17, 12)),
        (AIM_UP, 10, (9, 16), (3, 13), (15, 20)),
        (AIM_DOWN, 10, (10, 9), (3, 12), (16, 5)),
    ] {
        let mut rows = ref_move(
            30,
            0,
            0,
            255,
            &[
                r(7, 10, shin.0, shin.1, 5, 80 + extra, 361, 60, 58, 0, 0),
                r(7, 10, hip.0, hip.1, 5, 80 + extra, 361, 60, 58, 1, 0),
                r(7, 10, foot.0, foot.1, 7, 90 + extra, 361, 60, 58, 2, 0),
            ],
        );
        set_rows(&mut rows, &[0, 1, 2], |h| h.aim = aim);
        ftilt.hitboxes.extend(rows.hitboxes);
    }
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: an axe kick on frames 14-17, 11%, 285 degrees (down and forward). The heel: on grounded targets BK 35, KS 100, radius
    // 8; on aerial ones BK 22, KS 90, radius 4, hitlag x1.4. The shin: BK 35, KS 100. FAF 37. The kicking leg cannot be hit while
    // the kick is out (frames 14-17).
    let mut utilt = ref_move(
        37,
        0,
        0,
        255,
        &[
            r(14, 17, 10, 15, 8, 110, 285, 35, 100, 0, 0),
            r(14, 17, 10, 15, 4, 110, 285, 22, 90, 1, 0),
            r(14, 17, 7, 19, 5, 110, 285, 35, 100, 2, 0),
            r(14, 17, 4, 22, 4, 110, 285, 35, 100, 3, 0),
        ],
    );
    utilt.hitboxes[0].targets = TARGETS_GROUNDED;
    utilt.hitboxes[1].targets = TARGETS_AERIAL;
    utilt.hitboxes[1].hitlag = 140;
    utilt.part_intangible = vec![PartIntangible {
        start: 13,
        end: 16,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a low sweep on frames 11-12, 10%, 25 degrees, BK 38, KS 86 (the foot and shin); the box at the body sends at 155
    // degrees. FAF 35. (Its 50% trip can only trip on slopes at 25 degrees, and the stages are flat.)
    w.moves[MoveId::DTilt as usize] = ref_move(
        35,
        0,
        0,
        255,
        &[
            r(11, 12, 15, 3, 6, 100, 25, 38, 86, 0, 0),
            r(11, 12, 9, 4, 6, 100, 25, 38, 86, 1, 0),
            r(11, 12, -1, 4, 4, 100, 155, 38, 86, 2, 0),
        ],
    );

    // Dash attack: a shoulder charge. Clean on frames 7-9: 10%, radius 8; late 10-16: 6%, radius 6; 65 degrees, BK 90, KS 67, hitlag
    // x1.25, shield damage +1 (110 / 117 percent [D]), shieldstun x1.5. The swept boxes (10.0 down to 8.0 and to 6.0 of the
    // reference's units up, 12.0 ahead) are circles at their ends. FAF 35.
    let mut dash = ref_move(
        35,
        0,
        0,
        255,
        &[
            r(7, 9, 15, 13, 8, 100, 65, 90, 67, 0, 0),
            r(7, 9, 15, 10, 8, 100, 65, 90, 67, 0, 0),
            r(10, 16, 15, 13, 6, 60, 65, 90, 67, 0, 0),
            r(10, 16, 15, 8, 6, 60, 65, 90, 67, 0, 0),
        ],
    );
    set_rows(&mut dash, &[0, 1, 2, 3], |h| {
        h.hitlag = 125;
        h.shieldstun = 150;
    });
    set_rows(&mut dash, &[0, 1], |h| h.shield_damage = 110);
    set_rows(&mut dash, &[2, 3], |h| h.shield_damage = 117);
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash: a flaming elbow on frames 19-22, BK 36: straight 19% at 43 degrees (KS 94), aimed up 20% at 45 (KS 94), aimed
    // down 20% at 38 (KS 87); the elbow and the upper arm. Charges on frame 11. FAF 60.
    let mut fsmash = ref_move(60, 0, 0, 255, &[]);
    for (aim, damage, angle, ks, elbow, arm) in [
        (AIM_STRAIGHT, 190, 43, 94, (13, 14), (7, 15)),
        (AIM_UP, 200, 45, 94, (12, 20), (7, 17)),
        (AIM_DOWN, 200, 38, 87, (12, 8), (7, 12)),
    ] {
        let mut rows = ref_move(
            60,
            0,
            0,
            255,
            &[
                r(19, 22, elbow.0, elbow.1, 5, damage, angle, 36, ks, 0, 0),
                r(19, 22, arm.0, arm.1, 4, damage, angle, 36, ks, 1, 0),
            ],
        );
        set_rows(&mut rows, &[0, 1], |h| h.aim = aim);
        fsmash.hitboxes.extend(rows.hitboxes);
    }
    fsmash.charge_at = Some(10);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: two kicks overhead. Hit 1 (frames 22-23): the scoop in front 7% at 110 degrees (set knockback 150, KS 90), the box
    // above it 7% at 96 (set knockback 80, KS 90, frame 22 only), the top 12% at 80 (BK 0, KS 90) and the box behind it 12% at 80
    // (BK 5, set knockback 10, KS 90). Hit 2 (frames 28-29): 14% at 83 (KS 80), 14% at 85 (KS 81), 13% at 70 (KS 88), BK 70. Every box
    // is on the root, converted [D]. Charges on frame 8. FAF 46. Each kicking leg cannot be hit while its kick is out (frames 22-23 and
    // 28-29).
    let mut usmash = ref_move(
        46,
        0,
        0,
        255,
        &[
            r(22, 23, 13, 9, 7, 70, 110, 0, 90, 0, 0),
            r(22, 22, 6, 21, 8, 70, 96, 0, 90, 1, 0),
            r(22, 23, 6, 35, 6, 120, 80, 0, 90, 2, 0),
            r(22, 23, 2, 26, 6, 120, 80, 5, 90, 3, 0),
            r(28, 29, 7, 36, 8, 140, 83, 70, 80, 0, 1),
            r(28, 29, 3, 26, 7, 140, 85, 70, 81, 1, 1),
            r(28, 29, 1, 20, 6, 130, 70, 70, 88, 2, 1),
        ],
    );
    usmash.hitboxes[0].fixed_kb = 150;
    usmash.hitboxes[1].fixed_kb = 80;
    usmash.hitboxes[3].fixed_kb = 10;
    usmash.charge_at = Some(7);
    usmash.part_intangible = vec![
        PartIntangible {
            start: 21,
            end: 22,
            parts: PART_LIMB,
        },
        PartIntangible {
            start: 27,
            end: 28,
            parts: PART_LIMB,
        },
    ];
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a kick in front on frames 19-20 (14%, 28 degrees, BK 30, KS 90), then behind on 29-30 (18%, 28 degrees sent
    // backward, BK 20, KS 84); the foot, the knee and the thigh. Charges on frame 11. FAF 49.
    let mut dsmash = ref_move(
        49,
        0,
        0,
        255,
        &[
            r(19, 20, 15, 4, 6, 140, 28, 30, 90, 0, 0),
            r(19, 20, 10, 4, 5, 140, 28, 30, 90, 1, 0),
            r(19, 20, 6, 5, 5, 140, 28, 30, 90, 2, 0),
            r(29, 30, -15, 4, 6, 180, 28, 20, 84, 0, 1),
            r(29, 30, -10, 4, 5, 180, 28, 20, 84, 1, 1),
            r(29, 30, -6, 5, 5, 180, 28, 20, 84, 2, 1),
        ],
    );
    dsmash.charge_at = Some(10);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: two kicks. Hit 1 (frames 7-8), 4%, set weight: on aerial targets the shin 74 degrees (BK 50, KS 25) and the foot
    // 80 (BK 50, KS 25); on grounded ones the shin 68 and the foot 74 (BK 65, KS 75, SDI x1.4); the hip 68 (BK 50, KS 25) on anyone.
    // Hit 2 (frames 13-15), the other leg: 6%, angle 361, BK 45, KS 120. Landing lag 7, autocancels on frames 1-3 and from 32, FAF 40.
    let mut nair = ref_move(
        40,
        7,
        3,
        32,
        &[
            r(7, 8, 6, 12, 8, 40, 74, 50, 25, 0, 0),
            r(7, 8, 6, 12, 8, 40, 68, 65, 75, 1, 0),
            r(7, 8, 11, 11, 6, 40, 80, 50, 25, 2, 0),
            r(7, 8, 11, 11, 6, 40, 74, 65, 75, 3, 0),
            r(7, 8, 0, 12, 4, 40, 68, 50, 25, 4, 0),
            r(13, 15, 11, 13, 7, 60, 361, 45, 120, 0, 1),
            r(13, 15, 6, 13, 8, 60, 361, 45, 120, 1, 1),
            r(13, 15, 0, 12, 5, 60, 361, 45, 120, 2, 1),
        ],
    );
    set_rows(&mut nair, &[0, 1, 2, 3, 4], |h| h.set_weight = true);
    set_rows(&mut nair, &[0, 2], |h| h.targets = TARGETS_AERIAL);
    set_rows(&mut nair, &[1, 3], |h| {
        h.targets = TARGETS_GROUNDED;
        h.sdi = 140;
    });
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air, the flying knee. Clean on frame 14 only: the knee 22% at 32 degrees (BK 30, KS 81, radius 3.75 reference units,
    // electric: 1.5x hitlag) and the hip 6% at the 361 angle (BK 35, KS 80), both hitlag x1.2; late on frames 15-30: 3%, angle 361,
    // BK 35, KS 80. The knee wins where both reach. Landing lag 18, autocancels on frames 1-4 and from 42, FAF 46. (The late hit's 20%
    // trip chance on grounded targets is not modelled: no trips yet.)
    let mut fair = ref_move(
        46,
        18,
        4,
        42,
        &[
            r(14, 14, 7, 13, 5, 220, 32, 30, 81, 0, 0),
            r(14, 14, 0, 8, 4, 60, 361, 35, 80, 1, 0),
            r(15, 30, 7, 13, 6, 30, 361, 35, 80, 0, 0),
            r(15, 30, 0, 8, 4, 30, 361, 35, 80, 1, 0),
        ],
    );
    set_rows(&mut fair, &[0, 1], |h| h.hitlag = 120);
    fair.hitboxes[0].effect = EFFECT_ELECTRIC;
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: a backhand behind. Clean on frames 10-11, 13%; late 12-15, 8%; angle 361 sent backward, BK 27, KS 97 (the fist, the
    // forearm and the upper arm). Landing lag 10, autocancels on frames 1-4 and from 19, FAF 36.
    w.moves[MoveId::BAir as usize] = ref_move(
        36,
        10,
        4,
        19,
        &[
            r(10, 11, -12, 13, 7, 130, 361, 27, 97, 0, 0),
            r(10, 11, -8, 14, 5, 130, 361, 27, 97, 1, 0),
            r(10, 11, -4, 14, 5, 130, 361, 27, 97, 2, 0),
            r(12, 15, -12, 13, 7, 80, 361, 27, 97, 0, 0),
            r(12, 15, -8, 14, 5, 80, 361, 27, 97, 1, 0),
            r(12, 15, -4, 14, 5, 80, 361, 27, 97, 2, 0),
        ],
    );

    // Up air: an overhead flip kick. Clean on frames 7-10: 10% (the shin) and 9% (the foot) at 66 degrees, BK 20, KS 96; late 11-12:
    // 9% and 8% at 30 degrees, BK 12, KS 80. Landing lag 10, autocancels from frame 24 (not early), FAF 32.
    w.moves[MoveId::UAir as usize] = ref_move(
        32,
        10,
        0,
        24,
        &[
            r(7, 10, 3, 22, 6, 100, 66, 20, 96, 0, 0),
            r(7, 10, 0, 27, 6, 90, 66, 20, 96, 1, 0),
            r(11, 12, 3, 22, 6, 90, 30, 12, 80, 0, 0),
            r(11, 12, 0, 27, 6, 80, 30, 12, 80, 1, 0),
        ],
    );

    // Down air: a stomp on frames 16-18, 14%. The box below the feet is a meteor (270 degrees, BK 10, KS 100) and wins where both
    // reach; the box at the feet sends at the 361 angle (BK 40, KS 100). Both on the root, converted [D]. Landing lag 12,
    // autocancels on frames 1-3 and from 39, FAF 45.
    w.moves[MoveId::DAir as usize] = ref_move(
        45,
        12,
        3,
        39,
        &[
            r(16, 18, 1, -7, 7, 140, 270, 10, 100, 0, 0),
            r(16, 18, 1, 1, 8, 140, 361, 40, 100, 1, 0),
        ],
    );
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: standing on frames 6-7 (FAF 36), dash 9-10 (FAF 44), pivot 10-11 (FAF 39). Each reach is the reference's grounded-target
    // grab box (a swept circle at 8.0 of its units up) as circles along it [D]. (Its second, thinner box for aerial targets is not
    // modelled: grabs catch only fighters on the ground here.)
    let grab_line = |first: u8, last: u8, radius: i32, xs: &[i32]| -> Vec<R> {
        xs.iter()
            .map(|&x| r(first, last, x, 10, radius, 0, 361, 0, 0, 0, 0))
            .collect()
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(36, 0, 0, 255, &grab_line(6, 7, 4, &[5, 9, 12])),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(44, 0, 0, 255, &grab_line(9, 10, 3, &[5, 10, 15])),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(39, 0, 0, 255, &grab_line(10, 11, 4, &[5, 10, 15, 20])),
        HIT_GRAB,
    );
    // Pummel: a knee, 1.3%, set knockback 30 (KS 100), hitlag x2.1, radius 6.2 at 10.0 up and 12.0 ahead [D]. The reference hits on
    // frame 1 with FAF 7; ours on frame 2 with FAF 8 (a pummel cannot hit on its first frame) [D].
    w.moves[MoveId::Pummel as usize] = reference_pummel(13, 15, 13, 8);
    // Throws. Forward: a hit on the held fighter on frame 11 (3.5%, hitlag x1.5), the throw on 13 (4%, 45 degrees, BK 55, KS 120), FAF
    // 33. Back: the hit on 12 (3.5%, hitlag x0.6), the throw on 13 (4%, 135 degrees: backward and up, BK 60, KS 130), the thrower ends
    // facing the other way, FAF 46. Up: the hit on 12 (4%, hitlag x1.6), the throw on 14 (5%, 84 degrees, BK 68, KS 100), FAF 38.
    // Down: the throw on 21 (6%, 68 degrees, BK 65, KS 70), FAF 40. (The reference hits on frames 11-12 and 12-13; one hit each.)
    let mut fthrow = throw_after_hit(33, (11, 35), r(13, 13, 13, 11, 10, 40, 45, 55, 120, 0, 1));
    fthrow.hitboxes[0].hitlag = 150;
    w.moves[MoveId::FThrow as usize] = fthrow;
    let mut bthrow = throw_after_hit(46, (12, 35), r(13, 13, 13, 11, 10, 40, 135, 60, 130, 0, 1));
    bthrow.hitboxes[0].hitlag = 60;
    bthrow.turns_around = true;
    w.moves[MoveId::BThrow as usize] = bthrow;
    let mut uthrow = throw_after_hit(38, (12, 40), r(14, 14, 13, 11, 10, 50, 84, 68, 100, 0, 1));
    uthrow.hitboxes[0].hitlag = 160;
    w.moves[MoveId::UThrow as usize] = uthrow;
    w.moves[MoveId::DThrow as usize] = with_kind(
        ref_move(
            40,
            0,
            0,
            255,
            &[r(21, 21, 13, 11, 10, 60, 68, 65, 70, 0, 0)],
        ),
        HIT_THROW,
    );
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: a sweeping kick climbing up, frames 22-24, 10%, 45 degrees, BK 90, KS 20, shield damage +1; intangible through
    // frame 24, FAF 56. Its swept box (12.5 to 3.5 of the reference's units ahead, 5.0 up) is three circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &[
            r(22, 24, 16, 6, 6, 100, 45, 90, 20, 0, 0),
            r(22, 24, 10, 6, 6, 100, 45, 90, 20, 0, 0),
            r(22, 24, 4, 6, 6, 100, 45, 90, 20, 0, 0),
        ],
    );
    set_rows(&mut ledge, &[0, 1, 2], |h| h.shield_damage = 110);
    ledge.intangible = 24;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack (lying face up): a spinning kick on frames 14-15 and again on 20-21, each sweeping both sides (12.0 ahead to 12.0
    // behind, 9.0 up [D]); 6%, 48 degrees, BK 80, KS 48, shield damage +8; intangible through frame 21, FAF 46. (The reference does not
    // give the box's radius here: 6 [E]. Its face-down and trip get-up attacks have no slot of their own.)
    let mut rows = Vec::new();
    for (first, group) in [(14u8, 0u8), (20, 1)] {
        for x in [15, 8, 0, -8, -15] {
            rows.push(r(first, first + 1, x, 11, 6, 60, 48, 80, 48, 0, group));
        }
    }
    let mut getup = ref_move(46, 0, 0, 255, &rows);
    let all: Vec<usize> = (0..rows.len()).collect();
    set_rows(&mut getup, &all, |h| h.shield_damage = 233);
    getup.intangible = 21;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}
