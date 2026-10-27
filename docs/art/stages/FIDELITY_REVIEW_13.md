# Stage fidelity review 13 (2026-10-27): a path to 5 in every category, in a still

The owner: "I would like to do another analysis of all of the stages, and try to figure out how we can get to a 5/5 in all categories in
a still shot." Same eight categories as reviews 10 to 12 (5 = the still could sit beside the references), scored on the release
playtest-2026-10-27-sim146-review12. Unlike the earlier reviews, this one also measures each still's value structure (what share of the
frame is dark, how wide its value range is), because a still is judged as one picture, and that is where the frames are weakest.

**Evidence:** art/stage-refine dafcfb57 at the match camera and wide (`review13/fidelity13_sheet.jpg`, each row a stage, match left, wide
right: meadow, grove, beach, canyon, city); close crops (`review13/review13_crops.jpg`: the canyon's hoodoos, the canyon's sky, the grove's
mist pads, the grove's island underside, the meadow's wall, the beach's near water, the city's skyline, the meadow's sky).

## Scorecard (unchanged from review 12)

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** |
| Canyon | 4 | 4 | 4 | 5 | 4 | 4 | 4 | 4 | **4.1** |
| City | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** |
| Grove | 4 | 4 | 4 | 5 | 4 | 5 | 4 | 4 | **4.25** |
| Meadow | 4 | 4 | 4 | 5 | 4 | 5 | 4 | 4 | **4.25** |

Twenty-nine of forty cells are 4. The fixes so far added things and refined single surfaces; what the 4s now share is the picture as a
whole: its values, its light and its edges.

## What the measurements say

Each still's luma at the 5th and 95th percentiles, its range, and the share of the frame darker than 0.25:

| Stage (wide) | p5 | p95 | Range | Darks | Top third's range | Note |
|---|---|---|---|---|---|---|
| Meadow | 0.34 | 0.84 | 0.50 | 2% | 0.38 | high-key, no dark anywhere |
| Beach | 0.36 | 0.78 | 0.42 | 2% | 0.28 | the narrowest: a pastel haze over the whole frame |
| Canyon | 0.26 | 0.75 | 0.49 | 4% | 0.28 | a sunset with no shadows |
| Grove | 0.16 | 0.79 | 0.62 | 16% | 0.49 | full range |
| City | 0.05 | 0.70 | 0.65 | 27% | 0.54 | full range, one hue |

The references hold a deliberate dark anchor (roughly 8 to 15% of the frame under 0.25) and a range near 0.7. **The meadow, the beach
and the canyon have almost no darks**, so they read milky in a still however much detail is in them. The grove and city have the range
but lose in other ways (below). P1's value gap to the ground: meadow 0.016, canyon 0.041, grove 0.060, city 0.131, beach 0.183; the
meadow and canyon are the narrowest, and they're also the two high-key frames.

## What blocks 5, by category

| Category | What a 5 needs | What blocks it now |
|---|---|---|
| **Light** | One readable light: a lit side, a shade side, cast shadows that say where the sun is. | Shading is soft ambient everywhere. The canyon is a low sunset with no long shadows on its floor; the beach's dock casts nothing on the water. |
| **Surfaces** | Hand-made material that holds up close, with edges chosen, not uniform. | A pale cut-edge outline round nearly every shape (the hoodoos, rocks, roots, posts) reads as sticker outlines. The meadow wall is near-identical grey boxes. |
| **Play area and front** | The fighters' ground and the face under it as the finest work in the frame. | Fronts fill a third of the wide frame at one value: the canyon's pale pink block, the meadow's grey wall. |
| **Background land** | Clear depth planes, each a step of value. | The beach's headland and sea and the city's skyline layers sit close in value. |
| **Sky** | A composed sky: a few large layered cloud groups, a gradient with real range. | Small identical cloud cards in rows (the canyon and meadow most); the top third's range is 0.28 to 0.38 on three stages. The city's moon is cut by the frame's top. |
| **World and life** | Already 5 on four stages. | The canyon's life is small and far. |
| **Cohesion** | Every piece in one hand. | The grove's mist pads are flat purple discs and its roots pink ribbons, unlike the rest of the paper; the city is all one purple. |
| **Readability** | The fighters the first read, in value as well as hue. | The meadow's (0.016) and canyon's (0.041) value gaps: yellow fighters on a pale top. |

The fighters are half of every still, and their finish is the character and animation sessions' work. A marketing still would also hide
the P1/P2 tags.

## The plan to 5 (all refine what exists; none adds new things)

Ordered by how many cells each lifts. Each is mocked, measured (value range, darks, P1's gap) and picked by the owner as before.

| # | Item | Stages | Lifts | Effort |
|---|---|---|---|---|
| 1 | **A dark anchor and full value range:** deepen the shade side of forms and the frame's foot (fronts, near water, near land) and push the farthest planes lighter, so each still spans 0.1 to 0.85 with 8 to 12% darks; the play top a step deeper under the fighters on the meadow and canyon. Per-stage palette and baked shade, no new parts. | meadow, beach, canyon | light, cohesion, readability, background land | 1 session a stage |
| 2 | **A readable sun:** long cast shadows from the hoodoos, buttes and arch across the canyon's floor toward the camera, a warm rim on the lit sides; the dock's shadow on the beach's water; on the meadow, cloud shadows already move, so the hill's shade side only. | canyon, beach, meadow | light | 1 to 2 sessions |
| 3 | **Chosen cut edges:** the pale outline only where a paper edge would catch light (top and lit edges, silhouettes against a darker plane), tinted to its own surface, faded with depth; gone on the shade sides. A post-pass change, game-wide, so it is one system gain. | all | surfaces, cohesion | 1 session |
| 4 | **Composed skies:** replace the rows of small identical cloud cards with two or three large layered groups of varied size per stage, and widen the sky's gradient; the city's moon whole in frame. | meadow, canyon, beach, city | sky | 1 session a stage |
| 5 | **The grove's odd pieces:** the mist pads as soft layered sheets with scalloped, fading edges; the roots in bark brown with a dark side; the side pines varied in size and tiers. | grove | cohesion, surfaces | 1 session |
| 6 | **The meadow's wall:** fieldstones of varied size and tone, rounder, a few dark gaps, a cap of turf: its stone a hand-laid wall, not a grid. | meadow | play area and front, surfaces | 0.5 session |
| 7 | **The city's value hierarchy:** the near skyline layer a deeper silhouette, the far ones lighter and bluer, warm window and sign light spread through all of them, so the purple has steps. | city | background land, cohesion | 0.5 session |
| 8 | **The beach's near water:** the flat facets as paper wave sheets with a dark trough under each lip, as the far sea has. | beach | surfaces, background land | 0.5 session |

**Honest ceiling:** items 1 to 4 decide most of it; with them, light, sky, cohesion and readability should reach 5 on the meadow, beach
and canyon. Surfaces at 5 everywhere also depends on item 3 landing well. Items 5 to 8 close the remaining stage-by-stage cells. The
fighters stay outside this plan and are the largest single gap left in any still.

**Recommended start:** item 1 on the canyon (the narrowest-range, lowest-darks, lowest-readability frame that should be the most
dramatic), then the beach and meadow.

## Item 1 on the canyon, a dark anchor (mock, 2026-10-27)

`post.gdshader` `value_deep` [darks, foot, foot from, reach], on the stage and scenery only (ids from 0.8, nearer than the sky), faded out
by `reach` units behind the stage: the scenery's darks pushed down by `darks` x (1 - smoothstep(0.12, 0.72, luma)), its lights kept,
and the frame's foot deepened by `foot` from `foot from` of the way down. Set from a style's finish "value_deep" in `lighting.gd`, or
`--value=A|B|C`: A [0.35, 0, 0.6, 60], B [0.35, 0.3, 0.6, 60], C [0.5, 0.42, 0.55, 80]. Measured (darks under 0.25, wide / match;
range wide; P1's gap): today 4% / 4%, 0.49, 0.041; A 9% / 12%, 0.54, 0.061; B 11% / 19%, 0.56, 0.062; C 17% / 27%, 0.60, 0.076. On the
Stage Look Review ("valuedeep33", `vd33_*`); recommended B (inside the references' 8 to 15% wide). To ship: lighting.gd STYLES
"sunset" finish `"value_deep": [0.35, 0.3, 0.6, 60.0]`.

Built B (the owner's pick): lighting.gd STYLES "sunset" finish `"value_deep": [0.35, 0.3, 0.6, 60.0]`. In the game the wide still holds
11% darks, range 0.56. Before and after (wide): `review13/canyon_value_built.jpg`. All seven tests pass.
