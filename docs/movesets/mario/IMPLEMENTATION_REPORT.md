# Embers moveset: implementation report

The fourth moveset (in game: the **Tumbler**, class 3, weapon **Embers**), modelled mechanically on the reference game's Mario,
version 13.0.1. Phase 1 (this report): the body and every normal, sim version 57. The audit is `PROJECT_AUDIT.md`; every number and
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
| Specials | **still the claws' placeholders** (blaster, flash dash, fire flight, reflector). Phase 2 below |
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

* Specials are the claws' placeholders (phase 2).
* Up smash head intangibility (9-12) not modelled (needs per-hurtbox intangibility, E19).
* The face-down and trip get-up attacks have no slot (one get-up attack per weapon); the face-up one is used.
* Wall jump (the reference character has one): not built (E15).
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

## Phase 2 plan: the specials

Order: **E11 first** (checked against the Longsword and Claws up specials, the owner's decisions), then Super Jump Punch, Fireball
(E3b + age phases), Cape (E10, E10b, E16, uses E11), F.L.U.D.D. (E2 + the charge-scaled spawn), wall jump (E15) last. Each extension
below is general, defaults to today's behaviour, is hashed, has a content field and a test in its topic file, and bumps the sim
version once per phase.

### E11: per-move recovery rules

Shape (new optional fields on `Move`, content `recovery { ... }` block):

```text
ledge_front_from: u8   // move frame from which a ledge in front can be caught; 255 never. Default: 0 if grabs_ledge else 255
ledge_back_from:  u8   // the same for a ledge behind (the shorter back reach is the body's, as now)
fall_air_speed:   u16  // percent of the body's air speed and acceleration in the special fall after this move
                       //   (default: Ruleset::helpless_drift, today's global value)
fall_landing_lag: u8   // landing lag of that special fall (default 0 = the body's helpless_landing_lag, today's)
air: Option<{ gravity: Fx, max_fall: Fx, air_speed_percent: u16, air_friction: Fx }>  // the move's own air physics while it runs
                       //   without a motion override (Cape's 0.031 / 1.0 / speed / 2.5); default None = the body's
steer: Option<{ frame: u8, stick_min: Fx, max_degrees: u8 }>  // the motion rows are rotated toward the stick read on `frame`,
                       //   at most `max_degrees` (Super Jump Punch 12, the Longsword's 20); default None
```

`grabs_ledge` stays and means "both sides from frame 0" so old content reads the same. The special fall reads its numbers from the
move that caused it (`Fighter::move_id` is still that move in `Helpless`), so no new fighter state is needed. Ledge side: the ledge
found by `collision::find_ledge` is in front when its side faces the fighter, else behind.

What it changes for existing fighters: nothing until a move sets the fields. Then, by the owner's decisions:

* **Longsword up special** (reference: SmashWiki Marth up special, 13.0.1): ledge from the front from frame 8, never from behind during
  it; helpless air speed x0.4; landing lag 24 (the special fall's, today it is the body's 20); intangible 4-5 grounded, 1-5 in the air
  (today 1-5 both: needs a grounded/aerial split, a small `intangible_air` field or a script); hits 11% on 5-6 and 7% on 7-11 (today
  7% from 6); angle up to 20 degrees by the stick (0.5); "gravity after jump 0.06", "aerial height x1.1". The rise is root motion: its
  height needs measuring (question 2). Tests that move: `reference_moves::up_special_is_intangible_for_its_first_five_frames`,
  `up_special_rises_about_44_reference_units_and_leaves_the_ground`, `up_special_hits_on_frame_5_for_11_percent_then_ends_helpless`,
  `an_up_special_grabs_the_ledge_in_the_middle_of_the_move` and the three after it (they use the Longsword's up special; the
  mid-move grab must happen from frame 8 and in front), `ground_and_collision::helpless_landing_costs_extra_lag` (it reads the body's
  20) and `edges_and_recovery` recovery distances if they use it.
* **Claws up special** (reference: SmashWiki Wolf up special, 13.0.1): wind-up hit 18-21 (4%, 93 degrees, BK 50, KS 10, set weight),
  loop 27-32 (2.5%, 40 degrees grounded targets / 367 airborne, rehit 2, SDI 0, H 0.5), final 36-37 grounded / 37-38 aerial (5% and
  6%, 361, BK 60, KS 148 / 136, H 2.5); FAF 67 grounded; ledge from behind from 26 and from the front from 27; the dash decided on
  frame 16, 7 frames at 3.8 with friction 0.2 after 4 frames, then x0.7 (ground) / x0.8 (air); a bounce off the ground at an angle
  difference over 20 degrees (22 frames, x0.4 horizontal, 3 up, gravity 0.2); helpless air speed x0.65; landing lag 36. The dash and
  the bounce stay a script (`fire_aim.script`: `set_vel` per frame and `grounded`); the ledge windows and the special fall are E11.
  Tests that move: `kits::fire_wolf_goes_where_the_stick_pointed_and_turns_around_for_backward`, `reference_moves::fire_wolf_*` (three:
  hits, rise, mid-move ledge grab; renamed by the naming cleanup on `ship-polish`).
* **Mario's Super Jump Punch** (below), and later Link's Spin Attack, Donkey Kong's Spinning Kong, Cloud's Climhazzard, Captain
  Falcon's Falcon Dive, Palutena's Warp.

### Super Jump Punch (up special)

Data plus E11: hit 1 (3-6) and hits 2-6 (7-16, rehit 2, its own group re-armed) and hit 7 (17-18, its own group) as rows, all set
weight except hit 7, SDI 0 on 1-6, set knockback 160 / 150 / 150 and 180 / 170 / 110 / 110; intangible 3-6 (`Move::intangible` counts
from frame 0: needs an `intangible_from` field or the existing script `intangible(n)` on frame 2); B-reverse on frame 3 (exists:
`special_turn_window` is a ruleset value of 3 frames today, check it matches the stick sensitivity 0.3); E11: rise as motion rows
fitted to the measured height (x1.1 grounded, x1.08 aerial), `steer { frame 2, stick 0.5, 12 degrees }`, `ledge_front_from` and
`ledge_back_from` 13 (frame 14), `fall_air_speed` 60, `fall_landing_lag` 30, startup air speed x0.67. Tests: hit sequence and damage,
set knockback ignoring rage (already true: `fixed_knockback` skips rage), intangibility window, rise height within a stated
tolerance of the measured value, angle by stick, ledge grab not before frame 14, helpless drift and landing lag 30.

### Fireball (neutral special): E3b and age phases

Shape (on `ProjectileSpawn`, content fields with defaults):

```text
gravity: Fx, max_fall: Fx          // default 0 (today's straight flight); Fireball 0.1 (reference units), fall cap from footage
launch_degrees: i16                // initial angle (-10); default 0
ground: Die | Pass | Bounce { vy_min: Fx, vy_max: Fx, keep_x_percent: u8, count: u8, wall_max_degrees: u8 } | Lodge { frames }
                                   // default: today's behaviour (passes over the ground and dies in a solid wall)
phases: [(age_from: u8, Hitbox)]   // the hitbox by age (clean / mid / late); default: one hitbox with today's linear falloff
```

The bounce is checked against the stage's surfaces in `combat::update_projectiles` (deterministic, fixed point); a bounce count is a
hashed `Projectile` field (with its checksum case). Fireball: spawned on 17, speed 1.4, 70 frames, 7 bounces, vertical speed 1.4 to
1.6 after a bounce, x0.97, walls under 60 degrees bounce, three hit phases (5% BK 35 KS 20, 5% BK 28 KS 15, 4% BK 22 KS 10), H 0.6,
shield damage -2.5 / -2.0 (additive: a percent per phase, [D]); Mario keeps full air control (an aerial special with no motion: exists).
Also used by Yoshi's Egg Throw and Link's arrows (Lodge). Tests: arc and bounce heights, bounce count and life, phase damage by age,
wall bounce, reflected by the Claws' reflector.

### Cape (side special): E10, E10b, E16, E11

* **E10** `Hitbox::reverse: bool` (content `reverse`): on hit the victim's facing is flipped and its horizontal speed reversed (the
  reference's "Reverse" effect), after the launch is worked out; with set knockback 80 at 110 degrees. Default false.
* **E10b** `Reflector::reverse_vertical: bool` (default false: today's horizontal-only reversal), `Reflector::life_percent: u8`
  (default 100; Cape 75), `Reflector::max_damage_percent`-style cap left as is (the reference's 50% "damage threshold" is a reflect
  limit, to research); `Reflector::detect_from: u8` (default 255): a projectile touching the field from this frame jumps the move to
  `start` (Cape's detector on 6-8). The Claws' reflector is re-checked against its reference under decision 6 (vertical reversal).
* **E16** once-per-airtime flags: `Fighter::air_flags: u8` (hashed, checksum case), cleared on landing, on being hit and on a ledge
  grab; script `air_flag(n)` (read) and `set_air_flag(n)` (whitelist entries in `sim-script/src/api.rs`, implemented in
  `scripting.rs`, tested in `scripted_moves.rs`, documented in `docs/CONTENT.md`). Cape's +0.3 vertical speed applies only when flag 0
  is clear (the fall break once per airtime).
* **E11** for its aerial physics (gravity 0.031, fall 1, speed / 2.5, friction 0.0025) and ledge windows (front from 2, back from 16).
* Hit 12-14 (7%, r 7.5 / 5.0, shield +4), reflector 9-20 at (0, 6.5, 8) size 6.5 (radius 8 tenths [D]), x1.5 damage, FAF 36.
  Tests: a projectile reflected on 9-20 (and on 7 with the detector), its life cut, a melee hit turning the victim round, the fall break
  only once per airtime, ledge grabs by side and frame.

### F.L.U.D.D. (down special): E2 and a charge-scaled spawn

* **E2** `Hitbox::push: bool` (content `push`, the reference's flinchless / wind hit): damage still applies (0 here), the knockback is
  added to the victim's velocity as a push, with no hitstun, no hitlag, no tumble and no state change (the victim keeps control);
  shields take the push as pushback. Default false. Also used by Cloud's Finishing Touch wind and the Needles' Vanish wind box.
* **Charge-scaled spawn**: a script ability `spawn_scaled(fx, up, vx, vy, kb_add, size_percent)` (whitelist entry, test, doc), so each
  pump carries the charge's extra base knockback (up to +55) and size (x0.8 to x1.2) and speed (4.3 + up to 4.3). The pump's gravity
  (0.17, fall 3) is E3b's projectile `gravity`.
* Charge and store: the Needle Storm pattern (exists: `stall` / `goto` loop, `store`, shield / jump store; "charge-cancel with no
  lag"); the full charge is 80 or 100 frames (question 1). Release: 7 pumps every 5 frames, aimed from 12 degrees and turned by the
  stick 1.9 degrees a frame within +50 / -18 (a script: rotating the shot velocity by fixed sine and cosine constants), the recoil
  (0.02 per charge frame) by `add_vel`. FAF 49 from release.
* Pool: 7 pumps with a 12-frame life every 5 frames means at most 3 alive at once; `MAX_PROJECTILES` 8 is shared by every fighter, so a
  four-player match could run short (question 5).
* Tests: no damage and no hitstun on a hit, push strength rising with charge, a shielding target pushed, store and spend, aim limits,
  recoil.

### Wall jump: E15

`FighterParams::wall_jump: bool` plus `wall_jump_vx`, `wall_jump_vy`, `wall_jump_min_speed` (the reference's passive wall jump 1.3 /
2.3 / 0.2), a stick-away press near a wall in the air. Default off (no fighter has it today). Also Captain Falcon and the Needles.

## Questions for the owner

1. **F.L.U.D.D.'s full charge**: SmashWiki's parameter table says 80 frames, ultimateframedata.com says 100. Can you record it (or
   should I keep looking for a third source before phase 2)?
2. **Up-special rises are root motion**: the reference publishes only multipliers (Super Jump Punch x1.1 / x1.08, the Longsword's
   reference x1.1) and not the heights. For 1:1 travel I need a measured height and drift for Super Jump Punch, Cape's fall break, the
   Longsword's and the Claws' up specials: footage you record (frame-stepped in training mode on a flat stage, from a standing start
   and from a full hop), or may I fit them to a published recovery-distance figure if I find one (labelled [D])?
3. **Angled forward tilt / smash threshold**: ours is 0.25 of the stick ([E]). Keep, or do you have a measured value?
4. **Pummel FAF**: the Needles' pummel note says its FAF comes from a total that includes the pummel's hitlag; our hitlag also freezes
   the move, so that pummel may be too long by its hitlag (not re-checked this run). The Embers' pummel uses the reference's animation
   frames (FAF 7 + 1). Re-check the Needles' and Claws' pummels under the 1:1 rule?
5. **Projectile pool**: F.L.U.D.D. alone can hold 3 of the 8 shared projectile slots. Raise `MAX_PROJECTILES` (a state-size change, a
   net-fuzz run) when building phase 2?
6. **Get-up attacks by facing**: the reference has face-up, face-down and trip get-up attacks; we have one slot. Add two slots now
   (a small E17-like content change), or leave it until E17?
