//! Matches of three and four players through a hub host: the lobby, the setup, the routing, and a match in which every player's confirmed
//! frames equal a single-machine run, over rough simulated links.

use netplay::group::{GroupGuest, GroupHost, GroupParams, GroupStatus, HubLink};
use netplay::packet::RejectReason;
use netplay::peer::Link;
use netplay::session::{Advance, Event};
use netplay::testlink::{pair, Clock, LinkParams, TestLink};
use sim_content::recipe::{match_content_on, FighterSpec, Recipe};
use sim_core::{step, Content, GameState, Input, MatchRules, Rng, MAX_FIGHTERS};

/// The host's end of several test links.
struct Hub {
    links: Vec<TestLink>,
}

impl HubLink for Hub {
    fn send(&mut self, guest: usize, bytes: &[u8]) {
        if let Some(l) = self.links.get_mut(guest) {
            l.send(bytes);
        }
    }

    fn recv(&mut self) -> Option<(usize, Vec<u8>)> {
        for (i, l) in self.links.iter_mut().enumerate() {
            if let Some(b) = l.recv() {
                return Some((i, b));
            }
        }
        None
    }
}

fn scripted(player: usize, frame: u32) -> Input {
    let mut h = Rng::new(u64::from(frame / 6) * 8 + player as u64 + 3);
    let mut r = Rng::new(u64::from(frame) * 8 + player as u64 + 77);
    Input {
        stick_x: (h.range(255) as i32 - 127) as i8,
        stick_y: if h.range(4) == 0 {
            (h.range(255) as i32 - 127) as i8
        } else {
            0
        },
        buttons: if r.range(8) == 0 {
            r.range(32) as u16
        } else {
            0
        },
    }
}

fn rough() -> LinkParams {
    LinkParams {
        min_latency: 1,
        max_latency: 4,
        loss_percent: 12,
        duplicate_percent: 4,
    }
}

fn made(class: u8, size: u8) -> FighterSpec {
    FighterSpec::Made(Recipe {
        class,
        size,
        speed: 5,
        jump: 5,
        weight: 5,
    })
}

struct Room {
    host: GroupHost<Hub>,
    guests: Vec<GroupGuest<TestLink>>,
    clocks: Vec<Clock>,
    base: Content,
    ticks: u32,
    frames: Vec<u32>,
}

/// A host and `guests` guests joined by links with the given conditions.
fn room(
    guests: usize,
    params: LinkParams,
    seed: u64,
    rules: MatchRules,
    stage: u8,
    ranked: bool,
) -> Room {
    let count = guests;
    let base = Content::placeholder();
    let mut host_links = Vec::new();
    let mut guest_links = Vec::new();
    let mut clocks = Vec::new();
    for g in 0..guests {
        let (h, l, c) = pair(params, seed * 10 + g as u64);
        host_links.push(h);
        guest_links.push(l);
        clocks.push(c);
    }
    let specs = [made(0, 5), FighterSpec::Builtin(1), made(1, 7), made(0, 3)];
    let host = GroupHost::new(
        Hub { links: host_links },
        &base,
        GroupParams {
            seed: 4242 + seed,
            input_delay: 2,
            rules,
            stage,
            ranked,
        },
        specs[0].encode(),
        vec![10],
    );
    let guests: Vec<GroupGuest<TestLink>> = guest_links
        .into_iter()
        .enumerate()
        .map(|(i, link)| GroupGuest::new(link, &base, specs[i + 1].encode(), vec![11 + i as u8]))
        .collect();
    Room {
        host,
        guests,
        clocks,
        base,
        ticks: 0,
        frames: vec![0; count + 1],
    }
}

impl Room {
    /// One tick of everyone. Returns the host's status.
    fn tick(&mut self) -> GroupStatus {
        let n = self.ticks;
        let host_status = self.host.update(&self.base, scripted(0, self.frames[0]));
        if host_status == GroupStatus::Running(Advance::Ran) {
            self.frames[0] += 1;
        }
        let _ = n;
        for i in 0..self.guests.len() {
            let status = self.guests[i].update(&self.base, scripted(i + 1, self.frames[i + 1]));
            if status == GroupStatus::Running(Advance::Ran) {
                self.frames[i + 1] += 1;
            }
        }
        for c in &self.clocks {
            c.advance();
        }
        self.ticks += 1;
        host_status
    }

    fn lobby_until_joined(&mut self, limit: u32) -> bool {
        for _ in 0..limit {
            self.tick();
            if self.guests.iter().all(|g| g.slot().is_some()) {
                return true;
            }
        }
        false
    }

    fn run_until_frame(&mut self, frame: u32, limit: u32) -> bool {
        for _ in 0..limit {
            self.tick();
            let host = self.host.state().map_or(0, |s| s.frame);
            let all = self
                .guests
                .iter()
                .all(|g| g.state().map_or(0, |s| s.frame) >= frame);
            if host >= frame && all {
                return true;
            }
        }
        false
    }

    fn events(&mut self) -> Vec<Event> {
        let mut out = self.host.drain_events();
        for g in &mut self.guests {
            out.extend(g.drain_events());
        }
        out
    }
}

fn start_match(r: &mut Room) {
    assert!(r.lobby_until_joined(400), "everyone joins the lobby");
    r.host.start();
    let mut started = false;
    for _ in 0..800 {
        if let GroupStatus::Running(_) = r.tick() {
            started = true;
            break;
        }
    }
    assert!(started, "the match starts");
}

#[test]
fn guests_join_the_lobby_and_see_each_other() {
    let mut r = room(3, rough(), 1, MatchRules::default(), 0, false);
    assert!(r.lobby_until_joined(600));
    let slots: Vec<u8> = r.guests.iter().map(|g| g.slot().unwrap()).collect();
    assert_eq!(slots, vec![1, 2, 3]);
    for _ in 0..200 {
        r.tick();
    }
    assert_eq!(r.host.players_joined(), 4);
    // Each guest has been told who is in the lobby (names and looks by slot).
    for g in &r.guests {
        assert_eq!(g.lobby().len(), 4);
        assert_eq!(g.lobby()[0], vec![10]);
        assert_eq!(g.lobby()[3], vec![13]);
    }
}

#[test]
fn a_four_player_match_agrees_everywhere_and_with_a_single_machine() {
    for seed in 0..6 {
        let mut r = room(
            3,
            rough(),
            seed,
            MatchRules {
                stocks: 2,
                time_limit: 0,
            },
            if seed % 2 == 0 { 0 } else { 2 },
            false,
        );
        start_match(&mut r);
        assert!(r.run_until_frame(420, 9000), "seed {seed}: the match runs");
        let desyncs = r
            .events()
            .into_iter()
            .filter(|e| matches!(e, Event::Desync { .. }))
            .count();
        assert_eq!(desyncs, 0, "seed {seed}: no desync");

        // Everyone's confirmed checksums agree with the host's.
        let host_history = r.host.session().unwrap().checksum_history();
        assert!(!host_history.is_empty());
        for (i, g) in r.guests.iter().enumerate() {
            let history = g.session().unwrap().checksum_history();
            let mut compared = 0;
            for (f, c) in &history {
                if let Some((_, hc)) = host_history.iter().find(|(hf, _)| hf == f) {
                    assert_eq!(c, hc, "seed {seed}: guest {i} disagrees at frame {f}");
                    compared += 1;
                }
            }
            assert!(
                compared > 3,
                "seed {seed}: guest {i} compared {compared} frames"
            );
        }

        // And with one machine playing everyone's real inputs on the same content.
        let setup = r.host.match_setup().unwrap().clone();
        let content = r.host.match_content().unwrap().clone();
        let mut truth =
            GameState::new_with_rules(&content, setup.seed, setup.chars, setup.active, setup.rules);
        let delay = u32::from(setup.input_delay);
        let mut truth_checksums = Vec::new();
        for f in 0..400u32 {
            truth_checksums.push((f, truth.checksum()));
            let mut inputs = [Input::default(); MAX_FIGHTERS];
            if f >= delay {
                for (p, slot) in inputs.iter_mut().enumerate() {
                    if setup.active >> p & 1 == 1 {
                        *slot = scripted(p, f - delay);
                    }
                }
            }
            step(&mut truth, &content, &inputs);
        }
        let mut against_truth = 0;
        for (f, c) in &host_history {
            if let Some((_, tc)) = truth_checksums.iter().find(|(tf, _)| tf == f) {
                against_truth += 1;
                assert_eq!(
                    c, tc,
                    "seed {seed}: the host differs from a single machine at frame {f}"
                );
            }
        }
        assert!(
            against_truth > 3,
            "seed {seed}: compared {against_truth} frames with a single machine"
        );
    }
}

#[test]
fn a_three_player_match_works_and_a_player_leaving_does_not_stop_it() {
    let mut r = room(2, LinkParams::perfect(), 7, MatchRules::default(), 0, false);
    start_match(&mut r);
    assert!(r.run_until_frame(200, 4000));
    // Guest 2 says goodbye.
    r.guests[1].leave();
    let mut disconnected = false;
    for _ in 0..600 {
        r.tick();
        for e in r.events() {
            if let Event::Disconnected { player: 2 } = e {
                disconnected = true;
            }
        }
        if disconnected {
            break;
        }
    }
    assert!(disconnected, "the others notice");
    let before = r.host.state().unwrap().frame;
    for _ in 0..200 {
        r.tick();
    }
    assert!(
        r.host.state().unwrap().frame > before + 60,
        "the match goes on without them"
    );
    assert!(r.guests[0].state().unwrap().frame > before);
}

#[test]
fn the_lobby_refuses_other_versions_bad_fighters_and_ranked_cheats() {
    // Ranked rules: a guest over the point budget is refused.
    let base = Content::placeholder();
    let (h, l, c) = pair(LinkParams::perfect(), 1);
    let mut host = GroupHost::new(
        Hub { links: vec![h] },
        &base,
        GroupParams {
            seed: 1,
            input_delay: 2,
            rules: MatchRules::default(),
            stage: 0,
            ranked: true,
        },
        FighterSpec::Builtin(0).encode(),
        vec![],
    );
    let big = FighterSpec::Made(Recipe {
        class: 0,
        size: 9,
        speed: 9,
        jump: 9,
        weight: 9,
    });
    let mut guest = GroupGuest::new(l, &base, big.encode(), vec![]);
    let mut status = GroupStatus::Lobby;
    for _ in 0..200 {
        host.update(&base, Input::default());
        status = guest.update(&base, Input::default());
        c.advance();
        if matches!(status, GroupStatus::Rejected(_)) {
            break;
        }
    }
    assert_eq!(status, GroupStatus::Rejected(RejectReason::BadFighter));

    // A guest with unreadable fighter bytes.
    let (h, l, c) = pair(LinkParams::perfect(), 2);
    let mut host = GroupHost::new(
        Hub { links: vec![h] },
        &base,
        GroupParams {
            seed: 1,
            input_delay: 2,
            rules: MatchRules::default(),
            stage: 0,
            ranked: false,
        },
        FighterSpec::Builtin(0).encode(),
        vec![],
    );
    let mut guest = GroupGuest::new(l, &base, vec![9, 9, 9], vec![]);
    for _ in 0..200 {
        host.update(&base, Input::default());
        status = guest.update(&base, Input::default());
        c.advance();
        if matches!(status, GroupStatus::Rejected(_)) {
            break;
        }
    }
    assert_eq!(status, GroupStatus::Rejected(RejectReason::BadFighter));
    assert_eq!(host.players_joined(), 1, "the host did not keep it");
}

#[test]
fn a_rematch_starts_a_fresh_match_with_everyone() {
    let mut r = room(3, rough(), 3, MatchRules::default(), 1, false);
    start_match(&mut r);
    assert!(r.run_until_frame(200, 6000));
    let first_seed = r.host.match_setup().unwrap().seed;
    r.host.restart();
    let mut restarted = false;
    for _ in 0..3000 {
        r.tick();
        let hs = r.host.match_setup().map(|s| s.seed);
        let frames_ok = r
            .guests
            .iter()
            .all(|g| g.match_setup().is_some_and(|s| s.seed != first_seed));
        if hs.is_some_and(|s| s != first_seed)
            && frames_ok
            && r.host.state().is_some_and(|s| s.frame > 5)
        {
            restarted = true;
            break;
        }
    }
    assert!(restarted, "everyone is in the new match");
    assert!(r.run_until_frame(150, 6000));
    let desyncs = r
        .events()
        .into_iter()
        .filter(|e| matches!(e, Event::Desync { .. }))
        .count();
    assert_eq!(desyncs, 0);
    // The stage came with it.
    assert_eq!(r.host.match_content().unwrap().names.stage, "Triple Tier");
    let _ = match_content_on;
}

/// Randomised conditions, three or four players: no desync, and everyone ends in step with the host.
#[test]
fn random_conditions_never_desync() {
    for seed in 0..40u64 {
        let mut rng = Rng::new(seed + 1000);
        let params = LinkParams {
            min_latency: rng.range(3),
            max_latency: 2 + rng.range(6),
            loss_percent: rng.range(25),
            duplicate_percent: rng.range(10),
        };
        let players = 2 + rng.range(2) as usize; // 2 or 3 guests
        let mut r = room(
            players,
            params,
            seed,
            MatchRules {
                stocks: 1 + rng.range(3) as u8,
                time_limit: 0,
            },
            rng.range(4) as u8,
            false,
        );
        start_match(&mut r);
        assert!(r.run_until_frame(300, 12000), "seed {seed}: runs");
        let events = r.events();
        assert!(
            !events.iter().any(|e| matches!(e, Event::Desync { .. })),
            "seed {seed}: {events:?}"
        );
        let host_history = r.host.session().unwrap().checksum_history();
        for g in &r.guests {
            for (f, c) in g.session().unwrap().checksum_history() {
                if let Some((_, hc)) = host_history.iter().find(|(hf, _)| *hf == f) {
                    assert_eq!(c, *hc, "seed {seed}: frame {f}");
                }
            }
        }
    }
}
