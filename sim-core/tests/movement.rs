//! Phase 1 exit criteria: movement tech works from scripted inputs, for fighters with different
//! physics profiles. Each test drives player 0 with a recorded input script.

use sim_core::input::buttons::{JUMP, SHIELD};
use sim_core::state::{Fighter, FighterState as S, NONE};
use sim_core::{step, Content, Fx, GameState, Input, MAX_FIGHTERS};

fn inp(x: i8, y: i8, buttons: u16) -> Input {
    Input {
        stick_x: x,
        stick_y: y,
        buttons,
    }
}

struct Sim {
    content: Content,
    state: GameState,
}

impl Sim {
    fn new() -> Sim {
        Sim::with_chars([0, 1, 0, 1])
    }

    fn with_chars(chars: [u8; MAX_FIGHTERS]) -> Sim {
        let mut content = Content::placeholder();
        // Tall blast zone so tests can hold fighters high in the air without being KO.d.
        content.stage.blast_top = Fx::from_int(5000);
        let state = GameState::new(&content, 1, chars);
        Sim { content, state }
    }

    fn tick(&mut self, p0: Input) {
        let mut inputs = [Input::default(); MAX_FIGHTERS];
        inputs[0] = p0;
        step(&mut self.state, &self.content, &inputs);
    }

    fn ticks(&mut self, n: usize, p0: Input) {
        for _ in 0..n {
            self.tick(p0);
        }
    }

    fn f(&self) -> &Fighter {
        &self.state.fighters[0]
    }

    /// Puts a fighter in the air at a position with a velocity.
    fn put_airborne(&mut self, i: usize, x: i32, y: i32, vx: Fx, vy: Fx) {
        let f = &mut self.state.fighters[i];
        f.pos.x = Fx::from_int(x);
        f.pos.y = Fx::from_int(y);
        f.vel.x = vx;
        f.vel.y = vy;
        f.platform = NONE;
        f.state = S::Airborne;
        f.state_frame = 0;
    }

    /// Puts fighter 0 into a ledge hang on the left ledge via a real grab.
    fn hang_left_ledge(&mut self) {
        self.put_airborne(0, -12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
        self.tick(Input::default());
        assert_eq!(self.f().state, S::LedgeHang);
    }
}

/// Jump, then press shield during jump squat with the stick held at (x, y). Returns the sim 5 frames in.
fn wavedash(x: i8, y: i8) -> Sim {
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP));
    sim.tick(inp(x, y, SHIELD));
    sim.ticks(3, inp(x, y, SHIELD));
    sim
}

fn slide_distance(x: i8, y: i8) -> Fx {
    let mut sim = wavedash(x, y);
    let start = sim.state.fighters[0].pos.x;
    assert_eq!(
        sim.f().state,
        S::WaveLand,
        "stick ({x},{y}) should wavedash"
    );
    sim.ticks(30, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    sim.f().pos.x - start
}

// ---- Ground and air basics --------------------------------------------------------------------

#[test]
fn run_then_stop() {
    let mut sim = Sim::new();
    let run_speed = sim.content.fighters[0].run_speed;
    sim.ticks(40, inp(127, 0, 0));
    assert_eq!(sim.f().state, S::Run);
    assert_eq!(sim.f().vel.x, run_speed);
    assert_eq!(sim.f().facing, 1);
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    assert_eq!(sim.f().vel.x, Fx::ZERO);
}

#[test]
fn walking_off_the_stage_edge_makes_you_airborne() {
    let mut sim = Sim::new();
    sim.state.fighters[0].pos.x = Fx::from_int(10);
    sim.ticks(40, inp(127, 0, 0));
    assert!(!sim.f().grounded());
    assert!(matches!(sim.f().state, S::Airborne | S::LedgeHang));
}

#[test]
fn hold_jump_for_a_full_hop_tap_for_a_short_hop() {
    let peak = |held: bool| {
        let mut sim = Sim::new();
        let mut peak = Fx::ZERO;
        for t in 0..60 {
            let buttons = if held || t == 0 { JUMP } else { 0 };
            sim.tick(inp(0, 0, buttons));
            peak = peak.max(sim.f().pos.y);
        }
        peak
    };
    let (full, short) = (peak(true), peak(false));
    assert!(
        full > short,
        "full hop {full:?} should beat short hop {short:?}"
    );
    assert!(short > Fx::ZERO);
}

#[test]
fn jump_squat_delays_takeoff() {
    let mut sim = Sim::new();
    let squat = sim.content.fighters[0].jump_squat_frames;
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
    sim.ticks(usize::from(squat) - 1, inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::Airborne);
}

#[test]
fn fighters_with_different_profiles_move_differently() {
    let mut a = Sim::with_chars([0, 1, 0, 1]);
    let mut b = Sim::with_chars([1, 0, 1, 0]);
    let (mut peak_a, mut peak_b) = (Fx::ZERO, Fx::ZERO);
    for t in 0..80 {
        let i = inp(127, 0, JUMP);
        let i = if t > 0 { inp(127, 0, JUMP) } else { i };
        a.tick(i);
        b.tick(i);
        peak_a = peak_a.max(a.f().pos.y);
        peak_b = peak_b.max(b.f().pos.y);
    }
    assert_ne!(peak_a, peak_b, "jump height should differ between profiles");
    assert_ne!(
        a.f().pos.x,
        b.f().pos.x,
        "run speed should differ between profiles"
    );
}

#[test]
fn air_jumps_are_limited_and_refresh_on_landing() {
    let mut sim = Sim::new();
    sim.put_airborne(0, 0, 20, Fx::ZERO, Fx::ZERO);
    assert_eq!(sim.f().air_jumps_left, 1);
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().air_jumps_left, 0);
    sim.tick(inp(0, 0, 0));
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().air_jumps_left, 0, "no third jump");
    sim.ticks(200, inp(0, 0, 0));
    assert!(sim.f().grounded());
    assert_eq!(sim.f().air_jumps_left, 1);
}

#[test]
fn fast_fall_reaches_higher_terminal_speed() {
    let mut sim = Sim::new();
    sim.put_airborne(0, 0, 40, Fx::ZERO, Fx::ZERO);
    sim.ticks(60, inp(0, 0, 0));
    assert_eq!(sim.f().vel.y, -sim.content.fighters[0].max_fall_speed);
    sim.ticks(20, inp(0, -127, 0));
    assert!(sim.f().fast_fall);
    assert_eq!(sim.f().vel.y, -sim.content.fighters[0].fast_fall_speed);
}

// ---- Wavedash --------------------------------------------------------------------------------

#[test]
fn wavedash_converts_jump_plus_downward_air_dodge_into_a_ground_slide() {
    let mut sim = wavedash(100, -80);
    assert_eq!(sim.f().state, S::WaveLand);
    assert!(sim.f().grounded());
    assert!(!sim.f().air_dodge_used, "a waveland counts as landing");
    assert!(sim.f().vel.x > Fx::ZERO);
    let start = Fx::from_int(-4);
    sim.ticks(30, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
    assert!(
        sim.f().pos.x - start > Fx::ONE,
        "slid {:?}",
        sim.f().pos.x - start
    );
    assert_eq!(sim.f().pos.y, Fx::ZERO);
}

#[test]
fn wavedash_works_to_the_left_too() {
    let mut sim = wavedash(-100, -80);
    assert_eq!(sim.f().state, S::WaveLand);
    assert!(sim.f().vel.x < Fx::ZERO);
    sim.ticks(30, inp(0, 0, 0));
    assert!(sim.f().pos.x < Fx::from_int(-5));
}

#[test]
fn wavedash_buffer_accepts_shield_pressed_early_in_jump_squat() {
    // Shield pressed on the very first squat frame, still inside the buffer at takeoff.
    let mut sim = Sim::new();
    sim.tick(inp(0, 0, JUMP | SHIELD));
    sim.ticks(4, inp(100, -80, SHIELD));
    assert_eq!(sim.f().state, S::WaveLand);
}

#[test]
fn wavedash_distance_grows_smoothly_with_how_horizontal_the_stick_is() {
    let distances: Vec<Fx> = [20, 40, 60, 80, 100, 127]
        .iter()
        .map(|&x| slide_distance(x, -100))
        .collect();
    for pair in distances.windows(2) {
        assert!(pair[1] > pair[0], "not increasing: {distances:?}");
        assert!(
            pair[1] - pair[0] < Fx::HALF,
            "cliff between neighbours: {distances:?}"
        );
    }
}

#[test]
fn wavedash_cone_is_wide() {
    // Everything from nearly-horizontal-but-down to straight down still works.
    for (x, y) in [(127, -40), (127, -60), (90, -90), (40, -120), (0, -127)] {
        let sim = wavedash(x, y);
        assert_eq!(sim.f().state, S::WaveLand, "stick ({x},{y})");
    }
}

#[test]
fn straight_down_wavedash_does_not_slide() {
    let mut sim = wavedash(0, -127);
    assert_eq!(sim.f().state, S::WaveLand);
    let x = sim.f().pos.x;
    sim.ticks(30, inp(0, 0, 0));
    assert_eq!(sim.f().pos.x, x);
}

#[test]
fn too_horizontal_dodge_fails_gracefully_into_a_plain_air_dodge() {
    let mut sim = wavedash(127, -10);
    assert!(
        matches!(sim.f().state, S::AirDodge | S::Landing),
        "{:?}",
        sim.f().state
    );
    for _ in 0..40 {
        sim.tick(inp(127, -10, 0));
        assert_ne!(sim.f().state, S::WaveLand);
    }
}

#[test]
fn ground_assist_does_not_trigger_high_in_the_air() {
    let mut sim = Sim::new();
    sim.put_airborne(0, 0, 8, Fx::ZERO, Fx::ZERO);
    sim.tick(inp(100, -80, SHIELD));
    assert_eq!(sim.f().state, S::AirDodge);
    assert!(!sim.f().grounded());
}

#[test]
fn only_one_air_dodge_per_airtime() {
    let mut sim = Sim::new();
    sim.put_airborne(0, 0, 30, Fx::ZERO, Fx::ZERO);
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::AirDodge);
    sim.ticks(40, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Airborne);
    sim.tick(inp(0, 0, SHIELD));
    assert_ne!(
        sim.f().state,
        S::AirDodge,
        "second air dodge in one airtime"
    );
}

// ---- Shield drop -----------------------------------------------------------------------------

fn on_pass_through_platform() -> Sim {
    let mut sim = Sim::new();
    let f = &mut sim.state.fighters[0];
    f.pos.x = Fx::from_int(-5);
    f.pos.y = Fx::from_ratio(18, 5);
    f.platform = 1;
    sim
}

#[test]
fn shield_plus_down_drops_through_a_pass_through_platform() {
    let mut sim = on_pass_through_platform();
    sim.tick(inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    sim.tick(inp(0, -100, SHIELD));
    assert_eq!(sim.f().state, S::ShieldDrop);
    assert!(!sim.f().grounded());
    sim.ticks(80, inp(0, 0, 0));
    assert_eq!(sim.f().platform, 0, "should end up on the main stage");
    assert_eq!(sim.f().pos.y, Fx::ZERO);
}

#[test]
fn shield_drop_does_nothing_on_solid_ground() {
    let mut sim = Sim::new();
    sim.ticks(20, inp(0, -127, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    assert_eq!(sim.f().pos.y, Fx::ZERO);
}

#[test]
fn jump_out_of_shield_unless_pressing_down_on_a_platform() {
    let mut sim = Sim::new();
    sim.ticks(3, inp(0, 0, SHIELD));
    sim.tick(inp(0, 0, SHIELD | JUMP));
    assert_eq!(sim.f().state, S::JumpSquat);

    let mut sim = on_pass_through_platform();
    sim.ticks(3, inp(0, 0, SHIELD));
    sim.tick(inp(0, -100, SHIELD | JUMP));
    assert_eq!(sim.f().state, S::ShieldDrop, "down wins over jump");
}

#[test]
fn releasing_shield_returns_to_idle() {
    let mut sim = Sim::new();
    sim.ticks(5, inp(0, 0, SHIELD));
    assert_eq!(sim.f().state, S::Shield);
    sim.tick(inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Idle);
}

// ---- Platforms -------------------------------------------------------------------------------

#[test]
fn pass_through_platforms_can_be_jumped_up_through_and_landed_on() {
    let mut sim = Sim::new();
    sim.put_airborne(0, -5, 1, Fx::ZERO, Fx::from_ratio(1, 4));
    sim.ticks(120, inp(0, 0, 0));
    assert_eq!(sim.f().platform, 1);
    assert_eq!(sim.f().pos.y, Fx::from_ratio(18, 5));
}

// ---- Ledges ----------------------------------------------------------------------------------

#[test]
fn falling_next_to_a_ledge_grabs_it() {
    let mut sim = Sim::new();
    sim.hang_left_ledge();
    let base = sim.content.fighters[0].ledge_invuln_base;
    assert_eq!(sim.state.ledge_owner[0], 0);
    assert_eq!(sim.f().ledge_invuln, base);
    assert_eq!(sim.f().facing, 1, "faces the stage");
    assert_eq!(sim.f().ledge_grab_count, 1);
}

#[test]
fn a_far_away_fighter_does_not_grab() {
    let mut sim = Sim::new();
    sim.put_airborne(0, -25, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.ticks(5, Input::default());
    assert_eq!(sim.f().state, S::Airborne);
    assert_eq!(sim.state.ledge_owner[0], NONE);
}

#[test]
fn grabbing_an_occupied_ledge_trumps_the_occupant() {
    let mut sim = Sim::new();
    sim.hang_left_ledge();
    sim.put_airborne(1, -12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.tick(Input::default());
    assert_eq!(sim.state.fighters[1].state, S::LedgeHang);
    assert_eq!(sim.state.ledge_owner[0], 1);
    let loser = sim.f();
    assert_eq!(loser.state, S::Airborne);
    assert_eq!(loser.ledge, NONE);
    assert!(loser.ledge_cooldown > 0);
    assert!(loser.vel.x < Fx::ZERO, "knocked away from the stage");
}

#[test]
fn simultaneous_grabs_resolve_by_player_index() {
    let mut sim = Sim::new();
    sim.put_airborne(0, -12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.put_airborne(1, -12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.tick(Input::default());
    assert_eq!(sim.state.fighters[0].state, S::LedgeHang);
    assert_eq!(sim.state.fighters[1].state, S::Airborne);
    assert_eq!(sim.state.ledge_owner[0], 0);

    // Reversed roles: only players 2 and 3 compete, so 2 wins.
    let mut sim = Sim::new();
    sim.put_airborne(3, -12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.put_airborne(2, -12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.tick(Input::default());
    assert_eq!(sim.state.ledge_owner[0], 2);
}

#[test]
fn ledge_invincibility_diminishes_with_repeated_grabs_and_resets_on_ground() {
    let mut sim = Sim::new();
    let p = sim.content.fighters[0];
    sim.hang_left_ledge();
    assert_eq!(sim.f().ledge_invuln, p.ledge_invuln_base);

    // Drop, wait out the regrab cooldown, grab again.
    sim.tick(inp(0, -127, 0));
    assert_eq!(sim.f().state, S::Airborne);
    sim.put_airborne(0, -25, 0, Fx::ZERO, Fx::ZERO);
    sim.ticks(usize::from(p.ledge_regrab_cooldown) + 2, Input::default());
    sim.hang_left_ledge();
    assert_eq!(
        sim.f().ledge_invuln,
        p.ledge_invuln_base - p.ledge_invuln_decay
    );
    assert_eq!(sim.f().ledge_grab_count, 2);

    // Never below the floor.
    for _ in 0..20 {
        sim.tick(inp(0, -127, 0));
        sim.put_airborne(0, -25, 0, Fx::ZERO, Fx::ZERO);
        sim.ticks(usize::from(p.ledge_regrab_cooldown) + 2, Input::default());
        sim.hang_left_ledge();
    }
    assert_eq!(sim.f().ledge_invuln, p.ledge_invuln_floor);

    // Getting up onto the stage resets the count.
    sim.tick(inp(0, 127, 0));
    assert_eq!(sim.f().state, S::LedgeGetUp);
    sim.ticks(usize::from(p.ledge_getup_frames) + 1, Input::default());
    assert_eq!(sim.f().state, S::Idle);
    assert_eq!(sim.f().ledge_grab_count, 0);
}

#[test]
fn ledge_get_up_lands_on_the_stage() {
    let mut sim = Sim::new();
    sim.hang_left_ledge();
    sim.tick(inp(0, 127, 0));
    assert_eq!(sim.f().state, S::LedgeGetUp);
    assert_eq!(sim.f().platform, 0);
    assert_eq!(sim.f().pos.y, Fx::ZERO);
    assert!(sim.f().pos.x > Fx::from_int(-11));
    assert_eq!(sim.state.ledge_owner[0], NONE);
}

#[test]
fn ledge_roll_goes_farther_than_get_up() {
    let landing_x = |button: u16, y: i8| {
        let mut sim = Sim::new();
        sim.hang_left_ledge();
        sim.tick(inp(0, y, button));
        sim.f().pos.x
    };
    assert!(landing_x(SHIELD, 0) > landing_x(0, 127));
}

#[test]
fn ledge_jump_launches_up_and_toward_the_stage() {
    let mut sim = Sim::new();
    sim.hang_left_ledge();
    sim.tick(inp(0, 0, JUMP));
    assert_eq!(sim.f().state, S::Airborne);
    assert!(sim.f().vel.y > Fx::ZERO);
    assert!(sim.f().vel.x > Fx::ZERO);
}

#[test]
fn dropping_from_a_ledge_blocks_an_instant_regrab() {
    let mut sim = Sim::new();
    sim.hang_left_ledge();
    sim.tick(inp(-127, 0, 0));
    assert_eq!(sim.f().state, S::Airborne);
    sim.ticks(10, inp(0, 0, 0));
    assert_eq!(sim.f().state, S::Airborne);
}

#[test]
fn hang_time_is_capped() {
    let mut sim = Sim::new();
    let cap = usize::from(sim.content.fighters[0].ledge_hang_max);
    sim.hang_left_ledge();
    sim.ticks(cap - 2, Input::default());
    assert_eq!(sim.f().state, S::LedgeHang);
    sim.ticks(4, Input::default());
    assert_ne!(sim.f().state, S::LedgeHang);
    assert_eq!(sim.state.ledge_owner[0], NONE);
}

#[test]
fn right_ledge_mirrors_the_left() {
    let mut sim = Sim::new();
    sim.put_airborne(0, 12, -1, Fx::ZERO, Fx::from_ratio(-1, 20));
    sim.tick(Input::default());
    assert_eq!(sim.f().state, S::LedgeHang);
    assert_eq!(sim.state.ledge_owner[1], 0);
    assert_eq!(sim.f().facing, -1);
    sim.tick(inp(0, 127, 0));
    assert!(sim.f().pos.x < Fx::from_int(11));
    assert_eq!(sim.f().platform, 0);
}

// ---- Determinism of the new systems ----------------------------------------------------------

#[test]
fn scripted_wavedash_is_bit_identical_across_runs() {
    let a = wavedash(100, -80);
    let b = wavedash(100, -80);
    assert_eq!(a.state.checksum(), b.state.checksum());
}
