# Stages more their own: the style pass (2026-11-02)

The owner, after fidelity review 17: "do you have any recommendations on making them feel a bit more unique, stylized?" Then: "Please
continue with what you mentioned, starting with stepped skies."

**The diagnosis:** all five stages share one paper finish (the same fibre, print, cut edges, card clouds and folded trees). So they read as
one style in five palettes. The papercraft reference makes each place feel different by giving it its own craft material, not only its
own colours.

**The rules every step keeps:**
- Refine what is there; add nothing. No animals.
- The play areas stay calm and readable.
- The camera stays accurate to the reference. The grove stays violet.

## The plan

Across every stage:
1. **Stepped paper skies.** Each sky cut into flat paper layers with lit cut edges, like a diorama's backdrop, in place of a smooth
   gradient.
2. **Scenery on twos.** The windmill's sails, waves and smoke step at half the frame rate, like stop-motion. The fighters stay smooth.
3. **Each stage's paper colour on its cut edges.** White card for the meadow, kraft for the canyon, a blue-grey board for the city.

One craft per stage, applied to what is already there:
- **Meadow, patchwork and stitch:** the fields as fabric patches with stitched seams, the hedgerows as rows of stitching.
- **Canyon, corrugated cardboard:** the strata and mesas show their flutes on their cut edges; the sun's rays as cut strips.
- **Beach, origami:** the waves and sails as crisp folded paper; the foam as torn paper.
- **City, pop-up book:** the buildings as hinged pop-up layers, windows printed in halftone ink, the neon as coloured cellophane.
- **Grove, washi:** the canopy's edges and the lanterns in translucent paper glowing through; the mist as layers of tissue.

Order: the stepped skies first (every stage, no remodelling), then the canyon's cardboard (the weakest stage).

## Step 1: stepped paper skies (mock, 2026-11-02)

`sky.gdshader` `steps`: the sky over the horizon cut into layers. Each layer takes the sky's own colour (bands, glow, the sun's side and
the horizon haze) at its height, so a layer is flat up and down but still warmed across by the glow.
- The cuts sit closer together low down, where the cameras see most of the sky.
- They wander as if cut by hand, by a share of the gap under each.
- Each cut is a pixel soft, so it doesn't step at 1080 lines.
- Each layer has a light along its own cut edge and the soft shadow of the nearer layer under it.
- The cut-paper stratus and the card clouds stay in front.

`--sky-steps=T|A|B|C`, or the palette's `sky_steps` [layers, wander, cut edge, shadow, paper]:

| Option | What changes |
|---|---|
| Today | A smooth painted gradient |
| A, flat layers | Six layers, near-straight cuts, a light along each cut |
| B, cut layers | A, with the cuts wandering as if cut by hand and each nearer layer's soft shadow on the one behind |
| C, paper layers | B, with a paper fibre and a slight tone of its own on each sheet |

The play areas don't change, so the fighters' numbers hold. On the Stage Look Review ("skysteps65", `sk65_*`).

Recommended B: the wandering cuts and their shadows read as layers of cut paper, where A's near-straight cuts read as stripes. C looks the
same as B at normal size; its fibre only shows up close.
- It shows most on the canyon's sunset and the beach. The city's towers and the grove's trees hide most of their skies.
- The canyon's soft sun is a separate piece and stays smooth. A cut-paper sun would be a next step.

**Built B** (the owner's pick, 2026-11-02):
- `sky_steps=[6, 0.8, 0.7, 0.6, 0.0]` in each stage's palette (meadow, sunset, ocean, city, grove); `--sky-steps=T` brings back the smooth
  sky.
- The shipped skies measure the same as the mock's B.
- Before and after, the five stages at the wide camera (rows meadow, canyon, beach, city, grove; before left): `style/skies_built.jpg`.

All seven tests pass.

## Step 2: the canyon's cardboard (mock, 2026-11-03)

toon_common `corrugate` [board height, flute period, fluting, hollows]: what faces sideways shows the cut edge of stacked boards, counted
down from the front's top. Each board has a pale liner at its top and bottom and a fine seam to the next. Between the liners is the
fluting: a light wave from liner to liner, with the arches either side of it hollow and in shade. It is on the canyon's front beds and its
platforms' sides only. `--cardboard=T|A|B|C`, or the palette's [backdrop] `cardboard` (StageBake.CARDBOARD_TRY):

| Option | What changes |
|---|---|
| Today | The natural rock beds |
| A, light | Boards with fluting, the hollows a soft shade |
| B, deep | The hollows deep, as a card's cut edge shows them |
| C, bold | B, with boards half as tall again and broader flutes |

A first try with a light wave and pale hollows read as a zigzag print. The arches were then drawn hollow and in shade.

**Honest read: none of these reads as cardboard.** Flutes need straight, flat boards to read as card. On the front's eroded, wandering beds
(review 17's natural strata), they read as a woven chevron pattern, like a blanket, worst at the wide camera. Recommended: keep today's
front. On the Stage Look Review ("cardboard66", `cb66_*`).

Proposed instead for the canyon's own craft: **torn paper**. Each bed's edge torn, with the paper's pale core showing along the tear. It
suits rock shapes, and it is a classic papercraft way to draw strata.

## Step 2, again: the canyon's torn paper (mock, 2026-11-03)

The owner: "Yes, torn paper". `make_canyon_front.py --tear 1` hangs a torn edge along the top of each hard bed's top course:
- a strip a hair in front of the face, from the lit ledge down to a jagged, fibrous edge;
- finer than the bed's own columns, and tapering to nothing at the bed's ends;
- its own families, `tear` and `tear_shade` (a thin shadow following the tear).

The beds are unchanged; the tear has its own random numbers. The game colours the tear or hides it (StageBake.TEAR_TRY): the palette's
[backdrop] `torn`, or `--torn=T|A|B|C`, with T hidden.

| Option | What changes |
|---|---|
| Today | No tear: the natural beds |
| A, soft | The tear in a pale tone of the rock |
| B, paper core | The paper's cream core |
| C, with its shadow | B, with the thin shadow the torn layer lays under it |

The front's value holds (0.55 to 0.56) and its contrast rises a little (0.118 to 0.128); P1's gap holds (0.159). On the Stage Look Review
("torn67", `tp67_*`).

Recommended B: it reads as torn paper at both cameras and keeps the rock. C's shadow reads as an ink line, which the scenery doesn't use
anywhere else. A is faint at the wide camera.

**Built B** (the owner's pick, 2026-11-03):
- sunset.cfg [backdrop] `torn="B"`; the canyon's front rebuilt with `--tear 1` and the canyon re-lit. `--torn=T` hides the tears.
- The cardboard trial was removed (the toon shader's `corrugate`, StageBake's `CARDBOARD_TRY`), since the owner chose torn paper over it.
  Its mock stays on the Stage Look Review and in git history.
- Before and after at the match camera (top) and wide (bottom), before left: `style/torn_built.jpg`.
All seven tests pass.

## Step 3: the city's pop-up book print (mock, 2026-11-03)

The owner: "Next stage's craft". facade.gdshader `halftone` prints the skyline towers like a pop-up book's pages:
- each wall is printed flat, and its shade is carried by halftone dots of a darker ink on a 45-degree grid;
- each dot is sized so the wall keeps the tone it had;
- the dots fade back to the smooth shade where they would be under a few pixels.

Only the skyline's own shader changes. The middle and far layers' dots are larger in world units, so the dots stay about one size on screen.
The palette's [backdrop] `halftone`, or `--halftone=T|A|B|C` (StageArt.HALFTONE_TRY):

| Option | What changes |
|---|---|
| Today | Smooth shade |
| A, fine | Fine dots on the side walls only |
| B, printed shade | The fall toward the street in dots as well |
| C, bold | B, in larger dots of a darker ink |

P1's gap holds (0.150), and so do the backdrop's value and contrast. On the Stage Look Review ("halftone68", `ht68_*`).

Recommended B: its dots read as print on the towers' side walls without busying the window fronts behind the fighters. A barely shows; C's
dots read coarse.
- Most of the skyline seen is window fronts, so it is a modest change, showing most on the side walls and at the wide camera.
- Printing the fronts as well would be bolder, but it would busy the backdrop right behind the fighters, which the city's notes have kept
  calm.

**Built B** (the owner's pick, 2026-11-04):
- city.cfg [backdrop] `halftone="B"`; `--halftone=T` shows the smooth shade.
- The shipped skyline measures the same as the mock's B.
- Before and after at the match camera (top) and wide (bottom), before left: `style/halftone_built.jpg`.
All seven tests pass.

## Step 4: the beach's origami sea (mock, 2026-11-04)

The owner: "Next stage's craft". The beach's sea was already cut paper: tiers of card, each with a scalloped, lit edge. water.gdshader
`fold` folds each tier instead of shading it smoothly:
- the near face turned from the light and the far face toward it, with a crisp crease between, lit as a ridge;
- with pleats, the crease zigzags along the tier, so the faces are triangles, the rising ones lit and the falling ones shaded.

Only the water's shader changes. The palette's [backdrop] `sea_fold`, or `--origami=T|A|B|C` (StageArt.SEA_FOLD_TRY):

| Option | What changes |
|---|---|
| Today | Each tier shaded smoothly toward its edge |
| A, folded | Each tier folded along a straight crease |
| B, pleated | The crease zigzags, so the faces are triangles |
| C, wide pleats | B, with pleats twice as wide and a brighter crease |

P1's gap holds (0.165 to 0.170), and so do the sea's value and contrast. On the Stage Look Review ("origami69", `og69_*`).

Recommended B: its pleats read as folded paper across the near sea.
- It shows mostly at the wide camera. At the match camera the sea is mostly far water and reflections.
- The tiers wander, so the facets are irregular, and up close they can read as low-poly glass rather than pleats. C pushes that further;
  A is the quiet choice.

**Built B** (the owner's pick, 2026-11-04):
- ocean.cfg [backdrop] `sea_fold="B"`; `--origami=T` shows the smooth tiers.
- The shipped sea measures the same as the mock's B.
- Before and after at the match camera (top) and wide (bottom), before left: `style/origami_built.jpg`.
All seven tests pass (the runner test failed once on a coin pickup, "a coin picked up is gone: 9 from 9", and passed on the rerun: an intermittent failure in the run mode, not the sea).

## Step 5: the meadow's patchwork and stitch (mock, 2026-11-04)

The owner: "Next stage's craft". Running stitches, each a dash of thread, a gap, a dash, in their own family `stitch` that the game colours
or hides (StageBake.STITCH_TRY):
- along the lawn's card edge, right under the fighters (`make_meadow_play.py --stitch 1`);
- along the crown of every hedgerow on the farmland, like a quilt's seams (`make_meadow_ground.py --stitch 1`); the far hills are already
  drawn as a quilt.

The stitches have their own random numbers, so the land comes out as without them. With them hidden, the rebuilt and re-lit meadow measures
the same as today. The palette's [backdrop] `stitch`, or `--stitch=T|A|B|C`, with T hidden:

| Option | What changes |
|---|---|
| Today | No stitches |
| A, lawn edge | A running stitch along the lawn's card edge, in a cream thread |
| B, and the hedgerows | A, with seams along the hedgerows |
| C, red thread | B, in a red thread |

On the Stage Look Review ("stitch70", `st70_*`).

Recommended A: the cream stitch reads as a sewn seam right under the fighters.
- The hedgerows' seams don't land. At the wide camera they are small white dashes, and at the match camera they barely show. In red they
  read as berries.
- C's red thread along the lawn is the bolder craft accent, but it is a saturated red right under the fighters.

The mock's meadow models (the play area and ground with stitches, and the meadow re-lit) are in the worktree, not committed. Their backups
are in the session scratchpad (stitch15).
