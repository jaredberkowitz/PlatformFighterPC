# Project audit for the sixth moveset (reference: the reference game's Palutena)

What the engine needs to reproduce the reference game's Palutena 1:1 (version 13.0.1), traced through the code at sim version 65 (branch
`movesets`, commit `04efe16`), counting the extensions Captain Falcon's port is planning or building in parallel as they are written in
`docs/movesets/gauntlets/PROJECT_AUDIT.md` (E1, E3c, E6, E11 additions, E16 `presses` / `wall`, E17, E19, E21-E24). Nothing is
built: this is research only. The data is in `REFERENCE_DATA.md`; the engine inventory is `docs/movesets/SHARED_MECHANICS.md`; the
extension ids are those of `docs/movesets/ROSTER_STATUS.md`, with new ones numbered from E25.

## Proposed names (not registered)

| Thing | Name | Checked |
| --- | --- | --- |
| Class / weapon | **Beacon** | not on `content/blocklist.txt` (empty apart from comments); contains none of the `TRADEMARKS` terms in `sim-content/src/policy.rs` (after its look-alike folding); not used anywhere in the code, content or `godot/scripts` |
| Body | **seer** | same |
| Built-in fighter | **Seer** | same |
| Blurb | "Light from a distance: a tall caster who sets off blasts where she points, locks on and fires, blinks across the stage and answers a strike or a shot in kind." | original wording |

Alternatives if the owner prefers: weapon "Lightstaff", body "warden". Avoided on purpose: the reference's own move names (several of
its normals have names such as "Radiance"), and names that are other companies' marks. Class index 5 when registered (after the
Gauntlets at 4), with the HUD emblem left for the art agent.

## Iconic moves: what must match exactly

1. **Warp** (up special). Startup 1-15 with gravity 0.03 and maximum fall 0.75; the direction read on frame 16; 17 frames of travel
   at 3.5 reference units a frame (59.5 units, 7.44 world units), intangible and invisible 16-32, no hitbox; the reappearance on 33 at
   half speed (horizontal cap 2.7), 35 frames long; helpless after an aerial reappearance with **22 frames of landing lag**; a grounded
   reappearance that slides off a ledge is **edge-cancelled** into an ordinary fall. **Engine**: all of it exists (script `set_vel`,
   `intangible`, E11's move air physics and special-fall landing lag, the ground-slide edge rule), with one small E11 addition
   (`land_continue` up to a frame); ledge windows wait for research (gap 8); the stage-collision behaviour of the travel is [E]
   (gap 9).
2. **Explosive Flame** (side special). Placement on frame 1, 45 or (smash input) 75 reference units ahead, fixed in the stage; six
   loop hits every 5 frames from 28 that **pull toward the centre** (angle 160 by position) while the radius grows in seven steps; the
   5.5% last hit on 57 at 84 degrees, BK 60, KS 141, x1.5 hitlag; FAF 72; no clank. **Engine**: needs **E3a** (a stationary projectile
   that survives hits, per-target rehit, more hit phases by age), **E26** (angle by position from a projectile's centre, and a
   projectile's own facing), **E3c** for projectiles (no clank) and an E16 **`flick`** register (the smash input).
3. **Counter / Reflect Barrier** (down special). One stance, 7-34, with **two exclusive answers**: a melee hit is countered (1.3x,
   10% to 50%, BK 50 / KS 70 and BK 30 / KS 65, 361, x1.5 hitlag); a projectile touching the search box starts the reflection (a body
   reflector 1-4 and a **stationary barrier** that lasts 45 frames, beyond the reflection's FAF 35, with a set-weight push 2-11; x1.4
   damage, x1.25 speed, limit 50). Intangible 6-12. **Engine**: `Move::counter` and `Move::reflector` can both sit on one move, but they
   do not interact the reference's way (below). Needs **E10c** (counter / reflector routing: melee-only counter, a detector that starts
   another move, a counter damage cap) and **E25** (a reflector left in the stage as an object).
4. **Autoreticle** (neutral special). The search 7-14 (radius 120 around the chest, in front, not intangible, line of sight); the
   reticle fixed where the target's chest was; three 3.5% shots spawned at 23 / 29 / 35 (hitting 24-46, 30-52, 36-58) at speed 6 with a
   25-frame life toward it; nothing at all if no lock; FAF 68; x0.6 horizontal speed. **Engine**: needs an E16 **target lock**
   (the "nearest opponent" register, with the reference's range, front and line-of-sight rules) and a stage line-of-sight query; the
   rest exists (script `spawn`). How the shots fly to the reticle is [E] (gap 1).
5. **Neutral air**. Six loop hits every 4 frames from 5 at the autolink angle on airborne targets (BK 0, KS 100: drag-down at high
   percent) and set-knockback pop-ups on grounded ones; the late hits' set knockback with **-13 hitstun**; the last hit **always forward**
   (angle type 3) from a 9.8 / 11.5 radius around the staff; landing lag 12, autocancel 1-3 / 40+. **Engine**: autolink, rehit rhythm,
   grounded / aerial targets, set knockback exist; needs **E27** (a negative hitstun modifier) and **E26** (angle type forward).
6. **Up air and back air** (the kill moves). Up air: five 1% loop hits every 3 frames (autolink on airborne targets, 105-degree set
   knockback on grounded ones), the 5% last hit at 88 degrees BK 50 KS 152; landing lag 14. Back air: 12%, 361 backward, BK 30, KS 95,
   frames 8-10, landing lag 10, autocancel 1-2 / 35+, head and torso invincible on 7 and the left arm 7-10. **Engine**: data, plus **E26**
   (every up air box is angle type 4, backward, including those in front) and **E19** (partial invincibility).

Lesser signature points that must also be on [R]: the down throw combo throw (73 degrees, BK 90, KS 30) and its 8-frame hitlag on frame
24; the back throw KO throw (361, BK 65, KS 70); the forward and down smash windboxes; the up smash's tall pillar; the dash attack's
partial invincibility; the jab's rapid-jab entry rule.

## How the moves flow (traced)

Normals and aerials as in the Mario and Falcon audits (`ground` / `airborne` → `start_ground_attack` / `start_air_attack` →
`begin_attack` → `fighter::attack` each frame → `combat::resolve_hits`, landings through `land_out_of_move`). Specials:
`start_special` (`fighter.rs`) picks the slot from the stick (side special faces the stick's way; an up special held up and back comes
out reversed), sets `invuln` from `Move::intangible`, scales the aerial start momentum by `start_speed`. In `fighter::attack`: a script
`set_vel` or data motion moves the fighter through `move_by` (walls stop the horizontal speed, ceilings the vertical; a grounded motion
never sinks into the ground and leaves it past the edge; touching down ends the move into its landing lag unless `land_continue`); a
special in the air without a motion uses `move_air_physics` (E11's `air_gravity` / `air_max_fall` / `air_friction` / `air_drift`); a
grounded move without a motion slides on friction through `slide_on_platform`, which **ends the move into `Airborne` when it slides off
the platform** (the reference's special-move ledge-cancel, for free); at the end a grounded fighter goes `Idle`, an airborne one
`Helpless` if `helpless_after`, else `Airborne`. Projectiles: a script `spawn` / the move's spec queues one shot per fighter per frame
(`spawn_projectiles`); each frame `update_projectiles` runs its script, applies gravity, removes it at walls / blast zones / end of life,
then **reflector detectors** (`reflector_detects`), **reflectors** (`reflector_touching`), **clanks**, then the first fighter it overlaps
gets `apply_hit` and the projectile is **removed**. `apply_hit` first offers the hit to `catch_with_counter` (for melee **and**
projectile hits). Every projectile hit's launch direction is the sign of its velocity (`facing = vel.x < 0 ? -1 : 1`). A melee hit's
direction is the sign of the hitbox's authored `x` (behind the attacker → backward).

## Normals: what exists

| Needed | Exists? | Decision |
| --- | --- | --- |
| Jab 1 → rapid jab when attack was **pressed and released twice since jab 1 started** (window 10-24), or **held on hit** at 13; no jab 2 / 3 | partly: the Needles' scripted chain and rapid-jab loop exist; a script cannot count presses made in hitlag (jab 1's x1.6 hitlag) | reuse Falcon's planned **E16 `presses`** register (attack presses and releases since the chain began, from the input history); jab 1's script: `goto` the rapid jab on 10-24 when `presses` ≥ 2, or on 13 when `attack` is held and `hit` |
| Rapid jab loop (7 hits a cycle every 3 frames, loop 1-2 / 23-24) and finisher on release after one full cycle | yes (Needles pattern: `goto` loop, `rehit`, `attack` held) | data + script; 2 `ext` slots |
| **Shieldstun x7** on the rapid jab | no | Falcon's **E24** `Hitbox::shieldstun` |
| No-clank melee boxes (`clang=f` on the rapid jab, dash attack, smashes, down air, pummel, ledge / get-up attacks) | no | Falcon's **E3c** `Hitbox::clanks` |
| Hitstun modifiers: jab +4 | yes (`Hitbox::hitstun_add: u8`) | data |
| **Negative hitstun** (neutral air late loop -13) | **no**: `hitstun_add` is unsigned | **E27 (new, small)**: make it signed (`i8`, content accepts a negative value; hitstun floored at 0 [E: the reference floors it]); default 0 |
| Set knockback with set weight (forward tilt's three phases, the loop hits, the barrier windbox) | yes (`fixed_kb`, `set_weight`) | data |
| Multi-hit loops: up tilt every 3, neutral air every 4, up air every 3, each with a last hit | yes (`Move::rehit` + `rehit_groups`, last hit in its own group) | data. Note: our rehit is a rhythm from the first active frame, the reference's is per target from its first hit; identical for a target inside from the start, which is how the tests run [D] |
| Autolink loops (neutral air, up tilt, up air on airborne targets) | yes (367, `fighter::autolink`) | data |
| Grounded-only / aerial-only boxes (down tilt, neutral air, up air, down air, grabs) | yes (`Hitbox::targets`) | data |
| **Angle types 3 and 4** on boxes that reach both sides (neutral air's last hit "always forward" around the staff, every up air box backward including the ones in front, down air's grounded box backward from under the body) | **no**: the direction is the sign of the box's authored `x` | **E26 (new, small)**: `Hitbox::angle_type` = `BY_BOX` (today, default), `FORWARD`, `BACKWARD`, `BY_POSITION` (from where the victim is relative to the hitbox's centre: the reference's type 0). Read in `resolve_hits` (melee) and `update_projectiles` (projectiles, with E3a's projectile facing) |
| Windboxes (forward smash early / late, down smash, barrier) | yes (**E2** `Hitbox::push`, flinchless, no hitlag) | data; positions on `top` [D] |
| Partial invincibility: forward tilt's arm 14-35; dash attack's head and torso 5-6, left arm 5-16; back air's head and torso 7, arm 7-10 | no (whole body only) | Falcon's **E19** (per-hurtbox intangibility, owner decision 13). Limit: the reference's "invincible" (hit connects, no damage) and "intangible" (passes through) are one thing in the engine; a shared limit, noted |
| Up smash's tall pillar (three columns to 65 reference units up) | yes (many circles; no hitbox count limit) | data, about 30 circles [D] |
| Plain throws (no pre-hit), back throw turning round on 20 | yes (`HIT_THROW`, `turns_around`) | data |
| **Down throw's 8 frames of hitlag on frame 24** (no damage) | not as such | a 0% `HIT_PUMMEL` row on 24 with a hitlag multiplier giving 8 frames from the formula (`floor(6 x 1.34) = 8`) [D]; check that a 0% pummel freezes both and does nothing else; else a tiny `Move` pause field |
| Pummel on frame 1 | partly (one frame later, house rule) | data [D] |
| Ledge attack (shared intangibility decay), get-up attacks (face up / face down / trip) | yes (one get-up slot) | the face-up one; the others are the known limit (Mario's question 6) |
| Body: walk / run accelerations, 10-frame initial dash, traction, jumps, air numbers, dodges, heavy landing 6 | yes (`FighterParams`) | `FighterParams::seer`, a taller `ecb_height` [E]; no wall jump (`wall_jump` 0) but the wall-tech jump 1.3 / 2.2 (since sim 64 the same fields) |
| Jab lock (Autoreticle shots "can lock" a lying fighter) | not modelled for anyone (a weak hit on a fighter in `Knockdown` launches it like any hit) | a shared limit, not Palutena's: listed |

## Specials: what exists, and the exact shape of each needed extension

### Autoreticle: E16 target lock

* What exists: script `spawn(fx, up, vx, vy)` (one shot a frame; three shots six frames apart fit), the projectile pool (16), a shot
  removed by its first hit (the reference's shots do not pierce), reflectable shots, `start_speed` 60 for the x0.6 horizontal speed.
  Projectile scripts already see "the nearest other fighter" (`combat::surroundings`), but a **fighter** script sees no opponent, and
  nothing checks range, facing, intangibility or line of sight.
* **E16 addition: a target lock (exact shape).** A fighter function `lock(range)` that, when nothing is locked yet this move, looks
  for the nearest other fighter (lowest index on a tie) that is active with stocks left, **not intangible** (`invuln`, respawn,
  ledge), **in front** (`(target.x - x) x facing > 0` [E: gap 2]), whose chest is within `range` of the user's chest, and with **no
  solid platform crossing the segment** between the two chests (pass-through platforms: a flag argument, gap 2); on success it stores
  the target's chest position in hashed `Fighter::lock` (`Vec2`) and `Fighter::locked` (bool) and returns 1. Registers `lock_x`,
  `lock_y` (absolute) and `locked`. Cleared by `begin_attack`. A new `collision::segment_blocked(stage, a, b, pass_through)` (integer
  segment-against-rectangle test) does the line of sight. "Chest" is the hurtbox centre height of each body [E]. Whitelist entries +
  `scripted_moves.rs` tests (finds the nearest in front; ignores one behind, one out of range, an intangible one, one behind a block;
  the lock stays where it was when the target moves) + doc lines; checksum cases `lock` / `locked` in `state.rs`.
* The script: on frames 7-14 (move 6-13) call `lock(15)` (120 reference units [D]); on move frames 22, 28, 34 (the reference's 23 /
  29 / 35, so the shots hit from 24 / 30 / 36: the fireball's rule) if `locked`, `spawn` from the staff [E] with the velocity
  `0.75 x (lock - muzzle) / |lock - muzzle|` [D: straight at the fixed point, gap 1]; the projectile spec: 3.5%, 361, BK 40, KS 41, r 3,
  shield 51%, life 25. With no lock the move just plays out (FAF 68). Aerial: `start_speed` 60; the descent stall waits for gap 3
  (an `air_gravity` / `air_max_fall` pair if footage gives numbers).
* No `ext` slot (grounded and aerial share the timing; a landing mid-move: E11 `land_continue` until the FAF [E: gap 4]).

### Explosive Flame: E3a, E26, E3c (projectiles), E16 `flick`

* What exists: a script can remember where Palutena stood on frame 1 (`x`, `y` into two of its four `vars`) and spawn later with an
  offset that cancels the movement since (`spawn(facing x (x0 - x) + 5.625, y0 - y + h, 0, 0)`, `h` the point's height [E: gap 5]), so
  **the placement is exact without new code**. `start_speed` 80 for the x0.8 horizontal speed. A projectile with speed 0 stays put
  (gravity 0, no arc).
* **E16 addition: `flick`** (1 when the special was started with a smash-style flick of the stick toward its direction, by the same
  test as a dash flick, `Fighter::flick_x`, within `FLICK_BUFFER` of the special press). Recorded at `start_special` into a hashed
  `Fighter::special_flick` so the script can read it on any frame. Whitelist + test + doc line + checksum case.
* **E3a, multi-hit stationary projectile (exact shape).**
  * `ProjectileSpawn::pierce: bool` (default false): the projectile is not removed by a hit; it keeps going to the end of its life.
  * `ProjectileSpawn::rehit: u8` (default 0): a target already hit by a group can be hit by it again `rehit` frames later.
  * More hit phases by age: `phases` grows from 2 to **8** entries (`[Option<(u8, Hitbox)>; 8]`), and a phase whose hitbox has radius
    0 is "no hit" (the frame 56 gap before the last hit). The flame needs seven radius steps and the last hit: ages 1 / 5 / 9 / 13 / 17
    / 21 / 25 (r 6 / 8 / 9 / 11 / 12 / 14 / 15 tenths, group 0, 1.5%, 160, BK 50, KS 100), age 29 none, age 30 (group 1, 5.5%, 84, BK 60,
    KS 141, r 19, H 1.5); spawned on move frame 26 (the reference's 27, so its first hit is on 28, age 1), life 31 (a projectile is
    removed when its age reaches its life, before it hits, so the last hit at age 30 needs 31) [D].
  * Hashed `Projectile::hit_mask: u32` (the same `hit_bit(target, group)` layout as `Fighter::hit_mask`) and
    `Projectile::hit_age: [u8; MAX_FIGHTERS]` (the age of the last hit on each target). A target is hit by a box of group `g` if its bit
    is clear, or if `rehit > 0` and `age - hit_age >= rehit`. This is the reference's per-target rehit, exactly.
  * Hashed `Projectile::facing: i8` (the owner's facing at spawn), used for its hits' direction instead of the sign of its velocity
    when the velocity is zero, and for E26's forward / backward types.
  * Content: `pierce`, `rehit` and up to eight `phase` entries on a projectile spec; defaults leave every existing projectile
    unchanged. Tests in `multihit.rs` / a new `projectiles.rs`: a piercing projectile hits two fighters; a rehit of 5 hits a target
    standing inside six times on the loop, and once more on the last hit; a target walking in on age 3 is hit on 3, 8, 13; the radius
    grows on its ages; checksum cases for the three new fields.
* **E26 for projectiles**: the loop's angle 160 is type 0 (by position): with `BY_POSITION` the victim right of the flame's centre is
  sent at 160 toward the left (back into the centre) and the one on the left mirrored: the trapping the reference is known for. Today a
  speed-0 projectile sends everyone to the right.
* **E3c for projectiles** (Cloud's planned transcendent flag): the flame and its last hit never clank (`clang=f`); today an attack
  overlapping it could clank it away (`clank_projectile`). Autoreticle's shots keep clanking (no `clang=f`).
* Reflectable: a reflector touching the flame takes it over (our reflector reverses a zero velocity: it stays) [R: `reflectable=t`].
* Cancelling: spawning on frame 27 means a hit on Palutena before then leaves no flame [E: gap 7].
* No `ext` slot.

### Warp: everything exists, one small E11 addition

* The up special as **one move** (no `ext` slot): `air_gravity` 0.03 and `air_max_fall` 0.75 for the startup (E11, exists); a script
  that on move frame 15 (the reference's 16) reads the stick (neutral → straight up [E]; magnitude ignored [E]), calls
  `intangible(17)`, and `set_vel`s 0.4375 world units a frame along that direction for 17 frames; on move frame 32 (the reference's 33)
  sets half of it once; FAF 68, `helpless_after` (it applies only when airborne at the end: a grounded end goes `Idle`, which is the
  reference's grounded reappearance), `landing_lag` 22 and `fall_landing_lag` 22 (E11, exists). The view hides the fighter while
  intangible in this move (presentation: the art agent's, as for Flash Step).
* **Edge-cancel exists**: after the travel the grounded reappearance has no motion, slides on friction, and `slide_on_platform` ends
  the move into `Airborne` (not helpless) when it slides off a ledge.
* **Travel against the stage exists in the shape the reference most likely has** [E, gap 9]: a motion that touches the ground
  continues along it with `land_continue` (it cannot sink in), leaves at the edge, and stops at walls and ceilings. **E11 addition**:
  `land_continue` must hold only for the travel (to the reference's frame 32) and not for the aerial reappearance (landing then costs
  the 22 frames): turn the bool into a frame, `land_continue_to: u8` (0 = never, 255 = the whole move: today's `true`), migrated from the
  bool. Without it the move needs two `ext` slots and a `goto` at 33 (which costs one frame of movement: the known `goto` limit).
* Ledge windows: E11's `ledge_front_from` / `ledge_back_from` exist; the numbers wait for gap 8.
* The 2.7 horizontal cap after the teleport is above the 1.75 the reappearance carries, so it only matters if something adds speed [D];
  not modelled unless footage shows otherwise.
* Air control in the reappearance and the special fall: the shared helpless drift unless gap 10 gives numbers (`fall_drift` exists).

### Counter / Reflect Barrier: E10c and E25

**Can the existing counter and reflector sit on one move?** Yes as data: `Move::counter` and `Move::reflector` are independent
`Option` fields. But as built they do not give the reference's behaviour:

1. `catch_with_counter` is called from `apply_hit` for **every** hit, projectiles included, so a projectile that reaches the hurtbox is
   **countered** (the answer swings at its owner, wherever that is). The reference's counter never answers a projectile: the search box
   sends it to the reflection.
2. The reflector detector (E10b, `reflector_detects`) only skips the **same** move ahead to the reflector's first frame. The reference
   starts a **different** move (the reflection, its own 35-frame FAF and animations).
3. `Reflector` is a field attached to the fighter's current move: it ends when the move ends. The reference's barrier is a separate
   object that **outlives the reflection's FAF by ten frames** (lifetime 45) and stays where it was made.
4. `Counter` has a minimum damage but **no maximum** (the reference caps at 50%), and catches on the hurtbox, not on the reference's
   "counter bubble" (r 11.5 around the chest). Our hurtbox may already cover that bubble; check before adding anything.
5. A shot that touches the detector and the hurtbox on the same frame: `update_projectiles` runs the detector before the hit, so the
   switch happens first, and the reflection's body reflector (frames 1-4, move 0-3) must already be up on the frame the move starts.
   Needs a test (as for the cape's detector).

**E10c (counter / reflector routing), the exact shape**:

* `Counter::melee_only: bool` (default false: today's): projectile hits are not caught (they go on to the normal hit; with the search
  box in front of them they almost never get there).
* `Counter::max_damage: Fx` (default 0 = no cap): the answer's damage is clamped to it.
* `Reflector::detect_then: Option<u8>` (default none: today's skip-ahead): a projectile touching the detector inside its window starts
  that `ext` move at frame 0 (with `begin_attack`, so its own intangibility and reflector apply) instead of skipping ahead. The
  down special then carries `counter` (7-34, then the counterattack slot, 130%, min 10, max 50, melee only) and a `reflector` with only a
  detector (7-34, r 15 tenths at 2 forward, 12.5 up [D]) and `detect_then` the reflection slot; its field window is empty (start > end).
* Optional, only if the hurtbox check in point 4 shows a difference: `Counter::bubble: Option<(Fx, Fx, Fx)>` (x, y, radius), a hit
  touching it is caught even where it misses the hurtbox.
* Tests in `kits.rs` (or the new fighter's file): a melee hit in the window is countered for 1.3x, at least 10, at most 50; a
  projectile in the window is reflected by the new move and not countered; a projectile out of the window hits; the counter and the
  reflection never both happen.

**E25 (new): a reflector left in the stage.** The barrier as a projectile-pool entry that reflects:

* `ProjectileSpawn::reflects: Option<Reflector>` (default none). Such a projectile is stationary (speed 0) and, while alive, any
  **other owner's** projectile touching it is reflected with its numbers (x1.4 damage, x1.25 speed, x1 life, limit 50) exactly as
  `reflector_touching` does for a fighter's reflector, the new owner being the barrier's owner. In `update_projectiles` the reflector
  search covers fighters' reflectors (today) and live reflecting projectiles.
* A tall shape: `Reflector::half_height: Fx` (default 0 = a circle): a vertical capsule from `y - half_height` to `y + half_height` (the
  barrier is 19 reference units tall and 3.8 thick: half height 1.19 world units, radius 0.48 [D]). Usable by fighters' reflectors too.
* Its own hitbox is the windbox (a push, set weight, FKV 50, hitting on ages 1-10 via E3a's phases: a later radius-0 phase ends it) and
  it is `pierce` (E3a) so the push does not remove it.
* Spawned by the reflection move on its frame 0 at (11, 9.5) reference units ([D]: 1.375, 1.19 world units), life 45. It stays when
  the reflection ends at FAF 35, and when Palutena is hit [E: gap 13].
* Content: a `reflects { ... }` block on a projectile spec; hashed through the spec (content) and the projectile's existing fields (no
  new state beyond E3a's). Test: a shot reflected by a barrier spawned by a move that has already ended; a shot from its own owner
  passes through; the push moves a fighter without hitstun.

**Air physics of the stance** (E11, exists, plus one addition): `start_speed` 33, `air_friction` 0.0025, `air_gravity` 0.04,
`air_max_fall` 1.2; **E11 addition: `air_max_rise: Option<Fx>`** (0.8: the upward speed is held at or under it). The counterattack and
the reflection have their own aerial animations (79 / 59) but the same FAF as the grounded ones: one move each, `land_continue` [E: gap
4].

**Slots**: the counterattack and the reflection: two `ext` slots.

## Slots

| Use | Slots |
| --- | --- |
| Rapid jab loop, rapid jab finisher | 2 |
| Counterattack, reflection | 2 |
| Autoreticle, Explosive Flame, Warp | 0 (Warp: 2 without E11's `land_continue_to`) |
| **Total** | **4** (6 without the E11 addition) |

It fits in today's ten `ext` slots; it does not depend on E17 (Falcon's 16 slots, owner decision 11).

## Extension summary

| Id | Status | Palutena needs | Shape |
| --- | --- | --- | --- |
| E2 | built (sim 61) | windboxes (forward / down smash, barrier) | as built |
| E3a | not built (planned) | **yes**: the flame, the barrier | `pierce`, `rehit`, 8 phases (radius 0 = none), `Projectile::hit_mask` / `hit_age` / `facing` (above) |
| E3c | not built (Falcon: melee `Hitbox::clanks`; Cloud: projectiles) | **yes**: no-clank melee boxes and the flame | Falcon's `Hitbox::clanks`, also read for projectiles in `clank_projectile` |
| E10c | **new (extends E10 / E10b)** | **yes**: the down special | `Counter::melee_only`, `Counter::max_damage`, `Reflector::detect_then`, optional `Counter::bubble` |
| E11 | built (sim 58), Falcon adds four fields | **yes**: Warp, the stance's air physics | as built, plus `land_continue_to` (a frame instead of a bool) and `air_max_rise` |
| E16 | partly built (air flags); Falcon adds `presses`, `wall` | **yes**: `presses` (the rapid jab), **`lock(range)`** with `lock_x` / `lock_y` / `locked`, **`flick`** | whitelist entries + tests + doc lines; hashed `Fighter::lock`, `locked`, `special_flick`; `collision::segment_blocked` |
| E19 | Falcon builds it (owner decision 13) | forward tilt arm, dash attack, back air | as Falcon builds it |
| E20 | not built | **no**: nothing in the kit trips (no `trip` field on any hitbox; the roster note "the shots can trip" was a misreading of "the shots can lock") | |
| E24 | Falcon builds it | rapid jab shieldstun x7 | `Hitbox::shieldstun` |
| **E25** | **new** | **yes**: the Reflect Barrier as an object | `ProjectileSpawn::reflects`, `Reflector::half_height` |
| **E26** | **new, small** | **yes**: neutral air last hit, up air, down air, the flame's loop | `Hitbox::angle_type` (`BY_BOX` default, `FORWARD`, `BACKWARD`, `BY_POSITION`), melee and projectiles |
| **E27** | **new, small** | **yes**: neutral air late loop -13 | signed `Hitbox::hitstun_add` |

Not needed by Palutena: E1, E3b, E3d, E4, E5, E6, E7, E8, E9, E10 (reversing hits), E12, E13, E14, E15, E17, E20, E21, E22, E23.

Every new field defaults to today's behaviour, is hashed (content through `Weapon::hash_into`, state through `GameState::hash_into` with
a checksum case each), appears in `docs/CONTENT.md`, and gets a test in its topic file; no existing move uses them, so the other
fighters stay unchanged (check `base.pfc` and the existing kit tests). `land_continue_to` replaces a content field's type: a schema bump
with a migration (`true` → 255, `false` → 0), unless it is added as a new field beside the bool. One sim version bump per phase.

Shared limits this port exposes but does not fix (listed, not built): jab lock; invincibility vs intangibility; the per-target rehit
for **melee** (E3a gives it to projectiles only); one get-up attack slot; the throws' interrupted rows and thrower invincibility.

## Suggested phases

1. **Body and every normal** (`FighterParams::seer`, jab and rapid jab, tilts, dash attack, smashes with windboxes, the five aerials,
   grabs, pummel, throws, ledge and get-up attacks, registration). Engine: E26, E27 (both small, new), and Falcon's E3c (melee), E19,
   E24 and `presses` if they have landed (else their moves carry the listed limit until they do). Specials as placeholders.
2. **Explosive Flame and Autoreticle**: E3a, E3c for projectiles, E16 `flick` and `lock` with `segment_blocked`, E26 for projectiles.
3. **Warp and Counter / Reflect Barrier**: E11's `land_continue_to` and `air_max_rise`, E10c, E25; then the re-check of every iconic
   move in the hitbox view and against footage for the gaps.

## Questions for the owner

1. **Footage**: the open behaviours are listed in `REFERENCE_DATA.md` ("Gaps", 13 items). The most important for 1:1: Warp's ledge
   windows and its travel into the ground, walls and platforms (gaps 8, 9); how Autoreticle's shots fly and its "in front" region
   (gaps 1, 2); the counter's freeze before the counterattack (conflict 3). Can you record them (training mode, frame-stepped,
   hitbox view)? Without them these are [E].
2. **E25** (a reflector left in the stage, built on the projectile pool) is the one structural addition: build it, or keep the barrier
   on the reflection move (which drops the ten frames after FAF 35 and the "stays when she is hit" behaviour)?
3. **E26** changes nothing for existing moves by default, but several existing moves are reference type 3 / 4 boxes approximated by
   the box's side (back airs, the Embers' and Falcon's). Apply it only to this port, or re-check the others' angle types too (a separate
   change)?
4. **Counterattack frames** (conflict 2): 5-7 (ssbwiki.com, dragdown) or 4-6 (UFD, kuroganehammer.com)? Footage settles it.
5. **Heavy landing lag** 6 (parameter file `landing_heavy_frame` 7) vs kuroganehammer.com's "hard landing lag 4" (version 5.0.0,
   unclear whether it counts frames of lag or the FAF): keep the parameter file, as every earlier port did?
6. **Names**: Beacon / seer / Seer and the blurb above, or other names?
