# Art pipeline (Blender to Godot)

Blender 5.2 is installed (`winget install BlenderFoundation.Blender`; `C:\Program Files\Blender Foundation\Blender 5.2\blender.exe`).
Models are **made by script** so they are reproducible, reviewable in git and original. Style target: `docs/ART_DIRECTION.md`.

## Blob parts

`art/blender/make_blob_parts.py` builds the fighter's body parts (Body as a pear, Head, FootL/FootR with a flat sole and toe cap, HandL/HandR
mittens with a thumb) and exports `godot/models/blob_parts.glb`:

```
"C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" --background --python art/blender/make_blob_parts.py
```

Each part is modelled at the size and centre the game used for its old sphere, so `FighterView` (`godot/scripts/fighter_view.gd`,
`blob_parts()`) swaps the mesh in place and only chooses the colour. The glTF is read at run time (no editor import needed); if the file is
missing or damaged the fighter falls back to plain spheres. The face, hat, glasses, neckwear, sash and weapon are still built in code.

## The rigged blob (arms, legs, animation)

`art/blender/make_rigged_blob.py` builds the character with a 16-bone skeleton (hips, spine, head, two-segment arms with mitten hands, two-segment
legs with shoes) and 11 animation clips, and exports `godot/models/blob_rig.glb`:

```
"C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" --background --python art/blender/make_rigged_blob.py
```

* Parts are rigid (each weighted to one bone), like a vinyl toy. Colours come from the game by part name (skin, white gloves and socks, dark shoes).
* Clips: `idle`, `walk`, `run`, `jump`, `fall`, `crouch`, `shield`, `hurt` and three attacks driven by the move's progress, `attack_swing`,
  `attack_low`, `attack_kick`. They are keyed in code from "forward / outward / twist" angles, so they are easy to tune and re-export.
* `FighterView` (`_build_rig`, `_choose_clip`, `_animate`) reads the file once and copies it per fighter, picks the clip from the simulation's
  state (walk and run speed follow the fighter's real speed; an attack's clip is seeked to `state_frame / total_frames`, so the swing lands
  on the move's own frames), and freezes the pose during hitlag. The face, hat, glasses and neckwear are attached to the head and spine bones
  through follower nodes (`head_rig`, `torso_rig`). The squash, lean and spin effects still apply on top.
* If `blob_rig.glb` is missing the fighter falls back to `blob_parts.glb`, then to plain spheres.
* Animation never affects the simulation: it only reads the state.

**Swings.** The blade's tip is fixed by the move's hitboxes (while one is live, the tip is where that hitbox is now: `fighter_move_tip`).
In an attack the hand leaves its resting place and sweeps round the shoulder, and the arm reaches it with two-bone IK
(`_pose_blade`, `_aim_arm`), so the arm throws the blade through its arc and the blade always ends at the move's tip. Move clips
(`attack_fair`, `attack_bair`, `attack_nair`, `attack_uair`, `attack_dair`, `attack_smash`, `attack_swing`, `attack_low`, `attack_kick`,
`grab`, `throw`) set the body around it: a forward air folds the torso forward through the sweep, a back air turns the body away, a neutral
air spins, and so on. Clips are timed to the move (wind-up until the first hitbox, strike through the active frames, then recovery).
`roll`, `knockdown` and `ledge` clips cover rolls and dodges, lying down and hanging.

Known gaps: one clip per move type, not per move (every jab and tilt shares `attack_swing`); the arc is only as sweeping as the move's
hitbox data (a move with one static hitbox gets the wind-up sweep but a still strike); the sash and neckwear were designed for the old
sphere body and are only roughly fitted; the brawler's kicks use one generic kick clip; none of it has been judged by someone playing.

## Rules

* Cosmetic only: gameplay hurtboxes come from content, never from a mesh (`CLAUDE.md`).
* Original shapes only; keep the poly budget small (the whole set is about 140 KB).
* Change a shape: edit the script, rerun Blender, commit both the script and the `.glb`.

## Next steps for art

1. Aim the sword arm at the blade; clips for grabs, throws, rolls, knockdown, ledge and each smash/aerial.
2. Move the hat, glasses, neckwear and sash into the Blender script as meshes on the rig, then drop the code-built versions.
3. A face atlas texture and expression set for the head.
4. Stage backdrops, hit effects and a shield bubble.
