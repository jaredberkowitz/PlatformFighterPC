//! The Embers moveset on the tumbler body (class 3), modelled mechanically on a reference character (phase 1: the body and every
//! normal; the specials are still the claws'). Frame data against the reference for every normal, checked in the data and by running
//! the moves; hit properties; the jab's continuable windows and repeated jab; the angled forward tilt and smash; the down air's loop,
//! final hit and landing hit; landing lag and autocancels; grabs, pummel and throws; the body's jump air times; and determinism.
//! Reference data and sources: `docs/movesets/mario/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::{MoveId, Weapon, AIM_ANY, AIM_DOWN, AIM_STRAIGHT, AIM_UP, HIT_GRAB};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const TUMBLER: u8 = 3;

/// The tumbler (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([TUMBLER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn embers(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(TUMBLER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R] except the pummel's frame and FAF, [D]: see there).
const FRAME_DATA: [FrameData; 26] = [
    (MoveId::Jab, &[(2, 3)], 20, 0, 0, 255),
    (MoveId::Jab2, &[(2, 3)], 22, 0, 0, 255),
    (MoveId::Jab3, &[(3, 4)], 34, 0, 0, 255),
    (MoveId::FTilt, &[(5, 7)], 26, 0, 0, 255),
    (
        MoveId::UTilt,
        &[(5, 11), (5, 6), (7, 9), (10, 11)],
        30,
        0,
        0,
        255,
    ),
    (MoveId::DTilt, &[(5, 7)], 28, 0, 0, 255),
    (MoveId::DashAttack, &[(6, 9), (10, 25)], 38, 0, 0, 255),
    (MoveId::FSmash, &[(15, 17)], 48, 0, 0, 255),
    (MoveId::USmash, &[(9, 10), (11, 12)], 40, 0, 0, 255),
    (MoveId::DSmash, &[(5, 6), (14, 14)], 44, 0, 0, 255),
    (MoveId::NAir, &[(3, 5), (6, 27)], 46, 6, 2, 39),
    (MoveId::FAir, &[(16, 16), (17, 20), (21, 21)], 60, 17, 2, 43),
    (MoveId::BAir, &[(6, 7), (8, 10)], 34, 6, 5, 19),
    (MoveId::UAir, &[(4, 5), (6, 7)], 31, 6, 1, 17),
    (
        MoveId::DAir,
        &[(5, 5), (7, 7), (9, 9), (11, 11), (13, 13), (23, 23)],
        38,
        15,
        4,
        33,
    ),
    (MoveId::Ext0, &[(1, 2)], 16, 0, 0, 255),
    (MoveId::Grab, &[(6, 7)], 35, 0, 0, 255),
    (MoveId::DashGrab, &[(9, 10)], 43, 0, 0, 255),
    (MoveId::PivotGrab, &[(10, 11)], 37, 0, 0, 255),
    // (The pummel: the reference's hit on frame 1 and FAF 7, one frame later here [D].)
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(13, 13)], 28, 0, 0, 255),
    (MoveId::BThrow, &[(44, 44), (13, 39)], 60, 0, 0, 255),
    (MoveId::UThrow, &[(18, 18)], 40, 0, 0, 255),
    (MoveId::DThrow, &[(18, 18)], 40, 0, 0, 255),
    (MoveId::LedgeAttack, &[(24, 26)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(16, 17), (21, 22)], 46, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = embers(&sim);
    for (id, windows, faf, landing, ac_before, ac_from) in FRAME_DATA {
        let mv = w.get(id as u8);
        // Every hitbox lies in one of the reference's windows, and every window has hitboxes (move frames are the reference's
        // minus one).
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
    // The ledge attack's length is the body's (the state it runs in), the reference's FAF 56 too.
    assert_eq!(
        sim.content.fighters[usize::from(TUMBLER)].ledge_attack_frames,
        55
    );
    // Intangibility: the ledge attack through frame 26, the get-up attack through 22.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 26);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 22);
    // Smash charge frames: forward on 5, up on 6, down on 2 (move frames one less).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(4));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(5));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(1));
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (in the air high up for an aerial, else standing; aimed `aim`), with player 1 pinned where the move's
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
    let hb = *embers(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| h.start + 1 == first && (h.aim == AIM_ANY || h.aim == wanted))
        .expect("a hitbox in that window");
    let before = sim.fighter(1).percent;
    // (Move frame k is reached on tick k: the move began before tick 1.)
    for tick in 1..=80u8 {
        let me = sim.state.fighters[0].pos;
        // The target's middle hurtbox on the hitbox's centre, held there (no fall, no launch) until it is hit.
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
    // (move, the first frame of its first window, aim)
    for (id, first, aim) in [
        (MoveId::Jab, 2u8, 0i8),
        (MoveId::Jab2, 2, 0),
        (MoveId::Jab3, 3, 0),
        (MoveId::FTilt, 5, 0),
        (MoveId::FTilt, 5, 1),
        (MoveId::FTilt, 5, -1),
        (MoveId::UTilt, 5, 0),
        (MoveId::DTilt, 5, 0),
        (MoveId::DashAttack, 6, 0),
        (MoveId::FSmash, 15, 0),
        (MoveId::FSmash, 15, 1),
        (MoveId::FSmash, 15, -1),
        (MoveId::USmash, 9, 0),
        (MoveId::DSmash, 5, 0),
        (MoveId::DSmash, 14, 0),
        (MoveId::NAir, 3, 0),
        (MoveId::FAir, 16, 0),
        (MoveId::BAir, 6, 0),
        (MoveId::UAir, 4, 0),
        (MoveId::DAir, 5, 0),
    ] {
        // (A later window is checked only where a target there is out of the earlier windows' reach.)
        let got = first_hit_frame(id, first, aim);
        assert_eq!(got, Some(first), "{id:?} window from {first}, aim {aim}");
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
            // Out of the move at the end of move frame FAF - 2; the next frame (FAF) acts.
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
            MoveId::Ext0
                | MoveId::Pummel
                | MoveId::FThrow
                | MoveId::BThrow
                | MoveId::UThrow
                | MoveId::DThrow
                | MoveId::LedgeAttack
        ) {
            continue;
        }
        let aerial = id.is_aerial();
        assert_eq!(faf_of(id, aerial), u16::from(faf), "{id:?}");
    }
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority, damage in tenths, angle, BK, KS) for the main box of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 33] = [
        (MoveId::Jab, 2, 0, 22, 361, 20, 30),
        (MoveId::Jab, 2, 2, 22, 180, 15, 20),
        (MoveId::Jab2, 2, 0, 17, 361, 20, 30),
        (MoveId::Jab2, 2, 2, 17, 361, 15, 15),
        (MoveId::Jab3, 3, 0, 40, 361, 60, 80),
        (MoveId::FTilt, 5, 0, 70, 361, 55, 70),
        (MoveId::UTilt, 5, 0, 55, 96, 28, 130),
        (MoveId::DTilt, 5, 0, 70, 80, 35, 80),
        (MoveId::DTilt, 5, 1, 50, 80, 35, 80),
        (MoveId::DashAttack, 6, 0, 80, 50, 100, 43),
        (MoveId::DashAttack, 10, 0, 60, 48, 100, 30),
        (MoveId::FSmash, 15, 0, 147, 361, 25, 105),
        (MoveId::FSmash, 15, 1, 178, 361, 25, 99),
        (MoveId::USmash, 9, 0, 140, 83, 32, 94),
        (MoveId::DSmash, 5, 0, 100, 32, 30, 100),
        (MoveId::DSmash, 14, 0, 120, 30, 30, 100),
        (MoveId::NAir, 3, 0, 80, 361, 20, 100),
        (MoveId::NAir, 6, 0, 50, 361, 13, 90),
        (MoveId::FAir, 16, 0, 120, 361, 30, 80),
        (MoveId::FAir, 17, 0, 140, 280, 32, 78),
        (MoveId::FAir, 21, 0, 100, 361, 20, 80),
        (MoveId::BAir, 6, 0, 105, 361, 12, 106),
        (MoveId::BAir, 8, 0, 70, 361, 7, 90),
        (MoveId::UAir, 4, 0, 70, 75, 10, 135),
        (MoveId::DAir, 5, 0, 14, 94, 50, 15),
        (MoveId::DAir, 5, 1, 14, 94, 25, 15),
        (MoveId::DAir, 23, 0, 55, 75, 80, 100),
        (MoveId::Ext0, 1, 0, 20, 361, 0, 100),
        (MoveId::FThrow, 13, 0, 80, 45, 60, 65),
        (MoveId::BThrow, 44, 0, 110, 45, 70, 66),
        (MoveId::UThrow, 18, 0, 70, 90, 70, 72),
        (MoveId::DThrow, 18, 0, 50, 68, 40, 90),
        (MoveId::LedgeAttack, 24, 0, 90, 45, 90, 20),
    ];
    let sim = duel();
    let w = embers(&sim);
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
    // The other multipliers: hitlag, set knockback and set weight, SDI, shield damage.
    let get = |id: MoveId| w.get(id as u8);
    assert!(get(MoveId::Jab).hitboxes.iter().all(|h| h.hitlag == 180));
    assert!(get(MoveId::Jab2).hitboxes.iter().all(|h| h.hitlag == 120));
    assert!(get(MoveId::Jab3).hitboxes.iter().all(|h| h.hitlag == 200));
    assert!(get(MoveId::DashAttack)
        .hitboxes
        .iter()
        .all(|h| h.hitlag == 125));
    let pummel = get(MoveId::Pummel).hitboxes[0];
    assert_eq!(
        (pummel.damage, pummel.fixed_kb, pummel.hitlag),
        (fx(13, 10), 30, 210)
    );
    let dair = get(MoveId::DAir);
    assert!(dair.hitboxes.iter().all(|h| h.sdi == 80));
    assert!(dair.hitboxes.iter().all(|h| h.set_weight == (h.group == 0)));
    assert!(get(MoveId::Ext0).hitboxes.iter().all(|h| h.fixed_kb == 60));
    assert!(get(MoveId::GetUpAttack)
        .hitboxes
        .iter()
        .all(|h| h.shield_damage == 214 && h.angle == 48));
    // The angled forward smash: the explosion's growth is 101 aimed up and 100 aimed down.
    let fire = |aim: u8| {
        get(MoveId::FSmash)
            .hitboxes
            .iter()
            .find(|h| h.aim == aim && h.priority == 1)
            .unwrap()
            .knockback_growth
    };
    assert_eq!(
        (fire(AIM_STRAIGHT), fire(AIM_UP), fire(AIM_DOWN)),
        (99, 101, 100)
    );
}

#[test]
fn the_forward_smash_arm_wins_over_the_explosion_where_both_reach() {
    // Close in, both boxes overlap the target: the arm (14.7%) is the hit, so the explosion (17.8%) has to be spaced.
    let damage_at = |gap: Fx| {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, gap, -1);
        sim.state.fighters[1].percent = Fx::ZERO;
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
        for _ in 0..20 {
            sim.tick(inp(0, 0, 0));
        }
        sim.fighter(1).percent
    };
    let sim = duel();
    assert_eq!(damage_at(fx(8, 10)), dealt(&sim, 147), "close: the arm");
    assert_eq!(
        damage_at(fx(20, 10)),
        dealt(&sim, 178),
        "spaced: the explosion"
    );
}

// ---- Jab ---------------------------------------------------------------------------------------------------------------------

#[test]
fn pressing_attack_again_goes_jab_jab_kick_on_the_continuable_frames() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim.state.fighters[1].invuln = 255;
    // Tick t is jab 1's reference frame t. A second press early (frame 3) waits for frame 5.
    sim.tick(inp(0, 0, ATTACK));
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, MoveId::Jab as u8, "frame 4: still jab 1");
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        (sim.f().move_id, sim.f().state_frame),
        (MoveId::Jab2 as u8, 0),
        "jab 2's first frame is jab 1's frame 5"
    );
    // A press in jab 2 carries on into the kick on jab 2's frame 6.
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..3 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().move_id, MoveId::Jab2 as u8, "jab 2 frame 5");
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        (sim.f().move_id, sim.f().state_frame),
        (MoveId::Jab3 as u8, 0),
        "the kick starts on jab 2's frame 6"
    );
    // The kick ends the chain.
    for _ in 0..40 {
        sim.tick(inp(0, 0, ATTACK));
        sim.tick(inp(0, 0, 0));
        if sim.f().move_id != MoveId::Jab3 as u8 {
            break;
        }
    }
    assert_ne!(sim.f().move_id, MoveId::Jab2 as u8);
}

#[test]
fn holding_attack_throws_jab_1_again_from_frame_7() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim.state.fighters[1].invuln = 255;
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..5 {
        sim.tick(inp(0, 0, ATTACK));
    }
    assert_eq!(
        (sim.f().move_id, sim.f().state_frame),
        (MoveId::Jab as u8, 5)
    );
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(
        (sim.f().move_id, sim.f().state_frame),
        (MoveId::Jab as u8, 0),
        "held, jab 1 starts over on its frame 7"
    );
    // Held on, it keeps repeating jab 1 and never goes on to jab 2.
    for _ in 0..40 {
        sim.tick(inp(0, 0, ATTACK));
        assert_eq!(sim.f().move_id, MoveId::Jab as u8);
    }
    // Let go, it ends on its FAF.
    sim.ticks(25, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn a_single_jab_hits_for_2_2_percent_and_ends_on_frame_20() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim.tick(inp(0, 0, ATTACK));
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 22));
    assert_eq!(faf_of(MoveId::Jab, false), 20);
}

// ---- Angled forward tilt and smash -------------------------------------------------------------------------------------------

#[test]
fn the_stick_tilted_up_or_down_aims_the_forward_tilt_and_smash() {
    // (stick, the move, the aim)
    for (x, y, id, aim) in [
        (60i8, 0i8, MoveId::FTilt, 0i8),
        (60, 40, MoveId::FTilt, 1),
        (60, -40, MoveId::FTilt, -1),
        (127, 0, MoveId::FSmash, 0),
        (127, 50, MoveId::FSmash, 1),
        (127, -50, MoveId::FSmash, -1),
    ] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_int(10), -1);
        sim.tick(inp(x, y, ATTACK));
        assert_eq!(
            (MoveId::from_index(sim.f().move_id), sim.f().aim),
            (id, aim),
            "stick ({x}, {y})"
        );
        // Only that aim's boxes come out.
        let active = if id == MoveId::FTilt { 4 } else { 14 };
        while sim.f().state_frame < active {
            sim.tick(inp(0, 0, 0));
        }
        let wanted = match aim {
            1 => AIM_UP,
            -1 => AIM_DOWN,
            _ => AIM_STRAIGHT,
        };
        let w = embers(&sim);
        let live: Vec<_> =
            sim_core::combat::active_hitboxes(sim.f(), w.get(sim.f().move_id), Fx::ONE)
                .map(|(_, h, _)| h.aim)
                .collect();
        assert!(!live.is_empty(), "stick ({x}, {y}): boxes out");
        assert!(
            live.iter().all(|&a| a == wanted),
            "stick ({x}, {y}): {live:?}"
        );
    }
    // Another fighter's moves (no aimed boxes) are the same whatever the aim.
    let sim = duel();
    let longsword = &sim.content.weapons[0];
    assert!(longsword
        .moves
        .iter()
        .all(|m| m.hitboxes.iter().all(|h| h.aim == AIM_ANY)));
}

#[test]
fn an_upward_forward_tilt_hits_a_target_above_the_straight_one() {
    // A target standing on a ledge of air above the straight kick: only the kick aimed up reaches it.
    let hit = |aim: i8| {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FTilt);
        sim.state.fighters[0].aim = aim;
        let before = sim.fighter(1).percent;
        for _ in 0..10 {
            sim.put_airborne(1, fx(11, 10), fx(11, 10), Fx::ZERO, Fx::ZERO);
            sim.tick(inp(0, 0, 0));
        }
        sim.fighter(1).percent > before
    };
    assert!(hit(1), "aimed up, the kick reaches it");
    assert!(!hit(-1), "aimed down, it does not");
}

// ---- Down air ----------------------------------------------------------------------------------------------------------------

#[test]
fn down_air_lands_five_looping_hits_and_its_final_hit_once() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    let mut hits = Vec::new();
    let mut last = sim.fighter(1).percent;
    // The target pinned on the spinner's body all through the move (out of hitlag the sim may move it; it is put back).
    for tick in 1..=120u8 {
        if sim.fighter(1).hitlag == 0 {
            let me = sim.f().pos;
            sim.put_airborne(1, me.x, me.y, Fx::ZERO, Fx::ZERO);
        }
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            last = sim.fighter(1).percent;
            hits.push(sim.f().state_frame + 1);
        }
        if sim.f().state != S::Attack {
            let _ = tick;
            break;
        }
    }
    assert_eq!(
        hits,
        vec![5, 7, 9, 11, 13, 23],
        "the hits' reference frames"
    );
    assert_eq!(
        sim.fighter(1).percent,
        dealt(&sim, 14).mul_int(5) + dealt(&sim, 55)
    );
}

#[test]
fn the_down_air_loop_is_set_weight_and_its_final_hit_is_not() {
    // The same loop hit on a light (the rogue, 78) and a heavy (the tumbler itself, 98) target launches both the same; the final hit
    // sends the light one further.
    let launch = |victim: u8, id_frame: u8| {
        let mut sim = Sim::with_chars([TUMBLER, victim, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.state.fighters[1].percent = Fx::from_int(50);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
        sim.state.fighters[0].state_frame = u16::from(id_frame - 2);
        sim.state.fighters[0].hit_mask = 0;
        for _ in 0..30 {
            if sim.fighter(1).hitlag == 0 && sim.fighter(1).percent == Fx::from_int(50) {
                let me = sim.f().pos;
                sim.put_airborne(1, me.x, me.y, Fx::ZERO, Fx::ZERO);
            }
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).percent > Fx::from_int(50) && !sim.fighter(1).launch_pending {
                return sim.fighter(1).kb_vel.length();
            }
        }
        panic!("no hit");
    };
    let (light, heavy) = (launch(2, 5), launch(TUMBLER, 5));
    assert!(
        (light - heavy).abs() <= Fx::from_raw(64),
        "loop: {light:?} vs {heavy:?}"
    );
    let (light, heavy) = (launch(2, 23), launch(TUMBLER, 23));
    assert!(light > heavy, "final: {light:?} vs {heavy:?}");
}

/// Lands player 0 in the middle of `id` on the tick that takes it to move frame `frame_before_tick + 1`; returns its state, lag and
/// move after landing.
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
    let plain = duel().content.fighters[usize::from(TUMBLER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late autocancel window)
    for (id, mid, lag, late) in [
        (MoveId::NAir, 10u16, 6u8, 40u16),
        (MoveId::FAir, 20, 17, 44),
        (MoveId::BAir, 10, 6, 20),
        (MoveId::UAir, 8, 6, 18),
    ] {
        assert_eq!(
            land_during(id, mid),
            (S::Landing, lag, id as u8),
            "{id:?} mid-move"
        );
        let (state, got, _) = land_during(id, late);
        assert_eq!(
            (state, got),
            (S::Landing, plain),
            "{id:?} autocancelled late"
        );
    }
    // The early windows: the neutral air's frames 1-2 and the back air's 1-5 autocancel; the up air's only frame 1.
    assert_eq!(land_during(MoveId::BAir, 3).1, plain);
    assert_eq!(land_during(MoveId::UAir, 2).1, 6);
}

#[test]
fn a_down_air_landing_is_the_landing_hit_unless_it_autocancels() {
    // Mid-move (frame 10): the landing hit, its own 15-frame move with a 2% hit on its frames 1-2.
    let (state, _, mv) = land_during(MoveId::DAir, 10);
    assert_eq!((state, mv), (S::Attack, MoveId::Ext0 as u8));
    // In the late autocancel window (from frame 33): a plain landing.
    let (state, _, _) = land_during(MoveId::DAir, 34);
    assert_eq!(state, S::Landing);
    // The landing hit hits a fighter standing next to it.
    let mut sim = duel();
    sim.stand(1, fx(-85, 10), -1);
    sim.put_airborne(0, Fx::from_int(-9), fx(2, 100), Fx::ZERO, fx(-1, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = MoveId::DAir as u8;
    f.state_frame = 10;
    f.hit_mask = 0;
    sim.ticks(4, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 20));
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
    let w = embers(&sim);
    assert!(w
        .get(MoveId::Grab as u8)
        .hitboxes
        .iter()
        .all(|h| h.kind == HIT_GRAB));
}

#[test]
fn the_throws_release_on_their_frames_for_their_damage() {
    // (stick, release frame, damage in tenths)
    for (stick, frame, damage) in [
        ((127i8, 0i8), 13u8, 80),
        ((-127, 0), 44, 110),
        ((0, 127), 18, 70),
        ((0, -127), 18, 50),
    ] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(12, 10), -1);
        grab_now(&mut sim);
        let before = sim.fighter(1).percent;
        sim.tick(inp(stick.0, stick.1, 0));
        let mut hit = None;
        for t in 2..=110u8 {
            sim.tick(inp(0, 0, 0));
            if hit.is_none() && sim.fighter(1).percent > before {
                hit = Some(t);
            }
        }
        assert_eq!(hit, Some(frame), "throw {stick:?}");
        assert_eq!(
            sim.fighter(1).percent - before,
            dealt(&sim, damage),
            "throw {stick:?}"
        );
        if stick.0 < 0 {
            // The back throw: thrown away behind, and the thrower ends facing the way it threw.
            assert!(sim.fighter(1).pos.x < Fx::ZERO, "thrown behind");
            assert_eq!(sim.f().facing, -1, "facing the throw");
        }
    }
}

#[test]
fn the_back_throw_hits_a_bystander_while_it_spins_but_not_the_held_fighter() {
    let mut sim = Sim::with_chars([TUMBLER, 0, 0, 0]);
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim.stand(2, fx(-9, 10), 1);
    grab_now(&mut sim);
    let held = sim.fighter(1).percent;
    sim.tick(inp(-127, 0, 0));
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
        // (The bystander is held in place until the spin reaches it.)
        sim.stand(2, fx(-9, 10), 1);
    }
    sim.ticks(100, inp(0, 0, 0));
    assert_eq!(
        sim.fighter(2).percent,
        dealt(&sim, 80),
        "the bystander takes the spin's 8%"
    );
    assert_eq!(
        sim.fighter(1).percent - held,
        dealt(&sim, 110),
        "the held fighter only the throw's 11%"
    );
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

// ---- Body ----------------------------------------------------------------------------------------------------------------------

#[test]
fn the_tumblers_jumps_have_the_references_air_times() {
    // Short hop 40, full hop 56, short hop fast fall 28, full hop fast fall 39 (ultimateframedata.com); within one frame.
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
        (false, false, 40u32),
        (true, false, 56),
        (false, true, 28),
        (true, true, 39),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_tumbler_is_the_references_middleweight() {
    let p = duel().content.fighters[usize::from(TUMBLER)];
    let su = sim_core::FighterParams::su;
    assert_eq!(p.weight, Fx::from_int(98));
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(1208), su(87), su(1500), su(2400))
    );
    assert_eq!(
        (p.air_accel, p.air_accel_stick, p.air_friction),
        (su(10), su(70), su(15))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (52, 71, 87, 116)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (29, 34, 25)
    );
    assert_eq!(p.weapon, TUMBLER);
}

// ---- Determinism ---------------------------------------------------------------------------------------------------------------

#[test]
fn the_new_hitbox_fields_are_in_the_content_hash() {
    let base = Content::placeholder();
    let h = base.hash();
    let mut c = base.clone();
    c.weapons[3].moves[MoveId::FTilt as usize].hitboxes[0].aim = AIM_ANY;
    assert_ne!(h, c.hash(), "aim");
    let mut c = base.clone();
    c.weapons[3].moves[MoveId::DAir as usize].hitboxes[0].set_weight = false;
    assert_ne!(h, c.hash(), "set weight");
    let mut c = base;
    c.weapons[3].moves[MoveId::Jab as usize].next_from = 255;
    assert_ne!(h, c.hash(), "next from");
}

#[test]
fn random_play_with_the_tumbler_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [TUMBLER, 0, TUMBLER, 2];
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
        // Snapshot in the middle, finish from the copy: identical to the straight run.
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

// ---- Up special: the rising punch (phase 2) -----------------------------------------------------------------------------------

#[test]
fn the_rising_punch_has_the_references_hits() {
    use sim_core::moves::{TARGETS_AERIAL, TARGETS_ALL};
    let sim = duel();
    let mv = embers(&sim).get(MoveId::UpSpecial as u8);
    // (reference frames, group, damage in tenths, angle, BK, KS, set knockback, set weight, SDI, targets)
    let rows: Vec<_> = mv
        .hitboxes
        .iter()
        .map(|h| {
            (
                (h.start + 1, h.end + 1, h.group),
                (h.damage * Fx::from_int(10) + Fx::from_ratio(1, 100)).floor_int(),
                (h.angle, h.base_knockback, h.knockback_growth, h.fixed_kb),
                (h.set_weight, h.sdi, h.targets),
            )
        })
        .collect();
    let set = (true, 0, TARGETS_ALL);
    assert_eq!(
        rows,
        vec![
            ((3, 6, 0), 50, (60, 0, 100, 160), (true, 0, TARGETS_AERIAL)),
            ((3, 6, 0), 50, (86, 0, 100, 150), set),
            ((3, 6, 0), 50, (100, 0, 100, 150), set),
            ((7, 16, 1), 6, (60, 0, 100, 180), set),
            ((7, 16, 1), 6, (92, 0, 100, 170), set),
            ((7, 16, 1), 6, (60, 0, 100, 110), set),
            ((7, 16, 1), 6, (92, 0, 100, 110), set),
            ((17, 18, 2), 30, (60, 50, 145, 0), (false, 100, TARGETS_ALL)),
            ((17, 18, 2), 30, (60, 50, 145, 0), (false, 100, TARGETS_ALL)),
        ]
    );
    // Hits 2-6 land again every 2 frames from frame 7 (on 7, 9, 11, 13 and 15); hits 1 and 7 once.
    assert_eq!(mv.rehit, Some((6, 2)));
    assert_eq!(mv.rehit_groups, 0b10);
    // The hit radii, converted [D]: 2.5 / 4.0 / 4.0, 3.0 / 3.8 / 3.0 / 3.8 and 9.0 reference units.
    let radii: Vec<i32> = mv
        .hitboxes
        .iter()
        .map(|h| (h.radius * Fx::from_int(10) + Fx::from_ratio(1, 100)).floor_int())
        .collect();
    assert_eq!(radii, vec![3, 5, 5, 4, 5, 4, 5, 11, 11]);
}

/// The tumbler using its up special with a duelist floating at `(dx, dy)` from it; the ticks on which the duelist's percent rose.
fn rising_punch_hits(dx: Fx, dy: Fx, attacker_percent: Fx) -> (Vec<usize>, Sim) {
    let mut sim = duel();
    sim.stand(0, fx(-3, 1), 1);
    sim.state.fighters[0].percent = attacker_percent;
    sim.put_airborne(1, fx(-3, 1) + dx, dy, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].facing = -1;
    let mut ticks = Vec::new();
    let mut last = Fx::ZERO;
    for t in 1..=80 {
        sim.tick(if t == 1 {
            inp(0, 127, sim_core::input::buttons::SPECIAL)
        } else {
            inp(0, 0, 0)
        });
        if sim.fighter(1).percent > last {
            last = sim.fighter(1).percent;
            ticks.push(t);
        }
    }
    (ticks, sim)
}

#[test]
fn the_rising_punch_carries_a_fighter_through_all_seven_hits_for_11_percent() {
    let (ticks, sim) = rising_punch_hits(fx(6, 10), fx(6, 10), Fx::ZERO);
    assert_eq!(ticks.len(), 7, "{ticks:?}");
    assert_eq!(ticks[0], 3, "the first hit on frame 3");
    let total = sim.fighter(1).percent;
    assert!(
        (total - dealt(&sim, 110)).abs() < Fx::from_ratio(1, 100),
        "{total:?}"
    );
}

#[test]
fn the_rising_punchs_set_knockback_ignores_rage() {
    let (_, calm) = rising_punch_hits(fx(6, 10), fx(6, 10), Fx::ZERO);
    let (_, raging) = rising_punch_hits(fx(6, 10), fx(6, 10), Fx::from_int(150));
    // Up to the last hit (whose knockback is not set) the two victims are thrown exactly alike; the angry one's last hit is harder.
    let first_launch = |ticks_to: usize, rage: Fx| {
        let mut sim = duel();
        sim.stand(0, fx(-3, 1), 1);
        sim.state.fighters[0].percent = rage;
        sim.put_airborne(1, fx(-24, 10), fx(6, 10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 127, sim_core::input::buttons::SPECIAL));
        sim.ticks(ticks_to - 1, inp(0, 0, 0));
        sim.fighter(1).launch_kb
    };
    assert_eq!(
        first_launch(3, Fx::ZERO),
        first_launch(3, Fx::from_int(150))
    );
    assert!(raging.fighter(1).launch_kb >= calm.fighter(1).launch_kb);
}

#[test]
fn the_rising_punch_is_intangible_on_frames_3_to_6() {
    for air in [false, true] {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(9), -1);
        if air {
            sim.put_airborne(0, fx(-3, 1), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
        } else {
            sim.stand(0, fx(-3, 1), 1);
        }
        let mut seen = Vec::new();
        for t in 1..=10 {
            sim.tick(if t == 1 {
                inp(0, 127, sim_core::input::buttons::SPECIAL)
            } else {
                inp(0, 0, 0)
            });
            if sim_core::combat::is_intangible(sim.f()) {
                seen.push(t);
            }
        }
        assert_eq!(seen, vec![3, 4, 5, 6], "air {air}");
    }
}

#[test]
fn the_rising_punch_ends_in_a_special_fall_drifting_at_0_6_that_lands_with_30_frames_of_lag() {
    let mut sim = duel();
    sim.put_airborne(0, fx(-2, 1), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 127, sim_core::input::buttons::SPECIAL));
    sim.ticks(35, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack, "tick 36");
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Helpless, "the 37-frame animation is over");
    for _ in 0..60 {
        sim.tick(inp(127, 0, 0));
    }
    let cap = sim.content.fighters[usize::from(TUMBLER)].air_speed * fx(6, 10);
    assert!(sim.f().vel.x > Fx::ZERO && sim.f().vel.x <= cap + Fx::from_raw(64));
    sim.state.fighters[0].pos = Vec2::new(Fx::ZERO, fx(5, 10));
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            break;
        }
    }
    assert_eq!(sim.f().lag, 30);
}

/// The tick the tumbler catches the left ledge using its up special while held in reach of it (1.2 outside, 1.8 below, put back
/// there before every frame), facing `facing`.
fn rising_punch_ledge_tick(facing: i8) -> Option<usize> {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(9), -1);
    let hold = |sim: &mut Sim| {
        sim.state.fighters[0].pos = Vec2::new(fx(-122, 10), fx(-18, 10));
        sim.state.fighters[0].platform = sim_core::state::NONE;
    };
    hold(&mut sim);
    sim.state.fighters[0].state = S::Airborne;
    sim.state.fighters[0].facing = facing;
    sim.tick(inp(0, 127, sim_core::input::buttons::SPECIAL));
    (2..=37).find(|_| {
        hold(&mut sim);
        sim.tick(inp(0, 0, 0));
        sim.f().state == S::LedgeHang
    })
}

#[test]
fn the_rising_punch_catches_a_ledge_on_either_side_from_frame_14() {
    assert_eq!(rising_punch_ledge_tick(1), Some(14));
    assert_eq!(rising_punch_ledge_tick(-1), Some(14));
}

#[test]
fn the_rising_punch_rises_higher_from_the_ground_than_from_the_air() {
    let rise = |air: bool| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(9), -1);
        if air {
            sim.put_airborne(0, fx(-3, 1), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
        } else {
            sim.stand(0, fx(-3, 1), 1);
        }
        sim.tick(inp(0, 127, sim_core::input::buttons::SPECIAL));
        sim.ticks(1, inp(0, 0, 0));
        let y0 = sim.f().pos.y;
        sim.ticks(18, inp(0, 0, 0));
        sim.f().pos.y - y0
    };
    let (ground, air) = (rise(false), rise(true));
    // 39.6 reference units of rise in the motion [E], x1.1 from the ground and x1.08 from the air.
    let units = |n: i32| Fx::from_ratio(n, 8000);
    assert!((ground - units(43_560)).abs() < units(200), "{ground:?}");
    assert!((air - units(42_768)).abs() < units(200), "{air:?}");
}

#[test]
fn the_stick_angles_the_rising_punch_by_up_to_12_degrees() {
    let lean = |stick: i8| {
        let mut sim = duel();
        sim.stand(0, fx(-3, 1), 1);
        sim.tick(inp(0, 127, sim_core::input::buttons::SPECIAL));
        sim.tick(inp(stick, 0, 0));
        sim.tick(inp(stick, 0, 0));
        sim.f().steer
    };
    assert_eq!(lean(0), 0);
    assert_eq!(lean(127), 12);
    assert_eq!(lean(50), 0, "inside the stick sensitivity (0.5)");
}

// ---- Neutral special: the fireball (phase 2) ----------------------------------------------------------------------------------

/// The tumbler at `x` facing right throws a fireball; player 1 is parked out of the way unless placed. Ticks until it appears.
fn throw_fireball(sim: &mut Sim) -> usize {
    sim.tick(inp(0, 0, sim_core::input::buttons::SPECIAL));
    for t in 2..=30 {
        sim.tick(inp(0, 0, 0));
        if sim.state.projectiles.iter().any(|p| p.active) {
            return t;
        }
    }
    panic!("no fireball");
}

fn the_ball(sim: &Sim) -> Option<sim_core::Projectile> {
    sim.state.projectiles.iter().copied().find(|p| p.active)
}

#[test]
fn the_fireball_hits_from_frame_17_and_the_move_ends_on_frame_50() {
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-10), 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    // It appears at the end of frame 16, so its hit is out from frame 17 (a projectile hits from the frame after it appears).
    assert_eq!(throw_fireball(&mut sim), 16);
    // (Tick 16 done: 33 more to the end of tick 49, the move's last.)
    sim.ticks(32, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle, "actionable on frame 50");
}

#[test]
fn the_fireball_falls_bounces_off_the_ground_and_hits_for_70_frames() {
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-10), 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    throw_fireball(&mut sim);
    let first = the_ball(&sim).unwrap();
    // 1.4 reference units a frame, 10 degrees down.
    let units = |n: i32| Fx::from_ratio(n, 8000);
    assert!(
        (first.vel.x - units(1379)).abs() < units(5),
        "{:?}",
        first.vel
    );
    assert!(
        (first.vel.y + units(243)).abs() < units(5),
        "{:?}",
        first.vel
    );
    let mut frames = 0;
    let mut bounces = 0;
    let mut peak_after_bounce = Fx::ZERO;
    let mut last_vy = first.vel.y;
    while let Some(p) = the_ball(&sim) {
        if p.vel.y > Fx::ZERO && last_vy < Fx::ZERO {
            bounces += 1;
            // Off the ground at 1.4 to 1.6, keeping 0.97 of its forward speed.
            assert!(
                p.vel.y >= units(1400) && p.vel.y <= units(1600),
                "{:?}",
                p.vel
            );
            assert_eq!(p.bounces, bounces);
        }
        if bounces == 1 {
            peak_after_bounce = peak_after_bounce.max(p.pos.y);
        }
        last_vy = p.vel.y;
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    assert_eq!(
        frames, 71,
        "out for 71 frames, hitting on 70 of them (17-86)"
    );
    assert!(bounces >= 2, "{bounces}");
    // A bounce at 1.4 to 1.6 under gravity 0.1 rises 9.8 to 12.8 reference units.
    assert!(
        peak_after_bounce > units(9000) && peak_after_bounce < units(13_500),
        "{peak_after_bounce:?}"
    );
}

#[test]
fn the_fireball_is_gone_after_its_seventh_bounce() {
    let mut sim = duel();
    // A longer-lived ball, to see the bounce limit.
    let w = usize::from(sim.content.fighters[usize::from(TUMBLER)].weapon);
    let spec = sim.content.weapons[w].moves[MoveId::NSpecial as usize]
        .projectile
        .as_mut()
        .unwrap();
    spec.life = 255;
    spec.speed = Fx::ZERO;
    sim.stand(0, Fx::from_int(-10), 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    throw_fireball(&mut sim);
    let mut most = 0;
    for _ in 0..254 {
        match the_ball(&sim) {
            Some(p) => most = most.max(p.bounces),
            None => break,
        }
        sim.tick(inp(0, 0, 0));
    }
    assert!(
        the_ball(&sim).is_none(),
        "removed on the touch after the seventh bounce"
    );
    assert_eq!(most, 7);
}

/// Damage a fireball does to a duelist standing `gap` ahead of the tumbler, and the launch it gets.
fn fireball_hit(gap: Fx) -> (Fx, Fx, Sim) {
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-10), 1);
    sim.stand(1, Fx::from_int(-10) + gap, -1);
    sim.tick(inp(0, 0, sim_core::input::buttons::SPECIAL));
    for _ in 0..100 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            break;
        }
    }
    (sim.fighter(1).percent, sim.fighter(1).launch_kb, sim)
}

#[test]
fn the_fireballs_hit_weakens_with_its_age() {
    // Close: the clean hit (5%); a few units on: the late hit (5%, weaker knockback); far: the latest (4%).
    let (near, near_kb, _) = fireball_hit(fx(15, 10));
    let (mid, mid_kb, _) = fireball_hit(fx(40, 10));
    let (far, _, sim) = fireball_hit(fx(80, 10));
    assert_eq!(near, dealt(&sim, 50));
    assert_eq!(mid, dealt(&sim, 50));
    assert!(
        mid_kb < near_kb,
        "the late hit launches less: {mid_kb:?} {near_kb:?}"
    );
    assert_eq!(far, dealt(&sim, 40));
}

#[test]
fn a_shield_takes_half_the_fireballs_damage() {
    // The same 40 frames of a held shield, with and without a fireball into it: the most the shield loses in one frame (the hit,
    // with the shield's own steady drain).
    let shield_after = |throw: bool| {
        let mut sim = duel();
        sim.stand(0, Fx::from_int(-10), 1);
        sim.stand(1, Fx::from_int(-8), -1);
        let mut most = Fx::ZERO;
        for t in 1..=40 {
            let p0 = if throw && t == 1 {
                inp(0, 0, sim_core::input::buttons::SPECIAL)
            } else {
                inp(0, 0, 0)
            };
            let before = sim.fighter(1).shield_hp;
            sim.tick2(p0, inp(0, 0, sim_core::input::buttons::SHIELD));
            if t > 2 {
                most = most.max(before - sim.fighter(1).shield_hp);
            }
        }
        assert_eq!(sim.fighter(1).percent, Fx::ZERO);
        (most, sim)
    };
    let (hit, sim) = shield_after(true);
    let (drain, _) = shield_after(false);
    let lost = hit - drain;
    let half = dealt(&sim, 50) * Fx::from_ratio(50, 100);
    assert!(
        (lost - half).abs() < Fx::from_ratio(1, 100),
        "{lost:?} vs {half:?}"
    );
}

// ---- Side special: the cape (phase 2) ------------------------------------------------------------------------------------------

#[test]
fn the_cape_hits_on_frame_12_with_set_knockback_and_turns_the_victim_round() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(10, 10), -1);
    sim.tick(inp(127, 0, sim_core::input::buttons::SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
    let mut hit = None;
    for t in 2..=20 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            hit = Some(t);
            break;
        }
    }
    assert_eq!(hit, Some(12));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 70));
    // Set knockback 80 (KS 100, BK 0) at 110 degrees, and it faces away from the tumbler now.
    let kb = sim_core::combat::fixed_knockback(80, sim.content.fighters[0].weight, 0, 100);
    assert_eq!(sim.fighter(1).launch_kb, kb);
    assert_eq!(sim.fighter(1).facing, 1, "turned round");
    let mv = embers(&sim).get(MoveId::SideSpecial as u8);
    assert!(mv
        .hitboxes
        .iter()
        .all(|h| h.angle == 110 && h.reverse && h.shield_damage == 157));
}

/// A shot of player 1's neutral special (whatever its kind) placed at `pos` moving at `vel`, as if just fired.
fn shot_from_player_1(sim: &mut Sim, pos: Vec2, vel: Vec2) {
    sim.state.projectiles[0] = sim_core::Projectile {
        active: true,
        owner: 1,
        origin: 1,
        power: 100,
        move_id: MoveId::NSpecial as u8,
        pos,
        vel,
        age: 0,
        life: 40,
        bounces: 0,
        kb_add: 0,
        size: 100,
        vars: [0; sim_core::PROJECTILE_VARS],
    };
}

#[test]
fn the_cape_reflects_a_shot_back_for_one_and_a_half_times_its_damage_and_three_quarters_of_its_life(
) {
    // A tumbler fireball (player 1, far right) coming at the tumbler (player 0), which capes it on frames 9-20.
    let mut sim = Sim::with_chars([TUMBLER, TUMBLER, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(9), -1);
    sim.tick(inp(127, 0, sim_core::input::buttons::SPECIAL));
    sim.ticks(8, inp(0, 0, 0));
    // Frame 10: a shot level with the field, coming left, falling a little.
    let units = |n: i32| Fx::from_ratio(n, 8000);
    shot_from_player_1(
        &mut sim,
        Vec2::new(fx(22, 10), fx(8, 10)),
        Vec2::new(-units(1400), -units(200)),
    );
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, 0));
    let p = sim.state.projectiles[0];
    assert!(p.active);
    assert_eq!(p.owner, 0, "it is the tumbler's now");
    assert_eq!(p.power, 150);
    assert!(p.vel.x > Fx::ZERO, "going back: {:?}", p.vel);
    assert_eq!(
        p.vel.y,
        units(200),
        "its vertical speed turned round too [E]"
    );
    assert_eq!(p.life, 30, "0.75 of its 40 frames");
}

#[test]
fn a_shot_touching_the_capes_detector_brings_the_reflector_up_early() {
    let mut sim = Sim::with_chars([TUMBLER, TUMBLER, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(9), -1);
    sim.tick(inp(127, 0, sim_core::input::buttons::SPECIAL));
    sim.ticks(5, inp(0, 0, 0));
    assert_eq!(sim.f().state_frame, 5, "frame 6: the detector is out");
    // A slow shot just inside the detector, outside the reflector.
    shot_from_player_1(
        &mut sim,
        Vec2::new(fx(19, 10), fx(8, 10)),
        Vec2::new(-fx(1, 100), Fx::ZERO),
    );
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().state_frame,
        8,
        "skipped to frame 9 (the reflector's first)"
    );
}

#[test]
fn the_capes_lift_in_the_air_happens_once_until_the_tumbler_lands() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), fx(1, 10), -fx(1, 10));
    sim.tick(inp(127, 0, sim_core::input::buttons::SPECIAL));
    // Started in the air: 1 / 2.5 of the horizontal speed kept.
    assert_eq!(sim.f().vel.x, fx(1, 10) * Fx::from_ratio(40, 100));
    sim.tick(inp(0, 0, 0));
    let units = |n: i32| Fx::from_ratio(n, 8000);
    // The fall stopped and 0.3 up, less one frame of the cape's gravity (0.031).
    assert!((sim.f().vel.y - (units(300) - units(31))).abs() <= Fx::from_raw(2));
    sim.ticks(40, inp(0, 0, 0));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, -fx(1, 10));
    sim.tick(inp(127, 0, sim_core::input::buttons::SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert!(
        sim.f().vel.y < Fx::ZERO,
        "no second lift: {:?}",
        sim.f().vel.y
    );
    // It falls at most 1 (reference units a frame) during the cape.
    sim.ticks(20, inp(0, 0, 0));
    assert!(sim.f().vel.y >= -units(1000));
}

/// The tick the tumbler, held in reach of the left ledge, catches it with its cape, facing `facing`.
fn cape_ledge_tick(facing: i8) -> Option<usize> {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(9), -1);
    let hold = |sim: &mut Sim| {
        sim.state.fighters[0].pos = Vec2::new(fx(-122, 10), fx(-18, 10));
        sim.state.fighters[0].vel = Vec2::ZERO;
        sim.state.fighters[0].platform = sim_core::state::NONE;
    };
    hold(&mut sim);
    sim.state.fighters[0].state = S::Airborne;
    sim.state.fighters[0].facing = facing;
    sim.tick(inp(facing * 127, 0, sim_core::input::buttons::SPECIAL));
    if sim.f().state == S::LedgeHang {
        return Some(1);
    }
    (2..=36).find(|_| {
        hold(&mut sim);
        sim.tick(inp(0, 0, 0));
        sim.f().state == S::LedgeHang
    })
}

#[test]
fn the_cape_catches_a_ledge_in_front_from_frame_2_and_behind_from_16() {
    assert_eq!(cape_ledge_tick(1), Some(2));
    assert_eq!(cape_ledge_tick(-1), Some(16));
}

// ---- Down special: the water pump (phase 2) -------------------------------------------------------------------------------------

/// The tumbler standing at `x` facing right, the duelist parked far off (or placed by the caller).
fn pump_sim(x: Fx) -> Sim {
    let mut sim = duel();
    sim.stand(0, x, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    sim
}

fn pumps(sim: &Sim) -> Vec<sim_core::Projectile> {
    sim.state
        .projectiles
        .iter()
        .copied()
        .filter(|p| p.active)
        .collect()
}

const SPECIAL_DOWN: (i8, i8) = (0, -127);

#[test]
fn the_pump_winds_up_for_19_frames_then_charges_for_80_and_holds_the_full_charge() {
    use sim_core::input::buttons::{SHIELD, SPECIAL};
    let mut sim = pump_sim(Fx::from_int(-3));
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.ticks(17, inp(0, 0, 0));
    assert_eq!(
        sim.f().move_id,
        MoveId::DownSpecial as u8,
        "frame 18: winding up"
    );
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().move_id,
        MoveId::Ext1 as u8,
        "charging after frame 19"
    );
    // A full charge does not keep itself (owner, 2026-10-13, as the reference): the stance holds at full, long past the move's
    // length, until shield or special.
    sim.ticks(80 + 400, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack, "still in the charging stance");
    assert_eq!(sim.f().move_id, MoveId::Ext1 as u8);
    assert_eq!(sim.f().stored, 0, "nothing stored while holding");
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().stored, 80, "shield keeps the full charge");
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield, "shielding at once");
    // The next use shoots at once after the wind-up.
    sim.ticks(20, inp(0, 0, 0));
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.ticks(18, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, MoveId::Ext2 as u8, "shooting");
    assert_eq!(sim.f().stored, 0, "spent");
}

#[test]
fn special_shoots_a_held_full_charge_at_full_strength() {
    use sim_core::input::buttons::SPECIAL;
    let mut sim = pump_sim(Fx::from_int(-3));
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.ticks(18 + 200, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, MoveId::Ext1 as u8, "holding at full");
    let x0 = sim.f().pos.x;
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::Ext2 as u8, "special shoots");
    sim.ticks(2, inp(0, 0, 0));
    // The full charge's recoil: 0.02 x 80 reference units a frame backwards (facing right).
    assert!(sim.f().pos.x < x0, "pushed back by a full charge");
}

#[test]
fn a_held_full_charge_in_the_air_keeps_falling() {
    use sim_core::input::buttons::SPECIAL;
    let mut sim = pump_sim(Fx::from_int(-3));
    sim.put_airborne(0, Fx::from_int(-3), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.ticks(18 + 100, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, MoveId::Ext1 as u8);
    let y0 = sim.f().pos.y;
    sim.ticks(20, inp(0, 0, 0));
    assert!(sim.f().pos.y < y0, "falling while it holds");
    assert_eq!(sim.f().state, S::Attack);
}

#[test]
fn shield_keeps_a_part_charge_with_no_lag_and_the_next_use_charges_on_from_it() {
    use sim_core::input::buttons::{SHIELD, SPECIAL};
    let mut sim = pump_sim(Fx::from_int(-3));
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.ticks(18 + 30, inp(0, 0, 0));
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().stored, 31);
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield, "shielding at once");
    sim.ticks(20, inp(0, 0, 0));
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.ticks(18 + 48, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack, "31 + 48 frames: still charging");
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().stored, 80, "31 + 49: full at 80");
}

/// Uses the pump with `charge` frames of charge stored, the duelist `gap` ahead (in the air if `air`); returns the sim after
/// `ticks` frames of the shot.
fn pump_shot(charge: u8, gap: Fx, air: bool, ticks: usize, stick: i8) -> Sim {
    use sim_core::input::buttons::SPECIAL;
    let mut sim = pump_sim(Fx::from_int(-6));
    sim.state.fighters[0].stored = charge;
    if air {
        sim.put_airborne(0, Fx::from_int(-6), Fx::from_int(15), Fx::ZERO, Fx::ZERO);
    }
    sim.state.fighters[1].invuln = 0;
    sim.put_airborne(
        1,
        Fx::from_int(-6) + gap,
        fx(8, 10) + if air { Fx::from_int(15) } else { Fx::ZERO },
        Fx::ZERO,
        Fx::ZERO,
    );
    // Special, then special again in the wind-up: the shot at once.
    sim.tick(inp(SPECIAL_DOWN.0, SPECIAL_DOWN.1, SPECIAL));
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(16, inp(0, stick, 0));
    assert_eq!(sim.f().move_id, MoveId::Ext2 as u8);
    sim.ticks(ticks, inp(0, stick, 0));
    sim
}

#[test]
fn the_shot_sends_seven_pumps_every_5_frames_and_ends_on_frame_49() {
    let mut sim = pump_shot(0, Fx::from_int(20), false, 0, 0);
    let mut spawned = Vec::new();
    let mut seen = 0usize;
    for t in 1..=60 {
        sim.tick(inp(0, 0, 0));
        let n = sim
            .state
            .projectiles
            .iter()
            .filter(|p| p.active && p.age == 0)
            .count();
        if n > 0 {
            spawned.push(t);
        }
        seen = seen.max(pumps(&sim).len());
        if sim.f().state != S::Attack {
            assert_eq!(t, 48, "actionable on frame 49 of the shot");
            break;
        }
    }
    assert_eq!(spawned, vec![1, 6, 11, 16, 21, 26, 31]);
    assert!(seen <= 3, "at most three out at once: {seen}");
}

#[test]
fn a_pump_pushes_without_damage_or_hitstun_and_harder_with_the_charge() {
    let shove = |charge: u8| {
        let sim = pump_shot(charge, fx(25, 10), true, 6, 0);
        let v = sim.fighter(1);
        assert_eq!(v.percent, Fx::ZERO, "no damage");
        assert_eq!(v.state, S::Airborne, "no hitstun");
        v.vel.x
    };
    let weak = shove(0);
    let strong = shove(80);
    assert!(weak > Fx::ZERO, "pushed away: {weak:?}");
    assert!(
        strong > weak,
        "a full charge pushes harder: {strong:?} vs {weak:?}"
    );
}

#[test]
fn a_charged_shot_pushes_the_tumbler_back_and_flies_faster_and_bigger() {
    let units = |n: i32| Fx::from_ratio(n, 8000);
    let sim = pump_shot(80, Fx::from_int(30), true, 1, 0);
    // 0.02 reference units a frame for each of the 80 charge frames: 1.6, backward.
    assert!(
        (sim.f().vel.x + units(1600)).abs() < units(40),
        "{:?}",
        sim.f().vel
    );
    let p = pumps(&sim)[0];
    assert_eq!(p.size, 120);
    assert_eq!(p.kb_add, 55);
    // 8.6 a frame at 12 degrees up (less a frame of gravity).
    assert!((p.vel.x - units(8412)).abs() < units(30), "{:?}", p.vel);
    let p0 = pumps(&pump_shot(0, Fx::from_int(30), true, 1, 0))[0];
    assert_eq!((p0.size, p0.kb_add), (80, 0));
    assert!((p0.vel.x - units(4206)).abs() < units(30), "{:?}", p0.vel);
}

#[test]
fn the_stick_turns_the_shot_up_to_62_degrees_and_down_to_minus_6() {
    let angle_of = |stick: i8| {
        let sim = pump_shot(0, Fx::from_int(30), true, 31, stick);
        // (The last pump, just out on frame 31: its rise against its run.)
        let p = pumps(&sim).into_iter().find(|p| p.age == 0).unwrap();
        (p.vel.y / p.vel.x).mul_int(100).floor_int()
    };
    // tan 62 = 1.88, tan -6 = -0.105.
    let up = angle_of(127);
    let down = angle_of(-127);
    assert!((185..=188).contains(&up), "{up}");
    assert!((-12..=-10).contains(&down), "{down}");
}
