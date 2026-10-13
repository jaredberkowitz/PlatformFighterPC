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
three general rules brought to the reference's: the 361 angle, traction slowing grounded slides, and light hitstun landing on its
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
outline; it replaces the Tripo model, whose licence was unclear. Then magic-hour painted skies per stage, a golden raking key and
violet shadows on Meadow, straw hay bales and a built fence, and drifting motes and light shafts (`docs/ART_PIPELINE.md` twenty-ninth pass). Then the fighter motion polish (`docs/MOTION_POLISH_PLAN.md`, results in
its "Results" section; branch `motion-polish`): strike stretch, impact squash into an eased launch stretch, smear ribbons on punches and
kicks, a spring set for the chest, head, hat, fist and neckwear, cloth tails on spring chains, new clips (tech, platform drop, run stop,
heavy hitstun, an idle fidget), a fix for the head spinning in a long shield hold, and a short-contact run that no longer skates.

Moveset ports from the reference game (`docs/movesets/ROSTER_STATUS.md`): **sim 57**, the fourth class, Embers on the tumbler body
(modelled on the reference game's Mario), phase 1: the body and every normal on reference data (`docs/movesets/mario/`), with early jab
continuation (`Move::next_from`), angled forward tilts and smashes (`Hitbox::aim`) and set-weight hits (`Hitbox::set_weight`); its
specials are still the claws' until phase 2. **Sims 58 to 64** (phase 2, branch `movesets`): its four specials on reference data
(the rising punch, the bouncing fireball, the cape, the water pump) and the general pieces they needed: per-move recovery rules (ledge
windows by side and frame, the special fall's drift and landing lag per move, height multipliers, stick angling, a move's own air
physics, later or air-only intangibility, carrying on after touching down), set-knockback launches keeping plain launch physics,
projectile arcs, ground bounces and hits by age, reversing hits, push hits, fuller reflectors (vertical reversal, lifetime, damage limit,
projectile detector), once-per-airtime script flags, charged scaled shots, a 16-slot projectile pool, pummels with their own hitlag on
the reference's timing, and wall jumps (content schema 4). The Longsword's and the Claws' up specials were re-fitted to their
references with the new rules. **Sim 65** (owner's follow-up): a full water-pump charge holds its stance until special or shield, and
jump pressed against a wall (stick neutral or toward it) also wall-jumps without spending a midair jump. Details and what is still
estimated: `docs/movesets/mario/IMPLEMENTATION_REPORT.md`.
**Sim 66** (content schema 5): the fifth class, Gauntlets on the sprinter body (built-in fighter Sprinter, modelled on the reference
game's Captain Falcon), phase 1: the body (15-frame initial dash, wall jump 1.4 / 3.1) and every normal on reference data
(`docs/movesets/captain_falcon/`): the jab chain with the rapid jab's entry rule (three presses and two releases since jab 1, presses in
hitlag counted), its loop and finisher, the one-frame knee, the down air meteor, angled forward tilt and smash. General pieces: sixteen
`ext` slots (E17, schema 5), per-limb intangibility (E19, `Move::part_intangible`), per-hitbox shieldstun (E24, `Hitbox::shieldstun`)
and the `presses` / `releases` script registers. Its specials are still the claws' until phase 2 (plan in
`docs/movesets/captain_falcon/IMPLEMENTATION_REPORT.md`).
**Sim 67** (branch `seer-phase1`, no schema change): the sixth class, Staff on the seer body (built-in fighter Seer, modelled on the
reference game's Palutena), phase 1: the body (no wall jump, heavy landing 6 from the parameter file) and every normal on reference data
(`docs/movesets/palutena/`): jab 1 into the rapid jab (two presses and releases, or held on a hit), the multi-hit up tilt, neutral air
and up air with their last hits, the smashes' wind as pushes, the up smash's pillar. General pieces: angle types (E26,
`Hitbox::angle_type`), applied to the older classes too (owner decision 21: each change in `sim-core/tests/angle_types.rs`), and a
signed hitstun modifier (E27). Its specials are still the claws' until phase 2 (plan in
`docs/movesets/palutena/IMPLEMENTATION_REPORT.md`).
**Sims 67 to 71** (branch `falcon-phase2`, merged into `movesets` at **sim 72**): the Gauntlets' four specials on reference data, with
the reference's exact shield stun for every fighter (whole-number stuns were a frame short; the changed moves are listed in the
Gauntlets' report), armor (E1), frame-ranged air physics with fast-fall and turn frames (E11 additions), search boxes (E21), command
grabs (E6), no-clank hits (E3c), off-edge versions (E22) and the `wall` script register. **Sim 73** (branch `cleanup-72`): the
reference's angle type 1 (from the direction of travel) on the Claws' up special loop, every Gauntlets special's angle type, the Staff's
no-clank boxes, the Longsword's, Claws' and Needles' get-up attacks at the reference's angle 48, and a second source for the Sprinter's
42-frame neutral air dodge (owner decisions 18 and 23 to 27). The Gauntlets are now **playable** (travel distances [E]); details in
`docs/movesets/captain_falcon/IMPLEMENTATION_REPORT.md`.
**Sims 74 to 77** (branch `seer-phase2`, from `movesets` at sim 72 with the sim 73 cleanup merged in; no schema change): the Staff's
four specials on reference data (`docs/movesets/palutena/IMPLEMENTATION_REPORT.md`, tests `sim-core/tests/staff_specials.rs`): the
warp (74), the counter and reflect barrier (75, with E10c: melee-only counters with a damage cap, a projectile detector that starts
the reflection move, a standing-capsule reflector with a body field, `air_max_rise`, counters catching through their own
intangibility), the explosive flame (76, with E3a: piercing, per-target rehit and eight phases for projectiles, the `flick` register
and `spawn_at`), and the reticle (77, the `lock(range)` target lock with a line-of-sight test, `collision::segment_blocked`). The
Staff is now **playable**; the open [E] readings are listed in its report.
**Sim 78** (branch `counter-fix`, owner decisions 28 and 29; no schema change): counters never catch a push (a windbox hit), and a
counter catches on its own bubble (`Counter::bubble`, the reference's counter bubble) instead of the body's hurtboxes: the Longsword's
radius 10.5 at 10 up, the Staff's radius 11.5 at 10 up and 1.5 ahead [R]. The Longsword's answer is also capped at the reference's
50%, and a grab or a push no longer reaches through a counter stance's intangibility (only the hits the counter catches do). Tests in
`sim-core/tests/kits.rs` and `staff_specials.rs`; details in `docs/movesets/SHARED_MECHANICS.md` ("Counter").

**Sim 79** (branch `grab-dash`; 78 is the moveset session's counter fix): a dash flicked at any point of a grab, pummel or throw
comes out on the first frame the fighter can act if the stick is still held (`Fighter::dash_buffer`), so throws can be chased; the
reference buffers a dash only in the last 6 or 7 frames (owner's call to be more lenient). Tests `sim-core/tests/grab_dash.rs`.
**Sim 82** (branch `dash-buffer-landing`, on `movesets` at sim 80): landing lag keeps a dash the same way, and the keyboard sends a
direction key pressed during grab, throw or landing lag as a flick (a first press was otherwise a walk, so keyboard players could not
buffer a dash at all); tests in `grab_dash.rs`, `landing_buffer.rs` and `godot/tests/input_e2e.gd`.
**Sim 89** (branch `clutch-phase1`, on main at 64147b0 (sim 87; 88 is reserved for another session), content schema 6 unchanged): the ninth class, Clutch on the bounder body (built-in fighter Bounder, modelled on the reference game's Yoshi, owner decision 45), phase 1: the body (weight 104, the reference's ground, air, jump and dodge numbers, the 51.56-unit double jump) and every normal on [R] data. Engine, all general and off by default: the E1 **knockback armor kind** (`Armor::subtract`, `Armor::min_kb`, `withstands` sees the knockback) and **E33**, a body's air-jump armor and turnaround (`FighterParams::air_jump_armor`, `air_jump_turns`, hashed `Fighter::air_jump_frame`; optional fighter fields, no schema bump). Tests `sim-core/tests/clutch.rs`, `multihit.rs`; report `docs/movesets/yoshi/IMPLEMENTATION_REPORT.md`.

**Sim 86** (branch `bracers-phase1`, on `movesets` at f2b69b8, content schema 6; the number is a placeholder the owner renumbers at the merge): the eighth class, Bracers on the hauler body (built-in fighter Hauler, modelled on the reference game's Donkey Kong, owner decision 37), phase 1: the body (weight 127, the reference's ground, air, jump and dodge numbers, a bigger body [E]) and every normal on the raw hitbox tables' data, with two small general extensions: E20 trips (`Hitbox::trip`, a percent rolled on the game's own hashed generator only for a hit that can trip; the tripped fighter sits with the reference's hard-trip timings: `Fighter::tripped`, `Ruleset::trip_lag` / `trip_max` / `trip_invuln`) and E3g hits that clank without rebounding (`Hitbox::rebounds`). Older fighters changed by the reference data: the Gauntlets' forward air late hit now trips (20%), the Longsword's down tilt carries its 35% (it lifts, so it never trips on a flat stage), and the Claws' up smash and the Staff's forward, up and down tilts and neutral air clank without rebounding. The specials are still the claws' placeholders and the forward throw a placeholder throw. Report and the phase 2 / 3 plans: `docs/movesets/donkey_kong/IMPLEMENTATION_REPORT.md`. Tests `sim-core/tests/bracers.rs`, `trips.rs`.

**Sim 85** (branch `greatsword-phase2`, on `movesets` at a88c09f, content schema 6; sim 84 is reserved for the engine session): the Greatsword's phase 2, its four normal specials (the wave, the combo slash, the rising and falling slash, the gauge charge) and the engine they need: E9, a body gauge with a timed powered mode that multiplies the body's movement (the champion's Boost gauge; gain from damage dealt and taken, drain while charging, reset on a KO), E16 `boost` / `boosting` / `grab` / `boost_add`, E11 `start_speed_y` and `ledge_drop`, E22b `on_land`, E3f standing capsule hitboxes. Read-only bridge accessors for the HUD: `fighter_gauge`, `fighter_gauge_max`, `fighter_gauge_full`, `fighter_gauge_frames`. Every other fighter unchanged. Report: `docs/movesets/cloud/IMPLEMENTATION_REPORT.md`; phase 3 (the powered versions, E3e, E28) next.

**Sim 83** (branch `greatsword-phase1`, on `movesets` at sim 82; no schema change): the seventh class, Greatsword on the champion body
(built-in fighter Champion, modelled on the reference game's Cloud, owner decision 30), phase 1: the body (weight 100, wall jump 1.3 /
2.4, heavy landing 3) and every normal on the reference's data read from ssbwiki.com's raw hitbox tables (`docs/movesets/cloud/`): the
jab chain's continuable windows and held repeat, the forward smash's three set-weight hits, the up air as one hit, the forward and down
air meteors, hit-then-throw throws. No engine change. Its specials are still the claws' until phases 2 and 3 (the Boost gauge E9,
E3e, E3f, E11 and E16 additions, E22b, E28: plan in `docs/movesets/cloud/IMPLEMENTATION_REPORT.md`). Tests `sim-core/tests/greatsword.rs`.

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
