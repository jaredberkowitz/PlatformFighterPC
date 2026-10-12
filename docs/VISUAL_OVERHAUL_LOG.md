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
