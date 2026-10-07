//! The scripting VM for special moves and projectile behaviour (plan section 7.1).
//!
//! * **Integer only.** Every value is an `i32`; numbers are 16.16 fixed point, so `1` is 65536 and `0.5` is 32768.
//!   There is no float anywhere, and all arithmetic saturates instead of overflowing.
//! * **Bounded.** A run executes at most [`INSTRUCTION_BUDGET`] instructions and uses a fixed stack. A script that
//!   loops forever or misbehaves is stopped at the budget; the same instructions run on every machine, so both
//!   peers stop at the same place.
//! * **Whitelisted.** A script can only read the registers and call the functions listed in [`api`]. There is no
//!   file, network, clock or random access, and no way to name anything else.
//! * **Snapshot friendly.** A script keeps nothing between runs except the few persistent variables the host
//!   gives it (`var` declarations), which live in the host's game state, so rollback needs no special handling.
//!
//! This crate has no dependencies (not even the simulation): the host implements [`Host`] to say what the
//! registers read and what the functions do.

pub mod api;
pub mod check;
mod compile;
mod fixed;
mod vm;

pub use api::Kind;
pub use compile::normalize_source;
pub use compile::CompileError;
pub use fixed::{format_fixed, parse_fixed};
pub use vm::{run, Fault, Host, Outcome};

/// Bump when bytecode semantics change.
pub const SCRIPT_VERSION: u32 = 1;
/// Hard cap on VM instructions executed per run.
pub const INSTRUCTION_BUDGET: u32 = 1_000;
/// Operand stack size. The compiler rejects expressions that would need more.
pub const MAX_STACK: usize = 16;
/// Local variables (`let`) per script.
pub const MAX_LOCALS: usize = 8;
/// Longest program, in instructions.
pub const MAX_CODE: usize = 512;
/// Longest script source, in bytes.
pub const MAX_SOURCE: usize = 4096;
/// Fixed-point `1.0`.
pub const ONE: i32 = 1 << 16;

/// One VM instruction. Operands are part of the instruction, so a program is just a `Vec<Op>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Push(i32),
    /// Local variable access.
    Load(u8),
    Store(u8),
    /// Read a host register.
    Reg(u8),
    /// Persistent variable access (stored by the host).
    GetVar(u8),
    SetVar(u8),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    /// Absolute instruction index.
    Jump(u16),
    JumpIfZero(u16),
    /// Calls a whitelisted function; its arguments are on the stack and it leaves one result.
    Call(u8),
    Pop,
    Halt,
}

impl Op {
    /// A stable encoding, used for hashing content.
    fn encode(self, out: &mut Vec<u8>) {
        let (tag, operand): (u8, i32) = match self {
            Op::Push(v) => (0, v),
            Op::Load(n) => (1, i32::from(n)),
            Op::Store(n) => (2, i32::from(n)),
            Op::Reg(n) => (3, i32::from(n)),
            Op::GetVar(n) => (4, i32::from(n)),
            Op::SetVar(n) => (5, i32::from(n)),
            Op::Add => (6, 0),
            Op::Sub => (7, 0),
            Op::Mul => (8, 0),
            Op::Div => (9, 0),
            Op::Neg => (10, 0),
            Op::Not => (11, 0),
            Op::Eq => (12, 0),
            Op::Ne => (13, 0),
            Op::Lt => (14, 0),
            Op::Le => (15, 0),
            Op::Gt => (16, 0),
            Op::Ge => (17, 0),
            Op::And => (18, 0),
            Op::Or => (19, 0),
            Op::Jump(n) => (20, i32::from(n)),
            Op::JumpIfZero(n) => (21, i32::from(n)),
            Op::Call(n) => (22, i32::from(n)),
            Op::Pop => (23, 0),
            Op::Halt => (24, 0),
        };
        out.push(tag);
        out.extend_from_slice(&operand.to_le_bytes());
    }
}

/// A compiled script and the source it came from. Only the code (and kind) matter to the simulation; the
/// source is kept so editors and the content format can show it again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub kind: Kind,
    pub code: Vec<Op>,
    pub source: String,
}

impl Program {
    /// Compiles `source` for a script of the given kind.
    pub fn compile(kind: Kind, source: &str) -> Result<Program, CompileError> {
        compile::compile(kind, source)
    }

    /// The bytes the content hash covers: kind, version and code, not the source text (comments and spacing
    /// cannot change behaviour).
    pub fn hash_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.code.len() * 5 + 8);
        out.extend_from_slice(&SCRIPT_VERSION.to_le_bytes());
        out.push(self.kind as u8);
        out.extend_from_slice(&(self.code.len() as u32).to_le_bytes());
        for op in &self.code {
            op.encode(&mut out);
        }
        out
    }
}
