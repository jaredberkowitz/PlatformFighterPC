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
- [x] Grove: the pale corner shapes (the canopy's top rows; deepened, round 1).
- [ ] More rounds of the loop across all five stages.
- [x] Run HUD fighter plate in the match HUD's dark panel (round 15); the rest of the run HUD to check with the modes-online agent.
- [x] Controls screen hierarchy (round 12).
- [ ] Player-visible text says "strong attack", never "smash" (README, How to play.txt, in-game text); the owner chose "strong attack"
      (2026-10-15). Internal move ids (`fsmash` etc.) later, with the moveset session.
- [ ] Ink painted into the stage textures (stone and plank edges, trims, strata ledges) eased toward the Wind Waker HD reference, where
      colour and value carry the scenery; the post-pass and outline-shell levers made no visible difference (art review 6 round 22).
- [ ] Meadow: the soft blue-grey band behind the hills (not the ridge or the mist; likely the sky shader's horizon layer) to firm up
      into painted mountains or clear sky (art review 6 round 29).
- [ ] Mend `--demo=results` (it stops in the match) and review the results screen.
- [ ] In-motion checks for the refinement rounds (the outside brief): an attack with its hit effect, both fighters airborne, fighters
      spread so the camera pulls out; look for outline flicker, shadow crawl, effects hiding fighters.
- [ ] The run mode's own city windows (`run_stage.gd` facade values) to match the stage's calmer ones (check with the modes-online agent).

## Art review 6 leftovers
- [ ] Combat effects second pass: brief strong-hit accent, landing and dash dust in the same painted family, per-weapon spark colours.
- [ ] Camera and off-screen feedback checks (no camera behaviour change without the owner).
- [ ] Performance and regression pass: every stage, four fighters, supported resolutions.
- [ ] Run mode shadow crawl on camera pans, if still present.

## Beta gaps against the competition (Smash Ultimate, Rivals of Aether II, PEAK), 2026-10-15
Already in place: knock-out blasts, the final hit's slow-motion close-up, results screen with victory poses, damage that heats in
colour, music, alternate looks. What the competitors have that a beta of ours still lacks, art and presentation only:
- [ ] **Readable roster at a glance**, within the one body (owner, 2026-10-15: one shared model; variety comes from the character
      creator's customisation, never body variants). Make the creator's options (headwear, outfits, colours, accessories) bold enough
      that two fighters read apart in silhouette and colour at play distance.
- [ ] **Match entrances.** Each fighter arrives at the start (a drop-in, a dash in, a pose) rather than appearing in place.
- [ ] **Weapon art for the queued classes** (Dagger, Nodachi) and per-weapon hit sparks.
- [ ] **Living stages.** The competitors' backgrounds move (crowds, traffic, weather, wildlife); ours have a little (windmill, birds,
      boats). One or two signature moving things per stage, kept out of the fighting plane.
- [ ] **Off-screen fighter indicator** (the magnifier bubble at the screen edge): verify ours reads at a glance and stays clear of the HUD.
- [ ] **Model quality** (the PEAK bar): the body remodel and the pose overhaul above are the biggest single gap.
