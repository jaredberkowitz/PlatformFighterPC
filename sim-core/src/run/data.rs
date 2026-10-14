//! The endless run's content (R1, sim 120; rooms R1b, sim 122; the world R2a, sim 124): the run's rules, its hand-made rooms and
//! obstacle pieces, its mob kinds, the shop's wares and the shop's interior. Immutable and hashed, like [`crate::Content`];
//! [`RunContent::placeholder`] is the built-in set. Checked by `sim_content::check_run`, which also proves every room passable in
//! every variant.
//!
//! Units are the versus game's: world units (a fighter is about two tall), frames (60 a second), percent.
//!
//! **Rooms are hand-made; the dice choose the arrangement** (`docs/modes/RUNNER_WORLD.md`, 2.1): a room may be **mirrored**, has
//! **optional parts** that come with a chance (a second awning, a crate stack, a gap or no gap), and **slots** where an obstacle
//! **piece** (a dumpster with a crate, a parked van, a water tower...) may be fitted. Its ground leaves at a height of its own
//! ([`Room::exit_y`]: rising, falling, or a valley that comes back up), so the run's height drifts. Every footing object carries a
//! `look` tag ([`LOOK_NAMES`]) so the art can swap in without touching the sim; looks are labels (not hashed).

use crate::content::{Ledge, Platform};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::moves::{self, Hitbox};
use crate::vec2::Vec2;
use crate::SIM_VERSION;

/// At most this many platforms (solid blocks and pass-through platforms) in one room, its pieces included.
pub const ROOM_PLATFORMS: usize = 14;
/// At most this many ledges in one room.
pub const ROOM_LEDGES: usize = 2;
/// At most this many mob spawn markers in one room (a room's `spawned` bits are a `u8`).
pub const ROOM_SPAWNS: usize = 8;
/// At most this many coins in one room, its pieces' included (a room's `coins` bits are a `u32`).
pub const ROOM_COINS: usize = 32;
/// At most this many optional parts in one room (a room's `parts` bits are a `u8`).
pub const ROOM_PARTS: usize = 4;
/// At most this many obstacle slots in one room.
pub const ROOM_SLOTS: usize = 3;
/// The narrowest a room may be (world units, about one screen). Two of them must cover the view (`sim_content::check_run`).
pub const MIN_ROOM_WIDTH: i32 = 28;
/// The widest a room may be (about two screens).
pub const MAX_ROOM_WIDTH: i32 = 64;
/// The shop interior's platforms at most.
pub const INTERIOR_PLATFORMS: usize = 14;

/// What drives a mob ([`MobKind::behavior`]).
pub const MOB_WALKER: u8 = 0;
pub const MOB_FLYER: u8 = 1;

/// Room types ([`Room::kind`]).
/// A street: plain running with a few mobs (the most common).
pub const ROOM_STREET: u8 = 0;
/// An arena: its exit (a gate on its right edge) closes when you enter it and opens once every mob of the room is beaten.
pub const ROOM_ARENA: u8 = 1;
/// A shop: a storefront whose door (`Room::door`) you enter by pressing up in front of it, into the shop's interior. No mobs.
pub const ROOM_SHOP: u8 = 2;
/// A rest stop: no mobs, and a small heal as you enter.
pub const ROOM_REST: u8 = 3;
/// How many room types there are.
pub const ROOM_KINDS: u8 = 4;

/// Shop wares ([`ShopItem::kind`]): heal `amount` percent, or `amount` extra lives.
pub const ITEM_HEAL: u8 = 0;
pub const ITEM_LIFE: u8 = 1;
/// At most this many wares in the shop.
pub const MAX_SHOP_ITEMS: usize = 4;

/// Obstacle slot kinds ([`PieceSlot::kind`]) and which a piece fits ([`Piece::fits`], a bit per kind).
pub const SLOT_STREET: u8 = 0;
pub const SLOT_ROOF: u8 = 1;
pub const SLOT_AIR: u8 = 2;

/// What a footing object is (`look`): the art draws it by this; the sim never reads it. Index into [`LOOK_NAMES`]. Footing has a
/// lighter top edge in the art; decoration (lamp posts, signs, clotheslines) is drawn by the game alone and never stands here.
pub const LOOK_STREET: u8 = 1;
pub const LOOK_ROOF: u8 = 2;
pub const LOOK_AWNING: u8 = 3;
pub const LOOK_BALCONY: u8 = 4;
pub const LOOK_FIRE_ESCAPE: u8 = 5;
pub const LOOK_SCAFFOLD: u8 = 6;
pub const LOOK_DUMPSTER: u8 = 7;
pub const LOOK_CRATE: u8 = 8;
pub const LOOK_VAN: u8 = 9;
pub const LOOK_AC_UNIT: u8 = 10;
pub const LOOK_WATER_TOWER: u8 = 11;
pub const LOOK_CHIMNEY: u8 = 12;
pub const LOOK_ROOF_SHED: u8 = 13;
pub const LOOK_PARAPET: u8 = 14;
pub const LOOK_BILLBOARD: u8 = 15;
pub const LOOK_TIN_ROOF: u8 = 16;
pub const LOOK_STAIRS: u8 = 17;
pub const LOOK_RAMP: u8 = 18;
pub const LOOK_PLANTER: u8 = 19;
pub const LOOK_BUS_SHELTER: u8 = 20;
pub const LOOK_VENT_PIPE: u8 = 21;
pub const LOOK_SHOP_FLOOR: u8 = 22;
pub const LOOK_SHOP_WALL: u8 = 23;
pub const LOOK_SHOP_CEILING: u8 = 24;
pub const LOOK_COUNTER: u8 = 25;
pub const LOOK_PEDESTAL: u8 = 26;
pub const LOOK_SHELF: u8 = 27;
/// Every look's name, by its number (0 is "none": a footing must have a look).
pub const LOOK_NAMES: [&str; 28] = [
    "none",
    "street",
    "roof",
    "awning",
    "balcony",
    "fire_escape",
    "scaffold",
    "dumpster",
    "crate",
    "van",
    "ac_unit",
    "water_tower",
    "chimney",
    "roof_shed",
    "parapet",
    "billboard",
    "tin_roof",
    "stairs",
    "ramp",
    "planter",
    "bus_shelter",
    "vent_pipe",
    "shop_floor",
    "shop_wall",
    "shop_ceiling",
    "counter",
    "pedestal",
    "shelf",
];

/// Something to stand on in a room, a piece or the shop: its platform, what it is (`look`), and the optional part it belongs to
/// (`part`: 0 always there; `n` there only when part `n` is on; `-n` only when part `n` is off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Footing {
    pub platform: Platform,
    pub look: u8,
    pub part: i8,
}

impl Footing {
    /// There only with optional part `n` (1 to [`ROOM_PARTS`]).
    pub fn with(self, n: i8) -> Footing {
        Footing { part: n, ..self }
    }
    /// There only without optional part `n`.
    pub fn without(self, n: i8) -> Footing {
        Footing { part: -n, ..self }
    }
}

/// Where a mob appears in a room: its kind (index into [`RunContent::mobs`]) and its feet, in the room's own coordinates (x from
/// the room's left edge, y from the room's entry ground), and the optional part it belongs to (as [`Footing::part`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobSpawn {
    pub kind: u8,
    pub x: Fx,
    pub y: Fx,
    pub part: i8,
}

/// A coin to pick up (where its middle is), and its optional part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoinSpot {
    pub x: Fx,
    pub y: Fx,
    pub part: i8,
}

/// A ledge of a room, and its optional part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomLedge {
    pub ledge: Ledge,
    pub part: i8,
}

/// A marked place in a room where an obstacle piece may be fitted: the piece's anchor (the middle of its base) and the slot's
/// kind ([`SLOT_STREET`] or [`SLOT_ROOF`] on the ground, [`SLOT_AIR`] in the air), and the optional part it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PieceSlot {
    pub x: Fx,
    pub y: Fx,
    pub kind: u8,
    pub part: i8,
}

/// An obstacle piece: a small hand-made cluster fitted into a room's slot (Spelunky's obstacle blocks). Its geometry is relative
/// to the slot's anchor (x across its middle, y up from its base).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    /// A label (not hashed).
    pub name: String,
    /// Which slot kinds it fits: bit `1 << SLOT_*`.
    pub fits: u8,
    /// How wide it is (x from `-width / 2` to `width / 2`); a ground slot must have this much flat ground under it.
    pub width: Fx,
    pub platforms: Vec<Footing>,
    pub coins: Vec<CoinSpot>,
}

/// One hand-made room of the run. Geometry is in the room's own coordinates: x from 0 (its left edge) to `width`, y from the
/// ground where the room begins (0). Its ground touches its left edge at 0 and its right edge at `exit_y`; solid ground at the
/// seam joins the neighbouring room's into one block, so a fighter walks, dashes, rolls and wavedashes across the seam as on one
/// floor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// A label for the tools and the editors (not hashed).
    pub name: String,
    /// `ROOM_STREET`, `ROOM_ARENA`, `ROOM_SHOP` or `ROOM_REST`.
    pub kind: u8,
    pub width: Fx,
    /// The ground's height at the right edge (its elevation line: 0 flat or a valley, above 0 rising, below 0 falling).
    pub exit_y: Fx,
    /// A fork: a high road over the street (rooftops: gaps, coins, few mobs) and the low road on it, joining again at its end.
    pub fork: bool,
    /// May be mirrored (a rising room then falls).
    pub mirror: bool,
    /// Each optional part's chance, in percent (part 1 first).
    pub parts: Vec<u8>,
    pub platforms: Vec<Footing>,
    pub ledges: Vec<RoomLedge>,
    pub spawns: Vec<MobSpawn>,
    pub coins: Vec<CoinSpot>,
    pub slots: Vec<PieceSlot>,
    /// A shop's door: where to stand (feet) to go in. Unused in other rooms.
    pub door: Vec2,
}

/// The shop's inside (R2a, `RUNNER_WORLD.md` 6): a small room you walk into from a storefront's door, with a counter, the shopkeeper
/// and the wares on pedestals. Its own little stage, in its own coordinates (x from 0, the floor at 0), placed far from the street
/// ([`super::INTERIOR_X`]) while you are inside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interior {
    pub width: Fx,
    /// The floor, walls, ceiling, counter, pedestals and shelves.
    pub platforms: Vec<Footing>,
    /// Where you come in (feet x, on the floor), and the door you leave by (stand there and press up).
    pub enter: Fx,
    pub door: Fx,
    /// The shopkeeper's feet and body (a circle standing on them).
    pub keeper: Vec2,
    pub keeper_radius: Fx,
    /// Where each ware stands (the middle of its pedestal's top), in the wares' order.
    pub pedestals: Vec<Vec2>,
    /// How long the shopkeeper waves on entry, cheers a purchase, grumbles at a refusal and stays dizzy after a bonk (frames).
    pub wave_frames: u16,
    pub cheer_frames: u16,
    pub grumble_frames: u16,
    pub dizzy_frames: u16,
}

/// A kind of mob: its body, its movement and its one attack. The behaviour itself is a small state machine in Rust for now
/// (`run::mobs`); the numbers are here. (Moving the behaviour to scripts is planned for R2b.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobKind {
    /// [`MOB_WALKER`] (walks on the ground, hops over steps, lunges) or [`MOB_FLYER`] (hovers above the player, dives).
    pub behavior: u8,
    /// Hit points: damage taken (in percent) wears it down and at 0 the mob bursts. 0: no stamina (it must be launched off screen).
    pub stamina: Fx,
    /// Its weight in the knockback formula (100 is a middleweight fighter).
    pub weight: Fx,
    /// Its body: one circle of this radius standing on its feet.
    pub radius: Fx,
    /// Fall acceleration and top fall speed (a flyer falls only while launched).
    pub gravity: Fx,
    pub max_fall: Fx,
    /// Walking (or flying) speed and how quickly it gets there.
    pub speed: Fx,
    pub accel: Fx,
    /// A walker's hop over a step or onto a platform (upward speed).
    pub jump: Fx,
    /// How near the player (sideways) it must be to notice them.
    pub aggro: Fx,
    /// How near it must be to start its attack (sideways; a flyer's height band is `hover_height` above the player).
    pub attack_range: Fx,
    /// Frames of warning before the attack (it flashes), the attack's own frames, and the rest after it.
    pub windup: u8,
    pub attack_frames: u8,
    pub cooldown: u8,
    /// The lunge's (walker) or dive's (flyer) speed.
    pub lunge: Fx,
    /// How high above the player a flyer hovers.
    pub hover_height: Fx,
    /// Its hit, live through the attack: `x` ahead of the feet (the way it faces) and `y` up; the damage, angle and knockback are a
    /// fighter hitbox's (`start` and `end` are not used).
    pub hitbox: Hitbox,
    /// Coins it pays when beaten (before the combo multiplier).
    pub coins: u16,
}

/// One of the shop's wares (R1b: a placeholder list; R3 grows it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopItem {
    /// What the shop calls it (a label, not hashed).
    pub name: String,
    /// `ITEM_HEAL` or `ITEM_LIFE`.
    pub kind: u8,
    pub price: u16,
    /// Percent healed, or lives given.
    pub amount: Fx,
    /// How many one shop sells (0: no limit).
    pub limit: u8,
}

/// The run's rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunRules {
    /// Lives at the start (the fighter's stocks).
    pub lives: u8,
    /// How much of the world the camera shows across (world units). The sim needs it: the blast zone and spawning follow the view.
    pub view_width: Fx,
    /// Where the camera holds the player: this far across the view from its left edge (0 to 1). It follows both ways, inside the
    /// rooms in play.
    pub follow: Fx,
    /// How much of the way to where it wants to be the camera goes each frame (0 to 1), at most `cam_max_speed`.
    pub cam_ease: Fx,
    /// The fastest the camera moves.
    pub cam_max_speed: Fx,
    /// An optional forward creep (world units a frame), for A/B tests; 0 (the default) turns it off. When on, the camera's left
    /// edge never goes back past a mark that creeps on by itself (growing by `creep_growth` every 100 units travelled, up to
    /// `creep_max`), and that edge pushes the fighter along (at up to `push_speed`, keeping it `push_margin` inside). It never
    /// crushes.
    pub creep: Fx,
    pub creep_growth: Fx,
    pub creep_max: Fx,
    pub push_margin: Fx,
    pub push_speed: Fx,
    /// The blast zone: this far beyond the view's right edge, and these heights top and bottom (above the highest ground line of
    /// the rooms in play, below the lowest). The left side is the gate: a wall at the left edge of the oldest room kept.
    pub blast_margin: Fx,
    pub blast_top: Fx,
    pub blast_bottom: Fx,
    /// Mobs appear when their spawn marker comes within this distance of the view's right edge.
    pub spawn_ahead: Fx,
    /// After a lost life the fighter comes back this far across the view (0 to 1), above the first ground there.
    pub respawn_at: Fx,
    /// The combo multiplier: 1 plus one for every `combo_step` hits and kills in a row, at most `combo_max`.
    pub combo_step: u16,
    pub combo_max: u8,
    /// Frames a hit mob counts as the player's (a launched mob leaving the screen is a kill in this time).
    pub credit_frames: u16,
    /// Frames a beaten mob takes to burst (it flies with its launch meanwhile; the game draws the pop).
    pub burst_frames: u8,
    /// The window of rooms in play: up to this many rooms behind the current one are kept (the oldest is dropped as you enter a new
    /// room, and a wall closes its left edge), and this many ahead are laid (at least 1).
    pub rooms_behind: u8,
    pub rooms_ahead: u8,
    /// The player has entered a room once its feet are this far past the room's left edge.
    pub enter_depth: Fx,
    /// The gates' thickness (the left wall and an arena's exit).
    pub gate_thickness: Fx,
    /// A shop comes every `shop_min` to `shop_max` rooms (seeded).
    pub shop_min: u8,
    pub shop_max: u8,
    /// The other rooms' odds, as weights: streets, arenas (never two in a row) and rest stops.
    pub street_weight: u8,
    pub arena_weight: u8,
    pub rest_weight: u8,
    /// Coins for clearing an arena.
    pub arena_bonus: u16,
    /// Percent healed on entering a rest stop.
    pub rest_heal: Fx,
    /// How near the shop's door (sideways) the fighter must stand to go in; also how near a ware to buy it, and the way out.
    pub door_reach: Fx,
    /// R2a. The chance (percent) that a street is a fork, when the content has forks.
    pub fork_chance: u8,
    /// The chance (percent) that an obstacle slot gets a piece.
    pub piece_chance: u8,
    /// How far the run's ground may drift up or down from where it began (a room that would leave it is mirrored instead).
    pub drift: Fx,
    /// A coin picked up pays this (it does not count toward the combo), and is picked up within this distance of the body's middle.
    pub coin_value: u16,
    pub coin_reach: Fx,
    /// Frames a mob takes to come out of its door or stairwell (it can neither hit nor be hit meanwhile); 0 pops it in at once.
    pub emerge_frames: u8,
    /// The passability guarantee (`sim_content::check_run`): every room, in every variant, is crossed with jumps that climb at most
    /// `max_step` and clear gaps of at most `max_gap` (a test checks the worst jumper in the cast, made fighters included, beats both).
    pub max_step: Fx,
    pub max_gap: Fx,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunContent {
    pub rules: RunRules,
    /// The rooms. The first is where every run starts (a street with no mobs); after it the generator picks a type, then one room
    /// of that type (`RunRules`'s odds and shop spacing).
    pub rooms: Vec<Room>,
    /// The obstacle pieces rooms' slots may get.
    pub pieces: Vec<Piece>,
    pub mobs: Vec<MobKind>,
    /// What every shop sells.
    pub items: Vec<ShopItem>,
    /// Every shop's inside.
    pub interior: Interior,
}

fn tenths(n: i32) -> Fx {
    Fx::from_ratio(n, 10)
}

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

/// How deep the run's solid ground reaches below a room's own ground line. When laid, a block reaching this deep goes down to
/// [`super::DEEP`] (well below any blast zone), so nothing goes under it whatever height the room is at.
pub const BLOCK_BOTTOM: i32 = -24;

/// A solid block (tenths) from `bottom` to `top`.
fn slab(left: i32, right: i32, bottom: i32, top: i32, look: u8) -> Footing {
    Footing {
        platform: Platform {
            left: tenths(left),
            right: tenths(right),
            y: tenths(top),
            bottom: tenths(bottom),
            pass_through: false,
            rise: Fx::ZERO,
        },
        look,
        part: 0,
    }
}

/// Solid ground or a solid object (tenths), reaching the bottom.
fn solid(left: i32, right: i32, top: i32, look: u8) -> Footing {
    slab(left, right, BLOCK_BOTTOM * 10, top, look)
}

/// A solid ramp (tenths) from `from` at its left to `to` at its right, reaching the bottom.
fn ramp(left: i32, right: i32, from: i32, to: i32, look: u8) -> Footing {
    let mut f = solid(left, right, from, look);
    f.platform.rise = tenths(to - from);
    f
}

/// A pass-through platform (tenths).
fn plank(left: i32, right: i32, y: i32, look: u8) -> Footing {
    Footing {
        platform: Platform {
            left: tenths(left),
            right: tenths(right),
            y: tenths(y),
            bottom: tenths(y),
            pass_through: true,
            rise: Fx::ZERO,
        },
        look,
        part: 0,
    }
}

/// A sloped pass-through platform (tenths): stairs or a fire escape's flight.
fn flight(left: i32, right: i32, from: i32, to: i32, look: u8) -> Footing {
    let mut f = plank(left, right, from, look);
    f.platform.rise = tenths(to - from);
    f
}

fn ledge(x: i32, y: i32, side: i8) -> RoomLedge {
    RoomLedge {
        ledge: Ledge {
            x: tenths(x),
            y: tenths(y),
            side,
        },
        part: 0,
    }
}

fn spawn(kind: u8, x: i32, y: i32) -> MobSpawn {
    MobSpawn {
        kind,
        x: tenths(x),
        y: tenths(y),
        part: 0,
    }
}

fn coin(x: i32, y: i32) -> CoinSpot {
    CoinSpot {
        x: tenths(x),
        y: tenths(y),
        part: 0,
    }
}

/// `n` coins in a line (tenths) from `(x0, y0)` to `(x1, y1)`.
fn coin_line(x0: i32, y0: i32, x1: i32, y1: i32, n: i32) -> Vec<CoinSpot> {
    (0..n)
        .map(|k| {
            let t = if n > 1 { k * 1000 / (n - 1) } else { 0 };
            coin(x0 + (x1 - x0) * t / 1000, y0 + (y1 - y0) * t / 1000)
        })
        .collect()
}

fn slot(x: i32, y: i32, kind: u8) -> PieceSlot {
    PieceSlot {
        x: tenths(x),
        y: tenths(y),
        kind,
        part: 0,
    }
}

/// A mob's hit, from tenths of a unit and of a percent.
fn mob_hit(x: i32, y: i32, r: i32, damage: i32, angle: i16, bkb: i16, kbg: i16) -> Hitbox {
    Hitbox {
        start: 0,
        end: 0,
        x: tenths(x),
        y: tenths(y),
        radius: tenths(r),
        damage: tenths(damage),
        angle,
        base_knockback: bkb,
        knockback_growth: kbg,
        priority: 0,
        group: 0,
        kind: moves::HIT_NORMAL,
        shield_damage: 100,
        hitlag: 100,
        effect: moves::EFFECT_NORMAL,
        targets: moves::TARGETS_ALL,
        fixed_kb: 0,
        hitstun_add: 0,
        sdi: 100,
        link_ahead: Fx::ZERO,
        aim: moves::AIM_ANY,
        set_weight: false,
        reverse: false,
        push: false,
        shieldstun: 100,
        angle_type: moves::ANGLE_BY_BOX,
        command: false,
        clanks: true,
        rebounds: true,
        trip: 0,
        half_height: Fx::ZERO,
        posvec_x: Fx::ZERO,
        posvec_y: Fx::ZERO,
        posvec_frames: 0,
        half_width: Fx::ZERO,
        rules: moves::RULES_CURRENT,
        slip: moves::SLIP_NONE,
        fixed_stun: 0,
        unblockable: false,
        mode: moves::MODE_ANY,
        part: 0,
    }
}

impl RunRules {
    pub fn standard() -> RunRules {
        RunRules {
            lives: 3,
            view_width: int(30),
            follow: Fx::from_ratio(2, 5),
            cam_ease: Fx::from_ratio(1, 8),
            cam_max_speed: Fx::from_ratio(1, 2),
            creep: Fx::ZERO,
            creep_growth: Fx::ZERO,
            creep_max: Fx::ZERO,
            push_margin: tenths(8),
            push_speed: Fx::from_ratio(7, 20),
            blast_margin: int(6),
            blast_top: int(22),
            blast_bottom: int(-14),
            spawn_ahead: int(2),
            respawn_at: Fx::from_ratio(7, 20),
            combo_step: 5,
            combo_max: 5,
            credit_frames: 180,
            burst_frames: 20,
            rooms_behind: 2,
            rooms_ahead: 1,
            enter_depth: int(2),
            gate_thickness: int(1),
            shop_min: 5,
            shop_max: 7,
            street_weight: 70,
            arena_weight: 22,
            rest_weight: 8,
            arena_bonus: 25,
            rest_heal: int(20),
            door_reach: Fx::from_ratio(3, 2),
            fork_chance: 25,
            piece_chance: 70,
            drift: int(8),
            coin_value: 2,
            coin_reach: tenths(9),
            emerge_frames: 24,
            max_step: tenths(40),
            max_gap: tenths(45),
        }
    }
}

impl MobKind {
    /// A walker: rushes the player along the ground and lunges at them.
    pub fn walker() -> MobKind {
        MobKind {
            behavior: MOB_WALKER,
            stamina: int(12),
            weight: int(80),
            radius: tenths(6),
            gravity: Fx::from_ratio(3, 250),
            max_fall: Fx::from_ratio(11, 50),
            speed: Fx::from_ratio(3, 25),
            accel: Fx::from_ratio(1, 50),
            jump: Fx::from_ratio(1, 4),
            aggro: int(16),
            attack_range: tenths(22),
            windup: 18,
            attack_frames: 12,
            cooldown: 40,
            lunge: tenths(3),
            hover_height: Fx::ZERO,
            hitbox: mob_hit(7, 6, 6, 60, 45, 30, 60),
            coins: 5,
        }
    }

    /// A flyer: hovers above and ahead of the player and dives at them.
    pub fn flyer() -> MobKind {
        MobKind {
            behavior: MOB_FLYER,
            stamina: int(8),
            weight: int(60),
            radius: tenths(5),
            gravity: Fx::from_ratio(3, 250),
            max_fall: Fx::from_ratio(1, 5),
            speed: Fx::from_ratio(1, 10),
            accel: Fx::from_ratio(1, 100),
            jump: Fx::ZERO,
            aggro: int(14),
            attack_range: int(6),
            windup: 24,
            attack_frames: 36,
            cooldown: 50,
            lunge: Fx::from_ratio(7, 20),
            hover_height: int(5),
            hitbox: mob_hit(0, 5, 6, 80, 60, 35, 70),
            coins: 8,
        }
    }
}

impl Room {
    fn new(name: &str, kind: u8, width: i32) -> Room {
        Room {
            name: name.to_string(),
            kind,
            width: int(width),
            exit_y: Fx::ZERO,
            fork: false,
            mirror: true,
            parts: Vec::new(),
            platforms: Vec::new(),
            ledges: Vec::new(),
            spawns: Vec::new(),
            coins: Vec::new(),
            slots: Vec::new(),
            door: Vec2::ZERO,
        }
    }

    /// Its elevation line: 0 flat, 1 rising, 2 falling, 3 a valley (down and back up). Mirrored, rising and falling swap.
    pub fn elevation(&self, mirrored: bool) -> u8 {
        let exit = if mirrored { -self.exit_y } else { self.exit_y };
        if exit > Fx::ZERO {
            1
        } else if exit < Fx::ZERO {
            2
        } else if self.platforms.iter().any(|f| {
            !f.platform.pass_through && f.platform.y.min(f.platform.top_right()) < Fx::ZERO
        }) {
            3
        } else {
            0
        }
    }
}

impl CoinSpot {
    fn with_part(self, n: i8) -> CoinSpot {
        CoinSpot { part: n, ..self }
    }
    fn without_part(self, n: i8) -> CoinSpot {
        CoinSpot { part: -n, ..self }
    }
}

/// The shop's inside (placeholder: see the art list in `RUNNER_PLAN.md`): the door on the left, four pedestals, the counter with
/// the shopkeeper behind it on the right, shelves high on the back wall.
fn interior() -> Interior {
    Interior {
        width: int(24),
        platforms: vec![
            solid(0, 240, 0, LOOK_SHOP_FLOOR),
            // The walls and the ceiling: nothing leaves but by the door.
            solid(-10, 0, 140, LOOK_SHOP_WALL),
            solid(240, 250, 140, LOOK_SHOP_WALL),
            slab(-10, 250, 95, 110, LOOK_SHOP_CEILING),
            // The wares' pedestals (low stands you can hop on), the counter, and shelves high on the back wall.
            plank(74, 86, 9, LOOK_PEDESTAL),
            plank(104, 116, 9, LOOK_PEDESTAL),
            plank(134, 146, 9, LOOK_PEDESTAL),
            plank(164, 176, 9, LOOK_PEDESTAL),
            solid(190, 215, 13, LOOK_COUNTER),
            plank(60, 100, 46, LOOK_SHELF),
            plank(130, 170, 46, LOOK_SHELF),
        ],
        enter: int(3),
        door: int(3),
        keeper: Vec2::new(tenths(228), Fx::ZERO),
        keeper_radius: tenths(7),
        pedestals: vec![
            Vec2::new(int(8), tenths(9)),
            Vec2::new(int(11), tenths(9)),
            Vec2::new(int(14), tenths(9)),
            Vec2::new(int(17), tenths(9)),
        ],
        wave_frames: 90,
        cheer_frames: 70,
        grumble_frames: 60,
        dizzy_frames: 100,
    }
}

/// The obstacle pieces (`RUNNER_WORLD.md` 2.1).
fn pieces() -> Vec<Piece> {
    const STREET: u8 = 1 << SLOT_STREET;
    const ROOF: u8 = 1 << SLOT_ROOF;
    const AIR: u8 = 1 << SLOT_AIR;
    let piece = |name: &str, fits: u8, width: i32, platforms: Vec<Footing>, coins: Vec<CoinSpot>| {
        Piece {
            name: name.to_string(),
            fits,
            width: tenths(width),
            platforms,
            coins,
        }
    };
    vec![
        // A dumpster with a crate on its lid.
        piece(
            "dumpster",
            STREET,
            26,
            vec![
                solid(-12, 12, 12, LOOK_DUMPSTER),
                solid(-10, -1, 21, LOOK_CRATE),
            ],
            vec![coin(-5, 29)],
        ),
        // An AC unit and its vent stack.
        piece(
            "ac_vent",
            ROOF,
            24,
            vec![
                solid(-11, 3, 10, LOOK_AC_UNIT),
                solid(4, 11, 22, LOOK_VENT_PIPE),
            ],
            vec![],
        ),
        // A water tower on legs: its deck and the tank's lid.
        piece(
            "water_tower",
            ROOF,
            34,
            vec![
                plank(-16, 16, 36, LOOK_WATER_TOWER),
                plank(-11, 11, 66, LOOK_WATER_TOWER),
            ],
            vec![coin(0, 74)],
        ),
        // A billboard's catwalk, hanging over the gap or the street.
        piece(
            "billboard",
            AIR,
            40,
            vec![plank(-20, 20, 0, LOOK_BILLBOARD)],
            vec![coin(-10, 8), coin(0, 8), coin(10, 8)],
        ),
        // A parked delivery van.
        piece(
            "van",
            STREET,
            44,
            vec![solid(-22, 22, 19, LOOK_VAN)],
            vec![],
        ),
        // A construction scaffold: two planks.
        piece(
            "scaffold",
            STREET | ROOF,
            36,
            vec![
                plank(-18, 18, 24, LOOK_SCAFFOLD),
                plank(-14, 14, 48, LOOK_SCAFFOLD),
            ],
            vec![coin(0, 56)],
        ),
        // A fire escape's two landings, one above the other.
        piece(
            "fire_escape",
            AIR,
            48,
            vec![
                plank(-24, -2, 0, LOOK_FIRE_ESCAPE),
                plank(4, 24, 26, LOOK_FIRE_ESCAPE),
            ],
            vec![],
        ),
    ]
}

impl RunContent {
    /// The built-in run (R2a): placeholder rooms of city street and rooftops (a safe start, streets with ramps, stairs, tin roofs
    /// and a fork, two arenas, two shops and a rest stop), seven obstacle pieces, a walker (kind 0) and a flyer (kind 1), a shop
    /// selling a heal and an extra life, and its interior.
    pub fn placeholder() -> RunContent {
        let mut rooms = Vec::new();

        // Where every run begins: a quiet street with two awnings.
        let mut r = Room::new("start", ROOM_STREET, 32);
        r.mirror = false;
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(60, 120, 36, LOOK_AWNING),
            plank(200, 260, 36, LOOK_AWNING),
        ];
        rooms.push(r);

        // A street with a walker at its far end: a second awning and a stack of crates may be there, and a piece by the kerb.
        let mut r = Room::new("street", ROOM_STREET, 32);
        r.parts = vec![60, 50];
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(60, 120, 36, LOOK_AWNING),
            plank(200, 260, 36, LOOK_AWNING).with(1),
            solid(250, 262, 12, LOOK_CRATE).with(2),
            solid(252, 260, 22, LOOK_CRATE).with(2),
        ];
        r.spawns = vec![spawn(0, 280, 0)];
        r.coins = coin_line(70, 44, 110, 44, 3);
        r.slots = vec![slot(160, 0, SLOT_STREET)];
        rooms.push(r);

        // A gap between two roofs with a plank over it; a flyer waits above. Coins trace the jump.
        let mut r = Room::new("gap", ROOM_STREET, 30);
        r.platforms = vec![
            solid(0, 100, 0, LOOK_ROOF),
            solid(160, 300, 0, LOOK_ROOF),
            plank(115, 145, 36, LOOK_SCAFFOLD),
        ];
        r.ledges = vec![ledge(100, 0, 1), ledge(160, 0, -1)];
        r.spawns = vec![spawn(1, 130, 70), spawn(0, 240, 0)];
        r.coins = vec![coin(105, 30), coin(130, 46), coin(155, 30)];
        r.slots = vec![slot(230, 0, SLOT_ROOF)];
        rooms.push(r);

        // A taller building to climb over (maybe with a chimney), a walker on its roof and a flyer beyond.
        let mut r = Room::new("step", ROOM_STREET, 32);
        r.parts = vec![50];
        r.platforms = vec![
            solid(0, 80, 0, LOOK_ROOF),
            solid(80, 200, 25, LOOK_ROOF),
            solid(200, 320, 0, LOOK_ROOF),
            solid(150, 166, 50, LOOK_CHIMNEY).with(1),
        ];
        r.ledges = vec![ledge(80, 25, -1), ledge(200, 25, 1)];
        r.spawns = vec![spawn(0, 120, 25), spawn(1, 270, 80)];
        r.coins = vec![coin(158, 60).with_part(1)];
        r.slots = vec![slot(265, 0, SLOT_ROOF)];
        rooms.push(r);

        // Fire escapes stacked over the street (a balcony at the top sometimes): room to fight in the air.
        let mut r = Room::new("escapes", ROOM_STREET, 30);
        r.parts = vec![50];
        r.platforms = vec![
            solid(0, 300, 0, LOOK_STREET),
            plank(40, 100, 36, LOOK_FIRE_ESCAPE),
            plank(120, 180, 66, LOOK_FIRE_ESCAPE),
            plank(200, 260, 36, LOOK_FIRE_ESCAPE),
            plank(130, 170, 94, LOOK_BALCONY).with(1),
        ];
        r.spawns = vec![spawn(0, 80, 0), spawn(1, 150, 100), spawn(0, 240, 0)];
        r.coins = coin_line(140, 102, 160, 102, 3)
            .into_iter()
            .map(|c| c.with_part(1))
            .collect();
        rooms.push(r);

        // A wide gap with a balcony to catch below its near edge, then a long roof with an awning; sometimes the roof runs on
        // without a gap.
        let mut r = Room::new("drop", ROOM_STREET, 34);
        r.parts = vec![40];
        r.platforms = vec![
            solid(0, 90, 0, LOOK_ROOF).without(1),
            plank(95, 135, -20, LOOK_BALCONY).without(1),
            solid(140, 340, 0, LOOK_ROOF).without(1),
            solid(0, 340, 0, LOOK_ROOF).with(1),
            plank(160, 220, 36, LOOK_AWNING),
        ];
        r.ledges = vec![
            RoomLedge {
                part: -1,
                ..ledge(90, 0, 1)
            },
            RoomLedge {
                part: -1,
                ..ledge(140, 0, -1)
            },
        ];
        r.spawns = vec![spawn(1, 120, 60), spawn(0, 200, 0), spawn(0, 280, 0)];
        r.coins = vec![coin(115, 30).without_part(1)];
        r.slots = vec![slot(290, 70, SLOT_AIR)];
        rooms.push(r);

        // A long avenue (about two screens) under awnings and a balcony, maybe a bus shelter, with pieces along it.
        let mut r = Room::new("avenue", ROOM_STREET, 48);
        r.parts = vec![50];
        r.platforms = vec![
            solid(0, 480, 0, LOOK_STREET),
            plank(60, 120, 36, LOOK_AWNING),
            plank(200, 260, 60, LOOK_BALCONY),
            plank(340, 400, 36, LOOK_AWNING),
            plank(280, 320, 26, LOOK_BUS_SHELTER).with(1),
        ];
        r.spawns = vec![spawn(0, 140, 0), spawn(1, 240, 80), spawn(0, 400, 0)];
        r.coins = coin_line(210, 68, 250, 68, 3);
        r.slots = vec![
            slot(160, 0, SLOT_STREET),
            slot(440, 0, SLOT_STREET),
            slot(300, 75, SLOT_AIR),
        ];
        rooms.push(r);

        // A street rising to a bridge up a loading ramp (mirrored, it comes down).
        let mut r = Room::new("ramp", ROOM_STREET, 32);
        r.exit_y = int(4);
        r.platforms = vec![
            solid(0, 100, 0, LOOK_STREET),
            ramp(100, 200, 0, 40, LOOK_RAMP),
            solid(200, 320, 40, LOOK_ROOF),
        ];
        r.spawns = vec![spawn(0, 260, 40)];
        r.coins = coin_line(115, 16, 185, 44, 4);
        r.slots = vec![slot(50, 0, SLOT_STREET)];
        rooms.push(r);

        // Tin roofs sloping down into a valley and back up.
        let mut r = Room::new("tin_roofs", ROOM_STREET, 36);
        r.platforms = vec![
            solid(0, 80, 0, LOOK_ROOF),
            ramp(80, 160, 0, -30, LOOK_TIN_ROOF),
            solid(160, 220, -30, LOOK_ROOF),
            ramp(220, 300, -30, 0, LOOK_TIN_ROOF),
            solid(300, 360, 0, LOOK_ROOF),
        ];
        r.spawns = vec![spawn(0, 190, -30), spawn(1, 260, 60)];
        r.coins = coin_line(170, -20, 210, -20, 3);
        r.slots = vec![slot(190, -30, SLOT_ROOF)];
        rooms.push(r);

        // Stairs down from a roof to the street (mirrored, up).
        let mut r = Room::new("stairs", ROOM_STREET, 32);
        r.exit_y = int(-4);
        r.platforms = vec![
            solid(0, 90, 0, LOOK_ROOF),
            ramp(90, 170, 0, -40, LOOK_STAIRS),
            solid(170, 320, -40, LOOK_STREET),
            plank(220, 280, -4, LOOK_AWNING),
        ];
        r.spawns = vec![spawn(0, 260, -40)];
        r.coins = coin_line(100, 6, 160, -24, 3);
        r.slots = vec![slot(240, -40, SLOT_STREET)];
        rooms.push(r);

        // The fork: a fire escape climbs from the street to a high road of catwalks and scaffolds (gaps, coins, one flyer), over the
        // low road (the street: more mobs, safe footing). A coin trail up the fire escape shows the way; falling from the high road
        // lands on the street; both roads meet again at the room's end.
        let mut r = Room::new("fork", ROOM_STREET, 56);
        r.fork = true;
        r.mirror = false;
        r.parts = vec![50];
        r.platforms = vec![
            solid(0, 560, 0, LOOK_STREET),
            flight(40, 100, 0, 30, LOOK_FIRE_ESCAPE),
            plank(100, 140, 30, LOOK_FIRE_ESCAPE),
            flight(140, 190, 30, 55, LOOK_FIRE_ESCAPE),
            plank(190, 270, 55, LOOK_BILLBOARD),
            plank(300, 370, 60, LOOK_SCAFFOLD),
            plank(400, 470, 55, LOOK_BILLBOARD),
            flight(470, 530, 55, 25, LOOK_FIRE_ESCAPE),
        ];
        r.spawns = vec![
            spawn(0, 120, 0),
            spawn(0, 260, 0),
            spawn(0, 380, 0),
            spawn(1, 340, 100),
            MobSpawn {
                part: 1,
                ..spawn(1, 450, 40)
            },
        ];
        let mut trail = coin_line(55, 12, 95, 34, 3);
        trail.extend(coin_line(150, 42, 180, 62, 2));
        trail.extend([coin(285, 74), coin(320, 68), coin(350, 68), coin(385, 76)]);
        r.coins = trail;
        r.slots = vec![slot(320, 0, SLOT_STREET)];
        rooms.push(r);

        // An arena: a yard under fire escapes, four mobs.
        let mut r = Room::new("yard", ROOM_ARENA, 32);
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(50, 110, 36, LOOK_FIRE_ESCAPE),
            plank(130, 190, 66, LOOK_FIRE_ESCAPE),
            plank(210, 270, 36, LOOK_FIRE_ESCAPE),
        ];
        r.spawns = vec![
            spawn(0, 120, 0),
            spawn(0, 220, 0),
            spawn(1, 170, 90),
            spawn(0, 280, 0),
        ];
        rooms.push(r);

        // An arena around a raised roof, two walkers and two flyers.
        let mut r = Room::new("roofs", ROOM_ARENA, 36);
        r.platforms = vec![
            solid(0, 140, 0, LOOK_ROOF),
            solid(140, 220, 20, LOOK_ROOF_SHED),
            solid(220, 360, 0, LOOK_ROOF),
        ];
        r.ledges = vec![ledge(140, 20, -1), ledge(220, 20, 1)];
        r.spawns = vec![
            spawn(0, 80, 0),
            spawn(1, 180, 90),
            spawn(0, 280, 0),
            spawn(1, 310, 80),
        ];
        rooms.push(r);

        // A shop: a storefront with an awning over its door.
        let mut r = Room::new("shop", ROOM_SHOP, 32);
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(120, 200, 40, LOOK_AWNING),
        ];
        r.door = Vec2::new(int(16), Fx::ZERO);
        r.slots = vec![slot(270, 0, SLOT_STREET)];
        rooms.push(r);

        // A shop on a fork's low road: the street with the storefront, a fire escape up to catwalks over it.
        let mut r = Room::new("fork_shop", ROOM_SHOP, 48);
        r.fork = true;
        r.mirror = false;
        r.platforms = vec![
            solid(0, 480, 0, LOOK_STREET),
            plank(260, 340, 40, LOOK_AWNING),
            flight(40, 100, 0, 30, LOOK_FIRE_ESCAPE),
            plank(100, 140, 30, LOOK_FIRE_ESCAPE),
            flight(140, 190, 30, 55, LOOK_FIRE_ESCAPE),
            plank(190, 250, 55, LOOK_BILLBOARD),
            plank(280, 350, 62, LOOK_SCAFFOLD),
            plank(380, 440, 55, LOOK_BILLBOARD),
        ];
        r.door = Vec2::new(int(30), Fx::ZERO);
        let mut trail = coin_line(55, 12, 95, 34, 3);
        trail.extend([coin(265, 72), coin(365, 74), coin(410, 64)]);
        r.coins = trail;
        rooms.push(r);

        // A rest stop: a rooftop garden with a bench and, sometimes, a raised planter.
        let mut r = Room::new("garden", ROOM_REST, 30);
        r.parts = vec![60];
        r.platforms = vec![
            solid(0, 300, 0, LOOK_ROOF),
            plank(100, 200, 36, LOOK_PLANTER),
            solid(40, 70, 10, LOOK_PLANTER).with(1),
        ];
        rooms.push(r);

        RunContent {
            rules: RunRules::standard(),
            rooms,
            pieces: pieces(),
            mobs: vec![MobKind::walker(), MobKind::flyer()],
            items: vec![
                ShopItem {
                    name: "Heal 30%".to_string(),
                    kind: ITEM_HEAL,
                    price: 30,
                    amount: int(30),
                    limit: 0,
                },
                ShopItem {
                    name: "Extra life".to_string(),
                    kind: ITEM_LIFE,
                    price: 150,
                    amount: int(1),
                    limit: 1,
                },
            ],
            interior: interior(),
        }
    }

    /// Hash of the sim version plus everything here but the names and looks.
    pub fn hash(&self) -> u64 {
        let mut h = StateHasher::new();
        h.write_u16(SIM_VERSION);
        self.rules.hash_into(&mut h);
        h.write_u32(self.rooms.len() as u32);
        for c in &self.rooms {
            c.hash_into(&mut h);
        }
        h.write_u32(self.pieces.len() as u32);
        for p in &self.pieces {
            h.write_u8(p.fits);
            p.width.hash_into(&mut h);
            hash_footings(&p.platforms, &mut h);
            hash_coins(&p.coins, &mut h);
        }
        h.write_u32(self.mobs.len() as u32);
        for m in &self.mobs {
            m.hash_into(&mut h);
        }
        h.write_u32(self.items.len() as u32);
        for i in &self.items {
            h.write_u8(i.kind);
            h.write_u16(i.price);
            i.amount.hash_into(&mut h);
            h.write_u8(i.limit);
        }
        let s = &self.interior;
        s.width.hash_into(&mut h);
        hash_footings(&s.platforms, &mut h);
        s.enter.hash_into(&mut h);
        s.door.hash_into(&mut h);
        s.keeper.hash_into(&mut h);
        s.keeper_radius.hash_into(&mut h);
        h.write_u32(s.pedestals.len() as u32);
        for p in &s.pedestals {
            p.hash_into(&mut h);
        }
        for v in [
            s.wave_frames,
            s.cheer_frames,
            s.grumble_frames,
            s.dizzy_frames,
        ] {
            h.write_u16(v);
        }
        h.finish()
    }
}

/// Hashes a platform's geometry.
pub fn hash_platform(p: &Platform, h: &mut StateHasher) {
    p.left.hash_into(h);
    p.right.hash_into(h);
    p.y.hash_into(h);
    p.bottom.hash_into(h);
    h.write_bool(p.pass_through);
    p.rise.hash_into(h);
}

/// Hashes footings' geometry and parts (their looks are labels).
fn hash_footings(list: &[Footing], h: &mut StateHasher) {
    h.write_u32(list.len() as u32);
    for f in list {
        hash_platform(&f.platform, h);
        h.write_i8(f.part);
    }
}

fn hash_coins(list: &[CoinSpot], h: &mut StateHasher) {
    h.write_u32(list.len() as u32);
    for c in list {
        c.x.hash_into(h);
        c.y.hash_into(h);
        h.write_i8(c.part);
    }
}

impl StateHash for RunRules {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.lives);
        for v in [
            self.view_width,
            self.follow,
            self.cam_ease,
            self.cam_max_speed,
            self.creep,
            self.creep_growth,
            self.creep_max,
            self.push_margin,
            self.push_speed,
            self.blast_margin,
            self.blast_top,
            self.blast_bottom,
            self.spawn_ahead,
            self.respawn_at,
            self.enter_depth,
            self.gate_thickness,
            self.rest_heal,
            self.door_reach,
            self.drift,
            self.coin_reach,
            self.max_step,
            self.max_gap,
        ] {
            v.hash_into(h);
        }
        h.write_u16(self.combo_step);
        h.write_u8(self.combo_max);
        h.write_u16(self.credit_frames);
        h.write_u8(self.burst_frames);
        for v in [
            self.rooms_behind,
            self.rooms_ahead,
            self.shop_min,
            self.shop_max,
            self.street_weight,
            self.arena_weight,
            self.rest_weight,
            self.fork_chance,
            self.piece_chance,
            self.emerge_frames,
        ] {
            h.write_u8(v);
        }
        h.write_u16(self.arena_bonus);
        h.write_u16(self.coin_value);
    }
}

impl StateHash for Room {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.kind);
        self.width.hash_into(h);
        self.exit_y.hash_into(h);
        h.write_bool(self.fork);
        h.write_bool(self.mirror);
        h.write_u32(self.parts.len() as u32);
        for p in &self.parts {
            h.write_u8(*p);
        }
        hash_footings(&self.platforms, h);
        h.write_u32(self.ledges.len() as u32);
        for l in &self.ledges {
            l.ledge.x.hash_into(h);
            l.ledge.y.hash_into(h);
            h.write_i8(l.ledge.side);
            h.write_i8(l.part);
        }
        h.write_u32(self.spawns.len() as u32);
        for s in &self.spawns {
            h.write_u8(s.kind);
            s.x.hash_into(h);
            s.y.hash_into(h);
            h.write_i8(s.part);
        }
        hash_coins(&self.coins, h);
        h.write_u32(self.slots.len() as u32);
        for s in &self.slots {
            s.x.hash_into(h);
            s.y.hash_into(h);
            h.write_u8(s.kind);
            h.write_i8(s.part);
        }
        self.door.hash_into(h);
    }
}

impl StateHash for MobKind {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.behavior);
        for v in [
            self.stamina,
            self.weight,
            self.radius,
            self.gravity,
            self.max_fall,
            self.speed,
            self.accel,
            self.jump,
            self.aggro,
            self.attack_range,
            self.lunge,
            self.hover_height,
        ] {
            v.hash_into(h);
        }
        h.write_u8(self.windup);
        h.write_u8(self.attack_frames);
        h.write_u8(self.cooldown);
        self.hitbox.hash_into(h);
        h.write_u16(self.coins);
    }
}
