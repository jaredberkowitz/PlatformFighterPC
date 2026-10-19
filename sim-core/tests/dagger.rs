//! The Dagger moveset on the phantom body (class 12), modelled mechanically on a reference character (phase 1): the body, every normal
//! with and without the partner, and the partner's boxes (E52). Frame data against the reference for every normal, checked in the data
//! and by running the moves; hit properties; the partner's boxes hitting with the phantom's own on one frame; the awakened boxes
//! chosen as the attack begins. The gauge is `dagger_gauge.rs`, the specials `dagger_specials.rs`. Reference data and sources: the
//! thirteenth moveset's `REFERENCE_DATA.md` (`docs/movesets/dagger/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::JUMP;
use sim_core::moves::{
    Hitbox, MoveId, Weapon, AIM_UP, ANGLE_AUTOLINK, ANGLE_BACKWARD, ANGLE_BY_POSITION,
    ANGLE_FORWARD, HIT_PUMMEL, HIT_THROW, MODE_ANY, MODE_PLAIN, MODE_POWERED, TARGETS_AERIAL,
    TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{FighterParams, Fx, Vec2};

const PHANTOM: u8 = 12;

/// The phantom (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([PHANTOM, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(PHANTOM)].weapon)]
}

/// The partner's mode on (the gauge full, the mode's clock running), so attacks begun now are powered.
fn summon(sim: &mut Sim) {
    let f = &mut sim.state.fighters[0];
    f.gauge = Fx::from_int(100);
    f.gauge_mode = 1800;
}

/// A move's reference frame data: the move, its active windows [first, last] (with and without the partner), FAF, landing lag,
/// autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R]; the pummel's hit on frame 2 here, FAF 8 [D]; the down smash's FAF 52 is the
/// majority reading: conflict 2).
const FRAME_DATA: [FrameData; 24] = [
    (MoveId::Jab, &[(4, 5)], 24, 0, 0, 255),
    (MoveId::Jab2, &[(3, 4)], 24, 0, 0, 255),
    (MoveId::Jab3, &[(3, 4), (4, 4)], 35, 0, 0, 255),
    (MoveId::FTilt, &[(8, 9), (13, 15), (13, 19)], 35, 0, 0, 255),
    (
        MoveId::UTilt,
        &[(8, 8), (9, 9), (10, 19), (11, 19), (20, 23)],
        41,
        0,
        0,
        255,
    ),
    (MoveId::DTilt, &[(8, 9), (10, 11), (12, 14)], 38, 0, 0, 255),
    (
        MoveId::DashAttack,
        &[(6, 7), (15, 21), (15, 16), (17, 18), (19, 21)],
        46,
        0,
        0,
        255,
    ),
    (
        MoveId::FSmash,
        &[(16, 16), (17, 18), (19, 19)],
        48,
        0,
        0,
        255,
    ),
    (
        MoveId::USmash,
        &[(10, 14), (10, 10), (12, 13), (14, 14)],
        52,
        0,
        0,
        255,
    ),
    (MoveId::DSmash, &[(12, 13), (16, 17)], 52, 0, 0, 255),
    (MoveId::FAir, &[(7, 8), (7, 7), (12, 14)], 48, 12, 3, 42),
    (MoveId::BAir, &[(7, 7), (8, 8), (7, 8)], 32, 9, 3, 30),
    (MoveId::UAir, &[(5, 18), (20, 21)], 40, 14, 4, 40),
    (MoveId::DAir, &[(13, 16), (13, 14), (15, 16)], 47, 11, 3, 40),
    (MoveId::Grab, &[(6, 7)], 35, 0, 0, 255),
    (MoveId::DashGrab, &[(8, 9)], 43, 0, 0, 255),
    (MoveId::PivotGrab, &[(9, 10)], 38, 0, 0, 255),
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(9, 9)], 30, 0, 0, 255),
    (MoveId::BThrow, &[(14, 14)], 40, 0, 0, 255),
    (MoveId::UThrow, &[(16, 16)], 38, 0, 0, 255),
    (MoveId::DThrow, &[(23, 23)], 41, 0, 0, 255),
    (MoveId::LedgeAttack, &[(24, 26)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(15, 16), (24, 25)], 46, 0, 0, 255),
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
    // The neutral air: one box a frame on 12-27 (the knife's circle), landing lag 8, autocancels on 1-4 and from 48, FAF 55.
    let nair = w.get(MoveId::NAir as u8);
    assert!(nair
        .hitboxes
        .iter()
        .all(|h| h.start == h.end && (11..=26).contains(&h.start)));
    assert_eq!(
        (
            nair.total_frames + 2,
            nair.landing_lag,
            nair.autocancel_before,
            nair.autocancel_after
        ),
        (55, 8, 4, 47)
    );
    // Intangibility: the ledge attack through frame 26 (the first grab), the get-up attack through 25.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 26);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 25);
    // Smash charge frames (the reference's charge hold on frames 5, 4 and 5: move frames one less).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(4));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(3));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(4));
    // The jab chain: jab 1 into jab 2 from frame 7, jab 2 into jab 3 from 11 (no window past the FAF: both end on 24).
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 6));
    assert_eq!(jab1.late_next, None);
    let jab2 = w.get(MoveId::Jab2 as u8);
    assert_eq!((jab2.next, jab2.next_from), (Some(MoveId::Jab3 as u8), 10));
    assert_eq!(w.get(MoveId::Jab3 as u8).next, None);
    // The multi-hits: the up tilt's loop every 3 frames from 11, the up air's every 4 from 9.
    assert_eq!(w.get(MoveId::UTilt as u8).rehit, Some((10, 3)));
    assert_eq!(w.get(MoveId::UAir as u8).rehit, Some((8, 4)));
    assert!(w.limbs);
}

/// Starts `id` for player 0 (high in the air for an aerial, else standing), powered or not, with player 1 held where the move's box (a
/// hitbox of window `first`, out in that mode, that can hit an airborne fighter) is, and runs until player 1 is hit. Returns the
/// reference frame of the hit.
fn first_hit_frame(id: MoveId, first: u8, powered: bool) -> Option<u8> {
    let mut sim = duel();
    if powered {
        summon(&mut sim);
    }
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let out = if powered { MODE_POWERED } else { MODE_PLAIN };
    let hb = kit(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| {
            h.start + 1 == first
                && h.targets != TARGETS_GROUNDED
                && (h.mode == MODE_ANY || h.mode == out)
                && h.part == 0
        })
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
fn every_attack_first_hits_on_its_reference_frame_with_and_without_the_partner() {
    for (id, first) in [
        (MoveId::Jab, 4u8),
        (MoveId::Jab2, 3),
        (MoveId::Jab3, 3),
        (MoveId::FTilt, 8),
        (MoveId::UTilt, 8),
        (MoveId::DTilt, 8),
        (MoveId::DashAttack, 6),
        (MoveId::FSmash, 16),
        (MoveId::USmash, 10),
        (MoveId::DSmash, 12),
        (MoveId::NAir, 12),
        (MoveId::FAir, 7),
        (MoveId::BAir, 7),
        (MoveId::UAir, 5),
        (MoveId::DAir, 13),
    ] {
        for powered in [false, true] {
            assert_eq!(
                first_hit_frame(id, first, powered),
                Some(first),
                "{id:?} window from {first}, powered {powered}"
            );
        }
    }
}

/// Ticks from starting `id` (by `begin_attack`) until the fighter leaves the move, plus the frames that maps to: the reference's FAF.
fn faf_of(id: MoveId, aerial: bool, powered: bool) -> u16 {
    let mut sim = duel();
    if powered {
        summon(&mut sim);
    }
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
fn every_attack_returns_control_on_its_reference_faf_with_and_without_the_partner() {
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
        for powered in [false, true] {
            assert_eq!(
                faf_of(id, id.is_aerial(), powered),
                u16::from(faf),
                "{id:?} {powered}"
            );
        }
    }
    assert_eq!(faf_of(MoveId::NAir, true, false), 55);
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

/// A hit's expected numbers: the move, its first frame, priority, mode, part, damage in tenths, angle, BK, KS.
type HitRow = (MoveId, u8, u8, u8, u8, i32, i16, i16, i16);

/// The first box of `id` out on reference frame `first` with priority `priority`, in mode `mode`, of part `part`.
fn find(w: &Weapon, id: MoveId, first: u8, priority: u8, mode: u8, part: u8) -> Hitbox {
    *w.get(id as u8)
        .hitboxes
        .iter()
        .find(|h| {
            h.start + 1 == first && h.priority == priority && h.mode == mode && h.part == part
        })
        .unwrap_or_else(|| panic!("{id:?} {first} {priority} {mode} {part}"))
}

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, first frame, priority, mode, part, damage in tenths, angle, BK, KS)
    let table: [HitRow; 34] = [
        (MoveId::Jab, 4, 0, MODE_ANY, 0, 20, 361, 32, 30),
        (MoveId::Jab, 4, 2, MODE_ANY, 0, 20, 180, 32, 20),
        (MoveId::Jab, 4, 4, MODE_POWERED, 1, 7, 361, 33, 30),
        (MoveId::Jab, 4, 5, MODE_POWERED, 1, 7, 180, 33, 30),
        (MoveId::Jab2, 3, 0, MODE_ANY, 0, 15, 361, 25, 30),
        (MoveId::Jab2, 3, 2, MODE_ANY, 0, 15, 180, 30, 30),
        (MoveId::Jab2, 3, 4, MODE_POWERED, 1, 7, 361, 25, 30),
        (MoveId::Jab3, 3, 0, MODE_PLAIN, 0, 40, 361, 60, 80),
        (MoveId::Jab3, 3, 4, MODE_POWERED, 1, 31, 361, 65, 135),
        (MoveId::FTilt, 13, 2, MODE_PLAIN, 0, 50, 45, 45, 128),
        (MoveId::FTilt, 13, 0, MODE_POWERED, 0, 50, 45, 60, 121),
        (MoveId::FTilt, 13, 1, MODE_POWERED, 1, 50, 45, 60, 121),
        (MoveId::UTilt, 20, 3, MODE_PLAIN, 0, 10, 90, 35, 270),
        (MoveId::UTilt, 20, 3, MODE_POWERED, 0, 39, 90, 50, 180),
        (
            MoveId::UTilt,
            11,
            0,
            MODE_POWERED,
            1,
            7,
            ANGLE_AUTOLINK,
            50,
            20,
        ),
        (MoveId::DTilt, 8, 0, MODE_PLAIN, 0, 60, 75, 75, 66),
        (MoveId::DTilt, 12, 1, MODE_POWERED, 1, 70, 82, 70, 99),
        (MoveId::DashAttack, 15, 0, MODE_PLAIN, 0, 60, 361, 60, 85),
        (MoveId::DashAttack, 15, 1, MODE_POWERED, 1, 60, 361, 85, 82),
        (MoveId::FSmash, 16, 0, MODE_PLAIN, 0, 140, 361, 40, 98),
        (MoveId::FSmash, 19, 0, MODE_POWERED, 0, 140, 361, 0, 98),
        (MoveId::FSmash, 16, 2, MODE_POWERED, 1, 80, 361, 50, 151),
        (MoveId::USmash, 10, 0, MODE_PLAIN, 0, 120, 83, 43, 100),
        (MoveId::USmash, 10, 3, MODE_POWERED, 1, 50, 83, 59, 181),
        (MoveId::DSmash, 12, 0, MODE_PLAIN, 0, 120, 32, 30, 93),
        (MoveId::DSmash, 12, 0, MODE_POWERED, 0, 120, 32, 64, 84),
        (MoveId::DSmash, 16, 5, MODE_POWERED, 1, 60, 30, 40, 158),
        (MoveId::NAir, 12, 0, MODE_ANY, 0, 70, 46, 35, 93),
        (MoveId::NAir, 12, 3, MODE_POWERED, 1, 40, 46, 50, 139),
        (MoveId::FAir, 12, 0, MODE_PLAIN, 0, 50, 361, 46, 125),
        (MoveId::FAir, 12, 1, MODE_POWERED, 1, 80, 34, 58, 98),
        (MoveId::BAir, 7, 2, MODE_POWERED, 1, 70, 35, 58, 114),
        (MoveId::UAir, 20, 1, MODE_POWERED, 1, 70, 80, 50, 126),
        (MoveId::DAir, 15, 3, MODE_POWERED, 1, 80, 270, 35, 96),
    ];
    let sim = duel();
    let w = kit(&sim);
    for (id, first, priority, mode, part, damage, angle, bk, ks) in table {
        let hb = find(w, id, first, priority, mode, part);
        assert_eq!(
            (hb.damage, hb.angle, hb.base_knockback, hb.knockback_growth),
            (Fx::from_ratio(damage, 10), angle, bk, ks),
            "{id:?} from {first}, box {priority}"
        );
    }
    // The up air's looping 0.75% (not a whole tenth).
    let uair = find(w, MoveId::UAir, 5, 1, MODE_ANY, 0);
    assert_eq!(
        (uair.damage, uair.angle, uair.targets),
        (fx(3, 4), ANGLE_AUTOLINK, TARGETS_AERIAL)
    );
    // Angle types: the jab forward, the up smash by position, the back air backward.
    assert_eq!(
        find(w, MoveId::Jab, 4, 0, MODE_ANY, 0).angle_type,
        ANGLE_FORWARD
    );
    assert_eq!(
        find(w, MoveId::USmash, 10, 0, MODE_PLAIN, 0).angle_type,
        ANGLE_BY_POSITION
    );
    assert_eq!(
        find(w, MoveId::BAir, 7, 0, MODE_PLAIN, 0).angle_type,
        ANGLE_BACKWARD
    );
    // The forward tilt aimed up: hit 1's tip 180 degrees, set knockback 10, set weight.
    let up = w
        .get(MoveId::FTilt as u8)
        .hitboxes
        .iter()
        .find(|h| h.aim == AIM_UP && h.start + 1 == 8 && h.priority == 1)
        .copied()
        .unwrap();
    assert_eq!((up.angle, up.fixed_kb, up.set_weight), (180, 10, true));
    // The powered forward tilt's second hit: shieldstun x2, hitlag x1.7.
    let ft = find(w, MoveId::FTilt, 13, 0, MODE_POWERED, 0);
    assert_eq!((ft.shieldstun, ft.hitlag), (200, 170));
}

#[test]
fn the_partners_boxes_are_its_own_part_out_only_with_it_and_never_on_grabs_throws_or_recoveries() {
    let sim = duel();
    let w = kit(&sim);
    for id in 0..sim_core::moves::MoveId::COUNT as u8 {
        for h in &w.get(id).hitboxes {
            if h.part == 1 {
                assert_eq!(h.mode, MODE_POWERED, "{id}");
                assert!(!h.clanks && !h.rebounds, "{id}");
            }
        }
    }
    for id in [
        MoveId::Jab,
        MoveId::Jab2,
        MoveId::Jab3,
        MoveId::FTilt,
        MoveId::UTilt,
        MoveId::DTilt,
        MoveId::DashAttack,
        MoveId::FSmash,
        MoveId::USmash,
        MoveId::DSmash,
        MoveId::NAir,
        MoveId::FAir,
        MoveId::BAir,
        MoveId::UAir,
        MoveId::DAir,
    ] {
        assert!(
            w.get(id as u8).hitboxes.iter().any(|h| h.part == 1),
            "{id:?} has the partner's boxes"
        );
    }
    for id in [
        MoveId::Grab,
        MoveId::Pummel,
        MoveId::FThrow,
        MoveId::BThrow,
        MoveId::UThrow,
        MoveId::DThrow,
        MoveId::LedgeAttack,
        MoveId::GetUpAttack,
    ] {
        assert!(
            w.get(id as u8)
                .hitboxes
                .iter()
                .all(|h| h.part == 0 && h.mode == MODE_ANY),
            "{id:?}"
        );
    }
    assert_eq!(w.get(MoveId::Pummel as u8).hitboxes[0].kind, HIT_PUMMEL);
    for id in [
        MoveId::FThrow,
        MoveId::BThrow,
        MoveId::UThrow,
        MoveId::DThrow,
    ] {
        let h = w.get(id as u8).hitboxes[0];
        assert_eq!((h.kind, h.hitlag), (HIT_THROW, 0), "{id:?}");
    }
}

/// Player 1's percent after player 0's `id` lands on it at point blank, powered or not (counted from 0%).
fn damage_of(id: MoveId, powered: bool) -> Fx {
    let mut sim = duel();
    if powered {
        summon(&mut sim);
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(13, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            break;
        }
    }
    sim.fighter(1).percent
}

#[test]
fn with_the_partner_out_both_its_hit_and_the_phantoms_land_on_the_same_frame() {
    // The jab: 2% alone; with the partner 2% + 0.7% (the reference's "2.7"), both times the one-on-one 1.2.
    let k = Duel::mult();
    assert_eq!(damage_of(MoveId::Jab, false), Fx::from_int(2) * k);
    assert_eq!(
        damage_of(MoveId::Jab, true),
        (Fx::from_int(2) + fx(7, 10)) * k
    );
    // The down tilt: 6% alone, 6% + 7% with the partner.
    assert_eq!(damage_of(MoveId::DTilt, false), Fx::from_int(6) * k);
    assert_eq!(damage_of(MoveId::DTilt, true), Fx::from_int(13) * k);
    // And on one frame: the jab's two hits arrive together.
    let mut sim = duel();
    summon(&mut sim);
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(13, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
    let mut frames = Vec::new();
    let mut last = Fx::ZERO;
    for t in 1..=10 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent != last {
            frames.push(t + 1);
            last = sim.fighter(1).percent;
        }
    }
    assert_eq!(frames, vec![4], "one frame, the reference's 4");
}

struct Duel;
impl Duel {
    fn mult() -> Fx {
        sim_core::Content::placeholder().rules.damage_mult
    }
}

#[test]
fn an_attack_keeps_the_boxes_it_began_with_when_the_partner_comes_or_goes() {
    // Begun with the partner, the forward tilt's second hit is out to frame 19 even if the partner leaves at once (the reference:
    // "the awakened values apply until the attack ends"); begun without it, it ends on 15 even if the partner comes.
    for (start_powered, then_powered, out_on_17) in [(true, false, true), (false, true, false)] {
        let mut sim = duel();
        if start_powered {
            summon(&mut sim);
        }
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_int(10), -1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FTilt);
        assert_eq!(sim.f().move_powered, start_powered);
        if then_powered {
            summon(&mut sim);
        } else {
            sim.state.fighters[0].gauge_mode = 0;
        }
        let w = kit(&sim).clone();
        // (Move frame 16 is the reference's 17.)
        sim.state.fighters[0].state_frame = 16;
        let mv = w.get(MoveId::FTilt as u8);
        let out = sim_core::combat::active_hitboxes(sim.f(), mv, Fx::ONE).count() > 0;
        assert_eq!(out, out_on_17, "begun powered {start_powered}");
    }
}

#[test]
fn the_partners_down_air_stomp_meteors() {
    let mut sim = duel();
    summon(&mut sim);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    let w = kit(&sim).clone();
    let stomp = find(&w, MoveId::DAir, 15, 3, MODE_POWERED, 1);
    // Held low under the phantom, out of the knife's reach, until the stomp on frame 15.
    for _ in 1..=13 {
        let me = sim.f().pos;
        sim.put_airborne(
            1,
            me.x + stomp.x,
            me.y - Fx::from_int(3),
            Fx::ZERO,
            Fx::ZERO,
        );
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).percent, Fx::ZERO, "the knife missed");
    let hit = (0..4).any(|_| {
        sim.tick(inp(0, 0, 0));
        sim.fighter(1).state == S::Hitstun
    });
    assert!(hit, "the stomp lands");
    let angle = sim.fighter(1).launch_angle;
    // (270 degrees: straight down.)
    assert_eq!(angle, sim_core::trig::Angle::from_degrees(270).raw());
}

// ---- The body ----------------------------------------------------------------------------------------------------------------

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
fn the_phantoms_jumps_have_the_references_air_times() {
    // Short hop 30, full hop 45, short hop fast fall 20, full hop fast fall 30 (ultimateframedata.com); within one frame.
    for (full, ff, reference) in [
        (false, false, 30u32),
        (true, false, 45),
        (false, true, 20),
        (true, true, 30),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_phantom_is_the_references_quick_middleweight_and_is_class_12() {
    let sim = duel();
    let p = sim.content.fighters[usize::from(PHANTOM)];
    assert_eq!(p, FighterParams::phantom());
    let su = FighterParams::su;
    let gu = FighterParams::gu;
    assert_eq!(p.weight, Fx::from_int(93));
    assert_eq!(p.gravity, su(127));
    assert_eq!((p.max_fall_speed, p.fast_fall_speed), (su(1630), su(3097)));
    assert_eq!(
        (p.air_speed, p.air_accel, p.air_accel_stick, p.air_friction),
        (su(1100), su(10), su(70), su(10))
    );
    assert_eq!(
        (
            p.walk_speed,
            p.dash_speed,
            p.run_speed,
            p.dash_attack_friction
        ),
        (gu(1120), gu(1900), gu(2060), gu(130))
    );
    assert_eq!(
        (p.dash_frames, p.heavy_landing_lag, p.air_jump_drift),
        (10, 3, 90)
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (47, 67, 83, 100)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (29, 34, 25)
    );
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (1, su(1300), su(2700))
    );
    assert_eq!(p.gauge, FighterParams::phantom_gauge());
    assert_eq!(sim.content.names.fighters[usize::from(PHANTOM)], "phantom");
    assert_eq!(sim.content.names.weapons[usize::from(p.weapon)], "dagger");
}

#[test]
fn the_dagger_is_deterministic_and_rolls_back_with_the_partner_out() {
    // Two phantoms, one with the partner out, mashing everything: a snapshot resimulated gives the same checksums.
    let mut sim = Sim::with_chars([PHANTOM, PHANTOM, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    summon(&mut sim);
    let mut rng = sim_core::Rng::new(7);
    let mut inputs = Vec::new();
    for _ in 0..600 {
        let r = rng.next_u32();
        let pick = |s: u32| {
            inp(
                ((s & 0xff) as i8) / 2 * 2,
                (((s >> 8) & 0xff) as i8) / 2 * 2,
                ((s >> 16) & 0x3f) as u16,
            )
        };
        inputs.push([
            pick(r),
            pick(r.rotate_left(11)),
            Default::default(),
            Default::default(),
        ]);
    }
    let snapshot = sim.state;
    let mut sums = Vec::new();
    for i in &inputs {
        sim_core::step(&mut sim.state, &sim.content, i);
        sums.push(sim.state.checksum());
    }
    let mut again = snapshot;
    for (n, i) in inputs.iter().enumerate() {
        sim_core::step(&mut again, &sim.content, i);
        assert_eq!(again.checksum(), sums[n], "frame {n}");
    }
}
