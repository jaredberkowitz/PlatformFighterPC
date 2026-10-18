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

## Phase 2 (sim 90, a placeholder number to renumber at the merge; content schema 6): the specials

Branch `bracers-phase2`, from `movesets` at 17de032, with `origin/main` (sim 87: E28, E3e) and `movesets` (sim 89: the Clutch's
phase 1, knockback armor; then sim 93) merged in. Commits: 4e24f65 (trip follow-ups), eaa84e0 (E29 and the charged punch), badc0d8 (E5
bury and the headbutt), 536837f (the spin, push immunity), bbde085 (the hand slap), plus the merges 50e8af4, 556d486, fddff5e. The
owner asked for SIM_VERSION 90 on this branch; `movesets` is at 93 now, so the number must be renumbered (above 93) at the merge.

### Engine (every piece general, defaulting to the old behaviour unless the data uses it)

| What | Where | Default / who changed |
| --- | --- | --- |
| **Base trip chance** (decision 50): a hit of more than 55 knockback that would not lift a grounded fighter trips 7% of the time, and a hit with its own `trip` adds 7 to it (ssbwiki.com "Tripping": 40% becomes 47%); throws never trip [E]; one roll only when eligible | `combat::trip_chance`, `Ruleset::trip_base` (7) / `trip_base_kb` (55) | **every fighter** (owner's request): any low sideways / downward hit over 55 knockback on a grounded target can trip; `trip_base 0` turns it off |
| **Older kits' trip chances** (decision 51, ssbwiki.com raw tables re-read 2026-10-13) | `moves.rs`, `moves_gauntlets.rs`, `moves_greatsword.rs` | Claws down tilt 40% (361: **it can trip**, a behaviour change); Claws down smash 35%, Needles down tilt 40% and down smash second hit 30%, Embers down tilt 40%, Gauntlets down tilt 50%, Greatsword down tilt 20%: data only (they lift, so never trip on a flat stage) |
| **`aimed` script register** (decision 52; `aim` was taken by the water pump's `var aim`, which would no longer compile) | `sim-script/src/api.rs` `R_AIM` 29, `scripting.rs` | none; the Bracers' forward tilt reads it to end the straight one on 35 and the angled ones on 36 |
| **`fighter_tripped(i)`** bridge accessor (decision 52), read-only | `godot-bridge/src/lib.rs` | view only |
| **E29 additive charge damage**: `Move::charge_add { full, add, floor }` (damage + add x min(charge, full) / full, floored) and the script function `set_charge(n)` | `moves.rs` `ChargeAdd`, `combat::resolve_hits`, `api.rs` `F_SET_CHARGE` 50; content `charge_add { full add floor }`, validated | none |
| **E5 bury** (decision 38, with the stock terms): `EFFECT_BURY` (content `effect bury`), `FighterState::Buried`, hashed `Fighter::bury_timer` / `bury_kb` / `bury_angle` / `bury_set` (checksum cases); the length `ceil(55 + 0.5p + 1.5k)` +8 / +12 / -18 / -22 by the stock difference; the shared mash helper `grab::mash_cut` (the grab's 14 a press, 8 a flick; the egg can reuse it); out after the timer with 12 frames of lag (a `Rebound`, no invincibility) or launched by a kept knockback; a hit of 90 knockback frees and launches in full; a weaker one's launch is kept (the last one [E: gap 10]); bury hits weight-independent and **ignore subtractive knockback armor** (the reference's rule); no grabs (ordinary or command) on a buried fighter; a lying fighter is not buried, a tripped (sitting) one is; hurtboxes `bury_depth` (half the height) lower [E] (`combat::body_hurtboxes`, used by hit detection and the bridge's hurtbox view) | `combat.rs`, `fighter::bury` / `buried`, `grab.rs`, `state.rs`, `Ruleset::bury_*` | nothing buries without `effect bury` |
| **Push immunity**: `Move::push_immune_from` / `_to` (decision 41: "immune to most windboxes") | `combat::apply_hit`, content fields | 255 / 0 (never) |

### Built (`sim-core/src/moves_bracers.rs` `specials`; slots: ext0-ext9, 10 of 16 used; the cargo throws take ext10-13 in phase 3)

* **Charged punch** (neutral special; scripts `bracers_punch_charge`, `_release`, `_cancel`): the wind-up charges from frame 8 to 110 frames
  [R: 110; the frame mapping E, gap 1]; special at any point is kept and releases from frame 8 (the shortest release hits on 26-27 for
  10%); shield / dodge / air dodge / jump store the charge and go to the 4-frame cancel; a hit loses it (it lives in the script's
  variable until stored); at 110 it stores itself and the stance ends (on frame 117 [E]); it lands and goes over edges charging. The
  release (ext0 / ext1): 10% / 6% + 18 x charge / 110 rounded down (E29), BK 45 KS 70 / BK 60 KS 65, hit on the release's 19-20, FAF 63,
  lower arm intangible 4-20, no rebound; the aerial lands as the grounded one (E22b) and the grounded one goes over an edge as the
  aerial. The full punch (ext2 / ext3): 28 / 28 / 28 / 18% BK 35 KS 73 and 25% x4 BK 40 KS 72, super armor 9-20 / 15-20, hit 19-20, FAF
  48, counted from the press (its data is one frame earlier because it starts on the wind-up's frame 1 [D]); landing in the aerial one
  switches to the grounded one on the same frame (so to the grounded armor window). No special fall.
* **Headbutt** (side special, ext5 in the air): super armor 5-14, hit 20-21, FAF 63, the two grounded-only bury boxes (10% at 270, BK 40
  KS 40; 9.0.0's higher box), the aerial-only 8% box at 70 (BK 15 KS 40), the aerial version's bury box for anyone (BK 30: buries a
  grounded target, meteors an airborne one), shield +25 (350 / 413 [D]); on the ground the speed x0.5 / x0 by a script on frame 1 (one
  frame of traction late [E]); in the air `start_speed` 67, `start_speed_y` 50, gravity 0.0425 of the reference's units; ledges from
  frame 2 [E: conflict 4]; the aerial one lands as the grounded one [E: not published].
* **Spinning clothesline** (up special, ext6 in the air; scripts `bracers_spin`, `_air`): grounded super armor 5-17, lower arm 19-24, hit
  1 on 19 (grounded targets 95 / 80, aerial targets **366** (E28), set knockback 90), the loop's five hits on 25-27, 32-33, 40-42, 49-50,
  55-58 (re-armed by the script; 367 forward / 20 forward / 60 backward, H 0.8), the final 4% on 62 (BK 60 KS 157, H 2.0), FAF 105, the
  stick-steered slide (0.034 / 0.015 / 1.1, 0.04 after the last hit; order of accel and friction [E]), push-immune for the whole move
  [E: frames not published]; over an edge it goes on as the aerial one [E: gap 7]. Aerial: hit 1 on 4-6, twelve 1% hits on their spans
  (each re-armed), the 2% final on 39-43, lower arms 10-38, the rise (0.9 up, 0.07 a frame down as the gravity itself (decision 41)
  [E: conflict 6], to the body's fall speed [E]; stick drift 0.05 up to 1.7), ledges from 14 both sides, 70 frames then the special fall
  (landing lag 38), an early landing 42 frames (the grounded version's 63-105) [D].
* **Hand slap** (down special, ext7 loop, ext8 ending, ext9 aerial; scripts `bracers_slap`, `_loop`, `_end`): slaps on 12-13 and 23-24
  (two hits each pass), four 14% grounded-only boxes (78 outer / 84 inner, BK 85 KS 30, r 10.3 at ±6.1 / ±21.1, 5 up [D reading]),
  H 0.5, shieldstun x0.58, shield 107, no clank; the 26-frame loop banked by special presses (a press in the loop's 20-26 banks two,
  not added to one already banked [E: a reading]); the ending free on 47. Aerial: the 5% clap at 90 (FKV 36) on 19-21 and the 6% meteor
  at 270 (BK 30) on 28-30, 20% trips, no clank, FAF 50, landing lag 10; box places [E].

### Older fighters whose behaviour changed (list for the owner)

* **Every fighter**: the base 7% trip (any hit of more than 55 knockback that sends a grounded target sideways or down; a hit with its
  own trip chance gains 7 more: the Hauler's down tilt 47%, its down forward tilt 22%, the Gauntlets' late forward air 27%, the Claws'
  down tilt 47%). Throws are excluded.
* **Claws**: the down tilt trips (40% + 7). The other new chances (Claws down smash, Needles down tilt and down smash, Embers, Gauntlets,
  Greatsword down tilts) are data only on flat stages.
* Nothing else: E29, E5, push immunity and `aimed` are used only by the Bracers. `base.pfc` re-exported (it carries the new ruleset
  fields and the new trip data).

### Verified against [R] / [D] (tests that ran)

`sim-core/tests/bracers_specials.rs` (19): E29's formula (10 / 19 / 27 / 28 at 0 / 55 / 109 / 110, aerial 23); the shortest release
on 26 for 10%; releases at 0 / 55 / 100 / 109 frames of charge hitting 18 frames later for 10 / 19 / 26 / 27; the full charge storing
itself and the full punch hitting on 19 for 28%, FAF 48; the data of the releases, full punches and cancel; shield and jump cancels
keeping the charge, the shield up after 4 frames, a stored 23 released for 13%; a hit losing the charge; the full punch's armor holding
on 9, 14 and 20 and not on 8 or 21 (a jab on each frame); the aerial full punch landing into the grounded one on the same frame, the
aerial release ending free; the headbutt's data, its bury on frame 20 from the stick and special, the aerial start speeds, shield +25;
the spin's data (366 on aerial targets, the loop spans, the final hit, 19% aerial total), its hits only on the reference's frames (move
frames, hitlag aside), the aerial hit 1 on 4, the rise of about 6 reference units, the fall after 70 frames and its 38 frames of landing
lag, the slide with the stick to the 1.1 cap, push immunity; the slap's data, 47 / 73 / 99 / 125 FAF for 0, 1 early, 1 late and 2
presses, the first slap on 12 for 14% and no hit on an airborne target.
`sim-core/tests/bury.rs` (11): the length (138 at 0%, +8 / +12 / -18 / -22 by stocks, longer at 100%), weight independence, 138 frames
buried then 12 of lag and no invincibility, mashing (17 frames with a press every other frame), no grab or command grab, lying not
buried and tripped buried, the aerial headbutt's straight-down meteor, a 95-knockback hit freeing and launching, a 60-knockback hit kept
and launching at the end, the hurtboxes half the height lower, subtractive knockback armor ignored and non-subtractive armor holding,
a rollback replay through a bury with random mashing. `trips.rs` (10): the base chance's numbers, 7% over 300 seeds with one roll each
and none when off, the 47% combined chance over 60 seeds, every older kit's trip data, the Claws' down tilt tripping. `bracers.rs`: the
angled forward tilts end on 36 and the straight one on 35, the stick's aim read as it starts. `scripted_moves.rs`: `set_charge` with
`charge_add` (19%), `aimed` (1 / 0 / -1). `state::tests::checksum_sees_every_field` (the four bury fields and the state).

### Estimated or limited [E]

* The charge's frame mapping (gap 1) and the turnaround (shared B-reverse windows, gap 2); the punches' air physics (gap 6).
* The bury: the pop-out's height (none: an on-the-spot 12-frame rebound, gap 9), which weak hit's launch is kept (the last, gap 10), the
  buried hurtboxes' depth (half the height), the stock terms as ssbwiki.com's summary in frames (+8 / +12 / -18 / -22) rather than its
  printed formula, which does not give those numbers (15 x 1.5 = 22.5, not 18: conflict 12 below).
* The headbutt's ledge window (conflict 4) and landing; the spin's off-edge version (gap 7), push-immune frames (gap 14), the aerial
  gravity reading (conflict 6) and fall cap, its special-fall drift (gap 12); the slap's air physics (gap 8) and its "twice" rule's
  reading; positions on arm, hand and head bones throughout.
* Not modelled: the hit-while-carrying rows (phase 3), the slap's `direct=f` counter oddity, the headbutt's "hit while grabbing the
  ledge cuts its lag" (a ledge grab already ends the move).

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace --no-fail-fast` (all green); `cargo clippy --workspace --all-targets -- -D warnings`
(clean); `pftool fuzz-rollback 500` (0 mismatches, every class); `pftool net-fuzz 200` (0 mismatches, every class); `content/base.pfc`
re-exported (the equality test passes). **Not run**: the Godot tests.

### Training-mode checks (hitbox view)

Hauler vs a duelist at 0%: (1) the punch charged to full (the stance ends by itself about 2 seconds in), then a press: the fist out on
19-20, a jab on frame 12 not flinching it; (2) the headbutt on a standing duelist: buried (half sunk) for 138 frames, out with a short
lag; jabbed while buried it stays; smashed it flies; (3) the aerial headbutt on a duelist in the air: straight down; (4) the grounded
spin held forward slides to about 1.1 reference units a frame and its first hit sends an airborne duelist along with the spin (366);
(5) the slap pressed once more late in the loop: two more loops.

## For the art agent (phase 2)

* **Buried** (`FighterState::Buried`, the bridge's state name `Buried`): the fighter sunk to about half its height, arms up, struggling
  (a shake on mash inputs would read well); out of it the state is `Rebound` for 12 frames (a pop-up). Its hurtboxes are already lowered
  in `fighter_hurtboxes`.
* **The trip pose**: `fighter_tripped(i)` is true while a `Knockdown` is a trip: sit the fighter up instead of laying it flat.
* Specials' slots and poses (they fall back to shared clips now): neutral special = the wind-up (an arm swinging round and round, faster
  as it charges; the stance ends by itself at full), ext0 / ext1 the charged punch (hit 19-20, the fist straight out at chest height),
  ext2 / ext3 the full punch (the same, bigger, hit 19-20 counted from the press), ext4 the 4-frame cancel; side special / ext5 the
  headbutt (a big overhead head slam, hit 20-21, low in front); up special the grounded spin (arms out level, spinning, sliding; hits
  19-62), ext6 the aerial spin (rising a little, arms out, 70 frames, then the special fall); down special / ext7 the hand slap (both
  palms slamming the ground either side on 12-13 and 23-24 of each 26-frame loop), ext8 its ending (13 frames), ext9 the aerial clap
  (a clap high in front on 19-21, then both hands swung down on 28-30).

## Phase 3 (sim 98; content schema 6 unchanged): the carry (E7) and the cargo throws

Branch `bracers-phase3`, from `movesets` at 81d5f50 (sim 95), merged with `movesets` at a54bf60 (sim 97: Link phase 1, Yoshi phase 2,
the knockback fixes) before the docs; this branch takes 98. Built as the plan below says (decision 38), with these details.

### Engine (general, off unless a move has a `carry`)

| What | Where | Default / who changed |
| --- | --- | --- |
| **`Move::carry: Option<Carry { walk, clock_num, clock_den, jumps, hold_at, armor, throws, ledge }>`** on a throw (the lift); content `carry { ... }` block (`docs/CONTENT.md`), validated (on a throw slot; each throw a real move with a throw hit and no carry; clock fraction above 0; walk 1-1000; the armor's own checks); `Weapon::hash_into` | `moves.rs`, `sim-content/src/format.rs`, `lib.rs` | none |
| **`FighterState::Carrying`**: `fighter::carrying` / `carry_on_ground` (walk at `walk` percent of the walk speed and both accelerations, turning at once [E]; `jumps` jumps in all: the body's jump squat (counted in `lag`) and full / short hop, or a midair jump once off the ground; walking off an edge stays carrying with the jump's momentum rule; air physics as the body's, fast fall included; landing straight back into the carry; attack throws by the stick or the attack stick: up, down, back, else forward; a ledge only with `ledge`) | `fighter.rs` | none |
| The lift's end goes into the carry if the fighter is still held; a throw with `intangible` frames is intangible from its frame 1 (`grabbing`) | `fighter::attack`, `grabbing` | no older throw has `intangible` |
| **The clock**: `grab::start_carry_clock` on the lift's frame 1: `hold_time(p now) x num / den - (grab_start - grab_timer)`, at least 1; `grab_start` (hashed, set by `grab::connect`) is then cleared so it runs once; the clock runs while `Grabbing` or `Carrying` with the usual mash cuts; `grab::pin` holds the victim at `Carry::hold_at`; `drop_grab` frees a carrying holder as any holder | `grab.rs` | ordinary holds unchanged |
| **The carry's armor** in `combat::apply_hit`: a carrying defender's grab is dropped only once its armor (frames counted from the carry's start, capped at 255) has not withstood the hit; an armored hit leaves damage and hitlag | `combat.rs` | every other hit drops the grab first, as before |
| Hashed state: `Fighter::carry_jumps`, `Fighter::grab_start`, the new state; checksum cases `carry_jumps`, `grab_start`, `state carrying` | `state.rs` | |
| The step invariants and push bodies know `Carrying` (a mutual grab, either on the ground or in the air, its victim either) | `step.rs`, `push.rs` | |
| Read-only bridge accessor `fighter_grab_partner(i)` (who a fighter holds or is held by, or -1) | `godot-bridge/src/lib.rs` | view only |

### Built (`sim-core/src/moves_bracers.rs` `carry`; slots ext10-ext13, 14 of 16 `ext` used)

* **The lift** (forward throw): 15 frames, intangible 1-15 [R], no hit, the victim held overhead at (0, 2.8) world [E: the body's ECB
  height; the reference's hold bone is not published]; the carry from frame 16.
* **The carry**: walk 110 (x1.1 the walk and both accelerations [R], conflict 5 as decision 41), clock 11 / 6 [R], 1 jump [R count;
  which jump and its height: gap 3, the body's own], knockback armor 80 not subtractive [E: conflict 2 / gap 11, reasoning in
  `REFERENCE_DATA.md`], no ledge [E: gap 4]; throws forward / back / up / down = ext10 / 11 / 12 / 13.
* **Cargo throws** [R]: forward on 15 (FAF 46) 12% at 60 BK 90 KS 46; back turning round, behind on 16 (FAF 43) 13% at 72 BK 85 KS 52;
  up on 15 (FAF 30) 12% at 92 BK 65 KS 53; down on 17 (FAF 45) 11% at 46 BK 70 KS 45, with the script `bracers_cargo_down` stopping the
  sideways speed on 17 and, in the air, setting the vertical to 1.5 reference units (0.1875 world) [R]; on the ground only the stop
  [E]. No invincibility [R]. Each holds the victim overhead until it lets go, lands and goes on along the ground, and goes on in the air
  if it slides off an edge [E: neither published].
* The placeholder plain forward throw of phase 1 is gone.

### Verified against [R] / [D] (tests that ran)

`sim-core/tests/carry.rs` (13): the lift's data (no hit, FAF 16, intangible 15) and every other weapon's forward throw still plain;
each cargo throw's data (release frame, FAF, damage, angle, BK, KS, behind, no invincibility, held overhead); the lift intangible on
frames 1-15 and not 16, the victim overhead; the clock 165 - g at 0% straight out of the grab (g the frames the hold ran), 105 after 60
frames of grab, 320 - g at 50%, `grab_start` cleared once used; mashing in the grab carried over exactly, mashing in the carry shortening
it; the carry running out on its clock (one frame a frame) and letting go with the ordinary release; the walk reaching x1.1 the walk
speed and accelerating at x1.1 both accelerations, turning back, no shield / special / grab / dash; one jump in all (squat, airborne
still carrying, no second jump, landing back into the carry, a midair jump after walking off); walking off the stage with the victim
overhead every frame down to the blast zone, the holder's knock-out letting the victim go in the air; each cargo throw from the
stick (attack alone the forward one) hitting on its frame for its damage, ending on its FAF (hitlag aside), the back one turning round
and throwing behind; the down throw in the air stopping and rising at 0.1875 on 17, the up throw in the air; the armor holding a 20-BK
hit and broken by a 200-BK one (Hitstun, victim freed), no armor during a cargo throw; a rollback replay through a carry off the stage
with random mashing. `bracers.rs` updated (the forward throw is the lift). `state::tests::checksum_sees_every_field`.

### Estimated or limited [E]

* The armor's threshold and kind (conflict 2, gap 11); the carry's jump (gap 3), ledge (gap 4), release (gap 5), the holder's
  knock-out (gap 13); the hold's place overhead; the turn (at once, no turn animation); the aerial cargo throws' physics (the body's);
  landing during the carry or a cargo throw; the grounded down throw not hopping.
* Not modelled: the carry's interrupted row (4%, FKV 30) and each throw's (3%), as for every throw (house limit); a soft-platform drop
  while carrying.

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace --no-fail-fast` (all green, after the merge too); `cargo clippy --workspace --all-targets
-- -D warnings` (clean); `pftool fuzz-rollback 500` (0 mismatches) and `pftool net-fuzz 200` (0 mismatches) after the merge;
`content/base.pfc` re-exported (the equality tests pass). **Not run**: the Godot tests.

### Training-mode checks (hitbox view)

Hauler vs a duelist at 0%: (1) grab, stick forward: the duelist lifted overhead, the hauler walking slower than a walk feels faster
(x1.1), back turns at once; (2) walk off the edge: both fall, the duelist overhead, a jump once; (3) attack with each direction: the
forward toss, the back one turning round, the up one (the shortest), the down bear hug (in the air a small hop on its frame 17);
(4) a jab on the carrying hauler does not free the duelist; a smash does.

## For the art agent (phase 3)

* **The lift** (forward throw, `Attack` with `MoveId::FThrow`, 15 frames): both arms raising the held fighter overhead by frame 15.
* **`Carrying`** (the bridge's state name; the held fighter's holder is `fighter_grab_partner(i)`): a walk with both arms up holding
  the fighter on top of the head (the held fighter's feet at 2.8 world above the hauler's feet), on the ground, rising and falling; a
  jump pose. The held fighter (state `Grabbed`, its holder `Carrying`) should lie across the hands, struggling.
* **The cargo throws** (`Attack`, ext10-13): forward (ext10) a high forward toss released on 15; back (ext11) a turn and a backward
  toss on 16 (the fighter ends facing the other way); up (ext12) a straight-up toss on 15; down (ext13) a bear-hug slam on 17 (a small
  hop in the air). The held fighter stays overhead until each release.

## Questions for the owner (phase 2)

1. **SIM_VERSION**: this branch carries 90 as asked, but `movesets` is at 93; renumber above 93 at the merge.
2. **The `aimed` register name**: `aim` would have broken the water pump's scripts (`var aim`), so the register is `aimed`. Keep?
3. **The bury's stock terms**: the frames ssbwiki.com summarises (+8 / +12 / -18 / -22) are built; its printed formula gives +7.5 /
   +11.25 / -22.5 / -27. Keep the summary (as approved in the audit) or switch to the formula?
4. **Throws and the base trip**: excluded [E]. Should a thrown fighter be able to trip (a throw that sends low sideways)?
5. The footage requests in `REFERENCE_DATA.md` stand (the charge's frame mapping, the bury pop-out, the spin's off-edge and gravity, the
   slap's late press).

## Phase 2 plan as written before the build (kept for the record): the specials (decision 41: E5 and E29; E22b `on_land` and E11 `start_speed_y` now exist from Cloud's phase 2)

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

## Phase 3 plan as written before the build (kept for the record): the carry (E7, approved by decision 38)

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

## Sim 143: the aerial spin rises (owner decision 102)

The owner: "Hauler up special does not gain any height at all." Conflict 6 of the reference data (the aerial spin's "gravity
multiplier 0.07") had been read as the gravity itself, so the spin peaked about 6 reference units up on frame 13 and then sank for the
rest of the move. It is now read as a multiplier of the body's gravity (0.085 x 0.07, as the headbutt's 0.5 is): from a standstill in
the air the spin rises the whole move, about **48 reference units** (49.3 measured, peak just after the special fall begins on frame
70; before: 6.2). The grounded spin stays on the ground (0 before and after). `bracers_spin_air.script`; tests
`recovery_rise::the_haulers_aerial_spin_rises_through_the_whole_move`, `the_haulers_grounded_spin_stays_on_the_ground`,
`bracers_specials::the_aerial_spin_rises_slowly_and_ends_in_a_special_fall_with_38_frames_of_landing_lag`.
