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

## Owner questions (phase 1)

Answered on 2026-10-14 as owner decisions 87-92 in `../ROSTER_STATUS.md`: every phase 1 choice kept (the root box ranked last, the
estimates, the leader for several opponents, the KO credit with no time limit, the flare's charge standing), the orbs built general
but no bigger than the exile needs.

# Phase 2 (sim 128, branch `nodachi-phase2`)

From `origin/movesets` at 3fc1e80 (sim 126), merged with `origin/movesets` a94f78a (sim 127, the run's R2b); content schema 8 unchanged.
Reference data: `REFERENCE_DATA.md` ("Side special", "Down special", "Phase 1's leftovers"); engine trace: `PROJECT_AUDIT.md` ("Phase 2").

## Built

* **The shadow orbs** (side special): the charge from frame 9 up to 26 frames (`nodachi_orbs.script`), the beam out 8 frames after it
  (16-22 to 42-48, FAF 39 to 65), its two capsules (r 2.0 for everyone, r 3.5 for fighters in the air) at 1.611% growing to 4.5%
  (`ChargedShot::every_hit`), faster when charged. Its hit puts orbs on the fighter (**E59**, `Move::orbs`, `sim-core/src/orbs.rs`): one,
  two from 17 frames of charge, three at full, twice as many from a reflected beam, five on a fighter and five out at most, none on a
  fighter with another exile's orbs or whose orbs are striking; the clock of 240 frames, lengthened 30 to 180 by a later hit's charge,
  never past 450 from the first hit; circling, each hits other fighters for 7% (rehit 45, twice at most); then each strikes, 13 frames
  into the chest, exploding on 14 for 6.5% away from the exile; a striking orb reflected or pocketed flies on as a shot; a KO of the
  fighter takes its orbs away.
* **The barrier** (down special): the catching capsule in front on 7-27 (the body does not count); not struck, the answer (a standing
  projectile, reflectable) on 38-52 (1% vortex multi-hits every 5) and 53 (5%), FAF 87; struck, no hitlag for the exile
  (`Counter::still`), intangible 1-11, the answer on 7-21 / 22 read between 2% / 6.5% and 4% / 18% by the caught damage (`Move::lerp`:
  damage, BK, KS, radius, shield damage, hitlag), FAF 52; a hit above 25% breaks it (`Counter::break_above`): no damage, a stagger, FAF 41.
* **The down smash without ground contact**: its ground test on frame 21 (`Move::probe`), the blade alone on 21-22 (`DSMASH_OFF`).
* **The eight slashes' landing hit** (2.3%, 60, BK 90, KS 60 on the landing's frame 1), landing in the move (`landing_move`) or in the
  special fall after it (`Move::fall_landing_move`).
* **The forward air's wall stick**: on its hit frames a wall the blade meets sticks the exile (the script function `ground()`), three
  times an air time, not with the ground less than 14 units below; from frame 18 jump, attack or up jumps off (FAF 23), away or down
  lets go (FAF 20), and it lets go by itself after 120 frames.

## Verified against [R] / [D] (tests that ran)

* `sim-core/tests/nodachi_phase2.rs` (13): the beam's hit on 16 for 1.611% and FAF 39, on 42 for 4.5% and FAF 65 at full, its range by
  charge; the orb counts by charge (24 / 25 / 33 / 34 ticks held) and the five cap, the reflected doubling; the strike 240-254 frames after
  the first hit, 6.5% each, the orbs gone; a later hit's extension and the 450 cap; a bystander hit twice (7%, 45 frames apart) and the
  orb used up; a KO clearing them; a resimulation check; the barrier's miss (38 to 53, 1% / 5%, FAF 87); a caught 9% jab answered with 3%
  multi-hits and 12.25% (the formulas), a caught 20% with the maximum 4% / 18%, the exile unhurt; the answer's frames and intangibility;
  26% breaks it, 25% does not; the down smash at an edge switching to the blade alone; the special fall landing into the 2.3% hit; the
  wall stick (stick, hang still, jump up, let go, three times then no more, nothing in the open).
* `move_rules::the_nodachi_phase_2_fields_are_in_the_content_hash`; `scripted_moves::ground_says_whether_a_segment_meets_a_block_or_a_platform_top`;
  `state.rs` checksum cases `orb_clock`, `orb_age`, `projectile_charge`, `projectile_orb`, `projectile_held`, `projectile_orb_hits`;
  sim-content `the_nodachi_phase_2_fields_round_trip_and_are_checked` (round trip, older files unchanged, bad values refused).
* `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`; `pftool fuzz-rollback 300` and
  `pftool net-fuzz 100`: 0 mismatches each (300 runs, 165 exile fighters; 100 runs, 21 exile fighters).
* Other fighters: `content/base.pfc` changed only in the nodachi weapon; every other kit's tests unchanged and green.

## Estimates [E] (phase 2)

The beam's speeds (5.46 / 8.16 of the reference's units a frame) and its first hit's place (from the hitbox animation); the orbit
(radius 12, a turn a second); the extension in proportion to the charge; all orbs on a fighter striking together; a loose orb's flight
(the beam's uncharged speed, 60 frames); the barrier's capsule and the answers' centres (from the animations); the lerp's non-damage
fields read linearly; the break's push back; the barrier in the air with the body's physics; the slashes' landing box and the landing
in the special fall; the wall test (12.5 up, out to 32) and the wall jump's height (6.9 read as a height); the exile staying where it
stuck.

## Limits

The beam's 2% boxes for things that are not fighters and its passing through projectiles; orbs do not pop balloons; the wall stick's
camera-bound and wall-jump-bar rules; the up special's rotating boxes; one get-up attack slot; the grabs' aerial boxes; the wing's
training and stamina rules.

## Owner questions (phase 2)

In `../ROSTER_STATUS.md` ("Open for the owner from the Nodachi's phase 2"): the orbs' unpublished numbers, the barrier's places read
from the animation and its push, the wall stick jump's 6.9, the slashes' landing hit in both landings.

## For the art agent

Every move falls back to the shared archetypes until it has its own clip. Moves that need their own (reference frames; boxes in
`moves_nodachi.rs`):

* Phase 2 (sim 128): the side special (a snap of the fingers, 16-22 to 42-48: a thin dark beam; the orbs circling the hit fighter and
  flying into it on the strike: the view can draw `Projectile::orb` / `held`); the down special (a honeycomb barrier in front on 7-27,
  then a flurry of slashes in front, 38-53 unstruck, 7-22 struck, larger with a stronger catch; a stagger when it breaks); the down smash
  at an edge (the blade alone, 21-22); the slashes' landing; the forward air stuck in a wall (hanging by the blade), its jump (on 12) and
  letting go.
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

## Sim 130: up specials halt their starting momentum

Owner decision 97 (sim 130, branch `upspecial-momentum`): up specials halt the momentum they start with, as the reference does. The cause of the owner's "super-jump": the blade dash's script halved the sideways speed and zeroed the vertical on its frame 0, which never runs (a move's first frame in `attack()` is 1), so the 23-frame startup stall (0.9 of the speed kept a frame, a 0.0025 sink) carried a jump's whole rise: out of a full hop the dash rose 21.4 world units against 9.4 from a standstill, the eight slashes 22.5 against 5.7. The rule is now data on the move (`start_speed` 50, `start_speed_y` 0, applied before its first frame) and the dead line is gone. The slashes, reached by `goto`, inherit the stalled speed. Also checked: after its up special the exile acts only by landing (into its landing lag or the slashes' landing hit, decision 96) or catching a ledge; every button in the special fall does nothing (tested). Before, out of a jump the move rose higher than from a standstill at the same height; now the two match (`sim-core/tests/upspecial_momentum.rs`). The audit of every fighter and its sources: `docs/movesets/SHARED_MECHANICS.md`, "Starting momentum of specials".
