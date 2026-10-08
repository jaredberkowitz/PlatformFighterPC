//! A rollback session: one peer's view of a networked match.
//!
//! The session owns the simulation state. Each call to [`Session::advance`] takes the local player's input,
//! uses the real input for every remote player that has arrived and a *prediction* (their last known input) for
//! any that has not, and steps the sim. When a remote input arrives that differs from what was predicted, the
//! next `advance` restores the snapshot from that frame and re-simulates forward with the real inputs. If the
//! sim would run too far ahead of the newest confirmed remote input it **stalls** instead.
//!
//! It is pure: no sockets, no clocks. Packets go in through [`Session::handle_packet`] and come out of
//! [`Session::drain_outgoing`]; time is the number of `advance` calls. Both peers produce identical states for
//! every confirmed frame, and compare checksums of confirmed frames to detect a desync the moment it happens.

use crate::packet::{Packet, MAX_INPUTS_PER_PACKET};
use sim_core::{step, Content, GameState, Input, MAX_FIGHTERS};
use std::collections::{BTreeMap, BTreeSet};

/// The destination of a packet meant for every other player.
pub const BROADCAST: u8 = 255;

/// Frames of input history and snapshots kept. Must comfortably exceed `max_prediction + input_delay`.
pub const RING: usize = 128;
/// `advance` calls without any packet from a remote player before it counts as disconnected (10 s at 60 Hz).
pub const DISCONNECT_TIMEOUT: u32 = 600;
/// How often (in `advance` calls) the latest checksum is re-sent, in case a packet was lost.
const CHECKSUM_RESEND: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionConfig {
    /// Which player this peer controls.
    pub local: u8,
    /// Bit `n` set means player `n` takes part (the rest just stand there).
    pub active: u8,
    /// Local input is applied this many frames after it is read. More delay, fewer rollbacks.
    pub input_delay: u8,
    /// The sim may run at most this many frames past the newest confirmed remote input.
    pub max_prediction: u8,
    /// A checksum of the confirmed state is exchanged every this many frames.
    pub checksum_interval: u32,
}

impl SessionConfig {
    pub fn two_player(local: u8) -> SessionConfig {
        SessionConfig {
            local,
            active: 0b0011,
            input_delay: 2,
            max_prediction: 8,
            checksum_interval: 30,
        }
    }

    fn is_active(&self, p: usize) -> bool {
        self.active & (1 << p) != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The two peers disagree about the state before `frame`. The match is no longer in sync.
    Desync { frame: u32, local: u64, remote: u64 },
    /// A remote player stopped answering (or said goodbye).
    Disconnected { player: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advance {
    Ran,
    /// Too far ahead of the remote player: nothing was simulated and the local input was not consumed.
    Stalled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub rollbacks: u32,
    pub resimulated_frames: u32,
    pub longest_rollback: u32,
    pub stalls: u32,
}

#[derive(Clone, Copy)]
struct Slot {
    frame: u32,
    known: [Option<Input>; MAX_FIGHTERS],
    used: [Input; MAX_FIGHTERS],
}

impl Slot {
    fn fresh(frame: u32) -> Slot {
        Slot {
            frame,
            known: [None; MAX_FIGHTERS],
            used: [Input::default(); MAX_FIGHTERS],
        }
    }
}

pub struct Session {
    cfg: SessionConfig,
    state: GameState,
    /// The frame about to be simulated (`state` is the state before it).
    frame: u32,
    /// State before each frame, tagged with that frame.
    snaps: Vec<(u32, GameState)>,
    slots: Vec<Slot>,
    /// The next frame for which the local player's input will be supplied.
    local_next: u32,
    /// We hold every input of remote player `p` for frames below `received_upto[p]`.
    received_upto: [u32; MAX_FIGHTERS],
    /// Remote player `p` has told us it holds every one of our inputs below `acked[p]`.
    acked: [u32; MAX_FIGHTERS],
    connected: [bool; MAX_FIGHTERS],
    idle: [u32; MAX_FIGHTERS],
    pending_rollback: Option<u32>,
    advances: u32,
    next_check: u32,
    local_checksums: BTreeMap<u32, u64>,
    remote_checksums: BTreeMap<u32, u64>,
    reported: BTreeSet<u32>,
    events: Vec<Event>,
    /// Packets to send, each with the player it is for ([`BROADCAST`] for everyone).
    outbox: Vec<(u8, Vec<u8>)>,
    stats: Stats,
}

impl Session {
    pub fn new(cfg: SessionConfig, initial: GameState) -> Session {
        let mut s = Session {
            cfg,
            state: initial,
            frame: 0,
            snaps: vec![(u32::MAX, initial); RING],
            slots: vec![Slot::fresh(u32::MAX); RING],
            local_next: 0,
            received_upto: [0; MAX_FIGHTERS],
            acked: [0; MAX_FIGHTERS],
            connected: [true; MAX_FIGHTERS],
            idle: [0; MAX_FIGHTERS],
            pending_rollback: None,
            advances: 0,
            next_check: cfg.checksum_interval.max(1),
            local_checksums: BTreeMap::new(),
            remote_checksums: BTreeMap::new(),
            reported: BTreeSet::new(),
            events: Vec::new(),
            outbox: Vec::new(),
            stats: Stats::default(),
        };
        s.snaps[0] = (0, initial);
        // The first `input_delay` frames have no local input yet: they are neutral for everyone, on both peers.
        for f in 0..u32::from(cfg.input_delay) {
            s.slot_mut(f).known[usize::from(cfg.local)] = Some(Input::default());
            s.local_next = f + 1;
        }
        for p in 0..MAX_FIGHTERS {
            if !cfg.is_active(p) || p == usize::from(cfg.local) {
                s.connected[p] = p == usize::from(cfg.local);
            }
        }
        s
    }

    // ---- Read access ------------------------------------------------------------------------------------------

    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// The frame about to be simulated.
    pub fn frame(&self) -> u32 {
        self.frame
    }

    /// Every input below this frame is known, so the state before it can no longer change.
    pub fn confirmed_frame(&self) -> u32 {
        let mut c = u32::MAX;
        for p in 0..MAX_FIGHTERS {
            if !self.cfg.is_active(p) || !self.connected[p] {
                continue;
            }
            let upto = if p == usize::from(self.cfg.local) {
                self.local_next
            } else {
                self.received_upto[p]
            };
            c = c.min(upto);
        }
        c
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Checksums of confirmed frames (the state before that frame), in order. Both peers produce the same list.
    pub fn checksum_history(&self) -> Vec<(u32, u64)> {
        self.local_checksums.iter().map(|(f, c)| (*f, *c)).collect()
    }

    /// For debugging: the inputs known for `frame` so far.
    pub fn known_inputs(&self, frame: u32) -> [Option<Input>; MAX_FIGHTERS] {
        let slot = &self.slots[frame as usize % RING];
        if slot.frame == frame {
            slot.known
        } else {
            [None; MAX_FIGHTERS]
        }
    }

    /// For debugging: the inputs `frame` was last simulated with, if it is still in the ring.
    pub fn used_inputs(&self, frame: u32) -> Option<[Input; MAX_FIGHTERS]> {
        let slot = &self.slots[frame as usize % RING];
        (slot.frame == frame && frame < self.frame).then_some(slot.used)
    }

    pub fn is_connected(&self, player: usize) -> bool {
        self.connected.get(player).copied().unwrap_or(false)
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Like [`Session::drain_outgoing`], with the player each packet is for. With more than two players every remote gets its own packet
    /// (it carries that player's acknowledgement), so the caller must route them.
    pub fn drain_outgoing_to(&mut self) -> Vec<(u8, Vec<u8>)> {
        std::mem::take(&mut self.outbox)
    }

    pub fn drain_outgoing(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.outbox)
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect()
    }

    // ---- Packets in --------------------------------------------------------------------------------------------

    pub fn handle_packet(&mut self, bytes: &[u8]) {
        self.handle_packet_inner(None, bytes);
    }

    /// Like [`Session::handle_packet`] for a packet that arrived through a hub, where `from` is the player who sent it: a goodbye then
    /// only drops that player, not everyone.
    pub fn handle_packet_from(&mut self, from: u8, bytes: &[u8]) {
        self.handle_packet_inner(Some(usize::from(from)), bytes);
    }

    fn handle_packet_inner(&mut self, from: Option<usize>, bytes: &[u8]) {
        let Some(packet) = Packet::decode(bytes) else {
            return;
        };
        match packet {
            Packet::Inputs {
                player,
                ack,
                start,
                inputs,
            } => self.on_inputs(usize::from(player), ack, start, &inputs),
            Packet::Checksum { frame, value } => self.on_checksum(frame, value),
            Packet::Disconnect => match from {
                Some(p) => self.drop_player(p),
                None => {
                    for p in 0..MAX_FIGHTERS {
                        if p != usize::from(self.cfg.local) {
                            self.drop_player(p);
                        }
                    }
                }
            },
            // Handshake packets belong to the handshake, not to a running session.
            _ => {}
        }
    }

    fn on_inputs(&mut self, player: usize, ack: u32, start: u32, inputs: &[Input]) {
        if player >= MAX_FIGHTERS
            || player == usize::from(self.cfg.local)
            || !self.cfg.is_active(player)
            || !self.connected[player]
        {
            return;
        }
        self.idle[player] = 0;
        self.acked[player] = self.acked[player].max(ack);
        for (k, input) in inputs.iter().enumerate() {
            let frame = start.wrapping_add(k as u32);
            // Only frames within half a ring either side of now are accepted, so a delayed duplicate from the
            // distant past can never land on a ring slot that already holds newer inputs.
            let half = RING as u32 / 2;
            if frame.saturating_add(half) <= self.frame || frame >= self.frame.saturating_add(half)
            {
                continue;
            }
            let simulated = frame < self.frame;
            let slot = self.slot_mut(frame);
            if slot.known[player].is_some() {
                continue; // the first value for a frame wins
            }
            slot.known[player] = Some(*input);
            let wrong_guess = simulated && slot.used[player] != *input;
            if wrong_guess {
                self.pending_rollback = Some(self.pending_rollback.map_or(frame, |p| p.min(frame)));
            }
        }
        while self.known(self.received_upto[player], player).is_some() {
            self.received_upto[player] += 1;
        }
    }

    fn on_checksum(&mut self, frame: u32, value: u64) {
        self.remote_checksums.insert(frame, value);
        while self.remote_checksums.len() > 64 {
            if let Some((&oldest, _)) = self.remote_checksums.iter().next() {
                self.remote_checksums.remove(&oldest);
            }
        }
        self.compare(frame);
    }

    fn compare(&mut self, frame: u32) {
        let (Some(&local), Some(&remote)) = (
            self.local_checksums.get(&frame),
            self.remote_checksums.get(&frame),
        ) else {
            return;
        };
        if local != remote && self.reported.insert(frame) {
            self.events.push(Event::Desync {
                frame,
                local,
                remote,
            });
        }
    }

    fn drop_player(&mut self, p: usize) {
        if p < MAX_FIGHTERS && self.connected[p] && p != usize::from(self.cfg.local) {
            self.connected[p] = false;
            self.events.push(Event::Disconnected { player: p as u8 });
        }
    }

    // ---- Advancing ---------------------------------------------------------------------------------------------

    /// Runs one frame with `local` as the local player's input (applied `input_delay` frames from now).
    pub fn advance(&mut self, content: &Content, local: Input) -> Advance {
        self.advances = self.advances.wrapping_add(1);
        for p in 0..MAX_FIGHTERS {
            if p != usize::from(self.cfg.local) && self.cfg.is_active(p) && self.connected[p] {
                self.idle[p] += 1;
                if self.idle[p] > DISCONNECT_TIMEOUT {
                    self.drop_player(p);
                }
            }
        }
        self.apply_rollback(content);
        self.update_checksums();

        let too_far = self.frame
            >= self
                .confirmed_frame()
                .saturating_add(u32::from(self.cfg.max_prediction));
        if too_far {
            self.stats.stalls += 1;
            self.queue_packets();
            return Advance::Stalled;
        }

        let lp = usize::from(self.cfg.local);
        let input_frame = self.local_next;
        self.slot_mut(input_frame).known[lp] = Some(local);
        self.local_next += 1;

        let inputs = self.inputs_for(self.frame);
        self.slot_mut(self.frame).used = inputs;
        step(&mut self.state, content, &inputs);
        self.frame += 1;
        self.snaps[self.frame as usize % RING] = (self.frame, self.state);

        self.update_checksums();
        self.queue_packets();
        Advance::Ran
    }

    /// Restores the oldest frame whose prediction turned out wrong and re-simulates up to the present.
    fn apply_rollback(&mut self, content: &Content) {
        let Some(from) = self.pending_rollback.take() else {
            return;
        };
        if from >= self.frame {
            return;
        }
        let (tag, snap) = self.snaps[from as usize % RING];
        if tag != from {
            return; // too old to restore; cannot happen with a sane prediction window
        }
        let depth = self.frame - from;
        self.stats.rollbacks += 1;
        self.stats.resimulated_frames += depth;
        self.stats.longest_rollback = self.stats.longest_rollback.max(depth);
        let mut state = snap;
        for f in from..self.frame {
            self.snaps[f as usize % RING] = (f, state);
            let inputs = self.inputs_for(f);
            self.slot_mut(f).used = inputs;
            step(&mut state, content, &inputs);
        }
        self.snaps[self.frame as usize % RING] = (self.frame, state);
        self.state = state;
    }

    /// The inputs to simulate `frame` with: the real ones where known, a prediction (the player's last known input)
    /// where not, and neutral for anyone not in the match or disconnected.
    fn inputs_for(&self, frame: u32) -> [Input; MAX_FIGHTERS] {
        let mut out = [Input::default(); MAX_FIGHTERS];
        for (p, slot) in out.iter_mut().enumerate() {
            if !self.cfg.is_active(p) || !self.connected[p] {
                continue;
            }
            *slot = self
                .known(frame, p)
                .unwrap_or_else(|| self.predict(frame, p));
        }
        out
    }

    fn predict(&self, frame: u32, p: usize) -> Input {
        let lowest = frame.saturating_sub(RING as u32 - 1);
        (lowest..frame)
            .rev()
            .find_map(|f| self.known(f, p))
            .unwrap_or_default()
    }

    // ---- Checksums ---------------------------------------------------------------------------------------------

    fn update_checksums(&mut self) {
        if self.pending_rollback.is_some() {
            return;
        }
        let interval = self.cfg.checksum_interval.max(1);
        let confirmed = self.confirmed_frame().min(self.frame);
        while self.next_check <= confirmed {
            let frame = self.next_check;
            let (tag, snap) = self.snaps[frame as usize % RING];
            if tag == frame {
                self.local_checksums.insert(frame, snap.checksum());
                self.compare(frame);
            }
            self.next_check += interval;
        }
    }

    // ---- Packets out -------------------------------------------------------------------------------------------

    fn queue_packets(&mut self) {
        let lp = usize::from(self.cfg.local);
        for r in 0..MAX_FIGHTERS {
            if r == lp || !self.cfg.is_active(r) || !self.connected[r] {
                continue;
            }
            // Everything the remote has not acknowledged, up to what fits in one packet.
            let first =
                self.acked[r].max(self.local_next.saturating_sub(MAX_INPUTS_PER_PACKET as u32));
            let inputs: Vec<Input> = (first..self.local_next)
                .map(|f| self.known(f, lp).unwrap_or_default())
                .collect();
            self.outbox.push((
                r as u8,
                Packet::Inputs {
                    player: self.cfg.local,
                    ack: self.received_upto[r],
                    start: first,
                    inputs,
                }
                .encode(),
            ));
            if self.advances % CHECKSUM_RESEND == 0 {
                if let Some((&frame, &value)) = self.local_checksums.iter().next_back() {
                    self.outbox
                        .push((r as u8, Packet::Checksum { frame, value }.encode()));
                }
            }
        }
    }

    /// Tells the other peers this session is leaving.
    pub fn disconnect(&mut self) {
        self.outbox.push((BROADCAST, Packet::Disconnect.encode()));
    }

    // ---- Slots -------------------------------------------------------------------------------------------------

    fn slot_mut(&mut self, frame: u32) -> &mut Slot {
        let slot = &mut self.slots[frame as usize % RING];
        if slot.frame != frame {
            *slot = Slot::fresh(frame);
        }
        slot
    }

    fn known(&self, frame: u32, p: usize) -> Option<Input> {
        let slot = &self.slots[frame as usize % RING];
        if slot.frame == frame {
            slot.known[p]
        } else {
            None
        }
    }
}
