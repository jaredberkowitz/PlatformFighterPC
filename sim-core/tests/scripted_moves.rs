//! Scripts attached to moves and projectiles, run through the real simulation.

mod common;

use common::{fx, inp, Sim};
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{Motion, Move, MoveId};
use sim_core::state::FighterState as S;
use sim_core::{step, Content, Fx, GameState, Rng, MAX_FIGHTERS};
use sim_script::{Kind, Program};

fn fighter_script(src: &str) -> Program {
    Program::compile(Kind::Fighter, src).unwrap_or_else(|e| panic!("{e}"))
}

fn projectile_script(src: &str) -> Program {
    Program::compile(Kind::Projectile, src).unwrap_or_else(|e| panic!("{e}"))
}

/// A move with no hitboxes, `total` frames long, driven by `src`.
fn scripted(total: u8, src: &str) -> Move {
    Move {
        total_frames: total,
        script: Some(fighter_script(src)),
        ..Move::empty()
    }
}

/// Player 0 (the sword character) standing on the ground with `mv` as its neutral special.
fn with_special(mv: Move) -> Sim {
    let mut sim = Sim::with_chars([0, 1, 0, 1]);
    sim.content.weapons[0].moves[MoveId::NSpecial as usize] = mv;
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(7), -1);
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    sim
}

fn start(sim: &mut Sim) {
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().state, S::Attack, "the special should have started");
}

#[test]
fn a_script_can_move_the_fighter_like_a_data_motion() {
    let mut data = with_special(Move {
        total_frames: 30,
        motion: vec![Motion {
            start: 5,
            end: 14,
            vx: fx(1, 2),
            vy: Fx::ZERO,
        }],
        ..Move::empty()
    });
    let mut script = with_special(scripted(
        30,
        "if frame >= 5 && frame <= 14 { set_vel(0.5, 0); }",
    ));
    start(&mut data);
    start(&mut script);
    let start_x = script.f().pos.x;
    for t in 0..40 {
        data.tick(inp(0, 0, 0));
        script.tick(inp(0, 0, 0));
        assert_eq!(
            data.state.checksum(),
            script.state.checksum(),
            "diverged at tick {t}"
        );
    }
    assert!(script.f().pos.x > start_x + Fx::from_int(3), "it moved");
}

#[test]
fn a_script_reads_the_stick_relative_to_facing() {
    // Pushing the stick the way the fighter faces moves it forward; turned around, the same push goes backward.
    for (facing, stick) in [(1i8, 127i8), (-1, -127)] {
        let mut sim = with_special(scripted(20, "set_vel(stick_x, 0);"));
        sim.state.fighters[0].facing = facing;
        sim.tick(inp(0, 0, SPECIAL));
        let x0 = sim.f().pos.x;
        sim.tick(inp(stick, 0, 0));
        sim.tick(inp(stick, 0, 0));
        let moved = (sim.f().pos.x - x0).mul_int(i32::from(facing));
        assert!(moved > Fx::ZERO, "facing {facing}: moved {moved:?}");
    }
}

#[test]
fn goto_switches_to_another_move() {
    let mut sim = with_special(scripted(40, "if frame == 3 { goto(2); }"));
    start(&mut sim);
    for _ in 0..4 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().move_id, MoveId::UTilt as u8);
    assert_eq!(sim.f().state, S::Attack);
}

#[test]
fn end_finishes_the_move_early() {
    let mut sim = with_special(scripted(60, "if frame == 10 { end(); }"));
    start(&mut sim);
    let mut ended = None;
    for t in 2..70 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Attack {
            ended = Some(t);
            break;
        }
    }
    let ended = ended.expect("the move should have ended");
    assert!((10..=14).contains(&ended), "ended on tick {ended}");
}

#[test]
fn stall_holds_the_move_but_never_forever() {
    let mut sim = with_special(scripted(20, "stall();"));
    start(&mut sim);
    let mut ticks = 1;
    while sim.f().state == S::Attack {
        sim.tick(inp(0, 0, 0));
        ticks += 1;
        assert!(ticks < 200, "a stalling script held the fighter forever");
    }
    let limit = usize::from(sim.content.rules.charge_frames);
    assert!(
        ticks > 20 + limit / 2,
        "it did hold the move for a while ({ticks})"
    );
}

#[test]
fn variables_persist_through_a_move_and_reset_for_the_next() {
    let mut sim = with_special(scripted(30, "var n; n += 1; if n == 6 { end(); }"));
    start(&mut sim);
    for _ in 0..8 {
        sim.tick(inp(0, 0, 0));
    }
    assert_ne!(sim.f().state, S::Attack, "n reached 6 and the move ended");
    // Wait for control, then start the move again: the counter starts from zero again.
    sim.ticks(30, inp(0, 0, 0));
    sim.tick(inp(0, 0, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().vars[0],
        Fx::ONE.raw(),
        "one run of the script so far"
    );
}

#[test]
fn a_script_can_fire_a_projectile_with_its_own_velocity() {
    let mut sim = with_special(scripted(30, "if frame == 4 { spawn(1.5, 2, 0.4, 0.2); }"));
    // Borrow the blaster's projectile description.
    sim.content.weapons[0].moves[MoveId::NSpecial as usize].projectile =
        sim.content.weapons[1].moves[MoveId::NSpecial as usize].projectile;
    start(&mut sim);
    let mut seen = None;
    for _ in 0..12 {
        let pos = sim.f().pos;
        sim.tick(inp(0, 0, 0));
        if let Some(p) = sim.state.projectiles.iter().find(|p| p.active) {
            seen = Some((*p, pos));
            break;
        }
    }
    let (p, from) = seen.expect("a projectile should appear");
    assert_eq!(p.vel.x, fx(4, 10));
    assert_eq!(p.vel.y, fx(2, 10));
    assert_eq!(p.pos.x, from.x + fx(15, 10));
    assert_eq!(p.pos.y, from.y + Fx::from_int(2));
}

#[test]
fn a_facing_left_fighter_fires_the_other_way() {
    let mut sim = with_special(scripted(30, "if frame == 4 { spawn(1.5, 2, 0.4, 0.2); }"));
    sim.content.weapons[0].moves[MoveId::NSpecial as usize].projectile =
        sim.content.weapons[1].moves[MoveId::NSpecial as usize].projectile;
    sim.state.fighters[0].facing = -1;
    start(&mut sim);
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    let p = sim.state.projectiles.iter().find(|p| p.active).unwrap();
    assert!(p.vel.x < Fx::ZERO);
}

/// A sword character whose neutral special fires the blaster's shot, steered by `src`.
fn shooter(src: &str) -> Sim {
    let mut sim = with_special(Move {
        total_frames: 30,
        projectile: sim_blaster().projectile,
        projectile_script: Some(projectile_script(src)),
        ..Move::empty()
    });
    // The shot appears on the blaster's own frame, so make sure the move lasts until then.
    sim.content.weapons[0].moves[MoveId::NSpecial as usize].total_frames = 60;
    sim
}

fn sim_blaster() -> Move {
    Content::placeholder().weapons[1].moves[MoveId::NSpecial as usize].clone()
}

fn fire_and_follow(sim: &mut Sim, frames: usize) -> Vec<sim_core::Projectile> {
    start(sim);
    let mut shots = Vec::new();
    for _ in 0..frames {
        sim.tick(inp(0, 0, 0));
        if let Some(p) = sim.state.projectiles.iter().find(|p| p.active) {
            shots.push(*p);
        }
    }
    shots
}

#[test]
fn a_projectile_script_can_home_in_on_the_nearest_enemy() {
    // Fly flat, but drift toward the target's height.
    let mut sim = shooter("set_vel(pvx, sign(target_y + 3 - py) * 0.05);");
    sim.state.fighters[1].pos.y = Fx::from_int(8);
    sim.state.fighters[1].platform = sim_core::state::NONE;
    sim.state.fighters[1].state = S::Airborne;
    sim.state.fighters[1].invuln = 255;
    let shots = fire_and_follow(&mut sim, 40);
    assert!(shots.len() > 10, "the shot should fly for a while");
    let first = shots.first().unwrap();
    let last = shots.last().unwrap();
    assert!(
        last.pos.y > first.pos.y + Fx::from_int(1),
        "it climbed toward the target"
    );
}

#[test]
fn a_projectile_script_can_remove_the_projectile() {
    let mut sim = shooter("if age >= 6 { kill(); }");
    let shots = fire_and_follow(&mut sim, 40);
    assert!(!shots.is_empty());
    assert!(
        shots.iter().all(|p| p.age <= 6),
        "it never outlived the script's limit"
    );
    assert!(sim.state.projectiles.iter().all(|p| !p.active));
}

#[test]
fn projectile_variables_count_between_frames() {
    let mut sim = shooter("var n; n += 1;");
    let shots = fire_and_follow(&mut sim, 40);
    let last = shots.last().unwrap();
    assert_eq!(last.vars[0], Fx::from_int(i32::from(last.age)).raw());
}

#[test]
fn scripts_cannot_stall_the_frame() {
    // A runaway loop in both a move and a projectile finishes every frame and stays deterministic.
    let run = || {
        let mut sim = shooter("while true { set_vel(pvx, pvy); }");
        sim.content.weapons[0].moves[MoveId::NSpecial as usize].script =
            Some(fighter_script("while true { add_vel(0, 0); }"));
        for _ in 0..120 {
            sim.tick(inp(0, 0, SPECIAL));
        }
        sim.state.checksum()
    };
    assert_eq!(run(), run());
}

#[test]
fn different_scripts_change_the_content_hash() {
    let mut a = Content::placeholder();
    let b = a.clone();
    assert_eq!(a.hash(), b.hash());
    a.weapons[0].moves[MoveId::NSpecial as usize] = scripted(10, "end();");
    assert_ne!(a.hash(), b.hash());
    let h1 = a.hash();
    a.weapons[0].moves[MoveId::NSpecial as usize] = scripted(10, "turn();");
    assert_ne!(a.hash(), h1);
    // Comments and spacing do not.
    a.weapons[0].moves[MoveId::NSpecial as usize] = scripted(10, "end();");
    let h2 = a.hash();
    a.weapons[0].moves[MoveId::NSpecial as usize] = scripted(10, "// same\n  end( );");
    assert_eq!(a.hash(), h2);
}

fn scripted_content() -> Content {
    let mut c = Content::placeholder();
    let mut mv = scripted(
        40,
        "var n; n += 1; if frame >= 3 && frame <= 20 { set_vel(stick_x * 0.4, 0); } \
         if frame == 8 { spawn(1, 1.5, 0.5, 0); } if n == 25 { goto(0); }",
    );
    mv.projectile = sim_blaster().projectile;
    mv.projectile_script = Some(projectile_script(
        "var t; t += 1; if has_target { set_vel(pvx, sign(target_y - py) * 0.04); } if t > 40 { kill(); }",
    ));
    c.weapons[0].moves[MoveId::NSpecial as usize] = mv;
    c.weapons[1].moves[MoveId::SideSpecial as usize].script =
        Some(fighter_script("if frame == 5 { rehit(); }"));
    c
}

#[test]
fn scripted_content_is_deterministic_and_rolls_back() {
    let content = scripted_content();
    let inputs = random_inputs(&mut Rng::new(31), 900);
    let play = |frames: &[[sim_core::Input; MAX_FIGHTERS]]| {
        let mut s = GameState::new(&content, 4, [0, 1, 0, 1]);
        for i in frames {
            step(&mut s, &content, i);
        }
        s
    };
    assert_eq!(play(&inputs).checksum(), play(&inputs).checksum());

    // Snapshot in the middle, finish from the copy: identical to the straight run.
    let mut s = GameState::new(&content, 4, [0, 1, 0, 1]);
    for i in &inputs[..300] {
        step(&mut s, &content, i);
    }
    let saved = s;
    for i in &inputs[300..] {
        step(&mut s, &content, i);
    }
    let mut r = saved;
    for i in &inputs[300..] {
        step(&mut r, &content, i);
    }
    assert_eq!(s.checksum(), r.checksum());
    assert_eq!(s.checksum(), play(&inputs).checksum());
}

#[test]
fn scripts_actually_ran_in_the_fuzz() {
    // Guard against the test above passing because nothing used the scripts.
    let content = scripted_content();
    let (mut vars_changed, mut custom_shot) = (false, false);
    for seed in 31..51u64 {
        let inputs = random_inputs(&mut Rng::new(seed), 1500);
        let mut s = GameState::new(&content, 4, [0, 1, 0, 1]);
        for i in &inputs {
            step(&mut s, &content, i);
            vars_changed |= s.fighters.iter().any(|f| f.vars != [0; 4]);
            custom_shot |= s.projectiles.iter().any(|p| p.active && p.vars[0] != 0);
        }
    }
    assert!(vars_changed, "no move script ever ran");
    assert!(custom_shot, "no projectile script ever ran");
}

/// Scripts that fling the fighter around at random must not break the physics invariants.
#[test]
fn wild_scripts_cannot_push_fighters_into_the_stage() {
    let mut c = Content::placeholder();
    let wild = "set_vel(sin(frame * 37) * 3, cos(frame * 53) * 3); add_vel(stick_x, stick_y);                 if frame == 4 { spawn(0, 1, 1, -1); } if frame == 9 && hit { goto(8); }                 if frame == 12 { turn(); }";
    for weapon in 0..2 {
        for slot in [
            MoveId::NSpecial,
            MoveId::SideSpecial,
            MoveId::DownSpecial,
            MoveId::FSmash,
            MoveId::NAir,
        ] {
            let mv = &mut c.weapons[weapon].moves[slot as usize];
            if mv.is_empty() {
                mv.total_frames = 40;
            }
            if mv.hitboxes.is_empty() && mv.projectile.is_none() {
                mv.projectile =
                    Content::placeholder().weapons[1].moves[MoveId::NSpecial as usize].projectile;
            }
            mv.script = Some(fighter_script(wild));
        }
    }
    for seed in 0..30u64 {
        let inputs = random_inputs(&mut Rng::new(900 + seed), 1500);
        let mut s = GameState::new(&c, seed, [0, 1, 0, 1]);
        for (frame, i) in inputs.iter().enumerate() {
            step(&mut s, &c, i);
            for (n, f) in s.fighters.iter().enumerate() {
                for b in c.stage.platforms.iter().filter(|b| !b.pass_through) {
                    let inside = f.pos.x > b.left
                        && f.pos.x < b.right
                        && f.pos.y > b.bottom
                        && f.pos.y < b.y;
                    assert!(
                        !inside,
                        "seed {seed} frame {frame}: fighter {n} inside the stage at {:?} in {:?}",
                        f.pos, f.state
                    );
                }
            }
        }
    }
}

// ---- The sword character's scripted placeholder specials, as shipped ------------------------------------

#[test]
fn the_seeking_bolt_flies_forward_and_bends_toward_the_enemy() {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state = GameState::new_with_active(&sim.content, 1, [0, 0, 0, 0], 0b0011);
    sim.stand(0, Fx::from_int(-9), 1);
    sim.put_airborne(1, Fx::from_int(4), Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[1].invuln = 255;
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    sim.tick(inp(0, 0, SPECIAL));
    let mut path = Vec::new();
    for _ in 0..60 {
        sim.tick(inp(0, 0, 0));
        if let Some(p) = sim.state.projectiles.iter().find(|p| p.active) {
            path.push(p.pos);
        }
    }
    assert!(
        path.len() > 20,
        "the bolt should fly for a while, saw {} frames",
        path.len()
    );
    let (first, last) = (path[0], path[path.len() - 1]);
    assert!(last.x > first.x + Fx::from_int(4), "it flies forward");
    assert!(
        last.y > first.y + Fx::from_int(1),
        "it bends up toward the enemy"
    );
}

#[test]
fn the_lunge_moves_forward_and_hits() {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state = GameState::new_with_active(&sim.content, 1, [0, 0, 0, 0], 0b0011);
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(-2), -1);
    sim.state.fighters[2].invuln = 255;
    sim.state.fighters[3].invuln = 255;
    sim.tick(inp(0, 0, 0));
    let x0 = sim.f().pos.x;
    sim.tick(inp(127, 0, SPECIAL));
    let mut hit = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        hit |= sim.fighter(1).percent > Fx::ZERO;
    }
    assert!(hit, "the lunge should reach an enemy five units away");
    assert!(
        sim.f().pos.x > x0 + Fx::from_int(3),
        "and carry the fighter forward"
    );
}
