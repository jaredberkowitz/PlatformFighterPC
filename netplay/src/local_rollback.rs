//! Local rollback harness.
//!
//! Simulates a match where every player's input for frame `f` only *arrives* `delays[f][p]` ticks
//! late. Missing inputs are predicted (repeat the player's last known input); when the real input
//! arrives and differs from what was used, the harness restores the snapshot from that frame and
//! re-simulates forward. Once every input has arrived, the result must be bit-identical to a
//! straight run with perfect inputs, for every frame, not just the last one.

use sim_core::{step, Content, GameState, Input, Rng, MAX_FIGHTERS};

type FrameInputs = [Input; MAX_FIGHTERS];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub rollbacks: u32,
    pub resimulated_frames: u32,
    pub longest_rollback: u32,
}

/// Checksum of the state *before* each frame, plus one final entry after the last frame.
pub fn reference_checksums(
    content: &Content,
    initial: &GameState,
    inputs: &[FrameInputs],
) -> Vec<u64> {
    let mut state = *initial;
    let mut out = Vec::with_capacity(inputs.len() + 1);
    for frame_inputs in inputs {
        out.push(state.checksum());
        step(&mut state, content, frame_inputs);
    }
    out.push(state.checksum());
    out
}

/// Random per-input arrival delays in `0..=max_delay`. Player 0 is "local" and always arrives on time.
pub fn random_delays(rng: &mut Rng, frames: usize, max_delay: u32) -> Vec<[u32; MAX_FIGHTERS]> {
    (0..frames)
        .map(|_| {
            let mut d = [0u32; MAX_FIGHTERS];
            for slot in d.iter_mut().skip(1) {
                *slot = rng.range(max_delay + 1);
            }
            d
        })
        .collect()
}

/// Inputs to use for frame `f`: the real one if it has arrived, otherwise the last known input
/// from an earlier frame for that player.
fn pick_inputs(known: &[[Option<Input>; MAX_FIGHTERS]], f: usize) -> FrameInputs {
    let mut out = [Input::default(); MAX_FIGHTERS];
    for (p, slot) in out.iter_mut().enumerate() {
        *slot = known[f][p]
            .or_else(|| (0..f).rev().find_map(|earlier| known[earlier][p]))
            .unwrap_or_default();
    }
    out
}

/// Runs the rollback simulation and returns its per-frame checksums (same layout as
/// [`reference_checksums`]) along with rollback statistics.
#[allow(clippy::needless_range_loop)]
pub fn run_rollback(
    content: &Content,
    initial: &GameState,
    inputs: &[FrameInputs],
    delays: &[[u32; MAX_FIGHTERS]],
) -> (Vec<u64>, Stats) {
    assert_eq!(inputs.len(), delays.len());
    let n = inputs.len();
    let max_delay = delays
        .iter()
        .flat_map(|d| d.iter())
        .copied()
        .max()
        .unwrap_or(0) as usize;

    let mut known: Vec<[Option<Input>; MAX_FIGHTERS]> = vec![[None; MAX_FIGHTERS]; n];
    let mut used: Vec<FrameInputs> = vec![[Input::default(); MAX_FIGHTERS]; n];
    // snaps[f] is the state before frame f is simulated.
    let mut snaps: Vec<GameState> = Vec::with_capacity(n);
    let mut state = *initial;
    let mut stats = Stats::default();

    let total_ticks = n + max_delay + 1;
    let mut arrivals: Vec<Vec<(usize, usize)>> = vec![Vec::new(); total_ticks];
    for (f, frame_delays) in delays.iter().enumerate() {
        for (p, &delay) in frame_delays.iter().enumerate() {
            arrivals[f + delay as usize].push((f, p));
        }
    }

    for tick_arrivals in arrivals {
        let simulated = snaps.len();

        let mut earliest_mismatch: Option<usize> = None;
        for (f, p) in tick_arrivals {
            let real = inputs[f][p];
            known[f][p] = Some(real);
            if f < simulated && used[f][p] != real {
                earliest_mismatch = Some(earliest_mismatch.map_or(f, |e| e.min(f)));
            }
        }

        if let Some(from) = earliest_mismatch {
            stats.rollbacks += 1;
            let depth = (simulated - from) as u32;
            stats.resimulated_frames += depth;
            stats.longest_rollback = stats.longest_rollback.max(depth);

            state = snaps[from];
            for f in from..simulated {
                snaps[f] = state;
                used[f] = pick_inputs(&known, f);
                step(&mut state, content, &used[f]);
            }
        }

        if simulated < n {
            snaps.push(state);
            used[simulated] = pick_inputs(&known, simulated);
            step(&mut state, content, &used[simulated]);
        }
    }

    let mut checks: Vec<u64> = snaps.iter().map(GameState::checksum).collect();
    checks.push(state.checksum());
    (checks, stats)
}

/// Runs both and reports the first frame whose checksum differs, if any.
pub fn verify(
    content: &Content,
    initial: &GameState,
    inputs: &[FrameInputs],
    delays: &[[u32; MAX_FIGHTERS]],
) -> Result<Stats, String> {
    let expected = reference_checksums(content, initial, inputs);
    let (actual, stats) = run_rollback(content, initial, inputs, delays);
    match expected.iter().zip(&actual).position(|(a, b)| a != b) {
        None if expected.len() == actual.len() => Ok(stats),
        None => Err("checksum lists differ in length".to_string()),
        Some(frame) => Err(format!(
            "desync at frame {frame}: expected {:016x}, got {:016x}",
            expected[frame], actual[frame]
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::fuzz::random_inputs;

    fn setup(seed: u64, frames: usize) -> (Content, GameState, Vec<FrameInputs>) {
        let content = Content::placeholder();
        let initial = GameState::new(&content, seed, [0, 1, 0, 1]);
        let inputs = random_inputs(&mut Rng::new(seed), frames);
        (content, initial, inputs)
    }

    #[test]
    fn zero_delay_never_rolls_back() {
        let (content, initial, inputs) = setup(1, 300);
        let delays = vec![[0u32; MAX_FIGHTERS]; inputs.len()];
        let stats = verify(&content, &initial, &inputs, &delays).unwrap();
        assert_eq!(stats.rollbacks, 0);
    }

    #[test]
    fn late_inputs_cause_rollbacks_but_no_desync() {
        let (content, initial, inputs) = setup(2, 300);
        let delays = random_delays(&mut Rng::new(2), inputs.len(), 7);
        let stats = verify(&content, &initial, &inputs, &delays).unwrap();
        assert!(stats.rollbacks > 0, "test is not exercising rollback");
        assert!(stats.longest_rollback <= 8);
    }

    #[test]
    fn many_randomised_runs_have_zero_mismatches() {
        for seed in 0..200u64 {
            let (content, initial, inputs) = setup(seed, 240);
            let max_delay = 1 + (seed % 10) as u32;
            let delays = random_delays(&mut Rng::new(seed ^ 0xabcd), inputs.len(), max_delay);
            if let Err(e) = verify(&content, &initial, &inputs, &delays) {
                panic!("seed {seed}: {e}");
            }
        }
    }

    #[test]
    fn harness_detects_a_corrupted_run() {
        // Sanity check that verify() can actually fail: feed it inputs that never "arrive" correctly
        // by comparing against a reference built from different inputs.
        let (content, initial, inputs) = setup(3, 120);
        let other = random_inputs(&mut Rng::new(999), 120);
        let expected = reference_checksums(&content, &initial, &other);
        let delays = vec![[0u32; MAX_FIGHTERS]; inputs.len()];
        let (actual, _) = run_rollback(&content, &initial, &inputs, &delays);
        assert_ne!(expected, actual);
    }
}
