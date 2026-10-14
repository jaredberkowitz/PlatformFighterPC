//! The generic multi-hit and hit-property machinery (any moveset can use it): up to eight hit groups, a rehit rhythm that re-arms only
//! some groups (a looping phase and a final hit), grounded-only and aerial-only hitboxes, set knockback, hitstun modifiers, per-hit
//! SDI multipliers, the autolink angle (367) and landing hits. The reference's rules are in `docs/sheik/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::moves::{
    Hitbox, Motion, Move, MoveId, AIM_ANY, AIM_DOWN, AIM_STRAIGHT, AIM_UP, ANGLE_AUTOLINK,
    ANGLE_BACKWARD, ANGLE_BY_BOX, ANGLE_BY_POSITION, ANGLE_BY_TRAVEL, ANGLE_FORWARD, EFFECT_NORMAL,
    HIT_NORMAL, TARGETS_AERIAL, TARGETS_ALL, TARGETS_GROUNDED,
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
        link_ahead: Fx::ZERO,
        aim: AIM_ANY,
        set_weight: false,
        reverse: false,
        push: false,
        shieldstun: 100,
        angle_type: ANGLE_BY_BOX,
        command: false,
        clanks: true,
        rebounds: true,
        trip: 0,
        half_height: Fx::ZERO,
        posvec_x: Fx::ZERO,
        posvec_y: Fx::ZERO,
        posvec_frames: 0,
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
    let stun = |add: i16| {
        let mut h = hb(2, 2, 5, 45, 40, 100, 0);
        h.hitstun_add = add;
        let mut sim = setup(mv(30, vec![h]));
        start(&mut sim);
        sim.ticks(3, inp(0, 0, 0));
        sim.fighter(1).hitstun
    };
    assert_eq!(stun(2), stun(0) + 2);
    // A negative modifier takes frames away (E27, sim 67: the reference's -13 on a late multi-hit), and a launch keeps one frame.
    assert_eq!(stun(-13), stun(0) - 13);
    assert!(stun(0) > 13);
    assert_eq!(stun(-500), 1);
}

/// Where player 1 ends up being sent (the sign of its launch speed) when player 0's one hitbox at `x` (relative to its facing) with
/// `angle_type` hits it standing at `victim_x`; player 0 faces right at 0. The angle is 0 degrees (straight along "forward").
fn sent_by(angle_type: u8, x: i32, victim_x: i32) -> i32 {
    let mut h = hb(2, 2, 10, 45, 80, 100, 0);
    h.x = fx(x, 10);
    h.radius = fx(15, 10);
    h.angle_type = angle_type;
    let mut sim = setup(mv(30, vec![h]));
    sim.stand(1, fx(victim_x, 10), -1);
    start(&mut sim);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Hitstun, "hit");
    let v = sim.fighter(1).kb_vel.x;
    if v > Fx::ZERO {
        1
    } else if v < Fx::ZERO {
        -1
    } else {
        0
    }
}

#[test]
fn angle_types_choose_which_way_forward_is() {
    // (E26, sim 67.) A box reaching both sides of the attacker, the victim in front (x 0.5) or behind (x -0.5).
    // By box (the default, as every move before sim 67): the box's own side decides, whoever it hits.
    assert_eq!(sent_by(ANGLE_BY_BOX, 2, 5), 1);
    assert_eq!(sent_by(ANGLE_BY_BOX, 2, -5), 1, "a front box sends forward");
    assert_eq!(
        sent_by(ANGLE_BY_BOX, -2, 5),
        -1,
        "a back box sends backward"
    );
    // Forward and backward ignore where the victim is.
    assert_eq!(sent_by(ANGLE_FORWARD, -2, -5), 1);
    assert_eq!(sent_by(ANGLE_FORWARD, 2, 5), 1);
    assert_eq!(sent_by(ANGLE_BACKWARD, 2, 5), -1);
    assert_eq!(sent_by(ANGLE_BACKWARD, -2, -5), -1);
    // By position: away from the attacker, whatever the box.
    assert_eq!(sent_by(ANGLE_BY_POSITION, 2, -5), -1);
    assert_eq!(sent_by(ANGLE_BY_POSITION, -2, 5), 1);
    assert_eq!(sent_by(ANGLE_BY_POSITION, 0, 5), 1);
    assert_eq!(sent_by(ANGLE_BY_POSITION, 0, -5), -1);
}

/// As [`sent_by`], with player 0 sliding at `vx` (world tenths a frame, relative to its facing) through the move.
fn sent_moving(angle_type: u8, vx: i32) -> i32 {
    let mut h = hb(2, 2, 10, 45, 80, 100, 0);
    h.radius = fx(15, 10);
    h.angle_type = angle_type;
    let mut m = mv(30, vec![h]);
    m.motion = vec![Motion {
        start: 0,
        end: 29,
        vx: fx(vx, 10),
        vy: Fx::ZERO,
    }];
    let mut sim = setup(m);
    start(&mut sim);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::Hitstun, "hit");
    sim.fighter(1).kb_vel.x.raw().signum()
}

#[test]
fn the_travel_angle_type_sends_the_way_the_attacker_is_moving() {
    // (Sim 73, owner decision 23: the reference's type 1, speed-based.) A box in front, the victim in front; the attacker facing
    // right. Sliding backward it sends the victim backward (the way it moves), sliding forward or standing still forward.
    assert_eq!(sent_moving(ANGLE_BY_TRAVEL, -1), -1);
    assert_eq!(sent_moving(ANGLE_BY_TRAVEL, 1), 1);
    assert_eq!(sent_moving(ANGLE_BY_TRAVEL, 0), 1, "not moving: its facing");
    // The default (by box) ignores the movement.
    assert_eq!(sent_moving(ANGLE_BY_BOX, -1), 1);
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

#[test]
fn an_aimed_hitbox_is_out_only_when_the_move_was_aimed_its_way() {
    // One 1% box for each aim and one for any aim: aimed up, the up box and the any box land (2%), and so on.
    let build = || {
        let mut m = mv(
            30,
            vec![
                hb(2, 2, 1, 0, 0, 0, 0),
                hb(2, 2, 1, 0, 0, 0, 1),
                hb(2, 2, 1, 0, 0, 0, 2),
                hb(2, 2, 1, 0, 0, 0, 3),
            ],
        );
        m.hitboxes[1].aim = AIM_UP;
        m.hitboxes[2].aim = AIM_STRAIGHT;
        m.hitboxes[3].aim = AIM_DOWN;
        assert_eq!(m.hitboxes[0].aim, AIM_ANY);
        m
    };
    for aim in [1i8, 0, -1] {
        let mut sim = setup(build());
        start(&mut sim);
        sim.state.fighters[0].aim = aim;
        sim.ticks(40, inp(0, 0, 0));
        assert_eq!(sim.fighter(1).percent, dealt(&sim, 2), "aim {aim}");
    }
    // A move started any other way is straight (the aim is reset as an attack begins).
    let mut sim = setup(build());
    sim.state.fighters[0].aim = 1;
    start(&mut sim);
    assert_eq!(sim.f().aim, 0);
}

#[test]
fn a_set_weight_hit_launches_light_and_heavy_fighters_alike() {
    // The same hit on a light and a heavy target: normally the light one goes further; with set weight both go the same.
    let launch = |victim: u8, set_weight: bool| {
        let mut sim = Sim::with_chars([0, victim, 0, 0]);
        sim.state.fighters[2].active = false;
        sim.state.fighters[3].active = false;
        let mut m = mv(30, vec![hb(2, 2, 5, 80, 40, 100, 0)]);
        m.hitboxes[0].set_weight = set_weight;
        sim.content.weapons[0].moves[MoveId::NSpecial as usize] = m;
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::ONE, -1);
        sim.state.fighters[1].percent = Fx::from_int(60);
        start(&mut sim);
        for _ in 0..30 {
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).percent > Fx::from_int(60) && !sim.fighter(1).launch_pending {
                return sim.fighter(1).kb_vel.length();
            }
        }
        panic!("no hit");
    };
    // (The rogue weighs 78, the brawler 92.)
    assert!(launch(2, false) > launch(1, false));
    assert!((launch(2, true) - launch(1, true)).abs() <= Fx::from_raw(64));
}

#[test]
fn a_set_knockback_launch_keeps_the_plain_launch_physics() {
    // The reference: launches of over 32 frames of hitstun are sped up at first, except from set knockback, which also keeps the
    // fighter's own fall speed on a vertical launch. (Sim 59.)
    let launch = |fixed_kb: u16| {
        let mut h = hb(2, 2, 5, 50, 0, 100, 0);
        h.fixed_kb = fixed_kb;
        if fixed_kb == 0 {
            // About the same knockback without set knockback.
            h.base_knockback = 120;
        }
        let mut sim = setup(mv(30, vec![h]));
        start(&mut sim);
        for _ in 0..30 {
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
                break;
            }
        }
        (
            sim.fighter(1).launch_kb,
            sim.fighter(1).hitstun,
            sim.fighter(1).balloon_faf,
            sim.fighter(1).launch_set,
        )
    };
    let (kb, stun, balloon, set) = launch(150);
    assert!(set);
    assert!(stun > 32, "a long hitstun: {stun} from {kb:?}");
    assert_eq!(balloon, 0, "no speed-up");
    let (_, stun, balloon, set) = launch(0);
    assert!(!set && stun > 32);
    assert!(balloon > 0, "a normal launch of that strength is sped up");
}

#[test]
fn a_reversing_hit_turns_the_victim_round() {
    // The victim faces the attacker; a reversing hit leaves it facing away, a plain one facing it. (Sim 61.)
    for (reverse, facing) in [(false, -1), (true, 1)] {
        let mut h = hb(2, 2, 5, 80, 30, 50, 0);
        h.reverse = reverse;
        let mut sim = setup(mv(30, vec![h]));
        start(&mut sim);
        sim.ticks(3, inp(0, 0, 0));
        assert!(sim.fighter(1).percent > Fx::ZERO);
        assert_eq!(sim.fighter(1).facing, facing, "reverse {reverse}");
    }
    // Even a flinch (no hitstun) turns it round.
    let mut h = hb(2, 2, 1, 80, 0, 0, 0);
    h.reverse = true;
    let mut sim = setup(mv(30, vec![h]));
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).hitstun, 0);
    assert_eq!(sim.fighter(1).facing, 1);
}

#[test]
fn a_push_shoves_without_hitlag_or_hitstun() {
    // A 0% push at 0 degrees with set knockback 70: the victim keeps its state and is moving away; nobody is frozen. (Sim 61.)
    let mut h = hb(2, 2, 0, 0, 0, 100, 0);
    h.push = true;
    h.fixed_kb = 70;
    let mut sim = setup(mv(30, vec![h]));
    sim.put_airborne(1, Fx::ONE, fx(5, 10), Fx::ZERO, Fx::ZERO);
    start(&mut sim);
    sim.ticks(1, inp(0, 0, 0));
    let before = sim.fighter(1).vel.x;
    sim.tick(inp(0, 0, 0));
    let v = sim.fighter(1);
    assert_eq!(v.state, S::Airborne, "no hitstun");
    assert_eq!((v.hitlag, sim.fighter(0).hitlag), (0, 0), "no hitlag");
    let kb = sim_core::combat::fixed_knockback(70, sim.content.fighters[0].weight, 0, 100);
    let shove = kb * Fx::from_ratio(3, 800);
    assert!(
        (v.vel.x - before - shove).abs() < Fx::from_ratio(1, 50),
        "pushed by the knockback's speed: {:?} vs {:?}",
        v.vel.x - before,
        shove
    );
    // On the ground only its sideways part applies, and a shield is pushed back too.
    let mut h = hb(2, 2, 0, 45, 0, 100, 0);
    h.push = true;
    h.fixed_kb = 70;
    let mut sim = setup(mv(30, vec![h]));
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert!(sim.fighter(1).grounded() && sim.fighter(1).vel.x > Fx::ZERO);
    let mut sim = setup(mv(30, vec![h]));
    sim.state.fighters[1].state = S::Shield;
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert!(
        sim.fighter(1).vel.x > fx(1, 10),
        "{:?}",
        sim.fighter(1).vel.x
    );
}

// ---- Armor (E1) ------------------------------------------------------------------------------------------------------------------

/// Player 1 in a 60-frame move with `armor`, struck on frame 3 of player 0's hit of `damage` percent (BK 80: it would launch).
fn armored(armor: Option<sim_core::moves::Armor>, damage: i32, kind: u8) -> Sim {
    let mut h = hb(2, 4, damage, 45, 80, 100, 0);
    h.kind = kind;
    let mut sim = setup(mv(30, vec![h]));
    let mut stance = mv(60, Vec::new());
    stance.armor = armor;
    sim.content.weapons[1].moves[MoveId::NSpecial as usize] = stance;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::NSpecial);
    start(&mut sim);
    sim.ticks(4, inp(0, 0, 0));
    sim
}

#[test]
fn damage_armor_withstands_hits_up_to_its_threshold_and_super_armor_every_hit() {
    use sim_core::moves::{Armor, ARMOR_DAMAGE, ARMOR_SUPER, HIT_GRAB};
    let damage = |start: u8, threshold: i32| {
        Some(Armor {
            start,
            end: 30,
            kind: ARMOR_DAMAGE,
            threshold: Fx::from_int(threshold),
            subtract: false,
            min_kb: Fx::ZERO,
        })
    };
    // 9% and 10% against a 10% threshold: the damage and hitlag are taken, the move goes on.
    for d in [9, 10] {
        let sim = armored(damage(0, 10), d, HIT_NORMAL);
        let f = sim.fighter(1);
        assert_eq!(
            (f.state, f.move_id),
            (S::Attack, MoveId::NSpecial as u8),
            "{d}%"
        );
        assert_eq!(f.percent, dealt(&sim, d));
        assert!(f.hitlag > 0, "an armored hit still freezes");
        assert!(!f.launch_pending);
    }
    // 11% breaks it: hitstun.
    assert_eq!(
        armored(damage(0, 10), 11, HIT_NORMAL).fighter(1).state,
        S::Hitstun
    );
    // Outside the window the armor is gone, and without armor the 9% hit launches.
    assert_eq!(
        armored(damage(10, 10), 9, HIT_NORMAL).fighter(1).state,
        S::Hitstun
    );
    assert_eq!(armored(None, 9, HIT_NORMAL).fighter(1).state, S::Hitstun);
    // Super armor takes anything.
    let sup = Some(Armor {
        start: 0,
        end: 30,
        kind: ARMOR_SUPER,
        threshold: Fx::ZERO,
        subtract: false,
        min_kb: Fx::ZERO,
    });
    assert_eq!(armored(sup, 30, HIT_NORMAL).fighter(1).state, S::Attack);
    // A grab ignores armor.
    assert_eq!(armored(sup, 0, HIT_GRAB).fighter(1).state, S::Grabbed);
}

#[test]
fn knockback_armor_withstands_up_to_its_threshold_and_a_subtractive_one_takes_it_off() {
    use sim_core::moves::{Armor, ARMOR_KNOCKBACK, HIT_GRAB};
    let kind = |threshold: Fx, subtract: bool, min_kb: Fx| {
        Some(Armor {
            start: 0,
            end: 30,
            kind: ARMOR_KNOCKBACK,
            threshold,
            subtract,
            min_kb,
        })
    };
    // The hit's knockback without armor.
    let plain = armored(None, 12, HIT_NORMAL);
    assert!(plain.fighter(1).launch_pending);
    let k = plain.fighter(1).launch_kb;
    assert!(k > Fx::from_int(20));
    // At most the threshold: damage and hitlag only, the move goes on.
    let held = armored(kind(k, false, Fx::ZERO), 12, HIT_NORMAL);
    let f = held.fighter(1);
    assert_eq!((f.state, f.move_id), (S::Attack, MoveId::NSpecial as u8));
    assert_eq!(f.percent, plain.fighter(1).percent);
    assert!(f.hitlag > 0 && !f.launch_pending);
    // Above it, not subtractive: the whole knockback.
    let t = k - Fx::from_int(10);
    let broken = armored(kind(t, false, Fx::ZERO), 12, HIT_NORMAL);
    assert_eq!(broken.fighter(1).state, S::Hitstun);
    assert_eq!(broken.fighter(1).launch_kb, k);
    // Subtractive: the knockback less the threshold, and at least the minimum.
    let sub = armored(kind(t, true, Fx::ZERO), 12, HIT_NORMAL);
    assert_eq!(sub.fighter(1).launch_kb, Fx::from_int(10));
    let floor = armored(kind(t, true, Fx::from_int(15)), 12, HIT_NORMAL);
    assert_eq!(floor.fighter(1).launch_kb, Fx::from_int(15));
    // Outside its window it does nothing.
    let mut late = kind(t, true, Fx::ZERO);
    if let Some(a) = late.as_mut() {
        a.start = 10;
    }
    assert_eq!(armored(late, 12, HIT_NORMAL).fighter(1).launch_kb, k);
    // A grab ignores it.
    assert_eq!(
        armored(kind(Fx::from_int(500), true, Fx::ZERO), 0, HIT_GRAB)
            .fighter(1)
            .state,
        S::Grabbed
    );
}

// ---- Search boxes (E21) ----------------------------------------------------------------------------------------------------------

#[test]
fn a_search_box_touches_without_hitting_or_clanking_and_switches_the_move_next_frame() {
    use sim_core::moves::HIT_SEARCH;
    let mut probe = hb(2, 10, 0, 0, 0, 0, 0);
    probe.kind = HIT_SEARCH;
    let mut seek = mv(30, vec![probe]);
    seek.on_search = Some(MoveId::Ext9 as u8);
    let build = |opponent_attacks: bool| {
        let mut sim = setup(seek.clone());
        sim.content.weapons[0].moves[MoveId::Ext9 as usize] = mv(20, vec![hb(3, 3, 5, 0, 0, 0, 0)]);
        if opponent_attacks {
            // The opponent's own attack overlaps the search box: a normal hit would clank, a search box never does.
            sim.content.weapons[1].moves[MoveId::NSpecial as usize] =
                mv(30, vec![hb(1, 20, 0, 0, 0, 0, 0)]);
            sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::NSpecial);
        }
        sim.state.fighters[0].vel.x = Fx::from_ratio(1, 10);
        start(&mut sim);
        sim
    };
    for opponent_attacks in [false, true] {
        let mut sim = build(opponent_attacks);
        sim.ticks(2, inp(0, 0, 0));
        // Frame 2: the search box touches; nothing happens to anyone this frame.
        assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
        assert_eq!(sim.f().next_move, MoveId::Ext9 as u8 + 1);
        assert_eq!((sim.f().hitlag, sim.fighter(1).hitlag), (0, 0));
        assert_eq!(sim.fighter(1).percent, Fx::ZERO);
        assert_ne!(sim.f().state, S::Rebound, "no clank");
        // Next frame: the new move, on its first frame.
        sim.tick(inp(0, 0, 0));
        assert_eq!(
            (sim.f().move_id, sim.f().state_frame),
            (MoveId::Ext9 as u8, 0)
        );
        // Without on_search nothing switches.
    }
    let mut sim = build(false);
    sim.content.weapons[0].moves[MoveId::NSpecial as usize].on_search = None;
    sim.ticks(4, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
}
