//! Movement rules taken from the reference game: the 9-frame input buffer and the hold buffer, one press per action, the
//! short-hop aerial (and its damage), midair jumps that steer, light and heavy landings, and backward rolls.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD};
use sim_core::state::{FighterState as S, NONE};
use sim_core::Fx;

/// Raises and drops the shield, leaving the fighter at the start of the 11-frame shield release.
fn into_shield_release(sim: &mut Sim) {
    sim.ticks(5, inp(0, 0, SHIELD));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::ShieldRelease);
}

/// Ticks with no input until the fighter can act again (leaves the shield release); returns how many frames that took.
fn until_free(sim: &mut Sim) -> usize {
    let mut n = 0;
    while sim.f().state == S::ShieldRelease {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    n
}

// ---- Input buffer ---------------------------------------------------------------------------------

#[test]
fn a_press_up_to_nine_frames_early_is_buffered_and_one_ten_frames_early_is_not() {
    for (early, jumps) in [(9usize, true), (10, false)] {
        let mut sim = Sim::new();
        into_shield_release(&mut sim);
        let release = usize::from(sim.content.fighters[0].shield_release_frames);
        // Press jump `early` frames before the fighter can act, then let go.
        sim.ticks(release - early, inp(0, 0, 0));
        sim.tick(inp(0, 0, JUMP));
        until_free(&mut sim);
        sim.tick(inp(0, 0, 0));
        assert_eq!(
            sim.f().state == S::JumpSquat,
            jumps,
            "a press {early} frames early"
        );
    }
}

#[test]
fn holding_a_button_buffers_it_however_early_it_was_pressed() {
    let mut sim = Sim::new();
    into_shield_release(&mut sim);
    // Pressed on the first frame of the release (11 frames early) and held: the hold buffer.
    while sim.f().state == S::ShieldRelease {
        sim.tick(inp(0, 0, JUMP));
    }
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
}

#[test]
fn one_press_starts_one_action() {
    let mut sim = Sim::new();
    into_shield_release(&mut sim);
    sim.ticks(6, inp(0, 0, 0));
    sim.tick(inp(0, 0, ATTACK));
    until_free(&mut sim);
    let mut attacks = 0;
    let mut was_attacking = false;
    for _ in 0..120 {
        sim.tick(inp(0, 0, 0));
        let attacking = sim.f().state == S::Attack;
        if attacking && !was_attacking {
            attacks += 1;
        }
        was_attacking = attacking;
    }
    assert_eq!(
        attacks, 1,
        "the buffered press gives one jab, not one per frame of the buffer"
    );
}

#[test]
fn a_held_jump_used_for_the_jump_does_not_also_double_jump() {
    let mut sim = Sim::new();
    sim.ticks(40, inp(0, 0, JUMP));
    assert_eq!(
        sim.f().air_jumps_left,
        1,
        "holding jump through a full hop keeps the midair jump"
    );
}

// ---- Short hops -----------------------------------------------------------------------------------

/// Peak height of a ground jump with the given inputs on the first frames.
fn peak(first: &[u16]) -> Fx {
    let mut sim = Sim::new();
    let mut top = Fx::ZERO;
    for t in 0..80 {
        let b = first.get(t).copied().unwrap_or(0);
        sim.tick(inp(0, 0, b));
        top = top.max(sim.f().pos.y);
    }
    top
}

#[test]
fn jump_and_attack_together_is_a_short_hop_aerial_even_with_jump_held() {
    let full = peak(&[JUMP; 10]);
    let short = peak(&[JUMP]);
    let macro_hop = peak(&[JUMP | ATTACK, JUMP, JUMP, JUMP, JUMP, JUMP]);
    assert!(full > short * fx(3, 2));
    assert!(
        macro_hop < short + fx(1, 10),
        "jump and attack together is a short hop: {macro_hop:?} vs {short:?}"
    );
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP | ATTACK));
    for _ in 0..6 {
        sim.tick(inp(0, 0, JUMP));
    }
    assert_eq!(sim.f().state, S::Attack, "the aerial comes out right away");
    assert!(sim.f().short_hop);
}

/// Damage a neutral aerial deals to a fighter right next to it, from a short hop or not.
fn aerial_damage(short_hop: bool) -> Fx {
    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, fx(1, 2), Fx::from_int(10), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].short_hop = short_hop;
    sim.state.fighters[0].facing = 1;
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            break;
        }
    }
    sim.fighter(1).percent
}

#[test]
fn a_short_hop_aerial_deals_85_percent_damage() {
    let full = aerial_damage(false);
    let short = aerial_damage(true);
    assert!(full > Fx::ZERO, "the aerial connects");
    let ratio = short / full;
    assert!(
        (ratio - fx(85, 100)).abs() < fx(1, 100),
        "short hop / full: {ratio:?}"
    );
}

#[test]
fn landing_ends_the_short_hop() {
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP));
    sim.ticks(5, inp(0, 0, 0));
    assert!(sim.f().short_hop);
    sim.ticks(80, inp(0, 0, 0));
    assert_ne!(sim.f().platform, NONE);
    assert!(!sim.f().short_hop);
}

// ---- Midair jumps ---------------------------------------------------------------------------------

#[test]
fn a_midair_jump_takes_its_drift_from_the_stick() {
    let jump_with = |x: i8| {
        let mut sim = Sim::new();
        let air_speed = sim.content.fighters[0].air_speed;
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), air_speed, Fx::ZERO);
        sim.tick(inp(x, 0, JUMP));
        sim.f().vel.x
    };
    assert!(jump_with(-127) < Fx::ZERO, "it reverses the drift at once");
    assert_eq!(
        jump_with(0),
        Fx::ZERO,
        "with the stick neutral it goes straight up"
    );
    assert!(jump_with(127) > Fx::ZERO);
}

// ---- Landings -------------------------------------------------------------------------------------

#[test]
fn a_landing_at_full_fall_speed_is_heavy_and_a_short_hop_landing_is_light() {
    let p = Sim::new().content.fighters[0];
    assert_eq!((p.landing_lag, p.heavy_landing_lag), (2, 4));

    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP));
    let mut lag = None;
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            lag = Some(sim.f().lag);
            break;
        }
    }
    assert_eq!(lag, Some(p.landing_lag), "a short hop lands lightly");

    let mut sim = Sim::new();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    let mut lag = None;
    for _ in 0..300 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            lag = Some(sim.f().lag);
            break;
        }
    }
    assert_eq!(lag, Some(p.heavy_landing_lag), "a long fall lands heavily");
}

// ---- Rolls ----------------------------------------------------------------------------------------

#[test]
fn a_backward_roll_takes_longer_than_a_forward_one() {
    let roll = |x: i8| {
        let mut sim = Sim::new();
        sim.stand(0, Fx::ZERO, 1);
        sim.ticks(5, inp(0, 0, SHIELD));
        sim.tick(inp(x, 0, SHIELD));
        assert_eq!(sim.f().state, S::Roll);
        let mut n = 0;
        while sim.f().state == S::Roll {
            sim.tick(inp(0, 0, 0));
            n += 1;
        }
        n
    };
    let p = Sim::new().content.fighters[0];
    assert_eq!(roll(127), usize::from(p.roll_frames));
    assert_eq!(roll(-127), usize::from(p.roll_back_frames));
}

// ---- Short-hop fast-fall aerials and the attack stick -----------------------------------------------------------------------------

/// A short hop by `ch` (with `first` pressed on the take-off frame, stick `aim`), fast falling with a fresh down flick on the first
/// falling frame; the movement stick is otherwise `hold`. Returns (frames in the air, the furthest move frame reached, the move).
fn short_hop(ch: u8, first: u16, ff: bool, hold: (i8, i8)) -> (u32, u16, u8) {
    let mut sim = Sim::with_chars([ch, 1 - ch, 0, 1]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(1, Fx::from_int(10), -1);
    sim.tick(inp(hold.0, hold.1, JUMP | first));
    while sim.f().grounded() {
        sim.tick(inp(hold.0, hold.1, first & !JUMP));
    }
    let (mut n, mut reached, mut mv) = (0, 0, 255);
    let mut flicked = false;
    while !sim.f().grounded() && n < 200 {
        let down = ff && !flicked && sim.f().vel.y <= Fx::ZERO;
        flicked |= down;
        let y = if down { -127 } else { hold.1 };
        sim.tick(inp(hold.0, y, first & sim_core::input::buttons::C_ANY));
        if sim.f().state == S::Attack {
            reached = sim.f().state_frame;
            mv = sim.f().move_id;
        }
        n += 1;
    }
    (n, reached, mv)
}

#[test]
fn short_hop_air_times_match_the_reference() {
    // The reference's published short hop and short hop fast fall air times (ultimateframedata.com): the swordfighter 41 and 28,
    // the blaster fighter 30 and 21. Ours count the landing frame too: within one.
    for (ch, sh, shff) in [(0u8, 41u32, 28u32), (1, 30, 21)] {
        let (a, _, _) = short_hop(ch, 0, false, (0, 0));
        let (b, _, _) = short_hop(ch, 0, true, (0, 0));
        assert!(a.abs_diff(sh) <= 1, "class {ch}: short hop {a} vs {sh}");
        assert!(
            b.abs_diff(shff) <= 1,
            "class {ch}: short hop fast fall {b} vs {shff}"
        );
    }
}

#[test]
fn a_fast_fallen_short_hop_aerial_made_on_take_off_finishes_its_hits_before_landing() {
    use sim_core::input::buttons::{C_DOWN, C_LEFT, C_RIGHT, C_UP};
    use sim_core::moves::MoveId;
    // Each aerial from the attack stick on the take-off frame, fast falling as early as possible: every hit of the swordfighter's
    // aerials, and of the blaster fighter's (but its neutral air's long late hit, as in the reference), is out before the landing.
    for ch in [0u8, 1] {
        let sim = Sim::with_chars([ch, 1 - ch, 0, 1]);
        let weapon =
            &sim.content.weapons[usize::from(sim.content.fighters[usize::from(ch)].weapon)];
        for (c, id) in [
            (C_RIGHT, MoveId::FAir),
            (C_LEFT, MoveId::BAir),
            (C_UP, MoveId::UAir),
            (C_DOWN, MoveId::DAir),
        ] {
            let (_, reached, mv) = short_hop(ch, ATTACK | c, true, (0, 0));
            assert_eq!(
                mv, id as u8,
                "class {ch}: the attack stick picks the aerial"
            );
            let last_hit = weapon
                .get(id as u8)
                .hitboxes
                .iter()
                .map(|h| h.end)
                .max()
                .unwrap_or(0);
            assert!(
                reached >= u16::from(last_hit),
                "class {ch} {id:?}: reached frame {reached}, last hit on {last_hit}"
            );
        }
    }
}

#[test]
fn the_movement_stick_fast_falls_and_drifts_through_an_attack_stick_aerial() {
    use sim_core::input::buttons::C_UP;
    use sim_core::moves::MoveId;
    // Up air on the attack stick while the movement stick holds forward: the aerial is the up air (not a forward air), the fighter
    // keeps drifting forward, and the down flick still fast falls it.
    let (air, _, mv) = short_hop(0, ATTACK | C_UP, true, (127, 0));
    assert_eq!(mv, MoveId::UAir as u8);
    let (slow, _, _) = short_hop(0, ATTACK | C_UP, false, (127, 0));
    assert!(air + 8 < slow, "fast fell: {air} vs {slow}");
    // Drift: forward on the movement stick through the aerial moves the fighter further than with it let go.
    let drift = |x: i8| {
        let mut sim = Sim::alone();
        sim.tick(inp(0, 0, JUMP | ATTACK | C_UP));
        sim.ticks(20, inp(x, 0, C_UP));
        sim.f().pos.x
    };
    assert!(drift(127) > drift(0) + fx(1, 2), "drifting forward");
}

#[test]
fn the_attack_stick_on_the_ground_is_a_smash_in_its_direction() {
    use sim_core::input::buttons::{C_DOWN, C_LEFT, C_UP};
    use sim_core::moves::MoveId;
    for (c, id, facing) in [
        (C_UP, MoveId::USmash, 1),
        (C_DOWN, MoveId::DSmash, 1),
        (C_LEFT, MoveId::FSmash, -1),
    ] {
        let mut sim = Sim::alone();
        sim.tick(inp(0, 0, 0));
        sim.tick(inp(0, 0, ATTACK | c));
        assert_eq!(
            (sim.f().move_id, sim.f().facing),
            (id as u8, facing),
            "{id:?}"
        );
    }
}
