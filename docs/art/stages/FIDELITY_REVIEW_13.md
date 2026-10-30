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

## Item 1 on the meadow and the beach (mock, 2026-10-27)

The same `--value=A|B|C` (darks under 0.25, wide / match; P1's gap). Meadow: today 2% / 2%, 0.016; A 3% / 4%, 0.009; B 4% / 6%, 0.008;
C 10% / 17%, 0.026 (A and B darken the shade at the fighters' level more than the lawn round them, so the gap narrows; only C reaches
the references' range). Beach: today 2% / 7%, 0.183; A 3% / 16%, 0.210; B 3% / 18%, 0.213; C 4% / 20%, 0.231. The beach's water is
drawn after the finishing pass, so its wide still barely moves; a deeper `sea_near` (`--sea-near=D`, [1.8, 1.0]) tried with it barely
changed it either. Its wide darks have to come from item 8. On the Stage Look Review ("valuedeep34", `vm34_*`); recommended C on both.
To ship: lighting.gd STYLES "meadow" and "ocean" finish `"value_deep": [0.5, 0.42, 0.55, 80.0]`.

Built C on both (the owner's pick): lighting.gd STYLES "meadow" and "ocean" finish `"value_deep": [0.5, 0.42, 0.55, 80.0]`. In the game
at the match camera: the meadow 17% darks, P1's gap 0.035 (0.016 before); the beach 20% darks, P1's gap 0.234. Before and after
(match camera, meadow then beach): `review13/meadow_beach_value_built.jpg`. All seven tests pass.

## Item 2, a readable sun: what blocks it (investigation, 2026-10-27)

The canyon's floor shows no shadows from its buttes, arch or hoodoos, though the stage has realtime background shadows (shipped
7fc9efb, 2026-10-22: 260 units, four splits, `pf_cast_shade`). Findings, with debug renders (the cast tone set to black; then every
pixel with the shadow attenuation under 0.5 painted magenta):

- The key's settings are as shipped: shadows on, range 260, four splits, pitch -20 (tried -30), effective yaw -122 (the light travels
  toward the camera and right, from the drawn sun's side, so shadows should stretch toward the camera across the floor).
- At pitch -20 much of the floor is in the cel shade band, and the cast tone only darkens lit faces; at -30 the floor takes the light.
- The shadow maps work near the camera: faces turned from the sun are shadowed, and a near hoodoo's shadow lands on the floor beside
  it. **Further back (the arch and buttes, about 110 to 180 from the wide camera, the third and fourth splits) no shadow reaches the
  floor at all**, and the black cast tone shows nowhere on the grove's terraces either.
- None of the arch, butte or hoodoo pieces are trimmed from casting (`StageView.trim_shadow_casters`, logged); the floor sheets are
  trimmed (they are 400 wide) but still receive; a lower shadow bias (normal 0.3, bias 0.03) changes nothing.

So the far splits' shadows have stopped landing since the 22 October fix (something added since likely stretches those splits again,
as the far ranges once did), or the cast tone path no longer fires. Next step: bisect from 7fc9efb with the black-cast debug, then fix;
only after that can the readable sun be mocked. A fallback that needs no fix: bake the sun's shadow into the ground's vertices
(`light_stage.py --sun`, the retired pilot), softer but certain.

### Correction, and the mock (2026-10-27)

The far shadows were not broken. The black-cast test was wrong (a shadowed pixel keeps its ambient light, so it never goes black). In a
bisect checkout on F: (`F:/PlatformFighterBuild/bisect`, removed after), the canyon captured with the sun's shadows on and off (a
`--noshadow` patch) at 7fc9efb and at dafcfb57: the shadows darken 29.5% and 30.5% of the frame (34.0% and 30.8% of the band behind
the stage). Mapped, nearly all of it is the faces turned from the sun: the key came from behind the scene (effective yaw -122, pitch
-20), so every face toward the camera sat in shade and the rocks' shadows fell behind them. Nothing to bisect.

The fix is the sun's direction. `lighting.gd`: a mood's "sun" [pitch, yaw] replaces its key's angles, or `--sun=A|B|C`: A [-28, -10]
(effective yaw -90, low from the left, square across), B [-28, 10] (from the left and a little toward the camera), C B with the cast
tone (0.2, 0.14, 0.42). Measured against today (with the shipped dark anchor; wide darks; P1's gap): today 11%, 0.065; A 11%, 0.070;
B 12%, 0.075; C as B. The rocks gain a lit side toward the sun and their shadows run right across the floor; a modest change at this
size. On the Stage Look Review ("readablesun35", `sn35_*`); recommended B. To ship: MOODS "sunset" `"sun": [-28.0, 10.0]`.

Built B (the owner's pick): MOODS "sunset" `"sun": [-28.0, 10.0]`. P1's gap 0.074 in the game. Before and after (wide, left and right):
`review13/canyon_sun_built.jpg`. All seven tests pass.

## Item 3, chosen cut edges (mock, 2026-10-27)

`post.gdshader` `cut_choose` [lit side only, own colour, ink lift kept]: the cut-edge rim now sums the directions the depth drops away in,
and is kept (x) only where that edge faces the light (`-sheet_dir` across the screen); (y) takes its own surface's colour, lightened
(col * 1.25 + 0.05), in place of `cut_colour`; (z) scales the pale lift on the scenery's ink lines (`paper_edge`). A style's finish
"cut_choose" or `--cut=A|B|C`: A [1, 0, 1], B [1, 1, 1], C [1, 1, 0.4]. P1's gap, today then C: meadow 0.035 to 0.041, canyon 0.074 to
0.082, beach 0.230 to 0.238, city 0.132 to 0.151, grove 0.060 to 0.072. On the Stage Look Review ("cutedges36", `ce36_*`; the ten
oldest sections archived from the published copy only, for the artifact's file limit); recommended C. To ship: lighting.gd
STYLE_DEFAULT finish `"cut_choose": [1.0, 1.0, 0.4]` (every stage).

Built C (the owner's pick): lighting.gd STYLE_DEFAULT finish `"cut_choose": [1.0, 1.0, 0.4]`, every stage. The city's P1 gap 0.150 in the
game. Before and after (the hoodoos, the island underside): `review13/cut_edges_built.jpg`. All seven tests pass.

## Item 4, composed skies (mock, 2026-10-27)

`StageArt._composed_sky`: the palette's [backdrop] `sky_groups` [[x, y, z, size, kind], ...] or `--sky=A|B` (`SKY_GROUPS`, meadow and
sunset), a few larger clouds placed by hand in two groups apart, the sky over the fight open, the canyon's to the right clear of the
sun. The stage's own cloud call still draws its random numbers into a node thrown away, so the rest of the layout keeps its place. B also
makes the far layer (`_far_clouds`) a third as many, half again as large. The city's moon down from 34 to 19 (`moon_y`), whole in the
wide frame. The first placement (18 to 36 up) sat above the wide camera's frame, which ends near 27 at those depths; the meadow's now
11 to 18, the canyon's 18 to 25. The sky's value range barely moves (its gradient is the sky shader's). P1's gap: meadow 0.043 to
0.048 (B), canyon 0.078 to 0.081, city 0.152. On the Stage Look Review ("composedsky37", `sk37_*`); recommended B. To ship: meadow.cfg
and sunset.cfg [backdrop] `sky_groups` from `SKY_GROUPS`, `cloud_far` a third as many and 1.5 times the size, city.cfg `moon_y=19.0`.

Built B (the owner's pick): meadow.cfg and sunset.cfg [backdrop] `sky_groups`, sunset.cfg `cloud_far=[3, 1.425, 0.3]` (the meadow has no
far layer, so its A and B are the same), city.cfg `moon_y=19.0`. Before and after (the skies, wide): `review13/skies_built.jpg`. All seven
tests pass.

## Item 5, the grove's mist banks and roots (mock, 2026-10-27)

`make_grove_paper.py --banks soft` (`bank`): each mist bank three stacked sheets (1.0, 0.74, 0.46 of its size, 0.45 apart), in
`bank_mist_low` (0.64, 0.6, 0.84), `bank_mist` and `bank_mist_top` (0.93, 0.91, 0.99), their scallops deeper (0.3); and the front's roots
from the palette's [backdrop] `root_colour` or `--roots=A|B` (`StageBake._root_colour`): A (0.38, 0.26, 0.17), B (0.3, 0.2, 0.13); the bark
brown (0.46, 0.37, 0.3) read pink under the grove's light. The side pines are left: they are the stage's one modelled tree kind (the
owner's rule). P1's gap (all unlit) 0.072 today, 0.073 A, 0.072 B. On the Stage Look Review ("groveroots38", `gr38_*`); recommended B.
To ship: rebuild `godot/models/stages/grove_paper.glb` with `--banks soft`, re-light the grove, and grove.cfg [backdrop]
`root_colour=Color(0.3, 0.2, 0.13, 1)`.

Built B (the owner's pick): `godot/models/stages/grove_paper.glb` rebuilt with `--banks soft`, the grove re-lit (front, paper, play),
grove.cfg [backdrop] `root_colour=Color(0.3, 0.2, 0.13, 1)`. P1's gap 0.074. Before and after (the abyss, wide):
`review13/grove_mist_roots_built.jpg`. All seven tests pass.

## Item 6, the meadow's wall (mock, 2026-10-27)

`make_meadow_front.py --laid A|B` (`LAID` in `field_wall`): the same courses laid by hand. Stones 0.8 to 1.9 wide (some 1.9 to 3.1), a
second stone stacked in a course 65% of the time, each shortened by up to 28% and dropped within its course, rounder (`rough_stone` pull
0.26, bevel 0.24), with a chinking stone in 45% of the gaps. B sets them back 1.8 times as far and spreads their value wider (0.85).
Same colours, moss and cap. The mocks are unlit (today too). P1's gap 0.043 today, 0.046 A, 0.044 B. On the Stage Look Review
("wall39", `wl39_*`); recommended B (A if B's busier wall pulls the eye). To ship: rebuild `meadow_front.glb` with the shipped flags
plus `--laid B` and re-light the meadow.

Built B (the owner's pick): `godot/models/stages/meadow_front.glb` rebuilt with `--variant B --stones field --moss 1 --warm B --calm B
--laid B`, the meadow re-lit (front, play, ground; `--curve 4.0 --reach 4.5 --floor 0.28` after a fresh `--light-export`). P1's gap
0.046. Before and after (the wall, match camera then wide): `review13/meadow_wall_built.jpg`.

## Item 7, the city's value steps (mock, 2026-10-27)

`--skyline=A|B` (palette [backdrop] `skyline_steps`, `StageArt` city backdrop): the three skyline layers stepped, near (0.19, 0.17, 0.38),
middle (0.36, 0.37, 0.64), far (0.6, 0.68, 0.9), where they sat close in one purple. B also lights more of the two nearer layers'
windows (`skyline_warm`, 1.8 times the share). Wide darks 28% today, 32% with A and B; P1's gap 0.151, 0.155, 0.157. On the Stage Look
Review ("skyline40", `sy40_*`); recommended B. To ship: city.cfg [backdrop] `skyline_steps` and `skyline_warm=1.8`.

Built B (the owner's pick): city.cfg [backdrop] `skyline_steps` and `skyline_warm=1.8`. (They replace `skyline_shades`, so the
`[try_pre_night]` trial's own shades no longer show; it is not shipped.) P1's gap 0.154. Before and after (the city's top, wide, then the
beach's near water, match camera): `review13/city_beach_built.jpg`.

## Item 8, the beach's near water (mock, 2026-10-27)

`--near-waves=A|B` (palette [backdrop] `sea_near_waves`, water.gdshader `near_waves`, 0.6 and 1.2): the paper sea's sheets kept to the
frame's foot instead of fading into the deepening. Each sheet's tone step up to 1 + 3.5 times, shaded across further (to 0.7 to 1.25),
the shadow it lays on the next stronger, its lit lip softened less there. The first B (tone step 1 + 2 times, lips still softened)
changed 13% of the lower frame wide but could not be seen, so it was strengthened. P1's gap 0.238 today, 0.240 A, 0.243 B. On the Stage
Look Review ("nearwaves41", `nw41_*`); recommended B. To ship: ocean.cfg [backdrop] `sea_near_waves=1.2`.

Built B (the owner's pick): ocean.cfg [backdrop] `sea_near_waves=1.2`. P1's gap 0.244. Before and after: the lower half of
`review13/city_beach_built.jpg`.

## Item 4's second half, a wider sky gradient (mock, 2026-10-27)

The cameras see the sky from about 0.03 to 0.09 up. The canyon's and beach's skies used the shader's own bands (the middle tint at
0.035, the top colour at 0.13), so the frame only ever showed their pale lower half; the meadow's were already fitted (0.012, 0.07).
`--sky-range=A|B` (`StageArt.sky`, meadow, canyon and beach only): A fits the bands to the frame (0.012, 0.07); B also takes the top
colour 22% deeper and the middle tint 8%. The sky's value range (the frame's top third, wide, p5 to p95): meadow 0.39 today, 0.39 A
(its bands were already fitted), 0.43 B; canyon 0.27, 0.33, 0.38; beach 0.28, 0.33, 0.36. P1's gap holds (meadow noise 0.033 to 0.044,
canyon 0.078 to 0.081, beach 0.241 to 0.243). On the Stage Look Review ("skyrange42", `sr42_*`); recommended B on the canyon and the
beach, A (as today) on the meadow, whose deeper top goes grey. To ship: sunset.cfg and ocean.cfg [theme] `sky_bands=[0.012, 0.07]`,
their `sky_top` and `sky_mid` deepened.

Built B on the canyon and the beach (the owner's pick; the meadow as it is): sunset.cfg and ocean.cfg [theme] `sky_bands=[0.012, 0.07]`
and `sky_deepen=[0.22, 0.08]` (`StageArt.sky`: the top and middle deepened in the sky only, so the light's ambient, which reads
`sky_mid`, is unchanged). The sky's range as mocked: canyon 0.38, beach 0.36. Before and after (wide): `review13/sky_range_built.jpg`.
