//! The single-player endless run (R1, sim 120; rooms R1b, sim 122; the world R2a, sim 124; `docs/modes/RUNNER_PLAN.md` and
//! `docs/modes/RUNNER_WORLD.md`).
//!
//! A run is an ordinary match with one fighter, played by the versus `step` with every versus rule, on a stage that streams. The run's
//! own state, [`Run`], lives in [`GameState::run`] and is hashed with the rest; in versus it is inactive and nothing here runs.
//!
//! * **The world is a chain of rooms**, left to right ([`data::Room`]: streets, forks, arenas, shops, rest stops). The rooms in play
//!   are a small window ([`RoomSlot`], a ring): the current room (the furthest the player has entered), up to
//!   `RunRules::rooms_behind` behind it and `rooms_ahead` ahead. Entering a new room lays the next one ahead and drops the oldest
//!   behind; the left edge of the oldest room kept is a **wall** (a solid block in the stage, so fighters and mobs collide with it and
//!   can wall-jump off it). An arena closes a second wall, its **gate**, on its right edge until every mob of the room is beaten.
//! * **Each room is laid as a variant** (R2a): mirrored or not, its optional parts on or off, an obstacle piece (or none) in each of
//!   its slots, all drawn from the run's generator as it is laid and kept in its [`RoomSlot`]. Its ground starts at the height the
//!   last room ended ([`Run::next_y`]) and the run's height drifts within `RunRules::drift`. The layout functions
//!   ([`for_each_footing`] and friends) turn a slot back into its geometry, coins and mobs, so the sim and the game agree.
//! * **The camera is in the sim**: the blast zone and mob spawning follow it, so it must replay exactly. It follows the player both
//!   ways, smoothly, clamped to the window's walls. The game only draws it (and follows height).
//! * **The stage streams.** The rooms in play lay their platforms and ledges into fixed pools ([`Run::platforms`], [`Run::ledges`]).
//!   A pool entry keeps its index for as long as it lives, so a fighter's `platform` and `ledge` (indices) stay right as rooms come
//!   and go; flat solid ground that continues across a seam at one height is one entry, extended (and trimmed when the room behind is
//!   dropped), so walking, dashing, rolling and wavedashing over a seam are what they are on any floor; a slope meeting other ground
//!   joins it through `collision::continuation`. Each frame [`build_stage`] writes the pools and the two walls into the run's working
//!   copy of the content's [`Stage`] (free entries as padding that nothing touches), with the blast zones and the respawn point, and
//!   the versus code collides with that stage as with any other.
//! * **Mobs** ([`Mob`], a fixed pool of [`MAX_MOBS`]) take the versus knockback, hitstun and hitlag formulas from `combat`, and hit the
//!   fighter through `combat::apply_hit` (shields, DI, crouch cancelling, armor, counters: everything a fighter's hit gets). They come
//!   out of a door or a stairwell first (`RunRules::emerge_frames`).
//! * **Coins** lie along the rooms (trails tracing the good line); walking through one picks it up.
//! * **The shop is a room you walk into** (R2a): pressing up at a storefront's door takes the fighter into the shop's interior
//!   ([`data::Interior`]), a small stage of its own the run swaps to while the world outside stands still. The wares stand on
//!   pedestals (walk up to one and press up to buy it), and the shopkeeper reacts ([`Run::keeper_mood`]). Pressing up at the door
//!   leaves, back to the street. All of it is in the sim, so a run with shopping replays exactly.

pub mod data;
mod mobs;

pub use data::{
    CoinSpot, Footing, Interior, MobKind, MobSpawn, Piece, PieceSlot, Room, RoomLedge, RunContent,
    RunRules, ShopItem, ITEM_HEAL, ITEM_LIFE, LOOK_NAMES, MOB_FLYER, MOB_WALKER, ROOM_ARENA,
    ROOM_REST, ROOM_SHOP, ROOM_STREET,
};

use crate::collision;
use crate::combat::params_of;
use crate::content::{Content, Ledge, Platform, Stage, MAX_LEDGES};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::input::{Input, STICK_THRESHOLD};
use crate::rng::Rng;
use crate::state::{Fighter, FighterState as S, GameState, MatchRules, DRAW, NONE};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;
use data::{
    BLOCK_BOTTOM, MAX_SHOP_ITEMS, ROOM_COINS, ROOM_LEDGES, ROOM_PARTS, ROOM_PLATFORMS, ROOM_SLOTS,
    ROOM_SPAWNS,
};

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
const _: () = assert!(ROOM_COINS <= 32);
const _: () = assert!(ROOM_PARTS <= 8);
const _: () = assert!(POOL_PLATFORMS + 2 <= i8::MAX as usize);
const _: () = assert!(data::INTERIOR_PLATFORMS <= POOL_PLATFORMS);

/// Where a free pool entry is put in the stage: far away, a platform no point is ever on and a ledge no fighter can reach.
const FAR: i32 = -30000;
/// How deep solid ground reaches once laid (world height; far below any blast zone the run's drift allows).
pub const DEEP: i32 = -80;
/// How far above the blast zone's top the walls reach (nothing gets over them or stands on them).
const WALL_ABOVE: i32 = 8;
/// Where the shop's interior is put while the fighter is inside (world x of its left edge; its floor is at height 0).
pub const INTERIOR_X: i32 = -20000;

/// `Run::shop_note` values: what the last buy did.
pub const NOTE_NONE: u8 = 0;
pub const NOTE_BOUGHT: u8 = 1;
pub const NOTE_TOO_POOR: u8 = 2;
pub const NOTE_SOLD_OUT: u8 = 3;
pub const NOTE_NOT_NEEDED: u8 = 4;

/// `Run::keeper_mood` values: what the shopkeeper is doing.
pub const KEEPER_IDLE: u8 = 0;
/// Waving hello as you come in.
pub const KEEPER_WAVE: u8 = 1;
/// Cheering a purchase.
pub const KEEPER_CHEER: u8 = 2;
/// Grumbling (cheerfully) at a buy that could not be made.
pub const KEEPER_GRUMBLE: u8 = 3;
/// Bonked: seeing stars ("hey!"). Never hurt, it only spins for a moment.
pub const KEEPER_DIZZY: u8 = 4;

/// One entry of the stage's platform pool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolPlatform {
    pub live: bool,
    pub left: Fx,
    pub right: Fx,
    pub y: Fx,
    pub bottom: Fx,
    pub pass_through: bool,
    pub rise: Fx,
}

impl PoolPlatform {
    fn platform(&self) -> Platform {
        Platform {
            left: self.left,
            right: self.right,
            y: self.y,
            bottom: self.bottom,
            pass_through: self.pass_through,
            rise: self.rise,
        }
    }
}

/// One entry of the stage's ledge pool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolLedge {
    pub live: bool,
    pub x: Fx,
    pub y: Fx,
    pub side: i8,
}

/// A room in play: which one, its number in the run, where it is, the variant it was laid as, which of its spawn markers have made
/// their mob and which of its coins are taken.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoomSlot {
    pub live: bool,
    /// Index into `RunContent::rooms`.
    pub room: u8,
    /// Its number in the run (0 is the first room).
    pub seq: u32,
    /// Its left and right edges (world x), and its ground's height where it begins and where it ends.
    pub x: Fx,
    pub end: Fx,
    pub y: Fx,
    pub y_end: Fx,
    /// The variant: mirrored, which optional parts are on (bit `n - 1` for part `n`), and the piece in each slot (0 none, else the
    /// piece's index plus 1).
    pub mirror: bool,
    pub parts: u8,
    pub pieces: [u8; ROOM_SLOTS],
    /// Spawn markers that have made their mob (or are not in this variant), by the marker's index.
    pub spawned: u8,
    /// Coins taken, by the coin's number ([`for_each_coin`]).
    pub coins: u32,
}

/// `Mob::mode` values.
pub const MODE_IDLE: u8 = 0;
pub const MODE_MOVE: u8 = 1;
pub const MODE_WINDUP: u8 = 2;
pub const MODE_ATTACK: u8 = 3;
pub const MODE_COOLDOWN: u8 = 4;
/// Beaten: it flies off with its last launch for `RunRules::burst_frames`, then is gone. It cannot hit or be hit.
pub const MODE_BURST: u8 = 5;
/// Coming out of its door or stairwell for `RunRules::emerge_frames` (R2a): it neither moves, hits nor can be hit.
pub const MODE_EMERGE: u8 = 6;

/// An enemy (see `run::mobs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mob {
    pub active: bool,
    /// Index into `RunContent::mobs`.
    pub kind: u8,
    /// `MODE_*`, and the frames left in it (the windup, attack, cooldown, burst or emergence).
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
    /// Where the next room's left edge goes, and the height its ground begins at.
    pub next_x: Fx,
    pub next_y: Fx,
    /// Rooms laid so far (the next one's number), the last one laid (index into the content) and its type.
    pub placed: u32,
    pub last_room: u8,
    pub last_kind: u8,
    /// The type of each of the last 16 rooms laid, 4 bits each, newest in the low bits (for the map): the type in the low 3 bits,
    /// and 8 for a fork.
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
    /// Coins picked up along the way (a count, for the game's cues).
    pub coins_picked: u16,
    pub kills: u16,
    /// Hits and kills in a row without being hit, and the longest such run.
    pub combo: u16,
    pub best_combo: u16,
    /// The fighter's percent and lives at the end of the last frame (a rise, or a lost life, breaks the combo).
    pub last_percent: Fx,
    pub last_stocks: u8,
    /// Inside a shop (the world outside stands still), the room whose shop it is, how many of each ware that shop has sold, what
    /// the last buy did (`NOTE_*`) and how many buys have been tried (for the game's cues).
    pub inside: bool,
    pub shop_room: u32,
    pub shop_bought: [u8; MAX_SHOP_ITEMS],
    pub shop_note: u8,
    pub shop_acts: u16,
    /// The shopkeeper: what it is doing (`KEEPER_*`) and for how many more frames, the ware the player stands by (it looks at it;
    /// -1 none), and how many times it has been bonked.
    pub keeper_mood: u8,
    pub keeper_timer: u16,
    pub keeper_look: i8,
    pub keeper_bonks: u16,
    /// The player's stick height last frame (the door and the wares read presses, not holds).
    pub prev_stick_y: i8,
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
        next_y: Fx::ZERO,
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
            y: Fx::ZERO,
            y_end: Fx::ZERO,
            mirror: false,
            parts: 0,
            pieces: [0; ROOM_SLOTS],
            spawned: 0,
            coins: 0,
        }; RING],
        platforms: [PoolPlatform {
            live: false,
            left: Fx::ZERO,
            right: Fx::ZERO,
            y: Fx::ZERO,
            bottom: Fx::ZERO,
            pass_through: false,
            rise: Fx::ZERO,
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
        coins_picked: 0,
        kills: 0,
        combo: 0,
        best_combo: 0,
        last_percent: Fx::ZERO,
        last_stocks: 0,
        inside: false,
        shop_room: 0,
        shop_bought: [0; MAX_SHOP_ITEMS],
        shop_note: NOTE_NONE,
        shop_acts: 0,
        keeper_mood: KEEPER_IDLE,
        keeper_timer: 0,
        keeper_look: -1,
        keeper_bonks: 0,
        prev_stick_y: 0,
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

    /// The blast zone's bottom and top: below the lowest ground line of the rooms in play and above the highest.
    pub fn blast_y(&self, rules: &RunRules) -> (Fx, Fx) {
        let live = || self.slots.iter().filter(|s| s.live);
        let lo = live().map(|s| s.y.min(s.y_end)).min().unwrap_or(Fx::ZERO);
        let hi = live().map(|s| s.y.max(s.y_end)).max().unwrap_or(Fx::ZERO);
        (lo + rules.blast_bottom, hi + rules.blast_top)
    }

    /// The type of room `seq` (`ROOM_*`), if it is among the last 16 laid.
    pub fn kind_of(&self, seq: u32) -> Option<u8> {
        self.log_of(seq).map(|k| k & 7)
    }

    /// Whether room `seq` is a fork, if it is among the last 16 laid.
    pub fn fork_of(&self, seq: u32) -> Option<bool> {
        self.log_of(seq).map(|k| k & 8 != 0)
    }

    fn log_of(&self, seq: u32) -> Option<u8> {
        let back = self.placed.checked_sub(seq + 1)?;
        (back < 16).then(|| ((self.kinds_log >> (back * 4)) & 0xf) as u8)
    }
}

/// The fighter the run is about (the one player).
pub const PLAYER: usize = 0;

// ---- Laying a room out --------------------------------------------------------------------------------------------------------

/// Whether something of optional part `part` (0 always; `n` with part `n` on; `-n` with it off) is there with these parts on.
pub fn part_on(parts: u8, part: i8) -> bool {
    match part {
        0 => true,
        n if n > 0 => parts & (1 << (n - 1).min(7)) != 0,
        n => parts & (1 << (-n - 1).min(7)) == 0,
    }
}

/// The ground's height at a room's right edge, mirrored or not.
pub fn room_exit(room: &Room, mirror: bool) -> Fx {
    if mirror {
        -room.exit_y
    } else {
        room.exit_y
    }
}

/// From a room's own coordinates to the world, for one laid variant.
#[derive(Clone, Copy)]
struct Frame {
    x0: Fx,
    y0: Fx,
    width: Fx,
    exit_y: Fx,
    mirror: bool,
}

impl Frame {
    fn of(slot: &RoomSlot, room: &Room) -> Frame {
        Frame {
            x0: slot.x,
            y0: slot.y,
            width: room.width,
            exit_y: room.exit_y,
            mirror: slot.mirror,
        }
    }

    fn x(&self, x: Fx) -> Fx {
        self.x0 + if self.mirror { self.width - x } else { x }
    }

    fn y(&self, y: Fx) -> Fx {
        self.y0 + if self.mirror { y - self.exit_y } else { y }
    }

    fn point(&self, p: Vec2) -> Vec2 {
        Vec2::new(self.x(p.x), self.y(p.y))
    }

    /// A platform of the room, placed (solid ground reaching [`data::BLOCK_BOTTOM`] goes down to [`DEEP`]).
    fn platform(&self, p: &Platform) -> Platform {
        let (left, right, y, rise) = if self.mirror {
            (
                self.x(p.right),
                self.x(p.left),
                self.y(p.top_right()),
                -p.rise,
            )
        } else {
            (self.x(p.left), self.x(p.right), self.y(p.y), p.rise)
        };
        let bottom = if p.pass_through {
            y
        } else if p.bottom <= Fx::from_int(BLOCK_BOTTOM) {
            Fx::from_int(DEEP)
        } else {
            self.y(p.bottom)
        };
        Platform {
            left,
            right,
            y,
            bottom,
            pass_through: p.pass_through,
            rise,
        }
    }
}

/// A footing as laid in the world: its platform, its look, and the height its art stands on (its base: the ground under a piece,
/// a block's bottom, a plank's own height).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub platform: Platform,
    pub look: u8,
    pub base: Fx,
}

/// A slot's piece, if it has one: the piece and its anchor in the room's own coordinates.
fn slot_pieces<'a>(
    rc: &'a RunContent,
    room: &'a Room,
    slot: &'a RoomSlot,
) -> impl Iterator<Item = (&'a Piece, Vec2)> + 'a {
    room.slots
        .iter()
        .zip(slot.pieces.iter())
        .filter_map(move |(s, &p)| {
            let piece = rc.pieces.get(usize::from(p).checked_sub(1)?)?;
            Some((piece, Vec2::new(s.x, s.y)))
        })
}

/// Every footing of a laid room (its own, then its pieces'), in the world.
pub fn for_each_footing(rc: &RunContent, slot: &RoomSlot, mut f: impl FnMut(Placed)) {
    let Some(room) = rc.rooms.get(usize::from(slot.room)) else {
        return;
    };
    let frame = Frame::of(slot, room);
    for p in room
        .platforms
        .iter()
        .filter(|p| part_on(slot.parts, p.part))
    {
        let platform = frame.platform(&p.platform);
        f(Placed {
            platform,
            look: p.look,
            base: platform.bottom,
        });
    }
    for (piece, at) in slot_pieces(rc, room, slot) {
        let base = frame.y(at.y);
        for p in &piece.platforms {
            let mut local = p.platform;
            local.left += at.x;
            local.right += at.x;
            local.y += at.y;
            if local.bottom > Fx::from_int(BLOCK_BOTTOM) {
                local.bottom += at.y;
            }
            f(Placed {
                platform: frame.platform(&local),
                look: p.look,
                base,
            });
        }
    }
}

/// Every coin of a laid room that is there in its variant: its number (its bit in `RoomSlot::coins`) and where it is.
pub fn for_each_coin(rc: &RunContent, slot: &RoomSlot, mut f: impl FnMut(usize, Vec2)) {
    let Some(room) = rc.rooms.get(usize::from(slot.room)) else {
        return;
    };
    let frame = Frame::of(slot, room);
    let mut n = 0;
    for c in &room.coins {
        if part_on(slot.parts, c.part) && n < ROOM_COINS {
            f(n, frame.point(Vec2::new(c.x, c.y)));
        }
        n += 1;
    }
    for (piece, at) in slot_pieces(rc, room, slot) {
        for c in &piece.coins {
            if n < ROOM_COINS {
                f(n, frame.point(Vec2::new(at.x + c.x, at.y + c.y)));
            }
            n += 1;
        }
    }
}

/// Every ledge of a laid room that is there in its variant, in the world.
pub fn for_each_ledge(rc: &RunContent, slot: &RoomSlot, mut f: impl FnMut(Ledge)) {
    let Some(room) = rc.rooms.get(usize::from(slot.room)) else {
        return;
    };
    let frame = Frame::of(slot, room);
    for l in room.ledges.iter().filter(|l| part_on(slot.parts, l.part)) {
        f(Ledge {
            x: frame.x(l.ledge.x),
            y: frame.y(l.ledge.y),
            side: if slot.mirror {
                -l.ledge.side
            } else {
                l.ledge.side
            },
        });
    }
}

/// A shop room's door (feet), in the world.
pub fn door_of(rc: &RunContent, slot: &RoomSlot) -> Option<Vec2> {
    let room = rc.rooms.get(usize::from(slot.room))?;
    (room.kind == ROOM_SHOP).then(|| Frame::of(slot, room).point(room.door))
}

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
    build_stage(&state.run, rc, &mut world.stage);
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
/// gate), the camera, the coins and combo, and the shop's door. While inside a shop only the shop moves. `world` is the run's
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
    build_stage(&state.run, rc, &mut world.stage);
    let input = inputs[PLAYER];
    if state.run.over {
        remember_input(&mut state.run, input);
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    if state.run.inside {
        crate::step(state, world, inputs);
        shop_step(state, world, rc, input);
        remember_input(&mut state.run, input);
        build_stage(&state.run, rc, &mut world.stage);
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
    collect_coins(state, rc);
    settle_combo(state);
    check_arena(state, rc);
    if let Some(seq) = door_here(state, rc).filter(|_| pressed_up(&state.run, input)) {
        enter_shop(state, world, rc, seq);
    }
    remember_input(&mut state.run, input);
    build_stage(&state.run, rc, &mut world.stage);
    spawn_mobs(&mut state.run, rc);
    if !state.fighters[PLAYER].active {
        state.run.over = true;
        state.winner = DRAW;
    }
}

/// Writes the run's stage into `out`: outside, the pools (free entries as far-away padding), then the left wall and the arena's
/// gate (padding while open), with the blast zone and the respawn point; inside a shop, the shop's interior instead.
pub fn build_stage(run: &Run, rc: &RunContent, out: &mut Stage) {
    let rules = &rc.rules;
    let far = Fx::from_int(FAR);
    // (A pass-through platform whose right is left of its left: nothing ever spans it.)
    let padding = Platform {
        left: Fx::ONE,
        right: Fx::ZERO,
        y: far,
        bottom: far,
        pass_through: true,
        rise: Fx::ZERO,
    };
    let far_ledge = Ledge {
        x: far,
        y: far,
        side: -1,
    };
    out.platforms.clear();
    out.ledges.clear();
    if run.inside {
        let at = Fx::from_int(INTERIOR_X);
        for p in &rc.interior.platforms {
            let mut q = p.platform;
            q.left += at;
            q.right += at;
            out.platforms.push(q);
        }
        while out.platforms.len() < POOL_PLATFORMS + 2 {
            out.platforms.push(padding);
        }
        out.ledges.resize(POOL_LEDGES, far_ledge);
        let w = rc.interior.width;
        out.blast_left = at - Fx::from_int(10);
        out.blast_right = at + w + Fx::from_int(10);
        out.blast_bottom = Fx::from_int(-20);
        out.blast_top = Fx::from_int(30);
        out.spawns = [
            Vec2::new(at + rc.interior.enter, Fx::ZERO),
            Vec2::new(far, far),
            Vec2::new(far, far),
            Vec2::new(far, far),
        ];
        return;
    }
    for p in &run.platforms {
        out.platforms
            .push(if p.live { p.platform() } else { padding });
    }
    let (blast_bottom, blast_top) = run.blast_y(rules);
    let wall_top = blast_top + Fx::from_int(WALL_ABOVE);
    let wall_bottom = Fx::from_int(DEEP);
    out.platforms.push(Platform {
        left: run.wall_x - rules.gate_thickness,
        right: run.wall_x,
        y: wall_top,
        bottom: wall_bottom,
        pass_through: false,
        rise: Fx::ZERO,
    });
    out.platforms.push(if run.gate_closed {
        Platform {
            left: run.gate_x,
            right: run.gate_x + rules.gate_thickness,
            y: wall_top,
            bottom: wall_bottom,
            pass_through: false,
            rise: Fx::ZERO,
        }
    } else {
        padding
    });
    for l in &run.ledges {
        out.ledges.push(if l.live {
            Ledge {
                x: l.x,
                y: l.y,
                side: l.side,
            }
        } else {
            far_ledge
        });
    }
    // The left side is the wall: its blast line is behind it, out of reach.
    out.blast_left = run.wall_x - rules.gate_thickness - rules.blast_margin;
    out.blast_right = run.cam_x + rules.view_width + rules.blast_margin;
    out.blast_bottom = blast_bottom;
    out.blast_top = blast_top;
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
    let (_, top) = run.blast_y(rules);
    let steps = (rules.view_width / Fx::from_int(2)).floor_int().max(1);
    for k in 0..=steps {
        let x = x0 + Fx::from_int(k);
        if x > right {
            break;
        }
        let ground = run
            .platforms
            .iter()
            .filter(|p| p.live && x >= p.left && x <= p.right)
            .map(|p| p.platform().top_at(x))
            .filter(|&y| y < top)
            .max();
        if let Some(y) = ground {
            return Vec2::new(x, y);
        }
    }
    Vec2::new(x0, run.next_y)
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
/// while another of that type will do. A street is a fork `fork_chance` percent of the time (when the content has forks).
fn next_room(run: &mut Run, rc: &RunContent) -> u8 {
    let kind = next_kind(run, rc);
    if run.placed == 0 {
        return 0;
    }
    let count = rc.rooms.len().min(255);
    let forks = (1..count).any(|i| rc.rooms[i].kind == kind && rc.rooms[i].fork);
    let plain = (1..count).any(|i| rc.rooms[i].kind == kind && !rc.rooms[i].fork);
    // (A street draws whether to fork; other types take any room of theirs.)
    let want_fork = if kind == ROOM_STREET && forks && plain {
        Some(run.rng.range(100) < u32::from(rc.rules.fork_chance))
    } else {
        None
    };
    let last = usize::from(run.last_room);
    let of_kind = |i: usize| i != 0 && rc.rooms[i].kind == kind;
    let forked = |i: usize| want_fork.is_none_or(|w| rc.rooms[i].fork == w);
    // (The best choices first: the right type and fork-ness, not the last room laid; then letting go of the last room, then of
    // the fork-ness; the start room only if nothing else has the type.)
    let tiers: [&dyn Fn(usize) -> bool; 5] = [
        &|i| of_kind(i) && forked(i) && i != last,
        &|i| of_kind(i) && forked(i),
        &|i| of_kind(i) && i != last,
        &of_kind,
        &|i| rc.rooms[i].kind == kind,
    ];
    let Some(fits) = tiers.into_iter().find(|t| (0..count).any(t)) else {
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

/// Lays the next room at `next_x` (its ground at `next_y`) in a free slot, drawing its variant (false if the ring is full).
fn place_room(run: &mut Run, rc: &RunContent) -> bool {
    if rc.rooms.is_empty() {
        return false;
    }
    let Some(index) = run.slots.iter().position(|s| !s.live) else {
        return false;
    };
    let id = next_room(run, rc);
    let room = &rc.rooms[usize::from(id)];
    // The variant: mirrored (never leaving the drift band if the other way keeps inside it), the optional parts, the pieces.
    let mut mirror = room.mirror && run.rng.range(2) == 1;
    let drift = rc.rules.drift;
    if room.mirror && (run.next_y + room_exit(room, mirror)).abs() > drift {
        mirror = !mirror;
    }
    let mut parts = 0u8;
    for (n, &chance) in room.parts.iter().enumerate().take(ROOM_PARTS) {
        if run.rng.range(100) < u32::from(chance) {
            parts |= 1 << n;
        }
    }
    let mut pieces = [0u8; ROOM_SLOTS];
    for (n, s) in room.slots.iter().enumerate().take(ROOM_SLOTS) {
        if !part_on(parts, s.part) || run.rng.range(100) >= u32::from(rc.rules.piece_chance) {
            continue;
        }
        let fits = |p: &Piece| p.fits & (1 << s.kind.min(7)) != 0;
        let count = rc.pieces.iter().filter(|p| fits(p)).count().min(254) as u32;
        if count == 0 {
            continue;
        }
        let pick = run.rng.range(count) as usize;
        if let Some(i) = rc
            .pieces
            .iter()
            .enumerate()
            .filter(|(_, p)| fits(p))
            .nth(pick)
            .map(|(i, _)| i)
        {
            pieces[n] = (i + 1) as u8;
        }
    }
    // Spawn markers not in this variant count as spawned.
    let mut spawned = 0u8;
    for (j, s) in room.spawns.iter().enumerate().take(ROOM_SPAWNS) {
        if !part_on(parts, s.part) {
            spawned |= 1 << j;
        }
    }
    let x0 = run.next_x;
    let y0 = run.next_y;
    let slot = RoomSlot {
        live: true,
        room: id,
        seq: run.placed,
        x: x0,
        end: x0 + room.width,
        y: y0,
        y_end: y0 + room_exit(room, mirror),
        mirror,
        parts,
        pieces,
        spawned,
        coins: 0,
    };
    run.slots[index] = slot;
    let platforms = &mut run.platforms;
    for_each_footing(rc, &slot, |placed| {
        let p = placed.platform;
        let w = PoolPlatform {
            live: true,
            left: p.left,
            right: p.right,
            y: p.y,
            bottom: p.bottom,
            pass_through: p.pass_through,
            rise: p.rise,
        };
        // Flat solid ground carrying on from the last room at the same height is the same block, longer.
        let joined = platforms.iter_mut().find(|e| {
            e.live
                && !w.pass_through
                && !e.pass_through
                && e.rise == Fx::ZERO
                && w.rise == Fx::ZERO
                && e.right == w.left
                && e.y == w.y
                && e.left < x0
        });
        if let Some(e) = joined {
            e.right = w.right;
            e.bottom = e.bottom.min(w.bottom);
            return;
        }
        if let Some(e) = platforms.iter_mut().find(|e| !e.live) {
            *e = w;
        }
    });
    let ledges = &mut run.ledges;
    for_each_ledge(rc, &slot, |l| {
        if let Some(e) = ledges.iter_mut().find(|e| !e.live) {
            *e = PoolLedge {
                live: true,
                x: l.x,
                y: l.y,
                side: l.side,
            };
        }
    });
    run.last_room = id;
    run.last_kind = room.kind;
    run.kinds_log = run.kinds_log << 4 | u64::from(room.kind & 7 | u8::from(room.fork) << 3);
    run.placed = run.placed.wrapping_add(1);
    run.next_x = slot.end;
    run.next_y = slot.y_end;
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
            // (A slope keeps its line: its left end moves along it.)
            let top_right = p.y + p.rise;
            p.y = p.platform().top_at(wall);
            p.rise = top_right - p.y;
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

/// Spawns room `seq`'s markers not yet spawned (those left of `reach`, or all). A mob comes out of its door or stairwell first
/// (`emerge_frames`); a walker stands on the ground under its marker.
fn spawn_slot(run: &mut Run, rc: &RunContent, seq: u32, reach: Option<Fx>) {
    let Some(s) = run.slots.iter().position(|s| s.live && s.seq == seq) else {
        return;
    };
    let slot = run.slots[s];
    let room = &rc.rooms[usize::from(slot.room).min(rc.rooms.len() - 1)];
    let frame = Frame::of(&slot, room);
    for (j, marker) in room.spawns.iter().enumerate().take(ROOM_SPAWNS) {
        let bit = 1u8 << j;
        let pos = frame.point(Vec2::new(marker.x, marker.y));
        if slot.spawned & bit != 0 || reach.is_some_and(|r| pos.x >= r) {
            continue;
        }
        run.slots[s].spawned |= bit;
        let Some(kind) = rc.mobs.get(usize::from(marker.kind)) else {
            continue;
        };
        let Some(m) = run.mobs.iter().position(|m| !m.active) else {
            continue;
        };
        let mut pos = pos;
        // (A walker's floor: the highest pool entry under its marker, within half a unit.)
        let mut platform = NONE;
        if kind.behavior == MOB_WALKER {
            let mut best: Option<(usize, Fx)> = None;
            for (i, p) in run.platforms.iter().enumerate() {
                if !p.live || pos.x < p.left || pos.x > p.right {
                    continue;
                }
                let top = p.platform().top_at(pos.x);
                if (top - pos.y).abs() <= Fx::HALF && best.is_none_or(|(_, b)| top > b) {
                    best = Some((i, top));
                }
            }
            if let Some((i, top)) = best {
                platform = i as i8;
                pos.y = top;
            }
        }
        let emerge = rc.rules.emerge_frames;
        run.mobs[m] = Mob {
            active: true,
            kind: marker.kind,
            mode: if emerge > 0 { MODE_EMERGE } else { MODE_IDLE },
            timer: u16::from(emerge),
            pos,
            facing: -1,
            platform,
            stamina: kind.stamina,
            room: seq,
            ..Mob::default()
        };
    }
}

/// Coins the player walks (or flies) through: picked up, paying `coin_value` each.
fn collect_coins(state: &mut GameState, rc: &RunContent) {
    let f = state.fighters[PLAYER];
    if !f.active || f.state == S::Respawn {
        return;
    }
    let body = f.pos + Vec2::new(Fx::ZERO, Fx::ONE);
    let reach = rc.rules.coin_reach;
    let run = &mut state.run;
    for s in 0..RING {
        let slot = run.slots[s];
        if !slot.live {
            continue;
        }
        let mut taken = slot.coins;
        for_each_coin(rc, &slot, |n, at| {
            let d = at - body;
            if taken & (1 << n) == 0
                && d.x.abs() <= reach
                && d.y.abs() <= reach
                && d.x * d.x + d.y * d.y <= reach * reach
            {
                taken |= 1 << n;
            }
        });
        let new = (taken & !slot.coins).count_ones();
        if new > 0 {
            run.slots[s].coins = taken;
            run.coins = run
                .coins
                .saturating_add(u32::from(rc.rules.coin_value) * new);
            run.coins_picked = run.coins_picked.saturating_add(new as u16);
        }
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
        if collision::platform(&world.stage, g.platform).is_some()
            && !crate::fighter::keep_on_ground(g, &world.stage)
        {
            // Pushed off the end of its floor: it falls.
            g.platform = NONE;
            g.vel.y = Fx::ZERO;
            crate::fighter::stand_or_fall(g);
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

/// The fighter stands on the ground, free to act (not attacking, dodging, hurt or hanging).
fn free_on_ground(f: &Fighter) -> bool {
    f.active
        && f.grounded()
        && matches!(
            f.state,
            S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch
        )
}

/// The shop whose door the player stands at now, on its feet and free to act: the room's number.
pub fn door_here(state: &GameState, rc: &RunContent) -> Option<u32> {
    let run = &state.run;
    let f = &state.fighters[PLAYER];
    if !run.active || run.over || run.inside || !free_on_ground(f) {
        return None;
    }
    run.slots.iter().filter(|s| s.live).find_map(|s| {
        let door = door_of(rc, s)?;
        ((f.pos.x - door.x).abs() <= rc.rules.door_reach && (f.pos.y - door.y).abs() < Fx::HALF)
            .then_some(s.seq)
    })
}

/// Inside the shop, where the player stands: `Some(None)` at the way out, `Some(Some(i))` by ware `i`, `None` elsewhere (or not
/// free to act). The door wins where both are in reach.
pub fn shop_spot(state: &GameState, rc: &RunContent) -> Option<Option<usize>> {
    let run = &state.run;
    let f = &state.fighters[PLAYER];
    if !run.active || !run.inside || !free_on_ground(f) {
        return None;
    }
    let x = f.pos.x - Fx::from_int(INTERIOR_X);
    let reach = rc.rules.door_reach;
    if (x - rc.interior.door).abs() <= reach {
        return Some(None);
    }
    ware_near(rc, x).map(Some)
}

/// The ware whose pedestal is nearest `x` (interior coordinates), within reach.
fn ware_near(rc: &RunContent, x: Fx) -> Option<usize> {
    let wares = rc.items.len().min(MAX_SHOP_ITEMS);
    rc.interior
        .pedestals
        .iter()
        .take(wares)
        .enumerate()
        .map(|(i, p)| (i, (p.x - x).abs()))
        .filter(|(_, d)| *d <= rc.rules.door_reach)
        .min_by_key(|(_, d)| *d)
        .map(|(i, _)| i)
}

/// The stick pushed up this frame (it was not last frame).
fn pressed_up(run: &Run, input: Input) -> bool {
    input.stick_y >= STICK_THRESHOLD && run.prev_stick_y < STICK_THRESHOLD
}

fn remember_input(run: &mut Run, input: Input) {
    run.prev_stick_y = input.stick_y;
}

/// Puts the fighter on its feet at `at` (a door), still, on whatever ground is there.
fn place_at(f: &mut Fighter, at: Vec2, stage: &Stage) {
    f.pos = at;
    f.vel = Vec2::ZERO;
    f.kb_vel = Vec2::ZERO;
    f.ledge = NONE;
    f.platform = collision::standing_on(stage, at);
    crate::fighter::stand_or_fall(f);
}

/// Walks in through room `seq`'s door: the interior becomes the stage, the fighter stands inside its door, and the shopkeeper
/// waves. A shop keeps what it has sold for as long as its room is in play.
fn enter_shop(state: &mut GameState, world: &mut Content, rc: &RunContent, seq: u32) {
    let run = &mut state.run;
    if run.shop_room != seq {
        run.shop_room = seq;
        run.shop_bought = [0; MAX_SHOP_ITEMS];
    }
    run.inside = true;
    run.shop_note = NOTE_NONE;
    run.keeper_mood = KEEPER_WAVE;
    run.keeper_timer = rc.interior.wave_frames;
    run.keeper_look = -1;
    build_stage(&state.run, rc, &mut world.stage);
    let at = Vec2::new(Fx::from_int(INTERIOR_X) + rc.interior.enter, Fx::ZERO);
    let f = &mut state.fighters[PLAYER];
    place_at(f, at, &world.stage);
    f.facing = 1;
}

/// Walks out of the shop: the street becomes the stage again and the fighter stands at the storefront's door.
fn leave_shop(state: &mut GameState, world: &mut Content, rc: &RunContent) {
    let run = &mut state.run;
    run.inside = false;
    run.keeper_mood = KEEPER_IDLE;
    run.keeper_timer = 0;
    run.keeper_look = -1;
    let door = run
        .slot_of(run.shop_room)
        .and_then(|s| door_of(rc, s))
        .unwrap_or(Vec2::new(
            run.cam_x + rc.rules.view_width * rc.rules.follow,
            run.next_y,
        ));
    build_stage(&state.run, rc, &mut world.stage);
    place_at(&mut state.fighters[PLAYER], door, &world.stage);
}

/// A frame inside the shop (after the fighter's own step): the shopkeeper's moods and bonks, buying at a pedestal, leaving by the
/// door.
fn shop_step(state: &mut GameState, world: &mut Content, rc: &RunContent, input: Input) {
    let shop = &rc.interior;
    if mobs::keeper_bonked(state, world, rc) && state.run.keeper_mood != KEEPER_DIZZY {
        let run = &mut state.run;
        run.keeper_mood = KEEPER_DIZZY;
        run.keeper_timer = shop.dizzy_frames;
        run.keeper_bonks = run.keeper_bonks.saturating_add(1);
    }
    let run = &mut state.run;
    if run.keeper_timer > 0 {
        run.keeper_timer -= 1;
        if run.keeper_timer == 0 {
            run.keeper_mood = KEEPER_IDLE;
        }
    }
    let f = state.fighters[PLAYER];
    let x = f.pos.x - Fx::from_int(INTERIOR_X);
    state.run.keeper_look = if f.active {
        ware_near(rc, x).map_or(-1, |i| i as i8)
    } else {
        -1
    };
    if !pressed_up(&state.run, input) {
        return;
    }
    match shop_spot(state, rc) {
        Some(None) => leave_shop(state, world, rc),
        Some(Some(i)) => {
            let note = buy(state, rc, i);
            let run = &mut state.run;
            if run.keeper_mood != KEEPER_DIZZY {
                (run.keeper_mood, run.keeper_timer) = if note == NOTE_BOUGHT {
                    (KEEPER_CHEER, shop.cheer_frames)
                } else {
                    (KEEPER_GRUMBLE, shop.grumble_frames)
                };
            }
        }
        None => {}
    }
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
        self.next_y.hash_into(h);
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
            s.y.hash_into(h);
            s.y_end.hash_into(h);
            h.write_bool(s.mirror);
            h.write_u8(s.parts);
            for p in s.pieces {
                h.write_u8(p);
            }
            h.write_u8(s.spawned);
            h.write_u32(s.coins);
        }
        for p in &self.platforms {
            h.write_bool(p.live);
            p.left.hash_into(h);
            p.right.hash_into(h);
            p.y.hash_into(h);
            p.bottom.hash_into(h);
            h.write_bool(p.pass_through);
            p.rise.hash_into(h);
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
        h.write_u16(self.coins_picked);
        h.write_u16(self.kills);
        h.write_u16(self.combo);
        h.write_u16(self.best_combo);
        self.last_percent.hash_into(h);
        h.write_u8(self.last_stocks);
        h.write_bool(self.inside);
        h.write_u32(self.shop_room);
        for b in self.shop_bought {
            h.write_u8(b);
        }
        h.write_u8(self.shop_note);
        h.write_u16(self.shop_acts);
        h.write_u8(self.keeper_mood);
        h.write_u16(self.keeper_timer);
        h.write_i8(self.keeper_look);
        h.write_u16(self.keeper_bonks);
        h.write_i8(self.prev_stick_y);
    }
}
