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

**Built A** (the owner's pick, 2026-11-04):
- meadow.cfg [backdrop] `stitch="A"`; the meadow's play area rebuilt with `--stitch 1` and the meadow re-lit. `--stitch=T` hides it.
- The ground stays as it was, without the hedgerows' seams, since B and C weren't taken.
- Before and after at the match camera (top) and wide (bottom), before left: `style/stitch_built.jpg`.
All seven tests pass (the runner test failed twice in four runs, on its coin pickup and "the way out", and passed on the rerun: it waits on real-time timers and flakes when the machine is busy).

## Step 6: the grove's washi (mock, 2026-11-04)

The owner: "Next stage's craft". Washi is the Japanese paper that glows when light comes through it, its long fibres and thicker clouds
showing against the light. The grove already has the things washi is made into: paper lanterns and mist. One seamless texture holds the
paper (`tools/art/make_washi.py`, `godot/art/stage/png/washi.png`): long fibres, the sheet's thicker clouds and finer fibres, one per
channel.
- **The lanterns** (`shaders/washi.gdshader`), on the strings and under the platforms: each one's paper lit from inside in place of a flat
  glow. It is whiter where it faces the eye, and deeper and warmer toward its edges, where the light crosses more paper. Its fibres and
  clouds show darker against the light.
- **The mist** (`shaders/toon_tissue.gdshader`): each bank's three stacked sheets as tissue with no thickness, their card edges gone. Each
  sheet is see-through, so the valley shows faintly under one sheet and hardly under three. The edge is torn, wandering in and out with
  fibres reaching past it, and the paper's fibres and clouds lie over the sheets (toon_common `washi_paper`, `tissue`). The sheets are
  rebuilt in the game from the shipped ground (StageBake._tissue_mesh): nothing is remodelled or re-lit.

The palette's [backdrop] `washi`, or `--washi=T|A|B|C` (StageArt.WASHI_TRY):

| Option | What changes |
|---|---|
| Today | Flat lantern glows; the mist in card sheets |
| A, lanterns | The lanterns in washi, lit from inside |
| B, and the mist | A, with the mist in tissue |
| C, bolder | B, the tissue thinner, its fibres bolder, the lanterns' paper heavier |

With `tissue_sat` the sheets' own colour is more saturated: at its own colour, with the green valley showing through, the mist measured
grey (saturation 0.04, from 0.10). Tinted, B's mist measures saturation 0.07. The play area measures the same in every option (value
0.59 at the match camera).

On the Stage Look Review ("washi71", `wa71_*`).

Recommended B: the lanterns read as paper with a lamp inside, and the tissue mist is the grove's own, since no other stage has
see-through paper.
- The lanterns are small at the match camera. Their washi shows mostly as a whiter middle and a golden rim, not as fibres. They also glow a
  little less around them, since their rims are darker.
- The tissue gives up the crisp stacked card the mist had. It is softer, closer to real mist.
- C's thinner sheets show more of the valley, but its fibres start to read as scratches at the edges.

Capture note: while a full-screen game was open on this machine, the captures hung with vsync on (the window not responding, its CPU
idle). With `--disable-vsync` before the scene, each took about 20 seconds.
All seven tests pass.

**Built B** (the owner's pick, 2026-11-04):
- grove.cfg [backdrop] `washi="B"`. Nothing is remodelled or re-lit: the lanterns take their washi material, and the game rebuilds the mist
  sheets as tissue. `--washi=T` shows the flat lanterns and the card mist before.
- The shipped grove measures the same as the mock's B (the mist at value 0.54, saturation 0.07).
- Before and after at the match camera (top) and wide (bottom), before left: `style/washi_built.jpg`.
All seven tests pass.

With this, each stage has its own craft. Left of the plan are the two changes across every stage: scenery on twos, and each stage's
paper colour on its cut edges.

## Step 7: scenery on twos (mock, 2026-11-04)

The owner: "Scenery on twos". A stop-motion set is shot one picture at a time, and between pictures nothing moves. Here the scenery takes
its own pictures (`scripts/scene_clock.gd`), and everything in it moves only on them, all on the same frames:
- in the shaders (`shaders/scene_time.gdshaderinc`): the water, the leaves, the clouds' shadows, the sun's rays, the city's signs and
  windows, and the puddles;
- in the scripts: the windmill's sails, the boats, the traffic, the clouds, the smoke and the travellers (StageArt's movers,
  `scripts/world_motion.gd`), and the motes.

The fighters, the camera and the effects (already drawn on twos, `scripts/particles.gd`) stay smooth. The palette's [backdrop]
`scene_steps`, or `--scene-steps=T|A|B|C` (StageArt.SCENE_STEPS_TRY):

| Option | Pictures a second |
|---|---|
| Today | Every frame (60) |
| A, on twos | 30, as the effects |
| B, on fours | 15 |
| C, on sixes | 10 |

Measured from 11-second recordings (the `stage_clip` demo with Godot's movie writer, 60 frames a second), over five seconds: the windmill,
the beach's sea and the city's street change exactly 60, 30, 15 and 10 times a second for Today, A, B and C. The fighters change every
frame in all four. On the Stage Look Review ("twos72", `tw72_s<stage>.mp4`), each stage's four options play side by side from the same
frame.

Recommended B: at 15 a second the windmill, the water and the traffic read as a set animated by hand, while the fighters stay smooth over
it.
- A is what the plan said (half the frame rate), but at 30 a second it barely reads as a style.
- The beach's sea is the largest thing that moves. If any of this reads as the game stuttering rather than as a style, it will be the sea.
- C looks most like stop-motion, but at 10 a second the water and the traffic start to look slow rather than crafted.
- The motes are a GPU particle system: on the scenery's clock they hold still and then catch up (`speed_scale`), so they move on the same
  frames as everything else.
All seven tests pass.

The look bench (`tests/look_bench.gd`) failed on this mock at first: the pale fighter's edge was 0.44 against the ocean's bright haze.
The bench already overflowed Godot's shared shader buffer (479 "Too many instances" errors before the mock; it passed only because the
instances that lost their slots happened not to matter). The clock's two global values moved the loss onto that fighter's outline. The
buffer is now doubled (project.godot `limits/global_shader_variables/buffer_size=131072`): no overflow, and the bench passes. The real
stages never overflowed.

**Built B** (the owner's pick, 2026-11-05):
- Every stage palette's [backdrop] `scene_steps=15.0` (meadow, sunset, ocean, city, grove). `--scene-steps=T` brings back every frame.
- Recorded as built, the meadow's windmill and the beach's sea change 15 times a second, and the fighters every frame.
- Today against built, the windmill every second frame (each picture held for four frames), and the fighters: `style/twos_built.jpg`.
All seven tests pass, and the look bench passes with no overflow.

## Step 8: each stage's paper on its cut edges (mock, 2026-11-05)

The owner: "Cut-edge colours". Cut card shows its stock at the edge: a sheet printed green on white card has a white edge. Today every
cut edge in the game is a pale version of its own face (the lawn's edge pale green, the boardwalk's pale wood). Here each stage has its
own paper (StageArt.PAPER_CORES), and its cut edges show it:
- the meadow, white card, with its stitch in linen thread (the cream thread vanished on white);
- the canyon, kraft;
- the beach, bleached origami paper;
- the city, newsprint (the plan's blue-grey board turned the pink roof's edge purple);
- the grove, washi, faintly violet.

The cut edges are the modelled play areas' card edges (`card`, `plank_card`, `top_card` and `steel_card` in `StageBake.place_play`) and
the ground's cut sheets (the card families in `_paint_ground`). The cut-edge light (toon_common's paper finish) takes the paper's colour
through the global `pf_paper_core`, on side faces only, so the lawns' tops stay as they are. The palette's [backdrop] `paper_core_look`
(and `paper_core` for another colour), or `--paper-core=T|A|B|C` (StageArt.PAPER_CORE_TRY):

| Option | What changes |
|---|---|
| Today | Each edge a pale version of its own face |
| A, a hint | The paper 40 per cent of the way on the cut edges |
| B, the paper | 70 per cent, and the cut-edge light on the sides in the paper's colour |
| C, all the way | The cut edges are the paper |

The main edge measured at the match camera (mean colour of a strip of it; value, hue and saturation):

| Stage, paper | Today | A | B | C |
|---|---|---|---|---|
| Meadow, white card | 0.80, hue 89, sat 0.26 | 0.81, hue 67, sat 0.19 | 0.83, hue 52, sat 0.13 | 0.85, hue 37, sat 0.10 |
| Canyon, kraft | 0.80, hue 42, sat 0.48 | 0.70, hue 36, sat 0.55 | 0.60, hue 29, sat 0.60 | 0.48, hue 22, sat 0.65 |
| Beach, bleached paper | 0.62, hue 29, sat 0.47 | 0.76, hue 34, sat 0.33 | 0.82, hue 36, sat 0.22 | 0.86, hue 35, sat 0.15 |
| City, newsprint | 0.49, hue 346, sat 0.45 | 0.52, hue 341, sat 0.40 | 0.54, hue 335, sat 0.35 | 0.57, hue 324, sat 0.31 |
| Grove, washi | 0.69, hue 89, sat 0.34 | 0.71, hue 74, sat 0.16 | 0.74, hue 19, sat 0.06 | 0.74, hue 324, sat 0.10 |

Every stage's walkable top measures the same in every option (value within 0.02), and so do the walls. The exception is the beach's
deck: its planks' sides turn pale, and its contrast goes from 0.07 to 0.10. On the Stage Look Review ("papercore73"), a close-up sheet
of each stage's edge (`pc73_close.jpg`) and each stage's four options at the match and wide cameras (`pc73_s<stage>_<m|w>.jpg`).

Recommended B: each stage's edges read as its own stock (the meadow's white card, the beach's bleached paper, the grove's washi), and
they still keep a trace of the face they were cut from.
- The beach changes most. The boardwalk's front edge turns from wood to a pale band, and the gaps between the deck's planks become thin
  pale lines, under the fighters.
- Under the city's violet dusk, the newsprint reads mauve, not grey, because pale paper takes the light's colour (the edge's hue goes
  from 346 to 335 in B and 324 in C). The blue-grey board was more purple still.
- Kraft is the one paper darker than today's edge, so the canyon's lip darkens instead of lighting up. Of the five, it changes least.
- B's edge light on the sides barely shows, because the rims were already pale. It adds only a thin line along the city roof's lip.
All seven tests pass, and the look bench passes.
