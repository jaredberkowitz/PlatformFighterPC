//! The Popgun's side special, the monkey flip (class 11, the scamp body), modelled mechanically on a reference character: the leap
//! (ground and air), its command grab, the latch (E49: a ride on the caught fighter with the ordinary hold's clock and mash-out, a
//! repeating scratch), its attack and jump exits, the kick on a second press (which bars the up special until landing), the whiff and
//! the ledges. Reference data: the twelfth moveset's `REFERENCE_DATA.md` (`docs/movesets/diddy_kong/`).

mod common;

use common::{inp, Sim};
use sim_core::combat::params_of;
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use sim_core::moves::popgun_slots::{
    FLIP_AIR, FLIP_KICK, LATCH, LATCH_ATTACK, LATCH_JUMP, ROCKET_AGAIN,
};
use sim_core::moves::{MoveId, Weapon, HIT_GRAB, HIT_NORMAL, HIT_PUMMEL, HIT_THROW};
use sim_core::state::{FighterState as S, NONE};
use sim_core::{step, Fx, Input, MAX_FIGHTERS};

const SCAMP: u8 = 11;

/// The scamp (player 0) at 0 facing right and a duelist (player 1) standing at `gap` facing it, the others out of the match.
fn setup(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars([SCAMP, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

fn weapon(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(SCAMP)].weapon)]
}

/// Side special pressed on frame 1 (the stick forward), then `rest` every frame after; returns the frame the latch began, if it did.
fn flip_until_latch(sim: &mut Sim, frames: u32, rest: Input) -> Option<u32> {
    for frame in 1..=frames {
        sim.tick(if frame == 1 {
            inp(127, 0, SPECIAL)
        } else {
            rest
        });
        if sim.f().move_id == LATCH as u8 && sim.f().state == S::Attack {
            return Some(frame);
        }
    }
    None
}

/// A scamp latched onto the duelist standing 6 ahead (on frame 22 of the flip).
fn latched() -> Sim {
    let mut sim = setup(Fx::from_int(6));
    let at = flip_until_latch(&mut sim, 40, inp(0, 0, 0)).expect("latched");
    // (Caught in the grab's frames 18-36, the latch from the next frame.)
    assert!((19..=38).contains(&at), "{at}");
    sim
}

#[test]
fn the_flips_parts_have_the_references_frames_and_hits() {
    let sim = setup(Fx::from_int(6));
    let w = weapon(&sim);
    // The leap: the grab on 18-36 from the ground, 18-29 in the air (move frames one less), command grabs; FAF 62 / 48 (total frames two
    // less), the air version's landing lag 26 [R].
    let ground = w.get(MoveId::SideSpecial as u8);
    let air = w.get(FLIP_AIR as u8);
    for (mv, last, total, lag) in [(ground, 35u8, 60u8, 0u8), (air, 28, 46, 26)] {
        assert_eq!(mv.total_frames, total);
        assert_eq!(mv.landing_lag, lag);
        assert_eq!(mv.catch_move, Some(LATCH as u8));
        assert!(!mv.helpless_after, "not helpless after it");
        assert!(!mv.hitboxes.is_empty());
        for h in &mv.hitboxes {
            assert_eq!(
                (h.kind, h.command, h.start, h.end),
                (HIT_GRAB, true, 17, last)
            );
        }
    }
    assert_eq!(ground.air_version, Some(FLIP_AIR as u8));
    // The kick: clean 6-10 14% / 12%, late 11-25 12% / 10%, 361, BK 80, KS 50; FAF 33, landing lag 18 [R].
    let kick = w.get(FLIP_KICK as u8);
    assert_eq!((kick.total_frames, kick.landing_lag), (31, 18));
    let hits: Vec<(u8, u8, Fx)> = kick
        .hitboxes
        .iter()
        .map(|h| (h.start, h.end, h.damage))
        .collect();
    assert_eq!(
        hits,
        vec![
            (5, 9, Fx::from_int(14)),
            (5, 9, Fx::from_int(12)),
            (10, 24, Fx::from_int(12)),
            (10, 24, Fx::from_int(10)),
        ]
    );
    assert!(kick
        .hitboxes
        .iter()
        .all(|h| (h.angle, h.base_knockback, h.knockback_growth) == (361, 80, 50)));
    // The latch: a 1% scratch (361, BK 0, KS 0) on its frames 11 and 29 of every 48, the ordinary hold's clock.
    let latch = w.get(LATCH as u8);
    let l = latch.latch.expect("a latch");
    assert_eq!((l.clock_num, l.clock_den), (1, 1));
    let scratches: Vec<(u8, u8, Fx, u8)> = latch
        .hitboxes
        .iter()
        .map(|h| (h.kind, h.start, h.damage, h.group))
        .collect();
    assert_eq!(
        scratches,
        vec![(HIT_PUMMEL, 10, Fx::ONE, 0), (HIT_PUMMEL, 28, Fx::ONE, 1)]
    );
    // Its attack: 3% on 18, the throw 7% (361, BK 50, KS 100) on 20, FAF 37; its jump: 2% (361, BK 0, KS 50) on 18 holding a fighter
    // it does not tumble 20 frames, FAF 43 [R]. Neither has a clock.
    let attack = w.get(LATCH_ATTACK as u8);
    assert_eq!(attack.total_frames, 35);
    assert_eq!(attack.latch.map(|l| l.clock_num), Some(0));
    let throw = attack
        .hitboxes
        .iter()
        .find(|h| h.kind == HIT_THROW)
        .expect("throw");
    assert_eq!(
        (
            throw.start,
            throw.damage,
            throw.angle,
            throw.base_knockback,
            throw.knockback_growth
        ),
        (19, Fx::from_int(7), 361, 50, 100)
    );
    assert!(attack
        .hitboxes
        .iter()
        .any(|h| h.kind == HIT_PUMMEL && h.start == 17 && h.damage == Fx::from_int(3)));
    assert!(attack
        .hitboxes
        .iter()
        .any(|h| h.kind == HIT_NORMAL && (h.base_knockback, h.knockback_growth) == (60, 150)));
    let jump = w.get(LATCH_JUMP as u8);
    assert_eq!(jump.total_frames, 41);
    let kickoff = jump.hitboxes[0];
    assert_eq!(
        (
            kickoff.kind,
            kickoff.start,
            kickoff.damage,
            kickoff.base_knockback,
            kickoff.knockback_growth,
            kickoff.fixed_stun
        ),
        (HIT_THROW, 17, Fx::from_int(2), 0, 50, 20)
    );
}

#[test]
fn the_flip_catches_a_standing_fighter_and_rides_its_head_scratching_on_11_and_29() {
    let mut sim = latched();
    let d = sim.fighter(1);
    assert_eq!(d.state, S::Grabbed);
    assert_eq!(sim.f().grab_with, 1);
    // On its head: straight over its feet, 0.9 of its height up, in the air.
    let p = params_of(&sim.content, sim.fighter(1));
    let up = p.ecb_height * Fx::from_ratio(90, 100);
    assert_eq!(sim.f().pos, d.pos + sim_core::Vec2::new(Fx::ZERO, up));
    assert_eq!(sim.f().platform, NONE);
    // The scratches: 1% (1.2 with the one-on-one multiplier) on the latch's frames 11 and 29, then again 48 frames on.
    let mut hits = Vec::new();
    let mut last = sim.fighter(1).percent;
    for frame in 1..=110u32 {
        sim.tick(inp(0, 0, 0));
        let now = sim.fighter(1).percent;
        if now != last {
            hits.push((frame, now - last));
            last = now;
        }
    }
    let damage = Fx::ONE * sim.content.rules.damage_mult;
    assert_eq!(hits.len(), 4, "{hits:?}");
    assert!(hits.iter().all(|(_, d)| *d == damage));
    // (Each scratch's hitlag holds the latch, so the later ones come that many frames later.)
    let lag = u32::from(sim_core::combat::hitlag_frames(
        Fx::ONE,
        &sim.content.rules,
        false,
        false,
        Fx::ONE,
    ));
    // (The latch's frame 1, the reference's numbering, is the frame it began: its 11th is 10 frames on.)
    assert_eq!(hits[0].0, 10);
    assert_eq!(hits[1].0, 28 + lag);
    assert_eq!(hits[2].0, 10 + 48 + 2 * lag);
    assert_eq!(hits[3].0, 28 + 48 + 3 * lag);
}

#[test]
fn the_latch_runs_the_ordinary_hold_clock_and_mashing_cuts_it() {
    // Unmashed at 0%: the hold time (90 frames) and the latch lets go, the scamp falling free and the duelist standing.
    let mut sim = latched();
    let mut frames = 0;
    while sim.fighter(1).state == S::Grabbed && frames < 400 {
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    let unmashed = frames;
    assert!(sim.fighter(1).state != S::Grabbed);
    assert_eq!(sim.f().grab_with, NONE);
    assert_eq!(sim.f().state, S::Airborne);
    // (90 frames of clock, the catch frame's tick already taken, and the scratches' hitlag on top.)
    assert!((95..=120).contains(&unmashed), "{unmashed}");
    // Mashed: much sooner.
    let mut sim = latched();
    let mut frames = 0;
    while sim.fighter(1).state == S::Grabbed && frames < 400 {
        let b = if frames % 2 == 0 { ATTACK } else { 0 };
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
        frames += 1;
    }
    assert!(frames * 2 < unmashed, "{frames} vs {unmashed}");
}

#[test]
fn attack_scratches_and_throws_for_3_and_7_and_the_scamp_flips_off_free() {
    let mut sim = latched();
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().move_id, LATCH_ATTACK as u8);
    let mut launched_at = 0;
    for frame in 2..=60u32 {
        sim.tick(inp(0, 0, 0));
        if launched_at == 0 && sim.fighter(1).state == S::Hitstun {
            launched_at = frame;
        }
    }
    let m = sim.content.rules.damage_mult;
    assert_eq!(sim.fighter(1).percent - before, Fx::from_int(10) * m);
    assert!(launched_at > 0, "the throw launches it");
    assert_eq!(sim.f().grab_with, NONE);
    assert!(matches!(sim.f().state, S::Airborne | S::Idle | S::Landing));
}

#[test]
fn jump_kicks_off_for_2_holding_the_fighter_20_frames_and_the_scamp_jumps_up() {
    let mut sim = latched();
    let before = sim.fighter(1).percent;
    let y0 = sim.f().pos.y;
    // (Jump was not held at the catch: a press now is the jump off.)
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().move_id, LATCH_JUMP as u8);
    // (Jump held from before the flip is no press: only a new one jumps off.)
    let mut held = setup(Fx::from_int(6));
    flip_until_latch(&mut held, 40, inp(0, 0, JUMP)).expect("latched");
    for _ in 0..5 {
        held.tick(inp(0, 0, JUMP));
    }
    assert_eq!(held.f().move_id, LATCH as u8);
    let mut stun = 0;
    let mut highest = y0;
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        highest = highest.max(sim.f().pos.y);
        if sim.fighter(1).state == S::Hitstun && stun == 0 {
            stun = sim.fighter(1).hitstun;
        }
    }
    let m = sim.content.rules.damage_mult;
    assert_eq!(sim.fighter(1).percent - before, Fx::from_int(2) * m);
    // (Hitstun is counted down from 20 on the frame it began.)
    assert!((19..=20).contains(&stun), "{stun}");
    assert!(
        highest > y0 + Fx::from_int(2),
        "the scamp jumps off the head"
    );
    assert_eq!(sim.f().state, S::Airborne, "free, not helpless");
}

#[test]
fn a_fighter_caught_in_the_air_falls_with_the_scamp_on_it() {
    let mut sim = setup(Fx::from_int(10));
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..16 {
        sim.tick(inp(0, 0, 0));
    }
    // The duelist in the air just ahead of the flipping scamp as its grab comes out, off any platform.
    let at = sim.f().pos + sim_core::Vec2::new(Fx::ONE, Fx::from_ratio(3, 10));
    sim.put_airborne(1, at.x, at.y, Fx::ZERO, Fx::ZERO);
    for _ in 0..3 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.f().move_id,
        LATCH as u8,
        "a command grab catches an airborne fighter"
    );
    let y = sim.fighter(1).pos.y;
    assert!(y > Fx::ZERO && sim.fighter(1).platform == NONE);
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
    }
    // It fell (to the stage) and the scamp still rides it.
    assert!(sim.fighter(1).pos.y < y);
    assert_eq!(sim.fighter(1).state, S::Grabbed);
    assert_eq!(sim.f().pos.x, sim.fighter(1).pos.x);
    assert!(sim.f().pos.y > sim.fighter(1).pos.y);
}

#[test]
fn latched_offstage_both_fall_until_the_fighter_is_knocked_out_and_the_scamp_after_it() {
    // Past the stage's right edge, the duelist in the air: the scamp latches on and rides it down (the reference's latch offstage).
    let edge = sim_core::Content::placeholder().stage.platforms[0].right;
    let mut sim = setup(Fx::from_int(-10));
    sim.stand(0, edge - Fx::from_int(3), 1);
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..16 {
        sim.tick(inp(0, 0, 0));
    }
    let at = sim.f().pos + sim_core::Vec2::new(Fx::ONE, Fx::from_ratio(3, 10));
    sim.put_airborne(1, at.x.max(edge + Fx::ONE), at.y, Fx::ZERO, Fx::ZERO);
    let (stocks0, stocks1) = (sim.f().stocks, sim.fighter(1).stocks);
    let mut caught = false;
    let mut duelist_out = 0;
    for frame in 1..=400u32 {
        sim.tick(inp(0, 0, 0));
        caught |= sim.fighter(1).state == S::Grabbed;
        if duelist_out == 0 && sim.fighter(1).stocks < stocks1 {
            duelist_out = frame;
        }
    }
    assert!(caught);
    assert!(duelist_out > 0, "the duelist fell to its knock-out");
    assert!(sim.f().stocks < stocks0, "and the scamp with it");
}

#[test]
fn the_leap_catches_a_ledge_in_front_once_falling() {
    // Below the stage's right ledge, facing it: the flip rises and, coming down, catches the ledge.
    let edge = sim_core::Content::placeholder().stage.platforms[0].right;
    let mut sim = setup(Fx::from_int(-10));
    sim.put_airborne(
        0,
        edge + Fx::from_int(2),
        -Fx::from_ratio(3, 2),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = -1;
    sim.tick(inp(-127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, FLIP_AIR as u8);
    let mut hung = false;
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        hung |= sim.f().state == S::LedgeHang;
    }
    assert!(hung, "it caught the ledge");
}

#[test]
fn the_flip_catches_a_shielding_fighter() {
    let mut sim = setup(Fx::from_int(6));
    for frame in 1..=40u32 {
        sim.tick2(
            if frame == 1 {
                inp(127, 0, SPECIAL)
            } else {
                inp(0, 0, 0)
            },
            inp(0, 0, SHIELD),
        );
    }
    assert_eq!(sim.fighter(1).state, S::Grabbed);
}

#[test]
fn a_whiffed_flip_lands_and_ends_on_62_from_the_ground_and_48_free_in_the_air() {
    let mut sim = setup(Fx::from_int(-10));
    sim.stand(0, Fx::from_int(-8), 1);
    let mut frames = 0;
    sim.tick(inp(127, 0, SPECIAL));
    while sim.f().state == S::Attack && frames < 100 {
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    // The move's last frame is its 61st (FAF 62).
    assert_eq!(frames + 1, 61);
    assert!(sim.f().pos.x > Fx::from_int(-3), "it travels");
    // In the air (high up): FAF 48, then falling free (not helpless).
    let mut sim = setup(Fx::from_int(10));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, FLIP_AIR as u8);
    let mut frames = 0;
    while sim.f().state == S::Attack && frames < 100 {
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    assert_eq!(frames + 1, 47);
    assert_eq!(sim.f().state, S::Airborne);
    // Landing during the aerial flip costs 26 frames.
    let mut sim = setup(Fx::from_int(10));
    sim.put_airborne(0, Fx::ZERO, Fx::from_ratio(1, 2), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..40 {
        if sim.f().state != S::Attack {
            break;
        }
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 26);
}

#[test]
fn a_smash_input_flips_further() {
    let run = |flick: bool| {
        let mut sim = setup(Fx::from_int(10));
        if flick {
            // (A flick: the stick from neutral to the rim in one frame.)
            sim.tick(inp(0, 0, 0));
        } else {
            // (Held there a while: no flick.)
            for _ in 0..12 {
                sim.tick(inp(127, 0, 0));
            }
            sim.stand(0, Fx::ZERO, 1);
        }
        sim.tick(inp(127, 0, SPECIAL));
        assert_eq!(sim.f().special_flick, flick);
        for _ in 0..30 {
            sim.tick(inp(0, 0, 0));
        }
        sim.f().pos.x
    };
    assert!(run(true) > run(false));
}

#[test]
fn a_second_press_kicks_for_14_and_bars_the_up_special_until_landing() {
    let mut sim = setup(Fx::from_ratio(35, 10));
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..13 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, FLIP_KICK as u8);
    let before = sim.fighter(1).percent;
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    let taken = sim.fighter(1).percent - before;
    let m = sim.content.rules.damage_mult;
    assert!(
        taken == Fx::from_int(14) * m || taken == Fx::from_int(12) * m,
        "the clean kick: {taken:?}"
    );
    // In the air after a kick the up special does nothing until the scamp lands.
    let mut sim = setup(Fx::from_int(10));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..13 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, FLIP_KICK as u8);
    while sim.f().state == S::Attack {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Airborne);
    let facing = sim.f().facing;
    sim.tick(inp(-127, 127, SPECIAL));
    assert_eq!(sim.f().state, S::Airborne, "barred: nothing starts");
    assert_eq!(sim.f().facing, facing, "and nothing turns");
    // Landed, the up special is back.
    sim.stand(0, Fx::ZERO, 1);
    sim.state.fighters[0].air_flags = 0;
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
}

#[test]
fn the_kick_bar_is_cleared_by_landing() {
    let mut sim = setup(Fx::from_int(10));
    sim.put_airborne(0, Fx::from_int(-10), Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(127, 0, SPECIAL));
    for _ in 0..13 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, 0, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, FLIP_KICK as u8);
    assert_ne!(sim.f().air_flags, 0);
    for _ in 0..120 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_flags, 0);
}

#[test]
fn only_the_first_rocket_charge_of_an_air_time_falls_slowly() {
    // Owner decision 75: the first charge in the air marks the air time; a hit does not clear the mark, so a second charge after
    // being knocked out of the first (hit, then free again in the air) is the plain-fall version; landing clears it.
    let mut sim = setup(Fx::from_int(10));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
    sim.tick(inp(0, 0, SPECIAL));
    assert_ne!(sim.f().air_marks, 0);
    // (As a hit leaves it: the air flags cleared, the marks kept, free in the air.)
    let marks = sim.f().air_marks;
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].air_flags = 0;
    assert_eq!(sim.f().air_marks, marks);
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 127, SPECIAL));
    assert_eq!(sim.f().move_id, ROCKET_AGAIN as u8);
    for _ in 0..30 {
        sim.tick(inp(0, 0, SPECIAL));
    }
    let p = params_of(&sim.content, sim.f());
    assert!(
        sim.f().vel.y < -p.max_fall_speed / Fx::from_int(2),
        "the second charge falls at the body's speed"
    );
    // A real hit keeps the mark: a duelist's jab on a marked scamp in the air.
    let mut sim = setup(Fx::from_int(1));
    sim.state.fighters[0].air_marks = 1;
    sim.put_airborne(0, Fx::ZERO, Fx::from_ratio(1, 2), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].facing = -1;
    sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
    for _ in 0..4 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.f().air_marks, 1);
    // Landing clears it.
    let mut sim = setup(Fx::from_int(10));
    sim.state.fighters[0].air_marks = 1;
    sim.put_airborne(0, Fx::ZERO, Fx::from_ratio(1, 4), Fx::ZERO, Fx::ZERO);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_marks, 0);
}

#[test]
fn the_latch_resimulates_exactly_from_a_snapshot() {
    let mut sim = latched();
    let snap = sim.state;
    let mut inputs = Vec::new();
    for k in 0..90u32 {
        let mut i = [Input::default(); MAX_FIGHTERS];
        i[1] = inp(0, 0, if k % 3 == 0 { ATTACK } else { 0 });
        if k == 40 {
            i[0] = inp(0, 0, JUMP);
        }
        inputs.push(i);
    }
    for i in &inputs {
        step(&mut sim.state, &sim.content, i);
    }
    let straight = sim.state.checksum();
    let mut again = snap;
    for i in &inputs {
        step(&mut again, &sim.content, i);
    }
    assert_eq!(again.checksum(), straight);
}
