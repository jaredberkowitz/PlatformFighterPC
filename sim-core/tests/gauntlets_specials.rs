//! The Gauntlets' specials (class 4, the sprinter body; phase 2), modelled mechanically on a reference character: frame data, hits
//! and the behaviour of each special against `docs/movesets/captain_falcon/REFERENCE_DATA.md`, and the extensions they brought
//! (search boxes, command grabs, no-clank melee hits, situation changes that keep the frame, the wall register).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{MoveId, Weapon, ARMOR_DAMAGE, HIT_SEARCH, TARGETS_AERIAL, TARGETS_GROUNDED};
use sim_core::state::FighterState as S;
use sim_core::trig::Angle;
use sim_core::{Fx, Vec2};

const SPRINTER: u8 = 4;

const RAPTOR_GROUND_HIT: MoveId = MoveId::Ext3;
const RAPTOR_AIR: MoveId = MoveId::Ext4;
const RAPTOR_AIR_HIT: MoveId = MoveId::Ext5;

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

fn su(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

/// The reference frames (first, last) of a move's hitboxes of `kind`.
fn windows(w: &Weapon, id: MoveId, kind: u8) -> Vec<(u8, u8)> {
    let mut out: Vec<(u8, u8)> = w
        .get(id as u8)
        .hitboxes
        .iter()
        .filter(|h| h.kind == kind)
        .map(|h| (h.start + 1, h.end + 1))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The reference FAF of a move (total frames + 2).
fn faf(w: &Weapon, id: MoveId) -> u16 {
    u16::from(w.get(id as u8).total_frames) + 2
}

// ---- Side special: the dash, the uppercut and the overhand ---------------------------------------------------------------

#[test]
fn the_side_specials_frame_data_is_the_references() {
    let sim = duel();
    let w = gauntlets(&sim);
    let dash = w.get(MoveId::SideSpecial as u8);
    assert_eq!(windows(w, MoveId::SideSpecial, HIT_SEARCH), vec![(10, 25)]);
    assert_eq!(faf(w, MoveId::SideSpecial), 72, "missed on the ground");
    assert_eq!(dash.on_search, Some(RAPTOR_GROUND_HIT as u8));
    assert_eq!(dash.air_version, Some(RAPTOR_AIR as u8));
    // Search boxes on grounded and on airborne fighters.
    assert!(dash.hitboxes.iter().any(|h| h.targets == TARGETS_GROUNDED));
    assert!(dash.hitboxes.iter().any(|h| h.targets == TARGETS_AERIAL));

    let hit = w.get(RAPTOR_GROUND_HIT as u8);
    assert_eq!(windows(w, RAPTOR_GROUND_HIT, 0), vec![(5, 5), (6, 7)]);
    assert_eq!(faf(w, RAPTOR_GROUND_HIT), 28);
    let armor = hit.armor.expect("the uppercut is armored");
    assert_eq!(
        (armor.start + 1, armor.end + 1, armor.kind, armor.threshold),
        (1, 4, ARMOR_DAMAGE, Fx::from_int(10))
    );
    assert!(hit.hitboxes.iter().all(|h| (
        h.damage,
        h.angle,
        h.base_knockback,
        h.knockback_growth,
        h.hitstun_add,
        h.hitlag,
        h.shield_damage
    ) == (Fx::from_int(10), 85, 82, 30, 3, 150, 140)));

    let air = w.get(RAPTOR_AIR as u8);
    assert_eq!(windows(w, RAPTOR_AIR, HIT_SEARCH), vec![(19, 35)]);
    assert_eq!(
        faf(w, RAPTOR_AIR),
        80,
        "an animation of 79, then the special fall"
    );
    assert!(air.helpless_after);
    assert_eq!(
        (air.fall_drift, air.fall_landing_lag, air.landing_lag),
        (70, 22, 22)
    );
    assert_eq!(
        (air.ledge_front_from + 1, air.ledge_back_from + 1),
        (26, 26)
    );
    assert_eq!(air.on_search, Some(RAPTOR_AIR_HIT as u8));

    let over = w.get(RAPTOR_AIR_HIT as u8);
    assert_eq!(windows(w, RAPTOR_AIR_HIT, 0), vec![(5, 5), (6, 6)]);
    assert_eq!(faf(w, RAPTOR_AIR_HIT), 45);
    assert!(!over.helpless_after, "free after the overhand");
    assert_eq!(
        (
            over.lands_from + 1,
            over.ledge_front_from + 1,
            over.ledge_back_from + 1
        ),
        (8, 8, 8)
    );
    let meteor = over
        .hitboxes
        .iter()
        .find(|h| h.angle == 270)
        .expect("the meteor");
    assert_eq!(
        (
            meteor.targets,
            meteor.base_knockback,
            meteor.knockback_growth,
            meteor.hitlag
        ),
        (TARGETS_AERIAL, 60, 70, 200)
    );
    assert!(over
        .hitboxes
        .iter()
        .filter(|h| h.targets == TARGETS_GROUNDED)
        .all(|h| (h.angle, h.knockback_growth) == (60, 80)));
}

/// Player 0 (sprinter) dashes right from x = -6; player 1 stands (or shields) at `x`. Returns the sim after the move has had
/// `ticks` frames.
fn ground_boost(x: Fx, shield: bool, ticks: usize) -> Sim {
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-6), 1);
    sim.stand(1, x, -1);
    if shield {
        sim.state.fighters[1].state = S::Shield;
        sim.state.fighters[1].state_frame = 30;
    }
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
    for _ in 1..ticks {
        let p1 = if shield {
            inp(0, 0, sim_core::input::buttons::SHIELD)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, 0), p1);
    }
    sim
}

#[test]
fn a_missed_grounded_dash_ends_on_frame_72() {
    // Nobody in reach.
    let far = Fx::from_int(10);
    let sim = ground_boost(far, false, 70);
    assert_eq!(
        (sim.f().state, sim.f().move_id),
        (S::Attack, MoveId::SideSpecial as u8)
    );
    let sim = ground_boost(far, false, 71);
    assert_eq!(sim.f().state, S::Idle, "actionable on frame 72");
    assert!(sim.f().grounded());
}

#[test]
fn the_dash_switches_to_the_uppercut_on_touching_a_fighter_or_a_shield() {
    for shield in [false, true] {
        // The duelist 3 units ahead: the dash's search reaches it during frames 10-25.
        let mut sim = ground_boost(Fx::from_int(-3), shield, 1);
        let mut switched_at = None;
        for tick in 2..=40u16 {
            let p1 = if shield {
                inp(0, 0, sim_core::input::buttons::SHIELD)
            } else {
                inp(0, 0, 0)
            };
            sim.tick2(inp(0, 0, 0), p1);
            if sim.f().move_id == RAPTOR_GROUND_HIT as u8 && switched_at.is_none() {
                switched_at = Some(tick);
                assert_eq!(sim.f().state_frame, 0, "the uppercut's first frame");
                assert!(
                    sim.f().vel.x > Fx::ZERO,
                    "it keeps the dash's speed as it switches"
                );
            }
        }
        let at = switched_at.expect("the search touched the duelist");
        // Contact on reference frame 10 at the earliest, so the uppercut starts on frame 11 or later.
        assert!((11..=26).contains(&at), "switched on frame {at}");
        if shield {
            assert!(sim.fighter(1).shield_stun > 0 || sim.fighter(1).state == S::Shield);
            assert_eq!(sim.fighter(1).percent, Fx::ZERO);
        } else {
            assert_eq!(sim.fighter(1).percent, dealt(&sim, 100));
            assert!(matches!(sim.fighter(1).state, S::Hitstun | S::Airborne));
        }
    }
}

#[test]
fn the_search_box_deals_nothing_and_the_uppercut_cuts_the_dash_to_0_18() {
    let mut sim = ground_boost(Fx::from_int(-3), false, 1);
    let mut before = Fx::ZERO;
    for _ in 0..40 {
        let was_dash = sim.f().move_id == MoveId::SideSpecial as u8;
        let v = sim.f().vel.x;
        sim.tick(inp(0, 0, 0));
        if was_dash && sim.f().move_id == RAPTOR_GROUND_HIT as u8 {
            before = v;
            assert_eq!(
                sim.fighter(1).percent,
                Fx::ZERO,
                "the search itself hit nothing"
            );
            assert_eq!(sim.fighter(1).hitlag, 0);
            // Its frame 1: x0.18, then traction.
            sim.tick(inp(0, 0, 0));
            let p = &sim.content.fighters[usize::from(SPRINTER)];
            let slowed = before - before * Fx::from_ratio(82, 100);
            assert!(
                (sim.f().vel.x - (slowed - p.ground_friction).max(Fx::ZERO)).abs() < fx(1, 1000)
            );
            break;
        }
    }
    assert!(before > Fx::ZERO, "the switch happened");
}

#[test]
fn the_uppercut_withstands_a_hit_of_up_to_10_percent_on_frames_1_to_4() {
    use sim_core::moves::{Hitbox, Move, AIM_ANY, EFFECT_NORMAL, HIT_NORMAL, TARGETS_ALL};
    let jab = |damage: i32| Hitbox {
        start: 1,
        end: 1,
        x: fx(10, 10),
        y: fx(10, 10),
        radius: fx(9, 10),
        damage: Fx::from_int(damage),
        angle: 45,
        base_knockback: 80,
        knockback_growth: 100,
        priority: 0,
        group: 0,
        kind: HIT_NORMAL,
        shield_damage: 100,
        hitlag: 100,
        effect: EFFECT_NORMAL,
        targets: TARGETS_ALL,
        fixed_kb: 0,
        hitstun_add: 0,
        sdi: 100,
        link_ahead: Fx::ZERO,
        aim: AIM_ANY,
        set_weight: false,
        reverse: false,
        push: false,
        shieldstun: 100,
    };
    for (damage, holds) in [(10, true), (11, false)] {
        let mut sim = duel();
        sim.stand(0, Fx::ONE, -1);
        sim.stand(1, Fx::ZERO, 1);
        sim.content.weapons[0].moves[MoveId::NSpecial as usize] = Move {
            total_frames: 20,
            hitboxes: vec![jab(damage)],
            ..Move::empty()
        };
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], RAPTOR_GROUND_HIT);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::NSpecial);
        // Both on frame 1 next tick: the duelist's hit lands on the uppercut's frame 2.
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.fighter(0).percent, dealt(&sim, damage * 10));
        assert_eq!(sim.f().state == S::Attack, holds, "{damage}%");
    }
}

/// Player 0 in the air at height 20 facing right, its side special started; player 1 held in the air (or standing below) 2.0 ahead
/// until the search finds it.
fn air_boost(target_grounded: bool) -> (Sim, Option<u16>) {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, RAPTOR_AIR as u8, "the aerial version");
    let mut switched = None;
    for tick in 2..=60u16 {
        if switched.is_none() {
            let me = sim.f().pos;
            if target_grounded {
                // A floor under the target is not needed: it is held in place, on the "ground" (platform 0's height is not used).
                sim.state.fighters[1].pos = Vec2::new(me.x + fx(2, 1), me.y);
                sim.state.fighters[1].platform = 0;
                sim.state.fighters[1].state = S::Idle;
            } else {
                sim.put_airborne(1, me.x + fx(2, 1), me.y, Fx::ZERO, Fx::ZERO);
            }
        }
        sim.tick(inp(0, 0, 0));
        if switched.is_none() && sim.f().move_id == RAPTOR_AIR_HIT as u8 {
            switched = Some(tick);
        }
    }
    (sim, switched)
}

#[test]
fn the_aerial_dash_switches_to_the_overhand_which_meteors_airborne_fighters() {
    let (sim, switched) = air_boost(false);
    // The search is out from reference frame 19; the overhand starts the frame after contact.
    assert_eq!(switched, Some(20));
    let v = sim.fighter(1);
    assert_eq!(v.percent, dealt(&sim, 100));
    assert_eq!(
        v.launch_angle,
        Angle::from_degrees(270).raw(),
        "sent straight down"
    );
    assert_ne!(sim.f().state, S::Helpless, "free after a hit");
}

#[test]
fn the_overhand_sends_grounded_fighters_at_60_degrees() {
    let (sim, switched) = air_boost(true);
    assert!(switched.is_some());
    assert_eq!(sim.fighter(1).launch_angle, Angle::from_degrees(60).raw());
}

#[test]
fn the_overhand_sets_its_momentum_on_frame_6_and_cannot_land_before_frame_8() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), su(2000), Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], RAPTOR_AIR_HIT);
    sim.ticks(5, inp(0, 0, 0));
    assert_eq!(sim.f().state_frame, 5);
    assert_eq!(sim.f().vel, Vec2::new(-su(1500), su(2500)));
    // Just above the stage on frame 2: it touches down early and carries on.
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, fx(1, 10), Fx::ZERO, -fx(5, 10));
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], RAPTOR_AIR_HIT);
    sim.ticks(2, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
}

#[test]
fn a_missed_aerial_dash_ends_in_the_special_fall() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-6), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(127, 0, SPECIAL));
    sim.ticks(77, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        (sim.f().state, sim.f().move_id),
        (S::Helpless, RAPTOR_AIR as u8)
    );
}
