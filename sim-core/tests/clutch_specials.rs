//! The Clutch's specials (class 8, the bounder body), phase 2: the lobbed shell (up special) and the drop (down special), modelled
//! mechanically on a reference character's (13.0.1). Reference data and sources: the ninth moveset's `REFERENCE_DATA.md`
//! (`docs/movesets/`); what is [R], [D] and [E] is said there and in `moves_clutch.rs`.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{MoveId, Weapon, ANGLE_FORWARD};
use sim_core::state::FighterState as S;
use sim_core::trig::{cos, sin, Angle};
use sim_core::{Fx, Projectile, Vec2, MAX_FIGHTERS};

const BOUNDER: u8 = 8;

/// The bounder (player 0) and a duelist far away (player 1, intangible so nothing hits it), the others out of the match.
fn alone() -> Sim {
    let mut sim = Sim::with_chars([BOUNDER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[1].invuln = 255;
    sim.stand(0, Fx::ZERO, 1);
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(BOUNDER)].weapon)]
}

fn near(a: Fx, b: Fx, thousandths: i32) -> bool {
    (a - b).abs() <= fx(thousandths, 1000)
}

/// Player 0's live projectiles.
fn shells(sim: &Sim) -> Vec<Projectile> {
    sim.state
        .projectiles
        .iter()
        .filter(|p| p.active && p.origin == 0)
        .copied()
        .collect()
}

// ---- The lobbed shell (up special) ----------------------------------------------------------------------------------------------

/// Starts the up special from where player 0 is and runs it to the shell's frame (16): special held for `held` frames after the
/// press, the stick at `stick_x` on the throwing frame. Returns the shell as it leaves.
fn throw(sim: &mut Sim, held: u16, stick_x: i8) -> Projectile {
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(MoveId::from_index(sim.f().move_id), MoveId::UpSpecial);
    assert_eq!(sim.f().state, S::Attack);
    for frame in 1..=15u16 {
        let b = if frame <= held { SPECIAL } else { 0 };
        let x = if frame == 15 { stick_x } else { 0 };
        sim.tick(inp(x, 0, b));
        if frame < 15 {
            assert!(shells(sim).is_empty(), "no shell before frame 16");
        }
    }
    let out = shells(sim);
    assert_eq!(out.len(), 1, "one shell on frame 16");
    out[0]
}

fn launch(held: i32, degrees: i32) -> Vec2 {
    let speed = fx(1875, 10000) + fx(1025, 100000) * Fx::from_int(held);
    let a = Angle::from_degrees(degrees);
    Vec2::new(speed * cos(a), speed * sin(a) * fx(5, 4))
}

#[test]
fn the_shell_leaves_on_frame_16_and_the_move_ends_on_56() {
    let mut sim = alone();
    let shell = throw(&mut sim, 0, 0);
    // 5 ahead and 13 up of the reference's units [R].
    assert_eq!(shell.pos, Vec2::new(fx(5, 8), fx(13, 8)));
    assert_eq!(shell.life, 46);
    // FAF 56: the move's last frame is 55 (it ends on it), so the fighter acts on 56.
    let mut frame = 16u16;
    while sim.f().state == S::Attack {
        sim.tick(inp(0, 0, 0));
        frame += 1;
    }
    assert_eq!(frame + 1, 56);
    let mv = kit(&sim).get(MoveId::UpSpecial as u8);
    assert!(!mv.helpless_after, "never helpless");
    assert_eq!(mv.air_gravity, Some(fx(7, 800)));
    assert_eq!(mv.air_max_fall, Some(fx(129, 800)));
    assert_eq!((mv.air_drift, mv.start_speed_y), (90, 80));
}

#[test]
fn holding_special_throws_the_shell_faster_up_to_15_frames() {
    // 1.5 + 0.082 f reference units a frame (f the frames held, at most 15), at 58 degrees with the stick neutral, its vertical part
    // x1.25 [R].
    for (held, f) in [(0u16, 0i32), (7, 7), (15, 15), (40, 15)] {
        let mut sim = alone();
        let shell = throw(&mut sim, held, 0);
        let want = launch(f, 58);
        assert!(
            near(shell.vel.x, want.x, 1) && near(shell.vel.y, want.y, 1),
            "held {held}: {:?} against {want:?}",
            shell.vel
        );
    }
}

#[test]
fn the_stick_aims_the_shell_between_18_and_90_degrees() {
    for (stick, degrees) in [(127i8, 18), (0, 58), (-127, 90)] {
        let mut sim = alone();
        let shell = throw(&mut sim, 0, stick);
        let want = launch(0, degrees);
        assert!(
            near(shell.vel.x, want.x, 2) && near(shell.vel.y, want.y, 2),
            "stick {stick}: {:?} against {want:?}",
            shell.vel
        );
        // Never behind.
        assert!(shell.vel.x >= Fx::ZERO);
    }
}

#[test]
fn in_the_air_the_first_three_throws_each_hop_and_the_fourth_does_not() {
    // From a fall at the body's top speed (1.29): min(2.5 x 0.8^n + 0.8 x -1.29, 2.6) = 1.468, 0.968, 0.568, then no hop [R]
    // (0.1835, 0.121, 0.071 world [D]); the move's gravity (0.00875) acts on the hop's own frame.
    let mut sim = alone();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, fx(-129, 800));
    let gravity = fx(7, 800);
    for want in [
        Some(fx(1468, 8000)),
        Some(fx(968, 8000)),
        Some(fx(568, 8000)),
        None,
    ] {
        sim.state.fighters[0].vel.y = fx(-129, 800);
        sim.tick(inp(0, 127, SPECIAL));
        sim.tick(inp(0, 0, 0));
        let vy = sim.f().vel.y;
        let expect = want.unwrap_or(fx(-129, 800) * fx(4, 5)) - gravity;
        assert!(near(vy, expect, 1), "{vy:?} against {expect:?}");
        while sim.f().state == S::Attack {
            sim.tick(inp(0, 0, 0));
        }
        assert_eq!(sim.f().state, S::Airborne, "never helpless");
    }
    // Landing gives the hops back.
    sim.put_airborne(0, Fx::ZERO, fx(1, 10), Fx::ZERO, fx(-1, 10));
    sim.ticks(3, inp(0, 0, 0));
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_flags, 0);
}

#[test]
fn on_the_ground_the_throw_does_not_hop_or_spend_a_hop() {
    let mut sim = alone();
    throw(&mut sim, 0, 0);
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_flags, 0);
}

/// A shell of player 0's up special put straight into the pool at `pos` with `vel`.
fn put_shell(sim: &mut Sim, pos: Vec2, vel: Vec2) {
    sim.state.projectiles[0] = Projectile {
        active: true,
        owner: 0,
        origin: 0,
        power: 100,
        move_id: MoveId::UpSpecial as u8,
        pos,
        vel,
        age: 0,
        life: 46,
        facing: 1,
        size: 100,
        hit_age: [0; MAX_FIGHTERS],
        ..Projectile::default()
    };
}

#[test]
fn the_shell_hits_one_fighter_once_and_its_burst_hits_the_one_beside_it() {
    for keep in [true, false] {
        let mut sim = Sim::with_chars([BOUNDER, 0, 0, 0]);
        sim.state.fighters[3].active = false;
        sim.stand(0, Fx::from_int(-8), 1);
        sim.stand(1, Fx::from_int(3), -1);
        sim.stand(2, fx(35, 10), -1);
        let w = usize::from(sim.content.fighters[usize::from(BOUNDER)].weapon);
        sim.content.weapons[w].moves[MoveId::UpSpecial as usize]
            .projectile
            .as_mut()
            .and_then(|p| p.burst.as_mut())
            .expect("the burst")
            .keep_hits = keep;
        put_shell(
            &mut sim,
            Vec2::new(fx(23, 10), Fx::ONE),
            Vec2::new(fx(3, 10), Fx::ZERO),
        );
        sim.tick(inp(0, 0, 0));
        assert!(
            sim.state.projectiles[0].bursting,
            "it bursts on the fighter"
        );
        assert_eq!(sim.state.projectiles[0].vel, Vec2::ZERO);
        sim.ticks(10, inp(0, 0, 0));
        assert!(!sim.state.projectiles[0].active, "the burst is over");
        // 6% each (x1.2 one on one); the shell's own victim is not hit again by the burst (the reference's single hit).
        let six = Fx::from_int(6) * sim.content.rules.damage_mult;
        let first = if keep { six } else { six + six };
        assert_eq!(sim.fighter(1).percent, first, "keep_hits {keep}");
        assert_eq!(sim.fighter(2).percent, six);
    }
}

#[test]
fn the_shell_bounces_off_a_slow_landing_and_bursts_on_a_fast_one_a_third_one_or_the_end_of_its_life(
) {
    let fresh = || {
        let mut sim = alone();
        sim.stand(0, Fx::from_int(-8), 1);
        sim
    };
    // A slow landing (falling at most 0.2 a frame: the reference's 1.6) bounces, keeping 0.8 of its speed both ways.
    let mut sim = fresh();
    put_shell(
        &mut sim,
        Vec2::new(Fx::ZERO, fx(5, 100)),
        Vec2::new(fx(1, 10), fx(-1, 10)),
    );
    sim.tick(inp(0, 0, 0));
    let p = sim.state.projectiles[0];
    assert!(p.active && !p.bursting && p.bounces == 1);
    let fall = fx(1, 10) + fx(9, 800);
    assert_eq!(p.vel.y, fall * fx(4, 5));
    assert_eq!(p.vel.x, fx(1, 10) * fx(4, 5));
    assert_eq!(p.pos.y, Fx::ZERO);
    // A fast one bursts where it touched.
    let mut sim = fresh();
    put_shell(
        &mut sim,
        Vec2::new(Fx::ZERO, fx(2, 10)),
        Vec2::new(fx(1, 10), fx(-5, 10)),
    );
    sim.tick(inp(0, 0, 0));
    let p = sim.state.projectiles[0];
    assert!(p.active && p.bursting && p.pos.y == Fx::ZERO, "{p:?}");
    // After two bounces, the third landing bursts it.
    let mut sim = fresh();
    put_shell(
        &mut sim,
        Vec2::new(Fx::ZERO, fx(5, 100)),
        Vec2::new(Fx::ZERO, fx(-1, 10)),
    );
    sim.state.projectiles[0].bounces = 2;
    sim.tick(inp(0, 0, 0));
    assert!(sim.state.projectiles[0].bursting);
    // Still in the air at the end of its 46 frames, it bursts there, for 3 frames.
    let mut sim = fresh();
    put_shell(&mut sim, Vec2::new(Fx::ZERO, Fx::from_int(20)), Vec2::ZERO);
    sim.state.projectiles[0].age = 45;
    sim.tick(inp(0, 0, 0));
    assert!(sim.state.projectiles[0].bursting);
    sim.ticks(3, inp(0, 0, 0));
    assert!(sim.state.projectiles[0].active, "burst frames 1-3");
    sim.tick(inp(0, 0, 0));
    assert!(!sim.state.projectiles[0].active);
}

#[test]
fn the_shell_and_its_burst_have_the_references_hit() {
    let sim = alone();
    let spec = kit(&sim)
        .get(MoveId::UpSpecial as u8)
        .projectile
        .expect("the shell");
    let burst = spec.burst.expect("the burst");
    let (_, burst_hit) = burst.phases[0].expect("one phase");
    for (h, radius) in [(spec.hitbox, fx(3, 10)), (burst_hit, fx(8, 10))] {
        assert_eq!(h.damage, Fx::from_int(6));
        assert_eq!(
            (h.angle, h.base_knockback, h.knockback_growth),
            (70, 60, 50)
        );
        assert_eq!(h.angle_type, ANGLE_FORWARD);
        assert_eq!((h.hitlag, h.shield_damage), (60, 50));
        assert!(!h.rebounds && h.clanks);
        assert_eq!(h.radius, radius);
        assert_eq!(h.group, 0);
    }
    assert_eq!(
        (
            burst.life,
            burst.on_ground,
            burst.on_expire,
            burst.keep_hits,
            burst.on_wall
        ),
        (4, true, true, true, true)
    );
    let a = spec.arc;
    assert_eq!((a.bounces, a.bounce_keep, a.bounce_keep_y), (2, 80, 80));
    assert_eq!(a.bounce_max_fall, fx(16, 80));
    assert_eq!((a.gravity, a.max_fall), (fx(9, 800), fx(10, 8)));
}

// ---- The drop (down special) ----------------------------------------------------------------------------------------------------

use sim_core::moves::clutch_slots::{DROP_AIR, DROP_LANDING};
use sim_core::moves::TARGETS_GROUNDED;

/// Runs the down special from the ground at `x` (`air` None) or from that height in the air, the stick down held if `down`;
/// returns, for each tick from 1 (the move's frame 1), (move, move frame, state, height).
fn drop_run(air: Option<Fx>, x: Fx, down: bool, ticks: u32) -> (Sim, Vec<(MoveId, u16, S, Fx)>) {
    let mut sim = alone();
    match air {
        Some(h) => sim.put_airborne(0, x, h, Fx::ZERO, Fx::ZERO),
        None => sim.stand(0, x, 1),
    }
    let mut out = Vec::new();
    for t in 1..=ticks {
        let b = if t == 1 { SPECIAL } else { 0 };
        let y = if t == 1 || down { -127 } else { 0 };
        sim.tick(inp(0, y, b));
        let f = sim.f();
        out.push((
            MoveId::from_index(f.move_id),
            f.state_frame,
            f.state,
            f.pos.y,
        ));
    }
    (sim, out)
}

#[test]
fn the_grounded_drop_hops_drops_from_27_lands_on_39_and_ends_on_77() {
    let (_, run) = drop_run(None, Fx::ZERO, false, 90);
    assert_eq!((run[0].0, run[0].1), (MoveId::DownSpecial, 0));
    // On the ground through frame 8, rising from 9 (the hop [E]); the drop from 27 at 0.6 a frame [R].
    assert_eq!(run[7].3, Fx::ZERO);
    assert!(run[8].3 > Fx::ZERO);
    let fall = run[26].3 - run[27].3;
    assert!(near(fall, fx(6, 10), 1), "{fall:?}");
    // The landing (39 frames, FAF 40) starts on 39; the fighter acts again on 78 (the reference's 77 frames in all).
    let landed = run
        .iter()
        .position(|r| r.0 == DROP_LANDING && r.1 == 0 && r.2 == S::Attack)
        .expect("landed")
        + 1;
    assert_eq!(landed, 39);
    let free = run.iter().position(|r| r.2 != S::Attack).expect("over") + 2;
    assert_eq!(free, 78);
}

#[test]
fn the_aerial_drop_stalls_then_drops_from_19() {
    let (_, run) = drop_run(Some(Fx::from_int(30)), Fx::ZERO, false, 40);
    assert_eq!(run[0].0, DROP_AIR);
    // A small rise and a still top [E], then 0.6 a frame from 19 [R].
    let top = run[17].3;
    assert!(top > Fx::from_int(30) && top < Fx::from_int(36), "{top:?}");
    assert_eq!(run[16].3, top);
    assert!(near(run[18].3, top - fx(6, 10), 1));
    // The grounded rise is the aerial one x1.4 (the reference's grounded height multiplier) [R].
    let (_, ground) = drop_run(None, Fx::ZERO, false, 30);
    let ratio = ground[24].3 * Fx::from_int(10) / (top - Fx::from_int(30));
    assert!(near(ratio, Fx::from_int(14), 30), "{ratio:?}");
}

#[test]
fn the_drops_hits_are_the_references() {
    let sim = alone();
    let w = kit(&sim);
    let g = w.get(MoveId::DownSpecial as u8);
    let a = w.get(DROP_AIR as u8);
    assert_eq!(g.air_version, Some(DROP_AIR as u8));
    let hop = g.hitboxes[0];
    assert_eq!((hop.start, hop.end), (6, 6));
    assert_eq!(hop.damage, Fx::from_int(4));
    assert_eq!(
        (hop.angle, hop.fixed_kb, hop.knockback_growth),
        (80, 80, 100)
    );
    assert_eq!(hop.targets, TARGETS_GROUNDED);
    assert_eq!(hop.hitlag, 50);
    let fall = g.hitboxes[1];
    assert_eq!((fall.start, fall.group), (26, 1));
    assert_eq!(fall.damage, Fx::from_int(15));
    assert_eq!(
        (fall.angle, fall.base_knockback, fall.knockback_growth),
        (80, 60, 72)
    );
    assert_eq!((fall.hitlag, fall.shield_damage), (70, 167));
    for (h, start) in [(a.hitboxes[0], 18), (a.hitboxes[1], 19)] {
        assert_eq!(h.start, start);
        assert_eq!(h.damage, Fx::from_int(12));
        assert_eq!(
            (h.angle, h.base_knockback, h.knockback_growth),
            (80, 70, 68)
        );
        assert_eq!((h.hitlag, h.shield_damage), (70, 183));
        assert_eq!(h.angle_type, ANGLE_FORWARD);
    }
    for (m, from) in [(g, 26u8), (a, 18)] {
        assert_eq!((m.ledge_front_from, m.ledge_back_from), (from, from));
        let d = m.drop_through.expect("drops through soft platforms");
        assert_eq!((d.from, d.frames, d.while_down), (from, 7, true));
        assert_eq!(m.landing_move, Some(DROP_LANDING as u8));
        assert!(!m.helpless_after);
    }
    let landing = w.get(DROP_LANDING as u8);
    assert_eq!(u16::from(landing.total_frames) + 2, 40);
}

#[test]
fn landing_throws_two_stars_on_the_same_frame_one_each_way() {
    let (sim, _) = drop_run(None, Fx::ZERO, false, 40);
    // Landed on 39 (landing frame 1); the stars leave on landing frame 2 (tick 40), so they hit on its 3-12.
    let mut stars = shells(&sim);
    assert_eq!(stars.len(), 2, "both stars on landing frame 2");
    stars.sort_by_key(|p| p.vel.x);
    let x = sim.f().pos.x;
    assert_eq!(stars[0].vel.y, fx(1, 4));
    assert!(near(stars[0].vel.x, fx(-75, 1000), 1) && near(stars[1].vel.x, fx(75, 1000), 1));
    assert_eq!(stars[1].vel.y, fx(1, 4));
    assert_eq!(stars[0].pos, Vec2::new(x - Fx::ONE, fx(1, 8)));
    assert_eq!(stars[1].pos, Vec2::new(x + Fx::ONE, fx(1, 8)));
    assert!(stars.iter().all(|p| p.life == 11));
    let spec = kit(&sim)
        .get(DROP_LANDING as u8)
        .projectile
        .expect("the star");
    let h = spec.hitbox;
    assert_eq!(h.damage, Fx::from_int(4));
    assert_eq!(
        (h.angle, h.base_knockback, h.knockback_growth),
        (50, 50, 40)
    );
    assert_eq!((h.hitlag, h.shield_damage, h.rebounds), (0, 250, false));
    assert_eq!(h.radius, fx(5, 10));
}

#[test]
fn the_drop_passes_soft_platforms_only_at_its_start_or_while_down_is_held() {
    // From the left soft platform (x -5.5, 3.6 up): the 7-frame drop ends far above it, so it lands back on it; holding down it
    // falls through to the stage.
    let soft = fx(36, 10);
    for (down, floor) in [(false, soft), (true, Fx::ZERO)] {
        let mut sim = alone();
        sim.stand(0, fx(-55, 10), 1);
        sim.state.fighters[0].pos.y = soft;
        sim.state.fighters[0].platform = 1;
        // (Started directly: a stick flicked down on a soft platform drops through it before a special can start.)
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DownSpecial);
        // (Down is let go during the hop, so it does not drop through before the move.)
        for _ in 0..8 {
            sim.tick(inp(0, 0, 0));
        }
        let mut landed = None;
        for _ in 0..60 {
            sim.tick(inp(0, if down { -127 } else { 0 }, 0));
            if sim.f().grounded() {
                landed = Some(sim.f().pos.y);
                break;
            }
        }
        assert_eq!(landed, Some(floor), "down held {down}");
    }
}

#[test]
fn the_drop_catches_a_ledge_behind_it_once_it_falls() {
    let mut sim = alone();
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .expect("a right ledge");
    // Off the right edge, facing away from the stage (right), the drop started 1.5 above the ledge.
    sim.put_airborne(
        0,
        ledge.x + fx(12, 10),
        ledge.y + fx(15, 10),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, -127, SPECIAL));
    let mut caught = None;
    for t in 2..60u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            caught = Some(t);
            break;
        }
    }
    let t = caught.expect("caught the ledge behind");
    assert!(t >= 19, "only from the drop's start: {t}");
}
