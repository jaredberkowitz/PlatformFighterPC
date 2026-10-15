//! Orbs put on a fighter (E59, sim 128: the reference's shadow orbs; the numbers are the beam move's `moves::Orbs`).
//!
//! A beam's hit puts orbs on the fighter it hit ([`place`]); they are projectiles of the beam's move marked `Projectile::orb`, held by
//! that fighter (`Projectile::held`). The fighter's clock (`Fighter::orb_clock`, [`tick`]) counts down to the strike; meanwhile the
//! orbs circle its chest and hit bystanders they touch ([`update_orb`]); then every orb on it flies into its chest and strikes. A
//! striking orb that is reflected or pocketed flies on as a plain shot (`ORB_LOOSE`, through `combat::update_projectiles` with
//! [`loose_spec`]). A knock-out of the fighter takes its orbs away ([`clear_target`]).

use crate::combat::{self, Shape};
use crate::content::Content;
use crate::fixed::Fx;
use crate::moves::{Orbs, ProjectileArc, ProjectileSpawn, Tool, MAX_PHASES};
use crate::state::{GameState, Projectile, ORB_CIRCLE, ORB_LOOSE, ORB_STRIKE, REST_POCKETED};
use crate::trig::{self, Angle};
use crate::vec2::Vec2;
use crate::MAX_BODIES;

/// The orbs block of the move that fired `pr` (a beam or one of its orbs).
fn orbs_of(state: &GameState, content: &Content, pr: &Projectile) -> Option<Orbs> {
    let origin = usize::from(pr.origin).min(MAX_BODIES - 1);
    let p = combat::params_of(content, &state.fighters[origin]);
    combat::weapon_of(content, p).get(pr.move_id).orbs
}

/// An orb on a fighter (circling or striking).
fn on_fighter(p: &Projectile) -> bool {
    p.active && (p.orb == ORB_CIRCLE || p.orb == ORB_STRIKE)
}

/// Where fighter `d`'s orbs aim: its chest.
fn centre(state: &GameState, content: &Content, d: usize) -> Vec2 {
    let f = &state.fighters[d];
    f.pos
        + Vec2::new(
            Fx::ZERO,
            combat::chest_height(combat::params_of(content, f)),
        )
}

/// Beam `n` hit fighter `d`: puts its orbs on (within the limits) and starts or lengthens `d`'s clock.
pub fn place(state: &mut GameState, content: &Content, n: usize, d: usize) {
    let beam = state.projectiles[n];
    let Some(o) = orbs_of(state, content, &beam) else {
        return;
    };
    // None on a fighter with another firer's orbs, or whose orbs are striking.
    let mut on_d = 0u8;
    for p in state
        .projectiles
        .iter()
        .filter(|p| on_fighter(p) && usize::from(p.held) == d)
    {
        if p.origin != beam.origin || p.orb == ORB_STRIKE {
            return;
        }
        on_d += 1;
    }
    let out = state
        .projectiles
        .iter()
        .filter(|p| on_fighter(p) && p.origin == beam.origin)
        .count()
        .min(255) as u8;
    let want = o.placed(beam.charge, beam.owner != beam.origin);
    let room = o
        .per_target
        .saturating_sub(on_d)
        .min(o.per_owner.saturating_sub(out));
    let k = want.min(room);
    {
        let f = &mut state.fighters[d];
        if on_d == 0 {
            if k == 0 {
                return;
            }
            f.orb_clock = o.wait.max(1);
            f.orb_age = 0;
        } else {
            // (A later hit lengthens the wait, never past `cap` from the first.)
            let left = o.cap.saturating_sub(f.orb_age).max(1);
            f.orb_clock = f
                .orb_clock
                .saturating_add(o.extension(beam.charge))
                .min(left);
        }
    }
    let at = centre(state, content, d);
    for _ in 0..k {
        let Some(slot) = state.projectiles.iter_mut().find(|p| !p.active) else {
            break;
        };
        *slot = Projectile {
            active: true,
            owner: beam.owner,
            origin: beam.origin,
            power: 100,
            move_id: beam.move_id,
            pos: at,
            life: 255,
            size: 100,
            facing: beam.facing,
            stale: beam.stale,
            charge: beam.charge,
            orb: ORB_CIRCLE,
            held: d as u8,
            ..Projectile::default()
        };
    }
}

/// One frame of every fighter's orb clock (before the projectiles move): a clock with no orbs left circling stops; one that runs out
/// sends every orb on the fighter to strike.
pub fn tick(state: &mut GameState) {
    for d in 0..MAX_BODIES {
        if state.fighters[d].orb_clock == 0 {
            continue;
        }
        let circling = state
            .projectiles
            .iter()
            .any(|p| p.active && p.orb == ORB_CIRCLE && usize::from(p.held) == d);
        let f = &mut state.fighters[d];
        if !circling {
            f.orb_clock = 0;
            f.orb_age = 0;
            continue;
        }
        f.orb_age = f.orb_age.saturating_add(1);
        f.orb_clock -= 1;
        if f.orb_clock > 0 {
            continue;
        }
        f.orb_age = 0;
        for p in state
            .projectiles
            .iter_mut()
            .filter(|p| p.active && p.orb == ORB_CIRCLE && usize::from(p.held) == d)
        {
            p.orb = ORB_STRIKE;
            p.age = 0;
            p.hit_mask = 0;
            p.hit_age = [0; MAX_BODIES];
        }
    }
}

/// A knock-out of fighter `d`: the orbs on it are gone and its clock stops.
pub fn clear_target(state: &mut GameState, d: usize) {
    for p in state
        .projectiles
        .iter_mut()
        .filter(|p| on_fighter(p) && usize::from(p.held) == d)
    {
        p.active = false;
    }
    state.fighters[d].orb_clock = 0;
    state.fighters[d].orb_age = 0;
}

/// The plain shot a loose orb flies as: the beam's spawn with the orbs' strike hit, straight on, used up by a hit.
pub fn loose_spec(spec: &ProjectileSpawn, o: &Orbs) -> ProjectileSpawn {
    let mut s = *spec;
    s.hitbox = o.strike_hit;
    s.end_damage = o.strike_hit.damage;
    s.phases = [None; MAX_PHASES];
    s.charged = None;
    s.burst = None;
    s.pierce = false;
    s.rehit = 0;
    s.arc = ProjectileArc::NONE;
    s.tool = Tool::NONE;
    s
}

/// One frame of orb `n`, circling or striking: it moves, may be pocketed or reflected (striking), and hits.
pub fn update_orb(state: &mut GameState, content: &Content, n: usize) {
    let pr = state.projectiles[n];
    let Some(o) = orbs_of(state, content, &pr) else {
        state.projectiles[n].active = false;
        return;
    };
    let d = usize::from(pr.held).min(MAX_BODIES - 1);
    if !state.fighters[d].active {
        state.projectiles[n].active = false;
        return;
    }
    let owner = usize::from(pr.owner).min(MAX_BODIES - 1);
    let origin = usize::from(pr.origin).min(MAX_BODIES - 1);
    let scale = combat::params_of(content, &state.fighters[origin]).hitbox_scale;
    let at = centre(state, content, d);
    // (The age wraps while circling: the per-target rehit reads differences.)
    let age = pr.age.wrapping_add(1);
    let circling = pr.orb == ORB_CIRCLE;
    let pos = if circling {
        // Spread evenly round the chest, `radius` out, a turn every `period` frames.
        let mut k = 0u32;
        let mut count = 0u32;
        for (i, p) in state.projectiles.iter().enumerate() {
            if p.active && p.orb == ORB_CIRCLE && usize::from(p.held) == d {
                if i < n {
                    k += 1;
                }
                count += 1;
            }
        }
        let period = u32::from(o.period.max(1));
        let turn = (state.frame % period) * 4096 / period + k * 4096 / count.max(1);
        let a = Angle::from_raw((turn % 4096) as u16);
        let r = o.radius * scale;
        at + Vec2::new(r * trig::cos(a), r * trig::sin(a))
    } else {
        if age > o.strike {
            state.projectiles[n].active = false;
            return;
        }
        // Into the chest, arriving on the strike's last frame.
        let left = i32::from(o.strike - age) + 1;
        pr.pos + (at - pr.pos) * Fx::from_ratio(1, left)
    };
    let vel = pos - pr.pos;
    {
        let p = &mut state.projectiles[n];
        p.pos = pos;
        p.vel = vel;
        p.age = age;
    }
    let (mut hb, pierce, rehit) = if circling {
        (o.orbit_hit, true, o.orbit_rehit)
    } else {
        (o.strike_hit, false, 0)
    };
    hb.radius = hb.radius * scale;
    hb.half_height = hb.half_height * scale;
    hb.half_width = hb.half_width * scale;
    let shape = Shape {
        half_w: hb.half_width,
        half_h: hb.half_height,
        radius: hb.radius,
    };
    if !circling {
        // A striking orb can be put away in a pocket (it comes out as a plain shot) or turned round by a reflector.
        if let Some(pd) = combat::pocket_touching(state, content, owner, pos, shape) {
            let away = state.fighters[pd].facing;
            let p = &mut state.projectiles[n];
            p.rest = REST_POCKETED;
            p.orb = ORB_LOOSE;
            p.vel = Vec2::new(o.loose_speed.mul_int(i32::from(away)), Fx::ZERO);
            p.life = o.loose_life.max(1);
            state.fighters[pd].pocket = n as i8;
            state.fighters[pd].pocket_item = false;
            return;
        }
        combat::reflector_detects(state, content, owner, pos, shape);
        let damage = hb.damage * Fx::from_int(i32::from(pr.power)) / Fx::from_int(100);
        if let Some((r, rf)) = combat::reflector_touching(state, content, owner, pos, shape)
            .filter(|(_, rf)| rf.max_damage == Fx::ZERO || damage <= rf.max_damage)
        {
            let dir = state.fighters[r].facing;
            let speed = o.loose_speed * Fx::from_ratio(i32::from(rf.speed_percent), 100);
            let p = &mut state.projectiles[n];
            p.owner = r as u8;
            p.power = rf.damage_percent;
            p.orb = ORB_LOOSE;
            p.vel = Vec2::new(speed.mul_int(i32::from(dir)), Fx::ZERO);
            p.age = 0;
            p.life =
                ((u16::from(o.loose_life) * u16::from(rf.life_percent)) / 100).clamp(1, 255) as u8;
            p.hit_mask = 0;
            p.hit_age = [0; MAX_BODIES];
            p.facing = dir;
            return;
        }
    }
    hb.damage = hb.damage * Fx::from_int(i32::from(pr.power)) / Fx::from_int(100);
    for v in 0..MAX_BODIES {
        let fv = &state.fighters[v];
        // (A circling orb leaves the fighter it circles alone; nothing hits its owner.)
        if !combat::rivals(owner, v)
            || (circling && v == d)
            || !fv.active
            || !combat::hits_target(&hb, fv)
        {
            continue;
        }
        let counter = combat::counter_open(content, fv, false);
        let intangible = combat::is_intangible(fv);
        if intangible && counter.is_none() {
            continue;
        }
        let bit = combat::hit_bit(v, hb.group);
        let seen = state.projectiles[n].hit_mask & bit != 0;
        if seen && (rehit == 0 || age.wrapping_sub(state.projectiles[n].hit_age[v]) < rehit) {
            continue;
        }
        let pv = combat::params_of(content, fv);
        let blocked = !intangible
            && !hb.unblockable
            && combat::passive_shield_blocks(fv, pv, pos, shape, vel);
        let (touching, caught) = if blocked {
            (false, false)
        } else {
            let w = combat::weapon_of(content, pv);
            combat::reach(
                counter,
                intangible,
                &hb,
                &combat::body_hurtboxes(content, fv),
                combat::intangible_parts(fv, w),
                pos,
            )
        };
        if blocked {
            combat::passive_block(&mut state.fighters[v], &content.rules, &hb);
        } else if touching {
            // A circling orb sends the way it is going; a striking one away from its owner.
            let travel = if circling {
                if vel.x < Fx::ZERO {
                    -1
                } else {
                    1
                }
            } else {
                let (vx, ox) = (state.fighters[v].pos.x, state.fighters[owner].pos.x);
                if vx > ox {
                    1
                } else if vx < ox {
                    -1
                } else {
                    pr.facing
                }
            };
            let facing = combat::launch_side(
                hb.angle_type,
                travel,
                travel,
                travel,
                pos.x,
                state.fighters[v].pos.x,
            );
            combat::apply_hit(
                state,
                content,
                owner,
                v,
                &hb,
                facing,
                pos,
                false,
                combat::STUN_PROJECTILE,
                pr.stale,
                (pos, vel),
                caught,
                true,
            );
        } else {
            continue;
        }
        let p = &mut state.projectiles[n];
        p.hit_mask |= bit;
        p.hit_age[v] = age;
        if !pierce {
            p.active = false;
            return;
        }
        p.orb_hits = p.orb_hits.saturating_add(1);
        if p.orb_hits >= o.orbit_hits {
            p.active = false;
            return;
        }
    }
    // A strike that has reached the chest without a hit (a dodge) is spent.
    if !circling && age >= o.strike {
        state.projectiles[n].active = false;
    }
}
