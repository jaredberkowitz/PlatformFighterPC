//! Combos: a bot plays an aerial loop the way the reference players do (a short-hop aerial played out, land, dash after the target,
//! short hop again) and reports, hit by hit, whether the target was still trapped (in hitstun or hitlag) when the next hit landed: a
//! true combo. The test pins the loop the owner asked for (the staff seer's neutral air three times in a row, from their reference
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
