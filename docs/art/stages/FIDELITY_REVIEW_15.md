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

Built C (the owner's pick): meadow.cfg [backdrop] `fight_ground=[0.42, 0.08]`. In the game P1's gap is 0.128. Before and after (the fight,
match camera): `review15/meadow_ground_built.jpg`. All seven tests pass.

## Item 2, form the reference's way (mock, 2026-10-28)

`--front-form=A|B` (a style's finish `front_form` [lit edge, joint ink, band]; post.gdshader `front_form`). Within the band (4 units of
the stage plane, fading over 4 more), on the scenery's edges (the pass's own depth-step and crease tests, nearer side only):
- an edge on a face turned toward the sun (its view normal against `pf_sun_dir`) is lifted as a lit band in its own colour
  (`col * 1.3 + warm`), tested out to 2 and 3 line widths so it reads as a band;
- an edge on a face turned away is inked at `front_form.y` in place of the scenery's 0.4.

A [0.9, 0.7, 4], B [1.6, 1.0, 4]. A first try lit only a one-pixel line and barely showed.

| Front at the match camera (value, contrast) | Today | B |
|---|---|---|
| Meadow | 0.41, 0.074 | 0.42, 0.074 |
| Canyon | 0.29, 0.064 | 0.29, 0.066 |
| Beach | 0.29, 0.093 | 0.30, 0.091 |
| City | 0.32, 0.164 | 0.34, 0.170 |
| Grove | 0.29, 0.103 | 0.29, 0.110 |

The region averages hide a thin effect; by eye:
- the meadow's stones read as cut card again, each with a lit rim on its sun side;
- the city's frames and sills get crisp lit edges;
- the canyon's big flat blocks change least.

P1's gap holds. On the Stage Look Review ("frontform53", `ff53_*`, today and B); recommended B. To ship: STYLE_DEFAULT finish
`front_form=[1.6, 1.0, 4.0]`.

Built B (the owner's pick): STYLE_DEFAULT finish `front_form=[1.6, 1.0, 4.0]`, on every stage; `--front-form=T` turns it off for
comparison. P1's gap in the game:

| Meadow | Canyon | Beach | City | Grove |
|---|---|---|---|---|
| 0.129 | 0.188 | 0.231 (0.287 before) | 0.155 | 0.142 |

The beach's gap narrowed most (its lit sand edges sit near P1) and is still the widest. Before and after (left, right; the meadow, the
city; match camera): `review15/front_form_built.jpg`. All seven tests pass.

## Item 3, the canyon's two hues (mock, 2026-10-28)

**Why it was one hue:**
- **The shade went the short way round.** `pf_painted_shadow` turns a colour's hue toward the shadow hue by the shorter way round the
  wheel, at most `shadow_shift`. From the rock's orange (about 0.05) to violet (0.72) the shorter way runs backward through red and
  pink, and the 0.25 limit stopped it at magenta.
- **The violet fill and the grade pinked the lit rock.** The fill light and the grade's shadow tint mixed violet into the orange.
- **The ambient held the shade sides red.** It is untouched by the painted shade, so it adds the rock's own red to every shade side.
- **The distance is the same rosy rock as the front.**

`--two-hue=A|B|C|D|E` (Lighting.TWO_HUE_TRY, the canyon only; the shift limit is now a mood's `shift_cap`):

| | What it changes | Front value, hue, sat | P1's gap |
|---|---|---|---|
| Today | | 0.29, 356°, 0.69 | 0.188 |
| A | shade the long way to violet (shift 0.34), tint, ambient and casts cooler | 0.29, 356°, 0.68 | 0.206 |
| B | A bluer (0.4 to hue 0.68), sun gold, grade shadows blue-violet, violet air 0.16 | 0.29, 358°, 0.63 | 0.206 |
| C | B, fill and ambient blue, grade lights warmer | 0.31, 2°, 0.59 | 0.176 |
| D | C, most of the ambient into the key (`key_fill` [0.1, 0.3]) | 0.29, 3°, 0.64 | 0.188 |
| E | D, the air 0.34 toward a blue-violet (0.64, 0.64, 0.94) | 0.28, 2°, 0.64 | 0.189 |

**A to C barely show.** The sun comes from the front left, so few faces toward the camera sit in shade.

**E is the one that splits the frame:**
- warm near rock and sand;
- a lavender distance under the violet upper sky;
- the gold sun between them.

The front's red goes a little browner. The next lever, if that reads dull, is the rock's own colour, a re-bake.

On the Stage Look Review ("twohues54", `th54_*`: today, D, E). Today against E: `review15/canyon_two_hues_E.jpg`. Recommended E.

To ship E, write into MOODS.sunset:
- `shadow_hue` 0.68, `shadow_shift` 0.4, `shift_cap` 0.4;
- the tint, ambient, key, fill, grade and haze;

and into STYLES.sunset:
- `cast`;
- finish `haze_max` 0.34;
- `key_fill` [0.1, 0.3].

Built E (the owner's pick):
- **MOODS.sunset:** E's shade, tint, ambient, key, fill, grade and haze.
- **STYLES.sunset:** `cast`, `key_fill` [0.1, 0.3] and finish `haze_max` 0.34. A stage's own `key_fill` now comes before the colourful
  shade's 0.45.
- **No re-light.** The light export bakes only the occluders, not the mood.
- **Comparison:** `--two-hue=T` brings back the look before.

In the game the front measures 0.29, 3°, saturation 0.64; P1's gap is 0.191. Before and after (match camera):
`review15/canyon_two_hues_built.jpg`. All seven tests pass.

The fighters' shade takes 0.35 of the stage's shift, so on the canyon it now turns 0.14 rather than 0.09: a touch pinker in their shade
sides.
