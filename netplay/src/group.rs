//! Matches of three or four players over the network.
//!
//! The host is a **hub**: every guest has one link to the host and nothing else, and every rollback session packet (a player's inputs, a
//! checksum, a goodbye) travels inside a small envelope that says who it is for. The host handles what is addressed to it and forwards the rest
//! to the right guest, so the guests need no addresses of each other and no NAT traversal between themselves. The sessions themselves are the
//! ordinary [`Session`] with one packet per remote player; this module is only the lobby, the handshake and the routing.
//!
//! * **Lobby**: guests send `Hello` (sim version, base roster hash, their fighter, their name and look) until the host answers with a
//!   `Welcome` (their slot, 1 to 3, and who is in the lobby). The host starts the match when it chooses.
//! * **Start**: the host builds the match content from everyone's fighters and the stage, and sends each guest a `Setup` (seed, rules, who is
//!   playing, every fighter and name, the guest's own slot). The guest checks it (it works out the fighter numbers itself and insists on the
//!   host's) and answers `Ready`; when everyone has, all sessions start.
//! * **Rematch**: the host restarts with a new seed and a higher *epoch*; guests follow. Every envelope carries the epoch so packets of a
//!   finished match never reach the next one.
//!
//! Pure like the rest of this crate: sockets are behind [`HubLink`] (the host) and [`crate::peer::Link`] (a guest).

use crate::handshake::BaseCounts;
use crate::packet::{RejectReason, MAGIC};
use crate::peer::Link;
use crate::session::{Advance, Event, Session, SessionConfig, BROADCAST};
use sim_content::recipe::{match_content_on, resolve, FighterSpec};
use sim_core::{Content, GameState, Input, MatchRules, MAX_FIGHTERS, SIM_VERSION};

/// The host's side of the sockets: one numbered conversation per guest (guest `i` plays slot `i + 1`).
pub trait HubLink {
    fn send(&mut self, guest: usize, bytes: &[u8]);
    /// The next datagram from any guest, with the guest's number.
    fn recv(&mut self) -> Option<(usize, Vec<u8>)>;
}

const K_HELLO: u8 = 0x30;
const K_WELCOME: u8 = 0x31;
const K_SETUP: u8 = 0x32;
const K_READY: u8 = 0x33;
const K_REJECT: u8 = 0x34;
const K_RELAY: u8 = 0x35;
const K_LEAVE: u8 = 0x36;

/// Most guests a host serves (the host is the fourth player).
pub const MAX_GUESTS: usize = MAX_FIGHTERS - 1;
const RESEND_EVERY: u32 = 8;
const LOBBY_EVERY: u32 = 30;
const MAX_BLOB: usize = 600;

// ---- Wire ---------------------------------------------------------------------------------------------------------------

struct W(Vec<u8>);

impl W {
    fn new(kind: u8) -> W {
        let mut v = MAGIC.to_le_bytes().to_vec();
        v.push(kind);
        W(v)
    }
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.u16(b.len().min(MAX_BLOB) as u16);
        self.0.extend_from_slice(&b[..b.len().min(MAX_BLOB)]);
    }
}

struct R<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.b.len())?;
        let out = &self.b[self.at..end];
        self.at = end;
        Some(out)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u16(&mut self) -> Option<u16> {
        let b = self.take(2)?;
        Some(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = usize::from(self.u16()?);
        if n > MAX_BLOB {
            return None;
        }
        Some(self.take(n)?.to_vec())
    }
}

/// What the host decided for one match, as each player receives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupSetup {
    pub epoch: u8,
    pub seed: u64,
    pub input_delay: u8,
    pub rules: MatchRules,
    pub stage: u8,
    pub ranked: bool,
    /// Bit `n` set means player `n` takes part.
    pub active: u8,
    /// The receiving player's slot.
    pub slot: u8,
    /// Fighter numbers in the match content, by slot.
    pub chars: [u8; MAX_FIGHTERS],
    /// Fighter spec bytes of the players taking part, in slot order.
    pub specs: Vec<Vec<u8>>,
    /// Name and look bytes of each slot (empty for a slot not taking part).
    pub cosmetics: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Msg {
    Hello {
        version: u16,
        base_hash: u64,
        fighter: Vec<u8>,
        cosmetics: Vec<u8>,
    },
    Welcome {
        slot: u8,
        cosmetics: Vec<Vec<u8>>,
    },
    Setup(GroupSetup),
    Ready {
        epoch: u8,
    },
    Reject(u8),
    Relay {
        epoch: u8,
        dest: u8,
        src: u8,
        payload: Vec<u8>,
    },
    Leave,
}

impl Msg {
    fn encode(&self) -> Vec<u8> {
        match self {
            Msg::Hello {
                version,
                base_hash,
                fighter,
                cosmetics,
            } => {
                let mut w = W::new(K_HELLO);
                w.u16(*version);
                w.u64(*base_hash);
                w.bytes(fighter);
                w.bytes(cosmetics);
                w.0
            }
            Msg::Welcome { slot, cosmetics } => {
                let mut w = W::new(K_WELCOME);
                w.u8(*slot);
                w.u8(cosmetics.len() as u8);
                for c in cosmetics {
                    w.bytes(c);
                }
                w.0
            }
            Msg::Setup(s) => {
                let mut w = W::new(K_SETUP);
                w.u8(s.epoch);
                w.u64(s.seed);
                w.u8(s.input_delay);
                w.u8(s.rules.stocks);
                w.u16(s.rules.time_limit);
                w.u8(s.stage);
                w.u8(u8::from(s.ranked));
                w.u8(s.active);
                w.u8(s.slot);
                for c in s.chars {
                    w.u8(c);
                }
                w.u8(s.specs.len() as u8);
                for b in &s.specs {
                    w.bytes(b);
                }
                w.u8(s.cosmetics.len() as u8);
                for b in &s.cosmetics {
                    w.bytes(b);
                }
                w.0
            }
            Msg::Ready { epoch } => {
                let mut w = W::new(K_READY);
                w.u8(*epoch);
                w.0
            }
            Msg::Reject(reason) => {
                let mut w = W::new(K_REJECT);
                w.u8(*reason);
                w.0
            }
            Msg::Relay {
                epoch,
                dest,
                src,
                payload,
            } => {
                let mut w = W::new(K_RELAY);
                w.u8(*epoch);
                w.u8(*dest);
                w.u8(*src);
                w.0.extend_from_slice(payload);
                w.0
            }
            Msg::Leave => W::new(K_LEAVE).0,
        }
    }

    fn decode(bytes: &[u8]) -> Option<Msg> {
        if bytes.len() < 3 || bytes[..2] != MAGIC.to_le_bytes() {
            return None;
        }
        let mut r = R { b: bytes, at: 3 };
        Some(match bytes[2] {
            K_HELLO => Msg::Hello {
                version: r.u16()?,
                base_hash: r.u64()?,
                fighter: r.bytes()?,
                cosmetics: r.bytes()?,
            },
            K_WELCOME => {
                let slot = r.u8()?;
                let n = usize::from(r.u8()?);
                if n > MAX_FIGHTERS {
                    return None;
                }
                let mut cosmetics = Vec::new();
                for _ in 0..n {
                    cosmetics.push(r.bytes()?);
                }
                Msg::Welcome { slot, cosmetics }
            }
            K_SETUP => {
                let epoch = r.u8()?;
                let seed = r.u64()?;
                let input_delay = r.u8()?;
                let rules = MatchRules {
                    stocks: r.u8()?,
                    time_limit: r.u16()?,
                };
                if rules != rules.clamped() {
                    return None;
                }
                let stage = r.u8()?;
                let ranked = r.u8()? == 1;
                let active = r.u8()?;
                let slot = r.u8()?;
                let mut chars = [0u8; MAX_FIGHTERS];
                for c in &mut chars {
                    *c = r.u8()?;
                }
                let n = usize::from(r.u8()?);
                if n > MAX_FIGHTERS || active == 0 || active > 0b1111 || slot >= 4 {
                    return None;
                }
                let mut specs = Vec::new();
                for _ in 0..n {
                    specs.push(r.bytes()?);
                }
                let m = usize::from(r.u8()?);
                if m > MAX_FIGHTERS {
                    return None;
                }
                let mut cosmetics = Vec::new();
                for _ in 0..m {
                    cosmetics.push(r.bytes()?);
                }
                Msg::Setup(GroupSetup {
                    epoch,
                    seed,
                    input_delay,
                    rules,
                    stage,
                    ranked,
                    active,
                    slot,
                    chars,
                    specs,
                    cosmetics,
                })
            }
            K_READY => Msg::Ready { epoch: r.u8()? },
            K_REJECT => Msg::Reject(r.u8()?),
            K_RELAY => {
                let epoch = r.u8()?;
                let dest = r.u8()?;
                let src = r.u8()?;
                Msg::Relay {
                    epoch,
                    dest,
                    src,
                    payload: bytes[r.at..].to_vec(),
                }
            }
            K_LEAVE => Msg::Leave,
            _ => return None,
        })
    }
}

/// Where a group peer is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupStatus {
    /// In the lobby (host: waiting for guests and for the start; guest: waiting for the host's setup).
    Lobby,
    /// The host is starting the match: setups are out, waiting for everyone to be ready.
    Starting,
    Running(Advance),
    /// The host refused this guest (another version or roster, a full lobby, a fighter that is not allowed) or a setup could not be built.
    Rejected(RejectReason),
}

fn setup_content(
    base: &Content,
    setup: &GroupSetup,
) -> Result<(Content, [u8; MAX_FIGHTERS]), RejectReason> {
    let specs: Option<Vec<FighterSpec>> =
        setup.specs.iter().map(|b| FighterSpec::decode(b)).collect();
    let specs = specs.ok_or(RejectReason::BadFighter)?;
    if specs.len() != setup.active.count_ones() as usize {
        return Err(RejectReason::BadFighter);
    }
    let (content, built) = match_content_on(base, &specs, setup.ranked, setup.stage)
        .map_err(|_| RejectReason::BadFighter)?;
    let mut chars = [0u8; MAX_FIGHTERS];
    let mut k = 0;
    for (slot, c) in chars.iter_mut().enumerate() {
        if setup.active >> slot & 1 == 1 {
            *c = built[k];
            k += 1;
        }
    }
    Ok((content, chars))
}

fn start_session(
    content: &Content,
    setup: &GroupSetup,
    chars: [u8; MAX_FIGHTERS],
    local: u8,
) -> Session {
    let initial = GameState::new_with_rules(content, setup.seed, chars, setup.active, setup.rules);
    let cfg = SessionConfig {
        local,
        active: setup.active,
        input_delay: setup.input_delay,
        max_prediction: 8,
        checksum_interval: 30,
    };
    Session::new(cfg, initial)
}

// ---- The host ----------------------------------------------------------------------------------------------------------------

/// Settings the host picks for the matches it hosts.
#[derive(Clone, Debug)]
pub struct GroupParams {
    pub seed: u64,
    pub input_delay: u8,
    pub rules: MatchRules,
    pub stage: u8,
    pub ranked: bool,
}

struct Guest {
    fighter: Vec<u8>,
    cosmetics: Vec<u8>,
    ready: bool,
}

pub struct GroupHost<H: HubLink> {
    hub: H,
    base_hash: u64,
    counts: BaseCounts,
    params: GroupParams,
    my_fighter: Vec<u8>,
    my_cosmetics: Vec<u8>,
    guests: [Option<Guest>; MAX_GUESTS],
    wants_start: bool,
    starting: bool,
    epoch: u8,
    setups: Vec<Option<GroupSetup>>,
    session: Option<Session>,
    match_content: Option<Content>,
    started: Option<GroupSetup>,
    ticks: u32,
    events: Vec<Event>,
    rejected: Option<RejectReason>,
}

impl<H: HubLink> GroupHost<H> {
    pub fn new(
        hub: H,
        base: &Content,
        params: GroupParams,
        fighter: Vec<u8>,
        cosmetics: Vec<u8>,
    ) -> GroupHost<H> {
        GroupHost {
            hub,
            base_hash: base.hash(),
            counts: BaseCounts {
                fighters: base.fighters.len(),
                weapons: base.weapons.len(),
            },
            params,
            my_fighter: fighter,
            my_cosmetics: cosmetics,
            guests: [None, None, None],
            wants_start: false,
            starting: false,
            epoch: 0,
            setups: Vec::new(),
            session: None,
            match_content: None,
            started: None,
            ticks: 0,
            events: Vec::new(),
            rejected: None,
        }
    }

    /// The name-and-look bytes of each slot (empty where nobody has joined), slot 0 being the host.
    pub fn lobby(&self) -> Vec<Vec<u8>> {
        let mut out = vec![self.my_cosmetics.clone()];
        for g in &self.guests {
            out.push(g.as_ref().map(|g| g.cosmetics.clone()).unwrap_or_default());
        }
        out
    }

    pub fn players_joined(&self) -> usize {
        1 + self.guests.iter().flatten().count()
    }

    /// Starts the match as soon as at least one guest has joined (the host presses its Start button).
    pub fn start(&mut self) {
        self.wants_start = true;
    }

    /// After a match: starts another with the same players, a new seed and a higher epoch.
    pub fn restart(&mut self) {
        if self.session.is_some() {
            self.session = None;
            self.match_content = None;
            self.started = None;
            self.epoch = self.epoch.wrapping_add(1);
            self.params.seed = self
                .params
                .seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            for g in self.guests.iter_mut().flatten() {
                g.ready = false;
            }
            self.wants_start = true;
        }
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn state(&self) -> Option<&GameState> {
        self.session.as_ref().map(Session::state)
    }

    pub fn match_content(&self) -> Option<&Content> {
        self.match_content.as_ref()
    }

    /// The settings of the running match (as the host sent them to slot 1).
    pub fn match_setup(&self) -> Option<&GroupSetup> {
        self.started.as_ref()
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        let mut out = std::mem::take(&mut self.events);
        if let Some(s) = &mut self.session {
            out.extend(s.drain_events());
        }
        out
    }

    pub fn leave(&mut self) {
        for (i, g) in self.guests.iter().enumerate() {
            if g.is_some() {
                self.hub.send(i, &Msg::Leave.encode());
            }
        }
    }

    fn reject(&mut self, guest: usize, reason: RejectReason) {
        self.hub.send(guest, &Msg::Reject(reason as u8).encode());
    }

    fn welcome(&mut self, guest: usize) {
        let msg = Msg::Welcome {
            slot: guest as u8 + 1,
            cosmetics: self.lobby(),
        };
        self.hub.send(guest, &msg.encode());
    }

    fn handle(&mut self, guest: usize, bytes: &[u8]) {
        let Some(msg) = Msg::decode(bytes) else {
            return;
        };
        match msg {
            Msg::Hello {
                version,
                base_hash,
                fighter,
                cosmetics,
            } => {
                if self.session.is_some() || self.starting {
                    return; // a match is on: late joiners wait for the next lobby
                }
                if version != SIM_VERSION || base_hash != self.base_hash {
                    self.reject(guest, RejectReason::SimVersion);
                    return;
                }
                let ok = FighterSpec::decode(&fighter).is_some()
                    && resolve(
                        self.counts.fighters,
                        self.counts.weapons,
                        &[FighterSpec::decode(&fighter).unwrap_or(FighterSpec::Builtin(0))],
                        self.params.ranked,
                    )
                    .is_ok();
                if !ok {
                    self.reject(guest, RejectReason::BadFighter);
                    return;
                }
                if guest >= MAX_GUESTS {
                    self.reject(guest, RejectReason::Full);
                    return;
                }
                let joined_before = self.guests[guest].is_some();
                self.guests[guest] = Some(Guest {
                    fighter,
                    cosmetics,
                    ready: false,
                });
                if !joined_before {
                    // Tell everyone who is here now.
                    for i in 0..MAX_GUESTS {
                        if self.guests[i].is_some() {
                            self.welcome(i);
                        }
                    }
                } else {
                    self.welcome(guest);
                }
            }
            Msg::Ready { epoch } => {
                if epoch == self.epoch && self.starting {
                    if let Some(g) = self.guests.get_mut(guest).and_then(Option::as_mut) {
                        g.ready = true;
                    }
                }
            }
            Msg::Relay {
                epoch,
                dest,
                src,
                payload,
            } => {
                if epoch != self.epoch || usize::from(src) != guest + 1 {
                    return; // another match's packet, or a guest pretending to be someone else
                }
                if dest == 0 || dest == BROADCAST {
                    if let Some(s) = &mut self.session {
                        s.handle_packet_from(src, &payload);
                    }
                }
                if dest != 0 {
                    let forward = Msg::Relay {
                        epoch,
                        dest,
                        src,
                        payload,
                    }
                    .encode();
                    for i in 0..MAX_GUESTS {
                        let wanted = dest == BROADCAST || usize::from(dest) == i + 1;
                        if wanted && i != guest && self.guests[i].is_some() {
                            self.hub.send(i, &forward);
                        }
                    }
                }
            }
            Msg::Leave => {
                if self.session.is_none() && !self.starting {
                    self.guests[guest] = None;
                } else if let Some(s) = &mut self.session {
                    s.handle_packet_from(
                        guest as u8 + 1,
                        &crate::packet::Packet::Disconnect.encode(),
                    );
                }
            }
            _ => {}
        }
    }

    /// Builds the match from everyone in the lobby and sends the setups.
    fn begin_start(&mut self, base: &Content) {
        let mut slots: Vec<usize> = vec![0];
        slots.extend(
            (0..MAX_GUESTS)
                .filter(|i| self.guests[*i].is_some())
                .map(|i| i + 1),
        );
        let mut specs: Vec<Vec<u8>> = vec![self.my_fighter.clone()];
        let mut cosmetics: Vec<Vec<u8>> = vec![self.my_cosmetics.clone()];
        for i in 0..MAX_GUESTS {
            match &self.guests[i] {
                Some(g) => {
                    specs.push(g.fighter.clone());
                    cosmetics.push(g.cosmetics.clone());
                }
                None => cosmetics.push(Vec::new()),
            }
        }
        // `cosmetics` is by slot (host, then each guest slot, empty where nobody is).
        let active: u8 = slots.iter().fold(0, |m, s| m | (1 << s));
        let mut setup = GroupSetup {
            epoch: self.epoch,
            seed: self.params.seed,
            input_delay: self.params.input_delay,
            rules: self.params.rules,
            stage: self.params.stage,
            ranked: self.params.ranked,
            active,
            slot: 0,
            chars: [0; MAX_FIGHTERS],
            specs,
            cosmetics,
        };
        match setup_content(base, &setup) {
            Ok((content, chars)) => {
                setup.chars = chars;
                self.match_content = Some(content);
                self.setups = (0..MAX_FIGHTERS)
                    .map(|slot| {
                        (slot > 0 && active >> slot & 1 == 1).then(|| {
                            let mut s = setup.clone();
                            s.slot = slot as u8;
                            s
                        })
                    })
                    .collect();
                self.started = Some(setup);
                self.starting = true;
                self.wants_start = false;
            }
            Err(reason) => {
                self.rejected = Some(reason);
                self.wants_start = false;
            }
        }
    }

    /// One tick: receive, run the lobby or the match, send. `content` is the base roster.
    pub fn update(&mut self, content: &Content, local: Input) -> GroupStatus {
        self.ticks += 1;
        while let Some((guest, bytes)) = self.hub.recv() {
            if guest < MAX_GUESTS || bytes.get(2) == Some(&K_HELLO) {
                self.handle(guest, &bytes);
            }
        }
        if let Some(reason) = self.rejected {
            return GroupStatus::Rejected(reason);
        }
        if self.session.is_none() && !self.starting {
            if self.ticks % LOBBY_EVERY == 0 {
                for i in 0..MAX_GUESTS {
                    if self.guests[i].is_some() {
                        self.welcome(i);
                    }
                }
            }
            if self.wants_start && self.guests.iter().any(Option::is_some) {
                self.begin_start(content);
            }
            if !self.starting {
                return match self.rejected {
                    Some(r) => GroupStatus::Rejected(r),
                    None => GroupStatus::Lobby,
                };
            }
        }
        if self.starting && self.session.is_none() {
            if self.ticks % RESEND_EVERY == 0 {
                for i in 0..MAX_GUESTS {
                    let waiting = self.guests[i].as_ref().is_some_and(|g| !g.ready);
                    if waiting {
                        if let Some(Some(s)) = self.setups.get(i + 1) {
                            self.hub.send(i, &Msg::Setup(s.clone()).encode());
                        }
                    }
                }
            }
            let everyone = self.guests.iter().flatten().all(|g| g.ready);
            if !everyone {
                return GroupStatus::Starting;
            }
            let (Some(setup), Some(match_content)) =
                (self.started.clone(), self.match_content.as_ref())
            else {
                return GroupStatus::Starting;
            };
            self.session = Some(start_session(match_content, &setup, setup.chars, 0));
            self.starting = false;
        }
        let Some(session) = self.session.as_mut() else {
            return GroupStatus::Lobby;
        };
        let content = self.match_content.as_ref().unwrap_or(content);
        let advance = session.advance(content, local);
        let epoch = self.epoch;
        let outgoing = session.drain_outgoing_to();
        for (dest, payload) in outgoing {
            for i in 0..MAX_GUESTS {
                let wanted = dest == BROADCAST || usize::from(dest) == i + 1;
                if wanted && self.guests[i].is_some() {
                    let relay = Msg::Relay {
                        epoch,
                        dest,
                        src: 0,
                        payload: payload.clone(),
                    };
                    self.hub.send(i, &relay.encode());
                }
            }
        }
        GroupStatus::Running(advance)
    }
}

// ---- A guest -----------------------------------------------------------------------------------------------------------------

pub struct GroupGuest<L: Link> {
    link: L,
    version_hash: (u16, u64),
    base: Content,
    fighter: Vec<u8>,
    cosmetics: Vec<u8>,
    slot: Option<u8>,
    lobby: Vec<Vec<u8>>,
    epoch: Option<u8>,
    session: Option<Session>,
    match_content: Option<Content>,
    started: Option<GroupSetup>,
    ticks: u32,
    rejected: Option<RejectReason>,
    left: bool,
}

impl<L: Link> GroupGuest<L> {
    pub fn new(link: L, base: &Content, fighter: Vec<u8>, cosmetics: Vec<u8>) -> GroupGuest<L> {
        GroupGuest {
            link,
            version_hash: (SIM_VERSION, base.hash()),
            base: base.clone(),
            fighter,
            cosmetics,
            slot: None,
            lobby: Vec::new(),
            epoch: None,
            session: None,
            match_content: None,
            started: None,
            ticks: 0,
            rejected: None,
            left: false,
        }
    }

    /// This player's slot (1 to 3) once the host has welcomed it.
    pub fn slot(&self) -> Option<u8> {
        self.slot
    }

    /// The name-and-look bytes of every slot as the host last told us.
    pub fn lobby(&self) -> &[Vec<u8>] {
        &self.lobby
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn state(&self) -> Option<&GameState> {
        self.session.as_ref().map(Session::state)
    }

    pub fn match_content(&self) -> Option<&Content> {
        self.match_content.as_ref()
    }

    pub fn match_setup(&self) -> Option<&GroupSetup> {
        self.started.as_ref()
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        self.session
            .as_mut()
            .map_or_else(Vec::new, Session::drain_events)
    }

    pub fn leave(&mut self) {
        if !self.left {
            self.left = true;
            self.link.send(&Msg::Leave.encode());
            if let Some(s) = &mut self.session {
                s.disconnect();
            }
        }
    }

    fn handle(&mut self, bytes: &[u8]) {
        let Some(msg) = Msg::decode(bytes) else {
            return;
        };
        match msg {
            Msg::Welcome { slot, cosmetics } => {
                self.slot = Some(slot);
                self.lobby = cosmetics;
            }
            Msg::Reject(code) => {
                self.rejected = RejectReason::from_u8(code);
            }
            Msg::Setup(setup) => self.on_setup(setup),
            Msg::Relay {
                epoch,
                dest,
                src,
                payload,
            } => {
                let mine = self.slot.is_some_and(|s| s == dest) || dest == BROADCAST;
                if Some(epoch) == self.epoch && mine {
                    if let Some(s) = &mut self.session {
                        s.handle_packet_from(src, &payload);
                    }
                }
            }
            Msg::Leave => {
                // The host is going: the session finds out through its own timeout, but a goodbye is quicker.
                if let Some(s) = &mut self.session {
                    s.handle_packet_from(0, &crate::packet::Packet::Disconnect.encode());
                }
            }
            _ => {}
        }
    }

    fn on_setup(&mut self, setup: GroupSetup) {
        let ready = Msg::Ready { epoch: setup.epoch }.encode();
        if self.epoch == Some(setup.epoch)
            && self.started.as_ref().is_some_and(|s| s.seed == setup.seed)
        {
            self.link.send(&ready); // a repeat: the host did not hear us
            return;
        }
        if self
            .epoch
            .is_some_and(|e| setup.epoch.wrapping_sub(e) > 128)
        {
            return; // an old setup
        }
        match setup_content(&self.base, &setup) {
            Ok((content, chars)) => {
                // The host works out the fighter numbers too; insist that it did so the same way.
                if chars != setup.chars || self.slot.is_some_and(|s| s != setup.slot) {
                    self.rejected = Some(RejectReason::BadFighter);
                    return;
                }
                self.slot = Some(setup.slot);
                self.session = Some(start_session(&content, &setup, chars, setup.slot));
                self.match_content = Some(content);
                self.epoch = Some(setup.epoch);
                self.lobby = setup.cosmetics.clone();
                self.started = Some(setup);
                self.link.send(&ready);
            }
            Err(reason) => self.rejected = Some(reason),
        }
    }

    pub fn update(&mut self, content: &Content, local: Input) -> GroupStatus {
        self.ticks += 1;
        while let Some(bytes) = self.link.recv() {
            self.handle(&bytes);
        }
        if let Some(reason) = self.rejected {
            return GroupStatus::Rejected(reason);
        }
        let Some(session) = self.session.as_mut() else {
            if self.slot.is_none() && self.ticks % RESEND_EVERY == 1 {
                let hello = Msg::Hello {
                    version: self.version_hash.0,
                    base_hash: self.version_hash.1,
                    fighter: self.fighter.clone(),
                    cosmetics: self.cosmetics.clone(),
                };
                self.link.send(&hello.encode());
            }
            return GroupStatus::Lobby;
        };
        let content = self.match_content.as_ref().unwrap_or(content);
        let advance = session.advance(content, local);
        let outgoing = session.drain_outgoing_to();
        let (epoch, src) = (self.epoch.unwrap_or(0), self.slot.unwrap_or(0));
        for (dest, payload) in outgoing {
            let relay = Msg::Relay {
                epoch,
                dest,
                src,
                payload,
            };
            self.link.send(&relay.encode());
        }
        GroupStatus::Running(advance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_round_trip_and_garbage_is_refused() {
        let setup = GroupSetup {
            epoch: 3,
            seed: 99,
            input_delay: 2,
            rules: MatchRules {
                stocks: 3,
                time_limit: 120,
            },
            stage: 1,
            ranked: true,
            active: 0b0111,
            slot: 2,
            chars: [0, 1, 2, 0],
            specs: vec![vec![0, 0], vec![0, 1], vec![1, 0, 5, 5, 5, 5]],
            cosmetics: vec![vec![1, 2], vec![], vec![3], vec![]],
        };
        for m in [
            Msg::Hello {
                version: 5,
                base_hash: 77,
                fighter: vec![0, 1],
                cosmetics: vec![9; 12],
            },
            Msg::Welcome {
                slot: 1,
                cosmetics: vec![vec![1], vec![2, 3]],
            },
            Msg::Setup(setup),
            Msg::Ready { epoch: 4 },
            Msg::Reject(4),
            Msg::Relay {
                epoch: 1,
                dest: 255,
                src: 2,
                payload: vec![1, 2, 3, 4],
            },
            Msg::Leave,
        ] {
            assert_eq!(Msg::decode(&m.encode()), Some(m.clone()), "{m:?}");
        }
        assert_eq!(Msg::decode(b"nonsense"), None);
        let good = Msg::Ready { epoch: 1 }.encode();
        for cut in 0..good.len() {
            let _ = Msg::decode(&good[..cut]);
        }
        let setup_bytes = Msg::Setup(GroupSetup {
            epoch: 0,
            seed: 1,
            input_delay: 1,
            rules: MatchRules::default(),
            stage: 0,
            ranked: false,
            active: 0b11,
            slot: 1,
            chars: [0; 4],
            specs: vec![vec![0, 0]],
            cosmetics: vec![],
        })
        .encode();
        for cut in 0..setup_bytes.len() {
            let _ = Msg::decode(&setup_bytes[..cut]);
        }
    }
}
