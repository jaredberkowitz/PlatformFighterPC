//! Mobs: the run's enemies. Their hits taken and given use the versus formulas (`combat::knockback`, `hitstun_frames`,
//! `hitlag_frames`, `launch_angle`), and their hits on the fighter go through `combat::apply_hit`, so a shield, DI, crouch cancelling,
//! armor or a counter all work against them as against a fighter.
//!
//! Behaviour is a small state machine per kind (R1; `MobKind::behavior`): a walker walks at the player, stops at edges, hops steps
//! and lunges; a flyer hovers above and to one side of the player and dives at where they were. The numbers are content
//! (`MobKind`). R2 plans to move the behaviour itself into scripts (`sim-script`) so new enemies are data.

use super::data::{MobKind, RunContent, MOB_FLYER, MOB_WALKER};
use super::{
    count_combo, pay_kill, Mob, INTERIOR_X, MODE_ATTACK, MODE_BURST, MODE_COOLDOWN, MODE_EMERGE,
    MODE_IDLE, MODE_MOVE, MODE_WINDUP, PLAYER,
};
use crate::collision;
use crate::combat::{self, params_of, weapon_of};
use crate::content::{Content, Stage};
use crate::fixed::Fx;
use crate::moves::{Hitbox, ANGLE_AUTOLINK, ANGLE_PULL, ANGLE_VORTEX, HIT_NORMAL};
use crate::state::{FighterState as S, GameState, NONE, REST_FLYING};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

fn approach(v: Fx, target: Fx, delta: Fx) -> Fx {
    if v < target {
        (v + delta).min(target)
    } else {
        (v - delta).max(target)
    }
}

fn kind_of<'a>(rc: &'a RunContent, m: &Mob) -> &'a MobKind {
    &rc.mobs[usize::from(m.kind).min(rc.mobs.len() - 1)]
}

/// The mob's body: one circle standing on its feet.
fn body(m: &Mob, k: &MobKind) -> (Vec2, Fx) {
    (m.pos + Vec2::new(Fx::ZERO, k.radius), k.radius)
}

/// Whether a mob can be hit now (not bursting, nor still coming out of its door).
fn hittable(m: &Mob) -> bool {
    m.active && m.mode != MODE_BURST && m.mode != MODE_EMERGE
}

/// A hit on a mob: damage off its stamina, the percent the knockback reads, the launch, hitstun and hitlag (the attacker's too, for
/// a melee hit). `side` is the way 0 degrees points; `rage` the attacker's. Returns the hitlag.
fn take_hit(
    m: &mut Mob,
    k: &MobKind,
    hb: &Hitbox,
    side: i8,
    rage: Fx,
    content: &Content,
    burst_frames: u8,
) -> u8 {
    let rules = &content.rules;
    let damage = hb.damage;
    m.percent = (m.percent + damage).min(Fx::from_int(999));
    let kb = if hb.fixed_kb > 0 {
        combat::fixed_knockback(
            hb.fixed_kb,
            k.weight,
            hb.base_knockback,
            hb.knockback_growth,
        )
    } else {
        combat::knockback(
            m.percent,
            damage,
            k.weight,
            hb.base_knockback,
            hb.knockback_growth,
        ) * rage
    };
    // (The special angles of fighter moves, which carry or pull a fighter, send a mob at 45 degrees; the autolink's grounded 80 is
    // kept.)
    let degrees = match hb.angle {
        ANGLE_AUTOLINK => 80,
        ANGLE_VORTEX | ANGLE_PULL => 45,
        a => a,
    };
    let grounded = m.platform != NONE;
    let angle = combat::launch_angle(degrees, side, grounded, kb);
    let speed = kb * Fx::from_ratio(3, 800);
    m.kb_vel = Vec2::new(
        crate::trig::cos(angle) * speed,
        crate::trig::sin(angle) * speed,
    );
    m.hitstun = combat::hitstun_frames(kb, rules.hitstun_mult);
    if m.hitstun > 0 {
        m.vel = Vec2::ZERO;
        if m.kb_vel.y > Fx::ZERO {
            m.platform = NONE;
        } else if grounded {
            m.kb_vel.y = Fx::ZERO;
        }
        m.mode = MODE_IDLE;
        m.timer = 0;
        m.struck = false;
    }
    let scale = Fx::from_ratio(i32::from(hb.hitlag), 100);
    let lag = combat::hitlag_frames(damage, rules, false, false, scale);
    m.hitlag = lag;
    if k.stamina > Fx::ZERO {
        m.stamina -= damage;
        if m.stamina <= Fx::ZERO {
            m.mode = MODE_BURST;
            m.timer = u16::from(burst_frames.max(1));
            m.platform = NONE;
        }
    }
    lag
}

/// What a fighter's charged attack adds to its hit's damage (as `combat::resolve_hits` does for fighters).
fn charged(content: &Content, f: &crate::state::Fighter, hb: &Hitbox) -> Fx {
    let attack = weapon_of(content, params_of(content, f)).get(f.move_id);
    if let Some(add) = attack.charge_add {
        return add.damage(hb.damage, f.charge);
    }
    if f.charge == 0 {
        return hb.damage;
    }
    let rules = &content.rules;
    let full = if attack.charge_bonus > 0 {
        attack.charge_bonus
    } else {
        rules.charge_bonus_percent
    };
    let bonus = Fx::from_ratio(
        i32::from(f.charge) * i32::from(full),
        100 * i32::from(rules.charge_frames.max(1)),
    );
    hb.damage * (Fx::ONE + bonus)
}

/// The fighters' attacks and shots that reach a mob this frame. A move hits a mob once per hit group (the best priority box), as it
/// does a fighter; a shot once (a piercing one goes on).
pub(super) fn player_hits(state: &mut GameState, content: &Content, rc: &RunContent) {
    let burst = rc.rules.burst_frames;
    for a in 0..MAX_FIGHTERS {
        let f = state.fighters[a];
        if !f.active {
            continue;
        }
        let attacking = matches!(f.state, S::Attack | S::LedgeAttack);
        let stamp = 1 << 16 | (a as u32) << 8 | u32::from(f.move_id);
        // A swing is the same swing while its move goes on (its frame holds still through hitlag); it ends when the fighter stops
        // attacking, changes move, or starts the move again.
        for m in state
            .run
            .mobs
            .iter_mut()
            .filter(|m| m.hit_stamp >> 8 == (1 << 8 | a as u32))
        {
            if !attacking || m.hit_stamp != stamp || f.state_frame < m.hit_age {
                m.hit_stamp = 0;
                m.hit_groups = 0;
            } else {
                m.hit_age = f.state_frame;
            }
        }
        if !attacking || f.hitlag > 0 {
            continue;
        }
        let params = params_of(content, &f);
        let mv = weapon_of(content, params).get(f.move_id);
        let rage = combat::rage_multiplier(f.percent, &content.rules);
        let mut freeze = 0u8;
        for n in 0..state.run.mobs.len() {
            let m = state.run.mobs[n];
            if !hittable(&m) {
                continue;
            }
            let k = *kind_of(rc, &m);
            let groups = if m.hit_stamp == stamp {
                m.hit_groups
            } else {
                0
            };
            let (centre, radius) = body(&m, &k);
            let mut best: Option<Hitbox> = None;
            for (_, hb, at) in combat::active_hitboxes(&f, mv, params.hitbox_scale) {
                if hb.kind != HIT_NORMAL || hb.push || groups & (1 << hb.group.min(7)) != 0 {
                    continue;
                }
                let touching = combat::shapes_touch(
                    centre,
                    Fx::ZERO,
                    Fx::ZERO,
                    radius,
                    at,
                    hb.half_width,
                    hb.half_height,
                    hb.radius,
                );
                if touching && best.is_none_or(|b| hb.priority < b.priority) {
                    best = Some(hb);
                }
            }
            let Some(mut hb) = best else {
                continue;
            };
            hb.damage = charged(content, &f, &hb);
            let side = if hb.x < Fx::ZERO { -f.facing } else { f.facing };
            let m = &mut state.run.mobs[n];
            m.hit_stamp = stamp;
            m.hit_groups = groups | (1 << hb.group.min(7));
            m.hit_age = f.state_frame;
            m.credit = rc.rules.credit_frames;
            let lag = take_hit(m, &k, &hb, side, rage, content, burst);
            let (died, kind) = (m.mode == MODE_BURST, m.kind);
            freeze = freeze.max(lag);
            count_combo(&mut state.run);
            if died {
                pay_kill(&mut state.run, rc, kind);
            }
        }
        if freeze > 0 {
            let f = &mut state.fighters[a];
            f.hitlag = f.hitlag.max(freeze);
        }
    }
    shots(state, content, rc);
}

/// Shots (projectiles) from the fighters reaching mobs.
fn shots(state: &mut GameState, content: &Content, rc: &RunContent) {
    let burst = rc.rules.burst_frames;
    for n in 0..state.projectiles.len() {
        let pr = state.projectiles[n];
        let bit = 1u16 << n.min(15);
        if !pr.active || pr.rest != REST_FLYING {
            // (The slot is free or the shot is at rest: whoever it hit may be hit by the slot's next shot.)
            for m in state.run.mobs.iter_mut() {
                m.shot_mask &= !bit;
            }
            continue;
        }
        let origin = usize::from(pr.origin).min(MAX_FIGHTERS - 1);
        let owner = usize::from(pr.owner).min(MAX_FIGHTERS - 1);
        let op = params_of(content, &state.fighters[origin]);
        let Some(spec) = weapon_of(content, op).get(pr.move_id).projectile else {
            continue;
        };
        let (mut hb, pierce) = if pr.bursting {
            match spec.burst.and_then(|b| b.hit_at(pr.age)) {
                Some(h) => (h, true),
                None => continue,
            }
        } else {
            (spec.hit_at(pr.age).0, spec.pierce)
        };
        if hb.kind != HIT_NORMAL || hb.push {
            continue;
        }
        let size = Fx::from_ratio(i32::from(pr.size), 100);
        hb.radius = hb.radius * op.hitbox_scale * size;
        hb.half_height = hb.half_height * op.hitbox_scale * size;
        hb.half_width = hb.half_width * op.hitbox_scale;
        if spec.charged.is_some() && !pr.bursting {
            hb.damage = pr.damage;
        }
        hb.damage = hb.damage * Fx::from_int(i32::from(pr.power)) / Fx::from_int(100);
        if hb.radius <= Fx::ZERO || hb.damage <= Fx::ZERO {
            continue;
        }
        let side = match pr.vel.x.signum_int() {
            0 => pr.facing,
            s => s as i8,
        };
        let rage = combat::rage_multiplier(state.fighters[owner].percent, &content.rules);
        for i in 0..state.run.mobs.len() {
            let m = state.run.mobs[i];
            if !hittable(&m) || m.shot_mask & bit != 0 {
                continue;
            }
            let k = *kind_of(rc, &m);
            let (centre, radius) = body(&m, &k);
            if !combat::shapes_touch(
                centre,
                Fx::ZERO,
                Fx::ZERO,
                radius,
                pr.pos,
                hb.half_width,
                hb.half_height,
                hb.radius,
            ) {
                continue;
            }
            let m = &mut state.run.mobs[i];
            m.shot_mask |= bit;
            m.credit = rc.rules.credit_frames;
            take_hit(m, &k, &hb, side, rage, content, burst);
            let (died, kind) = (m.mode == MODE_BURST, m.kind);
            count_combo(&mut state.run);
            if died {
                pay_kill(&mut state.run, rc, kind);
            }
            if !pierce {
                state.projectiles[n].active = false;
                break;
            }
        }
    }
}

/// Moves a body sideways by `dx`, stopping at the face of a solid block (not at a slope's joint). Returns true if it met one.
fn move_x(stage: &Stage, pos: &mut Vec2, half: Fx, height: Fx, dx: Fx) -> bool {
    let mut x = pos.x + dx;
    let mut hit = false;
    for b in stage.platforms.iter().filter(|b| !b.pass_through) {
        let face_top = if dx > Fx::ZERO { b.y } else { b.top_right() };
        if pos.y >= face_top || pos.y + height <= b.bottom {
            continue;
        }
        if dx > Fx::ZERO && pos.x + half <= b.left && x + half > b.left {
            if collision::face_is_joint(stage, b, -1) {
                continue;
            }
            x = b.left - half;
            hit = true;
        } else if dx < Fx::ZERO && pos.x - half >= b.right && x - half < b.right {
            if collision::face_is_joint(stage, b, 1) {
                continue;
            }
            x = b.right + half;
            hit = true;
        }
    }
    pos.x = x;
    hit
}

/// Moves a body up or down by `dy`: landing on a surface it crosses going down (returns the platform), stopping under a ceiling
/// going up.
fn move_y(stage: &Stage, pos: &mut Vec2, height: Fx, dy: Fx) -> Option<usize> {
    let next = Vec2::new(pos.x, pos.y + dy);
    if dy <= Fx::ZERO {
        if let Some(i) = collision::find_landing(stage, *pos, next, false) {
            pos.y = stage.platforms[i].top_at(pos.x);
            return Some(i);
        }
        pos.y = next.y;
        return None;
    }
    let mut y = next.y;
    for b in stage.platforms.iter().filter(|b| !b.pass_through) {
        if pos.x >= b.left
            && pos.x <= b.right
            && pos.y + height <= b.bottom
            && y + height > b.bottom
        {
            y = b.bottom - height;
        }
    }
    pos.y = y;
    None
}

/// Moves a mob by its speed and launch, with the stage in the way (a flyer only while launched or diving). Returns true if it met a
/// wall.
fn physics(m: &mut Mob, k: &MobKind, stage: &Stage, decay: Fx, solid: bool) -> bool {
    let launched = m.hitstun > 0 || m.mode == MODE_BURST;
    if launched {
        let speed = m.kb_vel.length();
        m.kb_vel = if speed > decay {
            m.kb_vel * ((speed - decay) / speed)
        } else {
            Vec2::ZERO
        };
    }
    let falls = k.behavior == MOB_WALKER || launched;
    if falls && m.platform == NONE {
        m.vel.y = (m.vel.y - k.gravity).max(-k.max_fall);
    }
    let total = m.vel + m.kb_vel;
    if !solid || m.mode == MODE_BURST {
        m.pos += total;
        return false;
    }
    let height = k.radius.mul_int(2);
    let wall = move_x(stage, &mut m.pos, k.radius, height, total.x);
    if wall {
        m.vel.x = Fx::ZERO;
        m.kb_vel.x = -(m.kb_vel.x * Fx::HALF);
    }
    if let Some(p) = collision::platform(stage, m.platform) {
        // On its ground it follows the surface (a slope too), and carries on where a slope joins other ground.
        if m.pos.x >= p.left && m.pos.x <= p.right && (p.sloped() || p.y == m.pos.y) {
            m.pos.y = p.top_at(m.pos.x);
        } else if let Some(i) = collision::continuation(stage, m.platform, m.pos.x) {
            m.platform = i as i8;
            m.pos.y = stage.platforms[i].top_at(m.pos.x);
        } else {
            m.platform = NONE;
        }
    } else {
        m.platform = NONE;
    }
    if m.platform == NONE {
        let dy = total.y;
        let from = m.pos.y;
        if let Some(i) = move_y(stage, &mut m.pos, height, dy) {
            m.platform = i as i8;
            m.vel.y = Fx::ZERO;
            m.kb_vel.y = Fx::ZERO;
        } else if dy > Fx::ZERO && m.pos.y < from + dy {
            m.vel.y = Fx::ZERO;
            m.kb_vel.y = Fx::ZERO;
        }
    }
    wall
}

/// Every mob's frame: hitlag, then the launch or its behaviour, then its movement.
pub(super) fn update(state: &mut GameState, content: &Content, rc: &RunContent) {
    let target = state.fighters[PLAYER];
    let player = (target.active && target.state != S::Respawn).then_some(target.pos);
    let decay = content.rules.knockback_decay;
    for n in 0..state.run.mobs.len() {
        let mut m = state.run.mobs[n];
        if !m.active {
            continue;
        }
        let k = *kind_of(rc, &m);
        m.credit = m.credit.saturating_sub(1);
        if m.hitlag > 0 {
            m.hitlag -= 1;
            state.run.mobs[n] = m;
            continue;
        }
        if m.mode == MODE_EMERGE {
            // Coming out of its door: it stands still, then wakes (at once in an arena that is shut on it).
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                let run = &state.run;
                m.mode = if run.gate_closed && m.room == run.room {
                    MODE_MOVE
                } else {
                    MODE_IDLE
                };
            }
            state.run.mobs[n] = m;
            continue;
        }
        if m.mode == MODE_BURST {
            physics(&mut m, &k, &content.stage, decay, false);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.active = false;
            }
            state.run.mobs[n] = m;
            continue;
        }
        if m.hitstun > 0 {
            m.hitstun -= 1;
            if m.hitstun == 0 {
                // Out of the launch: what is left of it carries on as its own speed.
                m.vel += m.kb_vel;
                m.kb_vel = Vec2::ZERO;
                m.mode = MODE_MOVE;
            }
            physics(&mut m, &k, &content.stage, decay, true);
            state.run.mobs[n] = m;
            continue;
        }
        match k.behavior {
            MOB_FLYER => fly(&mut m, &k, &content.stage, player),
            _ => walk(&mut m, &k, &content.stage, player),
        }
        state.run.mobs[n] = m;
    }
}

fn face(m: &mut Mob, dx: Fx) {
    if dx.abs() > Fx::from_ratio(3, 10) {
        m.facing = if dx > Fx::ZERO { 1 } else { -1 };
    }
}

fn walk(m: &mut Mob, k: &MobKind, stage: &Stage, player: Option<Vec2>) {
    let grounded = m.platform != NONE;
    let friction = k.accel;
    match m.mode {
        MODE_IDLE => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            if player.is_some_and(|p| (p.x - m.pos.x).abs() < k.aggro) {
                m.mode = MODE_MOVE;
            }
        }
        MODE_MOVE => {
            let Some(p) = player else {
                m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
                return finish_walk(m, k, stage, false);
            };
            let dx = p.x - m.pos.x;
            face(m, dx);
            let mut want = k.speed.mul_int(i32::from(m.facing));
            if grounded {
                // Never off an edge on purpose.
                let ahead = Vec2::new(
                    m.pos.x + (k.radius + Fx::from_ratio(3, 10)).mul_int(i32::from(m.facing)),
                    // (A little above the feet, so ground rising up a slope ahead counts as floor.)
                    m.pos.y + Fx::from_ratio(6, 10),
                );
                let floor = collision::surface_below(stage, ahead, false);
                if floor.is_none_or(|(_, d)| d > Fx::from_ratio(3, 2)) {
                    want = Fx::ZERO;
                }
            }
            if dx.abs() < k.radius {
                want = Fx::ZERO;
            }
            m.vel.x = approach(m.vel.x, want, k.accel);
            if grounded && dx.abs() < k.attack_range && (p.y - m.pos.y).abs() < Fx::from_int(2) {
                m.mode = MODE_WINDUP;
                m.timer = u16::from(k.windup);
                m.vel.x = Fx::ZERO;
            }
            return finish_walk(m, k, stage, want != Fx::ZERO);
        }
        MODE_WINDUP => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = MODE_ATTACK;
                m.timer = u16::from(k.attack_frames.max(1));
                m.vel.x = k.lunge.mul_int(i32::from(m.facing));
                m.struck = false;
            }
        }
        MODE_ATTACK => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = MODE_COOLDOWN;
                m.timer = u16::from(k.cooldown);
            }
        }
        _ => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = MODE_MOVE;
            }
        }
    }
    finish_walk(m, k, stage, false);
}

/// A walker moves; walking into a step it hops.
fn finish_walk(m: &mut Mob, k: &MobKind, stage: &Stage, walking: bool) {
    let wall = physics(m, k, stage, Fx::ZERO, true);
    if wall && walking && m.platform != NONE && k.jump > Fx::ZERO {
        m.vel.y = k.jump;
        m.platform = NONE;
    }
}

fn fly(m: &mut Mob, k: &MobKind, stage: &Stage, player: Option<Vec2>) {
    // (Back in the air after a launch that put it on the ground.)
    m.platform = NONE;
    match m.mode {
        MODE_IDLE => {
            m.vel = Vec2::new(
                approach(m.vel.x, Fx::ZERO, k.accel),
                approach(m.vel.y, Fx::ZERO, k.accel),
            );
            if player.is_some_and(|p| (p.x - m.pos.x).abs() < k.aggro) {
                m.mode = MODE_MOVE;
            }
        }
        MODE_MOVE => {
            let Some(p) = player else {
                m.vel = Vec2::new(
                    approach(m.vel.x, Fx::ZERO, k.accel),
                    approach(m.vel.y, Fx::ZERO, k.accel),
                );
                m.pos += m.vel;
                return;
            };
            let dx = p.x - m.pos.x;
            face(m, dx);
            // Hover on its own side of the player, a little off them, `hover_height` up.
            let side = if m.pos.x >= p.x { 1 } else { -1 };
            let goal = Vec2::new(p.x + Fx::from_int(3 * side), p.y + k.hover_height);
            let to = goal - m.pos;
            let dist = to.length();
            let want = if dist > Fx::from_ratio(1, 5) {
                to * (k.speed / dist)
            } else {
                Vec2::ZERO
            };
            m.vel = Vec2::new(
                approach(m.vel.x, want.x, k.accel),
                approach(m.vel.y, want.y, k.accel),
            );
            m.pos += m.vel;
            if dx.abs() < k.attack_range && (m.pos.y - goal.y).abs() < Fx::from_ratio(3, 2) {
                m.mode = MODE_WINDUP;
                m.timer = u16::from(k.windup);
                m.target = p + Vec2::new(Fx::ZERO, Fx::ONE);
                m.vel = Vec2::ZERO;
            }
        }
        MODE_WINDUP => {
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                let to = m.target - m.pos;
                let dist = to.length().max(Fx::from_ratio(1, 10));
                m.vel = to * (k.lunge / dist);
                m.mode = MODE_ATTACK;
                m.timer = u16::from(k.attack_frames.max(1));
                m.struck = false;
            }
        }
        MODE_ATTACK => {
            // The dive stops on the ground.
            let height = k.radius.mul_int(2);
            m.pos.x += m.vel.x;
            let landed = if m.vel.y < Fx::ZERO {
                move_y(stage, &mut m.pos, height, m.vel.y).is_some()
            } else {
                m.pos.y += m.vel.y;
                false
            };
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 || landed {
                m.mode = MODE_COOLDOWN;
                m.timer = u16::from(k.cooldown);
                m.vel = Vec2::new(Fx::ZERO, k.speed * Fx::from_ratio(3, 5));
            }
        }
        _ => {
            m.pos += m.vel;
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = MODE_MOVE;
            }
        }
    }
}

/// Mobs mid-attack hitting the fighters, through the versus hit (`combat::apply_hit`). The mob stands in as the hit's source on a
/// fighter slot no one plays (restored right after): its position and facing are what the hit reads, with no rage or staleness of
/// its own.
pub(super) fn attacks(state: &mut GameState, content: &Content, rc: &RunContent) {
    for n in 0..state.run.mobs.len() {
        let m = state.run.mobs[n];
        if !m.active || m.mode != MODE_ATTACK || m.struck || m.hitlag > 0 || m.hitstun > 0 {
            continue;
        }
        let k = *kind_of(rc, &m);
        let hb = k.hitbox;
        let at = m.pos + Vec2::new(hb.x.mul_int(i32::from(m.facing)), hb.y);
        for d in 0..MAX_FIGHTERS {
            let fd = state.fighters[d];
            if !fd.active {
                continue;
            }
            let Some(proxy) = (0..MAX_FIGHTERS).find(|&i| i != d && !state.fighters[i].active)
            else {
                continue;
            };
            let counter = combat::counter_open(content, &fd, true);
            let intangible = combat::is_intangible(&fd);
            if intangible && counter.is_none() {
                continue;
            }
            let hurt = combat::body_hurtboxes(content, &fd);
            let shut = combat::intangible_parts(&fd, weapon_of(content, params_of(content, &fd)));
            let (touching, caught) = combat::reach(counter, intangible, &hb, &hurt, shut, at);
            if !touching {
                continue;
            }
            let saved = state.fighters[proxy];
            {
                let p = &mut state.fighters[proxy];
                p.pos = m.pos;
                p.facing = m.facing;
                p.vel = m.vel;
                p.percent = Fx::ZERO;
                p.stale = [0; crate::state::STALE_QUEUE];
                p.hitlag = 0;
            }
            let side = if hb.x < Fx::ZERO { -m.facing } else { m.facing };
            combat::apply_hit(
                state,
                content,
                proxy,
                d,
                &hb,
                side,
                m.pos,
                false,
                combat::STUN_NORMAL,
                0,
                (at, m.vel),
                caught,
            );
            state.fighters[proxy] = saved;
            let lag = combat::hitlag_frames(
                hb.damage,
                &content.rules,
                false,
                false,
                Fx::from_ratio(i32::from(hb.hitlag), 100),
            );
            let m = &mut state.run.mobs[n];
            m.struck = true;
            m.hitlag = lag;
            break;
        }
    }
}

/// Keeps every mob inside the rooms in play: past the left wall, short of the right end, and on its own side of a shut gate (flyers
/// do not collide with the stage, and nothing may slip through a wall).
pub(super) fn keep_in_window(state: &mut GameState, rc: &RunContent) {
    let run = &mut state.run;
    let thick = rc.rules.gate_thickness;
    let (wall, end) = (run.wall_x, run.window_end());
    let (gate_closed, gate) = (run.gate_closed, run.gate_x);
    for m in run.mobs.iter_mut() {
        if !m.active || m.mode == MODE_BURST {
            continue;
        }
        let r = kind_of(rc, m).radius;
        let (mut lo, mut hi) = (wall + r, end - r);
        if gate_closed {
            if m.pos.x > gate {
                lo = lo.max(gate + thick + r);
            } else {
                hi = hi.min(gate - r);
            }
        }
        let x = m.pos.x.max(lo).min(hi.max(lo));
        if x != m.pos.x {
            m.pos.x = x;
            m.vel.x = Fx::ZERO;
            m.kb_vel.x = Fx::ZERO;
        }
    }
}

/// Mobs gone from the world: above or below the blast zone, or out of the rooms in play; and mobs launched by the player off the
/// side of the screen. A mob the player hit lately is their kill; else it is just gone.
pub(super) fn leave_screen(state: &mut GameState, rc: &RunContent) {
    let r = &rc.rules;
    let run = &state.run;
    let (wall, end) = (run.wall_x, run.window_end());
    let left = run.cam_x - r.blast_margin;
    let right = run.cam_x + r.view_width + r.blast_margin;
    for n in 0..state.run.mobs.len() {
        let m = state.run.mobs[n];
        if !m.active {
            continue;
        }
        let launched = m.credit > 0 && (m.hitstun > 0 || m.mode == MODE_BURST);
        let out = m.pos.y < r.blast_bottom
            || m.pos.y > r.blast_top
            || m.pos.x < wall - r.blast_margin
            || m.pos.x > end + r.blast_margin
            || launched && (m.pos.x < left || m.pos.x > right);
        if !out {
            continue;
        }
        state.run.mobs[n].active = false;
        if m.mode != MODE_BURST && m.credit > 0 {
            pay_kill(&mut state.run, rc, m.kind);
        }
    }
}

/// Whether the player's attack or shot touches the shopkeeper this frame (inside the shop). It is never hurt: the shop only makes
/// it dizzy for a moment.
pub(super) fn keeper_bonked(state: &GameState, content: &Content, rc: &RunContent) -> bool {
    let shop = &rc.interior;
    let feet = Vec2::new(Fx::from_int(INTERIOR_X) + shop.keeper.x, shop.keeper.y);
    let centre = feet + Vec2::new(Fx::ZERO, shop.keeper_radius);
    let radius = shop.keeper_radius;
    let f = state.fighters[PLAYER];
    if f.active && matches!(f.state, S::Attack | S::LedgeAttack) && f.hitlag == 0 {
        let params = params_of(content, &f);
        let mv = weapon_of(content, params).get(f.move_id);
        let touched = combat::active_hitboxes(&f, mv, params.hitbox_scale)
            .into_iter()
            .any(|(_, hb, at)| {
                hb.kind == HIT_NORMAL
                    && combat::shapes_touch(
                        centre,
                        Fx::ZERO,
                        Fx::ZERO,
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
            && d.x.abs() <= radius + Fx::ONE
            && d.y.abs() <= radius + Fx::ONE
    })
}
