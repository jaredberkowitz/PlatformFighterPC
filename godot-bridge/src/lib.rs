//! Thin Godot wrapper around `sim-core`.
//!
//! GDScript gathers input, calls `tick()` once per 60 Hz physics frame, then reads state back to
//! draw it. Nothing here decides gameplay, and animation never drives the sim. Floats appear only
//! in the read-back getters, for display.

use godot::classes::{INode, Node};
use godot::prelude::*;
use sim_core::input::buttons;
use sim_core::{step, Content, Fx, GameState, Input, MAX_FIGHTERS, SIM_VERSION};
use std::collections::VecDeque;

/// Frames of history kept for stepping backwards in training mode (20 seconds).
const MAX_HISTORY: usize = 1200;

struct PfExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PfExtension {}

fn f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}

fn clamp_i8(v: i32) -> i8 {
    v.clamp(-127, 127) as i8
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct SimRunner {
    base: Base<Node>,
    content: Content,
    state: GameState,
    inputs: [Input; MAX_FIGHTERS],
    history: VecDeque<GameState>,
}

#[godot_api]
impl INode for SimRunner {
    fn init(base: Base<Node>) -> Self {
        let content = Content::placeholder();
        let state = GameState::new(&content, 1, [0, 1, 0, 1]);
        SimRunner {
            base,
            content,
            state,
            inputs: [Input::default(); MAX_FIGHTERS],
            history: VecDeque::new(),
        }
    }
}

#[godot_api]
impl SimRunner {
    // ---- Control ----

    /// Starts a fresh match. `chars` picks each player's physics profile (0 balanced, 1 floaty).
    #[func]
    fn start(&mut self, seed: i64, chars: PackedInt32Array) {
        let mut ids = [0u8; MAX_FIGHTERS];
        for (slot, c) in ids.iter_mut().zip(chars.as_slice()) {
            *slot = (*c).clamp(0, self.content.fighters.len() as i32 - 1) as u8;
        }
        self.state = GameState::new(&self.content, seed as u64, ids);
        self.inputs = [Input::default(); MAX_FIGHTERS];
        self.history.clear();
    }

    #[func]
    fn set_input(&mut self, player: i32, stick_x: i32, stick_y: i32, button_mask: i32) {
        if let Some(slot) = usize::try_from(player)
            .ok()
            .and_then(|p| self.inputs.get_mut(p))
        {
            *slot = Input {
                stick_x: clamp_i8(stick_x),
                stick_y: clamp_i8(stick_y),
                buttons: button_mask as u16,
            };
        }
    }

    /// Advances the simulation exactly one frame with the inputs last set.
    #[func]
    fn tick(&mut self) {
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(self.state);
        step(&mut self.state, &self.content, &self.inputs);
    }

    /// Steps one frame backwards (training mode). Returns false if there is no history left.
    #[func]
    fn step_back(&mut self) -> bool {
        match self.history.pop_back() {
            Some(s) => {
                self.state = s;
                true
            }
            None => false,
        }
    }

    /// Debug: teleport a fighter into the air at (x, y) with zero velocity.
    #[func]
    fn debug_place_airborne(&mut self, player: i32, x: f32, y: f32) {
        if let Some(fighter) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fighter.pos.x = Fx::from_raw((x * 65536.0) as i32);
            fighter.pos.y = Fx::from_raw((y * 65536.0) as i32);
            fighter.vel = sim_core::Vec2::ZERO;
            fighter.platform = sim_core::state::NONE;
            fighter.ledge = sim_core::state::NONE;
            fighter.state = sim_core::state::FighterState::Airborne;
            fighter.state_frame = 0;
        }
    }

    /// Debug: put a fighter into the special fall.
    #[func]
    fn debug_helpless(&mut self, player: i32) {
        if let Some(fighter) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            sim_core::fighter::enter_helpless(fighter);
        }
    }

    // ---- Match info ----

    #[func]
    fn frame(&self) -> i64 {
        i64::from(self.state.frame)
    }

    #[func]
    fn checksum(&self) -> GString {
        GString::from(format!("{:016x}", self.state.checksum()).as_str())
    }

    #[func]
    fn content_hash(&self) -> GString {
        GString::from(format!("{:016x}", self.content.hash()).as_str())
    }

    #[func]
    fn sim_version(&self) -> i32 {
        i32::from(SIM_VERSION)
    }

    #[func]
    fn history_len(&self) -> i32 {
        self.history.len() as i32
    }

    #[func]
    fn fighter_count(&self) -> i32 {
        MAX_FIGHTERS as i32
    }

    #[func]
    fn button_mask(&self, name: GString) -> i32 {
        let mask = match name.to_string().as_str() {
            "jump" => buttons::JUMP,
            "attack" => buttons::ATTACK,
            "special" => buttons::SPECIAL,
            "shield" => buttons::SHIELD,
            "grab" => buttons::GRAB,
            _ => 0,
        };
        i32::from(mask)
    }

    // ---- Fighters (read-only, for drawing) ----

    #[func]
    fn fighter_pos(&self, i: i32) -> Vector2 {
        self.fighter(i)
            .map_or(Vector2::ZERO, |fi| Vector2::new(f(fi.pos.x), f(fi.pos.y)))
    }

    #[func]
    fn fighter_vel(&self, i: i32) -> Vector2 {
        self.fighter(i)
            .map_or(Vector2::ZERO, |fi| Vector2::new(f(fi.vel.x), f(fi.vel.y)))
    }

    #[func]
    fn fighter_dodge_dir(&self, i: i32) -> Vector2 {
        self.fighter(i).map_or(Vector2::ZERO, |fi| {
            Vector2::new(f(fi.dodge_dir.x), f(fi.dodge_dir.y))
        })
    }

    #[func]
    fn fighter_state(&self, i: i32) -> GString {
        let name = self
            .fighter(i)
            .map_or(String::new(), |fi| format!("{:?}", fi.state));
        GString::from(name.as_str())
    }

    #[func]
    fn fighter_state_frame(&self, i: i32) -> i32 {
        self.fighter(i).map_or(0, |fi| i32::from(fi.state_frame))
    }

    #[func]
    fn fighter_facing(&self, i: i32) -> i32 {
        self.fighter(i).map_or(1, |fi| i32::from(fi.facing))
    }

    /// [char_id, platform, air_jumps_left, air_dodge_used, fast_fall, ledge, ledge_invuln,
    ///  ledge_grab_count, lag, ledge_cooldown, platform_ignore]
    #[func]
    fn fighter_info(&self, i: i32) -> PackedInt32Array {
        let v: Vec<i32> = self.fighter(i).map_or(vec![0; 11], |fi| {
            vec![
                i32::from(fi.char_id),
                i32::from(fi.platform),
                i32::from(fi.air_jumps_left),
                i32::from(fi.air_dodge_used),
                i32::from(fi.fast_fall),
                i32::from(fi.ledge),
                i32::from(fi.ledge_invuln),
                i32::from(fi.ledge_grab_count),
                i32::from(fi.lag),
                i32::from(fi.ledge_cooldown),
                i32::from(fi.platform_ignore),
            ]
        });
        PackedInt32Array::from(v.as_slice())
    }

    /// [ecb_half_width, ecb_height, ecb_side_height, wavedash_min_down, ground_assist_dist]
    #[func]
    fn fighter_body(&self, i: i32) -> PackedFloat32Array {
        let v: Vec<f32> = self.fighter(i).map_or(vec![0.0; 5], |fi| {
            let p = &self.content.fighters[usize::from(fi.char_id)];
            vec![
                f(p.ecb_half_width),
                f(p.ecb_height),
                f(p.ecb_side_height),
                f(p.wavedash_min_down),
                f(p.ground_assist_dist),
            ]
        });
        PackedFloat32Array::from(v.as_slice())
    }

    // ---- Combat (read-only, for drawing) ----

    #[func]
    fn fighter_percent(&self, i: i32) -> f32 {
        self.fighter(i).map_or(0.0, |fi| f(fi.percent))
    }

    /// [stocks, hitlag, hitstun, move_id, tumble, invuln, launch_pending, state_frame]
    #[func]
    fn fighter_combat(&self, i: i32) -> PackedInt32Array {
        let v: Vec<i32> = self.fighter(i).map_or(vec![0; 8], |fi| {
            vec![
                i32::from(fi.stocks),
                i32::from(fi.hitlag),
                i32::from(fi.hitstun),
                i32::from(fi.move_id),
                i32::from(fi.tumble),
                i32::from(fi.invuln),
                i32::from(fi.launch_pending),
                i32::from(fi.state_frame),
            ]
        });
        PackedInt32Array::from(v.as_slice())
    }

    #[func]
    fn fighter_kb_vel(&self, i: i32) -> Vector2 {
        self.fighter(i).map_or(Vector2::ZERO, |fi| {
            Vector2::new(f(fi.kb_vel.x), f(fi.kb_vel.y))
        })
    }

    /// Name of the move the fighter is performing, or an empty string.
    #[func]
    fn fighter_move_name(&self, i: i32) -> GString {
        let name = self.fighter(i).map_or("", |fi| {
            if fi.state == sim_core::state::FighterState::Attack {
                sim_core::moves::MoveId::from_index(fi.move_id).name()
            } else {
                ""
            }
        });
        GString::from(name)
    }

    /// [total_frames, active_start, active_end] of the current move, or zeros.
    #[func]
    fn fighter_move_timing(&self, i: i32) -> PackedInt32Array {
        let v: Vec<i32> = self.fighter(i).map_or(vec![0; 3], |fi| {
            if fi.state != sim_core::state::FighterState::Attack {
                return vec![0; 3];
            }
            let params = sim_core::combat::params_of(&self.content, fi);
            let mv = sim_core::combat::weapon_of(&self.content, params).get(fi.move_id);
            let start = mv.hitboxes.iter().map(|h| h.start).min().unwrap_or(0);
            let end = mv.hitboxes.iter().map(|h| h.end).max().unwrap_or(0);
            vec![i32::from(mv.total_frames), i32::from(start), i32::from(end)]
        });
        PackedInt32Array::from(v.as_slice())
    }

    /// Active hitboxes this frame as flat [x, y, radius, priority] groups.
    #[func]
    fn fighter_hitboxes(&self, i: i32) -> PackedFloat32Array {
        let mut v: Vec<f32> = Vec::new();
        if let Some(fi) = self.fighter(i) {
            if fi.state == sim_core::state::FighterState::Attack {
                let params = sim_core::combat::params_of(&self.content, fi);
                let mv = sim_core::combat::weapon_of(&self.content, params).get(fi.move_id);
                for (_, hb, center) in sim_core::combat::active_hitboxes(fi, mv) {
                    v.extend([
                        f(center.x),
                        f(center.y),
                        f(hb.radius),
                        f32::from(hb.priority),
                    ]);
                }
            }
        }
        PackedFloat32Array::from(v.as_slice())
    }

    /// Hurtbox circles as flat [x, y, radius] groups.
    #[func]
    fn fighter_hurtboxes(&self, i: i32) -> PackedFloat32Array {
        let mut v: Vec<f32> = Vec::new();
        if let Some(fi) = self.fighter(i) {
            let params = sim_core::combat::params_of(&self.content, fi);
            for (c, r) in sim_core::combat::hurtboxes(fi, params) {
                v.extend([f(c.x), f(c.y), f(r)]);
            }
        }
        PackedFloat32Array::from(v.as_slice())
    }

    /// Debug: set a fighter's damage percent.
    #[func]
    fn debug_set_percent(&mut self, player: i32, percent: f32) {
        if let Some(fi) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fi.percent = Fx::from_raw((percent.clamp(0.0, 999.0) * 65536.0) as i32);
        }
    }

    /// Debug: stand a fighter on the main stage at x, facing left (-1) or right (1), idle.
    #[func]
    fn debug_stand(&mut self, player: i32, x: f32, facing: i32) {
        if let Some(fi) = usize::try_from(player)
            .ok()
            .and_then(|p| self.state.fighters.get_mut(p))
        {
            fi.pos = sim_core::Vec2::new(Fx::from_raw((x * 65536.0) as i32), Fx::ZERO);
            fi.vel = sim_core::Vec2::ZERO;
            fi.kb_vel = sim_core::Vec2::ZERO;
            fi.platform = 0;
            fi.state = sim_core::state::FighterState::Idle;
            fi.state_frame = 0;
            fi.hitlag = 0;
            fi.hitstun = 0;
            fi.launch_pending = false;
            fi.facing = if facing < 0 { -1 } else { 1 };
        }
    }

    // ---- Stage (read-only) ----

    #[func]
    fn platform_count(&self) -> i32 {
        self.content.stage.platforms.len() as i32
    }

    /// [left, right, top_y, bottom_y, pass_through (1 or 0)]
    #[func]
    fn platform_rect(&self, i: i32) -> PackedFloat32Array {
        let v: Vec<f32> = usize::try_from(i)
            .ok()
            .and_then(|i| self.content.stage.platforms.get(i))
            .map_or(vec![0.0; 5], |p| {
                vec![
                    f(p.left),
                    f(p.right),
                    f(p.y),
                    f(p.bottom),
                    if p.pass_through { 1.0 } else { 0.0 },
                ]
            });
        PackedFloat32Array::from(v.as_slice())
    }

    #[func]
    fn ledge_count(&self) -> i32 {
        self.content.stage.ledges.len() as i32
    }

    /// (x, y, side)
    #[func]
    fn ledge_pos(&self, i: i32) -> Vector3 {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.content.stage.ledges.get(i))
            .map_or(Vector3::ZERO, |l| {
                Vector3::new(f(l.x), f(l.y), f32::from(l.side))
            })
    }

    /// Who holds ledge `i` (player index), or -1.
    #[func]
    fn ledge_owner(&self, i: i32) -> i32 {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.state.ledge_owner.get(i))
            .map_or(-1, |o| i32::from(*o))
    }

    /// [left, right, bottom, top]
    #[func]
    fn blast_zone(&self) -> PackedFloat32Array {
        let s = &self.content.stage;
        let v = [
            f(s.blast_left),
            f(s.blast_right),
            f(s.blast_bottom),
            f(s.blast_top),
        ];
        PackedFloat32Array::from(v.as_slice())
    }

    #[func]
    fn spawn_pos(&self, i: i32) -> Vector2 {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.content.stage.spawns.get(i))
            .map_or(Vector2::ZERO, |p| Vector2::new(f(p.x), f(p.y)))
    }
}

impl SimRunner {
    fn fighter(&self, i: i32) -> Option<&sim_core::Fighter> {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.state.fighters.get(i))
    }
}
