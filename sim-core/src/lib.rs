//! Deterministic simulation core.
//!
//! Rules for everything in this crate (and any crate that opts into the workspace lints):
//! * No floating point. Use [`fixed::Fx`] (16.16) and the lookup-table trig in [`trig`].
//! * No `HashMap`/`HashSet`. Use arrays, slot pools or `BTreeMap`.
//! * No clocks, threads, I/O or global state. All randomness comes from [`rng::Rng`] stored in state.
//! * Everything that changes lives in [`state::GameState`]; content is immutable and hashed.

pub mod collision;
pub mod combat;
pub mod content;
pub mod fighter;
pub mod fixed;
pub mod fuzz;
pub mod grab;
pub mod hash;
pub mod input;
pub mod moves;
pub mod rng;
pub mod scripting;
pub mod state;
pub mod step;
pub mod trig;
pub mod vec2;

pub use content::{Content, FighterParams, Stage};
pub use fixed::Fx;
pub use input::Input;
pub use rng::Rng;
pub use state::{Fighter, GameState, MatchRules, Projectile};
pub use step::step;
pub use vec2::Vec2;

/// Bump whenever simulation behaviour changes. Exchanged in the netplay handshake and stored in replays.
pub const SIM_VERSION: u16 = 25;
pub const MAX_FIGHTERS: usize = 4;
pub const MAX_SCRIPT_VARS: usize = 16;
/// Persistent script variables each fighter and each projectile carries (see `sim-script`).
pub const FIGHTER_VARS: usize = 4;
pub const PROJECTILE_VARS: usize = 2;
