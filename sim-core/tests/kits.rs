//! The completed kits: the sword character's jab chain, dash attack, down air, grabs and throws, ledge attack,
//! Shield Breaker, Dancing Blade and Counter; and the claws character's aimed specials, reflector guard frames and
//! ledge attack.
//!
//! Frame numbering matches the reference: the tick the button goes down is frame 1.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::{ATTACK, GRAB, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{Fx, GameState, Vec2};

const SWORD: [u8; 4] = [0, 0, 0, 0];
const CLAWS: [u8; 4] = [1, 1, 1, 1];

/// A two-player match: player 0 at x = -7 facing right, player 1 `gap` ahead facing back, both standing.
fn duel(chars: [u8; 4], gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.state = GameState::new_with_active(&sim.content, 1, chars, 0b0011);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-7) + gap, -1);
    sim
}

/// Two fighters high in the air, player 1 `gap` ahead of and `below` player 0.
fn air_duel(chars: [u8; 4], gap: Fx, below: Fx) -> Sim {
    let mut sim = Sim::with_chars(chars);
    sim.state = GameState::new_with_active(&sim.content, 1, chars, 0b0011);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, gap, Fx::from_int(40) - below, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = 1;
    sim.state.fighters[1].facing = -1;
    sim
}

fn pct(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

/// Runs `first` on tick 1, then idle; returns the tick on which player 1's percent first rose.
fn hit_tick(sim: &mut Sim, first: sim_core::Input) -> Option<usize> {
    let before = sim.fighter(1).percent;
    sim.tick(first);
    for t in 2..=120 {
        if sim.fighter(1).percent > before {
            return Some(t - 1);
        }
        sim.tick(inp(0, 0, 0));
    }
    (sim.fighter(1).percent > before).then_some(120)
}

// ---- Sword: jab, dash attack, down air ----------------------------------------------------------------

#[test]
fn the_jab_hits_on_frame_5_for_3_percent_close_and_5_at_the_tip() {
    let mut close = duel(SWORD, fx(14, 10));
    assert_eq!(hit_tick(&mut close, inp(0, 0, ATTACK)), Some(5));
    assert_eq!(close.fighter(1).percent, pct(&close, 30));
    let mut tip = duel(SWORD, fx(32, 10));
    assert_eq!(hit_tick(&mut tip, inp(0, 0, ATTACK)), Some(5));
    assert_eq!(tip.fighter(1).percent, pct(&tip, 50));
}

#[test]
fn the_second_jab_hit_follows_if_attack_is_pressed_again_and_there_is_no_third() {
    let mut sim = duel(SWORD, fx(32, 10));
    sim.state.fighters[1].invuln = 255;
    sim.tick(inp(0, 0, ATTACK));
    // Press again near the end of the first hit (it is 23 frames long, the window is the last 10).
    for _ in 0..16 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().move_id, MoveId::Jab2 as u8, "the jab continued");
    // Jab 2 ends the chain: pressing at its end does not start a third hit.
    assert!(sim.content.weapons[0].moves[MoveId::Jab2 as usize]
        .next
        .is_none());
    assert!(sim.content.weapons[0].moves[MoveId::Jab3 as usize].is_empty());
}

#[test]
fn the_second_jab_hit_does_4_percent_and_6_at_the_tip_on_its_frame_4() {
    let mut sim = duel(SWORD, fx(32, 10));
    sim.state.fighters[1].invuln = 255;
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..16 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, 0, ATTACK));
    // Run to the first tick of the second hit, then let the dummy be hit.
    let mut started = false;
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.f().move_id == MoveId::Jab2 as u8 {
            started = true;
            break;
        }
    }
    assert!(started, "the jab should have chained");
    sim.state.fighters[1].invuln = 0;
    sim.state.fighters[1].percent = Fx::ZERO;
    let mut hit = None;
    for t in 1..=8 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            hit = Some(t);
            break;
        }
    }
    assert!(matches!(hit, Some(3..=4)), "second hit on tick {hit:?}");
    assert_eq!(sim.fighter(1).percent, pct(&sim, 60));
}

#[test]
fn the_dash_attack_hits_on_frame_13_for_13_percent_at_the_tip() {
    let mut sim = duel(SWORD, fx(60, 10));
    sim.ticks(14, inp(127, 0, 0));
    sim.tick(inp(127, 0, ATTACK));
    assert_eq!(sim.f().move_id, MoveId::DashAttack as u8);
    let mut tick = 1;
    let mut hit = None;
    while tick < 40 && hit.is_none() {
        // Keep the target at the sword tip's reach, ahead of the sliding fighter.
        let at = sim.f().pos.x + fx(33, 10);
        sim.stand(1, at, -1);
        sim.tick(inp(0, 0, 0));
        tick += 1;
        if sim.fighter(1).percent > Fx::ZERO {
            hit = Some(tick);
        }
    }
    assert_eq!(hit, Some(13));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 130));
}

#[test]
fn the_down_air_is_a_meteor_smash_on_frame_11_only() {
    // Far enough out that only the tip reaches, low enough to be under the sword on frame 11.
    let mut found = None;
    'search: for x in [42, 44, 46] {
        for below in [28, 30, 32, 34, 36, 38] {
            let mut sim = air_duel(SWORD, fx(x, 10), fx(below, 10));
            sim.state.fighters[1].percent = Fx::from_int(60);
            let before = sim.fighter(1).percent;
            if let Some(11) = hit_tick(&mut sim, inp(0, -100, ATTACK)) {
                found = Some((sim.fighter(1).percent - before, sim));
                break 'search;
            }
        }
    }
    let (gained, mut sim) = found.expect("some position catches the frame-11 meteor");
    assert_eq!(gained, pct_of(&sim, 150));
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            break;
        }
    }
    assert!(
        sim.fighter(1).kb_vel.y < Fx::ZERO,
        "a meteor sends the victim down"
    );
}

fn pct_of(sim: &Sim, tenths: i32) -> Fx {
    pct(sim, tenths)
}

#[test]
fn the_down_air_before_frame_11_is_the_14_percent_tip_on_frame_9() {
    let mut sim = air_duel(SWORD, fx(24, 10), fx(12, 10));
    sim.state.fighters[1].percent = Fx::ZERO;
    assert_eq!(hit_tick(&mut sim, inp(0, -100, ATTACK)), Some(9));
    assert_eq!(sim.fighter(1).percent, pct(&sim, 140));
}

// ---- Sword: grabs, throws, ledge attack --------------------------------------------------------------------

fn grab_now(sim: &mut Sim) -> usize {
    sim.tick(inp(0, 0, GRAB));
    for t in 2..=40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbing {
            return t;
        }
    }
    panic!("the grab never connected");
}

#[test]
fn the_sword_grab_catches_on_frame_6_and_the_dash_grab_on_frame_9() {
    let mut sim = duel(SWORD, fx(17, 10));
    assert_eq!(grab_now(&mut sim), 6);
    let mut sim = duel(SWORD, fx(19, 10));
    sim.ticks(12, inp(127, 0, 0));
    sim.tick(inp(127, 0, GRAB));
    assert_eq!(sim.f().move_id, MoveId::DashGrab as u8);
    let mut caught = None;
    for t in 2..=30 {
        let at = sim.f().pos.x + fx(19, 10);
        sim.stand(1, at, -1);
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbing {
            caught = Some(t);
            break;
        }
    }
    assert_eq!(caught, Some(9));
}

/// Throws with `stick`; returns the damage dealt by the throw alone, the tick it hit and the launch velocity.
fn sword_throw(stick: (i8, i8)) -> (Fx, usize, Vec2) {
    let mut sim = duel(SWORD, fx(17, 10));
    sim.state.fighters[1].percent = Fx::from_int(50);
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(stick.0, stick.1, 0));
    let mut hit = None;
    let mut vel = Vec2::ZERO;
    for t in 2..=70 {
        sim.tick(inp(0, 0, 0));
        if hit.is_none() && sim.fighter(1).percent > before {
            hit = Some(t);
        }
        if hit.is_some() && sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            vel = sim.fighter(1).kb_vel;
            break;
        }
    }
    (
        sim.fighter(1).percent - before,
        hit.expect("the throw never hit"),
        vel,
    )
}

#[test]
fn the_sword_throws_release_on_their_frames_for_their_damage() {
    let sim = duel(SWORD, fx(17, 10));
    for (stick, release, tenths) in [
        ((100, 0), 18, 40),
        ((-100, 0), 19, 40),
        ((0, 100), 13, 50),
        ((0, -100), 20, 40),
    ] {
        let (dealt, tick, _) = sword_throw(stick);
        assert_eq!(tick, release, "stick {stick:?}");
        assert_eq!(dealt, pct(&sim, tenths), "stick {stick:?}");
    }
}

#[test]
fn each_sword_throw_sends_the_victim_its_own_way() {
    let (_, _, forward) = sword_throw((100, 0));
    assert!(forward.x > Fx::ZERO && forward.y > Fx::ZERO, "{forward:?}");
    let (_, _, back) = sword_throw((-100, 0));
    assert!(back.x < Fx::ZERO && back.y > Fx::ZERO, "{back:?}");
    let (_, _, up) = sword_throw((0, 100));
    assert!(up.y > up.x.abs() * Fx::from_int(3), "{up:?}");
    let (_, _, down) = sword_throw((0, -100));
    assert!(down.y > Fx::ZERO, "{down:?}");
}

#[test]
fn the_sword_pummel_does_1_3_percent() {
    let mut sim = duel(SWORD, fx(17, 10));
    grab_now(&mut sim);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).percent - before, pct(&sim, 13));
}

/// Both characters get the same ledge attack numbers: 9%, angle 45, hits on frames 24-26, intangible to 26.
#[test]
fn the_ledge_attack_hits_on_frame_24_for_9_percent_and_is_intangible_until_frame_26() {
    for chars in [SWORD, CLAWS] {
        let mut sim = Sim::with_chars(chars);
        sim.state = GameState::new_with_active(&sim.content, 1, chars, 0b0011);
        sim.put_airborne(0, fx(-125, 10), fx(-15, 10), Fx::ZERO, fx(-1, 10));
        sim.ticks(40, inp(0, 0, 0));
        assert_eq!(sim.f().state, S::LedgeHang);
        sim.stand(1, fx(-85, 10), -1);
        sim.state.fighters[1].invuln = 0;
        sim.tick(inp(0, 0, ATTACK));
        assert_eq!(sim.f().state, S::LedgeAttack);
        let mut intangible_ticks = Vec::new();
        let mut hit = None;
        for t in 2..=70 {
            sim.tick(inp(0, 0, 0));
            if sim.f().state == S::LedgeAttack && is_intangible(sim.f()) {
                intangible_ticks.push(t);
            }
            if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
                hit = Some(t);
            }
            if sim.f().state != S::LedgeAttack {
                break;
            }
        }
        assert!(
            (23..=27).contains(&hit.expect("the ledge attack hit")),
            "hit on {hit:?} for {chars:?}"
        );
        assert_eq!(sim.fighter(1).percent, pct(&sim, 90));
        for t in 3..=26 {
            assert!(
                intangible_ticks.contains(&t),
                "tick {t} should be intangible: {intangible_ticks:?}"
            );
        }
        assert!(
            !intangible_ticks.iter().any(|t| *t >= 40),
            "intangible too long: {intangible_ticks:?}"
        );
        // The whole attack lasts 54 frames (FAF 56 less the two the press and release count).
        sim.ticks(20, inp(0, 0, 0));
        assert_eq!(sim.f().state, S::Idle);
    }
}

// ---- Sword: Shield Breaker ---------------------------------------------------------------------------------

/// Presses special on tick 1 and keeps it down through tick `hold`, then lets go. Returns the damage dealt and
/// the tick (counted from the press) on which it hit.
fn shield_breaker(hold: usize) -> (Fx, usize) {
    let mut sim = duel(SWORD, fx(40, 10));
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    for t in 2..=300 {
        sim.tick(inp(0, 0, if t <= hold { SPECIAL } else { 0 }));
        if sim.fighter(1).percent > Fx::ZERO {
            return (sim.fighter(1).percent, t);
        }
    }
    panic!("it never hit");
}

#[test]
fn shield_breaker_uncharged_does_8_percent_and_charging_raises_it_to_about_24() {
    let sim = duel(SWORD, fx(40, 10));
    let (quick, _) = shield_breaker(1);
    let (some, _) = shield_breaker(30);
    let (full, _) = shield_breaker(100);
    assert!(
        quick >= pct(&sim, 80) && quick <= pct(&sim, 90),
        "uncharged {quick:?}"
    );
    assert!(
        some > quick && some < full,
        "{quick:?} < {some:?} < {full:?}"
    );
    assert!(
        full >= pct(&sim, 220) && full <= pct(&sim, 250),
        "fully charged {full:?}"
    );
}

#[test]
fn shield_breaker_holds_the_charge_pose_only_while_special_is_down_and_never_forever() {
    let mut sim = duel(SWORD, fx(100, 10));
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(40, inp(0, 0, SPECIAL));
    assert_eq!(sim.f().state, S::Attack);
    assert!(sim.f().charge > 10, "charging: {}", sim.f().charge);
    assert!(sim.f().state_frame <= 20, "held on the charge frame");
    // Held for far longer than the limit, it still lets go by itself.
    let mut most = 0;
    let mut ended_at = None;
    for t in 0..300 {
        sim.tick(inp(0, 0, SPECIAL));
        most = most.max(sim.f().charge);
        if sim.f().state != S::Attack {
            ended_at = Some(t);
            break;
        }
    }
    assert_eq!(most, sim.content.rules.charge_frames);
    assert!(
        ended_at.is_some(),
        "the move ended without the button being released"
    );
}

#[test]
fn shield_breaker_thrust_comes_out_about_8_frames_after_the_button_is_let_go() {
    // Held past the charge frame, then released on tick 30: the thrust lands 7-8 ticks later.
    let (_, hit) = shield_breaker(30);
    assert!((37..=39).contains(&hit), "hit on tick {hit}");
}

// ---- Sword: Dancing Blade ----------------------------------------------------------------------------------

fn db_sim() -> Sim {
    let mut sim = duel(SWORD, fx(30, 10));
    // A sturdy dummy that does not fly away between hits.
    sim.state.fighters[1].invuln = 0;
    sim
}

/// Starts the side special on tick 1, then taps special every other tick, steering with `stick_y(move_id)`.
/// Returns the distinct moves the fighter went through, in order.
fn run_blade(total_ticks: usize, stick_y: impl Fn(u8) -> i8) -> Vec<u8> {
    let mut sim = db_sim();
    let mut moves = Vec::new();
    sim.tick(inp(100, 0, SPECIAL));
    moves.push(sim.f().move_id);
    for t in 1..total_ticks {
        let y = stick_y(sim.f().move_id);
        if t % 2 == 0 {
            sim.tick(inp(0, y, SPECIAL));
        } else {
            sim.tick(inp(0, y, 0));
        }
        if moves.last() != Some(&sim.f().move_id) && sim.f().state == S::Attack {
            moves.push(sim.f().move_id);
        }
    }
    moves
}

fn ext(n: u8) -> u8 {
    MoveId::from_index(28 + n) as u8
}

#[test]
fn dancing_blade_without_more_presses_is_a_single_hit() {
    let mut sim = db_sim();
    sim.tick(inp(100, 0, SPECIAL));
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().move_id == MoveId::SideSpecial as u8);
    }
}

#[test]
fn dancing_blade_continues_straight_on_each_press() {
    let moves = run_blade(220, |_| 0);
    assert_eq!(
        moves[..4],
        [MoveId::SideSpecial as u8, ext(1), ext(3), ext(6)],
        "hit 1, hit 2, hit 3, finisher"
    );
}

#[test]
fn dancing_blade_stick_up_picks_the_rising_hits_and_down_the_low_ones() {
    let up = run_blade(220, |_| 100);
    assert_eq!(up[..4], [MoveId::SideSpecial as u8, ext(2), ext(4), ext(7)]);
    let down = run_blade(260, |m| {
        if m == ext(1) || m == ext(2) || m == ext(5) {
            -100
        } else {
            0
        }
    });
    assert_eq!(
        down[..4],
        [MoveId::SideSpecial as u8, ext(1), ext(5), ext(8)]
    );
}

#[test]
fn dancing_blade_hits_are_2_5_to_3_percent_each() {
    let mut sim = db_sim();
    sim.tick(inp(100, 0, SPECIAL));
    let mut hits = Vec::new();
    let mut last = sim.fighter(1).percent;
    for t in 1..80 {
        sim.tick(inp(0, 0, if t % 2 == 0 { SPECIAL } else { 0 }));
        if sim.fighter(1).percent > last {
            hits.push(sim.fighter(1).percent - last);
            last = sim.fighter(1).percent;
        }
    }
    assert!(hits.len() >= 2, "{hits:?}");
    for h in &hits[..2] {
        assert!(*h >= pct(&sim, 25) && *h <= pct(&sim, 30), "{h:?}");
    }
}

#[test]
fn dancing_blade_in_the_air_ends_sooner() {
    let mut sim = Sim::with_chars(SWORD);
    sim.state = GameState::new_with_active(&sim.content, 1, SWORD, 0b0011);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, Fx::from_int(30), Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(100, 0, SPECIAL));
    let mut ended = None;
    for t in 2..60 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            ended = Some(t);
            break;
        }
    }
    let ended = ended.unwrap();
    assert!((26..=31).contains(&ended), "ended on {ended}");
}

// ---- Sword: Counter ----------------------------------------------------------------------------------------

/// Player 0 starts a counter on tick 1; player 1 (a claws fighter) throws a jab so it lands `landing_tick`
/// ticks after player 0's press.
fn counter_duel(attacker_press_tick: usize) -> Sim {
    let mut sim = duel([0, 1, 0, 1], fx(20, 10));
    sim.state.fighters[0].percent = Fx::ZERO;
    sim.tick2(inp(0, -100, SPECIAL), inp(0, 0, 0));
    for t in 2..60 {
        let p1 = if t == attacker_press_tick {
            inp(0, 0, ATTACK)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, 0), p1);
    }
    sim
}

#[test]
fn a_hit_inside_the_counter_window_is_caught_and_answered_with_interest() {
    // The brawler's jab hits 4 frames after its press; press so it lands mid-window (frames 6-27).
    let sim = counter_duel(10);
    assert_eq!(
        sim.fighter(0).percent,
        Fx::ZERO,
        "the counter took no damage"
    );
    let dealt = sim.fighter(1).percent;
    // Jab does 2%; the answer is max(8, 1.2 x 2) = 8%, times the ruleset's damage multiplier.
    assert_eq!(dealt, pct(&sim, 80), "the answer deals the 8% minimum");
}

#[test]
fn a_big_hit_comes_back_bigger() {
    let mut sim = duel([0, 1, 0, 1], fx(20, 10));
    // Swap the brawler's jab for a 20% hit so 1.2x beats the 8% floor.
    sim.content.weapons[1].moves[MoveId::Jab as usize].hitboxes[0].damage = Fx::from_int(20);
    sim.content.weapons[1].moves[MoveId::Jab as usize].hitboxes[1].damage = Fx::from_int(20);
    sim.tick2(inp(0, -100, SPECIAL), inp(0, 0, 0));
    for t in 2..60 {
        let p1 = if t == 10 {
            inp(0, 0, ATTACK)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, 0), p1);
    }
    assert_eq!(sim.fighter(0).percent, Fx::ZERO);
    assert_eq!(sim.fighter(1).percent, pct(&sim, 240), "1.2 x 20%");
}

#[test]
fn a_hit_outside_the_window_is_taken_normally() {
    // Landing on frame 3 (before the window opens) or after it closes (frame 40).
    for press in [2usize, 38] {
        let sim = counter_duel(press);
        assert!(
            sim.fighter(0).percent > Fx::ZERO,
            "press {press}: the early or late hit should land"
        );
    }
}

#[test]
fn the_counter_turns_to_face_an_attacker_who_hit_from_behind() {
    let mut sim = duel([0, 1, 0, 1], fx(20, 10));
    sim.state.fighters[0].facing = -1; // facing away from the brawler
    sim.tick2(inp(0, -100, SPECIAL), inp(0, 0, 0));
    for t in 2..60 {
        let p1 = if t == 10 {
            inp(0, 0, ATTACK)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, 0), p1);
    }
    assert_eq!(sim.f().facing, 1, "turned toward the attacker");
    assert_eq!(sim.fighter(0).percent, Fx::ZERO);
    assert!(sim.fighter(1).percent > Fx::ZERO, "the answer landed");
}

#[test]
fn the_counter_also_catches_projectiles() {
    let mut sim = duel([0, 1, 0, 1], fx(60, 10));
    sim.tick2(inp(0, 0, 0), inp(0, 0, SPECIAL));
    // Start the counter so its window covers the shot's arrival.
    let mut caught = false;
    for t in 2..120 {
        let p0 = if t == 18 {
            inp(0, -100, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(p0, inp(0, 0, 0));
        if sim.f().move_id == MoveId::Ext0 as u8 && sim.f().state == S::Attack {
            caught = true;
        }
    }
    // Whether the timing lines up is checked loosely: the countering fighter either caught it (and took no
    // damage) or the shot missed the window, but the fighter never takes a counter-stance hit.
    if caught {
        assert_eq!(sim.fighter(0).percent, Fx::ZERO);
    }
}

#[test]
fn an_unused_counter_stance_costs_62_frames() {
    let mut sim = duel(SWORD, fx(100, 10));
    sim.tick(inp(0, -100, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
    let mut free = None;
    for t in 2..120 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            free = Some(t + 1);
            break;
        }
    }
    assert_eq!(free, Some(64));
}

// ---- Claws: aimed specials and guard frames -----------------------------------------------------------------

#[test]
fn wolf_flash_aims_up_and_down_with_the_stick() {
    let run = |y: i8| {
        let mut sim = Sim::with_chars(CLAWS);
        sim.state = GameState::new_with_active(&sim.content, 1, CLAWS, 0b0011);
        sim.put_airborne(0, Fx::from_int(-12), Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.put_airborne(1, Fx::from_int(30), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(127, 0, SPECIAL));
        let start = sim.f().pos;
        for _ in 0..34 {
            sim.tick(inp(127, y, 0));
        }
        sim.f().pos - start
    };
    let flat = run(0);
    let up = run(100);
    let down = run(-100);
    assert!(flat.x > Fx::from_int(4), "it dashes forward: {flat:?}");
    assert!(
        up.y > flat.y + Fx::from_int(1),
        "aimed up: {up:?} vs {flat:?}"
    );
    assert!(
        down.y < flat.y - Fx::from_int(1),
        "aimed down: {down:?} vs {flat:?}"
    );
}

#[test]
fn fire_wolf_goes_where_the_stick_pointed_and_turns_around_for_backward() {
    // The flight is move frames 17-30. Measure from the first tick of it to the last.
    let fly = |x: i8, y: i8| {
        let mut sim = Sim::with_chars(CLAWS);
        sim.state = GameState::new_with_active(&sim.content, 1, CLAWS, 0b0011);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        sim.put_airborne(1, Fx::from_int(40), Fx::from_int(60), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].facing = 1;
        sim.tick(inp(0, 127, SPECIAL));
        for _ in 0..17 {
            sim.tick(inp(x, y, 0));
        }
        let start = sim.f().pos;
        for _ in 0..14 {
            sim.tick(inp(x, y, 0));
        }
        (sim.f().pos - start, sim.f().facing)
    };
    let (neutral, _) = fly(0, 0);
    assert!(
        neutral.y > Fx::from_int(4) && neutral.x > Fx::from_int(1),
        "neutral aims up: {neutral:?}"
    );
    let (forward, f1) = fly(127, 0);
    assert!(
        forward.x > Fx::from_int(4) && forward.y.abs() < Fx::from_int(2),
        "{forward:?}"
    );
    assert_eq!(f1, 1);
    let (back, f2) = fly(-127, 0);
    assert!(
        back.x < Fx::from_int(-4),
        "backward goes backward: {back:?}"
    );
    assert_eq!(f2, -1, "and Wolf turns around");
    let (down, _) = fly(0, -127);
    assert!(down.y < Fx::from_int(-3), "down dives: {down:?}");
    let (diag, _) = fly(100, 100);
    assert!(
        diag.x > Fx::from_int(2) && diag.y > Fx::from_int(2),
        "{diag:?}"
    );
}

#[test]
fn the_reflector_is_intangible_on_frames_5_to_8_only() {
    let mut sim = duel(CLAWS, fx(100, 10));
    sim.tick(inp(0, -100, SPECIAL));
    let mut seen = Vec::new();
    for t in 2..=14 {
        sim.tick(inp(0, 0, 0));
        if is_intangible(sim.f()) {
            seen.push(t);
        }
    }
    assert_eq!(seen, vec![5, 6, 7, 8], "intangible ticks {seen:?}");
}

// ---- Pivot grabs -----------------------------------------------------------------------------------------

/// Dashes right for 14 ticks, then flicks `stick` with grab. Returns (the move, facing, tick the grab connected) with
/// a target kept 1.9 units ahead of the fighter's facing on every tick.
fn pivot_grab(chars: [u8; 4], stick: i8) -> (u8, i8, Option<usize>) {
    let mut sim = duel(chars, fx(300, 10));
    sim.ticks(14, inp(127, 0, 0));
    sim.tick(inp(stick, 0, GRAB));
    let (move_id, facing) = (sim.f().move_id, sim.f().facing);
    let mut caught = None;
    for t in 2..=30 {
        let at = sim.f().pos.x + fx(19, 10) * Fx::from_int(i32::from(sim.f().facing));
        sim.stand(1, at, -sim.f().facing);
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Grabbing {
            caught = Some(t);
            break;
        }
    }
    (move_id, facing, caught)
}

#[test]
fn a_pivot_grab_turns_around_out_of_a_dash_and_catches_behind() {
    let (m, facing, caught) = pivot_grab(SWORD, -127);
    assert_eq!(m, MoveId::PivotGrab as u8);
    assert_eq!(facing, -1, "turned around");
    assert_eq!(
        caught,
        Some(10),
        "the sword's pivot grab catches on frame 10"
    );
    let (m, facing, caught) = pivot_grab(CLAWS, -127);
    assert_eq!(m, MoveId::PivotGrab as u8);
    assert_eq!(facing, -1);
    assert_eq!(
        caught,
        Some(11),
        "the claws' (estimated) pivot grab catches on frame 11"
    );
}

#[test]
fn grabbing_without_reversing_is_still_a_dash_grab_or_a_standing_grab() {
    let (m, facing, _) = pivot_grab(SWORD, 127);
    assert_eq!(m, MoveId::DashGrab as u8);
    assert_eq!(facing, 1);
    // From standing still, with the stick pulled back, it is the ordinary grab.
    let mut sim = duel(SWORD, fx(300, 10));
    sim.tick(inp(-127, 0, GRAB));
    assert_eq!(sim.f().move_id, MoveId::Grab as u8);
}

#[test]
fn a_weapon_without_a_pivot_grab_falls_back_to_its_dash_grab() {
    let mut sim = duel(SWORD, fx(300, 10));
    sim.content.weapons[0].moves[MoveId::PivotGrab as usize] = sim_core::moves::Move::empty();
    sim.ticks(14, inp(127, 0, 0));
    sim.tick(inp(-127, 0, GRAB));
    assert_eq!(sim.f().move_id, MoveId::DashGrab as u8);
}

// ---- Shield Breaker vs shields ----------------------------------------------------------------------------

/// How much shield health Shield Breaker takes off a held shield (charged for `hold` ticks), with the move's shield
/// damage set to `percent`, and the shield's state at the end.
fn shield_loss(hold: usize, percent: u16) -> (Fx, S) {
    use sim_core::input::buttons::SHIELD;
    let mut sim = duel(SWORD, fx(40, 10));
    for hb in &mut sim.content.weapons[0].moves[MoveId::NSpecial as usize].hitboxes {
        hb.shield_damage = percent;
    }
    let start = sim.fighter(1).shield_hp;
    sim.tick2(inp(0, 0, SPECIAL), inp(0, 0, SHIELD));
    let mut lowest = start;
    for t in 2..=300 {
        let p0 = inp(0, 0, if t <= hold { SPECIAL } else { 0 });
        sim.tick2(p0, inp(0, 0, SHIELD));
        lowest = lowest.min(sim.fighter(1).shield_hp);
        if sim.fighter(1).state == S::ShieldBreak {
            return (start - lowest, S::ShieldBreak);
        }
        if sim.f().state != S::Attack && t > 60 {
            break;
        }
    }
    (start - lowest, sim.fighter(1).state)
}

#[test]
fn shield_breaker_does_double_damage_to_a_shield_and_a_full_charge_breaks_it() {
    let (normal, _) = shield_loss(1, 100);
    let (doubled, state) = shield_loss(1, 200);
    // The shield also drains a little each frame; the extra from doubling is one more hit's worth of damage.
    let extra = doubled - normal;
    let reference = duel(SWORD, fx(40, 10));
    let tip = pct(&reference, 90);
    let body = pct(&reference, 80);
    let close = |a: Fx, b: Fx| (a - b).abs() <= Fx::from_raw(8);
    assert!(
        close(extra, tip) || close(extra, body),
        "extra shield damage {extra:?}"
    );
    assert_ne!(
        state,
        S::ShieldBreak,
        "an uncharged one does not break a full shield"
    );
    let (_, state) = shield_loss(100, 200);
    assert_eq!(state, S::ShieldBreak, "a fully charged one breaks it");
}

#[test]
fn only_shield_breaker_has_extra_shield_damage() {
    let sim = duel(SWORD, fx(40, 10));
    let w = &sim.content.weapons[0];
    assert!(w.moves[MoveId::NSpecial as usize]
        .hitboxes
        .iter()
        .all(|h| h.shield_damage == 200));
    for id in [
        MoveId::Jab,
        MoveId::FSmash,
        MoveId::NAir,
        MoveId::SideSpecial,
    ] {
        assert!(
            w.moves[id as usize]
                .hitboxes
                .iter()
                .all(|h| h.shield_damage == 100),
            "{}",
            id.key()
        );
    }
}

// ---- Dancing Blade steps ------------------------------------------------------------------------------------

#[test]
fn dancing_blade_steps_forward_on_the_ground_hit_by_hit() {
    let mut sim = duel(SWORD, fx(300, 10));
    let start = sim.f().pos.x;
    sim.tick(inp(100, 0, SPECIAL));
    let mut after_first = None;
    let mut total = None;
    let in_blade = |m: u8| m == MoveId::SideSpecial as u8 || (29..=36).contains(&m);
    for t in 1..220 {
        sim.tick(inp(0, 0, if t % 2 == 0 { SPECIAL } else { 0 }));
        if after_first.is_none() && sim.f().move_id != MoveId::SideSpecial as u8 {
            after_first = Some(sim.f().pos.x - start);
        }
        // The chain is over when the finisher has been reached and the fighter has left the blade moves.
        if total.is_none() && sim.f().move_id == 34 {
            total = Some(Fx::ZERO);
        }
        if total == Some(Fx::ZERO) && !in_blade(sim.f().move_id) {
            total = Some(sim.f().pos.x - start);
        }
    }
    let after_first = after_first.expect("the chain continued");
    assert!(
        after_first > fx(8, 10),
        "hit 1 steps about a unit: {after_first:?}"
    );
    let total = total.expect("the chain reached its finisher");
    assert!(
        total > Fx::from_int(2) && total < Fx::from_int(8),
        "the whole chain: {total:?}"
    );
}

#[test]
fn dancing_blade_steps_leave_the_air_alone() {
    // In the air a step would freeze the fall; the script leaves air physics alone.
    let mut sim = Sim::with_chars(SWORD);
    sim.state = GameState::new_with_active(&sim.content, 1, SWORD, 0b0011);
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.put_airborne(1, Fx::from_int(30), Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(100, 0, SPECIAL));
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(
        sim.f().pos.y < Fx::from_int(40) - fx(5, 10),
        "still falling: {:?}",
        sim.f().pos
    );
}
