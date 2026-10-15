# Project audit for the fourteenth moveset

The fourth of the second wave (owner decision 65). Names (owner decision 65): class and weapon **Nodachi**, body and built-in fighter
**Exile**, class index 13. Everything is 13.0.1. Reference data: `REFERENCE_DATA.md`; what was built: `IMPLEMENTATION_REPORT.md`. Engine
inventory: `../SHARED_MECHANICS.md`. The reference character and its moves are not named in the repo's text.

## Iconic moves: what must match exactly

* **The wing** (top priority, the comeback mechanic): when it comes (a start table by stock difference and percent difference, read
  after each hit), what it gives (x1.3 damage that moves the KO percent only by the extra damage, eleven body multipliers, a third jump,
  20% damage armor on the three smash attacks from their charge frame to their hit), how it goes (points spent by base damage dealt and
  by KOs, the KO moving the points by the new stock difference) and that it does not come back until a KO.
* **The long blade**: reach (tilts and smashes 30 reference units ahead), the sweet spots (slashes in the middle, stabs at the tip).
* **The blade dash and the eight slashes** (up special): the hold on 18 that picks the form, the aim on 19 / 31, the dash's one hit,
  the eight hits on 36-50 dragging the target along, the ground versions stopping at edges, ledge rules, landing lags 27 / 56.
* **The flare** (neutral special): the release from 20, the three levels by hold time (31 frames, full charge), each fireball and burst
  (the vortex multi-hits, the giant flare's three rings by distance).
* The shadow orbs (side special): the beam's charge (from frame 9, up to 26 frames), its two capsules growing in damage, the orbs it
  puts on a fighter by charge (one, two, three; twice from a reflected beam; five at most), the clock (4 seconds, lengthened by later
  hits, never past 7.5), the circling orbs' bystander hits, the strike away from the owner (phase 2, sim 128).
* The barrier (down special): the catch in front only, the answer whether struck or not, its numbers read by the caught damage, no
  hitlag for the exile, the break above 25% (phase 2, sim 128).

## How the moves flow (traced)

Input → `fighter::update` → `ground` / `airborne` → `start_special` → `begin_attack` (**`move_powered` latched** for a new attack: the
gauge's mode or **a wing out**) → `attack()` (the charge frame; the scripts: the jab's held rule, the down air's stall and plunge, the
blade dash's aim and travel, the flare's release levels; the motion branch with **`edge_stop`** clamping the step to the platform before
moving) → `combat::resolve_hits` → `strike` → `combat::apply_hit` (**the wing's power** on the hit's damage after staleness; a move's
armor only where its **`armor_mode`** matches `move_powered`; after the percent rises **`wing::on_hurt`** (the KO credit, the wing's
start) and **`wing::on_dealt`** (its points)) → `combat::update_projectiles` (**rings**: every phase sharing the active start is out, the
first that reaches a fighter hits it) → `combat::check_ko` (**`wing::before_ko` / `after_ko`**: the credited fighter's points). The body
each frame: `step` uses **`FighterParams::moving`** (the gauge's mode and the wing's multipliers and jumps).

## What existed and was reused

* Every normal is data (`ref_move`, rows, hit groups, angle types, `aim`, set knockback and weight, `targets`, `clanks` / `rebounds`,
  shield damage, `charge_at`, `chain`, `landing_move`, **E34 position vectors** for the up tilt's lift, **E19** for its upper body).
* The up special: the brawler's aimed dash pattern (`fire_aim.script`: stick read on one frame, set velocities), `helpless_after`,
  `fall_landing_lag`, `ledge_front_from` / `ledge_back_from`, `rehit` for hits 2-7, the autolink angle for the drag.
* The flare: script `goto` between a charge move and release moves, projectiles with phases by age and **E3e bursts** (`on_wall`,
  `on_expire`), **E28 the vortex** for the bursts' multi-hits.
* The armor: **E1** damage armor.

## The extensions built (all general, all defaulting to today's behaviour)

### E57: a comeback form (`FighterParams::wing`, `content::Wing`, `sim-core/src/wing.rs`)

Traced first: nothing tracked a stock-and-percent comeback rule. The gauge (E9 / E53) is the closest piece, but it fills from damage and
runs on a timer; the wing starts by a percent threshold table and ends by points, so it is its own block beside the gauge, reusing the
gauge's way of applying multipliers (a copy of the body per frame) and `Fighter::move_powered` for the armor.

* `Wing { rows, points, per_dealt, ko_points, power, jumps, walk .. fall }`; `WingRow` (up to 8 points of a broken line by percent
  difference, jumps where two points share a difference); `WingRow::threshold`.
* Hashed `Fighter::wing_points` (out while above 0), `wing_spent` (lost since the last KO), `hit_by` (the KO credit).
* `wing::on_hurt` (from `apply_hit`, `pummel`, `curse_tick`), `on_dealt` (hits on fighters' bodies), `before_ko` / `after_ko` (from
  `check_ko`); `FighterParams::winged` and `moving`; the power in `apply_hit` and `pummel` (not items).
* `Move::armor_mode` (any / plain / powered): the move's armor by the attack's powered start; `begin_attack` latches a wing out as
  powered.

### E58: rings (`moves::rings_at`, `MAX_PHASES` 12)

A projectile had one hit at a time (phases replace one another by age). The giant flare's burst has three concentric boxes at once (the
lowest id winning). Phases sharing a start are now rings: all out together, in their order, the first that reaches a fighter hitting it;
they share a hit group (validated). Older content has no shared starts, so nothing else changed. The phase limit rose from 8 to 12 (three
stages of three rings).

### Small ones

* `Move::edge_stop`: a ground motion (data or script) and the ground slide stop at the platform's edge (the grounded up special).
* Validation: a follow-up slot (`ext*`) may be a move with no hitboxes (only time: the blade dash's end in the air, the flare's cancel).
* The fuzzers: `fuzz-rollback` plays the exile every fourth run from 1 with its wing out at 70% and its specials drilled; `net-fuzz` every
  eighth run from 0 as the joiner, specials drilled.

## Phase 2 (sim 128): traced and built

Traced first: the curse (E54) puts damage over time on a fighter a projectile hits, but has no orbs (nothing to see, reflect or hit
bystanders); the pocket (E42) holds a projectile frozen for a fighter, not around one. Neither stores a clock on the target. The
counter (E10c) catches on a bubble and answers with a move, but its answer deals caught damage x percent from the move's own boxes;
it had no break and the countering fighter always froze. Nothing tested the ground ahead of a move. A special fall always landed into
plain landing lag.

### E59: orbs (`Move::orbs`, `moves::Orbs`, `sim-core/src/orbs.rs`; owner decision 92)

The orbs are projectiles of the beam's move (the pool, `MAX_PROJECTILES` 16) marked `Projectile::orb` and held by a fighter
(`Projectile::held`); the fighter keeps the clock (`Fighter::orb_clock`, `orb_age`). `combat::update_projectiles` calls `orbs::place`
when the beam hits (not a shield, not a counter's catch) and hands circling and striking orbs to `orbs::update_orb` (their own motion
and hits: other fighters only while circling, rehit and a count of hits; away from the owner when striking; pocket and reflectors
while striking); a reflected or pocketed orb is `ORB_LOOSE` and flies through the ordinary path with `orbs::loose_spec`.
`orbs::tick` (in `step` before the projectiles) runs the clocks; `check_ko` calls `orbs::clear_target`. The beam's charge is kept on
the shot (`Projectile::charge`). Kept to what the exile needs: no options beyond the reference's rules.

### The barrier

* `Counter::break_above` / `break_move` (a caught hit above it deals nothing and starts the stagger) and `Counter::still` (no hitlag
  for the countering fighter), in `combat::catch_with_counter`.
* `Move::lerp` (`moves::Lerp`): the move's projectile is fired with `Fighter::counter_damage` (in `Projectile::damage`) and its phases
  are read between the written hits and the paired ones by it (`combat::lerped`).
* The answers are standing projectiles (the reference's: reflectable, absorbable), piercing with rehit 5.

### Small ones

* `ChargedShot::every_hit`: the charged damage on the shot's rings too (the beam's two capsules).
* A projectile's rings keep to their `targets` (the beam's aerial-only capsule); no older projectile had one.
* `Move::probe` (`moves::GroundProbe`, `collision::segment_meets_ground`): the down smash's ground test.
* `Move::fall_landing_move`: the special fall after the eight slashes lands into their landing hit.
* The script function `ground(forward, up, forward2, up2)`: the forward air's wall test.

## Move slots

`Ext0` the down air's landing hit, `Ext1` the eight slashes, `Ext2` the blade dash's end in the air, `Ext3`-`Ext5` the flare, bigger
flare and giant flare, `Ext6` the flare's cancel; phase 2: `Ext7` the barrier's struck answer, `Ext8` its break, `Ext9` the down smash
without ground contact, `Ext10` the eight slashes' landing, `Ext11`-`Ext13` the wall stick, its jump and its letting go. Two left
(`Ext14`, `Ext15`). (The script slot numbers skip `PivotGrab`, 38: `Ext10` is 39.)

## Not built (limits)

* The beam's 2% boxes for things that are not fighters, and its passing through other projectiles (no projectile-projectile
  collision).
* Orbs do not pop balloons (E41); a circling orb is never reflected or pocketed (the reference's rule) and the orbs on one fighter
  strike together [E].
* The wall stick's camera-bounds and wall-jump-bar rules (no such stage data), its "jostle distance".
* The up special's boxes turning with the dash (the reference's rotating root): here they stay ahead of the body.
* The wing's training and stamina rules (no such modes in the sim).
* The house limits: one get-up attack, the grabs' aerial boxes, the throws' bystander hit and thrower invincibility.
