//! Content to text and back: fighters, weapons (movesets), the stage and the ruleset.
//!
//! The reader reports every problem it finds, with line numbers, instead of stopping at the first. A file
//! that parses is not necessarily *valid*; see [`crate::validate`] for the range and sanity checks.

use crate::tree::{Block, Item};
use sim_core::content::{
    Content, FighterParams, Gauge, Ledge, Names, Platform, Ruleset, Stage, StageLook, Wing,
    WingRow, WING_POINTS,
};
use sim_core::moves::{
    AirBlock, Armor, Balloons, Burst, Carry, ChargeAdd, ChargedShot, Counter, CounterBubble, Curse,
    DropThrough, Encase, Fall, FlagVersion, GroundProbe, Hitbox, ItemBlast, ItemSpec, Latch, Lerp,
    Motion, Move, MoveId, Orbs, PartIntangible, Placed, PocketCatch, PocketRelease, ProjectileArc,
    ProjectileSpawn, Reflector, Tether, Tool, Toss, Trap, Weapon, AIM_ANY, AIM_DOWN, AIM_STRAIGHT,
    AIM_UP, ANGLE_BACKWARD, ANGLE_BY_BOX, ANGLE_BY_POSITION, ANGLE_BY_TRAVEL, ANGLE_FORWARD,
    ARMOR_DAMAGE, ARMOR_KNOCKBACK, ARMOR_SUPER, EFFECT_BURY, EFFECT_ELECTRIC, EFFECT_ENCASE,
    EFFECT_NORMAL, HIT_GRAB, HIT_NORMAL, HIT_PUMMEL, HIT_SEARCH, HIT_THROW, MAX_PHASES, MODE_ANY,
    MODE_PLAIN, MODE_POWERED, PART_HIGH, PART_LIMB, PART_LOW, PART_MIDDLE, POCKET_FREES,
    POCKET_NEVER, POCKET_YES, RULES_CURRENT, RULES_PREVIOUS, SLIP_HARD, SLIP_NONE, SLIP_SOFT,
    STRETCH_BACK_ROLL, STRETCH_NONE, STRETCH_ROLL, STRETCH_SPOT, TARGETS_AERIAL, TARGETS_ALL,
    TARGETS_GROUNDED,
};
use sim_core::{Fx, Vec2, MAX_FIGHTERS};
use sim_script::{format_fixed, parse_fixed, Kind, Program};
use std::collections::BTreeMap;

// ---- Scalars ----------------------------------------------------------------------------------------------

/// A value that has a text form.
pub trait Scalar: Sized + Copy {
    const WHAT: &'static str;
    fn parse(text: &str) -> Option<Self>;
    fn show(&self) -> String;
    fn zero() -> Self;
}

impl Scalar for Fx {
    const WHAT: &'static str = "a number";
    fn parse(text: &str) -> Option<Fx> {
        parse_fixed(text).map(Fx::from_raw)
    }
    fn show(&self) -> String {
        format_fixed(self.raw())
    }
    fn zero() -> Fx {
        Fx::ZERO
    }
}

macro_rules! int_scalar {
    ($($t:ty),*) => {$(
        impl Scalar for $t {
            const WHAT: &'static str = "a whole number";
            fn parse(text: &str) -> Option<$t> {
                text.parse().ok()
            }
            fn show(&self) -> String {
                self.to_string()
            }
            fn zero() -> $t {
                0
            }
        }
    )*};
}
int_scalar!(u8, u16, u32, i16);

impl Scalar for bool {
    const WHAT: &'static str = "true or false";
    fn parse(text: &str) -> Option<bool> {
        match text {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
    fn show(&self) -> String {
        self.to_string()
    }
    fn zero() -> bool {
        false
    }
}

// ---- Reading helpers ------------------------------------------------------------------------------------

fn err(errors: &mut Vec<String>, line: usize, message: impl AsRef<str>) {
    if line > 0 {
        errors.push(format!("line {line}: {}", message.as_ref()));
    } else {
        errors.push(message.as_ref().to_string());
    }
}

/// The `name value` lines of one block, taken one at a time so leftovers can be reported as unknown.
struct Fields<'a> {
    map: BTreeMap<&'a str, (&'a str, usize)>,
    context: String,
    line: usize,
}

impl<'a> Fields<'a> {
    fn new(block: &'a Block, context: &str, errors: &mut Vec<String>) -> Fields<'a> {
        let mut map = BTreeMap::new();
        for item in &block.items {
            if let Item::Field { name, value, line } = item {
                if map.insert(name.as_str(), (value.as_str(), *line)).is_some() {
                    err(
                        errors,
                        *line,
                        format!("`{name}` appears twice in {context}"),
                    );
                }
            }
        }
        Fields {
            map,
            context: context.to_string(),
            line: block.line,
        }
    }

    fn raw(&mut self, name: &str) -> Option<(&'a str, usize)> {
        self.map.remove(name)
    }

    fn parsed<T: Scalar>(&mut self, name: &str, errors: &mut Vec<String>) -> Option<T> {
        let (text, line) = self.raw(name)?;
        match T::parse(text) {
            Some(v) => Some(v),
            None => {
                err(
                    errors,
                    line,
                    format!(
                        "`{name}` in {} must be {}, not `{text}`",
                        self.context,
                        T::WHAT
                    ),
                );
                Some(T::zero())
            }
        }
    }

    /// A value that must be given (or come from the thing this one inherits from).
    fn need<T: Scalar>(&mut self, name: &str, fallback: Option<T>, errors: &mut Vec<String>) -> T {
        if let Some(v) = self.parsed::<T>(name, errors) {
            return v;
        }
        if let Some(v) = fallback {
            return v;
        }
        err(
            errors,
            self.line,
            format!("{} is missing `{name}`", self.context),
        );
        T::zero()
    }

    fn or<T: Scalar>(&mut self, name: &str, default: T, errors: &mut Vec<String>) -> T {
        self.parsed(name, errors).unwrap_or(default)
    }

    fn finish(self, errors: &mut Vec<String>) {
        for (name, (_, line)) in self.map {
            err(
                errors,
                line,
                format!("unknown field `{name}` in {}", self.context),
            );
        }
    }
}

fn children<'a>(block: &'a Block, kind: &str) -> Vec<&'a Block> {
    block
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Block(b) if b.kind == kind => Some(b),
            _ => None,
        })
        .collect()
}

/// Reports blocks and raw sections whose kind is not in `known`.
fn only_known(block: &Block, known: &[&str], context: &str, errors: &mut Vec<String>) {
    for item in &block.items {
        match item {
            Item::Block(b) if !known.contains(&b.kind.as_str()) => {
                err(
                    errors,
                    b.line,
                    format!("unknown section `{}` in {context}", b.kind),
                );
            }
            Item::Raw { kind, line, .. } if !known.contains(&kind.as_str()) => {
                err(
                    errors,
                    *line,
                    format!("unknown section `{kind}` in {context}"),
                );
            }
            _ => {}
        }
    }
}

fn at_most_one<'a>(
    blocks: Vec<&'a Block>,
    kind: &str,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<&'a Block> {
    if blocks.len() > 1 {
        err(
            errors,
            blocks[1].line,
            format!("{context} can have only one `{kind}`"),
        );
    }
    blocks.into_iter().next()
}

fn raw_script<'a>(
    block: &'a Block,
    kind: &str,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<(&'a str, usize)> {
    let mut found = block.items.iter().filter_map(|i| match i {
        Item::Raw {
            kind: k,
            text,
            line,
        } if k == kind => Some((text.as_str(), *line)),
        _ => None,
    });
    let first = found.next();
    if let Some((_, line)) = found.next() {
        err(
            errors,
            line,
            format!("{context} can have only one `{kind}`"),
        );
    }
    first
}

// ---- Fighter parameters and ruleset (generated from the struct definitions) ---------------------------

macro_rules! params_io {
    ($($name:ident : $ty:ty,)*) => {
        fn read_params(
            fields: &mut Fields,
            base: Option<&FighterParams>,
            weapon: u8,
            errors: &mut Vec<String>,
        ) -> FighterParams {
            FighterParams {
                $($name: fields.need::<$ty>(stringify!($name), base.map(|b| b.$name), errors),)*
                gauge: base.map_or(Gauge::NONE, |b| b.gauge),
                air_jump_armor: base.and_then(|b| b.air_jump_armor),
                air_jump_turns: false,
                passive_shield: base.and_then(|b| b.passive_shield),
                wing: base.and_then(|b| b.wing),
                air_jump_drift: 100,
                item_throws: base.map_or(sim_core::content::ItemThrows::STANDARD, |b| b.item_throws),
                weapon,
            }
        }

        fn write_params(p: &FighterParams, into: &mut Block) {
            $(into.field(stringify!($name), <$ty as Scalar>::show(&p.$name));)*
        }

        /// Every parameter name the fighter section accepts (besides `weapon` and `inherit`).
        pub const PARAM_NAMES: &[&str] = &[$(stringify!($name),)*];
    };
}

macro_rules! rules_io {
    ($($name:ident : $ty:ty,)*) => {
        fn read_rules(fields: &mut Fields, base: &Ruleset, errors: &mut Vec<String>) -> Ruleset {
            Ruleset {
                $($name: fields.need::<$ty>(stringify!($name), Some(base.$name), errors),)*
            }
        }

        fn write_rules(r: &Ruleset, into: &mut Block) {
            $(into.field(stringify!($name), <$ty as Scalar>::show(&r.$name));)*
        }

        pub const RULE_NAMES: &[&str] = &[$(stringify!($name),)*];
    };
}

params_io! {
    walk_speed: Fx,
    run_speed: Fx,
    dash_speed: Fx,
    dash_initial_speed: Fx,
    dash_accel: Fx,
    dash_brake: Fx,
    dash_frames: u8,
    dash_reverse_frames: u8,
    dash_turn_delay: u8,
    dash_shield_frame: u8,
    turn_frames: u8,
    run_turn_frames: u8,
    walk_accel: Fx,
    walk_accel_stick: Fx,
    run_accel: Fx,
    run_accel_stick: Fx,
    ground_friction: Fx,
    run_decel: Fx,
    landing_friction: Fx,
    dash_attack_friction: Fx,
    air_speed: Fx,
    air_accel: Fx,
    air_accel_stick: Fx,
    air_friction: Fx,
    gravity: Fx,
    max_fall_speed: Fx,
    fast_fall_speed: Fx,
    jump_squat_frames: u8,
    full_hop_velocity: Fx,
    hop_burst_velocity: Fx,
    hop_burst_frames: u8,
    short_hop_velocity: Fx,
    air_jumps: u8,
    air_jump_velocity: Fx,
    landing_lag: u8,
    heavy_landing_lag: u8,
    input_buffer: u8,
    air_dodge_frames: u8,
    air_dodge_speed: Fx,
    air_dodge_decay: Fx,
    air_dodge_windup: u8,
    air_dodge_sling: Fx,
    air_dodge_landing_lag: u8,
    air_dodge_buffer: u8,
    air_dodge_dir_down_frames: u8,
    air_dodge_dir_side_frames: u8,
    air_dodge_dir_up_frames: u8,
    air_dodge_intangible_start: u8,
    air_dodge_intangible_end: u8,
    air_dodge_dir_intangible_end: u8,
    air_dodge_fall_frame: u8,
    air_dodge_ledge_frame: u8,
    air_dodge_dir_landing_min: u8,
    air_dodge_landing_step: u8,
    ground_assist_dist: Fx,
    wavedash_min_down: Fx,
    waveland_speed: Fx,
    waveland_friction: Fx,
    waveland_lag: u8,
    shield_drop_buffer: u8,
    shield_drop_recovery: u8,
    shield_release_frames: u8,
    roll_frames: u8,
    roll_back_frames: u8,
    roll_speed: Fx,
    roll_move_start: u8,
    roll_move_end: u8,
    roll_intangible_start: u8,
    roll_intangible_end: u8,
    spot_dodge_frames: u8,
    spot_intangible_start: u8,
    spot_intangible_end: u8,
    shield_drop_speed: Fx,
    platform_ignore_frames: u8,
    ledge_reach_x: Fx,
    ledge_min_drop: Fx,
    ledge_reach_down: Fx,
    ledge_hang_dx: Fx,
    ledge_hang_dy: Fx,
    ledge_hang_max: u16,
    ledge_regrab_cooldown: u8,
    ledge_invuln_airtime: u8,
    ledge_invuln_damage: u8,
    ledge_invuln_min: u8,
    ledge_grab_frames: u8,
    ledge_grab_limit: u8,
    ledge_reach_back_x: Fx,
    ledge_getup_frames: u8,
    ledge_getup_intangible: u8,
    ledge_roll_frames: u8,
    ledge_roll_intangible: u8,
    ledge_jump_frames: u8,
    ledge_jump_intangible: u8,
    ledge_option_decay_2: u8,
    ledge_option_decay_3: u8,
    ledge_getup_dx: Fx,
    ledge_roll_dx: Fx,
    ledge_jump_velocity: Fx,
    ledge_jump_dx: Fx,
    ledge_trump_vx: Fx,
    ledge_trump_vy: Fx,
    wall_jump: u8,
    wall_jump_vx: Fx,
    wall_jump_vy: Fx,
    ecb_half_width: Fx,
    ecb_height: Fx,
    ecb_side_height: Fx,
    hitbox_scale: Fx,
    crouch_height: Fx,
    helpless_landing_lag: u8,
    ledge_attack_frames: u8,
    ledge_attack_dx: Fx,
    weight: Fx,
}

rules_io! {
    damage_mult: Fx,
    hitstun_mult: Fx,
    knockback_decay: Fx,
    tumble_knockback: Fx,
    sdi_distance: Fx,
    short_hop_damage: Fx,
    di_degrees: u8,
    lsi_up: Fx,
    lsi_down: Fx,
    lsi_vertical: u8,
    push_ratio: Fx,
    push_hard: Fx,
    push_speed: Fx,
    jump_momentum: Fx,
    jump_momentum_max: Fx,
    respawn_invuln: u8,
    tech_window: u8,
    tech_lag: u8,
    knockdown_lag: u8,
    knockdown_max: u8,
    getup_frames: u8,
    getup_intangible: u8,
    tech_invuln: u8,
    charge_frames: u8,
    charge_bonus_percent: u8,
    shield_max: Fx,
    shield_deplete: Fx,
    shield_regen: Fx,
    shield_restore_percent: u8,
    shield_break_frames: u16,
    shield_break_per_percent: u8,
    shield_break_min: u16,
    shield_break_hop: Fx,
    shield_mash_frames: u8,
    shield_stun_cap: u8,
    perfect_shield_window: u8,
    shield_min_frames: u8,
    shield_grab_delay: u8,
    perfect_shield_stun_cut: u8,
    grab_base_frames: u16,
    grab_percent_tenths: u16,
    grab_min_frames: u16,
    grab_mash_button: u8,
    grab_mash_stick: u8,
    grab_release_lag: u8,
    grab_immunity: u8,
    grab_distance: Fx,
    hitlag_per_damage: Fx,
    hitlag_base: Fx,
    hitlag_cap: u8,
    shield_hitlag_mult: Fx,
    crouch_cancel_kb: Fx,
    crouch_cancel_hitlag: Fx,
    crouch_cancel_hitlag_cap: u8,
    sdi_interval: u8,
    rage_start: Fx,
    rage_full: Fx,
    rage_max: Fx,
    stale_moves: u8,
    hitstun_dodge_cancel: u8,
    hitstun_attack_cancel: u8,
    hitstun_dodge_cancel_speed: Fx,
    hitstun_attack_cancel_speed: Fx,
    gravity_boost: Fx,
    gravity_boost_from: Fx,
    helpless_drift: Fx,
    special_turn_window: u8,
    special_reverse_window: u8,
    platform_drop_frames: u8,
    stick_platform_drop: u8,
    dodge_stale_max: u8,
    dodge_stale_first: u8,
    dodge_stale_every: u8,
    dodge_stale_spot: u8,
    dodge_stale_roll: u8,
    dodge_stale_back_roll: u8,
    dodge_stale_spot_shift: u8,
    dodge_stale_roll_shift: u8,
    dodge_stale_air_cut: Fx,
    autolink_max_speed: Fx,
    autolink_max_fall: Fx,
    pull_rate: Fx,
    pull_max_speed: Fx,
    pull_gap: Fx,
    electric_hitlag_mult: Fx,
    asdi_distance: Fx,
    sdi_combo_hits: u8,
    sdi_combo_mult: Fx,
    balloon_min_faf: u8,
    balloon_max_faf: u8,
    balloon_per_frame: Fx,
    balloon_max: Fx,
    launch_fall_accel: Fx,
    launch_fall: Fx,
    launch_fall_frames: u8,
    vertical_launch_fall: Fx,
    vertical_launch_from: u8,
    vertical_launch_to: u8,
    clank_range: Fx,
    rebound_cap: u8,
    grab_parry_lag: u8,
    tech_lockout: u8,
    wall_tech_frames: u8,
    wall_jump_decay: Fx,
    dash_interrupt_frame: u8,
    wall_tech_invuln: u8,
    bounce_keep: Fx,
    ground_bounce_speed: Fx,
    respawn_height: Fx,
    respawn_platform_frames: u8,
    trip_lag: u8,
    trip_max: u8,
    trip_invuln: u8,
    trip_base: u8,
    trip_base_kb: Fx,
    slip_lag: u8,
    slip_max: u8,
    bury_base: Fx,
    bury_percent: Fx,
    bury_kb: Fx,
    bury_behind_1: u8,
    bury_behind_2: u8,
    bury_ahead_1: u8,
    bury_ahead_2: u8,
    bury_break_kb: Fx,
    bury_exit_lag: u8,
    bury_depth: Fx,
    item_pick_frames: u8,
    item_side_frames: u8,
    item_up_frames: u8,
    item_down_frames: u8,
    item_dash_frames: u8,
    item_drop_frames: u8,
    item_side_speed: Fx,
    item_side_smash: Fx,
    item_side_angle: i16,
    item_side_smash_angle: i16,
    item_up_speed: Fx,
    item_up_smash: Fx,
    item_down_speed: Fx,
    item_down_smash: Fx,
    item_dash_speed: Fx,
    item_dash_angle: i16,
    item_drop_speed: Fx,
    item_drop_angle: i16,
    item_reach_front: Fx,
    item_reach_back: Fx,
    item_reach_up: Fx,
    item_hand_x: Fx,
    item_hand_y: Fx,
    prev_damage_mult: Fx,
    prev_hitlag_per_damage: Fx,
    prev_hitlag_base: Fx,
    prev_rage_max: Fx,
    prev_tumble_hitstun: u8,
}

// ---- Moves ------------------------------------------------------------------------------------------------

fn kind_name(kind: u8) -> &'static str {
    match kind {
        HIT_GRAB => "grab",
        HIT_THROW => "throw",
        HIT_PUMMEL => "pummel",
        HIT_SEARCH => "search",
        _ => "normal",
    }
}

/// The `phase { from ...; hitbox { ... } }` children of a projectile or of its burst.
fn read_phases(
    b: &Block,
    ctx: &str,
    errors: &mut Vec<String>,
) -> [Option<(u8, Hitbox)>; MAX_PHASES] {
    let mut phases = [None; MAX_PHASES];
    let blocks = children(b, "phase");
    if blocks.len() > MAX_PHASES {
        err(
            errors,
            b.line,
            format!("{ctx} has at most {MAX_PHASES} phases"),
        );
    }
    for (slot, pb) in phases.iter_mut().zip(blocks) {
        let pctx = format!("a phase of {ctx}");
        only_known(pb, &["hitbox"], &pctx, errors);
        let mut pf = Fields::new(pb, &pctx, errors);
        let from: u8 = pf.need("from", None, errors);
        pf.finish(errors);
        match at_most_one(children(pb, "hitbox"), "hitbox", &pctx, errors) {
            Some(h) => *slot = Some((from, read_hitbox(h, &pctx, errors))),
            None => err(errors, pb.line, format!("{pctx} needs a hitbox")),
        }
    }
    phases
}

/// Writes `phases` as `phase` children of `c`.
fn write_phases(c: &mut Block, phases: &[Option<(u8, Hitbox)>; MAX_PHASES]) {
    for (from, hb) in phases.iter().flatten() {
        let mut pb = Block::new("phase", None);
        pb.field("from", from.to_string());
        pb.push(write_hitbox(hb));
        c.push(pb);
    }
}

/// A stand-in for a hitbox that is missing (an error has been reported).
fn missing_hitbox() -> Hitbox {
    Hitbox {
        start: 0,
        end: 0,
        x: Fx::ZERO,
        y: Fx::ZERO,
        radius: Fx::ONE,
        damage: Fx::ZERO,
        angle: 0,
        base_knockback: 0,
        knockback_growth: 0,
        priority: 0,
        group: 0,
        kind: HIT_NORMAL,
        shield_damage: 100,
        hitlag: 100,
        effect: EFFECT_NORMAL,
        targets: TARGETS_ALL,
        fixed_kb: 0,
        hitstun_add: 0,
        sdi: 100,
        link_ahead: Fx::ZERO,
        aim: AIM_ANY,
        set_weight: false,
        reverse: false,
        push: false,
        shieldstun: 100,
        angle_type: ANGLE_BY_BOX,
        command: false,
        clanks: true,
        rebounds: true,
        trip: 0,
        half_height: Fx::ZERO,
        posvec_x: Fx::ZERO,
        posvec_y: Fx::ZERO,
        posvec_frames: 0,
        half_width: Fx::ZERO,
        rules: RULES_CURRENT,
        slip: SLIP_NONE,
        fixed_stun: 0,
        unblockable: false,
        mode: MODE_ANY,
        part: 0,
        blockable: false,
    }
}

fn read_hitbox(block: &Block, context: &str, errors: &mut Vec<String>) -> Hitbox {
    let mut f = Fields::new(block, context, errors);
    let kind = match f.raw("kind") {
        None => HIT_NORMAL,
        Some(("normal", _)) => HIT_NORMAL,
        Some(("grab", _)) => HIT_GRAB,
        Some(("throw", _)) => HIT_THROW,
        Some(("pummel", _)) => HIT_PUMMEL,
        Some(("search", _)) => HIT_SEARCH,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox kind must be normal, grab, throw, pummel or search, not `{other}`"),
            );
            HIT_NORMAL
        }
    };
    let effect = match f.raw("effect") {
        None | Some(("normal", _)) => EFFECT_NORMAL,
        Some(("electric", _)) => EFFECT_ELECTRIC,
        Some(("bury", _)) => EFFECT_BURY,
        Some(("encase", _)) => EFFECT_ENCASE,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox effect must be normal, electric, bury or encase, not `{other}`"),
            );
            EFFECT_NORMAL
        }
    };
    let targets = match f.raw("targets") {
        None | Some(("all", _)) => TARGETS_ALL,
        Some(("grounded", _)) => TARGETS_GROUNDED,
        Some(("aerial", _)) => TARGETS_AERIAL,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox targets must be all, grounded or aerial, not `{other}`"),
            );
            TARGETS_ALL
        }
    };
    let aim = match f.raw("aim") {
        None | Some(("any", _)) => AIM_ANY,
        Some(("up", _)) => AIM_UP,
        Some(("straight", _)) => AIM_STRAIGHT,
        Some(("down", _)) => AIM_DOWN,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox aim must be any, up, straight or down, not `{other}`"),
            );
            AIM_ANY
        }
    };
    let angle_type = match f.raw("angle_type") {
        None | Some(("box", _)) => ANGLE_BY_BOX,
        Some(("forward", _)) => ANGLE_FORWARD,
        Some(("backward", _)) => ANGLE_BACKWARD,
        Some(("position", _)) => ANGLE_BY_POSITION,
        Some(("travel", _)) => ANGLE_BY_TRAVEL,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!(
                    "hitbox angle_type must be box, forward, backward, position or travel, not `{other}`"
                ),
            );
            ANGLE_BY_BOX
        }
    };
    let rules = match f.raw("rules") {
        None | Some(("current", _)) => RULES_CURRENT,
        Some(("previous", _)) => RULES_PREVIOUS,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox rules must be current or previous, not `{other}`"),
            );
            RULES_CURRENT
        }
    };
    let slip = match f.raw("slip") {
        None | Some(("none", _)) => SLIP_NONE,
        Some(("hard", _)) => SLIP_HARD,
        Some(("soft", _)) => SLIP_SOFT,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox slip must be none, hard or soft, not `{other}`"),
            );
            SLIP_NONE
        }
    };
    // Sim 121 (E52): when the box is out, by the gauge's powered mode as the attack began.
    let mode = match f.raw("mode") {
        None | Some(("any", _)) => MODE_ANY,
        Some(("plain", _)) => MODE_PLAIN,
        Some(("powered", _)) => MODE_POWERED,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox mode must be any, plain or powered, not `{other}`"),
            );
            MODE_ANY
        }
    };
    let hb = Hitbox {
        start: f.need("start", None, errors),
        end: f.need("end", None, errors),
        x: f.need("x", None, errors),
        y: f.need("y", None, errors),
        radius: f.need("radius", None, errors),
        damage: f.need("damage", None, errors),
        angle: f.need("angle", None, errors),
        base_knockback: f.need("bkb", None, errors),
        knockback_growth: f.need("kbg", None, errors),
        priority: f.or("priority", 0, errors),
        group: f.or("group", 0, errors),
        kind,
        shield_damage: f.or("shield_damage", 100, errors),
        hitlag: f.or("hitlag", 100, errors),
        effect,
        targets,
        fixed_kb: f.or("fixed_kb", 0, errors),
        hitstun_add: f.or("hitstun_add", 0, errors),
        sdi: f.or("sdi", 100, errors),
        link_ahead: f.or("link_ahead", Fx::ZERO, errors),
        aim,
        set_weight: f.or("set_weight", false, errors),
        reverse: f.or("reverse", false, errors),
        push: f.or("push", false, errors),
        shieldstun: f.or("shieldstun", 100, errors),
        angle_type,
        command: f.or("command", false, errors),
        clanks: f.or("clanks", true, errors),
        rebounds: f.or("rebounds", true, errors),
        trip: f.or("trip", 0, errors),
        half_height: f.or("half_height", Fx::ZERO, errors),
        posvec_x: f.or("posvec_x", Fx::ZERO, errors),
        posvec_y: f.or("posvec_y", Fx::ZERO, errors),
        posvec_frames: f.or("posvec_frames", 0, errors),
        half_width: f.or("half_width", Fx::ZERO, errors),
        rules,
        slip,
        fixed_stun: f.or("fixed_stun", 0, errors),
        unblockable: f.or("unblockable", false, errors),
        mode,
        part: f.or("part", 0, errors),
        // Sim 123: a grab box shields and reflectors stop (E56).
        blockable: f.or("blockable", false, errors),
    };
    f.finish(errors);
    hb
}

fn write_hitbox(hb: &Hitbox) -> Block {
    let mut b = Block::new("hitbox", None);
    b.field("start", hb.start.to_string());
    b.field("end", hb.end.to_string());
    b.field("x", hb.x.show());
    b.field("y", hb.y.show());
    b.field("radius", hb.radius.show());
    b.field("damage", hb.damage.show());
    b.field("angle", hb.angle.to_string());
    b.field("bkb", hb.base_knockback.to_string());
    b.field("kbg", hb.knockback_growth.to_string());
    if hb.priority != 0 {
        b.field("priority", hb.priority.to_string());
    }
    if hb.group != 0 {
        b.field("group", hb.group.to_string());
    }
    if hb.kind != HIT_NORMAL {
        b.field("kind", kind_name(hb.kind));
    }
    if hb.shield_damage != 100 {
        b.field("shield_damage", hb.shield_damage.to_string());
    }
    if hb.hitlag != 100 {
        b.field("hitlag", hb.hitlag.to_string());
    }
    if hb.effect == EFFECT_ELECTRIC {
        b.field("effect", "electric");
    } else if hb.effect == EFFECT_BURY {
        b.field("effect", "bury");
    } else if hb.effect == EFFECT_ENCASE {
        b.field("effect", "encase");
    }
    match hb.targets {
        TARGETS_GROUNDED => b.field("targets", "grounded"),
        TARGETS_AERIAL => b.field("targets", "aerial"),
        _ => {}
    }
    if hb.fixed_kb != 0 {
        b.field("fixed_kb", hb.fixed_kb.to_string());
    }
    if hb.hitstun_add != 0 {
        b.field("hitstun_add", hb.hitstun_add.to_string());
    }
    if hb.sdi != 100 {
        b.field("sdi", hb.sdi.to_string());
    }
    if hb.link_ahead != Fx::ZERO {
        b.field("link_ahead", hb.link_ahead.show());
    }
    match hb.aim {
        AIM_UP => b.field("aim", "up"),
        AIM_STRAIGHT => b.field("aim", "straight"),
        AIM_DOWN => b.field("aim", "down"),
        _ => {}
    }
    if hb.set_weight {
        b.field("set_weight", "true");
    }
    if hb.reverse {
        b.field("reverse", "true");
    }
    if hb.push {
        b.field("push", "true");
    }
    if !hb.clanks {
        b.field("clanks", "false");
    }
    if !hb.rebounds {
        b.field("rebounds", "false");
    }
    if hb.trip != 0 {
        b.field("trip", hb.trip.to_string());
    }
    if hb.command {
        b.field("command", "true");
    }
    if hb.shieldstun != 100 {
        b.field("shieldstun", hb.shieldstun.to_string());
    }
    if hb.half_height != Fx::ZERO {
        b.field("half_height", hb.half_height.show());
    }
    if hb.half_width != Fx::ZERO {
        b.field("half_width", hb.half_width.show());
    }
    if hb.posvec_frames != 0 {
        b.field("posvec_x", hb.posvec_x.show());
        b.field("posvec_y", hb.posvec_y.show());
        b.field("posvec_frames", hb.posvec_frames.to_string());
    }
    match hb.angle_type {
        ANGLE_FORWARD => b.field("angle_type", "forward"),
        ANGLE_BACKWARD => b.field("angle_type", "backward"),
        ANGLE_BY_POSITION => b.field("angle_type", "position"),
        ANGLE_BY_TRAVEL => b.field("angle_type", "travel"),
        _ => {}
    }
    if hb.rules == RULES_PREVIOUS {
        b.field("rules", "previous");
    }
    match hb.slip {
        SLIP_HARD => b.field("slip", "hard"),
        SLIP_SOFT => b.field("slip", "soft"),
        _ => {}
    }
    if hb.fixed_stun != 0 {
        b.field("fixed_stun", hb.fixed_stun.to_string());
    }
    if hb.unblockable {
        b.field("unblockable", "true");
    }
    match hb.mode {
        MODE_PLAIN => b.field("mode", "plain"),
        MODE_POWERED => b.field("mode", "powered"),
        _ => {}
    }
    if hb.part != 0 {
        b.field("part", hb.part.to_string());
    }
    if hb.blockable {
        b.field("blockable", "true");
    }
    b
}

fn compile_script(
    kind: Kind,
    text: &str,
    line: usize,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<Program> {
    match Program::compile(kind, text) {
        Ok(p) => Some(p),
        Err(e) => {
            // The script's own line numbers count from its first line.
            err(
                errors,
                line,
                format!(
                    "{context}: {} script, {} (counting from the first line of the script)",
                    kind.name(),
                    e
                ),
            );
            None
        }
    }
}

fn read_move(block: &Block, context: &str, errors: &mut Vec<String>) -> Move {
    let mut m = Move::empty();
    let mut f = Fields::new(block, context, errors);
    m.total_frames = f.need("total_frames", None, errors);
    m.landing_lag = f.or("landing_lag", 0, errors);
    m.autocancel_before = f.or("autocancel_before", 0, errors);
    m.autocancel_after = f.or("autocancel_after", 255, errors);
    m.intangible = f.or("intangible", 0, errors);
    m.helpless_after = f.or("helpless_after", false, errors);
    m.turns_around = f.or("turns_around", false, errors);
    m.grabs_ledge = f.or("grabs_ledge", false, errors);
    m.ledge_front_from = f.or("ledge_front_from", 255, errors);
    m.ledge_back_from = f.or("ledge_back_from", 255, errors);
    m.fall_drift = f.or("fall_drift", 0, errors);
    m.fall_landing_lag = f.or("fall_landing_lag", 0, errors);
    m.rise_ground = f.or("rise_ground", 100, errors);
    m.rise_air = f.or("rise_air", 100, errors);
    m.start_speed = f.or("start_speed", 100, errors);
    m.steer_frame = f.or("steer_frame", 255, errors);
    m.steer_stick = f.or("steer_stick", 0, errors);
    m.steer_degrees = f.or("steer_degrees", 0, errors);
    m.air_gravity = f.parsed("air_gravity", errors);
    m.air_max_fall = f.parsed("air_max_fall", errors);
    m.air_friction = f.parsed("air_friction", errors);
    m.air_max_rise = f.parsed("air_max_rise", errors);
    m.air_drift = f.or("air_drift", 100, errors);
    m.intangible_from = f.or("intangible_from", 0, errors);
    m.intangible_from_air = f.or("intangible_from_air", 255, errors);
    m.land_continue = f.or("land_continue", false, errors);
    m.fast_fall_from = f.or("fast_fall_from", 0, errors);
    m.lands_from = f.or("lands_from", 0, errors);
    let turn_frame: Option<u8> = f.parsed("turn_frame", errors);
    let turn_stick: u8 = f.or("turn_stick", 0, errors);
    m.turn_at = turn_frame.map(|frame| (frame, turn_stick));
    let hold_x: Option<Fx> = f.parsed("hold_x", errors);
    let hold_y: Option<Fx> = f.parsed("hold_y", errors);
    match (hold_x, hold_y) {
        (Some(x), Some(y)) => m.hold_at = Some((x, y)),
        (None, None) => {}
        _ => err(
            errors,
            block.line,
            format!("{context}: hold_x and hold_y go together"),
        ),
    }
    m.counter_strike = f.or("counter_strike", false, errors);
    m.charge_bonus = f.or("charge_bonus", 0, errors);
    m.charge_at = f.parsed("charge_at", errors);
    m.next_window = f.or("next_window", 0, errors);
    m.next_from = f.or("next_from", 255, errors);
    m.cancel_from = f.or("cancel_from", 255, errors);
    // Sim 115: a chopping move (E43).
    m.chops = f.or("chops", false, errors);
    // Sim 119: damage to the move's own user.
    m.self_damage = f.or("self_damage", Fx::ZERO, errors);
    // Sim 121: the guard's armor (E55) and its gauge gain (E53).
    m.armor_damage = f.or("armor_damage", 1000, errors);
    m.armor_hitlag = f.or("armor_hitlag", 100, errors);
    m.gauge_gain = f.or("gauge_gain", 1000, errors);
    m.dodge_stale = f.or("dodge_stale", false, errors);
    // Sim 126: a stale dodge's stretch in time (owner decision 85) and a stopped blockable grab's early end (owner decision 84).
    m.stale_stretch = match f.raw("stale_stretch") {
        None | Some(("none", _)) => STRETCH_NONE,
        Some(("roll", _)) => STRETCH_ROLL,
        Some(("back_roll", _)) => STRETCH_BACK_ROLL,
        Some(("spot", _)) => STRETCH_SPOT,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("stale_stretch must be none, roll, back_roll or spot, not `{other}`"),
            );
            STRETCH_NONE
        }
    };
    m.shield_end = f.or("shield_end", 0, errors);
    // Sim 125: a motion that stops at the edge (the blade dash).
    m.edge_stop = f.or("edge_stop", false, errors);
    // Sim 125 (E57): when the move's armor works, by the attack's powered start (the wing's smash armor).
    m.armor_mode = match f.raw("armor_mode") {
        None | Some(("any", _)) => MODE_ANY,
        Some(("plain", _)) => MODE_PLAIN,
        Some(("powered", _)) => MODE_POWERED,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("armor_mode must be any, plain or powered, not `{other}`"),
            );
            MODE_ANY
        }
    };
    if let Some((key, line)) = f.raw("next") {
        match MoveId::from_key(key) {
            Some(id) => m.next = Some(id as u8),
            None => err(
                errors,
                line,
                format!("`next` must be a move name, not `{key}`"),
            ),
        }
    }
    m.rehit_groups = f.or("rehit_groups", 0xff, errors);
    m.gauge_drain = f.or("gauge_drain", false, errors);
    m.start_speed_y = f.or("start_speed_y", 100, errors);
    m.motion_drift = f.or("motion_drift", 0, errors);
    m.fall_speed = f.parsed("fall_speed", errors);
    m.fall_accel = f.parsed("fall_accel", errors);
    m.keep_charge = f.or("keep_charge", false, errors);
    // Sim 111 (E4): an item made in the hand, an item used up.
    let pull_frame: Option<u8> = f.parsed("pull_frame", errors);
    let pull_item: Option<u8> = f.parsed("pull_item", errors);
    match (pull_frame, pull_item) {
        (Some(frame), Some(item)) => m.pull = Some((frame, item)),
        (None, None) => {}
        _ => err(
            errors,
            block.line,
            format!("{context}: pull_frame and pull_item go together"),
        ),
    }
    m.spend_item = f.or("spend_item", 255, errors);
    m.push_immune_from = f.or("push_immune_from", 255, errors);
    m.push_immune_to = f.or("push_immune_to", 0, errors);
    let drop_front: Option<Fx> = f.parsed("ledge_drop_front", errors);
    let drop_back: Option<Fx> = f.parsed("ledge_drop_back", errors);
    match (drop_front, drop_back) {
        (Some(a), Some(b)) => m.ledge_drop = Some((a, b)),
        (None, None) => {}
        _ => err(
            errors,
            block.line,
            format!("{context}: ledge_drop_front and ledge_drop_back go together"),
        ),
    }
    for (name, slot) in [
        ("landing_move", &mut m.landing_move),
        ("on_search", &mut m.on_search),
        ("air_version", &mut m.air_version),
        ("smash_version", &mut m.smash_version),
        ("catch_move", &mut m.catch_move),
        ("off_edge", &mut m.off_edge),
        ("limit_version", &mut m.limit_version),
        ("on_land", &mut m.on_land),
        ("shed", &mut m.shed),
        // Sim 128: the special fall's landing hit.
        ("fall_landing_move", &mut m.fall_landing_move),
    ] {
        if let Some((key, line)) = f.raw(name) {
            match MoveId::from_key(key) {
                Some(id) => *slot = Some(id as u8),
                None => err(
                    errors,
                    line,
                    format!("`{name}` must be a move name, not `{key}`"),
                ),
            }
        }
    }
    // Sim 95: a jab's window past its end, ledges only while not rising, a move's own platform drop (E30).
    let late_until: Option<u8> = f.parsed("late_until", errors);
    match (f.raw("late_next"), late_until) {
        (Some((key, line)), Some(until)) => match MoveId::from_key(key) {
            Some(id) => m.late_next = Some((id as u8, until)),
            None => err(
                errors,
                line,
                format!("`late_next` must be a move name, not `{key}`"),
            ),
        },
        (None, None) => {}
        _ => err(
            errors,
            block.line,
            format!("{context}: late_next and late_until go together"),
        ),
    }
    m.ledge_falling_only = f.or("ledge_falling_only", false, errors);
    let drop_from: Option<u8> = f.parsed("drop_from", errors);
    let drop_frames: Option<u8> = f.parsed("drop_frames", errors);
    let drop_while_down: bool = f.or("drop_while_down", false, errors);
    match (drop_from, drop_frames) {
        (Some(from), Some(frames)) => {
            m.drop_through = Some(DropThrough {
                from,
                frames,
                while_down: drop_while_down,
            })
        }
        (None, None) if !drop_while_down => {}
        _ => err(
            errors,
            block.line,
            format!(
                "{context}: drop_from and drop_frames go together (drop_while_down needs them)"
            ),
        ),
    }
    let rehit_start: Option<u8> = f.parsed("rehit_start", errors);
    let rehit_every: Option<u8> = f.parsed("rehit_every", errors);
    match (rehit_start, rehit_every) {
        (Some(a), Some(b)) => m.rehit = Some((a, b)),
        (None, None) => {}
        _ => err(
            errors,
            block.line,
            format!("{context}: rehit_start and rehit_every go together"),
        ),
    }
    f.finish(errors);

    only_known(
        block,
        &[
            "hitbox",
            "motion",
            "projectile",
            "reflector",
            "counter",
            "script",
            "projectile_script",
            "part_intangible",
            "armor",
            "air",
            "charge_add",
            "carry",
            "encase",
            "version",
            "balloons",
            "pocket",
            "unpocket",
            "toss",
            "latch",
            "flag_version",
            "curse",
            "tether",
            "orbs",
            "lerp",
            "probe",
        ],
        context,
        errors,
    );
    // Sim 128: orbs a beam puts on a fighter (E59), a counter answer that grows with its catch, a ground probe.
    if let Some(b) = at_most_one(children(block, "orbs"), "orbs", context, errors) {
        let ctx = format!("the orbs of {context}");
        only_known(b, &["orbit_hit", "strike_hit"], &ctx, errors);
        let hit = |name: &str, errors: &mut Vec<String>| {
            let hctx = format!("the {name} of {ctx}");
            match at_most_one(children(b, name), name, &ctx, errors) {
                Some(hb) => {
                    only_known(hb, &["hitbox"], &hctx, errors);
                    match at_most_one(children(hb, "hitbox"), "hitbox", &hctx, errors) {
                        Some(h) => read_hitbox(h, &hctx, errors),
                        None => {
                            err(errors, hb.line, format!("{hctx} needs a hitbox"));
                            missing_hitbox()
                        }
                    }
                }
                None => {
                    err(errors, b.line, format!("{ctx} needs an {name}"));
                    missing_hitbox()
                }
            }
        };
        let orbit_hit = hit("orbit_hit", errors);
        let strike_hit = hit("strike_hit", errors);
        let mut of = Fields::new(b, &ctx, errors);
        m.orbs = Some(Orbs {
            count: of.need("count", None, errors),
            more_at: [
                of.or("more_at_1", 255, errors),
                of.or("more_at_2", 255, errors),
            ],
            reflected: of.or("reflected", 1, errors),
            per_target: of.need("per_target", None, errors),
            per_owner: of.need("per_owner", None, errors),
            wait: of.need("wait", None, errors),
            extend: of.or("extend", 0, errors),
            extend_to: of.or("extend_to", 0, errors),
            full: of.or("full", 1, errors),
            cap: of.need("cap", None, errors),
            radius: of.need("radius", None, errors),
            period: of.need("period", None, errors),
            orbit_hit,
            orbit_hits: of.need("orbit_hits", None, errors),
            orbit_rehit: of.or("orbit_rehit", 0, errors),
            strike: of.need("strike", None, errors),
            strike_hit,
            loose_speed: of.need("loose_speed", None, errors),
            loose_life: of.need("loose_life", None, errors),
        });
        of.finish(errors);
    }
    if let Some(b) = at_most_one(children(block, "lerp"), "lerp", context, errors) {
        let ctx = format!("the lerp of {context}");
        only_known(b, &["phase"], &ctx, errors);
        let phases = read_phases(b, &ctx, errors);
        let mut lf = Fields::new(b, &ctx, errors);
        m.lerp = Some(Lerp {
            full: lf.need("full", None, errors),
            phases,
        });
        lf.finish(errors);
    }
    if let Some(b) = at_most_one(children(block, "probe"), "probe", context, errors) {
        let ctx = format!("the probe of {context}");
        let mut pf = Fields::new(b, &ctx, errors);
        let mut miss = 0;
        match pf.raw("miss") {
            Some((key, line)) => match MoveId::from_key(key) {
                Some(id) => miss = id as u8,
                None => err(
                    errors,
                    line,
                    format!("`miss` must be a move name, not `{key}`"),
                ),
            },
            None => err(errors, b.line, format!("{ctx} is missing `miss`")),
        }
        m.probe = Some(GroundProbe {
            frame: pf.need("frame", None, errors),
            from: (
                pf.need("from_x", None, errors),
                pf.need("from_y", None, errors),
            ),
            to: (pf.need("to_x", None, errors), pf.need("to_y", None, errors)),
            miss,
        });
        pf.finish(errors);
    }
    // Sim 121: damage over time put on a fighter by the move's projectile (E54).
    if let Some(b) = at_most_one(children(block, "curse"), "curse", context, errors) {
        let ctx = format!("the curse of {context}");
        let mut cf = Fields::new(b, &ctx, errors);
        m.curse = Some(Curse {
            damage: cf.need("damage", None, errors),
            every: cf.need("every", None, errors),
            frames: cf.need("frames", None, errors),
        });
        cf.finish(errors);
    }
    // Sim 123: a tether (E56) and a dodge that stales like a roll.
    if let Some(b) = at_most_one(children(block, "tether"), "tether", context, errors) {
        let ctx = format!("the tether of {context}");
        let mut tf = Fields::new(b, &ctx, errors);
        m.tether = Some(Tether {
            from: tf.need("from", None, errors),
            to: tf.need("to", None, errors),
            ahead: tf.need("ahead", None, errors),
            behind: tf.or("behind", Fx::ZERO, errors),
            up: tf.need("up", None, errors),
            down: tf.or("down", Fx::ZERO, errors),
            hang: tf.or("hang", 0, errors),
            reel: tf.need("reel", None, errors),
            drag: tf.or("drag", 0, errors),
            limit: tf.need("limit", None, errors),
            // Sim 126 (owner decision 86): a press of special while hanging reels in.
            reel_press: tf.or("reel_press", false, errors),
        });
        tf.finish(errors);
    }
    // Sim 119: a latch (E49) and the versions picked by the once-per-airtime flags and marks (owner decision 75).
    if let Some(b) = at_most_one(children(block, "latch"), "latch", context, errors) {
        let ctx = format!("the latch of {context}");
        let mut lf = Fields::new(b, &ctx, errors);
        m.latch = Some(Latch {
            x: lf.or("x", Fx::ZERO, errors),
            height: lf.need("height", None, errors),
            clock_num: lf.or("clock_num", 0, errors),
            clock_den: lf.or("clock_den", 1, errors),
        });
        lf.finish(errors);
    }
    m.flag_versions = children(block, "flag_version")
        .into_iter()
        .map(|b| {
            let ctx = format!("a flag version of {context}");
            let mut f = Fields::new(b, &ctx, errors);
            let flags: u8 = f.or("flags", 0, errors);
            let marks: u8 = f.or("marks", 0, errors);
            let mut slot = None;
            if let Some((key, line)) = f.raw("move") {
                match MoveId::from_key(key) {
                    Some(id) => slot = Some(id as u8),
                    None => err(
                        errors,
                        line,
                        format!("`move` must be a move name, not `{key}`"),
                    ),
                }
            }
            f.finish(errors);
            FlagVersion { flags, marks, slot }
        })
        .collect();
    // Sim 117: a move's own toss of the item in hand (E47).
    if let Some(b) = at_most_one(children(block, "toss"), "toss", context, errors) {
        let ctx = format!("the toss of {context}");
        let mut tf = Fields::new(b, &ctx, errors);
        let speed: Fx = tf.need("speed", None, errors);
        m.toss = Some(Toss {
            frame: tf.need("frame", None, errors),
            again: tf.or("again", 255, errors),
            angle: tf.need("angle", None, errors),
            speed,
            smash_speed: tf.or("smash_speed", speed, errors),
            wait: tf.or("wait", 0, errors),
        });
        tf.finish(errors);
    }
    // Sim 115: the pocket and its release (E42).
    if let Some(b) = at_most_one(children(block, "pocket"), "pocket", context, errors) {
        let ctx = format!("the pocket of {context}");
        let mut pf = Fields::new(b, &ctx, errors);
        m.pocket = Some(PocketCatch {
            start: pf.need("start", None, errors),
            end: pf.need("end", None, errors),
            x: pf.or("x", Fx::ZERO, errors),
            y: pf.or("y", Fx::ZERO, errors),
            radius: pf.need("radius", None, errors),
        });
        pf.finish(errors);
    }
    if let Some(b) = at_most_one(children(block, "unpocket"), "unpocket", context, errors) {
        let ctx = format!("the release of {context}");
        let mut pf = Fields::new(b, &ctx, errors);
        m.unpocket = Some(PocketRelease {
            frame: pf.need("frame", None, errors),
            x: pf.or("x", Fx::ZERO, errors),
            y: pf.or("y", Fx::ZERO, errors),
            power: pf.or("power", 100, errors),
        });
        pf.finish(errors);
    }
    // Sim 113: random versions (E39) and balloons (E41).
    m.versions = children(block, "version")
        .into_iter()
        .map(|b| {
            let ctx = format!("a version of {context}");
            let mut f = Fields::new(b, &ctx, errors);
            let chance: u8 = f.need("chance", None, errors);
            let mut slot = 0;
            match f.raw("move") {
                Some((key, line)) => match MoveId::from_key(key) {
                    Some(id) => slot = id as u8,
                    None => err(
                        errors,
                        line,
                        format!("`move` must be a move name, not `{key}`"),
                    ),
                },
                None => err(errors, b.line, format!("{ctx} needs a `move`")),
            }
            f.finish(errors);
            (chance, slot)
        })
        .collect();
    if let Some(b) = at_most_one(children(block, "balloons"), "balloons", context, errors) {
        m.balloons = Some(read_balloons(
            b,
            &format!("the balloons of {context}"),
            errors,
        ));
    }
    if let Some(b) = at_most_one(children(block, "encase"), "encase", context, errors) {
        m.encase = Some(read_encase(b, &format!("the encase of {context}"), errors));
    }
    if let Some(b) = at_most_one(children(block, "carry"), "carry", context, errors) {
        m.carry = Some(read_carry(b, &format!("the carry of {context}"), errors));
    }
    if let Some(b) = at_most_one(children(block, "charge_add"), "charge_add", context, errors) {
        let ctx = format!("the charge_add of {context}");
        let mut f = Fields::new(b, &ctx, errors);
        m.charge_add = Some(ChargeAdd {
            full: f.need("full", None, errors),
            add: f.need("add", None, errors),
            floor: f.or("floor", false, errors),
        });
        f.finish(errors);
    }
    m.air_blocks = children(block, "air")
        .into_iter()
        .map(|b| {
            let ctx = format!("an air block of {context}");
            let mut f = Fields::new(b, &ctx, errors);
            let a = AirBlock {
                from: f.need("from", None, errors),
                to: f.need("to", None, errors),
                gravity: f.parsed("gravity", errors),
                max_fall: f.parsed("max_fall", errors),
                friction: f.parsed("friction", errors),
                drift: f.or("drift", 100, errors),
                accel: f.or("accel", 100, errors),
                speed: f.parsed("speed", errors),
                accel_stick: f.parsed("accel_stick", errors),
            };
            f.finish(errors);
            a
        })
        .collect();
    if let Some(b) = at_most_one(children(block, "armor"), "armor", context, errors) {
        m.armor = Some(read_armor(b, &format!("the armor of {context}"), errors));
    }
    m.part_intangible = children(block, "part_intangible")
        .into_iter()
        .map(|b| {
            let ctx = format!("a part_intangible of {context}");
            let mut f = Fields::new(b, &ctx, errors);
            let mut parts = 0;
            for (name, bit) in [
                ("low", PART_LOW),
                ("middle", PART_MIDDLE),
                ("high", PART_HIGH),
                ("limb", PART_LIMB),
            ] {
                if f.or(name, false, errors) {
                    parts |= bit;
                }
            }
            // (Sim 123: a window for one mode, as a hitbox's.)
            let mode = match f.raw("mode") {
                None | Some(("any", _)) => MODE_ANY,
                Some(("plain", _)) => MODE_PLAIN,
                Some(("powered", _)) => MODE_POWERED,
                Some((other, line)) => {
                    err(
                        errors,
                        line,
                        format!("`mode` must be any, plain or powered, not `{other}`"),
                    );
                    MODE_ANY
                }
            };
            let w = PartIntangible {
                start: f.need("start", None, errors),
                end: f.need("end", None, errors),
                parts,
                mode,
            };
            f.finish(errors);
            w
        })
        .collect();
    m.hitboxes = children(block, "hitbox")
        .into_iter()
        .map(|b| read_hitbox(b, &format!("a hitbox of {context}"), errors))
        .collect();
    m.motion = children(block, "motion")
        .into_iter()
        .map(|b| {
            let ctx = format!("a motion of {context}");
            let mut f = Fields::new(b, &ctx, errors);
            let mo = Motion {
                start: f.need("start", None, errors),
                end: f.need("end", None, errors),
                vx: f.need("vx", None, errors),
                vy: f.need("vy", None, errors),
            };
            f.finish(errors);
            mo
        })
        .collect();
    if let Some(b) = at_most_one(children(block, "projectile"), "projectile", context, errors) {
        let ctx = format!("the projectile of {context}");
        only_known(
            b,
            &["hitbox", "phase", "burst", "charged", "on_clank"],
            &ctx,
            errors,
        );
        let mut f = Fields::new(b, &ctx, errors);
        let hitbox = match at_most_one(children(b, "hitbox"), "hitbox", &ctx, errors) {
            Some(h) => read_hitbox(h, &ctx, errors),
            None => {
                err(errors, b.line, format!("{ctx} needs a hitbox"));
                missing_hitbox()
            }
        };
        let none = ProjectileArc::NONE;
        let arc = ProjectileArc {
            angle: f.or("angle", none.angle, errors),
            gravity: f.or("gravity", none.gravity, errors),
            max_fall: f.or("max_fall", none.max_fall, errors),
            bounces: f.or("bounces", none.bounces, errors),
            bounce_min: f.or("bounce_min", none.bounce_min, errors),
            bounce_max: f.or("bounce_max", none.bounce_max, errors),
            bounce_keep: f.or("bounce_keep", none.bounce_keep, errors),
            bounce_keep_y: f.or("bounce_keep_y", none.bounce_keep_y, errors),
            bounce_max_fall: f.or("bounce_max_fall", none.bounce_max_fall, errors),
            drag: f.or("drag", none.drag, errors),
            lodge: f.or("lodge", none.lodge, errors),
            lodge_angle: f.or("lodge_angle", none.lodge_angle, errors),
            // (Sim 111, E4: a lodged shot becomes this item of the weapon.)
            lodge_item: f.parsed("lodge_item", errors),
        };
        // Sim 99: a charge-scaled shot (E35) and a thrown tool's rules (E3d).
        let charged = at_most_one(children(b, "charged"), "charged", &ctx, errors).map(|cb| {
            let cctx = format!("the charge of {ctx}");
            let mut cf = Fields::new(cb, &cctx, errors);
            let c = ChargedShot {
                full: cf.need("full", None, errors),
                snap: cf.or("snap", 0, errors),
                damage_to: cf.need("damage_to", None, errors),
                speed_to: cf.need("speed_to", None, errors),
                life_to: cf.need("life_to", None, errors),
                size_from: cf.or("size_from", 100, errors),
                size_to: cf.or("size_to", 100, errors),
                floor: cf.or("floor", false, errors),
                times: cf.or("times", 100, errors),
                power_to: cf.or("power_to", 100, errors),
                // Sim 128: the charged damage on every hit (the beam's rings).
                every_hit: cf.or("every_hit", false, errors),
            };
            cf.finish(errors);
            c
        });
        let on_clank = at_most_one(children(b, "on_clank"), "on_clank", &ctx, errors).map(|cb| {
            let cctx = format!("the clank fall of {ctx}");
            let mut cf = Fields::new(cb, &cctx, errors);
            let fall = Fall {
                life: cf.need("life", None, errors),
                vx: cf.or("vx", Fx::ZERO, errors),
                vy: cf.or("vy", Fx::ZERO, errors),
                gravity: cf.or("gravity", Fx::ZERO, errors),
                max_fall: cf.or("max_fall", Fx::ZERO, errors),
            };
            cf.finish(errors);
            fall
        });
        let mut catch_move = None;
        if let Some((key, line)) = f.raw("catch_move") {
            match MoveId::from_key(key) {
                Some(id) => catch_move = Some(id as u8),
                None => err(
                    errors,
                    line,
                    format!("`catch_move` must be a move name, not `{key}`"),
                ),
            }
        }
        // (Sim 115: whether a pocket can take it, E42, and the water it gives a growing piece, E43.)
        let pocket = match f.raw("pocket") {
            None => POCKET_YES,
            Some(("yes", _)) => POCKET_YES,
            Some(("never", _)) => POCKET_NEVER,
            Some(("frees", _)) => POCKET_FREES,
            Some((key, line)) => {
                err(
                    errors,
                    line,
                    format!("`pocket` must be yes, never or frees, not `{key}`"),
                );
                POCKET_YES
            }
        };
        let tool = Tool {
            kind: f.or("kind", 0, errors),
            limit: f.or("limit", 0, errors),
            catch_from: f.or("catch_from", 255, errors),
            catch_move,
            on_clank,
            pocket,
            water: f.or("water", 0, errors),
        };
        let phases = read_phases(b, &ctx, errors);
        let burst = at_most_one(children(b, "burst"), "burst", &ctx, errors).map(|bb| {
            let bctx = format!("the burst of {ctx}");
            only_known(bb, &["phase"], &bctx, errors);
            let mut bf = Fields::new(bb, &bctx, errors);
            let burst = Burst {
                life: bf.need("life", None, errors),
                rehit: bf.or("rehit", 0, errors),
                on_wall: bf.or("on_wall", false, errors),
                on_ground: bf.or("on_ground", false, errors),
                on_expire: bf.or("on_expire", false, errors),
                keep_hits: bf.or("keep_hits", false, errors),
                on_hit: bf.or("on_hit", true, errors),
                on_shield: bf.or("on_shield", true, errors),
                y: bf.or("y", Fx::ZERO, errors),
                phases: read_phases(bb, &bctx, errors),
            };
            bf.finish(errors);
            burst
        });
        let spawn = ProjectileSpawn {
            frame: f.need("frame", None, errors),
            x: f.need("x", None, errors),
            y: f.need("y", None, errors),
            speed: f.need("speed", None, errors),
            life: f.need("life", None, errors),
            hitbox,
            end_damage: f.need("end_damage", None, errors),
            arc,
            phases,
            pierce: f.or("pierce", false, errors),
            rehit: f.or("rehit", 0, errors),
            burst,
            charged,
            tool,
        };
        f.finish(errors);
        m.projectile = Some(spawn);
    }
    if let Some(b) = at_most_one(children(block, "reflector"), "reflector", context, errors) {
        let ctx = format!("the reflector of {context}");
        let mut f = Fields::new(b, &ctx, errors);
        let r = Reflector {
            start: f.need("start", None, errors),
            end: f.need("end", None, errors),
            x: f.need("x", None, errors),
            y: f.need("y", None, errors),
            radius: f.need("radius", None, errors),
            damage_percent: f.need("damage_percent", None, errors),
            speed_percent: f.need("speed_percent", None, errors),
            reverse_vertical: f.or("reverse_vertical", false, errors),
            life_percent: f.or("life_percent", 100, errors),
            max_damage: f.or("max_damage", Fx::ZERO, errors),
            detect_start: f.or("detect_start", 0, errors),
            detect_end: f.or("detect_end", 255, errors),
            detect_x: f.or("detect_x", Fx::ZERO, errors),
            detect_y: f.or("detect_y", Fx::ZERO, errors),
            detect_radius: f.or("detect_radius", Fx::ZERO, errors),
            detect_then: None,
            half_height: f.or("half_height", Fx::ZERO, errors),
            body_start: f.or("body_start", 0, errors),
            body_end: f.or("body_end", 0, errors),
            body_x: f.or("body_x", Fx::ZERO, errors),
            body_y: f.or("body_y", Fx::ZERO, errors),
            body_radius: f.or("body_radius", Fx::ZERO, errors),
        };
        let mut r = r;
        if let Some((key, line)) = f.raw("detect_then") {
            match MoveId::from_key(key) {
                Some(id) => r.detect_then = Some(id as u8),
                None => err(
                    errors,
                    line,
                    format!("`detect_then` must be a move name, not `{key}`"),
                ),
            }
        }
        f.finish(errors);
        m.reflector = Some(r);
    }
    if let Some(b) = at_most_one(children(block, "counter"), "counter", context, errors) {
        let ctx = format!("the counter of {context}");
        let mut f = Fields::new(b, &ctx, errors);
        let then = match f.raw("then") {
            Some((key, line)) => match MoveId::from_key(key) {
                Some(id) => id as u8,
                None => {
                    err(
                        errors,
                        line,
                        format!("`then` must be a move name, not `{key}`"),
                    );
                    0
                }
            },
            None => {
                err(errors, b.line, format!("{ctx} is missing `then`"));
                0
            }
        };
        let c = Counter {
            start: f.need("start", None, errors),
            end: f.need("end", None, errors),
            then,
            percent: f.need("percent", None, errors),
            min_damage: f.need("min_damage", None, errors),
            max_damage: f.or("max_damage", Fx::ZERO, errors),
            melee_only: f.or("melee_only", false, errors),
            bubble: None,
            // Sim 128: the barrier's break and its still answer.
            break_above: f.or("break_above", Fx::ZERO, errors),
            break_move: 0,
            still: f.or("still", false, errors),
        };
        let mut c = c;
        if let Some((key, line)) = f.raw("break_move") {
            match MoveId::from_key(key) {
                Some(id) => c.break_move = id as u8,
                None => err(
                    errors,
                    line,
                    format!("`break_move` must be a move name, not `{key}`"),
                ),
            }
        }
        // The counter bubble: present when it has a radius (none: the body's hurtboxes catch, as older content means).
        let bubble = CounterBubble {
            x: f.or("bubble_x", Fx::ZERO, errors),
            y: f.or("bubble_y", Fx::ZERO, errors),
            half_height: f.or("bubble_half_height", Fx::ZERO, errors),
            radius: f.or("bubble_radius", Fx::ZERO, errors),
        };
        let c = Counter {
            bubble: (bubble.radius != Fx::ZERO).then_some(bubble),
            ..c
        };
        f.finish(errors);
        m.counter = Some(c);
    }
    if let Some((text, line)) = raw_script(block, "script", context, errors) {
        m.script = compile_script(Kind::Fighter, text, line, context, errors);
    }
    if let Some((text, line)) = raw_script(block, "projectile_script", context, errors) {
        m.projectile_script = compile_script(Kind::Projectile, text, line, context, errors);
    }
    m
}

fn write_move(id: MoveId, m: &Move) -> Block {
    let mut b = Block::new("move", Some(id.key()));
    let empty = Move::empty();
    b.field("total_frames", m.total_frames.to_string());
    if m.landing_lag != empty.landing_lag {
        b.field("landing_lag", m.landing_lag.to_string());
    }
    if m.autocancel_before != empty.autocancel_before {
        b.field("autocancel_before", m.autocancel_before.to_string());
    }
    if m.autocancel_after != empty.autocancel_after {
        b.field("autocancel_after", m.autocancel_after.to_string());
    }
    if m.intangible != 0 {
        b.field("intangible", m.intangible.to_string());
    }
    for (name, on) in [
        ("helpless_after", m.helpless_after),
        ("turns_around", m.turns_around),
        ("grabs_ledge", m.grabs_ledge),
        ("land_continue", m.land_continue),
    ] {
        if on {
            b.field(name, "true");
        }
    }
    // The per-move recovery rules, each only when it differs from the default.
    for (name, value, default) in [
        (
            "ledge_front_from",
            m.ledge_front_from,
            empty.ledge_front_from,
        ),
        ("ledge_back_from", m.ledge_back_from, empty.ledge_back_from),
        ("fall_drift", m.fall_drift, empty.fall_drift),
        (
            "fall_landing_lag",
            m.fall_landing_lag,
            empty.fall_landing_lag,
        ),
        ("rise_ground", m.rise_ground, empty.rise_ground),
        ("rise_air", m.rise_air, empty.rise_air),
        ("start_speed", m.start_speed, empty.start_speed),
        ("steer_frame", m.steer_frame, empty.steer_frame),
        ("steer_stick", m.steer_stick, empty.steer_stick),
        ("steer_degrees", m.steer_degrees, empty.steer_degrees),
        ("air_drift", m.air_drift, empty.air_drift),
        ("intangible_from", m.intangible_from, empty.intangible_from),
        (
            "intangible_from_air",
            m.intangible_from_air,
            empty.intangible_from_air,
        ),
        ("fast_fall_from", m.fast_fall_from, empty.fast_fall_from),
        ("lands_from", m.lands_from, empty.lands_from),
        ("start_speed_y", m.start_speed_y, empty.start_speed_y),
        ("motion_drift", m.motion_drift, empty.motion_drift),
        (
            "push_immune_from",
            m.push_immune_from,
            empty.push_immune_from,
        ),
        ("push_immune_to", m.push_immune_to, empty.push_immune_to),
    ] {
        if value != default {
            b.field(name, value.to_string());
        }
    }
    if let Some((frame, stick)) = m.turn_at {
        b.field("turn_frame", frame.to_string());
        b.field("turn_stick", stick.to_string());
    }
    if let Some((x, y)) = m.hold_at {
        b.field("hold_x", x.show());
        b.field("hold_y", y.show());
    }
    for (name, value) in [
        ("air_gravity", m.air_gravity),
        ("air_max_fall", m.air_max_fall),
        ("air_friction", m.air_friction),
        ("air_max_rise", m.air_max_rise),
        ("fall_speed", m.fall_speed),
        ("fall_accel", m.fall_accel),
    ] {
        if let Some(v) = value {
            b.field(name, v.show());
        }
    }
    if m.keep_charge {
        b.field("keep_charge", "true");
    }
    if let Some(c) = m.charge_at {
        b.field("charge_at", c.to_string());
    }
    if let Some(n) = m.next {
        b.field("next", MoveId::from_index(n).key());
        b.field("next_window", m.next_window.to_string());
        if m.next_from != 255 {
            b.field("next_from", m.next_from.to_string());
        }
    }
    if m.cancel_from != 255 {
        b.field("cancel_from", m.cancel_from.to_string());
    }
    if let Some((start, every)) = m.rehit {
        b.field("rehit_start", start.to_string());
        b.field("rehit_every", every.to_string());
    }
    if m.rehit_groups != 0xff {
        b.field("rehit_groups", m.rehit_groups.to_string());
    }
    for (name, slot) in [
        ("landing_move", m.landing_move),
        ("on_search", m.on_search),
        ("air_version", m.air_version),
        ("smash_version", m.smash_version),
        ("catch_move", m.catch_move),
        ("off_edge", m.off_edge),
        ("limit_version", m.limit_version),
        ("on_land", m.on_land),
        ("shed", m.shed),
        ("fall_landing_move", m.fall_landing_move),
    ] {
        if let Some(id) = slot {
            b.field(name, MoveId::from_index(id).key());
        }
    }
    if m.gauge_drain {
        b.field("gauge_drain", "true");
    }
    if let Some((front, back)) = m.ledge_drop {
        b.field("ledge_drop_front", front.show());
        b.field("ledge_drop_back", back.show());
    }
    if let Some((id, until)) = m.late_next {
        b.field("late_next", MoveId::from_index(id).key());
        b.field("late_until", until.to_string());
    }
    if m.ledge_falling_only {
        b.field("ledge_falling_only", "true");
    }
    if let Some(d) = m.drop_through {
        b.field("drop_from", d.from.to_string());
        b.field("drop_frames", d.frames.to_string());
        if d.while_down {
            b.field("drop_while_down", "true");
        }
    }
    if m.counter_strike {
        b.field("counter_strike", "true");
    }
    if m.charge_bonus != 0 {
        b.field("charge_bonus", m.charge_bonus.to_string());
    }
    for hb in &m.hitboxes {
        b.push(write_hitbox(hb));
    }
    for w in &m.part_intangible {
        let mut c = Block::new("part_intangible", None);
        c.field("start", w.start.to_string());
        c.field("end", w.end.to_string());
        for (name, bit) in [
            ("low", PART_LOW),
            ("middle", PART_MIDDLE),
            ("high", PART_HIGH),
            ("limb", PART_LIMB),
        ] {
            if w.parts & bit != 0 {
                c.field(name, "true");
            }
        }
        match w.mode {
            MODE_PLAIN => c.field("mode", "plain"),
            MODE_POWERED => c.field("mode", "powered"),
            _ => {}
        }
        b.push(c);
    }
    for a in &m.air_blocks {
        let mut c = Block::new("air", None);
        c.field("from", a.from.to_string());
        c.field("to", a.to.to_string());
        for (name, v) in [
            ("gravity", a.gravity),
            ("max_fall", a.max_fall),
            ("friction", a.friction),
        ] {
            if let Some(v) = v {
                c.field(name, v.show());
            }
        }
        if a.drift != 100 {
            c.field("drift", a.drift.to_string());
        }
        if a.accel != 100 {
            c.field("accel", a.accel.to_string());
        }
        for (name, v) in [("speed", a.speed), ("accel_stick", a.accel_stick)] {
            if let Some(v) = v {
                c.field(name, v.show());
            }
        }
        b.push(c);
    }
    if let Some(a) = &m.charge_add {
        let mut c = Block::new("charge_add", None);
        c.field("full", a.full.to_string());
        c.field("add", a.add.show());
        if a.floor {
            c.field("floor", "true");
        }
        b.push(c);
    }
    if let Some(a) = &m.armor {
        b.push(write_armor("armor", a));
    }
    if let Some(c) = &m.carry {
        b.push(write_carry(c));
    }
    if let Some(e) = &m.encase {
        b.push(write_encase(e));
    }
    for (chance, slot) in &m.versions {
        let mut c = Block::new("version", None);
        c.field("chance", chance.to_string());
        c.field("move", MoveId::from_index(*slot).key());
        b.push(c);
    }
    if let Some(bl) = &m.balloons {
        b.push(write_balloons(bl));
    }
    if let Some(pc) = &m.pocket {
        let mut c = Block::new("pocket", None);
        c.field("start", pc.start.to_string());
        c.field("end", pc.end.to_string());
        c.field("x", pc.x.show());
        c.field("y", pc.y.show());
        c.field("radius", pc.radius.show());
        b.push(c);
    }
    if let Some(r) = &m.unpocket {
        let mut c = Block::new("unpocket", None);
        c.field("frame", r.frame.to_string());
        c.field("x", r.x.show());
        c.field("y", r.y.show());
        c.field("power", r.power.to_string());
        b.push(c);
    }
    if m.chops {
        b.field("chops", "true");
    }
    if m.self_damage != Fx::ZERO {
        b.field("self_damage", m.self_damage.show());
    }
    if m.armor_damage != 1000 {
        b.field("armor_damage", m.armor_damage.to_string());
    }
    if m.armor_hitlag != 100 {
        b.field("armor_hitlag", m.armor_hitlag.to_string());
    }
    if m.gauge_gain != 1000 {
        b.field("gauge_gain", m.gauge_gain.to_string());
    }
    if let Some(c) = &m.curse {
        let mut cb = Block::new("curse", None);
        cb.field("damage", c.damage.show());
        cb.field("every", c.every.to_string());
        cb.field("frames", c.frames.to_string());
        b.push(cb);
    }
    if let Some(o) = &m.orbs {
        let mut ob = Block::new("orbs", None);
        ob.field("count", o.count.to_string());
        if o.more_at[0] != 255 {
            ob.field("more_at_1", o.more_at[0].to_string());
        }
        if o.more_at[1] != 255 {
            ob.field("more_at_2", o.more_at[1].to_string());
        }
        ob.field("reflected", o.reflected.to_string());
        ob.field("per_target", o.per_target.to_string());
        ob.field("per_owner", o.per_owner.to_string());
        ob.field("wait", o.wait.to_string());
        ob.field("extend", o.extend.to_string());
        ob.field("extend_to", o.extend_to.to_string());
        ob.field("full", o.full.to_string());
        ob.field("cap", o.cap.to_string());
        ob.field("radius", o.radius.show());
        ob.field("period", o.period.to_string());
        ob.field("orbit_hits", o.orbit_hits.to_string());
        ob.field("orbit_rehit", o.orbit_rehit.to_string());
        ob.field("strike", o.strike.to_string());
        ob.field("loose_speed", o.loose_speed.show());
        ob.field("loose_life", o.loose_life.to_string());
        for (name, hb) in [("orbit_hit", &o.orbit_hit), ("strike_hit", &o.strike_hit)] {
            let mut hb_block = Block::new(name, None);
            hb_block.push(write_hitbox(hb));
            ob.push(hb_block);
        }
        b.push(ob);
    }
    if let Some(l) = &m.lerp {
        let mut lb = Block::new("lerp", None);
        lb.field("full", l.full.show());
        write_phases(&mut lb, &l.phases);
        b.push(lb);
    }
    if let Some(p) = &m.probe {
        let mut pb = Block::new("probe", None);
        pb.field("frame", p.frame.to_string());
        pb.field("from_x", p.from.0.show());
        pb.field("from_y", p.from.1.show());
        pb.field("to_x", p.to.0.show());
        pb.field("to_y", p.to.1.show());
        pb.field("miss", MoveId::from_index(p.miss).key());
        b.push(pb);
    }
    if m.dodge_stale {
        b.field("dodge_stale", "true");
    }
    match m.stale_stretch {
        STRETCH_ROLL => b.field("stale_stretch", "roll"),
        STRETCH_BACK_ROLL => b.field("stale_stretch", "back_roll"),
        STRETCH_SPOT => b.field("stale_stretch", "spot"),
        _ => {}
    }
    if m.shield_end != 0 {
        b.field("shield_end", m.shield_end.to_string());
    }
    if m.edge_stop {
        b.field("edge_stop", "true");
    }
    match m.armor_mode {
        MODE_PLAIN => b.field("armor_mode", "plain"),
        MODE_POWERED => b.field("armor_mode", "powered"),
        _ => {}
    }
    if let Some(t) = &m.tether {
        let mut c = Block::new("tether", None);
        c.field("from", t.from.to_string());
        c.field("to", t.to.to_string());
        c.field("ahead", t.ahead.show());
        if t.behind != Fx::ZERO {
            c.field("behind", t.behind.show());
        }
        c.field("up", t.up.show());
        if t.down != Fx::ZERO {
            c.field("down", t.down.show());
        }
        if t.hang != 0 {
            c.field("hang", t.hang.to_string());
        }
        c.field("reel", t.reel.to_string());
        if t.drag != 0 {
            c.field("drag", t.drag.to_string());
        }
        c.field("limit", t.limit.to_string());
        if t.reel_press {
            c.field("reel_press", "true");
        }
        b.push(c);
    }
    if let Some(l) = &m.latch {
        let mut c = Block::new("latch", None);
        c.field("x", l.x.show());
        c.field("height", l.height.to_string());
        if l.clock_num != 0 {
            c.field("clock_num", l.clock_num.to_string());
            c.field("clock_den", l.clock_den.to_string());
        }
        b.push(c);
    }
    for v in &m.flag_versions {
        let mut c = Block::new("flag_version", None);
        if v.flags != 0 {
            c.field("flags", v.flags.to_string());
        }
        if v.marks != 0 {
            c.field("marks", v.marks.to_string());
        }
        if let Some(slot) = v.slot {
            c.field("move", MoveId::from_index(slot).key());
        }
        b.push(c);
    }
    if let Some(t) = &m.toss {
        let mut c = Block::new("toss", None);
        c.field("frame", t.frame.to_string());
        if t.again != 255 {
            c.field("again", t.again.to_string());
        }
        c.field("angle", t.angle.to_string());
        c.field("speed", t.speed.show());
        c.field("smash_speed", t.smash_speed.show());
        if t.wait != 0 {
            c.field("wait", t.wait.to_string());
        }
        b.push(c);
    }
    if let Some((frame, item)) = m.pull {
        b.field("pull_frame", frame.to_string());
        b.field("pull_item", item.to_string());
    }
    if m.spend_item != 255 {
        b.field("spend_item", m.spend_item.to_string());
    }
    if let Some(c) = &m.counter {
        let mut cb = Block::new("counter", None);
        cb.field("start", c.start.to_string());
        cb.field("end", c.end.to_string());
        cb.field("then", MoveId::from_index(c.then).key());
        cb.field("percent", c.percent.to_string());
        cb.field("min_damage", c.min_damage.show());
        if c.max_damage != Fx::ZERO {
            cb.field("max_damage", c.max_damage.show());
        }
        if c.melee_only {
            cb.field("melee_only", "true");
        }
        if let Some(bb) = &c.bubble {
            cb.field("bubble_x", bb.x.show());
            cb.field("bubble_y", bb.y.show());
            if bb.half_height != Fx::ZERO {
                cb.field("bubble_half_height", bb.half_height.show());
            }
            cb.field("bubble_radius", bb.radius.show());
        }
        if c.break_above != Fx::ZERO {
            cb.field("break_above", c.break_above.show());
            cb.field("break_move", MoveId::from_index(c.break_move).key());
        }
        if c.still {
            cb.field("still", "true");
        }
        b.push(cb);
    }
    for mo in &m.motion {
        let mut c = Block::new("motion", None);
        c.field("start", mo.start.to_string());
        c.field("end", mo.end.to_string());
        c.field("vx", mo.vx.show());
        c.field("vy", mo.vy.show());
        b.push(c);
    }
    if let Some(p) = &m.projectile {
        let mut c = Block::new("projectile", None);
        c.field("frame", p.frame.to_string());
        c.field("x", p.x.show());
        c.field("y", p.y.show());
        c.field("speed", p.speed.show());
        c.field("life", p.life.to_string());
        c.field("end_damage", p.end_damage.show());
        let none = ProjectileArc::NONE;
        let a = &p.arc;
        if a.angle != none.angle {
            c.field("angle", a.angle.to_string());
        }
        for (name, v, d) in [
            ("gravity", a.gravity, none.gravity),
            ("max_fall", a.max_fall, none.max_fall),
            ("bounce_min", a.bounce_min, none.bounce_min),
            ("bounce_max", a.bounce_max, none.bounce_max),
            ("bounce_max_fall", a.bounce_max_fall, none.bounce_max_fall),
            ("drag", a.drag, none.drag),
        ] {
            if v != d {
                c.field(name, v.show());
            }
        }
        for (name, v, d) in [
            ("lodge", a.lodge, none.lodge),
            ("lodge_angle", a.lodge_angle, none.lodge_angle),
            ("kind", p.tool.kind, Tool::NONE.kind),
            ("limit", p.tool.limit, Tool::NONE.limit),
            ("catch_from", p.tool.catch_from, Tool::NONE.catch_from),
            ("water", p.tool.water, Tool::NONE.water),
        ] {
            if v != d {
                c.field(name, v.to_string());
            }
        }
        match p.tool.pocket {
            POCKET_NEVER => c.field("pocket", "never"),
            POCKET_FREES => c.field("pocket", "frees"),
            _ => {}
        }
        if let Some(id) = p.tool.catch_move {
            c.field("catch_move", MoveId::from_index(id).key());
        }
        if let Some(i) = a.lodge_item {
            c.field("lodge_item", i.to_string());
        }
        if a.bounces != none.bounces {
            c.field("bounces", a.bounces.to_string());
        }
        if a.bounce_keep != none.bounce_keep {
            c.field("bounce_keep", a.bounce_keep.to_string());
        }
        if a.bounce_keep_y != none.bounce_keep_y {
            c.field("bounce_keep_y", a.bounce_keep_y.to_string());
        }
        if p.pierce {
            c.field("pierce", "true");
        }
        if p.rehit != 0 {
            c.field("rehit", p.rehit.to_string());
        }
        c.push(write_hitbox(&p.hitbox));
        write_phases(&mut c, &p.phases);
        if let Some(bu) = &p.burst {
            let mut bb = Block::new("burst", None);
            bb.field("life", bu.life.to_string());
            if bu.rehit != 0 {
                bb.field("rehit", bu.rehit.to_string());
            }
            for (name, on) in [
                ("on_wall", bu.on_wall),
                ("on_ground", bu.on_ground),
                ("on_expire", bu.on_expire),
                ("keep_hits", bu.keep_hits),
            ] {
                if on {
                    bb.field(name, "true");
                }
            }
            if !bu.on_hit {
                bb.field("on_hit", "false");
            }
            if !bu.on_shield {
                bb.field("on_shield", "false");
            }
            if bu.y != Fx::ZERO {
                bb.field("y", bu.y.show());
            }
            write_phases(&mut bb, &bu.phases);
            c.push(bb);
        }
        if let Some(ch) = &p.charged {
            let mut cb = Block::new("charged", None);
            cb.field("full", ch.full.to_string());
            if ch.snap != 0 {
                cb.field("snap", ch.snap.to_string());
            }
            cb.field("damage_to", ch.damage_to.show());
            cb.field("speed_to", ch.speed_to.show());
            cb.field("life_to", ch.life_to.to_string());
            if ch.size_from != 100 {
                cb.field("size_from", ch.size_from.to_string());
            }
            if ch.size_to != 100 {
                cb.field("size_to", ch.size_to.to_string());
            }
            if ch.floor {
                cb.field("floor", "true");
            }
            if ch.times != 100 {
                cb.field("times", ch.times.to_string());
            }
            if ch.power_to != 100 {
                cb.field("power_to", ch.power_to.to_string());
            }
            if ch.every_hit {
                cb.field("every_hit", "true");
            }
            c.push(cb);
        }
        if let Some(fall) = &p.tool.on_clank {
            let mut fb = Block::new("on_clank", None);
            fb.field("life", fall.life.to_string());
            for (name, v) in [
                ("vx", fall.vx),
                ("vy", fall.vy),
                ("gravity", fall.gravity),
                ("max_fall", fall.max_fall),
            ] {
                if v != Fx::ZERO {
                    fb.field(name, v.show());
                }
            }
            c.push(fb);
        }
        b.push(c);
    }
    if let Some(r) = &m.reflector {
        let mut c = Block::new("reflector", None);
        c.field("start", r.start.to_string());
        c.field("end", r.end.to_string());
        c.field("x", r.x.show());
        c.field("y", r.y.show());
        c.field("radius", r.radius.show());
        c.field("damage_percent", r.damage_percent.to_string());
        c.field("speed_percent", r.speed_percent.to_string());
        if r.reverse_vertical {
            c.field("reverse_vertical", "true");
        }
        if r.life_percent != 100 {
            c.field("life_percent", r.life_percent.to_string());
        }
        if r.max_damage != Fx::ZERO {
            c.field("max_damage", r.max_damage.show());
        }
        if r.detect_end != 255 {
            c.field("detect_start", r.detect_start.to_string());
            c.field("detect_end", r.detect_end.to_string());
            c.field("detect_x", r.detect_x.show());
            c.field("detect_y", r.detect_y.show());
            c.field("detect_radius", r.detect_radius.show());
        }
        if let Some(id) = r.detect_then {
            c.field("detect_then", MoveId::from_index(id).key());
        }
        if r.half_height != Fx::ZERO {
            c.field("half_height", r.half_height.show());
        }
        if r.body_radius != Fx::ZERO {
            c.field("body_start", r.body_start.to_string());
            c.field("body_end", r.body_end.to_string());
            c.field("body_x", r.body_x.show());
            c.field("body_y", r.body_y.show());
            c.field("body_radius", r.body_radius.show());
        }
        b.push(c);
    }
    for (kind, script) in [
        ("script", &m.script),
        ("projectile_script", &m.projectile_script),
    ] {
        if let Some(p) = script {
            b.items.push(Item::Raw {
                kind: kind.to_string(),
                text: p.source.clone(),
                line: 0,
            });
        }
    }
    b
}

fn read_weapon(block: &Block, earlier: &[(String, Weapon)], errors: &mut Vec<String>) -> Weapon {
    let name = block.name.clone().unwrap_or_default();
    let context = format!("weapon `{name}`");
    let mut f = Fields::new(block, &context, errors);
    let mut moves: Vec<Move> = (0..MoveId::COUNT).map(|_| Move::empty()).collect();
    let mut limbs = false;
    let mut items: Vec<ItemSpec> = Vec::new();
    if let Some((base, line)) = f.raw("inherit") {
        match earlier.iter().find(|(n, _)| n == base) {
            Some((_, w)) => {
                moves = w.moves.clone();
                limbs = w.limbs;
                items = w.items.clone();
            }
            None => err(
                errors,
                line,
                format!("{context} inherits from `{base}`, which is not defined above it"),
            ),
        }
    }
    let limbs = f.or("limbs", limbs, errors);
    f.finish(errors);
    only_known(block, &["move", "item"], &context, errors);
    // Its items (sim 111, E4), in order (their numbers are their places): written ones replace inherited ones.
    let own: Vec<ItemSpec> = children(block, "item")
        .into_iter()
        .enumerate()
        .map(|(i, ib)| read_item(ib, &format!("item {i} of {context}"), errors))
        .collect();
    if !own.is_empty() {
        items = own;
    }
    let mut seen: Vec<MoveId> = Vec::new();
    for mb in children(block, "move") {
        let Some(key) = mb.name.as_deref() else {
            err(errors, mb.line, format!("a move in {context} needs a name"));
            continue;
        };
        let Some(id) = MoveId::from_key(key) else {
            let all: Vec<&str> = MoveId::all().map(MoveId::key).collect();
            err(
                errors,
                mb.line,
                format!(
                    "`{key}` is not a move name (use one of: {})",
                    all.join(", ")
                ),
            );
            continue;
        };
        if seen.contains(&id) {
            err(errors, mb.line, format!("{context} defines `{key}` twice"));
            continue;
        }
        seen.push(id);
        moves[id as usize] = read_move(mb, &format!("{key} of {context}"), errors);
    }
    Weapon {
        moves,
        limbs,
        items,
    }
}

/// An `item { ... }` block of a weapon (E4, sim 111; see `sim_core::moves::ItemSpec`): `kind` and a `hitbox` (its thrown hit) required,
/// every other value optional (default 0, the keeps 100, `throw_mul` 1, `knock_hitlag` 100), and an optional `blast { frames [delay]
/// [hits_owner] hitbox { ... } }`.
fn read_item(b: &Block, ctx: &str, errors: &mut Vec<String>) -> ItemSpec {
    only_known(b, &["hitbox", "blast", "placed", "trap"], ctx, errors);
    let mut f = Fields::new(b, ctx, errors);
    let hit = match at_most_one(children(b, "hitbox"), "hitbox", ctx, errors) {
        Some(h) => read_hitbox(h, ctx, errors),
        None => {
            err(errors, b.line, format!("{ctx} needs a hitbox"));
            missing_hitbox()
        }
    };
    let mut it = ItemSpec::with_hit(hit);
    it.kind = f.need("kind", None, errors);
    for (name, v) in it.fx_fields_mut() {
        if let Some(x) = f.parsed::<Fx>(name, errors) {
            *v = x;
        }
    }
    for (name, v) in it.u8_fields_mut() {
        if let Some(x) = f.parsed::<u8>(name, errors) {
            *v = x;
        }
    }
    it.thrown_life = f.or("thrown_life", 0, errors);
    it.life = f.or("life", 0, errors);
    it.vanish = f.or("vanish", false, errors);
    // Sim 115: pocketing (E42) and placed pieces (E43).
    it.pocketable = f.or("pocketable", true, errors);
    f.finish(errors);
    it.placed = at_most_one(children(b, "placed"), "placed", ctx, errors).map(|pb| {
        let pctx = format!("the placing of {ctx}");
        let mut pf = Fields::new(pb, &pctx, errors);
        let plain = Placed::PLAIN;
        let slot = |name: &str, pf: &mut Fields, errors: &mut Vec<String>| match pf.raw(name) {
            None => 255,
            Some((key, line)) => match MoveId::from_key(key) {
                Some(id) => id as u8,
                None => {
                    err(
                        errors,
                        line,
                        format!("`{name}` must be a move name, not `{key}`"),
                    );
                    255
                }
            },
        };
        let grow_move = slot("grow_move", &mut pf, errors);
        let fall_move = slot("fall_move", &mut pf, errors);
        let placed = Placed {
            x: pf.or("x", plain.x, errors),
            hurt_half_height: pf.or("hurt_half_height", plain.hurt_half_height, errors),
            hitter_lag: pf.or("hitter_lag", plain.hitter_lag, errors),
            water: pf.or("water", plain.water, errors),
            grow_delay: pf.or("grow_delay", plain.grow_delay, errors),
            grow_into: pf.or("grow_into", plain.grow_into, errors),
            grow_move,
            cuts: pf.or("cuts", plain.cuts, errors),
            fall_move,
            chip: pf.or("chip", plain.chip, errors),
            chip_chance: pf.or("chip_chance", plain.chip_chance, errors),
        };
        pf.finish(errors);
        placed
    });
    it.blast = at_most_one(children(b, "blast"), "blast", ctx, errors).map(|bb| {
        let bctx = format!("the blast of {ctx}");
        only_known(bb, &["hitbox"], &bctx, errors);
        let mut bf = Fields::new(bb, &bctx, errors);
        let hit = match at_most_one(children(bb, "hitbox"), "hitbox", &bctx, errors) {
            Some(h) => read_hitbox(h, &bctx, errors),
            None => {
                err(errors, bb.line, format!("{bctx} needs a hitbox"));
                missing_hitbox()
            }
        };
        let blast = ItemBlast {
            hit,
            frames: bf.need("frames", None, errors),
            delay: bf.or("delay", 0, errors),
            hits_owner: bf.or("hits_owner", false, errors),
        };
        bf.finish(errors);
        blast
    });
    // Sim 117: a lying item's trap (E45).
    it.trap = at_most_one(children(b, "trap"), "trap", ctx, errors).map(|tb| {
        let tctx = format!("the trap of {ctx}");
        only_known(tb, &["hitbox"], &tctx, errors);
        let mut tf = Fields::new(tb, &tctx, errors);
        let hit = match at_most_one(children(tb, "hitbox"), "hitbox", &tctx, errors) {
            Some(h) => read_hitbox(h, &tctx, errors),
            None => {
                err(errors, tb.line, format!("{tctx} needs a hitbox"));
                missing_hitbox()
            }
        };
        let trap = Trap {
            hit,
            kick_x: tf.or("kick_x", Fx::ZERO, errors),
            kick_y: tf.or("kick_y", Fx::ZERO, errors),
        };
        tf.finish(errors);
        trap
    });
    it
}

/// Writes an item as an `item { ... }` block (every value, so the file says what the item does).
fn write_item(it: &ItemSpec) -> Block {
    let mut b = Block::new("item", None);
    b.field("kind", it.kind.to_string());
    let mut copy = *it;
    for (name, v) in copy.fx_fields_mut() {
        b.field(name, v.show());
    }
    for (name, v) in copy.u8_fields_mut() {
        b.field(name, v.to_string());
    }
    b.field("thrown_life", it.thrown_life.to_string());
    b.field("life", it.life.to_string());
    if it.vanish {
        b.field("vanish", "true");
    }
    if !it.pocketable {
        b.field("pocketable", "false");
    }
    b.push(write_hitbox(&it.hit));
    if let Some(pl) = &it.placed {
        let mut c = Block::new("placed", None);
        c.field("x", pl.x.show());
        c.field("hurt_half_height", pl.hurt_half_height.show());
        for (name, v) in [
            ("hitter_lag", pl.hitter_lag),
            ("water", pl.water),
            ("grow_delay", pl.grow_delay),
            ("grow_into", pl.grow_into),
            ("cuts", pl.cuts),
        ] {
            c.field(name, v.to_string());
        }
        for (name, v) in [("grow_move", pl.grow_move), ("fall_move", pl.fall_move)] {
            if v != 255 {
                c.field(name, MoveId::from_index(v).key());
            }
        }
        if pl.chip != 255 {
            c.field("chip", pl.chip.to_string());
            c.field("chip_chance", pl.chip_chance.to_string());
        }
        b.push(c);
    }
    if let Some(bl) = &it.blast {
        let mut c = Block::new("blast", None);
        c.field("frames", bl.frames.to_string());
        if bl.delay != 0 {
            c.field("delay", bl.delay.to_string());
        }
        if bl.hits_owner {
            c.field("hits_owner", "true");
        }
        c.push(write_hitbox(&bl.hit));
        b.push(c);
    }
    if let Some(t) = &it.trap {
        let mut c = Block::new("trap", None);
        c.field("kick_x", t.kick_x.show());
        c.field("kick_y", t.kick_y.show());
        c.push(write_hitbox(&t.hit));
        b.push(c);
    }
    b
}

fn write_weapon(name: &str, w: &Weapon) -> Block {
    let mut b = Block::new("weapon", Some(name));
    if w.limbs {
        b.field("limbs", "true");
    }
    for it in &w.items {
        b.push(write_item(it));
    }
    for (i, m) in w.moves.iter().enumerate() {
        if !m.is_empty() {
            b.push(write_move(MoveId::from_index(i as u8), m));
        }
    }
    b
}

// ---- Stage --------------------------------------------------------------------------------------------------

fn read_stage(block: &Block, errors: &mut Vec<String>) -> (String, Stage, StageLook) {
    let name = block.name.clone().unwrap_or_default();
    let context = format!("stage `{name}`");
    let mut f = Fields::new(block, &context, errors);
    // How it looks (presentation only; checked by validation).
    let mut look = StageLook::default();
    if let Some((v, _)) = f.raw("backdrop") {
        look.backdrop = v.to_string();
    }
    if let Some((v, _)) = f.raw("sky_top") {
        look.sky_top = v.to_string();
    }
    if let Some((v, _)) = f.raw("sky_bottom") {
        look.sky_bottom = v.to_string();
    }
    let blast_left = f.need("blast_left", None, errors);
    let blast_right = f.need("blast_right", None, errors);
    let blast_bottom = f.need("blast_bottom", None, errors);
    let blast_top = f.need("blast_top", None, errors);
    f.finish(errors);
    only_known(block, &["platform", "ledge", "spawn"], &context, errors);
    let platforms = children(block, "platform")
        .into_iter()
        .map(|b| {
            let mut f = Fields::new(b, "a platform", errors);
            let p = Platform {
                left: f.need("left", None, errors),
                right: f.need("right", None, errors),
                y: f.need("y", None, errors),
                bottom: f.need("bottom", None, errors),
                pass_through: f.or("pass_through", false, errors),
                // (Schema 8, sim 124: a slope; absent is flat.)
                rise: f.or("rise", Fx::ZERO, errors),
            };
            f.finish(errors);
            p
        })
        .collect();
    let ledges = children(block, "ledge")
        .into_iter()
        .map(|b| {
            let mut f = Fields::new(b, "a ledge", errors);
            let side: i16 = f.need("side", None, errors);
            let l = Ledge {
                x: f.need("x", None, errors),
                y: f.need("y", None, errors),
                // The range check (-1 or 1) is validation's job.
                side: side.clamp(-128, 127) as i8,
            };
            f.finish(errors);
            l
        })
        .collect();
    let mut spawns = [Vec2::ZERO; MAX_FIGHTERS];
    let spawn_blocks = children(block, "spawn");
    if spawn_blocks.len() != MAX_FIGHTERS {
        err(
            errors,
            block.line,
            format!(
                "{context} needs exactly {MAX_FIGHTERS} `spawn` points, has {}",
                spawn_blocks.len()
            ),
        );
    }
    for (slot, b) in spawns.iter_mut().zip(spawn_blocks) {
        let mut f = Fields::new(b, "a spawn point", errors);
        *slot = Vec2::new(f.need("x", None, errors), f.need("y", None, errors));
        f.finish(errors);
    }
    (
        name,
        Stage {
            platforms,
            ledges,
            spawns,
            blast_left,
            blast_right,
            blast_bottom,
            blast_top,
        },
        look,
    )
}

fn write_stage(name: &str, s: &Stage, look: &StageLook) -> Block {
    let mut b = Block::new("stage", Some(name));
    b.field("backdrop", look.backdrop.as_str());
    if !look.sky_top.is_empty() {
        b.field("sky_top", look.sky_top.as_str());
    }
    if !look.sky_bottom.is_empty() {
        b.field("sky_bottom", look.sky_bottom.as_str());
    }
    b.field("blast_left", s.blast_left.show());
    b.field("blast_right", s.blast_right.show());
    b.field("blast_bottom", s.blast_bottom.show());
    b.field("blast_top", s.blast_top.show());
    for p in &s.platforms {
        let mut c = Block::new("platform", None);
        c.field("left", p.left.show());
        c.field("right", p.right.show());
        c.field("y", p.y.show());
        c.field("bottom", p.bottom.show());
        c.field("pass_through", p.pass_through.to_string());
        if p.rise != Fx::ZERO {
            c.field("rise", p.rise.show());
        }
        b.push(c);
    }
    for l in &s.ledges {
        let mut c = Block::new("ledge", None);
        c.field("x", l.x.show());
        c.field("y", l.y.show());
        c.field("side", l.side.to_string());
        b.push(c);
    }
    for sp in &s.spawns {
        let mut c = Block::new("spawn", None);
        c.field("x", sp.x.show());
        c.field("y", sp.y.show());
        b.push(c);
    }
    b
}

// ---- Whole content --------------------------------------------------------------------------------------

/// Top-level section kinds a content file may contain (`bundle` is read by the bundle layer).
pub const SECTIONS: [&str; 5] = ["bundle", "ruleset", "weapon", "fighter", "stage"];

/// Reads everything in `root` except the `bundle` manifest. Every problem found is appended to `errors`;
/// the result is only meaningful when none were.
pub fn read_content(root: &Block, errors: &mut Vec<String>) -> Content {
    only_known(root, &SECTIONS, "the file", errors);
    for item in &root.items {
        if let Item::Field { name, line, .. } = item {
            err(errors, *line, format!("`{name}` is not inside any section"));
        }
    }

    let rules = match at_most_one(children(root, "ruleset"), "ruleset", "the file", errors) {
        Some(b) => {
            let mut f = Fields::new(b, "the ruleset", errors);
            let r = read_rules(&mut f, &Ruleset::standard(), errors);
            f.finish(errors);
            r
        }
        None => Ruleset::standard(),
    };

    let mut weapons: Vec<(String, Weapon)> = Vec::new();
    for b in children(root, "weapon") {
        match b.name.as_deref() {
            None | Some("") => err(errors, b.line, "a weapon needs a name"),
            Some(name) if weapons.iter().any(|(n, _)| n == name) => {
                err(errors, b.line, format!("weapon `{name}` is defined twice"));
            }
            Some(name) => {
                let w = read_weapon(b, &weapons, errors);
                weapons.push((name.to_string(), w));
            }
        }
    }

    let mut fighters: Vec<(String, FighterParams)> = Vec::new();
    for b in children(root, "fighter") {
        let Some(name) = b.name.as_deref().filter(|n| !n.is_empty()) else {
            err(errors, b.line, "a fighter needs a name");
            continue;
        };
        if fighters.iter().any(|(n, _)| n == name) {
            err(errors, b.line, format!("fighter `{name}` is defined twice"));
            continue;
        }
        let context = format!("fighter `{name}`");
        only_known(
            b,
            &[
                "gauge",
                "air_jump_armor",
                "passive_shield",
                "item_throws",
                "wing",
            ],
            &context,
            errors,
        );
        let mut f = Fields::new(b, &context, errors);
        let mut base: Option<FighterParams> = None;
        if let Some((from, line)) = f.raw("inherit") {
            match fighters.iter().find(|(n, _)| n == from) {
                Some((_, p)) => base = Some(*p),
                None => err(
                    errors,
                    line,
                    format!("{context} inherits from `{from}`, which is not defined above it"),
                ),
            }
        }
        let weapon = match f.raw("weapon") {
            Some((wname, line)) => match weapons.iter().position(|(n, _)| n == wname) {
                Some(i) => i as u8,
                None => {
                    err(
                        errors,
                        line,
                        format!("{context} uses weapon `{wname}`, which does not exist"),
                    );
                    0
                }
            },
            None => match &base {
                Some(p) => p.weapon,
                None => {
                    err(errors, b.line, format!("{context} is missing `weapon`"));
                    0
                }
            },
        };
        let mut params = read_params(&mut f, base.as_ref(), weapon, errors);
        // The air jump's turnaround (sim 89): optional, the inherited fighter's or false.
        params.air_jump_turns = f.or(
            "air_jump_turns",
            base.is_some_and(|p| p.air_jump_turns),
            errors,
        );
        // The air jump's drift multiplier (sim 95): optional, the inherited fighter's or 100.
        params.air_jump_drift = f.or(
            "air_jump_drift",
            base.map_or(100, |p| p.air_jump_drift),
            errors,
        );
        f.finish(errors);
        // The gauge (schema 6): a `gauge { ... }` block, or the inherited fighter's, or none.
        if let Some(g) = at_most_one(children(b, "gauge"), "gauge", &context, errors) {
            params.gauge = read_gauge(g, &format!("the gauge of {context}"), errors);
        }
        // The air jump's armor (sim 89): an `air_jump_armor { ... }` block, or the inherited fighter's, or none.
        if let Some(a) = at_most_one(
            children(b, "air_jump_armor"),
            "air_jump_armor",
            &context,
            errors,
        ) {
            params.air_jump_armor = Some(read_armor(
                a,
                &format!("the air-jump armor of {context}"),
                errors,
            ));
        }
        // The passive shield (sim 91, E14): a `passive_shield { ... }` block, or the inherited fighter's, or none.
        if let Some(s) = at_most_one(
            children(b, "passive_shield"),
            "passive_shield",
            &context,
            errors,
        ) {
            let ctx = format!("the passive shield of {context}");
            let mut f = Fields::new(s, &ctx, errors);
            params.passive_shield = Some(sim_core::content::PassiveShield {
                x: f.need("x", None, errors),
                y: f.need("y", None, errors),
                crouch_y: f.need("crouch_y", None, errors),
                half_height: f.or("half_height", Fx::ZERO, errors),
                radius: f.need("radius", None, errors),
            });
            f.finish(errors);
        }
        // The item throws' release frames (sim 111, E4): an `item_throws { ... }` block, or the inherited fighter's, or the standard.
        if let Some(t) = at_most_one(children(b, "item_throws"), "item_throws", &context, errors) {
            let ctx = format!("the item throws of {context}");
            let mut f = Fields::new(t, &ctx, errors);
            params.item_throws = sim_core::content::ItemThrows {
                forward: f.need("forward", None, errors),
                back: f.need("back", None, errors),
                up: f.need("up", None, errors),
                down: f.need("down", None, errors),
                dash: f.need("dash", None, errors),
                drop: f.need("drop", None, errors),
            };
            f.finish(errors);
        }
        // The wing (sim 125, E57): a `wing { ... }` block, or the inherited fighter's, or none.
        if let Some(wb) = at_most_one(children(b, "wing"), "wing", &context, errors) {
            params.wing = Some(read_wing(wb, &format!("the wing of {context}"), errors));
        }
        fighters.push((name.to_string(), params));
    }

    let stage = match at_most_one(children(root, "stage"), "stage", "the file", errors) {
        Some(b) => read_stage(b, errors),
        None => {
            err(errors, 0, "the file has no `stage`");
            (String::new(), Stage::placeholder(), StageLook::default())
        }
    };

    Content {
        names: Names {
            fighters: fighters.iter().map(|(n, _)| n.clone()).collect(),
            weapons: weapons.iter().map(|(n, _)| n.clone()).collect(),
            stage: stage.0,
        },
        fighters: fighters.into_iter().map(|(_, p)| p).collect(),
        weapons: weapons.into_iter().map(|(_, w)| w).collect(),
        stage: stage.1,
        rules,
        look: stage.2,
    }
}

/// An `armor { ... }` block (a move's, or a body's `air_jump_armor`): `start end` required; `kind` super, damage (the default) or
/// knockback; `threshold`, `subtract` and `min_kb` optional.
fn read_armor(b: &Block, ctx: &str, errors: &mut Vec<String>) -> Armor {
    let mut f = Fields::new(b, ctx, errors);
    let kind = match f.raw("kind") {
        None | Some(("damage", _)) => ARMOR_DAMAGE,
        Some(("super", _)) => ARMOR_SUPER,
        Some(("knockback", _)) => ARMOR_KNOCKBACK,
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("armor kind must be super, damage or knockback, not `{other}`"),
            );
            ARMOR_DAMAGE
        }
    };
    let a = Armor {
        start: f.need("start", None, errors),
        end: f.need("end", None, errors),
        kind,
        threshold: f.or("threshold", Fx::ZERO, errors),
        subtract: f.or("subtract", false, errors),
        min_kb: f.or("min_kb", Fx::ZERO, errors),
    };
    f.finish(errors);
    a
}

/// A move's `carry { ... }` block (E7, sim 98): `walk clock_num clock_den jumps hold_x hold_y` and the four throw slots `forward back
/// up down` are required; `ledge` defaults to false; an `armor { ... }` child is optional.
fn read_carry(b: &Block, ctx: &str, errors: &mut Vec<String>) -> Carry {
    only_known(b, &["armor"], ctx, errors);
    let mut f = Fields::new(b, ctx, errors);
    let mut throws = [0u8; 4];
    for (slot, name) in throws.iter_mut().zip(["forward", "back", "up", "down"]) {
        match f.raw(name) {
            Some((key, line)) => match MoveId::from_key(key) {
                Some(id) => *slot = id as u8,
                None => err(
                    errors,
                    line,
                    format!("{ctx}: `{name}` must be a move name, not `{key}`"),
                ),
            },
            None => err(errors, b.line, format!("{ctx}: needs `{name}`")),
        }
    }
    let c = Carry {
        walk: f.need("walk", None, errors),
        clock_num: f.need("clock_num", None, errors),
        clock_den: f.need("clock_den", None, errors),
        jumps: f.need("jumps", None, errors),
        hold_at: (
            f.need("hold_x", None, errors),
            f.need("hold_y", None, errors),
        ),
        armor: None,
        throws,
        ledge: f.or("ledge", false, errors),
    };
    f.finish(errors);
    let armor = at_most_one(children(b, "armor"), "armor", ctx, errors)
        .map(|a| read_armor(a, &format!("the armor of {ctx}"), errors));
    Carry { armor, ..c }
}

/// A move's `encase { ... }` block (E8, sim 107): every field is required (see `moves::Encase`).
fn read_encase(b: &Block, ctx: &str, errors: &mut Vec<String>) -> Encase {
    only_known(b, &[], ctx, errors);
    let mut f = Fields::new(b, ctx, errors);
    let e = Encase {
        base: f.need("base", None, errors),
        per_percent: f.need("per_percent", None, errors),
        percent_cap: f.need("percent_cap", None, errors),
        mash_stick: f.need("mash_stick", None, errors),
        mash_button: f.need("mash_button", None, errors),
        hit_cut: f.need("hit_cut", None, errors),
        damage_taken: f.need("damage_taken", None, errors),
        launch: (
            f.need("launch_x", None, errors),
            f.need("launch_y", None, errors),
        ),
        grow_from: f.need("grow_from", None, errors),
        grow_frames: f.need("grow_frames", None, errors),
        size: f.need("size", None, errors),
        shift: (
            f.need("shift_x", None, errors),
            f.need("shift_y", None, errors),
        ),
        air_shift: (
            f.need("air_shift_x", None, errors),
            f.need("air_shift_y", None, errors),
        ),
        exit_vy: f.need("exit_vy", None, errors),
        exit_intangible: f.need("exit_intangible", None, errors),
    };
    f.finish(errors);
    e
}

/// A move's `balloons { ... }` block (E41, sim 113): every field is required (see `moves::Balloons`).
fn read_balloons(b: &Block, ctx: &str, errors: &mut Vec<String>) -> Balloons {
    only_known(b, &[], ctx, errors);
    let mut f = Fields::new(b, ctx, errors);
    let bl = Balloons {
        count: f.need("count", None, errors),
        hp: f.need("hp", None, errors),
        x: f.need("x", None, errors),
        y: f.need("y", None, errors),
        radius: f.need("radius", None, errors),
        fuel: f.need("fuel", None, errors),
        refill: f.need("refill", None, errors),
        one_below: f.need("one_below", None, errors),
        pop_vy: f.need("pop_vy", None, errors),
    };
    f.finish(errors);
    bl
}

fn write_balloons(bl: &Balloons) -> Block {
    let mut b = Block::new("balloons", None);
    b.field("count", bl.count.to_string());
    b.field("hp", bl.hp.show());
    b.field("x", bl.x.show());
    b.field("y", bl.y.show());
    b.field("radius", bl.radius.show());
    b.field("fuel", bl.fuel.to_string());
    b.field("refill", bl.refill.to_string());
    b.field("one_below", bl.one_below.to_string());
    b.field("pop_vy", bl.pop_vy.show());
    b
}

fn write_encase(e: &Encase) -> Block {
    let mut b = Block::new("encase", None);
    b.field("base", e.base.show());
    b.field("per_percent", e.per_percent.show());
    b.field("percent_cap", e.percent_cap.show());
    b.field("mash_stick", e.mash_stick.show());
    b.field("mash_button", e.mash_button.show());
    b.field("hit_cut", e.hit_cut.show());
    b.field("damage_taken", e.damage_taken.to_string());
    b.field("launch_x", e.launch.0.show());
    b.field("launch_y", e.launch.1.show());
    b.field("grow_from", e.grow_from.to_string());
    b.field("grow_frames", e.grow_frames.to_string());
    b.field("size", e.size.to_string());
    b.field("shift_x", e.shift.0.show());
    b.field("shift_y", e.shift.1.show());
    b.field("air_shift_x", e.air_shift.0.show());
    b.field("air_shift_y", e.air_shift.1.show());
    b.field("exit_vy", e.exit_vy.show());
    b.field("exit_intangible", e.exit_intangible.to_string());
    b
}

fn write_carry(c: &Carry) -> Block {
    let mut b = Block::new("carry", None);
    b.field("walk", c.walk.to_string());
    b.field("clock_num", c.clock_num.to_string());
    b.field("clock_den", c.clock_den.to_string());
    b.field("jumps", c.jumps.to_string());
    b.field("hold_x", c.hold_at.0.show());
    b.field("hold_y", c.hold_at.1.show());
    for (name, slot) in ["forward", "back", "up", "down"].iter().zip(c.throws) {
        b.field(name, MoveId::from_index(slot).key());
    }
    if c.ledge {
        b.field("ledge", "true");
    }
    if let Some(a) = &c.armor {
        b.push(write_armor("armor", a));
    }
    b
}

fn write_armor(name: &str, a: &Armor) -> Block {
    let mut c = Block::new(name, None);
    c.field("start", a.start.to_string());
    c.field("end", a.end.to_string());
    c.field(
        "kind",
        match a.kind {
            ARMOR_SUPER => "super",
            ARMOR_KNOCKBACK => "knockback",
            _ => "damage",
        },
    );
    if a.threshold != Fx::ZERO {
        c.field("threshold", a.threshold.show());
    }
    if a.subtract {
        c.field("subtract", "true");
    }
    if a.min_kb != Fx::ZERO {
        c.field("min_kb", a.min_kb.show());
    }
    c
}

/// A fighter's `gauge { ... }` block: `max per_dealt per_taken mode_frames` are required, the multipliers (percents) default to 100.
fn read_gauge(block: &Block, context: &str, errors: &mut Vec<String>) -> Gauge {
    let mut f = Fields::new(block, context, errors);
    let none = Gauge::NONE;
    let g = Gauge {
        max: f.need("max", None, errors),
        per_dealt: f.need("per_dealt", None, errors),
        per_taken: f.need("per_taken", None, errors),
        mode_frames: f.need("mode_frames", None, errors),
        walk: f.or("walk", none.walk, errors),
        dash: f.or("dash", none.dash, errors),
        run: f.or("run", none.run, errors),
        traction: f.or("traction", none.traction, errors),
        air_speed: f.or("air_speed", none.air_speed, errors),
        air_accel: f.or("air_accel", none.air_accel, errors),
        air_friction: f.or("air_friction", none.air_friction, errors),
        gravity: f.or("gravity", none.gravity, errors),
        fall: f.or("fall", none.fall, errors),
        // Sim 121 (E53): the partner gauge's rules, all defaulting to the champion's gauge as it was.
        per_frame: f.or("per_frame", Fx::ZERO, errors),
        behind: [
            f.or("behind_1", 1000, errors),
            f.or("behind_2", 1000, errors),
        ],
        crowd: [
            [
                f.or("crowd_3_0", 1000, errors),
                f.or("crowd_3_1", 1000, errors),
                f.or("crowd_3_2", 1000, errors),
            ],
            [
                f.or("crowd_4_0", 1000, errors),
                f.or("crowd_4_1", 1000, errors),
                f.or("crowd_4_2", 1000, errors),
            ],
        ],
        mode_taken: [
            f.or("mode_taken_2", Fx::ZERO, errors),
            f.or("mode_taken_3", Fx::ZERO, errors),
            f.or("mode_taken_4", Fx::ZERO, errors),
        ],
        ko_keep: [
            f.or("ko_keep", Fx::ZERO, errors),
            f.or("ko_keep_2", Fx::ZERO, errors),
            f.or("ko_keep_3", Fx::ZERO, errors),
        ],
        keeps: f.or("keeps", false, errors),
        halts: f.or("halts", false, errors),
    };
    f.finish(errors);
    g
}

/// The partner-gauge fields of a `gauge { ... }` block (sim 121, E53) by name, in file order, with their defaults.
fn gauge_rule_fields(g: &Gauge) -> [(&'static str, String, &'static str); 17] {
    [
        ("per_frame", g.per_frame.show(), "0"),
        ("behind_1", g.behind[0].to_string(), "1000"),
        ("behind_2", g.behind[1].to_string(), "1000"),
        ("crowd_3_0", g.crowd[0][0].to_string(), "1000"),
        ("crowd_3_1", g.crowd[0][1].to_string(), "1000"),
        ("crowd_3_2", g.crowd[0][2].to_string(), "1000"),
        ("crowd_4_0", g.crowd[1][0].to_string(), "1000"),
        ("crowd_4_1", g.crowd[1][1].to_string(), "1000"),
        ("crowd_4_2", g.crowd[1][2].to_string(), "1000"),
        ("mode_taken_2", g.mode_taken[0].show(), "0"),
        ("mode_taken_3", g.mode_taken[1].show(), "0"),
        ("mode_taken_4", g.mode_taken[2].show(), "0"),
        ("ko_keep", g.ko_keep[0].show(), "0"),
        ("ko_keep_2", g.ko_keep[1].show(), "0"),
        ("ko_keep_3", g.ko_keep[2].show(), "0"),
        ("keeps", g.keeps.to_string(), "false"),
        ("halts", g.halts.to_string(), "false"),
    ]
}

fn write_gauge(g: &Gauge) -> Block {
    let mut b = Block::new("gauge", None);
    b.field("max", g.max.show());
    b.field("per_dealt", g.per_dealt.show());
    b.field("per_taken", g.per_taken.show());
    b.field("mode_frames", g.mode_frames.to_string());
    for (name, p) in g.percents() {
        if p != 100 {
            b.field(name, p.to_string());
        }
    }
    for (name, value, default) in gauge_rule_fields(g) {
        if value != default {
            b.field(name, value);
        }
    }
    b
}

/// A fighter's `wing { ... }` block (E57, sim 125): `points_m2 points_m1 points_0 points_p1 points_p2 per_dealt` are required;
/// `per_dealt_crowd` defaults to `per_dealt`, `ko_points` to 0, `jumps` to 0 and the percents (`power` and the body's numbers) to 100.
/// Each `point { stocks diff percent }` child is one point of the start table's row for that stock difference (-2 to 2), in order.
fn read_wing(block: &Block, ctx: &str, errors: &mut Vec<String>) -> Wing {
    only_known(block, &["point"], ctx, errors);
    let mut f = Fields::new(block, ctx, errors);
    let per_dealt: Fx = f.need("per_dealt", None, errors);
    let mut w = Wing {
        rows: [WingRow::EMPTY; 5],
        points: [Fx::ZERO; 5],
        per_dealt: [per_dealt, f.or("per_dealt_crowd", per_dealt, errors)],
        ko_points: f.or("ko_points", Fx::ZERO, errors),
        power: f.or("power", 100, errors),
        jumps: f.or("jumps", 0, errors),
        walk: f.or("walk", 100, errors),
        dash: f.or("dash", 100, errors),
        run: f.or("run", 100, errors),
        run_accel: f.or("run_accel", 100, errors),
        traction: f.or("traction", 100, errors),
        air_speed: f.or("air_speed", 100, errors),
        air_accel: f.or("air_accel", 100, errors),
        air_accel_stick: f.or("air_accel_stick", 100, errors),
        air_friction: f.or("air_friction", 100, errors),
        gravity: f.or("gravity", 100, errors),
        fall: f.or("fall", 100, errors),
    };
    for (i, name) in Wing::ROWS.iter().enumerate() {
        w.points[i] = f.need(&format!("points_{name}"), None, errors);
    }
    f.finish(errors);
    for p in children(block, "point") {
        let pctx = format!("a point of {ctx}");
        let mut pf = Fields::new(p, &pctx, errors);
        let stocks: i16 = pf.need("stocks", None, errors);
        let diff: i16 = pf.need("diff", None, errors);
        let percent: Fx = pf.need("percent", None, errors);
        pf.finish(errors);
        if !(-2..=2).contains(&stocks) {
            err(errors, p.line, format!("{pctx}: stocks must be -2 to 2"));
            continue;
        }
        let row = &mut w.rows[(stocks + 2) as usize];
        let n = usize::from(row.count);
        if n >= WING_POINTS {
            err(
                errors,
                p.line,
                format!("{pctx}: a row holds at most {WING_POINTS} points"),
            );
            continue;
        }
        row.diff[n] = diff;
        row.percent[n] = percent;
        row.count += 1;
    }
    w
}

fn write_wing(w: &Wing) -> Block {
    let mut b = Block::new("wing", None);
    for (i, name) in Wing::ROWS.iter().enumerate() {
        b.field(&format!("points_{name}"), w.points[i].show());
    }
    b.field("per_dealt", w.per_dealt[0].show());
    if w.per_dealt[1] != w.per_dealt[0] {
        b.field("per_dealt_crowd", w.per_dealt[1].show());
    }
    if w.ko_points != Fx::ZERO {
        b.field("ko_points", w.ko_points.show());
    }
    if w.jumps != 0 {
        b.field("jumps", w.jumps.to_string());
    }
    for (name, p) in w.percents() {
        if p != 100 {
            b.field(name, p.to_string());
        }
    }
    for (i, row) in w.rows.iter().enumerate() {
        for n in 0..usize::from(row.count).min(WING_POINTS) {
            let mut c = Block::new("point", None);
            c.field("stocks", (i as i32 - 2).to_string());
            c.field("diff", row.diff[n].to_string());
            c.field("percent", row.percent[n].show());
            b.push(c);
        }
    }
    b
}

/// A fighter section with every parameter written out (and its gauge, when the body has one).
pub fn fighter_block(name: &str, p: &FighterParams, weapon_name: &str) -> Block {
    let mut b = Block::new("fighter", Some(name));
    b.field("weapon", weapon_name);
    write_params(p, &mut b);
    if p.air_jump_turns {
        b.field("air_jump_turns", "true");
    }
    if p.air_jump_drift != 100 {
        b.field("air_jump_drift", p.air_jump_drift.to_string());
    }
    if p.gauge != Gauge::NONE {
        b.push(write_gauge(&p.gauge));
    }
    if let Some(a) = &p.air_jump_armor {
        b.push(write_armor("air_jump_armor", a));
    }
    if let Some(s) = &p.passive_shield {
        let mut c = Block::new("passive_shield", None);
        c.field("x", s.x.show());
        c.field("y", s.y.show());
        c.field("crouch_y", s.crouch_y.show());
        if s.half_height != Fx::ZERO {
            c.field("half_height", s.half_height.show());
        }
        c.field("radius", s.radius.show());
        b.push(c);
    }
    if let Some(w) = &p.wing {
        b.push(write_wing(w));
    }
    if p.item_throws != sim_core::content::ItemThrows::STANDARD {
        let mut c = Block::new("item_throws", None);
        for (name, v) in p.item_throws.frames() {
            c.field(name, v.to_string());
        }
        b.push(c);
    }
    b
}

/// The sections of a content file for `content`, in a fixed order.
pub fn write_content(content: &Content) -> Vec<Block> {
    let mut out = Vec::new();

    let mut rules = Block::new("ruleset", None);
    write_rules(&content.rules, &mut rules);
    out.push(rules);

    for (i, w) in content.weapons.iter().enumerate() {
        let name = content
            .names
            .weapons
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("weapon{i}"));
        out.push(write_weapon(&name, w));
    }
    for (i, p) in content.fighters.iter().enumerate() {
        let name = content
            .names
            .fighters
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("fighter{i}"));
        let weapon = content
            .names
            .weapons
            .get(usize::from(p.weapon))
            .cloned()
            .unwrap_or_else(|| format!("weapon{}", p.weapon));
        out.push(fighter_block(&name, p, &weapon));
    }
    let stage_name = if content.names.stage.is_empty() {
        "stage"
    } else {
        &content.names.stage
    };
    out.push(write_stage(stage_name, &content.stage, &content.look));
    out
}
