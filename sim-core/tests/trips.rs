//! Trips (E20, sim 86) and hits that clank without rebounding (E3g, sim 86), the reference's `trip` and `rebound=f` hitbox fields.
//!
//! A hit with a trip chance (`Hitbox::trip`, a percent) that lands on a fighter on the ground, with a launch that would not lift it
//! off the ground, rolls the game's own random generator (`GameState::rng`, hashed): on a success the fighter takes the damage and
//! hitlag but sits down (`Knockdown` with `Fighter::tripped`) instead of being launched, intangible through the hitlag and for
//! `Ruleset::trip_invuln` frames after, able to get up from frame `trip_lag` and standing up by itself on `trip_max` (the reference's
//! hard trip, 40 and 100). Nothing is rolled for a hit that cannot trip. A box with `Hitbox::rebounds` false clanks (its hit is
//! stopped) but its owner carries on with the move instead of rebounding.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

const HAULER: u8 = 7;

/// A hauler (player 0) about to down tilt a standing duelist (player 1) at 0%, the down tilt's trip chance set to `trip`; the others
/// out. The down tilt hits on its frame 6 (the tilt starts on the first tick).
fn down_tilt(trip: u8) -> Sim {
    let mut sim = Sim::with_chars([HAULER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    let w = usize::from(sim.content.fighters[usize::from(HAULER)].weapon);
    for h in &mut sim.content.weapons[w].moves[MoveId::DTilt as usize].hitboxes {
        h.trip = trip;
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(17, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DTilt);
    sim
}

/// Runs until player 1 has been hit (its percent rises); returns the tick.
fn until_hit(sim: &mut Sim, p1: sim_core::Input) -> u32 {
    for t in 1..=20 {
        sim.tick2(inp(0, 0, 0), p1);
        if sim.fighter(1).percent > Fx::ZERO {
            return t;
        }
    }
    panic!("the down tilt never hit");
}

#[test]
fn a_sure_trip_sits_the_target_down_instead_of_launching_it() {
    let mut sim = down_tilt(100);
    until_hit(&mut sim, inp(0, 0, 0));
    let f = *sim.fighter(1);
    assert_eq!((f.state, f.tripped), (S::Knockdown, true));
    assert!(f.grounded() && f.hitlag > 0 && !f.launch_pending);
    assert_eq!(f.percent, fx(6, 1) * sim.content.rules.damage_mult);
    // The same hit with no trip chance launches as usual (at 0% the 361 angle sends it along the ground in hitstun).
    let mut plain = down_tilt(0);
    until_hit(&mut plain, inp(0, 0, 0));
    assert_eq!(plain.fighter(1).state, S::Hitstun);
    assert!(!plain.fighter(1).tripped);
}

#[test]
fn a_slip_trips_whatever_the_launch_and_a_soft_one_has_no_hitlag() {
    // E44 (sim 117): a hit with `slip` trips a fighter on the ground every time, even one that would launch it upward (here the down
    // tilt turned to 90 degrees and strong), with no roll; a hard slip keeps the hitlag and the trip's timings, a soft one has no
    // hitlag and the ruleset's slip timings. An airborne fighter is launched as usual.
    for (slip, soft) in [
        (sim_core::moves::SLIP_HARD, false),
        (sim_core::moves::SLIP_SOFT, true),
    ] {
        let mut sim = down_tilt(0);
        let w = usize::from(sim.content.fighters[usize::from(HAULER)].weapon);
        for h in &mut sim.content.weapons[w].moves[MoveId::DTilt as usize].hitboxes {
            h.slip = slip;
            h.angle = 90;
            h.base_knockback = 100;
        }
        let rng = sim.state.rng;
        until_hit(&mut sim, inp(0, 0, 0));
        let f = *sim.fighter(1);
        assert_eq!(
            (f.state, f.tripped, f.trip_soft),
            (S::Knockdown, true, soft)
        );
        assert_eq!(f.hitlag > 0, !soft, "hitlag only on a hard slip");
        assert_eq!(sim.state.rng, rng, "nothing rolled");
    }
    let mut sim = down_tilt(0);
    let w = usize::from(sim.content.fighters[usize::from(HAULER)].weapon);
    for h in &mut sim.content.weapons[w].moves[MoveId::DTilt as usize].hitboxes {
        h.slip = sim_core::moves::SLIP_HARD;
    }
    sim.put_airborne(1, fx(17, 10), fx(2, 10), Fx::ZERO, Fx::ZERO);
    until_hit(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Hitstun, "in the air: launched");
    assert!(!sim.fighter(1).tripped);
}

/// After the trip's hitlag, (frame of the sit, state, intangible) for each tick while player 1 holds `p1`.
fn after_trip(p1: sim_core::Input, ticks: u32) -> Vec<(u16, S, bool)> {
    let mut sim = down_tilt(100);
    until_hit(&mut sim, inp(0, 0, 0));
    while sim.fighter(1).hitlag > 0 {
        assert!(
            is_intangible(sim.fighter(1)),
            "intangible through the hitlag"
        );
        sim.tick2(inp(0, 0, 0), p1);
    }
    let mut out = Vec::new();
    for _ in 0..ticks {
        sim.tick2(inp(0, 0, 0), p1);
        let f = sim.fighter(1);
        out.push((f.state_frame, f.state, is_intangible(f)));
    }
    out
}

#[test]
fn a_tripped_fighter_is_intangible_for_5_frames_and_gets_up_from_frame_40_or_by_itself_on_100() {
    // (The hitlag frames do not count: the sit's frame 1 is the first after them. The tick that ended the hitlag is frame 0's.)
    let idle = after_trip(inp(0, 0, 0), 110);
    let sat: Vec<_> = idle.iter().take_while(|r| r.1 == S::Knockdown).collect();
    assert!(
        sat[..5].iter().all(|r| r.2),
        "intangible on 1-5: {:?}",
        &sat[..6]
    );
    assert!(!sat[5].2, "tangible from frame 6");
    // Left alone it stands up on frame 100.
    assert_eq!(sat.last().map(|r| r.0), Some(99));
    assert_eq!(idle[sat.len()].1, S::GetUp);
    // Holding up (a get-up) from the start: nothing before frame 40, then it stands.
    let up = after_trip(inp(0, 127, 0), 50);
    let sat: Vec<_> = up.iter().take_while(|r| r.1 == S::Knockdown).collect();
    assert_eq!(sat.last().map(|r| r.0), Some(39));
    assert_eq!(up[sat.len()].1, S::GetUp);
    // A missed tech still has its own knockdown timings: the trip flag is gone once it stands.
    assert!(!up.iter().any(|r| r.1 == S::GetUp && r.0 > 30));
}

#[test]
fn the_chance_is_rolled_on_the_games_generator_once_per_hit_that_can_trip() {
    // A 40% trip (47% with the base chance) over many seeds: some trip, some do not, and each seed always does the same.
    let mut tripped = 0;
    for seed in 0..60u64 {
        let outcome = |seed: u64| {
            let mut sim = down_tilt(40);
            sim.state.rng = sim_core::Rng::new(seed);
            until_hit(&mut sim, inp(0, 0, 0));
            (sim.fighter(1).tripped, sim.state.rng, sim.state.checksum())
        };
        let (a, b) = (outcome(seed), outcome(seed));
        assert_eq!(a, b, "seed {seed}");
        let mut unrolled = sim_core::Rng::new(seed);
        unrolled.next_u64();
        assert_eq!(a.1, unrolled, "one roll, seed {seed}");
        tripped += u32::from(a.0);
    }
    assert!((16..=42).contains(&tripped), "{tripped} of 60 tripped");
    // A hit with no trip chance never touches the generator.
    let mut sim = down_tilt(0);
    let before = sim.state.rng;
    until_hit(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.state.rng, before);
}

#[test]
fn a_hit_that_lifts_its_target_never_trips() {
    // The hauler's down tilt at 100% knockback-heavy percent goes over the 361 angle's ground threshold and lifts: no roll, no trip.
    let mut sim = down_tilt(100);
    sim.state.fighters[1].percent = Fx::from_int(150);
    let before = sim.state.rng;
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::from_int(150) {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Hitstun);
    assert_eq!(sim.state.rng, before);
    // The swordfighter's down tilt (35%, 30 degrees: the reference's trip "only on slants") lifts too: never a trip here.
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    for h in &mut sim.content.weapons[0].moves[MoveId::DTilt as usize].hitboxes {
        assert_eq!(h.trip, 35);
        h.trip = 100;
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(25, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DTilt);
    until_hit(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Hitstun);
}

#[test]
fn the_older_kits_carry_their_reference_trip_chances() {
    // ssbwiki.com's raw tables (13.0.1; owner decisions 39 and 51): (weapon, move, the boxes' trip chances in order).
    let c = sim_core::Content::placeholder();
    let listed: [(usize, MoveId, &[u8]); 9] = [
        // The Gauntlets' forward air: the late hit's two boxes.
        (4, MoveId::FAir, &[0, 0, 20, 20]),
        // The Longsword's down tilt (lifts at 30 degrees: never trips on the flat stages).
        (0, MoveId::DTilt, &[35, 35]),
        // The Claws' down tilt (361: it can trip) and down smash (30-35 degrees: never on the flat).
        (1, MoveId::DTilt, &[40, 40, 40]),
        (1, MoveId::DSmash, &[35, 35, 35, 35]),
        // The Needles' down tilt (70 / 90) and the down smash's second hit (43): never on the flat.
        (2, MoveId::DTilt, &[40, 40, 40]),
        (2, MoveId::DSmash, &[0, 0, 0, 0, 30, 30]),
        // The Embers' down tilt (80), the Gauntlets' (25 / 155), the Greatsword's (80): never on the flat.
        (3, MoveId::DTilt, &[40, 40]),
        (4, MoveId::DTilt, &[50, 50, 50]),
        (6, MoveId::DTilt, &[20, 20, 20, 20, 20]),
    ];
    for (w, weapon) in c.weapons.iter().enumerate().take(7) {
        for (m, mv) in weapon.moves.iter().enumerate() {
            let trips: Vec<u8> = mv.hitboxes.iter().map(|h| h.trip).collect();
            match listed
                .iter()
                .find(|&&(lw, id, _)| lw == w && id as usize == m)
            {
                Some((_, _, want)) => assert_eq!(trips, want.to_vec(), "weapon {w} move {m}"),
                None => assert!(trips.iter().all(|&t| t == 0), "weapon {w} move {m}"),
            }
        }
    }
}

#[test]
fn the_claws_down_tilt_trips_a_standing_target() {
    // The Claws' down tilt (361 at low knockback, along the ground): at a sure chance it always trips.
    let mut sim = Sim::with_chars([1, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    for h in &mut sim.content.weapons[1].moves[MoveId::DTilt as usize].hitboxes {
        h.trip = 100;
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DTilt);
    until_hit(&mut sim, inp(0, 0, 0));
    assert!(sim.fighter(1).tripped);
}

// ---- The base chance (owner decision 50) ----------------------------------------------------------------------------------------

#[test]
fn the_base_chance_is_7_percent_over_55_knockback_and_adds_to_a_moves_own_chance() {
    use sim_core::combat::trip_chance;
    let c = sim_core::Content::placeholder();
    let rules = &c.rules;
    assert_eq!((rules.trip_base, rules.trip_base_kb), (7, Fx::from_int(55)));
    let mut hb = c.weapons[7].moves[MoveId::DTilt as usize].hitboxes[0];
    // A move's own 40% trips 47% of the time, whatever the knockback (ssbwiki.com, Tripping).
    assert_eq!(trip_chance(&hb, Fx::from_int(20), rules), 47);
    assert_eq!(trip_chance(&hb, Fx::from_int(70), rules), 47);
    // No chance of its own: 7% only over 55 knockback.
    hb.trip = 0;
    assert_eq!(trip_chance(&hb, Fx::from_int(55), rules), 0);
    assert_eq!(
        trip_chance(&hb, Fx::from_int(55) + Fx::from_raw(1), rules),
        7
    );
    // Capped at 100; a throw never trips.
    hb.trip = 99;
    assert_eq!(trip_chance(&hb, Fx::ZERO, rules), 100);
    hb.kind = sim_core::moves::HIT_THROW;
    assert_eq!(trip_chance(&hb, Fx::from_int(70), rules), 0);
}

/// The hauler's down tilt with no trip chance of its own, made to send straight sideways (angle 0) at 57 knockback (base 57, no
/// growth), into a standing duelist, rolling the generator from `seed`; `base` is the ruleset's base chance. Returns (tripped, the
/// generator after the hit).
fn sideways(seed: u64, base: u8) -> (bool, sim_core::Rng) {
    let mut sim = down_tilt(0);
    sim.content.rules.trip_base = base;
    let w = usize::from(sim.content.fighters[usize::from(HAULER)].weapon);
    for h in &mut sim.content.weapons[w].moves[MoveId::DTilt as usize].hitboxes {
        h.angle = 0;
        h.base_knockback = 57;
        h.knockback_growth = 0;
    }
    sim.state.rng = sim_core::Rng::new(seed);
    until_hit(&mut sim, inp(0, 0, 0));
    (sim.fighter(1).tripped, sim.state.rng)
}

#[test]
fn any_low_hit_over_55_knockback_trips_7_percent_of_the_time() {
    let mut tripped = 0;
    for seed in 0..300u64 {
        let (t, rng) = sideways(seed, 7);
        let mut once = sim_core::Rng::new(seed);
        once.next_u64();
        assert_eq!(rng, once, "one roll, seed {seed}");
        tripped += u32::from(t);
        // With the base chance off, the same hit never trips and rolls nothing.
        let (t0, rng0) = sideways(seed, 0);
        assert!(!t0);
        assert_eq!(rng0, sim_core::Rng::new(seed));
    }
    // 7% of 300 is 21.
    assert!((8..=36).contains(&tripped), "{tripped} of 300 tripped");
}

// ---- No rebound ------------------------------------------------------------------------------------------------------------------

/// Two haulers 2.0 apart, facing each other, each one frame before its forward smash's hit (reference frame 21); with `rebounds` the
/// smashes are made to rebound again (as before sim 86). Returns (anyone rebounded, anyone hit, both still in the smash after the
/// hit frames).
fn smashes_meet(rebounds: bool) -> (bool, bool, bool) {
    let mut sim = Sim::with_chars([HAULER, HAULER, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    if rebounds {
        let w = usize::from(sim.content.fighters[usize::from(HAULER)].weapon);
        for h in &mut sim.content.weapons[w].moves[MoveId::FSmash as usize].hitboxes {
            h.rebounds = true;
        }
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(20, 10), -1);
    for i in 0..2 {
        sim_core::fighter::begin_attack(&mut sim.state.fighters[i], MoveId::FSmash);
        sim.state.fighters[i].state_frame = 19;
    }
    let mut rebound = false;
    for _ in 0..4 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        rebound |= sim.f().state == S::Rebound || sim.fighter(1).state == S::Rebound;
    }
    (
        rebound,
        sim.f().percent > Fx::ZERO || sim.fighter(1).percent > Fx::ZERO,
        sim.f().state == S::Attack && sim.fighter(1).state == S::Attack,
    )
}

#[test]
fn smashes_that_cannot_rebound_clank_and_carry_on() {
    // The reference's `rebound=f`: the fists meet, neither hits, and both smashes go on to their end.
    assert_eq!(smashes_meet(false), (false, false, true));
    // Made to rebound, the same clank knocks both out of the smash: the setup does make the boxes meet.
    let (rebound, hit, _) = smashes_meet(true);
    assert!(rebound && !hit);
}

#[test]
fn the_older_kits_clank_without_rebounding_where_the_reference_does() {
    // ssbwiki.com's raw tables (13.0.1): `rebound=f` without `clang=f` on every box of the Claws' (weapon 1) up smash and the Staff's
    // (weapon 5) forward, up and down tilts and neutral air. No other older normal has it.
    let c = sim_core::Content::placeholder();
    let listed = [
        (1usize, MoveId::USmash),
        (5, MoveId::FTilt),
        (5, MoveId::UTilt),
        (5, MoveId::DTilt),
        (5, MoveId::NAir),
    ];
    for (w, weapon) in c.weapons.iter().enumerate().take(7) {
        for (m, mv) in weapon.moves.iter().enumerate() {
            let no_rebound = listed.iter().any(|&(lw, id)| lw == w && id as usize == m);
            assert!(
                mv.hitboxes.iter().all(|h| h.rebounds != no_rebound),
                "weapon {w} move {m}"
            );
        }
    }
}
