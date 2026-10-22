# Code-built art still in the backgrounds (2026-10-22)

The owner: worried about quality; switch the art over to Blender models wherever possible. And: the forest is a few sporadically placed
trees, not believable.

Each stage's background builder (`StageArt`) was traced for pieces still assembled from Godot primitives (`_box`, `_cylinder`,
`_sphere`) rather than loaded from a modelled glTF. The count is primitive calls in the builder; a star means the piece already has a
modelled part and the rest is primitives round it.

| Stage | Still code-built (background) |
|---|---|
| Meadow | the windmill* (74: only the tower is modelled; the base, door, windows, gallery, cap housing and all four sails are primitives), the sky islands and their detail (25), round trees*, flowers*, card clouds |
| Grove | flower clusters, the lantern strings (cords and bulbs), the great tree's canopy pieces*, card clouds |
| Beach | the bell tower (22), the stilt hut and jetty (26), the dock harbour pieces (22), the warehouse (17), the market stalls and awnings, the islets' detail (10), palms*, card clouds |
| Canyon | the rock arch (code under the modelled ground, hidden), card clouds; most else is modelled |
| City | the skyline towers (code facades, 20 per layer), the rooftop units (16), the street dressing |

The modelled side, for contrast: every stage's front, play area and background ground; the house kit; the world pieces
(`make_world.py`); the mood light; the side and terrace trees; the props in the paper kit.

## Order (gain against effort)

1. **Forests** (believability): the grove's terraces filled with a dense modelled forest (piloted below); then the meadow's pine belts.
2. **The meadow's windmill**: one Blender model (base, door, gallery, cap, sails), replacing 74 primitives.
3. **The beach's harbour**: bell tower, warehouse, stilt hut and jetty, market stalls, as Blender models in the house kit's style.
4. **The city's skyline**: modelled towers with real window frames and ledges, replacing the code facades.
5. **Small things**: lantern strings, flower clusters, islet detail, clouds as modelled cards.

## Pilot: the grove's forest

`make_grove_paper.py --variant H` (forest on by default, `--forest 0` without): the middle and back terraces filled with trees on a
jittered grid (7.5 and 6.5 apart), each a fluted, flared trunk and four pleated tiers in the grove tree's manner, the back row the
tallest (13 to 19) so it rises over the middle (10 to 14); clear of the edges, the village's great trees (12), the side and terrace trees
(9) and the space behind the fight; one spot in eight left as a clearing. 282 trees, about 100,000 triangles joined into a few dozen
meshes; GPU 2.79 ms against 2.53. Mocked with `--ground-mock`; before and after on the Stage Look Review.
