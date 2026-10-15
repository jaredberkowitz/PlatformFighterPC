//! The Nodachi's phase 2 (sim 128): the shadow orbs (side special, E59), the barrier (down special), the down smash without ground
//! contact, the eight slashes' landing hit and the forward air's wall stick. Reference data and sources:
//! `docs/movesets/nodachi/REFERENCE_DATA.md` ("Side special", "Down special", "Phase 1's leftovers").

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{JUMP, SPECIAL};
use sim_core::moves::nodachi_slots::{
    BARRIER_ANSWER, BARRIER_BREAK, DSMASH_OFF, OCTA, OCTA_LANDING, WALL_DROP, WALL_JUMP, WALL_STICK,
};
use sim_core::moves::{MoveId, Weapon};
use sim_core::state::{FighterState as S, ORB_CIRCLE, ORB_STRIKE};
use sim_core::{Fx, Vec2};

const EXILE: u8 = 13;

/// The exile (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([EXILE, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(EXILE)].weapon)]
}

fn k() -> Fx {
    sim_core::Content::placeholder().rules.damage_mult
}

fn close(a: Fx, b: Fx) -> bool {
    (a - b).abs() <= fx(1, 500)
}

fn orbs_on(sim: &Sim, d: usize) -> usize {
    sim.state
        .projectiles
        .iter()
        .filter(|p| {
            p.active && (p.orb == ORB_CIRCLE || p.orb == ORB_STRIKE) && usize::from(p.held) == d
        })
        .count()
}

// ---- The shadow orbs -----------------------------------------------------------------------------------------------------------

/// The side special at a duelist `gap` tenths ahead, special held for the first `held` ticks; the frames (reference numbering) on
/// which the duelist's percent rose and by how much, and the frame the exile could act again.
fn beam(sim: &mut Sim, held: u32, ticks: u32) -> (Vec<(u32, Fx)>, Option<u32>) {
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::SideSpecial);
    let mut rises = Vec::new();
    let mut last = sim.fighter(1).percent;
    let mut free = None;
    for t in 1..=ticks {
        sim.tick(inp(0, 0, if t < held { SPECIAL } else { 0 }));
        let p = sim.fighter(1).percent;
        if p != last {
            rises.push((t + 1, p - last));
            last = p;
        }
        if free.is_none() && sim.f().state != S::Attack {
            free = Some(t + 2);
        }
    }
    (rises, free)
}

fn beam_duel(gap: i32) -> Sim {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(gap, 10), -1);
    sim
}

#[test]
fn the_beam_hits_on_16_for_1_6_percent_and_frees_on_39_and_at_full_charge_on_42_for_4_5_and_65() {
    let mut sim = beam_duel(15);
    let (rises, free) = beam(&mut sim, 0, 60);
    assert_eq!(rises.first().map(|r| r.0), Some(16));
    assert!(close(rises[0].1, fx(29, 18) * k()), "{:?}", rises[0].1);
    assert_eq!(free, Some(39));
    // Held through the whole charge (26 frames): 42 and 65.
    let mut sim = beam_duel(15);
    let (rises, free) = beam(&mut sim, 60, 90);
    assert_eq!(rises.first().map(|r| r.0), Some(42));
    assert!(close(rises[0].1, fx(9, 2) * k()), "{:?}", rises[0].1);
    assert_eq!(free, Some(65));
    // It reaches farther charged: 7 frames at 5.46 a frame uncharged, at 8.16 charged (reference units).
    let mut sim = beam_duel(70);
    let (rises, _) = beam(&mut sim, 0, 60);
    assert!(
        rises.is_empty(),
        "uncharged it stops short of 7 world units"
    );
    let mut sim = beam_duel(70);
    let (rises, _) = beam(&mut sim, 60, 90);
    assert!(!rises.is_empty(), "charged it reaches 7");
}

#[test]
fn a_hit_puts_one_orb_on_two_from_17_frames_three_at_full_and_five_at_most() {
    for (held, want) in [(0u32, 1usize), (24, 1), (25, 2), (33, 2), (34, 3), (60, 3)] {
        // (`held` ticks of special, through tick `held - 1`: the charge counts the ticks held from the charge frame, the 8th, so
        // `held - 8` frames: 17 from 25, the whole 26 from 34.)
        let mut sim = beam_duel(15);
        beam(&mut sim, held, 70);
        assert_eq!(orbs_on(&sim, 1), want, "held {held}");
    }
    // Two full beams: 3 + 3, five at most.
    let mut sim = beam_duel(15);
    beam(&mut sim, 60, 70);
    beam(&mut sim, 60, 70);
    assert_eq!(orbs_on(&sim, 1), 5);
    // A reflected beam puts twice as many on (the rule; reflecting is the reflectors' business).
    let o = kit(&sim).get(MoveId::SideSpecial as u8).orbs.expect("orbs");
    assert_eq!(
        (o.placed(0, true), o.placed(17, true), o.placed(26, true)),
        (2, 4, 6)
    );
}

#[test]
fn the_orbs_strike_4_seconds_after_the_first_hit_for_6_5_each_and_are_gone() {
    let mut sim = beam_duel(15);
    let (rises, _) = beam(&mut sim, 25, 70);
    let first = rises[0].0;
    assert_eq!(orbs_on(&sim, 1), 2);
    // Keep the duelist where it is (the beam barely moves it); run to the strike.
    let mut strike = Vec::new();
    let mut last = sim.fighter(1).percent;
    for t in 71..=340u32 {
        sim.tick(inp(0, 0, 0));
        let p = sim.fighter(1).percent;
        if p != last {
            strike.push((t + 1, p - last));
            last = p;
        }
    }
    assert_eq!(orbs_on(&sim, 1), 0);
    assert_eq!(strike.len(), 2, "{strike:?}");
    for (_, d) in &strike {
        assert!(close(*d, fx(65, 10) * k()), "{d:?}");
    }
    // 240 frames after the hit they set off; each flies 13 frames and explodes on the 14th (it hits on touching the chest).
    let at = strike[0].0 - first;
    assert!((240..=254).contains(&at), "{at}");
}

#[test]
fn a_later_hit_lengthens_the_wait_but_never_past_7_5_seconds() {
    let mut sim = beam_duel(15);
    beam(&mut sim, 0, 40);
    let clock = sim.fighter(1).orb_clock;
    beam(&mut sim, 60, 70);
    // A full charge adds 3 seconds (less the frames that passed).
    let added = i32::from(sim.fighter(1).orb_clock) - i32::from(clock);
    assert!((180 - 70..=180).contains(&added), "{added}");
    for _ in 0..4 {
        beam(&mut sim, 60, 70);
    }
    let f = sim.fighter(1);
    assert!(u32::from(f.orb_clock) + u32::from(f.orb_age) <= 450);
}

#[test]
fn a_circling_orb_hits_a_bystander_twice_at_most_and_a_ko_takes_the_orbs_away() {
    let mut sim = Sim::with_chars([EXILE, 0, 0, 0]);
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b111;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    sim.stand(2, Fx::from_int(-8), 1);
    beam(&mut sim, 0, 40);
    assert_eq!(orbs_on(&sim, 1), 1);
    // A bystander standing in the orbit, beside the duelist: hit for 7%, again 45 frames later, then the orb is gone.
    let x = sim.fighter(1).pos.x + fx(15, 10);
    sim.stand(2, x, -1);
    let mut rises = Vec::new();
    let mut last = sim.fighter(2).percent;
    for t in 0..150u32 {
        sim.tick(inp(0, 0, 0));
        // (Held in place, standing.)
        sim.stand(2, x, -1);
        let p = sim.fighter(2).percent;
        if p != last {
            rises.push((t, p - last));
            last = p;
        }
    }
    assert_eq!(rises.len(), 2, "{rises:?}");
    assert!(rises.iter().all(|r| close(r.1, Fx::from_int(7) * k())));
    assert!(rises[1].0 - rises[0].0 >= 45);
    assert_eq!(orbs_on(&sim, 1), 0, "used up");
    // A KO of the fighter they circle takes them away.
    let mut sim = beam_duel(15);
    beam(&mut sim, 60, 70);
    assert_eq!(orbs_on(&sim, 1), 3);
    sim.state.fighters[1].pos.y = Fx::from_int(-100);
    sim.tick(inp(0, 0, 0));
    assert_eq!(orbs_on(&sim, 1), 0);
    assert_eq!(sim.fighter(1).orb_clock, 0);
}

#[test]
fn the_orbs_are_resimulated_exactly() {
    let mut sim = beam_duel(15);
    beam(&mut sim, 30, 50);
    let snapshot = sim.state;
    let mut straight = Vec::new();
    for _ in 0..300 {
        sim.tick(inp(0, 0, 0));
        straight.push(sim.state.checksum());
    }
    let mut again = Sim {
        content: sim.content.clone(),
        state: snapshot,
    };
    for (n, want) in straight.iter().enumerate() {
        again.tick(inp(0, 0, 0));
        assert_eq!(again.state.checksum(), *want, "frame {n}");
    }
}

// ---- The barrier ---------------------------------------------------------------------------------------------------------------

/// The down special, with the duelist (`gap` tenths ahead, facing it) jabbing from tick `jab_at` (0: never), its jab dealing `jab`
/// percent; the duelist's percent rises (reference frames of the exile's move) and the frame the exile could act again.
/// (The sim, the rises, the free frame and the moves the exile went through.)
type Barrier = (Sim, Vec<(u32, Fx)>, Option<u32>, Vec<u8>);

fn barrier(gap: i32, jab_at: u32, jab: i32) -> Barrier {
    let mut sim = duel();
    let w = usize::from(sim.content.fighters[0].weapon);
    for hb in &mut sim.content.weapons[w].moves[MoveId::Jab as usize].hitboxes {
        hb.damage = Fx::from_int(jab);
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(gap, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DownSpecial);
    let mut rises = Vec::new();
    let mut last = Fx::ZERO;
    let mut free = None;
    let mut moves = vec![MoveId::DownSpecial as u8];
    for t in 1..=120u32 {
        let b = if jab_at > 0 && t == jab_at {
            sim_core::input::buttons::ATTACK
        } else {
            0
        };
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
        let p = sim.fighter(1).percent;
        if p != last {
            rises.push((t + 1, p - last));
            last = p;
        }
        if sim.f().state == S::Attack && moves.last() != Some(&sim.f().move_id) {
            moves.push(sim.f().move_id);
        }
        if free.is_none() && sim.f().state != S::Attack {
            free = Some(t + 2);
        }
    }
    (sim, rises, free, moves)
}

#[test]
fn not_struck_the_barrier_answers_on_38_to_53_and_frees_on_87() {
    let (_, rises, free, moves) = barrier(18, 0, 0);
    assert_eq!(moves, vec![MoveId::DownSpecial as u8]);
    assert_eq!(free, Some(87));
    // The multi-hits every 5 frames from 38 (1%), the last on 53 (5%).
    let frames: Vec<u32> = rises.iter().map(|r| r.0).collect();
    assert_eq!(frames.first(), Some(&38), "{rises:?}");
    assert_eq!(frames.last(), Some(&53), "{rises:?}");
    assert!(close(rises.last().expect("hits").1, Fx::from_int(5) * k()));
    assert!(rises[..rises.len() - 1]
        .iter()
        .all(|r| close(r.1, Fx::from_int(1) * k())));
}

#[test]
fn struck_it_answers_at_once_with_damage_read_from_the_caught_hit() {
    // A 9% jab caught: multi-hits 2 + 2 x 9 / 18 = 3%, the last 6.5 + 11.5 x 9 / 18 = 12.25%.
    let (sim, rises, free, moves) = barrier(20, 8, 9);
    assert_eq!(
        moves[..2],
        [MoveId::DownSpecial as u8, BARRIER_ANSWER as u8]
    );
    assert!(free.is_some());
    let (last, multi) = rises.split_last().expect("the answer hits");
    assert!(close(last.1, fx(1225, 100) * k()), "{rises:?}");
    assert!(
        multi.iter().all(|r| close(r.1, Fx::from_int(3) * k())),
        "{rises:?}"
    );
    // The exile took nothing and was not frozen by the catch.
    assert_eq!(sim.f().percent, Fx::ZERO);
    // At 18% and above the answer is the maximum: 4% and 18%.
    let (_, rises, _, _) = barrier(20, 8, 20);
    let (last, multi) = rises.split_last().expect("the answer hits");
    assert!(close(last.1, Fx::from_int(18) * k()), "{rises:?}");
    assert!(multi.iter().all(|r| close(r.1, Fx::from_int(4) * k())));
}

#[test]
fn the_answer_runs_52_frames_and_is_intangible_through_11() {
    let sim = duel();
    let a = kit(&sim).get(BARRIER_ANSWER as u8);
    assert_eq!((a.total_frames, a.intangible), (52 - 2, 11));
    let p = a.projectile.expect("the answer is a projectile");
    assert_eq!((p.frame, p.life, p.rehit, p.pierce), (5, 17, 5, true));
    let m = kit(&sim).get(MoveId::DownSpecial as u8);
    let c = m.counter.expect("a counter");
    assert_eq!(
        (c.start, c.end, c.break_above, c.still),
        (6, 26, Fx::from_int(25), true)
    );
}

#[test]
fn a_hit_of_more_than_25_breaks_the_barrier_and_deals_nothing() {
    let (sim, rises, free, moves) = barrier(20, 8, 26);
    assert_eq!(moves[..2], [MoveId::DownSpecial as u8, BARRIER_BREAK as u8]);
    assert!(rises.is_empty());
    assert_eq!(sim.f().percent, Fx::ZERO);
    assert!(free.is_some());
    // Exactly 25 is still caught.
    let (_, _, _, moves) = barrier(20, 8, 25);
    assert_eq!(moves[1], BARRIER_ANSWER as u8);
}

// ---- Phase 1's leftovers -------------------------------------------------------------------------------------------------------

#[test]
fn the_down_smash_at_an_edge_meets_no_ground_and_only_its_blade_hits() {
    let run = |x: Fx| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(-10), 1);
        sim.stand(0, x, 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DSmash);
        for _ in 0..22 {
            sim.tick(inp(0, 0, 0));
        }
        sim.f().move_id
    };
    assert_eq!(run(Fx::ZERO), MoveId::DSmash as u8);
    assert_eq!(run(Fx::from_int(10)), DSMASH_OFF as u8);
    let sim = duel();
    let off = kit(&sim).get(DSMASH_OFF as u8);
    assert!(off.hitboxes.iter().all(|h| h.start == 20
        && h.end == 21
        && h.base_knockback == 75
        && h.knockback_growth == 63));
    assert_eq!(
        off.total_frames,
        kit(&sim).get(MoveId::DSmash as u8).total_frames
    );
}

#[test]
fn the_eight_slashes_land_into_their_landing_hit() {
    // Falling in the special fall after them, the exile lands into the 2.3% landing hit beside the duelist.
    let mut sim = duel();
    sim.stand(1, fx(12, 10), -1);
    sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].state = S::Helpless;
    sim.state.fighters[0].move_id = OCTA as u8;
    let mut landed = None;
    for t in 0..40 {
        sim.tick(inp(0, 0, 0));
        if landed.is_none() && sim.f().state == S::Attack {
            landed = Some(t);
        }
    }
    assert!(landed.is_some());
    assert!(close(sim.fighter(1).percent, fx(23, 10) * k()));
    let sim = duel();
    let octa = kit(&sim).get(OCTA as u8);
    assert_eq!(octa.landing_move, Some(OCTA_LANDING as u8));
    assert_eq!(octa.fall_landing_move, Some(OCTA_LANDING as u8));
}

#[test]
fn the_forward_air_sticks_in_a_wall_three_times_and_jumps_or_lets_go() {
    // Beside the stage's right wall (x 11, its top at 0), 1.5 below the top, facing it.
    let stick = |sim: &mut Sim| {
        sim.put_airborne(0, fx(125, 10), fx(-15, 10), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = -1;
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FAir);
        for _ in 0..14 {
            sim.tick(inp(0, 0, 0));
        }
        sim.f().move_id
    };
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    assert_eq!(stick(&mut sim), WALL_STICK as u8);
    // It hangs still; from frame 18 a jump press jumps off (upward).
    let at = sim.f().pos;
    for _ in 0..16 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().pos, at);
    assert_eq!(sim.f().move_id, WALL_STICK as u8);
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().move_id, WALL_JUMP as u8);
    for _ in 0..14 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().pos.y > at.y);
    // Twice more in the air time; the fourth stab does not stick.
    assert_eq!(stick(&mut sim), WALL_STICK as u8);
    for _ in 0..18 {
        sim.tick(inp(0, 0, 0));
    }
    // The stick away from the wall (right, the exile facing left) lets go.
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().move_id, WALL_DROP as u8);
    assert_eq!(stick(&mut sim), WALL_STICK as u8);
    assert_eq!(stick(&mut sim), MoveId::FAir as u8);
    // Out in the open (no wall), nothing sticks.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FAir);
    for _ in 0..14 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().move_id, MoveId::FAir as u8);
    let _ = Vec2::ZERO;
}
