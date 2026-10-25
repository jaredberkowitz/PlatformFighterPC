//! How ground momentum carries into a jump, for every fighter, in the reference's units (1 world unit = 8): the ground speed before
//! the jump, the speed on the first airborne frame, the distance from take-off to landing, the frames in the air and the facing.
//! A measuring tool, not a check (`docs/MOMENTUM.md` holds the table and what it is compared against). Run with
//! `cargo test --release -p sim-core --test momentum_table -- --ignored --nocapture`.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::JUMP;
use sim_core::state::FighterState as S;
use sim_core::{Fx, GameState, Input};

/// A world-unit value in the reference's units, as text with two decimals (integer maths only: no floats in the sim crate).
fn r(x: Fx) -> String {
    let h = i64::from(x.raw()) * 800 / 65536;
    let sign = if h < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", h.abs() / 100, h.abs() % 100)
}

fn solo(c: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    // A long flat floor, far from any edge or blast line.
    let st = &mut sim.content.stage;
    st.platforms[0].left = Fx::from_int(-300);
    st.platforms[0].right = Fx::from_int(300);
    st.platforms.truncate(1);
    st.ledges.clear();
    st.blast_left = Fx::from_int(-400);
    st.blast_right = Fx::from_int(400);
    sim.state = GameState::new_with_active(&sim.content, 1, [c, 0, 0, 0], 0b0001);
    sim.stand(0, Fx::from_int(-100), 1);
    sim.tick(inp(0, 0, 0));
    sim
}

/// Runs `before` ground inputs, then jumps with `jump` (held through the squat), then holds `air` until landing. Returns the
/// ground speed just before the jump, the speed on the first airborne frame, the distance from take-off to landing, the frames in
/// the air, and the facing in the air.
fn jump_case(
    c: u8,
    before: &[Input],
    jump: Input,
    air: Input,
    short: bool,
) -> (String, String, String, u32, i8) {
    let mut sim = solo(c);
    for i in before {
        sim.tick(*i);
    }
    let ground = r(sim.f().vel.x);
    sim.tick(jump);
    let mut takeoff = None;
    let mut x0 = Fx::ZERO;
    let mut frames = 0;
    let mut facing = 0;
    for k in 0..200 {
        let held = if short && k > 0 {
            Input { buttons: 0, ..jump }
        } else {
            jump
        };
        let input = if takeoff.is_none() {
            held
        } else {
            Input {
                buttons: if short { 0 } else { JUMP },
                ..air
            }
        };
        sim.tick(input);
        let f = sim.f();
        if takeoff.is_none() && !f.grounded() {
            takeoff = Some(r(f.vel.x));
            x0 = f.pos.x - f.vel.x;
            facing = f.facing;
        }
        if takeoff.is_some() {
            frames += 1;
            if f.grounded() || f.state == S::Landing {
                break;
            }
        }
    }
    (
        ground,
        takeoff.unwrap_or_default(),
        r(sim.f().pos.x - x0),
        frames,
        facing,
    )
}

#[test]
#[ignore]
fn table() {
    let n = Sim::new().content.fighters.len() as u8;
    let names = Sim::new().content.names.fighters.clone();
    let walk = vec![inp(60, 0, 0); 40];
    let mut dash = vec![inp(0, 0, 0)];
    dash.extend(vec![inp(127, 0, 0); 4]);
    let run = vec![inp(127, 0, 0); 40];
    println!("fighter | stand fwd | stand back | walk fwd | dash fwd | run fwd | run neutral | run back | RAR | run short hop fwd");
    for c in 0..n {
        let mut row = format!("{c:2} {:9}", names[usize::from(c)]);
        let cases: [(&[Input], Input, Input, bool); 9] = [
            (&[], inp(127, 0, JUMP), inp(127, 0, 0), false),
            (&[], inp(-127, 0, JUMP), inp(-127, 0, 0), false),
            (&walk, inp(60, 0, JUMP), inp(60, 0, 0), false),
            (&dash, inp(127, 0, JUMP), inp(127, 0, 0), false),
            (&run, inp(127, 0, JUMP), inp(127, 0, 0), false),
            (&run, inp(0, 0, JUMP), inp(0, 0, 0), false),
            (&run, inp(-127, 0, JUMP), inp(-127, 0, 0), false),
            (&run, inp(-127, 0, JUMP), inp(0, 0, 0), false),
            (&run, inp(127, 0, JUMP), inp(127, 0, 0), true),
        ];
        for (before, jump, air, short) in cases {
            let (g, v, d, fr, fc) = jump_case(c, before, jump, air, short);
            row += &format!(" | g{g} v{v} d{d} t{fr} f{fc}");
        }
        println!("{row}");
    }
}
