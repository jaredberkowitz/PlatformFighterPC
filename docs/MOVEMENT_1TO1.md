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

TABLES

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
