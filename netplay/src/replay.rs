//! Match records (replays): everything needed to play a match again, and nothing else. The simulation is deterministic, so the
//! seed, the match setup and every frame's inputs reproduce it exactly. The record also carries the final checksum, so a replay
//! can be *verified*: played back, it must end on that checksum (this is how replays double as regression tests).
//!
//! The record names the base roster by hash and the two fighters by their spec bytes (`sim_content::recipe::FighterSpec`), so
//! made fighters replay too, on any machine with the same base roster and sim version. Cosmetics (names and looks) are carried as
//! opaque bytes for the viewer and never reach the simulation.
//!
//! Layout (little-endian): `"PFMR"` | u16 format | u16 sim_version | u64 base_hash | u64 match_hash | u64 seed | chars[4] |
//! u8 active | u8 stocks | u16 time_limit | u8 stage | u8 spec count, each u8 length + bytes | u8 cosmetic count, each u16 length + bytes |
//! i8 winner | u64 final_checksum | u32 frames | per frame, per active player, 4 input bytes.
//!
//! Pure like the rest of this crate.

use sim_content::recipe::{match_content_on, FighterSpec};
use sim_core::{step, Content, GameState, Input, MatchRules, MAX_FIGHTERS, SIM_VERSION};

const MAGIC: &[u8; 4] = b"PFMR";
const FORMAT: u16 = 2;
/// Longest match a record may hold (an hour of frames).
pub const MAX_FRAMES: usize = 60 * 3600;
const MAX_SPEC_BYTES: usize = 16;
const MAX_COSMETIC_BYTES: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchRecord {
    pub sim_version: u16,
    /// Hash of the base roster the match was played on.
    pub base_hash: u64,
    /// Hash of the content the match actually ran on (the base roster plus any made fighters).
    pub match_hash: u64,
    pub seed: u64,
    pub chars: [u8; MAX_FIGHTERS],
    pub active: u8,
    pub rules: MatchRules,
    /// Index into `sim_content::stages`.
    pub stage: u8,
    /// Fighter specs of players 0 and 1, or empty when the match used the base roster's characters as they are.
    pub specs: Vec<Vec<u8>>,
    /// Per player, opaque to the simulation (name and look).
    pub cosmetics: Vec<Vec<u8>>,
    pub inputs: Vec<[Input; MAX_FIGHTERS]>,
    /// [`GameState::winner`] at the end (`sim_core::state::PLAYING` if the record stops before the match ended).
    pub winner: i8,
    /// Checksum of the state after the last frame.
    pub final_checksum: u64,
}

impl MatchRecord {
    /// A record of a match that has just been set up, with no frames yet.
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        base: &Content,
        match_content: &Content,
        seed: u64,
        chars: [u8; MAX_FIGHTERS],
        active: u8,
        rules: MatchRules,
        stage: u8,
        specs: Vec<Vec<u8>>,
    ) -> MatchRecord {
        MatchRecord {
            sim_version: SIM_VERSION,
            base_hash: base.hash(),
            match_hash: match_content.hash(),
            seed,
            chars,
            active,
            rules: rules.clamped(),
            stage,
            specs,
            cosmetics: Vec::new(),
            inputs: Vec::new(),
            winner: sim_core::state::PLAYING,
            final_checksum: 0,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(96 + self.inputs.len() * 8);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT.to_le_bytes());
        out.extend_from_slice(&self.sim_version.to_le_bytes());
        out.extend_from_slice(&self.base_hash.to_le_bytes());
        out.extend_from_slice(&self.match_hash.to_le_bytes());
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.chars);
        out.push(self.active);
        out.push(self.rules.stocks);
        out.extend_from_slice(&self.rules.time_limit.to_le_bytes());
        out.push(self.stage);
        out.push(self.specs.len() as u8);
        for s in &self.specs {
            out.push(s.len() as u8);
            out.extend_from_slice(s);
        }
        out.push(self.cosmetics.len() as u8);
        for c in &self.cosmetics {
            out.extend_from_slice(&(c.len() as u16).to_le_bytes());
            out.extend_from_slice(c);
        }
        out.push(self.winner as u8);
        out.extend_from_slice(&self.final_checksum.to_le_bytes());
        out.extend_from_slice(&(self.inputs.len() as u32).to_le_bytes());
        for frame in &self.inputs {
            for (p, input) in frame.iter().enumerate() {
                if self.active >> p & 1 == 1 {
                    out.extend_from_slice(&input.to_bytes());
                }
            }
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<MatchRecord, String> {
        let mut r = Reader { bytes, at: 0 };
        if r.take(4)? != MAGIC {
            return Err("not a replay file".to_string());
        }
        let format = r.u16()?;
        if format != FORMAT {
            return Err(format!("unsupported replay format {format}"));
        }
        let sim_version = r.u16()?;
        let base_hash = r.u64()?;
        let match_hash = r.u64()?;
        let seed = r.u64()?;
        let mut chars = [0u8; MAX_FIGHTERS];
        chars.copy_from_slice(r.take(MAX_FIGHTERS)?);
        let active = r.u8()?;
        if active == 0 || active > 0b1111 {
            return Err("replay names no players".to_string());
        }
        let rules = MatchRules {
            stocks: r.u8()?,
            time_limit: r.u16()?,
        };
        if rules != rules.clamped() {
            return Err("replay has match rules out of range".to_string());
        }
        let stage = r.u8()?;
        if stage >= sim_content::stages::COUNT {
            return Err("replay names a stage that does not exist".to_string());
        }
        let n_specs = usize::from(r.u8()?);
        if n_specs > MAX_FIGHTERS {
            return Err("replay has too many fighter specs".to_string());
        }
        let mut specs = Vec::with_capacity(n_specs);
        for _ in 0..n_specs {
            let len = usize::from(r.u8()?);
            if len > MAX_SPEC_BYTES {
                return Err("replay has an oversized fighter spec".to_string());
            }
            specs.push(r.take(len)?.to_vec());
        }
        let n_cosmetics = usize::from(r.u8()?);
        if n_cosmetics > MAX_FIGHTERS {
            return Err("replay has too many cosmetic profiles".to_string());
        }
        let mut cosmetics = Vec::with_capacity(n_cosmetics);
        for _ in 0..n_cosmetics {
            let len = usize::from(r.u16()?);
            if len > MAX_COSMETIC_BYTES {
                return Err("replay has an oversized cosmetic profile".to_string());
            }
            cosmetics.push(r.take(len)?.to_vec());
        }
        let winner = r.u8()? as i8;
        let final_checksum = r.u64()?;
        let frames = r.u32()? as usize;
        if frames > MAX_FRAMES {
            return Err("replay is too long".to_string());
        }
        let per_frame = active.count_ones() as usize * Input::BYTES;
        if r.remaining() != frames * per_frame {
            return Err("replay body length does not match its frame count".to_string());
        }
        let mut inputs = Vec::with_capacity(frames);
        for _ in 0..frames {
            let mut frame = [Input::default(); MAX_FIGHTERS];
            for (p, slot) in frame.iter_mut().enumerate() {
                if active >> p & 1 == 1 {
                    let c = r.take(Input::BYTES)?;
                    *slot = Input::from_bytes([c[0], c[1], c[2], c[3]]);
                }
            }
            inputs.push(frame);
        }
        Ok(MatchRecord {
            sim_version,
            base_hash,
            match_hash,
            seed,
            chars,
            active,
            rules,
            stage,
            specs,
            cosmetics,
            inputs,
            winner,
            final_checksum,
        })
    }

    /// The content the match ran on and its first state, rebuilt from the base roster. Refuses a record made by another sim
    /// version, on another base roster, or whose fighters no longer build the same content.
    pub fn rebuild(&self, base: &Content) -> Result<(Content, GameState), String> {
        if self.sim_version != SIM_VERSION {
            return Err(format!(
                "this replay is for sim version {}, this build is {SIM_VERSION}",
                self.sim_version
            ));
        }
        if self.base_hash != base.hash() {
            return Err("this replay was recorded on a different base roster".to_string());
        }
        let mut chars = self.chars;
        let parsed: Option<Vec<FighterSpec>> =
            self.specs.iter().map(|b| FighterSpec::decode(b)).collect();
        let parsed = parsed.ok_or("a fighter in this replay cannot be read")?;
        let (content, built) = match_content_on(base, &parsed, false, self.stage)
            .map_err(|e| format!("this replay is not valid: {e:?}"))?;
        // Fighter numbers go to the slots that took part, in slot order (a free-for-all can have a gap).
        let mut next = 0;
        for (slot, c) in chars.iter_mut().enumerate() {
            if self.active >> slot & 1 == 1 {
                if let Some(b) = built.get(next) {
                    *c = *b;
                }
                next += 1;
            }
        }
        if content.hash() != self.match_hash {
            return Err(
                "this replay's fighters build different content than when it was recorded"
                    .to_string(),
            );
        }
        let state = GameState::new_with_rules(&content, self.seed, chars, self.active, self.rules);
        Ok((content, state))
    }

    /// Plays the record from the start and fixes its ending: frames after the match was decided are dropped, and the winner and
    /// final checksum are set from the replay itself (so a sealed record always verifies).
    pub fn seal(&mut self, base: &Content) -> Result<(), String> {
        let (content, mut state) = self.rebuild(base)?;
        let mut keep = self.inputs.len();
        for (i, frame) in self.inputs.iter().enumerate() {
            step(&mut state, &content, frame);
            if state.winner != sim_core::state::PLAYING {
                keep = i + 1;
                break;
            }
        }
        self.inputs.truncate(keep);
        self.winner = state.winner;
        self.final_checksum = state.checksum();
        Ok(())
    }

    /// Plays the whole record and checks it ends where it did: the winner and the checksum. `Ok` carries the final state.
    pub fn verify(&self, base: &Content) -> Result<GameState, String> {
        let (content, mut state) = self.rebuild(base)?;
        for frame in &self.inputs {
            step(&mut state, &content, frame);
        }
        if state.winner != self.winner {
            return Err(format!(
                "replay ended with winner {}, but it was recorded as {}",
                state.winner, self.winner
            ));
        }
        if state.checksum() != self.final_checksum {
            return Err("replay does not end on its recorded checksum".to_string());
        }
        Ok(state)
    }
}

/// Collects a match's inputs frame by frame, in order, ignoring a frame it already has (a rolled-back session may confirm
/// the same frame twice).
pub struct Recorder {
    pub record: MatchRecord,
}

impl Recorder {
    pub fn new(record: MatchRecord) -> Recorder {
        Recorder { record }
    }

    /// Frames recorded so far.
    pub fn len(&self) -> u32 {
        self.record.inputs.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.record.inputs.is_empty()
    }

    /// Adds `frame`'s inputs if it is the next frame; returns whether it was taken.
    pub fn push(&mut self, frame: u32, inputs: [Input; MAX_FIGHTERS]) -> bool {
        if frame != self.len() || self.record.inputs.len() >= MAX_FRAMES {
            return false;
        }
        self.record.inputs.push(inputs);
        true
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.bytes.len());
        let end = end.ok_or("the replay file is cut short")?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }
    fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let b = self.take(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(u64::from_le_bytes(a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_content::recipe::{match_content, Recipe};
    use sim_core::fuzz::random_inputs;
    use sim_core::Rng;

    fn recorded(base: &Content, specs: Vec<Vec<u8>>, rules: MatchRules, seed: u64) -> MatchRecord {
        let mut r = MatchRecord::begin(base, base, seed, [0, 1, 0, 1], 0b0011, rules, 0, specs);
        // Built-in specs leave the content unchanged, so the match hash is the base's.
        let (content, state0) = r.rebuild(base).expect("rebuilds");
        let mut state = state0;
        let mut rec = Recorder::new(r.clone());
        for (f, i) in random_inputs(&mut Rng::new(seed), 1500).iter().enumerate() {
            step(&mut state, &content, i);
            assert!(rec.push(f as u32, *i));
        }
        let _ = state;
        r = rec.record.clone();
        r.seal(base).expect("seals");
        r
    }

    #[test]
    fn round_trip_and_verify() {
        let base = Content::placeholder();
        let r = recorded(
            &base,
            vec![vec![0, 0], vec![0, 1]],
            MatchRules {
                stocks: 2,
                time_limit: 30,
            },
            7,
        );
        let decoded = MatchRecord::decode(&r.encode()).unwrap();
        // Inactive players' inputs are not stored, so compare what a replay uses.
        assert_eq!(decoded.final_checksum, r.final_checksum);
        assert_eq!(decoded.inputs.len(), r.inputs.len());
        assert!(decoded.verify(&base).is_ok(), "the decoded record verifies");
        let mut cosmetics = r.clone();
        cosmetics.cosmetics = vec![vec![1, 2, 3], vec![]];
        assert_eq!(
            MatchRecord::decode(&cosmetics.encode()).unwrap().cosmetics,
            cosmetics.cosmetics
        );
    }

    #[test]
    fn a_made_fighter_replays() {
        let base = Content::placeholder();
        let made = FighterSpec::Made(Recipe {
            class: 1,
            size: 8,
            speed: 3,
            jump: 4,
            weight: 6,
        });
        let specs = vec![made.encode(), FighterSpec::Builtin(0).encode()];
        let parsed: Vec<FighterSpec> = specs
            .iter()
            .map(|b| FighterSpec::decode(b).unwrap())
            .collect();
        let (content, chars) = match_content(&base, &parsed, false).unwrap();
        let mut ids = [0, 1, 0, 1];
        ids[0] = chars[0];
        ids[1] = chars[1];
        let rules = MatchRules::default();
        let mut rec = Recorder::new(MatchRecord::begin(
            &base, &content, 11, ids, 0b0011, rules, 0, specs,
        ));
        let mut state = GameState::new_with_rules(&content, 11, ids, 0b0011, rules);
        for (f, i) in random_inputs(&mut Rng::new(11), 900).iter().enumerate() {
            step(&mut state, &content, i);
            rec.push(f as u32, *i);
        }
        let _ = state;
        let mut record = rec.record.clone();
        record.seal(&base).unwrap();
        let back = MatchRecord::decode(&record.encode()).unwrap();
        assert!(back.verify(&base).is_ok());
        // Tampering is caught.
        let mut bad = back.clone();
        for f in 100..400 {
            bad.inputs[f][0].stick_x = 127;
            bad.inputs[f][0].stick_y = 127;
        }
        assert!(bad.verify(&base).is_err(), "changed inputs end elsewhere");
        let mut wrong_version = back.clone();
        wrong_version.sim_version += 1;
        assert!(wrong_version.verify(&base).is_err());
        let mut wrong_base = back;
        wrong_base.base_hash ^= 1;
        assert!(wrong_base.verify(&base).is_err());
    }

    #[test]
    fn sealing_drops_the_frames_after_the_match_was_decided() {
        let base = Content::placeholder();
        let rules = MatchRules {
            stocks: 3,
            time_limit: 3,
        };
        let mut r = MatchRecord::begin(&base, &base, 5, [0, 1, 0, 1], 0b0011, rules, 0, vec![]);
        r.inputs = random_inputs(&mut Rng::new(5), 1500);
        r.seal(&base).unwrap();
        // The clock ends the match on frame 180 (it is decided after that frame's step).
        assert_eq!(r.inputs.len(), 180);
        assert_ne!(r.winner, sim_core::state::PLAYING);
        assert!(r.verify(&base).is_ok());
        // A record that never ends keeps every frame.
        let mut open = MatchRecord::begin(
            &base,
            &base,
            5,
            [0, 1, 0, 1],
            0b0011,
            MatchRules::default(),
            0,
            vec![],
        );
        open.inputs = random_inputs(&mut Rng::new(6), 100);
        open.seal(&base).unwrap();
        assert_eq!(open.inputs.len(), 100);
        assert_eq!(open.winner, sim_core::state::PLAYING);
        assert!(open.verify(&base).is_ok());
    }

    #[test]
    fn a_four_player_match_with_made_fighters_replays() {
        let base = Content::placeholder();
        let made = |class, size| {
            FighterSpec::Made(Recipe {
                class,
                size,
                speed: 5,
                jump: 5,
                weight: 5,
            })
        };
        let specs = vec![
            made(0, 3).encode(),
            FighterSpec::Builtin(1).encode(),
            made(1, 7).encode(),
            FighterSpec::Builtin(0).encode(),
        ];
        let parsed: Vec<FighterSpec> = specs
            .iter()
            .map(|b| FighterSpec::decode(b).unwrap())
            .collect();
        let (content, chars) = match_content(&base, &parsed, false).unwrap();
        let ids = [chars[0], chars[1], chars[2], chars[3]];
        let rules = MatchRules {
            stocks: 1,
            time_limit: 0,
        };
        let mut record = MatchRecord::begin(&base, &content, 21, ids, 0b1111, rules, 0, specs);
        record.inputs = random_inputs(&mut Rng::new(21), 6000);
        record.seal(&base).unwrap();
        // With one stock each and random play somebody falls long before 6000 frames: the match is decided and cut short.
        assert!(record.inputs.len() < 6000 || record.winner == sim_core::state::PLAYING);
        let back = MatchRecord::decode(&record.encode()).unwrap();
        assert_eq!(back.active, 0b1111);
        let end = back.verify(&base).unwrap();
        assert_eq!(end.roster, 0b1111);
        // Three players: only their inputs are stored.
        let mut three = back.clone();
        three.active = 0b0111;
        three.specs.truncate(3);
        three.chars[3] = 0;
        three.match_hash = {
            let (c, _) = match_content(&base, &parsed[..3], false).unwrap();
            c.hash()
        };
        three.seal(&base).unwrap();
        let three_back = MatchRecord::decode(&three.encode()).unwrap();
        assert!(three_back.verify(&base).is_ok());
        assert!(three.encode().len() < back.encode().len());
    }

    #[test]
    fn a_match_on_another_stage_replays_on_that_stage() {
        let base = Content::placeholder();
        let on = |stage| {
            let content = sim_content::stages::with_stage(&base, stage).unwrap();
            let mut r = MatchRecord::begin(
                &base,
                &content,
                8,
                [0, 1, 0, 1],
                0b0011,
                MatchRules::default(),
                stage,
                vec![],
            );
            r.inputs = random_inputs(&mut Rng::new(8), 1200);
            r.seal(&base).unwrap();
            r
        };
        let (a, b) = (on(0), on(2));
        assert_ne!(
            a.final_checksum, b.final_checksum,
            "the stage changes the match"
        );
        assert!(MatchRecord::decode(&b.encode())
            .unwrap()
            .verify(&base)
            .is_ok());
        let mut wrong = b.clone();
        wrong.stage = 1;
        assert!(
            wrong.verify(&base).is_err(),
            "the wrong stage does not verify"
        );
        let mut bad = b.encode();
        // The stage byte sits after the rules; an out-of-range one is refused.
        let at = 4 + 2 + 2 + 8 + 8 + 8 + 4 + 1 + 1 + 2;
        bad[at] = 200;
        assert!(MatchRecord::decode(&bad).is_err());
    }

    #[test]
    fn the_recorder_takes_frames_in_order_only() {
        let base = Content::placeholder();
        let mut rec = Recorder::new(MatchRecord::begin(
            &base,
            &base,
            1,
            [0, 1, 0, 1],
            0b0011,
            MatchRules::default(),
            0,
            vec![],
        ));
        let i = [Input::default(); MAX_FIGHTERS];
        assert!(!rec.push(1, i), "a gap is refused");
        assert!(rec.push(0, i));
        assert!(!rec.push(0, i), "so is a repeat");
        assert!(rec.push(1, i));
        assert_eq!(rec.len(), 2);
    }

    #[test]
    fn garbage_truncation_and_bad_headers_are_refused() {
        let base = Content::placeholder();
        let r = recorded(&base, vec![], MatchRules::default(), 3);
        let bytes = r.encode();
        assert!(MatchRecord::decode(b"nope").is_err());
        assert!(MatchRecord::decode(&bytes[..bytes.len() - 1]).is_err());
        for cut in 0..60 {
            assert!(MatchRecord::decode(&bytes[..cut]).is_err());
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(MatchRecord::decode(&extra).is_err());
        let mut no_players = bytes.clone();
        no_players[4 + 2 + 2 + 8 + 8 + 8 + 4] = 0;
        assert!(MatchRecord::decode(&no_players).is_err());
        // Fuzz: random flips never panic.
        let mut rng = Rng::new(9);
        for _ in 0..500 {
            let mut b = bytes.clone();
            let i = rng.range(60) as usize;
            b[i] ^= 1 << rng.range(8);
            let _ = MatchRecord::decode(&b);
        }
    }
}
