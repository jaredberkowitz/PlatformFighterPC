//! Grunts: the run's enemies (R2b, `RUNNER_WORLD.md` 4). Every grunt is fighter-shaped: a base-game body (`MobKind::body`), whose
//! hurtbox, weight, fall speed, walk and run speeds and jump it has, and a small kit of that body's own moves, whose real hitboxes,
//! frames and knockback it attacks with. Every attack is announced by a tell (a flash and a "!", `Tier::tell` frames) and then the
//! move's own wind-up.
//!
//! Hits taken use the versus formulas (`combat::knockback`, `hitstun_frames`, `hitlag_frames`, `launch_angle`); hits given go through
//! `combat::apply_hit` on a fighter slot no one plays, so shields, DI, crouch cancelling, armor and counters all work against them.
//! A grunt is a light entity in a fixed pool, not a fighter: simple ground physics (walk, run, hop, fall, land, slopes) through the
//! shared collision, and a small state machine per role:
//! * a **brawler** rushes you and jabs or forward-tilts;
//! * a **leaper** waits on high ground and leaps at you with an aerial;
//! * a **shieldbearer** guards toward you and pokes out of its guard: hits from the front are blocked (grab it, hit it from above,
//!   or cross it up before it turns), and a broken guard dazes it;
//! * a **spearman** pokes long and late along the ground (a forward tilt, or a dash attack from further out);
//! * a **lobber** throws its shot in an arc from high ground (a real projectile from a slot standing for its body: a reflector sends
//!   it back into the grunts);
//! * a **bruiser** walks in slowly and swings a big smash, with super armor through its tell and startup.

use super::data::{
    MobKind, RunContent, NO_MOVE, ROLE_BRUISER, ROLE_LEAPER, ROLE_LOBBER, ROLE_SHIELD, ROLE_SPEAR,
};
use super::{
    count_combo, pay_kill, Mob, INTERIOR_X, MODE_ATTACK, MODE_BURST, MODE_COOLDOWN, MODE_DAZED,
    MODE_EMERGE, MODE_GUARD, MODE_IDLE, MODE_LEAP, MODE_MOVE, MODE_WINDUP, PLAYER, SHOT_SLOT,
    VAN_DRIVING, VAN_HALF, VAN_HEIGHT, VAN_NONE, VAN_PARKED, VAN_WAITING,
};
use crate::collision;
use crate::combat::{self, params_of, weapon_of, HURT_CIRCLES};
use crate::content::{Content, FighterParams, Stage};
use crate::fixed::Fx;
use crate::moves::{
    Hitbox, Move, MoveId, ANGLE_AUTOLINK, ANGLE_PULL, ANGLE_VORTEX, HIT_GRAB, HIT_NORMAL, HIT_THROW,
};
use crate::state::{Fighter, FighterState as S, GameState, Projectile, NONE, REST_FLYING};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

/// A hit-stamp no swing of a fighter makes: the runaway van's (one hit on each grunt).
const VAN_STAMP: u32 = 2 << 16;

fn approach(v: Fx, target: Fx, delta: Fx) -> Fx {
    if v < target {
        (v + delta).min(target)
    } else {
        (v - delta).max(target)
    }
}

pub(super) fn kind_of<'a>(rc: &'a RunContent, m: &Mob) -> &'a MobKind {
    &rc.mobs[usize::from(m.kind).min(rc.mobs.len() - 1)]
}

/// A grunt's body: its roster fighter's numbers.
pub(super) fn body_of<'a>(content: &'a Content, k: &MobKind) -> &'a FighterParams {
    &content.fighters[usize::from(k.body).min(content.fighters.len() - 1)]
}

/// Move `slot` of a grunt's kit, from its body's weapon (none for an empty slot).
pub(super) fn move_of<'a>(content: &'a Content, k: &MobKind, slot: u8) -> Option<&'a Move> {
    let id = *k.moves.get(usize::from(slot))?;
    (id != NO_MOVE).then(|| weapon_of(content, body_of(content, k)).get(id))
}

/// A move's first and last active frames (0, 0 with no hitboxes).
fn active_frames(mv: &Move) -> (u16, u16) {
    let first = mv.hitboxes.iter().map(|h| h.start).min().unwrap_or(0);
    let last = mv.hitboxes.iter().map(|h| h.end).max().unwrap_or(0);
    (u16::from(first), u16::from(last))
}

/// The grunt as a fighter of its body, in the state its mode reads as (the hurtboxes and the move's hitboxes are read from it).
pub(super) fn as_fighter(m: &Mob, k: &MobKind) -> Fighter {
    let mut f = Fighter::spawn(m.pos, k.body, m.facing, m.platform, 0, Fx::ZERO);
    f.active = true;
    f.vel = m.vel;
    f.state = if m.mode == MODE_ATTACK {
        S::Attack
    } else if m.mode == MODE_GUARD {
        S::Shield
    } else if m.hitstun > 0 {
        S::Hitstun
    } else if m.platform == NONE {
        S::Airborne
    } else if m.vel.x != Fx::ZERO {
        S::Walk
    } else {
        S::Idle
    };
    if m.mode == MODE_ATTACK {
        f.move_id = k.moves[usize::from(m.move_slot).min(k.moves.len() - 1)];
        f.state_frame = m.frame;
    }
    f
}

/// The grunt's hurtboxes: its body's, as a fighter's.
pub(super) fn hurtboxes(content: &Content, m: &Mob, k: &MobKind) -> [(Vec2, Fx); HURT_CIRCLES] {
    let f = as_fighter(m, k);
    let p = body_of(content, k);
    combat::hurtboxes(&f, p, weapon_of(content, p))
}

/// Whether a hit of this shape at `at` touches the grunt's body.
pub(super) fn touches(hurt: &[(Vec2, Fx); HURT_CIRCLES], at: Vec2, hb: &Hitbox) -> bool {
    hurt.iter().any(|&(c, r)| {
        combat::shapes_touch(
            c,
            Fx::ZERO,
            Fx::ZERO,
            r,
            at,
            hb.half_width,
            hb.half_height,
            hb.radius,
        )
    })
}

/// Whether a grunt can be hit now (not bursting, nor still coming out of its door).
pub(super) fn hittable(m: &Mob) -> bool {
    m.active && m.mode != MODE_BURST && m.mode != MODE_EMERGE
}

/// Whether the grunt's armor is up (a bruiser: through the tell and the startup of its first move): a hit deals its damage but does
/// not flinch it.
pub(super) fn armored(content: &Content, m: &Mob, k: &MobKind) -> bool {
    if !k.armor || m.move_slot != 0 {
        return false;
    }
    match m.mode {
        MODE_WINDUP => true,
        MODE_ATTACK => move_of(content, k, 0).is_some_and(|mv| m.frame < active_frames(mv).0),
        _ => false,
    }
}

/// Whether a guarding grunt's shield takes a hit from `from` (the attacker's feet, or a shot) landing at `at`: only from in front, and
/// not from above (the attacker's feet above three quarters of the body, or the hit above its head).
pub(super) fn guard_blocks(content: &Content, m: &Mob, k: &MobKind, from: Vec2, at: Vec2) -> bool {
    if m.mode != MODE_GUARD || m.shield <= Fx::ZERO {
        return false;
    }
    let height = body_of(content, k).ecb_height;
    let in_front = (from.x - m.pos.x).mul_int(i32::from(m.facing)) >= Fx::ZERO;
    let above = from.y >= m.pos.y + height * Fx::from_ratio(3, 4) || at.y > m.pos.y + height;
    in_front && !above
}

/// A blocked hit: the guard loses the damage and the grunt slides back; a guard worn to nothing breaks and the grunt is dazed.
/// Returns the hitlag (the shield's).
pub(super) fn block(m: &mut Mob, hb: &Hitbox, side: i8, content: &Content, daze: u8) -> u8 {
    m.shield -= hb.damage;
    m.kb_vel = Vec2::new(
        (Fx::from_ratio(1, 10) + hb.damage * Fx::from_ratio(1, 100)).mul_int(i32::from(side)),
        Fx::ZERO,
    );
    m.hitstun = 0;
    let lag = combat::hitlag_frames(
        hb.damage,
        &content.rules,
        true,
        false,
        Fx::from_ratio(i32::from(hb.hitlag), 100),
    );
    m.hitlag = lag;
    if m.shield <= Fx::ZERO {
        m.shield = Fx::ZERO;
        m.mode = MODE_DAZED;
        m.timer = u16::from(daze.max(1));
        m.vel = Vec2::ZERO;
    }
    lag
}

/// A hit on a grunt: damage off its stamina, the percent the knockback reads, the launch, hitstun and hitlag (the attacker's too, for
/// a melee hit). `side` is the way 0 degrees points; `rage` the attacker's. With its armor up it takes the damage and the hitlag only.
/// Returns the hitlag.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_hit(
    m: &mut Mob,
    k: &MobKind,
    hb: &Hitbox,
    side: i8,
    rage: Fx,
    content: &Content,
    burst_frames: u8,
    armor: bool,
) -> u8 {
    let rules = &content.rules;
    let weight = body_of(content, k).weight;
    let damage = hb.damage;
    m.percent = (m.percent + damage).min(Fx::from_int(999));
    if !armor {
        let kb = if hb.fixed_kb > 0 {
            combat::fixed_knockback(hb.fixed_kb, weight, hb.base_knockback, hb.knockback_growth)
        } else {
            combat::knockback(
                m.percent,
                damage,
                weight,
                hb.base_knockback,
                hb.knockback_growth,
            ) * rage
        };
        // (The special angles of fighter moves, which carry or pull a fighter, send a grunt at 45 degrees; the autolink's grounded 80
        // is kept.)
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
            m.struck = 0;
        }
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
            if armor {
                // (Beaten through its armor: it still flies off.)
                m.kb_vel = Vec2::new(
                    Fx::from_ratio(1, 5).mul_int(i32::from(side)),
                    Fx::from_ratio(1, 5),
                );
            }
        }
    }
    lag
}

/// What a fighter's charged attack adds to its hit's damage (as `combat::resolve_hits` does for fighters).
fn charged(content: &Content, f: &Fighter, hb: &Hitbox) -> Fx {
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

/// The throw a fighter's grab gives a grunt it catches: its forward throw's hit (or the run's `grab_throw`).
fn throw_of(content: &Content, f: &Fighter, rc: &RunContent) -> Hitbox {
    weapon_of(content, params_of(content, f))
        .get(MoveId::FThrow as u8)
        .hitboxes
        .iter()
        .find(|h| h.kind == HIT_THROW)
        .copied()
        .unwrap_or(rc.rules.grab_throw)
}

/// The fighters' attacks and shots that reach a grunt this frame. A move hits a grunt once per hit group (the best priority box), as
/// it does a fighter; a shot once (a piercing one goes on). A guard in the way blocks (`guard_blocks`); armor takes the hit without
/// flinching; a grab throws the grunt (out of its guard too).
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
            let hurt = hurtboxes(content, &m, &k);
            let mut best: Option<(Hitbox, Vec2)> = None;
            let mut grab = false;
            for (_, hb, at) in combat::active_hitboxes(&f, mv, params.hitbox_scale) {
                if hb.push || groups & (1 << hb.group.min(7)) != 0 || !touches(&hurt, at, &hb) {
                    continue;
                }
                if hb.kind == HIT_GRAB {
                    grab = true;
                } else if hb.kind == HIT_NORMAL
                    && best.is_none_or(|(b, _)| hb.priority < b.priority)
                {
                    best = Some((hb, at));
                }
            }
            let m = &mut state.run.mobs[n];
            if grab && best.is_none() {
                // Caught: thrown with the fighter's forward throw, guard or no guard.
                let throw = throw_of(content, &f, rc);
                m.hit_stamp = stamp;
                m.hit_groups = 0xff;
                m.hit_age = f.state_frame;
                m.credit = rc.rules.credit_frames;
                if m.mode == MODE_GUARD {
                    m.mode = MODE_IDLE;
                }
                take_hit(m, &k, &throw, f.facing, rage, content, burst, false);
                let (died, kind) = (m.mode == MODE_BURST, m.kind);
                count_combo(&mut state.run);
                if died {
                    pay_kill(&mut state.run, rc, kind);
                }
                continue;
            }
            let Some((mut hb, at)) = best else {
                continue;
            };
            hb.damage = charged(content, &f, &hb);
            let side = if hb.x < Fx::ZERO { -f.facing } else { f.facing };
            m.hit_stamp = stamp;
            m.hit_groups = groups | (1 << hb.group.min(7));
            m.hit_age = f.state_frame;
            if guard_blocks(content, m, &k, f.pos, at) {
                let lag = block(m, &hb, side, content, rc.rules.daze_frames);
                freeze = freeze.max(lag);
                continue;
            }
            m.credit = rc.rules.credit_frames;
            let armor = armored(content, m, &k);
            let lag = take_hit(m, &k, &hb, side, rage, content, burst, armor);
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

/// Shots (projectiles) of the fighters reaching grunts (a lobber's shot a fighter reflected is the fighter's now).
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
        // (A lobber's own shot, not reflected, does not hit grunts.)
        if owner == SHOT_SLOT || !state.fighters[owner].active {
            continue;
        }
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
            let hurt = hurtboxes(content, &m, &k);
            if !touches(&hurt, pr.pos, &hb) {
                continue;
            }
            let m = &mut state.run.mobs[i];
            m.shot_mask |= bit;
            if guard_blocks(content, m, &k, pr.pos, pr.pos) {
                block(m, &hb, side, content, rc.rules.daze_frames);
            } else {
                m.credit = rc.rules.credit_frames;
                let armor = armored(content, m, &k);
                take_hit(m, &k, &hb, side, rage, content, burst, armor);
                let (died, kind) = (m.mode == MODE_BURST, m.kind);
                count_combo(&mut state.run);
                if died {
                    pay_kill(&mut state.run, rc, kind);
                }
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

/// Moves a grunt by its speed and launch, with gravity in the air and the stage in the way (a bursting one flies free). Returns true
/// if it met a wall.
fn physics(m: &mut Mob, p: &FighterParams, stage: &Stage, decay: Fx, solid: bool) -> bool {
    let launched = m.hitstun > 0 || m.mode == MODE_BURST;
    if launched {
        let speed = m.kb_vel.length();
        m.kb_vel = if speed > decay {
            m.kb_vel * ((speed - decay) / speed)
        } else {
            Vec2::ZERO
        };
    }
    if m.platform == NONE {
        m.vel.y = (m.vel.y - p.gravity).max(-p.max_fall_speed);
    }
    let total = m.vel + m.kb_vel;
    if !solid || m.mode == MODE_BURST {
        m.pos += total;
        return false;
    }
    let height = p.ecb_height;
    let wall = move_x(stage, &mut m.pos, p.ecb_half_width, height, total.x);
    if wall {
        m.vel.x = Fx::ZERO;
        m.kb_vel.x = -(m.kb_vel.x * Fx::HALF);
    }
    if let Some(pl) = collision::platform(stage, m.platform) {
        // On its ground it follows the surface (a slope too), and carries on where a slope joins other ground.
        if m.pos.x >= pl.left && m.pos.x <= pl.right && (pl.sloped() || pl.y == m.pos.y) {
            m.pos.y = pl.top_at(m.pos.x);
        } else if let Some(i) = collision::continuation(stage, m.platform, m.pos.x) {
            m.platform = i as i8;
            m.pos.y = stage.platforms[i].top_at(m.pos.x);
        } else {
            m.platform = NONE;
        }
        // (Jumping off it.)
        if m.vel.y > Fx::ZERO {
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

/// The frames of warning before an attack in the grunt's tier.
fn tell_of(rc: &RunContent, m: &Mob) -> u16 {
    rc.tiers
        .get(usize::from(m.tier))
        .map_or(18, |t| u16::from(t.tell))
        .max(1)
}

/// Starts the tell before move `slot`: it stops, flashes and shows a "!".
fn tell(m: &mut Mob, rc: &RunContent, slot: u8) {
    m.mode = MODE_WINDUP;
    m.timer = tell_of(rc, m);
    m.move_slot = slot;
    m.vel.x = Fx::ZERO;
}

/// Starts move `slot` of its kit (its own wind-up, then its hits).
fn begin_move(m: &mut Mob, content: &Content, k: &MobKind, slot: u8) {
    let total = move_of(content, k, slot).map_or(1, |mv| mv.total_frames.max(1));
    m.mode = MODE_ATTACK;
    m.move_slot = slot;
    m.frame = 0;
    m.timer = u16::from(total);
    m.struck = 0;
}

/// Every grunt's frame: hitlag, then the launch or its behaviour, then its movement.
pub(super) fn update(state: &mut GameState, content: &Content, rc: &RunContent) {
    let target = state.fighters[PLAYER];
    let player = (target.active && target.state != S::Respawn).then_some(target.pos);
    let decay = content.rules.knockback_decay;
    let stage = &content.stage;
    for n in 0..state.run.mobs.len() {
        let mut m = state.run.mobs[n];
        if !m.active {
            continue;
        }
        let k = *kind_of(rc, &m);
        let p = *body_of(content, &k);
        m.credit = m.credit.saturating_sub(1);
        if m.hitlag > 0 {
            m.hitlag -= 1;
            state.run.mobs[n] = m;
            continue;
        }
        if k.shield > Fx::ZERO && m.mode != MODE_DAZED {
            m.shield = (m.shield + rc.rules.shield_regen).min(k.shield);
        }
        match m.mode {
            MODE_EMERGE => {
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
            }
            MODE_BURST => {
                physics(&mut m, &p, stage, decay, false);
                m.timer = m.timer.saturating_sub(1);
                if m.timer == 0 {
                    m.active = false;
                }
            }
            _ if m.hitstun > 0 => {
                m.hitstun -= 1;
                if m.hitstun == 0 {
                    // Out of the launch: what is left of it carries on as its own speed.
                    m.vel += m.kb_vel;
                    m.kb_vel = Vec2::ZERO;
                    m.mode = MODE_MOVE;
                }
                physics(&mut m, &p, stage, decay, true);
            }
            _ => {
                let shot = behave(&mut m, &k, &p, content, rc, player);
                if let (Some(at), Some(pl)) = (shot, player) {
                    fire_shot(
                        state,
                        content,
                        rc,
                        &m,
                        &k,
                        pl + Vec2::new(Fx::ZERO, Fx::ONE),
                        at,
                    );
                }
            }
        }
        state.run.mobs[n] = m;
    }
}

/// Air control: drifts (at up to its body's air speed) to stand `short` short of `dx` away.
fn steer(m: &mut Mob, p: &FighterParams, dx: Fx, short: Fx) {
    let to = if dx > short {
        dx - short
    } else if dx < -short {
        dx + short
    } else {
        Fx::ZERO
    };
    let want = (to / Fx::from_int(12)).max(-p.air_speed).min(p.air_speed);
    m.vel.x = approach(m.vel.x, want, p.air_accel.max(Fx::from_ratio(1, 100)));
}

fn face(m: &mut Mob, dx: Fx) {
    if dx.abs() > Fx::from_ratio(3, 10) {
        m.facing = if dx > Fx::ZERO { 1 } else { -1 };
    }
}

/// A grunt's own frame (awake, not launched): its role's choices, its attack's frames, then its movement. Returns where a lobber's
/// shot leaves from, on the frame it throws.
fn behave(
    m: &mut Mob,
    k: &MobKind,
    p: &FighterParams,
    content: &Content,
    rc: &RunContent,
    player: Option<Vec2>,
) -> Option<Vec2> {
    let stage = &content.stage;
    let friction = p.walk_speed * Fx::from_ratio(1, 6);
    let grounded = m.platform != NONE;
    let mut shot = None;
    let mut walking = false;
    match m.mode {
        MODE_IDLE => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            if player.is_some_and(|pl| (pl.x - m.pos.x).abs() < k.aggro) {
                m.mode = if k.role == ROLE_SHIELD {
                    MODE_GUARD
                } else {
                    MODE_MOVE
                };
            }
        }
        MODE_MOVE | MODE_GUARD => {
            if k.role == ROLE_SHIELD && m.mode == MODE_MOVE {
                m.mode = MODE_GUARD;
            }
            walking = choose(m, k, p, rc, player, stage);
        }
        MODE_WINDUP => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                if k.role == ROLE_LEAPER && m.move_slot == NO_MOVE {
                    // The leap: up and toward you, the aerial on the way.
                    let to = player.map_or(Fx::ZERO, |pl| pl.x - m.pos.x);
                    let most = p.air_speed * Fx::from_ratio(6, 5);
                    m.vel.x = (to / Fx::from_int(40)).max(-most).min(most);
                    m.vel.y = p.full_hop_velocity * Fx::from_ratio(5, 4);
                    m.platform = NONE;
                    m.mode = MODE_LEAP;
                } else {
                    begin_move(m, content, k, m.move_slot);
                }
            }
        }
        MODE_LEAP => {
            let (dx, dy) =
                player.map_or((Fx::ZERO, Fx::ZERO), |pl| (pl.x - m.pos.x, pl.y - m.pos.y));
            face(m, dx);
            steer(m, p, dx, Fx::ZERO);
            // Over you on the way down, the down air; level with you and near, the forward air.
            let above = dy < -Fx::from_ratio(3, 2);
            if above && m.vel.y <= Fx::ZERO && dx.abs() < Fx::from_ratio(3, 2) {
                begin_move(m, content, k, 0);
            } else if !above && dy.abs() < Fx::from_int(2) && dx.abs() < k.reach2 {
                begin_move(m, content, k, 1);
            }
        }
        MODE_ATTACK => {
            m.frame = m.frame.saturating_add(1);
            if !grounded && k.role == ROLE_LEAPER {
                // Its aerial drifts onto you: the down air right over you, the forward air just short of you.
                let dx = player.map_or(Fx::ZERO, |pl| pl.x - m.pos.x);
                let short = if m.move_slot == 0 { Fx::ZERO } else { Fx::ONE };
                steer(m, p, dx, short);
            }
            let mv = move_of(content, k, m.move_slot);
            let (first, last) = mv.map_or((0, 0), active_frames);
            if grounded {
                if k.lunge > Fx::ZERO && m.frame >= first && m.frame <= last {
                    m.vel.x = k.lunge.mul_int(i32::from(m.facing));
                } else {
                    m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
                }
            }
            if k.role == ROLE_LOBBER && m.move_slot == 0 && m.frame == u16::from(k.shot_frame) {
                let pr = mv.and_then(|mv| mv.projectile);
                shot = Some(
                    pr.map_or(m.pos + Vec2::new(Fx::ZERO, Fx::from_int(2)), |s| {
                        m.pos
                            + Vec2::new(
                                (s.x * p.hitbox_scale).mul_int(i32::from(m.facing)),
                                s.y * p.hitbox_scale,
                            )
                    }),
                );
            }
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = MODE_COOLDOWN;
                m.timer = u16::from(k.cooldown);
            }
        }
        MODE_DAZED => {
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.shield = k.shield;
                m.mode = MODE_MOVE;
            }
        }
        _ => {
            // Cooling down after an attack.
            m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
            m.timer = m.timer.saturating_sub(1);
            if m.timer == 0 {
                m.mode = if k.role == ROLE_SHIELD {
                    MODE_GUARD
                } else {
                    MODE_MOVE
                };
            }
        }
    }
    let was_air = m.platform == NONE;
    let wall = physics(m, p, stage, Fx::ZERO, true);
    // Landing out of a leap or an aerial ends it.
    if was_air
        && m.platform != NONE
        && (m.mode == MODE_LEAP || m.mode == MODE_ATTACK && k.role == ROLE_LEAPER)
    {
        m.mode = MODE_COOLDOWN;
        m.timer = u16::from(k.cooldown);
        m.vel.x = Fx::ZERO;
    }
    // Walking into a step, it hops.
    if wall && walking && m.platform != NONE {
        m.vel.y = p.full_hop_velocity;
        m.platform = NONE;
    }
    shot
}

/// A grunt's choice while it moves (or guards): where it walks, and which attack it tells. Returns true while it walks.
fn choose(
    m: &mut Mob,
    k: &MobKind,
    p: &FighterParams,
    rc: &RunContent,
    player: Option<Vec2>,
    stage: &Stage,
) -> bool {
    let friction = p.walk_speed * Fx::from_ratio(1, 6);
    let grounded = m.platform != NONE;
    let Some(pl) = player else {
        m.vel.x = approach(m.vel.x, Fx::ZERO, friction);
        return false;
    };
    let dx = pl.x - m.pos.x;
    let dy = pl.y - m.pos.y;
    let far = dx.abs();
    let level = dy.abs() < Fx::from_int(2);
    if m.mode == MODE_GUARD {
        // It turns to face you only some frames after you cross it up.
        let behind = dx.mul_int(i32::from(m.facing)) < Fx::ZERO && far > Fx::from_ratio(3, 10);
        if behind {
            m.aux = m.aux.saturating_add(1);
            if m.aux >= u16::from(rc.rules.guard_turn) {
                m.facing = -m.facing;
                m.aux = 0;
            }
        } else {
            m.aux = 0;
        }
    } else {
        face(m, dx);
    }
    // Where it wants to stand: in reach of its first move (a lobber keeps its distance; a leaper stays on its roof).
    let mut speed = p.walk_speed * Fx::from_ratio(i32::from(k.walk), 100);
    if k.run > 0 && far > Fx::from_int(6) {
        speed = p.run_speed * Fx::from_ratio(i32::from(k.run), 100);
    }
    if m.mode == MODE_GUARD {
        speed = speed * Fx::HALF;
    }
    let stop = match k.role {
        ROLE_LOBBER => k.reach * Fx::HALF,
        ROLE_LEAPER => k.reach,
        _ => k.reach * Fx::from_ratio(4, 5),
    };
    let toward: i32 = if dx > Fx::ZERO { 1 } else { -1 };
    let mut want = if far > stop && (m.mode != MODE_GUARD || toward == i32::from(m.facing)) {
        speed.mul_int(toward)
    } else {
        Fx::ZERO
    };
    if grounded && want != Fx::ZERO {
        // Never off an edge on purpose.
        let half = p.ecb_half_width;
        let ahead = Vec2::new(
            m.pos.x + (half + Fx::from_ratio(3, 10)).mul_int(toward),
            // (A little above the feet, so ground rising up a slope ahead counts as floor.)
            m.pos.y + Fx::from_ratio(6, 10),
        );
        let floor = collision::surface_below(stage, ahead, false);
        if floor.is_none_or(|(_, d)| d > Fx::from_ratio(3, 2)) {
            want = Fx::ZERO;
        }
    }
    m.vel.x = approach(
        m.vel.x,
        want,
        speed * Fx::from_ratio(1, 6) + Fx::from_ratio(1, 200),
    );
    if !grounded {
        return false;
    }
    // The attacks, by role.
    let facing_you = dx.mul_int(i32::from(m.facing)) >= Fx::ZERO;
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
    if let Some(slot) = slot.filter(|_| facing_you) {
        if slot == NO_MOVE || move_of_any(k, slot) {
            tell(m, rc, slot);
            return false;
        }
    }
    want != Fx::ZERO
}

/// Whether the kit has a move in `slot`.
fn move_of_any(k: &MobKind, slot: u8) -> bool {
    k.moves
        .get(usize::from(slot))
        .is_some_and(|&id| id != NO_MOVE)
}

/// A lobber's shot: its first move's projectile, fired from `from` (on the slot standing for its body, `SHOT_SLOT`) in the arc that
/// comes down on `target` after `MobKind::shot_time` frames, at its tier's damage. A reflector turns it on the grunts.
fn fire_shot(
    state: &mut GameState,
    content: &Content,
    rc: &RunContent,
    m: &Mob,
    k: &MobKind,
    target: Vec2,
    from: Vec2,
) {
    let Some(mv) = move_of(content, k, 0) else {
        return;
    };
    let Some(spec) = mv.projectile else {
        return;
    };
    let t = Fx::from_int(i32::from(k.shot_time.max(1)));
    let d = target - from;
    let g = spec.arc.gravity;
    let vx = (d.x / t).max(-Fx::HALF).min(Fx::HALF);
    let vy = (d.y / t + g * t * Fx::HALF)
        .max(-Fx::from_ratio(3, 5))
        .min(Fx::from_ratio(9, 10));
    let power = rc
        .tiers
        .get(usize::from(m.tier))
        .map_or(100, |t| t.damage)
        .max(1);
    if let Some(slot) = state.projectiles.iter_mut().find(|p| !p.active) {
        *slot = Projectile {
            active: true,
            owner: SHOT_SLOT as u8,
            origin: SHOT_SLOT as u8,
            power,
            move_id: k.moves[0],
            pos: from,
            vel: Vec2::new(vx, vy),
            life: spec.life,
            size: 100,
            facing: m.facing,
            kind: spec.tool.kind,
            ..Projectile::default()
        };
    }
}

/// Hits the fighters with `hb` at `at` through the versus hit (`combat::apply_hit`), from a grunt or a hazard standing in on a
/// fighter slot no one plays (restored right after; `body` its char): `pos`, `facing` and `vel` are what the hit reads, with no rage
/// or staleness of its own. Returns true if it hit someone.
pub(super) fn hit_fighters(
    state: &mut GameState,
    content: &Content,
    body: u8,
    (pos, facing, vel): (Vec2, i8, Vec2),
    hb: &Hitbox,
    at: Vec2,
) -> bool {
    for d in 0..MAX_FIGHTERS {
        let fd = state.fighters[d];
        if !fd.active {
            continue;
        }
        let Some(proxy) =
            (0..MAX_FIGHTERS).find(|&i| i != d && i != SHOT_SLOT && !state.fighters[i].active)
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
        let (touching, caught) = combat::reach(counter, intangible, hb, &hurt, shut, at);
        if !touching {
            continue;
        }
        let saved = state.fighters[proxy];
        {
            let p = &mut state.fighters[proxy];
            p.char_id = body;
            p.pos = pos;
            p.facing = facing;
            p.vel = vel;
            p.percent = Fx::ZERO;
            p.stale = [0; crate::state::STALE_QUEUE];
            p.hitlag = 0;
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
        return true;
    }
    false
}

/// Grunts mid-attack hitting the fighters: the move's live hitboxes this frame (one hit a group), at the tier's damage.
pub(super) fn attacks(state: &mut GameState, content: &Content, rc: &RunContent) {
    for n in 0..state.run.mobs.len() {
        let m = state.run.mobs[n];
        if !m.active || m.mode != MODE_ATTACK || m.hitlag > 0 || m.hitstun > 0 {
            continue;
        }
        let k = *kind_of(rc, &m);
        let Some(mv) = move_of(content, &k, m.move_slot) else {
            continue;
        };
        let p = body_of(content, &k);
        let pct = rc
            .tiers
            .get(usize::from(m.tier))
            .map_or(100, |t| i32::from(t.damage));
        let fake = as_fighter(&m, &k);
        let live: Vec<(Hitbox, Vec2)> = combat::active_hitboxes(&fake, mv, p.hitbox_scale)
            .filter(|(_, hb, _)| hb.kind == HIT_NORMAL && !hb.push)
            .map(|(_, hb, at)| (hb, at))
            .collect();
        for (mut hb, at) in live {
            let bit = 1u8 << hb.group.min(7);
            if state.run.mobs[n].struck & bit != 0 {
                continue;
            }
            hb.damage = hb.damage * Fx::from_int(pct) / Fx::from_int(100);
            if !hit_fighters(state, content, k.body, (m.pos, m.facing, m.vel), &hb, at) {
                continue;
            }
            let lag = combat::hitlag_frames(
                hb.damage,
                &content.rules,
                false,
                false,
                Fx::from_ratio(i32::from(hb.hitlag), 100),
            );
            let g = &mut state.run.mobs[n];
            g.struck |= bit;
            g.hitlag = lag;
            break;
        }
    }
}

/// The runaway van (R2b, the first rare event): it waits at its street's start until the player lands on its roof, then drives
/// off (speeding up to `van_speed`) to the street's end, carrying its rider (untouchable while riding) and running into the grunts
/// in its way (each once: the player's kills); then it parks.
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
        for n in 0..state.run.mobs.len() {
            let m = state.run.mobs[n];
            if !hittable(&m) || m.hit_stamp == VAN_STAMP || (m.pos.x - v.x).abs() > reach {
                continue;
            }
            let k = *kind_of(rc, &m);
            let height = body_of(content, &k).ecb_height;
            if m.pos.y >= v.y + VAN_HEIGHT || m.pos.y + height <= v.y {
                continue;
            }
            let g = &mut state.run.mobs[n];
            g.hit_stamp = VAN_STAMP;
            g.credit = rules.credit_frames;
            if g.mode == MODE_GUARD {
                g.mode = MODE_IDLE;
            }
            take_hit(
                g,
                &k,
                &rules.van_hit,
                1,
                Fx::ONE,
                content,
                rules.burst_frames,
                false,
            );
            let (died, kind) = (g.mode == MODE_BURST, g.kind);
            v.hits = v.hits.saturating_add(1);
            count_combo(&mut state.run);
            if died {
                pay_kill(&mut state.run, rc, kind);
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

/// Keeps every grunt inside the rooms in play: past the left wall, short of the right end, and on its own side of a shut gate.
pub(super) fn keep_in_window(state: &mut GameState, content: &Content, rc: &RunContent) {
    let run = &mut state.run;
    let thick = rc.rules.gate_thickness;
    let (wall, end) = (run.wall_x, run.window_end());
    let (gate_closed, gate) = (run.gate_closed, run.gate_x);
    for m in run.mobs.iter_mut() {
        if !m.active || m.mode == MODE_BURST {
            continue;
        }
        let r = body_of(content, kind_of(rc, m)).ecb_half_width;
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

/// Grunts gone from the world: above or below the blast zone, or out of the rooms in play; and grunts launched by the player off the
/// side of the screen. A grunt the player hit lately is their kill; else it is just gone.
pub(super) fn leave_screen(state: &mut GameState, rc: &RunContent) {
    let r = &rc.rules;
    let run = &state.run;
    let (wall, end) = (run.wall_x, run.window_end());
    let (blast_bottom, blast_top) = run.blast_y(r);
    let left = run.cam_x - r.blast_margin;
    let right = run.cam_x + r.view_width + r.blast_margin;
    for n in 0..state.run.mobs.len() {
        let m = state.run.mobs[n];
        if !m.active {
            continue;
        }
        let launched = m.credit > 0 && (m.hitstun > 0 || m.mode == MODE_BURST);
        let out = m.pos.y < blast_bottom
            || m.pos.y > blast_top
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
