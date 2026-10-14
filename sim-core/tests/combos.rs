//! Combos: a bot plays an aerial loop the way the reference players do (a short-hop aerial played out, land, dash after the target,
//! short hop again) and reports, hit by hit, whether the target was still trapped (in hitstun or hitlag) when the next hit landed: a
//! true combo. `search_carry_chains` (`--ignored --nocapture`, best in `--release`) goes further: after each hit it tries a grid of
//! follow-ups (fast-fall timing, no jump / short hop / full hop / double jump, waiting, aerial timing, drift) and looks a few links
//! ahead, so a chain it reports is one the engine allows, whatever a bot would play. The test pins the loop the owner asked for (the staff seer's neutral air three times in a row, from their reference
//! clip); `probe_*` (run with `--ignored --nocapture`) print tables for checking other classes' chains.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const STAFF: u8 = 5;

fn trapped(sim: &Sim, i: usize) -> bool {
    let f = sim.fighter(i);
    f.hitlag > 0 || f.state == S::Hitstun || f.hitstun > 0
}

/// How the bot plays a loop.
#[derive(Clone, Copy)]
struct Loop {
    /// The attacker's class (the target is class 0).
    cls: u8,
    /// The target's damage at the start.
    percent: i32,
    /// How many short-hop aerials.
    aerials: usize,
    /// How far behind the target the attacker drifts to stay (units).
    behind: Fx,
    /// Fast fall at the apex (the staff only after its neutral air's last hit).
    fast_fall: bool,
    /// The stick (forward, up) that picks the aerial: (0, 0) a neutral air.
    stick: (i8, i8),
    /// Print every hit.
    verbose: bool,
    /// The target starts in the air this high above the ground (zero: standing), as after an opener that launched it.
    target_air: Fx,
    /// The last aerial is a back air: the attacker turns its back to the target first, then back airs toward it.
    back_air_finisher: bool,
}

/// What a loop did: hits, true-combo hits, and the move the attacker hit with last.
struct Played {
    hits: u32,
    combo: u32,
    last_move: u8,
}

/// Plays `l`: each short-hop aerial played out in full, land, dash after the target and short hop again once close.
fn play(l: Loop) -> Played {
    let mut sim = Sim::with_chars([l.cls, 0, 0, 1]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    // (Well left of the middle: three loops carry both fighters a long way right.)
    sim.stand(0, Fx::from_ratio(-92, 10), 1);
    sim.stand(1, Fx::from_int(-8), -1);
    if l.target_air > Fx::ZERO {
        let ground = sim.fighter(1).pos.y;
        sim.put_airborne(
            1,
            Fx::from_int(-8),
            ground + l.target_air,
            Fx::ZERO,
            Fx::ZERO,
        );
    }
    // The aerial in play (its stick, forward and up relative to the facing).
    let mut cur = l.stick;
    let mut last_move = 0u8;
    sim.state.fighters[1].percent = Fx::from_int(l.percent);
    let reach = Fx::from_ratio(16, 10);
    let slack = Fx::from_ratio(1, 10);
    let (mut started, mut hits, mut combo) = (0usize, 0u32, 0u32);
    let mut last_pct = sim.fighter(1).percent;
    let mut was_trapped = false;
    let mut flick_ready = false;
    let mut last_hit_frame = 0;
    for frame in 0..900 {
        let me = *sim.fighter(0);
        let t = *sim.fighter(1);
        let dx = t.pos.x - me.pos.x;
        let toward: i8 = if dx > Fx::ZERO { 127 } else { -127 };
        let free =
            me.grounded() && matches!(me.state, S::Idle | S::Walk | S::Run | S::Dash | S::Turn);
        let mut input = inp(0, 0, 0);
        // The finisher's back air reaches further than the loop's aerial: jump for it from further out.
        let reach = if l.back_air_finisher && started + 1 == l.aerials {
            reach + Fx::from_ratio(8, 10)
        } else {
            reach
        };
        if me.state == S::Landing && started < l.aerials && dx.abs() > reach {
            // A dash flicked during the landing lag comes out on its first free frame (the dash buffer).
            input = if flick_ready {
                inp(toward, 0, 0)
            } else {
                inp(0, 0, 0)
            };
            flick_ready = !flick_ready;
        } else if free && started < l.aerials {
            if dx.abs() > reach && !matches!(me.state, S::Dash | S::Run) {
                // A flick (neutral the frame before) to dash after the target.
                input = if flick_ready {
                    inp(toward, 0, 0)
                } else {
                    inp(0, 0, 0)
                };
                flick_ready = !flick_ready;
            } else if dx.abs() > reach {
                input = inp(toward, 0, 0);
            } else if l.back_air_finisher
                && started + 1 == l.aerials
                && me.facing == (if dx > Fx::ZERO { 1 } else { -1 })
                && matches!(me.state, S::Dash | S::Run)
            {
                // The finishing back air out of a dash: a reverse aerial rush (jump with a flick back: it turns in the jump, no
                // turnaround on the ground), short hop, and the back air toward the target.
                cur = (-100, 0);
                input = inp(-toward, 0, JUMP | ATTACK);
                started += 1;
            } else if l.back_air_finisher
                && started + 1 == l.aerials
                && me.facing == (if dx > Fx::ZERO { 1 } else { -1 })
            {
                // Standing: turn the back to the target (a tilt the other way) before the finishing back air.
                input = inp(-toward / 2, 0, 0);
            } else {
                cur = if l.back_air_finisher && started + 1 == l.aerials {
                    (-100, 0)
                } else {
                    l.stick
                };
                input = inp(cur.0 * me.facing, cur.1, JUMP | ATTACK);
                started += 1;
            }
        } else if me.state == S::JumpSquat
            || (!me.grounded() && me.state == S::Airborne && started > 0)
        {
            // Until the aerial starts, the stick picks it.
            let press = if me.state == S::Airborne { ATTACK } else { 0 };
            input = inp(cur.0 * me.facing, cur.1, press);
        } else if !me.grounded() && me.state == S::Attack {
            let want = dx.abs() - l.behind;
            let drift: i8 = if want > slack {
                toward
            } else if want < -slack {
                -toward
            } else {
                0
            };
            let ff = l.fast_fall && me.vel.y < Fx::ZERO && (l.cls != STAFF || me.state_frame >= 31);
            input = inp(drift, if ff { -127 } else { 0 }, 0);
        }
        sim.tick2(input, inp(0, 0, 0));
        let t = sim.fighter(1);
        if t.percent > last_pct {
            hits += 1;
            last_hit_frame = frame;
            last_move = sim.fighter(0).move_id;
            if was_trapped {
                combo += 1;
            }
            if l.verbose {
                println!(
                    "  f{frame} hit {hits} {:?} trapped before: {was_trapped}, hitstun {}, target at {:?}, attacker {:?} frame {}",
                    t.percent, t.hitstun, t.pos, sim.fighter(0).state, sim.fighter(0).state_frame
                );
            }
            last_pct = t.percent;
        }
        was_trapped = trapped(&sim, 1);
        if started >= l.aerials && sim.fighter(0).grounded() && frame > last_hit_frame + 60 {
            break;
        }
    }
    Played {
        hits,
        combo,
        last_move,
    }
}

fn staff_nair(percent: i32, behind: Fx) -> Loop {
    Loop {
        cls: STAFF,
        percent,
        aerials: 3,
        behind,
        fast_fall: false,
        stick: (0, 0),
        verbose: false,
        target_air: Fx::ZERO,
        back_air_finisher: false,
    }
}

/// The owner's reference clip, its loop: the staff seer's short-hop neutral air three times at 0%, each played out (loop and last hit),
/// landed and dashed after. Every hit after the opener lands while the target is still trapped.
#[test]
fn the_staff_neutral_air_loop_is_a_true_combo_at_low_percent() {
    let Played { hits, combo, .. } = play(staff_nair(0, Fx::from_ratio(3, 10)));
    assert!(
        hits >= 18,
        "all three neutral airs connect, loop and last hit: {hits} hits"
    );
    assert_eq!(
        combo,
        hits - 1,
        "every hit after the opener is a true combo ({combo} of {hits})"
    );
}

#[test]
#[ignore]
fn probe_staff_neutral_air_loop() {
    for pct in [0, 15, 30] {
        for behind in [Fx::from_ratio(3, 10), Fx::from_ratio(8, 10)] {
            println!("staff neutral air loop at {pct}%, {behind:?} behind:");
            let r = play(Loop {
                verbose: true,
                ..staff_nair(pct, behind)
            });
            println!("  => {} hits, {} true combo", r.hits, r.combo);
        }
    }
}

/// Every class's neutral, forward and up air loops, on a standing target and on one starting in the air ("(air)", as after a launching
/// opener: the reference's forward-air and up-air chains start there), the best of a few spacings: "combo/hits" per percent.
#[test]
#[ignore]
fn probe_every_class_aerial_loops() {
    let names = [
        "Longsword",
        "Claws",
        "Needles",
        "Embers",
        "Gauntlets",
        "Staff",
        "Greatsword",
        "Bracers",
    ];
    let aerials = [("nair", (0i8, 0i8)), ("fair", (100, 0)), ("uair", (0, 100))];
    for (cls, name) in names.iter().enumerate() {
        for (aerial, stick, target_air) in aerials
            .iter()
            .flat_map(|&(a, s)| [(a, s, Fx::ZERO), (a, s, Fx::from_int(2))])
        {
            let mut row = format!(
                "{name:<11} {aerial} {}:",
                if target_air > Fx::ZERO {
                    "(air)"
                } else {
                    "     "
                }
            );
            for percent in [0, 20, 40] {
                let mut best = (0, 0);
                for behind in [Fx::ZERO, Fx::from_ratio(4, 10), Fx::from_ratio(9, 10)] {
                    for fast_fall in [false, true] {
                        let l = Loop {
                            cls: cls as u8,
                            percent,
                            aerials: 4,
                            behind,
                            fast_fall,
                            stick,
                            verbose: false,
                            target_air,
                            back_air_finisher: false,
                        };
                        let r = play(l);
                        if (r.combo, r.hits) > (best.1, best.0) {
                            best = (r.hits, r.combo);
                        }
                    }
                }
                row += &format!("  {percent:>2}%: {}/{}", best.1, best.0);
            }
            println!("{row}");
        }
    }
}

/// The owner's reference clip in full: the staff seer's neutral air loop three times at 0%, then a back air to finish, out of a dash
/// buffered during the landing lag and a reverse aerial rush (the jump turns the attacker's back to the target). The back air lands, and
/// every hit after the opener is a true combo.
#[test]
fn the_staff_neutral_air_loop_finishes_with_a_back_air() {
    let r = play(Loop {
        aerials: 4,
        back_air_finisher: true,
        ..staff_nair(0, Fx::from_ratio(3, 10))
    });
    assert_eq!(
        r.last_move,
        sim_core::moves::MoveId::BAir as u8,
        "the finisher is the back air ({} hits)",
        r.hits
    );
    assert_eq!(
        r.combo,
        r.hits - 1,
        "every hit after the opener is a true combo ({} of {})",
        r.combo,
        r.hits
    );
}

// ---- Search: does a true-combo follow-up exist at all? -------------------------------------------------------------------------------
//
// A bot plays one routine; a chain it misses may still exist. Here, after each hit, every follow-up from a small set of plans is tried
// (fast fall timing, no jump / short hop / full hop / double jump, how long to wait first, when to press the aerial, where to drift
// relative to the target) and the earliest one that lands while the target is still trapped is taken. A carry chain (an aerial
// repeated while the target is carried across the stage) is as long as these links go.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hop {
    None,
    Short,
    Full,
    Double,
}

#[derive(Clone, Copy, Debug)]
struct Plan {
    /// Frames after the plan starts at which to fast fall (if falling), or never.
    fast_fall_at: Option<u16>,
    hop: Hop,
    /// Frames to wait once free before hopping (moving toward the target meanwhile).
    wait: u16,
    /// Frames after the hop (or after being free, with no hop) at which to press the aerial.
    aerial_at: u16,
    /// Where to drift: this far behind the target (negative: in front).
    offset: Fx,
}

fn plans() -> Vec<Plan> {
    let mut out = vec![];
    for fast_fall_at in [None, Some(0), Some(4), Some(9)] {
        for hop in [Hop::None, Hop::Short, Hop::Full, Hop::Double] {
            for wait in [0u16, 3, 7] {
                for aerial_at in [0u16, 3, 6, 9, 12, 16, 21] {
                    for offset in [-6, 0, 8] {
                        out.push(Plan {
                            fast_fall_at,
                            hop,
                            wait,
                            aerial_at,
                            offset: Fx::from_ratio(offset, 10),
                        });
                    }
                }
            }
        }
    }
    out
}

fn is_trapped(f: &sim_core::Fighter) -> bool {
    f.hitlag > 0 || f.state == S::Hitstun || f.hitstun > 0
}

/// Plays `plan` from `state` for the attacker (player 0), the aerial picked by `stick` (forward, up). Returns the state on the frame
/// the next hit lands and how many frames that took, or None (no hit, or with `need_trap` a hit on a target that was already free).
fn rollout(
    content: &sim_core::Content,
    mut state: sim_core::GameState,
    plan: Plan,
    stick: (i8, i8),
    need_trap: bool,
) -> Option<(sim_core::GameState, u32)> {
    let mut free_at: Option<u16> = None;
    let mut hopped_at: Option<u16> = None;
    let mut pressed = false;
    let start_pct = state.fighters[1].percent;
    let mut was_trapped = is_trapped(&state.fighters[1]);
    let step = Fx::from_ratio(1, 10);
    for t in 0..90u16 {
        let me = state.fighters[0];
        let tg = state.fighters[1];
        let side = if tg.pos.x >= me.pos.x { 1 } else { -1 };
        let dx = tg.pos.x - plan.offset.mul_int(side) - me.pos.x;
        let drift: i8 = if dx > step {
            127
        } else if dx < -step {
            -127
        } else {
            0
        };
        let free_air = !me.grounded() && me.state == S::Airborne;
        let free_ground =
            me.grounded() && matches!(me.state, S::Idle | S::Walk | S::Run | S::Dash | S::Turn);
        if free_at.is_none() && (free_air || free_ground) {
            free_at = Some(t);
        }
        let fall = match plan.fast_fall_at {
            Some(ff) if t >= ff && !me.grounded() && me.vel.y < Fx::ZERO && !me.fast_fall => -127,
            _ => 0,
        };
        let mut input = inp(drift, fall, 0);
        if let (false, Some(f)) = (pressed, free_at) {
            let hop_needed = plan.hop != Hop::None;
            if hop_needed && hopped_at.is_none() {
                let can = if plan.hop == Hop::Double {
                    free_air
                } else {
                    free_ground
                };
                if t >= f + plan.wait && can {
                    input = inp(drift, 0, JUMP);
                    hopped_at = Some(t);
                }
            } else {
                let from = hopped_at.unwrap_or(f + plan.wait);
                if t >= from + plan.aerial_at && free_air {
                    input = inp(stick.0 * me.facing, stick.1, ATTACK);
                    pressed = true;
                } else if plan.hop == Hop::Full && me.state == S::JumpSquat {
                    input = inp(drift, 0, JUMP);
                }
            }
        }
        let mut inputs = [sim_core::Input::default(); sim_core::MAX_FIGHTERS];
        inputs[0] = input;
        sim_core::step(&mut state, content, &inputs);
        if state.fighters[1].percent > start_pct {
            if need_trap && !was_trapped {
                return None;
            }
            return Some((state, u32::from(t) + 1));
        }
        was_trapped = is_trapped(&state.fighters[1]);
        if need_trap && !was_trapped {
            // The target is free: no follow-up from here can be a true combo.
            return None;
        }
    }
    None
}

/// The longest carry chain the search finds for class `cls` repeating the aerial picked by `stick`, opening on a target at `percent`
/// standing 1.2 units ahead. Returns (true-combo links after the opener, the target's damage at the end, how far it was carried).
fn search_chain(cls: u8, stick: (i8, i8), percent: i32, opener: Option<(i8, i8)>) -> (u32, Fx, Fx) {
    let mut sim = Sim::with_chars([cls, 0, 0, 1]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-8), 1);
    sim.stand(1, Fx::from_ratio(-68, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(percent);
    let start_x = sim.state.fighters[1].pos.x;
    let plans = plans();
    // The openers: a ground move (an up tilt, held a few frames first: the same stick pushed in one frame is a flick, a smash), or
    // short-hop versions of the aerial itself; each is followed as far as it chains and the longest chain counts.
    let mut openers: Vec<sim_core::GameState> = vec![];
    if let Some(tilt) = opener {
        let mut state = sim.state;
        let start = state.fighters[1].percent;
        for t in 0..40u32 {
            let mut inputs = [sim_core::Input::default(); sim_core::MAX_FIGHTERS];
            if t < 5 {
                inputs[0] = inp(tilt.0, tilt.1, 0);
            } else if t == 5 {
                inputs[0] = inp(tilt.0, tilt.1, ATTACK);
            }
            sim_core::step(&mut state, &sim.content, &inputs);
            if state.fighters[1].percent > start {
                openers.push(state);
                break;
            }
        }
    }
    for &p in plans.iter().filter(|p| {
        p.hop == Hop::Short
            && p.wait == 0
            && p.offset == Fx::ZERO
            && matches!(p.fast_fall_at, None | Some(0))
    }) {
        if let Some((s, _)) = rollout(&sim.content, sim.state, p, stick, false) {
            openers.push(s);
        }
    }
    let mut best = (0u32, Fx::ZERO, Fx::ZERO);
    for opened in openers {
        let (links, end) = longest(&sim.content, opened, &plans, stick, LOOK_AHEAD_DEPTH);
        if links > best.0 || best.1 == Fx::ZERO {
            let t = end.fighters[1];
            best = (links, t.percent, (t.pos.x - start_x).abs());
        }
        if best.0 >= LOOK_AHEAD_DEPTH {
            break;
        }
    }
    best
}

/// How many links ahead the search looks, and how many follow-ups it keeps at each link (the earliest, plus later ones spread over
/// the timings that worked, which can leave the target better placed for the next link).
const LOOK_AHEAD_DEPTH: u32 = 6;
const LOOK_AHEAD_WIDTH: usize = 3;

/// The longest run of true-combo links from `state` (at most `depth`), and the state at its end.
fn longest(
    content: &sim_core::Content,
    state: sim_core::GameState,
    plans: &[Plan],
    stick: (i8, i8),
    depth: u32,
) -> (u32, sim_core::GameState) {
    if depth == 0 {
        return (0, state);
    }
    let mut hits: Vec<(u32, sim_core::GameState)> = plans
        .iter()
        .filter_map(|&p| rollout(content, state, p, stick, true).map(|(s, t)| (t, s)))
        .collect();
    if hits.is_empty() {
        return (0, state);
    }
    hits.sort_by_key(|h| h.0);
    hits.dedup_by_key(|h| h.0);
    // The earliest, the latest and one in the middle.
    let n = hits.len();
    let mut picks = vec![0, n - 1, n / 2];
    picks.dedup();
    let mut best = (0, state);
    for &i in picks.iter().take(LOOK_AHEAD_WIDTH) {
        let (more, end) = longest(content, hits[i].1, plans, stick, depth - 1);
        if more + 1 > best.0 {
            best = (more + 1, end);
        }
        if best.0 == depth {
            break;
        }
    }
    best
}

#[test]
#[ignore]
fn search_carry_chains() {
    let names = [
        "Longsword",
        "Claws",
        "Needles",
        "Embers",
        "Gauntlets",
        "Staff",
        "Greatsword",
        "Bracers",
    ];
    let aerials = [
        ("nair", (0i8, 0i8)),
        ("fair", (100, 0)),
        ("uair", (0, 100)),
        ("bair", (-100, 0)),
    ];
    // (Up-air chains open with an up tilt, as in the reference; the rest with the aerial itself.)
    let only: Option<usize> = std::env::var("CLASS").ok().and_then(|s| s.parse().ok());
    for (cls, name) in names.iter().enumerate() {
        if only.is_some_and(|o| o != cls) {
            continue;
        }
        for (aerial, stick) in aerials {
            let mut row = format!("{name:<11} {aerial}:");
            for percent in [0, 20, 40, 60, 80] {
                let opener = if aerial == "uair" {
                    Some((0, 90))
                } else {
                    None
                };
                let (links, _, _) = search_chain(cls as u8, stick, percent, opener);
                row += &format!("  {percent:>2}%: {links}");
            }
            println!("{row}");
        }
    }
}
