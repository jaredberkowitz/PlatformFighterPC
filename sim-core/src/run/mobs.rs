//! Grunts: the run's enemies (R2b; rebuilt in R2c, sim 131, `RUNNER_WORLD.md` 4).
//!
//! **A grunt is a fighter.** Grunt `n` is the body `MAX_FIGHTERS + n` of `GameState::fighters` ([`slot`]), a fighter of its kind's
//! body (`MobKind::body`, a roster fighter), stepped by the versus step with every versus rule, both ways: hits, grabs with their hold,
//! pummels, all four throws, grab releases and mashing out; hitstun, tumble, techs, DI and SDI; shields, shield stun, shield breaks
//! and perfect shields; armor, counters, reflectors and absorbers; hitlag, crouch cancelling, ledges, the player's projectiles and
//! items; staleness and rage. Nothing here re-implements any of it. Grunts never hit, grab or reflect one another
//! (`combat::rivals`).
//!
//! What the run adds is a **player for each grunt** ([`think`]): its kind's behaviour presses buttons and moves the stick as a person
//! would, with the inputs the versus game reads. Every attack is announced by a **tell** (it stops, a "!" bobs over it: the tier's
//! `Tier::tell` frames), then the attack's input. By role:
//! * a **brawler** runs at you when far and jabs or forward-tilts up close;
//! * a **leaper** waits on high ground, crouches (its tell), jumps at you and drifts in with its down air (from above) or forward air;
//! * a **shieldbearer** walks in and holds up its shield near you (a versus shield: it shrinks, it breaks), and pokes out of it;
//! * a **spearman** pokes long with its forward tilt, or dashes in with its dash attack from further out;
//! * a **lobber** throws its shell (its body's up special, a real projectile, which the run aims to come down on you) from high
//!   ground;
//! * a **bruiser** walks in and swings its forward smash, with super armor through its tell and the smash's startup.
//!
//! In hitstun a grunt may DI (survival DI) and tech, and in a grab it may mash, at its tier's rates (`Tier::di`, `tech`, `mash`); at
//! tier 1 it does none of it. The run gives each grunt its kind's and tier's weight (`Fighter::weight`: far lighter than any fighter
//! at first, `Tier::weight`) and damage (`Fighter::damage_scale`). A grunt is knocked out **only by the blast zones that follow the
//! view** ([`grunt_zone`], `RunRules::ko_margin` outside it on every side), never by damage; a grunt launched backward goes through
//! the gate behind (which stays a wall for the player) and is knocked out by the zone there.

use super::data::{
    MobKind, RunContent, Tier, NO_MOVE, ROLE_BRUISER, ROLE_LEAPER, ROLE_LOBBER, ROLE_SHIELD,
    ROLE_SPEAR,
};
use super::{
    count_combo, pay_kill, Mob, INTERIOR_X, MAX_MOBS, MODE_ATTACK, MODE_COOLDOWN, MODE_EMERGE,
    MODE_IDLE, MODE_LEAP, MODE_MOVE, MODE_WINDUP, PLAYER, VAN_DRIVING, VAN_HALF, VAN_HEIGHT,
    VAN_NONE, VAN_PARKED, VAN_WAITING,
};
use crate::collision;
use crate::combat::{self, params_of, weapon_of};
use crate::content::Content;
use crate::fixed::Fx;
use crate::input::{buttons, Input, STICK_FLICK_FROM};
use crate::moves::{Hitbox, MoveId, HIT_NORMAL};
use crate::state::{Fighter, FighterState as S, GameState, NONE, REST_FLYING};
use crate::vec2::Vec2;
use crate::{MAX_BODIES, MAX_FIGHTERS};

/// `Mob::step` once the attack's move has come out (the grunt now waits for it to end).
const STARTED: u8 = 255;
/// Frames a grunt keeps trying to make its attack come out before giving up (it was not free to act).
const ATTEMPT_FRAMES: u16 = 30;

/// Grunt `n`'s body in `GameState::fighters`.
pub const fn slot(n: usize) -> usize {
    MAX_FIGHTERS + n
}

pub(super) fn kind_of<'a>(rc: &'a RunContent, m: &Mob) -> &'a MobKind {
    &rc.mobs[usize::from(m.kind).min(rc.mobs.len() - 1)]
}

/// The grunt's tier row (the last one past the end).
pub(super) fn tier_row(rc: &RunContent, tier: u8) -> Tier {
    rc.tiers[usize::from(tier).min(rc.tiers.len() - 1)]
}

/// The weight a grunt of this kind weighs in this tier: the kind's percent of its tier's, measured from -100 (the knockback formula
/// divides by the weight plus 100, so a kind at 150% takes two thirds of the knockback of one at 100% in every tier).
pub fn grunt_weight(rc: &RunContent, kind: u8, tier: u8) -> Fx {
    let k = &rc.mobs[usize::from(kind).min(rc.mobs.len() - 1)];
    let hundred = Fx::from_int(100);
    (tier_row(rc, tier).weight + hundred) * Fx::from_ratio(i32::from(k.weight), 100) - hundred
}

/// The grunts' blast zones (R2c): `(left, right, bottom, top)`, `ko_margin` outside the view on every side. The view is the camera's
/// (`Run::cam_x`, `Run::cam_y`, `view_width` by `view_height`).
pub fn grunt_zone(run: &super::Run, rules: &super::RunRules) -> (Fx, Fx, Fx, Fx) {
    let half = rules.view_height * Fx::HALF;
    (
        run.cam_x - rules.ko_margin,
        run.cam_x + rules.view_width + rules.ko_margin,
        run.cam_y - half - rules.ko_margin,
        run.cam_y + half + rules.ko_margin,
    )
}

/// Move `slot` of a grunt's kit (none for an empty slot).
fn kit_move(k: &MobKind, slot: u8) -> Option<MoveId> {
    let id = *k.moves.get(usize::from(slot))?;
    (id != NO_MOVE).then(|| MoveId::from_index(id))
}

/// The first active frame of a grunt's move (its startup ends there).
fn first_active(content: &Content, f: &Fighter, id: MoveId) -> u16 {
    let mv = weapon_of(content, params_of(content, f)).get(id as u8);
    u16::from(mv.hitboxes.iter().map(|h| h.start).min().unwrap_or(0))
}

/// A number in `0..100`, the same for every frame of the `every`-frame window it falls in, for this grunt and `salt` (as the
/// computer opponent's `cpu::roll`: a pure function of the frame, so it replays exactly and draws nothing from the run's generator).
fn roll(frame: u32, n: usize, every: u32, salt: u32) -> u32 {
    let mut x = (frame / every.max(1)).wrapping_mul(0x9E37_79B9)
        ^ (n as u32 + 17).wrapping_mul(0x85EB_CA6B)
        ^ salt.wrapping_mul(0xC2B2_AE35);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x % 100
}

/// `bit` pressed this frame only if it was not held last frame (a fresh press each time).
fn press(out: &mut Input, f: &Fighter, bit: u16) {
    if f.history[0].buttons & bit == 0 {
        out.buttons |= bit;
    }
}

fn sign(dx: Fx) -> i8 {
    if dx >= Fx::ZERO {
        1
    } else {
        -1
    }
}

/// Free to start a ground attack: standing (or walking, running, crouching) and not sliding fast.
fn free_on_ground(f: &Fighter) -> bool {
    f.grounded() && matches!(f.state, S::Idle | S::Walk | S::Run | S::Dash | S::Crouch)
}

/// The stick for walking at `percent` of its body's walk: kept short of a flick (and of a dash).
fn walk_stick(percent: u8) -> i8 {
    (i32::from(percent) * 127 / 100).clamp(30, 72) as i8
}

/// Starts the tell before move `slot`: it stops and a "!" shows (`MODE_WINDUP`).
fn tell(m: &mut Mob, rc: &RunContent, slot: u8) {
    m.mode = MODE_WINDUP;
    m.timer = u16::from(tier_row(rc, m.tier).tell.max(1));
    m.move_slot = slot;
    m.step = 0;
}

/// Ends an attack (or a failed one): it waits its cooldown.
fn cool_down(m: &mut Mob, k: &MobKind) {
    m.mode = MODE_COOLDOWN;
    m.timer = u16::from(k.cooldown.max(1));
    m.step = 0;
}

/// Every grunt's input this frame (the players' slots stay empty: the run fills the player's), and its run-side flags: super armor
/// through a bruiser's tell and startup, untouchable while it comes out of its door. Called before the step.
pub(super) fn think(
    state: &mut GameState,
    content: &Content,
    rc: &RunContent,
) -> [Input; MAX_BODIES] {
    let mut out = [Input::default(); MAX_BODIES];
    let target = state.fighters[PLAYER];
    let player = (target.active && target.state != S::Respawn).then_some(target.pos);
    let (gate_closed, room) = (state.run.gate_closed, state.run.room);
    for n in 0..MAX_MOBS {
        let mut m = state.run.mobs[n];
        if !m.active {
            continue;
        }
        let i = slot(n);
        let f = state.fighters[i];
        let k = *kind_of(rc, &m);
        let mut armor = false;
        let mut input = Input::default();
        m.credit = m.credit.saturating_sub(1);
        if m.mode == MODE_EMERGE {
            // Coming out of its door: it stands still, then wakes (at once in an arena that is shut on it).
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = if gate_closed && m.room == room {
                    MODE_MOVE
                } else {
                    MODE_IDLE
                };
            }
        } else {
            input = act(
                &mut m,
                &f,
                &k,
                content,
                rc,
                player,
                state.frame,
                n,
                &mut armor,
            );
        }
        let g = &mut state.fighters[i];
        g.super_armor = armor;
        if m.mode == MODE_EMERGE {
            g.invuln = g.invuln.max(2);
        }
        out[i] = input;
        state.run.mobs[n] = m;
    }
    out
}

/// One grunt's input this frame (its behaviour's mode in `m`).
#[allow(clippy::too_many_arguments)]
fn act(
    m: &mut Mob,
    f: &Fighter,
    k: &MobKind,
    content: &Content,
    rc: &RunContent,
    player: Option<Vec2>,
    frame: u32,
    n: usize,
    armor: &mut bool,
) -> Input {
    let mut out = Input::default();
    let tier = tier_row(rc, m.tier);
    let (dx, dy) = player.map_or((Fx::ZERO, Fx::ZERO), |pl| (pl.x - f.pos.x, pl.y - f.pos.y));
    let stage = &content.stage;
    match f.state {
        S::Hitstun
        | S::Rebound
        | S::ShieldBreak
        | S::Knockdown
        | S::GetUp
        | S::Buried
        | S::Encased => {
            // Knocked out of its plan.
            if matches!(m.mode, MODE_WINDUP | MODE_ATTACK | MODE_LEAP) {
                m.mode = MODE_MOVE;
                m.step = 0;
            }
            if f.state == S::Hitstun {
                // Survival DI, up and back in toward you (where the view is), at its tier's rate (decided once a launch).
                if roll(frame, n, 60, 1) < u32::from(tier.di) {
                    out.stick_x = 90 * sign(dx);
                    out.stick_y = 90;
                }
                // A tech, falling near the ground.
                let near_ground = collision::surface_below(stage, f.pos, false)
                    .is_some_and(|(_, d)| d < Fx::from_int(2));
                if near_ground
                    && f.vel.y + f.kb_vel.y < Fx::ZERO
                    && roll(frame, n, 30, 2) < u32::from(tier.tech)
                {
                    press(&mut out, f, buttons::SHIELD);
                }
            }
            return out;
        }
        S::Grabbed => {
            if roll(frame, n, 30, 3) < u32::from(tier.mash) {
                out.stick_x = if frame % 2 == 0 { 127 } else { -127 };
                press(&mut out, f, buttons::JUMP);
            }
            return out;
        }
        S::LedgeHang => {
            // A moment on the ledge, then up (toward the stage: it faces it).
            if f.state_frame > 20 {
                out.stick_x = 100 * f.facing;
            }
            return out;
        }
        _ => {}
    }
    // Its attack, once told.
    if m.mode == MODE_ATTACK {
        return attack(m, f, k, content, dx, armor);
    }
    if !f.grounded() {
        return air(m, f, k, dx, dy);
    }
    if !free_on_ground(f) && !matches!(f.state, S::Shield | S::Turn | S::Landing | S::JumpSquat) {
        return out;
    }
    if f.state == S::JumpSquat {
        // (A leap's squat: the jump held for a full hop, the stick toward you.)
        out.buttons |= buttons::JUMP;
        out.stick_x = 80 * sign(dx);
        return out;
    }
    match m.mode {
        MODE_IDLE => {
            if player.is_some_and(|_| dx.abs() < k.aggro && dy.abs() < Fx::from_int(10)) {
                m.mode = MODE_MOVE;
            }
        }
        MODE_MOVE => {
            out = choose(m, f, k, content, rc, player.is_some(), dx, dy);
        }
        MODE_WINDUP => {
            m.timer = m.timer.saturating_sub(1);
            // The tell: still (turned to you on its first frame), its shield up (a shieldbearer), crouched (a leaper), armored
            // (a bruiser).
            if m.step == 0 && sign(dx) != f.facing && k.role != ROLE_SHIELD {
                out.stick_x = 40 * sign(dx);
            }
            m.step = 1;
            match k.role {
                ROLE_SHIELD => out.buttons |= buttons::SHIELD,
                ROLE_LEAPER => out.stick_y = -127,
                _ => {}
            }
            *armor = k.armor;
            if m.timer == 0 {
                if k.role == ROLE_LEAPER {
                    // The leap: a full hop toward you, the aerial on the way.
                    m.mode = MODE_LEAP;
                    m.step = 0;
                    m.timer = 120;
                    out.stick_y = 0;
                    out.stick_x = 80 * sign(dx);
                    press(&mut out, f, buttons::JUMP);
                } else {
                    m.mode = MODE_ATTACK;
                    m.step = 0;
                    m.timer = ATTEMPT_FRAMES;
                    return attack(m, f, k, content, dx, armor);
                }
            }
        }
        MODE_LEAP => {
            // (Landed before its aerial.)
            m.timer = m.timer.saturating_sub(1);
            if f.state != S::JumpSquat && m.step > 3 || m.timer == 0 {
                cool_down(m, k);
            }
            m.step = m.step.saturating_add(1);
        }
        _ => {
            // Cooling down after an attack (a shieldbearer behind its shield while you are near).
            m.timer = m.timer.saturating_sub(1);
            if k.role == ROLE_SHIELD && dx.abs() < k.reach + Fx::ONE && dy.abs() < Fx::from_int(2) {
                out.buttons |= buttons::SHIELD;
            }
            if m.timer == 0 {
                m.mode = MODE_MOVE;
            }
        }
    }
    out
}

/// In the air: a leaper's leap (steering, then its aerial), or anyone else falling (drifting toward you, a midair jump if there is
/// no ground under it).
fn air(m: &mut Mob, f: &Fighter, k: &MobKind, dx: Fx, dy: Fx) -> Input {
    let mut out = Input::default();
    if !matches!(f.state, S::Airborne) {
        return out;
    }
    if m.mode == MODE_LEAP && k.role == ROLE_LEAPER {
        m.step = m.step.saturating_add(1);
        // Over you on the way down, the down air; level with you and near, the forward air (the attack stick: the movement stick
        // keeps drifting).
        let above = dy < -Fx::from_ratio(3, 2);
        let short = if above { Fx::ZERO } else { Fx::ONE };
        out.stick_x = if dx.abs() > short + Fx::HALF {
            (sign(dx) as i32 * 127) as i8
        } else {
            0
        };
        let aerial = if above && f.vel.y <= Fx::ZERO && dx.abs() < Fx::from_ratio(3, 2) {
            Some((0, buttons::C_DOWN))
        } else if !above && dy.abs() < Fx::from_int(2) && dx.abs() < k.reach2 {
            let c = if sign(dx) == f.facing {
                if f.facing > 0 {
                    buttons::C_RIGHT
                } else {
                    buttons::C_LEFT
                }
            } else {
                0
            };
            (c != 0).then_some((1, c))
        } else {
            None
        };
        if let Some((slot, c)) = aerial.filter(|_| f.history[0].buttons & buttons::ATTACK == 0) {
            out.buttons |= buttons::ATTACK | c;
            m.mode = MODE_ATTACK;
            m.move_slot = slot;
            m.step = 1;
            m.timer = 4;
        }
        return out;
    }
    // Falling (knocked off high ground, or off the end of its floor): toward you, and a midair jump with nothing below.
    out.stick_x = (sign(dx) as i32 * 100) as i8;
    let falling = f.vel.y < Fx::ZERO;
    if falling && f.air_jumps_left > 0 && f.history[0].buttons & buttons::JUMP == 0 && dy > Fx::ZERO
    {
        out.buttons |= buttons::JUMP;
    }
    out
}

/// The attack's input, until its move comes out; then nothing (held where the move wants it) until the move ends, and its cooldown.
fn attack(
    m: &mut Mob,
    f: &Fighter,
    k: &MobKind,
    content: &Content,
    dx: Fx,
    armor: &mut bool,
) -> Input {
    let mut out = Input::default();
    let Some(id) = kit_move(k, m.move_slot) else {
        cool_down(m, k);
        return out;
    };
    let toward = sign(dx);
    if m.step == STARTED {
        if f.state != S::Attack && f.state != S::LedgeAttack {
            cool_down(m, k);
            return out;
        }
        // A bruiser's armor through the startup of its smash.
        *armor = k.armor && m.move_slot == 0 && f.state_frame < first_active(content, f, id);
        if k.role == ROLE_LOBBER && id == MoveId::UpSpecial {
            out.buttons |= buttons::SPECIAL;
        }
        if k.role == ROLE_LEAPER && !f.grounded() {
            // Its aerial drifts onto you.
            let short = if m.move_slot == 0 { Fx::HALF } else { Fx::ONE };
            out.stick_x = if dx.abs() > short {
                (toward as i32 * 127) as i8
            } else {
                0
            };
        }
        return out;
    }
    if f.state == S::Attack && f.move_id == id as u8 {
        m.step = STARTED;
        *armor = k.armor && m.move_slot == 0;
        return attack(m, f, k, content, dx, armor);
    }
    m.timer = m.timer.saturating_sub(1);
    if m.timer == 0 {
        cool_down(m, k);
        return out;
    }
    *armor = k.armor && m.move_slot == 0;
    m.step = m.step.saturating_add(1).min(STARTED - 1);
    if f.state == S::Shield {
        // (A shieldbearer's poke: the shield comes down first.)
        return out;
    }
    let attack_free = f.history[0].buttons & buttons::ATTACK == 0;
    let p = params_of(content, f);
    let slow = f.vel.x.abs() <= p.walk_speed * Fx::from_ratio(11, 10);
    match id {
        MoveId::DashAttack => {
            if matches!(f.state, S::Dash | S::Run) {
                out.stick_x = (toward as i32 * 127) as i8;
                if attack_free {
                    out.buttons |= buttons::ATTACK;
                }
            } else if free_on_ground(f) {
                // A flick from neutral: the dash.
                out.stick_x =
                    if f.history[0].stick_x.unsigned_abs() < STICK_FLICK_FROM.unsigned_abs() {
                        (toward as i32 * 127) as i8
                    } else {
                        0
                    };
            }
        }
        MoveId::UpSpecial => {
            if free_on_ground(f) || f.state == S::Airborne {
                out.stick_y = 127;
                press(&mut out, f, buttons::SPECIAL);
            }
        }
        MoveId::FSmash => {
            if free_on_ground(f) && slow {
                out.stick_x = (toward as i32 * 127) as i8;
                if attack_free {
                    out.buttons |= buttons::ATTACK | buttons::STRONG;
                }
            }
        }
        MoveId::FTilt => {
            if free_on_ground(f) && slow && f.state != S::Dash {
                out.stick_x = 60 * toward;
                if attack_free {
                    out.buttons |= buttons::ATTACK;
                }
            }
        }
        _ => {
            // A jab (or any neutral attack): turned to you first.
            if free_on_ground(f) && slow && f.state != S::Dash {
                if f.facing != toward && dx.abs() > Fx::from_ratio(3, 10) {
                    out.stick_x = 40 * toward;
                } else if attack_free {
                    out.buttons |= buttons::ATTACK;
                }
            }
        }
    }
    out
}

/// A grunt's choice while it moves: where it walks (or runs), and which attack it tells. Returns the input.
#[allow(clippy::too_many_arguments)]
fn choose(
    m: &mut Mob,
    f: &Fighter,
    k: &MobKind,
    content: &Content,
    rc: &RunContent,
    seen: bool,
    dx: Fx,
    dy: Fx,
) -> Input {
    let mut out = Input::default();
    if !seen {
        return out;
    }
    let far = dx.abs();
    let level = dy.abs() < Fx::from_int(2);
    let toward = sign(dx);
    // The attacks, by role.
    let slot = match k.role {
        ROLE_LEAPER => (far <= k.reach && dy <= Fx::ONE).then_some(NO_MOVE),
        ROLE_LOBBER if level && far <= k.reach2 => Some(1),
        ROLE_LOBBER => (far <= k.reach && far > Fx::from_int(3)).then_some(0),
        ROLE_SPEAR if level && far <= k.reach => Some(0),
        ROLE_SPEAR => (level && far > k.reach + Fx::ONE && far <= k.reach2).then_some(1),
        ROLE_SHIELD if level && far <= k.reach2 => Some(1),
        ROLE_SHIELD => (level && far <= k.reach).then_some(0),
        ROLE_BRUISER => (level && far <= k.reach).then_some(0),
        _ if level && far <= k.reach => Some(0),
        _ => (level && far <= k.reach2).then_some(1),
    };
    if let Some(slot) = slot {
        if slot == NO_MOVE || kit_move(k, slot).is_some() {
            tell(m, rc, slot);
            if k.role == ROLE_SHIELD {
                out.buttons |= buttons::SHIELD;
            }
            return out;
        }
    }
    // Where it wants to stand: in reach of its first move (a lobber keeps its distance; a leaper stays on its roof).
    let stop = match k.role {
        ROLE_LOBBER => k.reach * Fx::HALF,
        ROLE_LEAPER => k.reach,
        _ => k.reach * Fx::from_ratio(4, 5),
    };
    if far <= stop {
        return out;
    }
    let p = params_of(content, f);
    // A step ahead (a block's face within reach: from a little behind, so a body already pressed against it finds it too) is jumped.
    let mut probe = f.pos - Vec2::new(Fx::HALF.mul_int(i32::from(toward)), Fx::ZERO);
    let step = collision::move_x(
        &content.stage,
        p,
        &mut probe,
        Fx::from_ratio(9, 10).mul_int(i32::from(toward)),
    );
    // Never off an edge on purpose.
    let ahead = Vec2::new(
        f.pos.x + (p.ecb_half_width + Fx::from_ratio(3, 10)).mul_int(i32::from(toward)),
        // (A little above the feet, so ground rising up a slope ahead counts as floor.)
        f.pos.y + Fx::from_ratio(6, 10),
    );
    let floor = collision::surface_below(&content.stage, ahead, false);
    // (Down off a ledge to you, onto ground below, if it is not one that keeps to high ground; never into a pit.)
    let drops_to_you = dy < -Fx::ONE
        && !matches!(k.role, ROLE_LEAPER | ROLE_LOBBER)
        && floor.is_some_and(|(_, d)| d < Fx::from_int(12));
    if !step && floor.is_none_or(|(_, d)| d > Fx::from_ratio(3, 2)) && !drops_to_you {
        return out;
    }
    if k.run > 0 && far > Fx::from_int(6) {
        // Running: a flick from neutral makes the dash, then the stick held.
        out.stick_x = if matches!(f.state, S::Dash | S::Run)
            || f.history[0].stick_x.unsigned_abs() < STICK_FLICK_FROM.unsigned_abs()
        {
            (toward as i32 * 127) as i8
        } else {
            0
        };
    } else if matches!(f.state, S::Dash | S::Run) && f.facing == toward {
        out.stick_x = (toward as i32 * 127) as i8;
    } else {
        out.stick_x = walk_stick(k.walk) * toward;
    }
    if step && f.history[0].buttons & buttons::JUMP == 0 {
        out.buttons |= buttons::JUMP;
    }
    out
}

/// After the step: the hits and grabs the player landed on each grunt (the combo and the credit for a kill), and a lobber's shell
/// aimed to come down on the player.
pub(super) fn after_step(state: &mut GameState, content: &Content, rc: &RunContent) {
    let target = state.fighters[PLAYER];
    for n in 0..MAX_MOBS {
        let m = state.run.mobs[n];
        if !m.active {
            continue;
        }
        let i = slot(n);
        let f = state.fighters[i];
        let k = *kind_of(rc, &m);
        let mut credit = m.credit;
        if f.percent > m.last_percent {
            count_combo(&mut state.run);
            credit = rc.rules.credit_frames;
        }
        if f.state == S::Grabbed && usize::try_from(f.grab_with) == Ok(PLAYER) {
            credit = rc.rules.credit_frames;
        }
        let g = &mut state.run.mobs[n];
        g.credit = credit;
        g.last_percent = f.percent;
        if k.role == ROLE_LOBBER && target.active {
            aim_shell(state, i, &k, target.pos + Vec2::new(Fx::ZERO, Fx::ONE));
        }
        // (An optional stamina: beaten where it stands once its percent reaches it.)
        if k.stamina > Fx::ZERO {
            let tough = Fx::from_ratio(i32::from(tier_row(rc, m.tier).stamina), 100);
            if f.percent >= k.stamina * tough {
                knock_out(state, content, rc, n);
            }
        }
    }
}

/// A lobber's shell on the frame its move throws it: sent in the arc that comes down on `target` after `MobKind::shot_time` frames
/// (the move's own script aims it by the stick; the run aims it at you instead).
fn aim_shell(state: &mut GameState, i: usize, k: &MobKind, target: Vec2) {
    let t = Fx::from_int(i32::from(k.shot_time.max(1)));
    for pr in state.projectiles.iter_mut() {
        if !pr.active || usize::from(pr.owner) != i || pr.age != 0 || pr.rest != REST_FLYING {
            continue;
        }
        let d = target - pr.pos;
        // (Its arc's gravity, from its shot's numbers: the flight's own.)
        let g = Fx::from_ratio(9, 800);
        let vx = (d.x / t).max(-Fx::HALF).min(Fx::HALF);
        let vy = (d.y / t + g * t * Fx::HALF)
            .max(-Fx::from_ratio(3, 5))
            .min(Fx::from_ratio(9, 10));
        pr.vel = Vec2::new(vx, vy);
    }
}

/// A grunt's slot freed: its grab let go, its item dropped, its shots gone, its ledge free, and every hit record of it cleared (a new
/// grunt in the slot is a new target).
pub(super) fn remove(state: &mut GameState, content: &Content, n: usize) {
    let i = slot(n);
    if state.fighters[i].active {
        crate::grab::drop_grab(state, content, i);
        crate::items::let_go(state, content, i);
    }
    for pr in state.projectiles.iter_mut() {
        if pr.active && (usize::from(pr.owner) == i || usize::from(pr.origin) == i) {
            pr.active = false;
        }
    }
    for o in state.ledge_owner.iter_mut() {
        if usize::try_from(*o) == Ok(i) {
            *o = NONE;
        }
    }
    crate::orbs::clear_target(state, i);
    forget(state, i);
    state.fighters[i].active = false;
    state.fighters[i].grab_with = NONE;
    state.run.mobs[n].active = false;
}

/// Clears every record of body `i` having been hit (moves' and shots' hit masks, items' hits).
pub(super) fn forget(state: &mut GameState, i: usize) {
    let mut bits = 0u128;
    for g in 0..8 {
        bits |= combat::hit_bit(i, g);
    }
    for f in state.fighters.iter_mut() {
        f.hit_mask &= !bits;
    }
    for p in state.projectiles.iter_mut() {
        p.hit_mask &= !bits;
        p.hit_age[i] = 0;
    }
    for it in state.items.iter_mut() {
        it.hit_mask &= !(1u16 << i);
        it.struck[i] = 0;
    }
}

/// Puts grunt `kind` (of `tier`, from room `room`) into a free slot at `pos` on `platform`: a fighter of its body, with its kind's
/// and tier's weight and damage, coming out of its door first (`emerge_frames`). Returns its number, or none if every slot is taken.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn(
    state: &mut GameState,
    content: &Content,
    rc: &RunContent,
    kind: u8,
    pos: Vec2,
    platform: i8,
    tier: u8,
    room: u32,
) -> Option<usize> {
    let n =
        (0..MAX_MOBS).find(|&n| !state.run.mobs[n].active && !state.fighters[slot(n)].active)?;
    let k = rc.mobs.get(usize::from(kind))?;
    let body = usize::from(k.body).min(content.fighters.len() - 1);
    let mut f = Fighter::spawn(
        pos,
        body as u8,
        -1,
        platform,
        content.fighters[body].air_jumps,
        content.rules.shield_max,
    );
    f.active = true;
    f.stocks = 1;
    f.weight = Some(grunt_weight(rc, kind, tier));
    f.damage_scale = u16::from(tier_row(rc, tier).damage.max(1));
    let i = slot(n);
    forget(state, i);
    state.fighters[i] = f;
    let emerge = rc.rules.emerge_frames;
    state.run.mobs[n] = Mob {
        active: true,
        kind,
        mode: if emerge > 0 { MODE_EMERGE } else { MODE_IDLE },
        timer: u16::from(emerge),
        room,
        tier,
        ..Mob::default()
    };
    Some(n)
}

/// A grunt knocked out: the KO's place for the game's effect, the player's kill if they hit it lately (the coins and the combo), and
/// its slot freed.
fn knock_out(state: &mut GameState, content: &Content, rc: &RunContent, n: usize) {
    let m = state.run.mobs[n];
    let f = state.fighters[slot(n)];
    let run = &mut state.run;
    run.kos = run.kos.wrapping_add(1);
    run.ko_at = f.pos;
    if m.credit > 0 {
        pay_kill(run, rc, m.kind);
    }
    remove(state, content, n);
}

/// Grunts that leave: knocked out past the blast zones that follow the view (when launched), or gone out of the world (below it,
/// or out of the rooms in play). A grunt the player hit lately is their kill. Before the step, a grunt launched in tumble into the
/// gate behind goes through it (the gate is a wall for the player only).
pub(super) fn knock_outs(state: &mut GameState, content: &Content, rc: &RunContent) {
    let rules = &rc.rules;
    let (left, right, bottom, top) = grunt_zone(&state.run, rules);
    let (wall, end) = (state.run.wall_x, state.run.window_end());
    let (blast_bottom, blast_top) = state.run.blast_y(rules);
    for n in 0..MAX_MOBS {
        if !state.run.mobs[n].active {
            continue;
        }
        let f = state.fighters[slot(n)];
        if !f.active {
            remove(state, content, n);
            continue;
        }
        let launched = f.state == S::Hitstun;
        let p = f.pos;
        let past_zone = p.x < left || p.x > right || p.y < bottom || p.y > top;
        let out_of_world = p.y < blast_bottom
            || p.y > blast_top
            || p.x < wall - rules.gate_thickness - rules.blast_margin
            || p.x > end + rules.blast_margin;
        if launched && past_zone || out_of_world {
            if launched || state.run.mobs[n].credit > 0 {
                knock_out(state, content, rc, n);
            } else {
                remove(state, content, n);
            }
        }
    }
}

/// Before the step: a grunt flying in tumble toward the gate behind (the left wall) is put through it, so it flies on into the zone
/// behind. The wall stays a wall for the player (owner decision, R2c).
pub(super) fn through_gate(state: &mut GameState, content: &Content, rc: &RunContent) {
    let wall = state.run.wall_x;
    let thick = rc.rules.gate_thickness;
    for n in 0..MAX_MOBS {
        if !state.run.mobs[n].active {
            continue;
        }
        let i = slot(n);
        let f = state.fighters[i];
        if f.state != S::Hitstun || !f.tumble {
            continue;
        }
        let half = params_of(content, &f).ecb_half_width;
        let dx = f.vel.x + f.kb_vel.x;
        // (Moving left and about to reach the wall's face this frame or the next.)
        if dx < Fx::ZERO && f.pos.x - half + dx.mul_int(2) <= wall && f.pos.x > wall - thick {
            let g = &mut state.fighters[i];
            g.pos.x = wall - thick - half - Fx::from_ratio(1, 10);
            g.platform = NONE;
        }
    }
}

/// Hits body `d` with `hb` at `at` through the versus hit (`combat::apply_hit`), from a hazard or the van standing in on a player slot
/// no one plays (restored right after): `pos`, `facing` and `vel` are what the hit reads, with no rage or staleness of its own.
/// Returns true if it landed (or a counter caught it).
pub(super) fn hit_body(
    state: &mut GameState,
    content: &Content,
    d: usize,
    (pos, facing, vel): (Vec2, i8, Vec2),
    hb: &Hitbox,
    at: Vec2,
) -> bool {
    let fd = state.fighters[d];
    if !fd.active {
        return false;
    }
    let Some(proxy) = (1..MAX_FIGHTERS).find(|&i| i != d && !state.fighters[i].active) else {
        return false;
    };
    let counter = combat::counter_open(content, &fd, true);
    let intangible = combat::is_intangible(&fd);
    if intangible && counter.is_none() {
        return false;
    }
    let hurt = combat::body_hurtboxes(content, &fd);
    let shut = combat::intangible_parts(&fd, weapon_of(content, params_of(content, &fd)));
    let (touching, caught) = combat::reach(counter, intangible, hb, &hurt, shut, at);
    if !touching {
        return false;
    }
    let saved = state.fighters[proxy];
    {
        let p = &mut state.fighters[proxy];
        p.char_id = 0;
        p.pos = pos;
        p.facing = facing;
        p.vel = vel;
        p.percent = Fx::ZERO;
        p.stale = [0; crate::state::STALE_QUEUE];
        p.hitlag = 0;
        p.damage_scale = 0;
        p.wing_points = Fx::ZERO;
    }
    let side = if hb.x < Fx::ZERO { -facing } else { facing };
    combat::apply_hit(
        state,
        content,
        proxy,
        d,
        hb,
        side,
        pos,
        false,
        combat::STUN_NORMAL,
        0,
        (at, vel),
        caught,
        false,
    );
    state.fighters[proxy] = saved;
    true
}

/// The runaway van (R2b, the first rare event): it waits at its street's start until the player lands on its roof, then drives
/// off (speeding up to `van_speed`) to the street's end, carrying its rider (untouchable while riding) and running into the grunts
/// in its way (each once, through the versus hit: the player's kills); then it parks.
pub(super) fn van(state: &mut GameState, content: &Content, rc: &RunContent) {
    let mut v = state.run.van;
    if v.mode == VAN_NONE {
        return;
    }
    let Ok(pi) = usize::try_from(v.platform) else {
        return;
    };
    let rules = &rc.rules;
    let f = state.fighters[PLAYER];
    let on_roof = f.active && f.platform == v.platform && f.grounded();
    if v.mode == VAN_WAITING && on_roof {
        v.mode = VAN_DRIVING;
    }
    if v.mode == VAN_DRIVING {
        v.speed = (v.speed + rules.van_accel).min(rules.van_speed);
        let street_end = state.run.slot_of(v.room).map_or(v.x, |s| s.end);
        let end = street_end.min(state.run.right_limit()) - VAN_HALF - Fx::ONE;
        let dx = v.speed.min((end - v.x).max(Fx::ZERO));
        v.timer = v.timer.saturating_add(1);
        if dx <= Fx::ZERO || v.timer >= rules.van_frames {
            v.mode = VAN_PARKED;
            v.speed = Fx::ZERO;
        }
        v.x += dx;
        let roof = &mut state.run.platforms[pi.min(state.run.platforms.len() - 1)];
        roof.left = v.x - VAN_HALF;
        roof.right = v.x + VAN_HALF;
        if on_roof {
            state.fighters[PLAYER].pos.x += dx;
        }
        // Grunts in its way are sent flying.
        let reach = VAN_HALF + Fx::from_ratio(3, 10);
        let at = Vec2::new(v.x + VAN_HALF, v.y + VAN_HEIGHT * Fx::HALF);
        for n in 0..MAX_MOBS {
            let m = state.run.mobs[n];
            let bit = 1u16 << n;
            let g = state.fighters[slot(n)];
            if !m.active || m.mode == MODE_EMERGE || v.struck & bit != 0 {
                continue;
            }
            if (g.pos.x - v.x).abs() > reach
                || g.pos.y >= v.y + VAN_HEIGHT
                || g.pos.y + Fx::from_int(2) <= v.y
            {
                continue;
            }
            let mut hb = rules.van_hit;
            // (Centred on the grunt, so it lands wherever the grunt stands against the van.)
            hb.radius = hb.radius.max(Fx::from_int(3));
            let centre = g.pos + Vec2::new(Fx::ZERO, Fx::ONE);
            if hit_body(
                state,
                content,
                slot(n),
                (at, 1, Vec2::new(v.speed, Fx::ZERO)),
                &hb,
                centre,
            ) {
                v.struck |= bit;
                v.hits = v.hits.saturating_add(1);
            }
        }
    }
    v.riding = on_roof && v.mode == VAN_DRIVING;
    if v.riding {
        let g = &mut state.fighters[PLAYER];
        g.invuln = g.invuln.max(2);
    }
    state.run.van = v;
}

/// Whether the player's attack (a normal hit) or a shot of theirs in flight touches this shape now (a box of half-size `half_w` by
/// `half_h` and radius `radius` around `centre`): what sets off a prop or bonks the shopkeeper.
pub(super) fn player_touches(
    state: &GameState,
    content: &Content,
    centre: Vec2,
    (half_w, half_h, radius): (Fx, Fx, Fx),
) -> bool {
    let f = state.fighters[PLAYER];
    if f.active && matches!(f.state, S::Attack | S::LedgeAttack) && f.hitlag == 0 {
        let params = params_of(content, &f);
        let mv = weapon_of(content, params).get(f.move_id);
        let touched = combat::active_hitboxes(&f, mv, params.hitbox_scale).any(|(_, hb, at)| {
            hb.kind == HIT_NORMAL
                && combat::shapes_touch(
                    centre,
                    half_w,
                    half_h,
                    radius,
                    at,
                    hb.half_width,
                    hb.half_height,
                    hb.radius,
                )
        });
        if touched {
            return true;
        }
    }
    state.projectiles.iter().any(|p| {
        let d = p.pos - centre;
        p.active
            && p.rest == REST_FLYING
            && usize::from(p.owner) == PLAYER
            && d.x.abs() <= half_w + radius + Fx::ONE
            && d.y.abs() <= half_h + radius + Fx::ONE
    })
}

/// Whether the player's attack or shot touches the shopkeeper this frame (inside the shop). It is never hurt: the shop only makes
/// it dizzy for a moment, then it sulks.
pub(super) fn keeper_bonked(state: &GameState, content: &Content, rc: &RunContent) -> bool {
    let shop = &rc.interior;
    let feet = Vec2::new(Fx::from_int(INTERIOR_X) + shop.keeper.x, shop.keeper.y);
    let centre = feet + Vec2::new(Fx::ZERO, shop.keeper_radius);
    player_touches(
        state,
        content,
        centre,
        (Fx::ZERO, Fx::ZERO, shop.keeper_radius),
    )
}

// (Grunt slots are bits of a `u16` in `Van::struck` and `Prop::hit_mobs`.)
const _: () = assert!(MAX_MOBS <= 16);
