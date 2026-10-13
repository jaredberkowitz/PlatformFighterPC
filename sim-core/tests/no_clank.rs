//! The reference's no-clank boxes (`clang=f`, E3c `Hitbox::clanks`) on the Staff (class 5, sim 73): every box of the rapid jab and
//! its finisher, the dash attack, the three smashes, the down air, the pummel, the ledge and the get-up attacks never clanks, as in
//! the Staff's reference data (its port's `REFERENCE_DATA.md` under `docs/movesets/`; ssbwiki.com raw hitbox tables, 13.0.1); every
//! other normal still does.

mod common;

use common::{fx, inp, Sim};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{Content, Fx};

const SEER: u8 = 5;

const NO_CLANK: [MoveId; 11] = [
    MoveId::Ext10,
    MoveId::Ext11,
    MoveId::Ext12,
    MoveId::DashAttack,
    MoveId::FSmash,
    MoveId::USmash,
    MoveId::DSmash,
    MoveId::DAir,
    MoveId::Pummel,
    MoveId::LedgeAttack,
    MoveId::GetUpAttack,
];

#[test]
fn the_staffs_reference_no_clank_moves_never_clank_and_its_other_normals_do() {
    let content = Content::placeholder();
    let w = &content.weapons[usize::from(content.fighters[usize::from(SEER)].weapon)];
    for id in NO_CLANK {
        let boxes = &w.moves[id as usize].hitboxes;
        assert!(!boxes.is_empty(), "{id:?} has hitboxes");
        assert!(boxes.iter().all(|h| !h.clanks), "{id:?}");
    }
    for id in [
        MoveId::Jab,
        MoveId::FTilt,
        MoveId::UTilt,
        MoveId::DTilt,
        MoveId::NAir,
        MoveId::FAir,
        MoveId::BAir,
        MoveId::UAir,
        MoveId::Grab,
        MoveId::FThrow,
        MoveId::BThrow,
        MoveId::UThrow,
        MoveId::DThrow,
    ] {
        assert!(
            w.moves[id as usize].hitboxes.iter().all(|h| h.clanks),
            "{id:?} clanks"
        );
    }
}

/// Two seers 1.6 apart, facing each other, each in its forward smash one frame before the hit (reference frame 18). Player `i`'s
/// forward smash is made to clank again where `clanks[i]`. Returns (anyone rebounded, both hit).
fn smashes_meet(clanks: [bool; 2]) -> (bool, bool) {
    let mut sim = Sim::with_chars([SEER, SEER, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    if clanks[0] || clanks[1] {
        // A copy of the staff whose forward smash clanks.
        let w = usize::from(sim.content.fighters[usize::from(SEER)].weapon);
        let mut copy = sim.content.weapons[w].clone();
        for h in &mut copy.moves[MoveId::FSmash as usize].hitboxes {
            h.clanks = true;
        }
        sim.content.weapons.push(copy);
        let mut p = sim.content.fighters[usize::from(SEER)];
        p.weapon = (sim.content.weapons.len() - 1) as u8;
        sim.content.fighters.push(p);
        let copy_id = (sim.content.fighters.len() - 1) as u8;
        for (i, &clanking) in clanks.iter().enumerate() {
            if clanking {
                sim.state.fighters[i].char_id = copy_id;
            }
        }
    }
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(16, 10), -1);
    for i in 0..2 {
        sim_core::fighter::begin_attack(&mut sim.state.fighters[i], MoveId::FSmash);
        sim.state.fighters[i].state_frame = 16;
    }
    let mut rebound = false;
    for _ in 0..6 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        rebound |= sim.f().state == S::Rebound || sim.fighter(1).state == S::Rebound;
    }
    (
        rebound,
        sim.f().percent > Fx::ZERO && sim.fighter(1).percent > Fx::ZERO,
    )
}

#[test]
fn two_staff_forward_smashes_that_meet_trade_instead_of_clanking() {
    assert_eq!(smashes_meet([false, false]), (false, true), "a trade");
    // A no-clank box on either side is enough: one clanking forward smash still trades.
    assert_eq!(smashes_meet([false, true]), (false, true));
    // Two that clank (as before sim 73) clank and both rebound: the setup does reach the boxes meeting.
    assert_eq!(smashes_meet([true, true]), (true, false));
}
