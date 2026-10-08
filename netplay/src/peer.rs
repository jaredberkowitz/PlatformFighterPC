//! A whole network participant: a [`Link`] (something that sends and receives datagrams), the handshake, and the
//! rollback session once the handshake succeeds. This is the glue the game and the tools share, so everything above the
//! socket behaves identically in tests and in play.
//!
//! Call [`Peer::update`] once per frame with the local input; it receives, handshakes or simulates, and sends.
//!
//! **Rematches.** When a match is over, either player calls [`Peer::request_rematch`]; once both have, each side starts a
//! fresh handshake over the same link (the host picks a new seed) and a new match begins. Every datagram carries a one-byte
//! *epoch* (the number of matches this link has played), so late packets of the finished match can never reach the next one.

use crate::handshake::{BaseCounts, Handshake, Outcome};
use crate::packet::MAGIC;
use crate::packet::{RejectReason, Setup};
use crate::session::{Advance, Event, Session, SessionConfig, Stats};
use sim_content::recipe::{match_content, FighterSpec};
use sim_core::{Content, GameState, Input, SIM_VERSION};

/// Datagram type of the rematch request. Handled here; the session and handshake never see it.
const T_REMATCH: u8 = 8;
/// Ticks between resends of a rematch request.
const REMATCH_EVERY: u32 = 6;

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
    /// The content of this match when players brought made fighters: the base roster plus theirs. `None` means the
    /// base content passed to `update` is the match's content.
    match_content: Option<Content>,
    /// How many matches this link has played; tags every datagram.
    epoch: u8,
    /// The settings the running match started with (host and joiner agree on them).
    started: Option<Setup>,
    wants_rematch: bool,
    remote_wants_rematch: bool,
    rematch_ticks: u32,
    /// What a fresh handshake needs: the host's settings, or the joiner's cosmetics and fighter, and the base content.
    host_setup: Option<Setup>,
    join_info: Option<(Vec<u8>, Vec<u8>)>,
    base_hash: u64,
    base_counts: BaseCounts,
}

impl<L: Link> Peer<L> {
    /// The host decides the match settings and is player 0.
    pub fn host(link: L, content: &Content, setup: Setup) -> Peer<L> {
        let mut peer = Peer::new(
            link,
            Handshake::host(SIM_VERSION, content.hash(), counts(content), setup.clone()),
            true,
            content,
        );
        peer.host_setup = Some(setup);
        peer
    }

    /// The joiner is player 1 and takes whatever the host decided, if the versions match.
    pub fn join(link: L, content: &Content, cosmetics: Vec<u8>) -> Peer<L> {
        Peer::join_with_fighter(link, content, cosmetics, Vec::new())
    }

    /// Like [`Peer::join`], bringing a fighter (`FighterSpec` bytes) to the match.
    pub fn join_with_fighter(
        link: L,
        content: &Content,
        cosmetics: Vec<u8>,
        fighter: Vec<u8>,
    ) -> Peer<L> {
        let mut peer = Peer::new(
            link,
            Handshake::join(
                SIM_VERSION,
                content.hash(),
                counts(content),
                cosmetics.clone(),
                fighter.clone(),
            ),
            false,
            content,
        );
        peer.join_info = Some((cosmetics, fighter));
        peer
    }

    fn new(link: L, handshake: Handshake, host: bool, content: &Content) -> Peer<L> {
        Peer {
            link,
            handshake,
            host,
            session: None,
            max_prediction: 8,
            checksum_interval: 30,
            heard_from_remote: false,
            events: Vec::new(),
            match_content: None,
            epoch: 0,
            started: None,
            wants_rematch: false,
            remote_wants_rematch: false,
            rematch_ticks: 0,
            host_setup: None,
            join_info: None,
            base_hash: content.hash(),
            base_counts: counts(content),
        }
    }

    /// The settings of the match in progress (seed, characters as agreed, rules...), once it has started.
    pub fn match_setup(&self) -> Option<&Setup> {
        self.started.as_ref()
    }

    /// Both players' cosmetic bytes in player order `[host, joiner]` (opaque), once the handshake has them.
    pub fn player_cosmetics(&self) -> [Vec<u8>; 2] {
        let theirs = self.handshake.their_cosmetics().to_vec();
        let mine = if self.host {
            self.host_setup
                .as_ref()
                .map(|s| s.cosmetics.clone())
                .unwrap_or_default()
        } else {
            self.join_info
                .as_ref()
                .map(|(c, _)| c.clone())
                .unwrap_or_default()
        };
        if self.host {
            [mine, theirs]
        } else {
            [theirs, mine]
        }
    }

    /// Asks for another match with the same opponent. It starts when both sides have asked.
    pub fn request_rematch(&mut self) {
        if self.session.is_some() {
            self.wants_rematch = true;
        }
    }

    /// `(this side has asked, the other side has asked)`.
    pub fn rematch_state(&self) -> (bool, bool) {
        (self.wants_rematch, self.remote_wants_rematch)
    }

    /// Sends one datagram, tagged with the epoch.
    fn send(&mut self, bytes: &[u8]) {
        let mut tagged = Vec::with_capacity(bytes.len() + 1);
        tagged.extend_from_slice(bytes);
        tagged.push(self.epoch);
        self.link.send(&tagged);
    }

    /// The next datagram of the current epoch. A request for the next epoch's handshake from someone we are also waiting
    /// to rematch with counts as their request; anything else from another epoch is dropped.
    fn recv(&mut self) -> Option<Vec<u8>> {
        while let Some(mut bytes) = self.link.recv() {
            let Some(tag) = bytes.pop() else {
                continue;
            };
            if tag == self.epoch {
                return Some(bytes);
            }
            if tag == self.epoch.wrapping_add(1) && self.wants_rematch {
                self.remote_wants_rematch = true;
            }
        }
        None
    }

    /// Both sides want another match: forget the finished one and handshake again, over the same link.
    fn begin_rematch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.wants_rematch = false;
        self.remote_wants_rematch = false;
        self.session = None;
        self.started = None;
        self.match_content = None;
        self.heard_from_remote = false;
        self.events.clear();
        self.handshake = if self.host {
            let mut setup = self.host_setup.clone().unwrap_or_default();
            // A new seed for the new match, the same on every run of the same host.
            setup.seed = setup
                .seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.host_setup = Some(setup.clone());
            Handshake::host(SIM_VERSION, self.base_hash, self.base_counts, setup)
        } else {
            let (cosmetics, fighter) = self.join_info.clone().unwrap_or_default();
            Handshake::join(
                SIM_VERSION,
                self.base_hash,
                self.base_counts,
                cosmetics,
                fighter,
            )
        };
    }

    /// The two players' fighter specs as `(host's, joiner's)`, once the handshake has them.
    pub fn fighter_specs(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.handshake.fighter_specs()
    }

    /// The content the match runs on once it has started with made fighters in it (otherwise `None`: the base content).
    /// The game needs it to draw the fighters and read their numbers.
    pub fn match_content(&self) -> Option<&Content> {
        self.match_content.as_ref()
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
            let outgoing = s.drain_outgoing();
            for p in outgoing {
                self.send(&p);
            }
        }
    }

    pub fn update(&mut self, content: &Content, local: Input) -> Status {
        // Receive everything that has arrived.
        while let Some(bytes) = self.recv() {
            if bytes.len() == 3 && bytes[..2] == MAGIC.to_le_bytes() && bytes[2] == T_REMATCH {
                if self.session.is_some() {
                    self.remote_wants_rematch = true;
                }
                continue;
            }
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

        if self.wants_rematch && self.remote_wants_rematch {
            self.begin_rematch();
        } else if self.wants_rematch {
            self.rematch_ticks += 1;
            if self.rematch_ticks % REMATCH_EVERY == 1 {
                let mut request = MAGIC.to_le_bytes().to_vec();
                request.push(T_REMATCH);
                self.send(&request);
            }
        }

        // Start the match when the handshake succeeds.
        if self.session.is_none() {
            if let Outcome::Ready(setup) = self.handshake.outcome().clone() {
                // Both sides build the same content from the two fighters' specs.
                if let Some((host, joiner)) = self.handshake.fighter_specs() {
                    let specs: Option<Vec<FighterSpec>> = [host, joiner]
                        .iter()
                        .map(|b| FighterSpec::decode(b))
                        .collect();
                    let built = specs.and_then(|s| match_content(content, &s, setup.ranked).ok());
                    match built {
                        Some((c, _)) => self.match_content = Some(c),
                        None => {
                            return Status::Rejected {
                                reason: RejectReason::BadFighter,
                                their_version: SIM_VERSION,
                                their_hash: 0,
                            }
                        }
                    }
                }
                let content = self.match_content.as_ref().unwrap_or(content);
                let cfg = SessionConfig {
                    local: u8::from(!self.host),
                    active: setup.active,
                    input_delay: setup.input_delay,
                    max_prediction: self.max_prediction,
                    checksum_interval: self.checksum_interval,
                };
                let initial = GameState::new_with_rules(
                    content,
                    setup.seed,
                    setup.chars,
                    setup.active,
                    setup.rules,
                );
                self.started = Some(setup.clone());
                self.session = Some(Session::new(cfg, initial));
            }
        }

        // The handshake keeps talking until the other side's inputs show it heard us (a lost reply must not
        // strand either side).
        if !self.heard_from_remote {
            for p in self.handshake.tick() {
                self.send(&p);
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
        let advance = session.advance(self.match_content.as_ref().unwrap_or(content), local);
        let outgoing = session.drain_outgoing();
        for p in outgoing {
            self.send(&p);
        }
        Status::Running(advance)
    }
}

fn counts(content: &Content) -> BaseCounts {
    BaseCounts {
        fighters: content.fighters.len(),
        weapons: content.weapons.len(),
    }
}
