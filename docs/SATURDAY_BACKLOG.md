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

## Bugs
- [x] Run slides facing left: fixed 2026-10-15 (the shared animation library; a second fighter's build switched the `@m` clips' loop
      off). Regression demo: `--demo=dashleft`.

## Body
- [ ] Review and merge the remodelled body and mitten hands (branch `body-cage`, another session's work) before posing on it.
- [ ] Open Blender before the session starts so the Blender connection loads (it timed out this time).

## Graphics refinement loop (screenshot every stage, fix the two weakest, repeat)
- [x] Grove: the pale corner shapes (the canopy's top rows; deepened, round 1).
- [ ] More rounds of the loop across all five stages.
- [ ] Canyon platforms: three dark claw-like slashes on the board's top near its end (close camera, `--demo=stage --stage=1 --cam=7`).
      Not a named mark in stage_art.gd; probably the wood tile's grain (`art/stage/wood*`) or a scuff (`scuff.svg`). Find with `--pick`.
- [x] Run HUD fighter plate in the match HUD's dark panel (round 15); the rest of the run HUD to check with the modes-online agent.
- [x] Controls screen hierarchy (round 12).
- [ ] Player-visible text says "strong attack", never "smash" (README, How to play.txt, in-game text); the owner chose "strong attack"
      (2026-10-15). Internal move ids (`fsmash` etc.) later, with the moveset session.
- [ ] Ink painted into the stage textures (stone and plank edges, trims, strata ledges) eased toward the Wind Waker HD reference, where
      colour and value carry the scenery; the post-pass and outline-shell levers made no visible difference (art review 6 round 22).
- [ ] Mend `--demo=results` (it stops in the match) and review the results screen.
- [ ] In-motion checks for the refinement rounds (the outside brief): an attack with its hit effect, both fighters airborne, fighters
      spread so the camera pulls out; look for outline flicker, shadow crawl, effects hiding fighters.
- [ ] The run mode's own city windows (`run_stage.gd` facade values) to match the stage's calmer ones (check with the modes-online agent).

## Pain points found 2026-10-15 (end of the refinement rounds; judged against the competitors, not yet fixed)
- [x] (Sun: smaller, lower left, round 39.) Canyon: the sun is a huge bright disc right behind the stage, and the thin horizontal cloud streaks cut across it as flat lines;
      a smaller sun set lower to one side, the streaks as soft painted clouds.
- [ ] City: the building under the stage is one evenly repeated window grid (the most regular surface left in the game); vary bays,
      a few lit/curtained windows, a ledge or sign. The far billboard (top right) is a pale blank panel: give it an image or remove it.
- [ ] Beach: the sea wall's stones are still large blocks at play distance; the sky's clouds are flat white shapes with no light side.
- [ ] Meadow: the floating islands crowd the top edge of the frame (several clipped by it); fewer, placed to frame the stage.
- [ ] Grove: still the palest stage under its pink light; the light shafts and pink grade could go further if the owner wants it
      nearer the meadow (owner chose to keep the pink-purple mood).
- [ ] All stages: the stage blocks' sides are flat cut-offs at the left and right ends (no end cap or rounded edge); the reference
      stages finish their ends.
- [ ] Fighters: the clothes' outline sits close to the body outline, doubling the line at the waist and cuffs at play size.

## Tools
- [ ] Material Maker 1.7 is at `C:/Users/jared/Tools/MaterialMaker/app/material_maker_1_7_windows/material_maker.exe` (checksum
      verified). Batch export (`--export-material --target ... -o <dir> <file.ptex>`) ran but wrote nothing ("Cannot open
      user://export_targets"). Still nothing after a first windowed launch, with the exact target name (`Godot/Godot 4 Standard`, from
      nodes/material.mmg) and directory or file-prefix outputs: batch export looks broken in 1.7 on Windows. Next: export from the app
      (File > Export) by hand, or check its issue tracker / an older release. First use: painted bark and soil textures to replace the
      hand-written SVG tiles (`tools/art/make_tiles.py`).
- [ ] godot-ai MCP: configured at 127.0.0.1:8000 but nothing on this machine provides it; reinstall from its source if wanted
      (`--pick=x,y` in main.gd covers finding what draws a pixel).

- [ ] **Re-instruct the owner on the Material Maker export first thing Saturday** (open each example, File > Export > Godot/Godot 4
      Standard, save to `art/materials/mm_exports/<name>/`). Material Maker exports, by hand (owner, 2026-10-15): the owner exports `dry_earth`, `rock`, `stylized_wall` and `wood`
      (examples folder) with File > Export > Godot/Godot 4 Standard into `art/materials/mm_exports/<name>/`. Then: review them in game
      against the style; wire a PNG-tile path into `Materials._make` (it only reads the SVG tiles now), tinting through the albedo's
      value so stages keep their own colours; start with soil (dry_earth) and bark (wood/rock), compared before/after per stage. Also a palm-trunk texture (horizontal
      ring scars, no vertical streaks; the palms are plain since round 37) and tree bark the owner calls realistic, not streaky.

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
