//! Network commands for `pftool`.
//!
//! pftool net-fuzz [runs]                      randomised sessions over a simulated lossy network; non-zero exit on any desync
//! pftool net-host <port> [frames] [--fast]    host a headless match over UDP and print the result
//! pftool net-join <host:port> [frames] [--fast]  join one
//! pftool net-relay <port>                      run a relay server for players who cannot connect directly
//! (net-host / net-join also take  --relay <addr> --room <n>  to go through a relay instead)

use netplay::local_rollback::reference_checksums;
use netplay::packet::Setup;
use netplay::peer::{Link, Peer, Status};
use netplay::session::{Advance, Event};
use netplay::testlink::{pair, LinkParams};
use sim_core::fuzz::random_inputs;
use sim_core::{Content, GameState, Input, Rng, MAX_FIGHTERS};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::thread::sleep;
use std::time::Duration;
use transport::{Relay, RelayLink, UdpLink};

fn any_local() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
}

fn resolve(text: &str) -> Result<SocketAddr, String> {
    text.to_socket_addrs()
        .map_err(|e| format!("cannot resolve {text}: {e}"))?
        .next()
        .ok_or_else(|| format!("no address for {text}"))
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// What a finished headless match reports.
pub struct Report {
    pub frames: u32,
    pub stalls: u32,
    pub rollbacks: u32,
    pub longest_rollback: u32,
    pub desyncs: Vec<Event>,
    pub history: Vec<(u32, u64)>,
}

/// Plays a match headless: each update feeds the local player's scripted input for the next frame it will simulate.
fn play<L: Link>(
    mut peer: Peer<L>,
    content: &Content,
    inputs: &[[Input; MAX_FIGHTERS]],
    frames: u32,
    pace: Option<Duration>,
) -> Result<Report, String> {
    let local = peer.local_player();
    let mut ran = 0u32;
    let mut last_progress = std::time::Instant::now();
    let mut desyncs = Vec::new();
    while ran < frames {
        let input = inputs
            .get(ran as usize)
            .map_or_else(Input::default, |i| i[local]);
        match peer.update(content, input) {
            Status::Running(Advance::Ran) => {
                ran += 1;
                last_progress = std::time::Instant::now();
            }
            Status::Rejected {
                reason,
                their_version,
                their_hash,
            } => {
                return Err(format!(
                    "refused ({reason:?}): the other side runs sim version {their_version}, content {their_hash:016x}"
                ));
            }
            _ => {}
        }
        desyncs.extend(
            peer.drain_events()
                .into_iter()
                .filter(|e| matches!(e, Event::Desync { .. })),
        );
        if last_progress.elapsed() > Duration::from_secs(20) {
            return Err("gave up: nothing happened for 20 seconds".to_string());
        }
        sleep(pace.unwrap_or(Duration::from_micros(300)));
    }
    // Keep talking briefly so the other side gets our last inputs and checksums.
    for _ in 0..90 {
        peer.update(content, Input::default());
        sleep(Duration::from_millis(2));
    }
    let stats = peer.stats();
    let history = peer
        .session()
        .map(|s| s.checksum_history())
        .unwrap_or_default();
    peer.leave();
    Ok(Report {
        frames: ran,
        stalls: stats.stalls,
        rollbacks: stats.rollbacks,
        longest_rollback: stats.longest_rollback,
        desyncs,
        history,
    })
}

fn script(seed: u64, frames: usize) -> Vec<[Input; MAX_FIGHTERS]> {
    random_inputs(&mut Rng::new(seed), frames)
}

fn match_setup(seed: u64) -> Setup {
    Setup {
        seed,
        chars: [0, 1, 0, 1],
        active: 0b0011,
        input_delay: 2,
        cosmetics: Vec::new(),
        // Vary the rules too: one stock and a short clock make matches end, so the winner is compared as well.
        rules: sim_core::MatchRules {
            stocks: (seed % 4) as u8,
            time_limit: if seed % 3 == 0 { 5 } else { 0 },
        },
        ..Setup::default()
    }
}

fn print_report(role: &str, r: &Report) {
    let last = r.history.last().map_or_else(
        || "none".to_string(),
        |(f, c)| format!("frame {f} checksum {c:016x}"),
    );
    println!(
        "{role}: {} frames, {} stalls, {} rollbacks (longest {}), {} desyncs, last confirmed checksum: {last}",
        r.frames,
        r.stalls,
        r.rollbacks,
        r.longest_rollback,
        r.desyncs.len()
    );
}

fn paced(args: &[String]) -> Option<Duration> {
    if args.iter().any(|a| a == "--fast") {
        None
    } else {
        Some(Duration::from_micros(16_667))
    }
}

pub fn cmd_host(args: &[String]) -> Result<(), String> {
    let port: u16 = args
        .first()
        .and_then(|p| p.parse().ok())
        .ok_or("usage: pftool net-host <port> [frames]")?;
    let frames: u32 = args.get(1).and_then(|f| f.parse().ok()).unwrap_or(600);
    let content = crate::content()?;
    let setup = match_setup(1);
    let inputs = script(1, frames as usize + 100);
    let report = if let Some(relay) = flag_value(args, "--relay") {
        let room: u64 = flag_value(args, "--room")
            .and_then(|r| r.parse().ok())
            .ok_or("--relay needs --room <number>")?;
        let mut link =
            RelayLink::connect(any_local(), resolve(relay)?, room).map_err(|e| e.to_string())?;
        link.announce();
        println!("hosting through relay {relay}, room {room}");
        play(
            Peer::host(link, &content, setup),
            &content,
            &inputs,
            frames,
            paced(args),
        )?
    } else {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let link = UdpLink::bind(addr, None).map_err(|e| e.to_string())?;
        println!("hosting on UDP port {port}; waiting for a joiner");
        play(
            Peer::host(link, &content, setup),
            &content,
            &inputs,
            frames,
            paced(args),
        )?
    };
    print_report("host", &report);
    if report.desyncs.is_empty() {
        Ok(())
    } else {
        Err("desync detected".to_string())
    }
}

pub fn cmd_join(args: &[String]) -> Result<(), String> {
    let target = args
        .first()
        .ok_or("usage: pftool net-join <host:port> [frames]")?;
    let frames: u32 = args.get(1).and_then(|f| f.parse().ok()).unwrap_or(600);
    let content = crate::content()?;
    let inputs = script(2, frames as usize + 100);
    let report = if let Some(relay) = flag_value(args, "--relay") {
        let room: u64 = flag_value(args, "--room")
            .and_then(|r| r.parse().ok())
            .ok_or("--relay needs --room <number>")?;
        let mut link =
            RelayLink::connect(any_local(), resolve(relay)?, room).map_err(|e| e.to_string())?;
        link.announce();
        play(
            Peer::join(link, &content, Vec::new()),
            &content,
            &inputs,
            frames,
            paced(args),
        )?
    } else {
        let link = UdpLink::bind(any_local(), Some(resolve(target)?)).map_err(|e| e.to_string())?;
        play(
            Peer::join(link, &content, Vec::new()),
            &content,
            &inputs,
            frames,
            paced(args),
        )?
    };
    print_report("joiner", &report);
    if report.desyncs.is_empty() {
        Ok(())
    } else {
        Err("desync detected".to_string())
    }
}

pub fn cmd_relay(args: &[String]) -> Result<(), String> {
    let port: u16 = args
        .first()
        .and_then(|p| p.parse().ok())
        .ok_or("usage: pftool net-relay <port>")?;
    let mut relay = Relay::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port))
        .map_err(|e| e.to_string())?;
    println!("relay listening on UDP port {port}");
    let mut last_report = 0;
    loop {
        relay.pump();
        if relay.forwarded() / 10_000 != last_report {
            last_report = relay.forwarded() / 10_000;
            println!(
                "{} datagrams forwarded across {} rooms",
                relay.forwarded(),
                relay.room_count()
            );
        }
        sleep(Duration::from_millis(1));
    }
}

/// A random fighter for the fuzzer: a built-in one a quarter of the time, otherwise a recipe with random stats.
fn random_spec(rng: &mut Rng) -> sim_content::recipe::FighterSpec {
    use sim_content::recipe::{FighterSpec, Recipe};
    if rng.range(4) == 0 {
        FighterSpec::Builtin(rng.range(3) as u8)
    } else {
        FighterSpec::Made(Recipe {
            class: rng.range(3) as u8,
            size: 1 + rng.range(9) as u8,
            speed: 1 + rng.range(9) as u8,
            jump: 1 + rng.range(9) as u8,
            weight: 1 + rng.range(9) as u8,
        })
    }
}

/// Randomised sessions over a simulated lossy network. Every confirmed frame of both peers must equal a
/// single-machine run of the same inputs, and neither side may report a desync.
pub fn cmd_fuzz(args: &[String]) -> Result<(), String> {
    let runs: u64 = args.first().and_then(|r| r.parse().ok()).unwrap_or(200);
    // `--only <seed>` replays a single run (to investigate a failure).
    let only: Option<u64> = flag_value(args, "--only").and_then(|v| v.parse().ok());
    let content = crate::content()?;
    let frames: u32 = flag_value(args, "--frames")
        .and_then(|v| v.parse().ok())
        .unwrap_or(360);
    let mut rollbacks = 0u64;
    let mut stalls = 0u64;
    let mut bad = 0;
    for seed in 0..runs {
        if only.is_some_and(|o| o != seed) {
            continue;
        }
        let mut rng = Rng::new(seed + 1000);
        let delay = rng.range(5) as u8;
        let params = LinkParams {
            min_latency: rng.range(4),
            max_latency: 4 + rng.range(10),
            loss_percent: rng.range(35),
            duplicate_percent: rng.range(15),
        };
        let mut setup = match_setup(seed);
        setup.input_delay = delay;
        let inputs = script(seed, frames as usize + 80);
        // Every other run, each player brings a random fighter: a built-in one or one made from random stats (casual
        // rules, so any stats are fine). Then the truth is a single-machine run on the content they build together.
        let specs = (seed % 2 == 1).then(|| (random_spec(&mut rng), random_spec(&mut rng)));
        let mut truth_content = content.clone();
        let mut truth_chars = setup.chars;
        if let Some((h, j)) = specs {
            setup.fighter = h.encode();
            let (c, chars) = sim_content::recipe::match_content(&content, &[h, j], false)
                .map_err(|e| format!("specs rejected: {e:?}"))?;
            truth_content = c;
            truth_chars[0] = chars[0];
            truth_chars[1] = chars[1];
        }

        let (la, lb, clock) = pair(params, seed);
        let mut a =
            Peer::host(la, &content, setup.clone()).with_max_prediction(4 + rng.range(7) as u8);
        let join_fighter = specs.map_or_else(Vec::new, |(_, j)| j.encode());
        let mut b = Peer::join_with_fighter(lb, &content, Vec::new(), join_fighter)
            .with_max_prediction(4 + rng.range(7) as u8);
        let (mut na, mut nb) = (0u32, 0u32);
        let mut ticks = 0;
        let spike_at = if std::env::var("PF_NOSPIKE").is_ok() {
            u32::MAX
        } else {
            100 + rng.range(100)
        };
        let recover_at = spike_at.saturating_add(30 + rng.range(60));
        while (na < frames || nb < frames) && ticks < 20_000 {
            if ticks == spike_at {
                clock.set_params(LinkParams {
                    loss_percent: 100,
                    ..params
                });
            }
            if ticks == recover_at {
                clock.set_params(params);
            }
            let ia = inputs
                .get(na as usize)
                .map_or_else(Input::default, |i| i[0]);
            let ib = inputs
                .get(nb as usize)
                .map_or_else(Input::default, |i| i[1]);
            if let Status::Running(Advance::Ran) = a.update(&content, ia) {
                na += 1;
            }
            if let Status::Running(Advance::Ran) = b.update(&content, ib) {
                nb += 1;
            }
            clock.advance();
            ticks += 1;
        }
        // The single-machine truth for the same inputs and delay.
        let initial = GameState::new_with_rules(
            &truth_content,
            setup.seed,
            truth_chars,
            setup.active,
            setup.rules,
        );
        let truth_inputs: Vec<[Input; MAX_FIGHTERS]> = (0..na.max(nb) + 5)
            .map(|f| {
                let mut i = [Input::default(); MAX_FIGHTERS];
                if f >= u32::from(delay) {
                    let src = inputs.get((f - u32::from(delay)) as usize);
                    if let Some(src) = src {
                        i[0] = src[0];
                        i[1] = src[1];
                    }
                }
                i
            })
            .collect();
        let truth = reference_checksums(&truth_content, &initial, &truth_inputs);
        let mut problem = None;
        for (name, peer) in [("host", &mut a), ("joiner", &mut b)] {
            if peer.session().is_none() {
                problem = Some(format!("{name} never started"));
                continue;
            }
            for e in peer.drain_events() {
                println!("{name} event: {e:?}");
                problem = Some(format!("{name} reported {e:?}"));
            }
            let s = peer.session().ok_or("no session")?;
            if s.checksum_history().len() < 5 {
                problem = Some(format!("{name} confirmed too few frames"));
            }
            for (frame, sum) in s.checksum_history() {
                if truth.get(frame as usize) != Some(&sum) {
                    problem = Some(format!(
                        "{name} differs from the single-machine run at frame {frame}"
                    ));
                    break;
                }
            }
            rollbacks += u64::from(peer.stats().rollbacks);
            stalls += u64::from(peer.stats().stalls);
        }
        if only.is_some() {
            let ha: std::collections::BTreeMap<u32, u64> = a
                .session()
                .map(|s| s.checksum_history())
                .unwrap_or_default()
                .into_iter()
                .collect();
            let hb: std::collections::BTreeMap<u32, u64> = b
                .session()
                .map(|s| s.checksum_history())
                .unwrap_or_default()
                .into_iter()
                .collect();
            for (f, ca) in &ha {
                println!(
                    "frame {f}: host {ca:016x} joiner {} truth {:016x}",
                    hb.get(f).map_or("-".to_string(), |c| format!("{c:016x}")),
                    truth.get(*f as usize).copied().unwrap_or(0)
                );
            }
            println!("host ran {na}, joiner ran {nb}, ticks {ticks}, spike at {spike_at}");
            if let Some(sb) = b.session() {
                let missing: Vec<u32> = (sb.frame().saturating_sub(60)..sb.frame() + 3)
                    .filter(|f| sb.known_inputs(*f)[0].is_none())
                    .collect();
                println!("joiner is missing host inputs for frames {missing:?}");
            }
            for (n, peer) in [("host", &a), ("joiner", &b)] {
                if let Some(sess) = peer.session() {
                    println!(
                        "{n}: frame {} confirmed {} stats {:?}",
                        sess.frame(),
                        sess.confirmed_frame(),
                        sess.stats()
                    );
                }
            }
            if let (Some(sa), Some(sb)) = (a.session(), b.session()) {
                for f in sa.frame().saturating_sub(100)..sa.frame().min(sb.frame()) {
                    let (ua, ub) = (sa.used_inputs(f), sb.used_inputs(f));
                    let t = truth_inputs.get(f as usize);
                    if let (Some(ua), Some(ub), Some(t)) = (ua, ub, t) {
                        if ua[..2] != ub[..2] || ua[..2] != t[..2] {
                            println!(
                                "   joiner knows host's input for {f}: {:?}; host knows own: {:?}",
                                sb.known_inputs(f)[0],
                                sa.known_inputs(f)[0]
                            );
                            println!(
                                "frame {f}: host used {:?} joiner used {:?} truth {:?}",
                                &ua[..2],
                                &ub[..2],
                                &t[..2]
                            );
                        }
                    }
                }
            }
        }
        if let Some(p) = problem {
            bad += 1;
            eprintln!(
                "run {seed}: {p} (latency {}-{}, loss {}%, delay {delay})",
                params.min_latency, params.max_latency, params.loss_percent
            );
        }
    }
    println!("{runs} runs, {bad} mismatches ({rollbacks} rollbacks, {stalls} stalls)");
    if bad == 0 {
        Ok(())
    } else {
        Err(format!("{bad} runs desynced"))
    }
}
