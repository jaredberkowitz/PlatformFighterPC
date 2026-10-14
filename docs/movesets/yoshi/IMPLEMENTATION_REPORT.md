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
* ~~Jab 1's window past its FAF~~: built in phase 2 (decision 58).
* House limits: the grabs' aerial-target boxes, the throws' interrupted rows and thrower invincibility, the down throw's victim
  invincibility during the folded hitlag, the forward air's "ledges from 68", the face-down and trip get-up attacks, the jab's
  objects-only box, crawling (the double jump's x1.2 drift: built in phase 2, decision 59), the 7.0.0 shield intangibility
  (decision 5).
* The down tilt's trips can only happen when the hit does not lift (the tip's 361 at low knockback); the reference says the same.
* Conflict 4 resolved by decision 47 (each hit alone); conflict 3 read as 60 units of knockback [R reading].

### Training-mode check (hitbox view)

Bounder vs a standing duelist: (1) double jump and have a duelist forward-tilt it at 0%: it keeps rising with a flash of hitlag; at
150% the jab sends it with the reduced launch; (2) hold attack on a whiff: jab 2 on frame 16; on a hit, on 6; (3) the down air over a
standing target: twelve flashes two frames apart from 16, the finisher on 41; (4) the tongue grab's reach at 2.5-2.8 units.

## Phase 2 (sim 95, content schema 6 unchanged): the lobbed shell, the drop, decisions 58-59

Branch `clutch-phase2`, from `movesets` at 457731b (sim 89). Commits: 2a9b6ef (engine: decisions 58-59 and the general pieces), c0e7698
(the two specials and their tests), 473f0a3 (merge of `movesets` at sim 93), the docs commit, then a second merge of `movesets` at sim 94 (the Bracers' phase 2): this branch takes **95**. Owner
decision 60 applies: where the reference publishes nothing, the value is our best reading, labelled [E], with no footage asked for.

### Engine (general; every default is the old behaviour)

* **Decision 58, a jab's window past its FAF**: `Move::late_next: Option<(move, last move frame)>` (content `late_next` / `late_until`).
  When the jab ends on the ground, hashed `Fighter::late_next` / `late_left` keep the window open while the fighter stands idle (any
  other state closes it); a jab press in it (`fighter::start_ground_attack`) starts the next hit instead of a new jab. **Other fighters
  changed** (every jab whose reference window runs past its FAF, from each port's `REFERENCE_DATA.md`): the Embers' jab 1 (FAF 20,
  window to 24) and jab 2 (22, to 24); the Gauntlets' jab 1 (18, to 30) and jab 2 into jab 3 (19, to 30); the Greatsword's jab 1 (30, to
  30: one frame); the Bracers' jab 1 (25, to 28); the Clutch's jab 1 (18, to 30). Not changed: the Staff (its rapid-jab window ends
  before its FAF), the Longsword, Claws and Needles (no published window past the FAF).
* **Decision 59, a body's air-jump drift**: `FighterParams::air_jump_drift` (percent, default 100; content `air_jump_drift`, inherited).
  The bounder's 120 (`jump_aerial_speed_x_mul` 1.2 [R]); no other body changes.
* **E11 addition** `Move::ledge_falling_only`: the move's ledge windows catch only while the vertical speed is not positive.
* **E30** `Move::drop_through { from, frames, while_down }` (decision 48): `platform_ignore` set on move frame `from` for `frames`, and
  kept on every later frame the stick is held down with `while_down`.
* **E3h** (decision 48): a script's second `spawn` / `spawn_at` in one frame fills a second request (hashed `Fighter::spawn2_request`,
  `spawn2_fixed`, `spawn2_pos`, `spawn2_vel`); `combat::spawn_projectiles` serves both (the second with plain size and knockback).
* **E3b additions**: `ProjectileArc::bounce_keep_y` (percent of the bounce's vertical speed, default 100) and `bounce_max_fall` (the
  fastest landing that still bounces, default 0: none).
* **E3e options**: `Burst::on_ground` (a landing it does not bounce from bursts it there), `on_expire` (the end of its life bursts it),
  `keep_hits` (the burst keeps the projectile's hits: the fighter the shell hit is not hit again).

### Built

* **Up special, the lobbed shell** (`moves_clutch.rs::lobbed_shell`, `scripts/clutch_egg_throw.script`): the shell on frame 16, FAF 56
  [R]; speed `0.1875 + 0.01025 f` world (`1.5 + 0.082 f` reference) with `f` the frames special is held, at most 15 [R] (counted on the
  move's frames 2-16 [E: the reference does not say which 15 frames]); the stick on frame 16 aims it 18-90 degrees, 58 neutral [R], in
  proportion to the stick's sideways tilt past 0.2 (full forward 18, full back 90) [E: how the reference's "sensitivity 0.5" maps];
  vertical part x1.25 [R]; spawned 5 ahead, 13 up [R]. In the air the vertical speed becomes `min(2.5 x 0.8^n + 0.8 y, 2.6)` reference
  units for the first three throws an air time [R], on the move's frame 2 [E: gap 13]; the move's gravity 0.07, fall 1.29, air speed
  x0.9, vertical x0.8 on starting [R]; never helpless [R]; ledges in front only while not rising [R; front only E]; touching down carries
  it on along the ground [E: no landing lag is published]. The hops come back on landing, a ledge or a hit (the shared once-per-airtime
  flags) [E: gap 13]. The shell: 6%, 70 (type 3), BK 60, KS 50, H 0.6, shield 50 [D], cannot rebound, reflectable, radius 3 [D];
  gravity 0.01125, fall 1.25 [D]; bounces at most twice, only off a landing at most 0.2 a frame (the reference's 1.6), keeping 0.8 of its
  speed both ways [E: conflict 10's reading]; it bursts on a fighter (shielding or not), a wall, a landing it does not bounce from, or
  at the end of its 46 frames [R]; the burst: the same hit at radius 8 [D] for 3 frames, never on the shell's own victim [R reading].
* **Down special, the drop** (`drop`, `scripts/clutch_drop*.script`): grounded, the hop's 4% hit on frame 7 (grounded fighters only,
  FKV 80, KS 100, H 0.5) [R], the rise on 9-22 to 7.5 units and a still top on 23-26 [E], the drop from 27 (15%, BK 60, KS 72, H 0.7,
  shield 167 [D]) at 0.6 a frame [R]; from level ground it lands on 39 and acts on 78 (the reference's 77 frames in all [R]: the rise
  height was fitted to it). Aerial (`DROP_AIR`, ext4): the rise on 2-15 to 5.36 units (the grounded one / 1.4, the reference's grounded
  height multiplier [R]) and a still top on 16-18 [E], the clean 12% on 19 and late from 20 (BK 70, KS 68, H 0.7, shield 183 [D]) [R].
  Both: sideways speed stopped [E]; through soft platforms for 7 frames from the drop's start and all of them while down is held [R]
  (E30); ledges in front and behind from the drop's start [R]; never helpless; the landing (`DROP_LANDING`, ext5): 39 frames, FAF 40
  [R], with the two stars leaving on its frame 2 and hitting on 3-12 [R] (4%, 50 (type 3), BK 50, KS 40, H 0, shield 250 [D], cannot
  rebound, reflectable; speed 0.075 / 0.25, gravity 0.025, fall 0.25 [D]). The drop lasts at most 253 frames (137 units of falling,
  past any blast zone).
* The claws' up special endings (`ext0`-`ext2`) are freed. Slots now: `ext3` down air landing, `ext4` aerial drop, `ext5` the drop's
  landing; `ext0`-`ext2` and `ext6`-`ext15` free for phase 3. Neutral and side specials: still the claws' (phase 3).

### Verified (tests that ran)

`sim-core/tests/clutch_specials.rs` (14): the shell on frame 16 at its spawn point, FAF 56, the move's air physics; its speed for 0 /
7 / 15 / 40 frames held; the aim at 18 / 58 / 90 degrees; the three hops from a full-speed fall (1.468 / 0.968 / 0.568 reference units)
and none on the fourth, never helpless, the hops back on landing; no hop on the ground; one hit on its victim and the burst on the
fighter beside it (and two hits without `keep_hits`); a slow landing bouncing x0.8 both ways, a fast one and a third one bursting, the
burst at the end of its life for 3 frames; the shell's and burst's hit data; the grounded drop's timeline (on the ground to 8, the
drop's 0.6 from 27, the landing on 39, control on 78); the aerial stall and drop from 19, the x1.4 ratio; every drop box; the two stars
on one frame with their speeds and hit; the soft platform held and passed; a ledge behind caught only once the drop has started.
`late_jab.rs` (3): every late window's data, a press on the FAF and on the window's last frame starting the next hit, one frame later
a new jab 1, a crouch closing the window. `clutch.rs` (26): the double jump's x1.2 drift (and x1.0 for the duelist); registration.
`move_rules.rs`: `ledge_falling_only_catches_a_ledge_only_once_the_fighter_stops_rising`,
`drop_through_passes_soft_platforms_for_its_frames_and_while_down_is_held`, `the_shell_and_drop_fields_are_in_the_content_hash`.
`scripted_moves.rs`: `two_spawns_on_one_frame_fire_two_shots_and_a_third_replaces_the_first`. `state::checksum_sees_every_field` (six
new cases), `content::tests::hash_changes_with_any_field` (`air_jump_drift`).

Checks: `cargo fmt --all`; `cargo test --workspace` (all passed); `cargo clippy --workspace --all-targets -- -D warnings` (clean);
`pftool fuzz-rollback 500` (0 mismatches; 243 bounders); `pftool net-fuzz 200` (0 mismatches; 51 bounders); `content/base.pfc`
re-exported (sim 95) and equal to the builders. **Not run**: the Godot tests.

### Estimated or limited [E] (decision 60: estimates, no footage needed)

* The shell's aim curve and its 0.2 deadzone; which 15 frames count as "held"; the hop's frame; the bounce reading (conflict 10:
  bounce at most 1.6, burst above); the shell's 1-unit capsule height is not modelled; a ledge in front only.
* The drop's rise (7.5 / 5.36 units, a linear slowing over 14 frames) and still top, fitted to the published 77 frames; sideways speed
  stopped; the "falls indefinitely" drop is 253 frames at most.
* A stick flicked down on a soft platform drops through it before a special can start (the shared input order): a quick down + special
  there gives the platform drop and an aerial neutral special, not the drop. The reference starts the special. (Shared input rule, not
  changed here.)
* Still the house limits listed for phase 1.

## Phase 3 (sim 107, content schema 6 unchanged): the tongue and its egg (E8), the rolling shell

Branch `clutch-phase3`, from `movesets` at af0949e (sim 102), merged with `movesets` at 2030552 (sim 103) and at 885a44e (sim 106):
this phase is **107** (104 was taken first; renumbered at the coordinator's request). Commits: 1fae479 (engine, moves, tests), de67063
(the roll's hit test, the bridge accessor), b84faa6 and 101cc30 (the merges), then the docs commit. The ssbwiki.com raw neutral
special page was re-read on 2026-10-14: it settles conflict 2 (below) and gives nothing new on the egg's physics.

### Engine (general; every default is today's behaviour)

* **E8, the egg** (owner decision 46, as designed in `PROJECT_AUDIT.md`, separate from the bury and sharing only the mash reading).
  * Content: a hit effect **`EFFECT_ENCASE`** (3; `effect encase`), only on a throw row, and **`Move::encase: Option<Encase>`** (an
    `encase { ... }` block, every field required: `base per_percent percent_cap mash_stick mash_button hit_cut damage_taken launch_x
    launch_y grow_from grow_frames size shift_x shift_y air_shift_x air_shift_y exit_vy exit_intangible`). Validation: the effect only on
    a throw hitbox, the block and an encase row go together, `base` above 0, no negative rates, `damage_taken` 0-200, `grow_from` 1-100,
    `size` 1-200. `Weapon::hash_into` hashes every field. `docs/CONTENT.md` lines.
  * State: **`FighterState::Encased`** and hashed **`Fighter::encase_timer`** (fixed point), **`encase_age`**, **`encase_char`** (the
    body that laid it: an index into `Content::fighters`) and **`encase_move`**: the egg's numbers are read from that body's move
    (`combat::encase_of`), so nothing else is copied into the victim. (The audit proposed the layer's slot, `encase_by`; the body index
    reads the same numbers from content alone, so hurtboxes and the bridge need no other fighter.) `scripting::World::encase` carries
    each fighter's numbers into `fighter::update` (no new argument).
  * Laying (`combat::apply_hit`, a throw row with the effect): the throw's damage is added as usual (staleness, the 1.2), then
    `fighter::encase`: the timer `base + per_percent x min(p, cap) - hit_cut x added` (p the percent before the throw), velocity
    `launch` by the layer's facing, no hitstun.
  * In the egg (`fighter::encased`): outside hitlag the timer drops 1, plus `mash_button` on a frame with a new press of any button and
    `mash_stick` on one with a stick flick (**`grab::mash_inputs`**, the shared reading, which `grab::mash_cut` now also uses); a flick
    adds `shift.0` sideways and, with the stick up, hops at `shift.1` on the ground (`air_shift` in the air, added); the fighter's own
    gravity, fall speed, traction and air friction; it lands and stays an egg, walks off edges, catches no ledge and techs nothing; at 0
    it breaks out (`Airborne`, `exit_vy`, `exit_intangible` frames).
  * Hit while in it: the damage times `damage_taken` percent (then the 1.2), the hitlag as usual, the timer cut by `hit_cut` x the
    percent actually added, no knockback, no hitstun, no state change (a bury or trip never applies); a push still pushes. Grabs and
    command grabs refuse it (`grab::grabbable`, `grab::catchable`).
  * Hurtbox (`combat::body_hurtboxes`): all four circles are one egg standing on the feet, `size` percent of the body's height across,
    growing from `grow_from` percent over `grow_frames` frames (`Encase::radius`). Bridge: read-only `fighter_egg_radius(i)`.
  * `step.rs`'s random-play invariant accepts `Encased` on the ground or in the air.

### Built

* **Neutral special, the tongue** (`moves_clutch.rs::tongue_and_lay`): one move on the ground and in the air (the reference's versions
  share their boxes and FAF; touching down carries it on [E]). A command grab: the mouth box on 19 (r 5.9 → 7, 7.0 up, 9.0 ahead [D]),
  the tip's aerial-only (r 4.0 → 5) and grounded-only (r 3.5 → 4) boxes on 19-22 swept 8.0 to 22.0 ahead at 6.0 up [E: the tip bone's
  reach, gap 3, as the grab]; whiff FAF 42 [R].
* **The catch, the swallow and the lay** (`LAY` = ext0): from the frame after the catch, a 6-frame swallow, then the reference's laying
  part (release on its 21, intangible on its 7-29, FAF 38 [R]): release on 27, intangible 13-35, FAF 44, so a catch on 19 lays on 46
  and acts on 63 (UFD's "62 when successful") [D: conflict 5]. The victim held at the feet inside the body [E]. The throw: 7%, 361,
  BK 0, KS 100, H 0 [R], effect encase. In the air the vertical speed becomes 0.125 as it lays (the reference's 1 [R],
  `clutch_lay.script`). No ledges [R].
* **The egg** (`Encase` on `LAY`), every number [R] from ssbwiki.com's parameters except where marked: 100 + 1.8 x min(p, 100) frames,
  2 off a percent actually taken (so **84 to 264 frames** unmashed: the page's own figures with the one-on-one multiplier), 7 a stick
  input, 12.6 a button input, 0.66x damage, laid at 0.7 / 2 (0.0875, 0.25 world) **behind** the layer [E: gap 5], 0.45x growing to full in
  20 frames, its full size one body height across [E: gap 6], shift 0.3 / hop 1 on the ground and 0.01 / 0.1 in the air, out at 2.5
  (0.3125) with 7 intangible frames; its gravity, fall and traction the victim's own [E: gap 7]; the midair jump kept on breaking out
  [E: gap 8]; no grabs on an egg [E: gap 9].
* **Side special, the rolling shell** (`rolling_shell`, `clutch_roll.script`, `clutch_roll_end.script`; ending `ROLL_END` = ext1),
  simple as decision 49 asks: the hop at 2 (0.25) on the first scripted frame, the roll's gravity 0.13 / fall 1.95 [R]; the roll from 21
  starting at 0.9 [R]; on the ground the stick forward accelerates 0.05 toward 1.4 + 2.44 x the stick (1.4 to 3.84 reference units: the
  base 1.4-3.2 and x1.2 by the stick [R]; the law between [E: gap 10]), neutral returns to 1.4, back brakes (0.12 a frame, friction +
  acceleration [R]) and turns round at 0 [E]; in the air the speed is kept [E]. The hit from 32 (the first touch of the ground) [R]:
  10% plus the speed in reference units via `charge_add` / `set_charge` (E29) [R], 80 (type 0), BK 70, KS 50, aerial-only r 2.3 → 3
  and grounded-only r 3.5 → 4 at 5.6 up, rehit every 32 [R]; a hit takes 1 off the speed and 40 frames off the roll [R]; one jump on
  the ground at 0.3 up [E: gap 11]; a wall turns it at 0.4x with 1.7 up [R]; special on the ground from 25 roll frames, or the end of
  180, goes into the ending: speed x0.7, vertical x0.5, friction 0.05, FAF 40 later [R] (85 at the quickest [R]); ledges in front only
  while falling [E: gap 12]; never helpless [R].
* Slots now: `ext0` the lay, `ext1` the roll's ending, `ext3` the down air landing, `ext4` aerial drop, `ext5` the drop's landing;
  `ext2` and `ext6`-`ext15` free.

### Verified (tests that ran)

`sim-core/tests/clutch_egg.rs` (20): the tongue's boxes, kinds and targets, whiff FAF 42; a catch on 19 on the ground, through a shield
and in the air; a catch on 19 laying on 46, acting on 63, the bounder intangible on 32-54 exactly; the throw's 7% (8.4 taken) and the
egg's launch (behind, 0.0875 / 0.25, no hitlag); the egg lasting **84 / 174 / 264 / 264** frames at 0 / 50 / 100 / 150%; mashing (a
press every other frame, a flick every other frame) against the formula; a jab on a landed egg: 0.66 x 1.2 of its damage, still an egg,
hitlag, the timer cut by twice the percent taken; no tongue or grab on an egg; the egg's growth (0.45, 0.725 at 10 frames, full at 20,
all four circles); the break-out at 0.3125 with 7 intangible frames; the ground shift and hop and the air nudge; an egg falling off the
stage knocked out; an aerial lay setting the bounder's speed to 0.125; an egg resimulated exactly from a snapshot; the formula's
numbers; the roll's hop, its touch-down about 32, its box data; its top speed 0.48 and 13.8% at 3.8; the quickest stop acting on 85
(a press on 44 too early); the roll ending itself on 201 (180 frames) and jumping once; a roll hit for 10 + speed x 1.2 and the speed
lost. `move_rules::the_egg_is_in_the_content_hash` (all 18 fields and the block); `state::checksum_sees_every_field` (the state and
the four fields); `sim-content/tests/bundle.rs::an_encase_block_round_trips_and_is_checked`; `clutch.rs` registration (every special
the Clutch's own).

Checks: `cargo fmt --all`; `cargo test --workspace` (all passed, after the sim 106 merge); `cargo clippy --workspace --all-targets -- -D
warnings` (clean); `pftool fuzz-rollback 300` (0 mismatches; 146 bounders); `pftool net-fuzz 100` (0 mismatches; 19 bounders);
`pftool content-check content/base.pfc` (ok); `content/base.pfc` re-exported (sim 107) and equal to the builders. **Not run**: the Godot
tests (no GDScript changed; one read-only bridge accessor added). Whether random play actually lays eggs is not counted by the fuzzers.

### Estimated or limited [E] (decision 60)

* The egg: laid behind (gap 5), one body height across (gap 6), the victim's own physics (gap 7), the midair jump kept and free to act
  on breaking out (gap 8), not grabbable or re-eggable (gap 9); a stick flick counts as one stick input whatever its direction, and the
  hop needs the stick up past the threshold; mashing counts at most one button and one flick a frame (the shared reading).
* The swallow's 6 frames (conflict 5) and whether the bounder is armored there (gap 4: not modelled; the interrupted 4% row neither);
  the held victim pinned at the feet; touching down during the aerial tongue or the swallow carries on.
* The roll (decision 49): the steering law, the turn, the jump's speed, the air behaviour, the ledges; **E31** (no hit in the air or
  while turning) and **E32** (the ledge stall) stay deferred, so its hit is out in the air and while turning and it rolls off ledges;
  it does not pass through shields (bodies only), has no floor bounce after the jump, and its special needs a press on the frame (no
  buffer).

### Training-mode check (hitbox view)

Bounder vs a standing duelist at 2.5 units: (1) neutral special at 0%: the egg is laid on 46 behind, breaks 84 frames later; at 100%,
264; (2) mash attack in the egg: out about twice as soon; (3) jab the egg: about 2.4% and no launch; (4) side special and hold forward:
the box appears as it lands (32), the speed tops out near 0.48 a frame; special after frame 46 stops it, actionable on 85.

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

| Up special (the lobbed shell) | shell leaves on 16, FAF 56 | a wind-up and an overhand lob from 0.6 ahead, 1.6 up; aimed 18-90 degrees by the stick; in the air a small hop on frame 2 |
| The shell (projectile) | 46 frames, then a 3-frame burst | a spotted shell (radius 0.3) on an arc that bounces low; the burst a 0.8-radius pop |
| Down special, grounded | hop hit 7, rise 9-22, top 23-26, drop from 27 | a crouch and hop (hit at the feet 1.25 ahead), a rise to 7.5 units, a curled still top, then a straight drop bottom-first (box 0.7 up) |
| Down special, aerial (`ext4`) | rise 2-15, top 16-18, drop from 19 | the same at 5.4 units |
| The drop's landing (`ext5`) | 39 frames; stars out on 2 | a heavy squat landing; two small stars thrown 1 unit to each side, arcing up |

Animation also needs (phase 3, built at sim 107):

| Move | First active / span | Pose and hitbox placement implied |
| --- | --- | --- |
| Neutral special, the tongue | 19-22, FAF 42 | the tongue shot out 1 to 2.75 units ahead at 0.75 up (a mouth box 1.1 ahead, 0.9 up on 19) |
| The lay (`ext0`) | swallow 1-6, lay from 7, release 27, FAF 44 | the victim pulled in and swallowed (hide the held fighter while its holder is in `ext0`: `fighter_grab_partner`), then a squat and the egg popped out behind |
| The egg (victim state `Encased`) | until it breaks | a spotted egg round the fighter, standing on its feet, radius `fighter_egg_radius(i)` (grows from 0.45x over 20 frames); wobbles when mashed; breaks into shell pieces as the fighter jumps out |
| Side special, the rolling shell | hop 1-20, roll 21-200, hit from 32 | the bounder curled into a shell, rolling (spin with its speed), turning round when it stops |
| The roll's ending (`ext1`) | 40 frames | the shell breaking open, the bounder hopping out |

## Phase 2 plan: Egg Throw and Yoshi Bomb (built: see "Phase 2" above; no separate falling slot was needed)

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

## Phase 3 plan: Egg Lay, then Egg Roll (built: see "Phase 3" above)

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

1. The tongue grab's reach is [E] (22 reference units standing): kept as the estimate (decision 60).
2. ~~Jab 1's window past its FAF~~: answered (decision 58), built in phase 2.
3. ~~The double jump's x1.2 drift~~: answered (decision 59), built in phase 2.
4. A bury hit should ignore a subtractive knockback armor (the reference's rule): build it with the Bracers' E5 (noted in
   `SHARED_MECHANICS.md`).
