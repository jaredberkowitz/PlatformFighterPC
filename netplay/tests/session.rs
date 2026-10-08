//! Networked sessions over a simulated lossy link: the two peers must agree on every confirmed frame, and agree with a
//! straight single-machine run of the same inputs.

use netplay::handshake::{BaseCounts, Handshake, Outcome};
use netplay::local_rollback::reference_checksums;
use netplay::packet::{Packet, RejectReason, Setup};
use netplay::peer::{Peer, Status};
use netplay::session::{Advance, Event, Session, SessionConfig, DISCONNECT_TIMEOUT};
use netplay::testlink::{pair, LinkParams, TestLink};
use sim_core::{Content, GameState, Input, Rng, MAX_FIGHTERS, SIM_VERSION};

const BASE: BaseCounts = BaseCounts {
    fighters: 2,
    weapons: 2,
};

fn setup() -> Setup {
    Setup {
        seed: 1234,
        chars: [0, 1, 0, 1],
        active: 0b0011,
        input_delay: 2,
        cosmetics: vec![9, 9],
        ..Setup::default()
    }
}

/// A deterministic input for `player` on `frame`, with plenty of changes so predictions are often wrong.
fn scripted(player: usize, frame: u32) -> Input {
    let mut rng = Rng::new(u64::from(frame) * 8 + player as u64 + 99);
    let held = frame / 7;
    let mut h = Rng::new(u64::from(held) * 8 + player as u64 + 5);
    Input {
        stick_x: (h.range(255) as i32 - 127) as i8,
        stick_y: if h.range(4) == 0 {
            (h.range(255) as i32 - 127) as i8
        } else {
            0
        },
        buttons: if rng.range(9) == 0 {
            (rng.range(32)) as u16
        } else {
            0
        },
    }
}

struct Match {
    a: Peer<TestLink>,
    b: Peer<TestLink>,
    clock: netplay::testlink::Clock,
    content: Content,
    /// Frames each peer has simulated (they also pick the scripted input for the next frame).
    na: u32,
    nb: u32,
}

fn start(params: LinkParams, seed: u64) -> Match {
    let content = Content::placeholder();
    let (la, lb, clock) = pair(params, seed);
    Match {
        a: Peer::host(la, &content, setup()),
        b: Peer::join(lb, &content, vec![7]),
        clock,
        content,
        na: 0,
        nb: 0,
    }
}

impl Match {
    /// One tick of both peers.
    fn tick(&mut self) {
        let ia = scripted(0, self.na);
        let ib = scripted(1, self.nb);
        if let Status::Running(Advance::Ran) = self.a.update(&self.content, ia) {
            self.na += 1;
        }
        if let Status::Running(Advance::Ran) = self.b.update(&self.content, ib) {
            self.nb += 1;
        }
        self.clock.advance();
    }

    /// Runs until both have simulated `frames` frames (or gives up).
    fn run(&mut self, frames: u32, max_ticks: u32) -> (u32, u32) {
        let mut t = 0;
        while (self.na < frames || self.nb < frames) && t < max_ticks {
            self.tick();
            t += 1;
        }
        (self.na, self.nb)
    }
}

/// The checksums a single machine gets from the true inputs of both players, with the same input delay.
fn reference(frames: u32) -> Vec<u64> {
    let content = Content::placeholder();
    let s = setup();
    let initial = GameState::new_with_active(&content, s.seed, s.chars, s.active);
    let delay = u32::from(s.input_delay);
    let inputs: Vec<[Input; MAX_FIGHTERS]> = (0..frames)
        .map(|f| {
            let mut i = [Input::default(); MAX_FIGHTERS];
            #[allow(clippy::needless_range_loop)]
            for p in 0..2 {
                i[p] = if f < delay {
                    Input::default()
                } else {
                    scripted(p, f - delay)
                };
            }
            i
        })
        .collect();
    reference_checksums(&content, &initial, &inputs)
}

fn assert_histories_match_reference(m: &mut Match, frames: u32) {
    let top =
        m.a.session()
            .unwrap()
            .frame()
            .max(m.b.session().unwrap().frame());
    let reference = reference(top.max(frames) + 5);
    for (name, peer) in [("host", &m.a), ("joiner", &m.b)] {
        let history = peer.session().unwrap().checksum_history();
        assert!(
            history.len() > 3,
            "{name} confirmed only {} checksums",
            history.len()
        );
        for (frame, checksum) in history {
            assert_eq!(
                checksum, reference[frame as usize],
                "{name}: confirmed state before frame {frame} differs from the single-machine run"
            );
        }
    }
}

// ---- Handshake -----------------------------------------------------------------------------------------------------

#[test]
fn matching_peers_connect_and_start_together() {
    let mut m = start(LinkParams::perfect(), 1);
    let (na, nb) = m.run(120, 400);
    assert!(na >= 120 && nb >= 120);
    assert_eq!(
        m.b.their_cosmetics(),
        &[9, 9],
        "the host's cosmetics arrived"
    );
    assert_eq!(
        m.a.their_cosmetics(),
        &[7],
        "the joiner's cosmetics arrived"
    );
    assert!(m.a.drain_events().is_empty() && m.b.drain_events().is_empty());
}

#[test]
fn a_different_sim_version_is_refused_by_both_sides() {
    let (mut h, mut j) = (
        Handshake::host(SIM_VERSION, 5, BASE, setup()),
        Handshake::join(SIM_VERSION + 1, 5, BASE, vec![], vec![]),
    );
    for _ in 0..40 {
        for p in j.tick() {
            h.handle_packet(&p);
        }
        for p in h.tick() {
            j.handle_packet(&p);
        }
    }
    assert!(matches!(
        h.outcome(),
        Outcome::Rejected {
            reason: RejectReason::SimVersion,
            ..
        }
    ));
    assert!(matches!(
        j.outcome(),
        Outcome::Rejected {
            reason: RejectReason::SimVersion,
            their_version,
            ..
        } if *their_version == SIM_VERSION
    ));
}

#[test]
fn different_content_is_refused() {
    let (mut h, mut j) = (
        Handshake::host(SIM_VERSION, 111, BASE, setup()),
        Handshake::join(SIM_VERSION, 222, BASE, vec![], vec![]),
    );
    for _ in 0..40 {
        for p in j.tick() {
            h.handle_packet(&p);
        }
        for p in h.tick() {
            j.handle_packet(&p);
        }
    }
    assert!(matches!(
        h.outcome(),
        Outcome::Rejected {
            reason: RejectReason::ContentHash,
            their_hash: 222,
            ..
        }
    ));
    assert!(matches!(
        j.outcome(),
        Outcome::Rejected {
            reason: RejectReason::ContentHash,
            their_hash: 111,
            ..
        }
    ));
}

#[test]
fn modified_content_is_caught_by_the_hash() {
    let a = Content::placeholder();
    let mut b = Content::placeholder();
    b.fighters[0].walk_speed += sim_core::Fx::from_raw(1);
    assert_ne!(a.hash(), b.hash());
    let (la, lb, clock) = pair(LinkParams::perfect(), 3);
    let mut host = Peer::host(la, &a, setup());
    let mut join = Peer::join(lb, &b, vec![]);
    let mut rejected = (false, false);
    for _ in 0..60 {
        if matches!(host.update(&a, Input::default()), Status::Rejected { .. }) {
            rejected.0 = true;
        }
        if matches!(join.update(&b, Input::default()), Status::Rejected { .. }) {
            rejected.1 = true;
        }
        clock.advance();
    }
    assert_eq!(rejected, (true, true));
    assert!(
        host.session().is_none() && join.session().is_none(),
        "no match starts"
    );
}

#[test]
fn the_handshake_survives_heavy_packet_loss() {
    for seed in 0..20 {
        let mut m = start(
            LinkParams {
                min_latency: 1,
                max_latency: 6,
                loss_percent: 50,
                duplicate_percent: 10,
            },
            seed,
        );
        let (na, nb) = m.run(60, 4000);
        assert!(na >= 60 && nb >= 60, "seed {seed}: {na}/{nb}");
    }
}

// ---- Sessions ------------------------------------------------------------------------------------------------------

#[test]
fn a_perfect_link_matches_the_single_machine_run() {
    let mut m = start(LinkParams::perfect(), 1);
    m.run(300, 800);
    assert_histories_match_reference(&mut m, 300);
}

#[test]
fn latency_loss_duplication_and_reordering_never_change_the_confirmed_states() {
    for seed in 0..30u64 {
        let mut m = start(
            LinkParams {
                min_latency: 0,
                max_latency: 9,
                loss_percent: 20,
                duplicate_percent: 10,
            },
            seed,
        );
        m.run(400, 6000);
        assert_histories_match_reference(&mut m, 400);
        let events: Vec<Event> =
            m.a.drain_events()
                .into_iter()
                .chain(m.b.drain_events())
                .collect();
        assert!(events.is_empty(), "seed {seed}: {events:?}");
        let rollbacks = m.a.stats().rollbacks + m.b.stats().rollbacks;
        assert!(
            rollbacks > 0,
            "seed {seed}: the test never exercised a rollback"
        );
    }
}

#[test]
fn a_long_lag_spike_stalls_the_sim_instead_of_running_away() {
    let mut m = start(LinkParams::perfect(), 5);
    m.run(60, 400);
    let (before_a, _) = (m.a.session().unwrap().frame(), 0);
    // Cut the link: nothing gets through for a while.
    m.clock.set_params(LinkParams {
        min_latency: 0,
        max_latency: 0,
        loss_percent: 100,
        duplicate_percent: 0,
    });
    for _ in 0..200 {
        m.tick();
    }
    let ahead = m.a.session().unwrap().frame() - before_a;
    // At most the prediction window (8) plus the input delay (2) plus what was already in flight.
    assert!(
        ahead <= 14,
        "ran {ahead} frames past the last confirmed input"
    );
    assert!(m.a.stats().stalls > 100);
    // The link comes back: both catch up and still agree.
    m.clock.set_params(LinkParams::perfect());
    for _ in 0..200 {
        m.tick();
    }
    let (fa, fb) = (
        m.a.session().unwrap().frame(),
        m.b.session().unwrap().frame(),
    );
    assert!(fa > before_a + 30 && fb > before_a + 30);
    assert_histories_match_reference(&mut m, 200);
}

#[test]
fn a_corrupted_state_is_reported_as_a_desync_with_its_frame() {
    // Two sessions wired directly, with one simulating from a different initial state.
    let content = Content::placeholder();
    let cfg = |local| SessionConfig {
        checksum_interval: 10,
        ..SessionConfig::two_player(local)
    };
    let s = setup();
    let good = GameState::new_with_active(&content, s.seed, s.chars, s.active);
    let mut bad = good;
    bad.fighters[0].percent = sim_core::Fx::from_int(1);
    let (mut a, mut b) = (Session::new(cfg(0), good), Session::new(cfg(1), bad));
    let mut desync = None;
    for f in 0..200u32 {
        a.advance(&content, scripted(0, f));
        b.advance(&content, scripted(1, f));
        for p in a.drain_outgoing() {
            b.handle_packet(&p);
        }
        for p in b.drain_outgoing() {
            a.handle_packet(&p);
        }
        for e in a.drain_events().into_iter().chain(b.drain_events()) {
            if let Event::Desync {
                frame,
                local,
                remote,
            } = e
            {
                assert_ne!(local, remote);
                desync.get_or_insert(frame);
            }
        }
    }
    let frame = desync.expect("the desync must be detected");
    assert!(frame <= 40, "detected only at frame {frame}");
    assert_eq!(frame % 10, 0, "reported against a checksum frame");
}

#[test]
fn a_peer_that_goes_silent_is_dropped_and_the_match_goes_on_for_the_other() {
    let content = Content::placeholder();
    let s = setup();
    let initial = GameState::new_with_active(&content, s.seed, s.chars, s.active);
    let mut a = Session::new(SessionConfig::two_player(0), initial);
    let mut stalled = 0;
    let mut dropped = false;
    for f in 0..(DISCONNECT_TIMEOUT + 300) {
        if a.advance(&content, scripted(0, f)) == Advance::Stalled {
            stalled += 1;
        }
        a.drain_outgoing();
        if a.drain_events()
            .contains(&Event::Disconnected { player: 1 })
        {
            dropped = true;
        }
    }
    assert!(dropped, "the silent peer is dropped after the timeout");
    assert!(stalled > 100, "the match waited for it first");
    assert!(a.frame() > 100, "and then carried on alone");
    assert!(!a.is_connected(1));
}

#[test]
fn a_goodbye_ends_the_connection_at_once() {
    let content = Content::placeholder();
    let s = setup();
    let initial = GameState::new_with_active(&content, s.seed, s.chars, s.active);
    let mut a = Session::new(SessionConfig::two_player(0), initial);
    a.handle_packet(&Packet::Disconnect.encode());
    assert!(a
        .drain_events()
        .contains(&Event::Disconnected { player: 1 }));
}

#[test]
fn garbage_and_stale_packets_are_ignored() {
    let content = Content::placeholder();
    let s = setup();
    let initial = GameState::new_with_active(&content, s.seed, s.chars, s.active);
    let mut a = Session::new(SessionConfig::two_player(0), initial);
    let mut rng = Rng::new(3);
    for f in 0..100u32 {
        let junk: Vec<u8> = (0..rng.range(60)).map(|_| rng.range(256) as u8).collect();
        a.handle_packet(&junk);
        // A packet claiming to be from ourselves, and one from the future.
        a.handle_packet(
            &Packet::Inputs {
                player: 0,
                ack: 0,
                start: 0,
                inputs: vec![Input::default(); 3],
            }
            .encode(),
        );
        a.handle_packet(
            &Packet::Inputs {
                player: 1,
                ack: 0,
                start: 1_000_000,
                inputs: vec![Input::default(); 3],
            }
            .encode(),
        );
        a.advance(&content, scripted(0, f));
    }
    assert!(a.drain_events().is_empty());
}
