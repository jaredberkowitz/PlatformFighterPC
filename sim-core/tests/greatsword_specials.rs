//! The Greatsword's specials, phase 2 (the normal versions; sim 85): the wave (ground and air, its tall hit, its fall break, the land
//! switch), the combo slash (each part only on a hit, shields included, and only with a new press), the rising slash (the rise, the
//! second press in 20-26, the falling slash's speed, meteor and landing hit, the ledge rules, the special fall) and the charge (0.3 a
//! frame, its cancels, a hit draining it). Reference data: `docs/movesets/cloud/REFERENCE_DATA.md`; [E] fits are noted per test.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{GRAB, JUMP, SHIELD, SPECIAL};
use sim_core::moves::greatsword_slots as slots;
use sim_core::moves::{
    greatsword, MoveId, Weapon, ANGLE_BY_TRAVEL, ANGLE_VORTEX, TARGETS_AERIAL, TARGETS_GROUNDED,
};
use sim_core::state::FighterState as S;
use sim_core::{Fx, Vec2};

const CHAMPION: u8 = 6;

/// The champion (player 0) standing at 0 facing right and a duelist (player 1) at the far end, the others out.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([CHAMPION, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn sword() -> Weapon {
    greatsword()
}

fn dealt(sim: &Sim, tenths: i32) -> Fx {
    Fx::from_ratio(tenths, 10) * sim.content.rules.damage_mult
}

fn su(n: i32) -> Fx {
    Fx::from_ratio(n, 8000)
}

/// Ticks of the current move from now until the fighter is out of `Attack` and the reference frame that gives (the house's FAF count).
fn run_out(sim: &mut Sim, input: impl Fn(u32) -> sim_core::Input) -> u32 {
    for t in 1..=400u32 {
        sim.tick(input(t));
        if sim.f().state != S::Attack {
            return t + 2;
        }
    }
    panic!("never ended");
}

// ---- The wave (neutral special) ----------------------------------------------------------------------------------------------------

#[test]
fn the_waves_data_is_the_references() {
    let w = sword();
    let ground = &w.moves[MoveId::NSpecial as usize];
    let air = &w.moves[slots::WAVE_AIR as usize];
    assert_eq!(ground.air_version, Some(slots::WAVE_AIR as u8));
    assert_eq!(air.on_land, Some(MoveId::NSpecial as u8));
    for (m, faf, speed, clean, late) in [(ground, 54, 2000, 80, 60), (air, 61, 1600, 64, 48)] {
        assert_eq!(m.total_frames + 2, faf);
        let p = m.projectile.expect("a wave");
        // Out on 17, hitting from 18 to 67.
        assert_eq!((p.frame, p.life), (16, 51));
        assert_eq!(p.speed, su(speed));
        let (first, _) = p.hit_at(1);
        let (later, _) = p.hit_at(19);
        let (still, _) = p.hit_at(18);
        assert_eq!(first.damage, Fx::from_ratio(clean, 10));
        assert_eq!(still.damage, Fx::from_ratio(clean, 10));
        assert_eq!(later.damage, Fx::from_ratio(late, 10));
        for h in [first, later] {
            assert_eq!(
                (h.angle, h.base_knockback, h.knockback_growth),
                (55, 38, 60)
            );
            assert_eq!(h.angle_type, ANGLE_BY_TRAVEL);
            assert_eq!((h.hitlag, h.shield_damage), (50, 50));
            assert_eq!(h.half_height, Fx::from_ratio(465, 800), "the tall wave");
        }
        assert!(!p.pierce);
        // The windbox: grounded fighters only, a push, frames 16-17.
        assert!(m.hitboxes.iter().all(|h| h.push
            && h.targets == TARGETS_GROUNDED
            && (h.start, h.end) == (15, 16)
            && h.fixed_kb == 40
            && !h.clanks));
    }
    assert_eq!((air.start_speed, air.start_speed_y), (40, 65));
    assert_eq!(air.air_gravity, Some(su(60)));
    assert!(!ground.catches_ledges() && !air.catches_ledges());
}

/// The champion fires the grounded wave at a duelist standing `x` world units away; returns the reference frame the duelist was
/// hit on and the damage.
fn wave_at(x: Fx) -> (u32, Fx) {
    let mut sim = duel();
    sim.stand(1, x, -1);
    sim.tick(inp(0, 0, SPECIAL));
    for t in 2..=90u32 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            return (t, sim.fighter(1).percent);
        }
    }
    panic!("the wave never hit");
}

#[test]
fn the_wave_hits_from_frame_18_clean_then_late_from_36() {
    let sim = duel();
    let (near, clean) = wave_at(Fx::from_int(3));
    assert!((18..=21).contains(&near), "{near}");
    assert_eq!(clean, dealt(&sim, 80));
    let (far, late) = wave_at(Fx::from_int(11));
    assert!(far >= 36, "{far}");
    assert_eq!(late, dealt(&sim, 60));
    // It goes about 12.5 world units in its 50 frames, then it is gone.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(30), -1);
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(16, inp(0, 0, 0));
    let shot = sim
        .state
        .projectiles
        .iter()
        .find(|p| p.active)
        .copied()
        .expect("out");
    assert_eq!(shot.vel.x, su(2000));
    sim.ticks(60, inp(0, 0, 0));
    assert!(sim.state.projectiles.iter().all(|p| !p.active), "expired");
}

#[test]
fn the_waves_tall_hit_catches_a_fighter_above_its_middle() {
    // A duelist held 1.4 world units off the ground in the wave's path: the wave's capsule reaches it.
    let mut sim = duel();
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 0..40 {
        sim.put_airborne(1, Fx::from_int(4), fx(14, 10), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            break;
        }
    }
    assert!(sim.fighter(1).percent > Fx::ZERO);
}

#[test]
fn the_wave_returns_control_on_54_grounded_and_61_aerial() {
    let mut sim = duel();
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(run_out(&mut sim, |_| inp(0, 0, 0)), 54);
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, slots::WAVE_AIR as u8);
    assert_eq!(run_out(&mut sim, |_| inp(0, 0, 0)), 61);
}

#[test]
fn the_aerial_wave_breaks_the_fall() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ONE, -Fx::ONE);
    sim.tick(inp(0, 0, SPECIAL));
    let f = sim.f();
    // 0.4 of the sideways and 0.65 of the falling speed (and one frame of its own gravity, 0.06).
    assert!((f.vel.x - fx(4, 10)).abs() <= fx(1, 50), "{:?}", f.vel.x);
    assert!(
        f.vel.y >= -fx(65, 100) - su(60) && f.vel.y < Fx::ZERO,
        "{:?}",
        f.vel.y
    );
}

#[test]
fn landing_before_the_wave_fires_fires_the_grounded_wave() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, fx(3, 10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, slots::WAVE_AIR as u8);
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        if sim.state.projectiles.iter().any(|p| p.active) {
            break;
        }
    }
    assert!(sim.f().grounded());
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8, "the grounded wave");
    let shot = sim
        .state
        .projectiles
        .iter()
        .find(|p| p.active)
        .expect("fired");
    assert_eq!(shot.vel.x, su(2000), "the grounded wave's speed");
    assert_eq!(shot.move_id, MoveId::NSpecial as u8);
}

#[test]
fn the_waves_windbox_pushes_a_grounded_fighter_next_to_it_without_hitting_it() {
    let mut sim = duel();
    sim.stand(1, fx(9, 10), -1);
    sim.tick(inp(0, 0, SPECIAL));
    let mut pushed = false;
    for _ in 0..18 {
        sim.tick(inp(0, 0, 0));
        pushed |= sim.fighter(1).vel.x > Fx::ZERO;
        assert_ne!(sim.fighter(1).state, S::Hitstun);
    }
    assert!(pushed);
}

// ---- The combo slash (side special) ------------------------------------------------------------------------------------------------

/// The champion starts the combo slash with a duelist held in the air in front of it (at its first box each frame) unless `whiff`;
/// `press(t)` says whether special is pressed on tick `t`. Returns the reference frame the champion is free on, the hits' damages in
/// order and the slots it went through.
fn combo(whiff: bool, shield: bool, press: impl Fn(u32) -> bool) -> (u32, Vec<Fx>, Vec<u8>) {
    let mut sim = duel();
    if shield {
        sim.stand(1, fx(13, 10), -1);
    }
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::SideSpecial as u8);
    let mut hits = Vec::new();
    let mut slots_seen = vec![sim.f().move_id];
    let mut last = Fx::ZERO;
    let mut shield_hp = sim.fighter(1).shield_hp;
    for t in 2..=200u32 {
        let me = sim.f().pos;
        if shield {
            sim.state.fighters[1].pos = Vec2::new(me.x + fx(13, 10), Fx::ZERO);
        } else if !whiff {
            let p = sim.state.fighters[1].percent;
            sim.put_airborne(1, me.x + fx(13, 10), me.y + fx(4, 10), Fx::ZERO, Fx::ZERO);
            sim.state.fighters[1].percent = p;
        }
        let p1 = if shield {
            inp(0, 0, SHIELD)
        } else {
            inp(0, 0, 0)
        };
        sim.tick2(inp(0, 0, if press(t) { SPECIAL } else { 0 }), p1);
        if sim.fighter(1).percent > last {
            hits.push(sim.fighter(1).percent - last);
            last = sim.fighter(1).percent;
        }
        if shield && sim.fighter(1).shield_hp < shield_hp {
            hits.push(Fx::ZERO);
            shield_hp = sim.fighter(1).shield_hp;
        }
        if slots_seen.last() != Some(&sim.f().move_id) && sim.f().state == S::Attack {
            slots_seen.push(sim.f().move_id);
        }
        if sim.f().state != S::Attack {
            return (t + 1, hits, slots_seen);
        }
    }
    panic!("never ended");
}

#[test]
fn the_combo_slashes_frame_data_and_hits_are_the_references() {
    let w = sword();
    let one = &w.moves[MoveId::SideSpecial as usize];
    let two = &w.moves[slots::COMBO_2 as usize];
    let three = &w.moves[slots::COMBO_3 as usize];
    assert_eq!(
        (
            one.total_frames + 2,
            two.total_frames + 2,
            three.total_frames + 2
        ),
        (41, 32, 56)
    );
    assert!(one.hitboxes.iter().all(|h| (h.start, h.end) == (9, 10)));
    assert!(two.hitboxes.iter().all(|h| (h.start, h.end) == (1, 2)));
    let windows: Vec<(u8, u8, u8)> = three
        .hitboxes
        .iter()
        .map(|h| (h.start, h.end, h.group))
        .collect();
    assert_eq!(
        windows,
        vec![
            (1, 2, 0),
            (1, 2, 0),
            (10, 11, 1),
            (10, 11, 1),
            (24, 25, 2),
            (24, 25, 2)
        ]
    );
    let angles: Vec<i16> = one.hitboxes.iter().map(|h| h.angle).collect();
    assert_eq!(angles, vec![75, 112, ANGLE_VORTEX]);
    assert!(one.hitboxes.iter().all(|h| h.damage == Fx::from_int(4)));
    assert!(two.hitboxes.iter().all(|h| h.damage == Fx::from_int(3)));
    let row: Vec<(i16, i16, i16, u16)> = three
        .hitboxes
        .iter()
        .map(|h| (h.angle, h.base_knockback, h.knockback_growth, h.fixed_kb))
        .collect();
    assert_eq!(
        row,
        vec![
            (280, 0, 100, 20),
            (70, 0, 100, 20),
            (95, 0, 100, 37),
            (110, 0, 100, 37),
            (50, 30, 140, 0),
            (50, 30, 140, 0)
        ]
    );
}

#[test]
fn the_combo_slash_goes_on_only_when_each_part_connects_and_special_is_pressed_again() {
    let sim = duel();
    // Pressing all the way: all five hits, 19% (4 + 3 + 3 + 3 + 6).
    let (_, hits, seen) = combo(false, false, |t| t % 2 == 0);
    assert_eq!(
        seen,
        vec![
            MoveId::SideSpecial as u8,
            slots::COMBO_2 as u8,
            slots::COMBO_3 as u8
        ]
    );
    let tenths: Vec<Fx> = [40, 30, 30, 30, 60]
        .iter()
        .map(|&d| dealt(&sim, d))
        .collect();
    assert_eq!(hits, tenths);
    // A whiff never carries on, however often special is pressed: slash 1's FAF 41.
    let (free, hits, seen) = combo(true, false, |t| t % 2 == 0 && t < 30);
    assert_eq!((free, hits.len(), seen.len()), (41, 0, 1));
    // A hit without a new press ends there too.
    let (free, hits, _) = combo(false, false, |_| false);
    // (FAF 41 plus the 8 frames of hitlag the 4% hit froze it for: floor(4 x 0.65 + 6).)
    assert_eq!((free, hits.len()), (41 + 8, 1));
    // Slash 2 with no press after it: it ends on its own FAF (32 of its frames).
    let (_, hits, seen) = combo(false, false, |t| t == 5);
    assert_eq!((hits.len(), seen.len()), (2, 2));
}

#[test]
fn a_shielded_slash_carries_on_too() {
    let (_, hits, seen) = combo(false, true, |t| t % 2 == 0);
    assert_eq!(seen.len(), 3, "every part on the shield: {seen:?}");
    assert!(hits.len() >= 3);
}

// ---- The rising slash (up special) -------------------------------------------------------------------------------------------------

#[test]
fn the_rise_climbs_the_fitted_height_and_falls_into_the_special_fall() {
    for (air, k) in [(false, 100), (true, 110)] {
        let mut sim = duel();
        if air {
            sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
        }
        let y0 = sim.f().pos.y;
        sim.tick(inp(0, 127, SPECIAL));
        assert_eq!(sim.f().move_id, MoveId::UpSpecial as u8);
        let mut at19 = Fx::ZERO;
        let mut peak = Fx::MIN;
        let mut helpless_on = 0;
        for t in 2..=70u32 {
            sim.tick(inp(0, 0, 0));
            if t == 19 {
                at19 = sim.f().pos.y - y0;
            }
            peak = peak.max(sim.f().pos.y - y0);
            if sim.f().state == S::Helpless && helpless_on == 0 {
                helpless_on = t;
            }
        }
        // 60 of the reference's units by frame 19 and 66 at the peak, x1.1 from the air (the fit, [E]), give or take the air
        // frames' gravity before the rise.
        let want19 = su(60_000) * Fx::from_ratio(k, 100);
        let want_peak = su(66_000) * Fx::from_ratio(k, 100);
        assert!(
            (at19 - want19).abs() <= fx(3, 10),
            "air {air}: {at19:?} vs {want19:?}"
        );
        assert!((peak - want_peak).abs() <= fx(3, 10), "air {air}: {peak:?}");
        // The animation's 57 frames, then the special fall (the house count: the move's last frame is the switch).
        assert_eq!(helpless_on, 57, "air {air}");
    }
}

#[test]
fn the_quickest_rise_and_fall_lands_on_45_and_is_free_on_71() {
    // ssbwiki.com's quickest loop (the same height): the second press on 20, the falling hits from 31, the landing hit on 45, free on
    // 71. The rise is fitted [E]; the falling speed (4.2) and every frame of the falling slash and its landing are [R].
    let mut sim = duel();
    sim.tick(inp(0, 127, SPECIAL));
    let mut landed_on = 0;
    let mut fall_started = 0;
    let mut free_on = 0;
    for t in 2..=120u32 {
        sim.tick(inp(0, 0, if t == 20 { SPECIAL } else { 0 }));
        if sim.f().move_id == slots::FALL as u8 && fall_started == 0 {
            fall_started = t;
        }
        if sim.f().move_id == slots::FALL_LANDING as u8 && landed_on == 0 {
            landed_on = t;
        }
        if t > 2 && sim.f().state != S::Attack && free_on == 0 {
            free_on = t + 1;
        }
    }
    assert_eq!(fall_started, 20);
    assert_eq!(landed_on, 45);
    assert_eq!(free_on, 71);
}

#[test]
fn the_second_press_counts_only_on_frames_20_to_26() {
    for (press, goes) in [(19u32, false), (20, true), (26, true), (27, false)] {
        let mut sim = duel();
        sim.tick(inp(0, 127, SPECIAL));
        let mut fell = false;
        for t in 2..=40u32 {
            sim.tick(inp(0, 0, if t == press { SPECIAL } else { 0 }));
            fell |= sim.f().move_id == slots::FALL as u8;
        }
        assert_eq!(fell, goes, "pressed on {press}");
    }
}

#[test]
fn the_falling_slash_drops_straight_down_at_4_2_cancelling_drift() {
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), fx(2, 10), Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], slots::FALL);
    sim.ticks(12, inp(127, 0, 0));
    let x = sim.f().pos.x;
    sim.ticks(5, inp(127, 0, 0));
    assert_eq!(sim.f().vel, Vec2::new(Fx::ZERO, -su(4200)));
    assert_eq!(sim.f().pos.x, x, "no drift");
}

#[test]
fn the_falling_slash_meteors_aerial_targets_and_sends_grounded_ones_at_45() {
    let w = sword();
    let fall = &w.moves[slots::FALL as usize];
    assert!(fall
        .hitboxes
        .iter()
        .all(|h| (h.start, h.end) == (11, 60) && !h.clanks));
    let meteor: Vec<_> = fall.hitboxes.iter().filter(|h| h.angle == 262).collect();
    assert!(
        !meteor.is_empty()
            && meteor
                .iter()
                .all(|h| h.targets == TARGETS_AERIAL && h.fixed_kb == 110)
    );
    // An aerial target below the slash goes down.
    let mut sim = duel();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], slots::FALL);
    sim.state.fighters[0].state_frame = 11;
    sim.state.fighters[1].percent = Fx::from_int(60);
    for _ in 0..30 {
        let me = sim.f().pos;
        if sim.fighter(1).state != S::Hitstun {
            let p = sim.fighter(1).percent;
            sim.put_airborne(1, me.x + fx(6, 10), me.y - fx(2, 10), Fx::ZERO, Fx::ZERO);
            sim.state.fighters[1].percent = p;
        }
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Hitstun && !sim.fighter(1).launch_pending {
            break;
        }
    }
    assert!(
        sim.fighter(1).kb_vel.y < Fx::ZERO,
        "{:?}",
        sim.fighter(1).kb_vel
    );
}

#[test]
fn the_falling_slash_lands_into_its_landing_hit_and_26_frames_of_lag() {
    let w = sword();
    let landing = &w.moves[slots::FALL_LANDING as usize];
    assert_eq!(landing.total_frames + 2, 27);
    assert!(landing.hitboxes.iter().all(|h| (h.start, h.end) == (0, 2)
        && h.damage == fx(35, 10)
        && (h.angle, h.base_knockback, h.knockback_growth) == (64, 64, 128)
        && !h.clanks));
    // A duelist standing where it lands takes the landing hit.
    let mut sim = duel();
    sim.stand(1, fx(9, 10), -1);
    sim.put_airborne(0, Fx::ZERO, fx(2, 10), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], slots::FALL);
    sim.state.fighters[0].state_frame = 11;
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.f().move_id, slots::FALL_LANDING as u8);
    assert!(sim.fighter(1).percent >= dealt(&sim, 35));
}

#[test]
fn the_rise_without_a_second_press_lands_from_the_special_fall_with_20_frames_of_lag() {
    let w = sword();
    let rise = &w.moves[MoveId::UpSpecial as usize];
    assert_eq!(
        (
            rise.fall_drift,
            rise.fall_landing_lag,
            rise.start_speed,
            rise.rise_ground,
            rise.rise_air
        ),
        (60, 20, 60, 100, 110)
    );
    let mut sim = duel();
    sim.tick(inp(0, 127, SPECIAL));
    let mut lag = 0;
    for _ in 0..200 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Landing {
            lag = sim.f().lag;
            break;
        }
    }
    assert_eq!(lag, 20);
}

#[test]
fn the_rises_hits_are_the_references() {
    let w = sword();
    let rise = &w.moves[MoveId::UpSpecial as usize];
    let stab: Vec<_> = rise.hitboxes.iter().filter(|h| h.start == 6).collect();
    assert!(!stab.is_empty());
    assert!(stab.iter().all(|h| h.damage == Fx::from_int(3)
        && h.fixed_kb == 80
        && h.hitlag == 250
        && h.sdi == 0
        && h.group == 0));
    let mut angles: Vec<i16> = stab.iter().map(|h| h.angle).collect();
    angles.dedup();
    assert_eq!(angles, vec![40, 78, 102]);
    let slash: Vec<_> = rise.hitboxes.iter().filter(|h| h.start >= 9).collect();
    assert!(slash.iter().all(|h| h.damage == Fx::from_int(4)
        && h.set_weight
        && h.sdi == 0
        && h.group == 1
        && h.hitlag == 120));
    assert!(slash
        .iter()
        .filter(|h| h.start <= 10 && h.angle != 90)
        .all(|h| h.fixed_kb == 121));
    assert!(slash
        .iter()
        .filter(|h| h.start == 11)
        .all(|h| h.fixed_kb == 0));
}

#[test]
fn the_rise_catches_ledges_from_frame_24() {
    let w = sword();
    let rise = &w.moves[MoveId::UpSpecial as usize];
    assert_eq!((rise.ledge_from(false), rise.ledge_from(true)), (23, 23));
    let fall = &w.moves[slots::FALL as usize];
    assert_eq!(fall.ledge_drop, Some((su(56_000), su(92_000))));
}

/// The champion starts the falling slash `above` world units above the right ledge, just off the stage facing it; whether it caught
/// the ledge.
fn falling_slash_by_the_ledge(above: Fx) -> bool {
    let mut sim = duel();
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .unwrap();
    sim.put_airborne(0, ledge.x + fx(12, 10), ledge.y + above, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = -1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], slots::FALL);
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            return true;
        }
    }
    false
}

#[test]
fn the_falling_slash_catches_a_ledge_only_after_falling_most_of_the_rise() {
    // From just above the ledge it falls past it (the reference's self-destruct); from the rise's height it catches it [E: gap 7].
    assert!(!falling_slash_by_the_ledge(Fx::from_int(2)));
    assert!(falling_slash_by_the_ledge(Fx::from_int(8)));
}

// ---- The charge (down special) -----------------------------------------------------------------------------------------------------

#[test]
fn the_charge_fills_the_gauge_in_334_frames_and_ends_itself_when_full() {
    let mut sim = duel();
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
    let mut full_on = 0;
    for t in 2..=400u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().gauge_mode > 0 {
            full_on = t;
            break;
        }
        assert_eq!(
            sim.f().move_id,
            MoveId::DownSpecial as u8,
            "still charging on {t}"
        );
    }
    // 0.3 a frame from its second frame: full on frame 335 of the move, 334 frames of charging [R: 334 / 335].
    assert_eq!(full_on, 335);
    assert_eq!(sim.f().gauge, Fx::from_int(100));
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().move_id,
        slots::CHARGE_CANCEL as u8,
        "a full gauge ends the charge [E]"
    );
}

#[test]
fn shield_jump_grab_and_special_cancel_the_charge_with_7_frames_of_lag() {
    for button in [SHIELD, JUMP, GRAB, SPECIAL] {
        let mut sim = duel();
        sim.tick(inp(0, -127, SPECIAL));
        sim.ticks(20, inp(0, 0, 0));
        let gauge = sim.f().gauge;
        assert!(gauge > Fx::ZERO);
        sim.tick(inp(0, 0, button));
        assert_eq!(
            sim.f().move_id,
            slots::CHARGE_CANCEL as u8,
            "button {button}"
        );
        // Free on the cancel's frame 8 (its FAF).
        let free = run_out(&mut sim, |_| inp(0, 0, 0));
        assert_eq!(free, 8, "button {button}");
        assert!(sim.f().gauge >= gauge, "the charge is kept");
    }
}

#[test]
fn a_hit_during_the_charge_drains_the_gauge() {
    let w = sword();
    assert!(w.moves[MoveId::DownSpecial as usize].gauge_drain);
    assert!(w.moves[slots::CHARGE_CANCEL as usize].gauge_drain);
    let mut sim = duel();
    sim.state.fighters[0].gauge = Fx::from_int(50);
    sim.stand(1, fx(15, 10), -1);
    sim.tick(inp(0, -127, SPECIAL));
    sim.ticks(5, inp(0, 0, 0));
    let before = sim.f().gauge;
    // The duelist's forward tilt.
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::FTilt);
    let mut taken = Fx::ZERO;
    for _ in 0..30 {
        let p = sim.f().percent;
        sim.tick(inp(0, 0, 0));
        if sim.f().percent > p {
            taken = sim.f().percent - p;
            break;
        }
    }
    assert!(taken > Fx::ZERO, "hit");
    // (It had charged on the frames before the hit.)
    assert!(sim.f().gauge < before, "drained");
    assert!(sim.f().gauge <= before + fx(3, 1) - taken);
}

#[test]
fn every_greatsword_special_slot_is_its_own_and_four_are_spare() {
    let w = sword();
    for id in [
        slots::WAVE_AIR,
        slots::COMBO_2,
        slots::COMBO_3,
        slots::FALL,
        slots::FALL_LANDING,
        slots::CHARGE_CANCEL,
        slots::POWER_WAVE,
        slots::POWER_WAVE_AIR,
        slots::POWER_COMBO,
        slots::POWER_RISE,
        slots::FINISHER,
        slots::FINISHER_AIR,
    ] {
        assert!(!w.moves[id as usize].is_empty(), "{id:?}");
    }
    // ext12 to ext15 (41 to 44) are spare.
    for i in 41..=44u8 {
        assert!(w.moves[usize::from(i)].is_empty());
    }
}
