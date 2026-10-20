# Grove pilot: baked light, then a Blender diorama (2026-10-20)

The owner's go-ahead after `FIDELITY_REVIEW.md`: pilot the two top ideas on the grove, the lowest-scoring stage. Step 1 bakes light into
the scenery and measures the gain; step 2 plans the grove's stage front as one authored Blender diorama.

## Step 1: baked occlusion

**Pipeline** (all presentation; the sim and its content are untouched):
1. `godot/scripts/stage_bake.gd` `export_scene`: on the stage demo with `--bake-export`, the built scenery's toon-shaded meshes go to
   `godot/art/baked/<theme>_<layout hash>/scene_in.glb`, each named by its build order (`b0007`), every other solid piece as an
   occluder (`o0003`), with a manifest (mesh sizes and surface counts, to refuse a stale bake).
2. `art/blender/bake_stage.py`: Blender gives each mesh over 0.4 square units a second UV set and its own occlusion texture (sized by
   its area, 10 texels a unit, 16 to 1024), bakes ambient occlusion with Cycles (reach 3 units) with the whole scene casting it, softens
   the bake noise, and writes the meshes back (`scene_baked.glb`) with the new UV set.
3. `stage_bake.gd` `apply`, at the end of `StageView.build`: for the exact stage the bake belongs to (theme and block layout), each
   built mesh is swapped for its baked copy and its toon material duplicated with `use_baked_ao` on. Custom stages on the grove theme,
   and any mesh whose size changed since the bake, are left as they are. `--no-bake` turns it off for comparisons.
4. `shaders/toon_common.gdshaderinc`: `use_baked_ao` (off by default, so the fighters never take it) multiplies the texture, read over
   UV2, into the material's AO, which darkens the ambient light fully and the direct light by `baked_ao_light` (set below).

Rebuild after any change to the grove's art:
```
tools/godot/Godot_v4.7.1-stable_win64_console.exe --path godot res://main.tscn -- --demo=stage --stage=4 --noui --bake-export --shots=<tmp>
"C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/bake_stage.py -- --dir godot/art/baked/<key>
```

**Settings:** occlusion reach 3 units, 256 samples, one soft blur pass; in the game `baked_ao_light` 0.85 (how far it also dims the
direct light) and `baked_ao_curve` 1.8 (deep crevices, clean open faces), chosen from three tunings at the match camera; the strength
fades from full near the fight to 0.3 by 60 units back, so the distance keeps its clear air (baked at full strength, the far ranges'
folds took more contrast than the style sheet allows). Meshes under 0.4 square units or wholly 40 units back are not baked (442 of 704).
`--bake-light=`, `--bake-curve=` and `--no-bake` on the demo command line are for tuning captures.

**Result** (`pilot_bake_grove.png`: match camera, the island close up, the side tree and hills, fighters in play; before left):
- Visible gain near the fight: the rolled lip now has a deep shaded underside, the quilted bank darkens under the rim, the bushes,
  mushrooms and stump sit in their own contact shade, the platforms shade their undersides.
- Small gain where the geometry is simple: the lawn slab, the smooth trunk and the flat quilt have little for occlusion to find. **The
  bake multiplies the detail the geometry has; it cannot add any.** That is the case for step 2.
- Distance: value 0.50 → 0.48, contrast 0.023 → 0.032, inside the style sheet.
- Fighters: unchanged (the shader switch is off for every fighter material; in play they measure 0.54 and 0.55 on a lawn of 0.70).
- Cost: draw calls 714 → 826 (each baked mesh has its own material copy, so fewer merge); GPU time the same within the machine's load
  today; load time up 0.1 to 2.6 s (the bake is read from glTF at run time); data 22 MB of meshes and 6.6 MB of textures.
- Lighting score for the grove: 2 → about 2.5 by eye.

**Known limits of the pilot:**
- The bake is read at run time from a folder Godot does not import (`.gdignore`), so it works from the project but would not ship in
  an exported build; production needs it imported.
- It is local: the bake output is git-ignored (about 58 MB with Blender's input file); rebuild it with the two commands above.
- Any change to the grove's art needs a re-bake; meshes whose size changed are skipped, not lit wrongly.
- Occlusion only. Bounce light (warm light from the lawn under the platforms, coloured shade) is the next bake layer, best done on the
  diorama.

## Step 2 plan: the grove's stage front as one Blender diorama

**Goal:** replace the code-built island (the quilted bank, lawn top and rolled lip), the great trunk and its foot, and the near props on
the top (stump, bushes, mushrooms, the three platforms) with one authored Blender scene built against the match camera. The background
stays code-built, folded and baked.

1. **Guides into Blender.** Extend the export to write the sim's block and platform rectangles, `FRONT_Z` and the match camera (field of
   view 30, its position and target in the demo frame) to the manifest; `art/blender/stage_guides.py` builds guide boxes and a matching
   camera, so every Blender render is the game's view.
2. **Look mocks, the owner picks** (rendered from that camera, before any modelling commitment):
   - A: the quilted bank kept (the owner's decision of 2026-10-19), modelled: puffed pillows with stitched creases, a thick rolled lip
     with moss tufts where top meets front;
   - B: roots and rocks after the grove reference: gnarled roots wrapping packed stones, mossy, under the lip (needs the owner to lift
     the quilted-bank hold);
   - C: a hybrid: the quilt above, roots wrapping it lower down.
3. **Build** in Blender (the fold kit, sculpted detail, live through the Blender connection), material families named so the game gives
   each its palette's toon material, as props are now. A budget for the whole diorama to agree: about 25,000 to 40,000 triangles (the
   workflow's 2,500 and 3,000 are per prop; the grove draws 0.7 M primitives today).
4. **Bake in Blender** on the diorama's own lightmap UVs: occlusion first, then bounce.
5. **In the game**, `StageArt` places the diorama for the exact grove layout (the bake's key); every other layout, such as the editor's
   custom stages, keeps the code-built front.
6. **Checks:** a test that the diorama's walkable top matches each rectangle's top within 0.02 and its ledge corners sit where they are
   grabbed; before/after from the same camera; fighters in play; the look bench; frame time; the six Godot tests.
7. **Production:** diorama and bake as imported resources under `godot/models/stages/`, so they ship.

**Owner decisions needed:** which front (A, B or C, from the mocks); the triangle budget; whether bake output goes into git.

## Step 2 mocks: three grove fronts (2026-10-20)

- **Guides:** the bake manifest now carries the stage's rectangles and front plane (`StageBake.export_scene`), and every demo shot writes
  its camera beside it (`<shot>.cam.json`: field of view, position, direction), for authoring from the game's view.
- **Builder:** `art/blender/make_grove_front.py --variant A|B|C` models the front on those rectangles (the island's outline kept within
  the blocks', starting under the turf lip's curl at 2.1 under the top), bakes its occlusion into linear vertex colours with the rest of
  the built grove imported round it, and writes a glTF. Families `quilt`, `button`, `earth`, `rock`, `root`, `moss`.
- **In the game:** `--front-mock=<glb>` on the stage demo (`StageBake.mock_front`) hides the code-built quilt, keel, block and inner
  ledge and places the model, painted from the palette with the scenery's toon material and its vertex occlusion.
- **First run fixed:** the hidden block and inner ledge sat inside the island against the new surfaces and baked them nearly black; they
  are left out of the bake's context.
- **Options** (`mocks/grove_front/`, match camera, wide and close-up; on the owner's fidelity page): A the quilt modelled (34,100
  triangles), B roots and rocks (12,700), C the quilt with roots (37,100). At the match camera only the top third of the front shows.

## Step 2 built: the grove's front, B roots and rocks (2026-10-20)

The owner picked B (after first saying A, then correcting), then asked to remove the rolled turf lip over it (it drew the eye from the
fighters and read as a separate piece).

- **Shape pass** (`make_grove_front.py`): blocky bevelled stones in their own tones (a per-stone tone multiplied into the baked
  occlusion), five thick ribbed roots sweeping diagonally across the stones with a branch each, the side roots wrapping round the
  island's curved flanks, some dangling under the keel, wide moss caps on the stones, a moss fringe under the lawn's edge. 16,300
  triangles. Every point clamped under the walkable top and within 0.9 of the blocks' exact outline.
- **Shipped:** `godot/models/stages/grove_front.glb`, an imported model, placed by `StageView` for the exact grove layout
  (`StageBake.FRONTS`, keyed like the bake) in place of the code-built quilt; `StageArt.front_modelled` leaves out the keel, the turf lip
  and its curl over it. Custom editor stages on the grove theme keep the code-built front. `--no-front` for comparisons.
- **Test:** `godot/tests/stage_front_test.gd` reads the sim's stages and checks each modelled front was built for one of them, stays
  under the walkable top, within a unit of the blocks' outline, and no further forward than the old lip. It caught both: points a unit
  outside the outline (the eased outline copied from the code-built quilt), and the top stones over the lawn's edge once the lip went.
- **Bugs fixed on the way:** the bake export picked up the previous stage still waiting to be freed (the demo builds the meadow first),
  704 meshes where the grove has 230, so in a normal match most of the first bake would have been skipped; the bake check now also
  compares the baked mesh itself, so a stale bake is refused instead of swapping wrong meshes in.
- **Result** (round 57, `pilot_grove_front.png`: match camera, wide, fighters in play): the front reads as roots and stones holding the
  island, close to the grove reference; the lawn's edge is one clean line. Fighters 0.55 and 0.56 on a lawn of 0.72. Fewer primitives
  than before (0.68 M against 0.71 M), draw calls about the same. Re-baked: 95 background meshes take baked light around the new front.
- **Tests:** all seven Godot tests pass.
