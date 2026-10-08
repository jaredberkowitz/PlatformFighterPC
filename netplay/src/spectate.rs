//! Spectating: watch a match as it is played. The host keeps a [`MatchRecord`] of the match (the seed, the setup, the fighters and every
//! confirmed frame's inputs, see [`crate::replay`]); a [`SpectatorServer`] streams that record to any number of spectators, and a
//! [`SpectatorClient`] rebuilds the match from it and plays it a little behind the live game. Because the simulation is deterministic,
//! a spectator needs nothing but the record: no rollback, no state transfer, and it can join late (it simply receives every frame from the
//! start and catches up).
//!
//! Wire format (little-endian, after the same two-byte magic as the game protocol): a kind byte, then
//! `Hello` (0x20): sim version u16, base roster hash u64 | `Setup` (0x21): a match record with no frames, encoded as in [`crate::replay`] |
//! `Frames` (0x22): seed u64, start frame u32, count u8, then count frames of 4 input bytes for each player in the match |
//! `Ack` (0x23): seed u64, frames held u32. Every message is resent until answered, so it works over a lossy link.
//!
//! Pure like the rest of this crate: the caller moves the datagrams and calls `tick` once per frame.

use crate::packet::MAGIC;
use crate::replay::MatchRecord;
use sim_core::{step, Content, GameState, Input, MAX_FIGHTERS, SIM_VERSION};

const T_HELLO: u8 = 0x20;
const T_SETUP: u8 = 0x21;
const T_FRAMES: u8 = 0x22;
const T_ACK: u8 = 0x23;

/// Most spectators one server serves.
pub const MAX_SPECTATORS: usize = 8;
/// Most frames in one datagram.
const CHUNK: usize = 30;
/// Frames the server may have in flight to one spectator beyond what it has acknowledged.
const WINDOW: u32 = 180;
/// Ticks between resends of an unanswered message.
const RESEND_EVERY: u32 = 8;

fn header(kind: u8) -> Vec<u8> {
    let mut v = MAGIC.to_le_bytes().to_vec();
    v.push(kind);
    v
}

fn body(bytes: &[u8]) -> Option<(u8, &[u8])> {
    if bytes.len() < 3 || bytes[..2] != MAGIC.to_le_bytes() {
        return None;
    }
    Some((bytes[2], &bytes[3..]))
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}
fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}
fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

struct Watcher {
    /// Whether it has said which match it holds (an ack for the current seed arrived).
    holds_setup: bool,
    /// Frames it has told us it holds contiguously.
    acked: u32,
    /// Next frame to send.
    sent: u32,
    idle: u32,
}

/// The host's side: serves the current match record to spectators identified by small numbers (the caller maps addresses to ids).
pub struct SpectatorServer {
    base_hash: u64,
    setup: Vec<u8>,
    seed: u64,
    active: u8,
    frames: Vec<[Input; MAX_FIGHTERS]>,
    watchers: Vec<Option<Watcher>>,
    ticks: u32,
}

impl SpectatorServer {
    pub fn new(base: &Content) -> SpectatorServer {
        SpectatorServer {
            base_hash: base.hash(),
            setup: Vec::new(),
            seed: 0,
            active: 0,
            frames: Vec::new(),
            watchers: (0..MAX_SPECTATORS).map(|_| None).collect(),
            ticks: 0,
        }
    }

    /// Follows the host's record of the match in progress: a new match (another seed) replaces the old, and new frames are picked up.
    pub fn sync(&mut self, record: &MatchRecord) {
        if self.setup.is_empty() || record.seed != self.seed {
            let mut head = record.clone();
            head.inputs.clear();
            let mut message = header(T_SETUP);
            message.extend_from_slice(&head.encode());
            self.setup = message;
            self.seed = record.seed;
            self.active = record.active;
            self.frames.clear();
            for w in self.watchers.iter_mut().flatten() {
                *w = Watcher {
                    holds_setup: false,
                    acked: 0,
                    sent: 0,
                    idle: 0,
                };
            }
        }
        self.frames = record.inputs.clone();
    }

    pub fn spectator_count(&self) -> usize {
        self.watchers.iter().flatten().count()
    }

    /// A datagram from spectator `id`. Returns the datagrams to send back to it.
    pub fn handle(&mut self, id: usize, bytes: &[u8]) -> Vec<Vec<u8>> {
        let Some((kind, rest)) = body(bytes) else {
            return Vec::new();
        };
        if id >= MAX_SPECTATORS {
            return Vec::new();
        }
        match kind {
            T_HELLO => {
                let (Some(version), Some(hash)) = (u16_at(rest, 0), u64_at(rest, 2)) else {
                    return Vec::new();
                };
                if version != SIM_VERSION || hash != self.base_hash {
                    return Vec::new();
                }
                if self.watchers[id].is_none() {
                    self.watchers[id] = Some(Watcher {
                        holds_setup: false,
                        acked: 0,
                        sent: 0,
                        idle: 0,
                    });
                }
                if self.setup.is_empty() {
                    Vec::new()
                } else {
                    vec![self.setup.clone()]
                }
            }
            T_ACK => {
                let (Some(seed), Some(held)) = (u64_at(rest, 0), u32_at(rest, 8)) else {
                    return Vec::new();
                };
                if let Some(w) = self.watchers[id].as_mut() {
                    if seed == self.seed {
                        w.holds_setup = true;
                        if held > w.acked {
                            w.acked = held.min(self.frames.len() as u32);
                            w.idle = 0;
                        }
                        w.sent = w.sent.max(w.acked);
                    }
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// One tick: the datagrams to send, as `(spectator id, bytes)`.
    pub fn tick(&mut self) -> Vec<(usize, Vec<u8>)> {
        self.ticks += 1;
        let mut out = Vec::new();
        let total = self.frames.len() as u32;
        for (id, slot) in self.watchers.iter_mut().enumerate() {
            let Some(w) = slot.as_mut() else {
                continue;
            };
            if !w.holds_setup {
                if !self.setup.is_empty() && self.ticks % RESEND_EVERY == 0 {
                    out.push((id, self.setup.clone()));
                }
                continue;
            }
            w.idle += 1;
            if w.idle > 2 * RESEND_EVERY && w.sent > w.acked {
                w.sent = w.acked; // nothing acknowledged for a while: send it again
                w.idle = 0;
            }
            while w.sent < total && w.sent < w.acked + WINDOW {
                let count = (total - w.sent).min(CHUNK as u32) as usize;
                let mut message = header(T_FRAMES);
                message.extend_from_slice(&self.seed.to_le_bytes());
                message.extend_from_slice(&w.sent.to_le_bytes());
                message.push(count as u8);
                for f in &self.frames[w.sent as usize..w.sent as usize + count] {
                    for (p, input) in f.iter().enumerate() {
                        if self.active >> p & 1 == 1 {
                            message.extend_from_slice(&input.to_bytes());
                        }
                    }
                }
                out.push((id, message));
                w.sent += count as u32;
            }
        }
        out
    }
}

/// The spectator's side: asks for the match, rebuilds it, and plays the frames as they arrive.
pub struct SpectatorClient {
    base: Content,
    ticks: u32,
    seed: Option<u64>,
    record: Option<MatchRecord>,
    content: Option<Content>,
    state: Option<GameState>,
    frames: Vec<[Input; MAX_FIGHTERS]>,
    refused: Option<String>,
}

impl SpectatorClient {
    pub fn new(base: &Content) -> SpectatorClient {
        SpectatorClient {
            base: base.clone(),
            ticks: 0,
            seed: None,
            record: None,
            content: None,
            state: None,
            frames: Vec::new(),
            refused: None,
        }
    }

    pub fn handle(&mut self, bytes: &[u8]) {
        let Some((kind, rest)) = body(bytes) else {
            return;
        };
        match kind {
            T_SETUP => {
                let Ok(record) = MatchRecord::decode(rest) else {
                    return;
                };
                if self.seed == Some(record.seed) {
                    return;
                }
                match record.rebuild(&self.base) {
                    Ok((content, state)) => {
                        self.seed = Some(record.seed);
                        self.content = Some(content);
                        self.state = Some(state);
                        self.frames.clear();
                        self.record = Some(record);
                        self.refused = None;
                    }
                    Err(e) => self.refused = Some(e),
                }
            }
            T_FRAMES => {
                let (Some(record), Some(seed)) = (self.record.as_ref(), self.seed) else {
                    return;
                };
                let (Some(their_seed), Some(start)) = (u64_at(rest, 0), u32_at(rest, 8)) else {
                    return;
                };
                if their_seed != seed || rest.len() < 13 {
                    return;
                }
                let count = usize::from(rest[12]);
                let per_frame = record.active.count_ones() as usize * Input::BYTES;
                let data = &rest[13..];
                if data.len() != count * per_frame || start as usize > self.frames.len() {
                    return;
                }
                for k in 0..count {
                    let index = start as usize + k;
                    if index < self.frames.len() {
                        continue; // a repeat
                    }
                    let mut frame = [Input::default(); MAX_FIGHTERS];
                    let mut at = k * per_frame;
                    for (p, slot) in frame.iter_mut().enumerate() {
                        if record.active >> p & 1 == 1 {
                            let c = &data[at..at + Input::BYTES];
                            *slot = Input::from_bytes([c[0], c[1], c[2], c[3]]);
                            at += Input::BYTES;
                        }
                    }
                    self.frames.push(frame);
                }
            }
            _ => {}
        }
    }

    /// One tick: the datagrams to send to the host.
    pub fn tick(&mut self) -> Vec<Vec<u8>> {
        self.ticks += 1;
        let mut out = Vec::new();
        if self.seed.is_none() {
            if self.ticks % RESEND_EVERY == 1 {
                let mut hello = header(T_HELLO);
                hello.extend_from_slice(&SIM_VERSION.to_le_bytes());
                hello.extend_from_slice(&self.base.hash().to_le_bytes());
                out.push(hello);
            }
        } else if self.ticks % 4 == 0 {
            let mut ack = header(T_ACK);
            ack.extend_from_slice(&self.seed.unwrap_or(0).to_le_bytes());
            ack.extend_from_slice(&(self.frames.len() as u32).to_le_bytes());
            out.push(ack);
        }
        out
    }

    /// Plays frames that have arrived, up to `budget` of them, and returns how many were played. A finished match stands still.
    pub fn advance(&mut self, budget: u32) -> u32 {
        let (Some(content), Some(state)) = (self.content.as_ref(), self.state.as_mut()) else {
            return 0;
        };
        let mut played = 0;
        while played < budget
            && (state.frame as usize) < self.frames.len()
            && state.winner == sim_core::state::PLAYING
        {
            step(state, content, &self.frames[state.frame as usize]);
            played += 1;
        }
        played
    }

    /// Frames received but not yet played.
    pub fn behind(&self) -> u32 {
        let played = self.state.as_ref().map_or(0, |s| s.frame as usize);
        self.frames.len().saturating_sub(played) as u32
    }

    pub fn frames_received(&self) -> u32 {
        self.frames.len() as u32
    }

    /// Whether the match has been set up (the first state exists).
    pub fn ready(&self) -> bool {
        self.state.is_some()
    }

    pub fn state(&self) -> Option<&GameState> {
        self.state.as_ref()
    }

    pub fn content(&self) -> Option<&Content> {
        self.content.as_ref()
    }

    pub fn record(&self) -> Option<&MatchRecord> {
        self.record.as_ref()
    }

    /// Why the host's match cannot be watched (another sim version or roster), if that is what happened.
    pub fn refused(&self) -> Option<&str> {
        self.refused.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::fuzz::random_inputs;
    use sim_core::{MatchRules, Rng};

    /// Runs a match with random inputs, giving the host's record after `frames` frames.
    fn record_of(base: &Content, seed: u64, frames: usize, rules: MatchRules) -> MatchRecord {
        let mut r = MatchRecord::begin(base, base, seed, [0, 1, 0, 1], 0b0011, rules, 0, vec![]);
        r.inputs = random_inputs(&mut Rng::new(seed), frames);
        r
    }

    /// Moves datagrams between a server and a client with loss, for `ticks` ticks.
    #[allow(clippy::too_many_arguments)]
    fn pump(
        server: &mut SpectatorServer,
        client: &mut SpectatorClient,
        record: &mut MatchRecord,
        full: &MatchRecord,
        ticks: u32,
        rng: &mut Rng,
        loss_percent: u32,
        grow_per_tick: usize,
    ) {
        for _ in 0..ticks {
            // The live match grows.
            let have = record.inputs.len();
            let to = (have + grow_per_tick).min(full.inputs.len());
            record.inputs = full.inputs[..to].to_vec();
            server.sync(record);
            for m in client.tick() {
                if rng.range(100) >= loss_percent {
                    for reply in server.handle(0, &m) {
                        if rng.range(100) >= loss_percent {
                            client.handle(&reply);
                        }
                    }
                }
            }
            for (id, m) in server.tick() {
                assert_eq!(id, 0);
                if rng.range(100) >= loss_percent {
                    client.handle(&m);
                }
            }
            client.advance(2);
        }
    }

    #[test]
    fn a_spectator_follows_the_match_to_the_same_ending() {
        let base = Content::placeholder();
        let rules = MatchRules {
            stocks: 2,
            time_limit: 20,
        };
        let full = record_of(&base, 5, 1500, rules);
        let mut live = full.clone();
        live.inputs.clear();
        let mut server = SpectatorServer::new(&base);
        let mut client = SpectatorClient::new(&base);
        let mut rng = Rng::new(1);
        pump(
            &mut server,
            &mut client,
            &mut live,
            &full,
            2500,
            &mut rng,
            0,
            1,
        );
        let end = client.state().expect("the match is on");
        // The host's own end, played straight.
        let mut sealed = full.clone();
        sealed.seal(&base).unwrap();
        assert_eq!(end.winner, sealed.winner);
        assert_eq!(end.checksum(), sealed.final_checksum);
    }

    #[test]
    fn a_lossy_link_and_a_late_joiner_still_see_every_frame() {
        let base = Content::placeholder();
        let full = record_of(
            &base,
            9,
            900,
            MatchRules {
                stocks: 3,
                time_limit: 0,
            },
        );
        // The match is already 600 frames old when the spectator arrives.
        let mut live = full.clone();
        live.inputs.truncate(600);
        let mut server = SpectatorServer::new(&base);
        let mut client = SpectatorClient::new(&base);
        let mut rng = Rng::new(2);
        pump(
            &mut server,
            &mut client,
            &mut live,
            &full,
            4000,
            &mut rng,
            25,
            1,
        );
        assert_eq!(client.frames_received(), 900, "all of it arrived");
        let straight = full.state_after();
        assert_eq!(client.state().unwrap().checksum(), straight.checksum());
    }

    #[test]
    fn a_new_match_replaces_the_old_one() {
        let base = Content::placeholder();
        let first = record_of(&base, 1, 300, MatchRules::default());
        let second = record_of(&base, 2, 300, MatchRules::default());
        let mut server = SpectatorServer::new(&base);
        let mut client = SpectatorClient::new(&base);
        let mut rng = Rng::new(3);
        let mut live = first.clone();
        live.inputs.clear();
        pump(
            &mut server,
            &mut client,
            &mut live,
            &first,
            800,
            &mut rng,
            10,
            1,
        );
        assert_eq!(client.record().unwrap().seed, 1);
        let mut live2 = second.clone();
        live2.inputs.clear();
        pump(
            &mut server,
            &mut client,
            &mut live2,
            &second,
            1200,
            &mut rng,
            10,
            1,
        );
        assert_eq!(client.record().unwrap().seed, 2, "it moved to the rematch");
        assert_eq!(client.frames_received(), 300);
    }

    #[test]
    fn made_fighters_and_other_stages_can_be_watched() {
        use sim_content::recipe::{match_content_on, FighterSpec, Recipe};
        let base = Content::placeholder();
        let made = FighterSpec::Made(Recipe {
            class: 1,
            size: 8,
            speed: 3,
            jump: 4,
            weight: 5,
        });
        let specs = vec![made.encode(), FighterSpec::Builtin(0).encode()];
        let parsed: Vec<FighterSpec> = specs
            .iter()
            .map(|b| FighterSpec::decode(b).unwrap())
            .collect();
        let (content, chars) = match_content_on(&base, &parsed, false, 2).unwrap();
        let mut full = MatchRecord::begin(
            &base,
            &content,
            4,
            [chars[0], chars[1], 0, 1],
            0b0011,
            MatchRules::default(),
            2,
            specs,
        );
        full.inputs = random_inputs(&mut Rng::new(4), 700);
        let mut live = full.clone();
        live.inputs.clear();
        let mut server = SpectatorServer::new(&base);
        let mut client = SpectatorClient::new(&base);
        pump(
            &mut server,
            &mut client,
            &mut live,
            &full,
            1500,
            &mut Rng::new(5),
            5,
            1,
        );
        assert_eq!(
            client.content().unwrap().stage.platforms.len(),
            1,
            "the stage came across"
        );
        assert_eq!(client.frames_received(), 700);
    }

    #[test]
    fn strangers_and_garbage_are_ignored() {
        let base = Content::placeholder();
        let mut server = SpectatorServer::new(&base);
        server.sync(&record_of(&base, 1, 10, MatchRules::default()));
        assert!(server.handle(0, b"nonsense").is_empty());
        assert!(server.handle(99, &[0x46, 0x50, T_HELLO]).is_empty());
        // A client on another sim version or roster is not served.
        let mut hello = header(T_HELLO);
        hello.extend_from_slice(&(SIM_VERSION + 1).to_le_bytes());
        hello.extend_from_slice(&base.hash().to_le_bytes());
        assert!(server.handle(0, &hello).is_empty());
        assert_eq!(server.spectator_count(), 0);
        let mut ok = header(T_HELLO);
        ok.extend_from_slice(&SIM_VERSION.to_le_bytes());
        ok.extend_from_slice(&base.hash().to_le_bytes());
        assert_eq!(
            server.handle(0, &ok).len(),
            1,
            "a good hello gets the setup"
        );
        assert_eq!(server.spectator_count(), 1);
        let mut client = SpectatorClient::new(&base);
        for len in 0..40 {
            client.handle(&vec![0x46; len]);
            client.handle(&[0x46, 0x50, T_FRAMES, 1, 2, 3]);
            client.handle(&[0x46, 0x50, T_SETUP, 9, 9]);
        }
        assert!(!client.ready());
    }

    trait StateAfter {
        fn state_after(&self) -> GameState;
    }

    impl StateAfter for MatchRecord {
        /// The state after playing every frame of the record (a match that ends stops at its end).
        fn state_after(&self) -> GameState {
            let (content, mut state) = self.rebuild(&Content::placeholder()).unwrap();
            for f in &self.inputs {
                step(&mut state, &content, f);
            }
            state
        }
    }
}
