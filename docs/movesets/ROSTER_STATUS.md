# Roster status: reference ports

Every fighter ported (or planned) from the reference platform fighter, target version **13.0.1**. In the game every port has an original
body, weapon and move names; the reference character is named here only to say where the mechanics come from. The engine inventory these
gap lists are measured against is `SHARED_MECHANICS.md`; the test bar is `TESTING_STANDARDS.md`.

**Owner priority (October 2026): mechanics match the reference 1:1, not approximately.** Each fighter's iconic moves (the ones players
know by feel) are listed below as **top-fidelity priorities**, with exactly what must match. A port is not "verified" while any of them is
approximate. Where the engine cannot reproduce one exactly today, the entry says so and names the extension that would make it exact.

Status words: **verified** (every move on [R] data, tested, no approximations left in the iconic moves), **playable** (complete kit, some
[E] values or known limits), **in progress**, **planned**, **not started**.

## Existing ports

| In game (class, weapon, body) | Reference | Status | Tests | Known limits (from `docs/COMBAT.md`, `docs/sheik/`) |
| --- | --- | --- | --- | --- |
| 0 Longsword, duelist body | Marth | playable | `reference_moves.rs`, `kits.rs`, `combat.rs` | Normals on [R] frames and damage. **Up special on the reference's numbers since sim 58** (hits, grounded / aerial intangibility, 39-frame animation, front ledge from 8, special fall x0.4 and 24 frames of landing lag, x1.1 aerial rise, 20-degree angling, 0.06 gravity after the rise); its rise itself is still [E] (about 44 reference units: no published height). Pummel on [R] timing (sim 63). Down tilt's trip chance and the up smash's launcher hit are not modelled. Hitbox positions [E] (blade shortened to 72% on purpose). Shield Breaker's shield multiplier is [E]. Counter's answer uses our `percent` model (check the reference's 1.2x and minimum). Get-up attack at the reference's angle 48 since sim 73 (it was 361; owner decision 27). |
| 1 Claws, brawler body | Wolf | playable | `reference_moves.rs`, `kits.rs`, `throw_combos.rs` | Normals on [R] data. **Up special on the reference's numbers since sim 58** (hits, the dash from its published parameters, the grounded / aerial endings, the ground bounce, ledge windows, special fall x0.65 and 36 frames of landing lag); its looping hits send the way the fighter is travelling (the reference's angle type 1, sim 73, owner decision 23). Get-up attack at angle 48 since sim 73 (owner decision 27). Wall jump (sim 64). Pummel on [R] timing (sim 63). **Still [E]**: the flash dash's distance and end hit, the reflector's frames, size and reflected speed (it turns only the horizontal speed: owner decision 2026-10-13, keep it unlike the cape's). Wall jump also from the jump button (sim 65). Throw angles partly [E]. |
| 2 Needles, rogue body | Sheik | playable | `needles.rs`, `multihit.rs` | Body, normals, Needle Storm, Vanish, Bouncing Fish on [R] data (`docs/sheik/REFERENCE_DATA.md`). **The side special is our own pulling line, not the reference's grenade** (owner's decision 6: it stays). Wall jump since sim 64 (1.4 / 2.8). Pummel on [R] timing (sim 63). Vanish: no reappearance wind box (the push hit, E2, now exists to build it), explosion moves with the fighter; its special fall uses the shared drift and landing lag (not re-researched). Bouncing Fish: no consecutive kicks, no wall bounce. Up smash arm intangibility not modelled. Rapid jab cannot see presses in hitlag. Its get-up attack is the Claws' (angle 48 since sim 73, the reference's too). |
| 3 Embers, tumbler body | Mario | **playable** (phases 1 and 2 done, sims 57 to 64) | `embers.rs` | Body, every normal and every special on [R] data (`docs/movesets/mario/`): the rising punch, the bouncing fireball, the cape (reverse, reflector with its detector, once-per-airtime lift), the water pump (charge, store, pushing pumps), and the wall jump. **Not verified**: the rising punch's rise is [E] (no published height; re-searched 2026-10-13), and a few readings are [E] (the cape's lift frame, a reflected shot's vertical reversal). **Owner decisions 2026-10-13 (sim 65)**: a full pump holds its stance until special or shield; the cape's reflector turns vertical speed round and the Claws' stays horizontal-only (meant to differ); jump pressed against a wall (stick neutral or toward it) also wall-jumps, for every wall-jumping body, without spending a midair jump. Up smash head intangibility not modelled (E19). One get-up attack slot (face up). Positions on non-root bones [E]. |

| 4 Gauntlets, sprinter body (built-in fighter Sprinter) | Captain Falcon | **playable** (phase 1 sim 66, phase 2 sims 67-71, merged at 72; follow-ups sim 73) | `gauntlets.rs`, `gauntlets_specials.rs`, `angle_types.rs` | Body, every normal and every special on [R] data (`docs/movesets/captain_falcon/`): the jab chain and rapid jab, the one-frame knee, the down air meteor, intangible legs (E19), shieldstun multipliers (E24); the side special's search dash (E21) into the armored uppercut (E1, 10% on 1-4) or the meteor overhand, helpless only after an aerial miss; the up special's leaping command grab (E6) with the explosion on bystanders, free after a catch; the punch, reversed on frame 15, never clanking (E3c), the aerial one's bleed and 35-degree boost; the slide kick with its wall crash (`wall`) and off-edge version (E22), the dive kick and its landing hit; every special box's angle type (sim 73). The neutral air dodge's 42 frames now has two sources (sim 73). **Not verified**: the specials' travel distances (dash, rise, slide, dive) and the aerial punch's boost speed are [E] (not published); the uppercut's armor window 1-4 (conflict 2). Limits: the knee's late-hit trip (E20); one get-up attack slot; the rapid jab's "let go" [E]; positions on non-root bones and the body's height [E]. |
| 5 Staff, seer body (built-in fighter Seer) | Palutena | **in progress** (phase 1 done, sim 67: the body and every normal) | `staff.rs`, `angle_types.rs` | Body and every normal on [R] data (`docs/movesets/palutena/`): jab 1 and the rapid jab's entry rule (two presses and releases on 10-24, or held on a hit at 13), its loop, finisher and x7 shieldstun; the forward tilt's set-weight phases; the up tilt's, neutral air's and up air's loops and last hits; the neutral air's late -13 hitstun (E27) and the angle types of every box (E26: the neutral air's last hit forward, every up air box backward); the smashes' wind as pushes re-applied every frame; the dash attack's and back air's head-and-torso intangibility (E19). **Specials are still the claws' placeholders** (phase 2). No-clank boxes (E3c) set on the reference's `clang=f` moves since sim 73 (rapid jab and finisher, dash attack, smashes, down air, pummel, ledge and get-up attacks; `no_clank.rs`). Limits: the arm-intangibility windows act on nothing (owner decision 26: a listed limit) (a held staff); one get-up attack slot; the rapid jab's "let go" [E] as the Gauntlets'; the down throw's hitlag folded into its timeline [D]; positions on the staff bone and the body's height [E]. |

**Angle types on the older ports (sim 67, owner decision 21).** Every move of the five older weapons with a box reaching the fighter's
centre or behind it now carries its reference angle type (ssbwiki.com, 13.0.1); the list is `sim-core/tests/angle_types.rs` and the
Staff's implementation report. Visible changes: boxes of type 0 (by position) send a target the way it is from the attacker, not by the
box's side (the Embers' up smash, every down smash, the up tilts and up airs sweeping over the head, the get-up attacks, the Gauntlets'
down tilt box at the body). The type 3 (forward) and type 4 (backward) boxes set on the older ports all sit on the side that already
sent that way, so they change nothing today (they are data for later repositioning). The reference's type 1 (from the direction of
travel) was added in sim 73 (`ANGLE_BY_TRAVEL`, owner decision 23) and set on the Claws' up special loop; the Gauntlets' specials got
their types in sim 73 too.

Top-fidelity priorities for the existing ports (to revisit once the shared extensions below exist):

* **Longsword**: the tipper on every sword move (done, [R]); Dolphin-Slash-style up special: hits, intangibility, ledge window,
  special-fall drift and landing lag, angling (done, sim 58); rise height (**[E]**, no published figure); Counter's window and answer
  (check against [R]).
* **Claws**: the fire flight's travel and hit sequence (done, sim 58, from the published parameters); the reflector (frames, field
  size, reflected speed and angle: still [E]; E10b's vertical reversal exists but stays off: owner decision 2026-10-13, the Claws' reflector stays horizontal-only);
  the flash dash's distance and end hit ([E]).
* **Needles**: up air drag-down (done, [R]); Bouncing Fish (done except consecutive kicks and wall bounce); Vanish (done except wind box,
  which E2's push hit can now build); the grenade (our own line stays, owner's decision 6); wall jump (done, sim 64).

## The seven reference fighters

Extensions are named E1 to E20 here and described once, in "Infrastructure" below. "Exists" means the mechanic is in
`SHARED_MECHANICS.md` and can be reused as is.

### Mario

* **Status (2026-10-13)**: phase 1 (sim 57: the body and every normal) and phase 2 (sims 58 to 64: the four specials and the wall
  jump, with E2, E3b, E10, E10b, E11, E15, E16 and the set-knockback launch rule) done on branch `movesets`; **playable**, not yet
  verified (the rising punch's rise is [E]). Report: `docs/movesets/mario/IMPLEMENTATION_REPORT.md`.
* **Kit mechanics**: three-hit jab (exists: `Move::next`); Fireball (gravity, bounces along the ground several times); Cape (reflects at
  1.5x, **turns opponents around and reverses their control**, a fall break in the air; no armor since an update); Super Jump Punch
  (seven hits 5 / 0.6 x5 / 3, intangible startup, angled 12 degrees by the stick, ledge sweetspot from frame 14, helpless); F.L.U.D.D.
  (charges, stores like Needle Storm, 7 shots, **0% damage push** with set knockback 70, aimable up or down, pushes Mario back when charged,
  charge-cancel by jump or spot dodge). Wall jump.
* **Top-fidelity priorities**:
  * **Super Jump Punch**: hit sequence (7 hits, damages, the multi-hit's link and the last hit's BK 50 set-knockback rule without rage),
    intangible frames, rise and drift (the move's own physics), the 12-degree angling, ledge grab from frame 14 only, helpless, special-fall
    landing lag. Needs **E11** (per-move physics, ledge window, landing lag) and checks that set knockback ignores rage (it does here:
    `fixed_kb` skips `rage`).
  * **Cape**: reflect timing and the 6.5-unit field position; the reversal of the opponent's facing and momentum; the fall break once per
    airtime. Needs **E10** (reverse-on-hit, reflector reversing vertical speed too) and **E16** (a once-per-airtime flag a script can read).
  * **F.L.U.D.D.**: charge time, storing and cancels, push strength by charge, no damage and no hitstun. Needs **E2** (wind boxes) and,
    for the stream, **E3a** (a projectile that hits each target once and passes on, or several pushing shots).
  * **Fireball**: arc, bounce count and height, life. Needs **E3b** (projectile ground bounce).
* **Gaps**: none left for the kit (E3a was not needed: each pump hits once). Open: the rising punch's rise height.
* **Cost**: done.

### Donkey Kong

* **Kit mechanics**: Giant Punch (110 frames to full charge since 6.0.0, storable, **super armor when fully charged** on frames 9-20
  grounded / 15-20 aerial, lower-arm intangibility frames 11-27, base + 18 x charge / 110); Headbutt (**buries grounded targets**, the
  airborne hit is a bury-effect hitbox at 270 that acts as a meteor, huge shield damage, **super armor frames 5-14**, extra hitbox against
  grounded targets since 9.0.0); Spinning Kong (ground: horizontal multi-hit travel; air: rising multi-hit recovery, helpless); Hand Slap
  (ground: **hits grounded targets only**, loops while special is pressed; air: a two-hit clap whose second hit is a meteor, 10 frames
  landing lag); **cargo carry** off the forward throw (walk, jump, carried off stage, four cargo throws, mash escape).
* **Top-fidelity priorities**:
  * **Giant Punch**: charge length, storing and the 4-frame charge-cancel lag, damage formula, full-charge super armor windows, the lower
    arm's intangibility. Needs **E1** (armor), **E19** (partial intangibility) for the arm; the 110-frame charge exceeds the 60-frame
    stall cap, so it must loop with `goto` (exists) or the cap must become per move.
  * **Headbutt**: bury on grounded hits (bury duration from percent, mash out, hit out), meteor in the air, shield damage, armor 5-14.
    Needs **E5** (bury) and **E1**.
  * **Cargo carry**: walking and jumping with the held fighter, walking off a ledge with it, the mash escape, the four cargo throws.
    Needs **E7** (carry state) and **E17** (more move slots: four cargo throws do not fit in the free slots).
  * **Spinning Kong**: ground travel distance and hit count; air rise, hit sequence, helpless and landing lag. Needs **E11**.
* **Gaps**: E1, E5, E7, E11, E17, E19. Hand Slap's grounded-only hits exist (`TARGETS_GROUNDED`); its loop is a script.
* **Cost**: high (carry is a new held state with its own movement; bury is a new victim state).

### Cloud

* **Kit mechanics**: Blade Beam (slow, long, disappears on contact; stronger early and on the ground; aerial version slower with a fall
  break); Cross Slash (five hits 4 / 3 / 3 / 3 / 6, **each follow-up only if the previous hit connected**, by pressing special again);
  Climhazzard (rise with set-knockback rising hit, press special again for the descending slash whose centre meteors, ledge snap from
  frame 24 since 7.0.0, landing lag 26, tech at the peak); Limit Charge (hold to charge 0.3 units a frame since 7.0.0, cancels by shield,
  roll, grab, special, jump or air dodge with 7 frames of lag; being hit while charging removes the charge's gain); **Limit gauge** (fills
  0.667 per percent dealt, 1 per percent taken; when full: walk speed and accel x1.15, dash and run x1.1, traction x1.15, air speed and
  accel x1.2, air friction x1.15, gravity and fall speed x1.1; expires 15 seconds after filling if no special is used; any special used
  empties it); **Limit specials**, each its own move: Blade Beam (faster, multi-hit, transcendent, intangible start, stage-spikes on
  walls), Cross Slash (all five hits automatically, intangible start, stops momentum in the air), Climhazzard (1.5x higher, more drift,
  ledge snap throughout, intangible start), Finishing Touch (1%, huge knockback and wind boxes; startup 12 / 14 in **13.0.1**).
* **Top-fidelity priorities**:
  * **Limit gauge and Limit Break**: gain rates from damage dealt and taken and from charging, the full-gauge stat multipliers, the 15-second
    expiry, emptying on use, being hit while charging. Needs **E9** (persistent gauge with gain hooks, a timed mode, runtime stat
    multipliers). **On stock loss the gauge and any Limit Break reset to empty** (owner decision 9, below). Evidence: the earlier
    game's Limit Break ended on a KO, and ssbwiki.com/Limit_Break says the 13.0.1 game *also* (in addition) added the 15-second expiry; a
    player guide (gameshedge.com Cloud guide) says the gauge resets on a KO. No source gives it as a table value, so it is the owner's
    call backed by these sources [R, owner]. Confirm in footage if a recording is available.
  * **Cross Slash**: per-hit continuation only on hit, input windows, damages, the Limit auto-sequence. Exists (Dancing Blade pattern,
    `hit` register), but the move count needs **E17**.
  * **Climhazzard**: rise height and drift, the second press, the descending meteor's sweetspot only after a fall distance, ledge windows,
    landing lag 26. Needs **E11**.
  * **Blade Beam (Limit)**: multi-hit and passing walls as a hit. Needs **E3a** (multi-hit projectile) and transcendent priority (**E3c**).
* **Gaps**: E2 (Finishing Touch wind boxes), E3a, E3c, E9, E11, E16 (a `gauge`/`limit` register, "was hit during this move"), E17 (four
  normal specials + four Limit specials + Cross Slash follow-ups + Climhazzard's descent do not fit in ten `ext` slots).
* **Cost**: moderate to high (E9 is a new kind of state: a fighter whose movement changes during a match).

### Palutena

* **Status (sim 67)**: **phase 1 done**: class 5 Staff on the seer body, built-in fighter Seer (owner decision 19), every normal on
  [R] data, with E26 (angle types, also applied to the older ports) and E27 (signed hitstun modifier); specials are placeholders. Report
  and phase-2 plan: `docs/movesets/palutena/IMPLEMENTATION_REPORT.md`.
* **Research (2026-10-13)**: `docs/movesets/palutena/REFERENCE_DATA.md` (the full kit at 13.0.1,
  labelled, with conflicts and 13 gaps) and `PROJECT_AUDIT.md` (what exists; the exact shape of E3a, the E16 target lock and `flick`,
  two small E11 additions; new E10c counter / reflector routing, E25 a reflector left in the stage, E26 angle types, E27 negative
  hitstun; 4 `ext` slots). **Corrections to the notes below from that research**: nothing in the kit trips (the shots can *lock*, so
  E20 is not needed); the Reflect Barrier is **stationary** in 13.0.1 (it moved in the earlier game) and outlives the reflection's
  FAF; the counter and the reflection are **exclusive** (melee is countered, projectiles reflected, never both); Warp's edge-cancel
  already works in the engine.
* **Kit mechanics**: Autoreticle (a reticle on the nearest opponent in range; three shots of 3.5% on frames 25 / 31 / 37 toward it; the
  shots can trip; a fall break in the air); Explosive Flame (placed about four body lengths ahead, 7.5 units with a smash input; six 1.5%
  loop hits and a 5.5% final with 1.5x hitlag; FAF 72); Warp (stick-aimed teleport, intangible 16-32, falls faster while warping, no hitbox,
  helpless unless it edge-cancels into landing; ledge-grab rules changed in 2.0.0); Counter (window 7-34, intangible 6-12, 1.3x, answer
  BK 50 / KS 30); Reflect Barrier (projectiles only, frames 1-42, 1.4x, no hitbox, no wind box in Ultimate). Counter and Reflect Barrier are
  **one down special** in Ultimate that does both.
* **Top-fidelity priorities**:
  * **Warp**: distance by stick direction, the intangible window, the fall speed while warping, where it ends against the stage (ground,
    walls, platforms), helpless, the edge cancel, ledge rules. Partly exists (Vanish: scripted teleport, intangibility, hidden view);
    needs **E11** and research into how the reference resolves stage collision during the warp.
  * **Explosive Flame**: placement distance, delay, loop and final hit timing. Needs **E3a** (a stationary multi-hit projectile with a
    final hit).
  * **Autoreticle**: target selection (range, line of sight), shot timing, trip. Needs **E16** (a fighter-script target register or a
    projectile spawned toward the nearest opponent) and **E20** (trip).
  * **Counter + Reflect Barrier** on one move: the counter catches melee, the barrier reflects projectiles, both at once. Exists in parts
    (`Move::counter` and `Move::reflector` are separate fields; check that one move can carry both and that the counter does not also
    catch the projectiles the barrier should reflect).
* **Gaps** (from the audit): E3a, E3c (projectiles), E10c, E11 (two fields), E16 (`lock`, `flick`, Falcon's `presses`), E19, E24,
  E25, E26, E27. Counter and reflector can sit on one move as data, but need E10c and E25 to behave as the reference.
* **Cost**: moderate.

### Yoshi

* **Kit mechanics**: **double jump armor** (subtracts 120 knockback from frame 1, for part of the jump; a hit that exceeds it lands with
  the difference) and its own jump curve; **egg shield** (covers the whole body and does not shrink; head and legs briefly intangible as it
  comes up, 7.0.0); Egg Lay (command grab: swallows into an egg, escape by mashing, longer at higher percent, intangible on escape; the
  trapped fighter can shift on the ground and hop; aerial version longer); Egg Roll (hop, then a rolling egg steered by the stick, damage
  by speed, one jump, bounces once, passes through shields, special cancels it, ledge stall); Egg Throw (aimed along a parabola by the
  stick, bounces on the ground keeping its speed, 6%, hitlag 0.6x; in the air the first three throws give a vertical boost); Yoshi Bomb
  (ground hop version 4% then the drop, 15% grounded / 12% aerial, landing stars of 4% each, sweetspots a ledge facing away).
* **Top-fidelity priorities**:
  * **Double jump armor and curve**: threshold, window, the jump's height and shape. Needs **E1** (knockback-subtracting armor, on an
    air jump rather than a move) and **E12** (per-body air-jump profile).
  * **Egg shield**: whole-body coverage that never shrinks (so it cannot be shield-poked) while other fighters' shields can be. **Today
    nobody's shield can be poked** (any hit on a shielding fighter is blocked), so the egg shield has nothing to differ from. Needs **E13**
    (shield geometry for everyone, per-fighter shield model). This changes every existing fighter: owner decision.
  * **Egg Lay**: who it catches (air and ground; shields to research), egg duration and escape, release. Needs **E6** (command grabs)
    and **E8** (encased-victim state).
  * **Egg Throw**: aiming, arc, bounce, the three per-airtime boosts. Needs **E3b** and **E16**.
* **Gaps**: E1, E3b, E6, E8, E12, E13, E16, E19 (partial intangibility on shield).
* **Cost**: high (E8 and E13, and E13 touches everyone).

### Captain Falcon

* **Status (sim 73)**: **playable**. Phase 1 (sim 66, schema 5): class 4 Gauntlets on the sprinter body, built-in fighter Sprinter,
  every normal on [R] data, with E17, E19, E24 and the `presses` / `releases` registers. Phase 2 (sims 67-71, merged at 72): the four
  specials, with the shield stun fix (every fighter), E1 armor, the E11 additions, E21 search boxes, E6 command grabs, E3c no-clank
  hits, E22 off-edge versions and the `wall` register. Sim 73: the specials' angle types, the air dodge's second source. Not verified
  while the specials' distances are [E]. Report: `docs/movesets/captain_falcon/IMPLEMENTATION_REPORT.md`; research: `REFERENCE_DATA.md`,
  `PROJECT_AUDIT.md`.
* **Kit mechanics**: Falcon Punch (turning around during startup gives the reversed punch: 25 / 28 grounded, 22 / 25 aerial; aerial nudged
  up or down by the stick; slides back at the end); Raptor Boost (grounded dash into an uppercut with **10% damage-based armor on frames
  1-5**; aerial overhand that meteors; **helpless only if the aerial version misses**); Falcon Dive (**command grab that catches any
  opponent in range**, including out of shield; 13% explosion throw; after a catch Falcon flips back free and may use it again; throw end
  lag 24 since 11.0.0); Falcon Kick (ground slide, aerial diagonal dive, weaker each hit, 9% landing hit, reduced landing lag; startup 13
  ground / 14 air since 8.0.0); wall jump; the knee (forward air sweetspot: exists as data).
* **Top-fidelity priorities**:
  * **Falcon Punch**: the turnaround window and the reversed version's damage and startup, the aerial nudge. Exists (script: `turn`,
    `goto`, stick reads), given enough slots.
  * **Falcon Dive**: catching airborne and grounded targets, the hold and explosion timing, the release launch, Falcon's state after a
    catch (free, can recover again) versus helpless on a miss, ledge rules. Needs **E6** (command grab of airborne targets with a move's own
    sequence) and **E11**.
  * **Raptor Boost**: armor threshold, aerial meteor, helpless on whiff only. Needs **E1** (damage-threshold armor); the helpless rule is
    a script `goto` to a non-helpless slot on hit (exists).
  * **Falcon Kick**: ground and air paths, landing hit (exists: `landing_move`), going off an edge mid-slide (exists), landing lag.
  * **Knee**: sweetspot timing and size (exists).
* **Gaps**: none left for the kit (E1, E3c, E6, E11 additions, E15, E21, E22 and `wall` built). Open: the root-motion distances.
* **Cost**: done.

### Link (adult)

* **Kit mechanics**: **Hylian Shield** (standing, walking or crouching, it blocks any projectile from the front with a small push; shield
  SDI since 4.0.0); Hero's Bow (charges to frame 52, arrow speed, range and damage 4 to 12 by charge, fires by itself at full after three
  seconds; arrows lodge in the ground or walls for 3 seconds and **can be picked up and thrown as items by anyone**, or fired as a double
  arrow at 1.5x); Boomerang (out: launches up; back: 3%, launches toward Link, slower; angled up or down; a smash input goes further;
  reflected it can hit Link; knocked down by another projectile; caught with a 9-frame catch); Remote Bomb (**pulled out as a held item,
  thrown or dropped; bounces off fighters; knocked about by attacks; detonated by pressing down special again; 30-second fuse; one at a
  time; damages Link too**; cannot be detonated while helpless or while an opponent holds it); Spin Attack (below); a tipper-less sword
  (sweet and sour by timing and blade part).
* **Top-fidelity priorities**:
  * **Spin Attack, grounded**: per the reference (SmashWiki hitbox data, 13.0.1) it is **a single hit in four timing phases** (clean
    frames 7-8 at 14%, mid 9-13 at 12%, late 14-23 at 9%, latest 24-39 at 7%; BK 60 / KS 88), chargeable for 60 frames to 1.6x
    (`1 + 0.6 x charge / 60`), interruptible from 77. *Not* a multi-hit (see the question below). Exists: hit phases as rows,
    `charge_at` with `charge_bonus` 60 and the 60-frame cap reproduce the reference formula exactly.
  * **Spin Attack, aerial**: eight spin hits alternating front (frames 8-9, 16-17, 22-23, 31-32) and back (12-13, 19-20, 26-27, 38-39),
    the first two front hits at the autolink angle, then 83 / 69 and back hits at 145 / 150 / 160, 4% then 2% each, a final 4% hit on
    47-49 at 361 BK 30 KS 200; the rise (initial vertical speed 2.4 reference units, gravity x0.55, air speed x0.5 and extra accel 0.22
    while rising); from frame 48 the ending physics (air speed 0.8) and the special fall (air speed x0.909, accel 0.025); **ledge grab
    from the front from frame 15 and from behind from 34**; jostling off on 26-49; **landing lag 30**. The hits exist (groups, autolink,
    angles, data rows). **The movement, ledge and landing rules do not**: today the special fall's drift and landing lag are global /
    per body and `grabs_ledge` is the whole move from either side. Needs **E11** (per-move gravity and air-control multipliers, per-move
    special-fall drift and landing lag, ledge-grab windows from a frame and by side). Without E11 it can only be fitted, not matched.
  * **Remote Bomb**: hold, throw, drop, detonate on command, fuse, one at a time, self damage. Needs **E4** (held items) and **E3d**
    (owner-commanded detonation, long fuse, per-owner limit, self damage).
  * **Boomerang**: out and return paths, the two hits' properties, the catch, one at a time. Needs **E3a** (hits on the way out and back
    without being used up), **E3d** (per-owner limit, catch).
  * **Hylian Shield**: which states, the front-facing check, the push. Needs **E14**.
* **Gaps**: E3a, E3b (arrows lodging), E3d, E4, E11, E14, E16.
* **Cost**: highest (items are a new system that other fighters can interact with).

## Infrastructure: the extensions

Each is the smallest general change that does the job, with a default that leaves existing content and fighters unchanged. **Broad
architectural changes (E4, E7, E9, E13, E17) are explained to the owner before they are built.**

| Id | Extension | Shape (sketch) | Needed by |
| --- | --- | --- | --- |
| E1 | Armor (**built, sim 67**: `Move::armor`, damage-threshold and super kinds; knockback kind reserved) | per-move windows `{start, end, kind, threshold}`: super armor (never flinches), damage threshold (flinches only above N%), knockback subtraction (the reference's knockback-threshold armor: subtract N, flinch with the rest); plus a body field for armor on an air jump. Applied in `combat::apply_hit` before the hitstun state change | DK, Falcon, Yoshi |
| E2 | Wind boxes | a hitbox flag: pushes (sets or adds velocity from its angle and knockback) with no damage, no hitstun, no hitlag, re-applies each frame, pushes shields too if the reference does | Mario (F.L.U.D.D.), Cloud (Finishing Touch), Needles (Vanish), Link (to verify on the Boomerang's return) |
| E3a | Multi-hit / piercing projectiles | a per-projectile hit memory (who it hit, like `hit_mask`), a rehit interval and a final-hit phase, and a flag for "survives a hit" | Cloud, Palutena, Link, Mario |
| E3b | Projectile ground contact | per-spec ground behaviour: bounce (with restitution and a count), stop and lie (lodge), or die. Today a projectile only dies in a solid wall | Mario, Yoshi, Link |
| E3c | Transcendent priority / no-clank hits | **built (sim 70)**: `Hitbox::clanks` (false: never clanks with attacks or projectiles), also read for projectiles | Cloud; Gauntlets (punch, uppercut), Staff (sim 73) |
| E3d | Projectile lifecycle | `life` past 255 frames, a per-owner live limit, the owner catching it, the owner's later move commanding it (detonate), an explosion phase (a hitbox at its position), self damage | Link |
| E4 | Held items | a small item pool in `GameState`: pick up, hold, throw (directions and smash throws), drop, catch, usable by any fighter. Only if the owner wants Link's bombs and arrows 1:1 | Link |
| E5 | Bury | a victim state: stuck in the ground for a time from percent, mash to escape, any hit pops it out with the reference's rules | DK |
| E6 | Command grabs (**built, sim 69**: `Hitbox::command`, `grab::catchable`, `Move::catch_move`, `Move::hold_at`) | a grab hitbox that can catch airborne (and, if the reference allows, shielding) targets, holds the victim at a move-defined point instead of the global `grab_distance`, and hands to the move's own sequence (`ext` slots) instead of `Grabbing` | Falcon, Yoshi |
| E7 | Carry | the holder walks, jumps and falls with the held fighter (cargo), with its own speed multipliers, throw slots and the victim's mash escape; walking off stage takes the victim along | DK |
| E8 | Encased victim | the victim becomes an object (an egg) with its own physics and a mash-out timer from percent, hittable, intangible on escape | Yoshi |
| E9 | Gauge and powered mode | hashed `Fighter` fields (a gauge in fixed point, a mode timer), gain hooks on damage dealt and taken, a mode that applies per-body stat multipliers (new content: a "mode" block of multipliers on `FighterParams`, applied where params are read), script registers to read and spend it | Cloud |
| E10 | Reversing hits and full reflection | a hitbox flag that turns the victim round (facing and horizontal momentum) without launching it; reflectors also reverse vertical speed (E10b) | Mario (Cape), Claws reflector check |
| E11 | Per-move recovery rules (**built, sims 58 and 67**: also frame-ranged `air` blocks with an acceleration multiplier, `fast_fall_from`, `turn_at`, `lands_from`) | per move: gravity, fall-speed and air-control multipliers while it runs; special-fall drift and landing lag after it (falling back to today's global/body values); ledge-grab window from a frame, front and back separately | Link, Mario, DK, Cloud, Palutena, Falcon; existing up specials |
| E12 | Air-jump profile | a per-body air-jump velocity curve (the reference's jump that rises slowly then fast) | Yoshi |
| E13 | Shield geometry and per-fighter shields | a shield bubble that shrinks with health so hits can poke past it, per-fighter shield size and model (the egg: whole body, no shrink) | Yoshi, then everyone |
| E14 | Passive projectile block | a body option: in given states, a projectile hitting from the front is blocked with a push | Link |
| E15 | Wall jump | a body option and per-body numbers | Mario, Falcon, Needles |
| E16 | Script surface (built so far: `air_flag`, `presses` / `releases`, `wall`) | registers: `grab` button, nearest opponent (fighter scripts), once-per-airtime flags, "was hit during this move", "hit a shield"; more fighter variables than 4 | Mario, Yoshi, Palutena, Cloud, Link |
| E17 | More move slots | **built (sim 66, schema 5)**: sixteen `ext` slots, `ext10` to `ext15` numbered 39 to 44 after `pivot_grab` | Falcon (rapid jab now), Cloud, DK, probably Link |
| E19 | Partial intangibility | **built (sim 66)**: `Move::part_intangible` windows naming hurtbox circles (low, middle, high, limb) | Falcon (up tilt, up smash), DK, Yoshi, Needles (up smash), Embers (up smash head) |
| E24 | Per-hitbox shieldstun | **built (sim 66)**: `Hitbox::shieldstun` percent | Falcon (rapid jab x9, dash attack x1.5) |
| E20 | Trip | a hitbox chance or flag that trips a grounded target (deterministic: from the sim's RNG) | Longsword (down tilt), Falcon (forward air late hit); not Palutena (research 2026-10-13) |
| E10c | Counter / reflector routing | `Counter::melee_only`, `Counter::max_damage`, `Reflector::detect_then` (a projectile detector that starts another move) | Palutena |
| E25 | Reflector left in the stage | a projectile that reflects other owners' shots (`ProjectileSpawn::reflects`), a tall reflector shape | Palutena |
| E26 | Angle types | **built (sim 67)**: `Hitbox::angle_type`: by box side (the default), forward, backward, by position (melee and projectiles); by travel (the reference's type 1) since sim 73 | Palutena; the older ports re-checked (decision 21); the Claws' up special loop (decision 23) |
| E21 | Search boxes | **built (sim 68)**: hit kind `HIT_SEARCH` (touches without hitting, shields included), `Move::on_search`, `Fighter::next_move`; `Move::air_version` | Falcon (Raptor Boost) |
| E22 | Off-edge version | **built (sim 71)**: `Move::off_edge` (a ground move going over an edge switches slot on the same frame) | Falcon (Falcon Kick) |
| E27 | Negative hitstun modifier | **built (sim 67)**: signed `Hitbox::hitstun_add` (i16; a launch keeps 1 frame) | Palutena |

(E18 is not used: Warp's stage collision is research, not yet an extension.)

## Recommended order

1. **Mario.** Every extension he needs is small and general (E2 wind boxes, E3b ground bounce, E10 reversing hits, E11 per-move recovery
   rules, E15 wall jump, one E16 flag), and each is reused by later ports. E11 is the one that makes iconic recoveries exact, and it can
   be checked first against an existing up special. His charge-and-store is the Needle Storm pattern.
2. **Captain Falcon.** Adds E1 (only the damage-threshold armor at first) and E6 (command grabs), reuses E11 and E15. Most of his kit is
   data and scripts the engine already runs (turnaround punch, landing hit, slides off edges).
3. **Palutena.** Adds E3a (multi-hit stationary projectile), E20 (trip) and the target register; reuses E11 for Warp and the existing
   counter and reflector.
4. **Cloud.** Adds E9 (gauge and powered mode, the first runtime stat change) and E17 (more slots), reuses E2, E3a and E11. Needs the
   Limit-across-stocks question answered first.
5. **Donkey Kong.** Adds the rest of E1 (super armor), E5 (bury), E7 (carry), E19, and uses E17 for cargo throws. Carry is the largest
   single state so far; doing it after E6 means the hold code has already been generalised once.
6. **Yoshi.** Adds E8 (egg), E12 (jump curve), the knockback-subtracting armor (E1); reuses E3b and E6. Regular shield (owner
   decision 5: no E13).
7. **Link.** Last because 1:1 needs E4 (items) and E3d (commanded, long-lived, one-at-a-time projectiles), the two biggest pieces of new
   infrastructure, plus E14. **If the owner wants Link sooner** (his Spin Attack is iconic), his sword normals, Spin Attack (after E11)
   and the bow can be built as a phase of their own, with bombs, boomerang and the shield in a later phase; he would not be "verified"
   until then.

## Owner decisions (2026-10-13)

These override anything above that disagrees.

1. **Order**: confirmed as above (Mario, Captain Falcon, Palutena, Cloud, Donkey Kong, Yoshi, Link).
2. **Spin Attack**: match the reference (grounded is a single charged hit, aerial is the multi-hit).
3. **Items (E4)**: an item system will be built. What Link's bombs and lodged arrows do with it (anyone can pick them up, or owner only) is
   **undecided**: re-evaluate with the owner when the item system is built.
4. **Link's forward air**: the multi-hit forward air must **drag the target down on its first hit**, so the hits connect. Research the
   reference's hit data first; if the reference's first hit does not already do this, it is an owner-chosen change, recorded as such in
   Link's reference data.
5. **Yoshi's shield**: a regular shield. **E13 (shield geometry) is dropped**; no change to anyone's shield.
6. **1:1 for existing kits**: the Needles' side special (our pulling line) is our own move and **stays as it is**. Every other Needles and
   Claws special should be made as close to 1:1 with its reference as the engine allows (re-research the [E] values, then match them).
7. **Up specials and E11**: when E11 lands, re-fit both the Claws' and the Longsword's up specials to their reference numbers.
8. **Naming cleanup**: reference character names in code and test names are being removed as a separate change.
9. **Cloud's Limit on stock loss**: the gauge (any partial charge too) and an active Limit Break reset to empty when he loses a stock.
10. **Captain Falcon names**: class **Gauntlets**, body and built-in fighter **Sprinter**.
11. **Move slots (E17)**: add 16 `ext` slots now, with Falcon (a schema change with a migration); Cloud and DK reuse it.
12. **Root-motion distances** (Falcon Dive rise, Raptor Boost dash, Falcon Kick slide, Super Jump Punch rise and the like): fit them to
    published recovery-distance or height figures, labelled [D]/[E] with the source. No footage for now.
13. **Per-limb intangibility (E19)**: build it with Falcon (his up tilt and up smash legs); DK, Yoshi and the Needles' up smash reuse it.
14. **Armor (E1)**: build the damage-threshold kind with Falcon, with the `kind` field in place for the later kinds (DK, Yoshi).
15. **Sprinter rapid jab**: the press that starts jab 1 counts toward the three presses, and mashing keeps the rapid jab going (both as
    built in sim 66).
16. **Command grabs and search boxes (E6, E21)**: build them as new hit kinds in `combat::resolve_hits`.
17. **Shield stun rounding**: fix it so whole-number shield stun matches the reference for every fighter (it came out one frame short).
18. **Sprinter neutral air dodge (42 frames, one source)**: keep it unless a second source shows otherwise; check it against the reference.
19. **Palutena names**: class/weapon **Staff** (the owner allowed Beacon or Staff; Staff fits the other weapon names), body and built-in
    fighter **Seer**.
20. **Reflect Barrier**: keep it on the reflection move (no E25 stage object). Recorded as an owner-chosen approximation: the 10 frames
    after FAF 35 and "stays when she is hit" are not modelled.
21. **Angle types (E26)**: build them for Palutena and also re-check the existing fighters' angle types (back airs and others) against
    the reference.
22. **Palutena data conflicts**: counterattack hitbox on frames 5-7; heavy landing lag from the parameter file (6 here).
23. **Angle type 1** (from the direction of travel): add it and use it for the Claws' up special loop, per the reference (built in sim 73:
    `ANGLE_BY_TRAVEL`, the way the attacker moves sideways, its facing when it is not moving sideways [E]).
24. **Negative hitstun modifiers**: keep the 1-frame hitstun floor (a launched victim keeps at least 1 frame; the reference's 0 would be
    a flinch here).
25. **The Staff's rapid jab**: its cycle reading and its "let go" rule stay the same as the Gauntlets' (not held and not pressed for 8
    frames).
26. **Held-weapon bodies' arm intangibility**: the windows stay a listed limit (no arm hurtbox for now; they act on nothing).
27. **Get-up attacks**: fix the Longsword's and the Claws' to the reference's angle 48 (they used 361). Done in sim 73; the Needles use
    the Claws' get-up attack, whose reference is also 48 (ssbwiki.com raw tables), so theirs changed too.

## Still open

From the Embers' phase 2 (details in `docs/movesets/mario/IMPLEMENTATION_REPORT.md`, "Owner follow-up"):

1. Up-special rise heights (the rising punch, the cape's lift, the Longsword's): re-searched 2026-10-13, still no published figure in
   units (list of sources in `docs/movesets/mario/REFERENCE_DATA.md`); footage would settle them.

Answered by the owner on 2026-10-13 (sim 65): a full water-pump charge holds the stance until special or shield (built); reflectors
stay as they are, the cape turning a shot's vertical speed round and the Claws' not (meant to differ); the jump button against a
wall also wall-jumps, stick neutral or toward the wall, for every wall-jumping body, without spending a midair jump (built).
