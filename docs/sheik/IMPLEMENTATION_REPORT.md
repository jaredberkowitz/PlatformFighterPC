# Needles moveset: implementation report

The third moveset (in game: the **Rogue**, class 2, weapon **Needles**), modelled mechanically on the reference game's Sheik. Built in
three phases on branch `phase7-match-rules` (sim versions 49 to 51). The audit is `PROJECT_AUDIT.md`; every number and its label
(verified [R], derived [D], estimated [E]) is in `REFERENCE_DATA.md`.

## What was built

| Area | Moves |
| --- | --- |
| Body | `FighterParams::rogue`: weight 78, the reference's walk, dash, run, air speed and acceleration, gravity, fall speeds, jump heights and air dodges; rolls and spot dodge |
| Priority moves (phase 1) | forward, up and down tilt; neutral, forward, back, up and down air, frame for frame |
| Rest of the normals (phase 2) | jab 1 and 2, rapid jab and finisher, the earlier game's dash attack, the forward, up and down smash, grabs, pummel, the four throws |
| Specials (phase 3) | Needle Storm (charge, store, throw), a pulling line of our own in place of Burst Grenade, Vanish, Bouncing Fish |
| Kept from the claws kit | ledge attack and get-up attack frames (their damage is the reference's) |

## Systems added or changed

Reused: the move format (`ref_move` rows in the reference's frame numbering), hit groups, rehit, projectiles, move scripts and their
`goto`, autocancel windows, landing lag, set knockback scaling, the content format and validation, the animation library and trails.

Added (all general, all hashed, all in the content format with defaults that keep older content unchanged):

* **Multi-hit engine** (sim 49): eight hit groups per move; a rehit that re-arms chosen groups only; per-hitbox targets (grounded or
  aerial only), set knockback, extra hitstun and SDI multiplier; the autolink angle 367 (carried along at the attacker's velocity plus a
  pull toward the hitbox, capped, no tumble); landing hits (an aerial's own landing move).
* **Reference rules** (sim 50): the Sakurai angle (0 on the ground up to 60 knockback, 38 from 88 and in the air); traction slows
  grounded slides; a non-tumble hitstun lands on its feet (no knockdown or tech after a drag-down).
* **Pull angle 368** (sim 51, ours): the victim is drawn to just in front of the attacker while its hitstun lasts.
* **`link_ahead`** (sim 51, ours): an autolink's pull can aim in front of its hitbox, so the up air leaves its target in front.
* **Stored charge** for scripts (`store(n)`, `stored`), kept on the fighter until a KO.
* View: the class in the roster, menus and HUD; kick clips; teal trails fitted to every move's hitboxes; needles and the line drawn as
  such; the fighter hidden during Vanish's teleport. Bridge: projectile slots carry velocity, class, move and owner.

## Verified, derived and estimated

Verified against the published tables: every move's active frames, FAF, landing lag and autocancel windows; damage, angles, base and
growth knockback, set knockback, hitlag and SDI multipliers, hitstun modifiers, grounded and aerial targets; the body's attributes
(short and full hop air times within a frame of the reference). Derived: radii (converted units), the back throw's angle seen from
behind, the down smash's charge frame, the rapid jab's press count and loop length. Estimated: hitbox positions throughout (the
reference attaches hitboxes to bones of its own skeleton), roll distance, the ledge and get-up attack timing, Vanish's startup hop, and
everything about the pulling line (it is ours by request).

## Tests

* `sim-core/tests/needles.rs` (33 tests): frame data tables for every move, the up air's loop, drag-down, fall cap, SDI, landing in front
  and standing, the down air's stall, dive and landing hit, the up tilt's two hits at any percent, the jab chain, rapid jab and finisher,
  both two-hit smashes, the dash attack's pop-up, throw damage, needle counts by charge, storing and spending needles, needles hitting,
  the line reeling in from the ground and the air with frame advantage, Vanish's direction, intangibility, explosion and helplessness,
  Bouncing Fish's kick, bounce and landing lag.
* `sim-core/tests/multihit.rs` (the engine features), `scripted_moves.rs` (the stored charge), and the existing combat, knockdown and
  hit-feel tests updated for the reference's Sakurai angle and standing landings.
* All 44 Rust suites, clippy, `net-fuzz` (200 runs, 0 mismatches) and the 19 Godot tests pass; contact sheets of the up air, forward
  tilt, forward smash, jab and all four specials were checked by eye.

## Limitations and next steps

* **Animations** are placeholders borrowed from the shared library and the claws (kicks, the fair chop, a lunge for the dash attack);
  the Needles clips are the next step, then refitting the trails to them.
* Needle Storm: the charge cancel's 4 frames of lag before the chosen action are not modelled.
* Bouncing Fish: no consecutive kicks after a bounce, and no wall bounces. Vanish: no reappearance wind box; the explosion moves with the
  fighter for its 3 frames instead of staying behind.
* Up smash arm intangibility and the back roll's separate intangibility are not modelled.
* The rapid jab cannot see presses made during hitlag, so it asks for two presses rather than three.
