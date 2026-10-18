# Gap analysis against a shipped competitor (2026-10-18)

The owner shared a match screenshot of a current commercial platform fighter (a harbour stage: a wooden dock of planks, crates and fruit
barrels, a painted town and mountains behind, turquoise water). This is what separates it from our stages, and what to do about it.
Our stages for comparison: `docs/screenshots/review8_all_stages.jpg`; our beach at a closer camera: `docs/screenshots/beach_close_camera_test.jpg`.

## What we tried first

A quick test on the meadow: environment ink lines off, a soft gradient light ramp instead of cel bands, screen-space ambient occlusion
on. The frame barely changed. Our gap is not in the lighting or the finishing pass; it is in what the stages are made of and how they
are framed. That is why the last rounds of small refinements felt like getting stuck.

## The differences, biggest first

1. **The stage is a place, not a box.** Theirs is a dock: every surface you stand on is a believable object (planks on posts, a crate,
   a hanging board on ropes), the edges are irregular, and the play space sits inside the scene. Ours is the classic abstract formula:
   one block with a painted front and slabs floating over a backdrop. That formula is the core of the "Flash game" read.
   - Do: rebuild each stage's playable structure as something that exists in its world, with the same collision. Beach: a dock or pier
     on posts. Meadow: a farm bridge or hay loft. Canyon: a mine scaffold or rope bridge between mesas. Grove: the great tree's roots and
     a treehouse deck. City: a rooftop with scaffolding and a water tank. Platforms can stay where they are and stay floating in play;
     visually they become boards, crates or hanging planks (posts and ropes are scenery, not collision).
2. **Texture density and painted materials.** Their wood, crates and barrels have hand-painted textures at high resolution: grain,
   worn edges with baked highlights, dark painted trim, colour variation inside every plank. Ours are flat cel fills or small procedural
   tiles. Up close (see the close camera test) our sand and walls are nearly blank.
   - Do: a set of hand-painted trim sheets per stage (planks, stone, earth, roofing, metal), 1–2k, with edge wear and lighting painted
     in, and models built to use them (bevelled edges, plank-by-plank geometry). Material Maker can make the base; painted passes on top.
3. **Composition and camera.** Their camera is lower and closer: fighters fill about a quarter of the screen height, foreground props
   (barrels, crates, fruit) overlap the bottom of the frame, and the scene recedes in clear layers (near pier, town across the water,
   mountains). Ours sits high and far: fighters fill about a tenth, the middle distance is empty, and the backdrop jumps from the stage
   straight to far hills.
   - Do: a foreground layer in front of each stage's base; a mid-ground layer between stage and horizon; and an owner decision on a
     closer default camera (a gameplay call, not an art one).
4. **Shading model.** Theirs is soft stylised lighting: smooth gradients, contact shadow in every crevice, specular glints, dark
   modelled trim instead of drawn ink. Ours is cel bands plus ink lines plus paper grain, which reads as an illustration. Once the
   geometry and textures carry the detail (1 and 2), move the environment to soft painted shading and keep cel and ink for the fighters.
5. **Background painting.** Theirs is a dense painted vista: a whole town in the middle distance, atmospheric perspective, many small
   shapes. Ours are a few large simple shapes. Do: dense set dressing built from a kit and instanced (a town, a forest, a desert
   settlement), or painted matte backdrops for the far layers.
6. **Characters.** Theirs are detailed, textured, designed creatures with strong silhouettes; ours are simple round bodies. This is the
   largest gap for a sellable look and the largest job; it needs its own art-direction plan (concepts, then models), separate from stages.
7. **Water and effects.** Their water has caustics, sparkle and depth colour; ours is banded. Smaller, later.

## Suggested plan

Stop the small per-surface passes. Do one stage as a vertical slice, the beach (it maps most directly to the reference): a dock built
of real planks, posts and crates on the existing collision; hand-painted trim textures; a foreground layer of barrels and nets; a
mid-ground harbour; soft environment shading. Review it against the reference, then roll the method to the other stages. Decide the
camera distance and the character direction in parallel.

Rule changes this implies: it adds new elements (a dock, foreground props, a mid-ground town), which the current "refine, don't add"
rule forbids, so it needs the owner's go-ahead.
