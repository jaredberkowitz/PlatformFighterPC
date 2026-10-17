# Platform Fighter (working title)

An original, legally distinct platform fighter with rollback netcode, a character creator and a stage creator.
Full plan: [`docs/Platform_Fighter_Project_Plan.docx`](docs/Platform_Fighter_Project_Plan.docx).
Art style target: [`docs/ART_DIRECTION.md`](docs/ART_DIRECTION.md). Weapons and movesets: [`docs/MOVESETS.md`](docs/MOVESETS.md). Combat: [`docs/COMBAT.md`](docs/COMBAT.md). Netplay: [`docs/NETPLAY.md`](docs/NETPLAY.md). Content format and scripting: [`docs/CONTENT.md`](docs/CONTENT.md). Editors: [`docs/EDITORS.md`](docs/EDITORS.md). Menus and the character creator: [`docs/MENUS.md`](docs/MENUS.md). **Where the project stands and what to do next: [`docs/STATUS.md`](docs/STATUS.md).**

**PROJECT DISCLAIMER: THIS IS OBVIOUSLY VERY HEAVILY CONTRIBUTED TO BY CLAUDE, AND AI IN GENERAL. THE PURPOSE OF THIS PROJECT IS NOT TO LEARN TO CODE, BUT TO TEST AI IN A DEVELOPMENT CAPACITY.**

## Getting started (teammates and playtesters)

### Just play (no tools needed)
1. Download the newest zip from the [Releases page](https://github.com/jaredberkowitz/PlatformFighterPC/releases) (the one marked
   *Latest*).
2. **Right-click the zip, choose "Extract All"**, then open the extracted folder. Running the game from inside the zip does not
   work: Windows only unpacks the .exe, without the game data next to it.
3. Double-click `PlatformFighter.exe`. If Windows says "Windows protected your PC", click **More info**, then **Run anyway** (the
   game is not code-signed).
4. Controls, what is new, and what to do if it will not start: `How to play.txt` in the same folder.

Needs Windows 10 or 11 (64-bit) and a graphics card with Vulkan or DirectX 12 (most from the last ten years; update its driver if
the game will not start).

### Work on the game (the source code)
1. Install [Git](https://git-scm.com/download/win), then clone: `git clone https://github.com/jaredberkowitz/PlatformFighterPC.git`
   (or use GitHub Desktop). Avoid a OneDrive-synced folder if you can: syncing can lock files mid-build.
2. **Double-click `setup.bat`.** It checks for Rust, Microsoft's C++ build tools and Godot 4.7.1, offers to install whatever is
   missing (with winget, or it tells you where to get it), builds the simulation and imports the game's assets. The first run takes
   a while (the C++ build tools are a few GB); later runs take seconds.
3. Then **`play.bat`** plays, **`play_editor.bat`** opens the content editors, and **`play_host.bat`** / **`play_join.bat`** play
   online. Each one re-runs the quick checks and rebuilds what changed.
4. To open the project in the Godot editor itself: `tools\godot\Godot_v4.7.1-stable_win64.exe --path godot -e` (after setup has
   built the simulation; Godot cannot load the game without it).

If something fails, copy the red text from the window and send it to the team.

## Status: Phases 0-3 built, Phase 4 (netplay) built and tested locally, Phase 5 (content format and scripting) built, Phase 6 (editors) built, playable in Godot

| Crate | Purpose | State |
| --- | --- | --- |
| `sim-core` | Deterministic sim: fixed point, trig, `GameState`, `step`, checksum, movement state machine | Phase 1 movement complete, see `docs/PHASE1_MOVEMENT.md` |
| `sim-content` | Text format, bundles with manifest/schema/hash, migrations, validation | Phase 5, see `docs/CONTENT.md` |
| `sim-script` | Integer-only scripting VM: compiler, bounded interpreter, whitelisted API | Phase 5, see `docs/CONTENT.md` |
| `netplay` | Rollback layer: wire format, handshake, session, `Peer` | Phase 4, see `docs/NETPLAY.md` |
| `transport` | UDP link and relay server (the only crate with sockets) | Phase 4 |
| `tools` | `pftool`: replay generator/runner, CI checksum dump, rollback and network fuzzers, host/join/relay | Working |
| `godot-bridge` | gdext `SimRunner` node: ticks the sim, exposes read-only state | Working |
| `godot/` | Godot 4.7 project: blob fighters, stage, training overlay, and the editors (`play_editor.bat`) | Playable test bed |

## Play it

The game opens on a main menu: **Play** (character select), **Character Creator** (make and save your own fighter, with stats
like size that change how it moves) and **Editors**. Details in `docs/MENUS.md`.


**Double-click `play.bat`** in the project folder (run `setup.bat` once first; see Getting started). It builds the simulation and
opens the game.

From a terminal you must be in the project folder, and in PowerShell a program in the current folder needs `.` in front:

```powershell
cd "C:\Users\jared\OneDrive\Desktop\Platform Fighter PC"
.\play.bat
```

`tools/godot/` is git-ignored: `setup.bat` downloads Godot 4.7.1 into it.

- **P1** WASD stick, Space jump, N short hop, J attack, I smash attack, K special, L or Shift shield, Ctrl = slow walk.
- **P2** Arrows, Enter jump, apostrophe short hop, comma attack, semicolon smash, period special, slash shield, backslash = slow walk.
- **Walk or dash** A first tap of a direction walks (it starts gently and builds up); a double tap dashes, and holding after a dash runs. Tapping the other way soon after a dash is a dash dance, but a run skids to a stop before turning.
- **Defence** Shield (L or Shift) blocks; flick a direction in the shield to roll, press down for a spot dodge, press attack for a shield grab. U grabs: attack to pummel, a direction to throw; mash buttons to escape a grab.
- **Gamepads** left stick (analog, so a light push walks), A/Y jump, B attack, X special, bumpers or triggers shield.
- **Combat** J attack: tap with the stick neutral for a jab, hold a direction first for a tilt, or press the smash key (I) with a direction for a smash. In the air the stick picks the aerial. K special: up for the sword fighter's rising slash, neutral for the brawler's blaster.
- **Training keys** F1 overlay, F2 ECB diamonds, F3 hitboxes and hurtboxes, F6 +25% damage to P2, F7 reset damage, F8 face-to-face, P pause, `.` step forward, `,` step back, R restart.

Try a wavedash (tap jump, then shield with the stick held down-diagonal), shield + down on a platform, or
falling near a ledge. Scripted screenshot demos: `Godot --path godot -- --demo=wavedash --shots=<folder>`
(names: wavedash, ledge, shielddrop, tour, portrait, combat, smash, and the move demos in `docs/COMBAT.md`).

## Play online (Phase 4)

Two players, each on their own PC: one double-clicks **`play_host.bat`** and tells the other their IP address (the window lists
it) and port 47000; the other double-clicks **`play_join.bat`** and types `ip:port`. Both play with player 1's keys on their own
keyboard. The host is player 1 (the sword fighter) and the joiner player 2 (the claw fighter). The top of the screen shows the
connection: frame, confirmed frame, rollbacks and stalls ("waiting for the other player" if the link is slow).

- **Same network or a VPN:** works as is.
- **Over the internet:** the host must forward UDP port 47000 to their PC (NAT hole punching is not built yet), **or** both use a
  relay server someone runs with `pftool net-relay 47000` on a machine with a public address:
  `Godot --path godot -- --host=0 --relay=RELAY_IP:47000 --room=1234` and `--join=- --relay=RELAY_IP:47000 --room=1234`.
- Both PCs must run the same build. A different sim version or content is refused before the match starts, with both numbers shown.
- Testing without a second PC: run `play_host.bat`, then open a second window with `Godot --path godot -- --join=127.0.0.1:47000`.

## Commands

Requires Rust (stable) plus the MSVC build tools on Windows (`setup.bat` installs both).

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p sim-core
cargo run --release -p tools --bin pftool -- selftest
cargo run --release -p tools --bin pftool -- fuzz-rollback 1000
cargo run --release -p tools --bin pftool -- net-fuzz 1000    # randomised lossy network sessions
cargo run -p tools -- content-check content/base.pfc           # load + validate a content bundle
```

The game reads its roster from `content/base.pfc` (`--content=PATH` after `--` picks another bundle). To change a
fighter, edit the file and run `content-check`; see `docs/CONTENT.md`.

## Determinism rules (enforced)

- No float arithmetic in sim crates (`clippy::float_arithmetic = deny`).
- No `HashMap`/`HashSet` (`clippy.toml`). Use arrays or `BTreeMap`.
- No unsafe code in sim crates.
- CI replays the same inputs on Windows, Linux, macOS and ARM and fails if any checksum differs.
