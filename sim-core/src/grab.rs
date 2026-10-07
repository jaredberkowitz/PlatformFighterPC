//! Grabs: catching a fighter, holding it (with a time limit that mashing shortens), and letting it go.
//!
//! The holder is in [`FighterState::Grabbing`] (or in a pummel or throw, which are ordinary moves), the held
//! fighter in [`FighterState::Grabbed`]; both point at each other with `grab_with`. The throw and pummel hits are
//! resolved in `combat::resolve_hits`; this module keeps the pair consistent and pinned together every frame.

use crate::collision;
use crate::content::Content;
use crate::fighter;
use crate::fixed::Fx;
use crate::input::buttons;
use crate::state::{Fighter, FighterState as S, GameState, NONE};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

/// True if this fighter can be caught: standing on something, not already tangled up, not recently released,
/// and not in hitstun or hanging from a ledge.
pub fn grabbable(f: &Fighter) -> bool {
    f.grounded()
        && f.grab_immune == 0
        && f.grab_with == NONE
        && !matches!(
            f.state,
            S::Hitstun | S::LedgeHang | S::LedgeGetUp | S::LedgeAttack | S::Grabbed | S::Grabbing
        )
}

/// Frames a fighter at `percent` stays held before breaking free on its own.
fn hold_time(content: &Content, percent: Fx) -> u16 {
    let r = &content.rules;
    let more = percent.floor_int().clamp(0, 999) * i32::from(r.grab_percent_tenths) / 10;
    (i32::from(r.grab_base_frames) + more).max(i32::from(r.grab_min_frames)) as u16
}

/// Puts `d` in `a`'s hands.
pub fn connect(state: &mut GameState, content: &Content, a: usize, d: usize) {
    // A fighter that was itself holding someone lets go first.
    drop_grab(state, content, d);
    let timer = hold_time(content, state.fighters[d].percent);
    let facing = -state.fighters[a].facing;
    fighter::become_holder(&mut state.fighters[a], d);
    fighter::become_held(&mut state.fighters[d], a, facing, timer);
    pin(state, content, a, d);
}

/// Everything that ends a grab other than a throw: the holder or the held fighter was hit, was knocked out, or the
/// held fighter broke free. Frees whoever `i` is paired with and puts both in a neutral state.
pub fn drop_grab(state: &mut GameState, content: &Content, i: usize) {
    let Ok(j) = usize::try_from(state.fighters[i].grab_with) else {
        return;
    };
    for k in [i, j] {
        let Some(f) = state.fighters.get_mut(k) else {
            continue;
        };
        let was_held = f.state == S::Grabbed;
        f.grab_with = NONE;
        f.grab_timer = 0;
        if was_held {
            fighter::free_held(f, content.rules.grab_immunity);
        } else if matches!(f.state, S::Grabbing | S::Attack) {
            fighter::free_holder(f, content.rules.grab_release_lag);
        }
    }
}

/// Keeps the held fighter in front of the holder, counts down the hold, and frees the pair if anything is out of step.
pub fn update(state: &mut GameState, content: &Content) {
    for h in 0..MAX_FIGHTERS {
        let Ok(v) = usize::try_from(state.fighters[h].grab_with) else {
            continue;
        };
        // Only the holder drives the pair; the held fighter's side is handled with it.
        if state.fighters[h].state == S::Grabbed {
            continue;
        }
        let consistent = v < MAX_FIGHTERS
            && state.fighters[v].grab_with == h as i8
            && state.fighters[v].state == S::Grabbed
            && matches!(state.fighters[h].state, S::Grabbing | S::Attack);
        if !consistent {
            drop_grab(state, content, h);
            continue;
        }
        pin(state, content, h, v);
        // The clock only runs while the holder is simply holding (not mid-pummel or mid-throw).
        if state.fighters[h].state != S::Grabbing || state.fighters[v].hitlag > 0 {
            continue;
        }
        let held = &state.fighters[v];
        let mut cut: u16 = 0;
        let pressed = [
            buttons::ATTACK,
            buttons::JUMP,
            buttons::SHIELD,
            buttons::SPECIAL,
            buttons::GRAB,
        ]
        .iter()
        .any(|b| held.pressed_within(*b, 1));
        if pressed {
            cut += u16::from(content.rules.grab_mash_button);
        }
        if held.flick_x(1) != 0 || held.flick_y(1) != 0 {
            cut += u16::from(content.rules.grab_mash_stick);
        }
        let left = state.fighters[v].grab_timer.saturating_sub(1 + cut);
        state.fighters[v].grab_timer = left;
        if left == 0 {
            drop_grab(state, content, h);
        }
    }
}

/// Stands the held fighter `grab_distance` in front of the holder, on the same surface, facing it.
fn pin(state: &mut GameState, content: &Content, h: usize, v: usize) {
    let holder = state.fighters[h];
    let mut x = holder.pos.x
        + content
            .rules
            .grab_distance
            .mul_int(i32::from(holder.facing));
    let mut y = holder.pos.y;
    if let Some(p) = collision::platform(&content.stage, holder.platform) {
        x = x.clamp(p.left, p.right);
        y = p.y;
    }
    let held = &mut state.fighters[v];
    held.pos = Vec2::new(x, y);
    held.vel = Vec2::ZERO;
    held.platform = holder.platform;
    held.facing = -holder.facing;
}
