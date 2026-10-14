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

## Phase 2 (sim 115, content schema 7 unchanged; branch `pockets-phase2` from `movesets` at cb89aa5, sim 113)

Owner decisions 69-71 (2026-10-14, recorded in `../ROSTER_STATUS.md`): keep the balloon and ride estimates; rework the older jab; no
other previous-version rules. Phase 2 also built what was planned as phase 3 (the axe, the falling tree, the wood chip): the moveset is
complete.

### Milestone 1: the older jab reworked (owner decision 70)

* **Engine**: `Move::cancel_from` (content `cancel_from`, default never): from that move frame, on the ground and out of hitlag, any
  action the fighter could start standing (`fighter::ground` run on a copy: a jump, a dash, a crouch, a shield, a grab, another attack, a
  special; not a walk, not the jab itself) ends the move and starts on the same frame. General: any move can use it.
* **Data**: jab 1 cancellable from its frame 7, jab 2 from 8 (their continuable frames: the previous version's jab cancel by a crouch
  or shield, ssbwiki.com `Jab_cancel`); jab 1's far glove takes 13.0.1's far-box pull (180 degrees forward, BK 15, KS 10); jab 2 stays
  the up-and-out launcher (BK 20, KS 120).
* **What the research found** (sources and the table in `REFERENCE_DATA.md`, "The rework"): the cancel is real and well sourced; **the
  pull is not in the 1.1.x tables** (ssbwiki.com and kuroganehammer.com's 1.1.6 page agree: jab 1 is 60 / 80 degrees, BK 10, KS 80,
  sent away). The pull the owner remembers is 13.0.1's own jab 1 (its 180-degree far boxes, in 13.0.1 since its first versions). It is
  built as asked, as a labelled blend (question 4). At our placement it acts only at the tip: the duelist's body reaches the near
  glove up to about 2.1 units out; between 2.2 and 2.5 only the far glove lands, and draws the foe in under 0.1 a frame.

### Milestone 2: the pocket (E42) and the tree (E43)

* **E42, the pocket** (general): `Move::pocket` (`PocketCatch { start end x y radius }`) and `Move::unpocket` (`PocketRelease { frame x
  y power }`), `Tool::pocket` (`yes` / `never` / `frees`), `ItemSpec::pocketable`; hashed `Fighter::pocket` / `pocket_item`,
  `Projectile::stale` (a shot's stale-move key: the release's for a released one), the rests `REST_POCKETED` / `ITEM_POCKETED`; the
  `pocketed` register (whitelist, `scripted_moves` test, `CONTENT.md`); the bridge's read-only `fighter_pocketed(i)`; pocketed shots and
  items are hidden from `projectile_slots` and the `item_*` accessors.
* **E43, placed pieces** (general): `ItemSpec::placed` (`Placed { x hurt_half_height hitter_lag water grow_delay grow_into grow_move
  cuts fall_move chip chip_chance }`), `Tool::water`, `Move::chops`; hashed `Item::water` / `grow_in` / `cuts`; `combat::fire` (a
  weapon move's projectile fired for anyone, anywhere); the `owns(kind)` function (whitelist, `scripted_moves` test, `CONTENT.md`).
* **The neutral special** (`pocket` in `moves_pockets.rs`, `pockets_pocket.script`): the reach (intangible 5-23, catching 8-23, FAF 51)
  and the release (`POCKET_OUT` = ext9: out on 9, FAF 31, x1.9) [R]; which projectiles: every one in the roster but the needles' line
  and the seer's explosive flame (`never`) and bursts; the rocket `frees` its maker [R]; items: every one but placed pieces.
* **The down special** (`tree`, `pockets_tree.script`, `pockets_shrug.script`): the router by `owns(kind)` (axe with a tree or a
  falling tree out, water with a sapling, else planting, or the failed planting in the air); `PLANT` (ext10, FAF 39), `WATER` (ext11,
  FAF 50, water 5-20), `AXE` (ext12, 6-8, FAF 48, `chops`), `PLANT_FAILS` (ext13); the growth's and the falling tree's shots on ext14 and
  ext15; items 0 sapling (kind 5), 1 tree (6), 2 wood chip (8); the falling tree's tool kind 7. All 16 `ext` slots are now in use.
* `base.pfc` re-exported; `docs/CONTENT.md` lines for every new field; validation for each; round trips
  (`the_pocket_and_tree_fields_round_trip_and_are_checked`) and content-hash cases (`move_rules::the_pocket_and_tree_fields_are_in_the_content_hash`).

### The combo test: the jab into the axe

`older_rules.rs` (`the_axe_after_jab_1_is_a_true_combo_and_after_jab_2_it_has_the_time_but_not_the_reach`, the printed tables), the
settler's tree standing far behind it so the down special is the axe (out on its frame 6), each follow-up pressed on the first frame
the settler can act (the cancel, after any hitlag):

| Percent | Advantage after jab 1 | after jab 1 > 2 | jab 1 > axe | jab 1 > 2 > axe (cancel) | jab 1 > axe, no cancel | jab 1 > 2 > axe, no cancel |
| --- | --- | --- | --- | --- | --- | --- |
| 0 / 10 / 20 | 6 / 7 / 8 | 12 / 14 / 16 | true | true | lands late | lands late |
| 30-100 | 9-17 | 18-31 | true | **misses** | lands late | misses |
| 110-150 | 19-23 | 33-40 | true | **misses** | misses | misses |

(The duelist, weight 90; the settler, 92, gives the same pattern.) **Jab 1 > axe is a true combo at every percent 0-150** (the previous
version's moveset notes' claim, "at kill percents", holds). **Jab 1 > jab 2 > axe has the time from 0%** (36 frames of advantage at
130% against the axe's 6: the forum's frame claim holds and more) **but connects only at 0-20%**: from 30% jab 2 has sent the foe up and
out of the axe's reach. The axe's boxes on the held axe are [E] (the reference attaches them to its axe bone); a higher early swing would
reach further up: question 5. Without the cancel (waiting for each jab's FAF, as the forum's calculator assumed), no axe follow-up is a
true combo. Other strings (`the_cancelled_jab_strings_into_its_next_hit_a_down_tilt_and_a_rising_neutral_air`): jab 1 > jab 2 true at
0-150%; jab 1 > down tilt at 30-100%; jab 1 > jump > neutral air at 10-60%; jab 1 > 2 > jab 1 at 0-20%, > down tilt at 0-10%. The grab
never catches the popped-up foe out of jab 1 > 2, and out of jab 1 only after the hitstun.

### Verified against [R] / [D] (phase 2, by tests that ran)

`pockets_pocket.rs` (11): intangible exactly 5-23 and FAF 51; a shot met on 8 and 23 is put away and not on 7 or 24; it waits frozen and
unseen 600 frames; the release on frame 9, FAF 31, the shot turned forward at its caught speed, power 190, now the settler's, staled as
the release; its hit 1.9x the shot's damage at that age (x1.2 one-on-one); own shots, the line and bursts are not taken; the pocketed
rocket frees its maker, the pocketed boomerang still counts as its maker's; an item put away frozen, still its maker's, out into the
hand on 9 with its clock restarted; the held-item swap; a KO loses either; rollback. `pockets_tree.rs` (14): the planting (the stage
on frame 2, the sapling on 20 at 10 ahead, FAF 39), the failed planting in the air, the sapling's 840 frames and the pull-up by another
settler's planting, the water (0%, a push, no hitstun, FAF 50), **the growth's clean hit 46 frames after the first drop [R: UFD]** for
18%, the late 13%, half a watering kept, the tree's 30 hp and 600 frames, shots used up on it, the axe on 6 for 14% and FAF 48, two cuts
on a side fell (a 7-frame freeze, 16 hp left after one), one a side does not, the falling tree's 25% no sooner than its frame 14 and
gone after its bounce, the wood chip and its 48.5%, the stage staying the axe until the falling tree is gone, rollback with trees
growing. `older_rules.rs` (15). `scripted_moves` (`pocketed`, `owns`), `move_rules`, the sim-content round trip, the checksum cases
(`pocket`, `pocket_item`, `projectile_pocketed`, `projectile_stale`, `item_pocketed`, `item_water`, `item_grow_in`, `item_cuts`).

### Estimated or limited [E] (phase 2)

* The pocket's reach (9.0 / 7.0, r6.0) and release point (7.0 / 6.0); no speed change on release (none published); a pocketed shot holds
  a slot of the shared pool; the release's staleness counts as the release move (ext9); a teammate's shot (x0.25) is not modelled (no
  teams); the reference's heavy items and items that act when picked up do not exist here.
* Which shots are pocketable: the explosive flame `never` is a reading of the reference's rule (it is reflectable in the raw table).
* The sapling's place (10.0 ahead) and frame (20); it cannot be hit; the water needed (8 frames) and the delay (26, fitted to UFD's 46);
  the water's shape and path (one falling shot, pushing every frame); the tree's hurtbox; the axe's boxes on the held axe; the chop's
  freeze (7, conflict 4); the falling tree's path, single box and 255-frame life (313 in the reference), no 15% landing sourspot; the
  wood chip's numbers; no fruit (1.5% of fells in the reference).
* `Move::cancel_from` does not spend the reference's crouch or shield frame before the next action.

### Tests and checks run (phase 2, after merging `movesets` 8759129, sim 114)

`cargo fmt --all`; `cargo test --workspace` (all green: 1,363 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D
warnings`; `pftool fuzz-rollback 300`: 0 mismatches, class 10 drawn 207 times, pockets 113 put away / 84 taken out, placed pieces 294
made / 73 felled (every third run the settler drills its specials); `pftool net-fuzz 100`: 0 mismatches, class 10 drawn 38 times,
pockets 3 put away, placed pieces 47 made / 15 felled. Godot tests not run (no GDScript changed; the bridge gained `fighter_pocketed`
and hides pocketed things).

### Training-mode checks (phase 2, hitbox view)

1. The pocket: a claws' blaster at the settler, neutral special on its frame 8: the shot vanishes, the settler takes nothing; neutral
   special again: out on frame 9, forward, 1.9x.
2. The tree: down special (plant), down special (water) from the same spot, wait: the tree on 46 frames from the first drop; two axes
   from one side: the tree falls away and bounces once.
3. The jab: at 0%, jab 1 then down special with the tree out: the axe lands in jab 1's hitstun.

### For the art agent (phase 2)

* **Clips needed**: the pocket reach (hand out in front on 8-23; the shrug when nothing was caught: compare `fighter_pocketed(i)` before
  and after), the pocket release (item or shot out on 9); the planting (shovel, the sapling set down on 20, 10.0 reference units ahead),
  the failed planting (a shrug in the air), the watering can (water poured 5-20 from about 11 ahead, 9 up), the axe swing (overhead down
  in front: early 6-7 high, late 8 low at 5.5-11.5 ahead).
* **Props**: the sapling and the tree are items (`item_kind` 5 and 6, `item_pos` the base); the tree grows over its first 13 frames (its
  growth hit is from frame 14); the falling tree is a projectile (move ext15 in `projectile_slots`) toppling away from the chopper; the
  water a projectile (move ext11); the wood chip an item (kind 8). The pocket's contents: `fighter_pocketed(i)` (0 / 1 / 2) for an icon
  over the damage meter. Cut marks on the tree (`Item::cuts`) are not exposed: ask if wanted.

## Sim 117: the axe's higher early swing (owner decisions 72-73, 2026-10-14)

* **Decision 72**: keep jab 1's pull (the 13.0.1 blend on the older jab). Nothing changed.
* **Decision 73**: the axe's early swing (the special's frames 6-7) gains a second head box at 12.0 of the reference's units ahead and
  17.0 up, radius 6.0 [E] (the axe head at the top of its arc), the same hit (14%, 361, BK 36, KS 100, +9 shield damage); the reference's
  head and handle boxes stay. Damage and angles unchanged.
* **Result** (`older_rules::the_axe_after_jab_1_is_a_true_combo_and_after_jab_2_it_reaches_to_60_percent`,
  `print_the_axe_after_the_jab`): jab 1 > jab 2 > axe (with the cancel) is a true combo at **0-60% on the duelist and 0-70% on the
  settler** (it was 0-20%); jab 1 > axe stays a true combo at 0-150%. Without the cancel nothing changes (jab 1 > jab 2 > axe lands late
  at 0-20%, misses above).
* **Not at 130%**: there jab 2 has the foe's feet about 32 reference units above the settler's on the axe's early frames (traced frame
  by frame: 30 to 34 up, 19 ahead), nearly two of our body heights; a box reaching that high would sit far over the settler's head, which
  the decision's "within reason" rules out. The time is there (36 frames of advantage at 130%), the reach is not. If the owner wants the
  130% string anyway, the choices are a much taller early box (about 25 units up, radius 8), or a lower jab 2 launch (a change to the
  1.1.x data): an owner question.

| Percent | duelist: jab 1 > 2 > axe | settler: jab 1 > 2 > axe |
| --- | --- | --- |
| 0-60 | true combo | true combo |
| 70 | misses | true combo |
| 80-150 | misses | misses |

## Questions for the owner

1. ~~The balloons and the ride's estimates~~: answered, keep them (decision 69).
2. The older jab's **held** rule (does holding attack repeat at the end of each punch, or sooner?): estimate kept.
3. ~~Other previous-version rules~~: answered, no (decision 71).
4. ~~The jab 1 pull~~: answered, keep the blend (decision 72).
5. ~~Jab 1 > jab 2 > axe~~: answered, raise the early swing (decision 73, sim 117): it now connects at 0-60% / 0-70%. At 130% the foe is
   out of any reasonable reach (above): a much taller box, a lower jab 2, or leave it?
6. The pocket's **reach and release point** and the tree's sizes are estimates (the hitbox footage would settle them); the explosive flame
   is not pocketable (a reading of the reference's rule): right?
