# Project audit for the seventh moveset (reference: the reference game's Cloud)

What the engine needs to reproduce the reference game's Cloud 1:1 (version 13.0.1), traced through the code at **sim version 77**
(content schema 5, branch `movesets`, commit `f41e3d7`). Nothing is built: this is research only. The data is in `REFERENCE_DATA.md`;
the engine inventory is `docs/movesets/SHARED_MECHANICS.md`; the extension ids are those of `docs/movesets/ROSTER_STATUS.md`, with new
ones numbered from E28. Owner decision 9 (the gauge and any Limit Break reset to empty on a stock loss) is taken as given.

## Proposed names (not registered)

| Thing | Name | Checked |
| --- | --- | --- |
| Class / weapon | **Greatsword** | `content/blocklist.txt` has no terms (comments only); contains none of the `TRADEMARKS` terms in `sim-content/src/policy.rs` after its look-alike folding; not used anywhere in the code, content, `godot/scripts` or docs |
| Body | **vanguard** | same |
| Built-in fighter | **Vanguard** | same |
| The gauge and its powered state | **Surge** (the "Surge gauge"; "surging" when full) | same; avoids the reference's own word for it |
| Blurb | "A huge blade and a rising gauge: long reach, a slow wave that crosses the stage, a combo slash that only carries on when it lands, and a surge that turns every special into a finisher." | original wording |

Alternatives if the owner prefers: weapon "Broadblade", body "drifter". Avoided on purpose: the reference's move and item names (the
sword's own name, "Limit", "Cross Slash" and the rest). Class index 6 when registered (after the Staff at 5; `CLASSES` becomes 7), with
the HUD emblem and the gauge's display left for the art agent.

## Iconic moves: what must match exactly

1. **Climhazzard** (up special), normal and Limit. Part 1: the stab on 7 (3%, set knockback 80 at 40 / 78 / 102, H 2.5, SDI 0) and the
   rising slash 10 / 11 / 12-15 (4%, set weight, FKV 121 then BK 98 / 50 / 70); the second press in **20-26** (Limit **33-39**); the rise
   at height multiplier x1 grounded / x1.1 aerial (Limit **x1.6 / x1.85**), horizontal speed x0.6 (Limit x0.67) on starting; ledges from
   **frame 24** (Limit: throughout); the special fall at air speed **x0.6** (Limit x0.8) with **20 frames** of landing lag; Limit
   intangible 5-12, its 6% / 7% (KS 142, then 110) hits. Part 2: the straight fall at **4.2** a frame (all horizontal speed cancelled),
   the **aerial-only 262-degree meteor** (FKV 110) and the 45-degree box from its frame 12, the **landing hit** (3.5%, 64, BK 64, KS 128)
   and **26 frames** of landing lag, ledges only after a fall distance, no cancel. **Engine**: everything exists except the ledge-after-a-
   fall rule (an **E11 addition**) and the rise height itself, which is not published (owner decision 12: fitted, [E]).
2. **Cross Slash** (side special). Hit 1 on 10-11; hit 2 only if hit 1 connected (shield included) and special is pressed from frame 10;
   part 3 (hits on its 2-3, 11-12, 25-26) only if hit 2 connected and special is pressed from its 14; FAFs 41 / 32 / 56; damages 4 / 3 /
   3 / 3 / 6. Limit: all five hits by themselves, intangible 6-11, FAF 61, momentum held in the air. **Engine**: exists (the Longsword's
   Dancing Blade pattern: `hit`, `special_tap`, `goto`); hitbox sizes and positions are [E] (no published table) and the Limit damage is
   a conflict.
3. **Finishing Touch** (the Limit down special). Hits on 12 / 13 / 14 grounded and 14 / 15 / 17 aerial (13.0.1), 1% at 80 degrees, BK
   60, KS 431 / 406 grounded and 371 / 349 aerial (weaker behind), H 2.0; the windbox both sides 22-26 / 25-29 (FKV 80); FAF **85 / 88**;
   the aerial hover (speeds x0.4, gravity 0.03, max fall 1.0); the gauge spent on the attempt. **Engine**: data, E2 pushes, E26 by
   position, E11 plus one **E11 addition** (`start_speed_y`); routing through E9.
4. **The Limit gauge** (our Surge). 100 units; **0.667 a percent dealt, 1 a percent taken**, nothing for shield damage; Limit Charge
   **0.3 a frame**; a hit during the charge or its cancel **drains** instead (and every hit while grabbed out of it); full: the stat
   multipliers (walk x1.15, dash and run x1.1, traction x1.15, air speed and acceleration x1.2, air friction x1.15, gravity and fall speeds
   x1.1, jump speeds unchanged) and the next special is its Limit version; spent by any Limit special; **900 frames** to expire; **reset
   on a stock loss** (owner decision 9). **Engine**: none of it exists: **E9**.
5. **Up air** (a single hit: 11% on 8 over the head, 11% on 9 on the blade, 8% on 10-23; landing lag 9, autocancel 1 / 32+) and **down
   air** (the aerial-only 270-degree meteor 15% BK 10 KS 100 on 11-13, 13% at 60 otherwise, 8% late 14-39; landing lag 16, autocancel
   1-4 / 46+). **Engine**: data (sweet / sour phases, `targets`). **Forward air** (the 270 aerial-only meteor sweetspot on 18-19) and
   **back air** (angle type 4) also data.
6. **Blade Beam**, normal and Limit. The windbox 16-17, the beam from 18 (8 / 6.4% clean to 35, 6 / 4.8% late), speed 2 / 1.6, life 50,
   angle type 1, the aerial fall break; Limit: intangible 10-17, a faster no-clank beam that **bursts on its first fighter or a wall** into
   five 2% vortex hits every 6 frames and a 3% finisher (19% on the first target, 13% behind). **Engine**: the normal beam needs a tall
   projectile hitbox (**E3f**) and a land switch (**E22b**); the Limit burst needs **E3e** and angle 366 (**E28**).

Lesser points that must also be on [R]: the forward smash's three single-frame hits (set weight, aerial / grounded boxes), the up smash's
hilt sourspot priority, the down smash's two hits (the second backward), the hit-then-throw throws, the down throw's 10-frame freeze,
the jab chain's continuable windows, the down tilt's slide, the wall jump.

## How the moves flow (traced)

As in the Palutena and Falcon audits: `fighter::update` → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` /
`start_special` → `begin_attack` → `fighter::attack` each frame → `combat::resolve_hits` → `grab::update` →
`combat::update_projectiles` / `spawn_projectiles` → ledge phase → `combat::check_ko` (`sim-core/src/step.rs`). Points that matter here:

- **Where a body's numbers are read.** `step.rs` phase 1 looks up `params_of(content, f)` (a `&FighterParams` from the content, by
  `char_id`) and hands it to `fighter::update`, which does all ground, air, jump and hitstun physics with it. The ledge phase copies it for
  `fighter::trump` / `grab_ledge`. `combat.rs` has its own `params_of` and reads only weight, `hitbox_scale`, the ECB (hurtboxes, chest
  height) and the weapon index. **Nothing can change a body's numbers during a match** (SHARED_MECHANICS section 6, "Runtime stat
  changes: none").
- **Where damage lands.** `combat::apply_hit` adds `hb.damage * damage_mult` to the victim's percent (after staleness and the short-hop
  factor; a shield returns earlier through `block`; a counter catch returns earlier through `catch_with_counter`). Every damaging path
  goes through it (melee, projectiles with their owner as `source`, throws, counter answers) **except** `combat::pummel` (pummels) and
  the grab parry's 1% in `resolve_hits`. A gauge hook needs those three places.
- **A stock loss** is `combat::check_ko` → `combat::respawn`, which **rebuilds the fighter with `Fighter::spawn`** (keeping only the
  stocks, the character, the facing and the input history). Any new `Fighter` field starts from its default on every respawn, so owner
  decision 9 comes for free once the gauge lives on `Fighter` (a test must still pin it).
- **Which slot a special uses.** `fighter::start_special` picks the direction slot, then swaps to `Move::air_version` when airborne
  (E21's), then `begin_attack`. A Limit version is one more swap in front of that one.
- **Scripts** see a snapshot world (`scripting::World`) and their fighter; `hit` is `hit_mask != 0`, which `resolve_hits` sets for every
  struck target **including a shield** (the bit is set before `block`), so "the follow-up only on hit (shield included)" reads exactly.
  There is no script register for the grab button (`buttons::GRAB` exists in `input.rs`).
- **Projectiles** have one hitbox at a time (phases by age), are removed by their first hit unless `pierce` (E3a), and are removed by a
  solid wall; their angle type 1 (`ANGLE_BY_TRAVEL`) uses their own travel (`combat::launch_side`).

## Normals: what exists

| Needed | Exists? | Decision |
| --- | --- | --- |
| Body: walk acceleration 0 + 0.1575, run 0.011 + 0.099, initial dash 2.145 for 10 frames, run 2.167, traction 0.106, air 1.155 with 0.01 + 0.06, air friction 0.009, gravity 0.098, fall 1.68 / 2.688, hops 32.5 / 17 / 32.5, ledge jump 36.25, heavy landing 3, weight 100, dodges (UFD) | yes (`FighterParams`) | `FighterParams::vanguard`, as `seer` / `sprinter`; jump air times 37 / 49 / 26 / 34 checked by test |
| Wall jump 1.3 / 2.4 | yes (E15) | `wall_jump` 1 |
| Jab 1 → 2 → 3 with continuable windows 8-30 and 9-30 | yes (`next`, `next_from`, `next_window`) | data; jab 1 repeated while attack is held from frame 16 ("consecutive jab") by a one-line script `goto` [R reading] |
| Jab 1's fighters-only 180-degree box | yes (the Staff's jab id 2: a plain box; "objects only" id 3 not modelled) | data |
| Forward tilt, up tilt (by position, sweeping from behind), dash attack (clean / late, shield +1) | yes | data; shield damage as a percent [D] |
| Down tilt slide with the back leg intangible 7-17 | slide: yes (data motion); the leg: **partly** | the sword weapon has no limb circle (`Weapon::limbs` false), so E19's `PART_LIMB` is the middle circle again and the window acts on nothing, as the Staff's arms (owner decision 26). Mapping it to `PART_LOW` would make the whole lower body intangible: **question 6**. The 20% trip cannot happen (ssbwiki.com): no E20 |
| Forward smash: three single-frame hits, set knockback with set weight, separate grounded / aerial boxes | yes (`fixed_kb`, `set_weight`, `targets`, hit groups) | data |
| Up smash with the hilt sourspot winning (lower id) | yes (`Hitbox::priority`) | data |
| Down smash hit 2 backward (angle type 4) | yes (E26) | data |
| Aerials: sweet / sour phases, aerial-only meteors (forward air, down air), the back air backward, landing lags 9 / 11 / 8 / 9 / 16 and autocancels | yes | data |
| Forward air's "ledges from frame 78" | **no** (no aerial ledge lockout in the engine; an aerial already catches ledges after its FAF) | a shared limit, listed |
| Grabs (standing / dash / pivot), pummel on frame 1 | yes (pummel one frame late: house rule) | data |
| Forward, back, up throws with a hit before the throw; back throw turning round on 15 | yes (`throw_after_hit`, `turns_around`) | data |
| Down throw's 10 frames of hitlag on frame 17 | not as such | the Staff's reading: a 0% `HIT_PUMMEL` row on 17 with a hitlag multiplier giving 10 frames (`floor(6 x 1.7) = 10`) [D] |
| Ledge attack, get-up attacks (face up / face down / trip) | yes (one get-up slot) | the face-up one; the other two are the known limit |
| No-clank boxes (pummel, throws' hits, ledge and get-up attacks, the beam windbox, Climhazzard part 2, the Limit beam) | yes (E3c `Hitbox::clanks`) | data |

## Specials: what exists, and the exact shape of each needed extension

### E9: the gauge and its powered mode (new; a broad change, to be explained to the owner before it is built)

**What exists**: `Fighter::stored` (one `u8` set only by a move script, cleared by a KO) is the closest thing; it cannot hold a fraction,
gain from damage, run a timer, or change the body. Bodies are immutable content.

**The exact shape** (every default leaves existing content and fighters unchanged):

* **Content, on the body** (`FighterParams`, `sim-core/src/content.rs`), a new copyable block `gauge: Gauge` with
  `Gauge { max: Fx, per_dealt: Fx, per_taken: Fx, mode_frames: u16, walk: u16, dash: u16, run: u16, traction: u16, air_speed: u16,
  air_accel: u16, air_friction: u16, gravity: u16, fall: u16 }`. `max` 0 means "no gauge" (every existing body); the multipliers are
  percents (100 = unchanged). The vanguard: max 100, per_dealt 0.667, per_taken 1, mode_frames 900, walk 115 (speed and both
  accelerations), dash 110 (`dash_speed`, the initial dash), run 110 (speed and both accelerations), traction 115 (every field built from
  traction: `ground_friction`, `dash_attack_friction`, `landing_friction` [D]), air_speed 120, air_accel 120 (both parts), air_friction
  115, gravity 110, fall 110 (`max_fall_speed` and `fast_fall_speed`). It sits on the body because the multipliers are body numbers and a
  created fighter of this class (a recipe) copies its class body, so it carries the gauge too. Content text: a `gauge { ... }` block in
  the fighter params; **schema 6** with a migration that adds nothing (an absent block is `max 0`), `docs/CONTENT.md` lines, and
  `FighterParams::hash_into`.
* **Content, on the move** (`Move`, `sim-core/src/moves.rs`): `limit_version: Option<u8>` (the slot used instead while the gauge is full;
  default none) and `gauge_drain: bool` (a hit taken during this move drains the gauge instead of filling it; Limit Charge and its cancel).
* **State, on the fighter** (`Fighter`, `sim-core/src/state.rs`, each in `hash_into` with a checksum case in `state.rs`'s tests):
  `gauge: Fx` (0 to `max`), `gauge_mode: u16` (frames left of the powered mode; 0 = not powered), `gauge_drain: bool` (grabbed out of a
  `gauge_drain` move: hits drain until released). All three reset by `respawn` (decision 9) without code.
* **Gain hooks** (one helper, `combat::gauge_hit(state, content, source, victim, damage)`, called where the percent rises: `apply_hit`,
  `pummel`, the grab parry): the attacker gains `per_dealt x damage` (a fighter hitting another fighter; nothing on shields or counters,
  which return before the percent rises); the victim gains `per_taken x damage`, or loses it when its current move has `gauge_drain` or
  its `gauge_drain` flag is set (floored at 0). `damage` is what was added to the percent (after the 1.2 multiplier and staleness:
  conflict 8, [E]). Reaching `max` while not powered starts the mode: `gauge = max`, `gauge_mode = mode_frames`. While powered nothing
  more is gained or drained.
* **The drain flag**: set in `grab::connect` (an ordinary grab, not E6's command grabs: the reference's rule) when the victim was in a
  `gauge_drain` move; cleared when the hold ends (`fighter::free_held` / `grab::drop_grab`).
* **The timer**: `fighter::update` counts `gauge_mode` down every frame (hitlag included [E: gap 12]); reaching 0 empties the gauge.
* **Spending**: `start_special`, before the `air_version` swap: if `gauge_mode > 0` and the chosen move has a `limit_version`, use that
  slot (then its own `air_version`), and set `gauge = 0`, `gauge_mode = 0` at once ("even if the move misses or is interrupted"). Part 2
  of Climhazzard and the Cross Slash follow-ups are `goto`s, not new specials, so they never spend.
* **Runtime stat multipliers**: a function `FighterParams::powered(&self, f: &Fighter) -> FighterParams` (by value) that returns `*self`
  unless `gauge_mode > 0`, else a copy with the multiplied fields (`x * percent / 100` in fixed point: 0.1575 x 1.15 = 0.181125 is exact to
  1/65536 [D]). `step.rs` uses it where it now passes `params_of(content, f)` to `fighter::update` and in the ledge phase; `combat.rs`
  keeps the plain body (weight, hurtboxes and hitbox scale do not change). **Jump launch speeds are not multiplied** (the reference's
  jumps are lower in Limit Break because gravity rises) [R reading]. A copy per fighter per frame is cheap.
* **Script surface (E16 additions)**: registers `gauge` (0 to 100) and `surging` (1 while powered), and the function `gauge_add(n)`
  (adds `n` units, capped at `max`, may start the mode; a negative `n` drains). Limit Charge's 0.3 a frame is `gauge_add(0.3)` in its
  script. Also the register `grab` (grab button held) for the Limit Charge cancel by grab. Each: a whitelist entry in
  `sim-script/src/api.rs`, its implementation in `sim-core/src/scripting.rs`, a test in `sim-core/tests/scripted_moves.rs`, a doc line
  in `docs/CONTENT.md`.
* **The view**: the HUD shows the gauge above the percent (and over the fighter while charging): presentation only, for the art agent,
  read through the bridge like the percent. The GDScript must not compute anything from it.
* **Rollback**: everything is `GameState`; nothing changes the wire format (inputs only), so `pftool net-fuzz` should stay at zero (run it
  anyway).
* **Tests** (in the fighter's new test file and a shared `gauge.rs`): gain from a dealt and a taken hit; nothing from a shielded hit;
  Limit Charge's 334 frames from empty; a hit while charging drains by its damage; a grab out of the charge drains on the pummel and the
  throw, and stops after release; the mode starts on filling, the multipliers apply to walk / dash / run / traction / air / gravity /
  fall and not to the jump speed; a Limit special is used instead and empties the gauge; the 900-frame expiry; a KO empties the gauge
  and the mode (decision 9); a body with `max 0` never changes (every other fighter's kit tests and `base.pfc` equality); checksum cases
  for the three fields; a rollback replay with the mode running.

### Blade Beam: E3f (tall projectile hitbox), E11 `start_speed_y`, E22b (on-land switch)

* What exists: the projectile spec (speed 0.25 / 0.2 world a frame, life 50, a clean phase to age 18 and the late phase after: E3a's
  phases), spawned on the move's frame 16 (the reference's 17) so its first hit is on 18; angle type 1 for projectiles; reflection;
  `Hitbox::push` for the body windbox, grounded fighters only; `air_version` for the aerial beam; E11 `air_gravity` 0.06 and
  `start_speed` 40 for the air; no ledge grab during the move (the default).
* **E3f (new, small): a tall projectile hitbox.** The beam is two reference circles stacked into a wall about 12 units tall (Limit: 19),
  and a projectile has one circle. Shape: `Hitbox::half_height: Fx` (default 0 = a circle): the box is a vertical capsule from
  `y - half_height` to `y + half_height`, tested with the same circle-against-capsule code the reflector's `half_height` (E10c) already
  uses. Read for projectiles in `update_projectiles` (and, for free, by melee boxes if wanted later). Content field, hashed with the
  hitbox; tests: a target above a circle but inside the capsule is hit. Without it the beam would be one circle r ~ 6 (wider than the
  reference's beam front to back) [E]: **question 4**.
* **E11 addition: `start_speed_y: u8`** (percent of the vertical speed kept on an aerial start, default 100): the beam's x0.65 (the fall
  break), Finishing Touch's x0.4, Limit Cross Slash's 0. One line beside `start_speed` in `start_special`.
* **E22b (new, small): `Move::on_land: Option<u8>`**, the mirror of E22's `off_edge`: an aerial move touching the ground switches to that
  slot on the same frame, keeping its frame, speed and hits. The reference: "if he lands before firing the beam, [it] counts as ... the
  grounded version". The aerial beam carries `on_land` = the grounded beam (which spawns the grounded projectile when its frame comes).
  Today the choice is a `goto`, which restarts at frame 0. Also the likely answer to Palutena's gap 4 and to the aerial Finishing Touch.
* The grounded x0.2 speed: a frame-0 `set_vel` in a one-line script [E: a slide that keeps a fifth of the speed].

### Cross Slash: exists (scripts and data)

* Hit 1 (side special slot), hit 2 (`ext`), part 3 (`ext`). Scripts as the Longsword's Dancing Blade: hit 1 remembers `special_tap`
  from its frame 10 (move frame 9), and from then on `goto`s hit 2 if `hit`; hit 2 the same from its 14 into part 3; otherwise the move
  ends at its FAF (41 / 32). Part 3 is plain data (three hits in three groups, FAF 56). `hit` counts shields, as the reference's
  continuation does [R reading]. A whiffed hit 1 simply ends; pressing again starts a new hit 1 (the reference's "first slash
  repeatedly").
* Hit 1's step forward: a data motion [E: gap 13]. Its 366 box: E28, or 367 if E28 is not built (conflict 2).
* Sizes and positions of every box: **[E]** (no hitbox table: gap 1); placed on the sword's arc in each pose. The left / right
  animation difference is not modelled (a mirrored move; the difference is unpublished).
* **Limit Cross Slash**: one `ext` move, five hits in five groups on UFD's frames (conflict 1), intangible 6-11 (`intangible_from` 5,
  `intangible` 10, both counted as the house does), autolink 367 on hits 1-2 (exists), FAF 61; in the air `start_speed` 0,
  `start_speed_y` 0 and `air_gravity` 0 until the end ("halts all momentum") [E for the after-move].

### Climhazzard: exists, plus one E11 addition

* Part 1 (up special slot): the rise as a data motion fitted to the [E] height (owner decision 12) with `rise_ground` 100 / `rise_air`
  110; `start_speed` 60; the hits as data (set knockback, set weight, SDI 0, H 2.5); a script that `goto`s part 2 when `special_tap` in
  20-26; `helpless_after` with `fall_drift` 60 and `fall_landing_lag` 20 (E11, exists); `ledge_front_from` 24 (the reference's 24, so
  move frame 23), `ledge_back_from` [E: the same]. The "initial thrust" ledge window before 24 waits for gap 6 (if it exists it needs a
  frame range, not just a start).
* Part 2 (`ext`): `set_vel(0, -0.525)` every frame (horizontal speed cancelled), the two falling boxes from its frame 12 (the meteor
  aerial-only), no clank; `landing_move` = the landing hit (`ext`, its length the 26 frames of lag); `helpless_after` at 61 (the
  helpless fall then uses the body's fall speed instead of keeping 4.2: a listed limit, it only matters after a full second of falling).
* **E11 addition: ledges after a fall.** Shape: `Move::ledge_drop: Option<(Fx, Fx)>` (front, back: how far below the height the move
  started at the fighter must be before a ledge in front / behind can be caught; default none) and a hashed `Fighter::move_y0: Fx` set by
  `begin_attack` (also on a `goto`, so part 2 measures from where the descent began). Checked in `fighter::attack`'s ledge test beside
  `ledge_front_from` / `ledge_back_from`. Part 2's values: "slightly less than" the normal rise (front) and the Limit rise (back) [E: gap
  7]. Tests in `edges_and_recovery.rs`; a checksum case for `move_y0`.
* **Limit Climhazzard** (`ext`, the up special's `limit_version`): its hits, intangible 5-12, `rise_ground` 160 / `rise_air` 185,
  `start_speed` 67, the second press in 33-39 into the same part 2, ledges from frame 1 front and back [R / E], `fall_drift` 80,
  landing lag 20, and an `air_blocks` air control while rising [E: gap 8].

### Limit Charge: E9, E16 `gauge_add` / `grab`

* The down special slot: a script that, from the charge's first frame [E: gap 9], calls `gauge_add(0.3)` each frame and `goto`s itself
  to stay under the 60-frame stall cap (the Needle Storm pattern; `vars` remember it has started). `gauge_drain` set. Cancels: `special_tap`,
  `shield`, `jump`, `grab`, the stick pushed sideways (a roll), in the air `shield` (an air dodge) → `goto` the cancel move (`ext`,
  FAF 8, `gauge_drain` set), whose end lets the held or buffered input act (the 9-frame buffer covers a press on its first frame). A full
  gauge while charging: the charge ends into the cancel [E: gap 10]. In the air: the body's air physics until gap 11 gives numbers.
* The `limit_version` of the down special is Finishing Touch (below).

### Finishing Touch: data, E2, E26, E11 `start_speed_y`

* Two `ext` moves (grounded, and its `air_version`), one hit group across the phases (radius-0 gaps are just frames with no box),
  KS 431 / 406 and 371 / 349, H 2.0, clanks; the windbox as an E2 push with angle 361 by position (both sides) on 22-26 / 25-29, no clank;
  FAF 85 / 88; air: `start_speed` 40, `start_speed_y` 40, `air_gravity` 0.03, `air_max_fall` 1.0. Landing during the aerial one: E22b
  `on_land` into the grounded one [E].

### Limit Blade Beam: E3e (burst on contact), E28 (angle 366), E3c, E3f

* **E3e (new): a projectile that bursts where it hits.** Shape: `ProjectileSpawn::burst: Option<Burst>` with `Burst { life: u8, rehit:
  u8, on_wall: bool, phases: [Option<(u8, Hitbox)>; MAX_PHASES] }` and a hashed `Projectile::bursting: bool`. In `update_projectiles`:
  the first fighter the beam hits takes the beam's own hit (6%); then the projectile stops (`vel` 0), `bursting` is set, `age` restarts
  at 0, `life` becomes the burst's, it becomes piercing with the burst's `rehit`, and its hitbox comes from the burst's phases (the loop
  in group 1, the final in group 2, so the first target is hit again). With `on_wall`, touching a solid wall bursts it there instead of
  removing it (the stage spike). Content: a `burst { ... }` block on the projectile spec. Tests: 6 + 5 x 2 + 3 = 19% on the first target
  and 13% on one standing behind it; a wall burst; reflection of a bursting projectile (the reference: reflectable; [E] whether a burst
  is); a checksum case for `bursting`.
* **E28 (new): angle 366** (the reference's vortex autolink: the victim is sent the way the attacker moves plus a pull to the hitbox's
  centre for 5 frames, without the 367's speed reduction or grounded 80-degree rule; ssbwiki.com `Autolink_angle`). Shape: `ANGLE_VORTEX`
  = 366 in `moves.rs`, handled in `combat::apply_hit` / `fighter::autolink` with the hashed fields autolink already has (`link`,
  `link_vel`, `link_point`) and one more hashed `Fighter::link_kind: u8` (367 or 366) and a 5-frame counter. For the burst (a
  projectile that does not move) it is the pull alone. Without it the loop could use 367 (which also pulls toward the centre, at a fifth
  of the distance) [E]: **question 5**.
* No clank (E3c, exists for projectiles), the capsule beam (E3f), intangibility 10-17 (exists), the aerial version through
  `air_version` and E22b.

## Slots (`ext`, 16 a weapon since E17)

| Use | Slots |
| --- | --- |
| Blade Beam aerial | 1 |
| Cross Slash hit 2, part 3 | 2 |
| Climhazzard part 2, its landing hit | 2 |
| Limit Charge cancel (the charge loops on its own slot) | 1 |
| Limit Blade Beam grounded and aerial | 2 |
| Limit Cross Slash | 1 |
| Limit Climhazzard | 1 |
| Finishing Touch grounded and aerial | 2 |
| **Total** | **12 of 16** |

Four spare, for whatever gaps 3 and 11 show (an aerial Cross Slash or Limit Charge with their own timing). The jab uses `Jab2` / `Jab3`.

## Extension summary

| Id | Status | Cloud needs | Shape |
| --- | --- | --- | --- |
| E2 | built | Blade Beam's body windbox, Finishing Touch's windbox | as built |
| E3a | built (sim 76) | the Limit burst's loop (pierce, rehit, phases) | as built, reused by E3e |
| E3c | built (sim 70) | no-clank boxes, the Limit beam (transcendent) | as built |
| **E3e** | **new** | the Limit beam's burst on a fighter or a wall | `ProjectileSpawn::burst`, hashed `Projectile::bursting` |
| **E3f** | **new, small** | the beam's tall hitbox (normal and Limit) | `Hitbox::half_height` (vertical capsule), read for projectiles |
| **E9** | **new, broad** | the gauge, the powered mode, Limit versions, the stock-loss reset | `FighterParams::gauge`, `Move::limit_version` / `gauge_drain`, hashed `Fighter::gauge` / `gauge_mode` / `gauge_drain`, `FighterParams::powered`, schema 6 |
| E11 | built, **two additions** | Climhazzard, Blade Beam, Finishing Touch, Limit Cross Slash | `start_speed_y`; `ledge_drop` with hashed `Fighter::move_y0` |
| E15 | built | wall jump | as built |
| E16 | partly built; **additions** | Limit Charge | registers `gauge`, `surging`, `grab`; function `gauge_add` |
| E17 | built | 12 `ext` slots | as built |
| E19 | built | the down tilt's leg (acts on nothing on a sword body) | owner question 6 |
| **E22b** | **new, small** | landing during the aerial beam (and Finishing Touch) | `Move::on_land` |
| E26 | built | angle types 1, 3, 4, 0 throughout | as built |
| **E28** | **new** | angle 366 on the Limit burst and Cross Slash hit 1 | `ANGLE_VORTEX`, hashed `Fighter::link_kind` and a 5-frame count |

Not needed: E1, E3b, E3d, E4, E5, E6, E7, E8, E10, E10c, E12, E13, E14, E20 (the down tilt's trip cannot happen), E21, E24, E25, E27.

Every new field defaults to today's behaviour, is hashed (content through `Weapon::hash_into` / `FighterParams::hash_into`, state through
`GameState::hash_into` with a checksum case each), appears in `docs/CONTENT.md`, and gets a test in its topic file; no existing move or
body uses them, so the other fighters stay unchanged (check `base.pfc` equality and the existing kit tests). E9's `gauge` block is a
schema bump (6) with a migration; the rest are new optional fields. One `SIM_VERSION` bump per phase.

Shared limits this port exposes but does not fix (listed, not built): an aerial's ledge lockout (the forward air's frame 78); one get-up
attack slot; the throws' interrupted rows and thrower invincibility; Cross Slash's facing-dependent animation; the helpless fall after
Climhazzard part 2 not keeping its speed.

## Suggested phases

1. **Body and every normal** (`FighterParams::vanguard`, jab chain, tilts, dash attack, smashes, the five aerials, grabs, pummel,
   throws, ledge and get-up attacks; class 6 Greatsword registered in `CLASSES` and `roster.gd`, content re-export). Engine: nothing
   new (all data on built extensions). Specials as placeholders.
2. **The normal specials and the gauge**: Blade Beam (E3f, E11 `start_speed_y`, E22b), Cross Slash (scripts), Climhazzard (E11
   `ledge_drop`), and **E9** with Limit Charge (`gauge_add`, `grab`). E9 is explained to the owner before it is built (the house rule for
   broad changes). After this phase the gauge fills, the mode runs and expires, the multipliers apply and a KO resets it, but no Limit
   version exists yet (`limit_version` unset: a special in the mode would just spend it, so the phase ends with every Limit slot built
   or the spend disabled).
3. **The Limit versions**: Limit Blade Beam (E3e, E28), Limit Cross Slash, Limit Climhazzard, Finishing Touch; then the re-check of
   every iconic move in the hitbox view and against footage for the gaps.

## Questions for the owner

1. **E9's shape** (the first runtime change to a body during a match): the gauge rules and multipliers on the body (`FighterParams`),
   the Limit routing on the moves, the state on the fighter, applied by building a powered copy of the body each frame. Approve this
   shape (and schema 6) before it is built?
2. **Gauge gain basis** (conflict 8): fill from the damage actually added to the percent (with the one-on-one 1.2 and staleness), or
   from the hit's base damage? Footage settles it.
3. **Footage** for the gaps (14 items in `REFERENCE_DATA.md`); the most important for 1:1: Cross Slash's boxes, windows and Limit
   damage (gaps 1-2, conflict 1), Climhazzard's height and ledge rules (gaps 5-7), Limit Charge's start, end and air behaviour (gaps
   9-11). Can you record them (training mode, frame-stepped, hitbox view, the gauge shown)?
4. **E3f** (a tall projectile hitbox) for the beam, or one bigger circle [E]?
5. **E28** (angle 366) for the Limit burst and Cross Slash hit 1, or the existing 367 autolink as a labelled approximation?
6. **The down tilt's leg intangibility** (7-17): leave it as a listed limit (a sword body has no leg circle, like the Staff's arms), or
   make the lower body circle intangible on those frames?
7. **Names**: Greatsword / vanguard / Vanguard, the Surge gauge and the blurb above, or other names?
