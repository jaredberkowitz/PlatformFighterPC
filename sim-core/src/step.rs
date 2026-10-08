//! The one entry point that advances the simulation by exactly one 60 Hz frame.

use crate::combat;
use crate::content::{Content, FighterParams};
use crate::fighter;
use crate::grab;
use crate::input::Input;
use crate::state::{Fighter, FighterState, GameState, DRAW, NONE, PLAYING};
use crate::MAX_FIGHTERS;

fn params_of<'a>(content: &'a Content, f: &Fighter) -> &'a FighterParams {
    let idx = usize::from(f.char_id).min(content.fighters.len().saturating_sub(1));
    &content.fighters[idx]
}

/// Frames per second of the simulation, for the match time limit.
const FPS: u32 = 60;

/// Decides whether the match is over: one fighter (or none) left standing, or the time ran out.
fn settle(state: &mut GameState) {
    if state.winner != PLAYING || state.roster.count_ones() < 2 {
        return;
    }
    let mut alive = (0..MAX_FIGHTERS).filter(|&i| state.fighters[i].active);
    let first = alive.next();
    let second = alive.next();
    state.winner = match (first, second) {
        (None, _) => DRAW,
        (Some(only), None) => only as i8,
        (Some(_), Some(_))
            if state.rules.time_limit > 0
                && state.frame >= u32::from(state.rules.time_limit) * FPS =>
        {
            // Most stocks, then least damage; level on both is a draw.
            let key = |i: usize| (state.fighters[i].stocks, state.fighters[i].percent);
            let beats = |a: usize, b: usize| {
                let (sa, pa) = key(a);
                let (sb, pb) = key(b);
                sa > sb || (sa == sb && pa < pb)
            };
            let mut best = 0;
            for i in (0..MAX_FIGHTERS).filter(|&i| state.fighters[i].active) {
                if beats(i, best) || !state.fighters[best].active {
                    best = i;
                }
            }
            let tied = (0..MAX_FIGHTERS)
                .any(|i| i != best && state.fighters[i].active && key(i) == key(best));
            if tied {
                DRAW
            } else {
                best as i8
            }
        }
        _ => PLAYING,
    };
}

pub fn step(state: &mut GameState, content: &Content, inputs: &[Input; MAX_FIGHTERS]) {
    // A finished match stands still.
    if state.winner != PLAYING {
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    let stage = &content.stage;

    // Phase 1: every fighter updates in player-index order and may request a ledge.
    let mut wants: [Option<u8>; MAX_FIGHTERS] = [None; MAX_FIGHTERS];
    for (i, (f, input)) in state.fighters.iter_mut().zip(inputs.iter()).enumerate() {
        if !f.active {
            continue;
        }
        let params = params_of(content, f);
        let weapon = combat::weapon_of(content, params);
        wants[i] = fighter::update(f, params, weapon, stage, &content.rules, *input);
    }

    // Phase 1b: hits. Everything has moved, so hitboxes and hurtboxes are where they will be seen.
    combat::resolve_hits(state, content);
    // Grabs: keep each held fighter in its holder's hands and let it break free.
    grab::update(state, content);
    // Existing projectiles move first, so a new one stays at the muzzle on the frame it appears.
    combat::update_projectiles(state, content);
    combat::spawn_projectiles(state, content);

    // Phase 2: ledge ownership.
    // Free ledges whose occupant left them (dropped, got up, jumped, or was hit).
    for (l, owner) in state.ledge_owner.iter_mut().enumerate() {
        if let Ok(o) = usize::try_from(*owner) {
            let f = &state.fighters[o];
            if f.state != FighterState::LedgeHang || usize::try_from(f.ledge) != Ok(l) {
                *owner = NONE;
            }
        }
    }
    // Contested grabs are decided by a fixed rule: the lowest player index wins, and a winner
    // trumps (knocks off) any current occupant. Losers simply keep falling.
    for l in 0..stage.ledges.len().min(state.ledge_owner.len()) {
        let Some(winner) = wants.iter().position(|w| *w == Some(l as u8)) else {
            continue;
        };
        if let Ok(o) = usize::try_from(state.ledge_owner[l]) {
            let params = *params_of(content, &state.fighters[o]);
            fighter::trump(&mut state.fighters[o], &params, stage);
        }
        let params = *params_of(content, &state.fighters[winner]);
        fighter::grab_ledge(&mut state.fighters[winner], &params, stage, l);
        state.ledge_owner[l] = winner as i8;
    }

    // Phase 3: fighters who left the blast zone lose a stock and respawn.
    combat::check_ko(state, content);

    state.frame = state.frame.wrapping_add(1);
    settle(state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzz::random_inputs;
    use crate::rng::Rng;

    fn run(content: &Content, seed: u64, inputs: &[[Input; MAX_FIGHTERS]]) -> GameState {
        let mut s = GameState::new(content, seed, [0, 1, 0, 1]);
        for i in inputs {
            step(&mut s, content, i);
        }
        s
    }

    #[test]
    fn same_inputs_same_checksum() {
        let content = Content::placeholder();
        let inputs = random_inputs(&mut Rng::new(1), 600);
        assert_eq!(
            run(&content, 9, &inputs).checksum(),
            run(&content, 9, &inputs).checksum()
        );
    }

    #[test]
    fn different_inputs_diverge() {
        let content = Content::placeholder();
        let a = random_inputs(&mut Rng::new(1), 600);
        let b = random_inputs(&mut Rng::new(2), 600);
        assert_ne!(
            run(&content, 9, &a).checksum(),
            run(&content, 9, &b).checksum()
        );
    }

    #[test]
    fn restore_then_replay_matches_straight_run() {
        let content = Content::placeholder();
        let inputs = random_inputs(&mut Rng::new(3), 400);

        let mut s = GameState::new(&content, 1, [0, 1, 0, 1]);
        for i in &inputs[..150] {
            step(&mut s, &content, i);
        }
        let saved = s;
        for i in &inputs[150..] {
            step(&mut s, &content, i);
        }
        let straight = s.checksum();

        let mut r = saved;
        for i in &inputs[150..] {
            step(&mut r, &content, i);
        }
        assert_eq!(r.checksum(), straight);
    }

    /// Random play must never panic or leave a fighter in an inconsistent state.
    #[test]
    fn random_play_keeps_invariants() {
        let content = Content::placeholder();
        for seed in 0..40u64 {
            let inputs = random_inputs(&mut Rng::new(seed), 1500);
            let mut s = GameState::new(&content, seed, [0, 1, 0, 1]);
            for i in &inputs {
                step(&mut s, &content, i);
                for (n, f) in s.fighters.iter().enumerate() {
                    for b in content.stage.platforms.iter().filter(|b| !b.pass_through) {
                        let inside = f.pos.x > b.left
                            && f.pos.x < b.right
                            && f.pos.y > b.bottom
                            && f.pos.y < b.y;
                        assert!(
                            !inside,
                            "seed {seed} fighter {n}: feet inside a solid block at {:?} in {:?}",
                            f.pos, f.state
                        );
                    }
                    // A grab is always mutual, and only grabbing or grabbed fighters are in one.
                    if f.grab_with != NONE {
                        let other = &s.fighters[f.grab_with as usize];
                        assert_eq!(
                            other.grab_with, n as i8,
                            "seed {seed} fighter {n}: one-sided grab"
                        );
                    }
                    if matches!(f.state, FighterState::Grabbing | FighterState::Grabbed) {
                        assert_ne!(
                            f.grab_with, NONE,
                            "seed {seed} fighter {n}: {:?} without a partner",
                            f.state
                        );
                    }
                    let on_ledge = f.state == FighterState::LedgeHang;
                    assert_eq!(
                        on_ledge,
                        f.ledge != NONE,
                        "seed {seed} fighter {n}: {:?}",
                        f.state
                    );
                    if on_ledge {
                        assert_eq!(s.ledge_owner[f.ledge as usize], n as i8);
                    }
                    let grounded_state = matches!(
                        f.state,
                        FighterState::Idle
                            | FighterState::Walk
                            | FighterState::Run
                            | FighterState::Dash
                            | FighterState::Turn
                            | FighterState::Crouch
                            | FighterState::JumpSquat
                            | FighterState::Landing
                            | FighterState::WaveLand
                            | FighterState::Shield
                            | FighterState::Roll
                            | FighterState::SpotDodge
                            | FighterState::Grabbing
                            | FighterState::Grabbed
                            | FighterState::Knockdown
                            | FighterState::GetUp
                            | FighterState::LedgeGetUp
                            | FighterState::LedgeAttack
                    );
                    // Attacks and hitstun can happen on the ground or in the air.
                    if !matches!(
                        f.state,
                        FighterState::Attack | FighterState::Hitstun | FighterState::ShieldBreak
                    ) {
                        assert_eq!(
                            grounded_state,
                            f.grounded(),
                            "seed {seed} fighter {n}: {:?}",
                            f.state
                        );
                    }
                }
            }
        }
    }
}
