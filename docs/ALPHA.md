# Public alpha: what is ready and what is not

The plan's Phase 8 exit criterion is "public alpha-ready". This is an honest checklist as of sim v24: it separates what is built and tested here
from what only a person on real hardware and a real network (or a publisher) can settle.

## Ready (built and tested in this repository)

* **The game**: movement, combat, two movesets and a character creator with a point budget, four stages, stocks and results, local 2 to 4 players,
  online 1v1 with rollback (direct or relay), online groups of 3 to 4 through a hub host, rematches, spectating, replays, controls and controller input, sound effects, rigged animated
  fighters. The simulation is deterministic and fuzzed: `cargo test --workspace` (about 520 tests), `pftool net-fuzz`, `pftool fuzz-rollback`,
  and 14 headless Godot tests (`docs/STATUS.md`).
* **Tooling**: `pftool` (replays, balance bot matches, content check and pack), the in-game editors, a content policy with name moderation
  (`docs/CONTENT_POLICY.md`), balance reports (`docs/BALANCE.md`).
* **Packaging scaffolding**: `godot/export_presets.cfg` (Windows) and `build_release.bat`, which builds the release bridge, copies the shipped
  content next to the executable and exports the game when Godot's export templates are installed. The game finds its content in the repository's
  `content` folder, next to the executable, or inside the project (`Roster.content_path`).

## Not verified (needs a person, a machine or a decision)

* **A packaged build has never been made here**: Godot's export templates are not installed on this machine, so the export, the bridge DLL path
  inside the exported folder and the runtime-loaded `.glb` models in a packed build are untested. Install the templates and run
  `build_release.bat`; expect to fix small path issues.
* **Real-internet netplay** (direct with a forwarded port, and through `pftool net-relay` on a public server) has only run on localhost and
  simulated lossy links. There is no NAT traversal and no public relay: a publisher must run one.
* **Controllers** (play and menus) and **the new screens** have not been used by a person. Sound effects have not been heard by one.
* **Balance**: the bot reports are a smoke detector only; the brawler is not yet judged against the sword by anyone who plays well.
* **Cross-platform determinism** is designed for (integer math, no floats in the sim). `.github/workflows/ci.yml` builds and tests on Windows, Linux, macOS (ARM)
  and Linux ARM and diffs the `pftool selftest` checksums across them, but that workflow has not been seen to run (no CI result has been checked), so only
  Windows has actually run the game's code.

## Not built yet

* **A list of games, skill-based matchmaking, regions**: Quick match exists (the relay pairs whoever is waiting, `docs/MATCHES.md`), but it is first-come only and
  needs someone to run and publish a relay; nobody has hosted one on the internet yet.
* **More archetypes** beyond the three classes (longsword, claws, maul), per-move animation clips beyond the three attack types, and *composed* music (the
  game has synthesised placeholder tracks that nobody has listened to).
* **Reporting and moderation for shared content** (needs an account or a server to act on; the name filter in the policy document is all there is),
  a privacy policy, terms of service and an age rating. The groundwork is written: what the game stores and sends is in `docs/DATA_FLOW.md`, and every
  dependency's licence is inventoried in `docs/THIRD_PARTY.md` (godot-rust is MPL-2.0; no font file is bundled yet).
* **An installer, auto-update and crash reporting.**
