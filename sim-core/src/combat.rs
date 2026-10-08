//! Hit detection and damage. Runs once per frame after every fighter has moved.
//!
//! Hitboxes and hurtboxes are circles. Hurtboxes come from the fighter's ECB data (never from the mesh);
//! hitboxes come from the weapon's move data. All maths is fixed point.
//!
//! Formulas follow the reference game's published knockback model:
//! `KB = ((((p/10 + p*d/20) * 200/(w+100) * 1.4) + 18) * g/100) + b`, where `p` is the percent after the
//! hit, `d` the damage, `w` the weight, `g` the knockback growth and `b` the base knockback.
//! Launch speed is `KB * 0.03` reference units per frame; hitstun is `KB * 0.4` frames; hitlag is
//! `floor(d/3 + 4)` frames for both fighters.

use crate::collision;
use crate::content::{Content, FighterParams, Ruleset};
use crate::fighter;
use crate::fixed::Fx;
use crate::grab;
use crate::moves::{Hitbox, Move, MoveId, Reflector, Weapon, HIT_GRAB, HIT_PUMMEL, HIT_THROW};
use crate::scripting;
use crate::state::{Fighter, FighterState as S, GameState, Projectile, NONE};
use crate::trig::Angle;
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

pub const HURT_CIRCLES: usize = 3;

pub fn params_of<'a>(content: &'a Content, f: &Fighter) -> &'a FighterParams {
    let idx = usize::from(f.char_id).min(content.fighters.len().saturating_sub(1));
    &content.fighters[idx]
}

pub fn weapon_of<'a>(content: &'a Content, p: &FighterParams) -> &'a Weapon {
    let idx = usize::from(p.weapon).min(content.weapons.len().saturating_sub(1));
    &content.weapons[idx]
}

/// Three circles stacked up the body, sized from the ECB.
pub fn hurtboxes(f: &Fighter, p: &FighterParams) -> [(Vec2, Fx); HURT_CIRCLES] {
    let radius = p.ecb_half_width * Fx::from_ratio(17, 20);
    let at = |twentieths: i32| {
        Vec2::new(
            f.pos.x,
            f.pos.y + p.ecb_height * Fx::from_ratio(twentieths, 20),
        )
    };
    [(at(6), radius), (at(11), radius), (at(16), radius)]
}

/// True while a fighter cannot be hit.
pub fn is_intangible(f: &Fighter) -> bool {
    !f.active
        || f.invuln > 0
        || f.ledge_invuln > 0
        || (f.state == S::AirDodge && (4..=28).contains(&f.state_frame))
}

/// The move's hitboxes that are active this frame, with their world-space centres.
///
/// `scale` is the fighter's `hitbox_scale`: the returned hitboxes already have their position and radius scaled by it.
pub fn active_hitboxes<'a>(
    f: &'a Fighter,
    mv: &'a Move,
    scale: Fx,
) -> impl Iterator<Item = (usize, Hitbox, Vec2)> + 'a {
    let frame = u8::try_from(f.state_frame).unwrap_or(u8::MAX);
    let facing = i32::from(f.facing);
    mv.hitboxes
        .iter()
        .enumerate()
        .filter(move |(_, hb)| hb.start <= frame && frame <= hb.end)
        .map(move |(i, hb)| {
            let mut hb = *hb;
            hb.x = hb.x * scale;
            hb.y = hb.y * scale;
            hb.radius = hb.radius * scale;
            let center = Vec2::new(f.pos.x + hb.x.mul_int(facing), f.pos.y + hb.y);
            (i, hb, center)
        })
}

fn overlaps(c1: Vec2, r1: Fx, c2: Vec2, r2: Fx) -> bool {
    let reach = r1 + r2;
    (c1 - c2).length_sq() <= reach * reach
}

/// Knockback in reference units. `percent` is the defender's percent *after* the hit.
pub fn knockback(percent: Fx, damage: Fx, weight: Fx, base: i16, growth: i16) -> Fx {
    let mut kb = percent * Fx::from_ratio(1, 10) + (percent * damage) * Fx::from_ratio(1, 20);
    kb = kb * (Fx::from_int(200) / (weight + Fx::from_int(100)));
    kb = kb * Fx::from_ratio(7, 5) + Fx::from_int(18);
    kb * Fx::from_ratio(i32::from(growth), 100) + Fx::from_int(i32::from(base))
}

pub fn hitlag_frames(damage: Fx) -> u8 {
    ((damage / Fx::from_int(3)) + Fx::from_int(4))
        .floor_int()
        .clamp(1, 30) as u8
}

pub fn hitstun_frames(kb: Fx, mult: Fx) -> u16 {
    (kb * mult * Fx::from_ratio(2, 5)).floor_int().clamp(0, 600) as u16
}

/// World launch angle: mirrored when the attacker faces left, with the 361 "Sakurai" angle resolved.
pub fn launch_angle(degrees: i16, attacker_facing: i8, defender_grounded: bool, kb: Fx) -> Angle {
    let mut d = i32::from(degrees);
    if d == 361 {
        d = if defender_grounded && kb < Fx::from_int(32) {
            0
        } else {
            44
        };
    }
    if attacker_facing < 0 {
        d = 180 - d;
    }
    Angle::from_degrees(d)
}

/// The bit that records "this attacker already hit fighter `d` with hitbox group `g`".
fn hit_bit(d: usize, group: u8) -> u8 {
    1 << (d + MAX_FIGHTERS * usize::from(group.min(1)))
}

/// Finds and applies this frame's hits. Everything is collected before anything changes, so a trade
/// (both fighters hitting each other on the same frame) works, and ties go to the lower player index.
///
/// A move hits each target at most once per hitbox *group*; groups are how a move has separate hits.
/// The loops need the attacker and defender indices themselves (hit masks, tie-breaking), not just items.
#[allow(clippy::needless_range_loop)]
pub fn resolve_hits(state: &mut GameState, content: &Content) {
    let mut chosen: [[Option<Hitbox>; MAX_FIGHTERS]; MAX_FIGHTERS] =
        [[None; MAX_FIGHTERS]; MAX_FIGHTERS];
    for a in 0..MAX_FIGHTERS {
        let fa = &state.fighters[a];
        if !matches!(fa.state, S::Attack | S::LedgeAttack) || fa.hitlag > 0 {
            continue;
        }
        let mv = weapon_of(content, params_of(content, fa)).get(fa.move_id);
        for d in 0..MAX_FIGHTERS {
            if a == d {
                continue;
            }
            let fd = &state.fighters[d];
            if is_intangible(fd) {
                continue;
            }
            let hurt = hurtboxes(fd, params_of(content, fd));
            let mut best: Option<Hitbox> = None;
            for (_, hb, center) in active_hitboxes(fa, mv, params_of(content, fa).hitbox_scale) {
                if fa.hit_mask & hit_bit(d, hb.group) != 0 {
                    continue;
                }
                let touching = match hb.kind {
                    // Throws and pummels act on the fighter being held, wherever the hitbox is.
                    HIT_THROW | HIT_PUMMEL => fa.grab_with == d as i8,
                    HIT_GRAB => {
                        grab::grabbable(fd)
                            && hurt
                                .iter()
                                .any(|(hc, hr)| overlaps(center, hb.radius, *hc, *hr))
                    }
                    _ => {
                        fa.grab_with != d as i8
                            && hurt
                                .iter()
                                .any(|(hc, hr)| overlaps(center, hb.radius, *hc, *hr))
                    }
                };
                if touching && best.is_none_or(|b| hb.priority < b.priority) {
                    best = Some(hb);
                }
            }
            chosen[a][d] = best;
        }
    }

    let mut struck = [false; MAX_FIGHTERS];
    for a in 0..MAX_FIGHTERS {
        for d in 0..MAX_FIGHTERS {
            if let Some(hb) = chosen[a][d] {
                if hb.kind == HIT_GRAB {
                    continue; // grabs are settled after every damaging hit this frame
                }
                if !struck[d] {
                    struck[d] = true;
                    let attacker = state.fighters[a];
                    state.fighters[a].hit_mask |= hit_bit(d, hb.group);
                    // A hitbox behind the attacker (back air) sends the victim backward, away from it.
                    let facing = if hb.x < Fx::ZERO {
                        -attacker.facing
                    } else {
                        attacker.facing
                    };
                    // A charged smash attack hits harder; the damage also feeds the knockback.
                    let mut hb = hb;
                    let attack =
                        weapon_of(content, params_of(content, &attacker)).get(attacker.move_id);
                    if attacker.charge > 0 {
                        let rules = &content.rules;
                        let full = if attack.charge_bonus > 0 {
                            attack.charge_bonus
                        } else {
                            rules.charge_bonus_percent
                        };
                        let bonus = Fx::from_ratio(
                            i32::from(attacker.charge) * i32::from(full),
                            100 * i32::from(rules.charge_frames.max(1)),
                        );
                        hb.damage = hb.damage * (Fx::ONE + bonus);
                    }
                    // The answer to a caught hit deals what was caught.
                    if attack.counter_strike {
                        hb.damage = attacker.counter_damage;
                    }
                    match hb.kind {
                        HIT_PUMMEL => pummel(state, content, a, d, &hb),
                        HIT_THROW => {
                            // The throw lets go: the held fighter is launched like any hit victim.
                            state.fighters[a].grab_with = NONE;
                            fighter::free_held(&mut state.fighters[d], 0);
                            apply_hit(
                                state,
                                content,
                                a,
                                d,
                                &hb,
                                facing,
                                attacker.pos,
                                true,
                                Fx::ONE,
                            );
                        }
                        _ => {
                            let mult = stun_multiplier(&attacker);
                            apply_hit(state, content, a, d, &hb, facing, attacker.pos, true, mult);
                        }
                    }
                }
            }
        }
    }

    // Grabs last: a fighter that was hit this frame, or whose grabber was, is not caught. Ties go to the
    // lower player index.
    for a in 0..MAX_FIGHTERS {
        for d in 0..MAX_FIGHTERS {
            let Some(hb) = chosen[a][d] else {
                continue;
            };
            if hb.kind != HIT_GRAB
                || struck[a]
                || struck[d]
                || state.fighters[a].state != S::Attack
                || !grab::grabbable(&state.fighters[d])
            {
                continue;
            }
            state.fighters[a].hit_mask |= hit_bit(d, hb.group);
            struck[d] = true;
            grab::connect(state, content, a, d);
        }
    }
}

/// A pummel: damage to the held fighter, a little hitlag for both, and it stays held.
fn pummel(state: &mut GameState, content: &Content, a: usize, d: usize, hb: &Hitbox) {
    let hitlag = hitlag_frames(hb.damage);
    state.fighters[a].hitlag = hitlag;
    let held = &mut state.fighters[d];
    held.percent = (held.percent + hb.damage * content.rules.damage_mult).min(Fx::from_int(999));
    held.hitlag = hitlag;
}

/// If fighter `d` is in a counter stance whose window is open, cancels the hit and switches `d` to its answer.
fn catch_with_counter(
    state: &mut GameState,
    content: &Content,
    source: usize,
    d: usize,
    damage: Fx,
    hitlag: u8,
) -> bool {
    let def = &state.fighters[d];
    if def.state != S::Attack {
        return false;
    }
    let weapon = weapon_of(content, params_of(content, def));
    let Some(c) = weapon.get(def.move_id).counter else {
        return false;
    };
    if def.state_frame < u16::from(c.start) || def.state_frame > u16::from(c.end) {
        return false;
    }
    let answer_id = MoveId::from_index(c.then);
    let intangible = weapon.get(c.then).intangible;
    let caught =
        (damage * Fx::from_int(i32::from(c.percent)) / Fx::from_int(100)).max(c.min_damage);
    let attacker_x = state.fighters[source].pos.x;
    let def = &mut state.fighters[d];
    // The answer is swung at whoever struck.
    if attacker_x != def.pos.x {
        def.facing = if attacker_x > def.pos.x { 1 } else { -1 };
    }
    def.counter_damage = caught;
    fighter::begin_attack(def, answer_id);
    def.invuln = def.invuln.max(intangible);
    def.hitlag = hitlag;
    true
}

/// Applies one hit: damage, knockback, hitlag and the victim's state change. `source` is the fighter
/// responsible (the attacker, or a projectile's owner). Only a melee attacker is frozen by hitlag.
#[allow(clippy::too_many_arguments)]
fn apply_hit(
    state: &mut GameState,
    content: &Content,
    source: usize,
    d: usize,
    hb: &Hitbox,
    facing: i8,
    source_pos: Vec2,
    freeze_source: bool,
    stun_mult: Fx,
) {
    let hitlag = hitlag_frames(hb.damage);
    // A fighter in a counter stance catches the hit instead of taking it.
    if catch_with_counter(state, content, source, d, hb.damage, hitlag) {
        if freeze_source {
            state.fighters[source].hitlag = hitlag;
        }
        return;
    }
    // A fighter that is hit lets go of, or is let go by, whoever it was holding or held by.
    grab::drop_grab(state, content, d);
    if freeze_source {
        state.fighters[source].hitlag = hitlag;
    }

    let defender_params = *params_of(content, &state.fighters[d]);
    let def = &mut state.fighters[d];
    if def.state == S::Shield {
        block(def, &content.rules, hb, facing, hitlag, stun_mult);
        return;
    }
    if def.state == S::ShieldBreak {
        // Hit while stunned: the stun ends and the shield comes back.
        fighter::restore_shield(def, &content.rules);
    }

    def.percent = (def.percent + hb.damage * content.rules.damage_mult).min(Fx::from_int(999));
    let kb = knockback(
        def.percent,
        hb.damage,
        defender_params.weight,
        hb.base_knockback,
        hb.knockback_growth,
    );
    let angle = launch_angle(hb.angle, facing, def.grounded(), kb);
    let stun = hitstun_frames(kb, content.rules.hitstun_mult);

    def.hitlag = hitlag;
    if stun == 0 {
        return; // a flinch: hitlag only
    }
    if source_pos.x != def.pos.x {
        def.facing = if source_pos.x > def.pos.x { 1 } else { -1 };
    }
    if def.ledge != NONE {
        def.ledge = NONE;
        def.ledge_cooldown = defender_params.ledge_regrab_cooldown;
    }
    def.state = S::Hitstun;
    def.state_frame = 0;
    def.hitstun = stun;
    def.launch_pending = true;
    def.launch_kb = kb;
    def.launch_angle = angle.raw();
    def.kb_vel = Vec2::ZERO;
    def.fast_fall = false;
    def.tumble = false;
    def.hit_mask = 0;
}

/// A hit on a raised shield: no damage or launch. The shield loses health (unless it was a perfect shield),
/// the blocker is stunned for `0.8 * damage * type + 2` frames and slides back. If the shield runs out it breaks.
fn block(def: &mut Fighter, rules: &Ruleset, hb: &Hitbox, facing: i8, hitlag: u8, stun_mult: Fx) {
    let perfect = def.state_frame <= u16::from(rules.perfect_shield_window);
    let raw = (hb.damage * Fx::from_ratio(8, 10) * stun_mult + Fx::from_int(2)).floor_int();
    let mut stun = raw.clamp(0, i32::from(rules.shield_stun_cap));
    def.hitlag = hitlag;
    if perfect {
        stun = (stun - i32::from(rules.perfect_shield_stun_cut)).max(0);
    } else {
        let shield_damage =
            hb.damage * rules.damage_mult * Fx::from_int(i32::from(hb.shield_damage))
                / Fx::from_int(100);
        def.shield_hp -= shield_damage;
        if def.shield_hp <= Fx::ZERO {
            fighter::break_shield(def, rules);
            return;
        }
    }
    def.shield_stun = stun as u8;
    // Pushback: (stun + 1) * 0.09 reference units a frame, at most 1.3 (less for a perfect shield).
    let mut push = (Fx::from_int(stun + 1) * Fx::from_ratio(9, 100)).min(Fx::from_ratio(13, 10))
        / Fx::from_int(8);
    if perfect {
        push = push * Fx::from_ratio(2, 5);
    }
    def.vel.x += push.mul_int(i32::from(facing));
}

/// How much a hit of this attacker's current move counts toward the shield stun it causes.
fn stun_multiplier(f: &Fighter) -> Fx {
    let id = MoveId::from_index(f.move_id);
    if id.is_aerial() {
        Fx::from_ratio(33, 100)
    } else if matches!(id, MoveId::FSmash | MoveId::USmash | MoveId::DSmash) {
        Fx::from_ratio(725, 1000)
    } else {
        Fx::ONE
    }
}

/// Creates the projectile a move asked for this frame. A move that already connected in melee (a bayonet
/// hit, for instance) does not also fire.
pub fn spawn_projectiles(state: &mut GameState, content: &Content) {
    for i in 0..MAX_FIGHTERS {
        if !state.fighters[i].spawn_request {
            continue;
        }
        state.fighters[i].spawn_request = false;
        let f = state.fighters[i];
        state.fighters[i].spawn_custom = false;
        // A move that already connected in melee does not also fire, unless its script asked for the shot.
        if f.hit_mask != 0 && !f.spawn_custom {
            continue;
        }
        let Some(spec) = weapon_of(content, params_of(content, &f))
            .get(f.move_id)
            .projectile
        else {
            continue;
        };
        let dir = i32::from(f.facing);
        let scale = params_of(content, &f).hitbox_scale;
        if let Some(slot) = state.projectiles.iter_mut().find(|p| !p.active) {
            *slot = Projectile {
                active: true,
                owner: i as u8,
                origin: i as u8,
                power: 100,
                move_id: f.move_id,
                pos: if f.spawn_custom {
                    f.pos + f.spawn_pos
                } else {
                    Vec2::new(
                        f.pos.x + (spec.x * scale).mul_int(dir),
                        f.pos.y + spec.y * scale,
                    )
                },
                vel: if f.spawn_custom {
                    f.spawn_vel
                } else {
                    Vec2::new(spec.speed.mul_int(dir), Fx::ZERO)
                },
                age: 0,
                life: spec.life,
                vars: [0; crate::PROJECTILE_VARS],
            };
        }
    }
}

/// Moves projectiles, despawns them (lifetime, walls, blast zone) and applies their hits.
pub fn update_projectiles(state: &mut GameState, content: &Content) {
    let stage = &content.stage;
    for n in 0..state.projectiles.len() {
        let pr = state.projectiles[n];
        if !pr.active {
            continue;
        }
        let owner = usize::from(pr.owner).min(MAX_FIGHTERS - 1);
        let origin = usize::from(pr.origin).min(MAX_FIGHTERS - 1);
        let origin_params = params_of(content, &state.fighters[origin]);
        let fired_by = weapon_of(content, origin_params).get(pr.move_id);
        let Some(spec) = fired_by.projectile else {
            state.projectiles[n].active = false;
            continue;
        };
        // The projectile's own script steers it before it moves.
        if let Some(program) = &fired_by.projectile_script {
            let around = surroundings(state, owner, pr.pos);
            if scripting::run_projectile(program, &mut state.projectiles[n], around) {
                state.projectiles[n].active = false;
                continue;
            }
        }
        let pr = state.projectiles[n];

        let age = pr.age.saturating_add(1);
        let pos = pr.pos + pr.vel;
        let in_wall = stage.platforms.iter().any(|b| {
            !b.pass_through && pos.x > b.left && pos.x < b.right && pos.y > b.bottom && pos.y < b.y
        });
        let outside = pos.x < stage.blast_left
            || pos.x > stage.blast_right
            || pos.y < stage.blast_bottom
            || pos.y > stage.blast_top;
        if age >= pr.life || in_wall || outside {
            state.projectiles[n].active = false;
            continue;
        }
        // A reflector in the way turns it around: it now belongs to the reflector and hits harder.
        let shot_radius = spec.hitbox.radius * origin_params.hitbox_scale;
        if let Some((r, rf)) = reflector_touching(state, content, owner, pos, shot_radius) {
            let p = &mut state.projectiles[n];
            p.owner = r as u8;
            p.power = rf.damage_percent;
            p.vel = Vec2::new(
                -(pr.vel.x * Fx::from_int(i32::from(rf.speed_percent)) / Fx::from_int(100)),
                pr.vel.y,
            );
            p.pos = pos;
            p.age = 0;
            continue;
        }
        state.projectiles[n].pos = pos;
        state.projectiles[n].age = age;

        // Damage falls off with distance travelled.
        let progress = Fx::from_ratio(i32::from(age), i32::from(pr.life.max(1)));
        let mut hb = spec.hitbox;
        hb.radius = hb.radius * origin_params.hitbox_scale;
        hb.damage = spec.hitbox.damage + (spec.end_damage - spec.hitbox.damage) * progress;
        hb.damage = hb.damage * Fx::from_int(i32::from(pr.power)) / Fx::from_int(100);

        for d in 0..MAX_FIGHTERS {
            if d == owner || is_intangible(&state.fighters[d]) {
                continue;
            }
            let fd = &state.fighters[d];
            let hurt = hurtboxes(fd, params_of(content, fd));
            if hurt
                .iter()
                .any(|(hc, hr)| overlaps(pos, hb.radius, *hc, *hr))
            {
                let facing = if pr.vel.x < Fx::ZERO { -1 } else { 1 };
                // Projectiles stun a shield much less.
                let mult = Fx::from_ratio(29, 100);
                apply_hit(state, content, owner, d, &hb, facing, pos, false, mult);
                state.projectiles[n].active = false;
                break;
            }
        }
    }
}

/// What a projectile script can see: its owner and the nearest other fighter that is still in the match.
fn surroundings(state: &GameState, owner: usize, at: Vec2) -> scripting::Surroundings {
    let mut target: Option<(Fx, Vec2)> = None;
    for (i, f) in state.fighters.iter().enumerate() {
        if i == owner || f.stocks == 0 || !f.active {
            continue;
        }
        let d = (f.pos - at).length_sq();
        if target.is_none_or(|(best, _)| d < best) {
            target = Some((d, f.pos));
        }
    }
    scripting::Surroundings {
        owner: state.fighters[owner].pos,
        target: target.map(|(_, p)| p),
    }
}

/// A fighter other than `owner` whose reflector is up and touches a projectile at `pos`.
fn reflector_touching(
    state: &GameState,
    content: &Content,
    owner: usize,
    pos: Vec2,
    radius: Fx,
) -> Option<(usize, Reflector)> {
    for (i, f) in state.fighters.iter().enumerate() {
        if i == owner || f.state != S::Attack {
            continue;
        }
        let Some(rf) = weapon_of(content, params_of(content, f))
            .get(f.move_id)
            .reflector
        else {
            continue;
        };
        let frame = u8::try_from(f.state_frame).unwrap_or(u8::MAX);
        if frame < rf.start || frame > rf.end {
            continue;
        }
        let scale = params_of(content, f).hitbox_scale;
        let centre = Vec2::new(
            f.pos.x + (rf.x * scale).mul_int(i32::from(f.facing)),
            f.pos.y + rf.y * scale,
        );
        if overlaps(pos, radius, centre, rf.radius * scale) {
            return Some((i, rf));
        }
    }
    None
}

/// Sends fighters that have left the blast zone back to their spawn point, minus a stock. A fighter that loses its
/// last stock is out of the match (no longer `active`); with unlimited stocks nobody loses any.
pub fn check_ko(state: &mut GameState, content: &Content) {
    let s = &content.stage;
    for i in 0..MAX_FIGHTERS {
        if !state.fighters[i].active {
            continue;
        }
        let p = state.fighters[i].pos;
        if p.x < s.blast_left || p.x > s.blast_right || p.y < s.blast_bottom || p.y > s.blast_top {
            grab::drop_grab(state, content, i);
            let counted = state.rules.stocks > 0;
            let f = &mut state.fighters[i];
            respawn(f, content, i, counted);
            if counted && f.stocks == 0 {
                f.active = false;
            }
        }
    }
}

pub fn respawn(f: &mut Fighter, content: &Content, index: usize, lose_stock: bool) {
    let params = params_of(content, f);
    let pos = content.stage.spawns[index];
    let platform = collision::standing_on(&content.stage, pos);
    let (char_id, facing) = (f.char_id, f.facing);
    let stocks = if lose_stock {
        f.stocks.saturating_sub(1)
    } else {
        f.stocks
    };
    *f = Fighter::spawn(
        pos,
        char_id,
        facing,
        platform,
        params.air_jumps,
        content.rules.shield_max,
    );
    f.stocks = stocks;
    f.active = true;
    f.invuln = content.rules.respawn_invuln;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(n: i32) -> Fx {
        Fx::from_int(n)
    }

    fn close(a: Fx, expected_thousandths: i32) -> bool {
        (a - Fx::from_ratio(expected_thousandths, 1000)).abs() < Fx::from_ratio(1, 100)
    }

    #[test]
    fn knockback_matches_the_reference_formula() {
        // d = 10, p = 10 after the hit, w = 100, bkb 40, kbg 100:
        // ((1 + 5) * 1.0 * 1.4 + 18) * 1.0 + 40 = 66.4
        assert!(close(knockback(fx(10), fx(10), fx(100), 40, 100), 66_400));
        // A heavier target is launched less.
        assert!(
            knockback(fx(50), fx(10), fx(120), 40, 100)
                < knockback(fx(50), fx(10), fx(80), 40, 100)
        );
        // More percent, more knockback; more growth, more knockback.
        assert!(
            knockback(fx(100), fx(10), fx(90), 20, 100)
                > knockback(fx(20), fx(10), fx(90), 20, 100)
        );
        assert!(
            knockback(fx(50), fx(10), fx(90), 20, 120) > knockback(fx(50), fx(10), fx(90), 20, 80)
        );
    }

    #[test]
    fn hitlag_follows_damage() {
        assert_eq!(hitlag_frames(fx(3)), 5);
        assert_eq!(hitlag_frames(fx(9)), 7);
        assert_eq!(hitlag_frames(fx(30)), 14);
        assert_eq!(hitlag_frames(fx(1000)), 30);
    }

    #[test]
    fn hitstun_scales_with_knockback_and_the_ruleset_multiplier() {
        let kb = Fx::from_ratio(664, 10);
        assert_eq!(hitstun_frames(kb, Fx::ONE), 26);
        assert_eq!(hitstun_frames(kb, Fx::from_ratio(105, 100)), 27);
        assert!(hitstun_frames(kb.mul_int(2), Fx::ONE) > hitstun_frames(kb, Fx::ONE));
    }

    #[test]
    fn launch_angles_mirror_with_facing_and_resolve_the_sakurai_angle() {
        let a = |deg, facing, grounded, kb| launch_angle(deg, facing, grounded, fx(kb)).raw();
        assert_eq!(a(45, 1, false, 100), Angle::from_degrees(45).raw());
        assert_eq!(a(45, -1, false, 100), Angle::from_degrees(135).raw());
        assert_eq!(
            a(361, 1, true, 20),
            Angle::from_degrees(0).raw(),
            "weak grounded hit is horizontal"
        );
        assert_eq!(a(361, 1, true, 60), Angle::from_degrees(44).raw());
        assert_eq!(a(361, 1, false, 20), Angle::from_degrees(44).raw());
        assert_eq!(a(361, -1, false, 60), Angle::from_degrees(136).raw());
    }

    #[test]
    fn circles_overlap_when_the_gap_is_within_the_radii() {
        let o = |x: i32, r1: i32, r2: i32| {
            overlaps(
                Vec2::ZERO,
                Fx::from_ratio(r1, 10),
                Vec2::new(Fx::from_ratio(x, 10), Fx::ZERO),
                Fx::from_ratio(r2, 10),
            )
        };
        assert!(o(10, 6, 5));
        assert!(o(11, 6, 5), "touching counts");
        assert!(!o(12, 6, 5));
    }
}
