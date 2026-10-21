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

## The city's front built: A, brownstone (2026-10-20)

The owner picked A. Shipped as `godot/models/stages/city_front.glb` for the exact city layout (`StageBake.FRONTS`). It replaces the
code-built facade, its windows and the tower under the block; the roof, its coping and brass rail, the shopfront and the street stay.
Under a modelled front the code-built tower is no longer made. Trimmed for the build from 51,600 to 26,500 triangles: the trim's bevels are
single chamfers, and the body behind the face (`brick_back`) is not subdivided.

**Result** (round 63, `city_front_built.png`: match camera, wide, fighters in play): brick pilasters between pairs of windows, arched
stone heads, a stone band at every floor and a bracketed cornice under the roof's edge; the floor under the block steps back over the
shopfront. Front 0.29 (0.41 before, as light as the roof's 0.40), so the play surface now reads first. Cost: about 15 more draw calls
(892 to 897 against 879 to 881) and 0.2 to 0.4 ms more GPU time (3.1 to 4.4 ms against 2.9 to 3.9 ms, two timings each, on a loaded
machine). **Tests:** all seven pass.

## The meadow's background: mocks (2026-10-20)

The grove's ground method on the meadow: its two hill rows, the rise behind the stage with its fields, and the pine clusters. The far
mountains, windmill, cottages and farm stay code-built.
- **Layout export:** `--bake-export` also writes `ground.json` (`StageBake.export_ground`): the hill rows' and the rise's bumps, each pine
  cluster's place, size and turn (`StageArt.pine_slots`) and the farm's span (`farm_span`). The modelled crests follow the same bumps,
  so whatever stands on them keeps its feet.
- **Hook:** `--ground-mock=` and `--bush-mock=` now work on the meadow. The hills, rise, fields and pine clusters are built into a node
  that is thrown away, with their numbers still drawn. The pine model (the "bush") goes at each cluster's spot. The farm is kept,
  re-seated on the modelled valley (`StageBake._height_at`, a ray against the model's triangles), and the land is kept level under it.
- **Builder:** `art/blender/make_meadow_ground.py --variant A|B|C`.
  - A deep folds: 11,400 triangles, pines 1,800 a cluster.
  - B paper layers: 17,600 triangles, pines 1,800. Its card edges are a pale tint of the rise's green; in cream they read as snow.
  - C faceted: 3,900 triangles, pines 290.
- **Note:** all three lose the code-built fields' round hedge bushes.
- **Next:** options in `mocks/meadow_ground/` (with a local `review.html`) and on the owner's page; recommended A.

## The meadow's background built: B, paper layers (2026-10-20)

The owner picked B, with three asks: calm the bright greens, stop things overlapping, and bring the hill under the stage into the style or
remove it. Shipped as `godot/models/stages/meadow_ground.glb` (20,500 triangles) and `meadow_pines.glb` (1,800 a cluster), for the exact
meadow layout (`StageBake.GROUNDS`).
- **Greens:** the hill rows, the rise, its fields and the hill are a step calmer than the palette (`StageBake._calm`: saturation x0.72,
  value x0.95). Each sheet's cut edge is a pale step of its own land (`card_far`, `card_near`, `card_rise`, `card_hill`), not one lime.
- **The stage's hill** is stacked sheets too: contours of the code-built hill's fall, thin near the wall and thicker further out
  (`hill_layers`). Under a modelled ground the code-built hill is not made. The hay, barrels and rocks on it are gathered in `HillProps`
  and re-seated on the sheets, as the farm is on the valley.
- **Overlaps:** under the modelled ground, no pine cluster stands within 22 units of the middle (they rose over both platforms), nor within
  5 of a cottage (they hid both). The disc pines are as narrow as the code-built pine.
- **Result** (round 64, `meadow_ground_built.png`: match camera, wide, fighters in play): the valley, hills and the hill under the stage
  read as one paper model. The valley's saturation is 0.22 (0.40 before); the lawn is unchanged at 0.63. Cheaper: 770 draw calls against
  843, 0.63 M primitives against 1.30 M, GPU 2.2 ms against 2.4 ms (two timings each). **Tests:** all seven pass.

## The beach's background in paper: mocks (2026-10-20)

The owner's paper pick on the meadow, carried to the beach: the harbour headland's hills and the three islands' mounds.
- **Kept:** the sea, quay, town, palms, rocks, hut, lighthouse and stilt hut stay code-built.
- **Layout export:** `ground.json` now also carries each island's sand and green mounds and each headland hill (`island_specs`,
  `headland_specs`), as the half-axes of their domes. Contours of those domes keep the houses, palms and hut on their footing.
- **Hook:** under a ground mock the mounds and hill domes are built into a node that is thrown away.
- **Builder:** `art/blender/make_beach_ground.py --variant A|B|C`.
  - A card contours: 14,900 triangles.
  - B foam board, thicker layers with a pale lip: 11,600 triangles.
  - C pop-up, the tall back hills as standing cut-out flats and the rest as card contours: 11,000 triangles.
- **First renders fixed:** concentric contours read as a mint wedding cake. Each layer now drifts and lobes by the dome's own shape, and
  the green land's cut edges are a shade under its tops (the camera sees the edges more than the tops out there).
- **Next:** options in `mocks/beach_ground/` (with a local `review.html`) and on the owner's page; recommended C.

## The beach's background built: C, pop-up (2026-10-20)

The owner picked C. Shipped as `godot/models/stages/beach_ground.glb` (11,000 triangles) for the exact beach layout
(`StageBake.GROUNDS`, with no second model: `GROUNDS` now allows an empty "bush"). It replaces the code-built island mounds and headland
domes; the tall back hills are standing cut-out flats, and the low hills and the islands are card contours.

**Result** (round 65, `beach_ground_built.png`: match camera, wide, fighters in play): the headland reads as a pop-up behind the town,
with a clean, calm silhouette, and the islands as stacked paper under their palms. The distance's value is unchanged (0.64) and the town's
is the same; the deck reads 0.49. Cost unchanged: 738 draw calls against 730, GPU 2.2 ms both ways (two timings each). **Tests:** all
seven pass.

## The grove's background in paper: mocks (2026-10-20)

The paper style on the grove: the ledges behind the island, the small trees on them and the far violet ranges. The side trees keep their
footing; the island, its front and the great tree are unchanged.
- **Far land:** `ground.json` now carries each painted range's crest (the highest vertex in 4-unit bins of its mesh, and its centre
  depth). `ridge()` returns the placed range model and marks it `ridge`. A ground model with `range*` pieces hides the marked ranges
  (`StageBake.place_ground`), so the shipped grove ground keeps today's ranges until a paper one ships.
- **Builder:** `art/blender/make_grove_paper.py --variant A|B|C`, every look with the ranges as three standing cut-out flats each.
  - A card terraces: 41,100 triangles.
  - B pop-up: shelves with a scalloped grass fringe and jagged cliff cards, and cross-slotted cut-out trees; 13,400 triangles.
  - C contour hills: 54,600 triangles (216,000 before its rings and hills were cut down).
- **Range flats tuned:** the crest's per-bin peaks made a sawtooth, so it is smoothed (a rolling max, then a light average). At the
  painted range's height the flats sat low; much taller, they walled off the sky. They are now 1.08, 1.0 and 0.9 of it, back to front.
- **Next:** options in `mocks/grove_paper/` (with a local `review.html`) and on the owner's page; recommended A.

## The grove's background built: A, card terraces (2026-10-20)

The owner picked A. Shipped as `godot/models/stages/grove_paper.glb` (30,300 triangles) in place of the faceted stepped ledges
(`grove_ground.glb`, retired; `make_grove_ground.py` still builds it), with the same modelled bushes (`StageBake.GROUNDS`).
- **Pieces:** each ledge is a stack of card sheets with a green top over teal cut edges, the small trees are paper disc pines, and the
  far ranges are standing cut-out flats, which hide the painted ranges.
- **Trimmed for the build:** five discs of eight points a pine (six of twelve made the trees most of the triangles).

**Result** (round 66, `grove_paper_built.png`: match camera, wide, fighters in play): the island stands in one paper diorama with the
meadow and the beach. The background is lighter than the faceted ledges: the mid-ground reads 0.50 (0.38) and the distance 0.64 (0.49).
That is inside the style sheet's mid-ground band, but closer to the fighters' 0.55; worth a look if the fighters lose any edge against it.
The lawn is unchanged (0.67). Cost: 814 draw calls against 806, GPU 2.4 ms both ways (two timings each). **Tests:** all seven pass.

## The canyon's background in paper: mocks (2026-10-20)

The paper style on the canyon: the buttes, the rock arch and the dunes. The desert floor, cacti, telegraph poles and sun stay code-built.
- **Layout export:** `ground.json` now carries each butte's and the arch's bounds. A prop is placed as one child per part, so the bounds
  merge every part placed (`StageArt._bounds_from`); the last child alone was only its rubble skirt. The dunes come with the ranges' crests.
- **Hook:** under a ground mock the buttes and the arch are built into a node that is thrown away. The dunes hide behind the model's
  `range*` cut-outs.
- **Builder:** `art/blender/make_canyon_paper.py --variant A|B|C`.
  - A card stacks, plates in the strata on a low skirt: 9,300 triangles.
  - B pop-up cut-outs: 3,800 triangles.
  - C both, near stacks and far cut-outs: 7,600 triangles.
- **First renders fixed:** the pale band was a step too pale in thin plates (the arch striped like candy; now 0.35 toward cream, not
  0.6). The skirts at the bounds' full width read as hat brims, and B's palest card stood in front and washed the buttes out (now at the
  back).
- **Next:** options in `mocks/canyon_paper/` (with a local `review.html`) and on the owner's page; recommended A.

## The canyon's background built: A, card stacks (2026-10-20)

The owner picked A. Shipped as `godot/models/stages/canyon_paper.glb` (9,400 triangles) for the exact canyon layout
(`StageBake.GROUNDS`, no second model). It replaces the modelled buttes, the arch and the painted dunes: the buttes and the arch are
stacks of thick card plates in the strata, and the dunes are cut-out flats along their crests.
- **Dunes deepened:** at the painted dunes' pale sands the flats took the full air of their depth and faded into the horizon. They are now
  a rosier, deeper sand (`dune_far`, `dune_near`), set in Godot.

**Result** (round 67, `canyon_paper_built.png`: match camera, wide, fighters in play): the buttes, the arch and the mesa's stacked-slab
front read as one stacked-card place. The near butte behind the fight reads 0.55 (0.50 before); the sand is unchanged (0.65). Cost: 907
draw calls against 884, GPU 2.2 ms both ways (two timings each). **Tests:** all seven pass.

## The city's skyline in paper: mocks (2026-10-20)

The towers are not props: each skyline layer is one procedurally built mesh whose windows, signs and billboards a facade shader paints.
Rebuilt in Blender they would lose all of that, so the paper looks are made in Godot (`--paper-sky=A|B|C` on the stage demo; unchanged
without it).
- **How:** `_paper_skyline` squashes each building into a standing card at its own stagger in depth. `_paper_edges` traces each card's
  silhouette from its own vertices (the highest in each half-unit, gaps filled) for a pale die-cut edge and a soft shadow behind it.
  The street (road, shops, lamps, cars) stays: it is built detail at the fight's foot.
- **Looks:**
  - A die-cut: thin staggered cards with a pale edge.
  - B shadowed: one flat card per layer, each tower shadowed.
  - C both: thicker staggered board with the edge and the shadow.
- **First renders fixed:** squashed alone, the cards looked exactly like the boxes from the match camera. The edge and the shadow are
  what read as paper. The edge cards were wound the wrong way (culled), and C's lean pushed the crowns behind their own edge, so it was
  dropped.
- **Next:** options in `mocks/city_paper/` (with a local `review.html`) and on the owner's page; recommended C.

## The city's skyline built: C, die-cut edge and shadow (2026-10-20)

The owner picked C. The city palette's [backdrop] now sets `paper_sky="C"` (`StageArt._paper_look`; `--paper-sky=off` shows the solid
towers for comparisons), so every city stage takes it, the run mode's included. Each tower is a thicker staggered card with a pale
die-cut edge and a soft shadow. Each layer's edges and shadows are one mesh of each kind: one per building cost sixty draw calls.

**Result** (round 68, `city_paper_built.png`: match camera, wide, fighters in play): the skyline reads as cut card standing in layers,
its windows, signs and billboards printed as before. The towers' value is about the same (0.31, was 0.30); the distance is a step
lighter (0.39, was 0.36). Cost: about 899 draw calls against 893, GPU the same (2.8 to 3.1 ms both ways, two timings each).
**Tests:** all seven pass.

With this, every stage has its background in paper: the meadow (layers), the beach (pop-up), the grove (card terraces), the canyon (card
stacks) and the city (die-cut cards).

## The meadow pilot: paper play area and the paper finish, mocks (2026-10-20)

Ideas 1 and 2 of `FIDELITY_REVIEW_2.md` on the meadow.
- **Paper finish:** `pf_paper_finish` (a shader global, [fibre grain, sheet wobble, cut-edge rim]) in `toon_common.gdshaderinc`, on
  scenery only (the shader's `fighter` flag), off by default; `--paper-finish=A|B|C` for the captures (`Lighting.paper_finish`). It is
  not the retired paper look: ink and light are untouched.
- **Paper play area:** `--play-mock=<glb>` (`StageView`): everything the blocks and platforms build is hidden and the model placed
  (`StageBake.place_play`). The builder is `art/blender/make_meadow_play.py --variant A|B|C`, on the manifest's collision, with every
  walking surface at the sim's height.
  - A card slab, with finish A: 2,600 triangles.
  - B layered sheets, with finish B: 5,300 triangles.
  - C pop-up ground row, with finish C: 2,600 triangles.
- **First render fixed:** the toon option `facets` rebuilds the normal from screen derivatives. On the lawn's flat top it turned over: the
  lawn lit as an underside (dull) and the fighters' contact shadow decals, which only fall on what faces up (`normal_fade`), vanished.
  The play pieces are flat, so they take no facets.
- **Next:** options in `mocks/meadow_play/` (with a local `review.html`) and on the owner's page; recommended B.

## The meadow pilot built: B, layered sheets with the paper finish (2026-10-20)

The owner picked B. Shipped as `godot/models/stages/meadow_play.glb` (5,300 triangles) for the exact meadow layout (`StageBake.PLAYS`;
`--no-play` for comparisons).
- **Pieces:** the lawn is a green sheet over soil and stone sheets with an inlaid cream path, and the platforms are green sheets over
  plank card with a scalloped grass fringe. It replaces everything the blocks and platforms built.
- **Paper finish:** the meadow palette's [backdrop] `paper_finish="B"` turns on grain, wobble and the cut edge for the meadow's scenery
  (StageView sets the global from it; `--paper-finish=off` for comparisons; other stages stay off).
- **Test:** `tests/stage_front_test.gd` now checks every `PLAYS` model. Each walkable top must stand at the sim's height (within 0.02)
  across its span, with nothing poking up out of it.

**Result** (round 70, `meadow_play_built.png`: match camera, wide, fighters in play): the lawn and platforms read as paper with the
background. The fighters keep their contact shadows. The lawn region measures lighter and less saturated (value 0.67, was 0.63;
saturation 0.38, was 0.64): the inlaid path is wider than the painted one. Cheaper: 669 draw calls against 770, 0.49 M primitives
against 0.63 M, GPU 2.1 ms against 2.2 ms (two timings each). **Tests:** all seven pass.

## The paper play area on every stage: mocks (2026-10-20)

The meadow pilot rolled out to the grove, canyon, beach and city.
- **Builder:** `art/blender/make_play.py --theme grove|sunset|ocean|city --variant A|B|C`. It builds each stage's exposed top and its
  platforms on the collision, with the stage's own materials and edge (grass and leaf scallops, sand over strata with rock jags, the
  dock's boards over joists and a beam, roofing with a coping and girders). The shipped meadow keeps `make_meadow_play.py`.
  - A card slab: 1,700 to 3,100 triangles.
  - B layered sheets: 2,800 to 5,900 triangles.
  - C pop-up ground row: 1,700 to 2,900 triangles.
- **Painter:** `StageBake.place_play` paints the generic families (`top`, `top_card`, `layer1`, `layer2`, `fringe`, `steel`...) per
  theme.
- **Kept:** the city's scaffold towers are marked `keep_play` and stay under a modelled play area; they are structure.
- **Test:** `tests/stage_front_test.gd` now skips covered tops (the grove's lower tiers).
- **Picks recommended:** grove B, canyon B, beach A, city A. Options in `mocks/play_all/` (with a local `review.html`) and on the
  owner's page.
- **Gotcha:** in a bash double-quoted path, `\$` escapes the dollar sign. `"$W\${1}_A.glb"` wrote every model to one literal
  `playall${1}_A.glb`, so use `/`.

## The paper play area built on every stage: grove B, canyon B, beach A, city A (2026-10-20)

The owner's picks. Shipped as `godot/models/stages/{grove,canyon,beach,city}_play.glb` (5,900 / 5,400 / 1,700 / 2,400 triangles) for each
exact layout (`StageBake.PLAYS`). Each stage's palette sets its paper finish (grove and canyon "B" with the cut edge, beach and city "A").
`--no-play --paper-finish=off` for comparisons. The alignment test checks all five play models.

**Result** (round 71, `<stage>_play_built.png`: match camera and fighters in play, before above after): every play area reads as
paper card with the backgrounds, and the fighters keep their contact shadows. Cheaper on every stage (draw calls before → after): grove
813 → 482, canyon 907 → 570, beach 738 → 732, city 899 → 787; GPU time the same or lower. **Tests:** all seven pass.

## Readability pass (2026-10-20, review 2's idea 4)

Fighters measured against what is behind and under them in play (the orange fighter's body is about 0.52):
- **Canyon:** the play area's sand is a step paler and cooler (`place_play`, sunset `top`). Its saturation went from 0.46 to 0.34, value
  0.76 to 0.80, hue 32 to 30 against the fighter's 20, so the orange fighter no longer stands on sand of its own hue. The buttes are a
  step deeper (`rock_a/b/c`).
- **Grove:** the paper terraces' cliffs are deeper (`cliff` 0.1, 0.24, 0.23) with fainter pale edges. Behind the fighters they went from
  0.50–0.52 to 0.46; the distance air lifts them, so more would blacken the grove's distance.
- **Meadow:** the inlaid path is narrower (0.7 either side of the play line, was 1.1). The lawn's saturation went from 0.38 to 0.43.

Before/after (fighters in play, before above after): `readability_pass.png`. **Tests:** all seven pass.

## Mood light in paper: mocks (2026-10-20, review 2's idea 5)

Light as cut paper on the three stages where it suits the setting. The owner approved it as an addition.
- **Builder:** `art/blender/make_mood.py --theme city|grove|ocean --variant A|B|C`, built in world space from the bake's manifest (and,
  for the beach, the dock's posts read from `beach_front.glb` via `--front`).
  - City searchlight beams: A two soft beams; B four narrower beams in pairs; C three flat paper beams with a deeper edge.
  - Grove lanterns: A six under the platforms; B sixteen along the strings; C both, with warm pools on the lawn.
  - Beach ripple rings round the dock's posts and the boardwalk's: A two rings; B plus a near-side crescent and a faint third ring;
    C two rings plus paper sparkles on the open water. The match camera crops the dock's own waterline, so the boardwalk behind it
    carries the rings into that shot.
- **Placer:** `StageBake.place_mood` (`--mood-mock=<glb>`, from `stage_view.gd`) gives each family an unshaded material (additive for
  beams, halos, pools, rings and sparkles), with no shadows; caps and cords are toon.
- **Gotchas:**
  - glTF export drops a vertex colour's alpha, so each fade is baked into the rgb (shade times fade, alpha 1) and the light is additive.
  - Keep only the `Col` attribute, set active and for rendering: with a second colour set beside it, the export wrote a white
    `COLOR_0`, which the game reads.
  - Rings of 0.4 to 0.6 hid behind the posts; 0.5 to 0.8 and 1.05 to 1.2 read from the match camera.
- **Picks recommended:** city B, grove C, beach B. Options in `mocks/mood/` (with a local `review.html`) and on the owner's page.
**Tests:** all seven Godot tests pass.

## Mood light: built (2026-10-20)

The owner's picks:
- **City B:** four searchlight beams in pairs, rising from behind the skyline (48 triangles).
- **Grove C:** lanterns under the platforms and along the strings, with warm pools on the lawn (4,100 triangles).
- **Beach B:** ripple rings with a near-side crescent round the dock's and the boardwalk's posts (9,280 triangles).

Shipped as `godot/models/stages/{city,grove,beach}_mood.glb` for each exact layout (`StageBake.MOODS`, placed by `place_mood` from
`stage_view.gd`). `--no-mood` to compare; `--mood-mock=<glb>` still tries one. `tests/stage_front_test.gd` checks each was built for a
sim stage and stays behind the fighters' plane over the play space.

**Result** (`mood_built.png`, before left, after right: city wide, grove match, beach match and wide): the city's sky gets its beams,
the grove its warm lanterns, and the beach's waterline its rings, the boardwalk's in the match camera. The fighters are untouched.
**Tests:** all seven Godot tests pass.
