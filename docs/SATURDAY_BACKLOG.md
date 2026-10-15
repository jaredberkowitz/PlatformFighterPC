# Saturday backlog (2026-10-17, after the usage reset)

Everything put off to save usage, in the order to take it. Tick items off here as they land.

## Builds
- [ ] Ship a build with everything since art 49 (canyon haze, city windows, pose atlas tool, anything below): export, desktop folder,
      zip, GitHub release (docs/STATUS.md and the desktop-build memory for the steps).

## Animation overhaul (the main job; start from docs/POSE_REVIEW.md)
- [ ] Up moves arch the back with the belly forward ("inhaling"): reach from the shoulders, hips over the feet.
- [ ] Locked straight arms at shoulder height in strikes, follow-throughs and unarmed idles ("hug"): elbow never fully straight,
      follow-through pulls back toward the chest.
- [ ] Free arm flung out sideways (T-pose) in unarmed fairs/fsmashes: guard the chin or chamber at the hip.
- [ ] Hurt pose: a real recoil (head and chest snap back from the hit, arms trail, knees give), not the sword across the face.
- [ ] Launch pose: much less stretch; curled or spread, limbs and weapon trailing the flight.
- [ ] Per-family pose sets: nine unarmed classes share one set; Dagger and Nodachi show no weapon (queued weapon art).
- [ ] Strike frames upside down on flips; one static jump pose (tuck at the apex); shield arm crossing the chest; hands shrinking on
      grips and sideways wrists; crossed-knee idle stances.
- [ ] Locomotion per family (idle, walk, run, dash, jump/fall/land, shield, dodge), then reactions (ledge, get-up, tumble).
- [ ] Before/after every fix with `--demo=poses --gallery-class=N` (both facings); all Godot tests and the look bench.

## Body
- [ ] Review and merge the remodelled body and mitten hands (branch `body-cage`, another session's work) before posing on it.
- [ ] Open Blender before the session starts so the Blender connection loads (it timed out this time).

## Graphics refinement loop (screenshot every stage, fix the two weakest, repeat)
- [ ] Grove: the pale washed-out shapes in the bottom corners. Not the canopy rows, far tree line, under-stage haze or post haze
      (each changed with no visible effect). First identify the object at those pixels, then fix.
- [ ] More rounds of the loop across all five stages.
- [ ] The run mode's own city windows (`run_stage.gd` facade values) to match the stage's calmer ones (check with the modes-online agent).

## Art review 6 leftovers
- [ ] Combat effects second pass: brief strong-hit accent, landing and dash dust in the same painted family, per-weapon spark colours.
- [ ] Camera and off-screen feedback checks (no camera behaviour change without the owner).
- [ ] Performance and regression pass: every stage, four fighters, supported resolutions.
- [ ] Run mode shadow crawl on camera pans, if still present.
