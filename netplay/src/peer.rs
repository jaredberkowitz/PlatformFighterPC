//! A whole network participant: a [`Link`] (something that sends and receives datagrams), the handshake, and the
//! rollback session once the handshake succeeds. This is the glue the game and the tools share, so everything above the
//! socket behaves identically in tests and in play.
//!
//! Call [`Peer::update`] once per frame with the local input; it receives, handshakes or simulates, and sends.
//!
//! **Rematches.** When a match is over, either player calls [`Peer::request_rematch`]; once both have, each side starts a
//! fresh handshake over the same link (the host picks a new seed) and a new match begins. Every datagram carries a one-byte
//! *epoch* (the number of matches this link has played), so late packets of the finished match can never reach the next one.
//!
//! **The lobby** (a peer made with [`Peer::with_lobby`]). Before each match the two players choose a fighter and a stage, each
//! locking the choice in; both see the other's choices as they change ([`Peer::lobby_view`]). Each side keeps sending its whole
//! choice ([`LobbyPick`]) a few times a second, so a lost datagram only delays things. Once both have locked both, the host draws
//! the stage (their common choice, or one of the two at random) and the handshake starts with those fighters, as it would without a
//! lobby. A rematch goes back to the lobby with the last choices selected but not locked.

use crate::handshake::{BaseCounts, Handshake, Outcome};
use crate::packet::MAGIC;
use crate::packet::{RejectReason, Setup};
use crate::session::{Advance, Event, Session, SessionConfig, Stats};
use sim_content::recipe::{match_content_on, FighterSpec};
use sim_core::{Content, GameState, Input, SIM_VERSION};

/// Datagram type of the rematch request. Handled here; the session and handshake never see it.
const T_REMATCH: u8 = 8;
/// Ticks between resends of a rematch request.
const REMATCH_EVERY: u32 = 6;
/// Datagram type of a lobby choice ([`LobbyPick`]). Handled here, like the rematch request.
const T_LOBBY: u8 = 9;
/// Ticks between resends of this side's lobby choice (it is also sent at once whenever it changes).
const LOBBY_EVERY: u32 = 10;
/// The most bytes a fighter spec or cosmetics may take in a lobby datagram.
const LOBBY_FIELD_MAX: usize = 64;

/// One player's choices in the lobby: the fighter (`sim_content::recipe::FighterSpec` bytes) with its look and name (cosmetics,
/// opaque), and the stage (an index into `sim_content::stages`), each locked in or not.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LobbyPick {
    pub fighter: Vec<u8>,
    pub cosmetics: Vec<u8>,
    pub fighter_locked: bool,
    pub stage: u8,
    pub stage_locked: bool,
}

impl LobbyPick {
    /// Both choices locked in.
    pub fn ready(&self) -> bool {
        self.fighter_locked && self.stage_locked
    }
}

/// What the lobby shows: this side's choices, and the other side's as last heard (`None` until they arrive).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LobbyView {
    pub mine: LobbyPick,
    pub theirs: Option<LobbyPick>,
}

const LOBBY_FIGHTER_LOCKED: u8 = 1;
const LOBBY_STAGE_LOCKED: u8 = 2;
/// The sender has left (back to the menus): the other side ends the lobby.
const LOBBY_LEFT: u8 = 4;

/// A lobby datagram: magic, type, the sender's sim version and base content hash (so two different builds find out at once),
/// flags, stage, then the fighter and cosmetics, each with a length byte.
fn encode_lobby(pick: &LobbyPick, version: u16, hash: u64, left: bool) -> Vec<u8> {
    let mut out = MAGIC.to_le_bytes().to_vec();
    out.push(T_LOBBY);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&hash.to_le_bytes());
    let mut flags = 0;
    if pick.fighter_locked {
        flags |= LOBBY_FIGHTER_LOCKED;
    }
    if pick.stage_locked {
        flags |= LOBBY_STAGE_LOCKED;
    }
    if left {
        flags |= LOBBY_LEFT;
    }
    out.push(flags);
    out.push(pick.stage);
    for field in [&pick.fighter, &pick.cosmetics] {
        let n = field.len().min(LOBBY_FIELD_MAX);
        out.push(n as u8);
        out.extend_from_slice(&field[..n]);
    }
    out
}

/// A decoded lobby datagram: (the sender's version, its hash, its choice, whether it has left).
fn decode_lobby(bytes: &[u8]) -> Option<(u16, u64, LobbyPick, bool)> {
    if bytes.len() < 17 || bytes[..2] != MAGIC.to_le_bytes() || bytes[2] != T_LOBBY {
        return None;
    }
    let version = u16::from_le_bytes([bytes[3], bytes[4]]);
    let hash = u64::from_le_bytes(bytes[5..13].try_into().ok()?);
    let flags = bytes[13];
    let stage = bytes[14];
    let mut at = 15;
    let mut fields = [Vec::new(), Vec::new()];
    for f in &mut fields {
        let n = usize::from(*bytes.get(at)?);
        at += 1;
        if n > LOBBY_FIELD_MAX {
            return None;
        }
        *f = bytes.get(at..at + n)?.to_vec();
        at += n;
    }
    let [fighter, cosmetics] = fields;
    Some((
        version,
        hash,
        LobbyPick {
            fighter,
            cosmetics,
            fighter_locked: flags & LOBBY_FIGHTER_LOCKED != 0,
            stage,
            stage_locked: flags & LOBBY_STAGE_LOCKED != 0,
        },
        flags & LOBBY_LEFT != 0,
    ))
}

/// The lobby between matches (see the module notes).
#[derive(Clone, Debug, Default)]
struct Lobby {
    view: LobbyView,
    ticks: u32,
    /// This side's choice changed since it was last sent.
    dirty: bool,
}

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
    /// In the lobby: the players are choosing (see [`Peer::lobby_view`]).
    Lobby,
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
    /// Whether matches start from a lobby (`with_lobby`), the lobby while it is open, and the choice it ended with (resent while
    /// the other side may still be in the lobby, and selected again in the next one).
    uses_lobby: bool,
    lobby: Option<Lobby>,
    last_pick: Option<LobbyPick>,
    lobby_ticks: u32,
    /// The other side left, or runs a different build (found from its lobby datagrams).
    lobby_rejected: Option<(RejectReason, u16, u64)>,
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
            uses_lobby: false,
            lobby: None,
            last_pick: None,
            lobby_ticks: 0,
            lobby_rejected: None,
        }
    }

    /// Starts in the lobby instead of handshaking at once: the players choose fighters and a stage first (see the module notes),
    /// and every rematch comes back to it. The host's other settings (rules, delay, ranked) stay those it was made with.
    pub fn with_lobby(mut self, pick: LobbyPick) -> Peer<L> {
        self.uses_lobby = true;
        self.lobby = Some(Lobby {
            view: LobbyView {
                mine: pick,
                theirs: None,
            },
            ticks: 0,
            dirty: true,
        });
        self
    }

    /// The lobby's choices while it is open (`None` during a match).
    pub fn lobby_view(&self) -> Option<&LobbyView> {
        self.lobby.as_ref().map(|l| &l.view)
    }

    /// Changes this side's lobby choice (ignored outside the lobby).
    pub fn set_lobby_pick(&mut self, pick: LobbyPick) {
        if let Some(l) = &mut self.lobby {
            if l.view.mine != pick {
                l.view.mine = pick;
                l.dirty = true;
            }
        }
    }

    /// The stage the host draws for two choices: the common one, else one of the two by the seed.
    pub fn draw_stage(mine: u8, theirs: u8, seed: u64) -> u8 {
        if mine == theirs || (seed >> 33) & 1 == 0 {
            mine
        } else {
            theirs
        }
    }

    /// Both players have locked in: the lobby closes and the match's handshake starts with their choices.
    fn close_lobby(&mut self) {
        let Some(lobby) = self.lobby.take() else {
            return;
        };
        let mine = lobby.view.mine;
        let theirs = lobby.view.theirs.unwrap_or_default();
        self.handshake = if self.host {
            let mut setup = self.host_setup.clone().unwrap_or_default();
            setup.fighter = mine.fighter.clone();
            setup.cosmetics = mine.cosmetics.clone();
            setup.stage = Peer::<L>::draw_stage(mine.stage, theirs.stage, setup.seed);
            self.host_setup = Some(setup.clone());
            Handshake::host(SIM_VERSION, self.base_hash, self.base_counts, setup)
        } else {
            self.join_info = Some((mine.cosmetics.clone(), mine.fighter.clone()));
            Handshake::join(
                SIM_VERSION,
                self.base_hash,
                self.base_counts,
                mine.cosmetics.clone(),
                mine.fighter.clone(),
            )
        };
        self.last_pick = Some(mine);
        self.lobby_ticks = 0;
    }

    fn send_lobby(&mut self, pick: &LobbyPick, left: bool) {
        let bytes = encode_lobby(pick, SIM_VERSION, self.base_hash, left);
        self.send(&bytes);
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
        if let Some(setup) = &mut self.host_setup {
            // A new seed for the new match, the same on every run of the same host.
            setup.seed = setup
                .seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
        }
        if self.uses_lobby {
            // Back to the lobby, the last choices selected but not locked.
            let mut pick = self.last_pick.clone().unwrap_or_default();
            pick.fighter_locked = false;
            pick.stage_locked = false;
            self.lobby = Some(Lobby {
                view: LobbyView {
                    mine: pick,
                    theirs: None,
                },
                ticks: 0,
                dirty: true,
            });
            return;
        }
        self.handshake = if self.host {
            let setup = self.host_setup.clone().unwrap_or_default();
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

    /// The link back, done with this peer (call [`Peer::leave`] first to tell the other side).
    pub fn into_link(self) -> L {
        self.link
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
        if self.uses_lobby {
            // (Whether in the lobby or in a match: the other side may be in either.)
            let pick = self
                .lobby
                .as_ref()
                .map(|l| l.view.mine.clone())
                .or_else(|| self.last_pick.clone())
                .unwrap_or_default();
            for _ in 0..3 {
                self.send_lobby(&pick, true);
            }
        }
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
            if let Some((version, hash, pick, left)) = decode_lobby(&bytes) {
                if left {
                    if self.lobby_rejected.is_none() && self.session.is_none() {
                        self.events.push(Event::Disconnected {
                            player: u8::from(self.host),
                        });
                    }
                    self.lobby_rejected = Some((RejectReason::Left, version, hash));
                } else if version != SIM_VERSION {
                    self.lobby_rejected = Some((RejectReason::SimVersion, version, hash));
                } else if hash != self.base_hash {
                    self.lobby_rejected = Some((RejectReason::ContentHash, version, hash));
                } else if let Some(l) = &mut self.lobby {
                    l.view.theirs = Some(pick);
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

        if let Some((reason, their_version, their_hash)) = self.lobby_rejected {
            if self.session.is_none() {
                return Status::Rejected {
                    reason,
                    their_version,
                    their_hash,
                };
            }
        }
        // The lobby: send this side's choice when it changes and every few ticks; close it once both have locked in.
        if let Some(mut lobby) = self.lobby.take() {
            lobby.ticks += 1;
            let send = lobby.dirty || lobby.ticks % LOBBY_EVERY == 1;
            lobby.dirty = false;
            let pick = lobby.view.mine.clone();
            let both = pick.ready() && lobby.view.theirs.as_ref().is_some_and(LobbyPick::ready);
            self.lobby = Some(lobby);
            if send || both {
                self.send_lobby(&pick, false);
            }
            if !both {
                return Status::Lobby;
            }
            self.close_lobby();
        } else if self.uses_lobby && !self.heard_from_remote {
            // The other side may still be in the lobby, waiting to hear that this one locked in.
            self.lobby_ticks += 1;
            if self.lobby_ticks % LOBBY_EVERY == 1 {
                if let Some(pick) = self.last_pick.clone() {
                    self.send_lobby(&pick, false);
                }
            }
        }

        // Start the match when the handshake succeeds.
        if self.session.is_none() {
            if let Outcome::Ready(setup) = self.handshake.outcome().clone() {
                // Both sides build the same content from the two fighters' specs and the chosen stage.
                let specs: Option<Vec<FighterSpec>> = match self.handshake.fighter_specs() {
                    Some((host, joiner)) => [host, joiner]
                        .iter()
                        .map(|b| FighterSpec::decode(b))
                        .collect(),
                    None => Some(Vec::new()),
                };
                if self.handshake.fighter_specs().is_some() || setup.stage != 0 {
                    let built = specs.and_then(|s| {
                        match_content_on(content, &s, setup.ranked, setup.stage).ok()
                    });
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
