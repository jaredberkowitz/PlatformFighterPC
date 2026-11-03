//! Frame-by-frame movement traces for every fighter, for the side-by-side comparison with the reference (`docs/MOVEMENT_1TO1.md`).
//! A measuring tool, not a check: it prints one line per fighter and case, `T <class> <case> <values...>`, the values raw fixed
//! point (65536 to a world unit; 1 world unit = 8 reference units), read by the comparison script. Run with
//! `cargo test --release -p sim-core --test movement_trace -- --ignored --nocapture`.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::JUMP;
use sim_core::state::FighterState as S;
use sim_core::{Fx, GameState, Input};

/// Fighter `c` alone on a long flat floor, standing at x -100 facing right.
fn solo(c: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    let st = &mut sim.content.stage;
    st.platforms[0].left = Fx::from_int(-300);
    st.platforms[0].right = Fx::from_int(300);
    st.platforms.truncate(1);
    st.ledges.clear();
    st.blast_left = Fx::from_int(-400);
    st.blast_right = Fx::from_int(400);
    st.blast_bottom = Fx::from_int(-4000);
    sim.state = GameState::new_with_active(&sim.content, 1, [c, 0, 0, 0], 0b0001);
    sim.stand(0, Fx::from_int(-100), 1);
    sim.tick(inp(0, 0, 0));
    sim
}

/// The same, in the air high above the floor with speed `vx`.
fn aloft(c: u8, vx: Fx) -> Sim {
    let mut sim = solo(c);
    sim.put_airborne(0, Fx::from_int(-100), Fx::from_int(2000), vx, Fx::ZERO);
    sim
}

fn line(c: u8, case: &str, values: &[i32]) {
    let v: Vec<String> = values.iter().map(|x| x.to_string()).collect();
    println!("T {c} {case} {}", v.join(" "));
}

/// Runs `inputs` and returns the horizontal speed after each frame.
fn vx_trace(sim: &mut Sim, inputs: &[Input]) -> Vec<i32> {
    inputs
        .iter()
        .map(|&i| {
            sim.tick(i);
            sim.f().vel.x.raw()
        })
        .collect()
}

/// Runs `inputs` and returns the position after each frame.
fn x_trace(sim: &mut Sim, inputs: &[Input]) -> Vec<i32> {
    inputs
        .iter()
        .map(|&i| {
            sim.tick(i);
            sim.f().pos.x.raw()
        })
        .collect()
}

fn hold(i: Input, n: usize) -> Vec<Input> {
    vec![i; n]
}

/// A jump from standing: `held` frames of jump (0: a tap released at once), then `after` held until landing. Returns the height
/// after each airborne frame (feet above the floor), raw.
fn hop(c: u8, held: usize, double_at: Option<usize>) -> Vec<i32> {
    hop_ff(c, held, double_at, false)
}

/// [`hop`], with a fast fall at the first frame it can come out (a flick down as the rise ends) when `fast` is set.
fn hop_ff(c: u8, held: usize, double_at: Option<usize>, fast: bool) -> Vec<i32> {
    let mut sim = solo(c);
    let floor = sim.f().pos.y;
    let mut out = Vec::new();
    let mut airborne = false;
    let mut flicked = false;
    for k in 0..400 {
        let jump = k == 0 || k < held || double_at.is_some_and(|d| k == d);
        let f = sim.f();
        let down = fast && airborne && (flicked || f.vel.y <= Fx::ZERO);
        flicked |= down;
        sim.tick(inp(
            0,
            if down { -127 } else { 0 },
            if jump { JUMP } else { 0 },
        ));
        let f = sim.f();
        if !f.grounded() {
            airborne = true;
            out.push((f.pos.y - floor).raw());
        } else if airborne {
            break;
        }
    }
    out
}

#[test]
#[ignore]
fn traces() {
    let n = Sim::new().content.fighters.len() as u8;
    let names = Sim::new().content.names.fighters.clone();
    for c in 0..n {
        let p = Sim::with_chars([c, 0, 0, 0]).content.fighters[usize::from(c)];
        println!("N {c} {}", names[usize::from(c)]);
        let fields: Vec<String> = p
            .fx_fields()
            .iter()
            .map(|(k, v)| format!("{k}={}", v.raw()))
            .chain(p.int_fields().iter().map(|(k, v)| format!("{k}={v}")))
            .chain([
                format!("jump_speed_x={}", p.jump_speed_x.raw()),
                format!("jump_speed_x_max={}", p.jump_speed_x_max.raw()),
                format!("jump_speed_x_mul={}", p.jump_speed_x_mul),
                format!("air_jump_drift={}", p.air_jump_drift),
            ])
            .collect();
        println!("P {c} {}", fields.join(" "));

        // Air: from rest, the stick held full, then let go; half-held from full speed; reversed from full speed; and speed above the
        // air speed with the stick held forward (a run jump's).
        let mut sim = aloft(c, Fx::ZERO);
        let accel = vx_trace(&mut sim, &hold(inp(127, 0, 0), 60));
        line(c, "air_accel_full", &accel);
        line(
            c,
            "air_release",
            &vx_trace(&mut sim, &hold(inp(0, 0, 0), 120)),
        );
        let mut sim = aloft(c, Fx::ZERO);
        line(
            c,
            "air_accel_half",
            &vx_trace(&mut sim, &hold(inp(64, 0, 0), 60)),
        );
        let mut sim = aloft(c, p.air_speed);
        line(
            c,
            "air_half_from_max",
            &vx_trace(&mut sim, &hold(inp(64, 0, 0), 60)),
        );
        let mut sim = aloft(c, p.air_speed);
        line(
            c,
            "air_reverse",
            &vx_trace(&mut sim, &hold(inp(-127, 0, 0), 60)),
        );
        let over = p.air_speed + Fx::from_ratio(1, 16);
        let mut sim = aloft(c, over);
        line(
            c,
            "air_over_fwd",
            &vx_trace(&mut sim, &hold(inp(127, 0, 0), 60)),
        );
        let mut sim = aloft(c, over);
        line(
            c,
            "air_over_neutral",
            &vx_trace(&mut sim, &hold(inp(0, 0, 0), 60)),
        );

        // Ground: a dash held into a run, then let go; a dash tapped and let go at once; a walk at a full walk and at half.
        let mut sim = solo(c);
        let mut run = vec![inp(0, 0, 0)];
        run.extend(hold(inp(127, 0, 0), 60));
        let v = vx_trace(&mut sim, &run);
        line(c, "dash_hold", &v);
        let x0 = sim.f().pos.x;
        let stop = vx_trace(&mut sim, &hold(inp(0, 0, 0), 60));
        line(c, "run_release", &stop);
        line(c, "run_release_dist", &[(sim.f().pos.x - x0).raw()]);
        let mut sim = solo(c);
        let mut tap = vec![inp(0, 0, 0), inp(127, 0, 0)];
        tap.extend(hold(inp(0, 0, 0), 60));
        let x0 = sim.f().pos.x;
        line(c, "dash_tap", &vx_trace(&mut sim, &tap));
        line(c, "dash_tap_dist", &[(sim.f().pos.x - x0).raw()]);
        let mut sim = solo(c);
        let x0 = sim.f().pos.x;
        let xs = x_trace(&mut sim, &run);
        line(
            c,
            "dash_hold_x",
            &xs.iter().map(|x| x - x0.raw()).collect::<Vec<_>>(),
        );
        for (case, tilt) in [("walk_full", 120i8), ("walk_thr", 76), ("walk_half", 50)] {
            let mut sim = solo(c);
            // A slow push (no flick): the stick creeps up over a few frames.
            let mut walk = vec![inp(20, 0, 0), inp(30, 0, 0)];
            walk.extend(hold(inp(tilt, 0, 0), 60));
            let v = vx_trace(&mut sim, &walk);
            let state = sim.f().state;
            line(c, case, &v);
            let x0 = sim.f().pos.x;
            line(
                c,
                &format!("{case}_release"),
                &vx_trace(&mut sim, &hold(inp(0, 0, 0), 40)),
            );
            line(
                c,
                &format!("{case}_release_dist"),
                &[(sim.f().pos.x - x0).raw(), i32::from(state == S::Walk)],
            );
        }

        // Landing with speed: touching down at the body's ground-jump maximum (a run jump's), stick neutral.
        let land_v = if p.jump_speed_x_max > Fx::ZERO {
            p.jump_speed_x_max
        } else {
            p.run_speed
        };
        let mut sim = solo(c);
        let floor = sim.f().pos.y;
        sim.put_airborne(
            0,
            Fx::from_int(-100),
            floor + Fx::from_ratio(1, 64),
            land_v,
            Fx::ZERO,
        );
        let x0 = sim.f().pos.x;
        let v = vx_trace(&mut sim, &hold(inp(0, 0, 0), 60));
        line(c, "landing_slide", &v);
        line(c, "landing_slide_dist", &[(sim.f().pos.x - x0).raw()]);

        // Jumps: a full hop, a short hop, a full hop with the midair jump at the full hop's peak.
        let full = hop(c, 30, None);
        line(c, "full_hop", &full);
        line(c, "short_hop", &hop(c, 0, None));
        line(c, "full_hop_ff", &hop_ff(c, 30, None, true));
        line(c, "short_hop_ff", &hop_ff(c, 0, None, true));
        let peak = full
            .iter()
            .enumerate()
            .max_by_key(|(_, h)| **h)
            .map_or(0, |(i, _)| i);
        // (Jump let go after 10 frames, so the press at the peak is a new one.)
        line(c, "double_jump", &hop(c, 10, Some(peak + 2)));
        // Falling: from rest high up, stick neutral, then the same with a fast fall at once.
        let mut sim = aloft(c, Fx::ZERO);
        let fall: Vec<i32> = (0..60)
            .map(|_| {
                sim.tick(inp(0, 0, 0));
                sim.f().vel.y.raw()
            })
            .collect();
        line(c, "fall", &fall);
        let mut sim = aloft(c, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        sim.tick(inp(0, 0, 0));
        let mut ff = Vec::new();
        for k in 0..30 {
            sim.tick(inp(0, if k < 10 { -127 } else { 0 }, 0));
            ff.push(sim.f().vel.y.raw());
        }
        line(c, "fast_fall", &ff);
    }
}
