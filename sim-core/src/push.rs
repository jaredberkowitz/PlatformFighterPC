//! Bodies on the ground push each other apart, so a fighter cannot walk, run or dash through another (as in the reference game).
//! Runs once a frame after every fighter has moved and before hits are resolved.
//!
//! Each fighter's body is `push_ratio` of its ECB's half-width either side of it. Two grounded fighters on the same platform whose
//! bodies overlap are separated in two parts:
//!
//! * a **hard core** (`push_hard` of the bodies' combined width) they cannot move into: whoever moved toward the other this frame
//!   is put back, in proportion to how far each came, so a runner stops against a standing fighter instead of shoving it along
//!   (fighters already inside it, one landing on the other, are not snapped out but eased apart by the soft edge);
//! * a **soft edge** outside it, eased apart at up to `push_speed` a frame each, so fighters that end up overlapping (a landing on
//!   top of someone) slide apart rather than snapping.
//!
//! Nothing here touches attacks: a fighter in an attack is never moved (the other one gives way), two attacking fighters pass
//! as they would, and hits, hurtboxes and grabs are unchanged. Rolls, spot dodges, get-ups, anyone intangible, hit or held, and
//! anyone in the air go straight through. No one is pushed off the edge of a platform: the push stops there and the other gives
//! way instead.

use crate::collision;
use crate::combat::params_of;
use crate::content::Content;
use crate::fixed::Fx;
use crate::state::{Fighter, FighterState as S, GameState};
use crate::MAX_FIGHTERS;

/// How a fighter takes part in pushing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    /// Passes through and is passed through.
    None,
    /// Blocks others but is not moved (an attack).
    Fixed,
    /// Blocks and is moved.
    Free,
}

fn body(f: &Fighter) -> Body {
    if !f.active || !f.grounded() || f.invuln > 0 || f.ledge_invuln > 0 || f.grab_with >= 0 {
        return Body::None;
    }
    match f.state {
        S::Roll
        | S::SpotDodge
        | S::GetUp
        | S::Knockdown
        | S::Hitstun
        | S::Grabbing
        | S::Grabbed
        | S::ShieldBreak
        | S::Respawn
        | S::LedgeGetUp
        | S::LedgeAttack => Body::None,
        S::Attack => Body::Fixed,
        _ => Body::Free,
    }
}

/// Moves fighter `i` by `dx` along the ground, as far as its platform allows; returns how far it actually went.
fn shift(state: &mut GameState, content: &Content, i: usize, dx: Fx) -> Fx {
    let f = &mut state.fighters[i];
    let mut to = f.pos.x + dx;
    if let Some(p) = collision::platform(&content.stage, f.platform) {
        to = to.clamp(p.left, p.right);
    }
    let moved = to - f.pos.x;
    f.pos.x = to;
    moved
}

/// Separates overlapping bodies. `before` is where each fighter stood at the start of the frame.
pub fn separate(state: &mut GameState, content: &Content, before: &[Fx; MAX_FIGHTERS]) {
    let rules = &content.rules;
    for i in 0..MAX_FIGHTERS {
        for j in i + 1..MAX_FIGHTERS {
            let (a, b) = (&state.fighters[i], &state.fighters[j]);
            let (bi, bj) = (body(a), body(b));
            if bi == Body::None || bj == Body::None || a.platform != b.platform {
                continue;
            }
            if bi == Body::Fixed && bj == Body::Fixed {
                continue;
            }
            let width = (params_of(content, a).ecb_half_width
                + params_of(content, b).ecb_half_width)
                * rules.push_ratio;
            let dx = a.pos.x - b.pos.x;
            let gap = dx.abs();
            if gap >= width {
                continue;
            }
            // Which way `i` is pushed (toward +x if it is to the right of `j`). Standing on the same spot, the way they came from,
            // and failing that by player order.
            let was = before[i] - before[j];
            let dir: i32 = if dx != Fx::ZERO {
                dx.signum_int()
            } else if was != Fx::ZERO {
                was.signum_int()
            } else {
                -1
            };
            // How far each moved toward the other this frame.
            let came_i = (Fx::ZERO.max((before[i] - a.pos.x).mul_int(dir))).min(width);
            let came_j = (Fx::ZERO.max((b.pos.x - before[j]).mul_int(dir))).min(width);
            let (free_i, free_j) = (bi == Body::Free, bj == Body::Free);

            // The hard core: whoever came into it this frame goes back out (shared by how far each came). Only the coming in is
            // undone: fighters already deep inside each other (one landed on the other) are left to the soft edge, so nobody jumps.
            let core = width * rules.push_hard;
            let inside = (core - gap).min(came_i + came_j);
            if inside > Fx::ZERO {
                let want_i = if !free_i {
                    Fx::ZERO
                } else if !free_j {
                    inside
                } else {
                    inside * came_i / (came_i + came_j)
                };
                // `j` covers the rest, including whatever `i` could not (at the edge of a platform), and back again.
                let moved_i = if free_i {
                    shift(state, content, i, want_i.mul_int(dir)).abs()
                } else {
                    Fx::ZERO
                };
                let rest = inside - moved_i;
                let moved_j = if free_j {
                    shift(state, content, j, -rest.mul_int(dir)).abs()
                } else {
                    Fx::ZERO
                };
                let short = rest - moved_j;
                if short > Fx::ZERO && free_i {
                    shift(state, content, i, short.mul_int(dir));
                }
            }

            // The soft edge: ease apart, each by up to `push_speed`.
            let gap = (state.fighters[i].pos.x - state.fighters[j].pos.x).abs();
            let overlap = width - gap;
            if overlap > Fx::ZERO {
                let half = (overlap / Fx::from_int(2)).min(rules.push_speed);
                let (step_i, step_j) = match (free_i, free_j) {
                    (true, true) => (half, half),
                    (true, false) => (overlap.min(rules.push_speed), Fx::ZERO),
                    _ => (Fx::ZERO, overlap.min(rules.push_speed)),
                };
                if step_i > Fx::ZERO {
                    shift(state, content, i, step_i.mul_int(dir));
                }
                if step_j > Fx::ZERO {
                    shift(state, content, j, -step_j.mul_int(dir));
                }
            }
        }
    }
}
