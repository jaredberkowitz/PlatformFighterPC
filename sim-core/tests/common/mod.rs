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
        Sim::with_chars([0, 1, 0, 1])
    }

    pub fn with_chars(chars: [u8; MAX_FIGHTERS]) -> Sim {
        let mut content = Content::placeholder();
        // Tall blast zone so tests can hold fighters high in the air without being KO.d.
        content.stage.blast_top = Fx::from_int(5000);
        // Moves deal their listed damage in tests: stale-move negation (and its fresh bonus) is tested on its own, with it on.
        content.rules.stale_moves = 0;
        let state = GameState::new(&content, 1, chars);
        Sim { content, state }
    }

    pub fn tick(&mut self, p0: Input) {
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = p0;
        step(&mut self.state, &self.content, &inputs);
    }

    /// Advances one frame with explicit input for players 0 and 1.
    pub fn tick2(&mut self, p0: Input, p1: Input) {
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = p0;
        inputs[1] = p1;
        step(&mut self.state, &self.content, &inputs);
    }

    pub fn fighter(&self, i: usize) -> &Fighter {
        &self.state.fighters[i]
    }

    /// Stands fighter `i` on the main stage at `x`, facing `facing`, idle.
    pub fn stand(&mut self, i: usize, x: Fx, facing: i8) {
        let f = &mut self.state.fighters[i];
        f.pos.x = x;
        f.pos.y = Fx::ZERO;
        f.vel = sim_core::Vec2::ZERO;
        f.platform = 0;
        f.state = S::Idle;
        f.state_frame = 0;
        f.facing = facing;
    }

    /// Two fighters facing each other, `gap` apart, player 0 on the left.
    pub fn duel(gap: Fx) -> Sim {
        let mut sim = Sim::new();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, gap, -1);
        sim
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
