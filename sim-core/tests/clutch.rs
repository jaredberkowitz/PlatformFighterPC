//! The Clutch moveset on the bounder body (class 8), modelled mechanically on a reference character (phase 1: the body, its double
//! jump's armor and turnaround, and every normal; the specials are still the claws'). Frame data against the reference for every
//! normal, checked in the data and by running the moves; hit properties (damage, angles and angle types, knockback, set weight, hitlag
//! and SDI multipliers, trip chances, shield damage, no-clank boxes); the jab's continuable window and its two held rules; the down
//! air's twelve kicks, finisher and landing hit; the head and foot intangibility; grabs, pummel and throws; landing lag and
//! autocancels; the body's attributes, jump air times and double jump; the double jump's subtractive knockback armor (E33);
//! registration; determinism. Reference data and sources: the ninth moveset's `REFERENCE_DATA.md` (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::intangible_parts;
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD};
use sim_core::moves::clutch_slots::DAIR_LANDING;
use sim_core::moves::{
    Move, MoveId, Weapon, AIM_DOWN, AIM_STRAIGHT, AIM_UP, ANGLE_AUTOLINK, ANGLE_FORWARD, HIT_GRAB,
    HIT_THROW, PART_HIGH, PART_LIMB, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const BOUNDER: u8 = 8;

/// The bounder (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([BOUNDER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(BOUNDER)].weapon)]
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

/// Every normal, from `REFERENCE_DATA.md` (all [R]; the up tilt's one window 8-15 is split into the parts of its sweep, [E]
/// positions; the grabs' mouth box is out on their first two frames).
const FRAME_DATA: [FrameData; 26] = [
    (MoveId::Jab, &[(3, 3)], 18, 0, 0, 255),
    (MoveId::Jab2, &[(3, 4)], 20, 0, 0, 255),
    (MoveId::FTilt, &[(5, 7)], 30, 0, 0, 255),
    (MoveId::UTilt, &[(8, 10), (11, 12), (13, 15)], 32, 0, 0, 255),
    (MoveId::DTilt, &[(8, 10)], 24, 0, 0, 255),
    (MoveId::DashAttack, &[(10, 12), (13, 20)], 41, 0, 0, 255),
    (MoveId::FSmash, &[(14, 16)], 53, 0, 0, 255),
    (MoveId::USmash, &[(11, 13), (14, 16)], 47, 0, 0, 255),
    (MoveId::DSmash, &[(7, 8), (22, 23)], 50, 0, 0, 255),
    (MoveId::NAir, &[(3, 4), (5, 11), (12, 25)], 45, 7, 2, 38),
    (MoveId::FAir, &[(16, 20)], 44, 12, 3, 40),
    (MoveId::BAir, &[(11, 11), (14, 14), (18, 19)], 56, 11, 5, 44),
    (MoveId::UAir, &[(5, 6)], 37, 8, 4, 31),
    (
        MoveId::DAir,
        &[
            (16, 16),
            (18, 18),
            (20, 20),
            (22, 22),
            (24, 24),
            (26, 26),
            (28, 28),
            (30, 30),
            (32, 32),
            (34, 34),
            (36, 36),
            (38, 38),
            (41, 41),
        ],
        57,
        17,
        13,
        50,
    ),
    (DAIR_LANDING, &[(1, 1)], 18, 0, 0, 255),
    (MoveId::Grab, &[(14, 15), (14, 21)], 49, 0, 0, 255),
    (MoveId::DashGrab, &[(16, 17), (16, 23)], 57, 0, 0, 255),
    (MoveId::PivotGrab, &[(17, 18), (17, 24)], 52, 0, 0, 255),
    (MoveId::Pummel, &[(2, 2)], 8, 0, 0, 255),
    (MoveId::FThrow, &[(15, 15)], 40, 0, 0, 255),
    (MoveId::BThrow, &[(20, 20)], 44, 0, 0, 255),
    (MoveId::UThrow, &[(13, 13)], 44, 0, 0, 255),
    (MoveId::DThrow, &[(26, 26)], 52, 0, 0, 255),
    (MoveId::LedgeAttack, &[(20, 22)], 56, 0, 0, 255),
    (MoveId::GetUpAttack, &[(14, 15), (20, 21)], 46, 0, 0, 255),
    // (The specials are still placeholders: phases 2 and 3.)
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
        if id != MoveId::DAir {
            assert_eq!(mv.rehit, None, "{id:?}: only the down air is a multi-hit");
        }
    }
    // The down air: its kick group re-armed every 2 frames from reference frame 16, the finisher (group 1) never; the landing hit.
    let dair = w.get(MoveId::DAir as u8);
    assert_eq!((dair.rehit, dair.rehit_groups), (Some((15, 2)), 0b1));
    assert_eq!(dair.landing_move, Some(DAIR_LANDING as u8));
    assert!(dair
        .hitboxes
        .iter()
        .all(|h| h.group == u8::from(h.start + 1 == 41)));
    // Intangibility: the ledge attack through frame 22 (the first grab), the get-up attack through 21.
    assert_eq!(w.get(MoveId::LedgeAttack as u8).intangible, 22);
    assert_eq!(w.get(MoveId::GetUpAttack as u8).intangible, 21);
    // Smash charge frames (ssbwiki.com's "charges between" 7-8, 7-8, 3-4: move frames one less than the first).
    assert_eq!(w.get(MoveId::FSmash as u8).charge_at, Some(6));
    assert_eq!(w.get(MoveId::USmash as u8).charge_at, Some(6));
    assert_eq!(w.get(MoveId::DSmash as u8).charge_at, Some(2));
    // The jab's continuable window from frame 6 (move frame 5) into jab 2, which ends the chain.
    let jab1 = w.get(MoveId::Jab as u8);
    assert_eq!((jab1.next, jab1.next_from), (Some(MoveId::Jab2 as u8), 5));
    assert_eq!(w.get(MoveId::Jab2 as u8).next, None);
}

#[test]
fn the_head_and_foot_intangibility_windows_are_the_references() {
    // The forward smash's head and nose 12-16 (the high circle and the limb circle reaching toward the hit), the up smash's kicking
    // foot 11-13 (the limb circle).
    let sim = duel();
    let w = kit(&sim);
    for (id, first, last, parts) in [
        (MoveId::FSmash, 12u16, 16u16, PART_HIGH | PART_LIMB),
        (MoveId::USmash, 11, 13, PART_LIMB),
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
    assert!(
        w.limbs,
        "kicks, a tail and a head: the limb circle is the body's own"
    );
    for (id, ..) in FRAME_DATA {
        if !matches!(id, MoveId::FSmash | MoveId::USmash) {
            assert!(w.get(id as u8).part_intangible.is_empty(), "{id:?}");
        }
    }
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
    let hb = *kit(&sim)
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
        (MoveId::Jab, 3u8),
        (MoveId::Jab2, 3),
        (MoveId::FTilt, 5),
        (MoveId::UTilt, 8),
        (MoveId::DTilt, 8),
        (MoveId::DashAttack, 10),
        (MoveId::FSmash, 14),
        (MoveId::USmash, 11),
        (MoveId::DSmash, 7),
        (MoveId::NAir, 3),
        (MoveId::FAir, 16),
        (MoveId::BAir, 11),
        (MoveId::UAir, 5),
        (MoveId::DAir, 16),
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
        // (Throws need someone held, the ledge attack a ledge; the landing hit is reached by landing.)
        if faf == 255
            || id == DAIR_LANDING
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
    let table: [(MoveId, u8, u8, i32, i16, i16, i16); 41] = [
        (MoveId::Jab, 3, 0, 30, 361, 40, 30),
        (MoveId::Jab, 3, 1, 30, 361, 35, 28),
        (MoveId::Jab, 3, 2, 30, 180, 10, 20),
        (MoveId::Jab2, 3, 0, 40, 361, 56, 70),
        (MoveId::Jab2, 3, 2, 40, 361, 56, 70),
        (MoveId::FTilt, 5, 0, 80, 88, 60, 75),
        (MoveId::FTilt, 5, 1, 80, 85, 60, 75),
        (MoveId::FTilt, 5, 2, 80, 85, 60, 75),
        (MoveId::UTilt, 8, 0, 70, 100, 72, 45),
        (MoveId::UTilt, 11, 2, 70, 100, 72, 45),
        (MoveId::DTilt, 8, 0, 50, 28, 67, 30),
        (MoveId::DTilt, 8, 1, 45, 28, 67, 30),
        (MoveId::DTilt, 8, 2, 40, 361, 67, 30),
        (MoveId::DashAttack, 10, 0, 110, 65, 90, 45),
        (MoveId::DashAttack, 13, 1, 80, 65, 90, 44),
        (MoveId::FSmash, 14, 0, 155, 361, 30, 102),
        (MoveId::FSmash, 14, 1, 140, 361, 30, 97),
        (MoveId::USmash, 11, 0, 140, 75, 38, 99),
        (MoveId::USmash, 14, 1, 120, 75, 38, 99),
        (MoveId::DSmash, 7, 0, 120, 26, 51, 75),
        (MoveId::DSmash, 7, 1, 100, 26, 46, 63),
        (MoveId::DSmash, 22, 0, 120, 26, 51, 83),
        (MoveId::DSmash, 22, 1, 100, 26, 44, 75),
        (MoveId::NAir, 3, 0, 100, 361, 40, 86),
        (MoveId::NAir, 5, 0, 70, 361, 30, 100),
        (MoveId::NAir, 12, 0, 50, 361, 30, 100),
        (MoveId::FAir, 16, 0, 150, 361, 0, 90),
        (MoveId::FAir, 16, 1, 140, 270, 0, 90),
        (MoveId::BAir, 11, 0, 35, ANGLE_AUTOLINK, 20, 100),
        (MoveId::BAir, 14, 0, 35, ANGLE_AUTOLINK, 20, 100),
        (MoveId::BAir, 18, 0, 55, 54, 30, 184),
        (MoveId::UAir, 5, 1, 120, 90, 25, 100),
        (MoveId::DAir, 16, 0, 23, ANGLE_AUTOLINK, 5, 80),
        (MoveId::DAir, 26, 1, 18, ANGLE_AUTOLINK, 5, 80),
        (MoveId::DAir, 28, 0, 19, ANGLE_AUTOLINK, 5, 100),
        (MoveId::DAir, 38, 2, 14, ANGLE_AUTOLINK, 5, 100),
        (MoveId::DAir, 41, 0, 28, 80, 55, 179),
        (MoveId::DAir, 41, 2, 28, 40, 55, 179),
        (DAIR_LANDING, 1, 0, 10, 361, 65, 100),
        (MoveId::LedgeAttack, 20, 0, 90, 45, 90, 20),
        (MoveId::GetUpAttack, 14, 0, 70, 48, 80, 48),
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
    for (id, hitlag) in [
        (MoveId::Jab, 150),
        (MoveId::Jab2, 200),
        (MoveId::FTilt, 100),
        (MoveId::UTilt, 100),
        (MoveId::DTilt, 100),
        (MoveId::DashAttack, 100),
        (MoveId::FSmash, 100),
        (MoveId::USmash, 100),
        (MoveId::DSmash, 100),
        (MoveId::NAir, 100),
        (MoveId::FAir, 100),
        (MoveId::UAir, 100),
        (MoveId::Pummel, 210),
    ] {
        assert!(all(id, &|h| h.hitlag == hitlag), "{id:?}");
    }
    assert!(all(MoveId::BAir, &|h| (h.hitlag, h.sdi, h.set_weight)
        == if h.start + 1 < 18 {
            (50, 50, true)
        } else {
            (150, 100, false)
        }));
    assert!(all(MoveId::DAir, &|h| (h.hitlag, h.sdi)
        == if h.start + 1 < 41 {
            (60, 180)
        } else {
            (150, 100)
        }));
    // The forward tilt and smash: the same hit data for each aim, three aims each.
    for id in [MoveId::FTilt, MoveId::FSmash] {
        for aim in [AIM_UP, AIM_STRAIGHT, AIM_DOWN] {
            let n = get(id).hitboxes.iter().filter(|h| h.aim == aim).count();
            assert_eq!(n, if id == MoveId::FTilt { 3 } else { 2 }, "{id:?}");
        }
    }
    // Trips: only the down tilt (30 / 30 / 40%).
    assert!(all(MoveId::DTilt, &|h| h.trip
        == if h.priority == 2 { 40 } else { 30 }));
    for (id, ..) in FRAME_DATA {
        if id != MoveId::DTilt {
            assert!(all(id, &|h| h.trip == 0 && h.hitstun_add == 0), "{id:?}");
        }
    }
    // Shield damage: the ledge attack's +1 (111), the get-up attack's +8 (214) [D]; nothing else adds any.
    assert!(all(MoveId::LedgeAttack, &|h| h.shield_damage == 111));
    assert!(all(MoveId::GetUpAttack, &|h| h.shield_damage == 214));
    for (id, ..) in FRAME_DATA {
        if !matches!(id, MoveId::LedgeAttack | MoveId::GetUpAttack) {
            assert!(
                all(id, &|h| h.shield_damage == 100 && h.shieldstun == 100),
                "{id:?}"
            );
        }
    }
    let pummel = get(MoveId::Pummel).hitboxes[0];
    assert_eq!(
        (pummel.damage, pummel.fixed_kb, pummel.angle_type),
        (fx(13, 10), 30, ANGLE_FORWARD)
    );
}

#[test]
fn only_the_references_no_clank_boxes_never_clank_and_every_box_rebounds() {
    // ssbwiki.com's raw tables (13.0.1): `clang=f rebound=f` on the pummel, the ledge attack and the get-up attacks (E3c); no normal
    // has `rebound=f` alone.
    let sim = duel();
    let w = kit(&sim);
    for (id, ..) in FRAME_DATA {
        let never = matches!(
            id,
            MoveId::Pummel | MoveId::LedgeAttack | MoveId::GetUpAttack
        );
        for h in &w.get(id as u8).hitboxes {
            assert_eq!(h.clanks, !never, "{id:?} clanks");
            assert!(h.rebounds, "{id:?} rebounds");
        }
    }
}

// ---- Jab -----------------------------------------------------------------------------------------------------------------------

/// Runs player 0's attack button down on the ticks in `down` (1-based) with player 1 in front (`hittable` or not); returns (tick,
/// move, move frame, state) after each tick for `ticks` ticks.
fn jab_run(down: &[u32], ticks: u32, hittable: bool) -> Vec<(u32, MoveId, u16, S)> {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(9, 10), -1);
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
    // Pressed on 1 and 3: jab 1 from tick 1 (hits on 3), jab 2 from 6 (jab 1's frame 6, its continuable window; hits 8-9) and
    // control back on 25: the reference's quickest loop (3, 8-9, FAF 18 / 25).
    let run = jab_run(&[1, 3], 40, false);
    assert_eq!(started(&run, MoveId::Jab), Some(1));
    assert_eq!(started(&run, MoveId::Jab2), Some(6));
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
    assert_eq!(out_on(MoveId::Jab), vec![3]);
    assert_eq!(out_on(MoveId::Jab2), vec![8, 9]);
    assert_eq!(
        control_back(&run),
        Some(25),
        "jab 2's FAF, counted from jab 1"
    );
    // Jab 1 alone gives control back on 18; more presses add no third hit (no rapid jab).
    assert_eq!(control_back(&jab_run(&[1], 30, false)), Some(18));
    assert_eq!(control_back(&jab_run(&[1, 3, 9, 14], 40, false)), Some(25));
}

#[test]
fn holding_attack_carries_on_from_frame_6_after_a_hit_and_from_frame_16_without_one() {
    // (Jab 1's last move frame before jab 2 starts: 4 is its frame 5, so jab 2 replaces its frame 6. A hit's hitlag holds the move
    // frame still, so this is counted in move frames, not ticks.)
    let last_jab1_frame = |run: &[(u32, MoveId, u16, S)]| {
        let at = started(run, MoveId::Jab2).expect("jab 2 started");
        run.iter()
            .rfind(|r| r.0 < at && r.1 == MoveId::Jab)
            .map(|r| r.2)
    };
    let held: Vec<u32> = (1..=60).collect();
    // Held, and jab 1 hit: jab 2 from jab 1's frame 6 (the reference's "held on hit").
    let hit = jab_run(&held, 60, true);
    assert_eq!(last_jab1_frame(&hit), Some(4));
    // Held, nothing hit: jab 2 from frame 16 ("held without hitting"), on tick 16.
    let miss = jab_run(&held, 40, false);
    assert_eq!(started(&miss, MoveId::Jab2), Some(16));
    assert_eq!(last_jab1_frame(&miss), Some(14));
    // Let go after the press: no jab 2 at all.
    assert_eq!(started(&jab_run(&[1], 40, true), MoveId::Jab2), None);
}

// ---- The down air ----------------------------------------------------------------------------------------------------------------

#[test]
fn the_down_air_kicks_twelve_times_then_finishes_for_28_percent() {
    // The target held under the middle kick every frame: a hit on each kick's frame (16, 18, ..., 38) and the finisher on 41, counted
    // in the move's own frames (hitlag freezes the kicker between them).
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    let mut rises = Vec::new();
    let mut last = sim.fighter(1).percent;
    for _ in 0..200 {
        let me = sim.f().pos;
        let middle = sim.content.fighters[0].ecb_height * Fx::from_ratio(11, 20);
        sim.put_airborne(1, me.x + fx(3, 10), me.y - middle, Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].vel = Vec2::ZERO;
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            last = sim.fighter(1).percent;
            rises.push(sim.f().state_frame as u8 + 1);
        }
        if sim.f().state != S::Attack {
            break;
        }
    }
    let mut expected: Vec<u8> = (0..12).map(|k| 16 + 2 * k).collect();
    expected.push(41);
    assert_eq!(rises, expected);
    // 6 x 2.3 + 6 x 1.9 + 2.8 = 28% (each hit's damage scaled by the one-on-one multiplier on its own).
    let total =
        (0..6).fold(Fx::ZERO, |s, _| s + dealt(&sim, 23) + dealt(&sim, 19)) + dealt(&sim, 28);
    assert_eq!(last, total);
    assert!((last - dealt(&sim, 280)).abs() < fx(1, 1000));
}

#[test]
fn the_down_air_landing_outside_its_windows_is_the_landing_hit() {
    let mut sim = duel();
    sim.stand(1, fx(3, 10), -1);
    sim.put_airborne(0, Fx::ZERO, fx(2, 100), Fx::ZERO, fx(-1, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = MoveId::DAir as u8;
    f.state_frame = 30;
    f.hit_mask = 0;
    let before = sim.fighter(1).percent;
    // It hits on the landing frame itself.
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        (sim.f().state, sim.f().move_id),
        (S::Attack, DAIR_LANDING as u8)
    );
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 10));
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            break;
        }
    }
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 10));
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
fn the_tongue_grab_catches_on_frame_14_at_long_range() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    assert_eq!(grab_now(&mut sim), 14);
    // The tongue reaches further than the shared grab [E: 22 reference units].
    let mut far = duel();
    far.stand(0, Fx::ZERO, 1);
    far.stand(1, fx(28, 10), -1);
    assert_eq!(grab_now(&mut far), 14);
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
    // (stick, the frame the held fighter's percent rises on, damage in tenths); no throw has a pre-hit. The down throw's 8 frames of
    // hitlag are folded into its timeline (release 26, [D]).
    for (stick, frame, damage) in [
        ((127i8, 0i8), 15u8, 90),
        ((-127, 0), 20, 90),
        ((0, 127), 13, 50),
        ((0, -127), 26, 40),
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
fn the_pummel_does_1_3_percent() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Pummel as u8);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent - before, dealt(&sim, 13));
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
    let plain = duel().content.fighters[usize::from(BOUNDER)].landing_lag;
    // (aerial, a move frame inside the move, its landing lag, a move frame in the late window, one in the early window; the tick adds
    // one to the move frame)
    for (id, mid, lag, late, early) in [
        (MoveId::NAir, 15u16, 7u8, 37u16, 0u16),
        (MoveId::FAir, 20, 12, 39, 1),
        (MoveId::BAir, 20, 11, 43, 3),
        (MoveId::UAir, 15, 8, 30, 2),
    ] {
        assert_eq!(
            land_during(id, mid),
            (S::Landing, lag, id as u8),
            "{id:?} mid-move"
        );
        assert_eq!(land_during(id, late).1, plain, "{id:?} late");
        assert_eq!(land_during(id, early).1, plain, "{id:?} early");
    }
    // The down air: inside its windows a plain landing, outside them the landing hit (17 frames of lag, FAF 18).
    assert_eq!(land_during(MoveId::DAir, 49).1, plain);
    assert_eq!(land_during(MoveId::DAir, 11).1, plain);
    assert_eq!(
        land_during(MoveId::DAir, 30),
        (S::Attack, 0, DAIR_LANDING as u8)
    );
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
fn the_bounders_jumps_have_the_references_air_times() {
    // Short hop 38, full hop 60, short hop fast fall 26, full hop fast fall 42 (ultimateframedata.com); within one frame.
    for (full, ff, reference) in [
        (false, false, 38u32),
        (true, false, 60),
        (false, true, 26),
        (true, true, 42),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

#[test]
fn the_double_jump_rises_51_56_reference_units_and_peaks_on_about_frame_36() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    let y0 = sim.f().pos.y;
    sim.tick(inp(0, 0, JUMP));
    let mut top = sim.f().pos.y;
    let mut peak_frame = 1;
    for frame in 2..=60u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().pos.y > top {
            top = sim.f().pos.y;
            peak_frame = frame;
        }
    }
    // 51.56 of the reference's units is 6.445 world units [D].
    let rise = top - y0;
    assert!(
        (rise - fx(6445, 1000)).abs() < fx(5, 100),
        "the double jump rose {rise:?}"
    );
    assert!((35..=37).contains(&peak_frame), "peak on {peak_frame}");
}

#[test]
fn the_bounder_is_the_references_floaty_heavyweight() {
    let p = duel().content.fighters[usize::from(BOUNDER)];
    assert_eq!(p, FighterParams::bounder());
    let su = FighterParams::su;
    let gu = FighterParams::gu;
    assert_eq!(p.weight, Fx::from_int(104));
    assert_eq!(
        (p.walk_speed, p.dash_speed, p.run_speed),
        (gu(1208), gu(1980), gu(2046))
    );
    assert_eq!(
        (p.air_speed, p.gravity, p.max_fall_speed, p.fast_fall_speed),
        (su(1344), su(80), su(1290), su(2064))
    );
    assert_eq!(
        (p.air_accel, p.air_accel_stick, p.air_friction),
        (su(30), su(68), su(5))
    );
    assert_eq!(
        (
            p.air_dodge_frames,
            p.air_dodge_dir_down_frames,
            p.air_dodge_dir_side_frames,
            p.air_dodge_dir_up_frames
        ),
        (58, 79, 107, 130)
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
    assert_eq!((p.dash_frames, p.heavy_landing_lag), (14, 6));
    assert_eq!((p.ledge_option_decay_2, p.ledge_option_decay_3), (82, 50));
    assert_eq!(
        (p.wall_jump, p.wall_jump_vx, p.wall_jump_vy),
        (0, su(1300), su(2300))
    );
    assert_eq!(p.weapon, BOUNDER);
    assert_eq!(
        p.air_jump_armor,
        Some(FighterParams::bounder_air_jump_armor())
    );
    assert!(p.air_jump_turns);
    // No other built-in body has either.
    for (i, q) in Content::placeholder().fighters.iter().enumerate() {
        if i != usize::from(BOUNDER) {
            assert!(
                q.air_jump_armor.is_none() && !q.air_jump_turns,
                "fighter {i}"
            );
        }
    }
}

// ---- The double jump's armor (E33) -------------------------------------------------------------------------------------------------

#[test]
fn the_double_jump_counts_its_frames_and_anything_else_ends_it() {
    let jumped = || {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(10), -1);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, JUMP));
        sim
    };
    let mut sim = jumped();
    assert_eq!(sim.f().air_jump_frame, 1);
    // (The jump's speed, less the frame's gravity.)
    let p = sim.content.fighters[8];
    assert_eq!(sim.f().vel.y, p.air_jump_velocity - p.gravity);
    sim.ticks(9, inp(0, 0, 0));
    assert_eq!(sim.f().air_jump_frame, 10);
    sim.ticks(300, inp(0, 0, 0));
    assert_eq!(sim.f().air_jump_frame, 0, "a landing ends it");
    // An aerial ends it.
    let mut sim = jumped();
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!((sim.f().state, sim.f().air_jump_frame), (S::Attack, 0));
    // An air dodge ends it.
    let mut sim = jumped();
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!((sim.f().state, sim.f().air_jump_frame), (S::AirDodge, 0));
    // A first jump from the ground is not one.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim.ticks(6, inp(0, 0, JUMP));
    assert!(!sim.f().grounded());
    assert_eq!(sim.f().air_jump_frame, 0);
}

#[test]
fn the_double_jump_turns_round_with_the_stick_held_back() {
    for (char_id, turns) in [(BOUNDER, true), (0u8, false)] {
        let mut sim = Sim::with_chars([char_id, 0, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.stand(1, Fx::from_int(10), -1);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(-127, 0, JUMP));
        assert_eq!(sim.f().facing, if turns { -1 } else { 1 }, "body {char_id}");
        assert!(
            sim.f().vel.x < Fx::ZERO,
            "it drifts the way the stick holds"
        );
    }
    // Forward or neutral, it keeps its facing.
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(127, 0, JUMP));
    assert_eq!(sim.f().facing, 1);
}

#[test]
fn the_double_jump_drifts_1_2_times_the_stick_and_other_bodies_do_not() {
    // Owner decision 59: the body's `air_jump_drift` (the parameter file's `jump_aerial_speed_x_mul` 1.2 [R]); every other body 100.
    for (char_id, percent) in [(BOUNDER, 120), (0u8, 100)] {
        let mut sim = Sim::with_chars([char_id, 0, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.stand(1, Fx::from_int(10), -1);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
        let p = sim.content.fighters[usize::from(char_id)];
        assert_eq!(p.air_jump_drift, percent);
        sim.tick(inp(127, 0, JUMP));
        let full = p.air_speed * Fx::from_ratio(i32::from(percent), 100);
        // (The jump's own frame sets the speed; the air control of that frame may already trim it a little.)
        let vx = sim.f().vel.x;
        assert!(
            vx <= full && vx > full - p.air_accel - p.air_accel_stick - p.air_friction,
            "body {char_id}: {vx:?} against {full:?}"
        );
    }
}

/// The bounder (player 0) in the air, `air_jump_frame` at `frame` when the hit lands (0: not in a double jump), struck by player 1's
/// move: one box per `(first move frame, base knockback)` (no growth, so its knockback is the base knockback at any percent: the
/// reference's formula), each its own hit group, 1% each. Runs `ticks` ticks.
fn struck(frame: u8, hits: &[(u8, i16)], ticks: usize) -> Sim {
    let mut sim = duel();
    let mut template = sim.content.weapons[0].moves[MoveId::Jab as usize].hitboxes[0];
    let mut mv = Move::empty();
    mv.total_frames = 60;
    for (group, &(first, bk)) in hits.iter().enumerate() {
        template.start = first;
        template.end = first;
        template.x = Fx::ZERO;
        template.y = Fx::ONE;
        template.radius = Fx::from_int(3);
        template.damage = Fx::ONE;
        template.angle = 45;
        template.base_knockback = bk;
        template.knockback_growth = 0;
        template.group = group as u8;
        template.hitlag = 100;
        mv.hitboxes.push(template);
    }
    sim.content.weapons[0].moves[MoveId::NSpecial as usize] = mv;
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, fx(-5, 10), Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].facing = 1;
    // (The bounder's update counts the frame on before the hit is resolved; frame 1 is the jump itself, made on the first tick.)
    sim.state.fighters[0].air_jump_frame = frame.saturating_sub(1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::NSpecial);
    for t in 0..ticks {
        sim.state.fighters[1].vel = Vec2::ZERO;
        sim.state.fighters[1].pos.y = sim.state.fighters[0].pos.y;
        let jump = if frame == 1 && t == 0 { JUMP } else { 0 };
        sim.tick(inp(0, 0, jump));
    }
    sim
}

#[test]
fn the_double_jump_shrugs_off_up_to_120_units_of_knockback_on_its_frames_1_to_44() {
    // A 100-unit hit on frames 1 and 44: the damage and hitlag only; the jump goes on.
    for frame in [1u8, 44] {
        let sim = struck(frame, &[(1, 100)], 1);
        let f = sim.f();
        assert_eq!(f.percent, dealt(&sim, 10), "frame {frame}");
        assert_eq!(f.state, S::Airborne, "frame {frame}");
        assert!(f.hitlag > 0 && !f.launch_pending, "frame {frame}");
        assert_eq!(f.air_jump_frame, frame, "frame {frame}");
    }
    // Exactly 120 is still withstood.
    assert_eq!(struck(10, &[(1, 120)], 1).f().state, S::Airborne);
    // On frame 45 it is gone, and without a double jump there is none.
    for frame in [45u8, 0] {
        let sim = struck(frame, &[(1, 100)], 1);
        assert_eq!(sim.f().state, S::Hitstun, "frame {frame}");
        assert_eq!(sim.f().launch_kb, Fx::from_int(100), "frame {frame}");
        assert_eq!(sim.f().air_jump_frame, 0);
    }
}

#[test]
fn a_hit_that_breaks_the_armor_launches_with_its_knockback_less_120_and_at_least_60() {
    for (bk, launch) in [(200i16, 80), (150, 60), (121, 60), (300, 180)] {
        let sim = struck(10, &[(1, bk)], 1);
        assert_eq!(sim.f().state, S::Hitstun, "{bk}");
        assert_eq!(sim.f().launch_kb, Fx::from_int(launch), "{bk}");
    }
}

#[test]
fn each_hit_on_the_double_jump_is_compared_with_120_on_its_own() {
    // Two 100-unit hits (owner decision 47: no 120-unit pool): both withstood, the jump still going.
    let sim = struck(5, &[(1, 100), (15, 100)], 30);
    let f = sim.f();
    assert_eq!(f.percent, dealt(&sim, 20));
    assert_eq!(f.state, S::Airborne);
    assert!(f.air_jump_frame > 20);
}

#[test]
fn a_command_grab_catches_a_double_jumping_bounder() {
    let mut sim = struck(5, &[], 0);
    let mut grab = sim.content.weapons[0].moves[MoveId::Grab as usize].clone();
    for h in &mut grab.hitboxes {
        h.command = true;
        h.x = Fx::ZERO;
        h.y = Fx::ONE;
        h.radius = Fx::from_int(3);
    }
    sim.content.weapons[0].moves[MoveId::NSpecial as usize] = grab;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::NSpecial);
    for _ in 0..20 {
        sim.state.fighters[1].vel = Vec2::ZERO;
        sim.state.fighters[1].pos.y = sim.state.fighters[0].pos.y;
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbed {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Grabbed);
}

// ---- Registration and determinism ------------------------------------------------------------------------------------------------

#[test]
fn the_clutch_is_registered_as_class_8() {
    let c = Content::placeholder();
    assert_eq!(c.names.fighters[8], "bounder");
    assert_eq!(c.names.weapons[8], "clutch");
    assert_eq!(c.fighters[8], FighterParams::bounder());
    assert_eq!(c.weapons[8], sim_core::moves::clutch());
    // The neutral and side specials are still the claws' placeholders (phase 3); the up and down specials are phase 2's
    // (`clutch_specials.rs`), and the claws' up special endings (ext0-ext2) are gone with it.
    for id in [MoveId::UpSpecial, MoveId::DownSpecial] {
        assert_ne!(
            c.weapons[8].moves[id as usize], c.weapons[1].moves[id as usize],
            "{id:?}"
        );
    }
    for id in [MoveId::Ext0, MoveId::Ext1, MoveId::Ext2] {
        assert!(c.weapons[8].moves[id as usize].is_empty(), "{id:?}");
    }
    for id in [MoveId::NSpecial, MoveId::SideSpecial] {
        assert_eq!(
            c.weapons[8].moves[id as usize], c.weapons[1].moves[id as usize],
            "{id:?}"
        );
    }
}

#[test]
fn random_play_with_the_bounder_is_deterministic_and_rolls_back() {
    let content = Content::placeholder();
    let chars = [BOUNDER, 0, BOUNDER, 7];
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
