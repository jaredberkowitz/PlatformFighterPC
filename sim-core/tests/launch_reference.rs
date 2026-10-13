//! Launches and fast falls as in the reference game (sim v39): the speed-up of strong launches ("balloon" knockback) gives the same
//! real hitstun as the reference's own formula, skipping whole knockback frames (gravity included); hitstun can be cancelled only
//! once the launch has slowed; a tumbling fighter cannot fast fall until it acts; an aerial ends a fast fall.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::{ATTACK, SHIELD};
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// Launches player 0 high in the air with `kb` knockback at `degrees`, through one frame of hitlag (so the launch is applied as a
/// hit's would be). Returns the sim on the first frame of the launch.
fn launch(kb: i32, degrees: u16) -> Sim {
    let mut sim = Sim::alone();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    let f = &mut sim.state.fighters[0];
    f.hitstun = (kb * 4 / 10 - 1) as u16;
    f.launch_pending = true;
    f.launch_kb = Fx::from_int(kb);
    f.launch_angle = (4096 * u32::from(degrees) / 360) as u16;
    f.state = S::Hitstun;
    f.state_frame = 0;
    f.hitlag = 1;
    sim.tick(inp(0, 0, 0));
    sim
}

/// Frames until a launched fighter is free.
fn free_after(kb: i32) -> u32 {
    let mut sim = launch(kb, 45);
    let mut t = 0;
    while sim.f().state == S::Hitstun && t < 400 {
        sim.tick(inp(0, 0, 0));
        t += 1;
    }
    t
}

#[test]
fn strong_launches_end_as_soon_as_the_reference_says() {
    // The reference's real first actionable frames with the speed-up (worked out with its own formula: the reference knockback calculator's
    // port of the game's code), against the plain 0.4 * knockback.
    for (kb, faf) in [
        (60, 24),
        (90, 34),
        (100, 36),
        (120, 39),
        (140, 41),
        (160, 44),
        (200, 50),
    ] {
        assert_eq!(free_after(kb) + 1, faf, "knockback {kb}");
    }
}

#[test]
fn the_speed_up_skips_whole_frames_gravity_and_all() {
    // A sped-up launch follows the path of the same launch played one frame at a time: after the same number of knockback frames
    // the fighter is at the same height. Here a launch straight sideways is compared with gravity included.
    let mut fast = launch(160, 0);
    let mut slow = launch(160, 0);
    slow.state.fighters[0].balloon_faf = 0;
    // Play the fast one a few frames and count the knockback frames it used.
    let start = fast.f().balloon_left;
    for _ in 0..4 {
        fast.tick(inp(0, 0, 0));
    }
    let used = (start - fast.f().balloon_left).floor_int() as usize;
    assert!(
        used > 8,
        "the launch is sped up ({used} knockback frames in 4)"
    );
    let ceil = |x: Fx| (x + Fx::from_raw(65535)).floor_int();
    let used = (ceil(start) - ceil(fast.f().balloon_left)) as usize;
    for _ in 0..used {
        slow.tick(inp(0, 0, 0));
    }
    let (a, b) = (fast.f().pos, slow.f().pos);
    assert!(
        (a.x - b.x).abs() < Fx::from_ratio(1, 100) && (a.y - b.y).abs() < Fx::from_ratio(1, 100),
        "{a:?} vs {b:?}"
    );
}

#[test]
fn hitstun_can_be_cancelled_only_once_the_launch_has_slowed() {
    // A very hard launch (straight up, so it stays in the tall test blast zone) is still fast at frame 40: no air dodge out of it.
    let mut sim = launch(300, 90);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().kb_vel.length() >= sim.content.rules.hitstun_dodge_cancel_speed);
    sim.tick(inp(0, 0, SHIELD));
    assert_ne!(sim.f().state, S::AirDodge, "too fast to cancel");
}

#[test]
fn a_tumbling_fighter_cannot_fast_fall_until_it_acts() {
    let mut sim = launch(120, 80);
    while sim.f().state == S::Hitstun {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().damage_fall, "still tumbling after the hitstun");
    // Falling, a hard down does nothing.
    while sim.f().vel.y > Fx::ZERO {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, -127, 0));
    assert!(!sim.f().fast_fall);
    // An aerial ends the tumble; after it the fighter can fast fall again.
    sim.tick(inp(0, 0, ATTACK));
    assert!(!sim.f().damage_fall);
}

#[test]
fn an_aerial_ends_a_fast_fall() {
    let mut sim = Sim::alone();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall);
    sim.tick(inp(0, 0, ATTACK));
    assert_eq!(sim.f().state, S::Attack);
    assert!(!sim.f().fast_fall, "the aerial ends the fast fall");
    let max_fall = sim.content.fighters[0].max_fall_speed;
    assert!(sim.f().vel.y >= -max_fall, "back to the normal top speed");
    // A fresh press during the aerial fast falls again.
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall, "fast falls again during the aerial");
}

/// Launches player 0 hard enough to tumble, waits out the hitstun high up, then lets it fall to the stage below, giving `input` on
/// the last few frames before it lands. Returns the state it lands in.
fn tumble_down(act_first: bool, tech: bool) -> S {
    let mut sim = launch(120, 80);
    while sim.f().state == S::Hitstun {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().damage_fall);
    if act_first {
        // A midair jump ends the tumble.
        sim.tick(inp(0, 0, sim_core::input::buttons::JUMP));
        assert!(!sim.f().damage_fall);
    }
    // Drop it just above the stage, falling.
    let top = sim.content.stage.platforms[0].y;
    let x = (sim.content.stage.platforms[0].left + sim.content.stage.platforms[0].right)
        / Fx::from_int(2);
    let f = &mut sim.state.fighters[0];
    f.pos.x = x;
    f.pos.y = top + Fx::ONE;
    f.vel = sim_core::vec2::Vec2::new(Fx::ZERO, -Fx::from_ratio(1, 4));
    for t in 0..30 {
        let b = if tech && t == 0 { SHIELD } else { 0 };
        sim.tick(inp(0, 0, b));
        if sim.f().grounded() {
            return sim.f().state;
        }
    }
    panic!("never landed");
}

#[test]
fn landing_while_still_tumbling_is_a_missed_tech_unless_teched() {
    assert_eq!(
        tumble_down(false, false),
        S::Knockdown,
        "a missed tech lies down"
    );
    assert_ne!(
        tumble_down(false, true),
        S::Knockdown,
        "a shield press just before is a tech"
    );
    assert_ne!(
        tumble_down(true, false),
        S::Knockdown,
        "a fighter that acted lands normally"
    );
}
