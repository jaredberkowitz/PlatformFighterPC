//! Scripts attached to moves and projectiles, run through the real simulation.

mod common;

use common::{fx, inp, Sim};
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::{JUMP, SPECIAL};
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
fn a_script_sees_the_jump_button() {
    // The move ends on the first frame jump is held (a charge cancelled by a jump).
    let mut sim = with_special(scripted(40, "if jump { end(); }"));
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Attack, "not without jump");
    sim.tick(inp(0, 0, JUMP));
    assert_ne!(sim.f().state, S::Attack, "jump ended it");
}

#[test]
fn a_stored_charge_outlasts_the_move_and_a_ko() {
    // Each use adds one to the stored charge and ends: the count carries from one use to the next (unlike `var`s), and a KO clears it.
    let mut sim = with_special(scripted(30, "if frame == 1 { store(stored + 1); end(); }"));
    for _ in 0..3 {
        sim.tick(inp(0, 0, SPECIAL));
        sim.ticks(10, inp(0, 0, 0));
    }
    assert_eq!(sim.f().stored, 3, "three uses");
    // The script reads it back: the move ends at once only while it is 3.
    let mut sim2 = with_special(scripted(30, "if stored == 3 { end(); }"));
    sim2.state.fighters[0].stored = 3;
    start(&mut sim2);
    sim2.tick(inp(0, 0, 0));
    assert_ne!(sim2.f().state, S::Attack, "read back as 3");
    // Out of the blast zone: back with nothing stored.
    sim.state.fighters[0].pos.y = Fx::from_int(-200);
    sim.ticks(5, inp(0, 0, 0));
    assert_eq!(sim.f().stored, 0, "a KO clears it");
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
fn two_spawns_on_one_frame_fire_two_shots_and_a_third_replaces_the_first() {
    // E3h (sim 95): the second `spawn` (or `spawn_at`) of a frame fills a second request instead of replacing the first.
    let fire = |src: &str| {
        let mut sim = with_special(scripted(30, src));
        sim.content.weapons[0].moves[MoveId::NSpecial as usize].projectile =
            sim.content.weapons[1].moves[MoveId::NSpecial as usize].projectile;
        start(&mut sim);
        let mut from = sim.f().pos;
        for _ in 0..12 {
            if sim.state.projectiles.iter().any(|p| p.active) {
                break;
            }
            from = sim.f().pos;
            sim.tick(inp(0, 0, 0));
        }
        let mut shots: Vec<_> = sim
            .state
            .projectiles
            .iter()
            .filter(|p| p.active)
            .map(|p| (p.pos - from, p.vel))
            .collect();
        shots.sort_by_key(|(_, v)| v.x);
        shots
    };
    let two = fire("if frame == 4 { spawn(1, 2, 0.4, 0.2); spawn(-1, 2, -0.4, 0.2); }");
    assert_eq!(two.len(), 2, "both on the same frame");
    assert_eq!(
        two[0],
        (
            sim_core::Vec2::new(Fx::from_int(-1), Fx::from_int(2)),
            sim_core::Vec2::new(fx(-4, 10), fx(2, 10))
        )
    );
    assert_eq!(
        two[1],
        (
            sim_core::Vec2::new(Fx::ONE, Fx::from_int(2)),
            sim_core::Vec2::new(fx(4, 10), fx(2, 10))
        )
    );
    let three = fire(
        "if frame == 4 { spawn(1, 2, 0.4, 0.2); spawn(-1, 2, -0.4, 0.2); spawn(0, 1, 0.1, 0); }",
    );
    assert_eq!(three.len(), 2);
    assert!(
        three[1].1.y == Fx::ZERO && three[1].1.x < fx(2, 10),
        "the third replaced the first"
    );
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
fn a_projectile_reads_its_owners_stick() {
    // Sim 117: `owner_stick_x` / `owner_stick_y` are the owner's stick this frame (absolute: x positive right), what the scamp's
    // rocket barrels read to be steered for a while. The shot keeps them in its variables: held right and down, 1 and -1.
    let mut sim = shooter("var x; var y; x = owner_stick_x; y = owner_stick_y;");
    start(&mut sim);
    let mut seen = None;
    for _ in 0..40 {
        sim.tick(inp(127, -127, 0));
        if let Some(p) = sim.state.projectiles.iter().find(|p| p.active) {
            seen = Some(p.vars);
        }
    }
    let v = seen.expect("a shot");
    assert_eq!((v[0], v[1]), (Fx::ONE.raw(), (-Fx::ONE).raw()));
    // Neutral: 0.
    let mut sim = shooter("var x; x = owner_stick_x + 5;");
    let shots = fire_and_follow(&mut sim, 40);
    assert_eq!(shots.last().unwrap().vars[0], Fx::from_int(5).raw());
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

#[test]
fn presses_and_releases_count_the_attack_button_since_the_attack_began_hitlag_included() {
    // The script keeps 10 x presses + releases where a test can read it.
    let mut sim = with_special(scripted(60, "store(presses * 10 + releases);"));
    start(&mut sim);
    // Started by special: nothing counted yet.
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().stored, 0);
    // Press, let go, press (held).
    sim.tick(inp(0, 0, sim_core::input::buttons::ATTACK));
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, sim_core::input::buttons::ATTACK));
    sim.tick(inp(0, 0, sim_core::input::buttons::ATTACK));
    assert_eq!(sim.f().stored, 21);
    // During hitlag the script does not run, but the counts go on: let go and press again while frozen.
    sim.state.fighters[0].hitlag = 4;
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, sim_core::input::buttons::ATTACK));
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().stored, 21, "frozen");
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().stored, 33, "the presses in hitlag counted");
    // A new attack starts counting again.
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().stored, 0);
}

#[test]
fn the_wall_register_says_the_last_step_was_stopped_by_a_wall() {
    // A slide to the left along the stage floor into nothing: wall stays 0. In the air beside the stage's side wall (x = -11),
    // moving into it: wall turns 1 (kept as the stored charge where the test can read it).
    let mut sim = with_special(scripted(
        40,
        "if frame >= 1 { set_vel(0.3, 0); } if wall { store(1); }",
    ));
    start(&mut sim);
    sim.ticks(5, inp(0, 0, 0));
    assert_eq!(sim.f().stored, 0, "nothing in the way");
    let mut sim = with_special(scripted(
        40,
        "if frame >= 1 { set_vel(0.3, -0.01); } if wall { store(1); }",
    ));
    let half = sim.content.fighters[0].ecb_half_width;
    sim.put_airborne(
        0,
        Fx::from_int(-11) - half - fx(5, 10),
        Fx::from_int(-3),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = 1;
    start(&mut sim);
    sim.ticks(6, inp(0, 0, 0));
    assert_eq!(sim.f().stored, 1, "ran into the wall");
}

#[test]
fn a_once_per_airtime_flag_lasts_until_the_fighter_lands() {
    // Each use in the air adds a lift only while flag 2 is clear, and sets it.
    let mut sim = with_special(scripted(
        12,
        "if frame == 1 && air_flag(2) == 0 { set_air_flag(2); add_vel(0, 0.5); }",
    ));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    let lift = |sim: &mut Sim| {
        sim.tick(inp(0, 0, SPECIAL));
        let before = sim.f().vel.y;
        sim.tick(inp(0, 0, 0));
        let after = sim.f().vel.y;
        sim.ticks(14, inp(0, 0, 0));
        after > before
    };
    assert!(lift(&mut sim), "the first use lifts");
    assert_eq!(sim.f().air_flags, 0b100);
    assert!(!lift(&mut sim), "the second in the same airtime does not");
    // Landing clears it.
    sim.stand(0, Fx::ZERO, 1);
    sim.tick(inp(0, 0, 0));
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].air_flags = 0b100;
    sim.state.fighters[0].pos.y = fx(1, 100);
    sim.state.fighters[0].vel.y = -Fx::ONE;
    sim.ticks(3, inp(0, 0, 0));
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_flags, 0, "landing clears the flags");
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
    assert!(lift(&mut sim), "so the next airtime lifts again");
}

#[test]
fn a_script_can_fire_a_bigger_harder_hitting_projectile() {
    // The same shot fired plain and scaled (30 more base knockback, 150% size): the scaled one carries both. (Sim 62.)
    let fire = |src: &str| {
        let mut sim = with_special(scripted(30, src));
        sim.content.weapons[0].moves[MoveId::NSpecial as usize].projectile =
            sim.content.weapons[1].moves[MoveId::NSpecial as usize].projectile;
        start(&mut sim);
        sim.ticks(4, inp(0, 0, 0));
        sim.state
            .projectiles
            .iter()
            .copied()
            .find(|p| p.active)
            .unwrap()
    };
    let plain = fire("if frame == 3 { spawn(1, 1, 0.4, 0); }");
    let scaled = fire("if frame == 3 { spawn_scaled(1, 1, 0.4, 0, 30, 150); }");
    assert_eq!((plain.kb_add, plain.size), (0, 100));
    assert_eq!((scaled.kb_add, scaled.size), (30, 150));
    assert_eq!(plain.pos, scaled.pos);
    assert_eq!(plain.vel, scaled.vel);
}

#[test]
fn a_script_can_fire_a_projectile_at_a_fixed_point_while_the_fighter_moves() {
    // Falling in the air: `spawn` is an offset from where the fighter ends the frame, `spawn_at` the absolute point given. (Sim 76.)
    let fire = |src: &str| {
        let mut sim = with_special(scripted(30, src));
        sim.content.weapons[0].moves[MoveId::NSpecial as usize].projectile =
            sim.content.weapons[1].moves[MoveId::NSpecial as usize].projectile;
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(10), Fx::ZERO, -Fx::ONE);
        start(&mut sim);
        sim.ticks(4, inp(0, 0, 0));
        sim.state
            .projectiles
            .iter()
            .copied()
            .find(|p| p.active)
            .unwrap()
    };
    let fixed = fire("if frame == 3 { spawn_at(2, 7, 0, 0); }");
    assert_eq!(
        fixed.pos,
        sim_core::Vec2::new(Fx::from_int(2), Fx::from_int(7))
    );
    let offset = fire("if frame == 3 { spawn(2, 7, 0, 0); }");
    assert_ne!(offset.pos.y, Fx::from_int(7));
}

/// Player 0 at -7 facing right uses a special that calls `lock(range)` on its frame 2 and keeps what it found in vars; returns the
/// sim after frame 3 of the move.
fn locking(range: i32, arrange: impl Fn(&mut Sim)) -> Sim {
    let src = format!("if frame == 2 {{ lock({range}); }}");
    let mut sim = with_special(scripted(30, &src));
    arrange(&mut sim);
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    sim
}

#[test]
fn lock_finds_the_nearest_opponent_in_front_and_keeps_its_chest() {
    // Player 1 at 7 (14 ahead), player 2 at -2 (5 ahead): the nearer one is locked, at its chest.
    let sim = locking(15, |sim| {
        sim.state.fighters[2].invuln = 0;
        sim.stand(2, Fx::from_int(-2), -1);
    });
    let f = sim.f();
    assert!(f.locked);
    let chest = sim_core::combat::chest_height(&sim.content.fighters[0]);
    assert_eq!(f.lock, sim_core::Vec2::new(Fx::from_int(-2), chest));
    // The lock stays where it was found when the target moves on, for the rest of the attack.
    let mut sim = sim;
    sim.stand(2, Fx::from_int(3), -1);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(sim.f().lock.x, Fx::from_int(-2));
    // A new attack starts without one.
    sim.ticks(30, inp(0, 0, 0));
    start(&mut sim);
    assert!(!sim.f().locked);
}

#[test]
fn lock_ignores_fighters_behind_out_of_range_intangible_or_behind_a_block() {
    // Behind: the only other fighter at -10 (player 0 faces right at -7).
    let sim = locking(15, |sim| sim.stand(1, Fx::from_int(-10), 1));
    assert!(!sim.f().locked, "behind");
    // Out of range: 14 ahead with a range of 10.
    let sim = locking(10, |_| {});
    assert!(!sim.f().locked, "out of range");
    assert!(locking(15, |_| {}).f().locked, "in range");
    // Intangible.
    let sim = locking(15, |sim| sim.state.fighters[1].invuln = 30);
    assert!(!sim.f().locked, "intangible");
    // A solid block across the line between the chests (a pass-through platform does not count).
    let sim = locking(15, |sim| {
        sim.content
            .stage
            .platforms
            .push(sim_core::content::Platform {
                left: Fx::ZERO,
                right: Fx::ONE,
                y: Fx::from_int(4),
                bottom: Fx::ZERO,
                pass_through: false,
            });
    });
    assert!(!sim.f().locked, "behind a block");
    let sim = locking(15, |sim| {
        sim.content
            .stage
            .platforms
            .push(sim_core::content::Platform {
                left: Fx::ZERO,
                right: Fx::ONE,
                y: Fx::from_int(4),
                bottom: Fx::ZERO,
                pass_through: true,
            });
    });
    assert!(sim.f().locked, "a pass-through platform does not block");
}

#[test]
fn segment_blocked_is_a_line_of_sight_through_solid_blocks_only() {
    use sim_core::collision::segment_blocked;
    use sim_core::Vec2;
    let stage = Content::placeholder().stage;
    let p = |x: i32, y: i32| Vec2::new(Fx::from_int(x), Fx::from_int(y));
    // Above the main stage: clear; through it: blocked; along its top surface: clear (touching is not crossing).
    assert!(!segment_blocked(&stage, p(-5, 1), p(5, 1)));
    assert!(segment_blocked(&stage, p(0, 2), p(0, -4)));
    assert!(!segment_blocked(&stage, p(-5, 0), p(5, 0)));
    // The pass-through platforms (y 3.6) never block.
    assert!(!segment_blocked(&stage, p(5, 6), p(5, 1)));
}

// ---- The gauge (E9) through scripts: `boost`, `boosting`, `grab`, `boost_add` ---------------------------------------------------

/// The champion (class 6, the body with a gauge) as player 0, with `mv` as its neutral special, standing alone.
fn champion_with_special(mv: Move) -> Sim {
    let mut sim = Sim::with_chars([6, 1, 0, 1]);
    let w = usize::from(sim.content.fighters[6].weapon);
    sim.content.weapons[w].moves[MoveId::NSpecial as usize] = mv;
    sim.stand(0, Fx::from_int(-7), 1);
    sim.stand(1, Fx::from_int(7), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

#[test]
fn boost_add_fills_the_gauge_and_starts_the_powered_mode_which_the_registers_read() {
    // 40 units a frame: 40, 80, then full (100) on the third frame, which starts the mode; after that it does nothing, even a drain.
    let src = "var seen; var mode; if boosting { boost_add(-50); } else { boost_add(40); } seen = boost; mode = boosting;";
    let mut sim = champion_with_special(scripted(30, src));
    start(&mut sim);
    // (The script first runs on the move's next frame.)
    sim.tick(inp(0, 0, 0));
    let one = Fx::ONE.raw();
    assert_eq!(sim.f().gauge, Fx::from_int(40));
    assert_eq!(sim.f().vars[0], 40 * one);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().gauge, Fx::from_int(80));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().gauge, Fx::from_int(100));
    assert_eq!(sim.f().gauge_mode, 900);
    assert_eq!(sim.f().vars[1], one, "boosting reads the mode");
    sim.tick(inp(0, 0, 0));
    assert_eq!(
        sim.f().gauge,
        Fx::from_int(100),
        "nothing drains while the mode runs"
    );
    assert_eq!(sim.f().vars[0], 100 * one);
}

#[test]
fn boost_add_drains_down_to_empty_and_does_nothing_for_a_body_without_a_gauge() {
    let mut sim = champion_with_special(scripted(30, "boost_add(-5);"));
    sim.state.fighters[0].gauge = Fx::from_int(7);
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().gauge, Fx::from_int(2));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().gauge, Fx::ZERO, "never below empty");
    // The duelist's body has no gauge: the register reads 0 and boost_add changes nothing.
    let mut sim = with_special(scripted(
        30,
        "var seen; boost_add(50); seen = boost + boosting;",
    ));
    start(&mut sim);
    sim.ticks(3, inp(0, 0, 0));
    assert_eq!(
        (sim.f().gauge, sim.f().gauge_mode, sim.f().vars[0]),
        (Fx::ZERO, 0, 0)
    );
}

#[test]
fn the_grab_register_reads_the_grab_button() {
    use sim_core::input::buttons::GRAB;
    let mut sim = with_special(scripted(30, "var g; g = grab;"));
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().vars[0], 0);
    sim.tick(inp(0, 0, GRAB));
    assert_eq!(sim.f().vars[0], Fx::ONE.raw());
}

#[test]
fn the_fast_fall_register_reads_a_fast_fall_during_the_move() {
    // An aerial-style special high up: falling with the stick snapped down fast falls, which the script sees (sim 91).
    let mut sim = with_special(scripted(60, "var ff; ff = fast_fall;"));
    sim.put_airborne(0, Fx::from_int(-7), Fx::from_int(30), Fx::ZERO, Fx::ZERO);
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().vars[0], 0, "not fast falling");
    for _ in 0..4 {
        sim.tick(inp(0, 0, 0));
    }
    sim.tick(inp(0, -127, 0));
    sim.tick(inp(0, -127, 0));
    assert!(sim.f().fast_fall, "the snap down fast falls");
    assert_eq!(sim.f().vars[0], Fx::ONE.raw());
}

// ---- `set_charge` and `charge_add` (E29, sim 90) ---------------------------------------------------------------------------------

#[test]
fn set_charge_sets_the_moves_charge_which_charge_add_turns_into_damage() {
    // A neutral special with a 10% hit on frame 5 and an additive charge of 18 over 110 frames: the script sets 55 frames of charge,
    // read back by `charge`, and the hit deals 19%.
    let mut mv = scripted(30, "var seen; set_charge(55); seen = charge;");
    mv.hitboxes = sim_core::moves::claws().moves[MoveId::Jab as usize]
        .hitboxes
        .clone();
    mv.hitboxes.truncate(1);
    for h in &mut mv.hitboxes {
        h.start = 5;
        h.end = 5;
        h.x = Fx::ONE;
        h.y = Fx::ONE;
        h.radius = Fx::from_int(2);
        h.damage = Fx::from_int(10);
    }
    mv.charge_add = Some(sim_core::moves::ChargeAdd {
        full: 110,
        add: Fx::from_int(18),
        floor: true,
    });
    let mut sim = with_special(mv);
    sim.stand(1, fx(-58, 10), -1);
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().charge, 55);
    assert_eq!(sim.f().vars[0], Fx::from_int(55).raw());
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(19) * sim.content.rules.damage_mult
    );
}

// ---- `aimed` (owner decision 52, sim 90) ---------------------------------------------------------------------------------------

#[test]
fn the_aimed_register_reads_how_the_attack_was_aimed_as_it_started() {
    // A forward tilt tilted up, straight or down: the script keeps what `aimed` read (1, 0, -1).
    for (stick_y, want) in [(50i8, 1), (0, 0), (-50, -1)] {
        let mut sim = Sim::with_chars([0, 1, 0, 1]);
        sim.content.weapons[0].moves[MoveId::FTilt as usize].script =
            Some(fighter_script("var a; a = aimed;"));
        sim.stand(0, Fx::from_int(-7), 1);
        sim.stand(1, Fx::from_int(7), -1);
        sim.tick(inp(60, stick_y, sim_core::input::buttons::ATTACK));
        assert_eq!(MoveId::from_index(sim.f().move_id), MoveId::FTilt);
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().vars[0], Fx::from_int(want).raw(), "stick {stick_y}");
    }
}

// ---- `owned` and the tool limit (E3d, sim 102) -------------------------------------------------------------------------------------

#[test]
fn the_owned_register_counts_the_fighters_shots_out_and_a_limit_stops_more() {
    // A neutral special firing the blaster's shot every frame from frame 2, its count read into a var: with a limit of 2 (tool kind
    // 4), two go out and no more, and `owned` sees both (as the frame began). Without a limit every frame fires one.
    for limit in [2u8, 0] {
        let mut mv = scripted(
            40,
            "var n; n = owned; if frame >= 2 { spawn(1, 1, 0.01, 0); }",
        );
        let mut spec = sim_blaster().projectile.expect("a shot");
        spec.life = 200;
        spec.tool.kind = 4;
        spec.tool.limit = limit;
        mv.projectile = Some(spec);
        let mut sim = with_special(mv);
        start(&mut sim);
        for _ in 0..6 {
            sim.tick(inp(0, 0, 0));
        }
        let out: Vec<_> = sim.state.projectiles.iter().filter(|p| p.active).collect();
        assert!(out.iter().all(|p| p.kind == 4));
        if limit == 2 {
            assert_eq!(out.len(), 2);
            assert_eq!(sim.f().vars[0], Fx::from_int(2).raw());
        } else {
            assert_eq!(out.len(), 5);
            assert_eq!(sim.f().vars[0], Fx::from_int(4).raw(), "as the frame began");
        }
    }
}

/// The wanderer's bomb item (E4), as a spec any weapon can carry in a test.
fn bomb_spec() -> sim_core::moves::ItemSpec {
    let c = Content::placeholder();
    c.weapons[usize::from(c.fighters[9].weapon)].items[0]
}

/// Item `spec` of fighter `maker`'s weapon, lying at `x` on the stage (E4). Returns its slot.
fn item_at(sim: &mut Sim, maker: usize, spec: u8, x: Fx) -> usize {
    let n = sim_core::items::make(
        &mut sim.state,
        &sim.content,
        maker,
        spec,
        sim_core::Vec2::new(x, Fx::ZERO),
        MoveId::NSpecial as u8,
    )
    .expect("a free slot");
    sim.state.items[n].platform = 0;
    n
}

#[test]
fn the_holding_register_reads_the_kind_of_the_item_in_hand() {
    // Sim 111 (E4): `holding` is the kind of the item in the fighter's hand as the frame began (0 empty-handed).
    for held in [false, true] {
        let mut sim = with_special(scripted(20, "var h; h = holding;"));
        let mut spec = bomb_spec();
        spec.kind = 7;
        sim.content.weapons[0].items = vec![spec];
        if held {
            let n = item_at(&mut sim, 0, 0, Fx::from_int(-7));
            sim.state.items[n].holder = 0;
            sim.state.items[n].rest = sim_core::state::ITEM_HELD;
            sim.state.fighters[0].holding = n as i8;
        }
        start(&mut sim);
        sim.tick(inp(0, 0, 0));
        let want = if held { 7 } else { 0 };
        assert_eq!(sim.f().vars[0], Fx::from_int(want).raw(), "held {held}");
    }
}

#[test]
fn detonate_bursts_the_fighters_items_after_their_delay_but_not_one_another_fighter_holds() {
    // Sim 111 (E4): `detonate()` on frame 1 sets every blast-capable item the fighter made bursting `delay` frames later (counting that
    // frame as the first), except one another fighter holds; the owner's own `owned` sees both.
    let mut sim = with_special(scripted(
        30,
        "var n; n = owned; if frame == 1 { detonate(); }",
    ));
    let mut spec = bomb_spec();
    spec.kind = 5;
    if let Some(b) = spec.blast.as_mut() {
        b.delay = 3;
    }
    sim.content.weapons[0].items = vec![spec];
    // The special pulls the item kind `owned` counts (its pull frame is never reached).
    sim.content.weapons[0].moves[MoveId::NSpecial as usize].pull = Some((29, 0));
    let free = item_at(&mut sim, 0, 0, Fx::from_int(-2));
    let taken = item_at(&mut sim, 0, 0, Fx::from_int(7));
    sim.state.items[taken].holder = 1;
    sim.state.items[taken].rest = sim_core::state::ITEM_HELD;
    sim.state.fighters[1].holding = taken as i8;
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().vars[0], Fx::from_int(2).raw(), "owned counts both");
    assert_eq!(sim.state.items[free].fuse, 2);
    sim.tick(inp(0, 0, 0));
    assert_ne!(sim.state.items[free].rest, sim_core::state::ITEM_BLAST);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.state.items[free].rest, sim_core::state::ITEM_BLAST);
    assert_eq!(sim.state.items[taken].rest, sim_core::state::ITEM_HELD);
    assert_eq!(sim.state.items[taken].fuse, 0);
}

#[test]
fn the_balloons_register_reads_the_balloons_carried_in_a_balloon_move() {
    // Sim 113 (E41): `balloons` is how many balloons the fighter carries in its balloon move: two from a full tank, one from a low
    // one, and none in a move without a `balloons` block.
    for (block, used, want) in [(true, 0u16, 2), (true, 200 * 16, 1), (false, 0, 0)] {
        let mut mv = scripted(20, "var b; b = balloons;");
        if block {
            mv.balloons = Some(sim_core::moves::pockets_slots::balloon_block());
        }
        let mut sim = with_special(mv);
        sim.state.fighters[0].helium_used = used;
        start(&mut sim);
        sim.tick(inp(0, 0, 0));
        assert_eq!(
            sim.f().vars[0],
            Fx::from_int(want).raw(),
            "block {block}, used {used}"
        );
    }
}

#[test]
fn the_pocketed_register_reads_what_the_pocket_holds() {
    // Sim 115 (E42): `pocketed` is 0 with an empty pocket, 1 with a projectile put away, 2 with an item.
    for (pocket, item, want) in [
        (sim_core::state::NONE, false, 0),
        (3, false, 1),
        (2, true, 2),
    ] {
        let mut sim = with_special(scripted(20, "var p; p = pocketed;"));
        sim.state.fighters[0].pocket = pocket;
        sim.state.fighters[0].pocket_item = item;
        start(&mut sim);
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().vars[0], Fx::from_int(want).raw(), "{pocket} {item}");
    }
}

#[test]
fn owns_counts_the_fighters_items_and_projectiles_of_a_kind() {
    // Sim 115 (E43): `owns(kind)` counts the live items and projectiles of that kind the fighter made, wherever they are (a pocketed
    // shot too); another fighter's do not count, nor kind 0.
    let mut sim = with_special(scripted(
        20,
        "var a; var b; var c; a = owns(7); b = owns(9); c = owns(0);",
    ));
    let mut spec = bomb_spec();
    spec.kind = 7;
    sim.content.weapons[0].items = vec![spec];
    item_at(&mut sim, 0, 0, Fx::from_int(-3));
    let theirs = item_at(&mut sim, 0, 0, Fx::from_int(-5));
    sim.state.items[theirs].origin = 1;
    for (n, kind) in [(0usize, 7u8), (1, 9), (2, 9)] {
        sim.state.projectiles[n] = sim_core::Projectile {
            active: true,
            kind,
            pos: sim_core::Vec2::new(Fx::from_int(-20), Fx::from_int(20)),
            life: 200,
            size: 100,
            rest: sim_core::state::REST_POCKETED,
            ..sim_core::Projectile::default()
        };
    }
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    let v = sim.f().vars;
    assert_eq!(
        (v[0], v[1], v[2]),
        (Fx::from_int(2).raw(), Fx::from_int(2).raw(), 0)
    );
}

#[test]
fn the_pull_wait_register_reads_the_wait_a_toss_started_counting_down() {
    // Sim 117 (E47): `pull_wait` is the frames left of the wait a move's toss started (`Toss::wait`), counting down a frame each frame
    // from the start of the next; 0 when there is none.
    let mut sim = with_special(scripted(20, "var w; w = pull_wait;"));
    sim.state.fighters[0].pull_wait = 10;
    // (The press's frame counts it down once, the next frame once more before the script reads it.)
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().vars[0], Fx::from_int(8).raw());
    let mut sim = with_special(scripted(20, "var w; w = pull_wait;"));
    start(&mut sim);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().vars[0], 0);
}
