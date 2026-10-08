//! A spectator over real sockets on localhost: the host streams its match record, and the spectator ends on the same state.

use netplay::peer::Link;
use netplay::replay::MatchRecord;
use netplay::spectate::{SpectatorClient, SpectatorServer};
use sim_core::fuzz::random_inputs;
use sim_core::{Content, MatchRules, Rng};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread::sleep;
use std::time::{Duration, Instant};
use transport::{SpectatorSocket, UdpLink};

fn local() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}

#[test]
fn a_spectator_watches_a_match_over_real_udp() {
    let base = Content::placeholder();
    let rules = MatchRules {
        stocks: 3,
        time_limit: 0,
    };
    let mut full = MatchRecord::begin(&base, &base, 77, [0, 1, 0, 1], 0b0011, rules, 0, vec![]);
    full.inputs = random_inputs(&mut Rng::new(77), 500);

    let mut socket = SpectatorSocket::bind(local(), 8).unwrap();
    let host_addr = socket.local_addr().unwrap();
    let mut server = SpectatorServer::new(&base);
    let mut link = UdpLink::bind(local(), Some(host_addr)).unwrap();
    let mut client = SpectatorClient::new(&base);

    let mut live = full.clone();
    live.inputs.clear();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(15) {
        // The match grows by a frame a tick.
        let to = (live.inputs.len() + 2).min(full.inputs.len());
        live.inputs = full.inputs[..to].to_vec();
        server.sync(&live);
        while let Some((id, bytes)) = socket.recv() {
            for reply in server.handle(id, &bytes) {
                socket.send(id, &reply);
            }
        }
        for (id, bytes) in server.tick() {
            socket.send(id, &bytes);
        }
        while let Some(bytes) = link.recv() {
            client.handle(&bytes);
        }
        for m in client.tick() {
            link.send(&m);
        }
        client.advance(4);
        if client.frames_received() as usize == full.inputs.len() && client.behind() == 0 {
            break;
        }
        sleep(Duration::from_millis(2));
    }
    assert_eq!(client.frames_received() as usize, full.inputs.len());
    let (content, mut state) = full.rebuild(&base).unwrap();
    for f in &full.inputs {
        sim_core::step(&mut state, &content, f);
    }
    assert_eq!(client.state().unwrap().checksum(), state.checksum());
    assert_eq!(server.spectator_count(), 1);
}
