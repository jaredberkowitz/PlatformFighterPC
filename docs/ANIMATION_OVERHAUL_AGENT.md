# Animation overhaul agent: plan and brief

Give this file to a new agent to start the full animation pass. It follows the same workflow as the character pass
(`docs/art/characters/CHARACTER_ANALYSIS.md`), the menus (`docs/UI_OVERHAUL_AGENT.md`) and the stages (`docs/STAGE_REFINEMENT_AGENT.md`):
- references first, then a measured audit;
- one visible defect at a time, each fix shown before and after from the same camera and frame.

**Why now:** the art audit of 2026-10-24 (`docs/art/ART_AUDIT_2026-10-24.md`) puts the stages, the UI and the model at 4 to 4.5 out of 5,
but animation at 2.5. The still image is well ahead of the motion, and playtesters see the fighters in motion.

The animation is not starting from zero. Read what exists and **pick up from there; do not restart it:**
- the architecture and earlier passes, in `docs/ANIMATION.md`;
- the pose review, in `docs/POSE_REVIEW.md`;
- the motion layers, in `docs/MOTION_POLISH_PLAN.md`;
- the "Animation overhaul" checklist in `docs/SATURDAY_BACKLOG.md`.

## Role

You are a senior gameplay animator and Godot 4 technical animator. Your one job is to take the fighters' animation from a working
prototype to polished, readable, weighty fighting-game motion. This is an original 3D-on-2D platform fighter: never use "Smash" or any
Nintendo names or assets in the repository (the owner's references may be named in conversation, never in files).

**The bar** is the reference platform fighter's motion read through our own round, cartoony body:
- compact chamber, then full extension;
- the whole body lines up behind the strike;
- the strike is held for a moment;
- the follow-through carries on (aerials roll and curl);
- fists closed, wrists straight, human elbows and knees.

Exaggerated but physically sensible: a support leg is planted or tucked, and the arms counterbalance.

## How animation works here (read before changing anything)

- **The sim owns everything that matters to play.** Frame data, hitboxes, movement and timing are Rust, and they are deterministic for
  rollback. Animation is presentation only:
  - never change startup, active or recovery frames for looks;
  - never add root motion;
  - never move a hitbox.
  The view reads the move's timing and live hitboxes from the snapshot and poses the body to match.
- **Clips** are authored in Blender by script. `art/blender/make_rigged_blob.py`:
  - `BODY_MOVES` and `move()` hold the keys per clip (wind, strike, rest, at fixed frames);
  - the base body comes from `make_cage_body.py`, then `fit_body.py --shoulder-lift 0.11`, then
    `make_rigged_blob.py --body ... --joints ... --neck-drop 0`, then `godot --headless --path godot --import`.
- **Mapping:** `godot/scripts/anim_library.gd`:
  - `CLASS_MOVES` maps each class's move to a clip;
  - `CLASS_FAMILY` and `FAMILY_CLIPS` give per-family idle and run;
  - `OWN_ARMS` marks clips that pose both arms themselves.
- **Code layers on top of the clips**, in `godot/scripts/fighter_view.gd` (about 4,800 lines, so read the part you change):
  - weapon aim (`_blade_target`, `strike_progress`, `_reach_weight`, `_thrust`);
  - the arm and leg IK to the live hitbox;
  - body follow and turn (`_body_follow`, `attack_turn`, `_face_camera`);
  - the air counter-pitch (`_air_counter`);
  - springs, smears and deformation (`motion_springs.gd`, `smear.gd`, `motion_deform.gd`);
  - the swing arcs (`ARCS`, which follow the hitboxes until this pass makes the blade match them).
- **Facing left** plays mirrored `@m` clips. Attack clips pose the free arm on the FAR arm, and the near arm is aimed by code. Check both
  facings for every change.
- **Exaggeration:** clip angles are amplified (`AMP`, then `BODY_AMP` 0.85), and the shoulder and elbow are clamped after it (172 and 145
  degrees). Only clips named `attack_`, `sword_`, `kick_`, `gaunt_`, `tool_` or `body_` are amplified. Author big angles before the amp.
- **14 classes** (`roster.gd` `CLASS_NAMES`):
  - Longsword 0, Claws 1, Needles 2, Embers 3, Gauntlets 4, Staff 5, Greatsword 6, Bracers 7, Clutch 8, Buckler 9, Pockets 10,
    Popgun 11, Dagger 12 and Nodachi 13;
  - 17 built-in fighters use them;
  - each class's reference fighter is in `docs/MOVESETS.md` and `docs/movesets/*/`.

## Scope

**In scope:**
- every clip, its keys and timing curves;
- the class-to-clip mapping;
- the code layers above;
- per-family pose sets;
- secondary motion (springs, squash, overlap);
- the swing arcs once the blade matches the hitboxes;
- the poses of the weapons the fighters already hold.

**Out of scope** (recommend, never change):
- frame data, hitboxes and anything in the sim (that is the moveset session's; changing it needs a `SIM_VERSION` from the
  "Moveset porting agent" session);
- the body model's shape (the character pass owns it; if a pose cannot work on the body, report it with a capture);
- stages, the UI and the effects' look;
- new weapons or props. The Dagger and Nodachi still have none: ask the owner before modelling any.

**Hard rules:**
- `CLAUDE.md` applies in full.
- Visuals never touch the sim; the content hash must not change.
- Hitlag freezes a pose, and a replay seek or a rollback must redraw it exactly. `tests/motion_test.gd` checks this for every layer.
- Ask the owner before using Cascadeur, Mixamo or the Blender MCP's generate and asset tools. The live Blender connection
  (`localhost:9876`) is fine for posing and renders.
- Refine, don't add: improve existing clips before inventing new systems.

## Phase 0: set up

1. **Branch:**
   - Work in your own worktree and branch (for example `art/animation`), never in another session's.
   - Branch from `origin/main`: since 2026-10-25 it holds the latest body, faces, effects, stages and UI (the art branches were merged
     there before this pass).
   - In a fresh worktree, run `--import` twice; the first run can segfault and half-write fonts.
2. **Read**, in this order:
   - `docs/ANIMATION.md`;
   - `docs/POSE_REVIEW.md`;
   - the Animation overhaul section of `docs/SATURDAY_BACKLOG.md`;
   - `docs/MOTION_POLISH_PLAN.md`;
   - `docs/BODY_PROPORTIONS.md`;
   - `docs/art/ART_AUDIT_2026-10-24.md` (animation rows).
3. **References.** These are the owner's standing references:
   - the hitbox animations on ultimateframedata.com (`hitboxes/<fighter>/<Move>.gif`; list them by regex on the page's HTML);
   - the owner's gifs: the reference plumber's forward air from the previous game (the gold standard), the wolf's forward and neutral
     airs, the cloud swordsman's forward air, the ninja's forward tilt, the prince swordsman's forward air and the princess swordswoman's back
     air;
   - the owner's combo video, which shows real in-game motion with a grid: https://www.youtube.com/watch?v=H8fBhZuywgo.

   How to read them:
   - decode gifs frame by frame in the built-in browser with WebCodecs `ImageDecoder`, awaiting `tracks.ready` and `completed`
     (otherwise frame counts are wrong), and tile them on a canvas;
   - for video, seek the page's `<video>` and draw frames to a canvas.

   Never download the footage into the repository, and never feed it to a generator.
4. **Baseline captures** (below) of every class before any change.

## Capture tools (need a window, so not `--headless`)

Godot is `tools/godot/Godot_v4.7.1-stable_win64_console.exe`; run it from the repository root with `--path godot`.

| What | Command |
|---|---|
| Pose atlas, one class: each move at wind-up, strike and follow-through from its real frame data | `res://main.tscn -- --demo=poses --gallery-class=N --noui --shots=<dir>` |
| Every normal and special of a class, every 3 frames | `-- --demo=gallery --gallery-class=N --noui --noecb --shots=<dir>` |
| Contact sheet of one clip or move | `--script res://tests/anim_sheet.gd -- --what=<run, idle, jump, hurt, launch, fsmash, nair...> --class=N --out=<png>` (`--facing=-1`, `--zoom=`) |
| One move on an opponent, every frame | `-- --demo=fx_<sword, claws, needles, embers, gauntlets, staff>_<move> --shots=<dir>` (`_whiff` to show the arc and hitboxes; hitboxes show unless `--noui`) |
| Hops, landings, dash dance, walk | `--demo=hop`, `landing`, `dashdance`, `walk`, `dashleft` (the facing-left regression) |
| Combine and compare images | `tools/art/review/` `crop.gd`, `grid.gd`, `stack.gd`, `diff.gd`, `grey.gd` (`--headless -s <script> -- ...`) |
| Grey renders of a posed clip in Blender, no cel shading | the method is in `ANIMATION.md`; judge deformation there, because the cel look hides and fakes problems |

## Phase 1: audit (inspect only, edit nothing)

Return to the owner:
1. **The baseline:** the pose atlas of all 14 classes, plus locomotion and reaction sheets of the sword and one unarmed family.
2. **A side-by-side table per reference move:** ours against the reference at the same moments (wind-up, strike, follow-through).
   Measure where you can:
   - frames of visible anticipation before the first active frame;
   - how far the strike extends (limb or blade reach compared with the hitbox);
   - how much the hips and chest turn;
   - how long the strike is held;
   - how far the follow-through travels.
3. **The most visible weaknesses, ranked**, re-checked against the pose review's list rather than taken as given. The audit of 10-24
   saw:
   - an upright, stiff idle with locked knees instead of a low battle stance with weight shifts and breathing;
   - one held pose each for the walk and the jump (no tuck at the apex, no legs opening on the way down);
   - attacks with almost no anticipation, the hips never leading, and strikes ending on a straight locked arm;
   - the sword's neutral air reading as a downward poke, not a spin;
   - little weight or squash anywhere;
   - nine unarmed classes sharing nearly one pose set, and the Dagger and Nodachi holding no weapon;
   - the swing arc floating off the blade, because the blade does not reach the hitbox.

   For each weakness give the evidence, the cause (clip, mapping or code layer: file and function), the fix and its risk.
4. **A proposed order** (below), adjusted by what the audit finds.

Wait for the owner's approval before changing anything significant.

## Phase 2: refine, one visible defect at a time

**Proposed order:**
1. **Stance and locomotion per family:**
   - families: sword, brawler, heavy, quick, caster, tool;
   - clips: idle (low, grounded, breathing, weight shift), walk, run, dash, jump (crouch, rise, tuck, open, fall), land with squash,
     shield, dodges;
   - these are on screen every second of a match, so they come first.
2. **Attacks, class by class**, the Longsword first, against its reference:
   - each signature move gets its own six-pose keys: rest, anticipation, launch, impact (held), follow-through, settle (the owner's
     notes in `ANIMATION.md` and the swing notes);
   - the hips lead the chest, and the chest leads the arm;
   - the off hand aims or guards, never flung out;
   - elbows never lock.

   Then the brawler family and each remaining class.
3. **Blade and arc agreement:** once a weapon's swing reaches its hitboxes, switch its `ARCS` back from following the hitboxes to
   following the blade tip.
4. **Reactions:** hurt by strength and direction, launch, tumble, ledge hang and climb, get-up, shield break, KO.
5. **Secondary motion and polish:** hats and hair overlap, squash and stretch on every jump and landing, idle fidgets.

**For each item:**
1. Capture the problem in a fixed setup: the same demo frame, class and facing.
2. Find the cause. Read the clip, mapping and code layer first; half of "pose" problems here were code layers, such as an arm hinge or
   body follow fighting the clip.
3. Fix it in the right layer: the keys in the clip, which clip in the mapping, timing and whole-body moves in the code.
4. Render after in the identical setup, and compose a before/after into `docs/screenshots/anim/<pass>.png` at match distance and close
   up.
5. Check both facings, the moves that share the clip or code layer, hitlag freezing, and a replay seek.
6. Run the Godot tests: `anim_library_test`, `fighter_view_test` (it pins some class mappings: update it with the change), `motion_test`,
   `match_flow_test` and `replay_test`, and `look_bench`, which needs a window.
7. Log it in `docs/ANIMATION.md` (a new numbered pass section): the defect, cause, fix, files, what was checked and what is left. Tick
   the item in the backlog.
8. Commit, naming the before/after image.

**Rules that worked:**
- Mock before changing a design decision. When there are two ways to read a move, render two to four options and let the owner pick.
- Judge from grey Blender renders as well as in game.
- Use a second pair of eyes: suggest a separate judge agent round every few passes, scoring against the references (as
  `docs/gauntlet/CHARACTER_JUDGE.md` does for the model).
- Revert what makes no visible difference at match speed and distance.
- Report honestly: say what was not checked (other classes, a controller, both facings).

## Coordination with other sessions

- **Builds:** the owner plays from `C:\Users\jared\OneDrive\Desktop\Platform Fighter Game`. Before installing a build:
  - fetch, and merge every art branch the folder's current build contains (`art/stage-refine`, `art/paper-fx`, `art/character-beta`)
    into your build branch;
  - read the first line of the folder's `How to play.txt` to see whose build is in there, so you never roll another session's work
    back;
  - put your notes block on top under "Both players need the same version of the game.", renaming the previous one to EARLIER BUILD.

  The full build steps are in the owner's notes (export the pack, keep the DLL and content, verify the content hash, test the zip).
- The stage session (`art/stage-refine`) and the effects branch (`art/paper-fx`) touch `fighter_view.gd` lightly. Merge often.

## Definition of done

- Every class has its own stance, locomotion and attack keys that read as combat at match speed.
- Each attack is checked against its reference move side by side.
- The blade or limb meets its hitbox, and the arcs follow the blade.
- Reactions read by strength and direction.
- The tests pass, or fail exactly as on the base branch (noted).
- No sim changes; the content hash is unchanged.
- Each pass has a before/after image and a log entry, with a build for the owner after each family or class.

## First task

Do Phase 0 and Phase 1 and report back:
- the baseline atlases;
- the reference side-by-side table for the Longsword and one unarmed class;
- the ranked weaknesses with causes and fixes;
- the proposed order.

Edit nothing until the owner approves.
