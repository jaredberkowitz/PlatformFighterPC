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
pub mod run;
pub mod stages;
pub mod tree;

pub use bundle::{load, to_text, Bundle, Manifest, SCHEMA_VERSION};
pub use run::check_run;

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
    validate_item_rules(&content.rules, &content.fighters, &mut errors);
    validate_previous_rules(&content.rules, &mut errors);
    validate_look(&content.look, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// The ruleset's item rules (E4, sim 111): every length at least 1 and every body's release frame inside its throw, the speeds and
/// the reach not negative, the angles within a turn.
fn validate_item_rules(
    r: &sim_core::content::Ruleset,
    fighters: &[FighterParams],
    errors: &mut Vec<String>,
) {
    let lengths = [
        r.item_pick_frames,
        r.item_side_frames,
        r.item_up_frames,
        r.item_down_frames,
        r.item_dash_frames,
        r.item_drop_frames,
    ];
    if lengths.contains(&0) {
        errors.push("ruleset: the item actions' lengths must be at least 1".to_string());
    }
    let values = [
        r.item_side_speed,
        r.item_side_smash,
        r.item_up_speed,
        r.item_up_smash,
        r.item_down_speed,
        r.item_down_smash,
        r.item_dash_speed,
        r.item_drop_speed,
        r.item_reach_front,
        r.item_reach_back,
        r.item_reach_up,
    ];
    if values.iter().any(|v| *v < Fx::ZERO || *v > MAX_VALUE) {
        errors
            .push("ruleset: the item speeds and reach cannot be negative or too large".to_string());
    }
    let angles = [
        r.item_side_angle,
        r.item_side_smash_angle,
        r.item_dash_angle,
        r.item_drop_angle,
    ];
    if angles.iter().any(|a| !(-180..=180).contains(a)) {
        errors.push("ruleset: the item throw angles must be from -180 to 180".to_string());
    }
    for (i, f) in fighters.iter().enumerate() {
        let t = &f.item_throws;
        let late = t.forward > r.item_side_frames
            || t.back > r.item_side_frames
            || t.up > r.item_up_frames
            || t.down > r.item_down_frames
            || t.dash > r.item_dash_frames
            || t.drop > r.item_drop_frames;
        if late {
            errors.push(format!(
                "fighter {i}: an item throw lets go after the ruleset's length for it"
            ));
        }
    }
}

/// The previous reference version's numbers (E40, sim 113): none negative or too large.
fn validate_previous_rules(r: &sim_core::content::Ruleset, errors: &mut Vec<String>) {
    let values = [
        r.prev_damage_mult,
        r.prev_hitlag_per_damage,
        r.prev_hitlag_base,
        r.prev_rage_max,
    ];
    if values.iter().any(|v| *v < Fx::ZERO || *v > MAX_VALUE) {
        errors.push(
            "ruleset: prev_damage_mult, prev_hitlag_per_damage, prev_hitlag_base and prev_rage_max cannot be negative or too large"
                .to_string(),
        );
    }
    // (A soft trip's get-up options cannot come after its own get-up: E44, sim 117.)
    if r.slip_lag > r.slip_max {
        errors.push("ruleset: slip_lag cannot be above slip_max".to_string());
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
        // Sim 121 (E53): the partner gauge's rules.
        if g.per_frame < Fx::ZERO || g.per_frame > Fx::from_int(100) {
            errors.push(format!(
                "fighter {i}: the gauge's per_frame must be 0 to 100"
            ));
        }
        if g.behind
            .iter()
            .chain(g.crowd.iter().flatten())
            .any(|&m| m == 0 || m > 4000)
        {
            errors.push(format!(
                "fighter {i}: the gauge's behind and crowd multipliers must be 1 to 4000 (per mille)"
            ));
        }
        if g.mode_taken
            .iter()
            .any(|&v| v < Fx::ZERO || v > Fx::from_int(1000))
        {
            errors.push(format!(
                "fighter {i}: the gauge's mode_taken must be 0 to 1000 frames a percent"
            ));
        }
        if g.ko_keep.iter().any(|&v| v < Fx::ZERO || v > g.max) {
            errors.push(format!(
                "fighter {i}: the gauge's ko_keep must be 0 to its max"
            ));
        }
    }
    // Sim 111 (E4): the item throws' release frames.
    if f.item_throws
        .frames()
        .iter()
        .any(|(_, v)| *v == 0 || *v > 60)
    {
        errors.push(format!(
            "fighter {i}: the item throws' release frames must be from 1 to 60"
        ));
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
    // Sim 125 (E57): the wing.
    if let Some(w) = &f.wing {
        if let Some(e) = wing_error(w) {
            errors.push(format!("fighter {i}: the wing: {e}"));
        }
    }
}

/// What is wrong with a wing's numbers (E57), if anything.
fn wing_error(w: &sim_core::content::Wing) -> Option<&'static str> {
    let in_range = |p: Fx| p >= Fx::ZERO && p <= Fx::from_int(999);
    if w.points
        .iter()
        .any(|&p| p <= Fx::ZERO || p > Fx::from_int(10_000))
    {
        Some("its points must be above 0 (at most 10000)")
    } else if w
        .per_dealt
        .iter()
        .any(|&p| p < Fx::ZERO || p > Fx::from_int(100))
        || w.ko_points < Fx::ZERO
        || w.ko_points > Fx::from_int(10_000)
    {
        Some("per_dealt must be 0 to 100 and ko_points 0 to 10000")
    } else if w.percents().iter().any(|&(_, p)| p == 0 || p > 1000) || w.jumps > 5 {
        Some("its percents must be 1 to 1000 and jumps at most 5")
    } else if w.rows.iter().any(|r| {
        let n = usize::from(r.count);
        n == 0
            || r.percent[..n].iter().any(|&p| !in_range(p))
            || r.diff[..n].windows(2).any(|d| d[1] > d[0])
    }) {
        Some("each start row needs points (percents 0 to 999) with differences that never rise")
    } else {
        None
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

/// The most items one weapon may make (E4, sim 111).
const MAX_ITEM_SPECS: usize = 8;

/// An item's numbers (E4, sim 111; see `sim_core::moves::ItemSpec`).
fn validate_item(i: usize, n: usize, it: &sim_core::moves::ItemSpec, errors: &mut Vec<String>) {
    let mut copy = *it;
    // (The damage modifier, sim 117, may be negative: the reference's modifiers are often -0.4.)
    let negative = copy
        .fx_fields_mut()
        .iter()
        .any(|(name, v)| (**v < Fx::ZERO && *name != "damage_add") || **v > MAX_VALUE);
    if negative || it.damage_add < -MAX_VALUE {
        errors.push(format!(
            "weapon {i} item {n}: its values cannot be negative (damage_add aside) or above {}",
            MAX_VALUE.floor_int()
        ));
    }
    // A trap (E45, sim 117): a real circle that trips (a slip, hard or soft), its kick not negative upward.
    if let Some(t) = &it.trap {
        if t.hit.radius <= Fx::ZERO
            || t.hit.slip == sim_core::moves::SLIP_NONE
            || t.hit.slip > sim_core::moves::SLIP_SOFT
            || t.kick_x < Fx::ZERO
            || t.kick_y < Fx::ZERO
            || t.hit.damage < Fx::ZERO
        {
            errors.push(format!(
                "weapon {i} item {n}: a trap needs a hit radius above 0, a slip (hard or soft), a damage and kicks not negative"
            ));
        }
    }
    let keeps_ok = copy.u8_fields_mut().iter().all(|(_, v)| **v <= 200);
    let ordered = it.bounce_x.0 <= it.bounce_x.1
        && it.bounce_y.0 <= it.bounce_y.1
        && it.knock_speed.0 <= it.knock_speed.1
        && it.knock_damage.0 <= it.knock_damage.1;
    let ok = it.kind > 0
        && keeps_ok
        && ordered
        && ((it.pick_width > Fx::ZERO && it.pick_height > Fx::ZERO) || it.placed.is_some())
        && it.throw_mul > Fx::ZERO
        && it.hit.radius > Fx::ZERO
        && it.hit.slip <= sim_core::moves::SLIP_SOFT
        && (it.gravity == Fx::ZERO || it.max_fall > Fx::ZERO)
        && (it.hp == Fx::ZERO || it.hurt_radius > Fx::ZERO);
    if !ok {
        errors.push(format!(
            "weapon {i} item {n}: needs a kind from 1 to 255, a pick-up box (unless placed), a throw_mul and a hit radius above 0, a max_fall with \
             gravity, a hurt_radius with hp, percents of at most 200 and its ranges low to high"
        ));
    }
    if let Some(b) = &it.blast {
        if b.frames == 0 || b.hit.radius <= Fx::ZERO {
            errors.push(format!(
                "weapon {i} item {n}: a blast needs frames and a hit radius above 0"
            ));
        }
    }
}

fn validate_weapon(i: usize, w: &Weapon, errors: &mut Vec<String>) {
    if w.items.len() > MAX_ITEM_SPECS {
        errors.push(format!("weapon {i}: at most {MAX_ITEM_SPECS} items"));
    }
    for (n, it) in w.items.iter().enumerate() {
        validate_item(i, n, it, errors);
        // A placed piece (E43, sim 115): what it grows into and drops is one of the weapon's items, its shots real moves with a
        // projectile, its numbers in range.
        if let Some(pl) = &it.placed {
            let item_ok = |k: u8| usize::from(k) < w.items.len();
            let shot_ok = |s: u8| {
                s == 255
                    || (usize::from(s) < MoveId::COUNT
                        && w.moves
                            .get(usize::from(s))
                            .is_some_and(|m| !m.is_empty() && m.projectile.is_some()))
            };
            if (pl.water > 0 && !item_ok(pl.grow_into))
                || (pl.chip != 255 && !item_ok(pl.chip))
                || !shot_ok(pl.grow_move)
                || !shot_ok(pl.fall_move)
                || pl.hitter_lag > 200
                || pl.chip_chance > 1000
                || pl.hurt_half_height < Fx::ZERO
            {
                errors.push(format!(
                    "weapon {i} item {n}: a placed piece's grow_into and chip must be the weapon's items, grow_move and fall_move moves with a projectile, hitter_lag at most 200, chip_chance at most 1000 and hurt_half_height not negative"
                ));
            }
        }
    }
    if w.moves.len() != MoveId::COUNT {
        errors.push(format!(
            "weapon {i}: needs exactly {} moves, has {}",
            MoveId::COUNT,
            w.moves.len()
        ));
    }
    // Moves some aerial lands into (a landing hit): their hitboxes may come out on their first frame, the landing frame.
    let landing_moves: Vec<u8> = w
        .moves
        .iter()
        .flat_map(|mv| [mv.landing_move, mv.fall_landing_move])
        .flatten()
        .collect();
    // A thrown tool's catch move (E3d) is an animation only: it may have nothing in it but its frames.
    let catch_moves: Vec<u8> = w
        .moves
        .iter()
        .filter_map(|mv| mv.projectile.and_then(|p| p.tool.catch_move))
        .collect();
    for (m, mv) in w.moves.iter().enumerate() {
        let id = MoveId::from_index(m as u8);
        let name = id.name();
        for (field, slot) in [
            ("landing_move", mv.landing_move),
            ("on_search", mv.on_search),
            ("air_version", mv.air_version),
            ("smash_version", mv.smash_version),
            ("catch", mv.projectile.and_then(|p| p.tool.catch_move)),
            ("catch_move", mv.catch_move),
            ("off_edge", mv.off_edge),
            ("limit_version", mv.limit_version),
            ("on_land", mv.on_land),
            ("detect_then", mv.reflector.and_then(|r| r.detect_then)),
            ("shed", mv.shed),
            // Sim 128.
            ("fall_landing_move", mv.fall_landing_move),
            ("probe miss", mv.probe.map(|p| p.miss)),
            (
                "counter break_move",
                mv.counter
                    .filter(|c| c.break_above != Fx::ZERO)
                    .map(|c| c.break_move),
            ),
        ] {
            if let Some(l) = slot {
                if usize::from(l) >= MoveId::COUNT
                    || w.moves.get(usize::from(l)).is_none_or(Move::is_empty)
                {
                    errors.push(format!("weapon {i} {name}: {field} needs a real move"));
                }
            }
        }
        // (A shed move must have the projectile it sheds: E48, sim 117.)
        if let Some(s) = mv.shed {
            if w.moves
                .get(usize::from(s))
                .is_some_and(|sm| sm.projectile.is_none())
            {
                errors.push(format!(
                    "weapon {i} {name}: shed needs a move with a projectile"
                ));
            }
        }
        if mv.is_empty() {
            // Only special moves may be left unimplemented.
            if !id.may_be_empty() {
                errors.push(format!("weapon {i} {name}: has no frames"));
            }
            continue;
        }
        // Random versions (E39, sim 113): real moves of their own (not drawing again), chances summing to at most 100.
        if !mv.versions.is_empty() {
            let total: u32 = mv.versions.iter().map(|(c, _)| u32::from(*c)).sum();
            if total > 100 || mv.versions.iter().any(|(c, _)| *c == 0) {
                errors.push(format!(
                    "weapon {i} {name}: the versions' chances must be above 0 and add up to at most 100"
                ));
            }
            for (_, slot) in &mv.versions {
                let real = usize::from(*slot) < MoveId::COUNT
                    && usize::from(*slot) != m
                    && w.moves
                        .get(usize::from(*slot))
                        .is_some_and(|v| !v.is_empty() && v.versions.is_empty());
                if !real {
                    errors.push(format!(
                        "weapon {i} {name}: each version needs a real move of its own (with no versions)"
                    ));
                }
            }
        }
        // A jab cancel (sim 115): a frame inside the move.
        if mv.cancel_from != 255 && mv.cancel_from >= mv.total_frames {
            errors.push(format!(
                "weapon {i} {name}: cancel_from must be a frame of the move (below its total frames) or absent"
            ));
        }
        // The pocket and its release (E42, sim 115): frames inside the move, a real circle, a power of 1 to 1000 percent.
        if let Some(pc) = &mv.pocket {
            if pc.start > pc.end || pc.end >= mv.total_frames || pc.radius <= Fx::ZERO {
                errors.push(format!(
                    "weapon {i} {name}: a pocket needs start at most end, end inside the move and a radius above 0"
                ));
            }
        }
        if let Some(r) = &mv.unpocket {
            if r.frame >= mv.total_frames || r.power == 0 || r.power > 1000 {
                errors.push(format!(
                    "weapon {i} {name}: an unpocket needs its frame inside the move and a power of 1 to 1000"
                ));
            }
        }
        // Balloons (E41, sim 113).
        if let Some(b) = &mv.balloons {
            if b.count == 0
                || b.count > 8
                || b.hp <= Fx::ZERO
                || b.radius <= Fx::ZERO
                || b.fuel == 0
                || b.fuel > 4000
                || b.one_below > b.fuel
                || b.pop_vy < Fx::ZERO
            {
                errors.push(format!(
                    "weapon {i} {name}: balloons need count 1 to 8, hp and radius above 0, fuel 1 to 4000, one_below at most fuel and pop_vy not below 0"
                ));
            }
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
        // An egg (E8): its throw rows and its numbers go together, and its numbers make sense.
        let encase_rows = mv
            .hitboxes
            .iter()
            .filter(|h| h.effect == sim_core::moves::EFFECT_ENCASE)
            .collect::<Vec<_>>();
        if encase_rows
            .iter()
            .any(|h| h.kind != sim_core::moves::HIT_THROW)
        {
            errors.push(format!(
                "weapon {i} {name}: only a throw hitbox can have effect encase"
            ));
        }
        match &mv.encase {
            Some(e) => {
                if encase_rows.is_empty() {
                    errors.push(format!(
                        "weapon {i} {name}: an encase block needs a throw hitbox with effect encase"
                    ));
                }
                if e.base <= sim_core::Fx::ZERO
                    || e.per_percent < sim_core::Fx::ZERO
                    || e.percent_cap < sim_core::Fx::ZERO
                    || e.mash_stick < sim_core::Fx::ZERO
                    || e.mash_button < sim_core::Fx::ZERO
                    || e.hit_cut < sim_core::Fx::ZERO
                {
                    errors.push(format!(
                        "weapon {i} {name}: an encase needs base above 0 and per_percent, percent_cap, mash_stick, mash_button and hit_cut not below 0"
                    ));
                }
                if e.damage_taken > 200
                    || e.grow_from == 0
                    || e.grow_from > 100
                    || e.size == 0
                    || e.size > 200
                {
                    errors.push(format!(
                        "weapon {i} {name}: an encase needs damage_taken 0 to 200, grow_from 1 to 100 and size 1 to 200"
                    ));
                }
            }
            None => {
                if !encase_rows.is_empty() {
                    errors.push(format!(
                        "weapon {i} {name}: a throw hitbox with effect encase needs the move's encase block"
                    ));
                }
            }
        }
        if mv.hitboxes.is_empty()
            && mv.projectile.is_none()
            && mv.script.is_none()
            && mv.counter.is_none()
            && !catch_moves.contains(&(m as u8))
            && mv.carry.is_none()
            && mv.motion.is_empty()
            && mv.pull.is_none()
            && mv.pocket.is_none()
            && mv.unpocket.is_none()
            && mv.toss.is_none()
            && mv.reflector.is_none()
            // (Sim 125: a follow-up slot may be only time: the blade dash's end in the air, the flare's cancel.)
            && !MoveId::from_index(m as u8).is_ext()
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
            // (Sim 125, E58: phases with the same start are rings, out together; they share a hit group.)
            let mut last = 0;
            let mut group = 0;
            for (from, hb) in p.phases.iter().flatten() {
                if *from == 0
                    || *from < last
                    || *from >= p.life
                    || (*from == last && hb.group != group)
                {
                    errors.push(format!(
                        "weapon {i} {name}: projectile phases must start in order, after age 0 and within its life (rings sharing a start share a group)"
                    ));
                }
                last = *from;
                group = hb.group;
            }
            if let Some(b) = &p.burst {
                let mut last = 0;
                let mut group = 0;
                let mut ordered = b.life > 0;
                for (from, hb) in b.phases.iter().flatten() {
                    ordered &= *from > 0
                        && *from >= last
                        && (*from > last || hb.group == group)
                        && *from < b.life
                        && hb.radius >= Fx::ZERO
                        && hb.half_height >= Fx::ZERO;
                    last = *from;
                    group = hb.group;
                }
                if !ordered || last == 0 {
                    errors.push(format!(
                        "weapon {i} {name}: a projectile's burst needs a lifetime and at least one phase, its phases in order, \
                         after age 0 and within its life, with no negative radius or half_height"
                    ));
                }
            }
            // Sim 99: drag and lodging (E35, E3b), a charge-scaled shot (E35), a thrown tool (E3d).
            if p.arc.drag < Fx::ZERO || p.arc.lodge_angle > 90 {
                errors.push(format!(
                    "weapon {i} {name}: a projectile's drag cannot be negative and its lodge_angle is at most 90"
                ));
            }
            if let Some(c) = &p.charged {
                let ok = c.full > 0
                    && c.damage_to >= Fx::ZERO
                    && c.speed_to >= Fx::ZERO
                    && c.life_to > 0
                    && (10..=1000).contains(&c.size_from)
                    && (10..=1000).contains(&c.size_to)
                    && p.end_damage == p.hitbox.damage;
                if !ok {
                    errors.push(format!(
                        "weapon {i} {name}: a charged projectile needs full above 0, no negative damage_to or speed_to, a                          life_to, sizes from 10 to 1000 and no damage falloff (end_damage equal to its damage)"
                    ));
                }
            }
            if let Some(fall) = &p.tool.on_clank {
                if fall.life == 0 || fall.gravity < Fx::ZERO || fall.max_fall < Fx::ZERO {
                    errors.push(format!(
                        "weapon {i} {name}: a projectile's clank fall needs a life and no negative gravity or max_fall"
                    ));
                }
            }
            // Sim 111 (E4): the charged share, and a lodge that makes an item.
            if p.charged
                .is_some_and(|c| c.power_to == 0 || c.power_to > 255)
            {
                errors.push(format!(
                    "weapon {i} {name}: a charged projectile's power_to is a percent from 1 to 255"
                ));
            }
            if p.charged.is_some_and(|c| c.times == 0 || c.times > 100) {
                errors.push(format!(
                    "weapon {i} {name}: a charged projectile's times is a percent from 1 to 100"
                ));
            }
            if let Some(item) = p.arc.lodge_item {
                if usize::from(item) >= w.items.len() || p.arc.lodge == 0 || p.arc.bounces > 0 {
                    errors.push(format!(
                        "weapon {i} {name}: a projectile's lodge_item must be one of the weapon's items, with a lodge and no bounces"
                    ));
                }
            }
            if p.tool.catch_move.is_some() && p.tool.catch_from == 255 {
                errors.push(format!(
                    "weapon {i} {name}: a projectile's catch_move needs a catch_from age"
                ));
            }
        }
        // Sim 111 (E4): an item made in the hand and an item used up, inside the move.
        if let Some((frame, item)) = mv.pull {
            if usize::from(item) >= w.items.len() || frame >= mv.total_frames {
                errors.push(format!(
                    "weapon {i} {name}: pull_item must be one of the weapon's items and pull_frame inside the move"
                ));
            }
        }
        if mv.spend_item != 255 && mv.spend_item >= mv.total_frames {
            errors.push(format!(
                "weapon {i} {name}: spend_item must be a frame inside the move (or 255 for none)"
            ));
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
            if p.start > p.end
                || p.end > mv.total_frames
                || p.parts == 0
                || p.parts > 15
                || p.mode > sim_core::moves::MODE_POWERED
            {
                errors.push(format!(
                    "weapon {i} {name}: part_intangible {}..{} must lie inside the move, name at least one part and a mode any, plain or powered",
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
            .any(|hb| hb.half_height < Fx::ZERO || hb.half_width < Fx::ZERO)
        {
            errors.push(format!(
                "weapon {i} {name}: a hitbox's half_height and half_width cannot be negative"
            ));
        }
        if mv
            .hitboxes
            .iter()
            .chain(mv.projectile.iter().flat_map(|p| {
                std::iter::once(&p.hitbox).chain(p.phases.iter().flatten().map(|(_, hb)| hb))
            }))
            .any(|hb| hb.rules > sim_core::moves::RULES_PREVIOUS)
        {
            errors.push(format!(
                "weapon {i} {name}: a hitbox's rules must be current or previous"
            ));
        }
        if mv
            .hitboxes
            .iter()
            .chain(mv.projectile.iter().flat_map(|p| {
                std::iter::once(&p.hitbox).chain(p.phases.iter().flatten().map(|(_, hb)| hb))
            }))
            .any(|hb| hb.slip > sim_core::moves::SLIP_SOFT)
        {
            errors.push(format!(
                "weapon {i} {name}: a hitbox's slip must be none, hard or soft"
            ));
        }
        // Sim 119: a latch (E49) rides at a height above 0 with a clock divisor above 0; a flag version reads some flag or mark and
        // names a real move, each one; damage to the user not negative.
        if mv
            .latch
            .is_some_and(|l| l.height == 0 || l.clock_den == 0 || l.x.abs() > MAX_VALUE)
        {
            errors.push(format!(
                "weapon {i} {name}: a latch needs a height and a clock_den above 0"
            ));
        }
        for v in &mv.flag_versions {
            let bad_slot = v.slot.is_some_and(|l| {
                usize::from(l) >= MoveId::COUNT
                    || w.moves.get(usize::from(l)).is_none_or(Move::is_empty)
            });
            if (v.flags == 0 && v.marks == 0) || bad_slot {
                errors.push(format!(
                    "weapon {i} {name}: a flag version needs flags or marks and, if it names a move, a real one"
                ));
            }
        }
        if mv.self_damage < Fx::ZERO || mv.self_damage > Fx::from_int(999) {
            errors.push(format!(
                "weapon {i} {name}: self_damage must be from 0 to 999"
            ));
        }
        // Sim 121: the guard's armor damage and hitlag (E55), its gauge gain (E53), a projectile's curse (E54), the partner's boxes
        // (E52).
        if mv.armor_damage > 1000 || mv.armor_hitlag == 0 || mv.armor_hitlag > 400 {
            errors.push(format!(
                "weapon {i} {name}: armor_damage must be 0 to 1000 (per mille) and armor_hitlag 1 to 400 (percent)"
            ));
        }
        if mv.gauge_gain > 10_000 {
            errors.push(format!(
                "weapon {i} {name}: gauge_gain must be 0 to 10000 (per mille)"
            ));
        }
        // Sim 128: orbs (E59), a counter answer that grows with its catch, a ground probe, a counter's break.
        if let Some(o) = &mv.orbs {
            let hit_ok =
                |hb: &sim_core::moves::Hitbox| hb.radius > Fx::ZERO && hb.damage >= Fx::ZERO;
            if mv.projectile.is_none()
                || o.count == 0
                || o.reflected == 0
                || o.per_target == 0
                || o.per_owner == 0
                || o.wait == 0
                || o.cap < o.wait
                || o.full == 0
                || o.period == 0
                || o.strike == 0
                || o.orbit_hits == 0
                || o.loose_life == 0
                || o.radius < Fx::ZERO
                || o.loose_speed < Fx::ZERO
                || o.extend_to < o.extend
                || !hit_ok(&o.orbit_hit)
                || !hit_ok(&o.strike_hit)
            {
                errors.push(format!(
                    "weapon {i} {name}: orbs need a projectile, a count, reflected, per_target, per_owner, wait, full, period, strike, orbit_hits and loose_life of at least 1, a cap no shorter than the wait, extend_to no less than extend, no negative radius or loose_speed, and two hits with a radius"
                ));
            }
        }
        if let Some(l) = &mv.lerp {
            let pairs = mv.projectile.is_some_and(|p| {
                p.charged.is_none()
                    && l.phases
                        .iter()
                        .zip(p.phases.iter())
                        .all(|(a, b)| match (a, b) {
                            (Some((fa, ha)), Some((fb, _))) => fa == fb && ha.radius > Fx::ZERO,
                            (None, _) => true,
                            _ => false,
                        })
            });
            if l.full <= Fx::ZERO || !pairs {
                errors.push(format!(
                    "weapon {i} {name}: a lerp needs a full above 0 and a projectile, not charged, whose phases its own phases pair with (the same starts, each with a radius)"
                ));
            }
        }
        if let Some(p) = &mv.probe {
            if p.frame == 0 || p.frame > mv.total_frames {
                errors.push(format!(
                    "weapon {i} {name}: a probe's frame must lie inside the move"
                ));
            }
        }
        if let Some(c) = &mv.counter {
            if c.break_above < Fx::ZERO {
                errors.push(format!(
                    "weapon {i} {name}: the counter's break_above must not be negative"
                ));
            }
        }
        if let Some(c) = &mv.curse {
            if c.damage <= Fx::ZERO
                || c.damage > Fx::from_int(100)
                || c.every == 0
                || c.frames == 0
                || mv.projectile.is_none()
            {
                errors.push(format!(
                    "weapon {i} {name}: a curse needs a projectile, a damage above 0 (at most 100), every and frames at least 1"
                ));
            }
        }
        if mv
            .hitboxes
            .iter()
            .any(|hb| hb.mode > sim_core::moves::MODE_POWERED || hb.part > 1)
        {
            errors.push(format!(
                "weapon {i} {name}: a hitbox's mode must be any, plain or powered and its part 0 or 1"
            ));
        }
        // Sim 126: a stale dodge's stretch (owner decision 85) is one of the ruleset's dodge rates and needs a staling dodge; a
        // stopped grab's early end (owner decision 84) needs a blockable grab box and lies inside the move.
        if mv.stale_stretch > sim_core::moves::STRETCH_SPOT
            || (mv.stale_stretch != sim_core::moves::STRETCH_NONE && !mv.dodge_stale)
        {
            errors.push(format!(
                "weapon {i} {name}: stale_stretch must be none, roll, back_roll or spot, and needs dodge_stale"
            ));
        }
        if mv.shield_end > 0
            && (mv.shield_end >= mv.total_frames || !mv.hitboxes.iter().any(|hb| hb.blockable))
        {
            errors.push(format!(
                "weapon {i} {name}: shield_end needs a blockable grab box and must be shorter than the move"
            ));
        }
        // Sim 125 (E57): an armor mode names when an armor works, so it needs an armor.
        if mv.armor_mode > sim_core::moves::MODE_POWERED
            || (mv.armor_mode != sim_core::moves::MODE_ANY && mv.armor.is_none())
        {
            errors.push(format!(
                "weapon {i} {name}: armor_mode must be any, plain or powered, and needs an armor"
            ));
        }
        // Sim 123: a blockable box is a grab box (E56); a tether's frames lie inside the move, its reach is not negative (at most 64
        // world units), it reels in at least one frame, its drag is at most 1000 per mille and it allows at least one catch.
        if mv
            .hitboxes
            .iter()
            .any(|hb| hb.blockable && hb.kind != sim_core::moves::HIT_GRAB)
        {
            errors.push(format!(
                "weapon {i} {name}: only a grab hitbox can be blockable"
            ));
        }
        if let Some(t) = &mv.tether {
            let reach = [t.ahead, t.behind, t.up, t.down];
            if t.from > t.to
                || t.to >= mv.total_frames
                || reach.iter().any(|v| *v < Fx::ZERO || *v > MAX_VALUE)
                || t.reel == 0
                || t.drag > 1000
                || t.limit == 0
            {
                errors.push(format!(
                    "weapon {i} {name}: a tether needs from..to inside the move, a reach from 0 to 64, a reel of at least 1 frame, a drag of at most 1000 and a limit of at least 1"
                ));
            }
        }
        // A move's toss (E47, sim 117): its frames inside the move, its speeds not negative, its angle a real one.
        if let Some(t) = &mv.toss {
            let inside = |f: u8| f < mv.total_frames;
            if !inside(t.frame)
                || (t.again != 255 && !inside(t.again))
                || t.speed < Fx::ZERO
                || t.smash_speed < Fx::ZERO
                || t.speed > MAX_VALUE
                || t.smash_speed > MAX_VALUE
                || !(-180..=180).contains(&t.angle)
            {
                errors.push(format!(
                    "weapon {i} {name}: a toss needs its frame (and again) inside the move, speeds not negative and an angle from -180 to 180"
                ));
            }
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

/// A slope's limits (sim 124): at most 1 in 2 (about 26.6 degrees: the brief's gentle ramps up to about 25), and a solid sloped
/// block's underside below its whole top.
pub fn slope_problem(p: &sim_core::content::Platform) -> Option<String> {
    if !p.sloped() {
        return None;
    }
    let width = p.right - p.left;
    if p.rise.abs().mul_int(2) > width {
        return Some("slopes more than 1 in 2 (about 26.6 degrees)".to_string());
    }
    if !p.pass_through && p.bottom >= p.y.min(p.top_right()) {
        return Some("is a sloped block whose bottom is not below its whole top".to_string());
    }
    None
}

/// Whether a ledge sits at a sloped platform's corner (ledges hang only from flat corners).
pub fn ledge_on_slope(
    l: &sim_core::content::Ledge,
    platforms: &[sim_core::content::Platform],
) -> bool {
    platforms.iter().any(|p| {
        p.sloped() && ((l.x == p.left && l.y == p.y) || (l.x == p.right && l.y == p.top_right()))
    })
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
    for (i, p) in s.platforms.iter().enumerate() {
        if let Some(e) = slope_problem(p) {
            errors.push(format!("stage: platform {i} {e}"));
        }
    }
    for (i, l) in s.ledges.iter().enumerate() {
        if ledge_on_slope(l, &s.platforms) {
            errors.push(format!(
                "stage: ledge {i} hangs from a sloped platform's corner (ledges only at flat corners)"
            ));
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
    fn the_pocket_and_tree_fields_round_trip_and_are_checked() {
        // Sim 115: the settler's pocket and release (E42), its chopping axe, the rocket's and the line's pocket rules, the water, the
        // sapling's and tree's placed blocks and pocketability (E43) come back from a file unchanged; a file without them keeps
        // today's behaviour; bad values are refused.
        use sim_core::moves::{POCKET_FREES, POCKET_NEVER, POCKET_YES};
        let base = Content::placeholder();
        let w = usize::from(base.fighters[10].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "pocket {",
            "unpocket {",
            "chops true",
            "pocket frees",
            "pocket never",
            "water 1",
            "pocketable false",
            "placed {",
            "grow_move ext14",
            "fall_move ext15",
            "chip_chance 485",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no pocket, every shot pocketable by a pocket that exists, no placed pieces, nothing chops.
        let stripped = text
            .lines()
            .filter(|l| {
                !l.contains("pocket frees") && !l.contains("pocket never") && !l.contains("chops")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let older = crate::bundle::load(&stripped).expect("reads").content;
        let rocket = older.weapons[w].moves[MoveId::SideSpecial as usize]
            .projectile
            .expect("the rocket");
        assert_eq!(rocket.tool.pocket, POCKET_YES);
        assert!(!older.weapons[w].moves[MoveId::Ext12 as usize].chops);
        assert_eq!(
            base.weapons[w].moves[MoveId::SideSpecial as usize]
                .projectile
                .map(|p| p.tool.pocket),
            Some(POCKET_FREES)
        );
        assert_eq!(
            base.weapons[2].moves[MoveId::SideSpecial as usize]
                .projectile
                .map(|p| p.tool.pocket),
            Some(POCKET_NEVER)
        );
        // A word that is not a pocket rule.
        let bad = text.replacen("pocket frees", "pocket maybe", 1);
        assert!(crate::bundle::load(&bad).is_err());
        // Bad values: a pocket past the move, a release of power 0, a placed piece growing into an item the weapon has not, a chance
        // over 1000.
        let mut c = base.clone();
        if let Some(p) = c.weapons[w].moves[MoveId::NSpecial as usize]
            .pocket
            .as_mut()
        {
            p.end = 250;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("pocket")));
        let mut c = base.clone();
        if let Some(p) = c.weapons[w].moves[MoveId::Ext9 as usize].unpocket.as_mut() {
            p.power = 0;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("unpocket")));
        let mut c = base.clone();
        if let Some(p) = c.weapons[w].items[0].placed.as_mut() {
            p.grow_into = 7;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("placed")));
        let mut c = base.clone();
        if let Some(p) = c.weapons[w].items[1].placed.as_mut() {
            p.chip_chance = 1001;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("placed")));
    }

    #[test]
    fn the_popgun_banana_fields_round_trip_and_are_checked() {
        // Sim 117: the Popgun's (class 11) peel: the slip hits (E44), the toss (E47), the item's damage modifier, uses and trap (E45),
        // and the ruleset's soft-trip timings come back from a file unchanged; a file without them keeps today's behaviour; bad values
        // are refused.
        use sim_core::moves::{SLIP_HARD, SLIP_NONE, SLIP_SOFT};
        let base = Content::placeholder();
        let w = usize::from(base.fighters[11].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "slip hard",
            "slip soft",
            "toss {",
            "smash_speed",
            "wait 85",
            "again 12",
            "uses 2",
            "damage_add 1",
            "trap {",
            "kick_y",
            "slip_lag 30",
            "slip_max 90",
            "shed ext8",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no slip, no toss, no modifier, no limit, no trap, the standard soft-trip timings.
        let stripped = text
            .replace(" slip hard", "")
            .lines()
            .filter(|l| {
                !l.trim_start().starts_with("uses ")
                    && !l.contains("damage_add")
                    && !l.contains("slip_lag")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let older = crate::bundle::load(&stripped)
            .unwrap_or_else(|e| panic!("{}", e.join("\n")))
            .content;
        let peel = older.weapons[w].items[0];
        assert_eq!(peel.hit.slip, SLIP_NONE);
        assert_eq!((peel.uses, peel.damage_add), (0, Fx::ZERO));
        assert_eq!(older.rules.slip_lag, 30);
        assert_eq!(base.weapons[w].items[0].hit.slip, SLIP_HARD);
        assert_eq!(
            base.weapons[w].items[0].trap.map(|t| t.hit.slip),
            Some(SLIP_SOFT)
        );
        // A word that is not a slip.
        let bad = text.replacen("slip hard", "slip maybe", 1);
        assert!(crate::bundle::load(&bad).is_err());
        // Bad values: a toss past the move, a trap that does not trip, a slip out of range, a soft trip's options after its get-up.
        let mut c = base.clone();
        if let Some(t) = c.weapons[w].moves[MoveId::DownSpecial as usize]
            .toss
            .as_mut()
        {
            t.frame = 250;
        }
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("toss")));
        let mut c = base.clone();
        if let Some(t) = c.weapons[w].items[0].trap.as_mut() {
            t.hit.slip = SLIP_NONE;
        }
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("trap")));
        let mut c = base.clone();
        c.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].slip = 3;
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("slip")));
        let mut c = base.clone();
        c.rules.slip_lag = 91;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("slip_lag")));
        // A move that sheds one with no projectile (E48).
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::UpSpecial as usize].shed = Some(MoveId::Jab as u8);
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("shed")));
    }

    #[test]
    fn the_popgun_flip_fields_round_trip_and_are_checked() {
        // Sim 119: the latch (E49), the flag versions (owner decision 75), a move's self damage, a hitbox's fixed stun and unblockable
        // flag come back from a file unchanged; a file without them keeps today's behaviour; bad values are refused.
        use sim_core::moves::popgun_slots::{LATCH, LATCH_ATTACK, ROCKET_AGAIN};
        let base = Content::placeholder();
        let w = usize::from(base.fighters[11].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "latch {",
            "height 90",
            "clock_num 1",
            "flag_version {",
            "marks 1",
            "flags 2",
            "move ext5",
            "self_damage 5",
            "fixed_stun 20",
            "unblockable true",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no clock, no versions, no self damage, the plain hitstun, shields block.
        let stripped = text
            .replace(" clock_num 1 clock_den 1", "")
            .replace(" fixed_stun 20", "")
            .replace(" unblockable true", "")
            .lines()
            .filter(|l| !l.trim_start().starts_with("self_damage"))
            .collect::<Vec<_>>()
            .join("\n");
        let older = crate::bundle::load(&stripped)
            .unwrap_or_else(|e| panic!("{}", e.join("\n")))
            .content;
        let latch = older.weapons[w].moves[LATCH as usize]
            .latch
            .expect("a latch");
        assert_eq!((latch.clock_num, latch.clock_den), (0, 1));
        assert!(older.weapons[w]
            .moves
            .iter()
            .all(|m| m.self_damage == Fx::ZERO
                && m.hitboxes
                    .iter()
                    .all(|h| h.fixed_stun == 0 && !h.unblockable)));
        assert_eq!(
            older.weapons[w].moves[MoveId::UpSpecial as usize].flag_versions[1].slot,
            Some(ROCKET_AGAIN as u8)
        );
        // Bad values: a latch without a height, a flag version reading nothing or naming an empty move, negative self damage.
        let mut c = base.clone();
        if let Some(l) = c.weapons[w].moves[LATCH_ATTACK as usize].latch.as_mut() {
            l.height = 0;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("latch")));
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::UpSpecial as usize].flag_versions[0].flags = 0;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("flag version")));
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::UpSpecial as usize].flag_versions[1].slot =
            Some(MoveId::Ext15 as u8);
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("flag version")));
        let mut c = base.clone();
        c.weapons[0].moves[MoveId::Jab as usize].self_damage = -Fx::ONE;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("self_damage")));
    }

    #[test]
    fn the_dagger_fields_round_trip_and_are_checked() {
        // Sim 121: a hitbox's mode and part (E52), a move's armor damage and hitlag (E55), its gauge gain (E53) and curse (E54), a
        // burst's `on_shield`, and the gauge's partner rules (E53) come back from a file unchanged; a file without them keeps today's
        // behaviour; bad values are refused.
        let base = Content::placeholder();
        let w = usize::from(base.fighters[12].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "mode powered",
            "mode plain",
            "part 1",
            "armor_damage 400",
            "armor_hitlag 150",
            "gauge_gain 6737",
            "curse {",
            "every 45",
            "on_shield false",
            "per_frame",
            "behind_1 1250",
            "crowd_3_1 1100",
            "crowd_4_2 1333",
            "mode_taken_2 16",
            "ko_keep 20",
            "ko_keep_3 30",
            "keeps true",
            "halts true",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: every box out always and the fighter's own, the whole damage, no curse, bursts on shields, the plain gauge.
        let stripped = text
            .replace(" mode powered", "")
            .replace(" mode plain", "")
            .replace(" part 1", "")
            .replace(" on_shield false", "")
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                ![
                    "armor_damage",
                    "armor_hitlag",
                    "gauge_gain",
                    "per_frame",
                    "behind_",
                    "crowd_",
                    "mode_taken_",
                    "ko_keep",
                    "keeps",
                    "halts",
                    "curse {",
                ]
                .iter()
                .any(|k| t.starts_with(k))
            })
            .collect::<Vec<_>>()
            .join("\n");
        let older = crate::bundle::load(&stripped)
            .unwrap_or_else(|e| panic!("{}", e.join("\n")))
            .content;
        let ow = &older.weapons[w];
        assert!(ow
            .moves
            .iter()
            .all(|m| m.armor_damage == 1000 && m.gauge_gain == 1000 && m.curse.is_none()));
        assert!(ow
            .moves
            .iter()
            .flat_map(|m| m.hitboxes.iter())
            .all(|h| h.mode == 0 && h.part == 0));
        let g = older.fighters[12].gauge;
        assert!(g.per_frame == Fx::ZERO && !g.keeps && !g.halts && g.ko_keep == [Fx::ZERO; 3]);
        // Bad values.
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::Jab as usize].hitboxes[0].mode = 3;
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("mode")));
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::DownSpecial as usize].armor_damage = 1001;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("armor_damage")));
        let mut c = base.clone();
        if let Some(cu) = c.weapons[w].moves[MoveId::SideSpecial as usize]
            .curse
            .as_mut()
        {
            cu.every = 0;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("curse")));
        let mut c = base.clone();
        c.fighters[12].gauge.ko_keep[2] = Fx::from_int(101);
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("ko_keep")));
        let mut c = base.clone();
        c.fighters[12].gauge.behind[0] = 0;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("behind")));
    }

    #[test]
    fn the_nodachi_fields_round_trip_and_are_checked() {
        // Sim 125: the exile's wing (E57), a move's armor mode and a motion that stops at the edge come back from a file unchanged; a
        // file without them keeps today's behaviour; bad values are refused.
        let base = Content::placeholder();
        let exile = 13;
        let w = usize::from(base.fighters[exile].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "wing {",
            "points_m2 110",
            "per_dealt 0.29999",
            "per_dealt_crowd 0.15",
            "ko_points 50",
            "jumps 1",
            "power 130",
            "run_accel 120",
            "point { stocks -2 diff -70 percent 30 }",
            "armor_mode powered",
            "edge_stop true",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no wing, armor always, motions leave edges.
        let stripped = text
            .replace(" armor_mode powered", "")
            .replace("armor_mode powered", "")
            .lines()
            .filter(|l| !l.trim_start().starts_with("edge_stop"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut out = String::new();
        let mut skip = 0usize;
        for l in stripped.lines() {
            let t = l.trim_start();
            if skip > 0 || t.starts_with("wing {") {
                skip += l.matches('{').count();
                skip -= l.matches('}').count();
                continue;
            }
            out.push_str(l);
            out.push('\n');
        }
        let older = crate::bundle::load(&out)
            .unwrap_or_else(|e| panic!("{}", e.join("\n")))
            .content;
        assert!(older.fighters.iter().all(|f| f.wing.is_none()));
        assert!(older.weapons[w]
            .moves
            .iter()
            .all(|m| !m.edge_stop && m.armor_mode == sim_core::moves::MODE_ANY));
        // Bad values.
        let mut c = base.clone();
        if let Some(wing) = c.fighters[exile].wing.as_mut() {
            wing.points[2] = Fx::ZERO;
        }
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("wing")));
        let mut c = base.clone();
        if let Some(wing) = c.fighters[exile].wing.as_mut() {
            wing.rows[0].diff[1] = 100;
        }
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("wing")));
        let mut c = base.clone();
        c.weapons[0].moves[sim_core::moves::MoveId::Jab as usize].armor_mode =
            sim_core::moves::MODE_POWERED;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("armor_mode")));
        let bad = text.replacen("point { stocks -2", "point { stocks -3", 1);
        assert!(crate::bundle::load(&bad).is_err());
    }
    #[test]
    fn the_nodachi_phase_2_fields_round_trip_and_are_checked() {
        // Sim 128: the exile's orbs (E59), a counter answer's lerp, its break and still answer, a ground probe, a special fall's
        // landing move and a charged shot's every_hit come back from a file unchanged; a file without them keeps today's behaviour;
        // bad values are refused.
        let base = Content::placeholder();
        let w = usize::from(base.fighters[13].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "orbs {",
            "per_target 5",
            "orbit_hit {",
            "strike_hit {",
            "lerp {",
            "probe {",
            "break_above 25",
            "still true",
            "fall_landing_move ext10",
            "every_hit true",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no orbs, no lerp, no probe, no break, a hitlag on the catch, the plain special fall landing.
        let stripped = text
            .replace(" every_hit true", "")
            .replace(" break_above 25 break_move ext8", "")
            .replace(" still true", "");
        let mut out = String::new();
        let mut skip = 0usize;
        for l in stripped.lines() {
            let t = l.trim_start();
            if skip > 0
                || t.starts_with("orbs {")
                || t.starts_with("lerp {")
                || t.starts_with("probe {")
            {
                skip += l.matches('{').count();
                skip -= l.matches('}').count();
                continue;
            }
            if [
                "every_hit",
                "break_above",
                "break_move",
                "still ",
                "fall_landing_move",
            ]
            .iter()
            .any(|f| t.starts_with(f))
            {
                continue;
            }
            out.push_str(l);
            out.push('\n');
        }
        let older = crate::bundle::load(&out)
            .unwrap_or_else(|e| panic!("{}", e.join("\n")))
            .content;
        assert!(older.weapons[w].moves.iter().all(|m| m.orbs.is_none()
            && m.lerp.is_none()
            && m.probe.is_none()
            && m.fall_landing_move.is_none()
            && m.counter
                .is_none_or(|c| c.break_above == Fx::ZERO && !c.still)
            && m.projectile
                .is_none_or(|p| p.charged.is_none_or(|c| !c.every_hit))));
        // Bad values.
        let side = sim_core::moves::MoveId::SideSpecial as usize;
        let mut c = base.clone();
        if let Some(o) = c.weapons[w].moves[side].orbs.as_mut() {
            o.cap = 10;
        }
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("orbs")));
        let mut c = base.clone();
        c.weapons[w].moves[side].orbs = None;
        c.weapons[0].moves[sim_core::moves::MoveId::Jab as usize].orbs =
            base.weapons[w].moves[side].orbs;
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("orbs")));
        let a = sim_core::moves::nodachi_slots::BARRIER_ANSWER as usize;
        let mut c = base.clone();
        if let Some(l) = c.weapons[w].moves[a].lerp.as_mut() {
            l.phases[1] = l.phases[1].map(|(_, h)| (3, h));
        }
        assert!(validate(&c).unwrap_err().iter().any(|e| e.contains("lerp")));
        let mut c = base.clone();
        if let Some(p) = c.weapons[w].moves[sim_core::moves::MoveId::DSmash as usize]
            .probe
            .as_mut()
        {
            p.miss = sim_core::moves::MoveId::Ext15 as u8;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("probe")));
    }
    #[test]
    fn the_dagger_phase_2_fields_round_trip_and_are_checked() {
        // Sim 123: a move's tether (E56), a blockable grab box, a dodge that stales like a roll, a part intangibility window's mode, and
        // the phantom's gauge gain from damage dealt (owner decision 80) come back from a file unchanged; a file without them keeps
        // today's behaviour; bad values are refused.
        let base = Content::placeholder();
        let w = usize::from(base.fighters[12].weapon);
        let air = sim_core::moves::dagger_slots::HOOK_AIR as usize;
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "tether {",
            "reel 12",
            "drag 240",
            "limit 3",
            "blockable true",
            "dodge_stale true",
            "mode powered",
            "per_dealt 1.29999",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no tether, no blockable box, no staling dodge, every window always.
        let stripped = text
            .replace(" blockable true", "")
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("tether {")
                    && !t.starts_with("dodge_stale")
                    && !(t.starts_with("part_intangible") && t.contains("mode"))
            })
            .collect::<Vec<_>>()
            .join("\n");
        let older = crate::bundle::load(&stripped)
            .unwrap_or_else(|e| panic!("{}", e.join("\n")))
            .content;
        let ow = &older.weapons[w];
        assert!(ow
            .moves
            .iter()
            .all(|m| m.tether.is_none() && !m.dodge_stale));
        assert!(ow
            .moves
            .iter()
            .flat_map(|m| m.hitboxes.iter())
            .all(|h| !h.blockable));
        // Bad values.
        let mut c = base.clone();
        if let Some(t) = c.weapons[w].moves[air].tether.as_mut() {
            t.reel = 0;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("tether")));
        let mut c = base.clone();
        if let Some(t) = c.weapons[w].moves[air].tether.as_mut() {
            t.to = 200;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("tether")));
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::Jab as usize].hitboxes[0].blockable = true;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("blockable")));
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::USmash as usize].part_intangible[0].mode = 3;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("part_intangible")));
    }

    #[test]
    fn the_dagger_phase_3_fields_round_trip_and_are_checked() {
        // Sim 126: a stopped blockable grab's early end (owner decision 84), a stale dodge's stretch (85) and a tether's reel by a
        // press (86) come back from a file unchanged; a file without them keeps the older behaviour; bad values are refused.
        let base = Content::placeholder();
        let w = usize::from(base.fighters[12].weapon);
        let air = sim_core::moves::dagger_slots::HOOK_AIR as usize;
        let dash = sim_core::moves::dagger_slots::GUN_DASH as usize;
        let up = MoveId::UpSpecial as usize;
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "shield_end 20",
            "stale_stretch roll",
            "stale_stretch back_roll",
            "reel_press true",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // Without them: no early end, no stretch, no reel by a press.
        let stripped = text
            .replace(" reel_press true", "")
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("shield_end") && !t.starts_with("stale_stretch")
            })
            .collect::<Vec<_>>()
            .join(
                "
",
            );
        let older = crate::bundle::load(&stripped)
            .unwrap_or_else(|e| {
                panic!(
                    "{}",
                    e.join(
                        "
"
                    )
                )
            })
            .content;
        assert!(older.weapons[w].moves.iter().all(|m| m.shield_end == 0
            && m.stale_stretch == sim_core::moves::STRETCH_NONE
            && m.tether.is_none_or(|t| !t.reel_press)));
        // Bad values.
        let bad = text.replacen("stale_stretch roll", "stale_stretch crawl", 1);
        assert!(crate::bundle::load(&bad).is_err());
        let mut c = base.clone();
        c.weapons[w].moves[dash].dodge_stale = false;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("stale_stretch")));
        let mut c = base.clone();
        c.weapons[w].moves[dash].stale_stretch = 4;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("stale_stretch")));
        let mut c = base.clone();
        c.weapons[w].moves[air].shield_end = 10;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("shield_end")));
        let mut c = base.clone();
        c.weapons[w].moves[up].shield_end = 200;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("shield_end")));
    }

    #[test]
    fn the_pockets_fields_round_trip_and_are_checked() {
        // Sim 113: the Pockets' (class 10) previous-rules jab (E40), its random turnip versions (E39), its balloons (E41), the bowling
        // ball's charge power and the rocket's burst that a fighter does not set off come back from a file unchanged, and so do the
        // ruleset's previous-version numbers.
        let base = Content::placeholder();
        let w = usize::from(base.fighters[10].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "rules previous",
            "version {",
            "balloons {",
            "power_to 120",
            "on_hit false",
            "prev_hitlag_per_damage",
            "prev_tumble_hitstun",
            "cancel_from 6",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // A file without them keeps today's behaviour: no previous rules, no versions, no balloons, full power, a burst on a hit.
        let stripped = text
            .lines()
            .filter(|l| !l.contains("rules previous") && !l.contains("prev_"))
            .collect::<Vec<_>>()
            .join("\n");
        let older = crate::bundle::load(&stripped).expect("reads").content;
        assert!(older.weapons[w].moves[MoveId::Jab as usize]
            .hitboxes
            .iter()
            .all(|h| h.rules == sim_core::moves::RULES_CURRENT));
        assert_eq!(older.rules, base.rules, "the standard numbers when absent");
        // Bad values are refused: a third rules version, chances over 100, a version that is not a move of its own, balloons with no
        // helium, a charge power of 0.
        // (Sim 115: a jab cancel past the move's end.)
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::Jab as usize].cancel_from = 200;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("cancel_from")));
        let mut c = base.clone();
        c.weapons[w].moves[MoveId::Jab as usize].hitboxes[0].rules = 2;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("rules")));
        let uair = MoveId::UAir as usize;
        let mut c = base.clone();
        c.weapons[w].moves[uair].versions[0].0 = 80;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("chances")));
        let mut c = base.clone();
        c.weapons[w].moves[uair].versions[0].1 = MoveId::UAir as u8;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("version")));
        let mut c = base.clone();
        if let Some(b) = c.weapons[w].moves[MoveId::UpSpecial as usize]
            .balloons
            .as_mut()
        {
            b.fuel = 0;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("balloons")));
        let mut c = base.clone();
        if let Some(ch) = c.weapons[w].moves[MoveId::FSmash as usize]
            .projectile
            .as_mut()
            .and_then(|p| p.charged.as_mut())
        {
            ch.power_to = 0;
        }
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("power_to")));
        let mut c = base.clone();
        c.rules.prev_damage_mult = -Fx::ONE;
        assert!(validate(&c)
            .unwrap_err()
            .iter()
            .any(|e| e.contains("prev_damage_mult")));
    }

    #[test]
    fn the_tool_fields_round_trip_and_are_checked() {
        // Sim 99: the Buckler's (class 9) charged arrow and wave (E35), lodging and drag, the boomerang's tool rules (E3d), its
        // smash-input version and the wave's lying capsule (E38) come back from a file unchanged.
        let base = Content::placeholder();
        let w = usize::from(base.fighters[9].weapon);
        let text = crate::bundle::to_text(&base, "test", "", "", false);
        for field in [
            "charged {",
            "on_clank {",
            "catch_move",
            "smash_version",
            "half_width",
            "lodge 180",
            "drag",
        ] {
            assert!(text.contains(field), "{field}");
        }
        let back = crate::bundle::load(&text).expect("reads back").content;
        assert_eq!(back, base);
        // A charged shot with falloff, a negative drag, and a catch move without a catch age are refused.
        let side = MoveId::SideSpecial as usize;
        let nsp = sim_core::moves::MoveId::Ext4 as usize;
        let mut c = base.clone();
        let p = c.weapons[w].moves[nsp].projectile.as_mut().unwrap();
        p.end_damage = Fx::ZERO;
        let errors = validate(&c).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("charged")), "{errors:?}");
        let mut c = base.clone();
        c.weapons[w].moves[nsp]
            .projectile
            .as_mut()
            .unwrap()
            .arc
            .drag = -Fx::ONE;
        let errors = validate(&c).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("drag")), "{errors:?}");
        let mut c = base.clone();
        c.weapons[w].moves[side]
            .projectile
            .as_mut()
            .unwrap()
            .tool
            .catch_from = 255;
        let errors = validate(&c).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("catch_from")),
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
        c.fighters[0].weapon = 14;
        c.weapons[0].moves[0].hitboxes[0].end = 200;
        c.weapons[1].moves.pop();
        let errors = validate(&c).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("weapon 14")), "{errors:?}");
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
