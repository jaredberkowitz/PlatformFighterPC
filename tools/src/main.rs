//! `pftool`: headless runner, replay generator and rollback fuzzer.
//!
//! pftool gen <out.pfr> <frames> <seed>   write a random replay
//! pftool run <replay.pfr>                replay it and print checksums every 60 frames
//! pftool selftest                        print checksum lines for the cross-platform CI gate
//! pftool fuzz-rollback [runs]            randomised local-rollback runs; non-zero exit on any desync
//! pftool net-fuzz | net-host | net-join | net-relay   networked play, see `net.rs`
//! pftool content-export <out.pfc> | content-check <file.pfc> | content-pack <in.pfc> <out.pfc>   content bundles
//!
//! Every command uses the built-in roster unless the environment variable PF_CONTENT names a bundle file.

mod content;
mod net;
mod replay;

use netplay::local_rollback::{random_delays, verify};
use replay::Replay;
use sim_core::fuzz::random_inputs;
use sim_core::{step, Content, GameState, Rng, SIM_VERSION};
use std::process::ExitCode;

const CHAR_IDS: [u8; 4] = [0, 1, 0, 1];
const CHECKPOINT_EVERY: usize = 60;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("gen") => cmd_gen(&args[1..]),
        Some("run") => cmd_run(&args[1..]),
        Some("selftest") => cmd_selftest(),
        Some("fuzz-rollback") => cmd_fuzz(&args[1..]),
        Some("net-fuzz") => net::cmd_fuzz(&args[1..]),
        Some("net-host") => net::cmd_host(&args[1..]),
        Some("net-join") => net::cmd_join(&args[1..]),
        Some("net-relay") => net::cmd_relay(&args[1..]),
        Some("content-export") => content::cmd_export(&args[1..]),
        Some("content-check") => content::cmd_check(&args[1..]),
        Some("content-pack") => content::cmd_pack(&args[1..]),
        _ => Err(
            "usage: pftool <gen|run|selftest|fuzz-rollback|net-fuzz|net-host|net-join|net-relay|content-export|content-check|content-pack> ..."
                .to_string(),
        ),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The roster commands run against: the built-in one, or the bundle named by `PF_CONTENT`.
fn content() -> Result<Content, String> {
    if let Ok(path) = std::env::var("PF_CONTENT") {
        return Ok(content::load_file(&path)?.content);
    }
    let c = Content::placeholder();
    sim_content::validate(&c).map_err(|e| e.join("; "))?;
    Ok(c)
}

fn make_replay(content: &Content, seed: u64, frames: usize) -> Replay {
    Replay {
        sim_version: SIM_VERSION,
        seed,
        content_hash: content.hash(),
        inputs: random_inputs(&mut Rng::new(seed), frames),
    }
}

/// Replays the inputs, returning `(frame, checksum)` at every checkpoint plus the final checksum.
fn simulate(replay: &Replay, content: &Content) -> Result<(Vec<(usize, u64)>, u64), String> {
    if replay.sim_version != SIM_VERSION {
        return Err(format!(
            "replay is for sim version {}, this build is {SIM_VERSION}",
            replay.sim_version
        ));
    }
    if replay.content_hash != content.hash() {
        return Err("replay was recorded with different content".to_string());
    }
    let mut state = GameState::new(content, replay.seed, CHAR_IDS);
    let mut checkpoints = Vec::new();
    for (i, inputs) in replay.inputs.iter().enumerate() {
        step(&mut state, content, inputs);
        if (i + 1) % CHECKPOINT_EVERY == 0 {
            checkpoints.push((i + 1, state.checksum()));
        }
    }
    Ok((checkpoints, state.checksum()))
}

fn cmd_gen(args: &[String]) -> Result<(), String> {
    let [out, frames, seed] = args else {
        return Err("usage: pftool gen <out.pfr> <frames> <seed>".to_string());
    };
    let frames: usize = frames.parse().map_err(|_| "frames must be a number")?;
    let seed: u64 = seed.parse().map_err(|_| "seed must be a number")?;
    let replay = make_replay(&content()?, seed, frames);
    std::fs::write(out, replay.encode()).map_err(|e| e.to_string())?;
    println!("wrote {frames} frames to {out}");
    Ok(())
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    let [path] = args else {
        return Err("usage: pftool run <replay.pfr>".to_string());
    };
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let replay = Replay::decode(&bytes)?;
    let (checkpoints, last) = simulate(&replay, &content()?)?;
    for (frame, sum) in checkpoints {
        println!("frame {frame} checksum {sum:016x}");
    }
    println!("final {last:016x}");
    Ok(())
}

/// Output must be byte-identical on every OS and CPU architecture. CI diffs it.
fn cmd_selftest() -> Result<(), String> {
    let content = content()?;
    println!(
        "sim_version={SIM_VERSION} content_hash={:016x}",
        content.hash()
    );
    for seed in 1..=3u64 {
        let replay = make_replay(&content, seed, 3600);
        let decoded = Replay::decode(&replay.encode())?;
        if decoded != replay {
            return Err("replay encode/decode round trip failed".to_string());
        }
        let (checkpoints, last) = simulate(&decoded, &content)?;
        for (frame, sum) in checkpoints.iter().filter(|(f, _)| f % 600 == 0) {
            println!("seed={seed} frame={frame} checksum={sum:016x}");
        }
        println!("seed={seed} final={last:016x}");
    }
    Ok(())
}

fn cmd_fuzz(args: &[String]) -> Result<(), String> {
    let runs: u64 = match args.first() {
        Some(s) => s.parse().map_err(|_| "runs must be a number")?,
        None => 1000,
    };
    let content = content()?;
    let mut rollbacks = 0u64;
    let mut resimulated = 0u64;
    let mut longest = 0u32;
    for seed in 0..runs {
        let initial = GameState::new(&content, seed, CHAR_IDS);
        let inputs = random_inputs(&mut Rng::new(seed), 600);
        let max_delay = 1 + (seed % 10) as u32;
        let delays = random_delays(&mut Rng::new(seed ^ 0x5eed), inputs.len(), max_delay);
        let stats = verify(&content, &initial, &inputs, &delays)
            .map_err(|e| format!("seed {seed}: {e}"))?;
        rollbacks += u64::from(stats.rollbacks);
        resimulated += u64::from(stats.resimulated_frames);
        longest = longest.max(stats.longest_rollback);
    }
    println!(
        "{runs} runs, 0 mismatches ({rollbacks} rollbacks, {resimulated} frames resimulated, longest {longest})"
    );
    Ok(())
}
