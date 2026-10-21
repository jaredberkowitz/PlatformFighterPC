# Lessons from the papercraft references (2026-10-21)

The owner's three screenshots of the papercraft adventure's town and cliffs (the benchmark game; never named in the repo), and what our
stages can learn from them. Options on the owner's Stage Look Review page and `mocks/look/review.html`.

## What the references do

- **Buildings at the characters' proportions:** big doors (about 1.3 characters), short walls, roofs as tall as the walls, curved (bell,
  gambrel) and sagging, built of thick overlapping strips; half-timbering with bold X braces, stone blocks as raised pieces, arched doors in
  chunky stone surrounds; few, big windows in thick frames. Nothing quite straight.
- **Ground:** grass is a sheet laid over the land, draped over a cliff's edge with a scalloped cut and its thickness showing; two or three
  flat greens as overlapping cut shapes; paths and paving as separate pale sheets; tufts as folded paper triangles.
- **Stone and cliffs:** chunky cardboard blocks, a few big bevelled planes each, warm grey-beige, stepped into ledges.
- **Distance:** soft, simple, layered hills, each layer simpler than the one in front.
- **Dressing:** bunting between poles, paper chains, origami cranes and pinwheels, confetti: the town is lived in.
- **Material and light:** no ink on the scenery (only the characters are outlined); edges read as lit bevels and soft occlusion; flat
  colour, high key, a faint fibre grain.

## Ranked for our stages

1. Scenery without ink, lit bevels instead (all stages; mocked).
2. The house kit at the characters' proportions (mocked, docs/art/stages/HOUSES.md "Version 2").
3. Grass as a draped sheet over the fronts and hills.
4. Chunky cardboard rock for the walls and cliffs.
5. Town dressing (bunting, paper chains), away from the fight.

## Lesson 1 mocks: scenery without ink

`--scenery-ink=A|B|C` (`Lighting.scenery_ink`): A the scenery's hull outlines and screen lines off (the global `pf_paper`, which the
outline shader reads for everything but the fighters), the lit edge as today; B A with the lit bevel edge doubled (post `paper_edge` 1.6);
C B with the deeper soft occlusion of the retired paper look. Unlike that look (retired 2026-10-19), the light and shade stay as they are.
Recommended: B.
