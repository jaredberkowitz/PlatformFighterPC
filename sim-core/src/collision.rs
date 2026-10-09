//! Surface and ledge queries against stage data.
//!
//! Phase 1 scope: platforms are top surfaces only (no walls or ceilings yet). A fighter's feet
//! point is its collision point; `ecb_*` params are reserved for hurtboxes in Phase 3.

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
        .position(|p| p.y == pos.y && spans(p, pos.x))
        .map_or(NONE, |i| i as i8)
}

/// The highest platform whose top was crossed going from `prev` to `next` (downward only).
pub fn find_landing(stage: &Stage, prev: Vec2, next: Vec2, ignore_one_way: bool) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, p) in stage.platforms.iter().enumerate() {
        if p.pass_through && ignore_one_way {
            continue;
        }
        if prev.y >= p.y && next.y <= p.y && spans(p, next.x) {
            match best {
                Some(b) if stage.platforms[b].y >= p.y => {}
                _ => best = Some(i),
            }
        }
    }
    best
}

/// The nearest surface at or below `pos`, with the distance down to it.
pub fn surface_below(stage: &Stage, pos: Vec2, ignore_one_way: bool) -> Option<(usize, Fx)> {
    let mut best: Option<(usize, Fx)> = None;
    for (i, p) in stage.platforms.iter().enumerate() {
        if (p.pass_through && ignore_one_way) || p.y > pos.y || !spans(p, pos.x) {
            continue;
        }
        let dist = pos.y - p.y;
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
        if pos.y >= b.y || head_y <= b.bottom {
            continue;
        }
        // The point of the block's face nearest the ECB's widest point.
        let near_y = side_y.clamp(b.bottom, b.y);
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
            x = b.left - hw;
            hit = true;
        } else if dx < Fx::ZERO && pos.x - hw >= b.right && x - hw < b.right {
            x = b.right + hw;
            hit = true;
        }
    }
    pos.x = x;
    hit
}

/// Resolves any overlap between the ECB diamond and solid blocks by pushing the fighter sideways
/// to the nearest face. Needed because the diamond widens as a fighter slides down a wall, which
/// can put a fighter that started outside the wall inside it. Returns true if it moved.
pub fn push_out(stage: &Stage, params: &FighterParams, pos: &mut Vec2) -> bool {
    let side_y = pos.y + params.ecb_side_height;
    let head_y = pos.y + params.ecb_height;
    let mut moved = false;
    for b in stage.platforms.iter().filter(|b| !b.pass_through) {
        if pos.y >= b.y || head_y <= b.bottom {
            continue;
        }
        let near_y = side_y.clamp(b.bottom, b.y);
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
