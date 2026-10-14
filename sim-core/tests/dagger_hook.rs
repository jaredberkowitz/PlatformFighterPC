//! The Dagger's plain up special, the hook (phase 2, sim 123): the grounded grab that a shield or a reflector stops and that misses a
//! fighter in hitstun, its pull and throw; the aerial hit and its once-per-air-time rise; the tether (E56): the catch window and reach,
//! the hang, the reel to the ledge, letting go, three catches before landing, a held ledge, the checksum and a resimulation. Also the
//! phase-1 limits closed here: the up smash's arm intangible only with the partner, angle 365 on the gun's jump, the gun's dodges staled
//! like rolls and the buffered dodge 4 frames later. Reference data: the thirteenth moveset's `REFERENCE_DATA.md`
//! (`docs/movesets/dagger/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{JUMP, SHIELD, SPECIAL};
use sim_core::moves::{dagger_slots, MoveId, Tether, HIT_GRAB, MODE_POWERED, PART_LIMB};
use sim_core::state::{FighterState as S, NONE};
use sim_core::{FighterParams, Fx, GameState, Vec2};

const PHANTOM: u8 = 12;

/// The phantom (player 0, facing right at 0) and a duelist (player 1, `gap` to its right), a two-fighter match.
fn duel(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars([PHANTOM, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

fn weapon(sim: &Sim) -> &sim_core::moves::Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(PHANTOM)].weapon)]
}

fn tether(sim: &Sim) -> Tether {
    weapon(sim).moves[dagger_slots::HOOK_AIR as usize]
        .tether
        .expect("the air hook has a tether")
}

/// Where the duelist's feet go so the hook's head (10.6 ahead, 38.0 up of the reference's units) is at its chest.
fn at_the_head(sim: &Sim) -> Vec2 {
    let p = &sim.content.fighters[0];
    Vec2::new(
        FighterParams::su(10_600),
        FighterParams::su(38_000) - p.ecb_height * fx(1, 2),
    )
}

// ---- On the ground --------------------------------------------------------------------------------------------------------------

#[test]
fn the_grounded_hook_is_a_grab_on_20_to_26_with_faf_60_that_catches_in_the_air_and_throws_for_11() {
    let w = weapon(&Sim::new()).clone();
    let up = &w.moves[MoveId::UpSpecial as usize];
    assert_eq!(up.total_frames, 58, "FAF 60");
    assert!(up
        .hitboxes
        .iter()
        .all(|h| h.kind == HIT_GRAB && h.command && h.blockable));
    // The rope on 20 only, the head on 20-26 (move frames one less).
    assert_eq!(up.hitboxes.iter().map(|h| h.start).min(), Some(19));
    assert_eq!(up.hitboxes.iter().map(|h| h.end).max(), Some(25));
    assert_eq!(up.catch_move, Some(dagger_slots::HOOK_PULL as u8));
    assert_eq!(up.air_version, Some(dagger_slots::HOOK_AIR as u8));
    assert_eq!(up.limit_version, Some(dagger_slots::WINGS as u8));
    let pull = &w.moves[dagger_slots::HOOK_PULL as usize];
    assert_eq!(pull.total_frames, 28, "FAF 30");
    let t = pull.hitboxes[0];
    assert_eq!((t.start, t.damage, t.angle), (5, Fx::from_int(11), 78));
    assert_eq!((t.base_knockback, t.knockback_growth), (75, 40));

    // A fighter in the air at the head is caught on frame 20 and thrown on the pull's frame 6: 11% (x the one-on-one 1.2).
    let mut sim = duel(Fx::from_int(10));
    let at = at_the_head(&sim);
    sim.tick(inp(0, 100, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    let mut caught = 0;
    for t in 2..=30u16 {
        if sim.fighter(1).state != S::Grabbed {
            sim.put_airborne(1, at.x, at.y, Fx::ZERO, Fx::ZERO);
        }
        sim.tick(inp(0, 0, 0));
        if caught == 0 && sim.fighter(1).state == S::Grabbed {
            caught = t;
        }
    }
    assert_eq!(caught, 20);
    let k = sim.content.rules.damage_mult;
    // (Caught on 20, the pull begins on 21: its frame 6 is 26.)
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, 100, SPECIAL));
    let mut hit = 0;
    for t in 2..=60u16 {
        if sim.fighter(1).state != S::Grabbed && sim.fighter(1).percent == Fx::ZERO {
            sim.put_airborne(1, at.x, at.y, Fx::ZERO, Fx::ZERO);
        }
        sim.tick(inp(0, 0, 0));
        if hit == 0 && sim.fighter(1).percent > Fx::ZERO {
            hit = t;
        }
    }
    assert_eq!(hit, 26);
    assert_eq!(sim.fighter(1).percent, Fx::from_int(11) * k);
    assert_eq!(sim.fighter(1).state, S::Hitstun);
}

#[test]
fn a_shield_stops_the_grounded_hook_and_a_fighter_in_hitstun_is_not_caught() {
    // Standing (or shielding) on the right platform (3 to 8, 3.6 up) at the head: shielding, not caught, and the hook catches no one
    // for the rest of the move.
    for shielding in [true, false] {
        let mut sim = duel(Fx::from_int(10));
        sim.stand(0, fx(22, 10), 1);
        sim.tick(inp(0, 100, SPECIAL));
        for _ in 2..=19 {
            sim.tick(inp(0, 0, 0));
        }
        let f = &mut sim.state.fighters[1];
        f.pos = Vec2::new(fx(35, 10), fx(18, 5));
        f.platform = 2;
        f.state = if shielding { S::Shield } else { S::Idle };
        f.state_frame = 20;
        let b = if shielding { SHIELD } else { 0 };
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
        assert_eq!(
            sim.fighter(1).state == S::Grabbed,
            !shielding,
            "shielding {shielding}"
        );
        if shielding {
            // The move goes on to its end, catching no one (the shield dropped on the next frame does not help).
            for _ in 21..=26 {
                sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
                assert_ne!(sim.fighter(1).state, S::Grabbed);
            }
            assert_eq!(sim.f().state, S::Attack);
        }
    }
    // In hitstun: not caught.
    let mut sim = duel(Fx::from_int(10));
    let at = at_the_head(&sim);
    sim.tick(inp(0, 100, SPECIAL));
    for _ in 2..=26 {
        sim.put_airborne(1, at.x, at.y, Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].state = S::Hitstun;
        sim.state.fighters[1].hitstun = 30;
        sim.tick(inp(0, 0, 0));
        assert_ne!(sim.fighter(1).state, S::Grabbed);
    }
}

// ---- In the air ---------------------------------------------------------------------------------------------------------------

#[test]
fn the_aerial_hook_hits_for_5_on_20_ends_on_45_without_the_special_fall_and_rises_once_per_air_time(
) {
    let mut sim = duel(Fx::from_int(10));
    let at = at_the_head(&sim);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 100, SPECIAL));
    assert_eq!(sim.f().move_id, dagger_slots::HOOK_AIR as u8);
    let k = sim.content.rules.damage_mult;
    let mut hit = 0;
    let mut t = 1;
    let mut rose = false;
    while sim.f().state == S::Attack && t < 80 {
        let me = sim.f().pos;
        if sim.fighter(1).percent == Fx::ZERO {
            sim.put_airborne(1, me.x + at.x, me.y + at.y, Fx::ZERO, Fx::ZERO);
        }
        sim.tick(inp(0, 0, 0));
        t += 1;
        if t == 5 {
            rose = sim.f().vel.y > Fx::ZERO;
        }
        if hit == 0 && sim.fighter(1).percent > Fx::ZERO {
            hit = t;
        }
    }
    assert_eq!(hit, 20);
    assert_eq!(sim.fighter(1).percent, Fx::from_int(5) * k);
    assert!(rose, "the first hook of an air time rises");
    assert_eq!(sim.f().state, S::Airborne, "no special fall");
    // FAF 45 (without the hit's hitlag).
    let mut solo = duel(Fx::from_int(10));
    solo.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    solo.tick(inp(0, 100, SPECIAL));
    let mut t = 1;
    while solo.f().state == S::Attack && t < 80 {
        solo.tick(inp(0, 0, 0));
        t += 1;
    }
    assert_eq!(t + 1, 45, "FAF 45");
    // A second in the same air time is the hook without the rise.
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 100, SPECIAL));
    assert_eq!(sim.f().move_id, dagger_slots::HOOK_AIR_SPENT as u8);
    for _ in 2..=6 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().vel.y < Fx::ZERO);
}

// ---- The tether ---------------------------------------------------------------------------------------------------------------

/// The phantom in the air beside the right ledge (at 11, 0), facing it, `below` under it and `out` outside it, falling slowly.
fn beside_the_ledge(out: Fx, below: Fx) -> Sim {
    let mut sim = duel(Fx::from_int(-5));
    sim.put_airborne(0, Fx::from_int(11) + out, -below, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = -1;
    sim
}

/// Starts the up special and returns the frame (the press is 1) it caught the tether, if it did within the move.
fn hook_until_caught(sim: &mut Sim) -> Option<u16> {
    sim.tick(inp(0, 100, SPECIAL));
    for t in 2..=45u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Tether {
            return Some(t);
        }
    }
    None
}

#[test]
fn the_tether_catches_a_ledge_in_reach_from_frame_12_hangs_8_frames_and_reels_in_12() {
    let t = tether(&Sim::new());
    assert_eq!(
        (t.from, t.to, t.hang, t.reel, t.drag, t.limit),
        (11, 24, 8, 12, 240, 3)
    );
    let mut sim = beside_the_ledge(Fx::from_int(1), Fx::from_int(3));
    assert_eq!(hook_until_caught(&mut sim), Some(12));
    assert_eq!(sim.f().tether_ledge, 1);
    assert_eq!(sim.f().tether_count, 1);
    // No one holds the ledge meanwhile.
    assert_eq!(sim.state.ledge_owner[1], NONE);
    // 8 frames hanging, 12 reeled in: on the ledge on the 20th frame after the catch.
    let mut on = 0;
    for k in 1..=30u16 {
        sim.tick(inp(0, 0, 0));
        if on == 0 && sim.f().state == S::LedgeHang {
            on = k;
        }
        if k == 8 {
            assert_eq!(sim.f().state, S::Tether);
        }
    }
    assert_eq!(on, 20);
    assert_eq!(sim.state.ledge_owner[1], 0);
    assert_eq!(sim.f().tether_ledge, NONE);
    // Out of reach (too low, too far out, behind, or the stage's own side): no catch.
    for (out, below, facing) in [
        (Fx::from_int(1), Fx::from_int(8), -1),
        (Fx::from_int(3), Fx::from_int(3), -1),
        (Fx::from_int(1), Fx::from_int(3), 1),
    ] {
        let mut sim = beside_the_ledge(out, below);
        sim.state.fighters[0].facing = facing;
        // (Facing away, the up special's own turn would face it round: hold the stick away.)
        sim.tick(inp(if facing > 0 { 100 } else { -100 }, 100, SPECIAL));
        let mut caught = false;
        for _ in 2..=45 {
            sim.tick(inp(0, 0, 0));
            caught |= sim.f().state == S::Tether;
        }
        assert!(!caught, "{out:?} {below:?} {facing}");
    }
}

#[test]
fn the_stick_down_lets_go_and_three_catches_are_the_most_before_landing() {
    let mut sim = beside_the_ledge(Fx::from_int(1), Fx::from_int(3));
    assert_eq!(hook_until_caught(&mut sim), Some(12));
    sim.tick(inp(0, -100, 0));
    assert_eq!(sim.f().state, S::Airborne);
    assert_eq!(sim.f().tether_ledge, NONE);
    // Twice more (after the regrab wait), then the hook catches nothing.
    for n in 2..=4u8 {
        let p = sim.state.fighters[0];
        sim.state.fighters[0].ledge_cooldown = 0;
        sim.put_airborne(0, Fx::from_int(12), Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].tether_count = p.tether_count;
        let caught = hook_until_caught(&mut sim);
        if n <= 3 {
            assert_eq!(caught, Some(12), "catch {n}");
            sim.tick(inp(0, -100, 0));
        } else {
            assert_eq!(caught, None, "a fourth catch");
        }
    }
    // Landing resets the count.
    assert_eq!(sim.f().tether_count, 3);
    sim.put_airborne(0, Fx::ZERO, fx(1, 10), Fx::ZERO, Fx::ZERO);
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().grounded());
    assert_eq!(sim.f().tether_count, 0);
}

#[test]
fn a_held_ledge_is_not_caught_and_one_taken_while_reeling_throws_the_fighter_off() {
    // The duelist hangs from the right ledge: the hook does not catch it.
    let mut sim = beside_the_ledge(Fx::from_int(1), Fx::from_int(3));
    let duelist = sim.content.fighters[0];
    sim_core::fighter::grab_ledge(&mut sim.state.fighters[1], &duelist, &sim.content.stage, 1);
    sim.state.ledge_owner[1] = 1;
    assert_eq!(hook_until_caught(&mut sim), None);
    // Taken during the hang: when the reel ends the phantom is thrown off into the air.
    let mut sim = beside_the_ledge(Fx::from_int(1), Fx::from_int(3));
    assert_eq!(hook_until_caught(&mut sim), Some(12));
    sim.tick(inp(0, 0, 0));
    let stage = sim.content.stage.clone();
    sim_core::fighter::grab_ledge(&mut sim.state.fighters[1], &duelist, &stage, 1);
    sim.state.ledge_owner[1] = 1;
    for _ in 0..19 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Airborne);
    assert_eq!(sim.state.ledge_owner[1], 1);
    assert!(sim.f().vel.y > Fx::ZERO, "the knocked-off hop");
}

#[test]
fn a_hit_knocks_the_phantom_off_its_tether() {
    let mut sim = beside_the_ledge(Fx::from_int(1), Fx::from_int(3));
    assert_eq!(hook_until_caught(&mut sim), Some(12));
    // A hit puts it in hitstun like any other state; the tether count stays.
    let mut s2 = sim.state;
    s2.fighters[0].state = S::Hitstun;
    s2.fighters[0].hitstun = 20;
    sim.state = s2;
    sim.tick(inp(0, 0, 0));
    assert_ne!(sim.f().state, S::Tether);
    assert_eq!(sim.f().tether_count, 1);
}

#[test]
fn the_tether_state_is_in_the_checksum_and_resimulates_exactly() {
    let mut sim = beside_the_ledge(Fx::from_int(1), Fx::from_int(3));
    assert_eq!(hook_until_caught(&mut sim), Some(12));
    sim.tick(inp(0, 0, 0));
    let snapshot: GameState = sim.state;
    let mut changed = snapshot;
    changed.fighters[0].tether_from.x += Fx::ONE;
    assert_ne!(changed.checksum(), snapshot.checksum());
    let mut changed = snapshot;
    changed.fighters[0].tether_count += 1;
    assert_ne!(changed.checksum(), snapshot.checksum());
    let mut changed = snapshot;
    changed.fighters[0].tether_ledge = 0;
    assert_ne!(changed.checksum(), snapshot.checksum());
    // Run on from the snapshot twice: the same checksums every frame, through the reel and onto the ledge.
    let mut a = snapshot;
    let mut b = snapshot;
    let inputs = [sim_core::Input::default(); sim_core::MAX_FIGHTERS];
    for _ in 0..30 {
        sim_core::step(&mut a, &sim.content, &inputs);
        sim_core::step(&mut b, &sim.content, &inputs);
        assert_eq!(a.checksum(), b.checksum());
    }
    assert_eq!(a.fighters[0].state, S::LedgeHang);
}

// ---- Phase 1 limits closed ----------------------------------------------------------------------------------------------------

#[test]
fn the_up_smash_arm_is_intangible_on_10_to_14_only_with_the_partner() {
    let w = weapon(&Sim::new()).clone();
    let us = &w.moves[MoveId::USmash as usize];
    assert_eq!(us.part_intangible.len(), 1);
    let p = us.part_intangible[0];
    assert_eq!(
        (p.start, p.end, p.parts, p.mode),
        (9, 13, PART_LIMB, MODE_POWERED)
    );
    for powered in [false, true] {
        let mut sim = duel(Fx::from_int(10));
        sim.state.fighters[0].gauge_mode = if powered { 1800 } else { 0 };
        sim.state.fighters[0].gauge = if powered { Fx::from_int(100) } else { Fx::ZERO };
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::USmash);
        for _ in 0..10 {
            sim.tick(inp(0, 0, 0));
        }
        let bits = sim_core::combat::intangible_parts(&sim.state.fighters[0], &w);
        assert_eq!(bits == PART_LIMB, powered, "powered {powered}");
    }
}

#[test]
fn the_gun_jumps_first_shots_send_at_half_the_phantoms_speed() {
    let w = weapon(&Sim::new()).clone();
    let jump = &w.moves[dagger_slots::GUN_JUMP as usize];
    let angles: Vec<i16> = jump.hitboxes.iter().map(|h| h.angle).collect();
    assert!(angles.contains(&sim_core::moves::ANGLE_HALF_LINK));
    assert!(!angles.contains(&sim_core::moves::ANGLE_AUTOLINK));
    // A fighter in the air hit by a 365 box is launched at half the attacker's speed, whatever the knockback.
    let mut sim = duel(Fx::from_int(10));
    sim.put_airborne(1, Fx::from_int(2), Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    let f = &mut sim.state.fighters[1];
    f.link = true;
    f.link_kind = sim_core::state::LINK_HALF;
    f.link_vel = Vec2::new(Fx::from_int(2), Fx::from_int(-1));
    f.launch_pending = true;
    f.launch_kb = Fx::from_int(200);
    f.hitstun = 20;
    f.state = S::Hitstun;
    f.hitlag = 1;
    sim.tick(inp(0, 0, 0));
    let f = sim.fighter(1);
    assert_eq!(f.kb_vel, Vec2::new(Fx::ONE, -fx(1, 2)));
    assert!(!f.tumble);
}

#[test]
fn the_guns_dodges_stale_like_rolls_and_a_buffered_dodge_comes_4_frames_later() {
    let w = weapon(&Sim::new()).clone();
    assert!(w.moves[dagger_slots::GUN_DASH as usize].dodge_stale);
    assert!(w.moves[dagger_slots::GUN_BACK as usize].dodge_stale);
    // A fresh dodge forward is intangible from its frame 4; each one adds a level of staleness.
    let first_intangible = |sim: &mut Sim| -> u16 {
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], dagger_slots::GUN_DASH);
        for t in 1..=20u16 {
            sim.tick(inp(0, 0, 0));
            if sim.f().invuln > 0 {
                return t;
            }
        }
        0
    };
    let mut sim = duel(Fx::from_int(30));
    assert_eq!(first_intangible(&mut sim), 3);
    assert_eq!(sim.f().dodge_stale, 1);
    // Fully stale (five levels): intangible 4 frames later (the ruleset's roll shift).
    let mut sim = duel(Fx::from_int(30));
    sim.state.fighters[0].dodge_stale = sim.content.rules.dodge_stale_max;
    let shift = u16::from(sim.content.rules.dodge_stale_roll_shift);
    assert_eq!(first_intangible(&mut sim), 3 + shift);

    // The gun's dodge out of a shot: with the stick held sideways and special held since before frame 26 (buffered), the dodge starts
    // 4 frames later than with the stick pushed on 26 itself.
    let start = |buffered: bool| -> u16 {
        let mut sim = duel(Fx::from_int(30));
        sim.tick(inp(0, 0, SPECIAL));
        for t in 2..=60u16 {
            let side = if buffered || t >= 26 { 100 } else { 0 };
            sim.tick(inp(side, 0, SPECIAL));
            if sim.f().move_id == dagger_slots::GUN_DASH as u8 {
                return t;
            }
        }
        0
    };
    let fresh = start(false);
    assert!(fresh > 0);
    assert_eq!(start(true), fresh + 4);
    // (The grounded jump out of the gun the same way.)
    let jump_start = |buffered: bool| -> u16 {
        let mut sim = duel(Fx::from_int(30));
        sim.tick(inp(0, 0, SPECIAL));
        for t in 2..=60u16 {
            let b = if buffered || t >= 26 { JUMP } else { 0 };
            sim.tick(inp(0, 0, b));
            if sim.f().move_id == dagger_slots::GUN_JUMP as u8 {
                return t;
            }
        }
        0
    };
    assert_eq!(jump_start(true), jump_start(false) + 4);
}
