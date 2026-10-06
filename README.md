# Platform Fighter (working title)

An original, legally distinct platform fighter with rollback netcode, a character creator and a stage creator.
Full plan: [`docs/Platform_Fighter_Project_Plan.docx`](docs/Platform_Fighter_Project_Plan.docx).
Art style target: [`docs/ART_DIRECTION.md`](docs/ART_DIRECTION.md).

## Status: Phase 0, Foundation

| Crate | Purpose | State |
| --- | --- | --- |
| `sim-core` | Deterministic sim: 16.16 fixed point, trig tables, `GameState`, `step`, checksum, snapshots | Working skeleton with placeholder run/jump/fall |
| `sim-content` | Content validation and balance guardrails | Basic validator |
| `sim-script` | Integer-only scripting VM | Stub (Phase 5) |
| `netplay` | Rollback layer | Local rollback harness (Phase 2 proof); transport is Phase 4 |
| `tools` | `pftool`: replay generator/runner, CI checksum dump, rollback fuzzer | Working |
| `godot-bridge` | gdext nodes that render sim state | Not started (Week 4) |

## Commands

Requires Rust (stable) plus the MSVC build tools on Windows.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p sim-core
cargo run --release -p tools --bin pftool -- selftest
cargo run --release -p tools --bin pftool -- fuzz-rollback 1000
```

## Determinism rules (enforced)

- No float arithmetic in sim crates (`clippy::float_arithmetic = deny`).
- No `HashMap`/`HashSet` (`clippy.toml`). Use arrays or `BTreeMap`.
- No unsafe code in sim crates.
- CI replays the same inputs on Windows, Linux, macOS and ARM and fails if any checksum differs.
