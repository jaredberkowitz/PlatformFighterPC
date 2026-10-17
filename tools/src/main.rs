//! `pftool`: headless runner, replay generator and rollback fuzzer.
//!
//! pftool gen <out.pfr> <frames> <seed>   write a random replay
//! pftool run <replay.pfr>                replay it and print checksums every 60 frames
//! pftool replay-verify <match.pfr>      play a replay saved by the game and check it ends where it was recorded
//! pftool balance [--legal] [--quick] ...   play bot matches between fighter builds and report how each does
//! pftool selftest                        print checksum lines for the cross-platform CI gate
//! pftool fuzz-rollback [runs]            randomised local-rollback runs; non-zero exit on any desync
//! pftool net-fuzz | net-host | net-join | net-relay | net-stun | net-code   networked play, see `net.rs`
//! pftool content-export <out.pfc> | content-check <file.pfc> | content-pack <in.pfc> <out.pfc>   content bundles
//! pftool run-bot [seeds] [frames] | run-bench [frames]   the single-player run: a bot's runs, and the step's cost with 16 mobs
//!
//! Every command uses the built-in roster unless the environment variable PF_CONTENT names a bundle file.

mod balance;
mod content;
mod net;
mod replay;
mod runs;

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
        Some("net-stun") => net::cmd_stun(&args[1..]),
        Some("net-code") => net::cmd_code(&args[1..]),
        Some("content-export") => content::cmd_export(&args[1..]),
        Some("content-check") => content::cmd_check(&args[1..]),
        Some("content-pack") => content::cmd_pack(&args[1..]),
        Some("run-bot") => runs::cmd_bot(&args[1..]),
        Some("run-bench") => runs::cmd_bench(&args[1..]),
        _ => Err(
            "usage: pftool <gen|run|replay-verify|balance|selftest|fuzz-rollback|net-fuzz|net-host|net-join|net-relay|net-stun|net-code|content-export|content-check|content-pack|run-bot|run-bench> ..."
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
    /// Sim 119 (E49): latches begun (a holder starting a latch move with a clock), left by an exit (attack or jump), and broken (the
    /// clock or a mash-out, or a hit).
    pub latches: u64,
    pub latch_exits: u64,
    pub latch_breaks: u64,
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
        self.latches += o.latches;
        self.latch_exits += o.latch_exits;
        self.latch_breaks += o.latch_breaks;
    }

    /// The fuzzers' line about items, pockets and placed pieces.
    pub(crate) fn line(&self) -> String {
        format!(
            "items: {} made, {} picked up, {} thrown or dropped, {} burst; pockets: {} put away, {} taken out; placed: {} made, {} felled; latches: {} begun, {} exits, {} broken",
            self.made,
            self.picked,
            self.thrown,
            self.bursts,
            self.pocketed,
            self.unpocketed,
            self.placed,
            self.felled,
            self.latches,
            self.latch_exits,
            self.latch_breaks
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

/// The first body of `content` whose gauge keeps itself through its specials (E53: the phantom's partner), if any.
pub(crate) fn partner_body(content: &Content) -> Option<u8> {
    content
        .fighters
        .iter()
        .position(|p| p.gauge.exists() && p.gauge.keeps)
        .map(|i| i as u8)
}

/// The first body of `content` with a wing (E57: the exile's), if any.
pub(crate) fn wing_body(content: &Content) -> Option<u8> {
    content
        .fighters
        .iter()
        .position(|p| p.wing.is_some())
        .map(|i| i as u8)
}

/// The first body of `content` whose weapon has a latch (E49: the scamp's flip), if any.
pub(crate) fn latch_body(content: &Content) -> Option<u8> {
    content
        .fighters
        .iter()
        .position(|p| {
            content
                .weapons
                .get(usize::from(p.weapon))
                .is_some_and(|w| w.moves.iter().any(|m| m.latch.is_some()))
        })
        .map(|i| i as u8)
}

/// Drills for a latching player (sim 119): every 120 frames player `player` flicks the side special toward `toward` (1 right, -1
/// left), and 60 frames later presses attack, jump or nothing in turn (the latch's attack, its jump, or the clock and the mash); it
/// stands still otherwise, except for a stretch of its random play every 1200 frames. Player `target`, if any, stands still for the
/// flip's first 40 frames (to be caught) and mashes with its random play after.
pub(crate) fn drill_latch(
    inputs: &mut [[sim_core::Input; sim_core::MAX_FIGHTERS]],
    player: usize,
    toward: i8,
    target: Option<usize>,
) {
    use sim_core::input::buttons::{ATTACK, JUMP, SPECIAL};
    for (i, frame) in inputs.iter_mut().enumerate() {
        let k = i % 120;
        if k == 0 {
            frame[player] = sim_core::Input {
                stick_x: toward.saturating_mul(127),
                stick_y: 0,
                buttons: SPECIAL,
            };
        } else if k == 60 {
            frame[player] = sim_core::Input {
                stick_x: 0,
                stick_y: 0,
                buttons: [ATTACK, JUMP, 0][(i / 120) % 3],
            };
        } else if i % 1200 < 1080 {
            frame[player] = sim_core::Input::default();
        }
        if let Some(t) = target {
            if k < 40 {
                frame[t] = sim_core::Input::default();
            }
        }
    }
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
    // (The latch, E49, of the move a fighter holds someone in, if any.)
    let latch_of = |f: &sim_core::Fighter| {
        (f.state == sim_core::state::FighterState::Attack && f.grab_with != sim_core::state::NONE)
            .then(|| {
                sim_core::combat::weapon_of(content, sim_core::combat::params_of(content, f))
                    .get(f.move_id)
                    .latch
            })
            .flatten()
    };
    for i in inputs {
        let before = state.items;
        let shots = state.projectiles;
        let holders = state.fighters;
        step(&mut state, content, i);
        for (a, b) in holders.iter().zip(state.fighters.iter()) {
            let (was, now) = (latch_of(a), latch_of(b));
            let clocked = |l: Option<sim_core::moves::Latch>| l.is_some_and(|l| l.clock_num > 0);
            if clocked(now) && was.is_none() {
                t.latches += 1;
            }
            if clocked(was) && now.is_some_and(|l| l.clock_num == 0) {
                t.latch_exits += 1;
            }
            if clocked(was) && now.is_none() {
                t.latch_breaks += 1;
            }
        }
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
        // Every third run from 2 (sim 119) fighter 2 plays the first body with a latch (the scamp), flipping at fighter 3 (who spawns
        // to its right) and leaving its latches by attack, jump or the clock in turn.
        let latching = seed % 3 == 2 && latch_body(&base).is_some();
        if let (true, Some(b)) = (latching, latch_body(&base)) {
            specs[2] = sim_content::recipe::FighterSpec::Builtin(b);
        }
        // Every fourth run from 3 (sim 121) fighter 3 plays the first body with a partner gauge (the phantom), its partner out from the
        // start, so the partner's boxes, its specials, the curse and the guard are played with.
        let partnered = seed % 4 == 3 && partner_body(&base).is_some();
        if let (true, Some(b)) = (partnered, partner_body(&base)) {
            specs[3] = sim_content::recipe::FighterSpec::Builtin(b);
        }
        // Every fourth run from 1 (sim 125) fighter 0 plays the first body with a wing (the exile), its wing out from the start at 70%,
        // its specials drilled, so the wing's numbers, power, armor, points and loss are played with.
        let winged = seed % 4 == 1 && wing_body(&base).is_some();
        if let (true, Some(b)) = (winged, wing_body(&base)) {
            specs[0] = sim_content::recipe::FighterSpec::Builtin(b);
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
        if partnered {
            let f = &mut initial.fighters[3];
            let g = content.fighters[usize::from(f.char_id)].gauge;
            f.gauge = g.max;
            f.gauge_mode = g.mode_frames;
        }
        if winged {
            let f = &mut initial.fighters[0];
            if let Some(w) = content.fighters[usize::from(f.char_id)].wing {
                f.wing_points = w.points[2];
                f.air_jumps_left = f.air_jumps_left.saturating_add(w.jumps);
                f.percent = sim_core::Fx::from_int(70);
            }
        }
        let mut inputs = random_inputs(&mut Rng::new(seed), 600);
        if winged {
            drill_specials(&mut inputs, 0);
        }
        if drilled {
            drill_specials(&mut inputs, 1);
        }
        if latching {
            drill_latch(&mut inputs, 2, 1, Some(3));
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
