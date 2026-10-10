//! A basic computer opponent, for testing and playing against alone.
//!
//! [`think`] reads the game state and returns the input one player would give this frame. It is a pure function of the state (no
//! memory, no clock, no outside randomness), so the same state always gives the same input: the match records a computer player's
//! inputs like anyone else's, replays play it back exactly, and rollback is unaffected. Its "choices" come from a hash of the frame
//! (in windows of a few frames, so a choice holds for a moment) and the player, never from the game's own random number generator.
//!
//! What it does, roughly in order of priority: mashes out of a grab, DIs toward the middle of the stage and techs in hitstun, picks a
//! ledge option after a moment on the ledge, recovers to the stage (air jump, then the up special), keeps to the stage while its
//! opponent is off it, shields an attack coming at it close in (and grabs out of the shield), and otherwise approaches and attacks
//! in range: jabs, tilts, grabs and the occasional smash on the ground, aerials toward the opponent in the air.
//!
//! `level` (1 to 9) is how quick and how sharp it is: a higher level decides more often, shields and techs more, and attacks more
//! readily. This is a sparring partner, not a full opponent: no spacing traps, punishes by frame data or habits.

use crate::content::Content;
use crate::fixed::Fx;
use crate::input::{buttons, Input};
use crate::state::{Fighter, FighterState as S, GameState};
use crate::MAX_FIGHTERS;

/// A number in `0..n`, the same for every frame of the `every`-frame window it falls in, for this player and `salt`.
fn roll(state: &GameState, me: usize, every: u32, salt: u32, n: u32) -> u32 {
    let mut x = (state.frame / every.max(1)).wrapping_mul(0x9E37_79B9)
        ^ (me as u32 + 1).wrapping_mul(0x85EB_CA6B)
        ^ salt.wrapping_mul(0xC2B2_AE35);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x % n.max(1)
}

/// `bit` pressed only if it was not held last frame, so holding a choice still makes one fresh press.
fn press(out: &mut Input, f: &Fighter, bit: u16) {
    if f.history[0].buttons & bit == 0 {
        out.buttons |= bit;
    }
}

/// Full stick toward `x` (from where the fighter stands).
fn toward(f: &Fighter, x: Fx) -> i8 {
    if x >= f.pos.x {
        127
    } else {
        -127
    }
}

/// The nearest other fighter still in the match.
fn nearest(state: &GameState, me: usize) -> Option<usize> {
    let f = &state.fighters[me];
    (0..MAX_FIGHTERS)
        .filter(|&i| i != me && state.fighters[i].active)
        .min_by_key(|&i| {
            let o = &state.fighters[i];
            ((o.pos.x - f.pos.x).abs() + (o.pos.y - f.pos.y).abs()).raw()
        })
}

/// The input player `me` gives this frame at `level` (1 to 9).
pub fn think(state: &GameState, content: &Content, me: usize, level: u8) -> Input {
    let mut out = Input::default();
    let Some(f) = state.fighters.get(me) else {
        return out;
    };
    if !f.active {
        return out;
    }
    let level = u32::from(level.clamp(1, 9));
    // How long a decision holds: about a fifth of a second at level 1, a tenth at level 9.
    let every = 14 - level;
    let chance = |salt: u32, percent: u32| roll(state, me, every, salt, 100) < percent;
    let (left, right, top) = content
        .stage
        .platforms
        .first()
        .map_or((Fx::from_int(-10), Fx::from_int(10), Fx::ZERO), |p| {
            (p.left, p.right, p.y)
        });
    let middle = (left + right) / Fx::from_int(2);

    match f.state {
        S::Grabbed => {
            // Mash: the stick from side to side and the jump button.
            out.stick_x = if state.frame % 2 == 0 { 127 } else { -127 };
            press(&mut out, f, buttons::JUMP);
            return out;
        }
        S::Hitstun => {
            // Survival DI: toward the middle of the stage and up.
            out.stick_x = if middle >= f.pos.x { 90 } else { -90 };
            out.stick_y = 90;
            // A tech, sometimes, when falling close to the ground.
            if f.vel.y < Fx::ZERO && f.pos.y - top < Fx::from_int(2) && chance(1, 10 * level) {
                press(&mut out, f, buttons::SHIELD);
            }
            return out;
        }
        S::LedgeHang => {
            // A moment on the ledge (shorter at higher levels), then a get-up: climb, jump or roll.
            if u32::from(f.state_frame) < 6 + (9 - level) * 3 {
                return out;
            }
            match roll(state, me, every, 2, 3) {
                0 => out.stick_x = toward(f, middle),
                1 => press(&mut out, f, buttons::JUMP),
                _ => press(&mut out, f, buttons::SHIELD),
            }
            return out;
        }
        S::Knockdown => {
            // Get up: in place, or rolling toward the middle.
            if chance(3, 50) {
                out.stick_x = toward(f, middle);
            } else {
                out.stick_y = 127;
            }
            return out;
        }
        _ => {}
    }

    // Off the stage: head back. An air jump when falling below the stage's top, then the up special once the jumps are spent.
    let off_stage = !f.grounded() && (f.pos.x < left || f.pos.x > right || f.pos.y < top - Fx::ONE);
    if off_stage && f.state != S::Attack {
        out.stick_x = toward(f, middle);
        if f.vel.y < Fx::ZERO && f.pos.y < top + Fx::from_int(2) {
            if f.air_jumps_left > 0 {
                press(&mut out, f, buttons::JUMP);
            } else if f.state != S::Helpless && f.pos.y < top {
                out.stick_y = 127;
                out.stick_x = 0;
                press(&mut out, f, buttons::SPECIAL);
            }
        }
        return out;
    }

    let Some(t) = nearest(state, me) else {
        return out;
    };
    let o = &state.fighters[t];
    let dx = o.pos.x - f.pos.x;
    let reach_x = dx.abs();
    let dy = o.pos.y - f.pos.y;
    let facing_it = (dx >= Fx::ZERO) == (f.facing > 0);

    // The opponent is off the stage: wait near the edge it went off, without following it out.
    let edge_margin = Fx::from_int(1);
    if f.grounded() && (o.pos.x < left || o.pos.x > right) {
        let post = o.pos.x.clamp(left + edge_margin, right - edge_margin);
        if (post - f.pos.x).abs() > Fx::HALF {
            out.stick_x = if post > f.pos.x { 60 } else { -60 };
        }
        return out;
    }

    // An attack coming at it close in: shield (more often at higher levels), and grab out of the shield when the attacker is
    // right there.
    if f.grounded() && o.state == S::Attack && reach_x < Fx::from_int(3) && chance(4, 8 * level) {
        out.buttons |= buttons::SHIELD;
        return out;
    }
    if f.state == S::Shield {
        if reach_x < Fx::from_int(2) && o.state != S::Attack && chance(5, 40 + 5 * level) {
            press(&mut out, f, buttons::GRAB);
        }
        return out;
    }

    if f.grounded() {
        let near = reach_x < Fx::from_int(2) + Fx::HALF && dy.abs() < Fx::from_int(2);
        if !near {
            // Approach, staying on the stage: a run, or a jump toward one standing above.
            let ahead = f.pos.x + dx.clamp(-Fx::ONE, Fx::ONE);
            if ahead > left + Fx::HALF && ahead < right - Fx::HALF {
                out.stick_x = toward(f, o.pos.x);
            }
            if dy > Fx::from_int(2) && chance(6, 30 + 4 * level) {
                press(&mut out, f, buttons::JUMP);
            }
            return out;
        }
        if !facing_it {
            // Turn round first, with a gentle push (not a dash).
            out.stick_x = if dx >= Fx::ZERO { 40 } else { -40 };
            return out;
        }
        // In range: attack more readily at higher levels; otherwise hold position for a moment.
        if !chance(7, 35 + 6 * level) {
            return out;
        }
        let side: i8 = if dx >= Fx::ZERO { 1 } else { -1 };
        match roll(state, me, every, 8, 10) {
            0..=2 => press(&mut out, f, buttons::ATTACK),
            3 | 4 => {
                out.stick_x = 50 * side;
                press(&mut out, f, buttons::ATTACK);
            }
            5 => {
                out.stick_y = -80;
                press(&mut out, f, buttons::ATTACK);
            }
            6 if dy > Fx::ONE => {
                out.stick_y = 80;
                press(&mut out, f, buttons::ATTACK);
            }
            6 | 7 => press(&mut out, f, buttons::GRAB),
            _ => {
                out.stick_x = 127 * side;
                out.buttons |= buttons::STRONG;
                press(&mut out, f, buttons::ATTACK);
            }
        }
        return out;
    }

    // In the air over the stage: drift toward the opponent and swing at it in reach (the stick toward it makes a forward aerial,
    // or a back aerial when facing away; up or down for one above or below).
    out.stick_x = toward(f, o.pos.x);
    if reach_x < Fx::from_int(2) + Fx::HALF && dy.abs() < Fx::from_int(2) + Fx::HALF {
        if dy > Fx::ONE {
            out.stick_x = 0;
            out.stick_y = 100;
        } else if dy < -Fx::ONE {
            out.stick_x = 0;
            out.stick_y = -80;
        }
        press(&mut out, f, buttons::ATTACK);
    } else if f.vel.y < Fx::ZERO && f.history[0].stick_y == 0 && chance(9, 4 * level) {
        // A fast fall now and then.
        out.stick_y = -127;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{step, Content};

    fn match_of(level: u8, frames: u32) -> GameState {
        let content = Content::placeholder();
        let mut state = GameState::new(&content, 7, [0, 1, 0, 1]);
        state.fighters[2].active = false;
        state.fighters[3].active = false;
        for _ in 0..frames {
            let mut inputs = [Input::default(); MAX_FIGHTERS];
            inputs[0] = think(&state, &content, 0, level);
            inputs[1] = think(&state, &content, 1, level);
            step(&mut state, &content, &inputs);
        }
        state
    }

    #[test]
    fn the_same_state_always_gives_the_same_input() {
        let a = match_of(5, 600);
        let b = match_of(5, 600);
        assert_eq!(a.checksum(), b.checksum());
    }

    #[test]
    fn two_computer_players_fight_each_other() {
        // Over twenty seconds they find each other and land hits.
        let s = match_of(7, 1200);
        let damage = s.fighters[0].percent + s.fighters[1].percent;
        assert!(damage > Fx::from_int(30), "only {damage:?} damage dealt");
    }
}
