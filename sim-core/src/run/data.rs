//! The endless run's content (R1, sim 120; rooms R1b, sim 122): the run's rules, its hand-made rooms, its mob kinds and the shop's
//! wares. Immutable and hashed, like [`crate::Content`]; [`RunContent::placeholder`] is the built-in set. Checked by
//! `sim_content::check_run`.
//!
//! Units are the versus game's: world units (a fighter is about two tall), frames (60 a second), percent.

use crate::content::{Ledge, Platform};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::moves::{self, Hitbox};
use crate::vec2::Vec2;
use crate::SIM_VERSION;

/// At most this many platforms (solid blocks and pass-through platforms) in one room.
pub const ROOM_PLATFORMS: usize = 6;
/// At most this many ledges in one room.
pub const ROOM_LEDGES: usize = 2;
/// At most this many mob spawn markers in one room (a room's `spawned` bits are a `u8`).
pub const ROOM_SPAWNS: usize = 8;
/// The narrowest a room may be (world units, about one screen). Two of them must cover the view (`sim_content::check_run`).
pub const MIN_ROOM_WIDTH: i32 = 28;
/// The widest a room may be (about two screens).
pub const MAX_ROOM_WIDTH: i32 = 64;

/// What drives a mob ([`MobKind::behavior`]).
pub const MOB_WALKER: u8 = 0;
pub const MOB_FLYER: u8 = 1;

/// Room types ([`Room::kind`]).
/// A street: plain running with a few mobs (the most common).
pub const ROOM_STREET: u8 = 0;
/// An arena: its exit (a gate on its right edge) closes when you enter it and opens once every mob of the room is beaten.
pub const ROOM_ARENA: u8 = 1;
/// A shop: a storefront whose door (`Room::door`) you enter by pressing up in front of it. No mobs.
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

/// Where a mob appears in a room: its kind (index into [`RunContent::mobs`]) and its feet, in the room's own coordinates (x from
/// the room's left edge, y from the run's ground line).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobSpawn {
    pub kind: u8,
    pub x: Fx,
    pub y: Fx,
}

/// One hand-made room of the run. Geometry is in the room's own coordinates: x from 0 (its left edge) to `width`, y from the run's
/// ground line (0). Solid ground that touches the room's left or right edge at the ground line joins the neighbouring room's into
/// one block, so a fighter walks, dashes, rolls and wavedashes across the seam as on one floor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// A label for the tools and the editors (not hashed).
    pub name: String,
    /// `ROOM_STREET`, `ROOM_ARENA`, `ROOM_SHOP` or `ROOM_REST`.
    pub kind: u8,
    pub width: Fx,
    pub platforms: Vec<Platform>,
    pub ledges: Vec<Ledge>,
    pub spawns: Vec<MobSpawn>,
    /// A shop's door: where to stand (feet) to go in. Unused in other rooms.
    pub door: Vec2,
}

/// A kind of mob: its body, its movement and its one attack. The behaviour itself is a small state machine in Rust for now
/// (`run::mobs`); the numbers are here. (Moving the behaviour to scripts is planned for R2.)
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
    /// The blast zone: this far beyond the view's right edge, and these heights (from the ground line) top and bottom. The left
    /// side is the gate: a wall at the left edge of the oldest room kept.
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
    /// How near the shop's door (sideways) the fighter must stand to go in.
    pub door_reach: Fx,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunContent {
    pub rules: RunRules,
    /// The rooms. The first is where every run starts (a street with no mobs); after it the generator picks a type, then one room
    /// of that type (`RunRules`'s odds and shop spacing).
    pub rooms: Vec<Room>,
    pub mobs: Vec<MobKind>,
    /// What every shop sells.
    pub items: Vec<ShopItem>,
}

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

fn tenths(n: i32) -> Fx {
    Fx::from_ratio(n, 10)
}

/// How deep the run's solid blocks reach (well below the blast zone, so nothing goes under them).
pub const BLOCK_BOTTOM: i32 = -24;

fn block(left: i32, right: i32, top_tenths: i32) -> Platform {
    Platform {
        left: int(left),
        right: int(right),
        y: tenths(top_tenths),
        bottom: int(BLOCK_BOTTOM),
        pass_through: false,
    }
}

fn shelf(left_tenths: i32, right_tenths: i32, y_tenths: i32) -> Platform {
    Platform {
        left: tenths(left_tenths),
        right: tenths(right_tenths),
        y: tenths(y_tenths),
        bottom: tenths(y_tenths),
        pass_through: true,
    }
}

fn ledge(x: i32, y_tenths: i32, side: i8) -> Ledge {
    Ledge {
        x: int(x),
        y: tenths(y_tenths),
        side,
    }
}

fn spawn(kind: u8, x: i32, y_tenths: i32) -> MobSpawn {
    MobSpawn {
        kind,
        x: int(x),
        y: tenths(y_tenths),
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

impl RunContent {
    /// The built-in run (R1b): placeholder rooms of city street and rooftops (a safe start, six streets, two arenas, a shop and a
    /// rest stop), a walker (kind 0) and a flyer (kind 1), and a shop selling a heal and an extra life.
    pub fn placeholder() -> RunContent {
        let room = |name: &str, kind: u8, width: i32, platforms, ledges, spawns| Room {
            name: name.to_string(),
            kind,
            width: int(width),
            platforms,
            ledges,
            spawns,
            door: Vec2::ZERO,
        };
        let mut shop = room(
            "shop",
            ROOM_SHOP,
            32,
            // The storefront's awning over the door.
            vec![block(0, 32, 0), shelf(120, 200, 40)],
            vec![],
            vec![],
        );
        shop.door = Vec2::new(int(16), Fx::ZERO);
        RunContent {
            rules: RunRules::standard(),
            rooms: vec![
                // Where every run begins: a quiet street with two awnings.
                room(
                    "start",
                    ROOM_STREET,
                    32,
                    vec![block(0, 32, 0), shelf(60, 120, 36), shelf(200, 260, 36)],
                    vec![],
                    vec![],
                ),
                // A street with a walker at its far end.
                room(
                    "street",
                    ROOM_STREET,
                    32,
                    vec![block(0, 32, 0), shelf(60, 120, 36), shelf(200, 260, 36)],
                    vec![],
                    vec![spawn(0, 26, 0)],
                ),
                // A gap between two roofs with a stepping stone over it; a flyer waits above.
                room(
                    "gap",
                    ROOM_STREET,
                    30,
                    vec![block(0, 10, 0), block(16, 30, 0), shelf(115, 145, 45)],
                    vec![ledge(10, 0, 1), ledge(16, 0, -1)],
                    vec![spawn(1, 13, 70), spawn(0, 24, 0)],
                ),
                // A taller building to climb over, with a walker on its roof and a flyer beyond.
                room(
                    "step",
                    ROOM_STREET,
                    32,
                    vec![block(0, 8, 0), block(8, 20, 25), block(20, 32, 0)],
                    vec![ledge(8, 25, -1), ledge(20, 25, 1)],
                    vec![spawn(0, 14, 25), spawn(1, 27, 80)],
                ),
                // Fire escapes stacked over the street: room to fight in the air.
                room(
                    "escapes",
                    ROOM_STREET,
                    30,
                    vec![
                        block(0, 30, 0),
                        shelf(40, 100, 36),
                        shelf(120, 180, 66),
                        shelf(200, 260, 36),
                    ],
                    vec![],
                    vec![spawn(0, 8, 0), spawn(1, 15, 100), spawn(0, 24, 0)],
                ),
                // A wide gap with a ledge to catch below its near edge, then a long roof with an awning.
                room(
                    "drop",
                    ROOM_STREET,
                    34,
                    vec![
                        block(0, 9, 0),
                        shelf(95, 135, -20),
                        block(14, 34, 0),
                        shelf(160, 220, 36),
                    ],
                    vec![ledge(9, 0, 1), ledge(14, 0, -1)],
                    vec![spawn(1, 12, 60), spawn(0, 20, 0), spawn(0, 28, 0)],
                ),
                // A long avenue (about two screens) under three awnings.
                room(
                    "avenue",
                    ROOM_STREET,
                    48,
                    vec![
                        block(0, 48, 0),
                        shelf(60, 120, 36),
                        shelf(200, 260, 60),
                        shelf(340, 400, 36),
                    ],
                    vec![],
                    vec![spawn(0, 14, 0), spawn(1, 24, 80), spawn(0, 40, 0)],
                ),
                // An arena: a yard under fire escapes, four mobs.
                room(
                    "yard",
                    ROOM_ARENA,
                    32,
                    vec![
                        block(0, 32, 0),
                        shelf(50, 110, 36),
                        shelf(130, 190, 66),
                        shelf(210, 270, 36),
                    ],
                    vec![],
                    vec![
                        spawn(0, 12, 0),
                        spawn(0, 22, 0),
                        spawn(1, 17, 90),
                        spawn(0, 28, 0),
                    ],
                ),
                // An arena around a raised roof, two walkers and two flyers.
                room(
                    "roofs",
                    ROOM_ARENA,
                    36,
                    vec![block(0, 14, 0), block(14, 22, 20), block(22, 36, 0)],
                    vec![ledge(14, 20, -1), ledge(22, 20, 1)],
                    vec![
                        spawn(0, 8, 0),
                        spawn(1, 18, 90),
                        spawn(0, 28, 0),
                        spawn(1, 31, 80),
                    ],
                ),
                shop,
                // A rest stop: a rooftop garden with a bench.
                room(
                    "garden",
                    ROOM_REST,
                    30,
                    vec![block(0, 30, 0), shelf(100, 200, 36)],
                    vec![],
                    vec![],
                ),
            ],
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
        }
    }

    /// Hash of the sim version plus everything here but the names.
    pub fn hash(&self) -> u64 {
        let mut h = StateHasher::new();
        h.write_u16(SIM_VERSION);
        self.rules.hash_into(&mut h);
        h.write_u32(self.rooms.len() as u32);
        for c in &self.rooms {
            c.hash_into(&mut h);
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
        h.finish()
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
        ] {
            h.write_u8(v);
        }
        h.write_u16(self.arena_bonus);
    }
}

impl StateHash for Room {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.kind);
        self.width.hash_into(h);
        h.write_u32(self.platforms.len() as u32);
        for p in &self.platforms {
            p.left.hash_into(h);
            p.right.hash_into(h);
            p.y.hash_into(h);
            p.bottom.hash_into(h);
            h.write_bool(p.pass_through);
        }
        h.write_u32(self.ledges.len() as u32);
        for l in &self.ledges {
            l.x.hash_into(h);
            l.y.hash_into(h);
            h.write_i8(l.side);
        }
        h.write_u32(self.spawns.len() as u32);
        for s in &self.spawns {
            h.write_u8(s.kind);
            s.x.hash_into(h);
            s.y.hash_into(h);
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
