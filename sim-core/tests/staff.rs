//! The Staff moveset on the seer body (class 5), modelled mechanically on a reference character (phase 1: the body and every normal;
//! the specials are still the claws'). Frame data against the reference for every normal, checked in the data and by running the
//! moves; hit properties; the jab's entry rule into the rapid jab (pressed and let go twice, or held on hit), its loop and finisher;
//! the multi-hit loops (up tilt, neutral air, up air) with their last hits; the neutral air's negative hitstun modifier (E27) and the
//! angle types of its last hit and of the up air (E26); the smashes' wind (pushes re-applied every frame); the dash attack's and back
//! air's partial intangibility; grabs, pummel and throws; landing lag and autocancels; the body's attributes and jump air times; and
//! determinism. Reference data and sources: `docs/movesets/staff/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::{hurtboxes, intangible_parts, resolve_hits};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::{
    MoveId, Weapon, ANGLE_AUTOLINK, ANGLE_BACKWARD, ANGLE_BY_POSITION, ANGLE_FORWARD, HIT_GRAB,
    PART_HIGH, PART_LIMB, PART_MIDDLE, TARGETS_AERIAL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const SEER: u8 = 5;

/// The rapid jab's first cycle, later cycles and finisher.
const RAPID: MoveId = MoveId::Ext10;
const RAPID_LOOP: MoveId = MoveId::Ext11;
const RAPID_END: MoveId = MoveId::Ext12;

/// The seer (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SEER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn staff(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SEER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R] except the pummel's frame and FAF, the down throw's release and FAF with its
/// hitlag folded in, and the rapid jab's later cycles, [D]).
const FRAME_DATA: [FrameData; 27] = [
    (MoveId::Jab, &[(8, 9)], 25, 0, 0, 255),
    (
        RAPID,
        &[
            (4, 4),
            (7, 7),
            (10, 10),
            (13, 13),
            (16, 16),
            (19, 19),
            (22, 22),
        ],
        26,
        0,
        0,
        255,
    ),
    // (The later cycles start on the cycle's frame 2: the same hits, one frame earlier in the move.)
    (
        RAPID_LOOP,
        &[
            (3, 3),
            (6, 6),
            (9, 9),
            (12, 12),
            (15, 15),
            (18, 18),
            (21, 21),
        ],
        25,
        0,
        0,
        255,
    ),
    (RAPID_END, &[(3, 4)], 40, 0, 0, 255),
    (
        MoveId::FTilt,
        &[(14, 16), (17, 20), (21, 23), (24, 35)],
        60,
        0,
        0,
        255,
    ),
    (MoveId::UTilt, &[(8, 22), (24, 25)], 56, 0, 0, 255),
    (MoveId::DTilt, &[(14, 27), (15, 27)], 40, 0, 0, 255),
    (MoveId::DashAttack, &[(6, 9), (10, 17)], 41, 0, 0, 255),
    (
        MoveId::FSmash,
        &[(18, 20), (21, 25), (26, 31)],
        64,
        0,
        0,
        255,
    ),
    (MoveId::USmash, &[(18, 29), (30, 35)], 64, 0, 0, 255),
    (
        MoveId::DSmash,
        &[(17, 17), (18, 19), (20, 29)],
        69,
        0,
        0,
        255,
    ),
    (
        MoveId::NAir,
        &[(5, 6), (7, 23), (24, 27), (29, 30)],
        52,
        12,
        3,
        40,
    ),
    (MoveId::FAir, &[(9, 11)], 45, 9, 3, 24),
    (MoveId::BAir, &[(8, 10)], 50, 10, 2, 35),
    (MoveId::UAir, &[(10, 22), (24, 25)], 52, 14, 2, 42),
    (MoveId::DAir, &[(10, 11)], 59, 11, 2, 48),
    (MoveId::Grab, &[(7, 8)], 40, 0, 0, 255),
    (MoveId::DashGrab, &[(10, 11)], 48, 0, 0, 255),
    (MoveId::PivotGrab, &[(11, 12)], 43, 0, 0, 255),
    // (The pummel: the reference's hit on frame 1 and FAF 7, one frame later here [D].)
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(20, 20)], 45, 0, 0, 255),
    (MoveId::BThrow, &[(20, 20)], 55, 0, 0, 255),
    (MoveId::UThrow, &[(17, 17)], 55, 0, 0, 255),
    // (The down throw: the reference's release 25 and FAF 46 plus its 8 frames of hitlag on frame 24 [D].)
    (MoveId::DThrow, &[(33, 33)], 54, 0, 0, 255),
    (MoveId::LedgeAttack, &[(23, 25)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(16, 17), (25, 26)], 46, 0, 0, 255),
    // (No jab 2 or 3.)
    (MoveId::Jab2, &[], 0, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = staff(&sim);
    for (id, windows, faf, landing, ac_before, ac_from) in FRAME_DATA {
        let mv = w.get(id as u8);
        if faf == 0 {
            assert!(mv.is_empty(), "{id:?} is empty");
            continue;
        }
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
    assert!(w.get(MoveId::Jab3 as u8).is_empty());
    assert_eq!(
        sim.content.fighters[usize::from(SEER)].ledge_attack_frames,
        55
    );
    // Intangibility: the ledge attack through frame 25, the get-up attack through 26.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 25);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 26);
    // Smash charge frames: forward on 10, up on 12, down on 13 (move frames one less).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(9));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(11));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(12));
    // The multi-hit rhythms: the up tilt every 3 from 8, the neutral air every 4 from 5, the up air every 3 from 10, the rapid jab
    // every 3 from its 4; only the loops re-arm (the last hits land once); the smashes' wind every frame.
    let rhythm = |id: MoveId| (w.get(id as u8).rehit, w.get(id as u8).rehit_groups);
    assert_eq!(rhythm(MoveId::UTilt), (Some((7, 3)), 0b1));
    assert_eq!(rhythm(MoveId::NAir), (Some((4, 4)), 0b1));
    assert_eq!(rhythm(MoveId::UAir), (Some((9, 3)), 0b1));
    assert_eq!(rhythm(RAPID).0, Some((3, 3)));
    assert_eq!(rhythm(MoveId::FSmash), (Some((20, 1)), 0b10));
    assert_eq!(rhythm(MoveId::DSmash), (Some((19, 1)), 0b10));
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (in the air high up for an aerial, else standing), with player 1 held where the move's first hitbox of
/// window `first` (one that can hit an airborne fighter) will be, and runs until player 1 is hit. Returns the reference frame of the
/// hit.
fn first_hit_frame(id: MoveId, first: u8) -> Option<u8> {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let hb = *staff(&sim)
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
        (MoveId::Jab, 8u8),
        (RAPID, 4),
        (RAPID_END, 3),
        (MoveId::FTilt, 14),
        (MoveId::UTilt, 8),
        (MoveId::DTilt, 14),
        (MoveId::DashAttack, 6),
        (MoveId::FSmash, 18),
        (MoveId::USmash, 18),
        (MoveId::DSmash, 17),
        (MoveId::NAir, 5),
        (MoveId::FAir, 9),
        (MoveId::BAir, 8),
        (MoveId::UAir, 10),
        (MoveId::DAir, 10),
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
        // (Throws need someone held, the ledge attack a ledge; the rapid jab's cycles hand on by themselves: checked below.)
        if faf == 0
            || matches!(
                id,
                MoveId::Pummel
                    | MoveId::FThrow
                    | MoveId::BThrow
                    | MoveId::UThrow
                    | MoveId::DThrow
                    | MoveId::LedgeAttack
                    | RAPID
                    | RAPID_LOOP
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
    // (move, reference first frame, priority (the reference's id), damage in tenths, angle, BK, KS) for the main boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 40] = [
        (MoveId::Jab, 8, 0, 30, 361, 25, 20),
        (MoveId::Jab, 8, 2, 30, 180, 20, 20),
        (RAPID, 4, 0, 6, 361, 10, 20),
        (RAPID_END, 3, 0, 35, 361, 55, 140),
        (MoveId::FTilt, 14, 1, 60, 90, 0, 100),
        (MoveId::FTilt, 14, 0, 60, 45, 0, 100),
        (MoveId::FTilt, 17, 1, 60, 285, 0, 100),
        (MoveId::FTilt, 17, 0, 60, 100, 0, 100),
        (MoveId::FTilt, 21, 1, 60, 20, 0, 100),
        (MoveId::FTilt, 21, 0, 60, 85, 0, 100),
        (MoveId::FTilt, 24, 1, 70, 40, 78, 80),
        (MoveId::UTilt, 8, 0, 14, ANGLE_AUTOLINK, 0, 100),
        (MoveId::UTilt, 8, 1, 14, 130, 0, 100),
        (MoveId::UTilt, 8, 2, 14, 190, 0, 100),
        (MoveId::UTilt, 24, 0, 45, 85, 75, 136),
        (MoveId::DTilt, 14, 0, 85, 35, 40, 100),
        (MoveId::DTilt, 14, 1, 50, 75, 38, 100),
        (MoveId::DashAttack, 6, 0, 110, 70, 71, 76),
        (MoveId::DashAttack, 6, 1, 110, 60, 71, 76),
        (MoveId::DashAttack, 10, 0, 70, 60, 61, 56),
        (MoveId::FSmash, 18, 0, 160, 361, 40, 100),
        (MoveId::FSmash, 18, 1, 130, 361, 40, 102),
        (MoveId::USmash, 18, 0, 160, 88, 53, 84),
        (MoveId::USmash, 18, 1, 120, 88, 58, 89),
        (MoveId::USmash, 18, 2, 90, 88, 62, 92),
        (MoveId::USmash, 30, 0, 130, 88, 53, 84),
        (MoveId::USmash, 30, 2, 70, 88, 62, 92),
        (MoveId::DSmash, 17, 0, 150, 60, 52, 103),
        (MoveId::DSmash, 18, 0, 130, 28, 30, 90),
        (MoveId::NAir, 5, 0, 14, ANGLE_AUTOLINK, 0, 100),
        (MoveId::NAir, 5, 3, 14, 95, 0, 100),
        (MoveId::NAir, 7, 2, 14, 100, 0, 100),
        (MoveId::NAir, 29, 0, 51, 55, 35, 155),
        (MoveId::FAir, 9, 0, 100, 45, 31, 92),
        (MoveId::BAir, 8, 0, 120, 361, 30, 95),
        (MoveId::UAir, 10, 0, 10, 105, 0, 100),
        (MoveId::UAir, 10, 1, 10, ANGLE_AUTOLINK, 0, 100),
        (MoveId::UAir, 24, 0, 50, 88, 50, 152),
        (MoveId::DAir, 10, 0, 110, 270, 20, 100),
        (MoveId::DAir, 10, 1, 110, 55, 30, 100),
    ];
    let sim = duel();
    let w = staff(&sim);
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
    // Hitlag, hitstun, SDI, shieldstun, set knockback and set weight, targets, shield damage, angle types.
    assert!(all(MoveId::Jab, &|h| (
        h.hitlag,
        h.hitstun_add,
        h.angle_type
    ) == (160, 4, ANGLE_FORWARD)));
    for id in [RAPID, RAPID_LOOP] {
        assert!(all(id, &|h| (h.hitlag, h.sdi, h.shieldstun) == (50, 40, 700)));
    }
    assert!(all(RAPID_END, &|h| h.hitlag == 200));
    // The forward tilt's first hit: set knockback with set weight; the head's FKV 25 / 25 / 35, the tail's 45 / 40 / 30.
    let ftilt = get(MoveId::FTilt);
    let fkv: Vec<(u8, u16)> = ftilt
        .hitboxes
        .iter()
        .filter(|h| h.group == 0)
        .map(|h| (h.priority, h.fixed_kb))
        .collect();
    assert_eq!(
        fkv,
        vec![(0, 45), (1, 25), (0, 40), (1, 25), (0, 30), (1, 35)]
    );
    assert!(ftilt
        .hitboxes
        .iter()
        .all(|h| (h.group == 0) == (h.set_weight && h.hitlag == 80 && h.sdi == 0)));
    assert!(ftilt.hitboxes.iter().all(|h| h.angle_type == ANGLE_FORWARD));
    // The up tilt's loop: set knockback 20 / 50 / 30, hitlag x0.5, SDI x0.3; the last hit hitlag x1.3; angle type 0.
    let utilt = get(MoveId::UTilt);
    assert!(utilt
        .hitboxes
        .iter()
        .all(|h| h.angle_type == ANGLE_BY_POSITION));
    for h in utilt.hitboxes.iter().filter(|h| h.group == 0) {
        assert_eq!(
            (h.fixed_kb, h.hitlag, h.sdi, h.shield_damage),
            ([20, 50, 30][usize::from(h.priority)], 50, 30, 171)
        );
    }
    assert!(utilt
        .hitboxes
        .iter()
        .filter(|h| h.group == 1)
        .all(|h| h.hitlag == 130 && h.fixed_kb == 0));
    // The down tilt: 8.5% on grounded targets, 5% on aerial ones.
    assert!(all(MoveId::DTilt, &|h| (h.targets == TARGETS_GROUNDED)
        == (h.damage == fx(85, 10))
        && h.targets != 0));
    // The dash attack's shield damage +3 (127 / 143).
    assert!(all(MoveId::DashAttack, &|h| h.shield_damage
        == if h.damage == Fx::from_int(11) {
            127
        } else {
            143
        }));
    // The forward and down smash's wind: pushes, set knockback 60 / 35 and 55.
    let pushes = |id: MoveId| -> Vec<u16> {
        let mut v: Vec<u16> = get(id)
            .hitboxes
            .iter()
            .filter(|h| h.push)
            .map(|h| h.fixed_kb)
            .collect();
        v.dedup();
        v
    };
    assert_eq!(pushes(MoveId::FSmash), vec![60, 35]);
    assert_eq!(pushes(MoveId::DSmash), vec![55]);
    assert!(all(MoveId::FSmash, &|h| h.push == (h.group == 1)));
    assert!(all(MoveId::FSmash, &|h| h.angle_type == ANGLE_FORWARD));
    assert!(all(MoveId::DSmash, &|h| h.angle_type == ANGLE_BY_POSITION));
    // The neutral air: the late loop takes 13 frames of hitstun off and its aerial boxes become set knockback 65 with no SDI; the
    // grounded boxes pop up by set knockback; the last hit is angle type 3, the loop type 0.
    let nair = get(MoveId::NAir);
    for h in &nair.hitboxes {
        let late = h.start + 1 == 24;
        assert_eq!(h.hitstun_add, if late { -13 } else { 0 }, "{h:?}");
        if h.group == 0 {
            assert_eq!(h.angle_type, ANGLE_BY_POSITION);
            let aerial = h.targets == TARGETS_AERIAL;
            assert!(aerial || h.targets == TARGETS_GROUNDED);
            let fkv = match (aerial, late, h.start + 1 == 5 && h.priority == 3) {
                (true, false, _) => 0,
                (_, _, true) => 80,
                _ => 65,
            };
            assert_eq!(h.fixed_kb, fkv, "{h:?}");
            assert_eq!(h.sdi, if aerial && late { 0 } else { 30 });
        } else {
            assert_eq!((h.angle_type, h.hitlag), (ANGLE_FORWARD, 150));
        }
    }
    // The up air: every box angle type 4 (backward), the grounded loop box set knockback 40, the aerial ones 20.
    let uair = get(MoveId::UAir);
    assert!(uair.hitboxes.iter().all(|h| h.angle_type == ANGLE_BACKWARD));
    for h in uair.hitboxes.iter().filter(|h| h.group == 0) {
        let grounded = h.targets == TARGETS_GROUNDED;
        assert_eq!(h.fixed_kb, if grounded { 40 } else { 20 });
        assert!(grounded == (h.priority == 0) && (grounded || h.targets == TARGETS_AERIAL));
    }
    // The back air sends backward (type 4), the down air's grounded box backward and its meteor on aerial targets only.
    assert!(all(MoveId::BAir, &|h| h.angle_type == ANGLE_BACKWARD
        && h.hitlag == 120));
    let dair = get(MoveId::DAir);
    assert!(dair.hitboxes.iter().all(|h| h.hitlag == 150
        && (h.angle == 270) == (h.targets == TARGETS_AERIAL)
        && (h.targets == TARGETS_GROUNDED) == (h.angle_type == ANGLE_BACKWARD)));
    let pummel = get(MoveId::Pummel).hitboxes[0];
    assert_eq!(
        (pummel.damage, pummel.fixed_kb, pummel.hitlag),
        (fx(13, 10), 30, 210)
    );
    assert!(all(MoveId::GetUpAttack, &|h| h.shield_damage == 214
        && h.angle_type == ANGLE_BY_POSITION));
    assert!(all(MoveId::LedgeAttack, &|h| h.shield_damage == 111
        && h.angle_type == ANGLE_FORWARD));
}

// ---- Jab -----------------------------------------------------------------------------------------------------------------------

/// Runs player 0's attack button down on the ticks in `down` (1-based) with player 1 in front (untouchable unless `hittable`);
/// returns (tick, move, move frame, state) after each tick for `ticks` ticks.
fn jab_run(down: &[u32], ticks: u32, hittable: bool) -> Vec<(u32, MoveId, u16, S)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
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

#[test]
fn the_quickest_rapid_jab_runs_on_the_references_frames() {
    // Pressed on 1 and 3 (two presses, two releases), then let go: jab 1 from tick 1, the rapid jab from 10 (jab 1's frame 10, the
    // start of its window), its hits on 13, 16 ... 31, the finisher from 33 (the cycle's frame 24) with its hit on 35-36 and control
    // back on 72. (Tick t of the run is frame t from jab 1's first frame: the reference's quickest loop.)
    let run = jab_run(&[1, 3], 90, false);
    assert_eq!(started(&run, MoveId::Jab), Some(1));
    assert_eq!(started(&run, RAPID), Some(10));
    assert_eq!(started(&run, RAPID_LOOP), None, "one cycle only");
    assert_eq!(started(&run, RAPID_END), Some(33));
    let sim = duel();
    let w = staff(&sim);
    let out_on = |id: MoveId| -> Vec<u32> {
        let mut ticks: Vec<u32> = run
            .iter()
            .filter(|(_, m, f, s)| {
                *m == id
                    && *s == S::Attack
                    && w.get(id as u8)
                        .hitboxes
                        .iter()
                        .any(|h| u16::from(h.start) <= *f && *f <= u16::from(h.end))
            })
            .map(|r| r.0)
            .collect();
        ticks.dedup();
        ticks
    };
    assert_eq!(out_on(MoveId::Jab), vec![8, 9]);
    assert_eq!(out_on(RAPID), vec![13, 16, 19, 22, 25, 28, 31]);
    assert_eq!(out_on(RAPID_END), vec![35, 36]);
    let back = run.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1);
    assert_eq!(back, Some(72), "the finisher's FAF, counted from jab 1");
}

#[test]
fn one_press_or_a_held_button_without_a_hit_gives_jab_1_alone() {
    // One press: jab 1 alone, FAF 25.
    let run = jab_run(&[1], 40, false);
    assert_eq!(started(&run, RAPID), None);
    assert_eq!(
        run.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1),
        Some(25)
    );
    // Held from the start but nothing hit: no rapid jab (holding goes into it only on a hit).
    let run = jab_run(&(1..=24).collect::<Vec<_>>(), 24, false);
    assert_eq!(started(&run, RAPID), None);
    // Pressed twice but the second release comes after the window (frame 24): no rapid jab.
    let mut down = vec![1];
    down.extend(3..=26);
    let run = jab_run(&down, 30, false);
    assert_eq!(started(&run, RAPID), None);
}

#[test]
fn holding_attack_when_jab_1_hits_goes_into_the_rapid_jab_on_frame_13() {
    // Held from tick 1 with a real target in front: jab 1 hits on its frame 8, both freeze for its hitlag, and the rapid jab starts
    // where jab 1's frame 13 would have been.
    let run = jab_run(&(1..=60).collect::<Vec<_>>(), 60, true);
    let at = started(&run, RAPID).expect("the rapid jab");
    let before = run[(at - 2) as usize];
    assert_eq!(
        (before.1, before.2),
        (MoveId::Jab, 11),
        "jab 1 ran to its frame 12"
    );
}

#[test]
fn the_rapid_jab_loops_while_attack_is_held_and_ends_in_the_finisher_when_let_go() {
    // Pressed twice, then held from tick 5 to 80: the first cycle loops into the later ones (ext11, from tick 33 every 22 frames) and
    // no finisher until attack is let go; then the finisher within 8 frames.
    let mut down: Vec<u32> = vec![1, 3];
    down.extend(5..=80);
    let run = jab_run(&down, 130, false);
    assert_eq!(started(&run, RAPID), Some(10));
    let loops: Vec<u32> = run
        .iter()
        .filter(|(_, m, f, s)| *m == RAPID_LOOP && *f == 0 && *s == S::Attack)
        .map(|r| r.0)
        .collect();
    assert_eq!(&loops[..3], &[33, 55, 77], "a 22-frame cycle");
    let end = started(&run, RAPID_END).expect("the finisher");
    assert!((81..=89).contains(&end), "finisher on {end}");
    // Mashing (a press every 6 frames, never held) keeps it going too.
    let mut down: Vec<u32> = vec![1, 3];
    down.extend((7..=90).step_by(6));
    let run = jab_run(&down, 100, false);
    let end = started(&run, RAPID_END).expect("the finisher");
    assert!(end > 90, "mashing kept the loop going until {end}");
}

#[test]
fn the_rapid_jab_stuns_a_shield_seven_times_longer() {
    // floor(damage x 0.8 x 7 + 2) for a 0.6% hit, against floor(0.6 x 0.8 + 2) without the multiplier.
    let stun = |multiplier: Option<u16>| {
        let mut sim = duel();
        if let Some(s) = multiplier {
            for h in &mut sim.content.weapons[usize::from(SEER)].moves[RAPID as usize].hitboxes {
                h.shieldstun = s;
            }
        }
        sim.stand(0, Fx::ZERO, 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], RAPID);
        sim.state.fighters[0].state_frame = 3;
        sim.stand(1, fx(16, 10), -1);
        sim.state.fighters[1].state = S::Shield;
        sim.state.fighters[1].state_frame = 30;
        resolve_hits(&mut sim.state, &sim.content);
        assert!(sim.fighter(1).hitlag > 0, "blocked");
        sim.fighter(1).shield_stun
    };
    assert_eq!(stun(None), 5);
    assert_eq!(stun(Some(100)), 2);
}

// ---- Multi-hits ------------------------------------------------------------------------------------------------------------------

/// Player 0 does `id` from move frame 0 (in the air for an aerial) with a duelist held in the air at `(dx, dy)` from it (world tenths)
/// every frame until the move ends; returns the percent dealt and the number of hits.
fn hits_on_a_held_target(id: MoveId, dx: i32, dy: i32) -> (Fx, u32) {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let mut hits = 0;
    let mut last = Fx::ZERO;
    for _ in 0..90 {
        let me = sim.state.fighters[0].pos;
        let p = sim.state.fighters[1];
        sim.put_airborne(1, me.x + fx(dx, 10), me.y + fx(dy, 10), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].percent = p.percent;
        // (Gravity and the launch are cancelled: the target stays where it is held.)
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            hits += 1;
            last = sim.fighter(1).percent;
        }
        if sim.f().state != S::Attack {
            break;
        }
    }
    (last, hits)
}

#[test]
fn the_loops_hit_every_rhythm_beat_and_the_last_hit_once() {
    let sim = duel();
    let total =
        |loops: i32, each: i32, last: i32| dealt(&sim, each).mul_int(loops) + dealt(&sim, last);
    // The up tilt: five 1.4% hits and the 4.5% last hit: 11.5%.
    assert_eq!(
        hits_on_a_held_target(MoveId::UTilt, 4, 14),
        (total(5, 14, 45), 6)
    );
    // The neutral air: six 1.4% hits and the 5.1% last hit: 13.5%.
    assert_eq!(
        hits_on_a_held_target(MoveId::NAir, 7, -2),
        (total(6, 14, 51), 7)
    );
    // The up air: five 1% hits and the 5% last hit: 10%.
    assert_eq!(
        hits_on_a_held_target(MoveId::UAir, 0, 16),
        (total(5, 10, 50), 6)
    );
}

#[test]
fn the_neutral_airs_late_loop_takes_13_frames_of_hitstun_off() {
    // The same airborne target hit by a mid-loop box and a late-loop box (both set knockback 65 on aerial targets once the late boxes'
    // FKV is set the same way): the late one's hitstun is 13 frames shorter. Measured on the late box with and without its modifier.
    let stun = |modifier: Option<i16>| {
        let mut sim = duel();
        if let Some(m) = modifier {
            for h in
                &mut sim.content.weapons[usize::from(SEER)].moves[MoveId::NAir as usize].hitboxes
            {
                if h.start + 1 == 24 {
                    h.hitstun_add = m;
                }
            }
        }
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NAir);
        sim.state.fighters[0].state_frame = 23;
        let me = sim.f().pos;
        sim.put_airborne(1, me.x + fx(7, 10), me.y - fx(2, 10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.fighter(1).state, S::Hitstun, "hit");
        sim.fighter(1).hitstun
    };
    assert_eq!(stun(None) + 13, stun(Some(0)));
}

/// The sign of the horizontal launch of player 1 hit by player 0's `id` (on move frame `frame`, player 0 facing right in the air)
/// with player 1 held at `(dx, dy)` world tenths from it.
fn launched_toward(id: MoveId, frame: u16, dx: i32, dy: i32) -> i32 {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
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
    sim.fighter(1).kb_vel.x.raw().signum()
}

#[test]
fn the_neutral_airs_last_hit_sends_forward_and_the_up_air_backward_on_both_sides() {
    // The neutral air's last hit (frame 29) on a target behind: sent the way the fighter faces (angle type 3).
    assert_eq!(launched_toward(MoveId::NAir, 28, -6, -2), 1);
    assert_eq!(launched_toward(MoveId::NAir, 28, 6, -2), 1);
    // The up air's last hit (frame 24) on a target in front: sent backward (type 4).
    assert_eq!(launched_toward(MoveId::UAir, 23, 8, 16), -1);
    assert_eq!(launched_toward(MoveId::UAir, 23, -8, 16), -1);
    // The back air behind: backward.
    assert_eq!(launched_toward(MoveId::BAir, 7, -18, 2), -1);
}

#[test]
fn the_forward_smashs_wind_pushes_every_frame_without_hitstun() {
    // A duelist in the air ahead of the wing tips, in the wind only: pushed away on each frame of the wind, never hit.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
    sim.state.fighters[0].state_frame = 19;
    let mut pushes = 0;
    for _ in 0..11 {
        sim.put_airborne(1, fx(32, 10), Fx::ZERO + fx(2, 10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        assert_ne!(sim.fighter(1).state, S::Hitstun);
        assert_eq!(sim.fighter(1).percent, Fx::ZERO);
        if sim.fighter(1).vel.x > Fx::ZERO {
            pushes += 1;
        }
    }
    assert_eq!(pushes, 11, "pushed on each of frames 21-31");
}

// ---- Partial intangibility (E19) -------------------------------------------------------------------------------------------------

/// The seer in `id` on move frame `frame` (player 0, in the air) and a duelist (player 1) whose attack is one tiny hitbox on the
/// centre of the seer's hurtbox circle `circle`; resolves that frame's hits and returns whether the seer was hit.
fn poke(id: MoveId, frame: u16, circle: usize) -> bool {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    sim.state.fighters[0].state_frame = frame;
    let p = sim.content.fighters[usize::from(SEER)];
    let w = &sim.content.weapons[usize::from(p.weapon)];
    let (centre, _) = hurtboxes(&sim.state.fighters[0], &p, w)[circle];
    sim.put_airborne(1, Fx::from_int(4), Fx::from_int(10), Fx::ZERO, Fx::ZERO);
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
    hb.y = centre.y - Fx::from_int(10);
    let before = sim.f().percent;
    resolve_hits(&mut sim.state, &sim.content);
    sim.f().percent > before
}

#[test]
fn the_dash_attack_and_back_air_leave_only_the_legs_hittable_for_a_moment() {
    // The dash attack's head and torso on frames 5-6, the back air's on frame 7: the middle and top circles (and the fourth, which a
    // held weapon keeps on the middle) miss; the low one is hit. The frame after, all are hit again.
    for (id, frames, after) in [
        (MoveId::DashAttack, &[4u16, 5][..], 6u16),
        (MoveId::BAir, &[6][..], 7),
    ] {
        for &f in frames {
            // (The head circle's centre is inside the torso's too, but not the legs'; the torso's centre is inside the legs'.)
            assert!(!poke(id, f, 2), "{id:?} frame {f}: the head and torso");
            assert!(poke(id, f, 0), "{id:?} frame {f}: the legs");
            let sim = duel();
            let mut fighter = *sim.f();
            fighter.state = S::Attack;
            fighter.move_id = id as u8;
            fighter.state_frame = f;
            assert_eq!(
                intangible_parts(&fighter, staff(&sim)),
                PART_HIGH | PART_MIDDLE | PART_LIMB
            );
        }
        assert!(poke(id, after, 2), "{id:?} frame {after}: the head again");
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
fn the_standing_grab_catches_on_frame_7() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    assert_eq!(grab_now(&mut sim), 7);
    assert!(staff(&sim)
        .get(MoveId::Grab as u8)
        .hitboxes
        .iter()
        .all(|h| h.kind == HIT_GRAB));
}

#[test]
fn the_throws_release_on_their_frames_for_their_damage() {
    // (stick, the frame the held fighter's percent rises on, damage in tenths)
    for (stick, frame, damage) in [
        ((127i8, 0i8), 20u8, 90),
        ((-127, 0), 20, 100),
        ((0, 127), 17, 80),
        ((0, -127), 33, 50),
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
        assert_eq!(rises, vec![frame], "throw {stick:?}");
        assert_eq!(
            sim.fighter(1).percent - before,
            dealt(&sim, damage),
            "throw {stick:?}"
        );
        if stick.0 < 0 {
            // The back throw: away behind, and the thrower ends facing the way it threw.
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
    let plain = duel().content.fighters[usize::from(SEER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window; the tick
    // adds one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 10u16, 12u8, 39u16, 1u16),
        (MoveId::FAir, 12, 9, 23, 1),
        (MoveId::BAir, 12, 10, 34, 0),
        (MoveId::UAir, 12, 14, 41, 0),
        (MoveId::DAir, 20, 11, 47, 0),
    ] {
        assert_eq!(land_during(id, mid), (S::Landing, lag), "{id:?} mid-move");
        assert_eq!(land_during(id, late), (S::Landing, plain), "{id:?} late");
        assert_eq!(land_during(id, early).1, plain, "{id:?} early");
    }
}

// ---- Body ----------------------------------------------------------------------------------------------------------------------

#[test]
fn the_seers_jumps_have_the_references_air_times() {
    // Short hop 34, full hop 50, short hop fast fall 24, full hop fast fall 35 (ultimateframedata.com); within one frame.
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
        (true, false, 50),
        (false, true, 24),
        (true, true, 35),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_seer_is_the_references_tall_middleweight() {
    let p = duel().content.fighters[usize::from(SEER)];
    let su = sim_core::FighterParams::su;
    assert_eq!(p.weight, Fx::from_int(91));
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(1000), su(120), su(1550), su(2480))
    );
    assert_eq!(
        (p.air_accel, p.air_accel_stick, p.air_friction),
        (su(10), su(105), su(10))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (49, 70, 82, 105)
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
    assert_eq!((p.dash_frames, p.heavy_landing_lag), (10, 6));
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (0, su(1300), su(2200))
    );
    assert_eq!(p.ledge_jump_dx, su(600));
    assert_eq!(p.weapon, SEER);
    assert!(p.ecb_height > sim_core::FighterParams::tumbler().ecb_height);
    assert!(!staff(&duel()).limbs, "a held staff");
}

// ---- Determinism ---------------------------------------------------------------------------------------------------------------

#[test]
fn the_new_fields_are_in_the_content_hash() {
    let base = Content::placeholder();
    let h = base.hash();
    let w = usize::from(SEER);
    let mut c = base.clone();
    c.weapons[w].moves[MoveId::NAir as usize].hitboxes[0].angle_type = ANGLE_FORWARD;
    assert_ne!(h, c.hash(), "angle type");
    let mut c = base.clone();
    let late = c.weapons[w].moves[MoveId::NAir as usize]
        .hitboxes
        .iter()
        .position(|hb| hb.hitstun_add == -13)
        .unwrap();
    c.weapons[w].moves[MoveId::NAir as usize].hitboxes[late].hitstun_add = 13;
    assert_ne!(h, c.hash(), "the modifier's sign");
}

#[test]
fn random_play_with_the_seer_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [SEER, 0, SEER, 4];
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
