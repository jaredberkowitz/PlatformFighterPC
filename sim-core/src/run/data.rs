//! The endless run's content (R1, sim 118): the run's rules, its hand-made stage chunks and its mob kinds. Immutable and hashed, like
//! [`crate::Content`]; [`RunContent::placeholder`] is the built-in set (five placeholder chunks, a walker and a flyer). Checked by
//! `sim_content::check_run`.
//!
//! Units are the versus game's: world units (a fighter is about two tall), frames (60 a second), percent.

use crate::content::{Ledge, Platform};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::moves::{self, Hitbox};
use crate::SIM_VERSION;

/// At most this many platforms (solid blocks and pass-through platforms) in one chunk.
pub const CHUNK_PLATFORMS: usize = 6;
/// At most this many ledges in one chunk.
pub const CHUNK_LEDGES: usize = 2;
/// At most this many mob spawn markers in one chunk (a chunk's `spawned` bits are a `u8`).
pub const CHUNK_SPAWNS: usize = 8;
/// The narrowest a chunk may be (world units). With the rules' view width this bounds how many chunks are ever live at once, and so
/// the stage's platform and ledge pools (see `run::RING`).
pub const MIN_CHUNK_WIDTH: i32 = 28;
/// The widest a chunk may be.
pub const MAX_CHUNK_WIDTH: i32 = 96;

/// What drives a mob ([`MobKind::behavior`]).
pub const MOB_WALKER: u8 = 0;
pub const MOB_FLYER: u8 = 1;

/// Where a mob appears in a chunk: its kind (index into [`RunContent::mobs`]) and its feet, in the chunk's own coordinates (x from
/// the chunk's left edge, y from the run's ground line).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobSpawn {
    pub kind: u8,
    pub x: Fx,
    pub y: Fx,
}

/// One hand-made piece of the run's stage. Geometry is in the chunk's own coordinates: x from 0 (its left edge) to `width`, y from
/// the run's ground line (0). Solid ground that touches the chunk's left or right edge at the same height as the neighbouring chunk's
/// joins it into one block, so a fighter walks, dashes, rolls and wavedashes across the seam as on one floor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    /// A label for the tools and the editors (not hashed).
    pub name: String,
    pub width: Fx,
    pub platforms: Vec<Platform>,
    pub ledges: Vec<Ledge>,
    pub spawns: Vec<MobSpawn>,
}

/// A kind of mob: its body, its movement and its one attack. The behaviour itself is a small state machine in Rust for R1
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

/// The run's rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunRules {
    /// Lives at the start (the fighter's stocks).
    pub lives: u8,
    /// How much of the world the camera shows across (world units). The sim needs it: the screen's left edge pushes, and the blast
    /// zones and spawning follow the view.
    pub view_width: Fx,
    /// Where the camera holds the player when following them right: this far across the view from its left edge (0 to 1).
    pub follow: Fx,
    /// The camera's own forward creep (world units a frame), at the start; it grows by `creep_growth` every 100 units travelled up to
    /// `creep_max`. 0 makes it a pure follow camera: the player sets the pace.
    pub creep: Fx,
    pub creep_growth: Fx,
    pub creep_max: Fx,
    /// The fastest the camera moves (following a fighter launched forward, for one).
    pub cam_max_speed: Fx,
    /// The screen's left edge keeps a fighter at least this far inside it, pushing at up to `push_speed` a frame.
    pub push_margin: Fx,
    pub push_speed: Fx,
    /// Pinned against a wall by the left edge: once the edge is this far past where it wants the fighter, the fighter is crushed
    /// and loses a life.
    pub crush_depth: Fx,
    /// The blast zone: this far beyond the view's left and right edges, and these heights (from the ground line) top and bottom.
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunContent {
    pub rules: RunRules,
    /// The first chunk is where every run starts; then all of them come round in a seeded order (each lap a new shuffle).
    pub chunks: Vec<Chunk>,
    pub mobs: Vec<MobKind>,
}

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

fn tenths(n: i32) -> Fx {
    Fx::from_ratio(n, 10)
}

/// How deep the run's solid blocks reach (well below the blast zone, so nothing goes under them).
const BLOCK_BOTTOM: i32 = -24;

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
    }
}

impl RunRules {
    pub fn standard() -> RunRules {
        RunRules {
            lives: 3,
            view_width: int(30),
            follow: Fx::from_ratio(2, 5),
            creep: Fx::from_ratio(1, 50),
            creep_growth: Fx::from_ratio(1, 400),
            creep_max: Fx::from_ratio(1, 16),
            cam_max_speed: Fx::from_ratio(1, 2),
            push_margin: tenths(8),
            push_speed: Fx::from_ratio(7, 20),
            crush_depth: int(1),
            blast_margin: int(6),
            blast_top: int(22),
            blast_bottom: int(-14),
            spawn_ahead: int(2),
            respawn_at: Fx::from_ratio(7, 20),
            combo_step: 5,
            combo_max: 5,
            credit_frames: 180,
            burst_frames: 20,
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
    /// The built-in run (R1): placeholder chunks of city street and rooftops, a walker (kind 0) and a flyer (kind 1).
    pub fn placeholder() -> RunContent {
        let chunk = |name: &str, width: i32, platforms, ledges, spawns| Chunk {
            name: name.to_string(),
            width: int(width),
            platforms,
            ledges,
            spawns,
        };
        RunContent {
            rules: RunRules::standard(),
            chunks: vec![
                // Where every run begins: a long street with two awnings and one walker at the far end.
                chunk(
                    "street",
                    32,
                    vec![block(0, 32, 0), shelf(60, 120, 36), shelf(200, 260, 36)],
                    vec![],
                    vec![spawn(0, 26, 0)],
                ),
                // A gap between two roofs with a stepping stone over it; a flyer waits above.
                chunk(
                    "gap",
                    30,
                    vec![block(0, 10, 0), block(16, 30, 0), shelf(115, 145, 45)],
                    vec![ledge(10, 0, 1), ledge(16, 0, -1)],
                    vec![spawn(1, 13, 70), spawn(0, 24, 0)],
                ),
                // A taller building to climb over, with a walker on its roof and a flyer beyond.
                chunk(
                    "step",
                    32,
                    vec![block(0, 8, 0), block(8, 20, 25), block(20, 32, 0)],
                    vec![ledge(8, 25, -1), ledge(20, 25, 1)],
                    vec![spawn(0, 14, 25), spawn(1, 27, 80)],
                ),
                // Fire escapes stacked over the street: room to fight in the air.
                chunk(
                    "escapes",
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
                chunk(
                    "drop",
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
            ],
            mobs: vec![MobKind::walker(), MobKind::flyer()],
        }
    }

    /// Hash of the sim version plus everything here but the chunks' names.
    pub fn hash(&self) -> u64 {
        let mut h = StateHasher::new();
        h.write_u16(SIM_VERSION);
        self.rules.hash_into(&mut h);
        h.write_u32(self.chunks.len() as u32);
        for c in &self.chunks {
            c.hash_into(&mut h);
        }
        h.write_u32(self.mobs.len() as u32);
        for m in &self.mobs {
            m.hash_into(&mut h);
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
            self.creep,
            self.creep_growth,
            self.creep_max,
            self.cam_max_speed,
            self.push_margin,
            self.push_speed,
            self.crush_depth,
            self.blast_margin,
            self.blast_top,
            self.blast_bottom,
            self.spawn_ahead,
            self.respawn_at,
        ] {
            v.hash_into(h);
        }
        h.write_u16(self.combo_step);
        h.write_u8(self.combo_max);
        h.write_u16(self.credit_frames);
        h.write_u8(self.burst_frames);
    }
}

impl StateHash for Chunk {
    fn hash_into(&self, h: &mut StateHasher) {
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
