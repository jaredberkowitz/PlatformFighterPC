//! Balloons (E41, sim 113: the reference's balloon flight, owner decision 68; numbers in `moves::Balloons`). A fighter in a move
//! with a `balloons` block carries balloons above its head: they come out when the move starts (how many by the helium left), use
//! up helium every frame, and pop when hits reach them without reaching the body. The state is the fighter's `balloons`,
//! `balloon_hp` and `helium_used`, all hashed.

use crate::combat::{params_of, shapes_touch, weapon_of};
use crate::content::Content;
use crate::fighter;
use crate::fixed::Fx;
use crate::moves::Balloons;
use crate::state::{Fighter, FighterState as S, GameState};
use crate::vec2::Vec2;

/// Sixteenths of a frame of flight in one frame (the unit of `Fighter::helium_used` and `Balloons::refill`).
pub const HELIUM_PER_FRAME: u16 = 16;

/// The balloon block of the move a fighter is in, if it is in one.
pub fn flying(content: &Content, f: &Fighter) -> Option<Balloons> {
    if f.state != S::Attack {
        return None;
    }
    weapon_of(content, params_of(content, f))
        .get(f.move_id)
        .balloons
}

/// The balloons' hurt circle (centre and radius) of a fighter carrying any, else `None`.
pub fn circle(content: &Content, f: &Fighter) -> Option<(Vec2, Fx)> {
    if f.balloons == 0 {
        return None;
    }
    let b = flying(content, f)?;
    let scale = params_of(content, f).hitbox_scale;
    let at = f.pos + Vec2::new((b.x * scale).mul_int(i32::from(f.facing)), b.y * scale);
    Some((at, b.radius * scale))
}

/// Whether a hit of shape (`half_w`, `half_h`, `radius`) at `center` reaches the balloons of `f`.
pub fn touches(
    content: &Content,
    f: &Fighter,
    center: Vec2,
    half_w: Fx,
    half_h: Fx,
    radius: Fx,
) -> bool {
    circle(content, f).is_some_and(|(at, r)| {
        shapes_touch(at, Fx::ZERO, Fx::ZERO, r, center, half_w, half_h, radius)
    })
}

/// A hit of `damage` percent (before the one-on-one multiplier) on the balloons of fighter `d`: the next balloon takes it and pops at
/// 0, at most one a hit; the last one popping drops the fighter into the special fall with the block's `pop_vy` upward speed.
pub fn hit(state: &mut GameState, content: &Content, d: usize, damage: Fx) {
    let Some(b) = flying(content, &state.fighters[d]) else {
        return;
    };
    let f = &mut state.fighters[d];
    if f.balloons == 0 {
        return;
    }
    f.balloon_hp -= damage;
    if f.balloon_hp > Fx::ZERO {
        return;
    }
    f.balloons -= 1;
    f.balloon_hp = b.hp;
    if f.balloons == 0 {
        let_go(f);
        f.vel.y = b.pop_vy;
    }
}

/// The balloons go (the helium ran out, or the last one popped): the special fall, with the move's own fall rules.
fn let_go(f: &mut Fighter) {
    f.balloons = 0;
    f.balloon_hp = Fx::ZERO;
    if f.grounded() {
        fighter::stand_or_fall(f);
    } else {
        fighter::enter_helpless(f);
    }
}

/// Once a frame, after the fighters have moved: balloons come out for a balloon move that has just begun, its helium is used (out of
/// hitlag) and runs out, and a fighter out of a balloon move has none and refills its tank.
pub fn update(state: &mut GameState, content: &Content) {
    for i in 0..crate::state::bodies(state) {
        if !state.fighters[i].active {
            continue;
        }
        let in_flight = flying(content, &state.fighters[i]);
        let weapon = weapon_of(content, params_of(content, &state.fighters[i]));
        let f = &mut state.fighters[i];
        match in_flight {
            Some(b) => {
                let full = b.fuel.saturating_mul(HELIUM_PER_FRAME);
                if f.balloons == 0 {
                    // The balloons come out: all of them, or one when the helium is low (the reference's 13.0.1 rule).
                    let left = full.saturating_sub(f.helium_used);
                    f.balloons = if left == 0 {
                        0
                    } else if left < b.one_below.saturating_mul(HELIUM_PER_FRAME) {
                        1
                    } else {
                        b.count
                    };
                    f.balloon_hp = b.hp;
                    if f.balloons == 0 {
                        let_go(f);
                        continue;
                    }
                }
                if f.hitlag == 0 {
                    f.helium_used = f.helium_used.saturating_add(HELIUM_PER_FRAME).min(full);
                    if f.helium_used >= full {
                        let_go(f);
                    }
                }
            }
            None => {
                f.balloons = 0;
                f.balloon_hp = Fx::ZERO;
                // (The tank refills at the weapon's balloon rate: the first balloon move's.)
                if f.helium_used > 0 {
                    if let Some(b) = weapon.moves.iter().find_map(|m| m.balloons) {
                        f.helium_used = f.helium_used.saturating_sub(b.refill);
                    }
                }
            }
        }
    }
}
