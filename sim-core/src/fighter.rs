//! Per-fighter state machine: ground movement (walk, dash, run, turn, crouch), jump squat, air
//! movement, air dodge, wavedash, shield drop, platform drop, helpless fall, and ledge options.
//!
//! Conventions: [`update`] runs once per frame per fighter in player-index order. A state entered
//! during a frame starts at `state_frame == 0` and is incremented at the start of the next update,
//! so "lasts N frames" means `state_frame >= N` ends it. All numbers come from [`FighterParams`].

use crate::collision;
use crate::content::{FighterParams, Ledge, Ruleset, Stage};
use crate::fixed::Fx;
use crate::input::{buttons, Input, STICK_DEADZONE, STICK_DOWN, STICK_FLICK_FROM, STICK_THRESHOLD};
use crate::moves::{Motion, MoveId, Weapon};
use crate::scripting;
use crate::state::{Fighter, FighterState as S, NONE};
use crate::trig::{self, Angle};
use crate::vec2::Vec2;

/// Frames of leniency for ground jump, shield-drop and tap-down presses.
const TAP_BUFFER: u8 = 3;
/// Frames of leniency for air jump and air dodge presses.
const AIR_ACTION_BUFFER: u8 = 2;
/// Frames in which a stick flick still counts as a dash input.
const FLICK_BUFFER: u8 = 2;
/// Frames in which a hard down press still triggers a fast fall. Long enough that a press made
/// just before the apex of a jump still works once the fighter starts falling.
const FAST_FALL_BUFFER: u8 = 10;

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
        self.history[..n].iter().any(|i| i.stick_y <= -STICK_DOWN)
    }

    /// True if the stick crossed down through the threshold within the last `frames` frames
    /// (a tap, as opposed to merely being held down).
    pub fn flicked_down(&self, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        (0..n).any(|i| {
            self.history[i].stick_y <= -STICK_DOWN
                && match self.history.get(i + 1) {
                    Some(older) => older.stick_y > -STICK_DOWN,
                    None => true,
                }
        })
    }

    /// True if the stick was pressed down *hard and fast* within the last `frames` frames: it reached
    /// the full threshold coming from near neutral. Merely holding down, or rolling slowly down to it,
    /// does not count. This is the fast-fall input: a flick or hard press on a stick, a double tap
    /// on a keyboard.
    pub fn hard_down(&self, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        (0..n).any(|i| {
            self.history[i].stick_y <= -STICK_THRESHOLD
                && match self.history.get(i + 1) {
                    Some(older) => older.stick_y > -STICK_FLICK_FROM,
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
    // Script variables belong to one attack (and the moves it switches to), so a new attack starts clean.
    if state == S::Attack && f.state != S::Attack {
        f.vars = [0; crate::FIGHTER_VARS];
    }
    f.state = state;
    f.state_frame = 0;
    // The fast opening of a full hop only survives while airborne or attacking in the air.
    if !matches!(state, S::Airborne | S::Attack) {
        f.hop_boost = 0;
    }
    if state != S::Attack {
        f.charge = 0;
    }
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
pub fn update(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
    input: Input,
) -> Option<u8> {
    f.history.rotate_right(1);
    f.history[0] = input;

    // Hitlag freezes everything about the fighter except its input history. A struck fighter can still
    // survival-DI, and its launch happens on the frame hitlag ends.
    if f.hitlag > 0 {
        if f.launch_pending {
            sdi(f, p, stage, rules);
        }
        f.hitlag -= 1;
        if f.hitlag == 0 && f.launch_pending {
            apply_launch(f, rules);
        }
        return None;
    }

    f.platform_ignore = f.platform_ignore.saturating_sub(1);
    f.grab_immune = f.grab_immune.saturating_sub(1);
    f.ledge_cooldown = f.ledge_cooldown.saturating_sub(1);
    f.ledge_invuln = f.ledge_invuln.saturating_sub(1);
    f.invuln = f.invuln.saturating_sub(1);
    f.state_frame = f.state_frame.saturating_add(1);
    // The shield refills whenever it is not up (a broken shield is restored when the stun ends).
    if !matches!(f.state, S::Shield | S::ShieldBreak) {
        f.shield_hp = (f.shield_hp + rules.shield_regen).min(rules.shield_max);
    }

    match f.state {
        S::Attack => return attack(f, p, weapon, stage, rules),
        S::Hitstun => hitstun(f, p, stage, rules),
        S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch => ground(f, p, weapon, stage),
        S::JumpSquat => jump_squat(f, p, stage),
        S::Airborne => return airborne(f, p, weapon, stage),
        S::Helpless => return helpless(f, p, stage),
        S::AirDodge => air_dodge(f, p, stage),
        S::Landing => landing(f, p, stage),
        S::WaveLand => wave_land(f, p, stage),
        S::Shield => shield(f, p, stage, rules),
        S::Roll => roll(f, p, stage),
        S::SpotDodge => spot_dodge(f, p, stage),
        S::ShieldBreak => shield_break(f, p, stage, rules),
        S::Knockdown => knockdown(f, p, weapon, stage, rules),
        S::GetUp => get_up_stand(f, p, stage, rules),
        S::Grabbing => grabbing(f),
        // Being held: the holder pins this fighter in place (see `grab::update`).
        S::Grabbed => {}
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

fn ground(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage) {
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
    if f.pressed_within(buttons::GRAB, ATTACK_BUFFER) {
        let id = if matches!(f.state, S::Dash | S::Run) {
            MoveId::DashGrab
        } else {
            MoveId::Grab
        };
        begin_attack(f, id);
        return;
    }
    if f.pressed_within(buttons::SPECIAL, ATTACK_BUFFER) && start_special(f, weapon) {
        return;
    }
    if f.pressed_within(buttons::ATTACK, ATTACK_BUFFER) {
        start_ground_attack(f, p);
        return;
    }

    // A flick starts a dash (and turns around instantly if it is against the facing). A run is
    // committed: flicking the other way skids to a stop and turns instead of dash dancing, and a
    // flick the same way changes nothing.
    let flick = f.flick_x(FLICK_BUFFER);
    if flick != 0 {
        if f.state == S::Run {
            if flick != f.facing {
                f.facing = flick;
                enter(f, S::Turn);
            }
        } else if !(f.state == S::Dash && flick == f.facing) {
            start_dash(f, p, flick);
        }
    }

    let dir = if input.stick_x > 0 { 1 } else { -1 };
    match f.state {
        S::Dash => {
            if !x_active(input) {
                // Releasing the stick cancels the dash: the speed bleeds off quickly on
                // braking (`dash_brake`), and a jump from here keeps it as air momentum.
                f.vel.x = approach(f.vel.x, Fx::ZERO, p.dash_brake);
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
        _ if input.stick_y <= -STICK_DOWN => {
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
        f.platform = NONE;
        f.fast_fall = false;
        enter(f, S::Airborne);
        if f.held(buttons::JUMP) && p.hop_burst_frames > 0 {
            // Full hop: a fast opening, then the arc (see `air_move`).
            f.vel.y = p.hop_burst_velocity;
            f.hop_boost = p.hop_burst_frames;
        } else {
            f.vel.y = if f.held(buttons::JUMP) {
                p.full_hop_velocity
            } else {
                p.short_hop_velocity
            };
        }
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
    f.hop_boost = 0;
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
    let moved = move_by(f, p, stage, f.vel);
    if moved.wall {
        f.vel.x = Fx::ZERO;
    }
    if moved.ceiling {
        f.vel.y = Fx::ZERO;
    }
    moved.landing
}

struct Moved {
    landing: Option<usize>,
    wall: bool,
    ceiling: bool,
}

/// Moves the fighter by `v` with wall, ceiling and landing collision, reporting what it touched.
fn move_by(f: &mut Fighter, p: &FighterParams, stage: &Stage, v: Vec2) -> Moved {
    let mut wall = collision::move_x(stage, p, &mut f.pos, v.x);
    let prev = f.pos;
    let mut ceiling = false;
    if v.y > Fx::ZERO {
        ceiling = collision::move_up(stage, p, &mut f.pos, v.y);
    } else {
        f.pos.y += v.y;
    }
    let landing = collision::find_landing(stage, prev, f.pos, f.platform_ignore > 0);
    if landing.is_none() && collision::push_out(stage, p, &mut f.pos) {
        wall = true;
    }
    Moved {
        landing,
        wall,
        ceiling,
    }
}

/// Air drift, gravity, movement and landing. Returns true if the fighter landed.
fn air_move(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> bool {
    let input = f.history[0];
    if x_active(input) {
        // Air acceleration (reference model): a base value plus an additional value scaled by stick
        // tilt, the same whether speeding up, slowing down or reversing. Tilt also scales the speed
        // the stick is asking for. Air friction is only for no input, and for momentum above the
        // maximum air speed (a run or dash jump), which it bleeds off slowly.
        let tilt = input.stick_x_fx();
        let target = tilt * p.air_speed;
        let over_max = f.vel.x.signum_int() == tilt.signum_int() && f.vel.x.abs() > p.air_speed;
        if over_max {
            let cap = p.air_speed.mul_int(tilt.signum_int());
            f.vel.x = approach(f.vel.x, cap, p.air_friction);
        } else {
            let accel = p.air_accel + p.air_accel_stick * tilt.abs();
            f.vel.x = approach(f.vel.x, target, accel);
        }
    } else {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.air_friction);
    }
    if !f.fast_fall && f.vel.y <= Fx::ZERO && f.hard_down(FAST_FALL_BUFFER) {
        f.fast_fall = true;
        // Fast fall snaps straight to fast-fall speed (it is not a gradual speed-up), so it is
        // unmistakable when it happens.
        f.vel.y = f.vel.y.min(-p.fast_fall_speed);
    }
    let terminal = if f.fast_fall {
        p.fast_fall_speed
    } else {
        p.max_fall_speed
    };
    // Fast opening of a full hop: constant speed, no gravity. After its last frame the hop
    // continues as a normal arc.
    let mut opening_ends = false;
    if f.hop_boost > 0 {
        f.hop_boost -= 1;
        opening_ends = f.hop_boost == 0;
    } else {
        f.vel.y = (f.vel.y - p.gravity).max(-terminal);
    }

    match air_integrate(f, p, stage) {
        Some(i) => {
            land(f, p, stage, i);
            true
        }
        None => {
            if opening_ends && f.vel.y > Fx::ZERO {
                f.vel.y = p.full_hop_velocity;
            }
            false
        }
    }
}

/// Holding the stick down declines a ledge grab (walk off the stage, or fast fall past it, without catching it).
fn declines_ledge(f: &Fighter) -> bool {
    f.history[0].stick_y <= -STICK_DOWN
}

fn ledge_request(f: &Fighter, p: &FighterParams, stage: &Stage) -> Option<u8> {
    if f.vel.y <= Fx::ZERO && f.ledge_cooldown == 0 && !declines_ledge(f) {
        collision::find_ledge(stage, f.pos, p).map(|i| i as u8)
    } else {
        None
    }
}

fn airborne(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage) -> Option<u8> {
    if f.pressed_within(buttons::JUMP, AIR_ACTION_BUFFER) && f.air_jumps_left > 0 {
        f.vel.y = p.air_jump_velocity;
        f.hop_boost = 0;
        f.air_jumps_left -= 1;
        f.fast_fall = false;
    }
    if f.pressed_within(buttons::SHIELD, AIR_ACTION_BUFFER) && !f.air_dodge_used {
        start_air_dodge(f, p, stage);
        return None;
    }
    if f.pressed_within(buttons::SPECIAL, ATTACK_BUFFER) && start_special(f, weapon) {
        return None;
    }
    if f.pressed_within(buttons::ATTACK, ATTACK_BUFFER) {
        start_air_attack(f);
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

fn shield(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    f.shield_hp -= rules.shield_deplete;
    if f.shield_hp <= Fx::ZERO {
        break_shield(f, rules);
        return;
    }
    // After blocking a hit the shield stays up and the fighter cannot act until the stun runs out.
    if f.shield_stun > 0 {
        f.shield_stun -= 1;
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
    } else if f.pressed_within(buttons::ATTACK | buttons::GRAB, ATTACK_BUFFER) {
        // Attack or grab out of the shield is a shield grab.
        begin_attack(f, MoveId::Grab);
    } else if f.flick_x(TAP_BUFFER) != 0 {
        // A flick sideways rolls that way.
        f.dodge_dir = Vec2::new(Fx::from_int(i32::from(f.flick_x(TAP_BUFFER))), Fx::ZERO);
        enter(f, S::Roll);
    } else if !on_pass_through(f, stage) && f.hard_down(TAP_BUFFER) {
        enter(f, S::SpotDodge);
    }
}

/// True if `frame` is within `start..=end`.
fn within(frame: u16, start: u8, end: u8) -> bool {
    frame >= u16::from(start) && frame <= u16::from(end)
}

/// A roll out of a shield: intangible for part of it, moving along the ground for part of it.
fn roll(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    if within(
        f.state_frame,
        p.roll_intangible_start,
        p.roll_intangible_end,
    ) {
        f.invuln = f.invuln.max(1);
    }
    let dir = f.dodge_dir.x.signum_int();
    if within(f.state_frame, p.roll_move_start, p.roll_move_end) {
        f.vel.x = p.roll_speed.mul_int(dir);
    } else {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    }
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if f.state_frame >= u16::from(p.roll_frames) {
        enter(f, S::Idle);
    }
}

/// A dodge in place out of a shield.
fn spot_dodge(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    if within(
        f.state_frame,
        p.spot_intangible_start,
        p.spot_intangible_end,
    ) {
        f.invuln = f.invuln.max(1);
    }
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if f.state_frame >= u16::from(p.spot_dodge_frames) {
        enter(f, S::Idle);
    }
}

/// The shield ran out: the fighter is launched into a hop and then stunned for a long time, less long the
/// more damage it has. Mashing buttons shortens the stun.
pub fn break_shield(f: &mut Fighter, rules: &Ruleset) {
    let percent = f.percent.floor_int().clamp(0, 999);
    let frames =
        i32::from(rules.shield_break_frames) - i32::from(rules.shield_break_per_percent) * percent;
    f.hitstun = frames.max(i32::from(rules.shield_break_min)) as u16;
    f.shield_stun = 0;
    f.vel = Vec2::new(Fx::ZERO, rules.shield_break_hop);
    f.platform = NONE;
    f.fast_fall = false;
    enter(f, S::ShieldBreak);
}

fn shield_break(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    let pressed = [
        buttons::ATTACK,
        buttons::JUMP,
        buttons::SHIELD,
        buttons::SPECIAL,
    ]
    .iter()
    .any(|b| f.pressed_within(*b, 1));
    let mash = if pressed {
        u16::from(rules.shield_mash_frames)
    } else {
        0
    };
    f.hitstun = f.hitstun.saturating_sub(1 + mash);

    if f.grounded() {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
        if !slide_on_platform(f, p, stage) {
            return;
        }
    } else {
        // Falling with no control until it lands.
        f.vel.y = (f.vel.y - p.gravity).max(-p.max_fall_speed);
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.air_friction);
        if let Some(i) = air_integrate(f, p, stage) {
            land(f, p, stage, i);
            f.vel.x = Fx::ZERO;
        }
    }
    if f.hitstun == 0 {
        restore_shield(f, rules);
        if f.grounded() {
            enter(f, S::Idle);
        } else {
            enter(f, S::Airborne);
        }
    }
}

/// A shield that broke comes back with part of its health.
pub fn restore_shield(f: &mut Fighter, rules: &Ruleset) {
    f.shield_hp = rules.shield_max * Fx::from_ratio(i32::from(rules.shield_restore_percent), 100);
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
        if f.state == S::LedgeAttack {
            f.move_id = MoveId::LedgeAttack as u8;
            f.hit_mask = 0;
        }
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

/// Lying on the ground after a hard landing. After a short while the fighter chooses a get-up: attack, roll
/// sideways, or stand (stick up, jump, or just waiting).
fn knockdown(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage, rules: &Ruleset) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if f.state_frame < u16::from(rules.knockdown_lag) {
        return;
    }
    let flick = f.flick_x(TAP_BUFFER);
    if f.pressed_within(buttons::ATTACK, ATTACK_BUFFER) {
        begin_attack(f, MoveId::GetUpAttack);
        f.invuln = f
            .invuln
            .max(weapon.get(MoveId::GetUpAttack as u8).intangible);
    } else if flick != 0 {
        f.dodge_dir = Vec2::new(Fx::from_int(i32::from(flick)), Fx::ZERO);
        enter(f, S::Roll);
    } else if f.history[0].stick_y >= STICK_DOWN
        || f.pressed_within(buttons::JUMP, TAP_BUFFER)
        || f.state_frame >= u16::from(rules.knockdown_max)
    {
        enter(f, S::GetUp);
    }
}

/// Standing up: intangible at first, then free.
fn get_up_stand(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    if f.state_frame <= u16::from(rules.getup_intangible) {
        f.invuln = f.invuln.max(1);
    }
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if f.state_frame >= u16::from(rules.getup_frames) {
        enter(f, S::Idle);
    }
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

// ---- Combat ------------------------------------------------------------------------------------

/// Frames of leniency for an attack press.
const ATTACK_BUFFER: u8 = 2;
/// A direction flick this recent when attack is pressed makes it a smash attack; a stick that was
/// already held makes it a tilt.
const SMASH_FLICK_BUFFER: u8 = 4;

impl Fighter {
    /// Like [`Fighter::hard_down`], but upward.
    pub fn hard_up(&self, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        (0..n).any(|i| {
            self.history[i].stick_y >= STICK_THRESHOLD
                && match self.history.get(i + 1) {
                    Some(older) => older.stick_y < STICK_FLICK_FROM,
                    None => true,
                }
        })
    }

    /// +1 for a hard up press, -1 for a hard down press within `frames`, else 0.
    pub fn flick_y(&self, frames: u8) -> i8 {
        if self.hard_up(frames) {
            1
        } else if self.hard_down(frames) {
            -1
        } else {
            0
        }
    }
}

/// Holding a fighter: a stick direction throws it that way, attack pummels it.
fn grabbing(f: &mut Fighter) {
    let input = f.history[0];
    let t = STICK_THRESHOLD;
    let throw = if input.stick_y >= t {
        Some(MoveId::UThrow)
    } else if input.stick_y <= -t {
        Some(MoveId::DThrow)
    } else if input.stick_x.unsigned_abs() >= t.unsigned_abs() {
        if i32::from(input.stick_x.signum()) == i32::from(f.facing) {
            Some(MoveId::FThrow)
        } else {
            Some(MoveId::BThrow)
        }
    } else {
        None
    };
    if let Some(id) = throw {
        begin_attack(f, id);
    } else if f.pressed_within(buttons::ATTACK, ATTACK_BUFFER) {
        begin_attack(f, MoveId::Pummel);
    }
}

/// The fighter has caught `victim`.
pub fn become_holder(f: &mut Fighter, victim: usize) {
    f.grab_with = victim as i8;
    f.vel.x = Fx::ZERO;
    enter(f, S::Grabbing);
}

/// The fighter is caught by `holder` and faces it for `timer` frames unless it mashes free.
pub fn become_held(f: &mut Fighter, holder: usize, facing: i8, timer: u16) {
    f.grab_with = holder as i8;
    f.grab_timer = timer;
    f.facing = facing;
    f.vel = Vec2::ZERO;
    f.shield_stun = 0;
    f.fast_fall = false;
    enter(f, S::Grabbed);
}

/// A holder whose catch got away: it is stuck in the release for `lag` frames.
pub fn free_holder(f: &mut Fighter, lag: u8) {
    f.grab_with = NONE;
    if f.grounded() {
        enter_landing(f, lag);
    } else {
        enter(f, S::Airborne);
    }
}

/// A fighter that was held goes back to standing, and cannot be grabbed again for a moment.
pub fn free_held(f: &mut Fighter, immunity: u8) {
    f.grab_with = NONE;
    f.grab_timer = 0;
    f.grab_immune = immunity;
    if f.grounded() {
        enter(f, S::Idle);
    } else {
        enter(f, S::Airborne);
    }
}

fn begin_attack(f: &mut Fighter, id: MoveId) {
    f.move_id = id as u8;
    f.hit_mask = 0;
    enter(f, S::Attack);
    f.charge = 0;
}

fn start_ground_attack(f: &mut Fighter, p: &FighterParams) {
    let input = f.history[0];
    // The strong-attack button is a flick made in advance: it turns a direction into a smash attack.
    let strong = f.held(buttons::STRONG);
    // A dash attack comes out of a dash or run, and also just after letting go of the stick, while the fighter is
    // still sliding at dash speed (releasing the stick cancels a dash, which must not turn the attack into a jab).
    let sliding_fast = f.vel.x.abs() > p.walk_speed * Fx::from_ratio(12, 10)
        && !x_active(input)
        && input.stick_y.unsigned_abs() < STICK_DOWN.unsigned_abs()
        && f.vel.x.signum_int() == i32::from(f.facing);
    let id = if matches!(f.state, S::Dash | S::Run) || sliding_fast {
        MoveId::DashAttack
    } else if input.stick_y >= STICK_DOWN {
        if strong || f.hard_up(SMASH_FLICK_BUFFER) {
            MoveId::USmash
        } else {
            MoveId::UTilt
        }
    } else if input.stick_y <= -STICK_DOWN {
        if strong || f.hard_down(SMASH_FLICK_BUFFER) {
            MoveId::DSmash
        } else {
            MoveId::DTilt
        }
    } else if x_active(input) {
        f.facing = if input.stick_x > 0 { 1 } else { -1 };
        if strong || f.flick_x(SMASH_FLICK_BUFFER) != 0 {
            MoveId::FSmash
        } else {
            MoveId::FTilt
        }
    } else {
        MoveId::Jab
    };
    begin_attack(f, id);
}

fn start_air_attack(f: &mut Fighter) {
    let input = f.history[0];
    let id = if input.stick_y >= STICK_DOWN {
        MoveId::UAir
    } else if input.stick_y <= -STICK_DOWN {
        MoveId::DAir
    } else if x_active(input) {
        if (input.stick_x > 0) == (f.facing > 0) {
            MoveId::FAir
        } else {
            MoveId::BAir
        }
    } else {
        MoveId::NAir
    };
    begin_attack(f, id);
}

/// Advances a move one frame. Returns the ledge it wants to grab, if the move can grab ledges.
fn attack(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
) -> Option<u8> {
    let mv = weapon.get(f.move_id);
    let id = MoveId::from_index(f.move_id);

    // A smash attack holds on its charge frame while the button stays held.
    let charging = mv.charge_at.is_some_and(|c| u16::from(c) == f.state_frame)
        && f.charge < rules.charge_frames
        && f.held(buttons::ATTACK);
    if charging {
        f.charge += 1;
        f.state_frame -= 1;
    }

    // A multi-hit move lets its hits land again at a steady rhythm.
    if let Some((start, every)) = mv.rehit {
        let start = u16::from(start);
        if f.state_frame >= start && (f.state_frame - start) % u16::from(every.max(1)) == 0 {
            f.hit_mask = 0;
        }
    }

    // A move can fire a projectile on one frame (the step applies the request).
    if let Some(spec) = mv.projectile {
        if f.state_frame == u16::from(spec.frame) {
            f.spawn_request = true;
        }
    }

    // The move's script steers the move: motion, branching into another move, projectiles, ending early.
    let mut script_motion = None;
    if let Some(program) = &mv.script {
        let effects = scripting::run_fighter(program, f);
        if let Some(next) = effects.goto {
            begin_attack(f, MoveId::from_index(next));
            return None;
        }
        // Holding a frame is capped by the same limit as charging, so a script cannot hold a fighter forever.
        if effects.stall && f.charge < rules.charge_frames && f.state_frame > 0 {
            f.charge += 1;
            f.state_frame -= 1;
        }
        if effects.end {
            f.state_frame = u16::from(mv.total_frames);
        }
        script_motion = effects.motion;
    }

    // Scripted motion (a rising special) overrides normal physics while it is active.
    let frame = f.state_frame;
    let motion = script_motion
        .map(|(vx, vy)| Motion {
            start: 0,
            end: u8::MAX,
            vx,
            vy,
        })
        .or_else(|| {
            mv.motion
                .iter()
                .find(|m| frame >= u16::from(m.start) && frame <= u16::from(m.end))
                .copied()
        });
    if let Some(m) = motion {
        // A motion along the ground follows the ground: it can leave it by going up, but never sinks into it.
        let vy = if f.grounded() {
            m.vy.max(Fx::ZERO)
        } else {
            m.vy
        };
        f.vel = Vec2::new(m.vx.mul_int(i32::from(f.facing)), vy);
        f.hop_boost = 0;
        // A dash along the ground is not a landing, however flat it is.
        let on_ground = f.grounded() && vy <= Fx::ZERO;
        if vy > Fx::ZERO {
            f.platform = NONE;
        }
        let moved = move_by(f, p, stage, f.vel);
        if moved.wall {
            f.vel.x = Fx::ZERO;
        }
        if moved.ceiling {
            f.vel.y = Fx::ZERO;
        }
        if let (Some(platform), false) = (moved.landing, on_ground) {
            land(f, p, stage, platform);
            enter_landing(f, mv.landing_lag);
            return None;
        }
        // A dash along the ground that carries the fighter past the edge leaves the ground.
        if let Some(plat) = collision::platform(stage, f.platform) {
            if f.pos.x < plat.left || f.pos.x > plat.right {
                f.platform = NONE;
            }
        }
    } else if id.is_aerial() || (id.is_special() && !f.grounded()) {
        if air_move(f, p, stage) {
            // Landing early or late in the move autocancels: only the normal landing lag.
            let clean =
                frame < u16::from(mv.autocancel_before) || frame >= u16::from(mv.autocancel_after);
            enter_landing(f, if clean { p.landing_lag } else { mv.landing_lag });
            return None;
        }
    } else {
        // A dash attack carries its speed a long way; other ground moves stop on normal friction.
        let friction = if id == MoveId::DashAttack {
            p.run_decel
        } else {
            p.ground_friction
        };
        f.vel.x = approach(f.vel.x, Fx::ZERO, friction);
        if !slide_on_platform(f, p, stage) {
            return None;
        }
    }

    if f.state_frame >= u16::from(mv.total_frames) {
        if mv.turns_around {
            f.facing = -f.facing;
        }
        // A pummel hands back to holding the fighter, if it is still held.
        if MoveId::from_index(f.move_id) == MoveId::Pummel && f.grab_with != NONE {
            enter(f, S::Grabbing);
            return None;
        }
        // A jab continues into its next hit if attack was pressed shortly before the end.
        if let Some(next) = mv.next {
            if f.grounded() && f.pressed_within(buttons::ATTACK, mv.next_window) {
                begin_attack(f, MoveId::from_index(next));
                return None;
            }
        }
        if f.grounded() {
            enter(f, S::Idle);
        } else if mv.helpless_after {
            enter_helpless(f);
        } else {
            enter(f, S::Airborne);
        }
    }

    // Up specials can grab a ledge in mid-move, rising or not, so a recovery that reaches it is forgiving.
    if mv.grabs_ledge && f.ledge_cooldown == 0 && !declines_ledge(f) {
        return collision::find_ledge(stage, f.pos, p).map(|i| i as u8);
    }
    None
}

/// Starts a special move chosen by the stick, if the weapon has one there. Returns whether it started.
fn start_special(f: &mut Fighter, weapon: &Weapon) -> bool {
    let input = f.history[0];
    let id = if input.stick_y >= STICK_DOWN {
        MoveId::UpSpecial
    } else if input.stick_y <= -STICK_DOWN {
        MoveId::DownSpecial
    } else if x_active(input) {
        MoveId::SideSpecial
    } else {
        MoveId::NSpecial
    };
    let mv = weapon.get(id as u8);
    if mv.is_empty() {
        return false;
    }
    if id == MoveId::SideSpecial {
        f.facing = if input.stick_x > 0 { 1 } else { -1 };
    }
    begin_attack(f, id);
    f.invuln = f.invuln.max(mv.intangible);
    true
}

/// Launch physics while stunned. Gravity acts normally; the launch speed decays on top of it.
fn hitstun(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    f.hitstun = f.hitstun.saturating_sub(1);
    let speed = f.kb_vel.length();
    f.kb_vel = if speed > rules.knockback_decay {
        f.kb_vel * ((speed - rules.knockback_decay) / speed)
    } else {
        Vec2::ZERO
    };

    if f.grounded() {
        // A grounded hit slides the fighter along the floor.
        f.vel = Vec2::new(f.kb_vel.x, Fx::ZERO);
        if collision::move_x(stage, p, &mut f.pos, f.vel.x) {
            f.kb_vel.x = Fx::ZERO;
        }
        let still_on = collision::platform(stage, f.platform)
            .is_some_and(|plat| f.pos.x >= plat.left && f.pos.x <= plat.right);
        if !still_on {
            f.platform = NONE;
        }
    } else {
        f.vel.y = (f.vel.y - p.gravity).max(-p.max_fall_speed);
        let moved = move_by(f, p, stage, f.vel + f.kb_vel);
        if moved.wall {
            f.vel.x = Fx::ZERO;
            f.kb_vel.x = Fx::ZERO;
        }
        if moved.ceiling {
            f.vel.y = Fx::ZERO;
            f.kb_vel.y = Fx::ZERO;
        }
        if let Some(platform) = moved.landing {
            land(f, p, stage, platform);
            // A shield press just before touching down is a tech: in place, or a roll if the stick is flicked
            // sideways. Without one the fighter lies in a knockdown and picks a get-up.
            let teched = f.pressed_within(buttons::SHIELD, rules.tech_window);
            let roll_dir = f.flick_x(rules.tech_window);
            f.kb_vel = Vec2::ZERO;
            f.vel.x = Fx::ZERO;
            f.hitstun = 0;
            f.tumble = false;
            if teched && roll_dir != 0 {
                f.dodge_dir = Vec2::new(Fx::from_int(i32::from(roll_dir)), Fx::ZERO);
                enter(f, S::Roll);
            } else if teched {
                f.invuln = f.invuln.max(rules.tech_invuln);
                enter_landing(f, rules.tech_lag);
            } else {
                enter(f, S::Knockdown);
            }
            return;
        }
    }

    if f.hitstun == 0 {
        f.tumble = false;
        if f.grounded() {
            f.kb_vel = Vec2::ZERO;
            enter(f, S::Idle);
        } else {
            // Leftover launch speed carries on as ordinary air momentum.
            f.vel += f.kb_vel;
            f.kb_vel = Vec2::ZERO;
            enter(f, S::Airborne);
        }
    }
}

/// Turns a pending hit into motion at the end of hitlag. Directional influence: the part of the held stick
/// that is perpendicular to the launch direction bends the angle, by up to `di_degrees`.
fn apply_launch(f: &mut Fighter, rules: &Ruleset) {
    let angle = Angle::from_raw(f.launch_angle);
    let (ux, uy) = (trig::cos(angle), trig::sin(angle));
    let input = f.history[0];
    let cross = (ux * input.stick_y_fx() - uy * input.stick_x_fx()).clamp(-Fx::ONE, Fx::ONE);
    let bend =
        (cross.mul_int(i32::from(rules.di_degrees)) * Fx::from_ratio(4096, 360)).raw() / 65536;
    let bent = Angle::from_raw((i32::from(f.launch_angle) + bend).rem_euclid(4096) as u16);

    // 0.03 reference units per knockback unit, 8 reference units per world unit.
    let speed = f.launch_kb * Fx::from_ratio(3, 800);
    f.kb_vel = Vec2::new(trig::cos(bent) * speed, trig::sin(bent) * speed);
    f.vel = Vec2::ZERO;
    f.launch_pending = false;
    f.tumble = f.launch_kb >= rules.tumble_knockback;
    if f.grounded() {
        if f.kb_vel.y > Fx::ZERO {
            f.platform = NONE;
        } else {
            f.kb_vel.y = Fx::ZERO;
        }
    }
}

/// Survival DI: a stick flick during hitlag nudges the fighter.
fn sdi(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    let (dx, dy) = (f.flick_x(1), f.flick_y(1));
    let d = rules.sdi_distance;
    if dx != 0 {
        collision::move_x(stage, p, &mut f.pos, d.mul_int(i32::from(dx)));
    }
    if dy > 0 && !f.grounded() {
        collision::move_up(stage, p, &mut f.pos, d);
    } else if dy < 0 && !f.grounded() {
        match collision::surface_below(stage, f.pos, false) {
            Some((_, dist)) if dist < d => f.pos.y -= dist,
            _ => f.pos.y -= d,
        }
    }
}
