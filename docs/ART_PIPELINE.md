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
- **Faces** (since the twenty-sixth pass: parts, see the end; before that `godot/art/faces/<expression>.svg`): one drawing per expression (`deadpan`, `sleepy`, `grumpy`, `smug`, `hurt`, and `blink`,
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

- **How a fighter is turned** (`BODY_TURN`, `CHEST_TO_CAMERA`, `HEAD_TOWARD_FOE` in `fighter_view.gd`): the classic cheat for 3D fighters on a
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
  turns back toward the opponent (`HEAD_TOWARD_FOE`: the face about 40 degrees off straight-on) and every face drawing has its
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
- Close-up demos that photograph every frame: `--demo=sword_fair_close`, `sword_dair_close`, `claws_nair_close` (with `--noui --noecb
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

## Faces, big moments and painted stages (eighteenth pass)

**Faces by situation** (`_face_for` in `fighter_view.gd`, `godot/art/faces/`): a wince while frozen by a hit; **dazed** (spiral eyes,
a crooked open mouth) while flying from a big launch (tumbling) and while the shield is broken, with three cartoon stars circling the
head; **shocked** (wide eyes, a little round mouth, a sweat drop) when grabbed. The yell, the effort face while charging, focus and
blinking were already there.

**Knock-out moments**:
- A strong launch stretches the body along its flight (up to 45% longer and as much thinner, about its middle) and trails streaks behind
  it, on twos; both stop the moment the flight slows (`_launch_look`).
- **The finishing hit's impact frame**: for two frames the picture goes to two tones, the fighters pale with their ink lines against a
  dark frame, while the hit's arc, slash and burst stay in colour and glow on top (`impact` in `post.gdshader`, set by `main.gd` when the
  hit will knock out).

**Painted stages**:
- **Painted ridges** far behind each stage, drawn by a shader on flat cut-out cards (`shaders/painted_ridge.gdshader`, `RIDGES` in
  `stage_art.gd`): rolling hills, peaked mountains, flat-topped mesas or a bank of cloud, filled with a soft gradient, their tops caught
  by the light, a light brush grain, and their own haze toward the horizon. Coloured from the theme's sky, so the backdrop, the stage and
  the fighters' light share one scheme.
- **A colour grade per stage** (`grade` in each `MOODS` entry, applied by the finishing pass): the shadows and the lights of the whole
  picture tinted toward two colours (cool shadows and warm lights by day, violet and amber at sunset, blue at night, magenta in the city).

**Fixed**: the post pass read the object ids with a decoding this Godot version does not use, so a fighter against the stage got no id
line (only a depth line); the ids are now read as written (checked by drawing them).


## Moves for the base body (nineteenth pass)

The base body (the fighters' body since the eighteenth pass, see `ART_WORKFLOW.md`) has real arms and legs, so its moves are played
closer to life and posed again where the blob's read wrongly: the shoulders and hips wind back and turn through into each strike, the feet
stay on the ground, and the game's own lean, slide and body-follow are smaller, since the clips carry the body. Re-posed (`BODY_MOVES` in
`make_rigged_blob.py`), after how the reference fighters' moves read:

- **Forward air**: a crescent cut (or claw rake) down through the space in front, the weapon raised with the shoulders back and the knees
  drawn up, then the torso leaning over the swing with the front knee up and the back leg trailing; upright enough that the face stays on
  the hit. **Up air**: tucked, then arched back under an overhead crescent, looking up. **Back air**: the chest twisted right round to face
  behind as the swing rises behind. **Down air**: both arms raised, then swept down in front of the drawn-up knees.
- **Tilts**: the sword's forward tilt leans into a fast rising cut; the claws' forward tilt is a two-handed lunging slash (`attack_claws`).
- **The claws fighter** now follows its reference more closely: its **up air** and **down air** are claw slashes (overhead, and down below
  with both hands), no longer kicks, and its **down smash** is a low sweep in front and then behind (the `sword_dsmash` body).

- **Neutral airs**: the sword's (and the maul's, made from it) is an inward cut and then the blade carried right round the body as the
  whole fighter flips over backward once (`NAIR_SPIN` in `fighter_view.gd`: the body turns in the plane of the stage between the frames its
  hits circle, and the arm holds the blade out, so the blade follows the hits round); the claws' is the split kick.
- **Smashes**: the sword's up smash and down tilt are thrusts (`SWORD_THRUSTS`: the blade points at the hit from the start and the arm
  drives it in, instead of swinging round to it); its down smash is down low on a bent back knee, sweeping in front and then behind.
- **The maul** is swung two-handed (the other hand grips the shaft above the first), its heavy swings carried on the same body poses.
- **Fixed**: a clip that follows the move began with a cross-fade from the clip before, and seeking a paused clip never moves a
  cross-fade on, so some moves (the down smash's crouch, for one) never showed their own pose at all. They now start at once.

- **Side-on, as the reference game's fighters stand**: the hips and legs turn 80 degrees toward the way the fighter faces (was 60), the
  chest 14 back toward the camera and the head 6 more, so strides, lunges and kicks read across the screen while the drawn face keeps
  both eyes in view (`BODY_TURN`, `CHEST_TO_CAMERA`, `HEAD_TOWARD_FOE` in `fighter_view.gd`).
- **Glasses on the eyes**: they are fitted to the face drawing (its place on the head is in `base_rig.json` as `face`), scaled with it
  and centred on the part of the eye the drawings' heavy lids leave open (`_glasses_fit`, `GLASSES_DROP`); they had been fitted to the
  head's shape and sat on the brows. And a fighter shown without being played (the menus) now has its hats, glasses and neckwear
  where its idle pose has its head and chest, not where they would be standing straight.

Check them with `--demo=fx_<sword|claws|maul>_<move> --noui --noecb --shots=<folder>` (every tilt, smash and aerial; a picture on every
frame, the camera steady on the attacker).

## Pastel finish, props, hit identity and frame time (twenty-first pass)

**One look across the stages**: a pastel finish at the end of the post pass (`post.gdshader`): colours are pulled a little toward grey
(`pastel_desat`) and their darks lifted toward the stage mood's haze colour (`pastel_lift`, `"haze"` in `lighting.gd`'s `MOODS`), half
as much on the fighters, so the scene reads soft and airy while ink lines and effects stay crisp. **Block sides** are vector patterns
(quilted diamonds, wavy strata, rounded cobbles, a facade with lit windows) made as SVG in `stage_art.gd` and rasterised once at 256 px,
smooth where the old 64 px pixel patterns were blocky.

**Props** (`_props` in `stage_art.gd`): a few small low-poly things behind the fighters per theme, few-sided with flat colours and ink
outlines: hay bales, a barrel, a signpost and a stone on the meadow; mushrooms and a stump in the grove; rocks, a crate, a barrel and a
lantern post in the canyon; a beach umbrella and towel, a life ring and a bucket on the beach; stacked crates, a potted plant and a string
of coloured bulbs on the city roof.

**Merged scenery**: everything still in a backdrop that shares a material is merged into one mesh (`merge_static`), and flat colours share
one material each, so the extra props cost nothing: about 430 draw calls against 567 unmerged.

**Hits by fighter**: the sword's hits are a cool blue-white burst under its slash, the claws' three violet claw marks raked across a lilac
burst (`Kind.CLAW` in `effects.gd`), their fire moves still flame, kicks and punches of anyone a warm burst.

**Knock-outs**: every knock-out adds a shockwave (white, then the player's colour) and a flash to the beam and burst; the knock-out that
decides the match holds its close-up longer, slows to a quarter speed from the finishing hit, and blasts bigger with a longer slow motion.

**Frame time** (`--perf` now also prints GPU time, the render thread's CPU time, the ticks' and the frame scripts' time):
- The game's own work is about 6 ms a frame (GPU about 4.5 ms, render CPU 0.7, ticks 0.2, frame scripts 0.6). On the developer's laptop
  an empty Godot scene also runs at about 48 to 65 frames a second, so the frame rate measured there is the machine (other programs,
  power settings), not the game.
- **Vulkan on Windows** (`project.godot`, falling back to Direct3D 12 where Vulkan is missing): on Direct3D 12 every mesh made or freed
  during play (the weapon trails, the drawn hit effects) stalls the frame for several milliseconds (a 40-triangle `ImmediateMesh` rebuilt
  each frame cost about 9 ms against 0.08 on Vulkan); on Vulkan the same frames had half as many over 25 ms.
- **Hitbox display** (training, F3): one ring mesh drawn as a `MultiMesh`, moved and recoloured each tick, instead of a mesh rebuilt per
  tick (that alone was 6.6 ms a tick). Trails and paths with nothing to show are no longer cleared every frame.

**Swing arcs** (`ARCS` in `fighter_view.gd`, replacing the hand-traced smear and the sampled sweep): every weapon swing draws one smooth
crescent, pointed at both ends and thickest just behind its head, a bright core on the outside fading through the trail colour to
nothing inside. Each arc is a circle per move (the sword's fitted to its hitboxes; the claws' about the shoulder just past the hand),
the angles it covers and the active frames it is drawn on. The sword's head is wherever its blade tip is (never going back), so the arc
always ends at the blade; a claw is swept along its arc. It shows from just before the first active frame to the last, then fades from
the tail over 5 frames; hitlag freezes it. Thrusts, punches and kicks draw none. The separate slash and claw marks on a hit are gone:
the arc shows the cut, and the hit is a burst coloured by what landed it.

## Art-direction notes pass (twenty-second pass)

From the art-direction notes, what applied to this game (lighting, the cel shader and ink lines, parallax from the 3D camera, hit-stop
and directional knockback were already in, so they were tuned rather than rebuilt):

- **Match display** (`godot/ui/match_hud.gd`), after the reference game's: a tilted portrait of each fighter's own face (its real model
  and outfit, rendered once into a texture by a small private viewport, then never again, so it costs nothing in play), its damage in
  big italic numbers with a small tenth that heat from white through cream, orange and red to deep red (`heat`), stocks as little heads,
  the name on a dark slanted bar with a stripe and a P tag in the player's colour, and the weapon's emblem faint behind the number. Ink
  outlines and soft shadows throughout, the same as the clock. The clock reads 00:12 with "TIME LEFT", "TIME PLAYED" or, in free play, "FREE PLAY" over it (said once there, not on every
  card; with no clock it is a small chip at the top). The portrait backings are the player colours a little muted, so the numbers lead.
- **No damage over the heads in a match**: only a small "P1" tag in the player's colour (so players are told apart by more than colour).
  Training keeps the damage and name over the head (`FighterView.set_overhead`). `--match` on the command line shows a match's display
  without the menus.
- **Stage material**: a soft shadow on the block front under the cap (the edge of the fighting surface reads at a glance), tufts at
  uneven gaps and sizes with a stone now and then, and world-laid surfaces drift lighter and darker in broad patches so the pattern does
  not read as one stamped tile (`world_vary` in `shaders/toon.gdshader`, 0.08).
- **Atmospheric perspective** (`shaders/post.gdshader`): scenery behind the stage fades toward the mood's haze with distance behind the
  stage plane (`haze_start` 12, `haze_end` 140, `haze_max` 0.26; `stage_dist` is set each frame by `main.gd`), ink lines with it. The
  sky is left alone.
- **Foreground** pieces (low on the stage's front near each end) were tried and removed in the twenty-third pass: the match display
  hides that part of the screen, and when the camera pulls out they looked like bushes growing out of the stage's wall.
- **Hit restraint**: the shockwave ring of a strong hit is smaller and thinner, so the big effects stay readable without hiding the
  opponent.
- **Anti-aliasing**: MSAA stays at 4x (`msaa_3d=2` in `project.godot` means 4x): measured against 2x it costs about 0.1 ms of GPU
  time a frame for cleaner edges. With the machine idle the `--perf` tour ran at about 410 to 440 frames a second (GPU 1.4 ms, 456 draw
  calls).

**Outlines** (`shaders/outline.gdshader`): thinner (0.0022 of the distance, 0.02 to 0.11 units), pushed out only across the screen and
set a little back, so they rim the silhouette without cutting across the model where one part is in front of another. The pastel
finish no longer lifts near-black (`not_ink` in `post.gdshader`): it had turned the ink outlines grey in a match.

## Visual-polish roadmap, phases 0 and 1 (twenty-third pass)

**Phase 0 audit** (Godot 4.7.1, Forward+ on Vulkan with a Direct3D 12 fallback, 1280x720 base viewport stretched, MSAA 4x). One
scene builds everything in code: `main.tscn` / `main.gd` (camera, effects, match flow), `lighting.gd` (the environment, a warm key
light with shadows and a cool fill, per-stage moods), `stage_view.gd` / `stage_art.gd` (the stage and its scenery, procedural, not
Blender), `fighter_view.gd` (fighters: the rigged base body from `art/blender/make_rigged_blob.py`, imported as glTF, materials
converted to the cel material by `toonify.gd`), `shaders/toon.gdshader` + `toon_light.gdshaderinc` (the cel lighting),
`outline.gdshader` (inverted-hull ink), `post.gdshader` (screen-space ink lines, colour grade, pastel finish, distance haze, paper
grain), `ui/match_hud.gd`, `effects.gd` / `particles.gd`. The cel shader is a deliberate custom shader (painted shadow hues, a rim, a
stepped ramp), so the roadmap's "prefer StandardMaterial3D" does not apply to it. Found: two distance hazes stacked (the engine's depth
fog in one fixed blue for every stage, and the post pass's per-stage haze), a saturation boost of 1.12 that the pastel finish then
partly took back, SSAO at 1.2, soft and faint cast shadows, ink as heavy on props as on fighters. Baseline: all five stages from the
gameplay camera with the match display.

**Phase 1, lighting and rendering:**
- The engine's depth fog is off: the post pass's haze (in each stage's own colour) is the one atmospheric perspective.
- Less veil: saturation 1.04 (was 1.12), contrast 1.06 (was 1.03), pastel desaturation 0.1 (was 0.16) and lift 0.13 (was 0.2), distance
  haze at most 0.3.
- Firmer contact: cast shadows 0.62 dark (`shadow_amount` in `toon_light.gdshaderinc`, was 0.5), shadow blur 1.0 (was 2.0), the
  shadow range 70 units (the farthest the camera goes) in two splits, so the detail goes where the fighting is.
- SSAO a finishing touch: radius 1.0, intensity 0.9 (was 1.4 and 1.2).
- Anti-aliasing: MSAA 4x kept; SMAA on top was tried and made no visible difference (the screen-space ink is already smooth), so it
  is off. TAA was not used: it smears fast-moving fighters and effects, the worst trade for a fighting game.
- GPU time unchanged (about 1.5 ms a frame on the `--perf` tour).

**Phase 2, the material library** (`godot/scripts/materials.gd`): one cel material per surface family, each the shared cel shader with a
hand-drawn tile and its own response to light (`FAMILIES`): wood and grass matte with a firm step into shadow, bark soft and matte,
foliage softer still with a stronger rim (a puffy mass), stone crisp with a faint sheen. The fighters keep their own small sheen, so
they read a touch livelier than the scenery. Families: `Materials.world(tile, colour, size)` for stage surfaces laid in world space
(grass, wood, sand, roof, steel), `wood`, `bark`, `foliage` and `stone` for props and trees by their UVs, and `greens(base)`, four
coordinated greens (deep shade, body, light, sunlit tips) so a crown or bush reads as one plant. New tiles in `godot/art/stage/`:
`bark.svg` (vertical grooves, a few knots), `foliage.svg` (soft clumps with shaded undersides) and `stone.svg` (faint facets, hairline
cracks, speckles), made by `tools/art/make_tiles.py` (re-run it to change them); like every tile they are drawn on #ff00ff and only
translucent black and white, so one tile takes any colour. Used for: platform beams, fence posts, barrels, crates, signposts, the
lantern post, the life-ring post, the water tower, the windmill sails (wood); the great tree, the round trees, the palms and the stump
(bark, drawn on rather than built of boxes); crowns, bushes, palm fronds, the potted plant and the foreground bushes (foliage); rocks
and pebbles (stone). Measured on the same machine at the same moment: no change in GPU time or draw calls.

Blender's part, for imported props: keep the shapes, bevels and UVs there, give each surface a plain material in the right colour, and
let `toonify.gd` make it a cel material; a family can be assigned to it in Godot without touching the `.blend`.

**Phase 3, stage composition** (the grove is the stage the roadmap describes; the rest apply to every stage):
- **The great tree** is one hand-built mesh (`trunk_mesh`): it bows gently as it rises, tapers, and flares into its roots; the bark is
  drawn on; six broader roots. One mesh, so its ink line has no seams. (Hand-built meshes are left out of `merge_static`: merged in with
  the built-in shapes, the trunk vanished.)
- **Platforms**: a light edge where the top meets the front (it says where the platform ends) and a soft shade on the beam under the
  board; wood planks drift a little more in tone (`world_vary` 0.13 for the wood family).
- **The grass**: flowers and tufts in small clusters toward the stage's ends, none in the middle third (`_flower_clusters`).
- **Depth**: a far line of soft tree shapes on the hills (`_tree_line`), faded by the distance haze.
- **The island's underside** (`_island_underside`): a cone of earth hanging under the lowest block (fitted to it: `stage_view.gd` now
  passes the lowest block's left and right with the extent), a smaller tip below, rocks jutting from it and thin roots hanging from the
  rim, so the grove reads as a floating island when the camera pulls out.

**Phase 4, characters and outlines:** the fighters are now the clearest things on screen.
- **Outline hierarchy** (`outline_weight`, an instance parameter in `shaders/outline.gdshader`): the scenery's hull outline is 0.6 as
  thick and in a softer ink (`scenery_color`); each fighter's meshes set it to 1 (`_tag_object` in `fighter_view.gd`), the full black
  line. One shared outline material still serves everything.
- **Screen-space ink** on the stage and scenery at 0.72 strength (`scenery_lines` in `post.gdshader`); the fighters' lines in full.
- **A stronger rim on the fighters** (`rim_boost`, an instance parameter in `toon_light.gdshaderinc`, 1.4 for fighters,
  `FIGHTER_RIM_BOOST`), so they lift off the background; foliage's own rim brought down to 0.35 so the trees do not compete.
- Checked in close-up: even outlines, no gaps or dark seams where the fighters' parts meet. Screenshots in `docs/screenshots/`.

**Phase 5, combat effects:** the roadmap's effect set was already in (landing dust, hit bursts by what landed the hit, the swing
arcs, heavy-hit rings and impact spikes, launch streaks and puffs, shield flashes, and the sim's hitlag as hit-stop, scaled by damage
and set in the ruleset: `hitlag_per_damage`, `hitlag_base`, `hitlag_cap`), so this pass sharpened the difference between light and
heavy hits and kept effects off the action:
- **Camera shake only for hits that matter**: from more hitlag than a jab's or a tilt's (`HIT_SHAKE_FROM`, 9 frames, in `main.gd`),
  growing with it; it had been a little on every hit.
- **Sparks** 4 for a light hit up to 22 for a strong one (was 10 to 24); the burst smaller at the light end (`0.36 + 0.9 * strength`).
- **The hit flash** on the struck fighter scales with the hit: soft (0.3) for two frames on a light hit, bright (0.6) for three on a
  strong one, then faint; a jab had whitened the fighter as much as a smash.
- **Bursts sit a little toward the hitter**, so even a big one leaves the struck fighter's face in view.
- Effects clean themselves up (each frees itself when its life ends) and are a few small meshes each, drawn on twos.
Light against heavy (the sword's jab and forward smash): `docs/screenshots/phase5_light_vs_heavy.png`.

**Phase 6, presentation:**
- **Camera**: it already eases position and zoom smoothly (no snaps), frames both fighters with a margin, closes in on a close
  exchange and zooms on a finishing hit; now it also aims a little low (`HUD_LIFT`, about 4% of the view's height) so fighters on the
  ground sit clear of the match display along the bottom. Shake is kept for strong hits (phase 5).
- **HUD**: losing a stock is a moment of its own: the lost head swells and fades off the row and the portrait flashes red
  (`lost_at`, `LOST_MS` in `match_hud.gd`). The damage number already jumps and shakes on each hit, the round starts with "3, 2, 1, GO!"
  and ends with "GAME!" or "TIME!" and the results screen. The player tags over the fighters (and the training damage) now use the
  match display's typeface.
- **Audio**: skipped for now, as asked; the hit sounds already play on the impact frame (with the hit effect, as hitlag starts).
- **Performance** under combat load (`--perf` tour, the machine otherwise idle): 465 frames a second, GPU 1.3 ms, render CPU 0.6 ms,
  scripts 0.3 ms, 506 draw calls; 2 frames in 10 seconds over 25 ms (loading).

**Next, essential:** play-test the readability changes at real speed (the outline hierarchy and the light/heavy hit difference are
tuned from still frames); the other four stages have not had the grove's composition pass (their trees and props use the new
materials, but no hand-built focal shapes or undersides). **Optional:** sound design when it is time; a damage pop when a fighter
comes back at 0%; attack wind-up and recovery poses per move (animation work, best done with the movesets); per-stage tuning of the
haze and the colour grade after play-testing.

## Stance and faces after the concept art (twenty-fourth pass)

- **Stance**, as the reference game's fighters stand: the hips and legs turned 70 degrees toward the way the fighter faces (strides and
  kicks still read across the screen), the chest twisted back to 35 degrees off the camera (it mostly faces the viewer), and the head
  turned 10 degrees back toward the opponent, a three-quarter face with both eyes in view (`BODY_TURN`, `CHEST_TO_CAMERA`,
  `HEAD_TOWARD_FOE` in `fighter_view.gd`).
- **Faces** redrawn in the concept art's style (`tools/art/make_faces.py` draws every one; re-run it to change them): big round eyes, a
  cream ring round a large dark pupil with a small highlight, round pink cheeks, a small simple mouth; every expression (calm, sleepy,
  grumpy, smug, blink, happy, focus, attack, effort, hurt, shock, dazed) is the same face with its eyes and mouth changed, the lids in
  the fighter's own colour. The drawing is shown 1.15 times the shell's square (`face_zoom` in `shaders/face.gdshader`), since the
  concept's eyes are big for the head. The calm face is the one named `deadpan` in the catalog (kept, so saved fighters keep their face).
  Sheet: `docs/screenshots/faces_concept_style.png`.
- **The face fits the shell**: the face shell shows the drawing down to about row 320 of 512, so eyes (middles on row 214), cheeks
  (level with the bottoms of the eyes) and mouth (just under them) all sit above that; the first redraw had the mouth and cheeks
  lower, where they were cut off. The glasses follow the eyes' place on the head (`FACE_EYE_ROW`, `FACE_EYE_OUT`).

## Modelled clothes (twenty-fifth pass)

The base body's shirt and shorts are real meshes now, not painted on (`build_garments` in `art/blender/make_rigged_blob.py`, run with
`--body`; exported in `godot/models/base_rig.glb` as "Shirt" and "Shorts"). Each is a copy of the skin cut along its hems by planes (so
the hems are straight however the mesh is laid out): the shirt from just below the waist to the neck with short sleeves (55% down the
upper arm), the shorts from the waist to above the knee. They are pushed off the skin (the shirt 0.034, the shorts 0.022, so the
shirt's hem lies over the waistband), flare a little at the hems (the leg openings and the shirt's), and have a thickness of 0.012 so a
hem reads as a rolled edge. Every point takes the bone weights of the nearest point of the skin, so the clothes move exactly with the
body in every clip; the cuts' slivers are merged away first (they threw points far off). In the game (`fighter_view.gd`) the shirt takes
the look's shirt (white, the outfit colour, stripes or the flower print, laid by the garment's rest-pose texture map at the scales the
painted shirt used; hidden with no shirt) and the shorts the shorts colour; the skin shader stops painting them (`painted_clothes`),
and still paints the socks and shoes. The clothes keep the plain rim light (the fighters' stronger one picked out every fold).
One shirt and one pair of shorts for now, recoloured per look; other cuts later go in the same function. Known: in a full tuck a thin
glint of skin can show at the hip crease. Poses: `docs/screenshots/modelled_clothes.png`.

## Faces from parts (twenty-sixth pass)

The face is no longer one drawing per expression: it is put together on the head from **parts** a player chooses and places, like a
caricature kit (the character creator, `docs/MENUS.md`).

- **The parts** (`godot/art/face_parts/`, all drawn by `tools/art/make_face_parts.py` in the concept art's style; re-run it to change
  them): 8 eye shapes (`round` is the concept's), 6 brows, 6 noses, 6 cheeks, 9 resting mouths (`smile` is the concept's), and the
  expressions' shut eyes (blink, happy, effort, hurt, a dazed spiral) and mouths. Each is a 256 square; eyes, brows, cheeks, noses and
  shut eyes are white masks the game colours, mouths are in colour. Eyes, brows and cheeks are drawn for the right-hand side and mirrored.
- **Putting it together** (`godot/shaders/face_common.gdshaderinc`, used by `face.gdshader` on the head and `face_ui.gdshader` in the
  menus): the eye's white (cream), a pupil and highlight drawn in it (in the eye colour, looking the way the fighter faces), lids in the
  fighter's colour with an ink edge, then the ink outline; brows, cheeks and the nose are tinted masks; the mouth is laid on as drawn.
  Every part has a place, size, tilt and squash.
- **The look** (`godot/scripts/face_look.gd`, `loadout.gd`): each part's style, its colour (eyes, brows, cheeks) and adjustments in
  steps (eyes: height, spacing, size, tilt, stretch; brows: height, spacing, size, tilt; nose: height, size; mouth: height, size,
  width), appended to the look's bytes (about 30 now, at most 48); an older look reads with the standard face. Cheeks follow the eyes.
- **Expressions** are the player's own face changed (`FaceLook.EXPRESSIONS`): lids (top, slope, lower), brow tilt and lift, pupil
  size, shut or spiral eyes, and an expression mouth. Moods (deadpan, sleepy, grumpy, smug) are the same with the player's own mouth.
- **The face shell reaches the chin**: the base body's skin bulged past the shell toward the jaw and hid anything drawn below the
  mouth; each point of the shell is now pushed out until it clears the skin (`FACE_CLEARANCE` in `make_rigged_blob.py`), so parts can
  move anywhere on the face. The glasses follow the eyes' place and size (`FaceLook.eye_row`, `eye_out`).
- The old per-expression drawings (`godot/art/faces/`) and `tools/art/make_faces.py` are gone.

## Clothes that sit flat (twenty-seventh pass)

The modelled shirt and shorts creased and the shorts' legs differed. Causes and fixes (`build_garments`, `skin.gdshader`, `fighter_view.gd`):

- **Different legs**: the body mesh (from the generated model) is not symmetric, so a copy of it was not either. Each garment is now cut
  at the middle and its right half mirrored.
- **Creases from the body's bumps**: the cloth followed every bump of the skin (belly, chest, hips). It is smoothed (`GARMENT_SMOOTH`
  steps, the hems held) so it drapes over them, then pushed back out wherever smoothing pulled it toward the body.
- **Bunching when the body moves**: each point copied the nearest skin point's bone weights, so neighbours followed different bones and
  folded (the shirt's sides with the fists up). The cloth's weights are now smoothed over it (`GARMENT_WEIGHT_SMOOTH`).
- **Skin through the cloth**: under modelled clothes, a little inside the hems, the skin is not drawn at all.
- **Dark specks in folds**: the ink outline (an expanded hull just behind the surface) poked through where cloth folds inward. Garments
  use the same outline pushed further back (`CLOTH_OUTLINE_PUSH_BACK`): their silhouettes keep the line, their folds do not.

Second round (the desktop build still showed different legs and marks on the shirt):

- **Weights mirrored too**: the left half of each garment takes the right half's bone weights with left and right swapped, so both sides
  move alike; below the crotch each leg of the shorts follows its own thigh (fully at the hem), so the leg openings stay put.
- **No thickness**: the solidified inner layer crossed the outer one wherever the cloth folded (black scratches at the armpits). Garments
  are a single surface drawn on both sides (`toon_cloth.gdshader`, the inside a shade darker).
- **No line through cloth**: the skin's outline is left out under the clothes (the skin's vertex colour marks where;
  `hide_under_shirt` / `hide_under_shorts` in `outline.gdshader`), and the shirt has none round the armpits; the garments' own line is
  thinner and further back (`CLOTH_OUTLINE_THIN`, `CLOTH_OUTLINE_PUSH_BACK`).
- **Straight hems**, a snug waistband just under the shirt's hem (`SHORTS_TOP`), and only the shirt's arm and spine weights smoothed.

Third round (the shirt's hem still looked torn in the creator's stance):

- **The hem crumpled in the stance**: hem points had picked up a little of the thighs' and arms' influence (the arms hang beside the hips
  in the rest pose), so spreading the legs or raising the fists pulled them into teeth. Torso cloth now takes only the trunk's bones
  (hips, spine, neck), averaged per height over the torso's skin (`trunk_weights`); sleeves keep the arm's, and the shorts' legs blend
  into their own thigh below the crotch.
- **What counts as arm** is now closeness to the arm's axis (`ARM_RADIUS`, in the builder and `skin.gdshader`); far from the middle alone
  took the hips' sides for arms. The hems are smoothed along their own loops too (`HEM_SMOOTH`).
- **Finish**: a thin ink line along the shirt's bottom hem (`hem_ink` in `toon_common.gdshaderinc`), a softer light-to-shadow step on
  cloth (`CLOTH_RAMP_SOFTNESS`), full mesh detail at any size (`lod_bias`), and the skin hidden in one stretch from the shorts' hems to the
  neck when both are worn.

Posed: `docs/screenshots/clothes_smoothed.png`.

## A body after the concept art, modelled by script (twenty-eighth pass)

The base body is no longer the generated (Tripo) model: `art/blender/make_base_body.py` builds it from numbers read off the concept sheet,
then the usual steps measure and rig it (`docs/ART_WORKFLOW.md`, Characters). Lineup: `docs/screenshots/concept_body_lineup.png`, in gear:
`concept_body_gear.png`.

- **Proportions as on the sheet**, at the old height (2.2) so hats, the camera and the hurtbox picture still fit: a big round head (about a
  third of the height) sitting straight on a small pear-shaped trunk with no neck, the shoulders slim front to back under it so the head
  overhangs them all round (its outline reads as a whole circle, as drawn); a round belly pushed forward and a little seat behind; soft
  tube arms hanging out at about 20 degrees; short thick legs; wide puffy oval feet. The arms end at the top of the thigh (the sheet's
  reach to mid-thigh read as too long in play).
- **One seamless skin**: each part is a simple solid (a ball, a lathed trunk, limbs swept as chains of balls), joined by voxel remeshing,
  softened where they meet and reduced to 6,500 triangles. Parts stand at least 2% of the height apart below the armpits and crotch, or
  `fit_body.py` cannot tell them from the trunk.
- **Rigging**: `make_rigged_blob.py --neck-drop 0` (the head is not lowered: there is no neck); the fists are smaller (`FIST_SCALE` 0.7),
  the concept's small mitten hands.
- **Bare by default** (`Loadout.GEAR`, a new slot at the end of the look: none, shorts, shorts and gloves, full kit): every player starts
  as the plain coloured body with only a hat and glasses to tell them apart; shirts, neckwear and gear are the creator's choices (Gear is
  a new category there). Bare hands take the body's colour and skin, without the glove's cuff; without shoes the feet are the body. A look
  saved before the slot existed keeps the full kit it wore.
- **Shading closer to the sheet**: skin is matte with a softer step into shade (`RESPONSES.skin`); the outline is thinner and warmer, the
  body's own colour darkened rather than near-black (`FIGHTER_OUTLINE_WEIGHT` 0.82, `OUTLINE_TINT` 0.75). Still the strongest line on
  screen: the look bench passes (worst 0.90, needs 0.85).
- **Neckwear** sits just under the chin (`_collar` from `base_rig.json`'s neck), not on the chest.
- Known: a tiny ink speck can show at the chin's middle where the head meets the trunk; the fists are still the glove fist's shape in the
  body's colour (fingers modelled for bare hands would be the next step); the clips were re-keyed on the new joints but not re-posed for
  the new proportions.

## Magic-hour skies, raking light and drifting motes (twenty-ninth pass)

Aim: colours that carry and an ethereal air (painted skies of violets, pinks and oranges) while every stage keeps its own mood. Meadow
before / after: `docs/screenshots/ethereal_meadow_before.png`, `_after.png`; all five stages: `ethereal_all_stages.png`.

- **Painted sky** (`shaders/sky.gdshader`, built by `StageArt.sky`): four bands (top, a middle tint `sky_mid`, horizon, ground) placed low
  enough that the match camera, which sees only the lowest quarter of the sky, shows all of them; a warm glow low over the horizon behind
  the stage (`sky_glow`) and faint painted wisps (`sky_wisps`). Each theme sets its own: Meadow violet to pink to peach with a gold glow;
  the grove pink-lavender; Triple Tier deeper dusk; Flat Island a soft warm glow on its blue; Skyline a magenta horizon. Static (no time),
  so it costs nothing per frame.
- **Meadow's light** (`Lighting.MOODS.meadow`, `STYLES.meadow`): a lower, golden key (pitch -30, 1.38) that rakes across fighters and
  props, violet fill and shadows, a lavender-peach haze; crisper light-to-shadow steps, warmer tops and cooler undersides, and much less of
  the pastel wash (desat 0.02, lift 0.03, haze 0.18). The menus keep their own neutral daylight (`MENU_MOOD`), unchanged.
- **Props**: hay bales are woven straw (the straw hat's drawing) with a lit and a shaded side; the fence has pale posts with rounded caps,
  each in its own contact shade, and darker rails; the barrel has a darker lid. The lawn has broad sunlit and shaded patches
  (`grass_vary`).
- **Air**: golden pollen motes drifting over the meadow (pink ones under the grove's tree): GPU particles that fade in and out and bloom a
  little, fewer with the Effects setting and none on Fast (`StageArt._motes`); faint light shafts slanting across the far hills, behind
  the stage so they never wash the fighters (`_light_shafts`).
- Checks: match flow, art pipeline, fighter view, creator and editor tests passed; look bench PASSED (worst 0.90). Four-player Meadow
  frame time was measured with the machine busy (about 45 fps both before and after); the pass adds about 0.14 ms of GPU time.
- Not done: Godot 4 has no per-light contact shadows, and screen-space ambient occlusion cannot be used (the object id lives in the
  roughness channel), so grounding stays painted (contact patches, the fighters' drop shadow).


## Ship polish: stage undersides, clear vivid colour, mitten hands, the chin speck (thirtieth pass)

Aim (branch `ship-polish`): a sellable look, with the readability of a modern platform fighter, the painted style of the concept art and
the clarity of a cel-shaded adventure. Before / after: `docs/screenshots/underside_*`, `vivid_meadow_*`.

- **Under the stage** (`StageArt.underside`): what each place is made of hangs under its lowest block, set back behind the fighters and
  fading into a haze bank; block fronts calmer. See `ART_DIRECTION.md`.
- **Clear, not milky** (`VISUAL_STYLE.md` section 4): far scenery hazes toward each stage's own saturated colour instead of white,
  lighter paper tint, no default desaturation, thinner mist, richer skies placed where the camera sees them, cel-lit cumulus clouds.
- **Bare hands are mittens** (`make_rigged_blob.py` `mitten`, parts `Mitt.L/R`): a soft rounded paddle with a stubby thumb, shown in the
  body's colour when the loadout has no gloves (`FighterView`); the glove fists (`Hand`) stay for gloves. A rig without mittens falls
  back to the fists in the body's colour.
- **The chin speck**: the symmetric decimation in `make_base_body.py` left a two-sided fin on the mirror line under the chin (and one at
  the back of the hips); `mend_folds` joins copied points, triangulates and deletes any point with only two edges, so the skin is
  closed (every edge two faces). Body rebuilt (6,496 triangles), joints refitted (changes under 0.002), rig rebuilt.
- Checks: fighter view, loadout, art pipeline, match flow and creator flow tests passed; look bench PASSED (worst 0.90).
- **Crisper fighters, readable tags**: skin takes the stage's crisp cel step (`RESPONSES.skin` 0.16); match tags are bold, in the
  player's colour over a pointer, one size on screen at any zoom (`FighterView.TAG_PIXEL`). Look bench worst 0.88 (passes at 0.85).
- **Authored props by script** (`art/blender/make_props.py`, original designs): a lighthouse on its rocks (red bands, gallery and
  railing, glowing lantern, a halo), a sailing dinghy, and two sandstone buttes in strata on rubble slopes, one flat colour per part so
  `Toonify` makes each part cel. Placed by `StageArt._place_prop`, whose parts `merge_static` batches. Flat Island 476 draw calls (463
  before), Triple Tier 413 (427 before); GPU time unchanged within noise (about 1.4 ms on the developer's laptop).
- **More scripted props and colour fixes**: a ribbed saguaro with curving arms and a flower (`cactus`) and a palm crown of arching,
  leafleted fronds with coconuts (`palm_crown`, swaying on the game-built trunk). `make_props.py` now writes its palette as the colours
  should look in the game (it converts them to the linear values glTF keeps; before, every prop came out paler). Treetop Isle's hills
  are deeper greens and blues; Flat Island's stone front warmer stones in slate mortar. Flat Island 377 draw calls (476 before: the
  crowns replace seven shapes each), Triple Tier 417.

## A cuter body (thirty-first pass)

The owner asked for the base body to read cuter: rounder, shorter, stubbier. Before and after: `docs/screenshots/cute_body_front_before.png`,
`cute_body_front.png`; in play `cute_body_match.png`.

- **Proportions** (`art/blender/make_base_body.py`), still 2.2 tall so hats, camera and hurtboxes keep their fit: the head from about 35%
  of the height to about 43% (radius 0.4 to 0.475), a shorter, plumper trunk with a rounder belly, the hips from 0.82 down to 0.57,
  thicker short legs, chunkier arms held a little further out, bigger puffy feet. Joints refitted, rig and every clip rebuilt
  (`--neck-drop 0`, as always). The face drawing, hats, neckwear and clothes follow on their own from `base_rig.json`.
- **Glasses on the face**: each lens, rim and bridge is seated on the head's curve over the drawn eye and turned with it
  (`FighterView._on_face`), instead of on a flat plane in front of it, so on a fighter turned three-quarters the far lens stays on the
  face.
- **Re-posed for the big head**: no clip tips the head back past 8 degrees (`HEAD_BACK_LIMIT` in `make_rigged_blob.py`; lying down and
  ledge clips excepted); an overhead kick arches the body at most 26 degrees and the head counter-tilts against an arch, so the face stays
  toward the camera. Kicking legs stretch further than fists on the strike (`MotionDeform.LEG_STRETCH_FACTOR`) so the short legs reach.
- **Short legs scurry**: the run and dash take 1.4 times the steps (`STEP_CADENCE`), the walk is quicker (`WALK_TEMPO` 0.055). Planted
  foot slide (`tests/motion_probe.gd`): run about 0.09 and dash about 0.13 of ground speed (they had risen to about 0.36 and 0.2 with the
  shorter legs); walk 0.42 to 0.45.
- **Pale bodies get a darker line** (`OUTLINE_TINT_DARKEN_PALE`). The look bench now ignores backdrop-coloured pixels the fighter
  encloses (a pale belly lit the colour of a pale sky is not a lost silhouette; the misses sheet showed them as red lines inside the
  body); worst silhouette 0.87 (passes at 0.85).
- The body script also closes any slit left after removing decimation fins (`mend_folds`; none were found this time).
- Checks: art pipeline, match flow, fighter view, loadout, replay, motion, anim library, creator flow, content, controls and online flow
  tests passed.

## Faces after the concept, with depth (thirty-second pass)

Compared straight on with the concept's front view (`godot/tests/face_sheet.gd`, framed like the concept's crop): `docs/screenshots/
face_concept.png`, `face_before.png`, `face_after.png`.

- **Centred on the head**: the rig laid the drawing out about the middle of the head's points counted from the neck up, well above the
  round head's middle, so every face sat high. It is now centred on the ball's middle (its crown less its radius, `make_rigged_blob.py`).
- **The concept's layout** (`FaceLook`, measured in head radii off the sheet): eyes 0.41 out and 0.185 up, 0.25 across (they were 20%
  bigger and closer together); the smile 0.36 wide (it was two thirds that; only the resting smile is wider, the expressions' mouths keep
  their size); the blush 0.51 out and 0.2 down (it was further out and higher); a finer eye line (11 of 256, was 16). Lenses are sized to
  the eyes and kept round.
- **Depth in the drawing** (`face_common.gdshaderinc`, so the face on the head and the creator's pictures agree): the eye's white is
  shaded as a dome (cooler toward its rim, shadowed under the upper lid); the iris has a darker rim, a deep core, a warm glow welling up
  in its lower half, a main catchlight and a small second one opposite; lids are lit from above and cast a soft shadow on the eye; an
  open mouth is in shadow under the upper lip; the blush softens toward its edge. Expressions: `face_attack.png`, `face_shock.png`.
- Checks: creator flow, fighter view, loadout, art pipeline and match flow tests passed; look bench 0.87.

## Clear Meadow, warm Flat Island (thirty-third pass)

An audit of every stage in a four-player match (`docs/screenshots/clear_meadow.png`, `warm_flat_island.png`):

- **Meadow was fogged**: its distance haze (`lighting.gd` STYLES meadow `haze_max` 0.18) washed the trees behind the fence, the hills and the
  windmill toward pink. It is now 0.08 (no desaturation, lift 0.02, a little more s-curve), and its mist banks are as light as Treetop
  Isle's. The trees and hills read green, the windmill crisp, the hay golden; the painted violet-and-peach sky keeps the mood.
- **Flat Island's front was cold**: grey cobbles in slate mortar under warm sand. The stones are now warm sandstone (soil 0.82, 0.66, 0.5;
  dark 0.6, 0.48, 0.46) and the sand top golden (0.96, 0.8, 0.5) instead of bleached near-white.
- Look bench 0.87 (passes at 0.85).
- **The Seer (class 5, Staff)**: a staff on the weapon pivot, posed like the sword from the live hitboxes (a pale wooden shaft, a gold
  collar, a glowing head; `FighterView.STAFF_CLASS`), its own look (plum, a crown, a sash), an eye-of-light HUD emblem, and its moves
  mapped to the sword's cuts for the staff spins and sweeps (`AnimLibrary.CLASS_MOVES[5]`). The Gauntlets' specials are mapped too
  (charged punch, uppercut and overhand, the leaping grab's catch, the slide and dive kicks). `docs/screenshots/seer_staff_jab.png`.

## A day in the meadow, sky islands and a real lawn (thirty-fourth pass)

Toward the owner's reference (a bright stylised 3D game: deep blue sky, white clouds, islands floating in the sky, thick painted grass,
vivid greens) and the plan in their `game_wide_visual_fidelity_art_direction_plan.md` (hand-painted realism inside the cel style, every
stage its own identity). `docs/screenshots/meadow_day_sky_islands.png`, `meadow_lawn.png`, `grove_lawn.png`.

- **Meadow is midday**, not magic hour: a deep blue sky to a pale horizon with white wisps, white cumulus, a neutral sun from higher up,
  blue shade (`lighting.gd` MOODS meadow), saturated hills and blue mountains, no shafts of light. Treetop Isle keeps its pink dusk, the
  canyon its sunset, so the stages stay distinct.
- **Floating sky islands** (`StageArt._sky_islands`): earth keels with grassy caps and a lip of turf, some with a round tree or a bush,
  at depths from 70 to 140 units, smaller and higher with distance, framing the stage without crossing the middle of the sky.
- **The lawn as the reference paints it** (the owner's Wind Waker HD reference; a first try with dense blade cards read as scattered
  wisps, not grass, and was dropped): one clean, vivid lime sheet (meadow grass 0.46, 0.82, 0.16) with its variation in long soft
  sweeps and broad hollows on a large tile (`tools/art/make_tiles.py` `grass`, 9 units a repeat), no blades on the meadow or grove, so
  the platforms' and props' cast shadows read as crisp shapes on it. (`StageArt._lawn` and the root-to-tip tuft shading stay available
  for a stage that wants standing grass.)
- **Dressed stone under moss** (meadow): the front is big rounded stone blocks in rough courses (`masonry` pattern, 7 units a repeat,
  lit tops, shaded feet, a few cracks) in warm sandstone, with moss hanging over the top of the wall in clumpy tongues
  (`moss_color`/`moss_amount` in `toon_common.gdshaderinc`: lit top, shaded hanging edge, a darker rim and a soft cast shadow on the
  stone), in place of the cobbles and the green bubbles. `meadow_lawn_close.png`.
- **Round painted crowns** (`StageArt._leaf_cards`): a crown is its solid lobes, full size, painted with round leaf clumps (the
  `foliage` tile), with a crisp step from lit top to cool underside (ramp 0.08, shadow 0.78) and a warm rim. Pointed leaf cards were
  tried at a third of the count and dropped: like blade grass, they read as noise at play distance (`CROWN_CARD_SHARE` 0; it brings
  them back if a stage wants them). The meadow's round trees are yellow-green to match the lawn; Treetop Isle keeps its teal.
  `meadow_trees.png`, `grove_trees.png`.
- Costs (Meadow, stage demo): GPU about 1.8 to 1.9 ms, Treetop Isle 2.1 ms; look bench 0.87.

## Beach and canyon after the reference (thirty-fifth pass)

- **Flat Island**: the repeating pebble front (the owner's plan named it) is dressed sandstone blocks (`masonry`); the sea is vivid
  turquoise to deep blue (`water.gdshader` shallow, deep, streak, haze) under a stronger blue sky, with far less distance haze (STYLES
  ocean `haze_max` 0.07, a lighter mist). `docs/screenshots/beach_vivid.png`.
- **Triple Tier**: its sunset kept, the fog lifted (STYLES sunset `haze_max` 0.1, mists about half), so the mesas keep their strata
  colours against the sky. `canyon_clear.png`.
- Flat Island's horizon: the warm glow there washed a pale band over the sea; now a faint white one (0.15) over a bluer horizon.
- The meadow's round trees stand on tapering, slightly bowed trunks with flared roots (`trunk_mesh`), not straight cylinders.
- **Skyline** left as it is: a dark, moody stage where the fighters already stand out (the plan: keep a dark stage's mood).
- Look bench 0.87.


## Painted effects, stages that stand on the ground, signs of life (thirty-sixth pass)

- **Painted effects** (`shaders/fx.gdshaderinc`, opt-in per material through `FxMaterial.get_material(..., paint, puff)`):
  - `paint`: an effect's colour varies in brush streaks running out from its middle, and as it fades it breaks up into ragged chunks
    with a darker pooled edge, rather than going see-through like a vector shape. On the hit bursts, rings, speed lines, spikes, sparks,
    embers and the flame.
  - `puff`: each particle quad is a cel-shaded ball of smoke (lumpy rim, lit from the upper left, a cool shade crescent), eaten away by
    the same breakup. Dust at the feet, the launch trail (now one mesh in `fighter_view.gd`), a knock-out's smoke and chimney smoke.
- **More explosive hits** (`Effects.hit`): sparks are shards stretched along their flight, two in three thrown the way the hit sends;
  a strong hit adds speed lines and a ring of smoke balls behind the fighters (`Particles.impact_smoke`), so it frames the hit rather
  than hiding it. `fx_hit_before.png` / `fx_hit_after.png`, `fx_dust_puffs.png`.
- **The knock-out beam** (`Effects.Beam`): the pale see-through cylinders became a painted column in the player's colour (ink edge,
  deep body, lighter band, thin white core) whose edges crackle on twos; it slams open, holds, thins and breaks up.
  `fx_ko_before.png` / `fx_ko_after.png`.
- **Stages stand on the ground** (the owner: the hills' bottoms showed as circles, the city had no street, the windmill floated):
  - Hills are continuous land (`_land_band`): the same round hilltops melted into a valley floor, running 180 units down out of view.
    `hill_top` and the nearer `_fore_band` (meadow, grove) give heights to stand things on; trees, cottages and the windmill sit just
    behind a band's edge. `ground_meadow_before.png` / `ground_meadow_after.png`, `ground_grove_after.png`.
  - Skyline: the rooftop building runs down to a street (`_street`, `CITY_STREET_DROP` below the stage): pavements and kerbs, a road
    with a dashed line, street lamps with warm halos, cars driving both ways, and a shopfront with a striped awning at the building's
    foot. `ground_city_before.png` / `ground_city_after.png`.
  - Triple Tier: the mesa column stands in a desert floor (`DESERT_FLOOR`, level with the far buttes) with fallen rock at its foot, and
    less haze over it. `ground_canyon_after.png`.
- **The windmill** has a stone footing, a door, shuttered windows, a gallery, a shingled cap with a finial, and lattice sails.
- **Signs of life**: cottages with smoking chimneys on the meadow hills, telegraph poles and sagging wires across the desert, strings
  of lanterns under the great tree, and a fisherman's hut on stilts with a jetty and floats off Flat Island (`beach_stilt_hut.png`).
- Layouts kept: the hills draw the same random numbers as before, and the new pieces draw none, so nothing else moved. Look bench passes.
