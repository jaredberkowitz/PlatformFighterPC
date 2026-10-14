//! Move rules added with the Greatsword's phase 2 (sim 85), each general and off by default: E11's `start_speed_y` (a vertical
//! speed multiplier on an aerial start) and `ledge_drop` (ledges only after falling a distance, from `Fighter::move_y0`), E22b's
//! `on_land` (an aerial move going on as its grounded version on landing, keeping its frame), E3f's `Hitbox::half_height` (a standing
//! capsule hitbox, used by tall projectiles). Plus: every new content field is in the content hash.

mod common;

use common::{fx, inp, Sim};
use sim_core::input::buttons::SPECIAL;
use sim_core::moves::{
    DropThrough, Hitbox, Motion, Move, MoveId, ProjectileArc, ProjectileSpawn, Tool, MAX_PHASES,
};
use sim_core::state::FighterState as S;
use sim_core::Fx;

/// The duelist (player 0) with `mv` in slot `id`, the duelist at the far end, the others out.
fn with_move(id: MoveId, mv: Move) -> Sim {
    let mut sim = Sim::alone();
    sim.content.weapons[0].moves[id as usize] = mv;
    sim
}

fn hit(radius: Fx, half_height: Fx) -> Hitbox {
    let mut hb =
        sim_core::Content::placeholder().weapons[0].moves[MoveId::Jab as usize].hitboxes[0];
    hb.start = 1;
    hb.end = 1;
    hb.radius = radius;
    hb.half_height = half_height;
    hb.damage = Fx::from_int(5);
    hb
}

#[test]
fn start_speed_y_keeps_a_share_of_the_vertical_speed_on_an_aerial_start() {
    let vy_after = |keep: u8| {
        let mut sim = with_move(
            MoveId::NSpecial,
            Move {
                total_frames: 30,
                start_speed_y: keep,
                hitboxes: vec![hit(Fx::ONE, Fx::ZERO)],
                ..Move::empty()
            },
        );
        sim.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ONE);
        sim.tick(inp(0, 0, SPECIAL));
        assert_eq!(sim.f().state, S::Attack);
        sim.f().vel.y
    };
    let all = vy_after(100);
    let part = vy_after(65);
    let g = sim_core::Content::placeholder().fighters[0].gravity;
    // 0.65 of the speed it had (the gravity of one frame either way).
    assert!(part < all);
    assert!((part - Fx::from_ratio(65, 100)).abs() <= g, "{part:?}");
    // On the ground nothing changes (it applies to an aerial start only).
    assert_eq!(Move::empty().start_speed_y, 100);
}

/// Player 0 falls straight down past the stage's right ledge (facing the stage) in a move that catches ledges in front from its
/// first frame, starting `above` world units above the ledge; returns whether it caught it.
fn falls_past_the_ledge(above: Fx, drop: Option<(Fx, Fx)>) -> bool {
    let mut sim = with_move(
        MoveId::UpSpecial,
        Move {
            total_frames: 80,
            motion: vec![Motion {
                start: 0,
                end: 79,
                vx: Fx::ZERO,
                vy: fx(-1, 8),
            }],
            ledge_front_from: 0,
            ledge_back_from: 0,
            ledge_drop: drop,
            hitboxes: vec![hit(Fx::ONE, Fx::ZERO)],
            ..Move::empty()
        },
    );
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .expect("a right ledge");
    sim.put_airborne(0, ledge.x + fx(12, 10), ledge.y + above, Fx::ZERO, Fx::ZERO);
    sim.state.fighters[0].facing = -1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UpSpecial);
    for _ in 0..80 {
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            return true;
        }
    }
    false
}

#[test]
fn ledge_drop_catches_a_ledge_only_after_falling_far_enough_from_where_the_move_began() {
    let five = Some((Fx::from_int(5), Fx::from_int(5)));
    assert!(falls_past_the_ledge(Fx::ONE, None), "no rule: caught");
    assert!(
        !falls_past_the_ledge(Fx::ONE, five),
        "fallen 3.6 at most by the ledge: missed"
    );
    assert!(
        falls_past_the_ledge(Fx::from_int(3), five),
        "fallen 5 by then: caught"
    );
}

#[test]
fn begin_attack_remembers_where_the_move_began() {
    let mut sim = Sim::alone();
    sim.put_airborne(0, Fx::ZERO, Fx::from_int(7), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::NAir);
    assert_eq!(sim.f().move_y0, Fx::from_int(7));
}

/// Player 0 falls onto the stage during an aerial special (ext14) whose grounded version is the neutral special (with `on_land`) or
/// that has none; returns the state, move and frame on the tick after it touched down.
fn land_in(on_land: bool) -> (S, u8, u16, bool) {
    let mut sim = Sim::alone();
    let w = &mut sim.content.weapons[0].moves;
    w[MoveId::NSpecial as usize] = Move {
        total_frames: 40,
        hitboxes: vec![Hitbox {
            start: 30,
            end: 31,
            ..hit(Fx::ONE, Fx::ZERO)
        }],
        ..Move::empty()
    };
    w[MoveId::Ext14 as usize] = Move {
        total_frames: 45,
        landing_lag: 9,
        hitboxes: vec![Hitbox {
            start: 30,
            end: 31,
            ..hit(Fx::ONE, Fx::ZERO)
        }],
        on_land: on_land.then_some(MoveId::NSpecial as u8),
        ..Move::empty()
    };
    sim.put_airborne(0, Fx::ZERO, fx(5, 10), Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Ext14);
    for _ in 0..40 {
        sim.tick(inp(0, 0, 0));
        if sim.f().grounded() {
            break;
        }
    }
    let f = sim.f();
    (f.state, f.move_id, f.state_frame, f.grounded())
}

#[test]
fn on_land_goes_on_as_the_grounded_version_keeping_the_frame() {
    let (state, id, frame, grounded) = land_in(true);
    assert_eq!(
        (state, id, grounded),
        (S::Attack, MoveId::NSpecial as u8, true)
    );
    assert!(frame > 0, "the frame is kept, not restarted: {frame}");
    let (state, _, _, _) = land_in(false);
    assert_eq!(state, S::Landing, "without it: the move's landing lag");
}

#[test]
fn a_tall_projectile_hitbox_reaches_above_its_circle() {
    // A shot 3 world units up: its circle passes over a standing duelist's head, the capsule (1.0 above and below) does not.
    let shot = |half: Fx| {
        let mut sim = with_move(
            MoveId::NSpecial,
            Move {
                total_frames: 40,
                projectile: Some(ProjectileSpawn {
                    frame: 1,
                    x: Fx::ONE,
                    y: Fx::from_int(3),
                    speed: fx(2, 10),
                    life: 60,
                    hitbox: hit(fx(3, 10), half),
                    end_damage: Fx::from_int(5),
                    arc: ProjectileArc::NONE,
                    phases: [None; MAX_PHASES],
                    pierce: false,
                    rehit: 0,
                    burst: None,
                    charged: None,
                    tool: Tool::NONE,
                }),
                ..Move::empty()
            },
        );
        sim.stand(0, Fx::ZERO, 1);
        sim.stand(1, Fx::from_int(4), -1);
        sim.tick(inp(0, 0, SPECIAL));
        sim.ticks(60, inp(0, 0, 0));
        sim.fighter(1).percent
    };
    assert_eq!(shot(Fx::ZERO), Fx::ZERO, "the circle passes over");
    assert!(shot(Fx::ONE) > Fx::ZERO, "the capsule hits");
}

#[test]
fn the_new_fields_are_in_the_content_hash() {
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    for (name, edit) in [
        (
            "limit_version",
            (|m| m.limit_version = Some(30)) as fn(&mut Move),
        ),
        ("gauge_drain", |m| m.gauge_drain = true),
        ("start_speed_y", |m| m.start_speed_y = 65),
        ("ledge_drop", |m| m.ledge_drop = Some((Fx::ONE, Fx::ONE))),
        ("on_land", |m| m.on_land = Some(13)),
        ("half_height", |m| m.hitboxes[0].half_height = Fx::ONE),
        ("motion_drift", |m| m.motion_drift = 50),
    ] {
        let mut c = base.clone();
        edit(&mut c.weapons[0].moves[MoveId::NAir as usize]);
        assert_ne!(h, c.hash(), "{name}");
    }
    let mut c = base.clone();
    c.fighters[0].gauge.max = Fx::from_int(100);
    assert_ne!(h, c.hash(), "the gauge");
    for i in 0..9 {
        let mut c = base.clone();
        let g = &mut c.fighters[6].gauge;
        match i {
            0 => g.walk += 1,
            1 => g.dash += 1,
            2 => g.run += 1,
            3 => g.traction += 1,
            4 => g.air_speed += 1,
            5 => g.air_accel += 1,
            6 => g.air_friction += 1,
            7 => g.gravity += 1,
            _ => g.fall += 1,
        }
        assert_ne!(h, c.hash(), "gauge percent {i}");
    }
    for edit in [
        (|g| g.per_dealt = Fx::ONE) as fn(&mut sim_core::content::Gauge),
        |g| g.per_taken = Fx::HALF,
        |g| g.mode_frames = 1,
    ] {
        let mut c = base.clone();
        edit(&mut c.fighters[6].gauge);
        assert_ne!(h, c.hash());
    }
}

// ---- Sim 95: ledges only while not rising, a move's own platform drop (E30) ----------------------------------------------------

/// Player 0 beside the right ledge (as `falls_past_the_ledge`), starting 2.4 units below it and rising at 1/8 a frame for `rise` frames,
/// then falling at 1/8, during a move that catches ledges from its first frame (`falling_only` or not); returns the move frame the
/// ledge was caught on (None if never).
fn rise_by_the_ledge(rise: u8, falling_only: bool) -> Option<u16> {
    let mut motion = vec![Motion {
        start: 0,
        end: rise.saturating_sub(1),
        vx: Fx::ZERO,
        vy: fx(1, 8),
    }];
    motion.push(Motion {
        start: rise,
        end: 79,
        vx: Fx::ZERO,
        vy: fx(-1, 8),
    });
    let mut sim = with_move(
        MoveId::UpSpecial,
        Move {
            total_frames: 80,
            motion,
            ledge_front_from: 0,
            ledge_falling_only: falling_only,
            hitboxes: vec![hit(Fx::ONE, Fx::ZERO)],
            ..Move::empty()
        },
    );
    let ledge = *sim
        .content
        .stage
        .ledges
        .iter()
        .find(|l| l.side > 0)
        .expect("a right ledge");
    sim.put_airborne(
        0,
        ledge.x + fx(12, 10),
        ledge.y - fx(24, 10),
        Fx::ZERO,
        Fx::ZERO,
    );
    sim.state.fighters[0].facing = -1;
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::UpSpecial);
    for _ in 0..80 {
        let frame = sim.f().state_frame + 1;
        sim.tick(inp(0, 0, 0));
        if sim.f().state == S::LedgeHang {
            return Some(frame);
        }
    }
    None
}

#[test]
fn ledge_falling_only_catches_a_ledge_only_once_the_fighter_stops_rising() {
    // Rising past the ledge the whole time: caught on the way up without the rule, never with it.
    assert!(rise_by_the_ledge(70, false).is_some());
    assert_eq!(rise_by_the_ledge(70, true), None);
    // Rising for 12 frames then falling back: with the rule, caught only after the fall begins.
    let up = rise_by_the_ledge(12, false).expect("caught rising");
    let down = rise_by_the_ledge(12, true).expect("caught falling");
    assert!(up < 12 && down >= 12, "rising {up}, falling {down}");
}

/// Player 0 starts `above` world units over the left soft platform (x -5.5, y 3.6) and falls at 1/8 a frame during a move with
/// `drop` (holding the stick down if `down`); returns its height after 60 frames, or where it touched down.
fn fall_onto_the_soft_platform(above: Fx, drop: Option<DropThrough>, down: bool) -> Fx {
    let mut sim = with_move(
        MoveId::DownSpecial,
        Move {
            total_frames: 80,
            motion: vec![Motion {
                start: 0,
                end: 79,
                vx: Fx::ZERO,
                vy: fx(-1, 8),
            }],
            drop_through: drop,
            hitboxes: vec![hit(Fx::ONE, Fx::ZERO)],
            ..Move::empty()
        },
    );
    sim.put_airborne(0, fx(-55, 10), fx(36, 10) + above, Fx::ZERO, Fx::ZERO);
    sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::DownSpecial);
    for _ in 0..60 {
        sim.tick(inp(0, if down { -127 } else { 0 }, 0));
        if sim.f().grounded() {
            break;
        }
    }
    sim.f().pos.y
}

#[test]
fn drop_through_passes_soft_platforms_for_its_frames_and_while_down_is_held() {
    let soft = fx(36, 10);
    let seven = Some(DropThrough {
        from: 1,
        frames: 7,
        while_down: false,
    });
    // Half a unit above: without the rule it lands on the platform; with 7 frames of drop it is through before they run out, and
    // lands on the stage's floor.
    assert_eq!(fall_onto_the_soft_platform(fx(1, 2), None, false), soft);
    assert_eq!(
        fall_onto_the_soft_platform(fx(1, 2), seven, false),
        Fx::ZERO
    );
    // Two units above: the 7 frames run out first, so it lands on it, unless down is held (with `while_down`).
    assert_eq!(
        fall_onto_the_soft_platform(Fx::from_int(2), seven, true),
        soft
    );
    let held = Some(DropThrough {
        from: 1,
        frames: 7,
        while_down: true,
    });
    assert_eq!(
        fall_onto_the_soft_platform(Fx::from_int(2), held, false),
        soft
    );
    assert_eq!(
        fall_onto_the_soft_platform(Fx::from_int(2), held, true),
        Fx::ZERO
    );
}

#[test]
fn the_shell_and_drop_fields_are_in_the_content_hash() {
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    for (name, edit) in [
        (
            "late_next",
            (|m| m.late_next = Some((17, 29))) as fn(&mut Move),
        ),
        ("ledge_falling_only", |m| m.ledge_falling_only = true),
        ("drop_through", |m| {
            m.drop_through = Some(DropThrough {
                from: 1,
                frames: 7,
                while_down: false,
            })
        }),
    ] {
        let mut c = base.clone();
        edit(&mut c.weapons[0].moves[MoveId::NAir as usize]);
        assert_ne!(h, c.hash(), "{name}");
    }
    // The drop's frames and its down rule each count.
    let with_drop = |d: DropThrough| {
        let mut c = base.clone();
        c.weapons[0].moves[MoveId::NAir as usize].drop_through = Some(d);
        c.hash()
    };
    let d = DropThrough {
        from: 1,
        frames: 7,
        while_down: false,
    };
    assert_ne!(with_drop(d), with_drop(DropThrough { frames: 8, ..d }));
    assert_ne!(
        with_drop(d),
        with_drop(DropThrough {
            while_down: true,
            ..d
        })
    );
    // A shell's flight and burst options (the Clutch's up special, class 8's weapon).
    let shell = |edit: fn(&mut ProjectileSpawn)| {
        let mut c = base.clone();
        let w = usize::from(c.fighters[8].weapon);
        let p = c.weapons[w].moves[MoveId::UpSpecial as usize]
            .projectile
            .as_mut()
            .expect("the shell");
        edit(p);
        c.hash()
    };
    for edit in [
        (|p| p.arc.bounce_keep_y = 79) as fn(&mut ProjectileSpawn),
        |p| p.arc.bounce_max_fall = Fx::ONE,
        |p| p.burst.as_mut().expect("burst").on_ground = false,
        |p| p.burst.as_mut().expect("burst").on_expire = false,
        |p| p.burst.as_mut().expect("burst").keep_hits = false,
    ] {
        assert_ne!(h, shell(edit));
    }
    let _ = ProjectileArc::NONE;
}

#[test]
fn the_tool_fields_are_in_the_content_hash() {
    // Sim 99 (E35, E3b, E3d, E38): every new field of a projectile, a hitbox and a move changes the hash.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let shot = |edit: fn(&mut ProjectileSpawn)| {
        let mut c = base.clone();
        let p = c.weapons[1].moves[MoveId::NSpecial as usize]
            .projectile
            .as_mut()
            .expect("the blaster");
        edit(p);
        c.hash()
    };
    for edit in [
        (|p| p.arc.drag = Fx::ONE) as fn(&mut ProjectileSpawn),
        |p| p.arc.lodge = 1,
        |p| p.arc.lodge_angle = 9,
        |p| p.hitbox.half_width = Fx::ONE,
        |p| p.tool.kind = 1,
        |p| p.tool.limit = 1,
        |p| p.tool.catch_from = 4,
        |p| p.tool.catch_move = Some(0),
        |p| {
            p.tool.on_clank = Some(sim_core::moves::Fall {
                life: 1,
                vx: Fx::ZERO,
                vy: Fx::ZERO,
                gravity: Fx::ZERO,
                max_fall: Fx::ZERO,
            })
        },
        |p| {
            p.charged = Some(sim_core::moves::ChargedShot {
                full: 60,
                snap: 0,
                damage_to: Fx::ONE,
                speed_to: Fx::ONE,
                life_to: 1,
                size_from: 100,
                size_to: 100,
                floor: false,
                times: 100,
                power_to: 100,
            })
        },
    ] {
        assert_ne!(h, shot(edit));
    }
    let mut c = base.clone();
    c.weapons[1].moves[MoveId::NSpecial as usize].smash_version = Some(0);
    assert_ne!(h, c.hash(), "smash_version");
    let _ = Tool::NONE;
}

#[test]
fn the_egg_is_in_the_content_hash() {
    // E8 (sim 107): the block and every one of its numbers.
    use sim_core::moves::Encase;
    let base = sim_core::Content::placeholder();
    let lay = sim_core::moves::clutch_slots::LAY as usize;
    let w = base
        .weapons
        .iter()
        .position(|w| w.moves[lay].encase.is_some())
        .expect("the Clutch's lay");
    let e = base.weapons[w].moves[lay].encase.unwrap();
    let h = base.hash();
    let one = Fx::ONE;
    let edits: [(&str, Encase); 18] = [
        (
            "base",
            Encase {
                base: e.base + one,
                ..e
            },
        ),
        (
            "per_percent",
            Encase {
                per_percent: e.per_percent + one,
                ..e
            },
        ),
        (
            "percent_cap",
            Encase {
                percent_cap: e.percent_cap + one,
                ..e
            },
        ),
        (
            "mash_stick",
            Encase {
                mash_stick: e.mash_stick + one,
                ..e
            },
        ),
        (
            "mash_button",
            Encase {
                mash_button: e.mash_button + one,
                ..e
            },
        ),
        (
            "hit_cut",
            Encase {
                hit_cut: e.hit_cut + one,
                ..e
            },
        ),
        (
            "damage_taken",
            Encase {
                damage_taken: 50,
                ..e
            },
        ),
        (
            "launch x",
            Encase {
                launch: (one, e.launch.1),
                ..e
            },
        ),
        (
            "launch y",
            Encase {
                launch: (e.launch.0, one),
                ..e
            },
        ),
        ("grow_from", Encase { grow_from: 50, ..e }),
        (
            "grow_frames",
            Encase {
                grow_frames: 30,
                ..e
            },
        ),
        ("size", Encase { size: 90, ..e }),
        (
            "shift x",
            Encase {
                shift: (one, e.shift.1),
                ..e
            },
        ),
        (
            "shift y",
            Encase {
                shift: (e.shift.0, one),
                ..e
            },
        ),
        (
            "air_shift x",
            Encase {
                air_shift: (one, e.air_shift.1),
                ..e
            },
        ),
        (
            "air_shift y",
            Encase {
                air_shift: (e.air_shift.0, one),
                ..e
            },
        ),
        ("exit_vy", Encase { exit_vy: one, ..e }),
        (
            "exit_intangible",
            Encase {
                exit_intangible: 9,
                ..e
            },
        ),
    ];
    for (name, edit) in edits {
        let mut c = base.clone();
        c.weapons[w].moves[lay].encase = Some(edit);
        assert_ne!(h, c.hash(), "{name}");
    }
    let mut c = base.clone();
    c.weapons[w].moves[lay].encase = None;
    assert_ne!(h, c.hash(), "the block");
}

#[test]
fn the_item_fields_are_in_the_content_hash() {
    // Sim 111 (E4): every number of an item, its blast, a move's pull and spend, a shot's lodge item and charged share, a body's item
    // throws and the ruleset's item rules change the hash.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[9].weapon);
    assert!(!base.weapons[w].items.is_empty(), "the wanderer's items");
    let changed = |edit: &dyn Fn(&mut sim_core::Content)| {
        let mut c = base.clone();
        edit(&mut c);
        c.hash()
    };
    for k in 0..24 {
        assert_ne!(
            h,
            changed(&|c| {
                let mut it = c.weapons[w].items[0];
                *it.fx_fields_mut()[k].1 += Fx::from_raw(1);
                c.weapons[w].items[0] = it;
            }),
            "item value {k}"
        );
    }
    for k in 0..9 {
        assert_ne!(
            h,
            changed(&|c| {
                let mut it = c.weapons[w].items[0];
                *it.u8_fields_mut()[k].1 += 1;
                c.weapons[w].items[0] = it;
            }),
            "item percent {k}"
        );
    }
    type Edit = (&'static str, fn(&mut sim_core::Content, usize));
    let edits: [Edit; 13] = [
        ("kind", |c, w| c.weapons[w].items[0].kind += 1),
        ("thrown_life", |c, w| c.weapons[w].items[0].thrown_life += 1),
        ("vanish", |c, w| c.weapons[w].items[0].vanish ^= true),
        ("life", |c, w| c.weapons[w].items[0].life += 1),
        ("hit", |c, w| c.weapons[w].items[0].hit.base_knockback += 1),
        ("blast", |c, w| c.weapons[w].items[0].blast = None),
        ("blast delay", |c, w| {
            if let Some(b) = c.weapons[w].items[0].blast.as_mut() {
                b.delay += 1;
            }
        }),
        ("blast hits_owner", |c, w| {
            if let Some(b) = c.weapons[w].items[0].blast.as_mut() {
                b.hits_owner ^= true;
            }
        }),
        ("items", |c, w| c.weapons[w].items.truncate(1)),
        ("pull", |c, w| {
            c.weapons[w].moves[MoveId::DownSpecial as usize].pull = None
        }),
        ("spend_item", |c, w| {
            c.weapons[w].moves[MoveId::Jab as usize].spend_item = 3;
        }),
        ("lodge_item", |c, w| {
            if let Some(p) = c.weapons[w].moves[sim_core::moves::buckler_slots::BOW_FIRE as usize]
                .projectile
                .as_mut()
            {
                p.arc.lodge_item = None;
            }
        }),
        ("times", |c, w| {
            if let Some(ch) = c.weapons[w].moves[sim_core::moves::buckler_slots::BOW_FIRE as usize]
                .projectile
                .as_mut()
                .and_then(|p| p.charged.as_mut())
            {
                ch.times = 90;
            }
        }),
    ];
    for (name, edit) in edits {
        let mut c = base.clone();
        edit(&mut c, w);
        assert_ne!(h, c.hash(), "{name}");
    }
    let mut c = base.clone();
    c.fighters[0].item_throws.drop += 1;
    assert_ne!(h, c.hash(), "item_throws");
    let rules: [fn(&mut sim_core::content::Ruleset); 6] = [
        |r| r.item_pick_frames += 1,
        |r| r.item_side_frames += 1,
        |r| r.item_side_speed += Fx::from_raw(1),
        |r| r.item_drop_angle += 1,
        |r| r.item_reach_back += Fx::from_raw(1),
        |r| r.item_hand_y += Fx::from_raw(1),
    ];
    for edit in rules {
        let mut c = base.clone();
        edit(&mut c.rules);
        assert_ne!(h, c.hash(), "a ruleset item rule");
    }
}

#[test]
fn the_pocket_and_tree_fields_are_in_the_content_hash() {
    // Sim 115 (owner decision 70, E42, E43): a jab cancel, a pocket and its release, a chopping move, a projectile's pocket rule and
    // water, an item's pocketability and placed block each change the hash.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[10].weapon);
    let edits: [fn(&mut sim_core::Content, usize); 9] = [
        |c, w| c.weapons[w].moves[MoveId::Jab as usize].cancel_from = 5,
        |c, w| {
            if let Some(p) = c.weapons[w].moves[MoveId::NSpecial as usize]
                .pocket
                .as_mut()
            {
                p.end = 20;
            }
        },
        |c, w| {
            if let Some(p) = c.weapons[w].moves[MoveId::Ext9 as usize].unpocket.as_mut() {
                p.power = 150;
            }
        },
        |c, w| c.weapons[w].moves[MoveId::Ext12 as usize].chops = false,
        |c, _| {
            if let Some(p) = c.weapons[1].moves[MoveId::NSpecial as usize]
                .projectile
                .as_mut()
            {
                p.tool.pocket = sim_core::moves::POCKET_NEVER;
            }
        },
        |c, _| {
            if let Some(p) = c.weapons[1].moves[MoveId::NSpecial as usize]
                .projectile
                .as_mut()
            {
                p.tool.water = 2;
            }
        },
        |c, w| c.weapons[w].items[0].pocketable = true,
        |c, w| {
            if let Some(p) = c.weapons[w].items[1].placed.as_mut() {
                p.chip_chance = 1;
            }
        },
        |c, w| {
            if let Some(p) = c.weapons[w].items[0].placed.as_mut() {
                p.grow_delay = 1;
            }
        },
    ];
    for (k, edit) in edits.iter().enumerate() {
        let mut c = base.clone();
        edit(&mut c, w);
        assert_ne!(c.hash(), h, "edit {k}");
    }
}

#[test]
fn the_banana_fields_are_in_the_content_hash() {
    // Sim 117 (E44, E45, E47, E48): a move's shed, a hitbox's slip, a move's toss, an item's damage modifier, uses and trap, and the
    // ruleset's soft-trip timings each change the hash.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[11].weapon);
    let edits: [fn(&mut sim_core::Content, usize); 11] = [
        |c, w| c.weapons[w].moves[MoveId::UpSpecial as usize].shed = None,
        |c, _| {
            c.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].slip = sim_core::moves::SLIP_SOFT
        },
        |c, w| {
            if let Some(t) = c.weapons[w].moves[MoveId::DownSpecial as usize]
                .toss
                .as_mut()
            {
                t.angle = 100;
            }
        },
        |c, w| {
            if let Some(t) = c.weapons[w].moves[MoveId::DownSpecial as usize]
                .toss
                .as_mut()
            {
                t.wait = 60;
            }
        },
        |c, w| {
            if let Some(t) = c.weapons[w].moves[MoveId::DownSpecial as usize]
                .toss
                .as_mut()
            {
                t.smash_speed = Fx::ONE;
            }
        },
        |c, w| c.weapons[w].items[0].damage_add = Fx::ZERO,
        |c, w| c.weapons[w].items[0].uses = 3,
        |c, w| c.weapons[w].items[0].trap = None,
        |c, w| {
            if let Some(t) = c.weapons[w].items[0].trap.as_mut() {
                t.kick_x = Fx::ZERO;
            }
        },
        |c, _| c.rules.slip_lag = 29,
        |c, _| c.rules.slip_max = 89,
    ];
    for (k, edit) in edits.iter().enumerate() {
        let mut c = base.clone();
        edit(&mut c, w);
        assert_ne!(c.hash(), h, "edit {k}");
    }
}

#[test]
fn the_latch_and_flag_fields_are_in_the_content_hash() {
    // Sim 119 (E49, owner decision 75): a move's latch, flag versions and self damage, a hitbox's fixed stun and unblockable flag
    // each change the hash.
    use sim_core::moves::popgun_slots::LATCH;
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[11].weapon);
    let edits: [fn(&mut sim_core::Content, usize); 9] = [
        |c, w| c.weapons[w].moves[LATCH as usize].latch = None,
        |c, w| {
            if let Some(l) = c.weapons[w].moves[LATCH as usize].latch.as_mut() {
                l.height = 50;
            }
        },
        |c, w| {
            if let Some(l) = c.weapons[w].moves[LATCH as usize].latch.as_mut() {
                l.clock_den = 2;
            }
        },
        |c, w| {
            if let Some(l) = c.weapons[w].moves[LATCH as usize].latch.as_mut() {
                l.x = Fx::ONE;
            }
        },
        |c, w| {
            c.weapons[w].moves[MoveId::UpSpecial as usize]
                .flag_versions
                .clear()
        },
        |c, w| c.weapons[w].moves[MoveId::UpSpecial as usize].flag_versions[0].slot = Some(0),
        |c, _| c.weapons[0].moves[MoveId::Jab as usize].self_damage = Fx::ONE,
        |c, _| c.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].fixed_stun = 20,
        |c, _| c.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].unblockable = true,
    ];
    for (k, edit) in edits.iter().enumerate() {
        let mut c = base.clone();
        edit(&mut c, w);
        assert_ne!(c.hash(), h, "edit {k}");
    }
}

/// Two duelists face to face (player 0 at 0, player 1 at `gap` facing it), the others out.
fn facing_pair(gap: Fx) -> Sim {
    let mut sim = Sim::with_chars([0, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    sim.stand(0, Fx::ZERO, 1);
    sim.stand(1, gap, -1);
    sim
}

#[test]
fn an_unblockable_hit_lands_through_a_raised_shield() {
    // Sim 119: `Hitbox::unblockable` (the reference's rocket barrels): the shield does not stop it; without it the shield does.
    use sim_core::input::buttons::{ATTACK, SHIELD};
    for unblockable in [false, true] {
        let mut sim = facing_pair(Fx::from_int(1));
        for h in &mut sim.content.weapons[0].moves[MoveId::Jab as usize].hitboxes {
            h.unblockable = unblockable;
        }
        for _ in 0..10 {
            sim.tick2(inp(0, 0, 0), inp(0, 0, SHIELD));
        }
        assert_eq!(sim.fighter(1).state, S::Shield);
        let hp = sim.fighter(1).shield_hp;
        for frame in 0..12 {
            sim.tick2(
                inp(0, 0, if frame == 0 { ATTACK } else { 0 }),
                inp(0, 0, SHIELD),
            );
        }
        assert_eq!(sim.fighter(1).percent > Fx::ZERO, unblockable);
        assert_eq!(sim.fighter(1).state == S::Hitstun, unblockable);
        // (Blocked, the shield took the hit: more than it wears down on its own in those frames.)
        let worn = hp - sim.fighter(1).shield_hp;
        assert_eq!(worn > Fx::ONE, !unblockable, "{worn:?}");
    }
}

#[test]
fn a_fixed_stun_hit_holds_a_victim_it_does_not_tumble_exactly_that_long() {
    // Sim 119: `Hitbox::fixed_stun` (the reference's flip jump-off, 20 frames): a weak hit's hitstun becomes exactly that; a
    // tumbling one keeps its own.
    use sim_core::input::buttons::ATTACK;
    let stun_of = |fixed: u8, growth: i16| {
        let mut sim = facing_pair(Fx::from_int(1));
        for h in &mut sim.content.weapons[0].moves[MoveId::Jab as usize].hitboxes {
            h.fixed_stun = fixed;
            h.knockback_growth = growth;
        }
        let mut stun = 0;
        for frame in 0..14 {
            sim.tick(inp(0, 0, if frame == 0 { ATTACK } else { 0 }));
            if stun == 0 && sim.fighter(1).state == S::Hitstun {
                stun = sim.fighter(1).hitstun;
            }
        }
        stun
    };
    assert_eq!(stun_of(20, 0), 20);
    assert_ne!(stun_of(0, 0), 20);
    // (A tumbling launch is not held to it.)
    assert!(stun_of(20, 400) > 20);
}

#[test]
fn self_damage_hurts_the_user_on_the_moves_first_frame() {
    // Sim 119: `Move::self_damage` (the reference's rocket barrels crashing).
    let mut sim = with_move(
        MoveId::NSpecial,
        Move {
            total_frames: 20,
            self_damage: Fx::from_int(5),
            script: Some(
                sim_script::Program::compile(sim_script::Kind::Fighter, "var a;").unwrap(),
            ),
            ..Move::empty()
        },
    );
    sim.tick(inp(0, 0, SPECIAL));
    assert_eq!(sim.f().percent, Fx::ZERO);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().percent, Fx::from_int(5));
    sim.ticks(5, inp(0, 0, 0));
    assert_eq!(sim.f().percent, Fx::from_int(5), "once");
}

#[test]
fn a_flag_version_picks_its_move_or_bars_the_special_and_reads_marks_and_flags() {
    // Sim 119 (owner decision 75): `Move::flag_versions`: with a flag (or a mark) set, the special starts as the version's move, or
    // not at all when it names none; with neither set, as itself.
    use sim_core::moves::FlagVersion;
    let mut sim = with_move(
        MoveId::NSpecial,
        Move {
            total_frames: 20,
            script: Some(
                sim_script::Program::compile(sim_script::Kind::Fighter, "var a;").unwrap(),
            ),
            flag_versions: vec![
                FlagVersion {
                    flags: 0b10,
                    marks: 0,
                    slot: None,
                },
                FlagVersion {
                    flags: 0,
                    marks: 0b1,
                    slot: Some(MoveId::Ext0 as u8),
                },
            ],
            ..Move::empty()
        },
    );
    sim.content.weapons[0].moves[MoveId::Ext0 as usize] = Move {
        total_frames: 25,
        script: Some(sim_script::Program::compile(sim_script::Kind::Fighter, "var b;").unwrap()),
        ..Move::empty()
    };
    let start = |flags: u8, marks: u8| {
        let mut s = Sim::alone();
        s.content = sim.content.clone();
        s.put_airborne(0, Fx::ZERO, Fx::from_int(20), Fx::ZERO, Fx::ZERO);
        s.state.fighters[0].air_flags = flags;
        s.state.fighters[0].air_marks = marks;
        s.tick(inp(0, 0, SPECIAL));
        (s.f().state, s.f().move_id)
    };
    assert_eq!(start(0, 0), (S::Attack, MoveId::NSpecial as u8));
    assert_eq!(start(0, 1), (S::Attack, MoveId::Ext0 as u8));
    assert_eq!(start(0b10, 1), (S::Airborne, 0), "barred: nothing starts");
    assert_eq!(
        start(0b1, 0),
        (S::Attack, MoveId::NSpecial as u8),
        "another flag"
    );
}

#[test]
fn the_partner_guard_and_curse_fields_are_in_the_content_hash() {
    // Sim 121: a hitbox's mode and part (E52), a move's armor damage and hitlag (E55), gauge gain (E53) and curse (E54), a burst's
    // `on_shield`, and the gauge's partner rules (E53) each change the hash.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let edits: [fn(&mut sim_core::Content); 15] = [
        |c| c.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].mode = 2,
        |c| c.weapons[0].moves[MoveId::Jab as usize].hitboxes[0].part = 1,
        |c| c.weapons[0].moves[MoveId::Jab as usize].armor_damage = 400,
        |c| c.weapons[0].moves[MoveId::Jab as usize].armor_hitlag = 150,
        |c| c.weapons[0].moves[MoveId::Jab as usize].gauge_gain = 2000,
        |c| {
            c.weapons[0].moves[MoveId::Jab as usize].curse = Some(sim_core::moves::Curse {
                damage: Fx::ONE,
                every: 45,
                frames: 361,
            })
        },
        |c| {
            let w = usize::from(c.fighters[12].weapon);
            if let Some(b) = c.weapons[w].moves[MoveId::SideSpecial as usize]
                .projectile
                .as_mut()
                .and_then(|p| p.burst.as_mut())
            {
                b.on_shield = true;
            }
        },
        |c| c.fighters[12].gauge.per_frame = Fx::ZERO,
        |c| c.fighters[12].gauge.behind[0] = 1000,
        |c| c.fighters[12].gauge.crowd[1][2] = 1000,
        |c| c.fighters[12].gauge.mode_taken[0] = Fx::ONE,
        |c| c.fighters[12].gauge.ko_keep[0] = Fx::ZERO,
        |c| c.fighters[12].gauge.keeps = false,
        |c| c.fighters[12].gauge.halts = false,
        |c| c.fighters[12].gauge.crowd[0][1] = 1000,
    ];
    for (k, edit) in edits.iter().enumerate() {
        let mut c = base.clone();
        edit(&mut c);
        assert_ne!(c.hash(), h, "edit {k}");
    }
}

#[test]
fn the_tether_and_dagger_phase_2_fields_are_in_the_content_hash() {
    // Sim 123: a move's tether (E56) and each of its numbers, a dodge that stales like a roll, a blockable grab box, a part
    // intangibility window's mode.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let w = usize::from(base.fighters[12].weapon);
    let air = sim_core::moves::dagger_slots::HOOK_AIR as usize;
    type Edit = Box<dyn Fn(&mut sim_core::Content)>;
    let tether = |f: fn(&mut sim_core::moves::Tether)| -> Edit {
        Box::new(move |c| f(c.weapons[w].moves[air].tether.as_mut().expect("a tether")))
    };
    let edits: Vec<Edit> = vec![
        Box::new(move |c| c.weapons[w].moves[air].tether = None),
        tether(|t| t.from = 1),
        tether(|t| t.to = 30),
        tether(|t| t.ahead = Fx::ONE),
        tether(|t| t.behind = Fx::ONE),
        tether(|t| t.up = Fx::ONE),
        tether(|t| t.down = Fx::ONE),
        tether(|t| t.hang = 1),
        tether(|t| t.reel = 1),
        tether(|t| t.drag = 1),
        tether(|t| t.limit = 1),
        Box::new(|c| c.weapons[0].moves[MoveId::Jab as usize].dodge_stale = true),
        Box::new(|c| c.weapons[0].moves[MoveId::Grab as usize].hitboxes[0].blockable = true),
        Box::new(move |c| c.weapons[w].moves[MoveId::USmash as usize].part_intangible[0].mode = 0),
    ];
    for (k, edit) in edits.iter().enumerate() {
        let mut c = base.clone();
        edit(&mut c);
        assert_ne!(c.hash(), h, "edit {k}");
    }
}

#[test]
fn a_blockable_grab_box_is_stopped_by_a_reflector_field() {
    // Sim 123 (E56): the phantom's grounded hook against a reflector field held at the hook's head: nothing is caught.
    let mut sim = common::Sim::with_chars([12, 0, 0, 0]);
    sim.state.fighters[2].active = false;
    sim.state.fighters[3].active = false;
    let w = usize::from(sim.content.fighters[12].weapon);
    // A reflector made for the test on the duelist's neutral air: a big field round its body, up for the whole move.
    let dw = usize::from(sim.content.fighters[0].weapon);
    sim.content.weapons[dw].moves[MoveId::NAir as usize].reflector =
        Some(sim_core::moves::Reflector {
            start: 0,
            end: 60,
            radius: Fx::from_int(3),
            y: Fx::from_int(1),
            ..sim_core::moves::Reflector::NONE
        });
    sim.content.weapons[dw].moves[MoveId::NAir as usize].total_frames = 60;
    sim.content.weapons[dw].moves[MoveId::NAir as usize]
        .hitboxes
        .clear();
    sim.content.weapons[dw].moves[MoveId::NAir as usize].next = None;
    let head = sim.content.weapons[w].moves[MoveId::UpSpecial as usize]
        .hitboxes
        .iter()
        .map(|h| h.y)
        .max()
        .expect("boxes");
    for reflecting in [true, false] {
        let mut s = sim.state;
        let at = sim_core::Vec2::new(common::fx(13, 10), head - Fx::from_int(1));
        s.fighters[0].pos = sim_core::Vec2::ZERO;
        s.fighters[0].facing = 1;
        s.fighters[0].platform = 0;
        sim_core::fighter::begin_attack(&mut s.fighters[0], MoveId::UpSpecial);
        let mut caught = false;
        for _ in 0..30 {
            let f = &mut s.fighters[1];
            f.pos = at;
            f.vel = sim_core::Vec2::ZERO;
            f.platform = sim_core::state::NONE;
            if reflecting {
                if f.state != sim_core::state::FighterState::Attack
                    && f.state != sim_core::state::FighterState::Grabbed
                {
                    sim_core::fighter::begin_attack(f, MoveId::NAir);
                }
            } else if f.state != sim_core::state::FighterState::Grabbed {
                f.state = sim_core::state::FighterState::Airborne;
            }
            sim_core::step(
                &mut s,
                &sim.content,
                &[sim_core::Input::default(); sim_core::MAX_FIGHTERS],
            );
            caught |= s.fighters[1].state == sim_core::state::FighterState::Grabbed;
        }
        assert_eq!(caught, !reflecting, "reflecting {reflecting}");
    }
}

#[test]
fn a_powered_box_is_out_only_in_an_attack_begun_with_the_mode_and_a_plain_one_only_without() {
    // Sim 121 (E52): `Hitbox::mode`. The duelist's jab with its first box made plain and a copy of it (9%) made powered.
    let mut jab = sim_core::Content::placeholder().weapons[0].moves[MoveId::Jab as usize].clone();
    let plain = jab.hitboxes[0];
    let mut powered = plain;
    powered.mode = sim_core::moves::MODE_POWERED;
    powered.damage = Fx::from_int(9);
    jab.hitboxes = vec![plain, powered];
    jab.hitboxes[0].mode = sim_core::moves::MODE_PLAIN;
    for (mode, expect) in [(0u16, plain.damage), (60, Fx::from_int(9))] {
        let mut sim = with_move(MoveId::Jab, jab.clone());
        sim.stand(0, Fx::ZERO, 1);
        sim.state.fighters[0].gauge_mode = mode;
        sim_core::fighter::begin_attack(&mut sim.state.fighters[0], MoveId::Jab);
        sim.state.fighters[0].state_frame = u16::from(plain.start);
        let out: Vec<_> = sim_core::combat::active_hitboxes(sim.f(), &jab, Fx::ONE)
            .map(|(_, h, _)| h.damage)
            .collect();
        assert_eq!(out, vec![expect], "mode {mode}");
    }
}

#[test]
fn a_projectiles_curse_deals_its_damage_every_few_frames_and_ends() {
    // Sim 121 (E54): a curse on a fighter deals `damage` (times the one-on-one multiplier) every `every` frames for `frames`.
    let mut sim = Sim::alone();
    let k = sim.content.rules.damage_mult;
    let f = &mut sim.state.fighters[1];
    f.curse_left = 100;
    f.curse_every = 30;
    f.curse_damage = Fx::from_int(2);
    f.curse_by = 0;
    sim.ticks(100, inp(0, 0, 0));
    assert_eq!(
        sim.fighter(1).percent,
        Fx::from_int(6) * k,
        "ticks on 30, 60 and 90"
    );
    assert_eq!(sim.fighter(1).curse_left, 0);
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.fighter(1).percent, Fx::from_int(6) * k, "and no more");
}

#[test]
fn the_wing_armor_mode_and_edge_stop_are_in_the_content_hash() {
    // Sim 125: the exile's wing (E57) and each of its numbers, a move's armor mode, a motion that stops at the edge.
    let base = sim_core::Content::placeholder();
    let h = base.hash();
    let exile = 13;
    type Edit = Box<dyn Fn(&mut sim_core::Content)>;
    let wing = |f: fn(&mut sim_core::content::Wing)| -> Edit {
        Box::new(move |c| f(c.fighters[exile].wing.as_mut().expect("a wing")))
    };
    let edits: Vec<Edit> = vec![
        Box::new(move |c| c.fighters[exile].wing = None),
        wing(|w| w.points[0] = Fx::ONE),
        wing(|w| w.per_dealt[1] = Fx::ONE),
        wing(|w| w.ko_points = Fx::ONE),
        wing(|w| w.power = 101),
        wing(|w| w.jumps = 2),
        wing(|w| w.walk = 101),
        wing(|w| w.run_accel = 101),
        wing(|w| w.air_accel_stick = 101),
        wing(|w| w.fall = 101),
        wing(|w| w.rows[4].diff[0] = 121),
        wing(|w| w.rows[2].percent[1] = Fx::ONE),
        wing(|w| w.rows[3].count = 2),
        Box::new(|c| c.fighters[0].wing = Some(sim_core::FighterParams::exile_wing())),
        Box::new(|c| c.weapons[0].moves[MoveId::Jab as usize].armor_mode = 2),
        Box::new(|c| c.weapons[0].moves[MoveId::Jab as usize].edge_stop = true),
    ];
    for (k, edit) in edits.iter().enumerate() {
        let mut c = base.clone();
        edit(&mut c);
        assert_ne!(c.hash(), h, "edit {k}");
    }
}

#[test]
fn phases_sharing_a_start_are_rings_out_together_and_distinct_starts_one_at_a_time() {
    // Sim 125 (E58): hits by age with the same start are all out at once, in their order; different starts replace one another as
    // before (every older projectile).
    let mut a = hit(Fx::ONE, Fx::ZERO);
    a.priority = 0;
    let mut b = hit(Fx::from_int(2), Fx::ZERO);
    b.priority = 1;
    let mut c = hit(Fx::from_int(3), Fx::ZERO);
    c.priority = 2;
    let phases = sim_core::moves::phase_list(&[(2, a), (5, b), (5, c)]);
    let at = |age| {
        sim_core::moves::rings_at(&phases, age)
            .iter()
            .flatten()
            .map(|h| h.radius)
            .collect::<Vec<_>>()
    };
    assert!(at(1).is_empty());
    assert_eq!(at(3), vec![Fx::ONE]);
    assert_eq!(at(5), vec![Fx::from_int(2), Fx::from_int(3)]);
    assert_eq!(MAX_PHASES, 12);
}
