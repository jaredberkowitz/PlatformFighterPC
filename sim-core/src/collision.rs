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

/// A ledge within this fighter's grab box, if any. The fighter must be on the outside of the edge
/// (with a little tolerance so a fighter drifting past the corner can still grab).
pub fn find_ledge(stage: &Stage, pos: Vec2, params: &FighterParams) -> Option<usize> {
    let tolerance = Fx::HALF;
    stage.ledges.iter().position(|l| {
        let outside = if l.side < 0 { l.x - pos.x } else { pos.x - l.x };
        let dy = pos.y - l.y;
        outside >= -tolerance
            && outside <= params.ledge_reach_x
            && dy <= params.ledge_reach_up
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
        let near = Vec2::new(Fx::from_int(-21), Fx::from_int(-1));
        assert_eq!(find_ledge(&content.stage, near, &p), Some(0));
        let far = Vec2::new(Fx::from_int(-30), Fx::from_int(-1));
        assert_eq!(find_ledge(&content.stage, far, &p), None);
        let too_low = Vec2::new(Fx::from_int(-21), Fx::from_int(-5));
        assert_eq!(find_ledge(&content.stage, too_low, &p), None);
        let right = Vec2::new(Fx::from_int(21), Fx::from_int(-1));
        assert_eq!(find_ledge(&content.stage, right, &p), Some(1));
    }
}
