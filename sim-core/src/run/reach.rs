//! How far a fighter can get across a gap or up a climb (R2e): a double jump's reach from the fighter's numbers, and its recovery's
//! reach **measured** by playing the fighter's own up special in the sim.
//!
//! A room's skilled route is proven passable for a pair of jumps (`sim_content::check_run`): the double-jump routes for
//! `RunRules::skill_step` and `skill_gap`, the recovery routes (`Room::recovery`) for `recovery_step` and `recovery_gap`. When a run
//! starts, the fighter's two reaches are worked out here once and kept in the run ([`super::Run::jump_reach`],
//! [`super::Run::recovery_reach`]); a room whose skilled route asks more than the fighter can do is laid without that route
//! ([`super::RoomSlot::skill_off`]): never required, simply not offered.

use crate::content::{Content, FighterParams, Platform, Stage};
use crate::fixed::Fx;
use crate::input::{buttons, Input};
use crate::state::{FighterState as S, GameState, MatchRules};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

/// How high and how far a fighter's jumps reach: a full hop and one midair jump (its height), and the distance at full air speed
/// over the time those jumps keep it at or above the height it left from. Plain kinematics on the fighter's numbers (frames of 1/60 s).
/// (R2a's passability numbers are checked against it, `sim_content::run`.)
pub fn jump_reach(p: &FighterParams) -> (Fx, Fx) {
    let rise = |v0: Fx, g: Fx| -> (Fx, i32) {
        // (Height and frames up to the top, the speed losing `g` a frame.)
        let (mut v, mut h, mut t) = (v0, Fx::ZERO, 0);
        while v > Fx::ZERO && t < 600 {
            h += v;
            v -= g;
            t += 1;
        }
        (h, t)
    };
    let (h1, t1) = rise(p.full_hop_velocity, p.gravity);
    let burst = p.hop_burst_velocity.mul_int(i32::from(p.hop_burst_frames));
    let (h2, t2) = if p.air_jumps > 0 {
        rise(p.air_jump_velocity, p.gravity)
    } else {
        (Fx::ZERO, 0)
    };
    let height = h1 + burst + h2;
    // (Up with the hop, up again with the air jump, then down that far: roughly twice the rising frames of each.)
    let frames = 2 * (t1 + t2);
    (height, p.air_speed.mul_int(frames))
}

/// The most frames a measuring trial lasts.
const TRIAL_FRAMES: u32 = 420;

/// How high and how far fighter `char_id` of `world` gets with a full hop, one midair jump at its top, and its **up special** at the
/// top of that (its recovery), drifting forward all the while: the height above where it jumped from, and how far forward it is when
/// it comes back down to that height. Measured by playing it in the sim on a little ledge in empty space (two trials, the up special
/// aimed straight up and aimed up and forward; the best of each), so every recovery counts as it really plays: its burst, its
/// angle, its script, its helpless fall. `world`'s stage is borrowed for the trials and put back.
pub fn recovery_reach(world: &mut Content, char_id: u8) -> (Fx, Fx) {
    let saved = core::mem::replace(&mut world.stage, trial_stage());
    let mut best = (Fx::ZERO, Fx::ZERO);
    for aim in [(0i8, 127i8), (90, 90)] {
        let (h, d) = trial(world, char_id, aim);
        best = (best.0.max(h), best.1.max(d));
    }
    world.stage = saved;
    best
}

/// A ledge two units wide at height 0 in empty space, the blast zones far away.
fn trial_stage() -> Stage {
    let far = Fx::from_int(400);
    Stage {
        platforms: vec![Platform {
            left: Fx::from_int(-2),
            right: Fx::from_int(1),
            y: Fx::ZERO,
            bottom: Fx::from_int(-4),
            pass_through: false,
            rise: Fx::ZERO,
        }],
        ledges: Vec::new(),
        spawns: [Vec2::ZERO; MAX_FIGHTERS],
        blast_left: -far,
        blast_right: far,
        blast_bottom: -far,
        blast_top: far,
    }
}

/// One trial: hop, midair jump at the top, up special (stick `aim`) at the top of that, forward drift; returns (peak height, distance
/// forward on coming back down to the ledge's height).
fn trial(world: &Content, char_id: u8, aim: (i8, i8)) -> (Fx, Fx) {
    let rules = MatchRules {
        stocks: 1,
        time_limit: 0,
    };
    let mut s = GameState::new_with_rules(world, 0, [char_id, 0, 0, 0], 0b0001, rules);
    let start = s.fighters[0].pos;
    // Stages: 0 hopping (jump held until airborne), 1 rising from the hop, 2 rising from the midair jump, 3 recovering, 4 falling.
    let mut stage = 0u8;
    let mut wait = 0u32;
    let mut peak = start.y;
    let mut left_ground = false;
    for _ in 0..TRIAL_FRAMES {
        let f = s.fighters[0];
        let mut input = Input {
            stick_x: 127,
            stick_y: 0,
            buttons: 0,
        };
        let rising = f.vel.y + f.kb_vel.y > Fx::ZERO;
        match stage {
            0 => {
                input.buttons = buttons::JUMP;
                if !f.grounded() && f.state != S::JumpSquat {
                    left_ground = true;
                    stage = 1;
                    input.buttons = 0;
                }
            }
            1 => {
                if !rising && f.air_jumps_left > 0 && f.history[0].buttons & buttons::JUMP == 0 {
                    input.buttons = buttons::JUMP;
                    stage = 2;
                    wait = 3;
                } else if !rising && f.air_jumps_left == 0 {
                    stage = 2;
                }
            }
            2 => {
                wait = wait.saturating_sub(1);
                if wait == 0 && !rising && f.history[0].buttons & buttons::SPECIAL == 0 {
                    (input.stick_x, input.stick_y) = aim;
                    input.buttons = buttons::SPECIAL;
                    stage = 3;
                    wait = 12;
                }
            }
            3 => {
                // (The stick held where it aimed for the move's first frames, then forward.)
                if wait > 0 {
                    wait -= 1;
                    (input.stick_x, input.stick_y) = aim;
                    input.buttons = buttons::SPECIAL;
                }
                if wait == 0 {
                    stage = 4;
                }
            }
            _ => {}
        }
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = input;
        crate::step(&mut s, world, &inputs);
        let g = &s.fighters[0];
        peak = peak.max(g.pos.y);
        if !g.active {
            break;
        }
        // Back down to the ledge's height after the recovery: how far forward.
        if left_ground && stage >= 3 && g.pos.y <= start.y && g.vel.y + g.kb_vel.y <= Fx::ZERO {
            return (
                (peak - start.y).max(Fx::ZERO),
                (g.pos.x - start.x).max(Fx::ZERO),
            );
        }
    }
    let g = &s.fighters[0];
    (
        (peak - start.y).max(Fx::ZERO),
        (g.pos.x - start.x).max(Fx::ZERO),
    )
}
