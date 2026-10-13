# Animation: one body, many movesets

How the fighters move, how the clips are organised and shared, and the style every clip follows. Presentation only: animation never drives
the simulation (the sim's frame data is the only clock). Sister documents: `ART_DIRECTION.md`, `ART_PIPELINE.md`, `VISUAL_STYLE.md`,
`COMBAT.md`, `MOVESETS.md`.

## 1. Architecture (audit, 2026-10-11)

- **Engine**: Godot 4.7.1, Forward+. One `AnimationPlayer` per fighter, in manual process mode, driven every frame by
  `scripts/fighter_view.gd` from the sim snapshot (`main.gd` `_refresh`). No AnimationTree. Cross-fades are 0.1 s; attack clips are posed by
  seeking (no fade), so a strike lands exactly on its frame.
- **Rig and clips**: one skeleton (`root, hips, spine, head`, and per side `thigh, shin, foot, armU, armL, hand`), built with every clip by
  `art/blender/make_rigged_blob.py` (keyed procedurally: poses are code, so a change is reviewable and re-runnable). The base body is
  `godot/models/base_rig.glb` (`--body`); `blob_rig.glb` and `blob_rig_long.glb` are the older built bodies (`--blob`). The model is read
  once and copied per fighter, so **every fighter shares one clip library**; nothing is duplicated per moveset.
- **Rebuild**: `"C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/make_rigged_blob.py -- --body
  art/models/base_body/base_body.glb --joints art/models/base_body/joints.json --neck-drop 0` (about 20 s; prints the clip list).
- **Mapping**: `scripts/anim_library.gd` (the library): shared state clips (`STATE_CLIPS`), shared attack archetypes by move name
  (`SHARED_MOVES`), and per-class overrides (`CLASS_MOVES`) for moves whose movement genuinely differs. A missing mapping or a clip missing
  from the rig warns once and plays the fallback; clips added later have stand-ins for older rigs (`STAND_INS`).
- **Attack timing**: every attack clip uses one timeline (wind-up by frame 21, strike at 33, follow-through to 45, settled by 60), played
  by progress through the move's own frames (`fighter_view.gd` `strike_progress`): the strike lands on the first active frame, follows
  through while active, holds through end lag and settles at the end. So one clip serves moves of any length without changing their frame
  data, and the animation can never become the source of truth.
- **Timed states**: the dodges and the stand-up from the floor are played by progress through their own lengths (the bridge's
  `fighter_dodge`: the state's length from the fighter's parameters, and which way it goes), so a fighter with longer rolls still lands
  its roll's poses on its own frames. The roll's somersault and the spot dodge's step back into the stage are code on top.
- **Multihit**: the bridge exposes each move's separate hits (`fighter_move_windows`: one window per hitbox group, cut again at every
  re-hit of a `rehit` move). Between two hits the body draws back toward the wind-up and strikes again on the next hit's first frame;
  back-to-back hits take the draw-back on the frame just before, so the impact frame (frozen by hitlag) always shows the strike. The last
  hit gets the full follow-through.
- **Layers on top of the clip** (code, `fighter_view.gd`): the 3/4 body turn and chest-to-camera twist, the weapon arm's IK onto the live
  hitbox (`_aim_arm`), the kicking leg's IK (`_aim_leg`, kick clips only), the body bending to let a limb reach (`_body_follow`), squash and
  lean springs per state, the midair flip, tumble and launch poses, the flinch, ledge travel. These never touch hitboxes, hurtboxes or
  collision: those live in the sim.
- **Motion polish layers** (`docs/MOTION_POLISH_PLAN.md`), each in its own module:
  - *Deformation* (`motion_deform.gd`): the striking fist or leg stretches on the strike (volume kept; the IK reaches that much further,
    so the tip still lands on the hitbox); the victim is squashed through the freeze of a launching hit and released into a stretch along
    its flight. All of it is worked out from the move's frames, hitlag and velocity.
  - *Smears* (`smear.gd`): a punch or kick with no authored arc leaves a ribbon from its wind-up to the hit, in the class's trail colour,
    drawn into the swing arc's mesh.
  - *Springs* (`motion_springs.gd`): the hat, the chest and head lagging the hips, the head lagging a turn, a fist's follow-through, the
    neckerchief, and cloth tails on spring chains (`tail_rig.gd`: the scarf's end and the sash's end). Stepped with the sim frame: frozen in
    hitlag and while paused, at rest at once after a rollback or a replay seek (a backward jump, or one of more than two frames).
    `reset_motion()` puts everything back to rest on a rebuild.
  - Bones a clip leaves unkeyed (the exporter drops a track whose bone stays at rest) are put back to rest each frame before the turns and
    bends go on top (`_rest_unkeyed`); without it a long shield hold spun the head round.

## 2. Inventory

| Class | Clips |
| --- | --- |
| A. Shared, ready | `walk`, `run`, `dash`, `skid`, `fall`, `air_jump`, `crouch`, `shield`, `ledge`, `ledge_climb`, `grab`, `throw`, `victory_a/b/c` |
| B. Shared, polished in the first slice | `idle`, `jump`, `hurt` (now `jumpsquat` and `land` too) |
| B. Shared, polished in the second slice | `roll` (crouch, somersault, open out), `spot_dodge` (new: duck and tuck, stepped back into the stage), `air_dodge` (new: curl, then open into the fall), `knockdown` (slam, then flat), `getup` (new: push-up, crouch, stand) |
| B. Shared, polished in the motion pass | `shield` (a breath and a knock-back when hit, code layers), `run` and `dash` (a short-contact stance that does not skate) |
| B. New in the motion pass | `tech` (an intangible landing: a slap and a pop up), `platform_drop` (the first frames after dropping through a platform), `run_stop` (still sliding as a run stops), `hurt_heavy` and `hurt_up` (a fast launch that does not tumble, sideways or upward), `idle_look` (a fidget in a long idle) |
| C. Archetypes used by several moves | `attack_jab`, `attack_fair`, `attack_uair`, `attack_swing`, `attack_lunge`, `attack_smash`, `attack_low`, `attack_nair`, `attack_bair`, `attack_dair`, `attack_kick`, `grab`, `throw`, `sword_dsmash` |
| D. Specialised, keep unique | `sword_ftilt`, `sword_utilt`, `sword_dtilt`, `sword_fsmash`, `sword_usmash`, `sword_dair`, `kick_up`, `kick_low`, `kick_dash`, `kick_uair`, `kick_nair`, `kick_bair`, `attack_claws`, `blaster` |
| E. Unused | none (the claws' down air now plays its stomp, `kick_dair`) |
| F. Missing | none from the earlier list. Still rough: the clips were re-keyed on the base body's joints but not re-posed for its proportions |
| G. Not reusable as is | the `BODY_MOVES` poses are for the base body only; the old blob rigs have no `jumpsquat` or `land` (stand-ins play `crouch`) |

## 3. Which moves use which clip

Shared by both movesets (Longsword, Claws): `attack_jab` (jab, jab 2, jab 3), `attack_fair` (fair), `attack_uair` (uair), `grab` (grab,
dash grab, pivot grab, pummel), `throw` (all four throws), `attack_swing` (get-up attack; and the script-only follow-ups such as Dancing
Blade's later hits).

| Clip | Longsword | Claws |
| --- | --- | --- |
| `attack_uair` | uair, up special | uair |
| `attack_lunge` | dash attack, side special | |
| `attack_smash` | neutral special | fsmash |
| `attack_low` | down special (the counter stance), ledge attack's sweep | ledge attack's sweep |
| `attack_nair` / `attack_bair` | nair / bair | |
| `attack_dair` | | (none: the claws' dair plays `kick_dair`) |
| `sword_dsmash` | dsmash | dsmash (a claw swept in front then behind) |
| `attack_kick` | | side, up and down specials |

The Needles moveset (class 2) maps its kicks in `CLASS_MOVES[2]`: `kick_up` (ftilt, utilt), `kick_low` (dtilt), `kick_nair`, `kick_bair`,
`kick_uair` (uair, usmash), `kick_dair` (dair; its landing hit, `extra move 0`, plays `land`) and `kick_dash` (dash attack). The forward air
keeps the shared `attack_fair` chop. Its stand-in specials use the claws' clips until the real ones are built.

## 4. Style guide

The look is physical and loose between actions and decisive in them.

- **Weight**: every action has a compression before it and a release after it. The jump crouches with the arms swung back (`jumpsquat`)
  and throws them up through the push-off; the landing drops the weight onto bent knees with the arms flung out, rebounds and settles.
- **Silhouette first**: a pose must read as a shape at the match camera's distance. Change the shape, not only the angle: the forward air
  tucks compact, then opens long (front knee up, back leg kicked out, free arm flung back). Never put a limb across the face: the drawn
  face is half of every read (the jump raises the far arm, not the near one; the hit reaction keeps the face toward the camera).
- **Anticipation and follow-through**: the shared move timeline keys the wind-up pushed past its pose and the strike overshot
  (`amped`), and the game holds the follow-through through end lag. Attacks get more exaggeration than locomotion (`clip`'s `AMP`).
- **The body drives the hit**: shoulders and hips wind back, then turn through as the weight steps onto the front foot; the free arm
  counterbalances.
- **Restraint at rest**: the idle bounces and breathes (a slower breath under the bob, the head settling a beat late) but never wobbles.
  Springs, tails and smears all settle to nothing standing still.
- **Stretch and smear**: only on the frames that are fast: the strike (a fist or leg up to 22% longer), the launch, the throw of a punch or
  kick. Never across the face, never on a pose that is held.
- **Hit reactions**: snap (chest thrown back off the blow, arms flung out as the body leaves them, one knee jerked up), then slacken over
  the stun. Strength and direction come from code on top: a flinch for hits too weak to launch, tumble and a head-first launch along the
  real trajectory.
- **Same everywhere**: locomotion, jumps, landings, defence and hit reactions never change with the moveset.

## 5. Adding or changing a clip

1. Pose it in `make_rigged_blob.py` (a `BODY_MOVES` entry for an attack on the shared timeline, or a function keyed with `clip`), and add
   ground clips that keep both feet down to `STANDING`.
2. Rebuild the rig (above) and check it with a contact sheet: `Godot --path godot --script res://tests/anim_sheet.gd -- --what=<clip or
   move> --class=<0|1> --out=<png>` (`--timing` and `--tip` for attacks).
3. Map it in `anim_library.gd`: a shared archetype if the movement serves several moves, a class override only when a move genuinely moves
   differently. Never copy a clip per moveset.
4. `tests/anim_library_test.gd` checks every mapped clip is in the rig, every move of every moveset has a clip, overrides differ from the
   shared clip, and multihit strikes land per hit.

## 6. Plan

Done (first slice): the library and its test, per-hit strikes for multihit moves, the idle, the jump's take-off and rise, the landing, the
jab, the forward air and the hit reaction. Done (second slice): roll, spot dodge and air dodge as their own clips played on the sim's
frames, knockdown and get-up, and the claws' down air on its stomp. Done (motion polish, `MOTION_POLISH_PLAN.md`): deformation, smears,
springs and tails, the missing clips, the shield, locomotion slide. Checks: `tests/motion_test.gd` (rollback, hitlag and rest behaviour of
every layer) and `tests/motion_probe.gd` (foot slide). Next:

1. The body is now the cuter one (`ART_PIPELINE.md` thirty-first pass). Every clip was rebuilt on it and the worst reads fixed (the
   head never tips back past 8 degrees, overhead kicks keep the face to the camera, kicking legs stretch to reach); a pose-by-pose pass
   for its proportions is still open.
2. The walk still slides about 0.45 of its ground speed: short legs cannot plant at these walk speeds without scurrying.
