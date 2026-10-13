//! The Greatsword's powered versions, phase 3 (sim 87), and the general pieces they brought: E3e (a projectile that bursts on its
//! first fighter or a wall, `ProjectileSpawn::burst`, hashed `Projectile::bursting`), E28 (the vortex angle 366, hashed
//! `Fighter::link_kind` and `vortex`), E11's `motion_drift`; the powered wave, combo slash, rise and the finisher; any special in the
//! powered mode spending the gauge. Reference data: `docs/movesets/cloud/REFERENCE_DATA.md`; [E] fits are noted per test.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::greatsword_slots as slots;
use sim_core::moves::{
    greatsword, MoveId, Weapon, ANGLE_AUTOLINK, ANGLE_BY_POSITION, ANGLE_VORTEX,
};
use sim_core::state::{FighterState as S, LINK_VORTEX};
use sim_core::{Fx, Vec2};

const CHAMPION: u8 = 6;

/// The champion (player 0) at 0 facing right, a duelist (player 1) at the far end, the others out.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([CHAMPION, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

/// `duel` with the champion's powered mode running (a full gauge).
fn powered() -> Sim {
    let mut sim = duel();
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 900;
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

/// Ticks the current move out; the reference frame the fighter is free on (the house's FAF count, as `greatsword_specials.rs`).
fn run_out(sim: &mut Sim, input: impl Fn(u32) -> sim_core::Input) -> u32 {
    for t in 1..=400u32 {
        sim.tick(input(t));
        if sim.f().state != S::Attack {
            return t + 2;
        }
    }
    panic!("never ended");
}

/// The reference frames (1 = the frame the special was pressed) on which player 0 is intangible, for a move started by `press`.
fn intangible_frames(mut sim: Sim, press: sim_core::Input) -> Vec<u32> {
    let mut out = Vec::new();
    sim.tick(press);
    if is_intangible(sim.f()) {
        out.push(1);
    }
    for t in 2..=40u32 {
        sim.tick(inp(0, 0, 0));
        if is_intangible(sim.f()) {
            out.push(t);
        }
    }
    out
}

fn span(a: u32, b: u32) -> Vec<u32> {
    (a..=b).collect()
}

// ---- Spending ------------------------------------------------------------------------------------------------------------------------

#[test]
fn every_special_in_the_powered_mode_is_its_powered_version_and_spends_the_gauge() {
    let w = sword();
    for (id, power) in [
        (MoveId::NSpecial, slots::POWER_WAVE),
        (MoveId::SideSpecial, slots::POWER_COMBO),
        (MoveId::UpSpecial, slots::POWER_RISE),
        (MoveId::DownSpecial, slots::FINISHER),
    ] {
        assert_eq!(w.moves[id as usize].limit_version, Some(power as u8));
    }
    assert_eq!(
        w.moves[slots::POWER_WAVE as usize].air_version,
        Some(slots::POWER_WAVE_AIR as u8)
    );
    assert_eq!(
        w.moves[slots::FINISHER as usize].air_version,
        Some(slots::FINISHER_AIR as u8)
    );
    for (stick, air, want) in [
        ((0, 0), false, slots::POWER_WAVE),
        ((0, 0), true, slots::POWER_WAVE_AIR),
        ((127, 0), false, slots::POWER_COMBO),
        ((0, 127), false, slots::POWER_RISE),
        ((0, -127), false, slots::FINISHER),
        ((0, -127), true, slots::FINISHER_AIR),
    ] {
        let mut sim = powered();
        if air {
            sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        }
        sim.tick(inp(stick.0, stick.1, SPECIAL));
        assert_eq!(sim.f().move_id, want as u8, "{stick:?} air {air}");
        assert_eq!(
            (sim.f().gauge, sim.f().gauge_mode),
            (Fx::ZERO, 0),
            "spent on the attempt"
        );
        // Out of the mode the same input is the normal special again.
        let mut sim = duel();
        if air {
            sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        }
        sim.tick(inp(stick.0, stick.1, SPECIAL));
        assert_ne!(sim.f().move_id, want as u8);
    }
}

#[test]
fn follow_ups_do_not_spend_and_a_full_charge_does_not_start_the_finisher() {
    // The charge fills the gauge and ends itself into its cancel [E, owner decision 43]; the mode then runs, and only the next
    // special pressed is powered.
    let mut sim = duel();
    sim.state.fighters[0].gauge = Fx::from_int(99);
    sim.tick(inp(0, -127, SPECIAL));
    sim.ticks(10, inp(0, 0, 0));
    assert!(sim.f().gauge_mode > 0);
    assert_ne!(sim.f().move_id, slots::FINISHER as u8);
}

// ---- E3e and the powered wave --------------------------------------------------------------------------------------------------------

#[test]
fn the_powered_waves_data_is_the_references() {
    let w = sword();
    let ground = &w.moves[slots::POWER_WAVE as usize];
    let air = &w.moves[slots::POWER_WAVE_AIR as usize];
    assert_eq!(air.on_land, Some(slots::POWER_WAVE as u8));
    for (m, speed, beam, spin, last) in [(ground, 2950, 60, 20, 30), (air, 2400, 48, 16, 24)] {
        assert_eq!(m.total_frames + 2, 61, "FAF 61");
        assert_eq!(
            (m.intangible_from, m.intangible),
            (9, 17),
            "intangible 10-17"
        );
        let p = m.projectile.expect("a beam");
        // Out on 15, hitting from 16 to 62 (lifetime 47).
        assert_eq!((p.frame, p.life, p.speed), (14, 48, su(speed)));
        assert_eq!(p.hitbox.damage, Fx::from_ratio(beam, 10));
        assert!(!p.hitbox.clanks, "transcendent");
        assert_eq!(p.hitbox.half_height, Fx::from_ratio(675, 800));
        let b = p.burst.expect("bursts");
        assert_eq!((b.life, b.rehit, b.on_wall), (31, 6, true));
        let spin_hit = b.hit_at(1).expect("the loop");
        assert_eq!(b.hit_at(28), Some(spin_hit));
        let last_hit = b.hit_at(29).expect("the final hit");
        assert!(b.hit_at(0).is_none());
        assert_eq!(spin_hit.damage, Fx::from_ratio(spin, 10));
        assert_eq!(
            (
                spin_hit.angle,
                spin_hit.fixed_kb,
                spin_hit.knockback_growth,
                spin_hit.hitlag
            ),
            (ANGLE_VORTEX, 40, 100, 50)
        );
        assert_eq!(last_hit.damage, Fx::from_ratio(last, 10));
        assert_eq!(
            (
                last_hit.angle,
                last_hit.base_knockback,
                last_hit.knockback_growth
            ),
            (48, 50, 226)
        );
        assert_ne!(spin_hit.group, last_hit.group);
        assert_eq!(spin_hit.radius, fx(18, 10));
        assert!(!spin_hit.clanks && !last_hit.clanks);
    }
}

/// The champion fires the grounded powered wave at a duelist standing at `x` (and a second duelist at `behind`, if any). Returns the
/// percent each ends with, and the projectile's state after the first contact.
fn power_wave_at(x: Fx, behind: Option<Fx>) -> (Sim, Vec<(u32, Fx)>) {
    let mut sim = powered();
    sim.stand(1, x, -1);
    if let Some(b) = behind {
        sim.state.fighters[2].active = true;
        sim.stand(2, b, -1);
    }
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, slots::POWER_WAVE as u8);
    let mut hits = Vec::new();
    let mut last = Fx::ZERO;
    for t in 2..=150u32 {
        sim.tick(inp(0, 0, 0));
        let p = sim.fighter(1).percent;
        if p > last {
            hits.push((t, p - last));
            last = p;
        }
    }
    (sim, hits)
}

#[test]
fn the_powered_wave_bursts_on_its_first_fighter_for_19_and_13_behind_it() {
    let (sim, hits) = power_wave_at(Fx::from_int(4), Some(Fx::from_int(4) + fx(9, 10)));
    let sum = |v: &[(u32, Fx)]| v.iter().fold(Fx::ZERO, |a, (_, d)| a + *d);
    // 6 + 5 x 2 + 3 [R: "19% on the first target and 13% on any targets immediately behind"].
    let want: Vec<Fx> = [60, 20, 20, 20, 20, 20, 30]
        .iter()
        .map(|&d| dealt(&sim, d))
        .collect();
    let got: Vec<Fx> = hits.iter().map(|(_, d)| *d).collect();
    assert_eq!(got, want, "{hits:?}");
    assert_eq!(sum(&hits), dealt(&sim, 190));
    // The loop every 6 frames from the burst's first frame, the final hit on its 29th.
    let t0 = hits[0].0;
    let times: Vec<u32> = hits.iter().map(|(t, _)| t - t0).collect();
    assert_eq!(times, vec![0, 1, 7, 13, 19, 25, 29]);
    assert_eq!(sim.fighter(2).percent, dealt(&sim, 130), "13% behind");
    assert!(
        sim.state.projectiles.iter().all(|p| !p.active),
        "gone after its 30 frames"
    );
}

#[test]
fn the_burst_stands_still_where_it_hit_and_pulls_its_victims_in() {
    let mut sim = powered();
    sim.stand(1, Fx::from_int(4), -1);
    sim.tick(inp(0, 0, SPECIAL));
    let mut burst = None;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if let Some(p) = sim
            .state
            .projectiles
            .iter()
            .find(|p| p.active && p.bursting)
        {
            burst = Some(*p);
            break;
        }
    }
    let b = burst.expect("burst");
    assert_eq!(b.vel, Vec2::ZERO);
    assert_eq!(b.age, 0);
    // Five frames after a loop hit's hitlag the victim's middle is at the burst's centre (the vortex has no launch of its own when
    // the burst stands still).
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
        let p = sim
            .state
            .projectiles
            .iter()
            .find(|p| p.active)
            .expect("still bursting");
        assert_eq!(p.pos, b.pos, "it does not move");
    }
    let f = sim.fighter(1);
    assert_eq!(f.link_kind, LINK_VORTEX);
    let p = sim.content.fighters[0];
    let middle = f.pos + Vec2::new(Fx::ZERO, p.ecb_height / Fx::from_int(2));
    assert!(
        (middle - b.pos).length() <= fx(6, 10),
        "held near the burst: {:?} vs {:?}",
        middle,
        b.pos
    );
}

#[test]
fn the_aerial_powered_wave_is_the_weaker_one() {
    let mut sim = powered();
    sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, Fx::ZERO);
    sim.stand(1, Fx::from_int(3), -1);
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, slots::POWER_WAVE_AIR as u8);
    // (Landing before it fires makes it the grounded one: the champion is held in the air.)
    for _ in 0..120 {
        if sim.f().state == S::Attack && sim.f().move_id == slots::POWER_WAVE_AIR as u8 {
            let me = sim.f().pos.x;
            sim.state.fighters[0].pos = Vec2::new(me, fx(5, 10));
            sim.state.fighters[0].vel = Vec2::ZERO;
        }
        sim.tick(inp(0, 0, 0));
    }
    // 4.8 + 5 x 1.6 + 2.4 = 15.2 [R: "15%"] (each hit's fixed-point rounding apart).
    let got = sim.fighter(1).percent;
    assert!(
        (got - dealt(&sim, 152)).abs() <= Fx::from_raw(16),
        "{got:?}"
    );
}

/// A powered wave fired from under the right ledge toward the stage's side wall.
fn wall_shot() -> Sim {
    let mut sim = powered();
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .unwrap();
    sim.put_airborne(
        0,
        ledge.x + Fx::from_int(3),
        ledge.y - Fx::from_int(3),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = -1;
    sim
}

#[test]
fn the_powered_wave_bursts_on_a_wall_and_spikes_a_fighter_by_it() {
    let mut sim = wall_shot();
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .unwrap();
    sim.tick(inp(0, 0, SPECIAL));
    let mut burst = None;
    for _ in 0..60 {
        // The champion held where it fired from.
        let me = sim.f().pos;
        sim.state.fighters[0].pos = Vec2::new(me.x, ledge.y - Fx::from_int(3));
        sim.state.fighters[0].vel = Vec2::ZERO;
        sim.tick(inp(0, 0, 0));
        if let Some(p) = sim
            .state
            .projectiles
            .iter()
            .find(|p| p.active && p.bursting)
        {
            burst = Some(*p);
            break;
        }
    }
    let b = burst.expect("it burst on the wall instead of vanishing");
    assert!(
        b.pos.x > ledge.x && b.pos.x < ledge.x + Fx::ONE,
        "by the wall: {:?}",
        b.pos
    );
    // A fighter by the wall in the burst is hit by it.
    sim.put_airborne(
        1,
        b.pos.x + fx(5, 10),
        b.pos.y - fx(8, 10),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.ticks(35, inp(0, 0, 0));
    assert!(sim.fighter(1).percent > Fx::ZERO);
}

#[test]
fn the_normal_wave_still_vanishes_on_a_wall() {
    let mut sim = wall_shot();
    sim.state.fighters[0].gauge_mode = 0;
    sim.state.fighters[0].gauge = Fx::ZERO;
    sim.tick(inp(0, 0, SPECIAL));
    let mut seen = false;
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        seen |= sim.state.projectiles.iter().any(|p| p.active);
        assert!(sim.state.projectiles.iter().all(|p| !p.bursting));
    }
    assert!(seen && sim.state.projectiles.iter().all(|p| !p.active));
}

#[test]
fn the_burst_is_resimulated_exactly_from_a_snapshot() {
    // Rollback: from a snapshot taken during the burst, the same inputs give the same checksums; dropping `bursting` from the
    // snapshot gives a different match (so the flag is real state, and it is hashed: `state::checksum_sees_every_field`).
    let mut sim = powered();
    sim.stand(1, Fx::from_int(4), -1);
    sim.tick(inp(0, 0, SPECIAL));
    let mut snap = None;
    for t in 0..120u32 {
        sim.tick(inp(0, 0, 0));
        if snap.is_none()
            && sim
                .state
                .projectiles
                .iter()
                .any(|p| p.bursting && p.age == 3)
        {
            snap = Some((t, sim.state));
        }
    }
    let end = sim.state.checksum();
    let (_, s) = snap.expect("a burst");
    let mut again = Sim::with_chars([CHAMPION, 0, 0, 0]);
    again.state = s;
    let mut broken = again.state;
    let n = broken.projectiles.iter().position(|p| p.bursting).unwrap();
    broken.projectiles[n].bursting = false;
    while again.state.frame < sim.state.frame {
        again.tick(inp(0, 0, 0));
    }
    assert_eq!(again.state.checksum(), end);
    let mut other = Sim::with_chars([CHAMPION, 0, 0, 0]);
    other.state = broken;
    while other.state.frame < sim.state.frame {
        other.tick(inp(0, 0, 0));
    }
    assert_ne!(other.state.checksum(), end);
}

#[test]
fn the_burst_is_in_the_content_hash() {
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[usize::from(CHAMPION)].weapon);
    let mut c = base.clone();
    let p = c.weapons[w].moves[slots::POWER_WAVE as usize]
        .projectile
        .as_mut()
        .unwrap();
    p.burst.as_mut().unwrap().rehit = 5;
    assert_ne!(h, c.hash());
    let mut c = base.clone();
    c.weapons[w].moves[slots::POWER_WAVE as usize]
        .projectile
        .as_mut()
        .unwrap()
        .burst = None;
    assert_ne!(h, c.hash());
}

// ---- E28, the vortex ------------------------------------------------------------------------------------------------------------------

#[test]
fn a_vortex_hit_sends_a_grounded_fighter_along_the_attackers_movement_not_at_80() {
    // The normal wave (a projectile moving forward) with the 366 angle and a strong set knockback: a grounded victim goes along the
    // ground the way the wave moves (the 367's 80-degree grounded rule does not apply), at the knockback's launch speed.
    let mut sim = duel();
    let w = usize::from(sim.content.fighters[usize::from(CHAMPION)].weapon);
    {
        let p = sim.content.weapons[w].moves[MoveId::NSpecial as usize]
            .projectile
            .as_mut()
            .unwrap();
        p.hitbox.angle = ANGLE_VORTEX;
        p.hitbox.fixed_kb = 60;
        p.phases = [None; sim_core::moves::MAX_PHASES];
    }
    sim.stand(1, Fx::from_int(3), -1);
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        let f = sim.fighter(1);
        if f.state == S::Hitstun && !f.launch_pending {
            break;
        }
    }
    let f = sim.fighter(1);
    assert_eq!(f.link_kind, LINK_VORTEX);
    assert!(
        f.kb_vel.x > Fx::ZERO && f.kb_vel.y == Fx::ZERO,
        "{:?}",
        f.kb_vel
    );
    // The launch speed is the knockback's (60 x 0.03 / 8 world a frame, less one frame's decay), not the wave's own speed.
    assert!(f.kb_vel.x > su(2000), "{:?}", f.kb_vel);
}

#[test]
fn the_combo_slashs_nearest_box_is_the_vortex_and_the_powered_ones_are_autolinks() {
    let w = sword();
    let one = &w.moves[MoveId::SideSpecial as usize];
    assert_eq!(one.hitboxes[2].angle, ANGLE_VORTEX);
    let power = &w.moves[slots::POWER_COMBO as usize];
    assert!(power
        .hitboxes
        .iter()
        .filter(|h| h.group <= 1)
        .all(|h| h.angle == ANGLE_AUTOLINK && h.fixed_kb == 80));
}

// ---- The powered combo slash ---------------------------------------------------------------------------------------------------------

#[test]
fn the_powered_combo_slash_runs_all_five_hits_by_itself() {
    let w = sword();
    let m = &w.moves[slots::POWER_COMBO as usize];
    assert_eq!(m.total_frames + 2, 61, "FAF 61");
    let windows: Vec<(u8, u8, u8)> = m
        .hitboxes
        .iter()
        .map(|h| (h.start + 1, h.end + 1, h.group))
        .collect();
    assert_eq!(
        windows,
        vec![
            (10, 11, 0),
            (10, 11, 0),
            (15, 16, 1),
            (15, 16, 1),
            (21, 22, 2),
            (21, 22, 2),
            (28, 28, 3),
            (28, 28, 3),
            (38, 39, 4),
            (38, 39, 4)
        ]
    );
    // A duelist held in front of it takes all five with no more presses: 5 + 5 + 3 + 3 + 10 = 26% [R: UFD; conflict 1].
    let mut sim = powered();
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(sim.f().move_id, slots::POWER_COMBO as u8);
    let mut hits = Vec::new();
    let mut last = Fx::ZERO;
    for _ in 0..120 {
        let me = sim.f().pos;
        let p = sim.state.fighters[1].percent;
        sim.put_airborne(1, me.x + fx(13, 10), me.y + fx(4, 10), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[1].percent = p;
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > last {
            hits.push(sim.fighter(1).percent - last);
            last = sim.fighter(1).percent;
        }
        if sim.f().state != S::Attack {
            break;
        }
    }
    let want: Vec<Fx> = [50, 50, 30, 30, 100]
        .iter()
        .map(|&d| dealt(&sim, d))
        .collect();
    assert_eq!(hits, want);
}

#[test]
fn the_powered_combo_slash_is_intangible_on_6_to_11_and_free_on_61() {
    assert_eq!(
        intangible_frames(powered(), inp(127, 0, SPECIAL)),
        span(6, 11)
    );
    let mut sim = powered();
    sim.tick(inp(127, 0, SPECIAL));
    assert_eq!(run_out(&mut sim, |_| inp(0, 0, 0)), 61);
}

#[test]
fn the_powered_combo_slash_stops_all_momentum_in_the_air() {
    let mut sim = powered();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(40), fx(8, 10), fx(5, 10));
    sim.tick(inp(127, 0, SPECIAL));
    let y = sim.f().pos.y;
    let x = sim.f().pos.x;
    for _ in 0..50 {
        sim.tick(inp(127, 0, 0));
        assert_eq!(sim.f().vel, Vec2::ZERO);
        assert_eq!((sim.f().pos.x, sim.f().pos.y), (x, y));
    }
}

// ---- The powered rise ----------------------------------------------------------------------------------------------------------------

/// The peak height of a rise (`id` decides which via the mode) from the ground or the air.
fn rise_peak(power: bool, air: bool) -> Fx {
    let mut sim = if power { powered() } else { duel() };
    sim.stand(1, Fx::from_int(-8), 1);
    if air {
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    }
    let y0 = sim.f().pos.y;
    sim.tick(inp(0, 127, SPECIAL));
    let mut peak = Fx::MIN;
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        peak = peak.max(sim.f().pos.y - y0);
    }
    peak
}

#[test]
fn the_powered_rise_climbs_1_6_times_from_the_ground_and_1_85_from_the_air() {
    for (air, k) in [(false, 160), (true, 185)] {
        let want = su(66_000) * Fx::from_ratio(k, 100);
        let peak = rise_peak(true, air);
        assert!(
            (peak - want).abs() <= fx(3, 10),
            "air {air}: {peak:?} vs {want:?}"
        );
        assert!(peak > rise_peak(false, air));
    }
}

#[test]
fn the_powered_rises_data_and_hits_are_the_references() {
    let w = sword();
    let m = &w.moves[slots::POWER_RISE as usize];
    assert_eq!(
        (
            m.rise_ground,
            m.rise_air,
            m.start_speed,
            m.fall_drift,
            m.fall_landing_lag
        ),
        (160, 185, 67, 80, 20)
    );
    assert_eq!(
        (m.ledge_from(false), m.ledge_from(true)),
        (0, 0),
        "ledges throughout"
    );
    assert!(m.helpless_after && m.motion_drift > 0);
    let stab: Vec<_> = m.hitboxes.iter().filter(|h| h.group == 0).collect();
    assert!(stab.iter().all(|h| h.damage == Fx::from_int(6)
        && (h.start, h.end) == (6, 8)
        && h.fixed_kb == 80
        && h.sdi == 100
        && h.hitlag == 250));
    let slash: Vec<_> = m.hitboxes.iter().filter(|h| h.group == 1).collect();
    assert!(slash
        .iter()
        .all(|h| h.damage == Fx::from_int(7) && !h.set_weight));
    assert!(slash.iter().filter(|h| h.start == 9).all(|h| (
        h.angle,
        h.base_knockback,
        h.knockback_growth
    ) == (88, 50, 142)));
    assert!(slash
        .iter()
        .filter(|h| h.start == 10 && h.angle == 88)
        .all(|h| (h.base_knockback, h.knockback_growth, h.end) == (50, 110, 15)));
}

#[test]
fn the_powered_rise_is_intangible_on_5_to_12() {
    assert_eq!(
        intangible_frames(powered(), inp(0, 127, SPECIAL)),
        span(5, 12)
    );
}

#[test]
fn the_powered_rise_takes_the_second_press_on_33_to_39_into_the_ordinary_falling_slash() {
    for (press, goes) in [(32u32, false), (33, true), (39, true), (40, false)] {
        let mut sim = powered();
        sim.tick(inp(0, 127, SPECIAL));
        let mut fell = false;
        for t in 2..=50u32 {
            sim.tick(inp(0, 0, if t == press { SPECIAL } else { 0 }));
            fell |= sim.f().move_id == slots::FALL as u8 && sim.f().state == S::Attack;
        }
        assert_eq!(fell, goes, "pressed on {press}");
    }
}

#[test]
fn the_powered_rise_steers_while_it_rises_and_falls_helpless_after_69() {
    let drift = |stick: i8| {
        let mut sim = powered();
        sim.tick(inp(0, 127, SPECIAL));
        sim.ticks(20, inp(stick, 0, 0));
        sim.f().pos.x
    };
    assert!(drift(127) > drift(0) + fx(5, 10), "steers right");
    assert!(drift(-127) < drift(0) - fx(5, 10), "steers left");
    let mut sim = powered();
    sim.tick(inp(0, 127, SPECIAL));
    let mut helpless_on = 0;
    for t in 2..=90u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Helpless && helpless_on == 0 {
            helpless_on = t;
        }
    }
    assert_eq!(helpless_on, 69);
}

#[test]
fn motion_drift_off_keeps_the_motions_own_sideways_speed() {
    // The normal rise has none: the stick does not move it while it rises.
    let x = |stick: i8| {
        let mut sim = duel();
        sim.tick(inp(0, 127, SPECIAL));
        sim.ticks(16, inp(stick, 0, 0));
        sim.f().pos.x
    };
    assert_eq!(x(127), x(0));
}

#[test]
fn the_powered_rise_catches_a_ledge_while_rising() {
    let mut sim = powered();
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .unwrap();
    sim.put_airborne(
        0,
        ledge.x + fx(9, 10),
        ledge.y - Fx::from_int(4),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = -1;
    sim.tick(inp(0, 127, SPECIAL));
    let mut caught = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        caught |= sim.f().state == S::LedgeHang;
    }
    assert!(caught);
}

// ---- The finisher --------------------------------------------------------------------------------------------------------------------

#[test]
fn the_finishers_frames_and_hits_are_the_references() {
    let w = sword();
    let ground = &w.moves[slots::FINISHER as usize];
    let air = &w.moves[slots::FINISHER_AIR as usize];
    assert_eq!((ground.total_frames + 2, air.total_frames + 2), (85, 88));
    for (m, hits, wind, ks) in [
        (ground, vec![12u8, 13, 14], (22u8, 26u8), 100i16),
        (air, vec![14, 15, 17], (25, 29), 86),
    ] {
        let mut frames: Vec<u8> = m
            .hitboxes
            .iter()
            .filter(|h| !h.push)
            .map(|h| h.start + 1)
            .collect();
        frames.dedup();
        assert_eq!(frames, hits);
        assert!(m
            .hitboxes
            .iter()
            .filter(|h| !h.push)
            .all(|h| h.damage == Fx::ONE
                && (h.angle, h.base_knockback) == (80, 60)
                && h.angle_type == ANGLE_BY_POSITION
                && h.hitlag == 200
                && h.group == 0
                && h.clanks
                && !h.set_weight));
        let pushes: Vec<_> = m.hitboxes.iter().filter(|h| h.push).collect();
        assert!(!pushes.is_empty());
        assert!(pushes.iter().all(|h| (h.start + 1, h.end + 1) == wind
            && h.fixed_kb == 80
            && h.knockback_growth == ks
            && !h.clanks));
    }
    let ks = |m: &sim_core::moves::Move, frame: u8| {
        m.hitboxes
            .iter()
            .find(|h| h.start + 1 == frame && !h.push)
            .unwrap()
            .knockback_growth
    };
    assert_eq!(
        (ks(ground, 12), ks(ground, 13), ks(ground, 14)),
        (431, 406, 406)
    );
    assert_eq!((ks(air, 14), ks(air, 15), ks(air, 17)), (371, 371, 349));
    assert_eq!(
        (
            air.start_speed,
            air.start_speed_y,
            air.air_gravity,
            air.air_max_fall,
            air.on_land
        ),
        (
            40,
            40,
            Some(su(30)),
            Some(su(1000)),
            Some(slots::FINISHER as u8)
        )
    );
}

#[test]
fn the_finisher_hits_once_on_12_for_1_percent_and_launches_hard() {
    let mut sim = powered();
    sim.stand(1, fx(12, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(100);
    sim.tick(inp(0, -127, SPECIAL));
    let mut hit_on = 0;
    for t in 2..=40u32 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::from_int(100) && hit_on == 0 {
            hit_on = t;
        }
    }
    assert_eq!(hit_on, 12);
    assert_eq!(sim.fighter(1).percent, Fx::from_int(100) + dealt(&sim, 10));
    // At 100% the KS 431 launch is a kill-strength one (far above the tumble threshold).
    assert!(
        sim.fighter(1).launch_kb > Fx::from_int(200),
        "{:?}",
        sim.fighter(1).launch_kb
    );
}

#[test]
fn the_finisher_is_free_on_85_grounded_and_88_aerial_and_hovers_in_the_air() {
    let mut sim = powered();
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(run_out(&mut sim, |_| inp(0, 0, 0)), 85);
    let mut sim = powered();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ONE, -Fx::ONE);
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(sim.f().move_id, slots::FINISHER_AIR as u8);
    // 0.4 of both speeds on starting, then a slow fall held at 1.0 of the reference's units a frame.
    assert!((sim.f().vel.x - fx(4, 10)).abs() <= fx(1, 50));
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        assert!(sim.f().vel.y >= -su(1000));
    }
    let mut sim = powered();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, -127, SPECIAL));
    assert_eq!(run_out(&mut sim, |_| inp(0, 0, 0)), 88);
}

#[test]
fn the_finishers_windbox_pushes_both_sides_without_hitstun() {
    for side in [1, -1] {
        let mut sim = powered();
        sim.stand(1, fx(side * 37, 10), -side as i8);
        sim.tick(inp(0, -127, SPECIAL));
        let mut pushed = false;
        for t in 2..=31 {
            sim.tick(inp(0, 0, 0));
            pushed |= sim.fighter(1).vel.x.signum_int() == side;
            assert_ne!(
                sim.fighter(1).state,
                S::Hitstun,
                "side {side} frame {t} percent {:?} x {:?}",
                sim.fighter(1).percent,
                sim.fighter(1).pos.x
            );
        }
        assert!(pushed, "side {side}");
    }
}

#[test]
fn the_powered_waves_intangibility_is_10_to_17() {
    assert_eq!(
        intangible_frames(powered(), inp(0, 0, SPECIAL)),
        span(10, 17)
    );
}
