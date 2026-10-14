//! The Popgun's up special, the rocket barrels (class 11, the scamp body), modelled mechanically on a reference character: the charge
//! (frames 7 to 67, launching on 8 to 68), the launch speed by the charge and the air multiplier, the angle tilted while charging and
//! steered while flying, the flight's hits, the special fall and its landing lag, the crash into a wall, and the barrels flying off a
//! hit wearer (E48) to burst on the stage. Reference data: the twelfth moveset's `REFERENCE_DATA.md` (`docs/movesets/diddy_kong/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, SPECIAL};
use sim_core::moves::popgun_slots::{BARRELS, BARRELS_KIND, ROCKET_CRASH, ROCKET_FLIGHT};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{step, Fx, GameState, Input, Rng, MAX_FIGHTERS};

const SCAMP: u8 = 11;

/// The scamp (player 0) at `x` facing right on the stage and a duelist (player 1) far away, the others out of the match.
fn setup(x: Fx) -> Sim {
    let mut sim = Sim::with_chars([SCAMP, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, x, 1);
    sim.stand(1, Fx::from_int(14), -1);
    sim
}

/// Up special pressed on frame 1 (stick up) and special held through frame `held` (stick `stick_x` from frame 2); returns the frame
/// the flight began and the scamp's speed on it.
fn launch(sim: &mut Sim, held: u32, stick_x: i8) -> (u32, Fx, Fx) {
    for frame in 1..=80u32 {
        let input = if frame == 1 {
            inp(0, 127, SPECIAL)
        } else {
            inp(stick_x, 0, if frame <= held { SPECIAL } else { 0 })
        };
        sim.tick(input);
        if frame == 1 {
            assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
        }
        if sim.f().move_id == ROCKET_FLIGHT as u8 && sim.f().state_frame == 1 {
            return (frame, sim.f().vel.x, sim.f().vel.y);
        }
    }
    panic!("never launched");
}

#[test]
fn a_tap_launches_on_frame_8_at_3_1_and_a_full_charge_on_68_at_4_24() {
    let mut sim = setup(Fx::ZERO);
    let (at, vx, vy) = launch(&mut sim, 1, 0);
    assert_eq!(at, 8);
    assert_eq!(vx, Fx::ZERO);
    // (Straight up, on the ground: 3.1 of the reference's units a frame [R], less the first frame's falloff 54 / 54.)
    let s = Fx::from_ratio(31, 80);
    assert!((vy - s).abs() <= Fx::from_ratio(1, 100), "{vy:?} vs {s:?}");
    let mut sim = setup(Fx::ZERO);
    let (at, _, vy) = launch(&mut sim, 200, 0);
    assert_eq!(at, 68, "60 frames of charge at most");
    let full = (Fx::from_ratio(31, 10) + Fx::from_ratio(19, 1000).mul_int(60)) / Fx::from_int(8);
    assert!(
        (vy - full).abs() <= Fx::from_ratio(1, 100),
        "{vy:?} vs {full:?}"
    );
}

#[test]
fn from_the_air_it_charges_falling_slowly_and_launches_1_05_times_faster() {
    let mut sim = setup(Fx::ZERO);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 127, SPECIAL));
    for _ in 0..40 {
        sim.tick(inp(0, 0, SPECIAL));
    }
    let p = &sim.content.fighters[11];
    assert!(
        sim.f().vel.y > -p.max_fall_speed.mul_int(1) / Fx::from_int(2),
        "a slower fall"
    );
    assert!(sim.f().vel.y >= -Fx::from_ratio(5, 80));
    let mut sim = setup(Fx::ZERO);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    let (_, _, vy) = launch(&mut sim, 1, 0);
    let s = Fx::from_ratio(31, 80) * Fx::from_ratio(105, 100);
    assert!((vy - s).abs() <= Fx::from_ratio(1, 100), "{vy:?} vs {s:?}");
}

#[test]
fn the_stick_tilts_the_launch_while_charging_and_steers_the_flight() {
    // Charged 30 frames with the stick forward: tilted forward (up to 0.75 a frame), flying forward and up.
    let mut sim = setup(Fx::ZERO);
    let (_, vx, vy) = launch(&mut sim, 30, 127);
    assert!(vx > Fx::ZERO && vy > Fx::ZERO);
    // Backward: the other way.
    let mut sim = setup(Fx::ZERO);
    let (_, vx, _) = launch(&mut sim, 30, -127);
    assert!(vx < Fx::ZERO);
    // A tap and then the stick forward during the flight: it turns forward as it flies.
    let mut sim = setup(Fx::ZERO);
    launch(&mut sim, 1, 0);
    for _ in 0..20 {
        sim.tick(inp(127, 0, 0));
    }
    assert!(sim.f().vel.x > Fx::ZERO);
}

#[test]
fn the_flight_hits_on_its_frames_and_ends_in_the_special_fall_with_30_frames_of_landing_lag() {
    let sim = setup(Fx::ZERO);
    let w = &sim.content.weapons[usize::from(sim.content.fighters[11].weapon)];
    let flight = w.get(ROCKET_FLIGHT as u8);
    let phases: Vec<(u8, u8, Fx, i16)> = flight
        .hitboxes
        .iter()
        .filter(|h| h.priority == 0)
        .map(|h| (h.start, h.end, h.damage, h.angle))
        .collect();
    assert_eq!(
        phases,
        vec![
            (1, 2, Fx::from_int(10), 361),
            (3, 9, Fx::from_int(8), 80),
            (10, 28, Fx::from_int(6), 70),
        ]
    );
    assert!(flight.hitboxes.iter().all(|h| !h.clanks && h.group == 0));
    assert!(flight.helpless_after);
    assert_eq!(flight.fall_landing_lag, 30);
    // Run it: a tap from the ground rises, falls helpless and lands with 30 frames of lag.
    let mut sim = setup(Fx::ZERO);
    launch(&mut sim, 1, 0);
    let mut helpless = false;
    for _ in 0..300 {
        sim.tick(inp(0, 0, 0));
        helpless |= sim.f().state == S::Helpless;
        if sim.f().state == S::Landing {
            break;
        }
    }
    assert!(helpless);
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 30);
}

#[test]
fn a_tapped_flight_hits_a_fighter_above_for_10_percent() {
    let mut sim = setup(Fx::ZERO);
    sim.put_airborne(1, Fx::ZERO, Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    let before = sim.fighter(1).percent;
    launch(&mut sim, 1, 0);
    for _ in 0..6 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.fighter(1).percent > before);
}

#[test]
fn flying_into_a_wall_sets_the_barrels_off() {
    // Just off the stage's right edge, below its top, flying forward into its side: the crash's 18% explosion.
    let edge = sim_core::Content::placeholder().stage.platforms[0].right;
    let mut sim = setup(Fx::ZERO);
    sim.put_airborne(0, edge + fx(12, 10), -Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = -1;
    sim.put_airborne(1, edge + fx(20, 10), -Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    let mut crashed = false;
    for frame in 1..=120u32 {
        let input = if frame == 1 {
            inp(0, 127, SPECIAL)
        } else {
            // (Toward the stage: the scamp faces left.)
            inp(-127, 0, if frame <= 60 { SPECIAL } else { 0 })
        };
        sim.tick(input);
        if sim.f().move_id == ROCKET_CRASH as u8 {
            crashed = true;
            break;
        }
    }
    assert!(crashed, "it crashed into the wall");
    // The scamp takes 5% itself on the crash's first frame (sim 119: "about 5%" [R]; 5 [E]).
    let own = sim.f().percent;
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().percent - own, Fx::from_int(5));
    // The explosion (radius 20.0 of the reference's units, 2.5 world units round the feet) reaches the duelist 2 units away.
    let at = sim.f().pos;
    sim.put_airborne(1, at.x + Fx::from_int(2), at.y, Fx::ZERO, Fx::ZERO);
    let before = sim.fighter(1).percent;
    for _ in 0..4 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.fighter(1).percent - before,
        Fx::from_int(18) * sim.content.rules.damage_mult
    );
}

#[test]
fn hit_while_charging_the_barrels_fly_off_and_burst_on_the_stage() {
    // The duelist jabs the scamp charging on the ground: the barrels leave, rise, come down and burst on the stage.
    let mut sim = setup(Fx::ZERO);
    sim.stand(1, fx(13, 10), -1);
    sim.tick(inp(0, 127, SPECIAL));
    for _ in 0..8 {
        sim.tick2(inp(0, 0, SPECIAL), inp(0, 0, 0));
    }
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8, "charging");
    sim.tick2(inp(0, 0, SPECIAL), inp(0, 0, ATTACK));
    let mut shed = false;
    let mut burst = false;
    for k in 0..200 {
        // (Still charging while the jab lands.)
        sim.tick2(inp(0, 0, if k < 20 { SPECIAL } else { 0 }), inp(0, 0, 0));
        let b = sim
            .state
            .projectiles
            .iter()
            .find(|p| p.active && p.move_id == BARRELS as u8);
        if let Some(b) = b {
            shed = true;
            assert_eq!(b.owner, 0, "the scamp's");
            assert_eq!(b.kind, BARRELS_KIND);
            burst |= b.bursting;
        }
    }
    assert!(shed, "the barrels flew off");
    assert!(burst, "and burst on the stage");
    // They cannot be shielded (sim 119) [R].
    let w = &sim.content.weapons[usize::from(sim.content.fighters[11].weapon)];
    let spec = w.get(BARRELS as u8).projectile.expect("the barrels");
    assert!(spec.hitbox.unblockable);
    assert_ne!(sim.f().move_id, ROCKET_FLIGHT as u8, "no launch");
}

#[test]
fn a_flinch_in_the_flight_sheds_the_barrels_and_ends_in_the_special_fall() {
    // Flying, the scamp takes a hit too weak to launch it (the duelist's jab at 0%): the barrels fly off and it falls helpless.
    let mut sim = setup(Fx::ZERO);
    launch(&mut sim, 1, 0);
    // (Past the flight's hits, frame 28, so the scamp does not hit the duelist first.)
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
    }
    // A weak push of a hit, applied by hand: 1% with no knockback to speak of.
    let at = sim.f().pos;
    sim.put_airborne(1, at.x + fx(12, 10), at.y, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].facing = -1;
    let w = usize::from(sim.content.fighters[0].weapon);
    let mut jab = sim.content.weapons[w].moves[MoveId::Jab as usize].clone();
    for h in &mut jab.hitboxes {
        h.base_knockback = 0;
        h.knockback_growth = 0;
        h.damage = Fx::ONE;
    }
    sim.content.weapons[w].moves[MoveId::NAir as usize] = jab;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::NAir);
    let mut shed = false;
    let mut helpless = false;
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
        shed |= sim
            .state
            .projectiles
            .iter()
            .any(|p| p.active && p.move_id == BARRELS as u8);
        helpless |= sim.f().state == S::Helpless;
    }
    assert!(sim.f().percent > Fx::ZERO, "hit");
    assert!(shed);
    assert!(helpless);
}

#[test]
fn the_rocket_resimulates_exactly() {
    let content = sim_core::Content::placeholder();
    let chars = [SCAMP, 0, SCAMP, 0];
    let mut rng = Rng::new(3);
    let mut frames: Vec<[Input; MAX_FIGHTERS]> = Vec::new();
    for k in 0..700u32 {
        let mut i = [Input::default(); MAX_FIGHTERS];
        for (n, slot) in i.iter_mut().enumerate() {
            let x = (rng.range(255) as i32 - 127) as i8;
            *slot = if (k + n as u32 * 37) % 120 < 2 {
                inp(x, 127, SPECIAL)
            } else if (k + n as u32 * 37) % 120 < 30 {
                inp(x, 0, SPECIAL)
            } else {
                inp(x, 0, if rng.range(8) == 0 { ATTACK } else { 0 })
            };
        }
        frames.push(i);
    }
    let mut straight = GameState::new(&content, 3, chars);
    for i in &frames {
        step(&mut straight, &content, i);
    }
    let mut a = GameState::new(&content, 3, chars);
    for i in &frames[..350] {
        step(&mut a, &content, i);
    }
    let mut b = a;
    for i in &frames[350..] {
        step(&mut b, &content, i);
    }
    assert_eq!(b.checksum(), straight.checksum());
}
