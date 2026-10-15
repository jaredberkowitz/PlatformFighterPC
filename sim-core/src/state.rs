//! Everything that changes during a match lives in [`GameState`], and nothing else does.
//!
//! The state is plain-old-data and `Copy`, so a snapshot is a memcpy: `let saved = state;`
//! and a restore is `state = saved;`.

use crate::collision;
use crate::content::{Content, MAX_LEDGES};
use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};
use crate::input::Input;
use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::{FIGHTER_VARS, MAX_BODIES, MAX_FIGHTERS, MAX_SCRIPT_VARS, PROJECTILE_VARS};

/// Frames of input kept per fighter. Must cover the longest buffer window in `FighterParams`.
pub const HISTORY_LEN: usize = 12;
pub const NONE: i8 = -1;
/// [`GameState::winner`] while the match is still being played.
pub const PLAYING: i8 = -1;
/// [`GameState::winner`] when nobody won (the last fighters were knocked out together, or the time ran out level).
pub const DRAW: i8 = -2;
/// Fixed capacity of the projectile pool.
pub const MAX_PROJECTILES: usize = 16;
/// Fixed capacity of the item pool (E4, sim 111).
pub const MAX_ITEMS: usize = 8;
/// `Fighter::link_kind`: the autolink (angle 367).
pub const LINK_AUTOLINK: u8 = 0;
/// `Fighter::link_kind`: the vortex (angle 366, E28).
pub const LINK_VORTEX: u8 = 1;
/// `Fighter::link_kind`: a position vector (E34): carried straight to `link_point` over `posvec` frames.
pub const LINK_POSVEC: u8 = 2;
/// `Fighter::link_kind`: half the attacker's momentum (angle 365, sim 123).
pub const LINK_HALF: u8 = 3;
/// Frames a vortex hit (366) pulls its victim toward the hitbox centre (the reference's 5).
pub const VORTEX_FRAMES: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FighterState {
    Idle = 0,
    Walk,
    Run,
    Dash,
    Turn,
    Crouch,
    JumpSquat,
    Airborne,
    AirDodge,
    Landing,
    WaveLand,
    Shield,
    ShieldDrop,
    LedgeHang,
    LedgeGetUp,
    LedgeAttack,
    /// Special fall: no air jumps or air dodge, can still drift and grab ledges.
    Helpless,
    /// Performing a move; `Fighter::move_id` says which and `state_frame` is the move frame.
    Attack,
    /// Launched or flinching from a hit.
    Hitstun,
    /// Rolling away from a shield, intangible for part of it.
    Roll,
    /// Dodging in place from a shield, intangible for part of it.
    SpotDodge,
    /// Stunned after a shield broke (launched up, then helpless on the ground).
    ShieldBreak,
    /// Holding another fighter (can pummel or throw).
    Grabbing,
    /// Being held.
    Grabbed,
    /// Lying on the ground after landing in hitstun without a tech. Vulnerable until it chooses a get-up.
    Knockdown,
    /// Standing up from a knockdown, intangible for the first part.
    GetUp,
    /// Knocked back by a clank (or a grab parry): no action until `Fighter::lag` frames have passed.
    Rebound,
    /// A wall or ceiling tech: held against the surface for a moment, intangible, then free (a jump makes it a wall-tech jump).
    WallTech,
    /// Back after a knock-out, standing on the revival platform above the stage: invincible until it moves or the platform goes.
    Respawn,
    /// Jumping from a ledge: rising, intangible at first, unable to act until `ledge_jump_frames` have passed.
    LedgeJump,
    /// Letting go of the shield: no action for `shield_release_frames`.
    ShieldRelease,
    /// Buried in the ground by a bury hit (E5): no action until `Fighter::bury_timer` runs out (mashing shortens it), then out with a
    /// little lag, or launched by a knockback kept from a weaker hit taken meanwhile.
    Buried,
    /// Carrying a held fighter (E7, sim 98: the reference's cargo hold; see `moves::Carry`): walking, jumping (`Fighter::carry_jumps`)
    /// and falling with it, on the ground or in the air, until a throw, the hold's clock or a hit ends it.
    Carrying,
    /// Shut in an egg (E8, sim 107: see `moves::Encase`): no action until `Fighter::encase_timer` runs out (mashing shortens it, hits
    /// shorten it); it falls, lands and is hit as an egg, then breaks out into the air.
    Encased,
    /// Picking an item up from the ground (E4, sim 111): no action for the ruleset's `item_pick_frames`; the item is in hand from the
    /// first.
    ItemPickup,
    /// Throwing or dropping the item in hand (E4): `Fighter::item_throw` says which throw; the item leaves on the body's release frame
    /// for it and the action ends on the ruleset's total for it, on the ground or in the air.
    ItemThrow,
    /// Hanging from a tether and being reeled to the ledge it caught (E56, sim 123: see `moves::Tether`; `Fighter::tether_ledge`, the
    /// move that caught it in `move_id`). Does not hold the ledge: someone else can grab it meanwhile.
    Tether,
}

impl StateHash for FighterState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(*self as u8);
    }
}

/// How many recent moves the stale-move queue remembers.
pub const STALE_QUEUE: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fighter {
    /// Feet position (bottom centre of the ECB).
    pub pos: Vec2,
    pub vel: Vec2,
    /// Normalised direction of the current or last air dodge.
    pub dodge_dir: Vec2,
    /// Index into `Content::fighters`.
    pub char_id: u8,
    /// -1 faces left, +1 faces right.
    pub facing: i8,
    pub state: FighterState,
    /// Frames since entering `state`.
    pub state_frame: u16,
    /// Index of the platform being stood on, or [`NONE`].
    pub platform: i8,
    pub air_jumps_left: u8,
    pub air_dodge_used: bool,
    /// Frames of lag for the current `Landing` state (normal vs helpless landing).
    pub lag: u8,
    pub fast_fall: bool,
    /// Frames left of the fast opening of a full hop (see `FighterParams::hop_burst_frames`).
    pub hop_boost: u8,
    /// Frames since the current dash began (255 when not dashing or running); a reversal is only a dash dance while it is small.
    pub dash_age: u8,
    /// Frames a reversed dash still has to stand before it accelerates.
    pub dash_wait: u8,
    /// The horizontal speed the fighter had as it turned around out of a dash or run (0 for a turn from a walk or standing). A jump
    /// in the first frames of a dash turnaround keeps it (a reverse aerial rush); in a turn it marks a run's skid.
    pub turn_speed: Fx,
    /// While non-zero, pass-through platforms are ignored (set by shield drop).
    pub platform_ignore: u8,
    /// Ledge currently held, or [`NONE`].
    pub ledge: i8,
    pub ledge_invuln: u8,
    /// Ledge grabs since last landing or being hit: only the first gives ledge intangibility, the options' intangibility shrinks
    /// on later ones, and there is a limit (`FighterParams::ledge_grab_limit`).
    pub ledge_grab_count: u8,
    /// Frames since last standing on the ground (saturating); the first ledge grab's intangibility grows with it.
    pub airtime: u16,
    /// Buttons whose latest press has already started an action (bit per button), so one press is one action however long the
    /// input buffer is. A new press of the button clears its bit.
    pub buffer_used: u16,
    /// In the air from a short hop (cleared by landing, a midair jump, a ledge or a hit): aerials hit for `Ruleset::short_hop_damage`.
    pub short_hop: bool,
    /// The direction (-1 or 1) of the run or dash a jump was taken out of, 0 for any other jump: landing from it with the stick still
    /// held that way goes straight back into the run (`exit_landing`). Kept through the air and the landing, cleared by anything else
    /// done on the ground, a hit or a ledge.
    pub run_jump: i8,
    pub ledge_cooldown: u8,
    // ---- Combat ----
    /// Damage taken, in percent.
    pub percent: Fx,
    pub stocks: u8,
    /// Whether this fighter is in the match. A fighter that is not (the unused slots of a two-player match) is
    /// not updated, cannot be hit, grabbed or targeted, and never acts.
    pub active: bool,
    /// Frames of freeze after hitting or being hit. Nothing about the fighter advances during hitlag.
    pub hitlag: u8,
    /// Frames of hitstun remaining once hitlag ends.
    pub hitstun: u16,
    /// A hit is waiting to launch at the end of hitlag (so DI can bend it).
    pub launch_pending: bool,
    pub launch_kb: Fx,
    /// The pending (or current) launch is from set knockback, which keeps the plain launch physics: no speed-up and no shared fall
    /// speed for a vertical launch (the reference's rule for set-knockback hits).
    pub launch_set: bool,
    /// The pending (or current) launch is from a hit on the previous reference version's rules (E40, `Hitbox::rules`, sim 113): no
    /// speed-up, the gravity boost at any angle, no shared fall speed for a vertical launch.
    pub launch_older: bool,
    /// World angle of the pending launch, in `trig::Angle` units.
    pub launch_angle: u16,
    /// Current launch velocity, decaying each frame.
    pub kb_vel: Vec2,
    pub tumble: bool,
    /// Index of the move being performed, see `moves::MoveId`.
    pub move_id: u8,
    /// The move's hit history: bit `combat::hit_bit(n, g)` (`n + MAX_BODIES * g`) set means hit group `g` (0 to 7) of this move has
    /// already hit body `n` (sim 129: a grunt is a body too).
    pub hit_mask: u128,
    /// Frames the current smash attack has been charged.
    pub charge: u8,
    /// Shield health; it drains while the shield is up and refills when it is not.
    pub shield_hp: Fx,
    /// Frames of shield stun left after blocking a hit: the shield stays up and the fighter cannot act.
    pub shield_stun: u8,
    /// A shield grab pressed during shield stun (1), then held back for `Ruleset::shield_grab_delay` more frames of shield (2): the
    /// reference's slower post-shieldstun shield grab (sim 109). 0 otherwise.
    pub shield_grab: u8,
    /// Frames until the next survival-DI nudge can happen.
    pub sdi_wait: u8,
    /// Percent of the normal survival-DI distance during the current hitlag (the hit's own multiplier).
    pub sdi_mult: u16,
    /// The pending launch is an autolink (angle 367): when hitlag ends the fighter is sent at `link_vel` (the attacker's velocity
    /// when it hit) plus a share of the way from where it is to `link_point` (where the hitbox was), not along a knockback angle.
    pub link: bool,
    pub link_vel: Vec2,
    pub link_point: Vec2,
    /// Which link the pending `link` launch is: [`LINK_AUTOLINK`] (367) or [`LINK_VORTEX`] (366, E28: sent the way the attacker
    /// moved at the knockback's speed, plus a pull to `link_point` spread over `vortex` frames).
    pub link_kind: u8,
    /// Frames of a vortex pull left (366): each knockback frame moves the fighter a `1 / vortex` share of the way to `link_point`.
    pub vortex: u8,
    /// Frames of a position vector's carry left (E34, `Hitbox::posvec_frames`): set when such a hit lands, then each knockback frame
    /// moves the fighter a fixed step toward `link_point` (worked out at the launch), with no gravity or knockback decay, and the
    /// step stops when the count runs out.
    pub posvec: u8,
    /// The fighter is frozen by a projectile its body's passive shield blocked (E14, `FighterParams::passive_shield`): its hitlag
    /// allows shield SDI, as a raised shield's does. Cleared when the hitlag ends.
    pub shield_sdi: bool,
    /// Being pulled (the pull angle): while hitstun lasts the fighter is drawn toward `link_point` (just in front of whoever pulled
    /// it), see `fighter::pull_frame`.
    pub pull: bool,
    /// A charge kept between uses of a move (the needles a fighter has stored), set and read by move scripts (`store`, `stored`).
    pub stored: u8,
    /// Once-per-airtime flags (bit per flag), set by move scripts (`set_air_flag`) and cleared by landing, catching a ledge or being
    /// hit: what lets a move's boost happen only once until the fighter is back on the ground (the reference's cape stall).
    pub air_flags: u8,
    /// Once-per-airtime marks (bit per mark, sim 119, owner decision 75), set by move scripts (`set_air_mark`) and cleared only by
    /// landing or catching a ledge: unlike `air_flags`, a hit does not clear them (the reference's slower fall for only the first
    /// rocket-barrel charge of an air time). `Move::flag_versions` reads them.
    pub air_marks: u8,
    /// The gauge's powered mode was running when the current attack began (E52, sim 121): which of its `Hitbox::mode` boxes are out
    /// until it ends, even if the mode ends first (the reference's partner moves). Set by `fighter::begin_attack` for a new attack.
    pub move_powered: bool,
    /// Frames since the move's armor last withstood a hit, plus one (E55, sim 121: the guard's release window); 0 if it has not in this
    /// attack (a new attack clears it; a script's `goto` keeps it). The script register `withstood`.
    pub withstood: u8,
    /// The ledge the fighter's tether holds (E56, sim 123), -1 for none: meaningful in `FighterState::Tether` only.
    pub tether_ledge: i8,
    /// Tether catches since the fighter last landed or got up onto the stage (E56: `Tether::limit`; a hit does not reset it).
    pub tether_count: u8,
    /// Where the tether's reel began (E56): the fighter goes from here to the ledge's hanging point.
    pub tether_from: Vec2,
    /// The frame the current move ends on early, 0 for its own end (sim 126, owner decision 84: a blockable grab stopped by a shield
    /// or a reflector ends its move `Move::shield_end` frames later). A new move or a switch clears it (`fighter::begin_attack`).
    pub move_end: u16,
    /// Frames the current move's timeline has been held back by a stale dodge's stretch (sim 126, owner decision 85:
    /// `Move::stale_stretch`): the frames played are `state_frame + move_held`. A new move or a switch clears it.
    pub move_held: u16,
    /// Damage over time on the fighter (E54, sim 121: the reference's curse): frames left (0: none), the frames between its ticks and
    /// since the last one, the damage of each tick and the fighter credited with it.
    pub curse_left: u16,
    pub curse_every: u8,
    pub curse_tick: u8,
    pub curse_damage: Fx,
    pub curse_by: u8,
    /// The wing's points (E57, sim 125: `content::Wing`): above 0 while the fighter has its wing out; the wing goes when they run out.
    pub wing_points: Fx,
    /// The fighter has lost its wing since its last KO (E57): it does not come again until the fighter is knocked out.
    pub wing_spent: bool,
    /// The last fighter whose hit raised this fighter's percent since it last came back (E57: who a KO is credited to), or [`NONE`].
    pub hit_by: i8,
    /// Wall jumps since the fighter last stood on the ground or held a ledge (each is weaker: `Ruleset::wall_jump_decay`).
    pub wall_jumps: u8,
    /// A dash flicked during a grab, pummel or throw (-1 or 1, 0 for none), kept while the stick stays pushed that way and started on
    /// the first frame the fighter can act (see `fighter::update`).
    pub dash_buffer: i8,
    /// A platform drop waiting to happen: frames left (counting the one it falls on) before the fighter leaves the pass-through
    /// platform it flicked down on, 0 for none (see `Ruleset::platform_drop_frames`). A special on those frames is grounded.
    pub drop_pending: u8,
    /// Dodge staling (sim 112, the reference's): how stale this fighter's dodges are (0 fresh to `Ruleset::dodge_stale_max`). Every
    /// roll or spot dodge out of a shield and every air dodge adds one; it loses one `dodge_stale_first` frames after the last dodge
    /// ended and one every `dodge_stale_every` frames after that.
    pub dodge_stale: u8,
    /// The staleness the current dodge was made with (0 for a get-up or tech roll, which neither stale nor are staled).
    pub dodge_level: u8,
    /// Frames since the last dodge ended (or began, while it lasts), for the staleness to wear off.
    pub dodge_clock: u16,
    /// Attack-button presses and releases since the current attack began (the move a button or an action started; follow-ups
    /// reached by `next`, `goto` or a counter keep counting), counted every frame, hitlag included: what a rapid jab's entry rule
    /// reads (the script registers `presses` and `releases`). The press that started the attack (if attack did) is the first.
    pub presses: u8,
    pub releases: u8,
    /// A move switch decided after the fighter's own update (a search box's contact, a command grab's catch): `move id + 1`, started
    /// at the beginning of the next frame's update; 0 for none.
    pub next_move: u8,
    /// The fighter's last movement step was stopped by a wall (or pushed out of one): what the `wall` script register reads.
    pub touched_wall: bool,
    /// The hitstun's first actionable frame used for the launch speed-up (0: no speed-up).
    pub balloon_faf: u8,
    /// Knockback frames of a sped-up launch still to play.
    pub balloon_left: Fx,
    /// Still tumbling after a strong launch's hitstun ran out (the reference's damage fall): the fighter can act, but cannot fast fall
    /// until it does something (any action, a midair jump, a landing).
    pub damage_fall: bool,
    /// This special has already been B-reversed (only once per special).
    pub b_reversed: bool,
    /// How the current attack was aimed as it started: 1 up, -1 down, 0 straight (the reference's angled forward tilts and smashes;
    /// see `Hitbox::aim`).
    pub aim: i8,
    /// The current attack started in the air (it picks the move's aerial height multiplier and intangibility; see `Move::rise_air`).
    pub move_air: bool,
    /// Degrees the current move's motion was angled by the stick (positive toward forward; see `Move::steer_frame`).
    pub steer: i8,
    /// Hits taken in the current combo (it ends when the fighter is free again); survival DI grows with it.
    pub hits_taken: u8,
    /// Frames since the last tech press that counted (255: long ago).
    pub tech_timer: u8,
    /// How many automatic-SDI drifts the pending launch gets (an electric hit gives two).
    pub asdi: u8,
    /// The last moves this fighter connected with, newest first: `move id + 1`, with the top bit set if the hit was shielded; 0 is
    /// empty. Drives stale-move negation; a respawn clears it.
    pub stale: [u8; STALE_QUEUE],
    /// The fighter being held (when grabbing) or holding this one (when grabbed), or [`NONE`].
    pub grab_with: i8,
    /// Frames left before a held fighter breaks free by itself (mashing shortens it).
    pub grab_timer: u16,
    /// Frames left in which this fighter cannot be grabbed (after breaking out of a grab).
    pub grab_immune: u8,
    /// Invulnerable frames remaining (respawn).
    pub invuln: u8,
    /// The current move wants to spawn its projectile this frame (consumed by `step`).
    pub spawn_request: bool,
    /// ...and a script chose where the projectile appears and how fast it goes, instead of the move's defaults.
    pub spawn_custom: bool,
    /// ...at the absolute point `spawn_pos` (`spawn_at`), not an offset from the fighter.
    pub spawn_fixed: bool,
    pub spawn_pos: Vec2,
    pub spawn_vel: Vec2,
    /// ...with this much extra base knockback and this size (percent), from `spawn_scaled` (a charged shot).
    pub spawn_kb_add: u8,
    pub spawn_size: u16,
    /// A second shot asked for on the same frame (E3h, sim 95): a script's second `spawn` / `spawn_at` in one frame fills these instead
    /// of replacing the first (plain size and knockback; `spawn2_fixed` for an absolute point). Consumed by `step` with the first.
    pub spawn2_request: bool,
    pub spawn2_fixed: bool,
    pub spawn2_pos: Vec2,
    pub spawn2_vel: Vec2,
    /// A jab's continuable window past its end (`Move::late_next`, owner decision 58, sim 95): the move a jab press starts instead of a
    /// new jab (its index plus 1; 0 for none), and the frames it still lasts (counted down each frame; any new state clears it).
    pub late_next: u8,
    pub late_left: u8,
    /// Damage the fighter's counter stance caught; its counter-strike move deals this.
    pub counter_damage: Fx,
    /// The current special was started with a flick of the stick (a smash-style input, or the strong button): what the `flick`
    /// script register reads. Set when a special starts.
    pub special_flick: bool,
    /// The current attack's target lock (`lock` in a move script): where the target's chest was when it was found, and whether one was.
    pub lock: Vec2,
    pub locked: bool,
    /// The fighter's knockdown (and the get-up after it) is a trip (a `Hitbox::trip` chance came up, sim 86), with the ruleset's trip
    /// timings instead of a missed tech's.
    pub tripped: bool,
    /// The trip is a soft one (E44, sim 117: a `Hitbox::slip` soft, walking into a lying banana peel): the ruleset's `slip_*` timings.
    pub trip_soft: bool,
    /// Frames before the fighter's move can make another of its tossed items (E47, sim 117: `Toss::wait`, the reference's time before
    /// the next banana peel); counts down every frame. What the `pull_wait` script register reads.
    pub pull_wait: u8,
    /// Frames of the midair jump the fighter is in (sim 89, E33): 1 on the jump's own frame, one more each frame it stays in it (hitlag
    /// excluded, saturating); 0 when it is not in one (anything but `Airborne` clears it). The body's `air_jump_armor` reads it.
    pub air_jump_frame: u8,
    /// Frames left buried (`FighterState::Buried`, E5); mashing takes more off.
    pub bury_timer: u16,
    /// A launch kept for the end of the bury: a hit too weak to free the buried fighter (under `Ruleset::bury_break_kb`) leaves its
    /// knockback and world launch angle here (the last such hit's [E: gap 10]), `bury_set` saying there is one.
    pub bury_kb: Fx,
    pub bury_angle: u16,
    pub bury_set: bool,
    /// Jumps left in the current carry (`FighterState::Carrying`, E7): set from `Carry::jumps` when the carry starts.
    pub carry_jumps: u8,
    /// The hold's clock as it was set when this fighter was caught (`grab::connect`): what a carry measures the hold time already
    /// used from (`grab_start - grab_timer`, mashing included). 0 when not held by an ordinary grab, and once a carry has used it.
    pub grab_start: u16,
    /// Frames left in the egg (`FighterState::Encased`, E8), fixed point: mashing takes 12.6 and the like off. 0 when not in one.
    pub encase_timer: Fx,
    /// Frames since the egg was laid (its growth), saturating.
    pub encase_age: u8,
    /// Which body laid the egg (an index into `Content::fighters`) and with which move: where its `moves::Encase` numbers are read.
    pub encase_char: u8,
    pub encase_move: u8,
    /// The item slot in this fighter's hand (E4, sim 111: `GameState::items`), or [`NONE`]. Kept equal to the item's `holder`.
    pub holding: i8,
    /// What the fighter asks the item system to do this frame (`ORDER_*` bits, set during its update and served after the hits, in
    /// `items::update`); 0 for nothing.
    pub item_order: u8,
    /// The weapon item an `ORDER_PULL` makes (`Move::pull`'s index).
    pub item_spec: u8,
    /// The item throw being made (`THROW_*`, in `FighterState::ItemThrow`), with `THROW_SMASH` for a smash throw.
    pub item_throw: u8,
    /// The body's gauge (`content::Gauge`): 0 to its `max`. A KO empties it (owner decision 9: `Fighter::spawn` starts it at 0).
    pub gauge: Fx,
    /// Frames left of the gauge's powered mode (0: not powered).
    pub gauge_mode: u16,
    /// Held by an ordinary grab out of a `Move::gauge_drain` move: until the hold ends, damage taken drains the gauge.
    pub gauge_drain: bool,
    /// Balloons (E41, sim 113: `moves::Balloons`): how many the fighter carries in its balloon move (0 out of one), the damage the
    /// next one to pop has left, and the helium used, in sixteenths of a frame of flight (0: a full tank; a KO fills it).
    pub balloons: u8,
    pub balloon_hp: Fx,
    pub helium_used: u16,
    /// The pocket (E42, sim 115: `moves::PocketCatch`): the slot of the projectile (or, with `pocket_item`, the item) the fighter has
    /// put away, or [`NONE`]. The thing waits in its pool as `REST_POCKETED` / `ITEM_POCKETED` until it is taken out; a KO loses it.
    pub pocket: i8,
    pub pocket_item: bool,
    /// Where the fighter's feet were (height) when its current move began (also on a switch by `goto`): what `Move::ledge_drop`
    /// measures the fall from.
    pub move_y0: Fx,
    /// Sim 129 (the run's grunts, R2c): the percent of its hits' damage this body deals (0: 100, as every player), and super armor
    /// it has whatever it does (a hit deals its damage and freezes it, but does not launch it). Set only by the run's grunt layer.
    pub damage_scale: u16,
    pub super_armor: bool,
    /// Sim 129: the weight the knockback formula reads for this body instead of its body's (a grunt's, by its kind and tier).
    pub weight: Option<Fx>,
    /// Persistent script variables of the move being performed.
    pub vars: [i32; FIGHTER_VARS],
    /// `history[0]` is this frame's input, `history[1]` the previous frame's, and so on.
    pub history: [Input; HISTORY_LEN],
}

/// A projectile in flight. Slots are reused; `active` marks a live one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Projectile {
    pub active: bool,
    /// Who the hits are credited to (changes when the projectile is reflected).
    pub owner: u8,
    /// The fighter whose weapon holds its hit data (the one that fired it).
    pub origin: u8,
    /// Percent of its normal damage it deals (a reflected projectile hits harder).
    pub power: u8,
    /// The owner's move that fired it (for its hit data).
    pub move_id: u8,
    pub pos: Vec2,
    pub vel: Vec2,
    pub age: u8,
    pub life: u8,
    /// Times it has bounced off the ground (see `ProjectileArc::bounces`).
    pub bounces: u8,
    /// Extra base knockback and size (percent of its hitbox's radius) it was fired with (a charged shot; 0 and 100 otherwise).
    pub kb_add: u8,
    pub size: u16,
    /// Persistent variables of the projectile's script.
    pub vars: [i32; PROJECTILE_VARS],
    /// Who it has hit: bit `combat::hit_bit(fighter, group)`, as a move's `Fighter::hit_mask` (for a piercing or multi-hit one).
    pub hit_mask: u128,
    /// Its age when it last hit each body (the per-target rehit interval counts from it).
    pub hit_age: [u8; MAX_BODIES],
    /// The way its owner faced when it was made (-1 or 1; turned round by a reflection): "forward" for a projectile standing still.
    pub facing: i8,
    /// It has burst (`moves::Burst`): it stands still and hits with the burst's phases; its age counts from the burst.
    pub bursting: bool,
    /// The damage a charge-scaled shot was fired with (E35, `moves::ChargedShot`); 0 for any other (its hit's own damage).
    pub damage: Fx,
    /// Its tool kind (`moves::Tool::kind`, E3d), copied at the spawn: what the per-fighter limit and the `owned` register count.
    pub kind: u8,
    /// [`REST_FLYING`], [`REST_LODGED`] (stuck in a surface: `ProjectileArc::lodge`), [`REST_FALLEN`] (knocked down by a clank:
    /// `Tool::on_clank`) or [`REST_POCKETED`] (put away by a fighter's pocket, E42: it waits, unseen and frozen, until taken out).
    /// Lodged or fallen, it neither hits nor is steered by its script; its age counts from then.
    pub rest: u8,
    /// Sim 115: the stale-move key its hits count under in its owner's queue: the move that fired it, or the pocket that let it out
    /// (E42).
    pub stale: u8,
}

/// An item (E4, sim 111; see `moves::ItemSpec`): held, thrown, lying, stuck or bursting. Slots are reused; `active` marks a live one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub active: bool,
    /// The fighter whose weapon holds its numbers (`Weapon::items[spec]`): who made it, credited for its burst.
    pub origin: u8,
    /// The body of its maker (an index into `Content::fighters`) and its index in that body's weapon's `items`.
    pub maker: u8,
    pub spec: u8,
    /// Its `ItemSpec::kind`, copied when it was made (what `holding` and `owned` read).
    pub kind: u8,
    /// Who its thrown hit is credited to (the last fighter to throw it, or its maker).
    pub owner: u8,
    /// The fighter holding it, or [`NONE`].
    pub holder: i8,
    /// `ITEM_HELD`, `ITEM_THROWN`, `ITEM_FREE`, `ITEM_LODGED` or `ITEM_BLAST`.
    pub rest: u8,
    /// The move that made it (its burst's stale-move key).
    pub move_id: u8,
    /// The middle of its bottom edge.
    pub pos: Vec2,
    pub vel: Vec2,
    /// The platform it lies on, or [`NONE`].
    pub platform: i8,
    /// Frames since it was made, and its life (0: no end); a burst counts its own frames in `age` against its `frames`.
    pub age: u16,
    pub life: u16,
    /// Frames since it was thrown (its `thrown_life`).
    pub flight: u16,
    /// Hit points left (`ItemSpec::hp`).
    pub hp: Fx,
    /// The damage its thrown hit deals (set by the throw).
    pub damage: Fx,
    /// Frames until a commanded burst (`detonate()`), 0 for none.
    pub fuse: u8,
    /// Frozen by a hit it took.
    pub hitlag: u8,
    /// Bit per body its thrown hit or burst has hit.
    pub hit_mask: u16,
    /// For each fighter, the move (`move id + 1`) that last knocked it about (one knock a move; cleared once that fighter is not
    /// attacking).
    pub struck: [u8; MAX_BODIES],
    /// -1 or 1: the way its thrower faced (forward for its hit).
    pub facing: i8,
    /// Sim 113: the direction it was flying when it lodged (an arrow stuck in the ground or a wall), in whole degrees counter-clockwise
    /// from +x (-179 to 180); 0 for an item that never lodged. Read only by the view (`item_angle`), to point the stuck arrow.
    pub angle: i16,
    /// A placed piece (E43, sim 115: `moves::Placed`): the water it has had, the frames left before it grows (0: not growing), and its
    /// cuts (the chopping hits taken from the left in the low nibble, from the right in the high one).
    pub water: u8,
    pub grow_in: u8,
    pub cuts: u8,
    /// Sim 117 (`ItemSpec::uses`): the uses it has spent (a fighter hit while thrown, a fighter tripped while lying, a throw by a
    /// fighter).
    pub used: u8,
}

/// `Item::rest` values.
pub const ITEM_HELD: u8 = 0;
pub const ITEM_THROWN: u8 = 1;
pub const ITEM_FREE: u8 = 2;
pub const ITEM_LODGED: u8 = 3;
pub const ITEM_BLAST: u8 = 4;
/// Put away in a fighter's pocket (E42): unseen, frozen, untouchable, still in play (it counts for its maker's limits).
pub const ITEM_POCKETED: u8 = 5;

impl StateHash for Item {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        h.write_u8(self.origin);
        h.write_u8(self.maker);
        h.write_u8(self.spec);
        h.write_u8(self.kind);
        h.write_u8(self.owner);
        h.write_i8(self.holder);
        h.write_u8(self.rest);
        h.write_u8(self.move_id);
        self.pos.hash_into(h);
        self.vel.hash_into(h);
        h.write_i8(self.platform);
        h.write_u16(self.age);
        h.write_u16(self.life);
        h.write_u16(self.flight);
        self.hp.hash_into(h);
        self.damage.hash_into(h);
        h.write_u8(self.fuse);
        h.write_u8(self.hitlag);
        h.write_u16(self.hit_mask);
        for s in &self.struck {
            h.write_u8(*s);
        }
        h.write_i8(self.facing);
        h.write_u16(self.angle as u16);
        h.write_u8(self.water);
        h.write_u8(self.grow_in);
        h.write_u8(self.cuts);
        h.write_u8(self.used);
    }
}

/// `Projectile::rest` values.
pub const REST_FLYING: u8 = 0;
pub const REST_LODGED: u8 = 1;
pub const REST_FALLEN: u8 = 2;
pub const REST_POCKETED: u8 = 3;

impl StateHash for Projectile {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bool(self.active);
        h.write_u8(self.owner);
        h.write_u8(self.origin);
        h.write_u8(self.power);
        h.write_u8(self.move_id);
        self.pos.hash_into(h);
        self.vel.hash_into(h);
        h.write_u8(self.age);
        h.write_u8(self.life);
        h.write_u8(self.bounces);
        h.write_u8(self.kb_add);
        h.write_u16(self.size);
        for v in &self.vars {
            h.write_i32(*v);
        }
        h.write_u128(self.hit_mask);
        for a in &self.hit_age {
            h.write_u8(*a);
        }
        h.write_i8(self.facing);
        h.write_bool(self.bursting);
        self.damage.hash_into(h);
        h.write_u8(self.kind);
        h.write_u8(self.rest);
        h.write_u8(self.stale);
    }
}

/// How a match is won. Chosen per match (the host decides online), unlike [`crate::content::Ruleset`], which is part of the
/// roster's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchRules {
    /// Stocks each fighter starts with. 0 means unlimited: nobody is ever eliminated (training, free play).
    pub stocks: u8,
    /// Match length in seconds. 0 means no time limit. When it runs out in a stock match the fighter with the most stocks wins; if the
    /// leaders are level, they go to sudden death (`GameState::sudden_death`, sim 114; damage used to break the tie). In free play
    /// (no stocks) the least damage wins, level is a draw.
    pub time_limit: u16,
}

impl Default for MatchRules {
    fn default() -> MatchRules {
        MatchRules {
            stocks: 3,
            time_limit: 0,
        }
    }
}

impl MatchRules {
    pub const MAX_STOCKS: u8 = 9;
    /// Stocks shown (and kept) when the rules say unlimited.
    pub const UNLIMITED_DISPLAY: u8 = 99;

    /// The same rules with every value in range.
    pub fn clamped(self) -> MatchRules {
        MatchRules {
            stocks: self.stocks.min(MatchRules::MAX_STOCKS),
            time_limit: self.time_limit.min(60 * 60),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameState {
    pub frame: u32,
    pub rules: MatchRules,
    /// Bit `n` set means fighter `n` started the match (an eliminated fighter is no longer `active`, but stays here).
    pub roster: u8,
    /// [`PLAYING`] until the match ends, then the winner's index or [`DRAW`]. The simulation stops when it is set.
    pub winner: i8,
    pub rng: Rng,
    /// The players (`MAX_FIGHTERS`), then the run's grunts (`MAX_GRUNTS`, inactive outside a run: sim 129).
    pub fighters: [Fighter; MAX_BODIES],
    /// Which fighter holds each ledge, or [`NONE`].
    pub ledge_owner: [i8; MAX_LEDGES],
    pub projectiles: [Projectile; MAX_PROJECTILES],
    /// The item pool (E4, sim 111).
    pub items: [Item; MAX_ITEMS],
    pub script_vars: [i32; MAX_SCRIPT_VARS],
    /// Sudden death (sim 114, the reference's): the time ran out with the leaders level on stocks, so they play on with one stock
    /// each at 300%, with no clock, until a knock-out decides it.
    pub sudden_death: bool,
    /// The single-player run (sim 120, `crate::run`): its camera, streamed stage, mobs, coins and combo. Inactive in versus.
    pub run: crate::run::Run,
}

impl Fighter {
    pub fn spawn(
        pos: Vec2,
        char_id: u8,
        facing: i8,
        platform: i8,
        air_jumps: u8,
        shield_hp: Fx,
    ) -> Fighter {
        Fighter {
            pos,
            vel: Vec2::ZERO,
            dodge_dir: Vec2::ZERO,
            char_id,
            facing,
            state: if platform == NONE {
                FighterState::Airborne
            } else {
                FighterState::Idle
            },
            state_frame: 0,
            platform,
            air_jumps_left: air_jumps,
            air_dodge_used: false,
            lag: 0,
            fast_fall: false,
            hop_boost: 0,
            dash_age: 255,
            dash_wait: 0,
            turn_speed: Fx::ZERO,
            platform_ignore: 0,
            ledge: NONE,
            ledge_invuln: 0,
            ledge_grab_count: 0,
            airtime: 0,
            buffer_used: 0,
            short_hop: false,
            run_jump: 0,
            ledge_cooldown: 0,
            active: true,
            percent: Fx::ZERO,
            stocks: 3,
            hitlag: 0,
            hitstun: 0,
            launch_pending: false,
            launch_kb: Fx::ZERO,
            launch_set: false,
            launch_older: false,
            launch_angle: 0,
            kb_vel: Vec2::ZERO,
            tumble: false,
            move_id: 0,
            hit_mask: 0,
            charge: 0,
            sdi_mult: 100,
            link: false,
            link_vel: Vec2::ZERO,
            link_point: Vec2::ZERO,
            link_kind: LINK_AUTOLINK,
            vortex: 0,
            posvec: 0,
            shield_sdi: false,
            shield_hp,
            shield_stun: 0,
            shield_grab: 0,
            sdi_wait: 0,
            balloon_faf: 0,
            balloon_left: Fx::ZERO,
            damage_fall: false,
            b_reversed: false,
            aim: 0,
            move_air: false,
            steer: 0,
            hits_taken: 0,
            tech_timer: 255,
            asdi: 0,
            stale: [0; STALE_QUEUE],
            grab_with: NONE,
            grab_timer: 0,
            grab_immune: 0,
            invuln: 0,
            spawn_request: false,
            spawn_custom: false,
            spawn_fixed: false,
            spawn_pos: Vec2::ZERO,
            spawn_vel: Vec2::ZERO,
            spawn_kb_add: 0,
            spawn_size: 100,
            spawn2_request: false,
            spawn2_fixed: false,
            spawn2_pos: Vec2::ZERO,
            spawn2_vel: Vec2::ZERO,
            late_next: 0,
            late_left: 0,
            counter_damage: Fx::ZERO,
            pull: false,
            stored: 0,
            air_flags: 0,
            air_marks: 0,
            move_powered: false,
            withstood: 0,
            tether_ledge: NONE,
            tether_count: 0,
            tether_from: Vec2::ZERO,
            move_end: 0,
            move_held: 0,
            curse_left: 0,
            curse_every: 0,
            curse_tick: 0,
            curse_damage: Fx::ZERO,
            curse_by: 0,
            wing_points: Fx::ZERO,
            wing_spent: false,
            hit_by: NONE,
            wall_jumps: 0,
            dash_buffer: 0,
            drop_pending: 0,
            dodge_stale: 0,
            dodge_level: 0,
            dodge_clock: 0,
            presses: 0,
            releases: 0,
            next_move: 0,
            touched_wall: false,
            special_flick: false,
            lock: Vec2::ZERO,
            locked: false,
            tripped: false,
            trip_soft: false,
            pull_wait: 0,
            air_jump_frame: 0,
            bury_timer: 0,
            bury_kb: Fx::ZERO,
            bury_angle: 0,
            bury_set: false,
            carry_jumps: 0,
            grab_start: 0,
            encase_timer: Fx::ZERO,
            encase_age: 0,
            encase_char: 0,
            encase_move: 0,
            holding: NONE,
            item_order: 0,
            item_spec: 0,
            item_throw: 0,
            gauge: Fx::ZERO,
            gauge_mode: 0,
            gauge_drain: false,
            balloons: 0,
            balloon_hp: Fx::ZERO,
            helium_used: 0,
            pocket: NONE,
            pocket_item: false,
            move_y0: Fx::ZERO,
            damage_scale: 0,
            super_armor: false,
            weight: None,
            vars: [0; FIGHTER_VARS],
            history: [Input::default(); HISTORY_LEN],
        }
    }

    pub fn grounded(&self) -> bool {
        self.platform != NONE
    }
}

impl GameState {
    /// Fresh match: every fighter on its stage spawn point. `char_ids` indexes `content.fighters`.
    pub fn new(content: &Content, seed: u64, char_ids: [u8; MAX_FIGHTERS]) -> GameState {
        GameState::new_with_active(content, seed, char_ids, 0b1111)
    }

    /// Like [`GameState::new`], but only the fighters whose bit is set in `active` take part (bit `n` is
    /// fighter `n`). The others stay where they spawned, untouched and untouchable.
    pub fn new_with_active(
        content: &Content,
        seed: u64,
        char_ids: [u8; MAX_FIGHTERS],
        active: u8,
    ) -> GameState {
        GameState::new_with_rules(content, seed, char_ids, active, MatchRules::default())
    }

    /// Like [`GameState::new_with_active`], with the stocks and time limit of the match chosen.
    pub fn new_with_rules(
        content: &Content,
        seed: u64,
        char_ids: [u8; MAX_FIGHTERS],
        active: u8,
        rules: MatchRules,
    ) -> GameState {
        let rules = rules.clamped();
        let fighters = core::array::from_fn(|i| {
            if i >= MAX_FIGHTERS {
                // (A grunt's slot: empty until a run puts one there.)
                let mut f = Fighter::spawn(Vec2::ZERO, 0, 1, NONE, 0, Fx::ZERO);
                f.active = false;
                f.stocks = 0;
                return f;
            }
            let pos = content.stage.spawns[i];
            let idx = usize::from(char_ids[i]).min(content.fighters.len().saturating_sub(1));
            let mut f = Fighter::spawn(
                pos,
                char_ids[i],
                if i % 2 == 0 { 1 } else { -1 },
                collision::standing_on(&content.stage, pos),
                content.fighters[idx].air_jumps,
                content.rules.shield_max,
            );
            f.active = active >> i & 1 == 1;
            f.stocks = if rules.stocks == 0 {
                MatchRules::UNLIMITED_DISPLAY
            } else {
                rules.stocks
            };
            f
        });
        GameState {
            frame: 0,
            rules,
            roster: active & 0b1111,
            winner: PLAYING,
            rng: Rng::new(seed),
            fighters,
            ledge_owner: [NONE; MAX_LEDGES],
            projectiles: [Projectile::default(); MAX_PROJECTILES],
            items: [Item::default(); MAX_ITEMS],
            script_vars: [0; MAX_SCRIPT_VARS],
            sudden_death: false,
            run: crate::run::Run::INACTIVE,
        }
    }

    /// Per-frame checksum over the whole state.
    pub fn checksum(&self) -> u64 {
        let mut h = StateHasher::new();
        self.hash_into(&mut h);
        h.finish()
    }
}

impl StateHash for Fighter {
    fn hash_into(&self, h: &mut StateHasher) {
        self.pos.hash_into(h);
        self.vel.hash_into(h);
        self.dodge_dir.hash_into(h);
        h.write_u8(self.char_id);
        h.write_i8(self.facing);
        self.state.hash_into(h);
        h.write_u16(self.state_frame);
        h.write_i8(self.platform);
        h.write_u8(self.air_jumps_left);
        h.write_bool(self.air_dodge_used);
        h.write_u8(self.lag);
        h.write_bool(self.fast_fall);
        h.write_u8(self.hop_boost);
        h.write_u8(self.dash_age);
        h.write_u8(self.dash_wait);
        self.turn_speed.hash_into(h);
        h.write_u8(self.platform_ignore);
        h.write_i8(self.ledge);
        h.write_u8(self.ledge_invuln);
        h.write_u8(self.ledge_grab_count);
        h.write_u16(self.airtime);
        h.write_u16(self.buffer_used);
        h.write_bool(self.short_hop);
        h.write_i8(self.run_jump);
        h.write_u8(self.ledge_cooldown);
        h.write_bool(self.active);
        self.percent.hash_into(h);
        h.write_u8(self.stocks);
        h.write_u8(self.hitlag);
        h.write_u16(self.hitstun);
        h.write_bool(self.launch_pending);
        self.launch_kb.hash_into(h);
        h.write_bool(self.launch_set);
        h.write_bool(self.launch_older);
        h.write_u16(self.launch_angle);
        self.kb_vel.hash_into(h);
        h.write_bool(self.tumble);
        h.write_u8(self.move_id);
        h.write_u128(self.hit_mask);
        h.write_u8(self.charge);
        h.write_u16(self.sdi_mult);
        h.write_bool(self.link);
        self.link_vel.hash_into(h);
        self.link_point.hash_into(h);
        h.write_u8(self.link_kind);
        h.write_u8(self.vortex);
        h.write_u8(self.posvec);
        h.write_bool(self.shield_sdi);
        self.shield_hp.hash_into(h);
        h.write_u8(self.shield_stun);
        h.write_u8(self.shield_grab);
        h.write_u8(self.sdi_wait);
        h.write_u8(self.balloon_faf);
        self.balloon_left.hash_into(h);
        h.write_bool(self.damage_fall);
        h.write_bool(self.b_reversed);
        h.write_u8(self.aim as u8);
        h.write_bool(self.move_air);
        h.write_i8(self.steer);
        h.write_u8(self.hits_taken);
        h.write_u8(self.tech_timer);
        h.write_u8(self.asdi);
        for s in self.stale {
            h.write_u8(s);
        }
        h.write_i8(self.grab_with);
        h.write_u16(self.grab_timer);
        h.write_u8(self.grab_immune);
        h.write_u8(self.invuln);
        h.write_bool(self.spawn_request);
        self.counter_damage.hash_into(h);
        h.write_bool(self.pull);
        h.write_u8(self.stored);
        h.write_u8(self.air_flags);
        h.write_u8(self.air_marks);
        h.write_bool(self.move_powered);
        h.write_u8(self.withstood);
        h.write_i8(self.tether_ledge);
        h.write_u8(self.tether_count);
        self.tether_from.hash_into(h);
        h.write_u16(self.move_end);
        h.write_u16(self.move_held);
        h.write_u16(self.curse_left);
        h.write_u8(self.curse_every);
        h.write_u8(self.curse_tick);
        self.curse_damage.hash_into(h);
        h.write_u8(self.curse_by);
        self.wing_points.hash_into(h);
        h.write_bool(self.wing_spent);
        h.write_i8(self.hit_by);
        h.write_u8(self.wall_jumps);
        h.write_i8(self.dash_buffer);
        h.write_u8(self.drop_pending);
        h.write_u8(self.dodge_stale);
        h.write_u8(self.dodge_level);
        h.write_u16(self.dodge_clock);
        h.write_u8(self.presses);
        h.write_u8(self.releases);
        h.write_u8(self.next_move);
        h.write_bool(self.touched_wall);
        h.write_bool(self.special_flick);
        self.lock.hash_into(h);
        h.write_bool(self.locked);
        h.write_bool(self.tripped);
        h.write_bool(self.trip_soft);
        h.write_u8(self.pull_wait);
        h.write_u16(self.bury_timer);
        self.bury_kb.hash_into(h);
        h.write_u16(self.bury_angle);
        h.write_bool(self.bury_set);
        h.write_u8(self.carry_jumps);
        h.write_u16(self.grab_start);
        self.encase_timer.hash_into(h);
        h.write_u8(self.encase_age);
        h.write_u8(self.encase_char);
        h.write_u8(self.encase_move);
        h.write_u8(self.air_jump_frame);
        h.write_bool(self.spawn_custom);
        h.write_bool(self.spawn_fixed);
        self.spawn_pos.hash_into(h);
        self.spawn_vel.hash_into(h);
        h.write_u8(self.spawn_kb_add);
        h.write_u16(self.spawn_size);
        h.write_bool(self.spawn2_request);
        h.write_bool(self.spawn2_fixed);
        self.spawn2_pos.hash_into(h);
        self.spawn2_vel.hash_into(h);
        h.write_u8(self.late_next);
        h.write_u8(self.late_left);
        h.write_i8(self.holding);
        h.write_u8(self.item_order);
        h.write_u8(self.item_spec);
        h.write_u8(self.item_throw);
        self.gauge.hash_into(h);
        h.write_u16(self.gauge_mode);
        h.write_bool(self.gauge_drain);
        h.write_u8(self.balloons);
        self.balloon_hp.hash_into(h);
        h.write_u16(self.helium_used);
        h.write_i8(self.pocket);
        h.write_bool(self.pocket_item);
        self.move_y0.hash_into(h);
        h.write_u16(self.damage_scale);
        h.write_bool(self.super_armor);
        h.write_bool(self.weight.is_some());
        self.weight.unwrap_or(Fx::ZERO).hash_into(h);
        for v in &self.vars {
            h.write_i32(*v);
        }
        for input in &self.history {
            input.hash_into(h);
        }
    }
}

impl StateHash for GameState {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(self.frame);
        h.write_u8(self.rules.stocks);
        h.write_u16(self.rules.time_limit);
        h.write_u8(self.roster);
        h.write_i8(self.winner);
        h.write_bool(self.sudden_death);
        self.rng.hash_into(h);
        for f in &self.fighters {
            f.hash_into(h);
        }
        for o in &self.ledge_owner {
            h.write_i8(*o);
        }
        for p in &self.projectiles {
            p.hash_into(h);
        }
        for it in &self.items {
            it.hash_into(h);
        }
        for v in &self.script_vars {
            h.write_i32(*v);
        }
        self.run.hash_into(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fx;

    fn base() -> GameState {
        GameState::new(&Content::placeholder(), 5, [0, 1, 0, 1])
    }

    #[test]
    fn snapshot_is_a_copy() {
        let mut a = base();
        let saved = a;
        a.frame = 99;
        a.fighters[0].pos.x = Fx::from_int(7);
        assert_ne!(a.checksum(), saved.checksum());
        a = saved;
        assert_eq!(a.checksum(), saved.checksum());
    }

    #[test]
    fn fighters_spawn_standing_on_the_main_stage() {
        for f in &base().fighters {
            assert_eq!(f.platform, 0);
            assert_eq!(f.state, FighterState::Idle);
        }
    }

    /// Every field must feed the checksum, or a desync in it would go undetected.
    #[test]
    fn checksum_sees_every_field() {
        let state = base();
        let c = state.checksum();
        let changed: Vec<(&str, GameState)> = vec![
            ("frame", {
                let mut s = state;
                s.frame += 1;
                s
            }),
            ("rng", {
                let mut s = state;
                s.rng = Rng::new(6);
                s
            }),
            ("ledge_owner", {
                let mut s = state;
                s.ledge_owner[MAX_LEDGES - 1] = 0;
                s
            }),
            ("script_vars", {
                let mut s = state;
                s.script_vars[MAX_SCRIPT_VARS - 1] = 1;
                s
            }),
            ("pos", {
                let mut s = state;
                s.fighters[3].pos.y = Fx::from_raw(1);
                s
            }),
            ("vel", {
                let mut s = state;
                s.fighters[2].vel.y = Fx::from_raw(1);
                s
            }),
            ("dodge_dir", {
                let mut s = state;
                s.fighters[1].dodge_dir.x = Fx::from_raw(1);
                s
            }),
            ("char_id", {
                let mut s = state;
                s.fighters[0].char_id = 1;
                s
            }),
            ("facing", {
                let mut s = state;
                s.fighters[0].facing = -1;
                s
            }),
            ("state", {
                let mut s = state;
                s.fighters[0].state = FighterState::Shield;
                s
            }),
            ("state_frame", {
                let mut s = state;
                s.fighters[0].state_frame = 1;
                s
            }),
            ("platform", {
                let mut s = state;
                s.fighters[0].platform = NONE;
                s
            }),
            ("air_jumps_left", {
                let mut s = state;
                s.fighters[0].air_jumps_left = 0;
                s
            }),
            ("air_dodge_used", {
                let mut s = state;
                s.fighters[0].air_dodge_used = true;
                s
            }),
            ("lag", {
                let mut s = state;
                s.fighters[0].lag = 1;
                s
            }),
            ("fast_fall", {
                let mut s = state;
                s.fighters[0].fast_fall = true;
                s
            }),
            ("hop_boost", {
                let mut s = state;
                s.fighters[0].hop_boost = 1;
                s
            }),
            ("airtime", {
                let mut s = state;
                s.fighters[0].airtime = 7;
                s
            }),
            ("buffer_used", {
                let mut s = state;
                s.fighters[0].buffer_used = 1;
                s
            }),
            ("short_hop", {
                let mut s = state;
                s.fighters[0].short_hop = true;
                s
            }),
            ("run_jump", {
                let mut s = state;
                s.fighters[0].run_jump = 1;
                s
            }),
            ("dash_age", {
                let mut s = state;
                s.fighters[0].dash_age = 1;
                s
            }),
            ("dash_wait", {
                let mut s = state;
                s.fighters[0].dash_wait = 1;
                s
            }),
            ("turn_speed", {
                let mut s = state;
                s.fighters[0].turn_speed = Fx::from_raw(1);
                s
            }),
            ("platform_ignore", {
                let mut s = state;
                s.fighters[0].platform_ignore = 1;
                s
            }),
            ("ledge", {
                let mut s = state;
                s.fighters[0].ledge = 0;
                s
            }),
            ("ledge_invuln", {
                let mut s = state;
                s.fighters[0].ledge_invuln = 1;
                s
            }),
            ("ledge_grab_count", {
                let mut s = state;
                s.fighters[0].ledge_grab_count = 1;
                s
            }),
            ("ledge_cooldown", {
                let mut s = state;
                s.fighters[0].ledge_cooldown = 1;
                s
            }),
            ("percent", {
                let mut s = state;
                s.fighters[0].percent = Fx::from_int(1);
                s
            }),
            ("rules_stocks", {
                let mut s = state;
                s.rules.stocks = 1;
                s
            }),
            ("rules_time_limit", {
                let mut s = state;
                s.rules.time_limit = 1;
                s
            }),
            ("roster", {
                let mut s = state;
                s.roster = 0b0001;
                s
            }),
            ("winner", {
                let mut s = state;
                s.winner = 0;
                s
            }),
            ("sudden_death", {
                let mut s = state;
                s.sudden_death = true;
                s
            }),
            ("stocks", {
                let mut s = state;
                s.fighters[0].stocks = 1;
                s
            }),
            ("hitlag", {
                let mut s = state;
                s.fighters[0].hitlag = 1;
                s
            }),
            ("hitstun", {
                let mut s = state;
                s.fighters[0].hitstun = 1;
                s
            }),
            ("launch_pending", {
                let mut s = state;
                s.fighters[0].launch_pending = true;
                s
            }),
            ("launch_kb", {
                let mut s = state;
                s.fighters[0].launch_kb = Fx::from_int(1);
                s
            }),
            ("launch_set", {
                let mut s = state;
                s.fighters[0].launch_set = true;
                s
            }),
            ("launch_older", {
                let mut s = state;
                s.fighters[1].launch_older = true;
                s
            }),
            ("launch_angle", {
                let mut s = state;
                s.fighters[0].launch_angle = 1;
                s
            }),
            ("kb_vel", {
                let mut s = state;
                s.fighters[0].kb_vel.y = Fx::from_raw(1);
                s
            }),
            ("tumble", {
                let mut s = state;
                s.fighters[0].tumble = true;
                s
            }),
            ("move_id", {
                let mut s = state;
                s.fighters[0].move_id = 1;
                s
            }),
            ("hit_mask", {
                let mut s = state;
                s.fighters[0].hit_mask = 1;
                s
            }),
            ("hit_mask high groups", {
                let mut s = state;
                s.fighters[0].hit_mask = 1 << 28;
                s
            }),
            ("sdi_mult", {
                let mut s = state;
                s.fighters[0].sdi_mult = 120;
                s
            }),
            ("link", {
                let mut s = state;
                s.fighters[0].link = true;
                s
            }),
            ("link_vel", {
                let mut s = state;
                s.fighters[0].link_vel = Vec2::new(Fx::ZERO, Fx::ONE);
                s
            }),
            ("link_point", {
                let mut s = state;
                s.fighters[0].link_point = Vec2::new(Fx::ONE, Fx::ZERO);
                s
            }),
            ("charge", {
                let mut s = state;
                s.fighters[0].charge = 1;
                s
            }),
            ("shield_hp", {
                let mut s = state;
                s.fighters[0].shield_hp = Fx::from_int(7);
                s
            }),
            ("balloon_faf", {
                let mut s = state;
                s.fighters[0].balloon_faf = 40;
                s
            }),
            ("damage_fall", {
                let mut s = state;
                s.fighters[0].damage_fall = true;
                s
            }),
            ("b_reversed", {
                let mut s = state;
                s.fighters[0].b_reversed = true;
                s
            }),
            ("aim", {
                let mut s = state;
                s.fighters[0].aim = 1;
                s
            }),
            ("move_air", {
                let mut s = state;
                s.fighters[0].move_air = true;
                s
            }),
            ("steer", {
                let mut s = state;
                s.fighters[0].steer = 3;
                s
            }),
            ("balloon_left", {
                let mut s = state;
                s.fighters[0].balloon_left = Fx::ONE;
                s
            }),
            ("hits_taken", {
                let mut s = state;
                s.fighters[0].hits_taken = 3;
                s
            }),
            ("tech_timer", {
                let mut s = state;
                s.fighters[0].tech_timer = 2;
                s
            }),
            ("asdi", {
                let mut s = state;
                s.fighters[0].asdi = 2;
                s
            }),
            ("sdi_wait", {
                let mut s = state;
                s.fighters[0].sdi_wait = 1;
                s
            }),
            ("stale", {
                let mut s = state;
                s.fighters[0].stale[8] = 3;
                s
            }),
            ("shield_stun", {
                let mut s = state;
                s.fighters[0].shield_stun = 1;
                s
            }),
            ("shield_grab", {
                let mut s = state;
                s.fighters[0].shield_grab = 1;
                s
            }),
            ("grab_with", {
                let mut s = state;
                s.fighters[0].grab_with = 1;
                s
            }),
            ("grab_timer", {
                let mut s = state;
                s.fighters[0].grab_timer = 1;
                s
            }),
            ("grab_immune", {
                let mut s = state;
                s.fighters[0].grab_immune = 1;
                s
            }),
            ("invuln", {
                let mut s = state;
                s.fighters[0].invuln = 1;
                s
            }),
            ("spawn_request", {
                let mut s = state;
                s.fighters[0].spawn_request = true;
                s
            }),
            ("active", {
                let mut s = state;
                s.fighters[2].active = false;
                s
            }),
            ("spawn_fixed", {
                let mut s = state;
                s.fighters[0].spawn_fixed = true;
                s
            }),
            ("spawn_custom", {
                let mut s = state;
                s.fighters[0].spawn_custom = true;
                s
            }),
            ("spawn_pos", {
                let mut s = state;
                s.fighters[1].spawn_pos.y = Fx::from_raw(1);
                s
            }),
            ("spawn_vel", {
                let mut s = state;
                s.fighters[1].spawn_vel.x = Fx::from_raw(1);
                s
            }),
            ("counter_damage", {
                let mut s = state;
                s.fighters[1].counter_damage = Fx::from_raw(1);
                s
            }),
            ("pull", {
                let mut s = state;
                s.fighters[1].pull = true;
                s
            }),
            ("stored", {
                let mut s = state;
                s.fighters[2].stored = 1;
                s
            }),
            ("gauge", {
                let mut s = state;
                s.fighters[2].gauge = Fx::from_raw(1);
                s
            }),
            ("gauge_mode", {
                let mut s = state;
                s.fighters[2].gauge_mode = 1;
                s
            }),
            ("balloons", {
                let mut s = state;
                s.fighters[1].balloons = 2;
                s
            }),
            ("balloon_hp", {
                let mut s = state;
                s.fighters[1].balloon_hp = Fx::from_raw(1);
                s
            }),
            ("helium_used", {
                let mut s = state;
                s.fighters[3].helium_used = 16;
                s
            }),
            ("pocket", {
                let mut s = state;
                s.fighters[2].pocket = 3;
                s
            }),
            ("pocket_item", {
                let mut s = state;
                s.fighters[2].pocket_item = true;
                s
            }),
            ("gauge_drain", {
                let mut s = state;
                s.fighters[2].gauge_drain = true;
                s
            }),
            ("move_y0", {
                let mut s = state;
                s.fighters[2].move_y0 = Fx::from_raw(1);
                s
            }),
            ("wall_jumps", {
                let mut s = state;
                s.fighters[1].wall_jumps = 1;
                s
            }),
            ("dash_buffer", {
                let mut s = state;
                s.fighters[1].dash_buffer = 1;
                s
            }),
            ("drop_pending", {
                let mut s = state;
                s.fighters[1].drop_pending = 1;
                s
            }),
            ("dodge_stale", {
                let mut s = state;
                s.fighters[1].dodge_stale = 1;
                s
            }),
            ("dodge_level", {
                let mut s = state;
                s.fighters[1].dodge_level = 1;
                s
            }),
            ("dodge_clock", {
                let mut s = state;
                s.fighters[1].dodge_clock = 1;
                s
            }),
            ("air_flags", {
                let mut s = state;
                s.fighters[1].air_flags = 1;
                s
            }),
            ("air_marks", {
                let mut s = state;
                s.fighters[2].air_marks = 1;
                s
            }),
            ("move_powered", {
                let mut s = state;
                s.fighters[1].move_powered = true;
                s
            }),
            ("withstood", {
                let mut s = state;
                s.fighters[1].withstood = 1;
                s
            }),
            ("tether_ledge", {
                let mut s = state;
                s.fighters[1].tether_ledge = 0;
                s
            }),
            ("tether_count", {
                let mut s = state;
                s.fighters[1].tether_count = 1;
                s
            }),
            ("tether_from", {
                let mut s = state;
                s.fighters[1].tether_from = Vec2::new(Fx::ONE, Fx::ZERO);
                s
            }),
            ("move_end", {
                let mut s = state;
                s.fighters[1].move_end = 1;
                s
            }),
            ("move_held", {
                let mut s = state;
                s.fighters[1].move_held = 1;
                s
            }),
            ("curse_left", {
                let mut s = state;
                s.fighters[1].curse_left = 1;
                s
            }),
            ("curse_every", {
                let mut s = state;
                s.fighters[1].curse_every = 1;
                s
            }),
            ("curse_tick", {
                let mut s = state;
                s.fighters[1].curse_tick = 1;
                s
            }),
            ("curse_damage", {
                let mut s = state;
                s.fighters[1].curse_damage = Fx::ONE;
                s
            }),
            ("wing_points", {
                let mut s = state;
                s.fighters[1].wing_points = Fx::from_raw(1);
                s
            }),
            ("wing_spent", {
                let mut s = state;
                s.fighters[1].wing_spent = true;
                s
            }),
            ("hit_by", {
                let mut s = state;
                s.fighters[1].hit_by = 0;
                s
            }),
            ("curse_by", {
                let mut s = state;
                s.fighters[1].curse_by = 1;
                s
            }),
            ("presses", {
                let mut s = state;
                s.fighters[1].presses = 1;
                s
            }),
            ("releases", {
                let mut s = state;
                s.fighters[2].releases = 1;
                s
            }),
            ("touched_wall", {
                let mut s = state;
                s.fighters[2].touched_wall = true;
                s
            }),
            ("next_move", {
                let mut s = state;
                s.fighters[1].next_move = 30;
                s
            }),
            ("lock", {
                let mut s = state;
                s.fighters[1].lock.x = Fx::from_raw(1);
                s
            }),
            ("locked", {
                let mut s = state;
                s.fighters[1].locked = true;
                s
            }),
            ("special_flick", {
                let mut s = state;
                s.fighters[2].special_flick = true;
                s
            }),
            ("tripped", {
                let mut s = state;
                s.fighters[1].tripped = true;
                s
            }),
            ("trip_soft", {
                let mut s = state;
                s.fighters[2].trip_soft = true;
                s
            }),
            ("pull_wait", {
                let mut s = state;
                s.fighters[3].pull_wait = 1;
                s
            }),
            ("bury_timer", {
                let mut s = state;
                s.fighters[1].bury_timer = 1;
                s
            }),
            ("bury_kb", {
                let mut s = state;
                s.fighters[2].bury_kb = Fx::from_raw(1);
                s
            }),
            ("bury_angle", {
                let mut s = state;
                s.fighters[2].bury_angle = 1;
                s
            }),
            ("bury_set", {
                let mut s = state;
                s.fighters[3].bury_set = true;
                s
            }),
            ("state buried", {
                let mut s = state;
                s.fighters[3].state = FighterState::Buried;
                s
            }),
            ("carry_jumps", {
                let mut s = state;
                s.fighters[2].carry_jumps = 1;
                s
            }),
            ("grab_start", {
                let mut s = state;
                s.fighters[1].grab_start = 165;
                s
            }),
            ("state carrying", {
                let mut s = state;
                s.fighters[0].state = FighterState::Carrying;
                s
            }),
            ("state encased", {
                let mut s = state;
                s.fighters[1].state = FighterState::Encased;
                s
            }),
            ("encase_timer", {
                let mut s = state;
                s.fighters[1].encase_timer = Fx::from_ratio(126, 10);
                s
            }),
            ("encase_age", {
                let mut s = state;
                s.fighters[0].encase_age = 3;
                s
            }),
            ("encase_char", {
                let mut s = state;
                s.fighters[2].encase_char = 8;
                s
            }),
            ("encase_move", {
                let mut s = state;
                s.fighters[3].encase_move = 30;
                s
            }),
            ("air_jump_frame", {
                let mut s = state;
                s.fighters[2].air_jump_frame = 7;
                s
            }),
            ("spawn2_request", {
                let mut s = state;
                s.fighters[1].spawn2_request = true;
                s
            }),
            ("spawn2_fixed", {
                let mut s = state;
                s.fighters[1].spawn2_fixed = true;
                s
            }),
            ("spawn2_pos", {
                let mut s = state;
                s.fighters[2].spawn2_pos.y = Fx::from_raw(1);
                s
            }),
            ("spawn2_vel", {
                let mut s = state;
                s.fighters[3].spawn2_vel.x = Fx::from_raw(-1);
                s
            }),
            ("late_next", {
                let mut s = state;
                s.fighters[0].late_next = 17;
                s
            }),
            ("late_left", {
                let mut s = state;
                s.fighters[2].late_left = 3;
                s
            }),
            ("projectile_hit_mask", {
                let mut s = state;
                s.projectiles[3].hit_mask = 1 << 30;
                s
            }),
            ("projectile_hit_age", {
                let mut s = state;
                s.projectiles[3].hit_age[MAX_FIGHTERS - 1] = 1;
                s
            }),
            ("projectile_facing", {
                let mut s = state;
                s.projectiles[3].facing = -1;
                s
            }),
            ("projectile_bursting", {
                let mut s = state;
                s.projectiles[2].bursting = true;
                s
            }),
            ("link_kind", {
                let mut s = state;
                s.fighters[2].link_kind = LINK_VORTEX;
                s
            }),
            ("vortex", {
                let mut s = state;
                s.fighters[1].vortex = 5;
                s
            }),
            ("posvec", {
                let mut s = state;
                s.fighters[2].posvec = 10;
                s
            }),
            ("shield_sdi", {
                let mut s = state;
                s.fighters[3].shield_sdi = true;
                s
            }),
            ("fighter_vars", {
                let mut s = state;
                s.fighters[3].vars[FIGHTER_VARS - 1] = 1;
                s
            }),
            ("projectile_vars", {
                let mut s = state;
                s.projectiles[MAX_PROJECTILES - 1].vars[PROJECTILE_VARS - 1] = 1;
                s
            }),
            ("projectile", {
                let mut s = state;
                s.projectiles[MAX_PROJECTILES - 1].active = true;
                s
            }),
            ("projectile_owner", {
                let mut s = state;
                s.projectiles[0].owner = 1;
                s
            }),
            ("projectile_origin", {
                let mut s = state;
                s.projectiles[0].origin = 1;
                s
            }),
            ("projectile_power", {
                let mut s = state;
                s.projectiles[0].power = 150;
                s
            }),
            ("projectile_kb_add", {
                let mut s = state;
                s.projectiles[2].kb_add = 1;
                s
            }),
            ("projectile_size", {
                let mut s = state;
                s.projectiles[2].size = 120;
                s
            }),
            ("spawn_kb_add", {
                let mut s = state;
                s.fighters[1].spawn_kb_add = 1;
                s
            }),
            ("spawn_size", {
                let mut s = state;
                s.fighters[1].spawn_size = 80;
                s
            }),
            ("projectile_damage", {
                let mut s = state;
                s.projectiles[1].damage = Fx::from_int(7);
                s
            }),
            ("projectile_kind", {
                let mut s = state;
                s.projectiles[1].kind = 3;
                s
            }),
            ("projectile_rest", {
                let mut s = state;
                s.projectiles[1].rest = REST_LODGED;
                s
            }),
            ("projectile_pocketed", {
                let mut s = state;
                s.projectiles[1].rest = REST_POCKETED;
                s
            }),
            ("projectile_stale", {
                let mut s = state;
                s.projectiles[1].stale = 9;
                s
            }),
            ("projectile_bounces", {
                let mut s = state;
                s.projectiles[1].bounces = 1;
                s
            }),
            ("projectile_pos", {
                let mut s = state;
                s.projectiles[0].pos.x = Fx::from_raw(1);
                s
            }),
            ("holding", {
                let mut s = state;
                s.fighters[1].holding = 2;
                s
            }),
            ("item_order", {
                let mut s = state;
                s.fighters[2].item_order = 1;
                s
            }),
            ("item_spec", {
                let mut s = state;
                s.fighters[3].item_spec = 1;
                s
            }),
            ("item_throw", {
                let mut s = state;
                s.fighters[0].item_throw = 3;
                s
            }),
            ("state item_pickup", {
                let mut s = state;
                s.fighters[1].state = FighterState::ItemPickup;
                s
            }),
            ("state item_throw", {
                let mut s = state;
                s.fighters[1].state = FighterState::ItemThrow;
                s
            }),
            ("item", {
                let mut s = state;
                s.items[MAX_ITEMS - 1].active = true;
                s
            }),
            ("item_origin", {
                let mut s = state;
                s.items[0].origin = 1;
                s
            }),
            ("item_maker", {
                let mut s = state;
                s.items[0].maker = 9;
                s
            }),
            ("item_spec_index", {
                let mut s = state;
                s.items[0].spec = 1;
                s
            }),
            ("item_kind", {
                let mut s = state;
                s.items[0].kind = 2;
                s
            }),
            ("item_owner", {
                let mut s = state;
                s.items[1].owner = 3;
                s
            }),
            ("item_holder", {
                let mut s = state;
                s.items[1].holder = NONE;
                s
            }),
            ("item_rest", {
                let mut s = state;
                s.items[1].rest = ITEM_LODGED;
                s
            }),
            ("item_move_id", {
                let mut s = state;
                s.items[2].move_id = 16;
                s
            }),
            ("item_pos", {
                let mut s = state;
                s.items[2].pos.x = Fx::from_raw(1);
                s
            }),
            ("item_vel", {
                let mut s = state;
                s.items[2].vel.y = Fx::from_raw(-1);
                s
            }),
            ("item_platform", {
                let mut s = state;
                s.items[3].platform = NONE;
                s
            }),
            ("item_age", {
                let mut s = state;
                s.items[3].age = 1;
                s
            }),
            ("item_life", {
                let mut s = state;
                s.items[3].life = 1800;
                s
            }),
            ("item_flight", {
                let mut s = state;
                s.items[4].flight = 1;
                s
            }),
            ("item_hp", {
                let mut s = state;
                s.items[4].hp = Fx::from_int(50);
                s
            }),
            ("item_damage", {
                let mut s = state;
                s.items[4].damage = Fx::ONE;
                s
            }),
            ("item_fuse", {
                let mut s = state;
                s.items[5].fuse = 12;
                s
            }),
            ("item_hitlag", {
                let mut s = state;
                s.items[5].hitlag = 1;
                s
            }),
            ("item_hit_mask", {
                let mut s = state;
                s.items[5].hit_mask = 1;
                s
            }),
            ("item_struck", {
                let mut s = state;
                s.items[6].struck[MAX_FIGHTERS - 1] = 1;
                s
            }),
            ("item_facing", {
                let mut s = state;
                s.items[6].facing = -1;
                s
            }),
            ("item_angle", {
                let mut s = state;
                s.items[6].angle = -60;
                s
            }),
            ("item_pocketed", {
                let mut s = state;
                s.items[6].rest = ITEM_POCKETED;
                s
            }),
            ("item_water", {
                let mut s = state;
                s.items[6].water = 3;
                s
            }),
            ("item_grow_in", {
                let mut s = state;
                s.items[6].grow_in = 5;
                s
            }),
            ("item_cuts", {
                let mut s = state;
                s.items[6].cuts = 0x10;
                s
            }),
            ("item_used", {
                let mut s = state;
                s.items[5].used = 1;
                s
            }),
            ("history", {
                let mut s = state;
                s.fighters[0].history[HISTORY_LEN - 1].buttons = 1;
                s
            }),
        ];
        for (name, s) in changed {
            assert_ne!(c, s.checksum(), "checksum ignores `{name}`");
        }
    }
}
