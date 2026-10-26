# Stage fidelity review 12 (2026-10-26, after all the new art): where we stand

The owner: "Please continue on art improves that would get us to 5/5s." Review 11's four items and every new-art option are built and
released (playtest-2026-10-26-sim146-valley). Same method as reviews 10 and 11: eight categories scored 1 to 5 (5 = a still could sit
beside the references), the same references, the fighters' readability measured. The judge is the hand that built the work, so read the
scores as "the gaps this review can see".

**Evidence:** art/stage-refine 238df94c at the match camera and wide (`review12/fidelity12_sheet.jpg`: each row a stage, match camera
left, wide right: meadow, grove, beach, canyon, city).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 11 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** | 4.1 |
| Canyon | 4 | 4 | 4 | 5 | 4 | 4 | 4 | 4 | **4.1** | 4.0 |
| City | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** | 4.1 |
| Grove | 4 | 4 | 4 | 5 | 4 | 5 | 4 | 4 | **4.25** | 4.1 |
| Meadow | 4 | 4 | 4 | 5 | 4 | 5 | 4 | 4 | **4.25** | 4.25 |
| **Mean** | 4.0 | 4.0 | 4.0 | 4.6 | 4.0 | 4.8 | 4.0 | 4.0 | **4.18** | 4.13 |

- **Background land and world and life are at or near 5 everywhere.** The canyon's gorge, hoodoos, mesa foot and jointed front make its
  land a 5; the grove's islands over their valley with the falls and the lit village make its land a 5. The beach's land (sea and
  headland) and the city's towers stay 4.
- **The 4s left are the surfaces seen most, at the match camera:** the big flat planes the fighters stand on and in front of. The wide
  camera hides them among the new art; the match camera, which players see most, does not.
- **Readability holds:** P1's gap meadow 0.013, grove 0.062, beach 0.177, canyon 0.041, city 0.131 (the meadow's narrowest, as before).

## What still blocks 5 (at the match camera)

| Gap | What it is | Where |
|---|---|---|
| **The city's play roof** | The largest surface behind the fighters: one flat pink plane, its few tar patches lost under the lamp pools. | city: surfaces, play area |
| **The meadow's mountains** | The two great peaks behind the fight are big flat pale facets, the plainest thing in the meadow's match frame. | meadow: sky and land, surfaces |
| **The canyon's platforms** | The floating platforms are plain flat red slabs with a thin pale top. | canyon: play area |
| **Painted variation and the fighters** | As in reviews 10 and 11. | all |

## The order

1. **The city's roof laid in felt** (`make_play.py --roofing A|B`): lapped sheets running front to back, seams and laps flush with the
   top. Mocked below.
2. **The meadow's mountains:** broad painted value in the peaks' faces (lit and shaded flanks, snow worn thinner on the lower slopes),
   no new shapes.
3. **The canyon's platforms:** the sandstone beds and joints of the front on the platforms' faces.

All refine what exists.

## Item 1, the city's roof in felt (mock, 2026-10-26)

`make_play.py --roofing A|B` on the shipped city play (`--theme city --variant A --plat deck --wear 2`): felt sheets 1.25 to 1.55
across running front to back, each seam a 0.06 dark line (`top_deep`) with a 0.12 pale lap beside it (`top_lit`), flush at the top +
0.007 (the stage test lets nothing over 0.02 where fighters stand); B scatters gravel specks over it, which don't show at play size. The
roof region at the match camera: value 0.53 today (lit) and 0.55 with A or B (unlit, as mocks are), contrast 0.144 to 0.149. P1's gap
0.112 with A and B, the same as today's roof unlit (0.112): the narrowing from 0.131 is the missing light bake, not the felt. On the
Stage Look Review ("roofing30", `rf30_*`); recommended A. To ship: rebuild `godot/models/stages/city_play.glb` with `--roofing A`, then
re-light the city.

Built A (the owner's pick): `godot/models/stages/city_play.glb` rebuilt with `--roofing A`, the city re-lit (front, play; `--curve 4.0
--reach 4.5 --floor 0.28` after a fresh `--light-export`). In the game the roof measures value 0.51, contrast 0.143; P1's gap 0.134
(0.131 before). Before and after (the roof, match camera): `review12/city_roof_built.jpg`. All seven tests pass.

## Item 2, the meadow's mountains (mock, 2026-10-26)

`StageArt.ridge`: the palette's [backdrop] `range_drift` or `--range-drift=A|B` sets the modelled ranges' `paint_drift` (the play
areas' painted variation): A [3.0, 1.2, 24, 0.8], B [4.5, 1.6, 16, 1.0]. The depth haze takes back most of it: a first try at [1.6, 0.9,
30, 0.6] moved the peaks' contrast 0.105 to 0.112, and the rock bands (`range_form` 2) tried with it don't show. The peaks at the match
camera: contrast 0.105 today, 0.129 A, 0.119 B (B's smaller patches read as mottling); P1's gap 0.013 today, 0.017 A, 0.015 B. On the
Stage Look Review ("mountains31", `mt31_*`); recommended A. To ship: meadow.cfg [backdrop] `range_drift=[3.0, 1.2, 24.0, 0.8]` (shader,
no re-light).
