//! The Buckler moveset on the wanderer body (class 9), modelled mechanically on a reference character (phase 1: the body and its
//! passive shield, every sword normal, both versions of the up special; the other specials are still the claws'). Frame data against
//! the reference for every normal, checked in the data and by running the moves; hit properties; the jab's continuable windows and
//! held rules; the two-part forward smash keeping its charge (E36); the down air's bounce; grabs, pummel and throws; landing lag and
//! autocancels; the body's attributes and jump air times; the grounded spin (one charged hit in four phases) and the aerial spin (eight
//! hits, the position vectors (E34), the rise, ledges, the special fall (E11), landing lag); the passive shield (E14); up special and
//! up smash straight out of a shield for several classes (E37); registration; determinism. Reference data and sources:
//! `docs/movesets/link/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD, SPECIAL};
use sim_core::moves::buckler_slots::{
    AIR_SPIN_HITS, DAIR_BOUNCE, DAIR_BOUNCE_END, FSMASH_2, GROUND_SPIN_PHASES, SPIN_AIR,
    SPIN_GRAVITY, SPIN_RISE_SPEED,
};
use sim_core::moves::{
    Move, MoveId, Weapon, ANGLE_AUTOLINK, ANGLE_BACKWARD, ANGLE_FORWARD, HIT_GRAB, HIT_NORMAL,
    TARGETS_AERIAL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const WANDERER: u8 = 9;

/// The wanderer (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([WANDERER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(WANDERER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal and its follow-ups, from `REFERENCE_DATA.md` (all [R]; the forward tilt's early frame 15 and the up tilt's sweep are
/// their own windows; a throw's hit and its release are windows of a frame each, on their first frames; the up smash's clean and late
/// phases are windows).
const FRAME_DATA: [FrameData; 27] = [
    (MoveId::Jab, &[(7, 8)], 24, 0, 0, 255),
    (MoveId::Jab2, &[(5, 6)], 26, 0, 0, 255),
    (MoveId::Jab3, &[(6, 7)], 35, 0, 0, 255),
    (MoveId::FTilt, &[(15, 15), (16, 19)], 40, 0, 0, 255),
    (MoveId::UTilt, &[(8, 9), (10, 10), (11, 12)], 36, 0, 0, 255),
    (MoveId::DTilt, &[(10, 11)], 31, 0, 0, 255),
    (MoveId::DashAttack, &[(20, 23)], 57, 0, 0, 255),
    (MoveId::FSmash, &[(17, 18)], 51, 0, 0, 255),
    (FSMASH_2, &[(10, 11)], 61, 0, 0, 255),
    (
        MoveId::USmash,
        &[(10, 13), (14, 15), (25, 28), (29, 30), (41, 45)],
        78,
        0,
        0,
        255,
    ),
    (MoveId::DSmash, &[(12, 13), (24, 25)], 57, 0, 0, 255),
    (MoveId::NAir, &[(7, 8), (9, 31)], 39, 6, 3, 36),
    (MoveId::FAir, &[(16, 17), (24, 25)], 52, 11, 0, 53),
    (MoveId::BAir, &[(6, 8), (15, 17)], 31, 6, 0, 29),
    (MoveId::UAir, &[(11, 13), (14, 40)], 60, 14, 10, 43),
    (MoveId::DAir, &[(14, 19), (20, 64)], 80, 19, 9, 65),
    (DAIR_BOUNCE, &[(30, 50)], 80, 19, 0, 255),
    (MoveId::Grab, &[(6, 7)], 35, 0, 0, 255),
    (MoveId::DashGrab, &[(9, 10)], 43, 0, 0, 255),
    (MoveId::PivotGrab, &[(10, 11)], 38, 0, 0, 255),
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(12, 12), (13, 13)], 38, 0, 0, 255),
    (MoveId::BThrow, &[(12, 12), (14, 14)], 39, 0, 0, 255),
    (MoveId::UThrow, &[(26, 26), (28, 28)], 50, 0, 0, 255),
    (MoveId::DThrow, &[(22, 22), (24, 24)], 49, 0, 0, 255),
    (MoveId::LedgeAttack, &[(20, 22)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(17, 18), (21, 22)], 46, 0, 0, 255),
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
        assert_eq!(mv.rehit, None, "{id:?}: no rhythm rehit");
    }
    // The last frames after a bounce: FAF 80 of the down air (its frames 65 to 79), every landing autocancels.
    let end = w.get(DAIR_BOUNCE_END as u8);
    assert_eq!((end.total_frames, end.autocancel_after), (14, 0));
    // Intangibility: the ledge attack and the get-up attack through frame 22 (the first grab).
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 22);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 22);
    // Smash charge frames ("charges between" 9-10, 5-6, 4-5: move frames one less than the first).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(8));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(4));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(3));
    // The continuable windows: jab 1 into jab 2 from frame 10, jab 2 into jab 3 from 8; jab 3 ends the chain (no rapid jab).
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 9));
    let jab2 = w.get(MoveId::Jab2 as u8);
    assert_eq!((jab2.next, jab2.next_from), (Some(MoveId::Jab3 as u8), 7));
    assert_eq!(w.get(MoveId::Jab3 as u8).next, None);
    // The forward smash's second hit keeps the first's charge (E36); nothing else does.
    for (i, m) in w.moves.iter().enumerate() {
        assert_eq!(m.keep_charge, i == FSMASH_2 as usize, "slot {i}");
    }
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (in the air high up for an aerial, else standing), with player 1 held where the first box of window
/// `first` that can hit an airborne fighter will be, and runs until player 1 is hit. Returns the reference frame of the hit.
fn first_hit_frame(id: MoveId, first: u8) -> Option<u8> {
    let mut sim = duel();
    let aerial = id.is_aerial() || id == SPIN_AIR;
    if aerial {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let hb = *kit(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| h.start + 1 == first && h.targets != TARGETS_GROUNDED)
        .expect("a hitbox in that window");
    let before = sim.fighter(1).percent;
    for tick in 1..=90u8 {
        let me = sim.state.fighters[0].pos;
        let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
        let at = Vec2::new(me.x + hb.x, (me.y + hb.y - middle).max(fx(1, 10)));
        sim.put_airborne(1, at.x, at.y, Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            return Some(tick + 1);
        }
        if sim.f().state != S::Attack {
            return None;
        }
    }
    None
}

#[test]
fn every_attack_first_hits_on_its_reference_frame() {
    for (id, first) in [
        (MoveId::Jab, 7u8),
        (MoveId::Jab2, 5),
        (MoveId::Jab3, 6),
        (MoveId::FTilt, 15),
        (MoveId::UTilt, 8),
        (MoveId::DTilt, 10),
        (MoveId::DashAttack, 20),
        (MoveId::FSmash, 17),
        (FSMASH_2, 10),
        (MoveId::USmash, 10),
        (MoveId::DSmash, 12),
        (MoveId::NAir, 7),
        (MoveId::FAir, 16),
        (MoveId::BAir, 6),
        (MoveId::UAir, 11),
        (MoveId::DAir, 14),
        (MoveId::UpSpecial, 7),
        (SPIN_AIR, 8),
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
        // (Throws need someone held, the ledge attack a ledge; the after-bounce move is reached by bouncing.)
        if id == DAIR_BOUNCE
            || matches!(
                id,
                MoveId::Pummel
                    | MoveId::FThrow
                    | MoveId::BThrow
                    | MoveId::UThrow
                    | MoveId::DThrow
                    | MoveId::LedgeAttack
            )
        {
            continue;
        }
        assert_eq!(faf_of(id, id.is_aerial()), u16::from(faf), "{id:?}");
    }
    // The grounded spin, uncharged: FAF 77 (ssbwiki.com; 74 from release).
    assert_eq!(faf_of(MoveId::UpSpecial, false), 77);
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority (the reference's id), damage in tenths, angle, BK, KS) for the boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 52] = [
        (MoveId::Jab, 7, 0, 30, 361, 25, 25),
        (MoveId::Jab, 7, 1, 30, 361, 25, 25),
        (MoveId::Jab, 7, 2, 30, 180, 20, 15),
        (MoveId::Jab2, 5, 0, 30, 361, 25, 25),
        (MoveId::Jab2, 5, 2, 30, 180, 20, 15),
        (MoveId::Jab3, 6, 0, 40, 30, 70, 70),
        (MoveId::Jab3, 6, 2, 40, 30, 70, 70),
        (MoveId::FTilt, 16, 0, 130, 361, 55, 82),
        (MoveId::FTilt, 15, 3, 130, 361, 55, 82),
        (MoveId::UTilt, 8, 0, 110, 95, 30, 103),
        (MoveId::UTilt, 10, 1, 110, 85, 30, 111),
        (MoveId::UTilt, 11, 2, 110, 85, 30, 105),
        (MoveId::UTilt, 11, 3, 110, 85, 30, 104),
        (MoveId::DTilt, 10, 0, 90, 85, 90, 30),
        (MoveId::DashAttack, 20, 0, 140, 45, 70, 85),
        (MoveId::DashAttack, 20, 1, 130, 50, 85, 85),
        (MoveId::DashAttack, 20, 2, 120, 55, 85, 77),
        (MoveId::FSmash, 17, 0, 70, 69, 45, 12),
        (MoveId::FSmash, 17, 1, 140, 361, 30, 100),
        (MoveId::FSmash, 17, 2, 70, 60, 45, 12),
        (FSMASH_2, 10, 0, 130, 48, 85, 89),
        (FSMASH_2, 10, 2, 120, 48, 85, 89),
        (FSMASH_2, 10, 3, 120, 48, 85, 89),
        (MoveId::USmash, 10, 0, 40, 105, 0, 100),
        (MoveId::USmash, 10, 1, 40, 115, 0, 100),
        (MoveId::USmash, 10, 2, 40, 107, 0, 100),
        (MoveId::USmash, 14, 3, 40, 135, 0, 100),
        (MoveId::USmash, 25, 0, 30, 105, 0, 100),
        (MoveId::USmash, 41, 0, 110, 80, 60, 101),
        (MoveId::USmash, 41, 1, 110, 90, 60, 101),
        (MoveId::USmash, 41, 2, 100, 90, 60, 101),
        (MoveId::USmash, 41, 3, 90, 90, 60, 101),
        (MoveId::DSmash, 12, 0, 160, 78, 40, 88),
        (MoveId::DSmash, 12, 1, 170, 78, 40, 88),
        (MoveId::DSmash, 12, 3, 140, 78, 40, 88),
        (MoveId::DSmash, 24, 0, 120, 30, 80, 63),
        (MoveId::DSmash, 24, 3, 100, 30, 80, 63),
        (MoveId::NAir, 7, 0, 110, 361, 22, 100),
        (MoveId::NAir, 7, 1, 90, 361, 22, 100),
        (MoveId::NAir, 9, 0, 60, 361, 15, 100),
        (MoveId::FAir, 16, 0, 80, 43, 31, 28),
        (MoveId::FAir, 16, 1, 80, ANGLE_AUTOLINK, 25, 30),
        (MoveId::FAir, 16, 5, 80, 38, 26, 26),
        (MoveId::FAir, 24, 0, 100, 44, 43, 120),
        (MoveId::BAir, 6, 0, 50, 73, 0, 100),
        (MoveId::BAir, 15, 0, 70, 361, 45, 98),
        (MoveId::UAir, 11, 0, 150, 85, 23, 93),
        (MoveId::UAir, 14, 0, 130, 85, 18, 88),
        (MoveId::DAir, 14, 0, 180, 270, 30, 80),
        (MoveId::DAir, 14, 1, 180, 60, 30, 78),
        (MoveId::DAir, 20, 0, 150, 60, 30, 80),
        (DAIR_BOUNCE, 30, 0, 110, 65, 50, 80),
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
    // Hitlag multipliers.
    for (id, hitlag) in [
        (MoveId::Jab, 160),
        (MoveId::Jab2, 120),
        (MoveId::Jab3, 200),
        (MoveId::FTilt, 100),
        (MoveId::UTilt, 90),
        (MoveId::DTilt, 100),
        (FSMASH_2, 150),
        (MoveId::USmash, 100),
        (MoveId::DSmash, 100),
        (MoveId::NAir, 100),
        (MoveId::UAir, 120),
        (MoveId::DAir, 120),
        (MoveId::Pummel, 210),
    ] {
        assert!(all(id, &|h| h.hitlag == hitlag), "{id:?}");
    }
    assert!(all(MoveId::DashAttack, &|h| h.hitlag
        == if h.priority == 0 { 120 } else { 100 }
        && h.shieldstun == 140));
    assert!(all(MoveId::FAir, &|h| h.hitlag
        == if h.group == 0 { 80 } else { 120 }));
    assert!(all(MoveId::BAir, &|h| h.hitlag
        == if h.group == 0 { 80 } else { 100 }));
    // Set knockback, set weight and targets: the forward air's first hit (grounded / aerial boxes), the back air's first kick (FKV
    // 50 grounded, 36 aerial), the up smash's first two hits (FKV 55 / 38 / 28, the late aerial tip 45 and 40, SDI x0.5, set weight).
    assert!(all(MoveId::FAir, &|h| h.set_weight == (h.group == 0)
        && (h.group == 1
            || h.targets
                == if h.priority % 2 == 0 {
                    TARGETS_GROUNDED
                } else {
                    TARGETS_AERIAL
                })));
    assert!(all(MoveId::BAir, &|h| h.group == 1
        || (h.set_weight
            && h.fixed_kb
                == if h.targets == TARGETS_GROUNDED {
                    50
                } else {
                    36
                })));
    for h in &get(MoveId::USmash).hitboxes {
        if h.group == 2 {
            assert!(!h.set_weight && h.fixed_kb == 0 && h.hitstun_add == 3 && !h.clanks);
            continue;
        }
        let late_tip = h.priority == 3 && matches!(h.start + 1, 14 | 29);
        let fkv = match h.priority {
            0 => 55,
            1 => 38,
            3 if late_tip => [45, 40][usize::from(h.group)],
            _ => 28,
        };
        assert_eq!(
            (h.fixed_kb, h.set_weight, h.sdi, h.hitstun_add),
            (fkv, true, 50, if late_tip { 0 } else { 3 }),
            "up smash {} id {}",
            h.start + 1,
            h.priority
        );
    }
    // Trips: the down smash's 30% on every box, nothing else.
    for (id, ..) in FRAME_DATA {
        assert!(
            all(id, &|h| h.trip == if id == MoveId::DSmash { 30 } else { 0 }),
            "{id:?}"
        );
    }
    // Shield damage: the get-up attack's +8 (214) [D]; nothing else adds any.
    assert!(all(MoveId::GetUpAttack, &|h| h.shield_damage == 214));
    // The grab boxes.
    for id in [MoveId::Grab, MoveId::DashGrab, MoveId::PivotGrab] {
        assert!(all(id, &|h| h.kind == HIT_GRAB));
    }
}

#[test]
fn only_the_references_no_clank_boxes_never_clank() {
    // `clang=f rebound=f` (E3c): the forward and up tilts', the dash attack's and forward smash hit 2's arm and hilt boxes, the down
    // tilt, the forward air's inner boxes, the up smash's last hit, the pummel, the throws' hits, the ledge and get-up attacks; and
    // every box of the spin but the grounded spin's latest phase. Every other box clanks and rebounds.
    let sim = duel();
    let w = kit(&sim);
    let never = |id: MoveId, h: &sim_core::moves::Hitbox| -> bool {
        match id {
            MoveId::FTilt | MoveId::UTilt | MoveId::DashAttack => h.priority >= 2,
            MoveId::DTilt | MoveId::Pummel | MoveId::LedgeAttack | MoveId::GetUpAttack => true,
            MoveId::FAir => h.priority >= 2,
            MoveId::USmash => h.group == 2,
            MoveId::FThrow | MoveId::BThrow | MoveId::UThrow | MoveId::DThrow => {
                h.kind != sim_core::moves::HIT_THROW
            }
            MoveId::UpSpecial => h.start + 1 < 24,
            _ if id == FSMASH_2 => h.priority == 3,
            _ if id == SPIN_AIR => true,
            _ => false,
        }
    };
    let mut ids: Vec<MoveId> = FRAME_DATA.iter().map(|d| d.0).collect();
    ids.extend([MoveId::UpSpecial, SPIN_AIR]);
    for id in ids {
        for h in &w.get(id as u8).hitboxes {
            let n = never(id, h);
            assert_eq!(
                h.clanks,
                !n,
                "{id:?} {} id {} clanks",
                h.start + 1,
                h.priority
            );
            assert_eq!(h.rebounds, !n, "{id:?} rebounds");
        }
    }
}

// ---- Jab -----------------------------------------------------------------------------------------------------------------------

/// Runs player 0's attack button down on the ticks in `down` (1-based) with player 1 in front (`hittable` or not); returns (tick,
/// move, move frame, state) after each tick for `ticks` ticks.
fn jab_run(down: &[u32], ticks: u32, hittable: bool) -> Vec<(u32, MoveId, u16, S)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(13, 10), -1);
    if !hittable {
        sim.state.fighters[1].invuln = 255;
    }
    let mut out = Vec::new();
    for t in 1..=ticks {
        let b = if down.contains(&t) { ATTACK } else { 0 };
        sim.tick(inp(0, 0, b));
        out.push((
            t,
            MoveId::from_index(sim.f().move_id),
            sim.f().state_frame,
            sim.f().state,
        ));
    }
    out
}

/// The first tick on which `id` is at move frame 0.
fn started(run: &[(u32, MoveId, u16, S)], id: MoveId) -> Option<u32> {
    run.iter()
        .find(|(_, m, f, s)| *m == id && *f == 0 && *s == S::Attack)
        .map(|r| r.0)
}

fn control_back(run: &[(u32, MoveId, u16, S)]) -> Option<u32> {
    run.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1)
}

#[test]
fn the_quickest_jab_chain_runs_on_the_references_frames() {
    // Pressed on 1, 3 and 12: jab 1 from tick 1 (hits on 7-8), jab 2 from 10 (jab 1's frame 10; hits on 14-15), jab 3 from 17 (jab 2's
    // frame 8; hits on 22-23), control back on 51: the reference's quickest loop (7-8, 14-15, 22-23; FAF 24 / 34 / 51).
    let run = jab_run(&[1, 3, 12], 60, false);
    assert_eq!(started(&run, MoveId::Jab), Some(1));
    assert_eq!(started(&run, MoveId::Jab2), Some(10));
    assert_eq!(started(&run, MoveId::Jab3), Some(17));
    let sim = duel();
    let w = kit(&sim);
    let out_on = |id: MoveId| -> Vec<u32> {
        run.iter()
            .filter(|(_, m, f, s)| {
                *m == id
                    && *s == S::Attack
                    && w.get(id as u8)
                        .hitboxes
                        .iter()
                        .any(|h| u16::from(h.start) <= *f && *f <= u16::from(h.end))
            })
            .map(|r| r.0)
            .collect()
    };
    assert_eq!(out_on(MoveId::Jab), vec![7, 8]);
    assert_eq!(out_on(MoveId::Jab2), vec![14, 15]);
    assert_eq!(out_on(MoveId::Jab3), vec![22, 23]);
    assert_eq!(
        control_back(&run),
        Some(51),
        "jab 3's FAF, counted from jab 1"
    );
    // Jab 1 alone gives control back on 24, jab 1 and 2 on 34 (10 + 26 - 2).
    assert_eq!(control_back(&jab_run(&[1], 40, false)), Some(24));
    assert_eq!(control_back(&jab_run(&[1, 3], 50, false)), Some(35));
}

#[test]
fn holding_attack_carries_on_from_frame_10_after_a_hit_and_from_frame_14_without_one() {
    let held: Vec<u32> = (1..=80).collect();
    // Held, nothing hit: jab 2 from jab 1's frame 14 (tick 14), jab 3 from jab 2's frame 13.
    let miss = jab_run(&held, 60, false);
    assert_eq!(started(&miss, MoveId::Jab2), Some(14));
    assert_eq!(started(&miss, MoveId::Jab3), Some(14 + 12));
    // Held, and jab 1 hit: jab 2 from jab 1's frame 10 (counted in move frames: the hit's hitlag holds the move frame still).
    let hit = jab_run(&held, 80, true);
    let at = started(&hit, MoveId::Jab2).expect("jab 2 started");
    let last = hit
        .iter()
        .rfind(|r| r.0 < at && r.1 == MoveId::Jab)
        .map(|r| r.2);
    assert_eq!(last, Some(8), "jab 2 replaces jab 1's frame 10");
    // Let go after the press: no jab 2.
    assert_eq!(started(&jab_run(&[1], 40, true), MoveId::Jab2), None);
}

// ---- The forward smash's two hits (E36) -----------------------------------------------------------------------------------------

/// Player 0 does a forward smash held for `charge` frames, pressing attack again on `again` ticks after its start; player 1 stands in
/// front, hittable. Returns the percent rises (tick, amount).
fn fsmash_run(charge: u32, again: u32) -> (Vec<(u32, Fx)>, Sim) {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(16, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
    let mut rises = Vec::new();
    let mut last = sim.fighter(1).percent;
    for t in 1..=160u32 {
        let held = t < 8 + charge;
        let press = t == again;
        let b = if held || press { ATTACK } else { 0 };
        // (Player 1 stays where hit 2 reaches, standing.)
        sim.tick(inp(0, 0, b));
        if sim.fighter(1).percent > last {
            rises.push((t, sim.fighter(1).percent - last));
            last = sim.fighter(1).percent;
            sim.stand(1, fx(16, 10), -1);
        }
    }
    (rises, sim)
}

#[test]
fn the_forward_smash_carries_on_into_its_second_hit_which_keeps_the_charge() {
    // Uncharged, pressed again on tick 24 (its frame 25, in the window 23-36): hit 1 (7% blade or 14% tip) then hit 2 (13%).
    let (rises, sim) = fsmash_run(0, 24);
    assert_eq!(rises.len(), 2, "{rises:?}");
    assert_eq!(rises[1].1, dealt(&sim, 130));
    // Fully charged (60 frames): both hits x1.4 (hit 2 keeps hit 1's charge, E36).
    let (rises, sim) = fsmash_run(70, 24 + 60);
    assert_eq!(rises.len(), 2, "{rises:?}");
    let full = |tenths: i32| {
        Fx::from_ratio(tenths, 10) * Fx::from_ratio(14, 10) * sim.content.rules.damage_mult
    };
    assert!((rises[1].1 - full(130)).abs() < fx(1, 100), "{rises:?}");
    // Not pressed again: one hit.
    let (rises, _) = fsmash_run(0, 0);
    assert_eq!(rises.len(), 1);
    // Pressed before the buffer reaches the window (frame 10): no second hit.
    let (rises, _) = fsmash_run(0, 10);
    assert_eq!(rises.len(), 1);
}

// ---- The down air's bounce -----------------------------------------------------------------------------------------------------

/// The down air started high up over an airborne duelist held under its box; `fast_fall` snaps the stick down first. Returns the
/// sim after the first hit plus `after` ticks, and whether it bounced.
fn dair_hit(fast_fall: bool) -> Sim {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, fx(-1, 10));
    sim.state.fighters[0].facing = 1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    if fast_fall {
        sim.state.fighters[0].fast_fall = true;
    }
    let before = sim.fighter(1).percent;
    for _ in 0..40 {
        let me = sim.f().pos;
        sim.put_airborne(1, me.x, me.y - Fx::from_int(2), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            return sim;
        }
    }
    panic!("the down air never hit");
}

#[test]
fn the_down_air_bounces_off_a_target_when_not_fast_falling() {
    let mut sim = dair_hit(false);
    let hit_frame = sim.f().state_frame;
    // Hitlag, then the switch and the bounce: up at 1.33 of the reference's units a frame.
    let mut rose = false;
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.f().move_id == DAIR_BOUNCE as u8 && sim.f().vel.y > Fx::ZERO {
            rose = true;
            assert_eq!(sim.f().vel.y, Fx::from_ratio(16625, 100_000));
            break;
        }
    }
    assert!(rose, "it bounced");
    // The after-bounce box: out 30 frames after the hit (in the down air's own count), 11%.
    let start = sim.f().vars[0] >> 16;
    let w = kit(&sim);
    let first = w.get(DAIR_BOUNCE as u8).hitboxes[0].start;
    assert_eq!(i32::from(first) + start, i32::from(hit_frame) + 30);
    // Fast falling: no bounce.
    let mut ff = dair_hit(true);
    for _ in 0..30 {
        ff.tick(inp(0, 0, 0));
        assert_eq!(ff.f().move_id, MoveId::DAir as u8);
    }
}

#[test]
fn the_after_bounce_box_hits_again_and_the_move_ends_on_the_down_airs_faf() {
    // A target held under the fighter all along: the clean hit, then the after-bounce 11% 30 frames later; the move gives control
    // back on the down air's FAF counted through the bounce (80 frames of move, hitlag aside).
    let mut sim = dair_hit(false);
    let mut last = sim.fighter(1).percent;
    let mut later = Vec::new();
    for _ in 0..200 {
        let me = sim.f().pos;
        sim.put_airborne(1, me.x, me.y - Fx::from_int(1), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            later.push(sim.fighter(1).percent - last);
            last = sim.fighter(1).percent;
        }
        if sim.f().state != S::Attack {
            break;
        }
    }
    assert_eq!(later.first().copied(), Some(dealt(&sim, 110)));
    assert_ne!(sim.f().state, S::Attack, "the move ended");
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
fn the_hand_grab_catches_on_frame_6_at_short_range() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    assert_eq!(grab_now(&mut sim), 6);
}

#[test]
fn the_throws_hit_and_release_on_their_frames_for_their_damage() {
    // (stick, the frames the held fighter's percent rises on, total damage in tenths)
    for (stick, frames, damage) in [
        ((127i8, 0i8), vec![12u8, 13], 55),
        ((-127, 0), vec![12, 14], 55),
        ((0, 127), vec![26, 28], 70),
        ((0, -127), vec![22, 24], 60),
    ] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(12, 10), -1);
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
        assert!(
            (sim.fighter(1).percent - before - dealt(&sim, damage)).abs() < fx(1, 1000),
            "throw {stick:?}"
        );
        if stick.0 < 0 {
            assert!(sim.fighter(1).pos.x < Fx::ZERO, "thrown behind");
        }
    }
}

#[test]
fn the_pummel_does_1_3_percent() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Pummel as u8);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 13));
}

// ---- Landing ---------------------------------------------------------------------------------------------------------------------

fn land_during(id: MoveId, frame_before_tick: u16) -> (S, u8) {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-9), fx(2, 100), Fx::ZERO, fx(-1, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = id as u8;
    f.state_frame = frame_before_tick;
    f.hit_mask = 0;
    sim.tick(inp(0, 0, 0));
    (sim.f().state, sim.f().lag)
}

#[test]
fn aerials_cost_their_landing_lag_and_autocancel_in_their_windows() {
    let plain = duel().content.fighters[usize::from(WANDERER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window or none, one in the early window or none;
    // the tick adds one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 15u16, 6u8, Some(36u16), Some(1u16)),
        (MoveId::FAir, 20, 11, None, None),
        (MoveId::BAir, 12, 6, Some(28), None),
        (MoveId::UAir, 20, 14, Some(42), Some(8)),
        (MoveId::DAir, 30, 19, Some(64), Some(7)),
        (SPIN_AIR, 30, 30, None, None),
    ] {
        assert_eq!(land_during(id, mid), (S::Landing, lag), "{id:?} mid-move");
        if let Some(late) = late {
            assert_eq!(land_during(id, late).1, plain, "{id:?} late");
        }
        if let Some(early) = early {
            assert_eq!(land_during(id, early).1, plain, "{id:?} early");
        }
    }
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
fn the_wanderers_jumps_have_the_references_air_times() {
    // Short hop 33, full hop 45, short hop fast fall 22, full hop fast fall 30 (ultimateframedata.com); within one frame.
    for (full, ff, reference) in [
        (false, false, 33u32),
        (true, false, 45),
        (false, true, 22),
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
fn the_wanderer_is_the_references_heavy_low_jumping_swordsman() {
    let p = duel().content.fighters[usize::from(WANDERER)];
    assert_eq!(p, FighterParams::wanderer());
    let su = FighterParams::su;
    let gu = FighterParams::gu;
    assert_eq!(p.weight, Fx::from_int(104));
    assert_eq!(
        (p.walk_speed, p.dash_speed, p.run_speed),
        (gu(1247), gu(1980), gu(1534))
    );
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(924), su(96), su(1600), su(3040))
    );
    assert_eq!((p.air_accel, p.air_accel_stick), (su(10), su(40)));
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (49, 69, 82, 108)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (30, 35, 26)
    );
    assert_eq!((p.dash_frames, p.heavy_landing_lag), (12, 3));
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (0, su(1300), su(2600))
    );
    assert_eq!(p.passive_shield, Some(FighterParams::wanderer_shield()));
    assert!(!p.air_jump_turns && p.air_jump_armor.is_none());
}

// ---- The grounded spin ---------------------------------------------------------------------------------------------------------

/// The grounded spin with special held `held` ticks from the press, player 1 standing at `x` (in front or behind). Returns the tick
/// player 1 was hit on and the damage.
fn ground_spin(held: u32, x: Fx) -> Option<(u32, Fx)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, x, if x > Fx::ZERO { -1 } else { 1 });
    let before = sim.fighter(1).percent;
    for t in 1..=150u32 {
        let b = if t <= held { SPECIAL } else { 0 };
        sim.tick(inp(0, if t == 1 { 127 } else { 0 }, b));
        if t == 1 {
            assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
        }
        if sim.fighter(1).percent > before {
            return Some((t, sim.fighter(1).percent - before));
        }
    }
    None
}

#[test]
fn the_grounded_spin_is_one_hit_in_four_phases() {
    let sim = duel();
    let w = kit(&sim);
    let mv = w.get(MoveId::UpSpecial as u8);
    assert!(mv.hitboxes.iter().all(|h| h.group == 0), "one hit");
    assert_eq!(mv.charge_bonus, 60);
    assert_eq!(mv.air_version, Some(SPIN_AIR as u8));
    for h in &mv.hitboxes {
        let f = h.start + 1;
        let (_, _, blade, tip, angle, bkb, kbg) = GROUND_SPIN_PHASES
            .iter()
            .copied()
            .find(|p| p.0 <= f && f <= p.1)
            .expect("in a phase");
        let damage = if h.priority == 2 { tip } else { blade };
        assert_eq!(
            (h.damage, h.angle, h.base_knockback, h.knockback_growth),
            (Fx::from_ratio(damage, 10), angle, bkb, kbg),
            "frame {f} id {}",
            h.priority
        );
        assert_eq!(h.hitlag, if f <= 8 && h.priority < 2 { 140 } else { 100 });
    }
    // The tip is 0.8 of the blade in every phase.
    for (.., blade, tip, _, _, _) in GROUND_SPIN_PHASES {
        assert_eq!(tip * 10, blade * 8);
    }
    // A target in front, tapped (no hold past frame 2): the clean blade hit on frame 7, 14%.
    let (t, d) = ground_spin(1, fx(12, 10)).expect("hit");
    assert_eq!(t, 7);
    assert_eq!(d, dealt(&sim, 140));
    // Behind, the blade comes round on frame 9 (the mid phase, 12%) [E: the sweep].
    let (t, d) = ground_spin(1, fx(-12, 10)).expect("hit");
    assert_eq!(t, 9);
    assert_eq!(d, dealt(&sim, 120));
}

#[test]
fn holding_special_charges_the_grounded_spin_up_to_x1_6() {
    let sim = duel();
    // Held 31 ticks: frame 3 holds for 29 frames (charge 29), the hit 29 ticks later, x(1 + 0.6 x 29 / 60).
    let (t, d) = ground_spin(31, fx(12, 10)).expect("hit");
    assert_eq!(t, 7 + 29);
    let expect = Fx::from_int(14)
        * (Fx::ONE + Fx::from_ratio(29 * 60, 100 * 60))
        * sim.content.rules.damage_mult;
    assert!((d - expect).abs() < fx(1, 100), "{d:?} vs {expect:?}");
    // Held for ever: the charge stops at 60 frames, 22.4%.
    let (t, d) = ground_spin(150, fx(12, 10)).expect("hit");
    assert_eq!(t, 7 + 60);
    assert!((d - dealt(&sim, 224)).abs() < fx(1, 100), "{d:?}");
}

// ---- The aerial spin -------------------------------------------------------------------------------------------------------------

#[test]
fn the_aerial_spin_has_the_references_eight_hits_and_finisher() {
    let sim = duel();
    let mv = kit(&sim).get(SPIN_AIR as u8);
    assert!(mv.hitboxes.iter().all(|h| !h.clanks && !h.rebounds));
    for &(first, damage, angle, angle_type, fkv, set_weight, front) in &AIR_SPIN_HITS {
        let boxes: Vec<_> = mv
            .hitboxes
            .iter()
            .filter(|h| h.start + 1 == first)
            .collect();
        assert!(!boxes.is_empty(), "a hit on {first}");
        for h in boxes {
            assert_eq!(h.end, h.start + 1, "two frames");
            assert_eq!(
                (
                    h.damage,
                    h.angle,
                    h.angle_type,
                    h.fixed_kb,
                    h.set_weight,
                    h.sdi,
                    h.group
                ),
                (
                    Fx::from_ratio(damage, 10),
                    angle,
                    angle_type,
                    fkv,
                    set_weight,
                    30,
                    0
                ),
                "hit on {first}"
            );
            assert_eq!((h.base_knockback, h.knockback_growth), (0, 100));
            let posvec = (h.posvec_x, h.posvec_y, h.posvec_frames);
            if front {
                assert_eq!(posvec, (Fx::ZERO, Fx::ZERO, 0));
            } else {
                assert_eq!(
                    posvec,
                    (fx(10, 8), fx(20, 8), 10),
                    "hit on {first}: [10, 20] for 10 frames"
                );
                assert_eq!(h.angle_type, ANGLE_BACKWARD);
            }
        }
    }
    let total: i32 = AIR_SPIN_HITS.iter().map(|h| h.1).sum::<i32>() + 40;
    assert_eq!(total, 220, "22% in all");
    for f in [47u8, 48, 49] {
        for h in mv.hitboxes.iter().filter(|h| h.start + 1 == f) {
            assert_eq!(
                (
                    h.damage,
                    h.angle,
                    h.base_knockback,
                    h.knockback_growth,
                    h.hitlag,
                    h.group,
                    h.angle_type
                ),
                (Fx::from_int(4), 361, 30, 200, 200, 1, ANGLE_FORWARD)
            );
        }
    }
    assert_eq!((mv.ledge_front_from, mv.ledge_back_from), (14, 33));
    assert_eq!((mv.landing_lag, mv.fall_landing_lag), (30, 30));
    assert!(mv.helpless_after);
    assert_eq!(mv.fall_speed, Some(Fx::from_ratio(839_916, 8_000_000)));
    assert_eq!(mv.fall_accel, Some(fx(25, 8000)));
    assert_eq!(u16::from(mv.total_frames), 90, "animation 91");
}

/// Starts the aerial spin from a standstill high up, by input (stick up and special).
fn air_spin(sim: &mut Sim) {
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().move_id, SPIN_AIR as u8);
}

#[test]
fn the_aerial_spin_rises_on_the_references_speed_and_gravity_then_falls_helpless() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    air_spin(&mut sim);
    let y0 = sim.f().pos.y;
    let mut top = y0;
    let mut peak = 0u32;
    let mut helpless_at = None;
    for t in 1..=140u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().pos.y > top {
            top = sim.f().pos.y;
            peak = t;
        }
        if helpless_at.is_none() && sim.f().state == S::Helpless {
            helpless_at = Some(t);
        }
    }
    // The rise worked out from the parameters: 0.3 a frame less 0.0066 each frame after [D].
    let (mut v, mut h, mut n) = (SPIN_RISE_SPEED, Fx::ZERO, 0u32);
    while v > Fx::ZERO {
        h += v;
        v -= SPIN_GRAVITY;
        n += 1;
    }
    assert!(
        (top - y0 - h).abs() < fx(5, 100),
        "rose {:?}, expected {h:?}",
        top - y0
    );
    assert!(peak.abs_diff(n) <= 1, "peak on {peak}, expected {n}");
    // About 6.8 world units (54.5 of the reference's): the [D] height in `REFERENCE_DATA.md`.
    assert!((h - fx(68, 10)).abs() < fx(2, 10), "{h:?}");
    // Animation 91 (the move's frames 0 to 90, the last one moving as the spin), then the special fall.
    assert_eq!(helpless_at, Some(90));
}

#[test]
fn the_special_fall_after_the_spin_uses_its_own_air_speed_and_acceleration() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    let f = &mut sim.state.fighters[0];
    f.state = S::Helpless;
    f.move_id = SPIN_AIR as u8;
    let p = sim.content.fighters[usize::from(WANDERER)];
    sim.tick(inp(127, 0, 0));
    // One frame of full drift: the base acceleration plus the special fall's 0.025.
    assert_eq!(sim.f().vel.x, p.air_accel + fx(25, 8000));
    for _ in 0..200 {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(
        sim.f().vel.x,
        Fx::from_ratio(839_916, 8_000_000),
        "x0.909 of the air speed"
    );
}

#[test]
fn a_back_hit_carries_its_target_to_10_ahead_and_20_above_in_10_frames() {
    // E34: the target behind the spinning fighter at its back hit 1 (frame 12) ends 10 frames after the launch at the point [10, 20]
    // from where the fighter's feet were when it hit, in front of it.
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], SPIN_AIR);
    sim.state.fighters[0].state_frame = 10;
    let before = sim.fighter(1).percent;
    let mut at = None;
    for _ in 0..10 {
        let me = sim.f().pos;
        let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
        sim.put_airborne(
            1,
            me.x - fx(14, 10),
            me.y + fx(14, 10) - middle,
            Fx::ZERO,
            Fx::ZERO,
        );
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            at = Some(me);
            break;
        }
    }
    let me = at.expect("the back hit landed");
    assert_eq!(sim.fighter(1).posvec, 10);
    let point = me + Vec2::new(fx(10, 8), fx(20, 8));
    while sim.fighter(1).hitlag > 0 || sim.fighter(1).launch_pending {
        sim.tick(inp(0, 0, 0));
    }
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    let d = sim.fighter(1).pos - point;
    assert!(d.length() < fx(5, 100), "ended {d:?} from the point");
    assert_eq!(sim.fighter(1).posvec, 0);
    assert_eq!(sim.fighter(1).state, S::Hitstun);
}

#[test]
fn a_target_in_front_is_carried_through_the_front_hits_and_the_finisher() {
    // An airborne target in front at the first hit: the autolink (367) front hits carry it along with the rising fighter, so it takes
    // front hits 1-4 (4 + 2 + 2 + 2%) and the finisher (4%): 14%. The back hits are behind the fighter (the reference's boxes): they
    // catch a target there and carry it round to the front (the position vector, above). [The box positions are E.]
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
    sim.tick(inp(0, 127, SPECIAL));
    let before = sim.fighter(1).percent;
    let mut hits = Vec::new();
    let mut last = before;
    for _ in 0..140 {
        // (Held in front at the first hit's middle box until it lands; then left to the spin.)
        if hits.is_empty() {
            let me = sim.f().pos;
            sim.put_airborne(
                1,
                me.x + fx(14, 10),
                me.y + fx(13, 10) - middle,
                Fx::ZERO,
                Fx::ZERO,
            );
        }
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            hits.push(sim.f().state_frame + 1);
            last = sim.fighter(1).percent;
        }
    }
    assert_eq!(
        hits,
        vec![8, 16, 22, 31, 47],
        "the front hits and the finisher, on their reference frames"
    );
    assert!((last - before - dealt(&sim, 140)).abs() < fx(1, 100));
}

// ---- The passive shield (E14) --------------------------------------------------------------------------------------------------

/// The wanderer (player 0) at 0 facing `facing`, in `stance` (stick for that), and a claws fighter (player 1) at 7 firing its blaster
/// at it; returns player 0's percent change and whether it was frozen or pushed.
fn shot_at(facing: i8, stance: (i8, i8), tweak: impl Fn(&mut Content)) -> (Fx, bool, Sim) {
    let mut sim = Sim::with_chars([WANDERER, 1, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    tweak(&mut sim.content);
    sim.stand(0, Fx::ZERO, facing);
    sim.stand(1, Fx::from_int(7), -1);
    let before = sim.fighter(0).percent;
    let mut blocked = false;
    for t in 0..60 {
        let p1 = if t == 0 {
            inp(0, 0, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(stance.0, stance.1, 0), p1);
        if sim.fighter(0).shield_sdi {
            blocked = true;
        }
    }
    (sim.fighter(0).percent - before, blocked, sim)
}

#[test]
fn the_shield_stops_shots_from_in_front_standing_walking_and_crouching() {
    for stance in [(0i8, 0i8), (40, 0), (0, -127)] {
        let (damage, blocked, sim) = shot_at(1, stance, |_| {});
        assert_eq!(damage, Fx::ZERO, "stance {stance:?}");
        assert!(blocked, "stance {stance:?}: frozen in the block's hitlag");
        assert!(
            sim.state.projectiles.iter().all(|p| !p.active),
            "the shot is gone"
        );
    }
    // Pushed back away from the shot.
    let (_, _, sim) = shot_at(1, (0, 0), |_| {});
    assert!(
        sim.fighter(0).pos.x < Fx::ZERO,
        "pushed back: {:?}",
        sim.fighter(0).pos.x
    );
    // From behind it hits.
    let (damage, blocked, _) = shot_at(-1, (0, 0), |_| {});
    assert!(damage > Fx::ZERO && !blocked);
    // A small shot at head height passes over the shield and hits.
    let (damage, blocked, _) = shot_at(1, (0, 0), |c| {
        let spec = c.weapons[1].moves[MoveId::NSpecial as usize]
            .projectile
            .as_mut()
            .unwrap();
        spec.y = Fx::from_int(2);
        spec.hitbox.radius = Fx::from_ratio(1, 10);
    });
    assert!(damage > Fx::ZERO && !blocked, "head shot");
    // Another body (the duelist's) has no shield: the same shot hits.
    let mut sim = Sim::with_chars([0, 1, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(7), -1);
    for t in 0..60 {
        sim.tick2(
            inp(0, 0, 0),
            if t == 0 {
                inp(0, 0, SPECIAL)
            } else {
                inp(0, 0, 0)
            },
        );
    }
    assert!(sim.fighter(0).percent > Fx::ZERO);
}

#[test]
fn the_shield_is_down_while_attacking_or_dashing() {
    // In the up smash (an attack) or running toward the shot, it hits.
    for stance in [0u8, 1] {
        let mut sim = Sim::with_chars([WANDERER, 1, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_int(7), -1);
        if stance == 0 {
            sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::USmash);
        }
        for t in 0..60 {
            let p0 = if stance == 1 && t >= 12 {
                inp(127, 0, 0)
            } else {
                inp(0, 0, 0)
            };
            sim.tick2(
                p0,
                if t == 0 {
                    inp(0, 0, SPECIAL)
                } else {
                    inp(0, 0, 0)
                },
            );
            if sim.fighter(0).percent > Fx::ZERO {
                break;
            }
        }
        assert!(sim.fighter(0).percent > Fx::ZERO, "stance {stance}");
        assert!(!sim.fighter(0).shield_sdi);
    }
}

// ---- Up special and up smash out of a shield (E37) -----------------------------------------------------------------------------

/// Class `class` shielding (player 0), then `buttons` with the stick up on one tick: the move it is in after that tick.
fn out_of_shield(class: u8, buttons: u16) -> (S, MoveId, bool) {
    let mut sim = Sim::with_chars([class, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim.ticks(5, inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    sim.tick(inp(0, 127, SHIELD | buttons));
    (
        sim.f().state,
        MoveId::from_index(sim.f().move_id),
        sim.f().grounded(),
    )
}

#[test]
fn up_special_and_up_smash_come_straight_out_of_a_shield_for_every_class() {
    for class in [0u8, 1, 3, 6, WANDERER] {
        assert_eq!(
            out_of_shield(class, SPECIAL),
            (S::Attack, MoveId::UpSpecial, true),
            "class {class}: the grounded up special on the same frame"
        );
        assert_eq!(
            out_of_shield(class, ATTACK),
            (S::Attack, MoveId::USmash, true),
            "class {class}: the up smash on the same frame"
        );
    }
    // Attack with the stick neutral is still the shield grab; jump still jumps.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.ticks(5, inp(0, 0, SHIELD));
    sim.tick(inp(0, 0, SHIELD | ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Grab as u8);
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.ticks(5, inp(0, 0, SHIELD));
    sim.tick(inp(0, 0, SHIELD | JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
}

#[test]
fn the_grounded_spin_out_of_a_shield_hits_on_its_frame_7() {
    // UFD's out-of-shield option: 7 frames (the front hit), no jump squat.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim.ticks(5, inp(0, 0, SHIELD));
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 127, SHIELD | SPECIAL));
    let mut t = 1;
    while sim.fighter(1).percent == before && t < 30 {
        sim.tick(inp(0, 0, 0));
        t += 1;
    }
    assert_eq!(t, 7);
}

// ---- Content and determinism -----------------------------------------------------------------------------------------------------

/// A named change to the content.
type Change = (&'static str, fn(&mut Content));

#[test]
fn the_new_content_fields_change_the_content_hash() {
    let base = Content::placeholder();
    let h = base.hash();
    let w = usize::from(WANDERER);
    let changes: [Change; 9] = [
        ("posvec_x", |c| {
            c.weapons[9].moves[SPIN_AIR as usize].hitboxes[3].posvec_x += Fx::from_raw(1)
        }),
        ("posvec_frames", |c| {
            c.weapons[9].moves[SPIN_AIR as usize].hitboxes[3].posvec_frames += 1
        }),
        ("keep_charge", |c| {
            c.weapons[9].moves[FSMASH_2 as usize].keep_charge = false
        }),
        ("fall_speed", |c| {
            c.weapons[9].moves[SPIN_AIR as usize].fall_speed = None
        }),
        ("fall_accel", |c| {
            c.weapons[9].moves[SPIN_AIR as usize].fall_accel = None
        }),
        ("air block speed", |c| {
            c.weapons[9].moves[SPIN_AIR as usize].air_blocks[0].speed = None
        }),
        ("air block accel_stick", |c| {
            c.weapons[9].moves[SPIN_AIR as usize].air_blocks[0].accel_stick = None
        }),
        ("passive_shield", |c| c.fighters[9].passive_shield = None),
        ("passive_shield crouch_y", |c| {
            if let Some(s) = c.fighters[9].passive_shield.as_mut() {
                s.crouch_y += Fx::from_raw(1);
            }
        }),
    ];
    assert_eq!(w, 9);
    for (name, change) in changes {
        let mut c = base.clone();
        change(&mut c);
        assert_ne!(h, c.hash(), "{name}");
    }
}

#[test]
fn the_buckler_is_registered_as_class_9() {
    let c = Content::placeholder();
    assert_eq!(c.names.fighters[9], "wanderer");
    assert_eq!(c.names.weapons[9], "buckler");
    assert_eq!(c.fighters[9], FighterParams::wanderer());
    assert_eq!(c.weapons[9], sim_core::moves::buckler());
    assert!(!c.weapons[9].limbs, "a held sword");
    // The neutral, side and down specials are still the claws' placeholders (phases 2 and 3); the up special is the spin.
    for id in [MoveId::NSpecial, MoveId::SideSpecial, MoveId::DownSpecial] {
        assert_eq!(
            c.weapons[9].moves[id as usize], c.weapons[1].moves[id as usize],
            "{id:?}"
        );
    }
    assert_ne!(
        c.weapons[9].moves[MoveId::UpSpecial as usize],
        c.weapons[1].moves[MoveId::UpSpecial as usize]
    );
    // Only the four follow-up slots are used.
    for i in 0..MoveId::COUNT as u8 {
        let id = MoveId::from_index(i);
        if id.is_ext() {
            let used = [SPIN_AIR, FSMASH_2, DAIR_BOUNCE, DAIR_BOUNCE_END].contains(&id);
            assert_eq!(
                !c.weapons[9].moves[usize::from(i)].is_empty(),
                used,
                "{id:?}"
            );
        }
    }
    // Every box of every move is a normal hit or a grab, throw or pummel.
    for m in &c.weapons[9].moves {
        let _: &Move = m;
        assert!(m.hitboxes.iter().all(|h| h.kind <= 3));
    }
    let _ = HIT_NORMAL;
}

#[test]
fn random_play_with_the_wanderer_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [WANDERER, 1, WANDERER, 2];
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
