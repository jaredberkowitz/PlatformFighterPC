//! Content: the text format ([`tree`], [`format`]), bundles with a manifest and versioning ([`bundle`]), and
//! validation. Loading is pure text in, [`Content`] out; reading files is the caller's job, so this crate stays
//! free of I/O like the rest of the simulation.
//!
//! Validation is the guardrail layer: refuse content that would break the simulation (or, later, ranked balance).

pub mod bundle;
pub mod doc;
pub mod format;
pub mod policy;
pub mod recipe;
pub mod stages;
pub mod tree;

pub use bundle::{load, to_text, Bundle, Manifest, SCHEMA_VERSION};

use sim_core::content::{MAX_LEDGES, MAX_PLATFORMS};
use sim_core::moves::{Move, MoveId, Weapon};
use sim_core::state::HISTORY_LEN;
use sim_core::{Content, FighterParams, Fx, Stage};

/// Most fighters and weapons one bundle may define (fighter ids are bytes, and bundles should stay small).
const MAX_FIGHTER_TYPES: usize = 64;
const MAX_WEAPONS: usize = 64;

/// Largest value any distance, speed or acceleration may take, in world units.
const MAX_VALUE: Fx = Fx::from_int(64);

/// Fields that must be strictly positive (zero would freeze or break the movement).
const MUST_BE_POSITIVE: [&str; 13] = [
    "hitbox_scale",
    "walk_speed",
    "dash_speed",
    "ecb_half_width",
    "ecb_height",
    "run_speed",
    "gravity",
    "max_fall_speed",
    "full_hop_velocity",
    "short_hop_velocity",
    "air_dodge_speed",
    "waveland_speed",
    "ground_assist_dist",
];

/// Per-frame multipliers: anything above 1.0 would make the velocity grow.
const MULTIPLIERS: [&str; 3] = ["air_dodge_decay", "air_dodge_sling", "waveland_friction"];

/// Returns every problem found, so creators see them all at once instead of one per attempt.
pub fn validate(content: &Content) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if content.fighters.is_empty() {
        errors.push("content defines no fighters".to_string());
    }
    for (i, f) in content.fighters.iter().enumerate() {
        validate_fighter(i, f, content.weapons.len(), &mut errors);
    }
    if content.weapons.is_empty() {
        errors.push("content defines no weapons".to_string());
    }
    if content.fighters.len() > MAX_FIGHTER_TYPES {
        errors.push(format!(
            "content defines more than {MAX_FIGHTER_TYPES} fighters"
        ));
    }
    if content.weapons.len() > MAX_WEAPONS {
        errors.push(format!("content defines more than {MAX_WEAPONS} weapons"));
    }
    validate_names(content, &mut errors);
    for (i, w) in content.weapons.iter().enumerate() {
        validate_weapon(i, w, &mut errors);
    }
    validate_stage(&content.stage, &mut errors);
    validate_look(&content.look, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Names are how content files refer to things, so they must be present, tidy and unique.
fn validate_names(content: &Content, errors: &mut Vec<String>) {
    let policy = policy::Policy::default();
    let groups: [(&str, &[String], usize); 2] = [
        ("fighter", &content.names.fighters, content.fighters.len()),
        ("weapon", &content.names.weapons, content.weapons.len()),
    ];
    for (what, names, count) in groups {
        if names.len() != count {
            errors.push(format!(
                "{what} names do not match the {what}s ({} names for {count})",
                names.len()
            ));
        }
        for (i, name) in names.iter().enumerate() {
            let tidy = !name.is_empty()
                && name.len() <= 32
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
            if !tidy {
                errors.push(format!(
                    "{what} {i}: name `{name}` must be 1 to 32 letters, digits, `_` or `-`"
                ));
            }
            if policy.is_blocked(name) {
                errors.push(format!(
                    "{what} {i}: name `{name}` uses a blocked term (someone else's trademark, or a word the project blocks)"
                ));
            }
            if names[..i].contains(name) {
                errors.push(format!("{what} name `{name}` is used twice"));
            }
        }
    }
}

fn validate_fighter(i: usize, f: &FighterParams, weapons: usize, errors: &mut Vec<String>) {
    if usize::from(f.weapon) >= weapons {
        errors.push(format!("fighter {i}: weapon {} does not exist", f.weapon));
    }
    if f.weight < Fx::from_int(20) || f.weight > Fx::from_int(400) {
        errors.push(format!("fighter {i}: weight must be between 20 and 400"));
    }
    for (name, value) in f.fx_fields() {
        if name == "weight" {
            continue;
        }
        if value < Fx::ZERO {
            errors.push(format!("fighter {i}: {name} must not be negative"));
        } else if value > MAX_VALUE {
            errors.push(format!("fighter {i}: {name} exceeds the allowed maximum"));
        }
        if value == Fx::ZERO && MUST_BE_POSITIVE.contains(&name) {
            errors.push(format!("fighter {i}: {name} must be positive"));
        }
        if value > Fx::ONE && MULTIPLIERS.contains(&name) {
            errors.push(format!("fighter {i}: {name} must be at most 1.0"));
        }
    }
    for (name, value) in f.int_fields() {
        let at_least_one = [
            "jump_squat_frames",
            "air_dodge_frames",
            "ledge_hang_max",
            "dash_frames",
            "input_buffer",
            "air_dodge_dir_down_frames",
            "air_dodge_dir_side_frames",
            "air_dodge_dir_up_frames",
            "air_dodge_landing_step",
            "roll_back_frames",
            "ledge_grab_limit",
        ];
        if value == 0 && at_least_one.contains(&name) {
            errors.push(format!("fighter {i}: {name} must be at least 1"));
        }
    }
    if usize::from(f.air_dodge_buffer) > HISTORY_LEN
        || usize::from(f.shield_drop_buffer) > HISTORY_LEN
        || usize::from(f.input_buffer) >= HISTORY_LEN
    {
        errors.push(format!(
            "fighter {i}: input buffers cannot exceed {HISTORY_LEN} frames"
        ));
    }
    if f.ledge_option_decay_2 > 100 || f.ledge_option_decay_3 > 100 {
        errors.push(format!(
            "fighter {i}: ledge_option_decay_2 and ledge_option_decay_3 are percents (at most 100)"
        ));
    }
    if f.ledge_reach_back_x > f.ledge_reach_x {
        errors.push(format!(
            "fighter {i}: ledge_reach_back_x exceeds ledge_reach_x"
        ));
    }
    if f.ledge_hang_dx < f.ecb_half_width {
        errors.push(format!(
            "fighter {i}: ledge_hang_dx is smaller than ecb_half_width, so hanging would embed in the wall"
        ));
    }
    if f.hitbox_scale < Fx::from_ratio(1, 4) || f.hitbox_scale > Fx::from_int(4) {
        errors.push(format!(
            "fighter {i}: hitbox_scale must be between 0.25 and 4"
        ));
    }
    if f.ecb_side_height >= f.ecb_height {
        errors.push(format!(
            "fighter {i}: ecb_side_height must be below ecb_height"
        ));
    }
    if hop_height(f.short_hop_velocity, f.gravity) > full_hop_height(f) {
        errors.push(format!("fighter {i}: short hop is higher than full hop"));
    }
    let g = &f.gauge;
    if g.max < Fx::ZERO || g.max > Fx::from_int(1000) {
        errors.push(format!("fighter {i}: the gauge's max must be 0 to 1000"));
    }
    if g.exists() {
        if g.per_dealt < Fx::ZERO
            || g.per_taken < Fx::ZERO
            || g.per_dealt > Fx::from_int(100)
            || g.per_taken > Fx::from_int(100)
        {
            errors.push(format!(
                "fighter {i}: the gauge's per_dealt and per_taken must be 0 to 100"
            ));
        }
        if g.mode_frames == 0 {
            errors.push(format!(
                "fighter {i}: the gauge's mode_frames must be at least 1"
            ));
        }
        for (name, p) in g.percents() {
            if p == 0 || p > 400 {
                errors.push(format!(
                    "fighter {i}: the gauge's {name} must be a percent from 1 to 400"
                ));
            }
        }
    }
    if let Some(s) = &f.passive_shield {
        if s.radius <= Fx::ZERO || s.half_height < Fx::ZERO {
            errors.push(format!(
                "fighter {i}: the passive shield needs a radius above 0 and no negative half_height"
            ));
        }
    }
    if let Some(a) = &f.air_jump_armor {
        if a.start > a.end {
            errors.push(format!(
                "fighter {i}: the air-jump armor's start must not be after its end"
            ));
        }
        if let Some(e) = armor_error(a) {
            errors.push(format!("fighter {i}: the air-jump armor: {e}"));
        }
    }
}

/// What is wrong with an armor's kind and numbers (a move's or a body's), if anything.
fn armor_error(a: &sim_core::moves::Armor) -> Option<&'static str> {
    use sim_core::moves::{ARMOR_DAMAGE, ARMOR_KNOCKBACK};
    if (a.kind == ARMOR_DAMAGE || a.kind == ARMOR_KNOCKBACK) && a.threshold <= Fx::ZERO {
        Some("damage and knockback armor need a threshold above 0")
    } else if (a.subtract || a.min_kb != Fx::ZERO) && a.kind != ARMOR_KNOCKBACK {
        Some("subtract and min_kb are for knockback armor only")
    } else if a.min_kb < Fx::ZERO || (a.min_kb > Fx::ZERO && !a.subtract) {
        Some("min_kb is the least launch of a subtractive armor (subtract true), 0 or more")
    } else {
        None
    }
}

/// Peak height of a hop that leaves the ground at `v` (gravity acts before each move).
fn hop_height(v: Fx, gravity: Fx) -> Fx {
    if gravity <= Fx::ZERO {
        return Fx::ZERO;
    }
    v * v / (gravity * Fx::from_int(2)) - v * Fx::HALF
}

/// Peak height of a full hop, including its fast opening if it has one.
pub(crate) fn full_hop_height(f: &FighterParams) -> Fx {
    hop_height(f.full_hop_velocity, f.gravity)
        + f.hop_burst_velocity.mul_int(i32::from(f.hop_burst_frames))
}

fn validate_weapon(i: usize, w: &Weapon, errors: &mut Vec<String>) {
    if w.moves.len() != MoveId::COUNT {
        errors.push(format!(
            "weapon {i}: needs exactly {} moves, has {}",
            MoveId::COUNT,
            w.moves.len()
        ));
    }
    // Moves some aerial lands into (a landing hit): their hitboxes may come out on their first frame, the landing frame.
    let landing_moves: Vec<u8> = w.moves.iter().filter_map(|mv| mv.landing_move).collect();
    for (m, mv) in w.moves.iter().enumerate() {
        let id = MoveId::from_index(m as u8);
        let name = id.name();
        for (field, slot) in [
            ("landing_move", mv.landing_move),
            ("on_search", mv.on_search),
            ("air_version", mv.air_version),
            ("catch_move", mv.catch_move),
            ("off_edge", mv.off_edge),
            ("limit_version", mv.limit_version),
            ("on_land", mv.on_land),
            ("detect_then", mv.reflector.and_then(|r| r.detect_then)),
        ] {
            if let Some(l) = slot {
                if usize::from(l) >= MoveId::COUNT
                    || w.moves.get(usize::from(l)).is_none_or(Move::is_empty)
                {
                    errors.push(format!("weapon {i} {name}: {field} needs a real move"));
                }
            }
        }
        if mv.is_empty() {
            // Only special moves may be left unimplemented.
            if !id.may_be_empty() {
                errors.push(format!("weapon {i} {name}: has no frames"));
            }
            continue;
        }
        if let Some(c) = &mv.carry {
            // A carry (E7): its lift is a throw slot, its throws real throws, its clock a fraction with a denominator.
            if !matches!(
                id,
                MoveId::FThrow | MoveId::BThrow | MoveId::UThrow | MoveId::DThrow
            ) {
                errors.push(format!(
                    "weapon {i} {name}: a carry belongs on a throw (the lift)"
                ));
            }
            if c.clock_num == 0 || c.clock_den == 0 || c.walk == 0 || c.walk > 1000 {
                errors.push(format!(
                    "weapon {i} {name}: a carry needs clock_num and clock_den above 0 and walk 1 to 1000"
                ));
            }
            for t in c.throws {
                let throw = w.moves.get(usize::from(t));
                let real = usize::from(t) < MoveId::COUNT
                    && throw.is_some_and(|m| {
                        !m.is_empty()
                            && m.carry.is_none()
                            && m.hitboxes
                                .iter()
                                .any(|h| h.kind == sim_core::moves::HIT_THROW)
                    });
                if !real {
                    errors.push(format!(
                        "weapon {i} {name}: each of a carry's throws needs a real move with a throw hit (and no carry of its own)"
                    ));
                }
            }
            if let Some(a) = &c.armor {
                if a.start > a.end {
                    errors.push(format!(
                        "weapon {i} {name}: the carry's armor start must not be after its end"
                    ));
                }
                if let Some(e) = armor_error(a) {
                    errors.push(format!("weapon {i} {name}: the carry's armor: {e}"));
                }
            }
        }
        if mv.hitboxes.is_empty()
            && mv.projectile.is_none()
            && mv.script.is_none()
            && mv.counter.is_none()
            && mv.carry.is_none()
        {
            errors.push(format!("weapon {i} {name}: has no hitboxes"));
        }
        if let Some(p) = &mv.projectile {
            if p.life == 0 {
                errors.push(format!("weapon {i} {name}: projectile has no lifetime"));
            }
            let a = &p.arc;
            if a.gravity < Fx::ZERO
                || a.max_fall < Fx::ZERO
                || a.bounce_min < Fx::ZERO
                || a.bounce_min > a.bounce_max
                || a.bounce_keep > 100
                || a.bounce_keep_y > 100
                || a.bounce_max_fall < Fx::ZERO
                || !(-90..=90).contains(&a.angle)
            {
                errors.push(format!(
                    "weapon {i} {name}: projectile flight out of range (angle -90 to 90, no negative gravity, fall or bounce \
                     speed, bounce_min at most bounce_max, bounce_keep and bounce_keep_y at most 100, no negative bounce_max_fall)"
                ));
            }
            let mut last = 0;
            for (from, _) in p.phases.iter().flatten() {
                if *from <= last || *from >= p.life {
                    errors.push(format!(
                        "weapon {i} {name}: projectile phases must start in order, after age 0 and within its life"
                    ));
                }
                last = *from;
            }
            if let Some(b) = &p.burst {
                let mut last = 0;
                let mut ordered = b.life > 0;
                for (from, hb) in b.phases.iter().flatten() {
                    ordered &= *from > last
                        && *from < b.life
                        && hb.radius >= Fx::ZERO
                        && hb.half_height >= Fx::ZERO;
                    last = *from;
                }
                if !ordered || last == 0 {
                    errors.push(format!(
                        "weapon {i} {name}: a projectile's burst needs a lifetime and at least one phase, its phases in order, \
                         after age 0 and within its life, with no negative radius or half_height"
                    ));
                }
            }
        }
        if mv.projectile_script.is_some() && mv.projectile.is_none() {
            errors.push(format!(
                "weapon {i} {name}: projectile_script needs a projectile"
            ));
        }
        for (label, script) in [
            ("script", &mv.script),
            ("projectile_script", &mv.projectile_script),
        ] {
            if let Some(program) = script {
                if let Some(problem) = sim_script::check::dry_run(program) {
                    errors.push(format!("weapon {i} {name}: {label} {problem}"));
                }
            }
        }
        if let Some(c) = &mv.counter {
            if usize::from(c.then) >= MoveId::COUNT
                || w.moves.get(usize::from(c.then)).is_none_or(Move::is_empty)
            {
                errors.push(format!(
                    "weapon {i} {name}: the counter's answer is not a move this weapon has"
                ));
            }
            if c.max_damage != Fx::ZERO && c.max_damage < c.min_damage {
                errors.push(format!(
                    "weapon {i} {name}: the counter's max_damage is below its min_damage"
                ));
            }
            if c.bubble
                .is_some_and(|b| b.radius <= Fx::ZERO || b.half_height < Fx::ZERO)
            {
                errors.push(format!(
                    "weapon {i} {name}: the counter's bubble needs a positive radius and no negative half height"
                ));
            }
            if c.start == 0 || c.start > c.end || c.end > mv.total_frames {
                errors.push(format!(
                    "weapon {i} {name}: counter window {}..{} is outside the move",
                    c.start, c.end
                ));
            }
        }
        if let Some(n) = mv.next {
            if usize::from(n) >= MoveId::COUNT || mv.next_window == 0 {
                errors.push(format!(
                    "weapon {i} {name}: chain needs a real move and a window"
                ));
            }
        }
        for p in &mv.part_intangible {
            if p.start > p.end || p.end > mv.total_frames || p.parts == 0 || p.parts > 15 {
                errors.push(format!(
                    "weapon {i} {name}: part_intangible {}..{} must lie inside the move and name at least one part",
                    p.start, p.end
                ));
            }
        }
        for a in &mv.air_blocks {
            if a.from > a.to
                || a.drift > 100
                || a.accel == 0
                || a.accel > 1000
                || a.gravity.is_some_and(|g| g < Fx::ZERO)
                || a.max_fall.is_some_and(|v| v <= Fx::ZERO)
                || a.friction.is_some_and(|v| v < Fx::ZERO)
                || a.speed.is_some_and(|v| v < Fx::ZERO)
                || a.accel_stick.is_some_and(|v| v < Fx::ZERO)
            {
                errors.push(format!(
                    "weapon {i} {name}: an air block needs from <= to, drift up to 100, accel 1 to 1000, no negative gravity, friction, speed or accel_stick and max_fall above 0"
                ));
            }
        }
        if mv.fall_speed.is_some_and(|v| v < Fx::ZERO)
            || mv.fall_accel.is_some_and(|v| v < Fx::ZERO)
        {
            errors.push(format!(
                "weapon {i} {name}: fall_speed and fall_accel cannot be negative"
            ));
        }
        for hb in &mv.hitboxes {
            if hb.posvec_frames == 0 && (hb.posvec_x != Fx::ZERO || hb.posvec_y != Fx::ZERO) {
                errors.push(format!(
                    "weapon {i} {name}: a position vector (posvec_x, posvec_y) needs posvec_frames above 0"
                ));
            }
        }
        if mv.turn_at.is_some_and(|(_, stick)| stick > 99) {
            errors.push(format!("weapon {i} {name}: turn_stick is at most 99"));
        }
        if mv
            .charge_add
            .is_some_and(|c| c.full == 0 || c.add < Fx::ZERO)
        {
            errors.push(format!(
                "weapon {i} {name}: charge_add needs full above 0 and add of at least 0"
            ));
        }
        if let Some(a) = &mv.armor {
            if a.start > a.end || a.end > mv.total_frames {
                errors.push(format!(
                    "weapon {i} {name}: armor {}..{} must lie inside the move",
                    a.start, a.end
                ));
            }
            if let Some(e) = armor_error(a) {
                errors.push(format!("weapon {i} {name}: {e}"));
            }
        }
        if mv.steer_degrees > 90 || mv.steer_stick > 99 {
            errors.push(format!(
                "weapon {i} {name}: steer_degrees is at most 90 and steer_stick at most 99"
            ));
        }
        if mv.fall_drift > 100
            || mv.air_drift > 100
            || mv.start_speed > 100
            || mv.start_speed_y > 100
            || mv.motion_drift > 100
        {
            errors.push(format!(
                "weapon {i} {name}: fall_drift, air_drift, start_speed, start_speed_y and motion_drift are percents up to 100"
            ));
        }
        if mv
            .ledge_drop
            .is_some_and(|(front, back)| front < Fx::ZERO || back < Fx::ZERO)
        {
            errors.push(format!(
                "weapon {i} {name}: ledge_drop_front and ledge_drop_back cannot be negative"
            ));
        }
        let projectile_boxes = mv.projectile.iter().flat_map(|p| {
            std::iter::once(p.hitbox).chain(p.phases.iter().flatten().map(|(_, hb)| *hb))
        });
        if mv
            .hitboxes
            .iter()
            .copied()
            .chain(projectile_boxes)
            .any(|hb| hb.half_height < Fx::ZERO)
        {
            errors.push(format!(
                "weapon {i} {name}: a hitbox's half_height cannot be negative"
            ));
        }
        if mv.air_gravity.is_some_and(|g| g < Fx::ZERO)
            || mv.air_max_fall.is_some_and(|v| v <= Fx::ZERO)
            || mv.air_friction.is_some_and(|v| v < Fx::ZERO)
        {
            errors.push(format!(
                "weapon {i} {name}: air_gravity and air_friction cannot be negative, air_max_fall must be above 0"
            ));
        }
        if let Some((_, every)) = mv.rehit {
            if every == 0 {
                errors.push(format!("weapon {i} {name}: rehit interval cannot be zero"));
            }
        }
        if let Some(at) = mv.charge_at {
            let first_hit = mv
                .hitboxes
                .iter()
                .map(|hb| hb.start)
                .min()
                .unwrap_or(u8::MAX);
            if at >= first_hit {
                errors.push(format!(
                    "weapon {i} {name}: charge frame {at} must come before the first hitbox ({first_hit})"
                ));
            }
        }
        for hb in &mv.hitboxes {
            if hb.group > 7 {
                errors.push(format!("weapon {i} {name}: hitbox group must be 0 to 7"));
            }
            if hb.trip > 100 {
                errors.push(format!(
                    "weapon {i} {name}: a hitbox's trip chance is a percent (0 to 100)"
                ));
            }
        }
        for hb in &mv.hitboxes {
            let first = if landing_moves.contains(&(m as u8)) {
                0
            } else {
                1
            };
            if hb.start < first || hb.start > hb.end || hb.end > mv.total_frames {
                errors.push(format!(
                    "weapon {i} {name}: hitbox frames {}..{} are outside the move",
                    hb.start, hb.end
                ));
            }
            if hb.radius <= Fx::ZERO || hb.damage < Fx::ZERO {
                errors.push(format!(
                    "weapon {i} {name}: hitbox needs a positive radius and non-negative damage"
                ));
            }
            if hb.base_knockback < 0 || hb.knockback_growth < 0 {
                errors.push(format!("weapon {i} {name}: knockback cannot be negative"));
            }
        }
    }
}

/// The backdrops the game knows how to draw (see `godot/scripts/stage_art.gd`).
pub const BACKDROPS: [&str; 6] = ["meadow", "grove", "sunset", "night", "ocean", "city"];

fn validate_look(look: &sim_core::content::StageLook, errors: &mut Vec<String>) {
    if !BACKDROPS.contains(&look.backdrop.as_str()) {
        errors.push(format!(
            "stage: backdrop must be one of {}, not `{}`",
            BACKDROPS.join(", "),
            look.backdrop
        ));
    }
    for (name, value) in [("sky_top", &look.sky_top), ("sky_bottom", &look.sky_bottom)] {
        let hex = value.len() == 6 && value.chars().all(|c| c.is_ascii_hexdigit());
        if !value.is_empty() && !hex {
            errors.push(format!(
                "stage: {name} must be a colour as six hex digits (like 5d8ce6), not `{value}`"
            ));
        }
    }
}

fn validate_stage(s: &Stage, errors: &mut Vec<String>) {
    if s.platforms.is_empty() {
        errors.push("stage: needs at least one platform".to_string());
    }
    if s.platforms.len() > MAX_PLATFORMS {
        errors.push(format!("stage: at most {MAX_PLATFORMS} platforms"));
    }
    if s.ledges.len() > MAX_LEDGES {
        errors.push(format!("stage: at most {MAX_LEDGES} ledges"));
    }
    for (i, p) in s.platforms.iter().enumerate() {
        if p.left >= p.right {
            errors.push(format!("stage: platform {i} has left >= right"));
        }
    }
    for (i, p) in s.platforms.iter().enumerate() {
        if !p.pass_through && p.bottom >= p.y {
            errors.push(format!("stage: solid block {i} has bottom >= top"));
        }
    }
    for (i, l) in s.ledges.iter().enumerate() {
        if l.side != -1 && l.side != 1 {
            errors.push(format!("stage: ledge {i} side must be -1 or 1"));
        }
    }
    if s.blast_left >= s.blast_right {
        errors.push("stage: blast_left must be left of blast_right".to_string());
    }
    if s.blast_bottom >= s.blast_top {
        errors.push("stage: blast_bottom must be below blast_top".to_string());
    }
    for (i, p) in s.platforms.iter().enumerate() {
        if p.y <= s.blast_bottom || p.y >= s.blast_top {
            errors.push(format!("stage: platform {i} is outside the blast zones"));
        }
    }
    for (i, p) in s.spawns.iter().enumerate() {
        if p.x <= s.blast_left
            || p.x >= s.blast_right
            || p.y <= s.blast_bottom
            || p.y >= s.blast_top
        {
            errors.push(format!("stage: spawn {i} is outside the blast zones"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_names_that_borrow_a_trademark_are_refused() {
        let mut c = Content::placeholder();
        c.names.fighters[0] = "Mario".to_string();
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("blocked term")),
            "{errors:?}"
        );
        let mut c = Content::placeholder();
        c.names.stage = "ok_stage".to_string();
        assert_eq!(validate(&c), Ok(()));
    }

    #[test]
    fn placeholder_content_is_valid() {
        assert_eq!(validate(&Content::placeholder()), Ok(()));
    }

    #[test]
    fn reports_every_problem() {
        let mut c = Content::placeholder();
        c.fighters[0].gravity = Fx::ZERO;
        c.fighters[1].run_speed = Fx::from_int(100);
        c.fighters[1].air_dodge_decay = Fx::from_int(2);
        c.stage.blast_left = c.stage.blast_right;
        let errors = validate(&c).unwrap_err();
        assert!(errors.len() >= 4, "{errors:?}");
    }

    #[test]
    fn rejects_zero_frame_counts_and_oversized_buffers() {
        let mut c = Content::placeholder();
        c.fighters[0].jump_squat_frames = 0;
        c.fighters[0].air_dodge_buffer = 200;
        let errors = validate(&c).unwrap_err();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }

    #[test]
    fn rejects_bad_ledges_and_platforms() {
        let mut c = Content::placeholder();
        c.stage.ledges[0].side = 0;
        c.stage.platforms[1].left = c.stage.platforms[1].right;
        assert_eq!(validate(&c).unwrap_err().len(), 2);
    }

    #[test]
    fn rejects_inconsistent_body_and_blocks() {
        let mut c = Content::placeholder();
        c.fighters[0].ledge_hang_dx = c.fighters[0].ecb_half_width - Fx::from_raw(1);
        c.fighters[0].ecb_side_height = c.fighters[0].ecb_height;
        c.stage.platforms[0].bottom = c.stage.platforms[0].y;
        assert_eq!(validate(&c).unwrap_err().len(), 3);
    }

    #[test]
    fn empty_content_is_rejected() {
        let mut c = Content::placeholder();
        c.fighters.clear();
        assert!(validate(&c).is_err());
    }
}

#[cfg(test)]
mod combat_tests {
    use super::*;
    use sim_script::{Kind, Program};

    #[test]
    fn rejects_endless_scripts_and_orphan_projectile_scripts() {
        let mut c = Content::placeholder();
        c.weapons[0].moves[MoveId::NSpecial as usize].script =
            Some(Program::compile(Kind::Fighter, "while true { add_vel(0, 0); }").unwrap());
        c.weapons[0].moves[MoveId::SideSpecial as usize].projectile_script =
            Some(Program::compile(Kind::Projectile, "kill();").unwrap());
        c.weapons[0].moves[MoveId::SideSpecial as usize].projectile = None;
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("never finishes")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("needs a projectile")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_a_counter_that_answers_with_nothing_or_has_a_bad_window() {
        let mut c = Content::placeholder();
        let down = &mut c.weapons[0].moves[MoveId::DownSpecial as usize];
        let counter = down.counter.as_mut().unwrap();
        counter.then = MoveId::Ext9 as u8; // an empty slot
        counter.start = 30; // after the end of the window
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("counter's answer")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("counter window")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_a_counter_bubble_without_a_radius_and_reads_one_back() {
        let mut c = Content::placeholder();
        let counter = c.weapons[0].moves[MoveId::DownSpecial as usize]
            .counter
            .as_mut()
            .unwrap();
        let bubble = counter.bubble.as_mut().expect("the Longsword's bubble");
        bubble.radius = Fx::ZERO;
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("counter's bubble")),
            "{errors:?}"
        );
        // A bubble written to a file and read back is the same (absent stays absent: the body's hurtboxes).
        let mut c = Content::placeholder();
        c.weapons[0].moves[MoveId::DownSpecial as usize]
            .counter
            .as_mut()
            .unwrap()
            .bubble
            .as_mut()
            .unwrap()
            .half_height = Fx::from_int(1);
        let text = crate::bundle::to_text(&c, "test", "", "", false);
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back.hash(), c.hash());
    }

    #[test]
    fn a_burst_and_motion_drift_round_trip_and_are_checked() {
        let base = Content::placeholder();
        let champion = base
            .fighters
            .iter()
            .position(|p| p.gauge.exists())
            .expect("the champion");
        let w = usize::from(base.fighters[champion].weapon);
        let burst_move = base.weapons[w]
            .moves
            .iter()
            .position(|m| m.projectile.is_some_and(|p| p.burst.is_some()))
            .expect("the powered wave bursts");
        // The shipped burst and the powered rise's motion_drift come back from a file unchanged.
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        assert!(text.contains("burst {") && text.contains("motion_drift 100"));
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // A burst with no phase, or out of order, and a motion_drift over 100 are refused.
        let mut c = base.clone();
        let b = c.weapons[w].moves[burst_move]
            .projectile
            .as_mut()
            .unwrap()
            .burst
            .as_mut()
            .unwrap();
        b.phases = [None; sim_core::moves::MAX_PHASES];
        let errors = validate(&c).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("burst")), "{errors:?}");
        let mut c = base.clone();
        c.weapons[w].moves[burst_move].motion_drift = 101;
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("motion_drift")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_bad_and_repeated_names() {
        let mut c = Content::placeholder();
        c.names.fighters[1] = c.names.fighters[0].clone();
        c.names.weapons[0] = "has space".to_string();
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("used twice")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("letters, digits")),
            "{errors:?}"
        );
        c = Content::placeholder();
        c.names.fighters.pop();
        assert!(validate(&c).is_err());
    }

    #[test]
    fn a_scripted_move_needs_no_hitboxes() {
        let mut c = Content::placeholder();
        c.weapons[0].moves[MoveId::DownSpecial as usize] = Move {
            total_frames: 20,
            script: Some(Program::compile(Kind::Fighter, "set_vel(0.3, 0);").unwrap()),
            ..Move::empty()
        };
        assert_eq!(validate(&c), Ok(()));
    }

    #[test]
    fn rejects_a_missing_weapon_and_bad_hitboxes() {
        let mut c = Content::placeholder();
        c.fighters[0].weapon = 10;
        c.weapons[0].moves[0].hitboxes[0].end = 200;
        c.weapons[1].moves.pop();
        let errors = validate(&c).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("weapon 10")), "{errors:?}");
        assert!(
            errors.iter().any(|e| e.contains("outside the move")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("needs exactly")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_an_absurd_weight() {
        let mut c = Content::placeholder();
        c.fighters[0].weight = Fx::from_int(5);
        assert!(validate(&c).is_err());
    }
}
