//! The reference's angle types on the older movesets (sim 67, owner decision 21): after E26 (`Hitbox::angle_type`) every move of the
//! five older weapons with a box reaching the fighter's centre or behind it carries its reference angle type (ssbwiki.com hitbox
//! tables, 13.0.1). The table below is the list of changes; the behaviour test shows one that changes where a hit sends.

mod common;

use common::{fx, inp, Sim};
use sim_core::moves::{
    MoveId, ANGLE_BACKWARD, ANGLE_BY_BOX, ANGLE_BY_POSITION, ANGLE_FORWARD, HIT_NORMAL,
};
use sim_core::state::FighterState as S;
use sim_core::{Content, Fx};

const P: u8 = ANGLE_BY_POSITION;
const F: u8 = ANGLE_FORWARD;
const B: u8 = ANGLE_BACKWARD;

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
        (1, UpSpecial, {
            let mut v = vec![P];
            v.extend([ANGLE_BY_BOX; 8]);
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
        // (The gauntlets' own specials, sim 68-71, are not re-checked for angle types yet.)
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
    assert_eq!(checked, 64);
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
fn a_target_in_front_of_the_embers_up_smash_is_now_sent_forward() {
    // The reference's up smash is angle type 0: 83 degrees away from the attacker. Its early boxes sit behind the head, so by box (the
    // old rule) a target in front was sent back over the attacker's head; by position it goes forward, the way it is.
    assert_eq!(embers_up_smash_sends(4, true), -1, "the old rule");
    assert_eq!(embers_up_smash_sends(4, false), 1);
    assert_eq!(embers_up_smash_sends(-4, false), -1);
}
