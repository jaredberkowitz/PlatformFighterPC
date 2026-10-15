//! The simulation's side of the scripting VM: what a script's registers read and what its functions do.
//!
//! A script never touches the game state directly. Everything goes through these two hosts, so the whitelist in
//! `sim_script::api` is the whole surface. Effects that need the rest of the game (firing a projectile, switching
//! move) are recorded on the fighter or returned, and the caller applies them.

use crate::collision;
use crate::content::{Gauge, Stage};
use crate::fixed::Fx;
use crate::input::{buttons, Input};
use crate::moves::MoveId;
use crate::state::{Fighter, Projectile};
use crate::trig::{self, Angle};
use crate::vec2::Vec2;
use crate::{FIGHTER_VARS, MAX_BODIES, PROJECTILE_VARS};
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

/// What a fighter's move script can see of the others (taken at the start of the frame, before anyone moves): for each fighter
/// its chest if it can be targeted (in the match with stocks left, not intangible), and the height of every chest above the feet.
#[derive(Clone, Copy, Debug)]
pub struct World<'a> {
    pub stage: &'a Stage,
    pub targets: [Option<Vec2>; MAX_BODIES],
    pub chest: [Fx; MAX_BODIES],
    /// Which fighter the script belongs to.
    pub me: usize,
    /// Every projectile as the frame began (the `owned` register counts its fighter's).
    pub projectiles: &'a [Projectile],
    /// For each fighter in an egg (E8), the numbers of the egg (`combat::encase_of`), read as the frame began.
    pub encase: [Option<crate::moves::Encase>; MAX_BODIES],
    /// Every item as the frame began (E4: `holding`, and the items `owned` counts).
    pub items: &'a [crate::state::Item],
    /// For each fighter, whether an item lay in its reach as the frame began (an attack or grab press picks it up).
    pub pickable: [bool; MAX_BODIES],
    /// Who holds each ledge as the frame began (`GameState::ledge_owner`; -1 free): a tether catches only a free one (E56, sim 123).
    pub ledges: [i8; crate::state::LEDGE_SLOTS],
    /// For each fighter that can be footstooled as the frame began (sim 135): the top of its head and its half width.
    pub heads: [Option<(Vec2, Fx)>; MAX_BODIES],
}

/// How many live projectiles fighter `me` fired (reflected or not) of tool kind `kind` (`moves::Tool::kind`), or, with kind 0, of
/// the move `move_id` with no kind: what a projectile limit and the `owned` register count (E3d).
pub fn owned(projectiles: &[Projectile], me: usize, kind: u8, move_id: u8) -> u8 {
    let n = projectiles
        .iter()
        .filter(|p| {
            p.active
                && usize::from(p.origin) == me
                && if kind != 0 {
                    p.kind == kind
                } else {
                    p.kind == 0 && p.move_id == move_id
                }
        })
        .count();
    u8::try_from(n).unwrap_or(u8::MAX)
}

struct FighterHost<'a> {
    f: &'a mut Fighter,
    world: &'a World<'a>,
    /// The body's gauge rules (`boost_add`).
    gauge: &'a Gauge,
    /// The tool kind of the move's projectile (0 for none): what `owned` counts.
    kind: u8,
    effects: FighterEffects,
}

impl FighterHost<'_> {
    fn forward(&self, x: Fx) -> Fx {
        x.mul_int(i32::from(self.f.facing))
    }

    /// The target lock (`lock(range)`): the nearest other fighter (lowest index on a tie) that can be targeted, in front of the
    /// fighter, its chest within `range` of the fighter's chest and no solid platform across the line between the two; its chest is
    /// kept as the lock point. Once locked, the attack keeps that point.
    fn lock(&mut self, range: Fx) -> bool {
        if self.f.locked {
            return true;
        }
        let w = self.world;
        let me = w.me.min(MAX_BODIES - 1);
        let from = Vec2::new(self.f.pos.x, self.f.pos.y + w.chest[me]);
        let mut best: Option<(Fx, Vec2)> = None;
        for (i, t) in w.targets.iter().enumerate() {
            let Some(chest) = *t else { continue };
            if !crate::combat::rivals(me, i) {
                continue;
            }
            let ahead = (chest.x - from.x).mul_int(i32::from(self.f.facing));
            let d = (chest - from).length();
            if ahead <= Fx::ZERO || d > range || collision::segment_blocked(w.stage, from, chest) {
                continue;
            }
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, chest));
            }
        }
        if let Some((_, chest)) = best {
            self.f.lock = chest;
            self.f.locked = true;
        }
        self.f.locked
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
            api::R_STORED => Fx::from_int(i32::from(f.stored)).raw(),
            api::R_JUMP => flag(f.held(buttons::JUMP)),
            api::R_PRESSES => Fx::from_int(i32::from(f.presses)).raw(),
            api::R_RELEASES => Fx::from_int(i32::from(f.releases)).raw(),
            api::R_WALL => flag(f.touched_wall),
            api::R_FLICK => flag(f.special_flick),
            api::R_LOCKED => flag(f.locked),
            api::R_LOCK_X => f.lock.x.raw(),
            api::R_LOCK_Y => f.lock.y.raw(),
            api::R_BOOST => f.gauge.raw(),
            api::R_BOOSTING => flag(f.gauge_mode > 0),
            api::R_GRAB => flag(f.held(buttons::GRAB)),
            api::R_AIM => Fx::from_int(i32::from(f.aim)).raw(),
            api::R_FAST_FALL => flag(f.fast_fall),
            // (Its projectiles, and since sim 111 its items of the move's kind: E4.)
            api::R_OWNED => Fx::from_int(i32::from(
                owned(self.world.projectiles, self.world.me, self.kind, f.move_id).saturating_add(
                    crate::items::owned(self.world.items, self.world.me, self.kind),
                ),
            ))
            .raw(),
            api::R_HOLDING => {
                Fx::from_int(i32::from(crate::items::held_kind(self.world.items, f))).raw()
            }
            api::R_BALLOONS => Fx::from_int(i32::from(f.balloons)).raw(),
            // (E42: 0 nothing, 1 a projectile, 2 an item.)
            api::R_POCKETED => {
                Fx::from_int(match (f.pocket != crate::state::NONE, f.pocket_item) {
                    (false, _) => 0,
                    (true, false) => 1,
                    (true, true) => 2,
                })
                .raw()
            }
            // (E47, sim 117: the wait a move's toss started.)
            api::R_PULL_WAIT => Fx::from_int(i32::from(f.pull_wait)).raw(),
            // (Sim 119: the flip's latch jumps off on a new press.)
            api::R_JUMP_TAP => flag(f.pressed_within(buttons::JUMP, 1)),
            // (Sim 121: the partner's moves, E52, and the guard's release, E55.)
            api::R_POWERED => flag(f.move_powered),
            api::R_WITHSTOOD => Fx::from_int(i32::from(f.withstood)).raw(),
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
            api::F_SPAWN | api::F_SPAWN_AT
                if self.f.spawn_request && self.f.spawn_custom && !self.f.spawn2_request =>
            {
                // A second shot on the same frame (E3h, sim 95): it fills the second request, plain size and knockback.
                self.f.spawn2_request = true;
                self.f.spawn2_fixed = id == api::F_SPAWN_AT;
                self.f.spawn2_pos = if id == api::F_SPAWN_AT {
                    Vec2::new(arg(0), arg(1))
                } else {
                    Vec2::new(self.forward(arg(0)), arg(1))
                };
                self.f.spawn2_vel = Vec2::new(self.forward(arg(2)), arg(3));
            }
            api::F_SPAWN | api::F_SPAWN_SCALED | api::F_SPAWN_AT => {
                self.f.spawn_request = true;
                self.f.spawn_custom = true;
                self.f.spawn_fixed = id == api::F_SPAWN_AT;
                self.f.spawn_pos = if id == api::F_SPAWN_AT {
                    Vec2::new(arg(0), arg(1))
                } else {
                    Vec2::new(self.forward(arg(0)), arg(1))
                };
                self.f.spawn_vel = Vec2::new(self.forward(arg(2)), arg(3));
                if id == api::F_SPAWN_SCALED {
                    self.f.spawn_kb_add = arg(4).floor_int().clamp(0, 255) as u8;
                    self.f.spawn_size = arg(5).floor_int().clamp(10, 400) as u16;
                } else {
                    self.f.spawn_kb_add = 0;
                    self.f.spawn_size = 100;
                }
            }
            api::F_INTANGIBLE => {
                let frames = arg(0).floor_int().clamp(0, 255) as u8;
                self.f.invuln = self.f.invuln.max(frames);
            }
            api::F_END => self.effects.end = true,
            api::F_TURN => self.f.facing = -self.f.facing,
            api::F_REHIT => self.f.hit_mask = 0,
            api::F_STALL => self.effects.stall = true,
            api::F_STORE => {
                self.f.stored = arg(0).floor_int().clamp(0, 255) as u8;
            }
            api::F_AIR_FLAG => {
                let bit = 1u8 << arg(0).floor_int().clamp(0, 7);
                return flag(self.f.air_flags & bit != 0);
            }
            api::F_SET_AIR_FLAG => {
                self.f.air_flags |= 1u8 << arg(0).floor_int().clamp(0, 7);
            }
            // (Sim 119, owner decision 75: marks a hit does not clear.)
            api::F_AIR_MARK => {
                let bit = 1u8 << arg(0).floor_int().clamp(0, 7);
                return flag(self.f.air_marks & bit != 0);
            }
            api::F_SET_AIR_MARK => {
                self.f.air_marks |= 1u8 << arg(0).floor_int().clamp(0, 7);
            }
            api::F_LOCK => return flag(self.lock(arg(0))),
            // (Sim 128: a probe of the stage, the exile's wall stick.)
            api::F_GROUND => {
                let a = self.f.pos + Vec2::new(self.forward(arg(0)), arg(1));
                let b = self.f.pos + Vec2::new(self.forward(arg(2)), arg(3));
                return flag(crate::collision::segment_meets_ground(
                    self.world.stage,
                    a,
                    b,
                ));
            }
            api::F_BOOST_ADD => crate::gauge::add(self.f, self.gauge, arg(0)),
            api::F_SET_CHARGE => self.f.charge = arg(0).floor_int().clamp(0, 255) as u8,
            api::F_DETONATE => self.f.item_order |= crate::items::ORDER_DETONATE,
            api::F_OWNS => {
                let kind = arg(0).floor_int().clamp(0, 255) as u8;
                let n = crate::items::owns(
                    self.world.items,
                    self.world.projectiles,
                    self.world.me,
                    kind,
                );
                return Fx::from_int(i32::from(n)).raw();
            }
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
pub fn run_fighter(
    program: &Program,
    f: &mut Fighter,
    world: &World,
    gauge: &Gauge,
    kind: u8,
) -> FighterEffects {
    let mut host = FighterHost {
        f,
        world,
        gauge,
        kind,
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
    /// The owner's stick this frame (-1..1 each way; x positive right, absolute, not by facing): what `owner_stick_x` /
    /// `owner_stick_y` read (sim 117: the rocket barrels steered for a while after they fly off).
    pub owner_stick: Vec2,
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
            api::R_OWNER_STICK_X => self.around.owner_stick.x.raw(),
            api::R_OWNER_STICK_Y => self.around.owner_stick.y.raw(),
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
