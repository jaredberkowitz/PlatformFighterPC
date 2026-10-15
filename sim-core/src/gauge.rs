//! A body's gauge and its powered mode (`content::Gauge`, owner decision 31): what fills it, what drains it, the mode's timer and
//! spending it. The state is on the fighter (`Fighter::gauge`, `gauge_mode`, `gauge_drain`) and hashed; the rules are content.
//!
//! Every place a fighter's percent rises calls [`on_damage`] with what was actually added (after staleness, the short-hop factor and
//! the one-on-one multiplier: owner decision 32). Shielded hits, counters and pushes never reach it (they return before the percent
//! rises).

use crate::content::{Content, Gauge};
use crate::fixed::Fx;
use crate::moves::Weapon;
use crate::state::{Fighter, FighterState, GameState};

/// Adds `amount` units (a negative amount drains) to the fighter's gauge, between 0 and its `max`. Reaching `max` starts the powered
/// mode. While the mode runs (or for a body without a gauge) nothing changes.
pub fn add(f: &mut Fighter, g: &Gauge, amount: Fx) {
    if !g.exists() || f.gauge_mode > 0 {
        return;
    }
    f.gauge = (f.gauge + amount).clamp(Fx::ZERO, g.max);
    if f.gauge >= g.max {
        f.gauge = g.max;
        f.gauge_mode = g.mode_frames.max(1);
    }
}

/// One frame of the gauge (every frame, hitlag included [E: gap 12]): the powered mode's timer, which empties the gauge when it runs
/// out (and with `halts` stands still while the fighter's shield is broken: the reference's stun), or, out of the mode, the gauge's
/// fill by itself (`per_frame`, E53: not on a revival platform).
pub fn tick(f: &mut Fighter, g: &Gauge) {
    if f.gauge_mode > 0 {
        if g.halts && f.state == FighterState::ShieldBreak {
            return;
        }
        f.gauge_mode -= 1;
        if f.gauge_mode == 0 {
            f.gauge = Fx::ZERO;
        }
    } else if g.per_frame > Fx::ZERO && f.state != FighterState::Respawn {
        add(f, g, g.per_frame);
    }
}

/// How many fighters are in the match (2 to 4) and how many stocks fighter `i` is behind the leader (0 when ahead or level, or when
/// stocks are not counted).
fn standing(state: &GameState, i: usize) -> (u32, u8) {
    let players = state.roster.count_ones().clamp(2, 4);
    if state.rules.stocks == 0 {
        return (players, 0);
    }
    let lead = (0..crate::MAX_FIGHTERS)
        .filter(|&j| j != i && state.roster >> j & 1 == 1)
        .map(|j| state.fighters[j].stocks)
        .max()
        .unwrap_or(0);
    (players, lead.saturating_sub(state.fighters[i].stocks))
}

/// The gauge's per-mille multiplier for a match of `players` fighters `behind` stocks behind (its `crowd` row; 1000 for two).
fn crowd(g: &Gauge, players: u32, behind: u8) -> i32 {
    let col = usize::from(behind.min(2));
    match players {
        3 => i32::from(g.crowd[0][col]),
        4 => i32::from(g.crowd[1][col]),
        _ => 1000,
    }
}

/// A KO (E53): what the fighter's gauge holds as it comes back, by how far behind the leader it now is (`ko_keep`, times the gauge's
/// `crowd` row), the mode ended. 0 for the champion's gauge (owner decision 9).
pub fn after_ko(state: &mut GameState, content: &Content, i: usize) {
    let g = crate::combat::params_of(content, &state.fighters[i]).gauge;
    let (players, behind) = standing(state, i);
    let keep = match behind {
        0 | 1 => g.ko_keep[0],
        2 => g.ko_keep[1],
        _ => g.ko_keep[2],
    };
    let f = &mut state.fighters[i];
    f.gauge_mode = 0;
    f.gauge = (keep * Fx::from_ratio(crowd(&g, players, behind), 1000)).clamp(Fx::ZERO, g.max);
}

/// Spends the gauge: it empties and the powered mode ends (a special's `limit_version` was used).
pub fn spend(f: &mut Fighter) {
    f.gauge = Fx::ZERO;
    f.gauge_mode = 0;
}

/// Whether damage taken now drains the fighter's gauge instead of filling it: it is in a `Move::gauge_drain` move, or held by a grab
/// out of one.
pub fn draining(f: &Fighter, w: &Weapon) -> bool {
    f.gauge_drain || (f.state == FighterState::Attack && w.get(f.move_id).gauge_drain)
}

/// `damage` percent was just added to fighter `victim`'s percent by fighter `source`'s hit: the source gains its `per_dealt` share
/// (a fighter hitting itself gains nothing for it), the victim its `per_taken` share, or loses it while draining.
pub fn on_damage(
    state: &mut GameState,
    content: &Content,
    source: usize,
    victim: usize,
    damage: Fx,
) {
    gain(state, content, source, victim, damage, true);
}

/// Damage over time (E54) was just added to `victim`'s percent, credited to `source`: the victim's gauge sees it as damage taken,
/// but the source gains nothing for it (sim 123, owner decision 80: damage dealt counts hits only).
pub fn on_damage_over_time(
    state: &mut GameState,
    content: &Content,
    source: usize,
    victim: usize,
    damage: Fx,
) {
    gain(state, content, source, victim, damage, false);
}

fn gain(
    state: &mut GameState,
    content: &Content,
    source: usize,
    victim: usize,
    damage: Fx,
    dealt: bool,
) {
    if damage <= Fx::ZERO {
        return;
    }
    let pv = *crate::combat::params_of(content, &state.fighters[victim]);
    let wv = crate::combat::weapon_of(content, &pv);
    let drain = draining(&state.fighters[victim], wv);
    let g = pv.gauge;
    let (players, behind) = standing(state, victim);
    // (E53: while the mode runs, damage taken cuts its time instead: a hit that ends it fills nothing.)
    let in_mode = state.fighters[victim].gauge_mode > 0;
    if g.exists() && in_mode {
        let per = g.mode_taken[(players - 2) as usize];
        // (Rounded to the nearest frame [E: the reference's rounding is not published].)
        let cut = (per * damage + Fx::HALF)
            .floor_int()
            .clamp(0, i32::from(u16::MAX)) as u16;
        if cut > 0 {
            let f = &mut state.fighters[victim];
            f.gauge_mode = f.gauge_mode.saturating_sub(cut);
            if f.gauge_mode == 0 {
                f.gauge = Fx::ZERO;
            }
        }
    }
    // The gain from damage taken: the body's rate, by the stock gap and the crowd, and by the move it was hit in (E53).
    let mut rate = g.per_taken;
    if behind > 0 {
        rate = rate * Fx::from_ratio(i32::from(g.behind[usize::from(behind.min(2)) - 1]), 1000);
    }
    rate = rate * Fx::from_ratio(crowd(&g, players, behind), 1000);
    let f = &state.fighters[victim];
    if f.state == FighterState::Attack {
        let gain = wv.get(f.move_id).gauge_gain;
        if gain != 1000 {
            rate = rate * Fx::from_ratio(i32::from(gain), 1000);
        }
    }
    let taken = rate * damage;
    if !in_mode {
        add(
            &mut state.fighters[victim],
            &g,
            if drain { -taken } else { taken },
        );
    }
    if dealt && source != victim {
        let ps = *crate::combat::params_of(content, &state.fighters[source]);
        add(
            &mut state.fighters[source],
            &ps.gauge,
            ps.gauge.per_dealt * damage,
        );
    }
}
