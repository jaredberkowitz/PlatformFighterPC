//! Thin Godot wrapper around `sim-core`.
//!
//! GDScript gathers input, calls `tick()` once per 60 Hz physics frame, then reads state back to
//! draw it. Nothing here decides gameplay, and animation never drives the sim. Floats appear only
//! in the read-back getters, for display.

use godot::classes::{INode, Node};
use godot::prelude::*;
use netplay::packet::Setup;
use netplay::peer::{Link, Peer, Status};
use netplay::session::{Advance, Event};
use sim_core::input::buttons;
use sim_core::{step, Content, Fx, GameState, Input, MAX_FIGHTERS, SIM_VERSION};
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use transport::{RelayLink, UdpLink};

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

fn clamp_i8(v: i32) -> i8 {
    v.clamp(-127, 127) as i8
}

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
    state: GameState,
    inputs: [Input; MAX_FIGHTERS],
    history: VecDeque<GameState>,
    /// Set while playing over the network; then `state` is a copy of the session's state.
    net: Option<Peer<NetLink>>,
    net_log: Vec<String>,
}

#[godot_api]
impl INode for SimRunner {
    fn init(base: Base<Node>) -> Self {
        let content = Content::placeholder();
        let state = GameState::new_with_active(&content, 1, [0, 1, 0, 1], 0b0011);
        SimRunner {
            base,
            content,
            content_name: String::new(),
            players: 0b0011,
            my_cosmetics: Vec::new(),
            state,
            inputs: [Input::default(); MAX_FIGHTERS],
            history: VecDeque::new(),
            net: None,
            net_log: Vec::new(),
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
        self.state = GameState::new_with_active(&self.content, 1, [0, 1, 0, 1], self.players);
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

    /// The other player's cosmetic bytes once the handshake has delivered them (empty until then).
    #[func]
    fn net_their_cosmetics(&self) -> PackedByteArray {
        self.net.as_ref().map_or_else(PackedByteArray::new, |p| {
            PackedByteArray::from(p.their_cosmetics())
        })
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
        self.state = GameState::new_with_active(&self.content, seed as u64, ids, self.players);
        self.inputs = [Input::default(); MAX_FIGHTERS];
        self.history.clear();
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
        step(&mut self.state, &self.content, &self.inputs);
    }

    /// Steps one frame backwards (training mode). Returns false if there is no history left.
    #[func]
    fn step_back(&mut self) -> bool {
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
                for (_, hb, center) in sim_core::combat::active_hitboxes(fi, mv) {
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
        if let Some(hb) = mv.hitboxes.iter().min_by_key(|h| (h.priority, h.start)) {
            return Vector3::new(f(hb.x), f(hb.y), f(hb.radius));
        }
        if let Some(p) = &mv.projectile {
            return Vector3::new(f(p.x), f(p.y), f(p.hitbox.radius));
        }
        Vector3::ZERO
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
            .map(|h| f(h.x.abs()) + f(h.radius))
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
        };
        self.net = Some(Peer::host(link, &self.content, setup));
        self.net_log.clear();
        GString::new()
    }

    fn net_start_join(&mut self, link: NetLink) -> GString {
        self.net = Some(Peer::join(link, &self.content, self.my_cosmetics.clone()));
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
        let status = peer.update(&self.content, input);
        if let Some(s) = peer.state() {
            self.state = *s;
        }
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
                self.net_log.push(format!(
                    "refused ({reason:?}): the other side has sim version {their_version}, content {their_hash:016x}; ours is {SIM_VERSION}, {:016x}",
                    self.content.hash()
                ));
                3
            }
        }
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

    #[func]
    fn net_leave(&mut self) {
        if let Some(mut p) = self.net.take() {
            p.leave();
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
