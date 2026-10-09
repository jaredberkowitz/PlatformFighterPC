//! Thin Godot wrapper around `sim-core`.
//!
//! GDScript gathers input, calls `tick()` once per 60 Hz physics frame, then reads state back to
//! draw it. Nothing here decides gameplay, and animation never drives the sim. Floats appear only
//! in the read-back getters, for display.

mod editor;

use godot::classes::{INode, Node};
use godot::prelude::*;
use netplay::group::{GroupGuest, GroupHost, GroupParams, GroupSetup, GroupStatus};
use netplay::packet::Setup;
use netplay::peer::{Link, Peer, Status};
use netplay::replay::{MatchRecord, Recorder};
use netplay::session::{Advance, Event};
use netplay::spectate::{SpectatorClient, SpectatorServer};
use sim_content::recipe::{match_content_on, FighterSpec};
use sim_content::stages;
use sim_core::input::buttons;
use sim_core::state::PLAYING;
use sim_core::{step, Content, Fx, GameState, Input, MatchRules, MAX_FIGHTERS, SIM_VERSION};
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use transport::{QuickMatch, RelayLink, SpectatorSocket, UdpLink};

/// The two ways a networked match can reach the other player.
enum NetLink {
    Udp(UdpLink),
    Relay(RelayLink),
}

impl Link for NetLink {
    fn send(&mut self, bytes: &[u8]) {
        match self {
            NetLink::Udp(l) => l.send(bytes),
            NetLink::Relay(l) => l.send(bytes),
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        match self {
            NetLink::Udp(l) => l.recv(),
            NetLink::Relay(l) => l.recv(),
        }
    }
}

fn any_local() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
}

fn resolve(text: &str) -> Result<SocketAddr, String> {
    text.to_socket_addrs()
        .map_err(|e| format!("cannot resolve {text}: {e}"))?
        .next()
        .ok_or_else(|| format!("no address for {text}"))
}

/// Frames of history kept for stepping backwards in training mode (20 seconds).
const MAX_HISTORY: usize = 1200;

struct PfExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PfExtension {}

fn f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}

/// The confirmed frames of a session from `from` on (frames every player's input is known for).
fn confirmed_since(session: &netplay::session::Session, from: u32) -> Vec<[Input; MAX_FIGHTERS]> {
    let confirmed = session.confirmed_frame().min(session.frame());
    (from..confirmed)
        .map(|f| {
            let known = session.known_inputs(f);
            let mut inputs = [Input::default(); MAX_FIGHTERS];
            for (slot, k) in inputs.iter_mut().zip(known.iter()) {
                if let Some(i) = k {
                    *slot = *i;
                }
            }
            inputs
        })
        .collect()
}

fn clamp_i8(v: i32) -> i8 {
    v.clamp(-127, 127) as i8
}

/// A replay being watched: the record and the state it starts from.
struct Playback {
    record: MatchRecord,
    initial: GameState,
}

/// Free play: no stocks to lose, no clock.
const UNLIMITED: MatchRules = MatchRules {
    stocks: 0,
    time_limit: 0,
};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct SimRunner {
    base: Base<Node>,
    content: Content,
    content_name: String,
    /// Which fighter slots are in the match (bit `n` is fighter `n`). The game has two players.
    players: u8,
    /// This player's cosmetic loadout bytes, sent to the other player in the handshake (opaque; never used by the sim).
    my_cosmetics: Vec<u8>,
    /// The fighter this player brings to an online match (`FighterSpec` bytes; empty uses the characters as given).
    my_fighter: Vec<u8>,
    /// Ranked rules for the match this player hosts: fighters over the point budget are refused.
    ranked: bool,
    /// Stocks and time limit of the next match (also sent to the other side when this player hosts). Unlimited by default,
    /// which is free play: nobody is eliminated.
    match_rules: MatchRules,
    /// Stage (index into `sim_content::stages`) for the next match; the host's choice goes to the joiner.
    match_stage: u8,
    /// The fighters of a local match (specs of players 0 and 1), kept so the match can be recorded.
    match_specs: Vec<Vec<u8>>,
    /// Records the match in progress for a replay; `None` when it cannot be reproduced (custom content, training edits).
    recorder: Option<Recorder>,
    /// Set while a replay is being watched.
    playback: Option<Playback>,
    /// The base roster, kept while an online match runs on the base roster plus made fighters.
    base_content: Option<Content>,
    state: GameState,
    inputs: [Input; MAX_FIGHTERS],
    history: VecDeque<GameState>,
    /// Set while playing over the network; then `state` is a copy of the session's state.
    net: Option<Peer<NetLink>>,
    net_log: Vec<String>,
    /// Hosting over UDP: the server that streams the match to spectators on the next port.
    spectators: Option<(SpectatorServer, SpectatorSocket)>,
    /// Watching someone else's match.
    watching: Option<(SpectatorClient, UdpLink)>,
    /// The seed of the match being watched, to notice a rematch.
    watching_seed: u64,
    /// A group match (three or four players): hosting it, or joined to one.
    group_host: Option<GroupHost<SpectatorSocket>>,
    group_guest: Option<GroupGuest<UdpLink>>,
    group_log: Vec<String>,
    /// Looking for an opponent through a relay's queue.
    quick: Option<QuickMatch>,
}

impl SimRunner {
    fn replay_bytes_with(&self, cosmetics: Vec<Vec<u8>>) -> PackedByteArray {
        let Some(rec) = self.recorder.as_ref() else {
            return PackedByteArray::new();
        };
        let mut record = rec.record.clone();
        if record.cosmetics.is_empty() {
            record.cosmetics = cosmetics;
        }
        match record.seal(&Content::placeholder()) {
            Ok(()) => PackedByteArray::from(record.encode().as_slice()),
            Err(e) => {
                godot_warn!("replay not saved: {e}");
                PackedByteArray::new()
            }
        }
    }

    fn load_specs(&mut self, raw: Vec<Vec<u8>>) -> VarDictionary {
        let mut out = VarDictionary::new();
        out.set("error", "");
        out.set("chars", &PackedInt32Array::new());
        if !(2..=MAX_FIGHTERS).contains(&raw.len()) {
            out.set("error", "a match needs two to four fighters");
            return out;
        }
        let base = Content::placeholder();
        let parsed: Option<Vec<FighterSpec>> = raw.iter().map(|b| FighterSpec::decode(b)).collect();
        let Some(parsed) = parsed else {
            out.set("error", "a fighter could not be read");
            return out;
        };
        match match_content_on(&base, &parsed, false, self.match_stage) {
            Ok((content, chars)) => {
                self.content = content;
                self.content_name = String::new();
                self.net = None;
                self.base_content = None;
                self.players = ((1u16 << raw.len()) - 1) as u8;
                self.match_specs = raw;
                self.recorder = None;
                self.playback = None;
                let mut ids = [0u8, 1, 0, 1];
                for (slot, c) in ids.iter_mut().zip(chars.iter()) {
                    *slot = *c;
                }
                self.state = GameState::new_with_rules(
                    &self.content,
                    1,
                    ids,
                    self.players,
                    self.match_rules,
                );
                self.inputs = [Input::default(); MAX_FIGHTERS];
                self.history.clear();
                let list: Vec<i32> = chars.iter().map(|c| i32::from(*c)).collect();
                out.set("chars", &PackedInt32Array::from(list.as_slice()));
            }
            Err(e) => out.set("error", format!("a fighter is not valid: {e:?}")),
        }
        out
    }

    /// A recorder for the local match just started, if it can be reproduced from the base roster and the fighters' specs.
    fn new_local_recorder(&self, seed: u64, ids: [u8; MAX_FIGHTERS]) -> Option<Recorder> {
        let base = Content::placeholder();
        if self.match_specs.is_empty() {
            if self.content.hash() != stages::with_stage(&base, self.match_stage)?.hash() {
                return None;
            }
        } else {
            let parsed: Option<Vec<FighterSpec>> = self
                .match_specs
                .iter()
                .map(|b| FighterSpec::decode(b))
                .collect();
            let (built, chars) = match_content_on(&base, &parsed?, false, self.match_stage).ok()?;
            if built.hash() != self.content.hash()
                || chars.iter().zip(ids.iter()).any(|(c, i)| c != i)
            {
                return None;
            }
        }
        Some(Recorder::new(MatchRecord::begin(
            &base,
            &self.content,
            seed,
            ids,
            self.players,
            self.state.rules,
            self.match_stage,
            self.match_specs.clone(),
        )))
    }

    /// A group match: starts a record when a match starts (and for a rematch), then adds every confirmed frame.
    fn record_group(&mut self, setup: &GroupSetup, from: u32, frames: Vec<[Input; MAX_FIGHTERS]>) {
        let fresh = self
            .recorder
            .as_ref()
            .is_none_or(|r| r.record.seed != setup.seed);
        if fresh {
            let base = self.base_content.as_ref().unwrap_or(&self.content);
            let mut record = MatchRecord::begin(
                base,
                &self.content,
                setup.seed,
                setup.chars,
                setup.active,
                setup.rules,
                setup.stage,
                setup.specs.clone(),
            );
            record.cosmetics = setup.cosmetics.clone();
            self.recorder = Some(Recorder::new(record));
        }
        let Some(rec) = self.recorder.as_mut() else {
            return;
        };
        for (k, inputs) in frames.into_iter().enumerate() {
            if !rec.push(from + k as u32, inputs) {
                break;
            }
        }
    }

    /// Hosting: streams the confirmed frames of the match to whoever is watching.
    fn serve_spectators(&mut self) {
        let (Some((server, socket)), Some(rec)) =
            (self.spectators.as_mut(), self.recorder.as_ref())
        else {
            return;
        };
        server.sync(&rec.record);
        while let Some((id, bytes)) = socket.recv() {
            for reply in server.handle(id, &bytes) {
                socket.send(id, &reply);
            }
        }
        for (id, bytes) in server.tick() {
            socket.send(id, &bytes);
        }
    }

    /// Online: starts a record when a match starts (and again for a rematch), then adds every confirmed frame.
    fn record_online(&mut self) {
        let Some(peer) = self.net.as_ref() else {
            return;
        };
        let (Some(setup), Some(session)) = (peer.match_setup(), peer.session()) else {
            return;
        };
        let fresh = self
            .recorder
            .as_ref()
            .is_none_or(|r| r.record.seed != setup.seed);
        if fresh {
            let base = self.base_content.as_ref().unwrap_or(&self.content);
            let specs = peer
                .fighter_specs()
                .map(|(h, j)| vec![h, j])
                .unwrap_or_default();
            let mut record = MatchRecord::begin(
                base,
                &self.content,
                setup.seed,
                setup.chars,
                setup.active,
                setup.rules,
                setup.stage,
                specs,
            );
            record.cosmetics = peer.player_cosmetics().to_vec();
            self.recorder = Some(Recorder::new(record));
        }
        let Some(rec) = self.recorder.as_mut() else {
            return;
        };
        let confirmed = session.confirmed_frame().min(session.frame());
        while rec.len() < confirmed {
            let f = rec.len();
            let known = session.known_inputs(f);
            let mut inputs = [Input::default(); MAX_FIGHTERS];
            for (slot, k) in inputs.iter_mut().zip(known.iter()) {
                if let Some(i) = k {
                    *slot = *i;
                }
            }
            if !rec.push(f, inputs) {
                break;
            }
        }
    }
}

#[godot_api]
impl INode for SimRunner {
    fn init(base: Base<Node>) -> Self {
        let content = Content::placeholder();
        let state = GameState::new_with_rules(&content, 1, [0, 1, 0, 1], 0b0011, UNLIMITED);
        SimRunner {
            base,
            content,
            content_name: String::new(),
            players: 0b0011,
            my_cosmetics: Vec::new(),
            my_fighter: Vec::new(),
            ranked: false,
            match_rules: UNLIMITED,
            match_stage: 0,
            match_specs: Vec::new(),
            recorder: None,
            playback: None,
            base_content: None,
            state,
            inputs: [Input::default(); MAX_FIGHTERS],
            history: VecDeque::new(),
            net: None,
            net_log: Vec::new(),
            spectators: None,
            watching: None,
            watching_seed: 0,
            group_host: None,
            group_guest: None,
            group_log: Vec::new(),
            quick: None,
        }
    }
}

#[godot_api]
impl SimRunner {
    // ---- Control ----

    /// Replaces the built-in roster with the content bundle in the file at `path`, and starts a fresh match with it.
    /// Returns an empty string on success, otherwise every problem found (nothing is changed then).
    #[func]
    fn load_content(&mut self, path: GString) -> GString {
        let path = path.to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => return GString::from(format!("cannot read {path}: {e}").as_str()),
        };
        let bundle = match sim_content::load(&text) {
            Ok(b) => b,
            Err(errors) => return GString::from(format!("{path}: {}", errors.join("; ")).as_str()),
        };
        if let Err(errors) = sim_content::validate(&bundle.content) {
            return GString::from(
                format!("{path} is not valid content: {}", errors.join("; ")).as_str(),
            );
        }
        self.content = bundle.content;
        self.content_name = bundle.manifest.name;
        self.net = None;
        self.match_specs.clear();
        self.recorder = None;
        self.playback = None;
        self.state = GameState::new_with_rules(
            &self.content,
            1,
            [0, 1, 0, 1],
            self.players,
            self.match_rules,
        );
        self.inputs = [Input::default(); MAX_FIGHTERS];
        self.history.clear();
        GString::new()
    }

    /// How many fighters take part (2 by default). The others are inert: they cannot be hit or targeted.
    /// Takes effect at the next `start`.
    #[func]
    fn set_players(&mut self, count: i32) {
        self.players = ((1u16 << count.clamp(1, MAX_FIGHTERS as i32)) - 1) as u8;
    }

    /// Sets this player's cosmetic loadout (a few opaque bytes). Call it before `net_host` / `net_join`; it travels
    /// in the handshake and never touches the simulation or its checksum. Longer than 64 bytes is cut.
    #[func]
    fn set_cosmetics(&mut self, bytes: PackedByteArray) {
        self.my_cosmetics = bytes.as_slice().iter().take(64).copied().collect();
    }

    /// The fighter this player brings to an online match (`FighterSpec` bytes: `[0, n]` for built-in fighter n,
    /// `[1, class, size, speed, jump, weight]` for a made one). Call before `net_host` / `net_join`. Empty means "use the
    /// characters given to `net_host`".
    #[func]
    fn set_fighter(&mut self, bytes: PackedByteArray) {
        self.my_fighter = bytes.as_slice().iter().take(16).copied().collect();
    }

    /// Builds the content for a local match from two fighters' specs (`FighterSpec` bytes), the same way an online match does,
    /// and loads it. Returns `{error, chars}`: the fighter numbers to pass to `start`, or an error text.
    #[func]
    fn load_match_fighters(
        &mut self,
        spec0: PackedByteArray,
        spec1: PackedByteArray,
    ) -> VarDictionary {
        self.load_specs(vec![spec0.to_vec(), spec1.to_vec()])
    }

    /// Like `load_match_fighters` for two to four fighters: `specs` is an array of `FighterSpec` byte arrays, one per player.
    /// Also sets how many players take part.
    #[func]
    fn load_match_roster(&mut self, specs: Array<PackedByteArray>) -> VarDictionary {
        let raw: Vec<Vec<u8>> = specs.iter_shared().map(|s| s.to_vec()).collect();
        self.load_specs(raw)
    }

    /// The finished (or abandoned) match as replay-file bytes, or empty if it cannot be replayed (training edits, custom
    /// content, nothing recorded). `cosmetics0/1` are the players' profile bytes for local matches; online records already
    /// have both.
    #[func]
    fn replay_bytes(
        &self,
        cosmetics0: PackedByteArray,
        cosmetics1: PackedByteArray,
    ) -> PackedByteArray {
        self.replay_bytes_with(vec![cosmetics0.to_vec(), cosmetics1.to_vec()])
    }

    /// `replay_bytes` for matches of up to four players: `cosmetics` is an array of profile byte arrays, one per player.
    #[func]
    fn replay_bytes_roster(&self, cosmetics: Array<PackedByteArray>) -> PackedByteArray {
        self.replay_bytes_with(cosmetics.iter_shared().map(|c| c.to_vec()).collect())
    }

    /// What a replay file says about itself, without loading it: `{ok, error, frames, seed, winner, stocks, time_limit,
    /// cosmetics0, cosmetics1, sim_version, playable}`.
    #[func]
    fn replay_peek(&self, bytes: PackedByteArray) -> VarDictionary {
        let mut out = VarDictionary::new();
        match MatchRecord::decode(bytes.as_slice()) {
            Ok(r) => {
                out.set("ok", true);
                out.set("error", "");
                out.set("frames", r.inputs.len() as i32);
                out.set("seed", r.seed as i64);
                out.set("winner", i32::from(r.winner));
                out.set("stocks", i32::from(r.rules.stocks));
                out.set("time_limit", i32::from(r.rules.time_limit));
                out.set("active", i32::from(r.active));
                let cos = |i: usize| {
                    r.cosmetics.get(i).map_or(PackedByteArray::new(), |c| {
                        PackedByteArray::from(c.as_slice())
                    })
                };
                out.set("cosmetics0", &cos(0));
                out.set("cosmetics1", &cos(1));
                out.set("cosmetics2", &cos(2));
                out.set("cosmetics3", &cos(3));
                out.set("players", r.active.count_ones() as i32);
                out.set("stage", i32::from(r.stage));
                out.set("sim_version", i32::from(r.sim_version));
                out.set("playable", r.sim_version == SIM_VERSION);
            }
            Err(e) => {
                out.set("ok", false);
                out.set("error", e);
            }
        }
        out
    }

    /// Loads a replay to watch: the match content and first state are rebuilt, and `replay_tick` plays it. Returns an error
    /// text, or an empty string.
    #[func]
    fn replay_load(&mut self, bytes: PackedByteArray) -> GString {
        let record = match MatchRecord::decode(bytes.as_slice()) {
            Ok(r) => r,
            Err(e) => return GString::from(e.as_str()),
        };
        let (content, initial) = match record.rebuild(&Content::placeholder()) {
            Ok(x) => x,
            Err(e) => return GString::from(e.as_str()),
        };
        self.content = content;
        self.content_name = String::new();
        self.net = None;
        self.base_content = None;
        self.match_specs.clear();
        self.recorder = None;
        self.players = record.active;
        self.match_rules = record.rules;
        self.state = initial;
        self.inputs = [Input::default(); MAX_FIGHTERS];
        self.history.clear();
        self.playback = Some(Playback { record, initial });
        GString::new()
    }

    /// Frames in the loaded replay (0 if none).
    #[func]
    fn replay_length(&self) -> i32 {
        self.playback
            .as_ref()
            .map_or(0, |p| p.record.inputs.len() as i32)
    }

    /// Plays the replay's next frame. Returns false once it has ended (or none is loaded).
    #[func]
    fn replay_tick(&mut self) -> bool {
        let Some(pb) = self.playback.as_ref() else {
            return false;
        };
        let Some(frame) = pb.record.inputs.get(self.state.frame as usize) else {
            return false;
        };
        self.inputs = *frame;
        self.tick();
        true
    }

    /// Jumps the replay to `frame` (clamped), by playing it again from the start.
    #[func]
    fn replay_seek(&mut self, frame: i32) {
        let Some(pb) = self.playback.as_ref() else {
            return;
        };
        let target = usize::try_from(frame)
            .unwrap_or(0)
            .min(pb.record.inputs.len());
        let mut state = pb.initial;
        for inputs in &pb.record.inputs[..target] {
            step(&mut state, &self.content, inputs);
        }
        self.state = state;
        self.history.clear();
    }

    /// Whether the replay's own end state is what it was recorded as (plays the whole thing; for tests and tools).
    #[func]
    fn replay_verify(&self) -> bool {
        self.playback
            .as_ref()
            .is_some_and(|p| p.record.verify(&Content::placeholder()).is_ok())
    }

    /// How the next match is won: `stocks` each (0 = unlimited, nobody is eliminated) and a time limit in seconds (0 =
    /// none). Takes effect at the next `start` or `net_host`; the joiner uses the host's.
    #[func]
    fn set_match_rules(&mut self, stocks: i32, time_limit: i32) {
        self.match_rules = MatchRules {
            stocks: stocks.clamp(0, i32::from(MatchRules::MAX_STOCKS)) as u8,
            time_limit: time_limit.clamp(0, 3600) as u16,
        }
        .clamped();
    }

    /// The rules of the running match as `[stocks, time_limit_seconds]`.
    #[func]
    fn match_rules(&self) -> PackedInt32Array {
        PackedInt32Array::from(
            [
                i32::from(self.state.rules.stocks),
                i32::from(self.state.rules.time_limit),
            ]
            .as_slice(),
        )
    }

    /// -1 while the match is on, a fighter's index when that fighter has won, -2 for a draw.
    #[func]
    fn winner(&self) -> i32 {
        i32::from(self.state.winner)
    }

    /// Whether fighter `i` is still in the match (not eliminated, and not an unused slot).
    #[func]
    fn fighter_active(&self, i: i32) -> bool {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.state.fighters.get(i))
            .is_some_and(|f| f.active)
    }

    /// Whether fighter `i` started this match (it may have been eliminated since).
    #[func]
    fn fighter_in_roster(&self, i: i32) -> bool {
        (0..MAX_FIGHTERS as i32).contains(&i) && self.state.roster >> i & 1 == 1
    }

    /// Frames played so far in this match.
    #[func]
    fn match_frame(&self) -> i32 {
        self.state.frame as i32
    }

    /// Ranked rules for a match this player hosts: fighters over the point budget are refused.
    #[func]
    fn set_ranked(&mut self, ranked: bool) {
        self.ranked = ranked;
    }

    /// Both players' fighter specs as `[host's, joiner's]` once the handshake has them (empty array until then).
    #[func]
    fn net_fighter_specs(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some((h, j)) = self.net.as_ref().and_then(|p| p.fighter_specs()) {
            out.push(&PackedByteArray::from(h.as_slice()).to_variant());
            out.push(&PackedByteArray::from(j.as_slice()).to_variant());
        }
        out
    }

    /// The other player's cosmetic bytes once the handshake has delivered them (empty until then).
    #[func]
    fn net_their_cosmetics(&self) -> PackedByteArray {
        self.net.as_ref().map_or_else(PackedByteArray::new, |p| {
            PackedByteArray::from(p.their_cosmetics())
        })
    }

    /// Like `load_content` but from bundle text (what the editors produce). Returns every problem, or an empty string.
    #[func]
    fn load_content_text(&mut self, text: GString) -> GString {
        let bundle = match sim_content::load(&text.to_string()) {
            Ok(b) => b,
            Err(errors) => return GString::from(errors.join("; ").as_str()),
        };
        if let Err(errors) = sim_content::validate(&bundle.content) {
            return GString::from(format!("not valid content: {}", errors.join("; ")).as_str());
        }
        self.content = bundle.content;
        self.content_name = bundle.manifest.name;
        self.net = None;
        self.match_specs.clear();
        self.recorder = None;
        self.playback = None;
        self.state = GameState::new_with_rules(
            &self.content,
            1,
            [0, 1, 0, 1],
            self.players,
            self.match_rules,
        );
        self.inputs = [Input::default(); MAX_FIGHTERS];
        self.history.clear();
        GString::new()
    }

    /// The loaded bundle's name, or an empty string for the built-in roster.
    #[func]
    fn content_name(&self) -> GString {
        GString::from(self.content_name.as_str())
    }

    /// Names of the fighters in the loaded roster, in index order.
    #[func]
    fn fighter_names(&self) -> PackedStringArray {
        let names: Vec<GString> = self
            .content
            .names
            .fighters
            .iter()
            .map(|n| GString::from(n.as_str()))
            .collect();
        PackedStringArray::from(names.as_slice())
    }

    /// Starts a fresh match. `chars` picks each player's physics profile (0 balanced, 1 floaty).
    #[func]
    fn start(&mut self, seed: i64, chars: PackedInt32Array) {
        let mut ids = [0u8; MAX_FIGHTERS];
        for (slot, c) in ids.iter_mut().zip(chars.as_slice()) {
            *slot = (*c).clamp(0, self.content.fighters.len() as i32 - 1) as u8;
        }
        self.state = GameState::new_with_rules(
            &self.content,
            seed as u64,
            ids,
            self.players,
            self.match_rules,
        );
        self.inputs = [Input::default(); MAX_FIGHTERS];
        self.history.clear();
        self.playback = None;
        self.recorder = self.new_local_recorder(seed as u64, ids);
    }

    #[func]
    fn set_input(&mut self, player: i32, stick_x: i32, stick_y: i32, button_mask: i32) {
        if let Some(slot) = usize::try_from(player)
            .ok()
            .and_then(|p| self.inputs.get_mut(p))
        {
            *slot = Input {
                stick_x: clamp_i8(stick_x),
                stick_y: clamp_i8(stick_y),
                buttons: button_mask as u16,
            };
        }
    }

    /// Advances the simulation exactly one frame with the inputs last set.
    #[func]
    fn tick(&mut self) {
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(self.state);
        if self.state.winner == PLAYING {
            if let Some(rec) = self.recorder.as_mut() {
                rec.push(self.state.frame, self.inputs);
            }
        }
        step(&mut self.state, &self.content, &self.inputs);
    }

    /// Steps one frame backwards (training mode). Returns false if there is no history left.
    #[func]
    fn step_back(&mut self) -> bool {
        self.recorder = None;
        match self.history.pop_back() {
            Some(s) => {
                self.state = s;
                true
            }
            None => false,
        }
    }

    /// Debug: teleport a fighter into the air at (x, y) with zero velocity.
    #[func]
    fn debug_place_airborne(&mut self, player: i32, x: f32, y: f32) {
        self.recorder = None;
        if let Some(fighter) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fighter.pos.x = Fx::from_raw((x * 65536.0) as i32);
            fighter.pos.y = Fx::from_raw((y * 65536.0) as i32);
            fighter.vel = sim_core::Vec2::ZERO;
            fighter.platform = sim_core::state::NONE;
            fighter.ledge = sim_core::state::NONE;
            fighter.state = sim_core::state::FighterState::Airborne;
            fighter.state_frame = 0;
        }
    }

    /// Debug: put a fighter into the special fall.
    #[func]
    fn debug_helpless(&mut self, player: i32) {
        self.recorder = None;
        if let Some(fighter) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            sim_core::fighter::enter_helpless(fighter);
        }
    }

    // ---- Match info ----

    #[func]
    fn frame(&self) -> i64 {
        i64::from(self.state.frame)
    }

    #[func]
    fn checksum(&self) -> GString {
        GString::from(format!("{:016x}", self.state.checksum()).as_str())
    }

    #[func]
    fn content_hash(&self) -> GString {
        GString::from(format!("{:016x}", self.content.hash()).as_str())
    }

    #[func]
    fn sim_version(&self) -> i32 {
        i32::from(SIM_VERSION)
    }

    #[func]
    fn history_len(&self) -> i32 {
        self.history.len() as i32
    }

    #[func]
    fn fighter_count(&self) -> i32 {
        MAX_FIGHTERS as i32
    }

    #[func]
    fn button_mask(&self, name: GString) -> i32 {
        let mask = match name.to_string().as_str() {
            "jump" => buttons::JUMP,
            "attack" => buttons::ATTACK,
            "special" => buttons::SPECIAL,
            "shield" => buttons::SHIELD,
            "grab" => buttons::GRAB,
            "strong" => buttons::STRONG,
            _ => 0,
        };
        i32::from(mask)
    }

    // ---- Fighters (read-only, for drawing) ----

    #[func]
    fn fighter_pos(&self, i: i32) -> Vector2 {
        self.fighter(i)
            .map_or(Vector2::ZERO, |fi| Vector2::new(f(fi.pos.x), f(fi.pos.y)))
    }

    #[func]
    fn fighter_vel(&self, i: i32) -> Vector2 {
        self.fighter(i)
            .map_or(Vector2::ZERO, |fi| Vector2::new(f(fi.vel.x), f(fi.vel.y)))
    }

    #[func]
    fn fighter_dodge_dir(&self, i: i32) -> Vector2 {
        self.fighter(i).map_or(Vector2::ZERO, |fi| {
            Vector2::new(f(fi.dodge_dir.x), f(fi.dodge_dir.y))
        })
    }

    #[func]
    fn fighter_state(&self, i: i32) -> GString {
        let name = self
            .fighter(i)
            .map_or(String::new(), |fi| format!("{:?}", fi.state));
        GString::from(name.as_str())
    }

    #[func]
    fn fighter_state_frame(&self, i: i32) -> i32 {
        self.fighter(i).map_or(0, |fi| i32::from(fi.state_frame))
    }

    #[func]
    fn fighter_facing(&self, i: i32) -> i32 {
        self.fighter(i).map_or(1, |fi| i32::from(fi.facing))
    }

    /// [char_id, platform, air_jumps_left, air_dodge_used, fast_fall, ledge, ledge_invuln,
    ///  ledge_grab_count, lag, ledge_cooldown, platform_ignore]
    #[func]
    fn fighter_info(&self, i: i32) -> PackedInt32Array {
        let v: Vec<i32> = self.fighter(i).map_or(vec![0; 11], |fi| {
            vec![
                i32::from(fi.char_id),
                i32::from(fi.platform),
                i32::from(fi.air_jumps_left),
                i32::from(fi.air_dodge_used),
                i32::from(fi.fast_fall),
                i32::from(fi.ledge),
                i32::from(fi.ledge_invuln),
                i32::from(fi.ledge_grab_count),
                i32::from(fi.lag),
                i32::from(fi.ledge_cooldown),
                i32::from(fi.platform_ignore),
            ]
        });
        PackedInt32Array::from(v.as_slice())
    }

    /// The fighter's class: the index of the moveset it uses (0 longsword, 1 claws, 2 maul, or a custom weapon).
    #[func]
    fn fighter_class(&self, i: i32) -> i32 {
        self.fighter(i).map_or(0, |fi| {
            i32::from(sim_core::combat::params_of(&self.content, fi).weapon)
        })
    }

    /// [ecb_half_width, ecb_height, ecb_side_height, wavedash_min_down, ground_assist_dist]
    #[func]
    fn fighter_body(&self, i: i32) -> PackedFloat32Array {
        let v: Vec<f32> = self.fighter(i).map_or(vec![0.0; 5], |fi| {
            let p = &self.content.fighters[usize::from(fi.char_id)];
            vec![
                f(p.ecb_half_width),
                f(p.ecb_height),
                f(p.ecb_side_height),
                f(p.wavedash_min_down),
                f(p.ground_assist_dist),
            ]
        });
        PackedFloat32Array::from(v.as_slice())
    }

    // ---- Combat (read-only, for drawing) ----

    #[func]
    fn fighter_percent(&self, i: i32) -> f32 {
        self.fighter(i).map_or(0.0, |fi| f(fi.percent))
    }

    /// [stocks, hitlag, hitstun, move_id, tumble, invuln, launch_pending, state_frame]
    #[func]
    fn fighter_combat(&self, i: i32) -> PackedInt32Array {
        let v: Vec<i32> = self.fighter(i).map_or(vec![0; 8], |fi| {
            vec![
                i32::from(fi.stocks),
                i32::from(fi.hitlag),
                i32::from(fi.hitstun),
                i32::from(fi.move_id),
                i32::from(fi.tumble),
                i32::from(fi.invuln),
                i32::from(fi.launch_pending),
                i32::from(fi.state_frame),
            ]
        });
        PackedInt32Array::from(v.as_slice())
    }

    /// Frames the fighter's current smash attack has been charged.
    #[func]
    fn fighter_charge(&self, i: i32) -> i32 {
        self.fighter(i).map_or(0, |fi| i32::from(fi.charge))
    }

    #[func]
    fn fighter_kb_vel(&self, i: i32) -> Vector2 {
        self.fighter(i).map_or(Vector2::ZERO, |fi| {
            Vector2::new(f(fi.kb_vel.x), f(fi.kb_vel.y))
        })
    }

    /// Name of the move the fighter is performing, or an empty string.
    #[func]
    fn fighter_move_name(&self, i: i32) -> GString {
        let name = self.fighter(i).map_or("", |fi| {
            if fi.state == sim_core::state::FighterState::Attack {
                sim_core::moves::MoveId::from_index(fi.move_id).name()
            } else {
                ""
            }
        });
        GString::from(name)
    }

    /// [total_frames, active_start, active_end] of the current move, or zeros.
    #[func]
    fn fighter_move_timing(&self, i: i32) -> PackedInt32Array {
        let v: Vec<i32> = self.fighter(i).map_or(vec![0; 3], |fi| {
            if fi.state != sim_core::state::FighterState::Attack {
                return vec![0; 3];
            }
            let params = sim_core::combat::params_of(&self.content, fi);
            let mv = sim_core::combat::weapon_of(&self.content, params).get(fi.move_id);
            let start = mv.hitboxes.iter().map(|h| h.start).min().unwrap_or(0);
            let end = mv.hitboxes.iter().map(|h| h.end).max().unwrap_or(0);
            vec![i32::from(mv.total_frames), i32::from(start), i32::from(end)]
        });
        PackedInt32Array::from(v.as_slice())
    }

    /// Active hitboxes this frame as flat [x, y, radius, priority] groups.
    #[func]
    fn fighter_hitboxes(&self, i: i32) -> PackedFloat32Array {
        let mut v: Vec<f32> = Vec::new();
        if let Some(fi) = self.fighter(i) {
            if fi.state == sim_core::state::FighterState::Attack {
                let params = sim_core::combat::params_of(&self.content, fi);
                let mv = sim_core::combat::weapon_of(&self.content, params).get(fi.move_id);
                for (_, hb, center) in
                    sim_core::combat::active_hitboxes(fi, mv, params.hitbox_scale)
                {
                    v.extend([
                        f(center.x),
                        f(center.y),
                        f(hb.radius),
                        f32::from(hb.priority),
                    ]);
                }
            }
        }
        PackedFloat32Array::from(v.as_slice())
    }

    /// Hurtbox circles as flat [x, y, radius] groups.
    #[func]
    fn fighter_hurtboxes(&self, i: i32) -> PackedFloat32Array {
        let mut v: Vec<f32> = Vec::new();
        if let Some(fi) = self.fighter(i) {
            let params = sim_core::combat::params_of(&self.content, fi);
            for (c, r) in sim_core::combat::hurtboxes(fi, params) {
                v.extend([f(c.x), f(c.y), f(r)]);
            }
        }
        PackedFloat32Array::from(v.as_slice())
    }

    /// The current move's aim point, relative to the feet with +x forward: (x, y, radius) of its
    /// sweet-spot hitbox (lowest priority number), or the projectile muzzle. Zeros when not attacking.
    #[func]
    fn fighter_move_tip(&self, i: i32) -> Vector3 {
        let Some(fi) = self.fighter(i) else {
            return Vector3::ZERO;
        };
        if fi.state != sim_core::state::FighterState::Attack {
            return Vector3::ZERO;
        }
        let params = sim_core::combat::params_of(&self.content, fi);
        let mv = sim_core::combat::weapon_of(&self.content, params).get(fi.move_id);
        let k = params.hitbox_scale;
        // While a hitbox is live the tip is where it is now, so a move that sweeps across several hitboxes draws its arc; before, it is
        // the move's first hitbox, and after, its last (so a swing finishes where it ended rather than snapping back to its start).
        let live =
            sim_core::combat::active_hitboxes(fi, mv, k).min_by_key(|(_, hb, _)| hb.priority);
        if let Some((_, hb, center)) = live {
            let facing = Fx::from_int(i32::from(fi.facing));
            return Vector3::new(
                f((center.x - fi.pos.x) * facing),
                f(center.y - fi.pos.y),
                f(hb.radius),
            );
        }
        let done = mv
            .hitboxes
            .iter()
            .all(|h| fi.state_frame > u16::from(h.end));
        let pick = if done && !mv.hitboxes.is_empty() {
            let top = mv.hitboxes.iter().map(|h| h.priority).min().unwrap_or(0);
            mv.hitboxes
                .iter()
                .filter(|h| h.priority == top)
                .max_by_key(|h| h.end)
        } else {
            mv.hitboxes.iter().min_by_key(|h| (h.priority, h.start))
        };
        if let Some(hb) = pick {
            return Vector3::new(f(hb.x * k), f(hb.y * k), f(hb.radius * k));
        }
        if let Some(p) = &mv.projectile {
            return Vector3::new(f(p.x * k), f(p.y * k), f(p.hitbox.radius * k));
        }
        Vector3::ZERO
    }

    /// The fighter's body and attack scale (1.0 is the moveset as written).
    #[func]
    fn fighter_scale(&self, i: i32) -> f32 {
        self.fighter(i).map_or(1.0, |fi| {
            f(sim_core::combat::params_of(&self.content, fi).hitbox_scale)
        })
    }

    /// How far out the weapon reaches at rest (the forward tilt's furthest hitbox edge).
    #[func]
    fn fighter_weapon_reach(&self, i: i32) -> f32 {
        let Some(fi) = self.fighter(i) else {
            return 0.0;
        };
        let params = sim_core::combat::params_of(&self.content, fi);
        let mv = sim_core::combat::weapon_of(&self.content, params)
            .get(sim_core::moves::MoveId::FTilt as u8);
        mv.hitboxes
            .iter()
            .map(|h| f((h.x.abs() + h.radius) * params.hitbox_scale))
            .fold(0.0, f32::max)
    }

    /// Projectile slots as flat [active, x, y, direction] groups, one per slot, in fixed order.
    #[func]
    fn projectile_slots(&self) -> PackedFloat32Array {
        let mut v: Vec<f32> = Vec::new();
        for p in &self.state.projectiles {
            v.extend([
                if p.active { 1.0 } else { 0.0 },
                f(p.pos.x),
                f(p.pos.y),
                if p.vel.x < Fx::ZERO { -1.0 } else { 1.0 },
            ]);
        }
        PackedFloat32Array::from(v.as_slice())
    }

    // ---- Network play ----
    //
    // Call one of the `net_host*` / `net_join*` functions, then `net_update` once per frame with the local
    // player's input. Everything else (`fighter_*`, `frame`, ...) then reads the session's state.

    /// Hosts a match over UDP on `port`. Returns an error message, or an empty string on success.
    #[func]
    fn net_host(&mut self, port: i32, chars: PackedInt32Array, input_delay: i32) -> GString {
        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port.clamp(0, 65535) as u16,
        );
        // Spectators connect to the next port (best effort: the match is played with or without them).
        let watch_addr = SocketAddr::new(addr.ip(), addr.port().wrapping_add(1));
        self.spectators = if port > 0 {
            SpectatorSocket::bind(watch_addr, 8)
                .ok()
                .map(|sock| (SpectatorServer::new(&Content::placeholder()), sock))
        } else {
            None
        };
        match UdpLink::bind(addr, None) {
            Ok(link) => self.net_start_host(NetLink::Udp(link), &chars, input_delay),
            Err(e) => GString::from(format!("cannot listen on port {port}: {e}").as_str()),
        }
    }

    /// Joins a match hosted at `addr` ("ip:port").
    #[func]
    fn net_join(&mut self, addr: GString) -> GString {
        let target = match resolve(&addr.to_string()) {
            Ok(a) => a,
            Err(e) => return GString::from(e.as_str()),
        };
        match UdpLink::bind(any_local(), Some(target)) {
            Ok(link) => self.net_start_join(NetLink::Udp(link)),
            Err(e) => GString::from(format!("cannot open a socket: {e}").as_str()),
        }
    }

    /// Hosts through a relay server (for players who cannot connect directly).
    #[func]
    fn net_host_relay(
        &mut self,
        relay: GString,
        room: i64,
        chars: PackedInt32Array,
        input_delay: i32,
    ) -> GString {
        match self.relay_link(&relay.to_string(), room) {
            Ok(link) => self.net_start_host(NetLink::Relay(link), &chars, input_delay),
            Err(e) => GString::from(e.as_str()),
        }
    }

    #[func]
    fn net_join_relay(&mut self, relay: GString, room: i64) -> GString {
        match self.relay_link(&relay.to_string(), room) {
            Ok(link) => self.net_start_join(NetLink::Relay(link)),
            Err(e) => GString::from(e.as_str()),
        }
    }

    fn relay_link(&self, relay: &str, room: i64) -> Result<RelayLink, String> {
        let mut link = RelayLink::connect(any_local(), resolve(relay)?, room as u64)
            .map_err(|e| format!("cannot open a socket: {e}"))?;
        link.announce();
        Ok(link)
    }

    fn net_start_host(
        &mut self,
        link: NetLink,
        chars: &PackedInt32Array,
        input_delay: i32,
    ) -> GString {
        let mut ids = [0u8, 1, 0, 1];
        for (slot, c) in ids.iter_mut().zip(chars.as_slice()) {
            *slot = (*c).clamp(0, self.content.fighters.len() as i32 - 1) as u8;
        }
        // The seed only needs to differ between matches; both peers then use the same one.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        let setup = Setup {
            seed,
            chars: ids,
            active: 0b0011,
            input_delay: input_delay.clamp(0, 8) as u8,
            cosmetics: self.my_cosmetics.clone(),
            fighter: self.my_fighter.clone(),
            ranked: self.ranked,
            rules: self.match_rules,
            stage: self.match_stage,
        };
        self.net = Some(Peer::host(link, &self.content, setup));
        self.net_log.clear();
        GString::new()
    }

    fn net_start_join(&mut self, link: NetLink) -> GString {
        self.net = Some(Peer::join_with_fighter(
            link,
            &self.content,
            self.my_cosmetics.clone(),
            self.my_fighter.clone(),
        ));
        self.net_log.clear();
        GString::new()
    }

    /// One frame of network play: -1 not networked, 0 still connecting, 1 simulated a frame, 2 waiting for the other
    /// player (nothing simulated), 3 refused (different version or content).
    #[func]
    fn net_update(&mut self, stick_x: i32, stick_y: i32, button_mask: i32) -> i32 {
        let Some(peer) = self.net.as_mut() else {
            return -1;
        };
        let input = Input {
            stick_x: clamp_i8(stick_x),
            stick_y: clamp_i8(stick_y),
            buttons: button_mask as u16,
        };
        // The handshake always builds a match from the base roster, even for a rematch.
        let base = self.base_content.as_ref().unwrap_or(&self.content);
        let status = peer.update(base, input);
        if let Some(s) = peer.state() {
            self.state = *s;
        }
        // Between two matches the roster goes back to the base one until the next match builds its own.
        if peer.match_content().is_none() {
            if let Some(base) = self.base_content.take() {
                self.content = base;
            }
        }
        // A match with made fighters runs on the base roster plus theirs: show and read that.
        if self.base_content.is_none() {
            if let Some(c) = peer.match_content() {
                self.base_content = Some(std::mem::replace(&mut self.content, c.clone()));
            }
        }
        self.record_online();
        self.serve_spectators();
        let Some(peer) = self.net.as_mut() else {
            return -1;
        };
        for e in peer.drain_events() {
            self.net_log.push(match e {
                Event::Desync {
                    frame,
                    local,
                    remote,
                } => {
                    format!("DESYNC at frame {frame}: local {local:016x}, remote {remote:016x}")
                }
                Event::Disconnected { player } => format!("player {} disconnected", player + 1),
            });
        }
        match status {
            Status::Connecting => 0,
            Status::Running(Advance::Ran) => 1,
            Status::Running(Advance::Stalled) => 2,
            Status::Rejected {
                reason,
                their_version,
                their_hash,
            } => {
                if reason == netplay::packet::RejectReason::BadFighter {
                    self.net_log.push(
                        "refused: a fighter in this match is not allowed (over the point budget under ranked rules, or not one this game has)"
                            .to_string(),
                    );
                } else {
                    self.net_log.push(format!(
                        "refused ({reason:?}): the other side has sim version {their_version}, content {their_hash:016x}; ours is {SIM_VERSION}, {:016x}",
                        self.content.hash()
                    ));
                }
                3
            }
        }
    }

    /// Hosts a group match (up to four players) on UDP `port`; guests join, and `group_start_match` begins it. Uses this player's
    /// fighter (`set_fighter`), cosmetics, rules, stage and ranked setting. Returns an error text or an empty string.
    #[func]
    fn group_host_start(&mut self, port: i32, input_delay: i32) -> GString {
        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port.clamp(0, 65535) as u16,
        );
        let socket = match SpectatorSocket::bind(addr, 3) {
            Ok(s) => s,
            Err(e) => return GString::from(format!("cannot listen on port {port}: {e}").as_str()),
        };
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        let params = GroupParams {
            seed,
            input_delay: input_delay.clamp(0, 8) as u8,
            rules: self.match_rules,
            stage: self.match_stage,
            ranked: self.ranked,
        };
        let base = Content::placeholder();
        self.net = None;
        self.group_guest = None;
        self.group_host = Some(GroupHost::new(
            socket,
            &base,
            params,
            self.my_fighter.clone(),
            self.my_cosmetics.clone(),
        ));
        // Spectators connect to the next port.
        let watch_addr = SocketAddr::new(addr.ip(), addr.port().wrapping_add(1));
        self.spectators = if port > 0 {
            SpectatorSocket::bind(watch_addr, 8)
                .ok()
                .map(|sock| (SpectatorServer::new(&base), sock))
        } else {
            None
        };
        self.group_log.clear();
        GString::new()
    }

    /// Joins a group match hosted at `addr` ("ip:port"). Returns an error text or an empty string.
    #[func]
    fn group_join(&mut self, addr: GString) -> GString {
        let target = match resolve(&addr.to_string()) {
            Ok(a) => a,
            Err(e) => return GString::from(e.as_str()),
        };
        match UdpLink::bind(any_local(), Some(target)) {
            Ok(link) => {
                let base = Content::placeholder();
                self.net = None;
                self.group_host = None;
                self.group_guest = Some(GroupGuest::new(
                    link,
                    &base,
                    self.my_fighter.clone(),
                    self.my_cosmetics.clone(),
                ));
                self.group_log.clear();
                GString::new()
            }
            Err(e) => GString::from(format!("cannot open a socket: {e}").as_str()),
        }
    }

    /// The host starts the match with whoever has joined.
    #[func]
    fn group_start_match(&mut self) {
        if let Some(h) = self.group_host.as_mut() {
            h.start();
        }
    }

    /// The host starts another match with the same players.
    #[func]
    fn group_restart(&mut self) {
        if let Some(h) = self.group_host.as_mut() {
            h.restart();
        }
    }

    /// One frame of a group match: -1 not in a group, 0 in the lobby, 1 simulated a frame, 2 waiting for the others, 3 refused,
    /// 4 the host is starting the match.
    #[func]
    fn group_update(&mut self, stick_x: i32, stick_y: i32, button_mask: i32) -> i32 {
        let input = Input {
            stick_x: clamp_i8(stick_x),
            stick_y: clamp_i8(stick_y),
            buttons: button_mask as u16,
        };
        let base = self
            .base_content
            .clone()
            .unwrap_or_else(|| self.content.clone());
        let (rec_seed, rec_len) = self
            .recorder
            .as_ref()
            .map_or((None, 0), |r| (Some(r.record.seed), r.len()));
        let (status, session_state, match_content, setup, events, frames) = if let Some(h) =
            self.group_host.as_mut()
        {
            let st = h.update(&base, input);
            let setup = h.match_setup().cloned();
            let from = setup
                .as_ref()
                .map_or(0, |s| if Some(s.seed) == rec_seed { rec_len } else { 0 });
            let frames = h
                .session()
                .map(|s| confirmed_since(s, from))
                .unwrap_or_default();
            (
                st,
                h.state().copied(),
                h.match_content().cloned(),
                setup,
                h.drain_events(),
                frames,
            )
        } else if let Some(g) = self.group_guest.as_mut() {
            let st = g.update(&base, input);
            let setup = g.match_setup().cloned();
            let from = setup
                .as_ref()
                .map_or(0, |s| if Some(s.seed) == rec_seed { rec_len } else { 0 });
            let frames = g
                .session()
                .map(|s| confirmed_since(s, from))
                .unwrap_or_default();
            (
                st,
                g.state().copied(),
                g.match_content().cloned(),
                setup,
                g.drain_events(),
                frames,
            )
        } else {
            return -1;
        };
        if let Some(s) = session_state {
            self.state = s;
        }
        // The match runs on the content the host built (the base roster, everyone's fighters and the stage).
        if let Some(c) = match_content.as_ref() {
            if self.base_content.is_none() {
                self.base_content = Some(std::mem::replace(&mut self.content, c.clone()));
            } else if self.content.hash() != c.hash() {
                self.content = c.clone();
            }
            if let Some(setup) = setup {
                self.players = setup.active;
                self.match_rules = setup.rules;
                let from = if Some(setup.seed) == rec_seed {
                    rec_len
                } else {
                    0
                };
                self.record_group(&setup, from, frames);
                self.serve_spectators();
            }
        }
        for e in events {
            self.group_log.push(match e {
                Event::Desync { frame, .. } => format!("DESYNC at frame {frame}"),
                Event::Disconnected { player } => format!("player {} disconnected", player + 1),
            });
        }
        match status {
            GroupStatus::Lobby => 0,
            GroupStatus::Starting => 4,
            GroupStatus::Running(Advance::Ran) => 1,
            GroupStatus::Running(Advance::Stalled) => 2,
            GroupStatus::Rejected(reason) => {
                self.group_log.push(format!("refused: {reason:?}"));
                3
            }
        }
    }

    /// The players' name-and-look bytes by slot (empty where nobody is), as the lobby or the running match has them.
    #[func]
    fn group_lobby(&self) -> Array<PackedByteArray> {
        let list: Vec<Vec<u8>> = if let Some(h) = self.group_host.as_ref() {
            h.match_setup()
                .map_or_else(|| h.lobby(), |s| s.cosmetics.clone())
        } else if let Some(g) = self.group_guest.as_ref() {
            g.lobby().to_vec()
        } else {
            Vec::new()
        };
        let mut out = Array::new();
        for c in list {
            out.push(&PackedByteArray::from(c.as_slice()));
        }
        out
    }

    /// This player's slot in the group (0 for the host), or -1.
    #[func]
    fn group_slot(&self) -> i32 {
        if self.group_host.is_some() {
            0
        } else {
            self.group_guest
                .as_ref()
                .and_then(|g| g.slot())
                .map_or(-1, i32::from)
        }
    }

    /// How many players are in the lobby (host) or were told to be (guest).
    #[func]
    fn group_players(&self) -> i32 {
        if let Some(h) = self.group_host.as_ref() {
            h.players_joined() as i32
        } else {
            self.group_guest.as_ref().map_or(0, |g| {
                g.lobby().iter().filter(|c| !c.is_empty()).count() as i32
            })
        }
    }

    #[func]
    fn group_take_log(&mut self) -> PackedStringArray {
        let lines: Vec<GString> = self
            .group_log
            .drain(..)
            .map(|l| GString::from(l.as_str()))
            .collect();
        PackedStringArray::from(lines.as_slice())
    }

    /// Starts watching a match hosted at `addr` ("ip:port", the port the players use; spectators connect to the next one).
    /// Returns an error text or an empty string.
    #[func]
    fn spectate_start(&mut self, addr: GString) -> GString {
        let target = match resolve(&addr.to_string()) {
            Ok(a) => SocketAddr::new(a.ip(), a.port().wrapping_add(1)),
            Err(e) => return GString::from(e.as_str()),
        };
        match UdpLink::bind(any_local(), Some(target)) {
            Ok(link) => {
                self.net = None;
                self.watching = Some((SpectatorClient::new(&Content::placeholder()), link));
                self.watching_seed = 0;
                GString::new()
            }
            Err(e) => GString::from(format!("cannot open a socket: {e}").as_str()),
        }
    }

    /// One frame of watching: -1 not watching, 0 connecting, 1 played a frame, 2 waiting for the host's frames, 3 refused.
    #[func]
    fn spectate_update(&mut self) -> i32 {
        let Some((client, link)) = self.watching.as_mut() else {
            return -1;
        };
        while let Some(bytes) = link.recv() {
            client.handle(&bytes);
        }
        for m in client.tick() {
            link.send(&m);
        }
        if client.refused().is_some() {
            return 3;
        }
        if !client.ready() {
            return 0;
        }
        // A new match (another seed) replaces the one on screen.
        let seed = client.record().map_or(0, |r| r.seed);
        if seed != self.watching_seed {
            self.watching_seed = seed;
            if let Some(c) = client.content() {
                self.content = c.clone();
            }
            if let Some(r) = client.record() {
                self.players = r.active;
                self.match_rules = r.rules;
            }
        }
        // Stay a little behind the host so the stream never runs dry; catch up quickly if far behind.
        let behind = client.behind();
        let over = client.state().is_some_and(|s| s.winner != PLAYING);
        let budget = match behind {
            0..=9 if !over => 0,
            10..=39 => 1,
            40..=179 => 3,
            _ => 8,
        };
        let played = client.advance(budget.max(u32::from(over)));
        if let Some(s) = client.state() {
            self.state = *s;
        }
        if played > 0 {
            1
        } else {
            2
        }
    }

    /// The watched match's player profile bytes (name and look) for player `i`, once known.
    #[func]
    fn spectate_cosmetics(&self, i: i32) -> PackedByteArray {
        let bytes = self
            .watching
            .as_ref()
            .and_then(|(c, _)| c.record())
            .and_then(|r| usize::try_from(i).ok().and_then(|i| r.cosmetics.get(i)))
            .cloned()
            .unwrap_or_default();
        PackedByteArray::from(bytes.as_slice())
    }

    /// Frames the watcher has received but not shown yet.
    #[func]
    fn spectate_behind(&self) -> i32 {
        self.watching.as_ref().map_or(0, |(c, _)| c.behind() as i32)
    }

    #[func]
    fn spectate_stop(&mut self) {
        self.watching = None;
    }

    /// How many people are watching the match this player hosts.
    #[func]
    fn spectator_count(&self) -> i32 {
        self.spectators
            .as_ref()
            .map_or(0, |(s, _)| s.spectator_count() as i32)
    }

    /// After a match: asks the other player for another one. It starts when both have asked (`net_update` then reports
    /// 0 while the new handshake runs, and 1 again once the new match is on).
    #[func]
    fn net_request_rematch(&mut self) {
        if let Some(p) = self.net.as_mut() {
            p.request_rematch();
        }
    }

    /// `[this player has asked, the other player has asked]` as 0 or 1.
    #[func]
    fn net_rematch_state(&self) -> PackedInt32Array {
        let (me, them) = self
            .net
            .as_ref()
            .map_or((false, false), |p| p.rematch_state());
        PackedInt32Array::from([i32::from(me), i32::from(them)].as_slice())
    }

    /// Which player this peer controls (0 for the host, 1 for the joiner), or -1.
    #[func]
    fn net_local_player(&self) -> i32 {
        self.net.as_ref().map_or(-1, |p| p.local_player() as i32)
    }

    /// A line of status text for the overlay.
    #[func]
    fn net_info(&self) -> GString {
        let Some(p) = self.net.as_ref() else {
            return GString::new();
        };
        let text = match p.session() {
            None => "NET: connecting...".to_string(),
            Some(s) => {
                let st = s.stats();
                format!(
                    "NET P{}  frame {}  confirmed {}  rollbacks {} (longest {})  stalls {}",
                    p.local_player() + 1,
                    s.frame(),
                    s.confirmed_frame().min(s.frame()),
                    st.rollbacks,
                    st.longest_rollback,
                    st.stalls
                )
            }
        };
        GString::from(text.as_str())
    }

    /// Messages about desyncs, disconnects and refusals since the last call.
    #[func]
    fn net_take_log(&mut self) -> PackedStringArray {
        let lines: Vec<GString> = self
            .net_log
            .drain(..)
            .map(|l| GString::from(l.as_str()))
            .collect();
        PackedStringArray::from(lines.as_slice())
    }

    /// Starts looking for an opponent through the relay at `relay` ("ip:port"). Returns an error text or an empty string.
    #[func]
    fn quickmatch_start(&mut self, relay: GString) -> GString {
        let target = match resolve(&relay.to_string()) {
            Ok(a) => a,
            Err(e) => return GString::from(e.as_str()),
        };
        match QuickMatch::connect(any_local(), target) {
            Ok(q) => {
                self.quick = Some(q);
                GString::new()
            }
            Err(e) => GString::from(format!("cannot open a socket: {e}").as_str()),
        }
    }

    /// Call once a frame while looking: an empty dictionary until an opponent is found, then `{room, host}`.
    #[func]
    fn quickmatch_poll(&mut self) -> VarDictionary {
        let mut out = VarDictionary::new();
        if let Some((room, host)) = self.quick.as_mut().and_then(|q| q.poll()) {
            self.quick = None;
            out.set("room", room as i64);
            out.set("host", host);
        }
        out
    }

    /// Stops looking.
    #[func]
    fn quickmatch_stop(&mut self) {
        self.quick = None;
    }

    #[func]
    fn net_leave(&mut self) {
        self.quick = None;
        if let Some(mut p) = self.net.take() {
            p.leave();
        }
        self.spectators = None;
        self.watching = None;
        if let Some(mut g) = self.group_host.take() {
            g.leave();
        }
        if let Some(mut g) = self.group_guest.take() {
            g.leave();
        }
        if let Some(base) = self.base_content.take() {
            self.content = base;
        }
    }

    /// Training mode: where fighter `player` will be over its next `frames` frames if it holds the stick at
    /// (`stick_x`, `stick_y`) and everyone else lets go of their controls. Runs on a copy of the state, so it
    /// is exactly what the sim would do (launch, DI, gravity, landing).
    #[func]
    fn predict_path(
        &self,
        player: i32,
        frames: i32,
        stick_x: i32,
        stick_y: i32,
    ) -> PackedVector2Array {
        let Some(i) = usize::try_from(player).ok().filter(|i| *i < MAX_FIGHTERS) else {
            return PackedVector2Array::new();
        };
        let mut s = self.state;
        let mut out: Vec<Vector2> = Vec::new();
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[i] = Input {
            stick_x: clamp_i8(stick_x),
            stick_y: clamp_i8(stick_y),
            buttons: 0,
        };
        for _ in 0..frames.clamp(0, 240) {
            step(&mut s, &self.content, &inputs);
            let p = s.fighters[i].pos;
            out.push(Vector2::new(f(p.x), f(p.y)));
        }
        PackedVector2Array::from(out.as_slice())
    }

    /// Whether the launch fighter `player` is in will knock them out if nobody touches a button: plays a copy of the match forward
    /// up to `frames` frames. Presentation only (the camera's knock-out zoom); the match itself is untouched.
    #[func]
    fn fighter_will_ko(&self, player: i32, frames: i32) -> bool {
        let Some(i) = usize::try_from(player).ok().filter(|i| *i < MAX_FIGHTERS) else {
            return false;
        };
        let mut s = self.state;
        let stocks = s.fighters[i].stocks;
        if !s.fighters[i].active {
            return false;
        }
        let inputs = [Input::default(); MAX_FIGHTERS];
        for _ in 0..frames.clamp(0, 300) {
            step(&mut s, &self.content, &inputs);
            if s.fighters[i].stocks < stocks
                || !s.fighters[i].active
                || s.fighters[i].state == sim_core::state::FighterState::Respawn
            {
                return true;
            }
            if s.fighters[i].state != sim_core::state::FighterState::Hitstun
                && s.fighters[i].hitlag == 0
            {
                return false;
            }
        }
        false
    }

    /// [knockback of the last launch, its angle in degrees, hitstun frames left]
    #[func]
    fn fighter_launch(&self, i: i32) -> PackedFloat32Array {
        let v: Vec<f32> = self.fighter(i).map_or(vec![0.0; 3], |fi| {
            vec![
                f(fi.launch_kb),
                f32::from(fi.launch_angle) * 360.0 / 4096.0,
                f32::from(fi.hitstun),
            ]
        });
        PackedFloat32Array::from(v.as_slice())
    }

    /// Shield health, 0 to the ruleset's maximum.
    #[func]
    fn fighter_shield(&self, i: i32) -> f32 {
        self.fighter(i).map_or(0.0, |fi| f(fi.shield_hp))
    }

    /// Full shield health (for drawing the bubble at the right size).
    #[func]
    fn shield_max(&self) -> f32 {
        f(self.content.rules.shield_max)
    }

    /// Debug: set a fighter's shield health.
    #[func]
    fn debug_set_shield(&mut self, player: i32, hp: f32) {
        self.recorder = None;
        if let Some(fi) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fi.shield_hp = Fx::from_raw((hp.clamp(0.0, 999.0) * 65536.0) as i32);
        }
    }

    /// Debug: set a fighter's damage percent.
    #[func]
    fn debug_set_percent(&mut self, player: i32, percent: f32) {
        self.recorder = None;
        if let Some(fi) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fi.percent = Fx::from_raw((percent.clamp(0.0, 999.0) * 65536.0) as i32);
        }
    }

    /// Debug: stand a fighter on the main stage at x, facing left (-1) or right (1), idle.
    #[func]
    fn debug_stand(&mut self, player: i32, x: f32, facing: i32) {
        self.recorder = None;
        if let Some(fi) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fi.pos = sim_core::Vec2::new(Fx::from_raw((x * 65536.0) as i32), Fx::ZERO);
            fi.vel = sim_core::Vec2::ZERO;
            fi.kb_vel = sim_core::Vec2::ZERO;
            fi.platform = 0;
            fi.state = sim_core::state::FighterState::Idle;
            fi.state_frame = 0;
            fi.hitlag = 0;
            fi.hitstun = 0;
            fi.launch_pending = false;
            fi.facing = if facing < 0 { -1 } else { 1 };
        }
    }

    // ---- Stage (read-only) ----

    #[func]
    fn platform_count(&self) -> i32 {
        self.content.stage.platforms.len() as i32
    }

    /// Chooses the stage for the next match (0 is the base roster's own); out-of-range values are ignored.
    #[func]
    fn set_match_stage(&mut self, index: i32) {
        if (0..i32::from(stages::COUNT)).contains(&index) {
            self.match_stage = index as u8;
        }
    }

    /// Swaps stage `index` into the loaded content right away (training and demos: a match from the menus builds its own content).
    /// Returns false for a stage that does not exist.
    #[func]
    fn use_stage(&mut self, index: i32) -> bool {
        let Ok(i) = u8::try_from(index) else {
            return false;
        };
        match stages::with_stage(&self.content, i) {
            Some(c) => {
                self.content = c;
                true
            }
            None => false,
        }
    }

    #[func]
    fn stage_count(&self) -> i32 {
        i32::from(stages::COUNT)
    }

    #[func]
    fn stage_name(&self, index: i32) -> GString {
        GString::from(stages::name(index.clamp(0, 255) as u8))
    }

    /// How the current stage looks: {backdrop, sky_top, sky_bottom} (the colours as `rrggbb`, empty for the backdrop's own).
    #[func]
    fn stage_look(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        out.set("backdrop", self.content.look.backdrop.as_str());
        out.set("sky_top", self.content.look.sky_top.as_str());
        out.set("sky_bottom", self.content.look.sky_bottom.as_str());
        out
    }

    /// The backdrops a stage can use.
    #[func]
    fn stage_backdrops(&self) -> PackedStringArray {
        let names: Vec<GString> = sim_content::BACKDROPS
            .iter()
            .map(|n| GString::from(*n))
            .collect();
        PackedStringArray::from(names.as_slice())
    }

    #[func]
    fn stage_blurb(&self, index: i32) -> GString {
        let blurb = stages::BLURBS.get(usize::try_from(index).unwrap_or(usize::MAX));
        GString::from(blurb.copied().unwrap_or(""))
    }

    /// Stage `index` as a list of `[left, right, top, bottom, pass_through]` rectangles, the blast zone `[l, r, bottom, top]`
    /// last, for drawing a preview without loading the stage.
    #[func]
    fn stage_preview(&self, index: i32) -> Array<PackedFloat32Array> {
        let mut out = Array::new();
        let Some(stage) = stages::preset(index.clamp(0, 255) as u8) else {
            return out;
        };
        for p in &stage.platforms {
            let v = [
                f(p.left),
                f(p.right),
                f(p.y),
                f(p.bottom),
                if p.pass_through { 1.0 } else { 0.0 },
            ];
            out.push(&PackedFloat32Array::from(v.as_slice()));
        }
        let blast = [
            f(stage.blast_left),
            f(stage.blast_right),
            f(stage.blast_bottom),
            f(stage.blast_top),
        ];
        out.push(&PackedFloat32Array::from(blast.as_slice()));
        out
    }

    /// [left, right, top_y, bottom_y, pass_through (1 or 0)]
    #[func]
    fn platform_rect(&self, i: i32) -> PackedFloat32Array {
        let v: Vec<f32> = usize::try_from(i)
            .ok()
            .and_then(|i| self.content.stage.platforms.get(i))
            .map_or(vec![0.0; 5], |p| {
                vec![
                    f(p.left),
                    f(p.right),
                    f(p.y),
                    f(p.bottom),
                    if p.pass_through { 1.0 } else { 0.0 },
                ]
            });
        PackedFloat32Array::from(v.as_slice())
    }

    #[func]
    fn ledge_count(&self) -> i32 {
        self.content.stage.ledges.len() as i32
    }

    /// (x, y, side)
    #[func]
    fn ledge_pos(&self, i: i32) -> Vector3 {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.content.stage.ledges.get(i))
            .map_or(Vector3::ZERO, |l| {
                Vector3::new(f(l.x), f(l.y), f32::from(l.side))
            })
    }

    /// Who holds ledge `i` (player index), or -1.
    #[func]
    fn ledge_owner(&self, i: i32) -> i32 {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.state.ledge_owner.get(i))
            .map_or(-1, |o| i32::from(*o))
    }

    /// [left, right, bottom, top]
    #[func]
    fn blast_zone(&self) -> PackedFloat32Array {
        let s = &self.content.stage;
        let v = [
            f(s.blast_left),
            f(s.blast_right),
            f(s.blast_bottom),
            f(s.blast_top),
        ];
        PackedFloat32Array::from(v.as_slice())
    }

    #[func]
    fn spawn_pos(&self, i: i32) -> Vector2 {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.content.stage.spawns.get(i))
            .map_or(Vector2::ZERO, |p| Vector2::new(f(p.x), f(p.y)))
    }
}

impl SimRunner {
    fn fighter(&self, i: i32) -> Option<&sim_core::Fighter> {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.state.fighters.get(i))
    }
}
