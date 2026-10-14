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
