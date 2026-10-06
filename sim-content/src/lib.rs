//! Content validation. The data *formats* and file loaders arrive in Phase 5, once the hand-coded
//! fighters have shown what the format needs. For now this is the guardrail layer: refuse content
//! that would break the simulation (or, later, ranked balance).

use sim_core::content::{MAX_LEDGES, MAX_PLATFORMS};
use sim_core::state::HISTORY_LEN;
use sim_core::{Content, FighterParams, Fx, Stage};

/// Largest value any distance, speed or acceleration may take, in world units.
const MAX_VALUE: Fx = Fx::from_int(64);

/// Fields that must be strictly positive (zero would freeze or break the movement).
const MUST_BE_POSITIVE: [&str; 12] = [
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
const MULTIPLIERS: [&str; 2] = ["air_dodge_decay", "waveland_friction"];

/// Returns every problem found, so creators see them all at once instead of one per attempt.
pub fn validate(content: &Content) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if content.fighters.is_empty() {
        errors.push("content defines no fighters".to_string());
    }
    for (i, f) in content.fighters.iter().enumerate() {
        validate_fighter(i, f, &mut errors);
    }
    validate_stage(&content.stage, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_fighter(i: usize, f: &FighterParams, errors: &mut Vec<String>) {
    for (name, value) in f.fx_fields() {
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
        ];
        if value == 0 && at_least_one.contains(&name) {
            errors.push(format!("fighter {i}: {name} must be at least 1"));
        }
    }
    if usize::from(f.air_dodge_buffer) > HISTORY_LEN
        || usize::from(f.shield_drop_buffer) > HISTORY_LEN
    {
        errors.push(format!(
            "fighter {i}: input buffers cannot exceed {HISTORY_LEN} frames"
        ));
    }
    if f.ledge_invuln_floor > f.ledge_invuln_base {
        errors.push(format!(
            "fighter {i}: ledge_invuln_floor exceeds ledge_invuln_base"
        ));
    }
    if f.ledge_hang_dx < f.ecb_half_width {
        errors.push(format!(
            "fighter {i}: ledge_hang_dx is smaller than ecb_half_width, so hanging would embed in the wall"
        ));
    }
    if f.ecb_side_height >= f.ecb_height {
        errors.push(format!(
            "fighter {i}: ecb_side_height must be below ecb_height"
        ));
    }
    if f.short_hop_velocity > f.full_hop_velocity {
        errors.push(format!("fighter {i}: short hop is higher than full hop"));
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
