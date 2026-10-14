//! A jab's continuable window past its FAF (owner decision 58, sim 95: `Move::late_next`): once the jab has ended, a jab press up to
//! the window's last reference frame starts the next hit, not a new jab; after it, or once the fighter does anything else, a press is a
//! new jab again. Every jab whose reference window runs past its FAF carries it (the Embers' jabs 1 and 2, the Gauntlets' jabs 1 and 2,
//! the Greatsword's jab 1, the Bracers' jab 1, the Clutch's jab 1, the Buckler's jabs 1 and 2, the Popgun's jabs 1 and 2); the others
//! do not.

mod common;

use common::{inp, Sim};
use sim_core::input::buttons::ATTACK;
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// (class, the jab, the hit it carries on into, its FAF, the continuable window's last frame), every [R].
const LATE: [(u8, MoveId, MoveId, u8, u8); 11] = [
    (3, MoveId::Jab, MoveId::Jab2, 20, 24),
    (3, MoveId::Jab2, MoveId::Jab3, 22, 24),
    (4, MoveId::Jab, MoveId::Jab2, 18, 30),
    (4, MoveId::Jab2, MoveId::Jab3, 19, 30),
    (6, MoveId::Jab, MoveId::Jab2, 30, 30),
    (7, MoveId::Jab, MoveId::Jab2, 25, 28),
    (8, MoveId::Jab, MoveId::Jab2, 18, 30),
    (9, MoveId::Jab, MoveId::Jab2, 24, 35),
    (9, MoveId::Jab2, MoveId::Jab3, 26, 35),
    // The Popgun (class 11, sim 117): jab 1 to 30, jab 2 to 30.
    (11, MoveId::Jab, MoveId::Jab2, 20, 30),
    (11, MoveId::Jab2, MoveId::Jab3, 21, 30),
];

/// Starts `jab` for class `class` standing alone (the move's frame 1 is tick 0), presses attack once on reference frame `press` of the
/// move (with the stick down on `crouch_at`, if any), and returns the move started on that tick (None if no attack started).
fn press_at(class: u8, jab: MoveId, press: u8, crouch_at: Option<u8>) -> Option<MoveId> {
    let mut sim = Sim::with_chars([class, class, class, class]);
    for i in 2..4 {
        sim.state.fighters[i].active = false;
    }
    // (A second fighter keeps the match going; it stands far away.)
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], jab);
    // Tick k is the move's reference frame k + 1.
    for frame in 2..=press {
        let down = crouch_at == Some(frame);
        let b = if frame == press { ATTACK } else { 0 };
        sim.tick(inp(0, if down { -127 } else { 0 }, b));
    }
    let f = sim.f();
    (f.state == S::Attack && f.state_frame == 0).then(|| MoveId::from_index(f.move_id))
}

#[test]
fn the_late_windows_are_the_references() {
    let sim = Sim::new();
    for (class, jab, next, faf, last) in LATE {
        let w = &sim.content.weapons[usize::from(sim.content.fighters[usize::from(class)].weapon)];
        let mv = w.get(jab as u8);
        assert_eq!(
            u16::from(mv.total_frames) + 2,
            u16::from(faf),
            "class {class} {jab:?}"
        );
        assert_eq!(
            mv.late_next,
            Some((next as u8, last - 1)),
            "class {class} {jab:?}"
        );
    }
    // The other jabs have none (their windows end before their FAF, or they have no reference window).
    for (class, w) in sim.content.weapons.iter().enumerate() {
        for jab in [MoveId::Jab, MoveId::Jab2, MoveId::Jab3] {
            let listed = LATE.iter().any(|&(c, j, ..)| {
                usize::from(sim.content.fighters[usize::from(c)].weapon) == class && j == jab
            });
            assert_eq!(
                w.get(jab as u8).late_next.is_some(),
                listed,
                "weapon {class} {jab:?}"
            );
        }
    }
}

#[test]
fn a_press_after_the_faf_and_inside_the_window_starts_the_next_hit() {
    for (class, jab, next, faf, last) in LATE {
        // After the FAF, on the window's last frame: the next hit.
        assert_eq!(
            press_at(class, jab, last, None),
            Some(next),
            "class {class} {jab:?} on {last}"
        );
        if last > faf {
            assert_eq!(
                press_at(class, jab, faf, None),
                Some(next),
                "class {class} {jab:?} on {faf}"
            );
        }
        // One frame later: a new jab 1.
        assert_eq!(
            press_at(class, jab, last + 1, None),
            Some(MoveId::Jab),
            "class {class} {jab:?} on {}",
            last + 1
        );
    }
}

#[test]
fn doing_anything_else_after_the_jab_closes_the_window() {
    // The Clutch's jab 1 (FAF 18, window to 30): a crouch on 20, then a press on 24, is a new jab 1.
    assert_eq!(press_at(8, MoveId::Jab, 24, None), Some(MoveId::Jab2));
    assert_eq!(press_at(8, MoveId::Jab, 24, Some(20)), Some(MoveId::Jab));
}
