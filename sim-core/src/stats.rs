//! The character-creator stats as arithmetic on fighter parameters (sim 142: moved here from `sim_content::recipe`, unchanged, so
//! that the run's stat items, R3, rebuild a fighter through the same code the creator's recipes use).
//!
//! Four stats where [`NEUTRAL`] (5) leaves a parameter set as it is: **size** (the body: hurtbox, reach, weight, run, jump, fall),
//! **speed** (ground and air speed and responsiveness; a little less weight), **jump** (jump heights) and **weight** (how hard to
//! launch; a little faster fall). Integer arithmetic only, so the same stats always give the same parameters. The creator clamps its
//! stats to 1..9; this function does not, so a run's stat items may go past 9.

use crate::content::FighterParams;
use crate::fixed::Fx;

/// The stat value that leaves the parameters unchanged.
pub const NEUTRAL: i32 = 5;

/// `v` times `percent` / 100.
fn scale(v: Fx, percent: i32) -> Fx {
    v * Fx::from_ratio(percent, 100)
}

/// Body size as a percent of the archetype's: 68 at size 1, 100 at size 5, 132 at size 9.
pub fn size_percent(size: i32) -> i32 {
    60 + 8 * size
}

/// `p` changed by the four stats (each [`NEUTRAL`] for no change).
pub fn apply(mut p: FighterParams, size: i32, speed: i32, jump: i32, weight: i32) -> FighterParams {
    let size = size_percent(size);
    let neutral = NEUTRAL;

    // Body: the collision box and where a ledge is held scale with size.
    for v in [
        &mut p.ecb_half_width,
        &mut p.ecb_height,
        &mut p.ecb_side_height,
        &mut p.ledge_reach_x,
        &mut p.ledge_min_drop,
        &mut p.ledge_reach_down,
        &mut p.ledge_hang_dx,
        &mut p.ledge_hang_dy,
    ] {
        *v = scale(*v, size);
    }

    // Attacks are the size of the body: a big fighter's hitboxes are bigger and reach further.
    p.hitbox_scale = scale(p.hitbox_scale, size);

    // Ground and air speed: the speed stat, and bigger bodies are slower.
    let bulk_slow = 100 - (size - 100) * 35 / 100;
    let pace = (80 + 4 * speed) * bulk_slow / 100;
    for v in [
        &mut p.walk_speed,
        &mut p.run_speed,
        &mut p.dash_speed,
        &mut p.dash_initial_speed,
        &mut p.air_speed,
        &mut p.jump_speed_x,
        &mut p.jump_speed_x_max,
        &mut p.air_dodge_speed,
        &mut p.waveland_speed,
    ] {
        *v = scale(*v, pace);
    }
    // Responsiveness follows the speed stat alone.
    let snap = 80 + 4 * speed;
    for v in [
        &mut p.dash_accel,
        &mut p.dash_brake,
        &mut p.walk_accel,
        &mut p.walk_accel_stick,
        &mut p.run_accel,
        &mut p.run_accel_stick,
        &mut p.ground_friction,
        &mut p.run_decel,
        &mut p.landing_friction,
        &mut p.air_accel,
        &mut p.air_accel_stick,
    ] {
        *v = scale(*v, snap);
    }

    // Jumps: the jump stat raises them, a big body lowers them.
    let lift = (100 + (jump - neutral) * 3) * (100 - (size - 100) * 20 / 100) / 100;
    for v in [
        &mut p.full_hop_velocity,
        &mut p.hop_burst_velocity,
        &mut p.short_hop_velocity,
        &mut p.air_jump_velocity,
        &mut p.ledge_jump_velocity,
    ] {
        *v = scale(*v, lift);
    }

    // Falling: big and heavy bodies drop faster.
    let pull = 100 + (size - 100) * 20 / 100 + (weight - neutral) * 2;
    p.gravity = scale(p.gravity, pull);
    let fall = 100 + (size - 100) * 15 / 100 + (weight - neutral) * 3;
    p.max_fall_speed = scale(p.max_fall_speed, fall);
    p.fast_fall_speed = scale(p.fast_fall_speed, fall);

    // Weight: size and the weight stat add to it, the speed stat takes a little off.
    let mass = (100 + (size - 100) * 120 / 100) * (100 + (weight - neutral) * 8) / 100
        * (100 - (speed - neutral) * 3)
        / 100;
    p.weight = scale(p.weight, mass);
    p
}
