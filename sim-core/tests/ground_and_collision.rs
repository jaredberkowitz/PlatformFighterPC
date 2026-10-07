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
    assert!(sim.f().vel.x >= p.dash_initial_speed && sim.f().vel.x < p.dash_speed);
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
    sim.state.fighters[0].pos = sim_core::Vec2::new(Fx::from_int(-5), Fx::from_ratio(18, 5));
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
    // A pillar just past the pass-through platforms (which end at x = 8), on the main stage.
    let mut sim = with_block(9, 10, 0, 4);
    let hw = sim.content.fighters[0].ecb_half_width;
    // Drifting into the wall from the left while below its top: blocked.
    sim.put_airborne(0, fx(15, 2), Fx::from_int(1), fx(1, 10), fx(1, 100));
    sim.ticks(40, inp(127, 0, 0));
    assert!(sim.f().pos.x <= Fx::from_int(9) - hw);
    // Starting above the top lets it land on the pillar.
    sim.put_airborne(0, fx(19, 2), Fx::from_int(8), Fx::ZERO, Fx::ZERO);
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
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
            Fx::from_int(-13),
            Fx::from_int(start_y),
            Fx::ZERO,
            Fx::ZERO,
        );
        sim.tick(inp(100, -90, SHIELD));
        for _ in 0..40 {
            sim.tick(inp(100, -90, 0));
            let f = sim.f();
            let inside = f.pos.x > Fx::from_int(-11)
                && f.pos.x < Fx::from_int(11)
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
    sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(-1), Fx::ZERO, fx(-1, 20));
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
        sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(-1), Fx::ZERO, fx(-1, 20));
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

// ---- Smooth dash and landing ------------------------------------------------------------------

#[test]
fn a_dash_ramps_up_instead_of_jumping_to_full_speed() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    let mut last = Fx::ZERO;
    let mut reached_top = false;
    for frame in 0..usize::from(p.dash_frames) {
        sim.tick(inp(127, 0, 0));
        let v = sim.f().vel.x;
        // The first frame is the initial speed plus one step; after that, one step at a time.
        let allowed = if frame == 0 {
            p.dash_initial_speed + p.dash_accel
        } else {
            p.dash_accel
        };
        assert!(v - last <= allowed, "jumped from {last:?} to {v:?}");
        assert!(v >= last, "dash slowed down mid-ramp");
        last = v;
        reached_top |= v == p.dash_speed;
    }
    assert!(reached_top, "never reached dash speed");
}

#[test]
fn dash_speed_eases_down_into_run_speed() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-10);
    let p = sim.content.fighters[0];
    sim.ticks(usize::from(p.dash_frames) + 1, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    let mut last = sim.f().vel.x;
    assert!(last > p.run_speed, "should still carry dash speed");
    for _ in 0..60 {
        sim.tick(inp(127, 0, 0));
        let v = sim.f().vel.x;
        assert!(last - v <= p.run_decel, "snapped from {last:?} to {v:?}");
        last = v;
    }
    assert_eq!(last, p.run_speed);
}

#[test]
fn landing_while_holding_the_stick_carries_momentum_into_a_run() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    sim.put_airborne(0, Fx::from_int(-8), Fx::from_int(2), fx(1, 10), Fx::ZERO);
    let mut landed_speed = None;
    let mut last = None;
    for _ in 0..60 {
        sim.tick(inp(127, 0, 0));
        let grounded = sim.f().grounded();
        let v = sim.f().vel.x;
        if grounded {
            landed_speed.get_or_insert(v);
            if let Some(prev) = last {
                // No snapping: whatever the state, speed only falls by the gentle easing rate.
                assert!(
                    prev - v <= p.run_decel,
                    "snapped from {prev:?} to {v:?} in {:?}",
                    sim.f().state
                );
            }
            last = Some(v);
        }
    }
    let landed = landed_speed.expect("never landed");
    assert!(landed > fx(1, 20), "landing killed the speed: {landed:?}");
    // Holding the stick the way you were moving continues straight into a run: no walk phase.
    assert_eq!(sim.f().state, S::Run);
    assert_eq!(sim.f().vel.x, p.run_speed);
}

#[test]
fn landing_without_input_slides_gently_rather_than_stopping_dead() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(2), fx(1, 10), Fx::ZERO);
    let mut speeds = Vec::new();
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            speeds.push(sim.f().vel.x);
        }
    }
    assert!(speeds.len() >= 2, "landing lag too short to observe");
    for pair in speeds.windows(2) {
        assert!(
            pair[0] - pair[1] <= p.landing_friction,
            "bled off faster than landing_friction"
        );
    }
    assert!(p.landing_friction < p.ground_friction);
}

// ---- Reference jump heights and momentum ------------------------------------------------------

/// World units per reference unit (see `FighterParams::su`).
fn reference(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

fn assert_close(actual: Fx, expected: Fx, what: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff < fx(1, 25),
        "{what}: got {actual:?}, expected {expected:?}"
    );
}

fn full_hop_apex(chars: [u8; 4]) -> Fx {
    let mut sim = Sim::with_chars(chars);
    let mut peak = Fx::ZERO;
    for _ in 0..90 {
        sim.tick(inp(0, 0, JUMP));
        peak = peak.max(sim.f().pos.y);
    }
    peak
}

fn short_hop_apex(chars: [u8; 4]) -> Fx {
    let mut sim = Sim::with_chars(chars);
    let mut peak = Fx::ZERO;
    for t in 0..90 {
        sim.tick(inp(0, 0, if t == 0 { JUMP } else { 0 }));
        peak = peak.max(sim.f().pos.y);
    }
    peak
}

fn air_jump_gain(chars: [u8; 4]) -> Fx {
    let mut sim = Sim::with_chars(chars);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    let mut peak = Fx::ZERO;
    sim.tick(inp(0, 0, JUMP));
    for _ in 0..90 {
        sim.tick(inp(0, 0, 0));
        peak = peak.max(sim.f().pos.y);
    }
    peak - Fx::from_int(30)
}

#[test]
fn duelist_jump_heights_match_the_reference_values() {
    let chars = [0, 1, 0, 1];
    assert_close(full_hop_apex(chars), reference(33_660), "full hop");
    assert_close(short_hop_apex(chars), reference(16_260), "short hop");
    assert_close(air_jump_gain(chars), reference(33_660), "double jump");
}

#[test]
fn brawler_jump_heights_match_the_reference_values() {
    let chars = [1, 0, 1, 0];
    assert_close(full_hop_apex(chars), reference(32_020), "full hop");
    assert_close(short_hop_apex(chars), reference(15_380), "short hop");
    assert_close(air_jump_gain(chars), reference(30_710), "double jump");
}

#[test]
fn the_double_jump_is_about_as_high_as_the_first_jump() {
    for chars in [[0, 1, 0, 1], [1, 0, 1, 0]] {
        let (first, second) = (full_hop_apex(chars), air_jump_gain(chars));
        assert!(
            second <= first + fx(1, 100),
            "double jump should not beat the first jump"
        );
        assert!(
            second >= first * fx(9, 10),
            "double jump is too weak: {second:?} vs {first:?}"
        );
    }
}

#[test]
fn a_running_jump_keeps_its_ground_speed_into_the_air() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-10);
    let p = sim.content.fighters[0];
    sim.ticks(30, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    let run = sim.f().vel.x;
    sim.tick(inp(127, 0, JUMP));
    // Jump squat, then takeoff: no friction at all, so no speed is lost on the way off the ground.
    for _ in 0..usize::from(p.jump_squat_frames) {
        sim.tick(inp(127, 0, JUMP));
    }
    assert!(!sim.f().grounded());
    assert_eq!(sim.f().vel.x, run, "lost speed leaving the ground");
    // In the air it keeps almost all of it, since only light air friction acts on the extra speed.
    sim.ticks(15, inp(127, 0, 0));
    assert!(sim.f().vel.x >= run - p.air_friction.mul_int(16));
    assert!(
        sim.f().vel.x > p.air_speed,
        "momentum should exceed plain air speed"
    );
}

#[test]
fn walking_off_the_edge_while_running_keeps_the_speed() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(6);
    sim.ticks(40, inp(127, 0, 0));
    assert!(!sim.f().grounded() || sim.f().state == S::LedgeHang);
    let run = sim.content.fighters[0].run_speed;
    assert!(sim.f().vel.x >= run - sim.content.fighters[0].air_friction.mul_int(30));
}

#[test]
fn air_speed_still_limits_drift_for_a_standing_jump() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.ticks(60, inp(127, 0, 0));
    assert_eq!(sim.f().vel.x, p.air_speed);
}

// ---- The ledge snap bug -----------------------------------------------------------------------

#[test]
fn walking_off_the_stage_does_not_snap_straight_onto_the_ledge() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-9);
    let mut positions = Vec::new();
    let mut hung_at = None;
    for t in 0..80 {
        sim.tick(inp(-127, 0, 0));
        positions.push(sim.f().pos);
        if sim.f().state == S::LedgeHang && hung_at.is_none() {
            hung_at = Some(t);
        }
    }
    // No frame may teleport the fighter: it falls first, and only then grabs.
    for pair in positions.windows(2) {
        let jump = (pair[1] - pair[0]).length();
        assert!(jump < Fx::ONE.mul_int(2), "teleported by {jump:?}");
    }
    let drop = sim.content.fighters[0].ledge_min_drop;
    if let Some(t) = hung_at {
        assert!(t > 2, "grabbed the instant it walked off");
        // The fighter is hanging below the ledge, never above it.
        assert!(positions[t].y <= -drop, "hung too high: {:?}", positions[t]);
    }
}

#[test]
fn a_fighter_dropping_in_from_above_the_stage_lands_instead_of_grabbing() {
    let mut sim = Sim::new();
    // Level with and just above the left ledge, over the stage, falling.
    sim.put_airborne(0, fx(-109, 10), fx(3, 10), Fx::ZERO, fx(-1, 20));
    sim.ticks(10, inp(0, 0, 0));
    assert_ne!(sim.f().state, S::LedgeHang);
    assert_eq!(sim.f().platform, 0);
}

#[test]
fn a_fighter_over_the_stage_cannot_grab_the_ledge_from_the_stage_side() {
    let mut sim = Sim::new();
    // Drifting left across the edge region at ledge height, facing the ledge from inside.
    sim.put_airborne(0, fx(-105, 10), fx(-1, 10), fx(-1, 20), fx(-1, 50));
    sim.ticks(3, inp(-127, 0, 0));
    assert_ne!(sim.f().state, S::LedgeHang);
}

// ---- Dash cancel -------------------------------------------------------------------------------

#[test]
fn releasing_the_stick_cancels_the_dash_straight_away() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    sim.ticks(4, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
    assert!(
        4 < usize::from(p.dash_frames),
        "test assumes the dash is still in progress"
    );
    let before = sim.f().vel.x;
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Idle,
        "dash should end the frame the stick is released"
    );
    // The speed carries into a slide on ground friction instead of stopping dead.
    let after = sim.f().vel.x;
    assert!(after > Fx::ZERO && after < before);
    assert!(before - after <= p.dash_brake);
    assert!(
        p.dash_brake > p.ground_friction,
        "cancelled dashes should brake harder than plain friction"
    );
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().vel.x, Fx::ZERO);
}

#[test]
fn a_quick_tap_makes_a_short_dash_and_a_held_stick_makes_a_long_one() {
    let travel = |held_frames: usize| {
        let mut sim = Sim::new();
        sim.state.fighters[0].pos.x = Fx::from_int(-10);
        let start = sim.f().pos.x;
        sim.ticks(held_frames, inp(127, 0, 0));
        sim.ticks(40, inp(0, 0, 0));
        sim.f().pos.x - start
    };
    let (tap, short, long) = (travel(1), travel(4), travel(12));
    assert!(tap > Fx::ZERO);
    assert!(tap < short && short < long, "{tap:?} {short:?} {long:?}");
}

#[test]
fn jumping_out_of_a_cancelled_dash_keeps_the_momentum_in_the_air() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-10);
    let p = sim.content.fighters[0];
    sim.ticks(6, inp(127, 0, 0));
    sim.tick(inp(0, 0, 0)); // release: dash cancels into a slide
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
    let squat_start = sim.f().vel.x;
    assert!(
        squat_start > p.air_speed,
        "test needs speed above air speed"
    );
    for _ in 0..usize::from(p.jump_squat_frames) {
        sim.tick(inp(0, 0, JUMP));
    }
    assert!(!sim.f().grounded());
    assert_eq!(
        sim.f().vel.x,
        squat_start,
        "jump squat must not eat the speed"
    );
    // With no stick held the air drift continues, slowed only by light air friction.
    sim.ticks(10, inp(0, 0, 0));
    assert!(sim.f().vel.x >= squat_start - p.air_friction.mul_int(11));
    assert!(sim.f().vel.x > p.air_speed);
}

#[test]
fn a_pivot_still_works_out_of_a_dash() {
    let mut sim = Sim::new();
    sim.ticks(3, inp(127, 0, 0));
    sim.tick(inp(-127, 0, 0));
    assert_eq!((sim.f().state, sim.f().facing), (S::Dash, -1));
}

// ---- Dash dancing and jumping in ----------------------------------------------------------------

#[test]
fn a_dash_dance_stays_in_a_tight_space_and_every_reversal_turns_instantly() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::ZERO;
    let (mut lo, mut hi) = (Fx::ZERO, Fx::ZERO);
    for cycle in 0..8 {
        let dir: i8 = if cycle % 2 == 0 { 1 } else { -1 };
        // Three frames each way is a typical dash-dance rhythm.
        for frame in 0..3 {
            sim.tick(inp(127 * dir, 0, 0));
            if frame == 0 {
                assert_eq!(
                    sim.f().state,
                    S::Dash,
                    "cycle {cycle}: reversal should start a dash at once"
                );
                assert_eq!(
                    sim.f().facing,
                    dir,
                    "cycle {cycle}: should face the new direction"
                );
                assert_eq!(
                    sim.f().vel.x.signum_int(),
                    i32::from(dir),
                    "cycle {cycle}: velocity should flip"
                );
            }
            lo = lo.min(sim.f().pos.x);
            hi = hi.max(sim.f().pos.x);
        }
    }
    assert!(
        hi - lo < Fx::from_int(3),
        "dash dance wandered {:?}",
        hi - lo
    );
}

#[test]
fn a_dash_dance_with_a_brief_neutral_between_dashes_also_works() {
    let mut sim = Sim::new();
    for cycle in 0..6 {
        let dir: i8 = if cycle % 2 == 0 { 1 } else { -1 };
        sim.ticks(3, inp(127 * dir, 0, 0));
        assert_eq!(sim.f().state, S::Dash);
        sim.tick(inp(0, 0, 0)); // the stick passes through neutral on the way across
    }
}

#[test]
fn dashing_in_then_jumping_gives_a_controllable_jump_in() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-10);
    let p = sim.content.fighters[0];
    // Dance a little, then dash toward the opponent and jump.
    for cycle in 0..4 {
        let dir: i8 = if cycle % 2 == 0 { -1 } else { 1 };
        sim.ticks(3, inp(127 * dir, 0, 0));
    }
    sim.ticks(5, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Dash);
    let takeoff_x = sim.f().pos.x;
    sim.tick(inp(127, 0, JUMP));
    sim.ticks(usize::from(p.jump_squat_frames), inp(127, 0, JUMP));
    assert!(!sim.f().grounded());
    assert!(
        sim.f().vel.x > p.air_speed,
        "should carry dash speed into the jump"
    );

    // Holding forward keeps the speed; letting go slows it a little; pulling back reduces the distance.
    let landing_x = |stick: i8| {
        let mut s = sim.state;
        let mut sim2 = Sim::new();
        sim2.state = s;
        for _ in 0..90 {
            sim2.tick(inp(stick, 0, 0));
            if sim2.f().grounded() {
                break;
            }
        }
        s = sim2.state;
        s.fighters[0].pos.x
    };
    let (forward, neutral, back) = (landing_x(127), landing_x(0), landing_x(-127));
    // Holding forward or neutral both keep the dash speed (only light air friction acts on it),
    // while pulling back brakes the drift, so landing spot is steerable in the air.
    assert!(forward >= neutral, "{forward:?} {neutral:?}");
    assert!(neutral > back + Fx::from_int(2), "{neutral:?} {back:?}");
    assert!(
        forward > takeoff_x + Fx::from_int(2),
        "a dash jump should travel well forward"
    );
}

// ---- Fast fall needs a hard press -----------------------------------------------------------------

fn falling_sim() -> Sim {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    // Let it start falling, with the stick at neutral.
    sim.ticks(10, inp(0, 0, 0));
    assert!(sim.f().vel.y < Fx::ZERO);
    sim
}

#[test]
fn a_hard_down_press_while_falling_fast_falls() {
    let mut sim = falling_sim();
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall);
    sim.ticks(40, inp(0, -127, 0));
    assert_eq!(sim.f().vel.y, -sim.content.fighters[0].fast_fall_speed);
}

#[test]
fn simply_holding_down_does_not_fast_fall() {
    let mut sim = Sim::new();
    // Jump with down already held the whole time (never a fresh press while falling).
    sim.put_airborne(
        0,
        Fx::ZERO,
        Fx::from_int(40),
        Fx::ZERO,
        Fx::from_ratio(1, 5),
    );
    sim.ticks(80, inp(0, -127, 0));
    // The very first frame counts as a fresh press only if it was while falling; here it was rising.
    assert!(
        !sim.f().fast_fall,
        "held down since before the apex must not fast fall"
    );
}

#[test]
fn a_soft_down_press_does_not_fast_fall_but_still_counts_as_down() {
    // The keyboard's first, softer down press (about 0.55) is below the hard-press threshold.
    let mut sim = falling_sim();
    sim.ticks(20, inp(0, -70, 0));
    assert!(!sim.f().fast_fall);
    // ... while still working as "down" on the ground: crouch and platform drop.
    let mut ground = Sim::new();
    ground.tick(inp(0, -70, 0));
    assert_eq!(ground.f().state, S::Crouch);
}

#[test]
fn rolling_the_stick_slowly_down_does_not_fast_fall() {
    let mut sim = falling_sim();
    for y in (0..=127).step_by(8) {
        sim.tick(inp(0, -(y as i8), 0));
    }
    sim.ticks(10, inp(0, -127, 0));
    assert!(!sim.f().fast_fall, "a slow roll is not a hard press");
}

#[test]
fn a_double_tap_style_input_fast_falls_on_the_second_press() {
    // Tap down softly, release, then press hard: what the keyboard's double tap produces.
    let mut sim = falling_sim();
    sim.ticks(3, inp(0, -70, 0));
    sim.ticks(2, inp(0, 0, 0));
    assert!(!sim.f().fast_fall);
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall);
}

#[test]
fn a_soft_down_tap_still_drops_through_a_platform() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos = sim_core::Vec2::new(Fx::from_int(-5), Fx::from_ratio(18, 5));
    sim.state.fighters[0].platform = 1;
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, -70, 0));
    assert_eq!(sim.f().state, S::Airborne);
    assert!(!sim.f().grounded());
}

// ---- Not slippery ---------------------------------------------------------------------------------

/// Distance covered from letting go of the stick until the fighter stops.
fn stopping_distance(held_frames: usize) -> Fx {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-10);
    sim.ticks(held_frames, inp(127, 0, 0));
    let release_x = sim.f().pos.x;
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        if sim.f().vel.x == Fx::ZERO {
            break;
        }
    }
    assert_eq!(sim.f().vel.x, Fx::ZERO, "never stopped");
    sim.f().pos.x - release_x
}

#[test]
fn stopping_from_a_full_dash_or_run_is_short_and_quick() {
    let body_width = Fx::from_ratio(8, 5); // 2 * ecb_half_width
    let from_dash = stopping_distance(10);
    let from_run = stopping_distance(40);
    assert!(
        from_dash < body_width.mul_int(1) * Fx::HALF + Fx::HALF,
        "dash cancel slid {from_dash:?}"
    );
    assert!(
        from_run < body_width * Fx::from_ratio(3, 5),
        "run stop slid {from_run:?}"
    );
    assert!(
        from_dash > Fx::ZERO && from_run > Fx::ZERO,
        "some slide is fine, none is not"
    );
}

#[test]
fn stopping_from_a_full_run_takes_under_ten_frames() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(-10);
    sim.ticks(40, inp(127, 0, 0));
    let mut frames = 0;
    while sim.f().vel.x != Fx::ZERO {
        sim.tick(inp(0, 0, 0));
        frames += 1;
        assert!(frames < 60);
    }
    assert!(frames <= 10, "took {frames} frames to stop");
}

#[test]
fn a_hard_press_just_before_the_apex_fast_falls_once_falling_starts() {
    // Press hard while still rising; the fighter reaches the apex a few frames later.
    let mut sim = Sim::new();
    sim.put_airborne(
        0,
        Fx::ZERO,
        Fx::from_int(40),
        Fx::ZERO,
        Fx::from_ratio(1, 20),
    );
    sim.ticks(1, inp(0, 0, 0));
    sim.tick(inp(0, -127, 0));
    assert!(
        sim.f().vel.y > Fx::ZERO,
        "test needs the fighter still rising"
    );
    assert!(!sim.f().fast_fall);
    sim.ticks(1, inp(0, 0, 0));
    for _ in 0..8 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(
        sim.f().fast_fall,
        "a recent hard press should apply once the fighter starts falling"
    );
}

#[test]
fn a_hard_press_long_before_the_apex_is_forgotten() {
    let mut sim = Sim::new();
    sim.put_airborne(
        0,
        Fx::ZERO,
        Fx::from_int(40),
        Fx::ZERO,
        Fx::from_ratio(1, 4),
    );
    sim.tick(inp(0, -127, 0));
    sim.ticks(60, inp(0, 0, 0));
    assert!(!sim.f().fast_fall);
}

// ---- Air acceleration and air friction (reference model) --------------------------------------

fn high_air(vx: Fx) -> Sim {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(500), vx, Fx::ZERO);
    sim
}

#[test]
fn air_acceleration_is_base_plus_additional_scaled_by_stick_tilt() {
    let p = Sim::new().content.fighters[0];
    // Full tilt: base + additional.
    let mut sim = high_air(Fx::ZERO);
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().vel.x, p.air_accel + p.air_accel_stick);

    // Half tilt: base + additional * tilt, and the speed asked for is tilt * max.
    let tilt = Input::axis(64);
    let mut sim = high_air(Fx::ZERO);
    sim.tick(inp(64, 0, 0));
    assert_eq!(sim.f().vel.x, p.air_accel + p.air_accel_stick * tilt);
    sim.ticks(60, inp(64, 0, 0));
    assert_eq!(
        sim.f().vel.x,
        tilt * p.air_speed,
        "half tilt should settle at half the max air speed"
    );
}

use sim_core::Input;

#[test]
fn full_tilt_reaches_the_maximum_air_speed_and_stops_there() {
    let p = Sim::new().content.fighters[0];
    let mut sim = high_air(Fx::ZERO);
    sim.ticks(60, inp(127, 0, 0));
    assert_eq!(sim.f().vel.x, p.air_speed);
    sim.ticks(10, inp(127, 0, 0));
    assert_eq!(
        sim.f().vel.x,
        p.air_speed,
        "must not exceed max air speed by drifting"
    );
}

#[test]
fn reversing_in_the_air_uses_the_same_acceleration_as_speeding_up() {
    let p = Sim::new().content.fighters[0];
    let accel = p.air_accel + p.air_accel_stick;
    let mut sim = high_air(p.air_speed);
    let mut last = sim.f().vel.x;
    for _ in 0..6 {
        sim.tick(inp(-127, 0, 0));
        assert_eq!(
            last - sim.f().vel.x,
            accel,
            "each frame of reversing changes speed by the full acceleration"
        );
        last = sim.f().vel.x;
    }
}

#[test]
fn with_no_stick_air_friction_slows_you_down_at_the_characters_own_rate() {
    for (chars, name) in [([0, 1, 0, 1], "duelist"), ([1, 0, 1, 0], "brawler")] {
        let mut sim = Sim::with_chars(chars);
        let p = sim.content.fighters[usize::from(chars[0])];
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(500), p.air_speed, Fx::ZERO);
        let mut last = sim.f().vel.x;
        for _ in 0..5 {
            sim.tick(inp(0, 0, 0));
            assert_eq!(last - sim.f().vel.x, p.air_friction, "{name}");
            last = sim.f().vel.x;
        }
    }
    let (duelist, brawler) = (
        Sim::new().content.fighters[0],
        Sim::new().content.fighters[1],
    );
    assert!(
        brawler.air_friction > duelist.air_friction,
        "the brawler sheds air speed faster"
    );
}

#[test]
fn momentum_above_max_air_speed_is_worn_down_by_friction_not_by_the_stick() {
    let p = Sim::new().content.fighters[0];
    let start = p.air_speed.mul_int(2);
    let mut sim = high_air(start);
    let mut last = start;
    for _ in 0..6 {
        sim.tick(inp(127, 0, 0)); // holding forward does not brake it
        assert_eq!(last - sim.f().vel.x, p.air_friction);
        last = sim.f().vel.x;
    }
    // Holding backward does brake it, at full air acceleration.
    sim.tick(inp(-127, 0, 0));
    assert_eq!(last - sim.f().vel.x, p.air_accel + p.air_accel_stick);
}

#[test]
fn easing_the_stick_back_slows_you_with_acceleration_not_friction() {
    let p = Sim::new().content.fighters[0];
    let tilt = Input::axis(64);
    let mut sim = high_air(p.air_speed);
    sim.tick(inp(64, 0, 0));
    assert_eq!(
        p.air_speed - sim.f().vel.x,
        p.air_accel + p.air_accel_stick * tilt
    );
    sim.ticks(40, inp(64, 0, 0));
    assert_eq!(sim.f().vel.x, tilt * p.air_speed);
}

#[test]
fn fast_fall_snaps_to_fast_fall_speed_on_the_frame_it_starts() {
    let mut sim = falling_sim();
    let p = sim.content.fighters[0];
    assert!(
        sim.f().vel.y > -p.fast_fall_speed,
        "should start slower than fast-fall speed"
    );
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall);
    assert_eq!(
        sim.f().vel.y,
        -p.fast_fall_speed,
        "speed must jump straight to fast-fall speed"
    );
}

#[test]
fn fast_fall_from_a_standstill_at_the_apex_is_immediate_too() {
    // Hard press exactly at the apex (vertical speed zero).
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().vel.y, -sim.content.fighters[0].fast_fall_speed);
}

// ---- Ledge grabs from the stage edge ---------------------------------------------------------------------

#[test]
fn walking_off_the_edge_grabs_only_once_the_body_is_clear_of_the_stage_and_snaps_a_short_way() {
    let mut sim = Sim::new();
    sim.stand(0, Fx::from_int(9), 1);
    for t in 0..120 {
        let before = sim.f().pos;
        sim.tick(inp(40, 0, 0));
        if sim.f().state == S::LedgeHang {
            let hw = sim.content.fighters[0].ecb_half_width;
            assert!(
                before.x >= Fx::from_int(11) + hw - fx(1, 5),
                "grabbed at tick {t} with the body still over the stage: {before:?}"
            );
            let snap = (sim.f().pos - before).length();
            assert!(snap < fx(13, 10), "snapped {snap:?}");
            return;
        }
    }
    panic!("walking off should grab the ledge");
}

#[test]
fn holding_down_walks_off_without_grabbing_the_ledge() {
    let mut sim = Sim::new();
    sim.stand(0, Fx::from_int(9), 1);
    for _ in 0..40 {
        sim.tick(inp(40, -127, 0));
        assert_ne!(sim.f().state, S::LedgeHang);
    }
    // Let go of down while falling beside the wall and the grab happens.
    let mut grabbed = false;
    for _ in 0..60 {
        sim.tick(inp(-60, 0, 0));
        grabbed |= sim.f().state == S::LedgeHang;
    }
    assert!(grabbed || sim.f().grounded() || sim.f().pos.y < Fx::from_int(-3));
}
