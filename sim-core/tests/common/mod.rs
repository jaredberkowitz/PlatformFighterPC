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

    /// Player 0 where it spawns with nobody in its way: player 1 at the far right end of the stage, the others out of the match
    /// (bodies on the ground do not pass through each other).
    pub fn alone() -> Sim {
        let mut sim = Sim::new();
        sim.stand(1, Fx::from_int(10), -1);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim
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

    /// Two fighters facing each other, `gap` apart, player 0 on the left. The duelist's jab is `plain_jab`'s.
    pub fn duel(gap: Fx) -> Sim {
        let mut sim = Sim::new();
        sim.plain_jab();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, gap, -1);
        sim
    }

    /// Gives the duelist (class 0) the plain jab the engine tests were written against: the sword kit's two hits on 13.0.1's rules,
    /// as the Longsword had before sim 138 (its jab is now the previous version's, `longsword_jab.rs`). Jab 1: frames 5-6, 5% at the
    /// tip and 3% on the blade, angle 361, BK 25, KS 15, FAF 26, chaining into jab 2 at its end; jab 2: frames 4-5, 6% / 4%, 361,
    /// BK 40, KS 30, FAF 29. The engine tests (hitlag, staling, rage, crouch, trades...) use it as a generic weak hit.
    pub fn plain_jab(&mut self) -> &mut Sim {
        use sim_core::moves::{MoveId, ANGLE_BY_BOX, RULES_CURRENT};
        let w = usize::from(self.content.fighters[0].weapon);
        let moves = &mut self.content.weapons[w].moves;
        let template = moves[MoveId::Jab as usize].hitboxes[0];
        // The kit's rows in world tenths at full reach, then the longsword's shorter blade (toward 0.4 / 1.1 by 0.72).
        let (px, py, k) = (
            Fx::from_ratio(2, 5),
            Fx::from_ratio(11, 10),
            Fx::from_ratio(72, 100),
        );
        let hb = |f: u8, x: i32, y: i32, r: i32, dmg: i32, bk: i16, ks: i16, prio: u8| {
            let mut h = template;
            h.start = f - 1;
            h.end = f - 1;
            h.x = px + (Fx::from_ratio(x, 10) - px) * k;
            h.y = py + (Fx::from_ratio(y, 10) - py) * k;
            h.radius = Fx::from_ratio(r, 10);
            h.damage = Fx::from_ratio(dmg, 10);
            h.angle = 361;
            h.base_knockback = bk;
            h.knockback_growth = ks;
            // The tip last (`tipper_last`): the body is 0, the tip 2.
            h.priority = if prio == 0 { 2 } else { prio - 1 };
            h.hitlag = 100;
            h.rules = RULES_CURRENT;
            h.angle_type = ANGLE_BY_BOX;
            h
        };
        let jab1 = &mut moves[MoveId::Jab as usize];
        jab1.hitboxes = vec![
            hb(5, 27, 26, 7, 50, 25, 15, 0),
            hb(6, 32, 13, 7, 50, 25, 15, 0),
            hb(5, 16, 19, 8, 30, 25, 15, 1),
            hb(6, 19, 12, 8, 30, 25, 15, 1),
        ];
        jab1.total_frames = 24;
        jab1.next = Some(MoveId::Jab2 as u8);
        jab1.next_window = 10;
        jab1.next_from = 255;
        jab1.cancel_from = 255;
        let jab2 = &mut moves[MoveId::Jab2 as usize];
        jab2.hitboxes = vec![
            hb(4, 32, 6, 7, 60, 40, 30, 0),
            hb(5, 29, 22, 7, 60, 40, 30, 0),
            hb(4, 19, 9, 8, 40, 40, 30, 1),
            hb(5, 18, 17, 8, 40, 40, 30, 1),
        ];
        jab2.total_frames = 27;
        jab2.next = None;
        jab2.next_from = 255;
        jab2.cancel_from = 255;
        self
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
