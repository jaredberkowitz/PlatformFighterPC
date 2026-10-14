# Visual polish: audit, plan and change report (2026-10-14)

The owner's visual polish iteration plan, merged with the open steps of `docs/ART_REVIEW_2.md`. The rule is to **iterate on what exists**:
fix a parameter, then a shader, then geometry, and remodel or replace only what smaller changes cannot fix. Each change is one variable,
compared before and after from the same camera, and kept only if it helps. Animation, weapon models and modelled props stay deferred to
the end (the owner's call); everything else in the plan is in scope, and anything skipped is listed under "Skipped, and why".

## Baseline (phase 1)

- **Renderer**: Godot 4.7.1, Vulkan Forward+, MSAA. One shared cel light (`shaders/toon_light.gdshaderinc`) for fighters and the stage, a
  post pass for ink, haze, the value hierarchy and paper (`shaders/post.gdshader`), per-stage moods and shading profiles
  (`scripts/lighting.gd`).
- **Lights**: a key with soft-edged cast shadows (two splits, 70 m), a cool fill from behind, flat ambient. No SSAO or SSIL; contact
  darkening comes from `stage_ao.gdshaderinc` and painted contact patches.
- **Fighters**: one skinned base body on a shared skeleton (`art/blender/make_base_body.py` builds it, `fit_body.py` measures it,
  `make_rigged_blob.py` rigs it into `godot/models/base_rig.glb`), clothes painted on by region (`shaders/skin.gdshader`), a 2D face,
  wardrobe pieces from `Loadout`, an inverted-hull outline plus the post pass's ink.
- **Stages**: built in code (`scripts/stage_art.gd`): blocks with a patterned front, grass or sand caps, props, backdrop layers.
- **Captures**: 1280x720. `--demo=stage --stage=N --match --noecb --chars=0,6` (play distance with the HUD) and
  `--demo=stage --stage=N --noui --noecb --chars=2,9 --cam=7` (close-up), every stage, plus `tests/lineup_shot.gd`.

### Found while capturing

- **The collision-box outlines were in every review shot.** With `--noecb`, the outlines were never updated, so all four stood visible
  at the stage's origin (the gold diamond beside the second fighter in earlier reviews). They are hidden until shown now. Shots only;
  play was never affected.

## Ranked opportunities

| # | Opportunity | Responsible files | Without remodelling? |
|---|---|---|---|
| 1 | **Fighter shading** is a soft three-band ramp with strong hue rotation: orange fighters showed orange, salmon and pink streaks. | `toon_light.gdshaderinc` | Yes (done, pass 52) |
| 2 | **The light/shadow edge on fighters is ragged**: the base body's decimation left long thin triangles whose normals jag the edge. | `make_base_body.py` | Needs a topology change (done, pass 52) |
| 3 | **The meadow platform**: lawn patches read as overlaid rectangles, the earth front repeats the same rounded panels, the lip is a hard line. | `stage_art.gd`, `toon_common.gdshaderinc` | Yes |
| 4 | **Hero props are flat**: the hay bales' checker reads as a decal, the barrel has no rim or bands, the fence is thin, bright and busy behind fighters, the windmill's sails are paper-thin. | `stage_art.gd` | Yes (geometry built in code) |
| 5 | **Ink line noise** inside fighters (doubled strokes, dashes between the legs) and contact/grounding of feet. | `post.gdshader`, `outline.gdshader`, `fighter_view.gd` | Yes |

After these: material families written down as standards (phase 6), the backdrop hierarchy and the floating islands (phase 7), motion
checks (phase 8), and a final grading and performance pass (phase 9).

**Is a character model change warranted?** Yes, one: the topology (item 2). The proportions and silhouette follow the concept sheet and
read well; the faces, wardrobe and moves depend on them, so they stay. Hands are the next candidate (mitten shapes read lumpy close up).

## Change report

### Pass 52: two-tone fighters and a clean body surface

- **Shading** (`toon_light.gdshaderinc`): fighters drop the stage's three-band ramp for a hard step between two flat tones
  (`PF_FIGHTER_STEP` 0.62, anti-aliased over `PF_FIGHTER_EDGE` 0.02). Their shadow takes a third of the stage's hue turn
  (`PF_FIGHTER_HUE`), so an orange shades deeper orange, not pink. The shadow tone is a step deeper (`PF_FIGHTER_SHADE` 0.8, which
  replaces `PF_FIGHTER_LIFT`), because the two tones need a clear value step to read as shapes at play size. The stage keeps its ramp.
  Tried and rejected: the step at 0.5 (the shadow was a sliver) and at 0.8 (too much of the face in shade).
- **Body topology** (`make_base_body.py`, `REDUCE = "quad"`): the body is reduced with a QuadriFlow quad remesh plus four smoothing
  steps instead of edge-collapse decimation. Same budget (6,386 triangles against 6,498), joints within a millimetre (refitted, rigged
  again; hats, glasses and weapons fit as before). Under the hard step the light/shadow edge is now one smooth curve.
- **Images**: `docs/screenshots/polish1_twotone_city.png`, `polish1_twotone_meadow.png` (before left, after right),
  `polish1_body_terminator.png` (step only on the left, step plus the new topology on the right), `polish1_lineup.png`.
- **Checks run**: fighter view, animation library and art pipeline tests pass; the look bench's worst silhouette holds at 0.89 (needs
  0.85); close-ups on the meadow and the night city, and the lineup.
- **Still open**: on the meadow, the key sits near the camera, so fighters show only a narrow shadow side there (per-stage mood, left
  as is for now); the fists' shading shows their own facets.

### Pass 53: one clean silhouette line on fighters (step 3 of docs/ART_REVIEW_2.md)

- **Problem**: a fighter's silhouette had two lines, the inverted-hull outline just outside it and the post pass's screen-space ink
  just inside it. Together they made a thick, dotted, wavering stroke at play size. Dropping the post line on fighters outright left
  blades and the shield rim with no line, and the hull alone was faint up close.
- **Fix**: the hull outline writes its own object id (`HULL_ID` 0.7; `outline.gdshader` is now emission over black so it can write
  one), and the post pass draws no line on or beside a hull line, only where a silhouette has none. The hull's minimum width is a
  little higher (0.02 to 0.028 world units) so it holds up close.
- **Images**: `docs/screenshots/polish2_silhouette_line.png` (before left, after right, play distance, 4x).
- **Checks run**: all five stages at play distance and close up; the look bench's worst silhouette 0.89 to 0.90. Performance (the KO
  demo, warm shader cache, alternated runs): 383 fps against 391 before, GPU 1.71 ms against 1.66 ms. The first run after a shader
  change hitches while the cache rebuilds; that is not a cost in play.

### Pass 54: the meadow platform (phase 4; step 6 of docs/ART_REVIEW_2.md for the meadow)

- **The front**: the meadow and the beach shared one masonry tile, a grid of rounded panels. The meadow now has its own `earth` pattern
  (`stage_art.gd`): broad beds of farm soil with soft waving seams, half-buried stones of very different sizes (a few boulders, more
  middling stones, many pebbles; lit tops, shade under them), fine roots hanging from the turf, grit. It is drawn at 512 px and laid
  every 12 units, so no repeat shows across the stage. The beach keeps the masonry, which suits a sea wall. First try had a boulder the
  size of a fighter right under the fighting; the boulders are smaller and closer in value to the soil now.
- **The lawn's patches**: long, hard-sided darker rectangles on the lawn were the platforms' cast shadows (confirmed by switching the
  stage AO off: they stayed). A plank three units up cast a shadow as crisp as a fighter's foot. The key light's angular size is now 5
  degrees (from 0.35), so a shadow softens with its distance from what casts it: contact shadows stay crisp, the platforms' shade
  becomes a soft pool. (2 degrees changed nothing visible; 10 lost the platforms' shade altogether.)
- **The stage AO under platforms** (`stage_ao.gdshaderinc`, found on the way): the baked map is flat (across and up only), so on the
  ground it shaded the whole depth of the lawn under a platform, back to front. Ground (a surface facing up) now takes it only under
  the platforms' depth (`pf_ao_depth`, from `StageArt.PLATFORM_DEPTH`), fading over a unit. Fighters and walls are unchanged.
- **Images**: `docs/screenshots/polish3_meadow_platform.png` (the platform and front, before left), `polish3_lawn_shade.png` (the lawn).
- **Checks run**: every stage at play distance and close up; the look bench (0.90); match flow, fighter view and art pipeline tests.
  Performance could not be measured cleanly this pass (the machine was busy: the GPU time swung between 1.8 and 5.5 ms with either
  shadow setting); to be measured again.

### Pass 55: the meadow's hero props (phase 5, first of each group)

- **Hay bales** (`_hay_bale`): were 12-sided drums with the straw hat's weave wrapped over them, ends included, so they read as
  checkered boxes. Now a lathe (`_lathe`, a new helper): a drum whose rim rounds over into the ends, strands of straw running round
  the body and a rolled spiral painted on each end (`_straw_texture`, drawn in code), and two thin bands of twine.
- **Barrel** (`_barrel`): a straight cylinder with flat bands. Now a bulging lathe of painted staves (`_stave_texture`: twelve boards,
  dark seams, lit edges, grain), iron hoops sized to the bulge where they sit, a rim of staves standing above an inset, boarded lid.
- **Windmill** (`_windmill`): the canvas sat in front of the lattice, so the sails read as four flat white boards. The canvas is now
  stretched behind the frame (rails and slats over it, as a common sail is built), with a leading board on the other side of each
  stock, and the hub is a drum with an iron nose on a windshaft that comes out of a housing on the tower, not a disc floating in
  front of it.
- **Fence**: sturdier posts of slightly different heights, each leaning a touch, and rails set bay by bay, each a little off level,
  instead of two perfect boards the length of the stage. Kept as quiet as before behind the fighters. (It draws from the meadow's
  random sequence, so the flowers placed after it moved.)
- **The earth front's tiling** (found while checking): each soil bed's seam started at x 0 without its wave (a step at every repeat),
  and the beds had straight bottoms with bare bands between them. Seams start on the wave now, each bed runs down under the next, the
  last one wrapped to the top, and the beds' shades are closer together. Checked by drawing the tile 2x2.
- **Images**: `docs/screenshots/polish4_bales.png`, `polish4_windmill.png`, `polish4_meadow_props.png` (before left).
- **Checks run**: the meadow close up and at play distance (1080p), match flow and art pipeline tests.

### Pass 56: trees, sky islands, turf edges (phase 5, the rest)

- **Trees** (`_round_tree`): every crown was the same big mass with a smaller one to its right. Now one of three shapes (a side lobe
  either way, a tall stacked crown, a wide crown with two shoulders), chosen from the tree's own position so nothing else moves.
- **Sky islands** (`_island_detail`): the turf over the edge was seven equal balls in a row (a bead necklace). Now nine tongues of
  different widths and lengths, overlapping and sunk into the lip. The roots, dark sticks that read as posts, are thin, tapering and
  bent, in two lengths each, closer to the earth's colour.
- **The stage's own lip** (grove, and any grass cap without moss): the same bead row, under the fighters. Each tongue now has its own
  width and hang (seeded from its place, so the layout holds).
- **Images**: `docs/screenshots/polish5_islands.png`, `polish5_grove_lip.png` (before left).
- **Checks run**: the meadow and the grove at 1080p, match flow test.

## Skipped, and why

Nothing yet.
