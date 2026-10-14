//! The Bracers moveset (the hauler body), modelled mechanically on a reference character: phase 1, the body's normals. A child
//! module of `moves` (it uses that file's private row builders). Data and sources: the eighth moveset's `REFERENCE_DATA.md`
//! (`docs/movesets/`), checked against ssbwiki.com's raw hitbox tables (13.0.1).

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

/// A point `at` reference tenths along a limb from `(sx, sy)` (a shoulder or hip, [E]) pointing along `(dx, dy)` (thousandths): the
/// reference's boxes on the arm and leg bones sit at a published distance along the bone [R] from a joint whose place in each pose is
/// ours [E].
const fn limb(sx: i32, sy: i32, dx: i32, dy: i32, at: i32) -> (i32, i32) {
    (sx + dx * at / 1000, sy + dy * at / 1000)
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

/// The Bracers moveset (the hauler body): huge fists and a heavy frame. Every normal follows the reference frame for frame (the
/// eighth moveset's `REFERENCE_DATA.md` in `docs/movesets/`, version 13.0.1, read from ssbwiki.com's raw hitbox tables): active
/// frames, FAF, landing lag and autocancel windows, the jab's continuable window, smash charge frames; damage, angles and angle types,
/// base and growth knockback, set knockback, hitlag multipliers, shield damage, hitstun modifiers, trip chances, grounded-only boxes,
/// hit groups, sweet and sour phases, no-clank boxes and boxes that clank without rebounding; the arm and head intangibility windows.
/// Radii and offsets on the reference's root bone (`top`) are its values converted (x 1.25, rounded half up to a tenth) [D]; boxes on
/// the arm, leg and head bones are placed on our body where that bone is in the move's pose [E]. The specials (phase 2) follow the
/// reference too (see `specials`); the carry is phase 3.
pub fn bracers() -> Weapon {
    let mut w = claws();
    // Fists and feet: the arm or leg throwing a hit can be hit (and its intangibility windows act on that circle).
    w.limbs = true;
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    specials(&mut w);
    w
}

// ---- Specials (phase 2) -------------------------------------------------------------------------------------------------------

/// The `ext` slots the specials use (`goto` numbers in the scripts: ext0 is 28 ... ext9 is 37).
pub const PUNCH_RELEASE: MoveId = MoveId::Ext0;
pub const PUNCH_RELEASE_AIR: MoveId = MoveId::Ext1;
pub const PUNCH_FULL: MoveId = MoveId::Ext2;
pub const PUNCH_FULL_AIR: MoveId = MoveId::Ext3;
pub const PUNCH_CANCEL: MoveId = MoveId::Ext4;
pub const HEADBUTT_AIR: MoveId = MoveId::Ext5;
pub const SPIN_AIR: MoveId = MoveId::Ext6;

/// Super armor (every hit withstood: the reference's) on move frames `start..=end`.
fn super_armor(start: u8, end: u8) -> Option<Armor> {
    Some(Armor {
        start,
        end,
        kind: ARMOR_SUPER,
        threshold: Fx::ZERO,
        subtract: false,
        min_kb: Fx::ZERO,
    })
}

fn specials(w: &mut Weapon) {
    // The claws' specials and their follow-up slots go: every special slot is the reference's or empty.
    for id in MoveId::all() {
        if id.is_special() || id.is_ext() {
            w.moves[id as usize] = Move::empty();
        }
    }
    punch(w);
    headbutt(w);
    spin(w);
}

fn spin(w: &mut Weapon) {
    // The spinning clothesline on the ground (the up special), every number [R] from ssbwiki.com's raw tables (13.0.1) unless marked;
    // box places on the arm bones [E] (the arm swung out level at chest height), on the root [D].
    // Hit 1 on frame 19, 5%, set knockback 90 (KS 30): against grounded targets 95 degrees (the fist, radius 6.0) and 80 (the arm, 5.0
    // and 4.0); against aerial ones the same boxes at the reference's 366 (E28, the vortex: 6.0.0). Shield +1 on the fist (120 [D]).
    let mut rows = Vec::new();
    for (aerial, angles) in [(false, [95i16, 80, 80]), (true, [366, 366, 366])] {
        for (i, (x, radius)) in [(180, 60), (100, 50), (50, 40)].into_iter().enumerate() {
            rows.push((
                ru(19, 19, x, 120, radius, 50, angles[i], 0, 30, i as u8, 0),
                aerial,
                90u16,
            ));
        }
    }
    // Hits 2-6, the loop: 25-27, 32-33, 40-42, 49-50, 55-58 (each a fresh hit: the script re-arms the group): the far arm 1.4% at the
    // autolink angle (367, forward, set knockback 90, the third hit's 80; KS 55, radius 4.0 at 25.0 ahead, 7.0 to 12.0 up), the arm
    // 1.4% at 20 degrees forward (set knockback 65, KS 100, radius 5.0, 12.5 ahead at 12.0 up), and behind 3.5% at 60 degrees always
    // backward (BK 35, KS 100, radius 5.7, 2.5 behind at 13.5 up: the ejection box); hitlag x0.8.
    let mut loops = Vec::new();
    for (n, (first, last)) in [(25u8, 27u8), (32, 33), (40, 42), (49, 50), (55, 58)]
        .into_iter()
        .enumerate()
    {
        let fkv = if n == 1 { 80 } else { 90 };
        for y in [70, 120] {
            loops.push((
                ru(first, last, 250, y, 40, 14, 367, 0, 55, 0, 1),
                fkv,
                ANGLE_FORWARD,
            ));
        }
        loops.push((
            ru(first, last, 125, 120, 50, 14, 20, 0, 100, 1, 1),
            65,
            ANGLE_FORWARD,
        ));
        loops.push((
            ru(first, last, -25, 135, 57, 35, 60, 35, 100, 2, 1),
            0,
            ANGLE_BACKWARD,
        ));
    }
    // The final hit on 62: 4% at 361 by position, BK 60, KS 157 (6.0.0), radius 8.0 along 20.0 ahead to 4.0 behind at 11.0 up, hitlag
    // x2.0.
    let finals: Vec<R> = along(200, -40, 40)
        .into_iter()
        .map(|x| ru(62, 62, x, 110, 80, 40, 361, 60, 157, 0, 2))
        .collect();
    let first_meta: Vec<(bool, u16)> = rows.iter().map(|(_, a, f)| (*a, *f)).collect();
    let loop_meta: Vec<(u16, u8)> = loops.iter().map(|(_, f, k)| (*f, *k)).collect();
    let all: Vec<R> = rows
        .into_iter()
        .map(|(r, ..)| r)
        .chain(loops.into_iter().map(|(r, ..)| r))
        .chain(finals)
        .collect();
    let mut ground = ref_move(105, 0, 0, 255, &all);
    let (n_first, n_loop) = (first_meta.len(), loop_meta.len());
    for (i, (aerial, fkv)) in first_meta.iter().enumerate() {
        let h = &mut ground.hitboxes[i];
        h.fixed_kb = *fkv;
        h.angle_type = ANGLE_BY_POSITION;
        h.targets = if *aerial {
            TARGETS_AERIAL
        } else {
            TARGETS_GROUNDED
        };
        if h.priority == 0 {
            h.shield_damage = shield(50, 10);
        }
    }
    for (i, (fkv, kind)) in loop_meta.iter().enumerate() {
        let h = &mut ground.hitboxes[n_first + i];
        h.fixed_kb = *fkv;
        h.angle_type = *kind;
        h.hitlag = 80;
    }
    for h in ground.hitboxes.iter_mut().skip(n_first + n_loop) {
        h.angle_type = ANGLE_BY_POSITION;
        h.hitlag = 200;
    }
    // Super armor 5-17 (6.0.0), the lower arm intangible on 19-24, FAF 105; pushes do not move it (decision 41: "immune to most
    // windboxes during the move", its frames not published: the whole move [E]). The slide and the loop's rhythm are the script's.
    // Over an edge it goes on as the aerial spin [E: gap 7].
    ground.armor = super_armor(4, 16);
    ground.part_intangible = vec![PartIntangible {
        start: 18,
        end: 23,
        parts: PART_LIMB,
    }];
    ground.push_immune_from = 0;
    ground.push_immune_to = ground.total_frames;
    ground.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bracers_spin.script"),
    ));
    ground.off_edge = Some(SPIN_AIR as u8);
    ground.air_version = Some(SPIN_AIR as u8);
    w.moves[MoveId::UpSpecial as usize] = ground;

    // In the air: hit 1 on 4-6 (5%, 85 degrees, BK 50, KS 10, radius 7.7 at 12.0 up, 7.0 behind and 4.0 ahead [D]); hits 2-13, twelve
    // 1% hits on 10-38 (the autolink angle, BK 55, KS 20: both arms out, radius 5.0 at 6.0 along them and 3.7 at -4.0 [E]), each a fresh
    // hit on its own span (the script); the final 2% on 39-43 (361, BK 72, KS 80, the same boxes). Angle type 0 throughout, shield +1
    // (120 / 200 / 150 [D]). The lower arms intangible on 10-38. 70 frames, then the special fall with 38 frames of landing lag; ledges
    // both sides from frame 14. Landing before the fall goes to the grounded version's frame 63: 42 frames of landing lag here [D].
    let mut rows = vec![
        ru(4, 6, -70, 120, 77, 50, 85, 50, 10, 0, 0),
        ru(4, 6, 40, 120, 77, 50, 85, 50, 10, 1, 0),
    ];
    for (first, last, damage, angle, bkb, kbg, group) in [
        (10u8, 38u8, 10, 367i16, 55i16, 20i16, 1u8),
        (39, 43, 20, 361, 72, 80, 2),
    ] {
        for (i, (x, radius)) in [(90, 50), (-90, 50), (30, 37), (-30, 37)]
            .into_iter()
            .enumerate()
        {
            rows.push(ru(
                first, last, x, 110, radius, damage, angle, bkb, kbg, i as u8, group,
            ));
        }
    }
    let mut air = ref_move(71, 42, 0, 255, &rows);
    set_all(&mut air, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.shield_damage = if h.damage == Fx::from_int(5) {
            shield(50, 10)
        } else if h.damage == Fx::ONE {
            shield(10, 10)
        } else {
            shield(20, 10)
        };
    });
    air.part_intangible = vec![PartIntangible {
        start: 9,
        end: 37,
        parts: PART_LIMB,
    }];
    air.helpless_after = true;
    air.fall_landing_lag = 38;
    air.ledge_front_from = 13;
    air.ledge_back_from = 13;
    air.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bracers_spin_air.script"),
    ));
    w.moves[SPIN_AIR as usize] = air;
}

fn headbutt(w: &mut Weapon) {
    // The headbutt (the side special, on the ground and in the air), every number [R] from ssbwiki.com's raw tables (13.0.1) unless
    // marked: super armor on 5-14 (both versions), the hit on 20-21, FAF 63; shield damage +25 (350 / 413 percent [D]); angle type 0 (by
    // position), hitlag and SDI x1.0, one hit. On the ground two bury boxes (E5) at 270 degrees that only reach grounded targets (10%,
    // BK 40, KS 40; radius 5.0 low in front, 19.0 to 7.0 ahead at 5.0 up, and 3.0 higher, 18.0 to 9.0 ahead at 13.0 up: 9.0.0) and an
    // 8% box at 70 (BK 15, KS 40, radius 10.0, 14.0 ahead at 11.0 up) for airborne ones. In the air one bury box for anyone (BK 30,
    // radius 6.5, 8.8 to 1.4 ahead at 5.0 up: it buries a grounded target and meteors an airborne one) and the same 8% box.
    let rows_ground: Vec<R> = along(190, 70, 40)
        .into_iter()
        .map(|x| ru(20, 21, x, 50, 50, 100, 270, 40, 40, 0, 0))
        .chain(
            along(180, 90, 40)
                .into_iter()
                .map(|x| ru(20, 21, x, 130, 30, 100, 270, 40, 40, 1, 0)),
        )
        .chain([ru(20, 21, 140, 110, 100, 80, 70, 15, 40, 2, 0)])
        .collect();
    let rows_air: Vec<R> = along(88, 14, 40)
        .into_iter()
        .map(|x| ru(20, 21, x, 50, 65, 100, 270, 30, 40, 0, 0))
        .chain([ru(20, 21, 140, 110, 100, 80, 70, 15, 40, 2, 0)])
        .collect();
    let build = |rows: &[R], grounded: bool| -> Move {
        let mut mv = ref_move(63, 0, 0, 255, rows);
        set_all(&mut mv, |h| {
            h.angle_type = ANGLE_BY_POSITION;
            if h.angle == 270 {
                h.effect = EFFECT_BURY;
                h.shield_damage = shield(100, 250);
                if grounded {
                    h.targets = TARGETS_GROUNDED;
                }
            } else {
                h.targets = TARGETS_AERIAL;
                h.shield_damage = shield(80, 250);
            }
        });
        mv.armor = super_armor(4, 13);
        // Ledges from frame 2 [E: conflict 4, "after frame 2" against "every frame"; the stricter reading], in front and behind.
        mv.ledge_front_from = 1;
        mv.ledge_back_from = 1;
        mv
    };
    // On the ground: the speed halved sideways and stopped upward on starting (x0.5 / x0) by a script; over an edge it goes on as
    // the aerial one [E: not published].
    let mut ground = build(&rows_ground, true);
    ground.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bracers_headbutt.script"),
    ));
    ground.off_edge = Some(HEADBUTT_AIR as u8);
    w.moves[MoveId::SideSpecial as usize] = ground;
    w.moves[MoveId::SideSpecial as usize].air_version = Some(HEADBUTT_AIR as u8);
    // In the air: x0.67 of the sideways speed and x0.5 of the vertical kept on starting, gravity x0.5 (0.0425 of the reference's
    // units: the body's 0.085 halved) while it lasts; touching down it goes on as the grounded one on the same frame [E: the
    // reference's landing is not published].
    let mut air = build(&rows_air, false);
    air.start_speed = 67;
    air.start_speed_y = 50;
    air.air_gravity = Some(Fx::from_ratio(425, 80_000));
    air.on_land = Some(MoveId::SideSpecial as u8);
    w.moves[HEADBUTT_AIR as usize] = air;
}

/// The arm thrown straight forward at chest height for the punches [E]: a point `at` reference tenths along it from the shoulder.
const fn punch_arm(at: i32) -> (i32, i32) {
    limb(30, 120, 1000, 0, at)
}

fn punch(w: &mut Weapon) {
    // The wind-up and charge (the neutral special, on the ground and in the air alike): the script (`bracers_punch_charge`) counts the
    // charge from frame 8 to 110 [R: 110 frames; the frame mapping is gap 1, E], releases on special, keeps the charge on a shield,
    // dodge or jump cancel, and stores a full charge by itself; a hit loses what was charged (the charge is only in the script's
    // variable until stored). It lands and carries on, and goes over an edge and carries on (the reference keeps charging) [E: gap 6].
    // The move is long enough to reach full from nothing (FAF 132); it never ends on its own before that.
    let mut charge = ref_move(132, 0, 0, 255, &[]);
    charge.land_continue = true;
    charge.off_edge = Some(MoveId::NSpecial as u8);
    charge.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bracers_punch_charge.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = charge;

    // The release (the uncharged punch, its damage raised by the charge: 10 + 18 x charge / 110 rounded down on the ground, 6 + the
    // same in the air: E29 `charge_add`), hitbox on the release's frames 19-20, FAF 63, the lower arm intangible on 4-20; the fist
    // (radius 5.5, 8.0 along the forearm), the forearm (4.0, -1.0) and the shoulder (3.0, -1.5) [R], the arm straight out at chest
    // height [E]; 361 by position (type 0), BK 45 KS 70 on the ground, BK 60 KS 65 in the air; it cannot rebound. The release's frame 1
    // is the frame special is pressed [E: gap 1]. The aerial one lands as the grounded one (E22b), the grounded one goes over an edge
    // as the aerial one; no special fall.
    let release = |damage: i32, bkb: i16, kbg: i16| -> Move {
        let rows: Vec<R> = [(140, 55, 0u8), (50, 40, 1), (-15, 30, 2)]
            .into_iter()
            .map(|(at, radius, id)| {
                let (x, y) = punch_arm(at);
                ru(19, 20, x, y, radius, damage, 361, bkb, kbg, id, 0)
            })
            .collect();
        let mut mv = ref_move(63, 0, 0, 255, &rows);
        set_all(&mut mv, |h| {
            h.angle_type = ANGLE_BY_POSITION;
            h.rebounds = false;
        });
        mv.part_intangible = vec![PartIntangible {
            start: 3,
            end: 19,
            parts: PART_LIMB,
        }];
        mv.charge_add = Some(ChargeAdd {
            full: 110,
            add: Fx::from_int(18),
            floor: true,
        });
        mv.script = Some(script(
            Kind::Fighter,
            include_str!("scripts/bracers_punch_release.script"),
        ));
        mv
    };
    let mut ground = release(100, 45, 70);
    ground.off_edge = Some(PUNCH_RELEASE_AIR as u8);
    let mut air = release(60, 60, 65);
    air.on_land = Some(PUNCH_RELEASE as u8);
    w.moves[PUNCH_RELEASE as usize] = ground;
    w.moves[PUNCH_RELEASE_AIR as usize] = air;

    // The full punch (its own, faster animation): hitbox 19-20, FAF 48, super armor 9-20 on the ground and 15-20 in the air (13.0.1);
    // the fist, forearm and shoulder 28% and the waist 18% (BK 35, KS 73) on the ground, every box 25% (BK 40, KS 72) in the air;
    // radii 5.0 / 6.5 / 4.0 / 3.0 [R]; 361 by position; it cannot rebound. It starts from the wind-up's frame 1 (the wind-up sees the
    // stored full charge on its first run), so every frame here is the reference's minus one more: counted from the press, the hit is on
    // 19-20 and the FAF 48 [D]. The aerial one lands as the grounded one keeping its frame, so landing in it moves to the grounded
    // armor window (the reference since 3.1.0); no special fall.
    let full = |damages: [i32; 4], bkb: i16, kbg: i16, armor_from: u8| -> Move {
        let waist = (10, 90);
        let mut rows = Vec::new();
        for (i, (at, radius)) in [(140, 50), (50, 65), (-15, 40)].into_iter().enumerate() {
            let (x, y) = punch_arm(at);
            rows.push(ru(
                18, 19, x, y, radius, damages[i], 361, bkb, kbg, i as u8, 0,
            ));
        }
        rows.push(ru(
            18, 19, waist.0, waist.1, 30, damages[3], 361, bkb, kbg, 3, 0,
        ));
        let mut mv = ref_move(47, 0, 0, 255, &rows);
        set_all(&mut mv, |h| {
            h.angle_type = ANGLE_BY_POSITION;
            h.rebounds = false;
        });
        mv.armor = super_armor(armor_from - 2, 18);
        mv
    };
    let mut ground = full([280, 280, 280, 180], 35, 73, 9);
    ground.off_edge = Some(PUNCH_FULL_AIR as u8);
    let mut air = full([250; 4], 40, 72, 15);
    air.on_land = Some(PUNCH_FULL as u8);
    w.moves[PUNCH_FULL as usize] = ground;
    w.moves[PUNCH_FULL_AIR as usize] = air;

    // The cancel: 4 frames of lag (FAF 5) [R], then the held shield, the buffered jump or the dodge comes out.
    let mut cancel = ref_move(5, 0, 0, 255, &[]);
    cancel.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bracers_punch_cancel.script"),
    ));
    w.moves[PUNCH_CANCEL as usize] = cancel;
}

fn jabs(w: &mut Weapon) {
    // Jab 1: a hook on frames 5-6, 4%, hitlag x1.5, angle type 3 throughout: two boxes at the 361 angle (BK 25 / KS 35, radius 6.0,
    // 11.5 of the reference's units up, 5.0 ahead; BK 20 / KS 30, radius 4.5, 10.5 up, 11.0 ahead [D]) and the fist pulling a fighter
    // in (180 degrees, BK 20, KS 20, radius 4.5, on the hand [E]: the reference's id 2, fighters only; its id 3, the same box for
    // objects, is not modelled). FAF 25. Attack pressed on frames 8-28 carries on into jab 2 from frame 8 (the continuable window).
    let mut jab1 = ref_move(
        25,
        0,
        0,
        255,
        &[
            ru(5, 6, 50, 115, 60, 40, 361, 25, 35, 0, 0),
            ru(5, 6, 110, 105, 45, 40, 361, 20, 30, 1, 0),
            ru(5, 6, 135, 110, 45, 40, 180, 20, 20, 2, 0),
        ],
    );
    set_all(&mut jab1, |h| {
        h.hitlag = 150;
        h.angle_type = ANGLE_FORWARD;
    });
    chain(&mut jab1, MoveId::Jab2, 8);
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2: an uppercut on its frames 4-6 (13.0.1; conflict 1), 6%, 70 degrees (type 3), BK 40, KS 100, hitlag x1.8: the fist (radius
    // 7.0, rising in front [E]), and boxes of radius 5.5 at 9.5 of the reference's units up, 7.0 ahead and 5.0 at 15.0 up, 5.0 ahead
    // [D]. FAF 32. It ends the chain. (Started on jab 1's frame 8 it hits on 11-13 with FAF 39: the reference's quickest loop.)
    let mut jab2 = ref_move(
        32,
        0,
        0,
        255,
        &[
            ru(4, 6, 60, 160, 70, 60, 70, 40, 100, 0, 0),
            ru(4, 6, 70, 95, 55, 60, 70, 40, 100, 1, 0),
            ru(4, 6, 50, 150, 50, 60, 70, 40, 100, 2, 0),
        ],
    );
    set_all(&mut jab2, |h| {
        h.hitlag = 180;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::Jab2 as usize] = jab2;
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: a backhand swipe on frames 7-9, angle 361 (type 3), BK 50, KS 78 (6.0.0). Straight 8%; aimed up or down 9%, and
    // the down one has a 15% trip chance. The fist (radius 5.5, 10.0 along the forearm), the elbow (5.0, 1.5 along it) and the
    // shoulder (3.5) [R], the arm straight out from the shoulder (3.0 ahead, 13.0 up) level, 30 degrees up or 30 down [E]. The lower
    // arm is intangible on frames 6-9. FAF 35 straight; the reference's angled versions last one frame longer, FAF 36 ("an oversight"):
    // the move is the angled length and its script ends it a frame sooner when it was not aimed (the `aimed` register, owner decision 52).
    let mut ftilt = ref_move(36, 0, 0, 255, &[]);
    ftilt.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/bracers_ftilt.script"),
    ));
    for (aim, damage, dir) in [
        (AIM_UP, 90, (866, 500)),
        (AIM_STRAIGHT, 80, (1000, 0)),
        (AIM_DOWN, 90, (866, -500)),
    ] {
        let mut rows = Vec::new();
        for (at, radius, id) in [(160, 55, 0u8), (75, 50, 1), (-5, 35, 2)] {
            let (x, y) = limb(30, 130, dir.0, dir.1, at);
            rows.push(ru(7, 9, x, y, radius, damage, 361, 50, 78, id, 0));
        }
        let mut mv = ref_move(35, 0, 0, 255, &rows);
        set_all(&mut mv, |h| {
            h.aim = aim;
            h.angle_type = ANGLE_FORWARD;
            if aim == AIM_DOWN {
                h.trip = 15;
            }
        });
        ftilt.hitboxes.extend(mv.hitboxes);
    }
    ftilt.part_intangible = vec![PartIntangible {
        start: 5,
        end: 8,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: the arm swept up from in front over the head to behind on frames 5-11, 100 degrees, BK 40, angle type 0 (by position:
    // a target behind is sent backward). The shoulder 8% (KS 105, radius 4.8, 2.7 along the upper arm), the forearm 9% (KS 110, 4.0,
    // 10.8 along the arm) and the fist end 10% (KS 115, 5.3, 8.0 along it) [R]; the arm from the shoulder (15.0 up) pointing up and
    // forward on 5-7, straight up on 8-9, up and back on 10-11 [E]. One hit. The lower arm is intangible on 5-11. FAF 39.
    let mut rows = Vec::new();
    for (first, last, dir) in [
        (5u8, 7u8, (500, 866)),
        (8, 9, (0, 1000)),
        (10, 11, (-500, 866)),
    ] {
        for (at, radius, damage, kbg, id) in [
            (27, 48, 80, 105, 0u8),
            (108, 40, 90, 110, 1),
            (80, 53, 100, 115, 2),
        ] {
            let (x, y) = limb(0, 150, dir.0, dir.1, at);
            rows.push(ru(first, last, x, y, radius, damage, 100, 40, kbg, id, 0));
        }
    }
    let mut utilt = ref_move(39, 0, 0, 255, &rows);
    set_all(&mut utilt, |h| h.angle_type = ANGLE_BY_POSITION);
    utilt.part_intangible = vec![PartIntangible {
        start: 4,
        end: 10,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a low slap along the floor on frames 6-7, 6%, angle 361 (type 0), BK 10, KS 80, +5 hitstun (13.0.1), a 40% trip
    // chance: the shoulder (radius 4.4, 2.3 along the upper arm), the forearm (4.4, 1.0 along it) and the hand (5.5, 8.5 along it) [R],
    // the arm reaching forward and down from a crouched shoulder (4.0 ahead, 7.0 up) [E]. The whole arm is intangible on 5-7. FAF 25.
    let mut rows = Vec::new();
    for (at, radius, id) in [(23, 44, 0u8), (80, 44, 1), (155, 55, 2)] {
        let (x, y) = limb(40, 70, 970, -242, at);
        rows.push(ru(6, 7, x, y, radius, 60, 361, 10, 80, id, 0));
    }
    let mut dtilt = ref_move(25, 0, 0, 255, &rows);
    set_all(&mut dtilt, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.hitstun_add = 5;
        h.trip = 40;
    });
    dtilt.part_intangible = vec![PartIntangible {
        start: 4,
        end: 6,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a rolling body attack. Clean on frames 9-12: 12%, 53 degrees, BK 85, KS 46, radius 7.0 at 8.0 of the reference's
    // units up and 8.0 ahead [D]; late on 13-24: 9%, 45 degrees, BK 60, KS 60, radius 5.5, 8.0 up, 5.2 ahead [D]; hitlag x1.25, angle
    // type 3. FAF 35.
    let mut dash = ref_move(
        35,
        0,
        0,
        255,
        &[
            ru(9, 12, 80, 80, 70, 120, 53, 85, 46, 0, 0),
            ru(13, 24, 52, 80, 55, 90, 45, 60, 60, 0, 0),
        ],
    );
    set_all(&mut dash, |h| {
        h.hitlag = 125;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Every smash box clanks but cannot rebound (the reference's `rebound=f`; E3g): a clank stops its hit for that frame and the smash
    // carries on.
    //
    // Forward smash: a two-handed clap in front on frames 22-23: the fists 22% (46 degrees, BK 21, KS 88, radius 5.7, 6.4 of the
    // reference's units up, 24.0 ahead) and the arms 21% (361, BK 18, KS 86, radius 3.7, 18.0 to 10.0 ahead) [D]; angle type 3,
    // hitlag x1.4, shield damage +1. Charges on frame 14. The head and lower arms are intangible on 20-26. FAF 55.
    let mut rows = vec![ru(22, 23, 240, 64, 57, 220, 46, 21, 88, 0, 0)];
    for x in along(180, 100, 40) {
        rows.push(ru(22, 23, x, 64, 37, 210, 361, 18, 86, 1, 0));
    }
    let mut fsmash = ref_move(55, 0, 0, 255, &rows);
    set_all(&mut fsmash, |h| {
        h.hitlag = 140;
        h.angle_type = ANGLE_FORWARD;
        h.rebounds = false;
        h.shield_damage = if h.priority == 0 {
            shield(220, 10)
        } else {
            shield(210, 10)
        };
    });
    fsmash.charge_at = Some(13);
    fsmash.part_intangible = vec![PartIntangible {
        start: 19,
        end: 25,
        parts: PART_HIGH | PART_LIMB,
    }];
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: an overhead clap on frames 14-15, 19%, 90 degrees (type 0), BK 40, KS 84, shield damage +4: the hands (radius 8.8,
    // 28.0 of the reference's units up) and the arms (7.0, 18.0 up, 5.0 ahead to 5.0 behind) [D]. Charges on frame 6. The head and
    // lower arms are intangible on 12-15. FAF 50.
    let mut rows = vec![ru(14, 15, 0, 280, 88, 190, 90, 40, 84, 0, 0)];
    for x in along(50, -50, 50) {
        rows.push(ru(14, 15, x, 180, 70, 190, 90, 40, 84, 1, 0));
    }
    let mut usmash = ref_move(50, 0, 0, 255, &rows);
    set_all(&mut usmash, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.rebounds = false;
        h.shield_damage = shield(190, 40);
    });
    usmash.charge_at = Some(5);
    usmash.part_intangible = vec![PartIntangible {
        start: 11,
        end: 14,
        parts: PART_HIGH | PART_LIMB,
    }];
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: both fists slammed down on either side. Clean on frames 11-12: the fists 17% (70 degrees, radius 4.0, 5.5 along the
    // forearms, low on both sides [E]) and the shoulders 14% (82 degrees, 4.0, 2.0 along the upper arms [E]); late on 13-14: the
    // shockwave 18% (55 degrees, BK 30, radius 6.0, 4.5 of the reference's units up, 19.0 ahead and 12.0 behind [D]) and the shoulders
    // 14% (98 degrees, 4.5); BK 35 (but the shockwave) and KS 88 throughout, angle type 0, shield damage +1. One hit. Charges on frame
    // 2. The lower arms are intangible on 11-14. FAF 56.
    let mut rows = Vec::new();
    for side in [1, -1] {
        rows.push(ru(11, 12, 150 * side, 30, 40, 170, 70, 35, 88, 0, 0));
    }
    for side in [1, -1] {
        rows.push(ru(11, 12, 60 * side, 90, 40, 140, 82, 35, 88, 2, 0));
    }
    rows.push(ru(13, 14, 190, 45, 60, 180, 55, 30, 88, 0, 0));
    rows.push(ru(13, 14, -120, 45, 60, 180, 55, 30, 88, 1, 0));
    for side in [1, -1] {
        rows.push(ru(13, 14, 60 * side, 90, 45, 140, 98, 35, 88, 2, 0));
    }
    let mut dsmash = ref_move(56, 0, 0, 255, &rows);
    set_all(&mut dsmash, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.rebounds = false;
        h.shield_damage = if h.damage == Fx::from_int(17) {
            shield(170, 10)
        } else if h.damage == Fx::from_int(18) {
            shield(180, 10)
        } else {
            shield(140, 10)
        };
    });
    // (The reference's shoulder ids are 2 and 3, the shockwave's 0 and 1: written above with the front and back pairs sharing an id.)
    for (i, h) in dsmash.hitboxes.iter_mut().enumerate() {
        if matches!(i, 1 | 3 | 7) {
            h.priority += 1;
        }
    }
    dsmash.charge_at = Some(1);
    dsmash.part_intangible = vec![PartIntangible {
        start: 10,
        end: 13,
        parts: PART_LIMB,
    }];
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: a spin with both arms out. Clean early on frame 10: the arms (radius 5.0, 5.0 along the forearms, out in front and
    // behind at 11.0 of the reference's units up [E]), the hip (5.7 [E]) and a box far in front (5.0 at 8.0 up, 15.0 ahead [D], 6.0.0);
    // clean late on 11-13 the same without the far box; all 12% (6.0.0), BK 20. Late on 14-26: 9%, BK 10. Angle 361, KS 100, type 0.
    // One hit. Landing lag 10, autocancels on frames 1-9 and from 27, FAF 39.
    let arm_boxes = |first: u8, last: u8, damage: i32, bkb: i16| -> Vec<R> {
        vec![
            ru(first, last, 100, 110, 50, damage, 361, bkb, 100, 0, 0),
            ru(first, last, -100, 110, 50, damage, 361, bkb, 100, 1, 0),
            ru(first, last, 0, 115, 57, damage, 361, bkb, 100, 2, 0),
        ]
    };
    let mut rows = arm_boxes(10, 10, 120, 20);
    rows.push(ru(10, 10, 150, 80, 50, 120, 361, 20, 100, 3, 0));
    rows.extend(arm_boxes(11, 13, 120, 20));
    rows.extend(arm_boxes(14, 26, 90, 10));
    let mut nair = ref_move(39, 10, 9, 27, &rows);
    set_all(&mut nair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: both fists swung overhead and down in front. Clean on frames 18-20: the fists 16% (radius 5.5) and the forearms 16%
    // (4.5), angle 361, BK 20, KS 85, high in front [E]. Late on 21-23: the fists a meteor (15%, 270 degrees, BK 30, KS 100, radius 5.5,
    // low in front [E]; on grounded targets too) and the forearms 13% (361, BK 30, KS 100, 5.0); hitlag x1.5. Angle type 3. Landing lag
    // 17, autocancels from frame 56 (2.0.0), FAF 56.
    let mut fair = ref_move(
        56,
        17,
        0,
        56,
        &[
            ru(18, 20, 120, 190, 55, 160, 361, 20, 85, 0, 0),
            ru(18, 20, 70, 160, 45, 160, 361, 20, 85, 1, 0),
            ru(21, 23, 140, 60, 55, 150, 270, 30, 100, 0, 0),
            ru(21, 23, 80, 90, 50, 130, 361, 30, 100, 1, 0),
        ],
    );
    for (i, h) in fair.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        if i >= 2 {
            h.hitlag = 150;
        }
    }
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: a kick behind. Clean on frames 7-8: 13%, BK 20, KS 97 (the foot, radius 7.0; the knee and hip, 5.0); late on 9-16: 8%,
    // BK 10, KS 100 (6.0; 4.0); angle 361 always sent backward (type 4). The leg straight out behind at 9.0 of the reference's units up
    // [E]. Landing lag 11, autocancels on frames 1-6 and from 31, FAF 32.
    let leg =
        |first: u8, last: u8, damage: i32, bkb: i16, kbg: i16, foot: i32, rest: i32| -> Vec<R> {
            vec![
                ru(first, last, -150, 85, foot, damage, 361, bkb, kbg, 0, 0),
                ru(first, last, -90, 90, rest, damage, 361, bkb, kbg, 1, 0),
                ru(first, last, -40, 95, rest, damage, 361, bkb, kbg, 2, 0),
            ]
        };
    let mut rows = leg(7, 8, 130, 20, 97, 70, 50);
    rows.extend(leg(9, 16, 80, 10, 100, 60, 40));
    let mut bair = ref_move(32, 11, 6, 31, &rows);
    set_all(&mut bair, |h| h.angle_type = ANGLE_BACKWARD);
    w.moves[MoveId::BAir as usize] = bair;

    // Up air: an arcing headbutt on frames 6-10, 13%, 90 degrees (type 0), BK 32, KS 90, radius 7.5 on the head, over the body [E].
    // The head is intangible on 5-10 (the high circle, and the limb circle that reaches out toward the head's box). Landing lag 15, autocancels on frames 1-5 and from 26, FAF 38.
    let mut uair = ref_move(
        38,
        15,
        5,
        26,
        &[ru(6, 10, 25, 225, 75, 130, 90, 32, 90, 0, 0)],
    );
    set_all(&mut uair, |h| h.angle_type = ANGLE_BY_POSITION);
    uair.part_intangible = vec![PartIntangible {
        start: 4,
        end: 9,
        parts: PART_HIGH | PART_LIMB,
    }];
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: a stomp on frames 14-16: below the feet a meteor (16%, 270 degrees, BK 30, KS 90, radius 7.0, 5.0 of the reference's
    // units below, 1.0 ahead; on grounded targets too) and the leg at the body (13%, 361, BK 20, KS 90, radius 5.8, 6.0 up) [D]; angle
    // type 0. Landing lag 14, autocancels on frames 1-2 and from 50, FAF 55.
    let mut dair = ref_move(
        55,
        14,
        2,
        50,
        &[
            ru(14, 16, 10, -50, 70, 160, 270, 30, 90, 0, 0),
            ru(14, 16, 0, 60, 58, 130, 361, 20, 90, 1, 0),
        ],
    );
    set_all(&mut dair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::DAir as usize] = dair;
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: standing on frames 8-10 (FAF 39), dash 11-13 (FAF 47), pivot 12-14 (FAF 42). Each reach is the reference's grounded-target
    // grab box (a swept circle at 7.0 of its units up: 4.0 to 13.7 ahead, radius 5.8; 4.0 to 15.9, 4.6; behind 4.0 to 21.5, 5.8,
    // written in front since the engine turns a pivot grab round) as circles along it [D]. (Its second, thinner box for aerial
    // targets is not modelled: grabs catch only fighters on the ground here.)
    let grab_line = |first: u8, last: u8, radius: i32, x0: i32, x1: i32| -> Vec<R> {
        along(x0, x1, 30)
            .into_iter()
            .map(|x| ru(first, last, x, 70, radius, 0, 361, 0, 0, 0, 0))
            .collect()
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(39, 0, 0, 255, &grab_line(8, 10, 58, 40, 137)),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(47, 0, 0, 255, &grab_line(11, 13, 46, 40, 159)),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(42, 0, 0, 255, &grab_line(12, 14, 58, 40, 215)),
        HIT_GRAB,
    );
    // Pummel: a headbutt, 1.6% on frame 2, set knockback 30 (KS 100), hitlag x2.3, radius 6.0 at 13.0 of the reference's units up and
    // 18.0 ahead [D], no clank. FAF 8. (The reference's hit is on frame 2, so no shift is needed.)
    let mut pummel = reference_pummel(16, u(180), u(130), u(60));
    set_all(&mut pummel, |h| {
        h.hitlag = 230;
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
    });
    w.moves[MoveId::Pummel as usize] = pummel;
    // Throws (no pre-hits; the "interrupted" rows and the thrower's invincibility are not modelled: house limits). Back: turns round
    // and throws behind on frame 15, 11% at 40 degrees, BK 60, KS 65, FAF 40. Up: on 14, 9% at 90, BK 70, KS 50, FAF 44. Down: on 19,
    // 7% at the 361 angle (along the ground), BK 60, KS 50, FAF 55. Forward: the reference's forward throw is the carry (phase 3);
    // until then a placeholder plain throw with the forward cargo throw's numbers (on 15, 12% at 60, BK 90, KS 46, FAF 46) [E].
    w.moves[MoveId::FThrow as usize] = with_kind(
        ref_move(
            46,
            0,
            0,
            255,
            &[r(15, 15, 13, 11, 10, 120, 60, 90, 46, 0, 0)],
        ),
        HIT_THROW,
    );
    let mut bthrow = with_kind(
        ref_move(
            40,
            0,
            0,
            255,
            &[r(15, 15, -13, 11, 10, 110, 40, 60, 65, 0, 0)],
        ),
        HIT_THROW,
    );
    bthrow.turns_around = true;
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = with_kind(
        ref_move(
            44,
            0,
            0,
            255,
            &[r(14, 14, 13, 11, 10, 90, 90, 70, 50, 0, 0)],
        ),
        HIT_THROW,
    );
    w.moves[MoveId::DThrow as usize] = with_kind(
        ref_move(
            55,
            0,
            0,
            255,
            &[r(19, 19, 13, 11, 10, 70, 361, 60, 50, 0, 0)],
        ),
        HIT_THROW,
    );
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: a sweep climbing up, frames 23-25, 10%, 45 degrees (type 3: forward on both sides), BK 90, KS 20, shield damage +1
    // (110 [D]), no clank; intangible through frame 25 (on the first grab; the shared decay after), FAF 56. Its swept box (12.0 of the
    // reference's units ahead to 5.5 behind, 5.5 up, radius 5.5) as circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along(120, -55, 45)
            .into_iter()
            .map(|x| ru(23, 25, x, 55, 55, 100, 45, 90, 20, 0, 0))
            .collect::<Vec<_>>(),
    );
    set_all(&mut ledge, |h| {
        h.shield_damage = shield(100, 10);
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
    });
    ledge.intangible = 25;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack (lying face up, the reference's "back" floor attack): behind on frames 18-19 (19.0 to 4.5 of the reference's units
    // behind), then in front on 23-24 (19.0 to 5.5 ahead), 6.5 up, radius 6.5; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8
    // (214 [D]), no clank; intangible through frame 24, FAF 46. (The face-down and trip get-up attacks have no slot of their own.)
    let mut rows = Vec::new();
    for x in along(-190, -45, 50) {
        rows.push(ru(18, 19, x, 65, 65, 70, 48, 80, 48, 0, 0));
    }
    for x in along(190, 55, 50) {
        rows.push(ru(23, 24, x, 65, 65, 70, 48, 80, 48, 0, 1));
    }
    let mut getup = ref_move(46, 0, 0, 255, &rows);
    set_all(&mut getup, |h| {
        h.shield_damage = shield(70, 80);
        h.angle_type = ANGLE_BY_POSITION;
        h.clanks = false;
    });
    getup.intangible = 24;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}
