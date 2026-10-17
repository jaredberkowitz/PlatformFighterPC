//! The endless run's content (R1, sim 120; rooms R1b, sim 122; the world R2a, sim 124; enemies and the curve R2b, sim 127): the run's
//! rules, its hand-made rooms and obstacle pieces, its grunt kinds, props (hazards and chests), difficulty tiers, the shop's wares and
//! the shop's interior. Immutable and hashed, like [`crate::Content`];
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

/// At most this many platforms (solid blocks and pass-through platforms) in one room, its pieces included (R2d: 16, was 14, for the
/// climbs' fire escapes).
pub const ROOM_PLATFORMS: usize = 16;
/// At most this many ledges in one room (R2d: 4, was 2; the shared ledge table grows only in a run, `state::ledges`).
pub const ROOM_LEDGES: usize = 4;
/// At most this many decks (R2d): a room's `decks` are bits of a byte.
pub const MAX_DECKS: usize = 8;
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

/// A grunt's role ([`MobKind::role`]): what its behaviour does (`run::mobs`). R2b (`RUNNER_WORLD.md` 4): every grunt is fighter-shaped,
/// a base-game body with a small kit borrowed from that body's moveset.
/// Rushes you, jabs and forward-tilts.
pub const ROLE_BRAWLER: u8 = 0;
/// Waits on high ground and leaps at you with an aerial (the anti-air lesson).
pub const ROLE_LEAPER: u8 = 1;
/// Holds a shield toward you and pokes out of it: beaten from above, by a grab, or by crossing it up.
pub const ROLE_SHIELD: u8 = 2;
/// A long, delayed poke along the ground (a forward tilt or a dash attack): jump it and punish.
pub const ROLE_SPEAR: u8 = 3;
/// Throws its shot in an arc from high ground (a real projectile: reflecting it works).
pub const ROLE_LOBBER: u8 = 4;
/// Slow and heavy, armored through its big smash's startup, a big launch.
pub const ROLE_BRUISER: u8 = 5;
pub const ROLES: u8 = 6;
/// The built-in grunt kinds, by their index in [`RunContent::mobs`] (one per role, in the roles' order).
pub const KIND_BRAWLER: u8 = 0;
pub const KIND_LEAPER: u8 = 1;
pub const KIND_SHIELD: u8 = 2;
pub const KIND_SPEAR: u8 = 3;
pub const KIND_LOBBER: u8 = 4;
pub const KIND_BRUISER: u8 = 5;
/// A spawn spot the budget fills ([`MobSpawn::kind`]): any kind the room's tier allows and the spot's placement suits.
pub const SPOT_ANY: u8 = 255;
/// No move in a kit slot ([`MobKind::moves`]).
pub const NO_MOVE: u8 = 255;
/// Moves in a grunt's kit at most.
pub const MOB_MOVES: usize = 3;
/// Placement rules ([`MobKind::place`], bits): it needs high ground (`RunRules::high_ground` above the room's lowest solid ground),
/// flat room (a flat platform at least `RunRules::flat_room` wide), or a platform of its own (never two such on one platform).
pub const PLACE_HIGH: u8 = 1;
pub const PLACE_FLAT: u8 = 2;
pub const PLACE_ALONE: u8 = 4;

/// What a prop is ([`PropKind::role`]): a gas canister that bursts a moment after it is hit, a loose sign that falls when hit (both
/// hurt grunts, and you), or a chest that pays when hit open.
pub const PROP_GAS: u8 = 0;
pub const PROP_SIGN: u8 = 1;
pub const PROP_CHEST: u8 = 2;
pub const PROP_ROLES: u8 = 3;
/// The built-in props, by index in [`RunContent::props`].
pub const PROP_KIND_GAS: u8 = 0;
pub const PROP_KIND_SIGN: u8 = 1;
pub const PROP_KIND_CHEST: u8 = 2;
/// At most this many props in one room.
pub const ROOM_PROPS: usize = 4;

/// Rare events ([`Room::event`]): none, or the runaway van (ride its roof through the street, scattering grunts).
pub const EVENT_NONE: u8 = 0;
pub const EVENT_VAN: u8 = 1;

/// Room types ([`Room::kind`]).
/// A street: plain running with a few mobs (the most common).
pub const ROOM_STREET: u8 = 0;
/// An arena: its exit (a gate on its right edge) closes when you enter it and opens once every mob of the room is beaten.
pub const ROOM_ARENA: u8 = 1;
/// A shop: a storefront whose door (`Room::door`) you enter by pressing up in front of it, into the shop's interior. No mobs.
pub const ROOM_SHOP: u8 = 2;
/// A rest stop: no mobs, and a small heal as you enter.
pub const ROOM_REST: u8 = 3;
/// A rare event's room (R2b, `RUNNER_WORLD.md` 5): at most one a city, weighted with a pity counter, a "?" on the map.
pub const ROOM_EVENT: u8 = 4;
/// A treasure alley (R2b, the owner's answer to R2a): the high road's own room after a branching fork, laid over the low road's room
/// until the player takes one of them. Never drawn by the generator alone.
pub const ROOM_ALLEY: u8 = 5;
/// How many room types there are.
pub const ROOM_KINDS: u8 = 6;

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
/// R2b: a catwalk between rooftops (the treasure alley's high road), and the runaway van's roof (moved by the sim; the game draws the
/// van there).
pub const LOOK_CATWALK: u8 = 28;
pub const LOOK_VAN_ROOF: u8 = 29;
/// R2d: the skyline's kit, the climbs' building faces and the interiors (`RUNNER_PLAN.md` 12, the art list).
pub const LOOK_CRANE: u8 = 30;
pub const LOOK_GIRDER: u8 = 31;
pub const LOOK_WALL: u8 = 32;
pub const LOOK_LEDGE_BLOCK: u8 = 33;
pub const LOOK_OFFICE_FLOOR: u8 = 34;
pub const LOOK_CEILING: u8 = 35;
pub const LOOK_DESK: u8 = 36;
pub const LOOK_CUBICLE: u8 = 37;
pub const LOOK_BAR_COUNTER: u8 = 38;
pub const LOOK_TABLE: u8 = 39;
pub const LOOK_STAIRWELL: u8 = 40;
pub const LOOK_TOWER: u8 = 41;
/// Every look's name, by its number (0 is "none": a footing must have a look).
pub const LOOK_NAMES: [&str; 42] = [
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
    "catwalk",
    "van_roof",
    "crane",
    "girder",
    "wall",
    "ledge_block",
    "office_floor",
    "ceiling",
    "desk",
    "cubicle",
    "bar_counter",
    "table",
    "stairwell",
    "tower",
];

/// Something to stand on in a room, a piece or the shop: its platform, what it is (`look`), and the optional part it belongs to
/// (`part`: 0 always there; `n` there only when part `n` is on; `-n` only when part `n` is off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Footing {
    pub platform: Platform,
    pub look: u8,
    pub part: i8,
    /// R2d. Part of the room's **skilled route** (a double jump, a wall jump, a ledge catch, a recovery across a gap: shorter, and
    /// it pays coins). Never needed: the room's **sure route** (everything else) is proven passable by the worst jumper in the cast
    /// without it (`sim_content::check_run`). Only pass-through footings may be skilled.
    pub skilled: bool,
}

impl Footing {
    /// On the skilled route.
    pub fn skilled(self) -> Footing {
        Footing {
            skilled: true,
            ..self
        }
    }
    /// There only with optional part `n` (1 to [`ROOM_PARTS`]).
    pub fn with(self, n: i8) -> Footing {
        Footing { part: n, ..self }
    }
    /// There only without optional part `n`.
    pub fn without(self, n: i8) -> Footing {
        Footing { part: -n, ..self }
    }
}

/// Where a grunt may appear in a room: its kind (index into [`RunContent::mobs`], or [`SPOT_ANY`] for one the room's budget picks
/// when the room is laid) and its feet, in the room's own coordinates (x from the room's left edge, y from the room's entry ground),
/// and the optional part it belongs to (as [`Footing::part`]).
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

/// A prop in a room (R2b): its kind (index into [`RunContent::props`]), where it stands (the middle of its base; a sign hangs with
/// its top here), and its optional part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropSpot {
    pub kind: u8,
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

/// R2e (the owner's note B: a believable city). A building mass behind the play plane: the face of a building from `left` to
/// `right` (room coordinates), its top at `top`, reaching down out of view. The game draws it as a facade with a roof (a parapet or
/// a tower's crown) and, at street level, its shopfronts and doors. What hangs on a building (a balcony, a fire escape, an awning, a
/// sign, a window sill) must hang on one of these (`run::support_of`, checked by `sim_content::check_run`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    pub left: Fx,
    pub right: Fx,
    pub top: Fx,
    /// `BUILDING_*`: how it is drawn (a label for the game; hashed with the room all the same).
    pub kind: u8,
}

/// Building kinds: a brick apartment block, an office block, a row of shops (lit shopfronts and doors at street level), a
/// warehouse, a glass tower.
pub const BUILDING_BRICK: u8 = 0;
pub const BUILDING_OFFICE: u8 = 1;
pub const BUILDING_SHOPS: u8 = 2;
pub const BUILDING_WAREHOUSE: u8 = 3;
pub const BUILDING_TOWER: u8 = 4;
pub const BUILDING_KINDS: u8 = 5;
/// At most this many building masses a room.
pub const ROOM_BUILDINGS: usize = 6;

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
    /// A shop's door: where to stand (feet) to go in; an event room's start mark (where the van waits). Unused in other rooms.
    pub door: Vec2,
    /// R2b. Its props (hazards and chests).
    pub props: Vec<PropSpot>,
    /// The difficulty tiers it comes in (its pool, `RunRules::tier_rooms`): from `min_tier` to `max_tier`.
    pub min_tier: u8,
    pub max_tier: u8,
    /// A rare event room's event (`EVENT_*`).
    pub event: u8,
    /// A branching fork (the owner's R2a answer): its high road leaves its right edge `lift` above the street, into room `branch_high`
    /// (an alley); the street leaves into room `branch_low`. Both are laid at its end, the alley `lift` higher; the one the player
    /// walks into is kept and the other dropped. 0 for none.
    pub branch_high: u8,
    pub branch_low: u8,
    pub lift: Fx,
    /// Only ever laid as a branch of a branching fork, never drawn by the generator by itself.
    pub paired: bool,
    /// R2d, the decks ([`RunContent::decks`]). The decks it may be entered on (bit `d` for deck `d`), and the deck it leaves on,
    /// counted from the one it enters on: 0 it stays, 1 it climbs a deck, -1 it drops one (mirrored, a climb drops and a drop
    /// climbs). A branching fork's `deck_change` is its low road's; its high road leaves `lift` higher.
    pub decks: u8,
    pub deck_change: i8,
    /// A true pit: a fall into one of its gaps is a lost life. Every other room catches a fall on something below (on the decks
    /// above the street, the deck below), from where its sure route leads on (`sim_content::check_run`).
    pub pit: bool,
    /// An interior (Canabalt's rooms): you crash in through a building's window, run its floor under a ceiling (desks, crates, a
    /// cubicle row, grunts) and burst out of a window on the far side. The game draws its back wall, windows and light.
    pub indoor: bool,
    /// R2e (note B). The building masses behind the play plane (`Building`): the city the room stands in.
    pub buildings: Vec<Building>,
}

/// One deck of the city (R2d, the review's F1): a band of heights the run's ground keeps to while it is on it, about one view tall.
/// The decks are data, lowest first: adding one (a sewer below the street, a higher skyline) is a content change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deck {
    /// A label (not hashed).
    pub name: String,
    /// Its ground line (world height; the run begins on `RunRules::start_deck`'s, at 0), and how far a room's ground may wander from
    /// it while the run stays on this deck.
    pub y: Fx,
    pub wander: Fx,
    /// How often the deck plan heads for it, as a weight (0: only a fork or a vertical room takes you there).
    pub weight: u8,
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

/// A kind of grunt (R2b, `RUNNER_WORLD.md` 4): a base-game body ([`MobKind::body`], an index into the roster: its hurtbox, weight, fall
/// speed, walk and run speeds and jump are that fighter's) with a small kit of that body's own moves (their real hitboxes, frames and
/// knockback), driven by a small state machine for its role (`run::mobs`). The game draws it with the fighter rig in a grunt outfit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobKind {
    /// `ROLE_*`: what its behaviour does.
    pub role: u8,
    /// Its body: an index into the roster (`Content::fighters`); its moves are that fighter's weapon's.
    pub body: u8,
    /// Its kit: `moves::MoveId` indices into its body's weapon ([`NO_MOVE`] for an empty slot). What each slot is for depends on the
    /// role (`run::mobs`).
    pub moves: [u8; MOB_MOVES],
    /// An optional percent at which it is beaten where it stands (times the tier's `stamina` percent; R2c). 0, every built-in kind:
    /// none, it must be launched out of the following blast zones (the owner's rule).
    pub stamina: Fx,
    /// Its weight, in percent of its tier's counted from -100 (`Tier::weight`, `run::grunt_weight`): the knockback formula reads that
    /// instead of its body's (R2c). The bruiser is the heaviest.
    pub weight: u16,
    /// Coins it pays when beaten (before the combo multiplier).
    pub coins: u16,
    /// What it costs from a room's budget, the first tier it comes in, how likely it is picked (a weight), and its placement rules
    /// (`PLACE_*`).
    pub cost: u8,
    pub min_tier: u8,
    pub odds: u8,
    pub place: u8,
    /// Its walking speed and its running speed (when far from you; 0 never runs), in percent of its body's.
    pub walk: u8,
    pub run: u8,
    /// How near (sideways) it notices you, how near it starts its first move (or leaps, or throws), and how near its second.
    pub aggro: Fx,
    pub reach: Fx,
    pub reach2: Fx,
    /// Frames it waits after an attack.
    pub cooldown: u8,
    /// Super armor through the tell and the startup of its first move (hits deal damage but do not flinch it).
    pub armor: bool,
    /// A lobber's shot (its first move's projectile, fired by the move itself): its flight time to the target (frames); the run aims
    /// it to come down on you.
    pub shot_time: u8,
}

/// One difficulty tier (R2b): what comes in a room once the run has reached it (`RunRules::tier_rooms` rooms each).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tier {
    /// The grunt kinds it allows: bit `1 << kind`.
    pub kinds: u8,
    /// A room's budget: this many points for every 16 units of its width (an arena gets `RunRules::arena_budget` percent of it).
    pub budget: u8,
    /// Frames of warning before every attack (the tell: a flash and a "!"), the grunts' stamina and their damage (percent).
    pub tell: u8,
    pub stamina: u8,
    pub damage: u8,
    /// R2c. The grunts' weight (each kind's `MobKind::weight` percent of it): far lighter than any fighter at first, so a solid hit
    /// launches them out of the view.
    pub weight: Fx,
    /// R2c. How often (percent) a grunt techs a landing in hitstun, DIs a launch (survival DI) and mashes out of a grab.
    pub tech: u8,
    pub di: u8,
    pub mash: u8,
    /// R2e (the owner: "start with a small maximum, two or three, and scale it up every difficulty tier"). The most grunts alive at
    /// once in this tier; a spawn spot past it waits until one is beaten (at most the pool, `run::MAX_MOBS`).
    pub cap: u8,
}

/// A prop (R2b, Punch Quest's torches): a hazard or a chest. Hit by the player's attack or shot, a gas canister burns for `fuse`
/// frames, then bursts with `hit` for `frames` frames; a loose sign falls and its `hit` is live while it falls; a chest opens and
/// pays `coins`. Hazards hurt grunts (the player's kill) and the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropKind {
    pub role: u8,
    /// Its box: half its width and its height (from its base, or down from where a sign hangs).
    pub half_width: Fx,
    pub height: Fx,
    pub fuse: u8,
    pub frames: u8,
    /// Its hit, centred `hit.y` above its base (a radius and the damage, angle and knockback of a fighter hitbox).
    pub hit: Hitbox,
    pub gravity: Fx,
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
    /// R2b. The difficulty curve: the tier steps up every `tier_rooms` rooms (to the last of `RunContent::tiers`).
    pub tier_rooms: u8,
    /// Placement: high ground is this far above the room's lowest solid ground; flat room is a flat platform at least this wide.
    pub high_ground: Fx,
    pub flat_room: Fx,
    /// An arena's budget, in percent of a street's of its width.
    pub arena_budget: u8,
    /// Rare events: a "city" is this many rooms (until R6's journey), at most one event in each; none before room `event_from`. A
    /// room's chance is `event_base` per mille, plus `event_step` per mille for every room laid since the last event (the pity).
    pub city_rooms: u8,
    pub event_from: u8,
    pub event_base: u16,
    pub event_step: u16,
    /// The runaway van: its top speed and how fast it gets there (units a frame), the most frames it drives, and its hit on the
    /// grunts it runs into (the player's kills).
    pub van_speed: Fx,
    pub van_accel: Fx,
    pub van_frames: u16,
    pub van_hit: Hitbox,
    /// R2c. The camera's height (the middle of the view): it follows the player's feet plus `cam_lift`, outside a dead zone of
    /// `cam_dead`, easing `cam_y_ease` of the way a frame, and stays between `cam_floor` above the blast zone's bottom and
    /// `cam_ceiling` below its top. `view_height` is how much of the world the view shows top to bottom.
    pub view_height: Fx,
    pub cam_lift: Fx,
    pub cam_dead: Fx,
    pub cam_y_ease: Fx,
    pub cam_floor: Fx,
    pub cam_ceiling: Fx,
    /// R2c. The grunts' blast zones follow the view, this far outside it on every side: a launched grunt past one is knocked out.
    pub ko_margin: Fx,
    /// R2d. The deck the run begins on (an index into `RunContent::decks`; its line is where the first room's ground is).
    pub start_deck: u8,
    /// The deck plan: the run keeps to a deck for `deck_rooms_min` to `deck_rooms_max` rooms (a stretch); then, `deck_fork_chance`
    /// percent of the time, a fork offers a choice of deck (it lasts a new stretch), else the plan picks the next deck itself and a
    /// climb, a drop or a shaft takes the run there.
    pub deck_rooms_min: u8,
    pub deck_rooms_max: u8,
    pub deck_fork_chance: u8,
    /// The skilled routes' jumps (`sim_content::check_run`): a skilled route may climb `skill_step` and cross `skill_gap` (the sure
    /// route only `max_step` and `max_gap`). A test checks that most of the cast reaches them, so they ask for skill, not a fighter.
    pub skill_step: Fx,
    pub skill_gap: Fx,
    /// In a room that changes deck the view's height follows faster: this much of the way a frame (else `cam_y_ease`).
    pub cam_y_ease_tall: Fx,
    /// R2e (the owner's note A). The fewest frames between two grunts coming out of doors in the same room (a stagger: they come
    /// one at a time, not all at once).
    pub emerge_gap: u16,
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
    /// R2b. The difficulty tiers, easiest first (at least one).
    pub tiers: Vec<Tier>,
    /// The props rooms may hold.
    pub props: Vec<PropKind>,
    /// R2d. The city's decks, lowest first (at least one, at most [`MAX_DECKS`]).
    pub decks: Vec<Deck>,
}

fn tenths(n: i32) -> Fx {
    Fx::from_ratio(n, 10)
}

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

/// How deep the run's solid ground reaches below a room's own ground line. When laid, a block reaching this deep goes down to
/// [`super::DEEP`] (well below any blast zone), so nothing goes under it whatever height the room is at. (R2d: -60, was -24, so a
/// room may reach two decks down.)
pub const BLOCK_BOTTOM: i32 = -60;

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
        skilled: false,
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
        skilled: false,
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

/// A spot the room's budget fills (tenths).
fn any(x: i32, y: i32) -> MobSpawn {
    spawn(SPOT_ANY, x, y)
}

fn prop(kind: u8, x: i32, y: i32) -> PropSpot {
    PropSpot {
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

/// A building mass (tenths) from `left` to `right`, its top at `top`.
fn bld(left: i32, right: i32, top: i32, kind: u8) -> Building {
    Building {
        left: tenths(left),
        right: tenths(right),
        top: tenths(top),
        kind,
    }
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
        blockable: false,
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
            drift: int(27),
            coin_value: 1,
            coin_reach: tenths(9),
            emerge_frames: 24,
            max_step: tenths(40),
            max_gap: tenths(45),
            tier_rooms: 6,
            high_ground: int(2),
            flat_room: int(6),
            arena_budget: 150,
            city_rooms: 24,
            event_from: 6,
            event_base: 20,
            event_step: 12,
            van_speed: Fx::from_ratio(2, 5),
            van_accel: Fx::from_ratio(1, 80),
            van_frames: 240,
            van_hit: mob_hit(0, 10, 16, 100, 50, 80, 70),
            view_height: int(18),
            cam_lift: int(2),
            cam_dead: int(2),
            cam_y_ease: Fx::from_ratio(1, 20),
            cam_floor: int(9),
            cam_ceiling: int(8),
            ko_margin: int(4),
            start_deck: 1,
            deck_rooms_min: 3,
            deck_rooms_max: 5,
            deck_fork_chance: 60,
            skill_step: tenths(65),
            skill_gap: tenths(75),
            cam_y_ease_tall: Fx::from_ratio(1, 8),
            emerge_gap: 60,
        }
    }
}

impl MobKind {
    /// The six built-in grunt kinds, in the roles' order ([`KIND_BRAWLER`]...). Bodies (the roster's order): 0 duelist (longsword),
    /// 2 rogue (needles), 4 sprinter (gauntlets), 7 hauler (bracers), 8 bounder (clutch), 9 wanderer (buckler).
    pub fn standard() -> Vec<MobKind> {
        use moves::MoveId as M;
        let base = MobKind {
            role: ROLE_BRAWLER,
            body: 4,
            moves: [M::Jab as u8, M::FTilt as u8, NO_MOVE],
            stamina: Fx::ZERO,
            weight: 100,
            coins: 5,
            cost: 2,
            min_tier: 0,
            odds: 40,
            place: 0,
            walk: 80,
            run: 70,
            aggro: int(16),
            reach: tenths(16),
            reach2: tenths(26),
            cooldown: 36,
            armor: false,
            shot_time: 0,
        };
        vec![
            // A brawler on the gauntlets' body: rushes you, a jab up close, a forward tilt a little further.
            base,
            // A leaper on the needles' body: waits on high ground and leaps at you with its down air (from above) or forward air.
            MobKind {
                role: ROLE_LEAPER,
                body: 2,
                weight: 85,
                moves: [M::DAir as u8, M::FAir as u8, NO_MOVE],
                coins: 7,
                cost: 3,
                odds: 30,
                place: PLACE_HIGH,
                walk: 60,
                run: 0,
                aggro: int(14),
                reach: int(9),
                reach2: tenths(25),
                cooldown: 40,
                ..base
            },
            // A shieldbearer on the buckler's body: guards toward you and pokes out with its forward tilt (a jab up close).
            MobKind {
                role: ROLE_SHIELD,
                body: 9,
                weight: 110,
                moves: [M::FTilt as u8, M::Jab as u8, NO_MOVE],
                coins: 8,
                cost: 3,
                min_tier: 1,
                odds: 25,
                place: PLACE_ALONE,
                walk: 50,
                run: 0,
                aggro: int(14),
                reach: tenths(28),
                reach2: tenths(16),
                cooldown: 30,
                ..base
            },
            // A spearman on the longsword's body: a delayed forward tilt, or a dash attack from further out. Jump it and punish.
            MobKind {
                role: ROLE_SPEAR,
                body: 0,
                moves: [M::FTilt as u8, M::DashAttack as u8, NO_MOVE],
                coins: 7,
                cost: 3,
                min_tier: 1,
                odds: 25,
                place: 0,
                walk: 60,
                run: 0,
                aggro: int(15),
                reach: tenths(30),
                reach2: int(6),
                cooldown: 44,
                ..base
            },
            // A lobber on the clutch's body: throws its shell in an arc from high ground (reflect it back), a jab up close.
            MobKind {
                role: ROLE_LOBBER,
                body: 8,
                weight: 90,
                moves: [M::UpSpecial as u8, M::Jab as u8, NO_MOVE],
                coins: 8,
                cost: 4,
                min_tier: 2,
                odds: 20,
                place: PLACE_HIGH,
                walk: 50,
                run: 0,
                aggro: int(18),
                reach: int(15),
                reach2: tenths(16),
                cooldown: 60,
                shot_time: 45,
                ..base
            },
            // A bruiser on the bracers' body: slow and heavy, armored through its forward smash's startup, a big launch.
            MobKind {
                role: ROLE_BRUISER,
                body: 7,
                weight: 150,
                moves: [M::FSmash as u8, M::Jab as u8, NO_MOVE],
                coins: 14,
                cost: 5,
                min_tier: 3,
                odds: 15,
                place: PLACE_FLAT,
                walk: 55,
                run: 0,
                aggro: int(14),
                reach: tenths(30),
                reach2: tenths(18),
                cooldown: 50,
                armor: true,
                ..base
            },
        ]
    }
}

impl Tier {
    /// The built-in curve: tier 0 has brawlers and leapers, then spearmen and shieldbearers, lobbers, bruisers; budgets grow, tells
    /// shrink from 18 frames, grunts grow tougher and hit harder.
    pub fn standard() -> Vec<Tier> {
        let all = |kinds: &[u8]| kinds.iter().fold(0u8, |m, &k| m | 1 << k);
        vec![
            Tier {
                kinds: all(&[KIND_BRAWLER, KIND_LEAPER]),
                budget: 3,
                tell: 18,
                stamina: 100,
                damage: 100,
                weight: int(-55),
                tech: 0,
                di: 0,
                mash: 0,
                cap: 2,
            },
            Tier {
                kinds: all(&[KIND_BRAWLER, KIND_LEAPER, KIND_SHIELD, KIND_SPEAR]),
                budget: 4,
                tell: 16,
                stamina: 110,
                damage: 105,
                weight: int(-40),
                // (Owner, 15 October 2026: techs, DI and mashing from tier 2 came "a bit too early"; they start at tier 3.)
                tech: 0,
                di: 0,
                mash: 0,
                cap: 3,
            },
            Tier {
                kinds: all(&[
                    KIND_BRAWLER,
                    KIND_LEAPER,
                    KIND_SHIELD,
                    KIND_SPEAR,
                    KIND_LOBBER,
                ]),
                budget: 5,
                tell: 14,
                stamina: 120,
                damage: 110,
                weight: int(-25),
                tech: 20,
                di: 35,
                mash: 45,
                cap: 4,
            },
            Tier {
                kinds: 0b11_1111,
                budget: 6,
                tell: 12,
                stamina: 135,
                damage: 120,
                weight: int(-10),
                tech: 30,
                di: 50,
                mash: 60,
                cap: 5,
            },
            Tier {
                kinds: 0b11_1111,
                budget: 7,
                tell: 10,
                stamina: 150,
                damage: 130,
                weight: int(5),
                tech: 40,
                di: 60,
                mash: 75,
                cap: 6,
            },
        ]
    }
}

impl PropKind {
    /// The built-in props: a gas canister, a loose sign, a chest ([`PROP_KIND_GAS`]...).
    pub fn standard() -> Vec<PropKind> {
        vec![
            PropKind {
                role: PROP_GAS,
                half_width: tenths(4),
                height: tenths(12),
                fuse: 14,
                frames: 4,
                hit: mob_hit(0, 6, 26, 140, 60, 50, 80),
                gravity: Fx::ZERO,
                coins: 0,
            },
            PropKind {
                role: PROP_SIGN,
                half_width: int(1),
                height: tenths(8),
                fuse: 0,
                frames: 0,
                hit: mob_hit(0, -4, 10, 100, 70, 40, 70),
                gravity: Fx::from_ratio(1, 50),
                coins: 0,
            },
            PropKind {
                role: PROP_CHEST,
                half_width: tenths(6),
                height: tenths(8),
                fuse: 0,
                frames: 0,
                hit: mob_hit(0, 4, 6, 0, 0, 0, 0),
                gravity: Fx::ZERO,
                coins: 30,
            },
        ]
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
            props: Vec::new(),
            min_tier: 0,
            max_tier: 255,
            event: EVENT_NONE,
            branch_high: 0,
            branch_low: 0,
            lift: Fx::ZERO,
            paired: false,
            decks: DECKS_LOW,
            deck_change: 0,
            pit: false,
            indoor: false,
            buildings: Vec::new(),
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

/// The street and the rooftops (the decks R2a's rooms were made for).
const DECKS_LOW: u8 = 0b011;
/// The rooftops and the skyline.
const DECKS_HIGH: u8 = 0b110;
/// Every deck.
const DECKS_ALL: u8 = 0b111;

/// The built-in city's three decks (R2d): the street, the rooftops (where the run begins, R2a's level) and the skyline, one view
/// apart.
pub fn standard_decks() -> Vec<Deck> {
    let deck = |name: &str, y: i32, weight: u8| Deck {
        name: name.to_string(),
        y: int(y),
        wander: int(6),
        weight,
    };
    vec![
        deck("street", -18, 3),
        deck("rooftops", 0, 4),
        deck("skyline", 18, 3),
    ]
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
    let piece =
        |name: &str, fits: u8, width: i32, platforms: Vec<Footing>, coins: Vec<CoinSpot>| Piece {
            name: name.to_string(),
            fits,
            width: tenths(width),
            platforms,
            coins,
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

// ---- R2d: the decks' rooms (`RUNNER_REVIEW.md` F1 and F2, `RUNNER_PLAN.md` 12) ------------------------------------------------------

/// A coin on a skilled route.
fn sk(x: i32, y: i32) -> CoinSpot {
    coin(x, y)
}

/// The climbs, the drops and the shafts: rooms that change deck. A mirrored climb is a drop and a mirrored drop a climb, so each
/// is proven both ways (`sim_content::check_run`).
fn vertical_rooms(rooms: &mut Vec<Room>) {
    // The fire-escape climb: a building face one deck tall, its fire escape's landings the sure route; window washers' planks and
    // a crane's girder the skilled one (three big jumps, with coins), out onto the roof.
    let mut r = Room::new("escape_climb", ROOM_STREET, 36);
    r.exit_y = int(18);
    r.deck_change = 1;
    r.platforms = vec![
        solid(0, 200, 0, LOOK_STREET),
        solid(200, 360, 180, LOOK_WALL),
        plank(140, 200, 36, LOOK_FIRE_ESCAPE),
        plank(110, 170, 72, LOOK_FIRE_ESCAPE),
        plank(140, 200, 108, LOOK_FIRE_ESCAPE),
        plank(110, 170, 144, LOOK_FIRE_ESCAPE),
        plank(30, 70, 60, LOOK_SCAFFOLD).skilled(),
        plank(50, 90, 120, LOOK_SCAFFOLD).skilled(),
        plank(70, 200, 180, LOOK_GIRDER).skilled(),
    ];
    r.ledges = vec![ledge(200, 180, -1)];
    r.spawns = vec![
        any(60, 0),
        any(120, 0),
        any(170, 36),
        any(150, 108),
        any(280, 180),
        any(330, 180),
    ];
    r.coins = vec![coin(170, 44), coin(140, 116)];
    r.coins.extend([sk(50, 68), sk(70, 128)]);
    r.coins.extend(coin_line(90, 188, 180, 188, 4));
    r.slots = vec![slot(90, 0, SLOT_STREET), slot(300, 180, SLOT_ROOF)];
    r.buildings = vec![bld(100, 360, 180, BUILDING_BRICK)];
    rooms.push(r);

    // The terraces: a building stepping back as it rises, a ledge at every step (catch one and climb); the skilled route hops
    // balconies over the steps.
    let mut r = Room::new("terrace_climb", ROOM_STREET, 34);
    r.exit_y = int(18);
    r.deck_change = 1;
    r.platforms = vec![
        solid(0, 80, 0, LOOK_STREET),
        solid(80, 140, 36, LOOK_ROOF),
        solid(140, 200, 72, LOOK_ROOF),
        solid(200, 260, 108, LOOK_ROOF),
        solid(260, 300, 144, LOOK_ROOF),
        solid(300, 340, 180, LOOK_ROOF),
        plank(30, 70, 60, LOOK_BALCONY).skilled(),
        plank(120, 170, 120, LOOK_BALCONY).skilled(),
        plank(210, 260, 180, LOOK_BALCONY).skilled(),
    ];
    r.ledges = vec![
        ledge(80, 36, -1),
        ledge(140, 72, -1),
        ledge(200, 108, -1),
        ledge(260, 144, -1),
    ];
    r.spawns = vec![
        any(40, 0),
        any(110, 36),
        any(170, 72),
        any(230, 108),
        any(320, 180),
    ];
    r.coins = vec![sk(50, 68), sk(145, 128)];
    r.coins.extend(coin_line(220, 188, 250, 188, 2));
    r.buildings = vec![bld(0, 340, 200, BUILDING_BRICK)];
    rooms.push(r);

    // The great climb: a scaffold up a tower's face, two decks (the street to the skyline, two views and more); the skilled
    // route is a crane's hooks up the left, five big jumps where the scaffold takes nine.
    let mut r = Room::new("great_climb", ROOM_STREET, 48);
    r.exit_y = int(36);
    r.deck_change = 2;
    r.decks = 0b001;
    r.min_tier = 1;
    r.platforms = vec![
        solid(0, 400, 0, LOOK_STREET),
        solid(400, 480, 360, LOOK_TOWER),
        plank(300, 400, 36, LOOK_SCAFFOLD),
        plank(220, 320, 72, LOOK_SCAFFOLD),
        plank(300, 400, 108, LOOK_SCAFFOLD),
        plank(220, 320, 144, LOOK_SCAFFOLD),
        plank(300, 400, 180, LOOK_SCAFFOLD),
        plank(220, 320, 216, LOOK_SCAFFOLD),
        plank(300, 400, 252, LOOK_SCAFFOLD),
        plank(220, 320, 288, LOOK_SCAFFOLD),
        plank(300, 400, 324, LOOK_SCAFFOLD),
        plank(40, 100, 64, LOOK_CRANE).skilled(),
        plank(90, 150, 128, LOOK_CRANE).skilled(),
        plank(140, 200, 192, LOOK_CRANE).skilled(),
        plank(190, 250, 256, LOOK_CRANE).skilled(),
        plank(240, 300, 320, LOOK_CRANE).skilled(),
    ];
    r.ledges = vec![ledge(400, 360, -1)];
    r.spawns = vec![
        any(100, 0),
        any(250, 0),
        any(350, 108),
        any(260, 216),
        any(350, 324),
        any(440, 360),
    ];
    r.coins = vec![
        sk(70, 72),
        sk(120, 136),
        sk(170, 200),
        sk(220, 264),
        sk(270, 328),
    ];
    r.coins
        .extend([coin(350, 116), coin(270, 224), coin(350, 332)]);
    r.buildings = vec![bld(220, 480, 360, BUILDING_TOWER)];
    rooms.push(r);

    // The balcony drop: off a roof and down a building's face past balconies with grunts on them, to the street a deck below
    // (mirrored, a climb up the balconies).
    let mut r = Room::new("balcony_drop", ROOM_STREET, 36);
    r.exit_y = int(-18);
    r.deck_change = -1;
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 100, 0, LOOK_ROOF),
        solid(100, 360, -180, LOOK_STREET),
        plank(100, 160, -36, LOOK_BALCONY),
        plank(140, 200, -72, LOOK_BALCONY),
        plank(100, 160, -108, LOOK_BALCONY),
        plank(140, 200, -144, LOOK_BALCONY),
        plank(230, 300, -90, LOOK_BALCONY),
    ];
    r.ledges = vec![ledge(100, 0, 1)];
    r.spawns = vec![
        any(50, 0),
        any(130, -36),
        any(170, -72),
        any(260, -90),
        any(220, -180),
        any(320, -180),
    ];
    r.coins = coin_line(120, -20, 330, -170, 6);
    r.slots = vec![slot(320, -180, SLOT_STREET)];
    r.buildings = vec![
        bld(0, 200, 0, BUILDING_BRICK),
        bld(200, 360, -40, BUILDING_SHOPS),
    ];
    rooms.push(r);

    // The awning drop: a roof's edge, then awnings stepping down to the street (one way: never mirrored).
    let mut r = Room::new("awning_drop", ROOM_STREET, 32);
    r.exit_y = int(-18);
    r.deck_change = -1;
    r.decks = DECKS_HIGH;
    r.mirror = false;
    r.platforms = vec![
        solid(0, 60, 0, LOOK_ROOF),
        solid(60, 320, -180, LOOK_STREET),
        plank(70, 120, -50, LOOK_AWNING),
        plank(150, 200, -100, LOOK_AWNING),
        plank(230, 280, -140, LOOK_AWNING),
    ];
    r.spawns = vec![
        any(30, 0),
        any(95, -50),
        any(175, -100),
        any(255, -140),
        any(300, -180),
    ];
    r.coins = vec![coin(95, -40), coin(175, -90), coin(255, -130)];
    r.props = vec![prop(PROP_KIND_GAS, 290, -180)];
    r.buildings = vec![
        bld(0, 120, 0, BUILDING_BRICK),
        bld(120, 200, -60, BUILDING_BRICK),
        bld(200, 320, -100, BUILDING_SHOPS),
    ];
    rooms.push(r);

    // The light well: in through a doorway under the well's wall, up between two walls on alternating sills, out onto the roof;
    // skilled, up the outside and over the wall's top.
    let mut r = Room::new("light_well", ROOM_STREET, 30);
    r.exit_y = int(18);
    r.deck_change = 1;
    r.platforms = vec![
        solid(0, 190, 0, LOOK_STREET),
        slab(100, 120, 40, 180, LOOK_WALL),
        solid(190, 300, 180, LOOK_WALL),
        plank(120, 150, 36, LOOK_LEDGE_BLOCK),
        plank(160, 190, 72, LOOK_LEDGE_BLOCK),
        plank(120, 150, 108, LOOK_LEDGE_BLOCK),
        plank(160, 190, 144, LOOK_LEDGE_BLOCK),
        plank(30, 80, 64, LOOK_SCAFFOLD).skilled(),
        plank(50, 100, 128, LOOK_SCAFFOLD).skilled(),
    ];
    r.ledges = vec![ledge(190, 180, -1)];
    r.spawns = vec![
        any(50, 0),
        any(160, 0),
        any(135, 36),
        any(175, 72),
        any(250, 180),
    ];
    r.coins = vec![sk(55, 72), sk(75, 136), sk(110, 188), sk(155, 196)];
    r.coins.extend([coin(135, 44), coin(175, 152)]);
    r.buildings = vec![bld(100, 300, 180, BUILDING_BRICK)];
    rooms.push(r);

    // The elevator shaft: off a roof into an open shaft, down its cage's girders, out through the door at its foot (mirrored, up).
    let mut r = Room::new("elevator_shaft", ROOM_STREET, 30);
    r.exit_y = int(-18);
    r.deck_change = -1;
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 100, 0, LOOK_ROOF),
        solid(100, 300, -180, LOOK_STREET),
        slab(180, 200, -140, 40, LOOK_WALL),
        plank(100, 140, -40, LOOK_GIRDER),
        plank(140, 180, -80, LOOK_GIRDER),
        plank(100, 140, -120, LOOK_GIRDER),
        plank(140, 180, -150, LOOK_GIRDER),
    ];
    r.spawns = vec![
        any(50, 0),
        any(120, -40),
        any(160, -80),
        any(150, -180),
        any(260, -180),
    ];
    r.coins = coin_line(120, -30, 160, -140, 4);
    r.buildings = vec![bld(0, 200, 40, BUILDING_OFFICE)];
    rooms.push(r);
}

/// The skyline's rooms (and its arena): towers, catwalks, cranes and water towers. Every gap has the rooftops below it (a fall,
/// never a death) and a way back up.
fn skyline_rooms(rooms: &mut Vec<Room>) {
    // Catwalks between towers: the sure route crosses on the catwalk in each gap; the skilled one leaps each gap whole, under
    // coin girders. A fall lands on the roofs below, with sills back up.
    let mut r = Room::new("catwalks", ROOM_STREET, 40);
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 90, 0, LOOK_TOWER),
        solid(160, 220, 0, LOOK_TOWER),
        solid(290, 400, 0, LOOK_TOWER),
        solid(90, 160, -140, LOOK_ROOF),
        solid(220, 290, -140, LOOK_ROOF),
        plank(110, 140, 0, LOOK_CATWALK),
        plank(240, 270, 0, LOOK_CATWALK),
        plank(90, 120, -105, LOOK_FIRE_ESCAPE),
        plank(130, 160, -70, LOOK_FIRE_ESCAPE),
        plank(90, 120, -35, LOOK_FIRE_ESCAPE),
        plank(220, 250, -105, LOOK_FIRE_ESCAPE),
        plank(260, 290, -70, LOOK_FIRE_ESCAPE),
        plank(220, 250, -35, LOOK_FIRE_ESCAPE),
        plank(100, 150, 55, LOOK_GIRDER).skilled(),
        plank(230, 280, 55, LOOK_GIRDER).skilled(),
    ];
    r.ledges = vec![
        ledge(90, 0, 1),
        ledge(160, 0, -1),
        ledge(220, 0, 1),
        ledge(290, 0, -1),
    ];
    r.spawns = vec![
        any(50, 0),
        any(190, 0),
        any(330, 0),
        any(370, 0),
        any(125, -140),
        any(255, -140),
    ];
    r.coins = coin_line(105, 63, 145, 63, 3);
    r.coins.extend(coin_line(235, 63, 275, 63, 3));
    r.coins.extend([coin(125, 8), coin(255, 8)]);
    r.buildings = vec![
        bld(90, 160, -20, BUILDING_BRICK),
        bld(220, 290, -20, BUILDING_BRICK),
    ];
    rooms.push(r);

    // Cranes: two jibs between tower tops with a hook hanging between them (the sure route) and a skilled leap from jib to jib under
    // a coin girder; the rooftops below, a crane's mast to climb back.
    let mut r = Room::new("cranes", ROOM_STREET, 44);
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 70, 0, LOOK_TOWER),
        solid(370, 440, 0, LOOK_TOWER),
        solid(70, 370, -130, LOOK_ROOF),
        plank(60, 180, 15, LOOK_CRANE),
        plank(195, 225, 0, LOOK_CRANE),
        plank(240, 380, 30, LOOK_CRANE),
        plank(70, 110, -95, LOOK_GIRDER),
        plank(120, 160, -60, LOOK_GIRDER),
        plank(70, 110, -25, LOOK_GIRDER),
        plank(180, 240, 75, LOOK_GIRDER).skilled(),
    ];
    r.spawns = vec![
        any(30, 0),
        any(120, 15),
        any(300, 30),
        any(340, 30),
        any(400, 0),
        any(250, -130),
    ];
    r.coins = coin_line(80, 23, 170, 23, 4);
    r.coins.extend(coin_line(190, 83, 230, 83, 3));
    r.coins.extend(coin_line(260, 38, 360, 38, 4));
    r.buildings = vec![bld(70, 370, -40, BUILDING_OFFICE)];
    rooms.push(r);

    // Water towers on a tower's roof: their decks and lids to fight on and over, coins on the lids.
    let mut r = Room::new("water_towers", ROOM_STREET, 36);
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 360, 0, LOOK_TOWER),
        plank(60, 120, 36, LOOK_WATER_TOWER),
        plank(70, 110, 66, LOOK_WATER_TOWER),
        plank(200, 260, 36, LOOK_WATER_TOWER),
        plank(210, 250, 66, LOOK_WATER_TOWER),
        plank(130, 190, 90, LOOK_BILLBOARD),
    ];
    r.spawns = vec![
        any(40, 0),
        any(90, 36),
        any(150, 0),
        any(230, 36),
        any(300, 0),
    ];
    r.coins = vec![coin(90, 74), coin(230, 74)];
    r.coins.extend(coin_line(140, 98, 180, 98, 3));
    r.slots = vec![slot(310, 0, SLOT_ROOF)];
    rooms.push(r);

    // An arena on a construction deck high up: scaffolds and a girder, a crane's jib over the middle.
    let mut r = Room::new("rig", ROOM_ARENA, 36);
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 360, 0, LOOK_TOWER),
        plank(40, 120, 40, LOOK_SCAFFOLD),
        plank(240, 320, 40, LOOK_SCAFFOLD),
        plank(130, 230, 75, LOOK_GIRDER),
        plank(150, 210, 110, LOOK_CRANE),
    ];
    r.spawns = vec![
        any(100, 0),
        any(200, 0),
        any(300, 0),
        any(80, 40),
        any(280, 40),
        any(180, 75),
    ];
    r.props = vec![prop(PROP_KIND_SIGN, 180, 103)];
    rooms.push(r);
}

/// The interiors (Canabalt's rooms, the owner's answer 3): a building's floor you crash into through its window and burst out of on
/// the far side, under a ceiling that cuts the jumps short. The game draws the back wall, the windows and the light.
fn interior_rooms(rooms: &mut Vec<Room>) {
    // An office floor: desks, a cubicle row and a crate stack between the windows.
    let mut r = Room::new("office", ROOM_STREET, 40);
    r.indoor = true;
    r.decks = DECKS_ALL;
    r.platforms = vec![
        solid(0, 400, 0, LOOK_OFFICE_FLOOR),
        slab(40, 360, 50, 200, LOOK_CEILING),
        solid(80, 110, 12, LOOK_DESK),
        solid(150, 210, 16, LOOK_CUBICLE),
        solid(250, 280, 12, LOOK_DESK),
        solid(310, 322, 12, LOOK_CRATE),
        solid(312, 320, 22, LOOK_CRATE),
    ];
    r.spawns = vec![
        any(130, 0),
        any(180, 16),
        any(230, 0),
        any(295, 0),
        any(340, 0),
    ];
    r.coins = coin_line(160, 24, 200, 24, 3);
    r.coins.extend([coin(95, 20), coin(265, 20)]);
    r.props = vec![prop(PROP_KIND_GAS, 60, 0)];
    r.buildings = vec![bld(40, 360, 200, BUILDING_OFFICE)];
    rooms.push(r);

    // A stairwell: in at the ground floor's window, up four flights, out of the top floor's window onto the roof a deck up
    // (mirrored, down).
    let mut r = Room::new("stairwell", ROOM_STREET, 34);
    r.indoor = true;
    r.exit_y = int(18);
    r.deck_change = 1;
    r.platforms = vec![
        solid(0, 290, 0, LOOK_OFFICE_FLOOR),
        solid(290, 340, 180, LOOK_ROOF),
        slab(40, 50, 45, 270, LOOK_WALL),
        slab(50, 300, 225, 270, LOOK_CEILING),
        flight(70, 160, 0, 45, LOOK_STAIRWELL),
        plank(160, 180, 45, LOOK_STAIRWELL),
        flight(70, 160, 90, 45, LOOK_STAIRWELL),
        plank(50, 70, 90, LOOK_STAIRWELL),
        flight(70, 160, 90, 135, LOOK_STAIRWELL),
        plank(160, 180, 135, LOOK_STAIRWELL),
        flight(70, 160, 180, 135, LOOK_STAIRWELL),
        plank(50, 290, 180, LOOK_OFFICE_FLOOR),
    ];
    r.spawns = vec![
        any(230, 0),
        any(170, 45),
        any(60, 90),
        any(240, 180),
        any(320, 180),
    ];
    r.coins = coin_line(80, 8, 150, 42, 3);
    r.coins.extend(coin_line(150, 97, 80, 131, 3));
    r.buildings = vec![bld(40, 300, 270, BUILDING_OFFICE)];
    rooms.push(r);

    // A rooftop bar under its canopy: the counter and tables to vault, coins on the canopy for whoever gets up there.
    let mut r = Room::new("rooftop_bar", ROOM_STREET, 32);
    r.indoor = true;
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 320, 0, LOOK_ROOF),
        slab(80, 260, 45, 60, LOOK_CEILING),
        solid(120, 170, 12, LOOK_BAR_COUNTER),
        solid(200, 215, 9, LOOK_TABLE),
        solid(232, 247, 9, LOOK_TABLE),
    ];
    r.spawns = vec![
        any(40, 0),
        any(100, 0),
        any(145, 12),
        any(224, 0),
        any(290, 0),
    ];
    r.coins = coin_line(100, 68, 240, 68, 5);
    r.buildings = vec![bld(80, 260, 60, BUILDING_SHOPS)];
    rooms.push(r);
}

/// The deck forks (the review's F1): a choice of deck that lasts several rooms. The up fork climbs from its street to a catwalk a
/// deck higher (the high road) over the street going on (the low road); the down fork goes on along the rooftops on a catwalk (the
/// high road) over the street a deck below (the low road). Both lead into the same pair of roads: the sky bridge over the street
/// under it. Whichever road you walk out on is the deck you keep to.
fn deck_forks(rooms: &mut Vec<Room>) {
    let bridge = rooms.len() as u8 + 2;
    let mut r = Room::new("deck_fork_up", ROOM_STREET, 48);
    r.fork = true;
    r.mirror = false;
    r.platforms = vec![
        solid(0, 480, 0, LOOK_STREET),
        plank(60, 120, 36, LOOK_FIRE_ESCAPE),
        plank(100, 160, 72, LOOK_FIRE_ESCAPE),
        plank(60, 120, 108, LOOK_FIRE_ESCAPE),
        plank(100, 160, 144, LOOK_FIRE_ESCAPE),
        plank(140, 480, 180, LOOK_CATWALK),
        plank(240, 300, 36, LOOK_AWNING),
        plank(360, 420, 36, LOOK_AWNING),
    ];
    r.spawns = vec![
        any(200, 0),
        any(300, 0),
        any(420, 0),
        any(260, 180),
        any(380, 180),
    ];
    r.coins = coin_line(80, 44, 130, 152, 4);
    r.coins.extend(coin_line(200, 188, 460, 188, 5));
    r.branch_high = bridge;
    r.branch_low = bridge + 1;
    r.lift = int(18);
    r.buildings = vec![
        bld(0, 200, 200, BUILDING_BRICK),
        bld(200, 480, 200, BUILDING_SHOPS),
    ];
    rooms.push(r);

    let mut r = Room::new("deck_fork_down", ROOM_STREET, 48);
    r.fork = true;
    r.mirror = false;
    r.exit_y = int(-18);
    r.deck_change = -1;
    r.decks = DECKS_HIGH;
    r.platforms = vec![
        solid(0, 160, 0, LOOK_ROOF),
        solid(160, 480, -180, LOOK_STREET),
        plank(160, 480, 0, LOOK_CATWALK),
        plank(200, 260, -60, LOOK_AWNING),
        plank(330, 390, -60, LOOK_AWNING),
    ];
    r.spawns = vec![
        any(80, 0),
        any(300, 0),
        any(420, 0),
        any(250, -180),
        any(400, -180),
    ];
    r.coins = coin_line(180, 8, 460, 8, 6);
    r.coins.extend(coin_line(210, -52, 380, -52, 3));
    r.branch_high = bridge;
    r.branch_low = bridge + 1;
    r.lift = int(18);
    r.buildings = vec![bld(160, 480, -20, BUILDING_SHOPS)];
    rooms.push(r);

    // The sky bridge: the high road's room, catwalks a deck over the street (under it, the low road's street: the only solid block),
    // a fire escape back up from the street at its end (the next room stands a deck higher). Its catwalk leaves its right edge.
    let mut r = Room::new("sky_bridge", ROOM_ALLEY, 40);
    r.paired = true;
    r.mirror = false;
    r.decks = DECKS_ALL;
    r.platforms = vec![
        solid(0, 400, -180, LOOK_STREET),
        plank(0, 120, 0, LOOK_CATWALK),
        plank(160, 260, 0, LOOK_CATWALK),
        plank(300, 400, 0, LOOK_CATWALK),
        plank(340, 400, -144, LOOK_FIRE_ESCAPE),
        plank(300, 360, -108, LOOK_FIRE_ESCAPE),
        plank(340, 400, -72, LOOK_FIRE_ESCAPE),
        plank(300, 360, -36, LOOK_FIRE_ESCAPE),
    ];
    r.coins = coin_line(20, 8, 100, 8, 4);
    r.coins.extend(coin_line(170, 8, 250, 8, 4));
    r.coins.extend([coin(140, 14), coin(280, 14)]);
    r.buildings = vec![bld(0, 400, -20, BUILDING_SHOPS)];
    rooms.push(r);

    // The low road's room: the street under the sky bridge (nothing of it reaches up to the catwalks).
    let mut r = Room::new("under_bridge", ROOM_STREET, 40);
    r.paired = true;
    r.mirror = false;
    r.decks = DECKS_ALL;
    r.platforms = vec![
        solid(0, 400, 0, LOOK_STREET),
        plank(60, 130, 36, LOOK_AWNING),
        plank(250, 320, 36, LOOK_AWNING),
        solid(180, 194, 12, LOOK_CRATE),
    ];
    r.spawns = vec![any(100, 0), any(220, 0), any(350, 0), any(290, 36)];
    r.coins = coin_line(260, 44, 310, 44, 3);
    r.buildings = vec![bld(0, 400, 160, BUILDING_SHOPS)];
    rooms.push(r);
}

impl RunContent {
    /// The built-in run (R2b): placeholder rooms of city street and rooftops (a safe start, streets with ramps, stairs, tin roofs,
    /// forks and a branching fork with its treasure alley, three arenas, two shops, a rest stop and the runaway van's street), seven
    /// obstacle pieces, the six grunt kinds, five difficulty tiers, three props, a shop selling a heal and an extra life, and its
    /// interior. Grunts come from spots the room's budget fills (`SPOT_ANY`).
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
        r.buildings = vec![
            bld(0, 160, 90, BUILDING_SHOPS),
            bld(160, 320, 120, BUILDING_BRICK),
        ];
        rooms.push(r);

        // A street: a second awning and a stack of crates may be there, a piece by the kerb, and maybe a gas canister.
        let mut r = Room::new("street", ROOM_STREET, 32);
        r.parts = vec![60, 50, 40];
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(60, 120, 36, LOOK_AWNING),
            plank(200, 260, 36, LOOK_AWNING).with(1),
            solid(250, 262, 12, LOOK_CRATE).with(2),
            solid(252, 260, 22, LOOK_CRATE).with(2),
        ];
        r.spawns = vec![
            any(280, 0),
            any(90, 36),
            MobSpawn {
                part: 1,
                ..any(230, 36)
            },
            any(40, 0),
            any(215, 0),
        ];
        r.coins = coin_line(70, 44, 110, 44, 3);
        r.slots = vec![slot(160, 0, SLOT_STREET)];
        r.props = vec![PropSpot {
            part: 3,
            ..prop(PROP_KIND_GAS, 120, 0)
        }];
        r.buildings = vec![
            bld(0, 160, 90, BUILDING_SHOPS),
            bld(160, 320, 120, BUILDING_BRICK),
        ];
        rooms.push(r);

        // A gap between two roofs with a plank over it, a loose sign over the near roof. Coins trace the jump.
        // (R2e, the owner's answer: "a floor below". A lower roof in the alley catches a fall, with a fire escape back up.)
        let mut r = Room::new("gap", ROOM_STREET, 30);
        r.platforms = vec![
            solid(0, 100, 0, LOOK_ROOF),
            solid(160, 300, 0, LOOK_ROOF),
            plank(115, 145, 36, LOOK_SCAFFOLD),
            solid(100, 160, -140, LOOK_ROOF),
            plank(100, 130, -105, LOOK_FIRE_ESCAPE),
            plank(130, 160, -70, LOOK_FIRE_ESCAPE),
            plank(100, 130, -35, LOOK_FIRE_ESCAPE),
        ];
        r.ledges = vec![ledge(100, 0, 1), ledge(160, 0, -1)];
        r.spawns = vec![
            any(60, 0),
            any(130, 36),
            any(190, 0),
            any(270, 0),
            any(290, 0),
        ];
        r.coins = vec![coin(105, 30), coin(130, 46), coin(155, 30)];
        r.slots = vec![slot(230, 0, SLOT_ROOF)];
        r.props = vec![prop(PROP_KIND_SIGN, 50, 44)];
        r.buildings = vec![
            bld(0, 100, 70, BUILDING_BRICK),
            bld(100, 160, -20, BUILDING_BRICK),
            bld(160, 300, 60, BUILDING_OFFICE),
        ];
        rooms.push(r);

        // A taller building to climb over (maybe with a chimney).
        let mut r = Room::new("step", ROOM_STREET, 32);
        r.decks = DECKS_ALL;
        r.parts = vec![50];
        r.platforms = vec![
            solid(0, 80, 0, LOOK_ROOF),
            solid(80, 200, 25, LOOK_ROOF),
            solid(200, 320, 0, LOOK_ROOF),
            solid(150, 166, 50, LOOK_CHIMNEY).with(1),
        ];
        r.ledges = vec![ledge(80, 25, -1), ledge(200, 25, 1)];
        r.spawns = vec![
            any(120, 25),
            any(180, 25),
            any(40, 0),
            any(220, 0),
            any(300, 0),
        ];
        r.coins = vec![coin(158, 60).with_part(1)];
        r.slots = vec![slot(265, 0, SLOT_ROOF)];
        r.buildings = vec![bld(0, 320, 140, BUILDING_BRICK)];
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
        r.spawns = vec![
            any(80, 0),
            any(240, 0),
            any(150, 0),
            any(70, 36),
            any(230, 36),
            any(150, 66),
        ];
        r.coins = coin_line(140, 102, 160, 102, 3)
            .into_iter()
            .map(|c| c.with_part(1))
            .collect();
        r.props = vec![prop(PROP_KIND_GAS, 20, 0)];
        r.buildings = vec![bld(0, 300, 120, BUILDING_BRICK)];
        rooms.push(r);

        // A wide gap with a balcony to catch below its near edge, then a long roof with an awning; sometimes the roof runs on
        // without a gap. From tier 1.
        // (R2e, the owner's answer: "a floor below". A lower roof in the gap catches a fall, with a fire escape back up.)
        let mut r = Room::new("drop", ROOM_STREET, 34);
        r.min_tier = 1;
        r.parts = vec![40];
        r.platforms = vec![
            solid(0, 90, 0, LOOK_ROOF).without(1),
            plank(95, 135, -20, LOOK_BALCONY).without(1),
            solid(140, 340, 0, LOOK_ROOF).without(1),
            solid(90, 140, -140, LOOK_ROOF).without(1),
            plank(90, 115, -105, LOOK_FIRE_ESCAPE).without(1),
            plank(115, 140, -70, LOOK_FIRE_ESCAPE).without(1),
            plank(90, 115, -40, LOOK_FIRE_ESCAPE).without(1),
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
        r.spawns = vec![
            any(60, 0),
            any(200, 0),
            any(280, 0),
            any(320, 0),
            any(190, 36),
        ];
        r.coins = vec![coin(115, 30).without_part(1)];
        r.slots = vec![slot(290, 70, SLOT_AIR)];
        r.buildings = vec![
            bld(0, 90, 60, BUILDING_BRICK),
            bld(90, 140, -10, BUILDING_BRICK),
            bld(140, 340, 120, BUILDING_BRICK),
        ];
        rooms.push(r);

        // A long avenue (about two screens) under awnings and a balcony, maybe a bus shelter, with pieces along it and a loose sign
        // under an awning. From tier 1.
        let mut r = Room::new("avenue", ROOM_STREET, 48);
        r.min_tier = 1;
        r.parts = vec![50];
        r.platforms = vec![
            solid(0, 480, 0, LOOK_STREET),
            plank(60, 120, 36, LOOK_AWNING),
            plank(200, 260, 60, LOOK_BALCONY),
            plank(340, 400, 36, LOOK_AWNING),
            plank(280, 320, 26, LOOK_BUS_SHELTER).with(1),
        ];
        r.spawns = vec![
            any(40, 0),
            any(110, 0),
            any(240, 0),
            any(330, 0),
            any(400, 0),
            any(230, 60),
            any(90, 36),
            any(370, 36),
        ];
        r.coins = coin_line(210, 68, 250, 68, 3);
        r.slots = vec![
            slot(160, 0, SLOT_STREET),
            slot(440, 0, SLOT_STREET),
            slot(300, 75, SLOT_AIR),
        ];
        r.props = vec![prop(PROP_KIND_SIGN, 370, 30), prop(PROP_KIND_GAS, 22, 0)];
        r.buildings = vec![
            bld(0, 160, 100, BUILDING_SHOPS),
            bld(160, 340, 130, BUILDING_BRICK),
            bld(340, 480, 90, BUILDING_SHOPS),
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
        r.spawns = vec![any(260, 40), any(300, 40), any(90, 0), any(230, 40)];
        r.coins = coin_line(115, 16, 185, 44, 4);
        r.slots = vec![slot(50, 0, SLOT_STREET)];
        r.props = vec![prop(PROP_KIND_GAS, 215, 40)];
        r.buildings = vec![
            bld(0, 100, 80, BUILDING_SHOPS),
            bld(200, 320, 120, BUILDING_BRICK),
        ];
        rooms.push(r);

        // Tin roofs sloping down into a valley and back up.
        let mut r = Room::new("tin_roofs", ROOM_STREET, 36);
        r.decks = DECKS_ALL;
        r.platforms = vec![
            solid(0, 80, 0, LOOK_ROOF),
            ramp(80, 160, 0, -30, LOOK_TIN_ROOF),
            solid(160, 220, -30, LOOK_ROOF),
            ramp(220, 300, -30, 0, LOOK_TIN_ROOF),
            solid(300, 360, 0, LOOK_ROOF),
        ];
        r.spawns = vec![any(40, 0), any(20, 0), any(330, 0), any(310, 0)];
        r.coins = coin_line(170, -20, 210, -20, 3);
        r.slots = vec![slot(190, -30, SLOT_ROOF)];
        r.buildings = vec![bld(0, 360, 110, BUILDING_WAREHOUSE)];
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
        r.spawns = vec![
            any(290, -40),
            any(190, -40),
            any(40, 0),
            any(250, -4),
            any(70, 0),
        ];
        r.coins = coin_line(100, 6, 160, -24, 3);
        r.slots = vec![slot(240, -40, SLOT_STREET)];
        r.buildings = vec![
            bld(0, 90, 50, BUILDING_BRICK),
            bld(170, 320, 80, BUILDING_SHOPS),
        ];
        rooms.push(r);

        // The fork: a fire escape climbs from the street to a high road of catwalks and scaffolds (gaps, coins), over the low road
        // (the street: more grunts, safe footing). A coin trail up the fire escape shows the way; falling from the high road lands on
        // the street; both roads meet again at the room's end.
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
            any(120, 0),
            any(260, 0),
            any(380, 0),
            any(500, 0),
            any(230, 55),
            any(330, 60),
            MobSpawn {
                part: 1,
                ..any(430, 55)
            },
        ];
        let mut trail = coin_line(55, 12, 95, 34, 3);
        trail.extend(coin_line(150, 42, 180, 62, 2));
        trail.extend([coin(285, 74), coin(320, 68), coin(350, 68), coin(385, 76)]);
        r.coins = trail;
        r.slots = vec![slot(320, 0, SLOT_STREET)];
        r.buildings = vec![
            bld(0, 200, 90, BUILDING_BRICK),
            bld(200, 400, 140, BUILDING_OFFICE),
            bld(400, 560, 90, BUILDING_BRICK),
        ];
        rooms.push(r);

        // An arena: a yard under fire escapes, a loose sign under the top one.
        let mut r = Room::new("yard", ROOM_ARENA, 32);
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(50, 110, 36, LOOK_FIRE_ESCAPE),
            plank(130, 190, 66, LOOK_FIRE_ESCAPE),
            plank(210, 270, 36, LOOK_FIRE_ESCAPE),
        ];
        r.spawns = vec![
            any(120, 0),
            any(220, 0),
            any(280, 0),
            any(80, 36),
            any(160, 66),
            any(240, 36),
        ];
        r.props = vec![prop(PROP_KIND_SIGN, 160, 58)];
        r.buildings = vec![bld(0, 320, 110, BUILDING_BRICK)];
        rooms.push(r);

        // An arena around a raised roof.
        let mut r = Room::new("roofs", ROOM_ARENA, 36);
        r.decks = DECKS_ALL;
        r.platforms = vec![
            solid(0, 140, 0, LOOK_ROOF),
            solid(140, 220, 20, LOOK_ROOF_SHED),
            solid(220, 360, 0, LOOK_ROOF),
        ];
        r.ledges = vec![ledge(140, 20, -1), ledge(220, 20, 1)];
        r.spawns = vec![
            any(80, 0),
            any(180, 20),
            any(280, 0),
            any(320, 0),
            any(30, 0),
        ];
        r.buildings = vec![bld(0, 360, 90, BUILDING_OFFICE)];
        rooms.push(r);

        // A shop: a storefront with an awning over its door.
        let mut r = Room::new("shop", ROOM_SHOP, 32);
        r.decks = DECKS_ALL;
        r.platforms = vec![
            solid(0, 320, 0, LOOK_STREET),
            plank(120, 200, 40, LOOK_AWNING),
        ];
        r.door = Vec2::new(int(16), Fx::ZERO);
        r.slots = vec![slot(270, 0, SLOT_STREET)];
        r.buildings = vec![bld(0, 320, 100, BUILDING_SHOPS)];
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
        r.buildings = vec![
            bld(0, 200, 90, BUILDING_BRICK),
            bld(200, 480, 100, BUILDING_SHOPS),
        ];
        rooms.push(r);

        // A rest stop: a rooftop garden with a bench and, sometimes, a raised planter.
        let mut r = Room::new("garden", ROOM_REST, 30);
        r.decks = DECKS_ALL;
        r.parts = vec![60];
        r.platforms = vec![
            solid(0, 300, 0, LOOK_ROOF),
            plank(100, 200, 36, LOOK_PLANTER),
            solid(40, 70, 10, LOOK_PLANTER).with(1),
        ];
        r.buildings = vec![bld(0, 300, 80, BUILDING_BRICK)];
        rooms.push(r);

        // R2b. A wide arena plaza with two catwalks for lobbers and flat room for bruisers. From tier 1.
        let mut r = Room::new("plaza", ROOM_ARENA, 40);
        r.min_tier = 1;
        r.platforms = vec![
            solid(0, 400, 0, LOOK_STREET),
            plank(60, 140, 40, LOOK_SCAFFOLD),
            plank(260, 340, 40, LOOK_SCAFFOLD),
        ];
        r.spawns = vec![
            any(100, 0),
            any(200, 0),
            any(300, 0),
            any(360, 0),
            any(100, 40),
            any(300, 40),
            any(40, 0),
        ];
        r.props = vec![prop(PROP_KIND_GAS, 160, 0)];
        r.buildings = vec![bld(0, 400, 120, BUILDING_OFFICE)];
        rooms.push(r);

        // R2b. A branching fork (the owner's answer to R2a): fire escapes climb from the street to a catwalk that leaves the room's
        // right edge 8 up, into a treasure alley; the street goes on into the alley's street. Whichever road you walk out on is the
        // room you get; the other is dropped.
        let alley = rooms.len() as u8 + 1;
        let mut r = Room::new("fork_up", ROOM_STREET, 48);
        r.fork = true;
        r.mirror = false;
        r.platforms = vec![
            solid(0, 480, 0, LOOK_STREET),
            flight(40, 100, 0, 30, LOOK_FIRE_ESCAPE),
            plank(100, 140, 30, LOOK_FIRE_ESCAPE),
            flight(140, 190, 30, 55, LOOK_FIRE_ESCAPE),
            plank(190, 260, 55, LOOK_BILLBOARD),
            flight(260, 320, 55, 80, LOOK_FIRE_ESCAPE),
            plank(320, 480, 80, LOOK_CATWALK),
        ];
        r.spawns = vec![any(150, 0), any(300, 0), any(430, 0), any(225, 55)];
        let mut trail = coin_line(55, 12, 95, 34, 3);
        trail.extend(coin_line(150, 42, 180, 62, 2));
        trail.extend(coin_line(270, 72, 310, 86, 3));
        trail.extend(coin_line(360, 90, 460, 90, 4));
        r.coins = trail;
        r.slots = vec![slot(380, 0, SLOT_STREET)];
        r.branch_high = alley;
        r.branch_low = alley + 1;
        r.lift = int(8);
        r.buildings = vec![
            bld(0, 200, 90, BUILDING_BRICK),
            bld(200, 480, 100, BUILDING_BRICK),
        ];
        rooms.push(r);

        // R2b. The treasure alley, the high road's room after a branching fork: catwalks between the rooftops with coins, a chest and a
        // gas canister, then a fire escape down to the street, where the run goes on. It lies over its low road's room (`under_alley`)
        // until one of them is taken, so it has no solid block but the street under both.
        let mut r = Room::new("alley", ROOM_ALLEY, 48);
        r.paired = true;
        r.decks = DECKS_ALL;
        r.mirror = false;
        r.exit_y = int(-8);
        r.platforms = vec![
            solid(0, 480, -80, LOOK_STREET),
            plank(0, 120, 0, LOOK_CATWALK),
            plank(150, 270, 0, LOOK_CATWALK),
            flight(270, 430, 0, -80, LOOK_FIRE_ESCAPE),
        ];
        r.coins = coin_line(20, 8, 110, 8, 5);
        r.coins.extend(coin_line(125, 16, 145, 16, 2));
        r.coins.extend(coin_line(160, 8, 250, 8, 5));
        r.props = vec![prop(PROP_KIND_GAS, 100, 0), prop(PROP_KIND_CHEST, 220, 0)];
        r.buildings = vec![bld(0, 480, 40, BUILDING_BRICK)];
        rooms.push(r);

        // R2b. The low road's room after a branching fork: a plain street under the alley's catwalks (nothing reaches up to them).
        let mut r = Room::new("under_alley", ROOM_STREET, 48);
        r.paired = true;
        r.decks = DECKS_ALL;
        r.mirror = false;
        r.platforms = vec![
            solid(0, 480, 0, LOOK_STREET),
            plank(60, 130, 36, LOOK_AWNING),
            plank(300, 370, 36, LOOK_AWNING),
            solid(200, 214, 12, LOOK_CRATE),
        ];
        r.spawns = vec![
            any(100, 0),
            any(170, 0),
            any(260, 0),
            any(400, 0),
            any(330, 36),
            any(90, 36),
        ];
        r.coins = coin_line(310, 44, 360, 44, 3);
        r.buildings = vec![bld(0, 480, 120, BUILDING_BRICK)];
        rooms.push(r);

        // R2b. The rare event: the runaway van. A long street with the van waiting at its start; land on its roof and it drives off
        // down the street, scattering the grunts in its path, you riding on top untouchable. Coins over its path.
        let mut r = Room::new("van_street", ROOM_EVENT, 64);
        r.mirror = false;
        r.decks = DECKS_ALL;
        r.event = EVENT_VAN;
        r.platforms = vec![
            solid(0, 640, 0, LOOK_STREET),
            plank(100, 160, 46, LOOK_AWNING),
            plank(300, 360, 46, LOOK_AWNING),
            plank(500, 560, 46, LOOK_AWNING),
        ];
        r.door = Vec2::new(int(8), Fx::ZERO);
        r.spawns = vec![
            any(160, 0),
            any(220, 0),
            any(280, 0),
            any(340, 0),
            any(400, 0),
            any(460, 0),
            any(520, 0),
            any(580, 0),
        ];
        r.coins = coin_line(140, 34, 600, 34, 12);
        r.buildings = vec![
            bld(0, 220, 100, BUILDING_SHOPS),
            bld(220, 440, 130, BUILDING_BRICK),
            bld(440, 640, 110, BUILDING_SHOPS),
        ];
        rooms.push(r);

        vertical_rooms(&mut rooms);
        skyline_rooms(&mut rooms);
        interior_rooms(&mut rooms);
        deck_forks(&mut rooms);

        RunContent {
            rules: RunRules::standard(),
            rooms,
            pieces: pieces(),
            mobs: MobKind::standard(),
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
            tiers: Tier::standard(),
            props: PropKind::standard(),
            decks: standard_decks(),
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
        h.write_u32(self.tiers.len() as u32);
        for t in &self.tiers {
            t.weight.hash_into(&mut h);
            for v in [
                t.kinds, t.budget, t.tell, t.stamina, t.damage, t.tech, t.di, t.mash, t.cap,
            ] {
                h.write_u8(v);
            }
        }
        h.write_u32(self.props.len() as u32);
        for p in &self.props {
            h.write_u8(p.role);
            p.half_width.hash_into(&mut h);
            p.height.hash_into(&mut h);
            h.write_u8(p.fuse);
            h.write_u8(p.frames);
            p.hit.hash_into(&mut h);
            p.gravity.hash_into(&mut h);
            h.write_u16(p.coins);
        }
        h.write_u32(self.decks.len() as u32);
        for d in &self.decks {
            d.y.hash_into(&mut h);
            d.wander.hash_into(&mut h);
            h.write_u8(d.weight);
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
        h.write_bool(f.skilled);
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
        for v in [
            self.high_ground,
            self.flat_room,
            self.van_speed,
            self.van_accel,
            self.view_height,
            self.cam_lift,
            self.cam_dead,
            self.cam_y_ease,
            self.cam_floor,
            self.cam_ceiling,
            self.ko_margin,
        ] {
            v.hash_into(h);
        }
        for v in [
            self.tier_rooms,
            self.arena_budget,
            self.city_rooms,
            self.event_from,
        ] {
            h.write_u8(v);
        }
        for v in [self.event_base, self.event_step, self.van_frames] {
            h.write_u16(v);
        }
        self.van_hit.hash_into(h);
        for v in [
            self.start_deck,
            self.deck_rooms_min,
            self.deck_rooms_max,
            self.deck_fork_chance,
        ] {
            h.write_u8(v);
        }
        for v in [self.skill_step, self.skill_gap, self.cam_y_ease_tall] {
            v.hash_into(h);
        }
        h.write_u16(self.emerge_gap);
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
        h.write_u32(self.props.len() as u32);
        for p in &self.props {
            h.write_u8(p.kind);
            p.x.hash_into(h);
            p.y.hash_into(h);
            h.write_i8(p.part);
        }
        h.write_u8(self.min_tier);
        h.write_u8(self.max_tier);
        h.write_u8(self.event);
        h.write_u8(self.branch_high);
        h.write_u8(self.branch_low);
        self.lift.hash_into(h);
        h.write_bool(self.paired);
        h.write_u8(self.decks);
        h.write_i8(self.deck_change);
        h.write_bool(self.pit);
        h.write_bool(self.indoor);
        h.write_u32(self.buildings.len() as u32);
        for b in &self.buildings {
            b.left.hash_into(h);
            b.right.hash_into(h);
            b.top.hash_into(h);
            h.write_u8(b.kind);
        }
    }
}

impl StateHash for MobKind {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.role);
        h.write_u8(self.body);
        for m in self.moves {
            h.write_u8(m);
        }
        for v in [self.stamina, self.aggro, self.reach, self.reach2] {
            v.hash_into(h);
        }
        h.write_u16(self.coins);
        for v in [
            self.cost,
            self.min_tier,
            self.odds,
            self.place,
            self.walk,
            self.run,
            self.cooldown,
            self.shot_time,
        ] {
            h.write_u8(v);
        }
        h.write_bool(self.armor);
        h.write_u16(self.weight);
    }
}
