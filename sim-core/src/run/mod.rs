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
pub use mobs::{grunt_weight, grunt_zone, slot as grunt_slot};
pub mod items;
mod props;
mod reach;
pub use reach::{jump_reach, recovery_reach};

pub use data::{
    Building, CoinSpot, Deck, Footing, Interior, MobKind, MobSpawn, Piece, PieceSlot, PropKind,
    PropSpot, Room, RoomLedge, RunContent, RunRules, ShopItem, Tier, ITEM_HEAL, ITEM_KINDS,
    ITEM_LIFE, LOOK_NAMES, MAX_DECKS, NO_ITEM, PEDESTALS, ROOM_ALLEY, ROOM_ARENA, ROOM_EVENT,
    ROOM_REST, ROOM_SHOP, ROOM_STREET,
};

use crate::collision;
use crate::combat::params_of;
use crate::content::{Content, Ledge, Platform, Stage};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::input::{Input, STICK_THRESHOLD};
use crate::rng::Rng;
use crate::state::{Fighter, FighterState as S, GameState, MatchRules, DRAW, NONE};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;
use data::{
    BLOCK_BOTTOM, EVENT_VAN, MAX_SHOP_ITEMS, PLACE_ALONE, PLACE_FLAT, PLACE_HIGH, ROOM_COINS,
    ROOM_LEDGES, ROOM_PARTS, ROOM_PLATFORMS, ROOM_PROPS, ROOM_SLOTS, ROOM_SPAWNS, SPOT_ANY,
};

/// Grunts alive at once: the fighter slots after the players (sim 131, `crate::MAX_GRUNTS`).
pub const MAX_MOBS: usize = crate::MAX_GRUNTS;
/// Props (hazards and chests) in the rooms in play at once.
pub const MAX_PROPS: usize = 12;
/// Rooms in play at once: `rooms_behind + 1 + rooms_ahead` may be at most this (`sim_content::check_run`).
pub const RING: usize = 4;
/// The stage's platform pool: every platform of every room in play, and two more (the van's roof and a climb's lid, R2d).
pub const POOL_PLATFORMS: usize = RING * ROOM_PLATFORMS + 2;
/// The stage's platforms: the pool, then the left wall ([`WALL_INDEX`]) and an arena's gate ([`GATE_INDEX`]).
pub const WALL_INDEX: usize = POOL_PLATFORMS;
pub const GATE_INDEX: usize = POOL_PLATFORMS + 1;
/// The ledge pool: every ledge of every room in play (R2d: 16). The shared ledge table (`GameState::ledge_owner`) has room for
/// them, but only a run uses more than the versus stages' [`crate::content::MAX_LEDGES`] (`state::ledges`).
pub const POOL_LEDGES: usize = RING * ROOM_LEDGES;
const _: () = assert!(POOL_LEDGES <= crate::state::LEDGE_SLOTS);
const _: () = assert!(ROOM_SPAWNS <= 8);
const _: () = assert!(ROOM_COINS <= 32);
const _: () = assert!(ROOM_PARTS <= 8);
const _: () = assert!(POOL_PLATFORMS + 2 <= i8::MAX as usize);
const _: () = assert!(data::INTERIOR_PLATFORMS <= POOL_PLATFORMS);

/// Where a free pool entry is put in the stage: far away, a platform no point is ever on and a ledge no fighter can reach.
const FAR: i32 = -30000;
/// How deep solid ground reaches once laid (world height; far below any blast zone the run's drift allows).
pub const DEEP: i32 = -160;
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
/// R2b: the shopkeeper is sulking after a bonk and will not sell.
pub const NOTE_REFUSED: u8 = 5;
/// R3: the one consumable slot is taken (a bubble up, a bomb not yet burst).
pub const NOTE_SLOT_FULL: u8 = 6;
/// R3: the bell was rung (the unsold pedestals re-drawn).
pub const NOTE_REROLLED: u8 = 7;

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
/// Sulking after the bonk ("Hmph!", arms crossed): it refuses to sell until the player has gone on into a new room
/// (`Run::keeper_grudge`; R2c, the owner's rule).
pub const KEEPER_HMPH: u8 = 5;
/// R3: ringing up fresh stock after the bell.
pub const KEEPER_RESTOCK: u8 = 6;

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
    /// The room that laid it (`RoomSlot::seq`; ground joined across a seam keeps the first room's).
    pub seq: u32,
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
    /// The room that laid it.
    pub seq: u32,
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
    /// R2b. The grunt kind the budget gave each spawn spot when the room was laid ([`SPOT_ANY`] for none), and the room's difficulty
    /// tier.
    pub kinds: [u8; ROOM_SPAWNS],
    pub tier: u8,
    /// R2d. The lowest surface laid in it and the highest of its two ground ends (its blast lines and the camera's span), the deck
    /// it is entered on and the deck it leaves on (two different decks: a room that climbs or drops), and whether a lid has closed
    /// over it (a climb left behind: its back edge is a gate below you, [`enter_rooms`]).
    pub lo: Fx,
    pub hi: Fx,
    pub deck_in: u8,
    pub deck_out: u8,
    pub lid: bool,
    /// R2e (the owner's note A). The frame its last grunt came out of a door (plus 1; 0 none yet): the next waits
    /// `RunRules::emerge_gap` frames.
    pub emerged_at: u32,
    /// R2e. Its skilled route is not offered to this run's fighter, who cannot make it (`skilled_offered`): laid without it.
    pub skill_off: bool,
    /// R3 (F8). What it holds as a find (`data::FIND_*`, drawn as it is laid; where: [`items::find_spot`]), the item if an item
    /// (`NO_ITEM` else), and whether it has been taken.
    pub find: u8,
    pub find_item: u8,
    pub found: bool,
}

impl RoomSlot {
    /// It changes deck (a climb, a drop or a shaft): the camera follows its height faster, and it is a gate once left.
    pub fn vertical(&self) -> bool {
        self.deck_in != self.deck_out
    }
}

/// `Mob::mode` values.
pub const MODE_IDLE: u8 = 0;
pub const MODE_MOVE: u8 = 1;
pub const MODE_WINDUP: u8 = 2;
pub const MODE_ATTACK: u8 = 3;
pub const MODE_COOLDOWN: u8 = 4;
/// Coming out of its door or stairwell for `RunRules::emerge_frames` (R2a): it neither moves, hits nor can be hit.
pub const MODE_EMERGE: u8 = 6;
/// A leaper's leap at you, before its aerial. (5, 7 and 8 were R2b's bursting, guarding and dazed: a grunt is a fighter now, and
/// its shield, its broken shield and its launches are its fighter's.)
pub const MODE_LEAP: u8 = 9;

/// A grunt's behaviour (see `run::mobs`): what its kind's player has in mind. The grunt itself is fighter `MAX_FIGHTERS + n`
/// (`mobs::slot`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mob {
    pub active: bool,
    /// Index into `RunContent::mobs`.
    pub kind: u8,
    /// `MODE_*`, and the frames left in it (the tell, the cooldown, the emergence, or the tries left to make an attack come out).
    pub mode: u8,
    pub timer: u16,
    /// The move of its kit it is telling or making (a slot of `MobKind::moves`), and how far its input for it has got (255: the
    /// move is out).
    pub move_slot: u8,
    pub step: u8,
    /// Frames left in which leaving through a blast zone counts as the player's kill (a hit or a grab of theirs sets it).
    pub credit: u16,
    /// Its fighter's percent last frame (a rise is a hit: the combo and the credit).
    pub last_percent: Fx,
    /// The room it came from (`RoomSlot::seq`): an arena opens once all of its own grunts are beaten. Its room's tier.
    pub room: u32,
    pub tier: u8,
    /// R2e (F3). Off the ground at the end of last frame (a hit landing now is an air hit: it counts double toward the combo), the
    /// hits taken since it last stood (the first may launch it: a juggle), and the grunts it has bowled into while flying since its
    /// launch (a bit each: each once a launch).
    pub air: bool,
    pub juggle: u8,
    pub bowled: u16,
    /// R3 (hot hands). Frames left of its burn.
    pub burn: u8,
}

/// `Run::callout` values: the last bonus the HUD calls out.
pub const CALL_NONE: u8 = 0;
/// A juggled grunt knocked out: its coins times the juggle ("AIR x3").
pub const CALL_AIR: u8 = 1;
/// Launched out of the top: "SKYWARD!".
pub const CALL_SKY: u8 = 2;
/// Spiked out of the bottom: "GROUNDED!".
pub const CALL_SPIKE: u8 = 3;
/// A launched grunt bowled into another: "BOWLED!".
pub const CALL_BOWL: u8 = 4;

/// `Prop::mode` values.
pub const PROP_READY: u8 = 0;
/// A gas canister burning after a hit (`PropKind::fuse` frames), then bursting.
pub const PROP_LIT: u8 = 1;
pub const PROP_BURST: u8 = 2;
/// A sign falling (its hit live), then broken where it lands.
pub const PROP_FALLING: u8 = 3;
pub const PROP_BROKEN: u8 = 4;
/// A chest hit open (it has paid).
pub const PROP_OPEN: u8 = 5;

/// A hazard or a chest in a room in play (R2b; `run::props`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Prop {
    pub active: bool,
    /// Index into `RunContent::props`.
    pub kind: u8,
    /// The room it belongs to (`RoomSlot::seq`).
    pub room: u32,
    /// `PROP_*`, and the frames left in it.
    pub mode: u8,
    pub timer: u16,
    /// The middle of its base (a sign: of its top, where it hangs), and its speed while falling.
    pub pos: Vec2,
    pub vel: Vec2,
    /// Who its hit has hit: mob slots (bit each), and the player.
    pub hit_mobs: u16,
    pub hit_player: bool,
}

/// `Van::mode` values.
pub const VAN_NONE: u8 = 0;
/// Parked at the event street's start, engine running, until the player lands on its roof.
pub const VAN_WAITING: u8 = 1;
pub const VAN_DRIVING: u8 = 2;
/// Stopped at the end of its run.
pub const VAN_PARKED: u8 = 3;
/// The van's roof: half its width and its height over the street.
pub const VAN_HALF: Fx = Fx::from_ratio(22, 10);
pub const VAN_HEIGHT: Fx = Fx::from_ratio(19, 10);

/// The runaway van (R2b, the first rare event: `RUNNER_WORLD.md` 5): ride its roof down the street, untouchable, scattering grunts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Van {
    pub mode: u8,
    /// Its event room (`RoomSlot::seq`).
    pub room: u32,
    /// The middle of its wheels on the street, and its speed (units a frame) and frames driven.
    pub x: Fx,
    pub y: Fx,
    pub speed: Fx,
    pub timer: u16,
    /// Its roof's pool platform.
    pub platform: i8,
    /// The player is riding it this frame (untouchable).
    pub riding: bool,
    /// Grunts it has run into, and which grunt slots (a bit each: once each a ride).
    pub hits: u16,
    pub struck: u16,
}

/// Rooms the generator plans ahead at most on a line (R2e, F4: the map shows them; `RunRules::map_ahead`).
pub const PLAN: usize = 4;

/// A room the generator has planned (R2e, F4): which room, mirrored or not, the decks it enters and leaves on, and the number it
/// will be laid as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Planned {
    pub live: bool,
    pub room: u8,
    pub mirror: bool,
    pub deck_in: u8,
    pub deck_out: u8,
    pub seq: u32,
}

impl StateHash for Planned {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.live);
        h.write_u8(self.room);
        h.write_bool(self.mirror);
        h.write_u8(self.deck_in);
        h.write_u8(self.deck_out);
        h.write_u32(self.seq);
    }
}

/// The generator's state (R2e, F4): what R2d's run kept to draw its next room (its own stream, the shop spacing, the type of the
/// last room, the rare event's pity, the deck plan), on its own, so that it can plan rooms ahead, and plan each road of a branching
/// fork apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Planner {
    pub rng: Rng,
    /// The number the next room it plans will be laid as, and the height its ground begins at.
    pub count: u32,
    pub y: Fx,
    /// The last room planned (index into the content) and its type.
    pub last_room: u8,
    pub last_kind: u8,
    /// Rooms since the last shop, and how many more make the next one.
    pub since_shop: u8,
    pub shop_due: u8,
    /// Rare events: rooms since the last event (the pity counter), and the "city" of the last one plus 1 (0: none yet).
    pub event_pity: u16,
    pub event_city: u32,
    /// R2d. The deck plan: the deck the run is heading for or keeping to, and how many more rooms it keeps to it.
    pub plan_deck: u8,
    pub plan_left: u8,
}

impl Planner {
    pub const INACTIVE: Planner = Planner {
        rng: Rng::new(0),
        count: 0,
        y: Fx::ZERO,
        last_room: 0,
        last_kind: 0,
        since_shop: 0,
        shop_due: 0,
        event_pity: 0,
        event_city: 0,
        plan_deck: 0,
        plan_left: 0,
    };
}

impl StateHash for Planner {
    fn hash_into(&self, h: &mut StateHasher) {
        self.rng.hash_into(h);
        h.write_u32(self.count);
        self.y.hash_into(h);
        h.write_u8(self.last_room);
        h.write_u8(self.last_kind);
        h.write_u8(self.since_shop);
        h.write_u8(self.shop_due);
        h.write_u16(self.event_pity);
        h.write_u32(self.event_city);
        h.write_u8(self.plan_deck);
        h.write_u8(self.plan_left);
    }
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
    /// Rooms laid so far (the next one's number).
    pub placed: u32,
    /// The type of each of the last 16 rooms laid, 4 bits each, newest in the low bits (for the map): the type in the low 3 bits,
    /// and 8 for a fork.
    pub kinds_log: u64,
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
    /// R2c (the owner's rule). After a bonk the shopkeeper refuses to sell until the player has entered a room further on than the
    /// shop: the room number (`Run::room`) that lifts it, 0 for none.
    pub keeper_grudge: u32,
    /// The hazards and chests of the rooms in play, and chests opened.
    pub props: [Prop; MAX_PROPS],
    pub chests: u16,
    pub van: Van,
    /// Rooms laid but not taken (the other road of a branching fork), a bit each, newest in the low bit, as `kinds_log`.
    pub skipped_log: u16,
    /// R2c. The view's height (its middle; the game's camera follows it), and the grunts knocked out by the blast zones so far with
    /// where the last one left (for the game's KO effect).
    pub cam_y: Fx,
    pub kos: u16,
    pub ko_at: Vec2,
    /// R2e (F4). The generator, planning ahead ([`Planner`]: R2d's deck plan, the shop spacing, the rare events' pity): the main
    /// line's planner and the rooms it has planned, nearest first; once a branching fork is planned (`forked`), its two roads and a
    /// planner and plan for the line after each (index 0 the high road, 1 the low road).
    pub planner: Planner,
    pub plan: [Planned; PLAN],
    pub forked: bool,
    pub roads: [Planned; 2],
    pub branch_planner: [Planner; 2],
    pub branch_plan: [[Planned; PLAN]; 2],
    /// R2e. The fighter's reach (`x` across, `y` up): a full hop and a midair jump ([`jump_reach`]), and with its recovery special
    /// measured in the sim ([`recovery_reach`]). Worked out as the run starts; a room's skilled route it cannot make is not offered.
    pub reach_jump: Vec2,
    pub reach_recovery: Vec2,
    /// R2e (F3). The last bonus called out (`CALL_*`), a count that changes with every callout (the HUD shows each once), where it
    /// happened and the coins it paid; the tallies of skyward and grounded knock-outs, bowls and juggled knock-outs.
    pub callout: u8,
    pub callouts: u16,
    pub callout_at: Vec2,
    pub callout_coins: u16,
    pub sky_kos: u16,
    pub spike_kos: u16,
    pub bowls: u16,
    pub air_kos: u16,
    /// R3, the shop's stock: the item on each drawn pedestal (`NO_ITEM` empty), the bell's rings in this shop, and the stream the
    /// draws come from (each shop's own, from the run's seed and the shop's room: [`items::stock_shop`]).
    pub shop_stock: [u8; PEDESTALS],
    pub shop_rerolls: u8,
    pub shop_rng: Rng,
    /// R3. The run's seed (each shop's stream and nothing else).
    pub seed: u64,
    /// R3, the build: how many of each item kind the run owns (`data::ITEM_*`), and the one consumable slot (`NO_ITEM`, or the
    /// bubble up or the bomb not yet burst).
    pub owned: [u8; ITEM_KINDS],
    pub slot: u8,
    /// The bought bomb in the item pool (`state::NONE` until it is handed over at the door), and whether it has been thrown.
    pub bomb: i8,
    pub bomb_thrown: bool,
    /// Second wind's frames still to give once the respawn invincibility ends; steady combo's forgiving hit (ready or spent); the
    /// shadow orbs' clock to the next orb.
    pub wind: u16,
    pub guard: bool,
    pub orb_timer: u16,
    /// Where the run keeps its fighter's own parameters in its content copy (stat items rebuild from them: [`items::refresh`]), and
    /// the body whose weapon has the shadow orbs (`NO_ITEM` none) with that move.
    pub base_fighter: u8,
    pub orb_body: u8,
    pub orb_move: u8,
    /// R3, cues for the game: bubbles popped, combos saved by steady combo, grunt knock-outs healed by lifesteal, finds taken (and
    /// what the last was, `data::FIND_*`), and the "city" of the last 1-up found plus 1.
    pub pops: u16,
    pub saves: u16,
    pub steals: u16,
    pub finds: u16,
    pub find_last: u8,
    pub life_city: u32,
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
        kinds_log: 0,
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
            kinds: [SPOT_ANY; ROOM_SPAWNS],
            tier: 0,
            lo: Fx::ZERO,
            hi: Fx::ZERO,
            deck_in: 0,
            deck_out: 0,
            lid: false,
            emerged_at: 0,
            skill_off: false,
            find: 0,
            find_item: NO_ITEM,
            found: false,
        }; RING],
        platforms: [PoolPlatform {
            live: false,
            left: Fx::ZERO,
            right: Fx::ZERO,
            y: Fx::ZERO,
            bottom: Fx::ZERO,
            pass_through: false,
            rise: Fx::ZERO,
            seq: 0,
        }; POOL_PLATFORMS],
        ledges: [PoolLedge {
            live: false,
            x: Fx::ZERO,
            y: Fx::ZERO,
            side: 0,
            seq: 0,
        }; POOL_LEDGES],
        layout: 0,
        mobs: [Mob {
            active: false,
            kind: 0,
            mode: 0,
            timer: 0,
            move_slot: 0,
            step: 0,
            credit: 0,
            last_percent: Fx::ZERO,
            room: 0,
            tier: 0,
            air: false,
            juggle: 0,
            bowled: 0,
            burn: 0,
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
        keeper_grudge: 0,
        props: [Prop {
            active: false,
            kind: 0,
            room: 0,
            mode: PROP_READY,
            timer: 0,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            hit_mobs: 0,
            hit_player: false,
        }; MAX_PROPS],
        chests: 0,
        van: Van {
            mode: VAN_NONE,
            room: 0,
            x: Fx::ZERO,
            y: Fx::ZERO,
            speed: Fx::ZERO,
            timer: 0,
            platform: NONE,
            riding: false,
            hits: 0,
            struck: 0,
        },
        skipped_log: 0,
        cam_y: Fx::ZERO,
        kos: 0,
        ko_at: Vec2::ZERO,
        planner: Planner::INACTIVE,
        plan: [Planned {
            live: false,
            room: 0,
            mirror: false,
            deck_in: 0,
            deck_out: 0,
            seq: 0,
        }; PLAN],
        forked: false,
        roads: [Planned {
            live: false,
            room: 0,
            mirror: false,
            deck_in: 0,
            deck_out: 0,
            seq: 0,
        }; 2],
        branch_planner: [Planner::INACTIVE; 2],
        branch_plan: [[Planned {
            live: false,
            room: 0,
            mirror: false,
            deck_in: 0,
            deck_out: 0,
            seq: 0,
        }; PLAN]; 2],
        reach_jump: Vec2::ZERO,
        reach_recovery: Vec2::ZERO,
        callout: CALL_NONE,
        callouts: 0,
        callout_at: Vec2::ZERO,
        callout_coins: 0,
        sky_kos: 0,
        spike_kos: 0,
        bowls: 0,
        air_kos: 0,
        shop_stock: [NO_ITEM; PEDESTALS],
        shop_rerolls: 0,
        shop_rng: Rng::new(0),
        seed: 0,
        owned: [0; ITEM_KINDS],
        slot: NO_ITEM,
        bomb: NONE,
        bomb_thrown: false,
        wind: 0,
        guard: false,
        orb_timer: 0,
        base_fighter: 0,
        orb_body: NO_ITEM,
        orb_move: 0,
        pops: 0,
        saves: 0,
        steals: 0,
        finds: 0,
        find_last: 0,
        life_city: 0,
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

    /// The blast zone's bottom and top: below the lowest surface of the rooms in play (R2d: a deck below caught a fall; a climb's
    /// lid is its lowest), and above the highest ground line.
    pub fn blast_y(&self, rules: &RunRules) -> (Fx, Fx) {
        let live = || self.slots.iter().filter(|s| s.live);
        let lo = live()
            .map(|s| {
                if s.lid && s.y_end > s.y {
                    s.y_end
                } else {
                    s.lo
                }
            })
            .min()
            .unwrap_or(Fx::ZERO);
        // (R2e: a drop's lid closes its top.)
        let hi = live()
            .map(|s| {
                if s.lid && s.y_end < s.y {
                    lid_line(s, rules)
                } else {
                    s.hi
                }
            })
            .max()
            .unwrap_or(Fx::ZERO);
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

    /// Whether room `seq` was laid but not taken (the other road of a branching fork), if it is among the last 16 laid.
    pub fn skipped(&self, seq: u32) -> Option<bool> {
        let back = self.placed.checked_sub(seq + 1)?;
        (back < 16).then(|| self.skipped_log >> back & 1 != 0)
    }

    /// The rooms in play ahead of the current one, nearest first (two at the same place: a branching fork's two roads, not taken
    /// yet).
    pub fn ahead(&self) -> ([RoomSlot; RING], usize) {
        let mut out = [RoomSlot::default(); RING];
        let mut n = 0;
        for s in self.slots.iter().filter(|s| s.live && s.seq > self.room) {
            out[n] = *s;
            n += 1;
        }
        out[..n].sort_by_key(|s| s.seq);
        (out, n)
    }

    /// The two roads of a branching fork, laid and not taken yet: (the high road's alley, the low road's room).
    pub fn branches(&self) -> Option<(RoomSlot, RoomSlot)> {
        let (a, n) = self.ahead();
        (n >= 2 && a[0].x == a[1].x).then_some((a[0], a[1]))
    }
}

/// The difficulty tier of room `seq` (R2b): one step every `RunRules::tier_rooms` rooms, up to the last tier.
pub fn tier_of(rc: &RunContent, seq: u32) -> u8 {
    let step = u32::from(rc.rules.tier_rooms.max(1));
    let last = rc.tiers.len().saturating_sub(1).min(255) as u32;
    (seq / step).min(last) as u8
}

/// The "city" room `seq` is in (until R6's journey: `RunRules::city_rooms` rooms each); at most one rare event in each.
pub fn city_of(rc: &RunContent, seq: u32) -> u32 {
    seq / u32::from(rc.rules.city_rooms.max(1))
}

// ---- The decks (R2d) -------------------------------------------------------------------------------------------------------------

/// The deck whose line is nearest height `y` (the lower on a tie).
pub fn deck_at(rc: &RunContent, y: Fx) -> u8 {
    let mut best = (0u8, Fx::MAX);
    for (i, d) in rc.decks.iter().enumerate().take(MAX_DECKS) {
        let dist = (y - d.y).abs();
        if dist < best.1 {
            best = (i as u8, dist);
        }
    }
    best.0
}

/// Whether height `y` lies in deck `d`'s band (its line, give or take its wander).
pub fn in_band(rc: &RunContent, d: u8, y: Fx) -> bool {
    rc.decks
        .get(usize::from(d))
        .is_some_and(|k| (y - k.y).abs() <= k.wander)
}

/// The deck room `room` leaves on, laid mirrored or not and entered on deck `entry`; `None` if it may not be entered there. (A
/// mirrored room is entered where it would leave: a mirrored climb drops.)
pub fn exit_deck(room: &Room, mirror: bool, entry: u8) -> Option<u8> {
    let c = i16::from(room.deck_change);
    let first = if mirror {
        i16::from(entry) - c
    } else {
        i16::from(entry)
    };
    let fits = |d: i16| (0..MAX_DECKS as i16).contains(&d);
    if !fits(first) || room.decks & (1 << first) == 0 {
        return None;
    }
    let out = if mirror { first } else { first + c };
    fits(out).then_some(out as u8)
}

/// The mirrors a room may be laid as.
fn mirrors(room: &Room) -> &'static [bool] {
    if room.mirror {
        &[false, true]
    } else {
        &[false]
    }
}

/// Room `room`, entered on deck `entry` at height `y`, can be laid so that it leaves on deck `out`, its ground ending in that deck's
/// band (some mirror does it).
pub fn leads_to(rc: &RunContent, room: &Room, entry: u8, y: Fx, out: u8) -> bool {
    mirrors(room).iter().any(|&m| {
        exit_deck(room, m, entry) == Some(out) && in_band(rc, out, y + room_exit(room, m))
    })
}

/// A branching fork whose two roads are on different decks (a deck fork), entered on deck `entry`.
pub fn is_deck_fork(rc: &RunContent, room: &Room, entry: u8) -> bool {
    let Some(d) = rc.decks.get(usize::from(entry)) else {
        return false;
    };
    let low = d.y + room.exit_y;
    room.branch_high != 0 && deck_at(rc, low) != deck_at(rc, low + room.lift)
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
    /// R2d: on the room's skilled route ([`Footing::skilled`]).
    pub skilled: bool,
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
        .filter(|p| part_on(slot.parts, p.part) && !(p.skilled && slot.skill_off))
    {
        let platform = frame.platform(&p.platform);
        f(Placed {
            platform,
            look: p.look,
            base: platform.bottom,
            skilled: p.skilled,
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
                skilled: false,
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
        if part_on(slot.parts, c.part) && !(c.skilled && slot.skill_off) && n < ROOM_COINS {
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

/// Every prop of a laid room that is there in its variant: its number, its spot and where it stands in the world.
pub fn for_each_prop(rc: &RunContent, slot: &RoomSlot, mut f: impl FnMut(usize, &PropSpot, Vec2)) {
    let Some(room) = rc.rooms.get(usize::from(slot.room)) else {
        return;
    };
    let frame = Frame::of(slot, room);
    for (n, p) in room.props.iter().enumerate().take(ROOM_PROPS) {
        if part_on(slot.parts, p.part) {
            f(n, p, frame.point(Vec2::new(p.x, p.y)));
        }
    }
}

/// Every building mass of a laid room (R2e, note B), in the world: (left, right, top, kind).
pub fn for_each_building(rc: &RunContent, slot: &RoomSlot, mut f: impl FnMut(Fx, Fx, Fx, u8)) {
    let Some(room) = rc.rooms.get(usize::from(slot.room)) else {
        return;
    };
    let frame = Frame::of(slot, room);
    for b in &room.buildings {
        let (a, c) = (frame.x(b.left), frame.x(b.right));
        f(a.min(c), a.max(c), frame.y(b.top), b.kind);
    }
}

/// `support_of` answers: what holds a footing up (R2e, the owner's note B: nothing floats).
/// Nothing: it floats (refused by `sim_content::check_run`).
pub const SUPPORT_NONE: u8 = 0;
/// It is the ground, a roof or a tower (a block reaching the bottom), or an object standing on one.
pub const SUPPORT_GROUND: u8 = 1;
/// It hangs on the face of a building behind it (a balcony, a fire escape, an awning, a sill, a wall or ceiling inside one).
pub const SUPPORT_WALL: u8 = 2;
/// It spans between two faces (a catwalk or girder bridging a gap: drawn as a truss).
pub const SUPPORT_SPAN: u8 = 3;
/// It stands on posts down to the surface below (a scaffold's poles, a water tower's legs, a crane's mast, a catwalk's stilts).
pub const SUPPORT_POSTS: u8 = 4;

/// Looks that hang on a building's face and need one behind them.
const ATTACHED: [u8; 7] = [
    data::LOOK_BALCONY,
    data::LOOK_FIRE_ESCAPE,
    data::LOOK_AWNING,
    data::LOOK_LEDGE_BLOCK,
    data::LOOK_STAIRWELL,
    data::LOOK_WALL,
    data::LOOK_CEILING,
];

/// What holds footing `k` of a laid room's footings (`all`, as [`for_each_footing`] gives them) up, and a height: for posts the lowest
/// they reach down to (the surface under each end; the game draws each to its own), for a footing on a building the solid floor under
/// its middle (or its own top). A block reaching the bottom is the ground; a look
/// that hangs on a building needs a building mass behind it (its top at least half a unit over a plank, or level with a block's top),
/// and so does anything in an interior; anything else is held by a building behind it if there is one (bolted to its face), else spans two
/// faces (each end within 0.6 of a block's face or a building's edge at least as high), else stands on posts down to whatever lies
/// under both its ends.
pub fn support_of(
    all: &[Placed],
    k: usize,
    buildings: &[(Fx, Fx, Fx, u8)],
    indoor: bool,
) -> (u8, Fx) {
    let p = all[k].platform;
    let look = all[k].look;
    if !p.pass_through && p.bottom <= Fx::from_int(DEEP) {
        return (SUPPORT_GROUND, p.bottom);
    }
    let top = p.y.max(p.top_right());
    let margin = if p.pass_through { Fx::HALF } else { Fx::ZERO };
    let behind = buildings
        .iter()
        .any(|&(l, r, t, _)| l <= p.left && r >= p.right && t >= top + margin);
    if ATTACHED.contains(&look) || indoor {
        if !behind {
            return (SUPPORT_NONE, top);
        }
        // (The floor under its middle, for the game: a door under an awning that has ground just below it.)
        let mid = (p.left + p.right) / Fx::from_int(2);
        let floor = all
            .iter()
            .enumerate()
            .filter(|(j, q)| {
                *j != k
                    && !q.platform.pass_through
                    && mid >= q.platform.left
                    && mid <= q.platform.right
                    && q.platform.top_at(mid) < low_of(&p)
            })
            .map(|(_, q)| q.platform.top_at(mid))
            .max()
            .unwrap_or(top);
        return (SUPPORT_WALL, floor);
    }
    let low = p.y.min(p.top_right());
    if behind {
        // (A catwalk, a girder or a billboard's walkway in front of a building is bolted to its face.)
        return (SUPPORT_WALL, low);
    }
    let near = Fx::from_ratio(3, 5);
    let face_at = |x: Fx| {
        all.iter().enumerate().any(|(j, q)| {
            let q = q.platform;
            j != k
                && !q.pass_through
                && ((q.right - x).abs() <= near || (q.left - x).abs() <= near)
                && q.y.max(q.top_right()) >= low - Fx::from_ratio(1, 10)
        }) || buildings.iter().any(|&(l, r, t, _)| {
            ((r - x).abs() <= near || (l - x).abs() <= near) && t >= low - Fx::from_ratio(1, 10)
        })
    };
    if face_at(p.left) && face_at(p.right) {
        return (SUPPORT_SPAN, low);
    }
    // Posts: the highest surface under each end: solid ground first (the poles reach down to it, past any plank), then a plank, then
    // a building's roof.
    let under = |x: Fx| {
        let below = |solid: bool| {
            all.iter()
                .enumerate()
                .filter(|(j, q)| {
                    *j != k
                        && q.platform.pass_through != solid
                        && x >= q.platform.left
                        && x <= q.platform.right
                        && q.platform.top_at(x) < p.top_at(x) - Fx::from_ratio(1, 5)
                })
                .map(|(_, q)| q.platform.top_at(x))
                .max()
        };
        let roofs = buildings
            .iter()
            .filter(|&&(l, r, t, _)| x >= l && x <= r && t < p.top_at(x) - Fx::from_ratio(1, 5))
            .map(|&(_, _, t, _)| t)
            .max();
        below(true).or_else(|| below(false)).or(roofs)
    };
    let inset = ((p.right - p.left) * Fx::from_ratio(1, 10)).min(Fx::from_ratio(3, 10));
    match (under(p.left + inset), under(p.right - inset)) {
        (Some(a), Some(b)) => (SUPPORT_POSTS, a.min(b)),
        _ => (SUPPORT_NONE, low),
    }
}

fn low_of(p: &Platform) -> Fx {
    p.y.min(p.top_right())
}

/// Every footing of a laid room with what holds it up ([`support_of`]).
pub fn supports(rc: &RunContent, slot: &RoomSlot) -> Vec<(Placed, u8, Fx)> {
    let mut all = Vec::new();
    for_each_footing(rc, slot, |p| all.push(p));
    let mut bs = Vec::new();
    for_each_building(rc, slot, |l, r, t, k| bs.push((l, r, t, k)));
    let indoor = rc
        .rooms
        .get(usize::from(slot.room))
        .is_some_and(|r| r.indoor);
    (0..all.len())
        .map(|k| {
            let (s, y) = support_of(&all, k, &bs, indoor);
            (all[k], s, y)
        })
        .collect()
}

/// The jumps a room's skilled route needs (R2e: `Room::recovery`): (up, across).
pub fn skilled_need(rules: &RunRules, room: &Room) -> Vec2 {
    match room.recovery {
        data::RECOVER_UP => Vec2::new(rules.skill_gap, rules.recovery_step),
        data::RECOVER_ACROSS => Vec2::new(rules.recovery_gap, rules.skill_step),
        _ => Vec2::new(rules.skill_gap, rules.skill_step),
    }
}

/// Whether a fighter of reach `jump` (a double jump) and `recovery` (with its up special) makes room `room`'s skilled route (R2e:
/// the owner's "recovery specials sometimes"; a route it cannot make is never offered to it).
pub fn skilled_offered(rules: &RunRules, room: &Room, jump: Vec2, recovery: Vec2) -> bool {
    let need = skilled_need(rules, room);
    let (up, across) = if room.recovery == 0 {
        (jump.y, jump.x)
    } else {
        (jump.y.max(recovery.y), jump.x.max(recovery.x))
    };
    up >= need.y && across >= need.x
}

/// Grunt `n` as a fighter (R2c: it is one, `GameState::fighters[MAX_FIGHTERS + n]`), for the game to draw with the fighter rig.
pub fn grunt_fighter(state: &GameState, n: usize) -> Fighter {
    state.fighters[mobs::slot(n.min(MAX_MOBS - 1))]
}

/// Puts a grunt of `kind` (of `tier`, belonging to the current room) awake at `at`, on the ground there if there is any, facing the
/// player: for tests, tools and the game's debug calls. `world.stage` must be the run's (`build_stage`). Returns its number.
pub fn place_grunt(
    state: &mut GameState,
    world: &Content,
    rc: &RunContent,
    kind: u8,
    at: Vec2,
    tier: u8,
) -> Option<usize> {
    let platform = collision::standing_on(&world.stage, at);
    let room = state.run.room;
    let n = mobs::spawn(state, world, rc, kind, at, platform, tier, room)?;
    let m = &mut state.run.mobs[n];
    m.mode = MODE_MOVE;
    m.timer = 0;
    let player_x = state.fighters[PLAYER].pos.x;
    let f = &mut state.fighters[mobs::slot(n)];
    f.facing = if player_x >= at.x { 1 } else { -1 };
    Some(n)
}

/// Removes grunt `n` (for tests, tools and the game's debug calls).
pub fn remove_grunt(state: &mut GameState, world: &Content, n: usize) {
    if n < MAX_MOBS {
        mobs::remove(state, world, n);
    }
}

/// Removes every grunt.
pub fn clear_grunts(state: &mut GameState, world: &Content) {
    for n in 0..MAX_MOBS {
        if state.run.mobs[n].active {
            mobs::remove(state, world, n);
        }
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
    let ground = rc
        .decks
        .get(usize::from(rc.rules.start_deck))
        .map_or(Fx::ZERO, |d| d.y);
    // (R2e: the generator plans on its own stream, so a room's variant drawn later never changes what the map showed.)
    let mut planner = Planner::INACTIVE;
    planner.rng = Rng::new(seed ^ 0x504c_414e_4e45_5253);
    planner.y = ground;
    planner.shop_due = draw_shop_due(&mut planner, &rc.rules);
    // (R2d: the first room is on the start deck's line, and the run keeps to that deck for a stretch.)
    planner.plan_deck = rc.rules.start_deck;
    planner.plan_left = draw_stretch(&mut planner, &rc.rules);
    run.planner = planner;
    run.next_y = ground;
    run.room = 0;
    run.van.platform = NONE;
    // (R3: each shop's stock and each room's find come from streams of their own, from the seed; the fighter's own parameters are
    // kept in the run's content copy for the stat items to rebuild from; the orbs item fires the body whose weapon has them.)
    run.seed = seed;
    run.base_fighter = items::stash_base(world, char_id);
    if let Some((body, mv)) = items::find_orbs(world) {
        run.orb_body = body;
        run.orb_move = mv;
    }
    // (R2e: the fighter's reach, once: a skilled route it cannot make is not offered.)
    if let Some(p) = world.fighters.get(usize::from(char_id)) {
        let (h, d) = jump_reach(p);
        run.reach_jump = Vec2::new(d, h);
        let (h, d) = recovery_reach(world, char_id);
        run.reach_recovery = Vec2::new(d, h);
    }
    place_room(&mut run, rc);
    while run.ahead().1 < usize::from(rc.rules.rooms_ahead.max(1)) {
        if !place_room(&mut run, rc) {
            break;
        }
    }
    let start = Vec2::new(Fx::from_int(6), ground);
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
    state.run.cam_y = start.y + rc.rules.cam_lift;
    spawn_mobs(&mut state, world, rc);
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
    // (R3: the build on the fighter: its parameters rebuilt from its own with the stat items, its damage and its shield.)
    items::refresh(state, world, rc);
    let input = inputs[PLAYER];
    if state.run.over {
        remember_input(&mut state.run, input);
        state.frame = state.frame.wrapping_add(1);
        return;
    }
    if state.run.inside {
        // (The world outside stands still: its grunts with it.)
        let saved = state.fighters;
        for f in state.fighters[MAX_FIGHTERS..].iter_mut() {
            f.active = false;
        }
        crate::step(state, world, inputs);
        state.fighters[MAX_FIGHTERS..].copy_from_slice(&saved[MAX_FIGHTERS..]);
        shop_step(state, world, rc, input);
        remember_input(&mut state.run, input);
        build_stage(&state.run, rc, &mut world.stage);
        return;
    }
    // The grunts are fighters (sim 131): their players' inputs, then the versus step for everyone.
    mobs::through_gate(state, world, rc);
    let mut all = mobs::think(state, world, rc);
    all[..MAX_FIGHTERS].copy_from_slice(inputs);
    items::before_step(state, world, rc);
    let pops = state.run.pops;
    crate::step::step_bodies(state, world, &all);
    mobs::after_step(state, world, rc);
    mobs::bowl(state, world, rc);
    mobs::van(state, world, rc);
    props::update(state, world, rc);
    enter_rooms(state, world, rc);
    follow_camera(state, &rc.rules);
    if creeping(&rc.rules) {
        push_from_edge(state, world, &rc.rules);
    }
    mobs::knock_outs(state, world, rc);
    collect_coins(state, rc);
    items::take_finds(state, rc);
    items::after_step(state, world, rc, pops);
    settle_combo(state);
    check_arena(state, rc);
    if let Some(seq) = door_here(state, rc).filter(|_| pressed_up(&state.run, input)) {
        enter_shop(state, world, rc, seq);
    }
    remember_input(&mut state.run, input);
    build_stage(&state.run, rc, &mut world.stage);
    spawn_mobs(state, world, rc);
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
fn draw_shop_due(p: &mut Planner, rules: &RunRules) -> u8 {
    let lo = rules.shop_min.max(1);
    let hi = rules.shop_max.max(lo);
    lo + p.rng.range(u32::from(hi - lo) + 1) as u8
}

/// How many rooms the run keeps to a deck (R2d), drawn from the rules' range.
fn draw_stretch(p: &mut Planner, rules: &RunRules) -> u8 {
    let lo = rules.deck_rooms_min.max(1);
    let hi = rules.deck_rooms_max.max(lo);
    lo + p.rng.range(u32::from(hi - lo) + 1) as u8
}

/// The deck the plan heads for next (R2d): another deck than `here`, by the decks' weights (`here` when there is no other).
fn draw_deck(p: &mut Planner, rc: &RunContent, here: u8) -> u8 {
    let weight = |d: usize| {
        if d == usize::from(here) {
            0
        } else {
            u32::from(rc.decks[d].weight)
        }
    };
    let n = rc.decks.len().min(MAX_DECKS);
    let total: u32 = (0..n).map(weight).sum();
    if total == 0 {
        return here;
    }
    let mut roll = p.rng.range(total);
    for d in 0..n {
        if roll < weight(d) {
            return d as u8;
        }
        roll -= weight(d);
    }
    here
}

/// One of the rooms `fits` allows, drawn evenly (`None` if there is none).
fn pick(rng: &mut Rng, count: usize, fits: impl Fn(usize) -> bool) -> Option<u8> {
    let n = (0..count).filter(|&i| fits(i)).count() as u32;
    if n == 0 {
        return None;
    }
    let mut k = rng.range(n);
    for i in 0..count {
        if fits(i) {
            if k == 0 {
                return Some(i as u8);
            }
            k -= 1;
        }
    }
    None
}

/// The next room's type: the first room is a street; a shop when one is due; now and then the city's rare event (R2b,
/// [`event_due`]); else streets, arenas (never two in a row) and rest stops by the rules' weights. A type with no rooms in the
/// content is never picked.
fn next_kind(p: &mut Planner, rc: &RunContent, here: &dyn Fn(usize) -> bool, event: bool) -> u8 {
    if p.count == 0 {
        return ROOM_STREET;
    }
    // (R2d: only the types some room keeping to this deck has.)
    let has = |k: u8| (0..rc.rooms.len()).any(|i| rc.rooms[i].kind == k && here(i));
    p.since_shop = p.since_shop.saturating_add(1);
    if p.since_shop >= p.shop_due && has(ROOM_SHOP) {
        p.since_shop = 0;
        p.shop_due = draw_shop_due(p, &rc.rules);
        return ROOM_SHOP;
    }
    if event {
        return ROOM_EVENT;
    }
    let r = &rc.rules;
    let weight = |k: u8| -> u32 {
        if !has(k) {
            return 0;
        }
        u32::from(match k {
            ROOM_ARENA if p.last_kind == ROOM_ARENA => 0,
            ROOM_REST if p.last_kind == ROOM_REST => 0,
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
    let mut roll = p.rng.range(total);
    for k in kinds {
        let w = weight(k);
        if roll < w {
            return k;
        }
        roll -= w;
    }
    ROOM_STREET
}

/// A room drawn by the deck plan (a climb, a drop or a deck fork) counts toward the next shop, as any other room does (R2d).
fn count_room(p: &mut Planner) {
    p.since_shop = p.since_shop.saturating_add(1);
}

/// Whether the room being laid is the city's rare event (R2b, Punch Quest's egg): never before room `event_from`, at most one in
/// each city, and otherwise `event_base` per mille plus `event_step` per mille for every room laid since the last one (the pity
/// counter: rare, but never too rare).
fn event_due(p: &mut Planner, rc: &RunContent) -> bool {
    let r = &rc.rules;
    if p.count < u32::from(r.event_from) || !rc.rooms.iter().any(|x| x.kind == ROOM_EVENT) {
        return false;
    }
    let city = city_of(rc, p.count) + 1;
    if p.event_city == city {
        return false;
    }
    let chance = u32::from(r.event_base) + u32::from(r.event_step) * u32::from(p.event_pity);
    if p.rng.range(1000) < chance.min(1000) {
        p.event_pity = 0;
        p.event_city = city;
        true
    } else {
        p.event_pity = p.event_pity.saturating_add(1);
        false
    }
}

/// The next room: the first one to begin with, then (R2d) the deck plan: when the run has kept to its deck for its stretch, a deck
/// fork (`deck_fork_chance` percent of the time) or a new deck the plan picks; while the run is not on the plan's deck, a room that
/// climbs or drops toward it. Otherwise a room that keeps to the deck, of the chosen type from the pool of the room's difficulty
/// tier (R2b: `Room::min_tier` to `max_tier`), never the first room again, never a room laid only as a fork's road
/// (`Room::paired`), nor the last room laid while another of that type will do. A street is a fork `fork_chance` percent of the time
/// (when the content has forks).
fn next_room(p: &mut Planner, rc: &RunContent, no_fork: bool) -> u8 {
    if p.count == 0 {
        return 0;
    }
    let tier = tier_of(rc, p.count);
    let count = rc.rooms.len().min(255);
    let (entry, y) = (deck_at(rc, p.y), p.y);
    // (R2e: one split in view at a time: a road's line plans no branching fork.)
    let drawn =
        |i: usize| i != 0 && !rc.rooms[i].paired && !(no_fork && rc.rooms[i].branch_high != 0);
    let in_tier = |i: usize| rc.rooms[i].min_tier <= tier && tier <= rc.rooms[i].max_tier;
    let last = usize::from(p.last_room);
    let stays = |i: usize| {
        drawn(i)
            && leads_to(rc, &rc.rooms[i], entry, y, entry)
            && !is_deck_fork(rc, &rc.rooms[i], entry)
    };
    // (A shop due comes first, where the deck has one: the shops keep their spacing.)
    let shop_now = p.since_shop.saturating_add(1) >= p.shop_due
        && (0..count).any(|i| stays(i) && rc.rooms[i].kind == ROOM_SHOP);
    // (Then the city's rare event, when its roll comes up: it is rolled for every room, as before the decks.)
    let event_now = !shop_now
        && (0..count).any(|i| stays(i) && rc.rooms[i].kind == ROOM_EVENT)
        && event_due(p, rc);
    let planned = !shop_now && !event_now;
    // The deck plan: a stretch kept, then a choice.
    if planned && entry == p.plan_deck && p.plan_left == 0 {
        let fork = |i: usize| {
            let r = &rc.rooms[i];
            drawn(i)
                && is_deck_fork(rc, r, entry)
                && exit_deck(r, false, entry).is_some_and(|o| {
                    let high = y + r.exit_y + r.lift;
                    leads_to(rc, r, entry, y, o) && in_band(rc, deck_at(rc, high), high)
                })
        };
        if p.rng.range(100) < u32::from(rc.rules.deck_fork_chance) {
            let found = if (0..count).any(|i| fork(i) && in_tier(i)) {
                pick(&mut p.rng, count, |i| fork(i) && in_tier(i))
            } else {
                pick(&mut p.rng, count, fork)
            };
            if let Some(i) = found {
                count_room(p);
                return i;
            }
        }
        p.plan_deck = draw_deck(p, rc, entry);
        p.plan_left = draw_stretch(p, &rc.rules);
    }
    if planned && entry != p.plan_deck {
        let goal = p.plan_deck;
        let toward = |i: usize| {
            let r = &rc.rooms[i];
            drawn(i)
                && r.branch_high == 0
                && mirrors(r).iter().any(|&m| {
                    exit_deck(r, m, entry).is_some_and(|o| {
                        o != entry
                            && (o > entry) == (goal > entry)
                            && o.abs_diff(entry) <= goal.abs_diff(entry)
                            && in_band(rc, o, y + room_exit(r, m))
                    })
                })
        };
        let choices: [&dyn Fn(usize) -> bool; 3] = [
            &|i| toward(i) && in_tier(i) && i != last,
            &|i| toward(i) && in_tier(i),
            &toward,
        ];
        if let Some(fits) = choices.into_iter().find(|t| (0..count).any(t)) {
            if let Some(i) = pick(&mut p.rng, count, fits) {
                count_room(p);
                return i;
            }
        }
        // (Nothing here leads there: keep to this deck.)
        p.plan_deck = entry;
    }
    p.plan_left = p.plan_left.saturating_sub(1);
    let kind = next_kind(p, rc, &stays, event_now);
    let usable = |i: usize| stays(i) && rc.rooms[i].kind == kind;
    let pool = |i: usize| usable(i) && in_tier(i);
    let forks = (1..count).any(|i| pool(i) && rc.rooms[i].fork);
    let plain = (1..count).any(|i| pool(i) && !rc.rooms[i].fork);
    // (A street draws whether to fork; other types take any room of theirs.)
    let want_fork = if kind == ROOM_STREET && forks && plain {
        Some(p.rng.range(100) < u32::from(rc.rules.fork_chance))
    } else {
        None
    };
    let forked = |i: usize| want_fork.is_none_or(|w| rc.rooms[i].fork == w);
    // (The best choices first: the right type, tier and fork-ness, not the last room laid; then letting go of the last room, the
    // fork-ness and the tier; then any room keeping to the deck; the start room only if nothing else will do.)
    let tiers: [&dyn Fn(usize) -> bool; 6] = [
        &|i| pool(i) && forked(i) && i != last,
        &|i| pool(i) && forked(i),
        &|i| pool(i) && i != last,
        &pool,
        &usable,
        &stays,
    ];
    let Some(fits) = tiers.into_iter().find(|t| (0..count).any(t)) else {
        return 0;
    };
    pick(&mut p.rng, count, fits).unwrap_or(0)
}

/// Plans the next room on `p`'s line (R2e): the room [`next_room`] draws and the mirror it is laid as (R2d's rule: a room that
/// changes deck goes toward the plan's deck, any other keeps to its deck, its ground ending in the band), with its decks and the
/// number it will be laid as. A branching fork leaves `p` at its start (its roads are planned by [`fork_roads`]); any other room
/// moves `p` on past it.
fn plan_one(p: &mut Planner, rc: &RunContent, no_fork: bool) -> Planned {
    let id = next_room(p, rc, no_fork);
    let room = &rc.rooms[usize::from(id).min(rc.rooms.len() - 1)];
    let y0 = p.y;
    let mut mirror = room.mirror && p.rng.range(2) == 1;
    let entry = deck_at(rc, y0);
    if room.mirror {
        let goal = p.plan_deck;
        let good = |m: bool| {
            exit_deck(room, m, entry).is_some_and(|o| {
                let toward = if room.deck_change == 0 {
                    o == entry
                } else {
                    o != entry && (o > entry) == (goal > entry)
                };
                toward && in_band(rc, o, y0 + room_exit(room, m))
            })
        };
        if !good(mirror) && good(!mirror) {
            mirror = !mirror;
        }
    }
    let planned = Planned {
        live: true,
        room: id,
        mirror,
        deck_in: entry,
        deck_out: deck_at(rc, y0 + room_exit(room, mirror)),
        seq: p.count,
    };
    p.last_room = id;
    p.last_kind = room.kind;
    if room.branch_high == 0 {
        p.count = p.count.wrapping_add(1);
        p.y = y0 + room_exit(room, mirror);
    }
    planned
}

/// A branching fork `fork` has been planned on the main line (R2e): its two roads (laid together once it is the current room) and
/// a line of its own after each, from a copy of the main planner (its own stream, drawn from the main one; after a deck fork, each
/// road's deck for a new stretch, as R2d did when a road was taken).
fn fork_roads(run: &mut Run, rc: &RunContent, fork: Planned) {
    let room = &rc.rooms[usize::from(fork.room).min(rc.rooms.len() - 1)];
    let y_end = run.planner.y + room_exit(room, false);
    for b in 0..2 {
        let id = if b == 0 {
            room.branch_high
        } else {
            room.branch_low
        };
        let road = &rc.rooms[usize::from(id).min(rc.rooms.len() - 1)];
        let y0 = y_end + if b == 0 { room.lift } else { Fx::ZERO };
        let entry = if road.paired && road.deck_change == 0 {
            deck_at(rc, y0 + road.exit_y)
        } else {
            deck_at(rc, y0)
        };
        run.roads[b] = Planned {
            live: true,
            room: id,
            mirror: false,
            deck_in: entry,
            deck_out: deck_at(rc, y0 + road.exit_y),
            seq: fork.seq.wrapping_add(1 + b as u32),
        };
        let mut q = run.planner;
        q.rng = Rng::new(run.planner.rng.next_u64());
        q.count = fork.seq.wrapping_add(3);
        q.y = y0 + road.exit_y;
        q.last_room = id;
        q.last_kind = road.kind;
        run.branch_planner[b] = q;
        run.branch_plan[b] = [Planned::default(); PLAN];
    }
    if run.roads[0].deck_out != run.roads[1].deck_out {
        for b in 0..2 {
            let q = &mut run.branch_planner[b];
            q.plan_deck = run.roads[b].deck_out;
            q.plan_left = draw_stretch(q, &rc.rules);
        }
    }
    run.forked = true;
}

/// How many rooms of a line are planned.
fn planned_len(line: &[Planned; PLAN]) -> usize {
    line.iter().take_while(|p| p.live).count()
}

/// Rooms laid ahead of the current one on the main line (none when the current room is a branching fork: its roads are the lines').
fn laid_ahead(run: &Run) -> usize {
    if run.branches().is_some() {
        0
    } else {
        run.ahead().1
    }
}

/// Plans ahead (R2e, F4): the main line up to `map_ahead` rooms past the current one (rooms laid ahead count), stopping at a
/// branching fork; after one, each road's line up to the same depth. Deterministic, from the planners' own streams: what the map
/// shows is what comes.
fn fill_plan(run: &mut Run, rc: &RunContent) {
    if rc.rooms.is_empty() {
        return;
    }
    let h = usize::from(rc.rules.map_ahead).clamp(1, PLAN);
    let laid = laid_ahead(run);
    let mut len = planned_len(&run.plan);
    while !run.forked && laid + len < h && len < PLAN {
        let next = plan_one(&mut run.planner, rc, false);
        run.plan[len] = next;
        len += 1;
        if rc.rooms[usize::from(next.room).min(rc.rooms.len() - 1)].branch_high != 0 {
            fork_roads(run, rc, next);
        }
    }
    if run.forked {
        let fork_depth = laid + len;
        for b in 0..2 {
            let mut n = planned_len(&run.branch_plan[b]);
            while fork_depth + 2 + n <= h && n < PLAN {
                let next = plan_one(&mut run.branch_planner[b], rc, true);
                run.branch_plan[b][n] = next;
                n += 1;
            }
        }
    }
}

/// The road of a branching fork taken (R2e): its line becomes the main one.
fn take_line(run: &mut Run, b: usize) {
    run.planner = run.branch_planner[b];
    run.plan = run.branch_plan[b];
    run.forked = false;
    run.roads = [Planned::default(); 2];
    run.branch_planner = [Planner::INACTIVE; 2];
    run.branch_plan = [[Planned::default(); PLAN]; 2];
}

/// One room on the map ahead (R2e, F4): how far ahead (1 the next), on which line (0 the main one, 1 a branching fork's high
/// road's, 2 its low road's), which room and how laid, the number it is or will be, and whether it is laid already.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapNode {
    pub depth: u8,
    pub line: u8,
    pub room: u8,
    pub mirror: bool,
    pub deck_in: u8,
    pub deck_out: u8,
    pub seq: u32,
    pub laid: bool,
}

/// The map ahead (R2e, F4): the rooms laid ahead and planned, nearest first, a branching fork's two roads and their lines.
pub fn map_ahead(run: &Run) -> Vec<MapNode> {
    let mut out = Vec::new();
    let node = |depth: usize, line: u8, p: &Planned, laid: bool| MapNode {
        depth: depth.min(255) as u8,
        line,
        room: p.room,
        mirror: p.mirror,
        deck_in: p.deck_in,
        deck_out: p.deck_out,
        seq: p.seq,
        laid,
    };
    let of_slot = |s: &RoomSlot| Planned {
        live: true,
        room: s.room,
        mirror: s.mirror,
        deck_in: s.deck_in,
        deck_out: s.deck_out,
        seq: s.seq,
    };
    if !run.active {
        return out;
    }
    let mut depth = 0;
    if let Some((high, low)) = run.branches() {
        out.push(node(1, 1, &of_slot(&high), true));
        out.push(node(1, 2, &of_slot(&low), true));
        depth = 1;
    } else {
        let (ahead, n) = run.ahead();
        for s in &ahead[..n] {
            depth += 1;
            out.push(node(depth, 0, &of_slot(s), true));
        }
        for p in run.plan.iter().take_while(|p| p.live) {
            depth += 1;
            out.push(node(depth, 0, p, false));
        }
        if !run.forked {
            return out;
        }
        for (b, road) in run.roads.iter().enumerate() {
            out.push(node(depth + 1, b as u8 + 1, road, false));
        }
        depth += 1;
    }
    for b in 0..2 {
        for (k, p) in run.branch_plan[b].iter().take_while(|p| p.live).enumerate() {
            out.push(node(depth + 1 + k, b as u8 + 1, p, false));
        }
    }
    out
}

/// Lays the next room (false if there is no room for it in the ring, or none may come yet). After a branching fork its two roads
/// come together, once the fork is the current room (the alley `Room::lift` higher, over the low road's room), and nothing is laid
/// past them until the player takes one ([`enter_rooms`]).
fn place_room(run: &mut Run, rc: &RunContent) -> bool {
    if rc.rooms.is_empty() || run.branches().is_some() {
        return false;
    }
    let free = run.slots.iter().filter(|s| !s.live).count();
    if let Some(last) = run
        .slots
        .iter()
        .find(|s| s.live && s.seq.wrapping_add(1) == run.placed)
        .copied()
    {
        let room = &rc.rooms[usize::from(last.room).min(rc.rooms.len() - 1)];
        if room.branch_high != 0 {
            if last.seq != run.room || free < 2 {
                return false;
            }
            let (x, y) = (last.end, last.y_end);
            let roads = run.roads;
            place_slot(run, rc, roads[0].room, x, y + room.lift, false);
            place_slot(run, rc, roads[1].room, x, y, false);
            fill_plan(run, rc);
            return true;
        }
    }
    if free == 0 {
        return false;
    }
    // (R2e: the next planned room; the plan is filled ahead again after it.)
    fill_plan(run, rc);
    let next = run.plan[0];
    if !next.live {
        return false;
    }
    run.plan.copy_within(1.., 0);
    run.plan[PLAN - 1] = Planned::default();
    let laid = place_slot(run, rc, next.room, run.next_x, run.next_y, next.mirror);
    fill_plan(run, rc);
    laid
}

/// Lays room `id` at `(x0, y0)` (its left edge and its ground there) in a free slot, drawing its variant, its grunts (the budget,
/// [`assign_kinds`]) and laying its pools, props and event.
fn place_slot(run: &mut Run, rc: &RunContent, id: u8, x0: Fx, y0: Fx, mirror: bool) -> bool {
    let Some(index) = run.slots.iter().position(|s| !s.live) else {
        return false;
    };
    let room = &rc.rooms[usize::from(id).min(rc.rooms.len() - 1)];
    // The variant: mirrored as planned (R2e: `plan_one`), the optional parts, the pieces.
    // (A fork's road keeps to the deck it leaves on: a treasure alley's catwalk may start in another deck's band.)
    let entry = if room.paired && room.deck_change == 0 {
        deck_at(rc, y0 + room.exit_y)
    } else {
        deck_at(rc, y0)
    };
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
    let seq = run.placed;
    let mut slot = RoomSlot {
        live: true,
        room: id,
        seq,
        x: x0,
        end: x0 + room.width,
        y: y0,
        y_end: y0 + room_exit(room, mirror),
        mirror,
        parts,
        pieces,
        spawned: 0,
        coins: 0,
        kinds: [SPOT_ANY; ROOM_SPAWNS],
        tier: tier_of(rc, seq),
        lo: y0.min(y0 + room_exit(room, mirror)),
        hi: y0.max(y0 + room_exit(room, mirror)),
        deck_in: entry,
        deck_out: deck_at(rc, y0 + room_exit(room, mirror)),
        lid: false,
        emerged_at: 0,
        skill_off: false,
        find: 0,
        find_item: NO_ITEM,
        found: false,
    };
    slot.skill_off = room.platforms.iter().any(|f| f.skilled)
        && !skilled_offered(&rc.rules, room, run.reach_jump, run.reach_recovery);
    // (R3, F8: a find on a skilled route, a treasure alley or a fork's road.)
    items::roll_find(run, rc, &mut slot);
    let mut lo = slot.lo;
    for_each_footing(rc, &slot, |p| {
        lo = lo.min(p.platform.y.min(p.platform.top_right()));
    });
    slot.lo = lo;
    // Its grunts: the budget fills the spots; a spot left empty (or not in this variant) counts as spawned.
    slot.kinds = assign_kinds(run, rc, &slot);
    for j in 0..ROOM_SPAWNS {
        if slot.kinds[j] == SPOT_ANY {
            slot.spawned |= 1 << j;
        }
    }
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
            seq,
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
                seq,
            };
        }
    });
    let props = &mut run.props;
    for_each_prop(rc, &slot, |_, spot, at| {
        if let Some(p) = props.iter_mut().find(|p| !p.active) {
            *p = Prop {
                active: true,
                kind: spot.kind,
                room: seq,
                pos: at,
                ..Prop::default()
            };
        }
    });
    // The runaway van waits at its mark, its roof a pass-through platform in the pool (moved as it drives).
    if room.event == EVENT_VAN {
        let at = Frame::of(&slot, room).point(room.door);
        if let Some(i) = run.platforms.iter().position(|e| !e.live) {
            run.platforms[i] = PoolPlatform {
                live: true,
                left: at.x - VAN_HALF,
                right: at.x + VAN_HALF,
                y: at.y + VAN_HEIGHT,
                bottom: at.y + VAN_HEIGHT,
                pass_through: true,
                rise: Fx::ZERO,
                seq,
            };
            run.van = Van {
                mode: VAN_WAITING,
                room: seq,
                x: at.x,
                y: at.y,
                platform: i as i8,
                ..Van::default()
            };
        }
    }
    run.kinds_log = run.kinds_log << 4 | u64::from(room.kind & 7 | u8::from(room.fork) << 3);
    run.skipped_log <<= 1;
    run.placed = run.placed.wrapping_add(1);
    run.next_x = slot.end;
    run.next_y = slot.y_end;
    run.layout = run.layout.wrapping_add(1);
    true
}

/// A room's grunts (R2b, Dead Cells' budget): its tier's budget (`Tier::budget` points for every 16 units of its width; an arena
/// `RunRules::arena_budget` percent of that) is spent on its spawn spots, taken in a seeded order, each given a kind its tier allows
/// whose cost still fits and whose placement rules the spot meets: high ground for `PLACE_HIGH`, a flat platform at least
/// `flat_room` wide for `PLACE_FLAT`, a platform no other `PLACE_ALONE` grunt stands on for `PLACE_ALONE`. A spot inside a piece
/// laid over it, or with nothing under it in this variant, stays empty. An arena always gets at least one grunt. A spot naming a kind
/// keeps it.
pub fn assign_kinds(run: &mut Run, rc: &RunContent, slot: &RoomSlot) -> [u8; ROOM_SPAWNS] {
    let mut kinds = [SPOT_ANY; ROOM_SPAWNS];
    let room = &rc.rooms[usize::from(slot.room).min(rc.rooms.len() - 1)];
    let Some(tier) = rc.tiers.get(usize::from(slot.tier)) else {
        return kinds;
    };
    let rules = &rc.rules;
    let mut feet: Vec<Platform> = Vec::new();
    let mut looks: Vec<u8> = Vec::new();
    for_each_footing(rc, slot, |p| {
        feet.push(p.platform);
        looks.push(p.look);
    });
    let lowest = feet
        .iter()
        .filter(|p| !p.pass_through)
        .map(|p| p.y.min(p.top_right()))
        .min()
        .unwrap_or(slot.y);
    let frame = Frame::of(slot, room);
    let n = room.spawns.len().min(ROOM_SPAWNS);
    // (A seeded order: Fisher-Yates on the run's generator.)
    let mut order = [0usize; ROOM_SPAWNS];
    for (i, o) in order.iter_mut().enumerate() {
        *o = i;
    }
    for i in (1..n).rev() {
        let j = run.rng.range(i as u32 + 1) as usize;
        order.swap(i, j);
    }
    let mut budget =
        (room.width * Fx::from_int(i32::from(tier.budget)) / Fx::from_int(16)).floor_int();
    if room.kind == ROOM_ARENA {
        budget = budget * i32::from(rules.arena_budget) / 100;
    }
    // The spots that can hold a grunt: (spot, its footing, high ground, flat room).
    let mut spots: [(usize, usize, bool, bool); ROOM_SPAWNS] = [(0, 0, false, false); ROOM_SPAWNS];
    let mut count = 0;
    for &j in &order[..n] {
        let s = room.spawns[j];
        if !part_on(slot.parts, s.part) {
            continue;
        }
        let pos = frame.point(Vec2::new(s.x, s.y));
        let support = feet
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                pos.x >= p.left && pos.x <= p.right && (p.top_at(pos.x) - pos.y).abs() <= Fx::HALF
            })
            .max_by_key(|(_, p)| p.top_at(pos.x))
            .map(|(i, _)| i);
        let Some(si) = support else {
            continue;
        };
        let buried = feet.iter().any(|p| {
            !p.pass_through
                && pos.x > p.left
                && pos.x < p.right
                && p.top_at(pos.x) > pos.y + Fx::HALF
        });
        if buried {
            continue;
        }
        if s.kind != SPOT_ANY {
            kinds[j] = s.kind;
            continue;
        }
        let sp = feet[si];
        let high = pos.y - lowest >= rules.high_ground;
        let flat = !sp.sloped() && sp.right - sp.left >= rules.flat_room;
        spots[count] = (j, si, high, flat);
        count += 1;
    }
    let mut alone_on = 0u32;
    // (R2e, F3: a kind is placed more often on the footing it perches on: leapers on catwalks, lobbers on cranes.)
    let odds = |k: usize, si: usize| {
        let m = &rc.mobs[k];
        let look = looks.get(si).copied().unwrap_or(0);
        let perched = look < 64 && m.perch & (1u64 << look) != 0;
        u32::from(m.odds)
            * if perched {
                u32::from(m.perch_odds.max(1))
            } else {
                1
            }
    };
    let fits =
        |k: usize, budget: i32, (_, si, high, flat): (usize, usize, bool, bool), alone_on: u32| {
            let m = &rc.mobs[k];
            k < 8
                && tier.kinds & (1 << k) != 0
                && m.min_tier <= slot.tier
                && m.odds > 0
                && i32::from(m.cost) <= budget
                && (m.place & PLACE_HIGH == 0 || high)
                && (m.place & PLACE_FLAT == 0 || flat)
                && (m.place & PLACE_ALONE == 0 || alone_on & (1 << si.min(31)) == 0)
        };
    let mut any = false;
    for &spot in &spots[..count] {
        let total: u32 = (0..rc.mobs.len())
            .filter(|&k| fits(k, budget, spot, alone_on))
            .map(|k| odds(k, spot.1))
            .sum();
        if total == 0 {
            continue;
        }
        let mut roll = run.rng.range(total);
        for k in 0..rc.mobs.len() {
            if !fits(k, budget, spot, alone_on) {
                continue;
            }
            let w = odds(k, spot.1);
            if roll < w {
                let m = &rc.mobs[k];
                kinds[spot.0] = k as u8;
                budget -= i32::from(m.cost);
                if m.place & PLACE_ALONE != 0 {
                    alone_on |= 1 << spot.1.min(31);
                }
                any = true;
                break;
            }
            roll -= w;
        }
    }
    // An arena needs something to beat: the cheapest kind that fits the first spot that takes one.
    if room.kind == ROOM_ARENA && !any && kinds.iter().all(|&k| k == SPOT_ANY) {
        'found: for &spot in &spots[..count] {
            let cheapest = (0..rc.mobs.len())
                .filter(|&k| fits(k, i32::MAX, spot, alone_on))
                .min_by_key(|&k| rc.mobs[k].cost);
            if let Some(k) = cheapest {
                kinds[spot.0] = k as u8;
                break 'found;
            }
        }
    }
    kinds
}

/// Drops the rooms more than `behind` behind the current one: their platforms (ground joined across the seam is trimmed to the new
/// wall), ledges, mobs and props go, and the left wall moves to the oldest room kept.
fn drop_behind(state: &mut GameState, world: &Content, behind: u8) {
    let run = &mut state.run;
    // (The `behind` nearest rooms behind are kept: a road not taken left a gap in the numbers.)
    let room = run.room;
    let mut dropped = false;
    for i in 0..RING {
        let s = run.slots[i];
        let nearer = run
            .slots
            .iter()
            .filter(|o| o.live && o.seq < room && o.seq > s.seq)
            .count();
        if s.live && s.seq < room && nearer >= usize::from(behind) {
            run.slots[i].live = false;
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
    // (Grunts left behind the new wall are gone, first: what they stood on goes too. R2d.)
    for n in 0..MAX_MOBS {
        if state.run.mobs[n].active && state.fighters[mobs::slot(n)].pos.x < wall {
            mobs::remove(state, world, n);
        }
    }
    let run = &mut state.run;
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
    for p in run.props.iter_mut().filter(|p| p.active) {
        if run.slots.iter().all(|s| !s.live || s.seq != p.room) {
            p.active = false;
        }
    }
    if run.van.mode != VAN_NONE && run.slots.iter().all(|s| !s.live || s.seq != run.van.room) {
        run.van = Van::default();
        run.van.platform = NONE;
    }
    run.floor_x = run.floor_x.max(wall);
    run.layout = run.layout.wrapping_add(1);
    unground_dropped(state);
}

/// Bodies standing on a pool platform that is gone fall (their `platform` index no longer names it).
fn unground_dropped(state: &mut GameState) {
    for f in state.fighters.iter_mut().filter(|f| f.active) {
        if usize::try_from(f.platform)
            .is_ok_and(|i| i < POOL_PLATFORMS && !state.run.platforms[i].live)
        {
            f.platform = NONE;
            crate::fighter::stand_or_fall(f);
        }
    }
}

/// Drops the road of a branching fork the player did not take: the room, everything it laid in the pools (by `seq`), its mobs and
/// props. It is marked on the map as skipped.
fn drop_road(state: &mut GameState, world: &Content, seq: u32) {
    let run = &mut state.run;
    for s in run.slots.iter_mut() {
        if s.live && s.seq == seq {
            s.live = false;
        }
    }
    for (i, p) in run.platforms.iter_mut().enumerate() {
        let stood_on = state
            .fighters
            .iter()
            .any(|f| f.active && f.platform == i as i8);
        if p.live && p.seq == seq && !stood_on {
            *p = PoolPlatform::default();
        }
    }
    for (i, l) in run.ledges.iter_mut().enumerate() {
        if l.live && l.seq == seq && state.ledge_owner[i] == NONE {
            *l = PoolLedge::default();
        }
    }
    for p in run.props.iter_mut().filter(|p| p.active && p.room == seq) {
        p.active = false;
    }
    if let Some(back) = run.placed.checked_sub(seq + 1).filter(|&b| b < 16) {
        run.skipped_log |= 1 << back;
    }
    run.layout = run.layout.wrapping_add(1);
    for n in 0..MAX_MOBS {
        if state.run.mobs[n].active && state.run.mobs[n].room == seq {
            mobs::remove(state, world, n);
        }
    }
    unground_dropped(state);
}

/// The player walking into the next room makes it the current one: the oldest room behind is dropped, the next one ahead laid, and
/// the room does what its type does (an arena shuts its gate and wakes its mobs, a rest stop heals). At a branching fork's end the
/// road the player walks out on decides (R2b): above the middle of the two roads it is the high road's alley, else the low road's
/// room; the other is dropped at once.
fn enter_rooms(state: &mut GameState, world: &Content, rc: &RunContent) {
    let f = state.fighters[PLAYER];
    if !f.active || f.state == S::Respawn {
        return;
    }
    let rules = &rc.rules;
    while !state.run.gate_closed {
        let (ahead, n) = state.run.ahead();
        if n == 0 {
            return;
        }
        if f.pos.x < ahead[0].x + rules.enter_depth {
            return;
        }
        let next = if let Some((high, low)) = state.run.branches() {
            let middle = (high.y + low.y) / Fx::from_int(2);
            let (take, other) = if f.pos.y >= middle {
                (high, low)
            } else {
                (low, high)
            };
            drop_road(state, world, other.seq);
            state.run.next_x = take.end;
            state.run.next_y = take.y_end;
            // (R2e: the road's own planned line goes on; after a deck fork, on that road's deck for a new stretch.)
            take_line(&mut state.run, usize::from(take.seq != high.seq));
            take
        } else {
            ahead[0]
        };
        state.run.room = next.seq;
        let room = &rc.rooms[usize::from(next.room).min(rc.rooms.len() - 1)];
        // (At a branching fork both roads come next: the ring keeps one room fewer behind for them.)
        let behind = if room.branch_high != 0 {
            rules.rooms_behind.min((RING - 3) as u8)
        } else {
            rules.rooms_behind
        };
        // (R2d: a room that changed deck is a gate once left: nothing before it is kept, and a climb closes a lid at its top.)
        let behind = behind.min(back_to_vertical(&state.run));
        drop_behind(state, world, behind);
        close_lids(state, world, rc);
        // (A grudge the shopkeeper bore lifts once the player is a room further on: R2c.)
        if state.run.keeper_grudge != 0 && state.run.room >= state.run.keeper_grudge {
            state.run.keeper_grudge = 0;
        }
        while state.run.ahead().1 < usize::from(rules.rooms_ahead.max(1)) {
            if !place_room(&mut state.run, rc) {
                break;
            }
        }
        match room.kind {
            ROOM_ARENA => {
                let run = &mut state.run;
                run.gate_closed = true;
                run.gate_x = next.end;
                run.layout = run.layout.wrapping_add(1);
                // Everything in it comes out, awake.
                spawn_slot(state, world, rc, next.seq, None);
                for m in state.run.mobs.iter_mut() {
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

/// How many rooms behind the current one reach back to the nearest room behind that changed deck (`u8::MAX` for none).
fn back_to_vertical(run: &Run) -> u8 {
    let mut behind = [RoomSlot::default(); RING];
    let mut n = 0;
    for s in run.slots.iter().filter(|s| s.live && s.seq < run.room) {
        behind[n] = *s;
        n += 1;
    }
    behind[..n].sort_by_key(|s| core::cmp::Reverse(s.seq));
    behind[..n]
        .iter()
        .position(|s| s.vertical())
        .map_or(u8::MAX, |k| k as u8 + 1)
}

/// The vertical gate (R2d): a room behind the current one that climbed to it gets a lid, a solid floor across its whole width at its
/// top, so you cannot go back down it (its back edge on the vertical axis). Grunts left under the lid go.
fn close_lids(state: &mut GameState, world: &Content, rc: &RunContent) {
    let rules = &rc.rules;
    let room = state.run.room;
    for i in 0..RING {
        let s = state.run.slots[i];
        if !s.live || s.seq >= room || s.lid || !s.vertical() {
            continue;
        }
        if s.y_end < s.y {
            close_drop(state, world, rc, i);
            continue;
        }
        let Some(e) = state.run.platforms.iter().position(|p| !p.live) else {
            continue;
        };
        let run = &mut state.run;
        run.platforms[e] = PoolPlatform {
            live: true,
            left: s.x,
            right: s.end,
            y: s.y_end,
            bottom: s.y_end - rules.gate_thickness,
            pass_through: false,
            rise: Fx::ZERO,
            seq: s.seq,
        };
        run.slots[i].lid = true;
        run.layout = run.layout.wrapping_add(1);
        for n in 0..MAX_MOBS {
            let p = state.fighters[mobs::slot(n)].pos;
            if state.run.mobs[n].active && p.x >= s.x && p.x <= s.end && p.y < s.y_end {
                mobs::remove(state, world, n);
            }
        }
    }
}

/// The lid over a drop left behind (R2e, the owner's R2d answer 5: "Yes"): a solid ceiling across its whole width `lid_clear` over
/// its floor, so you cannot go back up past it (a branching fork's deck drop is a fork, not a drop: no lid). Grunts left above it go.
fn close_drop(state: &mut GameState, world: &Content, rc: &RunContent, i: usize) {
    let s = state.run.slots[i];
    let room = &rc.rooms[usize::from(s.room).min(rc.rooms.len() - 1)];
    if room.branch_high != 0 {
        return;
    }
    let Some(e) = state.run.platforms.iter().position(|p| !p.live) else {
        return;
    };
    let rules = &rc.rules;
    let under = s.y_end + rules.lid_clear;
    let run = &mut state.run;
    run.platforms[e] = PoolPlatform {
        live: true,
        left: s.x,
        right: s.end,
        y: under + rules.gate_thickness,
        bottom: under,
        pass_through: false,
        rise: Fx::ZERO,
        seq: s.seq,
    };
    run.slots[i].lid = true;
    run.layout = run.layout.wrapping_add(1);
    for n in 0..MAX_MOBS {
        let p = state.fighters[mobs::slot(n)].pos;
        if state.run.mobs[n].active && p.x >= s.x && p.x <= s.end && p.y > under {
            mobs::remove(state, world, n);
        }
    }
}

/// The height a lidded room's lid closes the blast zone at: a climb's top (below), a drop's lid (above).
fn lid_line(s: &RoomSlot, rules: &RunRules) -> Fx {
    if s.y_end < s.y {
        s.y_end + rules.lid_clear + rules.gate_thickness
    } else {
        s.y_end
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
    let left = run.mobs.iter().any(|m| m.active && m.room == slot.seq);
    if all_out && !left {
        run.gate_closed = false;
        run.cleared = run.cleared.saturating_add(1);
        run.coins = run.coins.saturating_add(u32::from(rc.rules.arena_bonus));
        run.layout = run.layout.wrapping_add(1);
    }
}

/// Mobs from the spawn spots that have come within `spawn_ahead` of the view's right edge, in the current room and those before it,
/// and in the room ahead unless an arena's gate is shut before it or it is one of a fork's two roads not taken yet.
fn spawn_mobs(state: &mut GameState, world: &Content, rc: &RunContent) {
    let run = &state.run;
    let reach = run.cam_x + rc.rules.view_width + rc.rules.spawn_ahead;
    let (ahead, n) = run.ahead();
    let next = (n >= 1 && run.branches().is_none() && !run.gate_closed).then_some(ahead[0].seq);
    let (slots, room, shut) = (run.slots, run.room, run.gate_closed);
    for slot in slots {
        if !slot.live || slot.seq > room && Some(slot.seq) != next {
            continue;
        }
        // (A shut arena's grunts all come, out of reach or not, as grunt slots free up.)
        let reach = (!(shut && slot.seq == room)).then_some(reach);
        spawn_slot(state, world, rc, slot.seq, reach);
    }
}

/// Spawns room `seq`'s spots not yet spawned (those left of `reach`, or all), each the kind the budget gave it, its stamina by the
/// room's tier. A grunt comes out of its door or stairwell first (`emerge_frames`), standing on the ground under its spot.
fn spawn_slot(
    state: &mut GameState,
    world: &Content,
    rc: &RunContent,
    seq: u32,
    reach: Option<Fx>,
) {
    let run = &state.run;
    let Some(s) = run.slots.iter().position(|s| s.live && s.seq == seq) else {
        return;
    };
    let slot = run.slots[s];
    let room = &rc.rooms[usize::from(slot.room).min(rc.rooms.len() - 1)];
    let frame = Frame::of(&slot, room);
    // (R2e, the owner's note A: at most the current tier's cap of grunts alive at once, and one out of a door at a time in a room,
    // `emerge_gap` frames apart. A spot past either waits its turn.)
    let cap = usize::from(mobs::tier_row(rc, tier_of(rc, run.room)).cap.max(1)).min(MAX_MOBS);
    let gap = u32::from(rc.rules.emerge_gap);
    for (j, marker) in room.spawns.iter().enumerate().take(ROOM_SPAWNS) {
        let bit = 1u8 << j;
        let pos = frame.point(Vec2::new(marker.x, marker.y));
        if state.run.slots[s].spawned & bit != 0 || reach.is_some_and(|r| pos.x >= r) {
            continue;
        }
        let kind_id = slot.kinds[j];
        if rc.mobs.get(usize::from(kind_id)).is_none() {
            state.run.slots[s].spawned |= bit;
            continue;
        }
        let alive = state.run.mobs.iter().filter(|m| m.active).count();
        let last = state.run.slots[s].emerged_at;
        if alive >= cap || last != 0 && state.frame.wrapping_add(1).wrapping_sub(last) < gap {
            return;
        }
        let mut pos = pos;
        // (Its floor: the highest pool entry under its spot, within half a unit.)
        let mut platform = NONE;
        let mut best: Option<(usize, Fx)> = None;
        for (i, p) in state.run.platforms.iter().enumerate() {
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
        // (With every grunt slot taken, the spot waits for one to free up.)
        if mobs::spawn(state, world, rc, kind_id, pos, platform, slot.tier, seq).is_some() {
            state.run.slots[s].spawned |= bit;
            state.run.slots[s].emerged_at = state.frame.wrapping_add(1).max(1);
        }
    }
}

/// Coins the player walks (or flies) through: picked up, paying `coin_value` each.
fn collect_coins(state: &mut GameState, rc: &RunContent) {
    let f = state.fighters[PLAYER];
    if !f.active || f.state == S::Respawn {
        return;
    }
    let body = f.pos + Vec2::new(Fx::ZERO, Fx::ONE);
    // (R3: the coin magnet reaches further.)
    let reach = items::coin_reach(&state.run, rc);
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
    // Its height (R2c): the player's feet plus a lift, outside a dead zone, inside the blast zone's span (as the game's camera
    // did by itself before; the grunts' blast zones follow it).
    if f.active && f.state != S::Respawn {
        let (bottom, top) = run.blast_y(rules);
        // (R2d: in a room that changes deck it follows faster.)
        let ease = if run.slot_of(run.room).is_some_and(|s| s.vertical()) {
            rules.cam_y_ease_tall
        } else {
            rules.cam_y_ease
        };
        let want = (f.pos.y + rules.cam_lift)
            .min(top - rules.cam_ceiling)
            .max(bottom + rules.cam_floor);
        let d = want - run.cam_y;
        if d.abs() > rules.cam_dead {
            let goal = if d > Fx::ZERO {
                want - rules.cam_dead
            } else {
                want + rules.cam_dead
            };
            let step = ((goal - run.cam_y) * ease)
                .abs()
                .max(Fx::from_ratio(1, 64))
                .min(rules.cam_max_speed)
                .min((goal - run.cam_y).abs());
            run.cam_y = if goal > run.cam_y {
                run.cam_y + step
            } else {
                run.cam_y - step
            };
        }
    }
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

/// A hit taken (the percent rose) or a life lost breaks the combo. R3: with steady combo, the first hit taken in a combo does not
/// (its forgiving hit comes back once the combo has broken).
fn settle_combo(state: &mut GameState) {
    let f = state.fighters[PLAYER];
    let run = &mut state.run;
    let hit = f.percent > run.last_percent;
    let lost = f.stocks < run.last_stocks || !f.active;
    if hit && !lost && run.combo > 0 && run.guard && items::steady(run) {
        run.guard = false;
        run.saves = run.saves.wrapping_add(1);
    } else if hit || lost {
        run.combo = 0;
        run.guard = items::steady(run);
    }
    run.last_percent = f.percent;
    run.last_stocks = f.stocks;
}

/// One more hit or kill in the combo.
fn count_combo(run: &mut Run) {
    run.combo = run.combo.saturating_add(1);
    run.best_combo = run.best_combo.max(run.combo);
}

/// A kill: the coins and the tally. R2e (F3): the kind's coins times `air` (a juggle kept off the ground), plus `bonus` (a direction
/// bonus), all times the combo multiplier (counted with this kill). Returns the coins paid.
fn pay_kill(run: &mut Run, rc: &RunContent, kind: u8, air: u32, bonus: u32) -> u32 {
    count_combo(run);
    run.kills = run.kills.saturating_add(1);
    let coins = rc.mobs.get(usize::from(kind)).map_or(0, |k| k.coins);
    let paid = (u32::from(coins) * air.max(1) + bonus) * run.multiplier(&rc.rules);
    run.coins = run.coins.saturating_add(paid);
    paid
}

/// Calls out a bonus (R2e: the HUD shows each once, by `Run::callouts`).
fn call_out(run: &mut Run, what: u8, at: Vec2, coins: u32) {
    run.callout = what;
    run.callouts = run.callouts.wrapping_add(1);
    run.callout_at = at;
    run.callout_coins = coins.min(u32::from(u16::MAX)) as u16;
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
    // (R3: the bell on the counter, `items::BELL_SPOT`, where nearer than any ware.)
    let bell = (x - rc.interior.bell).abs();
    match ware_near(rc, x) {
        Some(i) if (rc.interior.pedestals[i].x - x).abs() <= bell => Some(Some(i)),
        _ if bell <= reach => Some(Some(items::BELL_SPOT)),
        w => w.map(Some),
    }
}

/// The ware whose pedestal is nearest `x` (interior coordinates), within reach.
fn ware_near(rc: &RunContent, x: Fx) -> Option<usize> {
    let wares = items::ware_spots(rc);
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
        // (R3: a new shop's pedestals are stocked as you first walk in.)
        items::stock_shop(run, rc, seq);
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
        // (Then it sulks, and sells nothing until the player has gone on into a room further than the shop: R2c.)
        run.keeper_grudge = run.room.max(run.shop_room) + 1;
    }
    let run = &mut state.run;
    if run.keeper_timer > 0 {
        run.keeper_timer -= 1;
        if run.keeper_timer == 0 {
            run.keeper_mood = KEEPER_IDLE;
        }
    }
    if run.keeper_grudge != 0 && run.keeper_mood != KEEPER_DIZZY {
        // (Its sulk lasts as long as the grudge.)
        run.keeper_mood = KEEPER_HMPH;
        run.keeper_timer = 0;
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
        Some(Some(items::BELL_SPOT)) => {
            items::reroll(state, rc);
        }
        Some(Some(i)) => {
            let note = buy(state, rc, i);
            let run = &mut state.run;
            if run.keeper_mood != KEEPER_DIZZY && run.keeper_mood != KEEPER_HMPH {
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

/// Buys ware spot `i` of the shop the player is in (R3: [`items::buy`]).
pub fn buy(state: &mut GameState, rc: &RunContent, i: usize) -> u8 {
    items::buy(state, rc, i)
}

impl StateHash for Mob {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        h.write_u8(self.kind);
        h.write_u8(self.mode);
        h.write_u16(self.timer);
        h.write_u8(self.move_slot);
        h.write_u8(self.step);
        h.write_u16(self.credit);
        self.last_percent.hash_into(h);
        h.write_u32(self.room);
        h.write_u8(self.tier);
        h.write_bool(self.air);
        h.write_u8(self.juggle);
        h.write_u16(self.bowled);
        h.write_u8(self.burn);
    }
}

impl StateHash for Run {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        // (Versus: an inactive run is never touched; R2d.)
        if !self.active {
            return;
        }
        h.write_bool(self.over);
        self.rng.hash_into(h);
        self.cam_x.hash_into(h);
        self.floor_x.hash_into(h);
        self.start_x.hash_into(h);
        self.best_x.hash_into(h);
        self.next_x.hash_into(h);
        self.next_y.hash_into(h);
        h.write_u32(self.placed);
        h.write_u64(self.kinds_log);
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
            for k in s.kinds {
                h.write_u8(k);
            }
            h.write_u8(s.tier);
            s.lo.hash_into(h);
            s.hi.hash_into(h);
            h.write_u8(s.deck_in);
            h.write_u8(s.deck_out);
            h.write_bool(s.lid);
            h.write_u32(s.emerged_at);
            h.write_bool(s.skill_off);
            h.write_u8(s.find);
            h.write_u8(s.find_item);
            h.write_bool(s.found);
        }
        for p in &self.platforms {
            h.write_bool(p.live);
            p.left.hash_into(h);
            p.right.hash_into(h);
            p.y.hash_into(h);
            p.bottom.hash_into(h);
            h.write_bool(p.pass_through);
            p.rise.hash_into(h);
            h.write_u32(p.seq);
        }
        for l in &self.ledges {
            h.write_bool(l.live);
            l.x.hash_into(h);
            l.y.hash_into(h);
            h.write_i8(l.side);
            h.write_u32(l.seq);
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
        h.write_u32(self.keeper_grudge);
        for p in &self.props {
            h.write_bool(p.active);
            h.write_u8(p.kind);
            h.write_u32(p.room);
            h.write_u8(p.mode);
            h.write_u16(p.timer);
            p.pos.hash_into(h);
            p.vel.hash_into(h);
            h.write_u16(p.hit_mobs);
            h.write_bool(p.hit_player);
        }
        h.write_u16(self.chests);
        let v = &self.van;
        h.write_u8(v.mode);
        h.write_u32(v.room);
        v.x.hash_into(h);
        v.y.hash_into(h);
        v.speed.hash_into(h);
        h.write_u16(v.timer);
        h.write_i8(v.platform);
        h.write_bool(v.riding);
        h.write_u16(v.hits);
        h.write_u16(v.struck);
        h.write_u16(self.skipped_log);
        self.cam_y.hash_into(h);
        h.write_u16(self.kos);
        self.ko_at.hash_into(h);
        self.planner.hash_into(h);
        for p in &self.plan {
            p.hash_into(h);
        }
        h.write_bool(self.forked);
        for p in &self.roads {
            p.hash_into(h);
        }
        for b in 0..2 {
            self.branch_planner[b].hash_into(h);
            for p in &self.branch_plan[b] {
                p.hash_into(h);
            }
        }
        self.reach_jump.hash_into(h);
        self.reach_recovery.hash_into(h);
        h.write_u8(self.callout);
        h.write_u16(self.callouts);
        self.callout_at.hash_into(h);
        h.write_u16(self.callout_coins);
        h.write_u16(self.sky_kos);
        h.write_u16(self.spike_kos);
        h.write_u16(self.bowls);
        h.write_u16(self.air_kos);
        for v in self.shop_stock {
            h.write_u8(v);
        }
        h.write_u8(self.shop_rerolls);
        self.shop_rng.hash_into(h);
        h.write_u64(self.seed);
        for v in self.owned {
            h.write_u8(v);
        }
        h.write_u8(self.slot);
        h.write_i8(self.bomb);
        h.write_bool(self.bomb_thrown);
        h.write_u16(self.wind);
        h.write_bool(self.guard);
        h.write_u16(self.orb_timer);
        h.write_u8(self.base_fighter);
        h.write_u8(self.orb_body);
        h.write_u8(self.orb_move);
        h.write_u16(self.pops);
        h.write_u16(self.saves);
        h.write_u16(self.steals);
        h.write_u16(self.finds);
        h.write_u8(self.find_last);
        h.write_u32(self.life_city);
    }
}
