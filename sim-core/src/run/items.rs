//! The run's items (R3, sim 142: `docs/modes/RUNNER_R3_PLAN.md`): the shop's stock and its bell, buying, the build a run carries
//! (stat stacks, properties, the one consumable slot), the rules each item is, and the finds on routes (F8).
//!
//! * **The catalogue is data** (`RunContent::items`, [`super::data::standard_items`]): each entry is one kind of rule (`ITEM_*`), its
//!   rarity, base price and numbers. Everything bought is lost when the run ends.
//! * **The shop** has two staples on their own stands (heal, extra life) and four pedestals drawn from the pools by rarity (common,
//!   rare, prized from `RunRules::prized_tier`), from a stream of the shop's own ([`stock_shop`]): the same seed gives the same shop.
//!   The bell on the counter re-draws the unsold pedestals ([`reroll`]), its price doubling with each ring in that shop. Prices rise
//!   `price_step` percent a tier ([`price`]).
//! * **Stats** rebuild the fighter through `stats::apply` (the creator's recipe arithmetic) into the run's own content copy, from the
//!   fighter's own parameters kept there as the run starts ([`stash_base`], [`refresh`]). Versus content is never touched.
//! * **Properties** are hooks in the run's step, and (the bubble, the sturdy shield) in run-only branches of the shared code; none of
//!   them can act outside a run (a match has no active run, and the fighter fields they set are 0 in versus).
//! * **Finds** ([`roll_find`], [`take_finds`]): a skilled route, a treasure alley or a fork's road may hold an item (free), food, or
//!   once a city a 1-up.

use super::data::{
    ShopItem, FIND_FOOD, FIND_ITEM, FIND_LIFE, FIND_NONE, ITEM_BOMB, ITEM_BUBBLE, ITEM_HEAL,
    ITEM_HOT_HANDS, ITEM_IRON, ITEM_KINDS, ITEM_LIFE, ITEM_LIFESTEAL, ITEM_MAGNET, ITEM_ORBS,
    ITEM_POWER, ITEM_REACH, ITEM_SECOND_WIND, ITEM_SPEED, ITEM_SPRING, ITEM_STEADY, ITEM_STURDY,
    ITEM_WEIGHT, ITEM_WING, MAX_SHOP_ITEMS, NO_ITEM, PEDESTALS, RARITY_COMMON, RARITY_PRIZED,
    RARITY_RARE, RARITY_STAPLE, ROOM_ALLEY, STAPLES,
};
use super::{
    city_of, for_each_footing, mobs, tier_of, Placed, RoomSlot, Run, RunContent, KEEPER_DIZZY,
    KEEPER_HMPH, KEEPER_RESTOCK, MAX_MOBS, NOTE_BOUGHT, NOTE_NONE, NOTE_NOT_NEEDED, NOTE_REFUSED,
    NOTE_REROLLED, NOTE_SLOT_FULL, NOTE_SOLD_OUT, NOTE_TOO_POOR, PLAYER,
};
use crate::combat::{self, params_of};
use crate::content::Content;
use crate::fixed::Fx;
use crate::moves::MoveId;
use crate::rng::Rng;
use crate::state::{
    FighterState as S, GameState, MatchRules, Projectile, ITEM_FREE, ITEM_THROWN, NONE, ORB_CIRCLE,
};
use crate::vec2::Vec2;
use crate::MAX_FIGHTERS;

/// The bell's spot in [`super::shop_spot`] (after the wares).
pub const BELL_SPOT: usize = MAX_SHOP_ITEMS;
/// The player slot no one plays that stands in as the shadow orbs' firer (its body is the one whose weapon has the orbs).
pub const ORB_PROXY: usize = MAX_FIGHTERS - 1;

/// The catalogue entry of item kind `kind`.
pub fn index_of(rc: &RunContent, kind: u8) -> Option<usize> {
    rc.items.iter().position(|i| i.kind == kind)
}

pub fn item_of(rc: &RunContent, kind: u8) -> Option<&ShopItem> {
    rc.items.iter().find(|i| i.kind == kind)
}

/// How many of item kind `kind` the run owns.
pub fn level(run: &Run, kind: u8) -> u8 {
    run.owned.get(usize::from(kind)).copied().unwrap_or(0)
}

/// Kind `kind`'s `amount` times the stacks owned.
fn stacked(run: &Run, rc: &RunContent, kind: u8) -> Fx {
    let n = level(run, kind);
    if n == 0 {
        return Fx::ZERO;
    }
    item_of(rc, kind).map_or(Fx::ZERO, |i| i.amount.mul_int(i32::from(n)))
}

/// The catalogue's staples, in order (each has its own stand).
fn staple(rc: &RunContent, n: usize) -> Option<usize> {
    rc.items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.rarity == RARITY_STAPLE)
        .nth(n)
        .map(|(k, _)| k)
}

/// How many ware spots the shop has (the staples' stands, then the pedestals; at most the interior's stands).
pub fn ware_spots(rc: &RunContent) -> usize {
    rc.interior.pedestals.len().min(MAX_SHOP_ITEMS)
}

/// The item (catalogue index) on ware spot `spot` of the shop the player is in: a staple, or a pedestal's stock (none if empty).
pub fn ware_item(run: &Run, rc: &RunContent, spot: usize) -> Option<usize> {
    if spot >= ware_spots(rc) {
        return None;
    }
    if spot < STAPLES {
        return staple(rc, spot);
    }
    let i = *run.shop_stock.get(spot - STAPLES)?;
    (usize::from(i) < rc.items.len()).then_some(usize::from(i))
}

/// The price of catalogue item `idx` in a shop of tier `tier` (0 the first): a staple's as it is; the others `price_step` percent
/// more a tier.
pub fn price(rc: &RunContent, idx: usize, tier: u8) -> u32 {
    let Some(item) = rc.items.get(idx) else {
        return 0;
    };
    let base = u32::from(item.price);
    if item.rarity == RARITY_STAPLE {
        return base;
    }
    let pct = 100 + u32::from(rc.rules.price_step) * u32::from(tier);
    (base * pct + 50) / 100
}

/// What the bell costs now in the shop the player is in.
pub fn bell_price(run: &Run, rc: &RunContent) -> u32 {
    u32::from(rc.rules.reroll_price) << u32::from(run.shop_rerolls.min(16))
}

/// Whether the run could own one more of catalogue item `idx` (under its cap).
fn under_cap(run: &Run, item: &ShopItem) -> bool {
    item.cap == 0 || level(run, item.kind) < item.cap
}

/// Draws an item for a pedestal (or a find) of tier `tier` from the pools: a rarity by the rules' odds, then an item of it by its
/// odds, leaving out staples, items already in `taken` and items the run owns its cap of (another pool if that one is empty).
pub fn draw(rng: &mut Rng, rc: &RunContent, run: &Run, tier: u8, taken: &[u8]) -> u8 {
    let r = &rc.rules;
    let roll = rng.range(100);
    let prized_ok = tier >= r.prized_tier;
    let prized = if prized_ok {
        u32::from(r.prized_odds)
    } else {
        0
    };
    let want = if roll < prized {
        RARITY_PRIZED
    } else if roll < prized + u32::from(r.rare_odds) {
        RARITY_RARE
    } else {
        RARITY_COMMON
    };
    let ok = |k: usize, rarity: u8| {
        let i = &rc.items[k];
        i.rarity == rarity
            && i.odds > 0
            && !taken.contains(&(k as u8))
            && under_cap(run, i)
            && k < usize::from(NO_ITEM)
    };
    let order: [u8; 3] = match want {
        RARITY_PRIZED => [RARITY_PRIZED, RARITY_RARE, RARITY_COMMON],
        RARITY_RARE => [RARITY_RARE, RARITY_COMMON, RARITY_PRIZED],
        _ => [RARITY_COMMON, RARITY_RARE, RARITY_PRIZED],
    };
    for rarity in order {
        if rarity == RARITY_PRIZED && !prized_ok {
            continue;
        }
        let total: u32 = (0..rc.items.len())
            .filter(|&k| ok(k, rarity))
            .map(|k| u32::from(rc.items[k].odds))
            .sum();
        if total == 0 {
            continue;
        }
        let mut pick = rng.range(total);
        for k in 0..rc.items.len() {
            if !ok(k, rarity) {
                continue;
            }
            let w = u32::from(rc.items[k].odds);
            if pick < w {
                return k as u8;
            }
            pick -= w;
        }
    }
    NO_ITEM
}

/// Stocks shop `seq`'s pedestals as the player first walks in: its own stream (from the run's seed and the shop's room, so a replay
/// gets the same shop), no bell rung yet.
pub fn stock_shop(run: &mut Run, rc: &RunContent, seq: u32) {
    run.shop_rng = Rng::new(
        run.seed ^ 0x5348_4f50_5354_4f43 ^ u64::from(seq).wrapping_mul(0x9E37_79B9_7F4A_7C15),
    );
    run.shop_rerolls = 0;
    run.shop_stock = [NO_ITEM; PEDESTALS];
    let tier = tier_of(rc, seq);
    let mut rng = run.shop_rng;
    let mut stock = [NO_ITEM; PEDESTALS];
    for k in 0..PEDESTALS.min(ware_spots(rc).saturating_sub(STAPLES)) {
        stock[k] = draw(&mut rng, rc, run, tier, &stock[..k]);
    }
    run.shop_rng = rng;
    run.shop_stock = stock;
}

/// Rings the bell: the unsold pedestals are re-drawn for `bell_price` coins (which then doubles, in this shop). Refused while the
/// shopkeeper sulks or the coins are short.
pub fn reroll(state: &mut GameState, rc: &RunContent) -> u8 {
    let run = &mut state.run;
    let cost = bell_price(run, rc);
    let note = if run.keeper_grudge != 0 {
        NOTE_REFUSED
    } else if run.coins < cost {
        NOTE_TOO_POOR
    } else {
        NOTE_REROLLED
    };
    if note == NOTE_REROLLED {
        run.coins -= cost;
        run.shop_rerolls = run.shop_rerolls.saturating_add(1);
        let tier = tier_of(rc, run.shop_room);
        let mut rng = run.shop_rng;
        let mut stock = run.shop_stock;
        // (The sold pedestals stay empty; the others are drawn again, none twice.)
        let unsold: [bool; PEDESTALS] = core::array::from_fn(|k| {
            run.shop_bought[STAPLES + k] == 0 && k + STAPLES < ware_spots(rc)
        });
        for k in 0..PEDESTALS {
            if unsold[k] {
                stock[k] = NO_ITEM;
            }
        }
        for k in 0..PEDESTALS {
            if unsold[k] {
                let taken = stock;
                stock[k] = draw(&mut rng, rc, run, tier, &taken);
            }
        }
        run.shop_rng = rng;
        run.shop_stock = stock;
    }
    let run = &mut state.run;
    if run.keeper_mood != KEEPER_DIZZY && run.keeper_mood != KEEPER_HMPH {
        (run.keeper_mood, run.keeper_timer) = if note == NOTE_REROLLED {
            (KEEPER_RESTOCK, rc.interior.cheer_frames)
        } else {
            (super::KEEPER_GRUMBLE, rc.interior.grumble_frames)
        };
    }
    run.shop_note = note;
    run.shop_acts = run.shop_acts.wrapping_add(1);
    note
}

/// Why catalogue item `idx` cannot be had now, if it cannot (not needed, or the consumable slot taken); `None` if it can.
fn refusal(state: &GameState, rc: &RunContent, idx: usize) -> Option<u8> {
    let item = rc.items.get(idx)?;
    let f = &state.fighters[PLAYER];
    let run = &state.run;
    let not_needed = match item.kind {
        ITEM_HEAL => f.percent <= Fx::ZERO,
        ITEM_LIFE => f.stocks >= MatchRules::MAX_STOCKS,
        _ => !under_cap(run, item),
    };
    if not_needed {
        return Some(NOTE_NOT_NEEDED);
    }
    if matches!(item.kind, ITEM_BUBBLE | ITEM_BOMB) && run.slot != NO_ITEM {
        return Some(NOTE_SLOT_FULL);
    }
    None
}

/// Buys ware spot `spot` of the shop the player is in. Refused (nothing paid) while the shopkeeper sulks, for a pedestal sold or
/// empty, a staple past its limit in this shop, coins short, an item it would do nothing for (a full heal, the most lives, a cap
/// reached), or a consumable with the slot taken.
pub fn buy(state: &mut GameState, rc: &RunContent, spot: usize) -> u8 {
    let note = match ware_item(&state.run, rc, spot) {
        _ if spot >= ware_spots(rc) => NOTE_NONE,
        _ if state.run.keeper_grudge != 0 => NOTE_REFUSED,
        None => NOTE_SOLD_OUT,
        Some(idx) => {
            let item = &rc.items[idx];
            let run = &state.run;
            let cost = price(rc, idx, tier_of(rc, run.shop_room));
            if item.limit > 0 && run.shop_bought[spot] >= item.limit {
                NOTE_SOLD_OUT
            } else if run.coins < cost {
                NOTE_TOO_POOR
            } else if let Some(no) = refusal(state, rc, idx) {
                no
            } else {
                let run = &mut state.run;
                run.coins -= cost;
                run.shop_bought[spot] = run.shop_bought[spot].saturating_add(1);
                grant(state, rc, idx);
                NOTE_BOUGHT
            }
        }
    };
    state.run.shop_note = note;
    state.run.shop_acts = state.run.shop_acts.wrapping_add(1);
    note
}

/// The run gets catalogue item `idx` (bought or found): a staple at once; a consumable into the slot; a stat or a property owned
/// one more (its rule reads the count).
pub fn grant(state: &mut GameState, rc: &RunContent, idx: usize) {
    let Some(item) = rc.items.get(idx) else {
        return;
    };
    let f = &mut state.fighters[PLAYER];
    match item.kind {
        ITEM_HEAL => f.percent = (f.percent - item.amount).max(Fx::ZERO),
        ITEM_LIFE => {
            let more = item.amount.floor_int().clamp(1, 9) as u8;
            f.stocks = f.stocks.saturating_add(more).min(MatchRules::MAX_STOCKS);
        }
        _ => {}
    }
    let (percent, stocks) = (f.percent, f.stocks);
    let run = &mut state.run;
    match item.kind {
        ITEM_BUBBLE => run.slot = ITEM_BUBBLE,
        ITEM_BOMB => {
            run.slot = ITEM_BOMB;
            run.bomb = NONE;
            run.bomb_thrown = false;
        }
        ITEM_STEADY => run.guard = true,
        _ => {}
    }
    if let Some(n) = run.owned.get_mut(usize::from(item.kind)) {
        *n = n.saturating_add(1);
    }
    run.last_percent = percent;
    run.last_stocks = stocks;
}

/// The bubble takes the hit (called from `combat::apply_hit`, in a run only): it pops, the slot is free, and the player is
/// untouchable for the rest of the frame (the run gives the bubble's frames after the step).
pub fn pop_bubble(state: &mut GameState) {
    state.run.slot = NO_ITEM;
    state.run.pops = state.run.pops.wrapping_add(1);
    let f = &mut state.fighters[PLAYER];
    f.invuln = f.invuln.max(2);
}

// ---- The run's start --------------------------------------------------------------------------------------------------------

/// Keeps fighter `char_id`'s own parameters in the run's content copy (an extra fighter entry, found again by its name when the same
/// content starts another run, and the fighter put back from it first), so stat items rebuild from them. Returns its index.
pub fn stash_base(world: &mut Content, char_id: u8) -> u8 {
    let tag = format!("run-base-{char_id}");
    let c = usize::from(char_id).min(world.fighters.len().saturating_sub(1));
    if let Some(i) = world.names.fighters.iter().position(|n| *n == tag) {
        if i < world.fighters.len() && i <= 255 {
            world.fighters[c] = world.fighters[i];
            return i as u8;
        }
    }
    if world.fighters.len() >= 255 || world.fighters.is_empty() {
        return char_id;
    }
    while world.names.fighters.len() < world.fighters.len() {
        let n = world.names.fighters.len();
        world.names.fighters.push(format!("fighter{n}"));
    }
    world.fighters.push(world.fighters[c]);
    world.names.fighters.truncate(world.fighters.len() - 1);
    world.names.fighters.push(tag);
    (world.fighters.len() - 1) as u8
}

/// The body whose weapon has the shadow orbs (E59), and that move: what the orbs item fires them as.
pub fn find_orbs(world: &Content) -> Option<(u8, u8)> {
    for (b, p) in world.fighters.iter().enumerate().take(255) {
        let w = combat::weapon_of(world, p);
        if let Some(m) = w.moves.iter().position(|m| m.orbs.is_some()) {
            return Some((b as u8, m as u8));
        }
    }
    None
}

/// The body whose weapon makes a bomb (an item with a burst), and the item's index there.
fn find_bomb(world: &Content) -> Option<(u8, u8)> {
    for (b, p) in world.fighters.iter().enumerate().take(255) {
        let w = combat::weapon_of(world, p);
        if let Some(i) = w.items.iter().position(|s| s.blast.is_some()) {
            return Some((b as u8, i as u8));
        }
    }
    None
}

// ---- Every frame ------------------------------------------------------------------------------------------------------------

/// The build on the fighter, before each step: the run's copy of its parameters rebuilt from its own through the creator's stats
/// (speed, weight, spring) with its extra wings and longer reach, its damage (power) and its shield (sturdy). With nothing owned the
/// copy is the fighter's own.
pub fn refresh(state: &mut GameState, world: &mut Content, rc: &RunContent) {
    let run = &state.run;
    let char_id = usize::from(state.fighters[PLAYER].char_id);
    let base = usize::from(run.base_fighter);
    if base != char_id && base < world.fighters.len() && char_id < world.fighters.len() {
        world.fighters[char_id] = built(world.fighters[base], run, rc);
    }
    let power = stacked(run, rc, ITEM_POWER).floor_int();
    let sturdy = item_of(rc, ITEM_STURDY).filter(|_| level(run, ITEM_STURDY) > 0);
    let f = &mut state.fighters[PLAYER];
    f.damage_scale = if power > 0 {
        (100 + power).clamp(1, i32::from(u16::MAX)) as u16
    } else {
        0
    };
    (f.shield_scale, f.shield_wear) = match sturdy {
        Some(i) => (
            (100 + i.amount.floor_int()).clamp(1, 255) as u8,
            (100 - i.extra.floor_int()).clamp(1, 255) as u8,
        ),
        None => (0, 0),
    };
}

/// A fighter's own parameters `base` with the run's stat items, extra wings and reach on.
pub fn built(
    base: crate::content::FighterParams,
    run: &Run,
    rc: &RunContent,
) -> crate::content::FighterParams {
    let points = |kind: u8| stacked(run, rc, kind).floor_int();
    let (speed, weight, jump) = (points(ITEM_SPEED), points(ITEM_WEIGHT), points(ITEM_SPRING));
    let mut p = if speed == 0 && weight == 0 && jump == 0 {
        base
    } else {
        let n = crate::stats::NEUTRAL;
        crate::stats::apply(base, n, n + speed, n + jump, n + weight)
    };
    let wings = points(ITEM_WING);
    if wings > 0 {
        p.air_jumps = (i32::from(p.air_jumps) + wings).clamp(0, 255) as u8;
    }
    let reach = points(ITEM_REACH);
    if reach > 0 {
        p.hitbox_scale = p.hitbox_scale * Fx::from_ratio(100 + reach, 100);
    }
    p
}

/// Before the step (outside a shop): iron startup's super armor through the startup of a smash attack, and the shadow orbs kept
/// circling.
pub fn before_step(state: &mut GameState, world: &Content, rc: &RunContent) {
    let f = state.fighters[PLAYER];
    let smash = matches!(
        MoveId::from_index(f.move_id),
        MoveId::FSmash | MoveId::USmash | MoveId::DSmash
    );
    let armor = level(&state.run, ITEM_IRON) > 0
        && f.state == S::Attack
        && smash
        && f.state_frame < mobs::first_active(world, &f, MoveId::from_index(f.move_id));
    state.fighters[PLAYER].super_armor = armor;
    orbs(state, world, rc);
}

/// The shadow orbs (the Exile's, E59, `Move::orbs`): with the item, up to its `amount` orbs circle the player, credited to the
/// player and fired as the orb body's move from the unplayed player slot [`ORB_PROXY`]; the player's orb clock is held so they never
/// strike, and a spent orb is replaced every `frames` frames.
fn orbs(state: &mut GameState, world: &Content, rc: &RunContent) {
    let Some(item) = item_of(rc, ITEM_ORBS).filter(|_| level(&state.run, ITEM_ORBS) > 0) else {
        return;
    };
    let run = &state.run;
    if run.orb_body == NO_ITEM || usize::from(run.orb_body) >= world.fighters.len() {
        return;
    }
    let (body, mv) = (run.orb_body, run.orb_move);
    let proxy = &mut state.fighters[ORB_PROXY];
    if !proxy.active {
        proxy.char_id = body;
    }
    let f = state.fighters[PLAYER];
    if !f.active || f.state == S::Respawn {
        return;
    }
    let circling = state
        .projectiles
        .iter()
        .filter(|p| p.active && p.orb == ORB_CIRCLE && usize::from(p.held) == PLAYER)
        .count();
    if circling > 0 {
        let g = &mut state.fighters[PLAYER];
        g.orb_clock = g.orb_clock.max(600);
    }
    let want = item.amount.floor_int().clamp(0, 8) as usize;
    let run = &mut state.run;
    run.orb_timer = run.orb_timer.saturating_add(1);
    if circling >= want || run.orb_timer < item.frames.max(1) && circling > 0 {
        return;
    }
    run.orb_timer = 0;
    let at = f.pos + Vec2::new(Fx::ZERO, combat::chest_height(params_of(world, &f)));
    let Some(slot) = state.projectiles.iter_mut().find(|p| !p.active) else {
        return;
    };
    *slot = Projectile {
        active: true,
        owner: PLAYER as u8,
        origin: ORB_PROXY as u8,
        power: 100,
        move_id: mv,
        pos: at,
        life: 255,
        size: 100,
        facing: f.facing,
        stale: mv,
        orb: ORB_CIRCLE,
        held: PLAYER as u8,
        ..Projectile::default()
    };
    let g = &mut state.fighters[PLAYER];
    g.orb_clock = g.orb_clock.max(600);
}

/// After the step (outside a shop), before the combo is settled: the bubble's frames after a pop, second wind after a lost life,
/// hot hands' burns, and the bought bomb (handed over, and burst where its throw lands or hits).
pub fn after_step(state: &mut GameState, world: &Content, rc: &RunContent, pops_before: u16) {
    if state.run.pops != pops_before {
        let frames = item_of(rc, ITEM_BUBBLE).map_or(0, |i| i.frames);
        let f = &mut state.fighters[PLAYER];
        f.invuln = f.invuln.max(frames.min(255) as u8);
    }
    // Second wind: a lost life gives its frames once the respawn invincibility is over.
    let f = state.fighters[PLAYER];
    if f.stocks < state.run.last_stocks {
        let frames = item_of(rc, ITEM_SECOND_WIND).map_or(0, |i| i.frames);
        state.run.wind = frames.saturating_mul(u16::from(level(&state.run, ITEM_SECOND_WIND)));
    }
    if state.run.wind > 0 && f.active && f.state != S::Respawn && f.invuln <= 2 {
        state.run.wind -= 1;
        state.fighters[PLAYER].invuln = 2;
    }
    burns(state, rc);
    bomb(state, world);
}

/// Hot hands: the burn a hit of the player's on a grunt starts (frames; 0 without the item).
pub fn burn_frames(run: &Run, rc: &RunContent) -> u8 {
    if level(run, ITEM_HOT_HANDS) == 0 {
        return 0;
    }
    item_of(rc, ITEM_HOT_HANDS).map_or(0, |i| i.frames.min(255) as u8)
}

/// Hot hands: a burning grunt takes `amount` percent every `extra` frames of its burn (not a hit: no combo, no credit).
fn burns(state: &mut GameState, rc: &RunContent) {
    let Some(item) = item_of(rc, ITEM_HOT_HANDS) else {
        return;
    };
    let period = item.extra.floor_int().clamp(1, 255) as u8;
    let dose = item
        .amount
        .mul_int(i32::from(level(&state.run, ITEM_HOT_HANDS).max(1)));
    for n in 0..MAX_MOBS {
        let m = state.run.mobs[n];
        if !m.active || m.burn == 0 {
            continue;
        }
        let burn = m.burn - 1;
        if burn % period == 0 {
            let g = &mut state.fighters[mobs::slot(n)];
            g.percent = (g.percent + dose).min(Fx::from_int(999));
            state.run.mobs[n].last_percent = g.percent;
        }
        state.run.mobs[n].burn = burn;
    }
}

/// The bought bomb: once the player is outside with a free hand it is put in it (the buckler's bomb, made from the body whose weapon
/// has one); thrown, it bursts the moment its throw hits something or it comes to rest, and its burst never hurts the player. The slot
/// is free once it is gone.
fn bomb(state: &mut GameState, world: &Content) {
    if state.run.slot != ITEM_BOMB {
        return;
    }
    let Some((body, spec)) = find_bomb(world) else {
        state.run.slot = NO_ITEM;
        return;
    };
    let Ok(n) = usize::try_from(state.run.bomb) else {
        let f = state.fighters[PLAYER];
        if crate::items::can_hold(&f) && f.holding == NONE && f.state != S::Respawn {
            let at = crate::items::hand(&f, &world.rules);
            if let Some(n) = crate::items::make_as(
                state,
                world,
                PLAYER,
                body,
                spec,
                at,
                crate::items::ITEM_THROW_KEY,
            ) {
                crate::items::hand_to(state, world, PLAYER, n);
                state.run.bomb = n as i8;
                state.run.bomb_thrown = false;
            }
        }
        return;
    };
    let it = state.items.get(n).copied().unwrap_or_default();
    if !it.active || it.maker != body || it.spec != spec || usize::from(it.origin) != PLAYER {
        state.run.slot = NO_ITEM;
        state.run.bomb = NONE;
        state.run.bomb_thrown = false;
        return;
    }
    if it.rest == crate::state::ITEM_BLAST {
        return;
    }
    // (Thrown since it was handed over: its flight counts only while thrown, and a drop from a hit leaves it at 0.)
    if it.rest == ITEM_THROWN || it.flight > 0 {
        state.run.bomb_thrown = true;
    }
    let hit = it.rest == ITEM_THROWN && it.hit_mask != 0;
    let landed = state.run.bomb_thrown && it.rest == ITEM_FREE;
    if hit || landed {
        crate::items::burst_now(state, world, n);
        state.items[n].hit_mask |= 1 << PLAYER;
    }
}

// ---- Finds on routes (F8) ---------------------------------------------------------------------------------------------------

/// Whether room slot `slot` may hold a find: a fork's road (a treasure alley or the road below it), a treasure alley, or a room whose
/// skilled route is laid.
fn may_find(rc: &RunContent, slot: &RoomSlot) -> bool {
    let Some(room) = rc.rooms.get(usize::from(slot.room)) else {
        return false;
    };
    room.paired
        || room.kind == ROOM_ALLEY
        || (!slot.skill_off && room.platforms.iter().any(|p| p.skilled))
}

/// Where a room's find stands (its feet): on top of the middle of its highest skilled footing, or else of its highest pass-through
/// footing, or else of its highest footing.
pub fn find_spot(rc: &RunContent, slot: &RoomSlot) -> Option<Vec2> {
    let mut all: Vec<Placed> = Vec::new();
    for_each_footing(rc, slot, |p| all.push(p));
    let top = |p: &Placed| p.platform.y.max(p.platform.top_right());
    let best =
        |f: &dyn Fn(&Placed) -> bool| all.iter().filter(|p| f(p)).max_by_key(|p| top(p)).copied();
    let p = best(&|p| p.skilled)
        .or_else(|| best(&|p| p.platform.pass_through))
        .or_else(|| best(&|_| true))?;
    let mid = (p.platform.left + p.platform.right) / Fx::from_int(2);
    Some(Vec2::new(mid, p.platform.top_at(mid)))
}

/// As a room is laid: whether it holds a find, and what (from a stream of its own, from the run's seed and the room's number, so the
/// rooms themselves never change for it). Once a city a 1-up may come instead.
pub fn roll_find(run: &mut Run, rc: &RunContent, slot: &mut RoomSlot) {
    slot.find = FIND_NONE;
    slot.find_item = NO_ITEM;
    if !may_find(rc, slot) || find_spot(rc, slot).is_none() {
        return;
    }
    let r = &rc.rules;
    let mut rng = Rng::new(
        run.seed ^ 0x4649_4e44_4649_4e44 ^ u64::from(slot.seq).wrapping_mul(0xD6E8_FEB8_6659_FD93),
    );
    let city = city_of(rc, slot.seq) + 1;
    if run.life_city != city && rng.range(100) < u32::from(r.life_chance) {
        run.life_city = city;
        slot.find = FIND_LIFE;
        return;
    }
    if rng.range(100) >= u32::from(r.find_chance) {
        return;
    }
    if rng.range(100) < u32::from(r.find_food) {
        slot.find = FIND_FOOD;
        return;
    }
    let item = draw(&mut rng, rc, run, slot.tier, &[]);
    if item != NO_ITEM {
        slot.find = FIND_ITEM;
        slot.find_item = item;
    }
}

/// Finds the player touches are taken: food heals, a 1-up gives a life, an item is the player's (free). One that would do nothing
/// now (a full heal, the most lives, a cap reached, the slot taken) stays where it is.
pub fn take_finds(state: &mut GameState, rc: &RunContent) {
    let f = state.fighters[PLAYER];
    if !f.active || f.state == S::Respawn {
        return;
    }
    let body = f.pos + Vec2::new(Fx::ZERO, Fx::ONE);
    let reach = rc.rules.coin_reach + Fx::HALF;
    for s in 0..state.run.slots.len() {
        let slot = state.run.slots[s];
        if !slot.live || slot.find == FIND_NONE || slot.found {
            continue;
        }
        let Some(at) = find_spot(rc, &slot) else {
            continue;
        };
        let d = at + Vec2::new(Fx::ZERO, Fx::HALF) - body;
        if d.x.abs() > reach || d.y.abs() > reach + Fx::HALF {
            continue;
        }
        let taken = match slot.find {
            FIND_FOOD => {
                let g = &mut state.fighters[PLAYER];
                let ok = g.percent > Fx::ZERO;
                if ok {
                    g.percent = (g.percent - rc.rules.food_heal).max(Fx::ZERO);
                    state.run.last_percent = g.percent;
                }
                ok
            }
            FIND_LIFE => {
                let g = &mut state.fighters[PLAYER];
                let ok = g.stocks < MatchRules::MAX_STOCKS;
                if ok {
                    g.stocks += 1;
                    state.run.last_stocks = g.stocks;
                }
                ok
            }
            _ => {
                let idx = usize::from(slot.find_item);
                let ok = idx < rc.items.len() && refusal(state, rc, idx).is_none();
                if ok {
                    grant(state, rc, idx);
                }
                ok
            }
        };
        if taken {
            let run = &mut state.run;
            run.slots[s].found = true;
            run.finds = run.finds.wrapping_add(1);
            run.find_last = slot.find;
            run.layout = run.layout.wrapping_add(1);
        }
    }
}

/// Lifesteal: a grunt knock-out of the player's heals.
pub fn steal(state: &mut GameState, rc: &RunContent) {
    let heal = stacked(&state.run, rc, ITEM_LIFESTEAL);
    if heal <= Fx::ZERO {
        return;
    }
    let f = &mut state.fighters[PLAYER];
    f.percent = (f.percent - heal).max(Fx::ZERO);
    let p = f.percent;
    state.run.last_percent = p.min(state.run.last_percent);
    state.run.steals = state.run.steals.wrapping_add(1);
}

/// How near a coin is picked up: the rules' reach, or the coin magnet's.
pub fn coin_reach(run: &Run, rc: &RunContent) -> Fx {
    let magnet = if level(run, ITEM_MAGNET) > 0 {
        item_of(rc, ITEM_MAGNET).map_or(Fx::ZERO, |i| i.amount)
    } else {
        Fx::ZERO
    };
    rc.rules.coin_reach.max(magnet)
}

/// Whether steady combo forgives a hit now.
pub fn steady(run: &Run) -> bool {
    level(run, ITEM_STEADY) > 0
}

/// Every item kind's count (for the HUD and the results): the owned array as kinds.
pub fn owned_kinds(run: &Run) -> [u8; ITEM_KINDS] {
    run.owned
}
