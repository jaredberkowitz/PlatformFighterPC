# Embers moveset: implementation report

The fourth moveset (in game: the **Tumbler**, class 3, weapon **Embers**), modelled mechanically on the reference game's Mario,
version 13.0.1. Phase 1: the body and every normal, sim version 57. Phase 2 (below): the specials, the wall jump and the shared
extensions they needed, sims 58 to 64. The audit is `PROJECT_AUDIT.md`; every number and
its label ([R] published, [D] derived, [E] estimate) is in `REFERENCE_DATA.md`.

## What was built (phase 1)

| Area | Built |
| --- | --- |
| Body | `FighterParams::tumbler`: weight 98, walk 1.155, initial dash 1.936 (10 frames), run 1.76, traction 0.102, walk / run acceleration from the parameter file, air speed 1.208, air acceleration 0.01 + 0.07, air friction 0.015, gravity 0.087, fall 1.5 / 2.4, full hop 36.33, short hop 17.54, double jump 36.33, ledge jump 38.165, heavy landing 3 frames of lag, the reference's air dodges (52; 71 / 87 / 116), rolls and spot dodge |
| Jab | jab 1 (2.2%) → jab 2 (1.7%) → jab 3 (4%) on the reference's continuable frames (5 and 6) with buffered presses; held attack repeats jab 1 from frame 7 |
| Tilts | forward tilt (angled up, straight or down), up tilt (the fist sweeping front to back), down tilt (leg 7%, toe 5%) |
| Dash attack | clean 6-9 (8%, 50 degrees, BK 100, KS 43), late 10-25 (6%, 48 degrees) |
| Smashes | forward smash angled three ways (the arm wins over the explosion where both reach; the explosion's growth per angle), up smash, down smash front then back |
| Aerials | neutral (clean / late), forward (early / meteor clean / late), back, up (front-to-back sweep), down (five set-weight loop hits, a final hit, a landing hit) with the reference's landing lags and autocancel windows |
| Grabs, throws | standing / dash / pivot grabs with the reference's reach; pummel 1.3% (set knockback 30); forward, back (with its spin hitting bystanders, turning round), up and down throws |
| Ledge and floor | ledge attack (9%, 24-26, intangible through 26), get-up attack face up (behind then in front, 7%) |
| Specials | phase 1: the claws' placeholders; **phase 2 (below): the rising punch, the fireball, the cape, the water pump** |
| Registration | class 3 in `sim-content/src/recipe.rs` (`CLASSES = 4`), `Content::placeholder` (fighter "tumbler", weapon "embers"), `godot/scripts/roster.gd` (`CLASS_NAMES`, a blurb, the built-in "Tumbler"), `content/base.pfc` re-exported |

Names: the class **Embers**, the body **tumbler**, the built-in fighter **Tumbler**, the blurb "Fast fists and feet: an all-rounder with
high jumps, up-air juggles, a spinning down air and a meteor overhand." None is on the trademark list (`sim-content/src/policy.rs`)
or `content/blocklist.txt` (the content loader's checks pass on `base.pfc`).

## Engine extensions (all general, defaulting to the old behaviour)

* **`Move::next_from`** (content `next_from`, default 255): a jab whose next hit may start before its end. From that move frame a press
  of attack (within `next_window`, buffered, so a press in the hitlag counts like the reference's buffer) carries on at once. Before, a
  chain only continued at the end of the move, or by a script that cannot see presses made in hitlag. Test:
  `kits::a_jab_with_an_early_continuable_window_carries_on_as_soon_as_attack_is_pressed_in_it`.
* **`Hitbox::aim`** (content `aim any|up|straight|down`, default any) and **`Fighter::aim`** (hashed; checksum case `aim` in
  `state.rs`): a forward tilt or smash started with the stick tilted up or down by at least `STICK_AIM` (32, about 0.25; short of the
  up / down attack line at 64) is aimed that way, and only the hitboxes for that aim come out. Every other move resets the aim to
  straight. Test: `multihit::an_aimed_hitbox_is_out_only_when_the_move_was_aimed_its_way`.
* **`Hitbox::set_weight`** (content `set_weight`, default false): the knockback is worked out as if the victim weighed 100. Test:
  `multihit::a_set_weight_hit_launches_light_and_heavy_fighters_alike`.

None of the existing moves uses them, so the other fighters are unchanged: every existing kit test passes as before, and `base.pfc`
differs from the old one only by the new class (and the sim version and hash). `SIM_VERSION` 56 → 57; no schema bump (only optional
fields with defaults were added).

## Verified, derived, estimated

* **Verified against the reference [R]**: every normal's active frames, FAF, landing lag and autocancel windows; damage, angle, BK,
  KS, set knockback, set weight, hitlag and SDI multipliers; the jab's continuable frames and quickest loop; the smash charge frames;
  the ledge and get-up attacks' frames and intangibility; the body's numbers, and its short hop, full hop and fast-fall air times
  (within one frame of 40 / 56 / 28 / 39).
* **Derived [D]**: radii and `top`-bone positions (converted units), the additive shield damage as a percent, the pummel one frame
  later (hit on 2, FAF 8, as our pummel cannot hit on its first frame), the reading of "consecutive jab frame 7" as the held repeat,
  the back roll's intangibility (the forward roll's window).
* **Estimated [E]**: positions of hitboxes on other bones (the jab 2 punch, the tilts, up smash, aerials, the forward smash arm and
  explosion, the angled poses), the angled-move stick threshold, the back throw's collateral positions.

## Tests (actually run)

New file `sim-core/tests/embers.rs` (22 tests): frame data of every normal against the table (data); every attack's first hit on
its reference frame by running it (including all three aims of the forward tilt and smash, both down smash hits, the down air's
loop); every attack's FAF by running it; damage / angle / BK / KS of every phase and the other multipliers; the forward smash's arm
beating the explosion; jab → jab 2 → jab 3 on frames 5 and 6; the held jab repeating from frame 7 and ending on FAF 20; the stick's
aim choosing the boxes; an upward forward tilt hitting a target above; the down air's five loop hits on 5, 7, 9, 11, 13 and its final
hit on 23 (12.5% total); set weight (a light and a heavy target launched the same by the loop, not by the final hit); landing lag and
autocancel windows of the aerials; the down air's landing hit; the standing grab on frame 6; the throws' release frames and damage
(and the back throw turning round); the back throw's spin hitting a bystander but not the held fighter; the pummel; the jump air
times; the body numbers; the new hitbox and move fields in the content hash; random play with the tumbler deterministic and
identical after a mid-match snapshot (6 seeds, 900 frames).

Also changed: `ground_attributes.rs` (the tumbler's ground numbers; the "no two fighters move the same" test now allows a shared
acceleration, since the reference's Mario and Wolf share 0.105 / 0.10978, but still requires different speeds and traction), and the
tests that counted the built-in fighters or classes (`sim-content` `recipe.rs`, `doc.rs`, `tests/bundle.rs`, `tests/recipes.rs`;
Godot `content_e2e.gd`, `editor_api_test.gd`, `online_fighters_test.gd`).

Commands run (with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, the drive being nearly full):
`cargo fmt --all`; `cargo test --workspace --exclude godot-bridge` (47 suites, 717 tests, all pass); `cargo test -p godot-bridge`
(pass); `cargo clippy --workspace --all-targets -- -D warnings` (clean); `pftool fuzz-rollback 100` and `pftool net-fuzz 50` (0
mismatches; both use fixed rosters of classes 0 and 1, so the tumbler's determinism is covered by the snapshot test above). The Godot
headless tests were **not run** (no Godot in this worktree); the three edited Godot tests only change counts and names.

## Training-mode check (what a unit test cannot show)

Training (F1/F2, hitbox view F3), Tumbler against any fighter at 0%, stage centre:
1. Stick forward-up a little (not an up tilt) + attack: the forward tilt's kick comes out above the straight one; forward-down: below.
2. Flick forward with the stick a little up / down + attack: the forward smash's explosion moves up / down.
3. Tap attack three times quickly: jab, straight, kick. Hold attack: the first jab repeats.
4. Down air into a standing opponent from a short hop: five small hits, the final hit, and on landing early the 2% landing hit.

## Limits (phase 1)

* Up smash head intangibility (9-12) not modelled (needs per-hurtbox intangibility, E19).
* The face-down and trip get-up attacks have no slot (one get-up attack per weapon); the face-up one is used.
* Wall jump (the reference character has one): built in phase 2 (E15).
* The throws' thrower intangibility and the throws' "interrupted" rows are not modelled.
* The angled-move stick threshold is an estimate; with a keyboard, angling needs a partial stick (the attack stick always aims
  straight).

## For the art agent

New moves fall back to the shared archetypes (`anim_library.gd` `SHARED_MOVES`); none has its own clip yet. Clips that would match the
hit data (first active frame, active span, pose):

| Move | Active | Pose and where the hitboxes are |
| --- | --- | --- |
| Jab 1 / 2 / 3 | 2-3 / 2-3 / 3-4 | a short punch at chest height, a straight with the body behind it, a rising front kick to head height |
| Forward tilt (x3) | 5-7 | a lunging side kick at waist height; the up variant kicks to head height, the down variant to the shins |
| Up tilt | 5-11 | a spinning uppercut: the fist sweeps from in front, over the head, to behind |
| Down tilt | 5-7 | a crouching leg sweep along the floor, toe furthest out |
| Dash attack | 6-25 | a long feet-first slide |
| Forward smash (x3) | 15-17 (charge on 5) | a palm thrust with a burst of flame past the palm; angled up / down |
| Up smash | 9-12 (charge on 6) | a small hop and a headbutt sweeping back to front |
| Down smash | 5-6 front, 14 behind (charge on 2) | a low breakdance spin kicking in front, then behind |
| Neutral air | 3-27 | a held flying kick |
| Forward air | 16-21 | a wind-up then an overhand punch swung down in front (the meteor on 17-20) |
| Back air | 6-10 | a dropkick behind with both feet |
| Up air | 4-7 | an overhead kick sweeping front to back |
| Down air | 5-13 loop, 23 final; landing 1-2 | a spin with arms out (the loop), ending spread out (the final hit); a small landing spin |
| Back throw | spin 13-39, release 44 | the held fighter swung round three times, thrown behind; the thrower ends facing the throw |
| Forward / up / down throw | 13 / 18 / 18 | a spin and throw forward; a throw straight up; a slam down |
| Ledge attack | 24-26 | a dropkick climbing up |
| Get-up attack | 16-17 behind, 21-22 in front | punches behind then in front from lying face up |

The HUD's class emblem (`godot/ui/match_hud.gd` `_emblem_mark`) has no mark for class 3 yet (it draws the sword's).

## Phase 2 (sims 58 to 64): the specials, the shared extensions, the up-special re-fits

Branch `movesets` (this worktree's branch built on it), one commit per green milestone:

| Commit | Sim | Built |
| --- | --- | --- |
| `10f3793` | 58 | **E11, per-move recovery rules**, and the Longsword's and the Claws' up specials re-fitted to their references |
| `45fcd17` | 59 | **The rising punch** (up special), and set-knockback launches keeping the plain launch physics |
| `fe3cc93` | 60 | **The fireball** (neutral special), with **E3b**: projectile arcs, ground bounces and hits by age |
| `9e80692` | 61 | **The cape** (side special), with **E10** reversing hits, **E2** push hits, **E10b** fuller reflectors, **E16** once-per-airtime flags |
| `23d8a10` | 62 | **The water pump** (down special), with `spawn_scaled` and a 16-slot projectile pool |
| `c6a6f8d` | 63 | Pummels on the reference's timing with their own hitlag (the Needles', Claws' and Longsword's) |
| `7618f6f` | 64 | **E15, wall jumps** (content schema 4) |

Every number and its label is in `REFERENCE_DATA.md` ("Specials", "Wall jump", "Defaults chosen").

### The engine extensions (all general, each off by default)

* **E11, per-move recovery rules** (`Move`, content fields in `docs/CONTENT.md`): ledge windows from a frame, in front and behind
  separately (`ledge_front_from`, `ledge_back_from`; `grabs_ledge` still means both from frame 0); the special fall's drift and landing
  lag per move (`fall_drift`, `fall_landing_lag`; the fall reads the move that caused it, so no new state); height multipliers on the
  motion (`rise_ground`, `rise_air`); momentum kept on an aerial start (`start_speed`); stick angling of the motion (`steer_frame`,
  `steer_stick`, `steer_degrees`, hashed `Fighter::steer`); the move's own air physics without a motion (`air_gravity`, `air_max_fall`,
  `air_friction`, `air_drift`); intangibility starting later or only in the air (`intangible_from`, `intangible_from_air`); carrying on
  after touching down (`land_continue`). Hashed `Fighter::move_air` remembers where an attack began. The flat field layout leaves room
  for the next port's `fast_fall_from`, frame-ranged air blocks, a per-move `turn_at` and `lands_from` without changing these.
* **Set knockback keeps plain launch physics** (SmashWiki, Knockback): no launch speed-up and no shared vertical fall speed for a
  set-knockback launch (hashed `Fighter::launch_set`). It changes every set-knockback hit in the game, as the reference does; no
  existing test moved.
* **E3b, projectile flight** (`ProjectileSpawn::arc`, `phases`): a launch angle, gravity and a fall cap, bounces off platform tops (a
  count, a vertical speed band, a share of the speed kept; hashed `Projectile::bounces`), and up to two later hits by age.
* **E10, reversing hits** (`Hitbox::reverse`): the victim ends up facing away from the attacker, its horizontal speed reversed.
* **E2, push hits** (`Hitbox::push`, the reference's flinchless hit): the knockback is added to the victim's speed (sideways only on the
  ground) with no hitlag, hitstun or change of state; a shield is pushed back.
* **E10b, fuller reflectors** (`Reflector`): the shot's vertical speed turned round too, a share of its lifetime, a damage limit, and a
  projectile detector that brings the reflector up early.
* **E16, once-per-airtime flags** (hashed `Fighter::air_flags`, script `air_flag(n)` / `set_air_flag(n)`): cleared by landing, a
  ledge or a hit.
* **Scaled shots** (script `spawn_scaled`, hashed `Fighter::spawn_kb_add` / `spawn_size` and `Projectile::kb_add` / `size`); the VM
  passes up to 8 arguments (`sim_script::api::MAX_ARITY`, checked at compile time). `MAX_PROJECTILES` 8 to 16.
* **Pummel hitlag**: a pummel uses its hit's hitlag multiplier, as every other hit does.
* **E15, wall jumps** (`FighterParams::wall_jump`, `wall_jump_vx`, `wall_jump_vy`, `Ruleset::wall_jump_decay` 0.96, hashed
  `Fighter::wall_jumps`; `collision::wall_beside`): touching a wall in the air, the stick flicked away: a jump away, each further one
  before landing or a ledge x0.96. The same speeds are every body's wall-tech jump. Content schema 3 to 4 with a migration
  (`fighters_get_wall_jumps`) that gives an old fighter no wall jump and its old wall-tech jump.

### The Embers' specials

* **Up, the rising punch**: hit 1 (5%, set knockback 160 / 150 / 150, set weight, no SDI; the body box misses grounded fighters), hits
  2-6 (0.6% every 2 frames), hit 7 (3%, BK 50, KS 145) on the reference's frames; 11% when all connect; intangible 3-6; special fall
  x0.6, landing lag 30; ledges on either side from 14; x1.1 / x1.08 rise; 12-degree angling; 0.67 startup momentum. **The rise is [E].**
* **Neutral, the fireball**: out on 17 (hits 17-86), FAF 50; 1.4 at 10 degrees down, gravity 0.1, 7 bounces at 1.4 to 1.6 keeping 0.97;
  clean / late / latest hits; hitlag x0.6; half damage to shields.
* **Side, the cape**: the swing (7%, 110 degrees, set knockback 80, reverse, shield +4) on 12-14; the reflector on 9-20 (x1.5 damage,
  x0.75 life, 50% limit) with its detector on 6-8; aerial physics (1 / 2.5 of the momentum, gravity 0.031, fall 1, friction 0.0025) and
  the once-per-airtime lift; ledges in front from 2, behind from 16; FAF 36.
* **Down, the water pump**: wind-up 19, 80 frames of charge (shield or jump keep it with no lag; a full charge holds the stance, sim 65; a
  stored charge carries on), the shot: recoil 0.02 a charge frame, seven pushing pumps every 5 frames growing with the charge, aimed
  12 degrees and turned by the stick to +50 / -18; FAF 49 from the shot.
* **Wall jump**: 1.3 away, 2.3 up (the parameter file), x0.96 each further one.

Slots: `down_special` (wind-up), `ext1` (charge), `ext2` (shot); `ext0` stays the down air's landing hit. The claws' leftover follow-up
slots are cleared from the Embers' weapon.

### What changed for the Longsword

The up special (SmashWiki, 13.0.1): the clean hit on 5 (11%, 361 / 74 degrees, BK 70, KS 74) and the blade on 6-11 (7%, BK 20, KS 90,
hitlag x0.9, winning where both reach on 6); intangible 4-5 from the ground (1-5 before), 1-5 in the air; a 39-frame animation (48
before); the special fall x0.4 (0.5) with 24 frames of landing lag (20); ledges in front from frame 8 (any frame, either side, before),
none behind during the move; x1.1 rise from the air; 0.66 of the momentum kept; angling up to 20 degrees (none before); gravity 0.06
after the rise. Its rise (about 44 reference units) is still the earlier estimate. The up special is now built after the blade is
shortened, so its hand hits sit on the reference's root offsets. The pummel is the reference's (1.3%, hitlag x2.1, FAF 8 here; 20
before). The wall-tech jump is the reference's 1.3 / 2.4 (its air speed and midair jump before). Tests that moved:
`reference_moves::up_special_is_intangible_*` (now 4-5 / 1-5), `up_special_hits_on_frame_5_*` (helpless after 39), new
`up_special_has_the_references_clean_and_late_hits`, `up_special_catches_a_ledge_in_front_from_frame_8_*`,
`the_special_fall_after_the_up_special_*`, `up_special_rises_1_1_times_as_high_from_the_air`, `the_stick_angles_the_up_special_*`.

### What changed for the Claws

The up special (SmashWiki, 13.0.1) on every published number: the wind-up hit 18-21 (4%, 93 degrees, BK 50, KS 10, set weight); the
dash's loop 27-32 every 2 frames (2.5%, hitlag x0.5, SDI 0; 40 degrees and BK 60 / 70 on grounded targets, the autolink angle and BK
35 / 40 on airborne ones); the last hit by where it ends: on the ground 36-37 (5%, KS 148, hitlag x2.5; the move ends on 67), in the air
37-38 (6%, KS 136; the 71-frame animation, then the special fall x0.65 with 36 frames of landing lag); the movement built from the
published parameters (startup x0.67 momentum, friction 0.01, gravity 0.02; aimed on frame 16, straight up with the stick neutral [E];
the dash 26-32 at 3.8 less 0.2 a frame after 4; x0.7 / x0.8 after it; then friction 0.1 / 0.04 and gravity 0.1); a dash more than 20
degrees into the ground bounces off (22 frames, x0.4, 3 up, gravity 0.2, friction 0.16), a shallower one slides on; ledges behind from
26, in front from 27. Slots `ext0` (ground ending), `ext1` (air ending), `ext2` (bounce). Hitbox positions on the hip and legs are [E];
the loop's angle is measured from the facing, not the direction of travel (the reference's "speed-based" angle) [E]. The pummel is the
reference's (FAF 8 here; 20 before). It now wall jumps (1.3 / 2.3, as the reference's Wolf can). Tests that moved:
`kits::flame_kick_goes_where_the_stick_pointed_*` (the dash distance is now the reference's 25.4 units); new
`reference_moves::flame_kick_has_the_references_hits`, `_wind_up_hit_lands_on_frame_18`, `_catches_ledges_behind_from_frame_26_*`,
`_on_the_ground_ends_standing_on_frame_67`, `_in_the_air_falls_helpless_after_71_frames_*`, `_dashing_steeply_into_the_ground_*`;
`godot-bridge` `hit_windows_*` (the main move now has four hit windows, the last hit is its ending's).

### What changed for the Needles

The pummel (FAF 7 here, 16 before; hitlag x1.7 now applied), the wall jump (1.4 / 2.8, as the reference's Sheik has one) and the
wall-tech jump at the same speeds. The test that moved: `needles::the_rest_of_the_kit_has_the_references_frame_data` (pummel FAF 7).

### Verified, derived, estimated (phase 2)

* **[R]**: every special's hit data and frames, intangibility, FAF, ledge windows, special-fall drift and landing lag, height
  multipliers, angle limits, startup momentum, the fireball's flight and bounces, the cape's reflector and aerial physics, the pump's
  charge, pumps, recoil and aim limits, the wall-jump speeds and decay, the Claws' up-special parameters, the Longsword's up-special
  hits and rules, the pummels.
* **[D]**: positions on the root, radii, shield damage as a percent, the pump's charge counted from the charge's start, the fireball's
  appearing a frame early (a projectile hits from the next frame), the detector capsule as one circle, the bounce-speed band and count
  readings, the Claws' dash distance from its parameters.
* **[E]**: the rising punch's rise; the Longsword's rise; the frames the stick angle is read on and the reading of "sensitivity"; the
  cape's lift frame and stopped fall; a reflected shot's vertical reversal; the 50% limit's reading; the
  pump's aim threshold, nozzle and its push as an added velocity; pumps removed by their first hit; the Claws' neutral aim straight up,
  its bounce ending in the special fall, positions on bones; the wall-contact range; the wall-tech jump speeds used for the wall jump.

### Tests (actually run)

New or extended: `sim-core/tests/embers.rs` (46 tests: 24 new for the four specials), `reference_moves.rs` (the Longsword's and
Claws' up specials, 12 new or rewritten), `edges_and_recovery.rs` (E11 defaults and fields, the wall jump), `multihit.rs` (set-knockback
launch physics, reversing hits, push hits), `scripted_moves.rs` (`air_flag` / `set_air_flag`, `spawn_scaled`), `kits.rs`,
`needles.rs`, `sim-content/tests/bundle.rs` (the schema 3 to 4 migration; the gravity-field edits made specific to fighters), the
checksum cases in `state.rs` (`move_air`, `steer`, `launch_set`, `air_flags`, `wall_jumps`, `spawn_kb_add`, `spawn_size`,
`projectile_bounces`, `projectile_kb_add`, `projectile_size`), `godot-bridge` (`hit_windows_*`).

Commands, at every milestone (with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`): `cargo fmt --all`; `cargo test --workspace`
(at the end: 48 suites, 765 tests, all pass); `cargo clippy --workspace --all-targets -- -D warnings` (clean);
`content-export content/base.pfc`; `pftool fuzz-rollback 100` (0 mismatches) and `pftool net-fuzz 50` (0 mismatches) after each
milestone. Both fuzzers use fixed rosters of classes 0 and 1, so the Longsword's and Claws' changes are fuzzed; the Embers' specials are
covered by `embers::random_play_with_the_tumbler_is_deterministic_and_rolls_back` (6 seeds, a mid-match snapshot). The Godot headless
tests were **not run** (no Godot in this worktree); no GDScript was changed.

### Training-mode check (phase 2)

Training, hitbox view (F3), Tumbler at stage centre:
1. Up special next to an airborne opponent: seven hits, 11% (the first hit misses a grounded opponent at the body; the others reach).
2. Neutral special on the flat stage: the ball dips, bounces two or three times along the stage and is gone after 70 frames.
3. Side special against an opponent's projectile: it flies back; against a melee opponent: they end up facing away.
4. Down special, wait about 1.7 seconds or longer: the stance holds at full; shield stores it, special shoots; down special again
   with a stored full charge: the pumps push the opponent without damage.
5. Against the stage's side below the ledge, flick the stick away: a wall jump; the third in a row is visibly lower. Pressing jump
   there (stick neutral or toward the wall) wall-jumps too, and the midair jump is still there afterwards.

### Limits (phase 2)

* The rising punch's and the Longsword's rises are estimates (no published heights; open).
* The Claws' looping hits angle from the facing, not from the direction of travel.
* A push is an added velocity rather than the reference's decaying knockback velocity.
* One frame without movement where a script switches move (`goto`): the Claws' up special at the end of its dash and on a bounce.
* Partial intangibility (E19), get-up attack variants (E17), the throws' thrower intangibility: unchanged from phase 1.
* `wall_jump_enable_x_speed` and `passive_wall_x_speed` (parameter file) are not modelled.

### For the art agent (phase 2)

New moves and slots (all fall back to the shared archetypes in `anim_library.gd`):

| Move | Active | Pose |
| --- | --- | --- |
| Up special (Embers) | 3-18, rising until about 20, special fall from 38 | a jumping uppercut, fist up, spinning slightly on the rise |
| Neutral special (Embers) | ball out on 17 | a forward palm push throwing a ball from the hand at chest height |
| Side special (Embers) | swing 12-14, reflect 9-20 | a cape swept from behind across the front |
| Down special (Embers) | wind-up 1-19, charge (ext1), shot (ext2) 1-43 | a water pack worn on the back, nozzle forward at chest height, aimed up or down |
| Wall jump (tumbler, brawler, rogue) | | a kick off the wall, turned away from it |
| Claws up special | wind-up hit 18-21, dash 26-32, ending in `ext0` (ground, hit 2-3) or `ext1` (air, hit 3-4), bounce `ext2` | a spinning flame dash along the aimed direction |
| Longsword up special | 5-11 | the same rising slash; the special fall from 40 |

The Claws' up special's last hit moved to its ending slots (`ext0` / `ext1`), so `godot-bridge`'s per-move hit windows show four strikes
on the main move and one on the ending.

### Questions for the owner (answered 2026-10-13: see "Owner follow-up" at the end)

1. **Rises**: no published height or recovery distance was found for the rising punch, the cape's lift or the Longsword's rise. Keep
   the estimates until footage, or do you know a source?
2. **A full pump charge**: keep itself at once (as built, like the other auto-charging specials), or hold the stance until special or
   shield?
3. **Reflected shots' vertical speed**: built as turned round for the cape (a shot comes back the way it came). The Claws' reflector still
   turns only the horizontal speed: make both the same once confirmed?
4. **Wall jump input**: built as a flick of the stick away from the wall (the reference's). Also allow the jump button against a
   wall (an older game's option)?

## Owner follow-up (sim 65, 2026-10-13)

The owner answered the four phase-2 questions.

1. **A full pump charge holds the stance** (as the reference). `pump_charge.script` no longer stores a full charge by itself: the
   charging stance stays at 80 until special (the shot) or shield / jump (stored, no lag). The move restarts itself (`goto` to its
   own slot on frame 90) so it can hold indefinitely, as the Needles' Needle Volley does. Shield / jump storing and a stored charge
   carrying into the next use are unchanged.
2. **Reflectors stay as they are** (owner decision): the cape turns a reflected shot's vertical speed round; the Claws' reflector
   turns only the horizontal speed. They are meant to differ.
3. **Wall jump from the jump button** (shared, every body with `wall_jump` set: tumbler, brawler, rogue): touching a wall in the air,
   jump pressed with the stick neutral or toward the wall kicks off it, at the same speeds and decay as the flick. It is checked
   before the midair jump in `fighter.rs` `airborne` and uses the press up, so it never spends a midair jump. With the stick held away
   (not flicked) the press is a midair jump. ssbwiki.com `Wall_jump` publishes the jump-button input only for an older game, so this
   is the owner's rule; the flick stays the reference's.
4. **Rise heights**: re-searched, nothing credible in units was found (the list of what was searched is in `REFERENCE_DATA.md`,
   "Defaults chosen", item 2). The rising punch's rise, the Longsword's rise and the cape's lift frame stay [E]; tests unchanged.

Tests (sim 65): `embers::the_pump_winds_up_for_19_frames_then_charges_for_80_and_holds_the_full_charge` (replaces `..._keeps_a_full_charge`:
holds 480 frames, nothing stored until shield), `special_shoots_a_held_full_charge_at_full_strength`,
`a_held_full_charge_in_the_air_keeps_falling`, `shield_keeps_a_part_charge_*` (now ends with a shield press that stores 31 + 49 = 80);
`edges_and_recovery::jump_against_a_wall_with_the_stick_neutral_is_a_wall_jump_and_keeps_the_midair_jump`,
`jump_against_a_wall_with_the_stick_toward_it_is_a_wall_jump_too`, `jump_against_a_wall_with_the_stick_held_away_is_a_midair_jump`,
`every_wall_jumping_body_takes_the_jump_button` (bodies without a wall jump get their midair jump).

Limit: a held full charge in the air has one frame without movement each 90 frames, where the move restarts (the known `goto` limit).

## Sim 130: up specials halt their starting momentum

Owner decision 97 (sim 130, branch `upspecial-momentum`): up specials halt the momentum they start with, as the reference does. The rising punch now sets `start_speed_y` 0 (the published 0.67 horizontal kept; no vertical multiplier is published, and the rise is the animation's) [E: reading]; frame 1 no longer carries a jump's rise. The Longsword's up special (whose rules are listed in this report too) the same: `start_speed_y` 0 next to its published 0.66 horizontal. Before, out of a jump the move rose higher than from a standstill at the same height; now the two match (`sim-core/tests/upspecial_momentum.rs`). The audit of every fighter and its sources: `docs/movesets/SHARED_MECHANICS.md`, "Starting momentum of specials".

## Sim 143: the cape out of a jump (owner decision 102)

The cape's lift (0.3, first use in an air time) is added to a fall but no longer to a jump's rise, and a later cape stops a rise: with
the cape's 0.031 pull a kept rise floated the fighter 20 to 58 reference units over the jump it was started from [E: the reading of
the reference's "vertical speed added"]. Tests: `special_start_momentum::the_lifts_do_not_keep_a_jumps_rise`,
`no_special_rises_over_the_jump_it_was_started_from`.
