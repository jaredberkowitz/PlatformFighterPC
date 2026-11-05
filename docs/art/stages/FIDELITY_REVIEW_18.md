# Stage fidelity review 18 (2026-11-05): after the style pass

The owner, after the style pass (`STYLE_PASS.md`) was complete: "Run review 18". Every stage is judged as built on art/stage-refine
b91195d4, with all eight style pass steps in:
- stepped paper skies;
- the canyon's torn paper;
- the city's halftone print;
- the beach's origami sea;
- the meadow's stitch;
- the grove's washi;
- scenery on twos;
- each stage's paper on its cut edges.

The same eight categories as reviews 10 to 17 (5 = the still could sit beside the references), judged at the player's 1920x1080.

**Evidence:**
- `review18/fidelity18_sheet.jpg`: every stage as built (rows meadow, grove, beach, canyon, city; match camera left, wide right).
- `review18/grey18_sheet.jpg`: the match camera in grey, for the readability check (meadow, grove; beach, canyon; city).
- `review18/review18_crops.jpg`: the two weakest visuals at full size. The grove's forest behind the island in colour and in grey,
  and the beach's deck before the cut-edge colours and as built.

## What the style pass changed

Each stage now has its own craft. At the match camera:

| Stage | Its craft | At the match camera |
|---|---|---|
| Meadow | The stitch on the lawn's card lip | A fine dashed line, the quietest of the five. The patchwork wasn't taken (the owner's pick A). |
| Canyon | Torn paper along the strata | Reads at once |
| Beach | The origami sea | Reads at once |
| City | Halftone print in the shade | On the pale towers' shade, at full size |
| Grove | Washi lanterns, tissue mist | The lanterns read; the mist shows at the wide camera |

Across every stage: the stepped paper skies, the stage's own paper on its cut edges, and the scenery at 15 pictures a second. The
last is judged in motion, not in these stills.

## Measurements

At the match camera unless noted, review 17 (after the pass and the grove's distance) and now:

| Stage | Front value (contrast) | Floor value | Darks, wide | P1's gap | P1's head against the floor (Delta E) |
|---|---|---|---|---|---|
| Meadow | 0.55 (0.140) to 0.56 (0.141) | 0.60 to 0.60 | 4% to 3% | 0.107 to 0.108 | 39 to 39 |
| Canyon | 0.56 (0.120) to 0.57 (0.128) | 0.66 to 0.66 | 3% to 3% | 0.159 to 0.154 | 22 to 21 |
| Beach | 0.36 (0.090) to 0.37 (0.091) | 0.48 to 0.49 | 3% to 3% | 0.167 to 0.155 | 30 to 32 |
| City | 0.27 (0.168) to 0.27 (0.167) | 0.52 to 0.52 | 36% to 37% (night) | 0.150 to 0.149 | 29 to 27 |
| Grove | 0.48 (0.117) to 0.48 (0.114) | 0.60 to 0.60 | 7% to 7% | 0.096 to 0.096 | 39 to 39 |

The style pass left the play areas where review 17 had them. Every fighter still stands clear of its floor by value (a gap of 0.12 or
more) or by colour (a Delta E of 30 or more): the meadow and grove by colour, the canyon and city by value, the beach by both.

**New this review, how busy each layer is:** the share of edge pixels (Canny, 0.08/0.2) in the band above the play area and on the play
area's top. The references keep their distance calm and simpler than the ground in front, and every stage but one does too:

| Stage | Background | Play area's top |
|---|---|---|
| Meadow | 2.1% | 5.4% |
| Canyon | 0.8% | 2.1% |
| Beach | 1.6% | **6.4%** (4.4% before the cut-edge colours) |
| City | 3.1% | 4.9% |
| Grove | **4.4%** | 3.8% |

- **The grove's backdrop is the busiest of the five**, busier even than the night city's hundreds of windows, and busier than the
  island's own top. Its forest also sits close to the lawn in value (0.51 to 0.54, against the lawn's 0.61), with more contrast
  between its facets (0.16 to 0.19) than the lawn has (0.13). The island doesn't stand out from it; in the reference, the forest
  recedes darker and simpler, so the island pops.
- **The beach's deck is now the busiest floor.** The cut-edge colours turned the gaps between its planks into pale lines (the
  owner's pick B, which said the beach would change most). The reference's boards each have their own tone and dark gaps; ours are
  long, even strips ruled with light lines.

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|---|---|
| Beach | 5 | 5 | **4** | 5 | 5 | 5 | 5 | 5 | **4.88** |
| Canyon | 5 | 5 | 5 | 5 | 5 | 4 | 5 | 5 | 4.88 |
| City | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5.00 |
| Grove | 5 | 5 | 5 | **4** | 5 | 5 | 5 | 5 | **4.88** |
| Meadow | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5.00 |

Mean **4.93** (review 17: 4.98). Bold: down. Thirty-seven of forty cells are 5.
- **The beach's play area, 5 to 4:** the deck's pale seams, above.
- **The grove's background land, 5 to 4:** the forest hasn't changed since review 17, where I scored it 5. It's measured here for the
  first time, and it's the one backdrop busier than its own play area.
- **The canyon's world and life stays 4** by the rule (no animals, life far back at true size).
- The eight categories measure closeness to the references, not identity. The style pass's gain, each stage in its own craft, doesn't
  show in them.

## The plan (both refine what exists; neither adds anything)

| # | Item | Stage | Lifts | Effort |
|---|---|---|---|---|
| 1 | **The grove's forest into the air.** The forest between the island and the far range takes the violet air from nearer (the palette's [air] `start`, today 80), so it lifts toward the haze and its facets calm, and the island stands in front. It lifts toward the haze rather than greying: in 2026-10-22's retune, the air from 40 greyed the middle distance to a saturation of 0.18 to 0.29. Options: the air from 40; from 40 with twice the lift; from 20 with twice the lift. | grove | background land | 0.5 session |
| 2 | **The beach's deck boards.** Each board in its own tone, as the reference's are (the deck's two plank tones further apart and spread unevenly), and the gaps read as joints rather than ruled lines. Options: the gaps back toward the wood (the cut-edge paper on the deck's front edge only); the boards in their own tones; both. | beach | play area and front | 0.5 session |

**Honest ceiling:**
- With both, the mean is back to 4.98: 39 of 40 cells at 5, with the canyon's world and life at 4 by the rule.
- The fighters stay outside this plan, and they remain the largest gap in any still.

**Recommended start:** item 1. The grove's backdrop affects the whole frame, and the fix is in the palette.

**Also noted, not planned:**
- The meadow's front stacks five materials under the lawn: the card lip, the earth, the leaf fringe, the logs and the stones. It's
  busy, but the owner picked each layer and it reads.
- The canyon's fighters stand on gold sand with the lowest colour difference of the five (21). They separate by value (0.154).
- The grey check finds nothing under the fight that outshouts the fighters. The brightest new lines are the cut-edge lips, which
  mark the play line, as the references' crisp lit edges do.

## Item 1, the grove's forest into the air (mock, 2026-11-05)

The owner: "Start item 1". Two things made the forest the busiest backdrop:
- **The modelled trees took no air at all** (`StageBake.place_trees`). Every other piece of scenery fades with its depth through the
  palette's [air]; the 21 trees (from 18 to 128 units back, median 48) kept their full colour and ink at any distance.
- **The grove's air starts 80 units back**, behind most of the forest, and lifts only 0.045 by 160.

The mock (`--pal-try=air_a|air_b|air_c`, grove.cfg):
- The trees take the air at their depth, as the rest of the scenery does (palette [air] `trees`; the great tree stands in the fight and is
  left out). In the air they lose their ink, as the style sheet has it for the mid-ground.
- A near band of air lies over everything between the island and the far ranges (palette [air] `near` [start, end, amount],
  `StageArt._air_near`). It only adds where the curve beyond gives less, so the far ranges the owner picked after review 17 stay as they
  were.
- The near air is mixed in as light (`air_mix` in `toon_common.gdshaderinc`, at a lit face's brightness, [air] `near_light`): a share of
  what reaches the eye is the air's, so a tree's lit side and shaded side both settle toward it. A first try that only lifted the trees'
  colour (as the far air does) barely changed them (4.4% to 4.3%): the shaded side is a share of the light, so it stayed as far under the
  lit side.

| Option | The forest | Busy behind (edges) | Its folds (contrast) | Its colour (saturation) |
|---|---|---|---|---|
| Today | Full colour and ink at every depth | 4.4% (the island's top 3.8%) | 0.16 to 0.19 | 0.29 to 0.30 |
| A, into the violet haze | A third into the grove's violet air by 75 units back (near [15, 75, 0.33]) | 4.0% | 0.15 to 0.16 | 0.12 to 0.18 |
| B, its own light green | A third into a light tint of its own colour ([air] `own_hue`) | 3.9% | 0.14 to 0.16 | 0.32 to 0.33 |
| C, its own green, halfway | B at 0.5 | 3.8% | 0.14 to 0.15 | 0.33 to 0.35 |

The island is unchanged: its lawn measures the same, and P1's gap goes from 0.096 to 0.099. At the wide camera the band above the island
goes from 4.6% to 4.3%, 4.2% and 4.0%. On the Stage Look Review ("forestair75", `fa75_*`).

Recommended B: the forest steps back in its own green with calmer folds, the island's lawn now leads, and the backdrop comes down to
about the island's own busyness.
- None of the three takes the backdrop clearly under the island (B 3.9% against 3.7%, C level). What's left is the lanterns, the clouds and
  the side platforms: lights and play pieces, which the air leaves alone.
- A is closest to the reference's violet forest, but it greys the forest (saturation 0.29 to 0.12 to 0.18), the greying October's air
  retune undid. Its trees also go violet while the ground they stand on stays green.
- C starts to wash the forest pale.
- The air as light is new to the toon shader and off by default; only the grove's trial uses it. The world-building pieces, which are
  coloured directly, take the near air as a lighter colour, not as light.
All tests pass but the runner test, on its known coin-pickup flake (PR #15, still open); the look bench passes.

**Built B** (the owner's pick, 2026-11-05):
- grove.cfg [air] `near=[15.0, 75.0, 0.33]`, `near_light=1.0`, `own_hue=true`, `trees=true`. `--pal-try=pre_air` shows the forest before.
- The built grove measures the same as the mock's B: the band above the island at 3.9% edges (the island's top 3.7%), the forest's folds
  0.14 to 0.16 in contrast, its saturation 0.32 to 0.33, P1's gap 0.100.
- Before and after (match camera top, wide bottom, before left): `review18/grove_forest_built.jpg`.

With it the grove's background land is a 5: the forest steps back behind the island, and what's left above it is lights and play pieces.
The mean is 4.95, with 38 of 40 cells at 5. Left: the beach's play area (item 2) and the canyon's world and life (4 by the rule).
All tests pass but the runner test, on its known coin-pickup flake (PR #15, still open); the look bench passes.

## Item 2, the beach's deck boards (mock, 2026-11-05)

The owner: "Start item 2". The deck (beach_play.glb) is seven card boards with narrow gaps between them, each board's top in `top` and
its sides in `top_card`. Since the cut-edge colours, `top_card` takes the bleached paper, and it is both the deck's front edge and every
board's cut side seen through the gaps: the ruled pale lines. Nothing is remodelled: as the play area loads, `StageBake._split_boards`
splits the gaps' faces out of `top_card` (a face toward the camera with another board's face in front of it, over the same span) and
each board's top out of `top` (its own connected sheet). The vertex data and its baked shade are untouched. The palette's [backdrop]
`deck_look`, or `--deck=T|A|B|C` (`StageBake.DECK_TRY`):

| Option | The deck | Busy (edges) | Its contrast | P1's gap | P1's head against the deck (Delta E) |
|---|---|---|---|---|---|
| Today | Pale cut sides in every gap | 6.4% | 0.106 | 0.155 | 32 |
| A, dark joints | The cut sides in the gaps in the deck's wood, 0.35 darker; the paper only on the deck's front edge | 3.6% | 0.076 | 0.171 | 35 |
| B, boards in their own tones | Each board one of four tones (`_board_tone` at 2.0: as it is, darker and redder, lighter and yellower, weathered grey), the gaps as today | 5.9% | 0.100 | 0.129 | 33 |
| C, both | A and B | 4.5% | 0.107 | 0.147 | 30 |

Busy is the share of edge pixels on the play area's top; it was 4.4% before the cut-edge colours. The tones at 1.0 barely showed (about
0.04 to 0.09 in luma, board to board), so B and C are at 2.0. On the Stage Look Review ("deckboards76", `db76_*`).

Recommended A: the deck reads as boards with dark gaps, as the reference's do, and it is the calmest it has been, quieter than before
the cut-edge colours. P1 stands out from it more than today.
- A keeps the bleached paper on the deck's front edge, where it marks the play line.
- B and C's lighter and greyer boards lift the deck on average (value 0.49 to 0.53 and 0.56), and the fighters lose some of their gap
  (0.129 and 0.147). In C the grey boards read a little patchy. Balanced to keep the deck's value, the tones would be another round.
- Only the main deck changes; the side platforms are a separate model.
All seven tests pass (the runner test too, this run); the look bench passes.

**Built A** (the owner's pick, 2026-11-05):
- ocean.cfg [backdrop] `deck_look="A"`. `--deck=T` shows the gaps' pale cut sides before.
- The built beach measures as the mock's A: the play area's top at 3.7% edges (from 6.4%), the deck's contrast 0.083 (from 0.106), P1's
  gap 0.174 (from 0.155). It differs from the mock's capture only by a few pixels of framing as the fighters move.
- Before and after (match camera top, wide bottom, before left): `review18/beach_deck_built.jpg`.

With it the beach's play area is a 5. **Review 18 is done:** the mean is 4.98, with 39 of 40 cells at 5. The one 4 is the canyon's world
and life, held there by the rule (no animals, life far back at true size). The fighters remain the largest gap in any still.
All seven tests pass; the look bench passes.
