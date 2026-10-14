//! E40 (sim 113, owner decisions 66-67): a hit on the previous reference version's rules (1.1.x, `Hitbox::rules` `RULES_PREVIOUS`),
//! and the Settler's jab, the only move that uses them. The rules, each against the same hit on 13.0.1's (sources in
//! the eleventh moveset's `REFERENCE_DATA.md` (`docs/movesets/`)): no one-on-one damage multiplier; that version's hitlag formula, a crouch shortening only
//! the victim's; one more frame of hitstun on a tumbling launch; no launch speed-up; the gravity boost on every tumbling launch and no
//! shared fall speed for a vertical one; its rage. Then the jab: it pops the foe up and away, and what follows it as a true combo.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB};
use sim_core::moves::{MoveId, RULES_CURRENT, RULES_PREVIOUS};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const SETTLER: u8 = 10;

/// The settler (player 0) facing right at 0 and a `victim` body (player 1) standing just in front, at `percent`; the others out.
fn setup(victim: u8, percent: i32) -> Sim {
    let mut sim = Sim::with_chars([SETTLER, victim, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(11, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(percent);
    sim
}

/// The settler's jab on 13.0.1's rules instead (the same data otherwise): what E40 changes.
fn on_current_rules(sim: &mut Sim) {
    let w = usize::from(sim.content.fighters[usize::from(SETTLER)].weapon);
    for id in [MoveId::Jab, MoveId::Jab2] {
        for h in &mut sim.content.weapons[w].moves[id as usize].hitboxes {
            h.rules = RULES_CURRENT;
        }
    }
}

/// Starts jab 1 and runs until player 1 is hit; returns the hitlag both were given then.
fn jab_hit(sim: &mut Sim) -> (u8, u8) {
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..10 {
        if sim.fighter(1).hitlag > 0 {
            return (sim.fighter(0).hitlag, sim.fighter(1).hitlag);
        }
        sim.tick(inp(0, 0, 0));
    }
    panic!("the jab never hit");
}

#[test]
fn a_previous_rules_hit_adds_its_damage_with_no_one_on_one_multiplier() {
    // 3% on the previous rules; the same hit on 13.0.1's adds 3 x 1.2 = 3.6%.
    let mut older = setup(0, 0);
    jab_hit(&mut older);
    assert_eq!(older.fighter(1).percent, Fx::from_int(3));
    let mut now = setup(0, 0);
    on_current_rules(&mut now);
    jab_hit(&mut now);
    assert_eq!(now.fighter(1).percent, fx(36, 10));
}

#[test]
fn its_hitlag_is_the_previous_formula_and_a_crouch_shortens_only_the_victims() {
    // floor(3 x 0.3846154 + 5) = 6 frames for both (13.0.1: floor(3 x 0.65 + 6) = 7).
    let mut older = setup(0, 0);
    assert_eq!(jab_hit(&mut older), (6, 6));
    let mut now = setup(0, 0);
    on_current_rules(&mut now);
    assert_eq!(jab_hit(&mut now), (7, 7));
    // Against a crouch: the previous version shortened only the victim's (6 x 0.67 = 4); 13.0.1 shortens both (7 x 0.67 = 4).
    let crouched = |sim: &mut Sim| {
        sim.state.fighters[1].state = S::Crouch;
        sim.tick2(inp(0, 0, ATTACK), inp(0, -127, 0));
        for _ in 0..10 {
            if sim.fighter(1).hitlag > 0 {
                return (sim.fighter(0).hitlag, sim.fighter(1).hitlag);
            }
            sim.tick2(inp(0, 0, 0), inp(0, -127, 0));
        }
        panic!("never hit");
    };
    assert_eq!(crouched(&mut setup(0, 0)), (6, 4));
    let mut now = setup(0, 0);
    on_current_rules(&mut now);
    assert_eq!(crouched(&mut now), (4, 4));
}

/// The settler's jab 1 made a strong vertical hit (90 degrees, BK `bkb`, KS 100) on the given rules; the victim is a settler too
/// (gravity 0.078, so the gravity boost is not 0). Returns the sim once the victim is launched.
fn strong_launch(rules: u8, bkb: i16) -> Sim {
    let mut sim = setup(SETTLER, 0);
    let w = usize::from(sim.content.fighters[usize::from(SETTLER)].weapon);
    for h in &mut sim.content.weapons[w].moves[MoveId::Jab as usize].hitboxes {
        h.rules = rules;
        h.angle = 90;
        h.base_knockback = bkb;
        h.knockback_growth = 100;
    }
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        let v = sim.fighter(1);
        if v.state == S::Hitstun && v.hitlag == 0 && !v.launch_pending {
            return sim;
        }
    }
    panic!("never launched");
}

#[test]
fn a_tumbling_launch_gets_a_frame_more_hitstun_no_speed_up_and_the_gravity_boost() {
    let older = strong_launch(RULES_PREVIOUS, 150);
    let now = strong_launch(RULES_CURRENT, 150);
    let (o, n) = (older.fighter(1), now.fighter(1));
    assert!(o.launch_older && !n.launch_older);
    // (Damage 3% and 3.6% feed the same knockback: the formula reads the damage before the multiplier.)
    assert_eq!(o.launch_kb, n.launch_kb);
    assert!(o.launch_kb >= Fx::from_int(80), "a tumble");
    // One more frame of hitstun (the 13.0.1 count, here, has already been sped up by its first launch frame).
    let full = |kb: Fx| ((kb.mul_int(2) / Fx::from_int(5)).floor_int() - 1) as u16;
    assert_eq!(
        o.hitstun,
        full(o.launch_kb) + 1,
        "the previous version's tumble frame"
    );
    assert!(
        n.balloon_faf > 0 && o.balloon_faf == 0,
        "no speed-up on the previous rules"
    );
    assert!(n.hitstun < o.hitstun);
    // Straight up: 13.0.1 gives no gravity boost in the vertical band; the previous rules do: 5 x (0.078 - 0.075) / 8 a frame more.
    let boost = Fx::from_int(5) * (Fx::from_ratio(78, 8000) - Fx::from_ratio(75, 8000));
    assert!((o.kb_vel.y - n.kb_vel.y - boost).abs() <= Fx::from_ratio(1, 1000));
}

#[test]
fn a_vertical_previous_rules_launch_falls_at_the_victims_own_speed() {
    // 13.0.1 holds a vertical tumble's fall speed at the shared 1.8 (0.225); the previous rules leave the settler's own 1.32 (0.165).
    let mut older = strong_launch(RULES_PREVIOUS, 260);
    let mut now = strong_launch(RULES_CURRENT, 260);
    let mut fastest = [Fx::ZERO; 2];
    for (k, sim) in [&mut older, &mut now].into_iter().enumerate() {
        for _ in 0..200 {
            sim.tick(inp(0, 0, 0));
            if sim.fighter(1).state != S::Hitstun {
                break;
            }
            fastest[k] = fastest[k].min(sim.fighter(1).vel.y);
        }
    }
    assert!(fastest[0] >= -FighterParamsSu::fall(1320), "{fastest:?}");
    assert!(fastest[1] < -FighterParamsSu::fall(1320), "{fastest:?}");
}

/// Reference units (thousandths) a frame in world units.
struct FighterParamsSu;
impl FighterParamsSu {
    fn fall(thousandths: i32) -> Fx {
        Fx::from_ratio(thousandths, 8000) + Fx::from_ratio(1, 1000)
    }
}

#[test]
fn rage_reaches_the_previous_versions_1_15() {
    // The attacker at 150%: x1.15 on the previous rules, x1.1 on 13.0.1's.
    let launch = |rules: u8| {
        let mut sim = setup(0, 50);
        sim.state.fighters[0].percent = Fx::from_int(150);
        if rules == RULES_CURRENT {
            on_current_rules(&mut sim);
        }
        jab_hit(&mut sim);
        sim.fighter(1).launch_kb
    };
    let (o, n) = (launch(RULES_PREVIOUS), launch(RULES_CURRENT));
    let ratio = o / n;
    assert!((ratio - fx(115, 110)).abs() < fx(1, 1000), "{o:?} {n:?}");
}

// ---- The Settler's jab ---------------------------------------------------------------------------------------------------------

/// The victim's launch after the settler's jab 1 (and jab 2 when `both`) once its hitlag is over: the speed it was sent at.
fn launch_after(victim_x: Fx, percent: i32, both: bool) -> sim_core::Vec2 {
    let mut sim = setup(0, percent);
    sim.stand(1, victim_x, -1);
    let target = if both { 2 } else { 1 };
    for t in 1..=40u32 {
        let b = if t == 1 || (both && (6..=8).contains(&t)) {
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

/// Where only jab 1's far glove reaches the duelist (world units ahead of the settler).
fn tip() -> Fx {
    fx(TIP.0, TIP.1)
}
// (The duelist's body circles reach the near glove up to about 2.1 units out; 2.2 to 2.5 only the far one.)
const TIP: (i32, i32) = (22, 10);

#[test]
fn jab_1s_far_glove_draws_the_foe_in_and_jab_2_pops_it_up_and_out() {
    // Owner decision 70: jab 1's far glove pulls the foe toward the settler (13.0.1's 180-degree far boxes, BK 15, KS 10); jab 2 is
    // the launcher, up and away (60 / 80 degrees, BK 20, KS 120). The settler faces right: toward it is -x.
    for percent in [0, 50, 100, 150] {
        let pull = launch_after(tip(), percent, false);
        assert!(pull.x < Fx::ZERO, "{percent}%: drawn in {pull:?}");
        // (A slight pull: a weak, nearly fixed knockback.)
        assert!(pull.x > -fx(1, 10), "{percent}%: slight {pull:?}");
        let pop = launch_after(fx(11, 10), percent, true);
        assert!(
            pop.y > Fx::ZERO && pop.x > Fx::ZERO,
            "{percent}%: jab 2 up and out {pop:?}"
        );
    }
}

#[test]
fn either_jab_can_be_cancelled_into_any_action_from_its_continuable_frame() {
    // The previous version's jab cancel (owner decision 70): from jab 1's frame 7 and jab 2's frame 8 a jump, a shield, a grab, a
    // dash, a crouch, a tilt or a special starts at once; a frame earlier it is still the jab.
    let jump = sim_core::input::buttons::JUMP;
    let shield = sim_core::input::buttons::SHIELD;
    let special = sim_core::input::buttons::SPECIAL;
    let cases: [(u16, i8, i8, S); 7] = [
        (jump, 0, 0, S::JumpSquat),
        (shield, 0, 0, S::Shield),
        (GRAB, 0, 0, S::Attack),
        (0, 127, 0, S::Dash),
        (0, 0, -127, S::Crouch),
        (ATTACK, 0, -90, S::Attack),
        (special, 0, 0, S::Attack),
    ];
    let jab = MoveId::Jab as u8;
    let jab2 = MoveId::Jab2 as u8;
    for both in [false, true] {
        // (Jab 2 pressed on jab 1's frame 7 starts on tick 7: its frame 8 is tick 14.)
        let cancel = if both { 14u32 } else { 7 };
        for (button, x, y, want) in cases {
            for early in [true, false] {
                let mut sim = setup(0, 0);
                sim.stand(1, Fx::from_int(3), -1);
                let at = if early { cancel - 1 } else { cancel };
                for t in 1..=at {
                    let (b, sx, sy) = if t == 1 || (both && t == 7) {
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

/// The settler's jab 1 at `percent` on a `victim`, then `follow` (from tick 2 on: `(tick, buttons, stick x, stick y)`, any tick not
/// listed neutral): the most hits the victim took in one combo (hits while still in hitstun).
fn combo(victim: u8, percent: i32, follow: &[(u32, u16, i8, i8)]) -> u8 {
    let mut sim = setup(victim, percent);
    let mut best = 0;
    for t in 1..=90u32 {
        let (b, x, y) = if t == 1 {
            (ATTACK, 0, 0)
        } else {
            follow
                .iter()
                .find(|(at, ..)| *at == t)
                .map_or((0, 0, 0), |(_, b, x, y)| (*b, *x, *y))
        };
        sim.tick(inp(x, y, b));
        best = best.max(sim.fighter(1).hits_taken);
    }
    best
}

const JAB2: [(u32, u16, i8, i8); 3] = [(6, ATTACK, 0, 0), (7, ATTACK, 0, 0), (8, ATTACK, 0, 0)];

#[test]
fn jab_1_into_jab_2_is_a_true_combo_at_every_percent() {
    // The 1.1.x jab's whole point: jab 1 pops the foe up a little and jab 2 (pressed from jab 1's frame 7, out on 9) catches it in
    // its hitstun, from 0% to 150% (on the duelist and on the settler's own weight).
    for victim in [0u8, SETTLER] {
        for percent in (0..=150).step_by(10) {
            let n = combo(victim, percent, &JAB2);
            assert!(
                n >= 2,
                "victim {victim} at {percent}%: {n} hits in the combo"
            );
        }
    }
}

/// The tick the settler can first act after jab 1 alone (its cancel on frame 7, owner decision 70), or after jab 1 into jab 2
/// (`both`: jab 2 pressed on jab 1's frame 7, so its frame 8 is tick 14).
fn free_tick(both: bool) -> u32 {
    if both {
        14
    } else {
        7
    }
}

/// Plays jab 1 (and jab 2 when `both`) at `percent` on a `victim` standing at `at`, up to the tick before the settler can act.
fn jab_until_free(victim: u8, percent: i32, both: bool, at: Fx) -> Sim {
    let mut sim = setup(victim, percent);
    sim.stand(1, at, -1);
    for t in 1..free_tick(both) {
        let b = if t == 1 || (both && t == 7) {
            ATTACK
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
    }
    sim
}

/// Frames the victim is still in hitstun (its hitlag included) on the first frame the settler can act after jab 1 alone, or after
/// jab 1 into jab 2 (`both`), with the jab cancel: the frame advantage on hit, 0 if the victim is already free.
fn advantage(victim: u8, percent: i32, both: bool) -> i32 {
    let sim = jab_until_free(victim, percent, both, fx(11, 10));
    let v = sim.fighter(1);
    if v.state == S::Hitstun {
        i32::from(v.hitstun) + i32::from(v.hitlag)
    } else {
        0
    }
}

#[test]
fn jab_1_into_jab_2_leaves_time_for_a_frame_6_move_from_130_percent() {
    // The source (a forum analysis of the previous version with the Kurogane calculator): "at 130% a frame-perfect jab 1 > jab 2
    // > axe is guaranteed against any character" (the axe comes out on frame 6). Here: on the duelist and on the settler, from 130%
    // the victim is still in hitstun for at least 6 frames when the settler can act after jab 2 (its cancel on frame 8).
    for victim in [0u8, SETTLER] {
        for percent in [130, 140, 150] {
            let a = advantage(victim, percent, true);
            assert!(a >= 6, "victim {victim} at {percent}%: {a} frames");
        }
    }
}

/// A follow-up after the jab: `presses` are `(ticks after the settler can act, buttons, stick x, stick y)`, the stick held from the
/// first of them on.
pub type Follow = &'static [(u32, u16, i8, i8)];

/// Jab 1 (and jab 2 when `both`) at `percent` on a `victim` standing at `at`, cancelled on its first frame into `follow`. Returns
/// `Some(true)` if the follow-up's first hit landed while the victim was still in hitstun from the jab (a true combo), `Some(false)`
/// if it landed later, `None` if it missed.
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

/// The follow-ups the owner named (decision 70): the tilts, the grab, a jump into a rising neutral air (the jump squat is 3 frames).
const UP_TILT: Follow = &[(0, ATTACK, 0, 90)];
const FORWARD_TILT: Follow = &[(0, ATTACK, 90, 0)];
const DOWN_TILT: Follow = &[(0, ATTACK, 0, -90)];
const GRAB_IT: Follow = &[(0, GRAB, 0, 0)];
const JUMP_NAIR: Follow = &[(0, sim_core::input::buttons::JUMP, 0, 0), (3, ATTACK, 0, 0)];
const JAB_2: Follow = &[(0, ATTACK, 0, 0)];

#[test]
fn the_jab_keeps_going_as_one_combo_at_low_percents() {
    // Mashed, the 1.1.x jab repeats (jab 1, jab 2, jab 1, ...: the version's endless jab with no finisher, each next hit from the
    // continuable frame); at low percents each hit lands while the last one's hitstun lasts (the duelist, from 0%).
    let mut sim = setup(0, 0);
    let mut most = 0;
    for t in 0..60 {
        sim.tick(inp(0, 0, if t % 2 == 0 { ATTACK } else { 0 }));
        most = most.max(sim.fighter(1).hits_taken);
    }
    assert!(most >= 3, "{most} hits in one combo");
}

#[test]
fn the_cancelled_jab_strings_into_tilts_grabs_and_aerials() {
    // The jab cancel's use (ssbwiki.com `Jab_cancel`: "take advantage of the hitstun and low knockback of the first hits"), on the
    // duelist: out of jab 1 > jab 2 cancelled on jab 2's frame 8, every follow-up the owner named (the tilts, the grab, a jump into a
    // rising neutral air) lands in jab 2's hitstun at every percent 0-150; out of jab 1 alone (cancelled on frame 7) the up tilt is a
    // true combo at 10-20%, the forward tilt at 20-120%, a jump into a neutral air at 10-60%, jab 2 always.
    for percent in (0..=150).step_by(10) {
        for f in [JAB_2, UP_TILT, FORWARD_TILT, DOWN_TILT, GRAB_IT, JUMP_NAIR] {
            assert_eq!(
                follow_up(0, percent, true, f),
                Some(true),
                "{percent}% {f:?}"
            );
        }
        assert_eq!(
            follow_up(0, percent, false, JAB_2),
            Some(true),
            "{percent}%"
        );
        let true_combo = |f: Follow| follow_up(0, percent, false, f) == Some(true);
        assert_eq!(
            true_combo(UP_TILT),
            (10..=20).contains(&percent),
            "{percent}% up tilt"
        );
        assert_eq!(
            true_combo(FORWARD_TILT),
            (20..=120).contains(&percent),
            "{percent}% forward tilt"
        );
        assert_eq!(
            true_combo(JUMP_NAIR),
            (10..=60).contains(&percent),
            "{percent}% neutral air"
        );
    }
}

#[test]
fn jab_1s_pull_at_the_tip_keeps_the_foe_in_reach_of_jab_2() {
    // Owner decision 70: at the far glove's reach the pull draws the foe in, and jab 2 (out on jab 1's frame 9) still reaches it.
    for percent in (0..=150).step_by(30) {
        let hit = follow_up_at(0, percent, false, tip(), JAB_2);
        assert!(hit.is_some(), "{percent}%: jab 2 reaches the pulled foe");
    }
}

#[test]
fn print_the_jabs_frame_advantage_and_follow_ups() {
    // A table for the implementation report (run with --nocapture): frame advantage on the first frame the settler can act (the jab
    // cancel: jab 1's frame 7, jab 2's frame 8) after jab 1 and after jab 1 > 2 on the duelist (weight 90), and each follow-up out of
    // jab 1, out of jab 1 > 2, and out of jab 1 at the tip (the pull): x a true combo, o it lands after the hitstun, . a miss. In order:
    // jab 2, up tilt, forward tilt, down tilt, grab, jump > neutral air.
    let cell = |r: Option<bool>| match r {
        Some(true) => 'x',
        Some(false) => 'o',
        None => '.',
    };
    let follows = [JAB_2, UP_TILT, FORWARD_TILT, DOWN_TILT, GRAB_IT, JUMP_NAIR];
    for percent in (0..=150).step_by(10) {
        let a1 = advantage(0, percent, false);
        let a2 = advantage(0, percent, true);
        let after = |both: bool, at: Fx| -> String {
            follows
                .iter()
                .map(|f| cell(follow_up_at(0, percent, both, at, f)))
                .collect()
        };
        eprintln!(
            "{percent:>3}%  jab1 {a1:>3}  jab1>2 {a2:>3}  after 1: {}  after 1>2: {}  after 1 at the tip: {}",
            after(false, fx(11, 10)),
            after(true, fx(11, 10)),
            after(false, tip()),
        );
    }
}
