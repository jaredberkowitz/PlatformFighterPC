# Stage fidelity review 2 (2026-10-20, after the Blender fronts and the paper backgrounds)

The owner, at the end of this pass: another analysis of every stage, in the same way as the first (`FIDELITY_REVIEW.md`). Same six
categories and the same 1 to 5 scale (5 = hard to tell apart in quality from the reference, the owner's bar), judged by eye against
the same references and checked against measurements.

**Evidence:** round 69 (`docs/gauntlet/round69/`: every stage at the match camera, greyscales, fighters in play `play/`, wide `wide/`),
compared with round 56 (the first review's captures). Page images: `docs/art/stages/review2/` (before, now, reference, in play and
wide for each stage; a local `index.html` to view them).

## Scorecard (round 56 → round 69)

| Stage | Lighting | Surfaces | Modelling | Composition | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|
| Meadow | 3 → 3 | 3 → 3 | 3 → 4 | 3 → 4 | 4 → 4 | 4 → 4 | 3.3 → **3.7** |
| Grove | 2 → 3 | 2 → 3 | 3 → 4 | 2 → 3 | 2 → 4 | 4 → 4 | 2.5 → **3.5** |
| Beach | 3 → 3 | 3 → 3 | 3 → 4 | 3 → 3 | 3 → 4 | 4 → 4 | 3.2 → **3.5** |
| Canyon | 3 → 3 | 2 → 3 | 3 → 4 | 2 → 3 | 3 → 4 | 3 → 3 | 2.7 → **3.3** |
| City | 3 → 3 | 3 → 3 | 3 → 4 | 3 → 3 | 3 → 3 | 3 → 3 | 3.0 → **3.2** |

Every stage moved up; the grove most (2.5 to 3.5), from the bottom to the middle. **Modelling** moved on every stage (the fronts and the
backgrounds are modelled now) and **Cohesion** on four (one paper language behind each fight). **Lighting** barely moved: it is now the
lowest column on every stage, with **Surfaces**.

## What the numbers say (round 56 → 69, value and contrast at the match camera)

- **Meadow:** the valley is calmer (saturation 0.40 to 0.22) and has more form (contrast 0.07 to 0.15); the front is warmer and more
  varied (contrast 0.04 to 0.10); the lawn unchanged (0.64).
- **Grove:** the distance lightened (0.50 to 0.64) and the mid-ground has form (contrast 0.09 to 0.12); the front under the lawn is now
  dark roots and rock (0.29, was the pale quilted lip at 0.61), so the lawn's edge reads as one clean line.
- **Beach:** the dock's posts read (0.34 to 0.43, contrast unchanged at 0.16); the distance a touch more colourful.
- **Canyon:** the front calmer (0.32 to 0.28); the near butte behind the fight lighter (0.50 to 0.55, near the fighters' value); the
  distance flatter (contrast 0.019 to 0.003: the paper dunes sit softly into the horizon).
- **City:** the front a step under the roof (0.40 to 0.29: the play surface reads first); the towers' value unchanged (0.30).

## Weak points, ranked (severity 1 to 5)

| # | Weak point | Severity | Stages | Evidence and cause |
|---|---|---|---|---|
| 1 | **The play surface and platforms are now the plainest things in frame** | 4 | all | The fronts and backgrounds are modelled, but what the fighters stand on is still code-built and fills about a third of every frame in play (`play/`): the lawn tiles, the deck's planks, the canyon's flat sand slab, the roof. So are the floating platforms (all stages), the meadow's fence and the city's girders. Next to the new pieces they read as the old method. |
| 2 | **Light is still flat away from the models** | 4 | all | Baked scenery light covers only the grove (`StageBake`, the bake output is local). The new models carry their own occlusion, but the play surface, platforms and code-built props do not, so there is no contact shade where the old meets the new (the canyon's sand over its slabs, the city's roof over the brownstone). The references' richness is mood light: lanterns in the grove, searchlights over the city, warm bounce under the dock. |
| 3 | **Two languages, near and far** | 3 | meadow, city, beach | The backgrounds are paper; the fronts are solid modelled material (the meadow's logs and stones, the city's brick, the beach's timber). The canyon is the exception: its stacked-slab front and its card-stack buttes are one idea. Elsewhere the eye reads a 3D diorama in front of a paper backdrop. |
| 4 | **Paper without paper's surface** | 3 | meadow, beach, grove, canyon, city | The paper reads by shape alone. No fibre, print grain, slight colour wobble or soft cut-edge highlight, which is what sells paper up close; the play surfaces' tiles are generic. |
| 5 | **Readability edge cases** | 3 | canyon, grove, city | The canyon: the orange fighter on orange sand (0.65), the butte behind the fight now 0.55 against fighters near 0.5. The grove: the mid-ground at 0.50 against the fighters' 0.55. The city: the darkest body on the dark backdrop (a fighter-side fix, open since round 44). |
| 6 | **Flat floors** | 2 | canyon | The desert floor is one flat plane with scattered rocks between the buttes; the other stages' floors are now land, sea or street. |
| 7 | **Atmosphere and motion** | 2 | grove, city, beach | The paper backgrounds are still: no lantern glow, searchlights, ripples round the posts or drifting leaves (the references use them). |

## Ideas for the next pass, ranked by expected gain

1. **The play surface and platforms in Blender, in the paper language** (weak points 1 and 3). Each stage's top as a thick card slab
   with a die-cut edge and its own printed surface (grass, sand, planks, roofing), the floating platforms as matching card or timber
   props, all on the sim's collision with the alignment test extended to platforms. This finishes the code-to-Blender move for the
   play area and ties the near to the far.
2. **One paper finish for all scenery** (weak points 3 and 4). A scenery-only shader option: a faint paper fibre grain, a slight
   per-sheet colour wobble and a pale cut-edge rim on silhouettes (the city's die-cut edge as a shader). Applied to fronts, props and
   backgrounds alike; fighters untouched. Cheap and systemic.
3. **Re-bake every stage's light around the new pieces** (weak point 2). The pipeline exists; only the grove is baked. Add contact shade
   where the play surface meets the front.
4. **Readability fixes** (weak point 5): the canyon's sand a step cooler or the butte behind the fight a step darker; the grove's
   mid-ground a step darker. Small palette passes, measured.
5. **Mood light, in paper** (weak points 2 and 7, *needs the owner's go-ahead: it adds elements*): cut-out light beams for the city's
   searchlights, paper lanterns glowing in the grove, ripple rings round the beach posts.
6. **Concept paint-overs** of the paper target per stage before ideas 1 and 2, as the first review suggested; with the paper language
   settled, a target frame in our style would make a 5 measurable.

**Recommended next step:** a pilot of ideas 1 and 2 together on the meadow (the highest score and the clearest near/far split): its
lawn top and the two platforms as paper card on the collision, and the paper finish on its front and background, measured as before.
If it lifts the meadow to a 4, roll it out stage by stage.
