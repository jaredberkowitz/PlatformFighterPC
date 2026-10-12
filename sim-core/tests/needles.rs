//! The Needles moveset on the rogue body (class 2), modelled mechanically on the reference game's Sheik: frame data against the
//! reference, the up air's looping autolink hits and final hit (and its drag-down), the down air's dive and landing hit, the up tilt's
//! two hits, and the body's jump air times; then the rest of the kit (the jabs and rapid jab, the earlier game's dash attack, the two-hit
//! smashes, grabs and throws) and light hits landing on their feet. Reference data and sources: `docs/sheik/REFERENCE_DATA.md`.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use sim_core::moves::{MoveId, ANGLE_AUTOLINK, TARGETS_AERIAL, TARGETS_GROUNDED};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const ROGUE: u8 = 2;

/// The rogue (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([ROGUE, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

fn needles(sim: &Sim) -> &sim_core::moves::Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(ROGUE)].weapon)]
}

/// Rising out of a jump, about a full hop's speed partway up.
const RISING: Fx = Fx::from_raw(65536 / 5);

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// A move's reference frame data: the move, its active windows [first, last], FAF, landing lag, autocancel before / from.
type FrameData = (MoveId, &'static [(u8, u8)], u8, u8, u8, u8);

#[test]
fn the_priority_moves_have_the_references_frame_data() {
    // (move, reference active windows [first, last], FAF, landing lag, autocancel before / from)
    let table: [FrameData; 8] = [
        (MoveId::FTilt, &[(5, 8)], 25, 0, 0, 255),
        (MoveId::UTilt, &[(5, 8), (16, 23)], 33, 0, 0, 255),
        (MoveId::DTilt, &[(5, 6)], 27, 0, 0, 255),
        (MoveId::NAir, &[(3, 6), (7, 30)], 50, 6, 2, 31),
        (MoveId::FAir, &[(5, 6), (7, 7)], 35, 5, 4, 11),
        (MoveId::BAir, &[(4, 6), (7, 14)], 38, 7, 3, 31),
        (MoveId::UAir, &[(4, 15), (23, 24)], 44, 13, 3, 44),
        (MoveId::DAir, &[(15, 18), (19, 33)], 55, 22, 2, 53),
    ];
    let sim = duel();
    let w = needles(&sim);
    for (id, windows, faf, landing, ac_before, ac_from) in table {
        let mv = w.get(id as u8);
        // Every hitbox lies in one of the reference's windows, and every window has hitboxes (move frames are the reference's
        // minus one).
        for hb in &mv.hitboxes {
            assert!(
                windows
                    .iter()
                    .any(|&(a, b)| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: hitbox {}..{} (reference {}..{}) is in no window of {windows:?}",
                hb.start,
                hb.end,
                hb.start + 1,
                hb.end + 1
            );
        }
        for &(a, b) in windows {
            assert!(
                mv.hitboxes
                    .iter()
                    .any(|hb| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: window {a}..{b} has no hitbox"
            );
        }
        assert_eq!(u16::from(mv.total_frames) + 2, u16::from(faf), "{id:?} FAF");
        assert_eq!(mv.landing_lag, landing, "{id:?} landing lag");
        assert_eq!(mv.autocancel_before, ac_before, "{id:?} autocancel before");
        let from = if ac_from == 255 { 255 } else { ac_from - 1 };
        assert_eq!(mv.autocancel_after, from, "{id:?} autocancel from");
    }
    // The up air's structure: the loop re-armed every 4 frames from reference frame 4 (group 0 only), the final hit its own group,
    // grounded and airborne loop boxes with their own angles.
    let uair = w.get(MoveId::UAir as u8);
    assert_eq!(uair.rehit, Some((3, 4)));
    assert_eq!(uair.rehit_groups, 0b1);
    let loop_boxes: Vec<_> = uair.hitboxes.iter().filter(|h| h.group == 0).collect();
    assert!(loop_boxes
        .iter()
        .any(|h| h.targets == TARGETS_AERIAL && h.angle == ANGLE_AUTOLINK));
    assert!(loop_boxes
        .iter()
        .any(|h| h.targets == TARGETS_GROUNDED && h.angle == 80));
    assert!(uair
        .hitboxes
        .iter()
        .any(|h| h.group == 1 && h.hitlag == 200));
}

#[test]
fn the_rogues_jumps_have_the_references_air_times() {
    // Short hop 32, full hop 47, short hop fast fall 23, full hop fast fall 33 (ultimateframedata.com); within one frame (ours
    // count the landing frame).
    let air_time = |full: bool, ff: bool| {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(10), -1);
        sim.stand(0, Fx::ZERO, 1);
        sim.tick(inp(0, 0, JUMP));
        while sim.f().grounded() {
            sim.tick(inp(0, 0, if full { JUMP } else { 0 }));
        }
        let mut n = 0u32;
        let mut flicked = false;
        while !sim.f().grounded() && n < 200 {
            let down = ff && !flicked && sim.f().vel.y <= Fx::ZERO;
            flicked |= down;
            let held = if full { JUMP } else { 0 };
            sim.tick(inp(0, if down { -127 } else { 0 }, held));
            n += 1;
        }
        n
    };
    for (full, ff, reference) in [
        (false, false, 32u32),
        (true, false, 47),
        (false, true, 23),
        (true, true, 33),
    ] {
        let t = air_time(full, ff);
        assert!(
            t.abs_diff(reference) <= 1,
            "full {full} fast fall {ff}: {t} frames, reference {reference}"
        );
    }
}

/// The rogue in the air starting its up air with the duelist just above it, the rogue moving up at `attacker_vy` (rising out of a
/// jump) or falling; with `sdi_away` the duelist flicks away through the hits. Returns the sim after `frames` frames.
fn uair_on_airborne(attacker_vy: Fx, frames: usize, sdi_away: bool) -> Sim {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, attacker_vy);
    sim.put_airborne(1, fx(1, 10), fx(218, 10), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
    for k in 0..frames {
        // Survival DI away (up and right), a fresh flick on every other frame.
        let p1 = if sdi_away && k % 2 == 0 {
            inp(127, 127, 0)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, 0), p1);
    }
    sim
}

#[test]
fn up_air_loops_three_hits_then_lands_its_final_hit_once() {
    // Out of a jump (the rogue still rising): the loop carries the target up with it into the final kick.
    let sim = uair_on_airborne(RISING, 80, false);
    // 3 loop hits of 1% and one final of 4% (short-hop damage does not apply: not a short hop).
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 70));
}

#[test]
fn up_air_drags_a_target_down_with_a_falling_attacker() {
    // The rogue falling at its normal top speed through its up air: each looping hit sends the target at the rogue's (downward)
    // velocity plus a pull toward the kick, so the target comes down with it (no faster than the autolink's fall cap) instead of being
    // knocked away, and all three loop hits connect.
    // Just past the apex (falling at half the top speed) the drag holds through all three loop hits; at the full top speed the rogue
    // outpaces the capped pull and the loop starts to lose the target (two hits), as the reference's cap implies.
    let top = duel().content.fighters[usize::from(ROGUE)].max_fall_speed;
    for (vy, loop_hits) in [(top / Fx::from_int(2), 3), (top, 2)] {
        let mut sim = duel();
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, -vy);
        sim.put_airborne(1, fx(1, 10), fx(218, 10), Fx::ZERO, Fx::ZERO);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
        let y0 = sim.fighter(1).pos.y;
        while sim.f().state_frame < 15 {
            sim.tick(inp(0, 0, 0));
        }
        let target = sim.fighter(1);
        assert!(
            target.pos.y < y0,
            "the target was carried down: {:?} -> {:?}",
            y0,
            target.pos.y
        );
        assert!(
            target.kb_vel.y < Fx::ZERO,
            "and is still going down, not knocked away"
        );
        assert_eq!(
            sim.fighter(1).percent,
            dealt(&sim, 10 * loop_hits),
            "{loop_hits} loop hits falling at {vy:?}"
        );
    }
}

#[test]
fn a_fast_falling_attacker_outruns_the_drag_and_loses_the_target() {
    // Nothing holds the target to the kick: fast falling from the first frame, the rogue falls faster than an autolinked target can be
    // pulled (the reference's fall cap), so after the first loop hit the target is left behind. No forced attachment.
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, fx(1, 10), fx(218, 10), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
    sim.tick(inp(0, -127, 0));
    while sim.f().state_frame < 30 && !sim.f().grounded() {
        sim.tick(inp(0, -127, 0));
    }
    assert!(
        sim.fighter(1).percent < dealt(&sim, 30),
        "the loop broke off: {:?}",
        sim.fighter(1).percent
    );
}

#[test]
fn survival_di_moves_a_looped_target_and_the_pull_starts_from_there() {
    // Survival DI works through the loop's hitlag (x1.2 here): the target is nudged, and the next pull is worked out from the
    // nudged place. With the standard survival DI distance the pull brings it back, so the loop still holds (as the reference's
    // autolink is meant to); flicking changes where it is, frame by frame.
    let path = |sdi: bool| {
        let sim = uair_on_airborne(RISING, 4, sdi);
        sim.fighter(1).pos.x - sim.f().pos.x
    };
    assert!(
        path(true) > path(false),
        "nudged away during the first hitlag"
    );
}

#[test]
fn up_air_has_no_phantom_hits_on_a_target_out_of_reach() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, Fx::from_int(5), Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, Fx::ZERO);
}

#[test]
fn up_air_pops_a_grounded_target_up_without_linking_it() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(3, 10), -1);
    // The rogue just above the target's head, falling onto it with the loop out.
    sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].pos.y = fx(2, 10);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
    sim.ticks(5, inp(0, 0, 0));
    let t = sim.fighter(1);
    assert!(t.percent > Fx::ZERO, "the grounded loop box hit");
    assert!(!t.link, "a grounded target is not linked");
}

#[test]
fn down_air_dives_and_its_landing_hit_comes_out_once_on_landing() {
    // The rogue starts its down air above the stage with the duelist standing ahead of where it lands.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(3), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    let y0 = sim.f().pos.y;
    // The stall: it rises first.
    sim.ticks(6, inp(0, 0, 0));
    assert!(sim.f().pos.y > y0, "the stall rises");
    let mut landed_into = None;
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        if sim.f().grounded() && landed_into.is_none() {
            landed_into = Some((sim.f().state, sim.f().move_id));
        }
    }
    assert_eq!(
        landed_into,
        Some((S::Attack, MoveId::Ext0 as u8)),
        "a landing mid-dive starts the landing hit"
    );
    // The landing hit is 2%; the dive itself did not reach the duelist, so that is all it took, once.
    let percent = sim.fighter(1).percent;
    assert!(
        percent == dealt(&sim, 20) || percent == Fx::ZERO,
        "at most one landing hit: {percent:?}"
    );
}

#[test]
fn down_air_landing_hit_strikes_a_target_beside_the_landing() {
    let mut sim = duel();
    // Low enough that the dive (1.2 forward, 6 down a frame from frame 15) lands just short of the duelist.
    sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, Fx::ZERO);
    sim.stand(1, fx(7, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    let mut from_landing = Fx::ZERO;
    for _ in 0..60 {
        let before = sim.fighter(1).percent;
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Attack && sim.f().move_id == MoveId::Ext0 as u8 {
            from_landing += sim.fighter(1).percent - before;
        }
    }
    assert_eq!(from_landing, dealt(&sim, 20), "one 2% landing hit");
}

#[test]
fn a_missed_down_air_still_has_its_landing_lag() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DAir);
    while !sim.f().grounded() {
        sim.tick(inp(0, 0, 0));
    }
    // 22 frames of landing lag: still in the landing move 20 frames later, free soon after.
    let mut busy = 0;
    while sim.f().state == S::Attack && busy < 60 {
        sim.tick(inp(0, 0, 0));
        busy += 1;
    }
    assert!((20..=23).contains(&busy), "landing lag {busy} frames");
}

#[test]
fn up_tilt_holds_the_target_for_its_second_hit_at_any_percent() {
    // A target in the air in front at the stretch kick's height (the first hit is hard to land on a grounded one, as in the
    // reference): the set knockback holds it for the axe kick whatever its percent.
    for percent in [0, 60, 120] {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.put_airborne(1, fx(5, 10), fx(12, 10), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].percent = Fx::from_int(percent);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UTilt);
        sim.ticks(40, inp(0, 0, 0));
        let took = sim.fighter(1).percent - Fx::from_int(percent);
        assert_eq!(took, dealt(&sim, 70), "both hits at {percent}%");
    }
}

#[test]
fn forward_tilt_launches_nearly_straight_up_for_a_follow_up() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(9, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FTilt);
    let mut n = 0;
    while !sim.fighter(1).launch_pending && n < 20 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    let degrees = i32::from(sim.fighter(1).launch_angle) * 360 / 4096;
    assert!((70..=90).contains(&degrees), "launch angle {degrees}");
}

/// Checks each move against reference frame data: every hitbox in one of the windows, every window with a hitbox, the FAF, the landing
/// lag and the autocancel windows.
fn check_frame_data(w: &sim_core::moves::Weapon, table: &[FrameData]) {
    for &(id, windows, faf, landing, ac_before, ac_from) in table {
        let mv = w.get(id as u8);
        for hb in &mv.hitboxes {
            assert!(
                windows
                    .iter()
                    .any(|&(a, b)| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: hitbox {}..{} (reference {}..{}) is in no window of {windows:?}",
                hb.start,
                hb.end,
                hb.start + 1,
                hb.end + 1
            );
        }
        for &(a, b) in windows {
            assert!(
                mv.hitboxes
                    .iter()
                    .any(|hb| hb.start + 1 == a && hb.end + 1 == b),
                "{id:?}: window {a}..{b} has no hitbox"
            );
        }
        assert_eq!(u16::from(mv.total_frames) + 2, u16::from(faf), "{id:?} FAF");
        assert_eq!(mv.landing_lag, landing, "{id:?} landing lag");
        assert_eq!(mv.autocancel_before, ac_before, "{id:?} autocancel before");
        let from = if ac_from == 255 { 255 } else { ac_from - 1 };
        assert_eq!(mv.autocancel_after, from, "{id:?} autocancel from");
    }
}

#[test]
fn the_rest_of_the_kit_has_the_references_frame_data() {
    // The jabs, rapid jab (ours loops in 14 frames, see moves.rs), finisher, smashes, grabs and throws (the hit, then the release) from
    // the reference; the dash attack from the earlier game (clean 6, late 7-12, FAF 36).
    let table: [FrameData; 16] = [
        (MoveId::Jab, &[(2, 2)], 18, 0, 0, 255),
        (MoveId::Jab2, &[(3, 3)], 18, 0, 0, 255),
        (MoveId::Ext1, &[(5, 13)], 16, 0, 0, 255),
        (MoveId::Ext2, &[(5, 6)], 36, 0, 0, 255),
        (MoveId::DashAttack, &[(6, 6), (7, 12)], 36, 0, 0, 255),
        (MoveId::FSmash, &[(12, 12), (20, 21)], 45, 0, 0, 255),
        (MoveId::USmash, &[(11, 11), (14, 15)], 53, 0, 0, 255),
        (MoveId::DSmash, &[(8, 9), (15, 16)], 49, 0, 0, 255),
        (MoveId::Grab, &[(6, 7)], 37, 0, 0, 255),
        (MoveId::DashGrab, &[(9, 10)], 45, 0, 0, 255),
        (MoveId::PivotGrab, &[(11, 12)], 40, 0, 0, 255),
        (MoveId::Pummel, &[(2, 2)], 16, 0, 0, 255),
        (MoveId::FThrow, &[(20, 20), (22, 22)], 40, 0, 0, 255),
        (MoveId::BThrow, &[(15, 15), (17, 17)], 36, 0, 0, 255),
        (MoveId::UThrow, &[(19, 19), (21, 21)], 50, 0, 0, 255),
        (MoveId::DThrow, &[(26, 26), (30, 30)], 54, 0, 0, 255),
    ];
    let sim = duel();
    let w = needles(&sim);
    check_frame_data(w, &table);
    // No third jab; the throws total the reference's 7 / 7 / 6 / 6%.
    assert!(w.get(MoveId::Jab3 as u8).is_empty());
    for (id, tenths) in [
        (MoveId::FThrow, 70),
        (MoveId::BThrow, 70),
        (MoveId::UThrow, 60),
        (MoveId::DThrow, 60),
    ] {
        let total = w
            .get(id as u8)
            .hitboxes
            .iter()
            .fold(Fx::ZERO, |a, h| a + h.damage);
        assert_eq!(total, Fx::from_ratio(tenths, 10), "{id:?} damage");
    }
}

/// The rogue standing at 0 with the duelist just in front, facing it.
fn up_close(gap: Fx) -> Sim {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

/// The moves player 0 went through (each new move once, in order) over `frames` frames of `input(frame)`.
fn moves_seen(sim: &mut Sim, frames: usize, input: impl Fn(usize) -> sim_core::Input) -> Vec<u8> {
    let mut seen: Vec<u8> = Vec::new();
    let mut last_frame = u16::MAX;
    for k in 0..frames {
        sim.tick(input(k));
        let f = sim.f();
        if f.state == S::Attack {
            // A move started again (the rapid jab's loop) counts once more only when it changes slot.
            if seen.last() != Some(&f.move_id) || f.state_frame < last_frame && seen.is_empty() {
                seen.push(f.move_id);
            }
            last_frame = f.state_frame;
        }
    }
    seen
}

#[test]
fn mashing_attack_goes_jab_jab_rapid_jab_then_the_finisher() {
    let mut sim = up_close(fx(9, 10));
    // Mash for 60 frames (a press every 4 frames), then let go.
    let seen = moves_seen(&mut sim, 140, |k| {
        inp(0, 0, if k < 60 && k % 4 < 2 { ATTACK } else { 0 })
    });
    assert_eq!(
        seen,
        vec![
            MoveId::Jab as u8,
            MoveId::Jab2 as u8,
            MoveId::Ext1 as u8,
            MoveId::Ext2 as u8
        ],
        "jab, jab 2, the rapid jab, the finisher"
    );
    // 2 + 1.6, at least eight 0.3% stabs (the loops' worth less rounding), and the 2% finisher.
    let took = sim.fighter(1).percent;
    assert!(
        took >= dealt(&sim, 20 + 16 + 24 + 20),
        "the rapid jab kept hitting: {took:?}"
    );
}

#[test]
fn holding_attack_through_the_hits_runs_the_rapid_jab_until_let_go() {
    let mut sim = up_close(fx(9, 10));
    let seen = moves_seen(&mut sim, 120, |k| {
        inp(0, 0, if k < 70 { ATTACK } else { 0 })
    });
    assert_eq!(
        seen,
        vec![
            MoveId::Jab as u8,
            MoveId::Jab2 as u8,
            MoveId::Ext1 as u8,
            MoveId::Ext2 as u8
        ]
    );
}

#[test]
fn one_press_is_one_jab() {
    let mut sim = up_close(fx(9, 10));
    let seen = moves_seen(&mut sim, 40, |k| inp(0, 0, if k < 1 { ATTACK } else { 0 }));
    assert_eq!(seen, vec![MoveId::Jab as u8]);
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 20));
}

#[test]
fn the_rapid_jab_holds_its_target_in_place() {
    // The stabs' +2 hitstun keeps the target caught: it is still in hitstun on every frame of a loop after the first stab.
    let mut sim = up_close(fx(9, 10));
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Ext1);
    let mut caught = 0;
    for _ in 0..40 {
        sim.tick(inp(0, 0, ATTACK));
        if sim.fighter(1).state == S::Hitstun {
            caught += 1;
        }
    }
    assert!(caught >= 34, "caught on {caught} of 40 frames");
    assert_eq!(
        sim.f().move_id,
        MoveId::Ext1 as u8,
        "still looping while held"
    );
}

#[test]
fn dash_attack_is_the_earlier_games_pop_up() {
    let mut sim = up_close(fx(6, 10));
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DashAttack);
    let mut n = 0;
    while !sim.fighter(1).launch_pending && n < 20 {
        sim.tick(inp(0, 0, 0));
        n += 1;
    }
    let degrees = i32::from(sim.fighter(1).launch_angle) * 360 / 4096;
    assert!(
        (79..=80).contains(&degrees),
        "the clean hit's 80 degrees: {degrees}"
    );
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 100));
}

#[test]
fn both_smash_kicks_land_at_low_and_high_percent() {
    // Forward smash and down smash: the first kick's set knockback holds the target for the second (5 + 8% and 4 + 6%).
    for (id, gap, both) in [
        (MoveId::FSmash, fx(8, 10), 130),
        (MoveId::DSmash, fx(7, 10), 100),
    ] {
        for percent in [0, 100] {
            let mut sim = up_close(gap);
            sim.state.fighters[1].percent = Fx::from_int(percent);
            sim_core::fighter::begin_attack(&mut sim.state.fighters[0], id);
            sim.ticks(40, inp(0, 0, 0));
            let took = sim.fighter(1).percent - Fx::from_int(percent);
            assert_eq!(took, dealt(&sim, both), "{id:?} both hits at {percent}%");
        }
    }
}

#[test]
fn a_light_hit_lands_on_its_feet_and_a_tumble_lies_down() {
    // Just above the floor in hitstun, falling: a light (non-tumble) hitstun lands standing and finishes on the ground; a tumble that
    // lands without a tech is a knockdown.
    for (tumble, lies_down) in [(false, false), (true, true)] {
        let mut sim = duel();
        sim.stand(0, Fx::from_int(-10), 1);
        sim.put_airborne(1, Fx::ZERO, fx(3, 10), Fx::ZERO, Fx::ZERO);
        let f = &mut sim.state.fighters[1];
        f.state = S::Hitstun;
        f.hitstun = 20;
        f.tumble = tumble;
        f.kb_vel = sim_core::Vec2::new(Fx::ZERO, -fx(1, 10));
        let mut states = Vec::new();
        for _ in 0..40 {
            sim.tick(inp(0, 0, 0));
            states.push(sim.fighter(1).state);
        }
        assert_eq!(
            states.contains(&S::Knockdown),
            lies_down,
            "tumble {tumble}: {states:?}"
        );
        if !tumble {
            assert!(sim.fighter(1).grounded());
            assert!(
                states.iter().take(15).all(|s| *s == S::Hitstun),
                "the hitstun carries on on the ground"
            );
            assert_eq!(*states.last().unwrap(), S::Idle);
        }
    }
}

#[test]
fn a_drag_down_up_air_lands_its_target_standing() {
    // The up air dragging a target down to the floor: the looping hits are light, so it lands on its feet (no tech, no knockdown), as
    // the reference's drag-downs do, open to a follow-up.
    let top = duel().content.fighters[usize::from(ROGUE)].max_fall_speed;
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, fx(8, 10), Fx::ZERO, -top / Fx::from_int(2));
    sim.put_airborne(1, fx(1, 10), fx(8, 10) + fx(18, 10), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
    let mut states = Vec::new();
    for _ in 0..90 {
        sim.tick(inp(0, 0, 0));
        states.push(sim.fighter(1).state);
    }
    assert!(sim.fighter(1).percent > Fx::ZERO, "the loop hit");
    assert!(sim.fighter(1).grounded(), "and the target came down");
    assert!(
        !states.contains(&S::Knockdown) && !states.contains(&S::GetUp),
        "without a knockdown: {states:?}"
    );
}

// ---- Specials ----------------------------------------------------------------------------------------------------------------------

/// Runs `frames` frames of `input(frame)` for player 0 and counts the projectiles it fired (new ones appearing).
fn shots_fired(sim: &mut Sim, frames: usize, input: impl Fn(usize) -> sim_core::Input) -> usize {
    let mut fired = 0;
    for k in 0..frames {
        sim.tick(input(k));
        fired += sim
            .state
            .projectiles
            .iter()
            .filter(|p| p.active && p.owner == 0 && p.age == 1)
            .count();
    }
    fired
}

/// Presses special on the frames in `presses` (one frame each), holding it through `held` if given, with `extra` buttons on their frames.
fn needle_input(
    k: usize,
    presses: &[usize],
    held: usize,
    extra: &[(usize, u16)],
) -> sim_core::Input {
    let mut b = if presses.contains(&k) || k < held {
        SPECIAL
    } else {
        0
    };
    for &(at, button) in extra {
        if at == k {
            b |= button;
        }
    }
    inp(0, 0, b)
}

fn needle_range() -> Sim {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

#[test]
fn needle_storm_charges_by_itself_and_a_second_press_throws() {
    // As the reference: one press starts the charge, which carries on with the button let go (a needle on frame 8, then every 15
    // frames); pressing again throws what has built up. A second press during the wind-up throws the first needle at once.
    for (second, needles) in [(3usize, 1usize), (15, 1), (30, 2), (45, 3), (75, 5)] {
        let mut sim = needle_range();
        let fired = shots_fired(&mut sim, second + 60, |k| {
            needle_input(k, &[0, second], 0, &[])
        });
        assert_eq!(fired, needles, "second press on frame {second}");
        assert_eq!(sim.f().stored, 0);
    }
}

#[test]
fn a_full_charge_stores_itself_and_the_next_press_throws_all_six() {
    let mut sim = needle_range();
    // Held (or not: it charges either way) until full: six needles, stored, nothing thrown, free to move again.
    let fired = shots_fired(&mut sim, 110, |k| needle_input(k, &[], 110, &[]));
    assert_eq!(fired, 0, "a full charge throws nothing by itself");
    assert_eq!(sim.f().stored, 6);
    assert_ne!(sim.f().state, S::Attack, "and ends");
    sim.ticks(5, inp(0, 0, 0));
    let fired = shots_fired(&mut sim, 40, |k| needle_input(k, &[0], 0, &[]));
    assert_eq!(fired, 6, "thrown at once");
    assert_eq!(sim.f().stored, 0, "spent");
}

#[test]
fn shield_or_jump_stores_a_part_charge_and_the_next_press_charges_on_from_it() {
    for (button, name) in [(SHIELD, "shield"), (JUMP, "jump")] {
        let mut sim = needle_range();
        // Three needles (frames 8, 23 and 38), then the button: kept, nothing thrown.
        let fired = shots_fired(&mut sim, 60, |k| needle_input(k, &[0], 0, &[(44, button)]));
        assert_eq!(fired, 0, "{name}: storing throws nothing");
        assert_eq!(sim.f().stored, 3, "{name}: three kept");
        // Back on the ground, still: the next press carries on charging from three; a press 20 frames later throws four.
        sim.ticks(40, inp(0, 0, 0));
        let fired = shots_fired(&mut sim, 70, |k| needle_input(k, &[0, 25], 0, &[]));
        assert_eq!(fired, 4, "{name}: charged on from three");
    }
}

#[test]
fn a_charge_in_the_air_carries_on_after_landing() {
    let mut sim = needle_range();
    sim.put_airborne(0, Fx::ZERO, fx(15, 10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SPECIAL));
    let mut charging_on_ground = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        charging_on_ground |= sim.f().grounded()
            && sim.f().move_id == MoveId::Ext3 as u8
            && sim.f().state == S::Attack;
    }
    assert!(charging_on_ground, "still charging on the ground");
}

#[test]
fn needles_hit_a_fighter_down_the_stage() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(5), -1);
    shots_fired(&mut sim, 120, |k| needle_input(k, &[0, 45], 0, &[]));
    let took = sim.fighter(1).percent;
    assert!(
        took >= dealt(&sim, 3 * 8) && took <= dealt(&sim, 3 * 15),
        "three needles of 0.8 to 1.5%: {took:?}"
    );
}

/// The rogue throws its pulling line at the duelist standing `gap` away (or in the air at `height`), then waits `frames` frames.
fn hooked(gap: Fx, height: Fx, frames: usize) -> Sim {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    if height > Fx::ZERO {
        sim.put_airborne(1, gap, height, Fx::ZERO, Fx::ZERO);
    } else {
        sim.stand(1, gap, -1);
    }
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::SideSpecial);
    sim.ticks(frames, inp(0, 0, 0));
    sim
}

#[test]
fn the_line_reels_a_hit_fighter_in_to_just_in_front() {
    for gap in [Fx::from_int(3), Fx::from_int(6)] {
        let sim = hooked(gap, Fx::ZERO, 55);
        let ahead = sim.fighter(1).pos.x - sim.f().pos.x;
        assert!(
            sim.fighter(1).percent == dealt(&sim, 30),
            "the line hit from {gap:?}"
        );
        assert!(
            ahead > fx(6, 10) && ahead < fx(14, 10),
            "reeled in from {gap:?} to {ahead:?} in front"
        );
    }
}

#[test]
fn a_reeled_in_fighter_is_still_in_hitstun_when_the_thrower_can_act() {
    // The hitstun outlasts the pull and the throw's ending lag: a follow-up is guaranteed.
    let mut sim = hooked(Fx::from_int(5), Fx::ZERO, 0);
    let mut free_at = None;
    for k in 0..90 {
        sim.tick(inp(0, 0, 0));
        if free_at.is_none() && sim.f().state != S::Attack {
            free_at = Some(k);
            assert_eq!(
                sim.fighter(1).state,
                S::Hitstun,
                "the target is still caught when the thrower is free"
            );
            assert!(
                sim.fighter(1).hitstun >= 5,
                "with time to spare: {}",
                sim.fighter(1).hitstun
            );
        }
    }
    assert!(free_at.is_some());
}

#[test]
fn a_fighter_reeled_in_from_the_air_lands_on_its_feet() {
    let mut sim = hooked(Fx::from_int(5), Fx::from_int(3), 0);
    let mut states = Vec::new();
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        states.push(sim.fighter(1).state);
    }
    assert!(sim.fighter(1).percent > Fx::ZERO, "hit");
    assert!(sim.fighter(1).grounded(), "brought down to the thrower");
    assert!(!states.contains(&S::Knockdown), "standing: {states:?}");
    let ahead = sim.fighter(1).pos.x - sim.f().pos.x;
    assert!(
        ahead > fx(5, 10) && ahead < fx(15, 10),
        "in front: {ahead:?}"
    );
}

#[test]
fn vanish_teleports_the_way_the_stick_points_and_leaves_the_fighter_helpless() {
    // Neutral: straight up, 20 frames at 1.5 reference units; pushed sideways: 2.5 units a frame along the stick.
    for (stick, dx_min, dy_min) in [(0i8, -1, 3), (127, 5, -1)] {
        let mut sim = duel();
        sim.stand(1, Fx::from_int(10), -1);
        sim.put_airborne(0, Fx::from_int(-8), Fx::from_int(8), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UpSpecial);
        let p0 = sim.f().pos;
        let mut intangible = 0;
        for k in 0..60 {
            sim.tick(inp(stick, 0, 0));
            if sim_core::combat::is_intangible(sim.f()) && (20..54).contains(&k) {
                intangible += 1;
            }
        }
        let d = sim.f().pos - p0;
        assert!(
            d.x >= Fx::from_int(dx_min) && d.y >= Fx::from_int(dy_min),
            "stick {stick}: moved {d:?}"
        );
        assert!(
            (33..=34).contains(&intangible),
            "intangible through the teleport: {intangible}"
        );
        let mut n = 0;
        while sim.f().state == S::Attack && n < 60 {
            sim.tick(inp(0, 0, 0));
            n += 1;
        }
        assert_eq!(sim.f().state, S::Helpless, "helpless after");
    }
}

#[test]
fn vanish_explodes_where_it_starts() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(12, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UpSpecial);
    sim.ticks(45, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 120));
}

#[test]
fn bouncing_fish_kicks_down_on_a_fighter_ahead_and_bounces_back_off_it() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(46, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DownSpecial);
    let mut bounced_at = None;
    for k in 0..70 {
        sim.tick(inp(0, 0, 0));
        if bounced_at.is_none() && sim.f().move_id == MoveId::Ext5 as u8 {
            bounced_at = Some((k, sim.f().pos.x));
        }
    }
    assert_eq!(sim.fighter(1).percent, dealt(&sim, 110), "the kick");
    let (_, x) = bounced_at.expect("it bounced");
    assert!(sim.f().pos.x < x, "back off the target");
}

#[test]
fn a_missed_bouncing_fish_lands_with_its_lag() {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(-10), 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DownSpecial);
    sim.tick(inp(0, 0, 0));
    while !sim.f().grounded() {
        sim.tick(inp(0, 0, 0));
    }
    assert!(
        sim.f().pos.x > Fx::from_int(4),
        "it flipped forward: {:?}",
        sim.f().pos.x
    );
    assert_eq!(sim.f().state, S::Landing);
    assert_eq!(sim.f().lag, 36);
}

#[test]
fn a_drag_down_up_air_lands_its_target_in_front() {
    // The loop's pull aims a little in front of the kick, so the dragged target comes down in front of the rogue, not on top of it.
    for facing in [1i8, -1] {
        let top = duel().content.fighters[usize::from(ROGUE)].max_fall_speed;
        let mut sim = duel();
        sim.put_airborne(0, Fx::ZERO, fx(8, 10), Fx::ZERO, -top / Fx::from_int(2));
        sim.state.fighters[0].facing = facing;
        sim.put_airborne(
            1,
            fx(1, 10).mul_int(i32::from(facing)),
            fx(26, 10),
            Fx::ZERO,
            Fx::ZERO,
        );
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UAir);
        sim.ticks(60, inp(0, 0, 0));
        let ahead = (sim.fighter(1).pos.x - sim.f().pos.x).mul_int(i32::from(facing));
        assert!(sim.fighter(1).percent > Fx::ZERO);
        assert!(
            ahead > fx(5, 10) && ahead < fx(25, 10),
            "facing {facing}: landed {ahead:?} in front"
        );
    }
}

#[test]
fn pressing_special_again_kicks_the_bouncing_fish_early() {
    // Pressed again before frame 16, the kick comes out on frame 18 instead of 23 (the early kick, ext6), so it lands shorter.
    let first_hit = |press: bool| {
        let mut sim = duel();
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_int(-10), 1);
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DownSpecial);
        let mut kicked = None;
        for k in 0..40 {
            sim.tick(inp(0, 0, if press && k == 5 { SPECIAL } else { 0 }));
            let f = sim.f();
            let kicking = f.state == S::Attack
                && sim_core::combat::active_hitboxes(f, needles(&sim).get(f.move_id), Fx::ONE)
                    .next()
                    .is_some();
            if kicking && kicked.is_none() {
                kicked = Some((k, f.move_id));
            }
        }
        kicked.expect("it kicked")
    };
    let (normal_at, normal_move) = first_hit(false);
    let (early_at, early_move) = first_hit(true);
    assert_eq!(normal_move, MoveId::DownSpecial as u8);
    assert_eq!(early_move, MoveId::Ext6 as u8);
    assert_eq!(normal_at - early_at, 5, "five frames sooner (23 to 18)");
}
