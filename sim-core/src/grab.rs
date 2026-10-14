//! Grabs: catching a fighter, holding it (with a time limit that mashing shortens), and letting it go.
//!
//! The holder is in [`FighterState::Grabbing`] (or in a pummel or throw, which are ordinary moves, or carrying the held fighter:
//! [`FighterState::Carrying`], E7), the held
//! fighter in [`FighterState::Grabbed`]; both point at each other with `grab_with`. The throw and pummel hits are
//! resolved in `combat::resolve_hits`; this module keeps the pair consistent and pinned together every frame.

use crate::collision;
use crate::combat;
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
            S::Hitstun
                | S::LedgeHang
                | S::LedgeGetUp
                | S::LedgeAttack
                | S::Grabbed
                | S::Grabbing
                | S::Carrying
                | S::Buried
                | S::Encased
        )
}

/// True if a command grab (`Hitbox::command`) can catch this fighter: on the ground or in the air, shielding or in hitstun, as long
/// as it is not already tangled up in a grab, not just released from one, and not hanging from (or climbing up) a ledge.
pub fn catchable(f: &Fighter) -> bool {
    f.grab_immune == 0
        && f.grab_with == NONE
        && !matches!(
            f.state,
            S::LedgeHang
                | S::LedgeGetUp
                | S::LedgeAttack
                | S::Grabbed
                | S::Grabbing
                | S::Carrying
                | S::Respawn
                | S::Buried
                | S::Encased
        )
}

/// Whether hitbox `hb` can catch `f` (an ordinary grab or a command grab).
pub fn can_catch(hb: &crate::moves::Hitbox, f: &Fighter) -> bool {
    if hb.command {
        catchable(f)
    } else {
        grabbable(f)
    }
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
    let facing = -state.fighters[a].facing;
    let holder = &state.fighters[a];
    let weapon = combat::weapon_of(content, combat::params_of(content, holder));
    let catch = weapon.get(holder.move_id).catch_move;
    if let Some(next) = catch {
        // A command grab: the holder goes on with its catch move (next frame), keeping its speed, in the air if it is there; the
        // held fighter has no hold clock (the clock only runs while the holder simply holds), unless the catch move is a latch with
        // one (E49: the ordinary hold time, scaled).
        let timer = match weapon.get(next).latch {
            Some(l) if l.clock_num > 0 => {
                let full = i32::from(hold_time(content, state.fighters[d].percent))
                    * i32::from(l.clock_num)
                    / i32::from(l.clock_den.max(1));
                full.clamp(1, i32::from(u16::MAX)) as u16
            }
            _ => 0,
        };
        let h = &mut state.fighters[a];
        h.grab_with = d as i8;
        h.next_move = next.saturating_add(1);
        fighter::become_held(&mut state.fighters[d], a, facing, timer);
    } else {
        // An ordinary grab out of a gauge-draining move (the reference's charge) keeps draining the held fighter's gauge on every hit
        // until the hold ends (not a command grab: the reference's rule).
        let victim = &state.fighters[d];
        let drain = crate::gauge::draining(
            victim,
            combat::weapon_of(content, combat::params_of(content, victim)),
        );
        let timer = hold_time(content, state.fighters[d].percent);
        fighter::become_holder(&mut state.fighters[a], d);
        fighter::become_held(&mut state.fighters[d], a, facing, timer);
        state.fighters[d].gauge_drain = drain;
        // (Kept for a carry, which measures the hold time already used from it.)
        state.fighters[d].grab_start = timer;
    }
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
        f.grab_start = 0;
        if was_held {
            fighter::free_held(f, content.rules.grab_immunity);
        } else if matches!(f.state, S::Grabbing | S::Attack | S::Carrying) {
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
            && matches!(
                state.fighters[h].state,
                S::Grabbing | S::Attack | S::Carrying
            );
        if !consistent {
            drop_grab(state, content, h);
            continue;
        }
        start_carry_clock(state, content, h, v);
        pin(state, content, h, v);
        // The clock only runs while the holder is simply holding or carrying (not mid-pummel or mid-throw), or in a latch with a
        // clock (E49).
        let holder = &state.fighters[h];
        let latched =
            holder.state == S::Attack && latch_of(content, holder).is_some_and(|l| l.clock_num > 0);
        if !(matches!(holder.state, S::Grabbing | S::Carrying) || latched)
            || state.fighters[v].hitlag > 0
        {
            continue;
        }
        let cut = mash_cut(&state.fighters[v], &content.rules);
        let left = state.fighters[v].grab_timer.saturating_sub(1 + cut);
        state.fighters[v].grab_timer = left;
        if left == 0 {
            drop_grab(state, content, h);
        }
    }
}

/// A carry's clock (E7, see `moves::Carry`), set on the first frame of its lift: the hold time at the held fighter's percent now, times
/// `clock_num / clock_den`, less the hold time already used (`g`: from the clock as it was set on the catch, mashing included), at
/// least one frame. The catch's clock (`grab_start`) is cleared once used, so this happens once per hold. (The clock stands still during the lift,
/// as in any move made while holding, so setting it at the lift's start or end is the same.)
fn start_carry_clock(state: &mut GameState, content: &Content, h: usize, v: usize) {
    let holder = &state.fighters[h];
    if holder.state != S::Attack || holder.state_frame != 0 || state.fighters[v].grab_start == 0 {
        return;
    }
    let weapon = combat::weapon_of(content, combat::params_of(content, holder));
    let Some(carry) = weapon.get(holder.move_id).carry else {
        return;
    };
    let held = &mut state.fighters[v];
    let used = i32::from(held.grab_start.saturating_sub(held.grab_timer));
    let full = i32::from(hold_time(content, held.percent)) * i32::from(carry.clock_num)
        / i32::from(carry.clock_den.max(1));
    held.grab_timer = (full - used).clamp(1, i32::from(u16::MAX)) as u16;
    held.grab_start = 0;
}

/// The frames this frame's mashing takes off a struggle (a hold, a bury: the shared mash): `grab_mash_button` for a new press of any
/// button, `grab_mash_stick` for a flick of the stick (the reference's 14.4 and 8, rounded).
pub fn mash_cut(f: &Fighter, rules: &crate::content::Ruleset) -> u16 {
    let (pressed, flicked) = mash_inputs(f);
    let mut cut: u16 = 0;
    if pressed {
        cut += u16::from(rules.grab_mash_button);
    }
    if flicked {
        cut += u16::from(rules.grab_mash_stick);
    }
    cut
}

/// The shared mash reading (a hold, a bury, an egg): whether this frame has a new press of any button, and whether it has a flick of
/// the stick.
pub fn mash_inputs(f: &Fighter) -> (bool, bool) {
    let pressed = [
        buttons::ATTACK,
        buttons::JUMP,
        buttons::SHIELD,
        buttons::SPECIAL,
        buttons::GRAB,
    ]
    .iter()
    .any(|b| f.pressed_within(*b, 1));
    (pressed, f.flick_x(1) != 0 || f.flick_y(1) != 0)
}

/// The holder's move as far as holding goes: the catch move it is about to start (`next_move`), else its move.
fn holding_move(holder: &Fighter) -> u8 {
    if holder.next_move != 0 {
        holder.next_move - 1
    } else {
        holder.move_id
    }
}

/// The latch (E49) of the move a holder in an attack holds its fighter in, if it has one.
fn latch_of(content: &Content, holder: &Fighter) -> Option<crate::moves::Latch> {
    if holder.state != S::Attack {
        return None;
    }
    combat::weapon_of(content, combat::params_of(content, holder))
        .get(holding_move(holder))
        .latch
}

/// Stands the held fighter `grab_distance` in front of the holder, on the same surface, facing it; or, when the holder's move (or
/// the catch move it is about to start) has a `hold_at`, holds it there (in the air if the holder is, or if it is held up).
fn pin(state: &mut GameState, content: &Content, h: usize, v: usize) {
    let holder = state.fighters[h];
    if let Some(l) = latch_of(content, &holder) {
        // A latch (E49): the held fighter falls on its own and the holder rides it.
        let p = *combat::params_of(content, &state.fighters[v]);
        // (Hitlag freezes the fall, as it freezes everything else.)
        if state.fighters[v].hitlag == 0 {
            fighter::held_fall(&mut state.fighters[v], &p, &content.stage);
        }
        let held = state.fighters[v];
        let up = p.ecb_height * Fx::from_ratio(i32::from(l.height), 100);
        let rider = &mut state.fighters[h];
        rider.pos = held.pos + Vec2::new(l.x.mul_int(i32::from(rider.facing)), up);
        rider.vel = held.vel;
        rider.platform = if up <= Fx::ZERO { held.platform } else { NONE };
        state.fighters[v].facing = -state.fighters[h].facing;
        return;
    }
    let weapon = combat::weapon_of(content, combat::params_of(content, &holder));
    let current = holding_move(&holder);
    let hold_at = match holder.state {
        S::Attack => weapon.get(current).hold_at,
        // (A carry holds its fighter where its lift's `Carry::hold_at` says.)
        S::Carrying => weapon.get(holder.move_id).carry.map(|c| c.hold_at),
        _ => None,
    };
    if let Some((dx, dy)) = hold_at {
        let held = &mut state.fighters[v];
        held.pos = holder.pos + Vec2::new(dx.mul_int(i32::from(holder.facing)), dy);
        held.vel = Vec2::ZERO;
        held.platform = if holder.platform != NONE && dy <= Fx::ZERO {
            holder.platform
        } else {
            NONE
        };
        held.facing = -holder.facing;
        return;
    }
    let mut x = holder.pos.x
        + content
            .rules
            .grab_distance
            .mul_int(i32::from(holder.facing));
    let mut y = holder.pos.y;
    if let Some(p) = collision::platform(&content.stage, holder.platform) {
        x = x.clamp(p.left, p.right);
        y = p.top_at(x);
    }
    let held = &mut state.fighters[v];
    held.pos = Vec2::new(x, y);
    held.vel = Vec2::ZERO;
    held.platform = holder.platform;
    held.facing = -holder.facing;
}
