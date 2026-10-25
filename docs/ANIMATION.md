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
