# Testing standards for moveset ports

How a ported fighter is tested before it is called done. These are the rules the Needles port (`docs/sheik/`) already follows, written
down so every port is held to the same bar. Unit tests prove behaviour; animations never do.

## Where tests live

* One integration test file per fighter in `sim-core/tests/` (models: `needles.rs`, `reference_moves.rs`, `reference_movement.rs`,
  `multihit.rs`, `kits.rs`). Name it after the weapon (for example `cargo.rs`), never after the reference character.
* Use the shared harness `sim-core/tests/common/mod.rs` (`Sim`, `inp`, `fx`, `Sim::duel`, `Sim::alone`, `stand`, `tick`, `tick2`). It turns
  stale moves off so listed damage can be checked; a test that needs staling turns it back on.
* A shared-system change gets its own test in the topic file it belongs to (`multihit.rs`, `scripted_moves.rs`, `grabs.rs`,
  `shields.rs`, ...), not only in the fighter's file.
* Test names say the behaviour in plain words (`up_air_loops_three_hits_then_lands_its_final_hit_once`), so a failing name explains itself.

## What every port must have

### 1. Frame data (one table test, every move)

A table of `(move, reference active windows [first, last], FAF, landing lag, autocancel before / from)` in the reference's numbering, checked
two ways: against the data (every hitbox lies in a listed window, every window has a hitbox, `total_frames == FAF - 2`), and by running the
move in the sim (the first frame a hitbox can touch a target is the reference's first active frame; control returns on FAF; landing inside
and outside the autocancel windows gives the plain and the move's landing lag). Every number in the table is an [R] value from
`REFERENCE_DATA.md`; a [D] or [E] value in a table is marked so in a comment.

### 2. Hit properties

* Damage per hit (times `Ruleset::damage_mult`, the reference's 1.2 one-on-one multiplier), angle, BK, KS, FKV, hitlag and SDI
  multipliers, grounded/aerial targets.
* Sweet and sour spots: which hitbox wins where both reach (priority), and the damage of each.
* Multi-hit moves: how many hits land, on which frames, the rehit interval, that the loop is re-armed and the final hit lands once, that a
  target out of reach takes no phantom hits, and that the final hit launches (the loop does not).
* Launches that matter for the kit (combo throws that never kill, a kill throw, a meteor) checked by where the victim goes.

### 3. Specials

Cover what the special *does*, not only its first hit:

* charge: grows with time, persists when stored (shield/roll/jump), released early and at full, lost on KO if the reference loses it;
* armor and intangibility windows: a hit inside the window does what the reference says, one outside does not;
* projectiles: spawn frame and position, speed, life, falloff, what destroys them (hit, wall, ground, owner), how many may exist, clanks,
  reflection;
* reflect and counter: what is caught, the answer, the timing window, facing;
* grabs and held states: who can be caught (grounded, airborne, shielding), how the hold ends (throw, mash out, timer, hit, KO, offstage),
  where both fighters end up;
* ground versus air variants, and what happens crossing between them (running off an edge, landing mid-move);
* recovery moves: height and distance reached (within a stated tolerance of the reference, from the [R] or [D] value), ledge grab window,
  helpless and landing lag after.

### 4. Movement

Short hop, full hop and air jump air times (frames to land, also with a fast fall), against the reference's attributes (model:
`needles::the_rogues_jumps_have_the_references_air_times`); ground numbers in `ground_attributes::every_fighter_has_its_reference_characters_ground_numbers`
(add the new body to it); initial dash length.

### 5. Determinism

* Every new `GameState` or `Fighter` field is in its `hash_into`, with a case in the checksum tests in `sim-core/src/state.rs` that fails
  if the field is dropped from the hash.
* Stateful specials (charge, gauge, held target, projectile lifecycle) get a resimulation check: save the state mid-special, run on,
  restore, run the same inputs again, same checksum (model: `scripted_moves::scripted_content_is_deterministic_and_rolls_back`).
* `pftool fuzz-rollback` and `pftool net-fuzz` at zero mismatches; the new class should appear in the fuzz's random rosters.

### 6. Regressions

* `cargo test --workspace` green, including every other class's kit tests unchanged.
* `content/base.pfc` re-exported and equal to the builders (`sim-content/tests/bundle.rs::the_hand_coded_roster_round_trips_exactly`).
* A change to a shared system must leave the existing fighters' behaviour identical; if it cannot (a rule brought to the reference's),
  that is said in the report and in `STATUS.md`, with the tests that changed and why.
* Every recipe stays valid (`sim-content/tests/recipes.rs::every_recipe_is_valid_content`); the new class is selectable and resettable.

## When a unit test cannot prove it

Write a reproducible training-mode scenario in the fighter's `IMPLEMENTATION_REPORT.md`: stage, positions, percents, inputs frame by frame
(or the demo flag), what to look for in the hitbox view (F3) and the overlay. Never rely on watching an animation to say a move is right.

## Before reporting done

`cargo fmt --all`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and the fuzzers when state or the wire
format changed. The report lists only tests that were actually run, with their counts.
