# Stage paint-overs: the paper target (2026-10-21, review 2's idea 6)

One target frame per stage, in our own style and from the match camera, so the next passes are measured against a picture of ours and not
other games' looks. Each frame is the stage as it is today with the paper language painted over it. Each move is a separate layer, so
the owner can pick which ones become the target.

## Method
- **Capture:** the stage demo with `--depth` writes, beside the frame, its distance from the camera (`_depth.png`, white at the
  camera to black at 160 units, with tone mapping off) and the same frame without the fighters (`_stage.png`). The paint-over
  leaves the fighters as they were.
- **Paint:** `tools/art/paintover.py` (run with Blender's Python, which carries numpy), per stage look in its `LOOKS` table:
  `blender --background --python tools/art/paintover.py -- <frame.png> <depth.png> <out.png> <stage> [--only <move>]`.
- **Options:** `docs/art/stages/mocks/paintover/` (`po_<stage>_T` today, `_P` the paint-over, `_<move>` one move alone), with a local
  `review.html`, and on the owner's page.

## The moves, and what each would be in the game

| Move | What it paints | How the game would do it |
|---|---|---|
| **layers** | Each nearer sheet's soft shadow on the sheet behind it, cast away from the light: the scene reads as cut sheets standing apart | Baked drop-shadow cards behind the paper pieces in Blender, or a screen-space depth shadow on the scenery |
| **edges** | A pale die-cut edge along each near layer's outline against what is behind it | A depth-edge rim on the scenery (the paper finish's cut edge, by depth instead of by normal) |
| **crevice** | Shade where a surface sits deeper than its surroundings, near the fight only | The contact shade baked in idea 3, deeper, or the engine's ambient occlusion on the scenery |
| **light** | A warm wash from the light's side, a cool one away from it, a soft lift round the fight and a gentle fall-off to the corners | Palette and grading per stage (`Lighting`), no new pieces |
| **air** | The distance toward the stage's air colour, with lower contrast | The existing depth curve (`_air_amount`) pushed further |
| **grain** | A faint paper fibre, none in the sky | The paper finish's grain (`pf_paper_finish.x`) a step up |

## Measured: today → paint-over (match camera, the audit's regions)

| Stage | Play area value | Distance contrast | Notes |
|---|---|---|---|
| Meadow | lawn 0.64 → 0.69 | 0.089 → 0.057 | Distance lighter (0.61 → 0.66); the path's edge crisper (contrast 0.047 → 0.071) |
| Grove | lawn 0.74 → 0.78 | 0.089 → 0.077 | The sky calmer (contrast 0.037 → 0.020); the lip unchanged (0.29) |
| Beach | deck 0.50 → 0.53 | 0.068 → 0.051 | Distance lighter (0.63 → 0.69); sea brighter (0.58 → 0.68) |
| Canyon | sand 0.72 → 0.74 | 0.006 → 0.013 | Near sand separates from the far (contrast 0.055 → 0.071) |
| City | roof 0.59 → 0.64 | 0.091 → 0.078 | The sky a step darker (0.38 → 0.26), so the roof reads first |

The pattern on every stage: the play area a step lighter, the distance lighter and softer, the near layers' edges and their shadows sharper.
A stage reaching the paint-over's numbers and its layered look would score 5 on lighting and cohesion.
