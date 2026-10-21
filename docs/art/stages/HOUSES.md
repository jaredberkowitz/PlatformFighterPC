# The paper house kit, with a scale and footing pass (2026-10-21)

The owner's pick of what comes next after the world-building: the far towns' houses in the paper language, and a believability pass on
scale and footing as they go in. The meadow's village and farm and the beach's harbour town all repeat one model, the cel-shaded cottage
(`art/blender/make_props.py` `cottage`), from before the paper look; in the beach's town it stands fifteen times over with tinted walls.

## The kit

`art/blender/make_paper_houses.py --style A|B|C` builds four houses, each `prop_house_<type>` with its foot at the origin and its front to
the camera, every part in a family the game paints (`StageArt._paper`, with each house's roof and wall colours as tints):
- **cottage:** one storey, eaves to the front, a door, a window, a chimney;
- **townhouse:** two storeys, gable to the front: the town gets height and variety;
- **longhouse:** long and low, a door between two windows;
- **barn:** gable to the front, big doors braced in a white X, a loft door (kept low, as the barn it replaces: taller, it crowded the
  frame's corner).

**Styles:**
- **A, scored card:** crisp folded walls and roof, scored rows on the roof, card strips at the corners and floor line, card frames round the
  lit windows.
- **B, layered card:** timber strips applied over the walls, shutters, flower boxes under the windows, a lamp by the door, the roof in
  overlapping rows of card with a lit lower edge.
- **C, rolled:** the same massing, its corners, beams, frames, eaves and ridge rolled paper tubes, arched doors; the props' language.

`--house-mock=<glb>` on the stage demo puts a kit in: `_cottage` and `_barn` hand their places to it (the town's houses take the types in
turn, `TOWN_TYPES`, so no random numbers are drawn and nothing else moves). `--render <png>` on the builder draws the studio sheet.

## Scale and footing

**The ruler:** a fighter stands about 2.3 units to the top of the head (the name tag sits at 2.6). Life size against that: a door 2.5
high, a sill about 1.1, a storey about 3.2, a cottage's eaves about 3.6.

| What | Where | Size | Verdict |
|---|---|---|---|
| The kit's houses | town, hills | doors 2.5, eaves 3.4 to 3.6, two storeys 6.6 | life size |
| The old cottage | town, hills | eaves 3.8, door about 2.2 | about life size; the town's 0.85 to 1.15 jitter is a real town's variety |
| The meadow's farm | the valley's face | 0.6 of life size | **kept** (below) |
| The meadow's haystacks | the far field | life size | life size |
| The harbour's warehouse and bell tower | the quay, the hill | doors 3.1, tower 16 | life size |

**The farm stays at 0.6.** It stands on the meadow valley's painted face, a forced-perspective layer whose fields and hedgerows shrink as
they run down it (`StageArt._fields`); the farm belongs to that layer's scale. At life size, where it stands (about 40 units back), it
filled the frame's lower left corner. The rule for that face: whatever stands on it takes its scale; life-size things go on the real
ground (the far fields, the hills).

**Footing:** every house stands on a stone footing that runs 0.5 under its floor, so on a slope it sits into the land rather than on a
corner; the town seats each house on the hill under it, a little sunk in (`_harbour_town`), the hills' cottages on `hill_top`, the farm
on the face's highest point under it (`seat`). Checked at the match camera and wide: none floats.

## Options

On the owner's page and `mocks/houses/review.html`: the studio sheets (the four houses per style, with a fighter-height marker), the
beach's town and the meadow's farm cropped at 2x from the match camera, and both stages at the match camera and wide, today against A, B
and C. **Recommended: B, layered card.** At play distance its shutters and flower boxes read as lived-in, where A is crisp but plain and
C's rolled edges are lost (its arched doors remain).

## Built: style B, layered card (2026-10-21)

The owner's pick, shipped as `godot/models/props/paper_houses.glb` (`make_paper_houses.py --style B`), loaded for the themes with towns
(`StageBake.HOUSE_KIT`, `HOUSE_THEMES`: the meadow and the beach); `--no-houses` compares.
- The footing runs 0.5 under the floor (0.8 failed `prop_check_test`'s burial limit of a tenth of the height on the cottage); about 5,000
  triangles a house, under the 10,000 budget.
- `tests/stage_front_test.gd` checks the kit has every house `_cottage` and `_barn` ask for, each part in a family the game paints.
- Before/after: `houses_built.jpg` (before left; the beach above, the meadow below).
- **Tests:** all seven Godot tests pass.
