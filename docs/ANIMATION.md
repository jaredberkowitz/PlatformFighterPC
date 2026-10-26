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
2. ~~The walk still slides about 0.45 of its ground speed~~ (round 2, pass 20: each family walks its own way with planted feet, sliding
   0.06-0.16 of its speed).

Animation pass for the cute body (owner's direction, 2026-10-13): weighty and exaggerated for readability like the reference, but with
limbs that make physical sense (a support leg planted or tucked, arms counterbalancing), moves thrown at the opponent rather than at the
camera, and our own solutions where the reference flips the whole body. Done: the base body keeps 65% of the clips' exaggeration
(`BODY_AMP`); in a move the hips turn 14 degrees further toward the foe, the chest stops opening to the camera and the face may turn to
three-quarters (`attack_turn`); otherwise the head stays within 32 degrees of the camera (`_face_camera`); the neutral air kick is held
with the other leg tucked; the sword's neutral air is a windmill slash round an upright body (no somersault); the claws' flame rush is
a corkscrew about the upright body (no head-over-heels spin); wall jumps back-flip off the wall; Gauntlets' own clips (`gaunt_knee`,
`kick_axe`, `gaunt_stomp`, `gaunt_shoulder`, `gaunt_elbow`). Review every move with `--demo=gallery --gallery-class=N` and
`tests/montage.gd`.

## 7. Animation pass, 2026-10-14 (the owner: fluid, a bit exaggerated and cartoony, the reference's motion where it applies)

- **Swing timing** (`fighter_view.gd` `_blade_target`, `strike_progress`, `_reach_weight`): the wind-up eases into its anticipation and
  the strike accelerates into the hit (the blade whips rather than gliding at one speed); after the last hit the blade carries on
  `FOLLOW_THROUGH` (38) degrees the way it was travelling over the first `FOLLOW_SHARE` (28%) of the end lag, then eases home. The
  body's clip and the reaching limbs ease out over the end lag too, instead of holding the strike frozen and snapping back late. Frame
  data and hitboxes are untouched (the blade's tip is on the hit throughout the active frames).
- **More exaggeration**: the base body keeps 85% of the clips' amplification (`BODY_AMP`, was 65%): deeper wind-ups, bigger lunges,
  stronger arches, still with planted feet.
- **The reference's motions where the mapping was generic**: kicks where the reference kicks (the all-rounder's side kick, sweep,
  slide, flying, drop and flip kicks; the speedster's side kick, low kicks and flip kick; the claws fighter's and the sprinter's flip
  kick up air; the Wanderer's kicking neutral and back airs; the Needles fighter's kicking smashes), headbutts where the reference
  headbutts (new `body_headbutt`, up, and `body_headbutt_fwd`: the all-rounder's up smash, the Bounder's smashes and forward air, the
  Hauler's up air; `body_` clips aim no limb, the body is the weapon), the Hauler's two-fisted overhead hammer forward air, the
  Settler's slingshot forward air and shovel down smash.
- Images: `docs/screenshots/anim_fsmash_before_after.png` (the sword's forward smash, before above: it now carries through and eases
  home), `anim_reference_moves.png` (the headbutt up smash, the flip kick up air, the lunging headbutt).
- **Checked against the reference's hitbox animations** (ultimateframedata.com, decoded frame by frame): the sword's forward smash now
  ends in the reference's very deep lunge (back leg nearly flat, body low over the bent front knee); the claws fighter's forward tilt
  is its low, level lunge with one claw thrust ahead (was an upright step); its neutral air is the reference's held double kick,
  leaned far back with both legs out (new `kick_double`; was a one-leg sitting kick). The sword's airs (tucked knees, the back air
  turning the body round) and tilts already matched. `docs/screenshots/anim_reference_pass2.png`.
- **Pass 3** (the other references): the Wanderer's down and up airs are its reference's thrusts, the blade driven straight down below
  the tucked knees (`sword_plunge`) and straight up overhead with the body stretched beneath (`sword_rise`), the blade pointed at the
  hit throughout (`fighter_view.gd` `_thrust`, `BUCKLER_THRUSTS`); the Champion's down air plunges too. A grounded blade's tip turns up
  about the hand to stay above the floor (a deep lunge dropped it in). The all-rounder's, speedster's, Hauler's, Bounder's and Needles
  fighter's moves were checked against their references and already matched after pass 1. `docs/screenshots/anim_reference_pass3.png`.
- **Pass 4, the Settler's own clips** (its reference's tools; `tool_*` in `make_rigged_blob.py` BODY_MOVES, mapped in
  `AnimLibrary.CLASS_MOVES[10]`): `tool_chop` (the umbrella and the axe, both hands raised and brought down in front), `tool_heave` (the
  bowling ball), `tool_raise` (the firework, the rocket pulled out), `tool_trip` (the flowerpot dash attack), `tool_reach` and
  `tool_toss` (the pocket and its release), `tool_dig` (planting), `tool_pour` (the watering can), `tool_shrug` (the failed planting),
  `tool_ride` (riding the rocket) and `tool_float` (the balloons); the turnip airs' slots take the shared up and down airs. Clips in
  `AnimLibrary.OWN_ARMS` pose both arms themselves (the tool hand is not reached to a hit). `tool_` and `body_` clips take the attacks'
  exaggeration, and the shoulder and elbow are clamped after it (172 and 145 degrees): a raised arm amplified past vertical swung round
  behind the body (the Hauler's two-fisted hammer read only after this). On this body, arms raised as high as they go put the hands
  beside the head (its arms are short against the head). `docs/screenshots/anim_settler_tools.png`.
- **Pass 5, the whole body answers an aerial** (the owner: "the entire body compensates for the leg sticking out", as the reference's up
  airs turn the body; `fighter_view.gd` `_air_counter`): in the air the whole fighter pitches about its middle (`AIR_PIVOT`) with the
  move's reach: an up air tips it back under the kick or slash (up to `AIR_BACK` 36 degrees, held as the arc carries on past the top),
  a back air pitches it forward, head down, as the leg drives out behind (`AIR_FORWARD` 50), a kick out in front leans the body back
  against the leg (`AIR_KICK_BACK` 18, a jackknife), a swing in front curls the body over it, and a stomp below stays nearly upright.
  On the lean spring, so it overshoots a little and settles; on the ground the old lean over the feet is unchanged.
  `docs/screenshots/anim_air_counter.png` (the all-rounder's up, back and neutral airs, the sword's up air).
- Next: the Settler's props (pocket, shovel planting, watering can, axe) and its props; a pose-by-pose pass per moveset.

## 8. Animation overhaul (2026-10-25: `docs/ANIMATION_OVERHAUL_AGENT.md`, audit in `docs/animation/AUDIT.md`)

### Pass 1: a low battle stance per family

- **Defect** (audit item 1): every idle stood upright, the knees nearly straight. Measured in the built rig, the sword idle's hips were
  0.85 against 0.94 standing, with a 5-degree bend in the near (back) knee. The caster and tool idles had straight legs. The bob
  moved the hips by 0.001: you could not see it. The brawlers (Claws, Gauntlets, Bracers) held the sword family's guard, fists out wide,
  where their reference crouches.
- **Cause:**
  - `make_rigged_blob.py` `idle()` and `IDLE_STYLES` authored shallow knees and kept the back leg straight on purpose.
  - `plant()` resets the hips from the feet on every key, so the hip drop and bob written into the clip were thrown away.
  - The game turns the body most of the way to the camera (`BODY_TURN` 60, `CHEST_TO_CAMERA`), so a forward lean of the chest pointed
    mostly at the camera and did not read.
- **Fix:**
  - Both knees bent, the back knee turned out so it stays behind the front leg instead of crossing it. Hips: sword 0.69-0.72,
    brawler 0.60-0.64, quick 0.53-0.56, heavy 0.60-0.63, caster 0.79-0.81, tool 0.82-0.84.
  - The bob and a once-a-loop weight shift onto the front foot are now knee bends, so `plant` turns them into hip motion (about 0.03).
  - The idle loop runs 96 frames (`IDLE_FRAMES`) instead of 48: two bobs, one breath and one weight shift a loop. The breath lifts the
    arms too.
  - The chest bends sideways toward the foe as well (`STANCE_SIDE`, `side` in `IDLE_STYLES`), so the lean shows on screen.
  - A new brawler family idle (`idle_brawler`: hunched over bent knees, both fists up), mapped for classes 1, 4 and 7 in
    `AnimLibrary.CLASS_FAMILY`. The brawlers keep the shared run (`FAMILY_SHARES`).
- **Files:** `art/blender/make_rigged_blob.py`, `godot/models/base_rig.glb`, `godot/scripts/anim_library.gd`, `godot/scripts/fighter_view.gd`
  (`LOOPING`), `godot/tests/anim_library_test.gd` (a family may share a clip).
- **Image:** `docs/screenshots/anim/pass1_stance.jpg`. Contact-sheet frames per family, before above and after below, both facings,
  then the sword and claws fighters close up at the match camera (the pose demo's idle shot).
- **Checked:**
  - both facings (the mirrored clips lean the right way);
  - `anim_library_test`, `fighter_view_test`, `motion_test`, `match_flow_test`, `replay_test` and `look_bench` pass;
  - the rig rebuilt from the unchanged script renders pixel-identical to the committed one, so the only change is this pass's.
- **Left:**
  - the walk, run and dash per family (the next locomotion pass);
  - the caster and tool families stay the most upright, by design;
  - the idle fidget (`idle_look`) starts from the new stance but was not re-posed;
  - not tried with a controller or in a live match.

### Pass 2: the jump and fall arc

- **Defect** (audit item 2): one held pose for the whole rise (the near knee up) and another for the whole fall (arms out, paddling a few
  degrees). The reference fighters go through a take-off stretch, a tuck at the top and the legs opening on the way down.
- **Cause:**
  - `make_rigged_blob.py` `jump()` keyed one pose family, and `fall()` only paddled.
  - `fighter_view.gd` `_choose_clip` played `jump` by time from the take-off and switched to `fall` on the sign of the vertical speed,
    so nothing knew where in the arc the fighter was.
- **Fix:**
  - The air is posed by vertical speed, not time:
    - `jump` runs from the take-off (rising at `AIR_RISE_SPEED` 0.3 or faster) to the top: stretched long, then the near knee driven
      up, then a tuck at the top;
    - the new `fall_start` runs from the tuck at the top to the legs open, reaching for the floor (falling at `AIR_OPEN_SPEED` 0.15);
    - beyond that the paddling `fall` (re-keyed on the long open pose) loops.
  - A full hop, a short hop, a ledge jump and a double jump (after its flip) each run the same arc on their own timing.
  - Since the pose depends only on the snapshot's velocity, a rollback or a replay seek redraws it exactly.
  - Older rigs stand in `fall` for `fall_start`.
- **Files:** `art/blender/make_rigged_blob.py` (`air_stretch`, `air_tuck`, `air_open`, `air_fast`, `air_between`), `godot/models/base_rig.glb`,
  `godot/scripts/fighter_view.gd` (`_choose_clip`, `AIR_RISE_SPEED`, `AIR_OPEN_SPEED`), `godot/scripts/anim_library.gd` (`STAND_INS`).
- **Tools:** `tests/anim_sheet.gd --what=arc` (a whole full hop at the sim's gravity), and `--demo=fullhop` (a held jump in a match,
  every other frame).
- **Image:** `docs/screenshots/anim/pass2_air_arc.jpg`: the arc sheets before and after, both facings, then a real full hop in a match,
  before above and after below.
- **Checked:** both facings; `anim_library_test`, `fighter_view_test`, `motion_test`, `match_flow_test`, `replay_test` and `look_bench`
  pass.
- **Left:**
  - On the sword fighter the blade covers part of the tuck at the top.
  - A full hop is short (about 38 frames to the landing), so the open pose shows for only a few frames before the landing; long falls
    show it fully.
  - The moves that borrow `jump` as an attack clip (the Needles' bounce, the Settler's rocket hop) now play the rise to the tuck by the
    move's progress; not checked closely.

### Pass 3: wind-ups that coil, hold and snap (every attack, every class)

- **Defect** (audit item 3): the wind-up drifted from rest to the strike at one pace. The chamber was reached at 58% of the startup and
  left at once, so no frame showed a wound-up shape (the wind-up cells of smashes looked like the idle). Punches never drew back: the
  fist's IK started reaching for the hit at 35% of the startup, from the guard. The reference holds its coil for 4-5 frames, then snaps.
- **Cause:** `fighter_view.gd`:
  - `strike_progress` mapped the startup linearly onto the clip;
  - `_blade_target` reached the wind-up at half the startup and swung on from there;
  - `_reach_weight` eased the limbs out from 35% of the startup;
  - the fist's IK swept from the guard hand straight to the hit.
- **Fix:** one shared shape for the startup:
  - the body's clip reaches its chamber key (19 of 60) by `COIL_AT` (45%) of the startup;
  - it creeps through a hold to `SNAP_AT` (80%), then accelerates through the launch into the strike on the first active frame;
  - the blade does the same;
  - the limbs (fists and kicking legs) stay in their coil and reach out only with the snap (`snap_k`);
  - a punch draws its fist back beside the chest (`FIST_CHAMBER`) as the coil comes in (`coil_k`), holds it there, and throws it from
    there;
  - `make_rigged_blob.py`: the chamber keys are pushed deeper (`CHAMBER_AMP` 1.34, was 1.22 for the six-pose clips and 1.12 for the
    `move` clips).
  - Frame data and hitboxes are untouched; the strike still lands on the first active frame.
  - Very short startups (3-5 frames) coil in a frame or two and snap in one, as the reference's jabs do.
- **Files:** `godot/scripts/fighter_view.gd` (`strike_progress`, `coil_k`, `snap_k`, `COIL_AT`, `SNAP_AT`, `FIST_CHAMBER`, `_blade_target`,
  `_reach_weight`), `art/blender/make_rigged_blob.py` (`CHAMBER_AMP`), `godot/models/base_rig.glb`.
- **Image:** `docs/screenshots/anim/pass3_coil.jpg`:
  - whiff demos, before above and after below: the Longsword's forward smash (coiled by frame 5, held, snapping on 9-10), the Claws'
    forward smash (coiled by frame 9, held to 16, snapping on 17-20) and the Claws' jab;
  - the Claws' forward smash facing left.
- **Checked:** facing left; `anim_library_test`, `fighter_view_test`, `motion_test`, `match_flow_test`, `replay_test` and `look_bench`
  pass.
- **Left:**
  - The Longsword's forward smash coils with the blade level behind the hips; the reference raises it overhead behind (step 4, the
    sword's own moves, `SWING_FROM`).
  - The attack clips settle to an upright rest, then the idle sinks into the new stance, so a move ends with a small rise and drop.
    The attack clips' rest should become the stance (next, with the sword's moves).
  - Multihit re-strikes keep their own draw-back (unchanged).

### Pass 4: the Longsword against its reference (first round), and every grounded move kept low

- **Defects** (audit items 4 and 5 and the jab row):
  - The jab wound up with the blade upright over the face.
  - The forward tilt cut from an upright step where the reference lunges about twice as deep.
  - The sword arm stayed locked out straight through the end lag.
  - Every grounded move stood on nearly straight legs and ended rising to an upright stand before the idle sank into the new stance.
- **Cause:**
  - `make_rigged_blob.py`:
    - `_spec_pose` gave every leg a shin of -8 degrees unless a pose named it;
    - the six-pose clips' rest keys (0 and 60) were an upright stand;
    - `sword_ftilt` was posed shallow.
  - `fighter_view.gd`:
    - `SWING_FROM` (jab 70) raised the jab's blade straight up in front of the head;
    - the blade arm's `arm_k` stayed at 1 through the end lag.
- **Fix:**
  - Grounded moves keep a soft knee bend under their pose (`KNEE_SOFT`; poses that name a leg keep theirs), so `plant` sets them as low
    as the stance.
  - The six-pose clips start and end on the battle stance (`_stance_rest`).
  - `sword_ftilt` crouches into its coil (with the face kept in view) and cuts from a deep lunge: stance (50, -40), was (36, -28).
  - The jab winds up behind the head (`SWING_FROM` 120) and the forward smash further back (95).
  - After the last hit the sword arm draws back in as the blade eases home (`_reach_weight`).
- **Files:** `art/blender/make_rigged_blob.py` (`KNEE_SOFT`, `_spec_pose`, `_stance_rest`, `sword_ftilt`), `godot/models/base_rig.glb`,
  `godot/scripts/fighter_view.gd` (`SWING_FROM`, the blade's `arm_k`).
- **Image:** `docs/screenshots/anim/pass4_longsword.jpg`: whiff demos of the jab, forward tilt and forward smash, before above and after
  below, then the forward tilt facing left.
- **Checked:** facing left; `anim_library_test`, `fighter_view_test`, `motion_test`, `match_flow_test`, `replay_test` and `look_bench`
  pass.
- **Left:**
  - The forward smash's coil holds the blade back at the hips; the reference raises the hand overhead with the blade hanging behind.
    The arm here always points along the blade (`_pose_blade`), so this needs a hand path of its own.
  - The neutral air: its windmill round an upright body is the owner's earlier choice; the reference's body rolls with the circle
    (asked).
  - The forward air and the remaining moves are next.

### Pass 5: the brawler family, Claws first: modelled claws that meet the hit

- **Defect** (audit items 6 and 7, the Claws rows):
  - The Claws class showed bare fists: no claw was modelled, and the weapon pivot is hidden for unarmed classes.
  - At the strike the fist stopped 0.65-0.8 units short of the hitbox centre, and the claw arcs were drawn 0.8 past the hand, floating
    ahead of it.
- **Fix:**
  - **The model:** a modelled claw, `art/blender/make_weapons.py` `claw()` (new; the hand weapons by script, like `make_gear.py`),
    written to `godot/models/weapons/claw.glb`. It is 940 triangles, within the prop budget:
    - a rounded steel plate over the knuckles;
    - three thick curved claws ending in blunt hooks;
    - a wrap round the wrist in the class's violet.
  - **The mount:** `fighter_view.gd` `_mount_weapon_models` hangs one on each hand bone (`BoneAttachment3D`), turned along the forearm
    with the back of the hand outward, so the clench, the IK and every swing carry it. It is shown only for class 1, and its steel is
    darker than the blades so it reads against the white gloves.
  - **The reach:** the claw tips now land inside the hitboxes (`reach_claws` before and after, the sim's hitboxes drawn in red), and the
    claw arcs run at the tips (`ARC_PAST_HAND` 0.6, was 0.8).
  - **The forward air:** a fist's overhead forward air now chambers up toward the head (`OVERHEAD_FROM`) rather than at the chest. The
    short arms against the big head keep it at the face, not over it.
  - The family's own stance (`idle_brawler`) and the coil come from passes 1 and 3.
- **Files:** `art/blender/make_weapons.py`, `godot/models/weapons/claw.glb`, `godot/scripts/fighter_view.gd` (`_mount_weapon_models`,
  `claw_parts`, `CLAW_LENGTH`, `HAND_WEAPON_FROM_WRIST`, `ARC_PAST_HAND`, the overhead chamber).
- **Images:**
  - `docs/screenshots/anim/pass5_claws.jpg`: the reach with the hitboxes, before and after; the forward smash and forward air, before
    above and after below; the forward smash facing left.
  - `docs/screenshots/anim/claw_model.png`: the model.
- **Checked:** facing left; `anim_library_test`, `fighter_view_test`, `motion_test`, `match_flow_test`, `replay_test` and `look_bench`
  pass. The "not inside tree" errors in `anim_sheet` runs predate this pass (the same count without the claws).
- **Left:**
  - In the brawler idle the raised claws stand beside the face, touching the goggles' edge on some looks: tilt them forward if it
    reads as covering the face.
  - Gauntlets and Bracers keep their fists (their stance and coil are shared).
  - The claws are not on the old blob rigs.

### Pass 6: sword arcs on the blade's tip

- **Defect** (audit item 7): the swords' swing arcs floated beside or past the blade.
  - The audit measured that the blade does reach its hitboxes, so the arcs were the problem.
  - The jab also drew a tail swept backward behind the head during its wind-up.
- **Cause:** `fighter_view.gd` `ARCS` are fixed circles about a fixed standing shoulder (for example (0.5, 1.15), radius 2.3), drawn at
  `ARC_Z` in front of the body. The blade pivots at the moving hand of a lunging, leaning body.
- **Fix:** `_blade_arc_on_tip`:
  - While a sword's swing is live, after the arm's reach and the hilt's correction, the arc is redrawn on a circle about the weapon
    arm's real shoulder, through the blade's real tip (`BLADE_TIP_X` along the pivot), at the blade's own depth.
  - The authored arc still sets which way the swing runs and its tail.
  - Before the tip comes round to the swing's start, nothing is drawn.
  - Through the fade the arc keeps the last tip circle.
  - It depends only on the frame's pose, so hitlag freezes it and a seek redraws it.
  - It applies to every bladed class: Longsword, Greatsword and Buckler.
  - The claws' arcs were moved onto the claws in pass 5.
- **Files:** `godot/scripts/fighter_view.gd` (`_blade_arc_on_tip`, `BLADE_TIP_X`, `arc_z`, `_draw_arc`, `_update_arc`).
- **Image:** `docs/screenshots/anim/pass6_blade_arcs.jpg`: Longsword jab, forward smash, neutral air and forward air, before above and
  after below.
- **Checked:** the Greatsword's and Buckler's strikes in the pose atlas; `anim_library_test`, `fighter_view_test`, `motion_test`,
  `match_flow_test`, `replay_test` and `look_bench` pass.
- **Left:** the arc's thickness still follows the authored radius, so a short-armed swing's crescent is a little thick near the
  shoulder.

### Pass 7: the Longsword's neutral air (the owner's option B)

- **Defect** (audit item 8): the move read as a downward poke followed by an arm waving.
  - The wind-up held the blade straight down under the body.
  - On frame 17 the general air counter-pitch threw the body forward, head down, as the blade passed behind.
- **The owner's pick (B):** keep the upright windmill, tuck the knees, start the wind-up with the blade behind, and add a partial body
  roll that follows the blade round (about 30-40 degrees, never upside down).
- **Fix (`fighter_view.gd`):**
  - **The roll:** `_nair_roll` replaces the air counter-pitch for a sword's neutral air. Over the second hit's window the body tips back
    with the blade as it goes over the top and round behind (at most `NAIR_ROLL` 32 degrees), then eases upright as the blade comes
    round underneath.
  - **The wind-up:** it raises the blade up and back behind the body. Then the strike brings it round underneath into the rising first
    cut (`SWING_FROM` nair 225, taken the short way round: `SWING_ROUND`).
  - **The tuck:** the knees were already in the clip's strike pose.
- **Image:** `docs/screenshots/anim/pass7_longsword_nair.jpg`.
- **Checked:** facing left; the six tests pass.
- **Left:** the Greatsword's and Buckler's neutral airs are not this move (their own clips), so they are unchanged.

### Pass 8: reactions that stay alive

- **Defect** (audit item 9):
  - The hurt reel froze after 18 frames, and the heavy hitstun and upward launch held one pose after 24.
  - The tumble was the whole body spun stiff, its limbs fixed in the hurt pose.
- **Fix:**
  - **Long stuns** (`make_rigged_blob.py`): `hurt` runs 66 frames, with the arms swinging loosely in turn and the knees giving and
    recovering after the reel (`hurt_flail`). `hurt_heavy` and `hurt_up` run 60, their limbs trailing and swaying (`heavy_sway`,
    `up_sway`). A short stun still shows the snap and the reel on the same frames as before.
  - **The tumble:** a new looping `tumble` clip curls the body round with the arms and legs flailing in turn. `fighter_view.gd`
    `_choose_clip` plays it while the fighter tumbles slower than `LAUNCH_POSE_SPEED` (the head-over-heels spin is unchanged code).
    Older rigs stand in `hurt` for it.
- **Image:** `docs/screenshots/anim/pass8_reactions.jpg`: the tumble before and after, then the Claws facing left. The long-stun sway
  starts after frame 24, past the contact sheet's 30-frame window, so it is not in the image; it shows in long hitstun in a match.
- **Checked:** both facings; the six tests pass.
- **Left:**
  - The launch pose itself (a strong launch flying fast) is unchanged.
  - Ledge hang, get-up and shield break were reviewed in the audit sheets and read acceptably.
  - KO has no pose of its own (the effects carry it).

### Pass 9: the Dagger's and Nodachi's weapons (step 8, the remaining classes, part 1)

- **Defect** (audit item 6): the Dagger and Nodachi held no weapon. Both played the unarmed placeholder set with bare fists.
- **Fix:**
  - **The models** (`art/blender/make_weapons.py`, in the chunky rounded style of the claw and the gear):
    - `dagger()` (900 triangles): a short broad leaf blade with a thick spine, a round-cornered guard, a wrapped grip and a ball pommel;
    - `nodachi()` (788 triangles): a long slim gently curved blade with a chunky spine, a round guard, and a long two-handed wrapped grip
      with a cap, laid out like the game's longsword (blade 0.2 to 2.9 along the pivot).
  - **Wired like the existing weapons** (`fighter_view.gd` `_mount_weapon_models`, now one routine for every modelled hand weapon,
    with `_paint_weapon`):
    - The knife hangs on the weapon hand's bone, the near side for the facing, so it rides every punch the Dagger's moves throw, and its
      point reaches the jab's far hitboxes.
    - The Nodachi is a bladed class (`bladed`): its sword hangs on the weapon pivot, posed from the move data and stretched to its reach
      as the longsword is, and the longsword's mesh is hidden for it.
  - **Mapping** (`AnimLibrary.CLASS_MOVES[13]`): the Nodachi's sword moves play the sword cuts (tilts and smashes), and its down air
    the plunge.
- **Images:** `docs/screenshots/anim/pass9_dagger_nodachi.jpg` (before and after strike frames of both classes, then facing left), and
  `docs/screenshots/anim/dagger_nodachi_models.png` (the models).
- **Checked:** both facings; the six tests pass.
- **Left:**
  - The Nodachi has no swing arcs yet (`ARCS` has no entry for class 13). It keeps the quick family's idle, which holds the sword low.
  - The Dagger's moves keep the shared punches. Its own pose set comes with the remaining classes' signature moves (step 8, part 2).
  - Classes 2-11 have so far had only the game-wide passes (stance, air arc, coil, soft knees, arcs, reactions); their own signature
    keys are next.
- **Follow-up (9b):** the Nodachi, now a sword class, takes the sword stance (its long blade raised in guard) instead of the quick family's
  low crouch. It was removed from `AnimLibrary.CLASS_FAMILY`. Image: `docs/screenshots/anim/pass9b_nodachi_stance.png`. The tests pass.

### Pass 10: the pose review's open items (Staff, Greatsword, the Claws' forward air) and the Nodachi's arcs (step 8, part 2)

- **Staff:**
  - **Defect:** strikes were held one-handed at the end of a straight arm, reading as poking.
  - **Fix:** the other hand takes the shaft further up (`STAFF_GRIP_APART` 0.55, toward the head; the weapon hand is near the foot), so
    the staff swings in both hands. `_aim_arm` with two hands now applies to the staff as to the greatsword.
  - **Also:** held weapons (a staff, a sword with no authored arc) no longer draw the fist smear, which laid a punch's ribbon across the
    chest.
- **Greatsword:**
  - **Defect:** the body stayed upright while the huge blade moved.
  - **Fix:** grounded swings rock the whole body back through the coil (`GREAT_LOAD_BACK` 12) and throw it forward into the cut
    (`GREAT_LOAD_IN` 14), on the lean spring.
  - **Found and fixed:** pass 6's tip arcs had hidden the greatsword's arcs. Tip arcs are now limited to the longsword
    (`TIP_ARC_CLASSES`), and a tip that disagrees with the authored swing keeps the authored arc.
- **Claws' forward air:** re-checked; the free arm no longer flings wide (the free-arm clamps plus the pass 5 overhead chamber). No
  change needed.
- **Nodachi:** swing arcs for its jab, forward tilt, forward smash, dash attack and forward and back airs, fitted to its own active
  frames, in a violet-blue trail. The plunging down air draws none.
- **Image:** `docs/screenshots/anim/pass10_staff_greatsword_nodachi.jpg`.
- **Checked:** the six tests pass; the Greatsword, Buckler and Nodachi were checked in the pose atlas.
- **Left:** the rest of classes 2-11's signature keys, and the Longsword forward smash's overhead hand path.

### Pass 11: the Dagger's own knife moves, and side kicks (step 8, part 2: the quick family)

- **Dagger, defect:** every knife move played a shared punch (`AnimLibrary.CLASS_MOVES` had no entry for class 12). The forward, up and
  down smashes and the gun were one straight-armed punch four times; the forward and up airs, which are kicks in its reference, were a
  chop and an arm sweep.
- **Fix (mapping, `CLASS_MOVES[12]`), after the moveset report's list for the art agent and the reference's hitbox animations:**
  - an inward slash and an outward slash for jabs 1 and 3 (`attack_swing`), the backhanded stab for jab 2 (the shared jab);
  - the double thrust as a lunge (`attack_lunge`), the baseball slide low (`kick_low`), the roundhouse dash attack as a flying kick;
  - the spinning uppercut rising onto the feet (`sword_usmash`), the sweep in front then behind (`sword_dsmash`);
  - kicks for the forward and up airs (`kick_nair`, `kick_uair`, the leg aimed at the hit);
  - the gun drawn and aimed (`blaster`) for every gun slot and the curse bolt; the guard's stances low (`attack_low`); the hook and its
    pull as the up special and a throw.
- **Dagger forward smash (a new clip and a code layer):** the reference leans right back with the knife raised overhead behind, then
  cuts down in front as it dives into a lunge so low the body is nearly level.
  - `attack_knife_smash` (`make_rigged_blob.py` `BODY_MOVES`): leaned back on the back leg, then a lunge deeper than any other grounded
    move (stance 74 / -58, hips -0.2), the torso pitched in 24 degrees so the head (clamped at 8 degrees back) keeps the face to the camera.
  - `fighter_view.gd` `OVERHEAD_CHAMBER`: the knife hand cocks up behind the head through the coil (165 degrees about the shoulder) and is
    drawn back behind the head in depth (`OVERHEAD_BEHIND_Z`), so the knife does not cross the face; the strike comes over the top and down
    onto the hit.
- **Side kicks, defect:** three references' side kicks (the quick fighter's second forward smash kick, the all-rounder's and the rusher's
  forward tilts) played `kick_up`, the stretch kick straight up, upright: at their waist-high hitboxes it read as a toe poke.
- **Fix:** a new `kick_side` (`move_of`): the knee chambered high and tucked with the body turned away, then the leg driven out level as the
  body leans right back from it over the bent standing leg, the arms flung back, one long line from head to foot. Mapped for Needles'
  forward smash, Embers' and Gauntlets' forward tilts.
- **Files:** `godot/scripts/anim_library.gd`, `godot/scripts/fighter_view.gd` (`OVERHEAD_CHAMBER`, `OVERHEAD_BEHIND_Z`), `art/blender/make_rigged_blob.py`,
  `godot/models/base_rig.glb`, `godot/tests/anim_library_test.gd`, `godot/tests/fighter_view_test.gd` (the new pins), `godot/scripts/demo.gd`
  (the `fx_` demos take every class name).
- **Image:** `docs/screenshots/anim/pass11_dagger_side_kicks.jpg`: the Dagger's forward smash before and after (wind-up, strike, follow);
  the strike frames of its jab, forward tilt, down tilt, up smash, down smash, forward air, up air and gun, before above and after below;
  Embers' and Gauntlets' forward tilts before and after; Needles' forward smash before and after.
- **Checked:** both facings (contact sheets facing left); the six tests pass.
- **Left:**
  - The cocked knife's tip shows above the hat behind the head through the coil (the short arm cannot raise it clear of the head).
  - Needles' up tilt: its hitboxes stand straight above the head, and the kick, capped at 65 degrees on this body (`HIGH_KICK`), points at
    the camera and reads as an arm behind the arc. A body-proportion limit; not changed.
  - The Dagger's knife draws the fist's ribbon smear on its cuts, not an arc.

### Pass 12: the brawlers' own blows (Bracers' two-fisted moves, Gauntlets' charged punch)

- **Bracers, defect:** every move was one fist thrown, the other held at the chest, including the reference's two-handed blows (the clap
  far in front, both fists hammered down in front in the forward air, the aerial clap). An attack clip cannot raise the free arm: it is
  held in close to the body (`make_rigged_blob.py` `pose`, `FREE_ARM_*`), so the clip's two-handed down air sweep showed one arm.
- **Fix (`fighter_view.gd`):**
  - `BOTH_FISTS` (forward smash, forward air, aerial clap): the other arm reaches with the striking one, its fist just beside and behind it
    (`BOTH_FISTS_APART`), through the coil, the strike and the recovery (`_aim_arm` with two hands, as the greatsword and staff).
  - The forward air cocks both fists up beside the head (`OVERHEAD_CHAMBER`, now [degrees, depth] per move: 110 degrees, beside the head;
    hidden behind it, as the knife is, they did not read), then hammers them over the top and down in front.
  - The up smash stays one-handed: two fists straight up covered the face.
- **Gauntlets, defect:** the charged punch, its reference's signature move, played the shared heavy strike: at 4 frames before the hit it
  was an ordinary stance, the fist at the chest, after a 52-frame wind-up.
- **Fix:** a new `gaunt_punch` clip (`BODY_MOVES`): turned right away from the foe (spine twist -58), sunk onto the bent back leg, the free
  arm thrown out ahead to aim; then a huge lunge with the shoulders turned through. The fist is pulled far back behind the hip through the
  wind-up (`DEEP_CHAMBER`) and thrown from there. Mapped for the punch and its reversed and aerial versions.
- **Files:** `godot/scripts/fighter_view.gd` (`BOTH_FISTS`, `both_fists`, `DEEP_CHAMBER`, `OVERHEAD_CHAMBER`), `art/blender/make_rigged_blob.py`,
  `godot/models/base_rig.glb`, `godot/scripts/anim_library.gd`, `godot/tests/anim_library_test.gd`.
- **Image:** `docs/screenshots/anim/pass12_brawlers.jpg`: the Bracers' forward smash, up smash and forward air (wind-up, strike, follow),
  before above and after below; its forward air in a match, frame by frame; the Gauntlets' charged punch before and after, then frame by
  frame.
- **Checked:** facing left (contact sheets); the six tests pass.
- **Left:** the Bracers' arms-out spins (neutral air, up special) and the hand slap still play one-armed clips; its giant punch's wind-up
  (the arm swung round as it charges) is the shared heavy strike.

### Pass 13: spins that spin (Embers, Bracers) and the bounder's own specials (Clutch)

- **Spins, defect:** the references' spinning moves played still poses with one fist thrown: the all-rounder's down air (a spinning top,
  arms out) played the two-handed down air sweep; the Hauler's neutral air played a swing; its grounded and aerial spins (up special and
  its aerial version) played the shared uppercut.
- **Fix:**
  - `fighter_view.gd` `SPIN_MOVES` and `_whirl`: the whole body turns about its upright axis over the move's spinning frames ([first,
    last, turns], eased in and out by a smoothstep), mirrored by the facing. It depends only on the move's frame, so hitlag freezes it and a
    seek redraws it. Embers' down air (3 turns), Bracers' neutral air (1.5), up special (6) and aerial spin (7).
  - New clips `spin_arms` and `spin_arms_air` (`make_rigged_blob.py`): both arms held out level, one ahead and one behind along the line of
    the fight (held out to the sides of the turned body, the near fist sat in front of the face), lifting as the spin builds; feet apart on
    the ground, knees drawn up in the air. Both arms are the clip's (`AnimLibrary.OWN_ARMS`).
- **Clutch, defect:** its specials were placeholders: the tongue played the shared heavy punch, the drop (hop, rise, seat-first fall)
  played the crouched low strike throughout.
- **Fix:** the tongue plays the forward headbutt (the head lunged forward, mouth first); the drop a new `body_drop` (`BODY_MOVES`, the body
  is the weapon): crouched for the hop, then sat down in the air, legs out level in front, leaning back a little, arms out to the sides
  (raised, the near arm crossed the face). Its aerial version too; its landing plays the landing.
- **Files:** `godot/scripts/fighter_view.gd`, `godot/scripts/anim_library.gd`, `art/blender/make_rigged_blob.py` (`spin_pose`, `body_drop`,
  `grounded`), `godot/models/base_rig.glb`.
- **Image:** `docs/screenshots/anim/pass13_spins_bounder.jpg`: Embers' down air, Bracers' neutral air and up special, Clutch's drop and tongue
  (wind-up, strike, follow), before above and after below; then Embers' down air, Bracers' up special, Clutch's drop and tongue frame by
  frame.
- **Checked:** facing left (contact sheets); the six tests pass.
- **Left:** the Dagger's corkscrew neutral air and the Popgun's spinning neutral air could use `SPIN_MOVES` too (their weapon or kicking leg
  would need holding still through the turn).

### Pass 14: the Wanderer's sword trails and its spin attack (Buckler)

- **Defect:** the Buckler's sword swings drew no trail at all (`ARCS` had no entry for class 9, though it is listed for tip-following
  arcs), and its spin attack (up special), its reference's signature, played the shared uppercut with the body still.
- **Fix (`fighter_view.gd`):**
  - `ARCS[9]`: the jab, jab 2, forward, up and down tilts, dash attack, forward smash, down smash (front then behind) and forward air,
    fitted to the hitboxes' angles about the shoulder (`content/base.pfc`), a little smaller than the hitboxes reach (its sword is shorter
    than they are), and started two or three frames after the hitboxes' first frame, so the trail grows from the blade instead of ahead of
    it (checked frame by frame against the blade). The thrusts, kicks and the spin draw none.
  - The spin attack: the body turns round twice over frames 5-14 (`SPIN_MOVES`), crouched low and wide (`sword_dsmash`), the sword held
    out level at the hip (`SPIN_BLADE_TIP`) so the turning body sweeps it round; the aerial spin turns six times, tucked.
- **Image:** `docs/screenshots/anim/pass14_buckler.jpg`: the forward tilt and forward smash before and after, frame by frame; the up tilt's
  trail; the spin attack before and after.
- **Checked:** facing left (contact sheet); the six tests pass.
- **Left:** facing left, the Buckler's blade points at the camera in some recoveries and reads as a bare hilt (as on the base branch).

### Pass 15: the Scamp's own moves (Popgun) and the Settler's spinning neutral air (Pockets)

- **Defect:** the Popgun had no clips of its own (`CLASS_MOVES` had no entry for class 11): every move was the shared punch, the forward,
  up and down smashes one pose three times, the kicks and the flip kicks punches.
- **Fix (mapping, `CLASS_MOVES[11]`, after the moveset report's list for the art agent):** a side kick ending the jab, slaps swung across
  for the forward tilt (both hands: `BOTH_FISTS`) and forward smash, flip kicks overhead for the up smash and up air, the slaps low in front
  then behind for the down smash, kicks for the forward and back airs, both feet stomped down for the down air, the popgun drawn and aimed,
  the peel tossed over the shoulder, the cartwheel as a flying kick, and a spin with both arms out for the neutral air (`SPIN_MOVES`).
- **Pockets:** its reference's neutral air spins with the arms out; it played the sword's windmill pose. Now `spin_arms_air` with the body
  turning 2.5 times.
- **Image:** `docs/screenshots/anim/pass15_popgun_pockets.jpg`: the Popgun's jab, forward tilt, forward smash, up smash, down smash, neutral,
  forward, up and down airs at the strike, before above and after below; then the Pockets' and the Popgun's neutral airs frame by frame.
- **Checked:** facing left (contact sheets); the six tests pass.
- **Left:** the monkey flip, the rocket and its latch keep their fallbacks (they need their own poses: a flip, a ride, a seat on a head).

### Pass 16: the Longsword forward smash raised overhead (the open item from passes 3, 4 and 10)

- **Defect:** the coil held the blade level behind the hips, the hand at the chest. The reference raises the hand overhead from frame 6,
  the blade hanging down behind, and is fully coiled by frame 8.
- **Cause:** the arm always points along the blade at the hit's planned path (`_pose_blade`); the wind-up only turned the blade's angle
  (`SWING_FROM` 95), never the hand.
- **Fix (`fighter_view.gd` `BLADE_OVERHEAD`):** through the coil the sword hand goes up to 100 degrees about the live shoulder (as high as the
  arm reaches: beside the top of the head, behind it in depth, `BLADE_OVERHEAD_Z`, so the blade does not cross the face) and the blade hangs
  down behind at 225 degrees; through the snap the hand comes down and the blade comes over the top (the angle is eased in degrees, not the
  short way round, which would have swung it underneath) onto the cut. The strike and the follow-through are unchanged.
- **Image:** `docs/screenshots/anim/pass16_longsword_fsmash_overhead.jpg`: the whiff, frame by frame, before above and after below; frame 6
  close up, before and after.
- **Checked:** facing left (contact sheet; the blade foreshortening toward the camera in that sheet's recovery frames is the same on the
  base branch); the six tests pass.
- **Left:** the short arm cannot lift the hand clear above the head: it ends beside the top of it.

### Pass 17: secondary motion checked in live matches (step 9: take-off stretch, landing squash, hat spring, idle fidgets)

- **How it was checked:** the springs and the squash run on real time (`delta`), so frame-by-frame demo captures (slow frames) settle them
  at once and misread them. They were traced in a real-time computer match (`--demo=cpu`, the values printed every frame for one fighter)
  and captured with `--fixed-fps 60` for the images.
- **Take-off stretch, defect:** kicked from the crouch, the stretch peaked four frames after the feet left the ground; the first airborne
  frame was still squashed (0.13 against the crouch's 0.25). **Fix:** stretched at once on the first frame in the air (`TAKE_OFF_STRETCH`
  -0.15, measured -0.18 with the spring), easing to the rise's own stretch.
- **Landing squash, defect:** kicked from the fall speed, the squash took four frames to reach its depth (0.12 on the contact frame), so it
  peaked as the landing lag ended, and after an aerial it overshot to 0.40 (the body at 60% of its height). **Fix:** squashed at once on
  touching down, deeper for a faster fall (`LAND_IMPACT` 0.1 plus `LAND_IMPACT_PER_SPEED` 0.55 times the fall speed: measured 0.22 on the
  contact frame), then rebounded by the spring; after an aerial it now peaks at 0.28.
- **Hat spring, defect:** on a landing or a take-off the hat dipped about 0.04, a twentieth of the head: not visible at match distance.
  **Fix:** `HAT_DROP_KICK` 13 (was 6): about 0.09, within the spring's limits. The sideways roll (up to 0.33 radians on a dash) already read;
  unchanged.
- **Idle fidgets, defect:** the long idle's fidget (`idle_look`) was posed on the sword stance, so a brawler's, quick, heavy, caster or
  tool fighter's fidget popped out of its own stance into the sword guard and back. The sword one also straightened the chest and dropped
  the stance's side bend. **Fix:** a fidget per family on its own stance (`idle_look_<family>`, `make_rigged_blob.py` `glance_on`,
  `look_clip`; `AnimLibrary.FAMILY_CLIPS` now includes `idle_look`), keeping each stance's lean and side bend.
- **Files:** `godot/scripts/fighter_view.gd` (`TAKE_OFF_STRETCH`, `LAND_IMPACT*`, `HAT_DROP_KICK`, the fidget's clip), `godot/scripts/anim_library.gd`,
  `art/blender/make_rigged_blob.py`, `godot/models/base_rig.glb`.
- **Image:** `docs/screenshots/anim/pass17_secondary_motion.jpg`: a short hop at a fixed 60 frames a second (the crouch, the take-off and the
  landing), before above and after below; then the fidget of the brawler and the quick fighter, before and after.
- **Checked:** the six tests pass (`motion_test` covers the springs' freeze and rest after a seek).
- **Left:** the squash and lean springs step on real time, not the sim frame (older than the motion polish springs), so hitlag does not
  freeze them; unchanged here. Not tried with a controller.

**Smear fix (art director, 2026-10-25, after the art audit of that day).** In the paper look the fist and foot smear (`smear.gd`) read
as a glassy blue tube across the body on straight strikes (the Gauntlets' charged punch, the Dagger's forward smash): along a straight
path the ribbon lies over the arm, and its pale edges on both sides, white stripe and cream core shaded it as a lit glass rod. A smear
is now drawn only when the limb's path bends (no point further than `STRAIGHT`, 12% of its length, from the line between its ends
means no ribbon), as the reference draws none on straight thrusts. In paper it is a flat strip in a deeper class colour with a tint
along one side and one thin cut edge, 70% as wide, with no core and no ring round its head. Before/after:
`docs/screenshots/anim/smear_fix.png`. Checked: anim_library, fighter_view, motion, match_flow and replay tests.

### Round 2 (2026-10-25, from the art audit of that day, `docs/art/ART_AUDIT_2026-10-25.md`)

### Pass 18: the idle judged at the match camera (round 2 item 1)

- **Defect:** at the match camera (`--demo=stage`, both fighters standing) the sword family's head sat straight above its hips: the knees
  bent, but the chest and head stayed high. Contact sheets, close up, had hidden it.
- **Cause (`make_rigged_blob.py`):**
  - the sword stance leaned the chest 14 degrees and bent it 12 toward the foe, too little once the game turns the body to the camera;
  - the head may tip back at most 8 degrees against the chest (`HEAD_BACK_LIMIT`), so a deeper lean would have pointed the face at the
    floor and shown the hat's crown (tried first: it did).
- **Fix:**
  - The sword stance: chest lean 26 (was 14) and side bend 20 (was 12) over deeper knees (front 50/-82, was 44/-72; back -24/-54, was
    -20/-46). The head is now out over the front knee.
  - In idle clips the head may tip back 24 degrees against the chest (`HEAD_BACK_IDLE`), and each idle's head counters its lean, so the
    face stays level toward the camera (sword -19, brawler and quick -16).
  - The heavy, caster and tool fighters lean a step further (heavy 18 and side 12, caster 8 and 6, tool 6 and 5); the caster and tool stay
    the most upright, by design.
  - The attacks' rest keys keep the old stance values (`ATTACK_REST`): the attacks' exaggeration already sets them about as low and
    leaned as the new idle, so no move changes.
- **Image:** `docs/screenshots/anim/r2_pass18_idle_match_camera.jpg`: the stage demo at the match camera, before left and after right, the
  first fighter facing right and the second facing left (eight classes, all six families).
- **Checked:** both facings; the six tests pass.
- **Left:** the caster and tool families are still upright by design (a design question if the owner wants them lower).

### Pass 19: the family runs actually play (round 2 item 5, found on the way to the walk)

- **Defect:** every class ran the shared run. The quick, heavy, caster and tool runs built in pass 1 (`run_<family>`) never showed in a
  match or a contact sheet.
- **Cause:** `fighter_view.gd` `_choose_clip` returned the shared `run` for the Run state before it reached the family lookup
  (`AnimLibrary.family_clip`), and the gait code (a dash running on in step, the footfall squash and dust) tested for the clip names
  `run` and `dash` only.
- **Fix:** the Run state plays `AnimLibrary.family_clip(class, "run")`; the gait checks use `_gait_clip` (the dash, the shared run and
  every family run, not `run_stop`). `fighter_view_test` now pins the quick family's run and the sword family's shared run.
- **Image:** `docs/screenshots/anim/r2_pass19_family_runs.jpg`: run contact sheets of the quick, heavy, caster and tool classes, before
  (the shared run) and after (their own).
- **Checked:** the six tests pass. Foot slide (`motion_probe`, per family): the runs slide about as much as the shared run (0.4-0.75 of
  their speed, the heavy run 0.2); the probe's run cases are noisy (some speeds find no planted frame).
- **Left:** the runs' foot slide is unchanged (out of this round's list; noted for a later locomotion pass). The brawlers keep the shared
  run (`FAMILY_SHARES`) and the sword family has no run of its own.

### Pass 20: a walk per family, with planted feet (round 2 item 2)

- **Defect:** one shared walk for every class, a big swaggering stride that stood up out of each family's stance (the sword fighter
  strolled upright between two crouched idles), and its planted foot slid 0.23-0.31 of the ground speed (`tests/motion_probe.gd`).
- **Cause (`make_rigged_blob.py`):** the walk was keyed as leg angles (`WALK_THIGH` and friends), so where the planted foot went came out
  of the angles, eased between keys (so it sped up and slowed down while down), and `plant` then moved the hips to put the lower foot on
  the floor. Its stride did not match the 2.2 units a cycle the game plays it at (`fighter_view.gd` `WALK_TEMPO`).
- **Fix:**
  - **Feet authored, not angles:** `walk_style` gives each foot a path. While down it moves straight back under the body at an even pace;
    then it swings through on a Hermite curve whose ends move at that same pace (it leaves the floor still going back and lands already
    going back, so it neither drags nor skids). `foot_to` bends the thigh (forward and out) and the knee to put the ankle exactly there: a
    Newton solve on the rig, so the hips' bob, twist and roll are allowed for. A miss over 0.01 is printed by the build.
  - **The stride matches the ground:** a clip's length sets how far one cycle covers (`frames` x 0.055), and the foot's sweep while down
    is that distance seen through the body's three-quarter turn (`WALK_TURN_SHARE` 0.80, set from the probe). The quicker walks take
    short floor contacts with a little bounce between steps rather than sliding.
  - **A walk per family** (`WALK_STYLES`), each in its own stance: the sword family a guarded advance (the free hand up, small swings),
    the brawlers a hunched prowl with both fists up, the quick fighters low and springy with the lead hand out, the heavy one wide with
    the hips rolling and the arms swinging big, the caster upright and neat, the tool users a loose stroll. The heads counter the chest's
    lean as in the idles. Mapped with `AnimLibrary.FAMILY_CLIPS` (the sword family's is the shared `walk`); `fighter_view.gd` plays
    `family_clip(class, "walk")`.
- **Foot slide** (`motion_probe`, ratio of the ground speed, at 0.08 / 0.14 / 0.19 units a frame):

  | Family (class) | Before | After |
  |---|---|---|
  | Sword (0) | 0.31 / 0.28 / 0.23 | 0.11 / 0.09 / 0.08 |
  | Brawler (1) | same shared walk | 0.15 / 0.10 / 0.09 |
  | Quick (2) | | 0.15 / 0.09 / 0.06 |
  | Heavy (8) | | 0.16 / 0.11 / 0.10 |
  | Caster (3) | | 0.08 / 0.08 / 0.08 |
  | Tool (10) | | 0.11 / 0.10 / 0.08 |

- **Image:** `docs/screenshots/anim/r2_pass20_family_walks.jpg`: walk contact sheets of six classes (one per family), before (the shared
  walk) and after; then the sword fighter walking in a match (`--demo=walk`), before and after.
- **Checked:** facing left (contact sheets); the six tests pass; `fighter_view_test` pins the brawler's walk.
- **Left:** a very slow walk (the stick barely tilted, the clip at its slowest, `clampf` 0.5) still slides; the runs' slide is unchanged
  (pass 19). The quick walk's swinging foot falls up to 0.09 short of its path at the top of the knee lift (the thigh's limit); it does not
  show as a slide.

### Pass 21: the Dagger's cocked knife kept behind the head (round 2 item 3)

- **Defect:** through the forward smash's coil (frames 6-12) the knife's point stuck out over the top of the head, a grey spike through
  the hat's brim (pass 11's open item).
- **Cause:** `fighter_view.gd` `OVERHEAD_CHAMBER` cocked the knife hand at 165 degrees about the shoulder, only 0.3 behind the body in
  depth, so the knife, along the forearm, pointed up past the head.
- **Fix:** the hand cocks at 150 degrees and 0.5 behind: the knife is held hidden behind the head through the coil and comes over the top
  onto the cut as before (the strike and follow-through are unchanged).
- **Image:** `docs/screenshots/anim/r2_pass21_dagger_knife.jpg`: frames 4-14, before above and after below.
- **Checked:** facing left (contact sheet); the six tests pass.
- **Left:** the knife does not show at all through the coil now; the raised hand reads by the arm.

### Pass 22: the Hauler's giant punch (Bracers; round 2 item 3)

- **Defect:** the charge (neutral special, up to 130 frames, no hit of its own) stood in a still guard with both fists at the chest, and
  the punch it throws (extra moves 0-3) played the fallback swing. The reference swings its arm round and round, faster as it charges, then
  throws a huge lunging punch.
- **Cause:** `AnimLibrary.CLASS_MOVES[7]` had no entry for these moves; with no hitbox the charge's progress ran straight to the settle.
- **Fix:**
  - `gaunt_charge` (`make_rigged_blob.py` BODY_MOVES): planted wide, leaning back a touch, the chest open to the camera, the other fist low
    in front. `fighter_view.gd` `HELD_COIL` holds a charge in its wind-up pose for as long as it lasts, breathing a little.
  - `WINDMILL`: the punching fist goes round a circle about the arm's live shoulder, over the top behind the head (so it never crosses the
    face) and down in front, from one turn in 30 frames to one in 11 at full charge. By the move's frame only, so a freeze holds it and a
    seek redraws it.
  - The punch (extra moves 0-3) plays the rusher's charged punch, `gaunt_punch`, with its fist drawn back behind the hip (`DEEP_CHAMBER`)
    and starting coiled (`COILED_START`) rather than rising to the rest pose first.
  - `demo.gd`: `_again` in an `fx_` demo's name presses the special a second time 30 frames in (`fx_bracers_nspecial_again_whiff`).
- **Image:** `docs/screenshots/anim/r2_pass22_bracers_giant_punch.jpg`: the charge's frames 8-22 before and after, then the punch thrown
  (before, the demo never threw it: it had no second press).
- **Checked:** facing left (contact sheet); the six tests pass.
- **Left:** the charge's white flash (effects) covers the body for the first frames of the punch.

### Real-speed motion audit (round 2, after the owner's note: unnatural bends and weightless motion)

Captured every frame of every Longsword and Claws move at a fixed 60 frames a second (`--demo=gallery --gallery-step=1`, both facings,
`--gallery-facing=-1`; new flags in `demo.gd`), and measured each grounded clip's hip height per key in the rig build (`HIPS_DEBUG=1`).
Ranked by how visible it is at match speed:

1. **Every grounded move bobbed:** its first frame dropped the hips from the idle's 0.645 to 0.32-0.36 (a squat), and its last frame stood
   back up; the moves keyed with `move` rose 0.3 instead. Seen on every attack, every class. (Pass 23.)
2. **Wind-ups that float up:** the Longsword's forward smash coiled at hip height 0.80 on straight legs, the Settler's chop at 0.87. (Pass 23.)
3. **Hands over the face:** the sword hilt over the eyes for eight frames of the up tilt; the claws over the face through the forward air's
   wind-up; every fist's chamber placed in front of the face. (Pass 24.)
4. **Arm snaps:** the punching arm's reach switched off in one frame when a move ended (the Claws' jab snapped to the guard on frame 23),
   and switched on at full weight on frame 1. (Pass 25.)
5. **Punch ribbons:** a straight punch thrown from a chamber behind the hip still drew a ribbon, along the hips, apart from the arm. (Pass 26.)
6. Smaller: the shield, grab, jump crouch, landing and spot dodge started or ended standing 0.17-0.24 taller than the idle (pass 23).
   Not yet fixed: see the round's report.

### Pass 23: no bob into or out of a move

- **Cause (`make_rigged_blob.py`):** the attack rest keys (`_stance_rest`) put the stance's leg angles through `pose`, which multiplies them by
  the clip's exaggeration (`AMP` 1.7), so the rest key was a squat; `move` clips' rest keys stood tall; the state clips kept their own
  upright legs; coils were not limited.
- **Fix:** the rest and recovery keys are the idle's stance exactly (divided by `AMP` before `pose`), with the idle's head counter-tilt
  (`STANCE_HEAD`, `HEAD_LIMIT_NOW`); `move` clips' rest keys too (`_stance_over`); the shield, grab, throw, kick, jump crouch, landing and
  spot dodge start and end on the stance's legs; a grounded attack's coil never stands the hips above the stance (`COIL_HIPS_MAX` 0.64,
  `_sink_to`); the Longsword's forward smash coil sits back on its bent back leg with the front foot reaching ahead. Three aerial clips
  whose names do not end in "air" are no longer treated as grounded.
- **Image:** `docs/screenshots/anim/r2_pass23_no_bob_into_moves.jpg`.

### Pass 24: hands kept off the face

- **Cause (`fighter_view.gd`):** the high sword arm (`HIGH_BLADE_ARM` 80) put the hilt in front of the eyes; a fist's chamber and overhead
  cock were placed from the rest-pose shoulder, 0.4 ahead of the real one once the body turns to the camera.
- **Fix:** every class's arm is placed from its live shoulder; the high sword arm points at most 62 degrees up; any hand nearer the head's
  middle than `HEAD_CLEAR_R` goes behind the head in depth (`HEAD_BEHIND_Z`), so the blade or claw still shows round it.
- **Image:** `docs/screenshots/anim/r2_pass24_face_clear.jpg`.

### Pass 25: the arm's reach eases in and out

- **Fix (`fighter_view.gd`):** the arm IK weight eases in over a punch's first frames (`IK_EASE_IN` 3, finished before the first active
  frame) and lets go over `IK_FADE_FRAMES` (6) sim frames from where it was when the reach ends (a move over, a sword fighter breaking
  into a run). By sim frames, so hitlag holds it; a rollback or seek (`MotionSprings.jumped`) drops it at once.
- **Image:** `docs/screenshots/anim/r2_pass25_reach_eases.jpg`.

### Pass 26: no ribbon on a punch that barely turns

- **Fix:** `smear.gd` draws no ribbon on a path turning through less than `MIN_TURN` (50 degrees); `fighter_view.gd` traces it only from the
  snap (not the draw-back into the coil) and at the fist's own depth (`SMEAR_FORWARD`), not the arcs' fixed depth. Arced kicks and swings
  keep theirs.
- **Image:** `docs/screenshots/anim/r2_pass26_no_punch_ribbon.jpg`.
- **Checked (passes 23-26):** both facings (gallery captures facing left); the six tests pass.

### Pass 27: the body's squash, lean and lunge springs freeze in hitlag (round 2 item 4)

- **Defect:** a strong hit's freeze still wobbled: the body's squash, lean and lunge springs (`fighter_view.gd`, older than the motion
  springs) were integrated on real time, so they kept moving while the sim was frozen by hitlag or paused, and a rollback or seek left
  them mid-swing.
- **Fix:** `_body_spring_time` gives them the motion springs' clock: they move only while the sim frame advances (frozen in hitlag and a
  pause, `MotionSprings.PAUSED_AFTER`), and a jump in the sim frame (a rollback, a seek) puts them straight on their targets.
  `motion_test` checks both (a landing squash held through hitlag; the springs at rest on their targets after a seek).
  `tests/freeze_probe.gd` prints the squash through a frozen landing.
- **Image:** `docs/screenshots/anim/r2_pass27_springs_freeze.png`: the squash through a landing frozen for 12 frames, before (grey, it keeps
  moving) and after (red, held).
- **Checked:** the six tests pass.

### Pass 28: the fighters stand side-on (the owner's pick D from the stance turn mocks)

- **Defect** (the owner, with a screenshot): in the idle and the run the chest and face were nearly square to the camera, where the
  reference fighters stand side-on or three-quarter, turned along the stage toward the opponent, the face only partly toward the camera.
- **Cause (`fighter_view.gd`):** one global turn. The hips were turned `BODY_TURN` 60 off the camera, the chest twisted back
  `CHEST_TO_CAMERA` 24, the head turned `HEAD_TOWARD_FOE` 10 and clamped at `FACE_AWAY_MAX` 32, and moves added `BODY_TURN_IN_ATTACK` 14.
- **The mocks** (`docs/screenshots/anim/turn_mock/stance_turn_options.jpg`): A today, B three-quarter, C near side-on (loses the far eye), D a
  split turn. The owner picked D, with its two extras.
- **Fix:**
  - **Split turn:** `BODY_TURN` 80, `CHEST_TO_CAMERA` 6, `HEAD_TOWARD_FOE` 0, `FACE_AWAY_MAX` 40. The body stands side-on, and the face
    clamp holds the head back toward the camera, so the face shell's slide keeps both eyes on the face.
  - **Attacks:** `BODY_TURN_IN_ATTACK` 0. A move opens only the chest toward its hit (`CHEST_IN_ATTACK`) and never passes side-on.
  - **Far arm** (`make_rigged_blob.py`): carried further forward and in, in every family's idle (`FAR_ARM_FWD` 18, `FAR_ARM_IN` 8 for the
    sword idle, and each style's `arm_r`), the run (its swing centred forward) and the walks, so its hand shows past the chest.
  - The weapon stays in the near hand.
  - The menu and select previews (`hold_pose`) keep their own front view; only the far arm's small change shows there (checked with
    `tests/roster_shot.gd`).
- **Measured** (`tests/turn_probe.gd`, degrees off the camera, Longsword and Claws):

  | | Hips | Chest | Face | Both eyes |
  |---|---|---|---|---|
  | Idle, before | 55 | 50 | 32 | 100% |
  | Idle, after | 75 | 88 | 40 | 100% |
  | Run, before | 62 | 32 | 29 | 100% |
  | Run, after | 82 | 72 | 37 | 100% |

  - The far hand ahead of the chest, before the re-key and after it: 0.41-0.45 to 0.43-0.47 for the sword and brawler idles, 0.27 to 0.36
    for the caster, 0.18 to 0.39 for the tool users, 0.31 to 0.39 for the quick fighters. Today's front turn had 0.57.
  - The silhouette is a little narrower: the run 1.26 to 1.14.
  - Every family reads 100% for both eyes, both facings.
- **Image:** `docs/screenshots/anim/r2_pass28_side_on_stance.jpg`: before and after (the idle, the runs both ways, the Longsword's forward
  tilt and the Claws' forward smash), then every family at the match camera.
- **Checked:** both facings and all six families; the weapon classes (Longsword, Staff, Greatsword, Buckler, Nodachi); the brawler
  fists-up idle; the six tests pass (fighter_view_test pins no turn values).
- **Also:** `demo.gd` `loco` (locomotion every frame for the motion audit), and shots for `dashleft`.

### Pass 29: no hand over the face in the air or behind the shield (found by the side-on turn)

- **Defect:** once the body stood side-on (pass 28), the near arm of every unarmed class came up over the face whenever it was falling
  and through the shield, and over the mouth at the top of a jump. Measured by the new `tests/face_cover_probe.gd`: 100% of the frames
  of the open fall and the fast fall for the brawler, quick, caster, heavy and tool families, and 100% of the shield. The heavy run had 28%.
- **Cause (`make_rigged_blob.py`):** poses written for the old three-quarter view held the arms "out to the sides" (`out` 64-96). Side-on,
  out points the near arm at the camera.
- **Fix:**
  - `air_open` and `air_fast`: the near arm reaches forward and low along the stage, the far arm back or raised behind the head.
  - `air_tuck`: the near fist is drawn in at the chest.
  - The shield guard holds the fists in at the chest.
  - The heavy run's arm swing is 1.05 (was 1.25).
- **Measured:** the face is covered in 0% of the frames in every state clip and family, apart from 8% of the sword's jump crouch (the hilt
  passes) and 8% of the tool run.
- **Image:** `docs/screenshots/anim/r2_pass29_air_shield_face.jpg`.
- **Checked:** the six tests pass.

### Pass 30: the face kept up in pitched aerials (real-speed audit, every class)

- **Defect:** in aerials that pitch the whole body forward (the Longsword's forward, up and down airs; the same on every class's curled
  airs), the head looked at the floor, and only the hat's crown showed for 10-15 frames.
- **Cause (`fighter_view.gd` `_face_camera`):**
  - the face-down limit was measured in the stage frame, which leans with the body, so the whole body's pitch (`lean`, the air
    counter-pitch) was never counted;
  - the limit was lifted about the camera's axis, which on a side-on head rolls it sideways;
  - the head was turned before it was lifted, which spun a head looking straight down.
- **Fix:**
  - the head's pitch is measured against the world too (except while the body turns over on purpose: a flip, a roll, a tumble,
    `turning_over`);
  - it is lifted about the head's own side-to-side axis (`_rotate_bone_about`, at most `FACE_UP_MOST` 45);
  - it is lifted before it is turned, and the pair runs three times so both limits hold.
- **Image:** `docs/screenshots/anim/r2_pass30_face_in_aerials.jpg`.
- **Also:** `PF_FACE_COVER=1` in the environment makes a match print every frame where a hand or hilt covers the face; the headless
  `tests/face_cover_probe.gd` does the same for the state clips.

### Pass 31: the last stand-in specials (round 2 item 3: Bracers' hand slap, Popgun's monkey flip, latch and rocket)

- **Defect:** these moves played fallbacks:
  - the hand slap was a one-armed swing;
  - the monkey flip and the latch were the generic swing;
  - the rocket was the shared uppercut, then a punch.
- **Fix (`make_rigged_blob.py` BODY_MOVES, `AnimLibrary.CLASS_MOVES[7]` and `[11]`, `OWN_ARMS`):
  - `slap_ground`: both arms raised behind the head, then both palms slammed on the floor in front over deep knees, striking again on
    each hit of the loop.
  - `leap_grab`: crouched, then a dive with both hands reaching. `fighter_view.gd` `FLIP_MOVES` turns the body over forward between
    frames 12 and 24, by the move's frame.
  - The flip's kick is a leg straight out (`kick_nair`).
  - `latch_sit`: astride the head, the hands raised, then raking down to scratch. The latch attack does the same; the jump off is a stomp.
  - `rocket_fly`: the charge crouched and gripping the straps, held for as long as it lasts (`HELD_COIL`); the flight stretched long,
    the arms swept back, with the body pitched along its path (`FLIGHT_MOVES`). The crash plays the upward launch reaction.
  - The aerial clap plays the two-fisted down sweep.
- **Image:** `docs/screenshots/anim/r2_pass31_standin_specials.jpg`: each move before and after, frames 4-60.
- **Checked:** the six tests pass; facing right only (the clips are mirrored facing left like every other).
- **Left:** the flip's body roll is about its middle; the reference flips about the hands.

### Pass 32: a face guard for every class (real-speed audit, all 14 classes)

- **How it was found:** every class's 16 moves at 60 frames a second (`--demo=gallery`), with `PF_FACE_COVER=1` logging each frame in
  which a hand or hilt is in front of the head and within 0.4 of the face's middle. Total before: 1,376 frames. The worst were the
  Settler (218), the Clutch (207), the Greatsword (168), the Claws (154) and the Gauntlets (137): up moves, raised specials, two-handed
  grips, and the clips' free arms.
- **Fix (`fighter_view.gd` `_guard_face`):** after the arm IK, a hand that ends in front of the face is moved out past the face's edge
  on screen (`FACE_GUARD_R`) and half way back toward the head, by the arm's IK (`_ik_arm`, split out of `_aim_arm`). It is eased in by
  how near the face it is, so it never pops. Arms the clip poses are blended toward the guarded pose, and arms the IK already holds have
  their goal moved.
- **Measured** (frames with the face covered, before then after): Longsword 45 to 10, Claws 154 to 0, Needles 44 to 0, Embers 21 to 5,
  Gauntlets 137 to 1, Staff 95 to 10, Greatsword 168 to 81, Bracers 54 to 7, Clutch 207 to 5, Buckler 43 to 16, Settler 218 to 59,
  Popgun 84 to 0, Dagger 67 to 2, Nodachi 39 to 6. The total is 1,376 to 202.
- **Left:**
  - The Settler's balloon float (both hands up on the strings, 52 frames): the short arms cannot lift the hands clear of the big head.
  - The Greatsword's two-handed grip (its own hand depth code).
  - A forearm can still cross the face while the hand is clear (the Gauntlets' up air).
- **Image:** `docs/screenshots/anim/r2_pass32_face_guard.jpg`.
- **Checked:** the six tests pass.

### Pass 33: heavy fighters land heavy (real-speed audit: weight)

- **Defect** (the owner's note: "things that should be heavy moving as fast and light as the quick family"): every class shared one
  body spring. The Clutch and Greatsword landed, squashed and bounced back exactly as the Needles did.
- **Fix (`fighter_view.gd` `CLASS_WEIGHT`):**
  - The heavy classes (Greatsword, Bracers, Clutch) get springs 0.62 as stiff and damped toward critical, with a landing squash 1.25
    times as deep: they sink further and settle slowly, without a bounce.
  - The quick classes (Needles, Dagger) get 1.3 times as stiff with 0.8 the squash, so they snap back light.
  - Presentation only; the sim's timing is untouched.
- **Image:** `docs/screenshots/anim/r2_pass33_heavy_weight.png`: the squash through a landing, Longsword against Clutch.
  `tests/freeze_probe.gd -- --class=N --nofreeze` prints it.
- **Checked:** the six tests pass.
- **Left:** the heavy attacks' own timing (the coil and hold shares) is shared with every class.

### Round 3 (2026-10-26, from the art audit of that day, `docs/art/ART_AUDIT_2026-10-26.md`)

### Pass 34: timing against the reference: the hit held, a quick recovery, no lurch (round 3 item 1)

- **How it was measured:**
  - **Reference:** the hitbox animations of the reference swordsman and the reference claws fighter on the frame-data site, decoded frame
    by frame in the browser (`ImageDecoder`). Each frame's silhouette is cut from the plain background (hitbox colours left out), then:
    - its change from the frame before gives a speed curve;
    - its difference from the last frame gives a "distance from home" curve.
  - **Ours:** the same two curves from the joints and the blade's tip, printed every frame by a new debugging aid, `PF_MOTION_TRACE=1`
    (`fighter_view.gd` `_log_motion`), over the gallery demo. The demo has new `--gallery-air=Y`, `--gallery-frames=N` and
    `--gallery-moves=...` flags so that aerials end in the air.
  - **The report:** `tools/art/review/timing_report.py` reads both the same way. For each move it gives:
    - FT: the frames the swing keeps moving after the last active frame;
    - hold: the frames until the pose has gone a quarter of the way home;
    - rec: the frames from there until it is within 15% of home;
    - lurch: the speed of the move's first frame against its peak.
  - Our frame data matches the references' 1:1, so frame numbers compare directly.
- **Defects found:**
  - **The end lag drifted home at one slow ease across its whole length.** The strike was held for only about a third as long as the
    reference holds it, and the recovery dragged on into a long settle (4-11 frames of slow creep at the end). This is the "weightless"
    read. Examples: the Longsword's forward tilt stood back up by frame 20, where the reference is still deep in its lunge at frame 27;
    the Claws' forward smash stood up by frame 28.
  - **Moves lurched out of the idle.** The wind-up eased out, so a move's first frame was the fastest frame of its wind-up: on average
    0.57 of the strike's own peak speed. The reference starts gently (0.16-0.5).
  - **The lunge sprang back two frames after the hit.** The body's lunge spring let go at the last active frame plus two. So did the
    blade's 38-degree follow-through, which drifted over 28% of the end lag.
- **Fix (`fighter_view.gd`, shared by every class):**
  - **One end-lag shape** for the body clip (`strike_progress`), the blade (`_blade_target`), the reaching arm or leg (`_reach_weight`)
    and the lunge:
    - the swing reaches its follow-through within `FOLLOW_FRAMES` (3) and stops there (`follow_k`, an ease out);
    - the pose is held, creeping (`HOLD_CREEP` 0.15, the body clip 0.05), to `HOLD_TO` (48%) of the end lag;
    - it recovers by `RECOVER_TO` (92%) and settles by the end (`home_k`).
  - **The hips lead the recovery:** the weapon arm and the reaching limbs come home `ARM_LAG` (8% of the end lag) after the body.
  - **The blade** carries 30 degrees past its last hit, the reference's 25-30, in 3 frames. It was 38 degrees over 28% of the end lag.
  - **The lunge** is held through the hold and drawn back with the recovery.
  - **The wind-up** eases in and out into its coil (`coil_ease`, for the body, the blade and the fist's chamber), so the first frame is
    gentle.
  - **Per-move shapes** where both references agree:
    - the up tilt and back air hold briefly and recover slowly (`MOVE_END_SHAPES`);
    - the sword's jab and down tilt carry their cut on for 6 frames (`LONG_FOLLOW`);
    - the forward headbutt clip (Clutch) recovers early (`CLIP_END_SHAPES`). Its strike pose is head down with the face behind the hands,
      and holding it longer covered the face for 13 frames.
- **Measured** (frames; reference, before, after; FT, hold, rec):

  | Move | Longsword: reference | before | after | Claws: reference | before | after |
  |---|---|---|---|---|---|---|
  | Jab | 7 / 5 / 6 | 0 / 8 / 6 | 0 / 11 / 4 | 1 / 9 / 5 | 0 / 5 / 5 | 0 / 9 / 4 |
  | Forward tilt | 1 / 16 / 4 | 0 / 9 / 7 | 0 / 12 / 5 | 1 / 11 / 9 | 0 / 5 / 11 | 0 / 12 / 7 |
  | Up tilt | 1 / 8 / 10 | 0 / 11 / 5 | 0 / 11 / 5 | 1 / 7 / 13 | 0 / 5 / 10 | 0 / 8 / 9 |
  | Down tilt | 6 / 3 / 5 | 0 / 5 / 5 | 0 / 7 / 4 | 1 / 12 / 6 | 0 / 5 / 9 | 0 / 10 / 6 |
  | Forward smash | 1 / 16 / 17 | 0 / 18 / 12 | 0 / 25 / 7 | 0 / 9 / 6 | 0 / 4 / 6 | 0 / 8 / 6 |
  | Up smash | 1 / 22 / 13 | 0 / 7 / 22 | 0 / 21 / 11 | 1 / 11 / 5 | 0 / 5 / 12 | 0 / 13 / 6 |
  | Down smash | 1 / 26 / 4 | 0 / 16 / 10 | 0 / 18 / 9 | 0 / 9 / 8 | 0 / 11 / 5 | 0 / 11 / 6 |
  | Neutral air | 1 / 20 / 6 | 0 / 3 / 11 | 0 / 14 / 7 | 1 / 4 / 9 | 0 / 5 / 6 | 0 / 8 / 4 |
  | Forward air | 2 / 20 / 6 | 0 / 14 / 8 | 0 / 17 / 6 | 1 / 24 / 5 | 0 / 9 / 10 | 0 / 17 / 6 |
  | Back air | 1 / 12 / 14 | 0 / 6 / 15 | 0 / 10 / 12 | 1 / 13 / 14 | 0 / 11 / 12 | 0 / 16 / 9 |
  | Up air | 5 / 11 / 19 | 0 / 21 / 8 | 0 / 24 / 6 | 1 / 18 / 9 | 0 / 10 / 11 | 0 / 16 / 8 |
  | Down air | 4 / 33 / 9 | 0 / 27 / 11 | 0 / 32 / 8 | 1 / 19 / 15 | 0 / 4 / 20 | 2 / 21 / 6 |

  - The mean error against the reference fell from 6.7 to 3.6 frames for the hold and from 3.5 to 3.1 for the recovery.
  - The lurch fell from 0.57 to 0.29 of the peak speed.
  - Facing left measures the same (3.7 and 3.0).
  - What is left differs move by move, and the two references disagree on it (the neutral and up airs).
- **Image:** `docs/screenshots/anim/r3_pass34_timing.jpg`, every 2-3 frames at 60 fps, before above and after below:
  - the Longsword's forward tilt and forward air;
  - the Claws' forward smash;
  - the pose-distance curves of three Longsword moves, ours before and after against the reference.
- **Checked:**
  - both facings;
  - one class per family at real speed (Greatsword, Needles, Settler, Clutch: forward smash held, then recovering);
  - face cover (`PF_FACE_COVER`) per class, before then after: Longsword 10 to 13, Claws 0 to 0, Needles 15 to 15, Embers 5 to 3,
    Greatsword 88 to 92, Clutch 6 to 1, Settler 59 to 59;
  - the six tests pass.
- **Left:**
  - The reference's jab keeps its blade moving for 7 frames after the hit, and ours stops at once (FT 0). Our swing stops harder than the
    reference everywhere (FT 0 against 1).
  - In the held forward tilt the Longsword's hilt sits at the chin for about 10 frames.

### Pass 35: a run per family with planted feet, and the walks re-fitted to the side-on turn (round 3 item 2)

- **Defects:**
  - Every run's planted foot slid 0.4-0.8 of the ground speed (`tests/motion_probe.gd`; the heavy run 0.35).
  - The sword and brawler families shared one run.
  - **Found on the way:** the walks slid again, 0.19-0.27 of their speed (0.06-0.16 in pass 20). Their sweep was fitted to the old
    three-quarter turn, and since pass 28's side-on turn more of each step shows as ground.
- **Cause:**
  - The runs were keyed as leg angles (`RUN_THIGH` and friends), so where a foot went came out of the angles.
  - The game played them at a pace set by each fighter's dash length, not by the ground covered.
  - `WALK_TURN_SHARE` was 0.80, fitted to the 60-degree turn.
- **Fix:**
  - **`make_rigged_blob.py` `RUN_STYLES` and `run_style`:** each foot's path is authored, as the walks' are (`foot_to`).
    - While a foot is down (`down`, 24-30% of the cycle) it moves straight back at the ground's pace.
    - Then the heel kicks up behind, the knee drives through, and the foot reaches a little past its landing and paws back onto it, so it
      lands already moving with the ground.
    - A flight separates the steps.
  - **The pace:** a cycle covers `frames` x `RUN_TEMPO` (0.17) units of ground, and `fighter_view.gd` plays the base body's runs at
    `speed / RUN_TEMPO`. The dash keeps its own pace.
  - **The leg's range:** the runs are freed from the lunge clamp on the thigh, which had stopped the knee drive.
  - **Six runs:**
    - the sword family: the shared `run`, leaning in, the blade carried low behind, the far fist pumping;
    - the brawlers (new, `run_brawler`): hunched, both fists up and pumping;
    - quick: very low, the arms swept back;
    - heavy: upright, with a longer contact, a big drop on each step and wide arm swings;
    - caster: neat, the arms held in;
    - tool: a loose, bouncy jog.
  - `AnimLibrary.FAMILY_SHARES` is now empty.
  - `WALK_TURN_SHARE` is 0.96.
- **Measured** (`motion_probe`, slide as a share of the ground speed at 0.19 / 0.24 / 0.30):

  | Family (class) | Run before | Run after | Walk before (0.08 / 0.14 / 0.19) | Walk after |
  |---|---|---|---|---|
  | Sword (0) | 0.83 / 0.77 / n/a | 0.08 / 0.10 / 0.11 | 0.22 / 0.20 / 0.20 | 0.07 / 0.05 / 0.04 |
  | Brawler (1) | 0.83 / 0.77 / n/a (the shared run) | 0.09 / 0.11 / 0.11 | 0.24 / 0.20 / 0.20 | 0.06 / 0.05 / 0.03 |
  | Quick (2) | 0.99 / n/a / n/a | 0.06 / 0.06 / 0.09 | 0.22 / 0.19 / 0.17 | 0.08 / 0.06 / 0.05 |
  | Heavy (8) | 0.34 / 0.37 / n/a | 0.15 / 0.11 / 0.16 | 0.27 / 0.21 / 0.20 | 0.10 / 0.06 / 0.05 |
  | Caster (3) | 0.77 / 0.73 / n/a | 0.10 / 0.11 / 0.12 | 0.21 / 0.20 / 0.20 | 0.09 / 0.09 / 0.07 |
  | Tool (10) | n/a / 0.61 / n/a | 0.09 / 0.11 / 0.11 | 0.22 / 0.20 / 0.19 | 0.08 / 0.06 / 0.05 |

  "n/a" means the old run's foot was never down two frames running at that speed, so the probe found no planted frame.
- **Image:** `docs/screenshots/anim/r3_pass35_family_runs.jpg`: half a cycle of each family's run, before left and after right.
- **Checked:**
  - facing left (contact sheet);
  - `face_cover_probe`: every run and walk 0%;
  - the six tests pass. `fighter_view_test` now pins the brawler's run and its pace.
- **Left:**
  - The foot cannot fold quite as high as the quick run's heel kick asks: it misses by up to 0.05 at the top of the swing. This does not
    show as slide.
  - The old angle-keyed `run` is kept for the blob rigs only.

### Pass 36: the hurt reaction recoils off the hit, then recovers, by the hit's strength (round 3 item 3)

- **Defect:** a hit too weak to tumble leaned the whole body back 26 degrees for the entire hitstun, on top of the `hurt` clip's
  thrown-back chest. At match distance it read as one pose with the face to the sky for up to 30 frames. Light or strong, and from either
  side, it was the same.
- **Cause:** `fighter_view.gd` `_apply_combat` set `model.rotation.x` to -26 degrees whenever the `hurt` clip played. Through the hit's
  freeze it was upright, so the lean also popped in on the first frame after the hitlag.
- **Fix (`fighter_view.gd`, `hurt_recoil`, from the snapshot only):**
  - **The snap:** the body snaps away from the hit over 3 frames, back off a blow from the front and forward off one from behind (by the
    knockback's side against the facing). Through the freeze it is already 55% of the way.
  - **The hold:** it holds for 2-7 frames.
  - **The recovery:** it comes upright by 70% of the hitstun, leaving the clip's loose reel.
  - **By strength:** the hit's total hitstun (8 to 32 frames) sets the recoil, from 12 to 26 degrees, and the hold.
  - A freeze holds the recoil and a seek redraws it.
- **Tools:** `anim_sheet.gd --what=hurt` now gives the hitstun and knockback (`--hitstun=N`, 28 by default).
- **Tried and dropped:** tipping the head against the recoil. Side-on, the bend turned the face away rather than down.
- **Image:** `docs/screenshots/anim/r3_pass36_hurt_recoil.jpg`: hurt contact sheets for a 28-frame and a 12-frame stun, before left and
  after right, then facing left.
- **Checked:**
  - both facings;
  - the six tests pass;
  - `fighter_view_test` checks the recoil's direction, its strength and that it is upright before the stun ends.
- **Left:**
  - The `hurt_heavy` and `hurt_up` reactions (faster hits that do not tumble) keep their own clips.
  - For the first few frames of a strong recoil the face still looks up.

### Pass 37: the Greatsword's grip kept off the face (round 3 item 4)

- **Defect:** the Greatsword's two-handed grip passed in front of the face through its up tilt, up air, up special and forward air.
  `PF_FACE_COVER` counted 92 frames over its 16 moves, against 81 after pass 32 (pass 34's longer holds added some).
- **Cause (`fighter_view.gd` `_pose_blade`):**
  - its grip had its own depth (`great_z`, in front at 0.35 whenever it swings), so the head-clear rule that sends every other class's
    raised hand behind the head (pass 24) skipped it;
  - `great_z` also moved on real time, not the sim frame.
- **Fix:**
  - The same rule now applies to the greatsword (`_behind_head_k`, shared). A grip that comes up past the face goes behind the head in
    depth, and the broad blade still shows round and above the head. Its rest pose is covered too.
  - `great_z` moves by sim frames (`GREAT_Z_STEP`), so a freeze holds it and a seek puts it straight there.
- **Measured:** 92 to 13 frames (14 facing left). What is left is the hilt at the chin in the forward air, with the face showing
  (checked in the capture).
- **Image:** `docs/screenshots/anim/r3_pass37_greatsword_grip.jpg`: the up tilt and up air, before above and after below.

### Pass 38: the Settler's balloon float, re-posed (round 3 item 4; the owner's pick: a re-posed hold)

- **Defect:** for the whole float (up special, 52 frames) the near fist sat in front of the face.
- **Cause:**
  - The move has no hit, so its progress (`strike_progress`) ran straight on to the clip's rest key: the tool stance, whose guard holds
    the near fist at the chin.
  - The clip's own pose put both hands up beside the head, and side-on the near one is in front of the face in depth.
- **Fix:**
  - **`make_rigged_blob.py` `tool_float`:** the far hand holds the strings high over the head, a little behind it, and the near hand
    grips them lower, in front of the chest.
  - **`fighter_view.gd` `HELD_STRIKE`:** a move held in its strike pose for as long as it lasts. It rises into the pose over 8 frames,
    and the legs paddle.
- **Measured:** 52 frames to 0. The Settler's total is 59 to 6 (5 facing left).
- **Image:** `docs/screenshots/anim/r3_pass38_settler_float.jpg`.
- **Left:** there is still no balloon prop, so the strings are implied.

### Pass 39: the Gauntlets' overhead kick rises past the face (round 3 item 4)

- **Defect:** in the up air, the kicking leg went straight up across the face for 12 frames. The face-cover log counted only hands and
  the hilt, so pass 32 had seen it as a forearm and could not count it.
- **Cause:** `fighter_view.gd` `_aim_leg` aimed the near leg at the hitbox straight overhead, slightly toward the camera.
- **Tried first:** sending it behind the head in depth, as raised hands go. The short leg cannot reach above the head, so the kick
  vanished.
- **Fix:** a kick whose foot is above the chest and not far out in front keeps its foot `KICK_CLEAR_X` (0.8) ahead of the hips. It reads
  as a high kick rising past the face.
- **The log:** `PF_FACE_COVER` now counts the feet and shins too.
- **Measured** (Gauntlets): legs over the face 24 frames to 3 (the up smash's last frames). The other classes' legs over the face now
  show up in the log; see pass 40.
- **Image:** `docs/screenshots/anim/r3_pass39_overhead_kick.jpg`.
- **Checked (passes 37-39):**
  - facing left;
  - the six tests pass.

### Pass 40: a grounded blade's tip stays out of the floor (round 3 item 5, the real-speed audit)

- **Defect:** in the Greatsword's forward smash, from frame 28 to the recovery, the blade's last third was sunk in the stage through the
  held follow-through. The tip measured 0.9 below the floor.
- **Cause:**
  - The floor check (`_pose_blade`, `GROUND_CLEARANCE`) works in the blade's own frame, which leans and squashes with the body. The body's
    lean into the cut tipped that frame forward, so the tip went under the real floor while it measured clear.
  - The hilt is also moved afterwards to where the hand really got to, which is lower when the arm cannot reach.
  - Pass 34's longer hold, with its 30-degree follow-through, made it show for longer.
- **Fix (`fighter_view.gd` `_lift_tip_off_floor`):** after the hilt is placed in the hand, on the ground, the tip is measured in the world.
  If it is below the floor, the blade turns up about the hilt by the least angle that clears it, found by bisection, keeping its length.
  This applies to every blade and the staff.
- **Measured:** the Greatsword's forward smash tip went from -0.94 at its lowest to +0.06. No grounded Longsword move puts its tip below
  the floor.
- **Image:** `docs/screenshots/anim/r3_pass40_blade_off_floor.jpg`.
- **Checked:** the six tests pass.
- **Also found in this audit, not fixed** (pre-existing, the same before round 3):
  - **The Embers' forward air** shows the top of the head instead of the face for about 15 frames of its follow-through. Turning the head
    back toward the foe was tried and did not change what shows; it needs a look at the clip's curl for the unarmed classes.
  - **When grabbed,** a fighter's far hand crosses its face on the pummel hits (in a CPU match, about 4 frames a pummel).
  - **Legs over the face** (now counted, pass 39): the Needles' up air and up tilt (8 frames) and the Clutch's forward air and forward
    smash (11).
  - **The Longsword's held forward tilt** keeps the hilt at the chin for about 10 frames.
  - **The sword family's jump crouch** put the blade across the mouth (`face_cover_probe`: 100% of its 3 frames). It is fixed in pass 41.

### Pass 41: the sword guard sinks and dips with the jump crouch (round 3 item 5)

- **Defect:** on every jump the Longsword's resting blade crossed the mouth for the 3 frames of the crouch. `face_cover_probe` measured
  100% of the jump crouch. The rest guard is aimed at a fixed height and angle, so as the body sank the face came down onto it.
- **Fix (`fighter_view.gd` `_pose_blade`):**
  - out of a move, on the ground, the guard goes down by as much as the hips sink below the stance (`GUARD_HIPS` 0.64);
  - the blade dips `GUARD_DIP` (260) degrees a unit of that, about 40 degrees in a full crouch, so it lies level in front of the chest
    instead of rising across the face. It comes back up as the body rises.
- **Measured:** the jump crouch went from 100% to 0% in every family.
- **Image:** `docs/screenshots/anim/r3_pass41_jump_crouch_guard.jpg`.
- **Checked:** facing left (contact sheet); the landing; the six tests pass.

### Round 4 (2026-10-26, round 3's leftovers, approved by the owner)

### Pass 42: the swing carries on past its last hit instead of stopping dead (round 4 item 1)

- **Defect:** every swing stopped hard. After the last active frame a sword cut put 70% of its 30-degree follow-through into one frame and
  was still by the next; a raking claw stopped on the end of its arc; the lunge was held where the hit left it. Round 3 measured 0 frames of
  follow-through (FT) on every move against the reference's 1-7.
- **The tool, first:** `timing_report.py` read the trace's state frames (counted from 0) as the reference's move frames (counted from 1),
  so it measured each follow-through from one frame too late. It now adds one. It also skips the trace's non-point tokens (the pitch and
  the head's look, added in pass 40, had broken it), and prints a second, gentler measure, `carry`: the frames after the last hit still
  moving at 10% of the peak.
- **Fix (`fighter_view.gd`, shared):**
  - **The carry (`follow_k`):** past the last hit the swing keeps going at one pace for the move's carry frames, then slows to a stop over
    `CARRY_EASE` (2) more. It was a cubic ease out over 3 frames. The carry is per move: a sword's from the reference swordsman
    (`SWORD_CARRY`: the jab 4, the down tilt, up air and down air 3, the forward air 2, the rest 1) and everyone else's 1, the reference
    claws fighter's.
  - **The blade** carries on at `CARRY_DEG` (22) degrees a frame: about 33 degrees for a one-frame carry, the reference's 25-30, and up to
    110 for the jab's long cut (the floor stops it sooner). On the last frame of a hit that lasts more than one, it has already started
    (`_carry_lead`), so it no longer stops on the hit for a frame and starts again. Not for thrusts, and not for the neutral air, whose
    windmill already turns past the hit (carried further, the sword hand came over the face for 4 frames).
  - **A raking hand** (the claws' arcs) carries on round its arc at `LIMB_CARRY_DEG` (20) a frame (`_arc_point`'s `carry`), except an
    overhead hit, which would come down across the face.
  - **The lunge** goes on forward by `LUNGE_CARRY` (30%) through the carry, then settles into the hold.
  - The holds are round 3's (`HOLD_TO`, `RECOVER_TO`, `ARM_LAG`).
- **Tried and dropped:** carrying a kick on round the hip. The short leg folded across the chest and chin: 8-17 more frames of legs over
  the face in the Greatsword's down tilt, the Needles' down smash and the Clutch's jabs. Kicks carry with the lunge only.
- **Measured** (frames, `timing_report.py` with the fixed frame count; FT / hold / rec, then the 10% carry before and after):

  | Move | Longsword: reference | before | after (carry) | Claws: reference | before | after (carry) |
  |---|---|---|---|---|---|---|
  | Jab | 7 / 5 / 6 | 0 / 13 / 3 | 0 / 14 / 3 (0 to 0) | 1 / 9 / 5 | 0 / 10 / 4 | 0 / 10 / 4 (2 to 2) |
  | Forward tilt | 1 / 16 / 4 | 1 / 12 / 5 | 0 / 14 / 4 (2 to 2) | 1 / 11 / 9 | 0 / 13 / 7 | 2 / 11 / 7 (4 to 5) |
  | Up tilt | 1 / 8 / 10 | 1 / 11 / 5 | 0 / 12 / 5 (3 to 2) | 1 / 7 / 13 | 0 / 9 / 9 | 0 / 9 / 9 (0 to 0) |
  | Down tilt | 6 / 3 / 5 | 0 / 8 / 4 | 0 / 8 / 4 (1 to 2) | 1 / 12 / 6 | 0 / 12 / 5 | 0 / 12 / 6 (2 to 4) |
  | Forward smash | 1 / 16 / 17 | 0 / 26 / 7 | 0 / 27 / 6 (0 to 0) | 0 / 9 / 6 | 0 / 9 / 6 | 0 / 10 / 5 (2 to 3) |
  | Up smash | 1 / 22 / 13 | 0 / 22 / 12 | 0 / 22 / 12 (1 to 1) | 1 / 11 / 5 | 0 / 13 / 7 | 0 / 15 / 5 (0 to 0) |
  | Down smash | 1 / 26 / 4 | 0 / 16 / 12 | 0 / 17 / 11 (1 to 2) | 0 / 9 / 8 | 0 / 12 / 6 | 1 / 11 / 6 (6 to 6) |
  | Neutral air | 1 / 20 / 6 | 0 / 16 / 7 | 0 / 17 / 6 (1 to 0) | 1 / 4 / 9 | 0 / 9 / 4 | 0 / 10 / 3 (0 to 0) |
  | Forward air | 2 / 20 / 6 | 0 / 18 / 6 | 0 / 19 / 5 (2 to 3) | 1 / 24 / 5 | 0 / 18 / 6 | 0 / 18 / 6 (0 to 1) |
  | Back air | 1 / 12 / 14 | 1 / 10 / 12 | 1 / 10 / 12 (2 to 2) | 1 / 13 / 14 | 0 / 17 / 9 | 0 / 17 / 9 (6 to 6) |
  | Up air | 5 / 11 / 19 | 0 / 25 / 6 | 0 / 26 / 5 (7 to 7) | 1 / 18 / 9 | 0 / 17 / 8 | 0 / 18 / 7 (0 to 0) |
  | Down air | 4 / 33 / 9 | 1 / 32 / 8 | 3 / 32 / 7 (7 to 8) | 1 / 19 / 15 | 3 / 21 / 6 | 3 / 21 / 6 (5 to 6) |

  - The carry shows plainly frame by frame (the image) but moves FT little. FT counts frames at 30% of the move's peak speed, and our peak
    is one frame: the snap puts the whole wind-up-to-hit turn (about 100 degrees of blade) into the frame before the hit, so even a
    33-degree carry reads as slow beside it. The reference spreads its strike over the active frames. Closing that gap means letting the
    blade sweep through a still hitbox across its active frames rather than standing on it; that is a design question for the owner.
  - The holds and recoveries are unchanged within a frame.
- **Image:** `docs/screenshots/anim/r4_pass42_carry.jpg`, every frame at 60 fps: the Longsword's forward tilt (both facings) and forward
  air, the Claws' forward tilt, before above after.
- **Checked:**
  - both facings;
  - face cover over all 14 classes' moves, before then after: unchanged except one frame each in the Claws' down smash, the Dagger's up
    smash and back air and the Nodachi's neutral air (the carried hand or hilt passing the face's edge);
  - the six tests pass.
- **Left:** the Longsword's forward tilt now carries its blade past upright, so its hilt sits over the eyes through the hold: pass 43.

### Pass 43: the Longsword's held forward tilt keeps its hilt off the eyes (round 4 item 4)

- **Defect:** through the forward tilt's hold the hilt and its guard sat across the eyes (round 3 saw it at the chin for about 10 frames;
  pass 42's carry took the blade on past upright and the hand up with it). `PF_FACE_COVER` did not count it.
- **Cause:**
  - The cut ends overhead, and the short arm cannot hold the hand clear of the big head: with the blade upright the hand is at eye height,
    just ahead of the head's middle.
  - The rule that sends a raised hand behind the head (`_behind_head_k`, pass 24) measured from the head's middle. Side-on the face is
    drawn on the front of the head, so a hand over the eyes measured clear. The face-cover log measured the same way, so it missed it too.
- **Fix (`fighter_view.gd`):**
  - **Held leaning forward:** a grounded cut whose end lag would hold it steeper than `HELD_UP_MAX` (68 degrees above forward) eases to that
    angle through its carry, so the blade stands out ahead of the head rather than over it.
  - **Measured from the face:** through a sword's end lag the behind-the-head rule measures from the drawn face, `FACE_FWD` (0.3) ahead of
    the head's middle. The hand goes behind the head and the leaning blade shows beside it.
  - Neither applies to the Nodachi: its long blade, held forward, came home across the face (3 frames).
  - **The log:** `PF_FACE_COVER=front` measures from the face as well.
- **Measured** (`PF_FACE_COVER=front`, the Longsword's 16 moves): the forward tilt went from 17 frames to 1 and the up smash from 4 to 0.
  The class total went from 68 to 48. With the old measure, every bladed class is unchanged.
- **Image:** `docs/screenshots/anim/r4_pass43_ftilt_hilt.jpg`: the forward tilt every frame from its hit, before (pass 42) and after,
  both facings.
- **Checked:** both facings; the six tests pass.
- **Left (found by the new measure):** the Longsword's up special (18 frames), forward smash (7) and neutral air (5) still bring a hand or
  the hilt across the drawn face. They are not on the round 4 list; ranked in the round's audit.

### Tool fix (round 4 item 5): `anim_sheet.gd`'s launch-smoke errors

- **What:** every contact sheet logged `Cannot call method 'get_camera_3d' on a null value` from `fighter_view.gd` `_launch_smoke`, and 28
  `!is_inside_tree()` errors from `_skeleton_to_frame`, on its first frames.
- **Cause:** tool-only, and older than round 3 (the smoke's camera-facing puffs came with the painted effects, `19f8ec38`). The sheet
  applies the view inside `_initialize`, before the root viewport is in the tree, so the view had no viewport yet. The game only applies a
  view that is in the tree.
- **Fix (`tests/anim_sheet.gd` only):** it waits one frame after building the view. The sheet now logs no script errors (only the
  engine's usual leak notes at exit). No gameplay code changed.

### Pass 44: the Longsword's forward smash recovers slowly and its up air floats back sooner (round 4 item 2)

- **Defect** (measured against the reference swordsman, frames of hold / recovery): the forward smash held 27 and recovered in 6 against
  16 and 17, so after a long freeze it snapped home; the up air held 26 against 11 and recovered in 5 against 19.
- **Cause:**
  - Both used the shared end-lag shape (hold to 48% of the end lag, recover by 92%).
  - The forward smash's held cut also lay below the floor. The floor check (pass 40) pinned the tip still while the angle came home
    underneath, so its recovery showed only in its last few frames.
  - The up air carried its cut on for 3 frames (pass 42), taking the pose further from home after the hit, which lengthens the hold.
- **Fix (`fighter_view.gd`):**
  - **`CLASS_END_SHAPES`:** the Longsword's own shapes. The forward smash holds to 20% of its end lag and recovers slowly to 99%; the up
    air holds to 6% and floats back to 99%.
  - **The held cut stops at the floor:** on the ground a held cut is kept at the angle where the floor stops it, with the hand as low as a
    lunge takes it (`HELD_HAND_Y` 0.6). The recovery then rises from where the blade really is.
  - The up air carries 2 frames, not 3.
- **Measured** (the reference, then before and after, FT / hold / rec):
  - forward smash: 1 / 16 / 17; 0 / 27 / 6; 0 / 18 / 13;
  - up air: 5 / 11 / 19; 0 / 26 / 5; 0 / 20 / 9.
  - The up air's hold is still long: after its hit the whole pose keeps moving away from home for about 10 frames as the air arc
    blends, and the report counts that as hold.
- **The table re-measured for both classes** (FT / hold / rec; the reference first):

  | Move | Longsword: reference | now | Claws: reference | now |
  |---|---|---|---|---|
  | Jab | 7 / 5 / 6 | 0 / 13 / 3 | 1 / 9 / 5 | 0 / 10 / 4 |
  | Forward tilt | 1 / 16 / 4 | 0 / 13 / 5 | 1 / 11 / 9 | 2 / 11 / 7 |
  | Up tilt | 1 / 8 / 10 | 1 / 11 / 6 | 1 / 7 / 13 | 0 / 9 / 9 |
  | Down tilt | 6 / 3 / 5 | 0 / 8 / 4 | 1 / 12 / 6 | 0 / 12 / 6 |
  | Forward smash | 1 / 16 / 17 | 0 / 18 / 13 | 0 / 9 / 6 | 0 / 10 / 5 |
  | Up smash | 1 / 22 / 13 | 1 / 21 / 12 | 1 / 11 / 5 | 0 / 15 / 5 |
  | Down smash | 1 / 26 / 4 | 0 / 17 / 11 | 0 / 9 / 8 | 1 / 11 / 6 |
  | Neutral air | 1 / 20 / 6 | 0 / 17 / 6 | 1 / 4 / 9 | 0 / 10 / 3 |
  | Forward air | 2 / 20 / 6 | 0 / 19 / 5 | 1 / 24 / 5 | 0 / 18 / 6 |
  | Back air | 1 / 12 / 14 | 1 / 10 / 12 | 1 / 13 / 14 | 0 / 17 / 9 |
  | Up air | 5 / 11 / 19 | 0 / 20 / 9 | 1 / 18 / 9 | 0 / 18 / 7 |
  | Down air | 4 / 33 / 9 | 3 / 32 / 7 | 1 / 19 / 15 | 3 / 21 / 6 |

  - Mean error against the reference over both classes (frames): FT 1.6 to 1.5, hold 3.8 to 3.1, recovery 3.3 to 2.9. (Round 3's 3.6 and
    3.1 were counted one frame early; the same capture counted the new way gave 3.8 and 3.3.)
- **Image:** `docs/screenshots/anim/r4_pass44_longsword_timing.jpg`: the forward smash every 4 frames from its last hit (both facings) and
  the up air every 3, before above after.
- **Checked:**
  - both facings;
  - the Greatsword's forward smash keeps its tip on the floor as in pass 40;
  - face cover unchanged (both measures);
  - the six tests pass.

### Pass 45: the heavy family's run carries its weight (round 4 item 3)

- **Defect:** the heavy family's run (the Clutch, `AnimLibrary.CLASS_FAMILY`) read like the others: the same height, a quick cadence
  (19 frames a cycle, the shortest of the six) and a small bob.
- **Fix:**
  - **`make_rigged_blob.py` `RUN_STYLES["heavy"]`:**
    - the hips lower (0.68 to 0.61) and the strides longer at a slower cadence (22 frames a cycle; each covers 3.7 units of ground, not 3.2);
    - a longer, heavier contact (`down` 0.32) with a bigger drop (`bob` 0.12) that lands after the foot is planted (`sink`, the drop 3%
      of the cycle later);
    - the feet kept lower (`lift` 0.16, not 0.2), the body rocking side to side (`sway` 9, `twist` 16) and the arms swinging wider.
    - Longer strides were tried (23-25 frames, hips 0.55): the short legs could not reach the landing (the foot missed by up to 0.25), so
      the stride stops where the leg still reaches (0.06 at most, mid-swing).
  - **`fighter_view.gd` `STEP_SQUASH`:** each footfall's squash scales with the body's weight (`CLASS_WEIGHT`, squared): a heavy body
    squashes 1.56 times as much (0.11), a light one (Needles, Dagger) 0.64 times. The heavy weight is the Greatsword's, the Bracers' and
    the Clutch's, so the first two (which run the sword and brawler runs) land heavier too.
- **Measured** (`motion_probe`, slide as a share of the ground speed at 0.19 / 0.24 / 0.30): heavy 0.15 / 0.11 / 0.16 before, 0.14 / 0.14 /
  0.14 after. The sword's and the quick runs are unchanged (0.08 / 0.10 / 0.11 and 0.07 / 0.06 / 0.08). `face_cover_probe`: every run 0%.
- **Image:** `docs/screenshots/anim/r4_pass45_heavy_run.jpg`: the heavy run's contact sheet, before and after, both facings, and the sword
  run below for contrast.
- **Checked:** both facings; the six tests pass.
