//! The Popgun's neutral special, the peanut popgun (class 11, the scamp body), modelled mechanically on a reference character: the
//! uncharged shot on frame 16, the charge to frame 94 fired on release, the overcharge going off on frame 97, the shield cancel, the
//! peanut's hit by the charge. Reference data: the twelfth moveset's `REFERENCE_DATA.md` (`docs/movesets/diddy_kong/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{SHIELD, SPECIAL};
use sim_core::moves::popgun_slots::{POPGUN_FIRE, POPGUN_FULL, POPGUN_MISFIRE};
use sim_core::moves::{MoveId, ANGLE_FORWARD};
use sim_core::state::FighterState as S;
use sim_core::{Fx, Projectile};

const SCAMP: u8 = 11;

/// The scamp (player 0) at -3 facing right and a duelist (player 1) far to the right, the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SCAMP, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-3), 1);
    sim.stand(1, Fx::from_int(12), -1);
    sim
}

fn peanut(sim: &Sim) -> Option<Projectile> {
    sim.state
        .projectiles
        .iter()
        .copied()
        .find(|p| p.active && p.owner == 0)
}

/// Special pressed on frame 1 and held through frame `held` (0: a tap); returns (the frame the peanut appeared, the peanut then, the
/// frame the scamp could act again: the FAF).
fn shoot(held: u32) -> (Option<u32>, Option<Projectile>, u32) {
    let mut sim = duel();
    let mut appeared = None;
    let mut shot = None;
    for frame in 1..=260u32 {
        let b = if frame == 1 || frame <= held {
            SPECIAL
        } else {
            0
        };
        sim.tick(inp(0, 0, b));
        if appeared.is_none() {
            if let Some(p) = peanut(&sim) {
                appeared = Some(frame);
                shot = Some(p);
            }
        }
        if frame > 1 && sim.f().state != S::Attack {
            return (appeared, shot, frame + 1);
        }
    }
    panic!("the move never ended");
}

#[test]
fn a_tap_fires_the_uncharged_peanut_on_frame_16_with_faf_49() {
    let (at, p, faf) = shoot(0);
    assert_eq!(at, Some(16));
    let p = p.unwrap();
    assert_eq!(p.damage, Fx::from_ratio(43, 10), "4.3% uncharged [R]");
    assert!(p.vel.x > Fx::ZERO && p.vel.y > Fx::ZERO, "forward and up");
    assert_eq!(faf, 49);
}

#[test]
fn holding_charges_and_the_peanut_leaves_on_the_release_with_33_frames_after() {
    // Held through frame 49, let go on 50: 34 frames of charge (from 16), out on 50, FAF 83.
    let (at, p, faf) = shoot(49);
    assert_eq!(at, Some(50));
    assert_eq!(faf, 83);
    let p = p.unwrap();
    let want = Fx::from_ratio(43, 10)
        + (Fx::from_ratio(138, 10) - Fx::from_ratio(43, 10))
            * Fx::from_ratio(34, i32::from(POPGUN_FULL));
    assert_eq!(p.damage, want);
    // A full charge (let go on 94): 13.8%, faster and flatter than the uncharged peanut.
    let (at, full, faf) = shoot(93);
    assert_eq!((at, faf), (Some(94), 127));
    let full = full.unwrap();
    assert_eq!(full.damage, Fx::from_ratio(138, 10));
    let (_, tap, _) = shoot(0);
    let tap = tap.unwrap();
    assert!(full.vel.x > tap.vel.x.mul_int(2), "faster");
    let slope = |p: &Projectile| p.vel.y / p.vel.x;
    assert!(slope(&full) < slope(&tap), "flatter");
}

#[test]
fn held_to_frame_97_the_gun_goes_off_for_23_percent() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-2), -1);
    let mut boom = None;
    for frame in 1..=230u32 {
        sim.tick(inp(0, 0, SPECIAL));
        if frame == 97 {
            assert_eq!(sim.f().move_id, POPGUN_MISFIRE as u8, "frame 97");
        }
        if boom.is_none() && sim.fighter(1).percent > Fx::ZERO {
            boom = Some(frame);
        }
        if frame > 97 && sim.f().state != S::Attack {
            // (The scamp acts again on the next frame: 105 after the explosion's frame 97.)
            assert_eq!(frame + 1, 203, "FAF 203 [D]");
            break;
        }
    }
    assert_eq!(boom, Some(97));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(23) * sim.content.rules.damage_mult
    );
    assert!(peanut(&sim).is_none(), "no peanut");
    assert_eq!(sim.f().percent, Fx::ZERO, "no damage to the scamp [E]");
}

#[test]
fn the_shield_cancels_the_charge_from_frame_14() {
    // On frame 13 the shield does nothing (still raising the gun); on 14 it ends the move.
    for (frame, ends) in [(13u32, false), (14, true)] {
        let mut sim = duel();
        for f in 1..=frame {
            let b = if f == frame {
                SHIELD | SPECIAL
            } else {
                SPECIAL
            };
            sim.tick(inp(0, 0, b));
        }
        let still = sim.f().state == S::Attack && sim.f().move_id == MoveId::NSpecial as u8;
        assert_eq!(!still, ends, "shield on frame {frame}");
    }
}

#[test]
fn the_peanut_carries_the_references_hit() {
    let sim = duel();
    let w = &sim.content.weapons[usize::from(sim.content.fighters[11].weapon)];
    let fire = w.get(POPGUN_FIRE as u8);
    let p = fire.projectile.expect("the peanut");
    let h = p.hitbox;
    assert_eq!(
        (h.damage, h.angle, h.base_knockback, h.knockback_growth),
        (Fx::from_ratio(43, 10), 70, 40, 70)
    );
    assert_eq!(
        (h.hitlag, h.angle_type, h.clanks),
        (20, ANGLE_FORWARD, false)
    );
    assert_eq!(h.shield_damage, 77, "-1 on 4.3% [D]");
    let c = p.charged.expect("charge-scaled");
    assert_eq!((c.full, c.damage_to), (78, Fx::from_ratio(138, 10)));
    assert!(fire.keep_charge);
    let boom = w.get(POPGUN_MISFIRE as u8);
    let b = boom.hitboxes[0];
    assert_eq!(
        (
            b.damage,
            b.angle,
            b.base_knockback,
            b.knockback_growth,
            b.hitlag
        ),
        (Fx::from_int(23), 361, 30, 92, 150)
    );
    // (Radius 8.2 of the reference's units: 10.25 world tenths, rounded to 10 [D].)
    assert_eq!(b.radius, Fx::ONE);
    assert_eq!(
        b.start, 1,
        "on the misfire's second frame (the special's 97)"
    );
    let _ = fx(1, 1);
}

#[test]
fn the_peanut_hits_a_fighter_ahead_and_is_gone() {
    let mut sim = duel();
    sim.stand(1, Fx::from_int(-1), -1);
    for frame in 1..=40u32 {
        sim.tick(inp(0, 0, if frame == 1 { SPECIAL } else { 0 }));
    }
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_ratio(43, 10) * sim.content.rules.damage_mult
    );
    assert!(peanut(&sim).is_none());
}
