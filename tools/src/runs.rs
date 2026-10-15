//! `pftool run-bot [seeds] [frames]`: plays the single-player run (`sim_core::run`) with a simple bot and reports how far each run
//! got, and `pftool run-bench [frames]`: times the run's step with the mob pool full.

use sim_core::input::buttons::{ATTACK, JUMP};
use sim_core::run::{new_run, step_run, Mob, RunContent, MAX_MOBS, MODE_MOVE, PLAYER};
use sim_core::state::NONE;
use sim_core::{collision, Content, Fx, GameState, Input, Vec2, MAX_FIGHTERS};
use std::time::Instant;

/// A plain bot: runs right, jumps at walls, gaps and anything coming at it, and jabs mobs in front of it.
fn bot(state: &GameState, world: &Content, frame: u32) -> Input {
    let f = &state.fighters[PLAYER];
    let mut input = Input {
        stick_x: 127,
        stick_y: 0,
        buttons: 0,
    };
    let stage = &world.stage;
    let ahead = Vec2::new(f.pos.x + Fx::from_int(2), f.pos.y + Fx::from_ratio(1, 2));
    let floor = collision::surface_below(stage, ahead, false);
    let gap = floor.is_none_or(|(_, d)| d > Fx::from_int(2));
    let mut probe = f.pos;
    let p = world.fighters[usize::from(f.char_id).min(world.fighters.len() - 1)];
    let wall = collision::move_x(stage, &p, &mut probe, Fx::from_int(2));
    let near = state.run.mobs.iter().filter(|m| m.active).any(|m| {
        let d = m.pos - f.pos;
        d.x > Fx::ZERO && d.x < Fx::from_ratio(5, 2) && d.y.abs() < Fx::from_int(2)
    });
    // (Presses are edges: every other frame at most.)
    let even = frame % 2 == 0;
    if near && even {
        input.buttons |= ATTACK;
        input.stick_x = 0;
    } else if (gap || wall) && even && (f.platform != NONE || f.air_jumps_left > 0) {
        input.buttons |= JUMP;
    }
    input
}

pub fn cmd_bot(args: &[String]) -> Result<(), String> {
    let seeds: u64 = args
        .first()
        .map_or(Ok(8), |s| s.parse().map_err(|_| "seeds must be a number"))?;
    let frames: u32 = args.get(1).map_or(Ok(7200), |s| {
        s.parse().map_err(|_| "frames must be a number")
    })?;
    let rc = RunContent::placeholder();
    let errors = sim_content::check_run(&rc);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    for seed in 0..seeds {
        let mut world = Content::placeholder();
        let char_id = (seed % world.fighters.len() as u64) as u8;
        let mut state = new_run(&mut world, &rc, seed, char_id);
        let mut n = 0;
        while n < frames && !state.run.over {
            let mut inputs = [Input::default(); MAX_FIGHTERS];
            inputs[PLAYER] = bot(&state, &world, n);
            step_run(&mut state, &mut world, &rc, &inputs);
            n += 1;
        }
        let r = &state.run;
        println!(
            "seed {seed} fighter {}: {} frames ({}s), {} m, {} coins, {} kills, best combo {}, lives left {}, room {}, arenas cleared {}{}",
            state.fighters[PLAYER].char_id,
            n,
            n / 60,
            r.distance(),
            r.coins,
            r.kills,
            r.best_combo,
            state.fighters[PLAYER].stocks,
            r.room,
            r.cleared,
            if r.over { ", over" } else { "" }
        );
    }
    Ok(())
}

pub fn cmd_bench(args: &[String]) -> Result<(), String> {
    let frames: u32 = args.first().map_or(Ok(6000), |s| {
        s.parse().map_err(|_| "frames must be a number")
    })?;
    let rc = RunContent::placeholder();
    let mut world = Content::placeholder();
    let mut state = new_run(&mut world, &rc, 1, 0);
    let mut worst = 0f64;
    let mut total = 0f64;
    for n in 0..frames {
        // Keep the pool full around the player.
        let p = state.fighters[PLAYER].pos;
        for (k, m) in state.run.mobs.iter_mut().enumerate() {
            if !m.active {
                let kind = (k % rc.mobs.len()) as u8;
                *m = Mob {
                    active: true,
                    kind,
                    mode: MODE_MOVE,
                    pos: Vec2::new(p.x + Fx::from_int(k as i32 % 9 - 2), p.y + Fx::from_int(4)),
                    facing: -1,
                    platform: NONE,
                    stamina: rc.mobs[usize::from(kind)].stamina,
                    ..Mob::default()
                };
            }
        }
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[PLAYER] = bot(&state, &world, n);
        let t = Instant::now();
        step_run(&mut state, &mut world, &rc, &inputs);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        worst = worst.max(ms);
        if state.run.over {
            state = new_run(&mut world, &rc, u64::from(n), 0);
        }
    }
    let live = state.run.mobs.iter().filter(|m| m.active).count();
    println!(
        "{frames} frames with up to {MAX_MOBS} mobs ({live} now): {:.4} ms a frame on average, worst {:.4} ms",
        total / f64::from(frames),
        worst
    );
    Ok(())
}
