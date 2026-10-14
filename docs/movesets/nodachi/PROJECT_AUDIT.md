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
* The shadow orbs (side special) and the barrier (down special): phase 2.

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

## Move slots

`Ext0` the down air's landing hit, `Ext1` the eight slashes, `Ext2` the blade dash's end in the air, `Ext3`-`Ext5` the flare, bigger
flare and giant flare, `Ext6` the flare's cancel. Ten left for phase 2 (the shadow orbs, the barrier's answers and miss, the down smash
without ground contact, the forward air's wall stick).

## Not built (limits)

* The shadow orbs (side special) and the barrier (down special): phase 2. The orbs need a new general mechanic (projectiles held by a
  target, orbiting and striking on a timer, reflectable as they strike, hitting bystanders twice); the barrier needs a counter answer
  whose damage and size are interpolated on the struck hit's damage, that answers unstruck after a delay, and that breaks above 25%.
* The down smash without ground contact (needs a ground test at the blade's tip), the forward air's wall stick, the eight slashes'
  landing hit.
* The up special's boxes turning with the dash (the reference's rotating root): here they stay ahead of the body.
* The wing's training and stamina rules (no such modes in the sim).
* The house limits: one get-up attack, the grabs' aerial boxes, the throws' bystander hit and thrower invincibility.
