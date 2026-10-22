# Stage fidelity review 5 (2026-10-22, after the background rebuild, the shadows and the modelled backgrounds)

The owner, after the meadow's farmland: "it may be time for another art analysis in the same ways we have been doing." The same method
as review 4: every stage scored 1 to 5 in the same eight categories (5 = hard to tell apart in quality from the references, the owner's
bar), the pain points ranked, and an order. The references are the same (docs/art/benchmarks/, and docs/art/stages/LOOK.md).

**Evidence:** every stage as it ships today (branch art/stage-refine d9bf39f, the meadow's farmland included) at the match camera and
wide (`review5/fidelity5_sheet.jpg`: meadow, grove, beach, canyon, city, each match then wide), measured in review 3's regions (value,
contrast, saturation). The grove's regions predate its forest and terraces (its "sky" and "distance" boxes now fall on trees), so its
numbers are left out.

**Since review 4:** the background land on four stages (the meadow's farmland, the grove's terraces round the stage with a forest, the
canyon's sand sheets and cardboard buttes); card clouds; a light pass (form on the play tops, the late-afternoon beach); realtime
shadows in the background; the air retune; the believability pass; the detail passes; and the switch to Blender models (the windmill,
the beach's harbour buildings, the city's near towers).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 4 |
|---|---|---|---|---|---|---|---|---|---|---|
| Meadow | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | **3.9** | 3.8 |
| Grove | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | **3.9** | 3.4 |
| Beach | 4 | 4 | 4 | 3 | 3 | 4 | 4 | 4 | **3.8** | 3.8 |
| City | 4 | 4 | 3 | 4 | 2 | 4 | 3 | 3 | **3.4** | 3.4 |
| Canyon | 4 | 3 | 4 | 3 | 4 | 4 | 3 | 3 | **3.5** | 3.3 |
| **Mean** | 4.0 | 3.8 | 3.8 | 3.6 | 3.0 | 4.0 | 3.6 | 3.6 | **3.7** | 3.5 |

- **The background land rose most** (3.0 to 3.6): the grove from 2 to 4 (connected terraces, a real forest), the meadow from 3 to 4
  (fields that tile, lanes that connect, terraces), the canyon from 2 to 3 (sand sheets and cardboard buttes, but its arch and the
  trestle still read as first-pass shapes).
- **The beach's land drops a step to 3, for the sea:** with the town, the harbour and the hut modelled, the sea is now the weakest
  thing on the stage. It's one bright cyan sheet with stripe-like ripple lines, and it fills half the frame.
- **The sky is now the lowest column** (3.0, unchanged). Every stage but the canyon (its low sun and glow) has a plain gradient; the
  city's is a flat magenta.

## What the numbers say (review 4 → now, at the match camera)

- **Meadow:** the lawn still has little form (contrast 0.069 at value 0.69; review 4 found 0.076). The middle hills carry the detail
  now (0.132). The far left is still close to grey (saturation 0.10).
- **Beach:** the town across the bay keeps more colour (saturation 0.19; the graphics review measured 0.12). The sea is 0.61 at
  saturation 0.36 with contrast 0.072: its ripple lines are the only thing on it.
- **Canyon:** the front is unchanged, dark and saturated (0.30, saturation 0.61). The butte behind the fight is a step darker than
  the fighters now (0.47; it was 0.52).
- **City:** the towers behind the fight are still the most contrasty background in the game (0.159; the modelled frames added a
  little). The sky is dark and saturated (0.37, 0.58).

## Pain points, ranked (severity 1 to 5)

| # | Pain point | Severity | Stages | What it is |
|---|---|---|---|---|
| 1 | **The beach's sea** | 4 | beach | Half the frame is one bright cyan sheet with thin stripe-like ripple lines. The references: clear turquoise, lighter near the pier and the shore, deeper further out, soft painted highlights and the reflections of what stands in it. |
| 2 | **Plain skies** | 3 | all (city worst) | Smooth gradients behind card clouds. The references have a soft glow at the horizon, a band of far haze or low cloud, and layered card cloud banks. The city's sky is one flat magenta with dots for stars. |
| 3 | **The canyon's middle distance** | 3 | canyon | The arch's legs still read as stacks of discs beside the cardboard buttes. The trestle and its train cross the whole frame on one line just above the platforms. The heavy front (0.30, saturation 0.61) weighs down the lower half. |
| 4 | **The city behind the fight** | 3 | city | The busiest background in the game sits right behind the fighters (0.159). The scaffold platforms are code-built white lattices beside the modelled brownstone and towers. |
| 5 | **Lawns without form** | 2 | meadow, grove | The play tops are flat at the match camera (meadow contrast 0.069): no soft fall-off toward the front edge, little shade from the platforms above. |
| 6 | **The meadow's stage hill** | 2 | meadow | At the wide camera, the hill under the stage is a regular stepped cake of contour sheets. Next to the terraced farmland it reads as the one machine-made shape. |
| 7 | **Small things still code-built** | 2 | grove, beach, meadow | The lantern strings, flower clusters, the islets' detail (MODEL_AUDIT.md's last row). |

## The order (gain against effort)

1. **The beach's sea** (1): a water pass on the sea's shader. Depth colour from the shore and the pier out, soft painted highlights
   in place of the stripes, the pier's and boats' reflections, a lighter band where it meets the sand. The beach's biggest gain, and
   the largest single area of any stage.
2. **Skies** (2): a sky pass on every stage. A horizon glow, one band of far haze or low card cloud, and the city's sky as a night
   gradient with a glow over the skyline. Settings and one shader, every stage.
3. **The canyon's middle** (3): the arch rebuilt in the buttes' cardboard blocks; the trestle broken up by the buttes in front of it
   or brought lower; the front a step lighter and calmer.
4. **The city behind the fight** (4): the scaffold platforms modelled; the towers right behind the fighters a step quieter.
5. **Smaller:** the lawns' form (5), the meadow's stage hill in fewer, bigger, uneven sheets (6), and the small things (7).

## Item 1, the beach's sea (2026-10-22, mock)

`shaders/water.gdshader` gains `look` (`--sea-look=A|B|C` on the stage demo; 0, the sea as it was, is the default): A clear water
(colour by depth from the pier out, a wandering caustic net near the stage, fine ripples, no crest rows); B A with reflections traced
on the screen (the pier, boats, town, islands and lighthouse, darker than what they show, rippled, fading with distance; GPU about
+0.15 ms); C paper waves (wave-shaped card tiers with lit cut edges) with B's reflections. Lessons: colours written in the shader
are linear (written as display colours they read milky); a screen-space reflection sees the pier's deck top where the water would show
its underside, so reflections are kept darker than what they show. On the Stage Look Review ("sea", `sea_*`).
