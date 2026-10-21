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
