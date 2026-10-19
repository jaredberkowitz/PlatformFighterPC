//! `NetCode`: playing a friend by connection code, for the online screen. It owns the UDP socket from the moment the screen
//! opens (so the public address STUN finds is the one the match uses) until the punched link is handed to a `SimRunner`
//! (`net_host_code` / `net_join_code`). Being a `RefCounted`, it survives the change from the online screen to the match scene
//! inside `Roster.session`. All the work is in `transport::punch`; this only adapts it to Godot.

use godot::prelude::*;
use std::net::Ipv4Addr;
use transport::{CodeLink, CodeOptions, CodeSocket, CodeState, ConnectionCode};

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct NetCode {
    sock: Option<CodeSocket>,
    their: Option<ConnectionCode>,
}

/// `NetCode.poll()` results.
const NOT_STARTED: i32 = -1;
const FINDING: i32 = 0;
const READY: i32 = 1;
const WAITING: i32 = 2;
const CONNECTED: i32 = 3;
const FAILED: i32 = 4;

impl NetCode {
    /// Takes a socket back after a match (`CodeLink::release`): the same code, waiting for a friend again.
    pub fn adopt(&mut self, sock: CodeSocket) {
        self.sock = Some(sock);
        self.their = None;
    }

    /// Takes the punched link out (once connected); the `NetCode` is spent after that.
    pub fn take_link(&mut self) -> Option<CodeLink> {
        let sock = self.sock.take()?;
        if matches!(sock.state(), CodeState::Connected(_)) {
            sock.into_link()
        } else {
            self.sock = Some(sock);
            None
        }
    }
}

#[godot_api]
impl NetCode {
    /// Opens the socket on UDP `port` (any free port if it is taken) and starts finding this machine's public address.
    /// `stun`: STUN servers as "host:port"; empty uses the built-in public list, `["none"]` skips STUN (LAN only, tests).
    /// `lan`: a LAN address to use instead of the system's (tests pass "127.0.0.1"), or "". Returns an error text or "".
    #[func]
    fn start(&mut self, port: i32, stun: PackedStringArray, lan: GString) -> GString {
        let mut opts = CodeOptions::default();
        let list: Vec<String> = stun.as_slice().iter().map(|s| s.to_string()).collect();
        if list.len() == 1 && list[0] == "none" {
            opts.stun_servers.clear();
        } else if !list.is_empty() {
            opts.stun_servers = list;
        }
        if let Ok(ip) = lan.to_string().parse::<Ipv4Addr>() {
            opts.lan_override = Some(ip);
        }
        self.their = None;
        match CodeSocket::bind(port.clamp(0, 65535) as u16, opts) {
            Ok(s) => {
                self.sock = Some(s);
                GString::new()
            }
            Err(e) => GString::from(format!("cannot open a network socket: {e}").as_str()),
        }
    }

    /// Call every frame. -1 not started, 0 finding the address, 1 code ready, 2 waiting for the friend (probing), 3 connected,
    /// 4 failed (nothing answered in time).
    #[func]
    fn poll(&mut self) -> i32 {
        let Some(s) = self.sock.as_mut() else {
            return NOT_STARTED;
        };
        s.poll();
        match s.state() {
            CodeState::FindingAddress => FINDING,
            CodeState::Ready => READY,
            CodeState::Punching => WAITING,
            CodeState::Connected(_) => CONNECTED,
            CodeState::Failed => FAILED,
        }
    }

    /// This player's code ("K7QD-M2XA-..."), or "" while it is being worked out.
    #[func]
    fn my_code(&self) -> GString {
        self.sock
            .as_ref()
            .and_then(CodeSocket::my_code)
            .map_or_else(GString::new, |c| GString::from(c.encode().as_str()))
    }

    /// False when no STUN server answered: the code then only works on the same network.
    #[func]
    fn public_known(&self) -> bool {
        self.sock
            .as_ref()
            .is_some_and(|s| s.public_addr().is_some())
    }

    /// True when this router gives each destination its own port (codes very likely fail).
    #[func]
    fn strict_nat(&self) -> bool {
        self.sock.as_ref().is_some_and(CodeSocket::strict_nat)
    }

    /// Checks a friend's code: "" if it is good, else a message for the player.
    #[func]
    fn check_code(&self, text: GString) -> GString {
        match ConnectionCode::decode(&text.to_string()) {
            Ok(c) if self.sock.as_ref().and_then(CodeSocket::my_code) == Some(c) => {
                GString::from("That is your own code: paste your friend's code instead.")
            }
            Ok(_) => GString::new(),
            Err(e) => GString::from(e.to_string().as_str()),
        }
    }

    /// True when this player would host against `text` (the lower code hosts and sets the rules).
    #[func]
    fn hosts_against(&self, text: GString) -> bool {
        match (
            ConnectionCode::decode(&text.to_string()),
            self.sock.as_ref(),
        ) {
            (Ok(c), Some(s)) => s.hosts_against(&c),
            _ => false,
        }
    }

    /// Starts probing the friend with code `text`. Returns an error text or "".
    #[func]
    fn connect_to(&mut self, text: GString) -> GString {
        let err = self.check_code(text.clone());
        if !err.is_empty() {
            return err;
        }
        let Some(s) = self.sock.as_mut() else {
            return GString::from("not started");
        };
        let Ok(c) = ConnectionCode::decode(&text.to_string()) else {
            return GString::from("bad code");
        };
        if s.my_code().is_none() {
            return GString::from("Still finding your address: wait a moment.");
        }
        let _ = s.connect(c);
        self.their = Some(c);
        GString::new()
    }

    /// Stops probing (the code stays the same).
    #[func]
    fn cancel(&mut self) {
        if let Some(s) = self.sock.as_mut() {
            s.cancel();
        }
    }

    /// Whole seconds left before probing gives up.
    #[func]
    fn seconds_left(&self) -> i32 {
        self.sock
            .as_ref()
            .map_or(0, |s| s.time_left().as_secs() as i32)
    }

    /// Whether this player hosts the connected match.
    #[func]
    fn i_host(&self) -> bool {
        match (self.sock.as_ref(), self.their) {
            (Some(s), Some(c)) => s.hosts_against(&c),
            _ => false,
        }
    }

    /// Closes the socket.
    #[func]
    fn stop(&mut self) {
        self.sock = None;
        self.their = None;
    }
}
