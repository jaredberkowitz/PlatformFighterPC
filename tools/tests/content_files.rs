//! The content files shipped in the repository.

use sim_core::fuzz::random_inputs;
use sim_core::{step, Content, GameState, Rng};

fn read(relative: &str) -> String {
    let path = format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// `content/base.pfc` is what the game loads. While the built-in roster still exists in code, the two must be the
/// same thing. If this fails after you changed a number in code, re-export with
/// `cargo run -p tools -- content-export content/base.pfc "Base Roster"`; if you edited the file, copy the change
/// into the code (or retire the code version).
#[test]
fn the_shipped_base_bundle_is_the_built_in_roster() {
    let bundle =
        sim_content::load(&read("content/base.pfc")).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert!(bundle.verified, "the shipped bundle should carry a hash");
    assert_eq!(sim_content::validate(&bundle.content), Ok(()));
    assert_eq!(
        bundle.content,
        Content::placeholder(),
        "content/base.pfc differs from the built-in roster (see this test's comment)"
    );
}

#[test]
fn the_shipped_bundle_is_in_canonical_form() {
    let on_disk = read("content/base.pfc");
    let again = sim_content::to_text(
        &sim_content::load(&on_disk).unwrap().content,
        "Base Roster",
        "",
        "The built-in roster: a longsword duelist and a close-range brawler.",
        true,
    );
    assert_eq!(
        on_disk.replace("\r\n", "\n"),
        again,
        "re-export content/base.pfc"
    );
}

#[test]
fn playing_the_shipped_bundle_matches_playing_the_built_in_roster() {
    let file = sim_content::load(&read("content/base.pfc"))
        .unwrap()
        .content;
    let code = Content::placeholder();
    for seed in 0..4u64 {
        let inputs = random_inputs(&mut Rng::new(500 + seed), 3000);
        let mut a = GameState::new(&file, seed, [0, 1, 0, 1]);
        let mut b = GameState::new(&code, seed, [0, 1, 0, 1]);
        for (frame, i) in inputs.iter().enumerate() {
            step(&mut a, &file, i);
            step(&mut b, &code, i);
            assert_eq!(a.checksum(), b.checksum(), "seed {seed} frame {frame}");
        }
    }
}

/// The language reference must mention everything the whitelist offers, so the doc cannot drift from the code.
#[test]
fn the_content_doc_lists_every_script_name_and_move() {
    let doc = read("docs/CONTENT.md");
    for kind in [sim_script::Kind::Fighter, sim_script::Kind::Projectile] {
        for r in kind.registers() {
            assert!(
                doc.contains(&format!("`{}`", r.name)),
                "register `{}` is not in docs/CONTENT.md",
                r.name
            );
        }
        for f in kind.functions() {
            assert!(
                doc.contains(&format!("`{}(", f.name)),
                "function `{}` is not in docs/CONTENT.md",
                f.name
            );
        }
    }
    for m in sim_core::moves::MoveId::all() {
        assert!(
            doc.contains(m.key()),
            "move `{}` is not in docs/CONTENT.md",
            m.key()
        );
    }
}
