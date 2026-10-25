//! Bodies in the air (sim 146, after a playtest): only fighters both on the ground push each other, as the reference's jostle, so a
//! short hop crosses over a standing opponent and fighters pass each other in the air; a fighter landing inside another eases apart
//! from it along the ground. The ruleset's `air_push` 1 keeps the rule of sims 67 to 145 (bodies in the air pushed too).

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::JUMP;
use sim_core::{Fx, GameState};

/// Player 0 at x -2 and player 1 at x 0, both in the air at heights `y0` and `y1`, the others out of the way.
fn two_in_the_air(y0: i32, y1: i32) -> Sim {
    let mut sim = Sim::new();
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.put_airborne(0, Fx::from_int(-2), Fx::from_int(y0), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, Fx::ZERO, Fx::from_int(y1), Fx::ZERO, Fx::ZERO);
    sim
}

#[test]
fn fighters_in_the_air_pass_each_other() {
    let mut sim = two_in_the_air(12, 12);
    for _ in 0..30 {
        sim.tick(inp(127, 0, 0));
    }
    assert!(
        sim.fighter(0).pos.x > sim.fighter(1).pos.x,
        "blocked in the air"
    );
}

#[test]
fn with_air_push_drifting_into_a_fighter_in_the_air_stops_against_it() {
    let mut sim = two_in_the_air(12, 12);
    sim.content.rules.air_push = 1;
    for _ in 0..40 {
        sim.tick(inp(127, 0, 0));
        assert!(
            sim.fighter(0).pos.x < sim.fighter(1).pos.x,
            "drifted through at {:?}",
            sim.fighter(0).pos.x
        );
    }
}

/// Jumper `c` facing a standing mirror at 0 on a long floor, starting just outside its body (or a dash's length back), holding
/// forward; a short or full hop. Returns whether it landed beyond the other fighter, and how far the other one was moved.
fn hop_over(c: u8, dash: bool, short: bool) -> (bool, Fx) {
    let mut sim = Sim::with_chars([c, c, 0, 0]);
    let st = &mut sim.content.stage;
    st.platforms[0].left = Fx::from_int(-300);
    st.platforms[0].right = Fx::from_int(300);
    st.platforms.truncate(1);
    st.ledges.clear();
    st.blast_left = Fx::from_int(-400);
    st.blast_right = Fx::from_int(400);
    sim.state = GameState::new_with_active(&sim.content, 1, [c, c, 0, 0], 0b0011);
    let p = sim.content.fighters[usize::from(c)];
    let touch = (p.ecb_half_width + p.ecb_half_width) * sim.content.rules.push_ratio;
    let back = if dash {
        Fx::from_int(4)
    } else {
        Fx::from_ratio(1, 10)
    };
    sim.stand(0, -(touch + back), 1);
    sim.stand(1, Fx::ZERO, -1);
    sim.tick(inp(0, 0, 0));
    if dash {
        let mut n = 0;
        while sim.f().pos.x < -(touch + Fx::from_int(1)) && n < 60 {
            sim.tick(inp(127, 0, 0));
            n += 1;
        }
    }
    sim.tick(inp(127, 0, JUMP));
    let mut airborne = false;
    for k in 0..200 {
        sim.tick(inp(127, 0, if short && k > 0 { 0 } else { JUMP }));
        if !sim.f().grounded() {
            airborne = true;
        } else if airborne {
            break;
        }
    }
    (
        sim.f().pos.x > sim.fighter(1).pos.x,
        sim.fighter(1).pos.x.abs(),
    )
}

#[test]
fn every_fighters_short_hop_crosses_a_standing_mirror() {
    // Before sim 146 only two of fourteen did from a stand: a short hop peaks below the other's body height (the Longsword's 16.3
    // reference units under 17.6), so the bodies met all the way over and the push held them apart.
    for c in 0..Sim::new().content.fighters.len() as u8 {
        for dash in [false, true] {
            for short in [true, false] {
                let (over, shoved) = hop_over(c, dash, short);
                assert!(over, "fighter {c} dash {dash} short {short} did not cross");
                assert!(
                    shoved < Fx::from_ratio(1, 2),
                    "fighter {c} dash {dash} short {short} shoved the other {shoved:?}"
                );
            }
        }
    }
}

#[test]
fn landing_inside_a_standing_fighter_eases_apart_along_the_ground() {
    let mut sim = Sim::new();
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::ZERO, -1);
    sim.put_airborne(
        0,
        Fx::from_ratio(-1, 10),
        Fx::from_int(3),
        Fx::ZERO,
        Fx::ZERO,
    );
    let mut last = Fx::ZERO;
    for _ in 0..90 {
        sim.tick(inp(0, 0, 0));
        let gap = (sim.fighter(1).pos.x - sim.fighter(0).pos.x).abs();
        if sim.f().grounded() {
            assert!(gap >= last, "snapped back together: {gap:?} after {last:?}");
            last = gap;
        }
    }
    let p = sim.content.fighters[0];
    let q = sim.content.fighters[1];
    let width = (p.ecb_half_width + q.ecb_half_width) * sim.content.rules.push_ratio;
    assert!(sim.f().grounded());
    assert!(
        last >= width - Fx::from_ratio(1, 100),
        "still inside: {last:?}"
    );
}
