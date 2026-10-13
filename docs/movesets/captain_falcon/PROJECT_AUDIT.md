# Project audit for the fifth moveset (reference: the reference game's Captain Falcon)

What the engine needs to reproduce the reference game's Captain Falcon 1:1 (version 13.0.1), traced through the code at sim version 57
(commit `4515dc8`), counting the extensions that Mario's phase 2 is building in parallel as they are planned in
`docs/movesets/mario/IMPLEMENTATION_REPORT.md` (E11, E2, E3b, E10, E16, E15). Nothing is built yet: this is research only. The data is
in `REFERENCE_DATA.md`; the engine inventory is `docs/movesets/SHARED_MECHANICS.md`; the extension ids are those of
`docs/movesets/ROSTER_STATUS.md`.

## Proposed names (not registered)

| Thing | Name | Checked |
| --- | --- | --- |
| Class / weapon | **Gauntlets** | not on `content/blocklist.txt` (empty apart from comments) and contains none of the `TRADEMARKS` terms in `sim-content/src/policy.rs` (note that list blocks the substring "falco", so no name built on the reference's bird can pass) |
| Body | **sprinter** | same |
| Built-in fighter | **Sprinter** | same |
| Blurb | "Blazing speed and heavy hands: a dashing bruiser with a one-frame flying knee, a leaping grab that explodes and a wound-up punch that ends stocks." | original wording |

Class index 4 when registered (`CLASSES = 5`), with the HUD emblem left for the art agent.

## Iconic moves: what must match exactly

The owner's 1:1 rule: these are the moves players know by feel. A port is not "verified" while any of them is approximate.

1. **Falcon Punch** (neutral special). Startup 53, active 53-57, FAF 104; the **turnaround** read on frame 15 giving the reversed punch
   (turn on 35, hit 62-66, FAF 118, 28% BK 85 KS 65 grounded, 25% aerial); the grounded / aerial damage and the aerial's x1.15 hitboxes;
   the aerial movement (0.2 up on start, x0.92 a frame for 19 frames, the 35-degree stick boost on 51 / 60); no clank. **Engine**:
   the frames, hit data, turnaround and aerial drift exist (data plus a script: `frame`, `stick_x`, `turn`, `goto`, `add_vel`); the
   no-clank flag does not (E3c, extended to melee hitboxes); the boost's speed and the landing behaviour are not published (footage).
2. **The knee** (forward air). The **1-frame sweetspot on frame 14** (22%, 32 degrees, BK 30, KS 81, radius 3.75, electric, H 1.2), the
   6% hip sourspot on the same frame, the 3% late hit 15-30, autocancel 1-4 / 42+, landing lag 18, FAF 46. **Engine**: exists as data
   (rows, priority, `EFFECT_ELECTRIC`, `ref_move`). The late hit's 20% trip on grounded targets needs E20 (trip); the sweetspot's
   position on the knee is [E] by nature and must be checked in the hitbox view.
3. **Falcon Dive** (up special). Grab boxes on 14 (early) and 15-30 (late) that catch **airborne and grounded** targets, through shields and
   in hitstun; the catch (5% explosion on 2-3 that also launches bystanders, 13% throw at 50 degrees on 17, FAF 40); **free, not helpless,
   after a catch** (and able to Falcon Dive again); helpless after a miss with air speed x0.7 and landing lag 24; ledge grab from the
   front from 20 and from behind from 37 (and from 61 of the catch); fast fall from 50; B-reverse only on frame 14; the air acceleration
   x6 ("Tokyo drift") and x0.95 air speed; gravity 0.103 after the rise. **Engine**: needs **E6** (command grab) and **E11** (with three
   additions, below); the rise is root motion (footage).
4. **Raptor Boost** (side special). The search (grounded 10-25 with grounded / aerial target split, aerial 19-35), the switch to the hit
   on contact (shields included), **10% damage-based armor** on the grounded hit, the grounded uppercut (85 degrees, +3 hitstun), the
   aerial overhand's meteor on aerial targets and 60 degrees on grounded ones, the aerial hit's set momentum (1.5 back, 2.5 up on frame 6)
   and no landing before frame 8, **helpless only after an aerial miss** (x0.7, landing lag 22), ledge grabs from 26. **Engine**: needs
   **E1** (damage-threshold armor), **E21** (search boxes, new), **E11**.
5. **Falcon Kick** (down special). Ground slide with three phases (15 / 11 / 7) and FAF 69, or 66 if it goes off an edge; the wall crash
   on 14-36; the aerial dive with three phases and FAF 57; the 9% landing hit on both sides with landing lag 35; the on-hit slowdown.
   **Engine**: phases, landing hit (`landing_move`) and leaving the ground at an edge exist; the off-edge FAF needs **E22** (new), the
   wall crash needs a wall reaction (E16 addition); the slide and dive distances are root motion (footage).

Lesser signature points that must also be on [R]: the jab's knee finisher ("the Gentleman") and the rapid jab's entry rule; the up
air's late 30-degree kick; the down air's meteor sweetspot below the feet; the down throw's combo angle (68, BK 65, KS 70).

## How the moves flow (traced)

Normals: as in the Mario audit (`ground` → `start_ground_attack` → `begin_attack` → `fighter::attack` → `combat::resolve_hits`;
aerials via `airborne` → `start_air_attack`, landings through `land_out_of_move`). Specials: `start_special` picks the slot from the stick
(side special faces the stick's way; an up special held up and back comes out reversed), sets `invuln` from `Move::intangible`, then
`fighter::attack` each frame: the global up-special turn window (`Ruleset::special_turn_window`, frames 0-3) and the B-reverse window
(`special_reverse_window`, frames 0-4, once) apply to every up / neutral / side special; a special in the air uses the body's air physics
(`air_move`) unless a motion or script sets the velocity; landing goes through `land_out_of_move` (the move's `landing_lag`, or its
`landing_move`); the move's end goes to `Idle`, `Helpless` (`helpless_after`) or `Airborne`; `grabs_ledge` checks for a ledge every
frame of the move. Grabs: `HIT_GRAB` boxes reach only fighters for which `grab::grabbable` is true (**grounded**, not in hitstun, not on a
ledge, not already held); `grab::connect` puts the holder in `Grabbing` (which cannot move) and pins the victim `Ruleset::grab_distance`
in front, **snapped to the holder's platform**; the hold clock and mash-out run only while the holder is in `Grabbing`. The `hit` script
register is `hit_mask != 0`, which is also set by a hit on a shield.

## Normals: what exists

| Needed | Exists? | Decision |
| --- | --- | --- |
| Jab 1 → jab 2 → jab 3 on the continuable frames (5, 8), held repeat from 7 | yes: `Move::next_from` (Mario phase 1), a three-line script for the held repeat | data |
| Jab 2 branching to the rapid jab **only when attack was pressed 3 times and released twice since jab 1 started**, else to jab 3 | **no**: `next_from` has one `next`; a script cannot see presses made in hitlag (jab 1's 1.5% x1.6 hitlag) and has no press count | **E16 addition**: a `presses` register (attack presses and releases since the current chain began, counted by the input history so hitlag presses count), and jab 2 as a script that picks `goto` rapid jab or jab 3. Alternative: `Move::rapid_next { slot, presses, releases, window }` as data |
| Rapid jab loop (eight hits a cycle), finisher on release after one full cycle | yes: the Needles pattern (`goto` loop, `rehit`, `attack` held) | data + script |
| Hitstun modifiers (+3 jab 2, +2 rapid jab, +3 Raptor Boost) | yes: `Hitbox::hitstun_add` | data |
| **Shieldstun multipliers** (rapid jab x9, dash attack x1.5) | **no** (no per-hitbox shieldstun) | **E24 (new, small)**: `Hitbox::shieldstun: u16` percent (default 100) applied in `combat::block` |
| Angled forward tilt (+1% when angled) and forward smash (angle and growth by aim) | yes: `Hitbox::aim`, `Fighter::aim` | data |
| Set weight (neutral air hit 1) | yes: `Hitbox::set_weight` | data |
| Grounded-only / aerial-only boxes (up tilt, neutral air, grabs, Raptor Boost, Falcon Dive) | yes: `Hitbox::targets` | data |
| Meteors (up tilt 285, down air 270), the electric knee | yes (angles, `EFFECT_ELECTRIC`) | data |
| Up tilt lower-leg intangibility; up smash right / left lower-leg intangibility | **no** (whole body only) | limit until **E19** (owner decision) |
| Down tilt trip 50% | no trips | nothing lost: at 25 degrees it can only trip on slopes; our stages are flat [D] |
| Forward air late hit trip 20% | **no** | needs **E20** for 1:1 (grounded targets at low knockback only) |
| Hit-then-throw throws (forward, back, up) and the plain down throw | yes: `throw_after_hit` (a `HIT_PUMMEL` hit then `HIT_THROW`) | data; the back throw's release angle 135 is "behind and up" on the original facing [D] |
| Pummel hitting on frame 1 | partly: the pummel cannot hit on its first frame | one frame later [D], as for Mario |
| Ledge attack (intangibility decay), get-up attacks | yes (one get-up slot) | data; face-down and trip get-up attacks have no slots (Mario's question 6) |
| No-clank melee hitboxes (`clang=f`: Falcon Punch, Raptor Boost, ledge / get-up attacks, throw hits) | **no**: every grounded attack can clank | **E3c extended**: `Hitbox::clanks: bool` (default true), read in `combat::clash` |
| Jostling and the moves that turn it off | **no jostling at all** in the engine | a shared limit, not Falcon's; noted |
| Body: walk / run accelerations, 15-frame initial dash, traction, jumps, air numbers, dodges, heavy landing 3 | yes (`FighterParams`) | `FighterParams::sprinter`, with a taller `ecb_height` [E] |
| Wall jump (passive 1.4 / 3.1, enable 0.2) | **no** | **E15** (Mario phase 2 builds it; Falcon only adds his numbers) |

## Specials: what exists, and the exact shape of each needed extension

### Falcon Punch: data and a script, plus E3c

Four versions (grounded, grounded reversed, aerial, aerial reversed) differ in damage, knockback, radii and timing. Built as:
`NSpecial` = grounded; its script on frame 0 does `goto` the aerial slot when not `grounded`, and on frame 14 (reference 15) `turn`s and
`goto`s the reversed slot when the stick is held back past the threshold. The reversed slots are authored from the reference's frame 15
on (hit 62-66 becomes their 48-52, FAF 118 becomes 104 [D]), so the shared frames 1-14 are not repeated. The aerial script applies 0.2
up on frame 0, multiplies the forward speed by 0.92 a frame (`add_vel`) for 19 frames, and adds the boost on 51 / 60 at 35 degrees by the
stick (speed unknown: footage). Exists today except **E3c** (no clank) and the gap items. Slots: three `ext` (see "Slots" below).

### Raptor Boost: E1, E21, E11

* **E1 (damage-threshold armor), the exact shape.** `Move::armor: Option<Armor>` with
  `Armor { start: u8, end: u8, kind: u8, threshold: Fx }`, kinds `ARMOR_SUPER` (never flinches), `ARMOR_DAMAGE` (flinches only when the
  hit's damage, after `Ruleset::damage_mult`, exceeds `threshold x damage_mult`: the reference's "10% (12% in one-on-one)"), and
  `ARMOR_KNOCKBACK` (Yoshi later: subtract `threshold` from the knockback, flinch with the rest). In `combat::apply_hit`, after the damage
  and hitlag are applied and before the hitstun / launch: if the victim is in `Attack` with `state_frame` inside the window and the hit
  qualifies, the damage and both fighters' hitlag stand, nothing else happens (no hitstun, no launch, no state change, the move goes on).
  Grabs are not stopped by armor (the reference's rule). Content field `armor { start end kind threshold }`, default none; hashed in
  `Weapon::hash_into`; test in `multihit.rs` (a 9% hit does not interrupt, an 11% hit does; a grab still catches). Only `ARMOR_DAMAGE` is
  needed now; the other two kinds can wait for Donkey Kong and Yoshi (but the field should carry the kind from the start so the format
  does not change twice). The window is 1-5 or 1-4 (conflict 2 in `REFERENCE_DATA.md`).
* **E21 (new): search boxes.** Raptor Boost's dash has 0% "inert" boxes that only detect, never clank, and switch the move on contact,
  shields included. Nothing in the engine does this (a 0% `HIT_NORMAL` box would still cause hitlag, hitstun and a hit-mask entry). Shape:
  a hitbox kind `HIT_SEARCH` and `Move::on_search: Option<u8>` (an `ext` slot). In `resolve_hits`, a search box overlapping a hurtbox
  (respecting `targets`, ignoring shields as an obstacle) does nothing to the target; the attacker starts `on_search` at frame 0 at the
  end of the frame (keeping its velocity, which the hit move's script then scales by 0.18 on the ground). Search boxes are skipped by
  clanks. Default: no move has one. Also usable for the Cape's early projectile detector (E10b's `detect_from`) if that is generalised
  the same way, and for later "on contact" moves. Test: a search box hits nobody, switches the move on contact with a standing, shielding
  or airborne fighter, and never clanks. Open: whether it ignores intangible targets (gap 6).
* **E11** for the aerial version: ledge grabs from frame 26 on both sides (`ledge_front_from` / `ledge_back_from` 25); the special fall
  after a miss with `fall_air_speed` 70 and `fall_landing_lag` 22; the hit move's "no landing and no ledge before frame 8" (needs the
  `lands_from` addition below; `ledge_*_from` 7 covers the ledge part).
* **Helpless only on a miss**: exists (the dash has `helpless_after`; the hit move, reached by the search, does not).
* The aerial hit's set momentum on frame 6 (1.5 back, 2.5 up, reference units a frame): a script `set_vel` [D].
* The dash speed and distance: root motion [gap]; the dash is a motion or `set_vel` fitted to a measurement.

### Falcon Dive: E6 and E11

* **E6 (command grabs of airborne targets), the exact shape.**
  * A hitbox flag `Hitbox::command: bool` (content `command`) on a `HIT_GRAB` box. A command grab box uses
    `grab::catchable(f)` instead of `grabbable(f)`: not intangible, not already held or holding through another pair, not
    `grab_immune`, **grounded or airborne, in hitstun or not, shielding or not**. (Ledge-hanging fighters: excluded until gap 5 is
    answered.)
  * `Move::catch_move: Option<u8>` (an `ext` slot). When a command grab connects, `grab::connect` does `begin_attack(holder,
    catch_move)` instead of `become_holder` (the holder stays in `Attack`, keeps its velocity and air physics, can be airborne); the victim
    becomes `Grabbed` with **no hold timer** (the clock already only runs while the holder is in `Grabbing`, so nothing else changes).
  * `Move::hold_at: (Fx, Fx)` on the catch move: where the held fighter is pinned, relative to the holder's feet and facing, used by
    `grab::pin` instead of `grab_distance`; while the holder is airborne the victim is **not** snapped to a platform. Default: today's pin.
  * The catch move is then ordinary data: hit 1 is a `HIT_PUMMEL` row (5% to the held fighter, no release) plus a `HIT_NORMAL` row on the
    same frames for bystanders (normal hitboxes in a move never touch the held fighter: exists), and a `HIT_THROW` row on 17 (13%, 50, BK
    85, KS 62). `grab::update`'s consistency check already accepts a holder in `Attack`. If the holder is hit during the catch, the pair
    drops (exists). The catch ends `Airborne`, not helpless (`helpless_after` false), so Falcon Dive can be used again: exists.
  * Grab parry (two grabs meeting) and armor do not apply to command grabs [R for armor; parry unknown].
  * Content: `command` on hitboxes, `catch_move` and `hold_at` on moves, defaults false / none / none; hashed; test file
    `grabs.rs` (an airborne fighter is caught by a command grab and not by a standing grab; a fighter in hitstun is caught; the holder
    stays airborne and the victim travels with it; the bystander hit and the throw; no mash-out).
  * Yoshi's Egg Lay reuses the same three fields, with an encased-victim state (E8) on top.
* **E11** (Mario phase 2's shape) gives: `ledge_front_from` 19 and `ledge_back_from` 36 (frames 20 / 37); on the catch move,
  `ledge_front_from` 60; `fall_air_speed` 70, `fall_landing_lag` 24; the `air` block for x0.95 air speed and the gravity 0.103. **Three
  additions to E11 are needed for Falcon** (they are small and general; worth folding in while Mario's E11 is being built, or as an E11b):
  1. `fast_fall_from: u8` (default 0: today a move in the air that uses the body's air physics, `air_move`, can fast fall on any frame
     once falling): Falcon Dive allows it only from frame 50.
  2. `air.accel_percent` and a frame range for the `air` block (`air: { from, to, gravity, max_fall, air_speed_percent,
     air_accel_percent, air_friction }`): Falcon Dive's x6 air acceleration before the grab comes out and gravity 0.103 only after the
     rise ("when detaching"); Falcon Punch's 19-frame x0.92 could also be data this way instead of a script.
  3. `turn_at: Option<(u8, u8)>` (frame and stick threshold) overriding `Ruleset::special_turn_window` for this move: Falcon Dive turns
     only on frame 14 at stick 0.25, never in frames 1-4 as the global rule would let it. Default none (today's global window).
  4. `lands_from: u8` (default 0): the move ignores landings before this frame (Raptor Boost's aerial hit, frame 8).
* The rise and the post-catch backward flip: root motion [gap]. Fitted motion rows (or `set_vel` script) once measured.

### Falcon Kick: data, plus E22 and a wall reaction

* Phases, landing hit (`landing_move` started by a non-autocancelled landing; its length is the landing lag 35), flame: exist as data.
  Ground vs air: `DownSpecial` = grounded, script `goto` the aerial slot when not `grounded` on frame 0.
* **Off an edge (FAF 69 → 66)**: today a ground dash past the edge leaves the ground and carries on with the same move, so the FAF
  stays 69. **E22 (new): a situation change that keeps the frame.** `Move::off_edge: Option<u8>` (and, for symmetry and for the aerial
  Falcon Punch if footage shows it, `Move::on_land: Option<u8>`): when the fighter leaves the ground (or lands) mid-move, the move switches
  to that slot **at the same frame** (state_frame, hit mask and charge kept). Default none (today's behaviour). Falcon Kick's off-edge
  slot is a copy with FAF 66. Test: a ground move with `off_edge` that goes past the edge ends on the other slot's FAF and keeps its hit
  mask.
* **Wall crash (14-36, 53-frame crash)**: needs to know the move ran into a wall. **E16 addition**: a `wall` register (1 when the last
  move step was stopped by a wall), then a script `goto` the crash slot. Or as data: `Move::on_wall: Option<(u8, u8, u8)>` (window and
  slot). Either is small; the register is more general.
* The on-hit slowdown (x0.8, "maximum speed decreases by 4") and the ending friction 0.12: a script reading `hit` and scaling the slide
  speed [D]; the "4" needs interpreting against footage.
* Slide and dive distances: root motion [gap].

## Slots

`MoveId` has ten `ext` slots. Falcon's kit, built as above, needs:

| Use | Slots |
| --- | --- |
| Rapid jab loop, rapid jab finisher | 2 |
| Falcon Punch grounded reversed, aerial, aerial reversed | 3 |
| Raptor Boost grounded hit, aerial dash, aerial hit | 3 |
| Falcon Dive catch | 1 |
| Falcon Kick aerial, landing hit, off-edge copy, wall crash | 4 |
| **Total** | **13** (ten available) |

Ways to fit it, for the owner to choose (question 1):

* **E17 (more move slots)**, which Cloud and Donkey Kong need anyway: a content schema change (more `ext` keys, a migration). Clean,
  but it is one of the "explain before building" architectural changes.
* **E23 (new, small): a hitbox filter on the attacker's situation** (`Hitbox::when: ANY | GROUNDED | AIRBORNE`, the attacker's, not the
  victim's), so the grounded and aerial Falcon Punch share one move (same frames: 53-57 / 104) and the reversed pair shares another:
  saves 2 slots. With E22 the Falcon Kick off-edge copy is still needed, so the total is **11**: still one over without E17.
* Dropping the wall crash or the off-edge FAF to [E] limits: not acceptable under the 1:1 rule without the owner's say.

Recommendation: E17 in its smallest form (16 `ext` slots, no other change) at the start of Falcon's specials phase, after Mario's phase 2
lands, so Cloud and Donkey Kong inherit it.

## Extension summary

| Id | Status | Falcon needs | Shape |
| --- | --- | --- | --- |
| E1 | not built | **yes**: damage-threshold armor (Raptor Boost) | `Move::armor` with a kind field from the start (above) |
| E3c | not built (planned for projectiles) | **yes**: no-clank melee boxes | `Hitbox::clanks` (default true), also for projectiles |
| E6 | not built | **yes** (Falcon Dive) | `Hitbox::command`, `grab::catchable`, `Move::catch_move`, `Move::hold_at` (above) |
| E11 | Mario phase 2, building | **yes** (Falcon Dive, aerial Raptor Boost) | Mario's shape plus `fast_fall_from`, a framed `air` block with `accel_percent`, `turn_at`, `lands_from` |
| E15 | Mario phase 2, building | **yes** (wall jump) | Mario's shape; Falcon's numbers 1.4 / 3.1 / 0.2 |
| E16 | Mario phase 2 builds the per-airtime flags | **yes**: `presses` (rapid jab entry) and `wall` registers | whitelist entries + `scripted_moves.rs` tests + doc lines |
| E17 | not built | **yes** (13 slots needed, 10 exist) | owner decision |
| E19 | not built | up tilt and up smash leg intangibility | owner decision (limit until then) |
| E20 | not built | forward air late hit trip (20%) | trip from the sim's RNG |
| **E21** | **new** | **yes**: Raptor Boost's search | `HIT_SEARCH` kind + `Move::on_search` |
| **E22** | **new** | **yes**: Falcon Kick off an edge; maybe aerial Falcon Punch landing | `Move::off_edge` / `Move::on_land`, switch keeping the frame |
| **E23** | **new, optional** | saves 2 slots | `Hitbox::when` (attacker grounded / airborne) |
| **E24** | **new, small** | **yes**: rapid jab x9 and dash attack x1.5 shieldstun | `Hitbox::shieldstun` percent |

Not needed by Falcon: E2, E3a, E3b, E3d, E4, E5, E7, E8, E9, E10, E12, E13, E14.

Every new field defaults to today's behaviour, is hashed, appears in `docs/CONTENT.md`, and gets a test in its topic file; no existing
move would use them, so the other fighters stay unchanged (check `base.pfc` and the existing kit tests). One sim version bump per phase.

## Suggested phases

1. **Body and every normal** (no new engine work except E24 and the `presses` register, both small): `FighterParams::sprinter`, the jab
   chain with the rapid jab entry rule, tilts, dash attack, smashes, aerials (the knee), grabs, pummel, throws, ledge and get-up attacks,
   registration. Specials as placeholders.
2. **Falcon Punch and Falcon Kick** (E3c, E22, the `wall` register; slots permitting).
3. **Raptor Boost and Falcon Dive** (E1, E21, E6, E11 additions; E17 if chosen), then wall jump (E15's numbers) and the re-check of every
   iconic move in the hitbox view.

## Questions for the owner

1. **Slots**: 13 `ext` slots needed, 10 exist. E17 (16 slots, a schema change) now, or E23 plus one dropped variant, or something else?
2. **Root motion**: Raptor Boost's dash, Falcon Dive's rise and post-catch flip, Falcon Kick's slide and dive and Falcon Punch's slide are
   not published. Can you record them (training mode, frame-stepped, flat stage, standing start and from a full hop), as asked for Mario's
   Super Jump Punch? Without them these moves can only be fitted [E], which the 1:1 rule does not allow for Falcon Dive.
3. **Footage checks** for the open behaviours: aerial Falcon Punch landing mid-move; aerial Raptor Boost and Falcon Kick reuse in one
   airtime; Falcon Dive catching near the ground and catching a ledge-hanging fighter; Raptor Boost's armor ending on frame 4 or 5.
4. **E19** (partial intangibility) for the up tilt's and up smash's legs: build it with Falcon, or leave it as a listed limit until Donkey
   Kong?
5. **E1's scope**: build only the damage-threshold armor now (Falcon), but with the kind field ready for super armor and knockback
   armor, as proposed?
6. **Names**: Gauntlets / sprinter / Sprinter and the blurb above, or other names?
