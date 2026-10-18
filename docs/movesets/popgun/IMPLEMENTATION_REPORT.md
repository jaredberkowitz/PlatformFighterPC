# Implementation report: the twelfth moveset (reference: the reference game's Diddy Kong)

Class 11 **Popgun** on the **scamp** body, built-in fighter **Scamp** (owner decision 65). Reference data: `REFERENCE_DATA.md`; engine
audit: `PROJECT_AUDIT.md`. Everything is 13.0.1 (decision 68 applies; the older jab rule is not used).

## Phase 1 (sim 117, content schema 7 unchanged; branch `popgun-phase1` from `movesets` at faff36e, sim 115)

Sim 116 belongs to another session; 117 was taken as asked. The same branch also carries the Settler's axe change (owner decisions 72-73,
its own commit: `docs/movesets/pockets/IMPLEMENTATION_REPORT.md`, "Sim 117").

### Milestone 1: the body and every normal (`sim-core/src/moves_popgun.rs`, `FighterParams::scamp`)

* `FighterParams::scamp` (13.0.1): every attribute in `REFERENCE_DATA.md`: weight 90, the parameter file's ground and air numbers, the
  jumps (full 41.21, short 19.9, double 41.21 with its x0.9 drift: `air_jump_drift` 90), heavy landing 6, the dodges, a wall jump, the item
  release frames 7 / 9 / 11 / 6 / 4 / 5.
* Every normal on the 13.0.1 raw tables: the three-hit jab (continuable 6-30 and 7-30, past their FAFs by decision 58; jab 1 held repeats
  from frame 10), the forward tilt aimed up / straight / down (hands per aim [E], root boxes in all), the up tilt (SDI x1.5), the down
  tilt (20% trip), the three-hit dash attack (set weight and set knockback per target, no clank), the set-knockback forward and up
  smashes (grounded / aerial boxes), the down smash front then back, the five aerials with their landing lags and autocancels, the grabs,
  the pummel, the four throws (the up throw's 1% pre-hit), the ledge and get-up attacks. Bare hands and feet (`Weapon::limbs`).
* Registered: `CLASSES` 12, `Content::placeholder`, `roster.gd` (name, blurb, the built-in Scamp with the shared default look), the Godot
  tests' rosters (`content_e2e`, `editor_api_test`, `online_fighters_test`), index assumptions in the Rust tests moved by one (a made
  fighter is now index 12; class 12 is the unknown one), the fuzzers (they draw every class: class 11 drawn 94 and 10 times).

### Milestone 2: the banana peel (down special) with E44, E45, E47

* **E44 slips**: `Hitbox::slip` (`SLIP_HARD`, `SLIP_SOFT`), the ruleset's `slip_lag` 30 / `slip_max` 90, hashed `Fighter::trip_soft`.
* **E45 traps**: `ItemSpec::trap` (`Trap { hit kick_x kick_y }`), `ItemSpec::uses` (hashed `Item::used`), `ItemSpec::damage_add`;
  `items::traps`.
* **E47 a move's toss**: `Move::toss` (`Toss { frame again angle speed smash_speed wait }`), `items::THROW_MOVE`, hashed
  `Fighter::pull_wait`, the `pull_wait` register (whitelist, `scripted_moves` test, `CONTENT.md`).
* **The peel**: in hand on 4, tossed at 105 degrees on 20 (2.2, 3.036 smash, x toss strength 0.966667), FAF 40; with an item already held
  it is tossed first on 14 and the peel made then (`PEEL_WITH_ITEM`); one out and 85 frames after the last toss, else no peel
  (`PEEL_FAILS`, `PEEL_FAILS_WITH_ITEM`, FAF 50 [E]); the item (`banana_item`): 0% base + 1 + 1.7 x speed, x0.85 throws, a hard-slip hit,
  never clanking, a soft-slip trap at its foot, two uses, bouncing high off a fighter [E], sliding to a stop on landing [E].

### Milestone 3: the peanut popgun (neutral special)

* A scripted draw (`popgun_draw.script`): the uncharged peanut on 16 (FAF 49); held, the charge to 94 (78 frames) fired on the release
  (FAF the release + 33 [D]); held on 96, the misfire (`POPGUN_MISFIRE`, the explosion on 97: 23%, FAF 203 [D]); the shield cancels from 14.
* The peanut (`POPGUN_FIRE`'s E35 charged shot): 4.3% to 13.8%, 70 (3), BK 40, KS 70, hitlag x0.2, -1 shield (77% [D]), never clanking; its
  speed and angle by the charge in the script (1.6 at 35 degrees to 3.6 at 8 [E]); gone on the floor.

### Milestone 4: the rocket barrels (up special) with E48

* **E48 shed**: `Move::shed` (a hit landing on the fighter during the move fires that move's projectile at it; a flinch ends the move into
  the special fall); the projectile registers `owner_stick_x` / `owner_stick_y` (whitelist, `scripted_moves` test, `CONTENT.md`).
* **The charge** (`popgun_rocket.script`): from 7 while special is held, at most 60 frames, launching on 8 to 68 [R]; the stick tilts the
  launch (0.75 a frame, at most 45 [E]); in the air a slower fall [E: gravity 0.02, fall 0.5].
* **The flight** (`ROCKET_FLIGHT`, `popgun_rocket_fly.script`): 3.1 + 0.019 a charge frame, x1.05 from the air [R]; steered 1.5 a frame, at
  most 80 [E]; the speed falling off evenly to nothing over 54 frames [E]; the hits 10% / 8% / 6% on its frames 1-2 / 3-9 / 10-28 [R];
  the special fall with 30 frames of landing lag [R]; ledges from the launch [E].
* **The crash** (`ROCKET_CRASH`): a wall, or the ground after the flight's first 3 frames: 18%, radius 20.0, frames 1-2, FAF 58 [R].
* **The barrels** (`BARRELS`, shed by a hit during the charge or the flight): 1% to each fighter once, wobbling up and away, steered 40
  frames by the scamp's stick [E], bursting on the stage, a wall or after 180 frames [E] into the 18% explosion [R].

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/popgun.rs` (15): every normal's active windows, FAF, landing lag and autocancels in the data and by running them (first hit
frames, FAF), damage, angles, BK / KS, hitlag and SDI multipliers, set knockback and weight, targets, trips, angle types, shield damage,
no-clank boxes; the forward tilt's aims; the jab chain (6 / 7, held repeat every 9 frames) and jab 1's pull at the tip; grab on 7; the
throws' release frames and damage (the up throw 17 / 19); pummel 1%; jump air times within a frame (36 / 51 / 26 / 36); the body;
registration (three own specials, the side special still the claws'); random play deterministic and rolling back.
`popgun_peel.rs` (15): in hand on 4, tossed on 20 at the reference's velocity (105 degrees, 2.2 x 0.966667, x0.85 / x0.85²), free on 39
(FAF 40); the smash toss 3.036 / 2.2 faster; one out (the no-peel version, FAF 50) and the 85-frame wait (counted from the toss); an item
held tossed on 14, the peel made on 14 and tossed on 20; a lying peel's soft trip (no hitlag, no damage, kicked away, get-ups from 30,
up on 90); its last thrower walks over it, an opponent's peel trips it, a shield no help; two trips use it up; a thrown peel's hard trip
with hitlag and `1 + 1.7 x speed` damage (the smash forward throw's 3.64), gone after a throw and a hit; the toss's own hit (the smash
toss's speed) bouncing off high; an airborne fighter launched, not tripped; picked up and dropped by the duelist it trips the scamp and is
spent; the peel's numbers; resimulation. `popgun_peanut.rs` (6): the tap on 16 (4.3%, FAF 49), a charge released on 50 (FAF 83, the
damage in proportion), the full charge on 94 (13.8%, FAF 127, faster and flatter), the misfire on 97 (23%, FAF 203, no peanut, no damage
to the scamp), the shield cancel from 14, the peanut's hit numbers, a hit that uses the peanut up. `popgun_rocket.rs` (9): the tap's
launch on 8 at 3.1 and the full charge's on 68 at 4.24, the air charge's slower fall and x1.05, the tilt while charging and the steering
while flying, the flight's three phases, the special fall's 30 frames of landing lag, a hit on a fighter above, the wall crash's 18%, the
barrels shed by a hit while charging and bursting on the stage, a flinch in the flight shedding them into the special fall,
resimulation. `trips.rs` (`a_slip_trips_whatever_the_launch_and_a_soft_one_has_no_hitlag`), `scripted_moves` (`pull_wait`,
`owner_stick_*`), `move_rules` (`the_banana_fields_are_in_the_content_hash`), sim-content (`the_popgun_banana_fields_round_trip_and_are_
checked`), the checksum cases (`trip_soft`, `pull_wait`, `item_used`), `late_jab` (the Popgun's two late windows), `ground_attributes`
(the scamp added).

### Estimated or limited [E]

* Positions on the hand, arm, shoulder, knee, leg and waist bones (every normal but the root boxes); the shared body size.
* The peel's physics (gravity 0.1, friction, its box, pick-up box, bounce off a fighter), its trap's place and kick, the no-peel FAF 50,
  the one-hand toss order.
* The peanut's speeds, angles, gravity and life; one shield-damage percent at every charge.
* The rocket's charge fall, tilt and steering rates, the speed's fall-off over the flight (only the launch speed is published), ledges,
  the ground-crash rule (after 3 frames: the flight never goes down here, so a ground crash comes only from steering into a slope or a
  platform edge), the momentum stopped on an aerial start; the barrels' whole flight.
* Gaps: the side special (phase 2); the popgun's muzzle hit and taller late box; catching a peanut; the barrels' unblockability; the crash's
  damage to the scamp; the first-charge-only slow fall (here every charge, since the engine's per-air-time flags reset on a hit); the
  trip and face-down get-up attacks, the grabs' aerial boxes, throw bystander hits and invincibility (house limits).

### Tests and checks run (after merging `movesets` faff36e, sim 115: already contained, nothing to merge)

`cargo fmt --all`; `cargo test --workspace` (all green: 1,413 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings`;
`pftool fuzz-rollback 300`: 0 mismatches, class 11 drawn 94 times, items 505 made / 361 picked up / 350 thrown; `pftool net-fuzz 100`: 0
mismatches, class 11 drawn 10 times. Godot tests were updated (rosters) but not run here (no GDScript logic changed: roster entries only).

### Training-mode checks (hitbox view)

1. The peel: down special with the stick already held down (the tilt toss), then flicked with special (the smash toss, further): a
   computer opponent walking into the lying peel sits down at once (no freeze) and can get up from frame 30.
2. The popgun: hold special to the hat-grab (frame 94) and let go; hold on through 97 (the explosion beside the scamp).
3. The rocket: charge 60 frames in the air beside the stage with the stick toward it; get hit while charging (the barrels fly off and
   burst on the stage).

### For the art agent

* **Clips needed** (they fall back to shared archetypes today): the jab's three hits (slap on 3, other hand on 5, side kick on 5); the
  forward tilt's two-handed slap aimed up / straight / down (10-16); the up tilt's overhead arm sweep (6-11); the down tilt's low slap (4-5);
  the cartwheeling dash attack (hits 8-9, 16-17, 22-23); the forward smash's two slaps (charge pose on 4, 12-13, 21-23); the up smash's
  flip (charge on 1, 5-8, 12-15, 19-23); the down smash slapping in front then behind (charge on 2, 7-8, 12-13); the spinning neutral air
  (8-20), flip-kick forward air (6-16), back kick (5-8), overhead flip-kick up air (4-7), stomping down air (15-16); the one-hand grab
  (7-8) and throws (forward 21, back 18, up over the head 17 / 19, down 22); the peel pull (4) and over-the-shoulder toss (20), the held
  item thrown on 14, the shrug when no peel comes (FAF 50); the popgun raised (1-15), the charge held (the reference grabs its hat at full
  charge, 94), the shot on release, the misfire (97) and its 105-frame stun; the rocket's charge (7-67, jets flaring), the flight (the body
  following its angle), the crash, the barrels flying off.
* **Props**: the peel is an item (`item_kind` 9; lying, held, thrown; it disappears when its two uses are spent); the peanut a projectile
  (move ext3); the misfire's explosion a hitbox on ext4; the flying barrels a projectile (move ext8, tool kind 10) and their explosion its
  burst (`projectile_bursting`). A tripped fighter sits (`fighter_tripped`); a soft trip is the same pose.

## Phase 2 (sim 119, content schema 7 unchanged; branch `popgun-phase2` from `movesets` at c0f4e88, sim 117)

Sim 118 belongs to another session; 119 was taken as asked. Owner decisions 74-76 (2026-10-14): build the latch as a general held
state (E49); only the first rocket charge of an air time falls slowly; the Settler's jab > jab > axe stays (nothing built).

### Milestone 1: the monkey flip, decision 75 and the cheap gaps (one commit)

* **E49 the latch** (`Move::latch { x height clock_num clock_den }`, `grab::connect` / `update` / `pin`, `fighter::held_fall`): a
  command grab whose catch move has a latch holds its fighter on the ordinary hold clock (`grab_base_frames + grab_percent_tenths x
  percent`, times `clock_num / clock_den`) with the shared mash; the held fighter is not pinned in front but falls on its own (gravity
  to its top fall speed, landing on platforms; frozen in hitlag), and the holder rides it, `x` ahead of its feet and `height` percent
  of its body height up. The clock running out lets go as an ordinary hold. Exit moves have a latch with no clock (they ride until
  their throw). The repeating hit is pummel rows, the exits a script: nothing character-specific in the engine.
* **`Hitbox::fixed_stun`**: a hit that does not tumble holds its victim exactly that many frames (the jump-off's 20).
* **E50, decision 75**: hashed `Fighter::air_marks` (cleared by landing or a ledge, **not by a hit**), the script functions
  `air_mark(n)` / `set_air_mark(n)`, and `Move::flag_versions` (`flag_version { flags marks move }`): starting a special, the first
  version whose air flags or marks are set starts its move instead, or nothing at all when it names none (checked before the fighter
  turns). The rocket's charge sets mark 0 in the air; a later charge in the same air time is `ROCKET_AGAIN` (ext5) on the body's own
  fall; the kick sets air flag 1, which bars the up special.
* **E51**: `Hitbox::unblockable` (the flying barrels: a raised or passive shield does not stop them) and `Move::self_damage` (the
  crash: 5% to the scamp [E]).
* The `jump_tap` register (the latch jumps off on a new press only, not one held since before the flip).
* **The monkey flip** (`flip_special` in `moves_popgun.rs`; side special, `FLIP_AIR` ext9, `FLIP_KICK` ext10, `LATCH` ext11,
  `LATCH_ATTACK` ext12, `LATCH_JUMP` ext13; scripts `popgun_flip`, `popgun_flip_kick`, `popgun_latch`, `popgun_latch_attack`,
  `popgun_latch_jump`):
  * the leap: a crouch, launched on 13 (2.2 of the reference's units a frame ahead, 1.7 up; 2.8 ahead for a smash input [E]), no air
    control [E]; command-grab boxes on 18-36 (ground) / 18-29 (air) [R]; FAF 62 / 48, landing lag 26 [R]; not helpless [R]; the air
    version stops the scamp's momentum [E]; ledges in front once falling from the launch [E];
  * the kick on a second press of special from the launch to the end of the grab [E: the window]: 14% / 12% on 6-10, 12% / 10% on
    11-25, 361, BK 80, KS 50 [R]; FAF 33, landing lag 18 [R]; the up special barred until landing or a ledge [R] (a hit clears it [E]);
  * the latch: a 1% scratch (361, BK 0, KS 0) on its frames 11 and 29 of every 48 [R damage; E rhythm], on the ordinary hold clock;
    attack: a 3% scratch on 18 (the held fighter, and others near it, BK 60, KS 150) and the 7% throw (361, BK 50, KS 100) on 20, the
    scamp flipping off back and up, FAF 37; jump: the 2% kick (361, BK 0, KS 50) on 18 holding the fighter 20 frames when it does not
    tumble, the scamp jumping off at its full hop's speed [E], FAF 43.
* The fuzzers drill the latch (fuzz-rollback: every third run fighter 2 is the scamp flipping at fighter 3; net-fuzz: every eighth run
  from 4 the host is the scamp flipping at the joiner) and count latches begun, exits and broken.

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/popgun_flip.rs` (15): every part's frames, hits and FAF in the data; a standing fighter caught in the window and ridden
(0.9 of its height up, straight over its feet); the scratch on the latch's frames 11 and 29 and 48 later (each scratch's hitlag
counted); the hold clock (90 frames at 0%, the scamp then airborne) and mashing cutting it by more than half; the attack exit's 3% + 7%
and launch, the scamp free; the jump exit's 2%, the 20-frame hold, the scamp rising, free; a held jump from before the flip not
jumping off; an airborne fighter caught and falling with the scamp on it; offstage, both falling to their knock-outs; a shielding
fighter caught; the whiffed flip's FAF 62 (ground) and 48 (air, free after) and the 26 frames of landing lag; the smash input further;
the kick's clean hit and the up special barred after it (nothing starts, nothing turns) until landing; the kick's flag cleared by
landing; only the first rocket charge of an air time falling slowly (the mark kept through a real hit, cleared by landing); a ledge
caught by the leap; resimulation from a snapshot mid-latch. `popgun_rocket.rs`: the crash's 5% to the scamp, the barrels unblockable.
`move_rules.rs`: the new fields in the content hash; an unblockable hit through a shield; `fixed_stun`; `self_damage`; flag versions
(pick, bar, flags vs marks). `scripted_moves.rs`: `jump_tap`; `air_mark` kept through a hit and cleared on landing. `state.rs`: the
checksum case `air_marks`. sim-content: `the_popgun_flip_fields_round_trip_and_are_checked`. `popgun.rs`: the side special is now the
Popgun's own.

### Estimated or limited [E]

* The leap's launch frame, speeds, arc and smash-input distance; its grab boxes; the kick's input window and positions; the kick's trip
  chance (none here; the previous version's was 30%).
* The latch's ride point; the held fighter falling on its own gravity with no drift; the scratch rhythm (our reading of UFD's
  "11 / 29 / 59..."); the attack exit's backflip and the jump's speed; the latch attack's 5% second throw row (unused).
* Whether a hit clears the kick's up-special bar (here yes); the crash's 5% (the article's general figure).
* Still gaps: the popgun's muzzle hit (no angle, knockback or size published: a small data change once known) and its taller later
  box; catching a peanut (needs an aerial item catch for everyone: E4 has none, a broader change for the owner).

### Tests and checks run (after merging `movesets` c0f4e88, sim 117: already contained, nothing to merge)

`cargo fmt --all`; `cargo test --workspace` (all green: 1,436 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D
warnings`; `pftool fuzz-rollback 300`: 0 mismatches, class 11 drawn 181 times, latches 164 begun / 123 exits / 41 broken; `pftool
net-fuzz 100`: 0 mismatches, class 11 drawn 22 times, latches 17 begun / 11 exits / 5 broken. Godot tests: nothing they check changed (roster size,
names and accessors are as in phase 1).

### Training-mode checks (hitbox view)

1. The flip: side special at a standing computer opponent 6 units away: the scamp lands on its head; mash on the opponent's pad and
   watch the latch end sooner; press attack (the 3% + 7% throw) or jump (the kick and the 20-frame hold).
2. Offstage: flip onto an opponent recovering low past the ledge: both fall (jump to escape).
3. In the air: flip, press special again for the kick, then up special: nothing comes out until the scamp lands.
4. The rocket: charge in the air (slow fall), get hit out of it, charge again: the second falls at the normal speed.

### For the art agent

* **Clips needed** (they fall back to shared archetypes today): the flip's crouch (1-12) and the forward flip (launched on 13, the grab
  hands out 18-36 / 18-29); the kick (6-25, the leg straight out); the latch: sitting on the opponent's head, scratching on its frames
  11 and 29 of every 48; the latch attack (the scratch on 18, the throw on 20, the backflip off); the latch jump (the kick down on 18,
  the jump up). The held fighter needs a "ridden" pose (it is `Grabbed` with the scamp above it: `fighter_grab_partner` tells the view
  who rides whom).

## Questions for the owner

1. **Footage of the monkey flip**: the leap's distance and arc (and whether a smash input goes further in 13.0.1), the latch's
   scratch rhythm and the exits' paths would replace the estimates above.
2. **The kick's trip chance**: the previous version's 30% is not confirmed for 13.0.1; keep none, or use 30%?
3. **The kick's up-special bar**: does a hit clear it (as here) or only landing (switch it from the air flag to an air mark)?
4. **The crash's 5% self damage**: the figure is the article's general text; keep, or remove until a 13.0.1 source confirms it?
5. Still open from phase 1: footage for the rocket's flight and the peanut's and peel's arcs.

## Sim 143: landing speed (owner decision 102)

The shared landing cap (`docs/movesets/SHARED_MECHANICS.md`, "Landing speed": a landing keeps at most the body's dash speed) changes
one Popgun landing: the monkey flip steered into the stage landed faster than the dash speed and slid 26 reference units; now 19.
Test: `landing_momentum::no_special_lands_faster_than_its_bodys_dash_speed`.
