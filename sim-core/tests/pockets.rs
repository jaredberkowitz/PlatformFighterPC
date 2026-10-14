//! The Pockets moveset on the settler body (class 10), modelled mechanically on a reference character (phase 1: the body and every
//! normal; the rocket and the balloons are in `pockets_specials.rs`, the jab's previous-version rules in `older_rules.rs`). Frame data
//! against the reference for every normal, checked in the data and by running the moves (its three projectile normals too); hit
//! properties (damage, angles and angle types, knockback, set knockback and set weight, hitlag and SDI multipliers, trip chances,
//! shield damage, grounded- and aerial-only boxes, no-clank boxes, the bury); the up tilt's head intangibility and hitstun modifier;
//! the turnips' random counts (E39); grabs, pummel and throws; landing lag and autocancels; the body's attributes and jump air times;
//! registration; determinism. Reference data and sources: the eleventh moveset's `REFERENCE_DATA.md` (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::intangible_parts;
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::pockets_slots::{DAIR_ONE, DAIR_TWO, UAIR_ONE, UAIR_TWO};
use sim_core::moves::{
    MoveId, Weapon, ANGLE_BY_POSITION, ANGLE_FORWARD, EFFECT_BURY, HIT_GRAB, HIT_THROW, PART_HIGH,
    PART_LIMB, RULES_PREVIOUS, TARGETS_AERIAL, TARGETS_ALL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const SETTLER: u8 = 10;

/// The settler (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SETTLER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SETTLER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R]; the jab is the previous reference version's; the down throw's release and FAF
/// fold its 6 frames of hitlag in [D]; the dash attack, forward smash and the forward and back airs fire projectiles and have no
/// boxes of their own).
const FRAME_DATA: [FrameData; 29] = [
    (MoveId::Jab, &[(3, 4)], 22, 0, 0, 255),
    (MoveId::Jab2, &[(3, 4)], 24, 0, 0, 255),
    (MoveId::FTilt, &[(8, 11)], 34, 0, 0, 255),
    (MoveId::UTilt, &[(7, 18), (22, 25)], 48, 0, 0, 255),
    (MoveId::DTilt, &[(9, 11)], 38, 0, 0, 255),
    (MoveId::DashAttack, &[], 45, 0, 0, 255),
    (MoveId::FSmash, &[], 50, 0, 0, 255),
    (
        MoveId::USmash,
        &[(12, 13), (17, 18), (21, 22), (25, 26), (29, 30), (33, 34)],
        54,
        0,
        0,
        255,
    ),
    (
        MoveId::DSmash,
        &[(9, 9), (10, 11), (29, 29), (30, 31)],
        50,
        0,
        0,
        255,
    ),
    (MoveId::NAir, &[(3, 10), (11, 23)], 36, 8, 2, 27),
    (MoveId::FAir, &[], 40, 14, 2, 30),
    (MoveId::BAir, &[], 36, 14, 2, 30),
    (MoveId::UAir, &[(6, 7), (8, 21)], 43, 12, 5, 39),
    (UAIR_TWO, &[(6, 7), (8, 21)], 43, 12, 5, 39),
    (UAIR_ONE, &[(6, 7), (8, 21)], 43, 12, 5, 39),
    (MoveId::DAir, &[(11, 12), (13, 26)], 43, 12, 10, 36),
    (DAIR_TWO, &[(11, 12), (13, 26)], 43, 12, 10, 36),
    (DAIR_ONE, &[(11, 12), (13, 26)], 43, 12, 10, 36),
    (MoveId::Grab, &[(14, 16)], 44, 0, 0, 255),
    (MoveId::DashGrab, &[(16, 18)], 48, 0, 0, 255),
    (MoveId::PivotGrab, &[(17, 19)], 45, 0, 0, 255),
    (MoveId::Pummel, &[(3, 3)], 7, 0, 0, 255),
    (MoveId::FThrow, &[(15, 15)], 46, 0, 0, 255),
    (MoveId::BThrow, &[(11, 11)], 46, 0, 0, 255),
    (MoveId::UThrow, &[(13, 13)], 46, 0, 0, 255),
    (MoveId::DThrow, &[(24, 24)], 42, 0, 0, 255),
    (MoveId::LedgeAttack, &[(23, 25)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(16, 17), (20, 21)], 46, 0, 0, 255),
    // (The neutral and down specials are still placeholders: phase 2.)
    (MoveId::NSpecial, &[], 255, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = kit(&sim);
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
    // Intangibility: the ledge attack through frame 25 (the first grab), the get-up attack through 21.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 25);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 21);
    // Smash charge frames (ssbwiki.com's "charges between" 16-17, 7-8, 3-4: move frames one less than the first); the forward smash's
    // own 1.2x at full charge.
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(15));
    assert_eq!(w.get(MoveId::FSmash as u8).charge_bonus, 20);
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(6));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(2));
    // The jab (1.1.x): jab 1 into jab 2 from frame 7, jab 2 into jab 1 again from frame 8; no third hit.
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 6));
    let jab2 = w.get(MoveId::Jab2 as u8);
    assert_eq!((jab2.next, jab2.next_from), (Some(MoveId::Jab as u8), 7));
    // (Both cancellable from their continuable frames 7 and 8: owner decision 70.)
    assert_eq!((jab1.cancel_from, jab2.cancel_from), (6, 7));
    assert!(w.get(MoveId::Jab3 as u8).is_empty());
}

#[test]
fn the_up_tilt_has_the_head_intangible_on_7_to_25_and_nothing_else_is() {
    let sim = duel();
    let w = kit(&sim);
    let mut f = *sim.f();
    f.state = S::Attack;
    f.move_id = MoveId::UTilt as u8;
    for (frame, bits) in [
        (5u16, 0u8),
        (6, PART_HIGH | PART_LIMB),
        (24, PART_HIGH | PART_LIMB),
        (25, 0),
    ] {
        f.state_frame = frame;
        assert_eq!(intangible_parts(&f, w), bits, "move frame {frame}");
    }
    assert!(
        !w.limbs,
        "held tools: no limb circle (the arm's window acts on nothing)"
    );
    for (id, ..) in FRAME_DATA {
        if id != MoveId::UTilt {
            assert!(w.get(id as u8).part_intangible.is_empty(), "{id:?}");
        }
    }
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (high in the air for an aerial, else standing), with player 1 held where the move's box (a hitbox of
/// window `first` that can hit an airborne fighter) or its projectile is, and runs until player 1 is hit. Returns the reference
/// frame of the hit.
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
        .copied();
    let before = sim.fighter(1).percent;
    let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
    for tick in 1..=80u8 {
        let me = sim.state.fighters[0].pos;
        let shot = sim
            .state
            .projectiles
            .iter()
            .find(|p| p.active && p.owner == 0)
            .map(|p| p.pos);
        let at = match (shot, hb) {
            // (Where the shot will be after its next step: just ahead of where it is.)
            (Some(p), _) => p,
            (None, Some(h)) => Vec2::new(me.x + h.x, me.y + h.y),
            // (Before its shot appears: on the stage well ahead, out of its way.)
            (None, None) => Vec2::new(me.x + Fx::from_int(5), me.y),
        };
        sim.put_airborne(1, at.x, (at.y - middle).max(fx(1, 10)), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            return Some(tick + 1);
        }
        if sim.f().state != S::Attack && shot.is_none() && tick > 3 {
            return None;
        }
    }
    None
}

#[test]
fn every_attack_first_hits_on_its_reference_frame() {
    for (id, first) in [
        (MoveId::Jab, 3u8),
        (MoveId::Jab2, 3),
        (MoveId::FTilt, 8),
        (MoveId::UTilt, 7),
        (MoveId::DTilt, 9),
        (MoveId::USmash, 17),
        (MoveId::DSmash, 9),
        (MoveId::NAir, 3),
        (MoveId::UAir, 6),
        (MoveId::DAir, 11),
    ] {
        assert_eq!(
            first_hit_frame(id, first),
            Some(first),
            "{id:?} window from {first}"
        );
    }
}

#[test]
fn the_projectile_normals_first_hit_on_their_reference_frames() {
    for (id, first) in [
        (MoveId::DashAttack, 9u8),
        (MoveId::FSmash, 25),
        (MoveId::FAir, 10),
        (MoveId::BAir, 13),
    ] {
        assert_eq!(first_hit_frame(id, first), Some(first), "{id:?}");
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
        let aerial = id.is_aerial() || matches!(id, UAIR_TWO | UAIR_ONE | DAIR_TWO | DAIR_ONE);
        assert_eq!(faf_of(id, aerial), u16::from(faf), "{id:?}");
    }
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority (the reference's id), damage in tenths, angle, BK, KS) for the boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 44] = [
        (MoveId::Jab, 3, 0, 30, 60, 10, 80),
        // (The far box: 13.0.1's pull, owner decision 70: 180 degrees, BK 15, KS 10.)
        (MoveId::Jab, 3, 1, 30, 180, 15, 10),
        (MoveId::Jab2, 3, 0, 30, 60, 20, 120),
        (MoveId::Jab2, 3, 1, 30, 80, 20, 120),
        (MoveId::FTilt, 8, 0, 90, 45, 80, 50),
        (MoveId::FTilt, 8, 3, 90, 45, 80, 50),
        (MoveId::UTilt, 7, 0, 50, 108, 0, 100),
        (MoveId::UTilt, 7, 2, 50, 135, 35, 10),
        (MoveId::UTilt, 22, 0, 60, 80, 50, 140),
        (MoveId::UTilt, 22, 1, 60, 80, 50, 140),
        (MoveId::DTilt, 9, 0, 120, 75, 50, 95),
        (MoveId::DTilt, 9, 1, 100, 70, 50, 95),
        (MoveId::USmash, 12, 0, 30, 87, 0, 100),
        (MoveId::USmash, 12, 1, 30, 115, 0, 100),
        (MoveId::USmash, 17, 0, 15, 110, 55, 0),
        (MoveId::USmash, 17, 1, 15, 185, 40, 0),
        (MoveId::USmash, 29, 1, 15, 185, 40, 0),
        (MoveId::USmash, 33, 0, 46, 90, 30, 220),
        (MoveId::DSmash, 9, 0, 60, 60, 30, 60),
        (MoveId::DSmash, 9, 1, 60, 60, 50, 70),
        (MoveId::DSmash, 10, 0, 30, 60, 30, 50),
        (MoveId::DSmash, 10, 1, 30, 60, 50, 60),
        (MoveId::DSmash, 29, 0, 60, 60, 30, 60),
        (MoveId::DSmash, 30, 1, 30, 60, 50, 60),
        (MoveId::NAir, 3, 0, 90, 361, 20, 100),
        (MoveId::NAir, 3, 2, 90, 361, 20, 100),
        (MoveId::NAir, 11, 1, 50, 361, 20, 100),
        (MoveId::UAir, 6, 0, 130, 85, 40, 100),
        (MoveId::UAir, 8, 0, 60, 85, 40, 100),
        (UAIR_TWO, 6, 0, 100, 85, 40, 100),
        (UAIR_TWO, 8, 0, 50, 85, 40, 100),
        (UAIR_ONE, 6, 0, 80, 85, 40, 100),
        (UAIR_ONE, 8, 0, 40, 85, 40, 100),
        (MoveId::DAir, 11, 0, 130, 270, 20, 80),
        (MoveId::DAir, 11, 1, 130, 45, 30, 100),
        (MoveId::DAir, 13, 0, 60, 45, 30, 100),
        (DAIR_TWO, 11, 0, 100, 270, 10, 80),
        (DAIR_TWO, 13, 0, 50, 45, 30, 100),
        (DAIR_ONE, 11, 0, 80, 270, 0, 80),
        (DAIR_ONE, 11, 1, 80, 45, 30, 100),
        (DAIR_ONE, 13, 0, 40, 45, 30, 100),
        (MoveId::LedgeAttack, 23, 0, 90, 45, 90, 20),
        (MoveId::GetUpAttack, 16, 0, 70, 48, 80, 48),
        (MoveId::GetUpAttack, 20, 0, 70, 48, 80, 48),
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
    // Only the jab is on the previous version's rules (owner decisions 66-67).
    for (id, ..) in FRAME_DATA {
        let older = matches!(id, MoveId::Jab | MoveId::Jab2);
        assert!(all(id, &|h| (h.rules == RULES_PREVIOUS) == older), "{id:?}");
    }
    // Hitlag and SDI multipliers.
    assert!(all(MoveId::UTilt, &|h| h.hitlag
        == if h.group == 0 { 80 } else { 120 }));
    assert!(all(MoveId::USmash, &|h| (h.hitlag, h.sdi)
        == match h.group {
            0 => (50, 10),
            5 => (150, 100),
            _ => (100, 0),
        }));
    assert!(all(MoveId::DSmash, &|h| h.sdi == 0
        && h.hitlag
            == if h.damage == Fx::from_int(6) { 100 } else { 0 }));
    assert!(all(MoveId::Pummel, &|h| h.hitlag == 210));
    for id in [
        MoveId::FTilt,
        MoveId::DTilt,
        MoveId::NAir,
        MoveId::UAir,
        MoveId::DAir,
    ] {
        assert!(all(id, &|h| h.hitlag == 100 && h.sdi == 100), "{id:?}");
    }
    // Set knockback and set weight: the up tilt's grounded boxes 75, the up smash's launcher 120 and its loop 30 (set weight).
    assert!(all(MoveId::UTilt, &|h| h.fixed_kb
        == if h.group == 0 && h.targets == TARGETS_GROUNDED {
            75
        } else {
            0
        }));
    assert!(all(MoveId::UTilt, &|h| h.set_weight == (h.group == 0)));
    assert!(all(MoveId::USmash, &|h| (h.fixed_kb, h.set_weight)
        == match h.group {
            0 => (120, false),
            5 => (0, false),
            _ => (30, true),
        }));
    // The up tilt's hitstun modifier (+2 on every box of hit 1, the reference's glitch) and nothing else's.
    for (id, ..) in FRAME_DATA {
        assert!(
            all(id, &|h| h.hitstun_add
                == if id == MoveId::UTilt && h.group == 0 {
                    2
                } else {
                    0
                }),
            "{id:?}"
        );
    }
    // Targets: the up tilt's (grounded 0-1, aerial 2), the down tilt's far box, the up smash's launcher, the down smash's bury boxes.
    assert!(all(MoveId::UTilt, &|h| h.targets
        == match (h.group, h.priority) {
            (0, 2) => TARGETS_AERIAL,
            (0, _) => TARGETS_GROUNDED,
            _ => TARGETS_ALL,
        }));
    assert!(all(MoveId::DTilt, &|h| (h.targets == TARGETS_GROUNDED)
        == (h.priority == 1)));
    assert!(all(MoveId::USmash, &|h| (h.targets == TARGETS_GROUNDED)
        == (h.group == 0)));
    assert!(all(MoveId::DSmash, &|h| (h.targets == TARGETS_GROUNDED)
        == (h.priority == 0)
        && (h.effect == EFFECT_BURY) == (h.priority == 0)
        && !h.rebounds));
    // Trips: only the down tilt, 30% on both boxes.
    for (id, ..) in FRAME_DATA {
        assert!(
            all(id, &|h| h.trip == if id == MoveId::DTilt { 30 } else { 0 }),
            "{id:?}"
        );
    }
    // Angle types: type 3 (forward) on the forward tilt, the up tilt's second hit, the up smash's launcher, the down smash's first
    // hit, the pummel and the ledge attack; type 0 (by position) on the rest of the boxes.
    assert!(all(MoveId::FTilt, &|h| h.angle_type == ANGLE_FORWARD));
    assert!(all(MoveId::UTilt, &|h| h.angle_type
        == if h.group == 1 {
            ANGLE_FORWARD
        } else {
            ANGLE_BY_POSITION
        }));
    assert!(all(MoveId::DSmash, &|h| h.angle_type
        == if h.group == 0 {
            ANGLE_FORWARD
        } else {
            ANGLE_BY_POSITION
        }));
    for id in [
        MoveId::DTilt,
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
    // No clank on the pummel, the ledge attack and the get-up attack (`clang=f rebound=f`); no rebound on the down smash (`rebound=f`).
    for (id, ..) in FRAME_DATA {
        let never = matches!(
            id,
            MoveId::Pummel | MoveId::LedgeAttack | MoveId::GetUpAttack
        );
        assert!(all(id, &|h| h.clanks == !never), "{id:?} clanks");
        assert!(
            all(id, &|h| h.rebounds == !(never || id == MoveId::DSmash)),
            "{id:?} rebounds"
        );
    }
}

#[test]
fn the_projectile_normals_carry_the_references_hits() {
    let sim = duel();
    let w = kit(&sim);
    let shot = |id: MoveId| w.get(id as u8).projectile.expect("a projectile");
    // The flowerpot: 10% clean (radius 4.2), 6% late (3.7), 70 degrees type 3, BK 65, KS 60, hitlag x0.6, SDI x0.
    let pot = shot(MoveId::DashAttack);
    let late = pot.phases[0].expect("the late hit").1;
    assert_eq!(
        (
            pot.hitbox.damage,
            pot.hitbox.angle,
            pot.hitbox.base_knockback,
            pot.hitbox.knockback_growth
        ),
        (Fx::from_int(10), 70, 65, 60)
    );
    assert_eq!(
        (pot.hitbox.hitlag, pot.hitbox.sdi, pot.hitbox.angle_type),
        (60, 0, ANGLE_FORWARD)
    );
    assert_eq!(
        (late.damage, late.hitlag, late.sdi),
        (Fx::from_int(6), 60, 0)
    );
    // The bowling ball: 15% early, 17% later, 45 degrees type 3, BK 70, KS 95, SDI x0; 1.2x at a full charge.
    let ball = shot(MoveId::FSmash);
    let later = ball.phases[0].expect("the later hit").1;
    assert_eq!(
        (
            ball.hitbox.damage,
            later.damage,
            ball.hitbox.angle,
            ball.hitbox.base_knockback,
            ball.hitbox.knockback_growth
        ),
        (Fx::from_int(15), Fx::from_int(17), 45, 70, 95)
    );
    assert_eq!(ball.charged.map(|c| (c.full, c.power_to)), Some((60, 120)));
    // The pellets: 7 / 4 / 2.5% forward and 9 / 5 / 3% back, 361 type 3, BK 55 then 10, KS 100, half shield damage (60% on the
    // forward pellet's last hit).
    for (id, d) in [(MoveId::FAir, [70, 40, 25]), (MoveId::BAir, [90, 50, 30])] {
        let p = shot(id);
        let hits = [p.hitbox, p.phases[0].unwrap().1, p.phases[1].unwrap().1];
        for (h, (tenths, bkb)) in hits.iter().zip([(d[0], 55), (d[1], 10), (d[2], 10)]) {
            assert_eq!(
                (
                    h.damage,
                    h.angle,
                    h.base_knockback,
                    h.knockback_growth,
                    h.angle_type
                ),
                (Fx::from_ratio(tenths, 10), 361, bkb, 100, ANGLE_FORWARD),
                "{id:?}"
            );
        }
        assert_eq!(hits[0].shield_damage, 50);
        assert_eq!(hits[1].shield_damage, 50);
        assert_eq!(
            hits[2].shield_damage,
            if id == MoveId::FAir { 60 } else { 50 }
        );
        assert_eq!(
            p.life, 15,
            "{id:?}: its ages 1-14 (the reference's 14 frames)"
        );
    }
    // The back air's pellet flies backward.
    assert!(shot(MoveId::BAir).speed < Fx::ZERO);
}

#[test]
fn the_flowerpot_breaks_on_the_floor_after_frame_22_and_falls_on_off_a_ledge() {
    // On the stage: the pot's last hitting frame is 22 (it is gone by 24). Off the right edge: it falls on, still there on frame 60.
    let pot_frames = |x: Fx| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(-20), 1);
        sim.stand(0, x, 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DashAttack);
        let mut alive = Vec::new();
        for tick in 1..=80u16 {
            sim.tick(inp(0, 0, 0));
            if sim
                .state
                .projectiles
                .iter()
                .any(|p| p.active && p.owner == 0 && p.rest == 0)
            {
                alive.push(tick + 1);
            }
        }
        alive
    };
    let on = pot_frames(Fx::ZERO);
    assert_eq!(on.first(), Some(&8), "it appears on frame 8 (hits from 9)");
    assert_eq!(on.last(), Some(&22), "it touches the floor on frame 23");
    let edge = sim_core::Content::placeholder().stage.platforms[0].right;
    let off = pot_frames(edge - fx(3, 10));
    assert_eq!(
        off.last(),
        Some(&68),
        "off the ledge it falls on until frame 68"
    );
}

#[test]
fn the_bowling_ball_stops_after_frame_30_on_the_stage_and_hits_harder_charged() {
    let ball_frames = |x: Fx| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(-20), 1);
        sim.stand(0, x, 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
        let mut alive = Vec::new();
        for tick in 1..=160u16 {
            sim.tick(inp(0, 0, 0));
            if sim
                .state
                .projectiles
                .iter()
                .any(|p| p.active && p.owner == 0 && p.rest == 0)
            {
                alive.push(tick + 1);
            }
        }
        alive
    };
    let on = ball_frames(Fx::ZERO);
    assert_eq!((on.first(), on.last()), (Some(&24), Some(&30)));
    // Off the ledge it falls on, hitting until frame 139 (its life) unless it leaves through the blast zone first.
    let edge = sim_core::Content::placeholder().stage.platforms[0].right;
    let off = ball_frames(edge - fx(5, 10));
    assert!(off.len() > on.len() + 30, "{off:?}");
    assert!(off.last().is_some_and(|&f| f <= 139));
    assert_eq!(
        kit(&duel())
            .get(MoveId::FSmash as u8)
            .projectile
            .map(|p| p.life),
        Some(116)
    );
    // Charged to the full 60 frames: 15% x 1.2 = 18%.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
    let before = sim.fighter(1).percent;
    for _ in 0..140 {
        sim.tick(inp(0, 0, ATTACK));
    }
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 180));
}

// ---- The turnips (E39) ---------------------------------------------------------------------------------------------------------

#[test]
fn the_turnips_are_drawn_three_two_or_one_forty_thirty_thirty() {
    // Over many up airs and down airs started with different random states, each version comes out about as often as the reference
    // says (40 / 30 / 30 percent), and only on the move's first frame.
    let mut counts = [[0u32; 3]; 2];
    for seed in 0..600u64 {
        for (k, (id, two, one)) in [
            (MoveId::UAir, UAIR_TWO, UAIR_ONE),
            (MoveId::DAir, DAIR_TWO, DAIR_ONE),
        ]
        .into_iter()
        .enumerate()
        {
            let mut sim = duel();
            sim.state.rng = Rng::new(seed);
            sim.stand(1, Fx::from_int(10), -1);
            sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
            // (Started by the input, so the draw happens on the step that starts it.)
            sim.tick(inp(0, if id == MoveId::UAir { 127 } else { -127 }, ATTACK));
            assert_eq!(sim.f().state, S::Attack);
            let got = MoveId::from_index(sim.f().move_id);
            let v = if got == id {
                0
            } else if got == two {
                1
            } else {
                assert_eq!(got, one);
                2
            };
            counts[k][v] += 1;
            // It stays that version.
            sim.ticks(10, inp(0, 0, 0));
            assert_eq!(sim.f().move_id, got as u8);
        }
    }
    for c in counts {
        assert!((200..=280).contains(&c[0]), "three turnips {c:?}");
        assert!((140..=220).contains(&c[1]), "two {c:?}");
        assert!((140..=220).contains(&c[2]), "one {c:?}");
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
fn the_grab_catches_on_frame_14() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    assert_eq!(grab_now(&mut sim), 14);
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
    // (stick, the frame the held fighter's percent rises on, damage in tenths). The down throw's 6 frames of hitlag are folded into its
    // timeline (release 24 [D]).
    for (stick, frame, damage) in [
        ((127i8, 0i8), 15u8, 90),
        ((-127, 0), 11, 110),
        ((0, 127), 13, 100),
        ((0, -127), 24, 60),
    ] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(15, 10), -1);
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
            assert!(sim.fighter(1).pos.x < Fx::ZERO, "thrown behind");
            assert_eq!(sim.f().facing, -1, "facing the throw");
        }
    }
    let sim = duel();
    let w = kit(&sim);
    for (id, lag) in [
        (MoveId::FThrow, 0),
        (MoveId::BThrow, 100),
        (MoveId::UThrow, 0),
        (MoveId::DThrow, 0),
    ] {
        assert!(w
            .get(id as u8)
            .hitboxes
            .iter()
            .all(|h| h.kind == HIT_THROW && h.hitlag == lag));
    }
}

#[test]
fn the_pummel_does_1_3_percent() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Pummel as u8);
    sim.ticks(4, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 13));
}

// ---- The down smash's bury ---------------------------------------------------------------------------------------------------

#[test]
fn the_down_smash_buries_a_standing_fighter_and_launches_an_airborne_one() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(16, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DSmash);
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).state, S::Buried);
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 60));
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
    let plain = duel().content.fighters[usize::from(SETTLER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window; the tick adds
    // one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 15u16, 8u8, 26u16, 0u16),
        (MoveId::FAir, 20, 14, 29, 0),
        (MoveId::BAir, 20, 14, 29, 0),
        (MoveId::UAir, 15, 12, 38, 3),
        (UAIR_ONE, 15, 12, 38, 3),
        (MoveId::DAir, 20, 12, 35, 8),
        (DAIR_TWO, 20, 12, 35, 8),
    ] {
        assert_eq!(
            land_during(id, mid),
            (S::Landing, lag, id as u8),
            "{id:?} mid-move"
        );
        assert_eq!(land_during(id, late).1, plain, "{id:?} late");
        assert_eq!(land_during(id, early).1, plain, "{id:?} early");
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
fn the_settlers_jumps_have_the_references_air_times() {
    // Short hop 40, full hop 56, short hop fast fall 28, full hop fast fall 39 (ultimateframedata.com); within one frame.
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
fn the_settler_is_the_references_floaty_middleweight() {
    let p = duel().content.fighters[usize::from(SETTLER)];
    assert_eq!(p, FighterParams::settler());
    let su = FighterParams::su;
    assert_eq!(p.weight, Fx::from_int(92));
    assert_eq!(p.gravity, su(78));
    assert_eq!((p.max_fall_speed, p.fast_fall_speed), (su(1320), su(2112)));
    assert_eq!(
        (p.air_speed, p.air_accel, p.air_accel_stick, p.air_friction),
        (su(987), su(10), su(50), su(10))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (58, 81, 97, 129)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (30, 35, 26)
    );
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (1, su(1300), su(2300))
    );
    let t = p.item_throws;
    assert_eq!(
        (t.forward, t.back, t.up, t.down, t.dash, t.drop),
        (11, 11, 10, 10, 5, 6)
    );
    assert_eq!(p.air_jumps, 1);
}

#[test]
fn the_pockets_is_registered_as_class_10() {
    let c = Content::placeholder();
    assert_eq!(c.names.fighters[10], "settler");
    assert_eq!(c.names.weapons[10], "pockets");
    assert_eq!(c.fighters[10], FighterParams::settler());
    assert_eq!(c.weapons[10], sim_core::moves::pockets());
    // Every special is the Pockets' own (the side and up specials since phase 1, the neutral and down since phase 2, sim 115).
    for (id, own) in [
        (MoveId::NSpecial, true),
        (MoveId::SideSpecial, true),
        (MoveId::UpSpecial, true),
        (MoveId::DownSpecial, true),
    ] {
        assert_eq!(
            c.weapons[10].moves[id as usize] != c.weapons[1].moves[id as usize],
            own,
            "{id:?}"
        );
    }
    assert_eq!(sim_content_classes(), 11);
}

/// The number of classes a recipe can pick (the Pockets is the eleventh).
fn sim_content_classes() -> usize {
    Content::placeholder().weapons.len()
}

#[test]
fn random_play_with_the_settler_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [SETTLER, 0, SETTLER, 9];
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
