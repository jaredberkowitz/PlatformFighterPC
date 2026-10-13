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
| any command with env `PF_CONTENT=<file>` | Run the fuzzers, `net-host`/`net-join`, replays against that bundle instead |

## Exit criterion: data plays identically to code

`content/base.pfc` *is* the hand-coded roster written out as data. Tests prove it:
`sim-content/tests/bundle.rs` (round trip, `loaded == hand-coded`, same checksum on every frame of 6 random
matches) and `tools/tests/content_files.rs` (the shipped file, 4 matches of 3000 frames). While the Rust builders
still exist the two must agree; if you change a number in code, re-export the file, and the test tells you.

## File format

```text
# comment to end of line
bundle {                         # the manifest, required
    schema 5
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
| `rise_ground`, `rise_air` | percent of the motion's vertical speed for a move started on the ground / in the air (the reference's height multipliers) | 100, 100 |
| `start_speed` | started in the air, percent of the horizontal speed kept | 100 |
| `steer_frame`, `steer_stick`, `steer_degrees` | angling by the stick: on that move frame, a forward or backward tilt past `steer_stick` percent turns the motion by up to `steer_degrees`, evenly to the rim (255 = no angling) | 255, 0, 0 |
| `air_gravity`, `air_max_fall`, `air_friction`, `air_drift` | the move's own air physics while it runs without a motion (world units; `air_drift` a percent of the body's air speed and acceleration) | the body's; `air_drift` 100 |
| `land_continue` | touching down does not end the move: it carries on along the ground | false |
| `part_intangible { start end [low] [middle] [high] [limb] }` | any number: on move frames `start..=end` the named hurtbox circles cannot be hit (`true` each; the body's low, middle and high circles, and `limb`, the arm or leg an attack throws out), while the rest of the body can: the reference's per-limb intangibility (a kicking leg, a headbutting head) | none |
| `charge_at` | smash attack hold frame | none |
| `next`, `next_window`, `next_from` | jab chain: move to continue into, and the press window; with `next_from` (a move frame) a press from that frame on carries on at once (the reference's continuable window), otherwise only at the end | none; `next_from` 255 |
| `rehit_start`, `rehit_every` | multi-hit: when hits may land again | none |
| `rehit_groups` | the hit groups the rehit rhythm re-arms, a bit per group (a looping phase is re-armed, its final hit is not) | 255 (all) |
| `landing_move` | a move (usually an `ext` slot) started when this aerial lands outside its autocancel windows: a landing hit; its length is the landing lag | none |
| `hitbox { start end x y radius damage angle bkb kbg [priority] [group] [kind] [shield_damage] [hitlag] [effect] [targets] [fixed_kb] [hitstun_add] [sdi] [link_ahead] [aim] [set_weight] [reverse] [push] [shieldstun] [angle_type] }` | any number; `angle_type` is which way 0 degrees points (the reference's angle type): `box` (default: a box behind the attacker sends backward, any other forward), `forward` (the way the attacker faces), `backward` or `position` (away from the attacker, from the side the victim is on); for a projectile the attacker is the projectile (its direction of travel and its position); `shieldstun` is the percent of the normal shield stun the hit causes (the reference's per-hitbox shieldstun multiplier, applied to the stun's damage part; default 100); `kind` is `normal grab throw pummel`; `shield_damage` is the percent of its damage a shield takes (default 100); `hitlag` the percent of the normal hitlag (default 100); `effect` is `normal` or `electric`; `group` 0 to 7 (each group hits a target once per move, or once per rehit); `targets` is `all`, `grounded` or `aerial`; `fixed_kb` set knockback (the same at any percent; 0 = normal); `hitstun_add` extra hitstun frames (negative takes frames away; a launched victim keeps at least one); `sdi` the percent of the normal survival-DI distance (default 100). `link_ahead` moves an autolink's pull point that far in front of the hitbox (world units). `aim` is `any` (default), `up`, `straight` or `down`: the hitbox is only out when the move was aimed that way as it started (a forward tilt or smash with the stick tilted up or down, as the reference's angled moves). `set_weight` (`true`/`false`, default false) works the knockback out as if the victim weighed 100 (the reference's set weight). `reverse` (default false) turns the victim round (facing away from the attacker, horizontal speed reversed: the reference's reverse effect). `push` (default false) makes the hit a shove (the reference's flinchless hit): its knockback is added to the victim's speed (only sideways on the ground) with no hitlag, no hitstun and no change of state; a shield takes it as pushback. `angle` 361 is the reference's 361 angle (set by the knockback and by whether the victim is on the ground), 367 the autolink angle (an airborne victim is carried along at the attacker's velocity and pulled toward the hitbox; a grounded one goes up at 80) and 368 the pull angle (ours: no launch; the victim is drawn to just in front of the attacker, or a projectile's owner, while its hitstun lasts) | |
| `motion { start end vx vy }` | scripted-motion segments (`vx` forward-relative) | |
| `projectile { frame x y speed life end_damage [angle] [gravity] [max_fall] [bounces] [bounce_min] [bounce_max] [bounce_keep] hitbox { ... } [phase { from hitbox { ... } }] }` | at most one. Its flight: fired `angle` degrees above its facing (negative is down), pulled down by `gravity` to at most `max_fall` (0: no limit); with `bounces` above 0 it bounces off the top of any platform that many times (the next touch removes it), coming off at its falling speed held between `bounce_min` and `bounce_max`, keeping `bounce_keep` percent of its forward speed. Up to two `phase` blocks replace its hit from age `from` on (in order; a phased projectile's damage does not fall off) | angle 0, gravity 0, max_fall 0, bounces 0, bounce_keep 100 |
| `reflector { start end x y radius damage_percent speed_percent [reverse_vertical] [life_percent] [max_damage] [detect_start detect_end detect_x detect_y detect_radius] }` | at most one. `reverse_vertical` turns the shot's vertical speed round too; a reflected shot gets `life_percent` of its lifetime; a shot of more than `max_damage` (0: no limit) goes through; a shot touching the detector circle on `detect_start..=detect_end` skips the move ahead so the reflector is up the next frame | false, 100, 0, no detector |
| `counter { start end then percent min_damage }` | a counter stance: a hit landing in move frames `start..=end` is cancelled and the fighter switches to the move `then` (an `ext` slot), turned toward the attacker | |
| `counter_strike` | this move's hitboxes deal what the stance caught: `percent` of the caught hit's damage, at least `min_damage` | false |
| `charge_bonus` | extra damage at full charge for this move, in percent (0 uses the ruleset's `charge_bonus_percent`) | 0 |
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
`hit`, `stored`, `jump`, `presses`, `releases`. Horizontal speed and `stick_x` are **relative to facing** (positive is forward); positions are absolute.
`attack_tap` and `special_tap` are 1 only on the frame the button went down; to remember a tap, store it in a `var`.
`hit` is 1 once the move has connected. `stored` is the charge the fighter kept from an earlier use (see `store`).
`presses` and `releases` count the attack button's presses and releases since the attack began (the move a button started;
moves reached from it by `next`, `goto` or a counter keep counting), **hitlag included**, which no `attack_tap` can see; the press
that started an attack is 1 (and a release of it already made counts 1; an attack started by another button starts at 0). A rapid jab's entry rule reads them (the reference's "pressed
3 times and released twice since jab 1").

Functions: `set_vel(forward, up)`, `add_vel(forward, up)`, `spawn(forward, up, speed_forward, speed_up)`,
`intangible(frames)`, `end()`, `turn()`, `rehit()`, `stall()`, `goto(move)`, `store(n)`, `air_flag(n)`, `set_air_flag(n)`,
`spawn_scaled(forward, up, speed_forward, speed_up, knockback, size)`.

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
  should decide. One shot per fighter per frame.
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
`has_target` (the nearest *active* fighter that is not the owner; the lowest index wins a tie).
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
