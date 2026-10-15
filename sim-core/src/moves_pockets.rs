//! The Pockets moveset (the settler body), modelled mechanically on a reference character: phase 1, the body's normals and two
//! specials (the balloon flight and the rocket). A child module of `moves` (it uses that file's private row builders). Data and
//! sources: the eleventh moveset's `REFERENCE_DATA.md` (`docs/movesets/`), read from ssbwiki.com's raw hitbox tables
//! (13.0.1); the jab is the previous reference version's (1.1.x, owner decision 66) on that version's hit rules (E40, decision 67).

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

/// A reference speed or length (thousandths of its units) in world units: / 8 [D].
fn rs(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

/// A projectile's hit from a reference row (radius and damage in tenths; the frames do not matter on a projectile).
fn shot_hit(radius: i32, damage: i32, angle: i16, bkb: i16, kbg: i16, angle_type: u8) -> Hitbox {
    let mut h = row(&ru(1, 1, 0, 0, radius, damage, angle, bkb, kbg, 0, 0));
    h.angle_type = angle_type;
    h
}

/// The up air with two turnips, and with one (the 3-turnip version is the up air's own slot): E39 random versions.
pub const UAIR_TWO: MoveId = MoveId::Ext0;
pub const UAIR_ONE: MoveId = MoveId::Ext1;
/// The down air with two turnips, and with one.
pub const DAIR_TWO: MoveId = MoveId::Ext2;
pub const DAIR_ONE: MoveId = MoveId::Ext3;
/// The rocket (side special) ridden, and its three endings: a hit (the bounce), a wall (the crash) and jumping off (the hop).
pub const RIDE: MoveId = MoveId::Ext4;
pub const RIDE_BOUNCE: MoveId = MoveId::Ext5;
pub const RIDE_CRASH: MoveId = MoveId::Ext6;
pub const RIDE_HOP: MoveId = MoveId::Ext7;
/// The balloon flight's next stretch (the up special goes on as it every 248 frames, until the helium runs out).
pub const FLIGHT: MoveId = MoveId::Ext8;
/// The rocket's tool kind (one out at a time, E3d): the `owned` register's number for it.
pub const ROCKET_KIND: u8 = 4;

/// The Pockets moveset (the settler body): boxing gloves, an umbrella, a stick, a weed, a flowerpot, a bowling ball, a firework, a
/// shovel, a slingshot, turnips, a rocket and balloons. Every normal follows the reference frame for frame (the eleventh moveset's
/// `REFERENCE_DATA.md` in `docs/movesets/`, version 13.0.1, read from ssbwiki.com's raw hitbox tables): active frames, FAF,
/// landing lag and autocancel windows, smash charge frames and the forward smash's own 1.2x charge; damage, angles and angle types,
/// base and growth knockback, set knockback and set weight, hitlag and SDI multipliers, shield damage, trip chances, grounded- and
/// aerial-only boxes, hit groups, the hitstun modifier, bury, the up tilt's head intangibility, the random turnip counts (E39). Its
/// projectiles (the flowerpot, the bowling ball, the slingshot's pellets) are projectiles here too. **The jab is the previous
/// reference version's (1.1.x)** on that version's hit rules (owner decisions 66-67, E40). Radii and offsets on the reference's root
/// bone (`top`) are its values converted (x 1.25, rounded half up to a tenth) [D]; boxes on other bones (hands, held tools, hips) are
/// placed on our body where that bone is in the move's pose [E]. The neutral and down specials are still the claws' (phase 2).
pub fn pockets() -> Weapon {
    let mut w = claws();
    // Held tools (an umbrella, a stick, a shovel...): no limb circle, as the other held-weapon movesets.
    w.limbs = false;
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
    rocket(&mut w);
    balloons(&mut w);
    pocket(&mut w);
    tree(&mut w);
    w
}

/// The previous reference version's (1.1.x) jab hit (owner decision 66; ssbwiki.com's previous-version jab pages, hits 1 and 2,
/// kuroganehammer.com's 1.1.6 table): 3%, BK `bkb`, KS `kbg`, the near box (the reference's id 0, the glove's -1.2 along the
/// hand bone) at 60 degrees with radius 3.0 and the far one (id 1, 4.2 along it) at 80 degrees with radius 3.6, hitlag and SDI x1, on
/// that version's hit rules (E40) [R]. Where the hand is in the punch is ours: 5.5 of the reference's units up, the near box 6.8 ahead
/// and the far one 12.2 [E] (13.0.1's jab boxes reach 6.5 to 11.0 ahead at 5.5 up).
fn older_jab_rows(bkb: i16, kbg: i16) -> Vec<R> {
    vec![
        ru(3, 4, 68, 55, 30, 30, 60, bkb, kbg, 0, 0),
        ru(3, 4, 122, 55, 36, 30, 80, bkb, kbg, 1, 0),
    ]
}

fn jabs(w: &mut Weapon) {
    // Jab 1 (1.1.x): hits on frames 3-4, earliest continuable frame 7, FAF 22 [R]. A press from frame 7 carries on into jab 2 at once
    // (`next_from`); attack held carries on at the end of the move (the script) [E: the version's held rule is not published].
    // From frame 7 it can also be cancelled into any other action (`cancel_from`: that version's jab cancel by a crouch or a shield
    // in the continuable window, ssbwiki.com `Jab_cancel` [R]; acting on the same frame is [E], owner decision 70).
    // **The pull** (owner decision 70): the far glove box sends toward the settler as 13.0.1's own far jab 1 boxes do (180 degrees,
    // forward type, BK 15, KS 10: ssbwiki.com's raw 13.0.1 jab 1 table, ids 1-2 [R]); the 1.1.x tables (ssbwiki.com
    // and kuroganehammer.com 1.1.6) give that box 80 degrees, BK 10, KS 80, with no pull. The near box keeps the 1.1.x 60 degrees.
    let mut jab1 = ref_move(22, 0, 0, 255, &older_jab_rows(10, 80));
    set_all(&mut jab1, |h| {
        h.rules = RULES_PREVIOUS;
        h.angle_type = ANGLE_BY_POSITION;
    });
    if let Some(far) = jab1.hitboxes.iter_mut().find(|h| h.priority == 1) {
        far.angle = 180;
        far.angle_type = ANGLE_FORWARD;
        far.base_knockback = 15;
        far.knockback_growth = 10;
    }
    chain(&mut jab1, MoveId::Jab2, 7);
    jab1.cancel_from = 7 - 1;
    jab1.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_jab_1.script"),
    ));
    w.moves[MoveId::Jab as usize] = jab1;

    // Jab 2 (1.1.x): the other glove, the same boxes with BK 20, KS 120 (the launcher: up and out); hits on 3-4, earliest continuable
    // frame 8, FAF 24 [R]. It carries on into jab 1 again (the version's endless jab, no finisher [R]): pressed from frame 8, or held
    // at the end [E]; cancellable from frame 8 as jab 1 from 7.
    let mut jab2 = ref_move(24, 0, 0, 255, &older_jab_rows(20, 120));
    set_all(&mut jab2, |h| {
        h.rules = RULES_PREVIOUS;
        h.angle_type = ANGLE_BY_POSITION;
    });
    chain(&mut jab2, MoveId::Jab, 8);
    jab2.cancel_from = 8 - 1;
    jab2.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_jab_2.script"),
    ));
    w.moves[MoveId::Jab2 as usize] = jab2;
    w.moves[MoveId::Jab3 as usize] = Move::empty();
}

fn tilts(w: &mut Weapon) {
    // Forward tilt: an umbrella swung down in front on frames 8-11, 9%, 45 degrees (type 3), BK 80, KS 50 [R]: the canopy (radius 4.5,
    // the reference's id 0), its two edges (2.5, ids 1 and 2) and the shaft (2.0, id 3) [R], the umbrella held out ahead at head
    // height [E]. One hit. FAF 34.
    let mut ftilt = ref_move(
        34,
        0,
        0,
        255,
        &[
            ru(8, 11, 100, 80, 45, 90, 45, 80, 50, 0, 0),
            ru(8, 11, 60, 110, 25, 90, 45, 80, 50, 1, 0),
            ru(8, 11, 130, 50, 25, 90, 45, 80, 50, 2, 0),
            ru(8, 11, 60, 60, 20, 90, 45, 80, 50, 3, 0),
        ],
    );
    set_all(&mut ftilt, |h| h.angle_type = ANGLE_FORWARD);
    w.moves[MoveId::FTilt as usize] = ftilt;

    // Up tilt: a stick swung overhead. Hit 1 on frames 7-18, 5%, set weight, hitlag x0.8: against grounded fighters 108 degrees,
    // BK 0, KS 100, set knockback 75 (the stick's near and far boxes, radius 3.8 and 4.0, ids 0-1); against airborne ones 135 degrees,
    // BK 35, KS 10 (radius 4.0, id 2); +2 frames of hitstun on every box (the reference's modifier on id 2, applied to all by its
    // glitch) [R]. Hit 2 on 22-25: 6%, 80 degrees (type 3), BK 50, KS 140, hitlag x1.2 (radius 5.0 and 4.0) [R]. Angle type 0 for
    // hit 1. The stick over the head [E]. The head (and the arm: nothing here, a held weapon) intangible on 7-25. FAF 48.
    let mut rows = vec![
        ru(7, 18, 20, 130, 38, 50, 108, 0, 100, 0, 0),
        ru(7, 18, 30, 180, 40, 50, 108, 0, 100, 1, 0),
        ru(7, 18, 30, 180, 40, 50, 135, 35, 10, 2, 0),
    ];
    let hit2 = rows.len();
    rows.push(ru(22, 25, 0, 190, 50, 60, 80, 50, 140, 0, 1));
    rows.push(ru(22, 25, 0, 140, 40, 60, 80, 50, 140, 1, 1));
    let mut utilt = ref_move(48, 0, 0, 255, &rows);
    for (i, h) in utilt.hitboxes.iter_mut().enumerate() {
        if i < hit2 {
            h.set_weight = true;
            h.hitlag = 80;
            h.hitstun_add = 2;
            h.angle_type = ANGLE_BY_POSITION;
            if i < 2 {
                h.fixed_kb = 75;
                h.targets = TARGETS_GROUNDED;
            } else {
                h.targets = TARGETS_AERIAL;
            }
        } else {
            h.hitlag = 120;
            h.angle_type = ANGLE_FORWARD;
        }
    }
    utilt.part_intangible = vec![PartIntangible {
        start: 6,
        end: 24,
        parts: PART_HIGH | PART_LIMB,
        mode: MODE_ANY,
    }];
    w.moves[MoveId::UTilt as usize] = utilt;

    // Down tilt: a weed pulled up on frames 9-11, angle type 0, BK 50, KS 95, a 30% trip chance on both boxes: close 12% at 75 degrees
    // (radius 5.0, 3.0 of the reference's units up and ahead), far 10% at 70 (radius 6.0 from 10.0 to 2.0 ahead, 3.0 up, grounded
    // fighters only) [R]. One hit. FAF 38. (At these angles it lifts a grounded fighter, so it trips only on a slope, as the reference.)
    let mut rows = vec![ru(9, 11, 30, 30, 50, 120, 75, 50, 95, 0, 0)];
    rows.extend(along_rows(
        9,
        11,
        (100, 30),
        (20, 30),
        60,
        100,
        70,
        50,
        95,
        1,
        0,
    ));
    let mut dtilt = ref_move(38, 0, 0, 255, &rows);
    set_all(&mut dtilt, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.trip = 30;
        if h.priority == 1 {
            h.targets = TARGETS_GROUNDED;
        }
    });
    w.moves[MoveId::DTilt as usize] = dtilt;

    // Dash attack: a tumble that flings a flowerpot (a projectile here, as the reference's: `direct=f`, reflectable). Out on frame 9:
    // clean through frame 14, 10% (radius 4.2), late from 15, 6% (3.7); 70 degrees (type 3), BK 65, KS 60, hitlag x0.6, SDI x0 [R]. On
    // the ground it breaks on landing after frame 22; off a ledge it falls on, hitting until frame 68 [R]. Its flight is ours: from 10.0
    // of the reference's units ahead at 8.0 up, 1.0 a frame forward, falling to the floor on frame 23 [E]. (A shot made on a move frame
    // first hits on the next: the pot appears on the reference's frame 8, its ages 1-6 clean, from 7 late.) FAF 45.
    let mut dash = ref_move(45, 0, 0, 255, &[]);
    let mut clean = shot_hit(42, 100, 70, 65, 60, ANGLE_FORWARD);
    clean.hitlag = 60;
    clean.sdi = 0;
    let late = Hitbox {
        radius: Fx::from_ratio(u(37), 10),
        damage: Fx::from_int(6),
        ..clean
    };
    dash.projectile = Some(ProjectileSpawn {
        frame: 7,
        x: rs(10000),
        y: rs(8000),
        speed: rs(1000),
        life: 61,
        hitbox: clean,
        end_damage: clean.damage,
        arc: ProjectileArc {
            // (From 1.0 up, touching the floor on its 15th frame: 1.0 / 112, between 1.0 / (15 x 16 / 2) and 1.0 / (14 x 15 / 2).)
            gravity: Fx::from_ratio(1, 112),
            max_fall: rs(4000),
            // A touch of the floor ends it (stuck for a frame with no hit, then gone).
            lodge: 1,
            ..ProjectileArc::NONE
        },
        phases: phase_list(&[(7, late)]),
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
    });
    w.moves[MoveId::DashAttack as usize] = dash;
}

fn smashes(w: &mut Weapon) {
    // Forward smash: a bowling ball dropped in front (a projectile, `direct=f`, reflectable). Charges on frames 16-17; full charge
    // deals 1.2x (the move's own multiplier: `charge_bonus` 20 and, on the ball, `ChargedShot::power_to` 120) [R]. The ball hits from
    // frame 25: early 15% (radius 6.0) through 27, then 17% (3.7); 45 degrees (type 3), BK 70, KS 95, SDI x0 [R]. On the ground it
    // stops hitting after frame 30; off a ledge it falls on, hitting until 139 [R]. Its drop is ours: from 12.0 of the reference's units
    // ahead and 8.8 up, falling 0.8 a frame plus 0.16 a frame each frame, it reaches the floor on frame 31 [E]. (It appears on the
    // reference's frame 24 and first hits on 25: its ages 1-3 early, from 4 the later hit.) FAF 50.
    let mut fsmash = ref_move(50, 0, 0, 255, &[]);
    fsmash.charge_at = Some(15);
    fsmash.charge_bonus = 20;
    let mut early = shot_hit(60, 150, 45, 70, 95, ANGLE_FORWARD);
    early.sdi = 0;
    let mid = Hitbox {
        radius: Fx::from_ratio(u(37), 10),
        damage: Fx::from_int(17),
        ..early
    };
    fsmash.projectile = Some(ProjectileSpawn {
        frame: 23,
        x: rs(12000),
        y: rs(8800),
        speed: rs(800),
        life: 116,
        hitbox: early,
        end_damage: early.damage,
        arc: ProjectileArc {
            angle: -90,
            gravity: rs(160),
            max_fall: rs(3000),
            lodge: 1,
            ..ProjectileArc::NONE
        },
        phases: phase_list(&[(4, mid)]),
        pierce: false,
        rehit: 0,
        burst: None,
        charged: Some(ChargedShot {
            full: 60,
            snap: 0,
            damage_to: early.damage,
            speed_to: rs(800),
            life_to: 116,
            size_from: 100,
            size_to: 100,
            floor: false,
            times: 100,
            power_to: 120,
            every_hit: false,
        }),
        tool: Tool::NONE,
    });
    w.moves[MoveId::FSmash as usize] = fsmash;

    // Up smash: a firework. Charges on frames 7-8. Hit 1 on 12-13 against grounded fighters only: 3%, set knockback 120 (BK 0, KS 100),
    // hitlag x0.5, SDI x0.1, type 3, 87 degrees near the feet (radius 4.0, 0.5 to 2.5 ahead, 5.0 up) and 115 a little ahead (6.0, 6.0
    // ahead) [R]. Hits 2-5 on 17-18, 21-22, 25-26 and 29-30: 1.5% each, set weight, set knockback 30 (KS 0), SDI x0: 110 degrees (BK 55)
    // at 19.0 up and 185 (BK 40) at 25.0 up, radius 2.8, type 0 [R]. Hit 6 on 33-34: 4.6%, 90 degrees, BK 30, KS 220, radius 9.0 at
    // 24.0 up, hitlag x1.5 [R]. Every hit its own group. (The firework's hits are projectiles in the reference, so its user is not frozen
    // by their hitlag: not modelled.) FAF 54.
    let mut rows = vec![
        ru(12, 13, 15, 50, 40, 30, 87, 0, 100, 0, 0),
        ru(12, 13, 60, 50, 60, 30, 115, 0, 100, 1, 0),
    ];
    for (k, frame) in [17u8, 21, 25, 29].into_iter().enumerate() {
        let g = k as u8 + 1;
        rows.push(ru(frame, frame + 1, 0, 190, 28, 15, 110, 55, 0, 0, g));
        rows.push(ru(frame, frame + 1, 0, 250, 28, 15, 185, 40, 0, 1, g));
    }
    rows.push(ru(33, 34, 0, 240, 90, 46, 90, 30, 220, 0, 5));
    let mut usmash = ref_move(54, 0, 0, 255, &rows);
    for (i, h) in usmash.hitboxes.iter_mut().enumerate() {
        match i {
            0 | 1 => {
                h.fixed_kb = 120;
                h.hitlag = 50;
                h.sdi = 10;
                h.targets = TARGETS_GROUNDED;
                h.angle_type = ANGLE_FORWARD;
            }
            10 => {
                h.hitlag = 150;
                h.angle_type = ANGLE_BY_POSITION;
            }
            _ => {
                h.fixed_kb = 30;
                h.set_weight = true;
                h.sdi = 0;
                h.angle_type = ANGLE_BY_POSITION;
            }
        }
    }
    usmash.charge_at = Some(6);
    w.moves[MoveId::USmash as usize] = usmash;

    // Down smash: a shovel dug in front, then behind. Charges on frames 3-4. Hit 1: clean on frame 9, late on 10-11; hit 2 behind, clean
    // on 29, late on 30-31. Clean 6% (radius 4.0, 13.0 of the reference's units ahead / behind, 3.0 up), late 3% (6.0, 12.0) with no
    // hitlag; 60 degrees; against grounded fighters a bury (BK 30, KS 60 clean / 50 late), against airborne ones a plain hit (BK 50,
    // KS 70 / 60); SDI x0, shield damage +0, no rebound (`rebound=f`) [R]; hit 1 type 3, hit 2 type 0 [R]. FAF 50. (The bury box's
    // `noprone`, which skips a lying fighter, is the bury's own rule here: lying fighters are never buried.)
    let mut rows = Vec::new();
    for (group, sign, f0) in [(0u8, 1, 9u8), (1, -1, 29)] {
        for (first, last, x, radius, damage, kbg_bury, kbg_air) in [
            (f0, f0, 130, 40, 60, 60, 70),
            (f0 + 1, f0 + 2, 120, 60, 30, 50, 60),
        ] {
            rows.push(ru(
                first,
                last,
                sign * x,
                30,
                radius,
                damage,
                60,
                30,
                kbg_bury,
                0,
                group,
            ));
            rows.push(ru(
                first,
                last,
                sign * x,
                30,
                radius,
                damage,
                60,
                50,
                kbg_air,
                1,
                group,
            ));
        }
    }
    let mut dsmash = ref_move(50, 0, 0, 255, &rows);
    for h in &mut dsmash.hitboxes {
        h.sdi = 0;
        h.rebounds = false;
        if h.damage < Fx::from_int(6) {
            h.hitlag = 0;
        }
        if h.priority == 0 {
            h.effect = EFFECT_BURY;
            h.targets = TARGETS_GROUNDED;
        }
        h.angle_type = if h.group == 0 {
            ANGLE_FORWARD
        } else {
            ANGLE_BY_POSITION
        };
    }
    dsmash.charge_at = Some(2);
    w.moves[MoveId::DSmash as usize] = dsmash;
}

/// A slingshot pellet (forward and back air): `damage` 7% / 9% clean and the reference's shield damage (`sd`, tenths), its later hits
/// 4% / 5% and 2.5% / 3% (`mid`, `late`). Clean (3 frames) radius 5.0, BK 55; then radius 1.2, BK 10 (6 frames, then 5); 361 degrees
/// by travel (type 3: the way it flies), KS 100 [R]: its ages 1-3, 4-9 and 10-14 (a shot first hits the frame after it appears). Its flight is ours: 2.8 of the reference's
/// units a frame `back` or forward, falling under 0.08 a frame squared (the 13.0.1 pellets arc down) from 6.0 ahead and 6.0 up [E].
fn pellet(frame: u8, back: bool, hits: [(i32, i32); 3]) -> ProjectileSpawn {
    let mk = |radius: i32, (damage, sd): (i32, i32), bkb: i16| {
        let mut h = shot_hit(radius, damage, 361, bkb, 100, ANGLE_FORWARD);
        h.shield_damage = shield(damage, sd);
        h
    };
    let clean = mk(50, hits[0], 55);
    ProjectileSpawn {
        frame,
        x: if back { rs(-6000) } else { rs(6000) },
        y: rs(6000),
        // (Backward for the back air: a negative speed along the facing.)
        speed: if back { rs(-2800) } else { rs(2800) },
        life: 15,
        hitbox: clean,
        end_damage: clean.damage,
        arc: ProjectileArc {
            gravity: rs(80),
            ..ProjectileArc::NONE
        },
        phases: phase_list(&[(4, mk(12, hits[1], 10)), (10, mk(12, hits[2], 10))]),
        pierce: false,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool::NONE,
    }
}

/// A turnip aerial's rows (the up air's or the down air's) for `n` turnips [R: ssbwiki.com raw tables].
fn turnip_rows(up: bool, n: u8) -> Vec<R> {
    if up {
        // Clean on 6-7 at 20.0 of the reference's units up, late on 8-21 at 18.0 up; 85 degrees, BK 40, KS 100.
        let (clean, cr, late, lr) = match n {
            3 => (130, 75, 60, 48),
            2 => (100, 65, 50, 45),
            _ => (80, 52, 40, 38),
        };
        vec![
            ru(6, 7, 0, 200, cr, clean, 85, 40, 100, 0, 0),
            ru(8, 21, 0, 180, lr, late, 85, 40, 100, 0, 0),
        ]
    } else {
        // Clean on 11-12: a meteor at the bottom (270 degrees, KS 80, BK 20 / 10 / 0) inside the 45-degree box (BK 30, KS 100), which
        // it beats; late on 13-26: 45 degrees, BK 30, KS 100; below the feet.
        let (clean, meteor_bkb, mr, cr, late, lr, cy) = match n {
            3 => (130, 20, 32, 75, 60, 48, -55),
            2 => (100, 10, 30, 65, 50, 45, -55),
            _ => (80, 0, 27, 52, 40, 38, -60),
        };
        vec![
            ru(11, 12, 5, cy, mr, clean, 270, meteor_bkb, 80, 0, 0),
            ru(11, 12, 5, cy, cr, clean, 45, 30, 100, 1, 0),
            ru(13, 26, 5, -40, lr, late, 45, 30, 100, 0, 0),
        ]
    }
}

fn aerials(w: &mut Weapon) {
    // Neutral air: a spin with arms out, one hit in two phases: clean on frames 3-10 9% (the hips radius 5.0, the hands 3.6), late on
    // 11-23 5% (4.0, 3.0); 361 degrees (type 0), BK 20, KS 100 [R]; the hips at 5.5 of the reference's units up, the hands 5.0 ahead and
    // behind at 7.0 up [E]. Landing lag 8, autocancels on frames 1-2 and from 27, FAF 36.
    let mut rows = Vec::new();
    for (first, last, damage, hip, hand) in [(3u8, 10u8, 90, 50, 36), (11, 23, 50, 40, 30)] {
        rows.push(ru(first, last, 0, 55, hip, damage, 361, 20, 100, 0, 0));
        rows.push(ru(first, last, 50, 70, hand, damage, 361, 20, 100, 1, 0));
        rows.push(ru(first, last, -50, 70, hand, damage, 361, 20, 100, 2, 0));
    }
    let mut nair = ref_move(36, 8, 2, 27, &rows);
    set_all(&mut nair, |h| h.angle_type = ANGLE_BY_POSITION);
    w.moves[MoveId::NAir as usize] = nair;

    // Forward air: the slingshot forward. The pellet leaves on frame 10: 7% / 4% / 2.5%, shield damage -3.5 / -2 / -1 (50%, 50%, 60%)
    // [R]. Landing lag 14, autocancels on frames 1-2 and from 30, FAF 40.
    let mut fair = ref_move(40, 14, 2, 30, &[]);
    fair.projectile = Some(pellet(8, false, [(70, -35), (40, -20), (25, -10)]));
    w.moves[MoveId::FAir as usize] = fair;

    // Back air: the slingshot behind. The pellet leaves on frame 13: 9% / 5% / 3%, shield damage -4.5 / -2.5 / -1.5 (50%) [R]. Landing
    // lag 14, autocancels on frames 1-2 and from 30, FAF 36.
    let mut bair = ref_move(36, 14, 2, 30, &[]);
    bair.projectile = Some(pellet(11, true, [(90, -45), (50, -25), (30, -15)]));
    w.moves[MoveId::BAir as usize] = bair;

    // Up air and down air: turnips held up / swung down, 3, 2 or 1 of them at random: 40%, 30%, 30% (the reference's) [R], drawn when
    // the move starts (E39: the 3-turnip version is the move's own slot). Every box type 0 [R]. Up air: landing lag 12, autocancels on
    // frames 1-5 and from 39, FAF 43. Down air: landing lag 12, autocancels on 1-10 and from 36 (ssbwiki.com; ultimateframedata.com
    // says 1-13 and 39: conflict 3), FAF 43.
    for (up, slots) in [(true, [UAIR_TWO, UAIR_ONE]), (false, [DAIR_TWO, DAIR_ONE])] {
        let build = |n: u8| {
            let mut mv = if up {
                ref_move(43, 12, 5, 39, &turnip_rows(true, n))
            } else {
                ref_move(43, 12, 10, 36, &turnip_rows(false, n))
            };
            set_all(&mut mv, |h| h.angle_type = ANGLE_BY_POSITION);
            mv
        };
        let mut three = build(3);
        three.versions = vec![(30, slots[0] as u8), (30, slots[1] as u8)];
        let id = if up { MoveId::UAir } else { MoveId::DAir };
        w.moves[id as usize] = three;
        w.moves[slots[0] as usize] = build(2);
        w.moves[slots[1] as usize] = build(1);
    }
}

fn grabs_and_throws(w: &mut Weapon) {
    // Grabs: a hand reaching ahead, the reference's grounded-target box (id 0) swept along its capsule at 5.5 of its units up [R]:
    // standing on frames 14-16 (radius 5.0, 4.0 to 14.0 ahead), FAF 44; from a dash on 16-18 (4.0, 4.0 to 13.0), FAF 48; pivot on
    // 17-19 (5.0, 5.0 to 14.0; written in front, since the engine turns a pivot grab round), FAF 45. (The aerial-target box is not
    // modelled: grabs catch only fighters on the ground here.)
    let grab = |faf: u8, first: u8, last: u8, from: i32, to: i32, radius: i32| {
        with_kind(
            ref_move(
                faf,
                0,
                0,
                255,
                &along_rows(
                    first,
                    last,
                    (from, 55),
                    (to, 55),
                    radius,
                    0,
                    361,
                    0,
                    0,
                    0,
                    0,
                ),
            ),
            HIT_GRAB,
        )
    };
    w.moves[MoveId::Grab as usize] = grab(44, 14, 16, 40, 140, 50);
    w.moves[MoveId::DashGrab as usize] = grab(48, 16, 18, 40, 130, 40);
    w.moves[MoveId::PivotGrab as usize] = grab(45, 17, 19, 50, 140, 50);

    // Pummel: on frame 3, 1.3%, set knockback 30 (KS 100), 361 (type 3), hitlag x2.1, radius 5.0 at 10.0 ahead and 7.0 up, never clanks
    // [R]. FAF 7 (its hitlag freezes the move, so it is not in the FAF).
    let mut pummel = with_kind(
        ref_move(
            7,
            0,
            0,
            255,
            &[ru(3, 3, 100, 70, 50, 13, 361, 0, 100, 0, 0)],
        ),
        HIT_PUMMEL,
    );
    set_all(&mut pummel, |h| {
        h.fixed_kb = 30;
        h.hitlag = 210;
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
    });
    w.moves[MoveId::Pummel as usize] = pummel;

    // Throws (no pre-hits; the thrower's invincibility is not modelled: a house limit). Forward: on frame 15, 9% at 45 degrees, BK 108,
    // KS 10, no hitlag, FAF 46. Back: turns round on 11 and throws behind, 11% at 135 degrees (backward from its first facing), BK 46,
    // KS 88, FAF 46. Up: on 13, 10% at 90, BK 114, KS 34, no hitlag, FAF 46. Down: 6% at 70, BK 78, KS 60, no hitlag; the reference
    // freezes both fighters for 6 frames on its frame 17 and releases on 18 with FAF 36 counted without them, so here (nobody frozen)
    // the release is on 24 and the FAF 42, the same timeline [D]. (The thrown fighter's 3% hit on bystanders is not modelled.)
    let throw = |faf: u8, frame: u8, damage: i32, angle: i16, bkb: i16, kbg: i16, lag: u16| {
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
        m.hitboxes[0].hitlag = lag;
        m
    };
    w.moves[MoveId::FThrow as usize] = throw(46, 15, 90, 45, 108, 10, 0);
    let mut bthrow = throw(46, 11, 110, 135, 46, 88, 100);
    bthrow.turns_around = true;
    w.moves[MoveId::BThrow as usize] = bthrow;
    w.moves[MoveId::UThrow as usize] = throw(46, 13, 100, 90, 114, 34, 0);
    w.moves[MoveId::DThrow as usize] = throw(42, 24, 60, 70, 78, 60, 0);
}

fn ledge_and_floor(w: &mut Weapon) {
    // Ledge attack: frames 23-25, 9%, 45 degrees (type 3), BK 90, KS 20, shield damage +1 (111 [D]), never clanks; intangible through
    // frame 25 (on the first grab: 1-20 and 1-12 on the second and third, the shared 80% / 50%), FAF 56. Its swept box (6.5 ahead to 4.0
    // behind, 5.0 up, radius 5.0) as circles [D].
    let mut ledge = ref_move(
        56,
        0,
        0,
        255,
        &along_rows(23, 25, (65, 50), (-40, 50), 50, 90, 45, 90, 20, 0, 0),
    );
    set_all(&mut ledge, |h| {
        h.shield_damage = shield(90, 10);
        h.angle_type = ANGLE_FORWARD;
        h.clanks = false;
        h.rebounds = false;
    });
    ledge.intangible = 25;
    w.moves[MoveId::LedgeAttack as usize] = ledge;

    // Get-up attack (lying face up, the reference's "back" floor attack): behind on frames 16-17 (12.0 to 4.0 of the reference's units
    // behind), then in front on 20-21 (12.0 to 4.0 ahead), 5.0 up, radius 5.0; 7%, 48 degrees (type 0), BK 80, KS 48, shield damage +8
    // (214 [D]), never clanks; intangible through frame 21, FAF 46. (The face-down and trip get-up attacks have no slot of their own.)
    let mut rows = along_rows(16, 17, (-120, 50), (-40, 50), 50, 70, 48, 80, 48, 0, 0);
    rows.extend(along_rows(
        20,
        21,
        (120, 50),
        (40, 50),
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
    getup.intangible = 21;
    w.moves[MoveId::GetUpAttack as usize] = getup;
}

/// The rocket's hits (`ride`: the reference's x2.2 / x1.5 when ridden: 15.4%, 11% and 18%) [R]: flying 7% at 60 degrees, BK 65, KS 75,
/// radius 3.3; slowing 5% at 55, BK 60, KS 75, radius 3.0; the explosion 12% at 68, BK 65, KS 90, radius 7.0, never clanking; hitlag
/// x0.8, type 3, shield damage -3.5 / -2.5 / -6 (50%) [R].
fn rocket_hits(ride: bool) -> (Hitbox, Hitbox, Hitbox) {
    let (fly, slow, blast) = if ride { (154, 110, 180) } else { (70, 50, 120) };
    let mk = |radius: i32, damage: i32, angle: i16, bkb: i16, kbg: i16| {
        let mut h = shot_hit(radius, damage, angle, bkb, kbg, ANGLE_FORWARD);
        h.hitlag = 80;
        h.shield_damage = 50;
        h
    };
    let mut boom = mk(70, blast, 68, 65, 90);
    boom.clanks = false;
    boom.rebounds = false;
    (mk(33, fly, 60, 65, 75), mk(30, slow, 55, 60, 75), boom)
}

/// The side special: the rocket (the reference's, 13.0.1). The settler pulls it out and sets it down in front (FAF 36); held, the
/// settler rides it (`RIDE`, from frame 16 [E]). Let go, it is a projectile (the script `pockets_rocket.script` flies it): it appears
/// on frame 20 [E] and stays put without a hit until frame 52, flies with its 7% hit from 52 to 116, slows with its 5% hit from 117
/// to 176 [R: the hit windows], and bursts at the end (the 12% explosion) or against a wall [R]; a fighter it touches takes its hit
/// and it is gone (no burst) [R]. One out at a time (E3d, kind 4): a second use does nothing [R]. In the air the settler's speed stops
/// as it starts [R]. Its flight is ours: 1.7 of the reference's units a frame, then slowing by 1.7 / 60 a frame while dropping under
/// 0.04 a frame squared to at most 1.0 [E: unpublished, a little less than a large stage's length].
fn rocket(w: &mut Weapon) {
    let (fly, slow, blast) = rocket_hits(false);
    let mut mv = ref_move(36, 0, 0, 255, &[]);
    mv.start_speed = 0;
    mv.start_speed_y = 0;
    mv.land_continue = true;
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_rocket_hold.script"),
    ));
    let gap = Hitbox {
        radius: Fx::ZERO,
        ..fly
    };
    mv.projectile = Some(ProjectileSpawn {
        frame: 19,
        x: rs(10000),
        y: rs(4000),
        // (A sliver of speed that only gives the flight its direction: the script holds it still until it flies.)
        speed: Fx::from_raw(1),
        life: 157,
        hitbox: gap,
        end_damage: Fx::ZERO,
        arc: ProjectileArc::NONE,
        phases: phase_list(&[(32, fly), (97, slow)]),
        pierce: false,
        rehit: 0,
        burst: Some(Burst {
            life: 4,
            rehit: 0,
            on_wall: true,
            y: Fx::ZERO,
            phases: phase_list(&[(1, blast)]),
            on_ground: false,
            on_expire: true,
            keep_hits: false,
            on_hit: false,
            on_shield: true,
        }),
        charged: None,
        tool: Tool {
            kind: ROCKET_KIND,
            limit: 1,
            // (Pocketed, it frees its maker to set down another: the reference's exception, E42.)
            pocket: POCKET_FREES,
            ..Tool::NONE
        },
    });
    mv.projectile_script = Some(script(
        Kind::Projectile,
        include_str!("scripts/pockets_rocket.script"),
    ));
    w.moves[MoveId::SideSpecial as usize] = mv;

    // Ridden (`RIDE`, from the hold's frame 16 [E]): the settler sits on the rocket, which waits until frame 52, flies to 116 with its
    // 15.4% hit, slows to 176 with 11%, and bursts on 177-179 with 18% [R: the windows, the damage]; the stick speeds it up or slows it
    // [R], by up to 0.6 of the reference's units a frame [E]. A hit on anyone ends it in the bounce (`RIDE_BOUNCE`: not helpless [R]);
    // a wall in the crash (`RIDE_CRASH`: the 18% burst, then helpless [R]); jump in the hop off (`RIDE_HOP`: helpless [R]); the burst at
    // the flight's end leaves the settler helpless [R]. The script `pockets_ride.script`.
    let (rfly, rslow, rblast) = rocket_hits(true);
    let place = |h: Hitbox, first: u8, last: u8, group: u8| Hitbox {
        start: first - 16,
        end: last - 16,
        x: Fx::from_ratio(u(40), 10),
        y: Fx::from_ratio(u(20), 10),
        group,
        ..h
    };
    let mut ride = ref_move(182 - 15, 0, 0, 255, &[]);
    ride.hitboxes = vec![
        place(rfly, 52, 116, 0),
        place(rslow, 117, 176, 0),
        place(rblast, 177, 179, 1),
    ];
    ride.helpless_after = true;
    ride.land_continue = true;
    ride.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_ride.script"),
    ));
    w.moves[RIDE as usize] = ride;

    // The bounce off a hit: up at 2.0 of the reference's units a frame for 6 frames, then the body's own fall; not helpless; FAF 25 [E].
    let mut bounce = ref_move(25, 0, 0, 255, &[]);
    bounce.motion = vec![Motion {
        start: 0,
        end: 5,
        vx: Fx::ZERO,
        vy: rs(2000),
    }];
    w.moves[RIDE_BOUNCE as usize] = bounce;
    // The crash into a wall: the burst (18%, the wall burst's radius 6.5, 3.1 up [R]) on its frames 2-4, then helpless; 25 frames [E].
    let mut crash = ref_move(27, 0, 0, 255, &[]);
    crash.hitboxes = vec![Hitbox {
        start: 1,
        end: 3,
        x: Fx::from_ratio(u(40), 10),
        y: Fx::from_ratio(u(31), 10),
        radius: Fx::from_ratio(u(65), 10),
        ..rblast
    }];
    crash.helpless_after = true;
    w.moves[RIDE_CRASH as usize] = crash;
    // The hop off: up at 1.5 of the reference's units a frame for 4 frames, then helpless; 15 frames [E].
    let mut hop = ref_move(17, 0, 0, 255, &[]);
    hop.motion = vec![Motion {
        start: 0,
        end: 3,
        vx: Fx::ZERO,
        vy: rs(1500),
    }];
    hop.helpless_after = true;
    w.moves[RIDE_HOP as usize] = hop;
}

/// The balloons (E41) of the up special. The reference publishes none of these numbers (gaps 4-7) [E]: two balloons, one when the
/// helium is below half; each popped by any hit (0.1%); a tank of 300 frames of flight that refills at 6 / 16 of a frame a frame (a
/// full refill in about 13 seconds); one circle of radius 0.55 world units over the head, 3.0 up (the balloons on their strings above
/// our taller body); a pop of the last balloon sends the settler up at 1.0 of the reference's units a frame.
pub fn balloon_block() -> Balloons {
    Balloons {
        count: 2,
        hp: Fx::from_ratio(1, 10),
        x: Fx::ZERO,
        y: Fx::from_int(3),
        radius: Fx::from_ratio(55, 100),
        fuel: 300,
        refill: 6,
        one_below: 150,
        pop_vy: rs(1000),
    }
}

/// The up special: the balloon flight (the reference's, 13.0.1). The settler takes off and flies on balloons (E41): special pressed or
/// held flaps up, the stick steers, nothing pressed sinks, attack lets the balloons go (helpless) [R]; the flight lasts until the
/// helium runs out (helpless), or the balloons are popped (helpless, a little hop) [R]. Landing with the balloons: 20 frames of lag;
/// from the special fall: 30 [R]. Ledges from frame 3 [E]. The flight's physics are ours (gap 5) [E]: gravity 0.06 and fall 1.0 of the
/// reference's units, the body's air speed and acceleration, a flap setting the rising speed to 1.4 (two balloons) or 0.9 (one), every
/// 6 frames for presses and 12 held, rising at most 1.4; the take-off 1.6 up for its first 3 frames (the script `pockets_balloon.script`).
/// The move goes on as `FLIGHT` every 248 frames (a move lasts at most 253 here).
fn balloons(w: &mut Weapon) {
    let build = |first: bool| {
        let mut mv = ref_move(252, 20, 0, 255, &[]);
        mv.balloons = Some(balloon_block());
        mv.helpless_after = true;
        mv.fall_landing_lag = 30;
        mv.air_gravity = Some(rs(60));
        mv.air_max_fall = Some(rs(1000));
        mv.air_max_rise = Some(rs(1400));
        mv.ledge_front_from = 2;
        mv.ledge_back_from = 2;
        // (No fast fall on the balloons [E].)
        mv.fast_fall_from = 255;
        mv.script = Some(if first {
            script(
                Kind::Fighter,
                include_str!("scripts/pockets_balloon.script"),
            )
        } else {
            script(Kind::Fighter, include_str!("scripts/pockets_flight.script"))
        });
        mv
    };
    w.moves[MoveId::UpSpecial as usize] = build(true);
    w.moves[FLIGHT as usize] = build(false);
}

// ---- Phase 2: the pocket (neutral special) and the tree (down special) ------------------------------------------------------------

/// The pocket's release (the neutral special goes on as it when something is in the pocket).
pub const POCKET_OUT: MoveId = MoveId::Ext9;
/// The down special's four stages: the sapling planted, the watering can, the axe, and the planting that fails in the air.
pub const PLANT: MoveId = MoveId::Ext10;
pub const WATER: MoveId = MoveId::Ext11;
pub const AXE: MoveId = MoveId::Ext12;
pub const PLANT_FAILS: MoveId = MoveId::Ext13;
/// Holders of the tree's two projectiles (never started by a button): the growth's hit and the falling tree.
pub const TREE_GROWS: MoveId = MoveId::Ext14;
pub const TREE_FALLS: MoveId = MoveId::Ext15;
/// The weapon's items (E4 / E43): the sapling, the tree and the wood chip.
pub const SAPLING_ITEM: u8 = 0;
pub const TREE_ITEM: u8 = 1;
pub const CHIP_ITEM: u8 = 2;
/// Their kinds, and the falling tree's tool kind: what the down special's `owns(kind)` reads.
pub const SAPLING_KIND: u8 = 5;
pub const TREE_KIND: u8 = 6;
pub const FALL_KIND: u8 = 7;
pub const CHIP_KIND: u8 = 8;

/// A move reached by a script's `goto` on its parent special's frame 1 (the first a script sees): its frame k is the parent's k + 1,
/// so a reference frame `f` of the whole special is the child's `f - 1` in the reference's numbering [D]. `ref_move` with that shift.
fn child_move(faf: u8, rows: &[R]) -> Move {
    ref_move(faf - 1, 0, 0, 255, rows)
}

/// The neutral special: the pocket (13.0.1; ssbwiki.com's raw neutral special page and move page: the eleventh moveset's
/// `REFERENCE_DATA.md`).
///
/// * The reach: intangible on frames 5-23 and catching on 8-23, FAF 51 (animation 63) [R]. On those frames a circle in front of the
///   hand puts away the first projectile (of another owner) or item (not held) that touches it, while the pocket is empty (E42): 9.0
///   of the reference's units ahead, 7.0 up, radius 6.0 [E: the reference's catch area is only shown in hitbox footage]. Held an
///   item, the settler puts that item away instead, an item already in the pocket coming into the hand [R: the swap]. Kept until
///   taken out or the stock is lost [R]; no time limit [R: 13.0.1].
/// * The release (`POCKET_OUT`, the script `pockets_pocket.script` goes on as it on frame 1 with a projectile in the pocket, or an
///   item and an empty hand): the thing comes out on frame 9, FAF 31 (animation 35) [R]. An item into the hand; a projectile flies from
///   the hand (7.0 ahead, 6.0 up [E]) along the trajectory it had as it was caught, turned forward, dealing 1.9x its own damage (its
///   earlier multipliers dropped) [R], its clock starting again [R: timers reset on retrieval], staled as the release [R: the
///   staleness of the pocket's release].
/// * Which projectiles: any that can be reflected (`Tool::pocket`): not a burst [E], nor the needles' line (held by its thrower) or the
///   seer's explosive flame (a standing explosion), as the reference's rule for things with no model of their own [R rule; E: which].
///   The rocket frees its maker to fire another [R]; any other pocketed tool still counts as its maker's [R: "still in play"].
/// * In the air the same frames; touching down carries the move on [E].
fn pocket(w: &mut Weapon) {
    let mut mv = ref_move(51, 0, 0, 255, &[]);
    mv.intangible_from = 4;
    mv.intangible = 23;
    mv.intangible_from_air = 255;
    mv.land_continue = true;
    mv.pocket = Some(PocketCatch {
        start: 7,
        end: 22,
        x: rs(9000),
        y: rs(7000),
        radius: rs(6000),
    });
    mv.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_pocket.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = mv;

    // The release: out on the special's frame 9 (the child's 8, move frame 7), FAF 31 (the child's 30).
    let mut out = child_move(31, &[]);
    out.land_continue = true;
    out.unpocket = Some(PocketRelease {
        frame: 7,
        x: rs(7000),
        y: rs(6000),
        power: 190,
    });
    w.moves[POCKET_OUT as usize] = out;
}

/// The down special: the tree (13.0.1; ssbwiki.com's raw down special table and move page, ultimateframedata.com: the eleventh moveset's
/// `REFERENCE_DATA.md`).
///
/// The press picks the stage by what the settler has out (`pockets_tree.script`, `owns(kind)`): its tree standing or falling, the
/// axe; its sapling, the watering can; else, on the ground, the planting, and in the air the planting that fails [R].
///
/// * **Planting** (`PLANT`): FAF 39 [R]; the sapling (E43 placed item) is set down 10.0 of the reference's units ahead on the frame 20
///   [E: neither published]; one each (planting pulls up the settler's older one, and any sapling it is set on [R]). The sapling lasts
///   840 frames [R], cannot be hit [E], and grows with 8 frames of water [E: the amount is not published; R: it need not come at
///   once] 26 frames after its water is full, so a full watering from where it was planted has the growth's clean hit 46 frames after
///   the water first touched it [R: ultimateframedata.com's 46; D: the delay].
/// * **Failing** (`PLANT_FAILS`, in the air): a shrug, the planting's FAF [E].
/// * **Watering** (`WATER`): FAF 50, water on frames 5-20 [R]: 0%, 55 degrees forward (type 3), set knockback 25 (BK 0, KS 100),
///   radius 2.9, a push (the reference's flinchless windbox) [R]; reflectable and pocketable [R]. Here the water is one shot poured
///   from the spout (11.0 ahead, 9.0 up) at 0.35 of the reference's units a frame forward, falling under 0.07 [E], pushing each fighter
///   every frame it touches [E: the house's windbox], each frame touching a sapling giving it one frame of water (the waterer is
///   credited with the growth [R]). The settler may move and jump after it as the move's frames allow [R].
/// * **The growth** (`TREE_GROWS`'s projectile, at the sapling): clean on 14-19 18% (shield damage +6), late on 20-40 13%, 80 degrees,
///   BK 30, KS 100 [R]; clean radius 7.0 along 5.0 to 12.0 up (a standing capsule), late radius 9.0 at 11.0 up [R]; each fighter once.
///   The tree (E43): 30 hp, 600 frames [R]; its hurtbox a capsule of radius 6.0 from 3.0 to 21.0 up [E]; any hit takes hp, a shot is
///   used up on it [R: projectiles are blocked by the tree]; at 0 it is gone [R].
/// * **The axe** (`AXE`): early 6-7, late 8, FAF 48 [R]; 14%, angle 361 (type 0), BK 36, KS 100, shield damage +9 [R]: the head
///   (radius 5.0) and handle (3.0) on the held axe [R radii; E positions], and on 8 two more on the root (5.0 at 11.5 ahead / 5.5 up,
///   3.0 at 5.5 ahead / 6.0 up) [R]. A hit on a tree cuts it on that side; a second cut on a side fells it [R]; the settler is frozen
///   7 frames by the tree (ultimateframedata.com: 54 frames against 47, which it says include 13 of hitlag) [D: conflict 4].
/// * **The fall** (`TREE_FALLS`'s projectile, credited to the chopper, from the tree's feet away from the chopper): hits on 14 on,
///   25%, 60 degrees, BK 30, KS 100, shield damage +6, radius 7.0 at 17.5 up [R]; reflectable and pocketable [R]. Its flight is ours
///   [E]: it falls from 17.5 up at 0.6 forward under 0.16, lands near frame 15, bounces once [R] high enough to come down again near
///   frame 46 [R: 46 frames on the stage], and is gone; off a ledge it falls on [R] (its life stops at 255 frames, the reference's
///   313 [D]). With 48.5% [R] a wood chip (an item anyone can pick up, 3% thrown [R]; its other numbers [E]) lands where the tree stood;
///   the reference's 1.5% fruit is not modelled.
fn tree(w: &mut Weapon) {
    let mut router = ref_move(39, 0, 0, 255, &[]);
    router.land_continue = true;
    router.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_tree.script"),
    ));
    w.moves[MoveId::DownSpecial as usize] = router;

    // Planting: the sapling on the special's frame 20 (the child's move frame 18).
    let mut plant = child_move(39, &[]);
    plant.pull = Some((18, SAPLING_ITEM));
    w.moves[PLANT as usize] = plant;
    let mut fails = child_move(39, &[]);
    fails.land_continue = true;
    fails.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/pockets_shrug.script"),
    ));
    w.moves[PLANT_FAILS as usize] = fails;

    // Watering: the water appears on the special's frame 4 (the child's move frame 2) and hits on 5-20 (ages 1-16).
    let mut water = row(&ru(1, 1, 0, 0, 29, 0, 55, 0, 100, 0, 0));
    water.fixed_kb = 25;
    water.push = true;
    water.angle_type = ANGLE_FORWARD;
    let mut can = child_move(50, &[]);
    can.land_continue = true;
    can.projectile = Some(ProjectileSpawn {
        frame: 2,
        x: rs(11000),
        y: rs(9000),
        speed: rs(350),
        life: 17,
        hitbox: water,
        end_damage: Fx::ZERO,
        arc: ProjectileArc {
            gravity: rs(70),
            ..ProjectileArc::NONE
        },
        phases: [None; MAX_PHASES],
        pierce: true,
        rehit: 1,
        burst: None,
        charged: None,
        tool: Tool {
            water: 1,
            ..Tool::NONE
        },
    });
    w.moves[WATER as usize] = can;

    // The axe: early on the special's 6-7, late on 8 (the child's 5-6 and 7), FAF 48 (the child's 47).
    // (Owner decision 73: the early swing reaches higher and a little further than the reference's head box alone, the axe head at the
    // top of its arc, so jab 1 > jab 2 > axe connects where players report it: a second head box at 12.0 ahead and 17.0 up, radius 6.0
    // [E]; same hit.)
    let axe_rows = [
        ru(5, 6, 90, 120, 50, 140, 361, 36, 100, 0, 0),
        ru(5, 6, 120, 170, 60, 140, 361, 36, 100, 0, 0),
        ru(5, 6, 60, 90, 30, 140, 361, 36, 100, 1, 0),
        ru(7, 7, 110, 70, 50, 140, 361, 36, 100, 0, 0),
        ru(7, 7, 75, 60, 30, 140, 361, 36, 100, 1, 0),
        ru(7, 7, 115, 55, 50, 140, 361, 36, 100, 2, 0),
        ru(7, 7, 55, 60, 30, 140, 361, 36, 100, 3, 0),
    ];
    let mut axe = child_move(48, &axe_rows);
    set_all(&mut axe, |h| {
        h.angle_type = ANGLE_BY_POSITION;
        h.shield_damage = shield(140, 90);
    });
    axe.chops = true;
    axe.land_continue = true;
    w.moves[AXE as usize] = axe;

    // The growth's hit (fired at the sapling as the tree appears; ages 13-18 clean, 19-39 late).
    let mut clean = shot_hit(70, 180, 80, 30, 100, ANGLE_BY_POSITION);
    clean.y = rs(8500);
    clean.half_height = rs(3500);
    clean.shield_damage = shield(180, 60);
    clean.rebounds = false;
    let mut late = shot_hit(90, 130, 80, 30, 100, ANGLE_BY_POSITION);
    late.y = rs(11000);
    late.shield_damage = shield(130, 60);
    late.rebounds = false;
    let gap = Hitbox {
        radius: Fx::ZERO,
        ..clean
    };
    let mut grows = ref_move(3, 0, 0, 255, &[]);
    grows.projectile = Some(ProjectileSpawn {
        frame: 255,
        x: Fx::ZERO,
        y: Fx::ZERO,
        speed: Fx::ZERO,
        life: 40,
        hitbox: gap,
        end_damage: Fx::ZERO,
        arc: ProjectileArc::NONE,
        phases: phase_list(&[(13, clean), (19, late)]),
        pierce: true,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool {
            pocket: POCKET_NEVER,
            ..Tool::NONE
        },
    });
    w.moves[TREE_GROWS as usize] = grows;

    // The falling tree (from the tree's feet, away from the chopper; hits from age 13, the reference's frame 14).
    let mut falling = shot_hit(70, 250, 60, 30, 100, ANGLE_BY_POSITION);
    falling.shield_damage = shield(250, 60);
    falling.half_width = rs(5500);
    let fall_gap = Hitbox {
        radius: Fx::ZERO,
        ..falling
    };
    let mut falls = ref_move(3, 0, 0, 255, &[]);
    falls.projectile = Some(ProjectileSpawn {
        frame: 255,
        x: Fx::ZERO,
        y: rs(17500),
        speed: rs(600),
        life: 255,
        hitbox: fall_gap,
        end_damage: Fx::ZERO,
        arc: ProjectileArc {
            gravity: rs(160),
            max_fall: rs(2400),
            bounces: 1,
            bounce_min: rs(2480),
            bounce_max: rs(2480),
            bounce_keep: 100,
            ..ProjectileArc::NONE
        },
        phases: phase_list(&[(13, falling)]),
        pierce: true,
        rehit: 0,
        burst: None,
        charged: None,
        tool: Tool {
            kind: FALL_KIND,
            ..Tool::NONE
        },
    });
    w.moves[TREE_FALLS as usize] = falls;

    w.items = vec![sapling_item(), tree_item(), chip_item()];
}

/// The sapling (E43): set down, 840 frames, grows with 8 frames of water 26 frames later [see `tree`].
fn sapling_item() -> ItemSpec {
    let mut it = ItemSpec::with_hit(shot_hit(10, 0, 361, 0, 0, ANGLE_BY_POSITION));
    it.kind = SAPLING_KIND;
    it.pocketable = false;
    it.half_width = rs(2000);
    it.half_height = rs(3000);
    it.hurt_radius = rs(2000);
    it.life = 840;
    it.placed = Some(Placed {
        x: rs(10000),
        water: 8,
        grow_delay: 26,
        grow_into: TREE_ITEM,
        grow_move: TREE_GROWS as u8,
        ..Placed::PLAIN
    });
    it
}

/// The tree (E43): 30 hp, 600 frames [R]; two cuts on a side fell it [R]; its hurtbox a capsule of radius 6.0 around 12.0 up, 9.0 above
/// and below [E]; an attacker hitting it is frozen half the hitlag (the axe: 7 frames) [D].
fn tree_item() -> ItemSpec {
    let mut it = ItemSpec::with_hit(shot_hit(10, 0, 361, 0, 0, ANGLE_BY_POSITION));
    it.kind = TREE_KIND;
    it.pocketable = false;
    it.half_width = rs(6000);
    it.half_height = rs(12000);
    it.hurt_radius = rs(6000);
    it.hp = Fx::from_int(30);
    it.life = 600;
    it.placed = Some(Placed {
        hurt_half_height: rs(9000),
        hitter_lag: 50,
        cuts: 2,
        fall_move: TREE_FALLS as u8,
        chip: CHIP_ITEM,
        chip_chance: 485,
        ..Placed::PLAIN
    });
    it
}

/// The wood chip: an item anyone can pick up and throw, 3% [R]; light, it falls at the bomb's speeds and lies where it lands [E].
fn chip_item() -> ItemSpec {
    let mut it = ItemSpec::with_hit(shot_hit(25, 30, 361, 20, 60, ANGLE_BY_TRAVEL));
    it.kind = CHIP_KIND;
    it.gravity = rs(70);
    it.max_fall = rs(2300);
    it.friction = rs(30);
    it.half_width = rs(2500);
    it.half_height = rs(2000);
    it.pick_width = rs(5000);
    it.pick_height = rs(5000);
    it.bounce_min = Fx::from_int(8);
    it.bounce_x = (Fx::ZERO, Fx::from_int(8));
    it.bounce_y = (Fx::ZERO, Fx::from_int(8));
    it.thrown_life = 40;
    it.fighter_keep = (20, 20);
    it.fighter_lift = rs(1000);
    it
}
