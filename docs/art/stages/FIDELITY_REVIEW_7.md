# Stage fidelity review 7 (2026-10-22, after all of review 6)

The owner: "How would the judge rank our stage art now out of 5? I would like to know what it would take to get us to all 4s or 5s."
The same method as reviews 5 and 6: every stage scored 1 to 5 in the same eight categories (5 = a still could sit beside the
references), measured in review 3's regions, against the same references.

**Evidence:** the build released as playtest-2026-10-22-sim146-review6 (art/stage-refine aaf4c6b) at the match camera and wide
(`review7/fidelity7_sheet.jpg`: meadow, grove, beach, canyon, city), and the weak spots close (`review7/review7_crops.jpg`, left to
right, top to bottom: the meadow's sky, the grove's sky, the city's sky, the canyon behind the fight, the meadow's lawn, the beach's deck).

A caution: the judge is the same hand that built the work, so read a 4 as "the gap the review sees is closed", not as an outside verdict.

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 6 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Canyon | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 3.6 |
| City | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 3.6 |
| Meadow | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | **3.9** | 3.9 |
| Grove | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | **3.9** | 3.9 |
| **Mean** | 4.0 | 4.0 | 4.0 | 4.0 | 3.6 | 4.0 | 4.0 | 4.0 | **3.95** | 3.8 |

- **38 of 40 scores are 4; the two 3s are the meadow's and the grove's skies.** No score is 5.
- **The canyon rose to 4 everywhere:** solid buttes and outcrops (land), crisp sediment courses (surfaces), one rock language from the
  front to the far buttes (cohesion). Its front measures 0.36 (0.33 in review 6), its behind-the-fight 0.58.
- **The city's sky 2 to 4:** a night gradient (the frame's top 0.17, from 0.37 magenta) with the glow behind the skyline; the moon and
  searchlights at the wide camera. Cohesion 4: the cool towers (hue 256) part from the warm roof (347).
- **The meadow's sky stays 3:** the card clouds are cut by the frame's top edge at both cameras and the sky is one pale teal (0.72,
  saturation 0.30) behind the mountain. **The grove's stays 3:** between the trees the sky is a washed lilac-grey (saturation 0.07)
  with no gradient or glow, and it shows in every gap of the forest.

## To all 4s

Only the two skies. One round, settings and the sky shader already built:
- **Meadow:** the card clouds lowered and smaller so they sit wholly in the frame, in two depths; a horizon haze band and a warm glow
  behind the mountain (the sky pass's look, with the city's `sky_bands`/`glow_height` to fit the gradient into the little the cameras see).
- **Grove:** a real gradient (a deeper violet at the top, a warm peach glow low behind the forest) and more colour (the gaps measure
  saturation 0.07), so the sky between the trees reads as evening light, not grey.

## To 5s: what a 5 needs, column by column

A 5 means a still holds up beside the references. From here every column is a 4, so each step is detail and finish, not fixing shapes.
In rough order of gain:

| Column | What stands between 4 and 5 | Effort |
|---|---|---|
| **Surfaces** | The play tops are the most-seen surfaces and still plain: the meadow's lawn measures contrast 0.069, the grove's 0.063 (review 5's "lawns without form", never built). A 5 has worn grass, tufts and clover, a path that looks walked on, edge wear; the beach's deck boards weathered and varied, with nail heads and a replaced board. Modelled into the play pieces, the tops kept at the sim's height. | Medium |
| **Background land** | The canyon's mesa behind the fight reads as a plain pink box at the match camera, and the train crosses at the fighters' head height. The city's middle and far towers are still painted flat cards. | Medium |
| **Sky** | After the two fixes above: clouds with lit and shaded sides on every day stage (the references' skies are layered and lit), and stars or a light cloud layer in the city's night. | Small to medium |
| **World and life** | No clones left you can spot in a still. A 5 is a lived-in world at true scale in every view: the beach's town with people and laundry, the meadow's farm worked (a cart, a farmer), smoke from more chimneys. No animals (your rule). | Medium |
| **Light** | Bounce, grade, glow and shadows are in. A 5 has shaped light on the play area: soft sun pools and cloud shadows on the meadow, dappled light under the grove's tree (there now, faint), lamp pools at night in the city, and the far scenery lit with as much care as the front. | Medium to large |
| **Cohesion** | Already one language per stage. A 5 needs every small piece at the same finish (the last code-built bits: lawn flowers, the beach's buoys and floats, the canyon's telegraph poles). | Small |
| **Readability** | Good everywhere. At 5 it depends on the fighters, which are half of every still (the character work), and on clearing the canyon's train from behind the fighters' heads. | Small (stages) |
| **Play area and front** | Fronts and platforms are now made of their stages. A 5 has the fronts' detail at the size you see them at the wide camera too (the meadow's wall reads busy there). | Small |

**Realistically:** all 4s is one round. A mean of about 4.4 to 4.5, with 5s in sky, cohesion, readability and play area, is three or
four more rounds of the same method (the order below). A 5 in surfaces and light is the largest investment: hand-authored detail on
every big surface and a lighting finish per stage, the kind of work a texture artist does. That's where the references' budgets show.

## The order

1. The meadow's and grove's skies (to all 4s).
2. The play tops: worn lawns, a walked path, weathered deck boards (surfaces, every stage's most-seen surface).
3. The canyon behind the fight (the mesa's shape, the train's line) and the city's middle towers modelled.
4. Shaped light on the play areas (sun pools, cloud shadows, lamp pools).
5. A lived-in world pass (people at scale, worked farms, chimney smoke).
6. The last code-built small things.

## The meadow's and grove's skies (2026-10-22, mock)

`StageArt._cloud_band` and `_cloud_scale` read the palette's [backdrop] `cloud_band` [low, high] and `cloud_scale` for the meadow's and
grove's clouds (the same random draws, so nothing else moves). Mocked as palette trials (`--pal-try=sky_a|sky_b`, meadow.cfg and
grove.cfg [try_sky_a|b]): the meadow A the clouds at 7 to 17 (from 14 to 34) and 0.85 the size, the gradient fitted to the visible sky
(`sky_bands` [0.012, 0.07], a deeper top); B with a warm glow behind the mountains and a softer horizon. The grove A an evening
gradient fitted to the visible sky (violet-blue top, peach horizon, a warm glow held low); B with the clouds at 8 to 18. Measured at the
match camera, the meadow's top of frame goes from hue 197, saturation 0.31 to hue 215, 0.35 (B). The grove's sky shows only between
the trees, so its change is the smaller. On the Stage Look Review ("skies7", `sk_*`); recommended B on both.
