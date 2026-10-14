//! Per-fighter state machine: ground movement (walk, dash, run, turn, crouch), jump squat, air
//! movement, air dodge, wavedash, shield drop, platform drop, helpless fall, and ledge options.
//!
//! Conventions: [`update`] runs once per frame per fighter in player-index order. A state entered
//! during a frame starts at `state_frame == 0` and is incremented at the start of the next update,
//! so "lasts N frames" means `state_frame >= N` ends it. All numbers come from [`FighterParams`].

use crate::collision;
use crate::content::{FighterParams, Ledge, Ruleset, Stage};
use crate::fixed::Fx;
use crate::input::{
    buttons, Input, STICK_AIM, STICK_DASH, STICK_DEADZONE, STICK_DOWN, STICK_FLICK_FRAMES,
    STICK_FLICK_FROM, STICK_THRESHOLD,
};
use crate::moves::{Encase, Motion, Move, MoveId, Weapon};
use crate::scripting;
use crate::state::{Fighter, FighterState as S, NONE};
use crate::trig::{self, Angle};
use crate::vec2::Vec2;

/// Frames of leniency for stick taps (platform drop, roll and spot dodge out of a shield, get-up rolls).
const TAP_BUFFER: u8 = 3;
/// Buttons the hold buffer applies to: one still held (and not yet used) when its action becomes possible starts it. Not the
/// shield, which is held for its own sake.
const HOLD_BUFFERED: u16 = buttons::ATTACK | buttons::SPECIAL | buttons::JUMP | buttons::GRAB;
/// Frames in which a stick flick still counts as a dash input.
const FLICK_BUFFER: u8 = 2;
/// Frames in which a hard down press still triggers a fast fall, as in the reference: a press up to three frames before the apex
/// works once the fighter starts falling, if the stick is still held down.
const FAST_FALL_BUFFER: u8 = 3;
/// Frames a run skids on while its stick, rolled along the bottom or top to turn it, passes straight down or up.
const RUN_SKID_FRAMES: usize = 3;

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

    /// The input buffer: true if `button` was pressed this frame or up to `frames` frames earlier, or is still held (the hold
    /// buffer, see [`HOLD_BUFFERED`]), and that press has not already started an action.
    pub fn buffered(&self, button: u16, frames: u8) -> bool {
        self.buffer_used & button == 0
            && (self.pressed_within(button, frames.saturating_add(1))
                || (button & HOLD_BUFFERED != 0 && self.held(button)))
    }

    /// [`Fighter::buffered`], and if so the press is used up: it cannot start another action.
    pub fn take(&mut self, button: u16, frames: u8) -> bool {
        let ok = self.buffered(button, frames);
        if ok {
            self.buffer_used |= button;
        }
        ok
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
    /// the full threshold coming from near neutral within `STICK_FLICK_FRAMES`. Merely holding down, or
    /// rolling slowly down to it, does not count. This is the fast-fall input: a flick or hard press on
    /// a stick, a double tap on a keyboard.
    pub fn hard_down(&self, frames: u8) -> bool {
        let n = usize::from(frames).min(self.history.len());
        (0..n).any(|i| {
            self.history[i].stick_y <= -STICK_THRESHOLD
                && self
                    .history
                    .get(i + 1)
                    .is_none_or(|o| o.stick_y > -STICK_THRESHOLD)
                && self.flick_started(i, |o| o.stick_y > -STICK_FLICK_FROM)
        })
    }

    /// Whether, within `STICK_FLICK_FRAMES` before history frame `crossed`, the stick was where a flick
    /// starts (`low`): how fast it travelled to the threshold. Frames older than the history count.
    fn flick_started(&self, crossed: usize, low: impl Fn(&Input) -> bool) -> bool {
        (crossed + 1..=crossed + STICK_FLICK_FRAMES).any(|j| self.history.get(j).is_none_or(&low))
    }

    /// Direction (-1 or 1) of a horizontal flick within the last `frames` frames, or 0. A flick is
    /// the stick crossing the threshold from below, and it must still be held past it now.
    pub fn flick_x(&self, frames: u8) -> i8 {
        self.flick_x_past(frames, STICK_THRESHOLD)
    }

    /// [`Fighter::flick_x`] for a dash (and a dash's turnarounds): the stick has to go further, to `STICK_DASH`, so a quick
    /// push to a walk stays a walk.
    pub fn dash_flick(&self, frames: u8) -> i8 {
        self.flick_x_past(frames, STICK_DASH)
    }

    fn flick_x_past(&self, frames: u8, threshold: i8) -> i8 {
        let threshold = threshold.unsigned_abs();
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
                && self.history.get(i + 1).is_none_or(|o| !past(o.stick_x))
                && self.flick_started(i, |o| started_low(o.stick_x))
        });
        if crossed {
            dir
        } else {
            0
        }
    }
}

fn enter(f: &mut Fighter, state: S) {
    // A pending post-shieldstun shield grab belongs to the shield it was pressed in.
    f.shield_grab = 0;
    // A dodge is fresh unless the code that starts it stales it (`stale_dodge`): get-up and tech rolls are not staled.
    if matches!(state, S::Roll | S::SpotDodge | S::AirDodge) {
        f.dodge_level = 0;
    }
    // A pull lasts only while the fighter is in hitstun.
    if state != S::Hitstun {
        f.pull = false;
    }
    // Script variables belong to one attack (and the moves it switches to), so a new attack starts clean.
    if state == S::Attack && f.state != S::Attack {
        f.vars = [0; crate::FIGHTER_VARS];
    }
    if state != S::Hitstun {
        f.balloon_faf = 0;
        f.balloon_left = Fx::ZERO;
        f.hits_taken = 0;
    }
    // Doing anything at all (or landing, or being hit again) ends a damage fall.
    if state != S::Airborne {
        f.damage_fall = false;
        // ...and leaves a midair jump (its armor goes with it).
        f.air_jump_frame = 0;
    }
    // A jump out of a run or dash remembers the run's direction (the way the fighter is moving: a reverse aerial rush has already turned
    // it to face back) until it lands; anything else on the ground, a hit or a ledge forgets it.
    if state == S::JumpSquat {
        f.run_jump = if matches!(f.state, S::Run | S::Dash) {
            if f.vel.x > Fx::ZERO {
                1
            } else if f.vel.x < Fx::ZERO {
                -1
            } else {
                f.facing
            }
        } else {
            0
        };
    } else if matches!(
        state,
        S::Hitstun
            | S::LedgeHang
            | S::Grabbed
            | S::Knockdown
            | S::Respawn
            | S::ShieldBreak
            | S::Rebound
            | S::WallTech
    ) || (f.platform != NONE && state != S::Landing)
    {
        f.run_jump = 0;
    }
    f.state = state;
    f.state_frame = 0;
    // A jab's window past its end lasts only while the fighter stays as the jab left it (see `Move::late_next`).
    f.late_next = 0;
    f.late_left = 0;
    // A trip is one knockdown: anything after it (the get-up, a launch, another knockdown) is not a trip.
    f.tripped = false;
    f.trip_soft = false;
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

/// Acceleration toward a walking speed target: speed above it (left over from a landing or a turn) comes off at traction
/// (`ground_friction`), as in the reference game, and the walk acceleration (`walk_accel`, more the further the stick is pushed)
/// brings the walk up to speed.
fn ease_rate(vel: Fx, target: Fx, p: &FighterParams, input: Input) -> Fx {
    let overspeed = vel.signum_int() == target.signum_int() && vel.abs() > target.abs();
    if overspeed {
        p.ground_friction
    } else {
        stick_accel(p.walk_accel, p.walk_accel_stick, input)
    }
}

/// A ground acceleration as the reference game has it: a base plus a share scaled by how far the stick is pushed sideways.
fn stick_accel(base: Fx, stick: Fx, input: Input) -> Fx {
    base + stick * input.stick_x_fx().abs()
}

fn x_active(input: Input) -> bool {
    input.stick_x.unsigned_abs() >= STICK_DEADZONE.unsigned_abs()
}

/// The stick is on its rim at the top or bottom, and was pushed to one side within the last `RUN_SKID_FRAMES`: it is being
/// rolled from one side to the other, not let go (see the run in `ground`).
fn rolling(f: &Fighter) -> bool {
    let now = f.history[0];
    now.stick_y.unsigned_abs() >= STICK_DOWN.unsigned_abs()
        && f.history
            .iter()
            .skip(1)
            .take(RUN_SKID_FRAMES)
            .any(|&i| x_active(i))
}

/// Lag a dash flicked during is kept for (`Fighter::dash_buffer`): a grab, a pummel or a throw (command grabs too: a move that
/// catches, or anything done while holding someone), a landing's lag (sim v82), or the falling half of a jump (sim v93: from the apex
/// down, in the air or in an aerial, so the dash comes out on landing). Public for the keyboard reader, which sends a direction key
/// pressed during it at full strength, a flick (`godot/scripts/input_reader.gd`).
pub fn keeps_dash(f: &Fighter, weapon: &Weapon) -> bool {
    (f.state == S::Landing && f.grounded())
        || (!f.grounded() && f.vel.y <= Fx::ZERO && matches!(f.state, S::Airborne | S::Attack))
        || f.state == S::Attack
            && (matches!(
                MoveId::from_index(f.move_id),
                MoveId::Grab
                    | MoveId::DashGrab
                    | MoveId::PivotGrab
                    | MoveId::Pummel
                    | MoveId::FThrow
                    | MoveId::BThrow
                    | MoveId::UThrow
                    | MoveId::DThrow
            ) || f.grab_with != NONE
                || weapon.get(f.move_id).catch_move.is_some())
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
    world: &scripting::World,
) -> Option<u8> {
    f.history.rotate_right(1);
    f.history[0] = input;
    // A new press of a button can start a new action.
    f.buffer_used &= !(input.buttons & !f.history[1].buttons);
    // Attack presses and releases since the attack began, hitlag included (a rapid jab's entry rule).
    let (now, before) = (
        input.buttons & buttons::ATTACK != 0,
        f.history[1].buttons & buttons::ATTACK != 0,
    );
    if now && !before {
        f.presses = f.presses.saturating_add(1);
    } else if before && !now {
        f.releases = f.releases.saturating_add(1);
    }
    // A tech press opens a short window; another press only counts once `tech_lockout` frames have passed (no mashing).
    f.tech_timer = f.tech_timer.saturating_add(1);
    // The wait before a move can make another tossed item (E47, sim 117) counts down every frame, hitlag included [E].
    f.pull_wait = f.pull_wait.saturating_sub(1);
    // Dodge staleness wears off: a level `dodge_stale_first` frames after the last dodge ended, then one every `dodge_stale_every`.
    if !matches!(f.state, S::Roll | S::SpotDodge | S::AirDodge) {
        f.dodge_clock = f.dodge_clock.saturating_add(1);
        let first = u16::from(rules.dodge_stale_first);
        let every = u16::from(rules.dodge_stale_every.max(1));
        if f.dodge_stale > 0 && f.dodge_clock >= first && (f.dodge_clock - first) % every == 0 {
            f.dodge_stale -= 1;
        }
    }
    if f.pressed_within(buttons::SHIELD, 1) && f.tech_timer > rules.tech_lockout {
        f.tech_timer = 0;
    }

    // A dash flicked at any point of a grab, a pummel, a throw or a landing's lag is kept for the first frame the fighter can act, as
    // long as the stick stays pushed that way (hitlag included): more lenient than the reference's 6 or 7 frames, so a throw or an
    // aerial can be chased (owner's call). The flick that picked the throw is not one (it was made while holding, before the throw
    // began), nor one made while a jump is still rising; one made once it falls is (sim v93).
    if keeps_dash(f, weapon) {
        let flick = f.dash_flick(1);
        if flick != 0 {
            f.dash_buffer = flick;
        }
    } else if !(f.state == S::Idle && (f.state_frame == 0 || f.dash_age != 255))
        && f.state != S::Dash
    {
        // (In an initial dash, and standing out of one, it holds a flick for the dash's end or interrupt frame: see `ground`.)
        f.dash_buffer = 0;
    }
    if f.dash_buffer != 0
        && !(input.stick_x.signum() == f.dash_buffer
            && input.stick_x.unsigned_abs() >= STICK_DASH.unsigned_abs())
    {
        f.dash_buffer = 0;
    }

    // The gauge's powered mode counts down every frame, hitlag included [E: gap 12]; a hold out of a draining move drains only while
    // it lasts (a throw's hit, which lets go on the frame it lands, still drains: it is applied before this runs again).
    crate::gauge::tick(f);
    if f.state != S::Grabbed {
        f.gauge_drain = false;
    }
    // Hitlag freezes everything about the fighter except its input history. A struck fighter can still
    // survival-DI, and its launch happens on the frame hitlag ends.
    if f.hitlag > 0 {
        f.sdi_wait = f.sdi_wait.saturating_sub(1);
        if f.launch_pending {
            sdi(f, p, stage, rules, false);
        } else if f.state == S::Shield || f.shield_sdi {
            // (A passive shield's block, E14, allows shield SDI as a raised shield does.)
            sdi(f, p, stage, rules, true);
        }
        f.hitlag -= 1;
        if f.hitlag == 0 {
            f.shield_sdi = false;
        }
        if f.hitlag == 0 && f.launch_pending {
            asdi(f, p, stage, rules);
            apply_launch(f, p, rules);
        }
        return None;
    }
    // A move switch decided after last frame's update (a search box, a command grab) only waits for an attack that is still going.
    if f.state != S::Attack {
        f.next_move = 0;
    }
    // A midair jump is only one while the fighter is still in it (a state set outside `enter`, a launch, ends it too).
    if f.state != S::Airborne {
        f.air_jump_frame = 0;
    }

    f.airtime = if f.platform == NONE {
        f.airtime.saturating_add(1)
    } else {
        0
    };
    f.platform_ignore = f.platform_ignore.saturating_sub(1);
    // A jab's window past its end (`Move::late_next`) counts down while the fighter stands idle after it; anything else ends it.
    if f.state == S::Idle {
        f.late_left = f.late_left.saturating_sub(1);
    } else {
        f.late_left = 0;
    }
    if f.late_left == 0 {
        f.late_next = 0;
    }
    f.grab_immune = f.grab_immune.saturating_sub(1);
    f.ledge_cooldown = f.ledge_cooldown.saturating_sub(1);
    f.ledge_invuln = f.ledge_invuln.saturating_sub(1);
    f.invuln = f.invuln.saturating_sub(1);
    f.state_frame = f.state_frame.saturating_add(1);
    // The shield refills whenever it is not up (a broken shield is restored when the stun ends).
    if !matches!(f.state, S::Shield | S::ShieldBreak) {
        f.shield_hp = (f.shield_hp + rules.shield_regen).min(rules.shield_max);
    }

    // Walking or running off a platform takes the ground speed into the air the way a jump does (see `jump_squat`).
    let on_foot = f.grounded()
        && matches!(
            f.state,
            S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch | S::Landing
        );
    // A pending platform drop belongs to the ground states (see `ground`): anything else (a hit, a grab) ends it.
    if !matches!(
        f.state,
        S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch
    ) {
        f.drop_pending = 0;
    }
    match f.state {
        S::Attack => return attack(f, p, weapon, stage, rules, world),
        S::Hitstun => hitstun(f, p, stage, rules),
        S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch => {
            ground(f, p, weapon, stage, rules, world.pickable[world.me])
        }
        S::JumpSquat => jump_squat(f, p, weapon, stage, rules),
        S::Airborne => return airborne(f, p, weapon, stage, rules),
        S::ItemPickup => item_pickup(f, p, stage, rules),
        S::ItemThrow => return item_throw(f, p, stage, rules),
        S::Helpless => return helpless(f, p, weapon, stage, rules),
        S::AirDodge => return air_dodge(f, p, stage, rules),
        S::Landing => landing(f, p, stage),
        S::WaveLand => wave_land(f, p, stage),
        S::Shield => shield(f, p, weapon, stage, rules),
        S::Roll => roll(f, p, stage, rules),
        S::SpotDodge => spot_dodge(f, p, stage, rules),
        S::ShieldBreak => shield_break(f, p, stage, rules),
        S::Knockdown => knockdown(f, p, weapon, stage, rules),
        S::GetUp => get_up_stand(f, p, stage, rules),
        S::Grabbing => grabbing(f, weapon),
        S::Carrying => return carrying(f, p, weapon, stage, rules),
        // Being held: the holder pins this fighter in place (see `grab::update`).
        S::Grabbed => {}
        S::ShieldDrop => shield_drop(f, p, stage),
        S::LedgeHang => ledge_hang(f, p, weapon, stage),
        S::LedgeJump => ledge_jump(f, p, stage),
        S::ShieldRelease => shield_release(f, p, weapon, stage),
        S::Buried => buried(f, p, rules),
        S::Encased => encased(f, p, stage, world.encase[world.me].as_ref()),
        S::Rebound => rebound(f, p, stage),
        S::WallTech => wall_tech(f, p, rules),
        S::Respawn => revival(f, p, rules),
        S::LedgeGetUp | S::LedgeAttack => ledge_recover(f),
    }
    if on_foot && !f.grounded() && f.state == S::Airborne {
        f.vel.x = air_momentum(f.vel.x, p, rules);
    }
    None
}

/// Ground speed leaving the ground: `jump_momentum` of it, at most `jump_momentum_max` times the air speed (the reference game's
/// jump speed multiplier and cap; the same for stepping off a platform as for a jump).
fn air_momentum(vx: Fx, p: &FighterParams, rules: &Ruleset) -> Fx {
    let cap = p.air_speed * rules.jump_momentum_max;
    (vx * rules.jump_momentum).clamp(-cap, cap)
}

/// Ends whatever the fighter is doing: standing if it is on the ground, else falling (sim 113: balloons that let go on the ground).
pub fn stand_or_fall(f: &mut Fighter) {
    enter(f, if f.grounded() { S::Idle } else { S::Airborne });
}

/// Random versions (E39, sim 113: `Move::versions`): a move that has just begun (on its first frame) draws its version from the
/// game's random generator: `(chance, slot)` with `chance` percent each, else the move itself. A version has no versions of its own,
/// so the draw happens once.
pub fn draw_version(f: &mut Fighter, weapon: &Weapon, rng: &mut crate::rng::Rng) {
    if f.state != S::Attack || f.state_frame != 0 || f.hitlag > 0 {
        return;
    }
    let versions = &weapon.get(f.move_id).versions;
    if versions.is_empty() {
        return;
    }
    let roll = rng.range(100);
    let mut upto = 0u32;
    for (chance, slot) in versions {
        upto += u32::from(*chance);
        if roll < upto {
            f.move_id = *slot;
            return;
        }
    }
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
    f.touched_wall = collision::move_x(stage, p, &mut f.pos, f.vel.x);
    if f.touched_wall {
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

/// Like `slide_on_platform`, but the fighter stops at the platform's edge instead of going over it (rolls, tech rolls and spot
/// dodges, as in the reference: a roll into the edge ends on the edge).
fn slide_kept_on_platform(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    if collision::move_x(stage, p, &mut f.pos, f.vel.x) {
        f.vel.x = Fx::ZERO;
    }
    if let Some(plat) = collision::platform(stage, f.platform) {
        if f.pos.x < plat.left || f.pos.x > plat.right {
            f.pos.x = f.pos.x.clamp(plat.left, plat.right);
            f.vel.x = Fx::ZERO;
        }
    }
}

fn start_dash(f: &mut Fighter, p: &FighterParams, dir: i8) {
    // Turning around out of a dash or run is a dash dance: the fighter stands for `dash_turn_delay` frames, then dashes the new way.
    let reversing = f.facing != dir && matches!(f.state, S::Dash | S::Run);
    f.facing = dir;
    f.dash_age = 0;
    if reversing && p.dash_turn_delay > 0 {
        // (Kept for a jump in the turnaround: a reverse aerial rush.)
        f.turn_speed = f.vel.x;
        f.vel.x = Fx::ZERO;
        // The countdown ends on the frame the first step is taken, so the fighter stands for `dash_turn_delay` frames after the reversal.
        f.dash_wait = p.dash_turn_delay + 1;
        enter(f, S::Dash);
        return;
    }
    f.dash_wait = 0;
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

fn ground(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
    pick: bool,
) {
    let input = f.history[0];
    // (Set again below while a platform drop is still waiting; any other way out of the ground states ends it.)
    let drop_pending = core::mem::take(&mut f.drop_pending);
    // Frames since the last dash began: through the dash, the run, and the stand a dash ends in (for its interrupt frame).
    if matches!(f.state, S::Dash | S::Run) || (f.state == S::Idle && f.dash_age != 255) {
        f.dash_age = f.dash_age.saturating_add(1).min(254);
    } else {
        f.dash_age = 255;
    }
    let buffer = p.input_buffer;
    // The initial dash is committed (see `dash_frames`): no crouch, platform drop or tilt during it, and the shield only from
    // `dash_shield_frame`, which cuts the dash to about half its distance.
    let initial_dash = f.state == S::Dash;
    if f.take(buttons::JUMP, buffer) {
        // A reverse aerial rush: back with the jump (or just before it) out of a dash or run, or a jump in the first frames of a dash
        // turnaround, jumps facing back but still moving the way the fighter was going, so a back aerial leads.
        // (Out of a run any push back past the dead zone turns it, as it does without the jump; out of a dash it takes a flick, or
        // one held for the dash's end.)
        let back_in_run =
            f.state == S::Run && x_active(input) && (input.stick_x > 0) != (f.facing > 0);
        if matches!(f.state, S::Dash | S::Run)
            && f.dash_wait == 0
            && (f.dash_flick(FLICK_BUFFER) == -f.facing
                || back_in_run
                || (f.state == S::Dash && f.dash_buffer == -f.facing))
        {
            f.facing = -f.facing;
        } else if f.state == S::Dash && f.dash_wait > 0 {
            f.vel.x = f.turn_speed;
        }
        enter(f, S::JumpSquat);
        return;
    }
    let shield_held_back = initial_dash && f.state_frame < u16::from(p.dash_shield_frame);
    if f.held(buttons::SHIELD) && !shield_held_back {
        // The press is the shield's: an air dodge later needs a new one.
        f.buffer_used |= buttons::SHIELD;
        enter(f, S::Shield);
        return;
    }
    // A platform drop (owner decision 63, the reference's): the down flick keeps the fighter on the platform for
    // `platform_drop_frames` more frames, crouching, and it falls on the frame after. A special on the flick's frame, on any of
    // those, or on the frame it falls comes out on the ground instead: with the stick down, the down special. Nothing else
    // interrupts it (an attack or grab pressed meanwhile is buffered into the air).
    // (Off since sim 112, owner decision 2026-10-14: `Ruleset::stick_platform_drop`; the drop is shield + special.)
    if !initial_dash && rules.stick_platform_drop != 0 && on_pass_through(f, stage) {
        let mut left = drop_pending;
        if left == 0 && f.flicked_down(TAP_BUFFER) {
            left = rules.platform_drop_frames.saturating_add(1);
        }
        if left > 0 {
            if f.buffered(buttons::SPECIAL, buffer) && start_special(f, weapon) {
                f.buffer_used |= buttons::SPECIAL;
                return;
            }
            left -= 1;
            if left == 0 {
                start_platform_drop(f, p);
                return;
            }
            f.drop_pending = left;
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
            f.state = S::Crouch;
            slide_on_platform(f, p, stage);
            return;
        }
    }
    // Items (E4, the reference's): holding one, grab throws it the way the stick is held, or drops it with the stick in the middle;
    // with an item in reach, grab (or attack below) picks it up instead of grabbing.
    if f.holding != NONE && f.take(buttons::GRAB, buffer) {
        let running = matches!(f.state, S::Dash | S::Run);
        let throw = item_throw_by_stick(f, p, true, running, true);
        start_item_throw(f, p, rules, throw);
        return;
    }
    if pick && f.holding == NONE && f.take(buttons::GRAB, buffer) {
        start_item_pickup(f);
        return;
    }
    if f.take(buttons::GRAB, buffer) {
        // Grabbing while turning around out of a dash or run is a pivot grab: it comes out toward the way the
        // fighter is turning, while it keeps sliding. A weapon without one just uses its normal grab.
        // (A turn from standing still, with no speed to slide on, is not one.)
        let reversing = (f.state == S::Turn && f.vel.x.abs() > p.walk_speed)
            || (matches!(f.state, S::Dash | S::Run) && f.dash_flick(FLICK_BUFFER) == -f.facing);
        let pivot = reversing && !weapon.get(MoveId::PivotGrab as u8).is_empty();
        let id = if pivot {
            if f.state != S::Turn {
                f.facing = -f.facing;
            }
            MoveId::PivotGrab
        } else if matches!(f.state, S::Dash | S::Run) {
            MoveId::DashGrab
        } else {
            MoveId::Grab
        };
        begin_attack(f, id);
        return;
    }
    if f.buffered(buttons::SPECIAL, buffer) && start_special(f, weapon) {
        f.buffer_used |= buttons::SPECIAL;
        return;
    }
    // Holding an item, attack throws it by the stick (out of a dash or run, the dash throw); an item in reach is picked up by an attack
    // that is not a smash input (E4).
    if f.holding != NONE && f.take(buttons::ATTACK, buffer) {
        let running = matches!(f.state, S::Dash | S::Run);
        let throw = item_throw_by_stick(f, p, false, running, true);
        start_item_throw(f, p, rules, throw);
        return;
    }
    if pick && f.holding == NONE && f.buffered(buttons::ATTACK, buffer) && !smash_input(f, p) {
        f.buffer_used |= buttons::ATTACK;
        start_item_pickup(f);
        return;
    }
    if f.take(buttons::ATTACK, buffer) {
        start_ground_attack(f, p);
        return;
    }

    // A flick starts a dash. A run is committed: flicking the other way skids to a stop and turns, and a flick the same way changes
    // nothing. In an initial dash (sim 103, the reference's dash turnaround): a flick back in its first `dash_reverse_frames` frames
    // turns at once (a dash dance); later it is held (`dash_buffer`, while the stick stays there) and the dash turns as it ends; and
    // once the dash has ended with the stick let go, the next dash (a foxtrot, or a dash back) waits for its interrupt frame
    // (`Ruleset::dash_interrupt_frame`), held in `dash_buffer` as long as the stick stays there; meanwhile the fighter stands.
    // On the first frame the fighter can act after lag (a landing, an aerial's landing lag, the end of an attack), a dash flicked during
    // that lag still comes out, as long as the stick is still held past the dash threshold: the input buffer, for the stick. Otherwise
    // only a fresh flick dashes.
    let window = if f.state == S::Idle && f.state_frame <= 1 {
        buffer.saturating_add(1)
    } else {
        FLICK_BUFFER
    };
    let mut flick = f.dash_flick(window);
    // A dash flicked during a grab or throw (see `update`) comes out now.
    if flick == 0 && f.state == S::Idle && f.state_frame <= 1 {
        flick = f.dash_buffer;
    }
    // (Frames are counted from 1: the dash's first frame has `dash_age` 0.)
    let interrupted =
        f.state == S::Idle && f.dash_age.saturating_add(1) < rules.dash_interrupt_frame;
    if f.state != S::Dash && !interrupted {
        if flick == 0 && f.state == S::Idle {
            // A dash held for the interrupt frame comes out now.
            flick = f.dash_buffer;
        }
        f.dash_buffer = 0;
    }
    if flick != 0 {
        if f.state == S::Run {
            if flick != f.facing {
                // A run's turnaround: it skids to a stop facing the new way (a jump in it keeps the run's speed).
                f.facing = flick;
                f.turn_speed = f.vel.x;
                enter(f, S::Turn);
            }
        } else if f.state == S::Dash {
            if flick != f.facing {
                if f.state_frame < u16::from(p.dash_reverse_frames) {
                    start_dash(f, p, flick);
                } else {
                    // Held for the dash's end, where it turns.
                    f.dash_buffer = flick;
                }
            }
        } else if interrupted {
            // Held for the interrupt frame.
            f.dash_buffer = flick;
        } else if !(f.state == S::Turn && f.turn_speed != Fx::ZERO) {
            // (A run's skid is committed: the flick that turned it, still in the buffer, or a new one, does not dash out of it.)
            start_dash(f, p, flick);
        }
    }

    let dir = if input.stick_x > 0 { 1 } else { -1 };
    match f.state {
        S::Dash => {
            // Letting go of the stick does not stop the dash: it runs its full `dash_frames` either way.
            if f.dash_wait > 0 {
                // The turnaround of a reversed dash: standing, then the first step. Its standing frames do not count toward the dash,
                // so a dash back lasts `dash_turn_delay` frames longer (the reference's pivot dash: two frames longer than a dash).
                f.dash_wait -= 1;
                f.state_frame = 0;
                f.vel.x = Fx::ZERO;
                if f.dash_wait == 0 {
                    f.vel.x = p.dash_initial_speed.mul_int(i32::from(f.facing));
                }
            } else {
                let target = p.dash_speed.mul_int(i32::from(f.facing));
                f.vel.x = approach(f.vel.x, target, p.dash_accel);
                if f.state_frame >= u16::from(p.dash_frames) {
                    if core::mem::take(&mut f.dash_buffer) == -f.facing {
                        // A flick back held from later in the dash: it turns now.
                        start_dash(f, p, -f.facing);
                    } else if (x_active(input) && dir == f.facing) || rolling(f) {
                        f.state = S::Run;
                    } else {
                        // The stick was let go: the speed bleeds off quickly on braking (`dash_brake`). The next dash waits for
                        // the interrupt frame (above).
                        f.vel.x = approach(f.vel.x, Fx::ZERO, p.dash_brake);
                        f.state = S::Idle;
                    }
                }
            }
        }
        // Standing out of a dash, holding a dash for its interrupt frame: it neither walks nor turns meanwhile.
        S::Idle if interrupted && f.dash_buffer != 0 => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
        }
        S::Turn => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
            // (A run's skid, which keeps the run's speed for a reverse aerial rush, lasts longer than a turn from standing.)
            let length = if f.turn_speed != Fx::ZERO {
                p.run_turn_frames
            } else {
                p.turn_frames
            };
            if f.state_frame >= u16::from(length) {
                // Out of a run's skid with the stick still pushed hard the new way, the fighter runs on that way.
                let hard = input.stick_x.unsigned_abs() >= STICK_THRESHOLD.unsigned_abs();
                f.state = if f.turn_speed != Fx::ZERO && hard && dir == f.facing {
                    S::Run
                } else {
                    S::Idle
                };
                f.turn_speed = Fx::ZERO;
            }
        }
        // A stick rolled along the bottom (or the top) from one side to the other, the reference's way into a run's turnaround:
        // it only points straight down or up for a moment, so the run skids on through that moment (`RUN_SKID_FRAMES`) instead
        // of stopping or crouching, and the far side turns it. (Straight down still crouches, once the skid is over.)
        S::Run if !x_active(input) && rolling(f) => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.run_decel);
        }
        // While running, the stick's lower half on either side is still that side: down and forward keeps running, down and
        // back turns the run.
        _ if input.stick_y <= -STICK_DOWN && !(f.state == S::Run && x_active(input)) => {
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
            f.state = S::Crouch;
        }
        _ if x_active(input) => {
            if dir != f.facing {
                // (Pushed back out of a run without a flick, it is still the run's skid.)
                f.turn_speed = if f.state == S::Run { f.vel.x } else { Fx::ZERO };
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
                    stick_accel(p.run_accel, p.run_accel_stick, input)
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
                f.vel.x = approach(f.vel.x, target, ease_rate(f.vel.x, target, p, input));
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

fn jump_squat(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage, rules: &Ruleset) {
    // No friction here: a run or dash jump keeps its momentum into the air.
    if !slide_on_platform(f, p, stage) {
        return;
    }
    // The jump squat can be cancelled (E37, the reference's, owner decision 62), for every fighter: special with the stick up is the
    // grounded up special, and an up smash input (attack with the stick flicked up, the strong button with the stick up, or the
    // attack stick up) the up smash, still on the ground. Only a press made after the jump's own: one made with the jump stays the
    // short-hop aerial or aerial special it always was (the house's short-hop input); attack with the stick merely held up stays the
    // short-hop up air.
    if up_out_of_ground(f, p, weapon, true) {
        return;
    }
    if f.state_frame >= u16::from(p.jump_squat_frames) {
        f.platform = NONE;
        f.fast_fall = false;
        // The ground speed carries into the air reduced (`jump_momentum`) and capped (`jump_momentum_max` of the air speed), so a
        // running jump is a little faster than a full drift, not a run's speed carried through the whole jump.
        f.vel.x = air_momentum(f.vel.x, p, rules);
        enter(f, S::Airborne);
        // An attack pressed during the jump squat (or with the jump) makes it a short hop, held jump or not: the short-hop aerial.
        // The aerial itself comes out of the buffer on the first airborne frame.
        let aerial = f.buffer_used & buttons::ATTACK == 0
            && f.pressed_within(buttons::ATTACK, p.jump_squat_frames.saturating_add(1));
        let full = f.held(buttons::JUMP) && !aerial;
        f.short_hop = !full;
        if full && p.hop_burst_frames > 0 {
            // Full hop: a fast opening, then the arc (see `air_move`).
            f.vel.y = p.hop_burst_velocity;
            f.hop_boost = p.hop_burst_frames;
        } else {
            f.vel.y = if full {
                p.full_hop_velocity
            } else {
                p.short_hop_velocity
            };
        }
        // Wavedash input: a shield press during jump squat fires on the first airborne frame.
        if !f.air_dodge_used && f.take(buttons::SHIELD, p.air_dodge_buffer) {
            start_air_dodge(f, p, stage, rules);
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
    f.short_hop = false;
    f.ledge_grab_count = 0;
    f.air_flags = 0;
    f.wall_jumps = 0;
}

fn enter_landing(f: &mut Fighter, lag: u8) {
    f.lag = lag;
    if lag == 0 {
        exit_landing(f);
    } else {
        enter(f, S::Landing);
    }
}

/// Leaves a landing. A jump taken out of a run lands back into the run if the stick is still held the run's way (`run_jump`), so a
/// running jump keeps going; otherwise the fighter stands, and walks on if the stick is held (a run then needs a new dash). Speed above
/// walking speed comes off at traction.
fn exit_landing(f: &mut Fighter) {
    let input = f.history[0];
    let held = if input.stick_x > 0 { 1 } else { -1 };
    if f.run_jump != 0 && x_active(input) && held == f.run_jump {
        f.facing = f.run_jump;
        // (A run, not a dash: turning back is a skid, not a dash dance.)
        f.dash_age = 255;
        enter(f, S::Run);
        return;
    }
    enter(f, S::Idle);
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
    f.touched_wall = wall;
    Moved {
        landing,
        wall,
        ceiling,
    }
}

/// Air drift, gravity, movement and landing. Returns true if the fighter landed.
fn air_move(f: &mut Fighter, p: &FighterParams, stage: &Stage) -> bool {
    air_move_with(f, p, stage, true)
}

/// [`air_move`], with the fast fall allowed or not (a move's `fast_fall_from`).
fn air_move_with(f: &mut Fighter, p: &FighterParams, stage: &Stage, fast_fall_ok: bool) -> bool {
    air_steer_x(f, p);
    air_move_vertical(f, p, stage, fast_fall_ok)
}

/// The horizontal part of air control (see `air_move_with`): the stick's pull toward its share of the air speed, or air friction.
fn air_steer_x(f: &mut Fighter, p: &FighterParams) {
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
}

/// The rest of `air_move_with` after the horizontal control: fast fall, gravity, the move and its landing.
fn air_move_vertical(
    f: &mut Fighter,
    p: &FighterParams,
    stage: &Stage,
    fast_fall_ok: bool,
) -> bool {
    if fast_fall_ok
        && !f.fast_fall
        && !f.damage_fall
        && f.vel.y <= Fx::ZERO
        && f.hard_down(FAST_FALL_BUFFER)
        && f.history[0].stick_y <= -STICK_THRESHOLD
    {
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
    if f.vel.y <= Fx::ZERO {
        ledge_in_reach(f, p, stage)
    } else {
        None
    }
}

/// A ledge this fighter could catch now (rising or not): not just let go of, not declined by holding down, within the grab limit.
fn ledge_in_reach(f: &Fighter, p: &FighterParams, stage: &Stage) -> Option<u8> {
    if f.ledge_cooldown == 0 && !declines_ledge(f) && f.ledge_grab_count < p.ledge_grab_limit {
        collision::find_ledge(stage, f.pos, f.facing, p).map(|i| i as u8)
    } else {
        None
    }
}

/// Lag for touching down from the air: heavy when falling at full speed (fast falling or at the maximum fall speed), light
/// otherwise. Call before moving, while the falling speed is still known.
fn plain_landing_lag(f: &Fighter, p: &FighterParams) -> u8 {
    if f.fast_fall || f.vel.y <= -p.max_fall_speed {
        p.heavy_landing_lag
    } else {
        p.landing_lag
    }
}

fn airborne(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
) -> Option<u8> {
    let buffer = p.input_buffer;
    // A wall jump: touching a wall, the stick held away from it and jump pressed (owner decision, sim 101: the stick alone, or jump
    // with the stick neutral or toward the wall, made it too easy to do by accident). It is checked first, so it never spends a
    // midair jump; jump against a wall without the stick held away is an ordinary midair jump.
    if p.wall_jump != 0 {
        let side = collision::wall_beside(stage, p, f.pos, WALL_JUMP_REACH);
        let input = f.history[0];
        let away = x_active(input) && i32::from(input.stick_x.signum()) == -i32::from(side);
        if side != 0 && away && f.take(buttons::JUMP, buffer) {
            wall_jump(f, p, rules, -side);
        }
    }
    // The midair jump's frame count (its armor's clock): one more each frame the fighter stays in it.
    if f.air_jump_frame > 0 {
        f.air_jump_frame = f.air_jump_frame.saturating_add(1);
    }
    if f.air_jumps_left > 0 && f.take(buttons::JUMP, buffer) {
        f.vel.y = p.air_jump_velocity;
        // A midair jump sets the drift from the stick: it can change direction at once, and with the stick neutral it goes
        // straight up.
        let input = f.history[0];
        // (A body's own multiplier on that drift: `FighterParams::air_jump_drift`, owner decision 59.)
        f.vel.x = if x_active(input) {
            input.stick_x_fx() * p.air_speed * Fx::from_ratio(i32::from(p.air_jump_drift), 100)
        } else {
            Fx::ZERO
        };
        // A body that turns with its midair jump faces the way the stick is held back (E33).
        if p.air_jump_turns && x_active(input) && input.stick_x.signum() == -f.facing {
            f.facing = -f.facing;
        }
        f.air_jump_frame = 1;
        f.hop_boost = 0;
        f.air_jumps_left -= 1;
        f.fast_fall = false;
        f.damage_fall = false;
        f.short_hop = false;
    }
    if !f.air_dodge_used && f.take(buttons::SHIELD, buffer) {
        start_air_dodge(f, p, stage, rules);
        return None;
    }
    if f.buffered(buttons::SPECIAL, buffer) && start_special(f, weapon) {
        f.buffer_used |= buttons::SPECIAL;
        return None;
    }
    // Holding an item (E4): attack is the aerial throw by the stick, grab with the stick in the middle lets go of it at once (the
    // reference's aerial drop, no lag), or throws it the way the stick is held.
    if f.holding != NONE && f.take(buttons::ATTACK, buffer) {
        let throw = item_throw_by_stick(f, p, false, false, false);
        start_item_throw(f, p, rules, throw);
        return None;
    }
    if f.holding != NONE && f.take(buttons::GRAB, buffer) {
        let throw = item_throw_by_stick(f, p, true, false, false);
        if throw == crate::items::THROW_LET_GO {
            f.item_throw = throw;
            f.item_order |= crate::items::ORDER_RELEASE;
        } else {
            start_item_throw(f, p, rules, throw);
            return None;
        }
    }
    if f.take(buttons::ATTACK, buffer) {
        start_air_attack(f, p);
        return None;
    }
    let lag = plain_landing_lag(f, p);
    // Still tumbling from a strong launch (it has not acted since): landing is a tech, or a missed tech that leaves it lying down,
    // as at the end of a launch.
    let tumbling = f.damage_fall;
    if air_move(f, p, stage) {
        if tumbling {
            tumble_landing(f, rules);
        } else {
            enter_landing(f, lag);
        }
        return None;
    }
    ledge_request(f, p, stage)
}

/// Special fall: drift and ledge grabs only. Landing costs extra lag.
/// The special fall after an up special (or another move that leaves the fighter helpless): it can only drift a little (a share of
/// its air speed and acceleration, `helpless_drift`), fast fall, and catch a ledge, until it lands.
///
/// The move that caused it (still `move_id`) can set its own drift (`Move::fall_drift`) and landing lag (`Move::fall_landing_lag`),
/// as the reference does per move; otherwise they are the ruleset's and the body's.
fn helpless(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
) -> Option<u8> {
    let mv = weapon.get(f.move_id);
    let drift = if mv.fall_drift > 0 {
        Fx::from_ratio(i32::from(mv.fall_drift), 100)
    } else {
        rules.helpless_drift
    };
    let lag = if mv.fall_landing_lag > 0 {
        mv.fall_landing_lag
    } else {
        p.helpless_landing_lag
    };
    let mut q = *p;
    q.air_speed = mv.fall_speed.unwrap_or(p.air_speed * drift);
    if let Some(stick) = mv.fall_accel {
        // (The move's own stick acceleration: the base acceleration is the body's.)
        q.air_accel_stick = stick;
    } else {
        q.air_accel = p.air_accel * drift;
        q.air_accel_stick = p.air_accel_stick * drift;
    }
    if air_move(f, &q, stage) {
        enter_landing(f, lag);
        return None;
    }
    ledge_request(f, p, stage)
}

fn start_air_dodge(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    let input = f.history[0];
    let level = f.dodge_stale;
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
    if dir == Vec2::ZERO {
        // A neutral dodge keeps the fighter's momentum: it falls and drifts as usual while intangible.
        enter(f, S::AirDodge);
        stale_dodge(f, rules);
        return;
    }
    f.fast_fall = false;
    if p.air_dodge_windup > 0 {
        f.vel = sling_velocity(p, dir);
    } else {
        f.vel = dir * stale_dodge_speed(p, rules, level);
    }
    enter(f, S::AirDodge);
    stale_dodge(f, rules);
    if p.air_dodge_windup == 0 {
        ground_assist(f, p, stage);
    }
}

/// How long an air dodge lasts: `air_dodge_frames` for a neutral one; for a directional one, between the sideways length and the
/// straight down (or up) one by how steep it is.
/// How many frames an air dodge in direction `dir` lasts (`Vec2::ZERO` for a neutral one).
pub fn air_dodge_length(p: &FighterParams, dir: Vec2) -> u16 {
    if dir == Vec2::ZERO {
        return u16::from(p.air_dodge_frames);
    }
    let side = Fx::from_int(i32::from(p.air_dodge_dir_side_frames));
    let end = Fx::from_int(i32::from(if dir.y < Fx::ZERO {
        p.air_dodge_dir_down_frames
    } else {
        p.air_dodge_dir_up_frames
    }));
    // y squared runs 0 (sideways) .. 0.5 (diagonal) .. 1 (straight), which tracks the angle closely.
    let steep = dir.y * dir.y;
    (side + (end - side) * steep + Fx::HALF)
        .floor_int()
        .clamp(1, 255) as u16
}

/// Landing lag of a directional dodge: the most just after the slingshot, less the later it lands.
fn dodge_landing_lag(f: &Fighter, p: &FighterParams) -> u8 {
    let since = f.state_frame.saturating_sub(u16::from(p.air_dodge_windup));
    let less = since / u16::from(p.air_dodge_landing_step.max(1));
    let lag = u16::from(p.waveland_lag).saturating_sub(less);
    lag.max(u16::from(p.air_dodge_dir_landing_min)).min(255) as u8
}

/// The slingshot at the start of a directional air dodge: a short drift away from the chosen direction sideways, and up for a downward
/// dodge (so a dodge cannot touch down at once), before the dodge itself.
fn sling_velocity(p: &FighterParams, dir: Vec2) -> Vec2 {
    let speed = p.air_dodge_speed * p.air_dodge_sling;
    let up = if dir.y < Fx::ZERO { -dir.y } else { Fx::ZERO };
    Vec2::new(-dir.x * speed, up * speed)
}

/// Ground assist: a downward dodge that begins just above a surface counts as touching it.
fn ground_assist(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    if f.dodge_dir.y <= -p.wavedash_min_down {
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
    f.lag = dodge_landing_lag(f, p);
    land(f, p, stage, platform);
    // Slide speed scales smoothly with how horizontal the dodge was.
    f.vel.x = f.dodge_dir.x * p.waveland_speed;
    enter(f, S::WaveLand);
}

/// A directional air dodge's speed at staleness `level`: it travels less the staler it is.
fn stale_dodge_speed(p: &FighterParams, rules: &Ruleset, level: u8) -> Fx {
    let keep = (Fx::ONE - rules.dodge_stale_air_cut.mul_int(i32::from(level))).max(Fx::ZERO);
    p.air_dodge_speed * keep
}

fn air_dodge(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) -> Option<u8> {
    let neutral = f.dodge_dir == Vec2::ZERO;
    let intangible_end = if neutral {
        p.air_dodge_intangible_end
    } else {
        p.air_dodge_dir_intangible_end
    };
    if within(f.state_frame, p.air_dodge_intangible_start, intangible_end) {
        f.invuln = f.invuln.max(1);
    }
    if neutral {
        if air_move(f, p, stage) {
            enter_landing(f, p.air_dodge_landing_lag);
            return None;
        }
    } else {
        let windup = u16::from(p.air_dodge_windup);
        if f.state_frame < windup {
            // Still in the slingshot: the velocity set when the dodge began holds.
        } else if f.state_frame == windup && windup > 0 {
            f.vel = f.dodge_dir * stale_dodge_speed(p, rules, f.dodge_level);
            ground_assist(f, p, stage);
            if f.state != S::AirDodge {
                return None;
            }
        } else if f.state_frame >= u16::from(p.air_dodge_fall_frame) {
            // The dodge is spent: the fighter falls without control until it ends.
            f.vel.x = approach(f.vel.x, Fx::ZERO, p.air_friction);
            f.vel.y = (f.vel.y - p.gravity).max(-p.max_fall_speed);
        } else {
            f.vel = f.vel * p.air_dodge_decay;
        }
        if let Some(i) = air_integrate(f, p, stage) {
            if f.dodge_dir.y <= -p.wavedash_min_down {
                start_waveland(f, p, stage, i);
            } else {
                // Too horizontal to be a wavedash: a plain air dodge that happens to land.
                let lag = dodge_landing_lag(f, p);
                land(f, p, stage, i);
                enter_landing(f, lag);
            }
            return None;
        }
    }
    if f.state_frame >= air_dodge_length(p, f.dodge_dir) {
        enter(f, S::Airborne);
        return None;
    }
    if f.state_frame >= u16::from(p.air_dodge_ledge_frame) {
        return ledge_request(f, p, stage);
    }
    None
}

fn landing(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    // The landing's momentum bleeds off at `landing_friction` whatever the stick does (holding it does not carry the speed).
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.landing_friction);
    if slide_on_platform(f, p, stage) && f.state_frame >= u16::from(f.lag) {
        exit_landing(f);
    }
}

fn wave_land(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    f.vel.x = f.vel.x * p.waveland_friction;
    // A wavedash stops at the edge of the platform instead of sliding off it.
    if let Some(plat) = collision::platform(stage, f.platform) {
        let next = f.pos.x + f.vel.x;
        if next < plat.left || next > plat.right {
            f.pos.x = if next < plat.left {
                plat.left
            } else {
                plat.right
            };
            f.vel.x = Fx::ZERO;
        }
    }
    if slide_on_platform(f, p, stage) && f.state_frame >= u16::from(f.lag) {
        enter(f, S::Idle);
    }
}

// ---- Shield ----------------------------------------------------------------------------------

fn shield(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage, rules: &Ruleset) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    f.shield_hp -= rules.shield_deplete;
    if f.shield_hp <= Fx::ZERO {
        break_shield(f, rules);
        return;
    }
    let buffer = p.input_buffer;
    // After blocking a hit the shield stays up and the fighter cannot act until the stun runs out. A grab pressed meanwhile (attack or
    // grab: both are the shield grab) is the reference's post-shieldstun shield grab: the shield stays up `shield_grab_delay` frames
    // more before it starts (sim 109).
    if f.shield_stun > 0 {
        f.shield_stun -= 1;
        // (Attack with the stick up is the up smash out of the shield, not a grab.)
        let up = f.history[0].stick_y >= STICK_DOWN;
        if f.shield_grab == 0
            && (f.pressed_within(buttons::GRAB, 1) || (!up && f.pressed_within(buttons::ATTACK, 1)))
        {
            f.shield_grab = 1;
        }
        return;
    }
    if f.shield_grab == 1 && rules.shield_grab_delay > 0 {
        // This frame is the first of the extra shield frames.
        f.shield_grab = 2;
        f.shield_stun = rules.shield_grab_delay - 1;
        return;
    }
    if f.shield_grab != 0 {
        f.shield_grab = 0;
        f.buffer_used |= buttons::ATTACK | buttons::GRAB;
        begin_attack(f, MoveId::Grab);
        return;
    }
    // Let go of: the shield drops, once it has been up `shield_min_frames`.
    if !f.held(buttons::SHIELD) && f.state_frame >= u16::from(rules.shield_min_frames) {
        enter(f, S::ShieldRelease);
        return;
    }
    // Holding an item (E4): attack or grab out of the shield throws it by the stick (the reference's shield toss), forward with the
    // stick in the middle.
    if f.holding != NONE && (f.take(buttons::ATTACK, buffer) || f.take(buttons::GRAB, buffer)) {
        let throw = item_throw_by_stick(f, p, false, false, true);
        start_item_throw(f, p, rules, throw);
        return;
    }
    // Straight out of the shield (E37, the reference's, owner decision 56): special with the stick up is the grounded up special, and
    // attack with the stick (or the attack stick) up the up smash, with no jump first and no shield drop. They come before the jump,
    // so a jump pressed with them does not take the press.
    if up_out_of_ground(f, p, weapon, false) {
        return;
    }
    // Special in the shield on a pass-through platform drops through it (owner decision 2026-10-14, sim 112; it was down in the
    // shield, which kept spot dodges off platforms). With the stick up it is the up special, above.
    if on_pass_through(f, stage) && f.take(buttons::SPECIAL, buffer) {
        f.platform = NONE;
        f.platform_ignore = p.platform_ignore_frames;
        f.vel = Vec2::new(Fx::ZERO, -p.shield_drop_speed);
        enter(f, S::ShieldDrop);
    } else if f.take(buttons::JUMP, buffer) {
        enter(f, S::JumpSquat);
    } else if f.take(buttons::ATTACK, buffer) || f.take(buttons::GRAB, buffer) {
        // Attack or grab out of the shield is a shield grab.
        begin_attack(f, MoveId::Grab);
    } else if f.flick_x(TAP_BUFFER) != 0 {
        // A flick sideways rolls that way.
        f.dodge_dir = Vec2::new(Fx::from_int(i32::from(f.flick_x(TAP_BUFFER))), Fx::ZERO);
        enter(f, S::Roll);
        stale_dodge(f, rules);
    } else if f.hard_down(TAP_BUFFER) {
        enter(f, S::SpotDodge);
        stale_dodge(f, rules);
    }
}

/// Letting go of the shield: a few frames (`shield_release_frames`, the reference's 11) before anything else can be done, except a
/// jump, the up special or the up smash, which cut it short at any time, as they come straight out of the shield (the reference's;
/// sim 109). A grab cannot: a shield grab has to be made with the shield still up.
fn shield_release(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if up_out_of_ground(f, p, weapon, false) {
        return;
    }
    if f.take(buttons::JUMP, p.input_buffer) {
        enter(f, S::JumpSquat);
        return;
    }
    if f.state_frame >= u16::from(p.shield_release_frames) {
        enter(f, S::Idle);
    }
}

/// True if `frame` is within `start..=end`.
fn within(frame: u16, start: u8, end: u8) -> bool {
    frame >= u16::from(start) && frame <= u16::from(end)
}

/// Marks the dodge just begun as a staling one (see `Fighter::dodge_stale`): it is made with the staleness so far, and adds a level.
fn stale_dodge(f: &mut Fighter, rules: &Ruleset) {
    f.dodge_level = f.dodge_stale;
    f.dodge_stale = f.dodge_stale.saturating_add(1).min(rules.dodge_stale_max);
    f.dodge_clock = 0;
}

/// A stale dodge's timeline: its frame `frame` (as it plays, slowed by `percent` per staleness level) is this frame of the fresh
/// dodge.
fn stale_frame(frame: u16, percent: u8, level: u8) -> u16 {
    frame * 100 / (100 + u16::from(percent) * u16::from(level))
}

/// A dodge of `frames` frames made at staleness `level`, slowed by `percent` per level.
fn stale_length(frames: u8, percent: u8, level: u8) -> u16 {
    (u16::from(frames) * (100 + u16::from(percent) * u16::from(level))).div_ceil(100)
}

/// How many frames later a stale dodge's intangibility starts: `full` when fully stale, in between by level (rounded).
fn stale_shift(full: u8, level: u8, rules: &Ruleset) -> u8 {
    let max = u16::from(rules.dodge_stale_max.max(1));
    ((u16::from(full) * u16::from(level) + max / 2) / max) as u8
}

/// A roll out of a shield: intangible for part of it, moving along the ground for part of it. A stale one (`dodge_level`) plays
/// slower and is intangible later.
fn roll(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    let dir = f.dodge_dir.x.signum_int();
    let back = dir != i32::from(f.facing);
    let level = f.dodge_level;
    let percent = if back {
        rules.dodge_stale_back_roll
    } else {
        rules.dodge_stale_roll
    };
    let shift = stale_shift(rules.dodge_stale_roll_shift, level, rules);
    if within(
        f.state_frame,
        p.roll_intangible_start.saturating_add(shift),
        p.roll_intangible_end,
    ) {
        f.invuln = f.invuln.max(1);
    }
    if within(
        stale_frame(f.state_frame, percent, level),
        p.roll_move_start,
        p.roll_move_end,
    ) {
        f.vel.x = p.roll_speed.mul_int(dir);
    } else {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    }
    slide_kept_on_platform(f, p, stage);
    // Rolling backward (away from the way the fighter faces) takes longer.
    let frames = if back {
        p.roll_back_frames
    } else {
        p.roll_frames
    };
    if f.state_frame >= stale_length(frames, percent, level) {
        enter(f, S::Idle);
    }
}

/// A dodge in place out of a shield. A stale one (`dodge_level`) lasts longer and is intangible later.
fn spot_dodge(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    let level = f.dodge_level;
    let shift = stale_shift(rules.dodge_stale_spot_shift, level, rules);
    if within(
        f.state_frame,
        p.spot_intangible_start.saturating_add(shift),
        p.spot_intangible_end,
    ) {
        f.invuln = f.invuln.max(1);
    }
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    slide_kept_on_platform(f, p, stage);
    if f.state_frame >= stale_length(p.spot_dodge_frames, rules.dodge_stale_spot, level) {
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
    let lag = plain_landing_lag(f, p);
    if air_move(f, p, stage) {
        enter_landing(f, lag);
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
    f.ledge_invuln = if f.ledge_grab_count == 1 {
        first_grab_invuln(f, p)
    } else {
        0
    };
    f.short_hop = false;
    f.pos = collision::ledge_hang_pos(l, p);
    f.vel = Vec2::ZERO;
    f.facing = -l.side;
    // Grabbing a ledge refreshes airborne options (and the once-per-airtime flags).
    f.air_jumps_left = p.air_jumps;
    f.air_flags = 0;
    f.wall_jumps = 0;
    f.air_dodge_used = false;
    f.fast_fall = false;
    enter(f, S::LedgeHang);
}

/// Ledge intangibility on the first grab since landing or being hit: more after a long time in the air, less at high damage, on
/// top of the grab itself (see `FighterParams::ledge_invuln_airtime`).
fn first_grab_invuln(f: &Fighter, p: &FighterParams) -> u8 {
    let air = u32::from(f.airtime.min(300));
    let percent = f.percent.floor_int().clamp(0, 120) as u32;
    let from_air = u32::from(p.ledge_invuln_airtime) * air / 300;
    let from_damage = u32::from(p.ledge_invuln_damage) * (120 - percent) / 120;
    let total = (from_air + from_damage).max(u32::from(p.ledge_invuln_min))
        + u32::from(p.ledge_grab_frames);
    total.min(u32::from(u8::MAX)) as u8
}

/// A ledge option's intangibility on this grab: full on the first since landing or being hit, then cut, then none.
fn ledge_option_invuln(f: &Fighter, p: &FighterParams, frames: u8) -> u8 {
    let percent = match f.ledge_grab_count {
        0 | 1 => 100,
        2 => u32::from(p.ledge_option_decay_2),
        3 => u32::from(p.ledge_option_decay_3),
        _ => 0,
    };
    (u32::from(frames) * percent / 100) as u8
}

/// Knocks the current occupant off a ledge that someone else just grabbed.
pub fn trump(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    let side = collision::ledge_side(stage, f.ledge);
    f.ledge = NONE;
    f.ledge_invuln = 0;
    f.ledge_cooldown = p.ledge_regrab_cooldown;
    f.platform = NONE;
    f.vel = Vec2::new(p.ledge_trump_vx.mul_int(i32::from(side)), p.ledge_trump_vy);
    enter(f, S::Airborne);
}

fn release_ledge(f: &mut Fighter, p: &FighterParams) {
    f.ledge = NONE;
    f.ledge_cooldown = p.ledge_regrab_cooldown;
}

fn ledge_hang(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage) {
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

    if f.state_frame >= p.ledge_hang_max {
        // Hung on too long: the fighter lets go.
        release_ledge(f, p);
        enter(f, S::Airborne);
        return;
    }
    // The grab itself: nothing can be done until it is over (a press made now is buffered).
    if f.state_frame < u16::from(p.ledge_grab_frames) {
        return;
    }
    let buffer = p.input_buffer;
    // Options in priority order (jump wins over attack). Each has its own intangibility, which replaces the hang's.
    if f.take(buttons::JUMP, buffer) {
        let invuln = ledge_option_invuln(f, p, p.ledge_jump_intangible);
        release_ledge(f, p);
        f.ledge_invuln = 0;
        f.invuln = f.invuln.max(invuln);
        f.vel = Vec2::new(
            p.ledge_jump_dx.mul_int(-i32::from(l.side)),
            p.ledge_jump_velocity,
        );
        enter(f, S::LedgeJump);
    } else if f.take(buttons::SHIELD, buffer) {
        let invuln = ledge_option_invuln(f, p, p.ledge_roll_intangible);
        get_up(
            f,
            stage,
            &l,
            p.ledge_roll_dx,
            S::LedgeGetUp,
            p.ledge_roll_frames,
            invuln,
        );
    } else if f.take(buttons::ATTACK, buffer) {
        let intangible = weapon.get(MoveId::LedgeAttack as u8).intangible;
        let invuln = ledge_option_invuln(f, p, intangible);
        get_up(
            f,
            stage,
            &l,
            p.ledge_attack_dx,
            S::LedgeAttack,
            p.ledge_attack_frames,
            invuln,
        );
        if f.state == S::LedgeAttack {
            f.move_id = MoveId::LedgeAttack as u8;
            f.hit_mask = 0;
        }
    } else if up >= threshold || toward >= threshold {
        let invuln = ledge_option_invuln(f, p, p.ledge_getup_intangible);
        get_up(
            f,
            stage,
            &l,
            p.ledge_getup_dx,
            S::LedgeGetUp,
            p.ledge_getup_frames,
            invuln,
        );
    } else if up <= -threshold || toward <= -threshold {
        // Letting go by pressing down or away ends the ledge intangibility at once.
        release_ledge(f, p);
        f.ledge_invuln = 0;
        enter(f, S::Airborne);
    }
}

/// Rising from a ledge jump: drifting, intangible at first, unable to act until `ledge_jump_frames` have passed.
fn ledge_jump(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    let lag = plain_landing_lag(f, p);
    if air_move(f, p, stage) {
        enter_landing(f, lag);
    } else if f.state_frame >= u16::from(p.ledge_jump_frames) {
        enter(f, S::Airborne);
    }
}

/// Climbs onto the stage into `state` (a get-up, roll or attack) lasting `frames`, intangible for `invuln` frames.
fn get_up(f: &mut Fighter, stage: &Stage, l: &Ledge, dx: Fx, state: S, frames: u8, invuln: u8) {
    f.pos = Vec2::new(l.x - dx.mul_int(i32::from(l.side)), l.y);
    f.vel = Vec2::ZERO;
    f.ledge = NONE;
    f.ledge_invuln = 0;
    f.platform = collision::standing_on(stage, f.pos);
    if f.platform == NONE {
        enter(f, S::Airborne);
        return;
    }
    f.lag = frames;
    f.invuln = f.invuln.max(invuln);
    enter(f, state);
}

/// A trip (`Hitbox::trip`): the fighter sits down where it stands, intangible for `trip_invuln` frames, and gets up as from a
/// knockdown on the ruleset's trip timings (see [`knockdown`]), or its soft-trip timings with `soft` (E44, sim 117).
pub fn trip(f: &mut Fighter, rules: &Ruleset, soft: bool) {
    enter(f, S::Knockdown);
    f.tripped = true;
    f.trip_soft = soft;
    f.vel = Vec2::ZERO;
    f.kb_vel = Vec2::ZERO;
    f.launch_pending = false;
    f.hitstun = 0;
    // (Through the hit's hitlag, then `trip_invuln` frames: the count goes down at the start of each frame, before its hits.)
    f.invuln = f.invuln.max(rules.trip_invuln.saturating_add(1));
}

/// Buries a fighter (a bury hit, E5) for `frames` frames: no action, no speed, nothing kept yet.
pub fn bury(f: &mut Fighter, frames: u16) {
    enter(f, S::Buried);
    f.bury_timer = frames.max(1);
    f.bury_set = false;
    f.bury_kb = Fx::ZERO;
    f.bury_angle = 0;
    f.vel = Vec2::ZERO;
    f.kb_vel = Vec2::ZERO;
    f.launch_pending = false;
    f.hitstun = 0;
    f.fast_fall = false;
}

/// Buried: the timer runs down a frame at a time plus what mashing takes off (the shared mash, as a hold's). At 0 the fighter comes
/// out: launched by a kept knockback if a weaker hit left one, else up with `bury_exit_lag` frames of lag (a rebound, on the spot:
/// the pop-out's height is not published, gap 9 [E]). No invincibility either way (the reference's).
fn buried(f: &mut Fighter, p: &FighterParams, rules: &Ruleset) {
    f.vel = Vec2::ZERO;
    let cut = crate::grab::mash_cut(f, rules);
    f.bury_timer = f.bury_timer.saturating_sub(1 + cut);
    if f.bury_timer > 0 {
        return;
    }
    if f.bury_set {
        let kb = f.bury_kb;
        f.bury_set = false;
        f.bury_kb = Fx::ZERO;
        enter(f, S::Hitstun);
        f.hitstun = crate::combat::hitstun_frames(kb, rules.hitstun_mult).max(1);
        f.launch_kb = kb;
        f.launch_angle = f.bury_angle;
        f.launch_set = false;
        f.launch_older = false;
        f.link = false;
        f.pull = false;
        f.kb_vel = Vec2::ZERO;
        f.tumble = false;
        f.hits_taken = 1;
        apply_launch(f, p, rules);
    } else {
        enter(f, S::Rebound);
        f.lag = rules.bury_exit_lag;
    }
}

/// Shuts a fighter in an egg (E8, `moves::Encase`), laid by body `by_char`'s move `by_move` facing `facing`: `percent` is the fighter's
/// percent before the throw and `added` what the throw added (it cuts the egg's time like any hit on it).
pub fn encase(
    f: &mut Fighter,
    e: &Encase,
    percent: Fx,
    added: Fx,
    by_char: u8,
    by_move: u8,
    facing: i8,
) {
    enter(f, S::Encased);
    f.encase_timer = e.frames(percent) - e.hit_cut * added;
    f.encase_age = 0;
    f.encase_char = by_char;
    f.encase_move = by_move;
    f.vel = Vec2::new(e.launch.0.mul_int(i32::from(facing)), e.launch.1);
    if f.vel.y > Fx::ZERO {
        f.platform = NONE;
    }
    f.kb_vel = Vec2::ZERO;
    f.launch_pending = false;
    f.hitstun = 0;
    f.fast_fall = false;
    f.hop_boost = 0;
}

/// In an egg (E8): the timer runs down a frame at a time, and by the egg's mash numbers on a frame with a new button press or a stick
/// flick (the shared mash reading); a flick also shifts the egg (sideways, and up for a hop, on the ground; barely in the air). The egg
/// falls with the fighter's own gravity and fall speed, slides to a stop on its traction, lands and stays an egg (no ledge, no tech).
/// At 0 the fighter breaks out into the air at the egg's `exit_vy`, intangible for `exit_intangible` frames (its midair jumps kept
/// [E: gap 8]). An egg whose numbers cannot be found (no `Encase` on the move that laid it) breaks at once.
fn encased(f: &mut Fighter, p: &FighterParams, stage: &Stage, e: Option<&Encase>) {
    let Some(e) = e else {
        break_out(f, Fx::ZERO, 0);
        return;
    };
    f.encase_age = f.encase_age.saturating_add(1);
    let (button, stick) = crate::grab::mash_inputs(f);
    let mut cut = Fx::ONE;
    if button {
        cut += e.mash_button;
    }
    if stick {
        cut += e.mash_stick;
    }
    f.encase_timer -= cut;
    if f.encase_timer <= Fx::ZERO {
        break_out(f, e.exit_vy, e.exit_intangible);
        return;
    }
    if stick {
        let input = f.history[0];
        let side = if x_active(input) {
            i32::from(input.stick_x.signum())
        } else {
            0
        };
        let up = input.stick_y >= STICK_THRESHOLD;
        let (sx, sy) = if f.grounded() { e.shift } else { e.air_shift };
        f.vel.x += sx.mul_int(side);
        if up {
            if f.grounded() {
                f.vel.y = sy;
                f.platform = NONE;
            } else {
                f.vel.y += sy;
            }
        }
    }
    if f.grounded() {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
        f.vel.y = Fx::ZERO;
        f.touched_wall = collision::move_x(stage, p, &mut f.pos, f.vel.x);
        if f.touched_wall {
            f.vel.x = Fx::ZERO;
        }
        // Rolled or shifted off its platform: it falls.
        if collision::platform(stage, f.platform)
            .is_none_or(|pl| f.pos.x < pl.left || f.pos.x > pl.right)
        {
            f.platform = NONE;
        }
        return;
    }
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.air_friction);
    f.vel.y = (f.vel.y - p.gravity).max(-p.max_fall_speed);
    if let Some(i) = air_integrate(f, p, stage) {
        land(f, p, stage, i);
    }
}

/// Out of an egg: into the air at `vy`, intangible for `intangible` frames.
fn break_out(f: &mut Fighter, vy: Fx, intangible: u8) {
    f.encase_timer = Fx::ZERO;
    f.encase_age = 0;
    f.encase_char = 0;
    f.encase_move = 0;
    f.platform = NONE;
    f.vel.y = vy;
    f.invuln = f.invuln.max(intangible);
    enter(f, S::Airborne);
}

/// Lying on the ground after a hard landing (or sitting after a trip). After a short while the fighter chooses a get-up: attack,
/// roll sideways, or stand (stick up, jump, or just waiting).
fn knockdown(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, stage: &Stage, rules: &Ruleset) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    let (lag, max) = if f.tripped && f.trip_soft {
        (rules.slip_lag, rules.slip_max)
    } else if f.tripped {
        (rules.trip_lag, rules.trip_max)
    } else {
        (rules.knockdown_lag, rules.knockdown_max)
    };
    if f.state_frame < u16::from(lag) {
        return;
    }
    let flick = f.flick_x(TAP_BUFFER);
    if f.take(buttons::ATTACK, p.input_buffer) {
        begin_attack(f, MoveId::GetUpAttack);
        f.invuln = f
            .invuln
            .max(weapon.get(MoveId::GetUpAttack as u8).intangible);
    } else if flick != 0 {
        f.dodge_dir = Vec2::new(Fx::from_int(i32::from(flick)), Fx::ZERO);
        enter(f, S::Roll);
    } else if f.history[0].stick_y >= STICK_DOWN
        || f.take(buttons::JUMP, p.input_buffer)
        || f.state_frame >= u16::from(max)
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

fn ledge_recover(f: &mut Fighter) {
    // The option's length was set when it began (`get_up`); its intangibility runs on `invuln`.
    if f.state_frame >= u16::from(f.lag) {
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

/// Holding a fighter: a stick direction throws it that way, attack pummels it. (A throw with `intangible` frames is intangible from
/// its first frame: a carry's lift, E7.)
fn grabbing(f: &mut Fighter, weapon: &Weapon) {
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
        let mv = weapon.get(id as u8);
        if mv.intangible > 0 && mv.intangible_start(false) == 0 {
            f.invuln = f.invuln.max(mv.intangible);
        }
    } else if f.take(buttons::ATTACK, ATTACK_BUFFER) {
        begin_attack(f, MoveId::Pummel);
    }
}

/// Carrying a held fighter (E7, see `moves::Carry`; the move that lifted it is still `move_id`): walking only, at the carry's share of
/// the walk; `carry_jumps` jumps in all (on the ground the body's jump squat, counted in `lag`, then a full or short hop as the button
/// is held; in the air a midair jump); off an edge or after a jump it falls with the body's air physics, landing straight back into the
/// carry [E: the reference's carry landing is not published]; attack throws (by the stick); a ledge only if the carry allows it. The
/// held fighter is pinned and its clock runs in `grab::update`.
fn carrying(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
) -> Option<u8> {
    let Some(carry) = weapon.get(f.move_id).carry else {
        // (Not a carry's move: let go, as any hold that is out of step; `grab::update` frees the pair.)
        enter(f, if f.grounded() { S::Idle } else { S::Airborne });
        return None;
    };
    let buffer = p.input_buffer;
    if f.take(buttons::ATTACK, buffer) {
        let (aim, _) = attack_aim(f, p);
        let back = x_active(aim) && (aim.stick_x > 0) != (f.facing > 0);
        let slot = carry.throw_for(aim.stick_y >= STICK_DOWN, aim.stick_y <= -STICK_DOWN, back);
        f.lag = 0;
        begin_attack(f, MoveId::from_index(slot));
        return None;
    }
    let input = f.history[0];
    if !f.grounded() {
        if f.carry_jumps > 0 && f.take(buttons::JUMP, buffer) {
            // A midair jump (the carry's only one, if it has not jumped from the ground).
            f.carry_jumps -= 1;
            f.vel.y = p.air_jump_velocity;
            f.vel.x = if x_active(input) {
                input.stick_x_fx() * p.air_speed * Fx::from_ratio(i32::from(p.air_jump_drift), 100)
            } else {
                Fx::ZERO
            };
            f.hop_boost = 0;
            f.fast_fall = false;
        }
        if air_move(f, p, stage) {
            f.lag = 0;
            return None;
        }
        return if carry.ledge {
            ledge_request(f, p, stage)
        } else {
            None
        };
    }
    if f.lag > 0 {
        // The jump squat: no friction (the walk's speed goes into the jump), then off the ground.
        f.lag -= 1;
        if carry_on_ground(f, p, stage, rules) && f.lag == 0 {
            f.platform = NONE;
            f.fast_fall = false;
            f.vel.x = air_momentum(f.vel.x, p, rules);
            let full = f.held(buttons::JUMP);
            f.short_hop = !full;
            if full && p.hop_burst_frames > 0 {
                f.vel.y = p.hop_burst_velocity;
                f.hop_boost = p.hop_burst_frames;
            } else {
                f.vel.y = if full {
                    p.full_hop_velocity
                } else {
                    p.short_hop_velocity
                };
            }
        }
        return None;
    }
    if f.carry_jumps > 0 && f.take(buttons::JUMP, buffer) {
        f.carry_jumps -= 1;
        f.lag = p.jump_squat_frames.max(1);
        carry_on_ground(f, p, stage, rules);
        return None;
    }
    // The walk at `walk` percent of the body's (speed and both accelerations), turning at once with the stick [E].
    let k = Fx::from_ratio(i32::from(carry.walk), 100);
    let mut q = *p;
    q.walk_speed = p.walk_speed * k;
    q.walk_accel = p.walk_accel * k;
    q.walk_accel_stick = p.walk_accel_stick * k;
    if x_active(input) {
        let dir: i8 = if input.stick_x > 0 { 1 } else { -1 };
        f.facing = dir;
        let tilt = i32::from(
            input
                .stick_x
                .unsigned_abs()
                .min(STICK_THRESHOLD.unsigned_abs()),
        );
        let target = (Fx::from_ratio(tilt, i32::from(STICK_THRESHOLD)) * q.walk_speed)
            .mul_int(i32::from(dir));
        f.vel.x = approach(f.vel.x, target, ease_rate(f.vel.x, target, &q, input));
    } else {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    }
    carry_on_ground(f, p, stage, rules);
    None
}

/// Moves a carrying fighter along its platform; walking off the end takes it (and the held fighter) into the air, still carrying,
/// with the ground speed reduced as a jump's. Returns false if it left the ground.
fn carry_on_ground(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) -> bool {
    f.touched_wall = collision::move_x(stage, p, &mut f.pos, f.vel.x);
    if f.touched_wall {
        f.vel.x = Fx::ZERO;
    }
    let on = collision::platform(stage, f.platform)
        .is_some_and(|plat| f.pos.x >= plat.left && f.pos.x <= plat.right);
    if !on {
        f.platform = NONE;
        f.vel.y = Fx::ZERO;
        f.vel.x = air_momentum(f.vel.x, p, rules);
        f.lag = 0;
    }
    on
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
    f.grab_start = 0;
    f.grab_immune = immunity;
    if f.grounded() {
        enter(f, S::Idle);
    } else {
        enter(f, S::Airborne);
    }
}

pub fn begin_attack(f: &mut Fighter, id: MoveId) {
    // A new attack (not a move switching to its next part) remembers whether it began in the air and forgets the last one's angle.
    if f.state != S::Attack {
        f.move_air = !f.grounded();
        f.steer = 0;
        // The count of attack presses starts again with this attack: the press that started it, if attack started it (it may have
        // waited in the input buffer, and been let go already).
        let held = f.held(buttons::ATTACK);
        let by_attack = held || f.pressed_within(buttons::ATTACK, 10);
        f.presses = u8::from(by_attack);
        f.releases = u8::from(by_attack && !held);
        // A target lock belongs to one attack.
        f.locked = false;
        f.lock = Vec2::ZERO;
    }
    f.move_id = id as u8;
    f.move_y0 = f.pos.y;
    f.hit_mask = 0;
    f.b_reversed = false;
    f.aim = 0;
    enter(f, S::Attack);
    f.charge = 0;
}

/// E37: special with the stick up starts the grounded up special, and attack aimed up the up smash, straight out of a shield or a
/// jump squat (for every fighter, the reference's). Out of a shield attack with the stick or the attack stick up is enough (attack
/// alone there is the shield grab); out of a jump squat (`smash_only`) it takes an up smash input (a flick up, the strong button or
/// the attack stick), since attack with the stick held up is the short-hop up air. Returns whether one started.
fn up_out_of_ground(f: &mut Fighter, p: &FighterParams, weapon: &Weapon, smash_only: bool) -> bool {
    let buffer = p.input_buffer;
    let up = f.history[0].stick_y >= STICK_DOWN;
    // (Out of a jump squat only a press made after the jump's own counts: one made with the jump is the short-hop aerial or aerial
    // special it always was, the house's short-hop input.)
    let since_jump = u8::try_from(f.state_frame).unwrap_or(u8::MAX);
    let fresh = |f: &Fighter, b: u16| !smash_only || f.pressed_within(b, since_jump);
    if up
        && fresh(f, buttons::SPECIAL)
        && f.buffered(buttons::SPECIAL, buffer)
        && start_special(f, weapon)
    {
        f.buffer_used |= buttons::SPECIAL;
        return true;
    }
    // (Holding an item, attack throws it instead: E4.)
    if f.holding == crate::state::NONE
        && fresh(f, buttons::ATTACK)
        && f.buffered(buttons::ATTACK, buffer)
    {
        let (aim, strong) = attack_aim(f, p);
        let smash = strong || f.hard_up(SMASH_FLICK_BUFFER);
        if aim.stick_y >= STICK_DOWN && (smash || !smash_only) {
            f.buffer_used |= buttons::ATTACK;
            begin_attack(f, MoveId::USmash);
            return true;
        }
    }
    false
}

/// Starts a move's follow-up (its `next`, or a script's `goto`); one with `Move::keep_charge` keeps the charge its first part had (E36).
fn follow_up(f: &mut Fighter, weapon: &Weapon, next: u8) {
    let charge = f.charge;
    begin_attack(f, MoveId::from_index(next));
    if weapon.get(next).keep_charge {
        f.charge = charge;
    }
}

/// The input an attack is aimed by: the movement stick, or, when the attack came from the attack stick (C-stick), its direction
/// (the frame the attack was pressed on had a C direction), with the strong flag a C-stick attack carries on the ground. Out of a
/// run the attack stick up or down is a smash and sideways the dash attack.
fn attack_aim(f: &Fighter, p: &FighterParams) -> (Input, bool) {
    // (The press may have waited in the input buffer.)
    let n = usize::from(p.input_buffer) + 1;
    let pressed_at = (0..n.min(f.history.len())).find(|&i| {
        f.history[i].pressed(buttons::ATTACK)
            && !f
                .history
                .get(i + 1)
                .is_some_and(|o| o.pressed(buttons::ATTACK))
    });
    let mut aim = f.history[0];
    if let Some(c) = pressed_at.map(|i| f.history[i].buttons & buttons::C_ANY) {
        if c != 0 {
            let axis = |neg: u16, pos: u16| -> i8 {
                if c & pos != 0 {
                    127
                } else if c & neg != 0 {
                    -127
                } else {
                    0
                }
            };
            aim.stick_x = axis(buttons::C_LEFT, buttons::C_RIGHT);
            aim.stick_y = axis(buttons::C_DOWN, buttons::C_UP);
            return (aim, true);
        }
    }
    (aim, f.held(buttons::STRONG))
}

fn start_ground_attack(f: &mut Fighter, p: &FighterParams) {
    // The strong-attack button is a flick made in advance: it turns a direction into a smash attack. (So is the attack stick.)
    let (input, strong) = attack_aim(f, p);
    let c_stick = strong && input != f.history[0];
    // A dash attack comes out of a dash or run, and also just after one, while the fighter is still sliding at dash speed (so letting
    // go of the stick does not turn the attack into a jab).
    let sliding_fast = f.vel.x.abs() > p.walk_speed * Fx::from_ratio(12, 10)
        && !c_stick
        && !x_active(input)
        && input.stick_y.unsigned_abs() < STICK_DOWN.unsigned_abs()
        && f.vel.x.signum_int() == i32::from(f.facing);
    // Out of the initial dash a smash attack can be used (a flick up or down, or the strong button), but never a tilt: any other
    // attack is the dash attack. (A sideways flick is the dash's own, so only the strong button makes a forward smash there.)
    let dash_smash = if f.state == S::Dash {
        if input.stick_y >= STICK_DOWN && (strong || f.hard_up(SMASH_FLICK_BUFFER)) {
            Some(MoveId::USmash)
        } else if input.stick_y <= -STICK_DOWN && (strong || f.hard_down(SMASH_FLICK_BUFFER)) {
            Some(MoveId::DSmash)
        } else if strong {
            Some(MoveId::FSmash)
        } else {
            None
        }
    } else {
        None
    };
    let id = if let Some(smash) = dash_smash {
        smash
    } else if (matches!(f.state, S::Dash | S::Run) && !(c_stick && input.stick_y != 0))
        || sliding_fast
    {
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
    } else if f.late_left > 0 && f.late_next > 0 {
        // Inside a jab's continuable window past its end (`Move::late_next`): the jab's next hit, not a new jab.
        MoveId::from_index(f.late_next - 1)
    } else {
        MoveId::Jab
    };
    begin_attack(f, id);
    // A forward tilt or smash with the stick tilted up or down (short of an up or down attack) is aimed that way.
    if matches!(id, MoveId::FTilt | MoveId::FSmash) {
        f.aim = if input.stick_y >= STICK_AIM {
            1
        } else if input.stick_y <= -STICK_AIM {
            -1
        } else {
            0
        };
    }
}

/// An aerial. As in the reference it ends a fast fall: the fighter falls at its normal top speed again until it fast falls anew.
fn start_air_attack(f: &mut Fighter, p: &FighterParams) {
    if f.fast_fall {
        f.fast_fall = false;
        f.vel.y = f.vel.y.max(-p.max_fall_speed);
    }
    let (input, _) = attack_aim(f, p);
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

/// Whether the attack being pressed is a smash input (a flick of the stick its way, the strong button or the attack stick): such an
/// attack is the smash attack, not an item pick-up (E4) [E: the reference picks items up with a tilt or jab input].
fn smash_input(f: &Fighter, p: &FighterParams) -> bool {
    let (input, strong) = attack_aim(f, p);
    strong
        || (input.stick_y >= STICK_DOWN && f.hard_up(SMASH_FLICK_BUFFER))
        || (input.stick_y <= -STICK_DOWN && f.hard_down(SMASH_FLICK_BUFFER))
        || (x_active(input) && f.flick_x(SMASH_FLICK_BUFFER) != 0)
}

/// The item throw (`items::THROW_*`) the stick asks for (E4): up, down, forward or back by the stick (the attack stick too for an attack),
/// a smash throw with a smash input; out of a dash or run (`running`) the dash throw unless up or down; with the stick in the middle a
/// forward throw, or for the grab button (`grab`) the drop (`ground`) or the aerial let-go.
fn item_throw_by_stick(
    f: &Fighter,
    p: &FighterParams,
    grab: bool,
    running: bool,
    ground: bool,
) -> u8 {
    use crate::items::*;
    let (input, strong) = if grab {
        (f.history[0], f.held(buttons::STRONG))
    } else {
        attack_aim(f, p)
    };
    let up = input.stick_y >= STICK_DOWN;
    let down = input.stick_y <= -STICK_DOWN;
    let side = x_active(input);
    let throw = if up {
        THROW_UP
    } else if down {
        THROW_DOWN
    } else if running && !grab {
        return THROW_DASH;
    } else if side {
        if (input.stick_x > 0) == (f.facing > 0) {
            THROW_FORWARD
        } else {
            THROW_BACK
        }
    } else if grab {
        return if ground { THROW_DROP } else { THROW_LET_GO };
    } else {
        THROW_FORWARD
    };
    let smash = strong
        || (up && f.hard_up(SMASH_FLICK_BUFFER))
        || (down && f.hard_down(SMASH_FLICK_BUFFER))
        || (side && f.flick_x(SMASH_FLICK_BUFFER) != 0);
    if smash {
        throw | THROW_SMASH
    } else {
        throw
    }
}

/// Starts an item throw (E4, `FighterState::ItemThrow`): a back throw turns the fighter round first (it throws the way it then faces)
/// [E: the reference turns with it]; the item leaves on the body's release frame.
fn start_item_throw(f: &mut Fighter, p: &FighterParams, rules: &Ruleset, throw: u8) {
    if throw & !crate::items::THROW_SMASH == crate::items::THROW_BACK {
        f.facing = -f.facing;
    }
    if f.fast_fall {
        f.fast_fall = false;
        f.vel.y = f.vel.y.max(-p.max_fall_speed);
    }
    f.item_throw = throw;
    enter(f, S::ItemThrow);
    let (release, _) = crate::items::throw_frames(p, rules, throw);
    if release <= 1 {
        f.item_order |= crate::items::ORDER_RELEASE;
    }
}

/// Starts picking up the item in reach (E4, `FighterState::ItemPickup`): the item system puts the nearest one in the hand this frame.
fn start_item_pickup(f: &mut Fighter) {
    enter(f, S::ItemPickup);
    f.item_order |= crate::items::ORDER_PICK;
}

/// Picking an item up: sliding to a stop on the ground for the ruleset's `item_pick_frames` [E].
fn item_pickup(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
    if !slide_on_platform(f, p, stage) {
        return;
    }
    if f.state_frame + 1 >= u16::from(rules.item_pick_frames.max(1)) {
        enter(f, S::Idle);
    }
}

/// An item throw, on the ground or in the air (E4): the item leaves on the body's release frame (`FighterParams::item_throws`) and
/// the throw lasts the ruleset's length for it (the reference's totals: control back on the frame after). On the ground the fighter
/// slides to a stop (over an edge it goes on in the air); in the air it drifts as usual and touching down ends it with the plain
/// landing lag [E].
fn item_throw(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) -> Option<u8> {
    let (release, total) = crate::items::throw_frames(p, rules, f.item_throw);
    if f.state_frame + 1 == u16::from(release) {
        f.item_order |= crate::items::ORDER_RELEASE;
    }
    let done = f.state_frame + 1 >= u16::from(total.max(1));
    if f.grounded() {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
        let frame = f.state_frame;
        if !slide_on_platform(f, p, stage) {
            // (Over the edge: the throw goes on in the air.)
            f.state = S::ItemThrow;
            f.state_frame = frame;
        } else if done {
            enter(f, S::Idle);
        }
        return None;
    }
    let lag = plain_landing_lag(f, p);
    if air_move(f, p, stage) {
        enter_landing(f, lag);
        return None;
    }
    if done {
        enter(f, S::Airborne);
    }
    None
}

/// Advances a move one frame. Returns the ledge it wants to grab, if the move can grab ledges.
fn attack(
    f: &mut Fighter,
    p: &FighterParams,
    weapon: &Weapon,
    stage: &Stage,
    rules: &Ruleset,
    world: &scripting::World,
) -> Option<u8> {
    // A switch decided last frame after the fighter moved (a search box's contact, a command grab's catch): the new move's first
    // frame is this one, keeping the speed (as a `goto`).
    if f.next_move != 0 {
        let next = f.next_move - 1;
        f.next_move = 0;
        begin_attack(f, MoveId::from_index(next));
        return None;
    }
    let mv = weapon.get(f.move_id);
    let id = MoveId::from_index(f.move_id);

    // A move with its own turnaround frame (`turn_at`) turns only then, with the stick held back far enough, and in the air its
    // momentum turns with it; the shared windows below do not apply to it.
    if let Some((at, stick)) = mv.turn_at {
        let input = f.history[0];
        let back = -input.stick_x_fx().mul_int(i32::from(f.facing));
        if f.state_frame == u16::from(at) && back > Fx::from_ratio(i32::from(stick), 100) {
            f.facing = -f.facing;
            if f.platform == NONE {
                f.vel.x = -f.vel.x;
            }
        }
    }
    // In the first frames of an up special, the stick pushed back turns it around (up and back reverses it).
    // (An aerial or smash-input version a special started as counts as that special.)
    let current = f.move_id;
    let as_special = |s: MoveId| {
        let sp = weapon.get(s as u8);
        id == s || sp.air_version == Some(current) || sp.smash_version == Some(current)
    };
    if as_special(MoveId::UpSpecial)
        && mv.turn_at.is_none()
        && f.state_frame <= u16::from(rules.special_turn_window)
    {
        let input = f.history[0];
        if x_active(input) && i32::from(input.stick_x.signum()) == -i32::from(f.facing) {
            f.facing = -f.facing;
        }
    }
    // A B-reverse: in the first frames of a neutral or side special, the stick pushed the other way turns the fighter round, and in the
    // air its momentum goes the other way too.
    if (as_special(MoveId::NSpecial) || as_special(MoveId::SideSpecial))
        && mv.turn_at.is_none()
        && f.state_frame <= u16::from(rules.special_reverse_window)
        && !f.b_reversed
    {
        let input = f.history[0];
        if x_active(input) && i32::from(input.stick_x.signum()) == -i32::from(f.facing) {
            f.facing = -f.facing;
            f.b_reversed = true;
            if f.platform == NONE {
                f.vel.x = -f.vel.x;
            }
        }
    }

    // Intangibility that starts later in the move (the frames still count from the move's first: `intangible` is the last one).
    let from = mv.intangible_start(f.move_air);
    if from > 0 && f.state_frame == u16::from(from) {
        f.invuln = f.invuln.max(mv.intangible.saturating_sub(from));
    }
    // Angling by the stick, read once.
    if f.state_frame == u16::from(mv.steer_frame) {
        f.steer = steer_degrees(f.history[0], f.facing, mv);
    }

    // A smash attack holds on its charge frame while the button stays held.
    let charging = mv.charge_at.is_some_and(|c| u16::from(c) == f.state_frame)
        && f.charge < rules.charge_frames
        && f.held(buttons::ATTACK);
    if charging {
        f.charge += 1;
        f.state_frame -= 1;
    }

    // A multi-hit move lets its hits land again at a steady rhythm (only the groups it re-arms: a final hit lands once).
    if let Some((start, every)) = mv.rehit {
        let start = u16::from(start);
        if f.state_frame >= start && (f.state_frame - start) % u16::from(every.max(1)) == 0 {
            f.hit_mask &= !crate::combat::group_bits(mv.rehit_groups);
        }
    }

    // A move can fire a projectile on one frame (the step applies the request).
    if let Some(spec) = mv.projectile {
        if f.state_frame == u16::from(spec.frame) {
            f.spawn_request = true;
        }
    }
    // ...and make an item in the hand, or use up the one there, on one frame (E4: the item system serves the order).
    if let Some((frame, item)) = mv.pull {
        if f.state_frame == u16::from(frame) {
            f.item_order |= crate::items::ORDER_PULL;
            f.item_spec = item;
        }
    }
    if mv.spend_item != 255 && f.state_frame == u16::from(mv.spend_item) {
        f.item_order |= crate::items::ORDER_SPEND;
    }
    // ...or throw the item in hand at the move's own speed and angle (E47, sim 117): on `again` an item held as the move began, on
    // `frame` the move's own (which starts the wait before the next can be made).
    if let Some(t) = mv.toss {
        let main = f.state_frame == u16::from(t.frame);
        let again = t.again != 255 && f.state_frame == u16::from(t.again);
        if (main || again) && f.holding != NONE {
            f.item_throw = crate::items::THROW_MOVE;
            f.item_order |= crate::items::ORDER_RELEASE;
            if main && t.wait > 0 {
                f.pull_wait = t.wait;
            }
        }
    }
    // A pocket (E42) started with an item in the hand puts that item away on its first catching frame; a release takes out what was
    // put away on its frame.
    if let Some(pc) = mv.pocket {
        if f.state_frame == u16::from(pc.start) && f.holding != NONE {
            f.item_order |= crate::items::ORDER_STOW;
        }
    }
    if let Some(rel) = mv.unpocket {
        if f.state_frame == u16::from(rel.frame) {
            f.item_order |= crate::items::ORDER_UNPOCKET;
        }
    }

    // A cancellable jab (`Move::cancel_from`, sim 115): from that frame, on the ground, any action the fighter could start standing
    // ends the move and starts now (the previous reference version's jab cancel). A walk does not, and neither does the jab itself
    // (its press carries on into the next hit below).
    if mv.cancel_from != 255 && f.grounded() && f.state_frame >= u16::from(mv.cancel_from) {
        let mut g = *f;
        enter(&mut g, S::Idle);
        ground(&mut g, p, weapon, stage, rules, world.pickable[world.me]);
        let acted = !matches!(g.state, S::Idle | S::Walk)
            && !(g.state == S::Attack && g.move_id == MoveId::Jab as u8);
        if acted {
            *f = g;
            return None;
        }
    }

    // A jab with an early continuable window carries on into its next hit as soon as attack is pressed in it.
    if let Some(next) = mv.next {
        if f.state_frame >= u16::from(mv.next_from)
            && f.grounded()
            && f.take(buttons::ATTACK, mv.next_window)
        {
            follow_up(f, weapon, next);
            return None;
        }
    }

    // The move's script steers the move: motion, branching into another move, projectiles, ending early.
    let mut script_motion = None;
    if let Some(program) = &mv.script {
        // (The kind `owned` counts: the projectile's tool, else the item the move pulls: E4.)
        let kind = match mv.projectile.map_or(0, |s| s.tool.kind) {
            0 => mv
                .pull
                .and_then(|(_, i)| weapon.items.get(usize::from(i)))
                .map_or(0, |it| it.kind),
            k => k,
        };
        let effects = scripting::run_fighter(program, f, world, &p.gauge, kind);
        if let Some(next) = effects.goto {
            follow_up(f, weapon, next);
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

    // A move's own soft-platform drop (E30): a few frames from `from`, and with `while_down` any frame from then with the stick down.
    if let Some(d) = mv.drop_through {
        let from = u16::from(d.from);
        if f.state_frame == from {
            f.platform_ignore = f.platform_ignore.max(d.frames);
        }
        if d.while_down && f.state_frame >= from && f.history[0].stick_y <= -STICK_DOWN {
            f.platform_ignore = f.platform_ignore.max(1);
        }
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
                .map(|m| shaped_motion(*m, mv, f.move_air, f.steer))
        });
    if let Some(m) = motion {
        // A motion along the ground follows the ground: it can leave it by going up, but never sinks into it.
        let vy = if f.grounded() {
            m.vy.max(Fx::ZERO)
        } else {
            m.vy
        };
        if mv.motion_drift > 0 && !f.grounded() {
            // A rise with air control (`Move::motion_drift`): the motion sets the vertical speed, the stick the horizontal.
            let mut q = *p;
            let k = Fx::from_ratio(i32::from(mv.motion_drift), 100);
            q.air_speed = p.air_speed * k;
            q.air_accel = p.air_accel * k;
            q.air_accel_stick = p.air_accel_stick * k;
            air_steer_x(f, &q);
            f.vel.y = vy;
        } else {
            f.vel = Vec2::new(m.vx.mul_int(i32::from(f.facing)), vy);
        }
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
            if let Some(other) = mv.on_land {
                // An aerial move with a grounded version goes on as it, on the same frame (`Move::on_land`).
                f.move_id = other;
                return None;
            }
            if mv.land_continue || frame < u16::from(mv.lands_from) {
                // (The move goes on along the ground.)
            } else if mv.landing_move.is_some() {
                // (A dive with a landing hit, such as a down air: the same rule as any aerial landing.)
                land_out_of_move(f, mv, frame, plain_landing_lag(f, p));
                return None;
            } else {
                enter_landing(f, mv.landing_lag);
                return None;
            }
        }
        // A dash along the ground that carries the fighter past the edge leaves the ground (and a move with an `off_edge` version
        // goes on as it, on the same frame).
        if let Some(plat) = collision::platform(stage, f.platform) {
            if f.pos.x < plat.left || f.pos.x > plat.right {
                f.platform = NONE;
                if let Some(other) = mv.off_edge {
                    f.move_id = other;
                }
            }
        }
    } else if id.is_aerial() || ((id.is_special() || id.is_ext()) && !f.grounded()) {
        let plain = plain_landing_lag(f, p);
        let q = move_air_physics(p, mv, frame);
        let fast_fall_ok = frame >= u16::from(mv.fast_fall_from);
        // (A stance's top rising speed holds the fighter's upward speed down before it moves.)
        if let Some(rise) = mv.air_max_rise {
            f.vel.y = f.vel.y.min(rise);
            f.hop_boost = 0;
        }
        let landed = air_move_with(f, &q, stage, fast_fall_ok);
        if landed {
            if let Some(other) = mv.on_land {
                // An aerial move with a grounded version goes on as it, on the same frame, keeping its frame and hits (`Move::on_land`).
                f.move_id = other;
                return None;
            }
        }
        if landed && !mv.land_continue && frame >= u16::from(mv.lands_from) {
            // Landing early or late in the move autocancels: only the normal landing lag.
            land_out_of_move(f, mv, frame, plain);
            return None;
        }
    } else {
        // A dash attack slides on traction (`dash_attack_friction`); other ground moves stop on the doubled `ground_friction`.
        let friction = if id == MoveId::DashAttack {
            p.dash_attack_friction
        } else {
            p.ground_friction
        };
        f.vel.x = approach(f.vel.x, Fx::ZERO, friction);
        let (kept_frame, kept_charge) = (f.state_frame, f.charge);
        if !slide_on_platform(f, p, stage) {
            // Off the edge: a move with an `off_edge` version goes on as it, on the same frame; any other ends.
            let other = mv.off_edge?;
            f.state = S::Attack;
            f.state_frame = kept_frame;
            f.charge = kept_charge;
            f.move_id = other;
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
        // A carry's lift goes on into the carry, if the fighter is still held (E7).
        if let Some(carry) = mv.carry {
            if f.grab_with != NONE {
                enter(f, S::Carrying);
                f.carry_jumps = carry.jumps;
                f.lag = 0;
                return None;
            }
        }
        // A jab continues into its next hit if attack was pressed shortly before the end.
        if let Some(next) = mv.next {
            if f.grounded() && f.take(buttons::ATTACK, mv.next_window) {
                follow_up(f, weapon, next);
                return None;
            }
        }
        if f.grounded() {
            enter(f, S::Idle);
            // A jab whose continuable window runs past its end keeps it open for a press (owner decision 58).
            if let Some((next, until)) = mv.late_next {
                let left = u16::from(until).saturating_sub(u16::from(mv.total_frames));
                if left > 0 {
                    f.late_next = next.saturating_add(1);
                    f.late_left = (left + 1).min(255) as u8;
                }
            }
        } else if mv.helpless_after {
            enter_helpless(f);
        } else {
            enter(f, S::Airborne);
        }
    }

    // Up specials can grab a ledge in mid-move, rising or not, from the frames their rules allow (in front and behind separately).
    if mv.catches_ledges() {
        let ledge = ledge_in_reach(f, p, stage)?;
        let behind = stage
            .ledges
            .get(usize::from(ledge))
            .is_some_and(|l| l.side == f.facing);
        // (A move with a fall rule catches a ledge only once it has come far enough down from where it began: `Move::ledge_drop`.)
        let fallen = f.move_y0 - f.pos.y;
        let deep_enough = mv
            .ledge_drop
            .is_none_or(|(front, back)| fallen >= if behind { back } else { front });
        // (A move that catches ledges only while not rising: `Move::ledge_falling_only`.)
        let falling_ok = !mv.ledge_falling_only || f.vel.y <= Fx::ZERO;
        if f.state_frame >= u16::from(mv.ledge_from(behind)) && deep_enough && falling_ok {
            return Some(ledge);
        }
    }
    None
}

/// How far the stick angles a move with `steer_frame` (see `Move::steer_degrees`): nothing within `steer_stick` percent of the
/// centre, then evenly more up to `steer_degrees` at the rim; positive toward the way the fighter faces.
fn steer_degrees(input: Input, facing: i8, mv: &Move) -> i8 {
    let sx = input.stick_x_fx().mul_int(i32::from(facing));
    let dead = Fx::from_ratio(i32::from(mv.steer_stick.min(99)), 100);
    let over = sx.abs() - dead;
    if over <= Fx::ZERO {
        return 0;
    }
    let share = (over / (Fx::ONE - dead)).min(Fx::ONE);
    let deg = share
        .mul_int(i32::from(mv.steer_degrees))
        .floor_int()
        .clamp(0, 90);
    (deg * sx.signum_int()) as i8
}

/// A data motion row as this use of the move runs it: its rise scaled by the grounded or aerial height multiplier, then turned by
/// the stick's angle (positive degrees lean it toward the front).
fn shaped_motion(m: Motion, mv: &Move, air: bool, steer: i8) -> Motion {
    let rise = if air { mv.rise_air } else { mv.rise_ground };
    let vy = m.vy * Fx::from_ratio(i32::from(rise), 100);
    let (vx, vy) = if steer != 0 {
        // (forward, up) turned clockwise by `steer` degrees.
        let a = Angle::from_degrees(i32::from(steer));
        let (c, s) = (trig::cos(a), trig::sin(a));
        (m.vx * c + vy * s, vy * c - m.vx * s)
    } else {
        (m.vx, vy)
    };
    Motion { vx, vy, ..m }
}

/// The body's air physics with a move's own overrides (`Move::air_gravity` and the rest, or the air block covering move frame
/// `frame`: `Move::air_blocks`), for a move running without a motion.
fn move_air_physics(p: &FighterParams, mv: &Move, frame: u16) -> FighterParams {
    let block = mv
        .air_blocks
        .iter()
        .find(|b| u16::from(b.from) <= frame && frame <= u16::from(b.to));
    let gravity = block.and_then(|b| b.gravity).or(mv.air_gravity);
    let max_fall = block.and_then(|b| b.max_fall).or(mv.air_max_fall);
    let friction = block.and_then(|b| b.friction).or(mv.air_friction);
    let drift = block.map_or(mv.air_drift, |b| b.drift);
    let accel = block.map_or(100, |b| b.accel);
    let mut q = *p;
    if let Some(g) = gravity {
        q.gravity = g;
    }
    if let Some(v) = max_fall {
        q.max_fall_speed = v;
        q.fast_fall_speed = q.fast_fall_speed.max(v);
    }
    if let Some(fr) = friction {
        q.air_friction = fr;
    }
    if drift != 100 {
        let k = Fx::from_ratio(i32::from(drift), 100);
        q.air_speed = p.air_speed * k;
        q.air_accel = p.air_accel * k;
        q.air_accel_stick = p.air_accel_stick * k;
    }
    if accel != 100 {
        let k = Fx::from_ratio(i32::from(accel), 100);
        q.air_accel = q.air_accel * k;
        q.air_accel_stick = q.air_accel_stick * k;
    }
    if let Some(b) = block {
        if let Some(v) = b.speed {
            q.air_speed = v;
        }
        if let Some(v) = b.accel_stick {
            q.air_accel_stick = v;
        }
    }
    q
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
    // An up special held up and back comes out the other way (see also `special_turn_window`).
    if id == MoveId::UpSpecial
        && mv.turn_at.is_none()
        && x_active(input)
        && i32::from(input.stick_x.signum()) == -i32::from(f.facing)
    {
        f.facing = -f.facing;
    }
    // With the gauge's powered mode running, a special with a powered version (`limit_version`) uses it instead and spends the gauge
    // at once, whether it hits or not. (Until a weapon has such a version nothing is spent: phase 2 of the Greatsword.)
    let (id, mv) = match mv.limit_version {
        Some(l) if f.gauge_mode > 0 => {
            crate::gauge::spend(f);
            (MoveId::from_index(l), weapon.get(l))
        }
        _ => (id, mv),
    };
    // Started in the air, a special with an aerial version uses it (its own slot).
    let (id, mv) = match mv.air_version {
        Some(v) if !f.grounded() => (MoveId::from_index(v), weapon.get(v)),
        _ => (id, mv),
    };
    // A special input like a smash attack (the stick flicked sideways, up or down, or the strong button) is remembered for its
    // script (the `flick` register), and picks the move's smash-input version if it has one (`Move::smash_version`, E3d).
    let flick = f.held(buttons::STRONG)
        || f.flick_x(SMASH_FLICK_BUFFER) != 0
        || f.hard_up(SMASH_FLICK_BUFFER)
        || f.hard_down(SMASH_FLICK_BUFFER);
    let (id, mv) = match mv.smash_version {
        Some(v) if flick => (MoveId::from_index(v), weapon.get(v)),
        _ => (id, mv),
    };
    begin_attack(f, id);
    f.special_flick = flick;
    if mv.intangible_start(f.move_air) == 0 {
        f.invuln = f.invuln.max(mv.intangible);
    }
    // Started in the air, a special may keep only part of the fighter's momentum.
    if !f.grounded() && mv.start_speed != 100 {
        f.vel.x = f.vel.x * Fx::from_ratio(i32::from(mv.start_speed), 100);
    }
    // ...and of its vertical speed (the reference's vertical multiplier on startup: a stall).
    if !f.grounded() && mv.start_speed_y != 100 {
        f.vel.y = f.vel.y * Fx::from_ratio(i32::from(mv.start_speed_y), 100);
        f.hop_boost = 0;
    }
    true
}

/// How many knockback frames this frame of a launch plays (the reference game's speed-up of strong launches, often called "balloon"
/// knockback): `1 + (F - 30) / 10` at first for a hitstun ending on frame F (at most 80), easing back to one as the launch goes on.
fn launch_speedup(f: &Fighter, rules: &Ruleset) -> Fx {
    if f.balloon_faf == 0 {
        return Fx::ONE;
    }
    let faf = Fx::from_int(i32::from(f.balloon_faf));
    let first = (Fx::ONE
        + (faf - Fx::from_int(i32::from(rules.balloon_min_faf))) * rules.balloon_per_frame)
        .clamp(Fx::ONE, rules.balloon_max);
    let low = faf * Fx::from_ratio(3, 10);
    if low == faf {
        return first;
    }
    let ease = (low - f.balloon_left) / (low - faf);
    (first * ease).clamp(Fx::ONE, rules.balloon_max)
}

/// The gravity and fall speed of a launched fighter: for the first frames of a launch every fighter falls alike, and a tumbling
/// launch close to straight up falls at its own fixed speed for the whole hitstun.
///
/// A set-knockback launch keeps the plain physics: no shared vertical fall speed (the reference's rule). So does a launch too
/// weak to tumble: the fixed fall belongs to the reference's upward tumble (its "fly top" animation; sim 97).
fn launch_fall(f: &Fighter, p: &FighterParams, rules: &Ruleset) -> (Fx, Fx) {
    let degrees = i32::from(f.launch_angle) * 360 / 4096;
    // (Nor does a launch on the previous reference version's rules, E40: that version had no shared fall speed.)
    if !f.launch_set
        && !f.launch_older
        && f.tumble
        && (i32::from(rules.vertical_launch_from)..=i32::from(rules.vertical_launch_to))
            .contains(&degrees)
    {
        return (p.gravity, rules.vertical_launch_fall);
    }
    if f.state_frame <= u16::from(rules.launch_fall_frames) {
        return (rules.launch_fall_accel, rules.launch_fall);
    }
    (p.gravity, p.max_fall_speed)
}

/// Whether a tech press is fresh enough to tech a surface touched now.
fn tech_ready(f: &Fighter, rules: &Ruleset) -> bool {
    f.tech_timer < rules.tech_window
}

/// A wall or ceiling tech: the launch stops, the fighter clings to the surface for a moment and is intangible. `away` is the
/// direction away from a wall (0 for a ceiling).
fn start_wall_tech(f: &mut Fighter, rules: &Ruleset, away: i8) {
    f.kb_vel = Vec2::ZERO;
    f.vel = Vec2::ZERO;
    f.hitstun = 0;
    f.tumble = false;
    f.invuln = f.invuln.max(rules.wall_tech_invuln);
    f.dodge_dir = Vec2::new(Fx::from_int(i32::from(away)), Fx::ZERO);
    enter(f, S::WallTech);
}

fn wall_tech(f: &mut Fighter, p: &FighterParams, rules: &Ruleset) {
    f.vel = Vec2::ZERO;
    if f.state_frame < u16::from(rules.wall_tech_frames) {
        return;
    }
    let away = f.dodge_dir.x.signum_int();
    enter(f, S::Airborne);
    // Holding jump as it ends kicks off the wall (a wall-tech jump, at the body's wall-jump speeds).
    if away != 0 && f.held(buttons::JUMP) {
        // The press is used: it does not also become a midair jump.
        f.buffer_used |= buttons::JUMP;
        f.vel = Vec2::new(p.wall_jump_vx.mul_int(away), p.wall_jump_vy);
        f.facing = away as i8;
    }
}

/// How close to a wall (world units, from the side of the ECB) a fighter must be to kick off it [E: the reference's wall-contact
/// range is not published].
const WALL_JUMP_REACH: Fx = Fx::from_ratio(1, 8);

/// Kicks off a wall toward `away` (-1 or 1): the body's wall-jump speeds, weaker by `Ruleset::wall_jump_decay` for each wall jump
/// already made since the fighter last stood on the ground or held a ledge.
fn wall_jump(f: &mut Fighter, p: &FighterParams, rules: &Ruleset, away: i8) {
    let mut k = Fx::ONE;
    for _ in 0..f.wall_jumps.min(32) {
        k = k * rules.wall_jump_decay;
    }
    f.wall_jumps = f.wall_jumps.saturating_add(1);
    f.vel = Vec2::new(
        (p.wall_jump_vx * k).mul_int(i32::from(away)),
        p.wall_jump_vy * k,
    );
    f.facing = away;
    f.hop_boost = 0;
    // (A wall jump is not the midair jump: its armor, if any, ends.)
    f.air_jump_frame = 0;
    f.fast_fall = false;
    f.damage_fall = false;
}

/// On the revival platform: held in place and invincible until the player does something new (a stick held from before the
/// knock-out does not count) or the platform runs out; then an ordinary fall with full jumps and the respawn invincibility.
fn revival(f: &mut Fighter, p: &FighterParams, rules: &Ruleset) {
    f.vel = Vec2::ZERO;
    f.invuln = f.invuln.max(rules.respawn_invuln);
    let (now, before) = (f.history[0], f.history[1]);
    let active = now.stick_x.unsigned_abs() >= STICK_DEADZONE.unsigned_abs()
        || now.stick_y.unsigned_abs() >= STICK_DEADZONE.unsigned_abs()
        || now.buttons != 0;
    let fresh = now != before;
    if (active && fresh) || f.state_frame >= u16::from(rules.respawn_platform_frames) {
        f.invuln = rules.respawn_invuln;
        f.air_jumps_left = p.air_jumps;
        f.air_dodge_used = false;
        enter(f, S::Airborne);
    }
}

/// Knocked back by a clank: a short slide, then free.
fn rebound(f: &mut Fighter, p: &FighterParams, stage: &Stage) {
    if f.grounded() {
        f.vel.x = approach(f.vel.x, Fx::ZERO, p.ground_friction);
        if !slide_on_platform(f, p, stage) {
            return;
        }
    } else if air_move(f, p, stage) {
        enter_landing(f, p.landing_lag);
        return;
    }
    if f.state_frame >= u16::from(f.lag) {
        enter(f, if f.grounded() { S::Idle } else { S::Airborne });
    }
}

fn hitstun(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    // A sped-up launch (see `launch_speedup`) plays several whole knockback frames in one frame, all of it (the slowing, gravity and
    // the movement), as the reference does by skipping frames: the fighter follows the same path, only sooner.
    let mut steps = 1;
    if f.balloon_faf > 0 {
        let before = f.balloon_left;
        f.balloon_left -= launch_speedup(f, rules);
        let ceil = |x: Fx| (x + Fx::from_raw(65535)).floor_int().max(0);
        steps = (ceil(before) - ceil(f.balloon_left)).clamp(1, 8);
        // (The count starts at the first actionable frame, one past the last frame of hitstun.)
        f.hitstun = (ceil(f.balloon_left) - 1).clamp(0, 600) as u16;
    } else {
        f.hitstun = f.hitstun.saturating_sub(1);
    }
    // Hitstun cancel: deep into a long hitstun in the air, once the launch has slowed enough, an air dodge (and a little later an
    // aerial) ends it early. What is left of the launch carries on as ordinary momentum.
    if !f.grounded() && !f.launch_pending && f.hitstun > 0 {
        let speed = f.kb_vel.length();
        let dodge = f.state_frame >= u16::from(rules.hitstun_dodge_cancel)
            && speed < rules.hitstun_dodge_cancel_speed
            && !f.air_dodge_used
            && f.take(buttons::SHIELD, p.input_buffer);
        let attack = !dodge
            && f.state_frame >= u16::from(rules.hitstun_attack_cancel)
            && speed < rules.hitstun_attack_cancel_speed
            && f.take(buttons::ATTACK, p.input_buffer);
        if dodge || attack {
            f.vel += f.kb_vel;
            f.kb_vel = Vec2::ZERO;
            f.hitstun = 0;
            f.tumble = false;
            if dodge {
                start_air_dodge(f, p, stage, rules);
            } else {
                start_air_attack(f, p);
            }
            return;
        }
    }
    for _ in 0..steps {
        if !knockback_frame(f, p, stage, rules) {
            return;
        }
    }

    if f.hitstun == 0 {
        if f.grounded() {
            f.tumble = false;
            f.kb_vel = Vec2::ZERO;
            enter(f, S::Idle);
        } else {
            // Leftover launch speed carries on as ordinary air momentum. A tumbling fighter is still tumbling: it can act, but not
            // fast fall until it does.
            let tumbling = f.tumble;
            f.tumble = false;
            f.vel += f.kb_vel;
            f.kb_vel = Vec2::ZERO;
            enter(f, S::Airborne);
            f.damage_fall = tumbling;
        }
    }
}

/// One knockback frame: the launch slows, gravity pulls, the fighter moves, and walls, ceilings and the floor do what they do (a
/// bounce, a tech, a landing). Returns false when the hitstun ended here (a tech or a landing), so no more frames play.
fn knockback_frame(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) -> bool {
    // A position vector's carry (E34) moves at its fixed step, with no decay and no gravity, until its count runs out.
    let carried = f.posvec > 0 && !f.launch_pending;
    if carried {
        f.posvec -= 1;
        f.vel = Vec2::ZERO;
    } else {
        let decay = rules.knockback_decay;
        let speed = f.kb_vel.length();
        f.kb_vel = if speed > decay {
            f.kb_vel * ((speed - decay) / speed)
        } else {
            Vec2::ZERO
        };
    }

    if f.pull {
        pull_frame(f, rules);
    }
    let pulling = f.vortex > 0;
    let vortex = vortex_pull(f, p);
    if f.grounded() && vortex.y > Fx::ZERO {
        // (A vortex drawing a grounded fighter up lifts it off the floor.)
        f.platform = NONE;
    }
    if f.grounded() {
        if pulling {
            f.vel = Vec2::new(f.kb_vel.x + vortex.x, Fx::ZERO);
            if collision::move_x(stage, p, &mut f.pos, f.vel.x) {
                f.kb_vel.x = Fx::ZERO;
            }
            let still_on = collision::platform(stage, f.platform)
                .is_some_and(|plat| f.pos.x >= plat.left && f.pos.x <= plat.right);
            if !still_on {
                f.platform = NONE;
            }
            return true;
        }
        // A grounded hit slides the fighter along the floor, and its traction slows the slide too (the reference: low-traction
        // fighters slide farther when hit on the ground), so a jab's push stays short and its next hit reaches. (A pull is not slowed.)
        if !f.pull && !carried {
            f.kb_vel.x = approach(f.kb_vel.x, Fx::ZERO, p.ground_friction);
        }
        f.vel = Vec2::new(f.kb_vel.x, Fx::ZERO);
        if collision::move_x(stage, p, &mut f.pos, f.vel.x) || (carried && f.posvec == 0) {
            f.kb_vel.x = Fx::ZERO;
        }
        let still_on = collision::platform(stage, f.platform)
            .is_some_and(|plat| f.pos.x >= plat.left && f.pos.x <= plat.right);
        if !still_on {
            f.platform = NONE;
        }
        return true;
    }
    if !pulling && !carried {
        let (gravity, max_fall) = launch_fall(f, p, rules);
        f.vel.y = (f.vel.y - gravity).max(-max_fall);
    }
    let falling = f.vel.y + f.kb_vel.y + vortex.y;
    let heading_x = (f.vel.x + f.kb_vel.x + vortex.x).signum_int();
    let moved = move_by(f, p, stage, f.vel + f.kb_vel + vortex);
    if carried && f.posvec == 0 {
        // (The carry is over: it stops where it got to.)
        f.kb_vel = Vec2::ZERO;
    }
    if moved.wall {
        if tech_ready(f, rules) {
            start_wall_tech(f, rules, (-heading_x) as i8);
            return false;
        }
        // A tumbling fighter bounces off the wall; a lighter launch just stops against it.
        f.kb_vel.x = if f.tumble {
            -(f.kb_vel.x * rules.bounce_keep)
        } else {
            Fx::ZERO
        };
        f.vel.x = Fx::ZERO;
    }
    if moved.ceiling {
        if tech_ready(f, rules) {
            start_wall_tech(f, rules, 0);
            return false;
        }
        f.kb_vel.y = if f.tumble {
            -(f.kb_vel.y * rules.bounce_keep)
        } else {
            Fx::ZERO
        };
        f.vel.y = Fx::ZERO;
    }
    if let Some(platform) = moved.landing {
        // A lighter hit (not a tumble: a jab, an autolink loop dragging the fighter down) lands it on its feet, as in the reference:
        // the hitstun carries on on the ground, with no tech and no knockdown.
        if !f.tumble {
            land(f, p, stage, platform);
            f.kb_vel.y = Fx::ZERO;
            return true;
        }
        let teched = tech_ready(f, rules);
        // Spiked into the floor hard enough, a tumbling fighter bounces back up instead of lying down (unless it techs).
        if !teched && f.tumble && f.kb_vel.y < Fx::ZERO && -falling > rules.ground_bounce_speed {
            if let Some(plat) = stage.platforms.get(platform) {
                f.pos.y = plat.y;
            }
            f.kb_vel.y = -(f.kb_vel.y * rules.bounce_keep);
            f.vel.y = Fx::ZERO;
            return false;
        }
        land(f, p, stage, platform);
        f.kb_vel = Vec2::ZERO;
        f.hitstun = 0;
        f.tumble = false;
        tumble_landing(f, rules);
        return false;
    }
    true
}

/// One frame of a pull (the pull angle): the fighter moves `pull_rate` of the way to `link_point` (just in front of whoever pulled
/// it), at most `pull_max_speed`, with no fall of its own, and stops being pulled once it is there. The hitstun runs as usual.
fn pull_frame(f: &mut Fighter, rules: &Ruleset) {
    let to = f.link_point - f.pos;
    let len = to.length();
    if len <= Fx::from_ratio(1, 10) {
        f.pull = false;
        f.kb_vel = Vec2::ZERO;
        return;
    }
    let mut v = to * rules.pull_rate;
    let speed = v.length();
    if speed > rules.pull_max_speed {
        v = v * (rules.pull_max_speed / speed);
    }
    f.kb_vel = v;
    f.vel = Vec2::ZERO;
}

/// Touching down from a launch, or from the tumble after one: a shield press just before is a tech (in place, or a roll if the stick
/// is flicked sideways); without one it is a missed tech and the fighter lies in a knockdown and picks a get-up.
fn tumble_landing(f: &mut Fighter, rules: &Ruleset) {
    let teched = tech_ready(f, rules);
    let roll_dir = f.flick_x(rules.tech_window);
    f.vel.x = Fx::ZERO;
    if teched && roll_dir != 0 {
        f.dodge_dir = Vec2::new(Fx::from_int(i32::from(roll_dir)), Fx::ZERO);
        enter(f, S::Roll);
    } else if teched {
        f.invuln = f.invuln.max(rules.tech_invuln);
        enter_landing(f, rules.tech_lag);
    } else {
        enter(f, S::Knockdown);
    }
}

/// An autolink launch (angle 367, as the reference): the fighter is sent at the attacker's velocity when it hit plus a fifth of the way
/// from its body (its middle, where it is now, after any survival DI) to where the hitbox was, at most `autolink_max_speed` and
/// falling no faster than `autolink_max_fall`. It is not a tumble, DI has no angle to bend, and the hitstun is the knockback's as usual.
fn autolink(f: &mut Fighter, p: &FighterParams, rules: &Ruleset) {
    let middle = f.pos + Vec2::new(Fx::ZERO, p.ecb_height / Fx::from_int(2));
    let pull = (f.link_point - middle) * Fx::from_ratio(1, 5);
    let mut v = f.link_vel + pull;
    let len = v.length();
    if len > rules.autolink_max_speed && len > Fx::ZERO {
        v = v * (rules.autolink_max_speed / len);
    }
    v.y = v.y.max(-rules.autolink_max_fall);
    f.kb_vel = v;
    f.tumble = false;
    f.link = false;
    f.vel = Vec2::ZERO;
    f.launch_pending = false;
    f.balloon_faf = 0;
    f.balloon_left = Fx::ZERO;
    if f.grounded() {
        if f.kb_vel.y > Fx::ZERO {
            f.platform = NONE;
        } else {
            f.kb_vel.y = Fx::ZERO;
        }
    }
}

/// A vortex launch (angle 366, E28, ssbwiki.com `Autolink_angle`): the fighter is sent the way the attacker was moving when it
/// hit (`link_vel`'s direction) at the knockback's own launch speed, with no cap and no grounded rule, and for the next
/// [`crate::state::VORTEX_FRAMES`] knockback frames it is also drawn to the hitbox's centre (`vortex_pull`). An attacker standing
/// still gives the pull alone. Not a tumble below the tumble threshold; DI does not bend it [E: the reference's DI on 366 is not
/// published].
fn vortex_launch(f: &mut Fighter, rules: &Ruleset) {
    let speed = f.launch_kb * Fx::from_ratio(3, 800);
    let len = f.link_vel.length();
    f.kb_vel = if len > Fx::ZERO {
        f.link_vel * (speed / len)
    } else {
        Vec2::ZERO
    };
    f.tumble = f.launch_kb >= rules.tumble_knockback;
    f.link = false;
    f.vortex = crate::state::VORTEX_FRAMES;
    f.vel = Vec2::ZERO;
    f.launch_pending = false;
    f.balloon_faf = 0;
    f.balloon_left = Fx::ZERO;
}

/// A position-vector launch (E34, the reference's position vectors, ssbwiki.com `Autolink_angle`): the fighter's feet are carried
/// straight to `link_point` (set by the hit: a point relative to the attacker) at the speed that covers the distance in `posvec`
/// frames, with no gravity and no knockback decay on the way (`knockback_frame`); when the count runs out the carry stops [E: the
/// reference's "the launch speed drops off quickly" after the frames is taken as stopping] and the rest of the hitstun is an ordinary
/// fall. Not a tumble; DI does not bend it.
fn posvec_launch(f: &mut Fighter) {
    let n = Fx::from_int(i32::from(f.posvec.max(1)));
    let to = f.link_point - f.pos;
    f.kb_vel = Vec2::new(to.x / n, to.y / n);
    f.tumble = false;
    f.link = false;
    f.vel = Vec2::ZERO;
    f.launch_pending = false;
    f.balloon_faf = 0;
    f.balloon_left = Fx::ZERO;
    if f.grounded() {
        if f.kb_vel.y > Fx::ZERO {
            f.platform = NONE;
        } else {
            f.kb_vel.y = Fx::ZERO;
        }
    }
}

/// One frame of a vortex pull: the share (`1 / frames left`) of the way from the fighter's middle to the hitbox centre it covers
/// this frame, on top of its launch, so it is there when the count runs out (with no launch of its own). Its own fall is held
/// while it is pulled.
fn vortex_pull(f: &mut Fighter, p: &FighterParams) -> Vec2 {
    if f.vortex == 0 {
        return Vec2::ZERO;
    }
    let middle = f.pos + Vec2::new(Fx::ZERO, p.ecb_height / Fx::from_int(2));
    let (to, n) = (f.link_point - middle, Fx::from_int(i32::from(f.vortex)));
    let step = Vec2::new(to.x / n, to.y / n);
    f.vortex -= 1;
    f.vel = Vec2::ZERO;
    step
}

/// Landing during an aerial on its move frame `frame`: early or late in the move it autocancels (only the normal landing lag, `plain`);
/// otherwise the move's landing lag, or its landing move (a landing hit: its own short move, whose hitboxes come out on the landing
/// frame).
fn land_out_of_move(f: &mut Fighter, mv: &Move, frame: u16, plain: u8) {
    let clean = frame < u16::from(mv.autocancel_before) || frame >= u16::from(mv.autocancel_after);
    match mv.landing_move {
        Some(next) if !clean => begin_attack(f, MoveId::from_index(next)),
        _ => enter_landing(f, if clean { plain } else { mv.landing_lag }),
    }
}

/// Turns a pending hit into motion at the end of hitlag. Directional influence: the part of the held stick
/// that is perpendicular to the launch direction bends the angle, by up to `di_degrees`. Launch speed influence: holding the
/// stick up or down makes a tumbling launch a little faster or slower (`lsi_up`, `lsi_down`), unless it is close to vertical.
fn apply_launch(f: &mut Fighter, p: &FighterParams, rules: &Ruleset) {
    let angle = Angle::from_raw(f.launch_angle);
    let (ux, uy) = (trig::cos(angle), trig::sin(angle));
    let input = f.history[0];
    let cross = (ux * input.stick_y_fx() - uy * input.stick_x_fx()).clamp(-Fx::ONE, Fx::ONE);
    let bend =
        (cross.mul_int(i32::from(rules.di_degrees)) * Fx::from_ratio(4096, 360)).raw() / 65536;
    let bent = Angle::from_raw((i32::from(f.launch_angle) + bend).rem_euclid(4096) as u16);

    if f.link && f.link_kind == crate::state::LINK_VORTEX {
        vortex_launch(f, rules);
        return;
    }
    if f.link && f.link_kind == crate::state::LINK_POSVEC {
        posvec_launch(f);
        return;
    }
    if f.link {
        autolink(f, p, rules);
        return;
    }
    if f.pull {
        // A pull has no launch of its own: the fighter is drawn in frame by frame (`pull_frame`) while its hitstun lasts.
        f.kb_vel = Vec2::ZERO;
        f.vel = Vec2::ZERO;
        f.tumble = false;
        f.launch_pending = false;
        f.balloon_faf = 0;
        f.balloon_left = Fx::ZERO;
        return;
    }
    // 0.03 reference units per knockback unit, 8 reference units per world unit.
    let mut speed = f.launch_kb * Fx::from_ratio(3, 800);
    f.tumble = f.launch_kb >= rules.tumble_knockback;
    if f.tumble {
        // The angle after DI, in degrees (0 to 359), against the vertical bands where LSI does nothing.
        let degrees = i32::from(bent.raw()) * 360 / 4096;
        let margin = i32::from(rules.lsi_vertical);
        let vertical = (degrees - 90).abs() <= margin || (degrees - 270).abs() <= margin;
        let y = input.stick_y_fx().clamp(-Fx::ONE, Fx::ONE);
        if !vertical && y > Fx::ZERO {
            speed = speed * (Fx::ONE + (rules.lsi_up - Fx::ONE) * y);
        } else if !vertical && y < Fx::ZERO {
            speed = speed * (Fx::ONE + (Fx::ONE - rules.lsi_down) * y);
        }
    }
    f.kb_vel = Vec2::new(trig::cos(bent) * speed, trig::sin(bent) * speed);
    // The gravity boost of a tumbling launch (not one in the vertical band, which falls at its own fixed speed).
    let degrees = i32::from(f.launch_angle) * 360 / 4096;
    let vertical = (i32::from(rules.vertical_launch_from)..=i32::from(rules.vertical_launch_to))
        .contains(&degrees);
    // (The previous reference version's rules, E40, boost every tumbling launch: it had no vertical band.)
    if f.tumble && (!vertical || f.launch_older) {
        f.kb_vel.y += rules.gravity_boost * (p.gravity - rules.gravity_boost_from);
    }
    f.vel = Vec2::ZERO;
    f.launch_pending = false;
    // A tumbling launch is sped up at first (see `launch_speedup`), unless it is from set knockback (the reference: those keep the
    // plain launch physics).
    let faf = f.hitstun + 1;
    if f.tumble
        && !f.launch_set
        && !f.launch_older
        && faf > u16::from(rules.balloon_min_faf)
        && rules.balloon_max > Fx::ONE
    {
        f.balloon_faf = faf.min(u16::from(rules.balloon_max_faf)) as u8;
        f.balloon_left = Fx::from_int(i32::from(faf));
    } else {
        f.balloon_faf = 0;
        f.balloon_left = Fx::ZERO;
    }
    if f.grounded() {
        if f.kb_vel.y > Fx::ZERO {
            f.platform = NONE;
        } else {
            f.kb_vel.y = Fx::ZERO;
        }
    }
}

/// Automatic SDI: as an electric hit's hitlag ends, the victim drifts toward the held stick (twice), unless the launch is close to
/// straight up.
fn asdi(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset) {
    let times = f.asdi;
    f.asdi = 0;
    if times == 0 {
        return;
    }
    let degrees = i32::from(f.launch_angle) * 360 / 4096;
    if (i32::from(rules.vertical_launch_from)..i32::from(rules.vertical_launch_to))
        .contains(&degrees)
    {
        return;
    }
    let input = f.history[0];
    let d = rules.asdi_distance.mul_int(i32::from(times));
    let dx = if input.stick_x.unsigned_abs() >= STICK_DEADZONE.unsigned_abs() {
        input.stick_x.signum()
    } else {
        0
    };
    let dy = if input.stick_y.unsigned_abs() >= STICK_DEADZONE.unsigned_abs() {
        input.stick_y.signum()
    } else {
        0
    };
    if dx != 0 {
        collision::move_x(stage, p, &mut f.pos, d.mul_int(i32::from(dx)));
    }
    if dy > 0 && !f.grounded() {
        collision::move_up(stage, p, &mut f.pos, d);
    }
}

/// Survival DI: a stick flick during hitlag nudges the fighter, at most once every `sdi_interval` frames. On a shield the nudge
/// is sideways only and two thirds as far.
fn sdi(f: &mut Fighter, p: &FighterParams, stage: &Stage, rules: &Ruleset, shield: bool) {
    if f.sdi_wait > 0 {
        return;
    }
    let (dx, dy) = (f.flick_x(1), f.flick_y(1));
    if dx == 0 && dy == 0 {
        return;
    }
    f.sdi_wait = rules.sdi_interval;
    let mut d = if shield {
        rules.sdi_distance * Fx::from_ratio(2, 3)
    } else {
        rules.sdi_distance
    };
    // The hit's own SDI multiplier (a multi-hit's loop is easier to escape, a hit meant to connect harder).
    if !shield {
        d = d * Fx::from_ratio(i32::from(f.sdi_mult), 100);
    }
    // Every `sdi_combo_hits` hits of one combo, survival DI goes further (so long multi-hits can be escaped).
    if let Some(steps) = f.hits_taken.checked_div(rules.sdi_combo_hits) {
        for _ in 0..steps.min(8) {
            d = d * rules.sdi_combo_mult;
        }
    }
    if dx != 0 {
        collision::move_x(stage, p, &mut f.pos, d.mul_int(i32::from(dx)));
    }
    if shield {
        return;
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
