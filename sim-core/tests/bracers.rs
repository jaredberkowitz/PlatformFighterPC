//! The Bracers moveset on the hauler body (class 7), modelled mechanically on a reference character (phase 1: the body and every
//! normal; the specials are still the claws', the carry comes in phase 3). Frame data against the reference for every normal,
//! checked in the data and by running the moves; hit properties (damage, angles and angle types, knockback, set knockback, hitlag,
//! hitstun modifiers, trip chances, shield damage, no-clank and no-rebound boxes); the jab's continuable window; the aimed forward
//! tilt; the meteors; the arm and head intangibility; grabs, pummel and throws; landing lag and autocancels; the body's attributes
//! and jump air times; registration; determinism. Reference data and sources: the eighth moveset's `REFERENCE_DATA.md`
//! (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::intangible_parts;
use sim_core::input::buttons::{ATTACK, GRAB, JUMP};
use sim_core::moves::{
    MoveId, Weapon, AIM_DOWN, AIM_STRAIGHT, AIM_UP, ANGLE_BACKWARD, ANGLE_BY_POSITION,
    ANGLE_FORWARD, HIT_GRAB, HIT_THROW, PART_HIGH, PART_LIMB, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const HAULER: u8 = 7;

/// The hauler (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([HAULER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn arms(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(HAULER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R]; the forward throw is a placeholder until the carry, [E]; the up tilt's one window
/// 5-11 is split into the parts of its sweep, [E] positions).
const FRAME_DATA: [FrameData; 25] = [
    (MoveId::Jab, &[(5, 6)], 25, 0, 0, 255),
    (MoveId::Jab2, &[(4, 6)], 32, 0, 0, 255),
    (MoveId::FTilt, &[(7, 9)], 35, 0, 0, 255),
    (MoveId::UTilt, &[(5, 7), (8, 9), (10, 11)], 39, 0, 0, 255),
    (MoveId::DTilt, &[(6, 7)], 25, 0, 0, 255),
    (MoveId::DashAttack, &[(9, 12), (13, 24)], 35, 0, 0, 255),
    (MoveId::FSmash, &[(22, 23)], 55, 0, 0, 255),
    (MoveId::USmash, &[(14, 15)], 50, 0, 0, 255),
    (MoveId::DSmash, &[(11, 12), (13, 14)], 56, 0, 0, 255),
    (MoveId::NAir, &[(10, 10), (11, 13), (14, 26)], 39, 10, 9, 27),
    (MoveId::FAir, &[(18, 20), (21, 23)], 56, 17, 0, 56),
    (MoveId::BAir, &[(7, 8), (9, 16)], 32, 11, 6, 31),
    (MoveId::UAir, &[(6, 10)], 38, 15, 5, 26),
    (MoveId::DAir, &[(14, 16)], 55, 14, 2, 50),
    (MoveId::Grab, &[(8, 10)], 39, 0, 0, 255),
    (MoveId::DashGrab, &[(11, 13)], 47, 0, 0, 255),
    (MoveId::PivotGrab, &[(12, 14)], 42, 0, 0, 255),
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(15, 15)], 46, 0, 0, 255),
    (MoveId::BThrow, &[(15, 15)], 40, 0, 0, 255),
    (MoveId::UThrow, &[(14, 14)], 44, 0, 0, 255),
    (MoveId::DThrow, &[(19, 19)], 55, 0, 0, 255),
    (MoveId::LedgeAttack, &[(23, 25)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(18, 19), (23, 24)], 46, 0, 0, 255),
    // (The specials are still placeholders: phase 2.)
    (MoveId::NSpecial, &[], 255, 0, 0, 255),
];

#[test]
fn every_normal_has_the_references_frame_data() {
    let sim = duel();
    let w = arms(&sim);
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
        // (The forward tilt is as long as its angled versions, FAF 36; its script ends the straight one on 35: see below.)
        let data_faf = if id == MoveId::FTilt { 36 } else { faf };
        assert_eq!(
            u16::from(mv.total_frames) + 2,
            u16::from(data_faf),
            "{id:?} FAF"
        );
        assert_eq!(mv.landing_lag, landing, "{id:?} landing lag");
        assert_eq!(mv.autocancel_before, ac_before, "{id:?} autocancel before");
        let from = if ac_from == 255 { 255 } else { ac_from - 1 };
        assert_eq!(mv.autocancel_after, from, "{id:?} autocancel from");
        assert_eq!(mv.rehit, None, "{id:?}: no normal is a multi-hit");
    }
    // Intangibility: the ledge attack through frame 25 (the first grab), the get-up attack through 24.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 25);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 24);
    // Smash charge frames (ssbwiki.com's "charges between" 14-15, 6-7, 2-3: move frames one less than the first).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(13));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(5));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(1));
    // The jab's continuable window from frame 8 (move frame 7) into jab 2, which ends the chain.
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 7));
    assert_eq!(w.get(MoveId::Jab2 as u8).next, None);
}

#[test]
fn the_arm_and_head_intangibility_windows_are_the_references() {
    // (move, reference window, circles): the forward tilt's lower arm 6-9, the up tilt's 5-11, the down tilt's whole arm 5-7; the
    // forward smash's head and lower arms 20-26, the up smash's 12-15, the down smash's lower arms 11-14; the up air's head 5-10 (the
    // high circle, and the limb circle that reaches toward the head's box).
    let sim = duel();
    let w = arms(&sim);
    for (id, first, last, parts) in [
        (MoveId::FTilt, 6u16, 9u16, PART_LIMB),
        (MoveId::UTilt, 5, 11, PART_LIMB),
        (MoveId::DTilt, 5, 7, PART_LIMB),
        (MoveId::FSmash, 20, 26, PART_HIGH | PART_LIMB),
        (MoveId::USmash, 12, 15, PART_HIGH | PART_LIMB),
        (MoveId::DSmash, 11, 14, PART_LIMB),
        (MoveId::UAir, 5, 10, PART_HIGH | PART_LIMB),
    ] {
        let mut f = *sim.f();
        f.state = S::Attack;
        f.move_id = id as u8;
        for (frame, bits) in [
            (first - 2, 0u8),
            (first - 1, parts),
            (last - 1, parts),
            (last, 0),
        ] {
            f.state_frame = frame;
            assert_eq!(intangible_parts(&f, w), bits, "{id:?} move frame {frame}");
        }
    }
    assert!(w.limbs, "bare fists: the limb circle is a real arm");
}

// ---- Running the moves -------------------------------------------------------------------------------------------------------

/// Starts `id` for player 0 (in the air high up for an aerial, else standing), with player 1 held where the move's first hitbox of
/// window `first` (one that can hit an airborne fighter, not aimed up or down) will be, and runs until player 1 is hit. Returns the
/// reference frame of the hit.
fn first_hit_frame(id: MoveId, first: u8) -> Option<u8> {
    let mut sim = duel();
    if id.is_aerial() {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    let hb = *arms(&sim)
        .get(id as u8)
        .hitboxes
        .iter()
        .find(|h| {
            h.start + 1 == first
                && h.targets != TARGETS_GROUNDED
                && !matches!(h.aim, AIM_UP | AIM_DOWN)
        })
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
        (MoveId::Jab, 5u8),
        (MoveId::Jab2, 4),
        (MoveId::FTilt, 7),
        (MoveId::UTilt, 5),
        (MoveId::DTilt, 6),
        (MoveId::DashAttack, 9),
        (MoveId::FSmash, 22),
        (MoveId::USmash, 14),
        (MoveId::DSmash, 11),
        (MoveId::NAir, 10),
        (MoveId::FAir, 18),
        (MoveId::BAir, 7),
        (MoveId::UAir, 6),
        (MoveId::DAir, 14),
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

#[test]
fn the_angled_forward_tilts_end_a_frame_later_than_the_straight_one() {
    // The reference's FAF 36 aimed up or down, 35 straight (ssbwiki.com: "an oversight"), through the `aimed` register.
    for (aim, faf) in [(1i8, 36u16), (0, 35), (-1, 36)] {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(10), -1);
        sim.stand(0, Fx::from_int(-9), 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FTilt);
        sim.state.fighters[0].aim = aim;
        let mut out = 0;
        for tick in 1..=60u16 {
            sim.tick(inp(0, 0, 0));
            if sim.f().state != S::Attack {
                out = tick + 2;
                break;
            }
        }
        assert_eq!(out, faf, "aimed {aim}");
    }
    // Started by the stick, the aim is read as it starts: tilted down past the aim threshold, the forward tilt is the down one.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::from_int(-9), 1);
    sim.tick(inp(60, -50, ATTACK));
    assert_eq!(
        (MoveId::from_index(sim.f().move_id), sim.f().aim),
        (MoveId::FTilt, -1)
    );
}

// ---- Hit properties ----------------------------------------------------------------------------------------------------------

#[test]
fn every_hit_has_the_references_damage_angle_and_knockback() {
    // (move, reference first frame, priority (the reference's id), damage in tenths, angle, BK, KS) for the boxes of each phase.
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 37] = [
        (MoveId::Jab, 5, 0, 40, 361, 25, 35),
        (MoveId::Jab, 5, 1, 40, 361, 20, 30),
        (MoveId::Jab, 5, 2, 40, 180, 20, 20),
        (MoveId::Jab2, 4, 0, 60, 70, 40, 100),
        (MoveId::Jab2, 4, 2, 60, 70, 40, 100),
        (MoveId::UTilt, 5, 0, 80, 100, 40, 105),
        (MoveId::UTilt, 5, 1, 90, 100, 40, 110),
        (MoveId::UTilt, 8, 2, 100, 100, 40, 115),
        (MoveId::DTilt, 6, 0, 60, 361, 10, 80),
        (MoveId::DTilt, 6, 2, 60, 361, 10, 80),
        (MoveId::DashAttack, 9, 0, 120, 53, 85, 46),
        (MoveId::DashAttack, 13, 0, 90, 45, 60, 60),
        (MoveId::FSmash, 22, 0, 220, 46, 21, 88),
        (MoveId::FSmash, 22, 1, 210, 361, 18, 86),
        (MoveId::USmash, 14, 0, 190, 90, 40, 84),
        (MoveId::USmash, 14, 1, 190, 90, 40, 84),
        (MoveId::DSmash, 11, 0, 170, 70, 35, 88),
        (MoveId::DSmash, 11, 1, 170, 70, 35, 88),
        (MoveId::DSmash, 11, 2, 140, 82, 35, 88),
        (MoveId::DSmash, 11, 3, 140, 82, 35, 88),
        (MoveId::DSmash, 13, 0, 180, 55, 30, 88),
        (MoveId::DSmash, 13, 1, 180, 55, 30, 88),
        (MoveId::DSmash, 13, 3, 140, 98, 35, 88),
        (MoveId::NAir, 10, 3, 120, 361, 20, 100),
        (MoveId::NAir, 11, 2, 120, 361, 20, 100),
        (MoveId::NAir, 14, 0, 90, 361, 10, 100),
        (MoveId::FAir, 18, 0, 160, 361, 20, 85),
        (MoveId::FAir, 18, 1, 160, 361, 20, 85),
        (MoveId::FAir, 21, 0, 150, 270, 30, 100),
        (MoveId::FAir, 21, 1, 130, 361, 30, 100),
        (MoveId::BAir, 7, 0, 130, 361, 20, 97),
        (MoveId::BAir, 9, 2, 80, 361, 10, 100),
        (MoveId::UAir, 6, 0, 130, 90, 32, 90),
        (MoveId::DAir, 14, 0, 160, 270, 30, 90),
        (MoveId::DAir, 14, 1, 130, 361, 20, 90),
        (MoveId::LedgeAttack, 23, 0, 100, 45, 90, 20),
        (MoveId::GetUpAttack, 18, 0, 70, 48, 80, 48),
    ];
    let sim = duel();
    let w = arms(&sim);
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
        (MoveId::Jab2, 180),
        (MoveId::FTilt, 100),
        (MoveId::UTilt, 100),
        (MoveId::DTilt, 100),
        (MoveId::DashAttack, 125),
        (MoveId::FSmash, 140),
        (MoveId::USmash, 100),
        (MoveId::DSmash, 100),
        (MoveId::NAir, 100),
        (MoveId::BAir, 100),
        (MoveId::UAir, 100),
        (MoveId::DAir, 100),
    ] {
        assert!(all(id, &|h| h.hitlag == hitlag), "{id:?}");
    }
    assert!(all(MoveId::FAir, &|h| h.hitlag
        == if h.start + 1 == 21 { 150 } else { 100 }));
    // Angle types (the raw tables' `af`): forward for the jabs, forward tilt, dash attack, forward smash, forward air and ledge
    // attack; backward for the back air; by position (no `af`) for the up and down tilts, up and down smash, neutral, up and down air
    // and the get-up attack.
    for (id, t) in [
        (MoveId::Jab, ANGLE_FORWARD),
        (MoveId::Jab2, ANGLE_FORWARD),
        (MoveId::FTilt, ANGLE_FORWARD),
        (MoveId::DashAttack, ANGLE_FORWARD),
        (MoveId::FSmash, ANGLE_FORWARD),
        (MoveId::FAir, ANGLE_FORWARD),
        (MoveId::LedgeAttack, ANGLE_FORWARD),
        (MoveId::BAir, ANGLE_BACKWARD),
        (MoveId::UTilt, ANGLE_BY_POSITION),
        (MoveId::DTilt, ANGLE_BY_POSITION),
        (MoveId::USmash, ANGLE_BY_POSITION),
        (MoveId::DSmash, ANGLE_BY_POSITION),
        (MoveId::NAir, ANGLE_BY_POSITION),
        (MoveId::UAir, ANGLE_BY_POSITION),
        (MoveId::DAir, ANGLE_BY_POSITION),
        (MoveId::GetUpAttack, ANGLE_BY_POSITION),
    ] {
        assert!(all(id, &|h| h.angle_type == t), "{id:?}");
    }
    // The forward tilt: straight 8%, aimed up or down 9%; only the down one trips (15%).
    for h in &get(MoveId::FTilt).hitboxes {
        let (damage, trip) = match h.aim {
            AIM_STRAIGHT => (80, 0),
            AIM_UP => (90, 0),
            AIM_DOWN => (90, 15),
            _ => panic!("every forward tilt box is aimed"),
        };
        assert_eq!((h.damage, h.trip), (fx(damage, 10), trip));
    }
    assert_eq!(get(MoveId::FTilt).hitboxes.len(), 9);
    // The down tilt: +5 hitstun and a 40% trip on every box (13.0.1). Nothing else in the kit trips.
    assert!(all(MoveId::DTilt, &|h| h.hitstun_add == 5 && h.trip == 40));
    for (id, ..) in FRAME_DATA {
        if !matches!(id, MoveId::FTilt | MoveId::DTilt | MoveId::NSpecial) {
            assert!(all(id, &|h| h.trip == 0 && h.hitstun_add == 0), "{id:?}");
        }
    }
    // Shield damage: the smashes' +1 (forward 105, down 106 / 106 / 110) and +4 (up 121), the ledge attack's +1 (110), the get-up
    // attack's +8 (214) [D].
    assert!(all(MoveId::FSmash, &|h| h.shield_damage == 105));
    assert!(all(MoveId::USmash, &|h| h.shield_damage == 121));
    assert!(all(MoveId::DSmash, &|h| h.shield_damage
        == if h.damage == Fx::from_int(14) {
            107
        } else {
            106
        }));
    assert!(all(MoveId::LedgeAttack, &|h| h.shield_damage == 110));
    assert!(all(MoveId::GetUpAttack, &|h| h.shield_damage == 214));
    let pummel = get(MoveId::Pummel).hitboxes[0];
    assert_eq!(
        (pummel.damage, pummel.fixed_kb, pummel.hitlag),
        (fx(16, 10), 30, 230)
    );
}

#[test]
fn the_smashes_clank_without_rebounding_and_only_the_references_no_clank_boxes_never_clank() {
    // ssbwiki.com's raw tables (13.0.1): `rebound=f` alone on every smash box (E3g); `clang=f rebound=f` on the pummel, the ledge
    // attack and the get-up attacks (E3c). Every other normal clanks and rebounds.
    let sim = duel();
    let w = arms(&sim);
    for (id, ..) in FRAME_DATA {
        if id == MoveId::NSpecial {
            continue;
        }
        let smash = matches!(id, MoveId::FSmash | MoveId::USmash | MoveId::DSmash);
        let never = matches!(
            id,
            MoveId::Pummel | MoveId::LedgeAttack | MoveId::GetUpAttack
        );
        for h in &w.get(id as u8).hitboxes {
            assert_eq!(h.clanks, !never, "{id:?} clanks");
            assert_eq!(h.rebounds, !smash, "{id:?} rebounds");
        }
    }
}

// ---- Jab -----------------------------------------------------------------------------------------------------------------------

/// Runs player 0's attack button down on the ticks in `down` (1-based) with player 1 in front (untouchable); returns (tick, move,
/// move frame, state) after each tick for `ticks` ticks.
fn jab_run(down: &[u32], ticks: u32) -> Vec<(u32, MoveId, u16, S)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(20, 10), -1);
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
    // Pressed on 1 and 3: jab 1 from tick 1 (hits 5-6), jab 2 from 8 (jab 1's frame 8, its continuable window; hits 11-13) and control
    // back on 39: the reference's quickest loop (5-6, 11-13, FAF 25 / 39).
    let run = jab_run(&[1, 3], 50);
    assert_eq!(started(&run, MoveId::Jab), Some(1));
    assert_eq!(started(&run, MoveId::Jab2), Some(8));
    let sim = duel();
    let w = arms(&sim);
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
    assert_eq!(out_on(MoveId::Jab), vec![5, 6]);
    assert_eq!(out_on(MoveId::Jab2), vec![11, 12, 13]);
    let back = run.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1);
    assert_eq!(back, Some(39), "jab 2's FAF, counted from jab 1");
    // Jab 1 alone gives control back on 25.
    let one = jab_run(&[1], 40);
    assert_eq!(
        one.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1),
        Some(25)
    );
    // A third press does nothing more: there is no third hit and no rapid jab.
    let three = jab_run(&[1, 3, 12, 20], 50);
    assert_eq!(
        three.iter().find(|r| r.3 != S::Attack).map(|r| r.0 + 1),
        Some(39)
    );
}

// ---- Meteors and directions ----------------------------------------------------------------------------------------------------

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
fn the_forward_airs_late_fist_and_the_down_airs_feet_are_meteors() {
    // The forward air's late fist (frame 21) and the down air's box under the feet (frame 14): straight down.
    let fair = launch_of(MoveId::FAir, 20, 17, -9);
    assert!(fair.y < Fx::ZERO && fair.x == Fx::ZERO, "{fair:?}");
    let dair = launch_of(MoveId::DAir, 13, 1, -18);
    assert!(dair.y < Fx::ZERO && dair.x == Fx::ZERO, "{dair:?}");
    // The forward air's clean hit (frame 18) sends up and forward instead.
    let clean = launch_of(MoveId::FAir, 17, 15, 9);
    assert!(clean.y > Fx::ZERO && clean.x > Fx::ZERO, "{clean:?}");
}

#[test]
fn the_back_air_sends_backward_and_the_down_smash_sends_away_on_both_sides() {
    assert!(launch_of(MoveId::BAir, 6, -19, -2).x < Fx::ZERO);
    assert!(launch_of(MoveId::DSmash, 10, 19, 1).x > Fx::ZERO);
    assert!(launch_of(MoveId::DSmash, 10, -19, 1).x < Fx::ZERO);
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
fn the_standing_grab_catches_on_frame_8() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    assert_eq!(grab_now(&mut sim), 8);
    for id in [MoveId::Grab, MoveId::DashGrab, MoveId::PivotGrab] {
        assert!(arms(&sim)
            .get(id as u8)
            .hitboxes
            .iter()
            .all(|h| h.kind == HIT_GRAB));
    }
}

#[test]
fn the_throws_release_on_their_frames_for_their_damage() {
    // (stick, the frame the held fighter's percent rises on, damage in tenths); no throw has a pre-hit. The forward throw is the
    // carry's placeholder (the forward cargo throw's numbers [E]).
    for (stick, frame, damage) in [
        ((127i8, 0i8), 15u8, 120),
        ((-127, 0), 15, 110),
        ((0, 127), 14, 90),
        ((0, -127), 19, 70),
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
    let w = arms(&sim);
    for id in [
        MoveId::FThrow,
        MoveId::BThrow,
        MoveId::UThrow,
        MoveId::DThrow,
    ] {
        assert!(w.get(id as u8).hitboxes.iter().all(|h| h.kind == HIT_THROW));
    }
}

#[test]
fn the_pummel_does_1_6_percent() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Pummel as u8);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 16));
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
    let plain = duel().content.fighters[usize::from(HAULER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window; the tick adds
    // one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 15u16, 10u8, 26u16, Some(3u16)),
        (MoveId::FAir, 30, 17, 55, None),
        (MoveId::BAir, 15, 11, 30, Some(2)),
        (MoveId::UAir, 15, 15, 25, Some(2)),
        (MoveId::DAir, 30, 14, 49, Some(0)),
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
fn the_haulers_jumps_have_the_references_air_times() {
    // Short hop 40, full hop 53, short hop fast fall 27, full hop fast fall 37 (ultimateframedata.com); within one frame.
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
        (true, false, 53),
        (false, true, 27),
        (true, true, 37),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_hauler_is_the_references_super_heavyweight() {
    let p = duel().content.fighters[usize::from(HAULER)];
    assert_eq!(p, FighterParams::hauler());
    let su = FighterParams::su;
    let gu = FighterParams::gu;
    assert_eq!(p.weight, Fx::from_int(127));
    assert_eq!(
        (p.walk_speed, p.dash_speed, p.run_speed),
        (gu(1365), gu(2090), gu(1873))
    );
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(1208), su(85), su(1630), su(2608))
    );
    assert_eq!(
        (p.air_accel, p.air_accel_stick, p.air_friction),
        (su(10), su(50), su(15))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (48, 66, 82, 109)
    );
    assert_eq!(
        (
            p.air_dodge_intangible_start,
            p.air_dodge_intangible_end,
            p.air_dodge_dir_intangible_end
        ),
        (4, 32, 22)
    );
    assert_eq!(
        (p.roll_frames, p.roll_back_frames, p.spot_dodge_frames),
        (30, 35, 26)
    );
    assert_eq!((p.dash_frames, p.heavy_landing_lag), (15, 3));
    assert_eq!((p.ledge_option_decay_2, p.ledge_option_decay_3), (80, 48));
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (0, su(1300), su(2700))
    );
    assert_eq!(p.weapon, HAULER);
    // Bigger than the shared body [E].
    let shared = duel().content.fighters[0];
    assert!(p.ecb_height > shared.ecb_height && p.ecb_half_width > shared.ecb_half_width);
}

#[test]
fn the_bracers_are_registered_as_class_7() {
    let c = Content::placeholder();
    assert_eq!(c.names.fighters[7], "hauler");
    assert_eq!(c.names.weapons[7], "bracers");
    assert_eq!(c.fighters[7], FighterParams::hauler());
    assert_eq!(c.weapons[7], sim_core::moves::bracers());
    // The specials are the reference's own since phase 2 (`bracers_specials.rs`): none is the claws' any more.
    for id in [
        MoveId::NSpecial,
        MoveId::SideSpecial,
        MoveId::UpSpecial,
        MoveId::DownSpecial,
    ] {
        assert_ne!(
            c.weapons[7].moves[id as usize], c.weapons[1].moves[id as usize],
            "{id:?}"
        );
    }
}

// ---- Determinism ---------------------------------------------------------------------------------------------------------------

#[test]
fn random_play_with_the_hauler_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [HAULER, 0, HAULER, 4];
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
