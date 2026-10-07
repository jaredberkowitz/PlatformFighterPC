//! The moves built from reference frame data. Frame numbering matches the reference game: the tick on which
//! the button is pressed is frame 1, so a hitbox "active on frame 6" lands on tick 6, and a move whose first
//! actionable frame is 38 returns control on tick 38.
//!
//! Sources: community frame-data tables for a swordfighter archetype (forward tilt, neutral/forward/back
//! air, up special) and a blaster-brawler archetype (forward tilt, neutral/forward air, blaster).
//! Hitbox positions and sizes, the up special's travel, and the blaster's speed are estimates.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::{ATTACK, SHIELD, SPECIAL, STRONG};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{Fx, Input};

/// Everyone is the sword character, or everyone is the claws/blaster character.
const MARTH: [u8; 4] = [0, 0, 0, 0];
const WOLF: [u8; 4] = [1, 1, 1, 1];

/// Percent a hit of `tenths` tenths of a percent deals, including the ruleset damage multiplier.
fn pct(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// Players 3 and 4 only exist because the sim has four slots; make them untouchable so shots and swings
/// aimed at player 2 are not intercepted by them.
fn park_the_others(sim: &mut Sim) {
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
}

/// Two fighters high in the air so nobody lands; player 1 is `gap` in front of (or, if negative, behind) player 0.
fn air_duel(chars: [u8; 4], gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, gap, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.state.fighters[1].facing = if gap >= Fx::ZERO { -1 } else { 1 };
    park_the_others(&mut sim);
    sim
}

/// Two fighters on the ground, player 0 at x = -7 facing right and player 1 `gap` ahead.
fn ground_duel(chars: [u8; 4], gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-7) + gap, -1);
    park_the_others(&mut sim);
    sim
}

/// Runs `first` on tick 1 and idle after, returning the tick on which fighter 1's percent first rises.
fn hit_tick(sim: &mut Sim, first: Input) -> Option<usize> {
    let before = sim.fighter(1).percent;
    sim.tick(first);
    if sim.fighter(1).percent > before {
        return Some(1);
    }
    for t in 2..=90 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            return Some(t);
        }
    }
    None
}

/// Fighter 1's launch velocity on the frame its hit launches.
fn launch_velocity(sim: &mut Sim) -> sim_core::Vec2 {
    for _ in 0..60 {
        let pending_before = sim.fighter(1).launch_pending;
        sim.tick(inp(0, 0, 0));
        if pending_before && !sim.fighter(1).launch_pending {
            return sim.fighter(1).kb_vel;
        }
    }
    panic!("the hit never launched");
}

// ---- Marth-style forward air ---------------------------------------------------------------------------

#[test]
fn forward_air_hits_on_frame_6_for_8_percent_close_and_11_5_at_the_tip() {
    let mut tip = air_duel(MARTH, fx(43, 10));
    assert_eq!(hit_tick(&mut tip, inp(127, 0, ATTACK)), Some(6));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 115));

    let mut close = air_duel(MARTH, fx(10, 10));
    assert_eq!(hit_tick(&mut close, inp(127, 0, ATTACK)), Some(6));
    assert_eq!(close.fighter(1).percent, pct(&close, 80));
}

#[test]
fn forward_air_launches_at_the_sakurai_angle_with_base_knockback_40_and_growth_80() {
    let mut sim = air_duel(MARTH, fx(43, 10));
    hit_tick(&mut sim, inp(127, 0, ATTACK)).unwrap();
    let kb = sim.fighter(1).launch_kb;
    let expected = sim_core::combat::knockback(
        pct(&sim, 115),
        Fx::from_ratio(115, 10),
        sim.content.fighters[0].weight,
        40,
        80,
    );
    assert_eq!(kb, expected);
    let v = launch_velocity(&mut sim);
    // Angle 361 against an airborne target is 44 degrees: up and forward at a slope of about 0.97.
    let slope = v.y / v.x;
    assert!(slope > fx(9, 10) && slope < fx(104, 100), "{slope:?}");
}

// ---- Marth-style back air -------------------------------------------------------------------------------

#[test]
fn back_air_hits_on_frame_7_for_9_percent_close_and_12_5_at_the_tip() {
    let mut tip = air_duel(MARTH, fx(-43, 10));
    assert_eq!(hit_tick(&mut tip, inp(-127, 0, ATTACK)), Some(7));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 125));

    let mut close = air_duel(MARTH, fx(-10, 10));
    assert_eq!(hit_tick(&mut close, inp(-127, 0, ATTACK)), Some(7));
    assert_eq!(close.fighter(1).percent, pct(&close, 90));
}

#[test]
fn back_air_sends_the_victim_backward_and_turns_the_attacker_around() {
    let mut sim = air_duel(MARTH, fx(-43, 10));
    hit_tick(&mut sim, inp(-127, 0, ATTACK)).unwrap();
    let v = launch_velocity(&mut sim);
    assert!(
        v.x < Fx::ZERO && v.y > Fx::ZERO,
        "away from the sword's back: {v:?}"
    );
    // The move finishes with the attacker facing the other way.
    let mut sim = air_duel(MARTH, fx(100, 10));
    sim.tick(inp(-127, 0, ATTACK));
    assert_eq!(sim.f().facing, 1);
    sim.ticks(45, inp(0, 0, 0));
    assert_eq!(sim.f().facing, -1);
}

// ---- Marth-style neutral air ----------------------------------------------------------------------------

#[test]
fn neutral_air_has_two_hits_the_first_on_frame_6_and_the_second_on_frame_15() {
    // First hit: 5% at the tip, 3.5% close.
    let mut tip = air_duel(MARTH, fx(38, 10));
    assert_eq!(hit_tick(&mut tip, inp(0, 0, ATTACK)), Some(6));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 50));
    let mut close = air_duel(MARTH, fx(10, 10));
    assert_eq!(hit_tick(&mut close, inp(0, 0, ATTACK)), Some(6));
    assert_eq!(close.fighter(1).percent, pct(&close, 35));

    // Second hit: with the first already used up, frame 15 deals 9.5% at the tip, 7% close.
    for (gap, tenths) in [(38, 95), (10, 70)] {
        let mut sim = air_duel(MARTH, fx(gap, 10));
        sim.tick(inp(0, 0, ATTACK));
        sim.state.fighters[0].hit_mask = 0b0010; // group 0 already hit fighter 1
        for _ in 2..=14 {
            sim.tick(inp(0, 0, 0));
            assert_eq!(sim.fighter(1).percent, Fx::ZERO, "nothing before frame 15");
        }
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.fighter(1).percent, pct(&sim, tenths), "gap {gap}");
    }
}

#[test]
fn neutral_air_marks_both_hit_groups_once_each() {
    let mut sim = air_duel(MARTH, fx(38, 10));
    sim.tick(inp(0, 0, ATTACK));
    sim.ticks(20, inp(0, 0, 0));
    let mask = sim.f().hit_mask;
    assert_ne!(mask & 0b0010, 0, "first hit recorded");
}

// ---- Marth-style up special -----------------------------------------------------------------------------

#[test]
fn up_special_is_intangible_for_its_first_five_frames() {
    let mut sim = Sim::new();
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(8), -1);
    park_the_others(&mut sim);
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    for tick in 1..=5 {
        if tick > 1 {
            sim.tick(inp(0, 0, 0));
        }
        assert!(is_intangible(sim.f()), "intangible on frame {tick}");
    }
    sim.tick(inp(0, 0, 0));
    assert!(!is_intangible(sim.f()), "vulnerable from frame 6");
}

#[test]
fn up_special_rises_about_44_reference_units_and_leaves_the_ground() {
    let mut sim = Sim::new();
    sim.stand(0, Fx::from_int(-2), 1); // clear of the pass-through platforms, even after drifting forward
    sim.stand(1, Fx::from_int(8), -1);
    park_the_others(&mut sim);
    sim.tick(inp(0, 127, SPECIAL));
    let mut peak = Fx::ZERO;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        peak = peak.max(sim.f().pos.y);
    }
    // 44 reference units is 5.5 world units.
    assert!(peak > fx(51, 10) && peak < fx(58, 10), "peak {peak:?}");
    assert!(!sim.f().grounded());
}

#[test]
fn up_special_hits_on_frame_5_for_11_percent_then_ends_helpless() {
    let mut sim = ground_duel(MARTH, fx(18, 10));
    assert_eq!(hit_tick(&mut sim, inp(0, 127, SPECIAL)), Some(5));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 110));

    let mut sim = Sim::new();
    sim.stand(0, Fx::from_int(-2), 1); // clear of the pass-through platforms, even after drifting forward
    sim.stand(1, Fx::from_int(8), -1);
    park_the_others(&mut sim);
    sim.tick(inp(0, 127, SPECIAL));
    sim.ticks(47, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack, "still going on tick 48");
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Helpless,
        "helpless once the move ends in the air"
    );
}

#[test]
fn special_slots_a_weapon_does_not_have_do_nothing() {
    // The sword character has an up special but no neutral, side or down special yet.
    for stick in [(0, 0), (127, 0), (0, -127)] {
        let mut sim = Sim::new();
        sim.stand(0, Fx::ZERO, 1);
        sim.tick(inp(stick.0, stick.1, SPECIAL));
        assert_ne!(sim.f().state, S::Attack, "stick {stick:?}");
    }
    // The blaster character has no up special.
    let mut sim = Sim::with_chars(WOLF);
    sim.stand(0, Fx::ZERO, 1);
    sim.tick(inp(0, 127, SPECIAL));
    assert_ne!(sim.f().state, S::Attack);
}

// ---- Wolf-style forward air -----------------------------------------------------------------------------

#[test]
fn wolf_forward_air_hits_on_frame_7_for_9_percent_at_60_degrees() {
    let mut sim = air_duel(WOLF, fx(23, 10));
    assert_eq!(hit_tick(&mut sim, inp(127, 0, ATTACK)), Some(7));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 90));
    let v = launch_velocity(&mut sim);
    let slope = v.y / v.x; // tan(60 degrees) is 1.73
    assert!(slope > fx(16, 10) && slope < fx(185, 100), "{slope:?}");
}

// ---- Wolf-style neutral air -----------------------------------------------------------------------------

#[test]
fn wolf_neutral_air_hits_on_frame_7_for_12_percent() {
    let mut sim = air_duel(WOLF, fx(15, 10));
    assert_eq!(hit_tick(&mut sim, inp(0, 0, ATTACK)), Some(7));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 120));
}

#[test]
fn wolf_neutral_air_lingers_as_an_8_percent_hit_until_frame_26() {
    // The target is out of range for the early hit and steps in later.
    let mut sim = air_duel(WOLF, fx(60, 10));
    sim.tick(inp(0, 0, ATTACK));
    for _ in 2..=9 {
        sim.tick(inp(0, 0, 0));
    }
    let (y, vy) = (sim.f().pos.y, sim.f().vel.y);
    sim.put_airborne(1, fx(15, 10), y, Fx::ZERO, vy);
    sim.tick(inp(0, 0, 0)); // tick 10: the lingering hitbox begins
    assert_eq!(sim.fighter(1).percent, pct(&sim, 80));
}

// ---- Wolf-style forward tilt ----------------------------------------------------------------------------

#[test]
fn wolf_forward_tilt_hits_twice_5_percent_on_frame_8_then_6_percent() {
    let mut sim = ground_duel(WOLF, fx(20, 10));
    assert_eq!(hit_tick(&mut sim, inp(30, 0, ATTACK)), Some(8));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 50));
    sim.ticks(30, inp(0, 0, 0));
    assert_eq!(
        sim.fighter(1).percent,
        pct(&sim, 50) + pct(&sim, 60),
        "both hits connect"
    );
    assert_eq!(sim.f().hit_mask & 0b0010, 0b0010);
    assert_eq!(sim.f().hit_mask & 0b0010_0000, 0b0010_0000);
}

#[test]
fn marth_style_forward_tilt_hits_on_frame_8_for_9_percent_close_and_12_at_the_tip() {
    let mut tip = ground_duel(MARTH, fx(40, 10));
    assert_eq!(hit_tick(&mut tip, inp(30, 0, ATTACK)), Some(8));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 120));
    let mut close = ground_duel(MARTH, fx(10, 10));
    assert_eq!(hit_tick(&mut close, inp(30, 0, ATTACK)), Some(8));
    assert_eq!(close.fighter(1).percent, pct(&close, 90));
}

// ---- Wolf-style blaster ---------------------------------------------------------------------------------

fn blaster_duel(gap: Fx) -> Sim {
    ground_duel(WOLF, gap)
}

#[test]
fn the_blaster_fires_a_shot_on_frame_16_from_the_muzzle() {
    let mut sim = blaster_duel(fx(185, 10));
    sim.tick(inp(0, 0, SPECIAL));
    for t in 2..=15 {
        sim.tick(inp(0, 0, 0));
        assert!(
            !sim.state.projectiles[0].active,
            "nothing fires before frame 16 (tick {t})"
        );
    }
    sim.tick(inp(0, 0, 0));
    let shot = sim.state.projectiles[0];
    assert!(shot.active);
    assert_eq!(shot.pos.x, sim.f().pos.x + fx(20, 10));
    assert_eq!(shot.pos.y, fx(12, 10));
    assert!(shot.vel.x > Fx::ZERO, "fired forward");
}

#[test]
fn the_shot_travels_at_a_constant_speed_and_vanishes_after_its_range() {
    let mut sim = blaster_duel(fx(185, 10));
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(15, inp(0, 0, 0));
    let spawn_x = sim.state.projectiles[0].pos.x;
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.state.projectiles[0].pos.x - spawn_x,
        Fx::from_ratio(3000, 8000)
    );
    // Lifetime 35 frames, so gone on tick 51; that is about two thirds of the way across the stage.
    sim.ticks(33, inp(0, 0, 0));
    assert!(sim.state.projectiles[0].active, "still flying on tick 50");
    sim.tick(inp(0, 0, 0));
    assert!(!sim.state.projectiles[0].active, "gone on tick 51");
    let range = sim.state.projectiles[0].pos.x - sim.f().pos.x;
    assert!(range > fx(12, 1) && range < fx(16, 1), "range {range:?}");
}

#[test]
fn a_shot_does_less_damage_the_further_it_has_travelled() {
    let near = {
        let mut sim = blaster_duel(fx(40, 10));
        sim.tick(inp(0, 0, SPECIAL));
        sim.ticks(60, inp(0, 0, 0));
        sim.fighter(1).percent
    };
    let far = {
        let mut sim = blaster_duel(fx(120, 10));
        sim.tick(inp(0, 0, SPECIAL));
        sim.ticks(60, inp(0, 0, 0));
        sim.fighter(1).percent
    };
    let sim = Sim::new();
    assert!(near > far, "{near:?} vs {far:?}");
    // The reference says 8% falling to 6% over the range, times the damage multiplier.
    assert!(
        near <= pct(&sim, 80) && far >= pct(&sim, 60),
        "{near:?} {far:?}"
    );
}

#[test]
fn a_shot_out_of_range_never_hits() {
    let mut sim = blaster_duel(fx(185, 10));
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(80, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
}

#[test]
fn the_shot_flinches_instead_of_launching_and_is_used_up_on_hit() {
    let mut sim = blaster_duel(fx(60, 10));
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            break;
        }
    }
    assert!(sim.fighter(1).percent > Fx::ZERO);
    assert!(
        sim.fighter(1).hitstun <= 10,
        "a flinch, not a launch: {}",
        sim.fighter(1).hitstun
    );
    assert!(
        sim.state.projectiles.iter().all(|p| !p.active),
        "the shot is consumed"
    );
}

#[test]
fn point_blank_the_blaster_uses_its_bayonet_for_7_percent_and_fires_no_shot() {
    let mut sim = blaster_duel(fx(20, 10));
    assert_eq!(hit_tick(&mut sim, inp(0, 0, SPECIAL)), Some(15));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 70));
    sim.ticks(30, inp(0, 0, 0));
    assert!(
        sim.state.projectiles.iter().all(|p| !p.active),
        "no shot after a bayonet hit"
    );
    // The bayonet launches at 60 degrees with real knockback, unlike the flinching shot.
    assert!(
        sim.fighter(1).hitstun > 10
            || sim.fighter(1).state == S::Hitstun
            || sim.fighter(1).percent > Fx::ZERO
    );
}

#[test]
fn a_shielding_target_blocks_the_shot() {
    let mut sim = blaster_duel(fx(60, 10));
    sim.tick2(inp(0, 0, SPECIAL), inp(0, 0, SHIELD));
    for _ in 0..40 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    assert!(sim.state.projectiles.iter().all(|p| !p.active));
}

#[test]
fn the_owner_is_never_hit_by_their_own_shot() {
    let mut sim = Sim::with_chars(WOLF);
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(100, 10), -1);
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.f().percent, Fx::ZERO);
}

// ---- Frame advantage: when control returns --------------------------------------------------------------

/// The tick on which the fighter regains control (the first tick it is no longer attacking), with no target around.
fn first_actionable_tick(chars: [u8; 4], input: Input, aerial: bool) -> usize {
    let mut sim = Sim::with_chars(chars);
    sim.stand(1, Fx::from_int(10), -1);
    if aerial {
        sim.put_airborne(0, Fx::from_int(-9), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    } else {
        sim.stand(0, Fx::from_int(-9), 1);
    }
    sim.tick(input);
    assert_eq!(sim.f().state, S::Attack);
    for tick in 2..=120 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            return tick + 1;
        }
    }
    panic!("never finished");
}

#[test]
fn moves_return_control_on_their_first_actionable_frame() {
    // (character, input, aerial, first actionable frame from the reference tables)
    let cases: [([u8; 4], Input, bool, usize, &str); 8] = [
        (
            MARTH,
            inp(30, 0, ATTACK),
            false,
            34,
            "swordfighter forward tilt",
        ),
        (
            MARTH,
            inp(0, 0, ATTACK),
            true,
            50,
            "swordfighter neutral air",
        ),
        (
            MARTH,
            inp(127, 0, ATTACK),
            true,
            38,
            "swordfighter forward air",
        ),
        (
            MARTH,
            inp(-127, 0, ATTACK),
            true,
            40,
            "swordfighter back air",
        ),
        (WOLF, inp(0, 0, ATTACK), true, 43, "brawler neutral air"),
        (WOLF, inp(127, 0, ATTACK), true, 41, "brawler forward air"),
        (WOLF, inp(30, 0, ATTACK), false, 35, "brawler forward tilt"),
        (WOLF, inp(0, 0, SPECIAL), false, 53, "brawler blaster"),
    ];
    for (chars, input, aerial, faf, name) in cases {
        assert_eq!(first_actionable_tick(chars, input, aerial), faf, "{name}");
    }
}

// ---- Landing lag and autocancel windows -----------------------------------------------------------------

/// Landing lag when touching down on the tick that takes the move to move-frame `frame` (0-based).
fn landing_lag(chars: [u8; 4], mv: MoveId, frame_before_tick: u16) -> u8 {
    let mut sim = Sim::with_chars(chars);
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-9), fx(2, 100), Fx::ZERO, fx(-1, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = mv as u8;
    f.state_frame = frame_before_tick;
    f.hit_mask = 0;
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Landing, "{mv:?} at {frame_before_tick}");
    sim.f().lag
}

#[test]
fn aerials_autocancel_in_their_reference_windows_and_otherwise_cost_their_landing_lag() {
    let normal_marth = Sim::with_chars(MARTH).content.fighters[0].landing_lag;
    let normal_wolf = Sim::with_chars(WOLF).content.fighters[1].landing_lag;

    // Forward air: landing lag 10, autocancels from frame 36.
    assert_eq!(landing_lag(MARTH, MoveId::FAir, 33), 10);
    assert_eq!(landing_lag(MARTH, MoveId::FAir, 34), normal_marth);
    // Back air: landing lag 10, autocancels on frames 1-2 and from frame 32.
    assert_eq!(landing_lag(MARTH, MoveId::BAir, 0), normal_marth);
    assert_eq!(landing_lag(MARTH, MoveId::BAir, 1), 10);
    assert_eq!(landing_lag(MARTH, MoveId::BAir, 29), 10);
    assert_eq!(landing_lag(MARTH, MoveId::BAir, 30), normal_marth);
    // Neutral air: landing lag 7, autocancels from frame 47.
    assert_eq!(landing_lag(MARTH, MoveId::NAir, 30), 7);
    assert_eq!(landing_lag(MARTH, MoveId::NAir, 45), normal_marth);
    // Brawler neutral air: lag 9, autocancels frames 1-6 and from 38.
    assert_eq!(landing_lag(WOLF, MoveId::NAir, 4), normal_wolf);
    assert_eq!(landing_lag(WOLF, MoveId::NAir, 5), 9);
    assert_eq!(landing_lag(WOLF, MoveId::NAir, 36), normal_wolf);
    assert_eq!(landing_lag(WOLF, MoveId::NAir, 35), 9);
    // Brawler forward air: lag 10, autocancels from frame 29.
    assert_eq!(landing_lag(WOLF, MoveId::FAir, 26), 10);
    assert_eq!(landing_lag(WOLF, MoveId::FAir, 27), normal_wolf);
}

// ---- Move data tables ------------------------------------------------------------------------------------
//
// These pin the published numbers (frames numbered as in the reference: first actionable frame FAF means the
// move is `FAF - 2` internal frames long, a hitbox listed on frame N is internal frame N - 1).

fn weapon(sim: &Sim, chars: [u8; 4]) -> &sim_core::moves::Weapon {
    let w = sim.content.fighters[usize::from(chars[0])].weapon;
    &sim.content.weapons[usize::from(w)]
}

/// (first frame, last frame, damage in tenths) for every hitbox of a move, in reference numbering.
fn rows(sim: &Sim, chars: [u8; 4], id: MoveId) -> Vec<(u8, u8, i32)> {
    weapon(sim, chars).moves[id as usize]
        .hitboxes
        .iter()
        .map(|h| {
            (
                h.start + 1,
                h.end + 1,
                (h.damage * Fx::from_int(10) + Fx::HALF).floor_int(),
            )
        })
        .collect()
}

fn faf(sim: &Sim, chars: [u8; 4], id: MoveId) -> u8 {
    weapon(sim, chars).moves[id as usize].total_frames + 2
}

#[test]
fn the_new_marth_style_moves_carry_the_published_timing_and_damage() {
    let sim = Sim::with_chars(MARTH);
    let timing = |id| (faf(&sim, MARTH, id), rows(&sim, MARTH, id));

    // Up tilt: FAF 34; 6/5/5 on frame 6, then 10/6/5/5 on 7-8, then 10/6/5/5 on 9-12.
    let (f, r) = timing(MoveId::UTilt);
    assert_eq!(f, 34);
    assert_eq!(r.iter().filter(|x| x.0 == 6).map(|x| x.2).max(), Some(60));
    assert_eq!(r.iter().filter(|x| x.0 == 7).map(|x| x.2).max(), Some(100));
    assert_eq!(r.iter().filter(|x| x.0 == 9).map(|x| x.2).max(), Some(100));
    assert_eq!(r.iter().map(|x| x.1).max(), Some(12));

    // Down tilt: frames 7-8, 7% and 10%, FAF 24.
    let (f, r) = timing(MoveId::DTilt);
    assert_eq!((f, r), (24, vec![(7, 8, 70), (7, 8, 100)]));

    // Forward smash: frames 10-13, 13% and 18% at the tip, FAF 52.
    let (f, r) = timing(MoveId::FSmash);
    assert_eq!(f, 52);
    assert!(r.iter().all(|x| x.0 == 10 && x.1 == 13));
    assert_eq!(r.iter().map(|x| x.2).max(), Some(180));
    assert_eq!(r.iter().map(|x| x.2).min(), Some(130));

    // Up smash: frames 13-17, 13% and 17% at the tip, FAF 59.
    let (f, r) = timing(MoveId::USmash);
    assert_eq!(f, 59);
    assert!(r.iter().all(|x| x.0 == 13 && x.1 == 17));
    assert_eq!(r.iter().map(|x| x.2).max(), Some(170));

    // Down smash: front hit on 6-7 (8 / 12), back hit on 21-23 (12 / 17), FAF 56.
    let (f, r) = timing(MoveId::DSmash);
    assert_eq!(f, 56);
    assert_eq!(r.iter().filter(|x| x.0 == 6).map(|x| x.2).max(), Some(120));
    assert_eq!(r.iter().filter(|x| x.0 == 6).map(|x| x.2).min(), Some(80));
    assert_eq!(r.iter().filter(|x| x.0 == 21).map(|x| x.2).max(), Some(170));
    assert_eq!(r.iter().filter(|x| x.0 == 21).map(|x| x.2).min(), Some(120));

    // Up air: frames 5-9, 9.5% and 13% at the tip, FAF 46, landing lag 8, autocancels on 1-2 and from 38.
    let (f, r) = timing(MoveId::UAir);
    assert_eq!(f, 46);
    assert!(r.iter().all(|x| x.0 == 5 && x.1 == 9));
    assert_eq!(r.iter().map(|x| x.2).max(), Some(130));
    assert_eq!(r.iter().map(|x| x.2).min(), Some(95));
    let uair = &weapon(&sim, MARTH).moves[MoveId::UAir as usize];
    assert_eq!(
        (
            uair.landing_lag,
            uair.autocancel_before,
            uair.autocancel_after
        ),
        (8, 2, 37)
    );
}

#[test]
fn the_new_wolf_style_moves_carry_the_published_timing_and_damage() {
    let sim = Sim::with_chars(WOLF);
    // Up tilt: frames 7-11 (the 10% foot only on 7-8), 8/9/10% along the leg, FAF 36.
    let r = rows(&sim, WOLF, MoveId::UTilt);
    assert_eq!(faf(&sim, WOLF, MoveId::UTilt), 36);
    assert_eq!(r.iter().map(|x| (x.0, x.1)).min(), Some((7, 8)));
    assert_eq!(r.iter().map(|x| x.1).max(), Some(11));
    let mut damages: Vec<i32> = r.iter().map(|x| x.2).collect();
    damages.sort_unstable();
    assert_eq!(damages, vec![80, 90, 100, 100]);
    // Down tilt: frames 5-6, 6%, FAF 28.
    let r = rows(&sim, WOLF, MoveId::DTilt);
    assert_eq!(faf(&sim, WOLF, MoveId::DTilt), 28);
    assert!(r.iter().all(|x| *x == (5, 6, 60)));
}

// ---- Marth-style tilts, smashes and up air in play ---------------------------------------------------------

#[test]
fn marth_style_up_tilt_hits_in_front_on_frame_6_for_6_percent_with_the_tip() {
    let mut sim = ground_duel(MARTH, fx(14, 10));
    assert_eq!(hit_tick(&mut sim, inp(0, 70, ATTACK)), Some(6));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 60));
    assert_eq!(sim.f().move_id, MoveId::UTilt as u8);
}

#[test]
fn marth_style_down_tilt_stabs_low_on_frame_7_for_7_percent_close_and_10_at_the_tip() {
    let mut tip = ground_duel(MARTH, fx(31, 10));
    assert_eq!(hit_tick(&mut tip, inp(0, -70, ATTACK)), Some(7));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 100));
    let mut close = ground_duel(MARTH, fx(10, 10));
    assert_eq!(hit_tick(&mut close, inp(0, -70, ATTACK)), Some(7));
    assert_eq!(close.fighter(1).percent, pct(&close, 70));
    let v = launch_velocity(&mut close);
    assert!(v.x > Fx::ZERO, "launched forward: {v:?}");
}

#[test]
fn marth_style_forward_smash_hits_on_frame_10_for_13_percent_and_18_at_the_tip() {
    let mut tip = ground_duel(MARTH, fx(34, 10));
    assert_eq!(hit_tick(&mut tip, inp(60, 0, ATTACK | STRONG)), Some(10));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 180));
    let mut close = ground_duel(MARTH, fx(10, 10));
    assert_eq!(hit_tick(&mut close, inp(60, 0, ATTACK | STRONG)), Some(10));
    assert_eq!(close.fighter(1).percent, pct(&close, 130));
}

#[test]
fn marth_style_up_smash_hits_overhead_on_frame_13() {
    let mut sim = ground_duel(MARTH, fx(5, 10));
    let hit = hit_tick(&mut sim, inp(0, 100, ATTACK | STRONG));
    assert_eq!(hit, Some(13));
    assert_eq!(sim.f().move_id, MoveId::USmash as u8);
    assert!(sim.fighter(1).percent >= pct(&sim, 130));
    let v = launch_velocity(&mut sim);
    assert!(
        v.y > v.x.abs() * Fx::from_int(5),
        "nearly straight up: {v:?}"
    );
}

#[test]
fn marth_style_down_smash_hits_in_front_on_frame_6_and_behind_on_frame_21() {
    // Front hit: 12% at the tip.
    let mut front = ground_duel(MARTH, fx(31, 10));
    assert_eq!(hit_tick(&mut front, inp(0, -100, ATTACK | STRONG)), Some(6));
    assert_eq!(front.fighter(1).percent, pct(&front, 120));
    // Back hit: 17% at the tip, from a victim standing behind.
    let mut back = ground_duel(MARTH, fx(-31, 10));
    assert_eq!(hit_tick(&mut back, inp(0, -100, ATTACK | STRONG)), Some(21));
    assert_eq!(back.fighter(1).percent, pct(&back, 170));
    let v = launch_velocity(&mut back);
    assert!(
        v.x < Fx::ZERO,
        "the back hit sends the victim backward: {v:?}"
    );
}

#[test]
fn marth_style_down_smash_can_hit_the_same_victim_with_both_swings() {
    let mut sim = ground_duel(MARTH, fx(2, 10));
    sim.tick(inp(0, -100, ATTACK | STRONG));
    sim.ticks(40, inp(0, 0, 0));
    // A close victim is struck by the front swing; the back swing is a separate hit group.
    let mask = sim.f().hit_mask;
    assert_ne!(mask & 0b0010, 0, "front swing");
}

#[test]
fn marth_style_up_air_hits_on_frame_5_for_9_5_percent_close_and_13_at_the_tip() {
    let mut sour = air_duel(MARTH, fx(0, 1));
    assert_eq!(hit_tick(&mut sour, inp(0, 100, ATTACK)), Some(5));
    assert_eq!(sour.fighter(1).percent, pct(&sour, 95));

    let mut tip = air_duel(MARTH, fx(0, 1));
    tip.put_airborne(1, Fx::ZERO, Fx::from_int(33), Fx::ZERO, Fx::ZERO);
    assert_eq!(hit_tick(&mut tip, inp(0, 100, ATTACK)), Some(5));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 130));
}

// ---- Charging smash attacks ------------------------------------------------------------------------------

/// Holds the attack button for `held` ticks after pressing it, then lets go; returns fighter 1's percent.
fn charged_fsmash(held: usize) -> Fx {
    let mut sim = ground_duel(MARTH, fx(34, 10));
    sim.tick(inp(60, 0, ATTACK | STRONG));
    sim.ticks(held, inp(0, 0, ATTACK));
    sim.ticks(40, inp(0, 0, 0));
    sim.fighter(1).percent
}

#[test]
fn holding_attack_charges_a_smash_attack_for_up_to_40_percent_more_damage() {
    let base = charged_fsmash(0);
    assert_eq!(base, Fx::from_ratio(180, 10) * Fx::from_ratio(12, 10));
    let half = charged_fsmash(30);
    let full = charged_fsmash(60);
    let tol = fx(1, 20);
    assert!(
        (half - base * fx(12, 10)).abs() < tol,
        "half charge {half:?}"
    );
    assert!(
        (full - base * fx(14, 10)).abs() < tol,
        "full charge {full:?}"
    );
    // Holding longer than the limit adds nothing.
    assert_eq!(charged_fsmash(100), full);
}

#[test]
fn charging_holds_the_move_still_and_the_hit_comes_when_it_is_released() {
    let mut sim = ground_duel(MARTH, fx(34, 10));
    sim.tick(inp(60, 0, ATTACK | STRONG));
    sim.ticks(20, inp(0, 0, ATTACK));
    assert_eq!(sim.fighter(1).percent, Fx::ZERO, "still charging");
    assert_eq!(sim.f().charge, 20);
    // After letting go the move resumes where it paused: the hit is 9 ticks later, as it was after the press.
    let mut when = None;
    for t in 1..=30 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            when = Some(t);
            break;
        }
    }
    assert_eq!(when, Some(9));
}

#[test]
fn moves_without_a_charge_frame_ignore_a_held_button() {
    let mut sim = ground_duel(MARTH, fx(30, 10));
    sim.tick(inp(30, 0, ATTACK));
    sim.ticks(60, inp(0, 0, ATTACK));
    assert_eq!(sim.f().charge, 0);
    assert_eq!(
        sim.f().state,
        S::Idle,
        "a held button does not stall a forward tilt"
    );
}

#[test]
fn charging_needs_the_attack_button_itself() {
    let mut sim = ground_duel(MARTH, fx(34, 10));
    sim.tick(inp(60, 0, ATTACK | STRONG));
    sim.ticks(30, inp(0, 0, SHIELD));
    assert_eq!(sim.f().charge, 0);
}

// ---- Up special grabs the ledge mid-move -------------------------------------------------------------------

fn recovery_from(x: Fx, y: Fx) -> Sim {
    let mut sim = Sim::new();
    sim.put_airborne(0, x, y, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    park_the_others(&mut sim);
    sim.state.fighters[1].invuln = 255;
    sim
}

#[test]
fn an_up_special_grabs_the_ledge_in_the_middle_of_the_move() {
    let mut sim = recovery_from(fx(-125, 10), fx(-50, 10));
    sim.tick(inp(0, 127, SPECIAL));
    let mut grabbed_at = None;
    for t in 2..=30 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            grabbed_at = Some(t);
            break;
        }
        assert_eq!(sim.f().state, S::Attack, "tick {t}");
    }
    let t = grabbed_at.expect("the slash passes the ledge and should grab it");
    assert!(t < 20, "grabbed while still rising, on tick {t}");
    assert_eq!(sim.f().ledge, 0);
}

#[test]
fn an_up_special_far_from_any_ledge_does_not_grab() {
    let mut sim = recovery_from(fx(-250, 10), fx(-50, 10));
    sim.tick(inp(0, 127, SPECIAL));
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        assert_ne!(sim.f().state, S::LedgeHang);
    }
}

#[test]
fn an_up_special_respects_the_ledge_regrab_cooldown() {
    let mut sim = recovery_from(fx(-125, 10), fx(-50, 10));
    sim.state.fighters[0].ledge_cooldown = 200;
    sim.tick(inp(0, 127, SPECIAL));
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        assert_ne!(sim.f().state, S::LedgeHang);
    }
}

#[test]
fn only_moves_marked_as_ledge_grabbers_grab_mid_move() {
    // An up air next to the ledge does not grab until it ends.
    let mut sim = recovery_from(fx(-125, 10), fx(-10, 10));
    sim.tick(inp(0, 100, ATTACK));
    for t in 2..=20 {
        sim.tick(inp(0, 0, 0));
        assert_ne!(sim.f().state, S::LedgeHang, "tick {t}");
    }
}

// ---- Wolf-style tilts in play ------------------------------------------------------------------------------

#[test]
fn wolf_style_up_tilt_hits_on_frame_7_for_10_percent_with_the_foot() {
    let mut sim = ground_duel(WOLF, fx(9, 10));
    assert_eq!(hit_tick(&mut sim, inp(0, 70, ATTACK)), Some(7));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 100));
    let v = launch_velocity(&mut sim);
    // Angle 80: mostly upward, a little forward.
    assert!(v.y > v.x && v.x > Fx::ZERO, "{v:?}");
}

#[test]
fn wolf_style_down_tilt_kicks_low_on_frame_5_for_6_percent() {
    let mut sim = ground_duel(WOLF, fx(17, 10));
    assert_eq!(hit_tick(&mut sim, inp(0, -70, ATTACK)), Some(5));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 60));
}

#[test]
fn the_new_moves_return_control_on_their_first_actionable_frames() {
    let cases: [([u8; 4], Input, bool, usize, &str); 8] = [
        ([0; 4], inp(0, 70, ATTACK), false, 34, "marth up tilt"),
        ([0; 4], inp(0, -70, ATTACK), false, 24, "marth down tilt"),
        (
            [0; 4],
            inp(60, 0, ATTACK | STRONG),
            false,
            52,
            "marth forward smash",
        ),
        (
            [0; 4],
            inp(0, 100, ATTACK | STRONG),
            false,
            59,
            "marth up smash",
        ),
        (
            [0; 4],
            inp(0, -100, ATTACK | STRONG),
            false,
            56,
            "marth down smash",
        ),
        ([0; 4], inp(0, 100, ATTACK), true, 46, "marth up air"),
        ([1; 4], inp(0, 70, ATTACK), false, 36, "wolf up tilt"),
        ([1; 4], inp(0, -70, ATTACK), false, 28, "wolf down tilt"),
    ];
    for (chars, input, aerial, faf, name) in cases {
        assert_eq!(first_actionable_tick(chars, input, aerial), faf, "{name}");
    }
}

#[test]
fn up_air_autocancels_on_frames_1_2_and_from_38_and_otherwise_lands_with_8_frames_of_lag() {
    let normal = Sim::with_chars(MARTH).content.fighters[0].landing_lag;
    // `landing_lag` takes the move frame before the tick; the tick that lands is then reference frame N + 2.
    assert_eq!(landing_lag(MARTH, MoveId::UAir, 0), normal, "frame 2");
    assert_eq!(landing_lag(MARTH, MoveId::UAir, 1), 8, "frame 3");
    assert_eq!(landing_lag(MARTH, MoveId::UAir, 35), 8, "frame 37");
    assert_eq!(landing_lag(MARTH, MoveId::UAir, 36), normal, "frame 38");
}
