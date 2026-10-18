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

| In game (class, weapon, body) | Reference | Status | Tests | Known limits (from `docs/COMBAT.md`, `docs/movesets/needles/`) |
| --- | --- | --- | --- | --- |
| 0 Longsword, duelist body | Marth | playable | `reference_moves.rs`, `kits.rs`, `combat.rs`, `longsword_jab.rs` | Normals on [R] frames and damage. **The jab is the previous version's (1.1.x) since sim 138** (decision 98: E40 rules, the jab cancel on both hits, 13.0.1's tip pull). **Up special on the reference's numbers since sim 58** (hits, grounded / aerial intangibility, 39-frame animation, front ledge from 8, special fall x0.4 and 24 frames of landing lag, x1.1 aerial rise, 20-degree angling, 0.06 gravity after the rise); its rise itself is still [E] (about 44 reference units: no published height). Pummel on [R] timing (sim 63). Down tilt's trip chance and the up smash's launcher hit are not modelled. Hitbox positions [E] (blade shortened to 72% on purpose). Charged Thrust's shield multiplier is [E]. Counter on the reference's numbers since sim 78 (ssbwiki.com raw: window 6-27, 1.2x, at least 8% and at most 50%; it catches on its counter bubble, radius 10.5 at 10 up [R], and never catches a push). Get-up attack at the reference's angle 48 since sim 73 (it was 361; owner decision 27). |
| 1 Claws, brawler body | Wolf | playable | `reference_moves.rs`, `kits.rs`, `throw_combos.rs` | Normals on [R] data. **Up special on the reference's numbers since sim 58** (hits, the dash from its published parameters, the grounded / aerial endings, the ground bounce, ledge windows, special fall x0.65 and 36 frames of landing lag); its looping hits send the way the fighter is travelling (the reference's angle type 1, sim 73, owner decision 23). Get-up attack at angle 48 since sim 73 (owner decision 27). Wall jump (sim 64). Pummel on [R] timing (sim 63). **Still [E]**: the flash dash's distance and end hit, the reflector's frames, size and reflected speed (it turns only the horizontal speed: owner decision 2026-10-13, keep it unlike the cape's). Wall jump also from the jump button (sim 65). Throw angles partly [E]. |
| 2 Needles, rogue body | Sheik | playable | `needles.rs`, `multihit.rs` | Body, normals, Needle Volley, Flash Step, Flip Kick on [R] data (`docs/movesets/needles/REFERENCE_DATA.md`). **The side special is our own pulling line, not the reference's grenade** (owner's decision 6: it stays). Wall jump since sim 64 (1.4 / 2.8). Pummel on [R] timing (sim 63). Flash Step: no reappearance wind box (the push hit, E2, now exists to build it), explosion moves with the fighter; its special fall uses the shared drift and landing lag (not re-researched). Flip Kick: no consecutive kicks, no wall bounce. Up smash arm intangibility not modelled. Rapid jab cannot see presses in hitlag. Its get-up attack is the Claws' (angle 48 since sim 73, the reference's too). |
| 3 Embers, tumbler body | Mario | **playable** (phases 1 and 2 done, sims 57 to 64) | `embers.rs` | Body, every normal and every special on [R] data (`docs/movesets/embers/`): the rising punch, the bouncing fireball, the cape (reverse, reflector with its detector, once-per-airtime lift), the water pump (charge, store, pushing pumps), and the wall jump. **Not verified**: the rising punch's rise is [E] (no published height; re-searched 2026-10-13), and a few readings are [E] (the cape's lift frame, a reflected shot's vertical reversal). **Owner decisions 2026-10-13 (sim 65)**: a full pump holds its stance until special or shield; the cape's reflector turns vertical speed round and the Claws' stays horizontal-only (meant to differ); jump pressed against a wall (stick neutral or toward it) also wall-jumps, for every wall-jumping body, without spending a midair jump. Up smash head intangibility not modelled (E19). One get-up attack slot (face up). Positions on non-root bones [E]. |

| 4 Gauntlets, sprinter body (built-in fighter Sprinter) | Captain Falcon | **playable** (phase 1 sim 66, phase 2 sims 67-71, merged at 72; follow-ups sim 73) | `gauntlets.rs`, `gauntlets_specials.rs`, `angle_types.rs` | Body, every normal and every special on [R] data (`docs/movesets/gauntlets/`): the jab chain and rapid jab, the one-frame knee, the down air meteor, intangible legs (E19), shieldstun multipliers (E24); the side special's search dash (E21) into the armored uppercut (E1, 10% on 1-4) or the meteor overhand, helpless only after an aerial miss; the up special's leaping command grab (E6) with the explosion on bystanders, free after a catch; the punch, reversed on frame 15, never clanking (E3c), the aerial one's bleed and 35-degree boost; the slide kick with its wall crash (`wall`) and off-edge version (E22), the dive kick and its landing hit; every special box's angle type (sim 73). The neutral air dodge's 42 frames now has two sources (sim 73). **Not verified**: the specials' travel distances (dash, rise, slide, dive) and the aerial punch's boost speed are [E] (not published); the uppercut's armor window 1-4 (conflict 2). Limits: the knee's late-hit trip (E20); one get-up attack slot; the rapid jab's "let go" [E]; positions on non-root bones and the body's height [E]. |
| 6 Greatsword, champion body (built-in fighter Champion) | Cloud | **playable** (phase 1, sim 83: the body and every normal; phase 2, sim 85: the four normal specials and the Boost gauge; phase 3, sim 87: the four powered versions) | `greatsword.rs`, `greatsword_specials.rs`, `greatsword_powered.rs`, netplay `powered_rollback.rs`, `gauge.rs`, `move_rules.rs`, `angle_types.rs`, `ground_attributes.rs` | Body and every normal on [R] data read from ssbwiki.com's raw hitbox tables (`docs/movesets/greatsword/`): the jab chain on its continuable windows (8-30, 9-30) and the held repeat from frame 16; the forward smash's three single-frame set-weight hits with grounded / aerial boxes; the up smash's hilt priority; the down smash's two hits (the second backward); the up air as one hit in three phases; the forward and down air aerial-only meteors; hit-then-throw throws (the up throw's two pre-hits), the down throw's 10-frame hitlag folded into its timeline [D]; no-clank boxes and angle types from the raw tables; the wall jump 1.3 / 2.4. **Limits**: positions on the sword and leg bones [E]; the down tilt's slide distance [E] and its back-leg intangibility (acts on nothing, as the Staff's arms); the forward air's "ledges from frame 78" (no aerial ledge lockout); one get-up attack slot; the grabs' aerial-target boxes, the throws' interrupted rows and the thrower's invincibility (house limits). **Phase 2 (sim 85)**: the wave (ground / air, a standing-capsule hit, the fall break, the land switch), the combo slash (each part only on a hit, shields included), the rising slash (fitted rise [E] that reproduces the reference's quickest loop: landing hit on 45, free on 71; second press 20-26; falling slash at 4.2 with the aerial-only meteor and the landing hit; ledges from 24, the fall-gated ledges [E]), the gauge charge (0.3 a frame, cancels, drain); the Boost gauge itself (gain, drain, 900-frame powered mode with the multipliers, KO reset). Limits added: the combo slash's boxes [E]; the charge's start / full-gauge end / air physics [E] (owner decision 43). **Phase 3 (sim 87)**: E3e bursting projectiles, E28 the vortex angle 366 (now on the combo slash's nearest box), E11 `motion_drift`; the powered wave (intangible 10-17, a no-clank capsule beam that bursts on its first fighter or a wall: 19% / 13% behind, exact), the powered combo slash (all five hits by themselves, 26% [E: conflict 1], intangible 6-11, no momentum in the air), the powered rise (x1.6 / x1.85, intangible 5-12, air control while rising [E], ledges throughout, second press 33-39), the finisher (12 / 14 startup, FAF 85 / 88, KS 431-349, both-sides windbox, the hover); any special in the powered mode spends the gauge. Limits: the powered combo slash's total and boxes, the powered rise's air control (gap 8), the burst not reflectable [E]. Report: `docs/movesets/greatsword/IMPLEMENTATION_REPORT.md`. |
| 7 Bracers, hauler body (built-in fighter Hauler) | Donkey Kong | **in progress, every move built** (phase 1, sim 86: the body and every normal; phase 2, sim 94: every special; phase 3, sim 98, branch `bracers-phase3`: the carry (E7) and the four cargo throws; the iconic-move re-check against footage still open) | `bracers.rs`, `bracers_specials.rs`, `carry.rs`, `bury.rs`, `trips.rs`, `angle_types.rs`, `ground_attributes.rs` | Body and every normal on [R] data from ssbwiki.com's raw tables (`docs/movesets/bracers/`), the angled forward tilts ending on 36 since phase 2 (the `aimed` register). **Specials (phase 2)**: the charged punch (E29 additive charge damage: 110-frame charge kept on cancels, lost on a hit, stored at full; the super-armored full punch, the landing switch), the headbutt with the E5 bury (the reference's length with the stock terms, mashing, the 90-knockback break, kept launches) and the aerial meteor, the grounded and aerial spinning clothesline (366 first hit via E28, the loop, the stick slide, push immunity, the rise, the special fall), the hand slap (the banked 26-frame loop, grounded-only slaps, the aerial clap and meteor). **Limits**: positions on the arm, leg and head bones [E]; the charge's frame mapping, the bury's pop-out and kept-launch rule, the spin's off-edge version, gravity reading and push-immune frames, the slap's air physics and late-press reading [E] (listed in the report); one get-up attack slot; the grabs' aerial-target boxes, the throws' interrupted rows and the thrower's invincibility (house limits). **Carry (phase 3)**: the 15-frame invincible lift, the clock `(90 + 1.7p) x 11 / 6 - g` with the grab's time and mashing carried over, the x1.1 walk, one jump, off the stage with the victim, the four cargo throws on [R] frames and data; [E]: the armor's threshold (80, not subtractive: conflict 2, gap 11), the carry's jump and landing (gap 3), no ledge (gap 4), the ordinary release (gap 5), the holder's knock-out letting go (gap 13), the hold's place overhead, the turn (at once) |
| 8 Clutch, bounder body (built-in fighter Bounder) | Yoshi | **in progress** (phase 1, sim 89: the body with its double-jump armor and turnaround (E33) and every normal; phase 2, sim 95: the lobbed shell (up special) and the drop (down special); phase 3, sim 110: the tongue and its egg (neutral special, E8) and a simple rolling shell (side special)) | `clutch.rs`, `clutch_specials.rs`, `clutch_egg.rs`, `late_jab.rs`, `move_rules.rs`, `multihit.rs` (knockback armor), `angle_types.rs`, `ground_attributes.rs` | Normals on [R] data (ssbwiki.com raw tables, 13.0.1): the jab's continuable window and both held rules, the aimed forward tilt and smash, the down tilt's trips (E20), the head and foot intangibility (E19), the down air's twelve autolink kicks, finisher and landing hit, the set-weight back air links, the tongue grabs, the down throw's folded hitlag; jab 1's window past its FAF (decision 58) and the double jump's x1.2 drift (decision 59). Specials (phase 2): the shell's held speed, stick aim, three hops an air time, bounces and burst (one hit); the drop's hop hit, stall, 0.6 fall, platform drop (E30), ledges both sides and both landing stars on one frame (E3h). Estimates [E] (decision 60): the shell's aim curve and hop frame, the bounce reading (conflict 10), the drop's rise and stall; the egg's size, side, physics and break-out (gaps 5-9), the swallow's 6 frames (conflict 5), the roll's steering, jump and ledges. **Limits**: the roll's E31 / E32 (hit in the air and while turning, no ledge stall: decision 49); positions on the tail, head, leg and tongue bones [E] (the tongue's reach 22 reference units, gap 3); the grabs' aerial-target boxes, the throws' interrupted rows and thrower invincibility, the forward air's "ledges from 68", the face-down and trip get-up attacks (house limits); crawling, the 7.0.0 shield intangibility (decision 5). |
| 9 Buckler, wanderer body (built-in fighter Wanderer) | Link | **in progress** (phase 1, sims 91 / 96: the body with its passive shield (E14), every sword normal and both up specials; phase 2, sim 102: the bow, the boomerang and the sword wave (E35, E3b lodging, E3d's first part, E38); the bomb is the claws' placeholder until phase 3) | `buckler.rs`, `buckler_specials.rs`, `scripted_moves.rs` (`fast_fall`, `owned`), `angle_types.rs`, `ground_attributes.rs` | Normals on [R] data (ssbwiki.com raw tables, 13.0.1): the jab's continuable windows and held rules, tip / blade / arm / hilt boxes and their no-clank flags, the two-part forward smash keeping its charge (E36), the up smash's grounded / aerial set-knockback boxes and +3 hitstun, the forward air's autolink drag (decision 57), the down air's meteor and bounce (the `fast_fall` register), the hand grabs; the grounded spin (one charged hit in four phases, exact formula) and the aerial spin (eight hits and the finisher, E34 position vectors, the rise from its parameters, ledges 15 / 34, landing lag 30, the special fall's own speed and acceleration). **Limits**: positions on the sword, arm and leg bones [E]; the grounded spin's sweep round the body (gap 3) [E]; the aerial spin's air-acceleration reading and rise frame (gap 4) [E]; a position vector's drop-off after its frames (taken as a stop) [E]; the passive shield's size, place and push (gaps 1-2) [E]; the down air's after-bounce box lasting to its frame 64 [E]; the boomerang's turn back read from its published parameters (gap 6) [E]; arrow lodging on every surface at 9 degrees or steeper (gap 8) [E]; the bow in the air (gap 7) [E]; arrow pick-up, the double arrow and the bomb wait for items (phase 3); the grabs' aerial-target boxes, the throws' interrupted rows and thrower invincibility, the down throw's edge push, one get-up slot (house limits). |
| 5 Staff, seer body (built-in fighter Seer) | Palutena | **playable** (phase 1 sim 67: the body and every normal; phase 2 sims 74-77: the specials) | `staff.rs`, `staff_specials.rs`, `angle_types.rs` | Body and every normal on [R] data (`docs/movesets/staff/`): jab 1 and the rapid jab's entry rule (two presses and releases on 10-24, or held on a hit at 13), its loop, finisher and x7 shieldstun; the forward tilt's set-weight phases; the up tilt's, neutral air's and up air's loops and last hits; the neutral air's late -13 hitstun (E27) and the angle types of every box (E26: the neutral air's last hit forward, every up air box backward); the smashes' wind as pushes re-applied every frame; the dash attack's and back air's head-and-torso intangibility (E19). **Specials built in phase 2 (sims 74-77, branch `seer-phase2`)**: the warp, the counter and reflect barrier, the explosive flame and the reticle (`staff_specials.rs`); [E] readings: the warp's ledge windows and stage contact, the flame's height, the reticle's shot path and "in front" rule, the aerial specials' landings; the barrier ends with its move (decision 20). No-clank boxes (E3c) set on the reference's `clang=f` moves since sim 73 (rapid jab and finisher, dash attack, smashes, down air, pummel, ledge and get-up attacks; `no_clank.rs`). Limits: the arm-intangibility windows act on nothing (owner decision 26: a listed limit) (a held staff); one get-up attack slot; the rapid jab's "let go" [E] as the Gauntlets'; the down throw's hitlag folded into its timeline [D]; positions on the staff bone and the body's height [E]. |
| 10 Pockets, settler body (built-in fighter Settler) | Villager | **in progress** (phase 1, sim 113, branch `pockets-phase1`: the body, every normal, the rocket and the balloons; the jab is the previous reference version's on its rules, E40) | `pockets.rs`, `pockets_specials.rs`, `older_rules.rs`, `scripted_moves.rs`, `ground_attributes.rs` | Body (13.0.1) and every normal on [R] data from ssbwiki.com's raw tables (`docs/movesets/pockets/`): the projectile normals (flowerpot, bowling ball with its own 1.2x charge, slingshot pellets), the turnips' random counts (E39), the firework's six hits, the shovel's bury, the up tilt's head intangibility and +2. **The jab** (owner decisions 66-67): the 1.1.x jab (3% at 60 / 80, BK 10 / 20, KS 80 / 120, endless, no finisher) on that version's hit rules (E40): no 1.2x, the old hitlag, +1 tumble hitstun, no speed-up, the gravity boost, the old rage; jab 1 into jab 2 combos at 0-150%, 20 frames of advantage at 130%. **Specials**: the rocket (from 52, 7% / 5% / 12% burst, one out, ridden 15.4 / 11 / 18% with its bounce, crash and hop) and the balloons (E41: flaps, the helium tank, one balloon when low, pops, the special fall). [E]: the balloons' and the ride's numbers (unpublished: an owner question), the projectile normals' flights, positions on non-root bones, the jab's held rule. **Phase 2 (sim 115, branch `pockets-phase2`)**: the jab reworked (decision 70: `Move::cancel_from`, the jab cancel from frames 7 / 8, and 13.0.1's far-box pull on jab 1, a blend: the 1.1.x data has no pull); the pocket (E42: reach 8-23, intangible 5-23, FAF 51, release on 9 at x1.9, FAF 31, items and the swap, the rocket freeing its maker) and the tree (E43: planting, watering, growth 46 frames after the first drop, 18% / 13%, the tree's 30 hp and 600 frames, the axe 6-8 for 14% and its cuts, the falling tree's 25%, the wood chip): every special built. Jab 1 > axe is a true combo at 0-150%; jab 1 > 2 > axe only at 0-20% (reach, not time). [E]: the pocket's reach, the sapling, water and tree sizes, the axe's box places, the falling tree's path. Limits: `direct=f`, `noprone`, one get-up slot, the grabs' aerial boxes (house). |
| 11 Popgun, scamp body (built-in fighter Scamp) | Diddy Kong | **playable, every move built** (phase 1, sim 117: the body, every normal, the banana peel (E44, E45, E47), the peanut popgun and the rocket barrels (E48); phase 2, sim 119, branch `popgun-phase2`: the monkey flip with its latch (E49), the first-charge-only slow fall (decision 75), unblockable barrels, the crash's self damage) | `popgun.rs`, `popgun_peel.rs`, `popgun_peanut.rs`, `popgun_rocket.rs`, `popgun_flip.rs`, `trips.rs`, `late_jab.rs`, `scripted_moves.rs`, `move_rules.rs`, `ground_attributes.rs` | Body and every normal on [R] data from ssbwiki.com's raw tables (`docs/movesets/popgun/`): the three-hit jab (windows past the FAF, held repeat from 10), the aimed forward tilt, the three-hit dash attack, the set-knockback smashes, the aerials, grabs, throws. **The peel**: tossed at 105 degrees (2.2 / 3.036) on 20, FAF 40; one out and 85 frames between tosses; an item anyone picks up (E4), a thrown peel hard-trips (E44 slip), a lying one soft-trips whoever walks into it but its last thrower (E45 trap), two uses. **The popgun**: 16 uncharged, full charge 94, misfire 97, 4.3-13.8%. **The rocket**: charge 7-67, launch 3.1 + 0.019 a frame (x1.05 air), three hit phases, the special fall (30 landing), the wall crash (18%, r20), the barrels shed by a hit (E48) bursting on the stage, unblockable; the crash costs the scamp 5%; only the first charge of an air time falls slowly (decision 75). **The monkey flip** (sim 119): ground and air leaps launched on 13 with command grabs on 18-36 / 18-29 (FAF 62 / 48, landing lag 26), the kick on a second press (14 / 12%, 12 / 10% late, FAF 33, landing lag 18) barring the up special until landing, the latch (E49: the ordinary hold clock and mash-out, the caught fighter falling with the scamp on its head, a 1% scratch on 11 and 29 of every 48), its attack (3% + a 7% throw) and its jump (2%, the fighter held 20 frames). [E]: positions on non-root bones; the peel's and peanut's arcs; the rocket's flight fall-off, steering and charge fall; the barrels' flight; the flip's launch speed, arc, smash-input distance, grab box and kick window; the latch's ride point, scratch rhythm reading, the exits' jumps; the crash's 5%. Limits: the popgun's muzzle hit, catching peanuts; one get-up slot, the grabs' aerial boxes (house). |
| 12 Dagger, phantom body (built-in fighter Phantom) | the third of the second wave (`docs/movesets/dagger/`) | **every move built** (phase 1, sim 121: the body, every normal with and without the partner, the gauge and partner, the gun, the curse bolt, the guard, the partner's stance and wings; phase 2, sim 123, branch `dagger-phase2`: the hook and its tether (E56), damage dealt filling the gauge (decision 80), the gun's dodge staling and buffered delay, angle 365, the up smash's arm with the partner; phase 3, sim 126, branch `dagger-phase3`: a shield ending the grounded hook sooner, stale gun dodges stretched in time, the reel by a press, decisions 83-86) | `dagger.rs`, `dagger_gauge.rs`, `dagger_specials.rs`, `dagger_hook.rs`, `dagger_phase3.rs`, `move_rules.rs`, `scripted_moves.rs` | Body and every normal on [R] data from ssbwiki.com's raw tables (both the plain and the partner's tables). **The hook** (phase 2): the grounded grab on 20-26 (FAF 60) that a shield or reflector stops and that misses fighters in hitstun, its pull and 11% throw; the aerial hit (5%, 20-25, FAF 45, no special fall) with its once-per-air-time rise; **the tether** (E56): catches a free ledge from frame 12, hangs, reels in, three catches before landing, the stick down lets go [reach, hang 8 and reel 12 frames E]. **The gauge and partner** (E52, E53): the fill by itself (0.00925 a frame), 1.3 a percent taken, x1.625 / x1.95 a stock or two behind and the player table, 1.3 a percent dealt (decision 80, a house rule [E]), the guard's x6.7368, 1800 frames out with nothing filling, 16 frames cut a percent taken (10.2 with four fighters), a broken shield stopping the clock, 20 / 25 / 30 units after a KO; every normal but the pummel, throws, ledge and get-up attacks gets the partner's own hit on the same frame (damages add) and its awakened boxes, kept to the attack's end; every special's powered version without spending the gauge. **Specials**: the gun (the shot's three bands, presses and holds, the dodges, the jump, the shots below and the spiral, the partner's bullets), the curse bolt (1% then the 2% eruption, not on shields, 1% every 45 frames for 361; the partner's multi-hit bolt bursting on fighters and the ground, 1.5% every 40), the guard (super armor 3-19, held to 120, 0.4 of a hit, x1.5 hitlag, the backhand), the partner's counter (x1.6, 12-50%) / reflector (x1.6, x1.9) and wings. [E]: the gun's distances, the partner's arrival / leaving (not modelled) and longer rolls, the bolt's path, the wings' flight, the stance's bubbles, every box on a bone. Phase 3: a shield or reflector stop ends the grounded hook 20 frames later [E]; stale gun dodges play slower at the roll rates [E: which rate]; a press of special while hanging reels in at once (house). Limits: the buffered delay only out of a shot (not between dodges), the partner's arrival / leaving not modelled (decision 81), one get-up slot, the grabs' aerial boxes (house). |
| 13 Nodachi, exile body (built-in fighter Exile) | Sephiroth | **kit complete, estimates open** (phase 1, sim 125: the body, every normal, the wing (E57), the blade dash and eight slashes, the flare in three levels (E58 rings); phase 2, sim 128, branch `nodachi-phase2`: the shadow orbs (E59), the barrier, the down smash without ground contact, the slashes' landing hit, the forward air's wall stick) | `nodachi.rs`, `nodachi_wing.rs`, `nodachi_specials.rs`, `nodachi_phase2.rs`, `move_rules.rs` | Body and every normal on [R] data from ssbwiki.com's raw tables (`docs/movesets/nodachi/`): the long blade (30 of the reference's units of reach), the jab's held rule (10 after a hit, 19 without), the aimed forward tilt, the up tilt's lift by position vector and upper-body intangibility, the smashes' charge frames, the down air's stall, plunge and landing hit. **The wing** (E57): the reference's start table by stock and percent difference (every published example reproduced), the points (damage dealt, KOs), x1.3 damage that KOs earlier only by the extra, the multipliers, a third jump, 20% armor on the smash attacks (`Move::armor_mode`), gone until the next KO. **Specials**: the blade dash (the hold on 18, the aim on 19, the hit on 25-28, FAF 57 / animation 80, landing 27, ledges, edges) and the eight slashes (aim on 31, hits on 36-50 dragging, FAF 105, landing 56); the flare (release from 20; the bigger flare from 31 held; the giant flare at full charge; each burst: the vortex multi-hits, the three rings). [E]: the dash's and slashes' speeds, the flares' flight speeds, the giant flare's charge end (108), the forward and up smashes' blade read from the hitbox animation, every box on the blade, the KO credit, several opponents for the wing; the forward smash's 13.5% root box ranked last (conflict 2). **Phase 2 (sim 128)**: the shadow orbs (E59: the charged beam and its two capsules, one to three orbs by charge, twice from a reflected beam, five a fighter and five out, the 4-second clock lengthened by later hits to 7.5 at most, bystander hits twice, the strike of 6.5% each away from the owner, reflected or pocketed orbs flying loose) and the barrier (the bubble in front on 7-27, the miss answer on 38-53, the struck answer 2-4% / 6.5-18% read by the caught damage, intangible 1-11, FAF 52, broken above 25%); the down smash's ground test, the slashes' landing hit, the forward air's wall stick (three per air time). [E] phase 2: the beam's speeds and start, the orbit, the extension's share, a loose orb's flight, the barrier's capsule and answer centres, the break's push, the slashes' landing box, the wall test and the wall jump's height. Limits (house): the beam's 2% object-only boxes, the beam passing through projectiles, orbs not popping balloons, the up special's rotating boxes, one get-up slot, the grabs' aerial boxes. |

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
  special-fall drift and landing lag, angling (done, sim 58); rise height (**[E]**, no published figure); Counter's window, answer, 50%
  cap and bubble (done, sim 78, [R]).
* **Claws**: the fire flight's travel and hit sequence (done, sim 58, from the published parameters); the reflector (frames, field
  size, reflected speed and angle: still [E]; E10b's vertical reversal exists but stays off: owner decision 2026-10-13, the Claws' reflector stays horizontal-only);
  the flash dash's distance and end hit ([E]).
* **Needles**: up air drag-down (done, [R]); Flip Kick (done except consecutive kicks and wall bounce); Flash Step (done except wind box,
  which E2's push hit can now build); the grenade (our own line stays, owner's decision 6); wall jump (done, sim 64).

## The seven reference fighters

Extensions are named E1 to E20 here and described once, in "Infrastructure" below. "Exists" means the mechanic is in
`SHARED_MECHANICS.md` and can be reused as is.

### Mario

* **Status (2026-10-13)**: phase 1 (sim 57: the body and every normal) and phase 2 (sims 58 to 64: the four specials and the wall
  jump, with E2, E3b, E10, E10b, E11, E15, E16 and the set-knockback launch rule) done on branch `movesets`; **playable**, not yet
  verified (the rising punch's rise is [E]). Report: `docs/movesets/embers/IMPLEMENTATION_REPORT.md`.
* **Kit mechanics**: three-hit jab (exists: `Move::next`); Fireball (gravity, bounces along the ground several times); Cape (reflects at
  1.5x, **turns opponents around and reverses their control**, a fall break in the air; no armor since an update); Super Jump Punch
  (seven hits 5 / 0.6 x5 / 3, intangible startup, angled 12 degrees by the stick, ledge sweetspot from frame 14, helpless); F.L.U.D.D.
  (charges, stores like Needle Volley, 7 shots, **0% damage push** with set knockback 70, aimable up or down, pushes Mario back when charged,
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

* **Phase 1 built (sim 86 as a placeholder number, branch `bracers-phase1`)**: class 7 Bracers on the `hauler` body, built-in fighter
  Hauler (owner decision 37); every normal on the raw tables' data; E20 trips and E3g no-rebound hits built (decisions 39, 40); tests
  `bracers.rs`, `trips.rs`. Report: `docs/movesets/bracers/IMPLEMENTATION_REPORT.md`. Phase 2 (the specials, E5 bury, E29) next.

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

* **Phase 3 built (sim 87, branch `greatsword-phase3`, merged to main)**: E3e (`ProjectileSpawn::burst`, hashed `Projectile::bursting`), E28 (`ANGLE_VORTEX` 366, hashed `Fighter::link_kind` / `vortex`), E11 `Move::motion_drift`; the powered wave, combo slash, rise and finisher in `ext6`-`ext11`; every special spends the gauge in the powered mode; tests `greatsword_powered.rs` and netplay `powered_rollback.rs`. Owner decisions 42-44 recorded.
* **Phase 2 built (sim 85, branch `greatsword-phase2`)**: the four normal specials and E9 (the Boost gauge), E16 `boost` / `boosting` / `grab` / `boost_add`, E11 `start_speed_y` / `ledge_drop`, E22b `on_land`, E3f `Hitbox::half_height`; tests `greatsword_specials.rs`, `gauge.rs`, `move_rules.rs`. Phase 3 (E3e, E28, the Limit versions) is next.
* **Phase 1 built (sim 83, branch `greatsword-phase1`)**: class 6 Greatsword on the `champion` body, built-in fighter Champion (owner
  decision 30): the body and every normal, `sim-core/src/moves_greatsword.rs`, tests `sim-core/tests/greatsword.rs`; report and the
  phase-2/3 plan in `docs/movesets/greatsword/IMPLEMENTATION_REPORT.md`. The research's proposed names (vanguard, Surge) were replaced by the
  owner's (Champion, the gauge "Boost").
* **Research (2026-10-13)**: `docs/movesets/greatsword/REFERENCE_DATA.md` (the full kit at 13.0.1, labelled, 11 conflicts and 14 gaps) and
  `PROJECT_AUDIT.md` (what exists; the exact shape of E9; new E3e burst-on-contact projectiles, E3f tall projectile hitboxes, E22b
  on-land switch, E28 angle 366; two E11 additions; 12 of 16 `ext` slots; proposed names Greatsword / vanguard / Vanguard, gauge
  "Surge"). Nothing registered. **Corrections to the notes below from that research**: the up air is a single hit in three phases (no
  multi-hit); Climhazzard's meteor box is out from part 2's frame 12, and what waits for a fall distance is the **ledge grab**; "tech at
  the peak" was the earlier game's only; landing lag 26 is part 2's landing (part 1's special fall lands with 20); Limit Climhazzard's
  height multipliers are x1.6 grounded / x1.85 aerial (the "1.5x" is a rounded description); ssbwiki.com has **no Cross Slash hitbox
  table** (dragdown and UFD only), and the Limit Cross Slash's total is a three-way conflict (24 / 26 / 27%).
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
  * **Cross Slash**: per-hit continuation only on hit, input windows, damages, the Limit auto-sequence. Exists (Blade Flurry pattern,
    `hit` register), but the move count needs **E17**.
  * **Climhazzard**: rise height and drift, the second press, the descending meteor's sweetspot only after a fall distance, ledge windows,
    landing lag 26. Needs **E11**.
  * **Blade Beam (Limit)**: multi-hit and passing walls as a hit. Needs **E3a** (multi-hit projectile) and transcendent priority (**E3c**).
* **Gaps**: E2 (Finishing Touch wind boxes), E3a, E3c, E9, E11, E16 (a `gauge`/`limit` register, "was hit during this move"), E17 (four
  normal specials + four Limit specials + Cross Slash follow-ups + Climhazzard's descent do not fit in ten `ext` slots).
* **Cost**: moderate to high (E9 is a new kind of state: a fighter whose movement changes during a match).

### Palutena

* **Status (sim 77)**: **playable**. Phase 1 (sim 67): class 5 Staff on the seer body, built-in fighter Seer (owner decision 19), every
  normal on [R] data, with E26 (angle types, also applied to the older ports) and E27 (signed hitstun modifier). Phase 2 (sims 74-77,
  branch `seer-phase2`): the warp, the counter and reflect barrier (E10c), the explosive flame (E3a, `flick`, `spawn_at`) and the
  reticle (`lock`, `segment_blocked`). **Not verified**: the warp's ledge windows and stage contact (gaps 8, 9), the flame's height
  (gap 5), the reticle's shot path and "in front" rule (gaps 1, 2), the aerial specials' landings (gap 4), and the barrier ending with
  its move (owner decision 20). Report: `docs/movesets/staff/IMPLEMENTATION_REPORT.md`.
* **Research (2026-10-13)**: `docs/movesets/staff/REFERENCE_DATA.md` (the full kit at 13.0.1,
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
    walls, platforms), helpless, the edge cancel, ledge rules. Partly exists (Flash Step: scripted teleport, intangibility, hidden view);
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

* **Status (phase 3 built, sim 110, branch `clutch-phase3`)**: the tongue that lays its catch as an egg (neutral special, E8 built
  general) and a simple rolling shell (side special, decision 49; E31 and E32 still deferred); tests `sim-core/tests/clutch_egg.rs`
  (20), `move_rules.rs`, `bundle.rs`, `state.rs`. Every special is now the Clutch's own. Report: `docs/movesets/clutch/IMPLEMENTATION_REPORT.md`.
* **Status (phase 2 built, sim 95, branch `clutch-phase2`: the lobbed shell and the drop, decisions 58-59; phase 1 below)**: tests
  `sim-core/tests/clutch_specials.rs` (14), `late_jab.rs` (3), `move_rules.rs` (3 more), `scripted_moves.rs` (1 more). Egg Lay (E8) and
  Egg Roll are phase 3 (plan in the report).
* **Status (phase 1 built, sim 89, branch `clutch-phase1`)**: class 8 **Clutch** on the **bounder** body, built-in fighter **Bounder**
  (owner decision 45): the body, the double jump's subtractive knockback armor and turnaround (E33, with the E1 knockback kind built
  general), and every normal on [R] data from ssbwiki.com's raw tables; tests `sim-core/tests/clutch.rs` (25). Specials are still the
  claws' placeholders. Report and the phase-2/3 plan: `docs/movesets/clutch/IMPLEMENTATION_REPORT.md`. **Corrections from the research
  below**: the 13.0.1 double jump is an ordinary jump (no curve: E12 dropped, decision 47); the shield is a regular one (decision 5).
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
  while the specials' distances are [E]. Report: `docs/movesets/gauntlets/IMPLEMENTATION_REPORT.md`; research: `REFERENCE_DATA.md`,
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

* **Status (phase 1 built, sim 91, branch `buckler-phase1`)**: class 9 **Buckler** on the **wanderer** body, built-in fighter
  **Wanderer** (owner decision 53): the body and its passive shield (E14), every sword normal and both up specials (the grounded
  charged single hit and the aerial multi-hit with E34 position vectors, decision 55), E36 (the forward smash's second hit keeps its
  charge), the E11 special-fall speed and acceleration, the `fast_fall` register for the down air's bounce, and **E37** (decision 56:
  every fighter's up special and up smash straight out of a shield). Tests `sim-core/tests/buckler.rs` (30).
  **Phase 2 built (sim 102, branch `buckler-phase2`)**: the bow (E35 charge-scaled arrows, E3b lodging), the boomerang (E3d: one at a
  time, failed throw, smash-input version, catch, clank fall; its flight as a projectile script) and the sword wave (E35 with drag, E38's
  lying capsule). Tests `sim-core/tests/buckler_specials.rs` (20). The bomb is the claws' placeholder until phase 3 (E4 items). Report
  and the phase-3 plan: `docs/movesets/buckler/IMPLEMENTATION_REPORT.md`.
  **Phase 3 built (sim 111, content schema 7, branch `buckler-phase3`, owner decision 64)**: **E4 items** for every fighter (a hashed
  item pool, `ItemSpec` per weapon item, the universal pick-up, throws, smash throws, dash throw, drops and aerial throws, each body's
  release frames, the `holding` register and `detonate()`), the **bomb** (pull, 1800-frame fuse, detonation 12 frames after the press,
  self damage, 50 hp, bounces, anyone can pick it up), **lodged arrows as items** and the **double arrow**; the passive shield off while
  holding. Tests `sim-core/tests/items.rs` (23). Every move of the kit is now built; the iconic-move re-check in the hitbox view is
  next.
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

## The second wave (owner decision 65)

Toward a starting roster of 16 (the last two are the owner's to name later). Names: class / body.

| Reference | In game | Status | Defining mechanics (agent file) |
| --- | --- | --- | --- |
| Villager | **Pockets / Settler** (class 10) | **every move built** (phase 1 sim 113, phase 2 sim 115, `docs/movesets/pockets/`; estimates listed) | the previous version's jab (decisions 66-67, E40); Pocket; the rocket ride; the balloons pop (E41); Timber |
| Diddy Kong | **Popgun / Scamp** (class 11) | **every move built** (phase 1 sim 117, phase 2 sim 119, `docs/movesets/popgun/`; estimates listed) | the banana as an item anyone picks up, trips (E44, E45, E47); the popgun's charge and overcharge; the flip's latch (E49); the barrel jets' charge, steering and falling barrels (E48), the first charge's slow fall (decision 75) |
| (the third; `docs/movesets/dagger/`) | **Dagger / Phantom** (class 12) | **every move built** (phase 1 sim 121: the body, every normal with and without the partner, the gauge and partner (E52, E53), the gun, the curse bolt (E54), the guard (E55), the partner's stance and wings; phase 2 sim 123: the hook and its tether (E56), the gauge from damage dealt; phase 3 sim 126: decisions 84-86) | the gauge and the summoned partner; the gun; the hook; the guard that fills the gauge |
| Sephiroth | **Nodachi / Exile** (class 13) | **kit complete, estimates open** (phase 1 sim 125: the body, every normal, the wing (E57), the blade dash and eight slashes, the flare's three levels (E58); phase 2 sim 128: the shadow orbs (E59), the barrier, the down smash's ground test, the slashes' landing hit, the wall stick; `docs/movesets/nodachi/`) | the comeback wing; the long sword's tip; the orbiting flare; the charged flares; the dash / octaslash; the counter's follow-up |

## Infrastructure: the extensions

Each is the smallest general change that does the job, with a default that leaves existing content and fighters unchanged. **Broad
architectural changes (E4, E7, E9, E13, E17) are explained to the owner before they are built.**

| Id | Extension | Shape (sketch) | Needed by |
| --- | --- | --- | --- |
| E1 | Armor (**built, sim 67**: `Move::armor`, damage-threshold and super kinds; knockback kind reserved) | per-move windows `{start, end, kind, threshold}`: super armor (never flinches), damage threshold (flinches only above N%), knockback subtraction (the reference's knockback-threshold armor: subtract N, flinch with the rest); plus a body field for armor on an air jump. Applied in `combat::apply_hit` before the hitstun state change | DK, Falcon, Yoshi |
| E2 | Wind boxes | a hitbox flag: pushes (sets or adds velocity from its angle and knockback) with no damage, no hitstun, no hitlag, re-applies each frame, pushes shields too if the reference does | Mario (F.L.U.D.D.), Cloud (Finishing Touch), Needles (Flash Step), Link (to verify on the Boomerang's return) |
| E3a | Multi-hit / piercing projectiles (**built, sim 76**: `ProjectileSpawn::pierce`, `rehit`, eight phases; hashed `Projectile::hit_mask`, `hit_age`, `facing`) | a per-projectile hit memory (who it hit, like `hit_mask`), a rehit interval and a final-hit phase, and a flag for "survives a hit" | Cloud, Palutena, Link, Mario |
| E3b | Projectile ground contact | per-spec ground behaviour: bounce (with restitution and a count: built sim 60), stop and lie (**lodge: built sim 102**, `ProjectileArc::lodge` / `lodge_angle`, hashed `Projectile::rest`), or die | Mario, Yoshi, Link |
| E3c | Transcendent priority / no-clank hits | **built (sim 70)**: `Hitbox::clanks` (false: never clanks with attacks or projectiles), also read for projectiles | Cloud; Gauntlets (punch, uppercut), Staff (sim 73) |
| E3d | Projectile lifecycle | **built (sims 102 and 111)**: `ProjectileSpawn::tool` (kind and per-owner `limit`, the `owned` register, the owner's catch from an age with its catch move, `on_clank` fall), `Move::smash_version`; the bomb's long fuse, `detonate()` and self damage (`ItemBlast::hits_owner`) are on the item (E4, sim 111) | Link |
| E35 | Charge-scaled shots | **built (sim 102)**: `ProjectileSpawn::charged` (damage, speed, life, size by the firing charge, snap, floor), hashed `Projectile::damage`, `ProjectileArc::drag` | Link (the arrows, the sword wave) |
| E38 | Lying capsules | **built (sim 102)**: `Hitbox::half_width`, `combat::shapes_touch` (rounded rectangles) | Link (the sword wave) |
| E4 | Held items (**built, sim 111, schema 7**, owner decision 64: `GameState::items`, `Fighter::holding`, `moves::ItemSpec` on `Weapon::items`, `Move::pull` / `spend_item`, `ProjectileArc::lodge_item`, `FighterParams::item_throws`, the ruleset's `item_*`, `sim-core/src/items.rs`, the `holding` register and `detonate()`) | a small item pool in `GameState`: pick up, hold, throw (directions and smash throws), drop, usable by any fighter; catching a thrown item in the air is not built | Link |
| E5 | Bury | a victim state: stuck in the ground for a time from percent, mash to escape, any hit pops it out with the reference's rules | DK |
| E6 | Command grabs (**built, sim 69**: `Hitbox::command`, `grab::catchable`, `Move::catch_move`, `Move::hold_at`) | a grab hitbox that can catch airborne (and, if the reference allows, shielding) targets, holds the victim at a move-defined point instead of the global `grab_distance`, and hands to the move's own sequence (`ext` slots) instead of `Grabbing` | Falcon, Yoshi |
| E7 | Carry | the holder walks, jumps and falls with the held fighter (cargo), with its own speed multipliers, throw slots and the victim's mash escape; walking off stage takes the victim along | DK |
| E8 | Encased victim (**built, sim 110**: `Move::encase`, `EFFECT_ENCASE`, `FighterState::Encased`) | the victim becomes an object (an egg) with its own physics and a mash-out timer from percent, hittable, intangible on escape | Yoshi |
| E9 | Gauge and powered mode (**built, sim 85, schema 6**: `content::Gauge`, `sim-core/src/gauge.rs`, `Move::limit_version` / `gauge_drain`) | hashed `Fighter` fields (a gauge in fixed point, a mode timer), gain hooks on damage dealt and taken, a mode that applies per-body stat multipliers (new content: a "mode" block of multipliers on `FighterParams`, applied where params are read), script registers to read and spend it | Cloud |
| E10 | Reversing hits and full reflection | a hitbox flag that turns the victim round (facing and horizontal momentum) without launching it; reflectors also reverse vertical speed (E10b) | Mario (Cape), Claws reflector check |
| E11 | Per-move recovery rules (**built, sims 58 and 67**: also frame-ranged `air` blocks with an acceleration multiplier, `fast_fall_from`, `turn_at`, `lands_from`) | per move: gravity, fall-speed and air-control multipliers while it runs; special-fall drift and landing lag after it (falling back to today's global/body values); ledge-grab window from a frame, front and back separately | Link, Mario, DK, Cloud, Palutena, Falcon; existing up specials |
| E12 | Air-jump profile | a per-body air-jump velocity curve (the reference's jump that rises slowly then fast) | Yoshi |
| E13 | Shield geometry and per-fighter shields | a shield bubble that shrinks with health so hits can poke past it, per-fighter shield size and model (the egg: whole body, no shrink) | Yoshi, then everyone |
| E14 | Passive projectile block (**built, sim 91**: `FighterParams::passive_shield`, hashed `Fighter::shield_sdi`) | a body option: in given states, a projectile hitting from the front is blocked with a push | Link |
| E15 | Wall jump | a body option and per-body numbers | Mario, Falcon, Needles |
| E16 | Script surface (built so far: `fast_fall` (sim 91), `air_flag`, `presses` / `releases`, `wall`, `flick`, `lock(range)` with `locked` / `lock_x` / `lock_y`, `spawn_at`) | registers: `grab` button, nearest opponent (fighter scripts), once-per-airtime flags, "was hit during this move", "hit a shield"; more fighter variables than 4 | Mario, Yoshi, Palutena, Cloud, Link |
| E17 | More move slots | **built (sim 66, schema 5)**: sixteen `ext` slots, `ext10` to `ext15` numbered 39 to 44 after `pivot_grab` | Falcon (rapid jab now), Cloud, DK, probably Link |
| E19 | Partial intangibility | **built (sim 66)**: `Move::part_intangible` windows naming hurtbox circles (low, middle, high, limb) | Falcon (up tilt, up smash), DK, Yoshi, Needles (up smash), Embers (up smash head) |
| E24 | Per-hitbox shieldstun | **built (sim 66)**: `Hitbox::shieldstun` percent | Falcon (rapid jab x9, dash attack x1.5) |
| E20 | Trip (**built, sim 86**: `Hitbox::trip`, `Fighter::tripped`, `Ruleset::trip_lag` / `trip_max` / `trip_invuln`, rolled on `GameState::rng`) | a hitbox chance that trips a grounded target whose launch would not lift it (deterministic: from the sim's RNG, rolled only for a hit that can trip) | DK (down tilt, forward tilt down), Falcon (forward air late hit), Longsword (down tilt: data only, it lifts); not Palutena (research 2026-10-13) |
| E3g | Clank without rebound (**built, sim 86**: `Hitbox::rebounds`) | the reference's `rebound=f` without `clang=f`: the hit clanks but its owner carries on | DK (smashes; Giant Punch in phase 2), Claws (up smash), Staff (forward, up, down tilts, neutral air) |
| E10c | Counter / reflector routing (**built, sim 75**, with `Reflector::half_height` / `body_*` and counters catching through their own intangibility; sim 78: `Counter::bubble`, pushes never caught) | `Counter::melee_only`, `Counter::max_damage`, `Reflector::detect_then` (a projectile detector that starts another move), `Counter::bubble` | Palutena, Longsword |
| E25 | Reflector left in the stage (**not built**: owner decision 20 keeps the barrier on the move) | a projectile that reflects other owners' shots (`ProjectileSpawn::reflects`), a tall reflector shape | Palutena |
| E26 | Angle types | **built (sim 67)**: `Hitbox::angle_type`: by box side (the default), forward, backward, by position (melee and projectiles); by travel (the reference's type 1) since sim 73 | Palutena; the older ports re-checked (decision 21); the Claws' up special loop (decision 23) |
| E21 | Search boxes | **built (sim 68)**: hit kind `HIT_SEARCH` (touches without hitting, shields included), `Move::on_search`, `Fighter::next_move`; `Move::air_version` | Falcon (Raptor Boost) |
| E22 | Off-edge version | **built (sim 71)**: `Move::off_edge` (a ground move going over an edge switches slot on the same frame) | Falcon (Falcon Kick) |
| E27 | Negative hitstun modifier | **built (sim 67)**: signed `Hitbox::hitstun_add` (i16; a launch keeps 1 frame) | Palutena |
| E34 | Position vectors | **built (sim 91)**: `Hitbox::posvec_x` / `posvec_y` / `posvec_frames`: the victim is carried straight to a point relative to the attacker's feet over N frames (`LINK_POSVEC`, hashed `Fighter::posvec`) | Link (the aerial spin's back hits) |
| E36 | A follow-up keeps the charge | **built (sim 91)**: `Move::keep_charge` (a move reached by `next` or `goto` keeps the charge) | Link (the forward smash's second hit) |
| E37 | Up special and up smash out of a shield or a jump squat | **built (sim 91, owner decision 56, every fighter; the jump squat in sim 96, decision 62)**: in a shield, special with the stick up starts the grounded up special and attack with the stick (or the attack stick) up the up smash, at once; in a jump squat, special with the stick up or an up smash input pressed after the jump's own frame | everyone |
| E39 | Random versions | **built (sim 113)**: `Move::versions` (chance, slot), drawn on the move's first frame from `GameState::rng` (`fighter::draw_version`) | Villager (the turnips) |
| E40 | The previous reference version's hit rules | **built (sim 113, owner decisions 66-67)**: `Hitbox::rules`, the ruleset's `prev_*`, hashed `Fighter::launch_older`; damage, hitlag, rage, tumble hitstun, launch physics | Villager's jab; the Longsword's jab (sim 138, decision 98); others later on the owner's word |
| E41 | Balloons | **built (sim 113)**: `Move::balloons`, hashed `Fighter::balloons` / `balloon_hp` / `helium_used`, `sim-core/src/balloons.rs`, the `balloons` register | Villager (and later any balloon-like flight) |
| E42 | The pocket | **built (sim 115)**: `Move::pocket` / `unpocket`, `Tool::pocket`, `ItemSpec::pocketable`, hashed `Fighter::pocket` / `pocket_item`, `Projectile::stale`, `REST_POCKETED` / `ITEM_POCKETED`, the `pocketed` register | Villager's Pocket (and Isabelle's, if ever) |
| E43 | Placed pieces | **built (sim 115)**: `ItemSpec::placed`, `Tool::water`, `Move::chops`, hashed `Item::water` / `grow_in` / `cuts`, `combat::fire`, `owns(kind)` | Villager's Timber |
| E44 | Slip hits | **built (sim 117)**: `Hitbox::slip` (hard / soft), the ruleset's `slip_lag` / `slip_max`, hashed `Fighter::trip_soft` | Diddy Kong (the peel); any slip move later |
| E45 | Traps: lying items that trip | **built (sim 117)**: `ItemSpec::trap`, `uses` (hashed `Item::used`), `damage_add`; `items::traps` | Diddy Kong (the lying peel) |
| E47 | A move's toss of the item in hand | **built (sim 117)**: `Move::toss`, `items::THROW_MOVE`, hashed `Fighter::pull_wait`, the `pull_wait` register | Diddy Kong (the peel's toss and its 85-frame wait) |
| E48 | A move that sheds a projectile when hit | **built (sim 117)**: `Move::shed` (a flinch ends the move into the special fall); the projectile registers `owner_stick_x` / `owner_stick_y` | Diddy Kong (the rocket barrels flying off) |
| E49 | Latch (a command hold with a clock, mash-out, a repeating hit and button exits) | **built (sim 119, owner decision 74)**: `Move::latch { x height clock_num clock_den }` (the held fighter falls on its own and the holder rides it; the ordinary hold clock with the shared mash when `clock_num` > 0), `fighter::held_fall`; the repeating hit is pummel rows, the exits a script (`jump_tap`, `presses`); `Hitbox::fixed_stun` for the jump-off's 20 frames | Diddy Kong (the monkey flip) |
| E50 | Once-per-airtime marks and flag versions | **built (sim 119, owner decision 75)**: hashed `Fighter::air_marks` (cleared by landing or a ledge, not a hit), script `air_mark` / `set_air_mark`, `Move::flag_versions` (a version, or no start, by air flags or marks) | Diddy Kong (the first rocket charge's slow fall; the kick barring the up special) |
| E51 | Unblockable hits and self damage | **built (sim 119)**: `Hitbox::unblockable` (a raised or passive shield does not stop it), `Move::self_damage` | Diddy Kong (the flying barrels, the crash) |
| E11b | Special-fall and air-block speed and stick acceleration | **built (sim 91)**: `Move::fall_speed` / `fall_accel`, `AirBlock::speed` / `accel_stick` | Link (the aerial spin) |
| E52 | A summoned partner's boxes | **built (sim 121)**: `Hitbox::mode` (any / plain / powered, picked by the hashed `Fighter::move_powered` as the attack began) and `Hitbox::part` (a second attacker's own hit on the same frame); the `powered` register | the thirteenth port (its partner) |
| E53 | The gauge's partner rules | **built (sim 121)**: `Gauge::per_frame`, `behind`, `crowd`, `mode_taken`, `ko_keep`, `keeps`, `halts`; `Move::gauge_gain`; `gauge::after_ko` | the thirteenth port |
| E54 | Damage over time | **built (sim 121)**: `Move::curse` on a projectile's move, hashed `Fighter::curse_*`, `combat::curse_tick`; `Burst::on_shield` | the thirteenth port (the curse bolt) |
| E55 | A guard's armor | **built (sim 121)**: `Move::armor_damage`, `armor_hitlag`, hashed `Fighter::withstood`, the `withstood` register | the thirteenth port (the guard) |
| E57 | A comeback form (the wing) | **built (sim 125)**: `FighterParams::wing` (`content::Wing`: the start table, points, spending, power, jumps, multipliers), `sim-core/src/wing.rs`, hashed `Fighter::wing_points` / `wing_spent` / `hit_by`; `Move::armor_mode` | the fourteenth port (Sephiroth's wing) |
| E58 | Rings: several hits by age at once | **built (sim 125)**: phases sharing a start all out together, the first that touches a fighter hitting it (`moves::rings_at`); `MAX_PHASES` 12 | the fourteenth port (the giant flare's three rings) |

(E18 is not used: Warp's stage collision is research, not yet an extension. E46 is not used either: an item's uses, first planned on
their own, are part of E45.)

## Recommended order

1. **Mario.** Every extension he needs is small and general (E2 wind boxes, E3b ground bounce, E10 reversing hits, E11 per-move recovery
   rules, E15 wall jump, one E16 flag), and each is reused by later ports. E11 is the one that makes iconic recoveries exact, and it can
   be checked first against an existing up special. His charge-and-store is the Needle Volley pattern.
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
28. **Counters ignore pushes**: no counter (the Longsword's included) catches a push / windbox (a flinchless hit), as in the reference.
    Done in sim 78 (branch `counter-fix`): the Longsword's and the Staff's, the only counters.
29. **Counter bubble**: counters catch with the reference's counter bubble (`Counter::bubble`), not the body's hurtboxes. Done in sim
    78: the Longsword's radius 10.5 at 10 up, the Staff's radius 11.5 at 10 up and 1.5 ahead (ssbwiki.com raw parameters, [R]).
30. **Cloud names**: class/weapon **Greatsword**; body and built-in fighter **Champion** (the owner first chose "Fighter", then dropped it as too much rewrite)
    ; the gauge is called **Boost** for now (to be renamed).
31. **The gauge (E9)**: approved as designed in docs/movesets/greatsword/PROJECT_AUDIT.md (schema 6 with a migration), built general.
32. **Gauge gain from damage dealt**: counts the damage actually applied (after staleness and the 1v1 multiplier) [E: unpublished].
33. **Blade Beam's tall hitbox (E3f) and the vortex angle 366 (E28)**: build both 1:1.
34. **Greatsword up tilt sweep**: the early-behind / late-in-front reading stays [E] (owner default for the phase-1 question).
35. **Greatsword down tilt**: the 12-unit slide stays [E] and the back-leg intangibility stays a listed limit.
36. **Get-up attacks**: no second get-up slot for now (one get-up attack per weapon, as today).
(Also, phase 2 default: until phase 3 builds the powered versions, a Greatsword special used on a full gauge keeps the gauge: nothing spends without a `limit_version`.)
37. **Donkey Kong names**: class/weapon **Bracers**, body and built-in fighter **Hauler**.
38. **Bury (E5) and cargo carry (E7)**: both approved as designed in docs/movesets/bracers/PROJECT_AUDIT.md, built general. The bury
    includes the reference's stock-difference terms.
39. **Trips (E20)**: build them (deterministic, from the sim's RNG); also the Longsword's down tilt and the Sprinter's forward-air late hit.
40. **No-rebound hits (E3g)**: build `Hitbox::rebounds` for the moves that clank without rebounding.
41. **Defaults for the other Donkey Kong questions**: phases as the audit suggests (normals; specials with E5 and E29; carry with E7);
    Spinning Kong's windbox immunity is modelled; the aerial spin's "gravity 0.07" is read as the gravity itself [E]; conflicts (cargo
    armor 16 vs 80, carry walk) resolved by the porter with the reasoning recorded; E28 for the spin's first hit once Cloud phase 3 builds
    it (367 until then, labelled).
42. **Greatsword rise height**: keep the fitted Climhazzard rise [E] (fitted to the quickest loop: 60 reference units by frame 19, 66
    at the peak); the powered rise is the same shape at x1.6 / x1.85.
43. **Greatsword charge**: Limit Charge starts filling on its second frame and ends itself on a full gauge [E] (gaps 9-10).
44. **Gauge timer**: the 900-frame powered mode counts during hitlag [E] (gap 12).
45. **Yoshi names**: class/weapon **Clutch**, body and built-in fighter **Bounder**.
46. **The egg (E8)**: approved as designed in docs/movesets/clutch/PROJECT_AUDIT.md (separate from the bury, sharing only the mash helper);
    its unpublished parts (launch direction, fall, the break-out jump) are labelled [E] until footage.
47. **Double-jump armor**: each hit is compared with the 120-knockback threshold on its own (ssbwiki.com's reading). E33 replaces E12
    (the 13.0.1 double jump is an ordinary jump).
48. **Small pieces**: build E30 (Yoshi Bomb drops through platforms) and E3h (both landing stars on the same frame).
49. **Egg Roll is the lowest priority and will likely be replaced**: build it last and simply, from data and existing features only;
    E31 (no hitbox in the air / while turning) and E32 (the ledge stall) are deferred as listed limits, since they serve only Egg Roll.
50. **Base trip chance**: build the reference's base 7% trip for every fighter (a low, non-lifting hit over 55 knockback), rolled from the
    sim's RNG.
51. **Older kits' trip chances**: add them as data from the reference (Claws down tilt 40%, and the slope-only ones).
52. **Small extras**: an `aim` script register (the Bracers' angled forward tilt ends on the reference's FAF 36) and a read-only bridge
    accessor for `Fighter::tripped` (the view shows a trip as sitting).
53. **Link names**: class/weapon **Buckler**, body and built-in fighter **Wanderer**.
54. **Items (decision 3 settled)**: anyone can pick up Link's lodged arrows and his Remote Bomb and throw them, as in the reference (an
    opponent holding the bomb stops him detonating it).
55. **Position vectors (E34)**: build them, so the aerial Spin Attack carries a target hit from behind into the next front hit.
56. **Up special / up smash out of shield (E37)**: add it for every fighter, as in the reference.
57. **Forward air drag (decision 4)**: the reference already drags on hit 1 (the outer and middle boxes against airborne targets are 367
    with set weight); build it as the reference data, no owner change. Link's data conflicts take the research's proposed values.
60. **Yoshi estimates**: estimates are fine for the Bounder wherever the reference doesn't publish a value (the tongue's reach, the egg's
    launch and fall, Egg Throw's arc details, Yoshi Bomb's rise and stall, and the like). Label them [E]; no footage is needed for his gaps.

Recorded by the porter at sim 95 (owner defaults for the Yoshi phase-1 questions; numbered after 57 in the order given, 60 having been
recorded first):

58. **A jab's window past its FAF**: built general (`Move::late_next`): once a jab has ended, a press inside the reference's
    continuable window starts the next hit, not a new jab. Applied to every jab whose reference window runs past its FAF: the Embers'
    jab 1 (to 24) and jab 2 (to 24), the Gauntlets' jab 1 (to 30) and jab 2 into jab 3 (to 30), the Greatsword's jab 1 (to 30, one
    frame past), the Bracers' jab 1 (to 28), the Clutch's jab 1 (to 30) and the Buckler's jab 1 and jab 2 (to 35, sim 96). The Staff's rapid-jab window ends before its FAF; the
    Longsword, Claws and Needles jabs have no published window past their FAF.
59. **A body's air-jump drift**: `FighterParams::air_jump_drift` (percent, default 100); the bounder's 120 (the reference's
    `jump_aerial_speed_x_mul` 1.2). Every other body unchanged.

Owner defaults for the Link phase-1 questions (recorded at sim 96):

61. **The aerial spin on a target in front**: keep the reference's box layout (front hits in front, back hits behind): a target in front
    takes the four front hits and the finisher (5 of 9, 14%), a target behind is carried round to the front by the position vectors.
    [E] until footage of the reference's hitbox view shows otherwise.
62. **Up smash and up special out of a jump squat, for everyone**: as in the reference, a press made during a jump squat (after the
    jump's own frame) of special with the stick up starts the grounded up special, and an up smash input (attack with the stick flicked
    up, the strong button with the stick up, or the attack stick up) the up smash. Built in sim 96 with the shield version (E37); a press
    made with the jump itself stays the short-hop aerial (the house's short-hop input), and attack with the stick merely held up stays
    the short-hop up air.
64. **Items (E4), 2026-10-14**: approved exactly as planned in `docs/movesets/buckler/IMPLEMENTATION_REPORT.md` ("Phase 3 plan"): a
    hashed item pool and `Fighter::holding`; an `ItemSpec` per item kind in content (physics, bounces, pick-up box, throw-speed
    multiplier, the thrown hit and its damage formula, shield behaviour, fuse / life and hp) with a schema bump and migration; the
    universal actions for every fighter on the 13.0.1 rules (pick up; throws in four directions, smash throws and a dash throw, released
    on the body's frames, the wanderer's 7 / 9 / 11 / 6 / 4 / 5, totals 24 / 24 / 23 / 19 / 29 / 23; drops); inputs unchanged for a
    fighter holding nothing; the `holding` register and `detonate()`; read-only bridge accessors. The bomb (1800-frame fuse, detonation
    on command and its 12 frames, self damage, 50 hp, bounces, no pull while holding an item) and the arrows (a lodged arrow is an item;
    holding one the bow fires the double arrow; the passive shield off while holding). Built in sim 111 (schema 7).
63. **Down special on a soft platform, for everyone** (sim 100): as in the reference, a quick down then special on a pass-through
    platform is the grounded down special, not a drop then an aerial special. A down flick keeps the fighter on the platform for
    `Ruleset::platform_drop_frames` (2) before it falls. The 2 frames are derived from the published drop-cancel notes on ssbwiki:
    Terry's drop is 11 frames, 9 of them added, and specials cancel those added frames into grounded versions. A special on the
    flick's frame, either waiting frame or the falling frame comes out grounded. Attacks don't interrupt the drop, because no source
    says they do. The plain drop falls 2 frames later than before. The carry ignores it (the carry allows only throws).

(Phase 2 of the Bracers, sim 90 placeholder: decisions 50-52 built: the base 7% trip for every fighter, the older kits' trip
chances, the `aimed` register (named so because `aim` is a variable in the water pump's scripts), the `fighter_tripped` accessor.)

Owner decisions 2026-10-14 (recorded at sim 113, the Pockets' phase 1):

65. **The second wave**: Villager, Diddy Kong, Joker, Sephiroth, in that order, named Pockets / Settler, Popgun / Scamp, Dagger / Phantom,
    Nodachi / Exile; the last two of the 16 are named later.
66. **Villager's jab is the previous game's (version 1.1.x)**, not 13.0.1's.
67. **The older jab's behaviour is a general content option** (E40, `Hitbox::rules previous`), applied only to moves the owner names:
    Villager's jab now, the Longsword's jab and others later on the owner's say. Its goal: it pops the foe up and outwards so it combos.
68. **Everything else on the Settler is 1:1 with 13.0.1.**

Owner decisions 2026-10-14 (recorded at sim 115, the Pockets' phase 2):

69. **Keep the balloon and rocket-ride estimates** for now (the phase 1 question 1).
70. **Rework the older jab** (the owner's account: two punches that repeat, any action right after either, jab 2 the launcher up and out,
    a slight pull on jab 1 that lets jab 2 combo). Built (sim 115): `Move::cancel_from` (general, default never) lets jab 1 from
    its frame 7 and jab 2 from its frame 8 be cancelled into any standing action at once, the previous version's jab cancel (a crouch
    or shield in the continuable window: ssbwiki.com `Jab_cancel`); jab 1's far glove pulls toward the settler with 13.0.1's own
    far boxes (180 degrees, forward, BK 15, KS 10). **Research finding**: the 1.1.x tables (ssbwiki.com's previous-version pages and
    kuroganehammer.com's 1.1.6 page) have no pull on jab 1 (60 / 80 degrees, BK 10, KS 80): the pull is 13.0.1's jab 1 (and
    13.0.1's earlier versions'), so this is a deliberate blend; see the report's owner questions.
71. **No other 1.1.x rules**: no previous-version shield hitlag, no electric hitstun frame; everything but the jab's own hits stays on
    13.0.1's numbers (as built: E40 is per hit and only the jab sets it).

Owner decisions 2026-10-14 (recorded at sim 117):

72. **Keep the Settler's jab 1 pull** (the 13.0.1 far-box pull on the 1.1.x jab, decision 70's blend). No change.
73. **Raise the Settler's axe early swing** so jab 1 > jab 2 > axe connects as players reported, within reason, damage and angles
    unchanged. Built (sim 117): a second early head box at 12.0 ahead and 17.0 up, radius 6.0 [E]. The string is now a true combo at
    0-60% on the duelist and 0-70% on the settler (was 0-20%); at 130% the foe is about 32 reference units up on the axe's frames, out
    of any reasonable reach (an owner question in `villager/IMPLEMENTATION_REPORT.md`).

Owner decisions 2026-10-14 (recorded at sim 119, the Popgun's phase 2):

74. **Build the monkey flip's latch as a general held state (E49)**: a command hold on the normal grab clock with mash-out, a repeating
    hit, and attack / jump exits; the latch length, scratch rhythm and leap arc may be [E] where nothing is published. Built (sim 119):
    `Move::latch`, the clock the ordinary hold time, the scratch on 11 and 29 of every 48 [E reading of ultimateframedata.com].
75. **Only the first rocket-barrel charge of an air time falls slowly**, as in the reference: a once-per-airtime flag a hit does not
    clear, general and defaulted. Built (sim 119): `Fighter::air_marks`, `air_mark` / `set_air_mark`, `Move::flag_versions`.
76. **The Settler's jab > jab > axe stays at 0-60 / 70%.** Nothing built.

The thirteenth port (class 12 Dagger, body and built-in fighter Phantom): **phase 1 built (sim 121, branch `dagger-phase1`)**: the body,
every normal with and without the partner on 13.0.1 data, the gauge and partner (E52, E53), the gun, the curse bolt (E54), the guard (E55),
the partner's counter / reflector stance and wings; tests `dagger.rs` (11), `dagger_gauge.rs` (9), `dagger_specials.rs` (12). The hook (the
plain up special, a tether) is phase 2. Estimates: the gun's band distances, the partner's arrival / leaving (not modelled), the bolt's
path, the wings' flight, every box on a bone. Phase 1's questions were answered by decisions 80-82 (below); phase 2's open questions are
in `docs/movesets/dagger/IMPLEMENTATION_REPORT.md`.

Owner decisions 2026-10-14 (recorded at the Dagger's phase 1; each confirms what sim 119 built, so nothing changed):

77. **The Scamp's flip kick has no trip.** As built.
78. **A hit clears the kick's block on the up special** (the kick's bar is an air flag, which a hit clears). As built.
79. **Keep the rocket crash's 5% self damage.** As built.

**Phase 2 built (sim 123, branch `dagger-phase2`)**: the plain up special, the hook (the grounded grab and throw, the aerial hit and its
once-per-air-time rise) and its tether (E56, general: `Move::tether`, `FighterState::Tether`); the gun's dodges staled like rolls and the
buffered dodge 4 frames later; angle 365 (general: `ANGLE_HALF_LINK`); the up smash's arm intangible with the partner (a part
intangibility window's `mode`); the gauge from damage dealt. Tests `dagger_hook.rs` (11) and new cases in `dagger_gauge.rs`,
`move_rules.rs` and sim-content.

Owner decisions 2026-10-14 (answering the Dagger's phase 1 questions):

80. **Damage dealt fills the phantom's gauge as well as damage taken**, 1.3 a percent, without the stock-gap or crowd multipliers: a house
    rule, not the reference's [E]. Only the phantom's and its partner's hits on fighters count (not shields, not the curse's damage over
    time), and nothing while the partner is out. Built (sim 123): the gauge's existing `per_dealt` set to 1.3 for the phantom (still 0 by
    default; the champion's 0.667 unchanged); the curse's ticks no longer count as dealt for anyone (`gauge::on_damage_over_time`).
81. **The partner's arrival and leaving are close to instant**: not modelled (the partner is there the frame the gauge fills and gone
    the frame it empties). A decision, not an estimate.
82. **Keep the gun's band distances as estimates** (20 / 44 / 72 of the reference's units). As built.

Owner decisions 2026-10-14 (answering the Dagger's phase 2 questions; built in phase 3, sim 126, branch `dagger-phase3`):

83. **Keep the tether's estimates** (reach 40 up / 16 in front of the reference's units, hang 8, reel 12). No change.
84. **A shield ends the grounded hook sooner**, as the reference's cancel on contact: built general (`Move::shield_end`, hashed
    `Fighter::move_end`); the hook ends 20 frames after a shield or reflector stop (FAF 41 for a stop on 20) [E: no frame published].
85. **Staleness lengthens the gun's dodges** as it does rolls: built general (`Move::stale_stretch`, hashed `Fighter::move_held`): the
    whole move plays slower by the ruleset's roll rate (forward) or back roll rate (back) per level [E: which rate].
86. **A press of special while hanging from the hook reels in at once** (a house addition, not the reference's): built general
    (`Tether::reel_press`, off by default), on for the phantom.

**Phase 3 built (sim 126, branch `dagger-phase3`)**: decisions 84-86 above; tests `dagger_phase3.rs` (5), cases in `move_rules.rs`,
`state.rs` and sim-content. No open questions from phase 3 beyond footage for its two estimates.

Open for the owner from the Popgun's phase 2 (sim 119): footage for the flip's arc and the latch (decisions 77-79 answered the rest).

Answered (decisions 74-76) from the Popgun's phase 1 (sim 117; details in `docs/movesets/popgun/IMPLEMENTATION_REPORT.md`): the monkey
flip's latch as a general held state (E49) and footage of it; footage to fit the rocket's flight and the peel's and peanut's arcs; a
once-per-air-time flag a hit does not clear (the rocket's first-charge-only slow fall).

The fourteenth port (class 13 Nodachi, body and built-in fighter Exile; Sephiroth): **phase 1 built (sim 125, branch `nodachi-phase1`)**:
the body, every normal, the wing (E57, general: a body's comeback form with its start table, points, power, jumps, multipliers and
armor by mode), the blade dash and its eight slashes (up special; `Move::edge_stop`), the flare in three levels (neutral special; E58
rings for the giant flare's burst). Tests `nodachi.rs` (9), `nodachi_wing.rs` (7), `nodachi_specials.rs` (11). Phase 2: the shadow orbs
(side special: orbs stuck on a target, orbiting and striking on a timer, hitting bystanders; not the curse's damage over time, which has
no orbs: a new held-projectile mechanic) and the barrier (down special: a counter answering whether struck or not, its damage
interpolated on the struck hit's, breaking above 25%); the down smash without ground contact; the forward air's wall stick.

Owner decisions 2026-10-14 (answering the Nodachi's phase 1 questions; all confirm what was built):

87. **The forward smash's 13.5% root box** stays ranked last: the 20% blade middle wins over it.
88. **Keep the travel-speed and giant-flare charge-end estimates** (the blade dash, the eight slashes, the three flares' flights, the
    giant flare's frame 108).
89. **The wing with several opponents** is measured against the leader (most stocks). As built.
90. **The KO credit for the wing's points**: the last fighter to deal damage, no time limit. As built.
91. **The flare's charge** stands on the ground and drifts as usual in the air. As built.
92. **The shadow orbs as a general mechanic**: approved, but the owner sees reuse mainly as a run-mode power-up, not another fighter;
    kept general but no bigger than the exile needs (no speculative options). Built that way in phase 2 (E59, `Move::orbs`).

**Phase 2 built (sim 128, branch `nodachi-phase2`)**: the shadow orbs (side special; E59 `Move::orbs`, `sim-core/src/orbs.rs`: the
charged beam with `ChargedShot::every_hit`, orbs on the fighter it hits with the reference's counts and limits, the clock and its
extension and cap, circling bystander hits, the strike, reflected or pocketed orbs flying loose, a KO clearing them), the barrier (down
special: a counter bubble in front, the miss answer at 38-53, the struck answer read between its minimum and maximum by the caught
damage (`Move::lerp`), no hitlag for the exile (`Counter::still`), the break above 25% (`Counter::break_above` / `break_move`)), the
down smash without ground contact (`Move::probe`), the eight slashes' landing hit (`landing_move` and `Move::fall_landing_move`) and
the forward air's wall stick (the script function `ground()`, three per air time, jump or let go). Tests `nodachi_phase2.rs` (13),
cases in `move_rules.rs`, `scripted_moves.rs`, `state.rs` and sim-content.

Open for the owner from the Nodachi's phase 2 (details in `docs/movesets/nodachi/IMPLEMENTATION_REPORT.md`):

1. **The orbs' unpublished numbers**: the orbit (12 of the reference's units round the chest, a turn a second), the extension's
   share of the charge (linear between 0.5 and 3 seconds), all the orbs on a fighter striking together, a loose orb's flight (the
   beam's speed, 60 frames). Keep these estimates until there is footage?
2. **The barrier's places**: the catching capsule (7.7 ahead, 1.8 to 20.2 up, radius 5.9) and the answer's centre (14 ahead when not
   struck, 19 when struck, 10.5 up) were read from the reference's hitbox animations; the break's push back (5 units) is a guess. Keep?
3. **The wall stick's jump**: the published "initial vertical speed 6.9" is read as a jump 6.9 units high (a small hop); as a speed it
   would be far above any jump in the game. Keep the small hop, or is there footage?
4. **The slashes' landing hit** is built for both landings (during the move and in the special fall after it), its box (radius 8 at the
   feet) a guess. Keep?

Owner decisions 2026-10-14 (answering the Nodachi's phase 2 questions above, and a playtest finding):

93. **Keep the exile's orb estimates** (the orbit, the extension's share of the charge, the orbs striking together, a loose orb's
    flight). As built.
94. **Keep the barrier's positions and its break push** (the catching capsule, the answer centres, the 5-unit push). As built.
95. **The wall stick's 6.9 stays read as a hop height** (a small hop), not a speed. As built.
96. **Keep the eight slashes' landing hit on both landings** (during the move and in the special fall after it). As built.
97. **Up specials halt the momentum they start with, as the reference does** (sim 130, branch `upspecial-momentum`). The owner found a
    "super-jump": the exile's blade dash started just after a jump carried the jump's rise through its startup and went more than twice
    as high (21.4 world units against 9.4 from a standstill). The cause: the dash's script zeroed the speed on its frame 0, which never
    runs (a move's first frame in `attack()` is frame 1), so the 23-frame startup stall kept the jump's upward speed. Every built-in up
    special was checked (the table and sources are in `SHARED_MECHANICS.md`, "Starting momentum of specials"): the Longsword, Embers,
    Gauntlets, Staff, Greatsword (both rises), the Dagger's wings and the Nodachi now set `start_speed_y` 0; the Claws, Needles,
    Bracers, Buckler, Pockets and Popgun already halted or set their vertical speed; the Clutch keeps its published 0.8; the Dagger's
    aerial hook (an attack, not a recovery, with no special fall) is unchanged pending the owner (question below). Every up special that
    should leave its user helpless does; nothing lets the exile act after its up special but landing or a ledge (tested).

Owner decision 2026-10-17 (built in sim 138, branch `longsword-jab`):

98. **The Longsword's jab is the previous reference version's (1.1.x)** (decision 67 named it as a later candidate). The owner's account:
    two hits that repeat, any action right after either, jab 2 the one that sends up and out, a slight pull on jab 1 that lets jab 2
    combo. Built (`moves::longsword_older_jab`, tests `sim-core/tests/longsword_jab.rs`), on E40's `rules previous` for these hits only:
    * **Data [R]** (ssbwiki.com `Marth_(SSB4)/Neutral_attack/Hit_1` and `/Hit_2`, raw, with the 1.1.4 changes; kuroganehammer.com
      `Smash4/Marth` "[1.1.6]"; the two agree). Jab 1: frames 5-7, the blade (id 0) 3% at 78 degrees, the arm (id 1) 3% at 65, the
      tip (id 3) 5% at 90, all BK 55, KS 30, hitlag x0.7 (the tip x1.25); earliest continuable frame 18; FAF 26. Jab 2: frames 3-5,
      4% / 4% / 6% (tip) at 45 degrees, BK 62, KS 75, hitlag x0.7 / x1.25; FAF 29; the last hit (no continuable frame, no jab 3).
    * **The pull**: the 1.1.x tables have none (the tip is 90 degrees). As for the Settler (decision 72), the tip takes 13.0.1's jab
      1 tip pull (ssbwiki.com `Marth_(SSBU)/Neutral_attack/Hit_1`, id 2: 180 degrees, forward type, BK 30, KS 12 [R]) on 1.1.x's 5%
      and rules. 13.0.1's id 2 is "fighters only" with a 361-degree twin for the rest; ours hits everything [E].
    * **The cancel** (`Move::cancel_from`): jab 1 from its continuable frame 18 [R: the previous version's jab cancel opens there];
      jab 2 from 16 [E: it has no continuable frame; 16 is 11 frames after its last active one, as 18 is after jab 1's 7]. A jab press
      in either window starts the other hit, so the two repeat (jab 2 > jab 1 stands for the reference's crouch-cancel > jab 1).
    * **Positions and radii [E]**: the sword kit's slash path at the longsword's reach plus an arm box (radii 7 / 8 / 6 tenths).
    * **What it does** (the duelist, weight 90, and the rogue; follow-ups started on the first free frame, `print_the_jabs_frame_advantage_and_follow_ups`):
      frame advantage after jab 1 (near) 11 frames at 0% to 17 at 150%. **Jab 1 > jab 2 is a true combo at 0-150%** on both. Out of
      jab 1, true combos on the duelist: forward tilt, jump > neutral air, jump > forward air and the up special at 0-150%; up tilt at
      40-110% (late below 40); down tilt at 0-20% (0-150% on the rogue); dash attack at 90-150%. The grab and the up smash miss the
      popped-up foe. **Jab 2 is the launcher**: 18 frames of advantage at 0% to 48 at 150%, but the foe is up and out of reach of
      every grounded follow-up (only a dash > jump > forward air catches it, at 0%), as the version's "no longer punishable on hit"
      launcher. **At the tip** (3.5 ahead, only the tip reaches): the pull leaves 0-4 frames, so jab 2 reaches the foe at every
      percent but is a true combo only at 90-150%.
    * Engine tests that used the old placeholder jab as a generic weak hit now ask for it (`common::Sim::plain_jab`, used by
      `Sim::duel` and three other tests); the kit's three old jab tests in `kits.rs` were replaced by `longsword_jab.rs`.

Open for the owner from decision 98: answered by decision 101 (keep the tip pull; jab 2 acts from frame 16).

1. **The tip pull against the late window**: 13.0.1's pull (BK 30, KS 12) was made for 13.0.1's jab 2 window (from frame 11); on
   1.1.x's frame 18 it leaves only 0-4 frames, so tip > jab 2 combos only from 90%. Without the pull (pure 1.1.x, the tip at 90
   degrees BK 55) the tip leaves 12-22 frames and tip > jab 2 combos at 0-20% on the duelist, 0-110% on the rogue (measured). Keep
   the pull, or use the pure 1.1.x tip?
2. **Jab 2's cancel frame** (16) is ours; the reference's jab 2 has no window. Keep it, or cancel only from its FAF (29)?

Open for the owner from decision 97: answered by decision 99 (the aerial hook halts).

99. **Recoveries and air movement brought to the reference; the aerial hook halts momentum** (sim 139, branch `recovery-audit`;
    side specials added to the scope by the owner). Every body's air numbers were measured and checked against its reference
    character's parameter file (dragdown.wiki, cross-checked with ssbwiki.com's attributes): air speed, both air accelerations,
    friction, gravity, fall and fast fall, full hop, short hop and midair jump heights (`air_movement_reference.rs`). Fixed: the
    Claws' air friction (0.01, now the reference's 0.004); the midair jump's sideways share (`jump_aerial_speed_x_mul` 0.9) was
    missing on the Claws, Embers, Gauntlets, Staff and Pockets bodies; and the ground jump's horizontal speed now follows each body's
    `jump_speed_x` / `jump_speed_x_mul` / `jump_speed_x_max` instead of one shared estimate (0.85 of the ground speed, capped at 1.25x
    the air speed, no stick share): a standing jump with the stick held moves at once, a run jump keeps 70-90% of the run, capped at
    the body's own maximum (the Gauntlets' 2.1, so its run jumps carry much further; the Longsword's 1.2). How the reference combines
    the stick and the run is not published: the reading (the faster of the two the same way; a stick held back never cuts a reverse
    aerial rush) is an estimate. The Dagger's aerial hook now halts a jump's rise (`start_speed_y` 0), as the owner decided. The Needles' up special's
    special fall drifts at its published 0.6 (it had the shared 0.5). Side
    specials: the Longsword's blade flurry now keeps half its sideways speed and adds the published 1.4 upward in the air (every use; it
    slows a fall); the Claws' flash dash keeps 0.67 sideways and none vertical, holds still 13 frames then falls at 0.017, angles at
    most 10 degrees past half tilt (it was about 24), and slides on after the dash at 2.3 (air) / 2.5 (ground) slowed by 0.07 / 0.15
    with 0.08 gravity from 5 frames after (it stopped dead). Every other side special was checked and already matched what is
    published (the table is in `SHARED_MECHANICS.md`, "Starting momentum of specials"). Recovery rises driven by the reference's
    animation (most up specials) still have no published height; they stay estimates.

Open for the owner from decision 99:

1. **The ground jump's stick share**: the reference publishes `jump_speed_x` but not how it adds to a run's kept speed. Built: the
   faster of the two the same way, a stick held against the run keeping the run's share. If a standing jump now feels too quick
   sideways, the alternative is no stick share (the older feel, with the body's own run share and cap).
2. **The blade flurry's lift on every use** (no once-per-airtime limit is published): repeated uses keep slowing a fall. Limit it to
   once per air time?
3. **Footage** would settle the animation-driven recoveries (most up specials' heights, the Gauntlets' and Embers' especially) and the
   flash dash's own dash speed (5.5 units a frame, an estimate).

Answered by the owner on 2026-10-17 (decision 100): 1. a run jump takes the run speed (built, switchable); 2. the lift on every use
with diminishing returns (built); 3. no footage is coming, the estimates stay.

100. **A run jump takes the run; the blade flurry's lift diminishes** (sim 139 still, unreleased; branch `air-tweaks`; the owner's
    answers to decision 99's questions).
    * **The ground jump**: a jump with ground speed takes the run's kept share (`jump_speed_x_mul` of it, capped by the body's
      `jump_speed_x_max`) whatever the stick; a standing jump (no ground speed) moves sideways at the stick's `jump_speed_x`, as
      published. Reversible: the ruleset's `jump_stick_share` (0, the standard, this rule; 1, decision 99's "faster of the run's share
      and the stick's the same way"), validated (0 or 1), hashed, in `docs/CONTENT.md`, round-trip tested
      (`the_jump_stick_share_switch_round_trips_and_is_checked`), and tested both ways in `air_movement_reference.rs` (a full run
      with the stick forward, neutral and back; a slow walk with the stick full, where the two settings differ).
    * **The blade flurry's air lift** (the Longsword's side special): every use lifts, each further use in the same air time half the
      one before: **1.4, 0.7, 0.35, 0.175 reference units, then none** [E: no source describes a decay; halving is the owner's
      example]. Counted in the once-per-airtime flags 0-3 (`air_flags`, which landing, catching a ledge or being hit clear, as the
      cape's lift and the shell's hops). Tests: `sidespecial_momentum::the_blade_flurrys_lift_halves_with_each_use_in_an_air_time`,
      `the_blade_flurrys_count_builds_in_the_air_and_resets_on_landing`.
    * **No footage is coming**: the animation-driven recovery heights and the flash dash's 5.5 a frame stay estimates.

101. **The Longsword jab stays as built** (owner, 2026-10-17; no behaviour change, sim 141): jab 1's tip keeps the 13.0.1 pull (BK 30, KS 12, so tip > jab 2 combos from 90%), and jab 2 can act from its frame 16 [E].
## Still open

From the Embers' phase 2 (details in `docs/movesets/embers/IMPLEMENTATION_REPORT.md`, "Owner follow-up"):

1. Up-special rise heights (the rising punch, the cape's lift, the Longsword's): re-searched 2026-10-13, still no published figure in
   units (list of sources in `docs/movesets/embers/REFERENCE_DATA.md`); footage would settle them.

Answered by the owner on 2026-10-13 (sim 65): a full water-pump charge holds the stance until special or shield (built); reflectors
stay as they are, the cape turning a shot's vertical speed round and the Claws' not (meant to differ); the jump button against a
wall also wall-jumps, stick neutral or toward the wall, for every wall-jumping body, without spending a midair jump (built).
