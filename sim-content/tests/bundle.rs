//! The content format: round trips, error reporting, versioning, and the Phase 5 exit criterion (a fighter
//! defined in data plays identically to its hand-coded version).

use sim_content::bundle::{load_with, Migration};
use sim_content::tree::{Block, Item};
use sim_content::{load, to_text, validate};
use sim_core::fuzz::random_inputs;
use sim_core::moves::MoveId;
use sim_core::{step, Content, Fx, GameState, Rng, SIM_VERSION};
use sim_script::{Kind, Program};

fn text(content: &Content, pack: bool) -> String {
    to_text(content, "Test Pack", "tester", "for tests", pack)
}

fn errors(text: &str) -> Vec<String> {
    load(text).unwrap_err()
}

fn has(errors: &[String], needle: &str) -> bool {
    errors.iter().any(|e| e.contains(needle))
}

#[test]
fn the_hand_coded_roster_round_trips_exactly() {
    let content = Content::placeholder();
    let bundle = load(&text(&content, true)).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert!(bundle.verified);
    assert_eq!(
        bundle.content, content,
        "loaded content differs from the hand-coded one"
    );
    assert_eq!(bundle.content.hash(), content.hash());
    assert_eq!(bundle.manifest.name, "Test Pack");
    assert_eq!(bundle.manifest.author, "tester");
    assert_eq!(bundle.manifest.sim_version, Some(SIM_VERSION));
}

#[test]
fn writing_is_stable() {
    let content = Content::placeholder();
    let once = text(&content, true);
    let twice = text(&load(&once).unwrap().content, true);
    assert_eq!(once, twice);
}

#[test]
fn loose_bundles_have_no_hash_and_still_load() {
    let content = Content::placeholder();
    let t = text(&content, false);
    assert!(
        !t.lines().any(|l| l.trim_start().starts_with("hash ")),
        "a loose bundle carries no hash field"
    );
    let bundle = load(&t).unwrap();
    assert!(!bundle.verified);
    assert_eq!(bundle.content, content);
}

#[test]
fn a_loaded_roster_is_valid() {
    let bundle = load(&text(&Content::placeholder(), true)).unwrap();
    assert_eq!(validate(&bundle.content), Ok(()));
}

/// The Phase 5 exit criterion: content that went through the text format plays exactly like the original.
#[test]
fn data_defined_content_plays_identically() {
    let hand = Content::placeholder();
    let data = load(&text(&hand, true)).unwrap().content;
    for seed in 0..6u64 {
        let inputs = random_inputs(&mut Rng::new(100 + seed), 2400);
        let mut a = GameState::new(&hand, seed, [0, 1, 0, 1]);
        let mut b = GameState::new(&data, seed, [0, 1, 0, 1]);
        for (frame, i) in inputs.iter().enumerate() {
            step(&mut a, &hand, i);
            step(&mut b, &data, i);
            assert_eq!(a.checksum(), b.checksum(), "seed {seed}, frame {frame}");
        }
    }
}

#[test]
fn scripts_survive_the_text_format() {
    let mut c = Content::placeholder();
    let mv = &mut c.weapons[0].moves[MoveId::NSpecial as usize];
    mv.total_frames = 40;
    mv.script = Some(
        Program::compile(
            Kind::Fighter,
            "var n;\n  n += 1; // count\n  if frame >= 3 && frame <= 20 {\n    set_vel(stick_x * 0.4, 0);\n  }\n  if n == 25 { goto(0); }\n",
        )
        .unwrap(),
    );
    c.weapons[0].moves[MoveId::NSpecial as usize].projectile =
        c.weapons[1].moves[MoveId::NSpecial as usize].projectile;
    c.weapons[0].moves[MoveId::NSpecial as usize].projectile_script =
        Some(Program::compile(Kind::Projectile, "if age > 30 { kill(); }").unwrap());
    let bundle = load(&text(&c, true)).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert_eq!(bundle.content, c);
    let loaded = bundle.content.weapons[0].moves[MoveId::NSpecial as usize]
        .script
        .as_ref()
        .unwrap();
    assert_eq!(loaded.source.lines().count(), 6, "{}", loaded.source);
    assert_eq!(bundle.content.hash(), c.hash());
}

#[test]
fn script_mistakes_are_reported_with_the_file_line() {
    let c = Content::placeholder();
    let t = text(&c, false).replacen(
        "move neutral_special {",
        "move neutral_special {\n        script {\n            set_vel(1);\n        }",
        1,
    );
    // The brawler already has a neutral special; the sword character does not, so add one to the sword instead.
    let t = if t.contains("set_vel(1);") {
        t
    } else {
        text(&c, false).replacen(
            "weapon longsword {",
            "weapon longsword {\n    move neutral_special {\n        total_frames 20\n        script {\n            set_vel(1);\n        }\n    }",
            1,
        )
    };
    let e = errors(&t);
    assert!(has(&e, "takes 2"), "{e:?}");
    assert!(e.iter().any(|m| m.starts_with("line ")), "{e:?}");
}

#[test]
fn every_problem_in_a_file_is_reported_at_once() {
    let c = Content::placeholder();
    let t = text(&c, false)
        .replacen("walk_speed ", "walk_spead ", 1)
        .replacen("move jab {", "move jabb {", 1)
        .replacen("weapon claws", "weapon clawz", 1)
        .replacen("gravity ", "# gravity ", 1);
    let e = errors(&t);
    assert!(has(&e, "unknown field `walk_spead`"), "{e:?}");
    assert!(has(&e, "missing `walk_speed`"), "{e:?}");
    assert!(has(&e, "`jabb` is not a move name"), "{e:?}");
    assert!(
        has(&e, "uses weapon `claws`, which does not exist"),
        "{e:?}"
    );
    assert!(has(&e, "missing `gravity`"), "{e:?}");
    assert!(e.len() >= 5);
}

#[test]
fn bad_values_say_what_was_expected() {
    let c = Content::placeholder();
    let t = text(&c, false)
        .replacen("jump_squat_frames 3", "jump_squat_frames lots", 1)
        .replacen("pass_through true", "pass_through maybe", 1);
    let e = errors(&t);
    assert!(
        has(&e, "`jump_squat_frames`") && has(&e, "whole number"),
        "{e:?}"
    );
    assert!(has(&e, "pass_through") && has(&e, "true or false"), "{e:?}");
}

#[test]
fn damage_and_ranges_are_validations_not_parse_errors() {
    let c = Content::placeholder();
    let mut done = false;
    let t: String = text(&c, false)
        .lines()
        .map(|l| {
            if !done && l.trim_start().starts_with("gravity ") {
                done = true;
                "    gravity 0".to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let loaded = load(&t).unwrap();
    assert!(validate(&loaded.content).is_err());
}

#[test]
fn tampering_with_a_packed_bundle_is_detected() {
    let t = text(&Content::placeholder(), true);
    let tampered = t.replacen("walk_speed ", "walk_speed 9", 1);
    let e = errors(&tampered);
    assert!(has(&e, "does not match"), "{e:?}");
    // The same edit to a loose file is fine: nothing to compare with.
    let loose = text(&Content::placeholder(), false).replacen("walk_speed ", "walk_speed 9", 1);
    assert!(load(&loose).is_ok());
}

#[test]
fn comments_and_blank_lines_do_not_change_the_hash() {
    let t = text(&Content::placeholder(), true);
    let noisy = format!(
        "# extra comment\n\n\n{}\n# trailing\n",
        t.replace('\n', "\n\n")
    );
    assert!(load(&noisy).is_ok());
}

#[test]
fn content_for_another_sim_version_is_refused() {
    let t = text(&Content::placeholder(), false).replacen(
        &format!("sim_version {SIM_VERSION}"),
        &format!("sim_version {}", SIM_VERSION + 1),
        1,
    );
    let e = errors(&t);
    assert!(has(&e, "packed for sim version"), "{e:?}");
}

#[test]
fn newer_schemas_are_refused_and_missing_manifests_explained() {
    let t = text(&Content::placeholder(), false).replacen("schema 1", "schema 99", 1);
    assert!(has(&errors(&t), "update the game"));
    let no_bundle = "ruleset { damage_mult 1 }";
    assert!(has(&errors(no_bundle), "no `bundle` section"));
    let no_schema = "bundle { name x }";
    assert!(has(&errors(no_schema), "needs `schema`"));
}

/// An example migration: schema 1 called `gravity` by the old name `grav`.
fn rename_grav(root: &mut Block) {
    fn visit(b: &mut Block) {
        for item in &mut b.items {
            match item {
                Item::Field { name, .. } if name == "grav" => *name = "gravity".to_string(),
                Item::Block(inner) => visit(inner),
                _ => {}
            }
        }
    }
    visit(root);
}

#[test]
fn old_files_are_migrated_on_load() {
    let migrations: &[Migration] = &[(1, rename_grav)];
    // An old (schema 1) file that still says `grav`.
    let old = text(&Content::placeholder(), false).replace("gravity ", "grav ");
    assert!(has(
        &load_with(&old, &[], 2).unwrap_err(),
        "no migration from schema 1 to 2"
    ));
    let bundle = load_with(&old, migrations, 2).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert_eq!(bundle.content, Content::placeholder());
    assert_eq!(
        bundle.manifest.schema, 2,
        "the manifest now says the current schema"
    );
}

#[test]
fn migrations_chain_in_order() {
    fn add_note(root: &mut Block) {
        for item in &mut root.items {
            if let Item::Block(b) = item {
                if b.kind == "bundle" {
                    for f in &mut b.items {
                        if let Item::Field { name, value, .. } = f {
                            if name == "description" {
                                *value = "migrated twice".to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    let migrations: &[Migration] = &[(1, rename_grav), (2, add_note)];
    let old = text(&Content::placeholder(), false).replace("gravity ", "grav ");
    let bundle = load_with(&old, migrations, 3).unwrap();
    assert_eq!(bundle.manifest.schema, 3);
    assert!(bundle.manifest.description.contains("migrated twice"));
}

#[test]
fn a_custom_fighter_can_inherit_and_override() {
    let base = text(&Content::placeholder(), false);
    let extra = "\nfighter sprinter {\n    inherit brawler\n    walk_speed 3\n}\nweapon quickclaws {\n    inherit claws\n    move jab {\n        total_frames 10\n        hitbox { start 2 end 3 x 1 y 1 radius 1 damage 2 angle 361 bkb 10 kbg 10 }\n    }\n}\n";
    let c = load(&format!("{base}{extra}"))
        .unwrap_or_else(|e| panic!("{}", e.join("\n")))
        .content;
    assert_eq!(c.fighters.len(), 3);
    assert_eq!(c.names.fighters[2], "sprinter");
    let brawler = c.fighters[1];
    let sprinter = c.fighters[2];
    assert_eq!(sprinter.walk_speed, Fx::from_int(3));
    assert_eq!(
        sprinter.gravity, brawler.gravity,
        "unlisted values come from the inherited fighter"
    );
    assert_eq!(sprinter.weapon, brawler.weapon);
    assert_eq!(c.weapons.len(), 3);
    assert_eq!(c.weapons[2].moves[MoveId::Jab as usize].total_frames, 10);
    assert_eq!(
        c.weapons[2].moves[MoveId::FTilt as usize],
        c.weapons[1].moves[MoveId::FTilt as usize],
        "moves not mentioned are inherited"
    );
    assert_eq!(validate(&c), Ok(()));
}

#[test]
fn inheriting_from_something_unknown_is_an_error() {
    let t = format!(
        "{}\nfighter x {{ inherit nobody }}\n",
        text(&Content::placeholder(), false)
    );
    assert!(has(&errors(&t), "inherits from `nobody`"));
}

#[test]
fn duplicate_names_and_stray_content_are_errors() {
    let t = text(&Content::placeholder(), false);
    let twice = format!("{t}\nweapon claws {{ }}\n");
    assert!(has(&errors(&twice), "defined twice"));
    let stray = format!("{t}\nlooseword 1\n");
    assert!(has(&errors(&stray), "not inside any section"));
    let unknown = format!("{t}\nmystery {{ }}\n");
    assert!(has(&errors(&unknown), "unknown section `mystery`"));
}

#[test]
fn garbage_input_never_panics_and_always_errors_cleanly() {
    let good = text(&Content::placeholder(), true);
    let mut x: u32 = 17;
    for round in 0..300 {
        // Delete or scramble a random slice of a real file.
        let mut bytes: Vec<char> = good.chars().collect();
        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let at = (x >> 8) as usize % bytes.len();
        let len = ((x >> 4) as usize % 40).min(bytes.len() - at);
        if round % 2 == 0 {
            bytes.drain(at..at + len);
        } else {
            for c in &mut bytes[at..at + len] {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                *c = ['{', '}', '"', '9', 'x', ' ', '-'][(x >> 16) as usize % 7];
            }
        }
        let damaged: String = bytes.into_iter().collect();
        let _ = load(&damaged);
    }
}
