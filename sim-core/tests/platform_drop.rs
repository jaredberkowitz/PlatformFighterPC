//! Owner decision 63: a platform drop and a down special (the reference's rule). A down flick on a pass-through platform keeps the
//! fighter on it for `Ruleset::platform_drop_frames` (2) more frames before it falls, and a special on the flick's frame, on any of
//! those, or on the frame it falls comes out on the ground: with the stick down, the grounded down special. Attacks do not
//! interrupt the drop (they come out in the air after it). Solid ground is unchanged.
//!
//! Since sim 112 the down-flick drop is off by default (owner decision 2026-10-14, `Ruleset::stick_platform_drop`: the drop is
//! shield + special); these tests turn it on to keep that rule working.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{Fx, Vec2};

/// Classes (body and weapon index) checked: Longsword, Needles, Greatsword, Clutch, Buckler.
const CLASSES: [u8; 5] = [0, 2, 6, 8, 9];

fn sim_for(class: u8, platform: bool) -> Sim {
    let mut sim = Sim::with_chars([class, 1, 0, 1]);
    sim.content.rules.stick_platform_drop = 1;
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    if platform {
        let f = &mut sim.state.fighters[0];
        f.pos = Vec2::new(Fx::from_int(-5), Fx::from_ratio(18, 5));
        f.platform = 1;
    } else {
        sim.stand(0, Fx::ZERO, 1);
    }
    sim.tick(inp(0, 0, 0));
    sim
}

/// The window: special on the flick's frame (0) up to and including the frame the fighter would fall.
fn window(sim: &Sim) -> usize {
    usize::from(sim.content.rules.platform_drop_frames)
}

/// Flicks down (and holds it), presses special `delay` frames after the flick, and returns the sim on the special's frame.
fn down_then_special(class: u8, platform: bool, delay: usize) -> Sim {
    let mut sim = sim_for(class, platform);
    for _ in 0..delay {
        sim.tick(inp(0, -127, 0));
    }
    sim.tick(inp(0, -127, SPECIAL));
    sim
}

fn assert_grounded_down_special(sim: &Sim, what: &str) {
    let f = sim.f();
    assert_eq!(f.state, S::Attack, "{what}: state");
    assert_eq!(f.move_id, MoveId::DownSpecial as u8, "{what}: move");
    assert!(f.grounded(), "{what}: still on the ground");
}

#[test]
fn the_classes_checked_have_a_grounded_down_special() {
    let sim = Sim::new();
    for c in CLASSES {
        let w = &sim.content.weapons[usize::from(c)];
        assert!(!w.get(MoveId::DownSpecial as u8).is_empty(), "class {c}");
    }
}

#[test]
fn down_and_special_on_the_same_frame_is_the_grounded_down_special_on_a_platform() {
    for c in CLASSES {
        let sim = down_then_special(c, true, 0);
        assert_grounded_down_special(&sim, &format!("class {c}"));
        assert_eq!(sim.f().platform, 1, "class {c}: on the platform");
    }
}

#[test]
fn a_special_within_the_drop_window_is_still_the_grounded_down_special() {
    for c in CLASSES {
        let n = window(&sim_for(c, true));
        for delay in 0..=n {
            let sim = down_then_special(c, true, delay);
            assert_grounded_down_special(&sim, &format!("class {c}, {delay} frames after"));
            assert_eq!(sim.f().platform, 1, "class {c}, {delay}");
        }
    }
}

#[test]
fn a_special_just_after_the_window_comes_out_in_the_air() {
    for c in CLASSES {
        let n = window(&sim_for(c, true));
        let sim = down_then_special(c, true, n + 1);
        assert!(!sim.f().grounded(), "class {c}: dropped through first");
        assert_eq!(
            sim.f().state,
            S::Attack,
            "class {c}: the aerial down special"
        );
        // (A weapon with an aerial version uses its own slot; it is the down special either way.)
        assert_ne!(sim.f().platform, 1, "class {c}");
    }
}

#[test]
fn on_solid_ground_down_and_special_is_the_down_special_at_any_delay() {
    for c in CLASSES {
        for delay in 0..=4 {
            let sim = down_then_special(c, false, delay);
            assert_grounded_down_special(&sim, &format!("class {c}, solid, {delay} frames after"));
            assert_eq!(sim.f().platform, 0);
            assert_eq!(sim.f().pos.y, Fx::ZERO);
        }
    }
}

#[test]
fn a_plain_platform_drop_crouches_for_the_window_then_falls() {
    for c in CLASSES {
        let mut sim = sim_for(c, true);
        let n = window(&sim);
        for k in 0..n {
            sim.tick(inp(0, -127, 0));
            assert_eq!(sim.f().state, S::Crouch, "class {c}, frame {k}");
            assert_eq!(sim.f().platform, 1, "class {c}, frame {k}");
            assert_ne!(sim.f().drop_pending, 0, "class {c}, frame {k}: pending");
        }
        sim.tick(inp(0, -127, 0));
        assert_eq!(
            sim.f().state,
            S::Airborne,
            "class {c}: falls on frame {}",
            n + 1
        );
        assert!(!sim.f().grounded());
        assert_eq!(sim.f().drop_pending, 0);
        sim.ticks(80, inp(0, 0, 0));
        assert_eq!(sim.f().platform, 0, "class {c}: lands on the main stage");
    }
}

#[test]
fn letting_go_of_the_stick_after_the_flick_still_drops() {
    // The flick is the drop's input: it is committed, as before.
    let mut sim = sim_for(0, true);
    let n = window(&sim);
    sim.tick(inp(0, -127, 0));
    sim.ticks(n, inp(0, 0, 0));
    assert!(!sim.f().grounded());
}

#[test]
fn an_attack_during_the_window_does_not_stop_the_drop() {
    let mut sim = sim_for(0, true);
    let n = window(&sim);
    sim.tick(inp(0, -127, 0));
    sim.tick(inp(0, -127, ATTACK));
    for _ in 1..n {
        sim.tick(inp(0, -127, ATTACK));
    }
    assert!(!sim.f().grounded(), "the drop keeps its timing");
    // The buffered attack comes out in the air.
    sim.ticks(3, inp(0, -127, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    assert!(!sim.f().grounded());
    assert_ne!(sim.f().move_id, MoveId::DTilt as u8);
}

#[test]
fn a_zero_window_rule_drops_on_the_flick_but_a_same_frame_special_still_wins() {
    let mut sim = sim_for(0, true);
    sim.content.rules.platform_drop_frames = 0;
    let mut same = sim_for(0, true);
    same.content.rules.platform_drop_frames = 0;
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().state, S::Airborne, "the old timing");
    same.tick(inp(0, -127, SPECIAL));
    assert_grounded_down_special(&same, "same frame, zero window");
}

#[test]
fn a_resimulated_drop_matches() {
    // Snapshot mid-window, as a rollback would, and replay: the pending drop is part of the state.
    let mut a = sim_for(2, true);
    a.tick(inp(0, -127, 0));
    assert_ne!(a.f().drop_pending, 0);
    let snapshot = a.state;
    a.ticks(5, inp(0, -127, 0));
    let mut b = sim_for(2, true);
    b.state = snapshot;
    b.ticks(5, inp(0, -127, 0));
    assert_eq!(a.state.checksum(), b.state.checksum());
    // A snapshot that forgot the pending drop would not fall on the same frame.
    let mut c = sim_for(2, true);
    c.state = snapshot;
    c.state.fighters[0].drop_pending = 0;
    c.tick(inp(0, -127, 0));
    let mut d = sim_for(2, true);
    d.state = snapshot;
    d.tick(inp(0, -127, 0));
    assert_ne!(c.state.checksum(), d.state.checksum());
}

#[test]
fn a_carry_on_a_platform_ignores_down_and_special() {
    // The carry (the Bracers' cargo hold, sim 98) allows only throws and walking: down and special on a soft platform start no
    // special and do not drop it, at any delay.
    const HAULER: u8 = 7;
    for delay in 0..=3 {
        let mut sim = Sim::with_chars([HAULER, 0, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_ratio(15, 10), -1);
        sim.tick(inp(0, 0, GRAB));
        for _ in 0..40 {
            if sim.f().state == S::Grabbing {
                break;
            }
            sim.tick(inp(0, 0, 0));
        }
        assert_eq!(sim.f().state, S::Grabbing);
        // The forward throw is the lift into the carry.
        sim.tick(inp(127, 0, 0));
        for _ in 0..30 {
            if sim.f().state == S::Carrying {
                break;
            }
            sim.tick(inp(0, 0, 0));
        }
        assert_eq!(sim.f().state, S::Carrying);
        let lift = sim.f().move_id;
        let f = &mut sim.state.fighters[0];
        f.pos = Vec2::new(Fx::from_int(-5), Fx::from_ratio(18, 5));
        f.platform = 1;
        f.vel = Vec2::ZERO;
        sim.tick(inp(0, 0, 0));
        for _ in 0..delay {
            sim.tick(inp(0, -127, 0));
        }
        sim.tick(inp(0, -127, SPECIAL));
        for k in 0..6 {
            assert_eq!(sim.f().state, S::Carrying, "delay {delay}, frame {k}");
            assert_eq!(sim.f().move_id, lift, "delay {delay}: no special");
            assert_eq!(sim.f().platform, 1, "delay {delay}: no drop");
            assert_eq!(sim.f().drop_pending, 0);
            sim.tick(inp(0, -127, 0));
        }
    }
}
