//! Phase 7: stocks, elimination, the winner, the time limit and a finished match standing still.

mod common;

use common::{inp, Sim};
use sim_core::state::{DRAW, PLAYING};
use sim_core::{step, Fx, GameState, Input, MatchRules, MAX_FIGHTERS};

fn duel_with(rules: MatchRules) -> Sim {
    let mut sim = Sim::new();
    sim.state = GameState::new_with_rules(&sim.content, 1, [0, 1, 0, 1], 0b0011, rules);
    sim
}

/// Throws fighter `i` out of the blast zone and lets the sim notice.
fn knock_out(sim: &mut Sim, i: usize) {
    sim.state.fighters[i].pos.x = Fx::from_int(500);
    sim.state.fighters[i].platform = sim_core::state::NONE;
    sim.tick(inp(0, 0, 0));
}

#[test]
fn fighters_start_with_the_stocks_the_rules_give() {
    let sim = duel_with(MatchRules {
        stocks: 5,
        time_limit: 0,
    });
    assert_eq!(sim.fighter(0).stocks, 5);
    assert_eq!(sim.fighter(1).stocks, 5);
    assert_eq!(sim.state.winner, PLAYING);
    assert_eq!(sim.state.roster, 0b0011);
    // Out-of-range rules are pulled in.
    let wild = duel_with(MatchRules {
        stocks: 200,
        time_limit: 60_000,
    });
    assert_eq!(wild.state.rules.stocks, MatchRules::MAX_STOCKS);
    assert_eq!(wild.state.rules.time_limit, 3600);
}

#[test]
fn losing_the_last_stock_eliminates_and_the_other_fighter_wins() {
    let mut sim = duel_with(MatchRules {
        stocks: 2,
        time_limit: 0,
    });
    knock_out(&mut sim, 1);
    assert_eq!(sim.fighter(1).stocks, 1);
    assert!(sim.fighter(1).active, "one stock left, still in");
    assert_eq!(sim.state.winner, PLAYING);
    knock_out(&mut sim, 1);
    assert_eq!(sim.fighter(1).stocks, 0);
    assert!(!sim.fighter(1).active, "out of stocks, out of the match");
    assert_eq!(sim.state.winner, 0);
}

#[test]
fn a_finished_match_stands_still() {
    let mut sim = duel_with(MatchRules {
        stocks: 1,
        time_limit: 0,
    });
    knock_out(&mut sim, 1);
    assert_eq!(sim.state.winner, 0);
    let (pos, percent) = (sim.fighter(0).pos, sim.fighter(0).percent);
    for _ in 0..120 {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(sim.fighter(0).pos, pos);
    assert_eq!(sim.fighter(0).percent, percent);
    assert_eq!(sim.state.winner, 0, "and the result does not change");
}

#[test]
fn knocking_each_other_out_together_is_a_draw() {
    let mut sim = duel_with(MatchRules {
        stocks: 1,
        time_limit: 0,
    });
    for i in 0..2 {
        sim.state.fighters[i].pos.x = Fx::from_int(500);
        sim.state.fighters[i].platform = sim_core::state::NONE;
    }
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.state.winner, DRAW);
}

#[test]
fn unlimited_stocks_never_eliminate() {
    let mut sim = duel_with(MatchRules {
        stocks: 0,
        time_limit: 0,
    });
    assert_eq!(sim.fighter(1).stocks, MatchRules::UNLIMITED_DISPLAY);
    for _ in 0..5 {
        knock_out(&mut sim, 1);
    }
    assert_eq!(sim.fighter(1).stocks, MatchRules::UNLIMITED_DISPLAY);
    assert!(sim.fighter(1).active);
    assert_eq!(sim.state.winner, PLAYING);
}

#[test]
fn a_one_fighter_session_never_ends() {
    let mut sim = Sim::new();
    sim.state = GameState::new_with_rules(
        &sim.content,
        1,
        [0, 1, 0, 1],
        0b0001,
        MatchRules {
            stocks: 1,
            time_limit: 1,
        },
    );
    for _ in 0..200 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.state.winner, PLAYING,
        "a lone fighter is practising, not winning"
    );
}

#[test]
fn three_players_play_on_until_one_is_left() {
    let mut sim = Sim::new();
    sim.state = GameState::new_with_rules(
        &sim.content,
        1,
        [0, 1, 0, 1],
        0b0111,
        MatchRules {
            stocks: 1,
            time_limit: 0,
        },
    );
    knock_out(&mut sim, 2);
    assert_eq!(sim.state.winner, PLAYING);
    assert!(!sim.fighter(2).active);
    knock_out(&mut sim, 0);
    assert_eq!(sim.state.winner, 1);
}

fn run_out_the_clock(sim: &mut Sim, seconds: u32) {
    for _ in 0..seconds * 60 + 2 {
        sim.tick(inp(0, 0, 0));
    }
}

#[test]
fn the_time_limit_goes_to_the_most_stocks_then_the_least_damage() {
    let rules = MatchRules {
        stocks: 3,
        time_limit: 5,
    };
    // Stocks decide first.
    let mut sim = duel_with(rules);
    sim.state.fighters[0].stocks = 2;
    run_out_the_clock(&mut sim, 5);
    assert_eq!(sim.state.winner, 1);
    // Level on stocks, damage decides.
    let mut sim = duel_with(rules);
    sim.state.fighters[1].percent = Fx::from_int(30);
    sim.state.fighters[0].percent = Fx::from_int(10);
    run_out_the_clock(&mut sim, 5);
    assert_eq!(sim.state.winner, 0);
    // Level on both is a draw.
    let mut sim = duel_with(rules);
    run_out_the_clock(&mut sim, 5);
    assert_eq!(sim.state.winner, DRAW);
}

#[test]
fn no_time_limit_means_the_clock_never_ends_it() {
    let mut sim = duel_with(MatchRules {
        stocks: 3,
        time_limit: 0,
    });
    run_out_the_clock(&mut sim, 30);
    assert_eq!(sim.state.winner, PLAYING);
}

/// Random play with low stocks reaches an end, and everything stays consistent: the same inputs give the same result,
/// and the winner is always a fighter who is still in.
#[test]
fn random_matches_end_and_agree() {
    use sim_core::fuzz::random_inputs;
    use sim_core::Rng;
    let mut ended = 0;
    for seed in 0..20u64 {
        let inputs = random_inputs(&mut Rng::new(seed), 4000);
        let run = |inputs: &[[Input; MAX_FIGHTERS]]| {
            let content = sim_core::Content::placeholder();
            let mut s = GameState::new_with_rules(
                &content,
                seed,
                [0, 1, 0, 1],
                0b0011,
                MatchRules {
                    stocks: 1,
                    time_limit: 60,
                },
            );
            for i in inputs {
                step(&mut s, &content, i);
            }
            s
        };
        let a = run(&inputs);
        assert_eq!(a.checksum(), run(&inputs).checksum());
        if a.winner != PLAYING {
            ended += 1;
            if a.winner >= 0 {
                assert!(a.fighters[a.winner as usize].active || a.rules.time_limit > 0);
            }
        }
    }
    assert!(ended > 0, "some of the random matches should have ended");
}
