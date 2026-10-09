//! Hit feel, second pass (sim v28): the launch speed-up of strong hits, weight-free launch gravity and the vertical-launch fall speed,
//! electric hits and their automatic SDI, per-hitbox hitlag, survival DI growing over a combo, clanks and grab parries, tech timing
//! with its lockout, wall and ceiling techs, and bounces off walls and the floor.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::hitstun_frames;
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD};
use sim_core::moves::{MoveId, EFFECT_ELECTRIC};
use sim_core::state::{FighterState as S, NONE};
use sim_core::trig::Angle;
use sim_core::{Fx, Vec2};

fn duel() -> Sim {
    let mut sim = Sim::duel(fx(14, 10));
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// Puts player 1 high in the air, about to be launched by a hit of `kb` knockback at `degrees` (its hitlag ends next frame).
fn pending_launch(sim: &mut Sim, kb: i32, degrees: i32) {
    // The other fighter stands well out of the way (taking it out of the match would end the match).
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
}

/// Frames player 1 stays in hitstun after the launch.
fn hitstun_length(sim: &mut Sim) -> usize {
    sim.tick(inp(0, 0, 0)); // hitlag ends, the launch starts
    let mut n = 0;
    while sim.fighter(1).state == S::Hitstun && n < 400 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    n
}

#[test]
fn a_strong_launch_is_sped_up_and_its_hitstun_is_shorter() {
    // 145 knockback: 57 frames of hitstun on paper, about 41 in the reference game.
    let mut sim = duel();
    pending_launch(&mut sim, 145, 45);
    let nominal = usize::from(sim.fighter(1).hitstun);
    let real = hitstun_length(&mut sim);
    assert!(
        real < nominal - 10,
        "{real} frames against {nominal} on paper"
    );
    assert!((38..=45).contains(&real), "about 41 frames: {real}");
    // The first frame of the launch covers several frames of knockback.
    let mut sim = duel();
    pending_launch(&mut sim, 145, 45);
    sim.tick(inp(0, 0, 0));
    let before = sim.fighter(1).pos;
    let speed = sim.fighter(1).kb_vel.length();
    sim.tick(inp(0, 0, 0));
    let moved = (sim.fighter(1).pos - before).length();
    assert!(
        moved > speed * fx(25, 10),
        "moved {moved:?} with launch speed {speed:?}"
    );
}

#[test]
fn a_light_launch_is_not_sped_up() {
    let mut sim = duel();
    pending_launch(&mut sim, 50, 45);
    let nominal = usize::from(sim.fighter(1).hitstun);
    let real = hitstun_length(&mut sim);
    assert!(real.abs_diff(nominal) <= 1, "{real} vs {nominal}");
}

#[test]
fn every_fighter_falls_alike_early_in_a_launch_and_a_vertical_launch_falls_at_its_own_speed() {
    let fall = |chars: [u8; 4], degrees: i32, frames: usize| {
        let mut sim = Sim::with_chars(chars);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        pending_launch(&mut sim, 60, degrees);
        for _ in 0..=frames {
            sim.tick(inp(0, 0, 0));
        }
        sim.fighter(1).vel.y
    };
    // A sword fighter and a claws fighter are alike for the first frames of a launch (different gravity otherwise).
    assert_eq!(fall([0, 0, 0, 0], 30, 8), fall([1, 1, 1, 1], 30, 8));
    // Straight up: the fall speed is the vertical-launch one.
    let rules = Sim::new().content.rules;
    for chars in [[0, 0, 0, 0], [1, 1, 1, 1]] {
        let mut sim = Sim::with_chars(chars);
        pending_launch(&mut sim, 80, 90);
        let mut slowest = Fx::ZERO;
        for _ in 0..120 {
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).state != S::Hitstun {
                break;
            }
            slowest = slowest.min(sim.fighter(1).vel.y);
        }
        assert_eq!(slowest, -rules.vertical_launch_fall, "{chars:?}");
    }
}

/// Makes player 0's jab electric (or gives it a hitlag factor).
fn jab_hitboxes(sim: &mut Sim, f: impl Fn(&mut sim_core::moves::Hitbox)) {
    let w = usize::from(sim.content.fighters[0].weapon);
    for hb in &mut sim.content.weapons[w].moves[MoveId::Jab as usize].hitboxes {
        f(hb);
    }
}

fn jab(sim: &mut Sim, p1: sim_core::Input) {
    sim.tick2(inp(0, 0, ATTACK), p1);
    let before = sim.fighter(1).percent;
    for _ in 0..20 {
        if sim.fighter(1).percent > before {
            return;
        }
        sim.tick2(inp(0, 0, 0), p1);
    }
    panic!("the jab never connected");
}

#[test]
fn an_electric_hit_lasts_half_as_long_again_and_drifts_the_victim_with_the_stick() {
    let mut plain = duel();
    jab(&mut plain, inp(0, 0, 0));
    let mut zap = duel();
    jab_hitboxes(&mut zap, |hb| hb.effect = EFFECT_ELECTRIC);
    jab(&mut zap, inp(0, 0, 0));
    let (a, b) = (plain.fighter(1).hitlag, zap.fighter(1).hitlag);
    assert!(b > a && b <= a * 3 / 2 + 1, "{a} then {b}");
    assert_eq!(zap.fighter(1).asdi, 2);
    // Holding left through the hit, the electric victim ends its hitlag further left than a plain one (two automatic-SDI drifts).
    let end_x = |electric: bool| {
        let mut sim = duel();
        if electric {
            jab_hitboxes(&mut sim, |hb| hb.effect = EFFECT_ELECTRIC);
        }
        jab(&mut sim, inp(0, 0, 0));
        while sim.fighter(1).hitlag > 0 {
            sim.tick2(inp(0, 0, 0), inp(-100, 0, 0));
        }
        sim.fighter(1).pos.x
    };
    let drift = end_x(false) - end_x(true);
    let asdi = zap.content.rules.asdi_distance.mul_int(2);
    assert!(
        (drift - asdi).abs() < fx(1, 1000),
        "drifted {drift:?}, expected {asdi:?}"
    );
}

#[test]
fn a_hitbox_can_set_its_own_hitlag() {
    let mut sim = duel();
    jab_hitboxes(&mut sim, |hb| hb.hitlag = 0);
    jab(&mut sim, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).hitlag, 0);
    assert_eq!(sim.f().hitlag, 0);
    let mut sim = duel();
    jab_hitboxes(&mut sim, |hb| hb.hitlag = 200);
    let mut normal = duel();
    jab(&mut sim, inp(0, 0, 0));
    jab(&mut normal, inp(0, 0, 0));
    assert!(sim.fighter(1).hitlag >= normal.fighter(1).hitlag * 2 - 1);
}

#[test]
fn survival_di_goes_further_late_in_a_long_combo() {
    let nudge = |hits: u8| {
        let mut sim = duel();
        sim.content.rules.hitlag_base = Fx::from_int(20);
        jab(&mut sim, inp(0, 0, 0));
        sim.state.fighters[1].hits_taken = hits;
        let x0 = sim.fighter(1).pos.x;
        sim.tick2(inp(0, 0, 0), inp(127, 0, 0));
        sim.fighter(1).pos.x - x0
    };
    let (fresh, late) = (nudge(1), nudge(10));
    assert!(fresh > Fx::ZERO);
    let ratio = late / fresh;
    assert!(
        (ratio - fx(13225, 10000)).abs() < fx(1, 100),
        "1.15 squared: {ratio:?}"
    );
}

#[test]
fn two_jabs_that_meet_clank_and_both_fighters_rebound() {
    // Both fighters are sword fighters, close enough that their jabs meet.
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(30, 10), -1);
    sim.tick2(inp(0, 0, ATTACK), inp(0, 0, ATTACK));
    let mut rebounded = false;
    for _ in 0..12 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.f().state == S::Rebound {
            rebounded = true;
            break;
        }
    }
    assert!(
        rebounded,
        "no clank: {:?} {:?}",
        sim.f().state,
        sim.fighter(1).state
    );
    assert_eq!(sim.fighter(1).state, S::Rebound);
    assert_eq!(sim.f().percent, Fx::ZERO, "neither is hurt");
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    // Both are free again on the same frame.
    let mut n = 0;
    while sim.f().state == S::Rebound && n < 200 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        n += 1;
    }
    assert_ne!(sim.fighter(1).state, S::Rebound);
}

#[test]
fn a_much_stronger_attack_wins_a_clank_and_carries_on() {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    // Player 1's jab is made far weaker than player 0's (a 20% jab against a 3% one).
    let w = usize::from(sim.content.fighters[0].weapon);
    let jab = &mut sim.content.weapons[w].moves[MoveId::Jab as usize];
    let strong = jab.clone();
    let mut strong = strong;
    for hb in &mut strong.hitboxes {
        hb.damage = Fx::from_int(20);
    }
    // Player 0 uses the up tilt slot for the strong copy so the two fighters differ.
    sim.content.weapons[w].moves[MoveId::Jab as usize] = strong.clone();
    let mut weak = strong;
    for hb in &mut weak.hitboxes {
        hb.damage = Fx::from_int(3);
    }
    sim.content.weapons.push(sim.content.weapons[w].clone());
    let w2 = sim.content.weapons.len() - 1;
    sim.content.weapons[w2].moves[MoveId::Jab as usize] = weak;
    let mut p = sim.content.fighters[0];
    p.weapon = w2 as u8;
    sim.content.fighters.push(p);
    sim.state.fighters[1].char_id = (sim.content.fighters.len() - 1) as u8;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(30, 10), -1);
    sim.tick2(inp(0, 0, ATTACK), inp(0, 0, ATTACK));
    for _ in 0..12 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.fighter(1).state == S::Rebound {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Rebound, "the weaker one rebounds");
    assert_eq!(sim.f().state, S::Attack, "the stronger one carries on");
}

#[test]
fn two_fighters_grabbing_each_other_at_once_both_let_go() {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(15, 10), -1);
    sim.tick2(inp(0, 0, GRAB), inp(0, 0, GRAB));
    for _ in 0..20 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.f().state == S::Rebound {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Rebound);
    assert_eq!(sim.fighter(1).state, S::Rebound);
    assert_eq!(sim.f().percent, sim.content.rules.damage_mult, "1% each");
}

/// Player 1 tumbling at speed `v` (world units a frame), high above the floor's level so nothing else gets in the way.
fn tumbling(sim: &mut Sim, at: Vec2, v: Vec2) {
    // The other fighter stands well out of the way (taking it out of the match would end the match).
    sim.stand(0, Fx::from_int(9), -1);
    let f = &mut sim.state.fighters[1];
    f.pos = at;
    f.vel = Vec2::ZERO;
    f.platform = NONE;
    f.state = S::Hitstun;
    f.state_frame = 0;
    f.hitstun = 120;
    f.kb_vel = v;
    f.tumble = true;
    f.launch_pending = false;
    f.hitlag = 0;
}

#[test]
fn a_tech_press_counts_for_eleven_frames_and_mashing_locks_it_out() {
    let rules = Sim::new().content.rules;
    assert_eq!(rules.tech_window, 11);
    // Falling onto the stage: a press 8 frames before touching down techs.
    let land_after = |presses: &[usize]| {
        let mut sim = duel();
        tumbling(
            &mut sim,
            Vec2::new(Fx::ZERO, Fx::from_int(8)),
            Vec2::new(Fx::ZERO, fx(-1, 10)),
        );
        // Not a tumble, so it lands rather than bouncing off the floor.
        sim.state.fighters[1].tumble = false;
        let mut t = 0;
        while sim.fighter(1).platform == NONE && t < 200 {
            let b = if presses.contains(&t) { SHIELD } else { 0 };
            sim.tick2(inp(0, 0, 0), inp(0, 0, b));
            t += 1;
        }
        (t, sim.fighter(1).state)
    };
    let (touch, _) = land_after(&[]);
    assert!(touch > 15);
    assert_eq!(land_after(&[]).1, S::Knockdown);
    assert_ne!(land_after(&[touch - 8]).1, S::Knockdown, "teched");
    // A press too early, then the right one: the first locks the second out.
    assert_eq!(land_after(&[touch - 20, touch - 6]).1, S::Knockdown);
}

#[test]
fn a_tumble_into_a_wall_bounces_off_and_a_tech_holds_on() {
    // The main stage block's left side is a wall at x = -11 below the top (y = 0).
    let start = Vec2::new(Fx::from_int(-14), Fx::from_int(-3));
    let toward = Vec2::new(fx(5, 10), Fx::ZERO);
    let mut sim = duel();
    tumbling(&mut sim, start, toward);
    for _ in 0..12 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert!(
        sim.fighter(1).kb_vel.x < Fx::ZERO,
        "bounced back: {:?}",
        sim.fighter(1).kb_vel
    );
    let mut sim = duel();
    tumbling(&mut sim, start, toward);
    sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    let mut teched = false;
    for _ in 0..10 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.fighter(1).state == S::WallTech {
            teched = true;
            break;
        }
    }
    assert!(teched, "{:?}", sim.fighter(1).state);
    assert!(sim.fighter(1).invuln > 0, "intangible");
    // Holding jump as the tech ends kicks away from the wall.
    for _ in 0..30 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, JUMP));
    }
    assert_eq!(sim.fighter(1).state, S::Airborne);
    assert!(sim.fighter(1).vel.x < Fx::ZERO, "away from the wall");
}

#[test]
fn a_hard_spike_into_the_floor_bounces_instead_of_lying_down() {
    let mut sim = duel();
    tumbling(
        &mut sim,
        Vec2::new(Fx::ZERO, fx(20, 10)),
        Vec2::new(Fx::ZERO, fx(-6, 10)),
    );
    let mut bounced = false;
    for _ in 0..20 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        if sim.fighter(1).kb_vel.y > Fx::ZERO {
            bounced = true;
            break;
        }
    }
    assert!(bounced);
    assert_eq!(sim.fighter(1).state, S::Hitstun);
    assert_eq!(sim.fighter(1).platform, NONE);
}
