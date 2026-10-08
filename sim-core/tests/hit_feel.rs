//! Hit feel in the style of the reference game (sim v27): hitlag `floor(d * 0.65 + 6)`, hitstun `floor(KB * 0.4) - 1`, paced and
//! shorter survival DI, shield SDI, perfect-shield hitlag, crouch cancelling, rage, stale-move negation and hitstun cancelling.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::hitlag_frames;
use sim_core::input::buttons::{ATTACK, SHIELD};
use sim_core::state::FighterState as S;
use sim_core::{Fx, Vec2};

/// Player 0 jabs player 1 standing 1.4 away; only the two of them are in play. Returns the sim on the tick the jab connects.
fn jab_hit(sim: &mut Sim, p1: sim_core::Input) {
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    let before = sim.fighter(1).percent;
    sim.tick2(inp(0, 0, ATTACK), p1);
    for _ in 0..20 {
        if sim.fighter(1).percent > before {
            return;
        }
        sim.tick2(inp(0, 0, 0), p1);
    }
    panic!("the jab never connected");
}

fn duel() -> Sim {
    let mut sim = Sim::duel(fx(14, 10));
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim
}

#[test]
fn hitlag_is_the_reference_formula_for_both_fighters() {
    let mut sim = duel();
    jab_hit(&mut sim, inp(0, 0, 0));
    let dealt = sim.fighter(1).percent / sim.content.rules.damage_mult;
    let lag = hitlag_frames(dealt, &sim.content.rules, false, false, Fx::ONE);
    assert_eq!(
        lag,
        (dealt * fx(65, 100) + Fx::from_int(6)).floor_int() as u8
    );
    assert_eq!(sim.f().hitlag, lag);
    assert_eq!(sim.fighter(1).hitlag, lag);
}

#[test]
fn crouching_takes_less_knockback_and_less_hitlag() {
    let mut stand = duel();
    jab_hit(&mut stand, inp(0, 0, 0));
    let mut crouch = duel();
    // Player 1 crouches first.
    crouch.state.fighters[2].active = false;
    for _ in 0..4 {
        crouch.tick2(inp(0, 0, 0), inp(0, -127, 0));
    }
    assert_eq!(crouch.fighter(1).state, S::Crouch);
    jab_hit(&mut crouch, inp(0, -127, 0));
    let (a, b) = (stand.fighter(1), crouch.fighter(1));
    assert!(
        b.launch_kb < a.launch_kb,
        "{:?} vs {:?}",
        b.launch_kb,
        a.launch_kb
    );
    let ratio = b.launch_kb / a.launch_kb;
    assert!(
        (ratio - fx(85, 100)).abs() < fx(1, 100),
        "0.85 of it: {ratio:?}"
    );
    assert!(
        b.hitlag < a.hitlag,
        "and less hitlag: {} vs {}",
        b.hitlag,
        a.hitlag
    );
    assert!(crouch.f().hitlag < stand.f().hitlag, "for the attacker too");
}

#[test]
fn rage_makes_a_hurt_attacker_hit_harder() {
    let mut calm = duel();
    jab_hit(&mut calm, inp(0, 0, 0));
    let mut angry = duel();
    angry.state.fighters[0].percent = Fx::from_int(150);
    jab_hit(&mut angry, inp(0, 0, 0));
    let ratio = angry.fighter(1).launch_kb / calm.fighter(1).launch_kb;
    assert!(
        (ratio - fx(11, 10)).abs() < fx(1, 100),
        "a tenth more at 150%: {ratio:?}"
    );
    assert_eq!(
        angry.fighter(1).percent,
        calm.fighter(1).percent,
        "rage changes knockback, not damage"
    );
}

#[test]
fn a_move_used_again_and_again_deals_less_and_a_fresh_one_a_little_more() {
    let mut sim = duel();
    sim.content.rules.stale_moves = 1;
    let mut dealt = Vec::new();
    for _ in 0..4 {
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, fx(14, 10), -1);
        sim.state.fighters[1].percent = Fx::ZERO;
        jab_hit(&mut sim, inp(0, 0, 0));
        dealt.push(sim.fighter(1).percent);
        sim.ticks(80, inp(0, 0, 0));
    }
    let base = Fx::from_int(3) * sim.content.rules.damage_mult;
    let close = |a: Fx, b: Fx| (a - b).abs() < fx(1, 100);
    assert!(
        close(dealt[0], base * fx(105, 100)),
        "fresh: {:?}",
        dealt[0]
    );
    assert!(
        close(dealt[1], base * fx(91, 100)),
        "once stale: {:?}",
        dealt[1]
    );
    assert!(
        dealt[2] < dealt[1] && dealt[3] < dealt[2],
        "keeps weakening: {dealt:?}"
    );
    // A KO clears the queue.
    let respawned = {
        let mut f = sim.state.fighters[0];
        sim_core::combat::respawn(&mut f, &sim.content, 0, true);
        f
    };
    assert_eq!(respawned.stale, [0; 9]);
}

#[test]
fn survival_di_is_short_and_paced() {
    let mut sim = duel();
    // A long hitlag to flick in.
    sim.content.rules.hitlag_base = Fx::from_int(22);
    jab_hit(&mut sim, inp(0, 0, 0));
    let lag = usize::from(sim.fighter(1).hitlag);
    assert!(lag >= 20);
    let start = sim.fighter(1).pos.x;
    // Flick right on every other frame: more flicks than the pacing allows.
    let mut flicks = 0;
    for t in 0..lag - 1 {
        let x = if t % 2 == 0 { 127 } else { 0 };
        flicks += usize::from(x != 0);
        sim.tick2(inp(0, 0, 0), inp(x, 0, 0));
    }
    let moved = sim.fighter(1).pos.x - start;
    let step = sim.content.rules.sdi_distance;
    let nudges = (moved / step).floor_int() as usize;
    let most = (lag - 1).div_ceil(usize::from(sim.content.rules.sdi_interval));
    assert!(
        nudges >= 2 && nudges <= most,
        "{nudges} nudges from {flicks} flicks over {lag} frames"
    );
    assert!(nudges < flicks);
    assert_eq!(step, fx(1, 4), "two reference units");
}

#[test]
fn a_shielding_fighter_can_nudge_sideways_but_not_up() {
    let mut sim = duel();
    sim.content.rules.hitlag_base = Fx::from_int(22);
    for _ in 0..10 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    assert_eq!(sim.fighter(1).state, S::Shield);
    let before = sim.fighter(1).pos;
    sim.tick2(inp(0, 0, ATTACK), inp(0, 0, SHIELD));
    for _ in 0..20 {
        if sim.fighter(1).hitlag > 0 {
            break;
        }
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
    }
    assert!(sim.fighter(1).hitlag > 0, "the shield was hit");
    let x0 = sim.fighter(1).pos.x;
    sim.tick2(inp(0, 0, 0), inp(127, 127, SHIELD));
    let nudge = sim.fighter(1).pos.x - x0;
    assert!(nudge > Fx::ZERO, "sideways");
    assert!(nudge <= sim.content.rules.sdi_distance * fx(2, 3) + fx(1, 1000));
    assert_eq!(sim.fighter(1).pos.y, before.y, "never up");
}

#[test]
fn a_perfect_shield_freezes_only_the_attacker() {
    let mut sim = duel();
    // Raise the shield so that the jab lands inside the perfect-shield window.
    sim.tick2(inp(0, 0, ATTACK), inp(0, 0, 0));
    sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    for _ in 0..12 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
        if sim.f().hitlag > 0 {
            break;
        }
    }
    assert!(sim.f().hitlag > 0, "the attacker is frozen");
    assert_eq!(sim.fighter(1).state, S::Shield);
    assert_eq!(sim.fighter(1).hitlag, 0, "the perfect shielder is not");
}

#[test]
fn a_long_hitstun_can_be_air_dodged_out_of_after_forty_frames() {
    let mut sim = duel();
    let rules = sim.content.rules;
    let launch = |sim: &mut Sim| {
        sim.put_airborne(1, Fx::ZERO, Fx::from_int(40), Fx::ZERO, Fx::ZERO);
        let f = &mut sim.state.fighters[1];
        f.state = S::Hitstun;
        f.state_frame = 0;
        f.hitstun = 90;
        f.kb_vel = Vec2::new(fx(1, 2), fx(1, 2));
        f.tumble = true;
    };
    launch(&mut sim);
    // Pressing shield early does nothing.
    for t in 0..u16::from(rules.hitstun_dodge_cancel) - 1 {
        let b = if t % 2 == 0 { SHIELD } else { 0 };
        sim.tick2(inp(0, 0, 0), inp(0, 0, b));
        assert_eq!(sim.fighter(1).state, S::Hitstun, "frame {t}");
    }
    // From frame 40 it is an air dodge.
    for _ in 0..4 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
        if sim.fighter(1).state != S::Hitstun {
            break;
        }
    }
    assert_eq!(sim.fighter(1).state, S::AirDodge);
    // An attack works a little later, from frame 45.
    launch(&mut sim);
    sim.state.fighters[1].air_dodge_used = false;
    for _ in 0..rules.hitstun_attack_cancel - 1 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.fighter(1).state, S::Hitstun);
    sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    sim.tick2(inp(0, 0, 0), inp(0, 0, ATTACK));
    assert_eq!(sim.fighter(1).state, S::Attack);
}
