//! The Pockets' phase-1 specials (class 10, the settler): the rocket (side special: let go, ridden, one at a time) and the balloon
//! flight (up special, E41: the balloons, their pop, the helium, the rise, drift and fall). Reference data and the [E] readings:
//! the eleventh moveset's `REFERENCE_DATA.md` (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SPECIAL};
use sim_core::moves::pockets_slots::{
    balloon_block, FLIGHT, RIDE, RIDE_BOUNCE, RIDE_CRASH, RIDE_HOP, ROCKET_KIND,
};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{step, Content, Fx, GameState, Projectile, Rng, Vec2, MAX_FIGHTERS};

const SETTLER: u8 = 10;

/// The settler (player 0) at 0 facing right and a duelist (player 1) far to the right; the others out.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SETTLER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

fn rocket(sim: &Sim) -> Option<Projectile> {
    sim.state
        .projectiles
        .iter()
        .find(|p| p.active && p.owner == 0 && p.kind == ROCKET_KIND)
        .copied()
}

/// Side special tapped (pressed on tick 1 only, the stick forward that tick).
fn tap_rocket(sim: &mut Sim) {
    let x = 127 * sim.f().facing;
    sim.tick(inp(x, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
}

// ---- The rocket ----------------------------------------------------------------------------------------------------------------

#[test]
fn the_rocket_sits_still_until_frame_52_then_flies_and_control_comes_back_on_36() {
    let mut sim = duel();
    sim.state.fighters[1].invuln = 255;
    tap_rocket(&mut sim);
    assert_eq!(sim.f().state, S::Attack);
    let mut appeared = None;
    let mut moving = None;
    let mut free = None;
    let mut x0 = Fx::ZERO;
    for t in 2..=80u32 {
        sim.tick(inp(0, 0, 0));
        if let Some(r) = rocket(&sim) {
            if appeared.is_none() {
                appeared = Some(t);
                x0 = r.pos.x;
            }
            if moving.is_none() && r.pos.x - x0 > fx(1, 100) {
                moving = Some(t);
            }
        }
        if free.is_none() && sim.f().state != S::Attack {
            free = Some(t + 1);
        }
    }
    // It appears on frame 20 (move frame 19) and first moves on its age 32, the reference's frame 52 [R: the hit window].
    assert_eq!(appeared, Some(20));
    assert_eq!(moving, Some(52));
    assert_eq!(free, Some(36), "FAF 36");
}

#[test]
fn the_rockets_flying_hit_deals_7_percent_from_frame_52_and_it_is_gone() {
    let mut sim = duel();
    tap_rocket(&mut sim);
    let mut hit_on = None;
    for t in 2..=120u32 {
        // The target held on the rocket (once it is out), not hit before 52.
        if let Some(r) = rocket(&sim) {
            sim.put_airborne(
                1,
                r.pos.x + fx(2, 10),
                (r.pos.y - fx(12, 10)).max(fx(1, 10)),
                Fx::ZERO,
                Fx::ZERO,
            );
        }
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            hit_on = Some(t);
            break;
        }
    }
    assert_eq!(hit_on, Some(52));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 70));
    // Used up on the fighter: no burst, nothing left.
    assert!(rocket(&sim).is_none());
}

#[test]
fn the_rocket_slows_with_a_5_percent_hit_then_bursts_for_12_at_the_end() {
    // Late in its flight (age 120) its hit is the 5% one; left alone it bursts at age 157 (the reference's frame 177) with 12%.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    tap_rocket(&mut sim);
    let mut ages = Vec::new();
    for _ in 0..200 {
        sim.tick(inp(0, 0, 0));
        if let Some(r) = rocket(&sim) {
            ages.push((r.age, r.bursting, r.vel));
        }
    }
    let flying = ages
        .iter()
        .find(|(a, ..)| *a == 60)
        .expect("flying at 60")
        .2;
    let slowing = ages
        .iter()
        .find(|(a, b, _)| *a == 130 && !b)
        .expect("slowing at 130")
        .2;
    assert!(
        slowing.x.abs() < flying.x.abs() && slowing.y < Fx::ZERO,
        "it slows and drops"
    );
    assert!(
        ages.iter().any(|(_, b, _)| *b),
        "it bursts at the end of its flight"
    );
    // The hits by age, from the data.
    let w = &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SETTLER)].weapon)];
    let spec = w.get(MoveId::SideSpecial as u8).projectile.unwrap();
    assert_eq!(spec.hit_at(31).0.radius, Fx::ZERO, "no hit before 52");
    assert_eq!(spec.hit_at(32).0.damage, Fx::from_int(7));
    assert_eq!(spec.hit_at(97).0.damage, Fx::from_int(5));
    let burst = spec.burst.unwrap();
    assert!(burst.on_wall && burst.on_expire && !burst.on_hit);
    assert_eq!(
        burst.hit_at(1).map(|h| (h.damage, h.angle, h.clanks)),
        Some((Fx::from_int(12), 68, false))
    );
    assert_eq!(spec.life, 157);
}

#[test]
fn only_one_rocket_is_out_at_a_time() {
    let mut sim = duel();
    sim.state.fighters[1].invuln = 255;
    tap_rocket(&mut sim);
    sim.ticks(40, inp(0, 0, 0));
    assert!(rocket(&sim).is_some());
    // A second press: the move plays, but no second rocket appears.
    tap_rocket(&mut sim);
    sim.ticks(30, inp(0, 0, 0));
    let n = sim
        .state
        .projectiles
        .iter()
        .filter(|p| p.active && p.kind == ROCKET_KIND)
        .count();
    assert_eq!(n, 1);
}

#[test]
fn started_in_the_air_the_settler_stops() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(8), fx(2, 10), fx(3, 10));
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().state, S::Attack);
    assert!(sim.f().vel.x.abs() < fx(1, 100), "{:?}", sim.f().vel);
    assert!(sim.f().vel.y <= Fx::ZERO);
}

/// Side special held from tick 1 (the ride), the stick at `x`; ticks until `stop` is true or `n` ticks.
fn ride(sim: &mut Sim, x: i8, n: u32, stop: impl Fn(&Sim) -> bool) -> u32 {
    let first = sim.f().state != S::Attack;
    for t in 1..=n {
        // (The first tick's stick forward starts the side special.)
        let sx = if first && t == 1 {
            127 * sim.f().facing
        } else {
            x
        };
        sim.tick(inp(sx, 0, SPECIAL));
        if stop(sim) {
            return t;
        }
    }
    n
}

#[test]
fn held_the_settler_rides_the_rocket_from_frame_16_and_flies_from_52() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    let t = ride(&mut sim, 0, 30, |s| s.f().move_id == RIDE as u8);
    assert_eq!(t, 16, "on the reference's frame 16");
    assert!(rocket(&sim).is_none(), "no separate rocket");
    let x0 = sim.f().pos.x;
    ride(&mut sim, 0, 35, |_| false);
    assert!((sim.f().pos.x - x0).abs() < fx(1, 100), "still until 52");
    let x1 = sim.f().pos.x;
    ride(&mut sim, 0, 10, |_| false);
    assert!(
        sim.f().pos.x - x1 > fx(18, 10),
        "flying forward at 1.7 a frame"
    );
}

#[test]
fn a_ridden_rocket_hits_for_15_4_percent_and_bounces_the_settler_up() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(5), -1);
    ride(&mut sim, 0, 120, |s| s.fighter(1).percent > Fx::ZERO);
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 154));
    // (Through the hit's hitlag.)
    sim.ticks(16, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, RIDE_BOUNCE as u8);
    sim.ticks(30, inp(0, 0, 0));
    assert_ne!(sim.f().state, S::Helpless, "a hit leaves the settler free");
}

#[test]
fn jumping_off_or_riding_to_the_end_leaves_the_settler_helpless() {
    // Jump: the hop off, then helpless.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    ride(&mut sim, 0, 60, |_| false);
    sim.tick(inp(0, 0, JUMP));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, RIDE_HOP as u8);
    sim.ticks(20, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Helpless);
    // To the end (high in the air, nothing in the way): the 18% burst on its frames 177-179, then helpless.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-10), 1);
    sim.put_airborne(0, Fx::from_int(-9), Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    let w = &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SETTLER)].weapon)];
    let blast = w
        .get(RIDE as u8)
        .hitboxes
        .iter()
        .find(|h| h.group == 1)
        .copied()
        .unwrap();
    assert_eq!(
        (blast.start + 16, blast.end + 16, blast.damage),
        (177, 179, Fx::from_int(18))
    );
    let t = ride(&mut sim, 0, 250, |s| s.f().state != S::Attack);
    assert_eq!(sim.f().state, S::Helpless, "after {t} ticks");
}

#[test]
fn a_ridden_rocket_crashes_into_a_wall() {
    // Below the stage's top, beside it, facing it: the rocket flies into the stage's side.
    let mut sim = duel();
    let stage = sim.content.stage.platforms[0];
    sim.put_airborne(
        0,
        stage.left - Fx::from_int(8),
        stage.y - Fx::from_int(2),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = 1;
    let t = ride(&mut sim, 0, 200, |s| s.f().move_id == RIDE_CRASH as u8);
    assert!(t < 200, "crashed");
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Helpless);
}

#[test]
fn a_let_go_rocket_bursts_against_a_wall() {
    let mut sim = duel();
    let stage = sim.content.stage.platforms[0];
    sim.put_airborne(
        0,
        stage.left - Fx::from_int(6),
        stage.y - Fx::from_int(2),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = 1;
    tap_rocket(&mut sim);
    let mut burst = false;
    for _ in 0..150 {
        // (Keep the settler out of the way, hovering where it was.)
        sim.state.fighters[0].vel = Vec2::ZERO;
        sim.tick(inp(0, 0, 0));
        burst |= rocket(&sim).is_some_and(|r| r.bursting);
    }
    assert!(burst);
}

// ---- The balloons (E41) --------------------------------------------------------------------------------------------------------

fn up_special(sim: &mut Sim) {
    sim.tick(inp(0, 127, SPECIAL));
}

#[test]
fn the_balloon_flight_takes_off_with_two_balloons_and_uses_its_helium() {
    let mut sim = duel();
    up_special(&mut sim);
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    assert_eq!(sim.f().balloons, 2);
    sim.ticks(10, inp(0, 0, 0));
    assert!(!sim.f().grounded(), "took off");
    assert_eq!(sim.f().balloons, 2);
    assert_eq!(sim.f().helium_used, 11 * 16, "a frame of helium a frame");
}

#[test]
fn flapping_rises_and_doing_nothing_sinks() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    up_special(&mut sim);
    sim.ticks(40, inp(0, 0, 0));
    let y0 = sim.f().pos.y;
    sim.ticks(40, inp(0, 0, 0));
    assert!(sim.f().pos.y < y0, "sinks with nothing pressed");
    let y1 = sim.f().pos.y;
    for t in 0..30 {
        sim.tick(inp(0, 0, if t % 6 == 0 { SPECIAL } else { 0 }));
    }
    assert!(sim.f().pos.y > y1, "rises flapping");
    // The stick steers: held forward it drifts forward.
    let x0 = sim.f().pos.x;
    sim.ticks(20, inp(127, 0, 0));
    assert!(sim.f().pos.x > x0);
}

#[test]
fn the_helium_runs_out_after_300_frames_and_the_balloons_let_go() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    up_special(&mut sim);
    let mut t = 1;
    while sim.f().state == S::Attack && t < 400 {
        // (Flap now and then to stay up.)
        sim.tick(inp(0, 0, if t % 12 == 0 { SPECIAL } else { 0 }));
        t += 1;
    }
    assert_eq!(t, 300, "a 300-frame tank");
    assert_eq!(sim.f().state, S::Helpless);
    assert_eq!(sim.f().balloons, 0);
    // The flight goes on past one move's length (as `FLIGHT`).
    let _ = FLIGHT;
}

#[test]
fn attack_lets_the_balloons_go() {
    let mut sim = duel();
    up_special(&mut sim);
    sim.ticks(20, inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Helpless);
    assert_eq!(sim.f().balloons, 0);
}

#[test]
fn landing_with_the_balloons_costs_20_frames_and_from_the_fall_30() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, fx(5, 100), Fx::ZERO, fx(-1, 10));
    let f = &mut sim.state.fighters[0];
    f.state = S::Attack;
    f.move_id = MoveId::UpSpecial as u8;
    f.state_frame = 40;
    f.balloons = 2;
    sim.tick(inp(0, 0, 0));
    assert_eq!((sim.f().state, sim.f().lag), (S::Landing, 20));
    let w = &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SETTLER)].weapon)];
    assert_eq!(w.get(MoveId::UpSpecial as u8).fall_landing_lag, 30);
}

/// A shot of player 1 (a settler: its forward air's pellet, at the age of its small 4% hit, standing still) at `at`.
fn shot_at(sim: &mut Sim, at: Vec2) {
    let slot = sim
        .state
        .projectiles
        .iter()
        .position(|p| !p.active)
        .unwrap();
    sim.state.projectiles[slot] = Projectile {
        active: true,
        owner: 1,
        origin: 1,
        power: 100,
        move_id: MoveId::FAir as u8,
        pos: at,
        vel: Vec2::ZERO,
        age: 5,
        life: 15,
        size: 100,
        facing: -1,
        ..Projectile::default()
    };
}

/// Two settlers: player 0 flying high on its balloons (3 frames in), player 1 out of the way.
fn flying() -> Sim {
    let mut sim = Sim::with_chars([SETTLER, SETTLER, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::from_int(-2), Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    up_special(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    sim
}

#[test]
fn a_hit_on_the_balloons_pops_one_and_the_last_pops_into_the_special_fall() {
    let mut sim = flying();
    assert_eq!(sim.f().balloons, 2);
    let b = balloon_block();
    for left in [1u8, 0] {
        let me = sim.f().pos;
        // On the balloons' circle, above the head (out of the body's reach).
        shot_at(&mut sim, Vec2::new(me.x + b.x, me.y + b.y + fx(2, 10)));
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().balloons, left);
        assert_eq!(sim.f().percent, Fx::ZERO, "the body was not hit");
        assert!(
            !sim.state
                .projectiles
                .iter()
                .any(|p| p.active && p.owner == 1),
            "the shot is used up"
        );
        if left == 1 {
            assert_eq!(sim.f().state, S::Attack, "still flying on one");
        }
    }
    assert_eq!(sim.f().state, S::Helpless);
    assert!(
        sim.f().vel.y > Fx::ZERO,
        "a little hop as the last one pops"
    );
}

#[test]
fn a_hit_on_the_body_ends_the_flight_and_the_balloons_go() {
    let mut sim = flying();
    let me = sim.f().pos;
    shot_at(&mut sim, Vec2::new(me.x, me.y + fx(12, 10)));
    sim.tick(inp(0, 0, 0));
    assert!(sim.f().percent > Fx::ZERO);
    sim.ticks(12, inp(0, 0, 0));
    assert_eq!(sim.f().balloons, 0);
    assert_ne!(sim.f().state, S::Attack);
}

#[test]
fn a_tank_below_half_gives_one_balloon_and_it_refills_out_of_the_flight() {
    let mut sim = duel();
    sim.state.fighters[0].helium_used = 200 * 16;
    up_special(&mut sim);
    assert_eq!(sim.f().balloons, 1, "100 frames left: one balloon");
    // Standing, it refills at 6 sixteenths a frame.
    let mut sim = duel();
    sim.state.fighters[0].helium_used = 100 * 16;
    sim.ticks(10, inp(0, 0, 0));
    assert_eq!(sim.f().helium_used, 100 * 16 - 60);
    // An empty tank: no flight at all (the special fall at once in the air).
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].helium_used = 300 * 16;
    up_special(&mut sim);
    assert_eq!(sim.f().state, S::Helpless);
}

#[test]
fn the_balloons_and_the_rocket_resimulate_exactly() {
    // Random play with two settlers (rockets, balloons, turnips): a run restored from a mid-run snapshot ends the same.
    let content = Content::placeholder();
    let chars = [SETTLER, SETTLER, 0, 0];
    for seed in 0..8u64 {
        let mut inputs = sim_core::fuzz::random_inputs(&mut Rng::new(seed), 700);
        // (Plenty of specials: every 30 frames player 0 presses special with the stick up or forward.)
        for (i, frame) in inputs.iter_mut().enumerate() {
            if i % 30 == 0 {
                frame[0] = inp(
                    if i % 60 == 0 { 0 } else { 127 },
                    if i % 60 == 0 { 127 } else { 0 },
                    SPECIAL,
                );
            }
        }
        let play = |frames: &[[sim_core::Input; MAX_FIGHTERS]]| {
            let mut s = GameState::new(&content, seed, chars);
            for i in frames {
                step(&mut s, &content, i);
            }
            s
        };
        let straight = play(&inputs);
        let mut s = GameState::new(&content, seed, chars);
        for i in &inputs[..300] {
            step(&mut s, &content, i);
        }
        let mut r = s;
        for i in &inputs[300..] {
            step(&mut r, &content, i);
        }
        assert_eq!(r.checksum(), straight.checksum(), "seed {seed}");
    }
}
