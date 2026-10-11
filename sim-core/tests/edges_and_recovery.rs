//! Rolls stay on the platform they start on (sim v41), and the special fall after an up special only drifts a little.

mod common;

use common::{inp, Sim};
use sim_core::state::FighterState as S;
use sim_core::Fx;

#[test]
fn a_roll_into_the_edge_of_the_stage_ends_on_the_edge() {
    let mut sim = Sim::new();
    sim.stand(1, Fx::ZERO, 1);
    let right = sim.content.stage.platforms[0].right;
    sim.stand(0, right - Fx::HALF, -1);
    // A roll toward the edge (behind the fighter).
    let f = &mut sim.state.fighters[0];
    f.state = S::Roll;
    f.state_frame = 0;
    f.dodge_dir = sim_core::vec2::Vec2::new(Fx::ONE, Fx::ZERO);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().grounded(), "never leaves the stage");
        assert!(sim.f().pos.x <= right);
    }
    assert_eq!(sim.f().state, S::Idle, "the roll ends standing on the edge");
}

#[test]
fn a_roll_on_a_floating_platform_stops_at_its_end_too() {
    let mut sim = Sim::new();
    sim.stand(1, Fx::ZERO, 1);
    let Some((i, plat)) = sim
        .content
        .stage
        .platforms
        .iter()
        .enumerate()
        .find(|(_, p)| p.pass_through)
    else {
        return;
    };
    let (left, y) = (plat.left, plat.y);
    let f = &mut sim.state.fighters[0];
    f.pos.x = left + Fx::HALF;
    f.pos.y = y;
    f.platform = i as i8;
    f.vel = sim_core::vec2::Vec2::ZERO;
    f.facing = 1;
    f.state = S::Roll;
    f.state_frame = 0;
    f.dodge_dir = sim_core::vec2::Vec2::new(-Fx::ONE, Fx::ZERO);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().grounded(), "never leaves the platform");
        assert!(sim.f().pos.x >= left);
    }
}

#[test]
fn the_special_fall_only_drifts_a_little() {
    for ch in [0u8, 1] {
        let mut sim = Sim::with_chars([ch, 0, 0, 0]);
        sim.stand(1, Fx::from_int(9), -1);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].state = S::Helpless;
        for _ in 0..60 {
            sim.tick(inp(127, 0, 0));
        }
        assert_eq!(sim.f().state, S::Helpless);
        let p = sim.content.fighters[usize::from(ch)];
        let cap = p.air_speed * sim.content.rules.helpless_drift;
        assert!(sim.f().vel.x > Fx::ZERO, "it does drift");
        assert!(
            sim.f().vel.x <= cap + Fx::from_raw(64),
            "class {ch}: {:?} against {:?}",
            sim.f().vel.x,
            cap
        );
    }
}
