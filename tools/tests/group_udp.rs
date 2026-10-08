//! A four-player match over real sockets on localhost: a host and three guests agree on every confirmed frame.

use netplay::group::{GroupGuest, GroupHost, GroupParams, GroupStatus};
use netplay::session::{Advance, Event};
use sim_content::recipe::FighterSpec;
use sim_core::{Content, Input, MatchRules, Rng};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread::sleep;
use std::time::{Duration, Instant};
use transport::{SpectatorSocket, UdpLink};

fn local() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}

fn scripted(player: usize, frame: u32) -> Input {
    let mut h = Rng::new(u64::from(frame / 7) * 8 + player as u64 + 1);
    let mut r = Rng::new(u64::from(frame) * 8 + player as u64 + 5);
    Input {
        stick_x: (h.range(255) as i32 - 127) as i8,
        stick_y: 0,
        buttons: if r.range(8) == 0 {
            r.range(32) as u16
        } else {
            0
        },
    }
}

#[test]
fn four_players_play_over_real_udp_and_stay_in_sync() {
    let base = Content::placeholder();
    let socket = SpectatorSocket::bind(local(), 3).unwrap();
    let addr = socket.local_addr().unwrap();
    let mut host = GroupHost::new(
        socket,
        &base,
        GroupParams {
            seed: 31,
            input_delay: 2,
            rules: MatchRules {
                stocks: 2,
                time_limit: 0,
            },
            stage: 3,
            ranked: false,
        },
        FighterSpec::Builtin(0).encode(),
        vec![1],
    );
    let mut guests: Vec<GroupGuest<UdpLink>> = (0..3)
        .map(|i| {
            let link = UdpLink::bind(local(), Some(addr)).unwrap();
            GroupGuest::new(
                link,
                &base,
                FighterSpec::Builtin(i % 2).encode(),
                vec![2 + i],
            )
        })
        .collect();

    let mut frames = [0u32; 4];
    let mut started = false;
    let begin = Instant::now();
    while begin.elapsed() < Duration::from_secs(25) {
        if !started && guests.iter().all(|g| g.slot().is_some()) {
            host.start();
            started = true;
        }
        if let GroupStatus::Running(Advance::Ran) = host.update(&base, scripted(0, frames[0])) {
            frames[0] += 1;
        }
        for (i, g) in guests.iter_mut().enumerate() {
            if let GroupStatus::Running(Advance::Ran) =
                g.update(&base, scripted(i + 1, frames[i + 1]))
            {
                frames[i + 1] += 1;
            }
        }
        if frames.iter().all(|f| *f >= 300) {
            break;
        }
        sleep(Duration::from_micros(800));
    }
    assert!(
        frames.iter().all(|f| *f >= 300),
        "everyone played 300 frames: {frames:?}"
    );
    let mut events = host.drain_events();
    for g in &mut guests {
        events.extend(g.drain_events());
    }
    assert!(
        !events.iter().any(|e| matches!(e, Event::Desync { .. })),
        "{events:?}"
    );
    let host_history = host.session().unwrap().checksum_history();
    let mut compared = 0;
    for g in &guests {
        for (f, c) in g.session().unwrap().checksum_history() {
            if let Some((_, hc)) = host_history.iter().find(|(hf, _)| *hf == f) {
                assert_eq!(c, *hc, "frame {f}");
                compared += 1;
            }
        }
    }
    assert!(compared > 10, "compared {compared} confirmed frames");
}
