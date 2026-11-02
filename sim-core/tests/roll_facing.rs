//! Which way a fighter faces after a roll (sim 147, the reference's). A forward roll out of a shield ends turned round, facing
//! the way it came from, so rolling past an opponent leaves the fighter facing it. A back roll keeps its facing. Tech rolls and
//! get-up rolls go through the same state but keep their facing either way, as do a trip's. The turn lands on the frame the roll
//! ends, a stale roll's later end included, so whatever is buffered out of the roll comes out facing the new way.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::{FighterState as S, NONE};
use sim_core::{Fx, Vec2};

/// Fighter `ch` alone on the stage, facing `facing`, raises its shield and flicks the stick `x`: the frame its roll begins.
fn shield_roll(ch: u8, facing: i8, x: i8) -> Sim {
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.stand(1, fx(10, 1), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, facing);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.tick(inp(x, 0, SHIELD));
    assert_eq!(sim.f().state, S::Roll);
    sim
}

/// Plays fighter `i`'s roll out, the stick let go, checking that it faces `facing` on every frame of it. Returns how many frames
/// it took.
fn play_out(sim: &mut Sim, i: usize, facing: i8) -> usize {
    let mut frames = 0;
    while sim.fighter(i).state == S::Roll {
        assert_eq!(sim.fighter(i).facing, facing, "frame {frames} of the roll");
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        frames += 1;
        assert!(frames < 200, "the roll never ended");
    }
    frames
}

/// Player 1 falls tumbling in hitstun facing `facing`, with player 0 well away.
fn falling_victim(facing: i8) -> Sim {
    let mut sim = Sim::new();
    sim.stand(0, fx(-80, 10), 1);
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    let f = &mut sim.state.fighters[1];
    f.state = S::Hitstun;
    f.hitstun = 80;
    f.tumble = true;
    f.pos = Vec2::new(Fx::ZERO, fx(5, 10));
    f.vel = Vec2::new(Fx::ZERO, fx(-2, 10));
    f.platform = NONE;
    f.facing = facing;
    sim
}

/// The falling victim presses shield and flicks `x` as it lands: the frame its tech roll begins.
fn tech_roll(sim: &mut Sim, x: i8) {
    sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    for t in 0..8 {
        let stick = if t == 0 { x } else { x / 5 * 4 };
        sim.tick2(inp(0, 0, 0), inp(stick, 0, SHIELD));
        if sim.fighter(1).state != S::Hitstun {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Roll);
}

/// The falling victim lands with no tech and lies there long enough to get up.
fn knocked_down(facing: i8) -> Sim {
    let mut sim = falling_victim(facing);
    for _ in 0..9 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).state, S::Knockdown);
    sim.state.fighters[1].history = [sim_core::Input::default(); sim_core::state::HISTORY_LEN];
    let lag = usize::from(sim.content.rules.knockdown_lag);
    for _ in 0..lag + 1 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    sim
}

// ---- Rolls out of a shield ------------------------------------------------------------------------------------------

#[test]
fn every_body_ends_a_forward_roll_turned_round_on_the_frame_it_ends() {
    let bodies = Sim::new().content.fighters.len();
    for ch in 0..bodies as u8 {
        for facing in [1i8, -1] {
            let mut sim = shield_roll(ch, facing, 127 * facing);
            let p = sim.content.fighters[usize::from(ch)];
            let frames = play_out(&mut sim, 0, facing);
            assert_eq!(frames, usize::from(p.roll_frames), "body {ch}");
            assert_eq!(sim.f().state, S::Idle, "body {ch}");
            assert_eq!(sim.f().facing, -facing, "body {ch} facing {facing}");
        }
    }
}

#[test]
fn every_body_keeps_its_facing_through_a_back_roll() {
    let bodies = Sim::new().content.fighters.len();
    for ch in 0..bodies as u8 {
        for facing in [1i8, -1] {
            let mut sim = shield_roll(ch, facing, -127 * facing);
            let p = sim.content.fighters[usize::from(ch)];
            let frames = play_out(&mut sim, 0, facing);
            assert_eq!(frames, usize::from(p.roll_back_frames), "body {ch}");
            assert_eq!(sim.f().state, S::Idle, "body {ch}");
            assert_eq!(sim.f().facing, facing, "body {ch} facing {facing}");
        }
    }
}

#[test]
fn rolling_past_an_opponent_leaves_the_fighter_facing_it() {
    let mut sim = Sim::duel(fx(12, 10));
    park(&mut sim);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.tick(inp(127, 0, SHIELD));
    assert_eq!(sim.f().state, S::Roll);
    play_out(&mut sim, 0, 1);
    let past = sim.fighter(1).pos.x - sim.f().pos.x;
    assert!(past < Fx::ZERO, "it rolled past: {past:?}");
    assert_eq!(sim.f().facing, -1, "and faces back at it");
}

#[test]
fn the_next_roll_the_same_way_is_a_back_roll_and_the_way_back_is_forward_again() {
    let mut sim = shield_roll(0, 1, 127);
    let p = sim.content.fighters[0];
    play_out(&mut sim, 0, 1);
    assert_eq!(sim.f().facing, -1);
    // The same way again goes away from the way it now faces: longer, and still facing back.
    sim.ticks(3, inp(0, 0, SHIELD));
    sim.tick(inp(127, 0, SHIELD));
    assert_eq!(sim.f().state, S::Roll);
    // (Staled by the first roll, one level at the back roll's rate.)
    let rate = u16::from(sim.content.rules.dodge_stale_back_roll);
    let back = (u16::from(p.roll_back_frames) * (100 + rate)).div_ceil(100);
    assert_eq!(play_out(&mut sim, 0, -1), usize::from(back));
    assert_eq!(sim.f().facing, -1);
    // Back the other way it is a forward roll again, and turns.
    sim.ticks(3, inp(0, 0, SHIELD));
    sim.tick(inp(-127, 0, SHIELD));
    assert_eq!(sim.f().state, S::Roll);
    play_out(&mut sim, 0, -1);
    assert_eq!(sim.f().facing, 1);
}

// ---- Stale rolls ----------------------------------------------------------------------------------------------------

/// A roll flicked `x` by a fighter facing right, at staleness `level`: its length, with the facing checked on every frame up to the
/// last, and the facing after it.
fn stale_roll(level: u8, x: i8) -> (usize, i8) {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.stand(1, fx(10, 1), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.state.fighters[0].dodge_stale = level;
    sim.tick(inp(x, 0, SHIELD));
    assert_eq!(sim.f().state, S::Roll);
    assert_eq!(sim.f().dodge_level, level);
    let frames = play_out(&mut sim, 0, 1);
    (frames, sim.f().facing)
}

#[test]
fn a_stale_forward_roll_turns_only_when_its_longer_length_ends() {
    let sim = Sim::new();
    let rules = sim.content.rules;
    let fresh = sim.content.fighters[0].roll_frames;
    assert_eq!(stale_roll(0, 127), (usize::from(fresh), -1));
    for level in 1..=rules.dodge_stale_max {
        let percent = 100 + u16::from(rules.dodge_stale_roll) * u16::from(level);
        let stale = (u16::from(fresh) * percent).div_ceil(100);
        assert!(stale > u16::from(fresh), "level {level}");
        // (`play_out` checks it still faces right on every frame before the last, the fresh length's end among them.)
        assert_eq!(
            stale_roll(level, 127),
            (usize::from(stale), -1),
            "level {level}"
        );
    }
}

#[test]
fn a_stale_back_roll_keeps_its_facing_to_its_longer_end() {
    let sim = Sim::new();
    let rules = sim.content.rules;
    let fresh = sim.content.fighters[0].roll_back_frames;
    let percent = 100 + u16::from(rules.dodge_stale_back_roll) * u16::from(rules.dodge_stale_max);
    let stale = (u16::from(fresh) * percent).div_ceil(100);
    assert_eq!(
        stale_roll(rules.dodge_stale_max, -127),
        (usize::from(stale), 1)
    );
}

// ---- Tech and get-up rolls ------------------------------------------------------------------------------------------

#[test]
fn a_tech_roll_keeps_its_facing_either_way() {
    for facing in [1i8, -1] {
        for way in [1i8, -1] {
            let mut sim = falling_victim(facing);
            tech_roll(&mut sim, 127 * way);
            let x0 = sim.fighter(1).pos.x;
            play_out(&mut sim, 1, facing);
            assert_eq!(sim.fighter(1).state, S::Idle);
            assert_eq!(sim.fighter(1).facing, facing, "facing {facing} way {way}");
            assert_eq!(
                (sim.fighter(1).pos.x - x0).signum_int(),
                i32::from(way),
                "it rolled the way it was flicked"
            );
        }
    }
}

#[test]
fn a_get_up_roll_keeps_its_facing_either_way() {
    for facing in [1i8, -1] {
        for way in [1i8, -1] {
            let mut sim = knocked_down(facing);
            sim.tick2(inp(0, 0, 0), inp(127 * way, 0, 0));
            assert_eq!(sim.fighter(1).state, S::Roll);
            let x0 = sim.fighter(1).pos.x;
            play_out(&mut sim, 1, facing);
            assert_eq!(sim.fighter(1).state, S::Idle);
            assert_eq!(sim.fighter(1).facing, facing, "facing {facing} way {way}");
            assert_eq!(
                (sim.fighter(1).pos.x - x0).signum_int(),
                i32::from(way),
                "it rolled the way it was flicked"
            );
        }
    }
}

#[test]
fn a_trips_get_up_roll_keeps_its_facing_either_way() {
    for way in [1i8, -1] {
        let mut sim = knocked_down(1);
        sim.state.fighters[1].tripped = true;
        sim.state.fighters[1].state_frame = 200;
        sim.tick2(inp(0, 0, 0), inp(127 * way, 0, 0));
        assert_eq!(sim.fighter(1).state, S::Roll);
        play_out(&mut sim, 1, 1);
        assert_eq!(sim.fighter(1).facing, 1, "way {way}");
    }
}

#[test]
fn a_forward_roll_cut_short_leaves_no_turn_for_the_next_roll() {
    // A forward roll out of a shield that a hit stopped has not turned, and the mark it left does not reach a tech roll later.
    let mut sim = falling_victim(1);
    sim.state.fighters[1].roll_turns = true;
    tech_roll(&mut sim, 127);
    assert!(!sim.fighter(1).roll_turns);
    play_out(&mut sim, 1, 1);
    assert_eq!(sim.fighter(1).facing, 1);
}

// ---- Out of the roll ------------------------------------------------------------------------------------------------

/// Player 0 rolls forward past player 1 (1.2 apart), presses `press` once a few frames before the roll ends and holds the stick
/// `hold` from then on. Runs to the frame the roll ends.
fn out_of_a_roll_past_an_opponent(press: u16, hold: i8) -> Sim {
    let mut sim = Sim::duel(fx(12, 10));
    park(&mut sim);
    let frames = usize::from(sim.content.fighters[0].roll_frames);
    sim.ticks(6, inp(0, 0, SHIELD));
    sim.tick(inp(127, 0, SHIELD));
    assert_eq!(sim.f().state, S::Roll);
    sim.ticks(frames - 5, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Roll);
    assert_eq!(sim.f().facing, 1, "not turned yet");
    sim.tick(inp(hold, 0, press));
    while sim.f().state == S::Roll {
        sim.tick(inp(hold, 0, 0));
    }
    assert_eq!(sim.f().state, S::Idle);
    assert_eq!(sim.f().facing, -1, "the roll ended turned round");
    sim
}

/// Fighters 2 and 3 out of the match, so that the duel is only the two.
fn park(sim: &mut Sim) {
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
}

#[test]
fn a_grab_buffered_out_of_a_forward_roll_comes_out_the_new_way_and_catches() {
    let mut sim = out_of_a_roll_past_an_opponent(GRAB, 0);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::Grab as u8);
    assert_eq!(sim.f().facing, -1, "toward the fighter it rolled past");
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Grabbed {
            return;
        }
    }
    panic!("the grab should catch the fighter it rolled past");
}

#[test]
fn a_jab_buffered_out_of_a_forward_roll_hits_the_fighter_it_rolled_past() {
    let mut sim = out_of_a_roll_past_an_opponent(ATTACK, 0);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::Jab as u8);
    assert_eq!(sim.f().facing, -1);
    sim.ticks(12, inp(0, 0, 0));
    assert!(sim.fighter(1).percent > Fx::ZERO, "the jab connected");
}

#[test]
fn a_jump_or_special_buffered_out_of_a_forward_roll_starts_facing_the_new_way() {
    for (press, state) in [(JUMP, S::JumpSquat), (SPECIAL, S::Attack)] {
        let mut sim = out_of_a_roll_past_an_opponent(press, 0);
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().state, state);
        assert_eq!(sim.f().facing, -1);
    }
}

#[test]
fn a_tilt_buffered_out_of_a_forward_roll_goes_the_way_the_stick_is_held() {
    let mut sim = out_of_a_roll_past_an_opponent(ATTACK, 60);
    sim.tick(inp(60, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::FTilt as u8);
    assert_eq!(sim.f().facing, 1, "the attack's own turn, as from standing");
}

#[test]
fn a_dash_flicked_just_before_the_roll_ends_goes_the_way_it_was_flicked() {
    for way in [1i8, -1] {
        let mut sim = out_of_a_roll_past_an_opponent(0, 127 * way);
        sim.tick(inp(127 * way, 0, 0));
        assert_eq!(sim.f().state, S::Dash, "way {way}");
        assert_eq!(sim.f().facing, way, "way {way}");
    }
}

#[test]
fn a_stick_still_held_from_the_flick_turns_it_back_round() {
    let mut sim = shield_roll(0, 1, 127);
    while sim.f().state == S::Roll {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(sim.f().facing, -1, "the roll ended turned round");
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Turn);
    assert_eq!(sim.f().facing, 1, "it goes on the way the stick is held");
}
