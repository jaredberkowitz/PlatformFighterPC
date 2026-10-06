//! Phase 3: hits, knockback, hitlag, hitstun, DI, shielding, KOs and move selection.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::{hitlag_frames, hitstun_frames, knockback};
use sim_core::input::buttons::{ATTACK, SHIELD};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{Fx, Vec2};

const JAB: u8 = MoveId::Jab as u8;
const FTILT: u8 = MoveId::FTilt as u8;
const UTILT: u8 = MoveId::UTilt as u8;
const DTILT: u8 = MoveId::DTilt as u8;
const DASH_ATTACK: u8 = MoveId::DashAttack as u8;
const FSMASH: u8 = MoveId::FSmash as u8;
const USMASH: u8 = MoveId::USmash as u8;
const DSMASH: u8 = MoveId::DSmash as u8;
const NAIR: u8 = MoveId::NAir as u8;
const FAIR: u8 = MoveId::FAir as u8;
const BAIR: u8 = MoveId::BAir as u8;
const UAIR: u8 = MoveId::UAir as u8;
const DAIR: u8 = MoveId::DAir as u8;

/// A forward tilt from player 0 at the given gap, with the stick tilted but not flicked.
fn ftilt(sim: &mut Sim) {
    sim.tick(inp(30, 0, ATTACK));
}

/// Ticks until player 1 has taken damage, returning the tick count (the attack's first tick is 1).
fn ticks_until_hit(sim: &mut Sim, limit: usize) -> Option<usize> {
    let before = sim.fighter(1).percent;
    for t in 2..=limit {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            return Some(t);
        }
    }
    None
}

// ---- Timing and hitlag ---------------------------------------------------------------------------------

#[test]
fn a_jab_hits_on_its_first_active_frame_and_not_before() {
    let mut sim = Sim::duel(fx(14, 10));
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, JAB);
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    // The jab's hitboxes start on frame 3, which is the 4th tick counting the button press as the first.
    assert_eq!(ticks_until_hit(&mut sim, 12), Some(4));
}

#[test]
fn hitlag_freezes_both_fighters_for_the_same_number_of_frames() {
    let mut sim = Sim::duel(fx(14, 10));
    sim.tick(inp(0, 0, ATTACK));
    ticks_until_hit(&mut sim, 12).expect("jab should hit");
    let tip_damage = Fx::from_int(4);
    let lag = hitlag_frames(tip_damage);
    assert_eq!(sim.f().hitlag, lag);
    assert_eq!(sim.fighter(1).hitlag, lag);
    let frozen_frame = sim.f().state_frame;
    let victim_pos = sim.fighter(1).pos;
    for _ in 0..lag {
        sim.tick(inp(0, 0, 0));
        assert_eq!(
            sim.f().state_frame,
            frozen_frame,
            "attacker must not advance during hitlag"
        );
        assert_eq!(
            sim.fighter(1).pos,
            victim_pos,
            "victim must not move during hitlag"
        );
    }
    sim.tick(inp(0, 0, 0));
    assert!(
        sim.f().state_frame > frozen_frame,
        "the attack resumes after hitlag"
    );
}

#[test]
fn a_move_only_hits_a_target_once() {
    // Neutral air has a hitbox active for 14 frames; one use must deal its damage once.
    let mut sim = Sim::duel(fx(14, 10));
    sim.put_airborne(0, Fx::ZERO, fx(8, 10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, NAIR);
    sim.ticks(20, inp(0, 0, 0));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(6),
        "damage should be applied exactly once"
    );
}

// ---- Tip and hilt --------------------------------------------------------------------------------------

fn ftilt_damage_at(gap: Fx) -> Fx {
    let mut sim = Sim::duel(gap);
    ftilt(&mut sim);
    sim.ticks(14, inp(0, 0, 0));
    sim.fighter(1).percent
}

#[test]
fn the_tip_of_the_sword_hits_harder_than_the_hilt() {
    let hilt_only = ftilt_damage_at(fx(10, 10));
    let tip_only = ftilt_damage_at(fx(35, 10));
    let both = ftilt_damage_at(fx(22, 10));
    assert_eq!(hilt_only, Fx::from_int(6));
    assert_eq!(tip_only, Fx::from_int(8));
    assert_eq!(
        both,
        Fx::from_int(8),
        "when both overlap, the tip (priority 0) wins"
    );
}

#[test]
fn out_of_range_attacks_miss() {
    assert_eq!(ftilt_damage_at(fx(60, 10)), Fx::ZERO);
}

#[test]
fn the_claws_have_much_shorter_reach_than_the_sword() {
    let reach = |chars: [u8; 4]| {
        let mut sim = Sim::with_chars(chars);
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(35, 10), -1);
        ftilt(&mut sim);
        sim.ticks(14, inp(0, 0, 0));
        sim.fighter(1).percent > Fx::ZERO
    };
    assert!(reach([0, 0, 0, 0]), "the sword reaches 3.5 units");
    assert!(!reach([1, 0, 0, 0]), "the claws do not");
}

// ---- Knockback, launch and hitstun ---------------------------------------------------------------------

/// Runs a forward tilt and returns (victim after hitlag ends, expected knockback).
fn launched_victim(setup: impl Fn(&mut Sim)) -> (Sim, Fx) {
    let mut sim = Sim::duel(fx(22, 10));
    setup(&mut sim);
    ftilt(&mut sim);
    ticks_until_hit(&mut sim, 20).expect("forward tilt should hit");
    let lag = sim.fighter(1).hitlag;
    let kb = sim.fighter(1).launch_kb;
    for _ in 0..lag {
        sim.tick(inp(0, 0, 0));
    }
    (sim, kb)
}

#[test]
fn the_launch_follows_the_knockback_formula() {
    let (sim, kb) = launched_victim(|_| {});
    let weight = sim.content.fighters[1].weight;
    let expected = knockback(Fx::from_int(8), Fx::from_int(8), weight, 12, 90);
    assert_eq!(kb, expected);
    let victim = sim.fighter(1);
    assert!(!victim.launch_pending, "the hit launches when hitlag ends");
    // Launch speed is 0.03 reference units per knockback unit, 8 reference units per world unit.
    let expected_speed = expected * Fx::from_ratio(3, 800);
    let speed = victim.kb_vel.length();
    assert!(
        (speed - expected_speed).abs() < fx(1, 200),
        "{speed:?} vs {expected_speed:?}"
    );
    // Attacker faces right and the knockback is above the grounded threshold: up and to the right.
    assert!(victim.kb_vel.x > Fx::ZERO && victim.kb_vel.y > Fx::ZERO);
    assert!(!victim.grounded());
    assert_eq!(victim.state, S::Hitstun);
}

#[test]
fn more_percent_means_a_harder_launch_and_longer_hitstun() {
    let (low, _) = launched_victim(|_| {});
    let (high, _) = launched_victim(|s| s.state.fighters[1].percent = Fx::from_int(120));
    assert!(high.fighter(1).kb_vel.length() > low.fighter(1).kb_vel.length());
    assert!(high.fighter(1).hitstun > low.fighter(1).hitstun);
}

#[test]
fn a_heavier_target_is_launched_less() {
    let hit_weight = |weight: i32| {
        let (sim, _) = launched_victim(|s| s.content.fighters[1].weight = Fx::from_int(weight));
        sim.fighter(1).kb_vel.length()
    };
    assert!(hit_weight(60) > hit_weight(130));
}

#[test]
fn the_hit_assigns_the_hitstun_the_formula_says() {
    let (sim, kb) = launched_victim(|_| {});
    assert_eq!(
        sim.fighter(1).hitstun,
        hitstun_frames(kb, sim.content.rules.hitstun_mult)
    );
}

#[test]
fn hitstun_counts_down_one_frame_at_a_time_and_then_the_fighter_can_act() {
    // Set up directly (high in the air, so landing cannot cut it short).
    let mut sim = Sim::new();
    sim.stand(0, fx(-80, 10), 1);
    sim.put_airborne(1, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].state = S::Hitstun;
    sim.state.fighters[1].hitstun = 20;
    let mut frames = 0;
    while sim.fighter(1).state == S::Hitstun {
        sim.tick(inp(0, 0, 0));
        frames += 1;
        assert!(frames < 100);
    }
    assert_eq!(frames, 20);
    assert_eq!(
        sim.fighter(1).state,
        S::Airborne,
        "actionable again once hitstun ends"
    );
}

#[test]
fn a_weak_launch_that_lands_before_hitstun_ends_goes_to_knockdown_lag_not_a_free_action() {
    let (mut sim, _) = launched_victim(|_| {});
    let mut ended_in = None;
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state != S::Hitstun {
            ended_in = Some(sim.fighter(1).state);
            break;
        }
    }
    assert_eq!(ended_in, Some(S::Landing));
}

#[test]
fn the_knockback_direction_mirrors_when_the_attacker_faces_left() {
    let mut sim = Sim::new();
    sim.stand(0, fx(30, 10), -1);
    sim.stand(1, fx(8, 10), 1);
    sim.tick(inp(-30, 0, ATTACK));
    ticks_until_hit(&mut sim, 20).expect("should hit");
    let lag = sim.fighter(1).hitlag;
    sim.ticks(usize::from(lag), inp(0, 0, 0));
    assert!(sim.fighter(1).kb_vel.x < Fx::ZERO, "victim should fly left");
    assert!(sim.fighter(1).kb_vel.y > Fx::ZERO);
}

#[test]
fn a_weak_hit_on_a_grounded_fighter_slides_them_instead_of_launching_them() {
    // The jab's knockback is below the grounded threshold, so the Sakurai angle is horizontal.
    let mut sim = Sim::duel(fx(14, 10));
    sim.tick(inp(0, 0, ATTACK));
    ticks_until_hit(&mut sim, 12).expect("jab should hit");
    sim.ticks(usize::from(sim.fighter(1).hitlag), inp(0, 0, 0));
    let victim = sim.fighter(1);
    assert!(
        victim.grounded(),
        "a weak hit keeps the victim on the ground"
    );
    assert_eq!(victim.kb_vel.y, Fx::ZERO);
    assert!(victim.kb_vel.x > Fx::ZERO);
}

// ---- Directional influence -----------------------------------------------------------------------------

/// The launch velocity after the victim holds `stick` through hitlag.
fn launch_with_di(stick: (i8, i8)) -> Vec2 {
    let mut sim = Sim::duel(fx(22, 10));
    ftilt(&mut sim);
    for _ in 0..20 {
        sim.tick2(inp(0, 0, 0), inp(stick.0, stick.1, 0));
        if sim.fighter(1).launch_pending {
            break;
        }
    }
    assert!(sim.fighter(1).launch_pending, "the hit should land");
    while sim.fighter(1).launch_pending {
        sim.tick2(inp(0, 0, 0), inp(stick.0, stick.1, 0));
    }
    sim.fighter(1).kb_vel
}

#[test]
fn directional_influence_bends_the_launch_toward_the_held_direction() {
    let steepness = |v: Vec2| v.y / v.x;
    let up = steepness(launch_with_di((0, 100)));
    let none = steepness(launch_with_di((0, 0)));
    let down = steepness(launch_with_di((0, -100)));
    assert!(up > none && none > down, "{up:?} {none:?} {down:?}");
}

#[test]
fn di_does_not_change_launch_speed_and_is_limited_to_a_few_degrees() {
    let none = launch_with_di((0, 0));
    let up = launch_with_di((0, 127));
    assert!(
        (none.length() - up.length()).abs() < fx(1, 100),
        "DI changes direction, not speed"
    );
    // Pushing straight along the launch direction (no perpendicular part) changes nothing much.
    let along = launch_with_di((90, 90));
    assert!((along.length() - none.length()).abs() < fx(1, 100));
}

// ---- Survival DI -----------------------------------------------------------------------------------------

#[test]
fn a_flick_during_hitlag_nudges_the_victim_once() {
    let mut sim = Sim::duel(fx(22, 10));
    ftilt(&mut sim);
    ticks_until_hit(&mut sim, 20).expect("should hit");
    let before = sim.fighter(1).pos.x;
    sim.tick2(inp(0, 0, 0), inp(127, 0, 0));
    sim.tick2(inp(0, 0, 0), inp(127, 0, 0));
    sim.tick2(inp(0, 0, 0), inp(127, 0, 0));
    assert_eq!(
        sim.fighter(1).pos.x - before,
        sim.content.rules.sdi_distance,
        "one flick, one nudge"
    );
}

// ---- Shields, invulnerability, trades ------------------------------------------------------------------

#[test]
fn a_shielding_fighter_takes_no_damage_and_is_not_launched() {
    let mut sim = Sim::duel(fx(22, 10));
    sim.tick2(inp(30, 0, ATTACK), inp(0, 0, SHIELD));
    for _ in 0..16 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    let victim = sim.fighter(1);
    assert_eq!(victim.percent, Fx::ZERO);
    assert_eq!(victim.state, S::Shield);
    assert_ne!(
        sim.f().hit_mask & 2,
        0,
        "the attack still counted as a hit on the shield"
    );
}

#[test]
fn invulnerable_fighters_cannot_be_hit() {
    for setup in [
        (|s: &mut Sim| s.state.fighters[1].invuln = 100) as fn(&mut Sim),
        |s: &mut Sim| s.state.fighters[1].ledge_invuln = 100,
    ] {
        let mut sim = Sim::duel(fx(22, 10));
        setup(&mut sim);
        ftilt(&mut sim);
        sim.ticks(14, inp(0, 0, 0));
        assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    }
}

#[test]
fn the_middle_of_an_air_dodge_is_intangible() {
    let mut sim = Sim::duel(fx(22, 10));
    sim.put_airborne(1, fx(22, 10), Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    sim.tick2(inp(30, 0, ATTACK), inp(0, 0, SHIELD));
    for _ in 0..14 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(
        sim.fighter(1).percent,
        Fx::ZERO,
        "dodging through the move should avoid it"
    );
}

#[test]
fn two_fighters_hitting_each_other_on_the_same_frame_both_take_damage() {
    let mut sim = Sim::duel(fx(14, 10));
    sim.tick2(inp(0, 0, ATTACK), inp(0, 0, ATTACK));
    sim.ticks(8, inp(0, 0, 0));
    assert!(
        sim.fighter(0).percent > Fx::ZERO && sim.fighter(1).percent > Fx::ZERO,
        "a trade"
    );
}

// ---- KO and respawn --------------------------------------------------------------------------------------

#[test]
fn leaving_the_blast_zone_costs_a_stock_and_respawns_with_invulnerability() {
    let mut sim = Sim::new();
    sim.state.fighters[1].percent = Fx::from_int(150);
    sim.state.fighters[1].pos.x = Fx::from_int(40);
    sim.state.fighters[1].platform = sim_core::state::NONE;
    sim.tick(inp(0, 0, 0));
    let f = sim.fighter(1);
    assert_eq!(f.stocks, 2);
    assert_eq!(f.percent, Fx::ZERO);
    assert_eq!(f.pos, sim.content.stage.spawns[1]);
    assert_eq!(f.invuln, sim.content.rules.respawn_invuln);
}

// ---- Tech ------------------------------------------------------------------------------------------------

fn land_in_hitstun(press_shield: bool) -> u8 {
    let mut sim = Sim::new();
    sim.stand(0, fx(-80, 10), 1);
    let f = &mut sim.state.fighters[1];
    f.state = S::Hitstun;
    f.hitstun = 60;
    f.pos = Vec2::new(Fx::ZERO, fx(5, 10));
    f.vel = Vec2::new(Fx::ZERO, fx(-2, 10));
    f.platform = sim_core::state::NONE;
    let shield = if press_shield { SHIELD } else { 0 };
    sim.tick2(inp(0, 0, 0), inp(0, 0, shield));
    for _ in 0..6 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.fighter(1).state == S::Landing {
            return sim.fighter(1).lag;
        }
    }
    panic!("never landed");
}

#[test]
fn a_shield_press_just_before_landing_in_hitstun_is_a_tech() {
    let rules = Sim::new().content.rules;
    assert_eq!(land_in_hitstun(true), rules.tech_lag);
    assert_eq!(land_in_hitstun(false), rules.knockdown_lag);
    assert!(rules.tech_lag < rules.knockdown_lag);
}

// ---- Aerial landing lag ----------------------------------------------------------------------------------

fn nair_landing_lag(start_height: Fx) -> u8 {
    let mut sim = Sim::duel(fx(100, 10));
    sim.put_airborne(0, Fx::ZERO, start_height, Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            return sim.f().lag;
        }
    }
    panic!("never landed");
}

#[test]
fn landing_in_the_middle_of_an_aerial_costs_its_landing_lag_but_early_landings_autocancel() {
    let sim = Sim::new();
    let nair = &sim.content.weapons[0].moves[NAIR as usize];
    let normal = sim.content.fighters[0].landing_lag;
    assert_eq!(nair_landing_lag(fx(2, 1)), nair.landing_lag);
    assert_eq!(
        nair_landing_lag(fx(5, 100)),
        normal,
        "landing on frame 4 or earlier autocancels"
    );
    assert!(nair.landing_lag > normal);
}

// ---- Which move comes out --------------------------------------------------------------------------------

fn move_for_ground(held_frames: usize, stick: (i8, i8)) -> u8 {
    let mut sim = Sim::duel(fx(100, 10));
    sim.ticks(held_frames, inp(stick.0, stick.1, 0));
    sim.tick(inp(stick.0, stick.1, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    sim.f().move_id
}

#[test]
fn flicking_with_attack_is_a_smash_and_holding_the_stick_first_is_a_tilt() {
    assert_eq!(move_for_ground(0, (0, 0)), JAB);
    assert_eq!(move_for_ground(0, (127, 0)), FSMASH);
    assert_eq!(move_for_ground(0, (0, 127)), USMASH);
    assert_eq!(move_for_ground(0, (0, -127)), DSMASH);
    assert_eq!(move_for_ground(8, (60, 0)), FTILT);
    assert_eq!(move_for_ground(8, (0, 127)), UTILT);
    assert_eq!(move_for_ground(8, (0, -127)), DTILT);
}

#[test]
fn attacking_out_of_a_dash_is_a_dash_attack() {
    let mut sim = Sim::duel(fx(100, 10));
    sim.ticks(3, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
    sim.tick(inp(127, 0, ATTACK));
    assert_eq!(sim.f().move_id, DASH_ATTACK);
}

#[test]
fn aerials_follow_the_stick_relative_to_facing() {
    let air = |stick: (i8, i8)| {
        let mut sim = Sim::duel(fx(100, 10));
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(stick.0, stick.1, ATTACK));
        assert_eq!(sim.f().state, S::Attack);
        sim.f().move_id
    };
    assert_eq!(air((0, 0)), NAIR);
    assert_eq!(air((127, 0)), FAIR, "facing right, stick right");
    assert_eq!(air((-127, 0)), BAIR);
    assert_eq!(air((0, 127)), UAIR);
    assert_eq!(air((0, -127)), DAIR);
}

#[test]
fn an_attack_ends_and_returns_to_idle() {
    let mut sim = Sim::duel(fx(100, 10));
    sim.tick(inp(0, 0, ATTACK));
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}
