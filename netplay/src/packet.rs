//! The wire format. Plain bytes written by hand (no serialization library): little-endian integers, a magic
//! number and a type byte up front. Decoding never panics on garbage; it returns `None`.
//!
//! Cosmetic data ("loadouts") is carried as opaque bytes in the handshake and never looked at by the sim.

use sim_core::{Input, MatchRules, MAX_FIGHTERS};

pub const MAGIC: u16 = 0x5046; // "PF"
/// Most inputs one packet carries.
pub const MAX_INPUTS_PER_PACKET: usize = 40;
/// Most cosmetic bytes the handshake carries.
pub const MAX_COSMETIC_BYTES: usize = 512;

/// Why a handshake was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RejectReason {
    SimVersion = 1,
    ContentHash = 2,
    Full = 3,
    /// A fighter that is missing, cannot be decoded, does not exist, or is over the point budget under ranked rules.
    BadFighter = 4,
}

impl RejectReason {
    fn from_u8(v: u8) -> Option<RejectReason> {
        match v {
            1 => Some(RejectReason::SimVersion),
            2 => Some(RejectReason::ContentHash),
            3 => Some(RejectReason::Full),
            4 => Some(RejectReason::BadFighter),
            _ => None,
        }
    }
}

/// Everything the host decides for a match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Setup {
    pub seed: u64,
    pub chars: [u8; MAX_FIGHTERS],
    /// Bit `n` set means player `n` takes part.
    pub active: u8,
    pub input_delay: u8,
    /// The host's cosmetics (opaque).
    pub cosmetics: Vec<u8>,
    /// The fighter the host brings (`sim_content::recipe::FighterSpec` bytes), or empty to use `chars` as they are.
    /// Unlike cosmetics this reaches the simulation, so both sides must agree on it.
    pub fighter: Vec<u8>,
    /// Ranked rules: a fighter over the point budget is refused.
    pub ranked: bool,
    /// Stocks and time limit of the match.
    pub rules: MatchRules,
    /// Index into `sim_content::stages` (0 is the base roster's stage).
    pub stage: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    /// Joiner to host: "I run this sim version with this content".
    Hello {
        sim_version: u16,
        content_hash: u64,
        cosmetics: Vec<u8>,
        /// The fighter the joiner brings (spec bytes, or empty).
        fighter: Vec<u8>,
    },
    /// Host to joiner: the match settings. Repeats the host's version and hash so the joiner can check them too.
    Setup {
        sim_version: u16,
        content_hash: u64,
        setup: Setup,
    },
    /// Joiner to host: "I have your setup and am starting".
    Ready,
    /// Either side: the other's version or content differs, so there is no match.
    Reject {
        reason: RejectReason,
        sim_version: u16,
        content_hash: u64,
    },
    /// Inputs of `player` for frames `start..start + inputs.len()`, plus `ack`: the sender has every input of the
    /// receiver's player below this frame.
    Inputs {
        player: u8,
        ack: u32,
        start: u32,
        inputs: Vec<Input>,
    },
    /// The sender's checksum of the state before `frame`, once that frame was confirmed.
    Checksum {
        frame: u32,
        value: u64,
    },
    Disconnect,
}

const T_HELLO: u8 = 1;
const T_SETUP: u8 = 2;
const T_READY: u8 = 3;
const T_REJECT: u8 = 4;
const T_INPUTS: u8 = 5;
const T_CHECKSUM: u8 = 6;
const T_DISCONNECT: u8 = 7;

struct Writer(Vec<u8>);

impl Writer {
    fn new(kind: u8) -> Writer {
        let mut w = Writer(Vec::with_capacity(64));
        w.u16(MAGIC);
        w.u8(kind);
        w
    }
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.u16(b.len().min(MAX_COSMETIC_BYTES) as u16);
        self.0
            .extend_from_slice(&b[..b.len().min(MAX_COSMETIC_BYTES)]);
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        let s = self.data.get(self.at..end)?;
        self.at = end;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|b| u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Option<u64> {
        self.take(8)
            .map(|b| u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = usize::from(self.u16()?);
        if n > MAX_COSMETIC_BYTES {
            return None;
        }
        self.take(n).map(<[u8]>::to_vec)
    }
    fn finished(&self) -> bool {
        self.at == self.data.len()
    }
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Packet::Hello {
                sim_version,
                content_hash,
                cosmetics,
                fighter,
            } => {
                let mut w = Writer::new(T_HELLO);
                w.u16(*sim_version);
                w.u64(*content_hash);
                w.bytes(cosmetics);
                w.bytes(fighter);
                w.0
            }
            Packet::Setup {
                sim_version,
                content_hash,
                setup,
            } => {
                let mut w = Writer::new(T_SETUP);
                w.u16(*sim_version);
                w.u64(*content_hash);
                w.u64(setup.seed);
                for c in setup.chars {
                    w.u8(c);
                }
                w.u8(setup.active);
                w.u8(setup.input_delay);
                w.bytes(&setup.cosmetics);
                w.bytes(&setup.fighter);
                w.u8(u8::from(setup.ranked));
                w.u8(setup.rules.stocks);
                w.u16(setup.rules.time_limit);
                w.u8(setup.stage);
                w.0
            }
            Packet::Ready => Writer::new(T_READY).0,
            Packet::Reject {
                reason,
                sim_version,
                content_hash,
            } => {
                let mut w = Writer::new(T_REJECT);
                w.u8(*reason as u8);
                w.u16(*sim_version);
                w.u64(*content_hash);
                w.0
            }
            Packet::Inputs {
                player,
                ack,
                start,
                inputs,
            } => {
                let mut w = Writer::new(T_INPUTS);
                w.u8(*player);
                w.u32(*ack);
                w.u32(*start);
                let n = inputs.len().min(MAX_INPUTS_PER_PACKET);
                w.u8(n as u8);
                for i in &inputs[..n] {
                    w.0.extend_from_slice(&i.to_bytes());
                }
                w.0
            }
            Packet::Checksum { frame, value } => {
                let mut w = Writer::new(T_CHECKSUM);
                w.u32(*frame);
                w.u64(*value);
                w.0
            }
            Packet::Disconnect => Writer::new(T_DISCONNECT).0,
        }
    }

    /// Parses a datagram. Anything malformed, truncated, oversized or with trailing bytes is `None`.
    pub fn decode(data: &[u8]) -> Option<Packet> {
        let mut r = Reader { data, at: 0 };
        if r.u16()? != MAGIC {
            return None;
        }
        let packet = match r.u8()? {
            T_HELLO => Packet::Hello {
                sim_version: r.u16()?,
                content_hash: r.u64()?,
                cosmetics: r.bytes()?,
                fighter: r.bytes()?,
            },
            T_SETUP => {
                let sim_version = r.u16()?;
                let content_hash = r.u64()?;
                let seed = r.u64()?;
                let mut chars = [0u8; MAX_FIGHTERS];
                for c in &mut chars {
                    *c = r.u8()?;
                }
                let active = r.u8()?;
                let input_delay = r.u8()?;
                let cosmetics = r.bytes()?;
                let fighter = r.bytes()?;
                let ranked = match r.u8()? {
                    0 => false,
                    1 => true,
                    _ => return None,
                };
                let rules = MatchRules {
                    stocks: r.u8()?,
                    time_limit: r.u16()?,
                };
                if rules != rules.clamped() {
                    return None;
                }
                let stage = r.u8()?;
                Packet::Setup {
                    sim_version,
                    content_hash,
                    setup: Setup {
                        seed,
                        chars,
                        active,
                        input_delay,
                        cosmetics,
                        fighter,
                        ranked,
                        rules,
                        stage,
                    },
                }
            }
            T_READY => Packet::Ready,
            T_REJECT => Packet::Reject {
                reason: RejectReason::from_u8(r.u8()?)?,
                sim_version: r.u16()?,
                content_hash: r.u64()?,
            },
            T_INPUTS => {
                let player = r.u8()?;
                let ack = r.u32()?;
                let start = r.u32()?;
                let n = usize::from(r.u8()?);
                if n > MAX_INPUTS_PER_PACKET {
                    return None;
                }
                let mut inputs = Vec::with_capacity(n);
                for _ in 0..n {
                    let b = r.take(Input::BYTES)?;
                    inputs.push(Input::from_bytes([b[0], b[1], b[2], b[3]]));
                }
                Packet::Inputs {
                    player,
                    ack,
                    start,
                    inputs,
                }
            }
            T_CHECKSUM => Packet::Checksum {
                frame: r.u32()?,
                value: r.u64()?,
            },
            T_DISCONNECT => Packet::Disconnect,
            _ => return None,
        };
        r.finished().then_some(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::Rng;

    fn samples() -> Vec<Packet> {
        vec![
            Packet::Hello {
                sim_version: 17,
                content_hash: 0xdead_beef_1234,
                cosmetics: vec![1, 2, 3],
                fighter: vec![1, 0, 9, 5, 5, 1],
            },
            Packet::Setup {
                sim_version: 17,
                content_hash: 99,
                setup: Setup {
                    seed: 42,
                    chars: [0, 1, 0, 1],
                    active: 0b0011,
                    input_delay: 2,
                    cosmetics: vec![],
                    fighter: vec![0, 1],
                    ranked: true,
                    rules: MatchRules {
                        stocks: 4,
                        time_limit: 480,
                    },
                    stage: 2,
                },
            },
            Packet::Ready,
            Packet::Reject {
                reason: RejectReason::ContentHash,
                sim_version: 17,
                content_hash: 5,
            },
            Packet::Inputs {
                player: 1,
                ack: 77,
                start: 70,
                inputs: vec![
                    Input {
                        stick_x: -127,
                        stick_y: 5,
                        buttons: 0b101,
                    },
                    Input::default(),
                ],
            },
            Packet::Checksum {
                frame: 300,
                value: u64::MAX,
            },
            Packet::Disconnect,
        ]
    }

    #[test]
    fn every_packet_round_trips() {
        for p in samples() {
            assert_eq!(Packet::decode(&p.encode()), Some(p));
        }
    }

    #[test]
    fn truncated_or_extended_packets_are_rejected() {
        for p in samples() {
            let bytes = p.encode();
            for cut in 0..bytes.len() {
                assert_eq!(Packet::decode(&bytes[..cut]), None, "cut at {cut}");
            }
            let mut longer = bytes.clone();
            longer.push(0);
            assert_eq!(Packet::decode(&longer), None);
        }
    }

    #[test]
    fn garbage_never_panics() {
        let mut rng = Rng::new(7);
        for _ in 0..20_000 {
            let len = rng.range(80) as usize;
            let mut bytes: Vec<u8> = (0..len).map(|_| rng.range(256) as u8).collect();
            // Half of them start like a real packet so the parser goes deeper.
            if rng.range(2) == 0 && bytes.len() >= 3 {
                bytes[0] = (MAGIC & 0xff) as u8;
                bytes[1] = (MAGIC >> 8) as u8;
                bytes[2] = 1 + rng.range(8) as u8;
            }
            let _ = Packet::decode(&bytes);
        }
    }

    #[test]
    fn too_many_inputs_in_one_packet_is_refused() {
        let mut bytes = Packet::Inputs {
            player: 0,
            ack: 0,
            start: 0,
            inputs: vec![Input::default(); 3],
        }
        .encode();
        // Claim 200 inputs.
        let n_at = 2 + 1 + 1 + 4 + 4;
        bytes[n_at] = 200;
        assert_eq!(Packet::decode(&bytes), None);
    }
}
