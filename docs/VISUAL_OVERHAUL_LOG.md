# Visual overhaul: results log

Measurements and decisions for `docs/VISUAL_OVERHAUL_PLAN.md` (audit: `docs/VISUAL_OVERHAUL_AUDIT.md`). Stills: `docs/screenshots/overhaul_<phase>_*.png`.

## Phase 0: audit and baseline

See the audit. Baselines: `overhaul_0_skyline_hud.png`, `overhaul_0_fx.png` and the graphics pass's `fidelity_6_*.png`.

## Phase 1: lighting and post

* **Tone mapping** (`overhaul_1_tonemappers.png`: Skyline above, Meadow below; Filmic, ACES, AgX left to right). Kept Filmic. ACES is
  punchier at night but pushes the day stage's pale surfaces and the fighters' white clothes toward plain white; AgX greys the saturated
  palette the art direction wants. Exposure stays 1.05 (no global brightening to make lights read: the windows and halos are HDR instead).
* **Anti-aliasing** (`overhaul_1_antialiasing.png`, a close-up: MSAA only, plus FXAA, plus SMAA). MSAA smooths geometry but not the ink
  lines the finishing pass draws or the leaves' cut-outs; FXAA smooths them but softens every drawn detail (the gloves' marks, the faces);
  SMAA smooths the stair-steps and keeps the drawing crisp. Now SMAA on Best and Balanced (`visual_settings.gd`), none on Fast. Cost:
  Skyline GPU about 1.05 to 1.22 ms.
* **Shadows and contact**: no acne, peter-panning or halos found in the stills (the key light's shadows are two splits over 70 units,
  blur 0.5); props already sit on painted contact shade from the graphics pass; any prop missing it is covered in phase 3.
* No other change was needed for the plan's 1.1 / 1.2 (the moods, rim light and fighters' edge light already give the separation it asks
  for, and the look bench measures it).

## Phase 2: character rendering

* Audit: every fighter part is the shared cel family (`toon`, `skin`, `limb`, `toon_cloth`, `face`), lit by `toon_light.gdshaderinc`, with
  the hull outline (constant screen width, fighters at full weight and tinted with their own colour) plus the screen-space ink; every
  class and moveset uses the one base body, so changes there reach all of them. Before this phase every part had the same response.
* Material responses (`FighterView.RESPONSES`, applied with `respond()` on copies, so props sharing a cached material are untouched):
  skin a softer ramp and almost no sheen; gloves and soles rubbery (a firmer step, a wider soft sheen); shoes the same; cloth stays matte
  (the existing cloth ramp); the blade and its guard metal. A new optional term in the shared light, `metal` (0 by default): a crisp glint
  band and a stronger rim.
* The sword's blade was a thin box, so its face never changed angle and no light term could make it read as metal. It is now a blade
  shape of the same size (`_blade_shape`: a diamond cross-section, a ridge, flat bevels, a point): the upper bevels light and the lower
  ones shade, the classic two-tone blade, and the glint moves along the bevels as it swings (`overhaul_2_materials.png`, before and after
  pairs; `overhaul_2_closeup.png`).
* Outlines: already distance-compensated (`outline.gdshader`: constant screen width between a minimum and a maximum) and weighted
  (fighters full, scenery 0.6); fighters stay told apart by their tinted line, edge light and look. No change needed.
* Checks: fighter view test (every move of all three classes), loadout and art pipeline tests passed; look bench PASSED, worst 0.93.

## Phase 3: rooftop, platforms and props

All art only (`stage_art.gd`, the roof tile): no collision, platform position or size changed. `overhaul_3_skyline_before_after.png`.

* Rooftop: the pavers' seams are legible (a darker seam with a light bevel beside it, `godot/art/stage/roof.svg`) on top of the painted
  concrete wear from the graphics pass; the front edge already has its trim, ledge and shade band.
* Girder platforms: three parts that read apart: the top plate, the red web, and a darker steel flange underneath; a bright trim along the
  top front edge (it catches the light and marks where the platform ends) and the top plate's shade falling on the web.
* Props: the water tower gets iron hoops, a roof hatch and a ladder; crates (Skyline and Triple Tier) a frame of darker boards and an X
  brace; the air conditioner a grille and a fan on top; the vent a cap; the light mast a base plate. Every prop already stood on painted
  contact shade.

## Phase 4: background depth and life

The three skyline layers, their haze and the far blur came from the graphics pass; this phase is about the buildings themselves
(`facade.gdshader`, `StageArt._skyline_layer`). `overhaul_4_skyline.png` against `overhaul_3_skyline.png`.

* No two buildings share a pattern: each building's seed picks its bay width and height, its pane shape, its share of lit windows (0.4 to
  1.55 of the layer's) and a slightly different wall shade; about one in five has office strips (rows of wide panes with thin mullions,
  lit in runs of two bays, at half brightness: at full strength their long runs of light outshone the fighters in the first try).
* A few windows switch on and off on slow clocks (one in forty, each its own); signs stutter off now and then. Both shader-only.
* Lit signs: a new face kind in the facade shader (a neon border, a faintly lit panel and bars of lettering, in pink, cyan or amber), on
  about a third of the nearer buildings' fronts and as two rooftop billboards on the middle layer near the centre of the view. The first
  version's thin border vanished at that distance; the border, letters and signs are bigger now.
* The far layer drops frames, sills and floor bands (`detail` 0): only windows, so it stays a quiet silhouette.
* No new lights: everything glowing is emissive in the one facade material per layer.
* Cost: Skyline GPU 1.27 ms (1.22 after phase 1), 282 draw calls (267; the extra fifteen are phase 3's prop details).

## Phase 5: combat effects

Audit: hits already scaled with their strength (from hitlag): the burst's size and life, a ring from half strength, sparks, a directional
spike above 0.4, a white flash on the struck fighter, screen shake along the hit's line, scuffs; every effect frees itself when its life
ends (the scuffs are a pool of six, cleared each match). `overhaul_5_hits_before.png` / `_after.png`: rows are a jab, a forward tilt and a
finishing forward smash (claws), left to right through the hit.

* Three tiers that read apart: a light hit is a small spark; a medium one (strength over 0.25, a tilt) now adds a short dart the way it
  sends the fighter, so its direction reads; a strong one a bigger burst, a ring and a long spear (the dart and spear are one effect
  scaled: 0.9 + 3.3 x strength long).
* The strongest burst no longer covers the fighter it hit: it is a little smaller (0.36 + 0.74 x strength, from 0.9) and its ring thinner.
* Unchanged on purpose: hitlag, shake and every timing (presentation only).

## Phase 6: HUD

Audit: the HUD (`ui/match_hud.gd`) is drawn in its own canvas, so world lighting and the finishing pass never touch it, and it scales
with the window (`canvas_items` stretch, 1280 x 720 base). It already had the damage's heat colours (white through orange and red to
deep red), a spring pop and a shake on every hit, a hurt face on the portrait for heavy hits, stocks that pop off, and the player's tag
beside the name (so players are told apart by more than colour).

* Checked at 16:9, 16:10, 4:3 and 21:9 (`overhaul_6_hud_1024x768.png`, `overhaul_6_hud_1680x720.png`): nothing clips; the cards keep to
  the corners.
* The cards sit 30 pixels (about 4%) above the bottom instead of 20, clear of a television's overscan.
* A hit flashes the damage number toward white for 160 ms before it settles to its heat colour (the brief colour feedback; at high
  damage, where the number is deep red, it reads as a flare).
* "FREE PLAY" sits on a small dark slanted chip like the name bar's, instead of floating over the stage (`overhaul_6_hud_cards.png`).
* Match flow test PASSED.

## Phase 7: performance and regressions

* A busy fight to measure: `--demo=cpu4 --players=4` (four computers, one of each class and a second sword fighter, twenty seconds;
  `--players=N` is a new demo/test override of the player count). Frame time on this machine (Best, vsync off): Skyline 5.2 to 5.6 ms
  average, p99 11 to 12.5 ms, GPU 1.4 to 1.5 ms, 551 draw calls; Treetop Isle 5.7 to 5.8 ms average, p99 16.7 ms, GPU 1.8 ms, 636 draw
  calls. The CPU side (scripts, rendering setup, the four computers' thinking) is most of it; the shaders added in both passes cost about
  0.2 to 0.4 ms of GPU time in all. Within a 60 fps budget; Treetop Isle's p99 is at its edge.
* A bug the four-fighter run found (it predates this pass): with four players the HUD's cards (330 pixels each) did not fit 1280 and each
  portrait covered its neighbour's damage. Cards now shrink to fit, each scaled about its own bottom-left corner
  (`overhaul_7_hud_four.png`).
* Regressions: all 19 Godot tests and all 44 Rust suites passed; look bench PASSED, worst 0.93; the character select's previews are lit as
  before. Every class's moves render (the fighter view test plays each); nothing in this pass touches the simulation, collision, input or
  timing.
* Effects clean up after themselves (each frees itself when its life ends; the scuffs are a pool of six cleared each match). Shader
  warm-up: the first frame-time run after a change sometimes stalls (a cold shader cache); later runs do not.
* Compromises: the blade's metal reads mostly through its new shape (a flat card cannot glint); the second billboard sits behind nearer
  buildings at the default camera; the HUD flash and the window flicker are presentation only and do not follow replay seeks exactly.
