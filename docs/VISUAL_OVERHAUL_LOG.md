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
