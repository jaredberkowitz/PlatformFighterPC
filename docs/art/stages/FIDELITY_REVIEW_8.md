# Stage fidelity review 8 (2026-10-24, after review 7's work): the path to 5

The owner: "continue working on what we found from our previous analysis. If we finished everything from that, we can do another
analysis of all stages, to figure out how we can get to 5/5 in all categories." Review 7's list is done (its "lived-in world" item
needs new elements, which the standing rule rules out unless the owner asks). The same method: eight categories scored 1 to 5 (5 = a still
could sit beside the references), review 3's regions, the same references. The judge is the hand that built the work: read the scores
as "the gaps this review can see".

**Evidence:** the build released as playtest-2026-10-24-sim146-review7 plus the modelled small things (art/stage-refine 0fc03027), at the
match camera and wide (`review8/fidelity8_sheet.jpg`: meadow, grove, beach, canyon, city), and close crops (`review8/review8_crops.jpg`:
the meadow's wall, the meadow's mountain, the canyon's front wide, the grove's underside, the city's scaffold platform, the beach's sea).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 7 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Canyon | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| City | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Grove | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 3.9 |
| Meadow | 4 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | **3.9** | 3.9 |
| **Mean** | 4.0 | 3.8 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | **3.98** | 3.95 |

- **The skies closed** (the meadow and grove 3 to 4). **No score is 5.**
- **One 3, a defect:** the meadow's fieldstone wall, the most-seen surface under the fight, renders grey (saturation 0.02 at value 0.43:
  its stone tints don't survive the front's paint) with neon-green moss drips down the stones. It reads as a grid of grey cubes, colder
  and busier than the old wall. That's a step back, and the first thing to fix.

## What stands between each column and 5

Every column is at 4 now, so a 5 needs finish, not fixed shapes. Here is what keeps each from a 5, most visible first:

| Column | The gap | Where |
|---|---|---|
| **Surfaces** | The meadow's wall (above). The canyon's sediment courses read as wobbly stacked plywood at the wide camera: every course bulges by the same amount. A 5 has surfaces you'd believe up close: varied fieldstone in warm greys, courses of rock that run straight and break cleanly. | meadow, canyon |
| **Background land** | The meadow's big mountain fills the top third of the frame as a few flat teal planes with streaks; the references' ranges carry painted form (snowcaps, rock bands, a tree line, the far range paler). The grove's underside is busy, rock, roots and moss leaves all at one weight. | meadow, grove |
| **Play area** | The city's scaffold platforms are plain flat slabs (a dark purple card on a lattice); the other stages' platforms were rebuilt in their own materials, the city's weren't. | city |
| **World and life** | The beach's paper waves still read as white loops drawn across the sea. | beach |
| **Light** | The ceiling of the paper look: ambient and the paper finish carry most of every surface's light (the meadow's lawn with no sun at all falls only 0.71 to 0.65), so shaped light, cloud shade and form stay faint. A 5 needs the play area lit with a real key: more form on everything, deeper but coloured shade. It's a game-wide change that has to keep the fighters readable, and the largest risk on the list. | all |
| **Sky, cohesion** | Good. At 5 the day skies want a second, farther cloud layer, and every stage wants one finish level from the front to the horizon (the meadow's mountain is the odd one out). | meadow, grove |
| **Readability** | Good. At 5 it rests on the fighters, half of every still (the character work). | all |

## The order

1. **The meadow's wall fixed:** warm, varied fieldstone, with moss in the joints at a believable green, not neon drips. A defect, so first.
2. **The meadow's mountain:** painted form on the range (snowcaps, rock bands, a tree line at its foot), its far range paler.
3. **The canyon's front:** courses that run straight with clean breaks, not uniform wobble.
4. **The city's scaffold platforms** in the city's own materials (a steel deck with plates and rivets, a painted edge).
5. **The grove's underside** calmed: fewer, larger rock masses, roots in one or two groups, moss at the lips only.
6. **The beach's waves** as soft paper bands rather than white loops.
7. **The light at 5** (game-wide): a real key on the play area. Mock it with the fighters' readability measured.

All four 4.0s and a 3.9 is about where a paper look made with code and Blender scripts lands. Items 1 to 6 are refinements of what
exists, within the standing rule, and should lift surfaces, background land and the meadow to clear 4s with 5s in reach. Item 7 is the
one that can move every stage's light and surfaces toward 5.

## Item 1, the meadow's wall (2026-10-24, built)

Why it was grey: the fieldstone took the old wall's near-grey `stone` (0.66, 0.64, 0.60), and the front's calm (a desaturation) and its
cool shade with depth took what warmth was left; the moss took the lawn's lime. `make_meadow_front.py --warm A|B` paints the stones in
their own family `fieldstone` (0.80, 0.70, 0.56) and the moss in `wall_moss` (a deep olive), as low caps with no drips; B widens the
stones' warm-to-cool spread and keeps the moss under the logs. Built B (the owner asked for a fix rather than a pick), the meadow
re-lit: the wall measures hue 32, saturation 0.19 (from saturation 0.02). Before and after: `review8/meadow_wall_built.jpg`. Tests pass
except the runner's two known flaky checks, which failed in 3 of 5 runs this time (the runner doesn't load the meadow's front).

## Item 2, the meadow's mountains (2026-10-25, built)

`toon_common.gdshaderinc` `alt_bands` [snow line, rock top, tree line, amount] with `alt_snow`, `alt_rock`, `alt_foot`, `alt_rock_bands`:
a modelled range painted by height, snow on the faces turned up above a ragged snow line (more near the summits), broken rock strata
between, a deep green tree line round the foot. `StageArt.ridge` sets it from a spec's `alt` (heights above the range's foot) with the
palette's [backdrop] `range_form` (`--range-form=1|2`), the colours through the air at the range's depth. The meadow's specs: far
[54, 48, 22], near [38, 32, 16] (first tried higher, the main peak took almost none). A snow and tree line; B with rock bands (they barely
show). On the Stage Look Review ("mountain8", `m8_*`); recommended A.

## Item 3, the canyon's front (2026-10-25, built)

`make_canyon_front.py --straight A|B` (with `--cut 1 --courses 1`, `courses_straight`): each course laid in long flat-fronted beds of
rock (4 to 9 units, few clean vertical joints, each a little in or out; beds 0.75 thick) in place of one jutting front shared by every
course, which bulged them all alike into wobbly plywood at the wide camera. Blocks of 1.4 to 3.6 were tried first and read as a brick
wall. B sets every third bed back as a shadow line (barely visible). 21,600 triangles, from 38,500. On the Stage Look Review
("canyonfront8", `c8_*`); recommended A.

## Items 2 and 3 built (2026-10-25)

The owner picked A on both. The meadow's palette sets [backdrop] `range_form=1` (the snow and tree line, no rock bands). The canyon's
front is rebuilt with `make_canyon_front.py --variant A --cut 1 --courses 1 --straight A` (21,632 triangles), and the canyon is re-lit
(the light bake rewrote its paper and play pieces too). The canyon front measures value 0.35, saturation 0.57. Before and after
(the meadow at the match camera, the canyon wide): `review8/mountains_canyon_built.jpg`. All seven tests pass.

## Item 4, the city's platforms (2026-10-25, built)

`make_play.py --theme city --plat deck|boards` (with `--variant A --wear 2`): the one set of platforms still a plain dark card, in the
city's own materials, nothing above the walking surface. A, `deck`: steel plates with their seams (`steel_seam`), a painted safety line
along the front edge (`steel_paint`), on a riveted girder in red-oxide primer (`steel_primer`, `rivet`) with a joist under every seam. A
first try with a deep steel channel went black in its own shade, so its rivets read as a dotted strip of lights. B, `boards`: four
scaffold boards (`deck_board`) with steel hoops at their ends, on steel tubes with clamps, over a toe board in hazard stripes (`hazard`).
The new families are painted in `StageBake.place_play` and printed as steel or wood. On the Stage Look Review ("scaffold8", `p8_*`);
recommended A (it ties to the towers and the water tower, and stays calm by the fighters; B's stripes are the loudest thing at platform
height).

Built A (the owner's pick): city_play.glb rebuilt with `--theme city --variant A --plat deck --wear 2`, the city re-lit (`light_stage.py`
on the front and the play area, curve and reach 2.5). Before and after (match camera, then wide): `review8/city_platforms_built.jpg`.
All seven tests pass.

## Item 5, the grove's underside (2026-10-25, built)

`make_grove_front.py --variant B --calm A|B` (`stones_calm`): the same stones, roots and moss, fewer and larger. A: stones of 1.2 to
1.9 in rows about two units apart, their tones closer; moss caps only on the top row, under the lip; the roots in two groups (a pair on
the left, one on the right), each with its branch. B: the stones gathered into masses of two to four, each mass one tone, with dark earth
between; the roots in one group from the left. The calm stones lie flatter against the island (a first try at full depth hid the roots
behind them and read as pale grey blocks), and the roots stand a little further out. 11,100 (A) and 9,600 (B) triangles, from 16,300.
On the Stage Look Review ("underside8", `u8_*`); recommended A (two root groups frame the keel; B leaves its right side bare).

Built A (the owner's pick): grove_front.glb rebuilt with `--variant B --calm A` (11,146 triangles), the grove re-lit (`light_stage.py`
on the front and the play area, curve and reach 2.5; the background's paper, with its baked sun, left as it was). Before and after
(wide, then the match camera): `review8/grove_underside_built.jpg`. All seven tests pass.

## Item 6, the beach's waves (2026-10-25, mock)

`water.gdshader` `wave_band` (the palette's [backdrop] `sea_band`, `--sea-band=0|1|2`) on the paper sea (`look` 3): the white loops were
each sheet's thin near-white lip following the cusps between its caps. 1, soft bands: each edge a rounded swell (a plain sine, three
quarters as deep), no cusps; its lip a wide soft lift in the water's own paler tint; narrower sheets near the stage (9 units, from 14) so
the bands still read. 2, tone tiers: no lip; each sheet a deeper step of tone (0.18) shaded toward its edge. A first pass at the soft
settings left the sea almost plain; the contrast was raised. On the Stage Look Review ("waves8", `v8_*`); recommended A (it keeps the
waves and the bright sea; B reads more as depth than as waves).
