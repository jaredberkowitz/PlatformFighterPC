//! The one entry point that advances the simulation by exactly one 60 Hz frame.
//!
//! Phase 0 placeholder behaviour: run, jump and fall on a flat floor, all driven by
//! `FighterParams`. Real movement (jump squat, air dodge, wavedash, ledges) arrives in Phase 1.

use crate::content::{Content, FighterParams};
use crate::fixed::Fx;
use crate::input::{buttons, Input};
use crate::state::{Fighter, GameState};
use crate::MAX_FIGHTERS;

pub fn step(state: &mut GameState, content: &Content, inputs: &[Input; MAX_FIGHTERS]) {
    // Fixed iteration order (player index) keeps simultaneous interactions deterministic.
    for (fighter, input) in state.fighters.iter_mut().zip(inputs.iter()) {
        let idx = usize::from(fighter.char_id).min(content.fighters.len().saturating_sub(1));
        step_fighter(
            fighter,
            &content.fighters[idx],
            content.stage.floor_y,
            *input,
        );
    }
    state.frame = state.frame.wrapping_add(1);
}

fn step_fighter(f: &mut Fighter, params: &FighterParams, floor_y: Fx, input: Input) {
    let stick_x = input.stick_x_fx();
    f.vel.x = stick_x * params.run_speed;
    match stick_x.signum_int() {
        1 => f.facing = 1,
        -1 => f.facing = -1,
        _ => {}
    }

    if f.grounded && input.pressed(buttons::JUMP) {
        f.vel.y = params.jump_velocity;
        f.grounded = false;
    }
    if !f.grounded {
        f.vel.y = (f.vel.y - params.gravity).max(-params.max_fall_speed);
    }

    f.pos += f.vel;

    if f.pos.y <= floor_y && f.vel.y <= Fx::ZERO {
        f.pos.y = floor_y;
        f.vel.y = Fx::ZERO;
        f.grounded = true;
    } else {
        f.grounded = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzz::random_inputs;
    use crate::rng::Rng;

    fn run(content: &Content, seed: u64, inputs: &[[Input; MAX_FIGHTERS]]) -> GameState {
        let mut s = GameState::new(content, seed, [0, 1, 0, 1]);
        for i in inputs {
            step(&mut s, content, i);
        }
        s
    }

    #[test]
    fn same_inputs_same_checksum() {
        let content = Content::placeholder();
        let inputs = random_inputs(&mut Rng::new(1), 600);
        assert_eq!(
            run(&content, 9, &inputs).checksum(),
            run(&content, 9, &inputs).checksum()
        );
    }

    #[test]
    fn different_inputs_diverge() {
        let content = Content::placeholder();
        let a = random_inputs(&mut Rng::new(1), 600);
        let b = random_inputs(&mut Rng::new(2), 600);
        assert_ne!(
            run(&content, 9, &a).checksum(),
            run(&content, 9, &b).checksum()
        );
    }

    #[test]
    fn restore_then_replay_matches_straight_run() {
        let content = Content::placeholder();
        let inputs = random_inputs(&mut Rng::new(3), 400);

        let mut s = GameState::new(&content, 1, [0, 1, 0, 1]);
        for i in &inputs[..150] {
            step(&mut s, &content, i);
        }
        let saved = s;
        for i in &inputs[150..] {
            step(&mut s, &content, i);
        }
        let straight = s.checksum();

        let mut r = saved;
        for i in &inputs[150..] {
            step(&mut r, &content, i);
        }
        assert_eq!(r.checksum(), straight);
    }

    #[test]
    fn fighter_lands_and_stays_on_floor() {
        let content = Content::placeholder();
        let mut s = GameState::new(&content, 1, [0, 1, 0, 1]);
        let jump = Input {
            buttons: buttons::JUMP,
            ..Input::default()
        };
        step(&mut s, &content, &[jump; MAX_FIGHTERS]);
        assert!(!s.fighters[0].grounded);
        assert!(s.fighters[0].pos.y > Fx::ZERO);

        for _ in 0..200 {
            step(&mut s, &content, &[Input::default(); MAX_FIGHTERS]);
        }
        for f in &s.fighters {
            assert!(f.grounded);
            assert_eq!(f.pos.y, content.stage.floor_y);
            assert_eq!(f.vel.y, Fx::ZERO);
        }
    }

    #[test]
    fn different_physics_profiles_behave_differently() {
        let content = Content::placeholder();
        let mut s = GameState::new(&content, 1, [0, 1, 0, 1]);
        let jump = Input {
            buttons: buttons::JUMP,
            ..Input::default()
        };
        let mut peak = [Fx::ZERO; 2];
        step(&mut s, &content, &[jump; MAX_FIGHTERS]);
        for _ in 0..120 {
            for (p, fighter) in peak.iter_mut().zip(s.fighters.iter()) {
                *p = (*p).max(fighter.pos.y);
            }
            step(&mut s, &content, &[Input::default(); MAX_FIGHTERS]);
        }
        assert_ne!(peak[0], peak[1]);
    }
}
