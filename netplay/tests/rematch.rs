//! Rematches: after a match, both players ask, and a fresh match starts over the same link with a new seed. Nothing of the
//! finished match reaches the new one, and the two sides stay in step through it.

use netplay::packet::Setup;
use netplay::peer::{Peer, Status};
use netplay::session::{Advance, Event};
use netplay::testlink::{pair, Clock, LinkParams, TestLink};
use sim_core::{Content, Input, MatchRules, Rng};

fn scripted(player: usize, frame: u32) -> Input {
    let mut h = Rng::new(u64::from(frame / 7) * 8 + player as u64 + 5);
    Input {
        stick_x: (h.range(255) as i32 - 127) as i8,
        stick_y: 0,
        buttons: if h.range(5) == 0 {
            h.range(32) as u16
        } else {
            0
        },
    }
}

struct Pair {
    a: Peer<TestLink>,
    b: Peer<TestLink>,
    clock: Clock,
    base: Content,
    ticks: u32,
}

fn start(params: LinkParams, seed: u64) -> Pair {
    let base = Content::placeholder();
    let (la, lb, clock) = pair(params, seed);
    let setup = Setup {
        seed: 4242,
        chars: [0, 1, 0, 1],
        active: 0b0011,
        input_delay: 2,
        rules: MatchRules {
            stocks: 2,
            time_limit: 0,
        },
        ..Setup::default()
    };
    Pair {
        a: Peer::host(la, &base, setup),
        b: Peer::join(lb, &base, vec![]),
        clock,
        base,
        ticks: 0,
    }
}

impl Pair {
    /// One tick of both peers; returns whether each simulated a frame.
    fn tick(&mut self) -> (Status, Status) {
        let n = self.ticks;
        let sa = self.a.update(&self.base, scripted(0, n));
        let sb = self.b.update(&self.base, scripted(1, n));
        self.clock.advance();
        self.ticks += 1;
        (sa, sb)
    }

    fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    /// Ticks until both have a session whose frame has reached `frames`.
    fn run_until_frame(&mut self, frames: u32, limit: u32) -> bool {
        for _ in 0..limit {
            self.tick();
            let fa = self.a.state().map_or(0, |s| s.frame);
            let fb = self.b.state().map_or(0, |s| s.frame);
            if fa >= frames && fb >= frames {
                return true;
            }
        }
        false
    }

    fn desyncs(&mut self) -> usize {
        let mut n = 0;
        for e in self
            .a
            .drain_events()
            .into_iter()
            .chain(self.b.drain_events())
        {
            if matches!(e, Event::Desync { .. }) {
                n += 1;
            }
        }
        n
    }
}

fn clean() -> LinkParams {
    LinkParams {
        min_latency: 1,
        max_latency: 1,
        loss_percent: 0,
        duplicate_percent: 0,
    }
}

fn lossy() -> LinkParams {
    LinkParams {
        min_latency: 1,
        max_latency: 5,
        loss_percent: 15,
        duplicate_percent: 5,
    }
}

#[test]
fn both_asking_starts_a_fresh_match_with_a_new_seed() {
    let mut p = start(clean(), 1);
    assert!(p.run_until_frame(300, 2000), "the first match runs");
    let first_checksum = p.a.state().unwrap().checksum();
    assert_eq!(p.a.rematch_state(), (false, false));

    p.a.request_rematch();
    p.b.request_rematch();
    // Wait for the new match: its frame counter starts over.
    let mut restarted = false;
    for _ in 0..600 {
        p.tick();
        if let (Some(sa), Some(sb)) = (p.a.state(), p.b.state()) {
            if sa.frame < 100 && sb.frame < 100 && sa.frame > 5 && sb.frame > 5 {
                restarted = true;
                break;
            }
        }
    }
    assert!(restarted, "a new match starts after both ask");
    assert_eq!(
        p.a.rematch_state(),
        (false, false),
        "and the asking is cleared"
    );

    // The new match is a real one: it runs on and the two sides never disagree.
    assert!(p.run_until_frame(400, 3000));
    assert_eq!(p.desyncs(), 0, "no desync in the rematch");
    assert_ne!(p.a.state().unwrap().checksum(), first_checksum);
    // And it can happen again.
    p.a.request_rematch();
    p.b.request_rematch();
    p.run(200);
    assert!(p.run_until_frame(150, 3000));
    assert_eq!(p.desyncs(), 0);
}

#[test]
fn one_side_asking_waits_for_the_other() {
    let mut p = start(clean(), 2);
    assert!(p.run_until_frame(200, 2000));
    p.a.request_rematch();
    p.run(300);
    // Still the old match on both sides (its frame kept counting), and the joiner can see the request.
    assert!(p.a.state().unwrap().frame > 300, "the old match carries on");
    assert_eq!(p.a.rematch_state(), (true, false));
    assert_eq!(p.b.rematch_state(), (false, true));
    p.b.request_rematch();
    let mut restarted = false;
    for _ in 0..600 {
        p.tick();
        if p.a.state().is_some_and(|s| s.frame < 100) && p.b.state().is_some_and(|s| s.frame < 100)
        {
            restarted = true;
            break;
        }
    }
    assert!(restarted, "the match starts when the second one asks");
    assert!(p.run_until_frame(300, 3000));
    assert_eq!(p.desyncs(), 0);
}

#[test]
fn a_rematch_survives_a_rough_link() {
    for seed in 0..10 {
        let mut p = start(lossy(), seed);
        assert!(p.run_until_frame(250, 6000), "seed {seed}: first match");
        p.a.request_rematch();
        p.run(40);
        p.b.request_rematch();
        let mut restarted = false;
        for _ in 0..3000 {
            p.tick();
            if p.a.state().is_some_and(|s| s.frame < 200 && s.frame > 3)
                && p.b.state().is_some_and(|s| s.frame < 200 && s.frame > 3)
            {
                restarted = true;
                break;
            }
        }
        assert!(restarted, "seed {seed}: the rematch starts");
        assert!(p.run_until_frame(500, 8000), "seed {seed}: and runs");
        assert_eq!(p.desyncs(), 0, "seed {seed}: without a desync");
    }
}

#[test]
fn asking_before_a_match_does_nothing() {
    let mut p = start(clean(), 3);
    p.a.request_rematch();
    assert_eq!(p.a.rematch_state(), (false, false));
    assert!(p.run_until_frame(100, 2000));
    let (sa, _) = p.tick();
    assert_eq!(sa, Status::Running(Advance::Ran));
}

#[test]
fn the_hosts_stage_is_the_matchs_stage_on_both_sides() {
    let base = Content::placeholder();
    let (la, lb, clock) = pair(clean(), 9);
    let setup = Setup {
        seed: 1,
        chars: [0, 1, 0, 1],
        active: 0b0011,
        input_delay: 2,
        stage: 2,
        ..Setup::default()
    };
    let mut p = Pair {
        a: Peer::host(la, &base, setup),
        b: Peer::join(lb, &base, vec![]),
        clock,
        base,
        ticks: 0,
    };
    assert!(p.run_until_frame(120, 2000));
    for peer in [&p.a, &p.b] {
        let c = peer
            .match_content()
            .expect("a stage other than the first builds match content");
        assert_eq!(c.names.stage, "Flat Island");
        assert_eq!(c.stage.platforms.len(), 1);
    }
    assert_eq!(p.desyncs(), 0);
}
