//! A quick sanity run for authors and validators: execute a script many times against made-up inputs and report
//! the mistakes that are certain, such as invalid code or a loop that never ends.

use crate::vm::{run, Fault, Host};
use crate::{Program, INSTRUCTION_BUDGET};

/// A host with pseudo-random registers whose functions do nothing.
struct Dummy {
    seed: u32,
    vars: [i32; 4],
}

impl Dummy {
    fn next(&mut self) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.seed as i32 >> 12
    }
}

impl Host for Dummy {
    fn register(&self, id: u8) -> i32 {
        // Registers hold varied values, but the same value for the same register within one run.
        (self.seed ^ u32::from(id).wrapping_mul(2_654_435_761)) as i32 >> 12
    }
    fn var(&self, slot: u8) -> i32 {
        self.vars.get(usize::from(slot)).copied().unwrap_or(0)
    }
    fn set_var(&mut self, slot: u8, value: i32) {
        if let Some(v) = self.vars.get_mut(usize::from(slot)) {
            *v = value;
        }
    }
    fn call(&mut self, _id: u8, _args: &[i32]) -> i32 {
        0
    }
}

/// How many runs the check makes.
const RUNS: u32 = 64;

/// Returns a description of a certain problem, or `None` if the script looks sound.
pub fn dry_run(program: &Program) -> Option<String> {
    let mut host = Dummy {
        seed: 12345,
        vars: [0; 4],
    };
    let mut budget_hits = 0;
    for _ in 0..RUNS {
        host.next();
        match run(program, &mut host).fault {
            None => {}
            Some(Fault::Budget) => budget_hits += 1,
            Some(other) => return Some(format!("has invalid code ({other:?})")),
        }
    }
    if budget_hits == RUNS {
        return Some(format!(
            "never finishes: it uses all {INSTRUCTION_BUDGET} instructions every time (an endless loop?)"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;

    fn compile(src: &str) -> Program {
        Program::compile(Kind::Fighter, src).unwrap()
    }

    #[test]
    fn sound_scripts_pass() {
        assert_eq!(dry_run(&compile("if frame > 3 { set_vel(1, 0); }")), None);
        assert_eq!(
            dry_run(&compile("let i = 0; while i < 10 { i += 1; }")),
            None
        );
    }

    #[test]
    fn endless_loops_are_caught() {
        let msg = dry_run(&compile("while true { add_vel(0, 0); }")).unwrap();
        assert!(msg.contains("never finishes"), "{msg}");
    }

    #[test]
    fn sometimes_long_loops_are_allowed() {
        // Runs long only for some inputs: not a certain mistake.
        assert_eq!(
            dry_run(&compile("let i = 0; while i < frame * 100 { i += 1; }")),
            None
        );
    }

    #[test]
    fn broken_hand_built_code_is_caught() {
        use crate::Op;
        let p = Program {
            kind: Kind::Fighter,
            code: vec![Op::Add],
            source: String::new(),
        };
        assert!(dry_run(&p).unwrap().contains("invalid code"));
    }
}
