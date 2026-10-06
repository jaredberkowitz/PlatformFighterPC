//! Shared scripting harness for integration tests: drive player 0, inspect the sim.
#![allow(dead_code)]

use sim_core::state::{Fighter, FighterState as S, NONE};
use sim_core::{step, Content, Fx, GameState, Input, MAX_FIGHTERS};

pub fn inp(x: i8, y: i8, buttons: u16) -> Input {
    Input {
        stick_x: x,
        stick_y: y,
        buttons,
    }
}

pub struct Sim {
    pub content: Content,
    pub state: GameState,
}

impl Sim {
    pub fn new() -> Sim {
        let content = Content::placeholder();
        let state = GameState::new(&content, 1, [0, 1, 0, 1]);
        Sim { content, state }
    }

    pub fn tick(&mut self, p0: Input) {
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = p0;
        step(&mut self.state, &self.content, &inputs);
    }

    pub fn ticks(&mut self, n: usize, p0: Input) {
        for _ in 0..n {
            self.tick(p0);
        }
    }

    pub fn f(&self) -> &Fighter {
        &self.state.fighters[0]
    }

    pub fn put_airborne(&mut self, i: usize, x: Fx, y: Fx, vx: Fx, vy: Fx) {
        let f = &mut self.state.fighters[i];
        f.pos.x = x;
        f.pos.y = y;
        f.vel.x = vx;
        f.vel.y = vy;
        f.platform = NONE;
        f.state = S::Airborne;
        f.state_frame = 0;
    }
}

pub fn fx(n: i32, d: i32) -> Fx {
    Fx::from_ratio(n, d)
}
