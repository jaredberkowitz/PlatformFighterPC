# Content format and scripting (Phase 5)

Everything the simulation plays with, the roster, the movesets, the stage and the rules, can live in one text file
(a **bundle**, `.pfc`). The game loads `content/base.pfc` at start-up; `--content=PATH` after `--` picks another.
Two layers, as in the plan (section 7.1):

1. **Data**: physics numbers, hitbox timelines, projectiles, motions. This should cover most of a character.
2. **Scripts**: a small bounded language for what data cannot say: steering a special, branching into other
   moves, a projectile that homes. Scripts are compiled to bytecode and run by the VM in `sim-script`.

Tools (`cargo run -p tools -- ...`, or `target/release/pftool`):

| Command | What it does |
| --- | --- |
| `content-check <file.pfc>` | Load and validate; prints a summary or every problem with line numbers |
| `content-pack <in.pfc> <out.pfc>` | Validate, then write canonical text with a fresh hash and this build's sim version |
| `content-export <out.pfc> [name]` | Write the built-in roster (this is how `content/base.pfc` was made) |
| any command with env `PF_CONTENT=<file>` | Run the fuzzers, `net-host`/`net-join`, replays against that bundle instead (the fuzzers draw their fighters from every class the bundle has a weapon for, built-in or made, and print a per-class count) |

## Exit criterion: data plays identically to code

`content/base.pfc` *is* the hand-coded roster written out as data. Tests prove it:
`sim-content/tests/bundle.rs` (round trip, `loaded == hand-coded`, same checksum on every frame of 6 random
matches) and `tools/tests/content_files.rs` (the shipped file, 4 matches of 3000 frames). While the Rust builders
still exist the two must agree; if you change a number in code, re-export the file, and the test tells you.

## File format

```text
# comment to end of line
bundle {                         # the manifest, required
    schema 7
    name "Base Roster"
    sim_version 19
    hash af788ccc46c9f9e2
}
ruleset { damage_mult 1.2 ... }              # optional; missing values use the standard rules
weapon longsword { move jab { ... } ... }    # a moveset
fighter duelist { weapon longsword  walk_speed 1.4 ... }
stage proving_grounds { blast_left -28 ... platform { ... } ledge { ... } spawn { x -4 y 0 } ... }
```

* A line is `field value`, or `kind [name] { ... }`. Words are letters, digits and `_ . + -`; anything else
  needs `"quotes"`. Whitespace and line breaks are free.
* Numbers are decimal text (`1.5`, `-0.25`, `12`) read into 16.16 fixed point, rounded to the nearest step.
  Writing prints the shortest decimal that reads back to exactly the same value, which is why a value the
  hand-coded roster made as `1.05` may print as `1.04999`: the file is exact, not rounded.
* **Names**: fighters, weapons and the stage have names (1 to 32 letters, digits, `_`, `-`; unique). Names are
  labels only: they are not part of the content hash.
* **Fighters** need every value in `FighterParams` (including `hitbox_scale`, the size of the fighter's attacks: 1 is the moveset as written) (`sim-core/src/content.rs`), or `inherit other_fighter` to copy
  an earlier fighter and override a few. `weapon <name>` picks the moveset.
* **A fighter's passive shield** (sim 91, E14; optional, no schema change): `passive_shield { x y crouch_y [half_height] radius }`, a
  standing capsule centred `x` ahead of the feet (the way the fighter faces) and `y` up (`crouch_y` up while crouching), reaching
  `half_height` above and below its centre, of `radius`, scaled by `hitbox_scale`. While the fighter stands, walks or crouches on the
  ground, a projectile box that touches it from in front (in front of the fighter and not moving away from its front) is blocked: no
  damage, no launch; the fighter freezes for the hit's shield hitlag (with shield SDI) and is pushed back by a raised shield's
  pushback for that hit, with no shield damage or stun; the shot ends as on a shield (a piercing one goes on; a burst is never
  blocked). Melee hits are never blocked. Inherited by `inherit`; none by default. The wanderer's: `x 0.6 y 1 crouch_y 0.4
  half_height 0.35 radius 0.45` [E].
* **A fighter's air jump** (sim 89, E33; optional, no schema change): `air_jump_armor { start end [kind] [threshold] [subtract]
  [min_kb] }` (the same fields as a move's `armor`, below) is armor during the midair jump: frames counted from the jump (0 is the
  jump's own frame, as a move's are), lasting only while the fighter stays in the jump (an attack, a special, an air dodge, a wall
  jump, a landing, a ledge or a launch ends it); `air_jump_turns true` turns the fighter round when the stick is held back as it
  jumps. Both are inherited by `inherit` and default to none / false. The bounder's: `start 0 end 43 kind knockback threshold 120
  subtract true min_kb 60` (the reference's double-jump armor, frames 1-44). `air_jump_drift` (sim 95, owner decision 59; optional,
  inherited, default 100) is the midair jump's sideways speed in percent of the stick times the air speed (the reference's per-body
  `jump_aerial_speed_x_mul`; the bounder's 120).
* **A fighter's gauge** (schema 6, sim 85): an optional `gauge { max per_dealt per_taken mode_frames [walk] [dash] [run] [traction]
  [air_speed] [air_accel] [air_friction] [gravity] [fall] }` block inside the fighter (`sim_core::content::Gauge`; the game calls the
  champion's the **Boost** gauge, `content::GAUGE_NAME`). It fills by `per_dealt` units a percent of damage the fighter's hits add to
  another fighter and `per_taken` a percent added to its own (the damage actually added, after staleness and the one-on-one
  multiplier), never from shields or counters (and, since sim 123, a curse's damage over time counts as taken but not as dealt); while the fighter is in a `gauge_drain` move, or held by an ordinary grab that caught it
  in one, damage taken drains it instead. Reaching `max` starts the powered mode for `mode_frames` frames (counted every frame,
  hitlag included): nothing more is gained or lost, and the body moves with its numbers multiplied by the percents (default 100 each:
  `walk` the walk speed and both walk accelerations, `dash` both dash speeds, `run` the run speed and both run accelerations,
  `traction` `ground_friction` and `dash_attack_friction`, `air_speed`, `air_accel` both parts, `air_friction`, `gravity`, `fall` both
  fall speeds; jump speeds never change). When the time runs out the gauge empties; a special with a `limit_version` used in the mode
  spends it at once; a KO empties it and ends the mode. No block (or `max 0`): no gauge. A fighter that inherits keeps the other's
  gauge unless it has its own block. Ranges: `max` 0 to 1000, the rates 0 to 100, `mode_frames` at least 1, the percents 1 to 400.
  Sim 121 (E53, the phantom's partner gauge; each optional, defaults leave a gauge as before): `per_frame` (0 to 100, default 0) units
  fill it by themselves every frame (not on a revival platform, not in the mode); the gain from damage taken is multiplied by
  `behind_1` / `behind_2` (per mille, 1 to 4000, default 1000) one / two or more stocks behind the leader of a stock match, and with three
  or four fighters in the match by `crowd_3_0` .. `crowd_3_2` / `crowd_4_0` .. `crowd_4_2` (per mille, by the same gap: none, one, two
  or more; default 1000); while the mode runs nothing fills it and each percent taken cuts `mode_taken_2` / `_3` / `_4` frames off the
  mode with 2 / 3 / 4 fighters (rounded; 0 to 1000, default 0); a KO leaves `ko_keep` units (`ko_keep_2` two stocks behind the leader,
  `ko_keep_3` three or more; times the crowd row; 0 to `max`, default 0) and ends the mode; `keeps true` (default false): a special's
  `limit_version` does not spend the gauge; `halts true` (default false): the mode's clock stands still while the shield is broken.
* **A fighter's item throws** (schema 7, sim 111, E4): `item_throws { forward back up down dash drop }`, every value required: the frame
  (1 for the throw's first) on which each item throw lets go of the item (the reference's per-body release frames: forward, back,
  up and down throws, smash or not, on the ground or in the air; the dash throw; the grounded drop). From 1 to 60, and each inside
  the ruleset's length for that throw. Inherited by `inherit`; without the block, the standard 8 / 11 / 12 / 7 / 5 / 6.
* **Items** (schema 7, sim 111, E4; owner decisions 54 and 64): a weapon may make items, `item { ... }` blocks numbered by their order
  (0 first; a weapon that inherits keeps the other's items unless it writes its own). An item is not a projectile: any fighter can
  pick it up, hold it, throw it and drop it; it lies on the stage, can be knocked about and can burst. Fields (world units a frame,
  frames, percents; only `kind` and the `hitbox` required, the rest default to 0, the keeps to 100, `throw_mul` to 1, `knock_hitlag`
  to 100): `kind` (1 to 255: the number the `holding` and `owned` registers read), `gravity`, `max_fall` (needed with gravity),
  `air_friction` and `friction` (sideways slowing in the air and on the ground), `max_speed` (sideways, 0 no limit), `half_width`,
  `half_height` (its box, standing on its position), the bounces (off a floor `floor_keep_x` / `floor_keep_y` percent sideways / up,
  off a wall `wall_keep_x` / `wall_keep_y` and `wall_lift` up, off a ceiling `ceiling_keep_x` / `ceiling_keep_y`; only when its speed
  into the surface is above `bounce_min`, else it stops or lies; then held within `bounce_x_min` to `bounce_x_max` sideways and
  `bounce_y_min` to `bounce_y_max` up), `pick_width` / `pick_height` (half lengths of its pick-up box around its middle, above 0),
  `throw_mul` (the item's throw speed multiplier: a throw goes at the ruleset's speed times the thrower's toss strength `1 + (weight -
  100) / 300` times `throw_mul`, the sideways part times `throw_mul` once more, the reference's), the `hitbox` (its thrown hit, live
  from the throw until it lands, hits a fighter or a shield, or `thrown_life` frames pass, 0 none; it deals the hitbox's damage plus
  `damage_per_speed` times the throw's speed before `throw_mul`, a grounded drop halving the first part), `vanish true` (gone on a
  fighter, a shield, a surface or the end of `thrown_life`; otherwise it bounces off a fighter or a shield keeping `fighter_keep_x`
  (sent back) / `fighter_keep_y` percent and gaining `fighter_lift`), `life` (frames from its making, 0 until used; then it bursts or
  goes), `hp` and `hurt_radius` (with hp above 0, an attack touching that circle round its middle takes its damage off the hp and
  launches it along the attack's angle at `knock_speed_min` for `knock_damage_min` percent up to `knock_speed_max` for
  `knock_damage_max`, frozen `knock_hitlag` percent of the hit's hitlag; one knock per attacker's move; at 0 hp it bursts or goes),
  and `blast { frames [delay] [hits_owner] hitbox { ... } }` (its burst: on a `detonate()` after `delay` frames, at the end of its
  life or at 0 hp, it stands still and hits with the hitbox round its middle for `frames` frames, each fighter once, credited to the
  fighter whose weapon made it; with `hits_owner` that fighter too). Sim 115: `pocketable false` (default true) keeps a pocket (E42)
  from putting it away; `placed { [x] [hurt_half_height] [hitter_lag] [water] [grow_delay] [grow_into] [grow_move] [cuts] [fall_move]
  [chip] [chip_chance] }` makes it a **placed piece** (E43, the reference's sapling and tree; see `sim_core::moves::Placed`): a move's
  `pull` of it sets it down `x` ahead of the maker's feet on its platform (pulling up the placed pieces of its kind it lands on and
  the maker's older one), and it is never picked up, pocketed, moved or knocked about (it needs no pick-up box); with `hp` an attack
  or a shot touching its hurtbox (`hurt_radius` round its middle, reaching `hurt_half_height` above and below) takes hp (a shot is
  used up on it, a piercing one passes; the attacker is frozen `hitter_lag` percent of the hitlag, at most 200), at 0 it goes; with
  `water` above 0 the shots with `water` touching it give it water and once it has `water`, `grow_delay` frames later it becomes item
  `grow_into` and the projectile of move `grow_move` (a move name) is fired there for the last waterer; with `cuts` above 0, `cuts`
  hits of a `chops` move from one side fell it: the projectile of move `fall_move` is fired from its feet away from the chopper, for
  the chopper, and item `chip` (an item number) is made there `chip_chance` times in 1000 (at most 1000). Sim 117 (the reference's
  banana peel; all off by default): `damage_add` (may be negative) is added to the thrown hit's damage after a drop has halved its
  base (the reference's base damage modifier, which a drop does not halve); `uses` (0 no limit, at most 200) is how many uses the item
  has: a fighter its thrown hit reaches (not a shield), a fighter its `trap` trips and a throw by a fighter (not a move's `toss`) each
  spend one, and with none left it is gone (at once on a hit or a trip, at the end of its flight after a throw);
  `trap { [kick_x] [kick_y] hitbox { ... } }` (E45) makes it trip the fighters on the ground that walk into it while it lies or slides
  on a platform: a fighter (not the item's last thrower; not lying, sitting, in hitstun, held or intangible; shielding or not) whose
  lowest hurtbox circle touches the hitbox (placed `x` ahead of and `y` above the item's foot by its facing) trips by the hitbox's
  `slip` (needed: `hard` or `soft`) and takes its damage, then the item is kicked up and away from it at `kick_x` / `kick_y` (not
  negative). Up to 8 items a weapon; percents at most 200, values not negative (`damage_add` aside), ranges low to high.
  **The universal actions** (every fighter, the ruleset's `item_*` values and the body's
  `item_throws`): on the ground with an item in reach (the ruleset's reach box overlapping the item's pick-up box, the item lying or
  stuck), grab, or an attack that is not a smash input, picks it up (`item_pick_frames`, in hand at once); holding one, attack throws it
  by the stick (up, down, forward with the stick forward or in the middle, back, which turns the fighter round; a smash input makes it a
  smash throw; out of a dash or run the dash throw), grab throws it by the stick or with the stick in the middle drops it (the
  grounded drop) or, in the air, lets go of it at once with no speed and no lag; out of a shield attack or grab throws it. Specials,
  jumps, dodges and the shield keep it in the hand; a hit, a grab, a ledge or a knock-out lets go of it. A shot whose `lodge_item`
  names one of the weapon's items becomes that item where it lodges (stuck for `lodge` frames, then picked up it lasts as its kind does).
* **Weapons** list the moves they have; a move not mentioned is empty (and `validate` says which empty slots are
  allowed). `inherit other_weapon` starts from an earlier moveset and replaces the moves you write. `limbs true` marks a moveset whose
  attacks are the body's own (claws, fists, feet): while a hit is out, the limb throwing it can be hit (a held weapon cannot).
* **Moves** are named by slot: `jab ftilt utilt dtilt dash_attack fsmash usmash dsmash nair fair bair uair dair
  neutral_special side_special up_special down_special jab2 jab3 grab dash_grab pummel fthrow bthrow uthrow dthrow
  ledge_attack get_up_attack ext0 ext1 ext2 ext3 ext4 ext5 ext6 ext7 ext8 ext9 pivot_grab ext10 ext11 ext12 ext13 ext14 ext15`.
  Frames count from the move's first frame (see `docs/COMBAT.md`). The sixteen `ext` slots (`ext10` to `ext15` since schema 5,
  after `pivot_grab` so every older slot keeps its number) are never started by a button: a script reaches them with `goto`, or a
  counter stance answers with one. They are for the later hits of a multi-hit special, a counter-attack, and so on.
  `pivot_grab` is started by grabbing while turning around out of a dash or run; a weapon that leaves it empty uses its dash grab.

A move block (omitted fields take the default shown):

| Field | Meaning | Default |
| --- | --- | --- |
| `total_frames` | length; required | |
| `landing_lag`, `autocancel_before`, `autocancel_after` | aerial landing rules | 0, 0, 255 |
| `intangible` | intangible frames from the start | 0 |
| `helpless_after`, `turns_around`, `grabs_ledge` | flags (`grabs_ledge`: either side, any frame of the move) | false |
| `intangible_from`, `intangible_from_air` | the move frame the intangibility starts on (on the ground; in the air, 255 = as on the ground); `intangible` stays the last frame counted from the start | 0, 255 |
| `ledge_front_from`, `ledge_back_from` | per-move ledge windows: from this move frame a ledge in front / behind can be caught during the move (255 never) | 255, 255 |
| `fall_drift`, `fall_landing_lag` | the special fall after this move: percent of the body's air speed and acceleration kept (0 = the ruleset's `helpless_drift`), and its landing lag (0 = the body's `helpless_landing_lag`) | 0, 0 |
| `fall_speed`, `fall_accel` | the special fall's own air speed (world units, replacing the speed `fall_drift` gives) and stick acceleration (the share of the air acceleration the stick adds at a full tilt, replacing the body's `air_accel_stick`; the base acceleration is then the body's own): the reference's separate air speed and "additional acceleration" of a special fall (sim 91, E11) | none, none (`fall_drift` scales both) |
| `keep_charge` | a follow-up (a move reached by `next` or a script's `goto`) keeps the charge the move before it had, so its hits are charged too: the reference's two-part smash attack (sim 91, E36) | false |
| `rise_ground`, `rise_air` | percent of the motion's vertical speed for a move started on the ground / in the air (the reference's height multipliers) | 100, 100 |
| `start_speed` | started in the air, percent of the horizontal speed kept | 100 |
| `steer_frame`, `steer_stick`, `steer_degrees` | angling by the stick: on that move frame, a forward or backward tilt past `steer_stick` percent turns the motion by up to `steer_degrees`, evenly to the rim (255 = no angling) | 255, 0, 0 |
| `air_gravity`, `air_max_fall`, `air_friction`, `air_drift` | the move's own air physics while it runs without a motion (world units; `air_drift` a percent of the body's air speed and acceleration) | the body's; `air_drift` 100 |
| `air_max_rise` | while the move runs without a motion in the air, its upward speed is held at or under this (world units a frame: the reference's maximum vertical speed of a stance) | no limit |
| `land_continue` | touching down does not end the move: it carries on along the ground | false |
| `air { from to [gravity] [max_fall] [friction] [drift] [accel] [speed] [accel_stick] }` | any number: on move frames `from..=to` the move's air physics (without a motion) are these instead of the flat `air_*` fields (the first block covering the frame wins; an omitted value falls back to the flat field, then the body's). `drift` is a percent of the body's air speed and acceleration, `accel` a further percent on the acceleration alone (the reference's horizontal acceleration multiplier: x6 is 600, 1 to 1000). `speed` (world units) replaces the air speed and `accel_stick` the stick's share of the acceleration (after the percents; the base acceleration stays) for these frames: a move's own air speed and "additional acceleration" (sim 91) | none; drift 100, accel 100 |
| `fast_fall_from` | a fast fall is only possible from this move frame on | 0 (any frame) |
| `turn_frame`, `turn_stick` | the move's own turnaround: on move frame `turn_frame` the stick held backward past `turn_stick` percent turns the fighter round (in the air its horizontal speed too); it replaces the shared up-special turn window, the up-and-back start and the B-reverse window for this move | none |
| `lands_from` | touching down before this move frame does not end the move (it lands and carries on) | 0 |
| `part_intangible { start end [low] [middle] [high] [limb] [mode] }` | any number: on move frames `start..=end` the named hurtbox circles cannot be hit (`true` each; the body's low, middle and high circles, and `limb`, the arm or leg an attack throws out), while the rest of the body can: the reference's per-limb intangibility (a kicking leg, a headbutting head); `mode` (sim 123; `any`, the default, `plain` or `powered`) limits the window to an attack begun without or with the gauge's powered mode, as a hitbox's (the reference's arm intangibility that comes only with its partner) | none |
| `armor { start end [kind] [threshold] [subtract] [min_kb] }` | at most one: on move frames `start..=end` a hit deals its damage and hitlag but does not knock the fighter out of the move. `kind` `damage` (default: a hit of at most `threshold` percent, compared before the one-on-one multiplier, the reference's damage-based armor), `super` (every hit) or `knockback` (sim 89: a hit of at most `threshold` units of knockback, rage, staleness and set knockback included, before crouch cancelling: the reference's knockback-based armor). `subtract true` (knockback kind only): a hit that breaks it launches with its knockback less `threshold`, at least `min_kb` (only with `subtract`). Damage and knockback armor need a `threshold` above 0. Grabs ignore armor; a push still pushes | none |
| `charge_at` | smash attack hold frame | none |
| `next`, `next_window`, `next_from` | jab chain: move to continue into, and the press window; with `next_from` (a move frame) a press from that frame on carries on at once (the reference's continuable window), otherwise only at the end | none; `next_from` 255 |
| `cancel_from` | a cancellable jab (sim 115, owner decision 70: the previous reference version's jab cancel): from this move frame on, on the ground, any action the fighter could start standing (a jump, dash, crouch, shield, grab, another attack or a special; not a walk, and not the jab itself, which `next` carries on) ends the move and starts on the same frame; must be below the move's total frames | 255 (never) |
| `rehit_start`, `rehit_every` | multi-hit: when hits may land again | none |
| `rehit_groups` | the hit groups the rehit rhythm re-arms, a bit per group (a looping phase is re-armed, its final hit is not) | 255 (all) |
| `landing_move` | a move (usually an `ext` slot) started when this aerial lands outside its autocancel windows: a landing hit; its length is the landing lag | none |
| `on_search` | the move (an `ext` slot) a search box of this move switches to on touching a fighter: it starts on the next frame, keeping the speed | none |
| `air_version` | a special started in the air uses this move instead (its aerial version) | none |
| `smash_version` | a special input like a smash attack (the stick flicked, or the strong button: the `flick` register) uses this move instead, checked after `air_version` (sim 102, E3d: the reference's farther boomerang) | none |
| `catch_move` | the move (an `ext` slot) a command grab of this move (a `grab` hitbox with `command`) goes on with when it catches someone: it starts on the next frame, the holder keeping its speed and staying in the air if it is there; the held fighter has no hold clock and cannot mash out | none |
| `off_edge` | a ground move that goes over an edge goes on as this move (an `ext` slot: its version for leaving the ground, with its own FAF), on the same frame, keeping its hits so far; otherwise a move with a motion goes on as itself and one without ends | none |
| `limit_version` | used while the fighter's gauge's powered mode runs, a special (started by its button) is this move instead (its powered version, then that move's own `air_version` in the air) and the gauge is spent at once, whether it hits or not; follow-ups reached by `goto` never spend | none |
| `gauge_drain` | damage taken during this move drains the fighter's gauge instead of filling it; an ordinary grab that catches the fighter in this move keeps draining on every hit until the hold ends (the reference's charge and its cancel) | false |
| `start_speed_y` | started in the air, percent of the vertical speed kept (the reference's vertical multiplier on startup: a stall) | 100 |
| `motion_drift` | during the move's motion in the air, the fighter steers sideways with its own air control at this percent of the body's air speed and acceleration instead of moving at the motion's `vx` (the motion still sets the vertical speed: a rise with air control); 0 to 100 | 0 (the motion's `vx`) |
| `late_next`, `late_until` | together (sim 95, owner decision 58): a jab whose continuable window runs past its end. Once the move has ended on the ground, a press of attack that would start a jab starts `late_next` instead, up to move frame `late_until` (counted on from the move's first frame); the fighter stands idle meanwhile, and anything else it does closes the window | none |
| `ledge_falling_only` | `true`: the move's ledge windows catch a ledge only while the fighter is not rising (sim 95) | false |
| `drop_from`, `drop_frames`, `drop_while_down` | `drop_from` and `drop_frames` together (E30, sim 95): on move frame `drop_from` the fighter passes through soft platforms for `drop_frames` frames; with `drop_while_down true` also on every later frame of the move that the stick is held down. Solid ground always stops it | none |
| `ledge_drop_front`, `ledge_drop_back` | together: during the move a ledge in front / behind is caught only once the fighter is at least this far (world units) below where the move began (or where a `goto` switched to it), on top of `ledge_front_from` / `ledge_back_from` | none |
| `push_immune_from`, `push_immune_to` | on these move frames a push (a `push` hitbox: the reference's windbox) does not move the fighter at all (the reference's windbox immunity; sim 90) | 255, 0 (never) |
| `on_land` | an aerial move that touches the ground goes on as this move (usually its grounded version) on the same frame, keeping its frame and its hits so far, instead of landing: the mirror of `off_edge` | none |
| `shed` | a move name (E48, sim 117: the reference's rocket barrels flying off their hit wearer): a hit landing on the fighter during this move (not a push, not one its armor withstands) fires that move's projectile at the fighter's feet (the spec's offset, its facing), credited to the fighter; if the hit leaves the fighter in its move (a flinch), the move ends, into the special fall in the air or standing on the ground. A real move with a projectile | none |
| `hold_x`, `hold_y` | together: where a fighter held during this move is pinned, forward and up from the holder's feet (in the air if the holder is, or if `hold_y` is above 0); otherwise the ruleset's `grab_distance` in front, on the holder's platform | none |
| `pull_frame`, `pull_item` | together (schema 7, sim 111, E4): on move frame `pull_frame` the weapon's item number `pull_item` appears in the fighter's hand, if the hand is empty (the reference's bomb pull: none while holding an item); its burst stales as this move. Inside the move, and an item the weapon has | none |
| `spend_item` | on this move frame the item in the fighter's hand is used up (schema 7, sim 111, E4: the double arrow fires the arrow held); 255 never | 255 |
| `hitbox { start end x y radius damage angle bkb kbg [priority] [group] [kind] [shield_damage] [hitlag] [effect] [targets] [fixed_kb] [hitstun_add] [sdi] [link_ahead] [aim] [set_weight] [reverse] [push] [shieldstun] [angle_type] [command] [clanks] [rebounds] [trip] [posvec_x posvec_y posvec_frames] [half_width] [rules] [slip] [fixed_stun] [unblockable] [mode] [part] [blockable] }` | any number; `mode` (sim 121, E52; `any`, the default, `plain` or `powered`): the box is out always, only in an attack begun without the gauge's powered mode, or only in one begun with it (the mode as the attack began counts to its end: `Fighter::move_powered`; the reference's partner moves); `part` (sim 121, E52; 0, the default, or 1): the box is the fighter's own or a second attacker's moving with it (the reference's summoned partner): each part hits a target on its own, so on one frame a target can take the best box of each (the fighter's first, then the partner's: both damages, the partner's launch applied last); `blockable` (sim 123, E56; default false; only on a `grab` hitbox): a raised shield or a reflector field the box touches stops it, so that group of the move catches no one for the rest of the move (the move goes on), and it never catches a fighter in hitstun (the reference's hook: a special's grab box that can be shielded); `fixed_stun` (sim 119; 0 to 255, default 0): a hit that does not tumble its victim holds it in hitstun for exactly that many frames, whatever its knockback's own hitstun (the reference's flip jump-off: 20); `unblockable` (sim 119; default false): a raised shield, or a passive one, does not stop the hit: it lands as on a fighter without a shield (the reference's rocket barrels flying off their hit wearer); `slip` (sim 117, E44; default `none`) is the reference's slip effect: `hard` or `soft` makes the hit always trip a fighter on the ground that is not lying or sitting already, whatever its knockback and angle, instead of launching it (an airborne fighter takes the hit as usual): `hard` with the hit's damage and hitlag and the ruleset's `trip_*` timings (a thrown banana peel), `soft` with no hitlag and the ruleset's `slip_lag` / `slip_max` (walking into a lying peel); `rules` (sim 113, E40, owner decisions 66-67; default `current`) is which version of the reference's hit rules the hit follows: `current` (13.0.1's, every other hit) or `previous` (the previous reference version's, 1.1.x: the damage it adds is multiplied by the ruleset's `prev_damage_mult` (1: no one-on-one multiplier) instead of `damage_mult`, on a shield too; its hitlag is `floor(d x prev_hitlag_per_damage + prev_hitlag_base)` (0.3846154, 5) times the hit's factors, and a crouch shortens only the victim's; rage reaches `prev_rage_max` (0.15); a tumbling launch gets `prev_tumble_hitstun` (1) more frame of hitstun; and the launch keeps that version's physics: no speed-up of a strong launch, the gravity boost on every tumbling launch and no shared fall speed for a vertical one), only on moves the owner names (the Settler's jab); `half_width` (sim 102, E38; default 0) makes the box a lying capsule reaching that far ahead of and behind its centre (with `half_height` too, a rounded rectangle), scaled by the body's `hitbox_scale` but not by a projectile's charged size (the reference's horizontal capsule: a bolt); `posvec_frames` above 0 makes the hit a position vector (sim 91, E34, the reference's): the victim's feet are carried straight to the point `posvec_x` ahead of (the attacker's facing) and `posvec_y` above the attacker's feet where it hit, at the speed that covers the distance in `posvec_frames` frames, with no gravity on the way, and stop there; the angle and knockback still give the hitstun (scaled like the position; on a projectile the attacker is its owner); `shieldstun` is the percent of the normal shield stun the hit causes (the reference's per-hitbox shieldstun multiplier, applied to the stun's damage part; default 100); `kind` is `normal grab throw pummel search` (`search`: a search box, which only notices a fighter it touches, shields or not, and switches the move to its `on_search` on the next frame; it deals nothing and never clanks); `shield_damage` is the percent of its damage a shield takes (default 100); `hitlag` the percent of the normal hitlag (default 100); `effect` is `normal`, `electric`, `bury` (sim 90, E5: a fighter standing on the ground, or sitting after a trip, is buried instead of launched, for the ruleset's `bury_*` frames, and its knockback is worked out at weight 100 whoever is hit; an airborne one takes it as an ordinary hit; it ignores subtractive knockback armor) or `encase` (sim 110, E8: only on a `throw` hitbox of a move with an `encase` block, which shuts the released fighter in an egg instead of launching it); `group` 0 to 7 (each group hits a target once per move, or once per rehit); `targets` is `all`, `grounded` or `aerial`; `fixed_kb` set knockback (the same at any percent; 0 = normal); `hitstun_add` extra hitstun frames (negative takes frames away; a launched victim keeps at least one); `sdi` the percent of the normal survival-DI distance (default 100). `link_ahead` moves an autolink's pull point that far in front of the hitbox (world units). `aim` is `any` (default), `up`, `straight` or `down`: the hitbox is only out when the move was aimed that way as it started (a forward tilt or smash with the stick tilted up or down, as the reference's angled moves). `set_weight` (`true`/`false`, default false) works the knockback out as if the victim weighed 100 (the reference's set weight). `reverse` (default false) turns the victim round (facing away from the attacker, horizontal speed reversed: the reference's reverse effect). `clanks` (default true): false makes the hit never clank with other attacks or projectiles (the reference's `clang=f`; on a projectile, its transcendent priority): it trades instead. `rebounds` (default true): false makes a hit that clanks stop (it does not land that frame) while its owner carries on with the move instead of rebounding (the reference's `rebound=f` without `clang=f`; sim 86). `trip` (0 to 100, default 0) is the percent chance that the hit trips a fighter on the ground whose launch would not lift it off the ground (sent sideways or down): it takes the damage and hitlag and sits down instead of being launched (the ruleset's `trip_lag`, `trip_max` and `trip_invuln`), rolled on the game's own random generator (the reference's `trip`; sim 86). `command` (default false, on a `grab` hitbox) makes it a command grab: it catches fighters in the air too, shielding or in hitstun (not on a ledge), and the holder goes on with its move's `catch_move`. `push` (default false) makes the hit a shove (the reference's flinchless hit): its knockback is added to the victim's speed (only sideways on the ground) with no hitlag, no hitstun and no change of state; a shield takes it as pushback. `angle` 361 is the reference's 361 angle (set by the knockback and by whether the victim is on the ground), 365 (sim 123) the half-momentum angle (the victim is launched at half the attacker's velocity as it hit, whatever the knockback, on the ground or in the air; not a tumble), 367 the autolink angle (an airborne victim is carried along at the attacker's velocity and pulled toward the hitbox; a grounded one goes up at 80), 366 the vortex angle (grounded or not, the victim is sent the way the attacker was moving at the knockback's launch speed and drawn to the hitbox centre over its first 5 frames; for a projectile standing still, such as a burst, the pull alone) and 368 the pull angle (ours: no launch; the victim is drawn to just in front of the attacker, or a projectile's owner, while its hitstun lasts). `half_height` (default 0) makes the box a standing capsule from `y - half_height` to `y + half_height` (scaled like the radius): a tall wall, as a projectile's stacked circles; it is tested against the hurtboxes, a counter's bubble and a reflector's fields. `angle_type` is which way 0 degrees points (the reference's angle type): `box` (default: a box behind the attacker sends backward, any other forward), `forward` (the way the attacker faces), `backward`, `position` (away from the attacker, from the side the victim is on) or `travel` (the way the attacker is moving sideways, its facing when it is not: the reference's speed-based type 1); for a projectile the attacker is the projectile (its direction of travel and its position) | |
| `motion { start end vx vy }` | scripted-motion segments (`vx` forward-relative) | |
| `projectile { frame x y speed life end_damage [angle] [gravity] [max_fall] [bounces] [bounce_min] [bounce_max] [bounce_keep] [bounce_keep_y] [bounce_max_fall] [pierce] [rehit] [drag] [lodge] [lodge_angle] [lodge_item] [kind] [limit] [catch_from] [catch_move] [pocket] [water] hitbox { ... } [phase { from hitbox { ... } }] [burst { life [rehit] [on_wall] [on_ground] [on_expire] [keep_hits] [on_hit] [on_shield] [y] phase { from hitbox { ... } } }] [charged { full [snap] damage_to speed_to life_to [size_from] [size_to] [floor] [times] [power_to] }] [on_clank { life [vx] [vy] [gravity] [max_fall] }] }` | at most one. Sim 115: `pocket` is `yes` (the default: a pocket, E42, can put it away, the reference's rule for a shot that can be reflected), `never` (a line its thrower holds, a standing explosion) or `frees` (it can, and once put away it no longer counts as its maker's, who may fire another: the reference's rocket); `water` (default 0) is the water it gives each frame it touches a placed piece that grows (E43, the reference's watering can). Sim 113: `power_to` (1 to 255, default 100) makes every hit of a charged shot, its phases too, deal from 100 percent at no charge to `power_to` percent at `full` (the reference's smash-attack charge carried by a shot a smash attack makes, such as the settler's bowling ball at 120); `on_hit false` (default true) makes a bursting projectile burst only on a wall, the ground or its life's end (`on_wall`, `on_ground`, `on_expire`): a fighter it touches takes its hit and it is used up there (the reference's rocket). Sim 111 (schema 7, E4): `lodge_item` (an item number of the weapon; needs a `lodge` and no `bounces`) makes the lodged shot that item, stuck where it lodged for `lodge` frames, which any fighter can pick up; `times` (1 to 100, default 100) is the percent of the charged damage the shot deals, applied after `floor` and floored again (whole numbers: the double arrow's x0.8). Sim 99: `drag` slows it that much a frame along its flight down to a stop (E35); `lodge` (frames) makes it stick where it meets a platform top or a solid wall at `lodge_angle` degrees or more (0: any; shallower it is gone), and stuck it neither moves nor hits for `lodge` frames (E3b, the reference's arrows; not with `bounces`). `charged` (E35) scales the shot by the firing fighter's `charge` as it appears (a smash charge, or a script's `set_charge` carried by `keep_charge`): capped at `full`, `full` at once from `snap` (0: none); damage from the hit's to `damage_to` (rounded down with `floor`), speed from `speed` to `speed_to` (a script's `spawn` speed scaled by the same ratio), life from `life` to `life_to`, size from `size_from` to `size_to` percent, each in proportion; no falloff (`end_damage` must equal the damage). `kind` and `limit` (E3d): at most `limit` live shots of tool `kind` (or of this move when `kind` is 0) per fighter, counted by the `owned` register; from age `catch_from` a shot still its firer's that touches its firer is caught (removed), and a free firer (standing, walking, running, dashing, turning, crouching or airborne) starts `catch_move`; a dodging firer lets it pass. `on_clank`: a clank with an attack drops it instead of destroying it: it stops hitting, lasts `life` frames, moving `vx` the way it went (negative is back) and `vy` up, pulled down by `gravity` to `max_fall`, and lies on a platform it lands on. Its flight: fired `angle` degrees above its facing (negative is down), pulled down by `gravity` to at most `max_fall` (0: no limit); with `bounces` above 0 it bounces off the top of any platform that many times (the next touch removes it), coming off at its falling speed held between `bounce_min` and `bounce_max`, keeping `bounce_keep` percent of its forward speed and `bounce_keep_y` percent of that vertical speed (sim 95); a landing faster than `bounce_max_fall` (0: no limit) does not bounce. Up to two `phase` blocks replace its hit from age `from` on (in order; a phased projectile's damage does not fall off). `burst` (E3e): the first fighter it touches (shielding or not) takes its hit, then it stops there, `y` higher, and becomes the burst: its age starts again and it lasts `life` frames, piercing, with its own `rehit` and its hits from its `phase` blocks (no hit before the first; radius 0 a gap), hitting that fighter again; with `on_wall` a solid wall bursts it too (just short of it) instead of removing it; with `on_ground` (sim 95) a landing it does not bounce from bursts it there; with `on_expire` the end of its life bursts it where it is; with `keep_hits` the burst keeps the projectile's hits, so the fighter it hit is not hit again by a burst phase of the same group; a burst is never reflected and never clanks | angle 0, gravity 0, max_fall 0, bounces 0, bounce_keep 100, bounce_keep_y 100, bounce_max_fall 0 |
| `reflector { start end x y radius damage_percent speed_percent [reverse_vertical] [life_percent] [max_damage] [detect_start detect_end detect_x detect_y detect_radius] [detect_then] [half_height] [body_start body_end body_x body_y body_radius] }` | at most one. `reverse_vertical` turns the shot's vertical speed round too; a reflected shot gets `life_percent` of its lifetime; a shot of more than `max_damage` (0: no limit) goes through; a shot touching the detector circle on `detect_start..=detect_end` skips the move ahead so the reflector is up the next frame, or with `detect_then` (a move name) starts that move at once (its own reflector meets the shot on the same frame; the detector then works on its whole window); `half_height` above 0 makes the field a standing capsule from `y - half_height` to `y + half_height`; `body_*` is a second circle with the same numbers on its own frames (a body reflector beside a barrier; radius 0 for none) | false, 100, 0, no detector, skip ahead, a circle, none |
| `counter { start end then percent min_damage [max_damage] [melee_only] [bubble_x bubble_y [bubble_half_height] bubble_radius] }` | a counter stance: a hit landing in move frames `start..=end` is cancelled and the fighter switches to the move `then` (an `ext` slot), turned toward the attacker. It catches even while the stance is intangible (only the hits it catches: a grab or a push does not reach an intangible stance). `melee_only` leaves projectile hits alone (they land as usual). A push (`push` hitbox, a windbox) is never caught: it lands on the body as usual (since sim 78). The `bubble_*` fields (sim 78) give the area that catches, the reference's counter bubble: a circle of `bubble_radius` centred `bubble_x` ahead (mirrored by facing) and `bubble_y` up from the feet, or a standing capsule reaching `bubble_half_height` above and below that centre, scaled by the body's `hitbox_scale`. A hit touching the bubble is caught; one that reaches only the body is taken. Without `bubble_radius` (or 0) the body's hurtboxes catch, as before | no cap, false, no bubble |
| `counter_strike` | this move's hitboxes deal what the stance caught: `percent` of the caught hit's damage, at least `min_damage` and (above 0) at most `max_damage` | false |
| `charge_bonus` | extra damage at full charge for this move, in percent (0 uses the ruleset's `charge_bonus_percent`) | 0 |
| `charge_add { full add [floor] }` | additive charge damage (E29, sim 90): each hit deals its damage plus `add` times the share of `full` frames the move's charge reached (at most all of it; a script sets the charge with `set_charge`), rounded down to a whole percent with `floor true`, instead of the multiplicative `charge_bonus`; `full` above 0 | none |
| `encase { base per_percent percent_cap mash_stick mash_button hit_cut damage_taken launch_x launch_y grow_from grow_frames size shift_x shift_y air_shift_x air_shift_y exit_vy exit_intangible }` | at most one, every field required (E8, sim 110: the reference's egg). Its move's `effect encase` throw row shuts the released fighter in an egg (state `Encased`) for `base + per_percent x min(p, percent_cap)` frames (`p` its percent before the throw) less `hit_cut` for every percent the throw and each later hit actually add; the count runs a frame at a time outside hitlag and `mash_button` / `mash_stick` more on a frame with a new button press / a stick flick. It is laid at `launch_x` (forward by the thrower's facing) / `launch_y` world units a frame and falls with the fighter's own gravity, fall speed and traction (no ledges, no techs); on the ground a stick flick adds `shift_x` that way and a flick up hops it at `shift_y`, in the air `air_shift_x` / `air_shift_y`; a hit on it adds `damage_taken` percent of its damage with its hitlag and never launches it; grabs cannot catch it; its hurtbox is one round egg `size` percent of the body's height across, growing from `grow_from` percent over `grow_frames` frames; at 0 the fighter breaks out into the air at `exit_vy` with `exit_intangible` intangible frames. `base` above 0; `damage_taken` 0 to 200, `grow_from` 1 to 100, `size` 1 to 200; the move needs an `effect encase` throw row and such a row needs the block | none |
| `carry { walk clock_num clock_den jumps hold_x hold_y forward back up down [ledge] [armor { ... }] }` | at most one, on a throw (E7, sim 98: the reference's cargo hold): the move is a lift that, ending with the fighter still held, goes into carrying it (state `Carrying`). The clock is set on the lift's first frame to the hold time x `clock_num / clock_den` less the hold time already used (mashing included), and runs on with the usual mash cuts; the holder walks at `walk` percent of its walk speed and both walk accelerations, has `jumps` jumps in all, walks off edges and falls with the fighter held at `hold_x` / `hold_y` (forward, up from its feet), catches a ledge only with `ledge true` (letting go); attack throws with the move named by the stick (`forward` also for the stick in the middle; each a real move with a throw hitbox and no carry of its own); `armor` (the move armor's fields) covers the carry, its frames counted from the carry's first (255 and on as 255), and a hit it withstands does not end the carry. `clock_num`, `clock_den` above 0, `walk` 1 to 1000 | none |
| `version { chance move }` | any number (E39, sim 113: the reference's moves drawn at random as they start, such as the settler's turnips): on the move's first frame the game's random generator picks a version, `move` (a move name, a real move with no versions of its own) with `chance` percent each, the move itself with what is left of 100; the chances add up to at most 100 | none |
| `balloons { count hp x y radius fuel refill one_below pop_vy }` | at most one, every field required (E41, sim 113: the reference's balloon flight). While the fighter is in a move with this block it carries `count` balloons (1 when its helium is below `one_below` frames of flight) in one hurt circle of `radius` around the point `x` ahead and `y` above the feet (world units, scaled by the body's `hitbox_scale`); a hit that reaches the circle without touching the body takes `hp` off the next balloon (before the one-on-one multiplier) and pops it at 0, at most one a hit, the hit used up there; the last one popping drops the fighter into the special fall with `pop_vy` upward speed. Its helium is a tank of `fuel` frames of flight: each frame of a balloon move outside hitlag uses one, and when it is empty the balloons let go (the special fall); out of a balloon move the tank refills `refill` sixteenths of a frame a frame (the weapon's first balloon move's numbers), and a KO fills it. The `balloons` register reads the count. `count` 1 to 8, `hp` and `radius` above 0, `fuel` 1 to 4000, `one_below` at most `fuel`, `pop_vy` not negative | none |
| `pocket { start end [x] [y] radius }` | at most one (E42, sim 115: the reference's Pocket). On move frames `start..=end` a circle of `radius` at (`x` ahead, `y` up) from the feet (scaled by `hitbox_scale`) puts away the first thing that touches it while the fighter's pocket is empty and its hand too: a flying projectile of another owner whose `pocket` allows it (never a burst), or an item that is not held and is `pocketable`; holding an item on `start`, that item is put away instead and an item already in the pocket comes into the hand. The thing waits in its pool frozen and unseen, still counting as out for its maker, until an `unpocket` takes it out; a KO loses it. The `pocketed` register reads it. `start` at most `end`, `end` inside the move, `radius` above 0 | none |
| `unpocket { frame [x] [y] [power] }` | at most one (E42, sim 115). On move frame `frame` what the fighter put away comes out: an item into the empty hand (its clock from 0), a projectile from (`x` ahead, `y` up) at the speed it was caught with turned forward, its clock and hits starting again, now the fighter's, dealing `power` percent of its own damage (its earlier multipliers dropped; the reference's 190) and counting as this move for stale moves. `frame` inside the move, `power` 1 to 1000 | power 100 |
| `latch { [x] height [clock_num clock_den] }` | at most one (E49, sim 119: the reference's flip, clinging to the head of the fighter it caught): while the fighter holds someone in this move (reached by a command grab's `catch_move`, or by a `goto` from one), the held fighter is not pinned in front: it falls on its own (gravity to its top fall speed, landing on a platform, standing still on one) and the holder rides it, its feet `x` ahead (its facing; default 0) of the held fighter's feet and `height` percent (above 0) of the held fighter's body height above them. With `clock_num` above 0 the ordinary hold's clock runs in the move (set on the catch: the held fighter's hold time times `clock_num / clock_den`, `clock_den` above 0, default 1) and the shared mash shortens it; run out, the pair lets go as an ordinary hold. `clock_num` 0 (the default): it rides with no clock (an exit move: the latch's attack or jump). Its hits (pummel rows, throws) and exits (a script) are the move's own | none |
| `flag_version { [flags] [marks] [move] }` | any number (sim 119, owner decision 75): starting this special, the first version with any of its `flags` (bits of the fighter's once-per-airtime flags, `set_air_flag`: cleared by landing, a ledge or a hit) or any of its `marks` (bits of its once-per-airtime marks, `set_air_mark`: cleared only by landing or a ledge) set starts `move` instead (checked after `air_version`); with no `move` the special does not start at all (nothing turns, nothing is spent). Flags or marks must be set; the move a real one. The reference's rocket barrels: the slow-falling charge only the first in an air time, and barred after the flip's kick | none |
| `self_damage` | percent the fighter takes itself on the move's first frame (sim 119: the reference's rocket barrels crashing), 0 to 999 | 0 |
| `armor_damage`, `armor_hitlag` | sim 121 (E55, the reference's guard stance): per mille of a hit's damage the fighter takes on a frame the move's `armor` covers (0 to 1000), and percent of the hitlag a hit the armor withstands causes for both fighters (1 to 400); a withstood hit also starts the `withstood` register | 1000, 100 |
| `gauge_gain` | sim 121 (E53): per mille of the gauge's gain from damage taken while the fighter is in this move (0 to 10000; the reference's guard: 6737) | 1000 |
| `tether { from to ahead [behind] up [down] [hang] reel [drag] limit }` | at most one (sim 123, E56: the reference's tether recovery): on move frames `from..=to` (inside the move), in the air, a ledge no one holds whose corner lies `behind` behind to `ahead` in front of the fighter's feet and `down` below to `up` above them (world units, 0 to 64), with the fighter outside the ledge's edge, is caught: the fighter hangs from it `hang` frames (its speed cut by `drag` per mille a frame, 0 to 1000, no gravity), is reeled to the ledge's hanging point over `reel` frames (at least 1) and grabs the ledge as any grab (the body's ledge grab limit and intangibility). The stick held down lets go; a ledge someone holds when the reel ends throws the fighter off as a knocked-off fighter; at most `limit` catches (at least 1) until the fighter lands or gets up onto the stage. While hanging it is in `Tether` (and does not hold the ledge) | none |
| `dodge_stale` | sim 123 (`true`/`false`): the move is a dodge that stales like a roll (the reference's gun dodges): it is made with the dodge staleness so far and adds a level (the ruleset's `dodge_stale_*`), and its intangibility starts up to `dodge_stale_roll_shift` frames later when fully stale; its length is not stretched | false |
| `curse { damage every frames }` | at most one; needs the move's `projectile` (sim 121, E54: the reference's curse): a fighter the projectile's hit or its burst's damages (not a shield, not a counter) takes `damage` percent (above 0, at most 100; times the one-on-one multiplier) every `every` frames (at least 1) for `frames` frames (at least 1), credited to the projectile's owner, no hitlag and no flinch; a new curse replaces the old one, a KO ends it | none |
| `chops` | `true`: the move's hits cut a placed piece that can be felled (E43, sim 115: the reference's axe; see the item's `placed` block) | false |
| `toss { frame [again] angle speed [smash_speed] [wait] }` | at most one (E47, sim 117: the reference's banana peel, thrown by its move): on move frame `frame` (and on `again`, an item held as the move began), if the fighter holds an item it throws it `angle` degrees above forward (more than 90 is behind; -180 to 180) at `speed` (`smash_speed`, default `speed`, for a special input like a smash attack) times its toss strength, with the item's `throw_mul` as the universal throws; its thrown hit is a drop's (its base halved). A toss on `frame` starts the fighter's `pull_wait` at `wait` frames (0 none: the reference's wait before the next peel; the `pull_wait` register reads it). It is not one of the item's `uses`. The frames inside the move, speeds not negative | none |
| `script { ... }`, `projectile_script { ... }` | see below | none |

Errors are collected, not stopped at the first: a file with a mistyped field, a missing value and an unknown move
reports all three with line numbers. A file that *loads* can still be rejected by `validate` (ranges: zero gravity,
hitboxes outside the move, an endless script, ...). `content-check` runs both.

## Scripts

A script is attached to a move (`script`, runs every frame the move is active, not during hitlag) or to the
projectile the move fires (`projectile_script`, runs every frame the projectile exists). Example, the shipped first hit
of the sword character's Dancing Blade (`sim-core/src/scripts/dancing_blade_1.script`); it remembers a press of the special
button and carries on into the next hit:

```text
var queued;
var rising;
if grounded == 0 && frame >= 27 { end(); }
if special_tap && frame >= 4 {
    queued = 1;
    rising = 0;
    if stick_y > 0.5 { rising = 1; }
}
if queued && frame >= 18 {
    queued = 0;
    if rising { goto(30); } else { goto(29); }
}
```

No shipped move uses a projectile script now (the first placeholder, a homing bolt, was replaced by Shield Breaker); they are
covered by `sim-core/tests/scripted_moves.rs`.

### Language

* **Numbers are 16.16 fixed point** everywhere, including `frame` and `age`: `10` is ten, `0.5` is a half.
  `+ - * /` and comparisons work on them; `*` and `/` rescale. Comparisons and `!`, `&&`, `||` give 1 or 0; any
  non-zero value is true. Overflow saturates; dividing by zero gives 0.
* Statements: `let x = e;` (local, this run only, at most 8), `var n;` (persistent variable, see below),
  `x = e;`, `x += e;`, `x -= e;`, `if c { } else if c { } else { }`, `while c { }`, `return;`, and calls
  `name(args);`. Comments are `//`.
* **Persistent variables** are declared with `var`: 4 per fighter, 2 per projectile. They live in the game state, so
  rollback restores them. A fighter's variables start at 0 when a new attack begins and survive `goto`; a projectile's start at 0.
* **Budget**: at most 1000 instructions per run, 16 values deep on the operand stack, 512 instructions of code,
  4096 bytes of source. A script that runs out is stopped at the same instruction on every machine; effects it
  already had stay. `validate` rejects a script that runs out on every one of 64 trial runs (an endless loop).
* **Whitelist**: a script can read the registers below and call the functions below. There is no file, network,
  clock or random access, and no way to name anything else. Adding an ability means adding an entry to
  `sim-script/src/api.rs` and implementing it in `sim-core/src/scripting.rs`.

### Fighter scripts

Registers: `frame` (move frame, the same numbering as `hitbox start/end`), `facing`, `x`, `y`, `vx`, `vy`,
`percent`, `stick_x`, `stick_y`, `grounded`, `attack`, `attack_tap`, `special`, `special_tap`, `shield`, `charge`,
`hit`, `stored`, `jump`, `presses`, `releases`, `wall`, `flick`, `locked`, `lock_x`, `lock_y`, `boost`, `boosting`, `grab`, `aimed`, `fast_fall`, `owned`, `holding`, `balloons`, `pocketed`, `pull_wait`, `jump_tap`, `powered`, `withstood`. Horizontal speed and `stick_x` are **relative to facing** (positive is forward); positions are absolute.
`attack_tap`, `special_tap` and `jump_tap` (sim 119: what the flip's latch reads to jump off on a new press, not one held
since before) are 1 only on the frame the button went down; to remember a tap, store it in a `var`.
`hit` is 1 once the move has connected. `stored` is the charge the fighter kept from an earlier use (see `store`).
`presses` and `releases` count the attack button's presses and releases since the attack began (the move a button started;
moves reached from it by `next`, `goto` or a counter keep counting), **hitlag included**, which no `attack_tap` can see; the press
that started an attack is 1 (and a release of it already made counts 1; an attack started by another button starts at 0). A rapid jab's entry rule reads them (the reference's "pressed
3 times and released twice since jab 1"). `wall` is 1 when the fighter's last movement step (last frame's, as the script runs before the move moves) was stopped by a wall: what a slide that crashes into walls reads.
`boost` is the fighter's gauge (the body's Boost gauge, 0 to its `max`; 0 for a body without one), `boosting` 1 while its powered
mode runs, `grab` 1 while the grab button is held. `aimed` is how the attack was aimed as it started: 1 up, -1 down, 0 straight (a
forward tilt or smash with the stick tilted up or down; the same as `hitbox aim`), so an angled version can last longer than the
straight one (the hauler's forward tilt: sim 90). `fast_fall` is 1 while the fighter is fast falling (a fast fall lasts until
the next jump, landing, hit or aerial start: what a dive that only bounces off a target when it was not fast falling reads; sim 91).
`owned` is how many of the move's projectiles the fighter has out as the frame began (fired by it, reflected or not, lodged or
fallen too): of its tool `kind` when the projectile has one, else of this move: what a throw that fails while its boomerang is out
reads (sim 102, E3d). Since sim 111 (E4) it also counts the items the fighter made of that kind (the projectile's tool `kind`, or for
a move with no projectile the kind of the item it pulls), wherever they are: what the bomb's down special reads to detonate instead.
`holding` (sim 111, E4) is the kind of the item in the fighter's hand as the frame began (its `item`'s `kind`), 0 for none: what the
bow reads to fire the double arrow. `balloons` (sim 113, E41) is how many balloons the fighter carries in its balloon move (a move with
a `balloons` block), 0 out of one: what the settler's flight reads to flap weaker on one balloon. `pocketed` (sim 115, E42) is what
the fighter has put away in its pocket: 0 nothing, 1 a projectile, 2 an item: what the settler's pocket reads to take it out instead.
`pull_wait` (sim 117, E47) is the frames left of the wait a move's `toss` started (its `wait`), counting down every frame, 0 when
there is none: what the scamp's banana peel reads to make no peel while the wait since the last toss is not over.
`powered` (sim 121, E52) is 1 if the gauge's powered mode was running when the attack began (the move's `mode powered` boxes are
out to its end, even if the mode ends first): what the phantom's gun reads to fire its partner's extra shots and last its longer
length. `withstood` (sim 121, E55) is the frames since the move's armor last withstood a hit, plus one (counted outside hitlag), 0 if
it has not in this attack (a `goto` keeps it): what the phantom's guard reads to answer with its backhand when special is let go within
33 frames of a hit.
`flick` is 1 when the current special was started like a smash attack (the stick flicked its way, within the smash-flick buffer, or the strong button held): what a special with a farther smash-input version reads.

Functions: `set_vel(forward, up)`, `add_vel(forward, up)`, `spawn(forward, up, speed_forward, speed_up)`,
`intangible(frames)`, `end()`, `turn()`, `rehit()`, `stall()`, `goto(move)`, `store(n)`, `air_flag(n)`, `set_air_flag(n)`,
`spawn_scaled(forward, up, speed_forward, speed_up, knockback, size)`, `spawn_at(x, y, speed_forward, speed_up)`, `lock(range)`,
`boost_add(n)`, `set_charge(n)`, `detonate()`, `owns(kind)`, `air_mark(n)`, `set_air_mark(n)`.

* `air_mark(n)` is 1 if once-per-airtime mark `n` (0 to 7) is set; `set_air_mark(n)` sets it (sim 119, owner decision 75). Landing or
  catching a ledge clears all eight; unlike the flags, **a hit does not**: what makes the scamp's slow-falling rocket charge the first
  of an air time only, even after being hit out of it. A move's `flag_version` reads them.

* `owns(kind)` (sim 115, E43): how many live items and projectiles of `kind` (an `item`'s `kind`, a projectile's tool `kind`) the
  fighter made, wherever they are (lying, flying, held or pocketed by anyone), as the frame began; 0 for kind 0. What the settler's
  down special reads to plant, water or chop (its sapling, its tree and the falling tree are three kinds).

* `detonate()` (sim 111, E4): every item the fighter made that has a `blast`, is not bursting already and is not held by another fighter
  (one in its own hand counts) bursts its blast's `delay` frames later (the delay counts from this frame: a delay of 12 bursts on the
  12th frame after, counting this one as the first).

* `set_charge(n)` sets the move's charge (the `charge` register, 0 to 255 frames): a move with a `charge_add` block deals its damage
  plus the share of `full` the charge reached (the hauler's charged punch carries the wind-up's count into the punch with it: sim 90).

* `boost_add(n)` adds `n` units to the fighter's gauge (a negative `n` drains), kept between 0 and the body's `max`; filling it
  starts the powered mode; while the mode runs (or for a body without a gauge) it does nothing. A charge that fills the gauge calls it
  every frame (the champion's charge: `boost_add(0.3)`).

* `lock(range)` is a target lock (the reference's reticle): once this attack has a lock it returns 1; otherwise it looks for the
  nearest other fighter (lowest index on a tie) that can be hit (in the match, stocks left, not intangible), **in front** of the
  fighter, whose chest (the middle hurtbox circle) is within `range` world units of the fighter's own, with no solid block across
  the line between the two chests (pass-through platforms do not block), and keeps where that chest is as `lock_x`, `lock_y`
  (`locked` 1). The point stays where it was found when the target moves; a new attack forgets it.

* `spawn_at` is `spawn` at an absolute point of the stage (the projectile appears there however the fighter moves this frame): a
  summoned explosion fixed where the fighter stood when the move began.

* `spawn_scaled` is `spawn` for a charged shot: the projectile also gets `knockback` (0 to 255) more base knockback and `size`
  percent (10 to 400) of its hitbox's radius (the reference's water pump grows with its charge).

* `air_flag(n)` is 1 if once-per-airtime flag `n` (0 to 7) is set; `set_air_flag(n)` sets it. Landing, catching a ledge or being
  hit clears all eight, so a boost guarded by a flag happens once until the fighter is back on the ground (the reference's cape
  stall).

* `store(n)` keeps a charge of `n` (0 to 255) on the fighter after the move ends, read back by any later script as `stored`
  (the needles a fighter has charged and saved); a KO clears it. `store(0)` spends it.

* `set_vel` replaces the move's data motion and normal physics for that frame (like a `motion` segment); stop
  calling it and gravity and air drift resume. On the ground it follows the ground: it never pushes the fighter
  into it.
* `spawn` fires the move's `projectile` (its hitbox, damage and life) from the given offset with the given speed.
  A move with a `projectile` block also fires it by itself on its `frame`; give `frame 255` if only the script
  should decide. A second `spawn` or `spawn_at` on the same frame fires a second shot (E3h, sim 95: plain size and knockback);
  a third replaces the first.
* `goto(n)` switches to move slot `n` (0 `jab`, 1 `ftilt`, 2 `utilt`, 3 `dtilt`, 4 `dash_attack`, 5 `fsmash`, 6 `usmash`,
  7 `dsmash`, 8 `nair`, 9 `fair`, 10 `bair`, 11 `uair`, 12 `dair`, 13 `neutral_special`, 14 `side_special`, 15 `up_special`,
  16 `down_special`, 17 `jab2`, 18 `jab3`, 19 `grab`, 20 `dash_grab`, 21 `pummel`, 22 `fthrow`, 23 `bthrow`, 24 `uthrow`,
  25 `dthrow`, 26 `ledge_attack`, 27 `get_up_attack`, 28 to 37 `ext0` to `ext9`, 38 `pivot_grab`, 39 to 44 `ext10` to `ext15`). This is how a script makes a combo or a follow-up variant:
  hitboxes belong to moves, so a script chooses *which* move, it does not edit hitboxes. A move that sends itself
  to itself every frame stays in place until it is hit; that is the author's loop to avoid.
* `stall()` holds the move on its current frame this frame (a charge). The total is capped by the ruleset's
  `charge_frames`, so a script can never hold a fighter forever.
* `end()` finishes the move after this frame; `turn()` flips facing; `rehit()` lets the move's hitboxes hit the
  same fighters again; `intangible(n)` grants `n` frames of intangibility.

### Projectile scripts

Registers: `px`, `py`, `pvx`, `pvy` (absolute), `age`, `life`, `owner_x`, `owner_y`, `target_x`, `target_y`,
`has_target` (the nearest *active* fighter that is not the owner; the lowest index wins a tie), `owner_stick_x`, `owner_stick_y`
(sim 117: the owner's stick this frame, -1..1, x positive right, absolute: what the scamp's rocket barrels read to be steered for a
while after they fly off).
Functions: `set_vel(x, y)` (absolute, takes effect this frame), `kill()`.

### Both

`abs(v)`, `min(a, b)`, `max(a, b)`, `clamp(v, low, high)`, `sign(v)`, `floor(v)`, `sqrt(v)`, `sin(degrees)`, `cos(degrees)`.

### Mistakes are reported where you made them

`content-check` shows the file line of the script block and the line inside it:

```text
line 181: side_special of weapon `longsword`: fighter script, line 2: unknown name `foo` (counting from the first line of the script)
```

Line numbers inside a script count from its first non-blank line.

## Versioning and packaging (plan 7.5)

* `schema` versions the *file format*. A change that renames, removes or re-means a field bumps it and ships a
  migration in `sim-content/src/bundle.rs` (`MIGRATIONS`), which rewrites the old file's tree one version at a
  time on load, so old content keeps loading. Adding an optional field does not need a bump. A file with a newer
  schema than the build understands is refused with a clear message. Migrations are tested with a stand-in chain
  (`old_files_are_migrated_on_load`, `migrations_chain_in_order`). The first real one is schema 1 to 2 (sim v34 and v35): every
  fighter gained a required `crouch_height` and `dash_attack_friction`, and a schema 1 fighter that does not inherit gets 0.6 and
  half its `ground_friction` (tested by `a_schema_1_file_from_before_crouch_height_and_dash_attack_friction_still_loads`).
  Schema 2 to 3 (sim v54): a fighter's one `ground_accel` became the reference game's walk and run accelerations, a base and a
  share by the stick each (`walk_accel`, `walk_accel_stick`, `run_accel`, `run_accel_stick`); an old value becomes both bases with
  no stick share, which moves as it did (`a_schema_2_file_with_one_ground_accel_still_loads_and_moves_as_before`).
  Schema 3 to 4 (sim v64): fighters gained `wall_jump` (1 if the body kicks off walls with the stick flicked away from them, 0
  if not) and `wall_jump_vx` / `wall_jump_vy` (the speeds of that jump and of every wall-tech jump); an old fighter gets no wall jump
  and its air speed and midair jump speed for the wall-tech jump, as before
  (`a_schema_3_file_from_before_wall_jumps_still_loads_and_wall_techs_as_before`).
  Schema 4 to 5 (sim v66): a weapon's moveset grew from ten `ext` slots to sixteen (`ext10` to `ext15`, numbered 39 to 44 after
  `pivot_grab`, so no older slot or `goto` number moves). An older file has nothing to rewrite (its weapons simply leave the new
  slots empty), but a file that uses them cannot be read by an older build, hence the bump
  (`a_schema_4_file_from_before_sixteen_ext_slots_still_loads_unchanged`).
  Schema 5 to 6 (sim v85): fighters may have a `gauge { ... }` block (above), moves `limit_version`, `gauge_drain`, `start_speed_y`,
  `ledge_drop_front` / `ledge_drop_back` and `on_land`, hitboxes `half_height`. Every one defaults to the old behaviour (no gauge),
  so an older file has nothing to rewrite; the bump makes an older build refuse a file with a gauge block clearly
  (`a_schema_5_file_from_before_gauges_still_loads_unchanged`).
  Still schema 6 (sim v86, before schema 6 was released): hitboxes may have `trip` and `rebounds`, the ruleset `trip_lag`,
  `trip_max` and `trip_invuln`; every one defaults to the old behaviour. (If schema 6 has shipped by the time this merges, these
  belong in a schema 7 with a migration that rewrites nothing.)
  Still schema 6 (sim v87): a projectile may have a `burst { ... }` block (E3e), a move `motion_drift`, and a hitbox angle may be 366
  (the vortex, E28); every one defaults to the old behaviour, so an older file loads unchanged (the same note about schema 7 applies).
  Still schema 6 (sim v95): fighters may have `air_jump_drift`; moves `late_next` / `late_until`, `ledge_falling_only`,
  `drop_from` / `drop_frames` / `drop_while_down`; projectiles `bounce_keep_y`, `bounce_max_fall`; bursts `on_ground`, `on_expire`,
  `keep_hits`. Every one defaults to the old behaviour, so an older file loads unchanged.
  Still schema 6 (sim v90, the hauler's phase 2): the ruleset's `trip_base` (7) and `trip_base_kb` (55), the
  reference's base trip chance (owner decision 50): any hit of more than `trip_base_kb` knockback that would not lift a grounded
  fighter trips it `trip_base` percent of the time, and a hit with its own `trip` adds `trip_base` to it (a 40% hit trips 47% of the
  time); throws never trip. **This one changes older content** (a file without the field gets the standard 7, as the owner asked:
  every fighter trips as the reference's do); `trip_base 0` turns it off.
  Also (sim v90): hitboxes may have `effect bury`, moves a `charge_add { full add [floor] }` block, and the ruleset `bury_base` (55),
  `bury_percent` (0.5), `bury_kb` (1.5), `bury_behind_1` / `bury_behind_2` (8, 12), `bury_ahead_1` / `bury_ahead_2` (18, 22),
  `bury_break_kb` (90), `bury_exit_lag` (12) and `bury_depth` (0.5): the bury lasts `ceil(bury_base + bury_percent x percent +
  bury_kb x knockback)` frames, longer or shorter by the stock difference, mashing cuts it as a hold's, a hit of at least
  `bury_break_kb` frees and launches the fighter and a weaker one's launch is kept for when it ends, it comes out with `bury_exit_lag`
  frames of lag, and its hurtboxes sit `bury_depth` of its height lower. Every one defaults to the old behaviour (nothing buries
  without `effect bury`).
  Still schema 6 (sim v98): a throw may have a `carry { ... }` block (E7); it defaults to none, so an older file loads unchanged.
  Still schema 6 (sim v91): a hitbox may have `posvec_x`, `posvec_y` and `posvec_frames` (E34), a move `fall_speed`, `fall_accel`
  and `keep_charge`, an `air` block `speed` and `accel_stick`, a fighter a `passive_shield { ... }` block (E14); every one defaults
  to the old behaviour (the same note about schema 7 applies).
  Still schema 6 (sim v107): a move may have an `encase { ... }` block and a throw hitbox `effect encase` (E8); both default to
  none, so an older file loads unchanged.
  Schema 6 to 7 (sim v111, E4, owner decision 64): weapons may have `item { ... }` blocks, moves `pull_frame` / `pull_item` and
  `spend_item`, projectiles `lodge_item` and a `charged` block `times`, fighters an `item_throws { ... }` block, and the ruleset the
  item rules (`item_pick_frames`, `item_side_frames`, `item_up_frames`, `item_down_frames`, `item_dash_frames`, `item_drop_frames`,
  `item_side_speed`, `item_side_smash`, `item_side_angle`, `item_side_smash_angle`, `item_up_speed`, `item_up_smash`,
  `item_down_speed`, `item_down_smash`, `item_dash_speed`, `item_dash_angle`, `item_drop_speed`, `item_drop_angle`,
  `item_reach_front`, `item_reach_back`, `item_reach_up`, `item_hand_x`, `item_hand_y`; see `sim_core::content::Ruleset` for each,
  the standard values when absent). An older file makes no items, so nothing is rewritten; a fighter without `item_throws` gets the
  standard release frames (the bump makes an older build refuse a file with items clearly:
  `a_schema_6_file_from_before_items_still_loads_unchanged`).
  Still schema 7 (sim v123, the Dagger's phase 2): a move may have a `tether { ... }` block (E56) and `dodge_stale`, a hitbox
  `blockable`, a `part_intangible` block `mode`, and a hitbox's `angle` 365 now means half the attacker's momentum. Each defaults to
  the old behaviour (no tether, not a dodge, shields do not stop grabs, every window always), so an older file loads unchanged (no
  older content uses angle 365).
  Still schema 7 (sim v121, the Dagger's phase 1): a hitbox may have `mode` and `part` (E52), a move `armor_damage`, `armor_hitlag`,
  `gauge_gain` and a `curse { ... }` block (E53-E55), a projectile's `burst` block `on_shield false` (a shield takes the shot's hit and
  it is used up there, no burst), and a fighter's `gauge` block the partner rules (`per_frame`, `behind_*`, `crowd_*`, `mode_taken_*`,
  `ko_keep*`, `keeps`, `halts`). Each defaults to the old behaviour, so an older file loads unchanged.
  Still schema 7 (sim v119, the Popgun's phase 2): a hitbox may have `fixed_stun` and `unblockable`, a move a `latch { ... }` block
  (E49), `flag_version { ... }` blocks and `self_damage`. Each defaults to the old behaviour (the knockback's hitstun, shields block,
  no latch, no versions, no damage to the user), so an older file loads unchanged.
  Still schema 7 (sim v117, the Popgun): a hitbox may have `slip hard` / `slip soft` (E44), a move a `toss { ... }` block (E47) and
  `shed` (E48), an item `damage_add`, `uses` and a `trap { ... }` block (E45), and the ruleset `slip_lag` (30) and `slip_max` (90).
  Each defaults to the old behaviour (no slip, no toss, nothing shed, no modifier, no limit on uses, no trap), so an older file loads
  unchanged.
  Still schema 7 (sim v115): a move may have `cancel_from` (owner decision 70), `pocket { ... }` / `unpocket { ... }` blocks (E42)
  and `chops`; a projectile `pocket` and `water`; an item `pocketable` and a `placed { ... }` block (E43). Each defaults to the old
  behaviour (no cancel, no pocket, every shot and item pocketable only by a pocket that exists, no placed pieces), so an older file
  loads unchanged.
  Still schema 7 (sim v113): a hitbox may have `rules previous` (E40), a move `version { ... }` blocks (E39) and a `balloons { ... }`
  block (E41), a projectile's `charged` block `power_to` and its `burst` block `on_hit`, and the ruleset `prev_damage_mult` (1),
  `prev_hitlag_per_damage` (0.3846154), `prev_hitlag_base` (5), `prev_rage_max` (0.15) and `prev_tumble_hitstun` (1). Every one
  defaults to the old behaviour (no hit uses the previous rules unless it says so), so an older file loads unchanged.
  Still schema 6 (sim v102): a hitbox may have `half_width` (E38), a move `smash_version` (E3d), a projectile `drag`, `lodge`,
  `lodge_angle`, `kind`, `limit`, `catch_from`, `catch_move` and `charged { ... }` / `on_clank { ... }` blocks (E35, E3b, E3d); every
  one defaults to the old behaviour.
* `sim_version` is the simulation's behaviour version (`SIM_VERSION`). Content packed for another version is
  refused: the same numbers can play differently.
* `hash` fingerprints everything the simulation reads (sim version, parameters, movesets including script
  bytecode, stage, rules). A packed bundle whose content does not match its hash was edited after packing or is damaged.
  A file with no `hash` is *loose* and loads as written, which is the hand-editing workflow; `content-pack` stamps it.
  Comments, spacing, names and script comments never change the hash.
* Online, the handshake compares sim version and content hash and refuses a mismatch (`docs/NETPLAY.md`), so two
  players can only meet on identical content.
* Replays store the sim version and content hash they were recorded with and only play against the same pair.
  Keeping old simulations around for old replays is a per-release choice, not something the build does today.

## Making a custom fighter

1. `pftool content-export mine.pfc "My Pack"`, then open `mine.pfc`.
2. Add a fighter that inherits and overrides: `fighter racer { inherit brawler  walk_speed 3 }`.
3. Add a moveset: `weapon quickclaws { inherit claws  move jab { total_frames 10  hitbox { start 2 end 3 x 1 y 1 radius 1 damage 2 angle 361 bkb 10 kbg 10 } } }`, then `weapon quickclaws` in the fighter.
4. `pftool content-check mine.pfc`, fix what it lists, play with `--content=mine.pfc`.
5. When it is final, `content-pack mine.pfc mine_packed.pfc`.

Both players must load the same bundle to play online.

## What is not done

* The game loads one bundle at start-up. There is no in-game roster or bundle picker, and no bundle download.
* Cosmetics (colours, accessories, faces) are not in the bundle yet: they arrive with the editors (Phase 6).
* Stages are data only. Moving platforms and hazards (plan 7.2) and stage scripts do not exist yet.
* Scripts cannot change hitboxes, hurtboxes or ECBs, spawn more than one projectile per frame, read other
  fighters (only projectile scripts see a target), or use randomness. Each is a deliberate boundary to
  keep scripts safe under rollback, and can be widened one whitelist entry at a time.
* The built-in Rust roster still exists next to the file (and the test that keeps them equal). Retiring it
  means moving the remaining Rust-only users (tests) onto the file.
* `content-pack` rewrites a file in canonical form and drops comments, so pack to a different file.
* Balance guardrails are range checks only; no point budget or ranked limits yet (plan 7.4).
* Not tried: a real creator writing a fighter from this document. The inherit workflow and error messages are
  tested, but whether the format is pleasant to author is unproven until the Phase 6 editors drive it.
