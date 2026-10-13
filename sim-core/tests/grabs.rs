//! Grabs: catching, holding, pummels, throws and breaking free.
//!
//! Reference numbers (ssbwiki.com): a held fighter breaks free after `90 + 1.7 * percent` frames (at least 19), each
//! button press takes 14.4 frames off and each stick flick 8; a released fighter cannot be grabbed again for a while;
//! grabs ignore shields. The brawler's standing grab hits on frame 7 and dash grab on frame 8, pummel does 1.3%, and the
//! throws release on frames 11 / 24 / 27 / 26 (forward / back / up / down).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD};
use sim_core::moves::MoveId;
use sim_core::state::{FighterState as S, NONE};
use sim_core::{step, Fx, Input, MAX_FIGHTERS};

const BRAWLER: [u8; 4] = [1, 1, 1, 1];

fn park(sim: &mut Sim) {
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
}

/// Player 0 at x = -7 facing right, player 1 `gap` in front of it, everyone standing.
fn duel(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(BRAWLER);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-7) + gap, -1);
    park(&mut sim);
    sim
}

/// Presses grab on tick 1 and runs until player 0 is holding player 1. Returns the tick it connected.
fn grab_now(sim: &mut Sim) -> usize {
    sim.tick(inp(0, 0, GRAB));
    for t in 2..=40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbing {
            return t;
        }
    }
    panic!("the grab never connected");
}

fn pct(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

// ---- Catching --------------------------------------------------------------------------------------------------

#[test]
fn a_standing_grab_catches_on_frame_6() {
    let mut sim = duel(fx(15, 10));
    assert_eq!(grab_now(&mut sim), 6);
    assert_eq!(sim.fighter(1).state, S::Grabbed);
    assert_eq!(sim.f().grab_with, 1);
    assert_eq!(sim.fighter(1).grab_with, 0);
    assert_eq!(sim.f().move_id, MoveId::Grab as u8);
}

#[test]
fn a_grab_that_misses_returns_control_on_its_first_actionable_frame() {
    let mut sim = duel(fx(100, 10));
    sim.tick(inp(0, 0, GRAB));
    for t in 2..36 {
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().state, S::Attack, "tick {t}");
    }
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Idle,
        "no longer attacking on tick 36, so acting on 37"
    );
}

#[test]
fn a_dash_grab_catches_on_frame_8_and_keeps_moving() {
    let mut sim = Sim::with_chars(BRAWLER);
    sim.stand(0, Fx::from_int(-9), 1);
    sim.stand(1, Fx::from_int(-3), -1);
    park(&mut sim);
    sim.ticks(14, inp(127, 0, 0));
    assert!(matches!(sim.f().state, S::Run | S::Dash));
    sim.tick(inp(127, 0, GRAB));
    assert_eq!(sim.f().move_id, MoveId::DashGrab as u8);
    let mut caught = None;
    for t in 2..=30 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbing {
            caught = Some(t);
            break;
        }
    }
    assert_eq!(caught, Some(8));
}

#[test]
fn grabs_beat_shields() {
    let mut sim = duel(fx(15, 10));
    sim.state.fighters[1].shield_stun = 0;
    sim.tick2(inp(0, 0, GRAB), inp(0, 0, SHIELD));
    for _ in 2..=7 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    assert_eq!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn attack_out_of_a_shield_is_a_shield_grab() {
    let mut sim = duel(fx(15, 10));
    sim.ticks(8, inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    sim.tick(inp(0, 0, SHIELD | ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Grab as u8);
    for _ in 0..8 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Grabbing);
}

#[test]
fn only_standing_fighters_can_be_caught() {
    // In the air.
    let mut sim = duel(fx(15, 10));
    sim.put_airborne(
        1,
        Fx::from_int(-7) + fx(15, 10),
        fx(30, 10),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.tick(inp(0, 0, GRAB));
    sim.ticks(10, inp(0, 0, 0));
    assert_ne!(sim.f().state, S::Grabbing);
    // In hitstun.
    let mut sim = duel(fx(15, 10));
    sim.state.fighters[1].state = S::Hitstun;
    sim.state.fighters[1].hitstun = 30;
    sim.tick(inp(0, 0, GRAB));
    sim.ticks(10, inp(0, 0, 0));
    assert_ne!(sim.f().state, S::Grabbing);
}

#[test]
fn a_fighter_that_was_just_released_cannot_be_grabbed_straight_away() {
    let mut sim = duel(fx(15, 10));
    sim.state.fighters[1].grab_immune = 30;
    sim.tick(inp(0, 0, GRAB));
    sim.ticks(10, inp(0, 0, 0));
    assert_ne!(sim.f().state, S::Grabbing);
}

// ---- Holding ---------------------------------------------------------------------------------------------------

#[test]
fn the_held_fighter_stands_in_front_facing_the_holder_and_cannot_move() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    let pos = sim.fighter(1).pos;
    assert_eq!(
        pos.x,
        sim.f().pos.x + sim.content.rules.grab_distance,
        "held in front"
    );
    assert_eq!(sim.fighter(1).facing, -sim.f().facing);
    for _ in 0..20 {
        sim.tick2(inp(0, 0, 0), inp(127, 0, 0));
    }
    assert_eq!(sim.fighter(1).pos, pos);
    assert_eq!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn the_time_held_is_90_frames_plus_1_7_per_percent() {
    let held_for = |percent: i32| {
        let mut sim = duel(fx(15, 10));
        sim.state.fighters[1].percent = Fx::from_int(percent);
        grab_now(&mut sim);
        sim.fighter(1).grab_timer
    };
    // One frame of the clock has already run on the tick it connects.
    assert_eq!(held_for(0), 89);
    assert_eq!(held_for(100), 259);
    assert_eq!(held_for(200), 429);
}

#[test]
fn an_unmashed_grab_runs_out_by_itself() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    let mut freed_at = None;
    for t in 0..200 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state != S::Grabbed {
            freed_at = Some(t);
            break;
        }
    }
    let t = freed_at.expect("it should break free");
    assert!((85..=92).contains(&t), "held for {t} more frames");
    assert_eq!(sim.fighter(1).grab_with, NONE);
    assert_eq!(sim.f().grab_with, NONE);
}

#[test]
fn mashing_buttons_breaks_free_much_sooner() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    let mut freed_at = None;
    for t in 0..200 {
        let b = if t % 2 == 0 { JUMP } else { 0 };
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
        if sim.fighter(1).state != S::Grabbed {
            freed_at = Some(t);
            break;
        }
    }
    let t = freed_at.expect("it should break free");
    assert!(t < 30, "took {t} frames of mashing");
}

#[test]
fn breaking_free_leaves_the_holder_in_a_short_release_and_the_victim_immune_for_a_while() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    sim.state.fighters[1].grab_timer = 1;
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Idle);
    assert_eq!(
        sim.f().state,
        S::Landing,
        "the holder is stuck in the release"
    );
    assert_eq!(sim.f().lag, sim.content.rules.grab_release_lag);
    assert!(sim.fighter(1).grab_immune > 40);
    // Grabbing again right away does nothing.
    sim.ticks(40, inp(0, 0, 0));
    sim.tick(inp(0, 0, GRAB));
    sim.ticks(12, inp(0, 0, 0));
    assert_ne!(sim.f().state, S::Grabbing);
}

// ---- Pummel ----------------------------------------------------------------------------------------------------

#[test]
fn attack_pummels_the_held_fighter_for_1_3_percent_on_frame_2_and_keeps_holding() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::Pummel as u8);
    let mut hit = None;
    for t in 2..=20 {
        sim.tick(inp(0, 0, 0));
        if hit.is_none() && sim.fighter(1).percent > before {
            hit = Some(t);
        }
    }
    assert_eq!(hit, Some(2));
    assert_eq!(sim.fighter(1).percent - before, pct(&sim, 13));
    sim.ticks(10, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Grabbing, "back to holding");
    assert_eq!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn pummels_can_be_repeated() {
    let mut sim = duel(fx(15, 10));
    sim.state.fighters[1].percent = Fx::from_int(100); // held long enough for three
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    for _ in 0..3 {
        sim.tick(inp(0, 0, ATTACK));
        sim.ticks(30, inp(0, 0, 0));
    }
    assert!(sim.fighter(1).percent - before >= pct(&sim, 38));
}

// ---- Throws ----------------------------------------------------------------------------------------------------

/// Throws with `stick` and returns (tick the victim was hit, the victim's launch velocity, the sim just after).
fn throw(stick: (i8, i8), percent: i32) -> (usize, sim_core::Vec2, Sim) {
    let mut sim = duel(fx(15, 10));
    sim.state.fighters[1].percent = Fx::from_int(percent);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(stick.0, stick.1, 0));
    let mut hit = None;
    let mut vel = sim_core::Vec2::ZERO;
    for t in 2..=60 {
        sim.tick(inp(0, 0, 0));
        if hit.is_none() && sim.fighter(1).percent > before {
            hit = Some(t);
        }
        if hit.is_some() && sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            vel = sim.fighter(1).kb_vel;
            break;
        }
    }
    (hit.expect("the throw never hit"), vel, sim)
}

#[test]
fn the_throws_land_on_their_frames_for_their_damage() {
    // (Forward, back and up throws are a hit and then the throw, one frame apart, as in the reference: the first damage is the hit's.
    // The damage in all is the two together.)
    for (stick, release, tenths) in [
        ((100, 0), 10, 90),
        ((-100, 0), 23, 110),
        ((0, 100), 26, 70),
        ((0, -100), 26, 85),
    ] {
        let (tick, _, sim) = throw(stick, 0);
        assert_eq!(tick, release, "stick {stick:?}");
        let expected = pct(&sim, tenths);
        assert_eq!(sim.fighter(1).percent, expected, "stick {stick:?}");
    }
}

#[test]
fn each_throw_sends_the_victim_its_own_way() {
    let (_, forward, _) = throw((100, 0), 50);
    assert!(forward.x > Fx::ZERO && forward.y > Fx::ZERO, "{forward:?}");
    let (_, back, _) = throw((-100, 0), 50);
    assert!(back.x < Fx::ZERO && back.y > Fx::ZERO, "{back:?}");
    let (_, up, _) = throw((0, 100), 50);
    assert!(up.y > up.x.abs() * Fx::from_int(3), "{up:?}");
    let (_, down, _) = throw((0, -100), 50);
    assert!(down.x > Fx::ZERO, "{down:?}");
}

#[test]
fn a_throw_lets_go_and_the_thrower_recovers_on_its_first_actionable_frame() {
    let (_, _, mut sim) = throw((100, 0), 0);
    assert_eq!(sim.f().grab_with, NONE);
    assert_eq!(sim.fighter(1).grab_with, NONE);
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn the_forward_throw_returns_control_on_frame_33_plus_the_hitlag_of_its_hit_and_throw() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    sim.tick(inp(100, 0, 0)); // throw tick 1
    let mut free_at = None;
    for t in 2..=80 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            free_at = Some(t + 1);
            break;
        }
    }
    // The reference's first actionable frame leaves out hitlag: floor(5 * 0.65 + 6) = 9 frames for the 5% hit, then
    // floor(4 * 0.65 + 6) = 8 for the 4% throw.
    assert_eq!(free_at, Some(33 + 9 + 8));
}

// ---- Interruptions ----------------------------------------------------------------------------------------------

fn step_all(sim: &mut Sim, inputs: [Input; MAX_FIGHTERS]) {
    step(&mut sim.state, &sim.content, &inputs);
}

#[test]
fn hitting_the_holder_frees_the_held_fighter() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    // Player 2 steps up behind the holder and forward-tilts it.
    sim.state.fighters[2].invuln = 0;
    sim.stand(2, Fx::from_int(-7) - fx(12, 10), 1);
    let mut inputs = [Input::default(); MAX_FIGHTERS];
    inputs[2] = inp(30, 0, ATTACK);
    step_all(&mut sim, inputs);
    for _ in 0..30 {
        step_all(&mut sim, [Input::default(); MAX_FIGHTERS]);
    }
    assert_eq!(sim.fighter(1).grab_with, NONE);
    assert_ne!(sim.fighter(1).state, S::Grabbed);
    assert_eq!(sim.f().grab_with, NONE);
    assert_ne!(sim.f().state, S::Grabbing);
    assert!(sim.f().percent > Fx::ZERO, "the holder was hit");
}

#[test]
fn a_holder_that_is_knocked_out_frees_the_held_fighter() {
    let mut sim = duel(fx(15, 10));
    grab_now(&mut sim);
    sim.state.fighters[0].pos.y = Fx::from_int(-100); // below the blast zone
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.fighter(1).grab_with, NONE);
    assert_ne!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn the_held_fighter_stays_on_the_platform_when_held_near_an_edge() {
    let mut sim = Sim::with_chars(BRAWLER);
    sim.stand(0, fx(97, 10), 1); // near the right edge of the stage
    sim.stand(1, fx(111, 10), -1);
    park(&mut sim);
    sim.tick(inp(0, 0, GRAB));
    sim.ticks(10, inp(0, 0, 0));
    if sim.f().state == S::Grabbing {
        assert!(sim.fighter(1).pos.x <= Fx::from_int(11));
        assert_eq!(sim.fighter(1).pos.y, Fx::ZERO);
    }
}

// ---- Command grabs (E6) ------------------------------------------------------------------------------------------

/// Player 0 (brawler) with a grab of its own as its neutral special: a grab box on frames 2-5 one unit ahead (`command` or not) that
/// goes on with a catch move (ext9: the held fighter pinned 1.0 ahead and 1.0 up, a throw on frame 10, 30 frames), player 1 one unit
/// ahead, then `place` applied.
fn command_grab(command: bool, place: &dyn Fn(&mut Sim)) -> Sim {
    use sim_core::moves::{Move, HIT_GRAB, HIT_THROW};
    let mut sim = duel(Fx::ONE);
    let w = usize::from(sim.content.fighters[1].weapon);
    let mut grab = sim.content.weapons[w].moves[MoveId::Grab as usize].hitboxes[0];
    grab.start = 2;
    grab.end = 5;
    grab.x = Fx::ONE;
    grab.y = Fx::ONE;
    grab.radius = fx(8, 10);
    grab.kind = HIT_GRAB;
    grab.targets = sim_core::moves::TARGETS_ALL;
    grab.command = command;
    let mut throw = grab;
    throw.kind = HIT_THROW;
    throw.start = 10;
    throw.end = 10;
    throw.command = false;
    throw.damage = Fx::from_int(5);
    throw.angle = 45;
    throw.base_knockback = 60;
    throw.knockback_growth = 100;
    sim.content.weapons[w].moves[MoveId::Ext9 as usize] = Move {
        total_frames: 30,
        hitboxes: vec![throw],
        hold_at: Some((Fx::ONE, Fx::ONE)),
        ..Move::empty()
    };
    sim.content.weapons[w].moves[MoveId::NSpecial as usize] = Move {
        total_frames: 30,
        hitboxes: vec![grab],
        catch_move: Some(MoveId::Ext9 as u8),
        ..Move::empty()
    };
    place(&mut sim);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NSpecial);
    sim
}

/// A command-grab case: its name, where player 1 is put, and player 1's buttons.
type Case<'a> = (&'a str, &'a dyn Fn(&mut Sim), u16);

/// Runs the command grab's first two frames, keeping player 1 still (`p1` its input).
fn reach(sim: &mut Sim, p1: Input) {
    for _ in 0..2 {
        sim.state.fighters[1].vel = sim_core::Vec2::ZERO;
        sim.tick2(inp(0, 0, 0), p1);
    }
}

#[test]
fn a_command_grab_catches_airborne_shielding_and_hitstunned_fighters() {
    let airborne = |sim: &mut Sim| {
        let x = sim.fighter(1).pos.x;
        sim.put_airborne(1, x, Fx::ONE, Fx::ZERO, Fx::ZERO);
    };
    let shielding = |sim: &mut Sim| {
        sim.state.fighters[1].state = S::Shield;
        sim.state.fighters[1].state_frame = 20;
    };
    let stunned = |sim: &mut Sim| {
        let x = sim.fighter(1).pos.x;
        sim.put_airborne(1, x, Fx::ONE, Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].state = S::Hitstun;
        sim.state.fighters[1].hitstun = 30;
    };
    let cases: [Case; 3] = [
        ("airborne", &airborne, 0),
        ("shielding", &shielding, SHIELD),
        ("in hitstun", &stunned, 0),
    ];
    for (name, place, p1) in cases {
        let mut sim = command_grab(true, place);
        reach(&mut sim, inp(0, 0, p1));
        assert_eq!(sim.fighter(1).state, S::Grabbed, "{name}");
        assert_eq!((sim.f().state, sim.f().grab_with), (S::Attack, 1), "{name}");
        // Next frame the holder is in its catch move, not the ordinary hold.
        sim.tick(inp(0, 0, 0));
        assert_eq!(
            (sim.f().state, sim.f().move_id),
            (S::Attack, MoveId::Ext9 as u8),
            "{name}"
        );
    }
    // An ordinary grab box misses the same airborne fighter.
    let mut sim = command_grab(false, &airborne);
    reach(&mut sim, inp(0, 0, 0));
    assert_ne!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn a_command_grab_holds_its_catch_in_the_air_without_a_mash_out_and_throws_it() {
    // The holder in the air: it stays in the air, the held fighter pinned 1.0 ahead and 1.0 up of its feet.
    let mut sim = command_grab(true, &|sim: &mut Sim| {
        sim.put_airborne(0, Fx::from_int(-7), Fx::from_int(10), Fx::ZERO, Fx::ZERO);
        sim.put_airborne(1, Fx::from_int(-6), Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    });
    reach(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Grabbed);
    for n in 0..8 {
        // The held fighter mashes every button: no effect during a catch move.
        let mash = if n % 2 == 0 { ATTACK | JUMP | GRAB } else { 0 };
        sim.tick2(inp(0, 0, 0), inp(127, 0, mash));
        let (h, v) = (sim.fighter(0), sim.fighter(1));
        assert!(!h.grounded() && !v.grounded());
        assert_eq!(v.state, S::Grabbed);
        assert_eq!(v.pos, h.pos + sim_core::Vec2::new(Fx::ONE, Fx::ONE));
    }
    // The throw (frame 10 of the catch) lets go and launches.
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).grab_with, NONE);
    assert_eq!(sim.fighter(1).state, S::Hitstun);
    assert_eq!(sim.fighter(1).percent, pct(&sim, 50));
}
