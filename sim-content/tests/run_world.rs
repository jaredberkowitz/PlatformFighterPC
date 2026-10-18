//! The run's world (R2a): rooms as the generator really lays them over many seeds are passable, with the rules' jumps, in the
//! variant they were laid as (`sim_content::run::passable`, the proof `check_run` makes for every variant). R2b: a branching fork's
//! roads are both checked, and taken in turn.

use sim_content::run::passable;
use sim_core::run::{new_run, step_run, RoomSlot, RunContent};
use sim_core::{Content, Fx, Input, MAX_FIGHTERS};

#[test]
fn laid_rooms_are_passable_over_many_seeds() {
    let rc = RunContent::placeholder();
    let mut checked = 0;
    for seed in 0..40u64 {
        let mut world = Content::placeholder();
        let mut state = new_run(&mut world, &rc, seed, 0);
        let mut seen: Vec<u32> = Vec::new();
        for step in 0..30u64 {
            for s in state.run.slots.iter().filter(|s| s.live) {
                if seen.contains(&s.seq) {
                    continue;
                }
                seen.push(s.seq);
                // (The proof lays a room at the origin.)
                let at_origin = RoomSlot {
                    x: Fx::ZERO,
                    end: s.end - s.x,
                    y: Fx::ZERO,
                    y_end: s.y_end - s.y,
                    ..*s
                };
                let room = &rc.rooms[usize::from(s.room)];
                passable(&rc, room, &at_origin, rc.rules.max_step, rc.rules.max_gap)
                    .unwrap_or_else(|e| panic!("seed {seed}, room {} ({}): {e}", s.seq, room.name));
                checked += 1;
            }
            // Into the next room (at a branching fork, one road or the other in turn).
            let (ahead, n) = state.run.ahead();
            let next = if state.run.branches().is_some() {
                ahead[((seed + step) % 2) as usize]
            } else {
                ahead[0]
            };
            assert!(n >= 1, "seed {seed}: a room ahead");
            // (The camera on it, as if it had followed: the blast zone follows the camera.)
            state.run.cam_x = next.x - Fx::from_int(8);
            let f = &mut state.fighters[0];
            f.pos = sim_core::Vec2::new(next.x + Fx::from_int(3), next.y + Fx::from_int(2));
            f.vel = sim_core::Vec2::ZERO;
            f.platform = sim_core::state::NONE;
            f.state = sim_core::state::FighterState::Airborne;
            f.invuln = 30;
            for m in state.run.mobs.iter_mut() {
                m.active = false;
            }
            step_run(
                &mut state,
                &mut world,
                &rc,
                &[Input::default(); MAX_FIGHTERS],
            );
            state.run.gate_closed = false;
        }
    }
    assert!(checked > 1000, "{checked} rooms");
}

/// R3: the run's stat items rebuild the fighter through the creator's recipe arithmetic. For every class, a built-in fighter with
/// speed, spring and weight items (in the run, stepped) has exactly the parameters the character creator's recipe with those stats
/// gives (two speed points, two jump points and one weight point a stack), and with none the fighter's own.
#[test]
fn stat_items_equal_the_recipe_output() {
    use sim_content::recipe::Recipe;
    use sim_core::run::data::{ITEM_SPEED, ITEM_SPRING, ITEM_WEIGHT};
    use sim_core::run::items;
    let rc = RunContent::placeholder();
    let base = Content::placeholder();
    for class in 0..sim_content::recipe::CLASSES {
        let neutral = Recipe {
            class,
            ..Recipe::default()
        };
        assert_eq!(
            base.fighters[usize::from(class)],
            neutral.params(),
            "class {class}: the built-in fighter is its class's neutral recipe"
        );
        for (speed, jump, weight) in [(0u8, 0u8, 0u8), (1, 0, 0), (0, 2, 0), (0, 0, 3), (2, 1, 2)] {
            let mut world = Content::placeholder();
            let mut state = new_run(&mut world, &rc, 1, class);
            for (kind, n) in [
                (ITEM_SPEED, speed),
                (ITEM_SPRING, jump),
                (ITEM_WEIGHT, weight),
            ] {
                for _ in 0..n {
                    items::grant(&mut state, &rc, items::index_of(&rc, kind).unwrap());
                }
            }
            step_run(
                &mut state,
                &mut world,
                &rc,
                &[Input::default(); MAX_FIGHTERS],
            );
            let recipe = Recipe {
                class,
                speed: 5 + 2 * speed,
                jump: 5 + 2 * jump,
                weight: 5 + weight,
                ..Recipe::default()
            };
            assert_eq!(
                world.fighters[usize::from(class)],
                recipe.params(),
                "class {class}, {speed} speed, {jump} spring, {weight} weight"
            );
        }
    }
}
