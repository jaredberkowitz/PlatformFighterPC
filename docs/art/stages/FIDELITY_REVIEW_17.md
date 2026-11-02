# Stage fidelity review 17 (2026-11-02): the full pass

The owner, 2026-10-30: a pass on all of the stage art at full effort, to get closer to 5 in every category. Done straight on a branch,
art/stage-pass17 (from art/stage-refine e85e23d4), rather than as mocks to pick from. Every stage before and after, at the match camera
and wide, is on the Stage Look Review ("pass63", `ps63_*`). Not yet merged into art/stage-refine or in a build.

The same eight categories as reviews 10 to 16 (5 = the still could sit beside the references), judged at the player's 1920x1080.

**What the pass found:** most of what held the stages under 5 was faults, not taste. The flat, dark look, the slate tops and the "lit
from below" props each traced to a bug in a shared shader or a builder. Fixed at the root, every stage changed at once; the tuning that
followed was done on true light.

**Evidence:**
- `review17/fidelity17_sheet.jpg`: every stage after the pass (rows meadow, grove, beach, canyon, city; match camera left, wide right).
- `review17/before_after_<stage>.jpg`: each stage at the match camera, before (left) and after (right).
- `review17/review17_crops.jpg`: close crops at full size, before (left) and after (right): the meadow's stones, the grove's lip, the
  canyon's front and floor, the canyon's props.

## The root causes

| What was wrong | What it did on screen | Fixed in |
|---|---|---|
| **The faceted surfaces' normals pointed the wrong way** (`cross(dFdx, dFdy)`: this renderer's screen y runs down). Since 2026-10-18. | Every faceted face was lit as if from the far side. The stages' suns had been set from behind the scene to suit it, so every face toward the camera that wasn't faceted sat in shade: **the flat, dark look.** | `toon_common.gdshaderinc` (facets); the suns moved to the camera's side (meadow, beach), raking on the grove |
| **The drifting cloud shadows tint the colour of whatever faces up**, and only the floors had been cleared of them (the bright floors, review 16). | Slate tops on the meadow's stones and the grove's ledges; dark plum tops on every crate, barrel and rock over its own lit front, as if lit from below. | Off on the fronts (`StageBake.place_front`) and in the whole play area (`pf_play_area`, `stage_ao.gdshaderinc`) |
| **The noise hash in floats.** The GPU rounded one cell's hash differently from its neighbour's. | Hard vertical cuts in the sky's stratus at fixed screen columns on every stage; torn notches and fan-shaped slivers in the clouds. | An integer hash in the sky, clouds, brush and light |
| **The scenery's ink hull drew its back faces** in flat violet-grey, even with its ink off. | Purple slivers through the grove's moss lip, wherever an open shell showed its inside. | `outline.gdshader` |
| **A fixed pale grey** in the card-edge rim and the post's lit edges. | Grey metal strips on every top seen edge-on; pale grey frames round every dark recess. | Both now in the surface's own colour |
| **The far cards' tops flat per column** (`cutout` used one side's height at both). Since the cards were made. | The canyon's far horizon a stair of little steps against the glow. | `make_canyon_paper.py` |
| **The grove's lip tufts are domes turned over**, so each hung its lit crown under its shaded skirt. | Every tuft lit from below: a bright lime stripe along its hanging edge. | `make_grove_front.py` (light and shade swapped) |

## Per stage

**Meadow:**
- The sun on the camera's side.
- The lawn brighter, its path a warm sand, the banks a clear green in place of dark bands.
- The stones: pale card stone, each in its own tint and value again, with no slate tops and no stepped shadow dashes.
- The vines and moss in the lawn's own green, a step under it.
- Crisp tree and snow lines on the peaks.
- The terrace walls painted and printed; the furrows lighter.
- The farmhouse's smoke from its chimney's top: shorter, smaller, soft blobs (it stood across the dark pine like lumps of snow).

**Canyon:**
- The front as natural strata: long wandering beds of uneven thickness, soft beds set back, erosion hollows and loose joints
  (`make_canyon_front --strata C`).
- The platforms as two wandering beds in place of brick (`make_play --plat_joints C`).
- The far horizon smooth (the cards' tops), the far dunes' crest finer; the foreground out of the milky haze.
- The sand paler, its wear calmer, the real shadows on it lighter.
- The pebbles along the top's edge in small scatters (they read as a row of rivets).
- The crates, barrels and rocks lit on top.

**Beach:**
- The sun on the camera's side.
- The deck's honey planks and paler posts.
- The headland in sunlit greens.

**City:**
- The shop with goods in its windows, a lettered sign and a door lamp.
- The platform tops a pale lilac in place of near black.
- The patches quieter.

**Grove:**
- The sun raking.
- The rock a paler stone with a quieter print, clear of the real shadows' streaks.
- The trunks as long folded planes (they were a mosaic of little rectangles), the bark lighter.
- The lip: no purple slivers; the moss in the lawn's green, lit from above.
- The lawn biting into the path's edges in flat tongues (the path read as a stripe painted across it).
- The terraces lifted from near black; smaller glints.

**Every stage:**
- The clouds as clean cut-paper cumulus.
- The fighters' contact shadows in each floor's own deep tone.
- The platforms' shadows on the floors softer and half as strong.
- The print and paper fibre faded where they alias (a screen door on bevels at 1080 lines).
- Chimney smoke as small puffs rising apart.

**Also reaches the run mode:** it shares the shaders. Its tests pass, but its look has not been reviewed (its art is deferred).

## Measurements

At the match camera unless noted, before the pass (review 16's build plus item 6) and after it:

| Stage | Front value (contrast) | Floor value | Darks, wide | P1's gap | P1's head against the floor (Delta E) |
|---|---|---|---|---|---|
| Meadow | 0.46 (0.121) to **0.55 (0.140)** | 0.44 to **0.60** | 3% to 4% | 0.131 to 0.107 | 50 to 39 |
| Canyon | 0.26 (0.070) to **0.56 (0.120)** | 0.56 to **0.66** | 4% to 3% | 0.225 to 0.159 | 14 to **22** |
| Beach | 0.31 (0.106) to **0.36** (0.090) | 0.31 to **0.48** | 4% to 3% | 0.232 to 0.167 | 44 to 30 |
| City | 0.30 (0.177) to 0.27 (0.168) | 0.47 to **0.52** | 34% to 36% (night) | 0.183 to 0.150 | 39 to 29 |
| Grove | 0.29 (0.115) to **0.48** (0.117) | 0.54 to **0.60** | 17% to **7%** | 0.130 to 0.099 | 40 to 39 |

- **The fronts and floors are brighter on every stage but the city** (whose night is meant), most on the canyon, whose front doubled in value
  and nearly doubled in contrast. The grove's frame lost most of its near-blacks.
- **The fighters' value gap fell everywhere**, the cost of the bright floors you asked for. They still separate: a fighter reads if it stands
  apart from its floor by value (a gap of 0.12 or more) or by colour (a Delta E of 30 or more), and with its outline and contact shadow.
  Every stage clears one of the two: the meadow and grove by colour (39), the canyon by value (0.159; its colour difference, 22, is the
  lowest, though up from 14), the beach and city by both or by value.

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|---|---|
| Beach | 5 | **5** | 5 | 5 | 5 | 5 | 5 | 5 | **5.00** |
| Canyon | 5 | **5** | **5** | 5 | 5 | 4 | 5 | 5 | **4.88** |
| City | 5 | **5** | 5 | 5 | 5 | 5 | 5 | 5 | **5.00** |
| Grove | **5** | **5** | **5** | 5 | 4 | 5 | 5 | **5** | **4.88** |
| Meadow | 5 | **5** | 5 | 5 | 5 | 5 | 5 | **5** | **5.00** |

Mean **4.95** (review 16: 4.70). Bold: up. Thirty-eight of forty cells are 5.
- **Surfaces, 4 to 5 on all five.** With the faces lit the right way, every fold has a true lit side and shade side, so card reads as card:
  the meadow's stones, the canyon's beds, the grove's rock and moss, the dock's timber, the city's brick.
- **The grove's light** has a lit side and a shade side under the canopy, not a pink veil.
- **The play areas** of the canyon (strata, bright sand) and the grove (lit moss lip, a path the lawn bites into) are finished.
- **Readability on the meadow and grove** is judged by the rule above. By value alone they would still be 4.

## What's left

- **The grove's sky (4):** busy behind the trees, with little gradient in the gaps. It stays violet, as you chose; a clearer pink-to-violet
  gradient in the gaps is the one step left.
- **The canyon's world and life (4):** its life is small and far back, by the rule (no animals, true size). It may stay 4.
- **The fighters** remain outside this plan, and are now plainly the largest gap in any still.
- **At full size,** the canyon's beds show faint rectangles of tone, one per cut column. They read as paper pieces, so I kept them.
- **The run mode** shares the fixed shaders. Its tests pass, but its look hasn't been reviewed (its art is deferred).
- **Nothing of this is in a build yet.** Next: merge art/stage-pass17 into art/stage-refine and build, with the stage stills regenerated.

## After review 17: the grove's distance (mock, 2026-11-02)

The grove's sky, its one 4 that may change (the grove stays violet, as the owner chose). What made it busy is the near range behind the
forest (the baked ground's `range_near`). It is a saturated violet, darker than the sky, and it fills every gap between the trees, so the
frame read as a green and purple checkerboard. Found by painting the far families in pure colours.

`--pal-try=range_a|range_b|range_c` (grove.cfg [backdrop] `range_tones` [near, far], StageBake). Nothing is added or re-baked.

| Option | The near range: value, saturation, hue |
|---|---|
| Today | 0.47, 0.43, violet (266) |
| A, lighter | 0.63, 0.27, violet (276) |
| B, into the haze | 0.71, 0.16, lilac (287) |
| C, warm haze | 0.72, 0.18, pink lilac (314) |

P1's gap holds (0.095 to 0.102). On the Stage Look Review ("grovesky64", `gy64_*`).

Recommended B: the gaps read as calm distance, the trees stand out against it, and it is still violet. C turns the distance pink.

**Built B** (the owner's pick):
- grove.cfg [backdrop] `range_tones`; `--pal-try=pre_range` brings back the violets before.
- The near range measures 0.71 in value and 0.16 in saturation. P1's gap is 0.096.
- Before and after (match camera top, wide bottom): `review17/grove_distance_built.jpg`.

With it the grove's sky is a 5: the mean is 4.98, with 39 of 40 cells at 5. The one 4 left is the canyon's world and life, by the rule.
All seven tests pass.
