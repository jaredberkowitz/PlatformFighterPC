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
  Built in phase 2 (sim 123, the tether E56 below); the wings in phase 1.

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

### E56 (phase 2, sim 123): the tether (`Move::tether`, `FighterState::Tether`) and blockable grab boxes (`Hitbox::blockable`)

Traced first: no tether existed. The ledge code had three paths: `fighter::ledge_in_reach` / `collision::find_ledge` (a body's own ledge
grab box near the fighter), a move's `ledge_front_from` / `ledge_back_from` (up specials catching ledges with that same box), and
`step`'s phase 2 (who gets a contested ledge, trumping the holder). None reaches a ledge far above, and none hangs a fighter off a ledge
without holding it. Built, all general and defaulted:

* `Move::tether: Option<Tether { from to ahead behind up down hang reel drag limit }>`: on the move's frames, airborne, a free ledge in a
  box of reach (by the fighter's facing) is caught (`fighter::tether_catch`, reading `scripting::World::ledges`, the ledge holders as the
  frame began). The fighter enters the new `FighterState::Tether` (the move that caught stays in `move_id`, its numbers read from there):
  `hang` frames of damped drift (`drag` per mille a frame), then a straight `reel` to the ledge's hanging point, then a ledge request that
  `step` grants as any grab (so the body's grab limit, intangibility and the contested rule all apply). The stick held down lets go; a
  ledge held by someone when the reel ends throws the fighter off with the trump hop. Hashed `Fighter::tether_ledge`, `tether_count`
  (reset by landing or getting up, not by a hit: `limit` catches), `tether_from`. The tether never holds the ledge itself.
* `Hitbox::blockable` (grab boxes only): a raised shield or a reflector field the box touches stops that group for the rest of the move
  (`combat::resolve_hits`, with `reflector_touching`), and `grab::can_catch` misses fighters in hitstun.

### Phase 2's other general pieces (sim 123)

* **Angle 365** (`ANGLE_HALF_LINK`, `fighter::half_launch`, `LINK_HALF`): half the attacker's momentum, whatever the knockback.
* **Dodges in a move** (`Move::dodge_stale`): staled like a roll (`stale_dodge` on the move's first frame, `stale_shift` on its
  intangibility; the staleness does not wear off during it).
* **A part intangibility window's mode** (`PartIntangible::mode`, read with `Fighter::move_powered`): the up smash's arm only with the
  partner.
* **Damage dealt** (owner decision 80): the existing `Gauge::per_dealt` (the champion's) set to 1.3 for the phantom; a curse's ticks
  now go through `gauge::on_damage_over_time` (taken, not dealt).

### Small ones

* `start_special` takes the body's numbers (for `Gauge::keeps`).
* A move with only a reflector (the reflection) is valid content.
* The fuzzers: `fuzz-rollback` plays the phantom with its partner out every fourth run (from 3); `net-fuzz` plays it every eighth (from 6)
  with its specials drilled.

## Move slots

`Ext0` the hook in the air (with its rise), `Ext1` the hook's pull and throw, `Ext2` the hook in the air without the rise (phase 2);
`Ext3`-`Ext7` the gun's forms, `Ext8` the powered bolt, `Ext9`-`Ext11` the guard's hold, end and backhand, `Ext12`-`Ext14` the stance, its
counterattack and reflection, `Ext15` the wings. All sixteen used.

## Not built (limits)

* The partner's arrival and leaving animations (owner decision 81: kept instant) and its longer rolls (unpublished: gap 4).
* The gun's dodges are not stretched in length by staleness; the buffered delay only out of a shot, not between dodges.
* The hook: the stick's tilt of the rise; the head's travel along the rope (held at the far end); the tether's swing and
  `fighter_offset`; no reel-in by a button.
* The house limits: one get-up attack, the grabs' aerial boxes, the throws' bystander hit and thrower invincibility.
