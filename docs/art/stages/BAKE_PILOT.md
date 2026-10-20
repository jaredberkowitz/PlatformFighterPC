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

## Step 3: the grove's trees (2026-10-20)

The owner, after the front: the background now lacked by comparison; improve the trees, leaves and ground the same way. Trees first.

- **Mocks** (`art/blender/make_grove_trees.py --variant A|B|C`, `--tree-mock=<glb>` on the stage demo; `mocks/grove_trees/`, today and
  A to C at the match camera and wide): A a twisted trunk of ropey ridges under big deep-pleated folded domes; B the same trunk with
  thick limbs carrying heaps of leaf clumps; C a trunk of broad folded planes under stacked pleated tiers. Every option has buttress
  roots splaying into the ground, a mossy foot, and its own occlusion baked into vertex colours. The first crowns were a third smaller than
  the code-built footprint (the side trees read as bonsai) and were scaled up.
- **The owner picked the fourth panel, C** (named "D" by its place in the sheets). Built with five tiers from two thirds up the trunk
  (from the top, the side trees read as pines on stilts): 2,860 triangles, within the prop budget. Shipped as
  `godot/models/stages/grove_tree.glb`, by theme (`StageBake.TREES`), so custom grove stages take it too: `_tree` draws its random
  numbers as before (nothing else moves) and records where each tree stands; the model is placed there at that tree's size. Bark in the
  front's roots' brown, so the great tree and its roots are one tree. `--no-trees` for comparisons.
- **Result** (round 58, `pilot_grove_trees.png`: match camera, wide, fighters in play): the great tree, the side trees, the folded hills
  and the pines elsewhere in the game now share one pleated language. Draw calls 830 → 793, primitives about the same. The great trunk is
  0.27 behind the fighters (0.55): a clear step, darker than the style sheet's mid-ground band (0.45 to 0.70); `--tree-bark=light` shows
  it in the old lighter bark if the owner wants it back. The crowns no longer sway (the code-built crowns did). Re-baked: 83 background
  meshes take baked light.
- **Tests:** all seven Godot tests pass.

## Step 4 mocks: the grove's ground and bushes (2026-10-20)

- **Hook:** `--ground-mock=<glb>` and `--bush-mock=<glb>` on the stage demo (`StageArt.ground_mock`, `StageBake.mock_ground`): the
  grove's hill rows, rise, forest canopy, far tree line and bushes are built into a node that is thrown away (their random numbers still
  drawn, so nothing else moves), the bushes' places kept in `bush_slots`; the model is placed in world space, each piece painted from the
  palette with its own vertex occlusion and the air of its depth. The far mountain ranges stay.
- **Builder:** `art/blender/make_grove_ground.py --variant A|B|C` (ground on the side trees' measured footing, -9.3 at x -30 and 32,
  depth -55; bush at size 1). A stepped ledges (27,500 triangles), B a forest of pleated trees falling away (136,000, to be cut if
  picked), C floating islets (10,200). Bushes: folded leaf heaps (A, C) or pleated fern rosettes (B).
- **First renders fixed:** the terraces' faces wound away from the camera (culled; the trees stood on nothing), and the cliffs went a pale
  grey under the distance air; now a deep teal.
- Options in `mocks/grove_ground/` (wide and match camera) and on the owner's fidelity page. Recommended C.

## Step 4 built: the grove's ground and bushes, A stepped ledges (2026-10-20)

- **Shape pass:** the ledges' tops in a deeper version of the island lawn's yellow-green (the far ones cooler) against deep-teal cliffs
  (in the rise's own teal, top and cliff ran together); the small pleated trees kept out of the space behind the fight on the near
  ledges (one behind the left platform drew the eye). 27,450 triangles; bush 372.
- **Shipped:** `godot/models/stages/grove_ground.glb` and `grove_bush.glb`, placed for the exact grove layout (`StageBake.GROUNDS`; the
  ground stands on that layout's side-tree footing) in place of the code-built hill rows, rise, forest canopy, far tree line and bushes.
  `--no-ground` for comparisons. Custom grove stages keep the code-built ground (they still take the modelled trees).
- **Result** (round 59, `pilot_grove_ground.png`: match camera, wide, fighters in play): stepped green ledges with teal cliffs and small
  pleated trees recede behind the island, in one language with its front and trees. 0.57 M primitives (0.68 M before), 805 draw calls
  (793). Fighters 0.55 and 0.56 stand on the path (0.75). The lawn strip in front of the path reads 0.54 to 0.60 since the front pass
  (step 2), close to the fighters' value; it is under their feet, not behind them, but worth a look in the next grove pass.
- **Re-baked:** 81 background meshes take baked light. **Tests:** all seven Godot tests pass.

## The meadow's front: mocks (2026-10-20)

The same method on the next stage. `art/blender/make_meadow_front.py --variant A|B|C` builds the front on the meadow's block (following
its 0.13 batter, from under the lawn's slab to the block's foot, where the meadow's hill takes over), with a grass overhang along the top
and its own occlusion bake (stones in their own tones). A folded rock in the light moss (3,800 triangles), B two courses of logs over
bevelled stone courses (14,500), C an earth bank with stones set in it (6,600). Options in `mocks/meadow_front/` (close-up, match,
wide) on the owner's fidelity page; recommended A.

Fixed on the way: `mock_front` hid the meadow's hill (a terrace stage's `Underside` is the hill it stands on, not a keel); the turf
tongues no longer hang over a modelled front; the overhang's tufts turned over had their undersides facing the sky inside the lawn's
slab and baked a dark purple band, so they stand upright.

## The meadow's front built: B, log and stone wall (2026-10-20)

The owner picked B. Shipped as `godot/models/stages/meadow_front.glb` (14,500 triangles), placed for the exact meadow layout
(`StageBake.FRONTS`), in place of the code-built faceted ledge and block box; the meadow's hill below stays. The step that hides the
code-built front is now shared by the shipped path and the mocks (`hide_code_front`): the shipped path had left the meadow's ledge in
place behind the model, and the block's box takes a number when a stage is rebuilt (`block2`), which the old exact-name check missed.

**Result** (round 60, `meadow_front_built.png`: match camera, wide, fighters in play): two courses of logs with cut ends over bevelled
stone courses in their own tones, a grass overhang along the top: the farm's own build under its fence and bales. Front 0.38 (in the
sheet's 0.35 to 0.55 band, warm grey; it was 0.36 light moss), lawn 0.64, fighters 0.59. Cost: 19 more draw calls, 0.2 to 0.4 ms GPU
(two A/B timings each way; a first reading of 6.2 ms was machine load). The alignment test covers both fronts. **Tests:** all seven pass.

## The beach's dock front: mocks (2026-10-20)

`art/blender/make_beach_front.py --variant A|B|C` builds the dock's supports on its collision (under the code-built deck, down into the
sea a unit under the block's foot, a front row and a back row), with pale waterline bands, weed and ripple rings at the water, and its
own occlusion bake (at half strength: under the closed deck every post baked nearly black). A timber pier (4,500 triangles), B stone quay
in the harbour town's quay stone (13,100), C bundled piles lashed in threes (6,100). Options in `mocks/beach_front/` on the owner's page;
recommended A.

In the game: `_dock_block` marks its fascia, piles, waterline pieces and braces `code_front` (the deck, joists and the shade under it
stay), which `hide_code_front` hides; the beach's families (`post`, `beam`, `waterline`, `iron`, `rope`, `harbour_stone`, `foam`,
`weed`) are painted from its palette; on the dock nothing falls into the fronts' calm haze (the stone went flat grey, the posts blue),
and the foam never does.

## The beach's dock front built: A, timber pier (2026-10-20)

The owner picked A. Shipped as `godot/models/stages/beach_front.glb` (4,500 triangles) for the exact beach layout (`StageBake.FRONTS`),
in place of the code-built fascia, piles, braces and waterline pieces (`code_front`); the deck, joists and the shade under it stay.

**Result** (round 61, `beach_front_built.png`: match camera, wide, fighters in play): thick chamfered posts in two rows, a bolted front
beam, X bracing in alternate bays and a low rail, pale waterline bands and ripple rings at the sea. Deck 0.48 unchanged; posts 0.32
(0.16 to 0.25 before); fighters 0.53 and 0.60. Cheaper: 732 draw calls against about 920, GPU time the same (3.1 ms both ways, two
timings each). **Tests:** all pass; `runner_test` failed once on its known shop-hint timing race (fixed on its own branch, not merged
here) and passed on the next two runs.

## The canyon's front: mocks (2026-10-20)

`art/blender/make_canyon_front.py --variant A|B|C` builds the mesa on the canyon's block: from under the sand top down to the desert floor
(-16), the column a touch narrower under the block and widening toward the floor as the code-built mesa did, a plain core behind the face
(so no gap shows the sky), in the canyon's strata colours with the bands set further apart (`strata_a/b/c`: three close reds read as
one under the dusk). A stacked slabs (10,900 triangles), B fluted cliff (42,300), C stepped ledges (14,700). Faces are subdivided before
the occlusion bake (on big faces the vertex bake sampled only the corners and smeared). Options in `mocks/canyon_front/` on the owner's
page; recommended A.

In the game: the strata tops are named `strata_tops` and marked `code_front`, and `hide_code_front` hides `strata_ledges` too; the
mesa's column (`Underside`) is hidden as a floating island's keel is.

## The canyon's front built: A, stacked slabs (2026-10-20)

The owner picked A. Shipped as `godot/models/stages/canyon_front.glb` (10,900 triangles) for the exact canyon layout
(`StageBake.FRONTS`), in place of the strata tile, its ledges and the mesa's column; the sand top stays. The code-built column is no longer
made under a modelled front (the model carries the mesa down to the desert floor itself).

**Result** (round 62, `canyon_front_built.png`: match camera, wide, fighters in play): tiers of chunky rock, two or three slabs to a tier,
in the same blocky language as the far mesas and the arch. Front 0.28 (0.32 before, so a touch calmer under the fight); sand 0.65;
fighters 0.52 and 0.50. Cost unchanged: 882 draw calls against 885, GPU 2.5 to 2.7 ms both ways. **Tests:** all seven pass.

## The city's front: mocks (2026-10-20)

`art/blender/make_city_front.py --variant A|B|C` builds the building on the city's block: from under the roof's coping and brass rail
(which stay, with the roof) down to the street, where the code-built shopfront and awning stay. There are four floors on the block. Under
its foot the building steps back `UNDER_RECESS` (0.9), as the code-built tower does, with a stone belt and a row of windows. The windows
are real openings with the glass set back 0.28. Lit rooms (about a third) are warm with curtains, and dark rooms are a deep blue. The
glass, the bulbs and the neon are unshaded (`glow` in `place_front`), and the wall is the city's own brick material (`brick` family).
- A brownstone: 51,600 triangles.
- B fire escape: 40,500 triangles.
- C deco piers: 36,100 triangles.

The front measures 0.29, 0.27 and 0.30 against today's 0.41 (the roof is 0.40, so today's wall was as light as the play surface).
Options in `mocks/city_front/` on the owner's page; recommended A.

The occlusion bake also drops the code-built facade's pieces from the context. They are exported as occluders lying on the wall's own
face, and left in, they baked the new wall black. Every family keeps part of its brightness (0.3; the frames and curtains in their reveals
0.55; the glass 0.6).

In the game: the facade's MultiMeshes, the shade band under the coping, the corner brackets and the tower's cornice and band are marked
`code_front`. The tower itself is hidden as a floating island's keel is.
