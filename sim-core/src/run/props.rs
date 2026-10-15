//! Props (R2b, `RUNNER_WORLD.md` 4, Punch Quest's torches): hazards and chests standing in the rooms. The player's attack or shot sets
//! one off: a **gas canister** burns for a moment, then bursts; a **loose sign** falls, its hit live until it lands and breaks; a
//! **chest** opens and pays. A hazard's hit lands on the grunts (the player's kills, through the versus knockback formulas) and on the
//! player too (through the versus hit), once each.

use super::data::{PropKind, RunContent, PROP_CHEST, PROP_GAS, PROP_SIGN};
use super::mobs;
use super::{
    Prop, MAX_MOBS, MAX_PROPS, MODE_EMERGE, PLAYER, PROP_BROKEN, PROP_BURST, PROP_FALLING,
    PROP_LIT, PROP_OPEN, PROP_READY,
};
use crate::collision;
use crate::content::Content;
use crate::fixed::Fx;
use crate::moves::Hitbox;
use crate::state::GameState;
use crate::vec2::Vec2;

/// Frames a fallen sign lies broken before it is gone.
const BROKEN_FRAMES: u16 = 12;

fn kind_of<'a>(rc: &'a RunContent, p: &Prop) -> Option<&'a PropKind> {
    rc.props.get(usize::from(p.kind))
}

/// A prop's box: its middle and its half sizes (a sign hangs down from its spot; the others stand on theirs).
pub fn shape(k: &PropKind, p: &Prop) -> (Vec2, Fx, Fx) {
    let half_h = k.height * Fx::HALF;
    let centre = if k.role == PROP_SIGN {
        p.pos - Vec2::new(Fx::ZERO, half_h)
    } else {
        p.pos + Vec2::new(Fx::ZERO, half_h)
    };
    (centre, k.half_width, half_h)
}

/// Every prop's frame: set off by the player, burning, bursting, falling, broken or open.
pub(super) fn update(state: &mut GameState, content: &Content, rc: &RunContent) {
    for n in 0..MAX_PROPS {
        let mut p = state.run.props[n];
        if !p.active {
            continue;
        }
        let Some(k) = kind_of(rc, &p).copied() else {
            state.run.props[n].active = false;
            continue;
        };
        match p.mode {
            PROP_READY => {
                let (centre, hw, hh) = shape(&k, &p);
                if mobs::player_touches(state, content, centre, (hw, hh, Fx::ZERO)) {
                    match k.role {
                        PROP_GAS => {
                            p.mode = PROP_LIT;
                            p.timer = u16::from(k.fuse.max(1));
                        }
                        PROP_SIGN => {
                            p.mode = PROP_FALLING;
                            p.vel = Vec2::ZERO;
                        }
                        PROP_CHEST => {
                            p.mode = PROP_OPEN;
                            let run = &mut state.run;
                            run.coins = run.coins.saturating_add(u32::from(k.coins));
                            run.chests = run.chests.saturating_add(1);
                        }
                        _ => {}
                    }
                }
            }
            PROP_LIT => {
                p.timer = p.timer.saturating_sub(1);
                if p.timer == 0 {
                    p.mode = PROP_BURST;
                    p.timer = u16::from(k.frames.max(1));
                    p.hit_mobs = 0;
                    p.hit_player = false;
                }
            }
            PROP_BURST => {
                let at = p.pos + Vec2::new(Fx::ZERO, k.hit.y);
                strike(state, content, rc, &mut p, &k.hit, at);
                p.timer = p.timer.saturating_sub(1);
                if p.timer == 0 {
                    p.active = false;
                }
            }
            PROP_FALLING => {
                p.vel.y = (p.vel.y - k.gravity).max(-Fx::ONE);
                let bottom = p.pos - Vec2::new(Fx::ZERO, k.height);
                let next = bottom + p.vel;
                if let Some(i) = collision::find_landing(&content.stage, bottom, next, false) {
                    p.pos.y = content.stage.platforms[i].top_at(p.pos.x) + k.height;
                    p.mode = PROP_BROKEN;
                    p.timer = BROKEN_FRAMES;
                } else {
                    p.pos += p.vel;
                    let at = p.pos + Vec2::new(Fx::ZERO, k.hit.y);
                    strike(state, content, rc, &mut p, &k.hit, at);
                }
                // (Fallen out of the world.)
                if p.pos.y < state.run.blast_y(&rc.rules).0 {
                    p.active = false;
                }
            }
            PROP_BROKEN => {
                p.timer = p.timer.saturating_sub(1);
                if p.timer == 0 {
                    p.active = false;
                }
            }
            _ => {}
        }
        state.run.props[n] = p;
    }
}

/// A hazard's hit at `at`: every grunt it touches and the player, once each, through the versus hit (R2c: grunts are fighters; a
/// grunt's hit is the player's, for the combo and the kill).
fn strike(
    state: &mut GameState,
    content: &Content,
    _rc: &RunContent,
    p: &mut Prop,
    hb: &Hitbox,
    at: Vec2,
) {
    for n in 0..MAX_MOBS {
        let bit = 1u16 << n;
        let m = state.run.mobs[n];
        if !m.active || m.mode == MODE_EMERGE || p.hit_mobs & bit != 0 {
            continue;
        }
        let g = state.fighters[mobs::slot(n)];
        let side = if g.pos.x >= at.x { 1 } else { -1 };
        if mobs::hit_body(
            state,
            content,
            mobs::slot(n),
            (at, side, Vec2::ZERO),
            hb,
            at,
        ) {
            p.hit_mobs |= bit;
        }
    }
    if !p.hit_player {
        let f = state.fighters[PLAYER];
        let side = if f.pos.x >= at.x { 1 } else { -1 };
        if mobs::hit_body(state, content, PLAYER, (at, side, Vec2::ZERO), hb, at) {
            p.hit_player = true;
        }
    }
}
