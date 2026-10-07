//! The interpreter. A run is a pure function of the program, the host's registers and variables, and nothing else.

use crate::api::{self, FIRST_HOST_FUNC};
use crate::{Op, Program, INSTRUCTION_BUDGET, MAX_LOCALS, MAX_STACK, ONE};

/// What a script can see and do. The simulation implements this for fighters and projectiles.
pub trait Host {
    /// A named read-only value (see the register tables in [`api`]).
    fn register(&self, id: u8) -> i32;
    /// Persistent variable `slot`, kept in the game state.
    fn var(&self, slot: u8) -> i32;
    fn set_var(&mut self, slot: u8, value: i32);
    /// A host function (ids from [`FIRST_HOST_FUNC`] up). `args` holds exactly the declared number of values.
    fn call(&mut self, id: u8, args: &[i32]) -> i32;
}

/// Why a run stopped early. The effects it already had stay; the rest of the script does not run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Used up [`INSTRUCTION_BUDGET`].
    Budget,
    StackOverflow,
    StackUnderflow,
    /// A jump, local, register or function that does not exist (only possible for hand-built programs).
    BadOperand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub steps: u32,
    pub fault: Option<Fault>,
}

fn saturate(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn mul(a: i32, b: i32) -> i32 {
    saturate((i64::from(a) * i64::from(b)) >> 16)
}

fn div(a: i32, b: i32) -> i32 {
    if b == 0 {
        0
    } else {
        saturate((i64::from(a) << 16) / i64::from(b))
    }
}

fn truth(b: bool) -> i32 {
    if b {
        ONE
    } else {
        0
    }
}

fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Math done by the VM itself.
fn native(id: u8, args: &[i32]) -> Option<i32> {
    let a = *args.first()?;
    Some(match id {
        api::F_ABS => a.saturating_abs(),
        api::F_MIN => a.min(*args.get(1)?),
        api::F_MAX => a.max(*args.get(1)?),
        api::F_CLAMP => {
            let (lo, hi) = (*args.get(1)?, *args.get(2)?);
            if lo > hi {
                lo
            } else {
                a.clamp(lo, hi)
            }
        }
        api::F_SIGN => a.signum() * ONE,
        api::F_FLOOR => a & !(ONE - 1),
        api::F_SQRT => {
            if a <= 0 {
                0
            } else {
                // sqrt(raw / 2^16) * 2^16 = sqrt(raw * 2^16)
                isqrt((a as u64) << 16) as i32
            }
        }
        _ => return None,
    })
}

struct Stack {
    items: [i32; MAX_STACK],
    len: usize,
}

impl Stack {
    fn push(&mut self, v: i32) -> Result<(), Fault> {
        *self.items.get_mut(self.len).ok_or(Fault::StackOverflow)? = v;
        self.len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<i32, Fault> {
        self.len = self.len.checked_sub(1).ok_or(Fault::StackUnderflow)?;
        Ok(self.items[self.len])
    }
}

/// Runs the program once.
pub fn run<H: Host>(program: &Program, host: &mut H) -> Outcome {
    let mut steps = 0;
    let fault = execute(program, host, &mut steps).err();
    Outcome { steps, fault }
}

fn execute<H: Host>(program: &Program, host: &mut H, steps: &mut u32) -> Result<(), Fault> {
    let mut stack = Stack {
        items: [0; MAX_STACK],
        len: 0,
    };
    let mut locals = [0i32; MAX_LOCALS];
    let mut pc = 0usize;
    let registers = program.kind.registers();
    let functions = program.kind.functions();

    while let Some(&op) = program.code.get(pc) {
        if *steps >= INSTRUCTION_BUDGET {
            return Err(Fault::Budget);
        }
        *steps += 1;
        pc += 1;
        match op {
            Op::Push(v) => stack.push(v)?,
            Op::Load(n) => stack.push(*locals.get(usize::from(n)).ok_or(Fault::BadOperand)?)?,
            Op::Store(n) => {
                let v = stack.pop()?;
                *locals.get_mut(usize::from(n)).ok_or(Fault::BadOperand)? = v;
            }
            Op::Reg(id) => {
                if !registers.iter().any(|r| r.id == id) {
                    return Err(Fault::BadOperand);
                }
                stack.push(host.register(id))?;
            }
            Op::GetVar(n) => {
                if usize::from(n) >= program.kind.max_vars() {
                    return Err(Fault::BadOperand);
                }
                stack.push(host.var(n))?;
            }
            Op::SetVar(n) => {
                if usize::from(n) >= program.kind.max_vars() {
                    return Err(Fault::BadOperand);
                }
                let v = stack.pop()?;
                host.set_var(n, v);
            }
            Op::Neg => {
                let a = stack.pop()?;
                stack.push(a.saturating_neg())?;
            }
            Op::Not => {
                let a = stack.pop()?;
                stack.push(truth(a == 0))?;
            }
            Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Div
            | Op::Eq
            | Op::Ne
            | Op::Lt
            | Op::Le
            | Op::Gt
            | Op::Ge
            | Op::And
            | Op::Or => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(match op {
                    Op::Add => a.saturating_add(b),
                    Op::Sub => a.saturating_sub(b),
                    Op::Mul => mul(a, b),
                    Op::Div => div(a, b),
                    Op::Eq => truth(a == b),
                    Op::Ne => truth(a != b),
                    Op::Lt => truth(a < b),
                    Op::Le => truth(a <= b),
                    Op::Gt => truth(a > b),
                    Op::Ge => truth(a >= b),
                    Op::And => truth(a != 0 && b != 0),
                    _ => truth(a != 0 || b != 0),
                })?;
            }
            Op::Jump(target) => pc = usize::from(target),
            Op::JumpIfZero(target) => {
                if stack.pop()? == 0 {
                    pc = usize::from(target);
                }
            }
            Op::Call(id) => {
                let f = functions
                    .iter()
                    .find(|f| f.id == id)
                    .ok_or(Fault::BadOperand)?;
                let n = usize::from(f.arity);
                if stack.len < n {
                    return Err(Fault::StackUnderflow);
                }
                let base = stack.len - n;
                let mut args = [0i32; 4];
                args[..n].copy_from_slice(&stack.items[base..stack.len]);
                stack.len = base;
                let result = if id < FIRST_HOST_FUNC {
                    native(id, &args[..n]).ok_or(Fault::BadOperand)?
                } else {
                    host.call(id, &args[..n])
                };
                stack.push(result)?;
            }
            Op::Pop => {
                stack.pop()?;
            }
            Op::Halt => return Ok(()),
        }
    }
    Ok(())
}
