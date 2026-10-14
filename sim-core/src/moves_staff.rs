//! The Staff moveset (the seer body), modelled mechanically on a reference character: phase 1, the body's normals. A child module of
//! `moves` (it uses that file's private row builders).

use super::*;

/// The rapid jab's first cycle, its later cycles and its finisher (slots 39, 40 and 41: `goto` numbers in the jab scripts).
const RAPID: MoveId = MoveId::Ext10;
const RAPID_LOOP: MoveId = MoveId::Ext11;
const RAPID_END: MoveId = MoveId::Ext12;

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

/// Circles along a reference capsule from `x0` to `x1` (reference tenths, at height `y`), about `step` apart: the ends and evenly
/// spaced points between [D].
fn along(x0: i32, x1: i32, step: i32) -> Vec<i32> {
    let n = ((x1 - x0).abs() / step.max(1)).max(1);
    (0..=n).map(|i| x0 + (x1 - x0) * i / n).collect()
}

/// The Staff moveset (the seer body): a staff, a shield and light. Every normal follows the reference frame for frame
/// (`docs/movesets/staff/REFERENCE_DATA.md`, version 13.0.1): active frames, FAF, landing lag and autocancel windows; damage,
/// angles and angle types, base and growth knockback, set knockback and set weight, hitlag, SDI and shieldstun multipliers, hitstun
/// modifiers (the neutral air's late loop takes 13 frames off), shield damage, grounded- and aerial-only boxes, push hits (the
/// smashes' wind); the jab's entry rule into the rapid jab; the multi-hit loops of the up tilt, neutral air and up air with their last
/// hits; the dash attack's and back air's partial intangibility. Radii and offsets on the reference's root bone (`top`) are its values
/// converted (x 1.25, rounded half up to a tenth) [D]; boxes on the staff (`stick`) are placed on our body where the staff is in the
/// move's pose [E]. The specials are still the claws' (phase 2 builds the reference's).
pub fn staff() -> Weapon {
    let mut w = claws();
    // A held staff and shield: the hits reach out beyond the body, so no limb reaches out to be hit with them.
    w.limbs = false;
    jabs(&mut w);
    tilts(&mut w);
    smashes(&mut w);
    aerials(&mut w);
    grabs_and_throws(&mut w);
    ledge_and_floor(&mut w);
    // The reference's no-clank moves (`clang=f` on every box, ssbwiki.com raw hitbox tables, 13.0.1; sim 73): the rapid jab and its
    // finisher, the dash attack, the three smashes (their wind too), the down air, the pummel, the ledge and get-up attacks.
    for id in [
        RAPID,
        RAPID_LOOP,
        RAPID_END,
        MoveId::DashAttack,
        MoveId::FSmash,
        MoveId::USmash,
        MoveId::DSmash,
        MoveId::DAir,
        MoveId::Pummel,
        MoveId::LedgeAttack,
        MoveId::GetUpAttack,
    ] {
        for h in &mut w.moves[id as usize].hitboxes {
            h.clanks = false;
        }
    }
    // Phase 2: the specials (none of the claws' follow-up slots belong to the staff).
    for id in [MoveId::Ext0, MoveId::Ext1, MoveId::Ext2] {
        w.moves[id as usize] = Move::empty();
    }
    warp(&mut w);
    counter_and_barrier(&mut w);
    explosive_flame(&mut w);
    reticle(&mut w);
    w
}

/// Neutral special, the reticle and its three shots (the reference's, ssbwiki.com 13.0.1; `REFERENCE_DATA.md` "the reticle").
///
/// * The search on frames 7-14 (the `lock` register): the nearest opponent that can be hit, in front, chest within 120 reference
///   units (15 world) of the seer's chest, with nothing solid between [R rules; E: the exact "in front" region, chest to chest
///   rather than the search circle against the hurtbox, pass-through platforms not blocking (gaps 2)]. The reticle stays where
///   the target's chest was [R].
/// * With a lock, three shots made on frames 23, 29 and 35 (hitting from the next frame) from the staff (1.0 ahead, 1.3 up [E]),
///   straight at the reticle at 6 reference units a frame (0.75 world) [R speed; E: the path, gap 1], life 25 [R]: 3.5%, angle 361
///   forward, BK 40, KS 41, radius 2.3 reference units, shield damage -1.7 (51%) [R]; not piercing, reflectable, they clank [R].
///   Without a lock nothing comes out [R].
/// * FAF 68 [R]; x0.6 horizontal speed in the air [R]; the aerial descent stall is not published (the body's physics) [E: gap 3];
///   touching down carries the move on [E: gap 4].
fn reticle(w: &mut Weapon) {
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    let mut shot = row(&r(1, 1, 0, 0, u(23), 35, 361, 40, 41, 0, 0));
    shot.shield_damage = 51;
    shot.angle_type = ANGLE_FORWARD;
    let mut mv = ref_move(68, 0, 0, 255, &[]);
    mv.projectile = Some(ProjectileSpawn {
        // (Only the script fires it.)
        frame: 255,
        x: Fx::ONE,
        y: tenths(13),
        speed: su(6000),
        life: 25,
        hitbox: shot,
        end_damage: Fx::from_ratio(35, 10),
        arc: ProjectileArc::NONE,
        phases: [None; MAX_PHASES],
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
    });
    mv.start_speed = 60;
    mv.land_continue = true;
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/staff_reticle.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = mv;
}

/// Side special, the explosive flame (the reference's, ssbwiki.com 13.0.1; `REFERENCE_DATA.md` "Explosive Flame"): a stationary
/// explosion summoned at a distance, a projectile (E3a: it pierces, hits each fighter again 5 frames after its last hit on it, and
/// grows by its age).
///
/// * The point is fixed where the seer stood on frame 1: 45 reference units ahead (5.625 world units), 75 (9.375) when the special
///   was input like a smash attack (the `flick` register) [R]; 10 up (1.25) [E: gap 5, its height is not published]. The explosion is
///   made on frame 27 at that point, whatever the seer did since (it fell, it was carried) [R]; a B-reverse in the first frames
///   turns the point round with the seer [E].
/// * Six loop hits from frame 28 (its age 1), every 5 frames on a fighter inside it: 1.5%, 160 degrees **away from the explosion's
///   centre by position** (the target is drawn back into the middle: the trap), BK 50, KS 100, SDI x0.4, shield damage -0.7 (53%)
///   [R]; the radius grows on 28 / 32 / 36 / 40 / 44 / 48 / 52 (4.8 / 6.0 / 7.2 / 8.4 / 9.6 / 10.8 / 12.0 reference units) [R]; the loop
///   ends after 55, nothing on 56 [R].
/// * The last hit on frame 57 (its own group): 5.5%, 84 degrees the way the seer faced, BK 60, KS 141, radius 15.5, hitlag x1.5, SDI
///   x0.4, shield damage -2.7 (51%) [R]. Never clanks (`clang=f`) [R]; reflectable [R].
/// * FAF 72; x0.8 horizontal speed on starting in the air [R]; the body's air physics, and touching down carries the move on [E:
///   gaps 4 and 6].
fn explosive_flame(w: &mut Weapon) {
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    let flame = |radius: i32, damage: i32, angle: i16, bkb: i16, kbg: i16, group: u8| {
        let mut h = row(&r(1, 1, 0, 0, u(radius), damage, angle, bkb, kbg, 0, group));
        h.sdi = 40;
        h.clanks = false;
        h
    };
    let looped = |radius: i32| {
        let mut h = flame(radius, 15, 160, 50, 100, 0);
        h.angle_type = ANGLE_BY_POSITION;
        h.shield_damage = 53;
        h
    };
    let mut last = flame(155, 55, 84, 60, 141, 1);
    last.angle_type = ANGLE_FORWARD;
    last.hitlag = 150;
    last.shield_damage = 51;
    let mut gap = looped(0);
    gap.radius = Fx::ZERO;
    let mut mv = ref_move(72, 0, 0, 255, &[]);
    mv.projectile = Some(ProjectileSpawn {
        // (Made by the script on move frame 26, the reference's 27; it first hits the frame after, at age 1.)
        frame: 255,
        x: tenths(u(450)),
        y: tenths(u(100)),
        speed: Fx::ZERO,
        life: 31,
        hitbox: looped(48),
        end_damage: Fx::from_ratio(15, 10),
        arc: ProjectileArc::NONE,
        phases: phase_list(&[
            (5, looped(60)),
            (9, looped(72)),
            (13, looped(84)),
            (17, looped(96)),
            (21, looped(108)),
            (25, looped(120)),
            (29, gap),
            (30, last),
        ]),
        pierce: true,
        rehit: 5,
        burst: None,
        charged: None,
        // (A standing explosion, not a thing of its own: no pocket can take it, E42 [E].)
        tool: Tool {
            pocket: POCKET_NEVER,
            ..Tool::NONE
        },
    });
    mv.start_speed = 80;
    mv.land_continue = true;
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/staff_flame.script"),
    ));
    w.moves[MoveId::SideSpecial as usize] = mv;
}

/// The counterattack and the reflection (slots 29 and 30).
const COUNTERATTACK: MoveId = MoveId::Ext0;
const REFLECTION: MoveId = MoveId::Ext1;

/// The down special's air physics (the reference's parameters, in its units a frame), shared by the stance and its two answers [R]
/// (the answers' own aerial physics are not published: [E] the stance's): horizontal speed x0.33 on starting, air friction 0.0025,
/// gravity 0.04, top fall speed 1.2, top rising speed 0.8. Touching down carries the move on along the ground [E: gap 4].
fn stance_air(m: &mut Move) {
    m.start_speed = 33;
    m.air_friction = Some(Fx::from_ratio(25, 80_000));
    m.air_gravity = Some(su(40));
    m.air_max_fall = Some(su(1200));
    m.air_max_rise = Some(su(800));
    m.land_continue = true;
}

/// Down special: the counter and the reflect barrier, one stance with two separate answers (the reference's, ssbwiki.com 13.0.1;
/// `REFERENCE_DATA.md` "Down special"). A melee hit is countered; a projectile is reflected; never both.
///
/// * The stance (down special): FAF 71; intangible 6-12. It **counters melee hits only** on frames 7-34 (E10c `melee_only`), caught
///   on its counter bubble (radius 1.4 at 1.3 up, 0.2 ahead [R/D], sim 78) and never a push: the counterattack deals 1.3 times the caught damage, at least 10% and at most 50% (E10c `max_damage`). On the same frames a
///   **projectile search box** (radius 12.3 reference units on the chest: 1.5 world units at 0.2 ahead, 1.3 up [D]) starts the
///   reflection the moment a shot touches it (E10c `detect_then`).
/// * The counterattack (ext0, decision 22): the hit on frames 5-7 [R: ssbwiki.com, dragdown; UFD and kuroganehammer.com say 4-6],
///   intangible 1-5 [R: the majority; ssbwiki.com 1-6], FAF 37. Two boxes, both the caught damage: id 0 BK 50 / KS 70 (radius 7.0 at
///   10.5 up, 13.0 to 14.5 ahead) wins over id 1 BK 30 / KS 65 (radius 9.0, 9.0 to 20.0 ahead) [D positions on the root]; angle 361
///   sent forward, hitlag x1.5, no clank.
/// * The reflection (ext1): FAF 35. The body reflector on frames 1-4 (radius 11.5 at 10 up, 1.5 ahead: 1.4 at 1.3 up, 0.2 ahead [D])
///   and the barrier, a standing capsule (radius 3.8, from 2 to 21 reference units up, 10 ahead: radius 0.5, centre 1.4 up and 1.3
///   ahead, half height 1.2 [D]); both reflect at x1.4 damage, x1.25 speed, x1 lifetime, up to 50% shots [R]. The barrier's push
///   (frames 2-11): set knockback 50 with set weight at angle 361 forward, re-applied every frame as the smashes' wind (radius 5.0,
///   at the barrier from 5 to 18 up) [R numbers, D positions]. **Owner-chosen approximation (decision 20)**: the reference's barrier is
///   a separate object lasting 45 frames (ten past the reflection's FAF) that stays when the seer is hit; here it is part of the
///   reflection move and ends with it. A reflected shot turns its horizontal speed round only [E: gap 12].
fn counter_and_barrier(w: &mut Weapon) {
    let mut stance = ref_move(71, 0, 0, 255, &[]);
    stance.intangible_from = 5;
    stance.intangible = 12;
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    stance.counter = Some(Counter {
        start: 6,
        end: 33,
        then: COUNTERATTACK as u8,
        percent: 130,
        min_damage: Fx::from_int(10),
        max_damage: Fx::from_int(50),
        melee_only: true,
        // The counter bubble (ssbwiki.com 13.0.1, the parameters): radius 11.5 on the root bone at offset [0, 10, 1.5], so 10 up
        // and 1.5 ahead [R]; radius 1.4 at 1.3 up, 0.2 ahead [D] (the body reflector's circle).
        bubble: Some(CounterBubble {
            x: tenths(u(15)),
            y: tenths(u(100)),
            half_height: Fx::ZERO,
            radius: tenths(u(115)),
        }),
    });
    stance.reflector = Some(Reflector {
        damage_percent: 140,
        speed_percent: 125,
        max_damage: Fx::from_int(50),
        detect_start: 6,
        detect_end: 33,
        detect_x: tenths(u(15)),
        detect_y: tenths(u(100)),
        detect_radius: tenths(u(123)),
        detect_then: Some(REFLECTION as u8),
        ..Reflector::NONE
    });
    stance_air(&mut stance);
    w.moves[MoveId::DownSpecial as usize] = stance;

    let mut strike = ref_move(37, 0, 0, 255, &[]);
    for x in [130, 145] {
        strike
            .hitboxes
            .push(row(&ru(5, 7, x, 105, 70, 100, 361, 50, 70, 0, 0)));
    }
    for x in along(90, 200, 55) {
        strike
            .hitboxes
            .push(row(&ru(5, 7, x, 105, 90, 100, 361, 30, 65, 1, 0)));
    }
    for h in &mut strike.hitboxes {
        h.hitlag = 150;
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
    }
    strike.counter_strike = true;
    strike.intangible = 5;
    stance_air(&mut strike);
    w.moves[COUNTERATTACK as usize] = strike;

    let mut reflection = ref_move(
        35,
        0,
        0,
        255,
        &along(50, 180, 65)
            .into_iter()
            .map(|y| ru(2, 11, 110, y, 50, 0, 361, 0, 100, 0, 0))
            .collect::<Vec<_>>(),
    );
    for h in &mut reflection.hitboxes {
        h.push = true;
        h.fixed_kb = 50;
        h.set_weight = true;
        h.angle_type = ANGLE_FORWARD;
    }
    // (The push, reference frames 2-11, lands again every frame.)
    reflection.rehit = Some((1, 1));
    reflection.reflector = Some(Reflector {
        start: 0,
        end: reflection.total_frames,
        x: tenths(u(100)),
        y: tenths(u(115)),
        radius: tenths(u(38)),
        half_height: tenths(u(95)),
        damage_percent: 140,
        speed_percent: 125,
        max_damage: Fx::from_int(50),
        body_start: 0,
        body_end: 3,
        body_x: tenths(u(15)),
        body_y: tenths(u(100)),
        body_radius: tenths(u(115)),
        ..Reflector::NONE
    });
    stance_air(&mut reflection);
    w.moves[REFLECTION as usize] = reflection;
}

/// A reference speed or acceleration in thousandths of its units a frame, as world units a frame (1 world unit = 8) [D].
const fn su(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

/// Up special, the warp (the reference's, ssbwiki.com 13.0.1; `REFERENCE_DATA.md` "Warp").
///
/// * Startup on frames 1-15 with gravity 0.03 and a top fall speed of 0.75 (reference units a frame) [R].
/// * On frame 16 the stick picks the direction (straight up when near neutral, and how far it is pushed does not matter [E]); 17
///   frames of travel (16-32) at 3.5 reference units a frame (59.5 in all, 7.44 world units) [R], intangible on 16-32 [R], no hitbox.
///   The travel follows the ground if it meets it (it never sinks in, and glides on to the edge) and stops at walls and ceilings [E:
///   gap 9]; touching down during it does not end the move (`lands_from`, the reference's frame 33).
/// * The reappearance on frame 33 at half the speed [R], then the body's own air physics [E: gap 10] or the ground's friction; FAF 68
///   [R]. Ending in the air leaves the fighter helpless with 22 frames of landing lag [R]; landing during the reappearance costs the
///   same 22 [R]. Sliding off an edge during a grounded reappearance cancels it into an ordinary fall (the shared special-move edge
///   cancel) [R].
/// * Ledges can be caught from the reappearance on, in front and behind [E: gap 8; "grabs ledges more easily" since 2.0.0, no
///   numbers published].
fn warp(w: &mut Weapon) {
    let mut warp = ref_move(68, 22, 0, 255, &[]);
    warp.air_blocks = vec![AirBlock {
        from: 0,
        to: 14,
        gravity: Some(su(30)),
        max_fall: Some(su(750)),
        friction: None,
        drift: 100,
        accel: 100,
        speed: None,
        accel_stick: None,
    }];
    // Intangible on the reference's frames 16-32 (move frames 15-31).
    warp.intangible_from = 15;
    warp.intangible = 32;
    // Touching down before the reappearance carries the warp on along the ground.
    warp.lands_from = 32;
    warp.ledge_front_from = 32;
    warp.ledge_back_from = 32;
    warp.helpless_after = true;
    warp.fall_landing_lag = 22;
    warp.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/staff_warp.script"),
    ));
    w.moves[MoveId::UpSpecial as usize] = warp;
}

fn jabs(w: &mut Weapon) {
    // Jab 1: a staff jab on frames 8-9, 3%, hitlag x1.6, +4 hitstun, every box angle type 3 (forward). The near and middle boxes use
    // the 361 angle (BK 25, KS 20); the far one pulls a fighter in (180 degrees, BK 20, KS 20). FAF 25. No jab 2: on frames 10-24 it goes
    // on into the rapid jab once attack has been pressed and let go twice since jab 1 began, or on frame 13 with attack held when jab 1
    // has hit (a script). (The reference's fourth box is the far box for objects; no objects here.)
    let mut jab1 = ref_move(
        25,
        0,
        0,
        255,
        &[
            ru(8, 9, 50, 85, 22, 30, 361, 25, 20, 0, 0),
            ru(8, 9, 80, 85, 22, 30, 361, 25, 20, 1, 0),
            ru(8, 9, 120, 85, 25, 30, 180, 20, 20, 2, 0),
        ],
    );
    set_rows(&mut jab1, &[0, 1, 2], |h| {
        h.hitlag = 160;
        h.hitstun_add = 4;
        h.angle_type = ANGLE_FORWARD;
    });
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/staff_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;
    w.moves[MoveId::Jab2 as usize] = Move::empty();
    w.moves[MoveId::Jab3 as usize] = Move::empty();

    // The rapid jab: hits on its frames 4, 7, ... 22 (seven a cycle), each 0.6% at the 361 angle (BK 10, KS 20), angle type 3, hitlag
    // x0.5, SDI x0.4, shieldstun x7; the reference's capsule (8.0 to 13.0 of its units ahead, 8.5 up) as three circles. The cycle loops
    // from its frame 24 back to frame 2, so the first cycle (ext10) runs frames 1-23 and the later ones (ext11) frames 2-23 [D: the loop
    // points 1-2 and 23-24, read as the Gauntlets' were]. After one full cycle, letting go of attack ends it in the finisher (ext12) [E:
    // "let go" is not held and not pressed for 8 frames, as for the Gauntlets].
    let rapid_rows = |frames: &[u8]| -> Vec<R> {
        let mut rows = Vec::new();
        for &f in frames {
            for x in [80, 105, 130] {
                rows.push(ru(f, f, x, 85, 60, 6, 361, 10, 20, 0, 0));
            }
        }
        rows
    };
    let rapid_hit = |mv: &mut Move| {
        for h in &mut mv.hitboxes {
            h.hitlag = 50;
            h.sdi = 40;
            h.shieldstun = 700;
            h.angle_type = ANGLE_FORWARD;
        }
    };
    let mut rapid = ref_move(26, 0, 0, 255, &rapid_rows(&[4, 7, 10, 13, 16, 19, 22]));
    rapid_hit(&mut rapid);
    // (Reference frame 4 is move frame 3; the hits land again every 3 frames.)
    rapid.rehit = Some((3, 3));
    rapid.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/staff_rapid_jab.script"),
    ));
    w.moves[RAPID as usize] = rapid;
    // The later cycles: move frame k is the cycle's frame k + 2, so the hits are on move frames 2, 5, ... 20 (rows given as frames 3,
    // 6, ... 21 in the reference numbering of this move) and the loop point is move frame 22.
    let mut rapid_loop = ref_move(25, 0, 0, 255, &rapid_rows(&[3, 6, 9, 12, 15, 18, 21]));
    rapid_hit(&mut rapid_loop);
    rapid_loop.rehit = Some((2, 3));
    rapid_loop.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/staff_rapid_loop.script"),
    ));
    w.moves[RAPID_LOOP as usize] = rapid_loop;
    // The finisher: a burst on its frames 3-4, 3.5%, angle 361 (type 3), BK 55, KS 140, hitlag x2, radius 6.0 of the reference's units
    // at 9.0 up, 10.0 and 16.0 ahead. FAF 40.
    let mut finisher = ref_move(
        40,
        0,
        0,
        255,
        &[
            ru(3, 4, 100, 90, 60, 35, 361, 55, 140, 0, 0),
            ru(3, 4, 160, 90, 60, 35, 361, 55, 140, 1, 0),
        ],
    );
    set_rows(&mut finisher, &[0, 1], |h| {
        h.hitlag = 200;
        h.angle_type = ANGLE_FORWARD;
    });
    w.moves[RAPID_END as usize] = finisher;
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: the staff spun in front. Hit 1 on frames 14-23, in three phases of set knockback with set weight, 6%, the staff's
    // head (radius 4.1) and tail (3.0): early 14-16 at 90 / 45 degrees (FKV 25 / 45), mid 17-20 at 285 / 100 (FKV 25 / 40), late 21-23
    // at 20 / 85 (FKV 35 / 30); KS 100, hitlag x0.8, SDI x0, shield damage +1 (117 percent [D]). Hit 2 on frames 24-35: 7%, 40 degrees,
    // BK 78, KS 80, the head (4.8) and tail (3.9), shield damage +1 (114 [D]). Angle type 3 throughout. FAF 60. The staff arm is
    // intangible on frames 14-35 (recorded; with a held staff the arm is inside the body's circles [D]). The staff's head and tail are
    // 5.5 of the reference's units either side of its middle, in front at chest height [E].
    let (head, tail, y) = (u(55) + 12, 12 - u(55), 12);
    let mut ftilt = ref_move(60, 0, 0, 255, &[]);
    for (first, last, head_angle, head_fkv, tail_angle, tail_fkv) in [
        (14u8, 16u8, 90i16, 25u16, 45i16, 45u16),
        (17, 20, 285, 25, 100, 40),
        (21, 23, 20, 35, 85, 30),
    ] {
        let mut rows = ref_move(
            60,
            0,
            0,
            255,
            &[
                r(first, last, tail, y, u(30), 60, tail_angle, 0, 100, 0, 0),
                r(first, last, head, y, u(41), 60, head_angle, 0, 100, 1, 0),
            ],
        );
        rows.hitboxes[0].fixed_kb = tail_fkv;
        rows.hitboxes[1].fixed_kb = head_fkv;
        set_rows(&mut rows, &[0, 1], |h| {
            h.set_weight = true;
            h.hitlag = 80;
            h.sdi = 0;
            h.shield_damage = 117;
        });
        ftilt.hitboxes.extend(rows.hitboxes);
    }
    let mut hit2 = ref_move(
        60,
        0,
        0,
        255,
        &[
            r(24, 35, tail, y, u(39), 70, 40, 78, 80, 0, 1),
            r(24, 35, head, y, u(48), 70, 40, 78, 80, 1, 1),
        ],
    );
    set_rows(&mut hit2, &[0, 1], |h| h.shield_damage = 114);
    ftilt.hitboxes.extend(hit2.hitboxes);
    for h in &mut ftilt.hitboxes {
        h.angle_type = ANGLE_FORWARD;
    }
    ftilt.part_intangible = vec![PartIntangible {
        start: 13,
        end: 34,
        parts: PART_LIMB,
        mode: MODE_ANY,
    }];
    // It clanks but cannot rebound (the reference's `rebound=f` without `clang=f` on every box, ssbwiki.com raw table; E3g, sim 86).
    for h in &mut ftilt.hitboxes {
        h.rebounds = false;
    }
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: the staff spun overhead. Five loop hits on frames 8-22 (re-armed every 3: hits on 8, 11, 14, 17, 20), 1.4% each, set
    // knockback (BK 0, KS 100): the top box at the autolink angle (FKV 20), the lower sweep at 130 degrees (FKV 50) and the upper one at
    // 190 (FKV 30); hitlag x0.5, SDI x0.3, shield damage +1 (171 [D]). The last hit on frames 24-25 (its own group): 4.5%, 85 degrees,
    // BK 75, KS 136, hitlag x1.3, shield damage +1 (122 [D]). Angle type 0 (by position). Every box on the root, converted [D]: the
    // sweeps run from 10.5 of the reference's units ahead to 5.0 behind. FAF 56.
    let mut rows = Vec::new();
    rows.push(ru(8, 22, 40, 180, 30, 14, ANGLE_AUTOLINK, 0, 100, 0, 0));
    for x in along(105, -50, 50) {
        rows.push(ru(8, 22, x, 150, 20, 14, 130, 0, 100, 1, 0));
    }
    for x in along(105, -50, 50) {
        rows.push(ru(8, 22, x, 200, 20, 14, 190, 0, 100, 2, 0));
    }
    let loops = rows.len();
    for x in along(105, -50, 50) {
        rows.push(ru(24, 25, x, 230, 30, 45, 85, 75, 136, 0, 1));
    }
    for x in along(105, -50, 50) {
        rows.push(ru(24, 25, x, 170, 30, 45, 85, 75, 136, 1, 1));
    }
    let mut utilt = ref_move(56, 0, 0, 255, &rows);
    let fkv = [20u16, 50, 30];
    for h in &mut utilt.hitboxes[..loops] {
        h.fixed_kb = fkv[usize::from(h.priority)];
        h.hitlag = 50;
        h.sdi = 30;
        h.shield_damage = 171;
    }
    for h in &mut utilt.hitboxes[loops..] {
        h.hitlag = 130;
        h.shield_damage = 122;
    }
    for h in &mut utilt.hitboxes {
        h.angle_type = ANGLE_BY_POSITION;
    }
    // (Reference frame 8 is move frame 7.)
    utilt.rehit = Some((7, 3));
    utilt.rehit_groups = 0b1;
    // It clanks but cannot rebound (the reference's `rebound=f` without `clang=f` on every box, ssbwiki.com raw table; E3g, sim 86).
    for h in &mut utilt.hitboxes {
        h.rebounds = false;
    }
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: the staff swept along the floor. Early on frame 14, late 15-27: on grounded targets 8.5%, 35 degrees, BK 40, KS 100,
    // shield damage +1 (112 [D]); on aerial ones 5%, 75 degrees, BK 38, KS 100 (120 [D]); radius 2.7 along the staff (low in front
    // [E]); from frame 15 also the staff along the floor (19.5 to 9.8 of the reference's units ahead, 1.0 to 2.0 up, radius 2.5 [D]).
    // Angle type 0. FAF 40.
    let mut rows = Vec::new();
    for x in [6, 11, 16, 21] {
        rows.push(r(14, 27, x, 3, u(27), 85, 35, 40, 100, 0, 0));
        rows.push(r(14, 27, x, 3, u(27), 50, 75, 38, 100, 1, 0));
    }
    for x in along(195, 98, 40) {
        rows.push(ru(15, 27, x, 15, 25, 85, 35, 40, 100, 2, 0));
        rows.push(ru(15, 27, x, 15, 25, 50, 75, 38, 100, 3, 0));
    }
    let mut dtilt = ref_move(40, 0, 0, 255, &rows);
    for h in &mut dtilt.hitboxes {
        let grounded = h.damage == Fx::from_ratio(85, 10);
        h.targets = if grounded {
            TARGETS_GROUNDED
        } else {
            TARGETS_AERIAL
        };
        h.shield_damage = if grounded { 112 } else { 120 };
        h.angle_type = ANGLE_BY_POSITION;
    }
    // It clanks but cannot rebound (the reference's `rebound=f` without `clang=f` on every box, ssbwiki.com raw table; E3g, sim 86).
    for h in &mut dtilt.hitboxes {
        h.rebounds = false;
    }
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a lunge behind the shield. Clean on frames 6-9: 11%, 70 / 60 degrees, BK 71, KS 76 (the shield, radius 4.5, at 9.0
    // to 11.0 of the reference's units up and 13.0 ahead; the arm, 3.5, 10.0 up and 5.0 to 9.0 ahead), shield damage +3 (127 [D]); late
    // 10-17: 7%, 60 degrees, BK 61, KS 56 (radius 2.5, 9.0 to 12.0 up, 10.0 ahead), shield damage +3 (143 [D]). Angle type 3. FAF 41
    // (11.0.0). The head and torso are intangible on frames 5-6 and the shield arm on 5-16 (recorded; the arm is inside the body's
    // circles with a held weapon [D]).
    let mut dash = ref_move(
        41,
        0,
        0,
        255,
        &[
            ru(6, 9, 130, 90, 45, 110, 70, 71, 76, 0, 0),
            ru(6, 9, 130, 110, 45, 110, 70, 71, 76, 0, 0),
            ru(6, 9, 50, 100, 35, 110, 60, 71, 76, 1, 0),
            ru(6, 9, 90, 100, 35, 110, 60, 71, 76, 1, 0),
            ru(10, 17, 100, 90, 25, 70, 60, 61, 56, 0, 0),
            ru(10, 17, 100, 120, 25, 70, 60, 61, 56, 0, 0),
        ],
    );
    set_rows(&mut dash, &[0, 1, 2, 3], |h| h.shield_damage = 127);
    set_rows(&mut dash, &[4, 5], |h| h.shield_damage = 143);
    for h in &mut dash.hitboxes {
        h.angle_type = ANGLE_FORWARD;
    }
    dash.part_intangible = vec![
        PartIntangible {
            start: 4,
            end: 5,
            parts: PART_HIGH | PART_MIDDLE | PART_LIMB,
            mode: MODE_ANY,
        },
        PartIntangible {
            start: 4,
            end: 15,
            parts: PART_LIMB,
            mode: MODE_ANY,
        },
    ];
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash: wings of light in front, frames 18-20: the inner wing 16% (KS 100, shield damage +1: 106 [D]) wins where both reach,
    // the wing tips 13% (KS 102, 108 [D]); angle 361, BK 40, radius 5.1. Then the wind: early on 21-25 (a push, set knockback 60, radius
    // 12, 8.0 to 25.0 of the reference's units ahead), late on 26-31 (set knockback 35, out to 40.0), re-applied every frame. Angle type 3
    // throughout. Charges on frame 10. FAF 64. Every box on the root, converted [D].
    let mut rows = vec![
        ru(18, 20, 60, 110, 51, 160, 361, 40, 100, 0, 0),
        ru(18, 20, 119, 99, 51, 160, 361, 40, 100, 0, 0),
        ru(18, 20, 125, 98, 51, 130, 361, 40, 102, 1, 0),
        ru(18, 20, 185, 85, 51, 130, 361, 40, 102, 1, 0),
    ];
    let hits = rows.len();
    for x in along(80, 250, 85) {
        rows.push(ru(21, 25, x, 100, 120, 0, 361, 0, 100, 0, 1));
    }
    let early = rows.len();
    for x in along(80, 400, 80) {
        rows.push(ru(26, 31, x, 100, 120, 0, 361, 0, 100, 0, 1));
    }
    let mut fsmash = ref_move(64, 0, 0, 255, &rows);
    set_rows(&mut fsmash, &[0, 1], |h| h.shield_damage = 106);
    set_rows(&mut fsmash, &[2, 3], |h| h.shield_damage = 108);
    for (i, h) in fsmash.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_FORWARD;
        if i >= hits {
            h.push = true;
            h.fixed_kb = if i < early { 60 } else { 35 };
        }
    }
    // (The wind, group 1, pushes again every frame from frame 21; the hit, group 0, lands once.)
    fsmash.rehit = Some((20, 1));
    fsmash.rehit_groups = 0b10;
    fsmash.charge_at = Some(9);
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: a pillar of light in front (9.7 of the reference's units ahead), three columns from the floor up. Clean on frames 18-29:
    // up to 21.0 units 16% (radius 4.5, BK 53, KS 84), up to 44.0 12% (2.8, BK 58, KS 89), up to 65.0 9% (2.8, BK 62, KS 92); late on
    // 30-35: 13% / 9% / 7%, every column radius 2.8. 88 degrees, angle type 3; the lowest column wins where several reach. Charges on
    // frame 12. FAF 64.
    let mut rows = Vec::new();
    for (first, last, tops, dmgs, radii) in [
        (18u8, 29u8, [210, 440, 650], [160, 120, 90], [45, 28, 28]),
        (30, 35, [220, 440, 650], [130, 90, 70], [28, 28, 28]),
    ] {
        let knockback = [(53i16, 84i16), (58, 89), (62, 92)];
        let mut from = if first == 18 { 40 } else { 20 };
        for (id, (((top, damage), radius), (bk, ks))) in tops
            .into_iter()
            .zip(dmgs)
            .zip(radii)
            .zip(knockback)
            .enumerate()
        {
            for y in along(from, top, 40) {
                rows.push(ru(
                    first, last, 97, y, radius, damage, 88, bk, ks, id as u8, 0,
                ));
            }
            from = top;
        }
    }
    let mut usmash = ref_move(64, 0, 0, 255, &rows);
    for h in &mut usmash.hitboxes {
        h.angle_type = ANGLE_FORWARD;
    }
    usmash.charge_at = Some(11);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a burst on both sides. Clean on frame 17: 15%, 60 degrees, BK 52 (7.0.0), KS 103, hitlag x1.4, radius 3.7, 9.2 of the
    // reference's units up, 7.3 either side; late on 18-19: 13%, 28 degrees, BK 30, KS 90, hitlag x0.85, radius 6.0, 4.0 up, out to 14.0
    // either side; then the wind on 20-29: a push (set knockback 55, angle 361, radius 9.0) out to 24.0 either side, re-applied every
    // frame. Angle type 0 (by position): each side sends its own way. Charges on frame 13. FAF 69.
    let mut rows = Vec::new();
    for x in along(73, -73, 73) {
        rows.push(ru(17, 17, x, 92, 37, 150, 60, 52, 103, 0, 0));
    }
    let clean = rows.len();
    for x in along(140, -140, 70) {
        rows.push(ru(18, 19, x, 40, 60, 130, 28, 30, 90, 0, 0));
    }
    let late = rows.len();
    for x in along(240, -240, 80) {
        rows.push(ru(20, 29, x, 40, 90, 0, 361, 0, 100, 0, 1));
    }
    let mut dsmash = ref_move(69, 0, 0, 255, &rows);
    for (i, h) in dsmash.hitboxes.iter_mut().enumerate() {
        h.angle_type = ANGLE_BY_POSITION;
        if i < clean {
            h.hitlag = 140;
        } else if i < late {
            h.hitlag = 85;
        } else {
            h.push = true;
            h.fixed_kb = 55;
        }
    }
    dsmash.rehit = Some((19, 1));
    dsmash.rehit_groups = 0b10;
    dsmash.charge_at = Some(12);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

fn aerials(w: &mut Weapon) {
    // Neutral air: the staff spun round the body. Six loop hits on frames 5-27 (re-armed every 4: 5, 9, 13, 17, 21, 25), 1.4%, hitlag
    // x0.5, SDI x0.3, shield damage +1 (171 [D]), radius 4.2 on the staff's two ends: on aerial targets the autolink angle (BK 0, KS 100);
    // on grounded ones set knockback 100 degrees (FKV 65), the stronger frames 5-6 at 95 (FKV 80) on the tail. On 24-27 the aerial boxes
    // become set knockback (FKV 65, SDI 0) and every loop box takes 13 frames of hitstun off (7.0.0; the reference's engine applies it to
    // the grounded boxes too). The last hit on frames 29-30: 5.1%, 55 degrees, BK 35, KS 155, hitlag x1.5, shield damage +3 (159 [D]),
    // radius 9.8 on aerial targets, 11.5 on grounded ones, always sent the way the fighter faces (angle type 3); the loop is angle type 0.
    // Landing lag 12, autocancels on frames 1-3 and from 40, FAF 52. The staff's ends 5.2 / 5.4 of the reference's units either side
    // of the chest [E].
    let (front, back, y) = (u(52), -u(54), 12);
    let mut nair = ref_move(
        52,
        12,
        3,
        40,
        &[
            // Early (5-6): aerial 0 / 1, grounded 2 / 3.
            r(5, 6, front, y, u(42), 14, ANGLE_AUTOLINK, 0, 100, 0, 0),
            r(5, 6, back, y, u(42), 14, ANGLE_AUTOLINK, 0, 100, 1, 0),
            r(5, 6, front, y, u(42), 14, 100, 0, 100, 2, 0),
            r(5, 6, back, y, u(42), 14, 95, 0, 100, 3, 0),
            // Mid (7-23).
            r(7, 23, front, y, u(42), 14, ANGLE_AUTOLINK, 0, 100, 0, 0),
            r(7, 23, back, y, u(42), 14, ANGLE_AUTOLINK, 0, 100, 1, 0),
            r(7, 23, front, y, u(42), 14, 100, 0, 100, 2, 0),
            r(7, 23, back, y, u(42), 14, 100, 0, 100, 3, 0),
            // Late (24-27).
            r(24, 27, front, y, u(42), 14, ANGLE_AUTOLINK, 0, 100, 0, 0),
            r(24, 27, back, y, u(42), 14, ANGLE_AUTOLINK, 0, 100, 1, 0),
            r(24, 27, front, y, u(42), 14, 100, 0, 100, 2, 0),
            r(24, 27, back, y, u(42), 14, 100, 0, 100, 3, 0),
            // The last hit.
            r(29, 30, 0, y, u(98), 51, 55, 35, 155, 0, 1),
            r(29, 30, 0, y, u(115), 51, 55, 35, 155, 1, 1),
        ],
    );
    for (i, h) in nair.hitboxes.iter_mut().enumerate() {
        if i < 12 {
            let aerial = h.priority < 2;
            h.targets = if aerial {
                TARGETS_AERIAL
            } else {
                TARGETS_GROUNDED
            };
            h.hitlag = 50;
            h.sdi = 30;
            h.shield_damage = 171;
            h.angle_type = ANGLE_BY_POSITION;
            if !aerial {
                h.fixed_kb = if i == 3 { 80 } else { 65 };
            }
            if i >= 8 {
                h.hitstun_add = -13;
                if aerial {
                    h.fixed_kb = 65;
                    h.sdi = 0;
                }
            }
        } else {
            h.targets = if i == 12 {
                TARGETS_AERIAL
            } else {
                TARGETS_GROUNDED
            };
            h.hitlag = 150;
            h.shield_damage = 159;
            h.angle_type = ANGLE_FORWARD;
        }
    }
    // (Reference frame 5 is move frame 4.)
    nair.rehit = Some((4, 4));
    nair.rehit_groups = 0b1;
    // It clanks but cannot rebound (the reference's `rebound=f` without `clang=f` on every box, ssbwiki.com raw table; E3g, sim 86).
    for h in &mut nair.hitboxes {
        h.rebounds = false;
    }
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: a pulse from the palm in front on frames 9-11, 10%, 45 degrees (type 3), BK 31, KS 92, radius 5.5, 6.9 of the
    // reference's units up and 9.0 to 15.0 ahead, shield damage +3 (130 [D]). Landing lag 9 (11.0.0), autocancels on frames 1-3 and
    // from 24, FAF 45.
    let mut fair = ref_move(
        45,
        9,
        3,
        24,
        &along(90, 150, 30)
            .into_iter()
            .map(|x| ru(9, 11, x, 69, 55, 100, 45, 31, 92, 0, 0))
            .collect::<Vec<_>>(),
    );
    for h in &mut fair.hitboxes {
        h.shield_damage = 130;
        h.angle_type = ANGLE_FORWARD;
    }
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: the shield thrust behind on frames 8-10, 12%, angle 361 sent backward (type 4), BK 30, KS 95, hitlag x1.2, radius 7.0,
    // 10.2 of the reference's units up and 14.0 behind, shield damage +3 (125 [D]). The head and torso are intangible on frame 7 and the
    // shield arm on 7-10 (recorded; the arm is inside the body's circles with a held weapon [D]). Landing lag 10, autocancels on frames
    // 1-2 and from 35, FAF 50.
    let mut bair = ref_move(
        50,
        10,
        2,
        35,
        &[ru(8, 10, -140, 102, 70, 120, 361, 30, 95, 0, 0)],
    );
    bair.hitboxes[0].hitlag = 120;
    bair.hitboxes[0].shield_damage = 125;
    bair.hitboxes[0].angle_type = ANGLE_BACKWARD;
    bair.part_intangible = vec![
        PartIntangible {
            start: 6,
            end: 6,
            parts: PART_HIGH | PART_MIDDLE | PART_LIMB,
            mode: MODE_ANY,
        },
        PartIntangible {
            start: 6,
            end: 9,
            parts: PART_LIMB,
            mode: MODE_ANY,
        },
    ];
    w.moves[MoveId::BAir as usize] = bair;

    // Up air: the staff spun overhead. Five loop hits on frames 10-22 (re-armed every 3: 10, 13, 16, 19, 22), 1%, set knockback (BK 0,
    // KS 100), hitlag x0.6, SDI x0.7: on grounded targets 105 degrees (FKV 40, radius 4.0, 20.0 of the reference's units up, 6.0 either
    // side); on aerial ones the autolink angle (FKV 20: the top, radius 4.0 at 22.8 up, and the two sides, 3.0, from 18.0 up and 4.0 out
    // to 22.0 up and 9.0 out). The last hit on frames 24-25: 5%, 88 degrees, BK 50, KS 152, hitlag x1.5, SDI x0.7 (the top, radius 6.0 at
    // 23.0 up; the sides, 4.0, from 18.0 up and 4.0 out to 23.0 up and 12.0 out). Every box angle type 4 (backward), the ones in front
    // too. Landing lag 14, autocancels on frames 1-2 and from 42, FAF 52. Every box on the root, converted [D].
    let mut rows = Vec::new();
    for x in along(-60, 60, 60) {
        rows.push(ru(10, 22, x, 200, 40, 10, 105, 0, 100, 0, 0));
    }
    rows.push(ru(10, 22, 0, 228, 40, 10, ANGLE_AUTOLINK, 0, 100, 1, 0));
    for side in [1, -1] {
        rows.push(ru(
            10,
            22,
            40 * side,
            180,
            30,
            10,
            ANGLE_AUTOLINK,
            0,
            100,
            2,
            0,
        ));
        rows.push(ru(
            10,
            22,
            90 * side,
            220,
            30,
            10,
            ANGLE_AUTOLINK,
            0,
            100,
            2,
            0,
        ));
    }
    let loops = rows.len();
    rows.push(ru(24, 25, 0, 230, 60, 50, 88, 50, 152, 0, 1));
    for side in [1, -1] {
        rows.push(ru(24, 25, 40 * side, 180, 40, 50, 88, 50, 152, 1, 1));
        rows.push(ru(24, 25, 120 * side, 230, 40, 50, 88, 50, 152, 1, 1));
    }
    let mut uair = ref_move(52, 14, 2, 42, &rows);
    for (i, h) in uair.hitboxes.iter_mut().enumerate() {
        h.sdi = 70;
        h.angle_type = ANGLE_BACKWARD;
        if i < loops {
            h.hitlag = 60;
            if h.priority == 0 {
                h.targets = TARGETS_GROUNDED;
                h.fixed_kb = 40;
            } else {
                h.targets = TARGETS_AERIAL;
                h.fixed_kb = 20;
            }
        } else {
            h.hitlag = 150;
        }
    }
    // (Reference frame 10 is move frame 9.)
    uair.rehit = Some((9, 3));
    uair.rehit_groups = 0b1;
    w.moves[MoveId::UAir as usize] = uair;

    // Down air: the staff driven down on frames 10-11, 11%, hitlag x1.5: on aerial targets a meteor (270 degrees, BK 20, KS 100, radius
    // 4.2, 3.7 to 5.0 of the reference's units below the feet); on grounded ones 55 degrees sent backward (type 4, BK 30, KS 100, radius
    // 6.3, 5.0 below, 1.5 ahead to 0.5 behind). Landing lag 11, autocancels on frames 1-2 and from 48, FAF 59.
    let mut dair = ref_move(
        59,
        11,
        2,
        48,
        &[
            ru(10, 11, 5, -37, 42, 110, 270, 20, 100, 0, 0),
            ru(10, 11, 5, -50, 42, 110, 270, 20, 100, 0, 0),
            ru(10, 11, 15, -50, 63, 110, 55, 30, 100, 1, 0),
            ru(10, 11, -5, -50, 63, 110, 55, 30, 100, 1, 0),
        ],
    );
    for (i, h) in dair.hitboxes.iter_mut().enumerate() {
        h.hitlag = 150;
        if i < 2 {
            h.targets = TARGETS_AERIAL;
            h.angle_type = ANGLE_BY_POSITION;
        } else {
            h.targets = TARGETS_GROUNDED;
            h.angle_type = ANGLE_BACKWARD;
        }
    }
    w.moves[MoveId::DAir as usize] = dair;
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: standing on frames 7-8 (FAF 40), dash 10-11 (FAF 48), pivot 11-12 (FAF 43). Each reach is the reference's grounded-target
    // grab box (a swept circle at 8.0 of its units up; the standing one shortened to 9.4 in 11.0.0) as circles along it [D]. (Its
    // second, thinner box for aerial targets is not modelled: grabs catch only fighters on the ground here.)
    let grab_line = |first: u8, last: u8, radius: i32, x0: i32, x1: i32| -> Vec<R> {
        along(x0, x1, 40)
            .into_iter()
            .map(|x| ru(first, last, x, 80, radius, 0, 361, 0, 0, 0, 0))
            .collect()
    };
    w.moves[MoveId::Grab as usize] = with_kind(
        ref_move(40, 0, 0, 255, &grab_line(7, 8, 36, 40, 94)),
        HIT_GRAB,
    );
    w.moves[MoveId::DashGrab as usize] = with_kind(
        ref_move(48, 0, 0, 255, &grab_line(10, 11, 29, 40, 131)),
        HIT_GRAB,
    );
    w.moves[MoveId::PivotGrab as usize] = with_kind(
        ref_move(43, 0, 0, 255, &grab_line(11, 12, 36, 40, 189)),
        HIT_GRAB,
    );
    // Pummel: 1.3%, set knockback 30 (KS 100), hitlag x2.1, radius 5.0 at 9.0 of the reference's units up and 10.0 ahead [D]. The
    // reference hits on frame 1 with FAF 7; ours on frame 2 with FAF 8 (a pummel cannot hit on its first frame) [D].
    w.moves[MoveId::Pummel as usize] = reference_pummel(13, u(100), u(90), u(50));
    // Throws, each a single release (no hit on the held fighter first). Forward: on frame 20, 9%, 45 degrees, BK 90, KS 30, FAF 45.
    // Back: on 20, 10%, angle 361 behind, BK 65, KS 70, the thrower ends facing the way it threw, FAF 55. Up: on 17, 8%, 92 degrees,
    // BK 90, KS 60, FAF 55. Down: 5%, 73 degrees, BK 90, KS 30 (7.0.0); the reference freezes both fighters for 8 frames of hitlag on
    // its frame 24 and releases on 25 with FAF 46 counted without them, so here (with nobody frozen) the release is on frame 33 and the
    // FAF 54, the same timeline [D].
    let throw = |faf: u8, row: R| with_kind(ref_move(faf, 0, 0, 255, &[row]), HIT_THROW);
    w.moves[MoveId::FThrow as usize] = throw(45, r(20, 20, 13, 11, 10, 90, 45, 90, 30, 0, 0));
    let mut bthrow = throw(55, r(20, 20, -13, 11, 10, 100, 361, 65, 70, 0, 0));
    bthrow.turns_around = true;
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = throw(55, r(17, 17, 13, 11, 10, 80, 92, 90, 60, 0, 0));
    w.moves[MoveId::DThrow as usize] = throw(54, r(33, 33, 13, 11, 10, 50, 73, 90, 30, 0, 0));
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: a staff sweep climbing up, frames 23-25, 9%, 45 degrees (type 3: forward on both sides), BK 90, KS 20, shield damage
    // +1 (111 [D]); intangible through frame 25, FAF 56. Its swept box (9.2 of the reference's units ahead to 5.2 behind, 4.5 up, radius
    // 5.0) as circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along(92, -52, 48)
            .into_iter()
            .map(|x| ru(23, 25, x, 45, 50, 90, 45, 90, 20, 0, 0))
            .collect::<Vec<_>>(),
    );
    for h in &mut ledge.hitboxes {
        h.shield_damage = 111;
        h.angle_type = ANGLE_FORWARD;
    }
    ledge.intangible = 25;
    w.moves[MoveId::LedgeAttack as usize] = ledge;
    // Get-up attack (lying face up): behind on frames 16-17 (15.0 to 4.0 of the reference's units behind), then in front on 25-26 (4.0
    // to 15.0 ahead), 5.0 up, radius 5.0; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8 (214 [D]); intangible through frame
    // 26, FAF 46. (The face-down and trip get-up attacks have no slot of their own.)
    let mut rows = Vec::new();
    for x in along(-150, -40, 55) {
        rows.push(ru(16, 17, x, 50, 50, 70, 48, 80, 48, 0, 0));
    }
    for x in along(150, 40, 55) {
        rows.push(ru(25, 26, x, 50, 50, 70, 48, 80, 48, 0, 1));
    }
    let mut getup = ref_move(46, 0, 0, 255, &rows);
    for h in &mut getup.hitboxes {
        h.shield_damage = 214;
        h.angle_type = ANGLE_BY_POSITION;
    }
    getup.intangible = 26;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}
