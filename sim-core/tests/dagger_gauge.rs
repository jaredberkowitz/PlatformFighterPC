//! The phantom's gauge and its partner (E53, sim 121; the reference's, ssbwiki.com's article on the gauge and ultimateframedata.com,
//! 13.0.1): the fill by itself, from damage taken (and not dealt), the stock-gap and crowd multipliers, the guard's gain, the partner's
//! 1800 frames, the time each percent taken cuts, the stun that stops the clock, specials that do not spend it, and what a KO leaves.
//! Reference data: the thirteenth moveset's `REFERENCE_DATA.md` (`docs/movesets/joker/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{dagger_slots, MoveId};
use sim_core::state::FighterState as S;
use sim_core::{FighterParams, Fx, GameState, MatchRules};

const PHANTOM: u8 = 12;

/// The phantom (player 0) against a duelist (player 1), a two-fighter match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([PHANTOM, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(8), -1);
    sim
}

/// `damage` percent added to the phantom by the duelist (as a hit does: through the gauge's damage hook).
fn take(sim: &mut Sim, damage: Fx) {
    let f = &mut sim.state.fighters[0];
    let before = f.percent;
    f.percent += damage;
    let added = sim.state.fighters[0].percent - before;
    sim_core::gauge::on_damage(&mut sim.state, &sim.content, 1, 0, added);
}

#[test]
fn the_gauge_fills_by_itself_0_00925_a_frame_and_not_on_the_revival_platform() {
    let mut sim = duel();
    sim.ticks(1000, inp(0, 0, 0));
    assert_eq!(sim.f().gauge, fx(925, 100_000) * Fx::from_int(1000));
    // On the revival platform it waits.
    let mut sim = duel();
    sim.state.fighters[0].state = S::Respawn;
    sim.state.fighters[0].invuln = 200;
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().gauge, Fx::ZERO);
    // About three minutes from empty to full by itself [R: "10,810 frames"]; 0.00925 is 606 / 65536 in the fixed point, so 10,815 [D].
    let g = FighterParams::phantom_gauge();
    let frames = (g.max / g.per_frame).floor_int() + 1;
    assert!((10_810..=10_816).contains(&frames), "{frames}");
}

#[test]
fn damage_taken_fills_it_1_3_a_percent_and_damage_dealt_does_not() {
    let mut sim = duel();
    take(&mut sim, Fx::from_int(3));
    // (3% taken: 3.9 units, the reference's example.)
    assert!(
        (sim.f().gauge - fx(39, 10)).abs() <= fx(1, 10_000),
        "{:?}",
        sim.f().gauge
    );
    // A hit the phantom deals adds nothing to its gauge.
    let mut sim = duel();
    sim.stand(1, Fx::from_ratio(13, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.fighter(1).percent > Fx::ZERO);
    assert!(
        sim.f().gauge < fx(1, 10),
        "only the fill by itself: {:?}",
        sim.f().gauge
    );
    // A real hit taken: the damage actually added (the one-on-one 1.2 counted) times 1.3.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-1), 1);
    sim.stand(1, Fx::ZERO, -1);
    sim.state.fighters[1].char_id = 0;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::FTilt);
    let mut gauge_before_hit = Fx::ZERO;
    for _ in 0..20 {
        gauge_before_hit = sim.f().gauge;
        sim.tick(inp(0, 0, 0));
        if sim.f().percent > Fx::ZERO {
            break;
        }
    }
    let added = sim.f().percent;
    assert_eq!(
        sim.f().gauge - gauge_before_hit,
        added * fx(13, 10) + fx(925, 100_000),
        "1.3 a percent actually taken, plus that frame's fill"
    );
}

#[test]
fn a_stock_or_two_behind_the_gain_grows_and_with_more_fighters_the_crowd_row_applies() {
    for (stocks, crowd, expect) in [
        // (the phantom's stocks against 3, fighters in the match, units from 10% taken)
        (3u8, 2u8, fx(13, 1)),
        (2, 2, fx(1625, 100)),
        (1, 2, fx(1950, 100)),
        (2, 3, fx(1625, 100) * fx(11, 10)),
        (1, 4, fx(1950, 100) * fx(1333, 1000)),
        (3, 4, fx(13, 1)),
    ] {
        let mut sim = duel();
        sim.state.rules = MatchRules {
            stocks: 3,
            ..sim.state.rules
        };
        sim.state.roster = (1u8 << crowd) - 1;
        for i in 0..4 {
            sim.state.fighters[i].stocks = 3;
        }
        sim.state.fighters[0].stocks = stocks;
        take(&mut sim, Fx::from_int(10));
        let g = sim.f().gauge;
        let tol = fx(1, 100);
        assert!(
            g >= expect - tol && g <= expect + tol,
            "{stocks} stocks, {crowd} fighters: {g:?} vs {expect:?}"
        );
    }
}

#[test]
fn a_full_gauge_brings_the_partner_for_1800_frames_and_nothing_fills_it_meanwhile() {
    let mut sim = duel();
    // 76.93% taken summons it at once (the reference: "around 77%").
    take(&mut sim, fx(7693, 100));
    assert_eq!(sim.f().gauge_mode, 1800);
    assert_eq!(sim.f().gauge, Fx::from_int(100));
    take(&mut sim, Fx::ZERO);
    sim.ticks(1799, inp(0, 0, 0));
    assert_eq!(sim.f().gauge_mode, 1);
    sim.tick(inp(0, 0, 0));
    assert_eq!((sim.f().gauge_mode, sim.f().gauge), (0, Fx::ZERO));
    // The body's own numbers do not change with it (the reference's partner gives no speed).
    let p = sim.content.fighters[usize::from(PHANTOM)];
    assert_eq!(p.powered(true), p);
}

#[test]
fn each_percent_taken_with_the_partner_out_cuts_16_frames_or_10_2_with_four_fighters() {
    let mut sim = duel();
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 1800;
    // A 15% hit (as added) cuts 240 frames (the reference's example: 4 seconds).
    take(&mut sim, Fx::from_int(15));
    assert_eq!(sim.f().gauge_mode, 1800 - 240);
    // 112.5% in all ends it (the reference's figure since 7.0.0).
    take(&mut sim, fx(975, 10));
    assert_eq!((sim.f().gauge_mode, sim.f().gauge), (0, Fx::ZERO));
    // Four fighters: 10.2 a percent.
    let mut sim = duel();
    sim.state.roster = 0b1111;
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 1800;
    take(&mut sim, Fx::from_int(10));
    assert_eq!(sim.f().gauge_mode, 1800 - 102);
}

#[test]
fn a_broken_shield_stops_the_partners_clock() {
    let mut sim = duel();
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 1000;
    sim_core::fighter::break_shield(&mut sim.state.fighters[0], &sim.content.rules);
    assert_eq!(sim.f().state, S::ShieldBreak);
    let mut n = 0;
    while sim.f().state == S::ShieldBreak && n < 600 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    assert!(n > 10);
    // (Counted while stunned: nothing; the first frame back counts one.)
    assert!(sim.f().gauge_mode >= 999, "{}", sim.f().gauge_mode);
    // The champion's gauge (no `halts`) keeps counting.
    let mut sim = Sim::with_chars([6, 0, 0, 0]);
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 600;
    sim_core::fighter::break_shield(&mut sim.state.fighters[0], &sim.content.rules);
    sim.ticks(20, inp(0, 0, 0));
    assert_eq!(sim.f().gauge_mode, 580);
}

#[test]
fn with_the_partner_out_every_special_changes_and_none_spends_the_gauge() {
    for (stick_x, stick_y, expect) in [
        (0i8, 0i8, MoveId::NSpecial),
        (127, 0, dagger_slots::BOLT_POWERED),
        (0, -127, dagger_slots::STANCE),
        (0, 127, dagger_slots::WINGS),
    ] {
        let mut sim = duel();
        sim.state.fighters[0].gauge = Fx::from_int(100);
        sim.state.fighters[0].gauge_mode = 1800;
        sim.tick(inp(stick_x, stick_y, SPECIAL));
        assert_eq!(sim.f().state, S::Attack, "{expect:?}");
        assert_eq!(sim.f().move_id, expect as u8);
        assert!(sim.f().move_powered);
        assert_eq!(sim.f().gauge_mode, 1799, "the partner stays");
    }
}

#[test]
fn a_ko_leaves_20_units_or_25_and_30_further_behind() {
    for lose_to in [3u8, 2, 1] {
        // (The phantom against a duelist with 4 stocks: after this KO it has `lose_to` left, `4 - lose_to` behind, and the gap after
        // the KO picks the units.)
        let mut sim = duel();
        sim.state.rules = MatchRules {
            stocks: 4,
            ..sim.state.rules
        };
        sim.state.fighters[1].stocks = 4;
        sim.state.fighters[0].stocks = lose_to + 1;
        sim.state.fighters[0].gauge = Fx::from_int(100);
        sim.state.fighters[0].gauge_mode = 900;
        sim.state.fighters[0].pos.y = Fx::from_int(-100);
        sim.state.fighters[0].platform = sim_core::state::NONE;
        sim_core::combat::check_ko(&mut sim.state, &sim.content);
        // After it: 4 - lose_to stocks behind.
        let behind = 4 - lose_to;
        let want = match behind {
            0 | 1 => 20,
            2 => 25,
            _ => 30,
        };
        assert_eq!(sim.f().gauge, Fx::from_int(want), "{behind} behind");
        assert_eq!(sim.f().gauge_mode, 0);
    }
    // Three behind: 30.
    let mut sim = duel();
    sim.state.rules = MatchRules {
        stocks: 4,
        ..sim.state.rules
    };
    sim.state.fighters[1].stocks = 4;
    sim.state.fighters[0].stocks = 2;
    sim.state.fighters[0].pos.y = Fx::from_int(-100);
    sim_core::combat::check_ko(&mut sim.state, &sim.content);
    assert_eq!(sim.f().gauge, Fx::from_int(30));
    // The champion's gauge is still emptied by a KO (owner decision 9).
    let mut sim = Sim::with_chars([6, 0, 0, 0]);
    sim.state.fighters[0].gauge = Fx::from_int(50);
    sim.state.fighters[0].pos.y = Fx::from_int(-100);
    sim_core::combat::check_ko(&mut sim.state, &sim.content);
    assert_eq!(sim.f().gauge, Fx::ZERO);
}

#[test]
fn the_gauge_and_the_partner_are_in_the_checksum_and_resimulate_exactly() {
    let content = sim_core::Content::placeholder();
    let mut a = GameState::new(&content, 3, [PHANTOM, PHANTOM, 0, 0]);
    a.fighters[0].gauge = Fx::from_int(99);
    let mut b = a;
    b.fighters[0].gauge = Fx::from_int(98);
    assert_ne!(a.checksum(), b.checksum());
    let snapshot = a;
    let inputs = [inp(0, 0, 0); 4];
    let mut sums = Vec::new();
    for _ in 0..200 {
        sim_core::step(&mut a, &content, &inputs);
        sums.push(a.checksum());
    }
    let mut c = snapshot;
    for s in sums {
        sim_core::step(&mut c, &content, &inputs);
        assert_eq!(c.checksum(), s);
    }
}
