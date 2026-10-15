//! Footstools (sim 135, the reference's footstool jump): jump with the feet on an opponent's head bounces off it (a full hop's speed,
//! lower each time before landing, no midair jump spent, no damage); a grounded victim is frozen 21 frames, an airborne one pushed
//! down tumbling. Not within 16 frames of an attack.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::JUMP;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// Player 1 standing at x = 0; player 0 falling with its feet a little above player 1's head.
fn over_a_standing_head() -> Sim {
    let mut sim = Sim::new();
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    sim.stand(1, Fx::ZERO, -1);
    let top = sim.fighter(1).pos.y + sim.content.fighters[0].ecb_height;
    sim.put_airborne(0, fx(1, 10), top + fx(2, 10), Fx::ZERO, Fx::ZERO);
    sim
}

#[test]
fn jump_on_a_standing_head_bounces_off_and_freezes_it() {
    let mut sim = over_a_standing_head();
    let jumps = sim.fighter(0).air_jumps_left;
    sim.tick2(inp(0, 0, JUMP), inp(0, 0, 0));
    let a = sim.fighter(0);
    assert!(a.vel.y > Fx::ZERO, "bounced up");
    assert_eq!(a.air_jumps_left, jumps, "no midair jump spent");
    assert!(is_intangible(a), "briefly intangible");
    let v = sim.fighter(1);
    assert_eq!(v.state, S::Rebound, "frozen");
    assert_eq!(v.lag, sim.content.rules.footstool_ground_stun);
    assert_eq!(v.percent, Fx::ZERO, "no damage");
    assert_eq!(v.hit_by, 0, "a fall from it is credited to the footstooler");
}

#[test]
fn a_footstooled_fighter_in_the_air_is_pushed_down_tumbling() {
    let mut sim = over_a_standing_head();
    // Lift player 1 into the air with player 0 still on its head.
    let lift = fx(30, 10);
    let v = &mut sim.state.fighters[1];
    v.pos.y += lift;
    v.platform = sim_core::state::NONE;
    v.state = S::Airborne;
    sim.state.fighters[0].pos.y += lift;
    sim.tick2(inp(0, 0, JUMP), inp(0, 0, 0));
    let v = sim.fighter(1);
    assert_eq!(v.state, S::Hitstun);
    assert!(v.tumble, "tumbling, so it can tech the landing");
    assert!(v.vel.y < Fx::ZERO, "pushed down");
}

#[test]
fn each_footstool_before_landing_rebounds_lower() {
    let mut sim = over_a_standing_head();
    sim.tick2(inp(0, 0, JUMP), inp(0, 0, 0));
    let first = sim.fighter(0).vel.y;
    // The same footstool, as the second one since leaving the ground.
    let mut again = over_a_standing_head();
    again.state.fighters[0].footstools = 1;
    again.tick2(inp(0, 0, JUMP), inp(0, 0, 0));
    let second = again.fighter(0).vel.y;
    assert!(
        second < first && second > Fx::ZERO,
        "{second:?} after {first:?}"
    );
}

#[test]
fn no_footstool_just_after_an_attack_or_away_from_a_head() {
    let mut sim = over_a_standing_head();
    sim.state.fighters[0].footstool_wait = 5;
    let jumps = sim.fighter(0).air_jumps_left;
    sim.tick2(inp(0, 0, JUMP), inp(0, 0, 0));
    assert_eq!(
        sim.fighter(0).air_jumps_left,
        jumps - 1,
        "an ordinary midair jump"
    );
    assert_ne!(sim.fighter(1).state, S::Rebound);

    let mut sim = over_a_standing_head();
    sim.state.fighters[0].pos.x += fx(40, 10);
    let jumps = sim.fighter(0).air_jumps_left;
    sim.tick2(inp(0, 0, JUMP), inp(0, 0, 0));
    assert_eq!(
        sim.fighter(0).air_jumps_left,
        jumps - 1,
        "too far to the side: a midair jump"
    );
    assert_ne!(sim.fighter(1).state, S::Rebound);
}
