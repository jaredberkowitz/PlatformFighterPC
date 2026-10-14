//! Items (E4, sim 111; owner decisions 54 and 64): things a weapon makes that any fighter can pick up, hold, throw and drop, that lie
//! on the stage, can be knocked about and can burst (the reference's held items: the bomb, a lodged arrow). Their numbers are the
//! weapon's `moves::ItemSpec`s; the universal rules (throw lengths, speeds, reach) are the ruleset's `item_*` values and each body's
//! release frames `FighterParams::item_throws`.
//!
//! A fighter's own update only asks (`Fighter::item_order`: pick up, release, use up, pull, detonate); everything that touches the
//! pool happens here, once every fighter has moved and the hits are settled (`step`), in player-index order, so it is deterministic.

use crate::collision;
use crate::combat::{self, Shape};
use crate::content::{Content, FighterParams, Ruleset};
use crate::fixed::Fx;
use crate::moves::{Hitbox, ItemSpec, HIT_NORMAL};
use crate::state::{
    Fighter, FighterState as S, GameState, Item, ITEM_BLAST, ITEM_FREE, ITEM_HELD, ITEM_LODGED,
    ITEM_POCKETED, ITEM_THROWN, MAX_ITEMS, NONE, REST_FLYING, REST_POCKETED,
};
use crate::trig::{self, Angle};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

/// `Fighter::item_order` bits.
pub const ORDER_PICK: u8 = 1;
pub const ORDER_RELEASE: u8 = 2;
pub const ORDER_SPEND: u8 = 4;
pub const ORDER_PULL: u8 = 8;
pub const ORDER_DETONATE: u8 = 16;
/// Take out what the pocket holds (E42: `Move::unpocket`).
pub const ORDER_UNPOCKET: u8 = 32;
/// Put the item in the hand away in the pocket, swapping with an item already there (E42: a pocket started holding an item).
pub const ORDER_STOW: u8 = 64;

/// `Fighter::item_throw` values (the low bits), with [`THROW_SMASH`] for a smash throw.
pub const THROW_FORWARD: u8 = 0;
pub const THROW_BACK: u8 = 1;
pub const THROW_UP: u8 = 2;
pub const THROW_DOWN: u8 = 3;
pub const THROW_DASH: u8 = 4;
/// The grounded drop (the grab button with the stick in the middle).
pub const THROW_DROP: u8 = 5;
/// The aerial drop (the grab button in the air, the stick in the middle): let go at once, no speed, no lag.
pub const THROW_LET_GO: u8 = 6;
/// A move's own throw of the item in hand (E47, sim 117: `moves::Toss`), at the move's speed and angle, during the move.
pub const THROW_MOVE: u8 = 7;
pub const THROW_SMASH: u8 = 0x80;

/// The `moves::ItemSpec` of an item: its maker's weapon's.
pub fn spec_of<'a>(content: &'a Content, it: &Item) -> Option<&'a ItemSpec> {
    let p = content.fighters.get(usize::from(it.maker))?;
    combat::weapon_of(content, p)
        .items
        .get(usize::from(it.spec))
}

/// Whether a fighter in this state keeps an item in its hand: anything but being hit, held, stuck, egged, on a ledge, out of the
/// match or coming back after a knock-out (the reference drops a held item then) [R: hit, grabbed; E: the rest].
pub fn can_hold(f: &Fighter) -> bool {
    f.active
        && !matches!(
            f.state,
            S::Hitstun
                | S::Grabbed
                | S::Buried
                | S::Encased
                | S::LedgeHang
                | S::LedgeGetUp
                | S::LedgeAttack
                | S::LedgeJump
                | S::ShieldBreak
                | S::Knockdown
                | S::GetUp
                | S::Respawn
                | S::WallTech
        )
}

/// Where a held item sits: the ruleset's hand, ahead of the feet the way the fighter faces.
pub fn hand(f: &Fighter, rules: &Ruleset) -> Vec2 {
    f.pos
        + Vec2::new(
            rules.item_hand_x.mul_int(i32::from(f.facing)),
            rules.item_hand_y,
        )
}

/// Whether an item can be picked up from where it is: lying on a platform or stuck where it lodged, not held, flying, frozen or bursting.
fn lying(it: &Item) -> bool {
    it.active
        && it.holder == NONE
        && it.fuse == 0
        && (it.rest == ITEM_LODGED || (it.rest == ITEM_FREE && it.platform != NONE))
}

/// Whether fighter `f`'s reach (the ruleset's `item_reach_*` box: ahead, behind and up from its feet) overlaps the pick-up box of
/// item `it` (half lengths around its middle).
fn in_reach(f: &Fighter, rules: &Ruleset, it: &Item, spec: &ItemSpec) -> bool {
    let facing = i32::from(f.facing);
    let (a, b) = (
        f.pos.x - rules.item_reach_back.mul_int(facing),
        f.pos.x + rules.item_reach_front.mul_int(facing),
    );
    let (left, right) = (a.min(b), a.max(b));
    let (bottom, top) = (f.pos.y, f.pos.y + rules.item_reach_up);
    let mid = it.pos.y + spec.half_height;
    it.pos.x + spec.pick_width >= left
        && it.pos.x - spec.pick_width <= right
        && mid + spec.pick_height >= bottom
        && mid - spec.pick_height <= top
}

/// The item fighter `i` would pick up now: the nearest lying item in reach (the lowest slot on a tie), if any.
fn pick_target(state: &GameState, content: &Content, i: usize) -> Option<usize> {
    let f = &state.fighters[i];
    let rules = &content.rules;
    let mut best: Option<(Fx, usize)> = None;
    for (n, it) in state.items.iter().enumerate() {
        if !lying(it) {
            continue;
        }
        let Some(spec) = spec_of(content, it) else {
            continue;
        };
        // (A placed piece, E43, is never picked up.)
        if spec.placed.is_some() || !in_reach(f, rules, it, spec) {
            continue;
        }
        let d = (it.pos.x - f.pos.x).abs();
        if best.is_none_or(|(b, _)| d < b) {
            best = Some((d, n));
        }
    }
    best.map(|(_, n)| n)
}

/// For each fighter, whether an item lies within its reach as the frame begins (what an attack or grab press on the ground reads:
/// it picks the item up instead). A fighter already holding one, or out of the match, has none.
pub fn pickable(state: &GameState, content: &Content) -> [bool; MAX_FIGHTERS] {
    core::array::from_fn(|i| {
        let f = &state.fighters[i];
        f.active && f.holding == NONE && f.grounded() && pick_target(state, content, i).is_some()
    })
}

/// The kind (`ItemSpec::kind`) of the item fighter `f` holds, 0 for none: what the `holding` register reads.
pub fn held_kind(items: &[Item], f: &Fighter) -> u8 {
    usize::try_from(f.holding)
        .ok()
        .and_then(|n| items.get(n))
        .filter(|it| it.active)
        .map_or(0, |it| it.kind)
}

/// How many live items fighter `me` made of kind `kind` (0: none counted): added to the `owned` register.
pub fn owned(items: &[Item], me: usize, kind: u8) -> u8 {
    if kind == 0 {
        return 0;
    }
    let n = items
        .iter()
        .filter(|it| it.active && usize::from(it.origin) == me && it.kind == kind)
        .count();
    u8::try_from(n).unwrap_or(u8::MAX)
}

/// A body's toss strength (the reference's, ssbwiki.com `Item_throw`): `1 + (weight - 100) / 300` [R].
pub fn toss_strength(p: &FighterParams) -> Fx {
    Fx::ONE + (p.weight - Fx::from_int(100)) / Fx::from_int(300)
}

/// The speed (world units a frame, before the item's multiplier) and angle (degrees above forward) of throw `throw` (`THROW_*`, with
/// `THROW_SMASH`), from the ruleset.
pub fn throw_speed(rules: &Ruleset, throw: u8) -> (Fx, i16) {
    let smash = throw & THROW_SMASH != 0;
    match throw & !THROW_SMASH {
        THROW_FORWARD | THROW_BACK if smash => (rules.item_side_smash, rules.item_side_smash_angle),
        THROW_FORWARD | THROW_BACK => (rules.item_side_speed, rules.item_side_angle),
        THROW_UP => (
            if smash {
                rules.item_up_smash
            } else {
                rules.item_up_speed
            },
            90,
        ),
        THROW_DOWN => (
            if smash {
                rules.item_down_smash
            } else {
                rules.item_down_speed
            },
            -90,
        ),
        THROW_DASH => (rules.item_dash_speed, rules.item_dash_angle),
        THROW_DROP => (rules.item_drop_speed, rules.item_drop_angle),
        _ => (Fx::ZERO, 0),
    }
}

/// The frame (1 for the first) a throw lets go on, for this body, and the throw's length in frames (the ruleset's).
pub fn throw_frames(p: &FighterParams, rules: &Ruleset, throw: u8) -> (u8, u8) {
    let t = &p.item_throws;
    match throw & !THROW_SMASH {
        THROW_FORWARD => (t.forward, rules.item_side_frames),
        THROW_BACK => (t.back, rules.item_side_frames),
        THROW_UP => (t.up, rules.item_up_frames),
        THROW_DOWN => (t.down, rules.item_down_frames),
        THROW_DASH => (t.dash, rules.item_dash_frames),
        THROW_DROP => (t.drop, rules.item_drop_frames),
        _ => (1, 1),
    }
}

/// `v` held to `low..=high` in size, keeping its sign (0 stays 0).
fn clamp_mag(v: Fx, (low, high): (Fx, Fx)) -> Fx {
    if v == Fx::ZERO {
        return v;
    }
    let m = v.abs().clamp(low, high.max(low));
    if v < Fx::ZERO {
        -m
    } else {
        m
    }
}

fn pct(v: Fx, p: u8) -> Fx {
    v * Fx::from_ratio(i32::from(p), 100)
}

/// Lets go of whatever fighter `i` holds: the item falls from its hand with no speed (a hit, a grab, a ledge, a knock-out).
pub fn let_go(state: &mut GameState, content: &Content, i: usize) {
    let f = state.fighters[i];
    state.fighters[i].holding = NONE;
    let Ok(n) = usize::try_from(f.holding) else {
        return;
    };
    let Some(it) = state.items.get_mut(n) else {
        return;
    };
    if !it.active || it.holder != i as i8 {
        return;
    }
    it.holder = NONE;
    it.pos = hand(&f, &content.rules);
    it.vel = Vec2::ZERO;
    it.platform = NONE;
    if it.rest == ITEM_HELD {
        it.rest = ITEM_FREE;
    }
}

/// Starts an item's burst where it is (or, with no blast, removes it); a held one leaves the hand first.
fn burst(state: &mut GameState, content: &Content, n: usize) {
    let it = state.items[n];
    if let Ok(h) = usize::try_from(it.holder) {
        if state.fighters[h].holding == n as i8 {
            state.fighters[h].holding = NONE;
        }
    }
    let blast = spec_of(content, &it).and_then(|s| s.blast);
    let it = &mut state.items[n];
    it.holder = NONE;
    it.fuse = 0;
    match blast {
        Some(_) => {
            it.rest = ITEM_BLAST;
            it.age = 0;
            it.vel = Vec2::ZERO;
            it.hit_mask = 0;
            it.hitlag = 0;
        }
        None => it.active = false,
    }
}

/// Removes item `n` (its holder's hand empties).
fn remove(state: &mut GameState, n: usize) {
    let it = state.items[n];
    if let Ok(h) = usize::try_from(it.holder) {
        if state.fighters[h].holding == n as i8 {
            state.fighters[h].holding = NONE;
        }
    }
    state.items[n].active = false;
}

/// Makes a new item of `fighter`'s weapon item `spec` at `pos` (none if the pool is full). Returns its slot.
pub fn make(
    state: &mut GameState,
    content: &Content,
    fighter: usize,
    spec_index: u8,
    pos: Vec2,
    move_id: u8,
) -> Option<usize> {
    let body = state.fighters[fighter].char_id;
    make_as(state, content, fighter, body, spec_index, pos, move_id)
}

/// [`make`] with the item's numbers read from body `body`'s weapon (an index into `Content::fighters`) while it is credited to
/// `fighter` (what a tool setting up a match with items lying about uses).
pub fn make_as(
    state: &mut GameState,
    content: &Content,
    fighter: usize,
    body: u8,
    spec_index: u8,
    pos: Vec2,
    move_id: u8,
) -> Option<usize> {
    let f = state.fighters[fighter];
    let p = content.fighters.get(usize::from(body))?;
    let spec = *combat::weapon_of(content, p)
        .items
        .get(usize::from(spec_index))?;
    let n = state.items.iter().position(|it| !it.active)?;
    state.items[n] = Item {
        active: true,
        origin: fighter as u8,
        maker: body,
        spec: spec_index,
        kind: spec.kind,
        owner: fighter as u8,
        holder: NONE,
        rest: ITEM_FREE,
        move_id,
        pos,
        vel: Vec2::ZERO,
        platform: NONE,
        age: 0,
        life: spec.life,
        flight: 0,
        hp: spec.hp,
        damage: Fx::ZERO,
        fuse: 0,
        hitlag: 0,
        hit_mask: 0,
        struck: [0; MAX_FIGHTERS],
        facing: f.facing,
        angle: 0,
        water: 0,
        grow_in: 0,
        cuts: 0,
        used: 0,
    };
    Some(n)
}

/// One frame of every item: the fighters' orders, open pockets catching items, then each item's clock and movement, its hits and the
/// attacks that knock it about.
pub fn update(state: &mut GameState, content: &Content) {
    for i in 0..MAX_FIGHTERS {
        serve(state, content, i);
    }
    for i in 0..MAX_FIGHTERS {
        pocket_items(state, content, i);
    }
    for n in 0..MAX_ITEMS {
        if state.items[n].active {
            tick(state, content, n);
        }
    }
    traps(state, content);
    knock_about(state, content);
}

/// Lying items with a trap (E45, sim 117: `moves::Trap`) trip the fighters on the ground that walk into them: for each such item (in
/// slot order) the first fighter (in player order) whose legs touch its trap.
fn traps(state: &mut GameState, content: &Content) {
    for n in 0..MAX_ITEMS {
        let it = state.items[n];
        if !it.active
            || it.holder != NONE
            || it.rest != ITEM_FREE
            || it.platform == NONE
            || it.hitlag > 0
        {
            continue;
        }
        let Some(spec) = spec_of(content, &it).copied() else {
            continue;
        };
        let Some(trap) = spec.trap else {
            continue;
        };
        let facing = i32::from(it.facing);
        let at = it.pos + Vec2::new(trap.hit.x.mul_int(facing), trap.hit.y);
        for d in 0..MAX_FIGHTERS {
            let fd = &state.fighters[d];
            let exempt = matches!(
                fd.state,
                S::Knockdown
                    | S::GetUp
                    | S::Hitstun
                    | S::Grabbed
                    | S::Buried
                    | S::Encased
                    | S::Respawn
                    | S::LedgeHang
                    | S::LedgeGetUp
                    | S::LedgeAttack
                    | S::LedgeJump
            );
            if !fd.active
                || d == usize::from(it.owner)
                || !fd.grounded()
                || exempt
                || combat::is_intangible(fd)
            {
                continue;
            }
            let pd = combat::params_of(content, fd);
            // (Only the lowest hurtbox circle, the legs: the reference's "leg and lower body hurtboxes".)
            let shut = combat::intangible_parts(fd, combat::weapon_of(content, pd));
            if shut & crate::moves::PART_LOW != 0 {
                continue;
            }
            let (legs, r) = combat::body_hurtboxes(content, fd)[0];
            let touching = combat::shapes_touch(
                at,
                trap.hit.half_width,
                trap.hit.half_height,
                trap.hit.radius,
                legs,
                Fx::ZERO,
                Fx::ZERO,
                r,
            );
            if !touching {
                continue;
            }
            let rules = &content.rules;
            let def = &mut state.fighters[d];
            def.percent = (def.percent + trap.hit.damage).min(Fx::from_int(999));
            crate::fighter::trip(def, rules, trap.hit.slip != crate::moves::SLIP_HARD);
            let away: i32 = match it.pos.x.cmp(&def.pos.x) {
                core::cmp::Ordering::Less => -1,
                core::cmp::Ordering::Greater => 1,
                core::cmp::Ordering::Equal => facing,
            };
            let item = &mut state.items[n];
            item.used = item.used.saturating_add(1);
            if spent(&spec, item) {
                item.active = false;
            } else {
                item.vel = Vec2::new(trap.kick_x.mul_int(away), trap.kick_y);
                item.platform = NONE;
            }
            break;
        }
    }
}

/// Fighter `i`'s hand and orders: an item it can no longer hold falls; then pick-up, release, use, pull and detonate, in that order.
fn serve(state: &mut GameState, content: &Content, i: usize) {
    let rules = &content.rules;
    let order = core::mem::take(&mut state.fighters[i].item_order);
    // The hand and the item agree, or the hand empties.
    if let Ok(n) = usize::try_from(state.fighters[i].holding) {
        let it = state.items.get(n);
        if !it.is_some_and(|it| it.active && it.holder == i as i8) {
            state.fighters[i].holding = NONE;
        }
    }
    if state.fighters[i].holding != NONE && !can_hold(&state.fighters[i]) {
        let_go(state, content, i);
    }
    let f = state.fighters[i];
    if order & ORDER_PICK != 0 && f.state == S::ItemPickup && f.holding == NONE {
        if let Some(n) = pick_target(state, content, i) {
            // (A stuck one's lodging time ends: in hand it lasts as its kind does.)
            let life = spec_of(content, &state.items[n]).map_or(0, |sp| sp.life);
            let it = &mut state.items[n];
            if it.rest == ITEM_LODGED {
                it.life = life;
                it.age = 0;
            }
            it.holder = i as i8;
            it.owner = i as u8;
            it.rest = ITEM_HELD;
            it.platform = NONE;
            it.vel = Vec2::ZERO;
            it.pos = hand(&f, rules);
            it.facing = f.facing;
            state.fighters[i].holding = n as i8;
        }
    }
    // (A throw that is still going, or the aerial let-go, which is no action of its own, or a move's own toss: E47.)
    let move_toss = f.item_throw == THROW_MOVE && f.state == S::Attack;
    if order & ORDER_RELEASE != 0
        && (f.state == S::ItemThrow || f.item_throw == THROW_LET_GO || move_toss)
    {
        release(state, content, i);
    }
    let f = state.fighters[i];
    if order & ORDER_SPEND != 0 {
        if let Ok(n) = usize::try_from(f.holding) {
            remove(state, n);
        }
    }
    let f = state.fighters[i];
    // A placed piece (E43) is set down in front instead of in the hand.
    let placed = combat::weapon_of(content, combat::params_of(content, &f))
        .items
        .get(usize::from(f.item_spec))
        .and_then(|s| s.placed);
    if let (true, Some(pl)) = (order & ORDER_PULL != 0 && f.state == S::Attack, placed) {
        place(state, content, i, f.item_spec, &pl);
    } else if order & ORDER_PULL != 0 && f.holding == NONE && f.state == S::Attack {
        if let Some(n) = make(state, content, i, f.item_spec, hand(&f, rules), f.move_id) {
            let it = &mut state.items[n];
            it.holder = i as i8;
            it.rest = ITEM_HELD;
            state.fighters[i].holding = n as i8;
        }
    }
    if order & ORDER_STOW != 0 {
        stow(state, content, i);
    }
    if order & ORDER_UNPOCKET != 0 {
        unpocket(state, content, i);
    }
    if order & ORDER_DETONATE != 0 {
        for n in 0..MAX_ITEMS {
            let it = state.items[n];
            // (Not one another fighter holds, nor one already bursting or about to, nor one put away in a pocket: frozen, E42.)
            let held_by_other = it.holder != NONE && it.holder != i as i8;
            if !it.active
                || usize::from(it.origin) != i
                || held_by_other
                || it.rest == ITEM_BLAST
                || it.rest == ITEM_POCKETED
                || it.fuse > 0
            {
                continue;
            }
            match spec_of(content, &it).and_then(|s| s.blast) {
                Some(b) if b.delay > 0 => state.items[n].fuse = b.delay,
                Some(_) => burst(state, content, n),
                None => {}
            }
        }
    }
}

/// Fighter `i` lets go of its item along its throw (`Fighter::item_throw`).
fn release(state: &mut GameState, content: &Content, i: usize) {
    let f = state.fighters[i];
    let Ok(n) = usize::try_from(f.holding) else {
        return;
    };
    let it = state.items[n];
    let Some(spec) = spec_of(content, &it).copied() else {
        return;
    };
    let rules = &content.rules;
    let throw = f.item_throw;
    let p = combat::params_of(content, &f);
    // A move's own toss (E47) at its speed and angle (its smash speed for a special input like a smash attack); the others by the
    // ruleset's universal throws.
    let (speed, angle) = if throw == THROW_MOVE {
        match combat::weapon_of(content, p).get(f.move_id).toss {
            Some(t) if f.special_flick => (t.smash_speed, t.angle),
            Some(t) => (t.speed, t.angle),
            None => return,
        }
    } else {
        throw_speed(rules, throw)
    };
    let t = speed * toss_strength(p);
    let m = spec.throw_mul;
    let a = Angle::from_degrees(i32::from(angle));
    let side = i32::from(f.facing);
    let mut vx = (t * trig::cos(a) * m * m).mul_int(side);
    if spec.max_speed > Fx::ZERO {
        vx = vx.clamp(-spec.max_speed, spec.max_speed);
    }
    let vy = t * trig::sin(a) * m;
    // The thrown hit's damage: its base (halved for the grounded drop, and for a move's toss: the reference's), the modifier a drop does
    // not halve (sim 117), plus its share of the throw's speed (the reference's).
    let base = if throw & !THROW_SMASH == THROW_DROP || throw == THROW_MOVE {
        spec.hit.damage / Fx::from_int(2)
    } else {
        spec.hit.damage
    } + spec.damage_add;
    state.fighters[i].holding = NONE;
    let item = &mut state.items[n];
    item.holder = NONE;
    item.owner = i as u8;
    item.rest = ITEM_THROWN;
    item.pos = hand(&f, rules);
    item.vel = Vec2::new(vx, vy);
    item.platform = NONE;
    item.flight = 0;
    item.damage = base + spec.damage_per_speed * t;
    item.hit_mask = 0;
    item.facing = f.facing;
    // A fighter's throw spends one of the item's uses (sim 117); a move's toss does not.
    if spec.uses > 0 && throw != THROW_MOVE {
        item.used = item.used.saturating_add(1);
    }
}

/// Whether an item with limited uses has spent them all (sim 117, `ItemSpec::uses`).
fn spent(spec: &ItemSpec, it: &Item) -> bool {
    spec.uses > 0 && it.used >= spec.uses
}

/// One frame of item `n`: hitlag, life and fuse, then its movement (held, lying, flying) and the hits it makes.
fn tick(state: &mut GameState, content: &Content, n: usize) {
    let Some(spec) = spec_of(content, &state.items[n]).copied() else {
        state.items[n].active = false;
        return;
    };
    if state.items[n].hitlag > 0 {
        state.items[n].hitlag -= 1;
        return;
    }
    if state.items[n].rest == ITEM_BLAST {
        blast_frame(state, content, n, &spec);
        return;
    }
    // Put away in a pocket (E42): frozen, its clock stopped.
    if state.items[n].rest == ITEM_POCKETED {
        return;
    }
    {
        let it = &mut state.items[n];
        it.age = it.age.saturating_add(1);
    }
    let it = state.items[n];
    if it.life > 0 && it.age >= it.life {
        burst(state, content, n);
        if state.items[n].active {
            blast_frame(state, content, n, &spec);
        }
        return;
    }
    if it.fuse > 0 {
        state.items[n].fuse -= 1;
        if state.items[n].fuse == 0 {
            burst(state, content, n);
            blast_frame(state, content, n, &spec);
            return;
        }
    }
    // A placed piece that has its water grows after its delay (E43).
    if let Some(pl) = spec.placed.filter(|pl| pl.water > 0) {
        let item = &mut state.items[n];
        if item.grow_in == 0 && item.water >= pl.water {
            item.grow_in = pl.grow_delay.saturating_add(1);
        }
        if item.grow_in > 0 {
            item.grow_in -= 1;
            if item.grow_in == 0 {
                grow(state, content, n, &pl);
                return;
            }
        }
    }
    match it.rest {
        ITEM_HELD => {
            if let Ok(h) = usize::try_from(it.holder) {
                let f = state.fighters[h];
                state.items[n].pos = hand(&f, &content.rules);
                state.items[n].facing = f.facing;
            }
        }
        ITEM_LODGED => {}
        _ => {
            fly(state, content, n, &spec);
            if state.items[n].active && state.items[n].rest == ITEM_THROWN {
                thrown_hits(state, content, n, &spec);
            }
        }
    }
}

/// The movement of a free or thrown item: gravity, friction, the floor, walls and ceilings with the item's bounces, the blast zone.
fn fly(state: &mut GameState, content: &Content, n: usize, spec: &ItemSpec) {
    let stage = &content.stage;
    let mut it = state.items[n];
    if it.rest == ITEM_THROWN {
        it.flight = it.flight.saturating_add(1);
        if spec.thrown_life > 0 && it.flight >= spec.thrown_life {
            // (Its uses spent by the throw: the end of its flight is its end, sim 117.)
            if spec.vanish || spent(spec, &it) {
                state.items[n].active = false;
                return;
            }
            it.rest = ITEM_FREE;
        }
    }
    let limit = |v: Fx| {
        if spec.max_speed > Fx::ZERO {
            v.clamp(-spec.max_speed, spec.max_speed)
        } else {
            v
        }
    };
    if let Some(plat) = collision::platform(stage, it.platform) {
        // Lying (or sliding) on a platform: its friction, and over the end it falls.
        it.vel.y = Fx::ZERO;
        it.vel.x = approach_zero(it.vel.x, spec.friction);
        let mut to = Vec2::new(it.pos.x + it.vel.x, it.pos.y);
        // (On a slope it slides along the surface, sim 124.)
        if plat.sloped() {
            to.y = plat.top_at(to.x);
        }
        if inside_block(stage, to) {
            it.vel.x = Fx::ZERO;
        } else {
            it.pos = to;
        }
        if it.pos.x < plat.left || it.pos.x > plat.right {
            it.platform = NONE;
        }
        state.items[n] = it;
        return;
    }
    it.vel.y = (it.vel.y - spec.gravity).max(-spec.max_fall);
    it.vel.x = limit(approach_zero(it.vel.x, spec.air_friction));
    let from = it.pos;
    let to = from + it.vel;
    if it.vel.y < Fx::ZERO {
        if let Some(i) = collision::find_landing(stage, from, to, false) {
            if it.rest == ITEM_THROWN && spec.vanish {
                state.items[n].active = false;
                return;
            }
            let into = -it.vel.y;
            it.pos = Vec2::new(to.x, stage.platforms[i].top_at(to.x));
            if into > spec.bounce_min {
                it.vel.y = pct(into, spec.floor_keep.1).clamp(spec.bounce_y.0, spec.bounce_y.1);
                it.vel.x = clamp_mag(pct(it.vel.x, spec.floor_keep.0), spec.bounce_x);
            } else {
                it.vel.y = Fx::ZERO;
                it.platform = i as i8;
            }
            // (The thrown hit ends where it meets the ground [E]; with its uses spent by the throw it is gone there, sim 117.)
            if it.rest == ITEM_THROWN {
                if spent(spec, &it) {
                    state.items[n].active = false;
                    return;
                }
                it.rest = ITEM_FREE;
            }
            state.items[n] = it;
            return;
        }
    }
    if let Some(block) = stage.platforms.iter().find(|b| {
        !b.pass_through
            && to.x > b.left
            && to.x < b.right
            && to.y > b.bottom
            && to.y < b.top_at(to.x)
    }) {
        if it.rest == ITEM_THROWN && spec.vanish {
            state.items[n].active = false;
            return;
        }
        let side = from.x <= block.left || from.x >= block.right;
        if side {
            if it.vel.x.abs() > spec.bounce_min {
                it.vel.x = clamp_mag(-pct(it.vel.x, spec.wall_keep.0), spec.bounce_x);
                it.vel.y = pct(it.vel.y, spec.wall_keep.1) + spec.wall_lift;
            } else {
                it.vel.x = Fx::ZERO;
            }
        } else if it.vel.y > spec.bounce_min {
            it.vel.y = -clamp_mag(pct(it.vel.y, spec.ceiling_keep.1), spec.bounce_y);
            it.vel.x = clamp_mag(pct(it.vel.x, spec.ceiling_keep.0), spec.bounce_x);
        } else {
            it.vel.y = it.vel.y.min(Fx::ZERO);
        }
        state.items[n] = it;
        return;
    }
    it.pos = to;
    let outside = to.x < stage.blast_left
        || to.x > stage.blast_right
        || to.y < stage.blast_bottom
        || to.y > stage.blast_top;
    if outside {
        it.active = false;
    }
    state.items[n] = it;
}

fn approach_zero(v: Fx, by: Fx) -> Fx {
    if v > Fx::ZERO {
        (v - by).max(Fx::ZERO)
    } else {
        (v + by).min(Fx::ZERO)
    }
}

fn inside_block(stage: &crate::content::Stage, p: Vec2) -> bool {
    stage.platforms.iter().any(|b| {
        !b.pass_through && p.x > b.left && p.x < b.right && p.y > b.bottom && p.y < b.top_at(p.x)
    })
}

/// The item's middle (its hits and hurtbox are centred there).
fn middle(it: &Item, spec: &ItemSpec) -> Vec2 {
    it.pos + Vec2::new(Fx::ZERO, spec.half_height)
}

/// A thrown item's hit: the first fighter it touches (not its thrower, not one it hit) takes it, or its shield does, or a passive
/// shield stops it; then it vanishes (`ItemSpec::vanish`) or bounces off (`fighter_keep`, `fighter_lift`) and its hit ends.
fn thrown_hits(state: &mut GameState, content: &Content, n: usize, spec: &ItemSpec) {
    let it = state.items[n];
    let owner = usize::from(it.owner).min(MAX_FIGHTERS - 1);
    let at = middle(&it, spec);
    let mut hb: Hitbox = spec.hit;
    hb.damage = it.damage;
    let shape = Shape {
        half_w: hb.half_width,
        half_h: hb.half_height,
        radius: hb.radius,
    };
    for d in 0..MAX_FIGHTERS {
        if d == owner || it.hit_mask & (1 << d) != 0 {
            continue;
        }
        let fd = &state.fighters[d];
        let counter = combat::counter_open(content, fd, false);
        let intangible = combat::is_intangible(fd);
        if intangible && counter.is_none() {
            continue;
        }
        let pd = combat::params_of(content, fd);
        let blocked = !intangible && combat::passive_shield_blocks(fd, pd, at, shape, it.vel);
        let (touching, caught) = if blocked {
            (true, false)
        } else {
            let hurt = combat::body_hurtboxes(content, fd);
            let shut = combat::intangible_parts(fd, combat::weapon_of(content, pd));
            combat::reach(counter, intangible, &hb, &hurt, shut, at)
        };
        if !touching {
            continue;
        }
        // (A hit on a fighter, not a shield or a passive shield, spends one of a limited item's uses: sim 117.)
        let on_body = !blocked && state.fighters[d].state != S::Shield;
        if blocked {
            combat::passive_block(&mut state.fighters[d], &content.rules, &hb);
        } else {
            let travel = match it.vel.x.raw().signum() {
                0 => it.facing,
                s => s as i8,
            };
            let facing = combat::launch_side(hb.angle_type, travel, travel, travel, at.x, fd.pos.x);
            combat::apply_hit(
                state,
                content,
                owner,
                d,
                &hb,
                facing,
                at,
                false,
                combat::STUN_PROJECTILE,
                ITEM_THROW_KEY,
                (at, it.vel),
                caught,
            );
        }
        let item = &mut state.items[n];
        item.hit_mask |= 1 << d;
        if on_body && spec.uses > 0 {
            item.used = item.used.saturating_add(1);
        }
        if spec.vanish || spent(spec, item) {
            item.active = false;
        } else {
            item.vel = Vec2::new(
                -pct(item.vel.x, spec.fighter_keep.0),
                pct(item.vel.y, spec.fighter_keep.1) + spec.fighter_lift,
            );
            item.rest = ITEM_FREE;
        }
        return;
    }
}

/// The stale-move key thrown items hit with (no move slot: the reference's item throws stale apart from every move).
pub const ITEM_THROW_KEY: u8 = 60;

/// One frame of a burst: its hit on every fighter it touches and has not hit (its maker too with `hits_owner`), then its end.
fn blast_frame(state: &mut GameState, content: &Content, n: usize, spec: &ItemSpec) {
    let Some(b) = spec.blast else {
        state.items[n].active = false;
        return;
    };
    let it = state.items[n];
    let maker = usize::from(it.origin).min(MAX_FIGHTERS - 1);
    let at = middle(&it, spec);
    let hb = b.hit;
    for d in 0..MAX_FIGHTERS {
        if (d == maker && !b.hits_owner) || it.hit_mask & (1 << d) != 0 {
            continue;
        }
        let fd = &state.fighters[d];
        if !fd.active {
            continue;
        }
        let counter = combat::counter_open(content, fd, false);
        let intangible = combat::is_intangible(fd);
        if intangible && counter.is_none() {
            continue;
        }
        let pd = combat::params_of(content, fd);
        let hurt = combat::body_hurtboxes(content, fd);
        let shut = combat::intangible_parts(fd, combat::weapon_of(content, pd));
        let (touching, caught) = combat::reach(counter, intangible, &hb, &hurt, shut, at);
        if !touching {
            continue;
        }
        let facing = combat::launch_side(
            hb.angle_type,
            it.facing,
            it.facing,
            it.facing,
            at.x,
            fd.pos.x,
        );
        combat::apply_hit(
            state,
            content,
            maker,
            d,
            &hb,
            facing,
            at,
            false,
            combat::STUN_PROJECTILE,
            it.move_id,
            (at, Vec2::ZERO),
            caught,
        );
        state.items[n].hit_mask |= 1 << d;
    }
    let item = &mut state.items[n];
    item.age = item.age.saturating_add(1);
    if item.age >= u16::from(b.frames) {
        item.active = false;
    }
}

/// Attacks that touch an item with hit points knock it about: it takes the hit's damage off its `hp`, is launched along the hit's angle
/// and frozen for a share of the hitlag; one knock per attacker's move. At 0 hp it bursts (or goes).
fn knock_about(state: &mut GameState, content: &Content) {
    // A fighter no longer attacking can knock items again.
    for a in 0..MAX_FIGHTERS {
        if !matches!(state.fighters[a].state, S::Attack | S::LedgeAttack) {
            for it in &mut state.items {
                it.struck[a] = 0;
            }
        }
    }
    for a in 0..MAX_FIGHTERS {
        let fa = state.fighters[a];
        if !fa.active || !matches!(fa.state, S::Attack | S::LedgeAttack) || fa.hitlag > 0 {
            continue;
        }
        let pa = combat::params_of(content, &fa);
        let mv = combat::weapon_of(content, pa).get(fa.move_id);
        for n in 0..MAX_ITEMS {
            let it = state.items[n];
            if !it.active
                || it.holder != NONE
                || it.rest == ITEM_BLAST
                || it.rest == ITEM_POCKETED
                || it.struck[a] == fa.move_id.saturating_add(1)
            {
                continue;
            }
            let Some(spec) = spec_of(content, &it).copied() else {
                continue;
            };
            if spec.hp <= Fx::ZERO {
                continue;
            }
            let at = middle(&it, &spec);
            let tall = spec.placed.map_or(Fx::ZERO, |pl| pl.hurt_half_height);
            let hit = combat::active_hitboxes(&fa, mv, pa.hitbox_scale)
                .filter(|(_, hb, c)| {
                    hb.kind == HIT_NORMAL
                        && !hb.push
                        && combat::shapes_touch(
                            *c,
                            hb.half_width,
                            hb.half_height,
                            hb.radius,
                            at,
                            Fx::ZERO,
                            tall,
                            spec.hurt_radius,
                        )
                })
                .min_by_key(|(_, hb, _)| hb.priority)
                .map(|(_, hb, _)| hb);
            let Some(hb) = hit else {
                continue;
            };
            // A placed piece (E43) only loses hit points, freezes its attacker, and is cut by a chopping move.
            if let Some(pl) = spec.placed {
                strike_placed(state, content, a, n, &pl, hb.damage, mv.chops);
                continue;
            }
            let rules = &content.rules;
            let (d0, d1) = spec.knock_damage;
            let (s0, s1) = spec.knock_speed;
            let share = if d1 > d0 {
                ((hb.damage - d0) / (d1 - d0)).clamp(Fx::ZERO, Fx::ONE)
            } else {
                Fx::ONE
            };
            let speed = s0 + (s1 - s0) * share;
            let grounded = it.platform != NONE || it.rest == ITEM_LODGED;
            let facing = if hb.x < Fx::ZERO {
                -fa.facing
            } else {
                fa.facing
            };
            let angle =
                combat::launch_angle(hb.angle.min(360), facing, grounded, Fx::from_int(100));
            let lag = combat::hitlag_frames(hb.damage, rules, false, false, Fx::ONE);
            let item = &mut state.items[n];
            item.struck[a] = fa.move_id.saturating_add(1);
            item.hp -= hb.damage;
            item.vel = Vec2::new(trig::cos(angle) * speed, trig::sin(angle) * speed);
            item.platform = NONE;
            if item.rest == ITEM_LODGED {
                item.rest = ITEM_FREE;
            }
            item.hitlag = (u16::from(lag) * u16::from(spec.knock_hitlag) / 100) as u8;
            if item.hp <= Fx::ZERO {
                item.hitlag = 0;
                burst(state, content, n);
            }
        }
    }
}

/// For the view: the item in slot `n` if it is live (and not put away in a pocket: E42).
pub fn view(state: &GameState, n: usize) -> Option<&Item> {
    state
        .items
        .get(n)
        .filter(|it| it.active && it.rest != ITEM_POCKETED)
}

// ---- E42: the pocket -------------------------------------------------------------------------------------------------------------

/// Fighter `i`'s open, empty pocket puts away the first item it touches (lowest slot): not held, not bursting, not placed, and
/// `pocketable`. The item is frozen where it was.
fn pocket_items(state: &mut GameState, content: &Content, i: usize) {
    let Some((c, r)) = combat::pocket_circle(state, content, i) else {
        return;
    };
    for n in 0..MAX_ITEMS {
        let it = state.items[n];
        if !it.active
            || it.holder != NONE
            || !matches!(it.rest, ITEM_FREE | ITEM_THROWN | ITEM_LODGED)
            || it.fuse > 0
        {
            continue;
        }
        let Some(spec) = spec_of(content, &it) else {
            continue;
        };
        if !spec.pocketable || spec.placed.is_some() {
            continue;
        }
        let m = middle(&it, spec);
        if !combat::shapes_touch(
            c,
            Fx::ZERO,
            Fx::ZERO,
            r,
            m,
            spec.half_width,
            spec.half_height,
            Fx::ZERO,
        ) {
            continue;
        }
        let item = &mut state.items[n];
        item.rest = ITEM_POCKETED;
        item.vel = Vec2::ZERO;
        item.platform = NONE;
        let f = &mut state.fighters[i];
        f.pocket = n as i8;
        f.pocket_item = true;
        return;
    }
}

/// A pocket started with an item in the hand (E42): that item is put away, and an item already in the pocket comes into the hand.
fn stow(state: &mut GameState, content: &Content, i: usize) {
    let f = state.fighters[i];
    let Ok(held) = usize::try_from(f.holding) else {
        return;
    };
    if f.pocket != NONE && !f.pocket_item {
        return;
    }
    let pocketable = state
        .items
        .get(held)
        .and_then(|it| spec_of(content, it))
        .is_some_and(|s| s.pocketable);
    if !pocketable {
        return;
    }
    let old = f.pocket;
    {
        let it = &mut state.items[held];
        it.holder = NONE;
        it.rest = ITEM_POCKETED;
        it.vel = Vec2::ZERO;
        it.platform = NONE;
    }
    let g = &mut state.fighters[i];
    g.holding = NONE;
    g.pocket = held as i8;
    g.pocket_item = true;
    if let Ok(o) = usize::try_from(old) {
        to_hand(state, content, i, o);
    }
}

/// Item `n` (out of fighter `i`'s pocket) into its empty hand, its clock starting again.
fn to_hand(state: &mut GameState, content: &Content, i: usize, n: usize) {
    let f = state.fighters[i];
    let life = spec_of(content, &state.items[n]).map_or(0, |s| s.life);
    let it = &mut state.items[n];
    it.holder = i as i8;
    it.owner = i as u8;
    it.rest = ITEM_HELD;
    it.age = 0;
    it.life = life;
    it.fuse = 0;
    it.flight = 0;
    it.hit_mask = 0;
    it.vel = Vec2::ZERO;
    it.pos = hand(&f, &content.rules);
    it.facing = f.facing;
    state.fighters[i].holding = n as i8;
}

/// Takes out what fighter `i` put away (E42, `Move::unpocket`): an item into the empty hand, a projectile flying out in front at the
/// speed it was caught with, turned the way the fighter faces, at the release's power, now the fighter's.
fn unpocket(state: &mut GameState, content: &Content, i: usize) {
    let f = state.fighters[i];
    let Ok(n) = usize::try_from(f.pocket) else {
        return;
    };
    let p = combat::params_of(content, &f);
    let Some(rel) = combat::weapon_of(content, p).get(f.move_id).unpocket else {
        return;
    };
    if f.pocket_item {
        let live = state
            .items
            .get(n)
            .is_some_and(|it| it.active && it.rest == ITEM_POCKETED);
        if live && f.holding == NONE {
            state.fighters[i].pocket = NONE;
            state.fighters[i].pocket_item = false;
            to_hand(state, content, i, n);
        } else if !live {
            state.fighters[i].pocket = NONE;
            state.fighters[i].pocket_item = false;
        }
        return;
    }
    state.fighters[i].pocket = NONE;
    let Some(pr) = state.projectiles.get_mut(n) else {
        return;
    };
    if !pr.active || pr.rest != REST_POCKETED {
        return;
    }
    let s = p.hitbox_scale;
    let dir = i32::from(f.facing);
    pr.rest = REST_FLYING;
    pr.owner = i as u8;
    pr.facing = f.facing;
    pr.pos = f.pos + Vec2::new((rel.x * s).mul_int(dir), rel.y * s);
    pr.vel = Vec2::new(pr.vel.x.abs().mul_int(dir), pr.vel.y);
    pr.age = 0;
    pr.hit_mask = 0;
    pr.hit_age = [0; MAX_FIGHTERS];
    pr.vars = [0; crate::PROJECTILE_VARS];
    pr.bursting = false;
    pr.power = rel.power.clamp(1, 255) as u8;
    pr.stale = f.move_id;
}

/// A knock-out loses what the fighter had put away (E42): it is gone.
pub fn lose_pocket(state: &mut GameState, i: usize) {
    let f = state.fighters[i];
    if let Ok(n) = usize::try_from(f.pocket) {
        if f.pocket_item {
            if let Some(it) = state.items.get_mut(n) {
                if it.rest == ITEM_POCKETED {
                    it.active = false;
                }
            }
        } else if let Some(p) = state.projectiles.get_mut(n) {
            if p.rest == REST_POCKETED {
                p.active = false;
            }
        }
    }
    state.fighters[i].pocket = NONE;
    state.fighters[i].pocket_item = false;
}

// ---- E43: placed pieces ----------------------------------------------------------------------------------------------------------

/// Fighter `i` sets down item `spec_index` of its weapon (placed): `pl.x` ahead of its feet on the platform it stands on (kept on the
/// platform), pulling up the placed pieces of its kind it overlaps and its own older one of that kind.
fn place(
    state: &mut GameState,
    content: &Content,
    i: usize,
    spec_index: u8,
    pl: &crate::moves::Placed,
) {
    let f = state.fighters[i];
    let Some(plat) = collision::platform(&content.stage, f.platform) else {
        return;
    };
    let Some(spec) = combat::weapon_of(content, combat::params_of(content, &f))
        .items
        .get(usize::from(spec_index))
        .copied()
    else {
        return;
    };
    let s = combat::params_of(content, &f).hitbox_scale;
    let x = (f.pos.x + (pl.x * s).mul_int(i32::from(f.facing))).clamp(plat.left, plat.right);
    let at = Vec2::new(x, plat.top_at(x));
    for n in 0..MAX_ITEMS {
        let it = state.items[n];
        if !it.active || it.kind != spec.kind || it.kind == 0 {
            continue;
        }
        let Some(other) = spec_of(content, &it) else {
            continue;
        };
        let mine = usize::from(it.origin) == i;
        let under = (it.pos.x - at.x).abs() < other.half_width + spec.half_width;
        if other.placed.is_some() && (mine || under) {
            remove(state, n);
        }
    }
    if let Some(n) = make(state, content, i, spec_index, at, f.move_id) {
        let it = &mut state.items[n];
        it.platform = f.platform;
        it.facing = f.facing;
    }
}

/// A placed piece that has had its water grows (E43): it becomes its `grow_into` item where it stands, and its growth's hit comes
/// out there, credited to the last fighter whose water touched it.
fn grow(state: &mut GameState, content: &Content, n: usize, pl: &crate::moves::Placed) {
    let it = state.items[n];
    remove(state, n);
    if let Some(m) = make_as(
        state,
        content,
        usize::from(it.origin),
        it.maker,
        pl.grow_into,
        it.pos,
        it.move_id,
    ) {
        let t = &mut state.items[m];
        t.platform = it.platform;
        t.facing = it.facing;
    }
    if pl.grow_move != 255 {
        combat::fire(
            state,
            content,
            usize::from(it.owner),
            usize::from(it.origin),
            pl.grow_move,
            it.pos,
            it.facing,
        );
    }
}

/// The attack of fighter `a` hits placed piece `n` (E43): its hit points, the attacker's freeze, and a chop.
fn strike_placed(
    state: &mut GameState,
    content: &Content,
    a: usize,
    n: usize,
    pl: &crate::moves::Placed,
    damage: Fx,
    chops: bool,
) {
    let fa = state.fighters[a];
    let lag = combat::hitlag_frames(damage, &content.rules, false, false, Fx::ONE);
    let frozen = (u16::from(lag) * u16::from(pl.hitter_lag) / 100) as u8;
    state.fighters[a].hitlag = state.fighters[a].hitlag.max(frozen);
    let item = &mut state.items[n];
    item.struck[a] = fa.move_id.saturating_add(1);
    item.hp -= damage;
    if chops && pl.cuts > 0 {
        // (From the left, the low nibble; from the right, the high one.)
        let right = fa.pos.x > item.pos.x;
        let shift = if right { 4 } else { 0 };
        let side = ((item.cuts >> shift) & 0xf).saturating_add(1).min(15);
        item.cuts = (item.cuts & !(0xf << shift)) | (side << shift);
        if side >= pl.cuts {
            fell(state, content, a, n, pl);
            return;
        }
    }
    if state.items[n].hp <= Fx::ZERO {
        burst(state, content, n);
    }
}

/// A placed piece felled by fighter `a` (E43): gone, its falling hit fired the way away from `a`, credited to `a`, and maybe a chip.
fn fell(state: &mut GameState, content: &Content, a: usize, n: usize, pl: &crate::moves::Placed) {
    let it = state.items[n];
    remove(state, n);
    let fa = state.fighters[a];
    let away: i8 = if fa.pos.x < it.pos.x {
        1
    } else if fa.pos.x > it.pos.x {
        -1
    } else {
        fa.facing
    };
    if pl.fall_move != 255 {
        combat::fire(
            state,
            content,
            a,
            usize::from(it.origin),
            pl.fall_move,
            it.pos,
            away,
        );
    }
    if pl.chip != 255 && pl.chip_chance > 0 && state.rng.range(1000) < u32::from(pl.chip_chance) {
        if let Some(m) = make_as(state, content, a, it.maker, pl.chip, it.pos, it.move_id) {
            let c = &mut state.items[m];
            c.platform = it.platform;
            c.facing = away;
        }
    }
}

/// Water from a shot of `owner` at `pos` (E43, `Tool::water`): each placed piece that grows with water and touches it takes `amount`.
pub(crate) fn water(
    state: &mut GameState,
    content: &Content,
    pos: Vec2,
    shape: combat::Shape,
    amount: u8,
    owner: usize,
) {
    for n in 0..MAX_ITEMS {
        let it = state.items[n];
        if !it.active || it.rest == ITEM_POCKETED {
            continue;
        }
        let Some(spec) = spec_of(content, &it) else {
            continue;
        };
        let Some(pl) = spec.placed.filter(|pl| pl.water > 0) else {
            continue;
        };
        let m = middle(&it, spec);
        let touching = combat::shapes_touch(
            pos,
            shape.half_w,
            shape.half_h,
            shape.radius,
            m,
            spec.half_width,
            spec.half_height.max(pl.hurt_half_height),
            spec.hurt_radius,
        );
        if touching {
            let item = &mut state.items[n];
            item.water = item.water.saturating_add(amount);
            item.owner = owner.min(MAX_FIGHTERS - 1) as u8;
        }
    }
}

/// A shot with a hit of `damage` at `pos` meets a placed piece with hit points (E43): the lowest slot it touches takes the damage (at
/// 0 it is gone); returns whether the shot was used up on one.
pub(crate) fn shot_on_placed(
    state: &mut GameState,
    content: &Content,
    pos: Vec2,
    shape: combat::Shape,
    damage: Fx,
) -> bool {
    for n in 0..MAX_ITEMS {
        let it = state.items[n];
        if !it.active || it.rest == ITEM_POCKETED || it.rest == ITEM_BLAST {
            continue;
        }
        let Some(spec) = spec_of(content, &it).copied() else {
            continue;
        };
        let Some(pl) = spec.placed else {
            continue;
        };
        if spec.hp <= Fx::ZERO {
            continue;
        }
        let m = middle(&it, &spec);
        if combat::shapes_touch(
            pos,
            shape.half_w,
            shape.half_h,
            shape.radius,
            m,
            Fx::ZERO,
            pl.hurt_half_height,
            spec.hurt_radius,
        ) {
            state.items[n].hp -= damage;
            if state.items[n].hp <= Fx::ZERO {
                burst(state, content, n);
            }
            return true;
        }
    }
    false
}

/// How many live items and projectiles fighter `me` made of kind `kind` (0: none): the `owns(kind)` script function (E43).
pub fn owns(
    state_items: &[Item],
    projectiles: &[crate::state::Projectile],
    me: usize,
    kind: u8,
) -> u8 {
    if kind == 0 {
        return 0;
    }
    let shots = projectiles
        .iter()
        .filter(|p| p.active && usize::from(p.origin) == me && p.kind == kind)
        .count();
    let n = owned(state_items, me, kind) as usize + shots;
    u8::try_from(n).unwrap_or(u8::MAX)
}

/// For the view: the projectile in slot `n` if it is live and not put away in a pocket (E42).
pub fn projectile_shown(state: &GameState, n: usize) -> bool {
    state
        .projectiles
        .get(n)
        .is_some_and(|p| p.active && p.rest != REST_POCKETED)
}
