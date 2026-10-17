use sim_core::run::{self, new_run, step_run, RunContent};
use sim_core::{Content, Fx, Input, Vec2, MAX_FIGHTERS};
#[test]
fn dbg() {
    let rc = RunContent::placeholder();
    let mut world = Content::placeholder();
    let mut st = new_run(&mut world, &rc, 42, 1);
    for k in 0..14 {
        let (ahead, n) = st.run.ahead();
        let next = ahead[n - 1];
        let target = next.seq;
        st.run.cam_x = (next.x - rc.rules.view_width * rc.rules.follow).max(st.run.wall_x);
        st.run.gate_closed = false;
        run::clear_grunts(&mut st, &world);
        let f = &mut st.fighters[0];
        f.pos = Vec2::new(next.x + rc.rules.enter_depth + Fx::ONE, next.y + Fx::from_int(6));
        f.vel = Vec2::ZERO;
        f.platform = -1;
        f.state = sim_core::state::FighterState::Airborne;
        f.invuln = 30;
        step_run(&mut st, &mut world, &rc, &[Input::default(); MAX_FIGHTERS]);
        let names: Vec<_> = st.run.slots.iter().filter(|s| s.live).map(|s| (s.seq, rc.rooms[s.room as usize].name.clone())).collect();
        println!("{k}: entered {} room {} {:?} doors {}", st.run.room == target, st.run.room, names, st.run.slots.iter().filter(|s| s.live).filter_map(|s| run::door_of(&rc, s)).count());
    }
}
