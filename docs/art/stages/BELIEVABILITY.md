# Believability pass (2026-10-22)

The owner: "I am noticing some things having incorrect positioning, e.g. overlapping props, a house being placed in the middle of a road."
Every stage was checked two ways: by eye, at the match camera and wide, and by a temporary audit in the stage view that listed every
placed piece's footprint, reported pairs that overlap and anything standing on a lane, road or the dry wash.

## Found and fixed

| Stage | Problem | Fix |
|---|---|---|
| Meadow | The left lane ran straight under the barn (the lane at x -36, the farm -37.5 to -26.5): the house in the road. | The lane moved beside the farm, x -47 (`make_meadow_ground.py`); ground rebuilt, re-lit. |
| Meadow | The scarecrow stood right before the farmhouse's door; the washing line ran across the barn's front and the lane's edge. | Both moved into the field right of the farmhouse, each seated on the land by a ray cast onto the ground model (`make_world.py` `ground_y`). |
| Grove | The festival gates were placed where the side trees stand, so each trunk grew through its gate. | The gates moved beside the trees, toward the stage (x -19 and 21). |
| Canyon | The dry wash ran under the far butte at (44, -112). | Rerouted through the gaps between the buttes (`make_canyon_paper.py` `WASH`); ground rebuilt, re-lit. |
| Beach | The three boats moored at the town sat against the quay's foot and read as sitting on the sand, sails up. | Moored 4 units further out, on the water (`StageArt._harbour_town`). |

## Checked and left

- City: the scaffold platforms' legs reach the near curb, in front of the road, so the traffic passes behind them.
- Meadow: the pine cluster in front of the windmill and the farmhouse's smoke rising past it are 20 units apart in depth; they overlap
  only in the picture.
- Beach: the lighthouse and the stilt hut overlap only in the wide view (34 units apart in depth).
- The audit's other reports were parts of one object (a water tower's legs and tank, a gate's posts and lintel) or scattered flowers
  and ferns.

Before and after: `review5/believability_built.jpg` (meadow farm, grove gate, beach boats).
