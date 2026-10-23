# Stage fidelity review 6 (2026-10-22, after review 5's four items): the path to 5

The owner: "we are getting close to 4/5 in terms of our graphics, if we were to take a still image of each stage. What do you think we
would need to get our game up to 5/5 in every category?" The same method as review 5: every stage scored 1 to 5 in the same eight
categories (5 = hard to tell apart in quality from the references, the owner's bar), measured in review 3's regions, against the same
references (docs/art/benchmarks/, LOOK.md). This review adds, per category, what a 5 needs.

**Evidence:** every stage as it ships (branch art/stage-refine d5779de, the build playtest-2026-10-22-sim146-review5) at the match
camera and wide (`review6/fidelity6_sheet.jpg`: meadow, grove, beach, canyon, city, each match then wide), and close crops of the
weakest things (`review6/review6_crops.jpg`, left to right, top to bottom: the meadow's front, the meadow's stage hill, the grove's
forest, the grove's underside, the beach's sea, the canyon's strata, the city's sky, the canyon behind the fight).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 5 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 3.8 |
| Meadow | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | **3.9** | 3.9 |
| Grove | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | **3.9** | 3.9 |
| Canyon | 4 | 3 | 4 | 3 | 4 | 4 | 3 | 4 | **3.6** | 3.5 |
| City | 4 | 4 | 4 | 4 | 2 | 4 | 3 | 4 | **3.6** | 3.4 |
| **Mean** | 4.0 | 3.8 | 4.0 | 3.8 | 3.2 | 4.0 | 3.6 | 4.0 | **3.8** | 3.7 |

- **The beach is the first stage at 4 in every column.** The paper sea took its land from 3 to 4 (the sea measures 0.60 at saturation
  0.22, down from 0.36: no longer one bright cyan sheet), and the horizon light and streaks under its cloud bank took the sky to 4.
- **The city's play area and readability rose to 4:** the dark scaffolds sit with the brownstone, and the towers behind the fight
  dropped from 0.159 to 0.132 in contrast. Its sky is still 2, the lowest score left in the game.
- **The canyon's readability rose to 4** (a lighter front, 0.33 from 0.30, and the trestle behind the far rocks), but its land stays 3:
  the butte right behind the fighters is still a stack of round discs, and the plain at the wide camera is empty.
- **The meadow's and grove's skies stay 3:** sky B shows, but at both cameras the mountains and the forest cover most of the sky,
  and the meadow's card clouds are cut by the top of the frame.

## What 5 needs, category by category

A 4 here means "clearly the same kind of game as the references, made with care". A 5 means a still could sit beside them. Across
the five stages, the gap is the same few things:

| Category | Now | What stands between it and 5 |
|---|---|---|
| **Sky** | 3.2 | The city's flat magenta (one value, 0.37, saturation 0.58, with dots for stars). The meadow's flat card clouds cut by the frame's top edge. The grove's sky is a flat lilac between the trees. A 5 has a sky that's painted: a gradient to the horizon, a glow behind the skyline or the land, a band of haze, and clouds that sit wholly in the frame. |
| **Surfaces** | 3.8 | The closest, largest surfaces in every still are the weakest made. The meadow's front is a grid of near-identical grey bricks (saturation 0.11). The canyon's strata are soft and smudged at the wide camera (the bake is too coarse for that size). The grove's cliffs are flat dark-teal blocks. The play tops are flat greens (meadow contrast 0.069). A 5 has authored detail at the size you see it: chipped and varied stone with moss, crisp strata with cut edges, worn grass with tufts and paths that look walked on. |
| **Cohesion** | 3.6 | One thing per stage still looks made another way. The city's towers are flat cards in one purple family against a modelled front, and the canyon's smudged strata sit beside crisp paper props. A 5 has every layer at the same level of making, with a colour plan that separates the layers (the city is all purple and magenta). |
| **Background land** | 3.8 | The canyon: the round-disc butte behind the fight, the empty plain, the boxy arch. The meadow: the stage hill is a regular stepped cake at the wide camera. The grove: the flat purple void under the island. A 5 has land that's shaped by hand with no machine-made silhouettes. |
| **World and life** | 4.0 | Repeats. The grove's forest is one pine at one size, cloned. The beach's wave arcs are one scallop at an even spacing. The city's street people are tiny code-built figures. A 5 has no clone you can spot in a still. |
| **Light** | 4.0 | The background is lit as evenly as the front, with no colour bounce and no per-stage grade. Bright things don't glow (the lanterns, the city signs, the canyon sun only a little). A 5 has a clear key light with shaped shadows, warm or cool bounce into the shade, a little bloom on lights, and a final grade per stage. |
| **Play area and front** | 4.0 | The floating platforms are plain slabs on every stage (a green slab with a scalloped trim on the meadow and grove, a bare board on the canyon). A 5 has platforms made in the stage's own materials. |
| **Readability** | 4.0 | Good now on every stage. A 5 keeps the fighters clear at every spot of every stage (the beach's pale wave arcs and the city's lit windows are the risks), and depends on the fighters too. |

**Outside the stages:** a still is half fighters. The character work (art/character-beta: the face, the hands, the body) counts in
every screenshot; the stages can't reach 5 in a still on their own.

**How far 5 is:** the beach shows that every column at 4 is in reach. Moving a column from 4 to 5 is harder than 3 to 4, because a 5
needs authored detail, not just fixed shapes. Sky, play area, world, cohesion and readability can reach 5 with the tools we have (Blender
models, the bake, shaders). Surfaces and light at a true 5 need hand-made texture detail on every large surface and a lighting finish
per stage. That's the biggest investment. A realistic target for the next rounds is a mean of about 4.5, with 5s in the columns above.

## Per stage: what stands between it and 5

- **Beach (4.0):** the sea's wave arcs repeat one scallop at an even spacing (vary their size, spacing and length, foam where they meet
  the pilings and the shore); one cloud bank (two or three at different depths); the pier's deck boards are even (wear and a few
  replaced boards). Closest to 5.
- **Meadow (3.9):** the front's stone wall (fieldstone of varied size and colour, moss and soil at the top, the planks above it
  weathered); the stage hill (fewer, bigger, uneven sheets); the card clouds whole in the frame and layered; the lawn top (tufts, a worn
  path that reads as walked on); the platforms in wood and turf.
- **Grove (3.9):** the forest (three or four pine shapes and sizes, a few broadleaf or blossom trees for colour); the void under the
  island (a misty valley floor with depth, the reference's canyon); the cliffs (shaped rock with moss and roots, not blocks); the lantern
  strings modelled; the lawn top.
- **Canyon (3.6):** the butte behind the fight (rebuilt in the buttes' cardboard blocks, like the arch); the strata baked crisp at a
  finer texel size with cut edges; the plain (ripples, scattered rocks, a dry wash); the arch's silhouette chipped; the top platform in
  rock.
- **City (3.6):** the sky (a night gradient, a glow over the skyline, haze between the rows of towers so each row reads further back);
  the colour plan (a cooler sky and warm street light to separate the layers from the purple); the far towers with more shape than
  flat cards; the street people modelled.

## The order (gain against effort)

Each stage refined on its own, as before; two items are game-wide because they're one system.

1. **The city's sky and depth** (sky 2 to 4, cohesion 3 to 4): the largest single gap left. A sky shader look for night, haze
   between the tower rows, and the colour plan.
2. **The canyon behind the fight** (land 3 to 4): the disc butte rebuilt as rock, the plain given detail, the arch chipped.
3. **The fronts' surfaces** (surfaces 3.8 to 4.5): the meadow's stone wall, the canyon's strata, the grove's cliffs and underside.
   Game-wide part: a finer bake on the fronts (the canyon's smudge is the bake's texel size).
4. **No clones** (world to 4.5): the grove's forest in several shapes, the beach's wave arcs varied, the meadow's stage hill.
5. **The light finish** (light to 4.5, game-wide system): a grade per stage, bloom on lights, warm and cool bounce into the shade.
6. **The platforms** in each stage's own materials.
7. **The small code-built things:** the lantern strings, the flowers, the islets' detail, the street people.

## Item 1, the city's sky and depth (2026-10-22, mock)

Why the sky read flat: the cameras see the sky from the horizon to about 0.09 up, and the city's glow (centred at 0.03, reaching 0.3)
filled all of it; its `sun_side` mix toward the bright glow, small as it was, swamped a dark sky (in linear colour a tenth of a bright
colour outweighs a dark one). New: `sky.gdshader` `glow_at`, `glow_height` and `glow_below` (a glow rising from behind a skyline), the
theme's `sky_bands` [middle, top] and `glow_height` [centre, reach, full below]; `facade.gdshader` haze between the rows (`foot_haze`,
`foot_height`, `foot_color` from the street, `street_y`), the palette's `skyline_haze` [strength, height] a layer and
`skyline_haze_color`. And a general trial switch: `--pal-try=<name>` lays a palette's own [try_<name>] section ("section/key" keys) over
it, so a mock is a few palette lines and shipping one is moving them into place.

Options (city.cfg [try_night_a|b|c]): A night sky (the frame's top 0.07, deep indigo-blue, from 0.34 magenta; the glow low behind the
skyline); B A with haze between the rows; C B with a colour plan (a navy sky and blue-violet towers, warm kept for the street haze, the
windows, the glow and the play building). On the Stage Look Review ("citynight", `cn_*`); recommended C.

**Built (2026-10-22): city C** (the owner's pick): the night sky, the haze between the rows and the colour plan moved into city.cfg's
[theme], [backdrop] and [air]; the sky before it stays as [try_pre_night] (`--pal-try=pre_night` compares). No re-light: the sky's
colours don't reach the baked or lit scenery (the play building's roof measured the same, 0.61, in every option). Before and after:
`review6/city_night_built.jpg`. Next, item 2: the canyon behind the fight.

## Item 2, the canyon behind the fight (2026-10-22, mock)

The round stack behind the fighters was `butte_blocks`: every tier cut into bands, each band a small bevelled block shrunk a little, on a
six- or seven-sided ring, so each butte read as stacked discs. New in `make_canyon_paper.py` (on `--variant KC`, off unless asked):
`--arch chipped` (the arch's span broken off at one end at a slant, the piece at the left leg's foot, a notch stepped into its top);
`--buttes solid` (`butte_rock`: two or three massive tiers of four or five sides, set off each other and stepping in, a pale band between,
a dark cap, a lump of cap on the tall ones, fallen blocks); `--fight wide` (the butte behind the fight a broad flat-topped mesa, wider
than tall, a flat face to the camera; lower, so the train shows above it at the top platform's height); `--plain outcrops` (`outcrops`:
low ledges and boulders across the plain within the wide camera's view). Options, all `--arch chipped`: A `--buttes solid`; B with
`--fight wide`; C with `--plain outcrops`. Mocked unlit on `--ground-mock=` against today's ground built the same way. On the Stage Look
Review ("canyonbehind", `cb_*`); recommended C.

**Built (2026-10-22): canyon C** (the owner's pick): `canyon_paper.glb` rebuilt `--variant KC --arch chipped --buttes solid --fight wide
--plain outcrops` (21,100 triangles), the canyon re-lit (`--light-export`, `light_stage.py` on the front, play area and ground, curve and
reach 2.5). Before and after: `review6/canyon_behind_built.jpg`. Next, item 3: the fronts' surfaces.

## Item 3, the fronts' surfaces (2026-10-22, mock)

First a pipeline fix: `StageBake.export_scene` (`--bake-export`, the scene the front builders bake their shade against) exported the
shipped modelled front among its occluders, so any new front baked inside the old one and its faces came out black (the meadow's first
fieldstone, the canyon's first cut edges). The modelled front is marked (`modelled_front`) and left out; by its mark, not its name, as
beside a first build still being freed it is `@Node3D@2`. Today's fronts were rebuilt the same way for the comparison, all before the
light bake.

- **Meadow** (`make_meadow_front.py --variant B`): `--stones field` (fieldstone: courses of uneven height, big stones with small ones
  packed round them, each a lump of its own tone and tint, set in or out; its baked shade floored at a quarter, packed close the corners
  in its joints baked to nothing) and `--moss 1` (moss on the tops, thickest under the logs, tongues down a few faces, grass at the foot).
  Lessons: a stone's corners pulled in, never out (out, it ran into its neighbours); one depth for the whole stone; a clamped bevel.
- **Canyon** (`make_canyon_front.py --variant A`): `--cut 1` (each slab's top front edge chamfered, a finer mesh, 15,600 triangles) and
  `--courses 1` (each slab in thin sediment courses, each a little in or out and a step lighter or darker, 38,500 triangles).
- **Grove** (`make_grove_paper.py --variant H`): `--rock shaped` (each block a tone of its own, a deeper bevel in a lit rim `cliff_lit`,
  roots from the lips facing the camera) and `--abyss mist` (two layers of violet mist with cut edges in the abyss, `abyss_mist`, the tops
  of a forest rising through them in the valley's own hazed violet-greens, `valley_*`: in the forest's greens they took no air and stood
  out lime). 133,000 and 155,000 triangles, from 128,000.

On the Stage Look Review ("fronts", `fr_*`); recommended B on all three, each stage shipped on its own.

**Built (2026-10-22): B on all three fronts** (the owner's pick): `meadow_front.glb` `--variant B --stones field --moss 1` (13,700
triangles), `canyon_front.glb` `--variant A --cut 1 --courses 1` (38,600), `grove_paper.glb` `--variant H --rock shaped --abyss mist`
(155,000); the meadow, canyon and grove re-lit. Lit, the canyon's courses measured 0.30 (their extra cut edges and crevices), undoing
review 5's lift; `sunset.cfg` `front_lift` 0.2 to 0.35 puts it back at 0.33 (and the courses alternate round the slab's tone, not below
it). Before and after: `review6/fronts_built.jpg` (meadow, canyon and grove wide, then the meadow at the match camera). Next, item 4.

## Item 4, no clones (2026-10-22, mock)

- **Grove, the forest** (the owner's rule stands: one large consistent kind of tree): `StageBake.place_trees` takes a list of models,
  each slot picking one by its place, and with the palette's `tree_vary` (`--tree-vary=1`) turns each tree and sizes it within a sixth,
  by its place; the great tree (size 1) stays as it is, the front's roots run into it. `make_grove_trees.py --variant C --shape tall|broad`
  builds two more shapes of the same pleated tree (2,900 and 2,800 triangles). `--tree-mock=` takes several, comma-separated. A turned
  and sized; B three shapes as well.
- **Beach, the paper waves**: `water.gdshader` `wave_vary` (the palette's `sea_vary`, `--sea-vary=`): 1 each tier's arcs their own
  width and depth, wandering along the row; 2 with calm stretches where a tier's edge lies flat (no lip, and no shadow on the sheet
  behind, which is the nearer tier's).
- **Meadow, the stage hill**: `make_meadow_ground.py --hill big` (a third as many sheets, each of its own height, a broader wobble) and
  `--hill banks` (those as terraces, each shelf falling to the next on a grass bank in the farmland terraces' `bank`).

On the Stage Look Review ("noclones", `nc_*`); recommended B on all three.

**Built (2026-10-22): B on all three** (the owner's pick): the grove's trees in three shapes (`StageBake.TREES` a list; `grove_tree_tall.glb`,
`grove_tree_broad.glb`) and turned and sized (`grove.cfg` `tree_vary=1.0`); the beach's waves uneven and broken (`ocean.cfg`
`sea_vary=2`); the meadow's hill terraced (`meadow_ground.glb` `--hill banks`); the meadow and grove re-lit. Before and after:
`review6/no_clones_built.jpg` (grove, beach and meadow wide). The runner test's shop-hint check flaked once and passed twice on its own.

## Item 5, the light finish (2026-10-22, mock)

Game-wide, the one system item. `toon_light.gdshaderinc` gains bounce light (`pf_bounce`, a global: the colour of the ground the light
comes back from, its alpha how much): the side turned from the sun picks it up, most on faces turned down, half as much on a fighter.
`Lighting.stage_mood` reads the style's `bounce`, `grade_more` (on the mood's grade), `curve_more` (on the S-curve) and `glow_from` (the
HDR level the bloom starts at, 2.3), or `--light-finish=A|B|C` for `LIGHT_TRY`'s: A the bounce (meadow and grove a fresh green, the
canyon warm sand, the beach the sea's cyan, the city the street's pink); B with the grade 0.15 and the S-curve 0.06 stronger; C with the
bloom from 1.9, so the lanterns, string lights and windows glow. Lessons: a bounce at a third of these strengths lifted the shade by a
hundredth (the light's own scale takes most of it); a few thin pieces have no real normal, and an unguarded bounce turned their NaN into
white blots across every stage (`isnan` guard). The changes are small by design (the meadow's platform undersides 0.59 to 0.65, greener;
the canyon's paper finish carries its shade, so it moves least). On the Stage Look Review ("lightfinish", `lf_*`); recommended C.

**Built (2026-10-22): light finish C** (the owner's pick): each stage's style in `lighting.gd` `STYLES` carries `bounce`, `grade_more`
0.15, `curve_more` 0.06 and `glow_from` 1.9; `--light-finish=off` shows a stage without it. No re-light (the finish is all at run time).
Before and after: `review6/light_finish_built.jpg` (every stage at the match camera). The runner test's shop-hint check flaked once and
passed twice on its own. Next, item 6: the platforms.

## Item 6, the platforms (2026-10-22, mock)

The beach's platforms are already dock boards with rope and the city's a steel deck, so this is the meadow, the grove and the canyon.
New flags on the shipped play variants (every walkable top stays at the sim's height): `make_meadow_play.py --variant B --plat
timber|timber_moss` (a turf mat on two round logs, cut ends showing, rope lashings, an uneven tufted lip; with moss and grass hanging
over the logs); `make_play.py --theme grove --variant B --plat islet|islet_moss` (turf over a band of earth, a rocky underside of the
island's dark earth, roots; deeper, with moss); `make_play.py --theme sunset --variant B --plat rock|rock_under` (a sand top over two
strata courses, the lower set in and broken short; with a tapered rock under it). Shared helpers in both builders: `solid`, `log_x`,
`underside`, `hanging`. Lesson: the grove's underside in the pale `stone` read as a paper skirt; in its dark earth, as rock. On the Stage
Look Review ("platforms", `fp_*`); recommended A on all three (B's undersides and moss come down toward a fighter standing beneath).

**Built (2026-10-22): A on all three platforms** (the owner's pick): `meadow_play.glb` `--variant B --plat timber`, `grove_play.glb`
`--theme grove --variant B --plat islet`, `canyon_play.glb` `--theme sunset --variant B --plat rock`; the meadow, canyon and grove re-lit.
The walkable tops are unchanged (stage_front_test passes). Before and after: `review6/platforms_built.jpg` (meadow, grove, canyon at the
match camera). Next, item 7: the small code-built things.

## Item 7, the small code-built things (2026-10-22, mock)

The most visible left: the string lights (bare spheres on the grove's lantern strings and the city's party lights), the grove's
streamers (yellow, pink and blue diamonds reading as dangling crystals) and the city's townsfolk (trapezoid paper dolls). The lawn
flowers barely show at play distance and stay. New: `make_props.py` `paper_lantern` (a ribbed glowing paper body, lacquered caps, a
tassel; 772 triangles) and `string_bulb` (a glass bulb on a socket; 244), both hanging from their origin (listed as such in
`prop_check_test`); `StageArt._string_lights(..., lamp)` hangs them with the palette's [backdrop] `string_lights="model"` or
`--strings=model`, the grove's warm, the city's in their three colours; `make_world.py --streamers shide` (white paper zigzags, as on
the gates) and `--people round` (`figure_round`: the fighters' build, a round body, mitten hands, shoes, two dot eyes, a bobble hat or a
scarf on some; families `shoe`, `eye`, `white`). Options: A the modelled lights; B with the shide and the round townsfolk. On the Stage
Look Review ("smallthings", `sm_*`); recommended B.
