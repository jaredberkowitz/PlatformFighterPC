//! The Clutch's phase 3 (class 8, the bounder body): the tongue that lays a fighter as a shell (neutral special, with the egg: E8,
//! `moves::Encase`, `FighterState::Encased`) and the rolling shell (side special), modelled mechanically on a reference character's
//! (13.0.1). Reference data and sources: the ninth moveset's `REFERENCE_DATA.md` (`docs/movesets/`); what is [R], [D] and [E] is said
//! there and in `moves_clutch.rs`.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use sim_core::moves::{clutch_slots, MoveId, Weapon, HIT_GRAB, HIT_THROW, TARGETS_AERIAL};
use sim_core::state::FighterState as S;
use sim_core::{Fx, Vec2};

const BOUNDER: u8 = 8;

/// The bounder (player 0) at 0 facing right and a duelist (player 1) standing `gap` ahead facing it at `percent`, the others out.
fn pair(gap: Fx, percent: i32) -> Sim {
    let mut sim = Sim::with_chars([BOUNDER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim.state.fighters[0].invuln = 0;
    sim.state.fighters[1].invuln = 0;
    sim.state.fighters[1].percent = Fx::from_int(percent);
    sim
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(BOUNDER)].weapon)]
}

fn near(a: Fx, b: Fx, thousandths: i32) -> bool {
    (a - b).abs() <= fx(thousandths, 1000)
}

fn idle() -> sim_core::Input {
    inp(0, 0, 0)
}

/// Presses special (frame 1) and runs until player 1 is shut in the egg. Returns the frame (counted from the press as 1) the egg
/// was laid on, and the frame the tongue caught on.
fn lay(sim: &mut Sim) -> (u16, u16) {
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(MoveId::from_index(sim.f().move_id), MoveId::NSpecial);
    let mut frame = 1u16;
    let mut caught = 0;
    while sim.fighter(1).state != S::Encased {
        sim.tick(idle());
        frame += 1;
        if caught == 0 && sim.fighter(1).state == S::Grabbed {
            caught = frame;
        }
        assert!(frame < 80, "no egg");
    }
    (frame, caught)
}

/// Frames the egg lasts after it is laid (the update it breaks out on, counted from the frame after the lay), with player 1's input
/// on each frame from `p1`.
fn egg_frames(sim: &mut Sim, p1: impl Fn(u16) -> sim_core::Input) -> u16 {
    let mut n = 0u16;
    while sim.fighter(1).state == S::Encased {
        n += 1;
        sim.tick2(idle(), p1(n));
        assert!(n < 400, "the egg never broke");
    }
    n
}

/// `ceil(x)` for a positive fixed-point number.
fn ceil(x: Fx) -> u16 {
    let whole = x.floor_int();
    (if Fx::from_int(whole) == x {
        whole
    } else {
        whole + 1
    }) as u16
}

// ---- The tongue (neutral special) ------------------------------------------------------------------------------------------------

#[test]
fn the_tongue_is_a_command_grab_on_frames_19_to_22_with_faf_42() {
    let sim = pair(Fx::from_int(8), 0);
    let mv = kit(&sim).get(MoveId::NSpecial as u8);
    assert_eq!(mv.total_frames, 40, "FAF 42");
    for h in &mv.hitboxes {
        assert_eq!(h.kind, HIT_GRAB);
        assert!(h.command);
        assert_eq!(h.start, 18, "from frame 19");
        assert!(h.end == 18 || h.end == 21, "the mouth on 19, the tip to 22");
    }
    // The mouth box is out on frame 19 only, the tip's air box catches only fighters in the air.
    assert!(mv
        .hitboxes
        .iter()
        .any(|h| h.end == 18 && h.radius == fx(7, 10)));
    assert!(mv.hitboxes.iter().any(|h| h.targets == TARGETS_AERIAL));
    assert_eq!(mv.catch_move, Some(clutch_slots::LAY as u8));
    // A whiff: actionable on frame 42.
    let mut sim = pair(Fx::from_int(8), 0);
    sim.tick(inp(0, 0, SPECIAL));
    let mut frame = 1u16;
    while sim.f().state == S::Attack {
        sim.tick(idle());
        frame += 1;
    }
    assert_eq!(frame + 1, 42);
}

#[test]
fn the_tongue_catches_on_frame_19_on_the_ground_in_the_air_and_through_a_shield() {
    // On the ground, at 2.5 units.
    let mut sim = pair(fx(25, 10), 0);
    let (_, caught) = lay(&mut sim);
    assert_eq!(caught, 19);
    // Through a shield.
    let mut sim = pair(fx(25, 10), 0);
    sim.state.fighters[1].state = S::Shield;
    sim.state.fighters[1].shield_hp = sim.content.rules.shield_max;
    sim.tick2(inp(0, 0, SPECIAL), inp(0, 0, SHIELD));
    let mut caught = false;
    for _ in 0..25 {
        sim.tick2(idle(), inp(0, 0, SHIELD));
        caught |= sim.fighter(1).state == S::Grabbed;
    }
    assert!(caught, "a shield does not stop it");
    // In the air, held still at the mouth's height ahead.
    let mut sim = pair(Fx::from_int(8), 0);
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 0..25 {
        sim.put_airborne(1, fx(2, 1), fx(1, 2), Fx::ZERO, Fx::ZERO);
        sim.tick(idle());
        if sim.fighter(1).state == S::Grabbed {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn a_catch_on_19_lays_on_46_is_intangible_on_32_to_54_and_acts_on_63() {
    let mut sim = pair(fx(25, 10), 0);
    let (laid, caught) = lay(&mut sim);
    assert_eq!(caught, 19);
    // The laying part starts on 26 (a 6-frame swallow, conflict 5 [D]); its release on its 21: frame 46.
    assert_eq!(laid, 46);
    assert_eq!(MoveId::from_index(sim.f().move_id), clutch_slots::LAY);
    let mut frame = laid;
    while sim.f().state == S::Attack {
        sim.tick(idle());
        frame += 1;
    }
    assert_eq!(frame + 1, 63, "62 frames when successful (FAF 63)");
    // Intangible on the laying part's 7-29: the catch move's 13-35, frames 32 to 54 of the whole move.
    let mut sim = pair(fx(25, 10), 0);
    sim.tick(inp(0, 0, SPECIAL));
    for frame in 2..=60u16 {
        sim.tick(idle());
        let intangible = combat::is_intangible(sim.f());
        assert_eq!(
            intangible,
            (32..=54).contains(&frame),
            "frame {frame}: intangible {intangible}"
        );
    }
}

#[test]
fn the_laying_throw_deals_7_percent_and_lays_the_egg_behind() {
    let mut sim = pair(fx(25, 10), 0);
    let lay_move = kit(&sim).get(clutch_slots::LAY as u8).clone();
    let throw = lay_move
        .hitboxes
        .iter()
        .find(|h| h.kind == HIT_THROW)
        .unwrap();
    assert_eq!(throw.damage, Fx::from_int(7));
    assert_eq!(
        (throw.angle, throw.base_knockback, throw.knockback_growth),
        (361, 0, 100)
    );
    assert_eq!(throw.hitlag, 0);
    lay(&mut sim);
    let v = sim.fighter(1);
    assert_eq!(v.percent, Fx::from_int(7) * sim.content.rules.damage_mult);
    // 0.7 and 2 of the reference's units, behind the bounder (it faces right) [R speeds; E: the side].
    assert_eq!(v.vel, Vec2::new(-fx(7, 80), fx(2, 8)));
    assert!(!v.grounded());
    assert_eq!(v.hitlag, 0);
}

// ---- The egg (E8) ----------------------------------------------------------------------------------------------------------------

#[test]
fn the_egg_lasts_84_frames_at_0_percent_and_264_at_100_and_above() {
    // 100 + 1.8 x min(p, 100) less 2 a percent of the throw's 8.4 actually taken: 83.2 and 263.2, 84 and 264 frames (the reference's
    // "84 to 264 with the one-on-one multiplier") [R].
    for (percent, want) in [(0, 84u16), (50, 174), (100, 264), (150, 264)] {
        let mut sim = pair(fx(25, 10), percent);
        lay(&mut sim);
        let frames = egg_frames(&mut sim, |_| idle());
        assert_eq!(frames, want, "at {percent}%");
    }
}

#[test]
fn mashing_cuts_7_a_stick_flick_and_12_6_a_button_press() {
    // A button pressed every other frame: 1 + 12.6 on those frames, 1 on the others.
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    let pressed = |n: u16| {
        if n % 2 == 1 {
            inp(0, 0, ATTACK)
        } else {
            idle()
        }
    };
    let frames = egg_frames(&mut sim, pressed);
    let mut left = fx(2632, 10);
    let mut want = 0u16;
    while left > Fx::ZERO {
        want += 1;
        left -= Fx::ONE;
        if want % 2 == 1 {
            left -= fx(126, 10);
        }
    }
    assert_eq!(frames, want);
    // The stick flicked sideways from neutral every other frame: 1 + 7.
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    let flicked = |n: u16| {
        if n % 2 == 1 {
            inp(127, 0, 0)
        } else {
            idle()
        }
    };
    let frames = egg_frames(&mut sim, flicked);
    let mut left = fx(2632, 10);
    let mut want = 0u16;
    while left > Fx::ZERO {
        want += 1;
        left -= Fx::ONE;
        if want % 2 == 1 {
            left -= Fx::from_int(7);
        }
    }
    assert_eq!(frames, want);
}

#[test]
fn a_hit_on_the_egg_deals_0_66_times_never_launches_and_cuts_twice_the_damage_taken() {
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    // Let it land, then put a duelist's jab on it: the bounder is not the attacker here, player 2 is.
    for _ in 0..90 {
        sim.tick(idle());
    }
    assert!(sim.fighter(1).grounded(), "the egg landed and stays an egg");
    assert_eq!(sim.fighter(1).state, S::Encased);
    let x = sim.fighter(1).pos.x;
    sim.state.fighters[2] = sim.state.fighters[0];
    sim.state.fighters[2].char_id = 0;
    sim.state.fighters[2].active = true;
    sim.stand(2, x - fx(12, 10), 1);
    sim.state.fighters[0].pos.x = Fx::from_int(9);
    let before = sim.fighter(1).percent;
    let timer = sim.fighter(1).encase_timer;
    let mut inputs = [sim_core::Input::default(); sim_core::MAX_FIGHTERS];
    inputs[2] = inp(0, 0, ATTACK);
    let mut hit_at = None;
    for n in 0..12 {
        sim_core::step(&mut sim.state, &sim.content, &inputs);
        inputs[2] = idle();
        if sim.fighter(1).percent != before {
            hit_at = Some(n);
            break;
        }
    }
    assert!(hit_at.is_some(), "the jab hit the egg");
    // (Whichever of the jab's boxes landed: 0.66 of its damage, times the one-on-one 1.2.)
    let added = sim.fighter(1).percent - before;
    let mult = fx(66, 100) * sim.content.rules.damage_mult;
    assert!(
        sim.content.weapons[0]
            .get(MoveId::Jab as u8)
            .hitboxes
            .iter()
            .any(|h| near(added, h.damage * mult, 1)),
        "{added:?}"
    );
    assert!(added < Fx::from_int(3), "less than the jab's own 3%");
    let v = sim.fighter(1);
    assert_eq!(v.state, S::Encased, "never launched");
    assert!(v.hitlag > 0, "the hit's hitlag stands");
    // (A frame's countdown for every frame up to the hit's own, which ran before the hit.)
    let frames = Fx::from_int(hit_at.unwrap() + 1);
    assert!(near(
        v.encase_timer,
        timer - frames - added * Fx::from_int(2),
        1
    ));
}

#[test]
fn an_egg_cannot_be_grabbed_or_egged_again() {
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    for _ in 0..90 {
        sim.tick(idle());
    }
    let x = sim.fighter(1).pos.x;
    sim.stand(0, x - fx(2, 1), 1);
    // A second tongue and a grab both pass over it.
    for buttons in [SPECIAL, sim_core::input::buttons::GRAB] {
        sim.tick(inp(0, 0, buttons));
        for _ in 0..45 {
            sim.tick(idle());
            assert_eq!(sim.fighter(1).state, S::Encased);
        }
        sim.stand(0, x - fx(2, 1), 1);
    }
}

#[test]
fn the_egg_grows_from_0_45_to_full_in_20_frames() {
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    let height = sim.content.fighters[0].ecb_height;
    let radius = |sim: &Sim| combat::body_hurtboxes(&sim.content, sim.fighter(1))[0].1;
    assert_eq!(radius(&sim), height / Fx::from_int(2) * fx(45, 100));
    for _ in 0..10 {
        sim.tick(idle());
    }
    let r10 = radius(&sim);
    let want = height / Fx::from_int(2) * fx(725, 1000);
    assert!(
        near(r10, want, 2),
        "{r10:?} {want:?} age {}",
        sim.fighter(1).encase_age
    );
    for _ in 0..15 {
        sim.tick(idle());
    }
    assert_eq!(radius(&sim), height / Fx::from_int(2));
    // Every hurtbox is the egg.
    let hurt = combat::body_hurtboxes(&sim.content, sim.fighter(1));
    assert!(hurt.iter().all(|h| *h == hurt[0]));
}

#[test]
fn breaking_out_jumps_at_2_5_and_is_intangible_for_7_frames() {
    let mut sim = pair(fx(25, 10), 0);
    lay(&mut sim);
    egg_frames(&mut sim, |_| idle());
    let v = *sim.fighter(1);
    assert_eq!(v.state, S::Airborne);
    assert_eq!(v.vel.y, fx(25, 80));
    assert_eq!(v.encase_timer, Fx::ZERO);
    let mut intangible = 0;
    for _ in 0..12 {
        if combat::is_intangible(sim.fighter(1)) {
            intangible += 1;
        }
        sim.tick(idle());
    }
    // The break-out frame and 6 more.
    assert_eq!(intangible, 7);
}

#[test]
fn on_the_ground_a_flick_shifts_the_egg_and_a_flick_up_hops_it_in_the_air_barely() {
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    for _ in 0..90 {
        sim.tick(idle());
    }
    assert!(sim.fighter(1).grounded());
    let x = sim.fighter(1).pos.x;
    sim.tick2(idle(), inp(127, 0, 0));
    // 0.3 a press (0.0375 world) [R], less the frame's traction.
    let traction = sim.content.fighters[0].ground_friction;
    assert_eq!(sim.fighter(1).vel.x, fx(3, 80) - traction);
    assert!(sim.fighter(1).pos.x > x);
    sim.tick2(idle(), idle());
    sim.tick2(idle(), inp(0, 127, 0));
    assert!(!sim.fighter(1).grounded(), "a hop at 1 (0.125)");
    assert_eq!(sim.fighter(1).state, S::Encased);
    // In the air a flick only nudges it: 0.01 / 0.1 (0.00125 / 0.0125) [R].
    let vx = sim.fighter(1).vel.x;
    sim.tick2(idle(), idle());
    let drift = sim.fighter(1).vel.x;
    sim.tick2(idle(), inp(127, 0, 0));
    let air_friction = sim.content.fighters[0].air_friction;
    assert!(near(
        sim.fighter(1).vel.x - drift,
        fx(1, 800) - air_friction.min(drift.abs() + fx(1, 800)),
        1
    ));
    let _ = vx;
}

#[test]
fn an_egg_falling_off_the_stage_is_knocked_out() {
    let mut sim = pair(fx(25, 10), 100);
    lay(&mut sim);
    sim.put_airborne(1, Fx::from_int(15), Fx::from_int(-2), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].state = S::Encased;
    let stocks = sim.fighter(1).stocks;
    for _ in 0..300 {
        sim.tick(idle());
        if sim.fighter(1).state == S::Respawn {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Respawn);
    assert!(sim.fighter(1).stocks < stocks || sim.state.rules.stocks == 0);
    assert_eq!(sim.fighter(1).encase_timer, Fx::ZERO);
}

#[test]
fn an_aerial_lay_sets_the_bounders_vertical_speed_to_1() {
    let mut sim = pair(Fx::from_int(8), 0);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SPECIAL));
    let mut frame = 1;
    loop {
        sim.put_airborne(
            1,
            sim.f().pos.x + fx(2, 1),
            sim.f().pos.y + fx(1, 2),
            Fx::ZERO,
            Fx::ZERO,
        );
        sim.state.fighters[0].vel.y = Fx::ZERO;
        sim.tick(idle());
        frame += 1;
        if sim.fighter(1).state == S::Grabbed {
            break;
        }
        assert!(frame < 25);
    }
    while sim.fighter(1).state != S::Encased {
        sim.tick(idle());
    }
    // Set on the lay's frame, then the body's gravity for the frame's move.
    let gravity = sim.content.fighters[usize::from(BOUNDER)].gravity;
    assert_eq!(sim.f().vel.y, fx(1, 8) - gravity);
}

#[test]
fn an_egg_resimulates_exactly_from_a_snapshot() {
    let mut sim = pair(fx(25, 10), 40);
    lay(&mut sim);
    let saved = sim.state;
    let script = |n: u16| match n % 5 {
        0 => inp(127, 0, 0),
        2 => inp(0, 0, JUMP),
        3 => inp(0, 127, 0),
        _ => idle(),
    };
    let mut hashes = Vec::new();
    for n in 0..120u16 {
        sim.tick2(idle(), script(n));
        hashes.push(sim.state.checksum());
    }
    sim.state = saved;
    for (n, h) in hashes.iter().enumerate() {
        sim.tick2(idle(), script(n as u16));
        assert_eq!(sim.state.checksum(), *h, "frame {n}");
    }
}

#[test]
fn the_egg_formula_reads_its_numbers() {
    let sim = pair(Fx::ZERO, 0);
    let e = kit(&sim)
        .get(clutch_slots::LAY as u8)
        .encase
        .expect("the lay has an egg");
    assert_eq!(e.frames(Fx::ZERO), Fx::from_int(100));
    assert!(near(e.frames(Fx::from_int(100)), Fx::from_int(280), 2));
    assert_eq!(e.frames(Fx::from_int(150)), e.frames(Fx::from_int(100)));
    assert_eq!(ceil(e.frames(Fx::ZERO) - fx(168, 10)), 84);
    assert_eq!(
        (e.mash_stick, e.mash_button),
        (Fx::from_int(7), fx(126, 10))
    );
    assert_eq!((e.damage_taken, e.exit_intangible), (66, 7));
}

// ---- The rolling shell (side special) --------------------------------------------------------------------------------------------

/// The roll from a standing start, the stick at `stick` from the roll's start.
fn roll(sim: &mut Sim) {
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(MoveId::from_index(sim.f().move_id), MoveId::SideSpecial);
}

#[test]
fn the_roll_hops_at_2_and_touches_down_with_its_hit_out_on_32() {
    let mut sim = pair(Fx::from_int(10), 0);
    sim.state.fighters[1].invuln = 255;
    roll(&mut sim);
    sim.tick(idle());
    assert_eq!(sim.f().vel.y, fx(2, 8), "the hop at 0.25 on frame 2");
    let mut landed = 0;
    for frame in 3..=40u16 {
        sim.tick(idle());
        if landed == 0 && sim.f().grounded() {
            landed = frame;
        }
    }
    assert!((31..=33).contains(&landed), "touched down on {landed}");
    let mv = kit(&sim).get(MoveId::SideSpecial as u8);
    assert!(mv.hitboxes.iter().all(|h| h.start == 31));
    assert_eq!(mv.rehit, Some((31, 32)));
    let add = mv.charge_add.unwrap();
    assert_eq!((add.full, add.add), (40, Fx::from_int(4)));
    assert!(!mv.helpless_after);
}

#[test]
fn the_stick_speeds_the_roll_up_to_0_48_and_its_hit_to_10_plus_the_speed() {
    let mut sim = pair(Fx::from_int(10), 0);
    sim.state.fighters[1].invuln = 255;
    roll(&mut sim);
    let mut top = Fx::ZERO;
    for _ in 0..140 {
        // Held in the middle of the stage (it would roll off), the stick forward.
        sim.state.fighters[0].pos.x = Fx::ZERO;
        let x = if sim.f().facing > 0 { 127 } else { -127 };
        sim.tick(inp(x, 0, 0));
        top = top.max(sim.f().vel.x.abs());
        if sim.f().state != S::Attack {
            break;
        }
    }
    assert!(near(top, fx(48, 100), 1), "top speed {top:?}");
    // The damage at a speed: 10 plus the speed in the reference's units (charge in tenths of them).
    let mv = kit(&sim).get(MoveId::SideSpecial as u8);
    let add = mv.charge_add.unwrap();
    assert_eq!(add.damage(Fx::from_int(10), 38), fx(138, 10));
}

#[test]
fn special_stops_the_roll_from_25_frames_in_and_the_quickest_roll_acts_on_85() {
    let mut sim = pair(Fx::from_int(10), 0);
    sim.state.fighters[1].invuln = 255;
    roll(&mut sim);
    let mut frame = 1u16;
    while sim.f().state == S::Attack {
        // A press on 44 is too early (the roll's 24th frame); one on 46 (its 26th: 25 frames in) stops it.
        frame += 1;
        let b = if frame == 44 || frame == 46 {
            SPECIAL
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
        if frame == 44 {
            assert_eq!(MoveId::from_index(sim.f().move_id), MoveId::SideSpecial);
        }
        assert!(frame < 200);
    }
    assert_eq!(frame + 1, 85);
}

#[test]
fn the_roll_ends_by_itself_after_180_frames_and_jumps_once() {
    let mut sim = pair(Fx::from_int(10), 0);
    sim.state.fighters[1].invuln = 255;
    roll(&mut sim);
    let mut frame = 1u16;
    let mut jumps = 0;
    let mut was_rising = false;
    let mut ending = 0;
    while sim.f().state == S::Attack {
        // Jump held from frame 40: one jump only. (Held in the middle of the stage: it would roll off.)
        sim.state.fighters[0].pos.x = Fx::ZERO;
        let b = if frame >= 40 { JUMP } else { 0 };
        sim.tick(inp(0, 0, b));
        frame += 1;
        let rising = sim.f().vel.y > Fx::ZERO;
        if rising && !was_rising && frame > 20 {
            jumps += 1;
        }
        was_rising = rising;
        if ending == 0 && MoveId::from_index(sim.f().move_id) == clutch_slots::ROLL_END {
            ending = frame;
        }
        assert!(frame < 300);
    }
    assert_eq!(jumps, 1);
    // The roll from 21 for 180 frames: the ending from 201, actionable 40 frames on.
    assert_eq!(ending, 201);
    assert_eq!(frame + 1, 240);
}
