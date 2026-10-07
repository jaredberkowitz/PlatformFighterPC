//! The simulation's side of the scripting VM: what a script's registers read and what its functions do.
//!
//! A script never touches the game state directly. Everything goes through these two hosts, so the whitelist in
//! `sim_script::api` is the whole surface. Effects that need the rest of the game (firing a projectile, switching
//! move) are recorded on the fighter or returned, and the caller applies them.

use crate::fixed::Fx;
use crate::input::{buttons, Input};
use crate::moves::MoveId;
use crate::state::{Fighter, Projectile};
use crate::trig::{self, Angle};
use crate::vec2::Vec2;
use crate::{FIGHTER_VARS, PROJECTILE_VARS};
use sim_script::api;
use sim_script::{run, Host, Program, ONE};

fn flag(b: bool) -> i32 {
    if b {
        ONE
    } else {
        0
    }
}

/// Angles in scripts are degrees as fixed point; the trig tables use 4096 steps per turn.
fn angle(degrees: i32) -> Angle {
    let steps = (i64::from(degrees) * 4096) / (360i64 << 16);
    Angle::from_raw(steps.rem_euclid(4096) as u16)
}

fn math(id: u8, args: &[i32]) -> i32 {
    let a = angle(args.first().copied().unwrap_or(0));
    match id {
        api::F_SIN => trig::sin(a).raw(),
        _ => trig::cos(a).raw(),
    }
}

/// What a move script asked for that the move code applies afterwards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FighterEffects {
    /// Replaces the move's motion this frame: (forward speed, vertical speed).
    pub motion: Option<(Fx, Fx)>,
    pub end: bool,
    pub stall: bool,
    pub goto: Option<u8>,
}

struct FighterHost<'a> {
    f: &'a mut Fighter,
    effects: FighterEffects,
}

impl FighterHost<'_> {
    fn forward(&self, x: Fx) -> Fx {
        x.mul_int(i32::from(self.f.facing))
    }
}

impl Host for FighterHost<'_> {
    fn register(&self, id: u8) -> i32 {
        let f = &*self.f;
        let input = f.history[0];
        match id {
            api::R_FRAME => Fx::from_int(i32::from(f.state_frame)).raw(),
            api::R_FACING => Fx::from_int(i32::from(f.facing)).raw(),
            api::R_X => f.pos.x.raw(),
            api::R_Y => f.pos.y.raw(),
            api::R_VX => self.forward(f.vel.x).raw(),
            api::R_VY => f.vel.y.raw(),
            api::R_PERCENT => f.percent.raw(),
            api::R_STICK_X => self.forward(Input::axis(input.stick_x)).raw(),
            api::R_STICK_Y => Input::axis(input.stick_y).raw(),
            api::R_GROUNDED => flag(f.grounded()),
            api::R_ATTACK => flag(f.held(buttons::ATTACK)),
            api::R_ATTACK_TAP => flag(f.pressed_within(buttons::ATTACK, 1)),
            api::R_SPECIAL => flag(f.held(buttons::SPECIAL)),
            api::R_SPECIAL_TAP => flag(f.pressed_within(buttons::SPECIAL, 1)),
            api::R_SHIELD => flag(f.held(buttons::SHIELD)),
            api::R_CHARGE => Fx::from_int(i32::from(f.charge)).raw(),
            api::R_HIT => flag(f.hit_mask != 0),
            _ => 0,
        }
    }

    fn var(&self, slot: u8) -> i32 {
        self.f.vars.get(usize::from(slot)).copied().unwrap_or(0)
    }

    fn set_var(&mut self, slot: u8, value: i32) {
        if let Some(v) = self.f.vars.get_mut(usize::from(slot)) {
            *v = value;
        }
    }

    fn call(&mut self, id: u8, args: &[i32]) -> i32 {
        let arg = |n: usize| Fx::from_raw(args.get(n).copied().unwrap_or(0));
        match id {
            api::F_SIN | api::F_COS => return math(id, args),
            api::F_SET_VEL => self.effects.motion = Some((arg(0), arg(1))),
            api::F_ADD_VEL => {
                self.f.vel.x += self.forward(arg(0));
                self.f.vel.y += arg(1);
            }
            api::F_SPAWN => {
                self.f.spawn_request = true;
                self.f.spawn_custom = true;
                self.f.spawn_pos = Vec2::new(self.forward(arg(0)), arg(1));
                self.f.spawn_vel = Vec2::new(self.forward(arg(2)), arg(3));
            }
            api::F_INTANGIBLE => {
                let frames = arg(0).floor_int().clamp(0, 255) as u8;
                self.f.invuln = self.f.invuln.max(frames);
            }
            api::F_END => self.effects.end = true,
            api::F_TURN => self.f.facing = -self.f.facing,
            api::F_REHIT => self.f.hit_mask = 0,
            api::F_STALL => self.effects.stall = true,
            api::F_GOTO => {
                let last = MoveId::COUNT as i32 - 1;
                self.effects.goto = Some(arg(0).floor_int().clamp(0, last) as u8);
            }
            _ => {}
        }
        0
    }
}

/// Runs a fighter's move script for this frame.
pub fn run_fighter(program: &Program, f: &mut Fighter) -> FighterEffects {
    let mut host = FighterHost {
        f,
        effects: FighterEffects::default(),
    };
    run(program, &mut host);
    host.effects
}

/// What a projectile script sees of the world around it.
#[derive(Clone, Copy, Debug)]
pub struct Surroundings {
    pub owner: Vec2,
    /// The nearest fighter that is not the owner (lowest index on a tie).
    pub target: Option<Vec2>,
}

struct ProjectileHost<'a> {
    p: &'a mut Projectile,
    around: Surroundings,
    kill: bool,
}

impl Host for ProjectileHost<'_> {
    fn register(&self, id: u8) -> i32 {
        let p = &*self.p;
        let target = self.around.target.unwrap_or(Vec2::ZERO);
        match id {
            api::R_PX => p.pos.x.raw(),
            api::R_PY => p.pos.y.raw(),
            api::R_PVX => p.vel.x.raw(),
            api::R_PVY => p.vel.y.raw(),
            api::R_AGE => Fx::from_int(i32::from(p.age)).raw(),
            api::R_LIFE => Fx::from_int(i32::from(p.life)).raw(),
            api::R_OWNER_X => self.around.owner.x.raw(),
            api::R_OWNER_Y => self.around.owner.y.raw(),
            api::R_TARGET_X => target.x.raw(),
            api::R_TARGET_Y => target.y.raw(),
            api::R_HAS_TARGET => flag(self.around.target.is_some()),
            _ => 0,
        }
    }

    fn var(&self, slot: u8) -> i32 {
        self.p.vars.get(usize::from(slot)).copied().unwrap_or(0)
    }

    fn set_var(&mut self, slot: u8, value: i32) {
        if let Some(v) = self.p.vars.get_mut(usize::from(slot)) {
            *v = value;
        }
    }

    fn call(&mut self, id: u8, args: &[i32]) -> i32 {
        let arg = |n: usize| Fx::from_raw(args.get(n).copied().unwrap_or(0));
        match id {
            api::F_SIN | api::F_COS => return math(id, args),
            api::F_P_SET_VEL => self.p.vel = Vec2::new(arg(0), arg(1)),
            api::F_P_KILL => self.kill = true,
            _ => {}
        }
        0
    }
}

/// Runs a projectile's script for this frame. Returns true if the script removed the projectile.
pub fn run_projectile(program: &Program, p: &mut Projectile, around: Surroundings) -> bool {
    let mut host = ProjectileHost {
        p,
        around,
        kill: false,
    };
    run(program, &mut host);
    host.kill
}

const _: () = {
    assert!(FIGHTER_VARS == 4 && PROJECTILE_VARS == 2);
};
