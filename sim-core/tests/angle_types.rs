//! The reference's angle types on the older movesets (sim 67, owner decision 21): after E26 (`Hitbox::angle_type`) every move of the
//! five older weapons with a box reaching the fighter's centre or behind it carries its reference angle type (ssbwiki.com hitbox
//! tables, 13.0.1). The table below is the list of changes; the behaviour test shows one that changes where a hit sends.

mod common;

use common::{fx, inp, Sim};
use sim_core::moves::{
    MoveId, ANGLE_BACKWARD, ANGLE_BY_BOX, ANGLE_BY_POSITION, ANGLE_BY_TRAVEL, ANGLE_FORWARD,
    HIT_NORMAL,
};
use sim_core::state::FighterState as S;
use sim_core::{Content, Fx};

const P: u8 = ANGLE_BY_POSITION;
const F: u8 = ANGLE_FORWARD;
const B: u8 = ANGLE_BACKWARD;
const T: u8 = ANGLE_BY_TRAVEL;

/// (weapon, move, the angle type of each damaging hitbox in order; one entry for "all of them")
fn expected() -> Vec<(usize, MoveId, Vec<u8>)> {
    use MoveId::*;
    let all = |t: u8| vec![t];
    vec![
        // Longsword (weapon 0).
        (0, UTilt, all(P)),
        (0, FSmash, all(F)),
        (0, USmash, all(P)),
        (0, DSmash, all(P)),
        (0, NAir, all(P)),
        (0, FAir, all(F)),
        (0, BAir, all(B)),
        (0, UAir, all(P)),
        (0, DAir, all(P)),
        (0, GetUpAttack, all(P)),
        // Claws (weapon 1).
        (1, UTilt, all(P)),
        (1, DTilt, all(P)),
        (1, USmash, all(P)),
        (1, DSmash, all(P)),
        (1, NAir, all(P)),
        (1, BAir, all(B)),
        (1, UAir, all(P)),
        (1, DAir, all(P)),
        (1, DownSpecial, all(P)),
        (1, Ext0, all(F)),
        (1, Ext1, all(F)),
        (1, GetUpAttack, all(P)),
        // The up special's wind-up hit by position, its eight looping hits from the direction of travel (type 1: sim 73, owner
        // decision 23).
        (1, UpSpecial, {
            let mut v = vec![P];
            v.extend([T; 8]);
            v
        }),
        // Needles (weapon 2).
        (2, FTilt, all(F)),
        (2, UTilt, vec![F, F, F, P, P, P]),
        (2, DTilt, all(P)),
        (2, USmash, all(P)),
        (2, DSmash, vec![F, F, B, B, F, B]),
        (2, NAir, all(P)),
        (2, FAir, all(F)),
        (2, BAir, all(B)),
        (2, UAir, vec![F, P, P]),
        (2, DAir, all(P)),
        (2, Ext0, all(F)),
        (2, UpSpecial, all(P)),
        (2, DownSpecial, all(F)),
        (2, Ext6, all(F)),
        (2, GetUpAttack, all(P)),
        // Embers (weapon 3).
        (3, Jab2, all(F)),
        (3, FTilt, all(F)),
        (3, UTilt, all(P)),
        (3, USmash, all(P)),
        (3, DSmash, all(P)),
        (3, NAir, all(P)),
        (3, FAir, all(F)),
        (3, BAir, all(B)),
        (3, UAir, all(P)),
        (3, DAir, all(P)),
        (3, Ext0, all(P)),
        (3, UpSpecial, all(F)),
        (3, BThrow, all(P)),
        (3, LedgeAttack, all(P)),
        (3, GetUpAttack, all(P)),
        // Gauntlets (weapon 4).
        (4, FTilt, all(F)),
        (4, DTilt, all(P)),
        (4, USmash, vec![P, P, P, F, F, F, F]),
        (4, DSmash, all(P)),
        (4, NAir, all(F)),
        (4, FAir, all(F)),
        (4, BAir, all(B)),
        (4, UAir, all(P)),
        (4, DAir, all(P)),
        (4, LedgeAttack, all(F)),
        (4, GetUpAttack, all(P)),
        // The Gauntlets' specials (sim 73, ssbwiki.com raw tables: no `af` is the template's 0, by position): the four punches by
        // position; the uppercut and the overhand forward; the leap's explosion on bystanders forward (its row on the held
        // fighter is a pummel row, its throw keeps the default); the slide kick and its off-edge version by position, the dive kick
        // forward and its landing hit by position.
        (4, NSpecial, all(P)),
        (4, Ext0, all(P)),
        (4, Ext1, all(P)),
        (4, Ext2, all(P)),
        (4, Ext3, all(F)),
        (4, Ext5, all(F)),
        (4, Ext6, all(F)),
        (4, DownSpecial, all(P)),
        (4, Ext9, all(P)),
        (4, Ext7, all(F)),
        (4, Ext8, all(P)),
        // Greatsword (weapon 6, phase 1, sim 81: ssbwiki.com raw tables, 13.0.1). Every damaging normal carries its type: the jabs,
        // tilts but the up tilt, dash attack, forward smash, forward air and ledge attack forward (3); the up tilt, neutral, up and down
        // airs and the get-up attack by position (0); the back air backward (4); the up smash forward (3) but its late hit by position;
        // the down smash's first hit forward and its second backward.
        (6, Jab, all(F)),
        (6, Jab2, all(F)),
        (6, Jab3, all(F)),
        (6, FTilt, all(F)),
        (6, UTilt, all(P)),
        (6, DTilt, all(F)),
        (6, DashAttack, all(F)),
        (6, FSmash, all(F)),
        (6, USmash, [vec![F; 8], vec![P; 3]].concat()),
        (6, DSmash, [vec![F; 10], vec![B; 3]].concat()),
        (6, NAir, all(P)),
        (6, FAir, all(F)),
        (6, BAir, all(B)),
        (6, UAir, all(P)),
        (6, DAir, all(P)),
        (6, LedgeAttack, all(F)),
        (6, GetUpAttack, all(P)),
        // Bracers (weapon 7, phase 1, sim 86: ssbwiki.com raw tables, 13.0.1): the jabs, forward tilt, dash attack, forward smash,
        // forward air and ledge attack forward (3); the up and down tilts, up and down smashes, neutral, up and down airs and the get-up
        // attack by position (no `af`: 0); the back air backward (4).
        (7, Jab, all(F)),
        (7, Jab2, all(F)),
        (7, FTilt, all(F)),
        (7, UTilt, all(P)),
        (7, DTilt, all(P)),
        (7, DashAttack, all(F)),
        (7, FSmash, all(F)),
        (7, USmash, all(P)),
        (7, DSmash, all(P)),
        (7, NAir, all(P)),
        (7, FAir, all(F)),
        (7, BAir, all(B)),
        (7, UAir, all(P)),
        (7, DAir, all(P)),
        (7, LedgeAttack, all(F)),
        (7, GetUpAttack, all(P)),
    ]
}

#[test]
fn the_older_movesets_carry_their_reference_angle_types() {
    let content = Content::placeholder();
    let mut checked = 0;
    for (w, id, types) in expected() {
        let boxes: Vec<u8> = content.weapons[w].moves[id as usize]
            .hitboxes
            .iter()
            .filter(|h| h.kind == HIT_NORMAL)
            .map(|h| h.angle_type)
            .collect();
        assert!(!boxes.is_empty(), "weapon {w} {id:?} has hitboxes");
        if types.len() == 1 {
            assert!(
                boxes.iter().all(|&t| t == types[0]),
                "weapon {w} {id:?}: {boxes:?}"
            );
        } else {
            assert_eq!(boxes, types, "weapon {w} {id:?}");
        }
        checked += 1;
    }
    assert_eq!(checked, 108);
    // Every move not in the list keeps the default (boxes wholly in front, where it cannot change anything; and the moves whose
    // reference type is not built, like the claws' up special loop).
    for w in 0..5 {
        for id in MoveId::all() {
            if expected().iter().any(|(ew, eid, _)| *ew == w && *eid == id) {
                continue;
            }
            assert!(
                content.weapons[w].moves[id as usize]
                    .hitboxes
                    .iter()
                    .all(|h| h.angle_type == ANGLE_BY_BOX),
                "weapon {w} {id:?}"
            );
        }
    }
}

/// The sign of the horizontal launch of a duelist standing `victim_x` tenths from an embers fighter (player 0, facing right) hit by
/// the up smash's early boxes (frames 9-10, which sit just behind the head).
fn embers_up_smash_sends(victim_x: i32, by_box: bool) -> i32 {
    let mut sim = Sim::with_chars([3, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    if by_box {
        for h in &mut sim.content.weapons[3].moves[MoveId::USmash as usize].hitboxes {
            h.angle_type = ANGLE_BY_BOX;
        }
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(victim_x, 10), -1);
    sim.state.fighters[1].percent = Fx::from_int(60);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::USmash);
    sim.state.fighters[0].state_frame = 7;
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
fn the_longsword_claws_and_needles_get_up_attacks_launch_at_48_degrees() {
    // Owner decision 27 (sim 73): the reference's get-up attacks are 7% at angle 48, BK 80, KS 48 (ssbwiki.com raw tables, 13.0.1);
    // they were written as 361. The Needles use the claws' get-up attack (the reference's is also 48).
    let content = Content::placeholder();
    for w in 0..3 {
        let boxes = &content.weapons[w].moves[MoveId::GetUpAttack as usize].hitboxes;
        assert_eq!(boxes.len(), 2, "weapon {w}");
        for h in boxes {
            assert_eq!(
                (h.angle, h.base_knockback, h.knockback_growth),
                (48, 80, 48),
                "weapon {w}"
            );
            assert_eq!(h.damage, Fx::from_int(7), "weapon {w}");
        }
    }
}

#[test]
fn a_target_in_front_of_the_embers_up_smash_is_now_sent_forward() {
    // The reference's up smash is angle type 0: 83 degrees away from the attacker. Its early boxes sit behind the head, so by box (the
    // old rule) a target in front was sent back over the attacker's head; by position it goes forward, the way it is.
    assert_eq!(embers_up_smash_sends(4, true), -1, "the old rule");
    assert_eq!(embers_up_smash_sends(4, false), 1);
    assert_eq!(embers_up_smash_sends(-4, false), -1);
}
