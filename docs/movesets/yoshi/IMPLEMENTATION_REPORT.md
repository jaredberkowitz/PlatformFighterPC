# Implementation report: the ninth moveset (reference: the reference game's Yoshi)

In game: class 8 **Clutch** (weapon `clutch`), body `bounder` (`FighterParams::bounder`), built-in fighter **Bounder** (owner decision
45). Target version 13.0.1. Data: `REFERENCE_DATA.md`; what the engine had and needs: `PROJECT_AUDIT.md`.

## Phase 1 (sim 89, content schema 6 unchanged): the body, its double jump, and every normal

Branch `clutch-phase1`, from `movesets` at e7fd986, fast-forwarded to the Bracers' merge (9837e3d, sim 86), then merged with
`origin/main` at 64147b0 (sim 87: Cloud phase 3 and the Bracers). Sim 88 is reserved for another session; this phase is **89**.
Commits: 506f569 (engine, body, normals, registration), 233c6fe (the merge), ee10962 (tests and doc lines), then the docs commit.

### Engine (general; every default is today's behaviour)

* **E1, the knockback armor kind** (designed with the Bracers, built here so their cargo hold can reuse it). `Armor` gains
  `subtract: bool` and `min_kb: Fx`; `Armor::withstands(frame, damage, knockback)` and `Armor::broken_knockback(frame, knockback)`.
  `ARMOR_KNOCKBACK` withstands a hit whose knockback (the value `combat::apply_hit` works out: rage, staleness and set knockback
  included, before crouch cancelling, so the one-on-one multiplier never touches it) is at most `threshold`; with `subtract` a stronger
  hit launches with `max(k - threshold, min_kb)`, and its hitstun follows the reduced knockback. The validation refusal is gone;
  validation now asks for a threshold above 0 on damage and knockback armor, `subtract` / `min_kb` only on the knockback kind and
  `min_kb` only with `subtract`. Content: `armor { ... subtract true min_kb 60 }`; `Weapon::hash_into` hashes both fields. A push still
  pushes (with its full knockback); grabs never reach it.
* **E33, the air jump's armor and turnaround** (replaces the roster's E12: the 13.0.1 double jump is an ordinary jump, decision 47).
  `FighterParams::air_jump_armor: Option<Armor>` and `FighterParams::air_jump_turns: bool` (content: an optional `air_jump_armor { ... }`
  block and `air_jump_turns true` in the fighter section, inherited by `inherit`; no schema bump since they are optional with defaults;
  `FighterParams::hash_into` hashes them). State: hashed **`Fighter::air_jump_frame`** (checksum case in `state.rs`): 1 on the air
  jump's own frame, one more each `Airborne` frame outside hitlag (saturating), 0 on entering any other state, at the start of any
  non-`Airborne` update, on a wall jump and on a launch. `apply_hit` reads the body's air-jump armor beside the move armor, at window
  frame `air_jump_frame - 1` (move frames counted from 0). A withstood hit leaves the jump going (its speed kept, its count frozen in
  hitlag). With `air_jump_turns`, an air jump with the stick held back past the deadzone turns the fighter round (the drift still
  follows the stick).

### Built

* **Body** `FighterParams::bounder` (`sim-core/src/content.rs`), every value [R] from `REFERENCE_DATA.md` "Attributes": weight 104;
  walk 1.208 (accel 0.105 + 0.168), initial dash 1.98 for 14 frames, run 2.046 (0.044 + 0.1056), traction 0.095; air speed 1.344, air
  accel 0.03 + 0.068, air friction 0.005, gravity 0.08, fall 1.29 / 2.064; full hop 36.09, short hop 14.43, **double jump 51.56**,
  ledge jump 38.045; heavy landing lag 6; air dodges 58 (3-30) and 79 / 107 / 130 (3-21); rolls 30 / 35, spot dodge 26; ledge decay 82
  / 50; no wall jump (wall tech 1.3 / 2.3, the shared one); regular shield (decision 5); the shared body size [E]. **Double jump**:
  `air_jump_armor { start 0 end 43 kind knockback threshold 120 subtract true min_kb 60 }`, `air_jump_turns true`.
* **Every normal** (`sim-core/src/moves_clutch.rs`, a child module of `moves.rs`), read from ssbwiki.com's **raw** tables (re-fetched
  2026-10-13; they agree with `REFERENCE_DATA.md`). `Weapon::limbs` true (kicks, a tail, a head).
  * **Jab** 1 on 3 (3%, the 180-degree fighters-only pull box at the tip, H 1.5, FAF 18), jab 2 on 3-4 (4%, H 2, FAF 20); the
    continuable window from frame 6 by a press (`next_from`); **held after a hit from 6, held without a hit from 16**
    (`scripts/clutch_jab_1.script`). No rapid jab, no third hit.
  * **Forward tilt** 5-7 (8%, 88 / 85 / 85, three aims with the same numbers), **up tilt** 8-15 (7%, 100, BK 72 KS 45, by position; the
    tail high in front, overhead, behind), **down tilt** 8-10 (5 / 4.5 / 4%, 28 / 28 / 361, **trips 30 / 30 / 40%** via E20), **dash
    attack** 10-12 / 13-20 (11% KS 45, 8% KS 44).
  * **Forward smash** 14-16 (head 15.5% KS 102 over nose 14% KS 97, three aims at the [D] root offsets), head and nose intangible
    12-16, charges 7-8, FAF 53. **Up smash** 11-13 / 14-16 (14 / 12%, 75), the kicking foot intangible 11-13, FAF 47. **Down smash** 7-8
    in front and 22-23 behind (backward type), charges 3-4, FAF 50.
  * **Neutral air** 3-4 / 5-11 / 12-25 (10 / 7 / 5%, capsules [D]), **forward air** 16-20 (head 15% at 361 over the 270-degree snout
    meteor 14%, on everyone), **back air** 11 and 14 (3.5% set-weight autolinks, H 0.5, SDI 0.5) and 18-19 (5.5%, 54, KS 184, H 1.5), all
    backward, **up air** 5-6 (12%, 90), **down air**: twelve one-frame autolink kicks on 16-38 (2.3 / 1.8 then 1.9 / 1.4, KS 80 then
    100, H 0.6, SDI 1.8; the kick group re-armed every 2 frames, `rehit (15, 2)`), the finisher on 41 (2.8%, 80 / 40, BK 55 KS 179, H
    1.5), **28%** in all; landing outside its windows starts **the landing hit** (`ext3`: 1% on the landing frame, FAF 18). Landing lags
    7 / 12 / 11 / 8 / 17 and every autocancel window on [R].
  * **Grabs** (the tongue): standing 14-21 (FAF 49), dash 16-23 (57), pivot 17-24 (52); the mouth box on the first two frames [D] and the
    tongue tip's grounded box swept along an [E] reach. **Pummel** 1.3% (FKV 30, H 2.1, frame 2 / FAF 8 by the house shift, no clank).
    **Throws**: forward 15 / 40 and back 20 / 44 (turns; 9%, 45, BK 55, KS 66), up 13 / 44 (5%, 90, BK 65, KS 90), down 4% at 80 (BK 100,
    KS 25) with its 8 frames of hitlag folded in: release 26, FAF 52 [D].
  * **Ledge attack** 20-22 (9%, shield 111, intangible through 22, no clank, FAF 56); **get-up attack** face up: 14-15 in front, 20-21
    behind (7%, 48, shield 214, intangible through 21, no clank, FAF 46).
  * No-clank boxes exactly the raw tables' `clang=f` rows (pummel, ledge, get-up). No normal has `rebound=f` alone (E3g unused) or a
    shieldstun multiplier (E24 unused): checked in the raw tables.
* **Specials**: the claws' placeholders (`ext0`-`ext2`); `ext3` is the down air's landing hit.
* **Registered**: `CLASSES` 9 and `Recipe::params` 8 → `bounder` (`sim-content/src/recipe.rs`), `Content::placeholder` (fighter
  `bounder`, weapon `clutch`), `godot/scripts/roster.gd` (`CLASS_NAMES` "Clutch", the audit's blurb, built-in Bounder), `content/base.pfc`
  re-exported (sim 89). Count tests updated: sim-content `doc.rs`, `recipe.rs`, `tests/bundle.rs`, `tests/recipes.rs` (`MADE` 9); netplay
  `tests/fighters.rs` (class 9 unknown); godot `content_e2e.gd`, `creator_flow_test.gd`, `editor_api_test.gd`, `online_fighters_test.gd`.

### Verified (tests that ran)

`sim-core/tests/clutch.rs` (25 tests): every normal's windows, FAF, landing lag and autocancel in the data; first-hit frames (14 moves)
and FAF (every move but the throws, ledge attack and landing hit) by running each move; damage / angle / BK / KS of 41 boxes; hitlag,
SDI and set weight; aims; trips; shield damage and shieldstun; the no-clank set; the quickest jab loop tick for tick (3, 8-9, control
back 18 / 25); **held after a hit from jab 1's frame 6 and held without a hit from 16**, and no jab 2 when let go; the down air's
thirteen hits on 16, 18, ..., 38, 41 for 28% and its landing hit on the landing frame; aerial landing lag and autocancel (and the down
air's landing hit only outside its windows); the head and foot intangibility; the tongue grab on 14 (also at 2.8 units); each throw's
release frame and damage, the back throw behind and turned; the pummel; **jump air times 38 / 60 / 26 / 42 within a frame**; **the
double jump rises 51.56 reference units and peaks on frame 35-37** (the plain-jump reading of `REFERENCE_DATA.md`); the body's
attributes; **the armor**: a 100-unit hit on frames 1 and 44 only damages and freezes, 120 is withstood, frame 45 and a full hop launch
with 100; breaking hits launch at 80 / 60 / 60 / 180 for 200 / 150 / 121 / 300; two 100-unit hits both withstood (decision 47); a command
grab catches a double-jumping bounder; the frame count, and an aerial, an air dodge and a landing ending it; the turnaround (and none
for another body, none with the stick forward); class 8 registration; random play deterministic and rolling back.
`multihit::knockback_armor_withstands_up_to_its_threshold_and_a_subtractive_one_takes_it_off`; `content::tests::hash_changes_with_any_field`
(the new body fields and every armor field); `state::checksum_sees_every_field` (`air_jump_frame`);
`bundle::an_air_jump_armor_block_round_trips_is_inherited_and_is_checked`; `angle_types.rs` (17 Clutch moves, 125 checked);
`ground_attributes.rs` (the bounder); `moves::every_weapon_has_every_move_slot`.

Checks: `cargo fmt --all`; `cargo test --workspace` (1066 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings`
(clean); `pftool fuzz-rollback 500` (0 mismatches; 243 bounders among the fighters); `pftool net-fuzz 200` (0 mismatches; 51
bounders); `pftool content-check content/base.pfc` (ok); the base.pfc equality test. **Not run**: the Godot tests (updated for the
ninth class; they need the editor and a built bridge).

### Estimated or limited [E]

* **Positions on non-root bones** [E]: the tail (tilts, up air), head (forward air, pummel), leg and foot (dash attack, up smash) and
  the tongue's reach (8.0 to 22.0 reference units ahead, 24.0 from a dash; gap 3). Root-bone offsets are [D].
* **Jab 1's window past its FAF** (6-30 against FAF 18): after frame 18 a press starts a new jab 1, not jab 2 (one move, one length;
  the same limit as the Bracers' jab).
* House limits: the grabs' aerial-target boxes, the throws' interrupted rows and thrower invincibility, the down throw's victim
  invincibility during the folded hitlag, the forward air's "ledges from 68", the face-down and trip get-up attacks, the jab's
  objects-only box, crawling, the double jump's x1.2 horizontal multiplier (the shared stick drift), the 7.0.0 shield intangibility
  (decision 5).
* The down tilt's trips can only happen when the hit does not lift (the tip's 361 at low knockback); the reference says the same.
* Conflict 4 resolved by decision 47 (each hit alone); conflict 3 read as 60 units of knockback [R reading].

### Training-mode check (hitbox view)

Bounder vs a standing duelist: (1) double jump and have a duelist forward-tilt it at 0%: it keeps rising with a flash of hitlag; at
150% the jab sends it with the reduced launch; (2) hold attack on a whiff: jab 2 on frame 16; on a hit, on 6; (3) the down air over a
standing target: twelve flashes two frames apart from 16, the finisher on 41; (4) the tongue grab's reach at 2.5-2.8 units.

## For the art agent

Moves that need their own clip (each falls back to a shared archetype through `anim_library.gd` until then):

| Move | First active / span | Pose and hitbox placement implied |
| --- | --- | --- |
| Double jump | jump frames 1-44 armored | a flutter-kick double jump; turned round when the stick is held back (the view should read `facing` after the jump) |
| Jab 1 / 2 | 3 / 3-4 | two quick kicks at 0.9 units up, 0.7-1.7 ahead |
| Forward tilt | 5-7 | a tail swipe straight / 30 up / 20 down, tip 1.7 ahead |
| Up tilt | 8-15 | the tail swept from high in front over the head to behind (1.9-2.4 up) |
| Down tilt | 8-10 | the tail low along the floor, to 1.7 ahead |
| Dash attack | 10-20 | a flying side kick, foot 1.4 ahead at 0.75 up |
| Forward smash | 14-16 (charge 7-8) | a headbutt: head 1.2 ahead, nose 1.8 ahead at 0.8 up (aimed up 1.4 / down 0.5 up); the head is intangible 12-16 |
| Up smash | 11-16 (charge 7-8) | a flipping bicycle kick overhead, foot 2.4 up slightly behind; the foot intangible 11-13 |
| Down smash | 7-8, 22-23 | tail sweep in front, then behind |
| Neutral air | 3-25 | a held kick, 0.4-0.6 up from 0.9 ahead to behind |
| Forward air | 16-20 | an overhead headbutt swung down in front, the snout tip low ahead (meteor) |
| Back air | 11, 14, 18-19 | three tail/foot kicks behind at 1.25 units |
| Up air | 5-6 | a tail flip overhead, front to back |
| Down air | 16-41 + landing | the flutter kick: twelve stomps under the body, a final kick on 41, a landing stomp |
| Grabs | 14-21 / 16-23 / 17-24 | the tongue shot out to about 2.75 units at 0.75 up |
| Pummel, throws | 2; 15 / 20 / 13 / 26 | a chomp; forward and back tosses (back turns on 20), an upward toss, a ground slam |

Animation also needs (phases 2-3, later): the egg, the egg throw, the drop with its stars, the rolling egg.

## Phase 2 plan: Egg Throw and Yoshi Bomb

1. **Egg Throw** (up special, one slot; script reads `grounded`): count the frames special is held on 1-15, on 16 read the stick into
   18-90 degrees (default 58), `spawn` at (5, 13)/8 with speed `0.1875 + 0.01025 f` and the vertical part x1.25; in the air the hop
   `min(0.3125 x 0.8^n + 0.8 vy, 0.325)` for `n` 0-2 (three `air_flag`s, reset on landing), the move's gravity 0.00875 / fall 0.16125,
   `start_speed_y` 80, never helpless. The egg: 6% at 70 (type 3), H 0.6, shield 50, reflectable; `arc { gravity 0.01125, max_fall 1.25,
   bounces 2 }`, life 46. Engine: **E3e** (built by Cloud's phase 3 on main) with three options: burst `on_ground` (after its last bounce
   or a too-fast landing), `on_expire`, and `keep_hits` (the burst skips the egg's own victim); **E3b additions** `bounce_keep_y` and
   `bounce_max_fall` (conflict 10, [E]); **E11 addition** `ledge_falling_only`.
2. **Yoshi Bomb** (down special, its `air_version`, a falling loop slot and the landing): the hop's 4% grounded-only FKV 80 box on 7,
   the drop from 27 at 0.6 a frame (15%, shield 167), the aerial stall and the 12% clean / late boxes (shield 183), ledges both sides
   from the fall's start, the 39-frame landing (FAF 40) with two 4% stars on its frame 3 (H 0, shield 250, reflectable, gravity 0.025,
   fall 0.25). Engine: **E30** `Move::drop_through { from, frames, while_down }` (decision 48) and **E3h** a second spawn request so
   both stars leave on the same frame (decision 48). Gaps 14-15 (the rise and stall) fitted [E].
3. Slots: up special; down special, `ext4`-`ext6` (aerial start, fall, landing). Tests in `clutch_specials.rs`, `move_rules.rs`
   (E30, E11), `projectiles` / burst tests for the E3b and E3e options; one SIM_VERSION bump.

## Phase 3 plan: Egg Lay, then Egg Roll

1. **Egg Lay** with **E8** as approved (decision 46): the tongue (19: mouth box + aerial- and grounded-only tip boxes, command grab;
   20-22 the tip boxes; whiff FAF 42) and its aerial version; the catch moves (swallow and lay, `hold_at` in the mouth, laying throw on
   its 21, intangible 7-29, FAF 63 overall [E: conflict 5], the aerial lay's vertical speed 0.125). E8: `EFFECT_ENCASE`, `Move::encase`,
   `FighterState::Encased` with hashed `encase_timer` / `encase_age` / `encase_by` / `encase_move`; the timer `100 + 1.8 min(p, 100)` less
   2 a percent of the throw; mashing 7 / 12.6 (sharing the bury's mash helper once the Bracers' E5 is in); 0.66x damage, no knockback,
   2 frames off a percent taken (conflict 2 [E]); the shift and hop on the ground; the break-out at 0.3125 up with 7 intangible frames;
   no grabs on an egg; the egg's hurtbox growing from 0.45x in 20 frames. Its unpublished parts (launch direction, fall, break-out) [E]
   until footage. The view needs a new state clip (art agent).
2. **Egg Roll last and simple** (decision 49): a script from existing features only (the hop, a stick-steered roll speed, one jump, a
   special cancel after 25 roll frames, the ending slot), the hit by `set_charge` / E29 additive damage once the Bracers' phase 2 builds
   it (10% + speed), rehit 32. **E31 (no box in the air or while turning) and E32 (the ledge stall) stay deferred listed limits.**
3. Then the iconic-move re-check against footage for the gaps (`REFERENCE_DATA.md` gaps 1-16).

## Questions for the owner

1. The tongue grab's reach is [E] (22 reference units standing): footage at its furthest reach would make it [R].
2. Jab 1's continuable window runs past its FAF (6-30 vs 18): build a general "continue window past FAF" (a jab that ends but still
   takes a press as its next hit for a few frames), shared with the Bracers' jab, or keep the listed limit?
3. The double jump's x1.2 horizontal multiplier (`jump_aerial_speed_x_mul`): every body's air jump drifts by the stick times its air
   speed; add a per-body multiplier (small, general) or keep the shared rule?
4. A bury hit should ignore a subtractive knockback armor (the reference's rule): build it with the Bracers' E5 (noted in
   `SHARED_MECHANICS.md`).
