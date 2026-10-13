//! Rolls stay on the platform they start on (sim v41), and the special fall after an up special only drifts a little.

mod common;

use common::{inp, Sim};
use sim_core::state::FighterState as S;
use sim_core::Fx;

#[test]
fn a_roll_into_the_edge_of_the_stage_ends_on_the_edge() {
    let mut sim = Sim::new();
    sim.stand(1, Fx::ZERO, 1);
    let right = sim.content.stage.platforms[0].right;
    sim.stand(0, right - Fx::HALF, -1);
    // A roll toward the edge (behind the fighter).
    let f = &mut sim.state.fighters[0];
    f.state = S::Roll;
    f.state_frame = 0;
    f.dodge_dir = sim_core::vec2::Vec2::new(Fx::ONE, Fx::ZERO);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().grounded(), "never leaves the stage");
        assert!(sim.f().pos.x <= right);
    }
    assert_eq!(sim.f().state, S::Idle, "the roll ends standing on the edge");
}

#[test]
fn a_roll_on_a_floating_platform_stops_at_its_end_too() {
    let mut sim = Sim::new();
    sim.stand(1, Fx::ZERO, 1);
    let Some((i, plat)) = sim
        .content
        .stage
        .platforms
        .iter()
        .enumerate()
        .find(|(_, p)| p.pass_through)
    else {
        return;
    };
    let (left, y) = (plat.left, plat.y);
    let f = &mut sim.state.fighters[0];
    f.pos.x = left + Fx::HALF;
    f.pos.y = y;
    f.platform = i as i8;
    f.vel = sim_core::vec2::Vec2::ZERO;
    f.facing = 1;
    f.state = S::Roll;
    f.state_frame = 0;
    f.dodge_dir = sim_core::vec2::Vec2::new(-Fx::ONE, Fx::ZERO);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().grounded(), "never leaves the platform");
        assert!(sim.f().pos.x >= left);
    }
}

#[test]
fn the_special_fall_only_drifts_a_little() {
    for ch in [0u8, 1] {
        let mut sim = Sim::with_chars([ch, 0, 0, 0]);
        sim.stand(1, Fx::from_int(9), -1);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].state = S::Helpless;
        for _ in 0..60 {
            sim.tick(inp(127, 0, 0));
        }
        assert_eq!(sim.f().state, S::Helpless);
        let p = sim.content.fighters[usize::from(ch)];
        let cap = p.air_speed * sim.content.rules.helpless_drift;
        assert!(sim.f().vel.x > Fx::ZERO, "it does drift");
        assert!(
            sim.f().vel.x <= cap + Fx::from_raw(64),
            "class {ch}: {:?} against {:?}",
            sim.f().vel.x,
            cap
        );
    }
}

/// Player 0, facing right in the air, uses its up special with `input` (then `after` for a few frames). Returns its facing on frame 5.
fn up_special_facing(ch: u8, input: sim_core::Input, after: sim_core::Input) -> i8 {
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(input);
    for _ in 0..4 {
        sim.tick(after);
    }
    assert_eq!(sim.f().state, S::Attack);
    sim.f().facing
}

#[test]
fn up_and_back_reverses_an_up_special() {
    use sim_core::input::buttons::SPECIAL;
    for ch in [0u8, 1] {
        assert_eq!(
            up_special_facing(ch, inp(0, 127, SPECIAL), inp(0, 127, 0)),
            1,
            "class {ch}: straight up keeps facing"
        );
        assert_eq!(
            up_special_facing(ch, inp(-90, 90, SPECIAL), inp(-90, 90, 0)),
            -1,
            "class {ch}: up and back turns round"
        );
        // Up first, then back within the first frames, turns it too.
        assert_eq!(
            up_special_facing(ch, inp(0, 127, SPECIAL), inp(-100, 60, 0)),
            -1,
            "class {ch}: back just after"
        );
    }
}

/// A neutral or side special started facing right, in the air moving right at `vx`; then `after` held for `frames` frames. Returns the
/// facing and the horizontal speed.
fn b_reverse(ch: u8, input: sim_core::Input, after: sim_core::Input, frames: usize) -> (i8, Fx) {
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(
        0,
        Fx::ZERO,
        Fx::from_int(30),
        Fx::from_ratio(1, 2),
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = 1;
    sim.tick(input);
    for _ in 0..frames {
        sim.tick(after);
    }
    assert_eq!(sim.f().state, S::Attack);
    (sim.f().facing, sim.f().vel.x)
}

#[test]
fn a_neutral_or_side_special_b_reverses_with_the_stick_the_other_way() {
    // The reference game's B-reverse: the other direction within 4 frames of the special turns the fighter round and, in the air,
    // sends its momentum the other way.
    use sim_core::input::buttons::SPECIAL;
    for ch in [0u8, 1] {
        let (facing, vx) = b_reverse(ch, inp(0, 0, SPECIAL), inp(0, 0, 0), 3);
        assert!(
            facing == 1 && vx > Fx::ZERO,
            "class {ch}: a plain neutral special keeps its facing and momentum"
        );
        let (facing, vx) = b_reverse(ch, inp(0, 0, SPECIAL), inp(-127, 0, 0), 3);
        assert!(
            facing == -1 && vx < Fx::ZERO,
            "class {ch}: back within the window turns it and reverses the momentum"
        );
        // Late: after the window the stick no longer turns it.
        let mut sim = Sim::with_chars([ch, 0, 0, 0]);
        sim.stand(1, Fx::from_int(9), -1);
        sim.put_airborne(
            0,
            Fx::ZERO,
            Fx::from_int(30),
            Fx::from_ratio(1, 2),
            Fx::ZERO,
        );
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(0, 0, SPECIAL));
        let window = usize::from(sim.content.rules.special_reverse_window);
        for _ in 0..window + 1 {
            sim.tick(inp(0, 0, 0));
        }
        sim.tick(inp(-127, 0, 0));
        assert!(
            sim.f().state != S::Attack || sim.f().facing == 1,
            "class {ch}: too late to reverse"
        );
        // A side special (it faces the stick) reverses the same way, once only: wiggling back again does not turn it back.
        let mut sim = Sim::with_chars([ch, 0, 0, 0]);
        sim.stand(1, Fx::from_int(9), -1);
        sim.put_airborne(
            0,
            Fx::ZERO,
            Fx::from_int(30),
            Fx::from_ratio(1, 2),
            Fx::ZERO,
        );
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(127, 0, SPECIAL));
        if sim.f().state == S::Attack {
            sim.tick(inp(-127, 0, 0));
            assert_eq!(sim.f().facing, -1, "class {ch}: a side special reverses");
            sim.tick(inp(127, 0, 0));
            assert_eq!(sim.f().facing, -1, "class {ch}: only once");
        }
    }
}

// ---- Per-move recovery rules (sim 58) -------------------------------------------------------------------------------------------

/// The claws' blaster (a neutral special with no motion) with `edit` applied, used by player 0 high in the air moving at `vx`.
fn custom_air_special(edit: impl Fn(&mut sim_core::moves::Move), vx: Fx) -> Sim {
    use sim_core::input::buttons::SPECIAL;
    use sim_core::moves::MoveId;
    let mut sim = Sim::with_chars([1, 0, 0, 0]);
    let w = usize::from(sim.content.fighters[1].weapon);
    edit(&mut sim.content.weapons[w].moves[MoveId::NSpecial as usize]);
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), vx, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 0, SPECIAL));
    sim
}

#[test]
fn a_move_can_keep_only_part_of_the_momentum_it_starts_with() {
    let half = Fx::from_ratio(1, 2);
    let plain = custom_air_special(|_| {}, half);
    let slowed = custom_air_special(|m| m.start_speed = 40, half);
    assert_eq!(plain.f().vel.x, half);
    assert_eq!(slowed.f().vel.x, half * Fx::from_ratio(40, 100));
}

#[test]
fn a_move_can_have_its_own_gravity_and_fall_speed_while_it_runs() {
    let g = Fx::from_ratio(1, 200);
    let cap = Fx::from_ratio(1, 20);
    let mut sim = custom_air_special(
        |m| {
            m.air_gravity = Some(g);
            m.air_max_fall = Some(cap);
        },
        Fx::ZERO,
    );
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().vel.y, -g, "one frame of the move's own gravity");
    sim.ticks(20, inp(0, 0, 0));
    assert_eq!(sim.f().vel.y, -cap, "and its own top fall speed");
    // The body's own gravity is far stronger.
    let mut plain = custom_air_special(|_| {}, Fx::ZERO);
    plain.tick(inp(0, 0, 0));
    assert!(plain.f().vel.y < -g);
}

#[test]
fn a_move_with_no_recovery_rules_behaves_as_before() {
    // Every rule left at its default: the plain move, unchanged (the same velocity and position frame by frame).
    let defaults = sim_core::moves::Move::empty();
    assert_eq!(defaults.ledge_from(false), 255);
    assert_eq!(defaults.ledge_from(true), 255);
    assert!(!defaults.catches_ledges());
    let mut grabs = defaults.clone();
    grabs.grabs_ledge = true;
    assert_eq!(
        (grabs.ledge_from(false), grabs.ledge_from(true)),
        (0, 0),
        "older content's grabs_ledge is both sides from the first frame"
    );
    let mut a = custom_air_special(|_| {}, Fx::ONE);
    let mut b = custom_air_special(
        |m| {
            m.rise_ground = 100;
            m.rise_air = 100;
            m.air_drift = 100;
        },
        Fx::ONE,
    );
    for _ in 0..30 {
        a.tick(inp(60, 0, 0));
        b.tick(inp(60, 0, 0));
        assert_eq!(a.f().pos, b.f().pos);
        assert_eq!(a.f().vel, b.f().vel);
    }
}
