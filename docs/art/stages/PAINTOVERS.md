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

## Built: the owner's picks (2026-10-21)

The owner picked sheet shadows and cut edges on every stage, the light lift round the fight, and air on the meadow and beach. All are in
the match's finishing pass (`shaders/post.gdshader`), set per stage in `Lighting.STYLES` `finish`. They reach the stage and scenery
only (object ids from 0.8 up; the fighters and their outline hulls are untouched) and cost nothing new to build.
- **`sheets`:** each scenery pixel looks toward the stage's key light on screen (`sheet_dir`, from the key light). Where something stands
  half a unit to three nearer, it is in that thing's shadow; six depth taps at 5, 11 and 20 pixels. The game casts along the real
  light, so the shadows fall down and away from it, where the paint-over cast them a fixed way. Fighters cast it, never take it.
- **`cut_edge`:** the near side of each step back of more than half a unit, two pixels across, mixed toward a paper cream.
- **`fight_lift`:** a soft lift of the light round the frame's middle.
- Shadows and edges are full to 12 units behind the play area and gone by 50. (A first try faded them from 2 units behind and showed
  nothing: the camera is about 27 units out with a 30° lens, and the sheets that take the shadows are 15 to 60 units back.)
- **Air:** the meadow's and beach's `haze_max` 0.06 and 0.07 to 0.24.
- **Per stage** (sheets / cut edge / lift): meadow 0.49 / 0.5 / 0.15, grove 0.56 / 0.5 / 0.18, beach 0.42 / 0.45 / 0.15, canyon
  0.56 / 0.5 / 0.15, city 0.63 / 0.55 / 0.24.

**Result** (`paper_target_built.jpg`, today left, built right), against the paint-overs' numbers:

| Stage | Play area: today → built (target) | Distance value: today → built (target) |
|---|---|---|
| Meadow | lawn 0.64 → 0.66 (0.69) | 0.61 → 0.67 (0.66) |
| Grove | lawn 0.74 → 0.76 (0.78) | 0.62 → 0.63 (0.63) |
| Beach | deck 0.50 → 0.52 (0.53) | 0.63 → 0.67 (0.69) |
| Canyon | sand 0.72 → 0.74 (0.74) | 0.75 → 0.76 (0.75) |
| City | roof 0.59 → 0.63 (0.64) | 0.39 → 0.39 (0.32; the light move, not picked) |
**Tests:** all seven Godot tests pass.
