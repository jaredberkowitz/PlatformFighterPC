//! Placeholder for the scripting VM (plan section 7.1, built in Phase 5).
//!
//! Design constraints already fixed: integer-only state stored in `GameState::script_vars`,
//! a hard per-frame instruction budget, fixed-point math only, and a whitelisted API with no
//! file, network or clock access.

/// Bump when bytecode semantics change.
pub const SCRIPT_VERSION: u32 = 0;
/// Hard cap on VM instructions executed per fighter per frame.
pub const INSTRUCTION_BUDGET: u32 = 1_000;
