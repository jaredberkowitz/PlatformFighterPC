//! The Pockets' phase-2 neutral special (class 10, the settler): the pocket (E42). The reach's frames (intangible 5-23, catching
//! 8-23, FAF 51), what it can put away (another owner's projectile that can be reflected, a free item), the swap with a held item,
//! the release (frame 9, FAF 31, 1.9x, the trajectory turned forward, the clock reset), the rocket that frees its maker, a tool
//! that still counts as its maker's, the stock that loses it, and rollback. Reference data and the [E] readings: the eleventh
//! moveset's `REFERENCE_DATA.md` (`docs/movesets/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::buckler_slots::{BOMB_ITEM, BOMB_KIND};
use sim_core::moves::pockets_slots::{POCKET_OUT, ROCKET_KIND};
use sim_core::moves::MoveId;
use sim_core::state::{
    FighterState as S, ITEM_FREE, ITEM_HELD, ITEM_POCKETED, NONE, REST_FLYING, REST_POCKETED,
};
use sim_core::{step, Content, Fx, GameState, Projectile, Rng, Vec2, MAX_FIGHTERS};

const SETTLER: u8 = 10;
const CLAWS: u8 = 1;
const NEEDLES: u8 = 2;
const WANDERER: u8 = 9;

/// The settler (player 0) at 0 facing right, a `other` body (player 1) far to the right; the others out.
fn setup(other: u8) -> Sim {
    let mut sim = Sim::with_chars([SETTLER, other, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(8), -1);
    sim
}

/// A shot of player 1's `move_id` (its weapon's projectile) at `pos` going `vel`, in slot 0.
fn shot(sim: &mut Sim, move_id: MoveId, pos: Vec2, vel: Vec2) {
    let w = usize::from(sim.content.fighters[usize::from(sim.fighter(1).char_id)].weapon);
    let spec = sim.content.weapons[w].moves[move_id as usize]
        .projectile
        .expect("a projectile move");
    sim.state.projectiles[0] = Projectile {
        active: true,
        owner: 1,
        origin: 1,
        power: 100,
        move_id: move_id as u8,
        pos,
        vel,
        life: spec.life,
        facing: -1,
        kind: spec.tool.kind,
        rest: REST_FLYING,
        stale: move_id as u8,
        size: 100,
        ..Projectile::default()
    };
}

/// Where the reach's circle is for a settler at 0 facing right: 9.0 / 8 ahead, 7.0 / 8 up (world units).
fn in_the_reach() -> Vec2 {
    Vec2::new(fx(9, 8), fx(7, 8))
}

/// Starts the neutral special (tick 1 is its frame 1) and runs to `frame` (the reference's numbering).
fn special_to(sim: &mut Sim, frame: u32) {
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 2..=frame {
        sim.tick(inp(0, 0, 0));
    }
}

#[test]
fn the_reach_is_intangible_on_5_to_23_and_hands_back_control_on_51() {
    let mut sim = setup(CLAWS);
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    let mut intangible = Vec::new();
    let mut free = None;
    for t in 2..=60u32 {
        sim.tick(inp(0, 0, 0));
        if sim.f().invuln > 0 {
            intangible.push(t);
        }
        if free.is_none() && sim.f().state != S::Attack {
            free = Some(t + 1);
        }
    }
    assert_eq!(intangible.first(), Some(&5));
    assert_eq!(intangible.last(), Some(&23));
    assert_eq!(intangible.len(), 19);
    assert_eq!(free, Some(51), "FAF 51");
}

#[test]
fn a_shot_meeting_the_reach_on_8_to_23_is_put_away_and_not_before_or_after() {
    // A blaster shot set in the reach's circle on frame `f` (it is checked as it moves on the next frame).
    for (f, caught) in [(6u32, false), (7, true), (22, true), (23, false)] {
        let mut sim = setup(CLAWS);
        special_to(&mut sim, f);
        shot(
            &mut sim,
            MoveId::NSpecial,
            in_the_reach() + Vec2::new(fx(1, 10), Fx::ZERO),
            Vec2::new(-fx(1, 10), Fx::ZERO),
        );
        sim.tick(inp(0, 0, 0));
        let p = sim.state.projectiles[0];
        assert_eq!(
            p.active && p.rest == REST_POCKETED,
            caught,
            "set on frame {f}, met on {}",
            f + 1
        );
        if caught {
            assert_eq!(sim.f().pocket, 0);
            assert!(!sim.f().pocket_item);
            assert_eq!(sim.f().percent, Fx::ZERO);
        }
    }
}

/// The settler with player 1's blaster shot put away (its speed as caught: 1.0 a frame toward the settler); the reach over.
fn with_a_pocketed_shot() -> Sim {
    let mut sim = setup(CLAWS);
    special_to(&mut sim, 10);
    // (Set one frame's flight beyond the circle's middle: it meets the reach as it moves.)
    shot(
        &mut sim,
        MoveId::NSpecial,
        in_the_reach() + Vec2::new(Fx::ONE, -fx(1, 10)),
        Vec2::new(-Fx::ONE, fx(1, 10)),
    );
    for _ in 0..50 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Idle);
    assert_eq!(sim.state.projectiles[0].rest, REST_POCKETED);
    sim
}

#[test]
fn the_pocketed_shot_waits_unseen_and_frozen_with_no_time_limit() {
    let mut sim = with_a_pocketed_shot();
    let before = sim.state.projectiles[0];
    for _ in 0..600 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.state.projectiles[0], before);
    assert!(!sim_core::items::projectile_shown(&sim.state, 0));
}

#[test]
fn the_release_throws_it_on_frame_9_forward_at_its_caught_speed_and_control_comes_back_on_31() {
    let mut sim = with_a_pocketed_shot();
    sim.tick(inp(0, 0, SPECIAL));
    let mut out = None;
    let mut free = None;
    for t in 2..=40u32 {
        sim.tick(inp(0, 0, 0));
        if t == 2 {
            assert_eq!(
                sim.f().move_id,
                POCKET_OUT as u8,
                "the release from frame 2"
            );
        }
        let p = sim.state.projectiles[0];
        if out.is_none() && p.rest == REST_FLYING {
            out = Some(t);
            assert_eq!((p.owner, p.power, p.age), (0, 190, 0));
            // (Turned forward: the shot was going left at 1.0 and 0.1 up as it was caught.)
            assert_eq!(p.vel, Vec2::new(Fx::ONE, fx(1, 10)));
            assert_eq!(p.stale, POCKET_OUT as u8);
        }
        if free.is_none() && sim.f().state != S::Attack {
            free = Some(t + 1);
        }
    }
    assert_eq!(out, Some(9), "out on frame 9");
    assert_eq!(free, Some(31), "FAF 31");
    assert_eq!(sim.f().pocket, NONE);
}

#[test]
fn the_released_shot_hits_for_1_9_times_its_damage() {
    let mut sim = with_a_pocketed_shot();
    // Player 1 just in front, in the shot's way.
    sim.stand(1, fx(3, 1), -1);
    let w = usize::from(sim.content.fighters[usize::from(CLAWS)].weapon);
    let spec = sim.content.weapons[w].moves[MoveId::NSpecial as usize]
        .projectile
        .expect("the blaster");
    sim.tick(inp(0, 0, SPECIAL));
    let mut age = 0;
    for _ in 0..30 {
        if sim.state.projectiles[0].active {
            age = sim.state.projectiles[0].age + 1;
        }
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > Fx::ZERO {
            break;
        }
    }
    // (The blaster's damage falls off along its life: the hit's own damage at that age, then x1.9 and the one-on-one x1.2.)
    let progress = Fx::from_ratio(i32::from(age), i32::from(spec.life));
    let at_age = spec.hitbox.damage + (spec.end_damage - spec.hitbox.damage) * progress;
    let expected = at_age * Fx::from_int(190) / Fx::from_int(100) * sim.content.rules.damage_mult;
    assert!(
        (sim.fighter(1).percent - expected).abs() <= fx(1, 100),
        "{:?} against {:?}",
        sim.fighter(1).percent,
        expected
    );
}

#[test]
fn its_own_shots_a_line_and_a_burst_are_not_put_away() {
    // The settler's own shot (owner 0) passes the reach.
    let mut sim = setup(CLAWS);
    special_to(&mut sim, 10);
    shot(&mut sim, MoveId::NSpecial, in_the_reach(), Vec2::ZERO);
    sim.state.projectiles[0].owner = 0;
    sim.tick(inp(0, 0, 0));
    assert_ne!(sim.state.projectiles[0].rest, REST_POCKETED);
    // The needles' line (held by its thrower: `Tool::pocket` never).
    let mut sim = setup(NEEDLES);
    special_to(&mut sim, 10);
    shot(
        &mut sim,
        MoveId::SideSpecial,
        in_the_reach(),
        Vec2::new(-fx(1, 10), Fx::ZERO),
    );
    sim.tick(inp(0, 0, 0));
    assert_ne!(sim.state.projectiles[0].rest, REST_POCKETED);
    assert_eq!(sim.f().pocket, NONE);
}

#[test]
fn a_pocketed_rocket_frees_its_maker_and_a_pocketed_boomerang_does_not() {
    // Two settlers: player 1's rocket put away by player 0 no longer counts as player 1's (it may set down another: the reference's
    // exception).
    let mut sim = setup(SETTLER);
    special_to(&mut sim, 10);
    shot(
        &mut sim,
        MoveId::SideSpecial,
        in_the_reach(),
        Vec2::new(-fx(1, 10), Fx::ZERO),
    );
    assert_eq!(sim.state.projectiles[0].kind, ROCKET_KIND);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.state.projectiles[0].rest, REST_POCKETED);
    assert_eq!(
        sim_core::scripting::owned(&sim.state.projectiles, 1, ROCKET_KIND, 0),
        0
    );
    // The wanderer's boomerang (kind 1, one at a time) still counts as his: put away, it is still in play.
    let mut sim = setup(WANDERER);
    special_to(&mut sim, 10);
    shot(
        &mut sim,
        MoveId::SideSpecial,
        in_the_reach(),
        Vec2::new(-fx(1, 10), Fx::ZERO),
    );
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.state.projectiles[0].rest, REST_POCKETED);
    let kind = sim.state.projectiles[0].kind;
    assert!(kind > 0);
    assert_eq!(
        sim_core::scripting::owned(&sim.state.projectiles, 1, kind, 0),
        1
    );
}

/// The wanderer's bomb (player 1's item) lying at `x` on the stage, credited to player 1. Returns its slot.
fn bomb_at(sim: &mut Sim, x: Fx) -> usize {
    let n = sim_core::items::make_as(
        &mut sim.state,
        &sim.content,
        1,
        WANDERER,
        BOMB_ITEM,
        Vec2::new(x, Fx::ZERO),
        0,
    )
    .expect("a free item slot");
    sim.state.items[n].platform = 0;
    n
}

#[test]
fn an_item_in_the_reach_is_put_away_frozen_and_comes_out_into_the_hand() {
    let mut sim = setup(WANDERER);
    let n = bomb_at(&mut sim, fx(9, 8));
    sim.state.items[n].fuse = 0;
    let age = sim.state.items[n].age;
    special_to(&mut sim, 8);
    assert_eq!(sim.state.items[n].rest, ITEM_POCKETED);
    assert_eq!((sim.f().pocket, sim.f().pocket_item), (n as i8, true));
    assert!(sim_core::items::view(&sim.state, n).is_none(), "unseen");
    // Frozen: its clock stops (the bomb's 1800-frame life).
    for _ in 0..100 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.state.items[n].age, age + 7);
    // Still the wanderer's (in play for its limit).
    assert_eq!(sim_core::items::owned(&sim.state.items, 1, BOMB_KIND), 1);
    // The release: into the hand on frame 9, its clock from 0.
    sim.tick(inp(0, 0, SPECIAL));
    for _ in 2..=9 {
        sim.tick(inp(0, 0, 0));
    }
    let it = sim.state.items[n];
    // (Its clock starts again from 0 as it comes out, and has run the frame since.)
    assert_eq!((it.rest, it.holder, it.age), (ITEM_HELD, 0, 1));
    assert_eq!(sim.f().holding, n as i8);
    assert_eq!(sim.f().pocket, NONE);
}

#[test]
fn a_held_item_is_put_away_and_swapped_with_the_one_in_the_pocket() {
    let mut sim = setup(WANDERER);
    // A bomb in the pocket, another in the hand.
    let a = bomb_at(&mut sim, fx(9, 8));
    special_to(&mut sim, 8);
    assert_eq!(sim.f().pocket, a as i8);
    for _ in 0..50 {
        sim.tick(inp(0, 0, 0));
    }
    let b = bomb_at(&mut sim, fx(40, 1));
    sim.state.items[b].holder = 0;
    sim.state.items[b].rest = ITEM_HELD;
    sim.state.items[b].platform = NONE;
    sim.state.fighters[0].holding = b as i8;
    // Holding an item, the press is the reach: on frame 8 the held one goes in and the pocketed one comes out.
    special_to(&mut sim, 8);
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    assert_eq!(sim.state.items[b].rest, ITEM_POCKETED);
    assert_eq!(sim.state.items[a].rest, ITEM_HELD);
    assert_eq!((sim.f().holding, sim.f().pocket), (a as i8, b as i8));
}

#[test]
fn a_knock_out_loses_what_was_put_away() {
    let mut sim = with_a_pocketed_shot();
    sim.state.fighters[0].pos.y = sim.content.stage.blast_bottom - Fx::ONE;
    sim.tick(inp(0, 0, 0));
    assert!(!sim.state.projectiles[0].active);
    assert_eq!(sim.f().pocket, NONE);
    // An item too.
    let mut sim = setup(WANDERER);
    let n = bomb_at(&mut sim, fx(9, 8));
    special_to(&mut sim, 8);
    assert_eq!(sim.state.items[n].rest, ITEM_POCKETED);
    sim.state.fighters[0].pos.y = sim.content.stage.blast_bottom - Fx::ONE;
    sim.tick(inp(0, 0, 0));
    assert!(!sim.state.items[n].active);
    let _ = ITEM_FREE;
}

#[test]
fn pocketing_resimulates_exactly() {
    // The settler puts away a blaster shot among random play by the claws, then takes it out: a run restored from snapshots taken
    // before the catch, while it waits and after the release ends the same.
    for seed in 0..6u64 {
        let base = with_a_pocketed_shot();
        let content: Content = base.content.clone();
        let mut inputs = sim_core::fuzz::random_inputs(&mut Rng::new(seed), 200);
        for (i, frame) in inputs.iter_mut().enumerate() {
            frame[0] = if i == 20 {
                inp(0, 0, SPECIAL)
            } else {
                inp(0, 0, 0)
            };
            frame[2] = sim_core::Input::default();
            frame[3] = sim_core::Input::default();
        }
        let mut s: GameState = base.state;
        for at in [0usize, 10, 40] {
            let mut straight = s;
            for i in &inputs[at..] {
                step(&mut straight, &content, i);
            }
            let mut restored = s;
            let _ = MAX_FIGHTERS;
            for i in &inputs[at..] {
                step(&mut restored, &content, i);
            }
            assert_eq!(
                straight.checksum(),
                restored.checksum(),
                "seed {seed} from {at}"
            );
            if at == 0 {
                for i in &inputs[..10] {
                    step(&mut s, &content, i);
                }
            } else {
                for i in &inputs[10..40] {
                    step(&mut s, &content, i);
                }
            }
        }
    }
}
