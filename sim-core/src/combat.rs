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
use crate::content::{Content, FighterParams};
use crate::fixed::Fx;
use crate::moves::{Hitbox, Move, Weapon};
use crate::state::{Fighter, FighterState as S, GameState, NONE};
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
    f.invuln > 0
        || f.ledge_invuln > 0
        || (f.state == S::AirDodge && (4..=28).contains(&f.state_frame))
}

/// The move's hitboxes that are active this frame, with their world-space centres.
pub fn active_hitboxes<'a>(
    f: &'a Fighter,
    mv: &'a Move,
) -> impl Iterator<Item = (usize, &'a Hitbox, Vec2)> + 'a {
    let frame = u8::try_from(f.state_frame).unwrap_or(u8::MAX);
    let facing = i32::from(f.facing);
    mv.hitboxes
        .iter()
        .enumerate()
        .filter(move |(_, hb)| hb.start <= frame && frame <= hb.end)
        .map(move |(i, hb)| {
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

/// Finds and applies this frame's hits. Everything is collected before anything changes, so a trade
/// (both fighters hitting each other on the same frame) works, and ties go to the lower player index.
///
/// The loops need the attacker and defender indices themselves (hit masks, tie-breaking), not just items.
#[allow(clippy::needless_range_loop)]
pub fn resolve_hits(state: &mut GameState, content: &Content) {
    let mut chosen: [[Option<usize>; MAX_FIGHTERS]; MAX_FIGHTERS] =
        [[None; MAX_FIGHTERS]; MAX_FIGHTERS];
    for a in 0..MAX_FIGHTERS {
        let fa = &state.fighters[a];
        if fa.state != S::Attack || fa.hitlag > 0 {
            continue;
        }
        let mv = weapon_of(content, params_of(content, fa)).get(fa.move_id);
        for d in 0..MAX_FIGHTERS {
            if a == d || fa.hit_mask & (1 << d) != 0 {
                continue;
            }
            let fd = &state.fighters[d];
            if is_intangible(fd) {
                continue;
            }
            let hurt = hurtboxes(fd, params_of(content, fd));
            let mut best: Option<(u8, usize)> = None;
            for (index, hb, center) in active_hitboxes(fa, mv) {
                let touching = hurt
                    .iter()
                    .any(|(hc, hr)| overlaps(center, hb.radius, *hc, *hr));
                if touching && best.is_none_or(|(priority, _)| hb.priority < priority) {
                    best = Some((hb.priority, index));
                }
            }
            chosen[a][d] = best.map(|(_, index)| index);
        }
    }

    let mut struck = [false; MAX_FIGHTERS];
    for a in 0..MAX_FIGHTERS {
        for d in 0..MAX_FIGHTERS {
            if let Some(index) = chosen[a][d] {
                if !struck[d] {
                    struck[d] = true;
                    apply_hit(state, content, a, d, index);
                }
            }
        }
    }
}

fn apply_hit(state: &mut GameState, content: &Content, a: usize, d: usize, index: usize) {
    let attacker = state.fighters[a];
    let hb = weapon_of(content, params_of(content, &attacker))
        .get(attacker.move_id)
        .hitboxes[index];
    let hitlag = hitlag_frames(hb.damage);
    state.fighters[a].hit_mask |= 1 << d;
    state.fighters[a].hitlag = hitlag;

    let defender_params = *params_of(content, &state.fighters[d]);
    let def = &mut state.fighters[d];
    if def.state == S::Shield {
        // Blocked: no damage, no launch, a little pushback.
        def.hitlag = hitlag;
        def.vel.x += (hb.damage * Fx::from_ratio(1, 200)).mul_int(i32::from(attacker.facing));
        return;
    }

    def.percent = (def.percent + hb.damage).min(Fx::from_int(999));
    let kb = knockback(
        def.percent,
        hb.damage,
        defender_params.weight,
        hb.base_knockback,
        hb.knockback_growth,
    );
    let angle = launch_angle(hb.angle, attacker.facing, def.grounded(), kb);
    let stun = hitstun_frames(kb, content.rules.hitstun_mult);

    def.hitlag = hitlag;
    if stun == 0 {
        return; // a flinch: hitlag only
    }
    if attacker.pos.x != def.pos.x {
        def.facing = if attacker.pos.x > def.pos.x { 1 } else { -1 };
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

/// Sends fighters that have left the blast zone back to their spawn point, minus a stock.
pub fn check_ko(state: &mut GameState, content: &Content) {
    let s = &content.stage;
    for i in 0..MAX_FIGHTERS {
        let p = state.fighters[i].pos;
        if p.x < s.blast_left || p.x > s.blast_right || p.y < s.blast_bottom || p.y > s.blast_top {
            respawn(&mut state.fighters[i], content, i);
        }
    }
}

pub fn respawn(f: &mut Fighter, content: &Content, index: usize) {
    let params = params_of(content, f);
    let pos = content.stage.spawns[index];
    let platform = collision::standing_on(&content.stage, pos);
    let (char_id, facing, stocks) = (f.char_id, f.facing, f.stocks.saturating_sub(1));
    *f = Fighter::spawn(pos, char_id, facing, platform, params.air_jumps);
    f.stocks = stocks;
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
