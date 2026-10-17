//! The one entry point that advances the simulation by exactly one 60 Hz frame.

use crate::combat;
use crate::content::{Content, FighterParams};
use crate::fighter;
use crate::fixed::Fx;
use crate::grab;
use crate::input::Input;
use crate::state::{Fighter, FighterState, GameState, Item, Projectile, DRAW, NONE, PLAYING};
use crate::{MAX_BODIES, MAX_FIGHTERS};

fn params_of<'a>(content: &'a Content, f: &Fighter) -> &'a FighterParams {
    let idx = usize::from(f.char_id).min(content.fighters.len().saturating_sub(1));
    &content.fighters[idx]
}

/// Frames per second of the simulation, for the match time limit.
const FPS: u32 = 60;

/// Damage every fighter starts sudden death with (the reference's).
const SUDDEN_DEATH_PERCENT: i32 = 300;

/// Decides whether the match is over: one fighter (or none) left standing, or the time ran out. When the time runs out with the
/// leaders level on stocks, the match goes to sudden death instead (`start_sudden_death`) and only a knock-out ends it.
fn settle(state: &mut GameState, content: &Content) {
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
            if !state.sudden_death
                && state.rules.time_limit > 0
                && state.frame >= u32::from(state.rules.time_limit) * FPS =>
        {
            if state.rules.stocks > 0 {
                // A stock match: most stocks wins; the leaders level on stocks go to sudden death.
                let most = (0..MAX_FIGHTERS)
                    .filter(|&i| state.fighters[i].active)
                    .map(|i| state.fighters[i].stocks)
                    .max()
                    .unwrap_or(0);
                let mut leaders = (0..MAX_FIGHTERS)
                    .filter(|&i| state.fighters[i].active && state.fighters[i].stocks == most);
                match (leaders.next(), leaders.next()) {
                    (Some(one), None) => one as i8,
                    _ => {
                        start_sudden_death(state, content, most);
                        PLAYING
                    }
                }
            } else {
                // Free play against the clock: the least damage wins; level is a draw.
                let mut best = 0;
                for i in (0..MAX_FIGHTERS).filter(|&i| state.fighters[i].active) {
                    if !state.fighters[best].active
                        || state.fighters[i].percent < state.fighters[best].percent
                    {
                        best = i;
                    }
                }
                let tied = (0..MAX_FIGHTERS).any(|i| {
                    i != best
                        && state.fighters[i].active
                        && state.fighters[i].percent == state.fighters[best].percent
                });
                if tied {
                    DRAW
                } else {
                    best as i8
                }
            }
        }
        _ => PLAYING,
    };
}

/// Sudden death: the fighters with `most` stocks start again from their spawn points with one stock at 300%; everyone else is out.
/// The stage is cleared of projectiles, items and ledge holds. The clock no longer runs out.
fn start_sudden_death(state: &mut GameState, content: &Content, most: u8) {
    state.sudden_death = true;
    for i in 0..MAX_FIGHTERS {
        let old = state.fighters[i];
        if !old.active {
            continue;
        }
        if old.stocks != most {
            state.fighters[i].active = false;
            state.fighters[i].stocks = 0;
            continue;
        }
        let pos = content.stage.spawns[i];
        let params = params_of(content, &old);
        let mut f = Fighter::spawn(
            pos,
            old.char_id,
            if i % 2 == 0 { 1 } else { -1 },
            crate::collision::standing_on(&content.stage, pos),
            params.air_jumps,
            content.rules.shield_max,
        );
        f.active = true;
        f.stocks = 1;
        f.percent = Fx::from_int(SUDDEN_DEATH_PERCENT);
        state.fighters[i] = f;
    }
    state.projectiles = [Projectile::default(); crate::state::MAX_PROJECTILES];
    state.items = [Item::default(); crate::state::MAX_ITEMS];
    state.ledge_owner = [NONE; crate::state::LEDGE_SLOTS];
}

/// Advances one frame with the players' inputs (the grunts' slots, empty outside a run, read none).
pub fn step(state: &mut GameState, content: &Content, inputs: &[Input; MAX_FIGHTERS]) {
    let mut all = [Input::default(); MAX_BODIES];
    all[..MAX_FIGHTERS].copy_from_slice(inputs);
    step_bodies(state, content, &all);
}

/// Advances one frame with an input for every body: the players', then the grunts' (the run's grunt layer makes those, sim 131).
pub fn step_bodies(state: &mut GameState, content: &Content, inputs: &[Input; MAX_BODIES]) {
    // A finished match stands still.
    if state.winner != PLAYING {
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    let stage = &content.stage;

    // Phase 1: every fighter updates in player-index order and may request a ledge.
    let mut wants: [Option<u8>; MAX_BODIES] = [None; MAX_BODIES];
    let nb = crate::state::bodies(state);
    let mut before = [Fx::ZERO; MAX_BODIES];
    for (b, f) in before.iter_mut().zip(state.fighters.iter()).take(nb) {
        *b = f.pos.x;
    }
    // Which fighters have an item in reach as the frame begins (E4: an attack or grab press picks it up).
    let pickable = crate::items::pickable(state, content);
    // What move scripts see of the others this frame (a target lock): every chest, and which fighters can be targeted.
    let mut world = crate::scripting::World {
        stage,
        targets: [None; MAX_BODIES],
        chest: [Fx::ZERO; MAX_BODIES],
        me: 0,
        projectiles: &state.projectiles,
        encase: [None; MAX_BODIES],
        items: &state.items,
        pickable,
        ledges: state.ledge_owner,
        heads: [None; MAX_BODIES],
    };
    for (i, f) in state.fighters.iter().enumerate().take(nb) {
        let height = combat::chest_height(params_of(content, f));
        world.chest[i] = height;
        // (An egged fighter reads its egg's numbers from the move that laid it: E8.)
        world.encase[i] = combat::encase_of(content, f);
        if f.active && f.stocks > 0 && !combat::is_intangible(f) {
            world.targets[i] = Some(f.pos + crate::vec2::Vec2::new(Fx::ZERO, height));
            if fighter::footstoolable(f) {
                let p = params_of(content, f);
                world.heads[i] = Some((
                    f.pos + crate::vec2::Vec2::new(Fx::ZERO, p.ecb_height),
                    p.ecb_half_width,
                ));
            }
        }
    }
    for (i, (f, input)) in state
        .fighters
        .iter_mut()
        .zip(inputs.iter())
        .enumerate()
        .take(nb)
    {
        if !f.active {
            continue;
        }
        // (While the gauge's powered mode runs, or its wing is out (E57), the body moves with its multiplied numbers.)
        let params = params_of(content, f).moving(f.gauge_mode > 0, crate::wing::out(f));
        let weapon = combat::weapon_of(content, &params);
        world.me = i;
        wants[i] = fighter::update(f, &params, weapon, stage, &content.rules, *input, &world);
    }

    // Footstools made this frame (sim 135): the one stepped on is frozen on the ground, or pushed down tumbling in the air.
    for i in 0..nb {
        let j = core::mem::replace(&mut state.fighters[i].footstool_target, NONE);
        if j < 0 {
            continue;
        }
        let v = &mut state.fighters[j as usize];
        if v.active && !combat::is_intangible(v) {
            fighter::footstooled(v, &content.rules);
            v.hit_by = i as i8;
        }
    }

    // A move that has just begun draws its random version (E39), and balloons come out, use helium and go (E41).
    for i in 0..crate::state::bodies(state) {
        if state.fighters[i].active {
            let params = params_of(content, &state.fighters[i]);
            let weapon = combat::weapon_of(content, params);
            fighter::draw_version(&mut state.fighters[i], weapon, &mut state.rng);
        }
    }
    crate::balloons::update(state, content);

    // Bodies on the ground cannot pass through each other (see `push`).
    crate::push::separate(state, content, &before);
    // A grounded fighter on a slope keeps its feet on it after every sideways move this frame (sim 124; nothing on flat ground).
    for f in state.fighters.iter_mut().take(nb).filter(|f| f.active) {
        if crate::collision::platform(&content.stage, f.platform).is_some_and(|p| p.sloped()) {
            fighter::keep_on_ground(f, &content.stage);
        }
    }

    // Phase 1b: hits. Everything has moved, so hitboxes and hurtboxes are where they will be seen.
    combat::resolve_hits(state, content);
    // Grabs: keep each held fighter in its holder's hands and let it break free.
    grab::update(state, content);
    // Orb clocks (E59, sim 128): orbs whose wait has run out start to strike.
    crate::orbs::tick(state);
    // Existing projectiles move first, so a new one stays at the muzzle on the frame it appears.
    combat::update_projectiles(state, content);
    combat::spawn_projectiles(state, content);
    // Items last (E4): the fighters' orders (pick up, throw, pull, detonate), their flight, hits and bursts.
    crate::items::update(state, content);
    // Damage over time (E54, sim 121).
    combat::curse_tick(state, content);

    // Phase 2: ledge ownership.
    // Free ledges whose occupant left them (dropped, got up, jumped, or was hit).
    let nl = crate::state::ledges(state);
    for (l, owner) in state.ledge_owner.iter_mut().enumerate().take(nl) {
        if let Ok(o) = usize::try_from(*owner) {
            let f = &state.fighters[o];
            if f.state != FighterState::LedgeHang || usize::try_from(f.ledge) != Ok(l) {
                *owner = NONE;
            }
        }
    }
    // Contested grabs are decided by a fixed rule: the lowest player index wins, and a winner
    // trumps (knocks off) any current occupant. Losers simply keep falling.
    for l in 0..stage.ledges.len().min(nl) {
        let Some(winner) = wants.iter().position(|w| *w == Some(l as u8)) else {
            continue;
        };
        if let Ok(o) = usize::try_from(state.ledge_owner[l]) {
            let f = &state.fighters[o];
            let params = params_of(content, f).moving(f.gauge_mode > 0, crate::wing::out(f));
            fighter::trump(&mut state.fighters[o], &params, stage);
        }
        let f = &state.fighters[winner];
        let params = params_of(content, f).moving(f.gauge_mode > 0, crate::wing::out(f));
        fighter::grab_ledge(&mut state.fighters[winner], &params, stage, l);
        state.ledge_owner[l] = winner as i8;
    }

    // Phase 3: fighters who left the blast zone lose a stock and respawn.
    combat::check_ko(state, content);

    state.frame = state.frame.wrapping_add(1);
    settle(state, content);
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

    /// Sim 131: in versus the grunt slots stay exactly as a new match left them (nothing reads or writes them), which is why the checksum
    /// and every loop stop at the players outside a run (`state::bodies`).
    #[test]
    fn versus_never_touches_the_grunt_slots() {
        let content = Content::placeholder();
        for seed in 0..10u64 {
            let inputs = random_inputs(&mut Rng::new(seed), 1500);
            let mut s = GameState::new(&content, seed, [0, 1, 2, 3]);
            let empty = s.fighters[MAX_FIGHTERS..].to_vec();
            for i in &inputs {
                step(&mut s, &content, i);
            }
            assert_eq!(s.fighters[MAX_FIGHTERS..].to_vec(), empty, "seed {seed}");
            assert_eq!(crate::state::bodies(&s), MAX_FIGHTERS);
        }
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
                    if matches!(
                        f.state,
                        FighterState::Grabbing | FighterState::Grabbed | FighterState::Carrying
                    ) {
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
                    // A hand and its item always agree (E4).
                    if let Ok(h) = usize::try_from(f.holding) {
                        let it = &s.items[h];
                        assert!(
                            it.active && it.holder == n as i8,
                            "seed {seed} fighter {n}: holding a stray item"
                        );
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
                            | FighterState::ShieldRelease
                            | FighterState::Buried
                            | FighterState::Roll
                            | FighterState::SpotDodge
                            | FighterState::Grabbing
                            | FighterState::Grabbed
                            | FighterState::Knockdown
                            | FighterState::GetUp
                            | FighterState::LedgeGetUp
                            | FighterState::LedgeAttack
                            | FighterState::ItemPickup
                    );
                    // Attacks and hitstun can happen on the ground or in the air, and so can being held by a command grab (the
                    // holder in its catch move).
                    // (So can carrying a fighter, and being carried: E7.)
                    let command_held = f.state == FighterState::Grabbed
                        && usize::try_from(f.grab_with).is_ok_and(|h| {
                            matches!(
                                s.fighters[h].state,
                                FighterState::Attack | FighterState::Carrying
                            )
                        });
                    if !command_held
                        && !matches!(
                            f.state,
                            FighterState::Attack
                                | FighterState::Hitstun
                                | FighterState::ShieldBreak
                                | FighterState::Rebound
                                | FighterState::Carrying
                                | FighterState::Encased
                        )
                    {
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
