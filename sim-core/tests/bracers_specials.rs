//! The Bracers' specials on the hauler body (class 7, phase 2), modelled mechanically on a reference character: the charged punch
//! (E29 additive charge damage), the headbutt and its bury (E5), the spinning clothesline and the hand slap. Reference data and
//! sources: the eighth moveset's `REFERENCE_DATA.md` (`docs/movesets/`). Frames below are the reference's (frame 1 is the move's first,
//! the frame its button was pressed).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, JUMP, SHIELD, SPECIAL};
use sim_core::moves::{ChargeAdd, MoveId, Weapon, ARMOR_SUPER, PART_LIMB};
use sim_core::state::FighterState as S;
use sim_core::Fx;

const HAULER: u8 = 7;

/// The hauler (player 0) standing at 0 facing right and a duelist (player 1) standing `gap` ahead facing it, the others out.
fn duel(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars([HAULER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

fn arms(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(HAULER)].weapon)]
}

fn move_of(sim: &Sim, i: usize) -> MoveId {
    MoveId::from_index(sim.fighter(i).move_id)
}

/// Runs player 0 with `input(tick)` (ticks from 1) until player 1's percent rises; returns the tick, or None within `ticks`.
fn until_p1_hit(sim: &mut Sim, ticks: u32, input: impl Fn(u32) -> sim_core::Input) -> Option<u32> {
    let before = sim.fighter(1).percent;
    for t in 1..=ticks {
        sim.tick(input(t));
        if sim.fighter(1).percent > before {
            return Some(t);
        }
    }
    None
}

// ---- E29: additive charge damage ---------------------------------------------------------------------------------------------

#[test]
fn additive_charge_damage_is_the_references_formula_rounded_down() {
    let add = ChargeAdd {
        full: 110,
        add: Fx::from_int(18),
        floor: true,
    };
    // 10% grounded: 10 / 19 / 27 at 0 / 55 / 109 frames, 28 at full and never more.
    for (charge, want) in [(0u8, 10), (55, 19), (109, 27), (110, 28), (200, 28)] {
        assert_eq!(
            add.damage(Fx::from_int(10), charge),
            Fx::from_int(want),
            "{charge}"
        );
    }
    // 6% aerial: up to 24 at full; 23 one frame short.
    assert_eq!(add.damage(Fx::from_int(6), 109), Fx::from_int(23));
    // Without the floor it is the exact share.
    let exact = ChargeAdd {
        floor: false,
        ..add
    };
    assert_eq!(exact.damage(Fx::from_int(10), 11), fx(118, 10));
}

// ---- The charged punch -------------------------------------------------------------------------------------------------------

/// The duelist far enough that the charge never touches it but the punch's fist does.
const PUNCH_GAP: (i32, i32) = (24, 10);

/// Starts the punch (special on tick 1) and presses special again on tick `release` (move frame `release - 1`); returns the tick
/// the duelist was hit, and its damage (before the one-on-one multiplier).
fn punch_after(release: u32) -> (u32, Fx) {
    let mut sim = duel(fx(PUNCH_GAP.0, PUNCH_GAP.1));
    let tick = until_p1_hit(&mut sim, release + 80, |t| {
        if t == 1 || t == release {
            inp(0, 0, SPECIAL)
        } else {
            inp(0, 0, 0)
        }
    })
    .expect("the punch never hit");
    (tick, sim.fighter(1).percent / sim.content.rules.damage_mult)
}

#[test]
fn the_shortest_release_hits_on_frames_26_27_for_10_percent() {
    // Special again in the wind-up is kept: the punch comes out of the loop's start (frame 8), its hit on its own 19, overall 26.
    let (tick, damage) = punch_after(3);
    assert_eq!(tick, 26);
    assert_eq!(damage, Fx::from_int(10));
}

#[test]
fn a_release_deals_10_plus_18_x_its_charge_over_110_rounded_down() {
    // Released on move frame 7 + n, the charge is n frames: the hit comes 18 frames after the release.
    for (n, want) in [(0u32, 10), (55, 19), (100, 26), (109, 27)] {
        let release = 8 + n;
        let (tick, damage) = punch_after(release);
        assert_eq!(tick, release + 18, "charge {n}");
        assert_eq!(damage, Fx::from_int(want), "charge {n}");
    }
}

#[test]
fn a_full_charge_stores_itself_and_the_next_press_is_the_full_punch() {
    let mut sim = duel(Fx::from_int(12));
    sim.tick(inp(0, 0, SPECIAL));
    let mut ended = None;
    for t in 2..=140u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            ended = Some(t);
            break;
        }
    }
    // 110 frames of charge from frame 8: the stance ends on its own with the charge kept [E: the frame it becomes full, gap 1].
    assert_eq!(ended, Some(117));
    assert_eq!(sim.f().stored, 110);
    // The next press: the full punch, its hit on 19-20 counted from the press, FAF 48, and the store spent.
    let mut sim = duel(fx(PUNCH_GAP.0, PUNCH_GAP.1));
    sim.state.fighters[0].stored = 110;
    let tick = until_p1_hit(&mut sim, 40, |t| {
        inp(0, 0, if t == 1 { SPECIAL } else { 0 })
    });
    assert_eq!(tick, Some(19));
    assert_eq!(move_of(&sim, 0), MoveId::Ext2);
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(28) * sim.content.rules.damage_mult
    );
    assert_eq!(sim.f().stored, 0);
    let mut sim = duel(Fx::from_int(12));
    sim.state.fighters[0].stored = 110;
    sim.tick(inp(0, 0, SPECIAL));
    let mut faf = 0;
    for t in 2..=80u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            // (Out of the move on tick t, the move's frame t - 1: free on the next frame, the reference's t + 1.)
            faf = t + 1;
            break;
        }
    }
    assert_eq!(faf, 48, "free on frame 48");
}

#[test]
fn the_releases_and_full_punches_have_the_references_data() {
    let sim = duel(Fx::from_int(12));
    let w = arms(&sim);
    // Releases: hitbox 19-20, FAF 63, the lower arm intangible on 4-20, 10% / 6% raised by E29, no rebound.
    for (id, damage, bk, ks) in [(MoveId::Ext0, 10, 45, 70), (MoveId::Ext1, 6, 60, 65)] {
        let mv = w.get(id as u8);
        assert_eq!(mv.total_frames + 2, 63);
        assert_eq!(mv.hitboxes.len(), 3);
        for h in &mv.hitboxes {
            assert_eq!((h.start + 1, h.end + 1), (19, 20));
            assert_eq!(
                (h.damage, h.angle, h.base_knockback, h.knockback_growth),
                (Fx::from_int(damage), 361, bk, ks)
            );
            assert!(!h.rebounds && h.clanks);
        }
        let limb = mv.part_intangible[0];
        assert_eq!(
            (limb.start + 1, limb.end + 1, limb.parts),
            (4, 20, PART_LIMB)
        );
        assert_eq!(
            mv.charge_add,
            Some(ChargeAdd {
                full: 110,
                add: Fx::from_int(18),
                floor: true
            })
        );
        assert!(!mv.helpless_after);
    }
    assert_eq!(w.get(MoveId::Ext1 as u8).on_land, Some(MoveId::Ext0 as u8));
    // Full punches (one frame earlier in the data: they start on the wind-up's frame 1): super armor 9-20 / 15-20, hit 19-20, FAF 48.
    for (id, armor_from, damages, bk, ks) in [
        (MoveId::Ext2, 9u8, [28, 28, 28, 18], 35, 73),
        (MoveId::Ext3, 15, [25, 25, 25, 25], 40, 72),
    ] {
        let mv = w.get(id as u8);
        assert_eq!(mv.total_frames + 3, 48);
        let a = mv.armor.expect("armor");
        assert_eq!(
            (a.kind, a.start + 2, a.end + 2),
            (ARMOR_SUPER, armor_from, 20)
        );
        let got: Vec<Fx> = mv.hitboxes.iter().map(|h| h.damage).collect();
        assert_eq!(got, damages.map(Fx::from_int).to_vec());
        for h in &mv.hitboxes {
            assert_eq!((h.start + 2, h.end + 2), (19, 20));
            assert_eq!((h.base_knockback, h.knockback_growth), (bk, ks));
            assert!(!h.rebounds);
        }
        assert_eq!(mv.charge_add, None);
    }
    assert_eq!(w.get(MoveId::Ext3 as u8).on_land, Some(MoveId::Ext2 as u8));
    // The cancel: 4 frames of lag.
    assert_eq!(w.get(MoveId::Ext4 as u8).total_frames + 2, 5);
}

#[test]
fn a_shield_cancel_keeps_the_charge_and_the_next_punch_starts_from_it() {
    let mut sim = duel(Fx::from_int(12));
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 2..=30 {
        sim.tick(inp(0, 0, 0));
    }
    // Shield on frame 30: the charge counted on 7..29 (23 frames) is kept, and after the 4 frames of lag the shield is up.
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(move_of(&sim, 0), MoveId::Ext4);
    assert_eq!(sim.f().stored, 23);
    let mut shield_at = None;
    for t in 1..=8 {
        sim.tick(inp(0, 0, SHIELD));
        if sim.f().state == S::Shield {
            shield_at = Some(t);
            break;
        }
    }
    assert_eq!(
        shield_at,
        Some(4),
        "the shield comes out after 4 frames of lag"
    );
    // A jump cancels too.
    let mut sim = duel(Fx::from_int(12));
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(9, inp(0, 0, 0));
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(move_of(&sim, 0), MoveId::Ext4);
    assert_eq!(sim.f().stored, 3);
    // The next punch starts from the stored 23: released at once it deals floor(10 + 18 x 23 / 110) = 13.
    let mut sim = duel(fx(PUNCH_GAP.0, PUNCH_GAP.1));
    sim.state.fighters[0].stored = 23;
    let tick = until_p1_hit(&mut sim, 60, |t| {
        inp(0, 0, if t == 1 || t == 3 { SPECIAL } else { 0 })
    });
    assert_eq!(tick, Some(26));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(13) * sim.content.rules.damage_mult
    );
}

#[test]
fn a_hit_during_the_charge_loses_it() {
    let mut sim = duel(fx(15, 10));
    sim.state.fighters[0].stored = 40;
    sim.state.fighters[0].percent = Fx::from_int(100);
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(15, inp(0, 0, 0));
    assert_eq!(sim.f().stored, 0, "taken out of store while charging");
    // The duelist's forward smash knocks the hauler out of the charge.
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::FSmash);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::Hitstun {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Hitstun);
    assert_eq!(sim.f().stored, 0, "nothing kept");
}

#[test]
fn the_full_punch_is_super_armored_on_9_to_20_and_not_outside() {
    // The duelist (unhittable, so the punch never stops it) jabs the hauler on a chosen frame of the full punch; inside the armor the
    // hauler takes the damage and carries on, outside it is knocked out of the move.
    let jab_first = {
        let sim = duel(Fx::from_int(12));
        u32::from(sim.content.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].start)
    };
    for (frame, armored) in [
        (8u32, false),
        (9, true),
        (14, true),
        (20, true),
        (21, false),
    ] {
        let mut sim = duel(fx(15, 10));
        sim.state.fighters[0].stored = 110;
        sim.state.fighters[0].percent = Fx::from_int(150);
        sim.state.fighters[1].invuln = 255;
        // (A jab that never clanks, so it lands on the punch's own active frames too.)
        for h in &mut sim.content.weapons[0].moves[MoveId::Jab as usize].hitboxes {
            h.clanks = false;
        }
        let before = sim.f().percent;
        let mut hit_state = None;
        for t in 1..=40u32 {
            // The jab started so that its first active frame lands on the hauler's reference frame `frame` (tick `frame`).
            if t + jab_first == frame + 1 {
                sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::Jab);
            }
            sim.tick(inp(0, 0, if t == 1 { SPECIAL } else { 0 }));
            if sim.f().percent > before {
                assert_eq!(t, frame, "the jab lands on frame {frame}");
                // (The launch waits for the hitlag: see whether the move goes on after it.)
                while sim.f().hitlag > 0 {
                    sim.tick(inp(0, 0, 0));
                }
                sim.tick(inp(0, 0, 0));
                hit_state = Some(sim.f().state);
                break;
            }
        }
        let still = hit_state == Some(S::Attack);
        assert_eq!(still, armored, "hit on frame {frame}");
    }
}

#[test]
fn landing_in_the_aerial_full_punch_switches_to_the_grounded_one_on_the_same_frame() {
    let mut sim = duel(Fx::from_int(12));
    sim.state.fighters[0].stored = 110;
    sim.put_airborne(0, Fx::ZERO, fx(15, 10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert_eq!(move_of(&sim, 0), MoveId::Ext3);
    let mut landed = None;
    for _ in 0..30 {
        let frame = sim.f().state_frame;
        sim.tick(inp(0, 0, 0));
        if sim.f().grounded() {
            landed = Some((frame + 1, sim.f().state_frame));
            break;
        }
    }
    let (want, got) = landed.expect("it lands");
    assert_eq!(move_of(&sim, 0), MoveId::Ext2);
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(got, want, "the frame is kept");
    // No special fall after an aerial punch: it ends free in the air.
    let mut sim = duel(Fx::from_int(12));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SPECIAL));
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, SPECIAL));
    sim.ticks(8, inp(0, 0, 0));
    assert_eq!(move_of(&sim, 0), MoveId::Ext1);
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Airborne);
    let _ = ATTACK;
}

// ---- The headbutt ------------------------------------------------------------------------------------------------------------------

#[test]
fn the_headbutt_has_the_references_data() {
    use sim_core::moves::{EFFECT_BURY, TARGETS_AERIAL, TARGETS_ALL, TARGETS_GROUNDED};
    let sim = duel(Fx::from_int(12));
    let w = arms(&sim);
    let ground = w.get(MoveId::SideSpecial as u8);
    let air = w.get(MoveId::Ext5 as u8);
    assert_eq!(ground.air_version, Some(MoveId::Ext5 as u8));
    for (mv, bury_targets, bury_bk) in [(ground, TARGETS_GROUNDED, 40), (air, TARGETS_ALL, 30)] {
        assert_eq!(mv.total_frames + 2, 63);
        let a = mv.armor.expect("armor");
        assert_eq!((a.kind, a.start + 1, a.end + 1), (ARMOR_SUPER, 5, 14));
        assert_eq!((mv.ledge_front_from + 1, mv.ledge_back_from + 1), (2, 2));
        for h in &mv.hitboxes {
            assert_eq!((h.start + 1, h.end + 1), (20, 21));
            if h.angle == 270 {
                assert_eq!(
                    (
                        h.effect,
                        h.targets,
                        h.damage,
                        h.base_knockback,
                        h.knockback_growth
                    ),
                    (EFFECT_BURY, bury_targets, Fx::from_int(10), bury_bk, 40)
                );
                assert_eq!(h.shield_damage, 350);
            } else {
                assert_eq!(
                    (
                        h.angle,
                        h.targets,
                        h.damage,
                        h.base_knockback,
                        h.knockback_growth
                    ),
                    (70, TARGETS_AERIAL, Fx::from_int(8), 15, 40)
                );
                assert_eq!(h.shield_damage, 413);
            }
        }
    }
    assert_eq!((air.start_speed, air.start_speed_y), (67, 50));
    assert_eq!(air.air_gravity, Some(Fx::from_ratio(425, 80_000)));
    assert_eq!(air.on_land, Some(MoveId::SideSpecial as u8));
}

#[test]
fn the_headbutt_is_super_armored_on_5_to_14_and_buries_on_20() {
    // Started by the stick forward and special; a duelist standing in front is buried on frame 20.
    let mut sim = duel(fx(16, 10));
    let tick = until_p1_hit(&mut sim, 40, |t| {
        if t == 1 {
            inp(100, 0, SPECIAL)
        } else {
            inp(0, 0, 0)
        }
    });
    assert_eq!(tick, Some(20));
    assert_eq!(move_of(&sim, 0), MoveId::SideSpecial);
    assert_eq!(sim.fighter(1).state, S::Buried);
    // In the air the speed is cut on starting: x0.67 sideways, x0.5 upward.
    let mut sim = duel(Fx::from_int(12));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(30), fx(1, 2), fx(1, 2));
    sim.tick(inp(100, 0, SPECIAL));
    assert_eq!(move_of(&sim, 0), MoveId::Ext5);
    // (Kept on starting, then the frame's own drift and gravity: within a little of x0.67 and x0.5.)
    let v = sim.f().vel;
    assert!((v.x - fx(67, 200)).abs() < fx(3, 100), "{v:?}");
    assert!((v.y - fx(1, 4)).abs() < fx(3, 100), "{v:?}");
}

#[test]
fn a_shielded_headbutt_takes_25_more_percent_off_the_shield() {
    let mut sim = duel(fx(16, 10));
    sim.state.fighters[1].state = S::Shield;
    sim.state.fighters[1].state_frame = 20;
    let mut lost = Fx::ZERO;
    for t in 1..=25 {
        let hp = sim.fighter(1).shield_hp;
        sim.tick2(
            inp(
                if t == 1 { 100 } else { 0 },
                0,
                if t == 1 { SPECIAL } else { 0 },
            ),
            inp(0, 0, SHIELD),
        );
        lost = lost.max(hp - sim.fighter(1).shield_hp);
    }
    // On the hit's frame: (10 + 25) x the one-on-one multiplier, and that frame's own depletion.
    let want = Fx::from_int(35) * sim.content.rules.damage_mult + sim.content.rules.shield_deplete;
    assert!((lost - want).abs() < fx(1, 10), "lost {lost:?}");
}
