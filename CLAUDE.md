# Platform Fighter

Rollback-netcode 3D-on-2D platform fighter. The source of truth for design is `docs/Platform_Fighter_Project_Plan.docx`.
Roadmap phases and exit criteria are in section 8 of that plan; check which phase we're in before adding features.

## Hard rules for sim crates (`sim-core`, `sim-content`, `sim-script`, `netplay`)
- No floats, no `HashMap`/`HashSet`, no `unsafe`, no clocks/I/O/threads. Lints enforce this; don't `allow` them.
- All mutable game data lives in `GameState` and is hashed in `checksum()`. Adding a field means adding it to its `hash_into`
  and adding a test that fails if it's forgotten.
- Tunable numbers belong in content/params (`FighterParams`, `Stage`), never hard-coded in `step`.
- Changing sim behaviour: bump `SIM_VERSION`.
- Everything original: no Nintendo assets or names, don't use "Smash" anywhere.

## Networking
- `netplay` stays pure (no sockets/clocks/threads). Sockets live only in `transport`; the game and tools plug them in through `netplay::peer::Link`.
- Anything that changes the wire format or session rules must keep `pftool net-fuzz` at zero mismatches.

## Content and scripts
- The roster is data (`content/*.pfc`, format in `docs/CONTENT.md`). While the Rust builders in `sim-core/src/moves.rs` and
  `content.rs` still exist, `content/base.pfc` must equal them (a test enforces it): change one, re-export the other
  (`cargo run -p tools -- content-export content/base.pfc "Base Roster"`).
- Scripts (`sim-script`) are integer-only, bounded, and reach the game only through the whitelist in `sim-script/src/api.rs`,
  implemented in `sim-core/src/scripting.rs`. Script variables live in `Fighter::vars` / `Projectile::vars` and are hashed.
  A new script ability = a new whitelist entry + a test in `sim-core/tests/scripted_moves.rs` + the doc line (a test checks the doc).
- Bump `SCHEMA_VERSION` (and add a migration) when the file format changes meaning; bump `SIM_VERSION` when behaviour changes.

## Editors
- The editors (`godot/editor/`, docs in `docs/EDITORS.md`) have no rules of their own: they edit a content tree through
  `ContentEditor` (bridge) and Rust re-reads and validates the whole document. Put new rules in `sim-content`, not in GDScript.
- Cosmetic loadouts (`godot/scripts/loadout.gd`) are presentation only and never reach the sim; keep each catalog item within the mesh budget.

## Workflow
- `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` must pass; run `cargo fmt --all`.
- Art: round blob bodies + 2D faces + cosmetic accessories, see `docs/ART_DIRECTION.md`. Cosmetics never touch the sim.
