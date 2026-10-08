//! `pftool balance`: balance tooling. Plays many matches between scripted bots and reports how every fighter build does.
//!
//! The bot is simple on purpose (run at the other fighter, attack when close, shield and jump now and then, and recover when it falls off
//! the stage), so a result is only meaningful *between builds* of the same kind: when one build wins far more often than its siblings, or a
//! stat is worth far more than the others, that is worth a human look. It does not say what is fun, and a good player will find things the
//! bot does not. The matches are deterministic: the same command prints the same table.
//!
//! ```text
//! pftool balance [--seeds N] [--frames N] [--legal] [--stats 2,5,8] [--quick]
//! ```

use sim_content::recipe::{match_content_on, FighterSpec, Recipe};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use sim_core::state::{FighterState, DRAW, PLAYING};
use sim_core::{step, Content, Fx, GameState, Input, MatchRules, Rng, MAX_FIGHTERS};

/// What one bot does this frame.
pub struct Bot {
    rng: Rng,
    /// Frames the current held action (a shield) still has to run.
    hold: u8,
    hold_kind: u16,
}

impl Bot {
    pub fn new(seed: u64) -> Bot {
        Bot {
            rng: Rng::new(seed),
            hold: 0,
            hold_kind: 0,
        }
    }

    fn stick(dir: i32, amount: i32) -> i8 {
        (dir.signum() * amount).clamp(-127, 127) as i8
    }

    /// The bot's input for fighter `me` against the nearest other fighter still in the match.
    pub fn decide(&mut self, state: &GameState, content: &Content, me: usize) -> Input {
        let f = &state.fighters[me];
        if !f.active {
            return Input::default();
        }
        let opponent = (0..MAX_FIGHTERS)
            .filter(|&i| i != me && state.fighters[i].active)
            .min_by_key(|&i| (state.fighters[i].pos.x - f.pos.x).abs().raw());
        let Some(opponent) = opponent else {
            return Input::default();
        };
        let o = &state.fighters[opponent];
        let ground = &content.stage.platforms[0];
        let centre = (ground.left + ground.right) / Fx::from_int(2);
        let to_centre = (centre - f.pos.x).signum_int();
        let dx = o.pos.x - f.pos.x;
        let dir = dx.signum_int();
        // The brawler's reach is shorter than the sword's, so it must get closer before it swings.
        let reach = if content.fighters[usize::from(f.char_id)].weapon == 1 {
            Fx::from_ratio(5, 2)
        } else {
            Fx::from_int(4)
        };
        let near = dx.abs() < reach && (o.pos.y - f.pos.y).abs() < Fx::from_int(4);
        let airborne = !f.grounded();

        // Falling off the stage: get back. Drift to the middle, use the jumps, then the up special.
        let off = f.pos.x < ground.left - Fx::from_ratio(1, 2)
            || f.pos.x > ground.right + Fx::from_ratio(1, 2)
            || f.pos.y < Fx::from_int(-2);
        if matches!(f.state, FighterState::Hitstun) {
            // Survival DI: bend away from the blast zone.
            return Input {
                stick_x: Self::stick(to_centre, 127),
                stick_y: 60,
                buttons: 0,
            };
        }
        if off && airborne {
            let mut buttons = 0;
            if f.air_jumps_left > 0 && state.frame % 5 == 0 {
                buttons |= JUMP;
            }
            let mut stick_y = 0;
            if f.air_jumps_left == 0 && f.pos.y < Fx::from_int(-3) && state.frame % 3 == 0 {
                buttons |= SPECIAL;
                stick_y = 127;
            }
            return Input {
                stick_x: Self::stick(to_centre, 127),
                stick_y,
                buttons,
            };
        }

        if self.hold > 0 {
            self.hold -= 1;
            return Input {
                stick_x: 0,
                stick_y: 0,
                buttons: self.hold_kind,
            };
        }

        if !near {
            // Close the gap, with a hop now and then.
            let mut buttons = 0;
            if !airborne && self.rng.range(90) == 0 {
                buttons |= JUMP;
            }
            return Input {
                stick_x: Self::stick(dir, 127),
                stick_y: 0,
                buttons,
            };
        }

        // Close to the opponent.
        match self.rng.range(100) {
            0..=2 => {
                self.hold = 12;
                self.hold_kind = SHIELD;
                Input {
                    stick_x: 0,
                    stick_y: 0,
                    buttons: SHIELD,
                }
            }
            3..=9 if !airborne => Input {
                stick_x: Self::stick(dir, 90),
                stick_y: 0,
                buttons: JUMP,
            },
            10..=60 => Input {
                stick_x: Self::stick(dir, if airborne { 100 } else { 60 }),
                stick_y: 0,
                buttons: ATTACK,
            },
            61..=75 => Input {
                stick_x: 0,
                stick_y: 0,
                buttons: ATTACK,
            },
            _ => Input {
                stick_x: Self::stick(dir, 40),
                stick_y: 0,
                buttons: 0,
            },
        }
    }
}

/// How a match went for fighter 0: 1.0 a win, 0.0 a loss, 0.5 a draw.
pub fn play(content: &Content, chars: [u8; MAX_FIGHTERS], seed: u64, frames: u32) -> f64 {
    let rules = MatchRules {
        stocks: 2,
        time_limit: (frames / 60) as u16,
    };
    let mut state = GameState::new_with_rules(content, seed, chars, 0b0011, rules);
    let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
    let mut inputs = [Input::default(); MAX_FIGHTERS];
    for _ in 0..=frames + 60 {
        if state.winner != PLAYING {
            break;
        }
        for (i, bot) in bots.iter_mut().enumerate() {
            inputs[i] = bot.decide(&state, content, i);
        }
        step(&mut state, content, &inputs);
    }
    match state.winner {
        0 => 1.0,
        1 => 0.0,
        w if w == DRAW => 0.5,
        _ => 0.5,
    }
}

/// Fighter `a` against fighter `b`, both sides, over `seeds` seeds: the share of the matches `a` wins (draws count half).
pub fn versus(base: &Content, a: &FighterSpec, b: &FighterSpec, seeds: u64, frames: u32) -> f64 {
    let mut total = 0.0;
    let mut n = 0.0;
    for seed in 0..seeds {
        for swap in [false, true] {
            let specs = if swap { [*b, *a] } else { [*a, *b] };
            let Ok((content, chars)) = match_content_on(base, &specs, false, 0) else {
                continue;
            };
            let ids = [chars[0], chars[1], 0, 1];
            let result = play(&content, ids, seed * 7 + 1, frames);
            total += if swap { 1.0 - result } else { result };
            n += 1.0;
        }
    }
    if n == 0.0 {
        0.5
    } else {
        total / n
    }
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn cmd_balance(args: &[String]) -> Result<(), String> {
    let base = crate::content()?;
    let seeds: u64 = flag_value(args, "--seeds")
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    let frames: u32 = flag_value(args, "--frames")
        .and_then(|v| v.parse().ok())
        .unwrap_or(2400);
    let quick = args.iter().any(|a| a == "--quick");
    let legal_only = args.iter().any(|a| a == "--legal");
    let levels: Vec<u8> = flag_value(args, "--stats")
        .map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_else(|| if quick { vec![3, 7] } else { vec![2, 5, 8] });

    // Every combination of the stat levels, for both classes.
    let mut builds: Vec<Recipe> = Vec::new();
    for class in 0..base.weapons.len().min(2) as u8 {
        for &size in &levels {
            for &speed in &levels {
                for &jump in &levels {
                    for &weight in &levels {
                        let r = Recipe {
                            class,
                            size,
                            speed,
                            jump,
                            weight,
                        };
                        if !legal_only || r.is_legal() {
                            builds.push(r);
                        }
                    }
                }
            }
        }
    }
    // The panel every build is measured against: the two built-in fighters and a spread of the builds themselves.
    let mut panel: Vec<FighterSpec> = vec![FighterSpec::Builtin(0), FighterSpec::Builtin(1)];
    let step_by = (builds.len() / 6).max(1);
    for r in builds.iter().step_by(step_by).take(6) {
        panel.push(FighterSpec::Made(*r));
    }
    println!(
        "balance: {} builds ({}), {} opponents each, {} seeds, {} frames, both sides",
        builds.len(),
        if legal_only { "legal only" } else { "all" },
        panel.len(),
        seeds,
        frames
    );
    let mut scores: Vec<(Recipe, f64)> = Vec::new();
    for r in &builds {
        let spec = FighterSpec::Made(*r);
        let mut sum = 0.0;
        for opp in &panel {
            sum += versus(&base, &spec, opp, seeds, frames);
        }
        scores.push((*r, sum / panel.len() as f64));
    }
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    let show = |label: &str, rows: &[(Recipe, f64)]| {
        println!("\n{label}");
        println!("  win%   class size speed jump weight  points");
        for (r, s) in rows {
            println!(
                "  {:>4.0}   {:>5} {:>4} {:>5} {:>4} {:>6}  {:>6}",
                s * 100.0,
                r.class,
                r.size,
                r.speed,
                r.jump,
                r.weight,
                r.points()
            );
        }
    };
    let n = 8.min(scores.len() / 2);
    show("strongest builds", &scores[..n]);
    show("weakest builds", &scores[scores.len() - n..]);

    // What each stat is worth: the average win rate of the builds with that stat at each level.
    println!("\nwhat each stat is worth (average win%, by level)");
    for (name, pick) in [
        ("size  ", (|r: &Recipe| r.size) as fn(&Recipe) -> u8),
        ("speed ", |r| r.speed),
        ("jump  ", |r| r.jump),
        ("weight", |r| r.weight),
    ] {
        let mut line = format!("  {name}");
        for &level in &levels {
            let rows: Vec<f64> = scores
                .iter()
                .filter(|(r, _)| pick(r) == level)
                .map(|(_, s)| *s)
                .collect();
            if !rows.is_empty() {
                line += &format!(
                    "  {level}: {:>3.0}",
                    100.0 * rows.iter().sum::<f64>() / rows.len() as f64
                );
            }
        }
        println!("{line}");
    }
    for class in 0..2u8 {
        let rows: Vec<f64> = scores
            .iter()
            .filter(|(r, _)| r.class == class)
            .map(|(_, s)| *s)
            .collect();
        if !rows.is_empty() {
            println!(
                "  class {class} average: {:>3.0}",
                100.0 * rows.iter().sum::<f64>() / rows.len() as f64
            );
        }
    }
    let flagged: Vec<&(Recipe, f64)> = scores
        .iter()
        .filter(|(r, s)| (*s > 0.7 || *s < 0.3) && r.is_legal())
        .collect();
    println!(
        "\nlegal builds the bot wins more than 70% or less than 30% of the time: {}",
        flagged.len()
    );
    for (r, s) in flagged.iter().take(10) {
        println!(
            "  {:>3.0}%  class {} size {} speed {} jump {} weight {}",
            s * 100.0,
            r.class,
            r.size,
            r.speed,
            r.jump,
            r.weight
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin(i: u8) -> FighterSpec {
        FighterSpec::Builtin(i)
    }

    #[test]
    fn matches_are_deterministic_and_finish() {
        let base = Content::placeholder();
        let a = versus(&base, &builtin(0), &builtin(1), 2, 1200);
        let b = versus(&base, &builtin(0), &builtin(1), 2, 1200);
        assert_eq!(a, b, "the same matches give the same result");
        assert!((0.0..=1.0).contains(&a));
    }

    #[test]
    fn a_fighter_against_itself_is_an_even_match() {
        let base = Content::placeholder();
        let x = versus(&base, &builtin(0), &builtin(0), 3, 1200);
        assert!(
            (x - 0.5).abs() < 1e-9,
            "both sides play the same fighter: {x}"
        );
    }

    #[test]
    fn the_bots_actually_fight() {
        // Over a few seeds somebody gets hurt: not every match is a quiet draw.
        let base = Content::placeholder();
        let mut damage = Fx::ZERO;
        for seed in 0..4 {
            let rules = MatchRules {
                stocks: 2,
                time_limit: 20,
            };
            let mut state = GameState::new_with_rules(&base, seed, [0, 1, 0, 1], 0b0011, rules);
            let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
            let mut inputs = [Input::default(); MAX_FIGHTERS];
            for _ in 0..1200 {
                for (i, bot) in bots.iter_mut().enumerate() {
                    inputs[i] = bot.decide(&state, &base, i);
                }
                step(&mut state, &base, &inputs);
            }
            damage += state.fighters[0].percent + state.fighters[1].percent;
        }
        assert!(damage > Fx::from_int(20), "the bots land hits");
    }
}
