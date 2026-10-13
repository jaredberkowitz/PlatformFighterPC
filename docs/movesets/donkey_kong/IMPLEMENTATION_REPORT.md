# Implementation report: the eighth moveset (reference: the reference game's Donkey Kong)

In game: class 7 **Bracers** (weapon `bracers`), body `hauler` (`FighterParams::hauler`), built-in fighter **Hauler** (owner decision
37). Target version 13.0.1. Data: `REFERENCE_DATA.md`; what the engine had and needs: `PROJECT_AUDIT.md`; owner decisions 37-41 in
`docs/movesets/ROSTER_STATUS.md`.

## Phase 1 (sim 86, a placeholder number to renumber at the merge; content schema 6): the body and every normal

Branch `bracers-phase1`, from `movesets` at 5e7bcb3, then merged up to f2b69b8 (Cloud phase 2, sim 85, schema 6). Sim 84 is the engine
session's; this phase takes 86.

### Engine (two small general extensions, approved by decisions 39 and 40)

| Id | What | Where | Defaults (every older file and fighter unchanged unless its data uses it) |
| --- | --- | --- | --- |
| **E20 trips** | `Hitbox::trip` (0 to 100, the reference's `trip` percent; content `trip`, validated). In `combat::apply_hit`, after the armor test: a fighter on the ground, not already lying down, whose launch would not lift it (a world launch angle with no upward part: the engine lifts a grounded fighter on any upward angle) rolls `GameState::rng.range(100) < trip`. On a success `fighter::trip` sits it down: a knockdown with `Fighter::tripped` (hashed, a checksum case), the hit's damage and hitlag, intangible through the hitlag and `Ruleset::trip_invuln` (5) frames after, get-ups from frame `trip_lag` (40), a neutral get-up by itself on `trip_max` (100): the reference's hard trip (ssbwiki.com's trip pages, 13.0.1). A tripped fighter can be grabbed (the engine's knockdown already can, as the reference's sitting fighter). The flag clears on the next state change. The generator was already in `GameState` and hashed, and nothing is rolled for a hit that cannot trip, so a match where no trip-capable hit lands is frame for frame what it was | `moves.rs`, `combat.rs`, `fighter.rs`, `state.rs`, `content.rs` (Ruleset), `sim-content` format and validation | trip 0; ruleset 40 / 100 / 5 |
| **E3g no rebound** | `Hitbox::rebounds` (content `rebounds`, default true). In `combat::clank_attacks` a side only rebounds if one of its clanking boxes rebounds; otherwise the clank stops its hit for that frame and it carries on with the move. `clank_projectile` likewise does not knock such an attack into a rebound | `moves.rs`, `combat.rs`, `sim-content` format | true |

Both are optional content fields with defaults; schema 6 is not released yet, so they ride in it (`docs/CONTENT.md` says so: if schema
6 ships first, they belong in a schema 7 with a migration that rewrites nothing).

### Built

* **Body** `FighterParams::hauler` (`sim-core/src/content.rs`), every value from `REFERENCE_DATA.md` "Attributes" [R]: weight 127; walk
  1.365, initial dash 2.09 for 15 frames, run 1.873, traction 0.123 (`ground_friction` twice it, `dash_attack_friction` it); walk
  acceleration 0.105 + 0.105, run acceleration 0.044 + 0.07337; heavy landing **3** frames of lag (`landing_heavy_frame` 4, the house
  rule); air speed 1.208, air acceleration 0.01 + 0.05, air friction 0.015, gravity 0.085, fall 1.63 / fast fall 2.608; full hop 34, short
  hop 17.3, double jump 35.5, ledge jump 37 (drift 0.6, the shared value); air dodges 48 (intangible 4-32) and 66 / 82 / 109 (4-22); rolls
  30 / 35, spot dodge 26; ledge option intangibility 80% / 48% on the second and third grab; **no wall jump**, the wall-tech jump 1.3 /
  2.7. A bigger body [E]: ECB 1.0 half width and 2.8 tall (the shared 0.8 / 2.2), hanging 1.1 from the wall (the validator requires the
  hang to clear the body).
* **Every normal** (`sim-core/src/moves_bracers.rs`, a child module of `moves.rs`), every row read from ssbwiki.com's **raw** hitbox
  tables (re-fetched 2026-10-13 and compared with `REFERENCE_DATA.md`: they agree, and the raw tables settle the forward tilt's trip as
  the down-aimed version's only), on [R] frames, damage, angles and angle types, BK, KS, set knockback, hitlag multipliers, shield damage
  (`sd` as a percent [D]), hitstun modifiers, trip chances, no-clank and no-rebound boxes. Root-bone (`top`) offsets converted by `u`
  (x 1.25, rounded half away from zero) [D]; reference capsules as circles along them [D]; boxes on the arm, leg, hip and head bones at
  the published distance along the bone [R] from a joint placed on our body for the pose [E] (`limb`). `Weapon::limbs` is true (bare
  fists), so the E19 limb circle is a real forearm or head here and the arm windows act.
  * **Jab 1** (5-6, 4%, hitlag x1.5, angle type 3: two 361 boxes and the fist pulling in at 180, FAF 25), carrying on into **jab 2**
    from its frame 8 (the continuable window 8-28); **jab 2** (4-6 in 13.0.1, 6% at 70, BK 40, KS 100, x1.8, FAF 32) ends the chain: the
    quickest loop runs 5-6, 11-13, FAF 25 / 39 tick for tick.
  * **Forward tilt** 7-9 (361, BK 50, KS 78), aimed straight 8%, up or down 9%, the down one with a **15% trip**; the lower arm
    intangible 6-9; FAF 35. **Up tilt** 5-11 (100 degrees by position, BK 40; shoulder 8% KS 105, forearm 9% KS 110, fist 10% KS 115),
    the arm swept from in front over the head to behind [E], the lower arm intangible 5-11, FAF 39. **Down tilt** 6-7 (6%, 361 by
    position, BK 10, KS 80, **+5 hitstun**, **40% trip**), the whole arm intangible 5-7, FAF 25. **Dash attack** clean 9-12 (12% at 53, BK
    85, KS 46), late 13-24 (9% at 45, BK 60, KS 60), x1.25, FAF 35.
  * **Smashes**, every box clanking **without rebounding** (E3g): forward 22-23 (22% at 46 / 21% at 361, x1.4, shield +1, charges on
    14, head and lower arms intangible 20-26, FAF 55); up 14-15 (19% at 90 by position, shield +4, charges on 6, head and lower arms
    12-15, FAF 50); down clean 11-12 (fists 17% at 70, shoulders 14% at 82), late 13-14 (the shockwave 18% at 55 in front and behind,
    shoulders 14% at 98), one hit, by position, shield +1, charges on 2, lower arms 11-14, FAF 56.
  * **Neutral air** clean 10 (with the far box, 6.0.0) and 11-13 (12%, BK 20), late 14-26 (9%, BK 10), by position, landing lag 10,
    autocancel 1-9 / 27+, FAF 39. **Forward air** clean 18-20 (16% at 361, BK 20, KS 85), late 21-23 with the **fist a meteor** (15% at
    270, BK 30, KS 100, x1.5, on grounded targets too) and the forearm 13%; landing lag 17, autocancel from 56, FAF 56. **Back air** clean
    7-8 (13%, BK 20, KS 97), late 9-16 (8%, BK 10), always **backward** (type 4), landing lag 11, autocancel 1-6 / 31+, FAF 32. **Up air**
    6-10 (13% at 90, BK 32, KS 90), the head intangible 5-10, landing lag 15, autocancel 1-5 / 26+, FAF 38. **Down air** 14-16, the
    **feet a meteor** (16% at 270, BK 30, KS 90, everyone) and the leg 13% at 361; landing lag 14, autocancel 1-2 / 50+, FAF 55.
  * **Grabs** standing 8-10 / 39, dash 11-13 / 47, pivot 12-14 / 42 (the grounded-target box). **Pummel** 1.6% on frame 2 (FKV 30, x2.3,
    no clank; the reference's hit is already on frame 2, so no house shift), FAF 8. **Throws** (no pre-hits): back (turns and throws on
    15, 11% at 40, BK 60, KS 65, FAF 40), up (14, 9% at 90, BK 70, KS 50, FAF 44), down (19, 7% at 361 along the ground, BK 60, KS 50,
    FAF 55). **Forward throw: a placeholder** plain throw with the forward cargo throw's numbers (15, 12% at 60, BK 90, KS 46, FAF 46)
    [E] until the carry.
  * **Ledge attack** 23-25 (10% at 45 forward, BK 90, KS 20, shield +1, no clank, intangible through 25), FAF 56. **Get-up attack** face
    up: behind 18-19, in front 23-24 (7% at 48 by position, BK 80, KS 48, shield +8, no clank, intangible through 24), FAF 46.
* **Specials**: the claws' placeholders (as every class's phase 1). No `ext` slot is used by the normals.
* **Older fighters changed** (decisions 39, 40; from ssbwiki.com's raw tables, every older class's grounded normals and aerials scanned
  for `trip=` and for `rebound=f` without `clang=f`):
  * **Gauntlets** forward air late hit (15-30): **20% trip** (361 at low knockback: it can trip a grounded target now).
  * **Longsword** down tilt: **35% trip** as data; at 30 degrees it lifts the target, so it never trips on the flat stages (the reference
    says the same: "only on slants").
  * **Claws** up smash: every box clanks without rebounding.
  * **Staff** forward, up and down tilts and neutral air: every box clanks without rebounding (the neutral air is an aerial, which never
    clanks here, so only its data changes).
  * Nothing else in the older kits has `rebound=f` alone (the Embers' forward smash and the Longsword's up smash have it only together
    with `clang=f`).
* **Registered**: `CLASSES` 8 and `Recipe::params` (`sim-content/src/recipe.rs`), `Content::placeholder` (fighter `hauler`, weapon
  `bracers`), `godot/scripts/roster.gd` (`CLASS_NAMES` "Bracers", the audit's blurb, the built-in Hauler with `Loadout.default_for(7)`),
  `content/base.pfc` re-exported (sim 86). Count tests updated: sim-content `doc.rs`, `recipe.rs`, `tests/bundle.rs`, `tests/recipes.rs`
  (`MADE` 8); netplay `tests/fighters.rs` (class 8 is the unknown one); godot `content_e2e.gd`, `creator_flow_test.gd` ([0..7]),
  `editor_api_test.gd` (the made fighter is index 8), `online_fighters_test.gd` (ten and eight fighters). `ground_attributes.rs`: the
  hauler added; the "no two fighters" check now allows a shared initial dash speed (the brawler and the hauler both dash at the
  reference's 2.09) but still not a shared run speed. `angle_types.rs`: 16 Bracers moves (108 checked).

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/bracers.rs` (17 tests): every normal's windows, FAF, landing lag and autocancel in the data, charge frames, the
continuable window, no rehit; the arm and head intangibility windows frame by frame on the real limb; first-hit frames (14 moves) and
FAF (every move but the throws and the ledge attack) by running each move; damage / angle / BK / KS of 37 boxes, the hitlag multipliers,
every move's angle type, the forward tilt's aims, damages and trip, the down tilt's +5 hitstun and 40% trip (and no other trip or
hitstun change in the kit), shield damage; the no-clank and no-rebound sets; the quickest jab chain tick for tick (5-6, 11-13, FAF 25 /
39, no third hit); the forward air's late fist and the down air's feet straight down, the clean forward air up and forward; the back
air backward and the down smash away on both sides; the standing grab on frame 8; each throw's release frame and damage, the back throw
behind with the thrower turned; the pummel; aerial landing lag and autocancels; jump air times (40 / 53 / 27 / 37 within a frame);
the body's attributes; registration as class 7 with the claws' specials; random play deterministic and rolling back.

`sim-core/tests/trips.rs` (7 tests): a sure trip sits the target down with the damage and no launch, and no chance launches as usual;
intangible through the hitlag and on the sit's frames 1-5, get-ups from frame 40, standing up by itself on 100; the 40% chance over 60
seeds trips some and not others, each seed always the same, exactly one roll per trip-capable hit and none for a hit without a chance;
a hit that lifts (a high-percent down tilt, the Longsword's 30-degree down tilt at 100%) never trips and rolls nothing; the older kits'
trip data; two no-rebound forward smashes that meet clank, neither hits and both carry on (and made to rebound, both rebound: the
setup does reach the clank); the older kits' no-rebound data. Also `state::tests::checksum_sees_every_field` (`tripped`), and the unit
test `moves::every_weapon_has_every_move_slot` (now includes the bracers).

### Estimated or limited [E]

* **Positions on non-root bones** [E]: every arm, shoulder, hand, knee, foot, hip and head box. The joint and limb direction of each
  pose are in the code comments; the distance along the bone is the reference's.
* **The body's size** [E] (gap 15: no published hurtbox numbers): 1.0 half width, 2.8 tall.
* **The angled forward tilts' FAF**: the reference's 36 (one frame longer than straight, "an oversight") cannot differ inside one move
  here; all three end on 35. An `aim` script register (or a per-aim FAF) would make it exact; not built to keep the parallel branch's
  register numbering clear (question 2).
* **Trips**: the reference's base 7% trip on any non-lifting hit over 55 knockback is not built (it would change every fighter: question
  1); slopes do not exist; the trip get-up options are the knockdown's (the reference's trip get-up attack, 5% with intangibility 1-7
  and FAF 50, has no slot; the neutral get-up and rolls after a trip use the shared ones).
* Not modelled (house limits, as for every port): jab 1's objects-only box (id 3), the grabs' thin aerial-target boxes, the throws'
  "interrupted" rows and the thrower's invincibility, the face-down and trip get-up attacks (one slot), the shield's 7.0.0 head
  intangibility.

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace --no-fail-fast` (all green); `cargo clippy --workspace --all-targets -- -D warnings`
(clean); `pftool fuzz-rollback 1000` (0 mismatches); `pftool net-fuzz 200` (0 mismatches); `pftool content-check content/base.pfc`
(ok); the content re-export equals the builders (`base.pfc` equality test). **Not run**: the Godot tests (`godot/tests/*.gd`): updated
for the eighth class, but they need the editor and a built bridge. Every other fighter's kit tests are green; their content changed only
where the reference data listed above says so.

### Training-mode check (hitbox view)

Hauler vs a standing duelist at 0%: (1) the down tilt: the arm low along the floor on 6-7 out to about 2 world units; with the trip
chance it sometimes sits the duelist down (no launch line, a knockdown pose) and that duelist cannot act until frame 40 of the sit; (2)
the forward tilt aimed up, straight and down: the three arms at 30 degrees up, level and 30 down; (3) two haulers' forward smashes
meeting: a clank flash, no rebound, both swings finish; (4) the forward air's late fist (21-23) low in front, meteoring an airborne
target; (5) the down air's box under the feet on 14-16; (6) the up smash clap 3.5 units up.

## For the art agent

* An emblem for class 7 (Bracers) in `godot/ui/match_hud.gd`, a look for the built-in Hauler (it uses `Loadout.default_for(7)`
  for now), and a bigger, broader body (the sim's ECB is 2.8 tall and 1.0 half wide, against the
  shared 2.2 and 0.8): huge forearms and fists, short legs.
* **A trip pose**: a tripped fighter is in `Knockdown` with the bridge's state name unchanged; the sim knows it is a trip
  (`Fighter::tripped`). If the view should sit the fighter up rather than lay it flat, the bridge needs a read-only accessor for the flag
  (a one-line addition; not added here).
* Moves that want their own clip (they fall back to the shared archetypes now; first active frame, active span, pose): jab 1 (a hook,
  5-6, at chest height out to 1.7 ahead), jab 2 (an uppercut, 4-6, the fist rising in front to 2 up); forward tilt (a straight-armed
  backhand swipe, 7-9, the fist 2.4 ahead at 1.6 up, angled 30 up or down); up tilt (the arm swept from in front up over the head to
  behind, 5-11); down tilt (a crouched slap along the floor, 6-7, out to 2 ahead); dash attack (a rolling body tumble, 9-24); forward
  smash (a two-handed clap far in front at waist height, 22-23, 3 ahead at 0.8 up, wound back from 14); up smash (an overhead clap,
  14-15, 3.5 up); down smash (both fists slammed down to either side, 11-12, then a shockwave on 13-14, 2.4 in front and 1.5 behind);
  neutral air (a spin with both arms out, 10-26); forward air (both fists swung overhead and down in front, 18-23, low in front on 21-23);
  back air (a kick straight back, 7-16, the foot 1.9 behind); up air (an arcing headbutt, 6-10, the head 2.8 up); down air (a stomp,
  14-16, the feet under the body); the throws (a heave behind, a toss up, a slam down); ledge attack (a sweep climbing up, 23-25);
  get-up attack (behind 18-19, in front 23-24).

## Phase 2 plan: the specials (decision 41: E5 and E29; E22b `on_land` and E11 `start_speed_y` now exist from Cloud's phase 2)

Every piece defaults to today's behaviour for every other fighter, is hashed (content through `Weapon::hash_into` /
`Ruleset::hash_into`, state through `GameState::hash_into` with a checksum case each), appears in `docs/CONTENT.md`, and gets its own
test. One `SIM_VERSION` bump.

1. **E29, additive charge damage**: `Move::charge_add: Option<ChargeAdd { full: u8, add: Fx, floor: bool }>` (default none: today's
   multiplicative smash bonus) read in `combat::resolve_hits`: damage `base + add x min(charge, full) / full`, floored to a whole percent
   if `floor`; and a script function `set_charge(n)` (0 to 255, sets the hashed `Fighter::charge`) as a whitelist entry in
   `sim-script/src/api.rs` with its test in `scripted_moves.rs` and the doc line. Tests: 10 / 19 / 27% at charge 0 / 55 / 109, aerial 6
   to 23.
2. **The knockback armor kind** (E1 addition, needed by phase 3 but small and testable alone): `ARMOR_KNOCKBACK` with a `subtract`
   flag, `Armor::withstands(frame, damage, knockback)`, the validation refusal removed, tests beside the existing armor test in
   `multihit.rs`.
3. **Giant Punch** (neutral special, the pump pattern): frame 0 reads `stored` (110 → the full punch); the loop counts a var to 110,
   `goto`s itself, stores and ends on full; special releases (`goto` the grounded or aerial release by `grounded`); shield / jump (air
   dodge in the air) store and `goto` a 4-frame cancel `ext`; a hit loses the charge (the shared rule: `store(0)` on entry). Releases
   (`ext` x2): the uncharged rows with `charge_add { full 110, add 18, floor }`, hitbox on the release's 19-20, FAF 63, the lower arm
   intangible 4-20, `rebounds` false; the aerial one's `on_land` the grounded one. Full punches (`ext` x2): `armor` super 9-20 / 15-20,
   28% / 18% and 25% rows, FAF 48, `on_land`; no helplessness.
4. **E5, bury** (approved, decision 38, with the stock-difference terms): `EFFECT_BURY`, Ruleset bury numbers (55 + 0.5p + 1.5k
   rounded up, +8 / +12 behind, -18 / -22 ahead, break at 90 knockback, 12 frames of exit lag, the pop-out speed [E: gap 9]),
   `FighterState::Buried` with hashed `bury_timer`, `bury_kb`, `bury_angle`, `bury_set`; mash 8 / 14 reusing the grab's; no grab on a
   buried fighter; a fighter lying down cannot be buried; lowered hurtboxes; weaker hits store their knockback (the last kept [E: gap
   10]). Tests in a shared `bury.rs`; `step.rs`'s invariant test and the view's state map (art agent) learn the new state.
5. **Headbutt** (side special and its `air_version`): super armor 5-14, hitbox 20-21, FAF 63, the grounded bury boxes (grounded only),
   the aerial-only 70-degree box, the aerial bury box for anyone (a meteor on airborne targets), shield +25 (350 / 413 [D]); a frame-0
   `set_vel` x0.5 / x0 on the ground, `start_speed` 67 and `start_speed_y` 50 in the air, `air_gravity` 0.0425 reference units; ledges
   from frame 2 [E: conflict 4].
6. **Spinning Kong** (up special and its `air_version`): grounded super armor 5-17, the lower arm 19-24, hit 1 on 19 (the aerial-only
   boxes at 367 until E28 exists, then 366), the loop re-armed by script on 25, 32, 40, 49, 55, the final hit on 62, the stick-steered
   slide (accel 0.034, friction 0.015, max 1.1, 0.04 after the last hit), FAF 105, a push-immune window on the armor frames (decision
   41: "windbox immunity modelled"; a small `Move::push_immune` window, its frames [E]); aerial hit 1 on 4-6, twelve 1% autolink hits on
   their listed frames, the 2% final on 39-43, the lower arms 10-38, the rise (0.9 up, accel 0.05, max 1.7, gravity 0.07 as the gravity
   itself: decision 41), ledges from 14, 70 frames then the special fall with 38 frames of landing lag; an early landing `landing_lag` 42.
7. **Hand Slap** (down special and its `air_version`): the grounded loop (slaps on 12-13 and 23-24, a 26-frame loop banked by presses,
   twice for a press in its 20-26), 14% grounded-only boxes at 78 / 84, H 0.5, shieldstun x0.58, no clank, the ending FAF 47; the aerial
   claps (5% FKV 36 at 90 with a 20% trip, then the 6% meteor at 270 with a 20% trip), FAF 50, landing lag 10.
8. Slots: 14 of 16 `ext` (`PROJECT_AUDIT.md` "Slots"): the punch's two releases, two full punches and the cancel; the headbutt's,
   spin's and slap's aerial versions; the slap's loop and ending; the four cargo throws in phase 3.

## Phase 3 plan: the carry (E7, approved by decision 38)

1. **E7** as designed in `PROJECT_AUDIT.md` ("The cargo hold"): `Move::carry: Option<Carry { walk, clock_num, clock_den, jumps,
   hold_at, armor, throws, ledge }>` on the forward throw (the 15-frame invincible lift); `FighterState::Carrying` with hashed
   `carry_jumps` and the victim's `grab_start`; the clock `(90 + 1.7p) x 11 / 6 - g` with the grab's mash cuts carried over; the holder's
   walk-only movement at x1.1 the walk (conflict 5 resolved to the move page's figure, decision 41: the parameter file's lift values
   recorded), one jump, falling off the stage with the victim pinned overhead; no ledge grab [E: gap 4]; attack with a direction starts
   the matching cargo throw.
2. **The carry's knockback armor** from frame 16 until a throw starts: the threshold resolved by the porter (decision 41) to **80
   units, non-subtractive** [E], the reasoning in `REFERENCE_DATA.md` conflict 2 (two dragdown statements agree with a frame-16 start,
   80 is the tumble threshold, and ssbwiki.com's "16" reads as the frame).
3. **The four cargo throws** (`ext` x4): forward (15, FAF 46, 12% at 60, BK 90, KS 46), back (turns and releases on 16, FAF 43, 13% at
   72, BK 85, KS 52), up (15, FAF 30, 12% at 92, BK 65, KS 53), down (17, FAF 45, 11% at 46, BK 70, KS 45; on frame 17 the horizontal
   speed stops and the vertical becomes 0.1875 world). The forward throw placeholder of phase 1 becomes the lift.
4. Tests: a shared `carry.rs` (the clock at 0% straight out of the grab 165 and after 60 frames 105, mashing, the walk, one jump,
   walking off with the victim, each cargo throw, the armor, a KO during the carry, checksum cases, a rollback replay through a carry off
   the stage) and every other fighter's grabs unchanged (`grabs.rs`, `throw_combos.rs`, `base.pfc` equality); `pftool net-fuzz` at zero.
5. Then the iconic-move re-check in the hitbox view and against footage for the open gaps (`REFERENCE_DATA.md` gaps 1-15).

## Questions for the owner

1. **The reference's base 7% trip** (any hit over 55 knockback that does not lift a grounded target trips 7% of the time, on top of a
   move's own chance): build it? It would change every fighter (low sideways and downward hits from every class could trip), so it is
   not built.
2. **The other trip chances in the older kits** (ssbwiki.com raw tables): the Claws' down tilt 40% at 361 (it **can** trip: a
   behaviour change), and these that can never trip on a flat stage at their angles: the Claws' down smash 35%, the Needles' down tilt
   40% and down smash 30%, the Embers' down tilt 40%, the Gauntlets' down tilt 50%, the Greatsword's down tilt 20%. Add them as data (only
   the Claws' would play differently)?
3. **The angled forward tilt's FAF 36**: add a small `aim` script register (a whitelist entry; best after Cloud's E16 registers are
   merged, to keep the numbering clear) so the angled versions end one frame later as the reference does?
4. **The trip pose**: should the view show a trip as sitting (the reference) rather than the knockdown's lying pose? It needs a bridge
   accessor for `Fighter::tripped` (art agent's side).
5. The usual footage requests for phase 2 and 3 stand (`REFERENCE_DATA.md`, "Footage that would settle most of them").
