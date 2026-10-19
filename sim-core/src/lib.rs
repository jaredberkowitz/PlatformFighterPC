//! Deterministic simulation core.
//!
//! Rules for everything in this crate (and any crate that opts into the workspace lints):
//! * No floating point. Use [`fixed::Fx`] (16.16) and the lookup-table trig in [`trig`].
//! * No `HashMap`/`HashSet`. Use arrays, slot pools or `BTreeMap`.
//! * No clocks, threads, I/O or global state. All randomness comes from [`rng::Rng`] stored in state.
//! * Everything that changes lives in [`state::GameState`] (a run's too: [`run::Run`]); content is immutable and hashed.

pub mod balloons;
pub mod collision;
pub mod combat;
pub mod content;
pub mod cpu;
pub mod fighter;
pub mod fixed;
pub mod fuzz;
pub mod gauge;
pub mod grab;
pub mod hash;
pub mod input;
pub mod items;
pub mod moves;
pub mod orbs;
pub mod push;
pub mod rng;
pub mod run;
pub mod scripting;
pub mod state;
pub mod stats;
pub mod step;
pub mod trig;
pub mod vec2;
pub mod wing;

pub use content::{Content, FighterParams, Stage};
pub use fixed::Fx;
pub use input::Input;
pub use rng::Rng;
pub use state::{Fighter, GameState, Item, MatchRules, Projectile};
pub use step::step;
pub use vec2::Vec2;

/// Bump whenever simulation behaviour changes. Exchanged in the netplay handshake and stored in replays.
pub const SIM_VERSION: u16 = 144;
/// Players: the fighters a match's inputs drive (and whose stocks, spawns and results count).
pub const MAX_FIGHTERS: usize = 4;
/// Grunts (the run's enemies, R2c, sim 131): fighters no player drives, moved by the run's own input layer (`run::mobs`). They
/// live in `GameState::fighters` after the players (inactive outside a run) and are stepped by the same fighter, combat and grab
/// code as the players; they never hit, grab or reflect one another (`combat::rivals`).
pub const MAX_GRUNTS: usize = 12;
/// Every body the step moves: the players, then the grunts.
pub const MAX_BODIES: usize = MAX_FIGHTERS + MAX_GRUNTS;
pub const MAX_SCRIPT_VARS: usize = 16;
/// Persistent script variables each fighter and each projectile carries (see `sim-script`).
pub const FIGHTER_VARS: usize = 4;
pub const PROJECTILE_VARS: usize = 2;
