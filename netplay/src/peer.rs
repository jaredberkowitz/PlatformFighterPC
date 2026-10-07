//! A whole network participant: a [`Link`] (something that sends and receives datagrams), the handshake, and the
//! rollback session once the handshake succeeds. This is the glue the game and the tools share, so everything above the
//! socket behaves identically in tests and in play.
//!
//! Call [`Peer::update`] once per frame with the local input; it receives, handshakes or simulates, and sends.

use crate::handshake::{Handshake, Outcome};
use crate::packet::{RejectReason, Setup};
use crate::session::{Advance, Event, Session, SessionConfig, Stats};
use sim_core::{Content, GameState, Input, SIM_VERSION};

/// Anything that can move datagrams: a UDP socket, a relay connection, or a simulated lossy link in tests.
pub trait Link {
    fn send(&mut self, bytes: &[u8]);
    /// The next datagram that has arrived, if any.
    fn recv(&mut self) -> Option<Vec<u8>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Still agreeing on version and settings.
    Connecting,
    /// In a match; whether this call simulated a frame or stalled waiting for the other side.
    Running(Advance),
    /// The other side runs a different sim version or content: no match.
    Rejected {
        reason: RejectReason,
        their_version: u16,
        their_hash: u64,
    },
}

pub struct Peer<L: Link> {
    link: L,
    handshake: Handshake,
    host: bool,
    session: Option<Session>,
    max_prediction: u8,
    checksum_interval: u32,
    heard_from_remote: bool,
    events: Vec<Event>,
}

impl<L: Link> Peer<L> {
    /// The host decides the match settings and is player 0.
    pub fn host(link: L, content: &Content, setup: Setup) -> Peer<L> {
        Peer::new(
            link,
            Handshake::host(SIM_VERSION, content.hash(), setup),
            true,
        )
    }

    /// The joiner is player 1 and takes whatever the host decided, if the versions match.
    pub fn join(link: L, content: &Content, cosmetics: Vec<u8>) -> Peer<L> {
        Peer::new(
            link,
            Handshake::join(SIM_VERSION, content.hash(), cosmetics),
            false,
        )
    }

    fn new(link: L, handshake: Handshake, host: bool) -> Peer<L> {
        Peer {
            link,
            handshake,
            host,
            session: None,
            max_prediction: 8,
            checksum_interval: 30,
            heard_from_remote: false,
            events: Vec::new(),
        }
    }

    /// How far the sim may run ahead of confirmed remote input (default 8 frames).
    pub fn with_max_prediction(mut self, frames: u8) -> Peer<L> {
        self.max_prediction = frames;
        self
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn state(&self) -> Option<&GameState> {
        self.session.as_ref().map(Session::state)
    }

    pub fn local_player(&self) -> usize {
        usize::from(!self.host)
    }

    pub fn link_mut(&mut self) -> &mut L {
        &mut self.link
    }

    pub fn stats(&self) -> Stats {
        self.session
            .as_ref()
            .map(Session::stats)
            .unwrap_or_default()
    }

    /// The other side's cosmetic bytes (opaque).
    pub fn their_cosmetics(&self) -> &[u8] {
        self.handshake.their_cosmetics()
    }

    /// Desyncs and disconnects since the last call.
    pub fn drain_events(&mut self) -> Vec<Event> {
        let mut out = std::mem::take(&mut self.events);
        if let Some(s) = &mut self.session {
            out.extend(s.drain_events());
        }
        out
    }

    /// Tells the other side this peer is leaving.
    pub fn leave(&mut self) {
        if let Some(s) = &mut self.session {
            s.disconnect();
            for p in s.drain_outgoing() {
                self.link.send(&p);
            }
        }
    }

    pub fn update(&mut self, content: &Content, local: Input) -> Status {
        // Receive everything that has arrived.
        while let Some(bytes) = self.link.recv() {
            self.handshake.handle_packet(&bytes);
            if let Some(s) = &mut self.session {
                if !self.heard_from_remote
                    && crate::packet::Packet::decode(&bytes)
                        .is_some_and(|p| matches!(p, crate::packet::Packet::Inputs { .. }))
                {
                    self.heard_from_remote = true;
                }
                s.handle_packet(&bytes);
            }
        }

        // Start the match when the handshake succeeds.
        if self.session.is_none() {
            if let Outcome::Ready(setup) = self.handshake.outcome().clone() {
                let cfg = SessionConfig {
                    local: u8::from(!self.host),
                    active: setup.active,
                    input_delay: setup.input_delay,
                    max_prediction: self.max_prediction,
                    checksum_interval: self.checksum_interval,
                };
                let initial =
                    GameState::new_with_active(content, setup.seed, setup.chars, setup.active);
                self.session = Some(Session::new(cfg, initial));
            }
        }

        // The handshake keeps talking until the other side's inputs show it heard us (a lost reply must not
        // strand either side).
        if !self.heard_from_remote {
            for p in self.handshake.tick() {
                self.link.send(&p);
            }
        }

        if let Outcome::Rejected {
            reason,
            their_version,
            their_hash,
        } = self.handshake.outcome()
        {
            return Status::Rejected {
                reason: *reason,
                their_version: *their_version,
                their_hash: *their_hash,
            };
        }
        let Some(session) = &mut self.session else {
            return Status::Connecting;
        };
        let advance = session.advance(content, local);
        for p in session.drain_outgoing() {
            self.link.send(&p);
        }
        Status::Running(advance)
    }
}
