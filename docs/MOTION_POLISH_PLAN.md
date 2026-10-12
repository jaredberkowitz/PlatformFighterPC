# Fighter motion polish plan

Hand-off plan for a build agent. Goal: make fighters read as weighty, fast and alive at match-camera distance, building on the system
already in `godot/scripts/fighter_view.gd` and `docs/ANIMATION.md`. Companion to `GRAPHICS_FIDELITY_PLAN.md` (stages and scenery); the two
do not overlap and can run in either order.

Everything here is **presentation only**: nothing touches `sim-*`, `netplay`, content files or the checksum, so no `SIM_VERSION` or
`SCHEMA_VERSION` bump. Animation never becomes a source of truth for timing, hitboxes or hurtboxes.

## What already exists (do not rebuild)

Read `docs/ANIMATION.md` sections 1 to 6 and `docs/FEEL_AUDIT.md` first. Already in place:

| Area | Where |
| --- | --- |
| One rig, shared clip library, per-class overrides, progress-based attack timing, per-hit strikes for multihit moves | `anim_library.gd`, `fighter_view.gd` (`strike_progress`) |
| Whole-body squash and stretch, forward lean and attack lunge as damped springs; footfall squash on runs | `fighter_view.gd` `apply` (`SQUASH`, `LEAN`, `SPRING_K`) |
| Midair flip, roll somersault, spot-dodge step, ledge swing and travel, knockdown lift | `apply`, `_ledge_motion` |
| Flinch snap, launch pose along the path, tumble spin scaled by speed | `FLINCH_TIME`, `LAUNCH_POSE_SPEED`, `tumble_angle` |
| Hat lag and bounce (a two-axis spring on a pivot) | `_hat_spring` |
| Weapon-arm and kicking-leg IK onto the live hitbox, body bend to let a limb reach | `_aim_arm`, `_aim_leg`, `_body_follow` |
| Swing arcs (drawn crescents for blades and claws), speed lines, ghost fade, hitlag shake, whole-body hit flash, dust puffs | `ARCS`, `_apply_combat`, `_flash` |

Already-listed gaps (`ANIMATION.md` section 6 and inventory row F): heavy hitstun pose, shield breathing and shield-hit flinch, hand and
head follow-through, a tech, a platform drop, a run stop.

## Corrections to the earlier suggestion list

- **Foot IK at platform edges: dropped.** Stages are flat platforms and the simulation positions feet from the ECB, so IK has almost
  nothing to correct. The real foot problem, if there is one, is **foot sliding** while walking and running; that is measured in phase 0
  and fixed in phase 4 by clip speed and stride, not IK.
- **Spring bones are scoped, not general.** The rig has no hair, sash or tail bones and the neckwear is rigid meshes, so
  `SpringBoneSimulator3D` only pays off for a few new multi-segment accessories (phase 2b). Plain code springs, as `_hat_spring` already
  does, are the right tool for head, spine, hands and rigid accessories.
- Verified on this machine: Godot 4.7.1 has `SpringBoneSimulator3D`, `TwoBoneIK3D`, `LookAtModifier3D`, `IKModifier3D` and `Decal`.

## Hard constraints

- Read `CLAUDE.md` and `docs/STATUS.md` first. The working tree on `phase7-match-rules` may hold unrelated uncommitted sim and moveset
  changes. Branch from `main` (own branch or worktree) and never commit files from other work.
- No sim-crate edits. The view reads the snapshot dictionary `s` (built in `main.gd` `_refresh`). If a state cannot be told apart from
  the snapshot (for example a tech), add a **read-only bridge accessor** over existing state; that is not a sim change, but confirm it with
  the user before touching `pf_bridge`.
- **Rollback and replays**: the view only ever sees the latest snapshot, so any new state (springs, history buffers) must tolerate
  position and frame jumps (rollback corrections, replay seeks, respawn). Follow the `prev_frame` pattern (`frame - prev_frame <= 2`),
  clamp every spring output as `hat_roll` and `hat_drop` do, and add a `reset_motion()` used on respawn, seek and rebuild.
- **Hitlag**: while `s.hitlag > 0` the pose is frozen on purpose (`FEEL_AUDIT.md` A1). New motion must not unfreeze it. Springs pause
  during hitlag except the victim's deliberate shake.
- **Shared clips, one rig**: locomotion, jumps, landings, defence and hit reactions never change with the moveset. New clips are made in
  `art/blender/make_rigged_blob.py` and mapped in `anim_library.gd`; never copied per moveset. Needles class (2) uses its own kicks.
- Keep the face readable: no limb across the face (style guide, `ANIMATION.md` section 4). Smears and stretch must never hide the face.
- Fighters stay the most readable thing on screen: run `tests/look_bench.gd` after changes that alter silhouettes.
- Do not grow `fighter_view.gd` (2,655 lines) further than needed. Put new logic in small modules under `godot/scripts/`
  (`motion_springs.gd`, `smear.gd`, `tail_rig.gd`) with `fighter_view.gd` calling them.
- Four-player free-for-all exists: budget for four fighters. Target at most 0.4 ms total added per frame at 1080p on the dev machine;
  report the measured delta per phase. Tier **Fast** (`visual_settings.gd`) drops smear ribbons and spring-bone chains but keeps
  code springs.
- Everything original; never use the word "Smash"; no external assets in this plan.

## How to verify

```bash
# Contact sheet of a clip or move (see ANIMATION.md section 5)
tools/godot/Godot_v4.7.1-stable_win64_console.exe --path godot --script res://tests/anim_sheet.gd -- --what=<clip or move> --class=<0|1|2> --out=<png>
# Existing checks that must stay green
tools/godot/Godot_v4.7.1-stable_win64_console.exe --headless --path godot --script res://tests/anim_library_test.gd
tools/godot/Godot_v4.7.1-stable_win64_console.exe --headless --path godot --script res://tests/fighter_view_test.gd
# Rig rebuild after pose changes (about 20 s)
"C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/make_rigged_blob.py -- --body art/models/base_body/base_body.glb --joints art/models/base_body/joints.json
```

Also run the demo stills (`--demo=...` in `demo.gd`) at several frames around each change, plus `match_flow_test` and, once at the end,
`cargo test --workspace`. **Screenshots show poses, not feel.** Every phase ends with a short capture set (strips or a GIF) saved to
`docs/screenshots/motion_<phase>_*` and a request for the owner to play it. Report honestly what looked wrong.

## Phase 0: measure and de-risk (small, do first)

1. **Baseline strips** for the states most seen: idle, walk, run, dash, jumpsquat, jump, land, jab, ftilt, fair, fsmash, hurt, flinch,
   launch, tumble, knockdown, getup, shield. Save as `motion_0_*`.
2. **Foot-slide metric** (`godot/tests/motion_probe.gd`, headless-capable): for walk, run and dash at several sim speeds, sample the
   planted foot's world x per frame during its stance phase and report its drift in units per frame against the sim's ground speed.
   Record the baseline; phase 4 is driven by it.
3. **Spike: `SpringBoneSimulator3D` on this rig** (throwaway scene, not shipped). Questions: does it update while the `AnimationPlayer` is in
   manual process mode and with the existing `set_bone_global_pose_override` arm and leg IK? Does it behave when the fighter node is
   teleported each frame from the sim and when the model is scaled for squash? Can it be paused during hitlag? **Go/no-go**: if any
   answer is no or flaky, phase 2b uses a hand-written chain of code springs instead (same visual result, more code, fully deterministic
   from the snapshot).
4. **Pose-override check**: `set_bone_global_pose_override` is the old API. Confirm it emits no deprecation warnings in 4.7.1 and still
   composes with any new modifier. If it does not, phase 5 becomes required, not optional.

Done when: baselines and numbers saved, spike result written into this document under "Spike results".

## Phase 1: body deformation and smear (largest feel win per effort)

The model scale already squashes uniformly. Add deformation that follows the action:

1. **Strike stretch**: on the strike frames of an attack (from `move_timing`: first active frame through `last + 2`) scale the striking
   limb's bones along their length (about 1.12 to 1.25, eased in over the frame before and out over the follow-through), volume-
   preserving across the limb. The arm or leg IK (`_aim_arm`, `_aim_leg`) must target the stretched reach so the tip still lands on the
   hitbox. Bone scale should be tried on `armU`/`armL`/`hand` and `thigh`/`shin`/`foot` first.
2. **Launch stretch**: a fighter flying fast in hitstun (above `LAUNCH_POSE_SPEED`) stretches along its velocity (up to about 1.25 along,
   0.88 across, volume kept), easing in as it leaves hitlag and relaxing as speed falls; it replaces nothing in the tumble spin.
3. **Victim impact squash**: for the freeze of a strong hit, squash the victim along the hit direction (a few percent per tier of
   hitlag, capped), released into the launch stretch on the first frame after hitlag. Strength comes from the existing `flash_strength`
   input (hitlag) so it needs no new data.
4. **Smear ribbons for fists and feet**: the weapon arcs (`ARCS`, `_update_arc`) cover blades and claws. Generalise a drawn ribbon to
   the hand and foot tips of punches and kicks: keep a short ring buffer (3 to 5 frames) of the tip's world position, advanced only on
   frames where the sim frame advanced and never during hitlag, and draw one opaque crescent in the move's trail colour that fades over
   `ARC_FADE`-like frames. Reuse the arc mesh and material (see `ARC_STEPS`, `ARC_Z`) rather than a second system. Drawn only for strike
   frames of moves that have no authored arc.
5. **Landing and take-off**: add a very short horizontal stretch on take-off (the frame after `JumpSquat`) so the spring does not just
   compress and rebound.

Done when: contact sheets show visibly longer strikes and a launch that reads as speed; hitboxes and hurtboxes untouched (check by
drawing the debug overlay on top, as the move tip still lands on the live hitbox); silhouette safety (`look_bench`) green; the face is
never covered in any strike; frame-time delta reported; owner approves a playtest GIF set.

## Phase 2: secondary motion

### 2a. Code springs (do first)

Generalise `_hat_spring` into a small reusable spring set (`motion_springs.gd`): per-channel stiffness, damping, clamp, and the
`prev_frame` discontinuity guard. Use it for:

1. **Head and spine follow-through**: the head and upper spine lag the hips on starts, stops, dashes and turns (`ANIMATION.md` section 6
   item 3), a few degrees at most. Drives bone rotation offsets applied after the clip, like `_body_follow`.
2. **Hand follow-through after strikes**: a small overshoot and settle on the hands for the frames after the strike (item 3 again).
3. **Neckwear and hat accessories**: the existing rigid meshes (sash, neckerchief, scarf tail) get a pivot with the same spring so
   they swing with movement. A first pass needs no new bones.

### 2b. Real chains (only if the phase 0 spike was a go, otherwise as code-spring chains)

A self-contained **tail rig** (`tail_rig.gd`): a small `Skeleton3D` with 3 to 4 bones, a skinned ribbon mesh in the toon material, and one
`SpringBoneSimulator3D`, parented under the torso. Use it for the scarf tail first (replacing the rigid tail box in `_neck`), then a
sash end. Keep it off the main skeleton so the shared rig and every clip stay untouched. Respect the accessory mesh budget in
`loadout.gd`; do not add new catalog items in this plan.

Done when: moving, stopping and turning visibly drags the head, hat and neckwear a beat late and settles without wobble at rest
("restraint at rest", style guide); a rollback correction or replay seek does not cause a visible spring explosion (test by feeding a
position jump into `apply` in `fighter_view_test.gd`); hitlag freezes them; frame-time delta reported.

## Phase 3: missing and weak clips (needs Blender pose work)

Authored in `make_rigged_blob.py` (poses are code, reviewable), mapped in `anim_library.gd`, covered by `anim_library_test.gd`:

1. **Heavy hitstun**: a launched-but-not-tumbling pose (distinct from `hurt`), directional where cheap (high, middle, low hit; far-arm
   and near-arm are fine to share).
2. **Shield**: a small breathing hold that does not move the shield bubble, and a flinch clip or additive pulse on shield hits.
3. **Tech**: a quick pop-up for an in-place ground tech, with roll left and right variants. First establish what the snapshot reports for
   a ground tech (the sim lists `WallTech`, `Knockdown` for a missed tech, `GetUp`); if the view cannot tell a tech from a normal
   landing or roll, that needs a read-only bridge accessor (see constraints). Do not guess.
4. **Platform drop**: a short drop-through pose (a crouch tipping forward) on whatever the snapshot exposes for it.
5. **Run stop**: a skid-to-stand clip when a run ends (`Turn` handles reversals only).
6. **Idle variety**: one or two rare idle fidgets after several seconds of stillness (look around, shuffle); never during a match
   state that matters (only `Idle`, long stillness, training and menu previews).

Done when: each new clip has a contact sheet and a mapping; `anim_library_test` passes with the new mappings; a clip never changes
timing the simulation owns (`ANIMATION.md` section 1, "Timed states"); owner approves poses.

## Phase 4: locomotion quality

Driven by the phase 0 foot-slide numbers:

1. Tune per-clip `speed_scale` and stride (`_choose_clip`, `anim.speed_scale = pick[1]`) so planted feet do not slide at walk, run and
   dash speeds across the fighters' parameter range (heavy and light bodies have different speeds). Target: stance-foot drift under a
   few percent of ground speed; report before and after.
2. **Weight shift on a turn** and a short **settle** after a dash ends (a spring-driven small lean back), using the 2a spring set.
3. Walk-to-run blend continuity: check the 0.1 s cross-fade does not pop the feet.

Done when: the probe's slide numbers beat baseline for all three locomotion states and no clip pops at transitions.

## Phase 5: optional, only if phase 0 item 4 says so

Replace the `set_bone_global_pose_override` arm and leg IK with a `TwoBoneIK3D` modifier (or equivalent) on the shared skeleton, keeping the
same targets (live hitbox tip). Behaviour must be identical, verified with the move-tip sheets for every move of every class. Skip if the old API
works cleanly in 4.7.1.

## Phase 6: docs and wrap-up

- Update `docs/ANIMATION.md`: sections 1 (layers on top of the clip: deformation, smear, springs), 2 (inventory rows B and F), 4 (style
  guide: when stretch and smear are used) and 6 (plan: what is done and what is next).
- Update `docs/STATUS.md` with what shipped, what was skipped and why, and the measured frame-time and foot-slide numbers.
- Record the spike result and any bridge accessor request here.
- Final: `cargo fmt --all`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, the Godot tests above, and
  `look_bench`.

## Suggested order and sizing

Phase 0 (small), 1 (medium, biggest feel win), 2a (small to medium), 3 (medium to large, needs the owner's pose review), 2b (medium, after
the spike), 4 (small to medium), 5 (only if needed), 6 (small). Commit per phase with its captures. Phases 1 and 2a can be started in
parallel by separate agents if their module files stay separate.

## Spike results

*(To be filled in by phase 0.)*
