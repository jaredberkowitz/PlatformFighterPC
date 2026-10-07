//! Fighters that are not in the match (the unused slots of a two-player game) must not interfere.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, SPECIAL};
use sim_core::{step, Content, Fx, GameState, Input, MAX_FIGHTERS};

fn two_player_sim(chars: [u8; 4]) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.state = GameState::new_with_active(&sim.content, 1, chars, 0b0011);
    sim
}

#[test]
fn inactive_fighters_cannot_be_hit() {
    // Player 3 stands 3 units in front of player 0, exactly where a swing lands.
    let mut sim = two_player_sim([0, 0, 0, 0]);
    sim.stand(0, Fx::from_int(-4), 1);
    sim.stand(1, Fx::from_int(20), -1);
    sim.stand(2, Fx::from_int(-2), -1);
    for _ in 0..40 {
        sim.tick(inp(0, 0, ATTACK));
    }
    assert_eq!(
        sim.fighter(2).percent,
        Fx::ZERO,
        "the inactive fighter took a hit"
    );
    assert!(!sim.fighter(2).active);
}

#[test]
fn inactive_fighters_do_not_absorb_projectiles() {
    // The brawler's shot flies through the spot where player 3 stands and reaches player 1.
    let mut sim = two_player_sim([1, 1, 1, 1]);
    sim.stand(0, Fx::from_int(-9), 1);
    sim.stand(1, Fx::from_int(4), -1);
    sim.stand(2, Fx::from_int(-3), -1);
    sim.state.fighters[1].invuln = 0;
    let mut reached = false;
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        reached |= sim.fighter(1).percent > Fx::ZERO;
    }
    assert!(reached, "the shot should get past the inactive fighter");
    assert_eq!(sim.fighter(2).percent, Fx::ZERO);
}

#[test]
fn inactive_fighters_are_never_projectile_targets() {
    // The seeking bolt aims at the nearest *active* enemy: player 1, high above, not the ghost at ground level.
    let mut sim = two_player_sim([0, 0, 0, 0]);
    sim.stand(0, Fx::from_int(-9), 1);
    sim.put_airborne(1, Fx::from_int(4), Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].invuln = 255;
    sim.stand(2, Fx::from_int(1), -1);
    sim.tick(inp(0, 0, SPECIAL));
    let mut top = Fx::ZERO;
    for _ in 0..70 {
        sim.tick(inp(0, 0, 0));
        for p in sim.state.projectiles.iter().filter(|p| p.active) {
            top = top.max(p.pos.y);
        }
    }
    assert!(
        top > Fx::from_int(3),
        "the bolt rose toward player 1 (peak height {top:?})"
    );
}

#[test]
fn inactive_fighters_stay_put_whatever_happens() {
    let content = Content::placeholder();
    let mut s = GameState::new_with_active(&content, 3, [0, 1, 0, 1], 0b0011);
    let start = [s.fighters[2].pos, s.fighters[3].pos];
    let inputs = sim_core::fuzz::random_inputs(&mut sim_core::Rng::new(5), 1200);
    for i in &inputs {
        // Even if inputs arrive for the inactive slots, they do nothing.
        step(&mut s, &content, i);
    }
    assert_eq!([s.fighters[2].pos, s.fighters[3].pos], start);
    assert_eq!(s.fighters[2].percent, Fx::ZERO);
    assert_eq!(s.fighters[3].percent, Fx::ZERO);
}

#[test]
fn the_active_mask_is_part_of_the_state() {
    let content = Content::placeholder();
    let all = GameState::new(&content, 1, [0, 1, 0, 1]);
    let two = GameState::new_with_active(&content, 1, [0, 1, 0, 1], 0b0011);
    assert_ne!(all.checksum(), two.checksum());
    let _: [Input; MAX_FIGHTERS] = [Input::default(); MAX_FIGHTERS];
}
