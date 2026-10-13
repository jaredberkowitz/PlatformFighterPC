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

// ---- Wall jumps (sim 64) --------------------------------------------------------------------------------------------------------

/// Class `ch` falling against the left side of the main stage (its wall at x = -11), then the stick flicked away from it (left).
/// Returns the fighter after the flick.
fn flick_off_the_wall(ch: u8, sim: &mut Sim) -> sim_core::Fighter {
    let half = sim.content.fighters[usize::from(ch)].ecb_half_width;
    if sim.f().char_id != ch || sim.f().grounded() || sim.f().pos.y > Fx::from_int(-1) {
        sim.state.fighters[0].char_id = ch;
    }
    let y = sim.f().pos.y.min(Fx::from_int(-2));
    sim.put_airborne(
        0,
        Fx::from_int(-11) - half,
        y,
        Fx::ZERO,
        -Fx::from_ratio(1, 10),
    );
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(-127, 0, 0));
    *sim.f()
}

#[test]
fn a_wall_jumper_kicks_off_a_wall_with_the_stick_flicked_away_and_each_one_is_weaker() {
    let units = |n: i32| Fx::from_ratio(n, 8000);
    // The tumbler (class 3): 1.3 away, 2.3 up (the reference's).
    let mut sim = Sim::with_chars([3, 0, 0, 0]);
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    let f = flick_off_the_wall(3, &mut sim);
    assert_eq!(f.wall_jumps, 1);
    assert_eq!(f.facing, -1, "facing away from the wall");
    // (One frame of air movement since: gravity and drift.)
    assert!((f.vel.y - units(2300)).abs() < units(150), "{:?}", f.vel);
    assert!((f.vel.x + units(1300)).abs() < units(150), "{:?}", f.vel);
    // Back against the wall without landing: the next one is 0.96 as strong.
    let second = flick_off_the_wall(3, &mut sim);
    assert_eq!(second.wall_jumps, 2);
    assert!(
        second.vel.y < f.vel.y,
        "weaker: {:?} vs {:?}",
        second.vel,
        f.vel
    );
    let ratio = (second.vel.y + sim.content.fighters[3].gravity)
        / (f.vel.y + sim.content.fighters[3].gravity);
    assert!(
        (ratio - Fx::from_ratio(96, 100)).abs() < Fx::from_ratio(1, 100),
        "{ratio:?}"
    );
}

#[test]
fn a_body_without_a_wall_jump_only_drifts_away_from_the_wall() {
    // The duelist (class 0) has none.
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    let f = flick_off_the_wall(0, &mut sim);
    assert_eq!(f.wall_jumps, 0);
    assert!(f.vel.y < Fx::ZERO, "still falling: {:?}", f.vel);
}

#[test]
fn landing_lets_the_wall_jumps_start_over() {
    let mut sim = Sim::with_chars([3, 0, 0, 0]);
    sim.stand(1, Fx::from_int(9), -1);
    sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    flick_off_the_wall(3, &mut sim);
    assert_eq!(sim.f().wall_jumps, 1);
    // Down onto the stage.
    sim.put_airborne(0, Fx::ZERO, Fx::from_ratio(1, 10), Fx::ZERO, -Fx::ONE);
    sim.ticks(2, inp(0, 0, 0));
    assert!(sim.f().grounded());
    assert_eq!(sim.f().wall_jumps, 0);
}

// ---- Wall jumps from the jump button (owner, 2026-10-13) ------------------------------------------------------------------------

/// Class `ch` falling against the left side of the main stage (its wall at x = -11), facing it; then jump pressed with the stick at
/// `stick_x`. Returns the fighter and its midair jumps before the press.
fn jump_at_the_wall(ch: u8, stick_x: i8) -> (sim_core::Fighter, u8) {
    use sim_core::input::buttons::JUMP;
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.stand(1, Fx::from_int(9), -1);
    let half = sim.content.fighters[usize::from(ch)].ecb_half_width;
    sim.put_airborne(
        0,
        Fx::from_int(-11) - half,
        Fx::from_int(-3),
        Fx::ZERO,
        -Fx::from_ratio(1, 10),
    );
    sim.state.fighters[0].facing = 1;
    // The stick has been held there all along, so it is no flick.
    sim.state.fighters[0].history = [inp(stick_x, 0, 0); sim_core::state::HISTORY_LEN];
    sim.ticks(2, inp(stick_x, 0, 0));
    assert_eq!(sim.f().wall_jumps, 0, "no wall jump before the press");
    let jumps = sim.f().air_jumps_left;
    assert!(jumps > 0);
    sim.tick(inp(stick_x, 0, JUMP));
    (*sim.f(), jumps)
}

#[test]
fn jump_against_a_wall_with_the_stick_neutral_is_a_wall_jump_and_keeps_the_midair_jump() {
    let units = |n: i32| Fx::from_ratio(n, 8000);
    let (f, jumps) = jump_at_the_wall(3, 0);
    assert_eq!(f.wall_jumps, 1, "a wall jump");
    assert_eq!(f.air_jumps_left, jumps, "no midair jump spent");
    assert_eq!(f.facing, -1, "facing away from the wall");
    assert!(f.vel.x < Fx::ZERO, "away from the wall: {:?}", f.vel);
    assert!((f.vel.y - units(2300)).abs() < units(150), "{:?}", f.vel);
}

#[test]
fn jump_against_a_wall_with_the_stick_toward_it_is_a_wall_jump_too() {
    let (f, jumps) = jump_at_the_wall(3, 127);
    assert_eq!(f.wall_jumps, 1);
    assert_eq!(f.air_jumps_left, jumps, "no midair jump spent");
    assert!(f.vel.x < Fx::ZERO, "away from the wall: {:?}", f.vel);
}

#[test]
fn jump_against_a_wall_with_the_stick_held_away_is_a_midair_jump() {
    let (f, jumps) = jump_at_the_wall(3, -127);
    assert_eq!(f.wall_jumps, 0);
    assert_eq!(f.air_jumps_left, jumps - 1, "the midair jump");
}

#[test]
fn every_wall_jumping_body_takes_the_jump_button() {
    let mut any = 0;
    for ch in 0..Sim::new().content.fighters.len() as u8 {
        let can = Sim::new().content.fighters[usize::from(ch)].wall_jump != 0;
        let (f, jumps) = jump_at_the_wall(ch, 0);
        if can {
            any += 1;
            assert_eq!(f.wall_jumps, 1, "class {ch}");
            assert_eq!(f.air_jumps_left, jumps, "class {ch}");
        } else {
            assert_eq!(f.wall_jumps, 0, "class {ch}");
            assert_eq!(f.air_jumps_left, jumps - 1, "class {ch}: a midair jump");
        }
    }
    assert!(any >= 3);
}
