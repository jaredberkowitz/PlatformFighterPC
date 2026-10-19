//! Playing by connection code: UDP hole punching, so two players behind ordinary home routers connect with no server of ours
//! and no port forwarding.
//!
//! 1. [`CodeSocket::bind`] opens the UDP socket the match will use and asks public STUN servers, *from that socket*, what
//!    address the internet sees. With the LAN address and a random nonce that makes this player's [`ConnectionCode`].
//! 2. The players swap codes over any chat and both press Connect ([`CodeSocket::connect`]). Each side then sends small probe
//!    packets to the other's public and LAN addresses every 100 ms. A probe that gets out opens the sender's router for
//!    answers from that address; once both have sent, the probes get through and are answered.
//! 3. The first address that answers wins. [`CodeSocket::into_link`] turns the socket into a [`CodeLink`] and the ordinary
//!    netplay handshake runs over it. The lower nonce hosts (`ConnectionCode::hosts_against`).
//!
//! Probes carry both nonces, so a stranger's packet or one left over from an earlier session is ignored, and they never
//! reach the session: [`CodeLink`] answers late probes itself and drops them, along with stray STUN answers and anything from
//! an address that has not proven itself. The netplay wire format is untouched.
//!
//! Some routers ("strict" or symmetric NAT, common on mobile hotspots and some ISPs) give every destination a different
//! public port; the address STUN reports is then useless to the other player and punching fails. When two STUN servers report
//! different ports this is detected up front ([`CodeSocket::strict_nat`]).

use crate::code::{CodeError, ConnectionCode};
use crate::stun;
use netplay::peer::Link;
use std::hash::{BuildHasher, RandomState};
use std::io::{self, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// Probe datagrams start with these two bytes (netplay packets start with "FP", the little-endian bytes of its magic).
const PROBE_MAGIC: [u8; 2] = *b"HP";
const PROBE: u8 = 0;
const PROBE_ACK: u8 = 1;
const PROBE_LEN: usize = 2 + 1 + 2 + 2;
const MAX_DATAGRAM: usize = 1500;
/// At most this many addresses are trusted for one peer (its LAN address, its public one, a hairpin path).
const MAX_VERIFIED: usize = 4;

/// Timings, adjustable for tests.
#[derive(Clone, Debug)]
pub struct CodeOptions {
    /// STUN servers as "host:port". Empty: skip STUN (LAN only; tests).
    pub stun_servers: Vec<String>,
    /// Use this as the LAN address instead of asking the system (tests use 127.0.0.1).
    pub lan_override: Option<Ipv4Addr>,
    /// How long to wait for STUN answers before giving the code without a public address.
    pub stun_timeout: Duration,
    /// Time between STUN requests while looking.
    pub stun_retry: Duration,
    /// While waiting for the friend's code, a STUN request this often keeps the router's mapping alive.
    pub keepalive: Duration,
    /// Time between probes.
    pub probe_interval: Duration,
    /// How long to keep probing before giving up.
    pub punch_timeout: Duration,
}

impl Default for CodeOptions {
    fn default() -> CodeOptions {
        CodeOptions {
            stun_servers: stun::DEFAULT_SERVERS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            lan_override: None,
            stun_timeout: Duration::from_secs(5),
            stun_retry: Duration::from_millis(250),
            keepalive: Duration::from_secs(15),
            probe_interval: Duration::from_millis(100),
            // Longer than the 15 s the probes themselves need: the two players rarely press Connect at the same moment.
            punch_timeout: Duration::from_secs(45),
        }
    }
}

/// Where a [`CodeSocket`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeState {
    /// Asking STUN servers for the public address.
    FindingAddress,
    /// The code is ready; waiting for the player to paste the friend's code and press Connect.
    Ready,
    /// Probing the friend's addresses.
    Punching,
    /// The friend answered from this address.
    Connected(SocketAddr),
    /// Nothing answered in time.
    Failed,
}

/// Random bits without a dependency: std's per-process random hasher keys, mixed with the time and a counter.
fn random_u64(salt: u64) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    RandomState::new().hash_one((nanos, salt, std::process::id()))
}

/// This machine's address on its local network: the source address the system would use to reach the internet (no packet is
/// sent). Falls back to loopback when offline.
pub fn lan_ip() -> Ipv4Addr {
    let probe = || -> io::Result<Ipv4Addr> {
        let s = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        s.connect((Ipv4Addr::new(8, 8, 8, 8), 53))?;
        match s.local_addr()?.ip() {
            IpAddr::V4(ip) if !ip.is_unspecified() => Ok(ip),
            _ => Err(io::Error::other("no IPv4 route")),
        }
    };
    probe().unwrap_or(Ipv4Addr::LOCALHOST)
}

fn probe_bytes(kind: u8, from: u16, to: u16) -> [u8; PROBE_LEN] {
    let mut out = [0u8; PROBE_LEN];
    out[0..2].copy_from_slice(&PROBE_MAGIC);
    out[2] = kind;
    out[3..5].copy_from_slice(&from.to_le_bytes());
    out[5..7].copy_from_slice(&to.to_le_bytes());
    out
}

/// A probe or its answer: `Some(kind)` if `bytes` is one from the peer with nonce `theirs` meant for us (nonce `mine`).
fn read_probe(bytes: &[u8], mine: u16, theirs: u16) -> Option<u8> {
    if bytes.len() != PROBE_LEN || bytes[0..2] != PROBE_MAGIC {
        return None;
    }
    let from = u16::from_le_bytes([bytes[3], bytes[4]]);
    let to = u16::from_le_bytes([bytes[5], bytes[6]]);
    (from == theirs && to == mine && (bytes[2] == PROBE || bytes[2] == PROBE_ACK))
        .then_some(bytes[2])
}

fn is_probe(bytes: &[u8]) -> bool {
    bytes.len() == PROBE_LEN && bytes[0..2] == PROBE_MAGIC
}

/// The STUN side of a [`CodeSocket`].
struct StunState {
    /// Resolved servers arrive from a background lookup (DNS can take a moment, and the menu must not freeze).
    resolving: Option<Receiver<Vec<SocketAddr>>>,
    servers: Vec<SocketAddr>,
    /// Transaction ids: the base with the last byte set to the server's index.
    base_id: stun::TransactionId,
    /// What each server reported, by index.
    answers: Vec<(usize, SocketAddrV4)>,
    next_server: usize,
    last_sent: Option<Instant>,
    started: Instant,
    done: bool,
}

impl StunState {
    fn id_for(&self, server: usize) -> stun::TransactionId {
        let mut id = self.base_id;
        id[11] = server as u8;
        id
    }
}

/// The socket a code match is played on, from finding the public address to the punched connection.
pub struct CodeSocket {
    socket: UdpSocket,
    opts: CodeOptions,
    lan: SocketAddrV4,
    nonce: u16,
    stun: StunState,
    public: Option<SocketAddrV4>,
    strict: bool,
    last_keepalive: Instant,
    their: Option<ConnectionCode>,
    targets: Vec<SocketAddr>,
    punch_started: Option<Instant>,
    last_probe: Option<Instant>,
    verified: Vec<SocketAddr>,
    chosen: Option<SocketAddr>,
    failed: bool,
    buf: Vec<u8>,
}

impl CodeSocket {
    /// Binds UDP `port` on every interface (or any free port if that one is taken, or for 0) and starts asking STUN.
    pub fn bind(port: u16, opts: CodeOptions) -> io::Result<CodeSocket> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))
            .or_else(|_| UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)))?;
        socket.set_nonblocking(true)?;
        let local_port = socket.local_addr()?.port();
        let lan = SocketAddrV4::new(opts.lan_override.unwrap_or_else(lan_ip), local_port);
        let r = random_u64(u64::from(local_port));
        let r2 = random_u64(r);
        let mut base_id = [0u8; 12];
        base_id[..8].copy_from_slice(&r2.to_le_bytes());
        base_id[8..11].copy_from_slice(&r.to_le_bytes()[..3]);
        let resolving = if opts.stun_servers.is_empty() {
            None
        } else {
            let names = opts.stun_servers.clone();
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let found: Vec<SocketAddr> = names
                    .iter()
                    .filter_map(|n| n.to_socket_addrs().ok()?.find(SocketAddr::is_ipv4))
                    .collect();
                let _ = tx.send(found);
            });
            Some(rx)
        };
        let now = Instant::now();
        Ok(CodeSocket {
            socket,
            lan,
            nonce: (r >> 40) as u16,
            stun: StunState {
                done: resolving.is_none(),
                resolving,
                servers: Vec::new(),
                base_id,
                answers: Vec::new(),
                next_server: 0,
                last_sent: None,
                started: now,
            },
            opts,
            public: None,
            strict: false,
            last_keepalive: now,
            their: None,
            targets: Vec::new(),
            punch_started: None,
            last_probe: None,
            verified: Vec::new(),
            chosen: None,
            failed: false,
            buf: vec![0; MAX_DATAGRAM],
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn state(&self) -> CodeState {
        if let Some(a) = self.chosen {
            CodeState::Connected(a)
        } else if self.failed {
            CodeState::Failed
        } else if self.punch_started.is_some() {
            CodeState::Punching
        } else if !self.stun.done {
            CodeState::FindingAddress
        } else {
            CodeState::Ready
        }
    }

    /// This player's code, once the address search is over (with or without a public address).
    pub fn my_code(&self) -> Option<ConnectionCode> {
        (self.stun.done || self.punch_started.is_some()).then_some(ConnectionCode {
            public: self.public,
            lan: self.lan,
            nonce: self.nonce,
        })
    }

    /// The public address STUN reported, if any.
    pub fn public_addr(&self) -> Option<SocketAddrV4> {
        self.public
    }

    /// True when two STUN servers saw different public ports: the router gives every destination its own port, and codes
    /// will very likely not work from here.
    pub fn strict_nat(&self) -> bool {
        self.strict
    }

    /// Whether this player hosts against `their` code.
    pub fn hosts_against(&self, their: &ConnectionCode) -> bool {
        self.my_code().is_some_and(|m| m.hosts_against(their))
    }

    /// Time left before probing gives up, while punching.
    pub fn time_left(&self) -> Duration {
        self.punch_started.map_or(Duration::ZERO, |t| {
            self.opts.punch_timeout.saturating_sub(t.elapsed())
        })
    }

    /// Starts probing the friend. Refuses this player's own code. Calling it again (after a failure, or with another code)
    /// starts over.
    pub fn connect(&mut self, their: ConnectionCode) -> Result<(), CodeError> {
        if self.my_code() == Some(their) {
            return Err(CodeError::Invalid);
        }
        self.their = Some(their);
        self.targets.clear();
        self.targets.push(SocketAddr::V4(their.lan));
        if let Some(p) = their.public {
            if !self.targets.contains(&SocketAddr::V4(p)) {
                self.targets.push(SocketAddr::V4(p));
            }
        }
        self.punch_started = Some(Instant::now());
        self.last_probe = None;
        self.verified.clear();
        self.chosen = None;
        self.failed = false;
        Ok(())
    }

    /// Stops probing (the player went back); the code stays valid.
    pub fn cancel(&mut self) {
        self.punch_started = None;
        self.chosen = None;
        self.failed = false;
    }

    /// Call often (every frame): reads the socket, sends STUN requests and probes on schedule. Never blocks.
    pub fn poll(&mut self) {
        let now = Instant::now();
        self.poll_resolver();
        self.recv_all();
        self.send_stun(now);
        if let Some(started) = self.punch_started {
            if self.chosen.is_none() && !self.failed {
                if now.duration_since(started) >= self.opts.punch_timeout {
                    self.failed = true;
                } else if self
                    .last_probe
                    .is_none_or(|t| now.duration_since(t) >= self.opts.probe_interval)
                {
                    self.last_probe = Some(now);
                    if let Some(their) = self.their {
                        let probe = probe_bytes(PROBE, self.nonce, their.nonce);
                        for t in &self.targets {
                            let _ = self.socket.send_to(&probe, t);
                        }
                    }
                }
            }
        }
    }

    fn poll_resolver(&mut self) {
        if let Some(rx) = &self.stun.resolving {
            match rx.try_recv() {
                Ok(list) => {
                    self.stun.servers = list;
                    self.stun.resolving = None;
                    if self.stun.servers.is_empty() {
                        self.stun.done = true; // no DNS (offline): the code carries the LAN address only
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.stun.resolving = None;
                    self.stun.done = true;
                }
            }
        }
    }

    fn send_stun(&mut self, now: Instant) {
        if self.stun.servers.is_empty() {
            return;
        }
        if !self.stun.done {
            let enough = self.stun.answers.len() >= 2
                || self.stun.answers.len() >= self.stun.servers.len()
                || now.duration_since(self.stun.started) >= self.opts.stun_timeout;
            if enough {
                self.stun.done = true;
                self.last_keepalive = now;
                return;
            }
            if self
                .stun
                .last_sent
                .is_some_and(|t| now.duration_since(t) < self.opts.stun_retry)
            {
                return;
            }
            // Ask a server that has not answered yet, round robin.
            let n = self.stun.servers.len();
            for _ in 0..n {
                let i = self.stun.next_server % n;
                self.stun.next_server += 1;
                if !self.stun.answers.iter().any(|(s, _)| *s == i) {
                    let req = stun::binding_request(&self.stun.id_for(i));
                    let _ = self.socket.send_to(&req, self.stun.servers[i]);
                    break;
                }
            }
            self.stun.last_sent = Some(now);
        } else if self.punch_started.is_none()
            && now.duration_since(self.last_keepalive) >= self.opts.keepalive
        {
            // Keep the router's mapping for this socket alive while the players swap codes.
            self.last_keepalive = now;
            if let Some((i, _)) = self.stun.answers.first() {
                let req = stun::binding_request(&self.stun.id_for(*i));
                let _ = self.socket.send_to(&req, self.stun.servers[*i]);
            }
        }
    }

    fn recv_all(&mut self) {
        loop {
            let (n, from) = match self.socket.recv_from(&mut self.buf) {
                Ok(x) => x,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                // On Windows a datagram that bounced (nobody listening there yet) surfaces as a reset; skip it.
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return,
            };
            let bytes = &self.buf[..n];
            if let Some(id) = stun::transaction_id(bytes) {
                let server = usize::from(id[11]);
                if server < self.stun.servers.len() {
                    if let Some(addr) =
                        stun::parse_binding_response(bytes, &self.stun.id_for(server))
                    {
                        self.record_stun(server, addr);
                    }
                }
                continue;
            }
            let Some(their) = self.their else { continue };
            if self.punch_started.is_none() {
                continue;
            }
            // Anything else is a stranger, an old session, or the friend's handshake arriving early (it is resent).
            if let Some(kind) = read_probe(bytes, self.nonce, their.nonce) {
                if !self.verified.contains(&from) && self.verified.len() < MAX_VERIFIED {
                    self.verified.push(from);
                }
                if kind == PROBE {
                    let _ = self
                        .socket
                        .send_to(&probe_bytes(PROBE_ACK, self.nonce, their.nonce), from);
                } else if self.chosen.is_none() && !self.failed {
                    self.chosen = Some(from);
                }
            }
        }
    }

    fn record_stun(&mut self, server: usize, addr: SocketAddrV4) {
        if let Some(entry) = self.stun.answers.iter_mut().find(|(s, _)| *s == server) {
            entry.1 = addr; // a keepalive answer
        } else {
            self.stun.answers.push((server, addr));
        }
        // Two servers seeing different public ports: the router maps per destination.
        let ports: Vec<u16> = self.stun.answers.iter().map(|(_, a)| a.port()).collect();
        self.strict = ports.windows(2).any(|w| w[0] != w[1]);
        // The code uses the first server's answer (a keepalive refreshes it if the router moved us).
        self.public = self.stun.answers.first().map(|(_, a)| *a);
    }

    /// Once connected: the link the match is played over. `None` before that.
    pub fn into_link(self) -> Option<CodeLink> {
        let peer = self.chosen?;
        let their = self.their?;
        Some(CodeLink {
            peer,
            mine: self.nonce,
            theirs: their.nonce,
            inner: self,
        })
    }
}

/// The punched connection: like a direct `UdpLink`, but it only accepts datagrams from addresses that proved themselves with
/// the right nonces, answers late probes itself, and never passes a probe or a STUN packet to the session. It keeps the whole
/// [`CodeSocket`] it came from, so after the match [`CodeLink::release`] gives the socket back with the same code.
pub struct CodeLink {
    inner: CodeSocket,
    peer: SocketAddr,
    mine: u16,
    theirs: u16,
}

impl CodeLink {
    /// The friend's address the match is played with.
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.socket.local_addr()
    }

    /// Done with this friend: the socket goes back to waiting for a code to connect to, with the same code as before (the same
    /// port, public address and nonce), so a player keeps one code for as long as the game is open. The router's mapping stayed
    /// alive through the match's traffic; the keepalive takes over again from now.
    pub fn release(self) -> CodeSocket {
        let mut s = self.inner;
        s.their = None;
        s.targets.clear();
        s.punch_started = None;
        s.last_probe = None;
        s.verified.clear();
        s.chosen = None;
        s.failed = false;
        s.last_keepalive = Instant::now()
            .checked_sub(s.opts.keepalive)
            .unwrap_or_else(Instant::now);
        s
    }
}

impl Link for CodeLink {
    fn send(&mut self, bytes: &[u8]) {
        let _ = self.inner.socket.send_to(bytes, self.peer);
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        let s = &mut self.inner;
        loop {
            let (n, from) = match s.socket.recv_from(&mut s.buf) {
                Ok(x) => x,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return None,
            };
            let bytes = &s.buf[..n];
            if let Some(kind) = read_probe(bytes, self.mine, self.theirs) {
                // The friend is still probing (our answer was lost, or it has not seen one yet): answer, and trust the path.
                if !s.verified.contains(&from) && s.verified.len() < MAX_VERIFIED {
                    s.verified.push(from);
                }
                if kind == PROBE {
                    let _ = s
                        .socket
                        .send_to(&probe_bytes(PROBE_ACK, self.mine, self.theirs), from);
                }
                continue;
            }
            if is_probe(bytes) || stun::looks_like_stun(bytes) || !s.verified.contains(&from) {
                continue;
            }
            return Some(bytes.to_vec());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    fn opts(stun_servers: Vec<String>) -> CodeOptions {
        CodeOptions {
            stun_servers,
            lan_override: Some(Ipv4Addr::LOCALHOST),
            stun_timeout: Duration::from_millis(800),
            stun_retry: Duration::from_millis(30),
            keepalive: Duration::from_millis(100),
            probe_interval: Duration::from_millis(20),
            punch_timeout: Duration::from_secs(5),
        }
    }

    /// A fake STUN server on localhost: answers every Binding Request with the sender's address, with the port shifted by
    /// `shift` (so a test can pretend to be behind a router, or a strict one when two servers shift differently).
    struct FakeStun {
        socket: UdpSocket,
        shift: u16,
        answered: u32,
    }

    impl FakeStun {
        fn new(shift: u16) -> FakeStun {
            let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            socket.set_nonblocking(true).unwrap();
            FakeStun {
                socket,
                shift,
                answered: 0,
            }
        }

        fn addr(&self) -> String {
            self.socket.local_addr().unwrap().to_string()
        }

        fn pump(&mut self) {
            let mut buf = [0u8; 1500];
            while let Ok((n, from)) = self.socket.recv_from(&mut buf) {
                let (Some(id), SocketAddr::V4(from)) = (stun::transaction_id(&buf[..n]), from)
                else {
                    continue;
                };
                let seen = SocketAddrV4::new(*from.ip(), from.port().wrapping_add(self.shift));
                let _ = self
                    .socket
                    .send_to(&stun::binding_response(&id, seen), from);
                self.answered += 1;
            }
        }
    }

    fn run_until(mut f: impl FnMut() -> bool) -> bool {
        for _ in 0..1000 {
            if f() {
                return true;
            }
            sleep(Duration::from_millis(5));
        }
        false
    }

    #[test]
    fn stun_finds_the_public_address_from_the_match_socket() {
        let mut server = FakeStun::new(0);
        let mut sock = CodeSocket::bind(0, opts(vec![server.addr()])).unwrap();
        assert_eq!(sock.state(), CodeState::FindingAddress);
        assert_eq!(sock.my_code(), None);
        assert!(run_until(|| {
            server.pump();
            sock.poll();
            sock.state() == CodeState::Ready
        }));
        let me = match sock.local_addr().unwrap() {
            SocketAddr::V4(a) => SocketAddrV4::new(Ipv4Addr::LOCALHOST, a.port()),
            _ => unreachable!(),
        };
        assert_eq!(
            sock.public_addr(),
            Some(me),
            "the server saw the match socket itself"
        );
        let code = sock.my_code().unwrap();
        assert_eq!(code.public, Some(me));
        assert_eq!(ConnectionCode::decode(&code.encode()), Ok(code));
        assert!(!sock.strict_nat());
        // Keepalives keep going while the players swap codes.
        let before = server.answered;
        run_until(|| {
            server.pump();
            sock.poll();
            server.answered > before + 1
        });
        assert!(server.answered > before + 1);
    }

    #[test]
    fn two_servers_reporting_different_ports_mean_a_strict_router() {
        let mut a = FakeStun::new(0);
        let mut b = FakeStun::new(7);
        let mut sock = CodeSocket::bind(0, opts(vec![a.addr(), b.addr()])).unwrap();
        assert!(run_until(|| {
            a.pump();
            b.pump();
            sock.poll();
            sock.state() == CodeState::Ready
        }));
        assert!(sock.strict_nat());
    }

    #[test]
    fn with_no_stun_answer_the_code_still_carries_the_lan_address() {
        // A server address nobody listens on.
        let dead = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let dead_addr = dead.local_addr().unwrap().to_string();
        drop(dead);
        let mut o = opts(vec![dead_addr]);
        o.stun_timeout = Duration::from_millis(150);
        let mut sock = CodeSocket::bind(0, o).unwrap();
        assert!(run_until(|| {
            sock.poll();
            sock.state() == CodeState::Ready
        }));
        let code = sock.my_code().unwrap();
        assert_eq!(code.public, None);
        assert_eq!(*code.lan.ip(), Ipv4Addr::LOCALHOST);
    }

    /// Two players on one machine: fake STUN, codes swapped, both connect, each learns the other's address, and the punched
    /// links carry data both ways. A stranger, an old session's probe, and late probes never reach the session.
    #[test]
    fn two_sockets_punch_through_and_ignore_strangers() {
        let mut server = FakeStun::new(0);
        let mut a = CodeSocket::bind(0, opts(vec![server.addr()])).unwrap();
        let mut b = CodeSocket::bind(0, opts(vec![server.addr()])).unwrap();
        assert!(run_until(|| {
            server.pump();
            a.poll();
            b.poll();
            a.state() == CodeState::Ready && b.state() == CodeState::Ready
        }));
        let (ca, cb) = (a.my_code().unwrap(), b.my_code().unwrap());
        // Codes travel as text.
        let (ca, cb) = (
            ConnectionCode::decode(&ca.encode()).unwrap(),
            ConnectionCode::decode(&cb.encode()).unwrap(),
        );
        assert_ne!(
            a.hosts_against(&cb),
            b.hosts_against(&ca),
            "exactly one hosts"
        );
        assert_eq!(
            a.connect(ca),
            Err(CodeError::Invalid),
            "your own code is refused"
        );
        a.connect(cb).unwrap();
        // b presses Connect a little later.
        for _ in 0..10 {
            a.poll();
            sleep(Duration::from_millis(5));
        }
        assert_eq!(a.state(), CodeState::Punching);
        b.connect(ca).unwrap();
        assert!(run_until(|| {
            a.poll();
            b.poll();
            matches!(a.state(), CodeState::Connected(_))
                && matches!(b.state(), CodeState::Connected(_))
        }));
        let a_addr = a.local_addr().unwrap();
        let b_addr = b.local_addr().unwrap();
        let mut la = a.into_link().unwrap();
        let mut lb = b.into_link().unwrap();
        assert_eq!(la.peer().port(), b_addr.port());
        assert_eq!(lb.peer().port(), a_addr.port());

        // Noise aimed at a: a stranger, a probe from an old session, a stray STUN answer, a late probe from b.
        let stranger = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let a_local = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), a_addr.port());
        stranger.send_to(b"spoofed netplay bytes", a_local).unwrap();
        stranger
            .send_to(&probe_bytes(PROBE, cb.nonce, ca.nonce), a_local)
            .unwrap();
        stranger.send_to(b"after a forged probe", a_local).unwrap();
        stranger
            .send_to(&probe_bytes(PROBE, cb.nonce ^ 1, ca.nonce), a_local)
            .unwrap();
        stranger
            .send_to(
                &stun::binding_response(&[1; 12], SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9)),
                a_local,
            )
            .unwrap();
        lb.send(&probe_bytes(PROBE, cb.nonce, ca.nonce));
        lb.send(b"real data");
        let mut got = Vec::new();
        run_until(|| {
            while let Some(d) = la.recv() {
                got.push(d);
            }
            !got.is_empty()
        });
        // The stranger's forged probe made it trusted (the nonces are in the code, which only the friend should have), so
        // its next datagram may arrive; what matters is that probes and STUN never do, and the real data does.
        assert!(
            got.iter()
                .all(|d| !is_probe(d) && !stun::looks_like_stun(d)),
            "{got:?}"
        );
        assert!(got.iter().any(|d| d == b"real data"));
        assert!(
            !got.iter().any(|d| d == b"spoofed netplay bytes"),
            "the unproven stranger was ignored"
        );
        la.send(b"back");
        assert!(run_until(|| lb.recv().as_deref() == Some(&b"back"[..])));
    }

    #[test]
    fn a_released_link_keeps_its_code_and_connects_again() {
        // One code for as long as the game is open: after a match the socket goes back to waiting, with the same code, and the
        // two can connect again with the codes they already swapped.
        let mut server = FakeStun::new(0);
        let mut a = CodeSocket::bind(0, opts(vec![server.addr()])).unwrap();
        let mut b = CodeSocket::bind(0, opts(vec![server.addr()])).unwrap();
        assert!(run_until(|| {
            server.pump();
            a.poll();
            b.poll();
            a.state() == CodeState::Ready && b.state() == CodeState::Ready
        }));
        let (ca, cb) = (a.my_code().unwrap(), b.my_code().unwrap());
        for round in 0..3 {
            a.connect(cb).unwrap();
            b.connect(ca).unwrap();
            assert!(
                run_until(|| {
                    a.poll();
                    b.poll();
                    matches!(a.state(), CodeState::Connected(_))
                        && matches!(b.state(), CodeState::Connected(_))
                }),
                "round {round}: connected"
            );
            let mut la = a.into_link().unwrap();
            let mut lb = b.into_link().unwrap();
            la.send(b"match");
            assert!(run_until(|| lb.recv().as_deref() == Some(&b"match"[..])));
            a = la.release();
            b = lb.release();
            assert_eq!(a.state(), CodeState::Ready, "round {round}");
            assert_eq!(a.my_code(), Some(ca), "round {round}: the same code");
            assert_eq!(b.my_code(), Some(cb), "round {round}: the same code");
            // A late game datagram from the friend does not disturb the waiting socket.
            assert!(run_until(|| {
                server.pump();
                a.poll();
                b.poll();
                true
            }));
            assert_eq!(a.state(), CodeState::Ready);
        }
    }

    #[test]
    fn nobody_answering_fails_in_time() {
        let mut a = CodeSocket::bind(0, opts(Vec::new())).unwrap();
        a.opts.punch_timeout = Duration::from_millis(200);
        assert_eq!(
            a.state(),
            CodeState::Ready,
            "without STUN the code is ready at once"
        );
        let ghost = ConnectionCode {
            public: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9)),
            lan: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9),
            nonce: a.nonce.wrapping_add(1),
        };
        a.connect(ghost).unwrap();
        assert!(run_until(|| {
            a.poll();
            a.state() == CodeState::Failed
        }));
        assert_eq!(a.time_left(), Duration::ZERO);
        assert!(a.into_link().is_none());
    }
}
