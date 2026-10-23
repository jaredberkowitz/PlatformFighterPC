# Stage fidelity review (2026-10-20, after passes 1 to 6 and the folded style)

The owner: refinement has carried the stages a long way, but reaching the references' fidelity may need a change of approach, as the
move from code-built props to Blender models was. This review scores every stage again from fresh captures, ranks the weak points, and
proposes approach-level changes.

**Evidence:** round 56 (`docs/gauntlet/round56/`, every stage at the match camera, greyscales and fighters in play) and
`reference_pairs_r56.png` (each stage beside its reference). Scores are judged by eye against the references, which are rendered by much
larger teams; 5 means hard to tell apart in quality, the owner's bar. Facts about the code are checked, not guessed.

## Scorecard (1 to 5, 5 = on par with the reference)

| Stage | Lighting | Surfaces | Modelling | Composition | Cohesion | Readability | Mean |
|---|---|---|---|---|---|---|---|
| Meadow | 3 | 3 | 3 | 3 | 4 | 4 | 3.3 |
| Beach | 3 | 3 | 3 | 3 | 3 | 4 | 3.2 |
| City | 3 | 3 | 3 | 3 | 3 | 3 | 3.0 |
| Canyon | 3 | 2 | 3 | 2 | 3 | 3 | 2.7 |
| Grove | 2 | 2 | 3 | 2 | 2 | 4 | 2.5 |

## Weak points, ranked (severity 1 to 5, 5 = the biggest gap to the references)

| # | Weak point | Severity | Stages | Evidence and cause |
|---|---|---|---|---|
| 1 | **Flat light: no occlusion, no bounce, no light falling through the scene** | 5 | all | The references get most of their richness from light: dark crevices between roots and stones, warm bounce under the dock, shade pooling under trees. Ours shades each surface by its own normal into two or three cel bands, with a flat ambient colour (`lighting.gd`) and screen-space AO only on the highest quality. Nothing is baked; there is no global illumination. Every surface reads as lit the same way, at any depth. |
| 2 | **The play area is still built by code** | 5 | all | The props and ranges are modelled, but the pieces nearest the fighters are not: stage fronts (`solid_block`, `_faceted_ledge`, `quilted_island`, `_dock_block`, the city facade), the floating platforms, the fence, the girders and the city's towers are boxes, cylinders and generated grids in `stage_art.gd` (about 7,000 lines). The references' play areas are sculpted, bevelled, worn and irregular. |
| 3 | **The stage front fills a quarter of the frame with the least detail** | 4 | meadow, canyon, grove, beach, city | The meadow's cliff, the canyon's strata block, the grove's quilted underside, the beach's posts and the city's facade each take about a quarter to a third of the frame, below the fight, and are the plainest surfaces in it. The references either crop the front or make it the richest part (the grove's roots and rocks). |
| 4 | **Surfaces without painted detail** | 4 | canyon, grove, beach, city | No hand-painted wear, edge highlights, chips or colour variation at the scale the references have. Where texture exists it repeats visibly (the canyon's strata tile, the grove's quilt diamonds) or is absent (the lawns, by the style sheet's rule). The harbour reference's planks each carry painted grain and worn edges. |
| 5 | **Empty mid-ground** | 3 | grove, canyon, meadow (left) | The grove's island floats in a large plain teal field; the canyon's floor is one flat sand plane with scattered rocks; the references fill the same space with layered forest, cliffs or a dense town. (The owner's rule is refine, not add; this needs the owner's go-ahead.) |
| 6 | **Mixed construction languages in one frame** | 3 | grove, canyon | Folded-paper scenery behind smooth soft-shaded foreground (the grove's rolled lip and smooth trunk, its quilted diamond underside; the canyon's drawn strata front under folded mesas). The meadow is the most consistent. |
| 7 | **Atmosphere and motion** | 3 | grove, beach, city | No light shafts, lantern glows, water ripples round the posts, water depth colour or searchlights; the references use them to give mood and depth. |
| 8 | **Readability edge cases** | 2 | city, canyon | The look bench still fails the darkest body on the city's dark backdrop (a fighter-side fix); the orange fighter stands on orange sand on the canyon (separated by value, 0.13). |

## What changes the approach (ideas, ranked by expected gain)

1. **Bake light in Blender.** Build or import each stage's scenery in Blender and bake ambient occlusion and bounce light into vertex
   colours (or a lightmap), which the toon shader multiplies in. Static scenery makes this cheap at run time. It fixes weak point 1 on
   every stage at once and is the closest match to the code-to-Blender jump. Godot's own `LightmapGI` is an alternative inside the
   engine; the Blender bake keeps the cel look under control. *Test first:* bake one stage's props and front, compare at match distance.
2. **Author each stage as one Blender diorama.** Replace the code-assembled stage front, platforms and near props with one `.blend` per
   stage, built and lit against the match camera, with the sim's collision rectangles imported as guides so the top and ledges stay
   where the fighters stand. Code would only place it. This is the full version of the code-to-Blender move for the play area (weak
   points 2 and 3), and it lets the stage front become sculpted (roots and rocks, a carved cliff, a real pier).
3. **A painted texture pass with trim sheets.** One texel density, hand-painted trim sheets per material family (wood, stone, rock,
   brick, earth) with edge wear and colour variation, applied in Blender. Weak point 4; works with idea 2.
4. **Concept paint-overs before building.** One painted target frame per stage in our own style, from the match camera, approved by the
   owner before modelling, as the character pass did with mocks. The references are other games' looks; a target in ours makes a 5
   measurable.
5. **A softer scenery shader.** The fighters keep the cel look; the scenery moves to a smoother painted ramp with occlusion and a rim,
   as the beach already uses a softer ramp. Small, systemic, and it pairs with idea 1.
6. **Framing (the owner's call, out of this pass's scope):** a slightly higher, closer match camera shows less stage front and more of
   the fight and the background, as most of the references do.

**Recommended path:** a one-stage pilot of ideas 1 and 2 together, on the grove (the lowest score and the richest reference: roots,
rocks, a twisted trunk, a forest). Bake first, measure the gain, then rebuild its stage front as a sculpted diorama. If the pilot lifts
the grove to a 4, roll it out stage by stage. The Blender connection (MCP) is live for authoring; any generated model still needs the
owner's go-ahead first.
