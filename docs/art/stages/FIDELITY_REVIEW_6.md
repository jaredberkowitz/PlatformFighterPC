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
