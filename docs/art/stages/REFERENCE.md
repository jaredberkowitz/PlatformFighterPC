# Stage refinement: Phase 1 audit (2026-10-20)

The first task of `docs/STAGE_REFINEMENT_AGENT.md`: the baseline, a measured comparison with the owner's references, the ranked
weaknesses and a proposed order. Nothing in the game was changed for it.

## Setup

- Branch `art/stage-refine` in the worktree `.claude/worktrees/stage-refine`, cut from the tip of `art/cloth-cleanup` (818797e). The
  character PR is still open; its fighters (matte, bare legs) are what the stages must read against, so the stage work starts on top of
  it and rebases onto `main` once that PR merges.
- References (owner, 2026-10-20) copied under neutral names into `docs/art/benchmarks/`: `meadow_camp_reference.jpg`,
  `grove_reference.webp`, `harbour_reference.webp`, `canyon_reference.webp`, `city_reference.webp`. The meadow also keeps
  `papercraft_mountains_reference.png`. Ours beside each: `reference_pairs.png` (this folder; rows meadow, grove, beach, canyon, city).

## Baseline: round 44

- Match camera, every stage: `docs/gauntlet/round44/` (`stage<k>.png`, `grey<k>.png`, `contact_sheet.png`, before/after against 43:
  unchanged apart from the new fighters).
- Fighters in play (`--demo=moves`, the forward smash frame): `docs/gauntlet/round44/play/` with its own contact sheet.
- Look bench: `docs/gauntlet/round44/bench/look_bench.png`. **Fails** on one cell, as expected: the night/city mood's dark backdrop with
  the midnight body, edge 0.39 (needs 0.85). Every other cell passes (edge 0.88 to 1.00).

## Measured table

Screen RGB averages over a hand-placed region per layer (scratch `measure.gd`; value = the greyscale's luma, contrast = its standard
deviation in the region, a stand-in for texture busyness). Regions were checked on overlays. References measured the same way.
Band = `STYLE_SHEET.md` section 1. Values out of band in **bold**.

### Meadow (stage 0) vs the papercraft camp

| Layer | Ours: value (band) | Ours: hue, sat | Reference: value, hue, sat |
|---|---|---|---|
| Sky | **0.71** (0.75-0.95) | 191, 0.33 | 0.58, 197, 0.64 |
| Distance (ranges) | 0.53, **0.18 under the sky** (within 0.15) | 189, 0.47 | lit 0.42 / shade 0.39, 182, 0.48 |
| Mid-ground (valley) | 0.63 (0.45-0.70) | 100, 0.38 | pines 0.29, 166, 0.51 |
| Play surface (lawn) | 0.64 (0.45-0.72) | 91, 0.63 | lime 0.63, 88, 0.52 |
| Path | **0.80** | 74, 0.35 | cream 0.88, 61, 0.32 |
| Stage front (cliff) | **0.17** (0.35-0.55) | 182, 0.31 | ledge face 0.25 (`STAGE_REFERENCES.md`), a thin strip |
| Fighters | 0.59 / 0.59: **0.05 from the lawn** | | |

### Grove (stage 4) vs the floating-island grove

| Layer | Ours | Ours: hue, sat | Reference |
|---|---|---|---|
| Sky | **0.73** | 281, 0.16 | canopy 0.17, 335, 0.62 (no sky) |
| Distance (hills) | 0.48, **0.25 under the sky** | 265, 0.36 | violet forest 0.32, 266, 0.50 |
| Mid-ground (plain / side crown) | 0.57 / 0.49 | 151, 0.37 / 139, 0.31 | side islands 0.28, rock 0.20 |
| Play surface (lawn) | 0.56-0.62 | 105-110, 0.43-0.45 | 0.38, 69, 0.64 |
| Stage front (lip / keel) | 0.53 / 0.48 | 114, 0.48 / 13, 0.42 | rock and roots 0.22, 33, 0.18 |
| Great trunk | 0.44 | 6, 0.40 | 0.28, 32, 0.11 |
| Fighters | 0.55 / 0.56: **0.00 to 0.07 from the lawn** | | |

### Beach (stage 2) vs the harbour

| Layer | Ours | Ours: hue, sat | Reference |
|---|---|---|---|
| Sky | 0.76 | 192, 0.22 | 0.42, 192, 0.98 (deep clear blue) |
| Distance (headland) | 0.58, **0.18 under the sky** | 185, 0.44 | 0.68, 183, 0.33 |
| Mid-ground: town / sea | 0.64 / 0.57 | 178, **0.05** / 202, 0.55 | 0.64, 167, 0.32 / 0.48, 188, 0.71 |
| Play surface (deck) | **0.76** | 41-42, **0.27-0.30** | 0.55, 29, 0.60 (honey) |
| Stage front (posts and water) | **0.33**, contrast 0.176; posts alone 0.16 | 205, 0.17 | beam and posts 0.48, 26, 0.60 |
| Fighters | 0.64 / 0.57 | 33-56 | |

### Canyon (stage 1, held) vs the hazy desert

| Layer | Ours | Ours: hue, sat | Reference |
|---|---|---|---|
| Sky | **0.63** | 11, 0.44 | 0.65, 252, 0.30 (cool lilac) |
| Distance (dunes) | 0.74 (lighter than the sky) | 24, 0.38 | far ranges 0.71, 251, 0.22 |
| Mid-ground (mesa) | 0.50 | 356, 0.44 | butte 0.52, 323, 0.32; dunes 0.51, 348, 0.36 |
| Play surface (sand) | 0.60-0.65 | 20-25, 0.53-0.55 | near sand 0.59, 22, 0.59 |
| Stage front (strata) | **0.32** | 349, 0.54 | |
| Fighters | 0.52 / 0.50: 0.08 to 0.15 from the sand, **the same hue** (19-28) | | |

### City (stage 3, night) vs the night floating city

| Layer | Ours | Ours: hue, sat | Reference |
|---|---|---|---|
| Sky | 0.43 (night) | 324, **0.68** | 0.25, 286, 0.36 |
| Distance (pale far towers) | 0.30 | 257, 0.52 | far skyline 0.25, 305, 0.37 |
| Mid-ground (navy towers) | **0.22**, contrast 0.127 | 259, 0.51 | city mass 0.26, 306, 0.24 |
| Street | 0.34 | 276, 0.50 | |
| Play surface (roof) | **0.40-0.42** | 335-337, 0.48-0.50 | platforms 0.30 (pale slabs brightest of all) |
| Stage front (facade) | 0.41, contrast **0.137** (the roof 0.08) | 310, 0.27 | island rock 0.29 |
| Signs (OPEN, SHOP) | **0.54-0.55, above both fighters** | 300, 0.34 | |
| Fighters | 0.52 / 0.47 | | |

### Across the stages

| Measure | Meadow | Grove | Beach | Canyon | City |
|---|---|---|---|---|---|
| Sky value | 0.71 | 0.73 | 0.76 | 0.63 | 0.43 (night) |
| Sky minus distance | 0.18 | 0.25 | 0.18 | -0.11 | 0.13 |
| Distance minus mid-ground (positive: the distance lighter, as it should be) | **-0.10** | **-0.09** | **-0.06** | +0.24 | +0.08 |
| Play surface | 0.64 | 0.59 | 0.76 | 0.62 | 0.41 |
| Stage front | 0.17 | 0.48-0.53 | 0.33 | 0.32 | 0.41 |
| Front contrast vs top contrast | 0.02 / 0.11 | 0.09 / 0.08 | 0.18 / 0.09 | 0.08 / 0.08 | 0.14 / 0.08 |
| Look bench, worst edge (dark backdrop) | 1.00 | 1.00 | 1.00 | 1.00 | **0.39** |

Materials in view are counted from the code, not the image: every stage shares the cel shader family and the rendered tiles; the
outliers are listed below. The stage front is the least consistent layer: 0.17 to 0.53 across five stages that should share one band.

### Frame time (match camera, 10 s, `--perf`, this machine)

| Stage | Avg | p99 | Worst | Draw calls | Primitives | GPU |
|---|---|---|---|---|---|---|
| Meadow | 3.70 ms | 16.7 | 80.8 | 853 | 1.27 M | 2.20 ms |
| Canyon | 3.42 ms | 16.7 | 56.1 | 886 | 0.46 M | 2.23 ms |
| Beach | 4.01 ms | 6.0 | 142.7 | 921 | 0.90 M | 2.64 ms |
| City | 3.91 ms | 16.7 | 89.7 | 880 | 1.76 M | 2.78 ms |
| Grove | 3.44 ms | 18.2 | 54.8 | 716 | 0.77 M | 2.32 ms |

All far inside a 16.7 ms frame; the worst frames are the load-time hitches. Not checked: another GPU or the lowest graphics quality.

## Ranked weaknesses

**Confirmed** (measured and visible at match distance):

1. **City: the night backdrop swallows dark fighters, and the sky is the loudest thing in the frame.** The look bench fails (midnight body,
   edge 0.39); the navy tower layer is the darkest large area (0.22); the sky is a hot magenta (sat 0.68, hue 324) against blue-violet
   towers (259), where the reference's night is one muted violet family (sat 0.24 to 0.37, hue 286 to 306); the OPEN and SHOP signs
   (0.55) are brighter than both fighters. The reference also makes its play pieces the palest things in view; our roof (0.41) sits
   between the towers and the signs.
   *Cause:* `godot/stages/palettes/city.cfg` (`skyline_shades` darkest 0.17, 0.14, 0.32; `sky_glow` magenta at 0.45; `[air]` lift 0.2),
   `stage_art.gd` `_skyline_layer`, the signs in `facade.gdshader` `neon_sign`, the city mood's edge light in `lighting.gd` (0.55).
   *Fix:* step the tower layers lighter and toward one violet horizon by depth (the `[air]` curve), take the sky's saturation down and
   its hue toward violet, dim the signs under the fighters, lift the roof a step. Keep the night.
   *Risk:* the run mode is built from the city's art (check `run_shot`); the bench's dark backdrop is synthetic (ambient darkened), so the
   mood's edge light may need a small lift too, which touches how fighters are lit on the city only.
2. **Meadow: the cliff under the lawn is a near-black slab across a quarter of the frame.** Value 0.17, the darkest large area of all five
   stages, against a band of 0.35 to 0.55; it reads as a hole, not earth.
   *Cause:* `meadow.cfg` `ledge = (0.2, 0.3, 0.25)`, copied from the reference's ledge face, which there is a thin strip; on our stage it
   is a 250-pixel wall, and `_faceted_ledge`'s fall into shade takes it darker and cooler (hue 182). The style sheet disagrees with
   itself here: section 1's band against section 9's rule 4 (a dark, simple front).
   *Fix:* mock three values (about 0.25, 0.32, 0.40 at the top, falling a step lower) for the owner. *Risk:* low, one palette key.
3. **Depth: the distance is too far under the sky, and the skies sit under their band** (backlog item 1). Sky minus distance 0.18, 0.25
   and 0.18 on the meadow, grove and beach (rule: within 0.15); skies 0.71 to 0.76 (band 0.75 to 0.95). Worse, on those three the far
   ranges are *darker* than the mid-ground in front of them (by 0.06 to 0.10): the depth step runs backwards. On the grove the mid-ground
   plain (0.57) equals the lawn, so the island does not separate from what is behind it.
   *Cause:* the `[air]` curve (`StageArt._air`) only lifts what is past `start` by `lift` 0.15 to 0.18, and the ranges' and land bands'
   own palette colours are dark (`hills_far`, the ridge colours). *Fix:* one shared change: the curve lifts toward each sky's horizon
   colour and reaches the ranges and land bands, then each palette's far colours are re-measured. *Risk:* touches every stage at once;
   check the contact sheet and greyscales together.
4. **Beach: the wood is bleached and the posts nearly black.** The deck is 0.76 with sat 0.27 (the reference's honey: 0.55, sat 0.60);
   the posts are 0.16; the town is grey (sat 0.05; the reference's pastel town 0.32).
   *Cause:* `_dock_block` hard-codes the deck colour (0.86, 0.7, 0.5) and lifts the play line by `worn_tint` 1.6; the town is greyed by
   `[air]` desat 0.22 plus its own pale walls. The history (four moves of the deck colour in its comments) shows the tension: a honey
   deck at the reference's value sits on the fighters' value (0.57 to 0.64) and hue (33 to 56).
   *Fix:* mock options: a deeper honey (about 0.48, sat 0.5) under the fighters; today's value with sat about 0.45; and posts lifted to a
   wood brown about 0.30. The town takes back colour. *Risk:* the yellow fighter's separation; measure each option.
5. **Grove: the fighters stand on their own value.** The lawn under them is 0.56 to 0.62, the fighters 0.55 to 0.56 (rule: 0.05 apart);
   they read by outline and hue only. The reference's lawn is a yellow-green (hue 69) well clear of everything round it.
   *Cause:* `grove.cfg` grass colours. *Fix:* the lawn a step lighter and yellower (with item 3 making the plain behind it recede).
6. **The canyon is one warm hue from sky to front** (held: scored, not ranked). Sky 11, distance 24, mesa 356, sand 20 to 25 and the
   fighters 19 to 28: no cool counterweight, and the orange fighter stands on orange sand separated by value alone. The new reference
   keeps warm sand under a cool lilac sky and distance (hue 251). Owner's call (see the questions).
7. **Stage fronts busier than the tops.** The beach's posts against bright water (contrast 0.18 vs the deck 0.09) and the city's lit
   window grid (0.14 vs the roof 0.08). The sheet wants the front calmer. Partly fixed by items 1 and 4 (dimmer windows, lighter posts).

**Seen but not measured** (check after the systemic passes):

8. The grove's side crowns still read as stacked low-poly boxes (backlog item 3), and its great trunk is a smooth warm red-brown (sat
   0.40) where the reference's is grey-brown (sat 0.11) in ropey ridges.
9. No lit edge against the sky on the meadow's ranges and the grove's crowns (backlog item 2): visible, not measured.
10. The beach's lamp post is thin (backlog item 6) and the canyon's floating platforms are thin dark slabs; both small at match
    distance.

### The backlog's eight items against the new captures

| # | Item | Now |
|---|---|---|
| 1 | Distance in steps | Confirmed, weakness 3 |
| 2 | Light edges against the sky | Visible, not measured (9) |
| 3 | Grove crowns | Confirmed by eye (8) |
| 4 | City tower layers, street glow, awnings | Confirmed: towers 0.22 against a far layer of 0.30, a step of only 0.08 (1) |
| 5 | Canyon long shadows | Held |
| 6 | Beach lantern | Still thin; small at match distance (10) |
| 7 | Grove turf tufts | The lip reads smooth and rolled; low priority |
| 8 | Meadow platform shade | Soft shade bands already show on the lawn; low priority |

## Proposed order

1. **Systemic depth curve** (weakness 3, backlog 1): the `[air]` curve toward each sky's horizon on ranges and land bands; skies into band.
2. **Stage fronts into one band** (weaknesses 2 and 7): the meadow's cliff after the owner picks a mocked value; the beach's posts.
3. **City** (weakness 1): the look bench back to passing, towers stepped, sky calmed, signs under the fighters. Then the run mode check.
4. **Beach wood** (weakness 4): mocked honey options, the owner picks.
5. **Grove** (weakness 5, then 8): the lawn off the fighters' value; then crowns and trunk.
6. **Lit edges against the sky** (backlog 2): a scenery-only rim, gated by material.
7. **Canyon** (weakness 6): cooler-dusk mocks, refinements only (the owner likes it as it is).
8. Details last: the lantern, platform slabs, motion.

## The owner's decisions (2026-10-20)

- **Order approved** as above: the shared depth curve first.
- **Canyon:** the owner likes it as it is and wants it refined by learning from other sources, not replaced. `canyon_reference.webp`
  (the hazy desert) is a newer pick than the earlier description. Mock 2-3 options that keep the dusk, the sun and the look but bring
  in a cooler lilac upper sky and distance; the owner picks one or none.
- **Grove:** keep the bright day and the quilted bank; borrow only the depth lesson (darker, simpler surroundings so the island pops)
  and a yellower lawn.
- **City:** stays at night (the new reference is a night city too).
