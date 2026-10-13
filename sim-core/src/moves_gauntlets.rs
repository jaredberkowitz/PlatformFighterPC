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
    // The reference's angle types (ssbwiki.com hitbox tables, 13.0.1; sim 67), on every normal with a box reaching the fighter's
    // centre or behind it: forward (type 3) for the forward tilt, neutral and forward air and the ledge attack; backward (4) for
    // the back air; by position (0) for the down tilt (its box at the body sends a target behind the other way), down smash, up and
    // down air and the get-up attack; the up smash's first-hit boxes by position except the one behind the head (forward), its
    // second hit forward.
    angle_types(
        &mut w,
        &[
            (MoveId::FTilt, ANGLE_FORWARD),
            (MoveId::DTilt, ANGLE_BY_POSITION),
            (MoveId::DSmash, ANGLE_BY_POSITION),
            (MoveId::NAir, ANGLE_FORWARD),
            (MoveId::FAir, ANGLE_FORWARD),
            (MoveId::BAir, ANGLE_BACKWARD),
            (MoveId::UAir, ANGLE_BY_POSITION),
            (MoveId::DAir, ANGLE_BY_POSITION),
            (MoveId::LedgeAttack, ANGLE_FORWARD),
            (MoveId::GetUpAttack, ANGLE_BY_POSITION),
        ],
    );
    angle_type_rows(&mut w, MoveId::USmash, &[0, 1, 2], ANGLE_BY_POSITION);
    angle_type_rows(&mut w, MoveId::USmash, &[3, 4, 5, 6], ANGLE_FORWARD);
    raptor_boost(&mut w);
    falcon_dive(&mut w);
    falcon_punch(&mut w);
    falcon_kick(&mut w);
    w
}

const KICK_AIR: MoveId = MoveId::Ext7;
const KICK_LANDING: MoveId = MoveId::Ext8;
const KICK_OFF_EDGE: MoveId = MoveId::Ext9;
const KICK_CRASH: MoveId = MoveId::Ext13;

/// Down special, a flaming slide kick on the ground and a diving kick in the air (the reference's, ssbwiki.com 13.0.1).
///
/// * On the ground (down special): one hit in three phases, weaker as it goes: clean 13-16 (15%, 52 degrees, BK 60, KS 88), mid
///   17-23 (11%, 52, BK 90, KS 60), late 24-33 (7%, 46, BK 80, KS 65); FAF 69. Running into a wall on frames 14-36 crashes it (ext13, a
///   53-frame crash). Going over an edge it goes on as ext9, the same kick with FAF 66, keeping its speed in the air. A hit slows the
///   slide to 0.8.
/// * In the air (ext7): a diagonal dive, clean 14-18 (15%, 361, BK 80, KS 57, hitlag x1.2), mid 19-24 (12%, KS 46), late 25-27 (9%,
///   KS 46); FAF 57, not helpless. Touching down during it is the landing hit (ext8): 9% at 60 degrees (BK 80, KS 35) on both sides,
///   on its first two frames, then 35 frames of landing lag in all (FAF 36).
///
/// Positions on the leg [E]; the slide's and the dive's speeds are the reference's animation, not published: [E] (see "Fitted
/// distances" in the reference data).
fn falcon_kick(w: &mut Weapon) {
    let slide_rows = |shift: u8| -> Vec<R> {
        let mut rows = Vec::new();
        for (first, last, damage, angle, bkb, kbg, radius) in [
            (13, 16, 150, 52, 60, 88, 5),
            (17, 23, 110, 52, 90, 60, 4),
            (24, 33, 70, 46, 80, 65, 4),
        ] {
            for x in [12, 6] {
                rows.push(r(
                    first - shift,
                    last - shift,
                    x,
                    4,
                    radius,
                    damage,
                    angle,
                    bkb,
                    kbg,
                    0,
                    0,
                ));
            }
        }
        rows
    };
    let mut slide = ref_move(69, 0, 0, 255, &slide_rows(0));
    slide.air_version = Some(KICK_AIR as u8);
    slide.off_edge = Some(KICK_OFF_EDGE as u8);
    slide.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_kick.script"),
    ));
    w.moves[MoveId::DownSpecial as usize] = slide;

    // Over an edge: the same kick (its hits so far kept), FAF 66; the slide's speed carries on through its frames, then the air.
    let mut edge = ref_move(66, 0, 0, 255, &slide_rows(0));
    edge.air_blocks = vec![AirBlock {
        from: 0,
        to: 32,
        gravity: None,
        max_fall: None,
        friction: Some(Fx::ZERO),
        drift: 0,
        accel: 100,
    }];
    w.moves[KICK_OFF_EDGE as usize] = edge;

    let mut crash = ref_move(54, 0, 0, 255, &[]);
    crash.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_kick_crash.script"),
    ));
    w.moves[KICK_CRASH as usize] = crash;

    let mut dive = ref_move(
        57,
        0,
        0,
        255,
        &[
            r(14, 18, 8, 4, 7, 150, 361, 80, 57, 0, 0),
            r(14, 18, 4, 7, 7, 150, 361, 80, 57, 1, 0),
            r(19, 24, 8, 4, 6, 120, 361, 80, 46, 0, 0),
            r(19, 24, 4, 7, 6, 120, 361, 80, 46, 1, 0),
            r(25, 27, 8, 4, 6, 90, 361, 80, 46, 0, 0),
            r(25, 27, 4, 7, 6, 90, 361, 80, 46, 1, 0),
        ],
    );
    set_rows(&mut dive, &[0, 1], |h| h.hitlag = 120);
    dive.motion = vec![
        Motion {
            start: 0,
            end: 12,
            vx: su(300),
            vy: Fx::ZERO,
        },
        Motion {
            start: 13,
            end: 26,
            vx: su(2100),
            vy: -su(2100),
        },
    ];
    dive.landing_move = Some(KICK_LANDING as u8);
    w.moves[KICK_AIR as usize] = dive;

    let landing_rows: Vec<R> = [9, 3, -3, -9]
        .iter()
        .map(|&x| r(1, 2, x, 3, 6, 90, 60, 80, 35, 0, 0))
        .collect();
    w.moves[KICK_LANDING as usize] = ref_move(36, 0, 0, 255, &landing_rows);
}

const PUNCH_REVERSED: MoveId = MoveId::Ext0;
const PUNCH_AIR: MoveId = MoveId::Ext1;
const PUNCH_AIR_REVERSED: MoveId = MoveId::Ext2;

/// Neutral special, a wound-up flaming punch (the reference's, ssbwiki.com 13.0.1). Four versions:
///
/// * On the ground (neutral special): the hit on frames 53-57, 25%, angle 361, BK 93, KS 59; FAF 104.
/// * Reversed on the ground (ext0): the stick held back on frame 15 [E: past 0.25 of its travel] switches to it; it turns round on
///   frame 35 and hits on 62-66 for 28% (BK 85, KS 65); FAF 118, all counted from the start of the punch. The reversed slots begin at
///   the reference's frame 15 (their frame 0), so their rows are the reference's minus 14.
/// * In the air (ext1): 22% with boxes x1.15 the size; on starting a set upward speed of 0.2 (reference units a frame), the
///   horizontal speed x0.92 a frame for 19 frames (then the body's air control), and on frame 51 the stick held up or down between
///   0.25 and 0.65 of its travel gives a boost along 35 degrees up or down [E: its speed is not published, 1.0 here].
/// * Reversed in the air (ext2): 25%, the same boxes; the turn on 35, the hit on 62-66, the boost read on 60, FAF 118.
///
/// Every box: hitlag x1.5, and it never clanks (the reference's `clang=f`). Positions on the arm and shoulder [E]. Touching down
/// in the air versions carries the punch on along the ground [E: not published].
fn falcon_punch(w: &mut Weapon) {
    // The three boxes (fist, upper arm, forearm) of `damage` tenths at `bkb` / `kbg`, from reference frame `first` to `last`;
    // `big` the aerial x1.15 radii.
    let punch = |first: u8, last: u8, damage: i32, bkb: i16, kbg: i16, big: bool| -> Vec<R> {
        let (r0, r1, r2) = if big { (6, 4, 6) } else { (6, 3, 5) };
        vec![
            r(first, last, 13, 13, r0, damage, 361, bkb, kbg, 0, 0),
            r(first, last, 8, 14, r1, damage, 361, bkb, kbg, 1, 0),
            r(first, last, 18, 13, r2, damage, 361, bkb, kbg, 2, 0),
        ]
    };
    let finish = |mut m: Move| {
        for h in &mut m.hitboxes {
            h.hitlag = 150;
            h.clanks = false;
        }
        m
    };
    // The reversed slots start on the reference's frame 15: hit 62-66 is their 48-52, FAF 118 their 104.
    let mut ground = finish(ref_move(104, 0, 0, 255, &punch(53, 57, 250, 93, 59, false)));
    ground.air_version = Some(PUNCH_AIR as u8);
    ground.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_punch.script"),
    ));
    w.moves[MoveId::NSpecial as usize] = ground;

    let mut reversed = finish(ref_move(104, 0, 0, 255, &punch(48, 52, 280, 85, 65, false)));
    reversed.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_punch_reversed.script"),
    ));
    w.moves[PUNCH_REVERSED as usize] = reversed;

    // In the air: no air control and no air friction while the speed is cut by 0.92 a frame (frames 1-19).
    let still = |to: u8| AirBlock {
        from: 0,
        to,
        gravity: None,
        max_fall: None,
        friction: Some(Fx::ZERO),
        drift: 0,
        accel: 100,
    };
    let mut air = finish(ref_move(104, 0, 0, 255, &punch(53, 57, 220, 93, 59, true)));
    air.air_blocks = vec![still(18)];
    air.land_continue = true;
    air.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_punch_air.script"),
    ));
    w.moves[PUNCH_AIR as usize] = air;

    let mut air_reversed = finish(ref_move(104, 0, 0, 255, &punch(48, 52, 250, 93, 59, true)));
    air_reversed.air_blocks = vec![still(4)];
    air_reversed.land_continue = true;
    air_reversed.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_punch_air_reversed.script"),
    ));
    w.moves[PUNCH_AIR_REVERSED as usize] = air_reversed;
}

const DIVE_CATCH: MoveId = MoveId::Ext6;

/// Up special, a leaping command grab that explodes (the reference's, ssbwiki.com 13.0.1).
///
/// * The leap (up special): grab boxes on frame 14 (5.0 reference units at 13.0 up and 7.0 ahead; 5.5 at 8.8 up and 13.7 ahead on
///   grounded fighters only) and 15-30 (5.0 at 15.0 and at 11.0 up, 6.0 ahead), converted [D]. They are command grabs: they catch
///   fighters on the ground or in the air, shielding or in hitstun. The turnaround only on frame 14, with the stick held back past
///   0.25 (the reference's B-reverse frame); before it, x6 horizontal acceleration in the air (the reference's "Tokyo drift"), air
///   speed x0.95 throughout; after the rise, gravity 0.103. Fast fall from frame 50. Ledges in front from frame 20, behind from 37.
///   Animation 66, then the special fall (air speed x0.7, landing lag 24).
/// * The catch (ext6): the held fighter at the fist [E]; on frames 2-3 an explosion of 5% (361, BK 50, KS 100) on the held fighter
///   and on anyone else it reaches (radius 9.6 at the body, [D]); the throw on frame 17: 13%, 50 degrees, BK 85, KS 62. FAF 40, then
///   free (not helpless: the leap can be used again). Touching down during the catch does not end it [E: not published].
///
/// The rise and the backflip after the throw are the reference's animation, not published: [E] (see "Fitted distances" in the
/// reference data).
fn falcon_dive(w: &mut Weapon) {
    let grab = |first: u8, last: u8, x: i32, y: i32, radius: i32, priority: u8| {
        let mut h = row(&r(first, last, x, y, radius, 0, 361, 0, 0, priority, 0));
        h.kind = HIT_GRAB;
        h.command = true;
        h
    };
    let mut dive = ref_move(67, 0, 0, 255, &[]);
    dive.hitboxes = vec![
        grab(14, 14, 9, 16, 6, 0),
        grab(14, 14, 17, 11, 7, 1),
        grab(15, 30, 8, 19, 6, 0),
        grab(15, 30, 8, 14, 6, 1),
    ];
    dive.hitboxes[1].targets = TARGETS_GROUNDED;
    dive.turn_at = Some((13, 25));
    dive.air_blocks = vec![
        AirBlock {
            from: 0,
            to: 12,
            gravity: None,
            max_fall: None,
            friction: None,
            drift: 95,
            accel: 600,
        },
        AirBlock {
            from: 31,
            to: 255,
            gravity: Some(su(103)),
            max_fall: None,
            friction: None,
            drift: 95,
            accel: 100,
        },
    ];
    dive.fast_fall_from = 49;
    dive.ledge_front_from = 19;
    dive.ledge_back_from = 36;
    dive.helpless_after = true;
    dive.fall_drift = 70;
    dive.fall_landing_lag = 24;
    dive.catch_move = Some(DIVE_CATCH as u8);
    dive.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_dive.script"),
    ));
    w.moves[MoveId::UpSpecial as usize] = dive;

    let mut catch = ref_move(
        40,
        0,
        0,
        255,
        &[
            r(2, 3, 0, 10, 12, 50, 361, 50, 100, 0, 0),
            r(2, 3, 0, 10, 12, 50, 361, 50, 100, 0, 0),
            r(17, 17, 9, 10, 10, 130, 50, 85, 62, 0, 1),
        ],
    );
    catch.hitboxes[0].kind = HIT_PUMMEL;
    catch.hitboxes[2].kind = HIT_THROW;
    catch.hold_at = Some((Fx::from_ratio(9, 10), Fx::from_ratio(1, 2)));
    catch.land_continue = true;
    catch.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_dive_catch.script"),
    ));
    w.moves[DIVE_CATCH as usize] = catch;
}

/// Reference units (thousandths) to world units: 1 world unit is 8 reference units.
fn su(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

/// Slots of the specials' follow-ups (the rapid jab holds ext10 to ext12).
const RAPTOR_GROUND_HIT: MoveId = MoveId::Ext3;
const RAPTOR_AIR: MoveId = MoveId::Ext4;
const RAPTOR_AIR_HIT: MoveId = MoveId::Ext5;

/// Side special, a dash into an uppercut (on the ground) or an overhand (in the air): the reference's, ssbwiki.com 13.0.1.
///
/// * On the ground (side special): search boxes on frames 10-25 (radius 4.0 reference units, at 8.8 ahead: on grounded fighters from
///   9.0 to 5.0 up, on airborne ones from 12.0 to 5.0 up [D]); touching anyone, shields included, switches to the uppercut. Missed,
///   FAF 72.
/// * The uppercut (ext3): damage-based armor on frames 1-4 against hits of up to 10% (conflict 2 of the reference data: 1-4 by two
///   sources to one); the clean hit on frame 5, late on 6-7; 10%, 85 degrees, BK 82, KS 30, +3 hitstun, hitlag x1.5, shield damage +4
///   (140 percent [D]). The dash's speed is cut to 0.18 on switching. FAF 28.
/// * In the air (ext4): search on frames 19-35 (radius 4.1, 11.0 ahead, 12.0 to 3.0 up [D]); ledges from frame 26 on either side; landing
///   during it costs 22 frames; at its end (frame 79) the special fall (air speed x0.7, landing lag 22).
/// * The overhand (ext5): clean on frame 5, late on 6; 10%: on airborne fighters a meteor (270, BK 60, KS 70), on grounded ones 60
///   degrees (BK 60, KS 80), the arm's box behind 60 degrees on anyone; hitlag x2, shield damage +2 (120 [D]). On frame 6, whether it
///   hits or not, the fighter is set moving 1.5 back and 2.5 up (reference units a frame); no landing and no ledge before frame 8. FAF
///   45, then free (not helpless). Landing lag 21 (dragdown only: conflict 7).
///
/// The dash's speed and distance are the reference's animation, not published: [E] (see the reference data's "Fitted distances").
fn raptor_boost(w: &mut Weapon) {
    let search = |first: u8, last: u8, x: i32, y: i32, radius: i32| {
        let mut h = row(&r(first, last, x, y, radius, 0, 0, 0, 0, 0, 0));
        h.kind = HIT_SEARCH;
        h
    };
    // The grounded dash.
    let mut dash = ref_move(72, 0, 0, 255, &[]);
    for (y, targets) in [
        (11, TARGETS_GROUNDED),
        (6, TARGETS_GROUNDED),
        (15, TARGETS_AERIAL),
        (6, TARGETS_AERIAL),
    ] {
        let mut h = search(10, 25, 11, y, 5);
        h.targets = targets;
        dash.hitboxes.push(h);
    }
    dash.motion = vec![
        Motion {
            start: 0,
            end: 8,
            vx: su(800),
            vy: Fx::ZERO,
        },
        Motion {
            start: 9,
            end: 24,
            vx: su(2000),
            vy: Fx::ZERO,
        },
        Motion {
            start: 25,
            end: 29,
            vx: su(600),
            vy: Fx::ZERO,
        },
    ];
    dash.on_search = Some(RAPTOR_GROUND_HIT as u8);
    dash.air_version = Some(RAPTOR_AIR as u8);
    w.moves[MoveId::SideSpecial as usize] = dash;

    // The uppercut.
    let mut hit = ref_move(
        28,
        0,
        0,
        255,
        &[
            r(5, 5, 10, 13, 8, 100, 85, 82, 30, 0, 0),
            r(5, 5, 7, 11, 7, 100, 85, 82, 30, 1, 0),
            r(5, 5, 16, 11, 7, 100, 85, 82, 30, 2, 0),
            r(6, 7, 10, 16, 6, 100, 85, 82, 30, 0, 0),
        ],
    );
    set_rows(&mut hit, &[0, 1, 2, 3], |h| {
        h.hitstun_add = 3;
        h.hitlag = 150;
        h.shield_damage = 140;
    });
    // (The clean boxes never clank: the reference's `clang=f`.)
    set_rows(&mut hit, &[0, 1, 2], |h| h.clanks = false);
    hit.armor = Some(Armor {
        start: 0,
        end: 3,
        kind: ARMOR_DAMAGE,
        threshold: Fx::from_int(10),
    });
    hit.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_raptor_hit.script"),
    ));
    w.moves[RAPTOR_GROUND_HIT as usize] = hit;

    // The aerial dash.
    let mut air = ref_move(80, 22, 0, 255, &[]);
    for y in [15, 10, 4] {
        air.hitboxes.push(search(19, 35, 14, y, 5));
    }
    air.motion = vec![
        Motion {
            start: 0,
            end: 8,
            vx: su(500),
            vy: Fx::ZERO,
        },
        Motion {
            start: 9,
            end: 34,
            vx: su(1800),
            vy: -su(300),
        },
    ];
    air.on_search = Some(RAPTOR_AIR_HIT as u8);
    air.helpless_after = true;
    air.fall_drift = 70;
    air.fall_landing_lag = 22;
    air.ledge_front_from = 25;
    air.ledge_back_from = 25;
    w.moves[RAPTOR_AIR as usize] = air;

    // The overhand.
    let mut over = ref_move(
        45,
        21,
        0,
        255,
        &[
            r(5, 5, 12, 10, 10, 100, 270, 60, 70, 0, 0),
            r(5, 5, 12, 10, 10, 100, 60, 60, 80, 1, 0),
            r(5, 5, 5, 13, 3, 100, 60, 60, 80, 2, 0),
            r(6, 6, 12, 10, 10, 100, 270, 60, 70, 0, 0),
            r(6, 6, 12, 10, 10, 100, 60, 60, 80, 1, 0),
            r(6, 6, 7, 10, 5, 100, 60, 60, 80, 2, 0),
            r(6, 6, 4, 1, 5, 100, 60, 60, 80, 3, 0),
        ],
    );
    set_rows(&mut over, &[0, 3], |h| h.targets = TARGETS_AERIAL);
    set_rows(&mut over, &[1, 4], |h| h.targets = TARGETS_GROUNDED);
    set_rows(&mut over, &[0, 1, 2, 3, 4, 5, 6], |h| {
        h.hitlag = 200;
        h.shield_damage = 120;
    });
    over.lands_from = 7;
    over.ledge_front_from = 7;
    over.ledge_back_from = 7;
    over.script = Some(script(
        Kind::Fighter,
        include_str!("scripts/gauntlets_raptor_air_hit.script"),
    ));
    w.moves[RAPTOR_AIR_HIT as usize] = over;
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
