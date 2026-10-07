//! Content bundles: one text file holding a whole roster (rules, movesets, fighters, a stage) plus a manifest.
//!
//! ```text
//! bundle {
//!     schema 1                 # version of this file format; older files are migrated when loaded
//!     name "Base Roster"
//!     sim_version 19           # the simulation this content was packed for
//!     hash 1a2b3c4d5e6f7a8b    # fingerprint of the content; checked on load when present
//! }
//! ruleset { ... }   weapon "..." { ... }   fighter "..." { ... }   stage "..." { ... }
//! ```
//!
//! # Versioning policy
//!
//! * `schema` is the version of the *file format*. It goes up when a field is renamed, removed or changes meaning.
//!   Adding an optional field does not need a new schema. Every schema bump ships with a migration in
//!   [`MIGRATIONS`] that rewrites an older file's tree into the next version, so old content keeps loading.
//!   A file with a *newer* schema than this build understands is refused with a clear message.
//! * `sim_version` is the simulation's behaviour version. Content packed for another sim version is refused:
//!   the same numbers can play differently, and both players in a match must agree anyway.
//! * `hash` fingerprints everything the simulation reads. A packed bundle whose hash does not match was edited
//!   after packing (or damaged); a file without a hash is "loose" and is accepted as written, for hand-editing.
//!   Matches compare the hash of the loaded content (not the file), so formatting and comments never matter.
//! * Replays store the sim version and content hash they were recorded with and are only played back against
//!   the same pair; older sims are kept for old replays only if a release chooses to ship them.

use crate::format::{read_content, write_content, SECTIONS};
use crate::tree::{self, Block, Item};
use sim_core::{Content, SIM_VERSION};

/// The file format version this build writes and understands.
pub const SCHEMA_VERSION: u32 = 1;

/// Rewrites a file's tree from schema `from` to `from + 1`.
pub type Migration = (u32, fn(&mut Block));

/// Every migration, oldest first. Empty while schema 1 is the only schema there has been.
pub const MIGRATIONS: &[Migration] = &[];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub schema: u32,
    pub name: String,
    pub author: String,
    pub description: String,
    pub sim_version: Option<u16>,
    pub hash: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bundle {
    pub manifest: Manifest,
    pub content: Content,
    /// True if the file carried a hash and the content matched it.
    pub verified: bool,
}

/// Loads a bundle, applying this build's migrations.
pub fn load(text: &str) -> Result<Bundle, Vec<String>> {
    load_with(text, MIGRATIONS, SCHEMA_VERSION)
}

/// [`load`] with an explicit migration list and current schema (so migrations can be tested).
pub fn load_with(
    text: &str,
    migrations: &[Migration],
    current: u32,
) -> Result<Bundle, Vec<String>> {
    let mut root = tree::parse(text).map_err(|e| vec![e])?;

    let found = bundle_blocks(&root);
    let Some(bundle) = found.first() else {
        return Err(vec![
            "the file has no `bundle` section (it should start with `bundle { schema 1 ... }`)"
                .to_string(),
        ]);
    };
    if found.len() > 1 {
        return Err(vec![format!(
            "line {}: the file has more than one `bundle` section",
            found[1].line
        )]);
    }
    let schema = field(bundle, "schema")
        .and_then(|(v, _)| v.parse::<u32>().ok())
        .ok_or_else(|| {
            vec![format!(
                "line {}: `bundle` needs `schema` (a whole number)",
                bundle.line
            )]
        })?;

    if schema > current {
        return Err(vec![format!(
            "this content uses schema {schema}, but this build only understands up to schema {current}; update the game"
        )]);
    }
    for from in schema..current {
        let Some((_, apply)) = migrations.iter().find(|(f, _)| *f == from) else {
            return Err(vec![format!(
                "this content uses schema {schema}, and there is no migration from schema {from} to {}",
                from + 1
            )]);
        };
        apply(&mut root);
    }
    if schema < current {
        set_field(&mut root, "schema", &current.to_string());
    }

    let mut errors = Vec::new();
    let manifest = read_manifest(&root, &mut errors);
    let content = read_content(&root, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }

    let actual = content.hash();
    let mut verified = false;
    if let Some(packed_for) = manifest.sim_version {
        if packed_for != SIM_VERSION {
            return Err(vec![format!(
                "this content was packed for sim version {packed_for}, but this build is sim version {SIM_VERSION}; \
                 re-export or re-pack it with this build"
            )]);
        }
    }
    if let Some(hash) = manifest.hash {
        if hash != actual {
            return Err(vec![format!(
                "the content hash {hash:016x} in the bundle does not match what it contains ({actual:016x}): \
                 the file was edited after it was packed (re-pack it), or is damaged"
            )]);
        }
        verified = true;
    }
    Ok(Bundle {
        manifest,
        content,
        verified,
    })
}

/// Text for `content` with a manifest. `pack` is `true` for a distributable bundle (hash included).
pub fn to_text(
    content: &Content,
    name: &str,
    author: &str,
    description: &str,
    pack: bool,
) -> String {
    let mut manifest = Block::new("bundle", None);
    manifest.field("schema", SCHEMA_VERSION.to_string());
    manifest.field("name", name);
    if !author.is_empty() {
        manifest.field("author", author);
    }
    if !description.is_empty() {
        manifest.field("description", description);
    }
    manifest.field("sim_version", SIM_VERSION.to_string());
    if pack {
        manifest.field("hash", format!("{:016x}", content.hash()));
    }
    let mut root = Block::new("file", None);
    root.push(manifest);
    for section in write_content(content) {
        root.push(section);
    }
    let mut text = String::from(
        "# Platform fighter content bundle. Edit freely; run `pftool content-check` to validate it and\n\
         # `pftool content-pack` to stamp it with a fresh hash.\n\n",
    );
    // Blank line between top-level sections for readability.
    let body = tree::write(&root);
    let mut previous_closed = false;
    for line in body.lines() {
        let top_level = !line.starts_with(' ') && !line.is_empty();
        if top_level && previous_closed {
            text.push('\n');
        }
        text.push_str(line);
        text.push('\n');
        previous_closed = line == "}";
    }
    text
}

fn bundle_blocks(root: &Block) -> Vec<&Block> {
    root.items
        .iter()
        .filter_map(|i| match i {
            Item::Block(b) if b.kind == "bundle" => Some(b),
            _ => None,
        })
        .collect()
}

fn field<'a>(block: &'a Block, name: &str) -> Option<(&'a str, usize)> {
    block.items.iter().find_map(|i| match i {
        Item::Field {
            name: n,
            value,
            line,
        } if n == name => Some((value.as_str(), *line)),
        _ => None,
    })
}

fn set_field(root: &mut Block, name: &str, value: &str) {
    for item in &mut root.items {
        if let Item::Block(b) = item {
            if b.kind == "bundle" {
                for f in &mut b.items {
                    if let Item::Field {
                        name: n, value: v, ..
                    } = f
                    {
                        if n == name {
                            *v = value.to_string();
                        }
                    }
                }
            }
        }
    }
}

fn read_manifest(root: &Block, errors: &mut Vec<String>) -> Manifest {
    let Some(b) = bundle_blocks(root).into_iter().next() else {
        errors.push("the file has no `bundle` section".to_string());
        return Manifest {
            schema: 0,
            name: String::new(),
            author: String::new(),
            description: String::new(),
            sim_version: None,
            hash: None,
        };
    };
    const KNOWN: [&str; 6] = [
        "schema",
        "name",
        "author",
        "description",
        "sim_version",
        "hash",
    ];
    for item in &b.items {
        match item {
            Item::Field { name, line, .. } if !KNOWN.contains(&name.as_str()) => {
                errors.push(format!(
                    "line {line}: unknown field `{name}` in the bundle manifest"
                ));
            }
            Item::Block(inner) => errors.push(format!(
                "line {}: the bundle manifest has no sections",
                inner.line
            )),
            _ => {}
        }
    }
    let text = |name: &str| {
        field(b, name)
            .map(|(v, _)| v.to_string())
            .unwrap_or_default()
    };
    let sim_version = field(b, "sim_version").and_then(|(v, line)| match v.parse::<u16>() {
        Ok(n) => Some(n),
        Err(_) => {
            errors.push(format!("line {line}: `sim_version` must be a whole number"));
            None
        }
    });
    let hash = field(b, "hash").and_then(|(v, line)| match u64::from_str_radix(v, 16) {
        Ok(n) => Some(n),
        Err(_) => {
            errors.push(format!("line {line}: `hash` must be a hexadecimal number"));
            None
        }
    });
    Manifest {
        schema: field(b, "schema")
            .and_then(|(v, _)| v.parse().ok())
            .unwrap_or(0),
        name: text("name"),
        author: text("author"),
        description: text("description"),
        sim_version,
        hash,
    }
}

/// The section kinds a bundle may contain, for documentation and editors.
pub fn section_kinds() -> &'static [&'static str] {
    &SECTIONS
}
