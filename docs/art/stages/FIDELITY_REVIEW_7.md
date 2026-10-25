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

**Built (2026-10-22): skies B** (the owner's pick): meadow.cfg and grove.cfg [theme] and [backdrop] (the skies before as
[try_pre_sky7]); no re-light. That closes review 7's two 3s: by this review's own measure every column is now a 4. Before and after:
`review7/skies_built.jpg` (the meadow at the match camera and wide, the grove wide). All seven tests pass. Next: the play tops.

## The play tops (2026-10-22, mock)

The first step toward 5s in surfaces. The stage test lets nothing poke up where fighters stand, so the detail lies flat a hair over the
top (`wear_top`: hand-cut patches; `patch_flat`), with tufts or pebbles only in the front strip the test leaves out (`edge_tufts`,
`edge_pebbles`) and nail heads flush on the dock (`board_marks`). `--wear 1|2` on `make_meadow_play.py` and `make_play.py`; families
`top_deep`, `top_lit`, `top_worn` (a fifth of a step either side of the top's own colour, printed as the top), `nail`. A: worn and
varied (lawns: clover, sunlit patches and trodden earth fraying along the path, kept off it; canyon: drifts and pale wind patches; dock:
bleached and wet patches and two replaced boards; roof: tar patches and faded sheets); B: with the edges (the city's B is its A).
Lessons: at a tenth of a step the patches vanished under the print and the lawn's fall; laid over the path they broke it up. The lawns'
measured contrast moves little (meadow 0.071 to 0.073, grove 0.067 to 0.077): a quiet change by design. On the Stage Look Review
("playtops", `pt_*`); recommended B.

**Built (2026-10-22): play tops B on every stage** (the owner's pick): all five play pieces rebuilt with `--wear 2`; every stage re-lit.
Before and after: `review7/play_tops_built.jpg` (meadow, grove, canyon, beach, city). All seven tests pass (the runner's "coin picked
up" check flaked once and passed twice on its own). Next: the canyon behind the fight.

## The canyon behind the fight (2026-10-22, mock)

`make_canyon_paper.py --fight broken` (`butte_broken`): the butte behind the fight as a broken table of rock, wide and low (2.5 times
the bounds' half-width, 0.42 of the height: at 1.7 and 0.62 each block stood taller than wide, three chimneys), three blocks of uneven
height with crevices, the middle a step lower, each banded with a dark cap, fallen blocks; drawn from its own random numbers, the
wide mesa still drawn and thrown away so every butte after it keeps its layout. `make_world.py --trestle far` (175 back, deck at -9):
the train small and hazy near the horizon. Lowered instead (`-11` at 124 back) it ran right behind the fighters' name tags, so that was
dropped. Options: A the broken table; B with the railway far back. On the Stage Look Review ("canyon7", `c7_*`); recommended B.

**Built (2026-10-22): canyon B** (the owner's pick): `canyon_paper.glb` `--fight broken`, `canyon_world.glb` `--trestle far`; the canyon
re-lit. Before and after: `review7/canyon_behind7_built.jpg`. All seven tests pass. Next: the city's middle towers.

## The city's middle towers (2026-10-22, tried and dropped)

Built the middle layer as a model the way the near one is (its boxes exported, frames, sills, lintels and ledges as real relief, 116
towers, 89,000 triangles), and with the shader's full detail painted on it. At both cameras it made no visible difference
(`review7/city_mid_towers_test.jpg`: today, the model, today, the model with the full paint, at the wide camera): at that distance the
paper look squashes the layer into cards and the air hazes it, so relief and painted brick fall under a pixel. Not shipped, and the
code taken back out (kept as a patch in the session's scratch). What does make the middle layer read as more than flat cards is its
silhouette and light, not its surface: varied rooftops and crowns, and lit edges where the glow catches them. That folds into the
light item ("shaped light", lamp and glow pools) rather than a modelling pass.

## Shaped light (2026-10-23, mock)

Findings first. A "sun pool" lift on the play tops can't read: the filmic tone curve squeezes a lift on an already bright lawn to
almost nothing, and even with the sun switched off the meadow's lawn only fell 0.71 to 0.65 (ambient and the paper finish carry most of
its light), which is also why the existing cloud shadows never showed. And at 14 to 30 units across, a whole stage sat in one gap of
the cloud field. So the light is shaped from the shade side: `pf_cloud_tone` (a new global: the tone a clouded spot takes, laid on the
surface's own colour in toon_common's fragment, so it shades ambient and sun alike), smaller clouds (8 to 12 across) over about 60% of
the ground near the stage, and the sun falling in the gaps; `lighting.gd` `SHAPED_TRY`, `--shaped=A|B`, a style's own `cloud_tone`.
The city at night: the roof's lamp pools brighter and wider (the palette's `lamp_pools` [alpha, size]) and the street's glow catching the
towers (`facade.gdshader` `glow_edge`: a warm edge up each front's corner away from the moon, strongest low, and along the parapet;
the palette's `skyline_glow_edge` a layer); first drawn a third of a unit wide it was a pixel or two, so it is a unit wide. A soft,
B firm. On the Stage Look Review ("shapedlight", `sl_*`); recommended A (B darkens the play areas overall and greys the canyon's sand).
It reads more as a partly cloudy day than crisp pools of sun: the paper look's flat, bright shading limits it.

**Built (2026-10-23): shaped light A** (the owner's pick): A's `clouds` and `cloud_tone` in the meadow's, grove's, canyon's and beach's
styles; city.cfg `skyline_glow_edge=[0.45, 0.35, 0.2]` and `lamp_pools=[0.68, 1.25]`; `--shaped=T` shows the light before. No re-light.
Before and after: `review7/shaped_light_built.jpg` (meadow, grove, canyon, beach, city at the match camera). All seven tests pass.

## The last code-built small things (2026-10-24, shipped without a pick)

`make_paper_props.py` gains `telegraph_pole` (a scored rolled pole 9 tall, a braced crossarm, glass insulators), `net_float` (a rolled
float with a white band and a rope loop) and `leaf_tuft` (seven curled leaf strips); the kit `paper_props.glb` rebuilt with them
(`--style C --variant A --b stump,crate,cactus`), and the canyon's poles, the stilt hut's floats and the grove's flower clusters (the
kit's flower and leaf tuft) use them, the built ones kept as fallbacks. At both cameras they change nothing visible (the poles stand
behind the arch at the wide camera, the floats far back past the dock, the flowers are a few pixels), so it went in under the standing
rule (modelled over code-built) without a pick. All seven tests pass (`prop_check` lists the float as sitting in the water).

**Review 7's path-to-5 list is done except the "lived-in world" item** (people in the town, a worked farm, more chimney smoke): those
are new elements, which the standing rule (refine, don't add) rules out unless the owner asks for them.
