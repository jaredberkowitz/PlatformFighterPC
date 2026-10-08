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
* **Cross-platform determinism** is designed for (integer math, no floats in the sim) and gated by CI in the plan, but only Windows has ever
  run it. Linux, macOS and ARM builds, and the CI that compares checksums across them, are not set up.

## Not built yet

* **Matchmaking and a list of games**: needs a server. Today friends share an address, or a relay and a room number.
* **A third moveset ("remaining archetypes")**, per-move animation clips beyond the three attack types, and real music.
* **Reporting and moderation for shared content** (see the policy document), a privacy policy, terms of service, age rating and licences for the
  tools used (Godot is MIT; check godot-rust, Blender output and every dependency in `Cargo.lock` before shipping).
* **An installer, auto-update and crash reporting.**
