//! Source text to bytecode. The language is small on purpose:
//!
//! ```text
//! var charge;                     // persistent variable (kept between runs, rolled back with the game)
//! let reach = 2.5;                // local variable (this run only)
//! if frame == 12 && attack {      // numbers are fixed point; comparisons give 1 or 0
//!     set_vel(reach * stick_x, 0);
//! } else if hit { end(); }
//! while charge < 3 { charge += 1; }   // loops are allowed; the instruction budget stops runaway ones
//! ```

use crate::api::Kind;
use crate::fixed::parse_fixed;
use crate::{Op, Program, MAX_CODE, MAX_LOCALS, MAX_SOURCE, MAX_STACK, ONE};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for CompileError {}

type Result<T> = std::result::Result<T, CompileError>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Num(i32),
    Ident(String),
    Sym(&'static str),
    End,
}

#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    line: usize,
}

const TWO_CHAR: [&str; 8] = ["==", "!=", "<=", ">=", "&&", "||", "+=", "-="];
const ONE_CHAR: [&str; 14] = [
    "(", ")", "{", "}", ";", ",", "=", "<", ">", "+", "-", "*", "/", "!",
];

fn lex(source: &str) -> Result<Vec<Token>> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut i = 0;
    let err = |line: usize, message: String| CompileError { line, message };
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c.is_ascii_digit()
            || (c == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let raw = parse_fixed(&text)
                .ok_or_else(|| err(line, format!("`{text}` is not a number (or is too big)")))?;
            tokens.push(Token {
                tok: Tok::Num(raw),
                line,
            });
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token {
                tok: Tok::Ident(chars[start..i].iter().collect()),
                line,
            });
        } else {
            let two: String = chars[i..chars.len().min(i + 2)].iter().collect();
            let one = c.to_string();
            if let Some(s) = TWO_CHAR.iter().find(|s| **s == two) {
                tokens.push(Token {
                    tok: Tok::Sym(s),
                    line,
                });
                i += 2;
            } else if let Some(s) = ONE_CHAR.iter().find(|s| **s == one) {
                tokens.push(Token {
                    tok: Tok::Sym(s),
                    line,
                });
                i += 1;
            } else {
                return Err(err(line, format!("unexpected character `{c}`")));
            }
        }
    }
    tokens.push(Token {
        tok: Tok::End,
        line,
    });
    Ok(tokens)
}

struct Compiler {
    kind: Kind,
    tokens: Vec<Token>,
    pos: usize,
    code: Vec<Op>,
    locals: Vec<String>,
    vars: Vec<String>,
    depth: usize,
}

/// The canonical form of a script's text: no blank lines at either end, no common indentation, no trailing
/// spaces. Programs always keep this form, so the same script compares equal however it was indented in a file.
pub fn normalize_source(source: &str) -> String {
    let lines: Vec<&str> = source.lines().map(str::trim_end).collect();
    let Some(first) = lines.iter().position(|l| !l.is_empty()) else {
        return String::new();
    };
    let last = lines.iter().rposition(|l| !l.is_empty()).unwrap_or(first);
    let body = &lines[first..=last];
    let indent = body
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    body.iter()
        .map(|l| if l.len() >= indent { &l[indent..] } else { *l })
        .collect::<Vec<_>>()
        .join(
            "
",
        )
}

pub fn compile(kind: Kind, source: &str) -> std::result::Result<Program, CompileError> {
    let source = normalize_source(source);
    let source = source.as_str();
    if source.len() > MAX_SOURCE {
        return Err(CompileError {
            line: 1,
            message: format!("script is longer than {MAX_SOURCE} bytes"),
        });
    }
    let mut c = Compiler {
        kind,
        tokens: lex(source)?,
        pos: 0,
        code: Vec::new(),
        locals: Vec::new(),
        vars: Vec::new(),
        depth: 0,
    };
    while c.peek() != &Tok::End {
        c.statement()?;
    }
    c.emit(Op::Halt)?;
    Ok(Program {
        kind,
        code: c.code,
        source: source.to_string(),
    })
}

impl Compiler {
    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].tok
    }

    fn line(&self) -> usize {
        self.tokens[self.pos].line
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(CompileError {
            line: self.line(),
            message: message.into(),
        })
    }

    fn next(&mut self) -> Tok {
        let t = self.tokens[self.pos].tok.clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, sym: &'static str) -> bool {
        if self.peek() == &Tok::Sym(sym) {
            self.next();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, sym: &'static str) -> Result<()> {
        if self.eat(sym) {
            Ok(())
        } else {
            self.error(format!("expected `{sym}`"))
        }
    }

    fn ident(&mut self, what: &str) -> Result<String> {
        match self.next() {
            Tok::Ident(s) => Ok(s),
            _ => {
                self.pos = self.pos.saturating_sub(1);
                self.error(format!("expected {what}"))
            }
        }
    }

    /// Appends an instruction, tracking how deep the operand stack gets.
    fn emit(&mut self, op: Op) -> Result<usize> {
        let (pops, pushes) = match op {
            Op::Push(_) | Op::Load(_) | Op::Reg(_) | Op::GetVar(_) => (0, 1),
            Op::Store(_) | Op::SetVar(_) | Op::JumpIfZero(_) | Op::Pop => (1, 0),
            Op::Neg | Op::Not => (1, 1),
            Op::Jump(_) | Op::Halt => (0, 0),
            Op::Call(id) => {
                let arity = self
                    .kind
                    .functions()
                    .iter()
                    .find(|f| f.id == id)
                    .map_or(0, |f| usize::from(f.arity));
                (arity, 1)
            }
            _ => (2, 1),
        };
        self.depth = self.depth.saturating_sub(pops) + pushes;
        if self.depth > MAX_STACK {
            return self.error("expression is too deeply nested");
        }
        if self.code.len() >= MAX_CODE {
            return self.error(format!("script is longer than {MAX_CODE} instructions"));
        }
        self.code.push(op);
        Ok(self.code.len() - 1)
    }

    fn here(&self) -> u16 {
        self.code.len() as u16
    }

    fn patch(&mut self, at: usize, target: u16) {
        self.code[at] = match self.code[at] {
            Op::Jump(_) => Op::Jump(target),
            _ => Op::JumpIfZero(target),
        };
    }

    // ---- Statements ----

    fn block(&mut self) -> Result<()> {
        self.expect("{")?;
        while self.peek() != &Tok::Sym("}") {
            if self.peek() == &Tok::End {
                return self.error("expected `}` before the end of the script");
            }
            self.statement()?;
        }
        self.expect("}")
    }

    fn statement(&mut self) -> Result<()> {
        let Tok::Ident(word) = self.peek().clone() else {
            return self.error("expected a statement");
        };
        match word.as_str() {
            "let" => {
                self.next();
                let name = self.ident("a name")?;
                self.expect("=")?;
                self.expression()?;
                self.expect(";")?;
                if self.locals.len() >= MAX_LOCALS {
                    return self.error(format!("at most {MAX_LOCALS} `let` variables"));
                }
                if self.is_known(&name) {
                    return self.error(format!("`{name}` is already defined"));
                }
                self.locals.push(name);
                self.emit(Op::Store(self.locals.len() as u8 - 1))?;
            }
            "var" => {
                self.next();
                let name = self.ident("a name")?;
                self.expect(";")?;
                if self.vars.len() >= self.kind.max_vars() {
                    return self.error(format!(
                        "a {} script can declare at most {} `var` variables",
                        self.kind.name(),
                        self.kind.max_vars()
                    ));
                }
                if self.is_known(&name) {
                    return self.error(format!("`{name}` is already defined"));
                }
                self.vars.push(name);
            }
            "if" => self.if_statement()?,
            "while" => {
                self.next();
                let start = self.here();
                self.expression()?;
                let exit = self.emit(Op::JumpIfZero(0))?;
                self.block()?;
                self.emit(Op::Jump(start))?;
                let end = self.here();
                self.patch(exit, end);
            }
            "return" => {
                self.next();
                self.expect(";")?;
                self.emit(Op::Halt)?;
            }
            _ => self.simple_statement(&word)?,
        }
        Ok(())
    }

    fn if_statement(&mut self) -> Result<()> {
        self.next(); // `if`
        self.expression()?;
        let skip = self.emit(Op::JumpIfZero(0))?;
        self.block()?;
        if self.peek() == &Tok::Ident("else".to_string()) {
            self.next();
            let over = self.emit(Op::Jump(0))?;
            let else_start = self.here();
            self.patch(skip, else_start);
            if self.peek() == &Tok::Ident("if".to_string()) {
                self.if_statement()?;
            } else {
                self.block()?;
            }
            let end = self.here();
            self.patch(over, end);
        } else {
            let end = self.here();
            self.patch(skip, end);
        }
        Ok(())
    }

    /// An assignment (`x = e;`, `x += e;`) or a call used as a statement (`end();`).
    fn simple_statement(&mut self, name: &str) -> Result<()> {
        let next = self.tokens.get(self.pos + 1).map(|t| t.tok.clone());
        match next {
            Some(Tok::Sym(op @ ("=" | "+=" | "-="))) => {
                self.next();
                self.next();
                let (load, store) = if let Some(i) = self.locals.iter().position(|n| n == name) {
                    (Op::Load(i as u8), Op::Store(i as u8))
                } else if let Some(i) = self.vars.iter().position(|n| n == name) {
                    (Op::GetVar(i as u8), Op::SetVar(i as u8))
                } else if self.kind.registers().iter().any(|r| r.name == name) {
                    return self.error(format!(
                        "`{name}` is read-only; use a function such as set_vel to change it"
                    ));
                } else {
                    return self.error(format!(
                        "unknown variable `{name}` (declare it with `let` or `var` first)"
                    ));
                };
                if op != "=" {
                    self.emit(load)?;
                }
                self.expression()?;
                match op {
                    "+=" => {
                        self.emit(Op::Add)?;
                    }
                    "-=" => {
                        self.emit(Op::Sub)?;
                    }
                    _ => {}
                }
                self.expect(";")?;
                self.emit(store)?;
            }
            Some(Tok::Sym("(")) => {
                self.expression()?;
                self.expect(";")?;
                self.emit(Op::Pop)?;
            }
            _ => return self.error(format!("expected `=` or `(` after `{name}`")),
        }
        Ok(())
    }

    fn is_known(&self, name: &str) -> bool {
        self.locals.iter().any(|n| n == name)
            || self.vars.iter().any(|n| n == name)
            || self.kind.registers().iter().any(|r| r.name == name)
            || self.kind.functions().iter().any(|f| f.name == name)
            || matches!(
                name,
                "let" | "var" | "if" | "else" | "while" | "return" | "true" | "false"
            )
    }

    // ---- Expressions, lowest precedence first ----

    fn expression(&mut self) -> Result<()> {
        self.binary(0)
    }

    fn binary(&mut self, level: usize) -> Result<()> {
        const LEVELS: [&[(&str, Op)]; 5] = [
            &[("||", Op::Or)],
            &[("&&", Op::And)],
            &[("==", Op::Eq), ("!=", Op::Ne)],
            &[("<=", Op::Le), (">=", Op::Ge), ("<", Op::Lt), (">", Op::Gt)],
            &[("+", Op::Add), ("-", Op::Sub)],
        ];
        if level == LEVELS.len() {
            return self.term();
        }
        self.binary(level + 1)?;
        'more: loop {
            for (sym, op) in LEVELS[level] {
                if self.eat(sym) {
                    self.binary(level + 1)?;
                    self.emit(*op)?;
                    continue 'more;
                }
            }
            return Ok(());
        }
    }

    fn term(&mut self) -> Result<()> {
        self.unary()?;
        loop {
            if self.eat("*") {
                self.unary()?;
                self.emit(Op::Mul)?;
            } else if self.eat("/") {
                self.unary()?;
                self.emit(Op::Div)?;
            } else {
                return Ok(());
            }
        }
    }

    fn unary(&mut self) -> Result<()> {
        if self.eat("-") {
            self.unary()?;
            self.emit(Op::Neg)?;
        } else if self.eat("!") {
            self.unary()?;
            self.emit(Op::Not)?;
        } else {
            self.primary()?;
        }
        Ok(())
    }

    fn primary(&mut self) -> Result<()> {
        match self.next() {
            Tok::Num(v) => {
                self.emit(Op::Push(v))?;
            }
            Tok::Sym("(") => {
                self.expression()?;
                self.expect(")")?;
            }
            Tok::Ident(name) => self.name(&name)?,
            _ => {
                self.pos = self.pos.saturating_sub(1);
                return self.error("expected a number, a name or `(`");
            }
        }
        Ok(())
    }

    fn name(&mut self, name: &str) -> Result<()> {
        if self.peek() == &Tok::Sym("(") {
            let Some(func) = self
                .kind
                .functions()
                .iter()
                .find(|f| f.name == name)
                .copied()
            else {
                return self.error(format!(
                    "unknown function `{name}` (a {} script can call: {})",
                    self.kind.name(),
                    self.kind
                        .functions()
                        .iter()
                        .map(|f| f.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            };
            self.next();
            let mut args = 0;
            if self.peek() != &Tok::Sym(")") {
                loop {
                    self.expression()?;
                    args += 1;
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.expect(")")?;
            if args != usize::from(func.arity) {
                return self.error(format!(
                    "`{name}` takes {} value(s), not {args}",
                    func.arity
                ));
            }
            self.emit(Op::Call(func.id))?;
        } else if let Some(i) = self.locals.iter().position(|n| n == name) {
            self.emit(Op::Load(i as u8))?;
        } else if let Some(i) = self.vars.iter().position(|n| n == name) {
            self.emit(Op::GetVar(i as u8))?;
        } else if let Some(r) = self.kind.registers().iter().find(|r| r.name == name) {
            self.emit(Op::Reg(r.id))?;
        } else if name == "true" {
            self.emit(Op::Push(ONE))?;
        } else if name == "false" {
            self.emit(Op::Push(0))?;
        } else {
            return self.error(format!("unknown name `{name}`"));
        }
        Ok(())
    }
}
