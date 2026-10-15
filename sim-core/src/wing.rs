//! A body's comeback form (E57, sim 125: the reference's one wing; `content::Wing`): when it comes, how it is spent and when it goes.
//! The state is on the fighter (`Fighter::wing_points`, `wing_spent`, and `hit_by` for the KO it is credited with) and hashed; the
//! numbers are content. What the wing does while it is out is read elsewhere: the body's numbers in `step` (`FighterParams::moving`),
//! the damage in `combat::apply_hit`, the powered moves in `fighter::begin_attack`.

use crate::combat::params_of;
use crate::content::{Content, Wing};
use crate::fixed::Fx;
use crate::state::{Fighter, GameState};

/// Whether the fighter has its wing out.
pub fn out(f: &Fighter) -> bool {
    f.wing_points > Fx::ZERO
}

/// The opponent the wing measures fighter `i` against: of the other fighters in the match still playing, the one with the most stocks
/// (in a one-on-one match, the opponent; with more, the leader, the lowest player index on a tie [E: the reference's rule with
/// several opponents is not published]).
fn opponent(state: &GameState, i: usize) -> Option<usize> {
    let mut best: Option<usize> = None;
    for j in 0..crate::MAX_FIGHTERS {
        if j == i || state.roster >> j & 1 == 0 || !state.fighters[j].active {
            continue;
        }
        if best.is_none_or(|b| state.fighters[j].stocks > state.fighters[b].stocks) {
            best = Some(j);
        }
    }
    best
}

/// Fighter `i`'s stock difference with its opponent (its stocks less the opponent's), -2 to 2; 0 when stocks are not counted or
/// there is no opponent.
fn stock_gap(state: &GameState, i: usize) -> i32 {
    if state.rules.stocks == 0 {
        return 0;
    }
    match opponent(state, i) {
        Some(o) => {
            (i32::from(state.fighters[i].stocks) - i32::from(state.fighters[o].stocks)).clamp(-2, 2)
        }
        None => 0,
    }
}

/// The points the wing comes with at a stock difference.
fn grant(w: &Wing, gap: i32) -> Fx {
    w.points[(gap + 2) as usize]
}

/// Gives the fighter its wing with `points` points (one more midair jump for each of the wing's extra jumps, at once).
fn come(f: &mut Fighter, w: &Wing, points: Fx) {
    f.wing_points = points.max(Fx::from_raw(1));
    f.air_jumps_left = f.air_jumps_left.saturating_add(w.jumps);
}

/// Takes the wing away for good (until a KO): its midair jumps go with it.
fn lose(f: &mut Fighter, w: &Wing) {
    f.wing_points = Fx::ZERO;
    f.wing_spent = true;
    f.air_jumps_left = f.air_jumps_left.saturating_sub(w.jumps);
}

/// Fighter `victim`'s percent rose from `before` by a hit of fighter `source`'s (it may be its own): the KO credit, and the wing may
/// come (the start table's row for its stock difference, at the percent difference before the hit).
pub fn on_hurt(state: &mut GameState, content: &Content, source: usize, victim: usize, before: Fx) {
    if source != victim {
        state.fighters[victim].hit_by = source as i8;
    }
    let Some(w) = params_of(content, &state.fighters[victim]).wing else {
        return;
    };
    let f = &state.fighters[victim];
    if out(f) || f.wing_spent {
        return;
    }
    let gap = stock_gap(state, victim);
    let other = opponent(state, victim).map_or(Fx::ZERO, |o| state.fighters[o].percent);
    let threshold = w.rows[(gap + 2) as usize].threshold(before - other);
    if state.fighters[victim].percent >= threshold {
        come(&mut state.fighters[victim], &w, grant(&w, gap));
    }
}

/// Fighter `source`'s hit dealt `base` damage (the hitbox's own, before staleness, the short hop, the one-on-one multiplier and the
/// wing's power) to fighter `victim`'s body: the wing spends points for it.
pub fn on_dealt(state: &mut GameState, content: &Content, source: usize, victim: usize, base: Fx) {
    if source == victim || !out(&state.fighters[source]) {
        return;
    }
    let Some(w) = params_of(content, &state.fighters[source]).wing else {
        return;
    };
    let rate = if state.roster.count_ones() > 2 {
        w.per_dealt[1]
    } else {
        w.per_dealt[0]
    };
    let f = &mut state.fighters[source];
    f.wing_points -= rate * base;
    if f.wing_points <= Fx::ZERO {
        lose(f, &w);
    }
}

/// Fighter `victim` is about to be knocked out (its stock not yet taken): who it is credited to, and that fighter's stock difference
/// before the KO, for [`after_ko`].
pub fn before_ko(state: &GameState, victim: usize) -> Option<(usize, i32)> {
    let k = usize::try_from(state.fighters[victim].hit_by).ok()?;
    if k == victim || k >= state.fighters.len() {
        return None;
    }
    Some((k, stock_gap(state, k)))
}

/// After a KO credited to fighter `killer` (whose stock difference was `old_gap`): a wing out spends `ko_points`, then its points move
/// by the difference between what the new stock difference and the old one grant.
pub fn after_ko(state: &mut GameState, content: &Content, killer: usize, old_gap: i32) {
    if !out(&state.fighters[killer]) {
        return;
    }
    let Some(w) = params_of(content, &state.fighters[killer]).wing else {
        return;
    };
    let new_gap = stock_gap(state, killer);
    let f = &mut state.fighters[killer];
    f.wing_points = f.wing_points - w.ko_points + grant(&w, new_gap) - grant(&w, old_gap);
    if f.wing_points <= Fx::ZERO {
        lose(f, &w);
    }
}
