//! `pftool content-*`: export, check and pack content bundles.

use sim_content::Bundle;
use sim_core::moves::MoveId;
use sim_core::{Content, SIM_VERSION};
use std::fs;

/// Reads, parses and validates a bundle file, reporting every problem at once.
pub fn load_file(path: &str) -> Result<Bundle, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let bundle = sim_content::load(&text).map_err(|e| format!("{path}:\n  {}", e.join("\n  ")))?;
    sim_content::validate(&bundle.content)
        .map_err(|e| format!("{path} is not valid content:\n  {}", e.join("\n  ")))?;
    Ok(bundle)
}

fn describe(bundle: &Bundle) {
    let m = &bundle.manifest;
    let c = &bundle.content;
    println!(
        "{}  (schema {}, sim version {}, hash {:016x}, {})",
        if m.name.is_empty() {
            "(unnamed)"
        } else {
            &m.name
        },
        m.schema,
        m.sim_version
            .map_or("not stated".to_string(), |v| v.to_string()),
        c.hash(),
        if bundle.verified {
            "hash verified"
        } else {
            "loose: no hash in the file"
        }
    );
    for (i, f) in c.fighters.iter().enumerate() {
        println!(
            "  fighter {:<12} weapon {:<12} weight {}",
            c.names.fighters[i],
            c.names
                .weapons
                .get(usize::from(f.weapon))
                .map_or("?", String::as_str),
            f.weight.raw() >> 16
        );
    }
    for (i, w) in c.weapons.iter().enumerate() {
        let moves = w.moves.iter().filter(|m| !m.is_empty()).count();
        let scripted: Vec<&str> = w
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| m.script.is_some() || m.projectile_script.is_some())
            .map(|(n, _)| MoveId::from_index(n as u8).key())
            .collect();
        println!(
            "  weapon  {:<12} {moves} moves{}",
            c.names.weapons[i],
            if scripted.is_empty() {
                String::new()
            } else {
                format!(", scripts in: {}", scripted.join(", "))
            }
        );
    }
    println!("  stage   {}", c.names.stage);
}

/// `pftool content-export <out.pfc> [name]`: writes the built-in roster as a packed bundle.
pub fn cmd_export(args: &[String]) -> Result<(), String> {
    let Some(out) = args.first() else {
        return Err("usage: pftool content-export <out.pfc> [bundle name]".to_string());
    };
    let name = args.get(1).map_or("Base Roster", String::as_str);
    let text = sim_content::to_text(
        &Content::placeholder(),
        name,
        "",
        "The built-in roster: a longsword duelist and a close-range brawler.",
        true,
    );
    fs::write(out, text).map_err(|e| format!("cannot write {out}: {e}"))?;
    println!("wrote {out} for sim version {SIM_VERSION}");
    Ok(())
}

/// `pftool content-check <file.pfc>`: loads and validates, printing a summary or every problem.
pub fn cmd_check(args: &[String]) -> Result<(), String> {
    let [path] = args else {
        return Err("usage: pftool content-check <file.pfc>".to_string());
    };
    let bundle = load_file(path)?;
    describe(&bundle);
    println!("ok");
    Ok(())
}

/// `pftool content-pack <in.pfc> <out.pfc>`: validates, then writes the bundle in canonical form with a fresh
/// hash and this build's sim version. Comments in the input are not carried over, so pack to a different file.
pub fn cmd_pack(args: &[String]) -> Result<(), String> {
    let [input, output] = args else {
        return Err("usage: pftool content-pack <in.pfc> <out.pfc>".to_string());
    };
    if input == output {
        return Err(
            "pack to a different file: packing rewrites the text and drops comments".to_string(),
        );
    }
    let bundle = load_file(input)?;
    let m = &bundle.manifest;
    let text = sim_content::to_text(&bundle.content, &m.name, &m.author, &m.description, true);
    fs::write(output, text).map_err(|e| format!("cannot write {output}: {e}"))?;
    println!(
        "packed {output}  (hash {:016x}, sim version {SIM_VERSION})",
        bundle.content.hash()
    );
    Ok(())
}
