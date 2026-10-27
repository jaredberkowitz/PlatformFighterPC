# Stage fidelity review 14 (2026-10-27): what is left to reach 5 in a still

The owner: "Score every stage again in all categories as review 14 and plan what's left to reach 5/5 in a still." Same eight
categories as reviews 10 to 13 (5 = the still could sit beside the references) and the same value measurements as review 13, scored on
the release playtest-2026-10-27-sim146-review13 (all of review 13: the dark anchor, the canyon's sun, chosen cut edges, composed skies,
the grove's mist and roots, the meadow's wall, the city's skyline steps, the beach's near waves, the sky range).

**Evidence:** art/stage-refine b1611d63 at the match camera and wide (`review14/fidelity14_sheet.jpg`, each row a stage, match left, wide
right: meadow, grove, beach, canyon, city). Close crops (`review14/review14_crops.jpg`), left to right, top to bottom:
- the meadow's and canyon's match-camera sky, with clouds cut by the frame's top;
- the fighters on the meadow's and canyon's tops;
- the canyon's front, wide; the beach's islands;
- the grove's falls and mist; the meadow's lower terraces;
- the city's facade; the beach's sky, wide.

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | **5** | 4 | 4 | 5 | 5 | 5 | **4.5** |
| Canyon | 4 | 4 | 4 | 5 | 4 | 4 | 4 | 4 | **4.13** |
| City | 4 | 4 | 4 | **5** | 5 | 5 | 4 | 5 | **4.5** |
| Grove | 4 | 4 | 4 | 5 | 4 | 5 | 5 | 4 | **4.38** |
| Meadow | 4 | 4 | **5** | 5 | 4 | 5 | 4 | 4 | **4.38** |

Mean **4.38** (review 13's scorecard after its items 1 to 5: 4.30; review 12: 4.18). Bold: up since then.
- The meadow's hand-laid wall makes its front the frame's dark anchor.
- The beach's near waves give the pier's water forms of its own.
- The city's three-step skyline separates near from far.

Fifteen cells are 5, twenty-five are 4. Every light and surfaces cell is still 4.

**The canyon is now the weakest stage (4.13).** Its sky and value range improved, but it's still one pink-orange hue, its front is a
plain banded block, and its sun shadows don't read on the floor at a glance.

## What the measurements say

| Stage (wide) | p5 | p95 | Range | Darks | Top third's range | P1's gap | Since review 13 |
|---|---|---|---|---|---|---|---|
| Meadow | 0.21 | 0.85 | 0.64 | 10% | 0.39 | 0.043 | range 0.50 to 0.64, darks 2 to 10% |
| Beach | 0.22 | 0.78 | 0.56 | 6% | 0.36 | 0.239 | range 0.42 to 0.56, darks 2 to 6% |
| Canyon | 0.18 | 0.76 | 0.58 | 12% | 0.38 | 0.080 | range 0.49 to 0.58, darks 4 to 12% |
| Grove | 0.16 | 0.78 | 0.62 | 18% | 0.49 | 0.073 | about the same |
| City | 0.05 | 0.70 | 0.65 | 32% | 0.63 | 0.154 | darks 27 to 32% |

**The value problem review 13 found is mostly solved.** All five frames now have a dark anchor (6 to 32%) and a range of 0.56 to 0.65.
The beach is still the lightest (6%); its darks are the pier's legs and their reflections, and that's right for a sunny harbour.

**What's left is in three places:**
- **Readability on the bright stages.** P1's value gap is still 0.043 on the meadow, 0.073 on the grove and 0.080 on the canyon. The two
  stages scored 5 for readability measure 0.154 (city) and 0.239 (beach). The fighters' yellow and orange separate by hue on the
  meadow's green, but not by value, and barely by hue on the canyon's sand.
- **Light.** It still reads as soft ambient on every stage: big forms have no clear lit side and shade side.
- **Specific surfaces still made of primitives or flat cards.**

## What blocks 5, by category

| Category | What a 5 needs | What blocks it now |
|---|---|---|
| **Light** (all 4) | One readable light: a lit side, a shade side, shadows that say where the sun is. | The shade sides of big forms sit barely darker than their lit sides: the meadow's mountains, the canyon's hoodoos and buttes, the beach's headland, the grove's pine masses, the city's towers. The canyon's sun shadows on its floor are there but faint. |
| **Surfaces** (all 4) | Hand-made material that holds up close. | The meadow's lower terraces have stair-stepped, pixel-like edges. The beach's islands are stacked green slabs. The grove's falls are flat striped cards. The canyon's hoodoos are stacked toy shapes. The city's facade is one flat pink brick. |
| **Play area and front** | The fighters' ground and the face under it as the finest work in the frame. | Canyon: the front fills a third of the wide frame as one banded pink block. City: the facade is a single pink, and the roof's top is plain. Grove: the top's lavender path is a flat stripe. |
| **Background land** | Clear depth planes, each a step of value. | Beach: the islands are flat stacked slabs, and the headland is one pale green. |
| **Sky** | A composed sky with real range, nothing cut by the frame. | The match camera cuts the composed clouds at the frame's top on the meadow and canyon (hard pale slabs in the corners), and on the grove behind its trees. The beach's painted streaks read as stripes across its upper sky. |
| **World and life** | Already 5 on four stages. | The canyon's life (its train, halt and water tower) is small and far. The rule is "far back, true size", so this may stay a 4. |
| **Cohesion** | Every piece in one hand. | Canyon: everything is one pink-orange, the shade no cooler than the light. City: everything is one purple-pink, the play building no warmer than the night. Meadow: the glassy pale-blue mountains sit apart from the saturated greens. |
| **Readability** | The fighters the first read, in value as well as hue. | Meadow (0.043), grove (0.073) and canyon (0.080): yellow and orange fighters on a pale top. |

## The plan to 5 (all refine what exists; none adds new things)

Ordered by how many cells each lifts for its cost. Each is mocked, measured (value range, darks, P1's gap) and picked by the owner, as
before.

| # | Item | Stages | Lifts | Effort |
|---|---|---|---|---|
| 1 | **Clouds clear of the frame:** at the match camera, no cloud group cut by the frame's top: lower each group into the frame, or lift it out. Soften the beach's painted streaks into fewer, broader bands. | meadow, canyon, grove, beach | sky (4 cells) | 0.5 session |
| 2 | **The fighters' ground a step deeper:** the play tops of the meadow, grove and canyon a step darker and a little less saturated in the band the fighters stand on; the lips and edges stay pale. Target P1's gap at least 0.12. | meadow, grove, canyon | readability (3 cells) | 0.5 to 1 session |
| 3 | **A key-to-fill ratio:** the shade sides of the big forms a clear step darker than their lit sides (the mountains, hoodoos, buttes, headland, pine masses, towers). The fill light is lowered and the key raised per mood, with the post pass's dark anchor holding. A lighting change, so one system gain for every stage. | all | light (5 cells) | 1 to 2 sessions |
| 4 | **Warm and cool:** the shade a cool violet-blue on the canyon, so its light is warm and its shade is not; on the city, the play building's brick a warmer red and its roof a lit warm top against the cool night. | canyon, city | cohesion (2 cells), canyon light | 1 session |
| 5 | **The canyon's front:** the banded block broken into rock strata of different depths, the top beds overhanging, a shade gradient down the face, a few deep joints. Rebuilt with make_canyon_front.py. | canyon | play area and front, surfaces | 1 session |
| 6 | **The beach's islands and headland:** the stacked green slabs as rounded paper islands with a sand rim and a shade side; the headland with painted patches, like the meadow's peaks. | beach | background land, surfaces | 1 session |
| 7 | **The meadow's terraces:** the stair-stepped edges of the lower terraces cut as smooth curved card; the mountains' glassy pale facets warmed toward the land's greens. | meadow | surfaces, cohesion | 1 session |
| 8 | **The grove's falls and path:** the falls as layered paper sheets with a foam foot, not striped cards; the path a worn earth, not a lavender stripe. | grove | surfaces, play area and front | 0.5 to 1 session |

**Honest ceiling:**
- Items 1 to 3 lift twelve cells. With them the mean reaches 4.68; the canyon is the lowest at 4.5.
- Items 4 to 8 close most of the remaining stage-by-stage cells.
- Cells that may stay at 4: the canyon's world and life (the "far back, true size" rule), and the city's and grove's play area and
  front.
- The fighters stay outside this plan, and they are still the largest gap in any still. A marketing still would also hide the P1/P2 tags.

**Recommended start:** item 1, the cheapest and four cells, then item 2, then item 3 (the biggest gain but the most risk to the look
everyone has approved, so it is mocked at three strengths).

## Item 1, clouds clear of the frame (mock, 2026-10-27)

Every stage's match camera stands at the same place: 26.5 out at 6.85 up, pitched 10 degrees down, a 30 degree view, with no roll. Its
frame's top is therefore a plane holding the x axis: 6.85 + (26.5 - z) * 0.0875 up at depth z, about 18.8 at the composed clouds (z
-110). The wide camera's (60 out, 12.03 up) is about 8 higher. Zooming out only raises it.

`--cloud-clear=A|B` (palette [backdrop] `cloud_clear`; `StageArt._clear_of_frame`, `MATCH_TOP`, `WIDE_TOP`; meadow, canyon, beach and
grove):
- Every cloud `_clouds` builds is measured by its meshes' bounds.
- One crossing the match top is lowered until it's whole (then it's whole at every zoom).
- One above the match frame that crosses the wide camera's top comes down under it (some of the canyon's far layer, cut in today's wide
  frame too).
- B also sets each cloud it moves to 0.8 of its size.
- No random numbers move.
- On the beach, the sky's stratus bands become two broader ones (`sky_stratus` [layers, thickness] [2, 1.8]; sky.gdshader `stratus`):
  its three thin bands read as stripes over the deeper top.

A first B lifted the crossing clouds clear above the match frame instead. It came out the same as A: the two tops are about 8 apart and
these clouds are taller, so nearly every one fell back into the frame.

The sky's top third (wide), today then A and B: meadow 0.39, 0.40, 0.37; canyon 0.38, 0.41, 0.39; beach 0.36, 0.47, 0.40; grove 0.49,
0.48, 0.48. P1's gap holds: meadow 0.043, 0.035, 0.040 (its noise); canyon 0.080; beach 0.239 to 0.244; grove 0.073 to 0.071.

On the Stage Look Review ("cloudclear43", `cc43_*`); recommended B. To ship: [backdrop] `cloud_clear="B"` in meadow.cfg, sunset.cfg,
ocean.cfg and grove.cfg, and ocean.cfg [theme] `sky_stratus=[2.0, 1.8]`.

Built B (the owner's pick): [backdrop] `cloud_clear="B"` in meadow.cfg, sunset.cfg, ocean.cfg and grove.cfg; ocean.cfg [theme]
`sky_stratus=[2.0, 1.8]`. The shipped frames match the mock (what differs is the clouds' drift and the grove's swaying trees). Before
and after (the match camera's top: meadow, canyon, beach, grove): `review14/clouds_clear_built.jpg`. All seven tests pass.

## Item 2, the fighters' ground a step deeper (mock, 2026-10-27)

P1's gap (|its body's mean value - the rest of the box|) is the body lighter than its surround on all three stages (meadow 0.603 vs
0.563, canyon 0.626 vs 0.546, grove 0.599 vs 0.525), so a deeper ground widens it. The top fills a little over half the measured box,
so reaching 0.12 takes the top itself about 0.1 to 0.14 darker: a real step.

`--ground=A|B` (palette [backdrop] `fight_ground` [darker, less saturated]; `StageBake.place_play`; meadow, grove, canyon): the walked
families only (lawn, top, path, top_deep, top_lit, top_worn), not the card edges or the cut fringe. A [0.2, 0.0]; B [0.32, 0.08]. A
first try with a fifth of the saturation taken too turned the lawns a grey olive, so the colour is now kept.

| P1's gap | Today | A | B |
|---|---|---|---|
| Meadow | 0.040 | 0.087 | 0.110 |
| Canyon | 0.080 | 0.137 | 0.168 |
| Grove | 0.075 | 0.113 | 0.135 |

The wide frames' value structure is unchanged (darks 10%, 12 to 13%, 18 to 19%). The meadow falls short of 0.12: the rest of its gap
is the bright hills behind P1. The platforms' tops change too (fighters stand there). On the Stage Look Review ("fightground44",
`fg44_*`); recommended B. To ship: [backdrop] `fight_ground=[0.32, 0.08]` in meadow.cfg, sunset.cfg and grove.cfg.

Built B (the owner's pick): [backdrop] `fight_ground=[0.32, 0.08]` in meadow.cfg, sunset.cfg and grove.cfg. P1's gap in the game:
meadow 0.113, canyon 0.165, grove 0.134. Before and after (the fight, match camera): `review14/fight_ground_built.jpg`. All seven
tests pass.

## Item 3, the key against the fill (mock, 2026-10-27)

**Why review 9's deeper painted shade barely moved the scenery's contrast:** on a face turned from the sun, most of the light is the
ambient and the fill. On the meadow that's about 0.23 ambient and 0.4 fill, against 0.21 of key through the painted shade. Lit faces
take about 2.0.

`--key-fill=A|B|C` (a style's `key_fill` [ambient kept, fill kept]; `Lighting.KEY_FILL_TRY` A [0.7, 0.7], B [0.45, 0.45], C [0.2, 0.2]).
It acts on the scenery only:
- The ambient's removed share is subtracted as emission in toon_common (`pf_amb_cut`, times the surface's AO), so the fighters keep the
  whole ambient.
- Every directional light that isn't the key (told apart from the key by its direction against `pf_sun_dir`) is scaled in toon_light
  (`pf_key_fill`).
- The key rises by what lit faces lost of the ambient (1 plus that share over the key's), so lit faces keep their value.
- Both globals are set on every stage (identity unless chosen), as globals outlive a stage.

Wide, the share of the frame under 0.25, today then A, B and C:

| Stage | Today | A | B | C |
|---|---|---|---|---|
| Meadow | 10% | 11% | 13% | 14% |
| Canyon | 13% | 14% | 15% | 16% |
| Beach | 6% | 7% | 8% | 8% |
| City | 32% | 33% | 34% | 36% |
| Grove | 18% | 20% | 22% | 24% |

- **Lit faces hold:** the frame's 95th percentile doesn't move on any stage. The median drops 0.02 to 0.05.
- **P1's gap widens** from today to C: meadow 0.115 to 0.151, canyon 0.168 to 0.199, beach 0.243 to 0.255, city 0.157 to 0.183,
  grove 0.134 to 0.159.
- **P1's own values are unchanged** (canyon and city to the thousandth).

What it reaches is the toon material: the play areas, fronts, rocks, pines, houses and props. The canyon's arch and buttes show it most.
It doesn't reach the meadow's mountain range (painted_ridge.gdshader), the city's skyline (facade.gdshader) or the beach's headland.
The far forms' shade is also lifted back by the haze.

On the Stage Look Review ("keyfill45", `kf45_*`); recommended C. To ship: STYLE_DEFAULT `key_fill` [0.2, 0.2], or a level per style.

Built C on every stage (the owner's pick): STYLE_DEFAULT `key_fill=[0.2, 0.2]`. The shipped frames measure as mock C (canyon wide: darks
16%, p95 0.74; city: darks 36%). Before and after (wide: the canyon's rocks, the grove's pines, the meadow's pines and barn, the beach's
town): `review14/key_fill_built.jpg`. All seven tests pass.

## Item 4, warm against cool (mock, 2026-10-27)

`--warm-cool=A|B`, on the canyon and the city.

**Canyon.** Every face was one pink-orange: the painted shade turned only 0.1 toward the dusk's violet (shadow_hue 0.79), and with the
fill now low the painted shade is most of a shaded face. In `Lighting.stage_mood`:
- A: `shadow_shift` 0.2.
- B: 0.25 (the most the shade allows) toward hue 0.72, and `shadow_value` x0.9.

**City.** The brick (city.cfg `soil` 0.6, 0.46, 0.42) was calmed three quarters of the way into the stage's purple haze (`FRONT_CALM`
0.75), so the play building went mauve.
- A: the brick a warm red (`StageArt.theme`: `soil` 0.66, 0.36, 0.26 and `soil_dark` 0.48, 0.24, 0.2), and the lit roof 35% toward
  a warm peach (`StageBake.place_play`, palette [backdrop] `roof_warm`).
- B: A with the city's front calmed to 0.35 (a style's `front_calm`).

Values hold: wide darks and the 95th percentile unchanged on both stages. P1's gap holds: canyon 0.200 to 0.208, city 0.185 to 0.181.
The shade hue is shared with the fighters, so on the canyon their shaded side turns slightly cooler too (P1's body 0.626 to 0.635).
The canyon's step is gentle even at B.

On the Stage Look Review ("warmcool46", `wc46_*`); recommended B on both. To ship:
- `MOODS.sunset` `shadow_shift` 0.25, `shadow_hue` 0.72 and `shadow_value` 0.522;
- city.cfg `soil` and `soil_dark` as above, and [backdrop] `roof_warm=0.35`;
- the city style's `front_calm` 0.35.

Built B on both (the owner's pick):
- `MOODS.sunset` `shadow_hue` 0.72, `shadow_shift` 0.25, `shadow_value` 0.522;
- city.cfg `soil` (0.66, 0.36, 0.26) and `soil_dark` (0.48, 0.24, 0.2), and [backdrop] `roof_warm=0.35`;
- the city style's `front_calm` 0.35.

The shipped frames match the mock. Before and after (the canyon wide, the city's play building): `review14/warm_cool_built.jpg`. All
seven tests pass.

## Item 5, the canyon's front (mock, 2026-10-27)

`make_canyon_front.py --strata A|B` on the shipped flags (`--variant A --cut 1 --courses 1 --straight C --mass A`):
- **The tiers alternate hard and soft beds** (each tier's own random numbers, seed 900 + tier). The soft ones are 0.7 as thick and set
  back 0.2 to 0.42; the hard ones are 1.1 as thick and stand out 0.1 to 0.5. The top bed overhangs 0.8.
- **Each column's own set** is widened 1.8 times.
- **The core behind the face** sits at FZ - 1.6 instead of FZ - 0.5.
- **B** opens every third joint into a cleft (0.38 each side) and darkens the face toward its foot by 0.42 instead of 0.26.

A first version set the soft beds back 0.55 to 0.75. They fell behind the core, which showed between the ledges as black slots: a chest
of drawers.

The mocks are unlit (today too). The front at the wide camera: contrast 0.084 today, 0.097 A, 0.102 B; value 0.27, 0.27, 0.26. P1's gap
holds (0.207). On the Stage Look Review ("canyonfront47", `cf47_*`); recommended B. To ship: rebuild
`godot/models/stages/canyon_front.glb` with the shipped flags plus `--strata B`, and re-light the canyon (front, paper, play).

Built B (the owner's pick): `godot/models/stages/canyon_front.glb` rebuilt with `--variant A --cut 1 --courses 1 --straight C --mass A
--strata B`. The canyon is re-lit (front, paper, play; `--curve 4.0 --reach 4.5 --floor 0.28` after a fresh `--light-export`). In the
game, wide: the front's contrast goes from 0.087 to 0.105 and its value from 0.27 to 0.25. P1's gap is 0.208. Before and after:
`review14/canyon_front_built.jpg`. All seven tests pass.
