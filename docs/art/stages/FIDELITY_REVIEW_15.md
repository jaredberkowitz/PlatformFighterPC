# Stage fidelity review 15 (2026-10-28): after review 14 and colourful shade

The same eight categories as reviews 10 to 14 (5 = the still could sit beside the references), scored on art/stage-refine 207c707a.
That is all of review 14 plus the colourful shade from the owner's note ("the shadows are starting to wash out the colors on the stage
front"). The owner's direction since then weighs every score below: shade light and coloured as the papercraft reference has it, form
from lit edges and drawn detail rather than dark crevices.

**Evidence:** every stage at the match camera and wide (`review15/fidelity15_sheet.jpg`; rows meadow, grove, beach, canyon, city; match
left, wide right). Close crops (`review15/review15_crops.jpg`), left to right, top to bottom:
- the meadow's front stones; the fighters on the meadow's top;
- the canyon's hoodoos; the canyon's whole match frame;
- the beach's headland behind the town; the city's facade;
- a grove fall; a grove mist bank.

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|---|---|
| Beach | **5** | 4 | 5 | 4 | **5** | 5 | 5 | 5 | **4.75** |
| Canyon | 4 | 4 | 4 | 5 | **5** | 4 | 4 | **5** | **4.38** |
| City | **5** | 4 | **5** | 5 | 5 | 5 | **5** | 5 | **4.88** |
| Grove | 4 | 4 | 4 | 5 | 4 | 5 | 5 | **5** | **4.5** |
| Meadow | **5** | 4 | 5 | 5 | **5** | 5 | **5** | 4 | **4.75** |

Mean **4.65** (review 14: 4.38). Bold: up since then. Twenty-six of forty cells are 5.
- **Light:** the meadow, beach and city now have light, coloured shade with a clear lit side.
- **Sky:** every cloud is whole.
- **Readability:** the canyon and grove now hold P1's gap.
- **Cohesion:** the meadow's mountains are rock and the city's building is warm against the night.

**Every stage is still 4 on surfaces.** That is the category left on all five.

## What the measurements say

| Stage | Front value (sat) at the match camera | Darks, wide | P1's gap |
|---|---|---|---|
| Meadow | 0.41 (0.55) | 3% | 0.105 |
| Canyon | 0.29 (0.70) | 5% | 0.191 |
| Beach | 0.29 (0.41) | 6% | 0.287 |
| City | 0.32 (0.57) | 34% | 0.152 |
| Grove | 0.29 (0.31) | 17% | 0.145 |

**The fronts now sit near the reference's shaded faces.** It is the owner's direction: light frames with few darks, as the reference
has.

**The meadow's gap fell to 0.105:** the land round its fighters lightened with the shade. It is the only readability cell below 5.

## What blocks 5, by category

| Category | What blocks it now |
|---|---|
| **Surfaces** (all 4) | Meadow: the wall's stones lost their form with the dark crevices, pale blocks under dark moss caps. Canyon: the hoodoos are stacked drums. Beach: the big headland behind the town is one flat pale card. City: the facade is one repeating flat brick, the roof's top plain. Grove: the falls' sheets have straight tops and read as tiles, and the mist banks are flat violet. |
| **Light** (canyon, grove) | Canyon: still one pink-orange hue; its violet shade is gentle and its sun shadows faint. Grove: a soft pink veil with little lit side and shade side. |
| **Play area and front** (canyon, grove) | Canyon: the strata are layered, but still straight blocks. Grove: the top is a plain lawn with the earth path. |
| **Background land** (beach) | The headland's pop-up flats are one pale tone with no shade side. |
| **Sky** (grove) | Busy pink-violet behind the trees, little gradient. |
| **World and life** (canyon) | Its life is small and far (rule: far back, at true size); may stay 4. |
| **Cohesion** (canyon) | One hue from the floor to the sky. |
| **Readability** (meadow) | P1's gap 0.105. |

## The plan to 5 (all refine what exists; none adds new things)

| # | Item | Stages | Lifts | Effort |
|---|---|---|---|---|
| 1 | **The meadow's ground a step deeper again** (`fight_ground` from 0.32 to about 0.42), back above 0.12. | meadow | readability | small |
| 2 | **Form the reference's way:** a lit cut edge along the top and lit side of the fronts' stones, strata and bricks, and a thin drawn line in their joints, instead of dark crevices. One toon/post change, scenery only. | meadow, canyon, city (and grove, beach fronts) | surfaces (3 to 5 cells) | 1 session |
| 3 | **The canyon's two hues:** the shade and cast shadows a clear cool violet-blue, the sunlit sand warmer, the sky's upper violet carried into the far buttes. | canyon | light, cohesion | 1 session |
| 4 | **The beach's headland painted:** the pop-up flats given painted patches and a shade side toward the town, as the meadow's peaks have. | beach | background land, surfaces | 0.5 to 1 session |
| 5 | **The grove's falls and mist:** the sheets' tops and foam lips scalloped; the mist banks greyer at their tops, violet only in their shade. | grove | surfaces | 1 session |
| 6 | **The canyon's hoodoos:** eroded, tapering columns with a cap rock, not stacked drums. | canyon | surfaces | 1 session |
| 7 | **The grove's light and sky:** a warmer lit side under the canopy and a clearer gradient in the gaps. | grove | light, sky | 0.5 session |

**Honest ceiling:**
- Items 1 to 3 lift up to seven cells; the mean would reach about 4.8.
- The canyon's world and life and the grove's play area may stay 4.
- The fighters stay outside this plan, and they remain the largest gap in any still.

**Recommended start:** item 1, the cheapest, then item 2, the widest gain.

## Item 1, the meadow's ground a step deeper (mock, 2026-10-28)

`--ground=C|D` (`StageBake.place_play`, the same walked families as review 14's item 2): C [0.42, 0.08], D [0.5, 0.1], against the
meadow's shipped [0.32, 0.08]. Under the colourful shade the land round the fighters lightened, and P1's gap fell from 0.138 to 0.105.

| | Today | C | D |
|---|---|---|---|
| P1's gap | 0.105 | 0.121 | 0.135 |

The frame's values hold (wide darks 3%). D reads well at the match camera, but at the wide camera its top is a dark slab on a lighter
front and lighter land, the values upside down. On the Stage Look Review ("meadowground52", `mg52_*`); recommended C. To ship:
meadow.cfg [backdrop] `fight_ground=[0.42, 0.08]`.
