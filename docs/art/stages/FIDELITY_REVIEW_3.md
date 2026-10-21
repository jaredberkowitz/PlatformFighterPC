# Stage fidelity review 3 (2026-10-21, after the paper play areas, the light and the paper target)

The owner, after the paper target was built: a third analysis of every stage, re-scored and re-measured, and the systemic ideas
(ones not tied to a single stage) re-ranked. Same six categories, the same 1 to 5 scale (5 = hard to tell apart in quality from the
reference, the owner's bar), the same references, checked against measurements.

**Evidence:** round 72 (`docs/gauntlet/round72/`: every stage at the match camera, greyscales, fighters in play `play/`, wide `wide/`)
compared with round 69, review 2's captures. Page images: `docs/art/stages/review3/` (round 69, round 72, in play, wide and the
reference for each stage, with a local `index.html`).

**What changed since review 2:**
- the paper play areas on every stage;
- the paper finish;
- the readability pass;
- mood light on the city, grove and beach;
- contact shade baked into every Blender piece;
- the paper target from the paint-overs: sheet shadows, cut edges, the lift round the fight, and air on the meadow and beach.

## Scorecard (round 69 → round 72)

| Stage | Lighting | Surfaces | Modelling | Composition | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|
| Meadow | 3 → 4 | 3 → 3 | 4 → 4 | 4 → 4 | 4 → 4 | 4 → 4 | 3.7 → **3.8** |
| Grove | 3 → 4 | 3 → 3 | 4 → 4 | 3 → 4 | 4 → 4 | 4 → 4 | 3.5 → **3.8** |
| Canyon | 3 → 4 | 3 → 3 | 4 → 4 | 3 → 3 | 4 → 4 | 3 → 4 | 3.3 → **3.7** |
| City | 3 → 4 | 3 → 3 | 4 → 4 | 3 → 4 | 3 → 3 | 3 → 3 | 3.2 → **3.5** |
| Beach | 3 → 3 | 3 → 3 | 4 → 4 | 3 → 3 | 4 → 4 | 4 → 4 | 3.5 → **3.5** |

The average went from 3.44 to 3.67.
- **Lighting**, the lowest column in review 2, moved on four stages: the contact shade, the sheet shadows and the mood light did what
  they were for.
- **Surfaces** is now the lowest column on every stage, at 3 everywhere. Up close the paper is still flat colour; the references'
  surfaces are printed and textured.
- **The beach** did not move. Its light change is the smallest: the ripple rings show mostly in the wide view. Its deck and timber are
  still plain.
- **The city's** cohesion and readability stay at 3: a solid brick front under a paper skyline, and the darkest fighter on the dark
  backdrop.

## What the numbers say (round 69 → 72, value / contrast at the match camera)

- **Meadow:**
  - The distance is lighter and softer (0.59 / 0.091 → 0.67 / 0.060), as the paint-over asked.
  - The lawn has more form (contrast 0.087 → 0.122) at almost the same value (0.64 → 0.66).
  - The front is unchanged (0.38).
- **Grove:**
  - The lawn is lighter (0.67 → 0.76) and calmer (saturation 0.57 → 0.30), well above the fighters (about 0.55).
  - The mid-ground is now dark terraces (0.50 → 0.33), so the fight no longer sits on a mid-ground of its own value.
- **Canyon:**
  - The sand is paler and cooler (0.65 → 0.74, saturation 0.53 → 0.38), so the orange fighter no longer stands on sand of its own hue.
  - The butte behind the fight is a step darker (0.55 → 0.52), still close to the fighters.
- **City:**
  - The roof is the play surface and now reads first (0.40 → 0.63 in the audit's region), over a front at 0.30.
  - The towers have more edge contrast (0.12 → 0.16) from the cut edges.
- **Beach:**
  - The distance is softer (0.64 / 0.066 → 0.67 / 0.049) and the deck lighter (0.48 → 0.52).
  - Everything else is within 0.02.

## Weak points, ranked (severity 1 to 5)

| # | Weak point | Severity | Stages | Evidence and cause |
|---|---|---|---|---|
| 1 | **Paper without a printed surface** | 4 | all | Every paper piece is flat colour with a faint grain. Up close (`play/`) the lawn, sand, deck and roof read as plain card, and the fronts' logs, timber and brick as smooth paint. The references' surfaces carry print and texture: wood grain, grass blades, worn edges. This is the Surfaces column, now 3 everywhere. |
| 2 | **Code-built props are the plainest things on the play area** | 4 | all | Since the play areas went to Blender, the props standing on them are the last code-built objects near the fight: hay bales, fence, barrels and signs (meadow); mushrooms, bushes and stump (grove); crates, barrels and lamp (beach); rocks, cacti and crates (canyon); the water tower, crates, scaffolds and pipes (city). They cast the new contact shade but take none themselves, and the cut edge outlines the thinner ones (the fence) like stickers. |
| 3 | **Two languages, near and far** | 3 | meadow, city, beach | Unchanged from review 2: the fronts are solid modelled material (logs and stone, brick, timber) under paper backgrounds. The cut edges and sheet shadows help the far layers read as paper; the fronts still don't. |
| 4 | **The beach's light is flat** | 3 | beach | The one stage that did not move. Midday light with no mood except rings that the match camera mostly crops; no warm bounce under the dock. (Stage-specific.) |
| 5 | **Readability edge cases** | 2 | city, canyon | The city: the darkest body on the dark backdrop, a fighter-side fix open since round 44. The canyon: the butte right behind the fight at 0.52 against fighters near 0.5. Both milder than in review 2 (the canyon's sand and the grove's mid-ground are fixed). |
| 6 | **Atmosphere and motion** | 2 | all | The paper is still: lanterns don't sway, no leaves or petals drift, the searchlights are static. The light is there now; the motion is not. |
| 7 | **Flat floors** | 2 | canyon | Unchanged: the desert floor is one flat plane between the buttes. (Stage-specific.) |

## Systemic ideas, re-ranked (not tied to one stage)

1. **A modelled paper prop kit** (weak points 2 and 1). One shared Blender kit of the props the five stages use, in the paper language:
   crates, barrels, hay bales, a fence, signs, lamps, mushrooms, bushes, stumps, rocks, cacti, the water tower, scaffold and pipes.
   - Each is placed where its code-built prop stands, carrying its own occlusion and lit with the stage by `light_stage.py`.
   - The last code-built objects near the fight go, and the props take contact shade too.
   - Gain high, effort medium.
2. **Printed paper surfaces** (weak point 1). A small set of painted tiling textures per material family: grass, sand, boards, roofing,
   stone, brick, bark.
   - Each is a printed paper surface: fibre, a faint print or hatch, slight colour wobble, paler worn edges.
   - Applied in Blender to the play areas, fronts and props; the toon shader already takes a texture (`use_tex`).
   - This is what moves Surfaces from 3, the lowest column everywhere. Gain high, effort medium; best done with idea 1.
3. **One language near and far: the cheap part** (weak point 3). Two steps, no new geometry, in shader and palette:
   - The full paper finish with the cut-edge rim on every front (the beach and city use the lighter level today).
   - The sheet shadows reaching the fronts' own layers.
   - Gain medium, effort low. Rebuilding fronts as layered card would be the full fix, but per stage.
4. **Paper motion** (weak point 6; *needs the owner's go-ahead: it adds elements*). A shared, gentle sway on hanging and standing paper
   (lanterns, flags, foliage cards, the searchlights' slow sweep) and a few drifting paper leaves or petals per stage. Presentation
   only, one small shader and a particle look. Gain medium, effort low to medium.
5. **Framing** (the owner's call, out of this pass's scope). A slightly higher, closer match camera shows less front and more of the
   fight and the background. It changes every match, so it should be a playtest A/B. Gain possibly high; effort low; risk to feel.
6. **A softer scenery ramp** (from review 1). The beach's smoother light-to-shade turn on the scenery of every stage. Gain low; the
   paper finish and the bake already soften the scenery.

Stage-specific, for later: the beach's light and mood (weak point 4); the canyon's floor (weak point 7).

**Recommended next step:** a pilot of ideas 1 and 2 together on the meadow. It has the most props near the fight (hay bales, fence,
barrel, sign) and the most texture families (grass, logs, stone, path). The props would be modelled as paper with printed surfaces on
them and on the lawn and front, options from the match camera, then measured as before. If Surfaces reaches 4 on the meadow, roll it
out. Idea 3 can go alongside, as it is small.

## The meadow pilot: mocks (2026-10-21, ideas 1 to 3)

The owner's go-ahead: a modelled paper prop kit, printed paper surfaces and the full paper finish on the front, on the meadow, with
options from the match camera.
- **Props:** `art/blender/make_paper_props.py --style A|B|C`. One file per style holds the meadow's hay bale, barrel, signpost, rock,
  fence post and rail, and flower, each named `prop_<name>`, standing on the origin at its in-game size, with its own occlusion baked.
  - A folded card (1,400 triangles), B layered cut-outs (1,400), C rolled and scored (8,300).
  - `StageArt._paper` places each where today's prop stands (`--prop-mock=<glb>`; the builders hand their places to it, the random
    numbers drawn as before so nothing else moves). It paints by family and records the family for the prints.
- **Prints:** `tools/art/make_prints.py --style A|B|C` (Blender's numpy) makes one seamless 512-pixel texture per family (grass, soil,
  path, wood, stone, straw): A fibre and mottle, B hatched print, C halftone. The toon shader's new `print_tex`, `print_amount` and
  `print_scale` lay it in world space on the plane each surface faces, scenery only.
  - `StageBake.mock_surfaces` (`--print-mock=<folder>`) assigns them to the front, play area and paper props by family (`PRINTS`).
- **Front finish:** the shader's `paper_boost` scales the stage's paper finish per surface; `--front-finish=1.6` takes the meadow's "B"
  to about "C" on the front only.
- **Gotchas:**
  - On tops, the print is laid stretched five times along the depth. The match camera sees a top almost edge on, squeezed about that
    much, and laid evenly every mark blurred into the texture's grey even with anisotropic filtering.
  - Prints tiled every 1 to 2 units are invisible at match distance; 3 to 5 units per tile shows the marks.
  - A cut-out bale must stand facing the side, since the game turns bales a quarter round to show their ends.
- **Picks recommended:** C's props with B's print. Options in `mocks/meadow_kit/` (match camera, fighters in play, close-up; a local
  `review.html`) and on the owner's page.
**Tests:** all seven Godot tests pass (nothing changes without the mock flags).

## The meadow pilot built: C's props, B's print, the full finish on the front (2026-10-21)

The owner's picks.
- **Props:** `godot/models/props/paper_meadow.glb` (`make_paper_props.py --style C`, 8,300 triangles), the meadow's kit in
  `StageBake.PAPER_KITS`. The hay bales, barrel, signpost, rock, fence and flowers are rolled and scored paper, each where the built
  prop stood. `--no-paper-props` compares.
- **Print:** `godot/art/stage/prints/hatched/` (`make_prints.py --style B`). The meadow's palette sets `paper_print="hatched"`;
  `StageBake.surfaces` lays it on the front, play area and paper props by family. `--no-print` compares.
- **Front:** the palette's `front_finish=1.6`, the front at about the full paper finish.
- **Light:** the meadow's pieces re-lit around the new props (`light_stage.py`).
- **Test:** `tests/stage_front_test.gd` checks every kit has each prop the builders ask for, in families the game paints.

**Result** (`meadow_kit_built.jpg`, before left, after right; match camera above, fighters in play below):
- The lawn's value is unchanged (0.66 → 0.67) and the path has printed marks (contrast 0.048 → 0.067).
- The front is unchanged in value (0.39).
- The fighters read as before.
- Surfaces on the meadow: 3 → 4 by eye. Up close, the lawn, logs, stones, bales and posts now carry printed paper marks, where they
  were flat colour.
**Tests:** all seven Godot tests pass.

## The rollout: mocks (2026-10-21)

The owner's go-ahead: the rolled-and-scored props and the hatched print on the grove, canyon, beach and city, with options where a stage's
props need new designs.
- **Props:** `make_paper_props.py --style C --variant A|B` adds the other stages' props to the kit. The barrel and rock are the
  meadow's; new designs, two each, cover the mushroom, stump, crate, lamp post, life ring, potted plant, cactus, water tower, air
  conditioner and roof vent.
  - The builders hand their places to the kit, as on the meadow (`StageArt._paper`, `_paper_or_prop`; the water tower in `_city`).
  - The lamp's lantern takes the stage's lamp colour, unshaded, and keeps its halo (`PAPER_LIT`).
- **Prints:** four more families in `make_prints.py` "B": sand, strata, roofing and steel. `StageBake.print_for` reads each stage's own
  meaning of a family (`PRINTS_BY_THEME`: `top` is grass on the grove, sand on the canyon, boards on the beach, roofing on the city),
  then `PRINTS`, then the props' (`PRINTS_PROPS`). The beach's deck takes 0.22: at 0.4 the grain under the fighters was busy.
- **Gotcha:** a command-line path with spaces splits into arguments; the mock prints were copied to a folder without spaces.
- **Picks recommended:** A, with B's stump, crate and cactus. Options in `mocks/prop_rollout/` (match camera, fighters in play; a local
  `review.html`) and on the owner's page.

## The rollout built (2026-10-21)

The owner's picks: A for the new props, with B's stump, crate and cactus.
- **Kit:** one kit for every stage, `godot/models/props/paper_props.glb` (`make_paper_props.py --style C --variant A --b
  stump,crate,cactus`, 32,500 triangles in all, each prop under 10,000). It replaces the meadow's own kit (`StageBake.PAPER_KITS`).
- **Prints:** every stage's palette sets `paper_print="hatched"`.
- **Light:** all five stages re-lit around their new props (`light_stage.py`).
- **Tests:**
  - `prop_check_test` checks each prop of a kit on its own: budget, not floating, not buried. It found the coiled stump and the
    coil-stacked cactus standing a little off the ground; both are grounded now.
  - `stage_front_test` checks the kit has every prop the builders ask for.
- **Result:** `paper_props_built.jpg` (fighters in play, today left, built right: grove, canyon, beach, city).
**Tests:** all seven Godot tests pass (runner_test's shop check flaked once and passed on the next full run).
