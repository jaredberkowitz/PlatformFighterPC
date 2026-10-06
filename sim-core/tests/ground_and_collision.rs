//! Phase 1 completion: ground states, walls, ceilings, ECB, platform drop, helpless fall, ledge attack.

mod common;

use common::{fx, inp, Sim};
use sim_core::content::Platform;
use sim_core::fighter::enter_helpless;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::state::{FighterState as S, NONE};
use sim_core::Fx;

fn with_block(left: i32, right: i32, bottom: i32, top: i32) -> Sim {
    let mut sim = Sim::new();
    sim.content.stage.platforms.push(Platform {
        left: Fx::from_int(left),
        right: Fx::from_int(right),
        y: Fx::from_int(top),
        bottom: Fx::from_int(bottom),
        pass_through: false,
    });
    sim
}

// ---- Walk, dash, run, turn, crouch -------------------------------------------------------------

#[test]
fn a_gentle_tilt_walks_and_speed_scales_with_tilt() {
    let speed = |tilt: i8| {
        let mut sim = Sim::new();
        sim.ticks(30, inp(tilt, 0, 0));
        assert_eq!(sim.f().state, S::Walk, "tilt {tilt}");
        sim.f().vel.x
    };
    let (slow, fast) = (speed(30), speed(70));
    let walk = Sim::new().content.fighters[0].walk_speed;
    assert!(
        slow > Fx::ZERO && slow < fast && fast <= walk,
        "{slow:?} {fast:?} {walk:?}"
    );
}

#[test]
fn a_quick_flick_dashes_then_runs_while_held() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
    assert_eq!(sim.f().vel.x, p.dash_speed);
    sim.ticks(usize::from(p.dash_frames) + 2, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    sim.ticks(20, inp(127, 0, 0));
    assert_eq!(sim.f().vel.x, p.run_speed);
    assert!(p.dash_speed > p.run_speed);
}

#[test]
fn a_dash_ends_in_idle_if_the_stick_is_released() {
    let mut sim = Sim::new();
    let frames = usize::from(sim.content.fighters[0].dash_frames);
    sim.tick(inp(127, 0, 0));
    sim.ticks(frames + 1, inp(0, 0, 0));
    sim.ticks(10, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn slowly_rolling_the_stick_to_full_does_not_dash() {
    let mut sim = Sim::new();
    for x in (0..=127).step_by(13) {
        sim.tick(inp(x as i8, 0, 0));
        assert_ne!(sim.f().state, S::Dash, "dashed at x={x}");
    }
    sim.ticks(5, inp(127, 0, 0));
    assert_ne!(sim.f().state, S::Dash);
}

#[test]
fn reversing_a_dash_with_a_flick_is_a_dash_dance() {
    let mut sim = Sim::new();
    sim.ticks(4, inp(127, 0, 0));
    assert_eq!((sim.f().state, sim.f().facing), (S::Dash, 1));
    sim.tick(inp(-127, 0, 0));
    assert_eq!((sim.f().state, sim.f().facing), (S::Dash, -1));
    assert!(sim.f().vel.x < Fx::ZERO);
}

#[test]
fn a_slow_reverse_turns_around_before_walking() {
    let mut sim = Sim::new();
    assert_eq!(sim.f().facing, 1);
    sim.tick(inp(-40, 0, 0));
    assert_eq!(sim.f().state, S::Turn);
    assert_eq!(sim.f().facing, -1);
    let turn = usize::from(sim.content.fighters[0].turn_frames);
    sim.ticks(turn + 3, inp(-40, 0, 0));
    assert_eq!(sim.f().state, S::Walk);
    assert!(sim.f().vel.x < Fx::ZERO);
}

#[test]
fn crouch_holds_still_and_releases() {
    let mut sim = Sim::new();
    sim.ticks(10, inp(0, -100, 0));
    assert_eq!(sim.f().state, S::Crouch);
    assert_eq!(sim.f().vel.x, Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn jumping_works_out_of_every_ground_state() {
    for (name, x, y) in [("walk", 40, 0), ("run", 127, 0), ("crouch", 0, -100)] {
        let mut sim = Sim::new();
        sim.ticks(20, inp(x, y, 0));
        sim.tick(inp(x, y, JUMP));
        assert_eq!(sim.f().state, S::JumpSquat, "{name}");
    }
}

// ---- Tap-down platform drop -------------------------------------------------------------------

#[test]
fn tapping_down_on_a_pass_through_platform_drops_through() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos = sim_core::Vec2::new(Fx::from_int(-8), Fx::from_int(6));
    sim.state.fighters[0].platform = 1;
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().state, S::Airborne);
    assert!(!sim.f().grounded());
    sim.ticks(80, inp(0, 0, 0));
    assert_eq!(sim.f().platform, 0);
}

#[test]
fn tapping_down_on_solid_ground_just_crouches() {
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().state, S::Crouch);
    assert_eq!(sim.f().pos.y, Fx::ZERO);
}

// ---- Walls, ceilings, ECB ---------------------------------------------------------------------

#[test]
fn a_wall_stops_a_running_fighter() {
    let mut sim = with_block(8, 10, 0, 4);
    let hw = sim.content.fighters[0].ecb_half_width;
    sim.ticks(120, inp(127, 0, 0));
    assert_eq!(sim.f().pos.x, Fx::from_int(8) - hw);
    assert_eq!(sim.f().vel.x, Fx::ZERO);
    assert!(sim.f().grounded());
}

#[test]
fn a_wall_stops_an_airborne_fighter_and_its_top_can_be_landed_on() {
    // A pillar clear of the pass-through platforms (which end at x = 12).
    let mut sim = with_block(14, 16, 0, 4);
    let hw = sim.content.fighters[0].ecb_half_width;
    // Drifting into the wall from the left while below its top: blocked.
    sim.put_airborne(0, fx(23, 2), Fx::from_int(1), fx(1, 10), fx(1, 100));
    sim.ticks(40, inp(127, 0, 0));
    assert!(sim.f().pos.x <= Fx::from_int(14) - hw);
    // Starting above the top lets it land on the pillar.
    sim.put_airborne(0, Fx::from_int(13), Fx::from_int(8), fx(1, 10), Fx::ZERO);
    for _ in 0..60 {
        sim.tick(inp(127, 0, 0));
        if sim.f().grounded() {
            break;
        }
    }
    assert_eq!(sim.f().platform, 3, "landed on the pillar");
    assert_eq!(sim.f().pos.y, Fx::from_int(4));
}

#[test]
fn a_ceiling_stops_upward_movement() {
    let mut sim = with_block(-4, 4, 8, 12);
    let p = sim.content.fighters[0];
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(3), Fx::ZERO, fx(1, 4));
    let mut peak = Fx::ZERO;
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        peak = peak.max(sim.f().pos.y);
    }
    assert_eq!(
        peak,
        Fx::from_int(8) - p.ecb_height,
        "head stops at the underside"
    );
}

#[test]
fn the_ecb_is_a_diamond_so_a_low_corner_does_not_stop_it_like_a_wide_box() {
    // Feet just above a block top, drifting sideways: only the narrow bottom of the diamond is
    // beside the block, so it is not stopped at the full width.
    let mut sim = with_block(8, 10, 0, 4);
    let tiny = fx(1, 20);
    sim.put_airborne(
        0,
        Fx::from_int(7),
        Fx::from_int(4) + tiny,
        fx(1, 10),
        Fx::ZERO,
    );
    sim.tick(inp(127, 0, 0));
    assert!(sim.f().pos.x > Fx::from_int(7), "not blocked by a wide box");
}

#[test]
fn diagonal_air_dodge_into_a_corner_never_embeds_the_fighter() {
    let mut sim = Sim::new();
    for start_y in [-1, 0, 1, 2, 5] {
        sim.put_airborne(
            0,
            Fx::from_int(-23),
            Fx::from_int(start_y),
            Fx::ZERO,
            Fx::ZERO,
        );
        sim.tick(inp(100, -90, SHIELD));
        for _ in 0..40 {
            sim.tick(inp(100, -90, 0));
            let f = sim.f();
            let inside = f.pos.x > Fx::from_int(-20)
                && f.pos.x < Fx::from_int(20)
                && f.pos.y > Fx::from_int(-8)
                && f.pos.y < Fx::ZERO;
            assert!(!inside, "start_y {start_y}: embedded at {:?}", f.pos);
        }
    }
}

// ---- Helpless fall ----------------------------------------------------------------------------

#[test]
fn helpless_fighters_cannot_jump_or_air_dodge() {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    enter_helpless(&mut sim.state.fighters[0]);
    let jumps = sim.f().air_jumps_left;
    sim.tick(inp(0, 0, JUMP));
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Helpless);
    assert_eq!(sim.f().air_jumps_left, jumps);
    assert!(!sim.f().air_dodge_used);
    assert!(sim.f().vel.y < Fx::ZERO);
}

#[test]
fn helpless_landing_costs_extra_lag() {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    enter_helpless(&mut sim.state.fighters[0]);
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, sim.content.fighters[0].helpless_landing_lag);
    assert!(sim.f().lag > sim.content.fighters[0].landing_lag);
    let lag = usize::from(sim.f().lag);
    sim.ticks(lag + 2, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

#[test]
fn helpless_fighters_can_still_grab_ledges() {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::from_int(-21), Fx::from_int(-1), Fx::ZERO, fx(-1, 20));
    enter_helpless(&mut sim.state.fighters[0]);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::LedgeHang);
    assert_eq!(sim.state.ledge_owner[0], 0);
}

// ---- Ledge attack -----------------------------------------------------------------------------

#[test]
fn ledge_attack_gets_up_onto_the_stage_and_takes_longer_than_a_normal_get_up() {
    let recover = |button: u16, y: i8| {
        let mut sim = Sim::new();
        sim.put_airborne(0, Fx::from_int(-21), Fx::from_int(-1), Fx::ZERO, fx(-1, 20));
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().state, S::LedgeHang);
        sim.tick(inp(0, y, button));
        let state = sim.f().state;
        assert_eq!(sim.f().platform, 0);
        assert_ne!(sim.f().platform, NONE);
        let mut frames = 0;
        while sim.f().state == state {
            sim.tick(inp(0, 0, 0));
            frames += 1;
            assert!(frames < 200);
        }
        (state, frames)
    };
    let (attack_state, attack_frames) = recover(ATTACK, 0);
    let (getup_state, getup_frames) = recover(0, 127);
    assert_eq!(attack_state, S::LedgeAttack);
    assert_eq!(getup_state, S::LedgeGetUp);
    assert!(attack_frames > getup_frames);
}
