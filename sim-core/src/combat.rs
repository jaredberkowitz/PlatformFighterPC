//! Hit detection and damage. Runs once per frame after every fighter has moved.
//!
//! Hitboxes and hurtboxes are circles. Hurtboxes come from the fighter's ECB data (never from the mesh);
//! hitboxes come from the weapon's move data. All maths is fixed point.
//!
//! Formulas follow the reference game's published knockback model:
//! `KB = ((((p/10 + p*d/20) * 200/(w+100) * 1.4) + 18) * g/100) + b`, where `p` is the percent after the
//! hit, `d` the damage, `w` the weight, `g` the knockback growth and `b` the base knockback.
//! The knockback is then scaled by rage (the attacker's percent) and crouch cancelling, and the damage by stale-move negation.
//! Launch speed is `KB * 0.03` reference units per frame; hitstun is `floor(KB * 0.4) - 1` frames; hitlag is
//! `floor(d * 0.65 + 6)` frames for both fighters (less on a shield or against a crouch), at most 30.

use crate::collision;
use crate::content::{Content, FighterParams, Ruleset};
use crate::fighter;
use crate::fixed::Fx;
use crate::grab;
use crate::moves::{
    Hitbox, Move, MoveId, Reflector, Weapon, AIM_DOWN, AIM_STRAIGHT, AIM_UP, ANGLE_AUTOLINK,
    ANGLE_BACKWARD, ANGLE_BY_POSITION, ANGLE_BY_TRAVEL, ANGLE_FORWARD, ANGLE_HALF_LINK, ANGLE_PULL,
    ANGLE_VORTEX, ARMOR_KNOCKBACK, EFFECT_BURY, EFFECT_ELECTRIC, EFFECT_ENCASE, HIT_GRAB,
    HIT_NORMAL, HIT_PUMMEL, HIT_SEARCH, HIT_THROW, MODE_PLAIN, MODE_POWERED, RULES_PREVIOUS,
    TARGETS_AERIAL, TARGETS_GROUNDED,
};
use crate::scripting;
use crate::state::{
    Fighter, FighterState as S, GameState, Projectile, NONE, ORB_CIRCLE, ORB_LOOSE, ORB_NONE,
    ORB_STRIKE, REST_FALLEN, REST_FLYING, REST_LODGED, REST_POCKETED, STALE_QUEUE,
};
use crate::trig::Angle;
use crate::vec2::Vec2;
use crate::{MAX_BODIES, MAX_FIGHTERS};

/// Three up the body and one for a limb thrown out in an attack (see `hurtboxes`).
pub const HURT_CIRCLES: usize = 4;

pub fn params_of<'a>(content: &'a Content, f: &Fighter) -> &'a FighterParams {
    let idx = usize::from(f.char_id).min(content.fighters.len().saturating_sub(1));
    &content.fighters[idx]
}

pub fn weapon_of<'a>(content: &'a Content, p: &FighterParams) -> &'a Weapon {
    let idx = usize::from(p.weapon).min(content.weapons.len().saturating_sub(1));
    &content.weapons[idx]
}

/// Three circles stacked up the body, sized from the ECB (lower while crouching: `crouch_height`), and a fourth for a limb.
/// Standing or walking it is the guard held out in front. With a weapon of `limbs` (claws, fists, feet), while an attack's hit
/// is out the arm or leg throwing it reaches toward the farthest of its hitboxes and can be hit there; otherwise the fourth
/// circle is the middle one again.
pub fn hurtboxes(f: &Fighter, p: &FighterParams, w: &Weapon) -> [(Vec2, Fx); HURT_CIRCLES] {
    let radius = p.ecb_half_width * Fx::from_ratio(17, 20);
    let height = if f.state == S::Crouch && f.grounded() {
        p.ecb_height * p.crouch_height
    } else {
        p.ecb_height
    };
    let at =
        |twentieths: i32| Vec2::new(f.pos.x, f.pos.y + height * Fx::from_ratio(twentieths, 20));
    let middle = at(11);
    let mut limb = (middle, radius);
    if matches!(f.state, S::Idle | S::Walk) && f.grounded() {
        // Standing or walking, the fighter holds its guard up in front (fists, or a hilt), past the body circles: the fourth circle
        // covers the hands, so a hit that reaches them connects, as it looks like it should against someone facing you.
        let ahead = p.ecb_half_width * Fx::from_ratio(4, 5);
        limb = (
            middle + Vec2::new(ahead.mul_int(i32::from(f.facing)), Fx::ZERO),
            p.ecb_half_width * Fx::from_ratio(1, 2),
        );
    }
    if w.limbs && matches!(f.state, S::Attack | S::LedgeAttack) {
        let mut far = Fx::ZERO;
        for (_, hb, centre) in active_hitboxes(f, w.get(f.move_id), p.hitbox_scale) {
            let reach = (centre - middle).length();
            if hb.kind == HIT_NORMAL && reach > far {
                far = reach;
                // Most of the way out to the hit, and a little thinner than it: the forearm or shin behind the fist or foot.
                limb = (
                    middle + (centre - middle) * Fx::from_ratio(4, 5),
                    hb.radius * Fx::from_ratio(7, 10),
                );
            }
        }
    }
    [(at(6), radius), (middle, radius), (at(16), radius), limb]
}

/// The numbers of the egg a fighter is in (E8: the `moves::Encase` of the move that laid it, read from the body that laid it), or
/// `None` when it is not in one (or the move has none).
pub fn encase_of(content: &Content, f: &Fighter) -> Option<crate::moves::Encase> {
    if f.state != S::Encased {
        return None;
    }
    let p = content.fighters.get(usize::from(f.encase_char))?;
    weapon_of(content, p).get(f.encase_move).encase
}

/// [`hurtboxes`] as the match sees them: a buried fighter's (E5) sit `Ruleset::bury_depth` of its height lower, only its upper body
/// out of the ground [E].
pub fn body_hurtboxes(content: &Content, f: &Fighter) -> [(Vec2, Fx); HURT_CIRCLES] {
    let p = params_of(content, f);
    let mut hurt = hurtboxes(f, p, weapon_of(content, p));
    if f.state == S::Buried {
        let sunk = p.ecb_height * content.rules.bury_depth;
        for (c, _) in &mut hurt {
            c.y -= sunk;
        }
    }
    // An egg (E8) is one round hurtbox standing on the feet, growing as it was laid (`Encase::radius`) [E: its size is not published].
    if let Some(e) = encase_of(content, f) {
        let r = e.radius(p.ecb_height, f.encase_age);
        hurt = [(f.pos + Vec2::new(Fx::ZERO, r), r); HURT_CIRCLES];
    }
    hurt
}

/// How high above the feet a standing fighter's chest is: the centre of its middle hurtbox circle (see [`hurtboxes`]).
pub fn chest_height(p: &FighterParams) -> Fx {
    p.ecb_height * Fx::from_ratio(11, 20)
}

/// The hurtbox circles (bit `i` for circle `i` of [`hurtboxes`]) that the fighter's current move makes intangible this frame
/// (`Move::part_intangible`); 0 when the whole body can be hit.
pub fn intangible_parts(f: &Fighter, w: &Weapon) -> u8 {
    if !matches!(f.state, S::Attack | S::LedgeAttack) {
        return 0;
    }
    let frame = f.state_frame;
    w.get(f.move_id)
        .part_intangible
        .iter()
        .filter(|p| {
            u16::from(p.start) <= frame
                && frame <= u16::from(p.end)
                && moded(p.mode, f.move_powered)
        })
        .fold(0, |bits, p| bits | p.parts)
}

/// Whether a hitbox at `center` of `radius` (a standing capsule `half` above and below it when `half` is above 0: `Hitbox::half_height`)
/// touches one of the hurtbox circles that can be hit (`shut` from [`intangible_parts`]).
fn touches(
    hurt: &[(Vec2, Fx); HURT_CIRCLES],
    shut: u8,
    center: Vec2,
    half_w: Fx,
    half: Fx,
    radius: Fx,
) -> bool {
    hurt.iter().enumerate().any(|(i, (hc, hr))| {
        shut & (1 << i) == 0
            && shapes_touch(*hc, Fx::ZERO, Fx::ZERO, *hr, center, half_w, half, radius)
    })
}

/// A raised shield's bubble (sim 134): its centre (above the feet, moved by the stick's tilt) and its radius (shrinking with the
/// shield's health), as the game draws it.
pub fn shield_bubble(rules: &crate::content::Ruleset, f: &Fighter) -> (Vec2, Fx) {
    let max = shield_max_of(rules, f);
    let hp = if max > Fx::ZERO {
        (f.shield_hp / max).clamp(Fx::ZERO, Fx::ONE)
    } else {
        Fx::ONE
    };
    let scale = rules.shield_min_scale + (Fx::ONE - rules.shield_min_scale) * hp;
    let center = f.pos + Vec2::new(Fx::ZERO, rules.shield_height) + f.shield_tilt;
    (center, rules.shield_radius * scale)
}

/// The most shield health body `f` has: the ruleset's, or (a run's sturdy shield, sim 142) its `shield_scale` percent of it.
pub fn shield_max_of(rules: &crate::content::Ruleset, f: &Fighter) -> Fx {
    if f.shield_scale == 0 {
        rules.shield_max
    } else {
        rules.shield_max * Fx::from_ratio(i32::from(f.shield_scale), 100)
    }
}

/// A shield poke (sim 134, the reference's): a hit on a shielding fighter whose box misses the bubble reaches the body as though no
/// shield were up (the hit is made unblockable). Anything already unblockable, or a fighter not shielding, is left alone.
fn poke(rules: &crate::content::Ruleset, fd: &Fighter, hb: &mut Hitbox, at: Vec2) {
    if fd.state != S::Shield || hb.unblockable {
        return;
    }
    let (c, r) = shield_bubble(rules, fd);
    if !shapes_touch(
        c,
        Fx::ZERO,
        Fx::ZERO,
        r,
        at,
        hb.half_width,
        hb.half_height,
        hb.radius,
    ) {
        hb.unblockable = true;
    }
}

/// True while a fighter cannot be hit.
pub fn is_intangible(f: &Fighter) -> bool {
    !f.active || f.invuln > 0 || f.ledge_invuln > 0
}

/// The move's hitboxes that are active this frame, with their world-space centres.
///
/// `scale` is the fighter's `hitbox_scale`: the returned hitboxes already have their position and radius scaled by it.
pub fn active_hitboxes<'a>(
    f: &'a Fighter,
    mv: &'a Move,
    scale: Fx,
) -> impl Iterator<Item = (usize, Hitbox, Vec2)> + 'a {
    let frame = u8::try_from(f.state_frame).unwrap_or(u8::MAX);
    let facing = i32::from(f.facing);
    mv.hitboxes
        .iter()
        .enumerate()
        .filter(move |(_, hb)| {
            hb.start <= frame
                && frame <= hb.end
                && aimed(hb.aim, f.aim)
                && moded(hb.mode, f.move_powered)
        })
        .map(move |(i, hb)| {
            let mut hb = *hb;
            hb.x = hb.x * scale;
            hb.y = hb.y * scale;
            hb.radius = hb.radius * scale;
            hb.half_height = hb.half_height * scale;
            hb.half_width = hb.half_width * scale;
            hb.posvec_x = hb.posvec_x * scale;
            hb.posvec_y = hb.posvec_y * scale;
            let center = Vec2::new(f.pos.x + hb.x.mul_int(facing), f.pos.y + hb.y);
            (i, hb, center)
        })
}

/// Whether a hitbox with `aim` is out for an attack aimed `aim_of_move` (`Fighter::aim`: 1 up, -1 down, 0 straight).
fn aimed(aim: u8, aim_of_move: i8) -> bool {
    match aim {
        AIM_UP => aim_of_move > 0,
        AIM_DOWN => aim_of_move < 0,
        AIM_STRAIGHT => aim_of_move == 0,
        _ => true,
    }
}

/// Whether a hitbox with `mode` is out in an attack begun with the gauge's powered mode running or not (E52).
fn moded(mode: u8, powered: bool) -> bool {
    match mode {
        MODE_PLAIN => !powered,
        MODE_POWERED => powered,
        _ => true,
    }
}

fn overlaps(c1: Vec2, r1: Fx, c2: Vec2, r2: Fx) -> bool {
    let reach = r1 + r2;
    (c1 - c2).length_sq() <= reach * reach
}

/// Knockback in reference units. `percent` is the defender's percent *after* the hit.
pub fn knockback(percent: Fx, damage: Fx, weight: Fx, base: i16, growth: i16) -> Fx {
    let mut kb = percent * Fx::from_ratio(1, 10) + (percent * damage) * Fx::from_ratio(1, 20);
    kb = kb * (Fx::from_int(200) / (weight + Fx::from_int(100)));
    kb = kb * Fx::from_ratio(7, 5) + Fx::from_int(18);
    kb * Fx::from_ratio(i32::from(growth), 100) + Fx::from_int(i32::from(base))
}

/// Set knockback (a hitbox's `fixed_kb`): the reference's formula with the set value standing in for the victim's percent after the
/// hit and the damage term dropped, so it is the same at any percent: `((fkv * 10 / 20 + 1) * 1.4 * 200 / (weight + 100) + 18) * growth
/// / 100 + base`.
pub fn fixed_knockback(fkv: u16, weight: Fx, base: i16, growth: i16) -> Fx {
    let mut kb = Fx::from_int(i32::from(fkv)) * Fx::from_ratio(10, 20) + Fx::ONE;
    kb = kb * Fx::from_ratio(7, 5) * (Fx::from_int(200) / (weight + Fx::from_int(100)));
    kb += Fx::from_int(18);
    kb * Fx::from_ratio(i32::from(growth), 100) + Fx::from_int(i32::from(base))
}

/// Hitlag in frames for a hit of `damage`: `floor(d * 0.65 + 6)` in the standard rules, times the hitbox's own factor `scale`
/// (its hitlag multiplier, and 1.5 for an electric hit), times 0.67 on a shield, times 0.67 again against a crouch, at most
/// `hitlag_cap`. A hit that deals no damage has none.
pub fn hitlag_frames(damage: Fx, rules: &Ruleset, shield: bool, crouch: bool, scale: Fx) -> u8 {
    hitlag_frames_by(
        damage,
        (rules.hitlag_per_damage, rules.hitlag_base),
        rules,
        shield,
        crouch,
        scale,
    )
}

/// [`hitlag_frames`] with the formula's two numbers given (`per_damage`, `base`): the previous reference version's for a hit on its
/// rules (E40: `Ruleset::prev_hitlag_per_damage` and `prev_hitlag_base`).
pub fn hitlag_frames_by(
    damage: Fx,
    (per_damage, base): (Fx, Fx),
    rules: &Ruleset,
    shield: bool,
    crouch: bool,
    scale: Fx,
) -> u8 {
    if damage <= Fx::ZERO {
        return 0;
    }
    let mut lag = (damage * per_damage + base) * scale;
    if shield {
        lag = lag * rules.shield_hitlag_mult;
    }
    let mut frames = lag.floor_int();
    if crouch {
        frames = (Fx::from_int(frames) * rules.crouch_cancel_hitlag).floor_int();
    }
    frames.clamp(0, i32::from(rules.hitlag_cap)) as u8
}

/// Hitstun in frames: `floor(KB * 0.4 * mult) - 1` (the reference game takes one frame off every hitstun).
pub fn hitstun_frames(kb: Fx, mult: Fx) -> u16 {
    (((kb * mult).mul_int(2) / Fx::from_int(5)).floor_int() - 1).clamp(0, 600) as u16
}

/// How much of the reference game's staleness each queue slot takes off (newest first). A move not in the queue is "fresh" and
/// deals 1.05 times its damage instead.
const STALE_REDUCTORS: [i32; STALE_QUEUE] = [9000, 8545, 7635, 6790, 5945, 5035, 4255, 3345, 2500];
const FRESH_BONUS: i32 = 105_000;

/// The damage multiplier for `move_key` (a move id + 1) given a fighter's queue: 1 minus the reductors of every slot holding that
/// move (a shielded hit counts 0.85 of its slot), or the fresh bonus if it is not there. In hundred-thousandths.
fn stale_factor(queue: &[u8; STALE_QUEUE], move_key: u8) -> i32 {
    let mut cut = 0;
    let mut found = false;
    for (slot, entry) in queue.iter().enumerate() {
        if *entry != 0 && entry & 0x7f == move_key {
            found = true;
            let r = STALE_REDUCTORS[slot];
            cut += if entry & 0x80 != 0 { r * 85 / 100 } else { r };
        }
    }
    if found {
        100_000 - cut
    } else {
        FRESH_BONUS
    }
}

/// Puts a move at the front of the stale queue (the oldest drops off).
fn push_stale(queue: &mut [u8; STALE_QUEUE], move_key: u8, shielded: bool) {
    queue.rotate_right(1);
    queue[0] = move_key | if shielded { 0x80 } else { 0 };
}

/// Rage: the attacker's percent raises knockback linearly from `rage_start` to `rage_full`, by up to `rage_max`.
pub fn rage_multiplier(attacker_percent: Fx, rules: &Ruleset) -> Fx {
    rage_multiplier_up_to(attacker_percent, rules, rules.rage_max)
}

/// [`rage_multiplier`] reaching `max` instead of the ruleset's `rage_max` (the previous reference version's 0.15 for a hit on its
/// rules: E40).
pub fn rage_multiplier_up_to(attacker_percent: Fx, rules: &Ruleset, max: Fx) -> Fx {
    let span = rules.rage_full - rules.rage_start;
    if span <= Fx::ZERO {
        return Fx::ONE;
    }
    let t = ((attacker_percent - rules.rage_start) / span).clamp(Fx::ZERO, Fx::ONE);
    Fx::ONE + t * max
}

/// World launch angle: mirrored when the attacker faces left, with the 361 angle resolved.
pub fn launch_angle(degrees: i16, attacker_facing: i8, defender_grounded: bool, kb: Fx) -> Angle {
    // The reference's rules (ssbwiki.com, the 361 angle's page): against a grounded fighter 0 degrees up to 60 knockback, rising evenly to 38
    // at 88 and 38 above; against an airborne one always 38. A move aimed straight sideways (0) rises to 32 from 120 knockback.
    let mut d = i32::from(degrees);
    if d == 361 {
        d = if !defender_grounded || kb >= Fx::from_int(88) {
            38
        } else if kb <= Fx::from_int(60) {
            0
        } else {
            ((kb - Fx::from_int(60)).mul_int(38) / Fx::from_int(28)).floor_int()
        };
    } else if d == 0 && kb >= Fx::from_int(120) {
        d = 32;
    }
    if attacker_facing < 0 {
        d = 180 - d;
    }
    Angle::from_degrees(d)
}

/// The side (1 right, -1 left) a hit launches toward at 0 degrees, from its angle type (the reference's): `by_box` is the default
/// (the side of the attacker the hitbox is on, or a projectile's direction of travel), `forward` the way the attacker faces (a
/// projectile's travel), `travel` the way the attacker is moving (the sign of its horizontal speed; for a projectile its travel), for
/// "by travel", and `source_x` / `victim_x` where the attacker (or projectile) and the victim are, for "by position".
pub fn launch_side(
    angle_type: u8,
    by_box: i8,
    forward: i8,
    travel: i8,
    source_x: Fx,
    victim_x: Fx,
) -> i8 {
    match angle_type {
        ANGLE_FORWARD => forward,
        ANGLE_BACKWARD => -forward,
        ANGLE_BY_TRAVEL => travel,
        ANGLE_BY_POSITION => {
            if victim_x > source_x {
                1
            } else if victim_x < source_x {
                -1
            } else {
                forward
            }
        }
        _ => by_box,
    }
}

/// Whether bodies `a` and `b` can hit, grab, clank with and reflect one another: any two players (versus), a player and a grunt,
/// but never two grunts (sim 131: the run's enemies are one side). A hazard of the run hits from an unplayed player slot, so it
/// reaches grunts too.
pub fn rivals(a: usize, b: usize) -> bool {
    a != b && (a < MAX_FIGHTERS || b < MAX_FIGHTERS)
}

/// The bit that records "this attacker already hit fighter `d` with hitbox group `g`" (eight groups).
pub fn hit_bit(d: usize, group: u8) -> u128 {
    1 << (d + MAX_BODIES * usize::from(group.min(7)))
}

/// The bits of every fighter for the hit groups in `groups` (bit `g` for group `g`): what a multi-hit's rehit clears.
pub fn group_bits(groups: u8) -> u128 {
    let mut bits = 0;
    for g in 0..8u8 {
        if groups & (1 << g) != 0 {
            for d in 0..MAX_BODIES {
                bits |= hit_bit(d, g);
            }
        }
    }
    bits
}

/// Whether a hitbox can hit this fighter at all: a grounded-only one misses a fighter in the air and an aerial-only one a fighter on
/// the ground.
pub(crate) fn hits_target(hb: &Hitbox, target: &Fighter) -> bool {
    match hb.targets {
        TARGETS_GROUNDED => target.grounded(),
        TARGETS_AERIAL => !target.grounded(),
        _ => true,
    }
}

/// Finds and applies this frame's hits. Everything is collected before anything changes, so a trade
/// (both fighters hitting each other on the same frame) works, and ties go to the lower player index.
///
/// A move hits each target at most once per hitbox *group*; groups are how a move has separate hits.
/// The loops need the attacker and defender indices themselves (hit masks, tie-breaking), not just items.
#[allow(clippy::needless_range_loop)]
pub fn resolve_hits(state: &mut GameState, content: &Content) {
    // (Sized by the bodies in play: the players in versus, every body in a run. Versus pays nothing for the grunt slots.)
    if state.run.active {
        resolve_hits_n::<MAX_BODIES>(state, content);
    } else {
        resolve_hits_n::<MAX_FIGHTERS>(state, content);
    }
}

#[allow(clippy::needless_range_loop)]
fn resolve_hits_n<const N: usize>(state: &mut GameState, content: &Content) {
    let mut chosen: [[Option<Hitbox>; N]; N] = [[None; N]; N];
    // Where each chosen hitbox was (for an autolink's pull).
    let mut chosen_at = [[Vec2::ZERO; N]; N];
    // Whether each chosen hit reached the defender's open counter (its bubble, or its body when it has none).
    let mut caught = [[false; N]; N];
    // Attackers whose search box touched someone this frame (see `HIT_SEARCH`).
    let mut searched = [false; N];
    // Hits that reached a defender's balloons but not its body (E41, sim 113).
    let mut on_balloons: [[Option<Hitbox>; N]; N] = [[None; N]; N];
    // The best box of each attacker's partner (part 1, E52) on each defender: the box, where it was, whether a counter catches it.
    let mut partner: [[Option<(Hitbox, Vec2, bool)>; N]; N] = [[None; N]; N];
    // The groups of each attacker's blockable grab boxes (E56) a shield or a reflector stopped this frame.
    let mut stopped = [0u8; N];
    for a in 0..N {
        let fa = &state.fighters[a];
        if !matches!(fa.state, S::Attack | S::LedgeAttack) || fa.hitlag > 0 {
            continue;
        }
        let mv = weapon_of(content, params_of(content, fa)).get(fa.move_id);
        // (A blockable grab box touching someone's reflector field is stopped by it: E56.)
        for (_, hb, center) in active_hitboxes(fa, mv, params_of(content, fa).hitbox_scale) {
            if hb.kind == HIT_GRAB && hb.blockable {
                let shape = Shape {
                    half_w: hb.half_width,
                    half_h: hb.half_height,
                    radius: hb.radius,
                };
                if reflector_touching(state, content, a, center, shape).is_some() {
                    stopped[a] |= 1 << hb.group;
                }
            }
        }
        for d in 0..N {
            if !rivals(a, d) {
                continue;
            }
            let fd = &state.fighters[d];
            let counter = counter_open(content, fd, true);
            let intangible = is_intangible(fd);
            if intangible && counter.is_none() {
                continue;
            }
            let pd = params_of(content, fd);
            let wd = weapon_of(content, pd);
            let hurt = body_hurtboxes(content, fd);
            let shut = intangible_parts(fd, wd);
            let mut best: Option<Hitbox> = None;
            for (_, hb, center) in active_hitboxes(fa, mv, params_of(content, fa).hitbox_scale) {
                if fa.hit_mask & hit_bit(d, hb.group) != 0 || !hits_target(&hb, fd) {
                    continue;
                }
                if hb.kind == HIT_SEARCH {
                    // A search box only notices (shields do not matter); it never competes with the move's hits.
                    if fa.grab_with != d as i8
                        && touches(
                            &hurt,
                            shut,
                            center,
                            hb.half_width,
                            hb.half_height,
                            hb.radius,
                        )
                    {
                        searched[a] = true;
                    }
                    continue;
                }
                let (touching, catches) = match hb.kind {
                    // Throws and pummels act on the fighter being held, wherever the hitbox is.
                    HIT_THROW | HIT_PUMMEL => (fa.grab_with == d as i8, false),
                    // (Only hits reach through a counter's intangibility; a grab does not. A blockable grab box touching a raised
                    // shield is stopped by it: E56.)
                    HIT_GRAB => {
                        let touch = !intangible
                            && touches(
                                &hurt,
                                shut,
                                center,
                                hb.half_width,
                                hb.half_height,
                                hb.radius,
                            );
                        if touch && hb.blockable && fd.state == S::Shield {
                            stopped[a] |= 1 << hb.group;
                            (false, false)
                        } else {
                            (touch && grab::can_catch(&hb, fd), false)
                        }
                    }
                    _ if fa.grab_with == d as i8 => (false, false),
                    _ => reach(counter, intangible, &hb, &hurt, shut, center),
                };
                // (A partner's box competes only with its partner's other boxes: E52.)
                if hb.part != 0 {
                    if touching
                        && hb.kind == HIT_NORMAL
                        && partner[a][d].is_none_or(|(b, _, _)| hb.priority < b.priority)
                    {
                        partner[a][d] = Some((hb, center, catches));
                    }
                    continue;
                }
                if touching && best.is_none_or(|b| hb.priority < b.priority) {
                    best = Some(hb);
                    chosen_at[a][d] = center;
                    caught[a][d] = catches;
                }
                // (A hit that misses the body may still reach its balloons: E41.)
                if !touching
                    && hb.kind == HIT_NORMAL
                    && !hb.push
                    && fd.balloons > 0
                    && on_balloons[a][d].is_none_or(|b| hb.priority < b.priority)
                    && crate::balloons::touches(
                        content,
                        fd,
                        center,
                        hb.half_width,
                        hb.half_height,
                        hb.radius,
                    )
                {
                    on_balloons[a][d] = Some(hb);
                }
            }
            chosen[a][d] = best;
        }
    }

    // A stopped blockable grab (E56) catches no one with those groups for the rest of the move, and a move with a `shield_end` ends
    // that many frames later (sim 126, owner decision 84), unless it ends sooner anyway.
    for a in 0..N {
        if stopped[a] == 0 {
            continue;
        }
        let fa = &state.fighters[a];
        let end = weapon_of(content, params_of(content, fa))
            .get(fa.move_id)
            .shield_end;
        let fa = &mut state.fighters[a];
        if end > 0 {
            let at = fa.state_frame.saturating_add(u16::from(end));
            if fa.move_end == 0 || at < fa.move_end {
                fa.move_end = at;
            }
        }
        fa.hit_mask |= group_bits(stopped[a]);
        for c in chosen[a].iter_mut() {
            if c.is_some_and(|h| h.kind == HIT_GRAB && stopped[a] & (1 << h.group) != 0) {
                *c = None;
            }
        }
    }

    clank_attacks(state, content, &mut chosen);

    // Balloons (E41): a hit that reached only a defender's balloons is used up on them (the move does not hit that fighter again),
    // unless the move also hit the body this frame.
    for a in 0..N {
        for d in 0..N {
            if let (None, Some(hb)) = (chosen[a][d], on_balloons[a][d]) {
                state.fighters[a].hit_mask |= hit_bit(d, hb.group);
                crate::balloons::hit(state, content, d, hb.damage);
            }
        }
    }

    let mut struck = [false; N];
    // Who struck each fighter this frame (a partner's box may land on top of its own fighter's hit: E52).
    let mut struck_by = [usize::MAX; N];
    for a in 0..N {
        for d in 0..N {
            if let Some(hb) = chosen[a][d] {
                if hb.kind == HIT_GRAB {
                    continue; // grabs are settled after every damaging hit this frame
                }
                if !struck[d] {
                    struck[d] = true;
                    struck_by[d] = a;
                    strike(state, content, a, d, hb, chosen_at[a][d], caught[a][d]);
                }
            }
        }
    }
    // The partner's boxes (E52, part 1): a fighter one touches takes its hit as well, after its own fighter's (both damages, the
    // partner's launch applied last [E]), unless another attacker struck it this frame, its own fighter's hit was caught by a counter
    // or the attacker is no longer attacking (a clank's rebound).
    for a in 0..N {
        for d in 0..N {
            let Some((hb, at, catches)) = partner[a][d] else {
                continue;
            };
            let countered = chosen[a][d].is_some() && caught[a][d];
            if (struck[d] && struck_by[d] != a) || countered || state.fighters[a].state != S::Attack
            {
                continue;
            }
            struck[d] = true;
            struck_by[d] = a;
            strike(state, content, a, d, hb, at, catches);
        }
    }

    // Two fighters grabbing each other on the same frame both let go: a grab parry (1% each, and a rebound).
    for a in 0..N {
        for d in a + 1..N {
            let both = matches!(chosen[a][d], Some(h) if h.kind == HIT_GRAB && !h.command)
                && matches!(chosen[d][a], Some(h) if h.kind == HIT_GRAB && !h.command);
            if both && !struck[a] && !struck[d] {
                chosen[a][d] = None;
                chosen[d][a] = None;
                for (i, other) in [(a, d), (d, a)] {
                    let f = &mut state.fighters[i];
                    let before = f.percent;
                    f.percent = (f.percent + content.rules.damage_mult).min(Fx::from_int(999));
                    start_rebound(f, content.rules.grab_parry_lag, 0);
                    let added = state.fighters[i].percent - before;
                    crate::gauge::on_damage(state, content, other, i, added);
                }
            }
        }
    }

    // Grabs last: a fighter that was hit this frame, or whose grabber was, is not caught. Ties go to the
    // lower player index.
    for a in 0..N {
        for d in 0..N {
            let Some(hb) = chosen[a][d] else {
                continue;
            };
            if hb.kind != HIT_GRAB
                || struck[a]
                || struck[d]
                || state.fighters[a].state != S::Attack
                || state.fighters[a].grab_with != NONE
                || !grab::can_catch(&hb, &state.fighters[d])
            {
                continue;
            }
            state.fighters[a].hit_mask |= hit_bit(d, hb.group);
            struck[d] = true;
            grab::connect(state, content, a, d);
        }
    }

    // Search boxes last: an attacker that touched someone with one (and was not hit or knocked out of its move meanwhile) switches
    // to the move's `on_search` at the start of the next frame.
    for a in 0..N {
        let f = &state.fighters[a];
        if !searched[a] || struck[a] || f.state != S::Attack || f.next_move != 0 {
            continue;
        }
        if let Some(next) = weapon_of(content, params_of(content, f))
            .get(f.move_id)
            .on_search
        {
            state.fighters[a].next_move = next.saturating_add(1);
        }
    }
}

/// Lands attacker `a`'s chosen hit `hb` (centred `at`; `caught` by an open counter) on fighter `d`: the launch's side, a charged
/// smash's or a counter answer's damage, then a pummel, a throw or an ordinary hit.
fn strike(
    state: &mut GameState,
    content: &Content,
    a: usize,
    d: usize,
    hb: Hitbox,
    at: Vec2,
    caught: bool,
) {
    let attacker = state.fighters[a];
    state.fighters[a].hit_mask |= hit_bit(d, hb.group);
    // Which way "forward" is for the launch: by default a hitbox behind the attacker (back air) sends the victim backward, away from
    // it; the hit's angle type can say otherwise.
    let by_box = if hb.x < Fx::ZERO {
        -attacker.facing
    } else {
        attacker.facing
    };
    // The way the attacker is moving (the reference's speed-based type 1); not moving sideways, its facing [E].
    let travel = match attacker.vel.x.raw().signum() {
        0 => attacker.facing,
        s => s as i8,
    };
    let facing = launch_side(
        hb.angle_type,
        by_box,
        attacker.facing,
        travel,
        attacker.pos.x,
        state.fighters[d].pos.x,
    );
    // A charged smash attack hits harder; the damage also feeds the knockback.
    let mut hb = hb;
    let attack = weapon_of(content, params_of(content, &attacker)).get(attacker.move_id);
    if let Some(add) = attack.charge_add {
        // Additive charge damage (E29): the reference's charged punch.
        hb.damage = add.damage(hb.damage, attacker.charge);
    } else if attacker.charge > 0 {
        let rules = &content.rules;
        let full = if attack.charge_bonus > 0 {
            attack.charge_bonus
        } else {
            rules.charge_bonus_percent
        };
        let bonus = Fx::from_ratio(
            i32::from(attacker.charge) * i32::from(full),
            100 * i32::from(rules.charge_frames.max(1)),
        );
        hb.damage = hb.damage * (Fx::ONE + bonus);
    }
    // The answer to a caught hit deals what was caught.
    if attack.counter_strike {
        hb.damage = attacker.counter_damage;
    }
    match hb.kind {
        HIT_PUMMEL => pummel(state, content, a, d, &hb),
        HIT_THROW => {
            // The throw lets go: the held fighter is launched like any hit victim.
            state.fighters[a].grab_with = NONE;
            fighter::free_held(&mut state.fighters[d], 0);
            apply_hit(
                state,
                content,
                a,
                d,
                &hb,
                facing,
                attacker.pos,
                true,
                STUN_NORMAL,
                attacker.move_id,
                (at, attacker.vel),
                false,
                true,
            );
        }
        _ => {
            let mult = stun_multiplier(&attacker);
            poke(&content.rules, &state.fighters[d], &mut hb, at);
            apply_hit(
                state,
                content,
                a,
                d,
                &hb,
                facing,
                attacker.pos,
                true,
                mult,
                attacker.move_id,
                (at, attacker.vel),
                caught,
                true,
            );
        }
    }
}

/// Whether a fighter's current attack can clank: a grounded attack (aerials and ledge attacks never do), not frozen.
fn clanks(f: &Fighter) -> bool {
    f.state == S::Attack && f.grounded() && f.hitlag == 0
}

/// Where two attacks meet: the strongest damaging hitbox of `a` that touches one of `b`'s, and `b`'s strongest that touches one of
/// `a`'s, and whether each side has a box among them that rebounds (`Hitbox::rebounds`), if any touch.
fn clash(content: &Content, a: &Fighter, b: &Fighter) -> Option<(Fx, Fx, bool, bool)> {
    let (pa, pb) = (params_of(content, a), params_of(content, b));
    let (ma, mb) = (
        weapon_of(content, pa).get(a.move_id),
        weapon_of(content, pb).get(b.move_id),
    );
    let mut best: Option<(Fx, Fx, bool, bool)> = None;
    for (_, ha, ca) in active_hitboxes(a, ma, pa.hitbox_scale) {
        if ha.kind != HIT_NORMAL || !ha.clanks {
            continue;
        }
        for (_, hb, cb) in active_hitboxes(b, mb, pb.hitbox_scale) {
            if hb.kind != HIT_NORMAL || !hb.clanks || !overlaps(ca, ha.radius, cb, hb.radius) {
                continue;
            }
            let (da, db, ra, rb) = best.unwrap_or((Fx::ZERO, Fx::ZERO, false, false));
            best = Some((
                da.max(ha.damage),
                db.max(hb.damage),
                ra || ha.rebounds,
                rb || hb.rebounds,
            ));
        }
    }
    best
}

/// Rebound length for a clank whose stronger hit dealt `damage`: `floor((d + 4) * 15 / 8)`, at most `rebound_cap`.
pub fn rebound_frames(damage: Fx, rules: &Ruleset) -> u8 {
    ((damage + Fx::from_int(4)) * Fx::from_ratio(15, 8))
        .floor_int()
        .clamp(1, i32::from(rules.rebound_cap)) as u8
}

/// Knocks a fighter out of its attack into a rebound: frozen for `freeze` frames, then `lag` frames of nothing.
fn start_rebound(f: &mut Fighter, lag: u8, freeze: u8) {
    f.state = S::Rebound;
    f.state_frame = 0;
    f.lag = lag;
    f.hitlag = freeze;
    f.charge = 0;
    f.grab_with = NONE;
}

/// Clanks: when two grounded attacks' hitboxes meet, neither hits the other this frame. Within `clank_range` percent both attacks
/// are cancelled into a rebound; otherwise only the weaker one is, and the stronger carries on (it can still hit on a later frame).
/// The rebound lasts according to the stronger hit, after being frozen for its hitlag.
#[allow(clippy::needless_range_loop)]
fn clank_attacks<const N: usize>(
    state: &mut GameState,
    content: &Content,
    chosen: &mut [[Option<Hitbox>; N]; N],
) {
    let rules = &content.rules;
    for a in 0..N {
        for b in a + 1..N {
            let (fa, fb) = (state.fighters[a], state.fighters[b]);
            if !rivals(a, b) || !fa.active || !fb.active || !clanks(&fa) || !clanks(&fb) {
                continue;
            }
            let Some((da, db, rebound_a, rebound_b)) = clash(content, &fa, &fb) else {
                continue;
            };
            chosen[a][b] = None;
            chosen[b][a] = None;
            let stronger = da.max(db);
            let lag = rebound_frames(stronger, rules);
            let freeze = hitlag_frames(stronger, rules, false, false, Fx::ONE);
            let both = (da - db).abs() <= rules.clank_range;
            // (A side whose clanking hits all cannot rebound is only stopped from hitting this frame: it carries on with its move.)
            if (both || da < db) && rebound_a {
                start_rebound(&mut state.fighters[a], lag, freeze);
            }
            if (both || db < da) && rebound_b {
                start_rebound(&mut state.fighters[b], lag, freeze);
            }
        }
    }
}

/// A projectile meeting an attack: a grounded attack more than `clank_range` stronger destroys it and carries on; within the range
/// both stop (the attacker rebounds); a projectile that much stronger knocks the attacker into a rebound and flies on. An aerial is
/// only frozen for a moment and destroys the projectile unless the projectile is that much stronger. Returns true if the
/// projectile is destroyed.
fn clank_projectile(
    state: &mut GameState,
    content: &Content,
    owner: usize,
    pos: Vec2,
    radius: Fx,
    damage: Fx,
) -> bool {
    let rules = &content.rules;
    for i in 0..crate::state::bodies(state) {
        if !rivals(owner, i) {
            continue;
        }
        let f = state.fighters[i];
        if !f.active || f.state != S::Attack || f.hitlag > 0 {
            continue;
        }
        let p = params_of(content, &f);
        let mv = weapon_of(content, p).get(f.move_id);
        let mut touching: Option<Fx> = None;
        let mut rebounds = false;
        for (_, hb, c) in active_hitboxes(&f, mv, p.hitbox_scale) {
            if hb.kind == HIT_NORMAL && hb.clanks && overlaps(c, hb.radius, pos, radius) {
                touching = Some(touching.map_or(hb.damage, |d: Fx| d.max(hb.damage)));
                rebounds |= hb.rebounds;
            }
        }
        let Some(attack) = touching else {
            continue;
        };
        let stronger = attack.max(damage);
        let freeze = hitlag_frames(stronger, rules, false, false, Fx::ONE);
        let projectile_wins = damage - attack > rules.clank_range;
        if !f.grounded() {
            state.fighters[i].hitlag = freeze;
            return !projectile_wins;
        }
        if attack - damage > rules.clank_range {
            return true;
        }
        if !rebounds {
            // An attack that cannot rebound is not knocked out of its move by the projectile (the reference's `rebound=f`).
            return !projectile_wins;
        }
        start_rebound(
            &mut state.fighters[i],
            rebound_frames(stronger, rules),
            freeze,
        );
        return !projectile_wins;
    }
    false
}

/// The share of its hits' damage body `f` deals: all of it, except a grunt's (`Fighter::damage_scale`, its tier's).
pub fn dealt_scale(f: &Fighter) -> Fx {
    if f.damage_scale == 0 {
        Fx::ONE
    } else {
        Fx::from_ratio(i32::from(f.damage_scale), 100)
    }
}

/// A pummel: damage to the held fighter, a little hitlag for both, and it stays held.
fn pummel(state: &mut GameState, content: &Content, a: usize, d: usize, hb: &Hitbox) {
    // (With the hit's own hitlag multiplier, as any hit.)
    let scale = Fx::from_ratio(i32::from(hb.hitlag), 100);
    let hitlag = hitlag_frames(hb.damage, &content.rules, false, false, scale);
    state.fighters[a].hitlag = hitlag;
    // (A wing's power counts on a pummel too: E57.)
    let wing = match params_of(content, &state.fighters[a]).wing {
        Some(w) if crate::wing::out(&state.fighters[a]) => Fx::from_ratio(i32::from(w.power), 100),
        _ => Fx::ONE,
    } * dealt_scale(&state.fighters[a]);
    let held = &mut state.fighters[d];
    let before = held.percent;
    held.percent =
        (held.percent + hb.damage * wing * content.rules.damage_mult).min(Fx::from_int(999));
    held.hitlag = hitlag;
    let added = held.percent - before;
    crate::gauge::on_damage(state, content, a, d, added);
    if added > Fx::ZERO {
        crate::wing::on_hurt(state, content, a, d, before);
        crate::wing::on_dealt(state, content, a, d, hb.damage);
    }
}

/// Where an open counter catches: on the body's hurtboxes (a counter without a bubble), or on its bubble, a standing capsule
/// (centre, half height, radius) in world space.
#[derive(Clone, Copy)]
pub(crate) enum CatchArea {
    Body,
    Bubble(Vec2, Fx, Fx),
}

/// If `f` is in a counter stance whose window is open for a hit of this kind (`melee`, or a projectile's), where it catches. Such a
/// hit reaches the counter even while the stance is intangible (the reference's counters catch through their own intangibility).
pub(crate) fn counter_open(content: &Content, f: &Fighter, melee: bool) -> Option<CatchArea> {
    if !f.active || f.state != S::Attack {
        return None;
    }
    let p = params_of(content, f);
    let c = weapon_of(content, p).get(f.move_id).counter?;
    let open = u16::from(c.start) <= f.state_frame
        && f.state_frame <= u16::from(c.end)
        && (melee || !c.melee_only);
    if !open {
        return None;
    }
    Some(match c.bubble {
        None => CatchArea::Body,
        Some(b) => {
            let scale = p.hitbox_scale;
            let centre = Vec2::new(
                f.pos.x + (b.x * scale).mul_int(i32::from(f.facing)),
                f.pos.y + b.y * scale,
            );
            CatchArea::Bubble(centre, b.half_height * scale, b.radius * scale)
        }
    })
}

/// Whether hit `hb` at `center` reaches a fighter (`hurt` and `shut` its hurtboxes, `counter` its open counter if any), and whether
/// the counter catches it. A hit touching the bubble (or, for a counter without one, the body) is caught; a push never is (the
/// reference's counters ignore windboxes) and, like any hit the counter does not catch, needs a tangible body to land.
pub(crate) fn reach(
    counter: Option<CatchArea>,
    intangible: bool,
    hb: &Hitbox,
    hurt: &[(Vec2, Fx); HURT_CIRCLES],
    shut: u8,
    center: Vec2,
) -> (bool, bool) {
    let body = || touches(hurt, shut, center, hb.half_width, hb.half_height, hb.radius);
    let caught = match counter {
        _ if hb.push => false,
        None => false,
        Some(CatchArea::Body) => body(),
        Some(CatchArea::Bubble(c, half, r)) => shapes_touch(
            center,
            hb.half_width,
            hb.half_height,
            hb.radius,
            c,
            Fx::ZERO,
            half,
            r,
        ),
    };
    (caught || (!intangible && body()), caught)
}

/// If fighter `d` is in a counter stance whose window is open, cancels the hit and switches `d` to its answer.
fn catch_with_counter(
    state: &mut GameState,
    content: &Content,
    source: usize,
    d: usize,
    damage: Fx,
    hitlag: u8,
    melee: bool,
) -> bool {
    let def = &state.fighters[d];
    if def.state != S::Attack {
        return false;
    }
    let weapon = weapon_of(content, params_of(content, def));
    let Some(c) = weapon.get(def.move_id).counter else {
        return false;
    };
    if def.state_frame < u16::from(c.start) || def.state_frame > u16::from(c.end) {
        return false;
    }
    // A melee-only counter leaves projectiles alone (they are a reflector's business).
    if c.melee_only && !melee {
        return false;
    }
    // A hit too strong for the stance breaks it (sim 128: the reference's barrier): the hit deals nothing, the fighter staggers.
    if c.break_above > Fx::ZERO && damage > c.break_above {
        let def = &mut state.fighters[d];
        fighter::begin_attack(def, MoveId::from_index(c.break_move));
        def.hitlag = hitlag;
        return true;
    }
    let answer_id = MoveId::from_index(c.then);
    let intangible = weapon.get(c.then).intangible;
    let mut caught =
        (damage * Fx::from_int(i32::from(c.percent)) / Fx::from_int(100)).max(c.min_damage);
    if c.max_damage > Fx::ZERO {
        caught = caught.min(c.max_damage);
    }
    let attacker_x = state.fighters[source].pos.x;
    let def = &mut state.fighters[d];
    // The answer is swung at whoever struck.
    if attacker_x != def.pos.x {
        def.facing = if attacker_x > def.pos.x { 1 } else { -1 };
    }
    def.counter_damage = caught;
    fighter::begin_attack(def, answer_id);
    def.invuln = def.invuln.max(intangible);
    // (A still counter answers at once, the attacker alone frozen: sim 128.)
    def.hitlag = if c.still { 0 } else { hitlag };
    true
}

/// Applies one hit: damage, knockback, hitlag and the victim's state change. `source` is the fighter
/// responsible (the attacker, or a projectile's owner). Only a melee attacker is frozen by hitlag. `move_id` is the source's move
/// that hit, for stale-move negation. `link` is where the hitbox was and how fast its owner was moving (for an autolink angle).
/// `caught`: the hit reached the defender's open counter (see [`reach`]), which takes it instead.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_hit(
    state: &mut GameState,
    content: &Content,
    source: usize,
    d: usize,
    hb: &Hitbox,
    facing: i8,
    source_pos: Vec2,
    freeze_source: bool,
    stun_mult: u16,
    move_id: u8,
    link: (Vec2, Vec2),
    caught: bool,
    wing_power: bool,
) {
    // (A run's bubble, R3, sim 142: the player's next hit is absorbed, unless a raised shield or a counter takes it. Never in
    // versus: a match has no active run.)
    if state.run.active
        && d == crate::run::PLAYER
        && !caught
        && state.run.slot == crate::run::data::ITEM_BUBBLE
        && !(state.fighters[d].state == S::Shield && !hb.unblockable)
    {
        crate::run::items::pop_bubble(state);
        return;
    }
    let rules = &content.rules;
    // Stale-move negation: a move used again and again deals less damage (and a little less knockback); a fresh one a little more.
    let move_key = (move_id & 0x7f).saturating_add(1).min(0x7f);
    let stale = if rules.stale_moves != 0 {
        Fx::from_ratio(
            stale_factor(&state.fighters[source].stale, move_key),
            100_000,
        )
    } else {
        Fx::ONE
    };
    // (A grunt deals its tier's share of its moves' damage, knockback term included, as its body's hits do in R2b: sim 131.)
    let mut hb_in = *hb;
    hb_in.damage = hb_in.damage * dealt_scale(&state.fighters[source]);
    let mut hb = hb_in;
    // An aerial made during a short hop hits a little softer.
    let attacker = &state.fighters[source];
    let short_hop_aerial = freeze_source
        && attacker.short_hop
        && attacker.state == S::Attack
        && MoveId::from_index(attacker.move_id).is_aerial();
    let base_damage = if short_hop_aerial {
        hb.damage * rules.short_hop_damage
    } else {
        hb.damage
    };
    // A wing's power (E57, sim 125): the fighter's hits (not its items') deal more while its wing is out; the knockback formula's
    // damage term below is the move's own, so the move KOs earlier only by the extra damage (the reference's rule).
    let wing = if wing_power && crate::wing::out(attacker) {
        params_of(content, attacker)
            .wing
            .map_or(Fx::ONE, |w| Fx::from_ratio(i32::from(w.power), 100))
    } else {
        Fx::ONE
    };
    hb.damage = base_damage * stale * wing;
    // The knockback formula's damage term is the move's own damage, staleness counted at 0.3: the short hop's 0.85 is a damage
    // dealt multiplier and leaves it alone (the reference; rubendal's SSBU-Calculator, sim 97).
    let kb_damage = hb_in.damage * (Fx::ONE + (stale - Fx::ONE) * Fx::from_ratio(3, 10));

    // (An unblockable hit lands on a raised shield as on a fighter without one: sim 119.)
    let shielding = state.fighters[d].state == S::Shield && !hb.unblockable;
    let crouching = state.fighters[d].state == S::Crouch && state.fighters[d].grounded();
    // A hit on the previous reference version's rules (E40, owner decisions 66-67): its damage multiplier, hitlag, rage, hitstun and
    // launch (below).
    let older = hb.rules == RULES_PREVIOUS;
    let damage_mult = if older {
        rules.prev_damage_mult
    } else {
        rules.damage_mult
    };
    let lag_formula = if older {
        (rules.prev_hitlag_per_damage, rules.prev_hitlag_base)
    } else {
        (rules.hitlag_per_damage, rules.hitlag_base)
    };
    // The hitbox's own hitlag factor (ignored on a shield when it would shorten it) and the electric effect.
    let mut scale = Fx::from_ratio(i32::from(hb.hitlag), 100);
    if shielding && scale < Fx::ONE {
        scale = Fx::ONE;
    }
    let electric = hb.effect == EFFECT_ELECTRIC;
    if electric {
        scale = scale * rules.electric_hitlag_mult;
    }
    // (A push freezes nobody.)
    let hitlag = if hb.push {
        0
    } else {
        hitlag_frames_by(hb.damage, lag_formula, rules, shielding, crouching, scale)
    };
    // (The previous version shortened only the victim's hitlag against a crouch; the attacker's is the uncrouched one.)
    let source_hitlag = if older && crouching && !hb.push {
        hitlag_frames_by(hb.damage, lag_formula, rules, shielding, false, scale)
    } else {
        hitlag
    };
    // A fighter in a counter stance catches the hit instead of taking it.
    if caught && catch_with_counter(state, content, source, d, hb.damage, hitlag, freeze_source) {
        if freeze_source {
            state.fighters[source].hitlag = hitlag;
        }
        return;
    }
    // A fighter that is hit lets go of, or is let go by, whoever it was holding or held by. (A carrying fighter only once its carry's
    // armor has been seen not to withstand the hit, below.)
    let carrying = state.fighters[d].state == S::Carrying;
    if !carrying {
        grab::drop_grab(state, content, d);
    }
    if freeze_source {
        // Hitting several fighters on one frame freezes the attacker for the longest of their hitlags.
        state.fighters[source].hitlag = state.fighters[source].hitlag.max(source_hitlag);
    }
    if rules.stale_moves != 0 && hb.damage > Fx::ZERO {
        push_stale(&mut state.fighters[source].stale, move_key, shielding);
    }

    let rage = rage_multiplier_up_to(
        state.fighters[source].percent,
        rules,
        if older {
            rules.prev_rage_max
        } else {
            rules.rage_max
        },
    );
    let puller = state.fighters[source].pos;
    // A position vector's point (E34): ahead of and above the attacker's feet, by its facing, where it hit.
    let posvec_point = puller
        + Vec2::new(
            hb.posvec_x
                .mul_int(i32::from(state.fighters[source].facing)),
            hb.posvec_y,
        );
    let source_stocks = state.fighters[source].stocks;
    let bury_hit = hb.effect == EFFECT_BURY;
    // The egg (E8): a fighter already in one takes less damage and is never launched (below); a throw row with the encase effect lays
    // one with its move's numbers.
    let egg = encase_of(content, &state.fighters[d]);
    let new_egg = if hb.effect == EFFECT_ENCASE && egg.is_none() {
        let attacker = &state.fighters[source];
        weapon_of(content, params_of(content, attacker))
            .get(move_id)
            .encase
            .map(|e| (e, attacker.char_id, attacker.facing))
    } else {
        None
    };
    if let Some(e) = egg {
        hb.damage = hb.damage * Fx::from_ratio(i32::from(e.damage_taken), 100);
    }
    let mut defender_params = *params_of(content, &state.fighters[d]);
    // (A grunt weighs its kind's and tier's weight, not its body's: sim 131.)
    if let Some(w) = state.fighters[d].weight {
        defender_params.weight = w;
    }
    let def = &mut state.fighters[d];
    if shielding {
        block(def, rules, &hb, facing, hitlag, stun_mult, damage_mult);
        if hb.push {
            // A push shoves the shield back (its horizontal part).
            let kb = fixed_or_plain_kb(&hb, def.percent, kb_damage, defender_params.weight, rage);
            let angle = launch_angle(hb.angle, facing, true, kb);
            def.vel.x += crate::trig::cos(angle) * kb * Fx::from_ratio(3, 800);
        }
        return;
    }
    if def.state == S::ShieldBreak {
        // Hit while stunned: the stun ends and the shield comes back.
        fighter::restore_shield(def, rules);
    }
    // A move whose armor covers this frame may take only part of the hit's damage (E55, sim 121: the reference's guard, 0.4).
    if def.state == S::Attack && !hb.push {
        let mv = weapon_of(content, &defender_params).get(def.move_id);
        if mv.armor_damage != 1000
            && moded(mv.armor_mode, def.move_powered)
            && mv.armor.is_some_and(|a| a.covers(def.state_frame))
        {
            hb.damage = hb.damage * Fx::from_ratio(i32::from(mv.armor_damage), 1000);
        }
    }

    let before = def.percent;
    def.percent = (def.percent + hb.damage * damage_mult).min(Fx::from_int(999));
    let added = def.percent - before;
    // The gauges (the damage actually added: owner decision 32). The victim is still in the move it was hit in, so a hit during a
    // draining move drains.
    crate::gauge::on_damage(state, content, source, d, added);
    // The wing (E57): the KO credit and the wing's start on the victim's side, the points the attacker's wing spends.
    if added > Fx::ZERO {
        crate::wing::on_hurt(state, content, source, d, before);
        crate::wing::on_dealt(state, content, source, d, hb_in.damage);
    }
    let def = &mut state.fighters[d];
    if let Some((e, by_char, by_facing)) = new_egg {
        fighter::encase(def, &e, before, added, by_char, move_id, by_facing);
        def.hitlag = hitlag;
        def.sdi_wait = 0;
        return;
    }
    // The percent the knockback formula reads: the percent before the hit plus the hit's damage *without* the one-on-one
    // multiplier, which only scales the damage taken (the reference: it never moves the percent a move KOs at; sim 97).
    let kb_percent = (before + hb.damage).min(Fx::from_int(999));
    // (A set-weight hit treats every victim as weighing 100, and so does a bury hit: the reference's bury is weight-independent.)
    let weight = if hb.set_weight || bury_hit {
        Fx::from_int(100)
    } else {
        defender_params.weight
    };
    let mut kb = if hb.fixed_kb > 0 {
        fixed_knockback(hb.fixed_kb, weight, hb.base_knockback, hb.knockback_growth)
    } else {
        knockback(
            kb_percent,
            kb_damage,
            weight,
            hb.base_knockback,
            hb.knockback_growth,
        ) * rage
    };
    // (The bury's own length reads the knockback before crouch cancelling.)
    let full_kb = kb;
    if crouching {
        // Crouch cancelling: holding down on the ground takes some of the knockback off.
        kb = kb * rules.crouch_cancel_kb;
    }
    // Armor: the move's window (in an attack), or the body's air-jump armor (in a midair jump, its frames counted from 0 as a move's
    // are). It sees the hit's damage before the multiplier and its knockback (a crouching fighter has neither). A hit it withstands
    // only freezes (below, after a push has pushed); one that breaks a subtractive knockback armor launches with what is left.
    let armor = if def.state == S::Attack {
        // (A move's armor only in the attacks its `armor_mode` names: E57.)
        let mv = weapon_of(content, &defender_params).get(def.move_id);
        mv.armor
            .filter(|_| moded(mv.armor_mode, def.move_powered))
            .map(|a| (a, def.state_frame))
    } else if def.state == S::Airborne && def.air_jump_frame > 0 {
        defender_params
            .air_jump_armor
            .map(|a| (a, u16::from(def.air_jump_frame - 1)))
    } else if carrying {
        // A carry's armor (E7), its frames counted from the carry's first (255 and on as 255).
        weapon_of(content, &defender_params)
            .get(def.move_id)
            .carry
            .and_then(|c| c.armor)
            .map(|a| (a, def.state_frame.min(255)))
    } else {
        None
    };
    // (A bury hit ignores subtractive knockback armor: the reference's rule.)
    let armor = armor.filter(|(a, _)| !(bury_hit && a.kind == ARMOR_KNOCKBACK && a.subtract));
    // (A grunt's super armor, sim 131: its kind's, through a tell and its move's startup, withstands any hit but a push.)
    let armored = armor.is_some_and(|(a, frame)| a.withstands(frame, hb.damage, kb))
        || (def.super_armor && !hb.push);
    if !armored && !hb.push {
        if let Some((a, frame)) = armor {
            kb = a.broken_knockback(frame, kb);
        }
    }
    // The autolink angle: a grounded victim is sent at 80 degrees; an airborne one is carried along (see `autolink`).
    // The vortex angle (E28): grounded or not, the launch follows the attacker's movement plus a pull (see `fighter::vortex_launch`).
    let vortex_hit = hb.angle == ANGLE_VORTEX;
    let posvec_hit = hb.posvec_frames > 0;
    let half_hit = hb.angle == ANGLE_HALF_LINK;
    let link_hit =
        (hb.angle == ANGLE_AUTOLINK && !def.grounded()) || vortex_hit || posvec_hit || half_hit;
    // The pull angle: no launch; the victim is drawn in (see `fighter::pull_frame`).
    let pull_hit = hb.angle == ANGLE_PULL;
    let degrees = if hb.angle == ANGLE_AUTOLINK {
        80
    } else if vortex_hit || half_hit {
        // (A vortex launch is along the attacker's movement, not an angle: this is only for what reads the launch angle in hitlag.)
        90
    } else if pull_hit {
        0
    } else {
        hb.angle
    };
    let angle = launch_angle(degrees, facing, def.grounded(), kb);
    if hb.push {
        // A move immune to pushes in this window (the reference's windbox immunity) is not moved at all.
        if def.state == S::Attack {
            let mv = weapon_of(content, &defender_params).get(def.move_id);
            let frame = def.state_frame;
            if u16::from(mv.push_immune_from) <= frame && frame <= u16::from(mv.push_immune_to) {
                return;
            }
        }
        // No hitstun and no change of state: the knockback is a shove on top of the fighter's own speed (along the ground if it
        // stands on it).
        let speed = kb * Fx::from_ratio(3, 800);
        def.vel.x += crate::trig::cos(angle) * speed;
        if !def.grounded() {
            def.vel.y += crate::trig::sin(angle) * speed;
        }
        return;
    }
    // A hit on an egg (E8): the (reduced) damage and the hitlag stand, and the egg's time is cut by `hit_cut` a percent added.
    if let Some(e) = egg {
        def.hitlag = hitlag;
        def.sdi_wait = 0;
        def.sdi_mult = hb.sdi;
        def.encase_timer -= e.hit_cut * added;
        return;
    }
    // An armored hit: the damage and hitlag stand, nothing else happens. (A move's armor may lengthen the hitlag for both fighters and
    // remembers the hit for its script: E55.)
    if armored {
        let mut lag = hitlag;
        if def.state == S::Attack {
            let mv = weapon_of(content, &defender_params).get(def.move_id);
            if mv.armor_hitlag != 100 {
                lag = (u32::from(hitlag) * u32::from(mv.armor_hitlag) / 100)
                    .min(u32::from(rules.hitlag_cap)) as u8;
            }
            def.withstood = 1;
        }
        def.hitlag = lag;
        def.sdi_wait = 0;
        if freeze_source && lag > hitlag {
            state.fighters[source].hitlag = state.fighters[source].hitlag.max(lag);
        }
        return;
    }
    // A move that sheds a projectile when its user is hit (E48, sim 117: the rocket barrels flying off).
    let shed = if def.state == S::Attack {
        weapon_of(content, &defender_params)
            .get(def.move_id)
            .shed
            .map(|slot| (slot, def.pos, def.facing))
    } else {
        None
    };
    // A carry the hit breaks through lets go of its fighter (E7).
    if carrying {
        grab::drop_grab(state, content, d);
    }
    if let Some((slot, at, facing)) = shed {
        fire(state, content, d, d, slot, at, facing);
    }
    let def = &mut state.fighters[d];
    // The bury (E5). A buried fighter is freed only by a hit of at least `bury_break_kb` (it is launched in full, below); a weaker
    // hit's damage and hitlag stand and its launch is kept for when the bury ends (the last one kept [E: gap 10]). A grounded fighter
    // not lying down hit by a bury hit is buried; an airborne one takes the hit as usual (a meteor, for a downward bury hit).
    let was_buried = def.state == S::Buried;
    if was_buried {
        if kb < rules.bury_break_kb {
            def.hitlag = hitlag;
            def.sdi_wait = 0;
            def.sdi_mult = hb.sdi;
            def.bury_kb = kb;
            def.bury_angle = angle.raw();
            def.bury_set = true;
            return;
        }
        def.bury_timer = 0;
        def.bury_set = false;
    } else if bury_hit
        && def.grounded()
        && !(matches!(def.state, S::Knockdown | S::GetUp) && !def.tripped)
    {
        // (Lying down is safe from a bury; sitting after a trip is not: the reference's rule.)
        let victim_stocks = def.stocks;
        let frames = bury_frames(rules, def.percent, full_kb, source_stocks, victim_stocks);
        fighter::bury(def, frames);
        def.hitlag = hitlag;
        def.sdi_wait = 0;
        def.sdi_mult = hb.sdi;
        def.hits_taken = 1;
        return;
    }
    // A jab lock (sim 132, the reference's): a hit that would not tumble, on a fighter lying in a knockdown (not a trip), keeps it
    // floored instead of standing it up in hitstun, at most `jab_lock_limit` times a knockdown.
    if def.state == S::Knockdown
        && !def.tripped
        && !was_buried
        && kb < rules.tumble_knockback
        && def.jab_locks < rules.jab_lock_limit
    {
        fighter::jab_lock(def, rules);
        def.hitlag = hitlag;
        def.sdi_wait = 0;
        def.sdi_mult = hb.sdi;
        return;
    }
    // A slip (E44, sim 117: the reference's slip effect): a fighter on the ground, not lying or sitting already, always trips, whatever
    // the hit's launch; a soft slip with no hitlag. Nothing is rolled.
    if hb.slip != crate::moves::SLIP_NONE
        && !was_buried
        && def.grounded()
        && !matches!(def.state, S::Knockdown | S::GetUp)
    {
        let soft = hb.slip == crate::moves::SLIP_SOFT;
        fighter::trip(def, rules, soft);
        def.hitlag = if soft {
            0
        } else if crouching {
            hitlag.min(rules.crouch_cancel_hitlag_cap)
        } else {
            hitlag
        };
        def.sdi_wait = 0;
        def.sdi_mult = hb.sdi;
        return;
    }
    // A trip (the reference's trip chance, sim 86; the base chance, owner decision 50): a fighter on the ground whose launch would not
    // lift it off the ground (sent sideways or down; the engine lifts a grounded fighter on any upward angle) sits down instead of
    // being launched, if the game's own random roll comes up. Nothing is rolled for a hit that cannot trip, so other hits never use
    // up the generator.
    let chance = trip_chance(&hb, kb, rules);
    if chance > 0
        && !was_buried
        && def.grounded()
        && !matches!(def.state, S::Knockdown | S::GetUp)
        && !lifts_off(&content.stage, def, angle)
        && state.rng.range(100) < u32::from(chance)
    {
        fighter::trip(def, rules, false);
        def.hitlag = if crouching {
            hitlag.min(rules.crouch_cancel_hitlag_cap)
        } else {
            hitlag
        };
        def.sdi_wait = 0;
        def.sdi_mult = hb.sdi;
        return;
    }
    let mut stun = hitstun_frames(kb, rules.hitstun_mult);
    // (The previous reference version added a frame to a tumbling launch's hitstun: ssbwiki.com `Hitstun`.)
    if older && stun > 0 && kb >= rules.tumble_knockback {
        stun += u16::from(rules.prev_tumble_hitstun);
    }
    // A hit that holds a victim it does not tumble for a set time (sim 119: the reference's flip jump-off, 20 frames).
    if hb.fixed_stun > 0 && kb < rules.tumble_knockback {
        stun = u16::from(hb.fixed_stun);
    }
    if stun > 0 {
        // (A negative modifier shortens it, but a launched victim keeps at least one frame [E: the reference floors it at 0 and
        // still launches; here 0 would mean a flinch].)
        stun = (i32::from(stun) + i32::from(hb.hitstun_add)).clamp(1, i32::from(u16::MAX)) as u16;
    }

    def.hitlag = if crouching {
        hitlag.min(rules.crouch_cancel_hitlag_cap)
    } else {
        hitlag
    };
    def.sdi_wait = 0;
    def.sdi_mult = hb.sdi;
    // Electric hits let the victim drift with the held stick as hitlag ends (automatic SDI), twice.
    def.asdi = if electric { 2 } else { 0 };
    // Hits taken while still in hitstun are one combo (survival DI grows over a long one).
    def.hits_taken = if def.state == S::Hitstun {
        def.hits_taken.saturating_add(1)
    } else {
        1
    };
    if stun == 0 {
        // A flinch: hitlag only (a reversing hit still turns the fighter round).
        if hb.reverse {
            reverse(def, source_pos);
        }
        // ...but a move that shed something when hit (E48) ends: into the special fall in the air, standing on the ground.
        if shed.is_some() && def.state == S::Attack {
            if def.grounded() {
                fighter::stand_or_fall(def);
            } else {
                fighter::enter_helpless(def);
            }
        }
        return;
    }
    if source_pos.x != def.pos.x {
        def.facing = if source_pos.x > def.pos.x { 1 } else { -1 };
    }
    if hb.reverse {
        reverse(def, source_pos);
    }
    // Being hit clears the once-per-airtime flags.
    def.air_flags = 0;
    if def.ledge != NONE {
        def.ledge = NONE;
        def.ledge_cooldown = defender_params.ledge_regrab_cooldown;
    }
    def.state = S::Hitstun;
    def.state_frame = 0;
    def.tripped = false;
    def.trip_soft = false;
    def.air_jump_frame = 0;
    // Being hit gives back the air dodge and a fresh set of ledge grabs, and ends a short hop.
    def.air_dodge_used = false;
    def.ledge_grab_count = 0;
    def.short_hop = false;
    def.hitstun = stun;
    def.launch_pending = true;
    def.launch_kb = kb;
    def.launch_set = hb.fixed_kb > 0;
    def.launch_older = older;
    def.launch_angle = angle.raw();
    def.link = link_hit;
    def.link_kind = if posvec_hit {
        crate::state::LINK_POSVEC
    } else if vortex_hit {
        crate::state::LINK_VORTEX
    } else if half_hit {
        crate::state::LINK_HALF
    } else {
        crate::state::LINK_AUTOLINK
    };
    def.vortex = 0;
    def.posvec = hb.posvec_frames;
    def.link_point = if posvec_hit {
        posvec_point
    } else {
        link.0 + Vec2::new(hb.link_ahead.mul_int(i32::from(facing)), Fx::ZERO)
    };
    def.link_vel = link.1;
    def.pull = pull_hit;
    if pull_hit {
        // Just in front of whoever pulled it, on the victim's side.
        let side = if def.pos.x >= puller.x { 1 } else { -1 };
        def.link_point = puller + Vec2::new(rules.pull_gap.mul_int(side), Fx::ZERO);
    }
    def.kb_vel = Vec2::ZERO;
    def.fast_fall = false;
    def.tumble = false;
    def.hit_mask = 0;
}

/// How long a bury lasts (the reference's, ssbwiki.com "Buried", 13.0.1): `bury_base + bury_percent x percent + bury_kb x kb` rounded
/// up (`percent` the victim's after the hit, `kb` the hit's knockback at weight 100), plus `bury_behind_1` / `_2` frames when the
/// attacker has one / two or more stocks fewer than the victim, minus `bury_ahead_1` / `_2` when it has more (the page's summary of
/// its stock terms: 8, 12, 18, 22). At least 1.
pub fn bury_frames(
    rules: &Ruleset,
    percent: Fx,
    kb: Fx,
    attacker_stocks: u8,
    victim_stocks: u8,
) -> u16 {
    let t = rules.bury_base + percent * rules.bury_percent + kb * rules.bury_kb;
    let mut frames = (t + Fx::from_raw(65535)).floor_int();
    if attacker_stocks < victim_stocks {
        let behind = if victim_stocks - attacker_stocks >= 2 {
            rules.bury_behind_2
        } else {
            rules.bury_behind_1
        };
        frames += i32::from(behind);
    } else if attacker_stocks > victim_stocks {
        let ahead = if attacker_stocks - victim_stocks >= 2 {
            rules.bury_ahead_2
        } else {
            rules.bury_ahead_1
        };
        frames -= i32::from(ahead);
    }
    frames.clamp(1, i32::from(u16::MAX)) as u16
}

/// Whether a launch at `angle` lifts a grounded fighter off its ground: on flat ground any upward angle; on a slope (sim 124) only
/// an angle above the surface's own slope the way it sends (a low launch into a rising slope runs along it, so such a hit can trip:
/// the reference's "trips only on a slope"). Flat ground reads exactly as before.
pub fn lifts_off(stage: &crate::content::Stage, f: &crate::state::Fighter, angle: Angle) -> bool {
    let (sin, cos) = (crate::trig::sin(angle), crate::trig::cos(angle));
    let Some(p) = collision::platform(stage, f.platform).filter(|p| p.sloped()) else {
        return sin > Fx::ZERO;
    };
    // (The slope's rise the way the launch goes: positive when it climbs that way.)
    let rise = if cos < Fx::ZERO { -p.rise } else { p.rise };
    (p.right - p.left) * sin > rise * cos.abs()
}

/// The percent chance that a hit trips a grounded fighter it would not lift (see `apply_hit`): the reference's base chance
/// (`Ruleset::trip_base`, 7) for a hit of more than `trip_base_kb` (55) knockback, and a hit with a chance of its own (`Hitbox::trip`)
/// always adds the base chance to it, whatever its knockback (ssbwiki.com, Tripping: a 40% move trips 47% of the time). A throw never
/// trips [E: the reference's thrown fighters are launched]. At most 100.
pub fn trip_chance(hb: &Hitbox, kb: Fx, rules: &Ruleset) -> u8 {
    if hb.kind == HIT_THROW {
        0
    } else if hb.trip > 0 {
        hb.trip.saturating_add(rules.trip_base).min(100)
    } else if kb > rules.trip_base_kb {
        rules.trip_base.min(100)
    } else {
        0
    }
}

/// The knockback a hit gives (set knockback, or the formula with rage), before crouch cancelling.
fn fixed_or_plain_kb(hb: &Hitbox, percent: Fx, kb_damage: Fx, weight: Fx, rage: Fx) -> Fx {
    if hb.fixed_kb > 0 {
        fixed_knockback(hb.fixed_kb, weight, hb.base_knockback, hb.knockback_growth)
    } else {
        knockback(
            percent,
            kb_damage,
            weight,
            hb.base_knockback,
            hb.knockback_growth,
        ) * rage
    }
}

/// The reference's reverse effect: the victim ends up facing away from whoever hit it, with its horizontal speed reversed.
fn reverse(def: &mut Fighter, source_pos: Vec2) {
    def.facing = if source_pos.x > def.pos.x {
        -1
    } else if source_pos.x < def.pos.x {
        1
    } else {
        -def.facing
    };
    def.vel.x = -def.vel.x;
}

/// A hit on a raised shield: no damage or launch. The shield loses health (unless it was a perfect shield),
/// the blocker is stunned for `floor(0.8 * damage * type * multiplier + 2)` frames (the reference's formula, see
/// [`shield_stun_frames`]) and slides back. If the shield runs out it breaks.
fn block(
    def: &mut Fighter,
    rules: &Ruleset,
    hb: &Hitbox,
    facing: i8,
    hitlag: u8,
    stun_mult: u16,
    damage_mult: Fx,
) {
    let perfect = def.state_frame <= u16::from(rules.perfect_shield_window);
    let raw = shield_stun_frames(hb.damage, stun_mult, hb.shieldstun);
    let mut stun = raw.clamp(0, i32::from(rules.shield_stun_cap));
    // A perfect shield freezes only the attacker.
    def.hitlag = if perfect { 0 } else { hitlag };
    def.sdi_wait = 0;
    if perfect {
        stun = (stun - i32::from(rules.perfect_shield_stun_cut)).max(0);
    } else {
        let shield_damage =
            hb.damage * damage_mult * Fx::from_int(i32::from(hb.shield_damage)) / Fx::from_int(100);
        def.shield_hp -= shield_damage;
        if def.shield_hp <= Fx::ZERO {
            fighter::break_shield(def, rules);
            return;
        }
    }
    def.shield_stun = stun as u8;
    let mut push = shield_pushback(stun);
    if perfect {
        push = push * Fx::from_ratio(2, 5);
    }
    def.vel.x += push.mul_int(i32::from(facing));
}

/// A shield's pushback for `stun` frames of shield stun: (stun + 1) * 0.09 reference units a frame, at most 1.3 (world units).
fn shield_pushback(stun: i32) -> Fx {
    (Fx::from_int(stun + 1) * Fx::from_ratio(9, 100)).min(Fx::from_ratio(13, 10)) / Fx::from_int(8)
}

/// Whether fighter `fd`'s passive shield (E14, `FighterParams::passive_shield`) stops a projectile box at `at` (`half` and `radius`
/// already scaled) moving at `vel`: the fighter stands, walks or crouches on the ground, the shot is in front of it and not moving
/// away from its front, and the box touches the shield's capsule.
pub(crate) fn passive_shield_blocks(
    fd: &Fighter,
    pd: &FighterParams,
    at: Vec2,
    shot: Shape,
    vel: Vec2,
) -> bool {
    let Some(sh) = pd.passive_shield else {
        return false;
    };
    // (Holding an item turns it off: the reference moves the shield to the back, E4.)
    if !fd.grounded() || !matches!(fd.state, S::Idle | S::Walk | S::Crouch) || fd.holding != NONE {
        return false;
    }
    let facing = i32::from(fd.facing);
    let ahead = (at.x - fd.pos.x).mul_int(facing);
    if ahead <= Fx::ZERO || vel.x.mul_int(facing) > Fx::ZERO {
        return false;
    }
    let scale = pd.hitbox_scale;
    let y = if fd.state == S::Crouch {
        sh.crouch_y
    } else {
        sh.y
    };
    let centre = fd.pos + Vec2::new((sh.x * scale).mul_int(facing), y * scale);
    shapes_touch(
        at,
        shot.half_w,
        shot.half_h,
        shot.radius,
        centre,
        Fx::ZERO,
        sh.half_height * scale,
        sh.radius * scale,
    )
}

/// A projectile's hit stopped by a passive shield (E14): no damage or launch; the fighter freezes for the hit's hitlag on a shield
/// (with shield SDI, `Fighter::shield_sdi`) and is pushed back, away from its front, as a raised shield would be by the hit (no shield
/// damage, no shield stun) [E: the reference's push is "dependent on the damage and knockback", its formula unpublished].
pub(crate) fn passive_block(def: &mut Fighter, rules: &Ruleset, hb: &Hitbox) {
    let scale = Fx::from_ratio(i32::from(hb.hitlag), 100).max(Fx::ONE);
    def.hitlag = hitlag_frames(hb.damage, rules, true, false, scale);
    def.sdi_wait = 0;
    def.shield_sdi = def.hitlag > 0;
    let stun = shield_stun_frames(hb.damage, STUN_PROJECTILE, hb.shieldstun)
        .clamp(0, i32::from(rules.shield_stun_cap));
    def.vel.x -= shield_pushback(stun).mul_int(i32::from(def.facing));
}

/// Shield stun type multipliers (the reference's), in thousandths: most hits 1, aerials 0.33, smash attacks 0.725, projectiles 0.29.
pub const STUN_NORMAL: u16 = 1000;
pub const STUN_AERIAL: u16 = 330;
pub const STUN_SMASH: u16 = 725;
pub const STUN_PROJECTILE: u16 = 290;

/// Raw fixed-point units (1/65536) added before flooring the shield stun (about 0.0005 of a frame): they absorb the damage's own
/// fixed-point rounding (a 1.05 fresh-move bonus, a 1.3% hit), so a stun that should come out whole does.
const SHIELD_STUN_SLACK: i64 = 32;

/// `floor(0.8 * damage * type * multiplier + 2)` (ssbwiki.com, Shieldstun): `stun_mult` is the type in thousandths, `shieldstun`
/// the hitbox's own multiplier in percent. Worked in integers over the damage's raw fixed-point value: 0.8, 0.33 and 0.725 are not
/// exact in 16.16 fixed point, and multiplying by them used to make every whole result a frame short (10% gave 9, not 10). The
/// projectile penalty (0.29) applies only to a projectile hit whose own multiplier is 1, as in the reference.
pub fn shield_stun_frames(damage: Fx, stun_mult: u16, shieldstun: u16) -> i32 {
    let kind = if stun_mult == STUN_PROJECTILE && shieldstun != 100 {
        STUN_NORMAL
    } else {
        stun_mult
    };
    let scaled = i64::from(damage.raw().max(0)) * 8 * i64::from(kind) * i64::from(shieldstun);
    let raw = scaled / (10 * 1000 * 100) + SHIELD_STUN_SLACK;
    i32::try_from(raw >> Fx::FRAC_BITS)
        .unwrap_or(i32::MAX)
        .saturating_add(2)
}

/// How much a hit of this attacker's current move counts toward the shield stun it causes (the type multiplier, thousandths).
fn stun_multiplier(f: &Fighter) -> u16 {
    let id = MoveId::from_index(f.move_id);
    if id.is_aerial() {
        STUN_AERIAL
    } else if matches!(id, MoveId::FSmash | MoveId::USmash | MoveId::DSmash) {
        STUN_SMASH
    } else {
        STUN_NORMAL
    }
}

/// Creates the projectile a move asked for this frame (and a second one a script asked for on the same frame: E3h). A move that
/// already connected in melee (a bayonet hit, for instance) does not also fire.
pub fn spawn_projectiles(state: &mut GameState, content: &Content) {
    for i in 0..crate::state::bodies(state) {
        if !state.fighters[i].spawn_request {
            continue;
        }
        let f = state.fighters[i];
        {
            let g = &mut state.fighters[i];
            g.spawn_request = false;
            g.spawn_custom = false;
            g.spawn_fixed = false;
            g.spawn_kb_add = 0;
            g.spawn_size = 100;
            g.spawn2_request = false;
            g.spawn2_fixed = false;
        }
        // A move that already connected in melee does not also fire, unless its script asked for the shot.
        if f.hit_mask != 0 && !f.spawn_custom {
            continue;
        }
        let mv = weapon_of(content, params_of(content, &f)).get(f.move_id);
        let Some(spec) = mv.projectile else {
            continue;
        };
        // (A shot that grows with a counter's catch carries the caught damage: sim 128.)
        let lerp = mv.lerp.is_some();
        let dir = i32::from(f.facing);
        let scale = params_of(content, &f).hitbox_scale;
        let first = Shot {
            pos: if f.spawn_fixed {
                f.spawn_pos
            } else if f.spawn_custom {
                f.pos + f.spawn_pos
            } else {
                Vec2::new(
                    f.pos.x + (spec.x * scale).mul_int(dir),
                    f.pos.y + spec.y * scale,
                )
            },
            vel: if f.spawn_custom {
                f.spawn_vel
            } else if spec.arc.angle != 0 {
                let a = Angle::from_degrees(i32::from(spec.arc.angle));
                Vec2::new(
                    (spec.speed * crate::trig::cos(a)).mul_int(dir),
                    spec.speed * crate::trig::sin(a),
                )
            } else {
                Vec2::new(spec.speed.mul_int(dir), Fx::ZERO)
            },
            kb_add: if f.spawn_custom { f.spawn_kb_add } else { 0 },
            size: if f.spawn_custom { f.spawn_size } else { 100 },
        };
        add_projectile(state, i, &f, &spec, first, lerp);
        // A second shot on the same frame (a script's second `spawn`): plain size and knockback.
        if f.spawn_custom && f.spawn2_request {
            let second = Shot {
                pos: if f.spawn2_fixed {
                    f.spawn2_pos
                } else {
                    f.pos + f.spawn2_pos
                },
                vel: f.spawn2_vel,
                kb_add: 0,
                size: 100,
            };
            add_projectile(state, i, &f, &spec, second, lerp);
        }
    }
}

/// Where a new projectile starts and how it goes.
struct Shot {
    pos: Vec2,
    vel: Vec2,
    kb_add: u8,
    size: u16,
}

/// Puts a projectile of `spec` fired by fighter `i` into the first free slot of the pool (none if the pool is full, or if the fighter
/// already has its tool's limit out: `Tool::limit`, E3d). A charge-scaled shot (E35) takes its damage, speed, life and size from the
/// fighter's charge (`ChargedShot`); one that grows with a counter's catch (`lerp`, sim 128) carries the fighter's caught damage.
fn add_projectile(
    state: &mut GameState,
    i: usize,
    f: &Fighter,
    spec: &crate::moves::ProjectileSpawn,
    shot: Shot,
    lerp: bool,
) {
    let tool = spec.tool;
    if tool.limit > 0 && scripting::owned(&state.projectiles, i, tool.kind, f.move_id) >= tool.limit
    {
        return;
    }
    let (mut vel, mut life, mut size, mut damage) = (shot.vel, spec.life, shot.size, Fx::ZERO);
    let mut power = 100u8;
    if let Some(c) = spec.charged {
        // (Every hit of the shot, its phases too, by the charge: `ChargedShot::power_to`, sim 113.)
        power = c
            .lerp(
                Fx::from_int(100),
                Fx::from_int(i32::from(c.power_to)),
                c.level(f.charge),
            )
            .floor_int()
            .clamp(1, 255) as u8;
        let level = c.level(f.charge);
        // (The speed is a ratio of the uncharged one, so a script's own direction and speed are scaled alike.)
        if spec.speed != Fx::ZERO {
            let k = c.lerp(spec.speed, c.speed_to, level) / spec.speed;
            vel = Vec2::new(vel.x * k, vel.y * k);
        }
        life = c
            .lerp(
                Fx::from_int(i32::from(spec.life)),
                Fx::from_int(i32::from(c.life_to)),
                level,
            )
            .floor_int()
            .clamp(1, 255) as u8;
        let grown = c
            .lerp(
                Fx::from_int(i32::from(c.size_from)),
                Fx::from_int(i32::from(c.size_to)),
                level,
            )
            .floor_int()
            .clamp(1, 1000);
        size = (i32::from(size) * grown / 100).clamp(1, 1000) as u16;
        damage = c.lerp(spec.hitbox.damage, c.damage_to, level);
        if c.floor {
            damage = Fx::from_int(damage.floor_int());
        }
        // (A share of that damage, rounded down again with `floor`: the double arrow's x0.8, sim 111.)
        // (Worked in whole numbers when floored: 0.8 is not exact in fixed point, and 5 x 0.8 must give 4.)
        if c.times != 100 {
            damage = if c.floor {
                Fx::from_int(damage.floor_int() * i32::from(c.times) / 100)
            } else {
                damage * Fx::from_ratio(i32::from(c.times), 100)
            };
        }
    }
    if lerp {
        damage = f.counter_damage;
    }
    if let Some(slot) = state.projectiles.iter_mut().find(|p| !p.active) {
        *slot = Projectile {
            active: true,
            owner: i as u8,
            origin: i as u8,
            power,
            move_id: f.move_id,
            pos: shot.pos,
            vel,
            age: 0,
            life,
            bounces: 0,
            kb_add: shot.kb_add,
            size,
            vars: [0; crate::PROJECTILE_VARS],
            hit_mask: 0,
            hit_age: [0; MAX_BODIES],
            facing: f.facing,
            bursting: false,
            damage,
            kind: tool.kind,
            rest: REST_FLYING,
            stale: f.move_id,
            charge: f.charge,
            orb: ORB_NONE,
            held: 0,
            orb_hits: 0,
        };
    }
}

/// Fires the projectile of fighter `origin`'s move `move_id` (its weapon's spec) at `at` (the spec's offset from there, by `facing`),
/// along the spec's speed and angle the way `facing` points, credited to `owner`: a shot made by something other than the fighter's
/// own move frame (E43: a growing or falling placed piece). No limit is checked and no charge applies. Returns its slot.
pub(crate) fn fire(
    state: &mut GameState,
    content: &Content,
    owner: usize,
    origin: usize,
    move_id: u8,
    at: Vec2,
    facing: i8,
) -> Option<usize> {
    let f = state.fighters.get(origin)?;
    let p = params_of(content, f);
    let spec = weapon_of(content, p).get(move_id).projectile?;
    let scale = p.hitbox_scale;
    let dir = i32::from(facing);
    let vel = if spec.arc.angle != 0 {
        let a = Angle::from_degrees(i32::from(spec.arc.angle));
        Vec2::new(
            (spec.speed * crate::trig::cos(a)).mul_int(dir),
            spec.speed * crate::trig::sin(a),
        )
    } else {
        Vec2::new(spec.speed.mul_int(dir), Fx::ZERO)
    };
    let n = state.projectiles.iter().position(|q| !q.active)?;
    state.projectiles[n] = Projectile {
        active: true,
        owner: owner.min(MAX_BODIES - 1) as u8,
        origin: origin as u8,
        power: 100,
        move_id,
        pos: at + Vec2::new((spec.x * scale).mul_int(dir), spec.y * scale),
        vel,
        age: 0,
        life: spec.life,
        bounces: 0,
        kb_add: 0,
        size: 100,
        vars: [0; crate::PROJECTILE_VARS],
        hit_mask: 0,
        hit_age: [0; MAX_BODIES],
        facing,
        bursting: false,
        damage: Fx::ZERO,
        kind: spec.tool.kind,
        rest: REST_FLYING,
        stale: move_id,
        charge: 0,
        orb: ORB_NONE,
        held: 0,
        orb_hits: 0,
    };
    Some(n)
}

/// The fighter (other than `owner`) whose pocket (E42, `Move::pocket`) is open and empty and touches a projectile shape at `pos`: the
/// lowest index wins.
pub(crate) fn pocket_touching(
    state: &GameState,
    content: &Content,
    owner: usize,
    pos: Vec2,
    shape: Shape,
) -> Option<usize> {
    (0..crate::state::bodies(state)).find(|&d| {
        rivals(owner, d)
            && pocket_circle(state, content, d).is_some_and(|(c, r)| {
                shapes_touch(
                    pos,
                    shape.half_w,
                    shape.half_h,
                    shape.radius,
                    c,
                    Fx::ZERO,
                    Fx::ZERO,
                    r,
                )
            })
    })
}

/// Fighter `d`'s open pocket (E42) if it can put something away now: in a move with a `pocket` on its window's frames, its pocket
/// empty and nothing in its hand (a held item is put away instead: `items`). Returns its circle (centre, radius), scaled.
pub(crate) fn pocket_circle(state: &GameState, content: &Content, d: usize) -> Option<(Vec2, Fx)> {
    let f = &state.fighters[d];
    if !f.active || f.state != S::Attack || f.pocket != NONE || f.holding != NONE {
        return None;
    }
    let p = params_of(content, f);
    let pc = weapon_of(content, p).get(f.move_id).pocket?;
    let frame = f.state_frame;
    if frame < u16::from(pc.start) || frame > u16::from(pc.end) {
        return None;
    }
    let s = p.hitbox_scale;
    let c = f.pos + Vec2::new((pc.x * s).mul_int(i32::from(f.facing)), pc.y * s);
    Some((c, pc.radius * s))
}

/// Moves projectiles, despawns them (lifetime, walls, blast zone), lodges, drops and catches them (E3b, E3d) and applies their hits.
pub fn update_projectiles(state: &mut GameState, content: &Content) {
    let stage = &content.stage;
    for n in 0..state.projectiles.len() {
        let pr = state.projectiles[n];
        if !pr.active {
            continue;
        }
        // A pocketed one (E42) waits, frozen, until it is taken out.
        if pr.rest == REST_POCKETED {
            continue;
        }
        // An orb circling or striking a fighter (E59, sim 128) has rules of its own.
        if pr.orb == ORB_CIRCLE || pr.orb == ORB_STRIKE {
            crate::orbs::update_orb(state, content, n);
            continue;
        }
        let owner = usize::from(pr.owner).min(MAX_BODIES - 1);
        let origin = usize::from(pr.origin).min(MAX_BODIES - 1);
        let origin_params = params_of(content, &state.fighters[origin]);
        let fired_by = weapon_of(content, origin_params).get(pr.move_id);
        let Some(mut spec) = fired_by.projectile else {
            state.projectiles[n].active = false;
            continue;
        };
        // A loose orb (E59: reflected or let out of a pocket) flies as a plain shot with the orbs' strike hit.
        if pr.orb == ORB_LOOSE {
            let Some(o) = fired_by.orbs else {
                state.projectiles[n].active = false;
                continue;
            };
            spec = crate::orbs::loose_spec(&spec, &o);
        }
        // (A shot that grows with a counter's catch reads its hits between the written ones and the caught damage's: sim 128.)
        if let Some(l) = fired_by.lerp {
            spec.phases = lerped(&spec.phases, &l, pr.damage);
        }
        // A lodged or fallen projectile (E3b, E3d) only waits out its life (a fallen one falls first): no script, no hits.
        if pr.rest != REST_FLYING {
            rest_frame(&mut state.projectiles[n], &spec, stage);
            continue;
        }
        // The projectile's own script steers it before it moves.
        if let Some(program) = &fired_by.projectile_script {
            let around = surroundings(state, owner, pr.pos);
            if scripting::run_projectile(program, &mut state.projectiles[n], around) {
                state.projectiles[n].active = false;
                continue;
            }
        }
        let mut pr = state.projectiles[n];

        let age = pr.age.saturating_add(1);
        // Gravity (an arcing projectile), then the move; a bouncing one comes back up off the top of a platform it falls onto.
        let arc = spec.arc;
        // A projectile that has burst stands still (E3e).
        let burst = spec.burst.filter(|_| pr.bursting);
        if arc.gravity != Fx::ZERO && burst.is_none() {
            pr.vel.y -= arc.gravity;
            if arc.max_fall > Fx::ZERO {
                pr.vel.y = pr.vel.y.max(-arc.max_fall);
            }
        }
        // A shot with drag slows along its flight, down to a stop (E35).
        if arc.drag > Fx::ZERO && burst.is_none() {
            pr.vel = slowed(pr.vel, arc.drag);
        }
        let mut pos = pr.pos + pr.vel;
        let scale = origin_params.hitbox_scale;
        if arc.bounces > 0 && pr.vel.y < Fx::ZERO && burst.is_none() {
            if let Some(i) = collision::find_landing(stage, pr.pos, pos, false) {
                // (Too fast a landing does not bounce either: `ProjectileArc::bounce_max_fall`.)
                let too_fast = arc.bounce_max_fall > Fx::ZERO && -pr.vel.y > arc.bounce_max_fall;
                if pr.bounces >= arc.bounces || too_fast {
                    // A projectile that bursts on the ground (sim 95) bursts where it touched instead of vanishing.
                    if let Some(b) = spec.burst.filter(|b| b.on_ground) {
                        let at = Vec2::new(pos.x, stage.platforms[i].top_at(pos.x));
                        start_burst(&mut state.projectiles[n], &b, at, scale);
                        continue;
                    }
                    state.projectiles[n].active = false;
                    continue;
                }
                pos.y = stage.platforms[i].top_at(pos.x);
                pr.vel.y = (-pr.vel.y).clamp(arc.bounce_min, arc.bounce_max)
                    * Fx::from_ratio(i32::from(arc.bounce_keep_y), 100);
                pr.vel.x = pr.vel.x * Fx::from_ratio(i32::from(arc.bounce_keep), 100);
                pr.bounces += 1;
            }
        }
        state.projectiles[n].vel = pr.vel;
        state.projectiles[n].bounces = pr.bounces;
        let in_wall = stage.platforms.iter().any(|b| {
            !b.pass_through
                && pos.x > b.left
                && pos.x < b.right
                && pos.y > b.bottom
                && pos.y < b.top_at(pos.x)
        });
        let outside = pos.x < stage.blast_left
            || pos.x > stage.blast_right
            || pos.y < stage.blast_bottom
            || pos.y > stage.blast_top;
        // A lodging projectile (E3b) sticks where it meets a platform top or a solid wall, at a steep enough angle (else it is gone).
        if arc.lodge > 0 && arc.bounces == 0 && burst.is_none() && !outside && age < pr.life {
            if let Some((at, angle_ok)) = lodge_point(stage, pr.pos, pos, pr.vel, arc.lodge_angle) {
                // (A shot that lodges as an item, E4, becomes the item there: the reference's arrows anyone can pick up.)
                if let (true, Some(spec)) = (angle_ok, arc.lodge_item) {
                    state.projectiles[n].active = false;
                    if let Some(i) =
                        crate::items::make(state, content, origin, spec, at, pr.move_id)
                    {
                        let item = &mut state.items[i];
                        item.rest = crate::state::ITEM_LODGED;
                        item.life = u16::from(arc.lodge);
                        item.owner = pr.owner;
                        item.facing = pr.facing;
                        // (The way it was flying as it stuck, for the view to point it: sim 113.)
                        item.angle = crate::trig::degrees_of(pr.vel.x, pr.vel.y);
                    }
                    continue;
                }
                let p = &mut state.projectiles[n];
                if angle_ok {
                    p.rest = REST_LODGED;
                    p.pos = at;
                    p.vel = Vec2::ZERO;
                    p.age = 0;
                    p.life = arc.lodge;
                } else {
                    p.active = false;
                }
                continue;
            }
        }
        // A projectile that bursts on a wall (E3e) bursts just short of it (where it was), instead of being removed.
        if let Some(b) = spec.burst.filter(|b| b.on_wall && !pr.bursting) {
            if in_wall && age < pr.life && !outside {
                start_burst(&mut state.projectiles[n], &b, pr.pos, scale);
                continue;
            }
        }
        // ...and one that bursts at the end of its life (sim 95) bursts where it is.
        if let Some(b) = spec.burst.filter(|b| b.on_expire && !pr.bursting) {
            if age >= pr.life && !in_wall && !outside {
                start_burst(&mut state.projectiles[n], &b, pr.pos, scale);
                continue;
            }
        }
        if age >= pr.life || in_wall || outside {
            state.projectiles[n].active = false;
            continue;
        }
        // A reflector in the way turns it around: it now belongs to the reflector and hits harder. (One stronger than the
        // reflector's damage limit goes through; a burst is not reflected.)
        let size = Fx::from_ratio(i32::from(pr.size), 100);
        let shape = Shape {
            half_w: spec.hitbox.half_width * origin_params.hitbox_scale,
            half_h: spec.hitbox.half_height * origin_params.hitbox_scale * size,
            radius: spec.hitbox.radius * origin_params.hitbox_scale * size,
        };
        // (A charge-scaled shot's damage was set as it was fired: E35.)
        let base_damage = if spec.charged.is_some() {
            pr.damage
        } else {
            spec.hitbox.damage
        };
        let shot_damage = base_damage * Fx::from_int(i32::from(pr.power)) / Fx::from_int(100);
        let every_hit = spec.charged.is_some_and(|c| c.every_hit);
        // An open pocket in the way puts it away (E42): it stops where it was caught, keeping the speed it had.
        if burst.is_none() && spec.tool.pocket != crate::moves::POCKET_NEVER {
            if let Some(d) = pocket_touching(state, content, owner, pos, shape) {
                let p = &mut state.projectiles[n];
                p.rest = REST_POCKETED;
                p.pos = pos;
                // (A tool that frees its maker once pocketed no longer counts as its maker's: the reference's rocket.)
                if spec.tool.pocket == crate::moves::POCKET_FREES {
                    p.kind = 0;
                }
                state.fighters[d].pocket = n as i8;
                state.fighters[d].pocket_item = false;
                continue;
            }
        }
        if burst.is_none() {
            reflector_detects(state, content, owner, pos, shape);
        }
        if let Some((r, rf)) = reflector_touching(state, content, owner, pos, shape)
            .filter(|(_, rf)| rf.max_damage == Fx::ZERO || shot_damage <= rf.max_damage)
            .filter(|_| burst.is_none())
        {
            let speed = Fx::from_int(i32::from(rf.speed_percent)) / Fx::from_int(100);
            let p = &mut state.projectiles[n];
            p.owner = r as u8;
            p.power = rf.damage_percent;
            p.vel = Vec2::new(
                -(pr.vel.x * speed),
                if rf.reverse_vertical {
                    -(pr.vel.y * speed)
                } else {
                    pr.vel.y
                },
            );
            p.pos = pos;
            p.age = 0;
            // (Its hits start again for its new owner, and it faces the other way.)
            p.hit_mask = 0;
            p.hit_age = [0; MAX_BODIES];
            p.facing = -p.facing;
            p.life = ((u16::from(pr.life) * u16::from(rf.life_percent)) / 100).clamp(1, 255) as u8;
            continue;
        }
        // An attack in the way may clank with it: it is destroyed, or with `Tool::on_clank` (E3d) it drops.
        if spec.hitbox.clanks
            && burst.is_none()
            && clank_projectile(state, content, owner, pos, shape.radius, shot_damage)
        {
            let p = &mut state.projectiles[n];
            match spec.tool.on_clank {
                Some(fall) => {
                    let travel = if pr.vel.x < Fx::ZERO {
                        -1
                    } else if pr.vel.x > Fx::ZERO {
                        1
                    } else {
                        i32::from(pr.facing)
                    };
                    p.rest = REST_FALLEN;
                    p.pos = pos;
                    p.vel = Vec2::new((fall.vx * scale).mul_int(travel), fall.vy * scale);
                    p.age = 0;
                    p.life = fall.life.max(1);
                }
                None => p.active = false,
            }
            continue;
        }
        state.projectiles[n].pos = pos;
        state.projectiles[n].age = age;

        // Its owner catches it on its way back (E3d): touching its firer's body from `Tool::catch_from` on.
        if burst.is_none() && catch_by_owner(state, content, n, &spec, pos, shape, age) {
            continue;
        }

        // Damage falls off with distance travelled (unless the projectile's hit goes by phases of its age).
        let progress = Fx::from_ratio(i32::from(age), i32::from(pr.life.max(1)));
        // (Phases sharing a start are rings, all out at once: the first that touches a fighter hits it. E58, sim 125.)
        let (rings, phased) = match burst {
            Some(b) => {
                let r = crate::moves::rings_at(&b.phases, age);
                if r[0].is_none() {
                    continue;
                }
                (r, true)
            }
            None => {
                let mut r = crate::moves::rings_at(&spec.phases, age);
                let phased = r[0].is_some();
                if !phased {
                    r[0] = Some(spec.hitbox);
                }
                (r, phased)
            }
        };
        let (pierce, rehit) = match burst {
            Some(b) => (true, b.rehit),
            None => (spec.pierce, spec.rehit),
        };
        let fit = |mut hb: Hitbox| {
            hb.radius = hb.radius * origin_params.hitbox_scale * size;
            hb.half_height = hb.half_height * origin_params.hitbox_scale * size;
            hb.half_width = hb.half_width * origin_params.hitbox_scale;
            hb.base_knockback = hb.base_knockback.saturating_add(i16::from(pr.kb_add));
            if spec.charged.is_some() && (!phased || every_hit) {
                // (A charge-scaled shot keeps the damage it was fired with: no falloff; with `every_hit`, its rings too: sim 128.)
                hb.damage = pr.damage;
            } else if !phased && !spec.any_phase() {
                hb.damage = spec.hitbox.damage + (spec.end_damage - spec.hitbox.damage) * progress;
            }
            hb.damage = hb.damage * Fx::from_int(i32::from(pr.power)) / Fx::from_int(100);
            hb
        };
        let rings = rings.map(|r| r.map(fit));
        let Some(hb) = rings[0] else {
            continue;
        };
        // (A phase of radius 0 is a gap: no hit this frame.)
        if hb.radius <= Fx::ZERO {
            continue;
        }
        let hit_shape = Shape {
            half_w: hb.half_width,
            half_h: hb.half_height,
            radius: hb.radius,
        };
        // Placed pieces (E43): water touching one that grows waters it (the shot goes on); a shot with a hit is used up on one with hit
        // points (a piercing one passes).
        if burst.is_none() && spec.tool.water > 0 {
            crate::items::water(state, content, pos, hit_shape, spec.tool.water, owner);
        }
        if burst.is_none()
            && !pierce
            && hb.kind == HIT_NORMAL
            && !hb.push
            && hb.damage > Fx::ZERO
            && crate::items::shot_on_placed(state, content, pos, hit_shape, hb.damage)
        {
            state.projectiles[n].active = false;
            continue;
        }

        for d in 0..crate::state::bodies(state) {
            let fd = &state.fighters[d];
            let counter = counter_open(content, fd, false);
            let intangible = is_intangible(fd);
            if !rivals(owner, d) || (intangible && counter.is_none()) {
                continue;
            }
            // A fighter already hit by this group waits for the rehit interval (the reference's per-target rehit), or for ever.
            let bit = hit_bit(d, hb.group);
            let seen = state.projectiles[n].hit_mask & bit != 0;
            if seen && (rehit == 0 || age.saturating_sub(state.projectiles[n].hit_age[d]) < rehit) {
                continue;
            }
            let pd = params_of(content, fd);
            // A passive shield in the way stops the shot (E14): a burst is not stopped.
            if burst.is_none()
                && !intangible
                && !hb.unblockable
                && passive_shield_blocks(fd, pd, pos, hit_shape, pr.vel)
            {
                passive_block(&mut state.fighters[d], &content.rules, &hb);
                let p = &mut state.projectiles[n];
                p.hit_mask |= bit;
                p.hit_age[d] = age;
                if !pierce {
                    p.active = false;
                    break;
                }
                continue;
            }
            let wd = weapon_of(content, pd);
            let hurt = body_hurtboxes(content, fd);
            // The first ring that reaches the fighter is the hit (one ring for a shot without rings).
            let mut hb = hb;
            let (mut touching, mut caught) = (false, false);
            // (A ring for grounded or aerial fighters only passes the others by: sim 128.)
            for ring in rings
                .iter()
                .flatten()
                .filter(|r| r.radius > Fx::ZERO && hits_target(r, fd))
            {
                let (t, c) = reach(
                    counter,
                    intangible,
                    ring,
                    &hurt,
                    intangible_parts(fd, wd),
                    pos,
                );
                if t {
                    (hb, touching, caught) = (*ring, true, c);
                    break;
                }
            }
            if !touching
                && !intangible
                && hb.kind == HIT_NORMAL
                && !hb.push
                && fd.balloons > 0
                && crate::balloons::touches(
                    content,
                    fd,
                    pos,
                    hit_shape.half_w,
                    hit_shape.half_h,
                    hit_shape.radius,
                )
            {
                // (A shot that reaches only the balloons is used up on them, as on a fighter: E41.)
                crate::balloons::hit(state, content, d, hb.damage);
                let p = &mut state.projectiles[n];
                p.hit_mask |= bit;
                p.hit_age[d] = age;
                if !pierce {
                    p.active = false;
                    break;
                }
                continue;
            }
            if touching {
                // Forward is the way it travels; a projectile standing still faces the way its owner did when it was made.
                let travel = if pr.vel.x < Fx::ZERO {
                    -1
                } else if pr.vel.x > Fx::ZERO || pr.facing == 0 {
                    1
                } else {
                    pr.facing
                };
                let facing = launch_side(hb.angle_type, travel, travel, travel, pos.x, fd.pos.x);
                // Projectiles stun a shield much less.
                let mult = STUN_PROJECTILE;
                poke(&content.rules, &state.fighters[d], &mut hb, pos);
                // (Whether a shield took it, and what the fighter's percent was: a curse lands only on a hit that dealt damage, E54;
                // a burst that a shield does not set off, sim 121.)
                let shielded = state.fighters[d].state == S::Shield && !hb.unblockable;
                let percent_before = state.fighters[d].percent;
                apply_hit(
                    state,
                    content,
                    owner,
                    d,
                    &hb,
                    facing,
                    pos,
                    false,
                    mult,
                    pr.stale,
                    (pos, pr.vel),
                    caught,
                    true,
                );
                // A beam with orbs puts them on a fighter it hit (E59, sim 128): not a shield, not a counter's catch.
                if fired_by.orbs.is_some() && pr.orb == ORB_NONE && !shielded && !caught {
                    crate::orbs::place(state, content, n, d);
                }
                if let Some(c) = fired_by.curse {
                    if state.fighters[d].percent > percent_before {
                        let v = &mut state.fighters[d];
                        v.curse_left = c.frames;
                        v.curse_every = c.every;
                        v.curse_tick = 0;
                        v.curse_damage = c.damage;
                        v.curse_by = owner as u8;
                    }
                }
                let p = &mut state.projectiles[n];
                p.hit_mask |= bit;
                p.hit_age[d] = age;
                if burst.is_none() {
                    // The first fighter it touches makes a bursting projectile burst there (E3e), unless it bursts only on walls, the
                    // ground or its life's end (`Burst::on_hit` false, sim 113): then it is used up on the fighter as any shot. (Nor on a
                    // shield, for one that a shield does not set off: `Burst::on_shield`, sim 121.)
                    if let Some(b) = spec
                        .burst
                        .filter(|b| b.on_hit && (b.on_shield || !shielded))
                    {
                        start_burst(p, &b, pos, scale);
                        break;
                    }
                }
                if !pierce {
                    p.active = false;
                    break;
                }
            }
        }
    }
}

/// `phases` read between their written hits and `l`'s at the same places, by `damage` (sim 128: a counter's growing answer).
fn lerped(
    phases: &[Option<(u8, Hitbox)>; crate::moves::MAX_PHASES],
    l: &crate::moves::Lerp,
    damage: Fx,
) -> [Option<(u8, Hitbox)>; crate::moves::MAX_PHASES] {
    let mut out = *phases;
    for (slot, high) in out.iter_mut().zip(l.phases.iter()) {
        if let (Some((_, low)), Some((_, high))) = (slot.as_mut(), high) {
            *low = l.between(*low, *high, damage);
        }
    }
    out
}

/// One frame of every fighter's damage over time (E54, sim 121: the reference's curse): every `curse_every` frames while it lasts the
/// fighter takes its damage (times the one-on-one multiplier), credited to whoever put it there (the gauges see it as any damage); no
/// hitlag, no flinch. Counted every frame, hitlag included [E].
pub fn curse_tick(state: &mut GameState, content: &Content) {
    for i in 0..crate::state::bodies(state) {
        let f = &mut state.fighters[i];
        if f.curse_left == 0 || !f.active {
            continue;
        }
        f.curse_left -= 1;
        f.curse_tick = f.curse_tick.saturating_add(1);
        if f.curse_tick < f.curse_every.max(1) {
            continue;
        }
        f.curse_tick = 0;
        let before = f.percent;
        f.percent = (f.percent + f.curse_damage * content.rules.damage_mult).min(Fx::from_int(999));
        let added = f.percent - before;
        let by = usize::from(f.curse_by).min(MAX_BODIES - 1);
        crate::gauge::on_damage_over_time(state, content, by, i, added);
        // (A wing may come from it, as from any damage taken: E57.)
        if added > Fx::ZERO {
            crate::wing::on_hurt(state, content, by, i, before);
        }
    }
}

/// A projectile's shape for touching things: a rounded rectangle (`shapes_touch`), already scaled.
#[derive(Clone, Copy)]
pub(crate) struct Shape {
    pub(crate) half_w: Fx,
    pub(crate) half_h: Fx,
    pub(crate) radius: Fx,
}

/// `v` slowed by `drag` along its own direction, down to a stop (E35).
fn slowed(v: Vec2, drag: Fx) -> Vec2 {
    let speed = v.length();
    if speed <= drag {
        return Vec2::ZERO;
    }
    let k = (speed - drag) / speed;
    Vec2::new(v.x * k, v.y * k)
}

/// Where a lodging projectile going from `from` to `to` meets a surface (E3b): the top of a platform it falls onto (where it crosses
/// it), or a solid block it enters (just short of it, at `from`); and whether it meets it at `min_angle` degrees or more (0: any): a
/// wall by its horizontal speed against its vertical, a top or underside the other way round.
fn lodge_point(
    stage: &crate::content::Stage,
    from: Vec2,
    to: Vec2,
    vel: Vec2,
    min_angle: u8,
) -> Option<(Vec2, bool)> {
    let a = Angle::from_degrees(i32::from(min_angle));
    let (sin, cos) = (crate::trig::sin(a), crate::trig::cos(a));
    // (Steep enough: the speed across the surface over the speed along it is at least tan(min_angle).)
    let steep = |across: Fx, along: Fx| min_angle == 0 || across.abs() * cos >= along.abs() * sin;
    if vel.y < Fx::ZERO {
        if let Some(i) = collision::find_landing(stage, from, to, false) {
            let at = Vec2::new(to.x, stage.platforms[i].top_at(to.x));
            return Some((at, steep(vel.y, vel.x)));
        }
    }
    let block = stage.platforms.iter().find(|b| {
        !b.pass_through
            && to.x > b.left
            && to.x < b.right
            && to.y > b.bottom
            && to.y < b.top_at(to.x)
    })?;
    let side = from.x <= block.left || from.x >= block.right;
    let ok = if side {
        steep(vel.x, vel.y)
    } else {
        steep(vel.y, vel.x)
    };
    Some((from, ok))
}

/// One frame of a lodged or fallen projectile: it ages and goes at the end of its life; a fallen one falls (`Tool::on_clank`'s
/// gravity and top speed) and comes to rest on a platform top, and leaves at the blast zone.
fn rest_frame(
    p: &mut Projectile,
    spec: &crate::moves::ProjectileSpawn,
    stage: &crate::content::Stage,
) {
    p.age = p.age.saturating_add(1);
    if p.age >= p.life {
        p.active = false;
        return;
    }
    if p.rest != REST_FALLEN || p.vel == Vec2::ZERO {
        return;
    }
    if let Some(fall) = spec.tool.on_clank {
        p.vel.y = (p.vel.y - fall.gravity).max(-fall.max_fall);
    }
    let to = p.pos + p.vel;
    if p.vel.y < Fx::ZERO {
        if let Some(i) = collision::find_landing(stage, p.pos, to, false) {
            p.pos = Vec2::new(to.x, stage.platforms[i].top_at(to.x));
            p.vel = Vec2::ZERO;
            return;
        }
    }
    p.pos = to;
    if to.x < stage.blast_left
        || to.x > stage.blast_right
        || to.y < stage.blast_bottom
        || to.y > stage.blast_top
    {
        p.active = false;
    }
}

/// The owner's catch (E3d, `Tool::catch_from`): a projectile still its firer's, from that age on, touching its owner's body is removed
/// (true), and a free owner starts the catch move. An owner dodging lets it fly on.
fn catch_by_owner(
    state: &mut GameState,
    content: &Content,
    n: usize,
    spec: &crate::moves::ProjectileSpawn,
    pos: Vec2,
    shape: Shape,
    age: u8,
) -> bool {
    let p = state.projectiles[n];
    let tool = spec.tool;
    if tool.catch_from == 255 || age < tool.catch_from || p.owner != p.origin {
        return false;
    }
    let o = usize::from(p.owner).min(MAX_BODIES - 1);
    let f = &state.fighters[o];
    if !f.active || matches!(f.state, S::Roll | S::SpotDodge | S::AirDodge | S::Respawn) {
        return false;
    }
    let hurt = body_hurtboxes(content, f);
    if !touches(&hurt, 0, pos, shape.half_w, shape.half_h, shape.radius) {
        return false;
    }
    state.projectiles[n].active = false;
    let free = matches!(
        f.state,
        S::Idle | S::Walk | S::Run | S::Dash | S::Turn | S::Crouch | S::Airborne
    ) && f.hitlag == 0;
    if let (true, Some(m)) = (free, tool.catch_move) {
        let w = weapon_of(content, params_of(content, f));
        if !w.get(m).is_empty() {
            fighter::begin_attack(&mut state.fighters[o], MoveId::from_index(m));
        }
    }
    true
}

/// A projectile becomes its burst (E3e) at `at`: it stops, its age and life start again as the burst's, and its hits start afresh
/// (the fighter that stopped it is hit again).
fn start_burst(p: &mut Projectile, b: &crate::moves::Burst, at: Vec2, scale: Fx) {
    p.bursting = true;
    p.pos = at + Vec2::new(Fx::ZERO, b.y * scale);
    p.vel = Vec2::ZERO;
    p.age = 0;
    p.life = b.life;
    // (With `keep_hits` the fighters it already hit stay hit: a burst phase of the same group skips them.)
    if !b.keep_hits {
        p.hit_mask = 0;
        p.hit_age = [0; MAX_BODIES];
    }
}

/// What a projectile script can see: its owner and the nearest other fighter that is still in the match.
fn surroundings(state: &GameState, owner: usize, at: Vec2) -> scripting::Surroundings {
    let mut target: Option<(Fx, Vec2)> = None;
    for (i, f) in state
        .fighters
        .iter()
        .enumerate()
        .take(crate::state::bodies(state))
    {
        if !rivals(owner, i) || f.stocks == 0 || !f.active {
            continue;
        }
        let d = (f.pos - at).length_sq();
        if target.is_none_or(|(best, _)| d < best) {
            target = Some((d, f.pos));
        }
    }
    let stick = state.fighters[owner].history[0];
    scripting::Surroundings {
        owner: state.fighters[owner].pos,
        target: target.map(|(_, p)| p),
        owner_stick: Vec2::new(stick.stick_x_fx(), stick.stick_y_fx()),
    }
}

/// A fighter other than `owner` whose reflector is up and touches a projectile at `pos`.
pub(crate) fn reflector_touching(
    state: &GameState,
    content: &Content,
    owner: usize,
    pos: Vec2,
    shot: Shape,
) -> Option<(usize, Reflector)> {
    for (i, f) in state
        .fighters
        .iter()
        .enumerate()
        .take(crate::state::bodies(state))
    {
        if !rivals(owner, i) || f.state != S::Attack {
            continue;
        }
        let Some(rf) = weapon_of(content, params_of(content, f))
            .get(f.move_id)
            .reflector
        else {
            continue;
        };
        let frame = u8::try_from(f.state_frame).unwrap_or(u8::MAX);
        let scale = params_of(content, f).hitbox_scale;
        let at = |x: Fx, y: Fx| {
            Vec2::new(
                f.pos.x + (x * scale).mul_int(i32::from(f.facing)),
                f.pos.y + y * scale,
            )
        };
        // The field (a circle, or a standing capsule), and the body field beside it on its own frames.
        let field = frame >= rf.start
            && frame <= rf.end
            && shapes_touch(
                pos,
                shot.half_w,
                shot.half_h,
                shot.radius,
                at(rf.x, rf.y),
                Fx::ZERO,
                rf.half_height * scale,
                rf.radius * scale,
            );
        let body = rf.body_radius > Fx::ZERO
            && frame >= rf.body_start
            && frame <= rf.body_end
            && shapes_touch(
                pos,
                shot.half_w,
                shot.half_h,
                shot.radius,
                at(rf.body_x, rf.body_y),
                Fx::ZERO,
                Fx::ZERO,
                rf.body_radius * scale,
            );
        if field || body {
            return Some((i, rf));
        }
    }
    None
}

/// Whether two rounded rectangles touch (E38): each centred on `c`, its core reaching `w` ahead and behind and `h` above and below, of
/// radius `r` (a standing capsule when `w` is 0, a lying one when `h` is 0, a circle when both are). Their nearest points are the
/// gaps between the cores' spans on each axis (0 where they overlap): exact for these shapes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn shapes_touch(
    c1: Vec2,
    w1: Fx,
    h1: Fx,
    r1: Fx,
    c2: Vec2,
    w2: Fx,
    h2: Fx,
    r2: Fx,
) -> bool {
    let dx = ((c1.x - c2.x).abs() - w1 - w2).max(Fx::ZERO);
    let dy = ((c1.y - c2.y).abs() - h1 - h2).max(Fx::ZERO);
    let gap = Vec2::new(dx, dy);
    let reach = r1 + r2;
    gap.length_sq() <= reach * reach
}

/// A reflector's projectile detector (the reference's): a projectile touching it while it is out, before the reflector is up, skips
/// that fighter's move ahead so the reflector is up on the next frame.
pub(crate) fn reflector_detects(
    state: &mut GameState,
    content: &Content,
    owner: usize,
    pos: Vec2,
    shot: Shape,
) {
    for i in 0..crate::state::bodies(state) {
        let f = &state.fighters[i];
        if !rivals(owner, i) || f.state != S::Attack || f.hitlag > 0 {
            continue;
        }
        let Some(rf) = weapon_of(content, params_of(content, f))
            .get(f.move_id)
            .reflector
        else {
            continue;
        };
        let frame = f.state_frame;
        if rf.detect_end == 255
            || frame < u16::from(rf.detect_start)
            || frame > u16::from(rf.detect_end)
            || (rf.detect_then.is_none() && frame + 1 >= u16::from(rf.start))
        {
            continue;
        }
        let scale = params_of(content, f).hitbox_scale;
        let centre = Vec2::new(
            f.pos.x + (rf.detect_x * scale).mul_int(i32::from(f.facing)),
            f.pos.y + rf.detect_y * scale,
        );
        if shapes_touch(
            pos,
            shot.half_w,
            shot.half_h,
            shot.radius,
            centre,
            Fx::ZERO,
            Fx::ZERO,
            rf.detect_radius * scale,
        ) {
            match rf.detect_then {
                // The reflection move starts now (its frame 0 is this frame), so its reflector meets this shot at once.
                Some(next) => {
                    let w = weapon_of(content, params_of(content, f));
                    let then = w.get(next);
                    let invuln = if then.intangible_start(f.move_air) == 0 {
                        then.intangible
                    } else {
                        0
                    };
                    let f = &mut state.fighters[i];
                    fighter::begin_attack(f, MoveId::from_index(next));
                    f.invuln = f.invuln.max(invuln);
                }
                // (The next frame's update adds one.)
                None => state.fighters[i].state_frame = u16::from(rf.start) - 1,
            }
        }
    }
}

/// Sends fighters that have left the blast zone back to their spawn point, minus a stock. A fighter that loses its
/// last stock is out of the match (no longer `active`); with unlimited stocks nobody loses any.
pub fn check_ko(state: &mut GameState, content: &Content) {
    let s = &content.stage;
    // (Players only: a grunt is knocked out by the run's own zones, `run::grunts`.)
    for i in 0..MAX_FIGHTERS {
        if !state.fighters[i].active {
            continue;
        }
        let p = state.fighters[i].pos;
        if p.x < s.blast_left || p.x > s.blast_right || p.y < s.blast_bottom || p.y > s.blast_top {
            grab::drop_grab(state, content, i);
            // (An item in the hand is let go where the fighter left: E4; what it had put away is lost: E42.)
            crate::items::let_go(state, content, i);
            crate::items::lose_pocket(state, i);
            let counted = state.rules.stocks > 0;
            // (Who the KO is credited to, for a wing's points: E57.)
            let credit = crate::wing::before_ko(state, i);
            // (Orbs on the fighter go with it: E59.)
            crate::orbs::clear_target(state, i);
            let f = &mut state.fighters[i];
            respawn(f, content, i, counted);
            if counted && f.stocks == 0 {
                f.active = false;
            } else {
                // (A gauge that keeps some of itself through a KO, by the stocks now behind: E53.)
                crate::gauge::after_ko(state, content, i);
            }
            if let Some((killer, gap)) = credit {
                crate::wing::after_ko(state, content, killer, gap);
            }
        }
    }
}

pub fn respawn(f: &mut Fighter, content: &Content, index: usize, lose_stock: bool) {
    let params = params_of(content, f);
    let pos = content.stage.spawns[index];
    let platform = collision::standing_on(&content.stage, pos);
    let (char_id, facing) = (f.char_id, f.facing);
    // The controller does not forget what it was doing (a stick held through the knock-out stays "held").
    let history = f.history;
    let stocks = if lose_stock {
        f.stocks.saturating_sub(1)
    } else {
        f.stocks
    };
    *f = Fighter::spawn(
        pos,
        char_id,
        facing,
        platform,
        params.air_jumps,
        content.rules.shield_max,
    );
    f.stocks = stocks;
    f.active = true;
    f.history = history;
    f.invuln = content.rules.respawn_invuln;
    // Back on a revival platform above the spawn point.
    f.pos.y += content.rules.respawn_height;
    f.platform = NONE;
    f.state = S::Respawn;
    f.state_frame = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(n: i32) -> Fx {
        Fx::from_int(n)
    }

    fn close(a: Fx, expected_thousandths: i32) -> bool {
        (a - Fx::from_ratio(expected_thousandths, 1000)).abs() < Fx::from_ratio(1, 100)
    }

    #[test]
    fn knockback_matches_the_reference_formula() {
        // d = 10, p = 10 after the hit, w = 100, bkb 40, kbg 100:
        // ((1 + 5) * 1.0 * 1.4 + 18) * 1.0 + 40 = 66.4
        assert!(close(knockback(fx(10), fx(10), fx(100), 40, 100), 66_400));
        // A heavier target is launched less.
        assert!(
            knockback(fx(50), fx(10), fx(120), 40, 100)
                < knockback(fx(50), fx(10), fx(80), 40, 100)
        );
        // More percent, more knockback; more growth, more knockback.
        assert!(
            knockback(fx(100), fx(10), fx(90), 20, 100)
                > knockback(fx(20), fx(10), fx(90), 20, 100)
        );
        assert!(
            knockback(fx(50), fx(10), fx(90), 20, 120) > knockback(fx(50), fx(10), fx(90), 20, 80)
        );
    }

    #[test]
    fn hitlag_follows_the_reference_formula() {
        let r = Ruleset::standard();
        // floor(d * 0.65 + 6), at most 30.
        assert_eq!(hitlag_frames(fx(3), &r, false, false, Fx::ONE), 7);
        assert_eq!(hitlag_frames(fx(9), &r, false, false, Fx::ONE), 11);
        assert_eq!(hitlag_frames(fx(30), &r, false, false, Fx::ONE), 25);
        assert_eq!(hitlag_frames(fx(1000), &r, false, false, Fx::ONE), 30);
        assert_eq!(hitlag_frames(Fx::ZERO, &r, false, false, Fx::ONE), 0);
        // A shield takes a third off before rounding; a crouch a third off after.
        assert_eq!(hitlag_frames(fx(10), &r, true, false, Fx::ONE), 8);
        assert_eq!(hitlag_frames(fx(10), &r, false, true, Fx::ONE), 8);
        assert_eq!(hitlag_frames(fx(10), &r, false, false, Fx::ONE), 12);
    }

    #[test]
    fn shield_stun_is_the_reference_formula_with_whole_results_whole() {
        let s = |tenths: i32, kind: u16, own: u16| {
            shield_stun_frames(Fx::from_ratio(tenths, 10), kind, own)
        };
        // floor(0.8 d t m + 2): 10% -> 10 (was 9 in fixed point), 9% -> 9.2 -> 9, 15% -> 14.
        assert_eq!(s(100, STUN_NORMAL, 100), 10);
        assert_eq!(s(90, STUN_NORMAL, 100), 9);
        assert_eq!(s(150, STUN_NORMAL, 100), 14);
        // Smash attacks x0.725: 50% -> 0.58 x 50 + 2 = 31 exactly; 20% -> 13.6 -> 13.
        assert_eq!(s(500, STUN_SMASH, 100), 31);
        assert_eq!(s(200, STUN_SMASH, 100), 13);
        // Aerials x0.33: 25% -> 6.6 + 2 -> 8; 50% -> 13.2 + 2 -> 15.
        assert_eq!(s(250, STUN_AERIAL, 100), 8);
        assert_eq!(s(500, STUN_AERIAL, 100), 15);
        // A hitbox's own multiplier: 10% x1.5 -> 14; 0.6% x9 -> 6.32 -> 6.
        assert_eq!(s(100, STUN_NORMAL, 150), 14);
        assert_eq!(s(6, STUN_NORMAL, 900), 6);
        // Projectiles x0.29, unless the hitbox has its own multiplier: 25% -> 7.8 -> 7; with x2 -> 42.
        assert_eq!(s(250, STUN_PROJECTILE, 100), 7);
        assert_eq!(s(250, STUN_PROJECTILE, 200), 42);
        // A fresh move's 1.05 (in fixed point a hair under): 25 x 1.05 = 26.25 -> 21 + 2.
        let fresh = Fx::from_int(25) * Fx::from_ratio(105_000, 100_000);
        assert_eq!(shield_stun_frames(fresh, STUN_NORMAL, 100), 23);
        assert_eq!(shield_stun_frames(Fx::ZERO, STUN_NORMAL, 100), 2);
    }

    #[test]
    fn stale_moves_follow_the_reference_table() {
        let mut q = [0u8; STALE_QUEUE];
        assert_eq!(
            stale_factor(&q, 5),
            105_000,
            "a fresh move deals a little more"
        );
        push_stale(&mut q, 5, false);
        assert_eq!(stale_factor(&q, 5), 91_000);
        push_stale(&mut q, 5, false);
        assert_eq!(stale_factor(&q, 5), 100_000 - 9000 - 8545);
        // The wiki's worked example: slots 1, 2 and 6 give 77.42 percent.
        let mut q = [0u8; STALE_QUEUE];
        for key in [5u8, 9, 9, 9, 5, 5] {
            push_stale(&mut q, key, false);
        }
        assert_eq!(stale_factor(&q, 5), 77_420);
        // A full queue of one move is the most it can stale; a shielded hit counts 0.85 of its slot.
        let mut q = [0u8; STALE_QUEUE];
        for _ in 0..12 {
            push_stale(&mut q, 7, false);
        }
        assert_eq!(stale_factor(&q, 7), 46_950);
        let mut q = [0u8; STALE_QUEUE];
        push_stale(&mut q, 7, true);
        assert_eq!(stale_factor(&q, 7), 100_000 - 7650);
    }

    #[test]
    fn rage_adds_up_to_a_tenth_from_35_to_150_percent() {
        let r = Ruleset::standard();
        assert_eq!(rage_multiplier(fx(0), &r), Fx::ONE);
        assert_eq!(rage_multiplier(fx(35), &r), Fx::ONE);
        assert_eq!(
            rage_multiplier(fx(150), &r),
            Fx::ONE + Fx::from_ratio(1, 10)
        );
        assert_eq!(
            rage_multiplier(fx(400), &r),
            Fx::ONE + Fx::from_ratio(1, 10)
        );
        let mid = rage_multiplier(fx(92), &r);
        assert!(mid > Fx::ONE && mid < Fx::ONE + Fx::from_ratio(1, 10));
    }

    #[test]
    fn hitstun_scales_with_knockback_and_the_ruleset_multiplier() {
        let kb = Fx::from_ratio(664, 10);
        // floor(KB * 0.4) - 1.
        assert_eq!(hitstun_frames(kb, Fx::ONE), 25);
        assert_eq!(hitstun_frames(kb, Fx::from_ratio(105, 100)), 26);
        assert_eq!(
            hitstun_frames(fx(100), Fx::ONE),
            39,
            "the wiki's example: 100 knockback is 39 frames"
        );
        assert!(hitstun_frames(kb.mul_int(2), Fx::ONE) > hitstun_frames(kb, Fx::ONE));
    }

    #[test]
    fn launch_angles_mirror_with_facing_and_resolve_the_361_angle() {
        let a = |deg, facing, grounded, kb| launch_angle(deg, facing, grounded, fx(kb)).raw();
        assert_eq!(a(45, 1, false, 100), Angle::from_degrees(45).raw());
        assert_eq!(a(45, -1, false, 100), Angle::from_degrees(135).raw());
        assert_eq!(
            a(361, 1, true, 20),
            Angle::from_degrees(0).raw(),
            "weak grounded hit is horizontal"
        );
        // The reference's: grounded 0 up to 60 knockback, 38 from 88, evenly between (19 at 74); airborne always 38.
        assert_eq!(a(361, 1, true, 60), Angle::from_degrees(0).raw());
        assert_eq!(a(361, 1, true, 74), Angle::from_degrees(19).raw());
        assert_eq!(a(361, 1, true, 88), Angle::from_degrees(38).raw());
        assert_eq!(a(361, 1, true, 150), Angle::from_degrees(38).raw());
        assert_eq!(a(361, 1, false, 20), Angle::from_degrees(38).raw());
        assert_eq!(a(361, -1, false, 60), Angle::from_degrees(142).raw());
        // A sideways (0 degree) move rises to 32 from 120 knockback.
        assert_eq!(a(0, 1, true, 119), Angle::from_degrees(0).raw());
        assert_eq!(a(0, 1, false, 120), Angle::from_degrees(32).raw());
    }

    #[test]
    fn circles_overlap_when_the_gap_is_within_the_radii() {
        let o = |x: i32, r1: i32, r2: i32| {
            overlaps(
                Vec2::ZERO,
                Fx::from_ratio(r1, 10),
                Vec2::new(Fx::from_ratio(x, 10), Fx::ZERO),
                Fx::from_ratio(r2, 10),
            )
        };
        assert!(o(10, 6, 5));
        assert!(o(11, 6, 5), "touching counts");
        assert!(!o(12, 6, 5));
    }
}
