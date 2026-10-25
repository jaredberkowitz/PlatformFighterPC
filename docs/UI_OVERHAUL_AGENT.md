# UI overhaul agent: plan and brief

Give this file to a new agent to start a UI refinement pass. It follows the workflow that took the base character from prototype
quality to polished (see `docs/art/characters/CHARACTER_ANALYSIS.md` for that pass's record): reference first, a measured audit,
then one visible defect at a time, each fix shown before and after.

## Role

You are a senior game UI/UX artist and Godot 4 UI engineer. Your one job is to refine the game's existing menus and HUD from
prototype quality to polished, commercially presentable indie/AA quality. This is an original 3D-on-2D platform fighter (never use
"Smash" or any Nintendo names or assets). The look is sleek but cartoony: flat colour, thick ink edges, a slight forward slant, bold
condensed type, a little bounce. Aim for a confident, readable, cohesive interface, not more decoration.

## Scope

**In scope:** every screen and its pieces, through the shared kit:
- `godot/ui/`: `ui_draw.gd` (palette, drawing), `ui_kit.gd` (widgets, motion), `menu_art.gd` (painted cards).
- The menu scripts: `menu.gd`, `select.gd`, `creator.gd`, `controls.gd`, `online.gd`, `net_lobby.gd`, `replays.gd` and `pause_menu.gd`.
- The results screens: `results.gd`, `run_results.gd`.
- The HUDs: `match_hud.gd`, `run_hud.gd`.
- The scenes `godot/menu.tscn`, `select.tscn`, `creator.tscn`, `controls.tscn`, `online.tscn` and `replays.tscn`.
- UI fonts, icons and SVG art under `godot/fonts/` and `godot/art/`.

**Out of scope** (recommend, never change): the simulation and Rust crates, netcode and transport, gameplay, the character models,
stages, the content format, the editors' rules, and project-wide rendering settings. Menu and select previews show fighters through
`godot/ui/preview.gd`. Frame them, but do not change the fighters.

**Hard rules** (from `CLAUDE.md`):
- Menu art stays drawn, original and cartoony.
- Never compute physics from stats in GDScript.
- Cosmetics never touch the sim.
- `docs/UI_DESIGN_SYSTEM.md` is the rulebook: colours by token name, never a raw `Color(...)` in a screen. Change the system there
  first, then apply it.

## Phase 0: set up (before touching anything)

1. **Branch.** Create a branch (e.g. `ui/overhaul`). If the working tree has uncommitted work, commit it as a checkpoint on that
   branch first, so every later diff is yours alone.
2. **Read** `docs/UI_DESIGN_SYSTEM.md`, `docs/UI_REVIEW.md` (the last review's findings; check which are done) and `docs/MENUS.md`.
   Also skim `docs/STATUS.md`.
3. **Ask the owner for references.** These are screenshots of the UI they want to get close to (menus, select, HUD, results), to put
   in `docs/art/reference_ui/`. The character pass only moved fast once there were concrete references. Use them as a target for
   quality and clarity, never as a layout to copy.
4. **Baseline captures.** Photograph every screen in the same window size (1280x720, plus 1920x1080 for scaling), with the commands
   below. Keep them as `baseline/`.

## Capture tools (all need a window, so not `--headless`)

Godot is at `tools/godot/Godot_v4.7.1-stable_win64_console.exe`; run from the repository root with `--path godot`.

| What | Command |
|---|---|
| Any scene, after a wait | a small `SceneTree` script that loads a `.tscn`, waits ~3 s and saves `root.get_viewport().get_texture().get_image()` (write it in your scratch folder; the character pass used one called `scene_shot.gd`) |
| Match HUD | `--script res://tests/hud_shot.gd -- --out=<folder>` |
| Pause | `--script res://tests/pause_shot.gd -- --out=<folder>` |
| Online lobby | `--script res://tests/lobby_shot.gd -- --out=<folder>` |
| Roster / select cards | `--script res://tests/roster_shot.gd -- --out=<folder>` |
| Run mode HUD | `--script res://tests/run_shot.gd -- --out=<folder>` |
| A match, fixed frames | `-- --demo=moves --shots=<folder>` (add `--noui` to hide the HUD, `--match` to show only the match HUD); `--demo=results --stocks=1 --match` for the results screen |

Read each script's header for its options. To compare screens, write small scratch scripts that crop and stack
before/after images side by side with Godot's `Image` API (no Python imaging library is installed). Mark them with a red strip
for before and green for after.

## Phase 1: audit (inspect only, edit nothing)

Return to the owner:
1. Every screen with its baseline capture, its scene and script, and which kit pieces it uses.
2. **A measured reference table** (`docs/art/ui/REFERENCE.md`), comparing the references and our screens the same way:
   - title, body and caption type sizes as a share of the screen height;
   - margins and spacing;
   - button heights;
   - how many distinct colours, edge treatments and type styles each screen uses;
   - contrast of text against its panel;
   - how many separate elements the eye has to read.

   Measuring beats judging by eye. In the character pass the eyeball estimate was wrong by 35% until it was measured.
3. **The five to eight most visible weaknesses, ranked.** For each: what it is, the evidence (capture and measurement), the cause in
   code, the proposed fix, and its risk. Keep confirmed problems apart from guesses.
4. A proposed order. Systemic first (palette, type scale, panel material, spacing grid in the kit), then per screen. Polish (motion,
   sound, small details) comes last.

Wait for the owner's approval before changing anything significant.

## Phase 2: refine, one visible defect at a time

For each item:
1. **Reproduce.** Capture the problem in a fixed setup: same screen, window size, focus state and wait time.
2. **Find the cause.** Read the drawing code before changing numbers. In the character pass, half the "tuning" problems were bugs:
   - a sign error made shapes wider instead of narrower;
   - a garment was classified after it was puffed out;
   - a texture was too coarse.
3. **Fix it in the shared kit when it is shared.** Only fix a single screen when the problem is local to it.
4. **Render after, in the identical setup.** Compose a before/after image into `docs/art/ui/<pass>.png`.
5. **Check the neighbours.** Kit changes reach every screen. Re-capture all screens, not just the one fixed, and check both window
   sizes, gamepad focus (`pad_nav.gd`) and the keyboard. Check the long text cases too: long fighter names, 4 players, and online
   states.
6. **Run the tests:**
   - Godot, `--headless --path godot --script res://tests/<name>.gd`: `select_grid_test`, `controls_test`, `pause_test`,
     `creator_flow_test`, `editor_ui_test`, `match_flow_test`, `online_flow_test` and `input_e2e`.
   - Also `cargo test --workspace` if anything outside `godot/` changed.
   - Before blaming yourself for a failure, check whether it also fails on the base branch. `creator_flow_test` already fails on main
     (`unknown field ledge_exposed_frames in the ruleset`, an older built bridge).
7. **Log it.** Add a short entry to `docs/art/ui/UI_ANALYSIS.md`: defect, cause, fix, files, what was checked, and what is left.
8. **Commit** with a message that says what changed and why, naming the before/after image.

Rules that worked:
- **Mock before you change a design decision.** When a fix changes the look (a new layout, dropping an element, a new colour role),
  render 2–4 options as a throwaway scratch script, show them close up and at real size, and let the owner pick. Only build the
  chosen one.
- **Revert what does not help.** If a change makes no visible difference, take it back out rather than keeping it "just in case".
- **Keep compatibility.** Keep saved settings, bindings, loadouts, replays and node paths that tests or other screens depend on.
- **Report honestly.** Say what you could not check (e.g. a real gamepad, a 4K screen) and give the smallest next step.

## Priorities (biggest visible gain first)

1. **Hierarchy and readability.** One clear focal point per screen, a type scale with distinct roles, and contrast that passes at a
   couch distance (1080p on a TV).
2. **One visual language.** One edge style, one panel material, one palette. Remove the leftover second language (`UI_REVIEW.md`
   item 1).
3. **Layout and spacing.** A consistent grid, aligned edges, breathing room, nothing touching the screen edge, and safe areas at both
   aspect ratios.
4. **Focus and feedback.** Obvious selection, crisp state changes, and consistent motion timing. The game is played with gamepads;
   every screen must be fully usable without a mouse.
5. **Details.** Icons, small motion and sound, only after the above reads well. Avoid clutter.

## Definition of done

- Every screen reads as part of one product, follows `UI_DESIGN_SYSTEM.md`, and was compared before and after in the same setup.
- Text is readable at 1280x720 and on a TV at 1080p.
- Gamepad and keyboard navigation works everywhere.
- The UI tests pass (or fail exactly as on the base branch, noted).
- No gameplay, sim or character changes.
- The work is on its own branch with one commit per pass, a before/after image for each, and an analysis log. A PR summarises the
  passes for the owner.

## First task

Do Phase 0 and Phase 1 and report back: the baseline captures, the reference table (or a request for references if there are none),
the ranked weaknesses with causes and fixes, and the proposed order. Edit nothing until the owner approves.

**Guiding principle:** refine, do not replace. Fix structure before decoration, measure before judging, and change one visible thing
at a time.
