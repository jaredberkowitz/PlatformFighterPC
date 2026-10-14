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
use crate::{FIGHTER_VARS, MAX_FIGHTERS, MAX_SCRIPT_VARS, PROJECTILE_VARS};

/// Frames of input kept per fighter. Must cover the longest buffer window in `FighterParams`.
pub const HISTORY_LEN: usize = 12;
pub const NONE: i8 = -1;
/// [`GameState::winner`] while the match is still being played.
pub const PLAYING: i8 = -1;
/// [`GameState::winner`] when nobody won (the last fighters were knocked out together, or the time ran out level).
pub const DRAW: i8 = -2;
/// Fixed capacity of the projectile pool.
pub const MAX_PROJECTILES: usize = 16;
/// `Fighter::link_kind`: the autolink (angle 367).
pub const LINK_AUTOLINK: u8 = 0;
/// `Fighter::link_kind`: the vortex (angle 366, E28).
pub const LINK_VORTEX: u8 = 1;
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
    /// World angle of the pending launch, in `trig::Angle` units.
    pub launch_angle: u16,
    /// Current launch velocity, decaying each frame.
    pub kb_vel: Vec2,
    pub tumble: bool,
    /// Index of the move being performed, see `moves::MoveId`.
    pub move_id: u8,
    /// The move's hit history: bit `n + 4 * g` set means hit group `g` (0 to 7) of this move has already hit fighter `n`.
    pub hit_mask: u32,
    /// Frames the current smash attack has been charged.
    pub charge: u8,
    /// Shield health; it drains while the shield is up and refills when it is not.
    pub shield_hp: Fx,
    /// Frames of shield stun left after blocking a hit: the shield stays up and the fighter cannot act.
    pub shield_stun: u8,
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
    /// Being pulled (the pull angle): while hitstun lasts the fighter is drawn toward `link_point` (just in front of whoever pulled
    /// it), see `fighter::pull_frame`.
    pub pull: bool,
    /// A charge kept between uses of a move (the needles a fighter has stored), set and read by move scripts (`store`, `stored`).
    pub stored: u8,
    /// Once-per-airtime flags (bit per flag), set by move scripts (`set_air_flag`) and cleared by landing, catching a ledge or being
    /// hit: what lets a move's boost happen only once until the fighter is back on the ground (the reference's cape stall).
    pub air_flags: u8,
    /// Wall jumps since the fighter last stood on the ground or held a ledge (each is weaker: `Ruleset::wall_jump_decay`).
    pub wall_jumps: u8,
    /// A dash flicked during a grab, pummel or throw (-1 or 1, 0 for none), kept while the stick stays pushed that way and started on
    /// the first frame the fighter can act (see `fighter::update`).
    pub dash_buffer: i8,
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
    /// The body's gauge (`content::Gauge`): 0 to its `max`. A KO empties it (owner decision 9: `Fighter::spawn` starts it at 0).
    pub gauge: Fx,
    /// Frames left of the gauge's powered mode (0: not powered).
    pub gauge_mode: u16,
    /// Held by an ordinary grab out of a `Move::gauge_drain` move: until the hold ends, damage taken drains the gauge.
    pub gauge_drain: bool,
    /// Where the fighter's feet were (height) when its current move began (also on a switch by `goto`): what `Move::ledge_drop`
    /// measures the fall from.
    pub move_y0: Fx,
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
    pub hit_mask: u32,
    /// Its age when it last hit each fighter (the per-target rehit interval counts from it).
    pub hit_age: [u8; MAX_FIGHTERS],
    /// The way its owner faced when it was made (-1 or 1; turned round by a reflection): "forward" for a projectile standing still.
    pub facing: i8,
    /// It has burst (`moves::Burst`): it stands still and hits with the burst's phases; its age counts from the burst.
    pub bursting: bool,
}

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
        h.write_u32(self.hit_mask);
        for a in &self.hit_age {
            h.write_u8(*a);
        }
        h.write_i8(self.facing);
        h.write_bool(self.bursting);
    }
}

/// How a match is won. Chosen per match (the host decides online), unlike [`crate::content::Ruleset`], which is part of the
/// roster's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchRules {
    /// Stocks each fighter starts with. 0 means unlimited: nobody is ever eliminated (training, free play).
    pub stocks: u8,
    /// Match length in seconds. 0 means no time limit. When it runs out the fighter with the most stocks wins, then the
    /// one with the least damage; if that is level too, the match is a draw.
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
    pub fighters: [Fighter; MAX_FIGHTERS],
    /// Which fighter holds each ledge, or [`NONE`].
    pub ledge_owner: [i8; MAX_LEDGES],
    pub projectiles: [Projectile; MAX_PROJECTILES],
    pub script_vars: [i32; MAX_SCRIPT_VARS],
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
            shield_hp,
            shield_stun: 0,
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
            counter_damage: Fx::ZERO,
            pull: false,
            stored: 0,
            air_flags: 0,
            wall_jumps: 0,
            dash_buffer: 0,
            presses: 0,
            releases: 0,
            next_move: 0,
            touched_wall: false,
            special_flick: false,
            lock: Vec2::ZERO,
            locked: false,
            tripped: false,
            air_jump_frame: 0,
            bury_timer: 0,
            bury_kb: Fx::ZERO,
            bury_angle: 0,
            bury_set: false,
            gauge: Fx::ZERO,
            gauge_mode: 0,
            gauge_drain: false,
            move_y0: Fx::ZERO,
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
            script_vars: [0; MAX_SCRIPT_VARS],
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
        h.write_u16(self.launch_angle);
        self.kb_vel.hash_into(h);
        h.write_bool(self.tumble);
        h.write_u8(self.move_id);
        h.write_u32(self.hit_mask);
        h.write_u8(self.charge);
        h.write_u16(self.sdi_mult);
        h.write_bool(self.link);
        self.link_vel.hash_into(h);
        self.link_point.hash_into(h);
        h.write_u8(self.link_kind);
        h.write_u8(self.vortex);
        self.shield_hp.hash_into(h);
        h.write_u8(self.shield_stun);
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
        h.write_u8(self.wall_jumps);
        h.write_i8(self.dash_buffer);
        h.write_u8(self.presses);
        h.write_u8(self.releases);
        h.write_u8(self.next_move);
        h.write_bool(self.touched_wall);
        h.write_bool(self.special_flick);
        self.lock.hash_into(h);
        h.write_bool(self.locked);
        h.write_bool(self.tripped);
        h.write_u16(self.bury_timer);
        self.bury_kb.hash_into(h);
        h.write_u16(self.bury_angle);
        h.write_bool(self.bury_set);
        h.write_u8(self.air_jump_frame);
        h.write_bool(self.spawn_custom);
        h.write_bool(self.spawn_fixed);
        self.spawn_pos.hash_into(h);
        self.spawn_vel.hash_into(h);
        h.write_u8(self.spawn_kb_add);
        h.write_u16(self.spawn_size);
        self.gauge.hash_into(h);
        h.write_u16(self.gauge_mode);
        h.write_bool(self.gauge_drain);
        self.move_y0.hash_into(h);
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
        for v in &self.script_vars {
            h.write_i32(*v);
        }
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
            ("air_flags", {
                let mut s = state;
                s.fighters[1].air_flags = 1;
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
            ("air_jump_frame", {
                let mut s = state;
                s.fighters[2].air_jump_frame = 7;
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
