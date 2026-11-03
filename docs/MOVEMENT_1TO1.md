# Movement against the reference, 1:1 (2 November 2026, sim 148)

The owner asked for a 1:1 comparison of movement with the reference: air speed, air drift and acceleration, ground traction and the
rest, so the game feels as close to it as possible. This page holds every fighter's movement numbers next to the reference's, the rules
that use them side by side, what was different, what sim 148 changed, and what is still unverified.

## Sources

- **The numbers**: each reference character's parameter file as published for the final version (dragdown.wiki's per-character Data
  pages: walk, dash, run, traction, air speed and both air accelerations, air friction, gravity, fall and fast fall, the jump heights
  and the ground jump's horizontal rule), cross-checked with ultimateframedata.com's stats page and ssbwiki.com.
- **The airtimes**: ultimateframedata.com's stats page (short hop, full hop, both fast fallen), a whole-arc check of jump timing.
- **The rules**: a public re-creation of the reference's movement physics (its kinetic code: the ground stop, the run brake, the dash,
  the air control, the integrator that applies acceleration, a cap and a brake), with the reference's common parameter file (a public
  decode: `dash_speed_keep_frame` 6, `stop_over_speed_brake_mul` 2, `run_brake_brake_mul` 1, `damage_ground_mul` 1). The integrator's
  rule, from that re-creation: the speed accelerates toward a cap and anything above the cap is pulled down by the brake alone.
- **Our side**: the simulation itself, frame by frame (`sim-core/tests/movement_trace.rs`, an ignored measuring test), compared by a
  script against a model of the reference's rules fed its own numbers.

Units are the reference's (our world unit is 8 of them), speeds a frame.

## The rules, side by side

| Rule | The reference | Ours before sim 148 | Ours now |
|---|---|---|---|
| Stopping on the ground (standing, crouching, a landing, a shield, an attack) | Traction, doubled if the stop began faster than the body's walk speed (decided as the state begins) | Always the doubled traction | **As the reference** (`traction_rule` 1) |
| Letting go of a run (the run brake) | Traction, never doubled | Doubled traction: stopped in half the frames | **As the reference** |
| Run speed above the run's own (left from the dash) | Braked at traction | Braked at 0.06 for every body | **As the reference** |
| The dash's speed | Its `dash_speed` from the first frame, kept 6 frames; then held forward the run's acceleration toward the run speed (traction above it), anything else traction | Started at 0.9, eased up 0.3 a frame to `dash_speed`, kept to the dash's end whatever the stick | **As the reference** (`dash_keep_frames` 6) |
| A standing turn | Traction | Doubled traction | **As the reference**; a run's skid keeps the doubled brake |
| A shield pushed back by a hit | Traction (`damage_ground_mul` 1), never doubled | Doubled traction: half the reference's pushback distance | **As the reference** |
| Air drift with the stick part way | The cap is the air speed times the stick; above it only the air friction brakes | The air acceleration pulled the drift down to the stick's share at once | **As the reference** (`air_drift_brake` 1) |
| Air drift: acceleration, cap, letting go, turning round, speed above the air speed | Base + stick share toward the air speed times the stick; air friction above it and with the stick let go | Same | Same |
| A jump's first airborne frame | Moves (the jump rises on it) | Stood a frame at the ground's height: every jump one frame longer than the published airtimes | **Moves** (no switch: a timing fix) |
| Jump heights, gravity, fall and fast fall speeds, the jump squat | Per body | Same numbers | Same |
| A ground jump's sideways speed | The stick's share plus the ground speed's kept share, capped (a public re-creation of the reference's jump agrees) | Same since sim 144 (sim 145: a reverse aerial rush keeps the run's share) | Same |

Each new rule is one ruleset switch (`docs/CONTENT.md`), so the older feel can be tried against it: `traction_rule`, `dash_keep_frames`
and `air_drift_brake`, each 0 for the rule before sim 148.

What it changes in play, for the Longsword (Marth) as an example: a run let go now skids 18 frames and about 16 units (9 frames and
7.5 before); a walk let go slides 10 units (4.7); the dash covers 21.6 units in its 10 frames (20.1) and is at full speed on its first
frame (1.2 of 2.26 before); easing the stick to half while drifting keeps 0.96 of the 1.07 drift after 30 frames (0.54: it stopped at
once); every jump leaves the ground a frame sooner and lands a frame sooner (all 56 published airtimes now match exactly).

## Every fighter

### Attributes (the reference value; a bold number is ours where it differs)

| Fighter (reference) | Walk | Initial dash | Run | Traction | Air speed | Air accel. base + stick | Air friction | Gravity | Fall | Fast fall | Full hop | Short hop | Midair jump |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Longsword (Marth) | 1.575 | 2.255 | 1.964 | 0.114 | 1.071 | 0.01 + 0.07 | 0.00375 | 0.075 | 1.58 | 2.528 | 33.66 | 16.26 | 33.66 |
| Claws (Wolf) | 1.208 | 2.09 | 1.54 | 0.11 | 1.281 | 0.01 + 0.08 | 0.004 | 0.13 | 1.8 | 2.88 | 32.02 | 15.38 | 30.71 |
| Needles (Sheik) | 1.47 | 2.178 | 2.42 | 0.106 | 1.155 | 0.01 + 0.08 | 0.015 | 0.15 | 1.75 | 2.8 | 39 | 18.75 | 40 |
| Embers (Mario) | 1.155 | 1.936 | 1.76 | 0.102 | 1.208 | 0.01 + 0.07 | 0.015 | 0.087 | 1.5 | 2.4 | 36.33 | 17.54 | 36.33 |
| Gauntlets (Captain Falcon) | 0.987 | 1.98 | 2.552 | 0.105 | 1.218 | 0.01 + 0.065 | 0.0075 | 0.12 | 1.865 | 2.984 | 37.31 | 17.98 | 37.31 |
| Staff (Palutena) | 1.271 | 2.035 | 2.077 | 0.106 | 1 | 0.01 + 0.105 | 0.01 | 0.12 | 1.55 | 2.48 | 35.9 | 17.3 | 35.9 |
| Greatsword (Cloud) | 1.155 | 2.145 | 2.167 | 0.106 | 1.155 | 0.01 + 0.06 | 0.009 | 0.098 | 1.68 | 2.688 | 32.5 | 17 | 32.5 |
| Bracers (Donkey Kong) | 1.365 | 2.09 | 1.873 | 0.123 | 1.208 | 0.01 + 0.05 | 0.015 | 0.085 | 1.63 | 2.608 | 34 | 17.3 | 35.5 |
| Clutch (Yoshi) | 1.208 | 1.98 | 2.046 | 0.095 | 1.344 | 0.03 + 0.068 | 0.005 | 0.08 | 1.29 | 2.064 | 36.09 | 14.43 | 51.56 |
| Buckler (Link) | 1.247 | 1.98 | 1.534 | 0.113 | 0.924 | 0.01 + 0.04 | 0.0038 | 0.096 | 1.6 | 3.04 | 27.8 | 13.38 | 29 |
| Pockets (Villager) | 1.092 | 1.815 | 1.397 | 0.131 | 0.987 | 0.01 + 0.05 | 0.01 | 0.078 | 1.32 | 2.112 | 32.5 | 15.69 | 32.5 |
| Popgun (Diddy Kong) | 1.313 | 2.09 | 2.006 | 0.093 | 0.924 | 0 + 0.05 | 0.008 | 0.125 | 1.75 | 2.8 | 41.21 | 19.9 | 41.21 |
| Dagger (Joker) | 1.12 | 1.9 | 2.06 | 0.13 | 1.1 | 0.01 + 0.07 | 0.01 | 0.127 | 1.63 | 3.097 | 32.5 | 14.2 | 34 |
| Nodachi (Sephiroth) | 0.9 | 1.92 | 1.86 | 0.11 | 0.95 | 0.01 + 0.07 | 0.008 | 0.108 | 1.84 | 2.944 | 30.5 | 10 | 34 |

### Jump airtimes in frames (ours / the reference's published: short hop, full hop, both fast fallen at the peak)

| Fighter | Short hop | Full hop | Short hop, fast fall | Full hop, fast fall |
|---|---|---|---|---|
| Longsword | 41 / 41 | 55 / 55 | 28 / 28 | 38 / 38 |
| Claws | 30 / 30 | 43 / 43 | 21 / 21 | 31 / 31 |
| Needles | 32 / 32 | 47 / 47 | 23 / 23 | 33 / 33 |
| Embers | 40 / 40 | 56 / 56 | 28 / 28 | 39 / 39 |
| Gauntlets | 34 / 34 | 48 / 48 | 24 / 24 | 34 / 34 |
| Staff | 34 / 34 | 50 / 50 | 24 / 24 | 35 / 35 |
| Greatsword | 37 / 37 | 49 / 49 | 26 / 26 | 34 / 34 |
| Bracers | 40 / 40 | 53 / 53 | 27 / 27 | 37 / 37 |
| Clutch | 38 / 38 | 60 / 60 | 26 / 26 | 42 / 42 |
| Buckler | 33 / 33 | 45 / 45 | 22 / 22 | 30 / 30 |
| Pockets | 40 / 40 | 56 / 56 | 28 / 28 | 39 / 39 |
| Popgun | 36 / 36 | 51 / 51 | 26 / 26 | 36 / 36 |
| Dagger | 30 / 30 | 45 / 45 | 20 / 20 | 30 / 30 |
| Nodachi | 27 / 27 | 45 / 45 | 18 / 18 | 31 / 31 |

### Behaviour, measured (ours / the reference's rules with its numbers; reference units)

| Fighter | Dash, frame 1 speed | Dash distance over its frames | Run let go: frames, distance | Walk let go: distance | Landing at the jump cap: slide | Air, half stick from full drift: speed after 30 frames |
|---|---|---|---|---|---|---|
| Longsword | 2.255 / 2.255 | 21.63 / 21.63 (10 f) | 18 f, 15.96 / 18 f, 15.95 | 10.11 / 10.10 | 5.66 / 5.66 | 0.961 / 0.959 |
| Claws | 2.090 / 2.09 | 19.80 / 19.80 (10 f) | 14 f, 10.01 / 14 f, 10.01 | 6.03 / 6.03 | 4.85 / 4.84 | 1.164 / 1.161 |
| Needles | 2.178 / 2.178 | 15.41 / 15.41 (7 f) | 23 f, 26.43 / 23 f, 26.42 | 9.47 / 9.46 | 8.36 / 8.36 | 0.708 / 0.705 |
| Embers | 1.936 / 1.936 | 18.73 / 18.73 (10 f) | 18 f, 14.32 / 18 f, 14.31 | 5.98 / 5.97 | 5.16 / 5.15 | 0.761 / 0.758 |
| Gauntlets | 1.980 / 1.98 | 34.19 / 34.19 (15 f) | 25 f, 29.75 / 25 f, 29.75 | 4.16 / 4.16 | 10.35 / 9.38 (a) | 0.995 / 0.993 |
| Staff | 2.035 / 2.035 | 20.52 / 20.52 (10 f) | 20 f, 19.33 / 20 f, 19.32 | 6.99 / 6.98 | 4.97 / 4.97 | 0.703 / 0.700 |
| Greatsword | 2.145 / 2.145 | 21.54 / 21.54 (10 f) | 21 f, 21.09 / 21 f, 21.08 | 5.72 / 5.72 | 4.07 / 4.07 | 0.888 / 0.885 |
| Bracers | 2.090 / 2.09 | 29.49 / 29.49 (15 f) | 16 f, 13.34 / 16 f, 13.34 | 6.90 / 6.90 | 6.60 / 6.60 | 0.761 / 0.758 |
| Clutch | 1.980 / 1.98 | 28.25 / 28.25 (14 f) | 22 f, 21.03 / 22 f, 21.02 | 7.09 / 7.08 | 8.36 / 8.35 | 1.198 / 1.194 |
| Buckler | 1.980 / 1.98 | 21.74 / 21.74 (12 f) | 14 f, 9.67 / 14 f, 9.66 | 6.26 / 6.26 | 5.71 / 5.71 | 0.810 / 0.810 |
| Pockets | 1.815 / 1.815 | 21.14 / 21.14 (13 f) | 11 f, 6.77 / 11 f, 6.76 | 4.02 / 4.02 | 3.59 / 3.59 | 0.690 / 0.687 |
| Popgun | 2.090 / 2.09 | 20.56 / 20.56 (10 f) | 22 f, 20.67 / 22 f, 20.64 | 8.63 / 8.62 | 8.34 / 8.33 | 0.686 / 0.684 |
| Dagger | 1.900 / 1.9 | 19.63 / 19.63 (10 f) | 16 f, 15.31 / 16 f, 15.30 | 4.28 / 4.28 | 3.00 / 3.00 | 0.803 / 0.800 |
| Nodachi | 1.920 / 1.92 | 18.96 / 18.96 (10 f) | 17 f, 14.80 / 17 f, 14.80 | 3.24 / 3.24 | 3.86 / 3.86 | 0.712 / 0.710 |

Every attribute matches the parameter file (no bold cell), so the measured differences were all in the rules. (a) The Gauntlets' jump cap
(2.1) is faster than their dash speed (1.98), so the owner's landing cap (sim 143, decision 102: a landing keeps at most the dash
speed) cuts it at the touchdown first; the reference has no such cap. The braking after it is the reference's (the comparison script
lines the touchdown frame up a frame differently there).

## Still different or unverified

- **The walk.** The reference walks with the stick's tilt and has three walk speeds' worth of thresholds (`walk_slow_speed_mul`
  0.2, `walk_middle_ratio` 0.5, `walk_fast_ratio` 0.75); how its walk speed follows the stick is not in the public re-creation.
  Ours reaches full walk speed at the stick's 0.6 (the flick threshold). Unverified either way.
- **The jump squat.** The re-creation's notes say a jump squat brakes its momentum at traction; ours keeps the ground speed whole (no
  braking: `docs/MOMENTUM.md`). A run jump would leave at about the same speed (most bodies' runs reach the jump's cap either way);
  a walk jump would leave a little slower. Left until footage settles it.
- **The end of a let-go dash.** The reference has `dash_end_speed_mul` 0.67, probably taking a third off the speed as a let-go dash
  ends; ours takes `dash_brake` (0.52) off instead. Both give about the same slide (within 2 units on most bodies).
- **Walking or running off a ledge.** Ours keeps 85% of the ground speed (at most 1.25 times the air speed); the reference holds the
  ground speed apart in the air (`air_ground_speed_brake`, three times the air friction, and `ground_speed_inherit_limit`). Not
  changed.
- **The midair jump's sideways speed.** Ours: the stick times the air speed times the body's `jump_aerial_speed_x_mul`. The
  reference's rule is not in the re-creation. Not changed.
- **A run's skid (turnaround).** The reference doubles its brake under a condition the re-creation does not name; ours always
  brakes it at the doubled traction.
- **Ground knockback and wavedash slides.** The reference slides a grounded hit at traction (`damage_ground_mul` 1) and a waveland
  on the landing's traction; ours keeps the doubled friction for hits (jab follow-ups were measured on it) and the wavedash's own
  `waveland_friction`. Candidates for a later pass, with the moveset tests.
- **Rolls and spot dodges** keep the doubled friction for their leftover speed: the reference's are driven by their animation, and
  ours were fitted to it.
- **Heavy landings.** The reference lands heavy after falling at full speed for a body's set time; ours as soon as it falls at full
  speed (or fast falls).

## Side effects measured in the moveset tests

- The Longsword's jab (`longsword_jab.rs`): a dash attack two frames into a dash now misses after jab 1 (it was a true combo at
  90-150%), and catches the foe jab 2 launched at 0-20% (nothing on the ground did): the full-speed dash reaches further sooner.
- The staff seer's neutral air loop (`combos.rs`, the owner's reference clip): still a true combo throughout, with the jump held toward
  the target through the squat (the landing before it now stops on traction short of the fleeing target).
- Two bugs found on the way: bodies on the ground could swap sides when one stepped past the other's middle in a single frame (now
  pushed back to the side it came from); and a stand after a jump out of a dash was held back for that old dash's interrupt frame.

## How to measure again

`cargo test --release -p sim-core --test movement_trace -- --ignored --nocapture` prints every trace (`T <class> <case> <values>`,
raw fixed point); the comparison is a short script against the parameter file's numbers and the rules above.
