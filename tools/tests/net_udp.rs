//! Real sockets: two peers on localhost, directly, through the relay and by connection code (hole punching with a fake STUN
//! server), play a match and must agree on every confirmed frame.

use netplay::local_rollback::reference_checksums;
use netplay::packet::Setup;
use netplay::peer::{Link, Peer, Status};
use netplay::session::{Advance, Event};
use sim_core::fuzz::random_inputs;
use sim_core::{Content, GameState, Input, Rng, MAX_FIGHTERS};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use transport::{Relay, RelayLink, UdpLink};

const FRAMES: u32 = 240;
/// Both peers play this many frames so each has the other's inputs for all the frames we compare.
const TOTAL: u32 = FRAMES + 40;

fn local() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}

fn setup() -> Setup {
    Setup {
        seed: 55,
        chars: [0, 1, 0, 1],
        active: 0b0011,
        input_delay: 2,
        cosmetics: vec![1, 2, 3],
        ..Setup::default()
    }
}

struct Outcome {
    history: Vec<(u32, u64)>,
    events: Vec<Event>,
    cosmetics: Vec<u8>,
}

fn play<L: Link>(mut peer: Peer<L>, inputs: Vec<[Input; MAX_FIGHTERS]>) -> Outcome {
    let content = Content::placeholder();
    let local = peer.local_player();
    let mut ran = 0u32;
    let mut events = Vec::new();
    let started = Instant::now();
    while ran < TOTAL && started.elapsed() < Duration::from_secs(20) {
        let input = inputs
            .get(ran as usize)
            .map_or_else(Input::default, |i| i[local]);
        if let Status::Running(Advance::Ran) = peer.update(&content, input) {
            ran += 1;
        }
        events.extend(peer.drain_events());
        sleep(Duration::from_micros(500));
    }
    assert!(ran >= TOTAL, "only simulated {ran} frames");
    Outcome {
        history: peer.session().unwrap().checksum_history(),
        events,
        cosmetics: peer.their_cosmetics().to_vec(),
    }
}

fn truth(inputs: &[[Input; MAX_FIGHTERS]]) -> Vec<u64> {
    truth_on(&Content::placeholder(), setup().chars, inputs)
}

fn truth_on(
    content: &Content,
    chars: [u8; MAX_FIGHTERS],
    inputs: &[[Input; MAX_FIGHTERS]],
) -> Vec<u64> {
    let s = setup();
    let initial = GameState::new_with_active(content, s.seed, chars, s.active);
    let delay = u32::from(s.input_delay) as usize;
    let seq: Vec<[Input; MAX_FIGHTERS]> = (0..TOTAL as usize + 20)
        .map(|f| {
            let mut i = [Input::default(); MAX_FIGHTERS];
            if f >= delay {
                if let Some(src) = inputs.get(f - delay) {
                    i[0] = src[0];
                    i[1] = src[1];
                }
            }
            i
        })
        .collect();
    reference_checksums(content, &initial, &seq)
}

fn check(host: &Outcome, join: &Outcome, inputs: &[[Input; MAX_FIGHTERS]]) {
    assert!(host.events.is_empty(), "host events: {:?}", host.events);
    assert!(join.events.is_empty(), "joiner events: {:?}", join.events);
    assert_eq!(
        host.cosmetics,
        Vec::<u8>::new(),
        "the joiner sent no cosmetics"
    );
    assert_eq!(
        join.cosmetics,
        vec![1, 2, 3],
        "the host's cosmetics arrived"
    );
    let truth = truth(inputs);
    for (name, o) in [("host", host), ("joiner", join)] {
        assert!(
            o.history.len() >= 5,
            "{name} confirmed only {}",
            o.history.len()
        );
        for (frame, sum) in &o.history {
            assert_eq!(*sum, truth[*frame as usize], "{name} at frame {frame}");
        }
    }
}

#[test]
fn two_peers_over_real_udp_stay_in_sync() {
    let inputs = random_inputs(&mut Rng::new(7), TOTAL as usize + 100);
    let host_link = UdpLink::bind(local(), None).unwrap();
    let host_addr = host_link.local_addr().unwrap();
    let join_link = UdpLink::bind(local(), Some(host_addr)).unwrap();
    let content = Content::placeholder();
    let host = Peer::host(host_link, &content, setup());
    let join = Peer::join(join_link, &content, Vec::new());

    let (i1, i2) = (inputs.clone(), inputs.clone());
    let (tx, rx) = mpsc::channel();
    let t1 = thread::spawn(move || play(host, i1));
    let tx2 = tx.clone();
    let t2 = thread::spawn(move || tx2.send(play(join, i2)).unwrap());
    let host_outcome = t1.join().unwrap();
    t2.join().unwrap();
    let join_outcome = rx.recv().unwrap();
    check(&host_outcome, &join_outcome, &inputs);
}

#[test]
fn two_peers_through_the_relay_stay_in_sync() {
    let inputs = random_inputs(&mut Rng::new(8), TOTAL as usize + 100);
    let mut relay = Relay::bind(local()).unwrap();
    let relay_addr = relay.local_addr().unwrap();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop2 = stop.clone();
    let relay_thread = thread::spawn(move || {
        while !stop2.load(std::sync::atomic::Ordering::Relaxed) {
            relay.pump();
            sleep(Duration::from_micros(200));
        }
        relay.forwarded()
    });

    let content = Content::placeholder();
    let mut a = RelayLink::connect(local(), relay_addr, 4242).unwrap();
    let mut b = RelayLink::connect(local(), relay_addr, 4242).unwrap();
    a.announce();
    b.announce();
    let host = Peer::host(a, &content, setup());
    let join = Peer::join(b, &content, Vec::new());

    let (i1, i2) = (inputs.clone(), inputs.clone());
    let (tx, rx) = mpsc::channel();
    let t1 = thread::spawn(move || play(host, i1));
    let t2 = thread::spawn(move || tx.send(play(join, i2)).unwrap());
    let host_outcome = t1.join().unwrap();
    t2.join().unwrap();
    let join_outcome = rx.recv().unwrap();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let forwarded = relay_thread.join().unwrap();
    assert!(
        forwarded > 400,
        "the relay carried the match ({forwarded} datagrams)"
    );
    check(&host_outcome, &join_outcome, &inputs);
}

#[test]
fn two_made_fighters_over_real_udp_stay_in_sync() {
    use sim_content::recipe::{match_content, FighterSpec, Recipe};
    let big = FighterSpec::Made(Recipe {
        class: 0,
        size: 9,
        speed: 2,
        jump: 3,
        weight: 8,
    });
    let small = FighterSpec::Made(Recipe {
        class: 1,
        size: 1,
        speed: 9,
        jump: 8,
        weight: 2,
    });
    let inputs = random_inputs(&mut Rng::new(21), TOTAL as usize + 100);
    let content = Content::placeholder();
    let host_link = UdpLink::bind(local(), None).unwrap();
    let host_addr = host_link.local_addr().unwrap();
    let join_link = UdpLink::bind(local(), Some(host_addr)).unwrap();
    let mut host_setup = setup();
    host_setup.fighter = big.encode();
    let host = Peer::host(host_link, &content, host_setup);
    let join = Peer::join_with_fighter(join_link, &content, Vec::new(), small.encode());

    let (i1, i2) = (inputs.clone(), inputs.clone());
    let t1 = thread::spawn(move || play(host, i1));
    let t2 = thread::spawn(move || play(join, i2));
    let (a, b) = (t1.join().unwrap(), t2.join().unwrap());

    let (match_c, chars) = match_content(&content, &[big, small], false).unwrap();
    let mut ids = setup().chars;
    ids[0] = chars[0];
    ids[1] = chars[1];
    let truth = truth_on(&match_c, ids, &inputs);
    for (name, o) in [("host", &a), ("joiner", &b)] {
        assert!(o.events.is_empty(), "{name}: {:?}", o.events);
        assert!(o.history.len() >= 5);
        for (frame, sum) in &o.history {
            assert_eq!(*sum, truth[*frame as usize], "{name} at frame {frame}");
        }
    }
}

/// A fake STUN server in a thread: answers each Binding Request with the address it came from, until `stop` is set.
fn fake_stun(
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> (SocketAddr, thread::JoinHandle<u32>) {
    let socket = std::net::UdpSocket::bind(local()).unwrap();
    socket.set_nonblocking(true).unwrap();
    let addr = socket.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let mut answered = 0;
        let mut buf = [0u8; 1500];
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            while let Ok((n, from)) = socket.recv_from(&mut buf) {
                if let (Some(id), SocketAddr::V4(from)) =
                    (transport::stun::transaction_id(&buf[..n]), from)
                {
                    let _ = socket.send_to(&transport::stun::binding_response(&id, from), from);
                    answered += 1;
                }
            }
            sleep(Duration::from_millis(1));
        }
        answered
    });
    (addr, handle)
}

#[test]
fn two_peers_connecting_by_code_stay_in_sync() {
    use transport::{CodeOptions, CodeSocket, CodeState, ConnectionCode};
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (stun_addr, stun_thread) = fake_stun(stop.clone());
    let opts = CodeOptions {
        stun_servers: vec![stun_addr.to_string()],
        lan_override: Some(Ipv4Addr::LOCALHOST),
        ..CodeOptions::default()
    };
    let mut a = CodeSocket::bind(0, opts.clone()).unwrap();
    let mut b = CodeSocket::bind(0, opts).unwrap();
    let started = Instant::now();
    while (a.state() != CodeState::Ready || b.state() != CodeState::Ready)
        && started.elapsed() < Duration::from_secs(10)
    {
        a.poll();
        b.poll();
        sleep(Duration::from_millis(2));
    }
    // The codes travel as text, as players would paste them.
    let code_a = a.my_code().unwrap().encode();
    let code_b = b.my_code().unwrap().encode();
    let theirs_for_a = ConnectionCode::decode(&code_b).unwrap();
    let theirs_for_b = ConnectionCode::decode(&code_a).unwrap();
    assert!(
        theirs_for_b.public.is_some(),
        "STUN found an address: {code_a}"
    );
    let a_hosts = a.hosts_against(&theirs_for_a);
    assert_ne!(a_hosts, b.hosts_against(&theirs_for_b), "exactly one hosts");
    a.connect(theirs_for_a).unwrap();
    b.connect(theirs_for_b).unwrap();
    while !(matches!(a.state(), CodeState::Connected(_))
        && matches!(b.state(), CodeState::Connected(_)))
    {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "punching timed out: {:?} {:?}",
            a.state(),
            b.state()
        );
        a.poll();
        b.poll();
        sleep(Duration::from_millis(2));
    }
    // A stranger keeps throwing junk and made-up probes at both while they play.
    let targets = [
        a.local_addr().unwrap().port(),
        b.local_addr().unwrap().port(),
    ];
    let noisy = stop.clone();
    let noise = thread::spawn(move || {
        let s = std::net::UdpSocket::bind(local()).unwrap();
        let mut n = 0u8;
        while !noisy.load(std::sync::atomic::Ordering::Relaxed) {
            for p in targets {
                let to = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), p);
                let _ = s.send_to(&[0x46, 0x50, n, 1, 2, 3, 4, 5], to);
                let _ = s.send_to(&[b'H', b'P', 0, n, 0, n, 0], to);
            }
            n = n.wrapping_add(1);
            sleep(Duration::from_millis(3));
        }
    });
    let (la, lb) = (a.into_link().unwrap(), b.into_link().unwrap());
    let (host_link, join_link) = if a_hosts { (la, lb) } else { (lb, la) };
    let inputs = random_inputs(&mut Rng::new(9), TOTAL as usize + 100);
    let content = Content::placeholder();
    let host = Peer::host(host_link, &content, setup());
    let join = Peer::join(join_link, &content, Vec::new());
    let (i1, i2) = (inputs.clone(), inputs.clone());
    let (tx, rx) = mpsc::channel();
    let t1 = thread::spawn(move || play(host, i1));
    let t2 = thread::spawn(move || tx.send(play(join, i2)).unwrap());
    let host_outcome = t1.join().unwrap();
    t2.join().unwrap();
    let join_outcome = rx.recv().unwrap();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    noise.join().unwrap();
    assert!(
        stun_thread.join().unwrap() >= 2,
        "both asked the STUN server"
    );
    check(&host_outcome, &join_outcome, &inputs);
}
