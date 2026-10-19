//! The Nodachi moveset on the exile body (class 13), modelled mechanically on a reference character (phase 1): the body and every
//! normal. Frame data against the reference for every normal, checked in the data and by running the moves; hit properties; the
//! body's numbers and jumps. The wing is `nodachi_wing.rs`, the specials `nodachi_specials.rs`. Reference data and sources:
//! `docs/movesets/nodachi/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::JUMP;
use sim_core::moves::{
    Hitbox, MoveId, Weapon, AIM_DOWN, AIM_STRAIGHT, AIM_UP, ANGLE_BACKWARD, ANGLE_BY_POSITION,
    ANGLE_FORWARD, HIT_GRAB, HIT_PUMMEL, HIT_THROW, PART_HIGH, PART_MIDDLE, TARGETS_AERIAL,
    TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{FighterParams, Fx, Vec2};

const EXILE: u8 = 13;

/// The exile (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([EXILE, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(EXILE)].weapon)]
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R]; the pummel's hit on frame 2 here, FAF 8 [D]; the up tilt's FAF 45 is
/// ssbwiki.com's and kuroganehammer.com's: conflict 1).
const FRAME_DATA: [FrameData; 25] = [
    (MoveId::Jab, &[(5, 7)], 29, 0, 0, 255),
    (MoveId::Jab2, &[(5, 7)], 34, 0, 0, 255),
    (MoveId::Jab3, &[(7, 9)], 36, 0, 0, 255),
    (MoveId::FTilt, &[(14, 16)], 38, 0, 0, 255),
    (
        MoveId::UTilt,
        &[(10, 10), (13, 15), (16, 25)],
        45,
        0,
        0,
        255,
    ),
    (MoveId::DTilt, &[(9, 15), (16, 22)], 43, 0, 0, 255),
    (MoveId::DashAttack, &[(14, 15), (16, 19)], 47, 0, 0, 255),
    (MoveId::FSmash, &[(24, 25)], 68, 0, 0, 255),
    (
        MoveId::USmash,
        &[(23, 23), (24, 24), (25, 25), (26, 26), (27, 28)],
        64,
        0,
        0,
        255,
    ),
    (MoveId::DSmash, &[(21, 23)], 54, 0, 0, 255),
    (MoveId::NAir, &[(9, 10)], 34, 9, 1, 30),
    (MoveId::FAir, &[(13, 14), (15, 18)], 42, 14, 2, 39),
    (MoveId::BAir, &[(15, 16)], 47, 16, 4, 46),
    (
        MoveId::UAir,
        &[(16, 16), (17, 17), (18, 18), (19, 21)],
        54,
        21,
        4,
        52,
    ),
    (MoveId::DAir, &[(19, 23), (24, 39)], 61, 26, 4, 55),
    (MoveId::Grab, &[(7, 8)], 37, 0, 0, 255),
    (MoveId::DashGrab, &[(10, 11)], 45, 0, 0, 255),
    (MoveId::PivotGrab, &[(11, 12)], 40, 0, 0, 255),
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(14, 14), (15, 15)], 35, 0, 0, 255),
    (MoveId::BThrow, &[(18, 18), (19, 19)], 32, 0, 0, 255),
    (MoveId::UThrow, &[(13, 13), (14, 14)], 33, 0, 0, 255),
    (MoveId::DThrow, &[(40, 40)], 48, 0, 0, 255),
    (MoveId::LedgeAttack, &[(24, 26)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(19, 20), (24, 25)], 46, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = kit(&sim);
    for (id, windows, faf, landing, ac_before, ac_from) in FRAME_DATA {
        let mv = w.get(id as u8);
        for hb in &mv.hitboxes {
            assert!(
                windows
                    .iter()
                    .any(|&(a, b)| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: hitbox {}..{} is in no window of {windows:?}",
                hb.start + 1,
                hb.end + 1
            );
        }
        for &(a, b) in windows {
            assert!(
                mv.hitboxes
                    .iter()
                    .any(|hb| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: window {a}..{b} has no hitbox"
            );
        }
        assert_eq!(u16::from(mv.total_frames) + 2, u16::from(faf), "{id:?} FAF");
        assert_eq!(mv.landing_lag, landing, "{id:?} landing lag");
        assert_eq!(mv.autocancel_before, ac_before, "{id:?} autocancel before");
        let from = if ac_from == 255 { 255 } else { ac_from - 1 };
        assert_eq!(mv.autocancel_after, from, "{id:?} autocancel from");
    }
    // Intangibility: the ledge attack through frame 26 (the first grab), the get-up attack through 25; the up tilt's upper body on
    // 13-15.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 26);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 25);
    let ut = &w.get(MoveId::UTilt as u8).part_intangible;
    assert_eq!((ut.len(), ut[0].start, ut[0].end), (1, 12, 14));
    assert!(ut[0].parts & PART_HIGH != 0 && ut[0].parts & PART_MIDDLE != 0);
    // Smash charge frames (the reference's "charges between 8-9", 12-13, 8-9: move frames 8, 12, 8).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(8));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(12));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(8));
    // The jab chain: continuable from frame 10 in both, no window past the FAF; the held version by script.
    for (jab, next) in [(MoveId::Jab, MoveId::Jab2), (MoveId::Jab2, MoveId::Jab3)] {
        let m = w.get(jab as u8);
        assert_eq!(
            (m.next, m.next_from, m.late_next),
            (Some(next as u8), 9, None)
        );
        assert!(m.script.is_some());
    }
    assert_eq!(w.get(MoveId::Jab3 as u8).next, None);
    // The down air lands into its landing hit (the reference's 26 frames of landing lag: FAF 27).
    let dair = w.get(MoveId::DAir as u8);
    let landing = dair.landing_move.expect("a landing hit");
    let lm = w.get(landing);
    assert_eq!(lm.total_frames + 2, 27);
    assert!(lm.hitboxes.iter().all(|h| h.start == 0 && h.end <= 1));
    assert!(w.limbs);
}

/// Starts `id` for player 0 (high in the air for an aerial, else standing) with player 1 held where the move's box (a hitbox of
/// window `first` that can hit an airborne fighter) is, and runs until player 1 is hit. Returns the reference frame of the hit.
fn first_hit_frame(id: MoveId, first: u8) -> Option<u8> {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let hb = kit(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| h.start + 1 == first && h.targets != TARGETS_GROUNDED && h.kind == 0)
        .copied()
        .expect("a box that can hit an airborne fighter");
    let before = sim.fighter(1).percent;
    let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
    for tick in 1..=80u8 {
        let me = sim.state.fighters[0].pos;
        let x = me.x + hb.x.mul_int(i32::from(sim.f().facing));
        let at = Vec2::new(x, me.y + hb.y);
        sim.put_airborne(1, at.x, (at.y - middle).max(fx(1, 10)), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            return Some(tick + 1);
        }
        if sim.f().state != S::Attack && tick > 3 {
            return None;
        }
    }
    None
}

#[test]
fn every_attack_first_hits_on_its_reference_frame() {
    for (id, first) in [
        (MoveId::Jab, 5u8),
        (MoveId::Jab2, 5),
        (MoveId::Jab3, 7),
        (MoveId::FTilt, 14),
        (MoveId::UTilt, 13),
        (MoveId::DTilt, 9),
        (MoveId::DashAttack, 14),
        (MoveId::FSmash, 24),
        (MoveId::USmash, 23),
        (MoveId::DSmash, 21),
        (MoveId::NAir, 9),
        (MoveId::FAir, 13),
        (MoveId::BAir, 15),
        (MoveId::UAir, 16),
        (MoveId::DAir, 19),
    ] {
        assert_eq!(
            first_hit_frame(id, first),
            Some(first),
            "{id:?} from {first}"
        );
    }
}

/// Ticks from starting `id` (by `begin_attack`) until the fighter leaves the move, plus the frames that maps to: the reference's FAF.
fn faf_of(id: MoveId, aerial: bool) -> u16 {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    if aerial {
        sim.put_airborne(0, Fx::from_int(-9), Fx::from_int(80), Fx::ZERO, Fx::ZERO);
    } else {
        sim.stand(0, Fx::from_int(-9), 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    for tick in 1..=120u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            return tick + 2;
        }
    }
    panic!("{id:?} never finished");
}

#[test]
fn every_attack_returns_control_on_its_reference_faf() {
    for (id, _, faf, _, _, _) in FRAME_DATA {
        if matches!(
            id,
            MoveId::Pummel
                | MoveId::FThrow
                | MoveId::BThrow
                | MoveId::UThrow
                | MoveId::DThrow
                | MoveId::LedgeAttack
        ) {
            continue;
        }
        assert_eq!(faf_of(id, id.is_aerial()), u16::from(faf), "{id:?}");
    }
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

/// The first box of `id` out on reference frame `first` with priority `priority`.
fn find(w: &Weapon, id: MoveId, first: u8, priority: u8) -> Hitbox {
    *w.get(id as u8)
        .hitboxes
        .iter()
        .find(|h| h.start + 1 == first && h.priority == priority)
        .unwrap_or_else(|| panic!("{id:?} {first} {priority}"))
}

/// A hit's expected numbers: the move, its first frame, priority, damage in tenths, angle, BK, KS, hitlag percent.
type HitRow = (MoveId, u8, u8, i32, i16, i16, i16, u16);

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, first frame, priority (the reference's id), damage in tenths, angle, BK, KS, hitlag percent)
    let table: [HitRow; 37] = [
        (MoveId::Jab, 5, 0, 25, 361, 25, 10, 140),
        (MoveId::Jab, 5, 2, 25, 180, 25, 15, 140),
        (MoveId::Jab2, 5, 0, 20, 361, 35, 20, 130),
        (MoveId::Jab2, 5, 2, 20, 361, 25, 15, 130),
        (MoveId::Jab3, 7, 1, 40, 361, 70, 80, 150),
        (MoveId::FTilt, 14, 0, 75, 361, 41, 102, 60),
        (MoveId::FTilt, 14, 1, 120, 361, 41, 98, 120),
        (MoveId::FTilt, 14, 2, 90, 361, 41, 102, 60),
        (MoveId::UTilt, 10, 0, 10, 90, 0, 100, 40),
        (MoveId::UTilt, 13, 2, 140, 90, 74, 80, 160),
        (MoveId::UTilt, 13, 1, 125, 91, 69, 75, 120),
        (MoveId::UTilt, 13, 0, 110, 91, 69, 75, 70),
        (MoveId::UTilt, 16, 2, 100, 96, 60, 73, 60),
        (MoveId::DTilt, 9, 0, 80, 85, 84, 60, 100),
        (MoveId::DTilt, 16, 1, 60, 75, 79, 60, 100),
        (MoveId::DashAttack, 14, 1, 130, 53, 67, 82, 130),
        (MoveId::DashAttack, 16, 1, 70, 361, 66, 82, 100),
        (MoveId::FSmash, 24, 0, 135, 361, 54, 81, 70),
        (MoveId::FSmash, 24, 2, 200, 361, 57, 78, 130),
        (MoveId::FSmash, 24, 4, 150, 361, 57, 85, 90),
        (MoveId::USmash, 23, 0, 155, 92, 67, 79, 140),
        (MoveId::USmash, 24, 0, 170, 97, 67, 79, 140),
        (MoveId::USmash, 25, 2, 120, 83, 62, 79, 80),
        (MoveId::USmash, 26, 3, 170, 97, 67, 79, 140),
        (MoveId::DSmash, 21, 0, 165, 63, 85, 68, 130),
        (MoveId::DSmash, 21, 4, 130, 48, 80, 68, 120),
        (MoveId::NAir, 9, 0, 85, 60, 64, 72, 100),
        (MoveId::FAir, 13, 2, 110, 361, 39, 86, 130),
        (MoveId::FAir, 15, 0, 55, 361, 39, 86, 90),
        (MoveId::BAir, 15, 1, 145, 42, 47, 91, 130),
        (MoveId::BAir, 15, 2, 115, 42, 39, 104, 80),
        (MoveId::UAir, 16, 1, 110, 106, 55, 96, 110),
        (MoveId::UAir, 19, 3, 85, 74, 55, 96, 80),
        (MoveId::DAir, 19, 1, 150, 270, 20, 90, 130),
        (MoveId::DAir, 24, 0, 100, 48, 58, 73, 80),
        (MoveId::LedgeAttack, 24, 0, 90, 45, 90, 20, 100),
        (MoveId::GetUpAttack, 19, 0, 70, 48, 80, 48, 100),
    ];
    let sim = duel();
    let w = kit(&sim);
    for (id, first, priority, damage, angle, bk, ks, lag) in table {
        let hb = find(w, id, first, priority);
        assert_eq!(
            (
                hb.damage,
                hb.angle,
                hb.base_knockback,
                hb.knockback_growth,
                hb.hitlag
            ),
            (Fx::from_ratio(damage, 10), angle, bk, ks, lag),
            "{id:?} from {first}, box {priority}"
        );
    }
    // Hitstun modifiers: jab 1 +2, jab 2 +3.
    assert_eq!(find(w, MoveId::Jab, 5, 0).hitstun_add, 2);
    assert_eq!(find(w, MoveId::Jab2, 5, 0).hitstun_add, 3);
    // Angle types: the jab and smashes forward, the back air and the up smash's late hits backward, the neutral air by position.
    assert_eq!(find(w, MoveId::Jab, 5, 0).angle_type, ANGLE_FORWARD);
    assert_eq!(find(w, MoveId::BAir, 15, 0).angle_type, ANGLE_BACKWARD);
    assert_eq!(find(w, MoveId::USmash, 25, 0).angle_type, ANGLE_BACKWARD);
    assert_eq!(find(w, MoveId::USmash, 24, 0).angle_type, ANGLE_FORWARD);
    assert_eq!(find(w, MoveId::NAir, 9, 0).angle_type, ANGLE_BY_POSITION);
    // The forward tilt is aimed by the stick: every box belongs to one aim.
    let ft = &w.get(MoveId::FTilt as u8).hitboxes;
    for aim in [AIM_UP, AIM_STRAIGHT, AIM_DOWN] {
        assert!(ft.iter().any(|h| h.aim == aim), "aim {aim}");
    }
    // The up tilt's first hit: grounded targets only, a position vector to (2, 45) of the reference's units over 5 frames; never clanks.
    let lift = find(w, MoveId::UTilt, 10, 0);
    assert_eq!(
        (
            lift.targets,
            lift.posvec_x,
            lift.posvec_y,
            lift.posvec_frames
        ),
        (TARGETS_GROUNDED, fx(2, 8), fx(45, 8), 5)
    );
    assert!(!lift.clanks);
    // The up tilt's tip reaches 46.5 to 48 of the reference's units up (x 1.25: 5.8 to 6.0 world units).
    let tip = find(w, MoveId::UTilt, 13, 2);
    assert!(tip.y >= fx(58, 10) && tip.y <= fx(61, 10), "{:?}", tip.y);
    // The down air: the tip spikes aerial targets (270), sends grounded ones at 58; by position.
    let spike = find(w, MoveId::DAir, 19, 1);
    assert_eq!(
        (spike.targets, spike.angle_type),
        (TARGETS_AERIAL, ANGLE_BY_POSITION)
    );
    let ground = find(w, MoveId::DAir, 19, 0);
    assert_eq!((ground.targets, ground.angle), (TARGETS_GROUNDED, 58));
    // The down smash's eruption: shield damage +15 (the reference's additive 15 on 16.5: 191%), none rebound, the outer part never clanks.
    let erupt = find(w, MoveId::DSmash, 21, 0);
    assert_eq!((erupt.shield_damage, erupt.rebounds), (191, false));
    assert!(!find(w, MoveId::DSmash, 21, 1).clanks);
    // The pummel and throws.
    let pummel = w.get(MoveId::Pummel as u8).hitboxes[0];
    assert_eq!(
        (pummel.kind, pummel.fixed_kb, pummel.hitlag),
        (HIT_PUMMEL, 30, 220)
    );
    for (id, damage, angle, bk, ks) in [
        (MoveId::FThrow, 30, 28, 70, 68),
        (MoveId::BThrow, 35, 58, 60, 105),
        (MoveId::UThrow, 30, 98, 70, 110),
        (MoveId::DThrow, 40, 56, 80, 94),
    ] {
        let t = *w
            .get(id as u8)
            .hitboxes
            .iter()
            .find(|h| h.kind == HIT_THROW)
            .expect("a throw");
        assert_eq!(
            (
                t.damage,
                t.angle,
                t.base_knockback,
                t.knockback_growth,
                t.hitlag
            ),
            (Fx::from_ratio(damage, 10), angle, bk, ks, 0),
            "{id:?}"
        );
    }
    assert_eq!(
        w.get(MoveId::BThrow as u8)
            .hitboxes
            .iter()
            .find(|h| h.kind == HIT_THROW)
            .map(|h| h.angle_type),
        Some(ANGLE_BACKWARD)
    );
    assert!(w
        .get(MoveId::Grab as u8)
        .hitboxes
        .iter()
        .all(|h| h.kind == HIT_GRAB));
}

#[test]
fn the_long_blade_reaches_further_than_the_longsword() {
    // The forward smash's tip reaches 21.5 of the reference's units down the blade from a hilt 8 ahead: about 37 world tenths; the
    // forward tilt's tip box 31 of the reference's units ahead (38.75 tenths). The longsword's forward smash reaches less.
    let sim = duel();
    let w = kit(&sim);
    let reach = |w: &Weapon, id: MoveId| {
        w.get(id as u8)
            .hitboxes
            .iter()
            .map(|h| h.x + h.radius)
            .max()
            .unwrap_or(Fx::ZERO)
    };
    let sword = &sim.content.weapons[0];
    assert!(reach(w, MoveId::FSmash) > reach(sword, MoveId::FSmash));
    assert!(
        reach(w, MoveId::FTilt) >= fx(38, 10),
        "{:?}",
        reach(w, MoveId::FTilt)
    );
}

#[test]
fn the_down_air_stalls_then_plunges_at_4_2() {
    let mut sim = duel();
    sim.put_airborne(
        0,
        Fx::ZERO,
        Fx::from_int(60),
        Fx::from_int(1),
        Fx::from_int(-1),
    );
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    // Frames 1-18: no sideways speed, rising at 1 of the reference's units (0.125).
    for _ in 1..=17 {
        sim.tick(inp(127, 0, 0));
        assert_eq!(sim.f().vel, Vec2::new(Fx::ZERO, fx(1, 8)));
    }
    // From 19: falling at 4.2 (0.525).
    for _ in 18..=30 {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(sim.f().vel.y, -fx(42, 80));
    assert_eq!(sim.f().vel.x, Fx::ZERO);
}

#[test]
fn holding_attack_carries_the_jab_on_after_a_hit_from_10_and_without_one_from_19() {
    // Without a hit, held: jab 2 begins on frame 19 (the reference's "held without hitting").
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    // (Pressed on tick 1, which starts jab 1, then held.)
    let mut tick = 0u8;
    let mut jab2_on = None;
    while tick < 40 {
        sim.tick(inp(0, 0, sim_core::input::buttons::ATTACK));
        tick += 1;
        if sim.f().state == S::Attack && sim.f().move_id == MoveId::Jab2 as u8 {
            jab2_on = Some(tick);
            break;
        }
    }
    assert_eq!(jab2_on, Some(19), "jab 2 starts on frame 19");
    // After a hit, held: jab 2 from frame 10 (hitlag holds the move frame, so it comes later in ticks, but on jab 1's frame 10).
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(14, 10), -1);
    let mut last_jab1 = 0u16;
    for _ in 0..40 {
        sim.tick(inp(0, 0, sim_core::input::buttons::ATTACK));
        if sim.f().move_id == MoveId::Jab2 as u8 {
            break;
        }
        last_jab1 = sim.f().state_frame;
    }
    assert!(sim.fighter(1).percent > Fx::ZERO, "jab 1 hit");
    assert_eq!(sim.f().move_id, MoveId::Jab2 as u8);
    assert_eq!(
        last_jab1, 8,
        "jab 1's last frame is its 9th: jab 2 replaces its 10th"
    );
}

// ---- The body ----------------------------------------------------------------------------------------------------------------

fn air_time(full: bool) -> u32 {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim.tick(inp(0, 0, JUMP));
    while sim.f().grounded() {
        sim.tick(inp(0, 0, if full { JUMP } else { 0 }));
    }
    let mut n = 0u32;
    while !sim.f().grounded() && n < 200 {
        sim.tick(inp(0, 0, if full { JUMP } else { 0 }));
        n += 1;
    }
    n
}

#[test]
fn the_exiles_jumps_have_the_references_air_times() {
    // Short hop 27, full hop 45 (kuroganehammer.com); within one frame.
    for (full, reference) in [(false, 27u32), (true, 45)] {
        let t = air_time(full);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_exile_is_the_references_light_fast_faller_and_is_class_13() {
    let sim = duel();
    let p = sim.content.fighters[usize::from(EXILE)];
    assert_eq!(p, FighterParams::exile());
    let su = FighterParams::su;
    let gu = FighterParams::gu;
    assert_eq!(p.weight, Fx::from_int(79));
    assert_eq!(p.gravity, su(108));
    assert_eq!((p.max_fall_speed, p.fast_fall_speed), (su(1840), su(2944)));
    assert_eq!(
        (p.air_speed, p.air_accel, p.air_accel_stick, p.air_friction),
        (su(950), su(10), su(70), su(8))
    );
    assert_eq!(
        (
            p.walk_speed,
            p.dash_speed,
            p.run_speed,
            p.dash_attack_friction
        ),
        (gu(900), gu(1920), gu(1860), gu(110))
    );
    assert_eq!(
        (
            p.walk_accel,
            p.walk_accel_stick,
            p.run_accel,
            p.run_accel_stick
        ),
        (gu(20), gu(80), gu(30), gu(90))
    );
    assert_eq!(
        (
            p.dash_frames,
            p.heavy_landing_lag,
            p.air_jump_drift,
            p.air_jumps
        ),
        (10, 2, 100, 1)
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (44, 61, 74, 97)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (29, 34, 25)
    );
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (0, su(1300), su(2400))
    );
    assert_eq!(p.wing, Some(FighterParams::exile_wing()));
    assert_eq!(sim.content.names.fighters[usize::from(EXILE)], "exile");
    assert_eq!(sim.content.names.weapons[usize::from(p.weapon)], "nodachi");
    assert_eq!(sim_content_classes(), 14);
}

fn sim_content_classes() -> u8 {
    // (sim-core's tests cannot see sim-content: the class count is the content's weapon count, which recipes index.)
    sim_core::Content::placeholder().weapons.len() as u8
}
