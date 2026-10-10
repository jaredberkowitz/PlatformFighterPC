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
3. ~~A face atlas texture and expression set for the head.~~ Done: drawn SVG faces on a face shell (fifth pass).
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


## Lighting, shaders and drawn artwork (fifth pass)

The look now comes from lighting, shaders and authored artwork rather than from shapes built in code:

- **Lighting** (`godot/scripts/lighting.gd`, shared by the match and every menu preview): a warm key light with soft shadows, a cool fill
  from the other side, warm ambient light, a filmic tone curve, a gentle bloom, ambient occlusion in the match and a little extra saturation.
- **Shaders** (`godot/shaders/`): `toon.gdshader` is the soft cel material for every character surface (a colour, optionally times a tiled
  texture); `face.gdshader` lays the face drawing over the skin colour; both light through `toon_light.gdshaderinc` (light wraps round the
  form and steps softly into a cool shadow tint, a warm rim on the lit side, a small soft sheen).
- **Faces** (`godot/art/faces/<expression>.svg`): one drawing per expression (`deadpan`, `sleepy`, `grumpy`, `smug`, `hurt`, and `blink`,
  shown for a moment every few seconds). They are shown on a **face shell** in the rig (`Face`, a thin layer just in front of the head,
  following the head bone), whose UVs are a straight-on projection of a 1.155-unit square centred at height 1.585, so a drawing lands on the
  head as drawn (eyes at about y = 228 of 512). The upper lids are filled with the placeholder colour `#fe00ff`, which becomes the fighter's
  colour a shade darker when the face loads; anything else is drawn as is. Edit them in any vector editor.
- **Clothes**: shirts are a loadout slot (`white`, `outfit` colour, `striped`, `flower` print), worn on the body (which now has globe-style
  UVs) with short **sleeves** (`Sleeve.L/R` in the rig, open tubes over the upper arm, hidden without a shirt). Prints and the straw weave are
  tiles in `godot/art/cloth/` with `#ff00ff` as the placeholder for the shirt (or straw) colour. The sash is a flat band with three badges.
- The SVGs are kept as files (their `.import` says `keep`) and drawn into textures when first used (`godot/scripts/svg_art.gd`), so the
  colours can be swapped; a texture is made once per drawing and colour.
- **Checking**: `godot --path godot --script res://tests/lineup_shot.gd -- --out=<folder>` photographs the four default fighters close up
  (`lineup.png`) and every face and shirt (`lineup_faces.png`).


## Animated run, bigger attacks and expressions (sixth pass)

- **The run** (`run_pose` in the Blender script) is a key-pose cartoon sprint instead of a sine wave: a strong forward lean, the knee driven
  high in front, the heel kicked up behind, arms pumping wide with bent elbows, the body dropping as each foot lands and springing up between
  steps, shoulders twisting against the hips and the head nodding with each step. The dash uses the same stride launched lower and harder.
  In the game each footfall squashes the body a little (and each stride stretches it) in step with the clip, and kicks up a puff of dust.
  The sword and the maul are carried swept back behind the runner while the arm pumps.
- **The skid** (`skid` clip, for a turn out of a run): leaning back on a planted heel, arms flung back, with a stream of dust. The fighter keeps
  facing the way it was running until the skid ends (the simulation turns it at once; this is display only).
- **Attacks**: every move clip winds up past its pose (anticipation, x1.12), strikes, overshoots the strike (follow-through, x1.12) and
  settles; attack clips play larger overall. In the game the body is drawn back in the late wind-up and thrown forward through the active
  frames (a lunge), on top of the squash and stretch it already had.
- **Expressions in action** (`godot/art/faces/`): `attack` (a fierce yell, from the start of the swing to just after the hit), `effort`
  (gritted teeth and a sweat drop: shielding, hanging from a ledge, charging a smash, holding a grab), `focus` (running, dashing, jumping,
  dodging) and `happy` (a closed-eyed grin, on the results screen). The fighter's own face comes back when it is idle; blinks only show on
  the calm faces.
- **Details**: rolled glove cuffs at the wrists, a strap across each shoe in the outfit colour, and the hat on a spring: it lags behind a
  sudden start or stop and bounces on landings and jumps.
- **Fixed on the long-limbed rig**: the soles and the shorts' legs now stay on the feet and thighs (they were lifted with the body).
- **Checking motion**: `godot --path godot --script res://tests/anim_sheet.gd -- --what=run --class=0 --out=<file.png>` renders a contact
  sheet, frame by frame, of the run, dash, skid or any move (`--what=fsmash`, `nair`, ...).


## The body follows the limb (seventh pass)

The rule from here on: **when a limb reaches for a hit, the rest of the body moves with it.** The kicking leg (claws fighter) and the
weapon arm are aimed by the game at the move's live hitbox; the rest of the body now answers that, for every move, on top of its clip
(`_body_follow` in `fighter_view.gd`):

- The **torso bends at the waist** so the limb can get there: back for a kick high overhead (the up tilt leans back on the standing leg), forward
  over a kick behind, into a low swing in front. A leg is taken to swing comfortably from about 40 degrees behind to 100 in front; beyond
  that the body makes up the difference (capped). Arms reach almost anywhere, so they only add a small lean.
- The **head looks** up at a hit overhead and down at a low one.
- It **grows and fades with the move**: the limb eases toward its hit through the second half of the wind-up, is on it while the hit is live,
  and comes home through the first half of the recovery (`_reach_weight`); the body follows the same weight. (Before, a kicking leg stayed
  stretched toward the hit until the move ended.)
- The **leg IK** is rooted at the animated hip (not its rest position), and the **knee bends the way a knee does**: ahead of the leg in the
  plane of the kick (up for a kick in front, down and back for one behind), so it never folds backwards.

Clips touched to agree with it: the claws fighter's up tilt is now a high kick leaning back with the arms thrown out for balance; the sword
down tilt and the back air lean less (the body follow adds the rest). Check a move with its real hitbox:
`--script res://tests/anim_sheet.gd -- --what=utilt --class=1 --tip=1.1,3.5 --timing=34,7,11 --zoom=1.4 --out=<file.png>`.


## Turned for the camera, and a fighting stance (eighth pass)

- **How a fighter is turned** (`BODY_TURN`, `CHEST_TO_CAMERA`, `HEAD_TO_CAMERA` in `fighter_view.gd`): the classic cheat for 3D fighters on a
  2D stage. The hips and legs are turned 60 degrees toward the way the fighter faces, so strides, kicks, lunges and leans happen across the
  screen instead of toward the camera (where they were foreshortened); the chest then turns 30 degrees back toward the camera and the head 20
  more, so the body and the face are open to the player (the chest about 30 degrees off straight-on, the face about 10). Before, the whole
  body was turned 36 degrees and every forward motion went mostly into the screen. Leaning is now a pitch toward the fighter's front.
- **The stage frame**: a node in the model that undoes its turn, so everything aimed at a hit (the weapon, the arm and leg reaching, the
  body following them) still works in stage axes. The arm reach is now rooted at the animated shoulder, like the leg's at the hip.
- **Idle** (in the manner of a plumber's bouncy stance): feet apart with the far foot forward and the back leg nearly straight, knees
  bent, the weight bobbing down and up twice a cycle, fists up in front of the chest (the lead fist further out), the chest leaning in and
  turned with the lead shoulder, the head nodding with the bounce.
- **Walk**: a bouncy, swaggering step: knees lifting, a springy bob on each step, arms swinging wide with loose bent elbows, the shoulders
  twisting against the hips, a little lean.
- Menu previews keep their own turn (they set it themselves).
- **Looking at each other** (after the reference of how fighters stand in the reference game): the body stays open to the camera, but the head
  turns back toward the opponent (`HEAD_TO_CAMERA` is negative: the face about 40 degrees off straight-on) and every face drawing has its
  pupils toward the fighter's front (`GAZE` in the drawings, texture right). Facing left, the face shader mirrors the drawing (`mirror`,
  `FighterView.set_gaze`), so both fighters look at each other; the switch happens as the body swings past facing the camera. Menu
  previews look the way they are turned.


## Swing trails and iconic aerials (ninth pass)

- **Sword (and maul) trails** are now the reference game's kind: the whole crescent the blade sweeps, blue-white for the sword (orange-white for
  the maul), brightest along the edge the tip traces and fading toward the hilt and with age (8 frames), drawn in thin slices that turn round
  the hand so the arc is smooth, and frozen during hitlag (`_update_sweep`). The claws keep the hitbox ribbon, which now only draws when the
  hit travels across the body (a held kick no longer leaves a streak as the fighter falls).
- **The swing direction**: a swing's wind-up starts from where its first hit is, turned back by `SWING_FROM` for the moves that sweep top to
  bottom (forward air from behind the head, down air from high in front), and after its last hit the blade stays where the swing ended
  (the bridge reports the last hit's place once the hits are over) instead of snapping back.
- **Forward air**: wind-up behind the head, then the crescent from overhead through level to low in front.
- **Down air** (`sword_dair`): knees gathered and the sword raised in front, then the legs open wide, torso upright, head looking down as
  the crescent sweeps under from front to back.
- **Claws neutral air** (`kick_nair`, `SPLIT_KICKS`): both legs are aimed by the game, one kicked out ahead and one behind, the torso upright
  over the split with the arms thrown up, held through the long late hit.
- Close-up demos that photograph every frame: `--demo=marth_fair_close`, `marth_dair_close`, `wolf_nair_close` (with `--noui --noecb
  --shots=<folder>`).
- **Sword swing directions** (tenth pass): `SWING_FROM` now covers every sword move, so each blade comes from the right side (cuts that come
  down wind up from above, rising cuts from below); the resting blade is shorter to match the shorter reach; the move tip the game aims at is
  the hit farthest from the body (the end of the blade). `--demo=sword_gallery` photographs every sword move at wind-up, first hit, middle
  and recovery.


## Readability and finish (eleventh pass)

- **Chunkier fighters** (`LIMB` in the Blender script, 1.32): thicker arms and legs, big puffy gloves with a thumb and a knuckle ridge, bigger
  boots (soles and straps to match), rolled cuffs at the wrist, and a ball at each elbow and knee so a bent joint stays round. The skeleton
  and the animations are unchanged.
- **An outline that holds its weight** (`shaders/outline.gdshader`, the toon material's next pass): the ink line grows with distance from
  the camera, so it stays about the same thickness on screen when the camera pulls back (it used to thin out to nothing at long range).
- **Contact shadows**: a soft dark oval under each fighter, projected on whatever is below it (a `Decal`; the fighter's own meshes are on
  render layer 2 so it skips them; it only lands on upward-facing surfaces and fades with height).
- **Camera**: it now looks down at the stage a little (`CAMERA_PITCH`, 10 degrees, raised to match) so platform tops and the shadows show,
  and it comes in closer when the fighters are near each other (minimum distance 19, was 24).
- **Depth haze** (`lighting.gd`): fog that only thickens far behind the stage (depth 45 to 160), tinted to the stage's horizon colour, so the
  backdrop sits back and the fighters stand out. (A depth-of-field blur was tried and dropped: it bled the sky over thin things like the
  blade and the damage numbers.)
- **Hit effects** (`effects.gd`): a white shockwave ring, a ring in the attacker's colour and a burst of streaks where a hit lands, bigger for
  a harder hit; a ring at the feet on every jump (brighter for a midair jump).
- **Flashes** (an `instance uniform` in the toon and face shaders, so each fighter flashes on its own): white for a moment when hit, a soft
  yellow pulse while charging a smash.
- **Shield** (`shaders/shield.gdshader`): a bubble that is clear in the middle with a bright fresnel rim and a slight pulse, tinted by its
  health as before.


## Skinned limbs, real gloves, clothing that bends (twelfth pass)

- **Arms and legs are single smoothly skinned tubes** (`tube` and `chain_weights` in the Blender script): shoulder to wrist and hip to ankle,
  fuller at the upper arm, forearm and calf, weighted across the elbow and knee so a bend stays round like flesh instead of two capsules
  hinging (the elbow and knee balls are gone). A sock covers the lower leg, skinned the same way.
- **Cartoon gloves**: a puffy palm, four stubby fingers curled a little toward the body and a thumb, each a smooth tube.
- **Clothing skinned across the joints it covers**, so it bends with the body instead of slicing through the next piece: the shirt and the
  shorts blend from the hips to the spine across the waist (the same way, so their edges stay together), the shorts' legs from the hips
  into the thighs, the sleeves from the chest into the upper arms.
- A kicking leg is aimed a little toward the camera, so on the turned body it passes in front of the torso rather than through it.
- **No more clipping clothes**: the separate sleeve, shorts-leg and sock pieces are gone. Arms and legs carry UVs along their length (0 at the
  shoulder or hip, 1 at the wrist or ankle) and `shaders/limb.gdshader` paints the clothing on as bands with an ink hem line: a sleeve from
  the shoulder (with a shirt), the shorts' leg from the hip and a white sock below the knee. They bend exactly with the limb.
- **Fists**: the gloves are clenched, with the curled fingers as a row of knuckles and the thumb across the front.
- **Bigger swings**: attack clips play larger (x1.85 for punches and kicks, x1.75 for sword moves), the whole body pitches with each hit
  (into a hit in front or below, arching back under one overhead, curling forward away from one behind), the lunge is stronger, the torso
  bends twice as much toward an arm's hit, and a punch reaches out with the wind-up and comes home in the recovery like a kick.
- **Stage textures** (`godot/art/stage/*.svg`, drawn for this game in the same SVG pipeline as the faces, so they stay cel-shaded rather than
  photographic): painted grass clumps, wind-rippled sand, plank grain, concrete with seams, brushed steel. `#ff00ff` in a tile becomes the
  surface colour. The toon shader can lay a texture in world space (`world_tile`: world units per repeat, on top faces and fronts), so a
  surface tiles evenly whatever the block's size (`StageArt._surface`).


## Particles (thirteenth pass)

`godot/scripts/particles.gd`: a small particle system of its own (presentation only). Each particle is a camera-facing soft disc or a
spinning star, all drawn each frame into one mesh per effect; one-shot bursts free themselves when done. (Godot's `CPUParticles3D` was tried
first and did not render reliably here.)

- **Dust**: soft cream puffs that spread low and swell as they fade: landings, dash starts, every running footfall, skids, jumps.
- **Hit sparks**: discs flying out of a hit, white to gold to the attacker's colour, more and faster for a harder hit, drawn over the
  fighters, alongside the shockwave rings.
- **Knock-out burst**: sparks and confetti stars (the player's colour, a lighter shade, gold, white) thrown back toward the stage along the
  beam, and a cloud of smoke; placed a little in from the blast line so it is on screen.
- **Fire**: the brawler's rushing specials leave real flames, licks rising off the body from yellow to red (it replaces the old sphere).


## Walk, jumps and a living stage (fourteenth pass)

- **Walk**: a bigger, bouncier swagger: longer steps with the knee lifted high, a springy bob on every step, fists swinging wide and high,
  more shoulder twist and head nod.
- **Jump** (rising): the legs no longer stay together. The near leg (the right, nearest the game's camera on the turned body) drives its
  knee up in front while the far leg hangs long below with the toes pointed, the lead fist thrown up and the other arm swung back.
  (Front-and-back splits were tried first: with short legs and big shoes the back foot hid behind the round body.)
- **Fall**: the same open legs, arms raised out to the sides and paddling a little.
- **Midair jump**: a front flip: the fighter curls into a ball (`air_jump` clip) and the game spins it once about its middle
  (`FLIP_TIME`, 0.38 s), opening out into the rising pose.
- **Stages move** (cosmetic, own clock): clouds drift and wrap round (`Drift`), tree crowns and palm heads sway (`Sway`), the big tree's
  crown rocks slowly, the sailboat bobs and rolls (`Bob`), and flocks of birds flap across the day skies (`Flock`).
- `tests/anim_sheet.gd` now takes `--what=walk | jump | fall | airjump` too.


## Illustrated finish and hit weight (fifteenth pass)

**Style** (after an illustrated reference: thick ink, crisp two-tone shading, painted texture, a pale glow round the characters, a warm
printed-paper feel):
- **Post-process** (`shaders/post.gdshader`, on a quad in front of the match camera): ink lines wherever the depth or the surface direction
  jumps, so every silhouette and crease is inked, the stage and scenery included (fading with distance); a warm paper tone, soft paper
  blotches, a fine print grain and a vignette. It renders first among the see-through things, so effects, shields and damage numbers stay
  clean on top. `--nopost` turns it off.
- **Cel shading**: the light-to-shadow step is now crisp (`ramp_softness` 0.045), two clean tones.
- **Brush grain** (`shaders/brush.gdshaderinc`): fine wavering strokes and flecks painted along every cel surface's UVs (all Blender balls
  now carry UVs), characters and props alike; the world-tiled stage textures keep their own.
- **Halo** (`shaders/halo.gdshader`, a material overlay on each fighter part): a cream band outside the ink line, pushed back along the view
  ray so only the outer silhouette shows it.

**Ledge options as moves** (`_ledge_motion` in `fighter_view.gd`; the simulation still moves the fighter at once): catching the ledge swings
the body on the arms; getting up draws the body from where it hung, up first and then over the edge, through a new `ledge_climb` clip
(knee onto the edge, crouch, stand); a ledge roll travels the whole way in a forward roll; a ledge attack climbs and then sweeps (its hit
lands on the clip's strike); a ledge jump flips.

**Hit weight** (`main.gd`, `effects.gd`):
- Every hit now gets its feedback, flinches and each hit of a multi-hit move included (a hit is hitlag starting with damage taken; before,
  only launches counted).
- A directional impact: a white spike with an edge in the attacker's colour thrust the way the hit sends the fighter, with the shockwave,
  streaks and sparks.
- Heavy hits punch the camera in for a moment and flash the screen.
- The attacker shakes a little in hitlag too (the victim more).
- Hits on a shield ring the bubble and throw blue sparks; starting a fast fall flashes a small star with a whoosh.
- Kicks use the leg nearest the camera when the hit is in front (the model's left leg facing right): the claws neutral air's front kick
  is now the visible leg.

**Revised after play** (sixteenth pass):
- The cream halo is gone (only the black ink outline remains), the light-to-shadow step is soft again (`ramp_softness` 0.16), cast shadows
  are half strength (`shadow_amount`; a hat's shadow on the face was reading as a hard dark patch), the brush grain is off
  (`brush_strength` 0, the include stays for later), and the post-process's crease lines are lighter, so rounded bodies are not streaked.
- **Moves fill their end lag**: aerial and ground frame data already follow the reference game (a short hop fits one aerial; a full hop two
  only for a fast one, as there), but the body used to snap back to a relaxed pose right after the hit, so a move looked over early. Now it
  holds the follow-through and settles only near the end (`_attack_clip` progress after the last active frame is `0.63 + 0.37 k^1.8`), and
  the reaching limb, the body's pitch and the blade come home late the same way.

## Drawn effects, painted shadows and stage moods (seventeenth pass)

**Swing trails** (`fighter_view.gd`): a blade's crescent has a bright core along the tip's path with a crisp deeper rim; it starts three
frames before the first hit (the snap through), so it is already drawn when a hit freezes the swing, and always reaches the blade where it
is; as it ages it is eaten away from the hilt and thins to a point. Limb smears (the claws) taper along the path and with age, in three
layers (rim, colour, core); the ring that marked the live hitbox is gone.

**Drawn hit effects** (`effects.gd`, `main.gd`), by what landed the hit (`Effects.Kind`): a blow is a layered ink-outlined starburst
(outer flare, yellow body, white core) that pops out with an overshoot and breaks up; a blade adds a slash, a lens in the weapon's trail
colour laid along the swing; the maul's burst is blunter and bigger; fire (the brawler's rushing specials) is red-orange with embers.
They sit **where the hit met the fighter** (toward the hitter's live hitbox, no further than the body's edge), stretched the way the hit
sends it, and scale with the hit. The white hit flash lasts three frames, then stays faint, so the struck fighter stays readable.

**Painted shadows** (`shaders/toon_light.gdshaderinc`): a surface's shadow is its own colour turned toward the stage's shadow hue, a
little darker and richer (yellows shade orange, greens teal, blues violet); pale colours take a cool tint instead of going grey. The rim
light takes the stage's sky colour. **Stage moods** (`lighting.gd`, `MOODS`) set, per theme, the key and fill light (colour, strength,
angle), the ambient light, the shadow hue and the rim colour, through global shader parameters (`pf_*`, declared in `project.godot`).
Stage blocks and trims are on the same cel shader now (their patterns laid in world space).

**Processing**:
- **Effects are warmed up before the match**: behind the loading cover, every kind of hit, the impact, a block, the stars and rings, dust,
  a knock-out burst, embers and the flame are drawn once, and a card for every effect material, so no shader is compiled at the first
  hit. Effects share one material per kind (`fx_material.gd`).
- **Ink lines at real edges** (`shaders/post.gdshader`): every cel surface writes an object id as its roughness (each fighter its own,
  the stage and scenery a shared one); lines are drawn where the id changes and where the depth jumps (an arm in front of the body), and
  creases only on the stage and scenery, so there are no stray lines inside a fighter. Lines are thinner further away.
- **Only effects glow**: the cores of effects are drawn at four times white (`pf_fx_glow`) above a glow threshold (2.3) that lit surfaces
  stay under; the bloom is a screen blend.
- **Baked stage shading** (`stage_light.gd`, `shaders/stage_ao.gdshaderinc`): when a stage is built, how much sky each point near the
  stage plane sees is worked out from its blocks and platforms (in 2D, about 10 ms) into a small texture; every cel surface reads it as
  ambient occlusion, so the ground under a platform, a fighter standing there, and the foot of a block are softly shaded. **Fog layers**:
  bands of haze in the horizon's colour between the layers of scenery (`MIST` in `stage_art.gd`).
- **Effects on twos**: hit effects and particles are drawn 30 times a second, like hand-drawn effects, while the game runs at 60. Effects
  run on the match's clock (`FxMaterial.tick`), so they stop with a pause and slow with the knock-out slow motion.

Measured on the KO demo (`--demo=ko --perf`, the developer's laptop): about 350 frames a second against 200 before this pass, 614 draw
calls against 704, and fewer frames over 25 ms (14 against 37; most at load).

Checking effects: `--demo=fx_<sword|claws|maul>_<jab|ftilt|fsmash|nair|fair> --noui --shots=<dir>` lands one move on a fighter up
close with the camera steady on the attacker, a picture every frame.
