# Project status and how to resume

Read this first when picking the project up again. The design source of truth is `docs/Platform_Fighter_Project_Plan.docx`;
this file records where the build is against its roadmap (section 8) and what to do next. Last updated after the
Phase 7 work (complete): match rules, HUD, results, rematches, the online screen, replays, spectating, groups of 3 to 4 and Quick match; then music and two movement-tuning passes toward the reference game, `docs/PHASE1_MOVEMENT.md` (sim version 30: the 9-frame and hold input buffer, short-hop aerials, light and heavy landings, shield release, air dodge timings and the reference ledge rules; sim version 33: full ground speed and the committed initial dash; sim version 34: DI at the reference's 10 degrees, launch speed influence, hurtboxes lower in a crouch and on the claws' limbs, and the bruiser on the reference hammer heavyweight's attributes with four air jumps (`docs/FEEL_AUDIT.md`); sim version 35: less slippery (landing walks rather than runs, traction on overspeed and on the dash attack's slide) and the reverse aerial rush out of a dash or a run's skid (`docs/PHASE1_MOVEMENT.md`); sim version 36: running jumps take a reduced, capped share of the ground speed, bodies on the ground do not pass through each other (`sim-core/src/push.rs`), and the maul class is removed for now (two classes: longsword and claws); a basic computer opponent for testing and playing alone (`docs/CPU.md`; C in the character select, F5 in training); sim version 37: running off a platform takes speed into the air like a jump, and the claws' throws are the reference's hit-then-throw (combo throws, one kill throw); sim version 38: the jabs, dash attack, grabs, pummels and throws on the reference's published frame data (`ultimateframedata.com`; the aerials, tilts and smashes already matched); hit feel in `docs/COMBAT.md`, the feel audit in `docs/FEEL_AUDIT.md`, recorded CC0 hit sounds in `docs/AUDIO.md`). Since then: sim versions 39 to 44 (reference launches and fast falls, missed techs,
rolls stopping at edges, the special fall, up specials reversing with up-and-back, the ledge jump peaking the reference's height above
the stage, the get-up attacks on reference numbers); the base body with modelled clothes; menus in the concept art's painted style with
a bundled OFL font (`docs/MENUS.md`); a Mii-style character creator with faces built from placeable parts (`docs/ART_PIPELINE.md`); and
a visual quality pass on the menus (one light, depth, one card language). Then a rendering framework (`docs/VISUAL_STYLE.md`: shading
ramps, stage profiles, the look bench, display settings; an art pass with restrained grading, contact shading under props, deeper haze and
crisp cast shadows); B-reverse (sim 45); a 3-frame stick flick for dash and fast fall, so a controller's flick registers (sim 46); and the
shared animation library (`docs/ANIMATION.md`: one clip set for every moveset, mapped in `godot/scripts/anim_library.gd`, per-hit strikes
for multihit moves, a polished idle, jump, landing, jab, forward air and hit reaction). Then the multi-hit engine and a third class
(sim 49): eight hit groups per move, a rehit that clears chosen groups, the autolink angle 367, set knockback, extra hitstun, per-hitbox
SDI and landing hits; and the Needles rogue, whose body, tilts and aerials follow the reference's published data (`docs/sheik/`,
`docs/MOVESETS.md`). Sim 50: the rest of its normals, grabs and throws on reference data (its specials are still the claws kit's), and
three general rules brought to the reference's: the Sakurai angle, traction slowing grounded slides, and light hitstun landing on its
feet (no knockdown after a drag-down). Sim 51: its specials (Needle Storm, a pulling line of our own, Vanish, Bouncing Fish), the
pull angle (368) and a stored charge for scripts (`store`, `stored`). Sim 52: Needle Storm charges and stores as the reference, Bouncing
Fish's early kick. Sim 53 (from playtest notes): a dash takes a fuller flick (0.75 instead of 0.6, `STICK_DASH`) so a quick push
stays a walk; a run's stick rolled along the bottom or top turns it (the reference's reverse aerial rush motion), since down and
forward no longer crouches out of a run; and the guard held out in front is a hurtbox (`sim-core/tests/stick_feel.rs`). Sim 54
(content schema 3): every fighter's ground movement is its reference character's, from the parameter file: walk and run
acceleration by the stick, the rogue's 7-frame initial dash, heavy landings (`sim-core/tests/ground_attributes.rs`). Then the graphics fidelity pass (branch `graphics-fidelity`, `docs/GRAPHICS_FIDELITY_PLAN.md`, results and frame
times in `docs/GRAPHICS_FIDELITY_LOG.md`): painted facades and glowing windows on the skyline, per-theme value range and more contrast on
every stage, far scenery softly out of focus (Best), painted surface wear and hit scuffs, leafy tree crowns in the wind, a banded sea
with foam. Every phase measured within about 0.1 ms of GPU time of the baseline (Treetop Isle draws fewer calls than before); the look
bench is unchanged. Phase 7 (authored prop models) waits on the owner's choice of source. Then a visual overhaul (branch `visual-overhaul`,
`docs/VISUAL_OVERHAUL_PLAN.md`, audit and results in `docs/VISUAL_OVERHAUL_AUDIT.md` / `_LOG.md`): SMAA, per-material light responses
and a real blade, built stage props and girders, varied skyline buildings with signs, three hit tiers, HUD refinements (and cards that fit
four players), and a four-computer stress demo (`--demo=cpu4 --players=4`). Then an art-direction polish (branch `art-direction-polish`,
`docs/ART_DIRECTION_POLISH_LOG.md`): the attention hierarchy, palette and material rules written into `ART_DIRECTION.md`, calmer coral
girders, skyline buildings in varied muted hues with clustered windows and broken-up office strips, a quieter roof wall, and "FREE PLAY"
said once on the HUD. Then a new base body modelled by script after the concept sheet (`art/blender/make_base_body.py`,
`docs/ART_PIPELINE.md` twenty-eighth pass): the concept's proportions, bare by default (a new Gear slot), matte skin and a warmer, thinner
outline; it replaces the Tripo model, whose licence was unclear. Fighter motion polish is planned (`docs/MOTION_POLISH_PLAN.md`) but on
hold.

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
| 7 Game loop: spectating (direct hosts) | done | `docs/MATCHES.md` |
| 7 Game loop: online 3 to 4 players (hub host and lobby) | done, see `docs/MATCHES.md` | `docs/MATCHES.md` |
| 7 Game loop: matchmaking | **Quick match** done (relay queue pairs two waiting players); a list of games, ratings and regions are not built | `docs/MATCHES.md` |
| 8 Public alpha checklist | see `docs/ALPHA.md` (what is ready, unverified and not built) | `docs/ALPHA.md` |
| 8 Content and polish (real art/animation, audio, balance tooling, moderation) | **started**: rigged art and animation, soft cel shading and shared lighting, drawn SVG faces and cloth prints (`docs/ART_PIPELINE.md`), sound effects and synthesised music (`docs/AUDIO.md`), balance tooling (`docs/BALANCE.md`), content policy and name moderation (`docs/CONTENT_POLICY.md`) | `docs/ART_PIPELINE.md` |

## What "done" is verified by

`cargo test --workspace` (about 480 tests), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`,
`pftool fuzz-rollback 1000` and `pftool net-fuzz 2000` (both 0 mismatches), and the headless Godot tests
(`godot/tests/*.gd`, run with `Godot --headless --path godot --script res://tests/<name>.gd`):
`input_e2e`, `net_e2e`, `content_e2e`, `loadout_test`, `editor_api_test`, `editor_ui_test`, `creator_flow_test`, `replay_test`, `controls_test`, `sfx_test`, `fighter_view_test`, `policy_test`,
`online_fighters_test`, `match_flow_test`, `online_flow_test`, `cpu_test`. All of them pass at the last commit. Godot is expected at `tools/godot/Godot_v4.7.1-stable_win64_console.exe`
(not in git); Rust stable with the MSVC build tools builds the rest. `play.bat` opens the game, `play_editor.bat` the editors,
`play_host.bat` / `play_join.bat` an online match.

## Known gaps and honest caveats (do not lose these)

* **Real-internet netplay is untested.** Everything is localhost or simulated. A friend playtest is pending (see the memory note
  "netplay playtest pending"). No NAT traversal: port-forward UDP 47000, or run `pftool net-relay`.
* **Nobody has used the menus or editors by hand.** Tests drive them, screenshots look right, but feel is unproven.
* **Frame data**: many hitbox positions, throw knockbacks and landing lags are estimates (listed in `docs/COMBAT.md`). Pivot grabs
  were left for the user to test later.
* **Online has a form, not a game list.** The online screen (menu, Online) picks role, mode, connection, fighter, delay and (host) rules, and group matches have a
  lobby, plus Quick match through a relay, but there is no list of games. Nobody has used it over a real network.
* Controllers (play and menus) are implemented but untested on real hardware. The art is a rigged blob with a cel shader, shared lighting and drawn SVG faces and prints (`docs/ART_PIPELINE.md`); hats and glasses are still built in code.
* **Renderer**: Windows now uses Vulkan (Direct3D 12 stalled for milliseconds whenever a mesh was made during play; see the twenty-first
  pass in `docs/ART_PIPELINE.md`). Measure with `--perf`; the developer's laptop caps even an empty scene near 50 to 65 fps.
* Four stages, one mode (versus; local and online up to 4 players). No sudden death after a tied clock.

## Phase 7: what is left, suggested order

Done: online 3 to 4 players, stage select, stocks, elimination, winner, time limit, HUD, results, local and online rematch, online screen, replays (`docs/MATCHES.md`), key
rebinding and controller input/menus (`docs/MENUS.md`; the controller part is untested on hardware).

1. ~~Spectating~~ done for direct hosts (`docs/MATCHES.md`); through the relay it is still to do.
2. Optional polish: sudden death, a list-of-games lobby, results statistics, replay sharing.

## Where things live

* Sim: `sim-core` (state, step, combat, movement, grabs, scripting hosts), `sim-script` (VM), `sim-content` (text format, bundles,
  validation, recipes), `netplay` (wire format, handshake, rollback session, `Peer`), `transport` (UDP and relay: the only sockets),
  `tools` (`pftool`), `godot-bridge` (`SimRunner`, `ContentEditor`).
* Game: `godot/` (`scripts/` the match, `ui/` menus, `editor/` editors, `tests/` headless tests). Content: `content/base.pfc`
  (must equal the built-in roster in code; a test enforces it; re-export with `pftool content-export`).
* Rules for agents and contributors: `CLAUDE.md` (determinism rules, bump `SIM_VERSION` on any behaviour change, the content/script
  conventions, the editors and menus rules).
