//! The Popgun moveset on the scamp body (class 11), modelled mechanically on a reference character (phase 1: the body and every
//! normal here; the specials in `popgun_peel.rs`, `popgun_peanut.rs` and `popgun_rocket.rs`). Frame data against the reference for every normal, checked in the data and by
//! running the moves; hit properties (damage, angles and angle types, knockback, set knockback and set weight, hitlag and SDI
//! multipliers, trip chances, shield damage, grounded- and aerial-only boxes, no-clank boxes); the jab chain, its held repeat and its
//! window past the FAF; grabs, pummel and throws; landing lag and autocancels; the body's attributes and jump air times; registration;
//! determinism. Reference data and sources: the twelfth moveset's `REFERENCE_DATA.md` (`docs/movesets/popgun/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::{
    MoveId, Weapon, AIM_DOWN, AIM_STRAIGHT, AIM_UP, ANGLE_BACKWARD, ANGLE_BY_POSITION,
    ANGLE_FORWARD, HIT_GRAB, HIT_PUMMEL, HIT_THROW, TARGETS_AERIAL, TARGETS_ALL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const SCAMP: u8 = 11;

/// The scamp (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SCAMP, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SCAMP)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R]; the pummel's hit is on frame 2 here, FAF 8 [D]; the down tilt's FAF 19, the down
/// smash's 51 and the up air's 36 are the sources' majority readings: conflicts 1-3).
const FRAME_DATA: [FrameData; 25] = [
    (MoveId::Jab, &[(3, 3)], 20, 0, 0, 255),
    (MoveId::Jab2, &[(5, 5)], 21, 0, 0, 255),
    (MoveId::Jab3, &[(5, 5)], 35, 0, 0, 255),
    (
        MoveId::FTilt,
        &[(10, 11), (12, 12), (13, 16)],
        37,
        0,
        0,
        255,
    ),
    (MoveId::UTilt, &[(6, 11)], 30, 0, 0, 255),
    (MoveId::DTilt, &[(4, 5)], 19, 0, 0, 255),
    (
        MoveId::DashAttack,
        &[(8, 9), (16, 17), (22, 23)],
        41,
        0,
        0,
        255,
    ),
    (
        MoveId::FSmash,
        &[(12, 12), (13, 13), (21, 23)],
        51,
        0,
        0,
        255,
    ),
    (
        MoveId::USmash,
        &[(5, 5), (6, 8), (12, 14), (15, 15), (19, 23)],
        54,
        0,
        0,
        255,
    ),
    (MoveId::DSmash, &[(7, 8), (12, 13)], 51, 0, 0, 255),
    (MoveId::NAir, &[(8, 20)], 49, 8, 3, 27),
    (MoveId::FAir, &[(6, 9), (10, 16)], 47, 16, 2, 26),
    (MoveId::BAir, &[(5, 8)], 31, 7, 2, 20),
    (MoveId::UAir, &[(4, 7)], 36, 9, 0, 27),
    (MoveId::DAir, &[(15, 16), (15, 15)], 47, 17, 14, 35),
    (MoveId::Grab, &[(7, 8)], 37, 0, 0, 255),
    (MoveId::DashGrab, &[(10, 11)], 45, 0, 0, 255),
    (MoveId::PivotGrab, &[(11, 12)], 40, 0, 0, 255),
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(21, 21)], 32, 0, 0, 255),
    (MoveId::BThrow, &[(18, 18)], 52, 0, 0, 255),
    (MoveId::UThrow, &[(17, 17), (19, 19)], 27, 0, 0, 255),
    (MoveId::DThrow, &[(22, 22)], 32, 0, 0, 255),
    (MoveId::LedgeAttack, &[(23, 25)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(15, 16), (22, 23)], 46, 0, 0, 255),
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
        assert_eq!(
            mv.rehit, None,
            "{id:?}: no rehit (every multi-hit is groups)"
        );
    }
    // Intangibility: the ledge attack through frame 25 (the first grab), the get-up attack through 23.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 25);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 23);
    // Smash charge frames (the reference's charge hold on frames 4, 1 and 2: move frames one less); the reference's plain 1.4x bonus.
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(3));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(0));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(1));
    for id in [MoveId::FSmash, MoveId::USmash, MoveId::DSmash] {
        assert_eq!(w.get(id as u8).charge_bonus, 0, "{id:?}");
    }
    // The jab chain: jab 1 into jab 2 from frame 6 (to 30 past its FAF), jab 2 into jab 3 from 7 (to 30).
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 5));
    assert_eq!(jab1.late_next, Some((MoveId::Jab2 as u8, 29)));
    let jab2 = w.get(MoveId::Jab2 as u8);
    assert_eq!((jab2.next, jab2.next_from), (Some(MoveId::Jab3 as u8), 6));
    assert_eq!(jab2.late_next, Some((MoveId::Jab3 as u8, 29)));
    assert_eq!(w.get(MoveId::Jab3 as u8).next, None);
    assert!(
        w.limbs,
        "bare hands and feet: the limb throwing a hit can be hit"
    );
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (high in the air for an aerial, else standing), with player 1 held where the move's box (a hitbox of
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
        .find(|h| h.start + 1 == first && h.targets != TARGETS_GROUNDED)
        .copied()
        .expect("a box that can hit an airborne fighter");
    let before = sim.fighter(1).percent;
    let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
    for tick in 1..=80u8 {
        let me = sim.state.fighters[0].pos;
        let at = Vec2::new(me.x + hb.x, me.y + hb.y);
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
        (MoveId::Jab, 3u8),
        (MoveId::Jab2, 5),
        (MoveId::Jab3, 5),
        (MoveId::FTilt, 10),
        (MoveId::UTilt, 6),
        (MoveId::DTilt, 4),
        (MoveId::DashAttack, 8),
        (MoveId::FSmash, 12),
        (MoveId::USmash, 5),
        (MoveId::DSmash, 7),
        (MoveId::NAir, 8),
        (MoveId::FAir, 6),
        (MoveId::BAir, 5),
        (MoveId::UAir, 4),
        (MoveId::DAir, 15),
    ] {
        assert_eq!(
            first_hit_frame(id, first),
            Some(first),
            "{id:?} window from {first}"
        );
    }
}

/// Ticks from starting `id` (by `begin_attack`) until the fighter leaves the move, plus the frames that maps to: the reference's FAF.
fn faf_of(id: MoveId, aerial: bool) -> u16 {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    if aerial {
        sim.put_airborne(0, Fx::from_int(-9), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
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
        // (Throws need someone held, the ledge attack a ledge.)
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

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority (the reference's id), damage in tenths, angle, BK, KS) for the boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 47] = [
        (MoveId::Jab, 3, 0, 20, 361, 25, 25),
        (MoveId::Jab, 3, 1, 20, 361, 25, 25),
        (MoveId::Jab, 3, 2, 20, 180, 20, 15),
        (MoveId::Jab2, 5, 0, 15, 361, 20, 25),
        (MoveId::Jab2, 5, 2, 15, 180, 20, 15),
        (MoveId::Jab3, 5, 0, 40, 361, 60, 80),
        (MoveId::Jab3, 5, 2, 40, 361, 60, 80),
        (MoveId::FTilt, 10, 0, 100, 35, 60, 60),
        (MoveId::FTilt, 10, 2, 100, 35, 60, 60),
        (MoveId::FTilt, 10, 3, 100, 35, 60, 60),
        (MoveId::FTilt, 12, 1, 100, 35, 60, 60),
        (MoveId::FTilt, 13, 0, 70, 35, 60, 60),
        (MoveId::UTilt, 6, 0, 60, 92, 28, 150),
        (MoveId::UTilt, 6, 2, 60, 92, 28, 150),
        (MoveId::DTilt, 4, 0, 55, 60, 15, 75),
        (MoveId::DTilt, 4, 1, 55, 50, 15, 75),
        (MoveId::DashAttack, 8, 0, 20, 35, 0, 100),
        (MoveId::DashAttack, 8, 1, 20, 10, 0, 100),
        (MoveId::DashAttack, 8, 2, 20, 43, 0, 100),
        (MoveId::DashAttack, 16, 0, 20, 25, 0, 100),
        (MoveId::DashAttack, 16, 2, 20, 40, 0, 100),
        (MoveId::DashAttack, 22, 0, 30, 83, 80, 90),
        (MoveId::FSmash, 12, 0, 50, 10, 0, 100),
        (MoveId::FSmash, 12, 1, 50, 45, 0, 100),
        (MoveId::FSmash, 12, 2, 50, 80, 0, 100),
        (MoveId::FSmash, 12, 3, 50, 22, 0, 100),
        (MoveId::FSmash, 13, 0, 50, 0, 0, 100),
        (MoveId::FSmash, 21, 0, 110, 46, 40, 116),
        (MoveId::FSmash, 21, 1, 110, 50, 40, 116),
        (MoveId::FSmash, 21, 2, 90, 60, 40, 116),
        (MoveId::USmash, 5, 0, 25, 110, 0, 100),
        (MoveId::USmash, 5, 1, 25, 125, 0, 100),
        (MoveId::USmash, 6, 1, 25, 140, 0, 100),
        (MoveId::USmash, 12, 1, 25, 125, 0, 100),
        (MoveId::USmash, 15, 1, 25, 115, 0, 100),
        (MoveId::USmash, 19, 0, 60, 85, 55, 131),
        (MoveId::DSmash, 7, 0, 120, 33, 46, 77),
        (MoveId::DSmash, 12, 1, 150, 35, 38, 76),
        (MoveId::NAir, 8, 3, 60, 80, 60, 75),
        (MoveId::FAir, 6, 0, 100, 361, 30, 86),
        (MoveId::FAir, 10, 1, 80, 361, 30, 86),
        (MoveId::BAir, 5, 0, 90, 361, 30, 102),
        (MoveId::UAir, 4, 1, 70, 76, 57, 89),
        (MoveId::DAir, 15, 0, 130, 270, 25, 110),
        (MoveId::DAir, 15, 1, 100, 270, 20, 100),
        (MoveId::LedgeAttack, 23, 0, 90, 45, 90, 20),
        (MoveId::GetUpAttack, 22, 0, 70, 48, 80, 48),
    ];
    let sim = duel();
    let w = kit(&sim);
    for (id, first, priority, damage, angle, bk, ks) in table {
        let hb = w
            .get(id as u8)
            .hitboxes
            .iter()
            .find(|h| h.start + 1 == first && h.priority == priority)
            .unwrap_or_else(|| panic!("{id:?} {first} {priority}"));
        assert_eq!(
            (hb.damage, hb.angle, hb.base_knockback, hb.knockback_growth),
            (Fx::from_ratio(damage, 10), angle, bk, ks),
            "{id:?} from {first}, box {priority}"
        );
    }
    let get = |id: MoveId| w.get(id as u8);
    let all =
        |id: MoveId, f: &dyn Fn(&sim_core::moves::Hitbox) -> bool| get(id).hitboxes.iter().all(f);
    // Hitlag and SDI multipliers.
    assert!(all(MoveId::Jab, &|h| h.hitlag == 160));
    assert!(all(MoveId::Jab2, &|h| h.hitlag == 120));
    assert!(all(MoveId::Jab3, &|h| h.hitlag == 200));
    assert!(all(MoveId::UTilt, &|h| h.sdi == 150 && h.hitlag == 100));
    assert!(all(MoveId::DashAttack, &|h| h.hitlag
        == match h.group {
            0 => 50,
            1 => 80,
            _ => 100,
        }));
    assert!(all(MoveId::FSmash, &|h| h.hitlag
        == if h.group == 1 { 120 } else { 100 }));
    assert!(all(MoveId::Pummel, &|h| h.hitlag == 170));
    for id in [
        MoveId::FTilt,
        MoveId::DTilt,
        MoveId::USmash,
        MoveId::DSmash,
        MoveId::NAir,
        MoveId::FAir,
        MoveId::BAir,
        MoveId::UAir,
        MoveId::DAir,
    ] {
        assert!(all(id, &|h| h.hitlag == 100 && h.sdi == 100), "{id:?}");
    }
    // Set knockback and set weight: the dash attack's first two hits, the forward smash's first hit, the up smash's first two hits.
    assert!(all(MoveId::DashAttack, &|h| (h.set_weight, h.fixed_kb)
        == match (h.group, h.angle) {
            (0, 35) | (0, 43) => (true, 60),
            (0, _) | (1, 25) | (1, 10) => (true, 45),
            (1, _) => (true, 70),
            _ => (false, 0),
        }));
    assert!(all(MoveId::FSmash, &|h| (h.set_weight, h.fixed_kb)
        == match (h.group, h.priority) {
            (0, 0) => (true, 20),
            (0, 1) => (true, 35),
            (0, 2) => (true, 25),
            (0, _) => (true, 42),
            _ => (false, 0),
        }));
    assert!(all(MoveId::USmash, &|h| (h.set_weight, h.fixed_kb)
        == match (h.group, h.angle) {
            (0, 110) => (true, 110),
            (0, 125) => (true, 30),
            (0, 95) => (true, 50),
            (0, 130) => (true, 25),
            (0, 140) => (true, 15),
            (1, 105) => (true, 65),
            (1, 125) => (true, 45),
            (1, 115) => (true, 80),
            _ => (false, 0),
        }));
    // Targets: the dash attack's grounded / aerial boxes, the forward smash's root box (aerial) and arm box (grounded), the up smash's
    // first hit (the arm's two).
    assert!(all(MoveId::DashAttack, &|h| h.targets
        == match (h.group, h.priority) {
            (2, _) | (_, 2) => TARGETS_ALL,
            (_, 0) => TARGETS_GROUNDED,
            _ => TARGETS_AERIAL,
        }));
    assert!(all(MoveId::FSmash, &|h| h.targets
        == match (h.group, h.priority) {
            (0, 0) => TARGETS_AERIAL,
            (0, 2) => TARGETS_GROUNDED,
            _ => TARGETS_ALL,
        }));
    assert!(all(MoveId::USmash, &|h| h.targets
        == match (h.group, h.priority) {
            (0, 0) => TARGETS_GROUNDED,
            (0, 1) => TARGETS_AERIAL,
            _ => TARGETS_ALL,
        }));
    // Trips: only the down tilt, 20% on both boxes.
    for (id, ..) in FRAME_DATA {
        assert!(
            all(id, &|h| h.trip == if id == MoveId::DTilt { 20 } else { 0 }),
            "{id:?}"
        );
    }
    // Angle types: type 3 (forward) on the jabs, the forward tilt, the dash attack, the forward smash, the up smash's last hit, the
    // forward air, the pummel and the ledge attack; type 4 (backward) on the back air; type 0 (by position) on the rest.
    for id in [
        MoveId::Jab,
        MoveId::Jab2,
        MoveId::Jab3,
        MoveId::FTilt,
        MoveId::DashAttack,
        MoveId::FSmash,
        MoveId::FAir,
        MoveId::Pummel,
        MoveId::LedgeAttack,
    ] {
        assert!(all(id, &|h| h.angle_type == ANGLE_FORWARD), "{id:?}");
    }
    assert!(all(MoveId::BAir, &|h| h.angle_type == ANGLE_BACKWARD));
    assert!(all(MoveId::USmash, &|h| h.angle_type
        == if h.group == 2 {
            ANGLE_FORWARD
        } else {
            ANGLE_BY_POSITION
        }));
    for id in [
        MoveId::UTilt,
        MoveId::DTilt,
        MoveId::DSmash,
        MoveId::NAir,
        MoveId::UAir,
        MoveId::DAir,
        MoveId::GetUpAttack,
    ] {
        assert!(all(id, &|h| h.angle_type == ANGLE_BY_POSITION), "{id:?}");
    }
    // Shield damage: the ledge attack's +1 (111), the get-up attack's +8 (214) [D].
    assert!(all(MoveId::LedgeAttack, &|h| h.shield_damage == 111));
    assert!(all(MoveId::GetUpAttack, &|h| h.shield_damage == 214));
    // No clank (`clang=f rebound=f`) on the dash attack, the pummel, the up throw's hit, the ledge attack and the get-up attack.
    for (id, ..) in FRAME_DATA {
        let never = matches!(
            id,
            MoveId::DashAttack | MoveId::Pummel | MoveId::LedgeAttack | MoveId::GetUpAttack
        );
        if id == MoveId::UThrow {
            continue;
        }
        assert!(
            all(id, &|h| h.clanks == !never && h.rebounds == !never),
            "{id:?}"
        );
    }
    assert!(all(MoveId::UThrow, &|h| h.clanks == (h.kind == HIT_THROW)));
}

#[test]
fn the_forward_tilt_aims_its_hands_and_keeps_its_root_boxes() {
    let sim = duel();
    let ftilt = kit(&sim).get(MoveId::FTilt as u8);
    for aim in [AIM_UP, AIM_STRAIGHT, AIM_DOWN] {
        // Each aim has both hands on all three windows.
        assert_eq!(
            ftilt.hitboxes.iter().filter(|h| h.aim == aim).count(),
            6,
            "aim {aim}"
        );
    }
    let up = ftilt.hitboxes.iter().find(|h| h.aim == AIM_UP).unwrap();
    let down = ftilt.hitboxes.iter().find(|h| h.aim == AIM_DOWN).unwrap();
    assert!(up.y > down.y, "raised and lowered");
    // The root boxes come out in every aim (the reference's `top` boxes).
    assert_eq!(ftilt.hitboxes.iter().filter(|h| h.priority >= 2).count(), 2);
}

// ---- The jab ---------------------------------------------------------------------------------------------------------------------

#[test]
fn pressing_attack_goes_jab_jab_kick_and_holding_repeats_jab_1_from_frame_10() {
    // Pressed on the continuable frames: jab 1, jab 2 on its frame 6, jab 3 on jab 2's frame 7.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Jab as u8);
    sim.ticks(4, inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(
        (sim.f().move_id, sim.f().state_frame),
        (MoveId::Jab2 as u8, 0),
        "jab 2 on jab 1's frame 6"
    );
    sim.ticks(5, inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(
        (sim.f().move_id, sim.f().state_frame),
        (MoveId::Jab3 as u8, 0),
        "jab 3 on jab 2's frame 7"
    );
    // Held from the start: jab 1 again on its frame 10, and again.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    let mut starts = Vec::new();
    for t in 0..30u32 {
        sim.tick(inp(0, 0, ATTACK));
        if sim.f().state == S::Attack && sim.f().state_frame == 0 {
            assert_eq!(sim.f().move_id, MoveId::Jab as u8);
            starts.push(t);
        }
    }
    assert_eq!(&starts[..3], &[0, 9, 18], "a new jab 1 every 9 frames");
}

#[test]
fn jab_1_pulls_a_fighter_at_its_tip_in() {
    // Only the far box (180 degrees, type 3) reaches a fighter at the tip: it is drawn toward the scamp, not sent away.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    let tip = kit(&sim)
        .get(MoveId::Jab as u8)
        .hitboxes
        .iter()
        .find(|h| h.priority == 2)
        .unwrap()
        .x;
    let ecb = sim.content.fighters[0].ecb_half_width;
    // (The duelist's guard circle reaches 0.8 of its half width toward the scamp, radius half of it: just inside the far box.)
    sim.stand(1, tip + ecb + fx(3, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
    let x0 = sim.fighter(1).pos.x;
    let before = sim.fighter(1).percent;
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.fighter(1).percent > before, "hit");
    assert!(
        sim.fighter(1).pos.x <= x0,
        "drawn in (or held), never sent away"
    );
}

// ---- Grabs and throws --------------------------------------------------------------------------------------------------------

fn grab_now(sim: &mut Sim) -> u8 {
    sim.tick(inp(0, 0, GRAB));
    for t in 2..=40u8 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbing {
            return t;
        }
    }
    panic!("the grab never connected");
}

#[test]
fn the_grab_catches_on_frame_7() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(13, 10), -1);
    assert_eq!(grab_now(&mut sim), 7);
    for id in [MoveId::Grab, MoveId::DashGrab, MoveId::PivotGrab] {
        assert!(kit(&sim)
            .get(id as u8)
            .hitboxes
            .iter()
            .all(|h| h.kind == HIT_GRAB));
    }
}

#[test]
fn the_throws_release_on_their_frames_for_their_damage() {
    // (stick, the frames the held fighter's percent rises on, total damage in tenths). The up throw hits the held fighter for 1% on
    // frame 17 and throws on 19.
    for (stick, frames, damage) in [
        ((127i8, 0i8), vec![21u8], 90),
        ((-127, 0), vec![18], 120),
        ((0, 127), vec![17, 19], 50),
        ((0, -127), vec![22], 70),
    ] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(13, 10), -1);
        grab_now(&mut sim);
        let before = sim.fighter(1).percent;
        sim.tick(inp(stick.0, stick.1, 0));
        let mut rises = Vec::new();
        let mut last = before;
        for _ in 0..120 {
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).percent > last {
                last = sim.fighter(1).percent;
                rises.push(sim.f().state_frame as u8 + 1);
            }
        }
        assert_eq!(rises, frames, "throw {stick:?}");
        assert_eq!(
            sim.fighter(1).percent - before,
            dealt(&sim, damage),
            "throw {stick:?}"
        );
        if stick.0 < 0 {
            assert!(sim.fighter(1).pos.x < Fx::ZERO, "thrown behind");
        }
    }
    let sim = duel();
    let w = kit(&sim);
    for (id, angle, bkb, kbg, lag) in [
        (MoveId::FThrow, 46i16, 69i16, 65i16, 0u16),
        (MoveId::BThrow, 42, 60, 55, 100),
        (MoveId::UThrow, 87, 70, 72, 0),
        (MoveId::DThrow, 106, 85, 59, 0),
    ] {
        let throw = w
            .get(id as u8)
            .hitboxes
            .iter()
            .find(|h| h.kind == HIT_THROW)
            .unwrap();
        assert_eq!(
            (
                throw.angle,
                throw.base_knockback,
                throw.knockback_growth,
                throw.hitlag
            ),
            (angle, bkb, kbg, lag),
            "{id:?}"
        );
    }
    assert!(w
        .get(MoveId::UThrow as u8)
        .hitboxes
        .iter()
        .any(|h| h.kind == HIT_PUMMEL));
}

#[test]
fn the_pummel_does_1_percent() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(13, 10), -1);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Pummel as u8);
    sim.ticks(4, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 10));
}

// ---- Landing ---------------------------------------------------------------------------------------------------------------------

fn land_during(id: MoveId, frame_before_tick: u16) -> (S, u8, u8) {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-9), fx(2, 100), Fx::ZERO, fx(-1, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = id as u8;
    f.state_frame = frame_before_tick;
    f.hit_mask = 0;
    sim.tick(inp(0, 0, 0));
    (sim.f().state, sim.f().lag, sim.f().move_id)
}

#[test]
fn aerials_cost_their_landing_lag_and_autocancel_in_their_windows() {
    let plain = duel().content.fighters[usize::from(SCAMP)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window; the tick adds
    // one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 15u16, 8u8, 26u16, Some(1u16)),
        (MoveId::FAir, 15, 16, 25, Some(0)),
        (MoveId::BAir, 10, 7, 19, Some(0)),
        (MoveId::UAir, 10, 9, 26, None),
        (MoveId::DAir, 20, 17, 34, Some(12)),
    ] {
        assert_eq!(
            land_during(id, mid),
            (S::Landing, lag, id as u8),
            "{id:?} mid-move"
        );
        assert_eq!(land_during(id, late).1, plain, "{id:?} late");
        if let Some(early) = early {
            assert_eq!(land_during(id, early).1, plain, "{id:?} early");
        }
    }
    // The up air has no early window: landing on its frame 1 costs its 9 frames.
    assert_eq!(land_during(MoveId::UAir, 0).1, 9);
}

// ---- Body ----------------------------------------------------------------------------------------------------------------------

/// Frames in the air for a short or full hop, fast falling or not.
fn air_time(full: bool, ff: bool) -> u32 {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim.tick(inp(0, 0, JUMP));
    while sim.f().grounded() {
        sim.tick(inp(0, 0, if full { JUMP } else { 0 }));
    }
    let mut n = 0u32;
    let mut flicked = false;
    while !sim.f().grounded() && n < 200 {
        let down = ff && !flicked && sim.f().vel.y <= Fx::ZERO;
        flicked |= down;
        let held = if full { JUMP } else { 0 };
        sim.tick(inp(0, if down { -127 } else { 0 }, held));
        n += 1;
    }
    n
}

#[test]
fn the_scamps_jumps_have_the_references_air_times() {
    // Short hop 36, full hop 51, short hop fast fall 26, full hop fast fall 36 (ultimateframedata.com); within one frame.
    for (full, ff, reference) in [
        (false, false, 36u32),
        (true, false, 51),
        (false, true, 26),
        (true, true, 36),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_scamp_is_the_references_quick_middleweight() {
    let p = duel().content.fighters[usize::from(SCAMP)];
    assert_eq!(p, FighterParams::scamp());
    let su = FighterParams::su;
    assert_eq!(p.weight, Fx::from_int(90));
    assert_eq!(p.gravity, su(125));
    assert_eq!((p.max_fall_speed, p.fast_fall_speed), (su(1750), su(2800)));
    assert_eq!(
        (p.air_speed, p.air_accel, p.air_accel_stick, p.air_friction),
        (su(924), Fx::ZERO, su(50), su(8))
    );
    assert_eq!(p.air_jump_drift, 90);
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (45, 63, 75, 97)
    );
    assert_eq!(
        (
            p.air_dodge_intangible_start,
            p.air_dodge_intangible_end,
            p.air_dodge_dir_intangible_end
        ),
        (2, 27, 19)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (29, 34, 24)
    );
    assert_eq!(
        (
            p.roll_intangible_start,
            p.roll_intangible_end,
            p.spot_intangible_start,
            p.spot_intangible_end
        ),
        (4, 14, 3, 16)
    );
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (1, su(1300), su(2300))
    );
    let t = p.item_throws;
    assert_eq!(
        (t.forward, t.back, t.up, t.down, t.dash, t.drop),
        (7, 9, 11, 6, 4, 5)
    );
    assert_eq!(p.air_jumps, 1);
}

#[test]
fn the_popgun_is_registered_as_class_11() {
    let c = Content::placeholder();
    assert_eq!(c.names.fighters[11], "scamp");
    assert_eq!(c.names.weapons[11], "popgun");
    assert_eq!(c.fighters[11], FighterParams::scamp());
    assert_eq!(c.weapons[11], sim_core::moves::popgun());
    assert_eq!(c.fighters[11].weapon, 11);
    // Every special is the Popgun's own (the side special, the flip, since phase 2: sim 119).
    for (id, own) in [
        (MoveId::NSpecial, true),
        (MoveId::SideSpecial, true),
        (MoveId::UpSpecial, true),
        (MoveId::DownSpecial, true),
    ] {
        assert_eq!(
            c.weapons[11].moves[id as usize] != c.weapons[1].moves[id as usize],
            own,
            "{id:?}"
        );
    }
}

#[test]
fn random_play_with_the_scamp_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [SCAMP, 0, SCAMP, 10];
    for seed in 0..6u64 {
        let inputs = sim_core::fuzz::random_inputs(&mut Rng::new(seed), 900);
        let play = |frames: &[[sim_core::Input; MAX_FIGHTERS]]| {
            let mut s = GameState::new(&content, seed, chars);
            for i in frames {
                step(&mut s, &content, i);
            }
            s
        };
        let straight = play(&inputs);
        assert_eq!(straight.checksum(), play(&inputs).checksum());
        let mut s = GameState::new(&content, seed, chars);
        for i in &inputs[..400] {
            step(&mut s, &content, i);
        }
        let mut r = s;
        for i in &inputs[400..] {
            step(&mut r, &content, i);
        }
        assert_eq!(r.checksum(), straight.checksum(), "seed {seed}");
    }
}
