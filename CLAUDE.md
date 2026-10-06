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

## Workflow
- `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` must pass; run `cargo fmt --all`.
- Art: round blob bodies + 2D faces + cosmetic accessories, see `docs/ART_DIRECTION.md`. Cosmetics never touch the sim.
