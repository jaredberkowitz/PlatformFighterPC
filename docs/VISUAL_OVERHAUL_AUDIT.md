# Visual overhaul: audit (phase 0)

For `docs/VISUAL_OVERHAUL_PLAN.md`. Branch `visual-overhaul`, from `graphics-fidelity` (whose pass, `docs/GRAPHICS_FIDELITY_LOG.md`, already
covered much of this plan's stage side: see "Already done" below). Results go in `docs/VISUAL_OVERHAUL_LOG.md`.

## The pipeline as it is

| Area | Where | Notes |
| --- | --- | --- |
| Engine, renderer | Godot 4.7.1, Forward+ (Vulkan, D3D12 fallback), `godot/project.godot` | MSAA 3D 4x on Best, 2x Balanced, off Fast (`scripts/visual_settings.gd`) |
| Environment | `scripts/lighting.gd` `apply` / `stage_mood` | Filmic tonemap, exposure 1.05, white 5; ambient by mood; glow only above 2.3 (effects, lit windows), intensity per theme; saturation 1.04, contrast 1.06; no SSAO / SSR / SDFGI (incompatible with the object id in roughness, see `GRAPHICS_FIDELITY_PLAN.md`) |
| Lights | `lighting.gd` | A key (shadows: 2 splits, 70 units, blur 0.5) and a fill; no local lights. Moods and styles per theme |
| Cel shading | `shaders/toon.gdshader` + `toon_common.gdshaderinc` + `toon_light.gdshaderinc` | Ramp texture per theme (`pf_ramp`), painted shadow hue, deep shade in cast shadows, sky-coloured rim, sheen, fighters' edge light. Variants: `toon_cloth`, `skin`, `limb`, `face`, `toon_foliage` |
| Outlines | `shaders/outline.gdshader` (inverted hull, next pass) + `shaders/post.gdshader` (screen-space ink from object id and depth) | Hull: constant screen width, fighters weight 1, scenery 0.6, tinted per fighter. Screen ink: scenery creases and object edges |
| Finishing pass | `shaders/post.gdshader` (quad at priority -100) | Ink, haze, pastel finish, S-curve, paper grain, vignette, grade, far blur (Best), finishing-hit two-tone |
| Character | `scripts/fighter_view.gd` (`_build_base`, `toon`, `_skin_material`, `_limb_material`, `_cloth`), rig `art/blender/make_rigged_blob.py` | One base body for every class and moveset: parts by mesh name (Arm, Leg, Hand, Sole, Collar, Cuff, Strap, Shorts, Shirt, Foot, Face, Skin, Body); the sword's blade, guard and grip are separate parts. Looks from `scripts/loadout.gd` |
| Stage | `scripts/stage_art.gd`, `scripts/materials.gd` | Blocks, caps and platforms by theme; props; backdrops; Skyline is `_city` + `_skyline_layer` (`shaders/facade.gdshader`); girders for city platforms |
| Effects | `scripts/effects.gd`, `scripts/particles.gd`, `shaders/fx*.gdshader*`, swing arcs in `fighter_view.gd` | Bursts by kind (blow, blade, claw, fire), impact, launch line, shield hit, jump ring; hit strength from hitlag |
| HUD | `ui/match_hud.gd`, `ui/ui_draw.gd` | Drawn (Control `_draw`), so separate from world lighting and post; percent colour ramp (`heat`), pop on damage, hurt reaction, portraits from the fighter's look |
| Measuring | `--demo=stage --stage=N` (+ `--match` for the HUD, `--noui`), `--perf`, `tests/shot_stats.gd`, `tests/look_bench.gd`, `tests/image_diff.gd` | Frame-time runs on this machine sometimes stall at 60 fps; such runs are discarded |

Baselines: `docs/screenshots/overhaul_0_skyline_hud.png` (the rooftop with the HUD), `overhaul_0_fx.png` (a forward smash's hit, six
frames), and the five `fidelity_6_*.png` stills. Frame time: Skyline about 1.5 ms, GPU about 1.05 ms; Treetop Isle GPU about 1.47 ms.

## Already done (the graphics fidelity pass)

Lit, varied skyline windows that glow; per-theme value range and S-curve; far scenery blur; painted wear on stage surfaces; roof decals;
hit scuffs; foliage; water. This plan's 1.4 (no heavy cinematic effects), most of 4.1 (layers, haze, blur) and part of 4.2 (lit colours)
are already met; the work here builds on them rather than redoing them.

## What this plan adds, mapped to files (in order)

1. **Lighting and post** (`lighting.gd`, `visual_settings.gd`): compare the tone mappers 4.7 offers (Filmic, ACES, AgX) on the night and day
   palettes and keep the best; compare anti-aliasing for the screen-space ink (MSAA alone, plus FXAA, plus SMAA); check shadow acne and
   the contact shade under fighters and props.
2. **Character materials** (`fighter_view.gd`, `toon_light.gdshaderinc`): distinct responses by part through the existing material
   parameters (skin softer, gloves and shoes a rubbery sheen, the blade a metal highlight band, cloth matte), outline weight by distance
   checked at play zoom, players told apart against every stage.
3. **Rooftop, platforms, props** (`stage_art.gd`, the roof tile, `_girder`): paver seams on the roof, a readable front edge and underside,
   girders with distinct top, web and flange and an edge highlight, the water tower's hoops, crates' wear, the vent's grille, contact shade
   for every prop. Collision untouched (art only).
4. **Background** (`facade.gdshader`, `_skyline_layer`): more varied window patterns per building (bay width, pane shape, density, strip
   windows), a few lit signs, occasional window flicker; the far layer simplified.
5. **Combat effects** (`effects.gd`, `particles.gd`): weak, medium and strong hits clearly different in size, shape and duration; landing
   and jump dust checked; nothing lingering.
6. **HUD** (`match_hud.gd`): typography and spacing consistent, panels refined, damage feedback checked, safe margins at 16:9 and other ratios.
7. **Performance and regressions**: four fighters with effects, all classes' moves, the Godot and Rust tests.
