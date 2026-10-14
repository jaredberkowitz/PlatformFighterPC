# Implementation report: the eleventh moveset (reference: the reference game's Villager)

Class 10 **Pockets** on the **settler** body, built-in fighter **Settler** (owner decision 65). Reference data: `REFERENCE_DATA.md`;
engine audit: `PROJECT_AUDIT.md`. Owner decisions 65-68 (2026-10-14) are recorded in `../ROSTER_STATUS.md`.

## Phase 1 (sim 113, content schema 7 unchanged; branch `pockets-phase1` from `movesets` at 71cbbed, sim 111, with `movesets` 336b974 (sim 112) merged in)

### Milestone 1: the engine (all general; every default is today's behaviour)

* **E40, the previous reference version's hit rules** (decision 67): `Hitbox::rules` (`RULES_CURRENT` / `RULES_PREVIOUS`), the ruleset's
  `prev_damage_mult` 1, `prev_hitlag_per_damage` 0.3846154, `prev_hitlag_base` 5, `prev_rage_max` 0.15, `prev_tumble_hitstun` 1, hashed
  `Fighter::launch_older`; `combat::apply_hit` (damage, hitlag with a victim-only crouch, rage, +1 tumble hitstun), `block` (shield
  damage), `fighter::apply_launch` / `launch_fall` (no speed-up, the gravity boost at every angle, no shared vertical fall);
  `combat::hitlag_frames_by`, `rage_multiplier_up_to`. Only the Settler's jab uses it.
* **E39, random versions**: `Move::versions` drawn on a move's first frame from the hashed game generator (`fighter::draw_version`,
  called from `step` after the fighters' updates).
* **E41, balloons**: `Move::balloons` (`moves::Balloons`), hashed `Fighter::balloons` / `balloon_hp` / `helium_used`,
  `sim-core/src/balloons.rs` (the tank, the count, letting go), hits on the balloons only (melee in `resolve_hits`, shots in
  `update_projectiles`), the `balloons` script register (whitelist entry, `scripted_moves` test, `docs/CONTENT.md` line).
* `ChargedShot::power_to`, `Burst::on_hit`, `Item::angle` + the bridge's `item_angle(i)` (degrees; for the art director), `trig::degrees_of`,
  `fighter::stand_or_fall`; validation accepts a move that is only a motion. Content: every new field has a default, validation, a
  `docs/CONTENT.md` line and a round trip (`sim-content` `the_pockets_fields_round_trip_and_are_checked`).

### Milestone 2: the body, every normal, two specials (`sim-core/src/moves_pockets.rs`, scripts `pockets_*.script`)

* `FighterParams::settler` (13.0.1): every attribute in `REFERENCE_DATA.md`, a wall jump, the item throws.
* Every normal on the 13.0.1 raw tables, the jab on the 1.1.x data: see `REFERENCE_DATA.md`. Projectile normals as projectiles (the
  flowerpot, the bowling ball with its own 1.2x charge, the slingshot pellets); the turnips by E39; the firework's eleven boxes in six
  groups; the shovel's bury (E5) on grounded fighters.
* The rocket (side special): a projectile with a script (`pockets_rocket.script`), hits by age (E3a), one out (E3d kind 4), a burst on a
  wall or at its end and not on a fighter (E3e + `on_hit`); held from frame 16 the ride (`ext4`, `pockets_ride.script`) with its bounce
  (`ext5`), crash (`ext6`) and hop (`ext7`).
* The balloons (up special, `pockets_balloon.script`, and `ext8` `pockets_flight.script` for flights past 253 frames).
* Registered: `CLASSES` 11, `Content::placeholder`, `roster.gd` (name, blurb, the built-in Settler with the shared default look), the
  Godot tests' rosters, the fuzzers (they draw class 10: see below), base.pfc re-exported.

### The older jab: rules, sources and test results

The rules and their sources are the table in `REFERENCE_DATA.md` ("The previous version's hit rules"). Tested in
`sim-core/tests/older_rules.rs`, each against the same hit on 13.0.1's rules:

* 3% added (13.0.1: 3.6%); hitlag 6 and 6 (13.0.1: 7 and 7); against a crouch 6 for the attacker, 4 for the victim (13.0.1: 4 and 4).
* A strong vertical hit: the same knockback, one more frame of hitstun, no speed-up (13.0.1's is sped up), the gravity boost added
  (`5 x (0.078 - 0.075)` on a settler), the victim falling at its own 1.32 instead of 1.8.
* Rage at 150%: x1.15 (13.0.1: x1.1).
* The jab pops a standing foe up and away at 0-150%.
* Jab 1 into jab 2 is a true combo at every percent 0-150% (on the duelist and the settler); mashed, the endless jab stays one combo at
  low percents.
* Frame advantage on the first frame the settler can act (the duelist, weight 90; `print_the_jabs_frame_advantage_and_follow_ups`):

| Percent | After jab 1 | After jab 1 > 2 |
| --- | --- | --- |
| 0-20 | 0 | 0 |
| 30 / 40 / 50 | 0 | 2 / 3 / 5 |
| 60 / 70 / 80 | 0 | 7 / 9 / 10 |
| 90 / 100 / 110 / 120 | 1 / 2 / 4 / 5 | 13 / 15 / 17 / 18 |
| 130 / 140 / 150 | 6 / 7 / 8 | 20 / 22 / 24 |

  From 130% jab 1 > jab 2 leaves at least 6 frames (the source's "jab 1 > jab 2 > axe from 130%", the axe out on frame 6) on both
  bodies: tested. Jab 1 alone reaches 6 frames at 130% too (the source's "jab 1 into the axe at kill percent on floaty lightweights").
  **None of the phase-1 tilts reaches** the popped-up foe after either (the up, forward and down tilts all miss: the foe is 2-3 units
  away and rising); the axe (Timber, phase 2) and the aerials are what the advantage is for. Checking the axe follow-up is a phase-2
  test.

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/pockets.rs` (19): every normal's active windows, FAF, landing lag and autocancels in the data and by running them (the
projectile normals' first hits on 9 / 25 / 10 / 13); damage, angles, BK / KS, hitlag and SDI multipliers, set knockback and weight, the up
tilt's +2 and its head intangibility, targets, trips, angle types, shield damage, no-clank and no-rebound boxes; the pot's last frame 22
on the stage and 68 off a ledge, the ball's 30 and its 1.2x full charge (18%); the turnips drawn about 40 / 30 / 30 out of 600; grab on
14, throws on 15 / 11 / 13 / 24 with their damage, pummel 1.3%; the down smash's bury; jump air times within a frame; the body; the
registration; random play deterministic and rolling back. `pockets_specials.rs` (19): the rocket (appears on 20, moves on 52, FAF 36,
7% on 52 and gone, slows and drops, bursts at its end and on a wall, one at a time, the air stop), the ride (from 16, still to 52, then
flying; 15.4% and the bounce, not helpless; the hop and the end helpless; the wall crash), the balloons (two, a frame of helium a frame,
flap and sink and steer, 300 frames then helpless, attack lets go, 20 frames of landing, a hit on the balloons pops one and not the body,
the last into the fall with a hop, one balloon below half a tank, the refill, an empty tank, a hit on the body ends it), the
resimulation. `older_rules.rs` (11, above). `scripted_moves::the_balloons_register_reads_the_balloons_carried_in_a_balloon_move`,
`buckler_specials::a_lodged_arrow_keeps_the_angle_it_hit_at`, `trig::degrees_of_finds_the_direction`, the checksum cases
(`launch_older`, `balloons`, `balloon_hp`, `helium_used`, `item_angle`), `ground_attributes` (the settler added).

### Estimated or limited [E]

* Positions on the hand, tool, hip and item bones (every normal except the `top`-bone boxes); the shared body size.
* The jab's held rule (gap 9); the 1.1.x jab's boxes placed at 13.0.1's reach.
* The pot's and ball's flights, the pellets' speed and drop (gap 8).
* The rocket: its appearance, hold frame, speeds, the ride's numbers, the bounce, hop and crash (gaps 1-3); it is not knocked out by
  attacks or reflected while ridden; the wall burst uses the end burst's radius (7.0, not 6.5 at 3.1 up).
* The balloons (gaps 4-7): flight time 300, refill 6 / 16, one balloon below 150, gravity 0.06, fall 1.0, flaps 1.4 / 0.9 every 6 / 12,
  take-off 1.6 for 3, no fast fall, circle 0.55 at 3.0 up, any hit pops one, the pop's hop 1.0, ledges from frame 3.
* `direct=f` not modelled (the firework's user is frozen by its hits); `noprone`; the trip and face-down get-up attacks; the grabs'
  aerial boxes; throw invincibility and bystander hits (house limits).

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace` (all green: 1,325 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings`;
`pftool fuzz-rollback 300`: 0 mismatches, class 10 drawn 118 times; `pftool net-fuzz 100`: 0 mismatches, class 10 drawn 14 times
(both run after merging `movesets` 336b974). Godot tests were updated (rosters) but not run here.

### Training-mode checks (hitbox view)

1. The jab: settler vs. duelist at 130%, jab 1 then jab 2 on its frame 7: the duelist stays red through both, then rises out of reach.
2. The balloons: up special, tap special every 6 frames, let it run out (300 frames); hit the balloons with a forward air pellet from
   below and above (the circle above the head).
3. The rocket: tap toward a wall below the stage's edge (the burst), hold to ride it into a standing foe (the bounce).

### For the art agent

* **Clips needed** (they fall back to shared archetypes today): the umbrella forward tilt (8-11), the stick up tilt (7-18 swing, 22-25),
  the weed down tilt (9-11), the trip-and-pot dash attack (pot out on 8), the bowling-ball lift and drop (charge 16-17, ball on 24),
  the firework up smash (hits overhead 12-34), the shovel down smash (in front 9-11, behind 29-31), the slingshot forward / back air
  (pellet on 9 / 12), the turnip up / down air (6-21 overhead, 11-26 below), the spin neutral air, the boxing-glove jab (both hands,
  3-4), the rocket pull-out and ride (rocket appears on 19, ridden 16-179), the balloon flight (two balloons over the head, flapping).
* **Props**: the flowerpot, bowling ball and slingshot pellet are projectiles (`projectile_*` accessors), the rocket a projectile of tool
  kind 4; the balloons: the read-only bridge accessor `fighter_balloons(i)` (2, 1 or 0) for drawing them over the head (a circle of
  radius 0.55 world units, 3.0 above the feet, in the sim).
* **The arrow's lodge angle**: `item_angle(i)` (degrees counter-clockwise from the right, world axes; 0 if never lodged).

## Phase 2 plan: Pocket and Timber's planting

* **Pocket** (neutral special; intangible 5-23, catches 8-23, FAF 51; the throw: item out on 9, FAF 31 [R]): catch any projectile (and
  item) touching a box in front, keep it (hashed `Fighter::pocket`: the projectile's spec, owner's weapon and move, its power), show it,
  and fire it back on the next press (the move's own throw with the stored shot at x1.5? the multiplier and speed to research). Needs a
  "catch" detector like the reflector's (E10) and a stored-projectile state: a broad change (explain first). Items: pocket a held / lying
  item (E4).
* **Timber**, part 1: plant (FAF 38), water (the 0% push, FKV 25, `flinchless`: E2), the tree growing (18% clean 14-19, 13% 20-40 [R]),
  needs a stage object with hp (a new "placed article" with a hurtbox; Pocket and the tree both need articles: one extension).
* Tests: the axe follow-up of the older jab at 130% (the claim above).

## Phase 3 plan: the axe and the falling tree

* The axe (6-7 early, 8, FAF 48; 14%, 361, BK 36, KS 100 [R]), chopping the tree (hits on the tree's hp), the falling tree (25%, 60,
  14-313 [R]), the wood chip (an item).

## Questions for the owner

1. **The balloons and the rocket's ride have almost no published numbers** (flight time, flap, sink and steer speeds, the balloons' size
   and hit points, the recharge; the rocket's speeds and the ride). They are labelled [E]. Footage of the reference (a recording of a full
   balloon flight from the ground, one with a popped balloon, a ridden rocket across a stage) would let me fit them. Keep the estimates
   until then?
2. The older jab's **held** rule (does holding attack repeat at the end of each punch, or sooner?): estimate kept.
3. E40 changes only per-hit rules. Should the older rules also bring the previous version's shield hitlag (x0.8 above 1) and its
   electric hitstun frame? (Neither matters for the jab.)
