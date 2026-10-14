//! Checks for the endless run's content (`sim_core::run::RunContent`): the guardrails a chunk or a mob kind must meet so the streamed
//! stage and the mob pool always work (pool sizes, chunk widths, seams, sane numbers). Like the stage library, the run's content is
//! Rust data for now (`RunContent::placeholder`); when the stage editor authors chunks (R2) they come through here.

use sim_core::run::data::{
    CHUNK_LEDGES, CHUNK_PLATFORMS, CHUNK_SPAWNS, MAX_CHUNK_WIDTH, MIN_CHUNK_WIDTH, MOB_FLYER,
    MOB_WALKER,
};
use sim_core::run::RunContent;
use sim_core::Fx;

/// Every problem with a run's content (empty when it is fine).
pub fn check_run(rc: &RunContent) -> Vec<String> {
    let mut errors = Vec::new();
    let r = &rc.rules;
    let positive = [
        ("view_width", r.view_width),
        ("cam_max_speed", r.cam_max_speed),
        ("push_speed", r.push_speed),
        ("crush_depth", r.crush_depth),
        ("blast_margin", r.blast_margin),
    ];
    for (name, v) in positive {
        if v <= Fx::ZERO {
            errors.push(format!("run rules: {name} must be above 0"));
        }
    }
    if r.lives == 0 || r.lives > sim_core::MatchRules::MAX_STOCKS {
        errors.push(format!(
            "run rules: lives must be 1 to {}",
            sim_core::MatchRules::MAX_STOCKS
        ));
    }
    if r.view_width > Fx::from_int(60) {
        errors
            .push("run rules: view_width at most 60 (the chunk ring is sized for it)".to_string());
    }
    for (name, v) in [("follow", r.follow), ("respawn_at", r.respawn_at)] {
        if v <= Fx::ZERO || v >= Fx::ONE {
            errors.push(format!("run rules: {name} must be between 0 and 1"));
        }
    }
    if r.creep < Fx::ZERO || r.creep_growth < Fx::ZERO || r.creep_max < r.creep {
        errors.push(
            "run rules: creep and its growth at least 0, creep_max at least creep".to_string(),
        );
    }
    if r.creep_max >= r.push_speed {
        errors.push("run rules: the camera's creep must stay below push_speed".to_string());
    }
    if r.blast_bottom >= Fx::ZERO || r.blast_top <= Fx::ZERO {
        errors.push(
            "run rules: blast_bottom below the ground line and blast_top above it".to_string(),
        );
    }
    if r.combo_step == 0 || r.combo_max == 0 {
        errors.push("run rules: combo_step and combo_max at least 1".to_string());
    }
    if rc.chunks.is_empty() {
        errors.push("run: needs at least one chunk".to_string());
    }
    if rc.chunks.len() > 32 {
        errors.push("run: at most 32 chunks".to_string());
    }
    if rc.mobs.is_empty() || rc.mobs.len() > 255 {
        errors.push("run: needs 1 to 255 mob kinds".to_string());
    }
    for (i, c) in rc.chunks.iter().enumerate() {
        let at = format!("chunk {i} ({})", c.name);
        if c.width < Fx::from_int(MIN_CHUNK_WIDTH) || c.width > Fx::from_int(MAX_CHUNK_WIDTH) {
            errors.push(format!(
                "{at}: width must be {MIN_CHUNK_WIDTH} to {MAX_CHUNK_WIDTH}"
            ));
        }
        if c.platforms.len() > CHUNK_PLATFORMS {
            errors.push(format!("{at}: at most {CHUNK_PLATFORMS} platforms"));
        }
        if c.ledges.len() > CHUNK_LEDGES {
            errors.push(format!("{at}: at most {CHUNK_LEDGES} ledges"));
        }
        if c.spawns.len() > CHUNK_SPAWNS {
            errors.push(format!("{at}: at most {CHUNK_SPAWNS} mob spawns"));
        }
        // The ground at both seams: solid, at the ground line, touching the edge, so chunks join in any order.
        let solid_at = |x: Fx| {
            c.platforms
                .iter()
                .any(|p| !p.pass_through && p.y == Fx::ZERO && (p.left == x || p.right == x))
        };
        if !solid_at(Fx::ZERO) || !solid_at(c.width) {
            errors.push(format!(
                "{at}: needs solid ground at height 0 touching both its edges"
            ));
        }
        for (j, p) in c.platforms.iter().enumerate() {
            if p.left >= p.right || p.left < Fx::ZERO || p.right > c.width {
                errors.push(format!(
                    "{at}: platform {j} must lie within the chunk, left of its right"
                ));
            }
            if !p.pass_through && p.bottom >= p.y {
                errors.push(format!("{at}: solid block {j} has bottom >= top"));
            }
            if p.y <= r.blast_bottom || p.y >= r.blast_top {
                errors.push(format!("{at}: platform {j} is outside the blast zone"));
            }
            if !p.pass_through && p.bottom > r.blast_bottom {
                errors.push(format!(
                    "{at}: solid block {j} must reach below the blast zone (nothing may go under it)"
                ));
            }
        }
        for (j, l) in c.ledges.iter().enumerate() {
            if l.side != -1 && l.side != 1 {
                errors.push(format!("{at}: ledge {j} side must be -1 or 1"));
            }
            if l.x < Fx::ZERO || l.x > c.width {
                errors.push(format!("{at}: ledge {j} must lie within the chunk"));
            }
        }
        for (j, s) in c.spawns.iter().enumerate() {
            if usize::from(s.kind) >= rc.mobs.len() {
                errors.push(format!(
                    "{at}: spawn {j} names mob kind {} (no such kind)",
                    s.kind
                ));
            }
            if s.x < Fx::ZERO || s.x > c.width || s.y <= r.blast_bottom || s.y >= r.blast_top {
                errors.push(format!(
                    "{at}: spawn {j} must lie within the chunk and the blast zone"
                ));
            }
        }
    }
    for (i, k) in rc.mobs.iter().enumerate() {
        let at = format!("mob kind {i}");
        if k.behavior != MOB_WALKER && k.behavior != MOB_FLYER {
            errors.push(format!("{at}: behavior must be walker (0) or flyer (1)"));
        }
        for (name, v) in [
            ("weight", k.weight),
            ("radius", k.radius),
            ("gravity", k.gravity),
            ("max_fall", k.max_fall),
            ("speed", k.speed),
            ("accel", k.accel),
            ("lunge", k.lunge),
        ] {
            if v <= Fx::ZERO || v > Fx::from_int(400) {
                errors.push(format!("{at}: {name} must be above 0 and at most 400"));
            }
        }
        if k.stamina < Fx::ZERO || k.jump < Fx::ZERO || k.hover_height < Fx::ZERO {
            errors.push(format!("{at}: stamina, jump and hover_height at least 0"));
        }
        if k.attack_frames == 0 {
            errors.push(format!("{at}: attack_frames at least 1"));
        }
        if k.hitbox.radius <= Fx::ZERO || k.hitbox.damage < Fx::ZERO {
            errors.push(format!("{at}: its hit needs a radius above 0"));
        }
        if k.hitbox.kind != sim_core::moves::HIT_NORMAL {
            errors.push(format!("{at}: its hit must be an ordinary hit"));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::content::Platform;

    #[test]
    fn the_built_in_run_is_valid() {
        assert_eq!(check_run(&RunContent::placeholder()), Vec::<String>::new());
    }

    #[test]
    fn bad_chunks_are_refused() {
        let mut rc = RunContent::placeholder();
        rc.chunks[1].width = Fx::from_int(10);
        rc.chunks[2].platforms.push(Platform {
            left: Fx::from_int(3),
            right: Fx::from_int(1),
            y: Fx::ONE,
            bottom: Fx::ONE,
            pass_through: true,
        });
        rc.chunks[3].platforms[0].y = Fx::ONE;
        rc.chunks[4].spawns[0].kind = 9;
        let errors = check_run(&rc);
        for want in ["chunk 1", "chunk 2", "chunk 3", "chunk 4"] {
            assert!(
                errors.iter().any(|e| e.starts_with(want)),
                "{want}: {errors:?}"
            );
        }
    }
}
