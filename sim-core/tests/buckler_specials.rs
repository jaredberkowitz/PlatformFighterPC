//! The Buckler's phase 2 (class 9, the wanderer): the bow (the neutral special, E35 charge-scaled arrows, E3b lodging), the boomerang
//! (the side special: E3d's one-at-a-time limit, failed throw, smash-input version, catch and clank fall; its out-and-back flight as a
//! projectile script; its hits by age) and the sword wave (the forward smash at exactly 0%: E35 with drag, E38's lying capsule).
//! Reference data and sources: the tenth moveset's `REFERENCE_DATA.md` (`docs/movesets/link/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, SPECIAL, STRONG};
use sim_core::moves::buckler_slots::{BOOMERANG_CATCH, BOOMERANG_FAILED, BOOMERANG_FAR, BOW_FIRE};
use sim_core::moves::{MoveId, Weapon, ANGLE_BY_TRAVEL, ANGLE_FORWARD};
use sim_core::state::{FighterState as S, REST_FALLEN, REST_FLYING, REST_LODGED};
use sim_core::{Fx, Projectile, Vec2};

const WANDERER: u8 = 9;

/// The wanderer (player 0) at the stage's centre facing right and a duelist (player 1) far off at the right, the others out.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([WANDERER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(WANDERER)].weapon)]
}

fn shots(sim: &Sim) -> Vec<Projectile> {
    sim.state
        .projectiles
        .iter()
        .copied()
        .filter(|p| p.active)
        .collect()
}

fn shot(sim: &Sim) -> Projectile {
    let s = shots(sim);
    assert_eq!(s.len(), 1, "one projectile out");
    s[0]
}

fn near(a: Fx, b: Fx, tolerance_thousandths: i32) -> bool {
    (a - b).abs() <= Fx::from_ratio(tolerance_thousandths, 1000)
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// Presses special (stick neutral) and holds it for `held` ticks in all (1: a tap), then lets go; runs `ticks` ticks in all.
fn bow(sim: &mut Sim, held: usize, ticks: usize) {
    for t in 0..ticks {
        let b = if t < held { SPECIAL } else { 0 };
        sim.tick2(inp(0, 0, b), inp(0, 0, 0));
    }
}

// ---- The bow ----

#[test]
fn a_tapped_arrow_leaves_on_frame_16_at_the_uncharged_speed_and_control_returns_on_45() {
    let mut sim = duel();
    bow(&mut sim, 1, 15);
    assert!(shots(&sim).is_empty(), "no arrow before frame 16");
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    bow(&mut sim, 0, 1);
    let a = shot(&sim);
    assert_eq!(sim.f().move_id, BOW_FIRE as u8);
    assert_eq!(a.damage, Fx::from_int(4), "floor(4 + 8 x 0 / 60)");
    assert!(
        near(a.vel.length(), fx(325, 1000), 2),
        "2.6 / 8: {:?}",
        a.vel
    );
    assert!(a.vel.y > Fx::ZERO && a.vel.y < fx(2, 100), "2.5 degrees up");
    assert_eq!(a.life, 46);
    // Frame 16 + 27 more: the release's FAF 30 (45 in all): control on the 45th tick.
    bow(&mut sim, 0, 27);
    assert_eq!(sim.f().state, S::Attack);
    bow(&mut sim, 0, 1);
    assert_ne!(sim.f().state, S::Attack, "actionable on frame 45");
}

#[test]
fn the_charge_scales_the_arrow_and_36_held_frames_are_a_full_charge() {
    // (held, damage, speed in thousandths): c = held - 16 frames past the draw; 35 → floor(4 + 8 x 35 / 60) = 8, 36 → full.
    for (c, damage, speed) in [
        (0, 4, 325),
        (20, 6, 325 + 7400 * 20 / 60 / 8),
        (35, 8, 325 + 7400 * 35 / 60 / 8),
        (36, 12, 1250),
        (90, 12, 1250),
    ] {
        let mut sim = duel();
        // (Held to move frame 14 + c: released on frame 15 + c with c frames of charge.)
        bow(&mut sim, 15 + c, 15 + c);
        assert!(shots(&sim).is_empty(), "still held at c = {c}");
        bow(&mut sim, 0, 1);
        let a = shot(&sim);
        assert_eq!(a.damage, Fx::from_int(damage), "c = {c}");
        assert!(
            near(a.vel.length(), Fx::from_ratio(speed, 1000), 3),
            "c = {c}: {:?}",
            a.vel
        );
        assert_eq!(sim.f().charge, c as u8, "the release keeps the charge");
        // FAF 30 from the release.
        bow(&mut sim, 0, 27);
        assert_eq!(sim.f().state, S::Attack);
        bow(&mut sim, 0, 1);
        assert_ne!(sim.f().state, S::Attack);
    }
}

#[test]
fn the_bow_fires_by_itself_after_180_held_frames() {
    let mut sim = duel();
    bow(&mut sim, 1000, 15 + 180);
    assert!(shots(&sim).is_empty());
    bow(&mut sim, 1000, 1);
    let a = shot(&sim);
    assert_eq!(a.damage, Fx::from_int(12));
    assert_eq!(sim.f().move_id, BOW_FIRE as u8);
}

#[test]
fn an_arrow_hits_for_its_charged_damage_at_angle_361_type_forward() {
    let k = kit(&duel())
        .get(BOW_FIRE as u8)
        .projectile
        .expect("an arrow");
    assert_eq!((k.hitbox.angle, k.hitbox.angle_type), (361, ANGLE_FORWARD));
    assert_eq!(
        (k.hitbox.base_knockback, k.hitbox.knockback_growth),
        (10, 71)
    );
    assert_eq!(k.hitbox.hitlag, 80);
    assert!(!k.pierce);
    for (held, damage) in [(1, 40), (15 + 36, 120)] {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(4), -1);
        bow(&mut sim, held, held.max(16) + 20);
        assert_eq!(sim.fighter(1).percent, dealt(&sim, damage), "held {held}");
        assert!(shots(&sim).is_empty(), "used up by the hit");
    }
}

#[test]
fn an_arrow_lodges_in_the_ground_for_180_frames_and_hits_nobody_there() {
    let mut sim = duel();
    bow(&mut sim, 1, 16);
    let mut lodged_at = None;
    for t in 0..60 {
        bow(&mut sim, 0, 1);
        if let Some(a) = shots(&sim).first() {
            if a.rest == REST_LODGED {
                lodged_at = Some(t);
                assert_eq!(a.pos.y, Fx::ZERO, "on the stage's top");
                assert_eq!(a.vel, Vec2::ZERO);
                assert_eq!(a.life, 180);
                break;
            }
            assert_eq!(a.rest, REST_FLYING);
        }
    }
    let t = lodged_at.expect("the uncharged arrow lodges");
    assert!(t > 10 && t < 46, "lodged after {t} frames");
    // A fighter standing on it is not hit.
    let x = shot(&sim).pos.x;
    sim.stand(1, x, -1);
    bow(&mut sim, 0, 179);
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    assert_eq!(shot(&sim).rest, REST_LODGED);
    bow(&mut sim, 0, 1);
    assert!(shots(&sim).is_empty(), "gone after 180 frames");
}

#[test]
fn an_arrow_meeting_the_ground_more_shallowly_than_9_degrees_is_gone() {
    // A full arrow placed low and flat: 1.25 forward, 0.1 down (under 5 degrees).
    for (vy, lodges) in [(fx(-1, 10), false), (fx(-3, 10), true)] {
        let mut sim = duel();
        sim.state.projectiles[0] = Projectile {
            active: true,
            owner: 0,
            origin: 0,
            power: 100,
            move_id: BOW_FIRE as u8,
            pos: Vec2::new(Fx::from_int(-6), fx(5, 100)),
            vel: Vec2::new(fx(125, 100), vy),
            life: 46,
            size: 100,
            facing: 1,
            damage: Fx::from_int(12),
            ..Projectile::default()
        };
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        let p = sim.state.projectiles[0];
        assert_eq!(p.active, lodges, "vy {vy:?}");
        if lodges {
            assert_eq!(p.rest, REST_LODGED);
        }
    }
}

#[test]
fn the_bow_in_the_air_lands_and_carries_on() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, Fx::ZERO);
    bow(&mut sim, 29, 29);
    assert!(sim.f().grounded(), "landed while drawing");
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, NSPECIAL);
    bow(&mut sim, 0, 1);
    assert_eq!(shot(&sim).damage, Fx::from_int(5), "floor(4 + 8 x 14 / 60)");
}

const NSPECIAL: u8 = MoveId::NSpecial as u8;

// ---- The sword wave ----

/// Starts a forward smash (attack with the stick flicked forward), holding attack for `charge` more frames at the charge frame, and
/// runs to move frame 16 (the wave's frame).
fn forward_smash(sim: &mut Sim, charge: usize) {
    // (Attack held to the charge frame (move frame 8, the ninth tick) and `charge` ticks more there.)
    for t in 1..=(16 + charge) {
        let b = if t <= 8 + charge { ATTACK } else { 0 };
        let x = if t == 1 { 127 } else { 0 };
        sim.tick2(inp(x, 0, b), inp(0, 0, 0));
    }
}

#[test]
fn the_sword_wave_leaves_with_hit_1_only_at_0_percent_and_scales_with_the_charge() {
    for (charge, damage, speed, life, size) in [
        (0usize, 50, 300, 32, 80),
        (30, 90, 362, 36, 95),
        (60, 130, 425, 40, 110),
    ] {
        let mut sim = duel();
        forward_smash(&mut sim, charge);
        assert_eq!(sim.f().move_id, MoveId::FSmash as u8);
        assert_eq!(sim.f().charge as usize, charge);
        assert!(shots(&sim).is_empty(), "not before frame 17");
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        let w = shot(&sim);
        assert_eq!(w.damage, Fx::from_ratio(damage, 10), "charge {charge}");
        assert!(
            near(w.vel.x, Fx::from_ratio(speed, 1000), 2),
            "charge {charge}: {:?}",
            w.vel
        );
        assert_eq!(w.vel.y, Fx::ZERO);
        assert_eq!(w.life, life);
        assert_eq!(w.size, size);
        assert!(near(w.pos.x - sim.f().pos.x, fx(1375, 1000), 2));
    }
    // At 1% there is none.
    let mut sim = duel();
    sim.state.fighters[0].percent = Fx::ONE;
    forward_smash(&mut sim, 0);
    sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    assert!(shots(&sim).is_empty());
}

#[test]
fn the_sword_wave_slows_by_its_drag_and_hits_for_its_charged_damage() {
    let mut sim = duel();
    forward_smash(&mut sim, 0);
    sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    let v0 = shot(&sim).vel.x;
    sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    assert!(
        near(v0 - shot(&sim).vel.x, fx(25, 10000), 1),
        "0.00125 a frame"
    );
    let k = kit(&sim)
        .get(MoveId::FSmash as u8)
        .projectile
        .expect("the wave");
    assert_eq!((k.hitbox.angle, k.hitbox.angle_type), (361, ANGLE_FORWARD));
    assert_eq!(
        (k.hitbox.base_knockback, k.hitbox.knockback_growth),
        (45, 15)
    );
    // A duelist 6 ahead (past the sword's reach) takes the wave's 5%.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(6), -1);
    forward_smash(&mut sim, 0);
    for _ in 0..25 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 50));
}

#[test]
fn the_sword_waves_lying_capsule_reaches_further_than_its_circle() {
    // The same wave with and without its 0.875 half length: the capsule meets a duelist earlier.
    let first_hit = |capsule: bool| {
        let mut sim = duel();
        if !capsule {
            let w = usize::from(sim.content.fighters[usize::from(WANDERER)].weapon);
            let p = sim.content.weapons[w].moves[MoveId::FSmash as usize]
                .projectile
                .as_mut()
                .expect("the wave");
            p.hitbox.half_width = Fx::ZERO;
        }
        sim.stand(1, Fx::from_int(6), -1);
        forward_smash(&mut sim, 0);
        (0..40)
            .find(|_| {
                sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
                sim.fighter(1).percent > Fx::ZERO
            })
            .expect("it hits")
    };
    assert!(first_hit(true) + 2 <= first_hit(false));
}

// ---- The boomerang ----

/// A side special with the stick tilted forward (no flick); `y` tilts the stick up or down.
fn throw(sim: &mut Sim, y: i8) {
    sim.tick2(inp(60, y, SPECIAL), inp(0, 0, 0));
}

fn idle(sim: &mut Sim, n: usize) {
    for _ in 0..n {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
}

#[test]
fn the_boomerang_leaves_on_frame_27_aimed_by_the_stick_and_control_returns_on_46() {
    for (y, up) in [(0i8, 0i32), (127, 1), (-127, -1)] {
        let mut sim = duel();
        throw(&mut sim, 0);
        assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
        for _ in 0..25 {
            sim.tick2(inp(0, y, 0), inp(0, 0, 0));
        }
        assert!(shots(&sim).is_empty());
        sim.tick2(inp(0, y, 0), inp(0, 0, 0));
        let b = shot(&sim);
        assert!(near(b.vel.length(), fx(3, 10), 2), "2.4 / 8");
        let degrees = |v: Vec2| v.y / v.x;
        match up {
            0 => assert_eq!(b.vel.y, Fx::ZERO),
            1 => assert!(
                near(degrees(b.vel), fx(488, 1000), 5),
                "tan 26: {:?}",
                b.vel
            ),
            _ => assert!(
                near(degrees(b.vel), fx(-488, 1000), 5),
                "tan -26: {:?}",
                b.vel
            ),
        }
        assert_eq!(b.life, 125);
        assert_eq!(b.kind, 1);
        idle(&mut sim, 17);
        assert_eq!(sim.f().state, S::Attack);
        idle(&mut sim, 1);
        assert_ne!(sim.f().state, S::Attack, "actionable on 46");
    }
}

#[test]
fn a_smash_input_throws_it_faster_and_longer() {
    let mut sim = duel();
    sim.tick2(inp(127, 0, SPECIAL | STRONG), inp(0, 0, 0));
    assert_eq!(sim.f().move_id, BOOMERANG_FAR as u8);
    idle(&mut sim, 26);
    let b = shot(&sim);
    assert!(near(b.vel.x, fx(3875, 10000), 2), "3.1 / 8");
    assert_eq!(b.life, 167);
}

#[test]
fn the_boomerang_slows_turns_back_homes_and_is_caught() {
    let mut sim = duel();
    throw(&mut sim, 0);
    idle(&mut sim, 26);
    let mut furthest = Fx::ZERO;
    let mut caught = None;
    let mut turned = None;
    for t in 0..125u32 {
        idle(&mut sim, 1);
        let Some(b) = shots(&sim).first().copied() else {
            caught = Some(t);
            break;
        };
        furthest = furthest.max(b.pos.x);
        if b.age == 45 {
            assert!(
                near(b.vel.length(), fx(375, 10000), 1),
                "at its slowest: {:?}",
                b.vel
            );
        }
        if b.vel.x < Fx::ZERO && turned.is_none() {
            turned = Some(b.age);
        }
        if b.age > 60 {
            assert!(b.vel.length() <= fx(375, 1000) + fx(1, 1000));
        }
    }
    let turned = turned.expect("it turns back");
    assert!((45..=47).contains(&turned), "turned back at age {turned}");
    let t = caught.expect("caught");
    assert!(t > 45, "caught on its way back, after {t} frames");
    // Out about (2.4 + 0.3) / 2 x 45 = 61 of the reference's units (7.6 world units) [D].
    assert!(
        furthest > Fx::from_int(7) && furthest < Fx::from_int(9),
        "{furthest:?}"
    );
    assert_eq!(sim.f().move_id, BOOMERANG_CATCH as u8, "idle: the catch");
    assert_eq!(sim.f().state, S::Attack);
    idle(&mut sim, 8);
    assert_ne!(sim.f().state, S::Attack, "the catch's FAF 10");
}

#[test]
fn a_second_throw_with_one_out_is_the_failed_throw() {
    let mut sim = duel();
    throw(&mut sim, 0);
    idle(&mut sim, 45);
    assert_eq!(shots(&sim).len(), 1);
    // (The boomerang lifted well out of the way, so it is not caught during the second throw.)
    sim.state
        .projectiles
        .iter_mut()
        .for_each(|p| p.pos.y += Fx::from_int(20));
    throw(&mut sim, 0);
    idle(&mut sim, 1);
    assert_eq!(sim.f().move_id, BOOMERANG_FAILED as u8, "on its frame 2");
    idle(&mut sim, 30);
    assert_eq!(shots(&sim).len(), 1, "nothing thrown");
    // FAF 60 on the ground: 2 + 57 ticks so far gives frame 59.
    idle(&mut sim, 26);
    assert_eq!(sim.f().state, S::Attack);
    idle(&mut sim, 1);
    assert_ne!(sim.f().state, S::Attack);
}

#[test]
fn the_boomerangs_hits_go_by_its_age_and_the_way_it_travels() {
    let sim = duel();
    let p = kit(&sim)
        .get(MoveId::SideSpecial as u8)
        .projectile
        .expect("a boomerang");
    let hit = |age: u8| p.hit_at(age).0;
    for (age, damage, angle, bkb) in [
        (1u8, 80, 70, 80),
        (9, 80, 70, 80),
        (10, 60, 65, 60),
        (44, 60, 65, 60),
        (45, 30, 70, 50),
    ] {
        let h = hit(age);
        assert_eq!(
            (h.damage, h.angle, h.base_knockback, h.knockback_growth),
            (Fx::from_ratio(damage, 10), angle, bkb, 40),
            "age {age}"
        );
        assert_eq!(h.angle_type, ANGLE_BY_TRAVEL);
        assert_eq!(h.shield_damage, 50);
    }
    let far = kit(&sim)
        .get(BOOMERANG_FAR as u8)
        .projectile
        .expect("a far one");
    assert_eq!(far.hit_at(1).0.damage, fx(96, 10));
    assert_eq!(far.hit_at(59).0.damage, fx(72, 10));
    assert_eq!(far.hit_at(60).0.damage, Fx::from_int(3));
    assert!(!p.pierce);
    // Live: clean up close, late further out.
    for (x, damage) in [(3, 80), (6, 60)] {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(x), -1);
        throw(&mut sim, 0);
        idle(&mut sim, 60);
        assert_eq!(sim.fighter(1).percent, dealt(&sim, damage), "at {x}");
    }
}

#[test]
fn the_returning_boomerang_sends_its_target_toward_the_thrower() {
    let mut sim = duel();
    throw(&mut sim, 0);
    idle(&mut sim, 26 + 52);
    let b = shot(&sim);
    assert!(b.vel.x < Fx::ZERO);
    // A duelist stepped in just ahead of it on its way back.
    sim.stand(1, b.pos.x - fx(5, 10), 1);
    idle(&mut sim, 1);
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 30));
    let x0 = sim.fighter(1).pos.x;
    idle(&mut sim, 25);
    assert!(
        sim.fighter(1).pos.x < x0,
        "sent the way it was going: toward the thrower"
    );
}

#[test]
fn a_clank_knocks_the_boomerang_down_and_it_still_counts_as_out() {
    let mut sim = duel();
    throw(&mut sim, 0);
    idle(&mut sim, 26);
    let b0 = shot(&sim);
    // A duelist's jab out right where it is going: the boomerang meets it and drops.
    let duelist_jab = sim.content.weapons[usize::from(sim.content.fighters[0].weapon)]
        .get(MoveId::Jab as u8)
        .clone();
    let first = duelist_jab
        .hitboxes
        .iter()
        .map(|h| h.start)
        .min()
        .expect("a jab hit");
    let reach = duelist_jab
        .hitboxes
        .iter()
        .map(|h| h.x)
        .max()
        .expect("a jab hit");
    let f = &mut sim.state.fighters[1];
    f.pos.x = b0.pos.x + fx(3, 10) + reach;
    f.facing = -1;
    f.state = S::Attack;
    f.move_id = MoveId::Jab as u8;
    // (The frame before its first active one: the update moves it on.)
    f.state_frame = u16::from(first).saturating_sub(1);
    f.hit_mask = 0;
    let mut fell = false;
    for _ in 0..4 {
        idle(&mut sim, 1);
        if shots(&sim).first().is_some_and(|p| p.rest == REST_FALLEN) {
            fell = true;
            break;
        }
    }
    assert!(fell, "knocked down");
    let p = shot(&sim);
    assert!(
        p.vel.y > Fx::ZERO && p.vel.x < Fx::ZERO,
        "up and back: {:?}",
        p.vel
    );
    assert_eq!(p.life, 60);
    // Down to the stage, where it lies; meanwhile a throw fails.
    idle(&mut sim, 40);
    let p = shot(&sim);
    assert_eq!(p.pos.y, Fx::ZERO);
    let before = sim.fighter(1).percent;
    throw(&mut sim, 0);
    idle(&mut sim, 1);
    assert_eq!(sim.f().move_id, BOOMERANG_FAILED as u8);
    idle(&mut sim, 20);
    assert!(shots(&sim).is_empty(), "gone after its 60 frames");
    assert_eq!(
        sim.fighter(1).percent,
        before,
        "a fallen boomerang hits nobody"
    );
}

#[test]
fn a_reflected_boomerang_hits_its_thrower_and_is_not_caught() {
    let mut sim = duel();
    throw(&mut sim, 0);
    idle(&mut sim, 30);
    // As a reflector leaves it: the reflector's, coming back fast, its age started again.
    let p = &mut sim
        .state
        .projectiles
        .iter_mut()
        .find(|p| p.active)
        .expect("out");
    p.owner = 1;
    p.age = 0;
    p.vel = Vec2::new(-fx(3, 10), Fx::ZERO);
    p.facing = -1;
    p.hit_mask = 0;
    idle(&mut sim, 30);
    assert_eq!(
        sim.f().percent,
        dealt(&sim, 80),
        "its clean hit, on its thrower"
    );
    assert_ne!(sim.f().move_id, BOOMERANG_CATCH as u8);
}

#[test]
fn a_shielding_thrower_catches_the_boomerang_without_the_catch_move() {
    // Defending (shield held from the boomerang's turn on): it is caught (gone) but the catch move does not start.
    let mut sim = duel();
    throw(&mut sim, 0);
    idle(&mut sim, 26 + 40);
    for _ in 0..80 {
        sim.tick2(inp(0, 0, sim_core::input::buttons::SHIELD), inp(0, 0, 0));
        if shots(&sim).is_empty() {
            break;
        }
    }
    assert!(shots(&sim).is_empty(), "caught");
    assert_eq!(sim.f().state, S::Shield);
}

#[test]
fn the_boomerang_is_resimulated_exactly_from_a_snapshot() {
    let mut sim = duel();
    throw(&mut sim, 40);
    idle(&mut sim, 40);
    let saved = sim.state;
    let script: Vec<i8> = (0..90).map(|i| ((i * 37) % 255 - 127) as i8).collect();
    for &x in &script {
        sim.tick2(inp(x, 0, 0), inp(0, 0, 0));
    }
    let straight = sim.state.checksum();
    sim.state = saved;
    for &x in &script {
        sim.tick2(inp(x, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.state.checksum(), straight);
}
