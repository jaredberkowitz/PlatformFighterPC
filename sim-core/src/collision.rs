//! Surface and ledge queries against stage data.
//!
//! A fighter's feet point is its collision point. A platform's top may slope (`Platform::rise`, sim 124): every query reads the
//! top's height where it is asked (`Platform::top_at`), which on a flat platform is its `y`, so flat stages behave exactly as before.
//! Where a sloped surface meets another at the same height (a ramp onto a roof), the two join: walking across the joint carries on
//! along the next surface ([`continuation`]), and the block's face there is no wall ([`face_is_joint`]).

use crate::content::{FighterParams, Platform, Stage};
use crate::fixed::Fx;
use crate::state::NONE;
use crate::vec2::Vec2;

fn spans(p: &Platform, x: Fx) -> bool {
    x >= p.left && x <= p.right
}

pub fn platform(stage: &Stage, index: i8) -> Option<&Platform> {
    usize::try_from(index)
        .ok()
        .and_then(|i| stage.platforms.get(i))
}

/// The platform whose top surface the point is exactly standing on, or [`NONE`].
pub fn standing_on(stage: &Stage, pos: Vec2) -> i8 {
    stage
        .platforms
        .iter()
        .position(|p| spans(p, pos.x) && p.top_at(pos.x) == pos.y)
        .map_or(NONE, |i| i as i8)
}

/// The highest platform whose top was crossed going from `prev` to `next` (from on or above it to on or below it). On a flat
/// platform that is a downward crossing of its height; a slope can also be met moving sideways into its rising top.
pub fn find_landing(stage: &Stage, prev: Vec2, next: Vec2, ignore_one_way: bool) -> Option<usize> {
    let mut best: Option<(usize, Fx)> = None;
    for (i, p) in stage.platforms.iter().enumerate() {
        if p.pass_through && ignore_one_way {
            continue;
        }
        let top = p.top_at(next.x);
        if prev.y >= p.top_at(prev.x) && next.y <= top && spans(p, next.x) {
            match best {
                Some((_, b)) if b >= top => {}
                _ => best = Some((i, top)),
            }
        }
    }
    best.map(|(i, _)| i)
}

/// Walking off an end of platform `from` to `x`: the surface that carries on from that end at the same height, if a slope is
/// involved (a sloped platform meeting another, or a flat one meeting a slope), and spans `x`. Flat platforms meeting at one height
/// do not join here (versus stages behave as they always have; the run joins its flat ground into one platform instead).
pub fn continuation(stage: &Stage, from: i8, x: Fx) -> Option<usize> {
    let p = platform(stage, from)?;
    let right = if x > p.right {
        true
    } else if x < p.left {
        false
    } else {
        return None;
    };
    stage.platforms.iter().enumerate().find_map(|(i, q)| {
        let meets = if right {
            q.left == p.right && q.y == p.top_right()
        } else {
            q.right == p.left && q.top_right() == p.y
        };
        (i as i8 != from && meets && (p.sloped() || q.sloped()) && spans(q, x)).then_some(i)
    })
}

/// Whether a solid block's face on `side` (-1 left, 1 right) is a joint rather than a wall: another solid block meets it there at
/// the face's top, one of the two sloped, so a fighter coming up (or down) the slope walks onto the block instead of into its face.
/// (Nothing can be beside such a face below the slope: the slope's block is solid down to its bottom.)
pub fn face_is_joint(stage: &Stage, b: &Platform, side: i8) -> bool {
    let (x, top) = if side < 0 {
        (b.left, b.y)
    } else {
        (b.right, b.top_right())
    };
    stage.platforms.iter().any(|q| {
        !q.pass_through
            && (q.sloped() || b.sloped())
            && if side < 0 {
                q.right == x && q.top_right() == top
            } else {
                q.left == x && q.y == top
            }
    })
}

/// The nearest surface at or below `pos`, with the distance down to it.
pub fn surface_below(stage: &Stage, pos: Vec2, ignore_one_way: bool) -> Option<(usize, Fx)> {
    let mut best: Option<(usize, Fx)> = None;
    for (i, p) in stage.platforms.iter().enumerate() {
        if (p.pass_through && ignore_one_way) || !spans(p, pos.x) || p.top_at(pos.x) > pos.y {
            continue;
        }
        let dist = pos.y - p.top_at(pos.x);
        match best {
            Some((_, d)) if d <= dist => {}
            _ => best = Some((i, dist)),
        }
    }
    best
}

/// Moves the fighter horizontally by `dx`, stopping at the face of any solid block it would cross.
/// The ECB is a diamond: widest at `ecb_side_height` above the feet and narrowing to a point at the
/// feet and at the head, so its width is measured at the height where the block overlaps it.
/// Returns true if a wall was hit.
pub fn move_x(stage: &Stage, params: &FighterParams, pos: &mut Vec2, dx: Fx) -> bool {
    let side_y = pos.y + params.ecb_side_height;
    let head_y = pos.y + params.ecb_height;
    let mut x = pos.x + dx;
    let mut hit = false;
    for b in stage.platforms.iter().filter(|b| !b.pass_through) {
        // (The face this move could cross: the left one going right, the right one going left. Its top is the top at that end.)
        let face_top = if dx > Fx::ZERO { b.y } else { b.top_right() };
        if pos.y >= face_top || head_y <= b.bottom {
            continue;
        }
        // The point of the block's face nearest the ECB's widest point.
        let near_y = side_y.clamp(b.bottom, face_top);
        let (dist, span) = if near_y > side_y {
            (near_y - side_y, params.ecb_height - params.ecb_side_height)
        } else {
            (side_y - near_y, params.ecb_side_height)
        };
        if dist >= span {
            continue;
        }
        let hw = params.ecb_half_width * ((span - dist) / span);
        if dx > Fx::ZERO && pos.x + hw <= b.left && x + hw > b.left {
            if face_is_joint(stage, b, -1) {
                continue;
            }
            x = b.left - hw;
            hit = true;
        } else if dx < Fx::ZERO && pos.x - hw >= b.right && x - hw < b.right {
            if face_is_joint(stage, b, 1) {
                continue;
            }
            x = b.right + hw;
            hit = true;
        }
    }
    pos.x = x;
    hit
}

/// The side (-1 left, 1 right) of a solid wall within `reach` of the fighter's ECB, or 0 if none (a wall jump's contact).
pub fn wall_beside(stage: &Stage, params: &FighterParams, pos: Vec2, reach: Fx) -> i8 {
    for side in [1i8, -1] {
        let mut probe = pos;
        if move_x(stage, params, &mut probe, reach.mul_int(i32::from(side))) {
            return side;
        }
    }
    0
}

/// Resolves any overlap between the ECB diamond and solid blocks by pushing the fighter sideways
/// to the nearest face. Needed because the diamond widens as a fighter slides down a wall, which
/// can put a fighter that started outside the wall inside it. Returns true if it moved.
pub fn push_out(stage: &Stage, params: &FighterParams, pos: &mut Vec2) -> bool {
    let side_y = pos.y + params.ecb_side_height;
    let head_y = pos.y + params.ecb_height;
    let mut moved = false;
    for b in stage.platforms.iter().filter(|b| !b.pass_through) {
        let top = b.top_at(pos.x);
        if pos.y >= top || head_y <= b.bottom {
            continue;
        }
        let near_y = side_y.clamp(b.bottom, top);
        let (dist, span) = if near_y > side_y {
            (near_y - side_y, params.ecb_height - params.ecb_side_height)
        } else {
            (side_y - near_y, params.ecb_side_height)
        };
        if dist >= span {
            continue;
        }
        let hw = params.ecb_half_width * ((span - dist) / span);
        if pos.x + hw > b.left && pos.x - hw < b.right {
            let middle = (b.left + b.right) * Fx::HALF;
            pos.x = if pos.x < middle {
                b.left - hw
            } else {
                b.right + hw
            };
            moved = true;
        }
    }
    moved
}

/// Moves the fighter upward by `dy`, stopping when the top of its ECB meets the underside of a
/// solid block. Returns true if a ceiling was hit.
pub fn move_up(stage: &Stage, params: &FighterParams, pos: &mut Vec2, dy: Fx) -> bool {
    let top = pos.y + params.ecb_height;
    let mut y = pos.y + dy;
    let mut hit = false;
    for b in stage.platforms.iter().filter(|b| !b.pass_through) {
        if spans(b, pos.x) && top <= b.bottom && y + params.ecb_height > b.bottom {
            y = b.bottom - params.ecb_height;
            hit = true;
        }
    }
    pos.y = y;
    hit
}

/// A ledge within this fighter's grab box, if any. The fighter's body must be fully clear of the stage
/// wall (at least `ecb_half_width` outside the edge) and at least `ledge_min_drop` below the ledge, so a
/// fighter walking or hopping off the stage, whose body still overlaps the corner, does not snap onto it,
/// and a fighter over the stage never grabs it.
pub fn find_ledge(stage: &Stage, pos: Vec2, facing: i8, params: &FighterParams) -> Option<usize> {
    stage.ledges.iter().position(|l| {
        let outside = if l.side < 0 { l.x - pos.x } else { pos.x - l.x };
        let dy = pos.y - l.y;
        // A ledge behind the fighter (its back to the stage) has a shorter reach.
        let reach = if facing == l.side {
            params.ledge_reach_back_x
        } else {
            params.ledge_reach_x
        };
        outside >= params.ecb_half_width
            && outside <= reach
            && dy <= -params.ledge_min_drop
            && dy >= -params.ledge_reach_down
    })
}

/// Whether a solid platform (a block, not a pass-through platform) lies across the segment from `a` to `b` (its inside, not
/// merely touching an edge): a line of sight. Worked in fixed point by clipping the segment to each block (slab method).
pub fn segment_blocked(stage: &Stage, a: Vec2, b: Vec2) -> bool {
    let d = b - a;
    stage.platforms.iter().filter(|p| !p.pass_through).any(|p| {
        let (mut t0, mut t1) = (Fx::ZERO, Fx::ONE);
        // (A slope's box reaches its higher end.)
        for (start, delta, low, high) in [
            (a.x, d.x, p.left, p.right),
            (a.y, d.y, p.bottom, p.top_max()),
        ] {
            if delta == Fx::ZERO {
                if start <= low || start >= high {
                    return false;
                }
                continue;
            }
            let (mut ta, mut tb) = ((low - start) / delta, (high - start) / delta);
            if ta > tb {
                core::mem::swap(&mut ta, &mut tb);
            }
            t0 = t0.max(ta);
            t1 = t1.min(tb);
        }
        t0 < t1
    })
}

/// Whether the segment from `a` to `b` meets ground (sim 128: a move's ground probe, `moves::GroundProbe`): it goes into a block
/// ([`segment_blocked`]) or crosses a pass-through platform's top (its line, by its slope) within the platform's span.
pub fn segment_meets_ground(stage: &Stage, a: Vec2, b: Vec2) -> bool {
    if segment_blocked(stage, a, b) {
        return true;
    }
    stage.platforms.iter().filter(|p| p.pass_through).any(|p| {
        let width = p.right - p.left;
        let line = |x: Fx| {
            if p.rise == Fx::ZERO || width <= Fx::ZERO {
                p.y
            } else {
                p.y + p.rise * (x - p.left) / width
            }
        };
        let (fa, fb) = (a.y - line(a.x), b.y - line(b.x));
        if (fa > Fx::ZERO && fb > Fx::ZERO) || (fa < Fx::ZERO && fb < Fx::ZERO) {
            return false;
        }
        let x = if fa == fb {
            a.x
        } else {
            a.x + (b.x - a.x) * (fa / (fa - fb))
        };
        spans(p, x)
    })
}

/// Which side (-1 or +1) the given ledge index hangs on; 0 if the index is invalid.
pub fn ledge_side(stage: &Stage, index: i8) -> i8 {
    usize::try_from(index)
        .ok()
        .and_then(|i| stage.ledges.get(i))
        .map_or(0, |l| l.side)
}

/// Where a fighter's feet sit while hanging from the ledge.
pub fn ledge_hang_pos(ledge: &crate::content::Ledge, params: &FighterParams) -> Vec2 {
    Vec2::new(
        ledge.x + params.ledge_hang_dx.mul_int(i32::from(ledge.side)),
        ledge.y - params.ledge_hang_dy,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Content;

    fn v(x: i32, y: i32) -> Vec2 {
        Vec2::new(Fx::from_int(x), Fx::from_int(y))
    }

    #[test]
    fn landing_requires_crossing_from_above_within_span() {
        let stage = Content::placeholder().stage;
        assert_eq!(find_landing(&stage, v(0, 1), v(0, -1), false), Some(0));
        assert_eq!(
            find_landing(&stage, v(0, -1), v(0, -2), false),
            None,
            "already below"
        );
        assert_eq!(
            find_landing(&stage, v(30, 1), v(30, -1), false),
            None,
            "off the edge"
        );
    }

    #[test]
    fn highest_platform_wins_and_one_way_can_be_ignored() {
        let stage = Content::placeholder().stage;
        assert_eq!(find_landing(&stage, v(-8, 8), v(-8, -1), false), Some(1));
        assert_eq!(find_landing(&stage, v(-8, 8), v(-8, -1), true), Some(0));
    }

    #[test]
    fn surface_below_reports_distance() {
        let stage = Content::placeholder().stage;
        let (i, d) = surface_below(&stage, v(0, 3), false).unwrap();
        assert_eq!((i, d), (0, Fx::from_int(3)));
        assert_eq!(surface_below(&stage, v(30, 3), false), None);
        assert_eq!(surface_below(&stage, v(0, -3), false), None);
    }

    #[test]
    fn ledge_grab_box() {
        let content = Content::placeholder();
        let p = content.fighters[0];
        let at = |x: i32, y: i32| Vec2::new(Fx::from_int(x), Fx::from_int(y));
        assert_eq!(find_ledge(&content.stage, at(-12, -2), 0, &p), Some(0));
        assert_eq!(
            find_ledge(&content.stage, at(-21, -2), 0, &p),
            None,
            "too far out"
        );
        assert_eq!(
            find_ledge(&content.stage, at(-12, -5), 0, &p),
            None,
            "too far below"
        );
        assert_eq!(find_ledge(&content.stage, at(12, -2), 0, &p), Some(1));
    }

    #[test]
    fn a_fighter_over_the_stage_or_level_with_the_ledge_cannot_grab() {
        let content = Content::placeholder();
        let p = content.fighters[0];
        let at = |x: Fx, y: Fx| Vec2::new(x, y);
        let ledge_y = Fx::ZERO;
        // Inside the stage edge (the "grabbing from the stage side" case).
        assert_eq!(
            find_ledge(
                &content.stage,
                at(
                    Fx::from_ratio(-109, 10),
                    -p.ledge_min_drop * Fx::from_int(2)
                ),
                0,
                &p
            ),
            None
        );
        // Walking or hopping off: level with the ledge, outside of it.
        assert_eq!(
            find_ledge(&content.stage, at(Fx::from_ratio(-111, 10), ledge_y), 0, &p),
            None
        );
        // Still overlapping the stage corner: not yet clear of the wall, however far it has fallen.
        let low = -p.ledge_min_drop * Fx::from_int(2);
        assert_eq!(
            find_ledge(&content.stage, at(Fx::from_ratio(-111, 10), low), 0, &p),
            None
        );
        // Clear of the wall and a little way down, it can be grabbed.
        let clear = Fx::from_int(-11) - p.ecb_half_width;
        assert_eq!(find_ledge(&content.stage, at(clear, low), 0, &p), Some(0));
        assert_eq!(
            find_ledge(&content.stage, at(clear, -p.ledge_min_drop), 0, &p),
            Some(0)
        );
        assert_eq!(
            find_ledge(
                &content.stage,
                at(clear, -p.ledge_min_drop / Fx::from_int(2)),
                0,
                &p
            ),
            None,
            "not yet dropped far enough"
        );
    }

    #[test]
    fn push_out_moves_an_overlapping_fighter_to_the_nearest_face() {
        let content = Content::placeholder();
        let p = content.fighters[0];
        // Just inside the right face of the main block, below its top.
        let mut pos = Vec2::new(Fx::from_ratio(105, 10), Fx::from_int(-2));
        assert!(push_out(&content.stage, &p, &mut pos));
        assert!(pos.x >= Fx::from_int(11) + p.ecb_half_width - Fx::from_raw(2));
        // Standing on top is not an overlap.
        let mut top = Vec2::new(Fx::from_int(10), Fx::ZERO);
        assert!(!push_out(&content.stage, &p, &mut top));
    }
}
