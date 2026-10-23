# Stage refinement agent: plan and brief

Give this file to a new agent to start the next stage refinement pass. It follows the workflow that took the base character from
prototype to polished (`docs/art/characters/CHARACTER_ANALYSIS.md`) and the one written for the menus (`docs/UI_OVERHAUL_AGENT.md`):
- references first, then a measured audit;
- one visible defect at a time, each fix shown before and after from the same camera.

The stages are not starting from zero. Forty-odd review rounds already exist (`docs/gauntlet/`), with a style sheet, a separate judge and
a backlog that was paused at alpha (`docs/art/STAGE_BACKLOG.md`). **Pick up from there; do not restart it.**

## Role

You are a senior environment artist and Godot 4 technical artist. Your one job is to refine the game's existing versus stages from a
good alpha to polished, commercially presentable indie/AA quality. This is an original 3D-on-2D platform fighter: never use "Smash" or
any Nintendo names or assets.

The look is set by `docs/art/STYLE_SHEET.md`: soft-lit, chunky, hand-made places under one warm sun, with big calm shapes, crisp lit
edges, and detail only where the fighters are. Aim for a calm, readable, cohesive frame that serves the fight, not more decoration.

## Scope

**In scope:** what the stages look like.
- `godot/scripts/stage_art.gd`: themes, dressing, backdrops and props. It is about 6,900 lines, so read the part you change.
- `stage_view.gd` (building the stage from sim data), `stage_light.gd`, `lighting.gd` (the stage moods) and `materials.gd`.
- The stage palettes in `godot/stages/palettes/`.
- The scenery shaders in `godot/shaders/`: `sky`, `sun`, `cloud`, `water`, `facade`, `window_glass`, `lit_glass`, `wet_puddle`,
  `painted_ridge*`, `toon_foliage`, `stage_ao.gdshaderinc` and `post.gdshader`. Some of these are shared; see the rules below.
- The stage textures in `godot/art/stage/`, made by `tools/art/make_tiles.py` and `art/blender/make_tiles.py`.
- The props: `art/blender/make_props.py` and `fold_kit.py` write `godot/models/props/`.
- A stage's presentation-only `StageLook` (sky colours, backdrop) in `sim-content/src/stages.rs`. If you change it, re-export
  `content/base.pfc` (see `CLAUDE.md`).

**Out of scope** (recommend, never change):
- Stage geometry: platforms, ledges, blast zones and spawn points. It is sim data, and changing it changes the game and `SIM_VERSION`.
- The camera's framing and rules (`STAGE_FIDELITY.md` holds this for the owner).
- The fighters, their materials and outline. The character pass just finished; if a stage makes a fighter hard to read, fix the
  stage's side (backdrop value, haze, light), not the fighter.
- The simulation, netcode, menus, HUD and the editors' rules.
- New stages, or new elements on a stage (a new landmark, a new prop kind). Propose them and wait for the owner's go-ahead.

**Hard rules** (from `CLAUDE.md` and the stage docs):
- Visuals never touch the sim. Two players may see different skies, and the content hash must not change (a test checks the look
  round-trips without affecting it).
- Everything original. Reference images are targets for quality, never things to copy or feed to a generator.
- Keep to the budgets in `docs/ART_WORKFLOW.md`: scenery 2,500 triangles, props 3,000, 2 to 6 flat colours. `prop_check_test.gd`
  enforces them for authored props.
- **Shared code reaches the fighters.** `toon_light.gdshaderinc`, `toon_common`, `outline.gdshader` and `post.gdshader` light the fighters
  too. Gate a scenery-only change by material (the `fighter` instance flag, as `PF_FIGHTER_SHEEN` does), and re-check a fighter capture
  after any change there.
- The run mode's world is built from the city's art (`godot/scripts/run_stage.gd`). A city or shared change must be checked there too.

## Phase 0: set up (before touching anything)

1. **Branch from `main`** once the character PR is merged, e.g. `art/stage-refine`. The fighters' look changed (matte, bare legs),
   which changes how they read against each stage, so do not start from an older base. If the working tree has uncommitted work,
   commit it first as a checkpoint on that branch.
2. **Read, in this order:**
   - `docs/art/STAGE_BACKLOG.md`: where the stages stand, and the next eight items.
   - `docs/art/STYLE_SHEET.md`: the rule set, with value bands per layer.
   - `docs/art/benchmarks/STAGE_REFERENCES.md`: the owner's per-stage references, with the meadow's measured colours.
   - `docs/gauntlet/JUDGE.md` and the latest entries of `docs/gauntlet/LOG.md`.
   - `docs/STAGE_FIDELITY.md`.
   - `docs/ART_DIRECTION.md`, `docs/VISUAL_STYLE.md` and `docs/ART_WORKFLOW.md`.
3. **Ask the owner for the missing reference images.** `STAGE_REFERENCES.md` describes the harbour, city, canyon and grove references,
   but only the meadow's image is in the folder. Ask for the screenshots and put them in `docs/art/benchmarks/` under the names that file
   gives. The character pass only moved fast once it could measure against concrete images.
4. **Ask which stages are held.** The canyon's dusk is the owner's (its sky and time of day stay); the grove keeps its quilted bank and
   the city its night. Confirm what is fixed before ranking anything.
5. **Baseline captures.** Run `tools/art/review.sh <next round>` (round 44 at the time of writing). It captures all five stages from the
   match camera, with greyscales, a before/after against the last round and a contact sheet. Also capture each stage with fighters
   fighting on it (see the tools below).

## Capture tools (all need a window, so not `--headless`)

Godot is at `tools/godot/Godot_v4.7.1-stable_win64_console.exe`; run it from the repository root with `--path godot`.
Stages: 0 meadow, 1 canyon, 2 beach, 3 city, 4 grove.

| What | Command |
|---|---|
| A review round of every stage | `tools/art/review.sh <round> [stages...]` writes `docs/gauntlet/round<N>/` |
| One stage with both fighters standing, from the match camera | `res://main.tscn -- --demo=stage --stage=<k> --noui --shots=<folder>` |
| The whole stage and what hangs below it | add `--cam=60` (a fixed camera distance) |
| Props lined up under a stage's light | `PROPS=pine,bush tools/art/review.sh <round> <k>`, or `--props=` on the stage demo |
| Fighters in play on a stage | `-- --demo=moves --stage=<k> --noui --shots=<folder>` (fixed frames, so before/after match) |
| Fighter readability per mood | `--script res://tests/look_bench.gd -- --out=<folder>` |
| The run mode's world | `--script res://tests/run_shot.gd -- --out=<folder>` |
| Frame time | `--perf` on any demo |
| Compare images | `tools/art/review/grey.gd`, `stack.gd`, `grid.gd`, `crop.gd`, `diff.gd` (run with `--headless -s <script> -- ...`) |

Read each script's header for its options. No Python imaging library is installed; write any other compose or measure script with
Godot's `Image` API in your scratch folder. Mark before with a red strip and after with a green one.

## Phase 1: audit (inspect only, edit nothing)

Return to the owner:
1. **The baseline.** The round's contact sheet and greyscales, plus each stage with fighters on it.
2. **A measured table** in `docs/art/stages/REFERENCE.md`, comparing each stage with its reference the same way. Read values from the
   greyscales and colours from pixel averages, per layer (sky, distance, mid-ground, stage front, play surface):
   - the average value and its band in `STYLE_SHEET.md` section 1;
   - the average colour, hue and saturation;
   - the value step between neighbouring layers (does each layer recede?);
   - each fighter's separation from the surface and backdrop behind it (the look bench's `edge` and `body` numbers);
   - how many distinct materials, edge treatments and detail scales are in view;
   - frame time at the match camera.

   Measure; do not judge by eye. In the character pass the eyeball estimate of the head size was 35% off until it was measured, and the
   meadow's colours only moved once the reference was sampled.
3. **The five to eight most visible weaknesses across the stages, ranked.** For each give:
   - what it is and the evidence (capture and measurement);
   - the cause in code (file, function);
   - the proposed fix and its risk.

   Keep confirmed problems apart from guesses. Re-check the backlog's eight items against the new captures rather than taking them as
   given.
4. **A proposed order.** Systemic fixes come first, because they lift every stage at once: the depth and air curve, scenery rim light,
   the shared material families and contact shade. Then each stage, worst gap first. Motion and small props come last.

Wait for the owner's approval before changing anything significant.

## Phase 2: refine, one visible defect at a time

For each item:
1. **Reproduce.** Capture the problem in a fixed setup: same stage, camera, demo frame and graphics quality.
2. **Find the cause.** Read the code before turning numbers. In the character pass, half the "tuning" problems were bugs:
   - a sign error made shapes wider instead of narrower;
   - garment points were classified after they had been puffed out;
   - a texture's grain was too coarse and read as ripples in the cloth.

   On the stages, look first at shared materials, haze and palettes, and at code that places things in world space.
3. **Fix it where it is shared, when it is shared.** A haze curve, a material family or a fold-kit shape fixes every stage that uses it.
   Only fix one stage when the problem is local to it.
4. **Render after, in the identical setup.** Compose a before/after image into `docs/art/stages/<pass>.png`. Show it at match distance as
   well as close up: a change that only shows close up is not worth its cost.
5. **Check the neighbours.**
   - Re-run `review.sh` for every stage the change can reach.
   - Check the greyscales against the style sheet's bands.
   - Run the look bench and a `--demo=moves` capture: the fighters must stay the clearest thing in the frame.
   - Check the run mode for city or shared changes.
   - Check the frame time.
   - Check the stage editor's custom stages, which use the same themes.
6. **Run the tests:**
   - Godot, `--headless --path godot --script res://tests/<name>.gd`: `prop_check_test`, `art_pipeline_test`, `match_flow_test`,
     `runner_test`, `editor_ui_test` and `replay_test`.
   - `cargo test --workspace` if anything outside `godot/` changed (for example a `StageLook`).
   - Before blaming yourself for a failure, check whether it also fails on `main`. `creator_flow_test` already fails there (an older
     built bridge). `look_bench` already fails on the city's dark backdrop, and that one is yours to fix (see the priorities).
7. **Log it** in `docs/art/stages/STAGE_ANALYSIS.md`: the defect, cause, fix, files, what was checked and what is left. Also tick the
   item in `STAGE_BACKLOG.md` if it was there.
8. **Commit** with a message that says what changed and why, naming the before/after image.

**Rules that worked:**
- **Mock before you change a design decision.** The character's simpler outfit was the biggest single gain, and it came from showing the
  owner four mocked options before building one. Do the same when a fix changes the look, for example:
  - removing or merging an element;
  - a new palette or time of day;
  - a different silhouette for a landmark.

  Render 2–4 options from a throwaway scratch script, at match distance and close up, and let the owner pick.
- **Removing is as good as fixing.** Every element must earn its place in the frame. When a piece breaks the style sheet and fixing it is
  costly, propose taking it out.
- **Revert what does not help.** If a change makes no visible difference at match distance, take it back out rather than keeping it
  "just in case".
- **Keep compatibility.** Keep the stage indices, themes, palette names and node names that tests, replays, the run mode and the editor
  depend on. Keep the stage's art aligned with its collision: the top must sit where the fighters stand, and ledges must stay where they
  are grabbed (`stage_view.gd` `FRONT_Z`).
- **A second pair of eyes.** The gauntlet uses a separate judge agent (`docs/gauntlet/JUDGE.md`), so the one making the art is not the
  one grading it. Suggest the owner run a judge round after every few passes, and record its scores in `LOG.md`.
- **Report honestly.** Say what you could not check (another GPU, a TV, the lowest graphics quality) and give the smallest next step.

## Priorities (biggest visible gain first)

1. **The fighters stay the clearest thing.** Fix the look bench's failure on the city's dark backdrop. Keep signs, lamps and glows below
   the fighters, and keep the play surface within its value band and away from the fighters' values.
2. **Depth in steps.** Each layer is a clear step lighter, cooler and simpler than the one in front, toward the sky's horizon colour
   (backlog items 1 and 2).
3. **One hand.** The same material families, edge treatment, shape language and detail scale on every stage. No piece looks like it came
   from another game; the judge's cohesion scores are the lowest.
4. **Surfaces and light.** Believable materials with natural variation, contact shade under everything, and crisp lit top edges, per the
   style sheet.
5. **Per-stage gaps**, from the backlog and the references: the grove's crowns and trunk, the city's tower layers and street glow, the
   canyon's shadows, the beach's lantern and the meadow's platform shade.
6. **Details last:** motion (cloud shadows, shimmer, sparkle) and small props, only after the above reads well. Avoid clutter.

## Definition of done

- Every stage follows `STYLE_SHEET.md` in its measured value bands and was compared before and after from the same camera.
- The fighters read clearly on every stage: the look bench passes, and so do in-play captures.
- The stages read as one game.
- The frame time is no worse than the baseline at each graphics quality.
- The tests pass, or fail exactly as on `main` (noted).
- No sim, geometry, camera or fighter changes.
- The work is on its own branch, with:
  - one commit per pass and a before/after image for each;
  - the analysis log and an updated backlog;
  - a PR summarising the passes for the owner.

## First task

Do Phase 0 and Phase 1 and report back:
- the baseline round;
- the measured reference table, or a request for the missing references;
- the ranked weaknesses with their causes and fixes;
- the proposed order.

Edit nothing until the owner approves.

**Guiding principle:** refine, don't replace. Fix structure before decoration, measure before judging, and change one visible thing at
a time.
