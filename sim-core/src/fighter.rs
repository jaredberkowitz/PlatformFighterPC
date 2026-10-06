//! Per-fighter state machine: ground movement (walk, dash, run, turn, crouch), jump squat, air
//! movement, air dodge, wavedash, shield drop, platform drop, helpless fall, and ledge options.
//!
//! Conventions: [`update`] runs once per frame per fighter in player-index order. A state entered
//! during a frame starts at `state_frame == 0` and is incremented at the start of the next update,
//! so "lasts N frames" means `state_frame >= N` ends it. All numbers come from [`FighterParams`].

use crate::collision;
use crate::content::{FighterParams, Ledge, Stage};
use crate::fixed::Fx;
use crate::input::{buttons, Input, STICK_DEADZONE, STICK_FLICK_FROM, STICK_THRESHOLD};
use crate::state::{Fighter, FighterState as S, NONE};
use crate::vec2::Vec2;

/// Frames of leniency for ground jump, shield-drop and tap-down presses.
const TAP_BUFFER: u8 = 3;
/// Frames of leniency for air jump and air dodge presses.
const AIR_ACTION_BUFFER: u8 = 2;
/// Frames in which a stick flick still counts as a dash input.
const FLICK_BUFFER: u8 = 2;

impl Fighter {
    pub fn held(&self, mask: u16) -> bool {
        self.history[0].pressed(mask)
    }

    /// True if `mask` went from released to pressed within the last `frames` frames (this one included).
    pub fn pressed_within(&self, mask: u16, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        (0..n).any(|i| {
            self.history[i].pressed(mask)
                && !self
                    .history
                    .get(i + 1)
                    .is_some_and(|older| older.pressed(mask))
        })
    }

    /// True if the stick was pushed down past the tap threshold within the last `frames` frames.
    pub fn down_within(&self, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        self.history[..n]
            .iter()
            .any(|i| i.stick_y <= -STICK_THRESHOLD)
    }

    /// True if the stick crossed down through the threshold within the last `frames` frames
    /// (a tap, as opposed to merely being held down).
    pub fn flicked_down(&self, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        (0..n).any(|i| {
            self.history[i].stick_y <= -STICK_THRESHOLD
                && match self.history.get(i + 1) {
                    Some(older) => older.stick_y > -STICK_THRESHOLD,
                    None => true,
                }
        })
    }

    /// Direction (-1 or 1) of a horizontal flick within the last `frames` frames, or 0. A flick is
    /// the stick crossing the threshold from below, and it must still be held past it now.
    pub fn flick_x(&self, frames: u8) -> i8 {
        let threshold = STICK_THRESHOLD.unsigned_abs();
        let current = self.history[0].stick_x;
        if current.unsigned_abs() < threshold {
            return 0;
        }
        let dir = current.signum();
        let past = |x: i8| x.signum() == dir && x.unsigned_abs() >= threshold;
        let from = STICK_FLICK_FROM.unsigned_abs();
        let started_low = |x: i8| x.signum() != dir || x.unsigned_abs() < from;
        let n = usize::from(frames).min(self.history.len());
        let crossed = (0..n).any(|i| {
            past(self.history[i].stick_x)
                && match self.history.get(i + 1) {
                    Some(older) => started_low(older.stick_x),
                    None => true,
                }
        });
        if crossed {
            dir
        } else {
            0
        }
    }
}

fn enter(f: &mut Fighter, state: S) {
    f.state = state;
    f.state_frame = 0;
}

fn approach(v: Fx, target: Fx, delta: Fx) -> Fx {
    if v < target {
        (v + delta).min(target)
    } else {
        (v - delta).max(target)
    }
}

/// Acceleration toward a ground speed target: gentle `run_decel` when slowing from extra speed
/// (left over from a dash or a landing), normal `ground_accel` otherwise.
fn ease_rate(vel: Fx, target: Fx, p: &FighterParams) -> Fx {
    let overspeed = vel.signum_int() == target.signum_int() && vel.abs() > target.abs();
    if overspeed {
        p.run_decel
    } else {
        p.ground_accel
    }
}

fn x_active(input: Input) -> bool {
    input.stick_x.unsigned_abs() >= STICK_DEADZONE.unsigned_abs()
}

fn on_pass_through(f: &Fighter, stage: &Stage) -> bool {
    collision::platform(stage, f.platform).is_some_and(|p| p.pass_through)
}

/// Advances one fighter by a frame. Returns the ledge it wants to grab, if any; the caller
/// resolves competing grabs in a fixed order (see `step`).
pub fn update(f: &mut Fighter, p: &FighterParams, stage: &Stage, input: Input) -> Option<u8> {
    f.history.rotate_right(1);
    f.history[0] = input;
    f.platform_ignore = f.platform_ignore.saturating_sub(1);
    f.ledge_cooldown = f.ledge_cooldown.saturating_sub(1);
    f.ledge_invuln = f.ledge_invuln.saturating_sub(1);
    f.state_frame = f.state_frame.saturating_add(1);

    match f.state {
        S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch => ground(f, p, stage),
        S::JumpSquat => jump_squat(f, p, stage),
        S::Airborne => return airborne(f, p, stage),
        S::Helpless => return helpless(f, p, stage),
        S::AirDodge => air_dodge(f, p, stage),
        S::Landing => landing(f, p, stage),
        S::WaveLand => wave_land(f, p, stage),
        S::Shield => shield(f, p, stage),
        S::ShieldDrop => shield_drop(f, p, stage),
        S::LedgeHang => ledge_hang(f, p, stage),
        S::LedgeGetUp | S::LedgeAttack => ledge_recover(f, p),
    }
    None
}

/// Puts a fighter into the special fall (what an up-special will do when it ends).
pub fn enter_helpless(f: &mut Fighter) {
    f.fast_fall = false;
    enter(f, S::Helpless);
}

// ---- Ground ----------------------------------------------------------------------------------

/// Moves along the ground, stopping at walls and stepping off into the air if the platform ends.
/// Returns false if the fighter left the ground.
fn slide_on_platform(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> bool {
    if collision::move_x(stage, p, &mut f.pos, f.vel.x) {
        f.vel.x = Fx::ZERO;
    }
    let Some(plat) = collision::platform(stage, f.platform) else {
        f.platform = NONE;
        enter(f, S::Airborne);
        return false;
    };
    if f.pos.x < plat.left || f.pos.x > plat.right {
        f.platform = NONE;
        f.vel.y = Fx::ZERO;
        enter(f, S::Airborne);
        return false;
    }
    true
}

fn start_dash(f: &mut Fighter, p: &FighterParams, dir: i8) {
    f.facing = dir;
    // Start slow and ramp up (see `dash_accel`), unless already carrying more speed that way.
    let carrying = f.vel.x.signum_int() == i32::from(dir) && f.vel.x.abs() > p.dash_initial_speed;
    if !carrying {
        f.vel.x = p.dash_initial_speed.mul_int(i32::from(dir));
    }
    enter(f, S::Dash);
}

fn start_platform_drop(f: &mut Fighter, p: &FighterParams) {
    f.platform = NONE;
    f.platform_ignore = p.platform_ignore_frames;
    f.vel.y = -p.shield_drop_speed;
    enter(f, S::Airborne);
}

fn ground(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    let input = f.history[0];
    if f.pressed_within(buttons::JUMP, TAP_BUFFER) {
        enter(f, S::JumpSquat);
        return;
    }
    if f.held(buttons::SHIELD) {
        enter(f, S::Shield);
        return;
    }
    if on_pass_through(f, stage) && f.flicked_down(TAP_BUFFER) {
        start_platform_drop(f, p);
        return;
    }

    // A flick starts a dash (and turns around instantly if it is against the facing).
    let flick = f.flick_x(FLICK_BUFFER);
    if flick != 0 && !(f.state == S::Dash && flick == f.facing) {
        start_dash(f, p, flick);
    }

    let dir = if input.stick_x > 0 { 1 } else { -1 };
    match f.state {
        S::Dash => {
            if !x_active(input) {
                // Releasing the stick cancels the dash: the speed carries into a slide on ground
                // friction, and a jump from here keeps it as air momentum.
                f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
                f.state = S::Idle;
            } else {
                let target = p.dash_speed.mul_int(i32::from(f.facing));
                f.vel.x = approach(f.vel.x, target, p.dash_accel);
                if f.state_frame >= u16::from(p.dash_frames) {
                    let holding = dir == f.facing;
                    f.state = if holding { S::Run } else { S::Idle };
                }
            }
        }
        S::Turn => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
            if f.state_frame >= u16::from(p.turn_frames) {
                f.state = S::Idle;
            }
        }
        _ if input.stick_y <= -STICK_THRESHOLD => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
            f.state = S::Crouch;
        }
        _ if x_active(input) => {
            if dir != f.facing {
                f.facing = dir;
                f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
                enter(f, S::Turn);
            } else if f.state == S::Run {
                // Speed left over from a dash eases down gradually instead of snapping to run speed.
                let target = input.stick_x_fx() * p.run_speed;
                let overspeed =
                    f.vel.x.signum_int() == target.signum_int() && f.vel.x.abs() > target.abs();
                let rate = if overspeed {
                    p.run_decel
                } else {
                    p.ground_accel
                };
                f.vel.x = approach(f.vel.x, target, rate);
            } else {
                // Walking: speed scales with how far the stick is pushed, up to the dash threshold.
                let tilt = i32::from(
                    input
                        .stick_x
                        .unsigned_abs()
                        .min(STICK_THRESHOLD.unsigned_abs()),
                );
                let target = Fx::from_ratio(tilt, i32::from(STICK_THRESHOLD)) * p.walk_speed;
                let target = target.mul_int(i32::from(dir));
                f.vel.x = approach(f.vel.x, target, ease_rate(f.vel.x, target, p));
                f.state = S::Walk;
            }
        }
        _ => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
            f.state = S::Idle;
        }
    }
    slide_on_platform(f, p, stage);
}

fn jump_squat(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    // No friction here: a run or dash jump keeps its momentum into the air.
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if f.state_frame >= u16::from(p.jump_squat_frames) {
        f.vel.y = if f.held(buttons::JUMP) {
            p.full_hop_velocity
        } else {
            p.short_hop_velocity
        };
        f.platform = NONE;
        f.fast_fall = false;
        enter(f, S::Airborne);
        // Wavedash input: a shield press during jump squat fires on the first airborne frame.
        if f.pressed_within(buttons::SHIELD, p.air_dodge_buffer) && !f.air_dodge_used {
            start_air_dodge(f, p, stage);
        }
    }
}

fn land(f: &mut Fighter, p: &FighterParams, stage: &Stage, platform: usize) {
    if let Some(plat) = stage.platforms.get(platform) {
        f.pos.y = plat.y;
    }
    f.vel.y = Fx::ZERO;
    f.platform = platform as i8;
    f.air_jumps_left = p.air_jumps;
    f.air_dodge_used = false;
    f.fast_fall = false;
    f.ledge_grab_count = 0;
}

fn enter_landing(f: &mut Fighter, lag: u8) {
    f.lag = lag;
    if lag == 0 {
        exit_landing(f);
    } else {
        enter(f, S::Landing);
    }
}

/// Leaves a landing. Holding the stick the way you are moving continues straight into a run,
/// so a jump keeps its momentum all the way through the landing.
fn exit_landing(f: &mut Fighter) {
    let input = f.history[0];
    let moving_right = f.vel.x > Fx::ZERO;
    let holding = input.stick_x.unsigned_abs() >= STICK_THRESHOLD.unsigned_abs()
        && (input.stick_x > 0) == moving_right
        && f.vel.x != Fx::ZERO;
    if holding {
        f.facing = if moving_right { 1 } else { -1 };
        enter(f, S::Run);
    } else {
        enter(f, S::Idle);
    }
}

// ---- Air -------------------------------------------------------------------------------------

/// Moves by `vel` with wall and ceiling collision. Returns the platform landed on, if any.
fn air_integrate(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> Option<usize> {
    if collision::move_x(stage, p, &mut f.pos, f.vel.x) {
        f.vel.x = Fx::ZERO;
    }
    let prev = f.pos;
    if f.vel.y > Fx::ZERO {
        if collision::move_up(stage, p, &mut f.pos, f.vel.y) {
            f.vel.y = Fx::ZERO;
        }
    } else {
        f.pos.y += f.vel.y;
    }
    let landing = collision::find_landing(stage, prev, f.pos, f.platform_ignore > 0);
    if landing.is_none() && collision::push_out(stage, p, &mut f.pos) {
        f.vel.x = Fx::ZERO;
    }
    landing
}

/// Air drift, gravity, movement and landing. Returns true if the fighter landed.
fn air_move(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> bool {
    let input = f.history[0];
    if x_active(input) {
        let tilt = input.stick_x_fx();
        let target = tilt * p.air_speed;
        let overspeed = f.vel.x.signum_int() == target.signum_int() && f.vel.x.abs() > target.abs();
        if overspeed {
            // Already faster than air speed (a run or dash jump): keep it; only drag bleeds it off.
            f.vel.x = approach(f.vel.x, target, p.air_friction);
        } else {
            let accel = p.air_accel + p.air_accel_stick * tilt.abs();
            f.vel.x = approach(f.vel.x, target, accel);
        }
    } else {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.air_friction);
    }
    if !f.fast_fall && f.vel.y <= Fx::ZERO && input.stick_y <= -STICK_THRESHOLD {
        f.fast_fall = true;
    }
    let terminal = if f.fast_fall {
        p.fast_fall_speed
    } else {
        p.max_fall_speed
    };
    f.vel.y = (f.vel.y - p.gravity).max(-terminal);

    match air_integrate(f, p, stage) {
        Some(i) => {
            land(f, p, stage, i);
            true
        }
        None => false,
    }
}

fn ledge_request(f: &Fighter, p: &FighterParams, stage: &Stage) -> Option<u8> {
    if f.vel.y <= Fx::ZERO && f.ledge_cooldown == 0 {
        collision::find_ledge(stage, f.pos, p).map(|i| i as u8)
    } else {
        None
    }
}

fn airborne(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> Option<u8> {
    if f.pressed_within(buttons::JUMP, AIR_ACTION_BUFFER) && f.air_jumps_left > 0 {
        f.vel.y = p.air_jump_velocity;
        f.air_jumps_left -= 1;
        f.fast_fall = false;
    }
    if f.pressed_within(buttons::SHIELD, AIR_ACTION_BUFFER) && !f.air_dodge_used {
        start_air_dodge(f, p, stage);
        return None;
    }
    if air_move(f, p, stage) {
        enter_landing(f, p.landing_lag);
        return None;
    }
    ledge_request(f, p, stage)
}

/// Special fall: drift and ledge grabs only. Landing costs extra lag.
fn helpless(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> Option<u8> {
    if air_move(f, p, stage) {
        enter_landing(f, p.helpless_landing_lag);
        return None;
    }
    ledge_request(f, p, stage)
}

fn start_air_dodge(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    let input = f.history[0];
    let neutral = input.stick_x.unsigned_abs() < STICK_DEADZONE.unsigned_abs()
        && input.stick_y.unsigned_abs() < STICK_DEADZONE.unsigned_abs();
    let dir = if neutral {
        Vec2::ZERO
    } else {
        let v = Vec2::new(input.stick_x_fx(), input.stick_y_fx());
        let len = v.length();
        Vec2::new(v.x / len, v.y / len)
    };
    f.dodge_dir = dir;
    f.air_dodge_used = true;
    f.fast_fall = false;
    f.vel = dir * p.air_dodge_speed;
    enter(f, S::AirDodge);

    // Ground assist: a downward dodge begun just above a surface counts as touching it.
    if dir.y <= -p.wavedash_min_down {
        if let Some((platform, dist)) =
            collision::surface_below(stage, f.pos, f.platform_ignore > 0)
        {
            if dist <= p.ground_assist_dist {
                start_waveland(f, p, stage, platform);
            }
        }
    }
}

fn start_waveland(f: &mut Fighter, p: &FighterParams, stage: &Stage, platform: usize) {
    land(f, p, stage, platform);
    // Slide speed scales smoothly with how horizontal the dodge was.
    f.vel.x = f.dodge_dir.x * p.waveland_speed;
    enter(f, S::WaveLand);
}

fn air_dodge(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel = f.vel * p.air_dodge_decay;
    if let Some(i) = air_integrate(f, p, stage) {
        if f.dodge_dir.y <= -p.wavedash_min_down {
            start_waveland(f, p, stage, i);
        } else {
            // Too horizontal to be a wavedash: a plain air dodge that happens to land.
            land(f, p, stage, i);
            enter_landing(f, p.landing_lag);
        }
        return;
    }
    if f.state_frame >= u16::from(p.air_dodge_frames) {
        enter(f, S::Airborne);
    }
}

fn landing(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    // Landing keeps momentum: holding the stick the way you are moving carries it through the
    // landing, and otherwise it bleeds off at the gentler `landing_friction`.
    let input = f.history[0];
    let carrying = x_active(input) && (input.stick_x > 0) == (f.vel.x > Fx::ZERO);
    if !carrying {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.landing_friction);
    }
    if slide_on_platform(f, p, stage) && f.state_frame >= u16::from(f.lag) {
        exit_landing(f);
    }
}

fn wave_land(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = f.vel.x * p.waveland_friction;
    if slide_on_platform(f, p, stage) && f.state_frame >= u16::from(p.waveland_lag) {
        enter(f, S::Idle);
    }
}

// ---- Shield ----------------------------------------------------------------------------------

fn shield(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if !f.held(buttons::SHIELD) {
        enter(f, S::Idle);
        return;
    }
    // Pressing down wins over jumping when standing on a pass-through platform.
    if on_pass_through(f, stage) && f.down_within(p.shield_drop_buffer) {
        f.platform = NONE;
        f.platform_ignore = p.platform_ignore_frames;
        f.vel = Vec2::new(Fx::ZERO, -p.shield_drop_speed);
        enter(f, S::ShieldDrop);
    } else if f.pressed_within(buttons::JUMP, TAP_BUFFER) {
        enter(f, S::JumpSquat);
    }
}

fn shield_drop(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    if air_move(f, p, stage) {
        enter_landing(f, p.landing_lag);
    } else if f.state_frame >= u16::from(p.shield_drop_recovery) {
        enter(f, S::Airborne);
    }
}

// ---- Ledges ----------------------------------------------------------------------------------

/// Puts a fighter on a ledge. Called by `step` after it has decided who wins a contested grab.
pub fn grab_ledge(f: &mut Fighter, p: &FighterParams, stage: &Stage, ledge: usize) {
    let Some(l) = stage.ledges.get(ledge) else {
        return;
    };
    f.ledge = ledge as i8;
    f.ledge_grab_count = f.ledge_grab_count.saturating_add(1);
    let decay = u32::from(p.ledge_invuln_decay) * u32::from(f.ledge_grab_count.saturating_sub(1));
    let invuln = u32::from(p.ledge_invuln_base).saturating_sub(decay);
    f.ledge_invuln = invuln
        .max(u32::from(p.ledge_invuln_floor))
        .min(u32::from(u8::MAX)) as u8;
    f.pos = collision::ledge_hang_pos(l, p);
    f.vel = Vec2::ZERO;
    f.facing = -l.side;
    // Grabbing a ledge refreshes airborne options.
    f.air_jumps_left = p.air_jumps;
    f.air_dodge_used = false;
    f.fast_fall = false;
    enter(f, S::LedgeHang);
}

/// Knocks the current occupant off a ledge that someone else just grabbed.
pub fn trump(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    let side = collision::ledge_side(stage, f.ledge);
    f.ledge = NONE;
    f.ledge_cooldown = p.ledge_regrab_cooldown;
    f.platform = NONE;
    f.vel = Vec2::new(p.ledge_trump_vx.mul_int(i32::from(side)), p.ledge_trump_vy);
    enter(f, S::Airborne);
}

fn release_ledge(f: &mut Fighter, p: &FighterParams) {
    f.ledge = NONE;
    f.ledge_cooldown = p.ledge_regrab_cooldown;
}

fn ledge_hang(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    let Some(&l) = usize::try_from(f.ledge)
        .ok()
        .and_then(|i| stage.ledges.get(i))
    else {
        release_ledge(f, p);
        enter(f, S::Airborne);
        return;
    };
    let input = f.history[0];
    f.pos = collision::ledge_hang_pos(&l, p);
    f.vel = Vec2::ZERO;

    let threshold = i32::from(STICK_THRESHOLD);
    // Positive means pushing toward the stage.
    let toward = i32::from(input.stick_x) * -i32::from(l.side);
    let up = i32::from(input.stick_y);

    if f.pressed_within(buttons::JUMP, AIR_ACTION_BUFFER) {
        release_ledge(f, p);
        f.vel = Vec2::new(
            p.ledge_jump_dx.mul_int(-i32::from(l.side)),
            p.ledge_jump_velocity,
        );
        enter(f, S::Airborne);
    } else if f.pressed_within(buttons::SHIELD, AIR_ACTION_BUFFER) {
        get_up(f, stage, &l, p.ledge_roll_dx, S::LedgeGetUp);
    } else if f.pressed_within(buttons::ATTACK, AIR_ACTION_BUFFER) {
        // The attack's hitboxes arrive with combat in Phase 3; for now it is the movement and timing.
        get_up(f, stage, &l, p.ledge_attack_dx, S::LedgeAttack);
    } else if up >= threshold || toward >= threshold {
        get_up(f, stage, &l, p.ledge_getup_dx, S::LedgeGetUp);
    } else if up <= -threshold || toward <= -threshold || f.state_frame >= p.ledge_hang_max {
        release_ledge(f, p);
        enter(f, S::Airborne);
    }
}

fn get_up(f: &mut Fighter, stage: &Stage, l: &Ledge, dx: Fx, state: S) {
    f.pos = Vec2::new(l.x - dx.mul_int(i32::from(l.side)), l.y);
    f.vel = Vec2::ZERO;
    f.ledge = NONE;
    f.platform = collision::standing_on(stage, f.pos);
    enter(
        f,
        if f.platform == NONE {
            S::Airborne
        } else {
            state
        },
    );
}

fn ledge_recover(f: &mut Fighter, p: &FighterParams) {
    let frames = if f.state == S::LedgeAttack {
        p.ledge_attack_frames
    } else {
        p.ledge_getup_frames
    };
    if f.state_frame >= u16::from(frames) {
        // Stable ground again: the next ledge grab gets full invincibility.
        f.ledge_grab_count = 0;
        enter(f, S::Idle);
    }
}
