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

/// One frame of the powered mode's timer (every frame, hitlag included [E: gap 12]); when it runs out the gauge is empty again.
pub fn tick(f: &mut Fighter) {
    if f.gauge_mode > 0 {
        f.gauge_mode -= 1;
        if f.gauge_mode == 0 {
            f.gauge = Fx::ZERO;
        }
    }
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
    if damage <= Fx::ZERO {
        return;
    }
    let pv = *crate::combat::params_of(content, &state.fighters[victim]);
    let drain = draining(
        &state.fighters[victim],
        crate::combat::weapon_of(content, &pv),
    );
    let taken = pv.gauge.per_taken * damage;
    add(
        &mut state.fighters[victim],
        &pv.gauge,
        if drain { -taken } else { taken },
    );
    if source != victim {
        let ps = *crate::combat::params_of(content, &state.fighters[source]);
        add(
            &mut state.fighters[source],
            &ps.gauge,
            ps.gauge.per_dealt * damage,
        );
    }
}
