//! The Longsword's jab (sim 138, owner decision 98): the previous reference version's (1.1.x) two-hit jab on that version's hit rules
//! (E40), with the jab cancel (`Move::cancel_from`) on both hits and 13.0.1's tip pull on jab 1. Data and sources in
//! `docs/movesets/longsword/REFERENCE_DATA.md`. Like `older_rules.rs` for the Settler's: frame data, the hit properties, the cancel,
//! jab 2 sending up and out, and what follows the jab as a true combo.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, JUMP, SHIELD, SPECIAL};
use sim_core::moves::{MoveId, ANGLE_FORWARD, RULES_PREVIOUS};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const DUELIST: u8 = 0;
/// A second body for the percent windows (the rogue: lighter than the duelist).
const ROGUE: u8 = 2;

/// The duelist (player 0) facing right at 0 and a `victim` body (player 1) standing at 1.1 in front, at `percent`; the others out.
fn setup(victim: u8, percent: i32) -> Sim {
    let mut sim = Sim::with_chars([DUELIST, victim, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(11, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(percent);
    sim
}

fn weapon(sim: &Sim) -> &sim_core::moves::Weapon {
    let w = usize::from(sim.content.fighters[usize::from(DUELIST)].weapon);
    &sim.content.weapons[w]
}

#[test]
fn the_jab_is_the_previous_versions_on_its_frames_damage_and_knockback() {
    let sim = setup(DUELIST, 0);
    let w = weapon(&sim);
    let (j1, j2) = (&w.moves[MoveId::Jab as usize], &w.moves[MoveId::Jab2 as usize]);
    // FAF 26 and 29 (the move ends the frame before its FAF), continuable 18 [R] and jab 2's cancel 16 [E] (move frames are one less).
    assert_eq!((j1.total_frames, j2.total_frames), (24, 27));
    assert_eq!((j1.next, j1.next_from), (Some(MoveId::Jab2 as u8), 17));
    assert_eq!((j2.next, j2.next_from), (Some(MoveId::Jab as u8), 15));
    assert_eq!((j1.cancel_from, j2.cancel_from), (17, 15));
    assert!(w.moves[MoveId::Jab3 as usize].hitboxes.is_empty());
    // Active frames: jab 1 on 5-7, jab 2 on 3-5.
    let span = |m: &sim_core::moves::Move| {
        let first = m.hitboxes.iter().map(|h| h.start).min().unwrap();
        let last = m.hitboxes.iter().map(|h| h.end).max().unwrap();
        (first + 1, last + 1)
    };
    assert_eq!(span(j1), (5, 7));
    assert_eq!(span(j2), (3, 5));
    // Every box on the previous rules. After `tipper_last`: 0 the blade, 1 the arm, 2 the tip.
    let pct = |n: i32| Fx::from_ratio(n, 10);
    for h in &j1.hitboxes {
        assert_eq!(h.rules, RULES_PREVIOUS);
        let want = match h.priority {
            0 => (pct(30), 78, 55, 30, 70),
            1 => (pct(30), 65, 55, 30, 70),
            // The tip: 1.1.x's 5% with 13.0.1's pull (180, forward, BK 30, KS 12).
            _ => (pct(50), 180, 30, 12, 125),
        };
        assert_eq!(
            (
                h.damage,
                h.angle,
                h.base_knockback,
                h.knockback_growth,
                h.hitlag
            ),
            want
        );
        if h.priority == 2 {
            assert_eq!(h.angle_type, ANGLE_FORWARD);
        }
    }
    for h in &j2.hitboxes {
        assert_eq!(h.rules, RULES_PREVIOUS);
        let (dmg, lag) = if h.priority == 2 { (60, 125) } else { (40, 70) };
        assert_eq!(
            (
                h.damage,
                h.angle,
                h.base_knockback,
                h.knockback_growth,
                h.hitlag
            ),
            (pct(dmg), 45, 62, 75, lag)
        );
    }
}

/// Ticks the duelist's jab, out of reach (no hitlag): jab 1 pressed on tick 1, then jab 2 on tick 18 when `both`. Returns the first
/// frame (of the last jab) on which an input acts: the tick after the one that ends the move (as an input on tick 18 acts on frame
/// 18 in the cancel test).
fn free_after(both: bool) -> u32 {
    let mut sim = setup(DUELIST, 0);
    sim.stand(1, Fx::from_int(5), -1);
    let start = if both { 18 } else { 1 };
    for t in 1..=80u32 {
        let b = if t == 1 || (both && t == 18) {
            ATTACK
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
        if t > start && sim.fighter(0).state != S::Attack {
            return t - start + 2;
        }
    }
    panic!("the jab never ended");
}

#[test]
fn each_jab_lasts_to_its_faf_when_nothing_follows() {
    assert_eq!(free_after(false), 26, "jab 1 FAF");
    assert_eq!(free_after(true), 29, "jab 2 FAF");
}

#[test]
fn either_jab_can_be_cancelled_into_any_action_from_its_cancel_frame() {
    // The previous version's jab cancel (owner decision 98, as decision 70 for the Settler): from jab 1's frame 18 and jab 2's frame
    // 16 a jump, a shield, a grab, a dash, a crouch, a tilt or a special starts at once; a frame earlier it is still the jab.
    let cases: [(u16, i8, i8, S); 7] = [
        (JUMP, 0, 0, S::JumpSquat),
        (SHIELD, 0, 0, S::Shield),
        (GRAB, 0, 0, S::Attack),
        (0, 127, 0, S::Dash),
        (0, 0, -127, S::Crouch),
        (ATTACK, 0, -90, S::Attack),
        (SPECIAL, 0, 0, S::Attack),
    ];
    let (jab, jab2) = (MoveId::Jab as u8, MoveId::Jab2 as u8);
    for both in [false, true] {
        // (Jab 2 pressed on jab 1's frame 18 starts on tick 18: its frame 16 is tick 33.)
        let cancel = if both { 33u32 } else { 18 };
        for (button, x, y, want) in cases {
            for early in [true, false] {
                let mut sim = setup(DUELIST, 0);
                sim.stand(1, Fx::from_int(5), -1);
                let at = if early { cancel - 1 } else { cancel };
                for t in 1..=at {
                    let (b, sx, sy) = if t == 1 || (both && t == 18) {
                        (ATTACK, 0, 0)
                    } else if t == at {
                        (button, x, y)
                    } else {
                        (0, 0, 0)
                    };
                    sim.tick(inp(sx, sy, b));
                }
                let (now, id) = (sim.fighter(0).state, sim.fighter(0).move_id);
                if early {
                    assert!(
                        now == S::Attack && (id == jab || id == jab2),
                        "{both} {button} {x} {y}: one frame early it is still the jab ({now:?} {id})"
                    );
                } else {
                    assert_eq!(now, want, "{both} {button} {x} {y}");
                    if want == S::Attack {
                        assert!(
                            id != jab && id != jab2,
                            "{both} {button}: not the jab ({id})"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_two_hits_repeat_jab_2_carries_on_into_jab_1() {
    // Out of reach: jab 1, jab 2 pressed on 18, jab 1 again pressed on jab 2's frame 16 (tick 33).
    let mut sim = setup(DUELIST, 0);
    sim.stand(1, Fx::from_int(5), -1);
    for t in 1..=33u32 {
        let b = if t == 1 || t == 18 || t == 33 {
            ATTACK
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
        if t == 32 {
            assert_eq!(sim.fighter(0).move_id, MoveId::Jab2 as u8);
        }
    }
    assert_eq!(sim.fighter(0).move_id, MoveId::Jab as u8, "jab 1 again");
}

/// Whether the duelist's next frame is the first it can act on in move `id`: out of hitlag, its move frame reaching `from`.
fn next_frame_is(sim: &Sim, id: MoveId, from: u16) -> bool {
    let f = sim.fighter(0);
    f.state == S::Attack && f.move_id == id as u8 && f.hitlag == 0 && f.state_frame + 1 >= from
}

/// The victim's launch after jab 1 (and jab 2 when `both`, pressed when jab 1 can first carry on) once its hitlag is over.
fn launch_after(victim: u8, victim_x: Fx, percent: i32, both: bool) -> sim_core::Vec2 {
    let mut sim = setup(victim, percent);
    sim.stand(1, victim_x, -1);
    let target = if both { 2 } else { 1 };
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..80u32 {
        let b = if both && next_frame_is(&sim, MoveId::Jab, 17) {
            ATTACK
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
        let v = sim.fighter(1);
        if v.hits_taken >= target && v.hitlag == 0 && !v.launch_pending {
            return v.kb_vel;
        }
    }
    panic!("the jab never landed");
}

/// Where only jab 1's tip reaches the foe (world units ahead).
fn tip() -> Fx {
    fx(TIP_TENTHS, 10)
}
const TIP_TENTHS: i32 = 35;

#[test]
fn jab_1s_tip_draws_the_foe_in_and_jab_2_sends_it_up_and_out() {
    // The tip pulls toward the duelist (13.0.1's 180-degree tip, BK 30, KS 12): it faces right, so toward it is -x. Jab 2 is the
    // launcher, 45 degrees with BK 62, KS 75: up and away, harder than jab 1's blade.
    for victim in [DUELIST, ROGUE] {
        for percent in [0, 50, 100, 150] {
            let pull = launch_after(victim, tip(), percent, false);
            assert!(pull.x < Fx::ZERO, "{victim} {percent}%: drawn in {pull:?}");
            assert!(pull.x > -fx(2, 10), "{victim} {percent}%: slight {pull:?}");
            let near = launch_after(victim, fx(11, 10), percent, false);
            assert!(
                near.y > Fx::ZERO,
                "{victim} {percent}%: jab 1 pops up {near:?}"
            );
            let pop = launch_after(victim, fx(11, 10), percent, true);
            assert!(
                pop.y > Fx::ZERO && pop.x > Fx::ZERO,
                "{victim} {percent}%: jab 2 up and out {pop:?}"
            );
            assert!(
                pop.x > near.x.abs(),
                "{victim} {percent}%: jab 2 sends further out than jab 1 {pop:?} {near:?}"
            );
        }
    }
}

/// Jab 1 (and jab 2 when `both`, pressed on jab 1's first continuable frame) at `percent` on a `victim` standing at `at`, up to the
/// frame before the duelist can first act (jab 1's frame 18, jab 2's 16, after any hitlag).
fn jab_until_free(victim: u8, percent: i32, both: bool, at: Fx) -> Sim {
    let mut sim = setup(victim, percent);
    sim.stand(1, at, -1);
    sim.tick(inp(0, 0, ATTACK));
    for _ in 2..=90u32 {
        if !both && next_frame_is(&sim, MoveId::Jab, 17) {
            return sim;
        }
        if both && next_frame_is(&sim, MoveId::Jab2, 15) {
            return sim;
        }
        let b = if both && next_frame_is(&sim, MoveId::Jab, 17) {
            ATTACK
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
    }
    panic!("the jab never reached its cancel");
}

/// Frames the victim is still in hitstun (hitlag included) on the first frame the duelist can act: the frame advantage on hit.
fn advantage(victim: u8, percent: i32, both: bool, at: Fx) -> i32 {
    let sim = jab_until_free(victim, percent, both, at);
    let v = sim.fighter(1);
    if v.state == S::Hitstun {
        i32::from(v.hitstun) + i32::from(v.hitlag)
    } else {
        0
    }
}

type Follow = &'static [(u32, u16, i8, i8)];

/// The jab, cancelled on its first free frame into `follow` (`(ticks after, buttons, stick x, stick y)`, the stick held from the
/// first on). `Some(true)`: the follow-up's first hit landed in the jab's hitstun (a true combo); `Some(false)`: later; `None`: a miss.
fn follow_up_at(victim: u8, percent: i32, both: bool, at: Fx, follow: Follow) -> Option<bool> {
    let mut sim = jab_until_free(victim, percent, both, at);
    let (sx, sy) = (follow[0].2, follow[0].3);
    for k in 0..60u32 {
        let b = follow
            .iter()
            .find(|(when, ..)| *when == k)
            .map_or(0, |(_, b, ..)| *b);
        let (before, stunned) = (sim.fighter(1).percent, sim.fighter(1).state == S::Hitstun);
        sim.tick(inp(sx, sy, b));
        if sim.fighter(1).percent > before {
            return Some(stunned);
        }
    }
    None
}

fn follow_up(victim: u8, percent: i32, both: bool, follow: Follow) -> Option<bool> {
    follow_up_at(victim, percent, both, fx(11, 10), follow)
}

const JAB_AGAIN: Follow = &[(0, ATTACK, 0, 0)];
const UP_TILT: Follow = &[(0, ATTACK, 0, 90)];
const FORWARD_TILT: Follow = &[(0, ATTACK, 90, 0)];
const DOWN_TILT: Follow = &[(0, ATTACK, 0, -90)];
const GRAB_IT: Follow = &[(0, GRAB, 0, 0)];
const JUMP_NAIR: Follow = &[(0, JUMP, 0, 0), (3, ATTACK, 0, 0)];
const UP_SMASH: Follow = &[(0, sim_core::input::buttons::C_UP, 0, 0)];
const DASH_ATTACK: Follow = &[(0, 0, 127, 0), (2, ATTACK, 127, 0)];
const JUMP_FAIR: Follow = &[(0, JUMP, 90, 0), (3, ATTACK, 90, 0)];
const JUMP_UAIR: Follow = &[(0, JUMP, 0, 90), (3, ATTACK, 0, 90)];
const UP_SPECIAL: Follow = &[(0, SPECIAL, 0, 127)];

#[test]
fn print_the_jabs_frame_advantage_and_follow_ups() {
    // For the docs (run with --nocapture): frame advantage on the first frame the duelist can act after jab 1, after jab 1 at the
    // tip and after jab 1 > 2 on each body, then each follow-up (the next jab, up tilt, forward tilt, down tilt, grab, jump > neutral
    // air, up smash) out of jab 1, out of jab 1 > 2, and out of jab 1 at the tip: x a true combo, o lands after the hitstun, . a miss.
    let cell = |r: Option<bool>| match r {
        Some(true) => 'x',
        Some(false) => 'o',
        None => '.',
    };
    let follows = [
        JAB_AGAIN,
        UP_TILT,
        FORWARD_TILT,
        DOWN_TILT,
        GRAB_IT,
        JUMP_NAIR,
        UP_SMASH,
        DASH_ATTACK,
        JUMP_FAIR,
        JUMP_UAIR,
        UP_SPECIAL,
    ];
    for victim in [DUELIST, ROGUE] {
        for percent in (0..=150).step_by(10) {
            let a1 = advantage(victim, percent, false, fx(11, 10));
            let a2 = advantage(victim, percent, true, fx(11, 10));
            let at = advantage(victim, percent, false, tip());
            let after = |both: bool, at: Fx| -> String {
                follows
                    .iter()
                    .map(|f| cell(follow_up_at(victim, percent, both, at, f)))
                    .collect()
            };
            eprintln!(
                "v{victim} {percent:>3}%  jab1 {a1:>3} tip {at:>3} jab1>2 {a2:>3}  after 1: {}  after 1>2: {}  after the tip: {}",
                after(false, fx(11, 10)),
                after(true, fx(11, 10)),
                after(false, tip()),
            );
        }
    }
    let _ = follow_up;
}
