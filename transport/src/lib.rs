//! UDP transport for netplay: the part that touches sockets, so the `netplay` crate can stay pure.
//!
//! * [`UdpLink`]: a direct peer-to-peer link. The host binds a port; the joiner is given the host's address. The
//!   host learns the joiner's address from its first packet and ignores everyone else after that.
//! * [`RelayLink`] and [`Relay`]: a fallback for players who cannot connect directly (no port forwarding). Both
//!   clients send to a relay server with a shared room number and the relay forwards between them. It adds one hop of
//!   latency but needs no NAT traversal.
//!
//! Neither hole-punches NAT; direct play over the internet needs a forwarded port (or a LAN / VPN), and anything else
//! uses the relay.

use netplay::group::HubLink;
use netplay::peer::Link;
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

/// Largest datagram handled; protocol packets are far smaller.
const MAX_DATAGRAM: usize = 1500;

fn bind_nonblocking(addr: SocketAddr) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(addr)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

/// A direct link to one peer.
pub struct UdpLink {
    socket: UdpSocket,
    peer: Option<SocketAddr>,
    buf: Vec<u8>,
}

impl UdpLink {
    /// Binds `local`. The joiner passes the host's address as `peer`; the host passes `None` and learns the joiner's
    /// address from the first packet that arrives.
    pub fn bind(local: SocketAddr, peer: Option<SocketAddr>) -> io::Result<UdpLink> {
        Ok(UdpLink {
            socket: bind_nonblocking(local)?,
            peer,
            buf: vec![0; MAX_DATAGRAM],
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn peer(&self) -> Option<SocketAddr> {
        self.peer
    }
}

impl Link for UdpLink {
    fn send(&mut self, bytes: &[u8]) {
        if let Some(peer) = self.peer {
            // A full buffer or an unreachable peer just loses the datagram, like the network would.
            let _ = self.socket.send_to(bytes, peer);
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((n, from)) => {
                    match self.peer {
                        None => self.peer = Some(from),
                        Some(p) if p != from => continue, // not our peer
                        Some(_) => {}
                    }
                    return Some(self.buf[..n].to_vec());
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                // On Windows a datagram that bounced (peer not listening yet) surfaces as a reset; skip it.
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return None,
            }
        }
    }
}

// ---- Spectators ----------------------------------------------------------------------------------------------------

/// The host's socket for spectators: it talks to any number of addresses (a [`UdpLink`] talks to one). Addresses are mapped to small
/// numbers (the ids a `netplay::spectate::SpectatorServer` uses); at most `max` are remembered, and a datagram from a ninth stranger is
/// dropped.
pub struct SpectatorSocket {
    socket: UdpSocket,
    addrs: Vec<SocketAddr>,
    max: usize,
    buf: Vec<u8>,
}

impl SpectatorSocket {
    pub fn bind(local: SocketAddr, max: usize) -> io::Result<SpectatorSocket> {
        Ok(SpectatorSocket {
            socket: bind_nonblocking(local)?,
            addrs: Vec::new(),
            max,
            buf: vec![0; MAX_DATAGRAM],
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// The next datagram: the sender's id and the bytes.
    pub fn recv(&mut self) -> Option<(usize, Vec<u8>)> {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((n, from)) => {
                    let id = match self.addrs.iter().position(|a| *a == from) {
                        Some(i) => i,
                        None if self.addrs.len() < self.max => {
                            self.addrs.push(from);
                            self.addrs.len() - 1
                        }
                        None => continue,
                    };
                    return Some((id, self.buf[..n].to_vec()));
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return None,
            }
        }
    }

    pub fn send(&mut self, id: usize, bytes: &[u8]) {
        if let Some(addr) = self.addrs.get(id) {
            let _ = self.socket.send_to(bytes, addr);
        }
    }
}

/// The same socket is the host's side of a group match: guest `i` is the `i`-th address that wrote to it.
impl HubLink for SpectatorSocket {
    fn send(&mut self, guest: usize, bytes: &[u8]) {
        SpectatorSocket::send(self, guest, bytes);
    }

    fn recv(&mut self) -> Option<(usize, Vec<u8>)> {
        SpectatorSocket::recv(self)
    }
}

// ---- Relay -------------------------------------------------------------------------------------------------------

const RELAY_MAGIC: [u8; 2] = [0x52, 0x4c]; // "RL"
const RELAY_HEADER: usize = 2 + 8;

/// A client of a relay server: wraps every datagram with the room number.
pub struct RelayLink {
    socket: UdpSocket,
    relay: SocketAddr,
    room: u64,
    buf: Vec<u8>,
}

impl RelayLink {
    pub fn connect(local: SocketAddr, relay: SocketAddr, room: u64) -> io::Result<RelayLink> {
        Ok(RelayLink {
            socket: bind_nonblocking(local)?,
            relay,
            room,
            buf: vec![0; MAX_DATAGRAM],
        })
    }

    /// Registers with the relay without sending any payload (so the other side's packets can find us).
    pub fn announce(&mut self) {
        self.send(&[]);
    }
}

impl Link for RelayLink {
    fn send(&mut self, bytes: &[u8]) {
        let mut out = Vec::with_capacity(RELAY_HEADER + bytes.len());
        out.extend_from_slice(&RELAY_MAGIC);
        out.extend_from_slice(&self.room.to_le_bytes());
        out.extend_from_slice(bytes);
        let _ = self.socket.send_to(&out, self.relay);
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((n, from)) => {
                    if from != self.relay || n == 0 {
                        continue;
                    }
                    return Some(self.buf[..n].to_vec());
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return None,
            }
        }
    }
}

// ---- Quick match -------------------------------------------------------------------------------------------------

/// A datagram from the relay telling a waiting client its match: "RM", the room number, then 0 if it should host or 1 if it should join.
const MATCH_MAGIC: [u8; 2] = [0x52, 0x4d];
/// The room number clients send to join the quick-match queue (typed room numbers must be above zero).
pub const QUEUE_ROOM: u64 = 0;
/// Quick-match rooms are numbered from here up, well clear of the numbers people type.
const FIRST_MATCH_ROOM: u64 = 1 << 40;
/// How long a waiting client stays in the queue without asking again.
const QUEUE_PATIENCE: Duration = Duration::from_secs(6);

/// A client looking for an opponent through the relay: it asks the queue, and the relay answers when someone else has asked too.
pub struct QuickMatch {
    socket: UdpSocket,
    relay: SocketAddr,
    buf: Vec<u8>,
    polls: u32,
}

impl QuickMatch {
    pub fn connect(local: SocketAddr, relay: SocketAddr) -> io::Result<QuickMatch> {
        Ok(QuickMatch {
            socket: bind_nonblocking(local)?,
            relay,
            buf: vec![0; MAX_DATAGRAM],
            polls: 0,
        })
    }

    /// Call once a frame: keeps asking the queue, and returns `(room, host)` once an opponent has been found. Then connect a
    /// [`RelayLink`] to that room, hosting if told to.
    pub fn poll(&mut self) -> Option<(u64, bool)> {
        self.polls += 1;
        if self.polls % 30 == 1 {
            let mut out = RELAY_MAGIC.to_vec();
            out.extend_from_slice(&QUEUE_ROOM.to_le_bytes());
            let _ = self.socket.send_to(&out, self.relay);
        }
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((n, from)) => {
                    if from == self.relay && n == 2 + 8 + 1 && self.buf[..2] == MATCH_MAGIC {
                        let mut room = [0u8; 8];
                        room.copy_from_slice(&self.buf[2..10]);
                        return Some((u64::from_le_bytes(room), self.buf[10] == 0));
                    }
                }
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return None,
            }
        }
    }
}

/// The relay server: forwards each client's datagrams to the other client in the same room. Two clients per room.
pub struct Relay {
    socket: UdpSocket,
    rooms: BTreeMap<u64, [Option<SocketAddr>; 2]>,
    buf: Vec<u8>,
    forwarded: u64,
    /// Clients waiting for an opponent, oldest first.
    queue: Vec<(SocketAddr, Instant)>,
    next_match_room: u64,
    patience: Duration,
}

impl Relay {
    pub fn bind(addr: SocketAddr) -> io::Result<Relay> {
        Ok(Relay {
            socket: bind_nonblocking(addr)?,
            rooms: BTreeMap::new(),
            buf: vec![0; MAX_DATAGRAM],
            forwarded: 0,
            queue: Vec::new(),
            next_match_room: FIRST_MATCH_ROOM,
            patience: QUEUE_PATIENCE,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// How long a waiting client stays queued without asking again (tests shorten it).
    pub fn set_queue_patience(&mut self, patience: Duration) {
        self.patience = patience;
    }

    /// Clients waiting for an opponent right now.
    pub fn waiting(&self) -> usize {
        self.queue.len()
    }

    /// A client asked for a match: it joins the queue, and if someone is already waiting the two are paired.
    fn queue_up(&mut self, from: SocketAddr) {
        let now = Instant::now();
        let patience = self.patience;
        self.queue
            .retain(|(_, t)| now.duration_since(*t) < patience);
        if let Some(entry) = self.queue.iter_mut().find(|(a, _)| *a == from) {
            entry.1 = now; // still waiting
            return;
        }
        if self.queue.len() >= 256 {
            return; // a flood of made-up clients cannot grow the queue without bound
        }
        if self.queue.is_empty() {
            self.queue.push((from, now));
            return;
        }
        let (waiting, _) = self.queue.remove(0);
        let room = self.next_match_room;
        self.next_match_room += 1;
        for (addr, role) in [(waiting, 0u8), (from, 1u8)] {
            let mut out = MATCH_MAGIC.to_vec();
            out.extend_from_slice(&room.to_le_bytes());
            out.push(role);
            let _ = self.socket.send_to(&out, addr);
        }
    }

    pub fn forwarded(&self) -> u64 {
        self.forwarded
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Handles every datagram waiting right now.
    pub fn pump(&mut self) {
        loop {
            let (n, from) = match self.socket.recv_from(&mut self.buf) {
                Ok(x) => x,
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => return,
            };
            if n < RELAY_HEADER || self.buf[..2] != RELAY_MAGIC {
                continue;
            }
            let mut room_bytes = [0u8; 8];
            room_bytes.copy_from_slice(&self.buf[2..RELAY_HEADER]);
            let room = u64::from_le_bytes(room_bytes);
            if room == QUEUE_ROOM {
                self.queue_up(from);
                continue;
            }

            // Cap the number of rooms so a flood of made-up room numbers cannot use unbounded memory.
            if !self.rooms.contains_key(&room) && self.rooms.len() >= 4096 {
                continue;
            }
            let slots = self.rooms.entry(room).or_insert([None, None]);
            let me = match slots.iter().position(|s| *s == Some(from)) {
                Some(i) => i,
                None => match slots.iter().position(Option::is_none) {
                    Some(i) => {
                        slots[i] = Some(from);
                        i
                    }
                    None => continue, // a third client: ignored
                },
            };
            if n == RELAY_HEADER {
                continue; // an announcement carries no payload
            }
            if let Some(other) = slots[1 - me] {
                let _ = self.socket.send_to(&self.buf[RELAY_HEADER..n], other);
                self.forwarded += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::thread::sleep;
    use std::time::Duration;

    fn any() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
    }

    fn wait_for<L: Link>(link: &mut L, relay: Option<&mut Relay>) -> Option<Vec<u8>> {
        let mut relay = relay;
        for _ in 0..200 {
            if let Some(r) = relay.as_deref_mut() {
                r.pump();
            }
            if let Some(b) = link.recv() {
                return Some(b);
            }
            sleep(Duration::from_millis(5));
        }
        None
    }

    #[test]
    fn two_udp_links_exchange_datagrams_and_the_host_learns_the_joiner() {
        let mut host = UdpLink::bind(any(), None).unwrap();
        let mut join = UdpLink::bind(any(), Some(host.local_addr().unwrap())).unwrap();
        assert_eq!(host.peer(), None);
        join.send(b"hello");
        assert_eq!(wait_for(&mut host, None).as_deref(), Some(&b"hello"[..]));
        assert_eq!(host.peer(), Some(join.local_addr().unwrap()));
        host.send(b"welcome");
        assert_eq!(wait_for(&mut join, None).as_deref(), Some(&b"welcome"[..]));
    }

    #[test]
    fn a_stranger_is_ignored_once_the_peer_is_known() {
        let mut host = UdpLink::bind(any(), None).unwrap();
        let mut join = UdpLink::bind(any(), Some(host.local_addr().unwrap())).unwrap();
        let stranger = UdpSocket::bind(any()).unwrap();
        join.send(b"first");
        assert!(wait_for(&mut host, None).is_some());
        stranger
            .send_to(b"spoof", host.local_addr().unwrap())
            .unwrap();
        sleep(Duration::from_millis(30));
        assert_eq!(host.recv(), None);
    }

    #[test]
    fn the_relay_forwards_between_two_clients_in_a_room_and_keeps_rooms_apart() {
        let mut relay = Relay::bind(any()).unwrap();
        let relay_addr = relay.local_addr().unwrap();
        let mut a = RelayLink::connect(any(), relay_addr, 77).unwrap();
        let mut b = RelayLink::connect(any(), relay_addr, 77).unwrap();
        let mut other_room = RelayLink::connect(any(), relay_addr, 78).unwrap();
        let mut other_peer = RelayLink::connect(any(), relay_addr, 78).unwrap();
        a.announce();
        b.announce();
        other_room.announce();
        other_peer.announce();
        a.send(b"to b");
        assert_eq!(
            wait_for(&mut b, Some(&mut relay)).as_deref(),
            Some(&b"to b"[..])
        );
        b.send(b"to a");
        assert_eq!(
            wait_for(&mut a, Some(&mut relay)).as_deref(),
            Some(&b"to a"[..])
        );
        // Nothing leaked into the other room.
        relay.pump();
        assert_eq!(other_room.recv(), None);
        assert_eq!(other_peer.recv(), None);
        assert_eq!(relay.room_count(), 2);
        assert_eq!(relay.forwarded(), 2);
    }

    #[test]
    fn the_relay_ignores_garbage_and_a_third_client() {
        let mut relay = Relay::bind(any()).unwrap();
        let relay_addr = relay.local_addr().unwrap();
        let junk = UdpSocket::bind(any()).unwrap();
        junk.send_to(b"not a relay packet", relay_addr).unwrap();
        junk.send_to(&[], relay_addr).unwrap();
        let mut a = RelayLink::connect(any(), relay_addr, 1).unwrap();
        let mut b = RelayLink::connect(any(), relay_addr, 1).unwrap();
        let mut c = RelayLink::connect(any(), relay_addr, 1).unwrap();
        a.announce();
        b.announce();
        c.send(b"intruder");
        a.send(b"hi");
        assert_eq!(
            wait_for(&mut b, Some(&mut relay)).as_deref(),
            Some(&b"hi"[..])
        );
        sleep(Duration::from_millis(20));
        relay.pump();
        assert_eq!(a.recv(), None, "the intruder's packet was not forwarded");
    }
}

#[cfg(test)]
mod quick_match_tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::thread::sleep;

    fn any() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
    }

    type Found = Option<(u64, bool)>;

    fn poll_both(relay: &mut Relay, a: &mut QuickMatch, b: &mut QuickMatch) -> (Found, Found) {
        let (mut ra, mut rb) = (None, None);
        for _ in 0..200 {
            relay.pump();
            ra = ra.or(a.poll());
            rb = rb.or(b.poll());
            if ra.is_some() && rb.is_some() {
                break;
            }
            sleep(Duration::from_millis(5));
        }
        (ra, rb)
    }

    #[test]
    fn two_clients_looking_for_a_match_are_paired_into_one_room() {
        let mut relay = Relay::bind(any()).unwrap();
        let addr = relay.local_addr().unwrap();
        let mut a = QuickMatch::connect(any(), addr).unwrap();
        let mut b = QuickMatch::connect(any(), addr).unwrap();
        // The first waits alone.
        for _ in 0..40 {
            relay.pump();
            assert_eq!(a.poll(), None);
            sleep(Duration::from_millis(2));
        }
        assert_eq!(relay.waiting(), 1);
        let (ra, rb) = poll_both(&mut relay, &mut a, &mut b);
        let (room_a, host_a) = ra.expect("a is told");
        let (room_b, host_b) = rb.expect("b is told");
        assert_eq!(room_a, room_b, "the same room");
        assert!(room_a >= FIRST_MATCH_ROOM);
        assert!(host_a && !host_b, "the one who waited hosts");
        assert_eq!(relay.waiting(), 0);
        // And they can play through that room.
        let mut host = RelayLink::connect(any(), addr, room_a).unwrap();
        let mut join = RelayLink::connect(any(), addr, room_a).unwrap();
        host.announce();
        join.announce();
        host.send(b"hi");
        let mut got = None;
        for _ in 0..100 {
            relay.pump();
            got = got.or(join.recv());
            if got.is_some() {
                break;
            }
            sleep(Duration::from_millis(5));
        }
        assert_eq!(got.as_deref(), Some(&b"hi"[..]));
    }

    #[test]
    fn a_client_that_stops_asking_leaves_the_queue_and_rooms_do_not_repeat() {
        let mut relay = Relay::bind(any()).unwrap();
        relay.set_queue_patience(Duration::from_millis(60));
        let addr = relay.local_addr().unwrap();
        let mut gone = QuickMatch::connect(any(), addr).unwrap();
        gone.poll();
        sleep(Duration::from_millis(20));
        relay.pump();
        assert_eq!(relay.waiting(), 1);
        sleep(Duration::from_millis(120));
        // A newcomer finds the queue empty (the first has timed out), so it waits rather than being paired with a ghost.
        let mut b = QuickMatch::connect(any(), addr).unwrap();
        b.poll();
        sleep(Duration::from_millis(20));
        relay.pump();
        assert_eq!(relay.waiting(), 1);
        assert_eq!(b.poll(), None);
        // Two matches in a row get different rooms.
        relay.set_queue_patience(Duration::from_secs(6));
        let mut c = QuickMatch::connect(any(), addr).unwrap();
        let mut d = QuickMatch::connect(any(), addr).unwrap();
        let first = poll_both(&mut relay, &mut b, &mut c).0.unwrap().0;
        let mut e = QuickMatch::connect(any(), addr).unwrap();
        let second = poll_both(&mut relay, &mut d, &mut e).0.unwrap().0;
        assert_ne!(first, second);
    }

    #[test]
    fn typed_rooms_still_work() {
        let mut relay = Relay::bind(any()).unwrap();
        let addr = relay.local_addr().unwrap();
        let mut a = RelayLink::connect(any(), addr, 5).unwrap();
        let mut b = RelayLink::connect(any(), addr, 5).unwrap();
        a.announce();
        b.announce();
        a.send(b"x");
        let mut got = None;
        for _ in 0..100 {
            relay.pump();
            got = got.or(b.recv());
            if got.is_some() {
                break;
            }
            sleep(Duration::from_millis(5));
        }
        assert_eq!(got.as_deref(), Some(&b"x"[..]));
        assert_eq!(relay.waiting(), 0);
    }
}
