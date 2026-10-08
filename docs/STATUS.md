# Project status and how to resume

Read this first when picking the project up again. The design source of truth is `docs/Platform_Fighter_Project_Plan.docx`;
this file records where the build is against its roadmap (section 8) and what to do next. Last updated after the
Phase 7 work so far: match rules, HUD, results, rematches and the online screen (sim version 23).

## Roadmap progress

| Phase | State | Where to read |
| --- | --- | --- |
| 0 Foundation (fixed point, state, checksum, headless runner) | done | `README.md`, `CLAUDE.md` |
| 1 Movement | done | `docs/PHASE1_MOVEMENT.md` |
| 2 Local rollback | done (1000 randomised runs, 0 mismatches) | `netplay/src/local_rollback.rs` |
| 3 Combat (hits, DI, shields, grabs, techs, training mode) | done | `docs/COMBAT.md` |
| 4 Netplay (rollback, UDP, relay, handshake, desync detection) | built and tested locally; **never tried over the real internet** | `docs/NETPLAY.md` |
| 5 Content format and scripting VM | done | `docs/CONTENT.md` |
| 6 Editors (look, fighter, move, stage, package) | done, **untested by a human creator** | `docs/EDITORS.md` |
| (extra) Both characters' kits completed | done (estimates flagged in `docs/COMBAT.md`) | `docs/COMBAT.md` |
| (extra) Main menu, character creator, character select | done | `docs/MENUS.md` |
| (extra) Online play with made fighters, hitbox size scaling, point budget | done | `docs/MENUS.md`, `docs/NETPLAY.md` |
| 7 Game loop: stocks, winner, HUD, results, rematch (local and online), online screen | done, see `docs/MATCHES.md` | `docs/MATCHES.md` |
| 7 Game loop: replays (record, verify, watch) | done, see `docs/MATCHES.md` | `docs/MATCHES.md` |
| 7 Game loop: key rebinding, controller input and menus | done (controllers untested on hardware) | `docs/MENUS.md` |
| 7 Game loop: local 3 to 4 player free-for-alls | done (needs controllers 3 and 4; untested on hardware) | `docs/MATCHES.md` |
| 7 Game loop: stage select (4 stages, online and replays) | done | `docs/MATCHES.md` |
| 7 Game loop: online 3 to 4 players, spectating | **next** | see below |
| 8 Content and polish (real art/animation, audio, balance tooling, moderation) | later | Blender models were offered by the user |

## What "done" is verified by

`cargo test --workspace` (about 480 tests), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`,
`pftool fuzz-rollback 1000` and `pftool net-fuzz 2000` (both 0 mismatches), and the headless Godot tests
(`godot/tests/*.gd`, run with `Godot --headless --path godot --script res://tests/<name>.gd`):
`input_e2e`, `net_e2e`, `content_e2e`, `loadout_test`, `editor_api_test`, `editor_ui_test`, `creator_flow_test`, `replay_test`, `controls_test`,
`online_fighters_test`, `match_flow_test`, `online_flow_test`. All of them pass at the last commit. Godot is expected at `tools/godot/Godot_v4.7.1-stable_win64_console.exe`
(not in git); Rust stable with the MSVC build tools builds the rest. `play.bat` opens the game, `play_editor.bat` the editors,
`play_host.bat` / `play_join.bat` an online match.

## Known gaps and honest caveats (do not lose these)

* **Real-internet netplay is untested.** Everything is localhost or simulated. A friend playtest is pending (see the memory note
  "netplay playtest pending"). No NAT traversal: port-forward UDP 47000, or run `pftool net-relay`.
* **Nobody has used the menus or editors by hand.** Tests drive them, screenshots look right, but feel is unproven.
* **Frame data**: many hitbox positions, throw knockbacks and landing lags are estimates (listed in `docs/COMBAT.md`). Pivot grabs
  were left for the user to test later.
* **Online has a form, not a lobby.** The online screen (menu, Online) picks role, connection, fighter, delay and (host) rules, but there
  is no list of games or matchmaking. Nobody has used it over a real network.
* Controllers (play and menus) are implemented but untested on real hardware. The art is the procedural blob; real models come later.
* One stage, one mode (versus; local up to 4 players, online 2). No sudden death after a tied clock.

## Phase 7: what is left, suggested order

Done: stage select, stocks, elimination, winner, time limit, HUD, results, local and online rematch, online screen, replays (`docs/MATCHES.md`), key
rebinding and controller input/menus (`docs/MENUS.md`; the controller part is untested on hardware).

1. **Online 3 to 4 players**: local free-for-alls work (`docs/MATCHES.md`); `Session` has an `active` mask and the sim handles 4, but the
   handshake and `Peer` are 1v1. Validate 1v1 over the internet first.
2. **Spectating**: a spectator consumes the confirmed-input stream (the same data a replay records) from a host or relay.
3. Optional polish: sudden death, a list-of-games lobby or matchmaking, results statistics, replay sharing.

## Where things live

* Sim: `sim-core` (state, step, combat, movement, grabs, scripting hosts), `sim-script` (VM), `sim-content` (text format, bundles,
  validation, recipes), `netplay` (wire format, handshake, rollback session, `Peer`), `transport` (UDP and relay: the only sockets),
  `tools` (`pftool`), `godot-bridge` (`SimRunner`, `ContentEditor`).
* Game: `godot/` (`scripts/` the match, `ui/` menus, `editor/` editors, `tests/` headless tests). Content: `content/base.pfc`
  (must equal the built-in roster in code; a test enforces it; re-export with `pftool content-export`).
* Rules for agents and contributors: `CLAUDE.md` (determinism rules, bump `SIM_VERSION` on any behaviour change, the content/script
  conventions, the editors and menus rules).
