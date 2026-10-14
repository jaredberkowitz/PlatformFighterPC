//! The Popgun's down special, the banana peel (class 11, the scamp body), modelled mechanically on a reference character: E44 slip hits,
//! E45 traps, E47 a move's toss, the item's uses and the wait between peels. The neutral special is in `popgun_peanut.rs`, the up special
//! in `popgun_rocket.rs`. Reference data: the twelfth moveset's `REFERENCE_DATA.md` (`docs/movesets/popgun/`).

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::{ATTACK, GRAB, SHIELD, SPECIAL};
use sim_core::items;
use sim_core::moves::popgun_slots::{
    BANANA_ITEM, BANANA_KIND, PEEL_FAILS, PEEL_FAILS_WITH_ITEM, PEEL_WAIT, PEEL_WITH_ITEM,
};
use sim_core::moves::{MoveId, SLIP_HARD, SLIP_SOFT};
use sim_core::state::{FighterState as S, Item, ITEM_FREE, ITEM_HELD, ITEM_THROWN, NONE};
use sim_core::trig::{self, Angle};
use sim_core::{step, Fx, GameState, Rng, Vec2, MAX_FIGHTERS};

const SCAMP: u8 = 11;
const DUELIST: u8 = 0;

/// The scamp (player 0) at -3 facing right and a duelist (player 1) at 10 facing left, the others out of the match.
fn duel() -> Sim {
    let mut sim = Sim::with_chars([SCAMP, DUELIST, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-3), 1);
    sim.stand(1, Fx::from_int(10), -1);
    sim
}

fn peels(sim: &Sim) -> Vec<(usize, Item)> {
    sim.state
        .items
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, it)| it.active && it.kind == BANANA_KIND)
        .collect()
}

/// A peel made by the scamp (player 0) lying on the stage's top at `x`, its last thrower `owner`. Returns its slot.
fn lying_peel(sim: &mut Sim, x: Fx, owner: u8) -> usize {
    let n = items::make(
        &mut sim.state,
        &sim.content,
        0,
        BANANA_ITEM,
        Vec2::new(x, Fx::ZERO),
        MoveId::DownSpecial as u8,
    )
    .expect("a free slot");
    sim.state.items[n].platform = 0;
    sim.state.items[n].owner = owner;
    n
}

/// Starts the down special for the scamp by the input (stick down and special), then `n - 1` more frames with nothing pressed.
fn peel_frames(sim: &mut Sim, n: usize) {
    for k in 0..n {
        let input = if k == 0 {
            inp(0, -127, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick(input);
    }
}

// ---- The toss ----------------------------------------------------------------------------------------------------------------------

#[test]
fn the_peel_is_in_hand_on_frame_4_and_tossed_up_and_behind_on_20_and_control_returns_on_40() {
    let mut sim = duel();
    // (The stick already held down for longer than a flick's window, so the special is not a smash-style input: the tilt toss.)
    sim.ticks(6, inp(0, -127, 0));
    let mut in_hand = None;
    let mut tossed = None;
    let mut free = None;
    for frame in 1..=45u32 {
        let input = if frame == 1 {
            inp(0, -127, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick(input);
        if frame == 1 {
            assert_eq!(sim.f().move_id, MoveId::DownSpecial as u8);
        }
        let p = peels(&sim);
        if in_hand.is_none() && p.iter().any(|(_, it)| it.rest == ITEM_HELD) {
            in_hand = Some(frame);
        }
        if tossed.is_none() && p.iter().any(|(_, it)| it.rest == ITEM_THROWN) {
            tossed = Some(frame);
            // 105 degrees at 2.2 of the reference's units a frame times the scamp's toss strength (1 + (90 - 100) / 300), the peel's
            // 0.85 twice on the sideways speed and once on the upward [R].
            let it = p[0].1;
            let t = Fx::from_ratio(2200, 8000) * items::toss_strength(&sim.content.fighters[11]);
            let m = Fx::from_ratio(85, 100);
            let a = Angle::from_degrees(105);
            assert_eq!(
                it.vel.x,
                t * trig::cos(a) * m * m,
                "behind (the scamp faces right)"
            );
            assert!(it.vel.x < Fx::ZERO);
            // (Gravity has already pulled it once by the end of its first frame.)
            assert!((it.vel.y - t * trig::sin(a) * m).abs() <= Fx::from_ratio(1, 8));
        }
        if free.is_none() && sim.f().state != S::Attack {
            free = Some(frame);
        }
    }
    assert_eq!(in_hand, Some(4));
    assert_eq!(tossed, Some(20));
    // (Free on the 39th frame's end: it acts on the 40th, the move's FAF.)
    assert_eq!(free, Some(39), "the move's FAF 40");
    assert_eq!(peels(&sim).len(), 1);
    assert_eq!(sim.f().holding, NONE);
}

#[test]
fn a_smash_input_tosses_it_faster() {
    // The sideways speed the peel leaves with: the stick held down before special (a tilt) or flicked down with it (a smash input,
    // 3.036 instead of 2.2 [R]).
    let speed = |smash: bool| {
        let mut sim = duel();
        if !smash {
            sim.ticks(6, inp(0, -127, 0));
        }
        sim.tick(inp(0, -127, SPECIAL));
        // (Tossed on the 20th frame, the press being the first.)
        for _ in 0..19 {
            sim.tick(inp(0, 0, 0));
        }
        let (_, it) = peels(&sim)[0];
        assert_eq!(it.rest, ITEM_THROWN);
        it.vel.x.abs()
    };
    let ratio = speed(true) / speed(false);
    assert!((ratio - Fx::from_ratio(3036, 2200)).abs() < Fx::from_ratio(1, 100));
}

#[test]
fn one_peel_at_a_time_and_the_next_only_85_frames_after_the_last_toss() {
    let mut sim = duel();
    peel_frames(&mut sim, 45);
    assert_eq!(peels(&sim).len(), 1);
    // A second down special with the first out: no new peel, the longer ending.
    sim.tick(inp(0, -127, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, PEEL_FAILS as u8);
    let mut frames = 2;
    while sim.f().state == S::Attack {
        sim.tick(inp(0, 0, 0));
        frames += 1;
    }
    assert_eq!(frames + 1, 50, "no peel: FAF 50 [E]");
    assert_eq!(peels(&sim).len(), 1);
    // Once it is gone the wait counts from the last toss (on frame 20 of the first: 26 + 50 + 2 frames ago).
    let (n, _) = peels(&sim)[0];
    sim.state.items[n].active = false;
    let waited = 26 + frames;
    assert!(waited < usize::from(PEEL_WAIT));
    sim.tick(inp(0, -127, SPECIAL));
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().move_id, PEEL_FAILS as u8, "still waiting");
    while sim.f().state == S::Attack {
        sim.tick(inp(0, 0, 0));
    }
    sim.ticks(10, inp(0, 0, 0));
    assert_eq!(sim.f().pull_wait, 0);
    peel_frames(&mut sim, 80);
    assert_eq!(peels(&sim).len(), 1, "a new peel");
    assert_eq!(peels(&sim)[0].1.rest, ITEM_FREE, "landed");
}

#[test]
fn the_wait_is_85_frames_from_the_toss() {
    let mut sim = duel();
    peel_frames(&mut sim, 20);
    assert_eq!(sim.f().pull_wait, PEEL_WAIT, "set on the toss's frame");
    sim.ticks(84, inp(0, 0, 0));
    assert_eq!(sim.f().pull_wait, 1);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().pull_wait, 0);
}

#[test]
fn an_item_in_hand_is_tossed_first_on_14_and_the_peel_on_20() {
    let mut sim = duel();
    // The scamp holds a peel of another scamp's... here simply a peel it made, so this is the no-peel version: the held peel tossed.
    let n = lying_peel(&mut sim, Fx::from_int(-3), 0);
    {
        let it = &mut sim.state.items[n];
        it.platform = NONE;
        it.holder = 0;
        it.rest = ITEM_HELD;
    }
    sim.state.fighters[0].holding = n as i8;
    let mut left = None;
    for frame in 1..=50u32 {
        let input = if frame == 1 {
            inp(0, -127, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick(input);
        if frame == 2 {
            assert_eq!(
                sim.f().move_id,
                PEEL_FAILS_WITH_ITEM as u8,
                "its own peel held: one is out"
            );
        }
        if left.is_none() && sim.state.items[n].holder == NONE {
            left = Some(frame);
        }
    }
    assert_eq!(left, Some(14));
    assert_eq!(peels(&sim).len(), 1, "no second peel");

    // Holding an item that is not a peel of its own (another weapon's: a wanderer's bomb), with none out: the item on 14, a new peel
    // made then and tossed on 20.
    let mut sim = Sim::with_chars([SCAMP, 9, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::from_int(-3), 1);
    sim.stand(1, Fx::from_int(10), -1);
    let bomb = items::make(
        &mut sim.state,
        &sim.content,
        1,
        sim_core::moves::buckler_slots::BOMB_ITEM,
        Vec2::new(Fx::from_int(-3), Fx::ZERO),
        MoveId::DownSpecial as u8,
    )
    .unwrap();
    {
        let it = &mut sim.state.items[bomb];
        it.holder = 0;
        it.rest = ITEM_HELD;
    }
    sim.state.fighters[0].holding = bomb as i8;
    let mut bomb_left = None;
    let mut peel_made = None;
    let mut peel_tossed = None;
    for frame in 1..=45u32 {
        let input = if frame == 1 {
            inp(0, -127, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick(input);
        if frame == 2 {
            assert_eq!(sim.f().move_id, PEEL_WITH_ITEM as u8);
        }
        if bomb_left.is_none() && sim.state.items[bomb].holder == NONE {
            bomb_left = Some(frame);
        }
        let p = peels(&sim);
        if peel_made.is_none() && !p.is_empty() {
            peel_made = Some(frame);
        }
        if peel_tossed.is_none() && p.iter().any(|(_, it)| it.rest == ITEM_THROWN) {
            peel_tossed = Some(frame);
        }
    }
    assert_eq!(bomb_left, Some(14));
    assert_eq!(peel_made, Some(14));
    assert_eq!(peel_tossed, Some(20));
    let bomb_vx = sim.state.items[bomb].vel.x;
    let _ = bomb_vx;
}

// ---- The peel lying on the ground ---------------------------------------------------------------------------------------------------

/// The duelist (player 1) walks left into a peel lying at `x`; returns the frame it slipped on (1 = the first frame walking), if any.
fn walk_into(sim: &mut Sim) -> Option<u32> {
    for frame in 1..=120u32 {
        sim.tick2(inp(0, 0, 0), inp(-60, 0, 0));
        if sim.fighter(1).state == S::Knockdown {
            return Some(frame);
        }
    }
    None
}

#[test]
fn walking_into_a_lying_peel_is_a_soft_trip_with_no_hitlag() {
    let mut sim = duel();
    let n = lying_peel(&mut sim, Fx::from_int(6), 0);
    assert!(walk_into(&mut sim).is_some(), "it slipped");
    let d = *sim.fighter(1);
    assert!(d.tripped && d.trip_soft);
    assert_eq!(d.hitlag, 0, "no hitlag on a soft trip");
    assert_eq!(d.percent, Fx::ZERO);
    // One use spent; the peel flies up and away from it (to the left, the duelist came from the right).
    let it = sim.state.items[n];
    assert!(it.active);
    assert_eq!(it.used, 1);
    assert!(it.vel.y > Fx::ZERO && it.vel.x < Fx::ZERO);
    // Get-ups from frame 30, up by itself on frame 90 (the reference's soft trip).
    let rules = &sim.content.rules;
    assert_eq!((rules.slip_lag, rules.slip_max), (30, 90));
    let mut sitting = 0u32;
    while sim.fighter(1).state == S::Knockdown {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
        sitting += 1;
    }
    assert_eq!(
        sitting, 90,
        "a neutral get-up on its own after 90 frames sitting"
    );
}

#[test]
fn a_soft_trip_gets_up_from_frame_30() {
    let mut sim = duel();
    lying_peel(&mut sim, Fx::from_int(6), 0);
    walk_into(&mut sim).expect("slipped");
    let mut frame = 0u32;
    loop {
        frame += 1;
        sim.tick2(inp(0, 0, 0), inp(0, 127, 0));
        if sim.fighter(1).state == S::GetUp {
            break;
        }
        assert!(frame < 40);
    }
    assert_eq!(frame, 30);
}

#[test]
fn the_last_thrower_walks_over_its_own_peel_and_a_shield_does_not_stop_it() {
    // The scamp's own peel: it walks over it.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(15), -1);
    lying_peel(&mut sim, Fx::ZERO, 0);
    for _ in 0..40 {
        sim.tick(inp(60, 0, 0));
    }
    assert!(!sim.f().tripped, "its own peel never trips it");
    // An opponent's (the duelist threw it last): it slips.
    let mut sim = duel();
    sim.stand(1, Fx::from_int(15), -1);
    lying_peel(&mut sim, Fx::ZERO, 1);
    let mut slipped = false;
    for _ in 0..40 {
        sim.tick(inp(60, 0, 0));
        slipped |= sim.f().tripped;
    }
    assert!(slipped);
    assert!(sim.f().trip_soft);
    // Shielding on it: it still slips (the reference's unblockable peel).
    let mut sim = duel();
    sim.stand(1, Fx::from_int(15), -1);
    let x = sim.fighter(0).pos.x;
    sim.tick(inp(0, 0, SHIELD));
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    lying_peel(&mut sim, x, 1);
    sim.tick(inp(0, 0, SHIELD));
    assert!(sim.f().tripped);
}

#[test]
fn two_trips_use_the_peel_up() {
    let mut sim = duel();
    let n = lying_peel(&mut sim, Fx::from_int(6), 0);
    walk_into(&mut sim).expect("the first slip");
    assert!(sim.state.items[n].active);
    // Let the duelist stand up and the peel come down, then walk into it again.
    for _ in 0..200 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    assert_eq!(sim.state.items[n].rest, ITEM_FREE, "lying again");
    let x = sim.state.items[n].pos.x;
    sim.stand(1, x + Fx::from_int(2), -1);
    walk_into(&mut sim).expect("the second slip");
    assert!(!sim.state.items[n].active, "two uses: gone");
}

// ---- Thrown ---------------------------------------------------------------------------------------------------------------------------

/// The scamp holding a peel (its own, lying-made) and the duelist `gap` ahead; returns the peel's slot.
fn holding_peel(gap: Fx) -> (Sim, usize) {
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    let n = lying_peel(&mut sim, Fx::ZERO, 0);
    let it = &mut sim.state.items[n];
    it.platform = NONE;
    it.holder = 0;
    it.rest = ITEM_HELD;
    sim.state.fighters[0].holding = n as i8;
    (sim, n)
}

#[test]
fn a_thrown_peel_hard_trips_a_standing_fighter_with_its_damage_and_hitlag() {
    let (mut sim, n) = holding_peel(Fx::from_int(3));
    // A forward throw (attack with the stick forward).
    sim.tick(inp(127, 0, ATTACK));
    let mut tripped = false;
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.fighter(1).tripped {
            tripped = true;
            break;
        }
    }
    assert!(tripped);
    let d = *sim.fighter(1);
    assert!(!d.trip_soft, "a hard trip");
    assert!(d.hitlag > 0, "with hitlag");
    // Its damage: 0 (a throw does not halve it) + 1 + 1.7 a reference unit of throw speed (the stick flicked forward with attack: the
    // smash forward throw's 3.64 x 0.966667) [R].
    let (speed, _) = items::throw_speed(
        &sim.content.rules,
        items::THROW_FORWARD | items::THROW_SMASH,
    );
    let t = speed * items::toss_strength(&sim.content.fighters[11]);
    let damage = Fx::ONE + Fx::from_ratio(136, 10) * t;
    assert_eq!(d.percent, damage * sim.content.rules.damage_mult);
    // A fighter's throw and a hit: two uses, gone.
    assert!(!sim.state.items[n].active);
}

#[test]
fn the_moves_toss_hits_for_one_plus_1_7_per_unit_and_bounces_off_high() {
    // The scamp tosses a peel behind it onto the duelist standing there.
    let mut sim = duel();
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, Fx::from_int(-2), 1);
    let mut hit = None;
    for frame in 1..=60u32 {
        let input = if frame == 1 {
            inp(0, -127, SPECIAL)
        } else {
            inp(0, 0, 0)
        };
        sim.tick(input);
        if hit.is_none() && sim.fighter(1).percent > Fx::ZERO {
            hit = Some(frame);
            let (_, it) = peels(&sim)[0];
            assert_eq!(it.used, 1, "one use: the toss itself is none");
            assert!(it.vel.y > Fx::ZERO, "bounced off");
        }
    }
    assert!(hit.is_some(), "it came down on the duelist");
    // A drop's damage: the base halved (0), + 1, + 1.7 a reference unit of the toss's speed (the stick flicked down with special: the
    // smash toss's 3.036) x 0.966667 [R: 1 + 1.7 x toss speed].
    let t = Fx::from_ratio(3036, 8000) * items::toss_strength(&sim.content.fighters[11]);
    let damage = Fx::ONE + Fx::from_ratio(136, 10) * t;
    assert_eq!(
        sim.fighter(1).percent,
        damage * sim.content.rules.damage_mult
    );
    assert!(sim.fighter(1).tripped, "standing: tripped");
}

#[test]
fn a_thrown_peel_launches_a_fighter_in_the_air() {
    let (mut sim, _) = holding_peel(Fx::from_int(3));
    sim.put_airborne(1, Fx::from_int(3), fx(8, 10), Fx::ZERO, Fx::ZERO);
    sim.tick(inp(127, 0, ATTACK));
    let mut launched = false;
    for _ in 0..30 {
        sim.tick(inp(0, 0, 0));
        launched |= sim.fighter(1).state == S::Hitstun;
    }
    assert!(launched);
    assert!(!sim.fighter(1).tripped);
}

#[test]
fn anyone_picks_it_up_and_then_it_trips_the_scamp() {
    // The duelist picks up the scamp's peel and throws it at the scamp's feet: it is the duelist's now.
    let mut sim = duel();
    sim.stand(0, Fx::from_int(-6), 1);
    sim.stand(1, Fx::from_int(2), -1);
    let n = lying_peel(&mut sim, Fx::from_int(1), 0);
    sim.tick2(inp(0, 0, 0), inp(0, 0, GRAB));
    assert_eq!(sim.state.items[n].holder, 1, "picked up by the duelist");
    for _ in 0..10 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    // A drop (grab with the stick in the middle): it lies in front of the duelist, the duelist's.
    sim.tick2(inp(0, 0, 0), inp(0, 0, GRAB));
    for _ in 0..40 {
        sim.tick2(inp(0, 0, 0), inp(0, 0, 0));
    }
    let it = sim.state.items[n];
    assert!(it.active);
    assert_eq!(it.owner, 1);
    assert_eq!(it.used, 1, "the drop is a throw: one use");
    // The scamp walks into it: it slips, and the peel's second use is spent: gone.
    sim.stand(0, it.pos.x - Fx::from_int(2), 1);
    let mut slipped = false;
    for _ in 0..40 {
        sim.tick2(inp(60, 0, 0), inp(0, 0, 0));
        slipped |= sim.f().tripped;
    }
    assert!(slipped);
    assert!(!sim.state.items[n].active);
}

#[test]
fn the_peels_hits_are_slips_and_it_never_clanks() {
    let it = sim_core::moves::popgun_slots::banana_item();
    assert_eq!(it.hit.slip, SLIP_HARD);
    assert!(!it.hit.clanks);
    assert_eq!(it.trap.map(|t| t.hit.slip), Some(SLIP_SOFT));
    assert_eq!(it.uses, 2);
    assert_eq!(it.damage_add, Fx::ONE);
    assert_eq!(it.throw_mul, Fx::from_ratio(85, 100));
}

// ---- Determinism ----------------------------------------------------------------------------------------------------------------

#[test]
fn peels_resimulate_exactly_from_a_snapshot() {
    let mut sim = duel();
    let mut frames: Vec<[sim_core::Input; MAX_FIGHTERS]> = Vec::new();
    let mut rng = Rng::new(7);
    for k in 0..600u32 {
        let mut i = [sim_core::Input::default(); MAX_FIGHTERS];
        i[0] = if k % 90 == 0 {
            inp(0, -127, SPECIAL)
        } else {
            inp(if rng.range(3) == 0 { 60 } else { 0 }, 0, 0)
        };
        i[1] = inp(if k % 200 < 100 { -60 } else { 60 }, 0, 0);
        frames.push(i);
    }
    let start: GameState = sim.state;
    let mut straight = start;
    let mut made = 0;
    for i in &frames {
        step(&mut straight, &sim.content, i);
        made = made.max(
            straight
                .items
                .iter()
                .filter(|it| it.active && it.kind == BANANA_KIND)
                .count(),
        );
    }
    assert!(made > 0, "peels were made");
    let mut snap = start;
    for i in &frames[..300] {
        step(&mut snap, &sim.content, i);
    }
    let mut resumed = snap;
    for i in &frames[300..] {
        step(&mut resumed, &sim.content, i);
    }
    assert_eq!(resumed.checksum(), straight.checksum());
    sim.state = straight;
}
