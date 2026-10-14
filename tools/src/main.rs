//! `pftool`: headless runner, replay generator and rollback fuzzer.
//!
//! pftool gen <out.pfr> <frames> <seed>   write a random replay
//! pftool run <replay.pfr>                replay it and print checksums every 60 frames
//! pftool replay-verify <match.pfr>      play a replay saved by the game and check it ends where it was recorded
//! pftool balance [--legal] [--quick] ...   play bot matches between fighter builds and report how each does
//! pftool selftest                        print checksum lines for the cross-platform CI gate
//! pftool fuzz-rollback [runs]            randomised local-rollback runs; non-zero exit on any desync
//! pftool net-fuzz | net-host | net-join | net-relay   networked play, see `net.rs`
//! pftool content-export <out.pfc> | content-check <file.pfc> | content-pack <in.pfc> <out.pfc>   content bundles
//!
//! Every command uses the built-in roster unless the environment variable PF_CONTENT names a bundle file.

mod balance;
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
        Some("replay-verify") => cmd_replay_verify(&args[1..]),
        Some("balance") => balance::cmd_balance(&args[1..]),
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
            "usage: pftool <gen|run|replay-verify|balance|selftest|fuzz-rollback|net-fuzz|net-host|net-join|net-relay|content-export|content-check|content-pack> ..."
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

/// Plays a match record (a replay saved by the game, `user://replays/*.pfr`) and checks its winner and final checksum.
fn cmd_replay_verify(args: &[String]) -> Result<(), String> {
    let path = args
        .first()
        .ok_or("usage: pftool replay-verify <match.pfr>")?;
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let record = netplay::replay::MatchRecord::decode(&bytes)?;
    let state = record.verify(&Content::placeholder())?;
    println!(
        "{path}: {} frames, winner {}, final checksum {:016x}: verified",
        record.inputs.len(),
        state.winner,
        state.checksum()
    );
    Ok(())
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

/// A random fighter for the fuzzers: one of the content's built-in fighters a quarter of the time, otherwise a recipe of a
/// random class (every class the content has a weapon for) with random stats.
pub(crate) fn random_spec(rng: &mut Rng, content: &Content) -> sim_content::recipe::FighterSpec {
    use sim_content::recipe::{FighterSpec, Recipe, CLASSES};
    let fighters = content.fighters.len().clamp(1, 256) as u32;
    let classes = content.weapons.len().clamp(1, usize::from(CLASSES)) as u32;
    if rng.range(4) == 0 {
        FighterSpec::Builtin(rng.range(fighters) as u8)
    } else {
        FighterSpec::Made(Recipe {
            class: rng.range(classes) as u8,
            size: 1 + rng.range(9) as u8,
            speed: 1 + rng.range(9) as u8,
            jump: 1 + rng.range(9) as u8,
            weight: 1 + rng.range(9) as u8,
        })
    }
}

/// What items did in a run (E4), for the fuzzers' coverage line: made (pulled, or a lodged arrow), picked up, thrown or dropped, burst.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ItemTally {
    pub made: u64,
    pub picked: u64,
    pub thrown: u64,
    pub bursts: u64,
    /// Sim 115 (E42): things put away in a pocket (projectiles and items), and taken out again.
    pub pocketed: u64,
    pub unpocketed: u64,
    /// Sim 115 (E43): placed pieces made (saplings and trees), and felled trees (a falling tree's shot appearing).
    pub placed: u64,
    pub felled: u64,
}

impl ItemTally {
    pub(crate) fn add(&mut self, o: ItemTally) {
        self.made += o.made;
        self.picked += o.picked;
        self.thrown += o.thrown;
        self.bursts += o.bursts;
        self.pocketed += o.pocketed;
        self.unpocketed += o.unpocketed;
        self.placed += o.placed;
        self.felled += o.felled;
    }

    /// The fuzzers' line about items, pockets and placed pieces.
    pub(crate) fn line(&self) -> String {
        format!(
            "items: {} made, {} picked up, {} thrown or dropped, {} burst; pockets: {} put away, {} taken out; placed: {} made, {} felled",
            self.made,
            self.picked,
            self.thrown,
            self.bursts,
            self.pocketed,
            self.unpocketed,
            self.placed,
            self.felled
        )
    }
}

/// The first body of `content` whose weapon sets down placed pieces (E43: the settler), if any.
pub(crate) fn placed_body(content: &Content) -> Option<u8> {
    content
        .fighters
        .iter()
        .position(|p| {
            content
                .weapons
                .get(usize::from(p.weapon))
                .is_some_and(|w| w.items.iter().any(|it| it.placed.is_some()))
        })
        .map(|i| i as u8)
}

/// Drills for a pocket-and-tree player (sim 115): every 60 frames player `player` presses special, with the stick in the middle every
/// fifth time (the pocket or its release) and down otherwise (plant, water, chop, chop: a tree felled every 300 frames); it stands
/// still otherwise (so it waters and chops the tree it planted), except for a stretch of its random play every 600 frames.
pub(crate) fn drill_specials(
    inputs: &mut [[sim_core::Input; sim_core::MAX_FIGHTERS]],
    player: usize,
) {
    use sim_core::input::buttons::SPECIAL;
    for (i, frame) in inputs.iter_mut().enumerate() {
        if i % 60 == 0 {
            frame[player] = sim_core::Input {
                stick_x: 0,
                stick_y: if (i / 60) % 5 == 0 { 0 } else { -127 },
                buttons: SPECIAL,
            };
        } else if i % 600 < 540 {
            frame[player] = sim_core::Input::default();
        }
    }
}

/// Counts what the items did over a single-machine run from `initial` with `inputs`.
pub(crate) fn tally_items(
    content: &Content,
    initial: &GameState,
    inputs: &[[sim_core::Input; sim_core::MAX_FIGHTERS]],
) -> ItemTally {
    use sim_core::state::{
        ITEM_BLAST, ITEM_FREE, ITEM_HELD, ITEM_LODGED, ITEM_POCKETED, ITEM_THROWN, REST_POCKETED,
    };
    let mut t = ItemTally::default();
    let mut state = *initial;
    for i in inputs {
        let before = state.items;
        let shots = state.projectiles;
        step(&mut state, content, i);
        for (a, b) in shots.iter().zip(state.projectiles.iter()) {
            if b.active && b.rest == REST_POCKETED && !(a.active && a.rest == REST_POCKETED) {
                t.pocketed += 1;
            }
            if a.active && a.rest == REST_POCKETED && b.active && b.rest != REST_POCKETED {
                t.unpocketed += 1;
            }
            if b.active && !a.active && b.kind == sim_core::moves::pockets_slots::FALL_KIND {
                t.felled += 1;
            }
        }
        for (a, b) in before.iter().zip(state.items.iter()) {
            if !b.active {
                continue;
            }
            if b.rest == ITEM_POCKETED && !(a.active && a.rest == ITEM_POCKETED) {
                t.pocketed += 1;
            }
            if a.active && a.rest == ITEM_POCKETED && b.rest != ITEM_POCKETED {
                t.unpocketed += 1;
            }
            if !a.active || a.kind != b.kind {
                t.made += 1;
                if sim_core::items::spec_of(content, b).is_some_and(|s| s.placed.is_some()) {
                    t.placed += 1;
                }
                continue;
            }
            if b.rest == ITEM_HELD && (a.rest == ITEM_FREE || a.rest == ITEM_LODGED) {
                t.picked += 1;
            }
            if a.rest == ITEM_HELD && b.rest == ITEM_THROWN {
                t.thrown += 1;
            }
            if a.rest != ITEM_BLAST && b.rest == ITEM_BLAST {
                t.bursts += 1;
            }
        }
    }
    t
}

/// The first body of `content` whose weapon makes items (the wanderer), if any.
pub(crate) fn item_body(content: &Content) -> Option<u8> {
    content
        .fighters
        .iter()
        .position(|p| {
            content
                .weapons
                .get(usize::from(p.weapon))
                .is_some_and(|w| !w.items.is_empty())
        })
        .map(|i| i as u8)
}

/// The weapon (class) a fighter index of `content` plays, for the fuzzers' coverage line.
pub(crate) fn class_of(content: &Content, char_id: u8) -> usize {
    let i = usize::from(char_id).min(content.fighters.len().saturating_sub(1));
    content.fighters.get(i).map_or(0, |p| usize::from(p.weapon))
}

fn cmd_fuzz(args: &[String]) -> Result<(), String> {
    let runs: u64 = match args.first() {
        Some(s) => s.parse().map_err(|_| "runs must be a number")?,
        None => 1000,
    };
    // `--only <seed>` replays a single run (to investigate a failure).
    let only: Option<u64> = args
        .iter()
        .position(|a| a == "--only")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok());
    let base = content()?;
    let mut rollbacks = 0u64;
    let mut resimulated = 0u64;
    let mut longest = 0u32;
    let mut per_class = vec![0u64; base.weapons.len()];
    let mut items = ItemTally::default();
    for seed in 0..runs {
        if only.is_some_and(|o| o != seed) {
            continue;
        }
        // Four fighters from the whole roster (every class, built-in or made from random stats), drawn from the run's seed.
        let mut pick = Rng::new(seed ^ 0x00c1_a55e);
        let mut specs: Vec<_> = (0..4).map(|_| random_spec(&mut pick, &base)).collect();
        // Every other run one fighter is the first body that makes items (E4: its bombs, its arrows), so items are played with.
        if let (0, Some(b)) = (seed % 2, item_body(&base)) {
            specs[(seed / 2 % 4) as usize] = sim_content::recipe::FighterSpec::Builtin(b);
        }
        // Every third run (sim 115) fighter 1 plays the first body with placed pieces (the settler), drilling its pocket and its tree.
        let drilled = seed % 3 == 1;
        if let (true, Some(b)) = (drilled, placed_body(&base)) {
            specs[1] = sim_content::recipe::FighterSpec::Builtin(b);
        }
        let (content, chars) = sim_content::recipe::match_content(&base, &specs, false)
            .map_err(|e| format!("seed {seed}: specs rejected: {e:?}"))?;
        let chars = [chars[0], chars[1], chars[2], chars[3]];
        for &c in &chars {
            if let Some(n) = per_class.get_mut(class_of(&content, c)) {
                *n += 1;
            }
        }
        let mut initial = GameState::new(&content, seed, chars);
        // Two items lying on the stage from the start (E4), credited to fighter 0, so pick-ups and throws happen in every run.
        if let Some(b) = item_body(&content) {
            let kinds = content.weapons[usize::from(content.fighters[usize::from(b)].weapon)]
                .items
                .len();
            for (k, x) in [(0u8, -2), (1u8, 2)] {
                let spec = k.min(kinds.saturating_sub(1) as u8);
                let at = sim_core::Vec2::new(sim_core::Fx::from_int(x), sim_core::Fx::ZERO);
                if let Some(n) = sim_core::items::make_as(&mut initial, &content, 0, b, spec, at, 0)
                {
                    initial.items[n].platform =
                        sim_core::collision::standing_on(&content.stage, at);
                }
            }
        }
        let mut inputs = random_inputs(&mut Rng::new(seed), 600);
        if drilled {
            drill_specials(&mut inputs, 1);
        }
        items.add(tally_items(&content, &initial, &inputs));
        let max_delay = 1 + (seed % 10) as u32;
        let delays = random_delays(&mut Rng::new(seed ^ 0x5eed), inputs.len(), max_delay);
        let stats = verify(&content, &initial, &inputs, &delays)
            .map_err(|e| format!("seed {seed} (fighters {specs:?}): {e}"))?;
        rollbacks += u64::from(stats.rollbacks);
        resimulated += u64::from(stats.resimulated_frames);
        longest = longest.max(stats.longest_rollback);
    }
    println!(
        "{runs} runs, 0 mismatches ({rollbacks} rollbacks, {resimulated} frames resimulated, longest {longest}); fighters per class {per_class:?}; {}",
        items.line()
    );
    Ok(())
}
