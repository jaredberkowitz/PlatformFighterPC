//! The lobby (`Peer::with_lobby`): after connecting, both players choose a fighter and a stage and lock them in; the match then
//! starts with those fighters on the drawn stage (their common choice, or one of the two). A rematch comes back to the lobby with the
//! last choices selected. All of it over lossy links, many times over.

use netplay::packet::{RejectReason, Setup};
use netplay::peer::{LobbyPick, Peer, Status};
use netplay::session::Event;
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

/// A built-in fighter's spec bytes.
fn builtin(n: u8) -> Vec<u8> {
    vec![0, n]
}

fn pick(fighter: u8, stage: u8, locked: bool) -> LobbyPick {
    LobbyPick {
        fighter: builtin(fighter),
        cosmetics: vec![fighter, 7],
        fighter_locked: locked,
        stage,
        stage_locked: locked,
    }
}

struct Pair {
    a: Peer<TestLink>,
    b: Peer<TestLink>,
    clock: Clock,
    base: Content,
    ticks: u32,
    last: (Status, Status),
}

fn start(params: LinkParams, seed: u64, host_seed: u64) -> Pair {
    let base = Content::placeholder();
    let (la, lb, clock) = pair(params, seed);
    let setup = Setup {
        seed: host_seed,
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
        a: Peer::host(la, &base, setup).with_lobby(pick(0, 0, false)),
        b: Peer::join(lb, &base, vec![]).with_lobby(pick(1, 0, false)),
        clock,
        base,
        ticks: 0,
        last: (Status::Connecting, Status::Connecting),
    }
}

impl Pair {
    fn tick(&mut self) {
        let n = self.ticks;
        let sa = self.a.update(&self.base, scripted(0, n));
        let sb = self.b.update(&self.base, scripted(1, n));
        self.clock.advance();
        self.ticks += 1;
        self.last = (sa, sb);
    }

    fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    /// Ticks until both are in a match that has reached `frames`.
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

    /// The fighters (by the match's content: `chars` of players 0 and 1) and the stage, as each side sees the running match.
    fn agreed(&self) -> (u8, u8) {
        let sa = self.a.match_setup().expect("host in a match").stage;
        let sb = self.b.match_setup().expect("joiner in a match").stage;
        assert_eq!(sa, sb, "both sides play the same stage");
        let ha = self.a.fighter_specs().expect("specs");
        let hb = self.b.fighter_specs().expect("specs");
        assert_eq!(ha, hb, "both sides agree on the fighters");
        (sa, ha.1[1])
    }

    fn desyncs(&mut self) -> usize {
        self.a
            .drain_events()
            .into_iter()
            .chain(self.b.drain_events())
            .filter(|e| matches!(e, Event::Desync { .. }))
            .count()
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
        max_latency: 6,
        loss_percent: 20,
        duplicate_percent: 5,
    }
}

#[test]
fn the_match_waits_in_the_lobby_until_both_lock_in() {
    let mut p = start(clean(), 1, 99);
    p.run(30);
    assert_eq!(p.last, (Status::Lobby, Status::Lobby));
    // Each sees the other's choice as it changes.
    assert_eq!(p.a.lobby_view().unwrap().theirs, Some(pick(1, 0, false)));
    p.b.set_lobby_pick(pick(3, 2, false));
    p.run(5);
    assert_eq!(p.a.lobby_view().unwrap().theirs, Some(pick(3, 2, false)));
    // One locked in is not enough.
    p.a.set_lobby_pick(pick(0, 2, true));
    p.run(60);
    assert_eq!(p.last, (Status::Lobby, Status::Lobby));
    assert!(p.a.state().is_none());
    // Both: the match starts on the common stage with the chosen fighters.
    p.b.set_lobby_pick(pick(3, 2, true));
    assert!(p.run_until_frame(120, 600), "the match starts");
    assert_eq!(p.agreed(), (2, 3));
    assert!(p.a.lobby_view().is_none() && p.b.lobby_view().is_none());
    assert_eq!(p.desyncs(), 0);
}

#[test]
fn two_different_stages_draw_one_of_the_two() {
    let mut seen = [false, false];
    for host_seed in 0..40u64 {
        let mut p = start(
            clean(),
            host_seed,
            host_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );
        p.a.set_lobby_pick(pick(0, 1, true));
        p.b.set_lobby_pick(pick(1, 3, true));
        assert!(p.run_until_frame(10, 600));
        let (stage, _) = p.agreed();
        assert!(stage == 1 || stage == 3, "stage {stage}");
        seen[usize::from(stage == 3)] = true;
    }
    assert_eq!(seen, [true, true], "each player's stage comes up");
}

#[test]
fn a_rematch_goes_back_to_the_lobby_with_the_last_choices() {
    let mut p = start(clean(), 3, 5);
    p.a.set_lobby_pick(pick(2, 1, true));
    p.b.set_lobby_pick(pick(4, 1, true));
    assert!(p.run_until_frame(200, 1000));
    p.a.request_rematch();
    p.b.request_rematch();
    p.run(30);
    assert_eq!(p.last, (Status::Lobby, Status::Lobby));
    assert_eq!(p.a.lobby_view().unwrap().mine, pick(2, 1, false));
    assert_eq!(p.b.lobby_view().unwrap().mine, pick(4, 1, false));
    // A different fighter this time.
    p.a.set_lobby_pick(pick(2, 1, true));
    p.b.set_lobby_pick(pick(5, 1, true));
    assert!(p.run_until_frame(100, 1000));
    assert_eq!(p.agreed(), (1, 5));
    assert_eq!(p.desyncs(), 0);
}

#[test]
fn many_rematches_over_a_rough_link_all_start() {
    for seed in 0..6u64 {
        let mut p = start(lossy(), seed, seed + 11);
        for round in 0..6u8 {
            let stage = round % 4;
            // One side locks in well before the other, as people do.
            p.a.set_lobby_pick(pick(round % 6, stage, true));
            p.run(u32::from(round) * 13);
            p.b.set_lobby_pick(pick((round + 1) % 6, stage, true));
            assert!(
                p.run_until_frame(120, 6000),
                "seed {seed} round {round}: the match starts"
            );
            assert_eq!(
                p.agreed(),
                (stage, (round + 1) % 6),
                "seed {seed} round {round}"
            );
            assert_eq!(p.desyncs(), 0, "seed {seed} round {round}");
            // The rematch, asked a little apart.
            p.b.request_rematch();
            p.run(25);
            p.a.request_rematch();
            let mut back = false;
            for _ in 0..3000 {
                p.tick();
                if p.last == (Status::Lobby, Status::Lobby) {
                    back = true;
                    break;
                }
            }
            assert!(back, "seed {seed} round {round}: both back in the lobby");
        }
    }
}

#[test]
fn leaving_the_lobby_tells_the_other_player() {
    let mut p = start(lossy(), 9, 1);
    p.run(40);
    p.b.leave();
    let mut left = false;
    for _ in 0..200 {
        p.tick();
        if let Status::Rejected { reason, .. } = &p.last.0 {
            assert_eq!(*reason, RejectReason::Left);
            left = true;
            break;
        }
    }
    assert!(left, "the host hears that the joiner left");
}

#[test]
fn a_different_build_is_found_in_the_lobby() {
    let base = Content::placeholder();
    let mut other = base.clone();
    other.rules.stale_moves ^= 1;
    let (la, lb, clock) = pair(clean(), 4);
    let mut a = Peer::host(la, &base, Setup::default()).with_lobby(pick(0, 0, false));
    let mut b = Peer::join(lb, &other, vec![]).with_lobby(pick(1, 0, false));
    let mut refused = false;
    for _ in 0..60 {
        let sa = a.update(&base, Input::default());
        b.update(&other, Input::default());
        clock.advance();
        if let Status::Rejected { reason, .. } = sa {
            assert_eq!(reason, RejectReason::ContentHash);
            refused = true;
            break;
        }
    }
    assert!(refused);
}
