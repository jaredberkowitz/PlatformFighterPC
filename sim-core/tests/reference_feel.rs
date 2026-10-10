//! Closer to the reference game (sim v34): DI bends a launch by about 10 degrees at most, launch speed influence, crouching under
//! high hits, limbs that can be hit while they attack, and the bruiser's heavyweight attributes.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::{hitstun_frames, hurtboxes, params_of, weapon_of};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::trig::Angle;
use sim_core::{Fx, Vec2};

fn duel(chars: [u8; 4]) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(14, 10), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// The velocity player 1 is launched at by a hit of `kb` knockback at `degrees`, holding `stick` as its hitlag ends.
fn launch(kb: i32, degrees: i32, stick: (i8, i8)) -> Vec2 {
    let mut sim = duel([0, 0, 0, 0]);
    // The attacker out of the way, the victim high in the air.
    sim.stand(0, Fx::from_int(9), -1);
    sim.put_airborne(1, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    let f = &mut sim.state.fighters[1];
    f.state = S::Hitstun;
    f.state_frame = 0;
    f.launch_kb = Fx::from_int(kb);
    f.launch_angle = Angle::from_degrees(degrees).raw();
    f.hitstun = hitstun_frames(f.launch_kb, Fx::ONE);
    f.launch_pending = true;
    f.hitlag = 1;
    sim.tick2(inp(0, 0, 0), inp(stick.0, stick.1, 0));
    sim.fighter(1).kb_vel
}

/// `a` as a multiple of `b`'s length.
fn ratio(a: Vec2, b: Vec2) -> Fx {
    a.length() / b.length()
}

#[test]
fn di_bends_a_launch_by_about_ten_degrees_at_most() {
    let none = launch(120, 45, (0, 0));
    // Fully across the launch (up and back, square to a launch up and away).
    let across = launch(120, 45, (-90, 90));
    // The sine of the angle between them: 10 degrees is 0.174, the older game's 18 would be 0.309.
    let sine = (none.x * across.y - none.y * across.x) / (none.length() * across.length());
    assert!(sine > fx(15, 100) && sine < fx(19, 100), "{sine:?}");
}

#[test]
fn holding_up_or_down_speeds_up_or_slows_a_tumbling_launch() {
    let none = launch(120, 40, (0, 0));
    let up = ratio(launch(120, 40, (0, 127)), none);
    let down = ratio(launch(120, 40, (0, -127)), none);
    // The reference game's 1.095 and 0.92 (DI bends these a little too, and the first frame's decay blurs them).
    assert!(up > fx(107, 100) && up < fx(112, 100), "{up:?}");
    assert!(down > fx(90, 100) && down < fx(94, 100), "{down:?}");
    // Half way up is about half the effect.
    let half = ratio(launch(120, 40, (0, 64)), none);
    assert!(half > fx(102, 100) && half < fx(106, 100), "{half:?}");
}

#[test]
fn no_launch_speed_influence_near_vertical_or_without_tumble() {
    let near = |a: Vec2, b: Vec2| (a.length() - b.length()).abs() < fx(1, 200);
    // Straight up (and within 25 degrees of it, after DI).
    assert!(near(launch(120, 80, (0, 127)), launch(120, 80, (0, 0))));
    assert!(near(launch(120, 100, (0, -127)), launch(120, 100, (0, 0))));
    // A launch too weak to tumble.
    assert!(near(launch(60, 40, (0, 127)), launch(60, 40, (0, 0))));
}

#[test]
fn a_crouch_brings_the_hurtboxes_down() {
    let mut sim = duel([0, 0, 0, 0]);
    let top = |sim: &Sim| {
        let f = sim.fighter(0);
        let p = params_of(&sim.content, f);
        hurtboxes(f, p, weapon_of(&sim.content, p))
            .iter()
            .map(|(c, r)| c.y + *r - f.pos.y)
            .max()
            .unwrap()
    };
    let standing = top(&sim);
    sim.state.fighters[0].state = S::Crouch;
    let crouching = top(&sim);
    assert!(
        standing - crouching > fx(6, 10),
        "standing reaches {standing:?}, crouching {crouching:?}"
    );
    // A hit at head height that lands on the standing fighter passes over the crouch.
    let head = Vec2::new(Fx::ZERO, fx(21, 10));
    let touches = |sim: &Sim| {
        let f = sim.fighter(0);
        let p = params_of(&sim.content, f);
        hurtboxes(f, p, weapon_of(&sim.content, p))
            .iter()
            .any(|(c, r)| (*c - head).length() < *r + fx(3, 10))
    };
    sim.state.fighters[0].state = S::Idle;
    assert!(touches(&sim));
    sim.state.fighters[0].state = S::Crouch;
    assert!(!touches(&sim));
}

/// The fighter's fourth hurtbox (the limb) and its middle one, with `char_id` in the forward tilt's first active frame.
fn limb_in_ftilt(char_id: u8) -> ((Vec2, Fx), (Vec2, Fx)) {
    let mut sim = duel([char_id, 0, 0, 0]);
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = MoveId::FTilt as u8;
    let mv = {
        let p = params_of(&sim.content, &sim.state.fighters[0]);
        weapon_of(&sim.content, p).get(MoveId::FTilt as u8).clone()
    };
    let first = mv.hitboxes.iter().map(|h| h.start).min().unwrap();
    sim.state.fighters[0].state_frame = u16::from(first);
    let f = sim.fighter(0);
    let p = params_of(&sim.content, f);
    let hurt = hurtboxes(f, p, weapon_of(&sim.content, p));
    (hurt[3], hurt[1])
}

#[test]
fn a_claw_or_kick_can_be_hit_while_it_is_out_but_a_blade_cannot() {
    // The claws: the arm reaches out toward the hit.
    let (limb, middle) = limb_in_ftilt(1);
    assert!(limb.0.x > middle.0.x + Fx::ONE, "{limb:?}");
    // The sword: nothing past the body (the fourth circle is the middle one).
    let (limb, middle) = limb_in_ftilt(0);
    assert_eq!(limb, middle);
}

#[test]
fn the_bruiser_has_the_heavyweight_hammer_fighters_attributes() {
    let c = sim_core::Content::placeholder();
    let (duelist, brawler, bruiser) = (&c.fighters[0], &c.fighters[1], &c.fighters[2]);
    assert_eq!(bruiser.weight, Fx::from_int(127));
    // Four jumps in the air, a heavy fall, and the slowest drift.
    assert_eq!(bruiser.air_jumps, 4);
    assert!(
        bruiser.max_fall_speed > brawler.max_fall_speed
            && bruiser.max_fall_speed > duelist.max_fall_speed
    );
    assert!(bruiser.air_speed < duelist.air_speed && bruiser.air_speed < brawler.air_speed);
}

#[test]
fn the_bruiser_uses_all_four_air_jumps() {
    let mut sim = duel([2, 0, 0, 0]);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    let mut jumps = 0;
    for _ in 0..6 {
        let before = sim.f().vel.y;
        sim.tick(inp(0, 0, sim_core::input::buttons::JUMP));
        if sim.f().vel.y > before + fx(1, 10) {
            jumps += 1;
        }
        for _ in 0..20 {
            sim.tick(inp(0, 0, 0));
        }
    }
    assert_eq!(jumps, 4);
}
