# Project audit for the twelfth moveset (reference: the reference game's Diddy Kong)

The second of the second wave (owner decision 65: Villager, Diddy Kong, Joker, Sephiroth). Names (owner decision 65): class and weapon
**Popgun**, body and built-in fighter **Scamp**, class index 11. Everything is 13.0.1 (decision 68 applies; the older jab rule is not used
here). Reference data: `REFERENCE_DATA.md`; what was built: `IMPLEMENTATION_REPORT.md`. Engine inventory: `../SHARED_MECHANICS.md`.

## Iconic moves: what must match exactly

* **The banana peel** (down special): an item anyone can pick up and throw; tossed over the shoulder at its own angle and speed (a smash
  input faster); it trips: a thrown peel always trips a standing fighter (a hard trip, with hitlag) and hits an airborne one; a lying
  peel trips whoever walks into it but its last thrower (a soft trip, no hitlag, shields no help); two uses (hits, trips and throws); one
  out at a time and 85 frames between tosses. The trip itself is E20's; the rest needed E44, E45 and E47 (below).
* **The peanut popgun** (neutral special): the uncharged shot on 16, the charge to 94 fired on release, the explosion on 97 if held, the
  peanut's damage by the charge (4.3% to 13.8%), faster and flatter with it.
* **The rocket barrels** (up special): the charge (7 to 67), the launch speed by the charge (3.1 + 0.019 a frame, x1.05 in the air), the
  stick tilting it while charging and steering it while flying, the three hit phases, the special fall and 30 frames of landing lag, the
  crash into a wall, and the barrels flying off a hit wearer to burst on the stage. Needed E48 and two projectile registers.
* **The monkey flip** (side special): the leap's command grab latching onto the face (scratching, the attack's scratch-and-throw, the jump
  off the head), the kick on a second press, ground and air versions. Built in phase 2 (sim 119, E49; its latch numbers are
  largely unpublished, so several are [E]: see the report).

## How the moves flow (traced)

Input → `fighter::update` (the new `pull_wait` counts down here) → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` /
`start_special` (`special_flick` remembered) → `begin_attack` → `attack()` every frame (hitboxes, `next` / `next_from`, the new `Move::toss`
asking for `items::THROW_MOVE`, `Move::pull`, the move's script, motions, air physics, landing) → `combat::resolve_hits` →
`combat::apply_hit` (the new slip before the trip roll; the new `Move::shed` firing its projectile through `combat::fire`) →
`grab::update` → `combat::update_projectiles` (projectile scripts with the new `owner_stick_*`) / `spawn_projectiles` → `items::update`
(`serve` releasing a move's toss, then each item's tick, then the new `traps`, then `knock_about`).

## What existed and was reused

Every normal is data (`ref_move`, `row`, hit groups, set knockback and weight, `targets`, `trip`, angle types, `aim`, `rebounds`, `clanks`,
`charge_at`). The jab chain uses `next` / `next_from` / `late_next` (decision 58) and a one-line held-repeat script. The banana is an E4 item
(`ItemSpec`, `Move::pull`, the universal pick-up and throws, `owns(kind)`, `holding`), tripping through E20 (`fighter::trip`, the ruleset's
trip timings). The popgun is a scripted draw with `set_charge` / `keep_charge` and an E35 charge-scaled shot (as the Buckler's bow); its
explosion a plain hit. The rocket uses scripts (`set_vel`, `clamp`, `sin` / `cos`, `charge`, `wall`, `grounded`), `helpless_after` with its
own landing lag, ledge windows, an E3e burst for the barrels (`on_ground` with a one-bounce arc, `on_wall`, `on_expire`, `on_hit` false) and
`combat::fire` (E43). The body uses `FighterParams` (sim 54 ground numbers, `air_jump_drift` from decision 59, E4 item throws).

## The extensions built (all general, all defaulting to today's behaviour)

### E44: slip hits

`Hitbox::slip` (`SLIP_NONE` default, `SLIP_HARD`, `SLIP_SOFT`; content `slip hard|soft`), the ruleset's `slip_lag` (30) and `slip_max`
(90), hashed `Fighter::trip_soft`. In `combat::apply_hit`, before the trip roll, a slip hit on a grounded fighter not lying or sitting
always trips it (`fighter::trip(.., soft)`), whatever the launch; a soft one with no hitlag; `knockdown` reads the soft timings. Why a hit
field: the reference's slip is a hitbox effect (`effect=Slip`), on the peel's thrown and lying hits (and other characters' moves: a
waft, a cape in older games), so it belongs where every other hit property is.

### E45: traps (lying items that trip)

`ItemSpec::trap: Option<Trap { hit, kick_x, kick_y }>` (content `trap { kick_x kick_y hitbox { ... } }`), with `ItemSpec::uses` (hashed
`Item::used`) and `ItemSpec::damage_add`. `items::traps` runs after the items move: a lying item's trap hit touching a grounded fighter's
lowest hurtbox circle (not its last thrower; not lying, sitting, in hitstun, held or intangible; a shield no help) trips it by the hit's
slip; one use spent, the item kicked away. Uses: a fighter its thrown hit reaches (not a shield), a fighter its trap trips and a fighter's
throw each spend one; spent, the item is gone (at once on a hit or trip, at the end of its flight after a throw). `damage_add` is the
reference's base damage modifier, which a drop does not halve (the peel's `1 + 1.7 x speed`). Why on the item: the reference's lying
peel is the same item with its own hit; a trap is a property of the item, not of a move or a fighter.

### E47: a move's toss

`Move::toss: Option<Toss { frame, again, angle, speed, smash_speed, wait }>` (content `toss { ... }`), `items::THROW_MOVE`, hashed
`Fighter::pull_wait`, the `pull_wait` script register. On `frame` (and `again`, for an item held as the move began) the item in hand is
thrown at the move's own angle and speed (`smash_speed` for a smash-style special input) times the toss strength with the item's
`throw_mul`, its hit a drop's; `frame` starts `pull_wait` at `wait` (the reference's 85 frames before the next peel). Why: the peel's toss
is not one of the universal item throws (its own angle, speeds and drop damage), and the reference's other item-making specials throw
the same way.

### E48: a move that sheds a projectile when hit

`Move::shed: Option<u8>` (content `shed <move>`): a hit landing on the fighter during the move (not a push, not withstood by armor) fires
that move's projectile at the fighter (`combat::fire`), credited to it; a flinch also ends the move (the special fall in the air). Used by
the rocket barrels (flying off the hit wearer). Why a move field: it is what the move does when interrupted, the same shape as
`landing_move` or `catch_move`.

### Small ones

The projectile registers `owner_stick_x` / `owner_stick_y` (the barrels steered by their owner a while), `scripting::Surroundings::
owner_stick`. Validation accepts a move that only tosses.

### E49: the latch (sim 119, owner decision 74)

`Move::latch { x height clock_num clock_den }`. A command grab (E6) catching into a move with a clocked latch sets the ordinary hold
clock on the held fighter (scaled), `grab::update` runs it (with the shared mash) while the holder is in a clocked latch move, and
`grab::pin` does not pin the held fighter in front: it falls on its own (`fighter::held_fall`) and the holder rides it. Why a move
field: the latch is what the holder's move does with its catch, the same shape as `hold_at` and `carry`; its hits and exits stay the
move's own (pummel rows, a script), so no character code is in the engine. `Hitbox::fixed_stun` gives the jump-off its 20 frames.

### E50: once-per-airtime marks and flag versions (sim 119, owner decision 75)

Hashed `Fighter::air_marks` (cleared by landing or a ledge, not a hit), `air_mark` / `set_air_mark`, `Move::flag_versions`. Why: the
engine's air flags are cleared by a hit, which the reference's "first charge only" rule does not do; a version picked by flags or
marks also bars a special outright (the kick's rule), so one field serves both.

### E51: unblockable hits and self damage (sim 119)

`Hitbox::unblockable`, `Move::self_damage`: the flying barrels and the crash, both reference properties of hits and moves.

## Not built (limits)

* The popgun's muzzle hit (unpublished numbers) and taller later hit; a peanut caught and thrown back (no aerial catch).
* House limits: the grabs' aerial-target boxes; the throws' bystander hits and thrower invincibility; one get-up attack slot (the
  face-down and trip get-up attacks).
