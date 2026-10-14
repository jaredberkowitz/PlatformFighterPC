//! The single-player endless run (R1, sim 120; rooms R1b, sim 122; `docs/modes/RUNNER_PLAN.md`).
//!
//! A run is an ordinary match with one fighter, played by the versus `step` with every versus rule, on a stage that streams. The run's
//! own state, [`Run`], lives in [`GameState::run`] and is hashed with the rest; in versus it is inactive and nothing here runs.
//!
//! * **The world is a chain of rooms**, left to right ([`data::Room`]: streets, arenas, shops, rest stops). The rooms in play are a
//!   small window ([`RoomSlot`], a ring): the current room (the furthest the player has entered), up to `RunRules::rooms_behind`
//!   behind it and `rooms_ahead` ahead. Entering a new room lays the next one ahead and drops the oldest behind; the left edge of the
//!   oldest room kept is a **wall** (a solid block in the stage, so fighters and mobs collide with it and can wall-jump off it). An
//!   arena closes a second wall, its **gate**, on its right edge until every mob of the room is beaten.
//! * **The camera is in the sim**: the blast zone and mob spawning follow it, so it must replay exactly. It follows the player both
//!   ways, smoothly, clamped to the window's walls. The game only draws it (and follows height).
//! * **The stage streams.** The rooms in play lay their platforms and ledges into fixed pools ([`Run::platforms`], [`Run::ledges`]).
//!   A pool entry keeps its index for as long as it lives, so a fighter's `platform` and `ledge` (indices) stay right as rooms come
//!   and go; ground that continues across a seam at one height is one entry, extended (and trimmed when the room behind is dropped),
//!   so walking, dashing, rolling and wavedashing over a seam are what they are on any floor. Each frame [`build_stage`] writes the
//!   pools and the two walls into the run's working copy of the content's [`Stage`] (free entries as padding that nothing touches),
//!   with the blast zones and the respawn point, and the versus code collides with that stage as with any other.
//! * **Mobs** ([`Mob`], a fixed pool of [`MAX_MOBS`]) take the versus knockback, hitstun and hitlag formulas from `combat`, and hit the
//!   fighter through `combat::apply_hit` (shields, DI, crouch cancelling, armor, counters: everything a fighter's hit gets).
//! * **The shop** is a state of the run: standing at a shop's door and pressing up opens it, and while it is open the world stands
//!   still and the player's input picks and buys wares ([`data::ShopItem`]), all in the sim, so a run with shopping replays exactly.

pub mod data;
mod mobs;

pub use data::{
    MobKind, MobSpawn, Room, RunContent, RunRules, ShopItem, ITEM_HEAL, ITEM_LIFE, MOB_FLYER,
    MOB_WALKER, ROOM_ARENA, ROOM_REST, ROOM_SHOP, ROOM_STREET,
};

use crate::collision;
use crate::combat::params_of;
use crate::content::{Content, Ledge, Platform, Stage, MAX_LEDGES};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use crate::input::{Input, STICK_THRESHOLD};
use crate::rng::Rng;
use crate::state::{Fighter, FighterState as S, GameState, MatchRules, DRAW, NONE};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;
use data::{MAX_SHOP_ITEMS, ROOM_LEDGES, ROOM_PLATFORMS, ROOM_SPAWNS};

/// Mobs alive at once.
pub const MAX_MOBS: usize = 16;
/// Rooms in play at once: `rooms_behind + 1 + rooms_ahead` may be at most this (`sim_content::check_run`).
pub const RING: usize = 4;
/// The stage's platform pool: every platform of every room in play.
pub const POOL_PLATFORMS: usize = RING * ROOM_PLATFORMS;
/// The stage's platforms: the pool, then the left wall ([`WALL_INDEX`]) and an arena's gate ([`GATE_INDEX`]).
pub const WALL_INDEX: usize = POOL_PLATFORMS;
pub const GATE_INDEX: usize = POOL_PLATFORMS + 1;
/// The ledge pool is the versus ledge table (`GameState::ledge_owner`).
pub const POOL_LEDGES: usize = MAX_LEDGES;
const _: () = assert!(RING * ROOM_LEDGES <= POOL_LEDGES);
const _: () = assert!(ROOM_SPAWNS <= 8);
const _: () = assert!(POOL_PLATFORMS + 2 <= i8::MAX as usize);

/// Where a free pool entry is put in the stage: far away, a platform no point is ever on and a ledge no fighter can reach.
const FAR: i32 = -30000;
/// How far above the blast zone's top the walls reach (nothing gets over them or stands on them).
const WALL_ABOVE: i32 = 8;

/// `Run::shop_note` values: what the last press in the shop did.
pub const NOTE_NONE: u8 = 0;
pub const NOTE_BOUGHT: u8 = 1;
pub const NOTE_TOO_POOR: u8 = 2;
pub const NOTE_SOLD_OUT: u8 = 3;
pub const NOTE_NOT_NEEDED: u8 = 4;

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

/// A room in play: which one, its number in the run, where it is, and which of its spawn markers have made their mob.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoomSlot {
    pub live: bool,
    /// Index into `RunContent::rooms`.
    pub room: u8,
    /// Its number in the run (0 is the first room).
    pub seq: u32,
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
    /// The room it came from (`RoomSlot::seq`): an arena opens once all of its own mobs are beaten.
    pub room: u32,
}

/// The run's state (inactive in versus).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub active: bool,
    /// The last life is gone; the run stands still.
    pub over: bool,
    /// The run's own generator: the rooms and anything else the run draws, apart from the match's.
    pub rng: Rng,
    /// The view's left edge (world x).
    pub cam_x: Fx,
    /// With a creep on (`RunRules::creep`), the mark the view's left edge never goes back past.
    pub floor_x: Fx,
    /// Where the fighter started, and the furthest right it has been: the distance travelled.
    pub start_x: Fx,
    pub best_x: Fx,
    /// Where the next room's left edge goes.
    pub next_x: Fx,
    /// Rooms laid so far (the next one's number), the last one laid (index into the content) and its type.
    pub placed: u32,
    pub last_room: u8,
    pub last_kind: u8,
    /// The type of each of the last 16 rooms laid, 4 bits each, newest in the low bits (for the map).
    pub kinds_log: u64,
    /// Rooms laid since the last shop, and how many more make the next one.
    pub since_shop: u8,
    pub shop_due: u8,
    /// The current room: the furthest the player has entered (its number).
    pub room: u32,
    /// The left wall: the left edge of the oldest room kept.
    pub wall_x: Fx,
    /// An arena's gate: closed on its right edge (`gate_x`) from entering it until it is cleared.
    pub gate_closed: bool,
    pub gate_x: Fx,
    /// Arenas cleared.
    pub cleared: u16,
    pub slots: [RoomSlot; RING],
    pub platforms: [PoolPlatform; POOL_PLATFORMS],
    pub ledges: [PoolLedge; POOL_LEDGES],
    /// Counts every change to the stage, the walls included (the game redraws it when it changes).
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
    /// The shop: open (the world stands still), the ware picked (one past the last is "leave"), the room whose shop it is, how many
    /// of each ware that shop has sold, what the last press did (`NOTE_*`) and how many presses it has had (for the game's cues).
    pub shop_open: bool,
    pub shop_cursor: u8,
    pub shop_room: u32,
    pub shop_bought: [u8; MAX_SHOP_ITEMS],
    pub shop_note: u8,
    pub shop_acts: u16,
    /// The player's stick height and buttons last frame (the shop reads presses, not holds).
    pub prev_stick_y: i8,
    pub prev_buttons: u16,
}

impl Run {
    /// Not a run (versus).
    pub const INACTIVE: Run = Run {
        active: false,
        over: false,
        rng: Rng::new(0),
        cam_x: Fx::ZERO,
        floor_x: Fx::ZERO,
        start_x: Fx::ZERO,
        best_x: Fx::ZERO,
        next_x: Fx::ZERO,
        placed: 0,
        last_room: 0,
        last_kind: 0,
        kinds_log: 0,
        since_shop: 0,
        shop_due: 0,
        room: 0,
        wall_x: Fx::ZERO,
        gate_closed: false,
        gate_x: Fx::ZERO,
        cleared: 0,
        slots: [RoomSlot {
            live: false,
            room: 0,
            seq: 0,
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
            room: 0,
        }; MAX_MOBS],
        coins: 0,
        kills: 0,
        combo: 0,
        best_combo: 0,
        last_percent: Fx::ZERO,
        last_stocks: 0,
        shop_open: false,
        shop_cursor: 0,
        shop_room: 0,
        shop_bought: [0; MAX_SHOP_ITEMS],
        shop_note: NOTE_NONE,
        shop_acts: 0,
        prev_stick_y: 0,
        prev_buttons: 0,
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

    /// The optional creep this frame: faster the further the run has gone, up to its cap (0 with the standard rules).
    pub fn creep(&self, rules: &RunRules) -> Fx {
        let hundreds = (self.best_x - self.start_x).max(Fx::ZERO) / Fx::from_int(100);
        (rules.creep + rules.creep_growth * hundreds).min(rules.creep_max.max(rules.creep))
    }

    /// The room in play with this number.
    pub fn slot_of(&self, seq: u32) -> Option<&RoomSlot> {
        self.slots.iter().find(|s| s.live && s.seq == seq)
    }

    /// The right end of the rooms in play.
    pub fn window_end(&self) -> Fx {
        self.slots
            .iter()
            .filter(|s| s.live)
            .map(|s| s.end)
            .max()
            .unwrap_or(self.wall_x)
    }

    /// How far right anything may go now: the arena's gate while it is closed, else the end of the rooms in play.
    pub fn right_limit(&self) -> Fx {
        if self.gate_closed {
            self.gate_x
        } else {
            self.window_end()
        }
    }

    /// The type of room `seq` (`ROOM_*`), if it is among the last 16 laid.
    pub fn kind_of(&self, seq: u32) -> Option<u8> {
        let back = self.placed.checked_sub(seq + 1)?;
        (back < 16).then(|| ((self.kinds_log >> (back * 4)) & 0xf) as u8)
    }
}

/// The fighter the run is about (the one player).
pub const PLAYER: usize = 0;

/// Starts a run: `world` is the run's working copy of the content (the roster with the fighter; its stage is rewritten from the run
/// every frame), `char_id` the fighter in it. The first room's left edge is the left wall and the fighter stands in it.
pub fn new_run(world: &mut Content, rc: &RunContent, seed: u64, char_id: u8) -> GameState {
    let rules = MatchRules {
        stocks: rc.rules.lives,
        time_limit: 0,
    };
    let mut state = GameState::new_with_rules(world, seed, [char_id, 0, 0, 0], 0b0001, rules);
    let mut run = Run::INACTIVE;
    run.active = true;
    // (Its own stream, so the rooms depend on the seed alone.)
    run.rng = Rng::new(seed ^ 0x5255_4e5f_5345_4544);
    run.shop_due = draw_shop_due(&mut run, &rc.rules);
    run.next_x = Fx::ZERO;
    run.wall_x = Fx::ZERO;
    run.cam_x = Fx::ZERO;
    run.floor_x = Fx::ZERO;
    run.room = 0;
    while run.placed <= u32::from(rc.rules.rooms_ahead.max(1)) {
        if !place_room(&mut run, rc) {
            break;
        }
    }
    let start = Vec2::new(Fx::from_int(6), Fx::ZERO);
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
    spawn_mobs(&mut state.run, rc);
    state
}

/// Advances a run one frame: the versus step on the streamed stage, then the mobs, the rooms (entering, laying, dropping, an arena's
/// gate), the camera, the coins and combo, and the shop's door. While the shop is open only the shop moves. `world` is the run's
/// working content (see [`new_run`]); only its stage is written.
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
    let input = inputs[PLAYER];
    if state.run.over {
        remember_input(&mut state.run, input);
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    if state.run.shop_open {
        shop_step(state, rc, input);
        remember_input(&mut state.run, input);
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    crate::step(state, world, inputs);
    mobs::player_hits(state, world, rc);
    mobs::update(state, world, rc);
    mobs::attacks(state, world, rc);
    mobs::keep_in_window(state, rc);
    enter_rooms(state, rc);
    follow_camera(state, &rc.rules);
    if creeping(&rc.rules) {
        push_from_edge(state, world, &rc.rules);
    }
    mobs::leave_screen(state, rc);
    settle_combo(state);
    check_arena(state, rc);
    if let Some(seq) = door_here(state, rc).filter(|_| pressed_up(&state.run, input)) {
        open_shop(&mut state.run, seq);
    }
    remember_input(&mut state.run, input);
    build_stage(&state.run, &rc.rules, &mut world.stage);
    spawn_mobs(&mut state.run, rc);
    if !state.fighters[PLAYER].active {
        state.run.over = true;
        state.winner = DRAW;
    }
}

/// Writes the run's pools into `out` (free entries as far-away padding), then the left wall and the arena's gate (padding while
/// open), with the blast zone and the respawn point.
pub fn build_stage(run: &Run, rules: &RunRules, out: &mut Stage) {
    let far = Fx::from_int(FAR);
    // (A pass-through platform whose right is left of its left: nothing ever spans it.)
    let padding = Platform {
        left: Fx::ONE,
        right: Fx::ZERO,
        y: far,
        bottom: far,
        pass_through: true,
    };
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
            padding
        });
    }
    let wall_top = rules.blast_top + Fx::from_int(WALL_ABOVE);
    let wall_bottom = Fx::from_int(data::BLOCK_BOTTOM);
    out.platforms.push(Platform {
        left: run.wall_x - rules.gate_thickness,
        right: run.wall_x,
        y: wall_top,
        bottom: wall_bottom,
        pass_through: false,
    });
    out.platforms.push(if run.gate_closed {
        Platform {
            left: run.gate_x,
            right: run.gate_x + rules.gate_thickness,
            y: wall_top,
            bottom: wall_bottom,
            pass_through: false,
        }
    } else {
        padding
    });
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
    // The left side is the wall: its blast line is behind it, out of reach.
    out.blast_left = run.wall_x - rules.gate_thickness - rules.blast_margin;
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

/// Where a lost life comes back: `respawn_at` across the view (inside the walls), on the highest ground there (or the first ground
/// to its right).
fn respawn_point(run: &Run, rules: &RunRules) -> Vec2 {
    let right = run.right_limit() - Fx::from_int(2);
    let x0 = (run.cam_x + rules.view_width * rules.respawn_at)
        .max(run.wall_x + Fx::from_int(2))
        .min(right);
    let steps = (rules.view_width / Fx::from_int(2)).floor_int().max(1);
    for k in 0..=steps {
        let x = x0 + Fx::from_int(k);
        if x > right {
            break;
        }
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

/// Rooms between shops, drawn from the rules' range.
fn draw_shop_due(run: &mut Run, rules: &RunRules) -> u8 {
    let lo = rules.shop_min.max(1);
    let hi = rules.shop_max.max(lo);
    lo + run.rng.range(u32::from(hi - lo) + 1) as u8
}

/// The next room's type: the first room is a street; a shop when one is due; else streets, arenas (never two in a row) and rest
/// stops by the rules' weights. A type with no rooms in the content is never picked.
fn next_kind(run: &mut Run, rc: &RunContent) -> u8 {
    if run.placed == 0 {
        return ROOM_STREET;
    }
    let has = |k: u8| rc.rooms.iter().any(|r| r.kind == k);
    run.since_shop = run.since_shop.saturating_add(1);
    if run.since_shop >= run.shop_due && has(ROOM_SHOP) {
        run.since_shop = 0;
        run.shop_due = draw_shop_due(run, &rc.rules);
        return ROOM_SHOP;
    }
    let r = &rc.rules;
    let weight = |k: u8| -> u32 {
        if !has(k) {
            return 0;
        }
        u32::from(match k {
            ROOM_ARENA if run.last_kind == ROOM_ARENA => 0,
            ROOM_REST if run.last_kind == ROOM_REST => 0,
            ROOM_ARENA => r.arena_weight,
            ROOM_REST => r.rest_weight,
            _ => r.street_weight,
        })
    };
    let kinds = [ROOM_STREET, ROOM_ARENA, ROOM_REST];
    let total: u32 = kinds.iter().map(|&k| weight(k)).sum();
    if total == 0 {
        return ROOM_STREET;
    }
    let mut roll = run.rng.range(total);
    for k in kinds {
        let w = weight(k);
        if roll < w {
            return k;
        }
        roll -= w;
    }
    ROOM_STREET
}

/// The next room: the first one to begin with, then one of the chosen type, never the first room again nor the last room laid
/// while another of that type will do.
fn next_room(run: &mut Run, rc: &RunContent) -> u8 {
    let kind = next_kind(run, rc);
    if run.placed == 0 {
        return 0;
    }
    let count = rc.rooms.len().min(255);
    let last = usize::from(run.last_room);
    let strict = |i: usize| i != 0 && i != last && rc.rooms[i].kind == kind;
    let loose = |i: usize| rc.rooms[i].kind == kind;
    let fits: &dyn Fn(usize) -> bool = if (0..count).any(strict) {
        &strict
    } else if (0..count).any(loose) {
        &loose
    } else {
        return 0;
    };
    let candidates = (0..count).filter(|&i| fits(i)).count() as u32;
    let mut n = run.rng.range(candidates);
    for i in 0..count {
        if fits(i) {
            if n == 0 {
                return i as u8;
            }
            n -= 1;
        }
    }
    0
}

/// Lays the next room at `next_x` in a free slot (false if the ring is full).
fn place_room(run: &mut Run, rc: &RunContent) -> bool {
    if rc.rooms.is_empty() {
        return false;
    }
    let Some(slot) = run.slots.iter().position(|s| !s.live) else {
        return false;
    };
    let id = next_room(run, rc);
    let room = &rc.rooms[usize::from(id)];
    let x0 = run.next_x;
    run.slots[slot] = RoomSlot {
        live: true,
        room: id,
        seq: run.placed,
        x: x0,
        end: x0 + room.width,
        spawned: 0,
    };
    for p in &room.platforms {
        let w = PoolPlatform {
            live: true,
            left: x0 + p.left,
            right: x0 + p.right,
            y: p.y,
            bottom: p.bottom,
            pass_through: p.pass_through,
        };
        // Ground carrying on from the last room at the same height is the same block, longer.
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
    for l in &room.ledges {
        if let Some(e) = run.ledges.iter_mut().find(|e| !e.live) {
            *e = PoolLedge {
                live: true,
                x: x0 + l.x,
                y: l.y,
                side: l.side,
            };
        }
    }
    run.last_room = id;
    run.last_kind = room.kind;
    run.kinds_log = run.kinds_log << 4 | u64::from(room.kind & 0xf);
    run.placed = run.placed.wrapping_add(1);
    run.next_x = x0 + room.width;
    run.layout = run.layout.wrapping_add(1);
    true
}

/// Drops the rooms more than `rooms_behind` behind the current one: their platforms (ground joined across the seam is trimmed to
/// the new wall), ledges and mobs go, and the left wall moves to the oldest room kept.
fn drop_behind(state: &mut GameState, rules: &RunRules) {
    let run = &mut state.run;
    let keep_from = run.room.saturating_sub(u32::from(rules.rooms_behind));
    let mut dropped = false;
    for s in run.slots.iter_mut() {
        if s.live && s.seq < keep_from {
            s.live = false;
            dropped = true;
        }
    }
    if !dropped {
        return;
    }
    let wall = run
        .slots
        .iter()
        .filter(|s| s.live)
        .map(|s| s.x)
        .min()
        .unwrap_or(run.wall_x);
    run.wall_x = wall;
    for (i, p) in run.platforms.iter_mut().enumerate() {
        if !p.live || p.left >= wall {
            continue;
        }
        let stood_on = state
            .fighters
            .iter()
            .any(|f| f.active && f.platform == i as i8);
        if p.right <= wall && !stood_on {
            *p = PoolPlatform::default();
        } else if p.right > wall {
            p.left = wall;
        }
    }
    for (i, l) in run.ledges.iter_mut().enumerate() {
        if l.live && l.x <= wall && state.ledge_owner[i] == NONE {
            *l = PoolLedge::default();
        }
    }
    for m in run.mobs.iter_mut().filter(|m| m.active) {
        if m.pos.x < wall {
            m.active = false;
        } else if usize::try_from(m.platform).is_ok_and(|i| !run.platforms[i].live) {
            m.platform = NONE;
        }
    }
    run.floor_x = run.floor_x.max(wall);
    run.layout = run.layout.wrapping_add(1);
}

/// The player walking into the next room makes it the current one: the oldest room behind is dropped, the next one ahead laid, and
/// the room does what its type does (an arena shuts its gate and wakes its mobs, a rest stop heals).
fn enter_rooms(state: &mut GameState, rc: &RunContent) {
    let f = state.fighters[PLAYER];
    if !f.active || f.state == S::Respawn {
        return;
    }
    let rules = &rc.rules;
    while !state.run.gate_closed {
        let Some(next) = state.run.slot_of(state.run.room.wrapping_add(1)).copied() else {
            return;
        };
        if f.pos.x < next.x + rules.enter_depth {
            return;
        }
        state.run.room = next.seq;
        drop_behind(state, rules);
        while state.run.placed <= state.run.room + u32::from(rules.rooms_ahead.max(1)) {
            if !place_room(&mut state.run, rc) {
                break;
            }
        }
        let room = &rc.rooms[usize::from(next.room).min(rc.rooms.len() - 1)];
        match room.kind {
            ROOM_ARENA => {
                let run = &mut state.run;
                run.gate_closed = true;
                run.gate_x = next.end;
                run.layout = run.layout.wrapping_add(1);
                // Everything in it comes out, awake.
                spawn_slot(run, rc, next.seq, None);
                for m in run.mobs.iter_mut() {
                    if m.active && m.room == next.seq && m.mode == MODE_IDLE {
                        m.mode = MODE_MOVE;
                    }
                }
            }
            ROOM_REST => {
                let g = &mut state.fighters[PLAYER];
                g.percent = (g.percent - rules.rest_heal).max(Fx::ZERO);
                state.run.last_percent = g.percent;
            }
            _ => {}
        }
    }
}

/// An arena whose mobs are all beaten (all its markers spawned, none of its mobs left) opens its gate and pays its bonus.
fn check_arena(state: &mut GameState, rc: &RunContent) {
    let run = &mut state.run;
    if !run.gate_closed {
        return;
    }
    let Some(slot) = run.slot_of(run.room).copied() else {
        run.gate_closed = false;
        return;
    };
    let room = &rc.rooms[usize::from(slot.room).min(rc.rooms.len() - 1)];
    let all = (1u16 << room.spawns.len().min(ROOM_SPAWNS)) - 1;
    let all_out = u16::from(slot.spawned) & all == all;
    let left = run
        .mobs
        .iter()
        .any(|m| m.active && m.room == slot.seq && m.mode != MODE_BURST);
    if all_out && !left {
        run.gate_closed = false;
        run.cleared = run.cleared.saturating_add(1);
        run.coins = run.coins.saturating_add(u32::from(rc.rules.arena_bonus));
        run.layout = run.layout.wrapping_add(1);
    }
}

/// Mobs from the spawn markers that have come within `spawn_ahead` of the view's right edge, in the current room and those before it,
/// and in the room ahead unless an arena's gate is shut before it.
fn spawn_mobs(run: &mut Run, rc: &RunContent) {
    let reach = run.cam_x + rc.rules.view_width + rc.rules.spawn_ahead;
    for s in 0..RING {
        let slot = run.slots[s];
        if !slot.live || slot.seq > run.room.wrapping_add(1) {
            continue;
        }
        if slot.seq > run.room && run.gate_closed {
            continue;
        }
        spawn_slot(run, rc, slot.seq, Some(reach));
    }
}

/// Spawns room `seq`'s markers not yet spawned (those left of `reach`, or all).
fn spawn_slot(run: &mut Run, rc: &RunContent, seq: u32, reach: Option<Fx>) {
    let Some(s) = run.slots.iter().position(|s| s.live && s.seq == seq) else {
        return;
    };
    let slot = run.slots[s];
    let room = &rc.rooms[usize::from(slot.room).min(rc.rooms.len() - 1)];
    for (j, marker) in room.spawns.iter().enumerate().take(ROOM_SPAWNS) {
        let bit = 1u8 << j;
        if slot.spawned & bit != 0 || reach.is_some_and(|r| slot.x + marker.x >= r) {
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
        // (A walker's floor: the pool entry it stands on.)
        let platform = if kind.behavior == MOB_WALKER {
            run.platforms
                .iter()
                .position(|p| p.live && p.y == pos.y && pos.x >= p.left && pos.x <= p.right)
                .map_or(NONE, |i| i as i8)
        } else {
            NONE
        };
        *m = Mob {
            active: true,
            kind: marker.kind,
            mode: MODE_IDLE,
            pos,
            facing: -1,
            platform,
            stamina: kind.stamina,
            room: seq,
            ..Mob::default()
        };
    }
}

/// The camera follows the fighter both ways (easing, at up to `cam_max_speed`), held inside the rooms in play: never past the left
/// wall, nor past the right end (or a shut gate). With a creep on, its left edge also never goes back past the creeping mark.
fn follow_camera(state: &mut GameState, rules: &RunRules) {
    let f = state.fighters[PLAYER];
    let creep = if f.state == S::Respawn {
        Fx::ZERO
    } else {
        state.run.creep(rules)
    };
    let run = &mut state.run;
    let lo = run.wall_x;
    let hi = (run.right_limit() - rules.view_width).max(lo);
    if creep > Fx::ZERO {
        run.floor_x = (run.floor_x + creep).max(lo).min(hi);
    }
    let mut target = run.cam_x;
    if f.active {
        target = f.pos.x - rules.view_width * rules.follow;
        run.best_x = run.best_x.max(f.pos.x);
    }
    if creeping(rules) {
        target = target.max(run.floor_x);
    }
    let target = target.max(lo).min(hi);
    let d = target - run.cam_x;
    let step = (d.abs() * rules.cam_ease)
        .max(Fx::from_ratio(1, 64))
        .min(rules.cam_max_speed)
        .min(d.abs());
    run.cam_x = if d >= Fx::ZERO {
        run.cam_x + step
    } else {
        run.cam_x - step
    };
    run.cam_x = run.cam_x.max(lo).min(hi);
}

/// Whether the optional creep is on (`RunRules::creep`).
fn creeping(rules: &RunRules) -> bool {
    rules.creep > Fx::ZERO || rules.creep_growth > Fx::ZERO
}

/// With a creep on: the creeping mark pushes a fighter behind it back in (not one flying from a hit). It never crushes: a fighter
/// it cannot move stays where it is.
fn push_from_edge(state: &mut GameState, world: &Content, rules: &RunRules) {
    let limit = state.run.floor_x + rules.push_margin;
    for i in 0..MAX_FIGHTERS {
        let f = state.fighters[i];
        if !f.active || f.pos.x >= limit {
            continue;
        }
        let pinned_states = matches!(
            f.state,
            S::LedgeHang | S::Grabbed | S::Grabbing | S::Carrying | S::Buried | S::Respawn
        );
        if f.state == S::Hitstun || pinned_states {
            continue;
        }
        let want = (limit - f.pos.x).min(rules.push_speed);
        let params = *params_of(world, &f);
        let g = &mut state.fighters[i];
        collision::move_x(&world.stage, &params, &mut g.pos, want);
        if let Some(p) = collision::platform(&world.stage, g.platform) {
            if g.pos.x > p.right {
                // Pushed off the end of its floor: it falls.
                g.platform = NONE;
                g.vel.y = Fx::ZERO;
                crate::fighter::stand_or_fall(g);
            }
        }
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

// ---- The shop -------------------------------------------------------------------------------------------------------------------

/// The shop whose door the player stands at now, on its feet and free to act: the room's number.
pub fn door_here(state: &GameState, rc: &RunContent) -> Option<u32> {
    let run = &state.run;
    let f = &state.fighters[PLAYER];
    if !run.active || run.over || !f.active || !f.grounded() {
        return None;
    }
    let free = matches!(
        f.state,
        S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch
    );
    if !free {
        return None;
    }
    run.slots.iter().filter(|s| s.live).find_map(|s| {
        let room = rc.rooms.get(usize::from(s.room))?;
        let door = Vec2::new(s.x + room.door.x, room.door.y);
        (room.kind == ROOM_SHOP
            && (f.pos.x - door.x).abs() <= rc.rules.door_reach
            && (f.pos.y - door.y).abs() < Fx::HALF)
            .then_some(s.seq)
    })
}

/// The stick pushed up this frame (it was not last frame).
fn pressed_up(run: &Run, input: Input) -> bool {
    input.stick_y >= STICK_THRESHOLD && run.prev_stick_y < STICK_THRESHOLD
}

fn remember_input(run: &mut Run, input: Input) {
    run.prev_stick_y = input.stick_y;
    run.prev_buttons = input.buttons;
}

/// Goes into the shop of room `seq`. A shop keeps what it has sold for as long as its room is in play.
fn open_shop(run: &mut Run, seq: u32) {
    if run.shop_room != seq {
        run.shop_room = seq;
        run.shop_bought = [0; MAX_SHOP_ITEMS];
    }
    run.shop_open = true;
    run.shop_cursor = 0;
    run.shop_note = NOTE_NONE;
}

/// A frame in the shop: up and down pick a ware (the row after the last is "leave"), attack or jump buys it (or leaves), shield or
/// special leaves.
fn shop_step(state: &mut GameState, rc: &RunContent, input: Input) {
    let items = rc.items.len().min(MAX_SHOP_ITEMS);
    let rows = items as u8 + 1;
    let run = &mut state.run;
    let pressed = input.buttons & !run.prev_buttons;
    if input.stick_y >= STICK_THRESHOLD && run.prev_stick_y < STICK_THRESHOLD {
        run.shop_cursor = (run.shop_cursor + rows - 1) % rows;
    }
    if input.stick_y <= -STICK_THRESHOLD && run.prev_stick_y > -STICK_THRESHOLD {
        run.shop_cursor = (run.shop_cursor + 1) % rows;
    }
    if pressed & (SHIELD | SPECIAL) != 0 {
        run.shop_open = false;
        return;
    }
    if pressed & (ATTACK | JUMP) == 0 {
        return;
    }
    let pick = usize::from(run.shop_cursor);
    if pick >= items {
        run.shop_open = false;
        return;
    }
    buy(state, rc, pick);
}

/// Buys ware `i`: refused if that shop has sold its limit, the coins are short, or it would do nothing.
pub fn buy(state: &mut GameState, rc: &RunContent, i: usize) -> u8 {
    let Some(item) = rc.items.get(i).filter(|_| i < MAX_SHOP_ITEMS) else {
        return NOTE_NONE;
    };
    let note = {
        let run = &state.run;
        let f = &state.fighters[PLAYER];
        if item.limit > 0 && run.shop_bought[i] >= item.limit {
            NOTE_SOLD_OUT
        } else if run.coins < u32::from(item.price) {
            NOTE_TOO_POOR
        } else if match item.kind {
            ITEM_HEAL => f.percent <= Fx::ZERO,
            ITEM_LIFE => f.stocks >= MatchRules::MAX_STOCKS,
            _ => true,
        } {
            NOTE_NOT_NEEDED
        } else {
            NOTE_BOUGHT
        }
    };
    if note == NOTE_BOUGHT {
        let f = &mut state.fighters[PLAYER];
        match item.kind {
            ITEM_HEAL => f.percent = (f.percent - item.amount).max(Fx::ZERO),
            _ => {
                let more = item.amount.floor_int().clamp(1, 9) as u8;
                f.stocks = f.stocks.saturating_add(more).min(MatchRules::MAX_STOCKS);
            }
        }
        let (percent, stocks) = (f.percent, f.stocks);
        let run = &mut state.run;
        run.coins -= u32::from(item.price);
        run.shop_bought[i] = run.shop_bought[i].saturating_add(1);
        run.last_percent = percent;
        run.last_stocks = stocks;
    }
    state.run.shop_note = note;
    state.run.shop_acts = state.run.shop_acts.wrapping_add(1);
    note
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
        h.write_u32(self.room);
    }
}

impl StateHash for Run {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        h.write_bool(self.over);
        self.rng.hash_into(h);
        self.cam_x.hash_into(h);
        self.floor_x.hash_into(h);
        self.start_x.hash_into(h);
        self.best_x.hash_into(h);
        self.next_x.hash_into(h);
        h.write_u32(self.placed);
        h.write_u8(self.last_room);
        h.write_u8(self.last_kind);
        h.write_u64(self.kinds_log);
        h.write_u8(self.since_shop);
        h.write_u8(self.shop_due);
        h.write_u32(self.room);
        self.wall_x.hash_into(h);
        h.write_bool(self.gate_closed);
        self.gate_x.hash_into(h);
        h.write_u16(self.cleared);
        for s in &self.slots {
            h.write_bool(s.live);
            h.write_u8(s.room);
            h.write_u32(s.seq);
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
        h.write_bool(self.shop_open);
        h.write_u8(self.shop_cursor);
        h.write_u32(self.shop_room);
        for b in self.shop_bought {
            h.write_u8(b);
        }
        h.write_u8(self.shop_note);
        h.write_u16(self.shop_acts);
        h.write_i8(self.prev_stick_y);
        h.write_u16(self.prev_buttons);
    }
}
