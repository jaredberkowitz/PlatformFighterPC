# Stage art backlog (paused 2026-10-19, at alpha)

The owner and the playtesters rate the stage art a good alpha. Work moved to the character models; this is where the stages stand and
what is next when we return. Latest captures: `docs/gauntlet/round43/`. Latest full judge verdict: round 41 (meadow 4/4, beach 4/4,
canyon 3/3, city 3/3, grove 3/3; reference / ship-ready out of 5).

## Next, in order
1. **Distance in steps.** Each row of ranges, hill bands and city tower layers a clear step lighter and cooler than the one in front,
   toward the sky's horizon colour (extend the palettes' `[air]` curve, `StageArt._air`, to the ranges and land bands).
2. **Light edges against the sky.** A rim toward the sun on far ridges, tree crowns and towers (a fresnel term in the toon shader for
   scenery, gated per material).
3. **Grove crowns.** Fold planes angled 30 to 45 degrees so each dome rounds off (`fold_kit.folded_dome`); they still read as stacked
   boxes in greyscale.
4. **City.** Tower layers stepped back in value and window density; a warm street glow at the tower bases; fold the shopfront awnings.
5. **Canyon.** Long dusk shadows from the buttes across the desert floor. Light shafts are subtle with the sun low behind the dunes;
   they would show more with the sun higher behind the arch (the owner's call, it moves the sun on screen).
6. **Beach.** A chunky modelled lantern in place of the thin lamp post.
7. **Grove.** Overhanging turf tufts breaking the lip's silhouette; folds on the empty left mid-ground plane.
8. **Meadow.** Stronger platform cast shade on the lawn; possibly a stone outcrop (an addition, needs the owner's go-ahead) and a couple
   of clouds if the sky still reads empty (an addition).

## Recommended techniques not yet tried
- Baked crevice shading (ambient occlusion) in Blender into each prop's vertex colours, read by the toon shader.
- Motion: drifting clouds already move; cloud shadows over the far hills, heat shimmer in the canyon, sparkle on the sea.
- Paint-overs or approved concept frames per stage as targets (needs the owner or an image tool).
- A debug key overlaying the stage's reference in a corner; the review script measuring each depth layer's average value.

## Tools (docs/ART_WORKFLOW.md)
`tools/art/review.sh <round>` (captures, greyscale, before/after, contact sheet), `--props=` lineup, `godot/tests/prop_check_test.gd`,
stage palettes in `godot/stages/palettes/`, `art/blender/fold_kit.py`, `tools/art/review/diff.gd`.
