//! The Gauntlets' specials (class 4, the sprinter body; phase 2), modelled mechanically on a reference character: frame data, hits
//! and the behaviour of each special against `docs/movesets/captain_falcon/REFERENCE_DATA.md`, and the extensions they brought
//! (search boxes, command grabs, no-clank melee hits, situation changes that keep the frame, the wall register).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{
    MoveId, Weapon, ANGLE_BY_BOX, ARMOR_DAMAGE, HIT_SEARCH, TARGETS_AERIAL, TARGETS_GROUNDED,
};
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
        angle_type: ANGLE_BY_BOX,
        command: false,
        clanks: true,
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

// ---- Up special: the leaping command grab ----------------------------------------------------------------------------------

const DIVE_CATCH: MoveId = MoveId::Ext6;

#[test]
fn the_up_specials_frame_data_is_the_references() {
    use sim_core::moves::{HIT_GRAB, HIT_NORMAL, HIT_PUMMEL, HIT_THROW};
    let sim = duel();
    let w = gauntlets(&sim);
    let dive = w.get(MoveId::UpSpecial as u8);
    assert_eq!(
        windows(w, MoveId::UpSpecial, HIT_GRAB),
        vec![(14, 14), (15, 30)]
    );
    assert!(dive.hitboxes.iter().all(|h| h.command), "command grabs");
    assert_eq!(
        faf(w, MoveId::UpSpecial),
        67,
        "an animation of 66, then the special fall"
    );
    assert!(dive.helpless_after);
    assert_eq!((dive.fall_drift, dive.fall_landing_lag), (70, 24));
    assert_eq!(
        (dive.ledge_front_from + 1, dive.ledge_back_from + 1),
        (20, 37)
    );
    assert_eq!(dive.fast_fall_from + 1, 50);
    assert_eq!(
        dive.turn_at,
        Some((13, 25)),
        "turns only on frame 14, past 0.25"
    );
    let before = dive
        .air_blocks
        .iter()
        .find(|b| b.from == 0)
        .expect("startup block");
    assert_eq!((before.to + 1, before.accel, before.drift), (13, 600, 95));
    let after = dive
        .air_blocks
        .iter()
        .find(|b| b.from > 0)
        .expect("fall block");
    assert_eq!(after.gravity, Some(su(103)));
    assert_eq!(dive.catch_move, Some(DIVE_CATCH as u8));

    let catch = w.get(DIVE_CATCH as u8);
    assert_eq!(windows(w, DIVE_CATCH, HIT_PUMMEL), vec![(2, 3)]);
    assert_eq!(windows(w, DIVE_CATCH, HIT_NORMAL), vec![(2, 3)]);
    assert_eq!(windows(w, DIVE_CATCH, HIT_THROW), vec![(17, 17)]);
    assert_eq!(faf(w, DIVE_CATCH), 40);
    assert!(!catch.helpless_after, "free after a catch");
    let throw = catch.hitboxes.iter().find(|h| h.kind == HIT_THROW).unwrap();
    assert_eq!(
        (
            throw.damage,
            throw.angle,
            throw.base_knockback,
            throw.knockback_growth
        ),
        (Fx::from_int(13), 50, 85, 62)
    );
    assert!(catch
        .hitboxes
        .iter()
        .filter(|h| h.kind != HIT_THROW)
        .all(
            |h| (h.damage, h.angle, h.base_knockback, h.knockback_growth)
                == (Fx::from_int(5), 361, 50, 100)
        ));
}

/// The sprinter's up special from standing at x = -4 (stick up), with the duelist held still in the air at `target` (relative), until
/// it is caught. Returns the sim and the reference frame of the catch.
fn dive_at(target: Vec2, grounded_target: bool) -> (Sim, Option<u16>) {
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-4), 1);
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    let mut caught = None;
    for tick in 2..=40u16 {
        if caught.is_none() {
            let me = sim.f().pos;
            if grounded_target {
                sim.stand(1, me.x + target.x, -1);
            } else {
                sim.put_airborne(1, me.x + target.x, me.y + target.y, Fx::ZERO, Fx::ZERO);
            }
        }
        sim.tick(inp(0, 0, 0));
        if caught.is_none() && sim.fighter(1).state == S::Grabbed {
            caught = Some(tick);
        }
    }
    (sim, caught)
}

#[test]
fn the_leap_catches_an_airborne_fighter_on_frame_14_explodes_and_throws_it() {
    let (mut sim, caught) = dive_at(Vec2::new(fx(9, 10), fx(10, 10)), false);
    assert_eq!(caught, Some(14));
    // Run on through the catch: 5% from the explosion and 13% from the throw.
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 180));
    assert_eq!(sim.fighter(1).grab_with, sim_core::state::NONE);
    // Free afterwards, not in the special fall; and the leap can be used again.
    assert!(
        matches!(sim.f().state, S::Airborne | S::Idle | S::Landing),
        "{:?}",
        sim.f().state
    );
    if sim.f().state == S::Airborne {
        sim.tick(inp(0, 127, SPECIAL));
        assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    }
}

#[test]
fn the_leap_catches_a_grounded_or_shielding_fighter_in_front() {
    let (_, caught) = dive_at(Vec2::new(fx(15, 10), Fx::ZERO), true);
    assert_eq!(
        caught,
        Some(14),
        "the frame-14 box that reaches grounded fighters"
    );
    // Shielding: the same, through the shield.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-4), 1);
    sim.stand(1, Fx::from_int(-4) + fx(15, 10), -1);
    sim.state.fighters[1].state = S::Shield;
    sim.state.fighters[1].state_frame = 30;
    sim.tick(inp(0, 127, SPECIAL));
    let mut got = false;
    for _ in 0..20 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, sim_core::input::buttons::SHIELD));
        got |= sim.fighter(1).state == S::Grabbed;
    }
    assert!(got);
}

#[test]
fn the_explosion_also_hits_a_bystander() {
    let mut sim = Sim::with_chars([SPRINTER, 0, 0, 0]);
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-4), 1);
    sim.tick(inp(0, 127, SPECIAL));
    for _ in 2..=20 {
        if sim.fighter(1).state != S::Grabbed {
            let me = sim.f().pos;
            sim.put_airborne(1, me.x + fx(9, 10), me.y + Fx::ONE, Fx::ZERO, Fx::ZERO);
        }
        // The bystander, just behind the sprinter.
        let me = sim.f().pos;
        sim.put_airborne(2, me.x - fx(5, 10), me.y + fx(5, 10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.fighter(2).percent,
        dealt(&sim, 50),
        "5% to the bystander"
    );
}

#[test]
fn a_missed_leap_ends_in_the_special_fall_after_frame_66() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-4), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 127, SPECIAL));
    sim.ticks(64, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        (sim.f().state, sim.f().move_id),
        (S::Helpless, MoveId::UpSpecial as u8)
    );
}

#[test]
fn the_leap_turns_only_on_frame_14_and_drifts_hard_before_it() {
    // Up and back from the start: no turn until frame 14, then a turn (the momentum with it).
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-4), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(-127, 127, SPECIAL));
    assert_eq!(
        sim.f().facing,
        1,
        "up and back does not reverse it at the start"
    );
    sim.ticks(12, inp(-127, 0, 0));
    assert_eq!((sim.f().state_frame, sim.f().facing), (12, 1));
    let vx = sim.f().vel.x;
    assert!(vx < Fx::ZERO, "drifting back");
    sim.tick(inp(-127, 0, 0));
    assert_eq!(sim.f().facing, -1, "turned on frame 14");
    // The x6 acceleration: 12 frames held forward build far more speed than the body's own air control would.
    let drift = |accel: u16| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(10), -1);
        let w = usize::from(SPRINTER);
        sim.content.weapons[w].moves[MoveId::UpSpecial as usize].air_blocks[0].accel = accel;
        sim.put_airborne(0, Fx::from_int(-4), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(0, 127, SPECIAL));
        sim.ticks(3, inp(127, 0, 0));
        sim.f().vel.x
    };
    assert!(
        drift(600) > drift(100).mul_int(4),
        "{:?} vs {:?}",
        drift(600),
        drift(100)
    );
}

#[test]
fn the_leap_rises_and_cannot_fast_fall_before_frame_50() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-4), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 127, SPECIAL));
    let mut start = sim.f().pos.y;
    let mut top = start;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state_frame == 12 {
            start = sim.f().pos.y;
        }
        top = top.max(sim.f().pos.y);
    }
    // [E] about 34 reference units (4.3 world units) above the start of the rise.
    let risen = top - start;
    assert!(risen > fx(35, 10) && risen < fx(50, 10), "{risen:?}");
    // Frame 42: falling; a flick down does nothing yet.
    sim.tick(inp(0, -127, 0));
    assert!(!sim.f().fast_fall);
    sim.ticks(6, inp(0, 0, 0));
    // Reference frame 49: still not.
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().state_frame, 48);
    assert!(!sim.f().fast_fall);
    // Frame 50: the flick (still held, within the fast-fall buffer) takes.
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall, "frame 50");
}

// ---- Neutral special: the wound-up punch -----------------------------------------------------------------------------------

const PUNCH_REVERSED: MoveId = MoveId::Ext0;
const PUNCH_AIR: MoveId = MoveId::Ext1;
const PUNCH_AIR_REVERSED: MoveId = MoveId::Ext2;

#[test]
fn the_neutral_specials_frame_data_is_the_references() {
    let sim = duel();
    let w = gauntlets(&sim);
    // (The reversed versions start on the reference's frame 15: their frames are the reference's minus 14.)
    for (id, window, faf_from_start, damage, bkb, kbg, fist) in [
        (MoveId::NSpecial, (53, 57), 104, 25, 93, 59, fx(6, 10)),
        (PUNCH_REVERSED, (62, 66), 118, 28, 85, 65, fx(6, 10)),
        (PUNCH_AIR, (53, 57), 104, 22, 93, 59, fx(6, 10)),
        (PUNCH_AIR_REVERSED, (62, 66), 118, 25, 93, 59, fx(6, 10)),
    ] {
        let shift = if matches!(id, PUNCH_REVERSED | PUNCH_AIR_REVERSED) {
            14
        } else {
            0
        };
        assert_eq!(
            windows(w, id, 0),
            vec![(window.0 - shift, window.1 - shift)],
            "{id:?}"
        );
        assert_eq!(faf(w, id) + u16::from(shift), faf_from_start, "{id:?}");
        let mv = w.get(id as u8);
        assert!(mv.hitboxes.iter().all(|h| (
            h.damage,
            h.angle,
            h.base_knockback,
            h.knockback_growth,
            h.hitlag,
            h.clanks
        ) == (Fx::from_int(damage), 361, bkb, kbg, 150, false)));
        assert_eq!(mv.hitboxes[0].radius, fist);
    }
    // The aerial boxes are x1.15 the grounded ones (rounded to tenths).
    let (g, a) = (w.get(MoveId::NSpecial as u8), w.get(PUNCH_AIR as u8));
    let total = |m: &sim_core::moves::Move| m.hitboxes.iter().fold(Fx::ZERO, |s, h| s + h.radius);
    assert!(total(a) > total(g));
    assert_eq!(g.air_version, Some(PUNCH_AIR as u8));
}

/// The punch from standing at x = -4 facing right (`stick_x` held from frame 2), with the duelist standing at `target` (relative);
/// returns the reference frame player 1 was first hit on, and the sim after `ticks` frames.
fn punch(target: Fx, stick_x: i8, ticks: u16) -> (Option<u16>, Sim) {
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-4), 1);
    sim.stand(
        1,
        Fx::from_int(-4) + target,
        if target > Fx::ZERO { -1 } else { 1 },
    );
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    let mut hit = None;
    for tick in 2..=ticks {
        sim.tick(inp(stick_x, 0, 0));
        if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
            hit = Some(tick);
        }
    }
    (hit, sim)
}

#[test]
fn the_grounded_punch_hits_on_frame_53_for_25_percent_and_ends_on_104() {
    let (hit, sim) = punch(fx(18, 10), 0, 102);
    assert_eq!(hit, Some(53));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 250));
    assert_eq!(sim.f().state, S::Attack);
    // (Missed: the hit's hitlag would hold the move back.)
    let (_, sim) = punch(Fx::from_int(10), 0, 103);
    assert_eq!(sim.f().state, S::Idle, "actionable on 104");
}

#[test]
fn the_stick_held_back_on_frame_15_reverses_the_punch() {
    // The stick held back from frame 2: on frame 15 the punch switches, turns round on 35 and hits behind on 62, for 28%.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-4), 1);
    sim.stand(1, Fx::from_int(-4) - fx(18, 10), 1);
    sim.tick(inp(0, 0, SPECIAL));
    let mut hit = None;
    for tick in 2..=116u16 {
        // (Not in the first five frames, where the stick back is a B-reverse.)
        sim.tick(inp(if tick > 6 { -127 } else { 0 }, 0, 0));
        match tick {
            15 => assert_eq!(sim.f().move_id, PUNCH_REVERSED as u8),
            34 => assert_eq!(sim.f().facing, 1, "not turned yet"),
            35 => assert_eq!(sim.f().facing, -1, "turned on 35"),
            _ => {}
        }
        if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
            hit = Some(tick);
        }
    }
    assert_eq!(hit, Some(62));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 280));
    // Missed (nobody near): actionable on 118, counted from the start of the punch.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-4), 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim.tick(inp(0, 0, SPECIAL));
    for tick in 2..=116u16 {
        sim.tick(inp(if tick > 6 && tick < 30 { -127 } else { 0 }, 0, 0));
    }
    assert_eq!(sim.f().state, S::Attack);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle, "actionable on 118");
    // Not held back on frame 15: no reversal.
    let (_, sim) = punch(fx(18, 10), 0, 20);
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
}

#[test]
fn the_aerial_punch_rises_a_little_and_bleeds_its_speed_for_19_frames() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    let vx = su(1000);
    sim.put_airborne(0, Fx::from_int(-4), Fx::from_int(20), vx, -su(1000));
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, PUNCH_AIR as u8);
    sim.tick(inp(0, 0, 0));
    let g = sim.content.fighters[usize::from(SPRINTER)].gravity;
    // The set 0.2 upward speed, less a frame of gravity.
    assert_eq!(sim.f().vel.y, su(200) - g);
    // After the reference's 19 frames: x0.92 nineteen times, the stick not steering meanwhile.
    for _ in 2..=18 {
        sim.tick(inp(127, 0, 0));
    }
    let expected = Fx::from_ratio(92, 100);
    let mut v = vx;
    for _ in 0..19 {
        v = v * expected;
    }
    assert!(
        (sim.f().vel.x - v).abs() < fx(2, 1000),
        "{:?} vs {v:?}",
        sim.f().vel.x
    );
}

#[test]
fn the_aerial_punch_is_boosted_along_35_degrees_by_the_stick_on_frame_51() {
    let boost = |stick_y: i8| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(10), -1);
        sim.put_airborne(0, Fx::from_int(-4), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(0, 0, SPECIAL));
        sim.ticks(49, inp(0, 0, 0));
        sim.tick(inp(0, stick_y, 0));
        assert_eq!(sim.f().state_frame, 50);
        sim.f().vel
    };
    // Half way up: boosted up and forward; all the way up (past 0.65): nothing.
    let up = boost(64);
    assert!(up.x > Fx::ZERO && up.y > Fx::ZERO, "{up:?}");
    let down = boost(-64);
    assert!(down.y < Fx::ZERO && down.x > Fx::ZERO);
    let none = boost(127);
    assert!(none.y < Fx::ZERO && none.x == Fx::ZERO, "{none:?}");
}

#[test]
fn the_punch_never_clanks_with_a_grounded_attack() {
    // The duelist's jab meets the fist on frame 53: no rebound for either; the punch lands.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-4), 1);
    sim.stand(1, Fx::from_int(-4) + fx(30, 10), -1);
    sim.tick(inp(0, 0, SPECIAL));
    for tick in 2..=60u16 {
        let p1 = if tick == 51 {
            inp(0, 0, sim_core::input::buttons::ATTACK)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, 0), p1);
        assert_ne!(sim.f().state, S::Rebound, "frame {tick}");
    }
}

// ---- Down special: the slide kick and the dive ---------------------------------------------------------------------------

const KICK_AIR: MoveId = MoveId::Ext7;
const KICK_LANDING: MoveId = MoveId::Ext8;
const KICK_OFF_EDGE: MoveId = MoveId::Ext9;
const KICK_CRASH: MoveId = MoveId::Ext13;

#[test]
fn the_down_specials_frame_data_is_the_references() {
    let sim = duel();
    let w = gauntlets(&sim);
    for id in [MoveId::DownSpecial, KICK_OFF_EDGE] {
        assert_eq!(
            windows(w, id, 0),
            vec![(13, 16), (17, 23), (24, 33)],
            "{id:?}"
        );
        let phases: Vec<_> = w
            .get(id as u8)
            .hitboxes
            .iter()
            .map(|h| {
                (
                    h.damage.floor_int(),
                    h.angle,
                    h.base_knockback,
                    h.knockback_growth,
                    h.group,
                )
            })
            .collect();
        for p in [(15, 52, 60, 88, 0), (11, 52, 90, 60, 0), (7, 46, 80, 65, 0)] {
            assert!(phases.contains(&p), "{id:?} {p:?}");
        }
    }
    assert_eq!(faf(w, MoveId::DownSpecial), 69);
    assert_eq!(faf(w, KICK_OFF_EDGE), 66);
    assert_eq!(faf(w, KICK_CRASH), 54, "a 53-frame crash");
    let ground = w.get(MoveId::DownSpecial as u8);
    assert_eq!(
        (ground.off_edge, ground.air_version),
        (Some(KICK_OFF_EDGE as u8), Some(KICK_AIR as u8))
    );
    assert_eq!(windows(w, KICK_AIR, 0), vec![(14, 18), (19, 24), (25, 27)]);
    assert_eq!(faf(w, KICK_AIR), 57);
    assert!(!w.get(KICK_AIR as u8).helpless_after);
    assert_eq!(w.get(KICK_AIR as u8).landing_move, Some(KICK_LANDING as u8));
    assert_eq!(windows(w, KICK_LANDING, 0), vec![(1, 2)]);
    assert_eq!(faf(w, KICK_LANDING), 36, "35 frames of landing lag");
    let landing = w.get(KICK_LANDING as u8);
    assert!(
        landing.hitboxes.iter().any(|h| h.x < Fx::ZERO)
            && landing.hitboxes.iter().any(|h| h.x > Fx::ZERO)
    );
    assert!(landing.hitboxes.iter().all(|h| (
        h.damage,
        h.angle,
        h.base_knockback,
        h.knockback_growth
    ) == (Fx::from_int(9), 60, 80, 35)));
}

/// The ground kick from x (facing right), the duelist standing at `target` or far away; returns the first reference frame player 1
/// was hit on and the sim after `ticks`.
fn kick(x: Fx, target: Option<Fx>, ticks: u16) -> (Option<u16>, Sim) {
    let mut sim = duel();
    sim.stand(0, x, 1);
    sim.stand(1, target.unwrap_or(Fx::from_int(-10)), -1);
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
    let mut hit = None;
    for tick in 2..=ticks {
        sim.tick(inp(0, 0, 0));
        if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
            hit = Some(tick);
        }
    }
    (hit, sim)
}

#[test]
fn the_slide_kick_hits_cleanly_early_and_ends_on_69() {
    // A target close in front: the clean hit (frame 13 at the earliest).
    let (hit, sim) = kick(Fx::from_int(-6), Some(Fx::from_int(-4)), 40);
    let at = hit.expect("hit");
    assert!((13..=16).contains(&at), "frame {at}");
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 150));
    // Nobody: actionable on 69, still on the ground.
    let (_, sim) = kick(Fx::from_int(-6), None, 67);
    assert_eq!(sim.f().state, S::Attack);
    let (_, sim) = kick(Fx::from_int(-6), None, 68);
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn the_slide_kick_off_an_edge_goes_on_as_its_off_edge_version_and_ends_on_66() {
    let edge = Fx::from_int(11);
    let (_, sim) = kick(edge - Fx::from_int(2), None, 30);
    assert_eq!(sim.f().move_id, KICK_OFF_EDGE as u8);
    assert!(!sim.f().grounded());
    // The switch keeps the frame and the hits so far.
    let (_, mut sim) = kick(edge - Fx::from_int(2), None, 3);
    sim.state.fighters[0].hit_mask = 1 << 1;
    let mut switched = false;
    for _ in 0..30 {
        let frame = sim.f().state_frame;
        sim.tick(inp(0, 0, 0));
        if !switched && sim.f().move_id == KICK_OFF_EDGE as u8 {
            switched = true;
            assert_eq!(sim.f().state_frame, frame + 1);
            assert_eq!(sim.f().hit_mask, 1 << 1);
        }
    }
    assert!(switched);
    let (_, sim) = kick(edge - Fx::from_int(2), None, 64);
    assert_eq!(sim.f().state, S::Attack);
    let (_, sim) = kick(edge - Fx::from_int(2), None, 65);
    assert_eq!(sim.f().state, S::Airborne, "free on 66, not helpless");
}

#[test]
fn the_slide_kick_crashes_into_a_wall_on_frames_14_to_36() {
    let mut sim = duel();
    sim.content
        .stage
        .platforms
        .push(sim_core::content::Platform {
            left: Fx::from_int(0),
            right: Fx::from_int(2),
            y: Fx::from_int(3),
            bottom: Fx::ZERO,
            pass_through: false,
        });
    sim.stand(0, Fx::from_int(-3), 1);
    sim.stand(1, Fx::from_int(-10), -1);
    sim.tick(inp(0, -127, SPECIAL));
    let mut crashed = None;
    for tick in 2..=40u16 {
        sim.tick(inp(0, 0, 0));
        if crashed.is_none() && sim.f().move_id == KICK_CRASH as u8 {
            crashed = Some(tick);
        }
    }
    let at = crashed.expect("crashed");
    assert!((15..=37).contains(&at), "frame {at}");
}

#[test]
fn the_dive_kick_lands_into_its_landing_hit_on_both_sides() {
    // The dive from low over the stage (nobody near) touches down mid-move: the landing hit, 35 frames in all.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, KICK_AIR as u8);
    let mut landed_at = None;
    for tick in 2..=60u16 {
        sim.tick(inp(0, 0, 0));
        if landed_at.is_none() && sim.f().move_id == KICK_LANDING as u8 {
            landed_at = Some(tick);
            assert!(sim.f().grounded());
        }
        if let Some(t) = landed_at {
            if tick == t + 34 {
                assert_eq!(sim.f().state, S::Idle, "35 frames of lag");
            } else if tick < t + 34 {
                assert_eq!(sim.f().state, S::Attack);
            }
        }
    }
    assert!(landed_at.is_some());
    // The landing hit itself reaches both sides.
    let mut sim = Sim::with_chars([SPRINTER, 0, 0, 0]);
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(9, 10), -1);
    sim.stand(2, -fx(9, 10), 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], KICK_LANDING);
    sim_core::combat::resolve_hits(&mut sim.state, &sim.content);
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 90));
    assert_eq!(sim.fighter(2).percent, dealt(&sim, 90));
}
