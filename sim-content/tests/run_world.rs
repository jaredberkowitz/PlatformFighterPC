//! The run's world (R2a): rooms as the generator really lays them over many seeds are passable, with the rules' jumps, in the
//! variant they were laid as (`sim_content::run::passable`, the proof `check_run` makes for every variant).

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
        for _ in 0..30 {
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
            // Into the next room.
            let next = *state.run.slot_of(state.run.room + 1).unwrap();
            // (The camera on it, as if it had followed: the blast zone follows the camera.)
            state.run.cam_x = next.x - Fx::from_int(8);
            let f = &mut state.fighters[0];
            f.pos = sim_core::Vec2::new(next.x + Fx::from_int(3), next.y + Fx::from_int(12));
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
