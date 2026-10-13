//! Content to text and back: fighters, weapons (movesets), the stage and the ruleset.
//!
//! The reader reports every problem it finds, with line numbers, instead of stopping at the first. A file
//! that parses is not necessarily *valid*; see [`crate::validate`] for the range and sanity checks.

use crate::tree::{Block, Item};
use sim_core::content::{
    Content, FighterParams, Gauge, Ledge, Names, Platform, Ruleset, Stage, StageLook,
};
use sim_core::moves::{
    AirBlock, Armor, Counter, CounterBubble, Hitbox, Motion, Move, MoveId, PartIntangible,
    ProjectileArc, ProjectileSpawn, Reflector, Weapon, AIM_ANY, AIM_DOWN, AIM_STRAIGHT, AIM_UP,
    ANGLE_BACKWARD, ANGLE_BY_BOX, ANGLE_BY_POSITION, ANGLE_BY_TRAVEL, ANGLE_FORWARD, ARMOR_DAMAGE,
    ARMOR_KNOCKBACK, ARMOR_SUPER, EFFECT_ELECTRIC, EFFECT_NORMAL, HIT_GRAB, HIT_NORMAL, HIT_PUMMEL,
    HIT_SEARCH, HIT_THROW, MAX_PHASES, PART_HIGH, PART_LIMB, PART_LOW, PART_MIDDLE, TARGETS_AERIAL,
    TARGETS_ALL, TARGETS_GROUNDED,
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
    wall_tech_invuln: u8,
    bounce_keep: Fx,
    ground_bounce_speed: Fx,
    respawn_height: Fx,
    respawn_platform_frames: u8,
    trip_lag: u8,
    trip_max: u8,
    trip_invuln: u8,
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
        Some((other, line)) => {
            err(
                errors,
                line,
                format!("hitbox effect must be normal or electric, not `{other}`"),
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
    match hb.angle_type {
        ANGLE_FORWARD => b.field("angle_type", "forward"),
        ANGLE_BACKWARD => b.field("angle_type", "backward"),
        ANGLE_BY_POSITION => b.field("angle_type", "position"),
        ANGLE_BY_TRAVEL => b.field("angle_type", "travel"),
        _ => {}
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
        ("catch_move", &mut m.catch_move),
        ("off_edge", &mut m.off_edge),
        ("limit_version", &mut m.limit_version),
        ("on_land", &mut m.on_land),
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
        ],
        context,
        errors,
    );
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
            };
            f.finish(errors);
            a
        })
        .collect();
    if let Some(b) = at_most_one(children(block, "armor"), "armor", context, errors) {
        let ctx = format!("the armor of {context}");
        let mut f = Fields::new(b, &ctx, errors);
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
        };
        f.finish(errors);
        m.armor = Some(a);
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
            let w = PartIntangible {
                start: f.need("start", None, errors),
                end: f.need("end", None, errors),
                parts,
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
        only_known(b, &["hitbox", "phase"], &ctx, errors);
        let mut f = Fields::new(b, &ctx, errors);
        let hitbox = match at_most_one(children(b, "hitbox"), "hitbox", &ctx, errors) {
            Some(h) => read_hitbox(h, &ctx, errors),
            None => {
                err(errors, b.line, format!("{ctx} needs a hitbox"));
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
                }
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
        };
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
        };
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
    ] {
        if let Some(v) = value {
            b.field(name, v.show());
        }
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
        ("catch_move", m.catch_move),
        ("off_edge", m.off_edge),
        ("limit_version", m.limit_version),
        ("on_land", m.on_land),
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
        b.push(c);
    }
    if let Some(a) = &m.armor {
        let mut c = Block::new("armor", None);
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
        b.push(c);
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
        ] {
            if v != d {
                c.field(name, v.show());
            }
        }
        if a.bounces != none.bounces {
            c.field("bounces", a.bounces.to_string());
        }
        if a.bounce_keep != none.bounce_keep {
            c.field("bounce_keep", a.bounce_keep.to_string());
        }
        if p.pierce {
            c.field("pierce", "true");
        }
        if p.rehit != 0 {
            c.field("rehit", p.rehit.to_string());
        }
        c.push(write_hitbox(&p.hitbox));
        for (from, hb) in p.phases.iter().flatten() {
            let mut pb = Block::new("phase", None);
            pb.field("from", from.to_string());
            pb.push(write_hitbox(hb));
            c.push(pb);
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
    if let Some((base, line)) = f.raw("inherit") {
        match earlier.iter().find(|(n, _)| n == base) {
            Some((_, w)) => {
                moves = w.moves.clone();
                limbs = w.limbs;
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
    only_known(block, &["move"], &context, errors);
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
    Weapon { moves, limbs }
}

fn write_weapon(name: &str, w: &Weapon) -> Block {
    let mut b = Block::new("weapon", Some(name));
    if w.limbs {
        b.field("limbs", "true");
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
        only_known(b, &["gauge"], &context, errors);
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
        f.finish(errors);
        // The gauge (schema 6): a `gauge { ... }` block, or the inherited fighter's, or none.
        if let Some(g) = at_most_one(children(b, "gauge"), "gauge", &context, errors) {
            params.gauge = read_gauge(g, &format!("the gauge of {context}"), errors);
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
    };
    f.finish(errors);
    g
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
    b
}

/// A fighter section with every parameter written out (and its gauge, when the body has one).
pub fn fighter_block(name: &str, p: &FighterParams, weapon_name: &str) -> Block {
    let mut b = Block::new("fighter", Some(name));
    b.field("weapon", weapon_name);
    write_params(p, &mut b);
    if p.gauge != Gauge::NONE {
        b.push(write_gauge(&p.gauge));
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
