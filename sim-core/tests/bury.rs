//! The bury (E5, sim 90): a hit with `EFFECT_BURY` on a fighter standing on the ground (not lying down) buries it
//! (`FighterState::Buried`) for the reference's `ceil(55 + 0.5p + 1.5k)` frames (p its percent after the hit, k the hit's knockback at
//! weight 100), plus 8 / 12 frames when the attacker is one / two stocks behind and minus 18 / 22 when it is ahead; mashing shortens it
//! as it does a hold; it cannot be grabbed; at the end it comes out with 12 frames of lag (no invincibility), or is launched by the
//! knockback a weaker hit left meanwhile; a hit of at least 90 knockback frees it at once and launches it in full. An airborne fighter
//! takes a bury hit as an ordinary hit. A bury hit ignores subtractive knockback armor. The hauler's headbutt is the bury move here.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::{body_hurtboxes, bury_frames, knockback};
use sim_core::moves::{Armor, MoveId, ARMOR_KNOCKBACK, EFFECT_BURY};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, Fx, Input, Rng, MAX_FIGHTERS};

const HAULER: u8 = 7;

/// A hauler (player 0) one frame before its grounded headbutt's hit (frame 20) and a duelist (player 1) standing in front of it.
fn headbutt() -> Sim {
    let mut sim = Sim::with_chars([HAULER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(16, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::SideSpecial);
    sim.state.fighters[0].state_frame = 18;
    sim
}

/// Ticks until player 1's percent rises (at most 10).
fn land(sim: &mut Sim, p1: Input) {
    let before = sim.fighter(1).percent;
    for _ in 0..10 {
        sim.tick2(inp(0, 0, 0), p1);
        if sim.fighter(1).percent > before {
            return;
        }
    }
    panic!("the headbutt never hit");
}

#[test]
fn the_bury_lasts_the_references_formula() {
    let rules = Content::placeholder().rules;
    // At 0%: the headbutt's 10% (12 taken after the one-on-one multiplier, which the knockback does not see: sim 97), BK 40, KS 40
    // at weight 100.
    let k = knockback(fx(10, 1), Fx::from_int(10), Fx::from_int(100), 40, 40);
    let even = bury_frames(&rules, Fx::from_int(12), k, 3, 3);
    assert_eq!(even, 137, "ceil(55 + 6 + 1.5 x 50.56)");
    // Stock differences: the attacker behind by one / two (+8 / +12), ahead by one / two (-18 / -22).
    assert_eq!(bury_frames(&rules, Fx::from_int(12), k, 2, 3), 145);
    assert_eq!(bury_frames(&rules, Fx::from_int(12), k, 1, 3), 149);
    assert_eq!(bury_frames(&rules, Fx::from_int(12), k, 3, 2), 119);
    assert_eq!(bury_frames(&rules, Fx::from_int(12), k, 3, 1), 115);
    // At 100% it is longer (the percent term and the knockback both grow).
    let k100 = knockback(fx(110, 1), Fx::from_int(10), Fx::from_int(100), 40, 40);
    assert!(bury_frames(&rules, Fx::from_int(112), k100, 3, 3) > even + 50);
}

#[test]
fn a_grounded_headbutt_buries_for_the_formulas_frames_whatever_the_victims_weight() {
    let mut sim = headbutt();
    land(&mut sim, inp(0, 0, 0));
    let f = *sim.fighter(1);
    assert_eq!(f.state, S::Buried);
    assert_eq!(f.percent, Fx::from_int(10) * sim.content.rules.damage_mult);
    assert_eq!(f.bury_timer, 137);
    assert!(!f.launch_pending && f.hitlag > 0);
    // A heavier victim (the hauler itself) is buried exactly as long: the bury is weight-independent.
    let mut sim = headbutt();
    sim.state.fighters[1].char_id = HAULER;
    land(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).bury_timer, 137);
}

/// After the bury hit's hitlag, the frames player 1 (holding `p1(tick)`) spends buried, and what follows: (buried frames, state
/// after, frames of that state until it is free).
fn ride_out(sim: &mut Sim, p1: impl Fn(u32) -> Input) -> (u32, S, u32) {
    while sim.fighter(1).hitlag > 0 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    let mut buried = 0;
    let mut t = 0;
    while sim.fighter(1).state == S::Buried {
        t += 1;
        sim.tick2(inp(0, 0, 0), p1(t));
        buried += 1;
        assert!(buried < 400);
    }
    let after = sim.fighter(1).state;
    let mut lag = 0;
    while sim.fighter(1).state == after && lag < 200 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        lag += 1;
    }
    (buried, after, lag)
}

#[test]
fn the_buried_fighter_comes_out_by_itself_with_12_frames_of_lag() {
    let mut sim = headbutt();
    land(&mut sim, inp(0, 0, 0));
    let (buried, after, lag) = ride_out(&mut sim, |_| inp(0, 0, 0));
    assert_eq!(buried, 137);
    assert_eq!(after, S::Rebound);
    assert_eq!(lag, 12);
    assert_eq!(sim.fighter(1).invuln, 0, "no invincibility on the way out");
}

#[test]
fn mashing_shortens_the_bury_as_it_does_a_hold() {
    // A button pressed every other frame takes 14 more each press (the grab's `grab_mash_button`).
    let mut sim = headbutt();
    land(&mut sim, inp(0, 0, 0));
    let (buried, ..) = ride_out(&mut sim, |t| {
        inp(
            0,
            0,
            if t % 2 == 1 {
                sim_core::input::buttons::ATTACK
            } else {
                0
            },
        )
    });
    // Each pair of frames takes 2 + 14 = 16: 137 frames go in 9 pairs (144 > 137), the last on the 17th frame.
    assert_eq!(buried, 17);
}

#[test]
fn a_buried_fighter_cannot_be_grabbed_and_a_lying_one_cannot_be_buried() {
    let mut sim = headbutt();
    land(&mut sim, inp(0, 0, 0));
    assert!(!sim_core::grab::grabbable(sim.fighter(1)));
    assert!(!sim_core::grab::catchable(sim.fighter(1)));
    // Lying down after a knockdown: the headbutt hits it as an ordinary hit.
    let mut sim = headbutt();
    sim.state.fighters[1].state = S::Knockdown;
    land(&mut sim, inp(0, 0, 0));
    assert_ne!(sim.fighter(1).state, S::Buried);
    // A tripped fighter (sitting) can be buried.
    let mut sim = headbutt();
    sim.state.fighters[1].state = S::Knockdown;
    sim.state.fighters[1].tripped = true;
    land(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).state, S::Buried);
}

#[test]
fn an_airborne_target_takes_the_aerial_headbutt_as_a_meteor() {
    let mut sim = headbutt();
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Ext5);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].state = S::Attack;
    sim.state.fighters[0].state_frame = 18;
    sim.put_airborne(1, fx(10, 10), fx(198, 10), Fx::ZERO, Fx::ZERO);
    land(&mut sim, inp(0, 0, 0));
    let f = *sim.fighter(1);
    assert_eq!(f.state, S::Hitstun);
    assert!(f.launch_pending);
    let a = sim_core::trig::Angle::from_raw(f.launch_angle);
    assert!(
        sim_core::trig::sin(a) < Fx::from_ratio(-99, 100),
        "straight down"
    );
}

/// Player 1 buried by the headbutt, then (after the hitlag) hit by a test box of `bk` base knockback (no growth) at `angle`.
fn buried_then_hit(bk: i16, angle: i16) -> Sim {
    let mut sim = headbutt();
    land(&mut sim, inp(0, 0, 0));
    while sim.fighter(1).hitlag > 0 || sim.f().hitlag > 0 {
        sim.tick(inp(0, 0, 0));
    }
    // The hauler's jab made into the test hit.
    let w = usize::from(sim.content.fighters[usize::from(HAULER)].weapon);
    for h in &mut sim.content.weapons[w].moves[MoveId::Jab as usize].hitboxes {
        h.base_knockback = bk;
        h.knockback_growth = 0;
        h.angle = angle;
        h.angle_type = 0;
        h.trip = 0;
        h.y = fx(5, 10);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
    let before = sim.fighter(1).percent;
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            break;
        }
    }
    assert!(sim.fighter(1).percent > before, "the second hit landed");
    sim
}

#[test]
fn a_hit_of_90_knockback_frees_and_launches_a_buried_fighter() {
    let sim = buried_then_hit(95, 45);
    let f = sim.fighter(1);
    assert_eq!(f.state, S::Hitstun);
    assert_eq!(f.launch_kb, Fx::from_int(95));
    assert_eq!((f.bury_timer, f.bury_set), (0, false));
}

#[test]
fn a_weaker_hit_is_kept_and_launches_the_fighter_when_the_bury_ends() {
    let mut sim = buried_then_hit(60, 45);
    let f = *sim.fighter(1);
    assert_eq!(f.state, S::Buried);
    assert!(f.bury_set);
    assert_eq!(f.bury_kb, Fx::from_int(60));
    // Out of the bury: launched with that knockback, not the 12-frame exit.
    for _ in 0..300 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state != S::Buried {
            break;
        }
    }
    let f = sim.fighter(1);
    assert_eq!(f.state, S::Hitstun);
    assert_eq!(f.launch_kb, Fx::from_int(60));
    assert!(f.kb_vel.y > Fx::ZERO && !f.bury_set);
}

#[test]
fn a_buried_fighters_hurtboxes_sit_half_its_height_lower() {
    let mut sim = headbutt();
    let content = sim.content.clone();
    let standing = body_hurtboxes(&content, sim.fighter(1));
    land(&mut sim, inp(0, 0, 0));
    let buried = body_hurtboxes(&content, sim.fighter(1));
    let sunk = content.fighters[0].ecb_height * content.rules.bury_depth;
    // (The middle and high circles; the guard circle of a standing fighter moves too.)
    for i in 0..3 {
        assert_eq!(buried[i].0.y, standing[i].0.y - sunk, "circle {i}");
    }
}

#[test]
fn a_bury_hit_ignores_subtractive_knockback_armor_but_not_other_armor() {
    // The duelist in a jab with knockback armor through the whole move: subtractive, the bury goes through; non-subtractive (it
    // withstands the 51-unit hit), the duelist keeps attacking.
    for (subtract, buried) in [(true, true), (false, false)] {
        let mut sim = headbutt();
        let mv = &mut sim.content.weapons[0].moves[MoveId::Jab as usize];
        mv.total_frames = 60;
        mv.hitboxes.clear();
        mv.armor = Some(Armor {
            start: 0,
            end: 59,
            kind: ARMOR_KNOCKBACK,
            threshold: Fx::from_int(200),
            subtract,
            min_kb: Fx::ZERO,
        });
        // (A move with no hitboxes needs nothing more in a test.)
        sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::Jab);
        land(&mut sim, inp(0, 0, 0));
        assert_eq!(
            sim.fighter(1).state == S::Buried,
            buried,
            "subtract {subtract}"
        );
    }
    let _ = EFFECT_BURY;
}

#[test]
fn a_bury_rolls_back_and_replays_the_same() {
    // The headbutt buries the duelist, who mashes at random; a snapshot taken mid-bury and replayed gives the same frames.
    let mut sim = headbutt();
    let mut rng = Rng::new(9);
    let inputs: Vec<[Input; MAX_FIGHTERS]> = (0..240)
        .map(|_| {
            let mut i = [Input::default(); MAX_FIGHTERS];
            i[1] = inp(
                (rng.next_u32() % 255) as i8,
                (rng.next_u32() % 255) as i8,
                (rng.next_u32() & 0b1111) as u16,
            );
            i
        })
        .collect();
    let mut saved = None;
    let mut buried_seen = false;
    for (t, i) in inputs.iter().enumerate() {
        if t == 8 {
            saved = Some(sim.state);
        }
        step(&mut sim.state, &sim.content, i);
        buried_seen |= sim.state.fighters[1].state == S::Buried;
    }
    assert!(buried_seen, "the run did bury the duelist");
    let mut b = saved.expect("saved");
    assert_eq!(b.fighters[1].state, S::Buried);
    for i in &inputs[8..] {
        step(&mut b, &sim.content, i);
    }
    assert_eq!(sim.state.checksum(), b.checksum());
}
