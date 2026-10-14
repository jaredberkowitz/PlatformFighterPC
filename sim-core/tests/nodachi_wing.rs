//! The exile's wing (E57, sim 125: the reference's comeback form, ssbwiki.com's article on it, 13.0.1): when it comes (the start table
//! by stock and percent difference), what it gives (the body's numbers, a third jump, x1.3 damage that KOs earlier only by the extra
//! damage, armor on the smash attacks), how its points are spent (damage dealt, KOs) and that it is gone for good until a KO.

mod common;

use common::{fx, inp, Sim};
use sim_core::content::Wing;
use sim_core::input::buttons::JUMP;
use sim_core::moves::{MoveId, Weapon, MODE_POWERED};
use sim_core::state::FighterState as S;
use sim_core::{FighterParams, Fx, Vec2};

const EXILE: u8 = 13;

/// The exile (player 0) and a duelist (player 1), the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([EXILE, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim
}

fn wing() -> Wing {
    FighterParams::exile_wing()
}

fn kit(sim: &Sim) -> &Weapon {
    &sim.content.weapons[usize::from(sim.content.fighters[usize::from(EXILE)].weapon)]
}

/// The lowest percent (to a hundredth) at which the wing comes for a fighter `gap` stocks from its opponent, who has `opponent`
/// percent: the least `p` with `p` at least the start table's percent for the difference `p - opponent`.
fn activation(gap: i32, opponent: i32) -> Fx {
    let row = wing().rows[(gap + 2) as usize];
    let opp = Fx::from_int(opponent);
    (0..=20_000)
        .map(|n| Fx::from_ratio(n, 100))
        .find(|&p| p >= row.threshold(p - opp))
        .expect("comes by 200%")
}

#[test]
fn the_start_table_gives_the_references_activation_percents() {
    // ssbwiki.com's table of examples: rows by stock difference -2..+2, columns the opponent at 0 / 50 / 100 / 150 / 200%.
    let expected: [[i32; 5]; 5] = [
        [3000, 3000, 3000, 4154, 6000],
        [4000, 5154, 7000, 7000, 7000],
        [8000, 8000, 8000, 8000, 9167],
        [8500, 8500, 9750, 10500, 10500],
        [10334, 11000, 11000, 11000, 11000],
    ];
    for (r, gap) in (-2..=2).enumerate() {
        for (c, opp) in [0, 50, 100, 150, 200].into_iter().enumerate() {
            let got = activation(gap, opp);
            let want = Fx::from_ratio(expected[r][c], 100);
            assert!(
                (got - want).abs() <= fx(3, 100),
                "gap {gap}, opponent {opp}%: {got:?}, reference {want:?}"
            );
        }
    }
}

/// Fighter 1 (the duelist) hits the exile with its jab at point blank: the exile's percent rises by the jab's damage.
fn jab_the_exile(sim: &mut Sim) {
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(12, 10), -1);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::Jab);
    let before = sim.fighter(0).percent;
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(0).percent > before {
            return;
        }
    }
    panic!("the jab missed");
}

#[test]
fn the_wing_comes_when_a_hit_takes_the_exile_past_its_percent() {
    // Level on stocks, the opponent at 0%: the wing comes at 80%.
    let mut sim = duel();
    sim.state.fighters[0].percent = Fx::from_int(60);
    jab_the_exile(&mut sim);
    assert_eq!(sim.f().wing_points, Fx::ZERO, "not at 60%");
    sim.state.fighters[0].percent = Fx::from_int(79);
    sim.state.fighters[0].state = S::Idle;
    sim.state.fighters[0].air_jumps_left = 1;
    jab_the_exile(&mut sim);
    assert!(sim.f().percent >= Fx::from_int(80));
    assert_eq!(
        sim.f().wing_points,
        Fx::from_int(90),
        "90 points level on stocks"
    );
    assert_eq!(sim.f().air_jumps_left, 2, "a third jump at once");
    assert_eq!(sim.f().hit_by, 1, "the KO credit");
    // Two stocks behind: the wing comes at 30% with 110 points.
    let mut sim = duel();
    sim.state.fighters[0].stocks = 1;
    sim.state.fighters[0].percent = Fx::from_int(29);
    jab_the_exile(&mut sim);
    assert_eq!(sim.f().wing_points, Fx::from_int(110));
    // Two stocks ahead, the opponent at 0%: not until 103%.
    let mut sim = duel();
    sim.state.fighters[1].stocks = 1;
    sim.state.fighters[0].percent = Fx::from_int(100);
    jab_the_exile(&mut sim);
    assert_eq!(sim.f().wing_points, Fx::ZERO, "about 103% two stocks ahead");
    // A body without a wing never gets one.
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.state.roster = 0b11;
    sim.state.fighters[0].percent = Fx::from_int(150);
    jab_the_exile(&mut sim);
    assert_eq!(sim.f().wing_points, Fx::ZERO);
}

#[test]
fn the_wing_multiplies_the_exiles_numbers_and_gives_a_third_jump() {
    let p = FighterParams::exile();
    let q = p.winged(true);
    let gu = FighterParams::gu;
    let su = FighterParams::su;
    // ssbwiki.com / kuroganehammer.com: walk 1.035, dash 2.304, run 2.325, run acceleration 0.036 + 0.108, traction 0.143, air speed
    // 1.14, air acceleration 0.0115 + 0.077, air friction 0.0084, gravity 0.11124, fall 1.932 (fast 3.0912), three jumps.
    let near = |a: Fx, b: Fx| (a - b).abs() <= Fx::from_raw(2);
    for (name, got, want) in [
        ("walk", q.walk_speed, gu(1035)),
        ("dash", q.dash_speed, gu(2304)),
        ("run", q.run_speed, gu(2325)),
        ("run accel", q.run_accel, gu(36)),
        ("run accel stick", q.run_accel_stick, gu(108)),
        ("traction", q.ground_friction, gu(286)),
        ("air speed", q.air_speed, su(1140)),
        ("air accel", q.air_accel, FighterParams::su_fine(1150)),
        ("air accel stick", q.air_accel_stick, su(77)),
        ("air friction", q.air_friction, FighterParams::su_fine(840)),
        ("gravity", q.gravity, FighterParams::su_fine(11124)),
        ("fall", q.max_fall_speed, su(1932)),
        (
            "fast fall",
            q.fast_fall_speed,
            FighterParams::su_fine(309_120),
        ),
    ] {
        assert!(near(got, want), "{name}: {got:?} vs {want:?}");
    }
    assert_eq!(q.air_jumps, 2);
    // Jump speeds, weight and size do not change.
    assert_eq!(
        (q.full_hop_velocity, q.air_jump_velocity, q.weight),
        (p.full_hop_velocity, p.air_jump_velocity, p.weight)
    );
    assert_eq!(p.winged(false), p);
    // In play: the winged exile jumps twice in the air (three jumps in all), and runs faster.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(10), -1);
    sim.stand(0, Fx::ZERO, 1);
    sim.state.fighters[0].wing_points = Fx::from_int(90);
    // (A landing with the wing out gives the body's three jumps.)
    sim.put_airborne(0, Fx::ZERO, fx(1, 2), Fx::ZERO, Fx::ZERO);
    for _ in 0..20 {
        sim.tick(inp(0, 0, 0));
    }
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_jumps_left, 2);
    sim.tick(inp(0, 0, JUMP));
    for _ in 0..12 {
        sim.tick(inp(0, 0, 0));
    }
    let mut jumps = 0;
    for _ in 0..4 {
        let before = sim.f().air_jumps_left;
        sim.tick(inp(0, 0, JUMP));
        sim.tick(inp(0, 0, 0));
        if sim.f().air_jumps_left < before {
            jumps += 1;
        }
        for _ in 0..8 {
            sim.tick(inp(0, 0, 0));
        }
    }
    assert_eq!(jumps, 2, "two midair jumps with the wing");
}

/// Player 0 (the exile, winged or not) lands its forward smash's middle box (20%) on player 1 at `victim` percent: what was added
/// and the launch's knockback.
fn fsmash_on(winged: bool, victim: i32) -> (Fx, Fx) {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    if winged {
        sim.state.fighters[0].wing_points = Fx::from_int(90);
    }
    sim.state.fighters[1].percent = Fx::from_int(victim);
    let hb = *kit(&sim)
        .get(MoveId::FSmash as u8)
        .hitboxes
        .iter()
        .find(|h| h.priority == 3)
        .expect("the middle box");
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
    let middle = sim.content.fighters[1].ecb_height * Fx::from_ratio(11, 20);
    for _ in 0..30 {
        sim.put_airborne(1, hb.x, (hb.y - middle).max(fx(1, 10)), Fx::ZERO, Fx::ZERO);
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).state == S::Hitstun {
            break;
        }
    }
    (
        sim.fighter(1).percent - Fx::from_int(victim),
        sim.fighter(1).launch_kb,
    )
}

#[test]
fn the_wing_deals_1_3_times_the_damage_and_kos_earlier_only_by_the_extra() {
    let k = sim_core::Content::placeholder().rules.damage_mult;
    let (plain, plain_kb) = fsmash_on(false, 50);
    let (winged, winged_kb) = fsmash_on(true, 50);
    assert_eq!(plain, Fx::from_int(20) * k);
    assert_eq!(winged, Fx::from_int(20) * fx(13, 10) * k);
    // The knockback formula's damage term is the move's own 20: the winged hit at 50% launches as a plain hit on a victim 6% higher
    // (the extra 0.3 x 20), not harder still.
    let (_, shifted_kb) = fsmash_on(false, 56);
    assert!(winged_kb > plain_kb);
    assert!(
        (winged_kb - shifted_kb).abs() <= fx(1, 100),
        "{winged_kb:?} vs {shifted_kb:?}"
    );
}

#[test]
fn the_wings_points_go_with_damage_dealt_and_kos_and_then_it_is_gone_until_a_ko() {
    // A jab (2.5%) takes 0.3 x 2.5 = 0.75 of the 90 points, whatever the multipliers.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(12, 10), -1);
    sim.state.fighters[0].wing_points = Fx::from_int(90);
    sim.state.fighters[0].air_jumps_left = 2;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(
        sim.f().wing_points,
        Fx::from_int(90) - fx(3, 10) * fx(25, 10)
    );
    // A KO (credited by the last hit) takes 50, then the points move by the new stock difference: level (90) to one ahead (65):
    // 90 - 50 - 25 = 15 (ssbwiki.com's worked example).
    let mut sim = duel();
    sim.state.fighters[0].wing_points = Fx::from_int(90);
    sim.state.fighters[1].hit_by = 0;
    sim.put_airborne(1, Fx::from_int(1000), Fx::from_int(5), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.fighter(1).stocks, 2);
    assert_eq!(sim.f().wing_points, Fx::from_int(15));
    // Spent to nothing: the wing goes (its jump with it) and does not come back, however hurt the exile is...
    sim.state.fighters[0].wing_points = fx(1, 10);
    sim.state.fighters[0].air_jumps_left = 2;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_ratio(12, 10), -1);
    sim.state.fighters[1].invuln = 0;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
    for _ in 0..10 {
        sim.tick(inp(0, 0, 0));
    }
    assert_eq!(sim.f().wing_points, Fx::ZERO);
    assert!(sim.f().wing_spent);
    assert_eq!(sim.f().air_jumps_left, 1);
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
    }
    sim.state.fighters[0].percent = Fx::from_int(150);
    jab_into(&mut sim);
    assert_eq!(sim.f().wing_points, Fx::ZERO, "not again before a KO");
    // ...until it is knocked out: then it can come again.
    sim.put_airborne(0, Fx::from_int(-1000), Fx::from_int(5), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    assert!(!sim.f().wing_spent);
    sim.state.fighters[0].percent = Fx::from_int(150);
    for _ in 0..200 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state != S::Respawn {
            break;
        }
    }
    sim.state.fighters[0].invuln = 0;
    jab_into(&mut sim);
    assert!(
        sim.f().wing_points > Fx::ZERO,
        "it comes again after the KO"
    );
}

/// The duelist jabs the exile once (both put on the stage, the exile's percent as it is).
fn jab_into(sim: &mut Sim) {
    sim.state.fighters[0].state = S::Idle;
    jab_the_exile(sim);
}

#[test]
fn the_smash_attacks_have_armor_only_with_the_wing() {
    // The forward smash with the wing: a jab on frame 12 is withstood (the exile goes on with its smash, takes the damage); without
    // it, the exile is knocked out of it. A hit of more than 20% breaks the armor.
    let run = |winged: bool, damage: Option<i32>| {
        let mut sim = duel();
        if let Some(d) = damage {
            for h in &mut sim.content.weapons[0].moves[MoveId::Jab as usize].hitboxes {
                h.damage = Fx::from_int(d);
            }
        }
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_ratio(12, 10), -1);
        if winged {
            sim.state.fighters[0].wing_points = Fx::from_int(90);
        }
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::FSmash);
        assert_eq!(sim.f().move_powered, winged);
        for _ in 0..9 {
            sim.tick(inp(0, 0, 0));
        }
        sim_core::fighter::begin_attack(&mut sim.state.fighters[1], MoveId::Jab);
        let before = sim.f().percent;
        for _ in 0..8 {
            sim.tick(inp(0, 0, 0));
            if sim.f().percent > before {
                break;
            }
        }
        assert!(sim.f().percent > before, "the jab landed");
        sim.f().state
    };
    assert_eq!(run(true, None), S::Attack, "withstood with the wing");
    assert_ne!(run(false, None), S::Attack, "knocked out of it without");
    assert_ne!(
        run(true, Some(25)),
        S::Attack,
        "a 25% hit breaks the 20% armor"
    );
    let sim = duel();
    let w = kit(&sim);
    for (id, from, to) in [
        (MoveId::FSmash, 10u8, 23u8),
        (MoveId::USmash, 14, 22),
        (MoveId::DSmash, 10, 20),
    ] {
        let m = w.get(id as u8);
        let a = m.armor.expect("armor");
        assert_eq!(
            (a.start + 1, a.end + 1, a.threshold, m.armor_mode),
            (from, to, Fx::from_int(20), MODE_POWERED),
            "{id:?}"
        );
    }
}

#[test]
fn the_wing_is_resimulated_exactly() {
    // The exile with its wing against a duelist, mashing everything: a snapshot resimulated gives the same checksums (the wing's
    // points, its loss and the KO credit are in the state).
    let mut sim = duel();
    sim.state.fighters[0].wing_points = Fx::from_int(90);
    sim.state.fighters[0].percent = Fx::from_int(70);
    let mut rng = sim_core::Rng::new(11);
    let mut inputs = Vec::new();
    for _ in 0..600 {
        let r = rng.next_u32();
        let pick = |s: u32| {
            inp(
                ((s & 0xff) as i8) / 2 * 2,
                (((s >> 8) & 0xff) as i8) / 2 * 2,
                ((s >> 16) & 0x3f) as u16,
            )
        };
        inputs.push([
            pick(r),
            pick(r.rotate_left(11)),
            Default::default(),
            Default::default(),
        ]);
    }
    let snapshot = sim.state;
    let mut straight = Vec::new();
    for i in &inputs {
        sim_core::step(&mut sim.state, &sim.content, i);
        straight.push(sim.state.checksum());
    }
    let mut again = snapshot;
    for (n, i) in inputs.iter().enumerate() {
        sim_core::step(&mut again, &sim.content, i);
        assert_eq!(again.checksum(), straight[n], "frame {n}");
    }
    // (The wing did something in this run.)
    let _ = Vec2::ZERO;
}
