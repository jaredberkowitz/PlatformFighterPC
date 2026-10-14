//! Items (E4, sim 111; owner decisions 54 and 64): the universal pick-up, throws and drops every fighter has, the thrown hits, the
//! physics and bounces, knocking an item about, bursts, and the Buckler's bomb (the down special: pull, detonation, fuse, hit points,
//! self damage), its arrow items and its double arrow. Reference data: `docs/movesets/link/REFERENCE_DATA.md` ("Remote Bomb", "Bow and
//! Arrows", the item throw rows) and ssbwiki.com `Item_throw` (13.0.1).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, SPECIAL};
use sim_core::items::{
    self, THROW_BACK, THROW_DASH, THROW_DOWN, THROW_DROP, THROW_FORWARD, THROW_UP,
};
use sim_core::moves::buckler_slots::{
    ARROW_ITEM, BOMB_DETONATE, BOMB_ITEM, BOMB_KIND, BOW_FIRE, DOUBLE_FIRE, DOUBLE_SECOND,
};
use sim_core::moves::MoveId;
use sim_core::state::{
    FighterState as S, ITEM_BLAST, ITEM_FREE, ITEM_HELD, ITEM_LODGED, ITEM_THROWN, NONE,
};
use sim_core::trig::{self, Angle};
use sim_core::{step, Fx, GameState, Input, Item, Rng, Vec2, MAX_FIGHTERS};

const WANDERER: u8 = 9;
const DUELIST: u8 = 0;

/// Player 0 (`me`) at -3 facing right, player 1 (a wanderer, whose weapon makes the items) far to the right at 10 facing left, the
/// others out of the match.
fn sim_with(me: u8) -> Sim {
    sim_vs(me, WANDERER)
}

/// [`sim_with`] with player 1 of body `other` (player 0 must then be the wanderer that makes the items).
fn sim_vs(me: u8, other: u8) -> Sim {
    let mut sim = Sim::with_chars([me, other, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-3), 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

/// A wanderer's item (`spec`) lying on the stage's top at `x`, made by fighter `maker` (a wanderer). Returns its slot.
fn lying(sim: &mut Sim, maker: usize, spec: u8, x: Fx) -> usize {
    let n = items::make(
        &mut sim.state,
        &sim.content,
        maker,
        spec,
        Vec2::new(x, Fx::ZERO),
        MoveId::DownSpecial as u8,
    )
    .expect("a free slot");
    sim.state.items[n].platform = 0;
    n
}

/// Puts a new item of `maker`'s weapon (`spec`) in fighter `holder`'s hand. Returns its slot.
fn give(sim: &mut Sim, maker: usize, spec: u8, holder: usize) -> usize {
    let n = lying(sim, maker, spec, sim.state.fighters[holder].pos.x);
    let it = &mut sim.state.items[n];
    it.platform = NONE;
    it.holder = holder as i8;
    it.owner = holder as u8;
    it.rest = ITEM_HELD;
    sim.state.fighters[holder].holding = n as i8;
    n
}

fn live(sim: &Sim) -> Vec<Item> {
    sim.state
        .items
        .iter()
        .copied()
        .filter(|i| i.active)
        .collect()
}

fn tick(sim: &mut Sim, p0: Input) {
    sim.tick2(p0, inp(0, 0, 0));
}

fn idle(sim: &mut Sim, n: usize) {
    for _ in 0..n {
        tick(sim, inp(0, 0, 0));
    }
}

/// The frame (1 = the press) the item in `n` leaves player 0's hand, and the last frame of the throw (player 0 can act on the next:
/// the reference's total), for a press of `input` (held one frame).
fn throw_timing(sim: &mut Sim, n: usize, input: Input) -> (u32, u32) {
    let mut released = 0;
    for frame in 1..=80u32 {
        tick(sim, if frame == 1 { input } else { inp(0, 0, 0) });
        if released == 0 && sim.state.items[n].holder == NONE {
            released = frame;
        }
        if frame > 1 && sim.f().state != S::ItemThrow {
            return (released, frame);
        }
    }
    panic!("the throw never ended");
}

// ---- Picking up ----

#[test]
fn an_attack_or_grab_near_a_lying_item_picks_it_up_and_one_out_of_reach_is_a_jab() {
    for button in [ATTACK, GRAB] {
        let mut sim = sim_with(DUELIST);
        let n = lying(&mut sim, 1, BOMB_ITEM, Fx::from_int(-2));
        tick(&mut sim, inp(0, 0, button));
        assert_eq!(sim.f().state, S::ItemPickup, "{button}");
        assert_eq!(sim.f().holding, n as i8);
        assert_eq!(sim.state.items[n].holder, 0);
        assert_eq!(sim.state.items[n].rest, ITEM_HELD);
        // In hand from the first frame; free again after the ruleset's pick-up frames [E].
        let frames = u32::from(sim.content.rules.item_pick_frames);
        let mut free_on = 0;
        for f in 2..=20u32 {
            tick(&mut sim, inp(0, 0, 0));
            if sim.f().state != S::ItemPickup {
                free_on = f;
                break;
            }
        }
        assert_eq!(free_on, frames, "the last frame of the pick-up");
    }
    // Out of reach (6 units away), attack is the jab, as for every fighter.
    let mut sim = sim_with(DUELIST);
    lying(&mut sim, 1, BOMB_ITEM, Fx::from_int(3));
    tick(&mut sim, inp(0, 0, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::Jab as u8);
    assert_eq!(sim.f().holding, NONE);
}

#[test]
fn a_smash_input_near_an_item_is_the_smash_attack_not_a_pick_up() {
    let mut sim = sim_with(DUELIST);
    lying(&mut sim, 1, BOMB_ITEM, Fx::from_int(-2));
    // (A flick forward with attack, from the stick in the middle.)
    tick(&mut sim, inp(127, 0, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::FSmash as u8);
    assert_eq!(sim.f().holding, NONE);
}

#[test]
fn the_reach_is_13_5_ahead_and_6_8_behind_reference_units() {
    // The fighter at -3 facing right: an item whose pick-up box (4 reference units half wide) reaches 1.6875 ahead is caught, one
    // whose box starts further is not; behind, 0.848.
    let half = fx(4, 8);
    for (x, caught) in [
        (fx(-3 * 8 + 135 / 10, 8) + half - fx(1, 100), true),
        (Fx::from_int(-3) + fx(135, 80) + half + fx(1, 100), false),
        (Fx::from_int(-3) - fx(95, 112) - half + fx(1, 100), true),
        (Fx::from_int(-3) - fx(95, 112) - half - fx(1, 100), false),
    ] {
        let mut sim = sim_with(DUELIST);
        lying(&mut sim, 1, BOMB_ITEM, x);
        tick(&mut sim, inp(0, 0, GRAB));
        assert_eq!(sim.f().holding != NONE, caught, "item at {x:?}");
    }
}

// ---- Throwing ----

#[test]
fn the_wanderers_item_throws_let_go_on_7_9_11_6_4_5_and_last_24_24_23_19_29_23() {
    // (Control returns on the frame after the total: the reference's totals.)
    for (input, kind, release, total) in [
        (inp(0, 0, ATTACK), THROW_FORWARD, 7, 24),
        (inp(-127, 0, ATTACK), THROW_BACK, 9, 24),
        (inp(0, 127, ATTACK), THROW_UP, 11, 23),
        (inp(0, -127, ATTACK), THROW_DOWN, 6, 19),
        (inp(0, 0, GRAB), THROW_DROP, 5, 23),
    ] {
        let mut sim = sim_with(WANDERER);
        let n = give(&mut sim, 0, BOMB_ITEM, 0);
        let (out, free) = throw_timing(&mut sim, n, input);
        assert_eq!(sim.f().item_throw & 0x7f, kind);
        assert_eq!((out, free), (release, total), "throw {kind}");
    }
    // The dash throw, out of a run.
    let mut sim = sim_with(WANDERER);
    let n = give(&mut sim, 0, BOMB_ITEM, 0);
    for _ in 0..20 {
        tick(&mut sim, inp(127, 0, 0));
    }
    assert_eq!(sim.f().state, S::Run);
    let (out, free) = throw_timing(&mut sim, n, inp(127, 0, ATTACK));
    assert_eq!(sim.f().item_throw, THROW_DASH);
    assert_eq!((out, free), (4, 29));
}

#[test]
fn another_body_lets_go_on_its_own_frames() {
    // The duelist's forward throw lets go on 10 (its reference character's), with the same 24-frame length.
    let mut sim = sim_with(DUELIST);
    let n = give(&mut sim, 1, BOMB_ITEM, 0);
    assert_eq!(throw_timing(&mut sim, n, inp(0, 0, ATTACK)), (10, 24));
}

#[test]
fn a_thrown_bomb_flies_at_the_references_speed() {
    // Forward: 2.6 x the toss strength 1.01333 (weight 104), at 17 degrees, sideways x0.6264 twice, upward once; smash: 3.64 at 11.
    // (Attack with the stick in the middle is the forward throw; a flick forward with it the smash throw.)
    for (input, speed, degrees) in [
        (inp(0, 0, ATTACK), 2600, 17),
        (inp(127, 0, ATTACK), 3640, 11),
    ] {
        let mut sim = sim_with(WANDERER);
        let n = give(&mut sim, 0, BOMB_ITEM, 0);
        tick(&mut sim, input);
        idle(&mut sim, 6);
        let it = sim.state.items[n];
        assert_eq!(it.rest, ITEM_THROWN);
        let t = fx(speed, 8000) * (Fx::ONE + fx(4, 300));
        let m = fx(6264, 10000);
        let a = Angle::from_degrees(degrees);
        let vx = t * trig::cos(a) * m * m;
        // (Its own gravity, 0.07, has pulled it once.)
        let vy = t * trig::sin(a) * m - fx(70, 8000);
        let near = |a: Fx, b: Fx| (a - b).abs() < fx(2, 1000);
        assert!(
            near(it.vel.x, vx) && near(it.vel.y, vy),
            "{:?} vs {vx:?} {vy:?}",
            it.vel
        );
    }
}

#[test]
fn the_grounded_drop_goes_up_and_back_and_the_aerial_drop_lets_go_at_once() {
    let mut sim = sim_with(WANDERER);
    let n = give(&mut sim, 0, BOMB_ITEM, 0);
    throw_timing(&mut sim, n, inp(0, 0, GRAB));
    let it = sim.state.items[n];
    assert!(
        it.vel.x < Fx::ZERO || it.platform != NONE,
        "behind: 100 degrees"
    );
    // In the air: no speed, no lag; the fighter goes on falling.
    let mut sim = sim_with(WANDERER);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    let n = give(&mut sim, 0, BOMB_ITEM, 0);
    tick(&mut sim, inp(0, 0, GRAB));
    assert_eq!(sim.f().state, S::Airborne);
    assert_eq!(sim.f().holding, NONE);
    let it = sim.state.items[n];
    assert_eq!(it.rest, ITEM_THROWN);
    assert_eq!(it.vel.x, Fx::ZERO);
    assert!(it.vel.y <= Fx::ZERO);
}

#[test]
fn an_aerial_attack_while_holding_is_the_aerial_throw() {
    let mut sim = sim_with(WANDERER);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    let n = give(&mut sim, 0, BOMB_ITEM, 0);
    let (out, _) = throw_timing(&mut sim, n, inp(0, 127, ATTACK));
    assert_eq!(out, 11, "the up throw's release");
    assert!(sim.state.items[n].vel.y > Fx::ZERO);
}

#[test]
fn a_thrown_bomb_hits_for_1_percent_and_bounces_off_and_a_thrown_arrow_vanishes() {
    for spec in [BOMB_ITEM, ARROW_ITEM] {
        // (A duelist: the wanderer's passive shield would stop the item.)
        let mut sim = sim_vs(WANDERER, DUELIST);
        sim.stand(1, fx(-1, 2), -1);
        let n = give(&mut sim, 0, spec, 0);
        let mut hit_on = None;
        for frame in 1..40 {
            tick(
                &mut sim,
                if frame == 1 {
                    inp(0, 0, ATTACK)
                } else {
                    inp(0, 0, 0)
                },
            );
            if sim.fighter(1).percent > Fx::ZERO {
                hit_on = Some(frame);
                break;
            }
        }
        assert!(hit_on.is_some(), "item {spec} hits");
        let mult = sim.content.rules.damage_mult;
        if spec == BOMB_ITEM {
            assert_eq!(sim.fighter(1).percent, Fx::ONE * mult, "1% flat");
            let it = sim.state.items[n];
            assert!(
                it.active && it.rest == ITEM_FREE,
                "it bounces off, harmless"
            );
            assert!(it.vel.x <= Fx::ZERO, "back the other way");
        } else {
            // 1 - 0.4 + 0.8 x the throw's speed (2.6 x 1.01333 reference units).
            let d = fx(6, 10) + fx(8, 10) * fx(26, 10) * (Fx::ONE + fx(4, 300));
            let got = sim.fighter(1).percent / mult;
            assert!((got - d).abs() < fx(1, 100), "{got:?} vs {d:?}");
            assert!(!sim.state.items[n].active, "the arrow is gone");
        }
    }
}

#[test]
fn a_held_item_falls_when_its_holder_is_hit_and_the_hand_stays_matched() {
    let mut sim = sim_with(WANDERER);
    sim.stand(1, fx(1, 1), -1);
    let n = give(&mut sim, 0, BOMB_ITEM, 0);
    // The duelist jabs the holder.
    for _ in 0..12 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.f().percent > Fx::ZERO {
            break;
        }
    }
    assert!(sim.f().percent > Fx::ZERO);
    idle(&mut sim, 1);
    assert_eq!(sim.f().holding, NONE);
    assert_eq!(sim.state.items[n].holder, NONE);
    assert_eq!(sim.state.items[n].rest, ITEM_FREE);
}

#[test]
fn specials_jumps_and_the_shield_keep_the_item_in_hand() {
    let mut sim = sim_with(WANDERER);
    let n = give(&mut sim, 0, BOMB_ITEM, 0);
    tick(&mut sim, inp(0, 0, sim_core::input::buttons::SHIELD));
    idle(&mut sim, 20);
    tick(&mut sim, inp(0, 0, sim_core::input::buttons::JUMP));
    idle(&mut sim, 10);
    assert_eq!(sim.f().holding, n as i8);
    assert_eq!(sim.state.items[n].rest, ITEM_HELD);
}

#[test]
fn the_passive_shield_is_off_while_holding_an_item() {
    // A blaster shot from the claws (player 1) at a standing wanderer: blocked empty-handed, taken holding an item (E14, E4).
    for holding in [false, true] {
        let mut sim = Sim::with_chars([WANDERER, 1, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_int(6), -1);
        if holding {
            give(&mut sim, 0, ARROW_ITEM, 0);
        }
        for f in 0..60 {
            sim.tick2(inp(0, 0, 0), inp(0, 0, if f == 0 { SPECIAL } else { 0 }));
        }
        assert_eq!(sim.f().percent > Fx::ZERO, holding, "holding {holding}");
    }
}

// ---- Physics ----

#[test]
fn a_dropped_bomb_bounces_off_the_floor_and_comes_to_rest() {
    let mut sim = sim_with(WANDERER);
    let n = lying(&mut sim, 0, BOMB_ITEM, Fx::from_int(4));
    sim.state.items[n].platform = NONE;
    // (Under the soft platform at 3.6 there.)
    sim.state.items[n].pos.y = Fx::from_int(3);
    let mut bounced = false;
    let mut last_vy = Fx::ZERO;
    for _ in 0..400 {
        idle(&mut sim, 1);
        let it = sim.state.items[n];
        if last_vy < Fx::ZERO && it.vel.y > Fx::ZERO {
            bounced = true;
            // Off the floor at x0.7 of the fall, held within 0.1-2.3 reference units.
            assert!(it.vel.y <= fx(2300, 8000) && it.vel.y >= fx(100, 8000));
        }
        last_vy = it.vel.y;
    }
    assert!(bounced, "it bounced");
    let it = sim.state.items[n];
    assert!(
        it.platform != NONE && it.pos.y == Fx::ZERO,
        "it lies on the stage: {it:?}"
    );
}

// ---- The bomb ----

/// Presses down special once (stick down) and runs `n` frames in all.
fn down_special(sim: &mut Sim, n: usize) {
    for f in 0..n {
        tick(
            sim,
            if f == 0 {
                inp(0, -127, SPECIAL)
            } else {
                inp(0, 0, 0)
            },
        );
    }
}

#[test]
fn the_bomb_is_pulled_into_the_hand_on_frame_17_and_control_returns_on_40() {
    let mut sim = sim_with(WANDERER);
    down_special(&mut sim, 16);
    assert!(live(&sim).is_empty(), "nothing before frame 17");
    down_special_rest(&mut sim, 1);
    let b = live(&sim);
    assert_eq!(b.len(), 1);
    assert_eq!(
        (b[0].kind, b[0].rest, b[0].holder),
        (BOMB_KIND, ITEM_HELD, 0)
    );
    assert_eq!(b[0].life, 1800);
    assert_eq!(b[0].hp, Fx::from_int(50));
    // FAF 40: the move's last frame is 39 (it can act on 40).
    down_special_rest(&mut sim, 21);
    assert_eq!(sim.f().state, S::Attack);
    down_special_rest(&mut sim, 1);
    assert_eq!(sim.f().state, S::Idle);
}

fn down_special_rest(sim: &mut Sim, n: usize) {
    idle(sim, n);
}

#[test]
fn no_bomb_is_pulled_while_an_item_is_held() {
    let mut sim = sim_with(WANDERER);
    let arrow = give(&mut sim, 0, ARROW_ITEM, 0);
    down_special(&mut sim, 40);
    assert_eq!(live(&sim).len(), 1);
    assert_eq!(sim.f().holding, arrow as i8);
}

#[test]
fn pressing_again_detonates_the_bomb_on_frames_12_and_13_hitting_the_wanderer_too() {
    // A bomb in hand: the press is the detonation (FAF 40); the blast (7%, BK 60, KS 115, radius 14) is out on the press's frames 12
    // and 13 and hits the wanderer himself (self damage) and a duelist beside him.
    let mut sim = sim_with(WANDERER);
    sim.stand(1, fx(-2, 1), -1);
    give(&mut sim, 0, BOMB_ITEM, 0);
    let mult = sim.content.rules.damage_mult;
    let mut blast_on = None;
    for frame in 1..=14u32 {
        tick(
            &mut sim,
            if frame == 1 {
                inp(0, -127, SPECIAL)
            } else {
                inp(0, 0, 0)
            },
        );
        if frame == 2 {
            assert_eq!(sim.f().move_id, BOMB_DETONATE as u8);
        }
        if blast_on.is_none() && sim.f().percent > Fx::ZERO {
            blast_on = Some(frame);
        }
    }
    assert_eq!(blast_on, Some(12));
    assert_eq!(sim.f().percent, Fx::from_int(7) * mult, "self damage");
    assert_eq!(sim.fighter(1).percent, Fx::from_int(7) * mult);
    assert!(live(&sim).is_empty(), "gone after its two frames");
    assert_eq!(sim.f().holding, NONE);
}

#[test]
fn a_lying_bomb_is_detonated_from_afar_and_one_an_opponent_holds_is_not() {
    for opponent_holds in [false, true] {
        let mut sim = sim_with(WANDERER);
        sim.stand(1, Fx::from_int(5), -1);
        let n = if opponent_holds {
            give(&mut sim, 0, BOMB_ITEM, 1)
        } else {
            lying(&mut sim, 0, BOMB_ITEM, fx(55, 10))
        };
        down_special(&mut sim, 14);
        let it = sim.state.items[n];
        if opponent_holds {
            assert!(
                it.active && it.rest == ITEM_HELD,
                "held by the duelist: no blast"
            );
            assert_eq!(sim.fighter(1).percent, Fx::ZERO);
            assert_eq!(
                sim.f().move_id,
                BOMB_DETONATE as u8,
                "the press is still the detonation"
            );
        } else {
            assert!(
                sim.fighter(1).percent > Fx::ZERO,
                "the blast reached the duelist"
            );
            assert_eq!(sim.f().percent, Fx::ZERO, "too far to hurt the wanderer");
        }
    }
}

#[test]
fn the_bomb_bursts_at_the_end_of_its_1800_frame_fuse() {
    let mut sim = sim_with(WANDERER);
    let n = lying(&mut sim, 0, BOMB_ITEM, Fx::from_int(4));
    sim.state.items[n].age = 1798;
    idle(&mut sim, 1);
    assert_eq!(sim.state.items[n].rest, ITEM_FREE);
    idle(&mut sim, 1);
    assert_eq!(sim.state.items[n].rest, ITEM_BLAST);
}

#[test]
fn hits_knock_the_bomb_about_and_take_its_50_hp() {
    // A down smash (a smash input: near an item any other attack would pick it up) knocks a lying bomb away (its 1 to 3.2 speed for
    // 1% to 15%) and takes its damage off the 50 hp; at 0 it bursts [E: gap 11].
    let mut sim = sim_with(WANDERER);
    sim.stand(1, Fx::from_int(2), -1);
    let n = lying(&mut sim, 0, BOMB_ITEM, fx(9, 10));
    sim.tick2(inp(0, 0, 0), inp(0, -127, ATTACK));
    for _ in 0..30 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.state.items[n].hp < Fx::from_int(50) {
            break;
        }
    }
    let it = sim.state.items[n];
    assert!(it.hp < Fx::from_int(50), "it was hit");
    assert!(
        it.vel.x < Fx::ZERO || it.hitlag > 0,
        "sent away from the duelist"
    );
    // Down to its last hp, the next hit bursts it.
    let mut sim = sim_with(WANDERER);
    sim.stand(1, Fx::from_int(2), -1);
    let n = lying(&mut sim, 0, BOMB_ITEM, fx(9, 10));
    sim.state.items[n].hp = fx(1, 10);
    sim.tick2(inp(0, 0, 0), inp(0, -127, ATTACK));
    idle(&mut sim, 40);
    assert!(
        sim.fighter(1).percent > Fx::ZERO,
        "the blast hit the duelist"
    );
}

#[test]
fn an_opponent_picks_up_the_bomb_and_throws_it_as_its_own() {
    let mut sim = sim_vs(WANDERER, DUELIST);
    sim.stand(1, Fx::from_int(4), -1);
    let n = lying(&mut sim, 0, BOMB_ITEM, fx(35, 10));
    sim.tick2(inp(0, 0, 0), inp(0, 0, GRAB));
    idle(&mut sim, 10);
    assert_eq!(sim.fighter(1).holding, n as i8);
    assert_eq!(sim.state.items[n].owner, 1);
    assert_eq!(sim.state.items[n].origin, 0, "still the wanderer's bomb");
    // Its thrown hit is the duelist's: it hits the wanderer (turned away from it: his passive shield stops shots from in front).
    sim.stand(0, fx(25, 10), -1);
    for f in 0..30 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, if f == 0 { ATTACK } else { 0 }));
    }
    assert!(sim.f().percent > Fx::ZERO);
}

// ---- The arrows ----

#[test]
fn a_lodged_arrow_is_picked_up_and_the_bow_then_fires_the_double_arrow() {
    let mut sim = sim_with(WANDERER);
    let arrow = lying(&mut sim, 0, ARROW_ITEM, fx(-25, 10));
    sim.state.items[arrow].rest = ITEM_LODGED;
    sim.state.items[arrow].life = 180;
    tick(&mut sim, inp(0, 0, ATTACK));
    assert_eq!(sim.f().holding, arrow as i8);
    assert_eq!(
        sim.state.items[arrow].life, 0,
        "in hand it lasts until used"
    );
    idle(&mut sim, 10);
    // A tap of the bow: the first arrow on 16, the second 3 frames later, the arrow in hand used up.
    for _ in 0..15 {
        tick(&mut sim, inp(0, 0, SPECIAL));
    }
    tick(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, DOUBLE_FIRE as u8);
    let shots = |sim: &Sim| sim.state.projectiles.iter().filter(|p| p.active).count();
    assert_eq!(shots(&sim), 1);
    idle(&mut sim, 2);
    assert_eq!(shots(&sim), 1);
    assert!(live(&sim).is_empty(), "the arrow in hand is used up");
    assert_eq!(sim.f().holding, NONE);
    idle(&mut sim, 1);
    assert_eq!(shots(&sim), 2);
    assert_eq!(sim.f().move_id, DOUBLE_SECOND as u8);
    // Each floor(0.8 x 4) = 3%, BK 25, KS 110; lodging 120 and 30.
    let w = &sim.content.weapons[usize::from(sim.content.fighters[usize::from(WANDERER)].weapon)];
    for (slot, lodge) in [(DOUBLE_FIRE, 120), (DOUBLE_SECOND, 30)] {
        let p = w.get(slot as u8).projectile.expect("a shot");
        assert_eq!(
            (p.hitbox.base_knockback, p.hitbox.knockback_growth),
            (25, 110)
        );
        assert_eq!(p.arc.lodge, lodge);
    }
    let s: Vec<_> = sim.state.projectiles.iter().filter(|p| p.active).collect();
    assert!(s.iter().all(|p| p.damage == Fx::from_int(3)));
    // The second at x0.9 the speed.
    let (a, b) = (s[0].vel.x.max(s[1].vel.x), s[0].vel.x.min(s[1].vel.x));
    assert!((b * Fx::from_int(10) - a * Fx::from_int(9)).abs() < fx(1, 100));
    // Control back when a single arrow's release gives it back (30 frames after the release: FAF 45 for a tap) [E].
    let mut last = 3;
    while sim.f().state == S::Attack {
        idle(&mut sim, 1);
        last += 1;
    }
    let mut single = sim_with(WANDERER);
    for _ in 0..15 {
        tick(&mut single, inp(0, 0, SPECIAL));
    }
    tick(&mut single, inp(0, 0, 0));
    assert_eq!(single.f().move_id, BOW_FIRE as u8);
    let mut single_last = 0;
    while single.f().state == S::Attack {
        idle(&mut single, 1);
        single_last += 1;
    }
    assert_eq!(last, single_last, "frames after the release");
}

#[test]
fn the_double_arrows_damage_is_floor_0_8_of_the_single_arrows() {
    // 4 → 3, 5 → 4, 6 → 4, 7 → 5, 8 → 6, 12 → 9: the single's floor(4 + 8c / 60) at c = 0, 8, 15, 23, 30 and full (36 held).
    for (held, single, double) in [
        (0, 4, 3),
        (8, 5, 4),
        (15, 6, 4),
        (23, 7, 5),
        (30, 8, 6),
        (36, 12, 9),
    ] {
        let mut sim = sim_with(WANDERER);
        give(&mut sim, 0, ARROW_ITEM, 0);
        for _ in 0..15 + held {
            tick(&mut sim, inp(0, 0, SPECIAL));
        }
        tick(&mut sim, inp(0, 0, 0));
        let p = sim
            .state
            .projectiles
            .iter()
            .find(|p| p.active)
            .copied()
            .expect("an arrow");
        assert_eq!(
            p.damage,
            Fx::from_int(double),
            "held {held} (single {single})"
        );
    }
}

// ---- Determinism ----

#[test]
fn items_resimulate_exactly_from_a_snapshot() {
    // Random play by two wanderers and two duelists with bombs lying about: a snapshot taken mid-way and replayed gives the same
    // checksum every frame.
    let mut content = sim_core::Content::placeholder();
    content.rules.stale_moves = 0;
    let mut state = GameState::new(&content, 3, [WANDERER, DUELIST, WANDERER, DUELIST]);
    let mut rng = Rng::new(17);
    let inputs = sim_core::fuzz::random_inputs(&mut rng, 900);
    for (i, x) in [(0usize, -4), (2, 4)] {
        let n = items::make(
            &mut state,
            &content,
            i,
            BOMB_ITEM,
            Vec2::new(Fx::from_int(x), Fx::ZERO),
            16,
        )
        .expect("slot");
        state.items[n].platform = 0;
    }
    let mut saved = None;
    let mut sums = Vec::new();
    for (f, inp) in inputs.iter().enumerate() {
        if f == 300 {
            saved = Some(state);
        }
        step(&mut state, &content, inp);
        if f >= 300 {
            sums.push(state.checksum());
        }
    }
    let mut again = saved.expect("snapshot");
    for (k, inp) in inputs[300..].iter().enumerate() {
        step(&mut again, &content, inp);
        assert_eq!(again.checksum(), sums[k], "frame {}", 300 + k);
    }
    let _: [Input; MAX_FIGHTERS] = inputs[0];
}
