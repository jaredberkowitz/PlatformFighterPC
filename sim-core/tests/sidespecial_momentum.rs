//! Side specials and the momentum they start with (sim 139, owner decision 98), as `upspecial_momentum.rs` does for the up specials:
//! started in the air, each keeps, scales, zeroes or adds to the speed it began with by the reference's published rule (ssbwiki.com
//! `<Fighter>_(SSBU)/Side_special` "Parameters", 13.0.1). The per-fighter table and sources: `docs/movesets/SHARED_MECHANICS.md`
//! ("Starting momentum of specials").

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

fn sim_for(c: u8) -> Sim {
    let mut sim = Sim::with_chars([c, 0, 0, 0]);
    sim.stand(1, Fx::from_int(60), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

/// Class `c` in the air moving at (`vx`, `vy`), facing right; it starts its side special and runs the special's first frame. Returns
/// the speed after that frame.
fn started(c: u8, vx: Fx, vy: Fx) -> (Sim, Fx, Fx) {
    let mut sim = sim_for(c);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), vx, vy);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(127, 0, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack, "class {c}");
    let (x, y) = (sim.f().vel.x, sim.f().vel.y);
    (sim, x, y)
}

/// The reference's startup rule per class (sideways percent kept; vertical: kept, scaled, zeroed or added to), checked one frame in,
/// within a frame of gravity and friction.
#[test]
fn every_side_special_starts_with_its_reference_momentum() {
    let su = |n: i32| Fx::from_ratio(n, 8000);
    // (class, sideways percent, the vertical speed after one frame from `vy`)
    type Rule = fn(Fx) -> Fx;
    let rules: [(u8, i32, Rule); 6] = [
        // The duelist's blade flurry: x0.5 sideways, +1.4 up (both [R]).
        (0, 50, |vy| vy + Fx::from_ratio(1400, 8000)),
        // The brawler's flash dash: x0.67 sideways [R], held still (the startup gravity begins on frame 14 [R]).
        (1, 67, |_| Fx::ZERO),
        // The tumbler's cape: the air speed / 2.5 [R]; the first use stops a fall and adds 0.3 [R].
        (3, 40, |_| Fx::from_ratio(300, 8000)),
        // The seer's flame: x0.8 sideways [R]; the fall goes on.
        (5, 80, |vy| vy),
        // The hauler's headbutt: x0.67 sideways and x0.5 vertical in the air [R].
        (7, 67, |vy| vy * Fx::HALF),
        // The settler's rocket stops the momentum [R, qualitative].
        (10, 0, |_| Fx::ZERO),
    ];
    for (c, keep, rule) in rules {
        let p = sim_for(c).content.fighters[usize::from(c)];
        let (vx, vy) = (p.air_speed * Fx::HALF, -su(1000));
        let (_, x, y) = started(c, vx, vy);
        let want_x = vx * Fx::from_ratio(keep, 100);
        let slack = p.gravity + p.air_friction + su(50) + Fx::from_raw(4);
        assert!(
            (x - want_x).abs() <= slack,
            "class {c} sideways: {x:?} vs {want_x:?}"
        );
        let want_y = rule(vy);
        assert!(
            (y - want_y).abs() <= slack,
            "class {c} vertical: {y:?} vs {want_y:?}"
        );
    }
}

/// The blade flurry's lift has no once-per-airtime limit published: every use slows a fall the same.
#[test]
fn the_blade_flurrys_lift_slows_every_fall() {
    for _ in 0..2 {
        let (_, _, y) = started(0, Fx::ZERO, -fx(1580, 8000));
        assert!(y > -fx(300, 8000) && y < Fx::ZERO, "{y:?}");
    }
}

/// The flash dash: held still through its startup, angled at most 10 degrees by the stick, then sliding on at the reference's 2.3 units
/// a frame in the air (2.5 on the ground), helpless in the air afterwards.
#[test]
fn the_flash_dash_holds_angles_and_slides_as_the_reference() {
    let run = |y: i8| {
        let mut sim = sim_for(1);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, fx(1, 5));
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(127, 0, SPECIAL));
        let y0 = sim.f().pos.y;
        for _ in 1..=12 {
            sim.tick(inp(0, y, 0));
            assert_eq!(sim.f().pos.y, y0, "held still in the startup");
        }
        let before = sim.f().pos;
        for _ in 13..=17 {
            sim.tick(inp(0, y, 0));
        }
        let start = sim.f().pos;
        for _ in 18..=27 {
            sim.tick(inp(0, y, 0));
        }
        let dash = sim.f().pos - start;
        sim.tick(inp(0, y, 0));
        let slide = sim.f().vel.x;
        for _ in 29..=70 {
            sim.tick(inp(0, 0, 0));
        }
        (before, dash, slide, sim.f().state)
    };
    let (_, flat, slide, state) = run(0);
    let (_, up, _, _) = run(127);
    assert_eq!(flat.y, Fx::ZERO);
    // tan(10 degrees) is 0.176: at full tilt the dash climbs at most that share of its run.
    assert!(up.y > Fx::ZERO && up.y <= up.x * fx(180, 1000), "{up:?}");
    // The slide's first frame: 2.3 less one frame of the 0.07 friction.
    assert!(
        (slide - fx(2300 - 70, 8000)).abs() <= Fx::from_raw(64),
        "{slide:?}"
    );
    assert_eq!(state, S::Helpless);
}

/// A dash into a side special on the ground: the moves that set their own ground speed (the dashes) ignore the dash's speed; the others
/// slide on with the ground friction, as the reference's moves without a published multiplier [E: unpublished].
#[test]
fn a_dash_into_the_flash_dash_does_not_carry_on() {
    let mut sim = sim_for(1);
    sim.stand(0, Fx::from_int(-8), 1);
    sim.state.fighters[0].vel.x = fx(2, 10);
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
    for _ in 0..16 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().vel.x.abs() < fx(1, 100), "{:?}", sim.f().vel.x);
}
