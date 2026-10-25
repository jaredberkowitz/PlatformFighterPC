# Stage fidelity review 9 (2026-10-25, after review 8's work): the path to 5

The owner: "If there are no more items from the last analysis to work on, we can do another of all four stages." Review 8's list is
built and released (playtest-2026-10-25-sim146-review8). This review covers all five stages: meadow, grove, beach, canyon, city. The
method is the same: eight categories scored 1 to 5 (5 = a still could sit beside the references), review 3's regions, the same
references. The judge is the hand that built the work, so read the scores as "the gaps this review can see".

**Evidence:** art/stage-refine 2492df9 (the released build), at the match camera and wide (`review9/fidelity9_sheet.jpg`: each row a
stage, match camera left, wide right: meadow, grove, beach, canyon, city), and close crops (`review9/review9_crops.jpg`, left to right
and down: the meadow's wall wide, the canyon's front, a canyon platform's cactus, the canyon's sky, the beach's clouds, the beach's posts
in the water, the grove's abyss, the city's street).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 8 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| City | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Grove | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Meadow | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 3.9 |
| Canyon | 4 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | **3.9** | 4.0 |
| **Mean** | 4.0 | 3.8 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | **3.98** | 3.98 |

- **What moved:** the meadow's wall is no longer grey (surfaces 3 to 4) and its range has painted form. The city's platforms are steel,
  the grove's underside is calmer, and the beach's loops are gone. The stronger key gave every fighter a clear shadow side. Those are
  real gains, but each closes a gap inside a 4, so no score reaches 5.
- **One 3, a step back:** the canyon's front. Laid in long straight beds (review 8, item 3), the rock now takes its `strata` print (thin
  broken horizontal lines every few pixels) along each bed's full length, so the front and the platform slabs read as **planks of wood**
  with grain, at both cameras. The wobble is gone, but it isn't rock.

## What stands between each column and 5

| Column | The gap | Where |
|---|---|---|
| **Surfaces** | The canyon's front and platform slabs read as wood grain (above). At the wide camera the meadow's wall is a grid of small pale stones, each outlined: it's warm now, but it's the busiest, brightest mass under the fight. | canyon, meadow |
| **World and life** | Under the dock, every post in the water has the same set of white ripple rings, so they read as a row of targets. The canyon's cacti under the platforms are stacked paper coils (the owner's pick B, 2026-10-21) that read as springs at play size. | beach, canyon |
| **Sky** | The beach's clouds are one unbroken strip of joined cloud cards along a single line, a border across the sky. At 5 the day skies want a second, farther cloud layer (open since review 8). | beach, meadow, grove |
| **Background land, cohesion** | The grove's abyss has pale blue pine cards standing in the purple, which reads as trees growing in water. The main island's underside (stone and roots) and its neighbours' (teal hex columns) are two languages. | grove |
| **Play area** | Good. The city's street at the wide camera is flat bands of purple with nothing on the road surface, the one plain stretch on that stage. | city |
| **Light** | The strong key reached the fighters, but scenery contrast moved within 0.01. The next lever is the painted shadow colours themselves (`pf_shadow_value`, `scenery_shade`), a larger pass with the fighters' readability measured again. | all |
| **Readability** | Good. The grove is the narrowest (P1's value gap to the ground 0.04); the edge light and hue carry it. | grove |

## The order

1. **The canyon's rock print:** a sandstone print for the front's beds and the platform slabs (speckle, cross-bedding in short arcs, a
   few weathering streaks down the face), in place of the long thin lines that read as grain. A step back, so first.
2. **The meadow's wall at a distance:** fewer, larger stones with joints darker than the faces, and a little lower in value, so the wall
   reads as one mass that sits back under the lawn. Also tidy the haybales and barrel strewn on the slope at its foot.
3. **The beach:** the ripple rings varied and softer (not every post, different sizes and phases), and the cloud strip broken into
   separate banks at different heights and depths.
4. **The grove's abyss:** the pale pine cards out of the purple (a removal, within the standing rule), and the neighbour islands'
   undersides brought toward the main island's stone (the same rock colour family at least).
5. **The canyon's cacti:** the owner's call. Keep the rolled coils, or try A's ribbed cactus under the platforms where they read as
   springs.
6. **The city's street:** a road surface on the flat bands (an asphalt print, lane wear, a kerb line), a refinement of what's there.
7. **The light at 5:** the painted shadow colours, mocked with the fighters' readability measured (item 7 of review 8, continued).

Items 1 to 6 are refinements of what exists, within the standing rule. Items 1 and 2 lift the surfaces column; 3 and 4 the beach's and
grove's worlds and skies. Item 7 is still the one change that moves every stage's light toward 5.

## Item 1, the canyon's rock (2026-10-25, built)

`tools/art/make_prints.py` family `sandstone` (`--sandstone A|B`, appended after `steel` so every other family's random numbers, and so
its print, stay byte-identical, checked against `prints/hatched`): sand grains, soft patches a little deeper (a first try at full
strength read as camouflage), and short sets of cross-bedding arcs, no long lines; B adds faint weathering streaks and pits. Mocked with
`--print-mock=<folder>` as `strata.png`, so it lands on everything `strata` printed: the front's beds and the platform slabs (`layer1`,
`layer2`, `strata_*`; the buttes' `rock_*` families carry no print). The print takes the grain away at the match camera; at the wide
camera the even 0.75 beds, a step lighter and darker by turns, still stacked like planks, so C adds `make_canyon_front.py --straight C`:
beds of uneven thickness (thin partings of 0.3 to 0.5 between beds of 0.9 to 2.2), 6 to 14 long, the tone stepping less (15,848
triangles). On the Stage Look Review ("sand9", `s9_*`, C shown before the light bake); recommended C.

Built C (the owner's pick): `prints/hatched/sandstone.png` (`--style B --sandstone A`; only that file copied in, every other print
unchanged), mapped for the canyon's `layer1`, `layer2` and `strata_*` in `StageBake.PRINTS_BY_THEME`; canyon_front.glb rebuilt with
`--straight C`; the canyon re-lit (front, paper, play). The front measures value 0.39 (0.37 before: fewer, larger flat faces), with
sunset.cfg `front_lift` 0.35 to 0.28 (it moves the face only 0.01). Before and after (match camera, then wide):
`review9/canyon_rock_built.jpg`. All seven tests pass.
