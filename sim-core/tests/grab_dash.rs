//! Dashing out of a grab or a throw (sim v79): a dash flicked at any point of a grab, a pummel or a throw is kept, as long as the
//! stick stays pushed that way, and comes out on the first frame the fighter can act, so a throw that sends the opponent up and away
//! can be chased. (The reference only buffers a dash flicked in the last 6 or 7 frames; this is more lenient, the owner's call.)

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// `char_id` holding the duelist (at `percent`) after a standing grab, or a dash grab from further away.
fn holding(char_id: u8, dash_grab: bool, percent: i32) -> Sim {
    let mut sim = Sim::with_chars([char_id, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(if dash_grab { -6 } else { 0 }), 1);
    sim.stand(1, Fx::from_ratio(14, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(percent);
    sim.tick(inp(0, 0, 0));
    if dash_grab {
        let mut n = 0;
        while sim.fighter(1).pos.x - sim.fighter(0).pos.x > Fx::from_int(3) && n < 60 {
            sim.tick(inp(127, 0, 0));
            n += 1;
        }
        assert!(matches!(sim.f().state, S::Dash | S::Run));
    }
    sim.tick(inp(if dash_grab { 127 } else { 0 }, 0, GRAB));
    let mut n = 0;
    while sim.f().state != S::Grabbing && n < 40 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    assert_eq!(
        sim.f().state,
        S::Grabbing,
        "char {char_id}: the grab caught"
    );
    sim.tick(inp(0, 0, 0));
    sim
}

/// How many frames the throw that starts with `throw` (stick input) lasts, from the frame after it is picked.
fn throw_length(sim: &Sim, throw: (i8, i8)) -> usize {
    let mut probe = Sim {
        content: sim.content.clone(),
        state: sim.state,
    };
    probe.tick(inp(throw.0, throw.1, 0));
    let mut n = 0;
    while probe.f().state == S::Attack && n < 200 {
        probe.tick(inp(0, 0, 0));
        n += 1;
    }
    n
}

#[test]
fn a_dash_flicked_early_in_a_throw_comes_out_the_moment_it_ends() {
    for c in 0..6u8 {
        for dash_grab in [false, true] {
            let mut sim = holding(c, dash_grab, 0);
            let len = throw_length(&sim, (0, -127));
            sim.tick(inp(0, -127, 0));
            assert_eq!(sim.f().move_id, MoveId::DThrow as u8);
            // A flick forward a few frames into the throw (long before the reference's last 7), then held.
            for t in 0..len + 3 {
                let x = if t < 3 { 0 } else { 127 };
                sim.tick(inp(x, 0, 0));
                if sim.f().state != S::Attack {
                    // The throw has ended (standing, as after any move); the next frame is the first the fighter can act.
                    assert_eq!(t + 1, len, "char {c}: no frame lost");
                    sim.tick(inp(127, 0, 0));
                    assert_eq!(
                        sim.f().state,
                        S::Dash,
                        "char {c}, dash grab {dash_grab}: dashing on the first frame it can act"
                    );
                    break;
                }
            }
        }
    }
}

#[test]
fn letting_go_of_the_stick_cancels_the_buffered_dash() {
    let mut sim = holding(2, false, 0);
    let len = throw_length(&sim, (0, -127));
    sim.tick(inp(0, -127, 0));
    for t in 0..len + 3 {
        // Flicked, then let go a few frames before the end.
        let x = if (3..len - 4).contains(&t) { 127 } else { 0 };
        sim.tick(inp(x, 0, 0));
    }
    assert_ne!(sim.f().state, S::Dash);
}

#[test]
fn holding_the_stick_that_picked_a_forward_throw_walks_out_rather_than_dashing() {
    // The flick that chose the throw was made while holding, before the throw began: it is not a buffered dash.
    let mut sim = holding(0, false, 0);
    let len = throw_length(&sim, (127, 0));
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().move_id, MoveId::FThrow as u8);
    let mut states = Vec::new();
    for _ in 0..len + 3 {
        sim.tick(inp(127, 0, 0));
        states.push(sim.f().state);
    }
    assert!(!states.contains(&S::Dash), "{states:?}");
}

#[test]
fn a_dash_flicked_during_a_missed_grab_comes_out_when_it_ends() {
    let mut sim = Sim::alone();
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, GRAB));
    assert_eq!(sim.f().move_id, MoveId::Grab as u8);
    let mut n = 0;
    while sim.f().state == S::Attack && n < 80 {
        let x = if n < 5 { 0 } else { 127 };
        sim.tick(inp(x, 0, 0));
        n += 1;
    }
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
}

#[test]
fn a_command_grab_that_ends_in_the_air_does_not_dash() {
    // The sprinter's leaping grab: caught, carried and thrown in the air. A forward flick during it does not dash in the air, and
    // landing afterwards is the ordinary landing (a dash out of it needs a new flick).
    let mut sim = Sim::with_chars([4, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(14, 10), -1);
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 127, SPECIAL));
    let mut states = Vec::new();
    let mut caught = false;
    for t in 0..140 {
        let x = if t < 5 { 0 } else { 127 };
        sim.tick(inp(x, 0, 0));
        caught |= sim.f().grab_with >= 0;
        states.push(sim.f().state);
    }
    assert!(caught, "the leaping grab caught");
    assert!(!states.contains(&S::Dash), "{states:?}");
    assert_eq!(sim.f().dash_buffer, 0);
}

#[test]
fn the_rogues_down_throw_can_be_chased_with_a_dash_into_a_forward_air() {
    // At 0%: the throw, a dash flicked early in it, three frames of dash, a full hop, a forward air on leaving the ground. The
    // opponent is still in hitstun when it lands: a true combo, only possible because the dash comes out at once.
    let mut sim = holding(2, false, 0);
    let len = throw_length(&sim, (0, -127));
    sim.tick(inp(0, -127, 0));
    for t in 0..=len {
        sim.tick(inp(if t < 3 { 0 } else { 127 }, 0, 0));
    }
    assert_eq!(sim.f().state, S::Dash);
    let p0 = sim.fighter(1).percent;
    sim.ticks(3, inp(127, 0, 0));
    sim.tick(inp(127, 0, JUMP));
    let mut hit_in_stun = false;
    let mut attacked = false;
    for _ in 0..40 {
        let b = if !attacked && !sim.f().grounded() {
            attacked = true;
            ATTACK
        } else {
            JUMP
        };
        let in_stun = sim.fighter(1).state == S::Hitstun && sim.fighter(1).hitstun > 0;
        sim.tick(inp(127, 0, b));
        if sim.fighter(1).percent > p0 {
            hit_in_stun = in_stun;
            break;
        }
    }
    assert!(
        hit_in_stun,
        "the forward air connects before the opponent can act"
    );
}
