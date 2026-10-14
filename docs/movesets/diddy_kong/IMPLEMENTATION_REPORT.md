# Implementation report: the twelfth moveset (reference: the reference game's Diddy Kong)

Class 11 **Popgun** on the **scamp** body, built-in fighter **Scamp** (owner decision 65). Reference data: `REFERENCE_DATA.md`; engine
audit: `PROJECT_AUDIT.md`. Everything is 13.0.1 (decision 68 applies; the older jab rule is not used).

## Phase 1 (sim 117, content schema 7 unchanged; branch `popgun-phase1` from `movesets` at faff36e, sim 115)

Sim 116 belongs to another session; 117 was taken as asked. The same branch also carries the Settler's axe change (owner decisions 72-73,
its own commit: `docs/movesets/villager/IMPLEMENTATION_REPORT.md`, "Sim 117").

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

## Phase 2 plan: the monkey flip (side special), and the gaps

* **Research first**: the latch's 13.0.1 numbers are unpublished (its length and escape, the scratch's rhythm, the leap's path and grab
  radius, the kick's trip chance in 13.0.1). Footage of the reference's hitbox view (the owner's recordings) would settle them; without it
  they would be labelled [E] from the previous version's page.
* **E49 latch** (a proposal for the owner: a general held state, broad enough to explain first): a command grab (E6) whose `catch_move` is
  a hold on the victim's head (`Move::hold_at` above the victim), with the ordinary hold's clock and mash-out (`grab::hold_time`,
  `mash_cut`), an automatic repeating hit (a `rehit` loop on a pummel-like row), and two exits by button: attack (the 3% scratch on 18-20
  and the 7% throw on 20, FAF 21 / 36) and jump (the 2% footstool kick with its 20-frame immobilisation when it does not tumble: a
  general `Hitbox::stun_frames`-like field). Today's command grab holds with no clock or mash-out, so the latch needs at least the clock and
  the mash on a command hold, which is the general part.
* **The leap and kick**: two moves (ground 18-36 / air 18-29 grab windows, FAF 62 / 48, landing lag 26), the kick on a second press
  (6-10 / 11-25, 14 / 12% and 12 / 10%, landing lag 18), the smash-input leap further (`smash_version`), the kick barring the up special
  until landing (an air flag the up special's script reads).
* Gaps to close if the owner wants them: the popgun's muzzle hit, catching peanuts (an aerial item catch for everyone), unblockable
  projectiles (`Hitbox` flag; the barrels and the lying peel), the crash's self damage, the first-charge-only slow fall (a per-air-time
  flag that a hit does not clear).

## Questions for the owner

1. **The monkey flip's latch** (phase 2): approve the latch as a general held state (E49: a command hold with the ordinary clock and
   mash-out, an automatic repeating hit and two button exits), and can footage of the reference's latch be recorded (its length, the
   scratch rhythm, the leap's arc)?
2. **The rocket's flight path**: only the launch speed is published; the flight's fall-off (even to nothing over 54 frames), its steering
   and the charge's slow fall are estimates. Footage of a tap and a full charge from the ground would let them be fitted, as the up
   specials' rises were.
3. **The peanut's arc** and the **peel's arc** are estimates (no published speeds beyond the peel's toss): keep, or fit from footage?
4. **The slow fall for the first charge only**: the engine's once-per-air-time flags are cleared by a hit; make one that is not (so a
   charge after being hit out falls at the body's speed, as the reference), or keep every charge slow?
