//! Content validation. The data *formats* and file loaders arrive in Phase 5, once the hand-coded
//! fighters have shown what the format needs. For now this is the guardrail layer: refuse content
//! that would break the simulation (or, later, ranked balance).

use sim_core::{Content, Fx};

pub use sim_core::Content as LoadedContent;

/// Largest per-frame speed or acceleration content may declare, in world units.
const MAX_SPEED: Fx = Fx::from_int(4);

/// Returns every problem found, so creators see them all at once instead of one per attempt.
pub fn validate(content: &Content) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if content.fighters.is_empty() {
        errors.push("content defines no fighters".to_string());
    }
    for (i, f) in content.fighters.iter().enumerate() {
        let fields = [
            ("run_speed", f.run_speed),
            ("gravity", f.gravity),
            ("jump_velocity", f.jump_velocity),
            ("max_fall_speed", f.max_fall_speed),
        ];
        for (name, value) in fields {
            if value <= Fx::ZERO {
                errors.push(format!("fighter {i}: {name} must be positive"));
            } else if value > MAX_SPEED {
                errors.push(format!("fighter {i}: {name} exceeds the allowed maximum"));
            }
        }
    }

    let s = &content.stage;
    if s.blast_left >= s.blast_right {
        errors.push("stage: blast_left must be left of blast_right".to_string());
    }
    if s.blast_bottom >= s.blast_top {
        errors.push("stage: blast_bottom must be below blast_top".to_string());
    }
    if s.floor_y <= s.blast_bottom || s.floor_y >= s.blast_top {
        errors.push("stage: floor must lie inside the blast zones".to_string());
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

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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
        c.stage.blast_left = c.stage.blast_right;
        let errors = validate(&c).unwrap_err();
        assert!(errors.len() >= 3, "{errors:?}");
    }

    #[test]
    fn empty_content_is_rejected() {
        let mut c = Content::placeholder();
        c.fighters.clear();
        assert!(validate(&c).is_err());
    }
}
