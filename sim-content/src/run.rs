//! Checks for the endless run's content (`sim_core::run::RunContent`): the guardrails a room, a mob kind or a shop ware must meet so
//! the streamed stage, the room window, the generator and the mob pool always work (pool sizes, room widths and types, seams, sane
//! numbers). Like the stage library, the run's content is Rust data for now (`RunContent::placeholder`); when the stage editor
//! authors rooms (R2) they come through here.

use sim_core::run::data::{
    ITEM_HEAL, ITEM_LIFE, MAX_ROOM_WIDTH, MAX_SHOP_ITEMS, MIN_ROOM_WIDTH, MOB_FLYER, MOB_WALKER,
    ROOM_ARENA, ROOM_KINDS, ROOM_LEDGES, ROOM_PLATFORMS, ROOM_REST, ROOM_SHOP, ROOM_SPAWNS,
    ROOM_STREET,
};
use sim_core::run::{RunContent, RING};
use sim_core::Fx;

/// Every problem with a run's content (empty when it is fine).
pub fn check_run(rc: &RunContent) -> Vec<String> {
    let mut errors = Vec::new();
    let r = &rc.rules;
    let positive = [
        ("view_width", r.view_width),
        ("cam_max_speed", r.cam_max_speed),
        ("push_speed", r.push_speed),
        ("blast_margin", r.blast_margin),
        ("cam_ease", r.cam_ease),
        ("gate_thickness", r.gate_thickness),
        ("door_reach", r.door_reach),
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
    if r.view_width > Fx::from_int(2 * MIN_ROOM_WIDTH) {
        errors.push(format!(
            "run rules: view_width at most {} (two of the narrowest rooms must cover it)",
            2 * MIN_ROOM_WIDTH
        ));
    }
    if r.cam_ease > Fx::ONE {
        errors.push("run rules: cam_ease at most 1".to_string());
    }
    if r.rooms_ahead == 0 || usize::from(r.rooms_behind) + 1 + usize::from(r.rooms_ahead) > RING {
        errors.push(format!(
            "run rules: rooms_ahead at least 1, and rooms_behind + 1 + rooms_ahead at most {RING}"
        ));
    }
    if r.enter_depth < Fx::ZERO || r.enter_depth > Fx::from_int(MIN_ROOM_WIDTH / 2) {
        errors.push(format!(
            "run rules: enter_depth 0 to {}",
            MIN_ROOM_WIDTH / 2
        ));
    }
    if r.shop_min == 0 || r.shop_max < r.shop_min {
        errors.push("run rules: shop_min at least 1 and shop_max at least shop_min".to_string());
    }
    if r.street_weight == 0 {
        errors
            .push("run rules: street_weight at least 1 (streets are the common room)".to_string());
    }
    if r.rest_heal < Fx::ZERO {
        errors.push("run rules: rest_heal at least 0".to_string());
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
    if rc.rooms.is_empty() || rc.rooms.len() > 255 {
        errors.push("run: needs 1 to 255 rooms".to_string());
    }
    if rc
        .rooms
        .first()
        .is_some_and(|r| r.kind != ROOM_STREET || !r.spawns.is_empty())
    {
        errors.push(
            "run: the first room (where every run starts) must be a street with no mobs"
                .to_string(),
        );
    }
    let has = |k: u8| rc.rooms.iter().skip(1).any(|r| r.kind == k);
    if !has(ROOM_STREET) {
        errors.push("run: needs a street room besides the first".to_string());
    }
    if !has(ROOM_SHOP) {
        errors.push("run: needs a shop room".to_string());
    }
    if r.arena_weight > 0 && !has(ROOM_ARENA) {
        errors.push("run: arena_weight is above 0 but there is no arena room".to_string());
    }
    if r.rest_weight > 0 && !has(ROOM_REST) {
        errors.push("run: rest_weight is above 0 but there is no rest room".to_string());
    }
    if rc.items.is_empty() || rc.items.len() > MAX_SHOP_ITEMS {
        errors.push(format!("run: the shop needs 1 to {MAX_SHOP_ITEMS} wares"));
    }
    for (i, item) in rc.items.iter().enumerate() {
        let at = format!("ware {i} ({})", item.name);
        if item.kind != ITEM_HEAL && item.kind != ITEM_LIFE {
            errors.push(format!("{at}: kind must be heal (0) or life (1)"));
        }
        if item.price == 0 {
            errors.push(format!("{at}: price at least 1"));
        }
        if item.amount <= Fx::ZERO || item.kind == ITEM_LIFE && item.amount > Fx::from_int(9) {
            errors.push(format!("{at}: amount above 0 (at most 9 lives)"));
        }
    }
    if rc.mobs.is_empty() || rc.mobs.len() > 255 {
        errors.push("run: needs 1 to 255 mob kinds".to_string());
    }
    for (i, c) in rc.rooms.iter().enumerate() {
        let at = format!("room {i} ({})", c.name);
        if c.kind >= ROOM_KINDS {
            errors.push(format!(
                "{at}: kind must be street, arena, shop or rest (0 to {})",
                ROOM_KINDS - 1
            ));
        }
        if c.width < Fx::from_int(MIN_ROOM_WIDTH) || c.width > Fx::from_int(MAX_ROOM_WIDTH) {
            errors.push(format!(
                "{at}: width must be {MIN_ROOM_WIDTH} to {MAX_ROOM_WIDTH}"
            ));
        }
        if c.platforms.len() > ROOM_PLATFORMS {
            errors.push(format!("{at}: at most {ROOM_PLATFORMS} platforms"));
        }
        if c.ledges.len() > ROOM_LEDGES {
            errors.push(format!("{at}: at most {ROOM_LEDGES} ledges"));
        }
        if c.spawns.len() > ROOM_SPAWNS {
            errors.push(format!("{at}: at most {ROOM_SPAWNS} mob spawns"));
        }
        if (c.kind == ROOM_SHOP || c.kind == ROOM_REST) && !c.spawns.is_empty() {
            errors.push(format!("{at}: a shop or rest room has no mobs"));
        }
        if c.kind == ROOM_ARENA && c.spawns.is_empty() {
            errors.push(format!("{at}: an arena needs mobs to beat"));
        }
        if c.kind == ROOM_SHOP {
            let on_ground = c.platforms.iter().any(|p| {
                !p.pass_through && p.y == c.door.y && c.door.x >= p.left && c.door.x <= p.right
            });
            if !on_ground {
                errors.push(format!(
                    "{at}: the shop's door must stand on solid ground in the room"
                ));
            }
        }
        // (A ledge at a seam would hang off a wall or a gate.)
        if c.ledges.iter().any(|l| l.x <= Fx::ZERO || l.x >= c.width) {
            errors.push(format!(
                "{at}: no ledge on the room's edges (the walls stand there)"
            ));
        }
        // The ground at both seams: solid, at the ground line, touching the edge, so rooms join in any order.
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
                    "{at}: platform {j} must lie within the room, left of its right"
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
                errors.push(format!("{at}: ledge {j} must lie within the room"));
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
                    "{at}: spawn {j} must lie within the room and the blast zone"
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
    fn bad_rooms_are_refused() {
        let mut rc = RunContent::placeholder();
        rc.rooms[1].width = Fx::from_int(10);
        rc.rooms[2].platforms.push(Platform {
            left: Fx::from_int(3),
            right: Fx::from_int(1),
            y: Fx::ONE,
            bottom: Fx::ONE,
            pass_through: true,
        });
        rc.rooms[3].platforms[0].y = Fx::ONE;
        rc.rooms[4].spawns[0].kind = 9;
        let shop = rc.rooms.iter().position(|r| r.kind == ROOM_SHOP).unwrap();
        let extra = rc.rooms[1].spawns[0];
        rc.rooms[shop].spawns.push(extra);
        let errors = check_run(&rc);
        for want in [
            "room 1",
            "room 2",
            "room 3",
            "room 4",
            &format!("room {shop}"),
        ] {
            assert!(
                errors.iter().any(|e| e.starts_with(want)),
                "{want}: {errors:?}"
            );
        }
    }
}

#[cfg(test)]
mod room_rules {
    use super::*;

    #[test]
    fn the_window_and_the_generator_are_checked() {
        let mut rc = RunContent::placeholder();
        rc.rules.rooms_behind = 3;
        rc.rules.shop_max = 2;
        let extra = rc.rooms[1].spawns[0];
        rc.rooms[0].spawns.push(extra);
        rc.items.clear();
        let errors = check_run(&rc);
        for want in ["rooms_behind", "shop_max", "first room", "wares"] {
            assert!(
                errors.iter().any(|e| e.contains(want)),
                "{want}: {errors:?}"
            );
        }
        let mut rc = RunContent::placeholder();
        rc.rooms.retain(|r| r.kind != ROOM_ARENA);
        assert!(check_run(&rc).iter().any(|e| e.contains("arena")));
    }
}
