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
    ITEM_THROWN, MAX_ITEMS, NONE,
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
        if !in_reach(f, rules, it, spec) {
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
    };
    Some(n)
}

/// One frame of every item: the fighters' orders, then each item's clock and movement, its hits and the attacks that knock it about.
pub fn update(state: &mut GameState, content: &Content) {
    for i in 0..MAX_FIGHTERS {
        serve(state, content, i);
    }
    for n in 0..MAX_ITEMS {
        if state.items[n].active {
            tick(state, content, n);
        }
    }
    knock_about(state, content);
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
    // (A throw that is still going, or the aerial let-go, which is no action of its own.)
    if order & ORDER_RELEASE != 0 && (f.state == S::ItemThrow || f.item_throw == THROW_LET_GO) {
        release(state, content, i);
    }
    let f = state.fighters[i];
    if order & ORDER_SPEND != 0 {
        if let Ok(n) = usize::try_from(f.holding) {
            remove(state, n);
        }
    }
    let f = state.fighters[i];
    if order & ORDER_PULL != 0 && f.holding == NONE && f.state == S::Attack {
        if let Some(n) = make(state, content, i, f.item_spec, hand(&f, rules), f.move_id) {
            let it = &mut state.items[n];
            it.holder = i as i8;
            it.rest = ITEM_HELD;
            state.fighters[i].holding = n as i8;
        }
    }
    if order & ORDER_DETONATE != 0 {
        for n in 0..MAX_ITEMS {
            let it = state.items[n];
            // (Not one another fighter holds, nor one already bursting or about to.)
            let held_by_other = it.holder != NONE && it.holder != i as i8;
            if !it.active
                || usize::from(it.origin) != i
                || held_by_other
                || it.rest == ITEM_BLAST
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
    let (speed, angle) = throw_speed(rules, throw);
    let p = combat::params_of(content, &f);
    let t = speed * toss_strength(p);
    let m = spec.throw_mul;
    let a = Angle::from_degrees(i32::from(angle));
    let side = i32::from(f.facing);
    let mut vx = (t * trig::cos(a) * m * m).mul_int(side);
    if spec.max_speed > Fx::ZERO {
        vx = vx.clamp(-spec.max_speed, spec.max_speed);
    }
    let vy = t * trig::sin(a) * m;
    // The thrown hit's damage: its base (halved for the grounded drop) plus its share of the throw's speed (the reference's).
    let base = if throw & !THROW_SMASH == THROW_DROP {
        spec.hit.damage / Fx::from_int(2)
    } else {
        spec.hit.damage
    };
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
            if spec.vanish {
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
        let to = Vec2::new(it.pos.x + it.vel.x, it.pos.y);
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
            it.pos = Vec2::new(to.x, stage.platforms[i].y);
            if into > spec.bounce_min {
                it.vel.y = pct(into, spec.floor_keep.1).clamp(spec.bounce_y.0, spec.bounce_y.1);
                it.vel.x = clamp_mag(pct(it.vel.x, spec.floor_keep.0), spec.bounce_x);
            } else {
                it.vel.y = Fx::ZERO;
                it.platform = i as i8;
            }
            // (The thrown hit ends where it meets the ground [E].)
            if it.rest == ITEM_THROWN {
                it.rest = ITEM_FREE;
            }
            state.items[n] = it;
            return;
        }
    }
    if let Some(block) = stage.platforms.iter().find(|b| {
        !b.pass_through && to.x > b.left && to.x < b.right && to.y > b.bottom && to.y < b.y
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
    stage
        .platforms
        .iter()
        .any(|b| !b.pass_through && p.x > b.left && p.x < b.right && p.y > b.bottom && p.y < b.y)
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
        if spec.vanish {
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
            let hit = combat::active_hitboxes(&fa, mv, pa.hitbox_scale)
                .filter(|(_, hb, c)| {
                    hb.kind == HIT_NORMAL
                        && combat::shapes_touch(
                            *c,
                            hb.half_width,
                            hb.half_height,
                            hb.radius,
                            at,
                            Fx::ZERO,
                            Fx::ZERO,
                            spec.hurt_radius,
                        )
                })
                .min_by_key(|(_, hb, _)| hb.priority)
                .map(|(_, hb, _)| hb);
            let Some(hb) = hit else {
                continue;
            };
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

/// For the view: the item in slot `n` if it is live.
pub fn view(state: &GameState, n: usize) -> Option<&Item> {
    state.items.get(n).filter(|it| it.active)
}
