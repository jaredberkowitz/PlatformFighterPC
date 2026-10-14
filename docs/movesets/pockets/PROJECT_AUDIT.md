# Project audit for the eleventh moveset (reference: the reference game's Villager)

The first of the second wave (owner decision 65: Villager, Diddy Kong, Joker, Sephiroth). Names (owner decision 65): class and weapon
**Pockets**, body and built-in fighter **Settler**, class index 10. Everything is 13.0.1 (decision 68) except the jab, which is the previous
reference version's (1.1.x) on that version's hit rules (decisions 66-67). Reference data: `REFERENCE_DATA.md`; what was built:
`IMPLEMENTATION_REPORT.md`. Engine inventory: `../SHARED_MECHANICS.md`.

## Iconic moves: what must match exactly

* **The jab** (owner's choice: the previous version's): two 3% punches at 60 / 80 degrees that pop the foe up and away, an endless
  jab with no finisher, continuable from 7 / 8. Needs E40 (below) so the hits follow that version's rules.
* **The balloons** (up special): the flight (flap, sink, steer), the helium tank and its slow refill (one balloon when low), the
  balloons as their own hurtbox that pops one at a time, the special fall and a hop when the last pops, 20 / 30 frames of landing lag.
  Needs E41 (below). The reference publishes almost none of the numbers (gaps 4-7): **an owner question**.
* **The rocket** (side special): set down, flies from frame 52 with 7%, slows with 5%, bursts at the end or on a wall (12%), used up on
  a fighter; one at a time; ridden when held (15.4 / 11 / 18%), the stick's speed, the bounce off a hit, helpless otherwise. Built from
  existing parts (a projectile with a script, E3a phases, E3d's one-at-a-time, E3e's burst) plus `Burst::on_hit`. Its speeds are
  unpublished (gaps 1-3).
* The turnips' random counts (up and down air), the slingshot pellets, the flowerpot and the bowling ball (projectile normals, the
  ball's own 1.2x charge), the firework, the shovel's bury.

## How the moves flow (traced)

Input → `fighter::update` → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` / `start_special` → `begin_attack` (move
frame 0) → `attack()` every frame (hitboxes, `next` / `next_from`, the move's script, motions, air physics, landing) →
`combat::resolve_hits` (choose, clank, `apply_hit`) → `grab::update` → `combat::update_projectiles` / `spawn_projectiles` →
`items::update`. New in sim 113: right after the fighters' updates, `fighter::draw_version` (E39) and `balloons::update` (E41).

## What existed and was reused

Every normal is data (`ref_move`, `row`, hit groups, set knockback and weight, `hitstun_add`, `targets`, `trip`, angle types,
`part_intangible`, `charge_at` / `charge_bonus`, `EFFECT_BURY` (E5), `rebounds` (E3g), `clanks` (E3c)). The projectile normals use
`ProjectileSpawn` with `phases` (E3a), `ProjectileArc` gravity and `lodge` (E3b: a touch of the floor ends them) and `ChargedShot` (E35).
The jab chain uses `next` / `next_from` and a script for the held rule. The rocket uses a projectile script, `Tool` (E3d: kind 4, one
out), `Burst` (E3e), scripts with `goto`, the `hit`, `wall`, `jump`, `owned` registers. The body uses `FighterParams` (sim 54 ground
numbers, E4 item throws).

## The extensions built (all general, all defaulting to today's behaviour)

### E40: the previous reference version's hit rules (owner decision 67)

`Hitbox::rules` (`RULES_CURRENT` 0 default, `RULES_PREVIOUS` 1; content `rules previous`), the ruleset's `prev_damage_mult` (1),
`prev_hitlag_per_damage` (0.3846154), `prev_hitlag_base` (5), `prev_rage_max` (0.15), `prev_tumble_hitstun` (1), and hashed
`Fighter::launch_older`. In `combat::apply_hit` a previous-rules hit adds its damage x `prev_damage_mult` (shields too), freezes with the
old hitlag formula (a crouch shortening only the victim's), uses the old rage and adds a frame of hitstun to a tumble; in
`fighter::apply_launch` / `launch_fall` its launch gets no speed-up, the gravity boost at every angle and the victim's own fall speed
for a vertical launch. Only the Settler's jab sets it (decision 67: others on the owner's word). Why per hitbox: the owner's rule names
moves, and a move's boxes are where every other hit property lives; a per-move flag would be the same with less reach.

### E39: random versions

`Move::versions: Vec<(chance, slot)>` (content `version { chance move }`): on a move's first frame the game's random generator (the
hashed `GameState::rng`) picks a version slot, the move itself with what is left of 100. The up and down air's 3-turnip versions are the
moves' own slots, 2 and 1 turnips `ext0`-`ext3`. General: any move drawn at random as it starts.

### E41: balloons

`Move::balloons: Option<Balloons>` (`count hp x y radius fuel refill one_below pop_vy`), hashed `Fighter::balloons`, `balloon_hp`,
`helium_used`, `sim-core/src/balloons.rs`, the `balloons` script register. A hit that reaches the balloons' circle and not the body pops
one (melee in `resolve_hits`, shots in `update_projectiles`); the last pops into the special fall with a hop; the tank empties a frame of
flight a frame and refills out of the flight. The flight's physics are the move's air physics plus its script.

### Small ones

`ChargedShot::power_to` (a charged shot's every hit, its phases too, by the charge: the bowling ball's 1.2x), `Burst::on_hit` (false: a
burst only on a wall / the ground / its end: the rocket), `Item::angle` (hashed, the arrow's direction as it lodged) with the bridge's
`item_angle(i)` (the art director's request), `trig::degrees_of`, `fighter::stand_or_fall`, a motion-only move counted as a real move.

## Not built (phase 2 and 3, or limits)

* Pocket and Timber: built in phase 2 (sim 115: E42 the pocket, E43 placed pieces; `IMPLEMENTATION_REPORT.md`). Also there: the
  jab cancel (`Move::cancel_from`, owner decision 70). Not built: the tree's fruit, a pocketed teammate's shot (no teams).
* `direct=f` (the firework's hits leave its user unfrozen): not modelled (as the other ports).
* The rocket hit by an attack (it bursts), reflected while ridden: not modelled.
* The down smash's `noprone`; the face-down and trip get-up attacks (one get-up slot); the grabs' aerial-target boxes; the throws'
  invincibility and the thrown fighter's hit on bystanders (house limits).
