# New art toward 5 (2026-10-25): options for the owner to approve

The owner, after review 10: "Plan what new art (redesigned lower thirds, more world detail) would take the stages to 5, as options for
me to approve." Review 10 found every stage a clear 4 in every column, and the levers within the standing refine-not-add rule now move
the measurements by a hundredth or two. This plan lists the new art that could lift each stage, as options. **Nothing here is built
until the owner approves it**; each approved option then follows the usual round (two or three mocks at both cameras, the owner picks,
build, re-light, test, commit).

## The rules every option keeps

- **No animals** in the backgrounds, and world details **far back, at true size, through the air** (never upscaled to read).
- **Land connected and believable**: nothing floating that shouldn't, every tree seated, the stage's background trees one large kind.
- **The fight stays clear**: nothing new in the screen area round the play area and platforms; P1's value gap to the ground is measured
  on every mock and must not narrow.
- **One stage at a time**, piloted, then the next with its own reference.
- **The same make**: Blender scripts in `art/blender/` in the paper style (cut card, folded and rolled paper, the hatched print), each
  piece lit by the light bake. No generated models unless the owner asks per piece.
- **Budget**: each stage's new pieces within about 60,000 triangles, frame time checked on the Fast tier.

## Where each stage loses its 5

At the wide camera the lower third of every frame is the plainest part: the meadow's bare green hill, the grove's flat purple abyss, the
canyon's open sand, the beach's empty water under the dock, the city's flat street front. The middle distance has its big shapes (the
windmill, the town, the buttes, the forest) but little life between them. Those two gaps are what the options target.

## The options, by stage

### Meadow: the hill under the wall

| Option | What it is | Effort | Risk |
|---|---|---|---|
| **M1 Terraced fields** | The hill under the wall stepped into two or three farm terraces held by low fieldstone walls (the wall's own stone), with crop rows and a lane switchbacking down, joining the valley's farmland. | 1 session | Low: it continues what the valley already has |
| M2 Stream and footbridge | A small stream from a spring under the wall, down the hill in a cut paper channel, a plank footbridge, wet stones. | 1 session | Medium: water at the bottom of the frame draws the eye |
| M3 Wooded foot | The hill's lower slope breaking into a belt of the meadow's own pines and boulders, so the frame's bottom has depth layers. | 0.5 session | Low, but it adds more of what is already behind |

**Recommended: M1.** It turns the plainest third into the meadow's most characteristic thing, the farmland, at the right scale.

### Grove: the abyss

| Option | What it is | Effort | Risk |
|---|---|---|---|
| **G1 Falls into the mist** | Two or three waterfalls pouring off the terraces' lips into the abyss, their spray lifting into the mist, and the mist rebuilt as three layered cut sheets that thicken with depth. | 1.5 sessions | Medium: falls move, kept slow and calm |
| G2 The valley below | The abyss opened to a deep valley floor far below (a river, the forest's giant trees seen from above at true size) through two mist layers. | 2 sessions | High: a lot of new land; must read as far below, not as trees in water (review 9's lesson) |
| G3 Hanging roots and stones | The neighbour terraces' undersides grown long: roots and hanging stone draping into the mist, as the island's own. | 1 session | Low: it extends what is there |

**Recommended: G1, with G3's longer roots.** Falling water gives the abyss depth and life without new land.

### Canyon: the floor

| Option | What it is | Effort | Risk |
|---|---|---|---|
| **C1 The dry river gorge** | A dry riverbed cut across the floor in front of the mesa: stepped banks in the sandstone beds, a pebble bed winding through, the wash's existing line made into a real channel. | 1.5 sessions | Medium: a big new land form |
| C2 Hoodoos and boulder fields | Clusters of wind-carved hoodoos and fallen boulders on the floor at true size, their long sunset shadows. | 1 session | Low |
| C3 The mesa's foot | The stage mesa's base given a talus fan of fallen blocks and scree spreading onto the floor, so the mesa stands on the ground instead of on a flat. | 0.5 session | Low |

**Recommended: C3 with C2.** They ground the mesa and give the floor forms and shadows; C1 if the owner wants the boldest change.

### Beach: the water under the dock

| Option | What it is | Effort | Risk |
|---|---|---|---|
| **B1 The shallows** | The water under and around the dock turned to shallows: sand bars and rocks seen through clear water, the sea's colour deepening past them, the dock's posts standing in it. | 1 session | Low: a colour and form change on the existing sea |
| B2 Moored boats | Two rowboats tied to the dock's posts, rising gently on the swell. | 0.5 session | Medium: new things near the fight; kept low and to the sides |

**Recommended: B1.**

### City: the street front

| Option | What it is | Effort | Risk |
|---|---|---|---|
| **Ci1 Storefronts with depth** | The ground floor along the street rebuilt as recessed shopfronts: lit window displays, doorways, awnings at different heights, signs in relief. | 1.5 sessions | Low |
| Ci2 Street-level life | A newsstand, a hot-food cart and a bus stop on the pavements, at true size. | 1 session | Medium: clutter at the frame's bottom |

**Recommended: Ci1.**

## World detail, the middle distance (per stage, far back and at true size)

| Stage | Option | What it is |
|---|---|---|
| Meadow | W-M | A farmyard's working life far off: a cart on the lane, washing on a line, smoke from a bonfire in a field. |
| Grove | W-G | Far treehouses and rope bridges in the giant trees, lanterns lit, deep in the haze. |
| Canyon | W-C | A water tower and a small depot by the far railway, a dust trail on a road. |
| Beach | W-B | The harbour town's quay life: stalls, nets drying, a crane. |
| City | W-Ci | Rooftop life in the middle towers: water tanks, lit penthouse windows, a rooftop garden. |

These are small, far and slow. They lift "world and life", and the owner rejected the first versions of some of this
(near and upscaled), so each is at true size.

## Suggested order

1. **The grove's falls (G1 + G3)** and **the canyon's mesa foot (C3 + C2)**: the two plainest lower thirds.
2. **The meadow's terraced fields (M1)**.
3. **The beach's shallows (B1)** and **the city's storefronts (Ci1)**.
4. **World detail** stage by stage, after the lower thirds.

About 8 to 10 sessions for the recommended set. The fighters, half of every still, stay with the character and animation sessions.

## Approved (2026-10-25)

The owner approved the recommended set: M1, G1 + G3, C3 + C2, B1, Ci1, then world detail, starting with the grove's falls and the
canyon's mesa foot.

### Step 1, the grove's falls and the canyon's mesa foot (mock)

- **Grove** (`make_grove_paper.py`, on the shipped `--variant H --rock shaped --abyss mist --valley none`): `--falls N` (`falls`: edges
  over the open chasm facing the camera, 15 to 32 units from the middle, 12 apart; each fall cut strips in `fall` and `fall_shade`
  arcing out from the lip to the mist, `spray` puffs at its foot), `--mist 3` (three layers, `abyss_mist`, `abyss_mist2`, `abyss_mist3`,
  paler with depth), `--roots long` (the lips' roots 2.4 times as long, the same random draws). A `--falls 2`, B `--falls 3`. A first
  placement at about 40 from the middle fell off the wide frame. P1's gap 0.059 today, 0.064 (A), 0.062 (B). About 136,000 triangles
  (135,000 today).
- **Canyon** (`make_canyon_paper.py`, on the shipped KC): `--foot talus` (`mesa_foot`: either side of the stage mesa, a `scree` sheet
  and twelve fallen strata blocks, the biggest against the wall) and `--hoodoos N` (`hoodoos`: clusters of two to four columns of
  stacked strata narrowing to a pale band and a cap stone, 4 to 9 units, boulders and shade round them, 22 to 58 either side, off the
  wash, the arch and the buttes' feet). A three clusters, B five. P1's gap 0.036 today, 0.039 (A), 0.038 (B). 23,700 and 24,800
  triangles.

On the Stage Look Review ("falls12", `fl12_*`; "mesafoot12", `mf12_*`); recommended B on both.

Built B on both (the owner's picks): grove_paper.glb `--variant H --rock shaped --abyss mist --valley none --falls 3 --mist 3 --roots
long` (136,166 triangles; falls at x 24.5, 22.3 and -21.7), canyon_paper.glb `... --plain outcrops --foot talus --hoodoos 5` (24,760);
both stages re-lit (curve 4, reach 4.5, floor 0.28). P1's gap: grove 0.062, canyon 0.038. Before and after (wide):
`review10/falls_mesafoot_built.jpg`. Tests pass except the runner's known coin flake (it passed on a rerun; the runner doesn't load these
stages).

### Step 2, the meadow's terraced fields (mock)

`make_meadow_ground.py --hill fields|fields_lane` (with the shipped `--variant FC --pine D`; `terrace_fields`): the hill's terraces,
fewer and bigger as `--hill banks`, each shelf laid in runs of the valley's field kinds (`FIELD_KINDS`, rows on the crops) and held by a
two-course fieldstone wall (`terrace_wall`, `terrace_wall_b`, the second course offset half a block) down to the next; `fields_lane` a
lane along each shelf from the front's middle out to alternate sides, with a ramp down to the next. A first build left gaps in the second
course and set blocks in and out, so every block drew the post's cut edge; both fixed. The hill at the wide camera: value 0.39 to 0.62,
saturation 0.47 to 0.30 (grass to fields). P1's gap 0.052 today, 0.051 with either. On the Stage Look Review ("terraces13", `tf13_*`);
recommended B.

Built B (the owner's pick): meadow_ground.glb and meadow_pines.glb `--variant FC --pine D --hill fields_lane` (249,665 triangles), the
meadow re-lit (front, play, ground). P1's gap 0.052. Before and after (wide, then the match camera): `review10/meadow_terraces_built.jpg`.
All seven tests pass.

### Step 3, the beach's shallows (mock)

`water.gdshader` `shallows` [how strong, reach, rocks] in the clear and paper seas (`look` above 0), set by `StageArt._ocean` from the
palette's [backdrop] `sea_shallows` (`--shallows=A|B`, A [1, 40, 0], B [1, 40, 1]): long soft sand bars of pale sandy turquoise in
world-space noise, a slightly deeper rim round each, fading out from half the reach; B a few rocks on the bottom (a first, denser and
darker set read as blotches). Shader only, no cost to draw. The sea at the wide camera: contrast 0.084 to 0.099 (A) and 0.100 (B); P1's
gap 0.176 today, 0.168 (A), 0.172 (B). On the Stage Look Review ("shallows14", `sw14_*`); recommended A.
