//! The generic multi-hit and hit-property machinery (any moveset can use it): up to eight hit groups, a rehit rhythm that re-arms only
//! some groups (a looping phase and a final hit), grounded-only and aerial-only hitboxes, set knockback, hitstun modifiers, per-hit
//! SDI multipliers, the autolink angle (367) and landing hits. The reference's rules are in `docs/sheik/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::moves::{
    Hitbox, Move, MoveId, ANGLE_AUTOLINK, EFFECT_NORMAL, HIT_NORMAL, TARGETS_AERIAL, TARGETS_ALL,
    TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{Fx, Vec2};

/// A hitbox in front of the attacker, over a target standing 1.0 away: frames `start..=end` (sim frames), `damage` percent.
#[allow(clippy::too_many_arguments)]
fn hb(start: u8, end: u8, damage: i32, angle: i16, bkb: i16, kbg: i16, group: u8) -> Hitbox {
    Hitbox {
        start,
        end,
        x: fx(10, 10),
        y: fx(10, 10),
        radius: fx(9, 10),
        damage: Fx::from_int(damage),
        angle,
        base_knockback: bkb,
        knockback_growth: kbg,
        priority: 0,
        group,
        kind: HIT_NORMAL,
        shield_damage: 100,
        hitlag: 100,
        effect: EFFECT_NORMAL,
        targets: TARGETS_ALL,
        fixed_kb: 0,
        hitstun_add: 0,
        sdi: 100,
    }
}

/// Player 0 with `mv` as its neutral special (on the ground), player 1 standing 1.0 in front of it, the others out.
fn setup(mv: Move) -> Sim {
    let mut sim = Sim::new();
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.content.weapons[0].moves[MoveId::NSpecial as usize] = mv;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::ONE, -1);
    sim
}

fn start(sim: &mut Sim) {
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NSpecial);
}

/// `n` percent as dealt (the ruleset's damage multiplier applies).
fn dealt(sim: &Sim, n: i32) -> Fx {
    Fx::from_int(n) * sim.content.rules.damage_mult
}

fn mv(total: u8, hitboxes: Vec<Hitbox>) -> Move {
    Move {
        total_frames: total,
        hitboxes,
        ..Move::empty()
    }
}

#[test]
fn a_move_can_have_more_than_two_separate_hits() {
    // Three groups on frames 2, 10 and 18, each 1% and no knockback: three hits (two groups was the limit before).
    let mut sim = setup(mv(
        40,
        vec![
            hb(2, 2, 1, 0, 0, 0, 0),
            hb(10, 10, 1, 0, 0, 0, 1),
            hb(18, 18, 1, 0, 0, 0, 2),
        ],
    ));
    start(&mut sim);
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 3));
}

#[test]
fn the_rehit_rhythm_re_arms_the_loop_but_never_the_final_hit() {
    // A loop (group 0) on frames 2-13 re-armed every 4 frames, then a final hit (group 1) on frames 20-23. The rhythm runs on through
    // the final hit's frames, but only re-arms group 0, so the final hit lands once: 3 loop hits and 1 final.
    let build = |groups: u8| {
        let mut m = mv(
            40,
            vec![hb(2, 13, 1, 0, 0, 0, 0), hb(20, 23, 1, 0, 0, 0, 1)],
        );
        m.rehit = Some((2, 4));
        m.rehit_groups = groups;
        m
    };
    let mut sim = setup(build(0b1));
    start(&mut sim);
    sim.ticks(80, inp(0, 0, 0));
    assert_eq!(
        sim.fighter(1).percent,
        dealt(&sim, 4),
        "three loop hits and one final"
    );
    // Re-arming every group (the old behaviour) hits the final twice.
    let mut sim = setup(build(0xff));
    start(&mut sim);
    sim.ticks(80, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 5));
}

#[test]
fn grounded_only_and_aerial_only_hitboxes_pick_their_targets() {
    for (targets, airborne, hits) in [
        (TARGETS_GROUNDED, false, true),
        (TARGETS_GROUNDED, true, false),
        (TARGETS_AERIAL, false, false),
        (TARGETS_AERIAL, true, true),
    ] {
        let mut h = hb(2, 4, 3, 0, 0, 0, 0);
        h.targets = targets;
        let mut sim = setup(mv(30, vec![h]));
        if airborne {
            sim.put_airborne(1, Fx::ONE, fx(3, 10), Fx::ZERO, Fx::ZERO);
        }
        start(&mut sim);
        sim.ticks(4, inp(0, 0, 0));
        assert_eq!(
            sim.fighter(1).percent > Fx::ZERO,
            hits,
            "targets {targets}, airborne {airborne}"
        );
    }
}

/// Knockback a target at `percent` takes from `h` (read before its launch is applied).
fn launch_kb(h: Hitbox, percent: i32) -> Fx {
    let mut sim = setup(mv(30, vec![h]));
    sim.state.fighters[1].percent = Fx::from_int(percent);
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert!(sim.fighter(1).launch_pending || sim.fighter(1).state == S::Hitstun);
    sim.fighter(1).launch_kb
}

#[test]
fn set_knockback_is_the_same_at_any_percent() {
    let mut h = hb(2, 2, 3, 0, 0, 80, 0);
    h.fixed_kb = 30;
    assert_eq!(launch_kb(h, 0), launch_kb(h, 150));
    assert!(launch_kb(h, 0) > Fx::ZERO);
    // Without it the knockback grows with percent.
    h.fixed_kb = 0;
    assert!(launch_kb(h, 150) > launch_kb(h, 0));
}

#[test]
fn a_hitstun_modifier_adds_frames() {
    let stun = |add: u8| {
        let mut h = hb(2, 2, 5, 45, 40, 100, 0);
        h.hitstun_add = add;
        let mut sim = setup(mv(30, vec![h]));
        start(&mut sim);
        sim.ticks(3, inp(0, 0, 0));
        sim.fighter(1).hitstun
    };
    assert_eq!(stun(2), stun(0) + 2);
}

#[test]
fn a_hits_sdi_multiplier_scales_the_victims_nudge() {
    // A 10% hit (12 frames of hitlag); the victim flicks right once during it.
    let nudge = |sdi: u16| {
        // (Enough knockback to launch: survival DI applies to a hit that launches, not to a flinch.)
        let mut h = hb(2, 2, 10, 90, 40, 0, 0);
        h.sdi = sdi;
        let mut sim = setup(mv(30, vec![h]));
        start(&mut sim);
        sim.ticks(3, inp(0, 0, 0));
        assert!(sim.fighter(1).hitlag > 4);
        let x0 = sim.fighter(1).pos.x;
        sim.tick2(inp(0, 0, 0), inp(127, 0, 0));
        sim.fighter(1).pos.x - x0
    };
    let normal = nudge(100);
    assert!(normal > Fx::ZERO);
    assert_eq!(nudge(200), normal * Fx::from_int(2));
    assert_eq!(nudge(50), normal / Fx::from_int(2));
}

#[test]
fn autolink_sends_an_airborne_victim_with_the_attacker_and_toward_the_hitbox() {
    // The attacker falls past the victim with a 367 hitbox: when the hitlag ends the victim moves at the attacker's velocity plus a
    // fifth of the way to the hitbox, falling no faster than the cap. A grounded victim goes up at 80 degrees instead.
    let mut h = hb(1, 1, 1, ANGLE_AUTOLINK, 30, 100, 0);
    h.y = fx(5, 10);
    let mut sim = setup(mv(30, vec![h]));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(4), Fx::ZERO, -fx(2, 10));
    sim.put_airborne(1, fx(13, 10), Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    start(&mut sim);
    let mut n = 0;
    while !sim.fighter(1).launch_pending && n < 10 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    let victim = *sim.fighter(1);
    assert!(victim.link, "an airborne victim is linked");
    let attacker_vy = victim.link_vel.y;
    assert!(attacker_vy < Fx::ZERO, "the attacker was falling");
    while sim.fighter(1).hitlag > 0 {
        sim.tick(inp(0, 0, 0));
    }
    let v = sim.fighter(1).kb_vel;
    assert!(v.y < Fx::ZERO, "dragged downward: {v:?}");
    assert!(
        v.y >= -sim.content.rules.autolink_max_fall,
        "no faster than the cap: {v:?}"
    );
    assert!(
        v.x < Fx::ZERO,
        "pulled back toward the hitbox (in front of the attacker, behind the victim)"
    );
    assert!(v.length() <= sim.content.rules.autolink_max_speed + fx(1, 1000));
    assert!(!sim.fighter(1).tumble);

    // Grounded: an ordinary launch at 80 degrees, not a link.
    let mut sim = setup(mv(30, vec![hb(2, 2, 1, ANGLE_AUTOLINK, 30, 100, 0)]));
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert!(!sim.fighter(1).link);
    assert_eq!(
        sim.fighter(1).launch_angle,
        sim_core::trig::Angle::from_degrees(80).raw(),
        "80 degrees (the attacker faces right)"
    );
}

#[test]
fn survival_di_during_hitlag_changes_an_autolink_pull() {
    // The pull is worked out when hitlag ends, from where the victim is then: nudged away, the pull back is stronger.
    let pull = |flick: i8| {
        let h = hb(1, 1, 1, ANGLE_AUTOLINK, 30, 100, 0);
        let mut sim = setup(mv(30, vec![h]));
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(4), Fx::ZERO, Fx::ZERO);
        sim.put_airborne(1, fx(15, 10), Fx::from_int(4), Fx::ZERO, Fx::ZERO);
        start(&mut sim);
        while !sim.fighter(1).launch_pending {
            sim.tick(inp(0, 0, 0));
        }
        sim.tick2(inp(0, 0, 0), inp(flick, 0, 0));
        while sim.fighter(1).hitlag > 0 {
            sim.tick(inp(0, 0, 0));
        }
        sim.fighter(1).kb_vel.x
    };
    assert!(
        pull(127) < pull(0),
        "nudged further away, pulled back harder"
    );
}

#[test]
fn a_landing_hit_comes_out_on_a_real_landing_and_only_once() {
    // An aerial whose landing (outside its autocancel windows) starts a landing move with a 2% hitbox on its first frame.
    let build = |autocancel_all: bool| {
        let mut sim = Sim::new();
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        let mut air = mv(60, vec![]);
        air.landing_move = Some(MoveId::Ext0 as u8);
        air.landing_lag = 20;
        if autocancel_all {
            air.autocancel_before = 255;
        }
        let mut land = mv(20, vec![hb(0, 1, 2, 48, 77, 50, 0)]);
        land.hitboxes[0].y = fx(5, 10);
        sim.content.weapons[0].moves[MoveId::NAir as usize] = air;
        sim.content.weapons[0].moves[MoveId::Ext0 as usize] = land;
        sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, -fx(3, 10));
        sim.stand(1, Fx::ONE, -1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NAir);
        sim
    };
    let mut sim = build(false);
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 2), "one landing hit");
    // Autocancelled: a plain landing, no hit.
    let mut sim = build(true);
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
    let _ = Vec2::ZERO;
}
