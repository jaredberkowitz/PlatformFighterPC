# Stage fidelity review 11 (2026-10-26, after the new art): where we stand

The owner: "Please continue working on other outstanding items that would help us get to 5/5." Every item of the new-art plan
(`NEW_ART_PLAN.md`) is built and released (playtest-2026-10-26-sim146-newart), so this review scores the stages after it to find what
still blocks 5. The method is the same as review 10: eight categories scored 1 to 5 (5 = a still could sit beside the references),
review 3's regions, the same references, the fighters' readability measured. The judge is the hand that built the work, so read the
scores as "the gaps this review can see".

**Evidence:** art/stage-refine aa84c126 at the match camera and wide (`review11/fidelity11_sheet.jpg`: each row a stage, match camera
left, wide right: meadow, grove, beach, canyon, city), and close crops of the weakest spots (`review11/review11_crops.jpg`: the canyon's
front, the grove's abyss, the meadow's left lane, the beach's near sea, the city's pavement, the meadow's right lane).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** | 4.0 |
| Canyon | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| City | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** | 4.0 |
| Grove | 4 | 4 | 4 | 4 | 4 | 5 | 4 | 4 | **4.1** | 4.0 |
| Meadow | 4 | 4 | 4 | 5 | 4 | 5 | 4 | 4 | **4.25** | 4.0 |
| **Mean** | 4.0 | 4.0 | 4.0 | 4.2 | 4.0 | 4.8 | 4.0 | 4.0 | **4.13** | 4.0 |

- **World and life is the first column at 5 on four stages.** The meadow's farm (the windmill, smoke, the cart, the scarecrow and
  washing), the grove's lit village above the pines with its falls and fireflies, the beach's working quay and harbour, and the city's
  traffic, crossing, party and rooftops each read as a place in use. The canyon's railway, station and train are there but small and
  far; it stays a 4.
- **The meadow's background land is a 5:** the terraced fields joined to the valley's farmland make its lower third its richest part.
- **Readability holds.** P1's value gap to the ground with the standard box (0.40 to 0.50 across, 0.46 to 0.68 down): meadow 0.012 to
  0.017 (the same as before its world change, 0.017; review 10's 0.042 was measured with the meadow's own box), grove 0.060, beach 0.181,
  canyon 0.043, city 0.131. The meadow stays the narrowest, carried by the fighters' edge light and hue.

## What still blocks 5

| Gap | What it is | Where |
|---|---|---|
| **The canyon's front** | A third of the wide frame, and still the plainest: long pale beds, each block its own length, so it reads as stacked boards, not a mass of rock. Real sandstone cliffs are cut by vertical joints that run down through many beds, and darken toward their foot. | canyon: play area and front, background land |
| **The meadow's lanes** | The lanes down the terraces are cream-white ribbons with a green centre line, the brightest thing in the lower third, and broken where they cross each terrace wall. | meadow: cohesion |
| **The beach's near sea** | The lower third is broad flat turquoise facets with pale streaks; the shallows read only under the dock. | beach: background land, surfaces |
| **The grove's abyss** | Flat purple with soft blotches under the falls; the mist sheet's edge is one wavy line. | grove: background land |
| **Painted variation** | Still the surfaces and light columns' gap: most surfaces are one colour times a grey print. It worked on the meadow's and grove's play areas; the canyon's sand and the fronts did not take it. | every stage |
| **The fighters** | Half of every still; the character and animation sessions' work. | all |

## The order

1. **The canyon's front as jointed rock:** vertical joints through several beds, each column a little in or out, the face darker toward
   its foot (`make_canyon_front.py --mass A|B`). The biggest weak area left.
2. **The meadow's lanes:** the lane's colour from cream to the farm track's dust, joined across the terrace walls.
3. **The beach's near sea:** broad value drift in the water (deeper and cooler toward the frame's foot, the facets' pale streaks
   softened), so the lower third has depth.
4. **The grove's abyss:** depth in the purple, darker under the falls and the island, the mist edge broken into two or three sheets.

All four refine what exists. None adds new things, so none needs a new-art approval.

## Item 1, the canyon's front as jointed rock (mock, 2026-10-26)

`make_canyon_front.py --mass A|B` on the shipped front (`--variant A --cut 1 --courses 1 --straight C`): the face's own vertical joints,
5 to 11 apart, shared by every bed from top to bottom (one bed in seven bridges one), each column set in or out as a whole, each tier
one run that the joints break, and every vertex's tone a step deeper toward the foot (`darken_down`, to 0.78 at the floor); B also
stains some columns darker. A first try drew the joints per tier, 2.4 to 5 apart, and read as a brick wall. Measured: the front's
contrast 0.067 to 0.090 at the match camera (0.091 to 0.110 wide), value 0.39 to 0.37; P1's gap 0.046 today, 0.042 A, 0.043 B (noise).
On the Stage Look Review ("canyonmass21", `cm21_*`); recommended A. Shipping means a canyon re-light.

Built A (the owner's pick): `godot/models/stages/canyon_front.glb` rebuilt with `--mass A`, and the canyon re-lit (front, paper, play;
`--curve 4.0 --reach 4.5 --floor 0.28` after a fresh `--light-export`). Measured in the game: the front's value 0.37 at the match camera
(contrast 0.066 there, where the top two beds fill the region) and 0.41 wide (contrast 0.091 to 0.120); P1's gap 0.046. Before and
after (wide): `review11/canyon_front_built.jpg`. All seven tests pass (the runner's known flake once, then a pass).

## Item 2, the meadow's lanes (mock, 2026-10-26)

`StageBake._ground_colours`: the palette's [backdrop] `lane_look` or `--lane=A|B|C` sets the lane family (`lane`, `card_lane`), today
cream (0.95, 0.91, 0.74). A track dust (0.84, 0.74, 0.54), B trodden earth (0.74, 0.62, 0.44), C the farmyard's earth (0.62, 0.5,
0.34). The right lane's value at the wide camera: 0.84 today, 0.79 A, 0.74 B, 0.67 C (the fields round it 0.53). P1's gap 0.012 today,
0.014 A and B, 0.018 C. The lanes stay broken at each terrace wall (joining them is a land-model change). On the Stage Look Review
("lanes22", `ln22_*`); recommended C. To ship: meadow.cfg [backdrop] `lane_look="C"` (code colour, no re-light).

Built C (the owner's pick): meadow.cfg [backdrop] `lane_look="C"`. In the game the right lane measures value 0.67 at the wide camera.
Before and after (wide): `review11/meadow_lanes_built.jpg`. All seven tests pass.

## Item 3, the beach's near sea (mock, 2026-10-26)

`water.gdshader` `near_depth` [how much the water deepens and cools toward the frame's foot, its broad slow patches there], from the
palette's [backdrop] `sea_near` or `--sea-near=A|B|C` (A [0.8, 0], B [0.8, 1], C [1.25, 1]); with it the paper bands' pale lips soften
toward the foot. By the frame's height (`SCREEN_UV.y`, from half way down): in front of the stage the water lies within a few units of
the stage plane, so a fade by world depth reached only the frame's last rows. The finishing pass's tone curve takes back most of the
change (at 0.6 the foot moved 0.03), hence the strong settings. The foot's value at the wide camera: 0.56 today, 0.50 A, 0.49 B, 0.44
C; P1's gap 0.181 today, 0.182 A, 0.179 B, 0.176 C. On the Stage Look Review ("nearsea23", `ns23_*`); recommended C. To ship: ocean.cfg
[backdrop] `sea_near=[1.25, 1.0]` (shader, no re-light).

Built C (the owner's pick): ocean.cfg [backdrop] `sea_near=[1.25, 1.0]`. In the game the foot measures value 0.44 at the wide camera.
Before and after (the lower half, wide): `review11/beach_near_sea_built.jpg`. All seven tests pass.
