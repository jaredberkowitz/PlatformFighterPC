use sim_core::input::buttons::{ATTACK, JUMP};
use sim_core::run::data::RunContent;
use sim_core::run::{grunt_slot, new_run, step_run, MAX_MOBS, PLAYER};
use sim_core::state::NONE;
use sim_core::{collision, Content, Fx, Input, Vec2, MAX_FIGHTERS};

#[test]
fn diag() {
    let rc = RunContent::placeholder();
    let mut world = Content::placeholder();
    let mut st = new_run(&mut world, &rc, 0, 0);
    let mut stocks = st.fighters[0].stocks;
    let mut last_pct = Fx::ZERO;
    for n in 0..3000u32 {
        let f = st.fighters[PLAYER];
        let mut input = Input {
            stick_x: 127,
            stick_y: 0,
            buttons: 0,
        };
        let stage = &world.stage;
        let ahead = Vec2::new(f.pos.x + Fx::from_int(2), f.pos.y + Fx::from_ratio(1, 2));
        let gap =
            collision::surface_below(stage, ahead, false).is_none_or(|(_, d)| d > Fx::from_int(2));
        let near = (0..MAX_MOBS).filter(|&k| st.run.mobs[k].active).any(|k| {
            let d = st.fighters[grunt_slot(k)].pos - f.pos;
            d.x > Fx::ZERO && d.x < Fx::from_ratio(5, 2) && d.y.abs() < Fx::from_int(2)
        });
        if near && n % 2 == 0 {
            input.buttons |= ATTACK;
            input.stick_x = 0;
        } else if gap && n % 2 == 0 && (f.platform != NONE || f.air_jumps_left > 0) {
            input.buttons |= JUMP;
        }
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = input;
        step_run(&mut st, &mut world, &rc, &inputs);
        let f = st.fighters[PLAYER];
        if f.percent > last_pct {
            let hitters: Vec<String> = (0..MAX_MOBS)
                .filter(|&k| st.run.mobs[k].active)
                .map(|k| {
                    let g = st.fighters[grunt_slot(k)];
                    format!(
                        "k{} {:?} m{} {:?}",
                        st.run.mobs[k].kind,
                        g.state,
                        g.move_id,
                        (g.pos.x - f.pos.x).floor_int()
                    )
                })
                .collect();
            println!(
                "{n}: hit {:?} -> {:?} by {:?}",
                last_pct.floor_int(),
                f.percent.floor_int(),
                hitters
            );
        }
        last_pct = f.percent;
        if n > 1400 && n % 100 == 0 {
            for k in 0..MAX_MOBS {
                if st.run.mobs[k].active {
                    let g = st.fighters[grunt_slot(k)];
                    println!(
                        "   g{k} kind {} mode {} st {:?} pos {},{} room {}",
                        st.run.mobs[k].kind,
                        st.run.mobs[k].mode,
                        g.state,
                        g.pos.x.floor_int(),
                        g.pos.y.floor_int(),
                        st.run.mobs[k].room
                    );
                }
            }
            println!("{n}: pos {:?},{:?} {:?} plat {} cam {:?} wall {:?} end {:?} room {} ahead {} gate {} slots {:?}", f.pos.x.floor_int(), f.pos.y.floor_int(), f.state, f.platform, st.run.cam_x.floor_int(), st.run.wall_x.floor_int(), st.run.right_limit().floor_int(), st.run.room, st.run.ahead().1, st.run.gate_closed, st.run.slots.iter().filter(|s| s.live).map(|s| (s.seq, s.room, s.x.floor_int(), s.end.floor_int())).collect::<Vec<_>>());
        }
        if f.stocks < stocks {
            println!(
                "{n}: LIFE LOST at {:?} {:?} state {:?}",
                f.pos, last_pct, f.state
            );
            stocks = f.stocks;
        }
        if n == 2000 {
            for p in world.stage.platforms.iter().filter(|p| {
                p.right > Fx::from_int(40) && p.left < Fx::from_int(70) && p.y > Fx::from_int(-30)
            }) {
                println!(
                    "plat {}..{} y {} bottom {} pt {}",
                    p.left.floor_int(),
                    p.right.floor_int(),
                    p.y.floor_int(),
                    p.bottom.floor_int(),
                    p.pass_through
                );
            }
            for k in 0..4 {
                let g = st.fighters[grunt_slot(k)];
                println!(
                    "g{k} hist {:?} st {:?} {} mode {} step {}",
                    g.history[0], g.state, g.state_frame, st.run.mobs[k].mode, st.run.mobs[k].step
                );
            }
        }
        if st.run.over {
            break;
        }
    }
}
