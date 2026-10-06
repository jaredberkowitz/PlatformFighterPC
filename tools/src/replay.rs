//! Replay file: a seed plus the exact per-frame inputs, nothing else. The same stream drives
//! regression tests, the cross-platform CI gate and (later) in-game replays.
//!
//! Layout (all little-endian):
//! `"PFRP"` | u16 format | u16 sim_version | u64 seed | u64 content_hash | u32 frames |
//! frames * MAX_FIGHTERS * 4 input bytes

use sim_core::{Input, MAX_FIGHTERS};

const MAGIC: &[u8; 4] = b"PFRP";
const FORMAT: u16 = 1;
const HEADER_LEN: usize = 4 + 2 + 2 + 8 + 8 + 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replay {
    pub sim_version: u16,
    pub seed: u64,
    pub content_hash: u64,
    pub inputs: Vec<[Input; MAX_FIGHTERS]>,
}

impl Replay {
    pub fn encode(&self) -> Vec<u8> {
        let mut out =
            Vec::with_capacity(HEADER_LEN + self.inputs.len() * MAX_FIGHTERS * Input::BYTES);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT.to_le_bytes());
        out.extend_from_slice(&self.sim_version.to_le_bytes());
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.content_hash.to_le_bytes());
        out.extend_from_slice(&(self.inputs.len() as u32).to_le_bytes());
        for frame in &self.inputs {
            for input in frame {
                out.extend_from_slice(&input.to_bytes());
            }
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Replay, String> {
        if bytes.len() < HEADER_LEN || &bytes[..4] != MAGIC {
            return Err("not a replay file".to_string());
        }
        let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
        let u64_at = |o: usize| {
            let mut b = [0u8; 8];
            b.copy_from_slice(&bytes[o..o + 8]);
            u64::from_le_bytes(b)
        };
        let format = u16_at(4);
        if format != FORMAT {
            return Err(format!("unsupported replay format {format}"));
        }
        let frames = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]) as usize;
        let body = &bytes[HEADER_LEN..];
        if body.len() != frames * MAX_FIGHTERS * Input::BYTES {
            return Err("replay body length does not match its frame count".to_string());
        }
        let mut inputs = Vec::with_capacity(frames);
        let mut chunks = body.chunks_exact(Input::BYTES);
        for _ in 0..frames {
            let mut frame = [Input::default(); MAX_FIGHTERS];
            for slot in frame.iter_mut() {
                let c = chunks.next().ok_or("truncated replay")?;
                *slot = Input::from_bytes([c[0], c[1], c[2], c[3]]);
            }
            inputs.push(frame);
        }
        Ok(Replay {
            sim_version: u16_at(6),
            seed: u64_at(8),
            content_hash: u64_at(16),
            inputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::fuzz::random_inputs;
    use sim_core::Rng;

    #[test]
    fn round_trip() {
        let r = Replay {
            sim_version: 1,
            seed: 0xdead_beef,
            content_hash: 0x1234_5678_9abc_def0,
            inputs: random_inputs(&mut Rng::new(1), 100),
        };
        assert_eq!(Replay::decode(&r.encode()).unwrap(), r);
    }

    #[test]
    fn rejects_garbage_and_truncation() {
        assert!(Replay::decode(b"nope").is_err());
        let r = Replay {
            sim_version: 1,
            seed: 1,
            content_hash: 2,
            inputs: random_inputs(&mut Rng::new(1), 10),
        };
        let mut bytes = r.encode();
        bytes.pop();
        assert!(Replay::decode(&bytes).is_err());
    }
}
