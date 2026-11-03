//! Rolls out of a tech, a knockdown and a trip on their own frames (sim 149, D7; a public dump of the reference's game data). A tech
//! roll lasts 40 frames, intangible 1-20, on every body either way; a get-up roll 35, intangible from frame 1 through the body's own
//! window, which differs by body and by way (lying face up: the sim has one way of lying); a trip's roll 28, intangible 1-9. Frame 1
//! is the frame the roll begins on: the landing for a tech roll, the flick for the others. None is staled, however stale the fighter's
//! dodges are, none turns the fighter round, and each still moves the way it was flicked. A ruleset length of 0 plays the body's
//! shield roll, the rule before sim 149.

mod common;

use common::{fx, inp, Sim};
use sim_core::combat::is_intangible;
use sim_core::input::buttons::SHIELD;
use sim_core::state::{FighterState as S, NONE, ROLL_GETUP, ROLL_TECH, ROLL_TRIP};
use sim_core::{Fx, Vec2};

/// Each body's get-up roll windows lying face up, rolling forward and back (in roster order; a public dump of the reference's game
/// data: the last intangible frame).
const GETUP_WINDOWS: [(u16, u16); 14] = [
    (19, 19),
    (21, 22),
    (19, 22),
    (21, 22),
    (23, 19),
    (22, 22),
    (22, 22),
    (19, 24),
    (19, 24),
    (24, 31),
    (22, 22),
    (22, 22),
    (21, 22),
    (21, 22),
];

/// Body `ch` as player 0 in the middle of the stage facing `facing`, its dodges as stale as they get, with player 1 standing at the
/// far end and the others out.
fn alone(ch: u8, facing: i8) -> Sim {
    let mut sim = Sim::with_chars([ch, 0, 0, 0]);
    sim.stand(1, fx(10, 1), -1);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, facing);
    sim.state.fighters[0].dodge_stale = sim.content.rules.dodge_stale_max;
    sim
}

/// Player 0 of `sim` falls tumbling onto the stage, presses shield just before landing and flicks the stick `x` as it lands: the
/// landing frame, the tech roll's first.
fn tech_roll_with(mut sim: Sim, x: i8) -> Sim {
    let f = &mut sim.state.fighters[0];
    f.state = S::Hitstun;
    f.hitstun = 80;
    f.tumble = true;
    f.pos = Vec2::new(Fx::ZERO, fx(5, 10));
    f.vel = Vec2::new(Fx::ZERO, fx(-2, 10));
    f.platform = NONE;
    sim.tick(inp(0, 0, SHIELD));
    for t in 0..8 {
        let stick = if t == 0 { x } else { x / 5 * 4 };
        sim.tick(inp(stick, 0, SHIELD));
        if sim.f().state != S::Hitstun {
            break;
        }
    }
    assert_eq!(sim.f().state, S::Roll);
    assert_eq!(sim.f().roll_kind, ROLL_TECH);
    sim
}

/// Player 0 of `sim` lies in a knockdown (sits after a trip, with `tripped`) long enough to get up, then flicks the stick `x`: the
/// roll's first frame.
fn floor_roll_with(mut sim: Sim, tripped: bool, x: i8) -> Sim {
    let f = &mut sim.state.fighters[0];
    f.state = S::Knockdown;
    f.tripped = tripped;
    f.state_frame = 60;
    sim.tick(inp(x, 0, 0));
    assert_eq!(sim.f().state, S::Roll);
    let kind = if tripped { ROLL_TRIP } else { ROLL_GETUP };
    assert_eq!(sim.f().roll_kind, kind);
    sim
}

/// The roll begun this frame (its frame 1), played out with the stick let go: how many frames it lasted, the frames it was intangible
/// on, and how far it went. Checks it is not staled and keeps its facing on every frame and after.
fn play(sim: &mut Sim) -> (u16, Vec<u16>, Fx) {
    let facing = sim.f().facing;
    let x0 = sim.f().pos.x;
    assert_eq!(sim.f().dodge_level, 0, "a floor roll is never staled");
    let mut frame = 1u16;
    let mut intangible = Vec::new();
    while sim.f().state == S::Roll {
        assert_eq!(sim.f().facing, facing, "frame {frame}");
        if is_intangible(sim.f()) {
            intangible.push(frame);
        }
        sim.tick(inp(0, 0, 0));
        frame += 1;
        assert!(frame < 200, "the roll never ended");
    }
    assert_eq!(sim.f().state, S::Idle);
    assert_eq!(sim.f().facing, facing, "a floor roll never turns");
    (frame - 1, intangible, sim.f().pos.x - x0)
}

fn through(last: u16) -> Vec<u16> {
    (1..=last).collect()
}

/// Every body, facing either way, rolling either way: `start(body, facing, stick)` begins the roll and `check` gets what `play` found.
fn every_body_both_ways(
    start: fn(u8, i8, i8) -> Sim,
    check: impl Fn(u8, i8, i8, (u16, Vec<u16>, Fx)),
) {
    let bodies = Sim::new().content.fighters.len();
    for ch in 0..bodies as u8 {
        for facing in [1i8, -1] {
            for way in [1i8, -1] {
                let mut sim = start(ch, facing, 127 * way);
                let (frames, intangible, moved) = play(&mut sim);
                assert_eq!(
                    moved.signum_int(),
                    i32::from(way),
                    "body {ch} facing {facing} way {way}: it rolled the way it was flicked"
                );
                check(ch, facing, way, (frames, intangible, moved));
            }
        }
    }
}

#[test]
fn the_rulesets_floor_rolls_are_the_references() {
    let r = Sim::new().content.rules;
    assert_eq!((r.tech_roll_frames, r.tech_roll_intangible), (40, 20));
    assert_eq!(r.getup_roll_frames, 35);
    assert_eq!((r.trip_roll_frames, r.trip_roll_intangible), (28, 9));
    // (The reference's most common get-up windows, for a body without its own.)
    assert_eq!(
        (r.getup_roll_intangible, r.getup_roll_back_intangible),
        (21, 22)
    );
}

#[test]
fn every_body_tech_rolls_for_40_frames_intangible_from_the_landing_through_frame_20() {
    every_body_both_ways(
        |ch, facing, x| tech_roll_with(alone(ch, facing), x),
        |ch, facing, way, (frames, intangible, _)| {
            let at = format!("body {ch} facing {facing} way {way}");
            assert_eq!(frames, 40, "{at}");
            assert_eq!(intangible, through(20), "{at}");
        },
    );
}

#[test]
fn every_body_get_up_rolls_for_35_frames_intangible_through_its_own_window() {
    let content = Sim::new().content;
    assert_eq!(content.fighters.len(), GETUP_WINDOWS.len());
    for (ch, &(forward, back)) in GETUP_WINDOWS.iter().enumerate() {
        let p = content.fighters[ch];
        assert_eq!(
            (
                u16::from(p.getup_roll_intangible),
                u16::from(p.getup_roll_back_intangible)
            ),
            (forward, back),
            "body {ch}"
        );
    }
    every_body_both_ways(
        |ch, facing, x| floor_roll_with(alone(ch, facing), false, x),
        |ch, facing, way, (frames, intangible, _)| {
            let at = format!("body {ch} facing {facing} way {way}");
            let (forward, back) = GETUP_WINDOWS[usize::from(ch)];
            let window = if way == facing { forward } else { back };
            assert_eq!(frames, 35, "{at}");
            assert_eq!(intangible, through(window), "{at}");
        },
    );
}

#[test]
fn every_body_rolls_out_of_a_trip_for_28_frames_intangible_through_frame_9() {
    every_body_both_ways(
        |ch, facing, x| floor_roll_with(alone(ch, facing), true, x),
        |ch, facing, way, (frames, intangible, _)| {
            let at = format!("body {ch} facing {facing} way {way}");
            assert_eq!(frames, 28, "{at}");
            assert_eq!(intangible, through(9), "{at}");
        },
    );
}

#[test]
fn a_floor_roll_adds_no_dodge_staleness() {
    for kind in [ROLL_TECH, ROLL_GETUP, ROLL_TRIP] {
        let mut sim = alone(0, 1);
        sim.state.fighters[0].dodge_stale = 1;
        let mut sim = if kind == ROLL_TECH {
            tech_roll_with(sim, 127)
        } else {
            floor_roll_with(sim, kind == ROLL_TRIP, 127)
        };
        play(&mut sim);
        assert!(sim.f().dodge_stale <= 1, "kind {kind}");
    }
}

#[test]
fn a_body_without_its_own_get_up_window_takes_the_rulesets() {
    for (way, rule) in [(1i8, 21u16), (-1, 22)] {
        let mut sim = alone(0, 1);
        sim.content.fighters[0].getup_roll_intangible = 0;
        sim.content.fighters[0].getup_roll_back_intangible = 0;
        let mut sim = floor_roll_with(sim, false, 127 * way);
        let (frames, intangible, _) = play(&mut sim);
        assert_eq!(frames, 35, "way {way}");
        assert_eq!(intangible, through(rule), "way {way}");
    }
}

#[test]
fn a_ruleset_length_of_0_plays_the_bodys_shield_roll() {
    for kind in [ROLL_TECH, ROLL_GETUP, ROLL_TRIP] {
        for way in [1i8, -1] {
            let mut sim = alone(0, 1);
            sim.content.rules.tech_roll_frames = 0;
            sim.content.rules.getup_roll_frames = 0;
            sim.content.rules.trip_roll_frames = 0;
            let p = sim.content.fighters[0];
            let mut sim = if kind == ROLL_TECH {
                tech_roll_with(sim, 127 * way)
            } else {
                floor_roll_with(sim, kind == ROLL_TRIP, 127 * way)
            };
            let (frames, intangible, _) = play(&mut sim);
            let (length, start, end) = if way > 0 {
                (
                    p.roll_frames,
                    p.roll_intangible_start,
                    p.roll_intangible_end,
                )
            } else {
                (
                    p.roll_back_frames,
                    p.roll_back_intangible_start,
                    p.roll_back_intangible_end,
                )
            };
            let at = format!("kind {kind} way {way}");
            assert_eq!(frames, u16::from(length), "{at}");
            assert_eq!(
                intangible,
                (u16::from(start)..=u16::from(end)).collect::<Vec<_>>(),
                "{at}"
            );
        }
    }
}
