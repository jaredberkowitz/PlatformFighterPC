//! Online play with made fighters: each player brings a fighter (a built-in one or one made from a recipe), both sides build
//! the same match content from the two, and the match is as exact as ever: every confirmed frame equals a single-machine
//! run on that content. Also: ranked rules, and every way a fighter spec can be wrong.

use netplay::handshake::{BaseCounts, Handshake, Outcome};
use netplay::local_rollback::reference_checksums;
use netplay::packet::{Packet, RejectReason, Setup};
use netplay::peer::{Peer, Status};
use netplay::session::Advance;
use netplay::testlink::{pair, Clock, LinkParams, TestLink};
use sim_content::recipe::{match_content, FighterSpec, Recipe};
use sim_core::{Content, GameState, Input, Rng, MAX_FIGHTERS, SIM_VERSION};

const BASE: BaseCounts = BaseCounts {
    fighters: 2,
    weapons: 2,
};

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
            rng.range(32) as u16
        } else {
            0
        },
    }
}

fn setup(host: &FighterSpec, ranked: bool) -> Setup {
    Setup {
        seed: 777,
        chars: [0, 1, 0, 1],
        active: 0b0011,
        input_delay: 2,
        cosmetics: vec![1],
        fighter: host.encode(),
        ranked,
    }
}

struct Match {
    a: Peer<TestLink>,
    b: Peer<TestLink>,
    clock: Clock,
    base: Content,
    na: u32,
    nb: u32,
}

fn start(
    host: FighterSpec,
    joiner: FighterSpec,
    ranked: bool,
    params: LinkParams,
    seed: u64,
) -> Match {
    let base = Content::placeholder();
    let (la, lb, clock) = pair(params, seed);
    Match {
        a: Peer::host(la, &base, setup(&host, ranked)),
        b: Peer::join_with_fighter(lb, &base, vec![2], joiner.encode()),
        clock,
        base,
        na: 0,
        nb: 0,
    }
}

impl Match {
    fn run(&mut self, frames: u32, max_ticks: u32) -> (Status, Status) {
        let (mut sa, mut sb) = (Status::Connecting, Status::Connecting);
        for _ in 0..max_ticks {
            if self.na >= frames && self.nb >= frames {
                break;
            }
            sa = self.a.update(&self.base, scripted(0, self.na));
            sb = self.b.update(&self.base, scripted(1, self.nb));
            if sa == Status::Running(Advance::Ran) {
                self.na += 1;
            }
            if sb == Status::Running(Advance::Ran) {
                self.nb += 1;
            }
            self.clock.advance();
            if matches!(sa, Status::Rejected { .. }) && matches!(sb, Status::Rejected { .. }) {
                break;
            }
        }
        (sa, sb)
    }
}

/// A rough link: latency with a spread (so packets reorder), 15% loss and some duplicates.
fn lossy() -> LinkParams {
    LinkParams {
        min_latency: 1,
        max_latency: 5,
        loss_percent: 15,
        duplicate_percent: 5,
    }
}

fn made(class: u8, size: u8, speed: u8, jump: u8, weight: u8) -> FighterSpec {
    FighterSpec::Made(Recipe {
        class,
        size,
        speed,
        jump,
        weight,
    })
}

/// The checksums of a single machine playing the true inputs of both players on the match content.
fn reference(base: &Content, host: FighterSpec, joiner: FighterSpec, frames: u32) -> Vec<u64> {
    let (content, chars) = match_content(base, &[host, joiner], false).unwrap();
    // The other two slots keep the settings' characters (they are inactive in a two-player match).
    let mut ids = [0, 1, 0, 1];
    ids[0] = chars[0];
    ids[1] = chars[1];
    let initial = GameState::new_with_active(&content, 777, ids, 0b0011);
    let delay = 2;
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

fn check_exact(m: &Match, host: FighterSpec, joiner: FighterSpec) {
    let top =
        m.a.session()
            .unwrap()
            .frame()
            .max(m.b.session().unwrap().frame());
    let truth = reference(&m.base, host, joiner, top + 5);
    for (name, peer) in [("host", &m.a), ("joiner", &m.b)] {
        let history = peer.session().unwrap().checksum_history();
        assert!(history.len() > 3, "{name} confirmed only {}", history.len());
        for (frame, sum) in history {
            assert_eq!(
                sum, truth[frame as usize],
                "{name}, confirmed state before frame {frame}"
            );
        }
    }
}

#[test]
fn two_made_fighters_play_a_match_that_matches_a_single_machine_run_frame_for_frame() {
    let (host, joiner) = (made(0, 9, 3, 4, 4), made(1, 1, 7, 6, 6));
    for (params, seed) in [(LinkParams::perfect(), 1), (lossy(), 2), (lossy(), 3)] {
        let mut m = start(host, joiner, false, params, seed);
        m.run(300, 3000);
        assert!(m.na >= 300 && m.nb >= 300, "ran {} and {}", m.na, m.nb);
        check_exact(&m, host, joiner);
        // Both sides run on the same content, with both made fighters in it, and they really are different fighters.
        let (ca, cb) = (m.a.match_content().unwrap(), m.b.match_content().unwrap());
        assert_eq!(ca.hash(), cb.hash());
        assert_eq!(ca.fighters.len(), 4);
        assert_ne!(ca.fighters[2], ca.fighters[3]);
        assert_eq!(m.a.state().unwrap().fighters[0].char_id, 2);
        assert_eq!(m.a.state().unwrap().fighters[1].char_id, 3);
    }
}

#[test]
fn a_built_in_fighter_against_a_made_one_works_both_ways() {
    for (host, joiner) in [
        (FighterSpec::Builtin(1), made(0, 8, 2, 5, 5)),
        (made(1, 2, 8, 5, 5), FighterSpec::Builtin(0)),
        (FighterSpec::Builtin(0), FighterSpec::Builtin(1)),
    ] {
        let mut m = start(host, joiner, false, lossy(), 5);
        m.run(240, 3000);
        assert!(m.na >= 240 && m.nb >= 240);
        check_exact(&m, host, joiner);
    }
}

#[test]
fn ranked_rules_refuse_a_fighter_over_the_point_budget_whichever_side_brings_it() {
    let legal = made(0, 4, 6, 5, 5);
    let greedy = made(0, 9, 9, 9, 9);
    for (host, joiner) in [(greedy, legal), (legal, greedy), (greedy, greedy)] {
        let mut m = start(host, joiner, true, LinkParams::perfect(), 6);
        let (sa, sb) = m.run(60, 400);
        for s in [sa, sb] {
            assert!(
                matches!(
                    s,
                    Status::Rejected {
                        reason: RejectReason::BadFighter,
                        ..
                    }
                ),
                "{s:?}"
            );
        }
    }
    // Two legal fighters play fine under ranked rules, and the same greedy ones are fine in a casual match.
    let mut m = start(legal, made(1, 5, 5, 5, 5), true, LinkParams::perfect(), 7);
    m.run(120, 1000);
    assert!(m.na >= 120 && m.nb >= 120);
    let mut m = start(greedy, greedy, false, LinkParams::perfect(), 8);
    m.run(120, 1000);
    assert!(m.na >= 120 && m.nb >= 120);
}

#[test]
fn a_spec_that_is_wrong_is_refused_by_both_sides() {
    let fine = made(0, 5, 5, 5, 5);
    let cases: Vec<(&str, Vec<u8>, Vec<u8>)> = vec![
        ("garbage from the joiner", fine.encode(), vec![9, 9, 9]),
        (
            "a stat out of range",
            fine.encode(),
            vec![1, 0, 10, 5, 5, 5],
        ),
        ("an unknown class", fine.encode(), vec![1, 5, 5, 5, 5, 5]),
        ("an unknown built-in fighter", fine.encode(), vec![0, 77]),
        ("the host's is garbage", vec![1, 2, 3], fine.encode()),
        ("only the host brings one", fine.encode(), vec![]),
        ("only the joiner brings one", vec![], fine.encode()),
    ];
    for (what, host_bytes, join_bytes) in cases {
        let base = Content::placeholder();
        let (la, lb, clock) = pair(LinkParams::perfect(), 9);
        let mut setup = setup(&fine, false);
        setup.fighter = host_bytes;
        let mut a = Peer::host(la, &base, setup);
        let mut b = Peer::join_with_fighter(lb, &base, vec![], join_bytes);
        let mut last = (Status::Connecting, Status::Connecting);
        for _ in 0..60 {
            last = (
                a.update(&base, Input::default()),
                b.update(&base, Input::default()),
            );
            clock.advance();
        }
        assert!(
            matches!(
                last.0,
                Status::Rejected {
                    reason: RejectReason::BadFighter,
                    ..
                }
            ) && matches!(
                last.1,
                Status::Rejected {
                    reason: RejectReason::BadFighter,
                    ..
                }
            ),
            "{what}: {last:?}"
        );
    }
}

#[test]
fn a_joiner_will_not_accept_settings_that_name_the_wrong_fighters() {
    // A host that sends fighter numbers that do not follow from the two specs is refused.
    let base = Content::placeholder();
    let mut j = Handshake::join(
        SIM_VERSION,
        base.hash(),
        BASE,
        vec![],
        made(0, 5, 5, 5, 5).encode(),
    );
    let lying = Setup {
        seed: 1,
        chars: [3, 9, 0, 0], // the specs make [2, 3]
        active: 0b0011,
        input_delay: 2,
        cosmetics: vec![],
        fighter: made(1, 5, 5, 5, 5).encode(),
        ranked: false,
    };
    j.tick();
    j.handle_packet(
        &Packet::Setup {
            sim_version: SIM_VERSION,
            content_hash: base.hash(),
            setup: lying,
        }
        .encode(),
    );
    assert!(matches!(
        j.outcome(),
        Outcome::Rejected {
            reason: RejectReason::BadFighter,
            ..
        }
    ));
}

#[test]
fn matches_without_fighter_specs_are_unchanged() {
    // The old path: no specs, the characters in the settings are used on the base content.
    let base = Content::placeholder();
    let (la, lb, clock) = pair(LinkParams::perfect(), 4);
    let mut setup = setup(&FighterSpec::Builtin(0), false);
    setup.fighter = vec![];
    let mut a = Peer::host(la, &base, setup);
    let mut b = Peer::join(lb, &base, vec![]);
    for _ in 0..60 {
        a.update(&base, Input::default());
        b.update(&base, Input::default());
        clock.advance();
    }
    assert!(a.session().is_some() && b.session().is_some());
    assert!(a.match_content().is_none() && b.match_content().is_none());
}
