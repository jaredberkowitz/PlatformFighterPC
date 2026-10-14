//! The Pockets' phase-2 down special (class 10, the settler): the tree (E43). The stage the press picks (planting, the failed
//! planting in the air, watering, the axe), the sapling (where, when, one each, its life), the water (the push, the growing), the
//! growth's two hits and their timing, the tree (hit points, life, shots stopped), the axe (frames, damage, the chop on a side,
//! the freeze), the falling tree and the wood chip, and rollback. Reference data and the [E] readings: the eleventh moveset's
//! `REFERENCE_DATA.md` (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::pockets_slots::{
    AXE, CHIP_KIND, FALL_KIND, PLANT, PLANT_FAILS, SAPLING_ITEM, SAPLING_KIND, TREE_ITEM,
    TREE_KIND, WATER,
};
use sim_core::moves::MoveId;
use sim_core::state::{FighterState as S, Item, NONE, REST_FLYING};
use sim_core::{step, Content, Fx, GameState, Projectile, Rng, Vec2};

const SETTLER: u8 = 10;

/// The settler (player 0) at 0 facing right, a duelist (player 1) out of the way at 8; the others out.
fn setup() -> Sim {
    let mut sim = Sim::with_chars([SETTLER, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(8), -1);
    sim
}

fn down_special(sim: &mut Sim) {
    sim.tick(inp(0, -127, SPECIAL));
}

fn items_of(sim: &Sim, kind: u8) -> Vec<(usize, Item)> {
    sim.state
        .items
        .iter()
        .enumerate()
        .filter(|(_, it)| it.active && it.kind == kind)
        .map(|(n, it)| (n, *it))
        .collect()
}

/// Ticks with nothing pressed until the settler can act again; returns the frame it could (the press was frame 1).
fn run_to_free(sim: &mut Sim, pressed_at: u32, now: u32) -> u32 {
    let mut t = now;
    while sim.f().state == S::Attack {
        sim.tick(inp(0, 0, 0));
        t += 1;
        assert!(t < pressed_at + 200, "the move never ended");
    }
    t + 1 - pressed_at + 1
}

/// The settler's own placed item `spec` of its weapon at `x` on the stage.
fn put(sim: &mut Sim, spec: u8, x: Fx) -> usize {
    let n = sim_core::items::make(
        &mut sim.state,
        &sim.content,
        0,
        spec,
        Vec2::new(x, Fx::ZERO),
        0,
    )
    .expect("a free item slot");
    sim.state.items[n].platform = 0;
    n
}

#[test]
fn the_press_plants_a_sapling_10_ahead_on_frame_20_and_hands_back_control_on_39() {
    let mut sim = setup();
    down_special(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, PLANT as u8, "the planting from frame 2");
    let mut appeared = None;
    for t in 3..=30u32 {
        sim.tick(inp(0, 0, 0));
        if appeared.is_none() && !items_of(&sim, SAPLING_KIND).is_empty() {
            appeared = Some(t);
        }
    }
    assert_eq!(appeared, Some(20));
    let (_, s) = items_of(&sim, SAPLING_KIND)[0];
    assert_eq!(s.pos, Vec2::new(Fx::from_ratio(10, 8), Fx::ZERO));
    assert_eq!((s.platform, s.origin, s.life), (0, 0, 840));
    let free = run_to_free(&mut sim, 1, 30);
    assert_eq!(free, 39, "FAF 39");
}

#[test]
fn in_the_air_the_planting_fails() {
    let mut sim = setup();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    down_special(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, PLANT_FAILS as u8);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(items_of(&sim, SAPLING_KIND).is_empty());
}

#[test]
fn the_sapling_lasts_840_frames_and_planting_on_one_pulls_it_up() {
    let mut sim = setup();
    let n = put(&mut sim, SAPLING_ITEM, fx(10, 8));
    for _ in 0..839 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.state.items[n].active);
    sim.tick(inp(0, 0, 0));
    assert!(!sim.state.items[n].active, "gone after 840");
    // Two settlers: player 1 plants right where player 0's sapling stands; player 0's is pulled up, one sapling is left.
    let mut sim = Sim::with_chars([SETTLER, SETTLER, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, fx(25, 10), -1);
    let mine = put(&mut sim, SAPLING_ITEM, fx(10, 8));
    sim.tick2(inp(0, 0, 0), inp(0, -127, SPECIAL));
    for _ in 0..25 {
        sim.tick(inp(0, 0, 0));
    }
    let left = items_of(&sim, SAPLING_KIND);
    // (Pulled up: the new one took its slot.)
    assert_eq!(sim.state.items[mine].origin, 1, "pulled up");
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].1.origin, 1);
}

#[test]
fn with_a_sapling_out_the_press_waters_it_a_push_with_no_damage_and_control_comes_back_on_50() {
    let mut sim = setup();
    put(&mut sim, SAPLING_ITEM, fx(10, 8));
    // A duelist right in the water's way.
    sim.stand(1, fx(15, 10), -1);
    down_special(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, WATER as u8);
    let mut pushed = false;
    for _ in 3..=20 {
        sim.tick(inp(0, 0, 0));
        let v = sim.fighter(1);
        assert_eq!(v.percent, Fx::ZERO);
        assert_ne!(v.state, S::Hitstun);
        pushed |= v.pos.x > fx(15, 10);
    }
    assert!(pushed, "the water pushes");
    let free = run_to_free(&mut sim, 1, 20);
    assert_eq!(free, 50, "FAF 50");
}

/// The settler waters a sapling it planted 10 ahead (a full watering from where it planted). Returns the sim on the frame the water
/// first touched the sapling, and that frame (the press was frame 1).
fn watered() -> (Sim, u32) {
    let mut sim = setup();
    let n = put(&mut sim, SAPLING_ITEM, fx(10, 8));
    down_special(&mut sim);
    for t in 2..=30u32 {
        sim.tick(inp(0, 0, 0));
        if sim.state.items[n].water > 0 {
            return (sim, t);
        }
    }
    panic!("the water never touched the sapling");
}

#[test]
fn a_full_watering_grows_the_tree_and_its_clean_hit_comes_46_frames_after_the_first_drop() {
    let (mut sim, first) = watered();
    // The duelist stands at the sapling once the water is over (the water would push it away).
    let mut hit = None;
    for t in first + 1..=first + 70 {
        if t == first + 20 {
            sim.stand(1, fx(10, 8), -1);
        }
        sim.tick(inp(0, 0, 0));
        if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
            hit = Some(t);
        }
    }
    assert_eq!(hit.map(|h| h - first), Some(46));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(18) * sim.content.rules.damage_mult
    );
    assert!(items_of(&sim, SAPLING_KIND).is_empty());
    let trees = items_of(&sim, TREE_KIND);
    assert_eq!(trees.len(), 1);
    assert_eq!(trees[0].1.hp, Fx::from_int(30));
    assert_eq!(trees[0].1.life, 600);
}

#[test]
fn half_a_watering_is_kept_and_a_second_one_grows_it_sooner() {
    // Water stopped short (the settler shields out after 4 frames of water: the move is over, the sapling keeps what it had).
    let mut sim = setup();
    let n = put(&mut sim, SAPLING_ITEM, fx(10, 8));
    down_special(&mut sim);
    let mut t = 1;
    while sim.state.items[n].water < 4 {
        sim.tick(inp(0, 0, 0));
        t += 1;
        assert!(t < 30);
    }
    // (Knock the water away: the shot is removed.)
    for p in &mut sim.state.projectiles {
        p.active = false;
    }
    let kept = sim.state.items[n].water;
    assert!((4..8).contains(&kept));
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.state.items[n].water, kept, "kept");
    assert!(sim.state.items[n].active, "not grown yet");
}

#[test]
fn the_growth_late_hit_deals_13() {
    let (mut sim, first) = watered();
    // At the tree from 40 frames after the first drop: past the clean window (its frames 14-19).
    for t in first + 1..=first + 80 {
        if t == first + 60 {
            sim.stand(1, fx(10, 8), -1);
        }
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(13) * sim.content.rules.damage_mult
    );
}

/// The settler with its tree standing `x` ahead (made now, full hit points).
fn with_a_tree(x: Fx) -> (Sim, usize) {
    let mut sim = setup();
    let n = put(&mut sim, TREE_ITEM, x);
    (sim, n)
}

#[test]
fn with_a_tree_out_the_press_swings_the_axe_on_6_to_8_for_14_and_hands_back_control_on_48() {
    // The tree far behind; a duelist in front.
    let (mut sim, _) = with_a_tree(-Fx::from_int(6));
    sim.stand(1, fx(12, 10), -1);
    down_special(&mut sim);
    let mut hit = None;
    for t in 2..=20u32 {
        sim.tick(inp(0, 0, 0));
        if t == 2 {
            assert_eq!(sim.f().move_id, AXE as u8);
        }
        if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
            hit = Some(t);
        }
    }
    assert_eq!(hit, Some(6));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(14) * sim.content.rules.damage_mult
    );
    // (The duelist's hitlag froze the settler too: count from the swing.)
    let mut sim2 = with_a_tree(-Fx::from_int(6)).0;
    down_special(&mut sim2);
    let free = run_to_free(&mut sim2, 1, 1);
    assert_eq!(free, 48, "FAF 48");
}

#[test]
fn two_axe_cuts_on_one_side_fell_the_tree_and_one_on_each_side_does_not() {
    // The tree right in front: the first cut freezes the settler 7 frames and takes 14 hit points; the second on that side fells it.
    let (mut sim, n) = with_a_tree(fx(15, 10));
    down_special(&mut sim);
    let mut froze = 0;
    for _ in 2..=10 {
        sim.tick(inp(0, 0, 0));
        froze = froze.max(sim.f().hitlag);
    }
    assert_eq!(froze, 7, "frozen by the tree");
    assert_eq!(sim.state.items[n].hp, Fx::from_int(16));
    assert_eq!(sim.state.items[n].cuts, 1);
    run_to_free(&mut sim, 1, 10);
    down_special(&mut sim);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(!sim.state.items[n].active, "felled");
    let falling: Vec<Projectile> = sim
        .state
        .projectiles
        .iter()
        .filter(|p| p.active && p.kind == FALL_KIND)
        .copied()
        .collect();
    assert_eq!(falling.len(), 1);
    assert_eq!((falling[0].owner, falling[0].facing), (0, 1));
    // One cut from each side: still standing.
    let (mut sim, n) = with_a_tree(fx(15, 10));
    down_special(&mut sim);
    run_to_free(&mut sim, 1, 1);
    sim.state.fighters[0].pos.x = fx(30, 10);
    sim.state.fighters[0].facing = -1;
    down_special(&mut sim);
    run_to_free(&mut sim, 1, 1);
    assert!(sim.state.items[n].active, "one cut a side");
    assert_eq!(sim.state.items[n].cuts, 0x11);
}

#[test]
fn the_falling_tree_hits_for_25_from_its_frame_14_and_is_gone_on_the_stage() {
    let (mut sim, _) = with_a_tree(fx(15, 10));
    sim.state.items[0].cuts = 1;
    // A duelist beyond the tree.
    sim.stand(1, fx(35, 10), -1);
    down_special(&mut sim);
    let mut felled_at = None;
    let mut hit_at = None;
    for t in 2..=120u32 {
        sim.tick(inp(0, 0, 0));
        if felled_at.is_none()
            && sim
                .state
                .projectiles
                .iter()
                .any(|p| p.active && p.kind == FALL_KIND)
        {
            felled_at = Some(t);
        }
        if hit_at.is_none() && sim.fighter(1).percent > Fx::from_int(15) {
            hit_at = Some(t);
        }
    }
    let (f, h) = (felled_at.expect("felled"), hit_at.expect("hit"));
    assert!(h - f >= 13, "no hit before its frame 14 ({})", h - f);
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(25) * sim.content.rules.damage_mult
    );
    assert!(
        !sim.state
            .projectiles
            .iter()
            .any(|p| p.active && p.kind == FALL_KIND),
        "gone after its bounce"
    );
}

#[test]
fn the_tree_stops_shots_loses_hit_points_and_goes_after_600_frames() {
    let (mut sim, n) = with_a_tree(fx(3, 1));
    // A 5% shot of player 1's (the duelist has none: any shot of a weapon will do; the settler's pellet) flying into the tree.
    let w = usize::from(sim.content.fighters[usize::from(SETTLER)].weapon);
    let spec = sim.content.weapons[w].moves[MoveId::FAir as usize]
        .projectile
        .expect("the pellet");
    sim.state.fighters[1].char_id = SETTLER;
    sim.state.projectiles[0] = Projectile {
        active: true,
        owner: 1,
        origin: 1,
        power: 100,
        move_id: MoveId::FAir as u8,
        pos: Vec2::new(fx(45, 10), fx(12, 10)),
        vel: Vec2::new(-fx(3, 10), Fx::ZERO),
        life: spec.life,
        size: 100,
        facing: -1,
        rest: REST_FLYING,
        stale: MoveId::FAir as u8,
        ..Projectile::default()
    };
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(!sim.state.projectiles[0].active, "used up on the tree");
    assert!(sim.state.items[n].hp < Fx::from_int(30));
    assert_eq!(sim.fighter(0).percent, Fx::ZERO);
    let (mut sim, n) = with_a_tree(fx(3, 1));
    for _ in 0..599 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.state.items[n].active);
    sim.tick(inp(0, 0, 0));
    assert!(!sim.state.items[n].active, "600 frames");
}

#[test]
fn a_felled_tree_leaves_a_wood_chip_when_the_draw_says_so() {
    let (mut sim, n) = with_a_tree(fx(15, 10));
    let w = usize::from(sim.content.fighters[usize::from(SETTLER)].weapon);
    if let Some(pl) = sim.content.weapons[w].items[usize::from(TREE_ITEM)]
        .placed
        .as_mut()
    {
        pl.chip_chance = 1000;
    }
    sim.state.items[n].cuts = 1;
    down_special(&mut sim);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    let chips = items_of(&sim, CHIP_KIND);
    assert_eq!(chips.len(), 1);
    assert_eq!(chips[0].1.origin, 0);
    // The chance as built: 48.5% [R].
    let base = Content::placeholder();
    assert_eq!(
        base.weapons[w].items[usize::from(TREE_ITEM)]
            .placed
            .map(|p| p.chip_chance),
        Some(485)
    );
}

#[test]
fn the_stage_follows_what_is_out_until_the_falling_tree_is_gone() {
    // With the falling tree still out the press is the axe; once it is gone, the planting again.
    let (mut sim, n) = with_a_tree(fx(15, 10));
    sim.state.items[n].cuts = 1;
    down_special(&mut sim);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    // (Lifted high, the falling tree is still falling when the settler can act again.)
    for p in &mut sim.state.projectiles {
        if p.active && p.kind == FALL_KIND {
            p.pos.y += Fx::from_int(30);
        }
    }
    run_to_free(&mut sim, 1, 10);
    assert!(sim
        .state
        .projectiles
        .iter()
        .any(|p| p.active && p.kind == FALL_KIND));
    down_special(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, AXE as u8);
    for _ in 0..300 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(!sim
        .state
        .projectiles
        .iter()
        .any(|p| p.active && p.kind == FALL_KIND));
    down_special(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, PLANT as u8);
    let _ = NONE;
}

#[test]
fn the_tree_resimulates_exactly() {
    // Two settlers pressing down special every 20 frames among random play (plant, water, grow, chop, fall): a run restored from a
    // mid-run snapshot ends the same, and trees grew along the way.
    let content = Content::placeholder();
    let chars = [SETTLER, SETTLER, 0, 0];
    let mut trees = 0;
    for seed in 0..8u64 {
        let mut inputs = sim_core::fuzz::random_inputs(&mut Rng::new(seed), 900);
        for (i, frame) in inputs.iter_mut().enumerate() {
            frame[0] = if i % 20 == 0 {
                inp(0, -127, SPECIAL)
            } else {
                inp(0, 0, 0)
            };
            if i % 20 == 10 {
                frame[1] = inp(0, -127, SPECIAL);
            }
        }
        let mut s = GameState::new(&content, seed, chars);
        let mut snap = None;
        for (k, i) in inputs.iter().enumerate() {
            if k == 450 {
                snap = Some(s);
            }
            step(&mut s, &content, i);
            trees += s
                .items
                .iter()
                .filter(|it| it.active && it.kind == TREE_KIND)
                .count();
        }
        let mut r = snap.expect("a snapshot");
        for i in &inputs[450..] {
            step(&mut r, &content, i);
        }
        assert_eq!(r.checksum(), s.checksum(), "seed {seed}");
    }
    assert!(trees > 0, "trees grew");
}
