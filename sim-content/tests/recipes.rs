//! Character recipes: every combination of stats is valid content, and the stats change how a fighter plays in the real
//! simulation, in the directions the creator promises (bigger is heavier, slower, and jumps lower).

use sim_content::recipe::{readout, Recipe, MAX_STAT, MIN_STAT};
use sim_content::validate;
use sim_core::input::buttons::JUMP;
use sim_core::{step, Content, Fx, GameState, Input, MAX_FIGHTERS};

/// Every recipe there is, as content (in batches: a roster holds at most 64 fighters).
#[test]
fn every_recipe_is_valid_content() {
    let mut all = Vec::new();
    for class in 0..2u8 {
        for size in MIN_STAT..=MAX_STAT {
            for speed in MIN_STAT..=MAX_STAT {
                for jump in MIN_STAT..=MAX_STAT {
                    for weight in MIN_STAT..=MAX_STAT {
                        all.push(Recipe {
                            class,
                            size,
                            speed,
                            jump,
                            weight,
                        });
                    }
                }
            }
        }
    }
    assert_eq!(all.len(), 2 * 9 * 9 * 9 * 9);
    for chunk in all.chunks(60) {
        let mut c = Content::placeholder();
        c.fighters.clear();
        c.names.fighters.clear();
        for (i, r) in chunk.iter().enumerate() {
            c.fighters.push(r.params());
            c.names.fighters.push(format!("f{i}"));
        }
        assert_eq!(
            validate(&c),
            Ok(()),
            "first recipe of the batch: {:?}",
            chunk[0]
        );
    }
}

fn with(r: Recipe) -> Recipe {
    r
}

fn content_with(recipes: &[Recipe]) -> Content {
    let mut c = Content::placeholder();
    c.stage.blast_top = Fx::from_int(5000);
    for (i, r) in recipes.iter().enumerate() {
        c.fighters.push(r.params());
        c.names.fighters.push(format!("made{i}"));
    }
    c
}

fn inputs(p0: Input) -> [Input; MAX_FIGHTERS] {
    let mut i = [Input::default(); MAX_FIGHTERS];
    i[0] = p0;
    i
}

/// Distance run in 50 frames of holding right, from a standstill on the main stage.
fn run_distance(r: Recipe) -> Fx {
    let c = content_with(&[r]);
    let mut s = GameState::new_with_active(&c, 1, [2, 1, 0, 1], 0b0011);
    s.fighters[0].pos.x = Fx::from_int(-9);
    let start = s.fighters[0].pos.x;
    for _ in 0..50 {
        step(
            &mut s,
            &c,
            &inputs(Input {
                stick_x: 127,
                stick_y: 0,
                buttons: 0,
            }),
        );
    }
    s.fighters[0].pos.x - start
}

/// Peak height of a full hop (jump held).
fn jump_peak(r: Recipe) -> Fx {
    let c = content_with(&[r]);
    let mut s = GameState::new_with_active(&c, 1, [2, 1, 0, 1], 0b0011);
    let ground = s.fighters[0].pos.y;
    let mut peak = Fx::ZERO;
    for t in 0..80 {
        let held = if t < 20 { JUMP } else { 0 };
        step(
            &mut s,
            &c,
            &inputs(Input {
                stick_x: 0,
                stick_y: 0,
                buttons: held,
            }),
        );
        peak = peak.max(s.fighters[0].pos.y - ground);
    }
    peak
}

fn sized(size: u8) -> Recipe {
    Recipe {
        size,
        ..Recipe::default()
    }
}

#[test]
fn bigger_fighters_run_slower_and_jump_lower() {
    let runs: Vec<Fx> = [1, 3, 5, 7, 9]
        .iter()
        .map(|s| run_distance(sized(*s)))
        .collect();
    let jumps: Vec<Fx> = [1, 3, 5, 7, 9]
        .iter()
        .map(|s| jump_peak(sized(*s)))
        .collect();
    for w in runs.windows(2) {
        assert!(w[0] > w[1], "a bigger fighter runs less far: {runs:?}");
    }
    for w in jumps.windows(2) {
        assert!(w[0] > w[1], "a bigger fighter jumps lower: {jumps:?}");
    }
    // The effect is real, not a rounding wobble.
    assert!(runs[0] > runs[4] + Fx::from_int(1), "{runs:?}");
    assert!(jumps[0] > jumps[4] + Fx::HALF, "{jumps:?}");
}

#[test]
fn bigger_fighters_are_taller_wider_heavier_and_fall_faster() {
    let small = readout(&sized(1).params());
    let big = readout(&sized(9).params());
    assert!(big.height > small.height);
    assert!(big.weight > small.weight);
    assert!(big.fall_speed > small.fall_speed);
    assert!(sized(9).params().ecb_half_width > sized(1).params().ecb_half_width);
}

#[test]
fn the_speed_stat_makes_a_fighter_quicker_and_lighter() {
    let slow = Recipe {
        speed: 1,
        ..Recipe::default()
    };
    let fast = Recipe {
        speed: 9,
        ..Recipe::default()
    };
    assert!(run_distance(fast) > run_distance(slow) + Fx::from_int(1));
    assert!(fast.params().weight < slow.params().weight);
}

#[test]
fn the_jump_stat_raises_jumps_without_changing_how_fast_a_fighter_runs() {
    let low = Recipe {
        jump: 1,
        ..Recipe::default()
    };
    let high = Recipe {
        jump: 9,
        ..Recipe::default()
    };
    assert!(jump_peak(high) > jump_peak(low) + Fx::from_int(1));
    assert_eq!(run_distance(high), run_distance(low));
}

#[test]
fn the_weight_stat_makes_a_fighter_harder_to_launch() {
    let light = Recipe {
        weight: 1,
        ..Recipe::default()
    };
    let heavy = Recipe {
        weight: 9,
        ..Recipe::default()
    };
    assert!(heavy.params().weight > light.params().weight);
    // The same hit sends the lighter fighter further.
    let kb = |r: Recipe| {
        sim_core::combat::knockback(
            Fx::from_int(60),
            Fx::from_int(10),
            r.params().weight,
            40,
            100,
        )
    };
    assert!(kb(light) > kb(heavy));
}

#[test]
fn the_neutral_recipe_plays_exactly_like_the_archetype() {
    let c = content_with(&[with(Recipe::default())]);
    let reference = Content::placeholder();
    let mut a = GameState::new_with_active(&c, 1, [2, 1, 0, 1], 0b0011);
    let mut b = GameState::new_with_active(&reference, 1, [0, 1, 0, 1], 0b0011);
    for t in 0..300 {
        let i = Input {
            stick_x: if t % 90 < 45 { 127 } else { -127 },
            stick_y: 0,
            buttons: if t % 70 == 5 { JUMP } else { 0 },
        };
        step(&mut a, &c, &inputs(i));
        step(&mut b, &reference, &inputs(i));
    }
    assert_eq!(a.fighters[0].pos, b.fighters[0].pos);
}

#[test]
fn a_recipe_section_round_trips_through_the_content_format() {
    let r = Recipe {
        class: 1,
        size: 8,
        speed: 3,
        jump: 7,
        weight: 6,
    };
    let mut doc = sim_content::doc::Doc::from_content(&Content::placeholder(), "t", "", "");
    doc.put(r.section("maker", "claws"));
    let built = doc.build().unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(built.content.fighters[2], r.params());
    assert_eq!(built.content.names.fighters[2], "maker");
}

#[test]
fn creating_a_character_never_changes_the_other_fighters() {
    let mut doc = sim_content::doc::Doc::from_content(&Content::placeholder(), "t", "", "");
    let before = doc.build().unwrap().content;
    doc.put(Recipe::default().section("maker", "longsword"));
    let after = doc.build().unwrap().content;
    assert_eq!(&after.fighters[..2], &before.fighters[..]);
    assert_eq!(after.weapons, before.weapons);
}

// ---- Hitbox size scaling -------------------------------------------------------------------------------------

/// The furthest gap (in tenths of a unit) at which player 0's jab, from fighter `index`, still hits a standing target.
fn jab_reach(r: Recipe) -> i32 {
    use sim_core::input::buttons::ATTACK;
    let c = content_with(&[r]);
    let mut best = 0;
    for gap in (10..90).step_by(2) {
        let mut s = GameState::new_with_active(&c, 1, [2, 0, 0, 0], 0b0011);
        s.fighters[0].pos.x = Fx::from_int(-9);
        s.fighters[1].pos.x = Fx::from_int(-9) + Fx::from_ratio(gap, 10);
        s.fighters[1].facing = -1;
        s.fighters[1].invuln = 0;
        for t in 0..14 {
            let held = if t == 0 { ATTACK } else { 0 };
            step(
                &mut s,
                &c,
                &inputs(Input {
                    stick_x: 0,
                    stick_y: 0,
                    buttons: held,
                }),
            );
        }
        if s.fighters[1].percent > Fx::ZERO {
            best = gap;
        }
    }
    best
}

#[test]
fn bigger_fighters_attacks_reach_further_and_smaller_ones_less() {
    let small = jab_reach(sized(1));
    let normal = jab_reach(sized(5));
    let big = jab_reach(sized(9));
    assert!(
        small < normal && normal < big,
        "reach {small} < {normal} < {big}"
    );
}

#[test]
fn hitbox_scale_follows_size_and_is_exactly_one_at_the_neutral_size() {
    assert_eq!(sized(5).params().hitbox_scale, Fx::ONE);
    assert!(sized(9).params().hitbox_scale > Fx::from_ratio(125, 100));
    assert!(sized(1).params().hitbox_scale < Fx::from_ratio(75, 100));
}

#[test]
fn active_hitboxes_are_scaled_in_size_and_position() {
    use sim_core::combat::active_hitboxes;
    let c = Content::placeholder();
    let mv = &c.weapons[0].moves[sim_core::moves::MoveId::FTilt as usize];
    let mut f = GameState::new(&c, 1, [0, 0, 0, 0]).fighters[0];
    f.state_frame = u16::from(mv.hitboxes[0].start);
    let one: Vec<_> = active_hitboxes(&f, mv, Fx::ONE).collect();
    let two: Vec<_> = active_hitboxes(&f, mv, Fx::from_int(2)).collect();
    assert_eq!(one.len(), two.len());
    for (a, b) in one.iter().zip(&two) {
        assert_eq!(b.1.radius, a.1.radius * Fx::from_int(2));
        assert_eq!(b.2.x - f.pos.x, (a.2.x - f.pos.x) * Fx::from_int(2));
        assert_eq!(b.2.y - f.pos.y, (a.2.y - f.pos.y) * Fx::from_int(2));
    }
}

#[test]
fn a_big_fighters_projectile_leaves_from_further_out() {
    use sim_core::input::buttons::SPECIAL;
    let muzzle = |size: u8| {
        let r = Recipe {
            class: 1,
            size,
            ..Recipe::default()
        };
        let c = content_with(&[r]);
        let mut s = GameState::new_with_active(&c, 1, [2, 0, 0, 0], 0b0011);
        s.fighters[0].pos.x = Fx::from_int(-9);
        for t in 0..40 {
            let held = if t == 0 { SPECIAL } else { 0 };
            step(
                &mut s,
                &c,
                &inputs(Input {
                    stick_x: 0,
                    stick_y: 0,
                    buttons: held,
                }),
            );
            if let Some(p) = s.projectiles.iter().find(|p| p.active) {
                return p.pos.x - s.fighters[0].pos.x;
            }
        }
        panic!("no shot");
    };
    assert!(muzzle(9) > muzzle(5) && muzzle(5) > muzzle(1));
}
