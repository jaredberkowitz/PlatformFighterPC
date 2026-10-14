//! The carry (E7, sim 98): the Bracers' forward throw is the reference's cargo hold. The 15-frame invincible lift; the clock
//! `(90 + 1.7p) x 11 / 6 - g` with the grab's time and mashing carried over; the walk at x1.1; one jump; walking off the stage with the
//! held fighter overhead; a knock-out during the carry; mashing out and running out; the four cargo throws' frames and data (the back
//! one turning round, the down one's boost in the air); the knockback armor (80, not subtractive); a rollback replay through a carry
//! off the stage. Reference data: the eighth moveset's `REFERENCE_DATA.md` ("The cargo hold"), 13.0.1.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD, SPECIAL};
use sim_core::moves::bracers_slots::{
    CARGO_ARMOR_KB, CARGO_BACK, CARGO_DOWN, CARGO_FORWARD, CARGO_UP,
};
use sim_core::moves::{MoveId, Weapon, ARMOR_KNOCKBACK, HIT_THROW};
use sim_core::state::{FighterState as S, NONE};
use sim_core::{step, FighterParams, Fx, Input, Rng, Vec2, MAX_FIGHTERS};

const HAULER: u8 = 7;

/// The hauler (player 0) at `x` facing right and a duelist (player 1) just in front of it, the others out of the match.
fn duel_at(x: Fx) -> Sim {
    let mut sim = Sim::with_chars([HAULER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, x, 1);
    sim.stand(1, x + fx(15, 10), -1);
    sim
}

fn arms(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(HAULER)].weapon)]
}

fn body(sim: &Sim) -> FighterParams {
    sim.content.fighters[usize::from(HAULER)]
}

fn hold_y(sim: &Sim) -> Fx {
    arms(sim)
        .get(MoveId::FThrow as u8)
        .carry
        .expect("carry")
        .hold_at
        .1
}

/// Grabs the duelist (player 1 doing `p1` meanwhile).
fn grab(sim: &mut Sim, p1: Input) {
    sim.tick2(inp(0, 0, GRAB), p1);
    for _ in 0..40 {
        sim.tick2(inp(0, 0, 0), p1);
        if sim.f().state == S::Grabbing {
            return;
        }
    }
    panic!("the grab never connected");
}

/// The forward throw: the stick forward for one frame (the lift's frame 1), then the lift's frames 2-15. Returns the held fighter's
/// clock as the lift set it.
fn lift(sim: &mut Sim) -> u16 {
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(sim.f().move_id, MoveId::FThrow as u8);
    let clock = sim.fighter(1).grab_timer;
    for _ in 2..=15 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Carrying, "carrying from frame 16");
    clock
}

/// Grabs the duelist and lifts it: the hauler is carrying it.
fn carrying_at(x: Fx) -> Sim {
    let mut sim = duel_at(x);
    grab(&mut sim, inp(0, 0, 0));
    lift(&mut sim);
    sim
}

#[test]
fn the_forward_throw_is_the_carrys_lift_and_the_others_are_plain_throws() {
    let sim = duel_at(Fx::ZERO);
    let w = arms(&sim);
    let lift = w.get(MoveId::FThrow as u8);
    let carry = lift.carry.expect("the forward throw is the carry");
    assert!(lift.hitboxes.is_empty(), "the lift has no hit");
    assert_eq!(lift.total_frames + 2, 16, "carrying from frame 16");
    assert_eq!(lift.intangible, 15, "invincible 1-15");
    assert_eq!((carry.walk, carry.clock_num, carry.clock_den), (110, 11, 6));
    assert_eq!(carry.jumps, 1);
    assert!(!carry.ledge, "no ledge [E: gap 4]");
    let armor = carry.armor.expect("armor");
    assert_eq!(armor.kind, ARMOR_KNOCKBACK);
    assert_eq!(armor.threshold, Fx::from_int(CARGO_ARMOR_KB));
    assert!(!armor.subtract);
    assert_eq!(
        carry.throws,
        [
            CARGO_FORWARD as u8,
            CARGO_BACK as u8,
            CARGO_UP as u8,
            CARGO_DOWN as u8
        ]
    );
    // Every other weapon's forward throw is still a plain throw.
    for (i, other) in sim.content.weapons.iter().enumerate() {
        if i != usize::from(body(&sim).weapon) {
            assert!(
                other.get(MoveId::FThrow as u8).carry.is_none(),
                "weapon {i}"
            );
        }
    }
}

#[test]
fn the_cargo_throws_have_the_references_data() {
    let sim = duel_at(Fx::ZERO);
    let w = arms(&sim);
    // (slot, release frame, FAF, damage in tenths, angle, BK, KS, behind)
    for (id, release, faf, damage, angle, bk, ks, behind) in [
        (CARGO_FORWARD, 15u8, 46u8, 120, 60i16, 90i16, 46i16, false),
        (CARGO_BACK, 16, 43, 130, 72, 85, 52, true),
        (CARGO_UP, 15, 30, 120, 92, 65, 53, false),
        (CARGO_DOWN, 17, 45, 110, 46, 70, 45, false),
    ] {
        let mv = w.get(id as u8);
        assert_eq!(u16::from(mv.total_frames) + 2, u16::from(faf), "{id:?} FAF");
        assert_eq!(mv.hitboxes.len(), 1, "{id:?}: one throw row");
        let hb = mv.hitboxes[0];
        assert_eq!(hb.kind, HIT_THROW);
        assert_eq!((hb.start + 1, hb.end + 1), (release, release), "{id:?}");
        assert_eq!(hb.damage, Fx::from_ratio(damage, 10), "{id:?}");
        assert_eq!(
            (hb.angle, hb.base_knockback, hb.knockback_growth),
            (angle, bk, ks)
        );
        assert_eq!(hb.x < Fx::ZERO, behind, "{id:?}");
        assert_eq!(mv.turns_around, behind, "{id:?}");
        assert_eq!(
            mv.intangible, 0,
            "{id:?}: the cargo throws have no invincibility"
        );
        assert!(
            mv.hold_at.is_some(),
            "{id:?} holds overhead until the release"
        );
    }
}

#[test]
fn the_lift_is_invincible_for_15_frames_and_holds_the_fighter_overhead() {
    let mut sim = duel_at(Fx::ZERO);
    grab(&mut sim, inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    // Frame 1 of the lift: intangible through frame 15 (the counter runs down from 15).
    assert_eq!(sim.f().invuln, 15);
    let up = sim.fighter(1).pos - sim.f().pos;
    assert_eq!(up, Vec2::new(Fx::ZERO, hold_y(&sim)), "lifted overhead");
    for frame in 2..=15u8 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().invuln > 0, "intangible on {frame}");
    }
    assert_eq!(sim.f().state, S::Carrying);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().invuln, 0, "frame 16: no longer");
}

#[test]
fn the_clock_is_the_references_formula_with_the_grab_time_taken_off() {
    // At 0% straight out of the grab: 90 x 11 / 6 = 165, less the frames the hold already ran.
    let mut sim = duel_at(Fx::ZERO);
    grab(&mut sim, inp(0, 0, 0));
    let used = sim.fighter(1).grab_start - sim.fighter(1).grab_timer;
    assert_eq!(sim.fighter(1).grab_start, 90, "the hold at 0%");
    assert!(used <= 1);
    assert_eq!(lift(&mut sim), 165 - used);
    assert_eq!(sim.fighter(1).grab_start, 0, "used once");
    // After 60 frames of grab: 105.
    let mut sim = duel_at(Fx::ZERO);
    grab(&mut sim, inp(0, 0, 0));
    while sim.fighter(1).grab_timer > 30 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(lift(&mut sim), 105);
    // At 50%: (90 + 85) x 11 / 6 = 320 (rounded down), less the time used.
    let mut sim = duel_at(Fx::ZERO);
    sim.state.fighters[1].percent = Fx::from_int(50);
    grab(&mut sim, inp(0, 0, 0));
    let used = sim.fighter(1).grab_start - sim.fighter(1).grab_timer;
    assert_eq!(lift(&mut sim), 320 - used);
}

#[test]
fn mashing_during_the_grab_carries_over_and_mashing_during_the_carry_shortens_it() {
    // Mashing in the grab is in `g`: the carry's clock is that much shorter.
    let mut sim = duel_at(Fx::ZERO);
    grab(&mut sim, inp(0, 0, 0));
    for t in 0..10 {
        let b = if t % 2 == 0 { ATTACK } else { 0 };
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
    }
    let used = sim.fighter(1).grab_start - sim.fighter(1).grab_timer;
    assert!(used >= 10 + 5 * 14, "five presses taken off: {used}");
    assert_eq!(lift(&mut sim), 165 - used);
    // The carry's clock runs one a frame, and more for each press.
    let carried = |mash: bool| -> u32 {
        let mut sim = carrying_at(Fx::ZERO);
        let mut n = 0;
        while sim.f().state == S::Carrying {
            let b = if mash && n % 2 == 0 { ATTACK } else { 0 };
            sim.tick2(inp(0, 0, 0), inp(0, 0, b));
            n += 1;
            assert!(n < 400);
        }
        n
    };
    let plain = carried(false);
    let mashed = carried(true);
    assert!(mashed + 40 < plain, "mashing {mashed} against {plain}");
}

#[test]
fn the_carry_runs_out_on_its_clock_and_lets_go() {
    let mut sim = duel_at(Fx::ZERO);
    grab(&mut sim, inp(0, 0, 0));
    let clock = lift(&mut sim);
    let mut frames = 0u16;
    while sim.f().state == S::Carrying {
        assert_eq!(sim.fighter(1).state, S::Grabbed);
        sim.tick(inp(0, 0, 0));
        frames += 1;
        assert!(frames < 400);
    }
    // (The clock ran on the lift's last frame too, the frame the carry began.)
    assert_eq!(frames + 1, clock);
    assert_eq!(sim.f().grab_with, NONE);
    assert_eq!(sim.fighter(1).grab_with, NONE);
    assert_ne!(sim.fighter(1).state, S::Grabbed);
    assert!(
        sim.fighter(1).grab_immune > 0,
        "the ordinary release [E: gap 5]"
    );
}

#[test]
fn the_carry_walks_at_one_point_one_times_the_walk_and_does_nothing_else() {
    let mut sim = carrying_at(Fx::from_int(-6));
    let p = body(&sim);
    let walk = p.walk_speed * Fx::from_ratio(110, 100);
    for _ in 0..40 {
        sim.tick(inp(127, 0, 0));
        assert_eq!(sim.f().state, S::Carrying);
        assert_eq!(
            sim.fighter(1).pos,
            sim.f().pos + Vec2::new(Fx::ZERO, hold_y(&sim)),
            "the held fighter stays overhead"
        );
    }
    assert_eq!(sim.f().vel.x, walk, "x1.1 the walk speed");
    assert!(walk > p.walk_speed);
    // The acceleration: x1.1 both of the walk's.
    let mut sim = carrying_at(Fx::from_int(-6));
    sim.tick(inp(127, 0, 0));
    let k = Fx::from_ratio(110, 100);
    let accel = p.walk_accel * k + p.walk_accel_stick * k;
    assert_eq!(sim.f().vel.x, accel);
    // Back turns at once and walks back.
    for _ in 0..30 {
        sim.tick(inp(-127, 0, 0));
    }
    assert_eq!(sim.f().facing, -1);
    assert_eq!(sim.f().vel.x, -walk);
    // No dash, shield, special or grab: still carrying.
    for b in [SHIELD, SPECIAL, GRAB] {
        sim.tick(inp(0, 0, b));
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().state, S::Carrying, "button {b}");
    }
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(127, 0, 0));
    sim.tick(inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Carrying, "a flick does not dash");
    assert!(sim.f().vel.x <= walk);
}

#[test]
fn the_carry_has_one_jump_in_all() {
    let mut sim = carrying_at(Fx::ZERO);
    let p = body(&sim);
    sim.tick(inp(0, 0, JUMP));
    for _ in 0..p.jump_squat_frames {
        assert!(sim.f().grounded(), "the jump squat");
        sim.tick(inp(0, 0, JUMP));
    }
    assert!(!sim.f().grounded(), "off the ground");
    assert_eq!(sim.f().state, S::Carrying);
    assert_eq!(sim.f().carry_jumps, 0);
    assert_eq!(sim.fighter(1).pos.y, sim.f().pos.y + hold_y(&sim));
    // A second press in the air does nothing.
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
    }
    let vy = sim.f().vel.y;
    sim.tick(inp(0, 0, JUMP));
    assert!(sim.f().vel.y < vy, "no second jump");
    // Down again, still carrying, still no jump.
    let mut n = 0;
    while !sim.f().grounded() {
        sim.tick(inp(0, 0, 0));
        n += 1;
        assert!(n < 200);
    }
    assert_eq!(sim.f().state, S::Carrying, "lands into the carry");
    sim.tick(inp(0, 0, JUMP));
    for _ in 0..10 {
        sim.tick(inp(0, 0, JUMP));
    }
    assert!(sim.f().grounded(), "the one jump is spent");
    // Walked off without jumping, the jump is a midair one.
    let mut sim = carrying_at(Fx::from_int(8));
    while sim.f().grounded() {
        sim.tick(inp(127, 0, 0));
    }
    assert_eq!(sim.f().state, S::Carrying);
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().vel.y, p.air_jump_velocity - p.gravity);
    assert_eq!(sim.f().carry_jumps, 0);
}

#[test]
fn walking_off_the_stage_takes_the_held_fighter_along_and_a_knock_out_lets_go() {
    let mut sim = duel_at(Fx::from_int(8));
    // (At 50% the clock is long enough to reach the blast zone.)
    sim.state.fighters[1].percent = Fx::from_int(50);
    let stocks = sim.f().stocks;
    grab(&mut sim, inp(0, 0, 0));
    lift(&mut sim);
    let mut n = 0;
    while sim.f().grounded() {
        sim.tick(inp(127, 0, 0));
        n += 1;
        assert!(n < 100);
    }
    assert!(sim.f().pos.x > Fx::from_int(11), "over the edge");
    let mut lowest = Fx::ZERO;
    while sim.f().stocks == stocks {
        assert_eq!(sim.f().state, S::Carrying);
        assert_eq!(sim.fighter(1).grab_with, 0);
        assert_eq!(
            sim.fighter(1).pos,
            sim.f().pos + Vec2::new(Fx::ZERO, hold_y(&sim))
        );
        lowest = sim.f().pos.y;
        sim.tick(inp(0, 0, 0));
        n += 1;
        assert!(n < 400, "never fell out");
    }
    assert!(lowest < Fx::from_int(-15), "fell to the blast zone");
    // The holder went first: the held fighter is let go in the air, above it.
    assert_eq!(sim.fighter(1).grab_with, NONE);
    assert_eq!(sim.fighter(1).state, S::Airborne);
    assert!(
        sim.fighter(1).stocks == stocks,
        "the held fighter survives the moment"
    );
}

/// Carrying at `x` (in the air if `air`, walked off the right edge), then attack with the stick at `stick`. Returns the sim on the
/// throw's first frame.
fn throw_from_carry(x: Fx, air: bool, stick: (i8, i8)) -> Sim {
    let mut sim = carrying_at(x);
    if air {
        while sim.f().grounded() {
            sim.tick(inp(127, 0, 0));
        }
    }
    sim.tick(inp(stick.0, stick.1, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    sim
}

#[test]
fn each_cargo_throw_releases_on_its_frame_for_its_damage_and_ends_on_its_faf() {
    // (stick, slot, release frame, FAF, damage in tenths); attack alone is the forward throw.
    for (stick, id, release, faf, damage) in [
        ((0i8, 0i8), CARGO_FORWARD, 15u16, 46u16, 120),
        ((127, 0), CARGO_FORWARD, 15, 46, 120),
        ((-127, 0), CARGO_BACK, 16, 43, 130),
        ((0, 127), CARGO_UP, 15, 30, 120),
        ((0, -127), CARGO_DOWN, 17, 45, 110),
    ] {
        let mut sim = throw_from_carry(Fx::from_int(-4), false, stick);
        assert_eq!(sim.f().move_id, id as u8, "{stick:?}");
        let before = sim.fighter(1).percent;
        let mut rises = Vec::new();
        let mut end = 0;
        // (The release's hitlag freezes the move: those frames are not in the FAF.)
        let mut frozen = 0;
        for t in 1..=80u16 {
            if sim.f().hitlag > 0 {
                frozen += 1;
            }
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).percent > before && rises.is_empty() {
                rises.push(sim.f().state_frame + 1);
            }
            if sim.f().state != S::Attack {
                end = t + 2 - frozen;
                break;
            }
        }
        assert!(frozen > 0, "the release has hitlag");
        assert_eq!(rises, vec![release], "{stick:?}");
        assert_eq!(end, faf, "{stick:?} FAF");
        assert_eq!(
            sim.fighter(1).percent - before,
            Fx::from_ratio(damage, 10) * sim.content.rules.damage_mult,
            "{stick:?}"
        );
        assert_eq!(sim.f().grab_with, NONE);
        if stick.0 < 0 {
            assert_eq!(sim.f().facing, -1, "the back throw turns round");
            assert!(sim.fighter(1).pos.x < sim.f().pos.x, "thrown behind");
        } else if stick.1 == 0 {
            assert!(
                sim.fighter(1).pos.x > sim.f().pos.x,
                "{stick:?} thrown ahead"
            );
        }
    }
}

#[test]
fn the_cargo_throws_work_in_the_air_and_the_down_one_stops_and_boosts() {
    let mut sim = throw_from_carry(Fx::from_int(8), true, (0, -127));
    assert_eq!(sim.f().move_id, CARGO_DOWN as u8);
    while sim.f().state_frame < 16 {
        sim.tick(inp(0, 0, 0));
    }
    // Move frame 16 (the reference's 17): stopped sideways, 1.5 of the reference's units up (0.1875 world).
    assert_eq!(sim.f().vel, Vec2::new(Fx::ZERO, Fx::from_ratio(3, 16)));
    assert_eq!(sim.f().grab_with, NONE, "released on 17");
    // The up throw in the air lets go on 15.
    let mut sim = throw_from_carry(Fx::from_int(8), true, (0, 127));
    let before = sim.fighter(1).percent;
    for _ in 0..14 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.fighter(1).percent > before);
    assert!(!sim.f().grounded());
}

#[test]
fn the_carrys_armor_withstands_up_to_80_units_and_a_stronger_hit_ends_it() {
    // A third fighter (a duelist) jabs the hauler from behind; its jab made weak or strong.
    for (bk, held) in [(20i16, true), (200, false)] {
        let mut sim = carrying_at(Fx::ZERO);
        sim.state.fighters[2].active = true;
        sim.stand(2, fx(-15, 10), 1);
        let jab = &mut sim.content.weapons[0].moves[MoveId::Jab as usize];
        for hb in &mut jab.hitboxes {
            hb.base_knockback = bk;
            hb.knockback_growth = 0;
        }
        let before = sim.f().percent;
        sim_core::fighter::begin_attack(&mut sim.state.fighters[2], MoveId::Jab);
        for _ in 0..12 {
            sim.tick(inp(0, 0, 0));
        }
        assert!(sim.f().percent > before, "bk {bk}: hit");
        if held {
            assert_eq!(sim.f().state, S::Carrying, "withstood");
            assert_eq!(sim.fighter(1).grab_with, 0);
        } else {
            assert_eq!(sim.f().state, S::Hitstun, "broken through");
            assert_eq!(sim.fighter(1).grab_with, NONE, "let go");
        }
    }
    // During a cargo throw there is no armor: the weak jab ends it.
    let mut sim = throw_from_carry(Fx::ZERO, false, (127, 0));
    sim.state.fighters[2].active = true;
    sim.stand(2, fx(-15, 10), 1);
    let jab = &mut sim.content.weapons[0].moves[MoveId::Jab as usize];
    for hb in &mut jab.hitboxes {
        hb.base_knockback = 20;
        hb.knockback_growth = 0;
        // (The jab's hitbox reaches the hauler at its middle.)
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[2], MoveId::Jab);
    for _ in 0..8 {
        sim.tick(inp(0, 0, 0));
    }
    assert_ne!(sim.f().state, S::Attack, "no armor in the throw");
    assert_eq!(sim.fighter(1).grab_with, NONE);
}

#[test]
fn a_carry_off_the_stage_rolls_back_and_replays_the_same() {
    // The hauler walks off the edge carrying the duelist, who mashes at random; a snapshot taken mid-carry and replayed gives the
    // same frames.
    // (At 150% the clock is long; the duelist struggles on one frame in four.)
    let mut sim = duel_at(Fx::from_int(8));
    sim.state.fighters[1].percent = Fx::from_int(150);
    grab(&mut sim, inp(0, 0, 0));
    lift(&mut sim);
    let mut rng = Rng::new(5);
    let inputs: Vec<[Input; MAX_FIGHTERS]> = (0..200)
        .map(|t| {
            let mut i = [Input::default(); MAX_FIGHTERS];
            i[0] = inp(127, 0, if t == 25 { JUMP } else { 0 });
            if t % 4 == 0 {
                i[1] = inp(
                    (rng.next_u32() % 255) as i8,
                    (rng.next_u32() % 255) as i8,
                    (rng.next_u32() & 0b1111) as u16,
                );
            }
            i
        })
        .collect();
    let mut saved = None;
    let mut airborne_carry = false;
    for (t, i) in inputs.iter().enumerate() {
        if t == 20 {
            assert_eq!(sim.state.fighters[0].state, S::Carrying);
            saved = Some(sim.state);
        }
        step(&mut sim.state, &sim.content, i);
        airborne_carry |=
            sim.state.fighters[0].state == S::Carrying && !sim.state.fighters[0].grounded();
    }
    assert!(airborne_carry, "the run carried off the stage");
    let mut b = saved.expect("saved");
    for i in &inputs[20..] {
        step(&mut b, &sim.content, i);
    }
    assert_eq!(sim.state.checksum(), b.checksum());
}
