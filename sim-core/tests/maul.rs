//! The third class: the hammer bruiser's maul and body. Its normals are the sword's stretched out (slower, bigger, harder), and its
//! three specials are plain attacks that each hit something at some range.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, SPECIAL};
use sim_core::moves::{claws, longsword, maul, MoveId};
use sim_core::{Fx, GameState};

const BRUISER: [u8; 4] = [2, 2, 2, 2];

fn duel(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars(BRUISER);
    sim.state = GameState::new_with_active(&sim.content, 1, BRUISER, 0b0011);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-7) + gap, -1);
    sim
}

/// Whether `input` (held for one tick, then released) damages a target standing `gap` tenths ahead.
fn hits_at(gap: i32, input: sim_core::Input) -> bool {
    let mut sim = duel(fx(gap, 10));
    let before = sim.fighter(1).percent;
    sim.tick(input);
    for _ in 0..120 {
        sim.tick(inp(0, 0, 0));
    }
    sim.fighter(1).percent > before
}

#[test]
fn the_maul_is_slower_and_harder_than_the_sword_it_is_built_from() {
    let (sword, hammer) = (longsword(), maul());
    for id in [
        MoveId::Jab,
        MoveId::FTilt,
        MoveId::UTilt,
        MoveId::DTilt,
        MoveId::FAir,
        MoveId::FSmash,
    ] {
        let (s, h) = (&sword.moves[id as usize], &hammer.moves[id as usize]);
        assert!(h.total_frames > s.total_frames, "{} is slower", id.name());
        let first = |m: &sim_core::moves::Move| m.hitboxes.iter().map(|b| b.start).min().unwrap();
        assert!(first(h) > first(s), "{} starts up later", id.name());
        let best = |m: &sim_core::moves::Move| m.hitboxes.iter().map(|b| b.damage).max().unwrap();
        assert!(best(h) > best(s), "{} hits harder", id.name());
    }
}

#[test]
fn the_maul_keeps_the_sword_recovery_and_none_of_its_scripts() {
    let (sword, hammer) = (longsword(), maul());
    assert_eq!(
        hammer.moves[MoveId::UpSpecial as usize],
        sword.moves[MoveId::UpSpecial as usize],
        "the rising slash is the recovery"
    );
    for m in &hammer.moves {
        assert!(m.script.is_none() && m.projectile_script.is_none());
    }
    for i in 0..MoveId::COUNT as u8 {
        if MoveId::from_index(i).is_ext() {
            assert!(hammer.moves[usize::from(i)].is_empty());
        }
    }
    assert_ne!(hammer, claws());
}

#[test]
fn each_special_hits_something_at_some_range() {
    for (name, input) in [
        ("neutral special", inp(0, 0, SPECIAL)),
        ("side special", inp(127, 0, SPECIAL)),
        ("down special", inp(0, -127, SPECIAL)),
        ("forward tilt", inp(60, 0, ATTACK)),
    ] {
        let reach = (6..40).any(|gap| hits_at(gap, input));
        assert!(reach, "{name} never connects");
    }
}

#[test]
fn the_bruiser_is_heavy_and_slow_on_its_feet() {
    let c = sim_core::Content::placeholder();
    let (duelist, brawler, bruiser) = (&c.fighters[0], &c.fighters[1], &c.fighters[2]);
    assert!(bruiser.weight > duelist.weight && bruiser.weight > brawler.weight);
    assert!(bruiser.run_speed < duelist.run_speed && bruiser.run_speed < brawler.run_speed);
    assert_eq!(usize::from(bruiser.weapon), 2);
}
