//! Per-fighter state machine: ground and air movement, jump squat, air dodge, wavedash,
//! shield drop, and ledge hang options.
//!
//! Conventions: [`update`] runs once per frame per fighter in player-index order. A state entered
//! during a frame starts at `state_frame == 0` and is incremented at the start of the next update,
//! so "lasts N frames" means `state_frame >= N` ends it. All numbers come from [`FighterParams`].

use crate::collision;
use crate::content::{FighterParams, Ledge, Stage};
use crate::fixed::Fx;
use crate::input::{buttons, Input, STICK_DEADZONE, STICK_THRESHOLD};
use crate::state::{Fighter, FighterState as S, NONE};
use crate::vec2::Vec2;

/// Frames of leniency for ground jump and shield-drop presses.
const TAP_BUFFER: u8 = 3;
/// Frames of leniency for air jump and air dodge presses.
const AIR_ACTION_BUFFER: u8 = 2;

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

fn x_active(input: Input) -> bool {
    input.stick_x.unsigned_abs() >= STICK_DEADZONE.unsigned_abs()
}

fn face_stick(f: &mut Fighter, input: Input) {
    if x_active(input) {
        f.facing = if input.stick_x > 0 { 1 } else { -1 };
    }
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
        S::Idle | S::Run => ground(f, p, stage),
        S::JumpSquat => jump_squat(f, p, stage),
        S::Airborne => return airborne(f, p, stage),
        S::AirDodge => air_dodge(f, p, stage),
        S::Landing => landing(f, p, stage),
        S::WaveLand => wave_land(f, p, stage),
        S::Shield => shield(f, p, stage),
        S::ShieldDrop => shield_drop(f, p, stage),
        S::LedgeHang => ledge_hang(f, p, stage),
        S::LedgeGetUp => ledge_get_up(f, p),
    }
    None
}

/// Moves along the ground, stepping off into the air if the platform ends. Returns false if it fell off.
fn slide_on_platform(f: &mut Fighter, stage: &Stage) -> bool {
    f.pos.x += f.vel.x;
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
    if x_active(input) {
        f.vel.x = approach(f.vel.x, input.stick_x_fx() * p.run_speed, p.ground_accel);
        face_stick(f, input);
    } else {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    }
    if slide_on_platform(f, stage) {
        f.state = if f.vel.x == Fx::ZERO { S::Idle } else { S::Run };
    }
}

fn jump_squat(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, stage) {
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

fn enter_landing(f: &mut Fighter, p: &FighterParams) {
    enter(
        f,
        if p.landing_lag == 0 {
            S::Idle
        } else {
            S::Landing
        },
    );
}

/// Air drift, gravity, movement and platform landing. Returns true if the fighter landed.
fn air_move(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> bool {
    let input = f.history[0];
    if x_active(input) {
        f.vel.x = approach(f.vel.x, input.stick_x_fx() * p.air_speed, p.air_accel);
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

    let prev = f.pos;
    f.pos += f.vel;
    match collision::find_landing(stage, prev, f.pos, f.platform_ignore > 0) {
        Some(i) => {
            land(f, p, stage, i);
            true
        }
        None => false,
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
        enter_landing(f, p);
        return None;
    }
    if f.vel.y <= Fx::ZERO && f.ledge_cooldown == 0 {
        return collision::find_ledge(stage, f.pos, p).map(|i| i as u8);
    }
    None
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
    let prev = f.pos;
    f.pos += f.vel;
    if let Some(i) = collision::find_landing(stage, prev, f.pos, f.platform_ignore > 0) {
        if f.dodge_dir.y <= -p.wavedash_min_down {
            start_waveland(f, p, stage, i);
        } else {
            // Too horizontal to be a wavedash: a plain air dodge that happens to land.
            land(f, p, stage, i);
            enter_landing(f, p);
        }
        return;
    }
    if f.state_frame >= u16::from(p.air_dodge_frames) {
        enter(f, S::Airborne);
    }
}

fn landing(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if slide_on_platform(f, stage) && f.state_frame >= u16::from(p.landing_lag) {
        enter(f, S::Idle);
    }
}

fn wave_land(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = f.vel.x * p.waveland_friction;
    if slide_on_platform(f, stage) && f.state_frame >= u16::from(p.waveland_lag) {
        enter(f, S::Idle);
    }
}

fn shield(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, stage) {
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
        enter_landing(f, p);
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
        get_up(f, stage, &l, p.ledge_roll_dx);
    } else if up >= threshold || toward >= threshold {
        get_up(f, stage, &l, p.ledge_getup_dx);
    } else if up <= -threshold || toward <= -threshold || f.state_frame >= p.ledge_hang_max {
        release_ledge(f, p);
        enter(f, S::Airborne);
    }
}

fn get_up(f: &mut Fighter, stage: &Stage, l: &Ledge, dx: Fx) {
    f.pos = Vec2::new(l.x - dx.mul_int(i32::from(l.side)), l.y);
    f.vel = Vec2::ZERO;
    f.ledge = NONE;
    f.platform = collision::standing_on(stage, f.pos);
    if f.platform == NONE {
        enter(f, S::Airborne);
    } else {
        enter(f, S::LedgeGetUp);
    }
}

fn ledge_get_up(f: &mut Fighter, p: &FighterParams) {
    if f.state_frame >= u16::from(p.ledge_getup_frames) {
        // Stable ground again: the next ledge grab gets full invincibility.
        f.ledge_grab_count = 0;
        enter(f, S::Idle);
    }
}
