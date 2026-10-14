//! The Dagger's specials (phase 1, sim 121): the gun and its partner's bullets, the curse bolt (E54) and its powered version, the
//! guard (E55) with its gauge gain and backhand, the partner's counter and reflector stance, the partner's wings. Reference data: the
//! thirteenth moveset's `REFERENCE_DATA.md` (`docs/movesets/dagger/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{JUMP, SHIELD, SPECIAL};
use sim_core::moves::{dagger_slots, MoveId};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const PHANTOM: u8 = 12;

/// The phantom (player 0, facing right at 0) against a duelist (player 1, `gap` to its right), a two-fighter match.
fn duel(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars([PHANTOM, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

fn summon(sim: &mut Sim) {
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 1800;
}

fn mult(sim: &Sim) -> Fx {
    sim.content.rules.damage_mult
}

/// Presses special (with the stick) on the first frame and then holds `held` input; returns the reference frames on which player 1's
/// percent rose and by how much, over `frames` frames (the press frame is reference frame 1).
fn hits(sim: &mut Sim, stick: (i8, i8), held: u16, frames: u16) -> Vec<(u16, Fx)> {
    let mut out = Vec::new();
    let mut last = sim.fighter(1).percent;
    sim.tick(inp(stick.0, stick.1, SPECIAL));
    for t in 2..=frames {
        sim.tick(inp(0, 0, held));
        let p = sim.fighter(1).percent;
        if p != last {
            out.push((t, p - last));
            last = p;
        }
    }
    out
}

// ---- The gun ------------------------------------------------------------------------------------------------------------------

#[test]
fn the_gun_fires_on_frame_12_with_three_bands_by_distance_and_ends_on_37() {
    let k = |sim: &Sim| mult(sim);
    for (gap, damage) in [(fx(18, 10), 50), (fx(40, 10), 30), (fx(75, 10), 10)] {
        let mut sim = duel(gap);
        let got = hits(&mut sim, (0, 0), 0, 40);
        assert_eq!(
            got,
            vec![(12, Fx::from_ratio(damage, 10) * k(&sim))],
            "at {gap:?}"
        );
        // Close in it flinches or launches; far away it does neither.
        if damage == 10 {
            assert_ne!(sim.fighter(1).state, S::Hitstun);
        }
    }
    // FAF 37: the phantom acts again on the reference's frame 37.
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, 0, SPECIAL));
    let mut t = 1;
    while sim.f().state == S::Attack && t < 80 {
        sim.tick(inp(0, 0, 0));
        t += 1;
    }
    assert_eq!(t + 1, 37);
}

#[test]
fn pressing_again_fires_25_frames_later_and_holding_keeps_firing() {
    // A press on frame 20 is kept and fires again from 26: shots on 12 and 37 (far away: no hitlag to delay the second).
    let mut sim = duel(fx(75, 10));
    let mut got = Vec::new();
    let mut last = Fx::ZERO;
    sim.tick(inp(0, 0, SPECIAL));
    for t in 2..=70u16 {
        let b = if t == 20 { SPECIAL } else { 0 };
        sim.tick(inp(0, 0, b));
        if sim.fighter(1).percent != last {
            got.push(t);
            last = sim.fighter(1).percent;
        }
    }
    assert_eq!(&got[..2], &[12, 37]);
    // Held: it fires again when the move would end (slower than pressing) [E].
    let mut sim = duel(fx(75, 10));
    let got = hits(&mut sim, (0, 0), SPECIAL, 90);
    assert_eq!(
        got.iter().map(|g| g.0).collect::<Vec<_>>(),
        vec![12, 48, 84]
    );
}

#[test]
fn with_the_partner_each_shot_is_three_bullets_and_the_move_lasts_to_42() {
    let mut sim = duel(fx(40, 10));
    summon(&mut sim);
    let got = hits(&mut sim, (0, 0), 0, 50);
    let k = mult(&sim);
    assert_eq!(
        got,
        vec![
            (12, Fx::from_int(3) * k),
            (18, fx(3, 2) * k),
            (24, fx(3, 2) * k)
        ]
    );
    let mut sim = duel(Fx::from_int(10));
    summon(&mut sim);
    sim.tick(inp(0, 0, SPECIAL));
    let mut t = 1;
    while sim.f().state == S::Attack && t < 80 {
        sim.tick(inp(0, 0, 0));
        t += 1;
    }
    assert_eq!(t + 1, 42);
}

#[test]
fn the_stick_dodges_jumps_and_in_the_air_fires_below_or_spins() {
    // Sideways from frame 26 with special: the dodge forward (intangible 4-13, its shot on 20).
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 2..=25 {
        sim.tick(inp(0, 0, SPECIAL));
    }
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, dagger_slots::GUN_DASH as u8);
    let x0 = sim.f().pos.x;
    let mut intangible = Vec::new();
    for t in 2..=20u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().invuln > 0 {
            intangible.push(t);
        }
    }
    assert_eq!(intangible.first(), Some(&4));
    assert!(sim.f().pos.x > x0 + Fx::from_int(2), "it moved forward");
    // Back.
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 2..=25 {
        sim.tick(inp(0, 0, SPECIAL));
    }
    sim.tick(inp(-127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, dagger_slots::GUN_BACK as u8);
    // Jump on the ground.
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 2..=25 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().move_id, dagger_slots::GUN_JUMP as u8);
    sim.ticks(6, inp(0, 0, 0));
    assert!(!sim.f().grounded(), "it jumped");
    // In the air: down fires below, up spins.
    for (y, slot) in [
        (-127i8, dagger_slots::GUN_BELOW),
        (127, dagger_slots::GUN_SPIRAL),
    ] {
        let mut sim = duel(Fx::from_int(10));
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, SPECIAL));
        for _ in 2..=13 {
            sim.tick(inp(0, 0, SPECIAL));
        }
        sim.tick(inp(0, y, SPECIAL));
        assert_eq!(sim.f().move_id, slot as u8, "{y}");
    }
}

#[test]
fn the_gun_stalls_in_the_air() {
    let mut sim = duel(Fx::from_int(10));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, -fx(1, 4));
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().vel.y, Fx::ZERO, "the fall stopped");
    sim.ticks(10, inp(0, 0, 0));
    assert!(sim.f().pos.y > Fx::from_int(29));
}

// ---- The curse bolt -------------------------------------------------------------------------------------------------------------

#[test]
fn the_bolt_hits_erupts_a_frame_later_and_curses_1_percent_every_45_frames_for_361() {
    let mut sim = duel(Fx::from_int(3));
    let got = hits(&mut sim, (127, 0), 0, 420);
    let k = mult(&sim);
    // The bolt (1%), then the eruption (2%) on the next frame, then 8 ticks of 1% 45 frames apart.
    assert_eq!(got[0].1, Fx::ONE * k);
    assert_eq!(got[1], (got[0].0 + 1, Fx::from_int(2) * k));
    let ticks: Vec<_> = got[2..].to_vec();
    assert_eq!(ticks.len(), 8, "{got:?}");
    for w in ticks.windows(2) {
        assert_eq!(w[1].0 - w[0].0, 45);
    }
    assert!(ticks.iter().all(|t| t.1 == Fx::ONE * k));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(11) * k,
        "the reference's 11% in all"
    );
    assert_eq!(sim.fighter(1).curse_left, 0);
}

#[test]
fn a_shield_takes_the_bolt_with_no_eruption_and_no_curse() {
    let mut sim = duel(Fx::from_int(3));
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..60 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    assert_eq!(sim.fighter(1).curse_left, 0);
    assert!(sim.state.projectiles.iter().all(|p| !p.active));
}

#[test]
fn with_the_partner_the_bolt_is_stronger_and_curses_1_5_every_40() {
    let mut sim = duel(Fx::from_int(4));
    summon(&mut sim);
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, dagger_slots::BOLT_POWERED as u8);
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
    }
    let v = sim.fighter(1);
    assert!(v.percent > Fx::ZERO);
    assert_eq!((v.curse_every, v.curse_damage), (40, fx(3, 2)));
    // Fired down from the air it bursts on the ground.
    let mut sim = duel(Fx::from_int(9));
    summon(&mut sim);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(2), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(127, 0, SPECIAL));
    let mut burst = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        burst |= sim.state.projectiles.iter().any(|p| p.active && p.bursting);
    }
    assert!(burst, "it burst on the ground");
}

// ---- The guard and the stance ------------------------------------------------------------------------------------------------------

/// The duelist's forward tilt on the phantom (player 1 starts it at once), the phantom in `id` started `lead` frames earlier.
fn struck_in(id: MoveId, held: bool, lead: u16) -> Sim {
    let mut sim = duel(Fx::from_int(1));
    sim.state.fighters[1].char_id = 0;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
    for _ in 0..lead {
        sim.tick(inp(0, 0, if held { SPECIAL } else { 0 }));
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::FTilt);
    sim
}

#[test]
fn the_guard_takes_0_4_of_a_hit_without_flinching_and_fills_the_gauge_6_7368_times_faster() {
    let mut sim = struck_in(MoveId::DownSpecial, true, 3);
    let mut gauge_before = Fx::ZERO;
    for _ in 0..20 {
        gauge_before = sim.f().gauge;
        sim.tick(inp(0, 0, SPECIAL));
        if sim.f().percent > Fx::ZERO {
            break;
        }
    }
    let taken = sim.f().percent;
    // The duelist's forward tilt (its tip or blade) times 1.2, times 0.4.
    assert!(taken > Fx::ZERO);
    assert_ne!(sim.f().state, S::Hitstun, "super armor");
    let full = taken / fx(4, 10);
    assert!(
        full >= Fx::from_int(8) && full <= Fx::from_int(15),
        "{full:?}"
    );
    let gain = sim.f().gauge - gauge_before - fx(925, 100_000);
    let want = taken * fx(13, 10) * fx(6737, 1000);
    assert!((gain - want).abs() < fx(1, 100), "{gain:?} vs {want:?}");
    assert!(sim.f().withstood > 0);
    // 28.6% (before the 0.4) fills it from empty in one hit [R]: 28.6 x 0.4 x 1.3 x 6.7368 > 100.
    assert!(fx(286, 10) * fx(4, 10) * fx(13, 10) * fx(6737, 1000) >= Fx::from_int(100));
}

#[test]
fn letting_go_after_a_hit_answers_with_the_backhand_and_holding_keeps_the_guard_up() {
    // Held: the guard lasts past frame 19 (its hold loop), up to 120.
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
    let mut t = 1;
    while sim.f().state == S::Attack && t < 300 {
        sim.tick(inp(0, 0, SPECIAL));
        t += 1;
        if t == 60 {
            assert_eq!(sim.f().move_id, dagger_slots::GUARD_HOLD as u8);
        }
    }
    // (120 frames of guard, then 33 of lag.)
    assert!((150..=156).contains(&t), "{t}");
    // Not held and not hit: FAF 53.
    let mut sim = duel(Fx::from_int(10));
    sim.tick(inp(0, -127, SPECIAL));
    let mut t = 1;
    while sim.f().state == S::Attack && t < 300 {
        sim.tick(inp(0, 0, 0));
        t += 1;
    }
    assert_eq!(t + 1, 53);
    // Hit while held, then let go: the backhand.
    let mut sim = struck_in(MoveId::DownSpecial, true, 3);
    for _ in 0..20 {
        sim.tick(inp(0, 0, SPECIAL));
        if sim.f().withstood > 0 {
            break;
        }
    }
    assert!(sim.f().withstood > 0);
    let mut answered = false;
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        answered |= sim.f().move_id == dagger_slots::BACKHAND as u8;
    }
    assert!(answered);
}

#[test]
fn with_the_partner_the_down_special_counters_at_1_6_and_at_least_12_percent() {
    let mut sim = struck_in(dagger_slots::STANCE, false, 4);
    let mut answered = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        answered |= sim.f().move_id == dagger_slots::COUNTERATTACK as u8;
    }
    assert!(answered);
    assert_eq!(sim.f().percent, Fx::ZERO);
    // The answer: the caught damage x1.6, at least 12%.
    assert!(
        sim.fighter(1).percent >= Fx::from_int(12) * mult(&sim),
        "{:?}",
        sim.fighter(1).percent
    );
}

#[test]
fn with_the_partner_the_up_special_flies_intangible_then_falls_helpless() {
    let mut sim = duel(Fx::from_int(10));
    summon(&mut sim);
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().move_id, dagger_slots::WINGS as u8);
    assert!(sim.f().invuln >= 24);
    let mut top = Fx::ZERO;
    for _ in 0..120 {
        sim.tick(inp(0, 0, 0));
        top = top.max(sim.f().pos.y);
        if sim.f().state == S::Helpless {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Helpless);
    assert!(top > Fx::from_int(8), "a long flight up: {top:?}");
}
