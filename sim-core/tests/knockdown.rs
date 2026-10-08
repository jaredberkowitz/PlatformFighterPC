//! Landing in hitstun: techs (in place or rolling), knockdown and its get-up options, and ledge attacks.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::moves::MoveId;
use sim_core::state::{FighterState as S, NONE};
use sim_core::{Fx, Vec2};

/// Player 1 falls in hitstun toward the main stage; player 0 stands well away.
fn falling_victim() -> Sim {
    let mut sim = Sim::new();
    sim.stand(0, fx(-80, 10), 1);
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    let f = &mut sim.state.fighters[1];
    f.state = S::Hitstun;
    f.hitstun = 80;
    f.pos = Vec2::new(Fx::ZERO, fx(5, 10));
    f.vel = Vec2::new(Fx::ZERO, fx(-2, 10));
    f.platform = NONE;
    sim
}

/// Lands player 1 with `input` held on the landing frames, returns the sim just after it touched down.
fn land_with(input: sim_core::Input) -> Sim {
    let mut sim = falling_victim();
    sim.tick2(inp(0, 0, 0), input);
    for _ in 0..8 {
        sim.tick2(inp(0, 0, 0), input);
        if !matches!(sim.fighter(1).state, S::Hitstun) {
            return sim;
        }
    }
    panic!("never landed");
}

fn knocked_down() -> Sim {
    let mut sim = land_with(inp(0, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Knockdown);
    sim.state.fighters[1].history = [sim_core::Input::default(); sim_core::state::HISTORY_LEN];
    sim
}

// ---- Techs -----------------------------------------------------------------------------------------------------

#[test]
fn a_tech_in_place_is_intangible_and_short() {
    let sim = land_with(inp(0, 0, SHIELD));
    let f = sim.fighter(1);
    assert_eq!(f.state, S::Landing);
    assert_eq!(f.lag, sim.content.rules.tech_lag);
    assert!(is_intangible(f), "intangible while it techs");
}

#[test]
fn a_tech_with_the_stick_flicked_sideways_rolls_that_way() {
    let mut sim = falling_victim();
    // A flick during the tech window: neutral, then sideways, with shield held.
    sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    let mut landed = None;
    for t in 0..8 {
        let x = if t == 0 { 127 } else { 100 };
        sim.tick2(inp(0, 0, 0), inp(x, 0, SHIELD));
        if sim.fighter(1).state != S::Hitstun {
            landed = Some(sim.fighter(1).state);
            break;
        }
    }
    assert_eq!(landed, Some(S::Roll));
    let x0 = sim.fighter(1).pos.x;
    sim.ticks_both(40);
    assert!(sim.fighter(1).pos.x > x0 + fx(2, 1), "rolled right");
    assert_eq!(sim.fighter(1).state, S::Idle);
}

// ---- Knockdown --------------------------------------------------------------------------------------------------

#[test]
fn without_a_tech_the_fighter_lies_down_and_can_be_hit() {
    let sim = knocked_down();
    let f = sim.fighter(1);
    assert_eq!(f.state, S::Knockdown);
    assert!(f.grounded());
    assert!(!is_intangible(f), "a knocked down fighter is vulnerable");
}

#[test]
fn it_cannot_get_up_before_the_minimum_time() {
    let mut sim = knocked_down();
    let lag = usize::from(sim.content.rules.knockdown_lag);
    for _ in 0..lag - 3 {
        sim.tick2(inp(0, 0, 0), inp(0, 100, 0));
        assert_eq!(sim.fighter(1).state, S::Knockdown);
    }
}

#[test]
fn stick_up_or_jump_stands_up_with_a_stretch_of_intangibility() {
    for input in [inp(0, 100, 0), inp(0, 0, JUMP)] {
        let mut sim = knocked_down();
        let lag = usize::from(sim.content.rules.knockdown_lag);
        sim.ticks_both(lag + 1);
        sim.tick2(inp(0, 0, 0), input);
        assert_eq!(sim.fighter(1).state, S::GetUp);
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        assert!(is_intangible(sim.fighter(1)));
        let r = sim.content.rules;
        sim.ticks_both(usize::from(r.getup_frames) + 2);
        assert_eq!(sim.fighter(1).state, S::Idle);
    }
}

#[test]
fn attack_is_a_get_up_attack_that_hits_on_both_sides() {
    let mut sim = knocked_down();
    let lag = usize::from(sim.content.rules.knockdown_lag);
    sim.ticks_both(lag + 1);
    // Player 0 right next to it, in front.
    sim.stand(0, sim.fighter(1).pos.x + fx(12, 10), -1);
    sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
    assert_eq!(sim.fighter(1).state, S::Attack);
    assert_eq!(sim.fighter(1).move_id, MoveId::GetUpAttack as u8);
    assert!(is_intangible(sim.fighter(1)), "intangible as it rises");
    sim.ticks_both(30);
    assert!(sim.fighter(0).percent > Fx::ZERO, "the get-up attack hit");
}

#[test]
fn a_flick_sideways_is_a_get_up_roll() {
    let mut sim = knocked_down();
    let lag = usize::from(sim.content.rules.knockdown_lag);
    sim.ticks_both(lag + 1);
    let x0 = sim.fighter(1).pos.x;
    sim.tick2(inp(0, 0, 0), inp(-127, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Roll);
    sim.ticks_both(40);
    assert!(sim.fighter(1).pos.x < x0 - fx(2, 1));
}

#[test]
fn doing_nothing_gets_up_by_itself() {
    let mut sim = knocked_down();
    let max = usize::from(sim.content.rules.knockdown_max);
    sim.ticks_both(max + 2);
    assert_eq!(sim.fighter(1).state, S::GetUp);
}

// ---- Ledge attack ------------------------------------------------------------------------------------------------

#[test]
fn a_ledge_attack_hits_someone_standing_on_the_stage_next_to_the_ledge() {
    let mut sim = Sim::new();
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    // Player 0 grabs the left ledge; player 1 stands near where the climb ends.
    sim.put_airborne(0, fx(-125, 10), fx(-15, 10), Fx::ZERO, fx(-1, 10));
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::LedgeHang);
    sim.stand(1, fx(-85, 10), -1);
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().state, S::LedgeAttack);
    assert_eq!(sim.f().move_id, MoveId::LedgeAttack as u8);
    sim.ticks(70, inp(0, 0, 0));
    assert!(sim.fighter(1).percent > Fx::ZERO, "the ledge attack hit");
    assert_eq!(sim.f().state, S::Idle);
}

trait Both {
    fn ticks_both(&mut self, n: usize);
}

impl Both for Sim {
    fn ticks_both(&mut self, n: usize) {
        for _ in 0..n {
            self.tick2(inp(0, 0, 0), inp(0, 0, 0));
        }
    }
}
