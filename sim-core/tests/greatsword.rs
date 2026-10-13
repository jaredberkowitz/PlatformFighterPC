//! The Greatsword moveset on the champion body (class 6), modelled mechanically on a reference character (phase 1: the body and
//! every normal; the specials are still the claws'). Frame data against the reference for every normal, checked in the data and by
//! running the moves; hit properties (damage, angles and angle types, knockback, set knockback and set weight, hitlag and SDI,
//! targets, shield damage, no-clank boxes); the jab chain's continuable windows and the held consecutive jab; the forward smash's
//! three hits; the meteors; the up air as one hit; grabs, pummel and the hit-then-throw throws; landing lag and autocancels; the
//! down tilt's slide; the body's attributes and jump air times; registration; determinism. Reference data and sources:
//! the seventh moveset's `REFERENCE_DATA.md` (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::intangible_parts;
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::{
    MoveId, Weapon, ANGLE_BACKWARD, ANGLE_BY_POSITION, ANGLE_FORWARD, HIT_GRAB, HIT_PUMMEL,
    HIT_THROW, PART_LIMB, TARGETS_AERIAL, TARGETS_ALL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const CHAMPION: u8 = 6;

/// The champion (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([CHAMPION, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn sword(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(CHAMPION)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R] except the pummel's frame and FAF, the down throw's release and FAF with its
/// hitlag folded in, [D]; the neutral air's one window 5-15 is split into the parts of its circle, [E] positions).
const FRAME_DATA: [FrameData; 26] = [
    (MoveId::Jab, &[(4, 5)], 30, 0, 0, 255),
    (MoveId::Jab2, &[(5, 6)], 33, 0, 0, 255),
    (MoveId::Jab3, &[(6, 7)], 38, 0, 0, 255),
    (MoveId::FTilt, &[(9, 10)], 36, 0, 0, 255),
    (MoveId::UTilt, &[(6, 8), (9, 9)], 33, 0, 0, 255),
    (MoveId::DTilt, &[(7, 8), (9, 17)], 41, 0, 0, 255),
    (MoveId::DashAttack, &[(9, 13), (14, 18)], 46, 0, 0, 255),
    (
        MoveId::FSmash,
        &[(19, 19), (24, 24), (28, 28)],
        67,
        0,
        0,
        255,
    ),
    (
        MoveId::USmash,
        &[(12, 12), (13, 14), (15, 16)],
        46,
        0,
        0,
        255,
    ),
    (MoveId::DSmash, &[(8, 9), (21, 23)], 50, 0, 0, 255),
    (
        MoveId::NAir,
        &[(5, 7), (8, 10), (11, 13), (14, 15)],
        39,
        9,
        1,
        31,
    ),
    (MoveId::FAir, &[(18, 19), (20, 25)], 54, 11, 2, 43),
    (MoveId::BAir, &[(11, 11), (12, 12)], 43, 8, 4, 37),
    (MoveId::UAir, &[(8, 8), (9, 9), (10, 23)], 45, 9, 1, 32),
    (MoveId::DAir, &[(11, 13), (14, 39)], 65, 16, 4, 46),
    (MoveId::Grab, &[(9, 10)], 37, 0, 0, 255),
    (MoveId::DashGrab, &[(12, 13)], 45, 0, 0, 255),
    (MoveId::PivotGrab, &[(13, 14)], 40, 0, 0, 255),
    // (The pummel: the reference's hit on frame 1 and FAF 7, one frame later here [D].)
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(7, 7), (9, 9)], 36, 0, 0, 255),
    (MoveId::BThrow, &[(13, 13), (15, 15)], 32, 0, 0, 255),
    (MoveId::UThrow, &[(8, 8), (13, 13), (15, 15)], 51, 0, 0, 255),
    // (The down throw: the reference's release 18 and FAF 44 plus its 10 frames of hitlag on frame 17 [D].)
    (MoveId::DThrow, &[(28, 28)], 54, 0, 0, 255),
    (MoveId::LedgeAttack, &[(20, 22)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(20, 21), (24, 25)], 46, 0, 0, 255),
    // (The specials are still placeholders: phase 2.)
    (MoveId::NSpecial, &[], 255, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = sword(&sim);
    for (id, windows, faf, landing, ac_before, ac_from) in FRAME_DATA {
        if faf == 255 {
            continue;
        }
        let mv = w.get(id as u8);
        for hb in &mv.hitboxes {
            assert!(
                windows
                    .iter()
                    .any(|&(a, b)| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: hitbox {}..{} (reference {}..{}) is in no window of {windows:?}",
                hb.start,
                hb.end,
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
    // Intangibility: the ledge attack through frame 22, the get-up attack through 25.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 22);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 25);
    // Smash charge frames (ssbwiki.com's first frame of "charges between"): forward 5, up 6, down 2 (move frames one less).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(4));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(5));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(1));
    // The jab chain's continuable windows: jab 1 into jab 2 from its frame 8, jab 2 into jab 3 from its 9 (move frames one less).
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 7));
    let jab2 = w.get(MoveId::Jab2 as u8);
    assert_eq!((jab2.next, jab2.next_from), (Some(MoveId::Jab3 as u8), 8));
    assert_eq!(w.get(MoveId::Jab3 as u8).next, None);
    // No normal is a multi-hit: nothing re-arms.
    for (id, ..) in FRAME_DATA {
        assert_eq!(w.get(id as u8).rehit, None, "{id:?}");
    }
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (in the air high up for an aerial, else standing), with player 1 held where the move's first hitbox of
/// window `first` (one that can hit an airborne fighter) will be, and runs until player 1 is hit. Returns the reference frame of the
/// hit.
fn first_hit_frame(id: MoveId, first: u8) -> Option<u8> {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let hb = *sword(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| h.start + 1 == first && h.targets != TARGETS_GROUNDED && !h.push)
        .expect("a hitbox in that window");
    let before = sim.fighter(1).percent;
    for tick in 1..=80u8 {
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
        (MoveId::Jab, 4u8),
        (MoveId::Jab2, 5),
        (MoveId::Jab3, 6),
        (MoveId::FTilt, 9),
        (MoveId::UTilt, 6),
        (MoveId::DTilt, 7),
        (MoveId::DashAttack, 9),
        (MoveId::FSmash, 19),
        (MoveId::USmash, 12),
        (MoveId::DSmash, 8),
        (MoveId::NAir, 5),
        (MoveId::FAir, 18),
        (MoveId::BAir, 11),
        (MoveId::UAir, 8),
        (MoveId::DAir, 11),
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
        if faf == 255
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
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority (the reference's id), damage in tenths, angle, BK, KS) for the boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 45] = [
        (MoveId::Jab, 4, 0, 25, 361, 35, 25),
        (MoveId::Jab, 4, 1, 25, 361, 35, 25),
        (MoveId::Jab, 4, 2, 25, 180, 20, 15),
        (MoveId::Jab2, 5, 0, 20, 361, 35, 25),
        (MoveId::Jab2, 5, 2, 20, 361, 25, 20),
        (MoveId::Jab3, 6, 0, 35, 30, 75, 50),
        (MoveId::FTilt, 9, 0, 110, 361, 20, 100),
        (MoveId::UTilt, 6, 0, 80, 95, 46, 100),
        (MoveId::UTilt, 6, 3, 80, 95, 46, 100),
        (MoveId::UTilt, 9, 2, 80, 95, 46, 100),
        (MoveId::DTilt, 7, 0, 70, 80, 80, 75),
        (MoveId::DTilt, 7, 1, 60, 80, 80, 75),
        (MoveId::DTilt, 9, 0, 70, 80, 80, 75),
        (MoveId::DashAttack, 9, 0, 110, 60, 100, 70),
        (MoveId::DashAttack, 9, 2, 110, 60, 100, 70),
        (MoveId::DashAttack, 14, 0, 80, 65, 100, 50),
        (MoveId::FSmash, 19, 0, 30, 60, 30, 30),
        (MoveId::FSmash, 19, 1, 30, 170, 0, 100),
        (MoveId::FSmash, 19, 2, 30, 0, 0, 100),
        (MoveId::FSmash, 19, 3, 30, 180, 0, 100),
        (MoveId::FSmash, 24, 0, 30, 15, 0, 100),
        (MoveId::FSmash, 24, 1, 20, 15, 0, 100),
        (MoveId::FSmash, 24, 2, 40, 170, 0, 100),
        (MoveId::FSmash, 28, 0, 130, 361, 42, 115),
        (MoveId::FSmash, 28, 1, 120, 361, 42, 115),
        (MoveId::USmash, 12, 1, 130, 83, 40, 105),
        (MoveId::USmash, 13, 0, 120, 83, 40, 96),
        (MoveId::USmash, 13, 3, 110, 83, 40, 90),
        (MoveId::USmash, 15, 0, 80, 60, 32, 100),
        (MoveId::DSmash, 8, 0, 30, 170, 0, 97),
        (MoveId::DSmash, 8, 1, 30, 155, 0, 97),
        (MoveId::DSmash, 8, 2, 30, 160, 0, 97),
        (MoveId::DSmash, 8, 3, 30, 150, 0, 97),
        (MoveId::DSmash, 21, 0, 110, 35, 35, 92),
        (MoveId::NAir, 5, 0, 80, 40, 25, 100),
        (MoveId::FAir, 18, 0, 140, 270, 20, 90),
        (MoveId::FAir, 18, 1, 140, 60, 20, 100),
        (MoveId::FAir, 18, 2, 130, 45, 30, 83),
        (MoveId::FAir, 20, 3, 110, 45, 30, 83),
        (MoveId::BAir, 11, 0, 130, 361, 20, 95),
        (MoveId::UAir, 8, 0, 110, 80, 42, 80),
        (MoveId::UAir, 10, 0, 80, 80, 42, 80),
        (MoveId::DAir, 11, 0, 150, 270, 10, 100),
        (MoveId::DAir, 11, 1, 130, 60, 30, 105),
        (MoveId::DAir, 14, 1, 80, 65, 80, 55),
    ];
    let sim = duel();
    let w = sword(&sim);
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
        (MoveId::Jab, 150),
        (MoveId::Jab2, 120),
        (MoveId::Jab3, 200),
        (MoveId::FTilt, 100),
        (MoveId::UTilt, 100),
        (MoveId::DTilt, 100),
        (MoveId::DashAttack, 100),
        (MoveId::NAir, 100),
        (MoveId::BAir, 100),
    ] {
        assert!(all(id, &|h| h.hitlag == hitlag), "{id:?}");
    }
    // The forward smash: hit 1 and 2 set knockback with set weight (FKV 25 / 10 / 10 / 10, then 80 / 50 / 10), hitlag x1.5, SDI x0
    // then x0.5; hit 1's near boxes on grounded targets, the others on aerial ones; hit 3 hitlag x1.8; three hit groups.
    let fsmash = get(MoveId::FSmash);
    for h in &fsmash.hitboxes {
        match h.start + 1 {
            19 => {
                assert_eq!(
                    (h.fixed_kb, h.set_weight, h.hitlag, h.sdi, h.group),
                    (if h.priority == 0 { 25 } else { 10 }, true, 150, 0, 0)
                );
                let grounded = h.priority < 2;
                assert_eq!(
                    h.targets,
                    if grounded {
                        TARGETS_GROUNDED
                    } else {
                        TARGETS_AERIAL
                    }
                );
            }
            24 => assert_eq!(
                (h.fixed_kb, h.set_weight, h.hitlag, h.sdi, h.group),
                ([80, 50, 10][usize::from(h.priority)], true, 150, 50, 1)
            ),
            _ => assert_eq!(
                (h.fixed_kb, h.set_weight, h.hitlag, h.group),
                (0, false, 180, 2)
            ),
        }
    }
    // The up smash: hitlag x1.2 and angle type 3 but the late hit (x1, by position); one hit group.
    for h in &get(MoveId::USmash).hitboxes {
        let late = h.start + 1 == 15;
        assert_eq!(
            (h.hitlag, h.angle_type, h.group),
            if late {
                (100, ANGLE_BY_POSITION, 0)
            } else {
                (120, ANGLE_FORWARD, 0)
            }
        );
    }
    // The down smash: hit 1 set knockback with set weight (FKV 90 near, 85 far), SDI x0.5, ids 0-1 aerial-only and 2-3 grounded-only,
    // sent forward; hit 2 behind, hitlag x1.4, sent backward.
    for h in &get(MoveId::DSmash).hitboxes {
        if h.group == 0 {
            assert_eq!(
                (h.fixed_kb, h.set_weight, h.sdi, h.angle_type),
                (
                    if h.priority % 2 == 0 { 90 } else { 85 },
                    true,
                    50,
                    ANGLE_FORWARD
                )
            );
            assert_eq!(
                h.targets,
                if h.priority < 2 {
                    TARGETS_AERIAL
                } else {
                    TARGETS_GROUNDED
                }
            );
        } else {
            assert!(h.x < Fx::ZERO);
            assert_eq!((h.hitlag, h.angle_type), (140, ANGLE_BACKWARD));
        }
    }
    // The dash attack's shield damage +1 (109 / 113).
    assert!(all(MoveId::DashAttack, &|h| h.shield_damage
        == if h.damage == Fx::from_int(11) {
            109
        } else {
            113
        }));
    // The forward air: the meteor at the tip on aerial targets, its 60-degree twin on grounded ones, hitlag x1.4 clean and x1 late.
    for h in &get(MoveId::FAir).hitboxes {
        let clean = h.start + 1 == 18;
        assert_eq!(h.hitlag, if clean { 140 } else { 100 });
        assert_eq!(
            h.targets,
            match (clean, h.priority) {
                (true, 0) => TARGETS_AERIAL,
                (true, 1) => TARGETS_GROUNDED,
                _ => TARGETS_ALL,
            }
        );
        assert_eq!(h.angle_type, ANGLE_FORWARD);
    }
    // The down air: the 270-degree meteor on aerial targets only, hitlag x1.5; the rest x1.2 on everyone; by position.
    for h in &get(MoveId::DAir).hitboxes {
        assert_eq!((h.angle == 270), (h.targets == TARGETS_AERIAL));
        assert_eq!(h.hitlag, if h.angle == 270 { 150 } else { 120 });
        assert_eq!(h.angle_type, ANGLE_BY_POSITION);
    }
    // The up air: clean x1.2, late x1.
    for h in &get(MoveId::UAir).hitboxes {
        assert_eq!(h.hitlag, if h.start + 1 < 10 { 120 } else { 100 });
    }
    // The back air sends backward; the get-up attack by position; the ledge attack forward.
    assert!(all(MoveId::BAir, &|h| h.angle_type == ANGLE_BACKWARD));
    assert!(all(MoveId::GetUpAttack, &|h| h.shield_damage == 214
        && h.angle_type == ANGLE_BY_POSITION));
    assert!(all(MoveId::LedgeAttack, &|h| h.shield_damage == 110
        && h.angle_type == ANGLE_FORWARD));
    let pummel = get(MoveId::Pummel).hitboxes[0];
    assert_eq!(
        (pummel.damage, pummel.fixed_kb, pummel.hitlag),
        (fx(13, 10), 30, 210)
    );
}

#[test]
fn only_the_references_no_clank_boxes_never_clank() {
    // ssbwiki.com's raw tables (13.0.1): `clang=f` on the pummel, the throws' hits, the ledge attack and the get-up attacks only.
    let sim = duel();
    let w = sword(&sim);
    for (id, ..) in FRAME_DATA {
        if id == MoveId::NSpecial {
            continue;
        }
        for h in &w.get(id as u8).hitboxes {
            let never = matches!(
                id,
                MoveId::Pummel | MoveId::LedgeAttack | MoveId::GetUpAttack
            ) || h.kind == HIT_PUMMEL;
            assert_eq!(h.clanks, !never, "{id:?}");
        }
    }
}

// ---- Jab -----------------------------------------------------------------------------------------------------------------------

/// Runs player 0's attack button down on the ticks in `down` (1-based) with player 1 in front (untouchable); returns (tick, move,
/// move frame, state) after each tick for `ticks` ticks.
fn jab_run(down: &[u32], ticks: u32) -> Vec<(u32, MoveId, u16, S)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim.state.fighters[1].invuln = 255;
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

#[test]
fn the_quickest_jab_chain_runs_on_the_references_frames() {
    // Pressed on 1, 3 and 10: jab 1 from tick 1 (hits 4-5), jab 2 from 8 (jab 1's frame 8, its continuable window; hits 12-13), jab 3
    // from 16 (jab 2's frame 9; hits 21-22) and control back on 53. (Tick t of the run is frame t from jab 1's first frame: the
    // reference's quickest loop: 4-5, 12-13, 21-22, FAF 30 / 40 / 53.)
    let run = jab_run(&[1, 3, 10], 70);
    assert_eq!(started(&run, MoveId::Jab), Some(1));
    assert_eq!(started(&run, MoveId::Jab2), Some(8));
    assert_eq!(started(&run, MoveId::Jab3), Some(16));
    let sim = duel();
    let w = sword(&sim);
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
    assert_eq!(out_on(MoveId::Jab), vec![4, 5]);
    assert_eq!(out_on(MoveId::Jab2), vec![12, 13]);
    assert_eq!(out_on(MoveId::Jab3), vec![21, 22]);
    let back = run.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1);
    assert_eq!(back, Some(53), "jab 3's FAF, counted from jab 1");
    // Jab 1 alone (one press) gives control back on 30; jab 1 and 2 on 40.
    let one = jab_run(&[1], 40);
    assert_eq!(
        one.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1),
        Some(30)
    );
    let two = jab_run(&[1, 3], 50);
    assert_eq!(started(&two, MoveId::Jab3), None);
    assert_eq!(
        two.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1),
        Some(40)
    );
}

#[test]
fn a_press_before_the_continuable_window_waits_for_it() {
    // Pressed again on jab 1's frame 3 (before its window at 8): jab 2 still starts on frame 8, not sooner.
    let run = jab_run(&[1, 3], 12);
    assert_eq!(started(&run, MoveId::Jab2), Some(8));
    assert_eq!(run[6].1, MoveId::Jab, "frame 7: still jab 1");
}

#[test]
fn holding_attack_throws_jab_1_again_from_frame_16() {
    // Held from the press: jab 1 starts over on its frame 16 (the reference's consecutive jab frame) and never goes on to jab 2.
    let run = jab_run(&(1..=60).collect::<Vec<_>>(), 60);
    let restarts: Vec<u32> = run
        .iter()
        .filter(|(_, m, f, s)| *m == MoveId::Jab && *f == 0 && *s == S::Attack)
        .map(|r| r.0)
        .collect();
    assert_eq!(&restarts[..4], &[1, 16, 31, 46]);
    assert!(started(&run, MoveId::Jab2).is_none());
}

// ---- Single hits, sweetspots and meteors -------------------------------------------------------------------------------------

/// Player 0 does `id` from move frame 0 (in the air for an aerial) with a duelist held in the air at `(dx, dy)` from it (world
/// tenths) every frame until the move ends; returns the reference frames (of player 0's move) of each hit and the percent dealt.
fn hits_on_a_held_target(id: MoveId, dx: i32, dy: i32) -> (Vec<u16>, Fx) {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let mut hits = Vec::new();
    let mut last = Fx::ZERO;
    for _ in 0..90 {
        let me = sim.state.fighters[0].pos;
        let p = sim.state.fighters[1];
        sim.put_airborne(1, me.x + fx(dx, 10), me.y + fx(dy, 10), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].percent = p.percent;
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            hits.push(sim.f().state_frame + 1);
            last = sim.fighter(1).percent;
        }
        if sim.f().state != S::Attack {
            break;
        }
    }
    (hits, last)
}

#[test]
fn the_up_air_is_one_hit_strong_early_and_weak_late() {
    // 13.0.1: one hit (not a multi-hit), 11% on frames 8-9, 8% from 10.
    let sim = duel();
    let (hits, dmg) = hits_on_a_held_target(MoveId::UAir, 0, 20);
    assert_eq!((hits.len(), dmg), (1, dealt(&sim, 110)), "{hits:?}");
    // The same target arriving late (the move started 3 frames earlier) takes the late hit only.
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
    sim.state.fighters[0].state_frame = 10;
    let mut total = Fx::ZERO;
    for _ in 0..30 {
        let me = sim.f().pos;
        sim.put_airborne(1, me.x + fx(4, 10), me.y + fx(18, 10), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].percent = total;
        sim.tick(inp(0, 0, 0));
        total = sim.fighter(1).percent;
    }
    assert_eq!(total, dealt(&sim, 80));
}

#[test]
fn the_forward_smash_hits_three_times_on_frames_19_24_and_28() {
    // A duelist in the air in front of the blade: hit 1 (3%, pulls in), hit 2 and hit 3 each land, on their frames.
    let (hits, _) = hits_on_a_held_target(MoveId::FSmash, 12, 2);
    assert_eq!(hits, vec![19, 24, 28]);
}

#[test]
fn the_down_smash_hits_in_front_then_behind() {
    let (front, _) = hits_on_a_held_target(MoveId::DSmash, 15, 0);
    assert_eq!(front, vec![8]);
    let (behind, _) = hits_on_a_held_target(MoveId::DSmash, -9, 0);
    assert_eq!(behind, vec![21]);
}

/// The launch velocity of player 1 hit by player 0's `id` (on move frame `frame`, player 0 facing right, in the air for an aerial and
/// standing otherwise) with player 1 held in the air at `(dx, dy)` world tenths from it.
fn launch_of(id: MoveId, frame: u16, dx: i32, dy: i32) -> Vec2 {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    sim.state.fighters[0].state_frame = frame;
    sim.state.fighters[1].percent = Fx::from_int(80);
    let me = sim.f().pos;
    sim.put_airborne(1, me.x + fx(dx, 10), me.y + fx(dy, 10), Fx::ZERO, Fx::ZERO);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Hitstun, "{id:?} hit");
    sim.fighter(1).kb_vel
}

#[test]
fn the_forward_and_down_airs_meteor_aerial_targets() {
    // The forward air's tip on frame 18 and the down air's tip on frame 11: straight down.
    let fair = launch_of(MoveId::FAir, 17, 18, 1);
    assert!(fair.y < Fx::ZERO && fair.x == Fx::ZERO, "{fair:?}");
    let dair = launch_of(MoveId::DAir, 10, 0, -15);
    assert!(dair.y < Fx::ZERO && dair.x == Fx::ZERO, "{dair:?}");
    // The down air's late hit sends up and away instead.
    let late = launch_of(MoveId::DAir, 20, 0, -12);
    assert!(late.y > Fx::ZERO, "{late:?}");
}

#[test]
fn the_back_air_and_the_down_smashs_second_hit_send_backward() {
    assert!(launch_of(MoveId::BAir, 10, -14, -2).x < Fx::ZERO);
    assert!(launch_of(MoveId::DSmash, 20, -9, 2).x < Fx::ZERO);
}

#[test]
fn the_down_tilt_slides_forward_and_records_the_back_legs_window() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DTilt);
    sim.ticks(40, inp(0, 0, 0));
    // 12 frames at 1/8 a frame, then the ground's friction [E].
    assert!(sim.f().pos.x >= fx(15, 10), "slid {:?}", sim.f().pos.x);
    assert!(sim.f().grounded());
    // The back leg's intangibility (7-17) is recorded on the limb circle; with a held sword it acts on nothing (a listed limit).
    let mut f = *sim.f();
    f.state = S::Attack;
    f.move_id = MoveId::DTilt as u8;
    for (frame, bits) in [(5u16, 0u8), (6, PART_LIMB), (16, PART_LIMB), (17, 0)] {
        f.state_frame = frame;
        assert_eq!(intangible_parts(&f, sword(&sim)), bits, "frame {frame}");
    }
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
fn the_standing_grab_catches_on_frame_9() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    assert_eq!(grab_now(&mut sim), 9);
    for id in [MoveId::Grab, MoveId::DashGrab, MoveId::PivotGrab] {
        assert!(sword(&sim)
            .get(id as u8)
            .hitboxes
            .iter()
            .all(|h| h.kind == HIT_GRAB));
    }
}

#[test]
fn the_throws_hit_then_release_on_their_frames_for_their_damage() {
    // (stick, the frames the held fighter's percent rises on, total damage in tenths)
    for (stick, frames, damage) in [
        ((127i8, 0i8), vec![7u8, 9], 70),
        ((-127, 0), vec![13, 15], 60),
        ((0, 127), vec![8, 13, 15], 85),
        ((0, -127), vec![28], 70),
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
        assert_eq!(
            sim.fighter(1).percent - before,
            dealt(&sim, damage),
            "throw {stick:?}"
        );
        if stick.0 < 0 {
            // The back throw: away behind, and the thrower ends turned round.
            assert!(sim.fighter(1).pos.x < Fx::ZERO, "thrown behind");
            assert_eq!(sim.f().facing, -1, "facing the throw");
        }
        if stick.1 < 0 {
            // The down throw's 122 degrees send behind the thrower.
            assert!(sim.fighter(1).pos.x < fx(12, 10), "sent back");
        }
    }
    let sim = duel();
    let w = sword(&sim);
    for id in [MoveId::FThrow, MoveId::BThrow, MoveId::UThrow] {
        let kinds: Vec<u8> = w.get(id as u8).hitboxes.iter().map(|h| h.kind).collect();
        assert_eq!(kinds.last(), Some(&HIT_THROW), "{id:?}");
        assert!(kinds[..kinds.len() - 1].iter().all(|&k| k == HIT_PUMMEL));
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
    let plain = duel().content.fighters[usize::from(CHAMPION)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window if it can be
    // reached by falling; the tick adds one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 10u16, 9u8, 30u16, None),
        (MoveId::FAir, 30, 11, 42, Some(0u16)),
        (MoveId::BAir, 20, 8, 36, Some(2)),
        (MoveId::UAir, 20, 9, 31, None),
        (MoveId::DAir, 30, 16, 45, Some(2)),
    ] {
        assert_eq!(land_during(id, mid), (S::Landing, lag), "{id:?} mid-move");
        assert_eq!(land_during(id, late), (S::Landing, plain), "{id:?} late");
        if let Some(early) = early {
            assert_eq!(land_during(id, early).1, plain, "{id:?} early");
        }
    }
}

// ---- Body ----------------------------------------------------------------------------------------------------------------------

#[test]
fn the_champions_jumps_have_the_references_air_times() {
    // Short hop 37, full hop 49, short hop fast fall 26, full hop fast fall 34 (ultimateframedata.com); within one frame.
    let air_time = |full: bool, ff: bool| {
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
    };
    for (full, ff, reference) in [
        (false, false, 37u32),
        (true, false, 49),
        (false, true, 26),
        (true, true, 34),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_champion_is_the_references_middleweight() {
    let p = duel().content.fighters[usize::from(CHAMPION)];
    assert_eq!(p, FighterParams::champion());
    let su = FighterParams::su;
    assert_eq!(p.weight, Fx::from_int(100));
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(1155), su(98), su(1680), su(2688))
    );
    assert_eq!(
        (p.air_accel, p.air_accel_stick, p.air_friction),
        (su(10), su(60), su(9))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (47, 64, 79, 104)
    );
    assert_eq!(
        (
            p.air_dodge_intangible_start,
            p.air_dodge_intangible_end,
            p.air_dodge_dir_intangible_end
        ),
        (3, 30, 21)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (29, 34, 25)
    );
    assert_eq!((p.dash_frames, p.heavy_landing_lag), (10, 3));
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (1, su(1300), su(2400))
    );
    assert_eq!(p.ledge_jump_dx, su(600));
    assert_eq!(p.weapon, CHAMPION);
    assert!(!sword(&duel()).limbs, "a held sword");
}

#[test]
fn the_greatsword_is_registered_as_class_6() {
    let c = Content::placeholder();
    assert_eq!(c.names.fighters[6], "champion");
    assert_eq!(c.names.weapons[6], "greatsword");
    assert_eq!(c.fighters[6], FighterParams::champion());
    assert_eq!(c.weapons[6], sim_core::moves::greatsword());
    // The specials are still the claws' (phases 2 and 3).
    for id in [
        MoveId::NSpecial,
        MoveId::SideSpecial,
        MoveId::UpSpecial,
        MoveId::DownSpecial,
    ] {
        assert_eq!(
            c.weapons[6].moves[id as usize], c.weapons[1].moves[id as usize],
            "{id:?}"
        );
    }
}

// ---- Determinism ---------------------------------------------------------------------------------------------------------------

#[test]
fn random_play_with_the_champion_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [CHAMPION, 0, CHAMPION, 5];
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
