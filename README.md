# Platform Fighter (working title)

An original, legally distinct platform fighter with rollback netcode, a character creator and a stage creator.
Full plan: [`docs/Platform_Fighter_Project_Plan.docx`](docs/Platform_Fighter_Project_Plan.docx).
Art style target: [`docs/ART_DIRECTION.md`](docs/ART_DIRECTION.md). Weapons and movesets: [`docs/MOVESETS.md`](docs/MOVESETS.md).

## Status: Phase 1 complete (movement), playable in Godot

| Crate | Purpose | State |
| --- | --- | --- |
| `sim-core` | Deterministic sim: fixed point, trig, `GameState`, `step`, checksum, movement state machine | Phase 1 movement complete, see `docs/PHASE1_MOVEMENT.md` |
| `sim-content` | Content validation and balance guardrails | Basic validator |
| `sim-script` | Integer-only scripting VM | Stub (Phase 5) |
| `netplay` | Rollback layer | Local rollback harness (Phase 2 proof); transport is Phase 4 |
| `tools` | `pftool`: replay generator/runner, CI checksum dump, rollback fuzzer | Working |
| `godot-bridge` | gdext `SimRunner` node: ticks the sim, exposes read-only state | Working |
| `godot/` | Godot 4.7 project: blob fighters, stage, training overlay | Playable test bed |

## Play it

**Double-click `play.bat`** in the project folder. It builds the simulation and opens the game.

From a terminal you must be in the project folder, and in PowerShell a program in the current folder needs `.` in front:

```powershell
cd "C:\Users\jared\OneDrive\Desktop\Platform Fighter PC"
.\play.bat
```

`tools/godot/` is git-ignored. Download Godot 4.7.x and put it there, or open `godot/` in your own Godot.

- **P1** WASD stick, Ctrl = gentle tilt (walk), Space jump, J attack, K special, L or Shift shield.
- **P2** Arrows, Backslash = tilt, Enter jump, comma attack, period special, slash shield.
- **Gamepads** left stick, A/Y jump, X attack, B special, bumpers or triggers shield.
- **Training keys** F1 overlay, F2 ECB diamonds, P pause, `.` step forward, `,` step back, R restart.

Try a wavedash (tap jump, then shield with the stick held down-diagonal), shield + down on a platform, or
falling near a ledge. Scripted screenshot demos: `Godot --path godot -- --demo=wavedash --shots=<folder>`
(names: wavedash, ledge, shielddrop, tour, portrait).

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
