# Graphics fidelity plan

Hand-off plan for a build agent. Goal: close the gap between the current stage and scenery art and a shippable, professional look,
**without changing the illustrated style** (`ART_DIRECTION.md`, `VISUAL_STYLE.md`). Everything here is presentation only: nothing
touches `sim-*`, `netplay`, content files or the checksum, so no `SIM_VERSION` or `SCHEMA_VERSION` bump.

## Diagnosis (why it looks flat)

The shading framework is already good (cel ramp, painted shadows, rim/edge light, ink pass, haze, grade). The gap is content and a few
unused engine features:

| Symptom | Cause (verified in code) |
| --- | --- |
| Skyline backdrop is large flat slabs with sparse square windows | `StageArt._building` ([stage_art.gd:1144](../godot/scripts/stage_art.gd)) is an `_unshaded` BoxMesh plus one box mesh per window |
| Lit windows, string lights, lanterns do not read as light | Glow threshold is 2.3 (`Lighting.apply`), so only FX bloom; scenery is LDR |
| Whole frame sits in one mid-value band | `post.gdshader` `pastel_desat` / `pastel_lift` / haze / vignette compress contrast; values are shader defaults, not per stage |
| Large surfaces (rooftop, block fronts) read as one tile | World-laid tile plus slow drift only (`toon_common.gdshaderinc`); no mid or micro detail |
| Everything equally crisp | Depth of field exists only on the results screen (`main.gd`, `RESULTS_BLUR`) |
| Trees are balls and cylinders, no foliage volume or motion | `_round_tree`, `_tree` build from spheres; no wind, no translucency |
| Sea is flat | no depth-fade, foam or flow |

## Hard constraints (read before touching anything)

- Read `CLAUDE.md` and `docs/STATUS.md` first. The working tree on `phase7-match-rules` has **uncommitted, unrelated sim and moveset
  changes**. Start from `main` on a new branch (for example `graphics-fidelity`), ideally in a worktree, and never commit those files.
- Do not edit sim crates. All work is in `godot/` and `docs/`. Everything stays original; never use the word "Smash".
- **Object id is written to the roughness channel** (`ROUGHNESS = object_id` in `toon_common.gdshaderinc`; default 0.95 for scenery,
  below 0.6 for fighters). `post.gdshader` reads it for ink lines, haze, pastel and grade. Every new scenery shader must write
  `ROUGHNESS` at 0.8 or above (use 0.95) and must not be a transparent material if it should receive ink and haze. Consequence: do not use
  SSR or any roughness-driven feature, and SSAO stays off.
- The post pass is a quad at `render_priority = -100` on the camera (`main.gd`). Transparent things with a higher priority draw after
  it and so get no ink, haze or grade. That is intended for effects and fine for water and halos.
- Glow is deliberately fighter-safe: threshold 2.3, brightest lit surface about 1.9. Anything new that blooms must be HDR above the
  threshold and must be verified not to make fighters bloom.
- Per-instance shader uniforms already use `instance_index` 0, 1, 2, 3 and 6 in the toon family. Do not reuse them.
- `StageArt.merge_static` merges only `MeshInstance3D` children with a non-null `material_override`, a built-in (non-`ArrayMesh`)
  mesh, and no children. It drops per-instance uniforms. Design new meshes with that in mind (see phase 1).
- Menus use `Lighting.apply(env, root, preview=true)` in small viewports. Do not break them; test the menu previews.
- Keep the quality tiers honest (`visual_settings.gd`): **Fast** must drop every expensive new feature (glow, DoF, volumetrics), **Best**
  gets all of them.
- Cosmetic loadout catalog items stay within their mesh budget (`loadout.gd`); do not change fighters here.

## How to verify (use throughout)

Godot is expected at `tools/godot/Godot_v4.7.1-stable_win64_console.exe` (see `docs/STATUS.md`).

```bash
# Stage still with both fighters (Skyline is stage 3; 0 Meadow, 1 Triple Tier, 2 Flat Island, 4 Treetop Isle)
tools/godot/Godot_v4.7.1-stable_win64_console.exe --path godot -- --demo=stage --stage=3 --noui --noecb --shots=<folder>
# Fighter silhouette safety on every mood (needs a window)
tools/godot/Godot_v4.7.1-stable_win64_console.exe --path godot --script res://tests/look_bench.gd -- --out=<folder>
```

Also run the existing Godot tests listed in `docs/STATUS.md` (at least `art_pipeline_test`, `fighter_view_test`, `match_flow_test`,
`editor_ui_test`) and `cargo test --workspace` once at the end to prove nothing else moved. Save before/after stills to
`docs/screenshots/` as `fidelity_<phase>_<stage>_{before,after}.png` and look at them: a passing test is not evidence the picture got
better. Report honestly what did not work.

## Phase 0: baseline

1. Capture the stage still for all five stages and run `look_bench`; save as `fidelity_0_*`. Record the `look_bench` per-mood numbers.
2. Write `godot/tests/shot_stats.gd` (SceneTree script, no window needed): loads PNGs and prints luminance percentiles (p2, p50, p98),
   mean saturation, and the share of pixels above a luminance of 0.9. Run it on the baselines; these numbers are the yardstick for
   phase 2.
3. Add a frame-time probe to the stage demo run (average and 99th percentile frame time over the shot frames, printed). Record the baseline
   for the Skyline and Treetop Isle stages. Every later phase reports its delta.

Done when: baselines and numbers are committed, the probe prints in a normal demo run.

## Phase 1: Skyline facades and emissive light (highest visible payoff)

Replace window-by-window boxes with one facade shader, and make light sources real HDR light.

1. **`godot/shaders/facade.gdshader`** (spatial, unshaded like today's buildings, `ROUGHNESS = 0.95`). In world space (so tiling is
   independent of box UVs): a window grid (about 2.0 by 2.6 units to match today), per-window hash for lit or dark and for a small palette
   of lit colours (warm yellow most, a few pale blue "television" and magenta), window frame and sill shading, a faint floor band every few
   floors, a lighter parapet cap, a gradient darkening toward the street, and slight per-window brightness variation. Per-building seed,
   width and height come from vertex `COLOR` (not instance uniforms, which `merge_static` would drop).
2. **Geometry**: build each backdrop layer as one `ArrayMesh` with `SurfaceTool` (building boxes plus a parapet lip and a few rooftop
   boxes: vents, tanks, antennas, picked from the seed), one material per layer (`uniform` base shade and lit energy per layer, same colours as
   `_city` uses now). One `MeshInstance3D` per layer. `merge_static` skips `ArrayMesh`, as it should. Remove `_building`'s per-window loop.
3. **Emission**: lit windows output `EMISSION` at roughly 3 to 4 times white on the nearest layer, falling with layer so far windows
   glow less. Check that the HDR value survives `post.gdshader` (it mixes toward haze by up to 36% on far scenery; if a layer drops below
   the glow threshold, raise its energy rather than lowering the threshold, which is shared with the fighters).
4. **Halos**: for `_string_lights` and `_lantern_post` (and the city's antenna blinker), add small additive billboard quads with a radial
   falloff using the existing `fx.gdshader` family so they bloom; `glow_scale` controls it. They are see-through, so `merge_static` leaves
   them alone.
5. **Glow strength per theme**: add `glow` (intensity) to `Lighting.STYLES` so the city can be brighter than the meadow; apply it in
   `Lighting.stage_mood`. Tier **Fast** disables glow entirely.
6. Roof surface: stains and a puddle or two as flat decal-like quads (or `Decal` nodes) using a new `roof_detail` SVG tile in
   `godot/art/stage/` made in the same hand-drawn style as the existing tiles (`svg_art.gd`).

Done when: Skyline still shows lit windows blooming softly, varied building tops, no per-window meshes (report draw-call and node counts
before and after), fighters standing in front of bright windows do not bloom (screenshot with a white-shirted fighter), `look_bench`
still at or above 85% on every mood, frame-time delta reported.

## Phase 2: value range and grade

1. Move `pastel_desat`, `pastel_lift`, `vignette` and `haze_max` out of shader defaults into per-theme values in `Lighting.STYLES`,
   pushed to `post_mat` in `stage_mood` next to the grade. Default values must reproduce today's picture exactly (check with a pixel diff).
2. Then tune per stage toward more contrast: try `pastel_lift` 0.13 to 0.05, `pastel_desat` 0.10 to 0.06, and a gentle S-curve. Keep fighters
   the most vivid and readable thing. Target on the Skyline still: p98 minus p2 luminance spread at least 25% wider than baseline, with
   no stage worse on `look_bench`.
3. **Optional (stretch)**: a colour-grade LUT per theme through `Environment.adjustment_color_correction`, loaded from
   `godot/art/grade/<theme>.png` if present, so the look can be graded in an image editor. Confirm which texture types 4.7 accepts
   before building it; if it is awkward, skip.

Done when: before/after pairs for all five stages in `docs/screenshots/`, percentile numbers reported, `look_bench` green.

## Phase 3: depth

1. **Far-only depth of field during play**: set `cam.attributes` to a `CameraAttributesPractical` with far blur only, starting a little
   behind the stage plane (near `post_stage_dist` plus a margin; recompute where `post_stage_dist` is updated in `main.gd`, remembering
   the camera is pitched so view distance differs a little from `position.z`), small amount (start near 0.03 to 0.05). Fighters and the
   stage plane must stay sharp at every zoom. Tier **Best** only. The results screen already replaces the attributes; make sure leaving
   results restores play settings (rematch path).
2. **Foreground framing** (optional after 1 looks right): per-theme silhouettes (pipes and cables on the city, leaves on meadow and
   grove) **parented to the camera**, placed in view space at the outer 8% of the frame, so they never drift into the playfield as the camera
   zooms. Slight parallax from the camera's world x. Near-blurred if it is cheap. Rule: never overlap the central 80% of the screen.
3. Check that the post pass's ink lines still look right on blurred far scenery (they are computed before DoF, so they soften too).

Done when: stills show a crisp stage plane and soft far field, no fighter softening at any zoom (check a close-up shot and a wide shot),
tier behaviour verified, frame-time delta reported.

## Phase 4: surface detail

1. In `toon_common.gdshaderinc`, add an optional world-space painted detail overlay for world-laid surfaces only
   (`world_tile > 0`): two scales of a tiling hand-painted noise texture modulating lightness and a little hue, strength `detail_amount`
   (0 on fighters and everything by default so nothing else changes). Use the same approach as the existing drift and `brush.gdshaderinc`
   rather than a second lighting model. One extra texture sample, no extra lights.
2. Author the detail tiles as SVGs or small PNGs in `godot/art/stage/` (concrete, soil, stone), same style as the existing tiles.
3. Optional: a small normal-perturbation of the cel wrap term on those surfaces so the ramp bands break up slightly. Skip if it fights
   the ink pass.
4. Persistent hit scuffs: a handful of pooled `Decal` nodes (ring buffer, presentation only) placed on the stage surface at heavy ground
   hits and cleared at the start of a match. Skip if it costs frame time on Fast; off on Fast.

Done when: block fronts and the rooftop show mid-scale variation in stills at normal play zoom; no visible tiling seams; fighters
unchanged (pixel diff on a fighter-only crop).

## Phase 5: foliage and volume (Meadow, Treetop Isle)

1. **Leaf-cluster cards**: new shader on the toon family (`toon_foliage.gdshader`, include `toon_common`/`toon_light`), alpha scissor from
   a procedurally drawn leaf-cluster tile (SVG like the others), **normals bent from the cluster centre** so each canopy shades as one
   rounded volume under the cel ramp. Backlit translucency term in `light()` (brighten when the light is behind the leaf). Vertex wind
   sway driven by `TIME` and a global uniform (add a `pf_wind` entry to `[shader_globals]` in `project.godot`); presentation only.
2. Replace the ball-and-cylinder canopies in `_round_tree` and the grove's great tree with card clusters in a `MultiMeshInstance3D` where it
   helps; keep the trunk code. Keep scenery ink: cutout edges should still get lines from the post pass.
3. Grass and flower tufts near the ground line: `MultiMesh` of small cards with the same wind, in the **Best** and **Balanced** tiers.

Done when: Meadow and Treetop stills read as foliage with volume and gentle motion (capture two frames a second apart), cutout edges
have no shimmer at 4x MSAA, frame-time delta within budget (report; tune card count down if more than about 1 ms).

## Phase 6: water (Flat Island ocean)

Replace the flat sea in `_ocean` with a **`water.gdshader`**: transparent; depth-fade shoreline and foam using the depth texture where
it meets the island; colour from shallow to deep; two scrolling flow layers (a streak or ripple tile); small vertex swell; soft sparkle
on the lit side. Keep it in the stage's palette and the cel look: banded, not photoreal. It draws after the post pass, so it has no ink,
which is acceptable; confirm that it does not hide a fighter's lower body or ledge hang in any camera framing.

Do **not** build a waterfall unless a stage needs one; if added later, it is a scrolling streak texture plus `GPUParticles3D` mist
at its base, reusing this shader's foam.

Done when: Flat Island still shows a believable shoreline and flow, tier **Fast** uses a cheap unlit version, no ledge or fighter
readability regression.

## Phase 7: authored asset kit (needs the owner's call)

Procedural stage props cap the quality. Replace `StageArt._props` and the hand-built props with a small kit of authored `.glb` models run
through `Toonify.apply` (`toonify.gd`) so mixed sources unify under one look.

- **Stop and ask the user which source to use before downloading or generating anything**: CC0 packs (Kenney, Quaternius), a small
  kit built with the repo's Blender scripts (`art/blender/`), or TRELLIS-generated props (the repo's note: licence fine, about one free
  generation a day). Record every asset's origin and licence in `docs/THIRD_PARTY.md` as the existing tables do.
- Start with the city: air conditioner, vents, pipes, rooftop door, water tower, antenna dish, signage, crates (about 10 pieces), each
  within a small triangle budget stated in `ART_WORKFLOW.md`.
- Keep `merge_static` working (or merge at import), keep the contact shading (`_contact`) under props, and keep names original.

## Phase 8: documentation and wrap-up

- Update `docs/VISUAL_STYLE.md` (new sections: facades and emission, per-theme grade values, far DoF, detail overlay, foliage, water) and
  the stage section of `docs/ART_DIRECTION.md`; add the new shaders to the shader inventory.
- Update `docs/STATUS.md` with what shipped, what was skipped and why, and the measured frame-time deltas.
- Final: `cargo fmt --all`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, the Godot tests above, and
  a menu-preview check (character select and creator previews still lit correctly).

## Out of scope here (follow-up plans)

Fighter motion polish (smear frames, squash and stretch, spring bones on hats and sashes, foot IK at platform edges) is the other half of
"professional". It lives in `fighter_view.gd` and `anim_library.gd` and deserves its own plan. SDFGI, SSR, SSAO and full PBR texture sets are
rejected for this style (see constraints). Lightmap GI is a possible experiment only after phases 1 to 6, and only if it can be shown not to
interfere with the custom `light()` function.

## Suggested order and sizing

Phase 0 (small), 1 (largest visible win, medium), 2 (small, high value), 3 (small to medium), 4 (medium), 5 (medium to large), 6
(medium), 7 (blocked on the owner's decision), 8 (small). Commit per phase, with the before/after stills in the same commit.
