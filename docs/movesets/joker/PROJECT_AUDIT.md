# Project audit for the thirteenth moveset

The third of the second wave (owner decision 65). Names (owner decision 65): class and weapon **Dagger**, body and built-in fighter
**Phantom**, class index 12. Everything is 13.0.1. Reference data: `REFERENCE_DATA.md`; what was built: `IMPLEMENTATION_REPORT.md`.
Engine inventory: `../SHARED_MECHANICS.md`. The reference character, its moves and its partner are not named in the repo's text.

## Iconic moves: what must match exactly

* **The gauge and the summoned partner** (top priority). The gauge fills by itself (0.00925 a frame), from damage taken (1.3 a percent,
  x1.625 / x1.95 a stock or two behind, the player-count table) and much faster from hits on the guard (x6.7368 of the guard's 0.4 of the
  damage). Full, the partner comes for 1800 frames, nothing fills the gauge, each percent taken cuts 16 frames (10.2 with four fighters),
  a stun stops the clock, and a KO leaves 20 / 25 / 30 units. While it is out, every normal (but the pummel, throws, ledge and get-up
  attacks) gets the partner's own hit alongside the fighter's (the damages add) and the fighter's own boxes change ("awakened" values,
  kept to the attack's end), and every special becomes its powered version without spending the gauge.
* **The gun** (neutral special): the shot on 12 with three distance bands, the repeats, the dodges, the grounded jump, the aerial shots
  below and the spiral, each with the partner's extra bullets.
* **The guard** (down special): super armor that takes 0.4 of a hit and fills the gauge, held up to 120 frames, the backhand on letting go
  after a hit; with the partner the counter (x1.6, at least 12%) and the reflector (x1.6, x1.9 speed).
* **The curse bolt** (side special): the bolt and its eruption, the curse's damage over time, the partner's multi-hit bolt that also
  bursts on the ground.
* **The hook** (up special): the grounded pull-and-throw grab, the aerial hit, the tether to the ledge; with the partner the wings.
  **Phase 2** (the tether is a new system), except the wings (built now, the powered up special).

## How the moves flow (traced)

Input → `fighter::update` (the gauge's `tick`: the mode's clock, halted in a broken shield with `halts`, or the fill by itself) →
`ground` / `airborne` → `start_special` (a powered version through `limit_version`, spending the gauge unless it `keeps`) → `begin_attack`
(**`move_powered` latched** for a new attack, `withstood` cleared) → `attack()` (the `withstood` count, the scripts: `powered`,
`withstood`) → `combat::resolve_hits` (`active_hitboxes` filters by `Hitbox::mode` and `move_powered`; the best box of each **part** per
target; the fighter's own hits land, then the partner's) → `combat::apply_hit` (a move's armor window: `armor_damage` of the damage, its
`armor_hitlag`, `withstood`; then `gauge::on_damage`: the gain by stock gap, crowd and `gauge_gain`, or the mode's cut) →
`combat::update_projectiles` (a curse put on by a damaging hit; `Burst::on_shield`) → `items::update` → `combat::curse_tick` (the
damage over time) → `combat::check_ko` (`gauge::after_ko`: the units kept).

## What existed and was reused

* **E9 the gauge** (the champion's): the hashed `Fighter::gauge` / `gauge_mode`, `content::Gauge`, `gauge::add` / `tick` / `on_damage`,
  `Move::limit_version` (a special's powered version). Extended, not replaced (E53 below).
* Every normal is data (`ref_move`, rows, hit groups, `rehit`, set knockback and weight, `targets`, angle types, `aim`, `clanks` /
  `rebounds`, `shieldstun`, `charge_at`, `chain`).
* The guard: **E1** super armor; the stance: the Staff's counter + reflector routing (**E10c** `melee_only`, `detect_then`,
  `counter_strike`); the curse bolt: **E3e** bursts (phases, `rehit`, `on_ground` through a one-bounce arc); the gun: plain boxes and
  scripts (`goto`, `special_tap`, `stick_*`, `jump`, `grounded`), data motions; the wings: motions, `steer_*`, `helpless_after`,
  `fall_landing_lag`, `ledge_front_from`.

## The extensions built (all general, all defaulting to today's behaviour)

### E52: the partner's boxes (`Hitbox::mode`, `Hitbox::part`, `Fighter::move_powered`)

* `Hitbox::mode` (`MODE_ANY` default, `MODE_PLAIN`, `MODE_POWERED`): `combat::active_hitboxes` lets a box out only if the attack began
  with (or without) the gauge's powered mode. The hashed `Fighter::move_powered` is set by `begin_attack` for a new attack (a `goto` or a
  jab's `next` keeps it), so a move keeps the boxes it began with when the mode ends or starts during it (the reference's rule).
* `Hitbox::part` (0 default, 1): `resolve_hits` picks the best box of part 1 on each target separately (`partner[a][d]`) and lands it after
  the fighter's own (`strike`, the old per-hit code moved into a function), unless another attacker struck the target that frame, the
  fighter's own hit was caught by a counter, or the attacker rebounded. Both damages add; the second launch stands [E].
* The `powered` script register.

### E53: the gauge's partner rules (`content::Gauge`, `Move::gauge_gain`)

`per_frame`, `behind`, `crowd`, `mode_taken`, `ko_keep`, `keeps`, `halts` on the gauge (content fields `per_frame`, `behind_1/2`,
`crowd_3_*` / `crowd_4_*`, `mode_taken_2/3/4`, `ko_keep`, `ko_keep_2/3`, `keeps`, `halts`) and `Move::gauge_gain` (per mille). The
fighter count is `GameState::roster`'s, the stock gap the leader's stocks less the fighter's (a stock match). A hit taken while the mode
runs cuts the mode and fills nothing. `gauge::after_ko` (from `check_ko`) sets what a KO keeps. The champion's gauge is unchanged (all
defaults; `tests/gauge.rs` unchanged and green).

### E54: damage over time (`Move::curse`, the hashed `Fighter::curse_*`)

A curse on the projectile's move: a fighter whose percent a hit of that projectile (or its burst) raised takes `damage` (x the one-on-one
multiplier) every `every` frames for `frames`, credited to the projectile's owner (`combat::curse_tick`, after the items in `step`). Plus
`Burst::on_shield` (false: a shield takes the shot's hit with no burst).

### E55: the guard's armor (`Move::armor_damage`, `Move::armor_hitlag`, `Fighter::withstood`)

On a frame the move's armor covers, a hit deals `armor_damage` per mille of its damage; a withstood hit's hitlag is multiplied by
`armor_hitlag` for both fighters and starts the hashed `Fighter::withstood` (frames since, plus one, counted outside hitlag), which the
`withstood` register reads.

### Small ones

* `start_special` takes the body's numbers (for `Gauge::keeps`).
* A move with only a reflector (the reflection) is valid content.
* The fuzzers: `fuzz-rollback` plays the phantom with its partner out every fourth run (from 3); `net-fuzz` plays it every eighth (from 6)
  with its specials drilled.

## Move slots

`Ext0`-`Ext2` still hold the placeholder up special (the claws'); `Ext3`-`Ext7` the gun's forms, `Ext8` the powered bolt, `Ext9`-`Ext11` the
guard's hold, end and backhand, `Ext12`-`Ext14` the stance, its counterattack and reflection, `Ext15` the wings. Phase 2's hook needs the
three placeholder slots (ground hook, air hook, hook throw): all sixteen used.

## Not built (limits)

* The hook and its tether (phase 2: a tether to a ledge is a new system).
* The partner's arrival and leaving animations, and its longer rolls (unpublished: gaps 2-4).
* The gun's dodge staling and its 4-frame buffered delay; angle 365.
* The up smash's arm intangibility with the partner (an E19 window cannot be tied to the partner).
* The house limits: one get-up attack, the grabs' aerial boxes, the throws' bystander hit and thrower invincibility.
