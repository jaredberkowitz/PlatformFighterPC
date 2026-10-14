//! The single-player endless run (R1, sim 120; `docs/modes/RUNNER_PLAN.md`).
//!
//! A run is an ordinary match with one fighter, played by the versus `step` with every versus rule, on a stage that streams. The run's
//! own state, [`Run`], lives in [`GameState::run`] and is hashed with the rest; in versus it is inactive and nothing here runs.
//!
//! * **The camera is in the sim.** Its left edge pushes the fighter and can crush it against a wall (a lost life), and the blast zones
//!   and mob spawning follow it, so it must be deterministic and hashed. The game only draws it (and follows height).
//! * **The stage streams.** The chunks in play ([`ChunkSlot`], a small ring) lay their platforms and ledges into fixed pools
//!   ([`Run::platforms`], [`Run::ledges`]). A pool entry keeps its index for as long as it lives, so a fighter's `platform` and `ledge`
//!   (indices) stay right as chunks come and go; ground that continues across a chunk seam at one height is one entry, extended, so
//!   walking, dashing, rolling and wavedashing over a seam are what they are on any floor. Each frame [`build_stage`] writes the pools
//!   into the run's working copy of the content's [`Stage`] (free entries as padding that nothing touches), with the blast zones and the
//!   respawn point around the camera, and the versus code collides with that stage as with any other.
//! * **Mobs** ([`Mob`], a fixed pool of [`MAX_MOBS`]) take the versus knockback, hitstun and hitlag formulas from `combat`, and hit the
//!   fighter through `combat::apply_hit` (shields, DI, crouch cancelling, armor, counters: everything a fighter's hit gets).

pub mod data;
mod mobs;

pub use data::{Chunk, MobKind, MobSpawn, RunContent, RunRules, MOB_FLYER, MOB_WALKER};

use crate::collision;
use crate::combat::{self, params_of};
use crate::content::{Content, Ledge, Platform, Stage, MAX_LEDGES};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::input::Input;
use crate::rng::Rng;
use crate::state::{Fighter, FighterState as S, GameState, MatchRules, DRAW, NONE};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;
use data::{CHUNK_LEDGES, CHUNK_PLATFORMS, CHUNK_SPAWNS};

/// Mobs alive at once.
pub const MAX_MOBS: usize = 16;
/// Chunks in play at once. With chunks at least `data::MIN_CHUNK_WIDTH` wide and the standard view this is never short (a test
/// sweeps long runs to check).
pub const RING: usize = 4;
/// The stage's platform pool: every platform of every chunk in play.
pub const POOL_PLATFORMS: usize = RING * CHUNK_PLATFORMS;
/// The ledge pool is the versus ledge table (`GameState::ledge_owner`).
pub const POOL_LEDGES: usize = MAX_LEDGES;
const _: () = assert!(RING * CHUNK_LEDGES <= POOL_LEDGES);
const _: () = assert!(CHUNK_SPAWNS <= 8);

/// Where a free pool entry is put in the stage: far away, a platform no point is ever on and a ledge no fighter can reach.
const FAR: i32 = -30000;

/// One entry of the stage's platform pool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolPlatform {
    pub live: bool,
    pub left: Fx,
    pub right: Fx,
    pub y: Fx,
    pub bottom: Fx,
    pub pass_through: bool,
}

/// One entry of the stage's ledge pool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolLedge {
    pub live: bool,
    pub x: Fx,
    pub y: Fx,
    pub side: i8,
}

/// A chunk in play: which one, where its left edge is, and which of its spawn markers have made their mob.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChunkSlot {
    pub live: bool,
    pub chunk: u8,
    /// Its left and right edges (world x).
    pub x: Fx,
    pub end: Fx,
    pub spawned: u8,
}

/// `Mob::mode` values.
pub const MODE_IDLE: u8 = 0;
pub const MODE_MOVE: u8 = 1;
pub const MODE_WINDUP: u8 = 2;
pub const MODE_ATTACK: u8 = 3;
pub const MODE_COOLDOWN: u8 = 4;
/// Beaten: it flies off with its last launch for `RunRules::burst_frames`, then is gone. It cannot hit or be hit.
pub const MODE_BURST: u8 = 5;

/// An enemy (see `run::mobs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mob {
    pub active: bool,
    /// Index into `RunContent::mobs`.
    pub kind: u8,
    /// `MODE_*`, and the frames left in it (the windup, attack, cooldown or burst).
    pub mode: u8,
    pub timer: u16,
    /// Its feet.
    pub pos: Vec2,
    /// Its own movement, and its launch (which slows as a fighter's does).
    pub vel: Vec2,
    pub kb_vel: Vec2,
    pub facing: i8,
    /// The pool platform it stands on, or [`NONE`].
    pub platform: i8,
    pub percent: Fx,
    pub stamina: Fx,
    pub hitlag: u8,
    pub hitstun: u16,
    /// The swing that last hit it (`1 << 16 | attacker << 8 | move`, 0 for none), which of its hit groups have, and the swing's move
    /// frame when last seen: a lower one (or another move, or no attack) is a new swing.
    pub hit_stamp: u32,
    pub hit_groups: u8,
    pub hit_age: u16,
    /// Projectile slots whose shot has hit it (cleared as each slot empties).
    pub shot_mask: u16,
    /// Its current attack has connected (one hit per attack).
    pub struck: bool,
    /// Frames left in which leaving the screen counts as the player's kill.
    pub credit: u16,
    /// A flyer's dive target.
    pub target: Vec2,
}

/// The run's state (inactive in versus).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub active: bool,
    /// The last life is gone; the run stands still.
    pub over: bool,
    /// The run's own generator: chunk order and anything else the run draws, apart from the match's.
    pub rng: Rng,
    /// The view's left edge (world x). It only ever moves right.
    pub cam_x: Fx,
    /// Where the fighter started, and the furthest right it has been: the distance travelled.
    pub start_x: Fx,
    pub best_x: Fx,
    /// Where the next chunk's left edge goes.
    pub next_x: Fx,
    /// Chunks placed so far, the last one, and the chunks already used in this lap (bit per chunk).
    pub placed: u32,
    pub last_chunk: u8,
    pub lap: u32,
    pub slots: [ChunkSlot; RING],
    pub platforms: [PoolPlatform; POOL_PLATFORMS],
    pub ledges: [PoolLedge; POOL_LEDGES],
    /// Counts every change to the pools (the game redraws the stage when it changes).
    pub layout: u32,
    pub mobs: [Mob; MAX_MOBS],
    pub coins: u32,
    pub kills: u16,
    /// Hits and kills in a row without being hit, and the longest such run.
    pub combo: u16,
    pub best_combo: u16,
    /// The fighter's percent and lives at the end of the last frame (a rise, or a lost life, breaks the combo).
    pub last_percent: Fx,
    pub last_stocks: u8,
    /// Lives lost to the left edge.
    pub crushes: u8,
}

impl Run {
    /// Not a run (versus).
    pub const INACTIVE: Run = Run {
        active: false,
        over: false,
        rng: Rng::new(0),
        cam_x: Fx::ZERO,
        start_x: Fx::ZERO,
        best_x: Fx::ZERO,
        next_x: Fx::ZERO,
        placed: 0,
        last_chunk: 0,
        lap: 0,
        slots: [ChunkSlot {
            live: false,
            chunk: 0,
            x: Fx::ZERO,
            end: Fx::ZERO,
            spawned: 0,
        }; RING],
        platforms: [PoolPlatform {
            live: false,
            left: Fx::ZERO,
            right: Fx::ZERO,
            y: Fx::ZERO,
            bottom: Fx::ZERO,
            pass_through: false,
        }; POOL_PLATFORMS],
        ledges: [PoolLedge {
            live: false,
            x: Fx::ZERO,
            y: Fx::ZERO,
            side: 0,
        }; POOL_LEDGES],
        layout: 0,
        mobs: [Mob {
            active: false,
            kind: 0,
            mode: 0,
            timer: 0,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            kb_vel: Vec2::ZERO,
            facing: 0,
            platform: NONE,
            percent: Fx::ZERO,
            stamina: Fx::ZERO,
            hitlag: 0,
            hitstun: 0,
            hit_stamp: 0,
            hit_groups: 0,
            hit_age: 0,
            shot_mask: 0,
            struck: false,
            credit: 0,
            target: Vec2::ZERO,
        }; MAX_MOBS],
        coins: 0,
        kills: 0,
        combo: 0,
        best_combo: 0,
        last_percent: Fx::ZERO,
        last_stocks: 0,
        crushes: 0,
    };

    /// Whole world units travelled (the HUD's metres).
    pub fn distance(&self) -> i32 {
        (self.best_x - self.start_x).max(Fx::ZERO).floor_int()
    }

    /// The coin multiplier the combo gives now.
    pub fn multiplier(&self, rules: &RunRules) -> u32 {
        let step = rules.combo_step.max(1);
        (1 + u32::from(self.combo / step)).min(u32::from(rules.combo_max.max(1)))
    }

    /// The camera's creep this frame: faster the further the run has gone, up to its cap.
    pub fn creep(&self, rules: &RunRules) -> Fx {
        let hundreds = (self.best_x - self.start_x).max(Fx::ZERO) / Fx::from_int(100);
        (rules.creep + rules.creep_growth * hundreds).min(rules.creep_max.max(rules.creep))
    }
}

/// The fighter the run is about (the one player).
pub const PLAYER: usize = 0;

/// Starts a run: `world` is the run's working copy of the content (the roster with the fighter; its stage is rewritten from the run
/// every frame), `char_id` the fighter in it. The first chunk is laid with its ground under the view's left edge and the fighter
/// stands on it.
pub fn new_run(world: &mut Content, rc: &RunContent, seed: u64, char_id: u8) -> GameState {
    let rules = MatchRules {
        stocks: rc.rules.lives,
        time_limit: 0,
    };
    let mut state = GameState::new_with_rules(world, seed, [char_id, 0, 0, 0], 0b0001, rules);
    let mut run = Run::INACTIVE;
    run.active = true;
    // (Its own stream, so the chunk order depends on the seed alone.)
    run.rng = Rng::new(seed ^ 0x5255_4e5f_5345_4544);
    run.cam_x = Fx::ZERO;
    run.next_x = Fx::from_int(-4);
    place_chunks(&mut run, rc);
    let start = Vec2::new(Fx::from_int(8), Fx::ZERO);
    run.start_x = start.x;
    run.best_x = start.x;
    state.run = run;
    build_stage(&state.run, &rc.rules, &mut world.stage);
    let f = &state.fighters[PLAYER];
    let params = params_of(world, f);
    let mut spawned = Fighter::spawn(
        start,
        char_id,
        1,
        collision::standing_on(&world.stage, start),
        params.air_jumps,
        world.rules.shield_max,
    );
    spawned.stocks = f.stocks;
    spawned.active = true;
    state.fighters[PLAYER] = spawned;
    state.run.last_stocks = spawned.stocks;
    spawn_mobs(&mut state.run, rc, &world.stage);
    state
}

/// Advances a run one frame: the versus step on the streamed stage, then the mobs, the camera, the push, the coins and combo, and
/// the stage streaming on. `world` is the run's working content (see [`new_run`]); only its stage is written.
pub fn step_run(
    state: &mut GameState,
    world: &mut Content,
    rc: &RunContent,
    inputs: &[Input; MAX_FIGHTERS],
) {
    if !state.run.active {
        crate::step(state, world, inputs);
        return;
    }
    build_stage(&state.run, &rc.rules, &mut world.stage);
    if state.run.over {
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    crate::step(state, world, inputs);
    mobs::player_hits(state, world, rc);
    mobs::update(state, world, rc);
    mobs::attacks(state, world, rc);
    follow_camera(state, &rc.rules);
    push_from_edge(state, world, &rc.rules);
    mobs::leave_screen(state, rc);
    settle_combo(state);
    free_behind(state, &rc.rules);
    place_chunks(&mut state.run, rc);
    build_stage(&state.run, &rc.rules, &mut world.stage);
    spawn_mobs(&mut state.run, rc, &world.stage);
    if !state.fighters[PLAYER].active {
        state.run.over = true;
        state.winner = DRAW;
    }
}

/// Writes the run's pools into `out` (free entries as far-away padding), with the blast zone around the view and the respawn point.
pub fn build_stage(run: &Run, rules: &RunRules, out: &mut Stage) {
    let far = Fx::from_int(FAR);
    out.platforms.clear();
    for p in &run.platforms {
        out.platforms.push(if p.live {
            Platform {
                left: p.left,
                right: p.right,
                y: p.y,
                bottom: p.bottom,
                pass_through: p.pass_through,
            }
        } else {
            // (A pass-through platform whose right is left of its left: nothing ever spans it.)
            Platform {
                left: Fx::ONE,
                right: Fx::ZERO,
                y: far,
                bottom: far,
                pass_through: true,
            }
        });
    }
    out.ledges.clear();
    for l in &run.ledges {
        out.ledges.push(if l.live {
            Ledge {
                x: l.x,
                y: l.y,
                side: l.side,
            }
        } else {
            Ledge {
                x: far,
                y: far,
                side: -1,
            }
        });
    }
    out.blast_left = run.cam_x - rules.blast_margin;
    out.blast_right = run.cam_x + rules.view_width + rules.blast_margin;
    out.blast_bottom = rules.blast_bottom;
    out.blast_top = rules.blast_top;
    let back = respawn_point(run, rules);
    out.spawns = [
        back,
        Vec2::new(far, far),
        Vec2::new(far, far),
        Vec2::new(far, far),
    ];
}

/// Where a lost life comes back: `respawn_at` across the view, on the highest ground there (or the first ground to its right).
fn respawn_point(run: &Run, rules: &RunRules) -> Vec2 {
    let x0 = run.cam_x + rules.view_width * rules.respawn_at;
    let steps = (rules.view_width / Fx::from_int(2)).floor_int().max(1);
    for k in 0..=steps {
        let x = x0 + Fx::from_int(k);
        let top = run
            .platforms
            .iter()
            .filter(|p| p.live && x >= p.left && x <= p.right && p.y < rules.blast_top)
            .map(|p| p.y)
            .max();
        if let Some(y) = top {
            return Vec2::new(x, y);
        }
    }
    Vec2::new(x0, Fx::ZERO)
}

/// The next chunk: the first one to begin with, then every chunk once a lap in a seeded order (never the same twice running).
fn next_chunk(run: &mut Run, count: usize) -> u8 {
    if run.placed == 0 || count <= 1 {
        return 0;
    }
    let all = if count >= 32 {
        u32::MAX
    } else {
        (1u32 << count) - 1
    };
    if run.lap & all == all {
        run.lap = 0;
    }
    let free = |run: &Run, c: usize| run.lap & (1 << c) == 0 && c as u8 != run.last_chunk;
    let mut choices = (0..count.min(32)).filter(|&c| free(run, c)).count() as u32;
    if choices == 0 {
        // (Only the last chunk is left in this lap: start the next lap without it.)
        run.lap = 0;
        choices = (0..count.min(32)).filter(|&c| free(run, c)).count() as u32;
    }
    let mut pick = run.rng.range(choices.max(1));
    for c in 0..count.min(32) {
        if free(run, c) {
            if pick == 0 {
                return c as u8;
            }
            pick -= 1;
        }
    }
    0
}

/// Lays chunks ahead until the view and the spawn distance beyond it are covered (or the ring is full).
fn place_chunks(run: &mut Run, rc: &RunContent) {
    let rules = &rc.rules;
    let reach = run.cam_x + rules.view_width + rules.spawn_ahead + rules.blast_margin;
    while run.next_x < reach && !rc.chunks.is_empty() {
        let Some(slot) = run.slots.iter().position(|s| !s.live) else {
            return;
        };
        let id = next_chunk(run, rc.chunks.len());
        let chunk = &rc.chunks[usize::from(id)];
        let x0 = run.next_x;
        run.slots[slot] = ChunkSlot {
            live: true,
            chunk: id,
            x: x0,
            end: x0 + chunk.width,
            spawned: 0,
        };
        for p in &chunk.platforms {
            let w = PoolPlatform {
                live: true,
                left: x0 + p.left,
                right: x0 + p.right,
                y: p.y,
                bottom: p.bottom,
                pass_through: p.pass_through,
            };
            // Ground carrying on from the last chunk at the same height is the same block, longer.
            let joined = run.platforms.iter_mut().find(|e| {
                e.live && !w.pass_through && !e.pass_through && e.right == w.left && e.y == w.y
            });
            if let Some(e) = joined {
                e.right = w.right;
                e.bottom = e.bottom.min(w.bottom);
                continue;
            }
            if let Some(e) = run.platforms.iter_mut().find(|e| !e.live) {
                *e = w;
            }
        }
        for l in &chunk.ledges {
            if let Some(e) = run.ledges.iter_mut().find(|e| !e.live) {
                *e = PoolLedge {
                    live: true,
                    x: x0 + l.x,
                    y: l.y,
                    side: l.side,
                };
            }
        }
        run.lap |= 1 << u32::from(id).min(31);
        run.last_chunk = id;
        run.placed = run.placed.wrapping_add(1);
        run.next_x = x0 + chunk.width;
        run.layout = run.layout.wrapping_add(1);
    }
}

/// Frees the chunks, platforms and ledges wholly behind the blast zone's left edge (unless a fighter is still on one).
fn free_behind(state: &mut GameState, rules: &RunRules) {
    let edge = state.run.cam_x - rules.blast_margin;
    let mut changed = false;
    let run = &mut state.run;
    for s in run.slots.iter_mut() {
        if s.live && s.end < edge {
            s.live = false;
        }
    }
    for (i, p) in run.platforms.iter_mut().enumerate() {
        if p.live && p.right < edge {
            let stood_on = state
                .fighters
                .iter()
                .any(|f| f.active && f.platform == i as i8);
            if !stood_on {
                *p = PoolPlatform::default();
                changed = true;
            }
        }
    }
    for (i, l) in run.ledges.iter_mut().enumerate() {
        if l.live && l.x < edge && state.ledge_owner[i] == NONE {
            *l = PoolLedge::default();
            changed = true;
        }
    }
    for m in run.mobs.iter_mut().filter(|m| m.active) {
        if usize::try_from(m.platform).is_ok_and(|i| !run.platforms[i].live) {
            m.platform = NONE;
        }
    }
    if changed {
        run.layout = run.layout.wrapping_add(1);
    }
}

/// Mobs from the spawn markers that have come within `spawn_ahead` of the view's right edge.
fn spawn_mobs(run: &mut Run, rc: &RunContent, stage: &Stage) {
    let reach = run.cam_x + rc.rules.view_width + rc.rules.spawn_ahead;
    for s in 0..RING {
        let slot = run.slots[s];
        if !slot.live {
            continue;
        }
        let chunk = &rc.chunks[usize::from(slot.chunk).min(rc.chunks.len() - 1)];
        for (j, marker) in chunk.spawns.iter().enumerate().take(CHUNK_SPAWNS) {
            let bit = 1u8 << j;
            if slot.spawned & bit != 0 || slot.x + marker.x >= reach {
                continue;
            }
            run.slots[s].spawned |= bit;
            let Some(kind) = rc.mobs.get(usize::from(marker.kind)) else {
                continue;
            };
            let Some(m) = run.mobs.iter_mut().find(|m| !m.active) else {
                continue;
            };
            let pos = Vec2::new(slot.x + marker.x, marker.y);
            *m = Mob {
                active: true,
                kind: marker.kind,
                mode: MODE_IDLE,
                pos,
                facing: -1,
                platform: if kind.behavior == MOB_WALKER {
                    collision::standing_on(stage, pos)
                } else {
                    NONE
                },
                stamina: kind.stamina,
                ..Mob::default()
            };
        }
    }
}

/// The camera creeps on by itself and follows the fighter right (at up to `cam_max_speed`), never back. The creep waits while the
/// fighter is back on its revival platform.
fn follow_camera(state: &mut GameState, rules: &RunRules) {
    let f = state.fighters[PLAYER];
    let run = &mut state.run;
    let creep = if f.state == S::Respawn {
        Fx::ZERO
    } else {
        run.creep(rules)
    };
    let from = run.cam_x;
    let mut to = from + creep;
    if f.active {
        let wanted = f.pos.x - rules.view_width * rules.follow;
        if wanted > to {
            to = wanted.min(from + rules.cam_max_speed).max(to);
        }
        run.best_x = run.best_x.max(f.pos.x);
    }
    run.cam_x = to;
}

/// The screen's left edge pushes a fighter behind it back in (not one flying from a hit, which may leave the screen and be knocked
/// out). A fighter it cannot move, pinned against a wall, is crushed once the edge is `crush_depth` past it: it loses a life.
fn push_from_edge(state: &mut GameState, world: &Content, rules: &RunRules) {
    let limit = state.run.cam_x + rules.push_margin;
    for i in 0..MAX_FIGHTERS {
        let f = state.fighters[i];
        if !f.active || f.pos.x >= limit {
            continue;
        }
        let pinned_states = matches!(
            f.state,
            S::LedgeHang | S::Grabbed | S::Grabbing | S::Carrying | S::Buried | S::Respawn
        );
        // (Hanging from a ledge with the edge coming is being pinned against its wall.)
        let mut blocked = f.state == S::LedgeHang;
        if f.state != S::Hitstun && !pinned_states {
            let want = (limit - f.pos.x).min(rules.push_speed);
            let params = *params_of(world, &f);
            let g = &mut state.fighters[i];
            let from = g.pos.x;
            collision::move_x(&world.stage, &params, &mut g.pos, want);
            blocked = g.pos.x - from < want;
            if let Some(p) = collision::platform(&world.stage, g.platform) {
                if g.pos.x > p.right {
                    // Pushed off the end of its floor: it falls.
                    g.platform = NONE;
                    g.vel.y = Fx::ZERO;
                    crate::fighter::stand_or_fall(g);
                }
            }
        }
        if blocked && limit - state.fighters[i].pos.x >= rules.crush_depth {
            state.run.crushes = state.run.crushes.saturating_add(1);
            lose_life(state, world, i);
        }
    }
}

/// A life lost anywhere but the blast zone (crushed): as a knock-out, back on the revival platform.
fn lose_life(state: &mut GameState, world: &Content, i: usize) {
    crate::grab::drop_grab(state, world, i);
    crate::items::let_go(state, world, i);
    crate::items::lose_pocket(state, i);
    let counted = state.rules.stocks > 0;
    let f = &mut state.fighters[i];
    combat::respawn(f, world, i, counted);
    if counted && f.stocks == 0 {
        f.active = false;
    }
}

/// A hit taken (the percent rose) or a life lost breaks the combo.
fn settle_combo(state: &mut GameState) {
    let f = state.fighters[PLAYER];
    let run = &mut state.run;
    if f.percent > run.last_percent || f.stocks < run.last_stocks || !f.active {
        run.combo = 0;
    }
    run.last_percent = f.percent;
    run.last_stocks = f.stocks;
}

/// One more hit or kill in the combo.
fn count_combo(run: &mut Run) {
    run.combo = run.combo.saturating_add(1);
    run.best_combo = run.best_combo.max(run.combo);
}

/// A kill: the coins (times the combo multiplier, counted with this kill) and the tally.
fn pay_kill(run: &mut Run, rc: &RunContent, kind: u8) {
    count_combo(run);
    run.kills = run.kills.saturating_add(1);
    let coins = rc.mobs.get(usize::from(kind)).map_or(0, |k| k.coins);
    run.coins = run
        .coins
        .saturating_add(u32::from(coins) * run.multiplier(&rc.rules));
}

impl StateHash for Mob {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        h.write_u8(self.kind);
        h.write_u8(self.mode);
        h.write_u16(self.timer);
        self.pos.hash_into(h);
        self.vel.hash_into(h);
        self.kb_vel.hash_into(h);
        h.write_i8(self.facing);
        h.write_i8(self.platform);
        self.percent.hash_into(h);
        self.stamina.hash_into(h);
        h.write_u8(self.hitlag);
        h.write_u16(self.hitstun);
        h.write_u32(self.hit_stamp);
        h.write_u8(self.hit_groups);
        h.write_u16(self.hit_age);
        h.write_u16(self.shot_mask);
        h.write_bool(self.struck);
        h.write_u16(self.credit);
        self.target.hash_into(h);
    }
}

impl StateHash for Run {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        h.write_bool(self.over);
        self.rng.hash_into(h);
        self.cam_x.hash_into(h);
        self.start_x.hash_into(h);
        self.best_x.hash_into(h);
        self.next_x.hash_into(h);
        h.write_u32(self.placed);
        h.write_u8(self.last_chunk);
        h.write_u32(self.lap);
        for s in &self.slots {
            h.write_bool(s.live);
            h.write_u8(s.chunk);
            s.x.hash_into(h);
            s.end.hash_into(h);
            h.write_u8(s.spawned);
        }
        for p in &self.platforms {
            h.write_bool(p.live);
            p.left.hash_into(h);
            p.right.hash_into(h);
            p.y.hash_into(h);
            p.bottom.hash_into(h);
            h.write_bool(p.pass_through);
        }
        for l in &self.ledges {
            h.write_bool(l.live);
            l.x.hash_into(h);
            l.y.hash_into(h);
            h.write_i8(l.side);
        }
        h.write_u32(self.layout);
        for m in &self.mobs {
            m.hash_into(h);
        }
        h.write_u32(self.coins);
        h.write_u16(self.kills);
        h.write_u16(self.combo);
        h.write_u16(self.best_combo);
        self.last_percent.hash_into(h);
        h.write_u8(self.last_stocks);
        h.write_u8(self.crushes);
    }
}
