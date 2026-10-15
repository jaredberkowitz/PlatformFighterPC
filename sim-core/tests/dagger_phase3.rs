//! The Dagger's phase 3 (sim 126, owner decisions 84-86): a shield or a reflector stopping the grounded hook ends the move sooner
//! (`Move::shield_end`); the gun's dodges stretched in time when stale, as rolls are (`Move::stale_stretch`); a press of special while
//! hanging from the tether reels in at once (`Tether::reel_press`, a house addition). Each is a general content field, off by default.
//! Reference data: `docs/movesets/dagger/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{SHIELD, SPECIAL};
use sim_core::moves::{dagger_slots, MoveId, STRETCH_BACK_ROLL, STRETCH_NONE, STRETCH_ROLL};
use sim_core::state::{FighterState as S, NONE};
use sim_core::{Fx, GameState, Vec2};

const PHANTOM: u8 = 12;

fn duel(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars([PHANTOM, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

fn weapon_index(sim: &Sim) -> usize {
    usize::from(sim.content.fighters[usize::from(PHANTOM)].weapon)
}

fn weapon(sim: &Sim) -> &sim_core::moves::Weapon {
    &sim.content.weapons[weapon_index(sim)]
}

// ---- Decision 84: a shield ends the grounded hook sooner ------------------------------------------------------------------------

/// The phantom's grounded hook with the duelist on the right platform at the hook's head on frame 20, shielding or not (`None`: no
/// one there). Returns the frame (the press is 1) the phantom leaves the move, and the sim.
fn hook_into(shielding: Option<bool>) -> (u16, Sim) {
    let mut sim = duel(Fx::from_int(10));
    sim.stand(0, fx(22, 10), 1);
    sim.tick(inp(0, 100, SPECIAL));
    for _ in 2..=19 {
        sim.tick(inp(0, 0, 0));
    }
    let b = match shielding {
        Some(s) => {
            let f = &mut sim.state.fighters[1];
            f.pos = Vec2::new(fx(35, 10), fx(18, 5));
            f.platform = 2;
            f.state = if s { S::Shield } else { S::Idle };
            f.state_frame = 20;
            if s {
                SHIELD
            } else {
                0
            }
        }
        None => 0,
    };
    sim.tick2(inp(0, 0, 0), inp(0, 0, b));
    let mut out = 0;
    for t in 21..=80u16 {
        if sim.f().state != S::Attack || sim.f().move_id != MoveId::UpSpecial as u8 {
            out = t - 1;
            break;
        }
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
    }
    (out, sim)
}

#[test]
fn a_shield_stopping_the_grounded_hook_ends_it_20_frames_later() {
    let sim = Sim::new();
    let w = weapon(&sim);
    assert_eq!(w.moves[MoveId::UpSpecial as usize].shield_end, 20);
    // The air hooks have no grab to stop.
    assert_eq!(w.moves[dagger_slots::HOOK_AIR as usize].shield_end, 0);
    // Whiffed: its own end (out on 59, FAF 60).
    let (whiff, _) = hook_into(None);
    assert_eq!(whiff, 59);
    // Stopped by a shield on frame 20: out 20 frames later, standing (FAF 41 [E]).
    let (stopped, sim) = hook_into(Some(true));
    assert_eq!(stopped, 40);
    assert_eq!(sim.f().state, S::Idle);
    assert_eq!(sim.fighter(1).state, S::Shield);
    // Not shielding: caught (and the pull goes on as before).
    let (_, sim) = hook_into(Some(false));
    assert_ne!(sim.f().move_id, MoveId::UpSpecial as u8);
}

#[test]
fn without_shield_end_a_stopped_hook_runs_to_its_end_and_the_early_end_is_hashed() {
    // Default off: the same stop with the field cleared runs to the move's end (sim 123's behaviour).
    let mut sim = duel(Fx::from_int(10));
    let wi = weapon_index(&sim);
    sim.content.weapons[wi].moves[MoveId::UpSpecial as usize].shield_end = 0;
    sim.stand(0, fx(22, 10), 1);
    sim.tick(inp(0, 100, SPECIAL));
    for _ in 2..=19 {
        sim.tick(inp(0, 0, 0));
    }
    let f = &mut sim.state.fighters[1];
    f.pos = Vec2::new(fx(35, 10), fx(18, 5));
    f.platform = 2;
    f.state = S::Shield;
    f.state_frame = 20;
    sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    assert_eq!(sim.f().move_end, 0);
    for _ in 21..=59 {
        assert_eq!(sim.f().state, S::Attack);
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    assert_ne!(sim.f().state, S::Attack);

    // With it: the early end is part of the checksum and a resimulation from the stop ends the same frame.
    let mut sim = duel(Fx::from_int(10));
    sim.stand(0, fx(22, 10), 1);
    sim.tick(inp(0, 100, SPECIAL));
    for _ in 2..=19 {
        sim.tick(inp(0, 0, 0));
    }
    let f = &mut sim.state.fighters[1];
    f.pos = Vec2::new(fx(35, 10), fx(18, 5));
    f.platform = 2;
    f.state = S::Shield;
    f.state_frame = 20;
    sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    assert_eq!(sim.f().move_end, 39, "move frame 19 (the press is 0) + 20");
    let snapshot: GameState = sim.state;
    let mut changed = snapshot;
    changed.fighters[0].move_end = 0;
    assert_ne!(changed.checksum(), snapshot.checksum());
    let mut inputs = [sim_core::Input::default(); sim_core::MAX_FIGHTERS];
    inputs[1] = inp(0, 0, SHIELD);
    let (mut a, mut b) = (snapshot, snapshot);
    for _ in 21..=40 {
        sim_core::step(&mut a, &sim.content, &inputs);
        sim_core::step(&mut b, &sim.content, &inputs);
        assert_eq!(a.checksum(), b.checksum());
    }
    assert_eq!(a.fighters[0].state, S::Idle);
    // A new move clears it.
    sim_core::fighter::begin_attack(&mut a.fighters[0], MoveId::Jab);
    assert_eq!(a.fighters[0].move_end, 0);
}

// ---- Decision 85: stale gun dodges last longer, as rolls do -----------------------------------------------------------------------

/// Runs one gun dodge (`slot`) at staleness `stale` from a standstill: the frame it ends (the first is 1), the distance it went, the
/// move frame shown on frame `probe`, and the last frame it was intangible.
fn dodge(slot: MoveId, stale: u8, probe: u16) -> (u16, Fx, u16, u16) {
    let mut sim = duel(Fx::from_int(30));
    sim.state.fighters[0].dodge_stale = stale;
    let x0 = sim.f().pos.x;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], slot);
    let (mut end, mut shown, mut last_intangible) = (0, 0, 0);
    for t in 1..=120u16 {
        sim.tick(inp(0, 0, 0));
        if t == probe {
            shown = sim.f().state_frame;
        }
        if sim.f().invuln > 0 {
            last_intangible = t;
        }
        if sim.f().state != S::Attack {
            end = t;
            break;
        }
    }
    (end, (sim.f().pos.x - x0).abs(), shown, last_intangible)
}

/// The roll's own stretch (`fighter::stale_length`): `frames` made `percent` x `level` percent slower, rounded up.
fn stale_length(frames: u16, percent: u8, level: u8) -> u16 {
    (frames * (100 + u16::from(percent) * u16::from(level))).div_ceil(100)
}

#[test]
fn stale_gun_dodges_are_stretched_in_time_as_rolls_are() {
    let sim = Sim::new();
    let rules = sim.content.rules;
    let w = weapon(&sim);
    assert_eq!(
        w.moves[dagger_slots::GUN_DASH as usize].stale_stretch,
        STRETCH_ROLL
    );
    assert_eq!(
        w.moves[dagger_slots::GUN_BACK as usize].stale_stretch,
        STRETCH_BACK_ROLL
    );
    // The jump is not a dodge (the reference's: neither staled nor staling).
    assert_eq!(
        w.moves[dagger_slots::GUN_JUMP as usize].stale_stretch,
        STRETCH_NONE
    );
    let max = rules.dodge_stale_max;

    // Fresh: the forward dodge ends on its last frame (38, FAF 40), the back one on 44 (FAF 46).
    let (fresh, fresh_dist, shown, fresh_last) = dodge(dagger_slots::GUN_DASH, 0, 26);
    assert_eq!(fresh, 38);
    assert_eq!(shown, 26);
    let (fresh_back, fresh_back_dist, _, _) = dodge(dagger_slots::GUN_BACK, 0, 1);
    assert_eq!(fresh_back, 44);

    // Fully stale: as long as a forward roll as stale would be (the ruleset's `dodge_stale_roll` per level), its frames shown
    // slower (the fresh timeline's), going further, intangible later but to the same frame.
    let (stale, stale_dist, shown, stale_last) = dodge(dagger_slots::GUN_DASH, max, 26);
    assert_eq!(stale, stale_length(38, rules.dodge_stale_roll, max));
    assert!(stale > fresh);
    let k = u16::from(rules.dodge_stale_roll) * u16::from(max);
    assert_eq!(shown, 26 * 100 / (100 + k));
    assert!(stale_dist > fresh_dist);
    assert_eq!(stale_last, fresh_last);
    // The back dodge at the back roll's rate.
    let (stale_back, stale_back_dist, _, _) = dodge(dagger_slots::GUN_BACK, max, 1);
    assert_eq!(
        stale_back,
        stale_length(44, rules.dodge_stale_back_roll, max)
    );
    assert!(stale_back_dist > fresh_back_dist);
    // One level: a little longer.
    let (one, _, _, _) = dodge(dagger_slots::GUN_DASH, 1, 1);
    assert_eq!(one, stale_length(38, rules.dodge_stale_roll, 1));
}

#[test]
fn a_stale_dodges_hold_is_hashed_resimulates_and_is_off_by_default() {
    // With the stretch cleared, a stale dodge keeps the fresh length (sim 123's behaviour).
    let mut sim = duel(Fx::from_int(30));
    let wi = weapon_index(&sim);
    sim.content.weapons[wi].moves[dagger_slots::GUN_DASH as usize].stale_stretch = STRETCH_NONE;
    sim.state.fighters[0].dodge_stale = sim.content.rules.dodge_stale_max;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], dagger_slots::GUN_DASH);
    let mut end = 0;
    for t in 1..=120u16 {
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().move_held, 0);
        if sim.f().state != S::Attack {
            end = t;
            break;
        }
    }
    assert_eq!(end, 38);

    // Stretched: the held frames are in the checksum and a resimulation agrees every frame.
    let mut sim = duel(Fx::from_int(30));
    sim.state.fighters[0].dodge_stale = sim.content.rules.dodge_stale_max;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], dagger_slots::GUN_DASH);
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().move_held > 0);
    let snapshot: GameState = sim.state;
    let mut changed = snapshot;
    changed.fighters[0].move_held += 1;
    assert_ne!(changed.checksum(), snapshot.checksum());
    let inputs = [sim_core::Input::default(); sim_core::MAX_FIGHTERS];
    let (mut a, mut b) = (snapshot, snapshot);
    for _ in 0..40 {
        sim_core::step(&mut a, &sim.content, &inputs);
        sim_core::step(&mut b, &sim.content, &inputs);
        assert_eq!(a.checksum(), b.checksum());
    }
    assert_ne!(a.fighters[0].state, S::Attack);
    // A dodge made once adds one level only, however long its first frame is held.
    let mut sim = duel(Fx::from_int(30));
    sim.state.fighters[0].dodge_stale = 2;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], dagger_slots::GUN_DASH);
    for _ in 0..5 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().dodge_stale, 3);
    assert_eq!(sim.f().dodge_level, 2);
}

// ---- Decision 86: a press of special while hanging reels in at once ---------------------------------------------------------------

fn beside_the_ledge() -> Sim {
    let mut sim = duel(Fx::from_int(-5));
    sim.put_airborne(0, Fx::from_int(12), Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = -1;
    sim
}

/// Hooks the right ledge, then presses special on hang frame `press` (0: never); returns the frame after the catch the phantom holds
/// the ledge.
fn reel(sim: &mut Sim, press: u16) -> u16 {
    sim.tick(inp(0, 100, SPECIAL));
    let mut caught = false;
    for _ in 2..=45u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Tether {
            caught = true;
            break;
        }
    }
    assert!(caught);
    for k in 1..=30u16 {
        let b = if k == press { SPECIAL } else { 0 };
        sim.tick(inp(0, 0, b));
        if sim.f().state == S::LedgeHang {
            return k;
        }
    }
    0
}

#[test]
fn a_press_of_special_while_hanging_reels_in_at_once() {
    let sim = Sim::new();
    let t = weapon(&sim).moves[dagger_slots::HOOK_AIR as usize]
        .tether
        .expect("a tether");
    assert!(t.reel_press);
    assert!(
        weapon(&sim).moves[dagger_slots::HOOK_AIR_SPENT as usize]
            .tether
            .expect("a tether")
            .reel_press
    );
    // Untouched: 8 frames hanging and 12 reeling, on the ledge on the 20th.
    assert_eq!(reel(&mut beside_the_ledge(), 0), 20);
    // Pressed on hang frame 2: the reel starts that frame, on the ledge 11 frames later.
    assert_eq!(reel(&mut beside_the_ledge(), 2), 13);
    // Pressed on the last hang frame: the same as waiting one frame less.
    assert_eq!(reel(&mut beside_the_ledge(), 8), 19);
    // A press while reeling changes nothing.
    assert_eq!(reel(&mut beside_the_ledge(), 12), 20);
    // Off by default: the press does nothing.
    let mut sim = beside_the_ledge();
    let wi = weapon_index(&sim);
    for slot in [dagger_slots::HOOK_AIR, dagger_slots::HOOK_AIR_SPENT] {
        if let Some(t) = sim.content.weapons[wi].moves[slot as usize].tether.as_mut() {
            t.reel_press = false;
        }
    }
    assert_eq!(reel(&mut sim, 2), 20);
    assert_eq!(sim.f().tether_ledge, NONE);
}
