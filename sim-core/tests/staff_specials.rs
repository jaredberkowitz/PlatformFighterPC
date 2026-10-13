//! The Staff's specials (class 5, the seer body; phase 2), modelled mechanically on a reference character: frame data, movement
//! and the behaviour of each special against `docs/movesets/palutena/REFERENCE_DATA.md` ("Specials"), and the extensions they
//! brought.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{MoveId, Weapon};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const SEER: u8 = 5;

/// The seer (player 0) and a duelist (player 1, parked at the far right), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SEER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn staff(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SEER)].weapon)]
}

fn su(thousandths: i32) -> Fx {
    Fx::from_ratio(thousandths, 8000)
}

fn near(a: Fx, b: Fx, tolerance_thousandths: i32) -> bool {
    (a - b).abs() <= Fx::from_ratio(tolerance_thousandths, 1000)
}

// ---- Up special: the warp ------------------------------------------------------------------------------------------------

#[test]
fn the_warps_data_is_the_references() {
    let sim = duel();
    let warp = staff(&sim).get(MoveId::UpSpecial as u8);
    assert_eq!(u16::from(warp.total_frames) + 2, 68, "FAF 68");
    assert!(warp.hitboxes.is_empty(), "no hitbox");
    // Intangible on the reference's 16-32 (move frames 15-31).
    assert_eq!((warp.intangible_from, warp.intangible), (15, 32));
    // Startup gravity 0.03 and top fall speed 0.75 (reference units), frames 1-15.
    let startup = warp.air_blocks[0];
    assert_eq!((startup.from, startup.to), (0, 14));
    assert_eq!(
        (startup.gravity, startup.max_fall),
        (Some(su(30)), Some(su(750)))
    );
    assert!(warp.helpless_after);
    assert_eq!((warp.landing_lag, warp.fall_landing_lag), (22, 22));
    assert_eq!(
        warp.lands_from + 1,
        33,
        "touching down before the reappearance carries on"
    );
    assert_eq!(
        (warp.ledge_front_from + 1, warp.ledge_back_from + 1),
        (33, 33)
    );
}

/// Starts the warp in the air at (x, y) with the stick up, then holds `aim` from the next frame on. Returns the sim after the
/// starting frame (move frame 0).
fn air_warp(x: Fx, y: Fx) -> Sim {
    let mut sim = duel();
    sim.put_airborne(0, x, y, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    sim
}

#[test]
fn the_warp_travels_59_5_reference_units_straight_up_intangible_on_16_to_32() {
    let mut sim = air_warp(Fx::ZERO, Fx::from_int(8));
    let mut y = [Fx::ZERO; 67];
    for frame in 1..=40u16 {
        sim.tick(inp(0, 127, 0));
        assert_eq!(sim.f().state_frame, frame);
        y[usize::from(frame)] = sim.f().pos.y;
        // (Reference frame = move frame + 1.)
        let reference = frame + 1;
        assert_eq!(
            is_intangible(sim.f()),
            (16..=32).contains(&reference),
            "intangible on the reference's frames 16-32 only (frame {reference})"
        );
    }
    // The startup falls under gravity 0.03 (14 frames of it after the starting one), never past 0.75.
    assert!(y[14] < Fx::from_int(8));
    // 17 frames at 3.5 reference units a frame: 59.5 units, 7.4375 world units.
    assert_eq!(y[31] - y[14], su(3500).mul_int(17));
    // The reappearance: half the speed once.
    assert_eq!(y[32] - y[31], su(1750));
    assert_eq!(sim.f().pos.x, Fx::ZERO, "straight up");
}

#[test]
fn the_stick_on_frame_16_aims_the_warp_and_later_changes_do_not() {
    // Up and forward at 45 degrees, then the stick swung back down: the warp keeps the first direction.
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(8));
    sim.ticks(14, inp(0, 0, 0));
    let start = sim.f().pos;
    sim.tick(inp(90, 90, 0));
    sim.ticks(16, inp(-127, -127, 0));
    let moved = sim.f().pos - start;
    let leg = su(3500).mul_int(17) * fx(7071, 10000);
    assert!(near(moved.x, leg, 5), "forward {moved:?}");
    assert!(near(moved.y, leg, 5), "up {moved:?}");

    // A neutral stick on frame 16 warps straight up; the stick's tilt does not change the distance.
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(8));
    sim.ticks(14, inp(0, 0, 0));
    let start = sim.f().pos;
    sim.ticks(17, inp(0, 0, 0));
    assert_eq!(sim.f().pos.x, start.x);
    assert_eq!(sim.f().pos.y - start.y, su(3500).mul_int(17));
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(8));
    sim.ticks(14, inp(0, 0, 0));
    let start = sim.f().pos;
    sim.tick(inp(50, 0, 0));
    sim.ticks(16, inp(0, 0, 0));
    assert!(near(sim.f().pos.x - start.x, su(3500).mul_int(17), 2));
}

#[test]
fn an_aerial_warp_ends_helpless_and_lands_with_22_frames() {
    let mut sim = air_warp(Fx::ZERO, Fx::from_int(12));
    sim.ticks(66, inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Helpless,
        "FAF 68: helpless after the reappearance"
    );
    let mut frames = 0;
    while sim.f().state == S::Helpless && frames < 600 {
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 22);
}

#[test]
fn landing_during_the_reappearance_costs_22_frames() {
    // A sideways warp low over the stage: the reappearance falls onto it before frame 68.
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(2));
    sim.ticks(14, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    let mut landed_on = None;
    for _ in 16..=66 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            landed_on = Some(sim.f().state_frame);
            break;
        }
    }
    assert!(landed_on.is_some(), "landed during the reappearance");
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 22);
}

#[test]
fn a_warp_into_the_ground_glides_along_it_and_reappears_standing() {
    // Down and forward from just above the stage: it meets the ground, glides on along it and ends standing (no landing lag).
    let mut sim = air_warp(Fx::from_int(-6), Fx::from_int(1));
    sim.ticks(14, inp(0, 0, 0));
    let x0 = sim.f().pos.x;
    sim.tick(inp(90, -90, 0));
    sim.ticks(16, inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Attack,
        "still warping after touching down"
    );
    assert!(sim.f().grounded());
    assert_eq!(sim.f().pos.y, Fx::ZERO);
    assert!(sim.f().pos.x > x0 + Fx::from_int(4), "it glided on");
    sim.ticks(35, inp(0, 0, 0));
    assert_eq!(
        sim.f().state,
        S::Idle,
        "a grounded reappearance ends standing"
    );
}

#[test]
fn a_grounded_warp_along_the_stage_ends_standing_and_one_sliding_off_an_edge_is_cancelled() {
    // Sideways along the ground: it stays on the ground and ends standing at FAF 68.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-6), 1);
    sim.tick(inp(0, 127, SPECIAL));
    sim.ticks(14, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    sim.ticks(16, inp(0, 0, 0));
    assert!(sim.f().grounded());
    assert!(near(
        sim.f().pos.x,
        Fx::from_int(-6) + su(3500).mul_int(17),
        1
    ));
    sim.ticks(34, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);

    // Near the right edge: the travel stays on the stage, the reappearance slides off it and is cancelled into an ordinary fall.
    let mut sim = duel();
    sim.stand(0, fx(32, 10), 1);
    sim.tick(inp(0, 127, SPECIAL));
    sim.ticks(14, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    sim.ticks(17, inp(0, 0, 0));
    assert!(sim.f().grounded(), "the reappearance starts on the stage");
    let mut fell = false;
    for _ in 33..=66 {
        sim.tick(inp(0, 0, 0));
        assert_ne!(
            sim.f().state,
            S::Helpless,
            "an edge-cancelled warp is not helpless"
        );
        if sim.f().state == S::Airborne {
            fell = true;
            break;
        }
    }
    assert!(fell, "slid off the edge during the reappearance");
}

#[test]
fn the_warp_catches_a_ledge_from_the_reappearance_on() {
    // Below the left ledge, facing the stage: the warp straight up passes into reach on frame 32 but catches only from 33.
    let half_width = duel().content.fighters[usize::from(SEER)].ecb_half_width;
    let mut sim = air_warp(Fx::from_int(-11) - half_width - fx(2, 10), fx(-95, 10));
    let mut caught = None;
    for frame in 1..=40u16 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            caught = Some(frame);
            break;
        }
    }
    let frame = caught.expect("caught the ledge");
    assert!(
        frame + 1 >= 33,
        "not before the reappearance (caught on {})",
        frame + 1
    );
}

// ---- Down special: the counter and the reflect barrier ------------------------------------------------------------------

const COUNTERATTACK: MoveId = MoveId::Ext0;
const REFLECTION: MoveId = MoveId::Ext1;

#[test]
fn the_counter_and_barriers_data_is_the_references() {
    let sim = duel();
    let w = staff(&sim);
    let stance = w.get(MoveId::DownSpecial as u8);
    assert_eq!(u16::from(stance.total_frames) + 2, 71, "FAF 71");
    // Intangible 6-12.
    assert_eq!((stance.intangible_from, stance.intangible), (5, 12));
    let c = stance.counter.expect("a counter");
    assert_eq!((c.start + 1, c.end + 1), (7, 34), "counter window 7-34");
    assert_eq!(c.then, COUNTERATTACK as u8);
    assert_eq!(
        (c.percent, c.min_damage, c.max_damage, c.melee_only),
        (130, Fx::from_int(10), Fx::from_int(50), true)
    );
    let rf = stance.reflector.expect("a projectile search box");
    assert_eq!((rf.detect_start + 1, rf.detect_end + 1), (7, 34));
    assert_eq!(rf.detect_then, Some(REFLECTION as u8));
    assert!(rf.start > rf.end, "the stance itself reflects nothing");
    assert_eq!(rf.detect_radius, fx(15, 10));
    // The air: x0.33 on starting, friction 0.0025, gravity 0.04, top fall 1.2, top rise 0.8 (reference units).
    assert_eq!(stance.start_speed, 33);
    assert_eq!(
        (
            stance.air_friction,
            stance.air_gravity,
            stance.air_max_fall,
            stance.air_max_rise
        ),
        (
            Some(fx(25, 80_000)),
            Some(su(40)),
            Some(su(1200)),
            Some(su(800))
        )
    );

    let strike = w.get(COUNTERATTACK as u8);
    assert_eq!(u16::from(strike.total_frames) + 2, 37, "FAF 37");
    assert_eq!(strike.intangible, 5, "intangible 1-5");
    assert!(strike.counter_strike);
    assert!(strike
        .hitboxes
        .iter()
        .all(|h| (h.start + 1, h.end + 1, h.angle, h.hitlag, h.clanks) == (5, 7, 361, 150, false)));
    let kb: Vec<(u8, i16, i16)> = strike
        .hitboxes
        .iter()
        .map(|h| (h.priority, h.base_knockback, h.knockback_growth))
        .collect();
    assert!(kb.contains(&(0, 50, 70)) && kb.contains(&(1, 30, 65)));

    let reflection = w.get(REFLECTION as u8);
    assert_eq!(u16::from(reflection.total_frames) + 2, 35, "FAF 35");
    let rf = reflection.reflector.expect("the barrier");
    assert_eq!(
        (
            rf.damage_percent,
            rf.speed_percent,
            rf.life_percent,
            rf.max_damage
        ),
        (140, 125, 100, Fx::from_int(50))
    );
    assert_eq!(
        (rf.body_start + 1, rf.body_end + 1),
        (1, 4),
        "body reflector 1-4"
    );
    assert_eq!(
        (rf.start, rf.end),
        (0, reflection.total_frames),
        "the barrier to the end"
    );
    assert!(rf.half_height > Fx::ZERO, "a standing barrier");
    assert!(reflection.hitboxes.iter().all(|h| h.push
        && h.set_weight
        && h.fixed_kb == 50
        && (h.start + 1, h.end + 1) == (2, 11)));
    assert_eq!(reflection.rehit, Some((1, 1)));
}

/// The seer standing at 0 facing right; player 1 (`class`) standing at `x` facing left.
fn stance_against(class: u8, x: Fx) -> Sim {
    let mut sim = Sim::with_chars([SEER, class, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, x, -1);
    sim
}

#[test]
fn a_melee_hit_in_the_window_is_countered_for_1_3_times_between_10_and_50() {
    use sim_core::input::buttons::ATTACK;
    // A jab (a few percent) is answered with the 10% floor; the counterattack then strikes for it.
    let mut sim = stance_against(0, fx(15, 10));
    sim.tick2(inp(0, -127, SPECIAL), inp(0, 0, 0));
    sim.ticks(3, inp(0, 0, 0));
    let mut caught = false;
    for _ in 0..30 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
        if sim.f().move_id == COUNTERATTACK as u8 {
            caught = true;
            break;
        }
    }
    assert!(caught, "the jab was countered");
    assert_eq!(sim.f().percent, Fx::ZERO, "the counter took no damage");
    assert_eq!(sim.f().counter_damage, Fx::from_int(10));
    let before = sim.fighter(1).percent;
    for _ in 0..40 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(
        sim.fighter(1).percent - before,
        Fx::from_int(10) * sim.content.rules.damage_mult,
        "the counterattack deals what it caught"
    );

    // A huge hit is answered with at most 50%.
    let mut sim = stance_against(0, fx(15, 10));
    let w = usize::from(sim.content.fighters[0].weapon);
    for h in &mut sim.content.weapons[w].moves[MoveId::Jab as usize].hitboxes {
        h.damage = Fx::from_int(45);
    }
    sim.tick2(inp(0, -127, SPECIAL), inp(0, 0, 0));
    sim.ticks(3, inp(0, 0, 0));
    for _ in 0..30 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
        if sim.f().move_id == COUNTERATTACK as u8 {
            break;
        }
    }
    assert_eq!(sim.f().move_id, COUNTERATTACK as u8);
    assert_eq!(sim.f().counter_damage, Fx::from_int(50));
}

/// The claws' blaster fired at the seer, whose stance starts `delay` frames after the shot's owner presses special. Returns the sim
/// once the shot has been reflected or has hit (or 60 frames have passed).
fn shot_at_stance(delay: usize) -> Sim {
    let mut sim = stance_against(1, Fx::from_int(6));
    sim.tick2(inp(0, 0, 0), inp(0, 0, SPECIAL));
    for t in 1..60 {
        let seer = if t == delay {
            inp(0, -127, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(seer, inp(0, 0, 0));
        if sim.f().move_id == REFLECTION as u8 || sim.f().percent > Fx::ZERO {
            break;
        }
    }
    sim
}

#[test]
fn a_shot_touching_the_search_box_starts_the_reflection_and_is_never_countered() {
    let sim = shot_at_stance(8);
    assert_eq!(sim.f().move_id, REFLECTION as u8, "the reflection started");
    assert_eq!(sim.f().state_frame, 0, "on the frame the shot touched");
    assert_eq!(sim.f().percent, Fx::ZERO);
    let shot = sim
        .state
        .projectiles
        .iter()
        .find(|p| p.active)
        .expect("the shot is still flying");
    assert_eq!(shot.owner, 0, "it belongs to the seer now");
    assert_eq!(shot.power, 140, "x1.4 damage");
    assert_eq!(
        shot.vel.x,
        su(3000) * fx(125, 100),
        "turned round at x1.25 speed"
    );
}

/// The seer (standing at 0, facing right) in `move` at move frame `frame`, and the claws' blaster shot flying at it from `x` ahead
/// at `height`.
fn shot_into(move_id: MoveId, frame: u16, x: Fx, height: Fx) -> Sim {
    use sim_core::state::Projectile;
    use sim_core::Vec2;
    let mut sim = stance_against(1, Fx::from_int(9));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = move_id as u8;
    f.state_frame = frame;
    sim.state.projectiles[0] = Projectile {
        active: true,
        owner: 1,
        origin: 1,
        power: 100,
        move_id: MoveId::NSpecial as u8,
        pos: Vec2::new(x, height),
        vel: Vec2::new(-su(3000), Fx::ZERO),
        age: 1,
        life: 35,
        size: 100,
        ..Projectile::default()
    };
    sim
}

#[test]
fn a_shot_outside_the_window_hits() {
    // Before the search box (the stance's frame 1, not yet intangible) and after it (frame 36): the seer is hit and nothing reflects.
    // (Before: from close enough to land on frame 1 or 2.)
    for (frame, x) in [(0u16, fx(13, 10)), (36, Fx::from_int(3))] {
        let mut sim = shot_into(MoveId::DownSpecial, frame, x, fx(12, 10));
        for _ in 0..10 {
            sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        }
        assert!(sim.f().percent > Fx::ZERO, "frame {frame}: the shot landed");
        assert_ne!(sim.f().move_id, REFLECTION as u8);
        assert_ne!(sim.f().move_id, COUNTERATTACK as u8, "never countered");
    }
}

#[test]
fn the_reflections_barrier_reflects_shots_at_its_height_and_not_above() {
    // The seer in the reflection at its frame 10 (past the body reflector): a shot at 2.2 up meets the barrier, one at 4.0 up flies
    // over it.
    for (height, reflected) in [(fx(22, 10), true), (Fx::from_int(4), false)] {
        let mut sim = shot_into(REFLECTION, 10, Fx::from_int(3), height);
        for _ in 0..4 {
            sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        }
        let shot = sim.state.projectiles[0];
        assert_eq!(shot.owner == 0, reflected, "height {height:?}: {shot:?}");
    }
}

#[test]
fn a_melee_hit_during_the_reflection_is_taken() {
    use sim_core::input::buttons::ATTACK;
    // The reflection does not counter: a jab into it lands.
    let mut sim = stance_against(0, fx(15, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = REFLECTION as u8;
    f.state_frame = 12;
    for _ in 0..12 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
    }
    assert!(sim.f().percent > Fx::ZERO);
    assert_ne!(sim.f().move_id, COUNTERATTACK as u8);
}

#[test]
fn the_barriers_push_moves_a_fighter_without_hitstun() {
    let mut sim = stance_against(0, fx(14, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = REFLECTION as u8;
    f.state_frame = 0;
    let x0 = sim.fighter(1).pos.x;
    for _ in 0..10 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        assert_ne!(sim.fighter(1).state, S::Hitstun);
    }
    assert!(sim.fighter(1).pos.x > x0, "pushed away");
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
}

#[test]
fn the_stance_holds_the_rise_at_0_8_and_keeps_a_third_of_the_speed() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(6), fx(1, 5), Fx::ONE);
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
    assert_eq!(sim.f().vel.x, fx(1, 5) * fx(33, 100));
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().vel.y,
        su(800) - su(40),
        "held at 0.8, then gravity 0.04"
    );
}

#[test]
fn the_counter_catches_through_its_own_intangibility() {
    use sim_core::input::buttons::ATTACK;
    // The stance is intangible on 6-12, but its counter (7-34) still catches a melee hit then, as the reference's counters do.
    let mut sim = stance_against(0, fx(15, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = MoveId::DownSpecial as u8;
    f.state_frame = 6;
    f.invuln = 6;
    let mut caught_while_intangible = false;
    for _ in 0..8 {
        let was_intangible = is_intangible(sim.f());
        sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
        if sim.f().move_id == COUNTERATTACK as u8 {
            caught_while_intangible = was_intangible;
            break;
        }
    }
    assert!(caught_while_intangible);
}

#[test]
fn the_down_specials_new_fields_are_in_the_content_hash() {
    use sim_core::Content;
    let base = Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[usize::from(SEER)].weapon);
    let edit = |f: &dyn Fn(&mut sim_core::moves::Move)| {
        let mut c = base.clone();
        f(&mut c.weapons[w].moves[MoveId::DownSpecial as usize]);
        c.hash()
    };
    assert_ne!(
        h,
        edit(&|m| m.counter.as_mut().unwrap().melee_only = false),
        "melee_only"
    );
    assert_ne!(
        h,
        edit(&|m| m.counter.as_mut().unwrap().max_damage = Fx::ZERO),
        "max_damage"
    );
    assert_ne!(
        h,
        edit(&|m| m.reflector.as_mut().unwrap().detect_then = None),
        "detect_then"
    );
    assert_ne!(h, edit(&|m| m.air_max_rise = None), "air_max_rise");
    assert_ne!(
        h,
        edit(&|m| m.reflector.as_mut().unwrap().half_height = Fx::ONE),
        "half_height"
    );
    assert_ne!(
        h,
        edit(&|m| m.reflector.as_mut().unwrap().body_radius = Fx::ONE),
        "body_radius"
    );
    assert_ne!(
        h,
        edit(&|m| m.reflector.as_mut().unwrap().body_end = 9),
        "body_end"
    );
}

// ---- Side special: the explosive flame -----------------------------------------------------------------------------------

#[test]
fn the_explosive_flames_data_is_the_references() {
    use sim_core::moves::{ANGLE_BY_POSITION, ANGLE_FORWARD};
    let sim = duel();
    let mv = staff(&sim).get(MoveId::SideSpecial as u8);
    assert_eq!(u16::from(mv.total_frames) + 2, 72, "FAF 72");
    assert_eq!(mv.start_speed, 80);
    let p = mv.projectile.expect("the explosion");
    assert!(p.pierce, "not used up by a hit");
    assert_eq!(p.rehit, 5, "hits a fighter again 5 frames after");
    assert_eq!(p.life, 31, "the last hit at age 30 (frame 57)");
    assert_eq!(p.speed, Fx::ZERO, "it stays where it is made");
    // The loop's radius by age (ages 1 / 5 / ... / 25 are frames 28 / 32 / ... / 52): 4.8 to 12.0 reference units.
    let radii: Vec<(u8, Fx)> = core::iter::once((1, p.hitbox.radius))
        .chain(p.phases.iter().flatten().map(|(a, h)| (*a, h.radius)))
        .collect();
    let t = |n| fx(n, 10);
    assert_eq!(
        radii,
        vec![
            (1, t(6)),
            (5, t(8)),
            (9, t(9)),
            (13, t(11)),
            (17, t(12)),
            (21, t(14)),
            (25, t(15)),
            (29, Fx::ZERO),
            (30, t(19))
        ]
    );
    let looped =
        core::iter::once(p.hitbox).chain(p.phases.iter().flatten().take(6).map(|(_, h)| *h));
    for h in looped {
        assert_eq!(
            (
                h.damage,
                h.angle,
                h.base_knockback,
                h.knockback_growth,
                h.angle_type,
                h.group
            ),
            (fx(15, 10), 160, 50, 100, ANGLE_BY_POSITION, 0)
        );
        assert_eq!((h.sdi, h.shield_damage, h.clanks), (40, 53, false));
    }
    let (_, last) = p.phases[7].unwrap();
    assert_eq!(
        (
            last.damage,
            last.angle,
            last.base_knockback,
            last.knockback_growth,
            last.angle_type
        ),
        (fx(55, 10), 84, 60, 141, ANGLE_FORWARD)
    );
    assert_eq!(
        (last.hitlag, last.group, last.clanks, last.shield_damage),
        (150, 1, false, 51)
    );
}

/// The seer at `x` facing right presses side special with the stick at `stick_x` this frame (127 from neutral is a flick; 50 is
/// not). Returns the sim after that frame (move frame 0).
fn flame_from(x: Fx, stick_x: i8) -> Sim {
    let mut sim = duel();
    sim.stand(0, x, 1);
    sim.tick(inp(stick_x, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
    sim
}

fn explosion(sim: &Sim) -> Option<sim_core::Projectile> {
    sim.state.projectiles.iter().find(|p| p.active).copied()
}

#[test]
fn the_flame_appears_on_frame_27_45_units_ahead_or_75_with_a_flick() {
    for (stick, ahead) in [(50i8, fx(5625, 1000)), (127, fx(9375, 1000))] {
        let mut sim = flame_from(Fx::from_int(-6), stick);
        sim.state.fighters[0].special_flick = stick == 127 && sim.f().special_flick;
        assert_eq!(sim.f().special_flick, stick == 127, "the flick register");
        for frame in 1..=26u16 {
            sim.tick(inp(0, 0, 0));
            assert_eq!(
                explosion(&sim).is_some(),
                frame == 26,
                "made on the reference's frame 27 (move frame {frame})"
            );
        }
        let p = explosion(&sim).unwrap();
        assert_eq!(p.pos.x, Fx::from_int(-6) + ahead);
        assert_eq!(p.pos.y, fx(125, 100));
        assert_eq!(p.vel, sim_core::Vec2::ZERO);
        assert_eq!(p.facing, 1);
    }
}

#[test]
fn the_flame_stays_where_the_seer_stood_on_frame_1() {
    // In the air, falling: the explosion still appears at the point fixed as the move began.
    let mut sim = duel();
    sim.put_airborne(0, Fx::from_int(-6), Fx::from_int(8), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(50, 0, SPECIAL));
    let start = sim.f().pos;
    sim.ticks(26, inp(0, 0, 0));
    assert!(sim.f().pos.y < start.y, "the seer fell meanwhile");
    let p = explosion(&sim).expect("made");
    assert_eq!(p.pos.x, start.x + fx(5625, 1000));
    assert_eq!(p.pos.y, start.y + fx(125, 100));
}

#[test]
fn a_shielding_fighter_inside_it_is_hit_on_28_33_38_43_48_53_and_57() {
    use sim_core::input::buttons::SHIELD;
    let mut sim = flame_from(Fx::from_int(-6), 50);
    sim.stand(1, Fx::from_int(-6) + fx(5625, 1000), -1);
    let mut hits = Vec::new();
    let mut hp = sim.fighter(1).shield_hp;
    for frame in 1..=70u16 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
        let now = sim.fighter(1).shield_hp;
        if now < hp - fx(5, 10) {
            hits.push(frame + 1);
        }
        hp = now;
    }
    assert_eq!(hits, vec![28, 33, 38, 43, 48, 53, 57]);
}

#[test]
fn a_fighter_walking_in_late_is_hit_every_5_frames_from_its_first_hit() {
    use sim_core::input::buttons::SHIELD;
    // Out of reach until the explosion's age 3 (frame 30), then shielding inside: hit on 30, 35, 40, 45, 50, 55, and the last hit.
    let mut sim = flame_from(Fx::from_int(-6), 50);
    let centre = Fx::from_int(-6) + fx(5625, 1000);
    sim.stand(1, Fx::from_int(9), -1);
    let mut hits = Vec::new();
    for frame in 1..=70u16 {
        if frame + 1 == 30 {
            sim.stand(1, centre, -1);
            sim.state.fighters[1].state = S::Shield;
            // (Past the perfect-shield window.)
            sim.state.fighters[1].state_frame = 20;
        }
        let hp = sim.fighter(1).shield_hp;
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
        if sim.fighter(1).shield_hp < hp - fx(5, 10) {
            hits.push(frame + 1);
        }
    }
    assert_eq!(hits, vec![30, 35, 40, 45, 50, 55, 57]);
}

#[test]
fn the_flame_pierces_and_pulls_each_target_back_toward_its_centre() {
    // Two duelists inside it, one each side of the centre: both are hit on its first frame, and each is sent back across the
    // middle (160 degrees away from the centre, by position).
    let mut sim = Sim::with_chars([SEER, 0, 0, 0]);
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-8), 1);
    let centre = Fx::from_int(-8) + fx(5625, 1000);
    sim.stand(1, centre + fx(3, 10), -1);
    sim.stand(2, centre - fx(3, 10), 1);
    sim.tick(inp(50, 0, SPECIAL));
    sim.ticks(27, inp(0, 0, 0));
    for i in [1, 2] {
        assert!(
            sim.fighter(i).percent > Fx::ZERO,
            "fighter {i} was hit: {:?} {:?}",
            sim.fighter(i).pos,
            explosion(&sim).map(|p| p.pos)
        );
    }
    assert!(explosion(&sim).is_some(), "still there after hitting both");
    // (Launched when its hitlag ends: 160 degrees for the one on the right, toward the left; mirrored, 20, for the one on the left.)
    use sim_core::trig::Angle;
    assert_eq!(sim.fighter(1).launch_angle, Angle::from_degrees(160).raw());
    assert_eq!(sim.fighter(2).launch_angle, Angle::from_degrees(20).raw());
}

#[test]
fn the_flame_never_clanks_and_its_new_fields_are_hashed() {
    use sim_core::Content;
    let base = Content::placeholder();
    let w = usize::from(base.fighters[usize::from(SEER)].weapon);
    let p = base.weapons[w].moves[MoveId::SideSpecial as usize]
        .projectile
        .unwrap();
    assert!(!p.hitbox.clanks);
    let h = base.hash();
    let edit = |f: &dyn Fn(&mut sim_core::moves::ProjectileSpawn)| {
        let mut c = base.clone();
        f(c.weapons[w].moves[MoveId::SideSpecial as usize]
            .projectile
            .as_mut()
            .unwrap());
        c.hash()
    };
    assert_ne!(h, edit(&|p| p.pierce = false), "pierce");
    assert_ne!(h, edit(&|p| p.rehit = 4), "rehit");
    assert_ne!(h, edit(&|p| p.phases[7] = None), "the eighth phase");
}
