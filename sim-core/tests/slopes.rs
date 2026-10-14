//! Slopes (sim 124, the run's R2a: `docs/modes/RUNNER_WORLD.md` 2.2): a platform's top may rise or fall (`Platform::rise`, at most 1
//! in 2). A grounded fighter walks, dashes and runs along it at its ground speed with its feet on the surface; walking across the
//! joint between a ramp and the ground beyond keeps it on the ground; it lands on a slope, jumps off it, wavedashes and techs along
//! it; and a low hit into a rising slope can trip, as the reference's moves noted ("trips only on a slope"). Versus stages stay flat
//! and behave as before (every other test file).

mod common;

use common::{fx, inp, Sim};
use sim_core::content::{Ledge, Platform};
use sim_core::input::buttons::{JUMP, SHIELD};
use sim_core::moves::MoveId;
use sim_core::state::{FighterState as S, NONE};
use sim_core::{Fx, Vec2};

const FLOOR: i8 = 0;
const RAMP: i8 = 1;
const TOP: i8 = 2;
const FLIGHT: i8 = 3;

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

/// A slope stage: flat ground, a solid ramp rising 5 over 10 (1 in 2, about 26.6 degrees: the steepest allowed) onto a raised
/// flat roof, and a pass-through flight of stairs in the air over the ground (6 to 9 over 10). No ledges. Player 1 far off.
fn slope_sim(chars: [u8; 4]) -> Sim {
    let mut sim = Sim::with_chars(chars);
    let solid = |left: i32, right: i32, y: i32, rise: i32| Platform {
        left: int(left),
        right: int(right),
        y: int(y),
        bottom: int(-10),
        pass_through: false,
        rise: int(rise),
    };
    let s = &mut sim.content.stage;
    s.platforms = vec![
        solid(-40, -5, 0, 0),
        solid(-5, 5, 0, 5),
        solid(5, 40, 5, 0),
        Platform {
            left: int(-30),
            right: int(-20),
            y: int(6),
            bottom: int(6),
            pass_through: true,
            rise: int(3),
        },
    ];
    s.ledges = vec![
        Ledge {
            x: int(-40),
            y: Fx::ZERO,
            side: -1,
        };
        2
    ];
    s.blast_left = int(-80);
    s.blast_right = int(80);
    s.blast_bottom = int(-30);
    for f in sim.state.fighters.iter_mut().skip(2) {
        f.active = false;
    }
    // (Player 1 waits far off on the roof: a match with one fighter left is over.)
    let o = &mut sim.state.fighters[1];
    o.pos = Vec2::new(int(35), int(5));
    o.vel = Vec2::ZERO;
    o.platform = TOP;
    o.state = S::Idle;
    sim
}

/// Stands player 0 on platform `on` at `x` (its feet on the top there).
fn stand_on(sim: &mut Sim, on: i8, x: Fx, facing: i8) {
    let top = sim.content.stage.platforms[on as usize].top_at(x);
    let f = &mut sim.state.fighters[0];
    f.pos = Vec2::new(x, top);
    f.vel = Vec2::ZERO;
    f.platform = on;
    f.state = S::Idle;
    f.state_frame = 0;
    f.facing = facing;
}

/// The feet are on the platform the fighter stands on.
fn on_surface(sim: &Sim) -> bool {
    let f = sim.f();
    usize::try_from(f.platform).is_ok_and(|i| {
        let p = sim.content.stage.platforms[i];
        f.pos.x >= p.left && f.pos.x <= p.right && p.top_at(f.pos.x) == f.pos.y
    })
}

#[test]
fn top_at_reads_a_slope_and_a_flat_platform() {
    let sim = slope_sim([0; 4]);
    let ramp = sim.content.stage.platforms[RAMP as usize];
    assert_eq!(ramp.top_at(int(-5)), Fx::ZERO);
    assert_eq!(ramp.top_at(Fx::ZERO), fx(5, 2));
    assert_eq!(ramp.top_at(int(5)), int(5));
    assert_eq!(ramp.top_at(int(9)), int(5), "held to its span");
    let floor = sim.content.stage.platforms[FLOOR as usize];
    assert_eq!(floor.top_at(int(-20)), floor.y);
}

/// Running up the ramp and onto the roof, then back down to the ground: on the ground every frame (the joints carry the fighter
/// across), the feet on the surface, and the ground speed horizontal (each frame moves by the speed, as on flat ground).
#[test]
fn running_up_and_down_a_ramp_keeps_the_feet_on_it_and_the_ground_speed() {
    let mut sim = slope_sim([0; 4]);
    stand_on(&mut sim, FLOOR, int(-15), 1);
    let mut on_ramp = 0;
    let mut visited = [false; 3];
    for t in 0..200 {
        let x0 = sim.f().pos.x;
        sim.tick(inp(127, 0, 0));
        let f = *sim.f();
        assert!(f.grounded(), "frame {t}: left the ground at {:?}", f.pos);
        assert!(
            on_surface(&sim),
            "frame {t}: feet off the surface at {:?}",
            f.pos
        );
        visited[f.platform as usize] = true;
        if f.platform == RAMP && f.state == S::Run {
            on_ramp += 1;
            assert_eq!(
                f.pos.x - x0,
                f.vel.x,
                "frame {t}: the ground speed is horizontal"
            );
        }
        if f.pos.x > int(15) {
            break;
        }
    }
    assert!(
        visited.iter().all(|&v| v),
        "floor, ramp and roof: {visited:?}"
    );
    assert!(on_ramp > 5, "it ran along the ramp ({on_ramp} frames)");
    assert_eq!(sim.f().platform, TOP);
    assert_eq!(sim.f().pos.y, int(5));
    // The same run speed on the ramp as on flat ground.
    let run = sim.content.fighters[0].run_speed;
    // And back down.
    for t in 0..300 {
        sim.tick(inp(-127, 0, 0));
        let f = *sim.f();
        assert!(
            f.grounded(),
            "down, frame {t}: left the ground at {:?}",
            f.pos
        );
        assert!(on_surface(&sim), "down, frame {t}: feet off the surface");
        if f.platform == RAMP && f.state == S::Run {
            assert!(
                f.vel.x.abs() <= run + fx(1, 100),
                "no faster down the slope"
            );
        }
        if f.pos.x < int(-15) {
            break;
        }
    }
    assert_eq!(sim.f().platform, FLOOR);
}

/// Walking (a gentle tilt) and dashing on the ramp: the feet follow the surface.
#[test]
fn walking_and_dashing_on_a_ramp_follow_the_surface() {
    for (tilt, frames) in [(40i8, 40), (127, 8), (-40, 40), (-127, 8)] {
        let mut sim = slope_sim([0; 4]);
        stand_on(&mut sim, RAMP, Fx::ZERO, if tilt > 0 { 1 } else { -1 });
        for _ in 0..frames {
            sim.tick(inp(tilt, 0, 0));
            assert_eq!(sim.f().platform, RAMP);
            assert!(on_surface(&sim), "tilt {tilt}");
        }
        let x = sim.f().pos.x;
        assert!(
            if tilt > 0 { x > Fx::ZERO } else { x < Fx::ZERO },
            "tilt {tilt}: moved"
        );
        assert!(matches!(sim.f().state, S::Walk | S::Dash | S::Run));
    }
}

/// Falling onto the ramp lands on it, at its surface; so does landing on the pass-through flight (and dropping through it).
#[test]
fn landing_on_slopes() {
    let mut sim = slope_sim([0; 4]);
    sim.put_airborne(0, Fx::ZERO, int(8), Fx::ZERO, Fx::ZERO);
    for _ in 0..90 {
        sim.tick(inp(0, 0, 0));
        if sim.f().grounded() {
            break;
        }
    }
    assert_eq!(sim.f().platform, RAMP);
    assert!(on_surface(&sim));
    assert_eq!(sim.f().pos.y, fx(5, 2));
    // Drifting sideways into the ramp's rising top meets it as ground (not a wall).
    let mut sim = slope_sim([0; 4]);
    sim.put_airborne(0, int(-6), fx(1, 2), fx(3, 10), Fx::ZERO);
    for _ in 0..30 {
        sim.tick(inp(127, 0, 0));
        if sim.f().grounded() {
            break;
        }
    }
    assert!(
        sim.f().grounded() && on_surface(&sim),
        "landed on the ramp or the floor"
    );
    // The flight.
    let mut sim = slope_sim([0; 4]);
    sim.put_airborne(0, int(-25), int(12), Fx::ZERO, Fx::ZERO);
    for _ in 0..90 {
        sim.tick(inp(0, 0, 0));
        if sim.f().grounded() {
            break;
        }
    }
    assert_eq!(sim.f().platform, FLIGHT);
    assert_eq!(sim.f().pos.y, fx(15, 2));
    // Down through it to the floor (the stick's down flick drops, with that rule on).
    sim.content.rules.stick_platform_drop = 1;
    sim.ticks(10, inp(0, 0, 0));
    for _ in 0..60 {
        sim.tick(inp(0, -127, 0));
        if sim.f().platform == FLOOR {
            break;
        }
    }
    assert_eq!(sim.f().platform, FLOOR, "dropped through the flight");
}

/// Jumping off the ramp leaves it and lands back on a surface.
#[test]
fn jumping_off_a_ramp() {
    let mut sim = slope_sim([0; 4]);
    stand_on(&mut sim, RAMP, Fx::ZERO, 1);
    sim.tick(inp(0, 0, JUMP));
    let mut airborne = false;
    for _ in 0..120 {
        sim.tick(inp(0, 0, 0));
        airborne |= !sim.f().grounded();
        if airborne && sim.f().grounded() {
            break;
        }
    }
    assert!(airborne, "it jumped");
    assert!(
        sim.f().grounded() && on_surface(&sim),
        "and landed on the slope"
    );
    assert!(sim.f().pos.y > Fx::ZERO);
}

/// A wavedash on the ramp (a jump, then an air dodge down and forward into it) lands and slides along the surface.
#[test]
fn a_wavedash_on_a_ramp_slides_along_it() {
    for dir in [1i8, -1] {
        let mut sim = slope_sim([0; 4]);
        stand_on(&mut sim, RAMP, Fx::ZERO, dir);
        sim.tick(inp(0, 0, JUMP));
        let mut slid = false;
        let x0 = sim.f().pos.x;
        for t in 0..30 {
            let i = if t < 3 {
                inp(dir * 90, -90, SHIELD)
            } else {
                inp(0, 0, 0)
            };
            sim.tick(i);
            if sim.f().grounded() {
                assert!(on_surface(&sim), "dir {dir} frame {t}: off the surface");
                slid |= (sim.f().pos.x - x0).abs() > fx(1, 2);
            }
        }
        assert!(
            slid,
            "dir {dir}: it slid along the ramp ({:?})",
            sim.f().state
        );
    }
}

/// A tumbling fighter teching onto the ramp techs as on flat ground, at the ramp's surface; one that does not lies on it.
#[test]
fn a_tech_on_a_ramp() {
    for (press, want) in [(SHIELD, S::Landing), (0, S::Knockdown)] {
        let mut sim = slope_sim([0; 4]);
        let f = &mut sim.state.fighters[0];
        f.state = S::Hitstun;
        f.hitstun = 80;
        f.tumble = true;
        f.pos = Vec2::new(Fx::ZERO, int(3));
        f.vel = Vec2::new(Fx::ZERO, fx(-2, 10));
        f.platform = NONE;
        let mut landed = None;
        for _ in 0..20 {
            sim.tick(inp(0, 0, press));
            if sim.f().state != S::Hitstun {
                landed = Some(sim.f().state);
                break;
            }
        }
        assert_eq!(landed, Some(want));
        assert_eq!(sim.f().platform, RAMP);
        assert!(on_surface(&sim));
    }
}

/// The hit's angle against the slope: a low launch (here the Gauntlets' 25-degree down tilt, given a sure trip) into a rising
/// slope runs along the ground, so it trips; the same hit on flat ground lifts the target (no trip), and the Longsword's 30-degree
/// down tilt lifts even on the steepest slope.
#[test]
fn trips_on_a_slope() {
    const SPRINTER: u8 = 4;
    const DUELIST: u8 = 0;
    // (Attacker, victim on the ramp or the floor) -> the victim's state and whether it tripped.
    let hit = |attacker: u8, slope: bool| {
        let mut chosen = None;
        for d in [8, 11, 14, 17, 20] {
            let mut sim = slope_sim([attacker, 0, 0, 0]);
            let w = usize::from(sim.content.fighters[usize::from(attacker)].weapon);
            for h in &mut sim.content.weapons[w].moves[MoveId::DTilt as usize].hitboxes {
                h.trip = 100;
            }
            let (on, x0) = if slope {
                (RAMP, int(-4))
            } else {
                (FLOOR, int(-20))
            };
            stand_on(&mut sim, on, x0, 1);
            let vx = x0 + fx(d, 10);
            let top = sim.content.stage.platforms[on as usize].top_at(vx);
            let v = &mut sim.state.fighters[1];
            v.pos = Vec2::new(vx, top);
            v.vel = Vec2::ZERO;
            v.platform = on;
            v.state = S::Idle;
            v.facing = -1;
            v.percent = Fx::ZERO;
            sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DTilt);
            for _ in 0..30 {
                sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
                if sim.fighter(1).percent > Fx::ZERO {
                    break;
                }
            }
            if sim.fighter(1).percent > Fx::ZERO {
                let v = sim.fighter(1);
                chosen = Some((v.state, v.tripped));
                break;
            }
        }
        chosen.expect("the down tilt hit")
    };
    assert_eq!(
        hit(SPRINTER, true),
        (S::Knockdown, true),
        "a 25-degree hit into the slope trips"
    );
    assert!(!hit(SPRINTER, false).1, "on flat ground it lifts: no trip");
    assert!(
        !hit(DUELIST, true).1,
        "30 degrees lifts off a 26.6-degree slope"
    );
}

/// A ramp meeting the ground at the same height is a joint, not a wall: walking from the roof down onto the ramp and the floor,
/// and a body moving sideways along the joint, never stops there.
#[test]
fn a_ramp_joint_is_not_a_wall() {
    let sim = slope_sim([0; 4]);
    let stage = &sim.content.stage;
    let p = sim.content.fighters[0];
    // Just below the roof's top on the ramp, moving right across the roof's left face.
    let mut pos = Vec2::new(
        int(5) - fx(1, 10),
        stage.platforms[RAMP as usize].top_at(int(5) - fx(1, 10)),
    );
    let wall = sim_core::collision::move_x(stage, &p, &mut pos, fx(3, 10));
    assert!(!wall && pos.x > int(5), "through the joint: {pos:?}");
    assert!(sim_core::collision::face_is_joint(
        stage,
        &stage.platforms[TOP as usize],
        -1
    ));
    assert!(!sim_core::collision::face_is_joint(
        stage,
        &stage.platforms[TOP as usize],
        1
    ));
    assert_eq!(
        sim_core::collision::continuation(stage, RAMP, int(5) + fx(1, 10)),
        Some(TOP as usize)
    );
    assert_eq!(
        sim_core::collision::continuation(stage, FLOOR, int(-5) + fx(1, 10)),
        Some(RAMP as usize)
    );
}

/// Flat platforms meeting at one height do not join (versus behaves as it always has): only a slope makes a joint.
#[test]
fn flat_platforms_never_join() {
    let mut sim = slope_sim([0; 4]);
    sim.content.stage.platforms[RAMP as usize].rise = Fx::ZERO;
    let stage = &sim.content.stage;
    assert_eq!(
        sim_core::collision::continuation(stage, FLOOR, int(-5) + fx(1, 10)),
        None
    );
    assert!(!sim_core::collision::face_is_joint(
        stage,
        &stage.platforms[RAMP as usize],
        -1
    ));
}
