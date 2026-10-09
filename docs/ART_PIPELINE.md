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
legs with shoes) and its animation clips, and exports `godot/models/blob_rig.glb`:

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
`attack_jab`, `attack_lunge`, `grab`, `throw`) set the body around it: a forward air folds the torso forward through the sweep, a back air turns the body away, a neutral
air spins, and so on. Clips are timed to the move (wind-up until the first hitbox, strike through the active frames, then recovery).
`roll`, `knockdown` and `ledge` clips cover rolls and dodges, lying down and hanging.

**The brawler is long-limbed.** `make_rigged_blob.py -- --long` stretches the legs (x1.5) and arms (x1.4) in the rest pose and lifts the body to match,
exporting `godot/models/blob_rig_long.glb`; the game scales it by 0.92 to keep the same height. `FighterView` swaps to it when it first sees
a brawler (`long_limbs`), and the head, neckwear and arm reach follow. (The class check is `char == 1`, so a made fighter that is not index 1
uses the normal rig for now.)

**The brawler fights with fists** (no weapon is drawn). Its punches and sweeps use the same arm IK: the fist is the hand at the end of the
reach, aimed through the move's arc. Its kicks (neutral air, back air, up air, down air, tilts, specials) use the `attack_kick` leg clip.

**Neckwear** (sash, neckerchief, scarf) is built for the rigged torso (`_neck_on_rig`): rings sized to hug the body's ellipsoid, so nothing
pokes through in any pose. (The old flat boxes are only used when there is no rig.)

**Walk, run and dash** follow a sprint-style cycle (`walk_pose` in the Blender script): the leg swinging forward lifts its knee while the
planted leg is nearly straight; arms pump opposite the legs with the elbow bent more as it comes forward; the body leans into the run,
bounces once per step and the shoulders counter-twist against the hips; the head stays level-ish. `walk` (40 frames, light), `run`
(20 frames, 20 degree lean, big knee lift) and `dash` (16 frames, 28 degree lean) play at a speed set by the fighter's real speed.

**Readability.** Move clips are keyed about 55 percent bigger than natural (`AMP` in the Blender script; locomotion about 20 percent, hurt and
air poses 25 percent), and the body coils down while a move winds up and stretches tall through the strike (`apply`). Each attack leaves a
**crescent trail** behind the hitbox (`_update_trail`): every simulation frame the hitbox's centre (the fist, or the part of the blade that
hits) is added to a path, and the path is drawn as a smooth curve (Catmull-Rom) thickest at the hitbox and tapering to nothing behind it,
with a bright core inside a coloured edge (violet for the brawler, gold for the sword). While the hitbox is live a thin ring marks exactly
where it is. The points are in world space, so the trail stays where the swing was. It is cosmetic and only reads the move's timing and
hitbox.

**Which clip a move plays** (`_choose_clip` in `fighter_view.gd`; `fighter_view_test.gd` pins it): jabs `attack_jab`; dash attack and side special `attack_lunge`;
up tilt and up special `attack_uair`; down tilt and down special `attack_low`; neutral special and smashes `attack_smash`; the aerials their own; the claws
fighter's kicks and rushes `attack_kick`; everything else (forward tilt, ledge attacks) `attack_swing`.

**The maul** (third class) draws a hammer instead of the blade: a shaft that stretches to the move's reach with a fixed-size steel head at the sweet spot, and a red trail.

Known gaps: clips are per move *type*, not per move (forward tilt and the ledge attacks still share `attack_swing`, and every special of a class shares one of the above); the arc is only as sweeping as the move's
hitbox data (a move with one static hitbox gets the wind-up sweep but a still strike); the brawler's kicks use one generic kick clip;
feet still slide a little at game speed (the cycle is paced by speed, not locked to the ground); none of it has been judged by someone playing.

## Rules

* Cosmetic only: gameplay hurtboxes come from content, never from a mesh (`CLAUDE.md`).
* Original shapes only; keep the poly budget small (the whole set is about 140 KB).
* Change a shape: edit the script, rerun Blender, commit both the script and the `.glb`.

## Next steps for art

1. Aim the sword arm at the blade; clips for grabs, throws, rolls, knockdown, ledge and each smash/aerial.
2. Move the hat, glasses, neckwear and sash into the Blender script as meshes on the rig, then drop the code-built versions.
3. A face atlas texture and expression set for the head.
4. Stage backdrops, hit effects and a shield bubble.


## One clip per move (third pass)

Clips added for the moves that shared a generic one: the sword fighter's **forward, up and down tilts** and **forward, up and down smashes**
(`sword_*`), and the claws fighter's **neutral, back, up and down airs**, **up and down tilts**, **dash attack** (`kick_*`) and the
**blaster** (`blaster`). Each sets the body (weight, torso turn, free arm, other leg) on the shared move timeline, which the game stretches
to the move's own frame data, so wind-up, strike and recovery line up with the hitboxes. The weapon arm is still aimed at the live hitbox,
and now **the kicking leg is too** (two-bone IK from hip to ankle, `_aim_leg` in `fighter_view.gd`), so a kick's foot is where its hit is.

These are original animations made on our own rig, timed to the moves' published frame data and shaped after how the archetypal moves read
(a rising cut, an overhead arc, a spinning kick); nothing is taken from another game's files. Demos: `--demo=moves` (sword tilts and
smashes) and `--demo=kicks` (claws kicks), with `--stage=N --noui --noecb --shots=<folder>`.


## Victory poses, clothes, and checking against the reference (fourth pass)

- **Clothes**: the rig now has shorts (the outfit colour, the loadout's accent a shade darker) over the bottom of the body and the tops of
  the thighs, white soles under the shoes and a collar; the sash carries two badges; the eyes have catch-lights and the cheeks a touch of
  blush; every cel material has a soft rim light.
- **Victory poses** (`victory_a` weapon raised with a hand on the hip, `victory_b` a fist pump, `victory_c` a cheering hop), played by
  `FighterView.play_victory` on the results screen: a cheer first, then the class's own pose, with the sword or maul held up in the raised hand.
- **Reference check**: the public hitbox visualisations on ultimateframedata.com (the moves' frame data was already taken from there) were
  looked at for the poses of the forward and back airs: the forward air is now a mid-air crouch with the knees pulled up and the torso curling
  over the swing, and the back air tucks the front knee and trails the back leg. The claws fighter's up smash is a flip kick (`kick_uair`
  clip, leg aimed at its hitbox). These remain our own animations on our own rig; nothing was copied from the reference's files.
