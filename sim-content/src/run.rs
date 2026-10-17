//! Checks for the endless run's content (`sim_core::run::RunContent`): the guardrails a room, a piece, a mob kind, a shop ware or
//! the shop's interior must meet so the streamed stage, the room window, the generator and the mob pool always work (pool sizes, room
//! widths and types, seams, slopes, sane numbers), and **the passability proof** (R2a): every room, in every variant (mirrored or
//! not, every combination of its optional parts, every piece or none in every slot), can be crossed from its entry to its exit with
//! jumps that climb at most `RunRules::max_step` and clear at most `RunRules::max_gap`, and nothing it lets you reach is a trap you
//! cannot get out of. A test checks those two numbers against the worst jumper in the cast (`jump_reach`), made fighters included.
//! Like the stage library, the run's content is Rust data for now (`RunContent::placeholder`); when the stage editor authors rooms
//! they come through here.

use sim_core::content::{FighterParams, Platform};
use sim_core::run::data::{
    Room, BLOCK_BOTTOM, INTERIOR_PLATFORMS, ITEM_HEAL, ITEM_LIFE, LOOK_NAMES, MAX_ROOM_WIDTH,
    MAX_SHOP_ITEMS, MIN_ROOM_WIDTH, MOB_MOVES, NO_MOVE, PROP_ROLES, PROP_SIGN, ROLES, ROLE_LOBBER,
    ROOM_ALLEY, ROOM_ARENA, ROOM_COINS, ROOM_EVENT, ROOM_KINDS, ROOM_LEDGES, ROOM_PARTS,
    ROOM_PLATFORMS, ROOM_PROPS, ROOM_REST, ROOM_SHOP, ROOM_SLOTS, ROOM_SPAWNS, ROOM_STREET,
    SLOT_AIR, SPOT_ANY,
};
use sim_core::run::{
    deck_at, exit_deck, for_each_coin, for_each_footing, is_deck_fork, leads_to, part_on,
    room_exit, RoomSlot, RunContent, MAX_DECKS, RING,
};

/// How far below a room's lower ground end its lowest surface may be (R2d: a deck below, catching a fall, is about one view down).
const DEEPEST_CATCH: i32 = 20;
use sim_core::Fx;

/// Every problem with a run's content (empty when it is fine), the passability proof included.
pub fn check_run(rc: &RunContent) -> Vec<String> {
    let mut errors = check_run_shape(rc);
    if errors.is_empty() {
        errors.extend(check_passable(rc));
    }
    errors
}

/// The structural checks alone (fast): everything [`check_run`] checks but the passability proof.
pub fn check_run_shape(rc: &RunContent) -> Vec<String> {
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
        ("drift", r.drift),
        ("coin_reach", r.coin_reach),
        ("max_step", r.max_step),
        ("max_gap", r.max_gap),
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
    if r.fork_chance > 100 || r.piece_chance > 100 {
        errors.push("run rules: fork_chance and piece_chance are percents (0 to 100)".to_string());
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
    // (Solid ground reaches `run::DEEP` once laid: the lowest blast line the drift allows must stay above it.)
    if r.blast_bottom - r.drift - Fx::from_int(DEEPEST_CATCH)
        <= Fx::from_int(sim_core::run::DEEP + 10)
    {
        errors.push("run rules: drift and blast_bottom reach too deep".to_string());
    }
    if r.combo_step == 0 || r.combo_max == 0 {
        errors.push("run rules: combo_step and combo_max at least 1".to_string());
    }
    if rc.rooms.is_empty() || rc.rooms.len() > 255 {
        errors.push("run: needs 1 to 255 rooms".to_string());
    }
    if rc.rooms.first().is_some_and(|r| {
        r.kind != ROOM_STREET
            || !r.spawns.is_empty()
            || r.fork
            || r.exit_y != Fx::ZERO
            || exit_deck(r, false, r_start(rc)) != Some(r_start(rc))
    }) {
        errors.push(
            "run: the first room (where every run starts) must be a flat street with no mobs and no fork, on the start deck"
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
    deck_problems(rc, &mut errors);
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
    if rc.pieces.len() > 254 {
        errors.push("run: at most 254 pieces".to_string());
    }
    for (i, p) in rc.pieces.iter().enumerate() {
        let at = format!("piece {i} ({})", p.name);
        if p.fits == 0 || p.fits >= 1 << 3 {
            errors.push(format!("{at}: must fit street, roof or air slots"));
        }
        if p.width <= Fx::ZERO || p.width > Fx::from_int(12) {
            errors.push(format!("{at}: width above 0 and at most 12"));
        }
        for (j, f) in p.platforms.iter().enumerate() {
            footing_problems(
                &f.platform,
                f.look,
                r,
                (Fx::ZERO, Fx::ZERO),
                &format!("{at}: platform {j}"),
                &mut errors,
            );
            if f.part != 0 || f.skilled {
                errors.push(format!(
                    "{at}: platform {j} cannot belong to a part or a skilled route"
                ));
            }
            let half = p.width / Fx::from_int(2);
            if f.platform.left < -half || f.platform.right > half {
                errors.push(format!("{at}: platform {j} must lie within its width"));
            }
        }
    }
    for (i, c) in rc.rooms.iter().enumerate() {
        room_problems(rc, i, c, &mut errors);
    }
    interior_problems(rc, &mut errors);
    grunt_problems(rc, &mut errors);
    errors
}

/// The grunt kinds (R2b), the tiers and the props: every body and move is in the base roster (`Content::placeholder`, where a run's
/// world comes from), a lobber's first move fires a shot, one body throws for every lobber (its shots come from one fighter slot).
fn grunt_problems(rc: &RunContent, errors: &mut Vec<String>) {
    let roster = sim_core::Content::placeholder();
    if rc.mobs.is_empty() || rc.mobs.len() > 8 {
        errors.push("run: 1 to 8 grunt kinds (a tier's kinds are bits of a byte)".to_string());
    }
    for (i, k) in rc.mobs.iter().enumerate() {
        let at = format!("grunt kind {i}");
        if k.role >= ROLES {
            errors.push(format!("{at}: role must be 0 to {}", ROLES - 1));
        }
        let Some(body) = roster.fighters.get(usize::from(k.body)) else {
            errors.push(format!("{at}: body {} is not in the roster", k.body));
            continue;
        };
        let weapon = sim_core::combat::weapon_of(&roster, body);
        if k.moves[0] == NO_MOVE {
            errors.push(format!("{at}: needs a first move"));
        }
        for (j, &id) in k.moves.iter().enumerate().take(MOB_MOVES) {
            if id == NO_MOVE {
                continue;
            }
            let Some(mv) = weapon.moves.get(usize::from(id)) else {
                errors.push(format!(
                    "{at}: move {j} ({id}) is not in its body's moveset"
                ));
                continue;
            };
            let shoots = k.role == ROLE_LOBBER && j == 0;
            if shoots && mv.projectile.is_none() {
                errors.push(format!("{at}: a lobber's first move must fire a shot"));
            }
            if !shoots
                && !mv
                    .hitboxes
                    .iter()
                    .any(|h| h.kind == sim_core::moves::HIT_NORMAL)
            {
                errors.push(format!("{at}: move {j} ({id}) has no hit"));
            }
        }
        if k.role == ROLE_LOBBER && k.shot_time == 0 {
            errors.push(format!("{at}: a lobber needs its shot's flight time"));
        }
        for (name, v) in [("aggro", k.aggro), ("reach", k.reach), ("reach2", k.reach2)] {
            if v <= Fx::ZERO || v > Fx::from_int(40) {
                errors.push(format!("{at}: {name} must be above 0 and at most 40"));
            }
        }
        if k.stamina < Fx::ZERO || k.weight == 0 || k.weight > 400 {
            errors.push(format!("{at}: stamina at least 0, weight 1 to 400 percent"));
        }
        if k.walk == 0 || k.walk > 200 || k.run > 200 || k.cost == 0 {
            errors.push(format!(
                "{at}: walk 1 to 200 percent, run at most 200, cost at least 1"
            ));
        }
    }
    if rc.tiers.is_empty() || rc.tiers.len() > 255 {
        errors.push("run: needs 1 to 255 difficulty tiers".to_string());
    }
    let known = if rc.mobs.len() >= 8 {
        u8::MAX
    } else {
        (1u8 << rc.mobs.len()) - 1
    };
    for (i, t) in rc.tiers.iter().enumerate() {
        let at = format!("tier {i}");
        if t.kinds == 0 || t.kinds & !known != 0 {
            errors.push(format!(
                "{at}: kinds must name some of the grunt kinds and no others"
            ));
        }
        if t.tell == 0 || t.stamina == 0 || t.damage == 0 {
            errors.push(format!("{at}: tell, stamina and damage at least 1"));
        }
        if t.cap == 0 || usize::from(t.cap) > sim_core::run::MAX_MOBS {
            errors.push(format!(
                "{at}: cap (grunts alive at once) 1 to {}",
                sim_core::run::MAX_MOBS
            ));
        }
        if t.tech > 100 || t.di > 100 || t.mash > 100 {
            errors.push(format!(
                "{at}: tech, DI and mash are percents (at most 100)"
            ));
        }
        // (The knockback formula divides by the weight plus 100: every kind's weight must stay well above -100.)
        for j in 0..rc.mobs.len() {
            let w = sim_core::run::grunt_weight(rc, j as u8, i as u8);
            if w <= Fx::from_int(-90) || w > Fx::from_int(400) {
                errors.push(format!(
                    "{at}: grunt kind {j} would weigh {w:?} (above -90, at most 400)"
                ));
            }
        }
        let comes = (0..rc.mobs.len().min(8))
            .any(|k| t.kinds & (1 << k) != 0 && usize::from(rc.mobs[k].min_tier) <= i);
        if !comes {
            errors.push(format!("{at}: no kind it allows comes in it (min_tier)"));
        }
    }
    let r = &rc.rules;
    if r.tier_rooms == 0 || r.city_rooms == 0 {
        errors.push("run rules: tier_rooms and city_rooms at least 1".to_string());
    }
    if r.van_speed <= Fx::ZERO || r.van_accel <= Fx::ZERO || r.van_frames == 0 {
        errors.push("run rules: the van's speed, acceleration and frames above 0".to_string());
    }
    if r.view_height <= Fx::ZERO
        || r.ko_margin < Fx::ZERO
        || r.cam_dead < Fx::ZERO
        || r.cam_y_ease <= Fx::ZERO
    {
        errors.push(
            "run rules: view_height and cam_y_ease above 0, ko_margin and cam_dead at least 0"
                .to_string(),
        );
    }
    if r.high_ground <= Fx::ZERO || r.flat_room <= Fx::ZERO {
        errors.push("run rules: high_ground and flat_room above 0".to_string());
    }
    for (i, p) in rc.props.iter().enumerate() {
        let at = format!("prop {i}");
        if p.role >= PROP_ROLES {
            errors.push(format!("{at}: role must be 0 to {}", PROP_ROLES - 1));
        }
        if p.half_width <= Fx::ZERO || p.height <= Fx::ZERO || p.hit.radius <= Fx::ZERO {
            errors.push(format!("{at}: a box and a hit with sizes above 0"));
        }
        if p.role == PROP_SIGN && p.gravity <= Fx::ZERO {
            errors.push(format!("{at}: a sign needs gravity to fall"));
        }
    }
}

/// A footing's own problems: a look, a sane box, a gentle slope, inside the blast zone, solid ground reaching the bottom.
fn footing_problems(
    p: &Platform,
    look: u8,
    r: &sim_core::run::RunRules,
    (low, high): (Fx, Fx),
    at: &str,
    errors: &mut Vec<String>,
) {
    if look == 0 || usize::from(look) >= LOOK_NAMES.len() {
        errors.push(format!(
            "{at}: needs a look (what it is: awning, crate, roof...)"
        ));
    }
    if p.left >= p.right {
        errors.push(format!("{at}: left must be left of right"));
    }
    if let Some(e) = crate::slope_problem(p) {
        errors.push(format!("{at} {e}"));
    }
    // (R2d: the blast lines follow the rooms' lowest surfaces; a room reaches at most a deck below its lower end.)
    if p.y.min(p.top_right()) < low - Fx::from_int(DEEPEST_CATCH)
        || p.top_max() >= high + r.blast_top
    {
        errors.push(format!("{at}: is outside the blast zone"));
    }
    if !p.pass_through
        && p.bottom > Fx::from_int(BLOCK_BOTTOM)
        && p.bottom >= p.y.min(p.top_right())
    {
        errors.push(format!(
            "{at}: a solid block reaches the bottom, or floats (a ceiling, a wall over a doorway) with its bottom below its top"
        ));
    }
}

fn room_problems(rc: &RunContent, i: usize, c: &Room, errors: &mut Vec<String>) {
    let r = &rc.rules;
    let at = format!("room {i} ({})", c.name);
    if c.kind >= ROOM_KINDS {
        errors.push(format!(
            "{at}: kind must be street, arena, shop, rest, event or alley (0 to {})",
            ROOM_KINDS - 1
        ));
    }
    if c.min_tier > c.max_tier || usize::from(c.min_tier) >= rc.tiers.len().max(1) {
        errors.push(format!("{at}: its tiers must start at a tier there is"));
    }
    if c.kind == ROOM_EVENT && (c.event == 0 || c.mirror) {
        errors.push(format!(
            "{at}: an event room names its event and is never mirrored"
        ));
    }
    if c.kind != ROOM_EVENT && c.event != 0 {
        errors.push(format!("{at}: only an event room has an event"));
    }
    if c.props.len() > ROOM_PROPS
        || c.props.iter().any(|p| {
            usize::from(p.kind) >= rc.props.len() || p.part.unsigned_abs() as usize > c.parts.len()
        })
    {
        errors.push(format!(
            "{at}: at most {ROOM_PROPS} props, each a prop there is"
        ));
    }
    if c.kind == ROOM_ALLEY && !c.paired {
        errors.push(format!(
            "{at}: an alley only comes as a branching fork's high road (paired)"
        ));
    }
    if c.branch_high != 0 || c.branch_low != 0 {
        let high = rc.rooms.get(usize::from(c.branch_high));
        let low = rc.rooms.get(usize::from(c.branch_low));
        let ok = c.kind == ROOM_STREET
            && c.fork
            && !c.mirror
            && c.lift > Fx::ZERO
            && c.lift <= r.drift
            && high.is_some_and(|h| {
                h.kind == ROOM_ALLEY
                    && h.paired
                    && !h.mirror
                    && (h.exit_y == -c.lift || h.exit_y == Fx::ZERO)
            })
            && low.is_some_and(|l| {
                l.kind == ROOM_STREET && l.paired && !l.mirror && l.exit_y == Fx::ZERO
            })
            && high.zip(low).is_some_and(|(h, l)| h.width == l.width);
        if !ok {
            errors.push(format!(
                "{at}: a branching fork is a street fork lifting its high road (at most the drift) into a paired alley that comes back down by the lift or stays up, and a paired flat street as wide"
            ));
        }
        let high_exit = c.platforms.iter().any(|f| {
            f.part == 0
                && f.platform.right == c.width
                && f.platform.top_at(c.width) == c.exit_y + c.lift
        });
        if !high_exit {
            errors.push(format!(
                "{at}: its high road must leave its right edge at its lift"
            ));
        }
    }

    if c.fork && c.kind != ROOM_STREET && c.kind != ROOM_SHOP {
        errors.push(format!("{at}: only a street or a shop can be a fork"));
    }
    if c.width < Fx::from_int(MIN_ROOM_WIDTH) || c.width > Fx::from_int(MAX_ROOM_WIDTH) {
        errors.push(format!(
            "{at}: width must be {MIN_ROOM_WIDTH} to {MAX_ROOM_WIDTH}"
        ));
    }
    if c.exit_y.abs() > r.drift.mul_int(2) {
        errors.push(format!("{at}: its exit height is beyond the run's drift"));
    }
    room_deck_problems(rc, &at, c, errors);
    if c.platforms
        .iter()
        .any(|f| f.skilled && !f.platform.pass_through)
    {
        errors.push(format!(
            "{at}: a skilled route is pass-through platforms only (so the sure route is the room without them)"
        ));
    }
    if c.parts.len() > ROOM_PARTS || c.parts.iter().any(|&p| p > 100) {
        errors.push(format!(
            "{at}: at most {ROOM_PARTS} optional parts, each a percent chance"
        ));
    }
    if c.slots.len() > ROOM_SLOTS {
        errors.push(format!("{at}: at most {ROOM_SLOTS} piece slots"));
    }
    let bad_part = |p: i8| p.unsigned_abs() as usize > c.parts.len();
    if c.platforms.iter().any(|f| bad_part(f.part))
        || c.ledges.iter().any(|l| bad_part(l.part))
        || c.spawns.iter().any(|s| bad_part(s.part))
        || c.coins.iter().any(|s| bad_part(s.part))
        || c.slots.iter().any(|s| bad_part(s.part))
    {
        errors.push(format!(
            "{at}: something names an optional part it does not have"
        ));
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
    if c.kind == ROOM_ARENA && c.spawns.iter().all(|s| s.part != 0) {
        errors.push(format!("{at}: an arena needs mobs to beat"));
    }
    let solid_flat_at = |x: Fx, y: Fx| {
        c.platforms.iter().any(|f| {
            let p = &f.platform;
            f.part == 0 && !p.pass_through && x >= p.left && x <= p.right && p.top_at(x) == y
        })
    };
    if c.kind == ROOM_SHOP && !solid_flat_at(c.door.x, c.door.y) {
        errors.push(format!(
            "{at}: the shop's door must stand on solid ground in the room"
        ));
    }
    for (j, l) in c.ledges.iter().enumerate() {
        let l = &l.ledge;
        if l.side != -1 && l.side != 1 {
            errors.push(format!("{at}: ledge {j} side must be -1 or 1"));
        }
        // (A ledge at a seam would hang off a wall or a gate.)
        if l.x <= Fx::ZERO || l.x >= c.width {
            errors.push(format!(
                "{at}: no ledge on the room's edges (the walls stand there)"
            ));
        }
        let plats: Vec<Platform> = c.platforms.iter().map(|f| f.platform).collect();
        if crate::ledge_on_slope(l, &plats) {
            errors.push(format!(
                "{at}: ledge {j} hangs from a slope's corner (ledges only at flat corners)"
            ));
        }
    }
    let span = (c.exit_y.min(Fx::ZERO), c.exit_y.max(Fx::ZERO));
    for (j, f) in c.platforms.iter().enumerate() {
        footing_problems(
            &f.platform,
            f.look,
            r,
            span,
            &format!("{at}: platform {j}"),
            errors,
        );
        let p = &f.platform;
        if p.left < Fx::ZERO || p.right > c.width {
            errors.push(format!("{at}: platform {j} must lie within the room"));
        }
    }
    for (j, s) in c.spawns.iter().enumerate() {
        if s.kind != SPOT_ANY && usize::from(s.kind) >= rc.mobs.len() {
            errors.push(format!(
                "{at}: spawn {j} names mob kind {} (no such kind)",
                s.kind
            ));
        }
        if s.x < Fx::ZERO
            || s.x > c.width
            || s.y < span.0 - Fx::from_int(DEEPEST_CATCH)
            || s.y >= span.1 + r.blast_top
        {
            errors.push(format!(
                "{at}: spawn {j} must lie within the room and the blast zone"
            ));
        }
    }
    for (j, s) in c.slots.iter().enumerate() {
        if s.kind > SLOT_AIR {
            errors.push(format!("{at}: slot {j} must be a street, roof or air slot"));
        }
        if !rc.pieces.iter().any(|p| p.fits & (1 << s.kind.min(7)) != 0) {
            errors.push(format!("{at}: slot {j}: no piece fits it"));
        }
    }
    // Every variant's ground at the seams, ground under its slots, and its size in the pools (the passability proof is separate).
    let mut problems = Vec::new();
    for_each_variant(rc, c, i, |slot| {
        let mut plats = Vec::new();
        for_each_footing(rc, slot, |p| plats.push(p.platform));
        let mut coins = 0;
        for_each_coin(rc, slot, |_, _| coins += 1);
        let exit = room_exit(c, slot.mirror);
        // (An alley, the high road's room, is entered on its catwalk: it lies over its low road's room until one is taken, so it
        // has no solid block but the street under both.)
        let alley = c.kind == ROOM_ALLEY;
        // (R2d: the sky bridge, a deck fork's high road, also leaves on its catwalk.)
        let seam = |x: Fx, y: Fx| {
            plats.iter().any(|p| {
                (!p.pass_through || alley) && (p.left == x || p.right == x) && p.top_at(x) == y
            })
        };
        let street = plats.iter().find(|p| !p.pass_through).map(|p| p.y);
        if alley
            && plats.iter().any(|p| {
                !p.pass_through && (p.sloped() || Some(p.y) != street || p.y > exit.min(Fx::ZERO))
            })
        {
            problems.push(format!(
                "{at}: an alley has no solid block but the street under it (flat, at or below both its ends)"
            ));
        }
        // (R2d: a fall is never a death but in a true pit: something stands under every point of the room.)
        if !c.pit {
            let mut spans: Vec<(Fx, Fx)> = Vec::new();
            for_each_footing(rc, slot, |p| {
                if !p.skilled {
                    spans.push((p.platform.left, p.platform.right));
                }
            });
            spans.sort();
            let mut covered = Fx::ZERO;
            for (l, rr) in spans {
                if l > covered {
                    break;
                }
                covered = covered.max(rr);
            }
            if covered < c.width {
                problems.push(format!(
                    "{at}: a fall at {} lands on nothing (catch it on the deck below, or mark the room a pit) (mirrored {}, parts {:b})",
                    covered.floor_int(),
                    slot.mirror,
                    slot.parts
                ));
            }
        }
        if !seam(Fx::ZERO, Fx::ZERO) || !seam(c.width, exit) {
            problems.push(format!(
                "{at}: needs solid ground at both edges, at 0 where it begins and at its exit height where it ends (mirrored {}, parts {:b})",
                slot.mirror, slot.parts
            ));
        }
        if plats.len() > ROOM_PLATFORMS {
            problems.push(format!(
                "{at}: at most {ROOM_PLATFORMS} platforms with its pieces (parts {:b}, pieces {:?})",
                slot.parts, slot.pieces
            ));
        }
        if coins > ROOM_COINS {
            problems.push(format!("{at}: at most {ROOM_COINS} coins with its pieces"));
        }
        // A ground slot (when it is there) stands on flat solid ground wider than its widest piece.
        if slot.pieces == [0; ROOM_SLOTS] {
            for s in c
                .slots
                .iter()
                .filter(|s| s.kind != SLOT_AIR && part_on(slot.parts, s.part))
            {
                let widest = rc
                    .pieces
                    .iter()
                    .filter(|p| p.fits & (1 << s.kind.min(7)) != 0)
                    .map(|p| p.width)
                    .max()
                    .unwrap_or(Fx::ZERO);
                let half = widest / Fx::from_int(2) + Fx::HALF;
                let (lo, hi) = (s.x - half, s.x + half);
                let flat = c.platforms.iter().any(|f| {
                    let p = &f.platform;
                    part_on(slot.parts, f.part)
                        && !p.pass_through
                        && !p.sloped()
                        && p.y == s.y
                        && p.left <= lo
                        && p.right >= hi
                });
                if !flat {
                    problems.push(format!(
                        "{at}: a ground slot at {} needs flat solid ground under its widest piece (parts {:b})",
                        s.x.floor_int(),
                        slot.parts
                    ));
                }
            }
        }
    });
    problems.sort();
    problems.dedup();
    errors.extend(problems);
}

/// The start deck (clamped to the decks there are).
fn r_start(rc: &RunContent) -> u8 {
    rc.rules
        .start_deck
        .min(rc.decks.len().saturating_sub(1).min(255) as u8)
}

/// The decks (R2d): one to [`MAX_DECKS`], lowest first, their bands apart, inside the drift; the run starts on one; and every deck
/// the run can reach has a street that keeps to it (so the generator always has a room to lay there).
fn deck_problems(rc: &RunContent, errors: &mut Vec<String>) {
    let r = &rc.rules;
    if rc.decks.is_empty() || rc.decks.len() > MAX_DECKS {
        errors.push(format!("run: needs 1 to {MAX_DECKS} decks"));
        return;
    }
    if usize::from(r.start_deck) >= rc.decks.len() {
        errors.push("run rules: start_deck must be a deck there is".to_string());
    }
    if r.deck_rooms_min == 0 || r.deck_rooms_max < r.deck_rooms_min || r.deck_fork_chance > 100 {
        errors.push(
            "run rules: deck_rooms_min at least 1, deck_rooms_max at least that, deck_fork_chance a percent"
                .to_string(),
        );
    }
    if r.skill_step <= r.max_step || r.skill_gap <= r.max_gap {
        errors.push(
            "run rules: the skilled jumps (skill_step, skill_gap) must reach further than the sure ones".to_string(),
        );
    }
    if r.cam_y_ease_tall <= Fx::ZERO || r.cam_y_ease_tall > Fx::ONE {
        errors.push("run rules: cam_y_ease_tall above 0 and at most 1".to_string());
    }
    for (i, d) in rc.decks.iter().enumerate() {
        if d.wander < Fx::ZERO || d.y.abs() + d.wander > r.drift {
            errors.push(format!(
                "deck {i} ({}): its band must lie inside the run's drift",
                d.name
            ));
        }
    }
    for (i, w) in rc.decks.windows(2).enumerate() {
        if w[1].y - w[1].wander <= w[0].y + w[0].wander {
            errors.push(format!(
                "deck {} ({}): decks are listed lowest first, their bands apart",
                i + 1,
                w[1].name
            ));
        }
    }
    if rc.rooms.is_empty() {
        return;
    }
    // Every deck a room can lead to (or the run starts on) has a street that keeps to it.
    let mut reached = 1u16 << r_start(rc);
    for room in &rc.rooms {
        for d in 0..rc.decks.len() as u8 {
            for m in [false, true] {
                if let Some(o) = exit_deck(room, m, d) {
                    reached |= 1 << o;
                }
            }
        }
    }
    for (d, deck) in rc.decks.iter().enumerate() {
        if reached & (1 << d) == 0 {
            continue;
        }
        let keeps = rc.rooms.iter().enumerate().skip(1).any(|(i, room)| {
            room.kind == ROOM_STREET
                && !room.paired
                && leads_to(rc, room, d as u8, deck.y, d as u8)
                && !is_deck_fork(rc, &rc.rooms[i], d as u8)
        });
        if !keeps {
            errors.push(format!(
                "deck {d} ({}): needs a street room that keeps to it",
                deck.name
            ));
        }
    }
}

/// A room's decks (R2d): it enters on decks there are; a room that changes deck is a street, and from every deck it enters on it
/// leaves inside the band of the deck it names; a deck fork's high road leaves inside a deck's band too.
fn room_deck_problems(rc: &RunContent, at: &str, c: &Room, errors: &mut Vec<String>) {
    let n = rc.decks.len().min(MAX_DECKS);
    if !c.paired && c.decks == 0 {
        errors.push(format!("{at}: it enters on no deck"));
    }
    if n < 8 && c.decks >> n != 0 {
        errors.push(format!("{at}: it names a deck there is not"));
    }
    if c.deck_change != 0 && c.kind != ROOM_STREET {
        errors.push(format!("{at}: only a street climbs or drops a deck"));
    }
    for d in (0..n as u8).filter(|&d| c.decks & (1 << d) != 0) {
        let Some(out) = exit_deck(c, false, d) else {
            errors.push(format!(
                "{at}: entered on deck {d} it would leave on a deck there is not"
            ));
            continue;
        };
        if usize::from(out) >= n {
            errors.push(format!(
                "{at}: entered on deck {d} it would leave on a deck there is not"
            ));
            continue;
        }
        let (from, to) = (&rc.decks[usize::from(d)], &rc.decks[usize::from(out)]);
        let off = (from.y + c.exit_y - to.y).abs();
        let fits = if c.deck_change == 0 {
            c.exit_y.abs() <= from.wander.mul_int(2)
        } else {
            off + from.wander <= to.wander
        };
        if !fits {
            errors.push(format!(
                "{at}: entered on deck {d} its exit is outside deck {out}'s band"
            ));
        }
        if is_deck_fork(rc, c, d) {
            let high = from.y + c.exit_y + c.lift;
            let k = &rc.decks[usize::from(deck_at(rc, high))];
            if (high - k.y).abs() + from.wander > k.wander {
                errors.push(format!(
                    "{at}: entered on deck {d} its high road leaves outside a deck's band"
                ));
            }
        }
    }
}

fn interior_problems(rc: &RunContent, errors: &mut Vec<String>) {
    let s = &rc.interior;
    if s.width < Fx::from_int(8) || s.width > Fx::from_int(MAX_ROOM_WIDTH) {
        errors.push("shop interior: width 8 to 64".to_string());
    }
    if s.platforms.is_empty() || s.platforms.len() > INTERIOR_PLATFORMS {
        errors.push(format!(
            "shop interior: 1 to {INTERIOR_PLATFORMS} platforms"
        ));
    }
    for (j, f) in s.platforms.iter().enumerate() {
        if f.look == 0 || usize::from(f.look) >= LOOK_NAMES.len() {
            errors.push(format!("shop interior: platform {j} needs a look"));
        }
        if f.platform.left >= f.platform.right
            || (!f.platform.pass_through
                && f.platform.bottom >= f.platform.y.min(f.platform.top_right()))
        {
            errors.push(format!("shop interior: platform {j} is not a proper box"));
        }
    }
    let floor_at = |x: Fx| {
        s.platforms.iter().any(|f| {
            !f.platform.pass_through
                && x >= f.platform.left
                && x <= f.platform.right
                && f.platform.top_at(x) == Fx::ZERO
        })
    };
    for (name, x) in [("enter", s.enter), ("door", s.door)] {
        if x <= Fx::ZERO || x >= s.width || !floor_at(x) {
            errors.push(format!(
                "shop interior: {name} must stand on the floor (height 0) inside the room"
            ));
        }
    }
    if s.pedestals.len() < rc.items.len() {
        errors.push("shop interior: a pedestal for every ware".to_string());
    }
    for (j, p) in s.pedestals.iter().enumerate() {
        if p.x <= Fx::ZERO
            || p.x >= s.width
            || (p.x - s.door).abs() <= rc.rules.door_reach.mul_int(2)
        {
            errors.push(format!(
                "shop interior: pedestal {j} must be inside, away from the door"
            ));
        }
    }
    if s.keeper.x <= Fx::ZERO || s.keeper.x >= s.width || s.keeper_radius <= Fx::ZERO {
        errors.push("shop interior: the shopkeeper must stand inside, with a body".to_string());
    }
    if s.wave_frames == 0 || s.cheer_frames == 0 || s.grumble_frames == 0 || s.dizzy_frames == 0 {
        errors.push("shop interior: the shopkeeper's moods last at least a frame".to_string());
    }
}

/// Calls `f` with every variant of room `index` the generator can lay (at the origin): mirrored or not, every combination of its
/// parts, and every piece (or none) in each slot that is there.
pub fn for_each_variant(rc: &RunContent, room: &Room, index: usize, mut f: impl FnMut(&RoomSlot)) {
    let mirrors: &[bool] = if room.mirror {
        &[false, true]
    } else {
        &[false]
    };
    let parts = room.parts.len().min(ROOM_PARTS);
    for &mirror in mirrors {
        for bits in 0..(1u16 << parts) {
            let bits = bits as u8;
            let mut options: Vec<Vec<u8>> = Vec::new();
            for s in room.slots.iter().take(ROOM_SLOTS) {
                let mut o = vec![0u8];
                if part_on(bits, s.part) {
                    for (i, p) in rc.pieces.iter().enumerate() {
                        if p.fits & (1 << s.kind.min(7)) != 0 {
                            o.push((i + 1) as u8);
                        }
                    }
                }
                options.push(o);
            }
            let total: usize = options.iter().map(|o| o.len()).product();
            for mut k in 0..total {
                let mut pieces = [0u8; ROOM_SLOTS];
                for (n, o) in options.iter().enumerate() {
                    pieces[n] = o[k % o.len()];
                    k /= o.len();
                }
                let slot = RoomSlot {
                    live: true,
                    room: index as u8,
                    seq: 0,
                    x: Fx::ZERO,
                    end: room.width,
                    y: Fx::ZERO,
                    y_end: room_exit(room, mirror),
                    mirror,
                    parts: bits,
                    pieces,
                    spawned: 0,
                    coins: 0,
                    ..RoomSlot::default()
                };
                f(&slot);
            }
        }
    }
}

/// How high and how far a fighter's jumps reach: a full hop and one midair jump (its height), and the distance at full air speed
/// over the time those jumps keep it at or above the height it left from. Plain kinematics on the fighter's numbers (frames of 1/60 s).
pub fn jump_reach(p: &FighterParams) -> (Fx, Fx) {
    let rise = |v0: Fx, g: Fx| -> (Fx, i32) {
        // (Height and frames up to the top, the speed losing `g` a frame.)
        let (mut v, mut h, mut t) = (v0, Fx::ZERO, 0);
        while v > Fx::ZERO && t < 600 {
            h += v;
            v -= g;
            t += 1;
        }
        (h, t)
    };
    let (h1, t1) = rise(p.full_hop_velocity, p.gravity);
    let burst = p.hop_burst_velocity.mul_int(i32::from(p.hop_burst_frames));
    let (h2, t2) = if p.air_jumps > 0 {
        rise(p.air_jump_velocity, p.gravity)
    } else {
        (Fx::ZERO, 0)
    };
    let height = h1 + burst + h2;
    // (Up with the hop, up again with the air jump, then down that far: roughly twice the rising frames of each.)
    let frames = 2 * (t1 + t2);
    (height, p.air_speed.mul_int(frames))
}

/// One surface's top, for the proof, and whether it is on the skilled route.
#[derive(Clone, Copy)]
struct Top {
    p: Platform,
    skilled: bool,
}

/// The surfaces you can stand on, each cut where a solid block stands on it (a van on the street splits the street in two: walking
/// does not get past it, only a jump does).
fn split_tops(all: &[Platform], skilled: &[bool]) -> Vec<Top> {
    let mut out = Vec::new();
    for (i, p) in all.iter().enumerate() {
        let mut spans = vec![(p.left, p.right)];
        for (j, c) in all.iter().enumerate() {
            if i == j || c.pass_through || c.left >= p.right || c.right <= p.left {
                continue;
            }
            // (It stands on this surface, or reaches through it: its top is above this one's and its bottom at or below.)
            let (a, b) = (c.left.max(p.left), c.right.min(p.right));
            let here = p.top_at(a).max(p.top_at(b));
            if c.top_at(a).max(c.top_at(b)) <= here || c.bottom > here {
                continue;
            }
            spans = spans
                .into_iter()
                .flat_map(|(l, r)| {
                    let mut keep = Vec::new();
                    if c.left > l {
                        keep.push((l, c.left.min(r)));
                    }
                    if c.right < r {
                        keep.push((c.right.max(l), r));
                    }
                    keep
                })
                .filter(|(l, r)| l < r)
                .collect();
        }
        for (l, r) in spans {
            let mut q = *p;
            q.y = p.top_at(l);
            q.rise = p.top_at(r) - q.y;
            q.left = l;
            q.right = r;
            out.push(Top {
                p: q,
                skilled: skilled[i],
            });
        }
    }
    out
}

/// Whether solid block `c` reaches into the box `x_lo..x_hi` by `y_lo..y_hi` (open; a column when `x_lo == x_hi`).
fn intrudes(c: &Platform, x_lo: Fx, x_hi: Fx, y_lo: Fx, y_hi: Fx) -> bool {
    let across = if x_lo == x_hi {
        c.left < x_lo && x_lo < c.right
    } else {
        c.left < x_hi && c.right > x_lo
    };
    if !across || c.bottom >= y_hi {
        return false;
    }
    let a = x_lo.max(c.left);
    let b = x_hi.min(c.right);
    c.top_at(a).max(c.top_at(b)) > y_lo
}

/// Whether a jump from `(xa, ya)` to `(xb, yb)` is open: up from the takeoff, across above both, and down onto the landing, with a
/// fighter's height of room, through no solid block.
fn path_open(solids: &[Platform], xa: Fx, ya: Fx, xb: Fx, yb: Fx) -> bool {
    let head = Fx::from_ratio(9, 5);
    let h = ya.max(yb);
    let (lo, hi) = (xa.min(xb), xa.max(xb));
    !solids.iter().any(|c| {
        intrudes(c, xa, xa, ya, h + head)
            || (lo < hi && intrudes(c, lo, hi, h, h + head))
            || intrudes(c, xb, xb, yb, h + head)
    })
}

/// The edges of the proof's graph: from each surface, the surfaces a jump (or a walk, or a fall) reaches.
fn edges(tops: &[Top], solids: &[Platform], step: Fx, gap: Fx) -> Vec<Vec<usize>> {
    let clamp = |x: Fx, p: &Platform| x.clamp(p.left, p.right);
    let mut out = vec![Vec::new(); tops.len()];
    for (a, ta) in tops.iter().enumerate() {
        for (b, tb) in tops.iter().enumerate() {
            if a == b {
                continue;
            }
            let (pa, pb) = (&ta.p, &tb.p);
            let xs_a = [pa.left, pa.right, clamp(pb.left, pa), clamp(pb.right, pa)];
            let xs_b = [pb.left, pb.right, clamp(pa.left, pb), clamp(pa.right, pb)];
            let reached = xs_a.iter().any(|&xa| {
                xs_b.iter().any(|&xb| {
                    let (ya, yb) = (pa.top_at(xa), pb.top_at(xb));
                    let drop = (ya - yb).max(Fx::ZERO);
                    (xb - xa).abs() <= gap + drop / Fx::from_int(2)
                        && yb - ya <= step
                        && path_open(solids, xa, ya, xb, yb)
                })
            });
            if reached {
                out[a].push(b);
            }
        }
    }
    out
}

/// Every variant of every room, crossed with the rules' jumps (R2d: per route). **The sure route**: the room without its skilled
/// route's footings is crossed from its entry to its exit with the sure jumps (`max_step`, `max_gap`: the worst jumper in the cast),
/// and nothing a sure jumper reaches, skilled footings included, is a trap. **The skilled route**: with the skilled jumps
/// (`skill_step`, `skill_gap`) every skilled footing is reached from the entry and leads on to the exit, and in the room's authored
/// form (not mirrored, no parts, no pieces) it takes fewer jumps than the sure route. So a skilled route is never needed, and is
/// worth taking.
pub fn check_passable(rc: &RunContent) -> Vec<String> {
    let r = &rc.rules;
    let mut errors = Vec::new();
    for (i, room) in rc.rooms.iter().enumerate() {
        let mut bad: Option<String> = None;
        for_each_variant(rc, room, i, |slot| {
            if bad.is_some() {
                return;
            }
            let proof = passable(rc, room, slot, r.max_step, r.max_gap)
                .and_then(|()| skilled_route(rc, room, slot));
            if let Err(e) = proof {
                bad = Some(format!(
                    "room {i} ({}): {e} (mirrored {}, parts {:b}, pieces {:?})",
                    room.name, slot.mirror, slot.parts, slot.pieces
                ));
            }
        });
        errors.extend(bad);
        if room.platforms.iter().any(|f| f.skilled) {
            let authored = RoomSlot {
                live: true,
                room: i as u8,
                end: room.width,
                y_end: room.exit_y,
                ..RoomSlot::default()
            };
            let sure = Graph::of(rc, room, &authored, r.max_step, r.max_gap, false).hops();
            let skilled = Graph::of(rc, room, &authored, r.skill_step, r.skill_gap, true).hops();
            if !matches!((skilled, sure), (Some(k), Some(s)) if k < s) {
                errors.push(format!(
                    "room {i} ({}): its skilled route is no shorter than its sure route ({skilled:?} jumps against {sure:?})",
                    room.name
                ));
            }
        }
    }
    errors
}

/// The proof's graph for one variant (laid at the origin): the surfaces, the jumps between them, where it is entered and left.
struct Graph {
    tops: Vec<Top>,
    next: Vec<Vec<usize>>,
    starts: Vec<usize>,
    goals: Vec<usize>,
}

impl Graph {
    /// With the jumps `step` and `gap`, with the skilled route's footings or without them.
    fn of(
        rc: &RunContent,
        room: &Room,
        slot: &RoomSlot,
        step: Fx,
        gap: Fx,
        skilled: bool,
    ) -> Graph {
        let mut all = Vec::new();
        let mut flags = Vec::new();
        for_each_footing(rc, slot, |p| {
            if skilled || !p.skilled {
                all.push(p.platform);
                flags.push(p.skilled);
            }
        });
        let solids: Vec<Platform> = all.iter().copied().filter(|p| !p.pass_through).collect();
        let tops = split_tops(&all, &flags);
        let exit_y = room_exit(room, slot.mirror);
        // (An alley is entered, and a sky bridge left, on its catwalk: `room_problems`.)
        let alley = room.kind == ROOM_ALLEY;
        let touches = |p: &Platform, x: Fx, y: Fx| {
            (!p.pass_through || alley) && p.left <= x && x <= p.right && p.top_at(x) == y
        };
        let starts = (0..tops.len())
            .filter(|&k| touches(&tops[k].p, Fx::ZERO, Fx::ZERO))
            .collect();
        let goals = (0..tops.len())
            .filter(|&k| touches(&tops[k].p, room.width, exit_y))
            .collect();
        let next = edges(&tops, &solids, step, gap);
        Graph {
            tops,
            next,
            starts,
            goals,
        }
    }

    fn reach_from(&self, from: &[usize]) -> Vec<bool> {
        let mut seen = vec![false; self.tops.len()];
        let mut stack: Vec<usize> = from.to_vec();
        while let Some(k) = stack.pop() {
            if std::mem::replace(&mut seen[k], true) {
                continue;
            }
            stack.extend(self.next[k].iter().copied().filter(|&n| !seen[n]));
        }
        seen
    }

    /// The exit is reached from the entry, and from everything the entry reaches (no traps).
    fn crossed(&self) -> Result<(), String> {
        if self.starts.is_empty() || self.goals.is_empty() {
            return Err("no ground at its entry or exit".to_string());
        }
        let reached = self.reach_from(&self.starts);
        if !self.goals.iter().any(|&g| reached[g]) {
            return Err("its exit cannot be reached from its entry".to_string());
        }
        for k in (0..self.tops.len()).filter(|&k| reached[k]) {
            let on = self.reach_from(&[k]);
            if !self.goals.iter().any(|&g| on[g]) {
                let p = self.tops[k].p;
                return Err(format!(
                    "a trap: the surface from {} to {} at {} can be reached but not left toward the exit",
                    p.left.floor_int(),
                    p.right.floor_int(),
                    p.y.floor_int()
                ));
            }
        }
        Ok(())
    }

    /// The fewest jumps (walks and falls included) from the entry to the exit.
    fn hops(&self) -> Option<usize> {
        let mut dist = vec![usize::MAX; self.tops.len()];
        let mut queue = std::collections::VecDeque::new();
        for &s in &self.starts {
            dist[s] = 0;
            queue.push_back(s);
        }
        while let Some(k) = queue.pop_front() {
            if self.goals.contains(&k) {
                return Some(dist[k]);
            }
            for &n in &self.next[k] {
                if dist[n] == usize::MAX {
                    dist[n] = dist[k] + 1;
                    queue.push_back(n);
                }
            }
        }
        None
    }
}

/// The sure route's proof for one variant (laid at the origin): crossed with jumps of `step` and `gap` without the skilled route's
/// footings, and no traps with them.
pub fn passable(
    rc: &RunContent,
    room: &Room,
    slot: &RoomSlot,
    step: Fx,
    gap: Fx,
) -> Result<(), String> {
    Graph::of(rc, room, slot, step, gap, false)
        .crossed()
        .map_err(|e| format!("the sure route: {e}"))?;
    Graph::of(rc, room, slot, step, gap, true)
        .crossed()
        .map_err(|e| format!("with its skilled route, for a sure jumper: {e}"))
}

/// The skilled route's proof for one variant: with the skilled jumps, every skilled footing is reached from the entry, and no
/// surface reached is a trap.
pub fn skilled_route(rc: &RunContent, room: &Room, slot: &RoomSlot) -> Result<(), String> {
    let r = &rc.rules;
    let g = Graph::of(rc, room, slot, r.skill_step, r.skill_gap, true);
    if !g.tops.iter().any(|t| t.skilled) {
        return Ok(());
    }
    g.crossed().map_err(|e| format!("the skilled route: {e}"))?;
    let reached = g.reach_from(&g.starts);
    if let Some(k) = (0..g.tops.len()).find(|&k| g.tops[k].skilled && !reached[k]) {
        let p = g.tops[k].p;
        return Err(format!(
            "the skilled route: its footing from {} to {} at {} cannot be reached",
            p.left.floor_int(),
            p.right.floor_int(),
            p.y.floor_int()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::run::data::Footing;

    #[test]
    fn the_built_in_run_is_valid() {
        assert_eq!(check_run(&RunContent::placeholder()), Vec::<String>::new());
    }

    #[test]
    fn bad_rooms_are_refused() {
        let mut rc = RunContent::placeholder();
        rc.rooms[1].width = Fx::from_int(10);
        rc.rooms[2].platforms.push(Footing {
            platform: Platform {
                left: Fx::from_int(3),
                right: Fx::from_int(1),
                y: Fx::ONE,
                bottom: Fx::ONE,
                pass_through: true,
                rise: Fx::ZERO,
            },
            look: 3,
            part: 0,
            skilled: false,
        });
        rc.rooms[3].platforms[0].platform.y = Fx::ONE;
        rc.rooms[4].spawns[0].kind = 9;
        let shop = rc.rooms.iter().position(|r| r.kind == ROOM_SHOP).unwrap();
        let extra = rc.rooms[1].spawns[0];
        rc.rooms[shop].spawns.push(extra);
        let errors = check_run_shape(&rc);
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

    #[test]
    fn slopes_looks_parts_and_pieces_are_checked() {
        let mut rc = RunContent::placeholder();
        let ramp = rc.rooms.iter().position(|r| r.name == "ramp").unwrap();
        // Too steep.
        rc.rooms[ramp].platforms[1].platform.rise = Fx::from_int(8);
        rc.rooms[1].platforms[1].look = 0;
        rc.rooms[1].platforms[1].part = 5;
        rc.pieces[0].fits = 0;
        rc.interior.door = Fx::from_int(-3);
        let errors = check_run_shape(&rc);
        for want in [
            "slopes more than 1 in 2",
            "needs a look",
            "optional part it does not have",
            "piece 0",
            "door must stand on the floor",
        ] {
            assert!(
                errors.iter().any(|e| e.contains(want)),
                "{want}: {errors:?}"
            );
        }
        // A ledge on a slope's corner.
        let mut rc = RunContent::placeholder();
        let step = rc.rooms.iter().position(|r| r.name == "step").unwrap();
        rc.rooms[step].platforms[1].platform.rise = Fx::ONE;
        assert!(check_run_shape(&rc)
            .iter()
            .any(|e| e.contains("flat corners")));
    }

    #[test]
    fn an_impassable_room_is_refused() {
        // A wall too tall to climb, and a gap too wide to clear.
        let mut rc = RunContent::placeholder();
        let step = rc.rooms.iter().position(|r| r.name == "step").unwrap();
        rc.rooms[step].platforms[1].platform.y = Fx::from_int(9);
        rc.rooms[step].ledges.clear();
        assert!(check_shape_ok_then_pass(&rc)
            .iter()
            .any(|e| e.contains("cannot be reached")));
        let mut rc = RunContent::placeholder();
        let gap = rc.rooms.iter().position(|r| r.name == "gap").unwrap();
        rc.rooms[gap].platforms.retain(|f| !f.platform.pass_through);
        rc.rules.max_step = Fx::from_int(2);
        rc.rules.max_gap = Fx::from_int(4);
        assert!(check_passable(&rc)
            .iter()
            .any(|e| e.starts_with(&format!("room {gap}"))));
        // A pit you can drop into but not climb out of is a trap.
        let mut rc = RunContent::placeholder();
        rc.rules.max_gap = Fx::from_int(7);
        let gap = rc.rooms.iter().position(|r| r.name == "gap").unwrap();
        let plank = &mut rc.rooms[gap].platforms[2].platform;
        plank.y = Fx::from_int(-12);
        plank.bottom = Fx::from_int(-12);
        assert!(
            check_passable(&rc).iter().any(|e| e.contains("trap")),
            "{:?}",
            check_passable(&rc)
        );
    }

    /// The passability numbers hold for the worst jumper in the cast: every built-in fighter, and every class made with the lowest
    /// jump, the biggest body and the most weight (the lowest, heaviest jumps a made fighter can have), and the slowest drift.
    #[test]
    fn the_worst_jumper_in_the_cast_clears_the_rules() {
        use crate::format::Scalar;
        use crate::recipe::{Recipe, CLASSES, MAX_STAT, MIN_STAT};
        let rules = RunContent::placeholder().rules;
        let mut cast: Vec<(String, FighterParams)> = sim_core::Content::placeholder()
            .fighters
            .iter()
            .enumerate()
            .map(|(i, p)| (format!("built-in {i}"), *p))
            .collect();
        for class in 0..CLASSES {
            for speed in [MIN_STAT, MAX_STAT] {
                let r = Recipe {
                    class,
                    size: MAX_STAT,
                    speed,
                    jump: MIN_STAT,
                    weight: MAX_STAT,
                };
                cast.push((format!("made class {class} speed {speed}"), r.params()));
            }
        }
        let mut worst = (Fx::MAX, Fx::MAX);
        let mut short = Vec::new();
        for (name, p) in &cast {
            let (h, d) = jump_reach(p);
            worst = (worst.0.min(h), worst.1.min(d));
            if h < rules.max_step || d < rules.max_gap {
                short.push(format!("{name}: {} up, {} across", h.show(), d.show()));
            }
        }
        println!(
            "worst jumper: {} up, {} across",
            worst.0.show(),
            worst.1.show()
        );
        assert!(
            short.is_empty(),
            "short of the rules ({} up, {} across): {short:?}",
            rules.max_step.show(),
            rules.max_gap.show()
        );
    }

    fn check_shape_ok_then_pass(rc: &RunContent) -> Vec<String> {
        check_passable(rc)
    }

    #[test]
    fn every_variant_is_enumerated() {
        let rc = RunContent::placeholder();
        let avenue = rc.rooms.iter().position(|r| r.name == "avenue").unwrap();
        let mut n = 0;
        for_each_variant(&rc, &rc.rooms[avenue], avenue, |_| n += 1);
        // Mirrored or not, a part on or off, three slots: two street slots (none or one of three pieces) and an air slot (none or
        // one of two).
        assert_eq!(n, 2 * 2 * 4 * 4 * 3);
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

/// R2d: the decks and the routes.
#[cfg(test)]
mod decks_and_routes {
    use super::*;
    use crate::format::Scalar;

    fn room(rc: &RunContent, name: &str) -> usize {
        rc.rooms.iter().position(|r| r.name == name).unwrap()
    }

    /// The built-in city: three decks one view apart (the street, the rooftops, the skyline), the run starting on the rooftops, the
    /// drift covering them all (about 54 units top to bottom), and rooms that climb, drop and fork between them.
    #[test]
    fn the_city_has_three_decks_and_rooms_between_them() {
        let rc = RunContent::placeholder();
        let names: Vec<&str> = rc.decks.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["street", "rooftops", "skyline"]);
        assert_eq!(rc.decks[usize::from(rc.rules.start_deck)].y, Fx::ZERO);
        for w in rc.decks.windows(2) {
            assert_eq!(
                w[1].y - w[0].y,
                rc.rules.view_height,
                "a deck is one view tall"
            );
        }
        assert!(rc.rules.drift.mul_int(2) >= Fx::from_int(54));
        let count = |f: &dyn Fn(&Room) -> bool| rc.rooms.iter().filter(|r| f(r)).count();
        assert!(count(&|r| r.deck_change > 0) >= 4, "climbs");
        assert!(count(&|r| r.deck_change < 0) >= 3, "drops");
        assert!(count(&|r| r.indoor) >= 3, "interiors");
        let forks = (0..rc.rooms.len())
            .filter(|&i| (0..3).any(|d| is_deck_fork(&rc, &rc.rooms[i], d)))
            .count();
        assert!(forks >= 2, "deck forks");
        // Every deck has streets of its own, an arena, a shop and a rest stop.
        for d in 0..3u8 {
            let y = rc.decks[usize::from(d)].y;
            for kind in [ROOM_STREET, ROOM_ARENA, ROOM_SHOP, ROOM_REST] {
                assert!(
                    rc.rooms
                        .iter()
                        .skip(1)
                        .any(|r| r.kind == kind && !r.paired && leads_to(&rc, r, d, y, d)),
                    "deck {d}, room type {kind}"
                );
            }
        }
    }

    /// Adding a deck is a content change: a sewer below the street, with a street that keeps to it, is valid content as it is.
    #[test]
    fn a_deck_is_data() {
        let mut rc = RunContent::placeholder();
        rc.decks.insert(
            0,
            sim_core::run::Deck {
                name: "sewer".to_string(),
                y: Fx::from_int(-36),
                wander: Fx::from_int(6),
                weight: 2,
            },
        );
        rc.rules.start_deck += 1;
        rc.rules.drift = Fx::from_int(45);
        for r in rc.rooms.iter_mut() {
            r.decks <<= 1;
        }
        // (The sewer's own street: a copy of the plain street, entered only there.)
        let mut sewer = rc.rooms[room(&rc, "street")].clone();
        sewer.name = "sewer".to_string();
        sewer.decks = 0b1;
        rc.rooms.push(sewer);
        // (The rooms that climb from the street now climb from the sewer too.)
        let climb = room(&rc, "escape_climb");
        rc.rooms[climb].decks |= 0b1;
        assert_eq!(check_run_shape(&rc), Vec::<String>::new());
        // Without a street of its own, a deck is refused.
        let mut bad = rc.clone();
        bad.rooms.pop();
        assert!(check_run_shape(&bad)
            .iter()
            .any(|e| e.contains("needs a street room that keeps to it")));
    }

    #[test]
    fn bad_decks_are_refused() {
        let mut rc = RunContent::placeholder();
        rc.decks.swap(0, 2);
        rc.rules.start_deck = 9;
        let errors = check_run_shape(&rc);
        for want in ["lowest first", "start_deck"] {
            assert!(
                errors.iter().any(|e| e.contains(want)),
                "{want}: {errors:?}"
            );
        }
        let mut rc = RunContent::placeholder();
        rc.rules.drift = Fx::from_int(8);
        assert!(check_run_shape(&rc)
            .iter()
            .any(|e| e.contains("inside the run's drift")));
        // A climb that does not land on the next deck's band.
        let mut rc = RunContent::placeholder();
        let climb = room(&rc, "escape_climb");
        rc.rooms[climb].deck_change = 2;
        assert!(check_run_shape(&rc)
            .iter()
            .any(|e| e.starts_with(&format!("room {climb}"))));
    }

    /// A fall is never a death but in a room marked a pit: a gap with nothing under it is refused.
    #[test]
    fn a_fall_lands_on_something_unless_the_room_is_a_pit() {
        let mut rc = RunContent::placeholder();
        let gap = room(&rc, "gap");
        rc.rooms[gap].pit = false;
        assert!(check_run_shape(&rc)
            .iter()
            .any(|e| e.starts_with(&format!("room {gap}")) && e.contains("lands on nothing")));
        // The catwalks' gaps are caught by the roofs below: take one away (and the sills over it) and the room is refused.
        let mut rc = RunContent::placeholder();
        let cw = room(&rc, "catwalks");
        let mut k = 0;
        rc.rooms[cw].platforms.retain(|_| {
            k += 1;
            ![4, 8, 9, 10].contains(&k)
        });
        assert!(check_run_shape(&rc)
            .iter()
            .any(|e| e.contains("lands on nothing")));
    }

    /// The sure route never needs the skilled one: a room whose only way on is a skilled footing is refused, and so is a skilled
    /// route that is no shorter, or one that cannot be reached with the skilled jumps.
    #[test]
    fn the_skilled_route_is_never_needed_and_always_shorter() {
        let rc = RunContent::placeholder();
        let climb = room(&rc, "escape_climb");
        // Its fire escape gone, the climb needs its skilled route: refused.
        let mut bad = rc.clone();
        bad.rooms[climb]
            .platforms
            .retain(|f| f.skilled || f.look != sim_core::run::data::LOOK_FIRE_ESCAPE);
        assert!(check_passable(&bad)
            .iter()
            .any(|e| e.starts_with(&format!("room {climb}")) && e.contains("the sure route")));
        // Its skilled planks lowered to the fire escape's spacing: no shorter, refused.
        let mut bad = rc.clone();
        for f in bad.rooms[climb].platforms.iter_mut().filter(|f| f.skilled) {
            f.skilled = false;
        }
        bad.rooms[climb].platforms[2].skilled = true;
        assert!(check_passable(&bad)
            .iter()
            .any(|e| e.contains("no shorter")));
        // A skilled plank out of every reach: refused.
        let mut bad = rc.clone();
        let p = &mut bad.rooms[climb].platforms[6].platform;
        p.y = Fx::from_int(30);
        p.bottom = p.y;
        assert!(check_passable(&bad)
            .iter()
            .any(|e| e.contains("the skilled route") && e.contains("cannot be reached")));
        // A solid skilled footing is refused (the sure route is the room without them).
        let mut bad = rc.clone();
        bad.rooms[climb].platforms[0].skilled = true;
        assert!(check_run_shape(&bad)
            .iter()
            .any(|e| e.contains("pass-through platforms only")));
    }

    /// Every room with a skilled route: the sure route proven for every variant (`the_built_in_run_is_valid`), and here, for the
    /// record, the jumps each route takes in its authored form.
    #[test]
    fn the_skilled_routes_are_shorter() {
        let rc = RunContent::placeholder();
        let r = &rc.rules;
        let mut n = 0;
        for (i, c) in rc.rooms.iter().enumerate() {
            if !c.platforms.iter().any(|f| f.skilled) {
                continue;
            }
            let slot = RoomSlot {
                live: true,
                room: i as u8,
                end: c.width,
                y_end: c.exit_y,
                ..RoomSlot::default()
            };
            let sure = Graph::of(&rc, c, &slot, r.max_step, r.max_gap, false)
                .hops()
                .unwrap();
            let skilled = Graph::of(&rc, c, &slot, r.skill_step, r.skill_gap, true)
                .hops()
                .unwrap();
            println!("{}: sure {sure} jumps, skilled {skilled}", c.name);
            assert!(skilled < sure, "{}", c.name);
            n += 1;
        }
        assert!(n >= 6, "{n} rooms with a skilled route");
    }

    /// The skilled jumps ask for skill, not a fighter: above the sure ones, and within reach of most of the built-in cast (a full hop
    /// and a midair jump, without a recovery special).
    #[test]
    fn the_skilled_jumps_are_within_most_fighters_reach() {
        let rules = RunContent::placeholder().rules;
        let cast = sim_core::Content::placeholder().fighters;
        let mut reach = 0;
        for (i, p) in cast.iter().enumerate() {
            let (h, d) = jump_reach(p);
            println!("fighter {i}: {} up, {} across", h.show(), d.show());
            if h >= rules.skill_step && d >= rules.skill_gap {
                reach += 1;
            }
        }
        assert!(rules.skill_step > rules.max_step && rules.skill_gap > rules.max_gap);
        assert!(
            reach * 2 > cast.len(),
            "{reach} of {} fighters reach the skilled jumps",
            cast.len()
        );
    }
}
