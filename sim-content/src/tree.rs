//! The text syntax of content files, before any meaning is attached to it.
//!
//! ```text
//! # a comment
//! kind "optional name" {
//!     field value
//!     field "a quoted value"
//!     nested { field value }
//!     script {
//!         raw script text, kept exactly as written
//!     }
//! }
//! ```
//!
//! A line inside a block is either `field value`, or `kind [name] { ... }`. Words are letters, digits and
//! `_ . + -`; anything else needs quotes. Raw `script` blocks hold script source, braces and all.

use std::fmt::Write as _;

/// Biggest file this parser will look at. Content is small; a huge file is a mistake or an attack.
pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: String,
    pub name: Option<String>,
    pub items: Vec<Item>,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Field {
        name: String,
        value: String,
        line: usize,
    },
    Block(Block),
    /// A block whose content is script source.
    Raw {
        kind: String,
        text: String,
        line: usize,
    },
}

impl Block {
    pub fn new(kind: &str, name: Option<&str>) -> Block {
        Block {
            kind: kind.to_string(),
            name: name.map(str::to_string),
            items: Vec::new(),
            line: 0,
        }
    }

    pub fn field(&mut self, name: &str, value: impl Into<String>) {
        self.items.push(Item::Field {
            name: name.to_string(),
            value: value.into(),
            line: 0,
        });
    }

    pub fn push(&mut self, block: Block) {
        self.items.push(Item::Block(block));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Word(String),
    Str(String),
    Open,
    Close,
    Raw(String),
}

struct Lexed {
    tok: Tok,
    line: usize,
}

/// Block kinds whose body is script source.
const RAW_KINDS: [&str; 2] = ["script", "projectile_script"];

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-')
}

fn lex(text: &str) -> Result<Vec<Lexed>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Lexed> = Vec::new();
    let mut line = 1;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' {
            let start_line = line;
            let mut s = String::new();
            i += 1;
            loop {
                match chars.get(i) {
                    None | Some('\n') => {
                        return Err(format!("line {start_line}: a quoted value is not closed"))
                    }
                    Some('"') => {
                        i += 1;
                        break;
                    }
                    Some('\\') => {
                        match chars.get(i + 1) {
                            Some('"') => s.push('"'),
                            Some('\\') => s.push('\\'),
                            _ => return Err(format!("line {line}: bad escape in a quoted value")),
                        }
                        i += 2;
                    }
                    Some(ch) => {
                        s.push(*ch);
                        i += 1;
                    }
                }
            }
            out.push(Lexed {
                tok: Tok::Str(s),
                line: start_line,
            });
        } else if c == '{' {
            // A script block is raw text up to its matching brace.
            let raw_kind = matches!(
                out.last(),
                Some(Lexed { tok: Tok::Word(w), .. }) if RAW_KINDS.contains(&w.as_str())
            );
            out.push(Lexed {
                tok: Tok::Open,
                line,
            });
            i += 1;
            if raw_kind {
                let start_line = line;
                let mut depth = 1;
                let mut body = String::new();
                while depth > 0 {
                    let Some(&ch) = chars.get(i) else {
                        return Err(format!("line {start_line}: a script block is not closed"));
                    };
                    if ch == '/' && chars.get(i + 1) == Some(&'/') {
                        // Comments may contain braces; copy them through without counting.
                        while i < chars.len() && chars[i] != '\n' {
                            body.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    match ch {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        '\n' => line += 1,
                        _ => {}
                    }
                    if depth > 0 {
                        body.push(ch);
                    }
                    i += 1;
                }
                out.push(Lexed {
                    tok: Tok::Raw(body),
                    line: start_line,
                });
                out.push(Lexed {
                    tok: Tok::Close,
                    line,
                });
            }
        } else if c == '}' {
            out.push(Lexed {
                tok: Tok::Close,
                line,
            });
            i += 1;
        } else if is_word_char(c) {
            let start = i;
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
            out.push(Lexed {
                tok: Tok::Word(chars[start..i].iter().collect()),
                line,
            });
        } else {
            return Err(format!("line {line}: unexpected character `{c}`"));
        }
    }
    Ok(out)
}

/// Parses a whole file into a root block (kind `file`) holding every top-level item.
pub fn parse(text: &str) -> Result<Block, String> {
    if text.len() > MAX_FILE_BYTES {
        return Err(format!("file is larger than {MAX_FILE_BYTES} bytes"));
    }
    let tokens = lex(text.strip_prefix('\u{feff}').unwrap_or(text))?;
    let mut pos = 0;
    let mut root = Block::new("file", None);
    root.line = 1;
    parse_items(&tokens, &mut pos, &mut root, 0)?;
    if pos < tokens.len() {
        return Err(format!("line {}: unexpected `}}`", tokens[pos].line));
    }
    Ok(root)
}

/// Deepest nesting accepted (the format only needs about four levels).
const MAX_DEPTH: usize = 12;

fn parse_items(
    tokens: &[Lexed],
    pos: &mut usize,
    into: &mut Block,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(format!("line {}: blocks are nested too deeply", into.line));
    }
    while let Some(t) = tokens.get(*pos) {
        let line = t.line;
        let key = match &t.tok {
            Tok::Close => return Ok(()),
            Tok::Word(w) => w.clone(),
            _ => return Err(format!("line {line}: expected a name")),
        };
        *pos += 1;
        let next = tokens.get(*pos).map(|t| &t.tok);
        match next {
            Some(Tok::Open) => {
                *pos += 1;
                if let Some(Lexed {
                    tok: Tok::Raw(body),
                    ..
                }) = tokens.get(*pos)
                {
                    into.items.push(Item::Raw {
                        kind: key,
                        text: body.clone(),
                        line,
                    });
                    *pos += 1;
                    expect_close(tokens, pos, line)?;
                } else {
                    let mut block = Block::new(&key, None);
                    block.line = line;
                    parse_items(tokens, pos, &mut block, depth + 1)?;
                    expect_close(tokens, pos, line)?;
                    into.items.push(Item::Block(block));
                }
            }
            Some(Tok::Word(v) | Tok::Str(v)) => {
                let value = v.clone();
                *pos += 1;
                if matches!(tokens.get(*pos).map(|t| &t.tok), Some(Tok::Open)) {
                    // `kind name { ... }`
                    *pos += 1;
                    let mut block = Block::new(&key, Some(&value));
                    block.line = line;
                    parse_items(tokens, pos, &mut block, depth + 1)?;
                    expect_close(tokens, pos, line)?;
                    into.items.push(Item::Block(block));
                } else {
                    into.items.push(Item::Field {
                        name: key,
                        value,
                        line,
                    });
                }
            }
            _ => return Err(format!("line {line}: `{key}` needs a value or a block")),
        }
    }
    Ok(())
}

fn expect_close(tokens: &[Lexed], pos: &mut usize, opened_at: usize) -> Result<(), String> {
    match tokens.get(*pos) {
        Some(Lexed {
            tok: Tok::Close, ..
        }) => {
            *pos += 1;
            Ok(())
        }
        _ => Err(format!("line {opened_at}: this block is never closed")),
    }
}

// ---- Writing ----

/// True if `value` can be written as a bare word.
fn bare(value: &str) -> bool {
    !value.is_empty() && value.chars().all(is_word_char)
}

pub fn quote(value: &str) -> String {
    if bare(value) {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Writes a block tree back out as text that [`parse`] reads as the same tree.
pub fn write(root: &Block) -> String {
    let mut out = String::new();
    for item in &root.items {
        write_item(item, 0, &mut out);
    }
    out
}

fn write_item(item: &Item, depth: usize, out: &mut String) {
    let pad = "    ".repeat(depth);
    match item {
        Item::Field { name, value, .. } => {
            let _ = writeln!(out, "{pad}{name} {}", quote(value));
        }
        Item::Raw { kind, text, .. } => {
            let _ = writeln!(out, "{pad}{kind} {{");
            // Keep the source as written (relative indentation included), pushed in under the block.
            for l in text.lines() {
                if l.trim().is_empty() {
                    out.push('\n');
                } else {
                    let _ = writeln!(out, "{pad}    {}", l.trim_end());
                }
            }
            let _ = writeln!(out, "{pad}}}");
        }
        Item::Block(b) => {
            let name = b
                .name
                .as_ref()
                .map(|n| format!(" {}", quote(n)))
                .unwrap_or_default();
            // A block of only simple fields fits on one line.
            let simple = !b.items.is_empty()
                && b.items.iter().all(|i| matches!(i, Item::Field { .. }))
                && b.items.len() <= 14
                && b.kind != "fighter"
                && b.kind != "ruleset"
                && b.kind != "bundle";
            if simple {
                let fields: Vec<String> = b
                    .items
                    .iter()
                    .filter_map(|i| match i {
                        Item::Field { name, value, .. } => Some(format!("{name} {}", quote(value))),
                        _ => None,
                    })
                    .collect();
                let _ = writeln!(out, "{pad}{}{name} {{ {} }}", b.kind, fields.join(" "));
            } else {
                let _ = writeln!(out, "{pad}{}{name} {{", b.kind);
                for child in &b.items {
                    write_item(child, depth + 1, out);
                }
                let _ = writeln!(out, "{pad}}}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fields_blocks_and_names() {
        let t = parse("# hi\nname \"A B\"\nmove jab { total 18 hitbox { x 1.5 y -2 } }\nbare { }")
            .unwrap();
        assert_eq!(t.items.len(), 3);
        match &t.items[1] {
            Item::Block(b) => {
                assert_eq!(b.kind, "move");
                assert_eq!(b.name.as_deref(), Some("jab"));
                assert_eq!(b.items.len(), 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn script_blocks_are_raw_and_may_contain_braces() {
        let src =
            "move m { script {\n  if frame == 1 { end(); } // a } in a comment\n  turn();\n} }";
        let t = parse(src).unwrap();
        let Item::Block(m) = &t.items[0] else {
            panic!()
        };
        let Item::Raw { text, .. } = &m.items[0] else {
            panic!()
        };
        assert!(text.contains("if frame == 1 { end(); }"));
        assert!(text.contains("turn();"));
    }

    #[test]
    fn errors_name_the_line() {
        for (src, line) in [
            ("a 1\nb {\n c 1", 2),
            ("a \"x", 1),
            ("a 1\n\n}", 3),
            ("x\n", 1),
            ("a 1\n$", 2),
        ] {
            let e = parse(src).unwrap_err();
            assert!(e.starts_with(&format!("line {line}:")), "{src:?}: {e}");
        }
    }

    #[test]
    fn writing_then_parsing_gives_the_same_tree() {
        let src = "bundle { name \"My Pack\" }\nweapon w {\n move jab { total 1 hitbox { x 1 } script {\n if hit { end(); }\n } }\n}\n";
        let a = parse(src).unwrap();
        let b = parse(&write(&a)).unwrap();
        assert_eq!(strip(&a), strip(&b));
    }

    /// Line numbers differ between the two texts; compare structure only.
    fn strip(b: &Block) -> String {
        let mut c = b.clone();
        fn wipe(b: &mut Block) {
            b.line = 0;
            for i in &mut b.items {
                match i {
                    Item::Field { line, .. } => *line = 0,
                    Item::Raw { line, text, .. } => {
                        *line = 0;
                        *text = text
                            .lines()
                            .map(str::trim)
                            .collect::<Vec<_>>()
                            .join("\n")
                            .trim()
                            .to_string();
                    }
                    Item::Block(inner) => wipe(inner),
                }
            }
        }
        wipe(&mut c);
        format!("{c:?}")
    }

    #[test]
    fn quoting_round_trips() {
        for v in [
            "plain",
            "with space",
            "quote\"d",
            "back\\slash",
            "",
            "1.5",
            "-3",
        ] {
            let src = format!("k {}", quote(v));
            let t = parse(&src).unwrap();
            let Item::Field { value, .. } = &t.items[0] else {
                panic!()
            };
            assert_eq!(value, v);
        }
    }

    #[test]
    fn deep_nesting_and_huge_input_are_refused() {
        let deep = "a { ".repeat(40) + &"}".repeat(40);
        assert!(parse(&deep).unwrap_err().contains("nested too deeply"));
        assert!(parse(&"a 1\n".repeat(MAX_FILE_BYTES / 4 + 10)).is_err());
    }

    #[test]
    fn garbage_never_panics() {
        let mut x: u32 = 5;
        let alphabet: Vec<char> = "ab1 .-{}\"#\n\\x_".chars().collect();
        for _ in 0..5000 {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let len = (x >> 24) as usize % 50;
            let s: String = (0..len)
                .map(|_| {
                    x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    alphabet[(x >> 16) as usize % alphabet.len()]
                })
                .collect();
            let _ = parse(&s);
        }
    }
}
