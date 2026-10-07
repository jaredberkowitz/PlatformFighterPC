use sim_script::api::{self, Kind};
use sim_script::{parse_fixed, run, Fault, Host, Op, Program, INSTRUCTION_BUDGET, ONE};

/// A host that records what a script did.
#[derive(Default)]
struct Probe {
    regs: [i32; 32],
    vars: [i32; 4],
    calls: Vec<(u8, Vec<i32>)>,
}

impl Host for Probe {
    fn register(&self, id: u8) -> i32 {
        self.regs[usize::from(id)]
    }
    fn var(&self, slot: u8) -> i32 {
        self.vars[usize::from(slot)]
    }
    fn set_var(&mut self, slot: u8, value: i32) {
        self.vars[usize::from(slot)] = value;
    }
    fn call(&mut self, id: u8, args: &[i32]) -> i32 {
        self.calls.push((id, args.to_vec()));
        0
    }
}

fn fx(text: &str) -> i32 {
    parse_fixed(text).unwrap()
}

fn compile(src: &str) -> Program {
    Program::compile(Kind::Fighter, src).unwrap_or_else(|e| panic!("{e}\n{src}"))
}

fn error(src: &str) -> String {
    Program::compile(Kind::Fighter, src)
        .unwrap_err()
        .to_string()
}

fn run_with(src: &str, probe: &mut Probe) -> sim_script::Outcome {
    run(&compile(src), probe)
}

fn args(p: &Probe) -> Vec<Vec<i32>> {
    p.calls.iter().map(|c| c.1.clone()).collect()
}

#[test]
fn arithmetic_is_fixed_point() {
    let mut p = Probe::default();
    run_with("set_vel(1.5 * 2 + 0.25, 10 / 4 - 1);", &mut p);
    assert_eq!(p.calls, vec![(api::F_SET_VEL, vec![fx("3.25"), fx("1.5")])]);
}

#[test]
fn unary_precedence_and_comparisons() {
    let mut p = Probe::default();
    run_with(
        "set_vel(-2 * 3, 0); set_vel(1 < 2, 2 <= 1); set_vel(!0, !5); set_vel(1 && 0, 1 || 0);",
        &mut p,
    );
    assert_eq!(
        args(&p),
        vec![vec![fx("-6"), 0], vec![ONE, 0], vec![ONE, 0], vec![0, ONE],]
    );
}

#[test]
fn registers_locals_and_vars() {
    let mut p = Probe::default();
    p.regs[usize::from(api::R_FRAME)] = fx("12");
    p.regs[usize::from(api::R_STICK_X)] = fx("0.5");
    run_with(
        "var count; let k = frame * 2; count += 1; count += 1; set_vel(k * stick_x, count);",
        &mut p,
    );
    assert_eq!(p.calls[0].1, vec![fx("12"), fx("2")]);
    assert_eq!(p.vars[0], fx("2"), "the variable is stored by the host");
}

#[test]
fn variables_persist_between_runs() {
    let prog = compile("var n; n += 1; if n >= 3 { end(); }");
    let mut p = Probe::default();
    for _ in 0..5 {
        run(&prog, &mut p);
    }
    assert_eq!(p.vars[0], fx("5"));
    assert_eq!(p.calls.len(), 3, "end() ran on runs 3, 4 and 5");
}

#[test]
fn if_else_chains() {
    let src = "if frame == 1 { set_vel(1, 0); } else if frame == 2 { set_vel(2, 0); } else { set_vel(3, 0); }";
    for (frame, want) in [("1", "1"), ("2", "2"), ("7", "3")] {
        let mut p = Probe::default();
        p.regs[usize::from(api::R_FRAME)] = fx(frame);
        run_with(src, &mut p);
        assert_eq!(
            p.calls,
            vec![(api::F_SET_VEL, vec![fx(want), 0])],
            "frame {frame}"
        );
    }
}

#[test]
fn while_loops_run_and_stop() {
    let mut p = Probe::default();
    let out = run_with("let i = 0; while i < 5 { add_vel(1, 0); i += 1; }", &mut p);
    assert_eq!(out.fault, None);
    assert_eq!(p.calls.len(), 5);
}

#[test]
fn a_runaway_loop_hits_the_budget_and_keeps_its_effects() {
    let mut p = Probe::default();
    let out = run_with("while true { add_vel(1, 0); }", &mut p);
    assert_eq!(out.fault, Some(Fault::Budget));
    assert_eq!(out.steps, INSTRUCTION_BUDGET);
    assert!(p.calls.len() > 100, "effects before the cut-off stay");
    // And it is exactly the same every time.
    let mut q = Probe::default();
    run_with("while true { add_vel(1, 0); }", &mut q);
    assert_eq!(p.calls, q.calls);
}

#[test]
fn return_stops_early() {
    let mut p = Probe::default();
    run_with("end(); return; turn();", &mut p);
    assert_eq!(p.calls.len(), 1);
}

#[test]
fn math_functions() {
    let mut p = Probe::default();
    run_with(
        "set_vel(abs(-2.5), sign(-9)); set_vel(min(1, 2), max(1, 2)); set_vel(clamp(5, 0, 3), clamp(-5, 0, 3)); \
         set_vel(floor(2.75), floor(-0.5)); set_vel(sqrt(9), sqrt(2));",
        &mut p,
    );
    let got = args(&p);
    assert_eq!(got[0], vec![fx("2.5"), fx("-1")]);
    assert_eq!(got[1], vec![fx("1"), fx("2")]);
    assert_eq!(got[2], vec![fx("3"), 0]);
    assert_eq!(got[3], vec![fx("2"), fx("-1")]);
    assert_eq!(got[4][0], fx("3"));
    assert!(
        (got[4][1] - fx("1.41421")).abs() <= 2,
        "sqrt(2) = {}",
        got[4][1]
    );
}

#[test]
fn division_by_zero_and_overflow_are_defined() {
    let mut p = Probe::default();
    run_with(
        "set_vel(5 / 0, 30000 * 30000); set_vel(30000 + 30000, -30000 - 30000);",
        &mut p,
    );
    assert_eq!(p.calls[0].1, vec![0, i32::MAX]);
    assert_eq!(p.calls[1].1, vec![i32::MAX, i32::MIN]);
}

#[test]
fn compile_errors_name_the_line() {
    let e = error("end();\n\nfoo = 1;");
    assert!(e.starts_with("line 3:"), "{e}");
    assert!(error("set_vel(1);").contains("takes 2"));
    assert!(error("x = 1;").contains("read-only"));
    assert!(error("let a = 1; let a = 2;").contains("already defined"));
    assert!(error("let a = b;").contains("unknown name `b`"));
    assert!(error("explode();").contains("unknown function"));
    assert!(error("if 1 { end();").contains("expected `}`"));
    assert!(error("end()").contains("expected `;`"));
    assert!(error("let a = 1 $ 2;").contains("unexpected character"));
    assert!(error("let a = 1.2.3;").contains("not a number"));
}

#[test]
fn limits_are_enforced() {
    let many_locals: String = (0..9).map(|i| format!("let a{i} = 0;")).collect();
    assert!(error(&many_locals).contains("at most 8"));
    let many_vars = "var a; var b; var c; var d; var e;";
    assert!(error(many_vars).contains("at most 4"));
    assert!(Program::compile(Kind::Projectile, "var a; var b; var c;")
        .unwrap_err()
        .to_string()
        .contains("at most 2"));
    let deep = format!("let a = {}1{};", "(1 + ".repeat(20), ")".repeat(20));
    assert!(error(&deep).contains("too deeply nested"));
    let long = "end();".repeat(1000);
    assert!(error(&long).contains("longer than"));
}

#[test]
fn the_api_is_the_whitelist() {
    // A fighter script cannot call projectile functions, and vice versa.
    assert!(error("kill();").contains("unknown function"));
    assert!(Program::compile(Kind::Projectile, "end();").is_err());
    assert!(Program::compile(
        Kind::Projectile,
        "set_vel(pvx, pvy); if age > life { kill(); }"
    )
    .is_ok());
    // Nothing resembling I/O, time or randomness exists.
    for name in ["open", "read", "time", "now", "rand", "print", "http"] {
        assert!(
            error(&format!("{name}();")).contains("unknown function"),
            "{name}"
        );
    }
}

#[test]
fn comments_do_not_change_the_code() {
    let a = compile("end();");
    let b = compile("// stop right away\n  end();   // really\n");
    assert_eq!(a.code, b.code);
    assert_eq!(a.hash_bytes(), b.hash_bytes());
    assert_ne!(a.source, b.source);
}

#[test]
fn different_code_hashes_differently() {
    assert_ne!(
        compile("end();").hash_bytes(),
        compile("turn();").hash_bytes()
    );
    assert_ne!(
        compile("set_vel(1, 0);").hash_bytes(),
        compile("set_vel(1, 1);").hash_bytes()
    );
}

#[test]
fn hand_built_garbage_programs_cannot_crash_the_vm() {
    let junk = [
        vec![Op::Add],
        vec![Op::Push(1), Op::Jump(9999)],
        vec![Op::Load(200)],
        vec![Op::Call(250)],
        vec![Op::Reg(99)],
        vec![Op::GetVar(200)],
        vec![Op::Jump(0)],
        (0..40).map(|_| Op::Push(1)).collect(),
    ];
    for code in junk {
        let prog = Program {
            kind: Kind::Fighter,
            code,
            source: String::new(),
        };
        let out = run(&prog, &mut Probe::default());
        assert!(out.steps <= INSTRUCTION_BUDGET);
    }
}

/// Random text as source never panics the compiler.
#[test]
fn the_compiler_never_panics_on_garbage() {
    let mut x: u32 = 99;
    let alphabet: Vec<char> = "abcxyz_ 01.9;(){}+-*/=<>!&|,\n\t$\"".chars().collect();
    for _ in 0..5_000 {
        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let len = (x >> 24) as usize % 60;
        let src: String = (0..len)
            .map(|_| {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                alphabet[(x >> 16) as usize % alphabet.len()]
            })
            .collect();
        let _ = Program::compile(Kind::Fighter, &src);
        let _ = Program::compile(Kind::Projectile, &src);
    }
}
