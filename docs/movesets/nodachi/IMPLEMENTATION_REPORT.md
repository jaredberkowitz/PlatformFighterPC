# Implementation report: the Nodachi (phase 1)

Class 13 **Nodachi** on the **exile** body (built-in fighter **Exile**), modelled mechanically on the reference game's Sephiroth (13.0.1).
Phase 1, sim 125, branch `nodachi-phase1` (from `movesets` at d135bfa, merged with `origin/movesets` 042ce42: sim 124's run mode R2a and
content schema 8). Reference data: `REFERENCE_DATA.md`; engine trace and extensions: `PROJECT_AUDIT.md`.

## Built

* **The body** (`FighterParams::exile`): every 13.0.1 attribute (weight 79, walk 0.9, initial dash 1.92 for 10, run 1.86, traction
  0.11, air speed 0.95, air acceleration 0.01 + 0.07, gravity 0.108, fall 1.84 / 2.944, jumps 30.5 / 10.0 / 34.0, ledge jump 35.25, heavy
  landing lag 2, dodges 26 / 30 / 35 / 45 / 62 / 75 / 98, no wall jump).
* **Every normal** on [R] frames and hit data (`sim-core/src/moves_nodachi.rs`): the jab's held rule (a script: from 10 after a hit, 19
  without), the aimed forward tilt's three capsules, the up tilt's lift (a position vector to the blade's tip) and upper-body
  intangibility, the dash attack, the three smashes with their charge frames, the five aerials (the down air's stall, its 4.2 plunge and
  landing hit), grabs, pummel, the four throws with their pre-hits, the ledge and get-up attacks. The blade reaches 30 of the reference's
  units; the forward and up smashes' blade positions were read from the reference's hitbox animation [E].
* **The wing** (E57, general): the start table (every published example reproduced to 0.03%), the points (110 / 100 / 90 / 65 / 50, 0.3
  a percent of base damage dealt, 0.15 with more opponents, 50 a KO and the stock-difference move), x1.3 damage on hits, pummels and
  projectiles (not items) with the knockback formula's damage term unchanged, the body multipliers, a third jump given at once and taken
  when it goes, 20% armor on the smash attacks (forward 10-23, up 14-22, down 10-20) through `Move::armor_mode`, gone until a KO.
* **The up special**: the blade dash (hold on 18, aim on 19, the 7% hit on 25-28, FAF 57 on the ground, the animation to 80 in the air,
  landing lag 27, ledges from the dash, the grounded dash stopping at the edge) and the eight slashes (aim on 31, hits on 36, 38, ..., 48
  and 50 for 23.1%, the drag as the autolink angle, FAF 105, landing lag 56, no ledges until it ends, stopping at the edge).
* **The neutral special**: the charge (release from 20, shield or jump cancels in 7 frames), the flare (fireball from 41, its 10.5% burst
  3 frames after contact, FAF 84), the bigger flare (31 frames held, the vortex multi-hits and 9% finisher 10 frames after contact,
  FAF 104), the giant flare (at full charge, fireball on 130, three rings at once 7 frames after contact, FAF 182).
* **Registration**: `CLASSES` 14, the content placeholder (`exile`, `nodachi`), `content/base.pfc` re-exported, `godot/scripts/roster.gd`
  (`Nodachi`, the built-in `Exile` with the shared default look, an original blurb), the Godot tests that count fighters, the fuzzers.

## Verified against [R] / [D] (tests that ran)

* `sim-core/tests/nodachi.rs` (9): every normal's windows, FAF, landing lag and autocancels in the data; every attack's first hit and
  FAF by running it; 37 hits' damage, angle, BK, KS and hitlag; hitstun modifiers, angle types, the aimed tilt, the up tilt's position
  vector and tip height, the down air's spike and grounded box, the down smash's shield damage and no-rebound flags, the pummel and
  throws; the blade's reach; the down air's stall and plunge; the jab's held rule; the jumps' air times (27 / 45 within a frame); every
  body number.
* `sim-core/tests/nodachi_wing.rs` (7): the start table against ssbwiki.com's examples; the wing coming at 80% level, 30% two behind,
  not at 100% two ahead, never for a body without one; the multipliers and three jumps; x1.3 damage and the knockback shift; points by
  damage dealt and the worked KO example (90 -> 15); lost for good until a KO, back after one; the smash armor only with the wing (and
  broken by 25%); a resimulation check.
* `sim-core/tests/nodachi_specials.rs` (11): the form chosen on 18; the dash's hit on 25, FAF 57, travel and the edge stop; the air
  dash's special fall on 80; the eight hits on their frames for 23.1%; the flare's hit on 41 and burst on 44, FAF 84; the bigger flare
  from 31 frames held and its FAF 104; the giant flare's FAF 182; the shield cancel; the bursts' vortex and rings (inner 3.7%, outer 2%
  by distance); the wing powering the flare; a resimulation check with two exiles.
* `move_rules.rs`: the wing, armor mode and edge stop in the content hash; rings out together. `state.rs`: checksum cases `wing_points`,
  `wing_spent`, `hit_by`. sim-content: `the_nodachi_fields_round_trip_and_are_checked` (round trip, older files unchanged, bad values
  refused).
* `cargo test --workspace` (all green), `cargo clippy --workspace --all-targets -- -D warnings` (clean), `cargo fmt --all`;
  `fuzz-rollback 300`: 0 mismatches (134 exile fighters, a quarter with the wing out); `net-fuzz 100`: 0 mismatches (21 exile fighters).

## Estimates [E] and conflicts

* Every box on the blade's bone and the dash's rotating root (the forward and up smashes read from the hitbox animation).
* The forward smash's 13.5% root box ranked last (conflict 2 in `REFERENCE_DATA.md`).
* The blade dash's travel (11 a frame for 4 frames, a quarter kept) and the eight slashes' (3.3 a frame for 16), the up special's air
  stall in its startup, its boxes not turning with the dash.
* The flares' flight speeds (1.53 / 1.86 / 0.43 of the reference's units a frame) and the giant flare's charge end (108).
* The wing with several opponents (the leader), the KO credit (the last fighter to raise the victim's percent since it came back).
* The up tilt FAF (45, conflict 1); the flare's end-of-flight burst a frame later than the table (conflict 3).

## Not built (phase 2)

The shadow orbs (side special), the barrier (down special), the down smash without ground contact, the forward air's wall stick, the
eight slashes' landing hit. See `PROJECT_AUDIT.md` ("Not built").

## Owner questions

In `../ROSTER_STATUS.md` ("Open for the owner from the Nodachi's phase 1"): the forward smash's root box ranking, the unpublished
distances and speeds, the wing with several opponents, the KO credit, moving while charging the flare, and approval to build the shadow
orbs' held-projectile mechanic general.

## For the art agent

Every move falls back to the shared archetypes until it has its own clip. Moves that need their own (reference frames; boxes in
`moves_nodachi.rs`):

* The wing itself: a single dark wing on one shoulder while `Fighter::wing_points > 0` (bridge accessor not yet added: a small follow-up).
* Jab 1-2 (kicks, 5-7) and jab 3 (a pommel thrust, 7-9).
* Forward tilt (14-16): a long level slash, aimed up or down; up tilt (10 a knee lift, 13-25 a vertical thrust reaching 48 units up);
  down tilt (9-22, a low sliding kick); dash attack (14-19, a burst of dark magic from the hand).
* Forward smash (24-25): a great level slash, the blade 9 up from a hilt 8 ahead; up smash (23-28): the blade rising level at head
  height, through vertical to behind the head; down smash (21-23): the blade driven into the ground ahead, an eruption at its tip.
* Neutral air (9-10): a snap of the fingers, a burst round the body; forward air (13-18): a long stab ahead; back air (15-16): a slash
  behind; up air (16-21): an overhead arc front to back; down air (19-39): a stall, then a plunge with the blade straight down, a landing
  shockwave.
* Throws: forward (a point-blank burst), back (the fighter sent behind), up (an upward slash), down (a slam).
* The blade dash (25-28) and the eight slashes (36-50): a dash along the aim, the slashes in a flurry; the flare's charge (the hand's
  flame changing colour at 31 and full), the three fireballs and their bursts (small, wide lingering, giant three-ring).
