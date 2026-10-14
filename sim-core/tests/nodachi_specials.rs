//! The Nodachi's specials (phase 1): the blade dash and its eight slashes (up special), the flare in three levels (neutral special).
//! Reference data and sources: `docs/movesets/nodachi/REFERENCE_DATA.md` ("Up special", "Neutral special").

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{SHIELD, SPECIAL};
use sim_core::moves::nodachi_slots::{
    DASH_AIR_END, FLARE, FLARE_CANCEL, GIGAFLARE, MEGAFLARE, OCTA,
};
use sim_core::moves::{rings_at, MoveId, Weapon, ANGLE_AUTOLINK, ANGLE_VORTEX};
use sim_core::state::FighterState as S;
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

// ---- The blade dash ------------------------------------------------------------------------------------------------------------

/// Starts the up special on the ground at `x` facing right, the stick held at `stick` (x, y) and special held for `held` ticks.
fn up_special(
    sim: &mut Sim,
    x: Fx,
    stick: (i8, i8),
    held: u32,
    ticks: u32,
) -> Vec<(u32, S, u8, Fx)> {
    sim.stand(0, x, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UpSpecial);
    let mut run = Vec::new();
    for t in 1..=ticks {
        let b = if t <= held { SPECIAL } else { 0 };
        sim.tick(inp(stick.0, stick.1, b));
        run.push((t, sim.f().state, sim.f().move_id, sim.f().pos.x));
    }
    run
}

#[test]
fn a_tapped_up_special_is_the_blade_dash_and_held_through_18_the_eight_slashes() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(9), -1);
    let run = up_special(&mut sim, Fx::ZERO, (127, 0), 3, 30);
    assert!(run.iter().all(|r| r.2 == MoveId::UpSpecial as u8));
    let mut sim = duel();
    sim.stand(1, Fx::from_int(9), -1);
    let run = up_special(&mut sim, Fx::ZERO, (127, 0), 40, 30);
    // (The switch on the reference's frame 18.)
    let at = run.iter().find(|r| r.2 == OCTA as u8).map(|r| r.0 + 1);
    assert_eq!(at, Some(18));
}

#[test]
fn the_blade_dash_along_the_ground_hits_on_25_travels_and_ends_on_57() {
    // A duelist just ahead: the dash's one hit (7%) on frame 25.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(3), -1);
    let run = up_special(&mut sim, Fx::ZERO, (127, 0), 2, 60);
    let mut s = duel();
    s.stand(1, Fx::from_int(3), -1);
    s.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut s.state.fighters[0], MoveId::UpSpecial);
    let hit = (1..=40u32)
        .find(|_| {
            s.tick(inp(127, 0, 0));
            s.fighter(1).percent > Fx::ZERO
        })
        .map(|t| t + 1);
    assert_eq!(hit, Some(25));
    assert!(run.iter().any(|r| r.1 == S::Attack));
    assert_eq!(sim.fighter(1).percent, Fx::from_int(7) * k());
    // Alone: it stays on the ground and gives control back on 57; it covers the 4 dashing frames at 11 of the reference's units a
    // frame [E] and a little of the slowed speed after.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    let run = up_special(&mut sim, Fx::from_int(-9), (127, 0), 2, 60);
    let end = run.iter().find(|r| r.1 != S::Attack).map(|r| r.0 + 2);
    assert_eq!(end, Some(57));
    assert!(sim.f().grounded());
    let travelled = sim.f().pos.x - Fx::from_int(-9);
    assert!(
        travelled >= fx(55, 10) && travelled <= fx(85, 10),
        "{travelled:?}"
    );
}

#[test]
fn the_grounded_dash_stops_at_the_edge() {
    // From 2 units short of the stage's right edge (11), dashing right: it ends on the edge, still standing.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    let run = up_special(&mut sim, Fx::from_int(9), (127, 0), 2, 60);
    let end = run
        .iter()
        .find(|r| r.1 != S::Attack)
        .expect("the move ends");
    assert_eq!(
        (end.1, end.3),
        (S::Idle, Fx::from_int(11)),
        "standing on the edge"
    );
}

#[test]
fn aimed_up_the_dash_rises_then_the_special_fall_lands_with_27_frames_of_lag() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    let run = up_special(&mut sim, Fx::ZERO, (0, 0), 2, 200);
    // (Neutral: straight up.) It rose; it went on in the air past frame 57 (to 80), then fell helpless.
    assert!(run.iter().any(|r| r.2 == DASH_AIR_END as u8));
    let helpless = run.iter().find(|r| r.1 == S::Helpless).map(|r| r.0);
    // (Ticks count from 0: the special fall begins on the reference's frame 80, so control of the drift is back on 81.)
    assert_eq!(helpless, Some(79), "the special fall from frame 81");
    let landed = run
        .windows(2)
        .find(|w| w[0].1 == S::Helpless && w[1].1 == S::Landing)
        .map(|w| w[1].0);
    assert!(landed.is_some(), "it lands");
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    up_special(&mut sim, Fx::ZERO, (0, 0), 2, 200);
    let w = kit(&sim);
    assert_eq!(w.get(MoveId::UpSpecial as u8).fall_landing_lag, 27);
    assert_eq!(w.get(OCTA as u8).fall_landing_lag, 56);
}

#[test]
fn the_eight_slashes_hit_eight_times_on_the_references_frames_for_23_1_percent() {
    // A duelist held at the exile's side (in the air, so the aerial boxes are the ones used) through the move: hits on the
    // reference's frames 36, 38, ..., 48 and 50 (the move's own frames: hitlag holds them).
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UpSpecial);
    let mut frames = Vec::new();
    let mut last = Fx::ZERO;
    for t in 1..=160u32 {
        let me = sim.f().pos;
        sim.put_airborne(1, me.x + fx(15, 10), me.y + fx(3, 10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(127, 0, if t < 30 { SPECIAL } else { 0 }));
        if sim.fighter(1).percent != last {
            last = sim.fighter(1).percent;
            assert_eq!(sim.f().move_id, OCTA as u8);
            // (This move began on the reference's frame 18.)
            frames.push(sim.f().state_frame + 18);
        }
        if sim.f().state != S::Attack {
            break;
        }
    }
    assert_eq!(frames, vec![36, 38, 40, 42, 44, 46, 48, 50]);
    let want = (fx(23, 10) * Fx::from_int(7) + Fx::from_int(7)) * k();
    assert!((last - want).abs() <= fx(1, 1000), "{last:?} vs {want:?}");
    // The drag is the autolink angle; the last hit 361, BK 68, KS 104, hitlag x2.4.
    let w = kit(&sim);
    let octa = w.get(OCTA as u8);
    assert!(octa
        .hitboxes
        .iter()
        .any(|h| h.angle == ANGLE_AUTOLINK && h.group == 1));
    let last_hit = octa
        .hitboxes
        .iter()
        .find(|h| h.group == 2 && h.priority == 0)
        .expect("hit 8");
    assert_eq!(
        (
            last_hit.damage,
            last_hit.angle,
            last_hit.base_knockback,
            last_hit.knockback_growth,
            last_hit.hitlag
        ),
        (Fx::from_int(7), 361, 68, 104, 240)
    );
    assert_eq!(octa.total_frames + 18 + 1, 105, "FAF 105 on the ground");
}

// ---- The flare ---------------------------------------------------------------------------------------------------------------

/// Starts the neutral special standing at 0 facing right, special held for `held` ticks (0: a tap), a duelist standing `gap` ahead
/// (none if `gap` is 0). Returns the ticks on which the duelist's percent rose, with the rise, and the tick the exile could act.
fn flare(held: u32, gap: i32, ticks: u32) -> (Vec<(u32, Fx)>, Option<u32>, Vec<u8>) {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    if gap == 0 {
        // (Out of the way, not out of the match: a match with one fighter left is over.)
        sim.stand(1, Fx::from_int(-10), 1);
    } else {
        sim.stand(1, Fx::from_ratio(gap, 10), -1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NSpecial);
    let mut rises = Vec::new();
    let mut last = Fx::ZERO;
    let mut free = None;
    let mut moves = Vec::new();
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
        if sim.f().state == S::Attack && moves.last() != Some(&sim.f().move_id) {
            moves.push(sim.f().move_id);
        }
    }
    (rises, free, moves)
}

#[test]
fn a_tap_fires_the_flare_which_hits_on_41_and_bursts_3_frames_later() {
    let (rises, free, moves) = flare(0, 18, 120);
    assert_eq!(moves, vec![MoveId::NSpecial as u8, FLARE as u8]);
    assert_eq!(
        free,
        Some(84),
        "FAF 84 from the start (65 from the release on 20)"
    );
    assert_eq!(rises.first(), Some(&(41, Fx::from_int(3) * k())));
    assert_eq!(
        rises.get(1).map(|r| (r.0, r.1)),
        Some((44, fx(105, 10) * k()))
    );
    // Its radius grows 3.0 / 3.5 / 4.0 / 4.5 by age (from the release's 28, 35 and 40).
    let sim = duel();
    let spec = kit(&sim).get(FLARE as u8).projectile.expect("a fireball");
    let r = |age: u8| rings_at(&spec.phases, age)[0].map_or(spec.hitbox.radius, |h| h.radius);
    // (Whole world tenths: 0.4 / 0.4 / 0.5 / 0.6 of a unit.)
    assert!(r(1) <= r(7) && r(7) < r(14) && r(14) < r(19));
}

#[test]
fn held_31_frames_it_is_the_bigger_flare_and_held_to_the_end_the_giant_flare() {
    // (`flare(n)` holds special through the reference's frame n: let go on n + 1.)
    let (_, free, moves) = flare(30, 0, 150);
    assert_eq!(
        moves,
        vec![MoveId::NSpecial as u8, FLARE as u8],
        "30 held: the flare"
    );
    assert_eq!(free, Some(31 + 65 - 1), "FAF 65 from a release on 31");
    let (_, free, moves) = flare(31, 0, 150);
    assert_eq!(
        moves,
        vec![MoveId::NSpecial as u8, MEGAFLARE as u8],
        "31 held: the bigger flare"
    );
    assert_eq!(
        free,
        Some(32 + 73 - 1),
        "FAF 73 from a release on 32: the reference's 104 from the start"
    );
    let (_, free, _) = flare(33, 0, 150);
    assert_eq!(free, Some(34 + 73 - 1));
    let (_, free, moves) = flare(300, 0, 250);
    assert_eq!(moves[..2], [MoveId::NSpecial as u8, GIGAFLARE as u8]);
    assert_eq!(free, Some(182), "the giant flare's FAF 182");
}

#[test]
fn shield_cancels_the_charge_from_frame_20_in_7_frames_and_keeps_nothing() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NSpecial);
    for _ in 1..=24 {
        sim.tick(inp(0, 0, SPECIAL));
    }
    sim.tick(inp(0, 0, SPECIAL | SHIELD));
    assert_eq!(sim.f().move_id, FLARE_CANCEL as u8);
    let mut n = 0;
    while sim.f().state == S::Attack && n < 20 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    // (The press's frame is the cancel's first: 7 frames, 6 more ticks.)
    assert_eq!(n, 6);
    assert_eq!(sim.state.projectiles.iter().filter(|p| p.active).count(), 0);
}

#[test]
fn the_bigger_flares_burst_pulls_in_and_the_giant_flares_rings_hit_by_distance() {
    let sim = duel();
    let w = kit(&sim);
    // The bigger flare's burst: multi-hits at the vortex angle every 3 frames, then its own final hit.
    let mega = w.get(MEGAFLARE as u8).projectile.unwrap().burst.unwrap();
    assert_eq!(mega.rehit, 3);
    let multi = rings_at(&mega.phases, 10)[0].unwrap();
    assert_eq!(
        (multi.angle, multi.fixed_kb, multi.set_weight),
        (ANGLE_VORTEX, 50, true)
    );
    let last = rings_at(&mega.phases, 19)[0].unwrap();
    assert_eq!(
        (last.damage, last.angle, last.group),
        (Fx::from_int(9), 58, 1)
    );
    // The giant flare: three rings at once, the inner one first.
    let giga = w.get(GIGAFLARE as u8).projectile.unwrap().burst.unwrap();
    let early = rings_at(&giga.phases, 7);
    let radii: Vec<Fx> = early.iter().flatten().map(|h| h.radius).collect();
    assert_eq!(radii.len(), 3);
    assert!(radii[0] < radii[1] && radii[1] < radii[2]);
    let finals: Vec<Fx> = rings_at(&giga.phases, 25)
        .iter()
        .flatten()
        .map(|h| h.damage)
        .collect();
    assert_eq!(
        finals,
        vec![Fx::from_int(11), Fx::from_int(8), Fx::from_int(4)]
    );
    // In play: a duelist next to the giant flare's burst takes the inner ring's 3.7%; one far out the outer ring's 2%.
    // (Next to it the fireball itself hits first, 6%, then the burst 7 frames later; far out the fireball's flight ends short of the
    // duelist and the burst's outer ring reaches it.)
    let (rises, _, _) = flare(300, 25, 220);
    assert_eq!(rises.first().map(|r| r.1), Some(Fx::from_int(6) * k()));
    assert_eq!(rises.get(1).map(|r| r.0 - rises[0].0), Some(7));
    assert_eq!(rises.get(1).map(|r| r.1), Some(fx(37, 10) * k()));
    let (rises, _, _) = flare(300, 75, 220);
    assert_eq!(rises.first().map(|r| r.1), Some(Fx::from_int(2) * k()));
}

#[test]
fn the_wing_powers_the_flare_too() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(18, 10), -1);
    sim.state.fighters[0].wing_points = Fx::from_int(90);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NSpecial);
    for _ in 1..=45 {
        sim.tick(inp(0, 0, 0));
    }
    // (The fireball and its burst: 3% + 10.5%.)
    let want = (Fx::from_int(3) + fx(105, 10)) * fx(13, 10) * k();
    assert!((sim.fighter(1).percent - want).abs() <= fx(1, 1000));
}

#[test]
fn the_specials_are_resimulated_exactly() {
    // Two exiles mashing specials: a snapshot resimulated gives the same checksums.
    let mut sim = Sim::with_chars([EXILE, EXILE, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    let mut rng = sim_core::Rng::new(5);
    let mut inputs = Vec::new();
    for _ in 0..700 {
        let r = rng.next_u32();
        let pick = |s: u32| {
            let b = if s & 0x300 == 0 {
                SPECIAL
            } else {
                (s >> 16) as u16 & 0x1f
            };
            inp(
                ((s & 0xff) as i8) / 2 * 2,
                (((s >> 8) & 0xff) as i8) / 2 * 2,
                b,
            )
        };
        inputs.push([
            pick(r),
            pick(r.rotate_left(13)),
            Default::default(),
            Default::default(),
        ]);
    }
    let snapshot = sim.state;
    let mut straight = Vec::new();
    for i in &inputs {
        sim_core::step(&mut sim.state, &sim.content, i);
        straight.push(sim.state.checksum());
    }
    let mut again = snapshot;
    for (n, i) in inputs.iter().enumerate() {
        sim_core::step(&mut again, &sim.content, i);
        assert_eq!(again.checksum(), straight[n], "frame {n}");
    }
    let _ = Vec2::ZERO;
}
