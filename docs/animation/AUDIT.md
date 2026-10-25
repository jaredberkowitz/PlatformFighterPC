# Animation audit (Phase 1, 2026-10-25)

This is Phase 0 and Phase 1 of `docs/ANIMATION_OVERHAUL_AGENT.md`. It is inspection only: no game file was changed. It was captured
from branch `art/animation` at `origin/main` adfc46c6 (sim 146, content hash ef93425e6c64eee3).

Summary images are in `docs/animation/audit/`. The full capture set (about 1,400 PNGs: pose shots, contact sheets and whiff demos) stays
local, outside the repository. No reference footage is stored anywhere: the references were read in the browser only.

## 1. Baseline

| Sheet | What it shows |
|---|---|
| `atlas_00_longsword.jpg` ... `atlas_13_nodachi.jpg` | The pose atlas of each class (`--demo=poses --gallery-class=N --noui`). The first cell is the idle. After it, each move gets three cells, taken from its real frame data: wind-up (4 frames before the first active frame), strike (the first active frame) and follow-through (4 frames after the last active frame). A blank cell means the move has no hit frame to time the shot on. The demo dresses every class in the same default outfit. |
| `strike_frames_all_classes.jpg` | One row per class: the idle, then the strike frame of the jab, forward tilt, forward smash, neutral, forward and up airs. This is the variety check. |
| `locomotion_longsword.jpg`, `locomotion_claws.jpg` | Seven evenly spaced frames from each `tests/anim_sheet.gd` contact sheet. Rows: idle, walk, run, dash, jumpsquat, jump, fall, air jump, land, shield, roll and air dodge. |
| `reactions_longsword.jpg`, `reactions_claws.jpg` | The same layout for hurt, flinch, heavy hitstun, launch, tumble, knockdown and get-up. |
| `reach_longsword.jpg`, `reach_claws.jpg` | Whiff demos (`--demo=fx_<class>_<move>_whiff`) with the sim's own hitboxes drawn in red. Each circle is projected from `content/base.pfc` through the frame's saved camera. These sheets show where the blade or fist is against the hit. |
| `longsword_nair_frames.jpg` | The Longsword's neutral air, move frames 4, 6, 7, 8, 15, 17, 19 and 21, with the arcs and hitboxes shown. |

Not captured: facing left (the `@m` clips), a controller, and the other twelve classes' locomotion (it is shared apart from the family
idle and run).

## 2. Reference side-by-side

**Sources:**
- the reference frame-data site's hitbox animations, decoded frame by frame in the built-in browser (WebCodecs `ImageDecoder`, awaiting
  `tracks.ready` and `completed`), one gif frame per game frame (51 frames for a 51-frame move);
- the owner's combo video (`H8fBhZuywgo`, 360p, training stage with a grid), sought and drawn to a canvas.

Our frame data matches the references' 1:1. The Longsword's first active frames are jab 5, forward tilt 8 and forward smash 10; the
Claws' forward smash is 20 and its forward tilt 8/9. So both sides are compared on the same frame numbers. Frames are numbered from 1,
as on the reference site.

**How the measurements were taken:**
- **Anticipation:** frames in which the body is visibly coiled before the first active frame.
- **Extension:** how far the tip (blade or fist) is from the hitbox at the strike, in units. A fighter is about 2.34 units tall.
- **Turn:** hips and chest, judged on screen.
- **Hold:** frames the strike pose stays held after it lands.
- **Follow-through:** what the weapon and body do after the last active frame.

Our anticipation numbers come from the timing code: `strike_progress` and the six-pose clip keys in `make_rigged_blob.py` `clip()`. The
coil pose is key 19 of 60 (progress 0.317) and the first active frame is progress 0.55. So the coil is reached at about 58% of the startup
and left at once.

### Longsword (reference: the reference swordsman)

| Move | Moment | Reference | Ours | Gap |
|---|---|---|---|---|
| Jab (active 5-6) | Wind-up | Blade drawn low and back at frames 3-4: 2 frames of chamber. | At frame 1 the blade is held upright in front of the face, over the face. The chamber passes through about frame 3. | Similar length, but the chamber covers the face and reads as a salute. |
| | Strike | Arm out level with a slight bend. The blade is level and its tip is in the far (sweet) hitbox. Body side-on, front knee bent. | The tip reaches the sweet hitbox (`reach_longsword.jpg`), with the blade angled up. The body is upright. The arc is drawn beyond the tip. | Reach is good. The body is too upright, and the arc floats past the blade. |
| | Follow | By frame 10 the arm folds back and the blade lifts behind the shoulder: a retraction. | At frame 10 the arm is still dead straight at shoulder height. | **Locked arm**, no retraction. |
| Forward tilt (active 8-11) | Wind-up | A deep crouch at frames 4-6, the blade low behind. About 4 frames of coil. | An upright stance with the blade low in front of the hips. | **No coil.** The blade starts in front, not behind. |
| | Strike | A very deep lunge (back leg long, body low). The blade rises from low-forward to overhead. | The blade follows the same path, and its tip is in the hitbox at frames 8 and 10. The body only steps, staying upright. | The path matches. The **lunge depth is about half** the reference's. |
| | Follow | The lunge is held from frame 11 to beyond frame 23, blade high. | The blade ends upright in front of the face, and the body rises by about frame 15. | The hold is short, and the blade covers the face. |
| Forward smash (active 10-13) | Wind-up | The blade goes up overhead behind from frame 6. At frame 8 the body leans back and is fully coiled. About 4 frames of coil. | At frame 6 the blade is level behind at chest height and the body is upright. The overhead comes only at frame 10. | The coil is late and flat: **no lean back**. |
| | Strike | An overhead cut down in front into a deep lunge. | The blade reaches the top hitbox at frame 10 and the level one at frame 12. The lunge is deep. | Good. This is our best move. |
| | Follow | The deep lunge is held to about frame 35 (22 frames). | The lunge is held, then eases home. It is back at the rest pose about 17 frames after the last hit. | Close. |
| Neutral air (6-7, 15-21) | Whole move | The body tucks (knees up). Hit 1 slashes in front at chest height. Hit 2 is a full circle round the body, which turns with it. | The wind-up holds the blade straight down under the body (frame 4). Hit 1 sweeps from foot level up the front. Hit 2 does go right round (frames 15-21, `longsword_nair_frames.jpg`), but the body stays upright and still while the arm swings the blade. | The spin exists. It reads as a **downward poke, then an arm waving**, because the body never rolls with the blade. |
| Forward air (active 6-8) | Wind-up | In the video the body curls into a compact tuck for about 2 video frames (about 4 game frames), the blade cocked behind. | Knees up, blade raised. | Similar. |
| | Strike, follow | An overhead arc, then the body rotates forward over the cut. | An overhead arc. At the follow-through the body tips forward with the blade down. | Close, but the body rotates less. |

### Claws (reference: the reference claws fighter)

| Move | Moment | Reference | Ours | Gap |
|---|---|---|---|---|
| Idle | | A deep hunch, knees well bent, claws up in front of the chest. | Upright, knees nearly straight, fists held out to the sides. | **The biggest stance gap in the game.** |
| Jab (active 4) | Strike | A hunched swipe forward with 1-2 frames of chamber. | A straight punch with a smear. The fist reaches the near hitbox's edge, but the far hitbox (centre 1.9 forward) is about 0.8 units past the fist. | **Short of the hit.** |
| Forward tilt (active 8-10) | All | A crouched coil at frames 4-6. At frame 8 a long level strike, the body near horizontal. Held to frame 20. | A low lunge, but the fist stops about 0.65 units short of the hitbox centre, and the claw arc is drawn ahead of the fist. No coil in the wind-up. | Short reach, floating arc, no coil. |
| Forward smash (active 20-23) | All | A crouched coil from frame 12, the arm drawn back, held about 5 frames. Then a long lunge, arm fully out, held to about frame 32. | At the wind-up shot (frame 16) the pose is near neutral. At the strike the fist is about 0.75 units short of the sweet hitbox and the arc floats ahead. The free arm is pulled back. | **No coil hold, short reach.** |
| Forward air (active 7-9) | All | Both arms raised overhead at frames 3-5, then a swipe down in front. The body curls forward over it and stays curled to frame 30. | At the strike both arms are flung out sideways and the arc is drawn high and ahead, away from the hand. At the follow-through the body tips forward. | **Free arm flung wide.** No overhead chamber. |
| Neutral air (active 7-9, 10-26) | | A held double kick to the side, legs out, held for 17 frames. | A leaned-back kick (`kick_double`) with one leg forward, the other tucked. | Close in idea. The second leg does not read. |

**From the combo video** (real speed, grid stage): every frame of the reference's jumps and aerials is a different pose. Within about 6
game frames the body goes jumpsquat, then stretch, then a tucked ball with the knees to the chest, then the body pitched forward with the
legs trailing, then the overhead swing. Ours holds one pose for the whole rise and another for the whole fall (section 3, item 2).

## 3. Weaknesses, ranked (most visible first)

Each item gives what we see, why it happens (file and function), the fix, and the risk.

1. **Stance: the idle is upright and shallow, on every class, all the time.**
   - *Evidence:*
     - `locomotion_*.jpg` idle rows: knees nearly straight, and almost no change over the 12-frame loop at match distance.
     - The Claws' idle holds the fists out to the sides, where the reference crouches with its claws up.
     - The family idles (quick, heavy, caster, tool) differ only slightly (`strike_frames_all_classes.jpg`, first column).
   - *Cause:* `art/blender/make_rigged_blob.py` `main()` `idle()`:
     - the hips drop only 0.06-0.10;
     - the back leg is kept nearly straight on purpose ("a bent back knee would cross the front leg");
     - `plant()` takes over the hips' drop;
     - the bob and breath (4-6 degrees) are too small to read.
     - In `godot/scripts/anim_library.gd`, `FAMILY_CLIPS` covers only `idle` and `run`, and the brawler family (Claws, Gauntlets,
       Bracers) has no idle of its own.
   - *Fix:*
     - re-key a low stance: hips down about 0.2, both knees bent, the back knee kept behind the front one by turning it out rather than
       straightening it;
     - a slow weight shift and a breath large enough to read;
     - a guard per family: brawler crouched with claws up, quick low and springy, heavy wide.
   - *Risk:* low. Presentation only. Check the move from idle to walk, run and shield, both facings, and foot planting.

2. **Airborne: one held pose for the whole rise and one for the whole fall.**
   - *Evidence:* `locomotion_*.jpg` jump and fall rows. Fifteen frames each, all nearly the same. The reference changes pose every frame.
   - *Cause:*
     - `make_rigged_blob.py` `jump()` keys one pose family (tuck 0.62-1.0 of the same knee-up pose);
     - `fall()` only paddles 6-10 degrees;
     - `fighter_view.gd` `_choose_clip` switches hard from `jump` to `fall` on the sign of `vel.y`, and the jump clip plays by time from
       the take-off, so there is no apex tuck and no opening on the way down.
   - *Fix:*
     - key rise (stretch), apex (tuck), and descent (open, legs reaching for the landing);
     - drive the blend from the snapshot's vertical velocity (read-only, so it stays rollback-safe).
   - *Risk:* medium. Aerials start from these poses, and the air jump's flip and the landing squash read against them. `motion_test`
     must still pass (a seek redraws the same pose).

3. **Attacks have no coil hold and no snap (every class).**
   - *Evidence:* in the atlases the wind-up cells of smashes look like the idle (Longsword forward smash, Claws forward smash, Dagger's
     smashes). The reference holds its coil for 4-5 frames before the strike.
   - *Cause:*
     - `fighter_view.gd` `strike_progress` maps the startup linearly (`f / start * 0.55`);
     - with the six-pose keys in `make_rigged_blob.py` `clip()` (coil at 19, launch at 29, strike 32-36), the coil is reached at 58% of
       the startup and left at once. No frame shows a held, wound-up shape.
     - The coil poses themselves are mild (`attack_smash`: spine twist -38, small knee bend).
   - *Fix:*
     - a timing curve: reach the coil by about 45% of the startup, hold it (with a slight creep) to about 80%, then snap through the
       launch into the strike;
     - deepen the coil keys (lean back, knees loaded).
     - Only the view changes: frame data and hitboxes stay untouched.
   - *Risk:* medium. It touches every attack on every class through one function, which is high leverage. Check multihit re-strikes,
     hitlag freezes and very short startups (jabs at 3-5 frames, where a hold must not eat the chamber).

4. **Strikes read as arm-only: shallow lunges, and the turn is hidden by the camera.**
   - *Evidence:*
     - Longsword forward tilt: an upright step, against the reference's deep lunge.
     - The jab and specials are upright.
     - The chest's turn between the wind-up and the strike looks like 20-30 degrees on screen (judged from the sash; not measured).
       The jab clip authors a 64-degree chest swing before exaggeration.
   - *Cause:*
     - The `BODY_MOVES` strike poses give modest `stance` and `spine` pitch values.
     - The turn is mostly a twist about the vertical axis, which the 3/4 cheat (`BODY_TURN` 60) foreshortens.
     - `attack_turn` and `_face_camera` re-turn the chest and head toward the camera.
     - `plant()` keeps both feet on the floor, which limits the depth.
   - *Fix:*
     - carry the drive through pitch and stance depth (a lean into the cut, deeper lunges), which read side-on;
     - let the hips visibly lead by keying the launch pose earlier on longer startups.
   - *Risk:* medium. Keep the face readable (the existing rule: head within 32 degrees of the camera).

5. **Locked straight arms in follow-throughs** (pose review item 1, still present).
   - *Evidence:* Longsword jab, neutral special and side special follow-through cells; Claws jab and Dagger forward tilt follow-through.
   - *Cause:*
     - the weapon or fist arm stays IK-reached through the end lag (`_aim_arm` with `_reach_weight`);
     - the strike keys bend the elbow only a little;
     - the follow-through does not retract.
   - *Fix:* an elbow floor of about 15-20 degrees, and a retraction toward the chest after the last hit (`_reach_weight` easing out
     faster than the blade's `FOLLOW_THROUGH`).
   - *Risk:* low.

6. **Unarmed variety and weapons.**
   - *Evidence (`strike_frames_all_classes.jpg`):*
     - Popgun, Dagger and Nodachi have no overrides at all, and their strikes look the same.
     - The Dagger's forward, up and down smashes and its neutral special all play `attack_smash` (one pose four times).
     - The Dagger and Nodachi hold no weapon.
     - The Claws show bare fists: no claw is modelled (`fighter_view.gd`: "Claws just shrink it").
   - *Cause:* `anim_library.gd` `CLASS_MOVES` has no entries for classes 11, 12 and 13, and few for 7. There is no weapon art.
   - *Fix:* per-family sets (brawler, quick, heavy, tool), then each class's signature moves. Weapon art needs the owner (below).
   - *Risk:* large scope, low technical risk. `fighter_view_test` pins some mappings.

7. **Swing arcs off the weapon.**
   - *Evidence:*
     - **Longsword:** the blade does reach its hitboxes. At the strike frames of the jab, forward tilt and forward smash, the tip is in
       the sweet hitbox (`reach_longsword.jpg`), so the 10-24 audit's cause is wrong for the sword. Its arcs still float past or beside
       the blade, most on the jab and the tilts' follow-throughs.
     - **Claws:** the fist stops 0.65-0.8 units short of the hitbox centre, and the claw arcs are drawn 0.8 past the hand on purpose.
   - *Cause:*
     - `fighter_view.gd` `ARCS` are fixed circles about a fixed centre (for example (0.5, 1.15), radius 2.3) drawn at `ARC_Z` 0.6 in
       front, while the blade pivots at the moving hand of a lunging body;
     - claw arcs are `arm_reach + ARC_PAST_HAND`;
     - nothing is drawn where the claws would be.
   - *Fix:*
     - Phase 2 step 3 as planned: trail the actual tip;
     - for the Claws, either reach the fist further (stretch plus IK) or add a claw mesh so that the hand plus claw meets the hit
       (owner's call).
   - *Risk:* low to medium. The owner tuned the arcs' size and opacity before (2026-10-21, art review 6).

8. **The Longsword's neutral air reads as a poke, then a wave.**
   - *Evidence:* `longsword_nair_frames.jpg`, plus the atlas strike cell.
   - *Cause:*
     - the wind-up holds the blade straight down;
     - hit 1 starts at foot level (the sim's first hitbox is at y 0.16, a matter for the moveset session, not this pass);
     - the windmill is the arm alone round an upright, still torso (`attack_nair` keys, with `_air_counter` giving little pitch for a
       front-and-round arc).
   - *Fix:* tuck the knees and roll the body with hit 2 (the reference's body turns with the circle), and start the wind-up behind
     rather than straight down.
   - *Risk:* low.

9. **Reactions are better, but static.**
   - *Evidence (`reactions_*.jpg`):*
     - the hurt pose now recoils, and the sword hangs at the side (pose review item 6 is fixed), but it holds still after 18 frames;
     - heavy hitstun and launch are each one held pose;
     - the launch is no longer a stretched tube (item 7 is fixed);
     - the tumble is a rigid whole-body rotation.
   - *Cause:*
     - `make_rigged_blob.py` `hurt` is an 18-frame clip that then holds;
     - `hurt_heavy` and `hurt_up` are static;
     - the tumble spin is code (`tumble_angle`).
   - *Fix:* a slow reel loop over long hitstun, the pose scaled by knockback, and limbs that trail and flail in the tumble.
   - *Risk:* low.

10. **Smaller items** (pose review items 8-10 re-checked):
    - The shield is still upright with the sword across the chest (item 9).
    - The Claws' forward air free arm is still flung wide (item 2).
    - The arched-back up moves (item 3) and head-below-hips strikes (item 5) are largely fixed on classes 0 and 1. The other classes
      were checked at thumbnail scale only.
    - Squash on jumps and landings: the land row squashes. The take-off stretch and the springs could not be judged in contact sheets
      (`anim_sheet` renders from rest). Check them in a live match.

### The pose review's list, re-checked

| # | Item (2026-10-15) | Now |
|---|---|---|
| 1 | Locked straight arms | **Still present** (item 5 above) |
| 2 | Free arm flung wide | Still present on the Claws' forward air. Other strikes are better. |
| 3 | Arched back on up moves | Largely fixed (the head is clamped at 8 degrees) |
| 4 | One pose set for nine classes | Partly fixed (Needles, Embers, Gauntlets, Clutch, Bracers and Pockets have overrides). Popgun, Dagger and Nodachi have none. |
| 5 | Head below the hips | Fixed on classes 0 and 1 |
| 6 | Hurt pose | The pose is fixed. It is now **static** (item 9). |
| 7 | Launch pose | The tube is fixed. It is now **static**. |
| 8 | Jump | **Still one pose** (item 2) |
| 9 | Shield | Still crosses the chest |
| 10 | Hands and legs | Fists keep their size on the grip. Not re-checked closely. |

## 4. Proposed order

The brief's order, adjusted by what the audit found:

1. **Stance per family** (item 1). It is on screen every second, and on the reference claws fighter the gap is the largest.
2. **Air arc** (item 2): rise, tuck, open and fall, blended by vertical velocity. Then the landing squash and the take-off stretch.
3. **Attack timing curve** (item 3), one shared change in `strike_progress` plus deeper coil keys. Every class gains at once. Show it on
   the Longsword's forward tilt and forward smash and the Claws' forward smash before and after.
4. **Longsword attacks against the reference** (items 4, 5 and 8): jab (no face cover, retraction), forward tilt (deep lunge, held),
   neutral air (body roll), forward air, then the rest.
5. **Brawler family, Claws first** (items 6 and 7): stance, coil, reach to the hitbox, and the forward air's overhead chamber. The claw
   decision comes first (below).
6. **Arcs to the tip** (item 7), class by class, as each weapon's swing is right.
7. **Reactions** (item 9).
8. **The remaining classes**, Popgun, Dagger and Nodachi last (they need their own sets, and two need weapon art).
9. **Secondary motion and polish.**

After each family or class: a build for the owner, as in the brief.

## 5. Needed from the owner

- **Approval of this order** before Phase 2.
- **Claws:** should the Claws class show claws (a small mesh on the fist), or should the fist simply reach further? Either way the hand
  and the arc must meet the hitbox.
- **Weapons for the Dagger and Nodachi:** model them (Blender script, as the other props), or keep them unarmed for now?
- **Cascadeur:** not needed for the next steps. The stance, air arc and timing curve are script-keyed, with the live Blender connection
  for grey renders. It may help later for the reactions and tumble (physics-plausible flails). Ask again then.
- **Note for the moveset session (no action now):** the Longsword's neutral air hit 1 starts at foot height (frame 6 hitbox, y 0.16).
  The reference's first slash looks chest-high in its hitbox animation. Worth a check against the source data. This pass will not
  change it.
