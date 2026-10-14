//! E9, a body's gauge and its powered mode (`sim_core::content::Gauge`, the champion's Boost gauge; owner decisions 9, 31, 32):
//! gain from damage dealt and taken (the damage actually added), nothing from shields; draining during a `gauge_drain` move and while
//! held out of one; the mode starting on a full gauge, its multipliers (and the jump speeds left alone), its 900-frame expiry; a KO
//! emptying it; a powered version spending it, and no spending without one; bodies without a gauge unchanged; the state in the
//! checksum and rolling back. Reference values: the seventh moveset's `REFERENCE_DATA.md` (`docs/movesets/greatsword/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::content::Gauge;
use sim_core::fuzz::random_inputs;
use sim_core::input::buttons::{ATTACK, SHIELD, SPECIAL};
use sim_core::moves::MoveId;
use sim_core::state::FighterState as S;
use sim_core::{step, FighterParams, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const CHAMPION: u8 = 6;

/// `a` (class `ca`) and `b` (class `cb`) as players 0 and 1, the others out of the match.
fn pair(ca: u8, cb: u8) -> Sim {
    let mut sim = Sim::with_chars([ca, cb, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn weapon_index(sim: &Sim, class: u8) -> usize {
    usize::from(sim.content.fighters[usize::from(class)].weapon)
}

/// Fighter `a` starts its forward tilt (standing, facing right) and fighter `b` is held in the air on the tilt's first box until it
/// is hit (or `b` stands shielding in front of it when `shield`). Returns what `b`'s percent rose by.
fn tilt(sim: &mut Sim, a: usize, b: usize, shield: bool) -> Fx {
    sim.stand(a, Fx::ZERO, 1);
    let class = sim.state.fighters[a].char_id;
    let w = weapon_index(sim, class);
    let hb = sim.content.weapons[w].moves[MoveId::FTilt as usize].hitboxes[0];
    sim_core::fighter::begin_attack(&mut sim.state.fighters[a], MoveId::FTilt);
    let before = sim.state.fighters[b].percent;
    for _ in 0..40 {
        let me = sim.state.fighters[a].pos;
        if shield {
            sim.stand(b, me.x + hb.x, -1);
            sim.state.fighters[b].state = S::Shield;
            sim.state.fighters[b].state_frame = 20;
        } else {
            let middle = sim.content.fighters[usize::from(sim.state.fighters[b].char_id)]
                .ecb_height
                * Fx::from_ratio(11, 20);
            let y = (me.y + hb.y - middle).max(fx(1, 10));
            sim.put_airborne(b, me.x + hb.x, y, Fx::ZERO, Fx::ZERO);
        }
        let (i0, i1) = if shield && b == 1 {
            (inp(0, 0, 0), inp(0, 0, SHIELD))
        } else if shield {
            (inp(0, 0, SHIELD), inp(0, 0, 0))
        } else {
            (inp(0, 0, 0), inp(0, 0, 0))
        };
        sim.tick2(i0, i1);
        if sim.state.fighters[b].percent > before
            || sim.state.fighters[a].hit_mask != 0
            || sim.state.fighters[a].state != S::Attack
        {
            break;
        }
    }
    sim.state.fighters[b].percent - before
}

#[test]
fn the_champion_gains_two_thirds_of_a_unit_a_percent_dealt_and_one_a_percent_taken() {
    // Dealt: the forward tilt's 11% (13.2% after the one-on-one 1.2: the damage actually added, owner decision 32) is 0.667 x 13.2.
    let mut sim = pair(CHAMPION, 0);
    let added = tilt(&mut sim, 0, 1, false);
    assert!(added > Fx::ZERO, "the tilt hit");
    let g = sim.content.fighters[usize::from(CHAMPION)].gauge;
    assert_eq!(sim.fighter(0).gauge, g.per_dealt * added);
    assert_eq!(sim.fighter(1).gauge, Fx::ZERO, "the duelist has no gauge");
    // Taken: one unit a percent.
    let mut sim = pair(0, CHAMPION);
    let added = tilt(&mut sim, 0, 1, false);
    assert!(added > Fx::ZERO);
    assert_eq!(sim.fighter(1).gauge, added);
    // The reference's rates: 150% dealt or 100% taken fills it.
    assert_eq!((g.max, g.per_taken), (Fx::from_int(100), Fx::ONE));
    assert_eq!(g.per_dealt, Fx::from_ratio(667, 1000));
}

#[test]
fn a_shielded_hit_fills_nobodys_gauge() {
    for (a, b) in [(CHAMPION, 0u8), (0, CHAMPION)] {
        let mut sim = pair(a, b);
        let added = tilt(&mut sim, 0, 1, true);
        assert_eq!(added, Fx::ZERO, "blocked");
        assert_eq!(sim.fighter(1).state, S::Shield);
        assert_eq!(
            (sim.fighter(0).gauge, sim.fighter(1).gauge),
            (Fx::ZERO, Fx::ZERO)
        );
    }
}

#[test]
fn a_pummel_fills_both_gauges_and_the_grab_parry_too() {
    let mut sim = pair(CHAMPION, CHAMPION);
    sim.stand(1, fx(12, 10), -1);
    sim.tick(inp(0, 0, sim_core::input::buttons::GRAB));
    for _ in 0..20 {
        if sim.f().state == S::Grabbing {
            break;
        }
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Grabbing);
    let before = sim.fighter(1).percent;
    sim.tick(inp(0, 0, ATTACK));
    sim.ticks(4, inp(0, 0, 0));
    let added = sim.fighter(1).percent - before;
    assert!(added > Fx::ZERO, "the pummel hit");
    assert_eq!(sim.fighter(1).gauge, added);
    assert_eq!(sim.fighter(0).gauge, Fx::from_ratio(667, 1000) * added);
}

#[test]
fn a_hit_during_a_draining_move_drains_by_the_damage_taken() {
    // A test body: the champion's forward tilt marked `gauge_drain` (the shipped charge has it); the duelist's tilt hits it.
    let mut sim = pair(0, CHAMPION);
    let w = weapon_index(&sim, CHAMPION);
    sim.content.weapons[w].moves[MoveId::FTilt as usize].gauge_drain = true;
    sim.state.fighters[1].gauge = Fx::from_int(50);
    // The victim mid-move: it is still in that move when the hit lands.
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::FTilt);
    sim.state.fighters[1].facing = -1;
    let mut added = Fx::ZERO;
    let dw = weapon_index(&sim, 0);
    let hb = sim.content.weapons[dw].moves[MoveId::FTilt as usize].hitboxes[0];
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FTilt);
    for _ in 0..30 {
        let me = sim.state.fighters[0].pos;
        let f = &mut sim.state.fighters[1];
        f.pos = Vec2::new(me.x + hb.x, Fx::ZERO);
        let before = f.percent;
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).percent > before {
            added = sim.fighter(1).percent - before;
            break;
        }
    }
    assert!(added > Fx::ZERO, "the tilt hit");
    assert_eq!(sim.fighter(1).gauge, Fx::from_int(50) - added);
}

#[test]
fn a_grab_out_of_a_draining_move_drains_on_every_pummel_until_the_hold_ends() {
    let mut sim = pair(0, CHAMPION);
    let w = weapon_index(&sim, CHAMPION);
    sim.content.weapons[w].moves[MoveId::FTilt as usize].gauge_drain = true;
    sim.content.weapons[w].moves[MoveId::FTilt as usize]
        .hitboxes
        .clear();
    sim.stand(1, fx(12, 10), -1);
    sim.state.fighters[1].gauge = Fx::from_int(50);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::FTilt);
    sim.tick(inp(0, 0, sim_core::input::buttons::GRAB));
    for _ in 0..20 {
        if sim.f().state == S::Grabbing {
            break;
        }
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().state, S::Grabbing, "caught in the draining move");
    assert!(sim.fighter(1).gauge_drain);
    let mut drained = Fx::ZERO;
    for _ in 0..2 {
        let before = sim.fighter(1).percent;
        sim.tick(inp(0, 0, ATTACK));
        sim.ticks(10, inp(0, 0, 0));
        drained += sim.fighter(1).percent - before;
    }
    assert!(drained > Fx::ZERO);
    assert_eq!(sim.fighter(1).gauge, Fx::from_int(50) - drained);
    // Thrown: the throw's damage drains too, then the hold is over and the next hit fills again.
    let before = sim.fighter(1).percent;
    sim.tick(inp(127, 0, 0));
    sim.ticks(40, inp(0, 0, 0));
    let thrown = sim.fighter(1).percent - before;
    assert!(thrown > Fx::ZERO);
    assert_eq!(sim.fighter(1).gauge, Fx::from_int(50) - drained - thrown);
    assert!(!sim.fighter(1).gauge_drain, "released");
}

#[test]
fn a_full_gauge_starts_the_powered_mode_which_lasts_900_frames() {
    let mut sim = pair(CHAMPION, 0);
    sim.stand(1, Fx::from_int(10), -1);
    sim.state.fighters[0].gauge = Fx::from_ratio(995, 10);
    // Taking 1% (a pummel-sized hit) fills it.
    let g = sim.content.fighters[usize::from(CHAMPION)].gauge;
    sim_core::gauge::add(&mut sim.state.fighters[0], &g, Fx::ONE);
    assert_eq!(
        (sim.f().gauge, sim.f().gauge_mode),
        (Fx::from_int(100), 900)
    );
    for t in 1..=900 {
        sim.tick(inp(0, 0, 0));
        if t < 900 {
            assert_eq!(sim.f().gauge, Fx::from_int(100), "frame {t}");
        }
    }
    assert_eq!(
        (sim.f().gauge, sim.f().gauge_mode),
        (Fx::ZERO, 0),
        "expired"
    );
}

#[test]
fn the_powered_mode_multiplies_the_bodys_movement_but_not_its_jumps() {
    let p = FighterParams::champion();
    let q = p.powered(true);
    for (name, a, b, want) in [
        ("walk", p.walk_speed, q.walk_speed, 1150),
        ("walk accel", p.walk_accel_stick, q.walk_accel_stick, 1150),
        ("dash", p.dash_speed, q.dash_speed, 1100),
        ("run", p.run_speed, q.run_speed, 1100),
        ("run accel", p.run_accel_stick, q.run_accel_stick, 1100),
        ("traction", p.ground_friction, q.ground_friction, 1150),
        ("air speed", p.air_speed, q.air_speed, 1200),
        ("air accel", p.air_accel_stick, q.air_accel_stick, 1200),
        ("air friction", p.air_friction, q.air_friction, 1150),
        ("gravity", p.gravity, q.gravity, 1100),
        ("fall", p.max_fall_speed, q.max_fall_speed, 1100),
        ("fast fall", p.fast_fall_speed, q.fast_fall_speed, 1100),
    ] {
        // Exactly the percent of the raw fixed-point value, rounded toward zero [D].
        assert_eq!(
            i64::from(b.raw()),
            i64::from(a.raw()) * want / 1000,
            "{name}"
        );
    }
    // The reference's full-gauge walk acceleration 0.181125 and air speed 1.386, to the last fixed-point step.
    assert!((q.walk_accel_stick - FighterParams::gu_fine(18112)).abs() <= Fx::from_raw(2));
    assert!((q.air_speed - FighterParams::su(1386)).abs() <= Fx::from_raw(1));
    for (a, b) in [
        (p.full_hop_velocity, q.full_hop_velocity),
        (p.short_hop_velocity, q.short_hop_velocity),
        (p.air_jump_velocity, q.air_jump_velocity),
        (p.hop_burst_velocity, q.hop_burst_velocity),
        (p.weight, q.weight),
        (p.ecb_height, q.ecb_height),
    ] {
        assert_eq!(a, b);
    }
    assert_eq!(p.powered(false), p);
    assert_eq!(
        FighterParams::tumbler().powered(true),
        FighterParams::tumbler()
    );
}

#[test]
fn in_the_powered_mode_the_champion_drifts_and_falls_faster_in_the_match() {
    let fall = |mode: u16| {
        let mut sim = pair(CHAMPION, 0);
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(60), Fx::ZERO, Fx::ZERO);
        sim.state.fighters[0].gauge_mode = mode;
        sim.ticks(80, inp(127, 0, 0));
        (sim.f().vel.x, -sim.f().vel.y)
    };
    let (vx, vy) = fall(0);
    let (px, py) = fall(900);
    let p = FighterParams::champion();
    assert_eq!((vx, vy), (p.air_speed, p.max_fall_speed));
    let q = p.powered(true);
    assert_eq!((px, py), (q.air_speed, q.max_fall_speed));
}

#[test]
fn a_ko_empties_the_gauge_and_ends_the_powered_mode() {
    // Owner decision 9: a partial gauge and a running mode both go on a stock loss.
    for (gauge, mode) in [(Fx::from_int(40), 0u16), (Fx::from_int(100), 500)] {
        let mut sim = pair(CHAMPION, 0);
        let f = &mut sim.state.fighters[0];
        f.gauge = gauge;
        f.gauge_mode = mode;
        let bottom = sim.content.stage.blast_bottom;
        sim.put_airborne(0, Fx::ZERO, bottom - Fx::ONE, Fx::ZERO, Fx::ZERO);
        let stocks = sim.f().stocks;
        sim.tick(inp(0, 0, 0));
        assert_eq!(sim.f().stocks, stocks - 1, "KO");
        assert_eq!((sim.f().gauge, sim.f().gauge_mode), (Fx::ZERO, 0));
    }
}

#[test]
fn a_special_with_a_powered_version_uses_it_and_spends_the_gauge_and_one_without_spends_nothing() {
    // The shipped Greatsword (phase 3): a special in the mode is its powered version and spends the gauge (the reference's rule).
    let mut sim = pair(CHAMPION, 0);
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 600;
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().state, S::Attack);
    assert_eq!(
        sim.f().move_id,
        sim_core::moves::greatsword_slots::POWER_WAVE as u8
    );
    assert_eq!((sim.f().gauge, sim.f().gauge_mode), (Fx::ZERO, 0), "spent");
    // A special without a powered version (the Greatsword's taken off) is the normal one and keeps the gauge.
    let mut sim = pair(CHAMPION, 0);
    let w = weapon_index(&sim, CHAMPION);
    sim.content.weapons[w].moves[MoveId::NSpecial as usize].limit_version = None;
    sim.state.fighters[0].gauge = Fx::from_int(100);
    sim.state.fighters[0].gauge_mode = 600;
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    assert_eq!(sim.f().gauge, Fx::from_int(100), "kept");
    assert!(sim.f().gauge_mode > 0);
    // A test weapon whose neutral special has a powered version (ext15, a copy of the forward tilt).
    let mut sim = pair(CHAMPION, 0);
    let w = weapon_index(&sim, CHAMPION);
    let tiltm = sim.content.weapons[w].moves[MoveId::FTilt as usize].clone();
    sim.content.weapons[w].moves[MoveId::Ext15 as usize] = tiltm;
    sim.content.weapons[w].moves[MoveId::NSpecial as usize].limit_version =
        Some(MoveId::Ext15 as u8);
    // Not powered: the normal move.
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().move_id, MoveId::NSpecial as u8);
    let mut sim2 = pair(CHAMPION, 0);
    sim2.content = sim.content.clone();
    sim2.state.fighters[0].gauge = Fx::from_int(100);
    sim2.state.fighters[0].gauge_mode = 600;
    sim2.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim2.f().move_id, MoveId::Ext15 as u8, "the powered version");
    assert_eq!(
        (sim2.f().gauge, sim2.f().gauge_mode),
        (Fx::ZERO, 0),
        "spent on the attempt"
    );
}

#[test]
fn bodies_without_a_gauge_have_none() {
    let c = sim_core::Content::placeholder();
    for (i, p) in c.fighters.iter().enumerate() {
        if i == usize::from(CHAMPION) {
            assert_eq!(p.gauge, FighterParams::champion_gauge());
        } else if i == 12 {
            // (Sim 121: the phantom's partner gauge.)
            assert_eq!(p.gauge, FighterParams::phantom_gauge());
        } else {
            assert_eq!(p.gauge, Gauge::NONE, "fighter {i}");
        }
    }
    // The name the game gives it lives in one place.
    assert_eq!(sim_core::content::GAUGE_NAME, "Boost");
}

#[test]
fn random_play_with_the_gauge_running_is_deterministic_and_rolls_back() {
    let content = {
        let mut c = sim_core::Content::placeholder();
        c.rules.stale_moves = 1;
        c
    };
    let chars = [CHAMPION, 0, CHAMPION, 3];
    for seed in 0..4u64 {
        let inputs = random_inputs(&mut Rng::new(seed), 700);
        let fresh = || {
            let mut s = GameState::new(&content, seed, chars);
            s.fighters[0].gauge = Fx::from_int(90);
            s.fighters[2].gauge = Fx::from_int(100);
            s.fighters[2].gauge_mode = 300;
            s
        };
        let mut s = fresh();
        let mut saved = s;
        for (t, i) in inputs.iter().enumerate() {
            if t == 350 {
                saved = s;
            }
            step(&mut s, &content, i);
        }
        let mut r = saved;
        for i in &inputs[350..] {
            step(&mut r, &content, i);
        }
        assert_eq!(r.checksum(), s.checksum(), "seed {seed}");
        let _ = MAX_FIGHTERS;
    }
}
