//! The Gauntlets moveset on the sprinter body (class 4), modelled mechanically on a reference character (phase 1: the body and every
//! normal; the specials are still the claws'). Frame data against the reference for every normal, checked in the data and by running
//! the moves; hit properties; the jab chain, the held repeat and the rapid jab's entry rule (presses counted through hitlag), its loop
//! and finisher; the angled forward tilt and smash; the knee's one-frame sweetspot; the down air's meteor; per-limb intangibility
//! (the up tilt's and up smash's legs) and per-hitbox shieldstun (the dash attack and rapid jab); grabs, pummel and throws; landing
//! lag and autocancels; the body's attributes and jump air times; and determinism. Reference data and sources:
//! `docs/movesets/captain_falcon/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::{active_hitboxes, hurtboxes, intangible_parts, resolve_hits};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::{
    MoveId, Weapon, AIM_ANY, AIM_DOWN, AIM_STRAIGHT, AIM_UP, EFFECT_ELECTRIC, HIT_GRAB, PART_LIMB,
    TARGETS_AERIAL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const SPRINTER: u8 = 4;

/// The rapid jab's first cycle, later cycles and finisher.
const RAPID: MoveId = MoveId::Ext10;
const RAPID_LOOP: MoveId = MoveId::Ext11;
const RAPID_END: MoveId = MoveId::Ext12;

/// The sprinter (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SPRINTER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn gauntlets(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SPRINTER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R] except the pummel's frame and FAF and the rapid jab's later cycles, [D]).
const FRAME_DATA: [FrameData; 28] = [
    (MoveId::Jab, &[(3, 3)], 18, 0, 0, 255),
    (MoveId::Jab2, &[(5, 5)], 19, 0, 0, 255),
    (MoveId::Jab3, &[(6, 7)], 33, 0, 0, 255),
    (
        RAPID,
        &[
            (5, 5),
            (7, 7),
            (9, 9),
            (11, 11),
            (13, 13),
            (15, 15),
            (17, 17),
            (19, 19),
        ],
        22,
        0,
        0,
        255,
    ),
    // (The later cycles start on the cycle's frame 3: the same hits, two frames earlier in the move.)
    (
        RAPID_LOOP,
        &[
            (3, 3),
            (5, 5),
            (7, 7),
            (9, 9),
            (11, 11),
            (13, 13),
            (15, 15),
            (17, 17),
        ],
        20,
        0,
        0,
        255,
    ),
    (RAPID_END, &[(6, 7)], 50, 0, 0, 255),
    (MoveId::FTilt, &[(7, 10)], 30, 0, 0, 255),
    (MoveId::UTilt, &[(14, 17)], 37, 0, 0, 255),
    (MoveId::DTilt, &[(11, 12)], 35, 0, 0, 255),
    (MoveId::DashAttack, &[(7, 9), (10, 16)], 35, 0, 0, 255),
    (MoveId::FSmash, &[(19, 22)], 60, 0, 0, 255),
    (
        MoveId::USmash,
        &[(22, 23), (22, 22), (28, 29)],
        46,
        0,
        0,
        255,
    ),
    (MoveId::DSmash, &[(19, 20), (29, 30)], 49, 0, 0, 255),
    (MoveId::NAir, &[(7, 8), (13, 15)], 40, 7, 3, 32),
    (MoveId::FAir, &[(14, 14), (15, 30)], 46, 18, 4, 42),
    (MoveId::BAir, &[(10, 11), (12, 15)], 36, 10, 4, 19),
    (MoveId::UAir, &[(7, 10), (11, 12)], 32, 10, 0, 24),
    (MoveId::DAir, &[(16, 18)], 45, 12, 3, 39),
    (MoveId::Grab, &[(6, 7)], 36, 0, 0, 255),
    (MoveId::DashGrab, &[(9, 10)], 44, 0, 0, 255),
    (MoveId::PivotGrab, &[(10, 11)], 39, 0, 0, 255),
    // (The pummel: the reference's hit on frame 1 and FAF 7, one frame later here [D].)
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(11, 11), (13, 13)], 33, 0, 0, 255),
    (MoveId::BThrow, &[(12, 12), (13, 13)], 46, 0, 0, 255),
    (MoveId::UThrow, &[(12, 12), (14, 14)], 38, 0, 0, 255),
    (MoveId::DThrow, &[(21, 21)], 40, 0, 0, 255),
    (MoveId::LedgeAttack, &[(22, 24)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(14, 15), (20, 21)], 46, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = gauntlets(&sim);
    for (id, windows, faf, landing, ac_before, ac_from) in FRAME_DATA {
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
    assert_eq!(
        sim.content.fighters[usize::from(SPRINTER)].ledge_attack_frames,
        55
    );
    // Intangibility: the ledge attack through frame 24, the get-up attack through 21.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 24);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 21);
    // Smash charge frames: forward on 11, up on 8, down on 11 (move frames one less).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(10));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(7));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(10));
    // The jab chain: jab 1 continues into jab 2 from frame 5 (a script then picks the rapid jab or jab 3).
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 4));
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (in the air high up for an aerial, else standing; aimed `aim`), with player 1 held where the move's
/// first hitbox of window `first` will be, and runs until player 1 is hit. Returns the reference frame of the hit.
fn first_hit_frame(id: MoveId, first: u8, aim: i8) -> Option<u8> {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    sim.state.fighters[0].aim = aim;
    let wanted = match aim {
        1 => AIM_UP,
        -1 => AIM_DOWN,
        _ => AIM_STRAIGHT,
    };
    let hb = *gauntlets(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| {
            h.start + 1 == first
                && (h.aim == AIM_ANY || h.aim == wanted)
                && h.targets != TARGETS_GROUNDED
        })
        .expect("a hitbox in that window");
    let before = sim.fighter(1).percent;
    for tick in 1..=80u8 {
        let me = sim.state.fighters[0].pos;
        let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
        let at = Vec2::new(me.x + hb.x, (me.y + hb.y - middle).max(Fx::ZERO));
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
    for (id, first, aim) in [
        (MoveId::Jab, 3u8, 0i8),
        (MoveId::Jab2, 5, 0),
        (MoveId::Jab3, 6, 0),
        (RAPID, 5, 0),
        (RAPID_END, 6, 0),
        (MoveId::FTilt, 7, 0),
        (MoveId::FTilt, 7, 1),
        (MoveId::FTilt, 7, -1),
        (MoveId::UTilt, 14, 0),
        (MoveId::DTilt, 11, 0),
        (MoveId::DashAttack, 7, 0),
        (MoveId::FSmash, 19, 0),
        (MoveId::FSmash, 19, 1),
        (MoveId::FSmash, 19, -1),
        (MoveId::USmash, 22, 0),
        (MoveId::DSmash, 19, 0),
        (MoveId::DSmash, 29, 0),
        (MoveId::NAir, 7, 0),
        (MoveId::FAir, 14, 0),
        (MoveId::BAir, 10, 0),
        (MoveId::UAir, 7, 0),
        (MoveId::DAir, 16, 0),
    ] {
        assert_eq!(
            first_hit_frame(id, first, aim),
            Some(first),
            "{id:?} window from {first}, aim {aim}"
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
        // (Throws need someone held, the ledge attack a ledge; the rapid jab's cycles hand on by themselves: checked below.)
        if matches!(
            id,
            MoveId::Pummel
                | MoveId::FThrow
                | MoveId::BThrow
                | MoveId::UThrow
                | MoveId::DThrow
                | MoveId::LedgeAttack
                | RAPID
                | RAPID_LOOP
        ) {
            continue;
        }
        assert_eq!(faf_of(id, id.is_aerial()), u16::from(faf), "{id:?}");
    }
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority, damage in tenths, angle, BK, KS) for the main boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 37] = [
        (MoveId::Jab, 3, 0, 15, 361, 20, 25),
        (MoveId::Jab, 3, 1, 15, 180, 20, 20),
        (MoveId::Jab2, 5, 2, 15, 361, 20, 20),
        (MoveId::Jab3, 6, 0, 50, 361, 70, 100),
        (RAPID, 5, 0, 6, 361, 8, 15),
        (RAPID, 5, 1, 6, 361, 8, 15),
        (RAPID_END, 6, 0, 30, 45, 110, 55),
        (MoveId::FTilt, 7, 0, 80, 361, 60, 58),
        (MoveId::FTilt, 7, 2, 90, 361, 60, 58),
        (MoveId::UTilt, 14, 0, 110, 285, 35, 100),
        (MoveId::UTilt, 14, 1, 110, 285, 22, 90),
        (MoveId::DTilt, 11, 0, 100, 25, 38, 86),
        (MoveId::DTilt, 11, 2, 100, 155, 38, 86),
        (MoveId::DashAttack, 7, 0, 100, 65, 90, 67),
        (MoveId::DashAttack, 10, 0, 60, 65, 90, 67),
        (MoveId::FSmash, 19, 0, 190, 43, 36, 94),
        (MoveId::USmash, 22, 0, 70, 110, 0, 90),
        (MoveId::USmash, 22, 2, 120, 80, 0, 90),
        (MoveId::USmash, 28, 0, 140, 83, 70, 80),
        (MoveId::USmash, 28, 2, 130, 70, 70, 88),
        (MoveId::DSmash, 19, 0, 140, 28, 30, 90),
        (MoveId::DSmash, 29, 0, 180, 28, 20, 84),
        (MoveId::NAir, 7, 0, 40, 74, 50, 25),
        (MoveId::NAir, 7, 1, 40, 68, 65, 75),
        (MoveId::NAir, 13, 0, 60, 361, 45, 120),
        (MoveId::FAir, 14, 0, 220, 32, 30, 81),
        (MoveId::FAir, 14, 1, 60, 361, 35, 80),
        (MoveId::FAir, 15, 0, 30, 361, 35, 80),
        (MoveId::BAir, 10, 0, 130, 361, 27, 97),
        (MoveId::BAir, 12, 0, 80, 361, 27, 97),
        (MoveId::UAir, 7, 0, 100, 66, 20, 96),
        (MoveId::UAir, 11, 0, 90, 30, 12, 80),
        (MoveId::DAir, 16, 0, 140, 270, 10, 100),
        (MoveId::DAir, 16, 1, 140, 361, 40, 100),
        (MoveId::DThrow, 21, 0, 60, 68, 65, 70),
        (MoveId::LedgeAttack, 22, 0, 100, 45, 90, 20),
        (MoveId::GetUpAttack, 14, 0, 60, 48, 80, 48),
    ];
    let sim = duel();
    let w = gauntlets(&sim);
    for (id, first, priority, damage, angle, bk, ks) in table {
        let hb = w
            .get(id as u8)
            .hitboxes
            .iter()
            .find(|h| {
                h.start + 1 == first
                    && h.priority == priority
                    && h.aim != AIM_UP
                    && h.aim != AIM_DOWN
            })
            .unwrap_or_else(|| panic!("{id:?} {first} {priority}"));
        assert_eq!(
            (hb.damage, hb.angle, hb.base_knockback, hb.knockback_growth),
            (Fx::from_ratio(damage, 10), angle, bk, ks),
            "{id:?} from {first}, box {priority}"
        );
    }
    let get = |id: MoveId| w.get(id as u8);
    // Hitlag, hitstun, SDI, shieldstun, set knockback and set weight, targets, shield damage.
    assert!(get(MoveId::Jab).hitboxes.iter().all(|h| h.hitlag == 160));
    assert!(get(MoveId::Jab2)
        .hitboxes
        .iter()
        .all(|h| h.hitlag == 120 && h.hitstun_add == 3));
    assert!(get(MoveId::Jab3).hitboxes.iter().all(|h| h.hitlag == 200));
    for id in [RAPID, RAPID_LOOP] {
        assert!(get(id)
            .hitboxes
            .iter()
            .all(|h| (h.hitlag, h.sdi, h.hitstun_add, h.shieldstun) == (50, 40, 2, 900)));
    }
    assert!(get(MoveId::DashAttack)
        .hitboxes
        .iter()
        .all(|h| h.hitlag == 125 && h.shieldstun == 150));
    assert_eq!(
        get(MoveId::DashAttack)
            .hitboxes
            .iter()
            .map(|h| h.shield_damage)
            .collect::<Vec<_>>(),
        vec![110, 110, 117, 117]
    );
    let utilt = get(MoveId::UTilt);
    assert_eq!(
        (utilt.hitboxes[0].targets, utilt.hitboxes[1].targets),
        (TARGETS_GROUNDED, TARGETS_AERIAL)
    );
    assert_eq!(utilt.hitboxes[1].hitlag, 140);
    let usmash = get(MoveId::USmash);
    assert_eq!(
        usmash
            .hitboxes
            .iter()
            .map(|h| h.fixed_kb)
            .collect::<Vec<_>>(),
        vec![150, 80, 0, 10, 0, 0, 0]
    );
    let nair = get(MoveId::NAir);
    assert!(nair.hitboxes.iter().all(|h| h.set_weight == (h.group == 0)));
    assert!(nair
        .hitboxes
        .iter()
        .all(|h| (h.sdi == 140) == (h.targets == TARGETS_GROUNDED)));
    let fair = get(MoveId::FAir);
    assert_eq!(fair.hitboxes[0].effect, EFFECT_ELECTRIC);
    assert!(fair.hitboxes[1..]
        .iter()
        .all(|h| h.effect != EFFECT_ELECTRIC));
    assert_eq!(
        (fair.hitboxes[0].hitlag, fair.hitboxes[1].hitlag),
        (120, 120)
    );
    let pummel = get(MoveId::Pummel).hitboxes[0];
    assert_eq!(
        (pummel.damage, pummel.fixed_kb, pummel.hitlag),
        (fx(13, 10), 30, 210)
    );
    assert!(get(MoveId::GetUpAttack)
        .hitboxes
        .iter()
        .all(|h| h.shield_damage == 233));
    assert!(get(MoveId::LedgeAttack)
        .hitboxes
        .iter()
        .all(|h| h.shield_damage == 110));
    // The angled forward smash: 20% at 45 degrees aimed up, 20% at 38 (KS 87) aimed down.
    let elbow = |aim: u8| {
        let h = get(MoveId::FSmash)
            .hitboxes
            .iter()
            .find(|h| h.aim == aim && h.priority == 0)
            .unwrap();
        (h.damage, h.angle, h.knockback_growth)
    };
    assert_eq!(elbow(AIM_STRAIGHT), (Fx::from_int(19), 43, 94));
    assert_eq!(elbow(AIM_UP), (Fx::from_int(20), 45, 94));
    assert_eq!(elbow(AIM_DOWN), (Fx::from_int(20), 38, 87));
    // The angled forward tilt does 1% more on every box.
    for aim in [AIM_UP, AIM_DOWN] {
        let straight: Vec<Fx> = get(MoveId::FTilt)
            .hitboxes
            .iter()
            .filter(|h| h.aim == AIM_STRAIGHT)
            .map(|h| h.damage)
            .collect();
        let angled: Vec<Fx> = get(MoveId::FTilt)
            .hitboxes
            .iter()
            .filter(|h| h.aim == aim)
            .map(|h| h.damage - Fx::ONE)
            .collect();
        assert_eq!(straight, angled);
    }
}

// ---- Jab -----------------------------------------------------------------------------------------------------------------------

/// Runs player 0's attack button from `presses` (tick numbers, 1-based, on which attack is down) with player 1 untouchable in front;
/// returns (tick, move, move frame) after each tick for `ticks` ticks.
fn jab_run(down: &[u32], ticks: u32) -> Vec<(u32, MoveId, u16, S)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(14, 10), -1);
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
fn the_quickest_rapid_jab_runs_on_the_references_frames() {
    // Pressed on 1, 3 and 5 (three presses, two releases), then let go: jab 1 from tick 1, jab 2 from 5 (jab 1's frame 5), the rapid
    // jab from 11 (jab 2's frame 7), its hits on 15, 17 ... 29, the finisher from 30 (the cycle's frame 20) with its hit on 35-36 and
    // control back on 79. (Tick t of the run is frame t from jab 1's first frame: the reference's quickest loop.)
    let run = jab_run(&[1, 3, 5], 90);
    assert_eq!(started(&run, MoveId::Jab), Some(1));
    assert_eq!(started(&run, MoveId::Jab2), Some(5));
    assert_eq!(started(&run, RAPID), Some(11));
    assert_eq!(started(&run, RAPID_END), Some(30));
    assert_eq!(started(&run, MoveId::Jab3), None, "no jab 3");
    let sim = duel();
    let w = gauntlets(&sim);
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
    assert_eq!(out_on(MoveId::Jab), vec![3]);
    assert_eq!(out_on(MoveId::Jab2), vec![9]);
    assert_eq!(out_on(RAPID), vec![15, 17, 19, 21, 23, 25, 27, 29]);
    assert_eq!(out_on(RAPID_END), vec![35, 36]);
    let back = run.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1);
    assert_eq!(back, Some(79), "the finisher's FAF, counted from jab 1");
}

#[test]
fn a_press_after_the_rapid_window_gives_jab_3_and_holding_gives_no_rapid_jab() {
    // Two presses (the second held through jab 2's rapid window), then a third after it: jab 3, starting on that press.
    let mut down: Vec<u32> = vec![1];
    down.extend(3..=14);
    down.push(16);
    let run = jab_run(&down, 60);
    assert_eq!(started(&run, MoveId::Jab2), Some(5));
    assert_eq!(
        started(&run, RAPID),
        None,
        "only one release before frame 9"
    );
    assert_eq!(
        started(&run, MoveId::Jab3),
        Some(16),
        "jab 3 on the new press"
    );
    // A held press is no new press: jab 2 then runs out.
    let run = jab_run(&[1, 3, 4, 5, 6, 7, 8, 9], 60);
    assert_eq!(started(&run, MoveId::Jab2), Some(5));
    assert_eq!(started(&run, MoveId::Jab3), None);
    assert_eq!(started(&run, RAPID), None);
}

#[test]
fn holding_attack_throws_jab_1_again_from_frame_7() {
    let run = jab_run(&(1..=20).collect::<Vec<_>>(), 20);
    assert_eq!(run[5].1, MoveId::Jab);
    assert_eq!(
        (run[6].1, run[6].2),
        (MoveId::Jab, 0),
        "held, jab 1 starts over on its frame 7"
    );
    assert!(run.iter().all(|r| r.1 == MoveId::Jab));
}

#[test]
fn presses_during_hitlag_count_toward_the_rapid_jab() {
    // Jab 1 hits a real target on frame 3 and both freeze for its hitlag. Mashing during the hitlag (and not after it) still counts:
    // jab 2 follows and goes on into the rapid jab with no further press.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(14, 10), -1);
    sim.tick(inp(0, 0, ATTACK));
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, 0));
    assert!(sim.f().hitlag > 0, "jab 1 hit");
    let lag = sim.f().hitlag;
    assert!(lag >= 8, "hitlag {lag}");
    // Presses on the last frames of the hitlag.
    for t in 0..lag {
        let down = t == lag - 5 || t == lag - 3;
        sim.tick(inp(0, 0, if down { ATTACK } else { 0 }));
    }
    assert_eq!((sim.f().presses, sim.f().releases), (3, 3));
    let mut seen_rapid = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().move_id == RAPID as u8 {
            seen_rapid = true;
            break;
        }
    }
    assert!(seen_rapid, "the rapid jab came out");
}

#[test]
fn the_rapid_jab_loops_while_attack_is_held_and_ends_in_the_finisher_when_let_go() {
    // Pressed thrice, then held from tick 5 to 70: the first cycle loops into the later ones (ext11, from tick 30 every 17 frames)
    // and no finisher until attack is let go; then the finisher within 8 frames.
    let mut down: Vec<u32> = vec![1, 3];
    down.extend(5..=70);
    let run = jab_run(&down, 120);
    assert_eq!(started(&run, RAPID), Some(11));
    let loops: Vec<u32> = run
        .iter()
        .filter(|(_, m, f, s)| *m == RAPID_LOOP && *f == 0 && *s == S::Attack)
        .map(|r| r.0)
        .collect();
    assert_eq!(&loops[..3], &[30, 47, 64], "a 17-frame cycle");
    let end = started(&run, RAPID_END).expect("the finisher");
    assert!((71..=79).contains(&end), "finisher on {end}");
    // Mashing (a press every 6 frames, never held) keeps it going too.
    let mut down: Vec<u32> = vec![1, 3, 5];
    down.extend((9..=90).step_by(6));
    let run = jab_run(&down, 100);
    let end = started(&run, RAPID_END).expect("the finisher");
    assert!(end > 90, "mashing kept the loop going until {end}");
}

// ---- The knee, the meteor ------------------------------------------------------------------------------------------------------

/// Damage the forward air does to a duelist held at its knee box from the start of move frame `frame - 1` on.
fn knee_damage(frame: u16) -> Fx {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FAir);
    sim.state.fighters[0].state_frame = frame - 2;
    let knee = gauntlets(&sim).get(MoveId::FAir as u8).hitboxes[0];
    let me = sim.f().pos;
    let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
    sim.put_airborne(1, me.x + knee.x, me.y + knee.y - middle, Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    sim.fighter(1).percent
}

#[test]
fn the_knee_is_22_percent_on_frame_14_only_and_3_percent_after() {
    let sim = duel();
    assert_eq!(
        knee_damage(14),
        dealt(&sim, 220),
        "the sweetspot on frame 14"
    );
    assert_eq!(
        knee_damage(15),
        dealt(&sim, 30),
        "the late hit from frame 15"
    );
    assert_eq!(knee_damage(13), Fx::ZERO, "nothing before");
    // Electric: the sweetspot's hitlag is longer than the plain formula's (x1.2 and x1.5).
    let rules = &sim.content.rules;
    let plain = sim_core::combat::hitlag_frames(Fx::from_int(22), rules, false, false, Fx::ONE);
    let knee = sim_core::combat::hitlag_frames(
        Fx::from_int(22),
        rules,
        false,
        false,
        Fx::from_ratio(12, 10) * rules.electric_hitlag_mult,
    );
    assert!(knee > plain);
}

#[test]
fn the_down_air_below_the_feet_sends_straight_down() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    sim.state.fighters[0].state_frame = 14;
    sim.state.fighters[1].percent = Fx::from_int(60);
    let me = sim.f().pos;
    let height = sim.content.fighters[0].ecb_height;
    // The target's head just under the stomper's feet: only the box below the feet reaches it.
    sim.put_airborne(1, me.x, me.y - height - fx(2, 10), Fx::ZERO, Fx::ZERO);
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::from_int(60) && !sim.fighter(1).launch_pending {
            break;
        }
    }
    assert!(sim.fighter(1).percent > Fx::from_int(60), "hit");
    let v = sim.fighter(1).kb_vel;
    assert!(v.y < Fx::ZERO && v.x.abs() <= Fx::from_raw(64), "{v:?}");
}

// ---- Per-limb intangibility (E19) and per-hitbox shieldstun (E24) ---------------------------------------------------------------

/// The sprinter in `id` on move frame `frame` (player 0) and a duelist (player 1) whose attack is one tiny hitbox on the centre of
/// the sprinter's hurtbox circle `circle`; resolves that frame's hits and returns whether the sprinter was hit.
fn poke(sim: &mut Sim, id: MoveId, frame: u16, circle: usize) -> bool {
    sim.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    sim.state.fighters[0].state_frame = frame;
    let p = sim.content.fighters[usize::from(SPRINTER)];
    let w = &sim.content.weapons[usize::from(p.weapon)];
    let (centre, _) = hurtboxes(&sim.state.fighters[0], &p, w)[circle];
    // (In the air, so the two attacks cannot clank: only grounded attacks do.)
    sim.put_airborne(1, Fx::from_int(4), Fx::ZERO, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].facing = -1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::Jab);
    sim.state.fighters[1].state_frame = 1;
    let jab = &mut sim.content.weapons[0].moves[MoveId::Jab as usize];
    jab.hitboxes.truncate(1);
    let hb = &mut jab.hitboxes[0];
    hb.start = 0;
    hb.end = 5;
    hb.radius = fx(2, 100);
    hb.x = Fx::from_int(4) - centre.x;
    hb.y = centre.y;
    let before = sim.f().percent;
    resolve_hits(&mut sim.state, &sim.content);
    sim.f().percent > before
}

#[test]
fn the_up_tilts_and_up_smashs_kicking_leg_cannot_be_hit_while_the_kick_is_out() {
    // The leg (the limb circle, reaching toward the kick) on the up tilt's frames 14-17 and the up smash's 22-23 and 28-29: a hit
    // there misses; the body still takes a hit.
    for (id, frames) in [
        (MoveId::UTilt, &[13u16, 16][..]),
        (MoveId::USmash, &[21, 22, 27, 28][..]),
    ] {
        for &f in frames {
            let mut sim = duel();
            assert!(!poke(&mut sim, id, f, 3), "{id:?} frame {f}: the leg");
            assert_eq!(
                intangible_parts(sim.f(), gauntlets(&sim)),
                PART_LIMB,
                "{id:?} frame {f}"
            );
            let mut sim = duel();
            assert!(poke(&mut sim, id, f, 1), "{id:?} frame {f}: the body");
        }
    }
    // Without the window (any other move, or the default), the leg is hit.
    let mut sim = duel();
    sim.content.weapons[usize::from(SPRINTER)].moves[MoveId::UTilt as usize]
        .part_intangible
        .clear();
    assert!(poke(&mut sim, MoveId::UTilt, 14, 3), "the leg, no window");
    let mut sim = duel();
    assert_eq!(intangible_parts(sim.f(), gauntlets(&sim)), 0, "standing");
    let _ = &mut sim;
    for w in &duel().content.weapons[..4] {
        assert!(w.moves.iter().all(|m| m.part_intangible.is_empty()));
    }
}

/// The shield stun a duelist shielding in front takes from the sprinter's `id` on move frame `frame`.
fn shield_stun(id: MoveId, frame: u16, shieldstun: Option<u16>) -> u8 {
    let mut sim = duel();
    if let Some(s) = shieldstun {
        for h in &mut sim.content.weapons[usize::from(SPRINTER)].moves[id as usize].hitboxes {
            h.shieldstun = s;
        }
    }
    sim.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    sim.state.fighters[0].state_frame = frame;
    sim.stand(1, fx(16, 10), -1);
    sim.state.fighters[1].state = S::Shield;
    sim.state.fighters[1].state_frame = 30;
    resolve_hits(&mut sim.state, &sim.content);
    assert!(sim.fighter(1).hitlag > 0, "{id:?} blocked");
    sim.fighter(1).shield_stun
}

#[test]
fn the_dash_attack_and_rapid_jab_stun_a_shield_longer() {
    // floor(damage x 0.8 x multiplier + 2), exact (sim 67): the dash attack 10% x1.5 = 14, a rapid jab hit 0.6% x9 = 6.
    let stun = |tenths: i32, percent: i32| (tenths * 8 * percent / 10_000 + 2) as u8;
    assert_eq!((stun(100, 150), stun(100, 100)), (14, 10));
    assert_eq!(shield_stun(MoveId::DashAttack, 6, None), stun(100, 150));
    assert_eq!(
        shield_stun(MoveId::DashAttack, 6, Some(100)),
        stun(100, 100)
    );
    assert!(stun(100, 150) >= stun(100, 100) + 3);
    assert_eq!(shield_stun(RAPID, 4, None), stun(6, 900));
    assert_eq!(shield_stun(RAPID, 4, Some(100)), stun(6, 100));
    assert_eq!((stun(6, 900), stun(6, 100)), (6, 2));
    // Every other weapon keeps the normal stun.
    for w in &duel().content.weapons[..4] {
        assert!(w
            .moves
            .iter()
            .all(|m| m.hitboxes.iter().all(|h| h.shieldstun == 100)));
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
fn the_standing_grab_catches_on_frame_6() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    assert_eq!(grab_now(&mut sim), 6);
    assert!(gauntlets(&sim)
        .get(MoveId::Grab as u8)
        .hitboxes
        .iter()
        .all(|h| h.kind == HIT_GRAB));
}

#[test]
fn the_throws_hit_and_release_on_their_frames_for_their_damage() {
    // (stick, the frames the held fighter's percent rises on, total damage in tenths)
    for (stick, frames, damage) in [
        ((127i8, 0i8), vec![11u8, 13], 75),
        ((-127, 0), vec![12, 13], 75),
        ((0, 127), vec![12, 14], 90),
        ((0, -127), vec![21], 60),
    ] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(12, 10), -1);
        grab_now(&mut sim);
        let before = sim.fighter(1).percent;
        sim.tick(inp(stick.0, stick.1, 0));
        let mut rises = Vec::new();
        let mut last = before;
        // (Hitlag freezes the throw: count the throw's own frames.)
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
            // The back throw: away behind and up, and the thrower ends facing the way it threw.
            assert!(sim.fighter(1).pos.x < Fx::ZERO, "thrown behind");
            assert_eq!(sim.f().facing, -1, "facing the throw");
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
    let plain = duel().content.fighters[usize::from(SPRINTER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window or None)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 10u16, 7u8, 32u16, Some(1u16)),
        (MoveId::FAir, 20, 18, 42, Some(2)),
        (MoveId::BAir, 12, 10, 19, Some(2)),
        (MoveId::UAir, 8, 10, 24, None),
        (MoveId::DAir, 20, 12, 39, Some(1)),
    ] {
        assert_eq!(land_during(id, mid), (S::Landing, lag), "{id:?} mid-move");
        assert_eq!(land_during(id, late), (S::Landing, plain), "{id:?} late");
        match early {
            Some(e) => assert_eq!(land_during(id, e).1, plain, "{id:?} early"),
            None => assert_eq!(land_during(id, 1).1, lag, "{id:?} has no early window"),
        }
    }
}

// ---- Body ----------------------------------------------------------------------------------------------------------------------

#[test]
fn the_sprinters_jumps_have_the_references_air_times() {
    // Short hop 34, full hop 48, short hop fast fall 24, full hop fast fall 34 (ultimateframedata.com); within one frame.
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
        (false, false, 34u32),
        (true, false, 48),
        (false, true, 24),
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
fn the_sprinter_is_the_references_fast_heavyweight() {
    let p = duel().content.fighters[usize::from(SPRINTER)];
    let su = sim_core::FighterParams::su;
    assert_eq!(p.weight, Fx::from_int(104));
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(1218), su(120), su(1865), su(2984))
    );
    assert_eq!(
        (p.air_accel, p.air_accel_stick, p.air_friction),
        (su(10), su(65), sim_core::FighterParams::su_fine(750))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (42, 62, 72, 94)
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
        (30, 35, 26)
    );
    assert_eq!((p.dash_frames, p.heavy_landing_lag), (15, 3));
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (1, su(1400), su(3100))
    );
    assert_eq!(p.weapon, SPRINTER);
    // Taller than the shared body.
    assert!(p.ecb_height > sim_core::FighterParams::tumbler().ecb_height);
}

// ---- Determinism ---------------------------------------------------------------------------------------------------------------

#[test]
fn the_new_fields_are_in_the_content_hash() {
    let base = Content::placeholder();
    let h = base.hash();
    let w = usize::from(SPRINTER);
    let mut c = base.clone();
    c.weapons[w].moves[MoveId::DashAttack as usize].hitboxes[0].shieldstun = 100;
    assert_ne!(h, c.hash(), "shieldstun");
    let mut c = base.clone();
    c.weapons[w].moves[MoveId::UTilt as usize].part_intangible[0].end = 15;
    assert_ne!(h, c.hash(), "part intangible");
    let mut c = base.clone();
    c.weapons[w].moves[MoveId::Jab as usize].armor = Some(sim_core::moves::Armor {
        start: 0,
        end: 3,
        kind: sim_core::moves::ARMOR_SUPER,
        threshold: Fx::ZERO,
    });
    assert_ne!(h, c.hash(), "armor");
    for (name, edit) in [
        (
            "fast_fall_from",
            (|m| m.fast_fall_from = 9) as fn(&mut sim_core::moves::Move),
        ),
        ("lands_from", |m| m.lands_from = 9),
        ("on_search", |m| m.on_search = Some(30)),
        ("air_version", |m| m.air_version = Some(30)),
        ("catch_move", |m| m.catch_move = Some(30)),
        ("hold_at", |m| m.hold_at = Some((Fx::ONE, Fx::ONE))),
        ("command", |m| m.hitboxes[0].command = true),
        ("clanks", |m| m.hitboxes[0].clanks = false),
        ("turn_at", |m| m.turn_at = Some((13, 25))),
        ("air_blocks", |m| {
            m.air_blocks = vec![sim_core::moves::AirBlock {
                from: 0,
                to: 3,
                gravity: None,
                max_fall: None,
                friction: None,
                drift: 100,
                accel: 600,
            }]
        }),
    ] {
        let mut c = base.clone();
        edit(&mut c.weapons[w].moves[MoveId::NAir as usize]);
        assert_ne!(h, c.hash(), "{name}");
    }
    let mut c = base;
    c.weapons[w].moves[MoveId::Ext15 as usize] = c.weapons[w].moves[MoveId::Jab as usize].clone();
    assert_ne!(h, c.hash(), "the new slots");
}

#[test]
fn random_play_with_the_sprinter_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [SPRINTER, 0, SPRINTER, 3];
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

#[test]
fn the_shown_boxes_come_out_only_on_the_moves_frames() {
    // (A check of the view's helper: no Gauntlets hitbox is out on frame 0 of any move.)
    let sim = duel();
    let w = gauntlets(&sim);
    for id in [MoveId::Jab, MoveId::FAir, MoveId::USmash] {
        let mut f = *sim.f();
        f.state = S::Attack;
        f.move_id = id as u8;
        f.state_frame = 0;
        assert_eq!(active_hitboxes(&f, w.get(id as u8), Fx::ONE).count(), 0);
    }
}
