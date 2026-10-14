# Project audit for the eighth moveset (reference: the reference game's Donkey Kong)

What the engine needs to reproduce the reference game's Donkey Kong 1:1 (version 13.0.1), traced through the code at **sim version 83**
(content schema 5, branch `worktree-agent-a5386608fff089d47` at `0e72bcd`, which contains `a88c09f`). Nothing is built: this is research
only. The data is in `REFERENCE_DATA.md`; the engine inventory is `docs/movesets/SHARED_MECHANICS.md`; the extension ids are those of
`docs/movesets/ROSTER_STATUS.md` and `docs/movesets/greatsword/PROJECT_AUDIT.md` (E22b, E28, the E11 `start_speed_y`), with new ones numbered
from E29. The owner decisions in `ROSTER_STATUS.md` go up to **33** (the task brief said 1-36; 34-36 are not in any committed copy or
the main checkout: question 12).

## Proposed names (not registered)

| Thing | Name | Checked |
| --- | --- | --- |
| Class / weapon | **Bracers** (his arms are the weapon: most of his moves make the forearm intangible) | `content/blocklist.txt` has no terms (comments only); contains none of the `TRADEMARKS` terms in `sim-content/src/policy.rs` after its look-alike folding; not used anywhere in the code, content, `godot/scripts` or docs; not a reference fighter's name |
| Body | **hauler** (the cargo hold) | same |
| Built-in fighter | **Hauler** | same |
| Blurb | "Huge arms and a heavy frame: a wind-up punch you can save for later, a headbutt that plants foes in the ground, a spinning clothesline, ground slaps, and a throw that carries the catch anywhere, even off the stage." | original wording |

Alternatives if the owner prefers: weapon "Wraps", body "titan". Avoided on purpose: the reference's move names ("Giant Punch",
"Kong Karry" and the rest) and anything with the series' word for the character. Class index 7 when registered (after the Greatsword at
6; `CLASSES` becomes 8). This body has **no weapon object** (`Weapon::limbs` true, like the Claws and the Gauntlets), so E19's limb
circle is a real forearm here: the arm-intangibility windows act (unlike the Staff's and the Greatsword's).

## Iconic moves: what must match exactly

1. **Giant Punch** (neutral special). The 7-frame start into a charge that runs by itself for **110 frames**; special releases it at
   any point with **`floor(base + 18 x charge / 110)`** damage (10 → 27 grounded, 6 → 23 aerial) on the uncharged knockback (BK 45 KS 70 /
   BK 60 KS 65), hitbox on the release's 19-20, FAF 63 from the release, the lower arm intangible 4-20 of the release; shield, roll, spot
   dodge and jump (air dodge in the air) **cancel and keep** the charge after **4 frames**; a hit loses it; a KO loses it; at 110 the
   charge **stores itself** and the next press is the **full punch**: 28% (18% the waist box) BK 35 KS 73 grounded, 25% BK 40 KS 72
   aerial, hitbox 19-20, FAF 48, **super armor 9-20 grounded / 15-20 aerial**, landing in it switches to the grounded window; no
   helplessness in the air; the turnaround start. **Engine**: the charge-and-store pattern and the `goto` loop exist (the Embers' pump);
   super armor exists (E1); the arm window exists (E19, a real limb on this body). Missing: **additive charge damage over a per-move
   charge length** (new **E29**), the landing switch (**E22b**, proposed for Cloud), the no-rebound flag (new **E3g**, small).
2. **The cargo hold** (forward throw) and its **four cargo throws**. The 15-frame invincible lift; walking at the carry's speed (x1.1:
   conflict 5) with the victim overhead, one jump, carried off the stage; the clock **`(90 + 1.7p) x 11 / 6 - g`** with the grab's mashing
   (8 a stick input, 14 a button) carried over; knockback armor from frame 16 until a throw starts (threshold 16 or 80: conflict 2);
   attack + a direction throws: forward (release 15, FAF 46, 12% at 60, BK 90 KS 46), back (turns and releases on 16, FAF 43, 13% at 72,
   BK 85 KS 52), up (15, FAF 30, 12% at 92, BK 65 KS 53), down (17, FAF 45, 11% at 46, BK 70 KS 45, and on frame 17 the horizontal speed
   stops and the vertical speed becomes 1.5). **Engine**: none of the carry exists (**E7**, broad: a new holder state with its own
   movement); the knockback armor kind is reserved but not built (**E1 addition**).
3. **Spinning Kong** (up special). Grounded: super armor **5-17**, the lower arm intangible 19-24, hit 1 on 19 (grounded boxes 95 / 80
   FKV 90, aerial boxes at **366**), five loop hits on 25-27, 32-33, 40-42, 49-50, 55-58 (1.4 / 1.4 / 3.5, 367 forward, 20 forward, 60
   backward), the 4% final on 62 (BK 60 KS 157, H 2.0), FAF 105, the stick-steered slide (accel 0.034, friction 0.015, max 1.1, friction
   0.04 after the last hit). Aerial: hit 1 on 4-6, twelve 1% autolink hits on their listed frames, the 2% final on 39-43 (BK 72 KS 80),
   the lower arms intangible 10-38, the rise (0.9 up, accel 0.05, max 1.7, the gravity reading: conflict 6), ledges both sides from 14,
   70 frames then the special fall with **38** frames of landing lag; landing before the fall goes to the grounded version's frame 63.
   **Engine**: everything exists (scripts for the slide and the irregular rehits, E19, E1 super armor, E11 ledge windows and landing
   lag, `landing_lag` for the early landing) except angle **366** (**E28**, proposed for Cloud; 367 as a labelled approximation) and the
   "immune to most windboxes" rule (question 7).
4. **Headbutt** (side special) and its **bury**. Hitbox 20-21, super armor **5-14**, FAF 63; grounded: two **grounded-only bury boxes**
   at 270 (10%, BK 40 KS 40) and an aerial-only 8% box at 70; aerial: one bury box at 270 hitting anyone (a **meteor** on airborne
   targets, a bury on grounded ones) and the 8% box; shield damage +25; the start's speed multipliers (x0.5 / x0.67 sideways, x0 / x0.5
   up) and the gravity x0.5; ledges from frame 2 (conflict 4). The bury: `ceil(55 + 0.5p + 1.5k)` frames (plus the stock-difference
   terms), mashing 8 / 14.4, the automatic pop-out with 12 frames of lag, unburied by a hit of at least 90 units of knockback (launched in
   full), weaker hits' knockback stored for the end, no grabs on a buried fighter, nothing buries a fighter lying down. **Engine**: the
   move is data plus E1 super armor and the E11 `start_speed_y` (proposed for Cloud); **the bury does not exist** (**E5**, a new victim
   state).
5. **Hand Slap** (down special). Grounded: four 14% grounded-only boxes (r 10.3, 78 / 84) on both sides on 12-13 and 23-24, a 26-frame
   loop repeated by presses (twice for a press in the loop's 20-26), the ending's FAF 47 (13 in), H 0.5, shieldstun x0.58, no clank.
   Aerial: 5% FKV 36 at 90 on 19-21, then the 6% meteor at 270 on 28-30, FAF 50, landing lag 10, no clank. **Engine**: exists (scripts
   with `special_tap`, `Hitbox::targets`, `shieldstun`, `clanks`). The 20% trips are E20 (question 4).
6. **Back air** (13% clean 7-8, 8% late 9-16, angle type 4, landing lag 11, autocancel 1-6 / 31+) and the meteors (**forward air**'s late
   fist on 21-23 at 270; **down air**'s feet on 14-16 at 270; both hit grounded targets too). **Engine**: data.

Lesser points that must also be on [R]: the jab's continuable window 8-28 and jab 2's 13.0.1 frames; the forward tilt's three aims
(8% straight, 9% up / down, FAF 35 / 36, the down one's trip); the up tilt's three-strength sweep; the down tilt's +5 hitstun; the smashes'
head and arm intangibility; the pummel's 15-frame hitlag (H 2.3); the frame-4 air dodge; the large hurtbox.

## How the moves flow (traced)

As in the earlier audits: `fighter::update` → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` / `start_special` →
`begin_attack` → `fighter::attack` each frame → (after every fighter moved) `combat::resolve_hits` → `grab::update` →
`combat::update_projectiles` / `spawn_projectiles` → ledge phase → `combat::check_ko` (`sim-core/src/step.rs`). Points that matter here:

- **Armor** (`combat::apply_hit`, sim 67): after the damage is added and after a push has returned (so a push still pushes, as the
  reference's super armor), `if def.state == S::Attack && weapon.get(def.move_id).armor.is_some_and(|a| a.withstands(def.state_frame,
  hb.damage))` the victim takes only the hitlag. `Armor { start, end, kind, threshold }`, `ARMOR_SUPER` (0) and `ARMOR_DAMAGE` (1) are
  built and tested (`multihit::damage_armor_withstands_hits_up_to_its_threshold_and_super_armor_every_hit`); `ARMOR_KNOCKBACK` (2) is
  **reserved and refused by validation** (`sim-content/src/lib.rs`: "knockback armor is not built yet"). So, correcting the task brief,
  **super armor is built**; the knockback kind is the reserved one. Limits that matter: **one window per move** (DK needs one per move:
  fine, every armoured move is its own slot), **only in `S::Attack`** (the carry is not an attack: E7 must extend it), and
  `withstands` sees only the damage (the knockback kind needs the knockback: E1 addition). Grabs never reach `apply_hit` (they are
  resolved as `HIT_GRAB` in `resolve_hits`), so "grabs ignore armor" holds.
- **Ordinary grabs and throws** (`grab.rs`, `fighter::grabbing`): `grab::connect` puts the holder in `S::Grabbing` (`become_holder`
  stops its horizontal speed) and the victim in `S::Grabbed` with `grab_timer = 90 + 1.7 x percent` (`hold_time`, Ruleset
  `grab_base_frames` 90, `grab_percent_tenths` 17). `grab::update` pins the victim `grab_distance` in front on the holder's platform and
  **runs the clock only while the holder is in `Grabbing`** (not mid-pummel or mid-throw), cutting `1 + 14` a button press and `+ 8` a
  stick flick (`grab_mash_button` 14, `grab_mash_stick` 8: the reference's 14.4 rounded and 8); at 0 `drop_grab` frees both
  (`free_holder` with `grab_release_lag` 25, `free_held` with `grab_immunity` 60). `fighter::grabbing` reads the stick (up, down,
  forward, back) into `UThrow` / `DThrow` / `FThrow` / `BThrow` and attack into `Pummel`. The holder in `Grabbing` cannot move. A hit on
  the holder sends it to `Hitstun`, and the next `grab::update` finds the pair inconsistent and drops the grab. `check_ko` drops the grab
  of a fighter leaving the blast zone before respawning it.
- **Command grabs and `hold_at`** (E6, sim 69): a `Hitbox::command` grab box catches through `grab::catchable`; with `Move::catch_move`
  the holder stays in `S::Attack` (its speed and air physics kept) and the victim is `Grabbed` **with no clock and no mash-out**;
  `Move::hold_at` pins the victim at (forward, up) from the holder's feet, in the air if the holder is (`grab::pin`). This is the closest
  thing to a carry, but its victim cannot escape and its holder moves only by the move's own motion or script, never by walking.
- **Charge and store** (`fighter::attack`, `scripting.rs`): a smash's `charge_at` holds the frame while attack is held, up to
  `Ruleset::charge_frames` (60), and `combat::resolve_hits` multiplies the damage by `1 + charge x bonus / (100 x 60)`
  (`Move::charge_bonus` or `Ruleset::charge_bonus_percent` 40), reading the attacker's hashed `Fighter::charge: u8`. A script's `stall()`
  is capped by the same 60 frames per move; **`goto` clears `charge` and `hit_mask` but keeps the script vars**, so a charge longer
  than 60 frames loops by `goto`-ing its own slot and counts in a var: the Embers' water pump does exactly this for 80 frames
  (`sim-core/src/scripts/pump_charge.script`: `c = min(c + 1, 80)`, `goto(29)` from frame 90; shield or jump → `store(c)` and `end()`;
  special → the shot). `store(n)` / `stored` (`Fighter::stored: u8`, hashed, cleared by a KO) keeps a charge between uses. So the
  110-frame Giant Punch charge needs **no change to the stall cap**: it is the pump pattern. What does not exist is a way to turn a
  script-held charge into **added** damage: `charge` feeds a **multiplicative** bonus over the ruleset's 60 frames, and a `goto` into
  the release resets it to 0.
- **Specials** (`fighter::start_special`): picks the direction slot, swaps to `Move::air_version` in the air, `begin_attack`; in the air
  `start_speed` scales the horizontal speed (no vertical counterpart yet; no grounded counterpart: the Greatsword's audit uses a frame-0
  `set_vel` for a grounded slow-down). The B-reverse and up-special turn windows are shared.
- **Hit effects** (`Hitbox::effect`): only `EFFECT_NORMAL` and `EFFECT_ELECTRIC`. Victim states on hit: hitstun, flinch, push, reverse;
  **no bury, no trip** (`SHARED_MECHANICS.md` section 2).
- **Hurtboxes** (`combat::hurtboxes`): four circles from the body's ECB (low, middle, high, and the limb reaching toward the move's hits
  with `Weapon::limbs`); a crouch lowers them. A buried fighter would need its own placement.
- **Fighter states** (`state.rs` `FighterState`, hashed as a `u8`): `Grabbing` / `Grabbed` exist; there is no carrying or buried state.
  `step.rs`'s `random_play_keeps_invariants` checks that grabs are mutual and that `Grabbing` / `Grabbed` are grounded states unless held
  by a command grab: a carry in the air and a buried state must be added to it. The view maps state names to clips
  (`godot/scripts/anim_library.gd`, by the `Debug` name the bridge sends): two new states need entries (the art agent's).

## Normals: what exists

| Needed | Exists? | Decision |
| --- | --- | --- |
| Body: walk accel 0.105 + 0.105, run 0.044 + 0.07337, initial dash 2.09 for 15 frames, run 1.873, traction 0.123, air 1.208 with 0.01 + 0.05, air friction 0.015, gravity 0.085, fall 1.63 / 2.608, hops 34 / 17.3 / 35.5, ledge jump 37, heavy landing 3, weight 127, dodges (UFD: spot 26, rolls 30 / 35, air dodge 48 and 66 / 82 / 109, intangible from 4), ledge decay 80 / 48 | yes (`FighterParams`) | `FighterParams::hauler`, as `champion` / `sprinter`; jump air times 40 / 53 / 27 / 37 checked by test; no wall jump (`wall_jump` 0), the wall-tech jump 1.3 / 2.7; a larger ECB [E] |
| Jab 1 → 2 with the continuable window 8-28 | yes (`next`, `next_from`, `next_window`) | data; jab 1's fighters-only 180 box as the Greatsword's |
| Forward tilt aimed up / straight / down with different damage | yes (`Hitbox::aim`, the Embers') | data; **the aimed versions' FAF 36 against 35 straight** cannot differ inside one move (a listed one-frame limit, or a one-line script `end()` at 35 when not aimed: proposed) |
| Forward tilt down's 15% trip, down tilt's 40% trip, aerial Hand Slap's 20% trip | **no** (E20) | question 4 |
| Down tilt +5 hitstun | yes (`hitstun_add`) | data |
| Arm intangibility on the tilts and smashes, head on the smashes and up air | yes (E19 `PART_LIMB`, `PART_HIGH`; a real limb on this body) | data |
| Smashes and Giant Punch "cannot rebound" (`rebound=f` without `clang=f`) | **no** | new **E3g** (small): `Hitbox::rebounds: bool` (default true); in `combat::clank_attacks`, a fighter whose clanking box has `rebounds` false keeps going instead of `start_rebound`. Or a listed limit: question 6 |
| Aerials: sweet / sour phases, the late-fist and feet meteors (no target restriction), the back air backward, landing lags 10 / 17 / 11 / 15 / 14 and autocancels | yes | data |
| Grabs (standing / dash / pivot), pummel on frame 2 with H 2.3 | yes (no house shift needed: the reference's hit is already on 2) | data |
| Back, up, down throws (no pre-hits; the back throw turns on 15) | yes (`HIT_THROW`, `turns_around`) | data |
| Forward throw = the carry | **no** | E7 (below); phase 1 leaves `FThrow` as a placeholder (the cargo forward throw's numbers as a plain throw [E], replaced in phase 3) |
| Ledge attack, get-up attacks (face up / face down / trip) | yes (one get-up slot) | the face-up one; the other two are the known limit |
| No-clank boxes (pummel, ledge and get-up attacks, Hand Slap) | yes (E3c) | data |

## Specials: what exists, and the exact shape of each needed extension

### Giant Punch: E29 (additive charge damage), E22b (landing switch), E3g; the rest exists

* **The charge** (the neutral special slot): a script on the pump pattern. Frame 0: if `stored >= 110` → `goto(full)`; else `c =
  stored`, `store(0)` (a hit now loses everything, as the reference). From the loop's first frame [gap 1] `c = min(c + 1, 110)` each
  frame and `goto` its own slot before its length runs out; `special_tap` → `goto(release ground)` or `goto(release air)` by `grounded`;
  `shield` (also the start of a roll or spot dodge: shield with the stick) or `jump` (in the air `shield` = the air dodge) → `store(c)`
  and `goto(cancel)`; `c == 110` → `store(110)` and `end()` (the self-storing full charge). Hitlag freezes it as any move.
* **The cancel** (`ext`, 4 frames, no hitbox): ends at its FAF so the held shield, a buffered jump or the dodge comes out (the 9-frame
  input buffer covers a press made on its first frame) [E: the reference picks "the selected action" at the cancel's start].
* **E29 (new, small): additive charge damage with a per-move charge length.** Shape: `Move::charge_add: Option<ChargeAdd>` with
  `ChargeAdd { full: u8, add: Fx, floor: bool }` (default none: today's multiplicative smash bonus), and a script function
  `set_charge(n)` (0 to 255) that sets the hashed `Fighter::charge` (already in `hash_into`) on the release move after its `goto`. In
  `resolve_hits`, when the attacking move has `charge_add`, the damage is `base + add x min(charge, full) / full`, floored to a whole
  percent if `floor` (the reference: "rounded down"), instead of the multiplicative bonus; the knockback uses that damage as today. The
  Giant Punch release: `full` 110, `add` 18, `floor` true; its script's frame 0 calls `set_charge(c)` from the var the loop kept (vars
  survive the `goto`). Content: a `charge_add { full add floor }` block on the move, `docs/CONTENT.md` lines, `Weapon::hash_into`;
  validation (`full` > 0); a whitelist entry in `sim-script/src/api.rs` with its test in `scripted_moves.rs` and the doc line. Tests:
  10 / 19 / 27% at charge 0 / 55 / 109 (floor), aerial 6 → 23; a stored partial charge releases with its damage; a hit during the charge
  loses it. Why not reuse `charge_bonus`: it multiplies and divides by the ruleset's 60, so 18% over 10% (x2.8 at 110) would need a
  per-move denominator and could not floor; an additive field says what the reference says.
* **The releases** (`ext` grounded and aerial): the uncharged rows, hitbox on the release's 19-20, FAF 63, `part_intangible` limb 4-20,
  no armor; the aerial one's `on_land` = the grounded one (E22b). **The full punches** (`ext` grounded and aerial): `armor` super 9-20 /
  15-20, the 28 / 18 and 25% rows, FAF 48; the aerial one's `on_land` = the grounded one, which keeps the frame, so landing on frame 11
  gives armor from 11 (the reference's behaviour, dragdown) [R reading].
* **E22b** (proposed in `docs/movesets/greatsword/PROJECT_AUDIT.md`: `Move::on_land: Option<u8>`, an aerial move touching down switches to
  that slot on the same frame, keeping its frame, speed and hits). If Cloud's phase 2 builds it first, it is reused as is. Without it
  the aerial punch would end on landing (the shared landing) and lose the grounded armor window [E].
* **No helplessness** after the aerial punch (`helpless_after` false). Turnaround: the shared special-turn window for the start [E: gap
  2]. Air physics during the charge and the punch: the body's [E: gap 6].
* **E3g** (no rebound) on the punches' boxes: question 6.

### Headbutt: data, E1 super armor, E11 `start_speed_y`, E5 bury

* Two slots (the side special and its `air_version`). Super armor 5-14 (`ARMOR_SUPER`, exists); hitbox 20-21; FAF 63; shield damage
  350 / 413 [D]; `targets` grounded on the bury boxes and aerial on the 70-degree box (grounded version), the aerial bury box for anyone.
* Movement: aerial `start_speed` 67 (exists); **E11 `start_speed_y`** 50 aerial (proposed for Cloud: percent of the vertical speed kept
  on an aerial start); grounded x0.5 horizontal and x0 vertical by a frame-0 `set_vel` [E: as the Greatsword's beam]; `air_gravity`
  0.0425 reference units (→ 0.0053 world [D]: the body's 0.085 x 0.5).
* Ledges: `ledge_front_from` 1 (the reference's frame 2) and `ledge_back_from` [E: the same] (conflict 4). "Hitting as he grabs the
  ledge cuts the ending lag": a ledge grab already ends the move here [D].
* **E5 (new, broad: a new victim state; explain to the owner before building).** Shape (every default leaves existing content and
  fighters unchanged):
  * **Content**: a hit effect `EFFECT_BURY` (= 2, content `effect bury`) on `Hitbox::effect`. Ruleset numbers (`Ruleset`,
    `sim-core/src/content.rs`, in its `hash_into` and the content format, with defaults): `bury_base` 55, `bury_percent` 0.5, `bury_kb`
    1.5, `bury_behind` [8, 12] and `bury_ahead` [18, 22] (frames added / taken off by the user's stock difference), `bury_break_kb` 90,
    `bury_exit_lag` 12, `bury_exit_vy` [E: gap 9]; the mash cuts reuse `grab_mash_button` / `grab_mash_stick`.
  * **State**: a new `FighterState::Buried`; hashed `Fighter` fields `bury_timer: u16` and the stored launch of weaker hits
    `bury_kb: Fx`, `bury_angle: i16`, `bury_set: bool` (each in `Fighter::hash_into` with a checksum case in `state.rs`'s tests).
  * **Entering** (`combat::apply_hit`, after the damage and before the armor test): an `EFFECT_BURY` hit on a victim that is grounded
    and not lying down (`Knockdown`, `GetUp`), not already buried, not shielding (shields return earlier through `block`), not held,
    makes it `Buried` instead of launching it: `bury_timer = ceil(bury_base + bury_percent x percent + bury_kb x k)` plus or minus the
    stock terms (`percent` after the hit with the 1.2 multiplier; `k` the hit's knockback at weight 100: `combat::knockback` with weight
    100, before crouch cancelling), its hitlag as usual, its speed 0. An airborne victim (or one that cannot be buried) takes the hit as
    usual: the aerial Headbutt's 270-degree meteor needs nothing more. A hitbox's bury ignores a subtractive knockback armor (the
    reference's rule) and super armor? [E: the reference's super armor withstands it; nobody in this kit is buried while armoured].
  * **While buried** (`fighter::update`, a `buried(f, ...)` arm): no action; outside hitlag the timer counts down by 1 plus the mash cuts
    (the `grab::update` reading of presses and flicks, shared as a helper); at 0 the pop-out: a jump at `bury_exit_vy` [E] into a
    no-action window of `bury_exit_lag` (12) frames, then free; if a launch was stored, it is applied instead of the pop-out [E: gap 9].
    `grab::grabbable` and `grab::catchable` refuse a buried fighter. `combat::hurtboxes` lowers the circles for `Buried` (the low one
    under the ground, the others down by a body share `bury_depth` [E: gap 15]).
  * **Hit while buried** (`apply_hit`): knockback of at least `bury_break_kb` → leaves the bury and is launched in full (the ordinary
    path); weaker → damage and hitlag only, and its knockback and angle go into `bury_kb` / `bury_angle` / `bury_set` (the last one kept
    [E: gap 10]); a push still pushes? [E: it stays put; a push does not unbury].
  * **Platforms, edges**: a buried fighter stays put (no moving platforms here).
  * **Rollback and the wire**: everything is `GameState`; inputs only on the wire (`pftool net-fuzz` should stay at zero; run it).
  * **Tests** (a shared `bury.rs` and the fighter's file): the duration at 0% and 100% against the formula (even stocks, one behind, one
    ahead), mashing shortens it by 8 a flick and 14 a press, the pop-out's 12 frames, no grab on a buried fighter, a lying fighter not
    buried, an airborne one meteored, a 90-unit hit unburying and launching in full, a weak hit's knockback applied at the end, the
    hurtboxes lowered, checksum cases for the four fields, a rollback replay through a bury; every other fighter unchanged (`base.pfc`
    equality, the kit tests).

### Spinning Kong: exists (scripts and data), plus E28 or 367

* Grounded (up special slot): `armor` super 5-17; `part_intangible` limb 19-24; hit 1 on 19 in its own groups (grounded boxes
  `targets` grounded, the three aerial boxes at 366: **E28** if Cloud's phase builds it, else **367** as a labelled approximation:
  question 5); the loop hits in groups re-armed by a script `rehit()` on 25, 32, 40, 49, 55 (the irregular rhythm `Move::rehit` cannot
  express; scripts can); angle types 3 / 3 / 4; the final hit's group never re-armed; H 0.8 / 2.0; FAF 105. The slide: a script
  `set_vel` each frame from the stick, the body speed and the reference's accel 0.034 / friction 0.015 / max 1.1 / friction 0.04 after
  62 (world units [D]). Going off an edge: gap 7 (E22 `off_edge` exists if it turns out to switch versions).
* Aerial (`air_version` `ext`): hit 1 on 4-6, the twelve 1% hits by script `rehit()` on 10, 13, 15, 18, 20, 22, 25, 27, 30, 32, 34, 37
  (one group, re-armed), the final in its own group; `part_intangible` limb 10-38 (both arms are one limb circle here: the reference's
  "lower arms" [D]); the rise: `start_speed_y`-free, a frame-0 `set_vel(.., 0.1125)` then E11 `air_gravity` (conflict 6), `air_drift` /
  `air_blocks` with the accel 0.05 and max 1.7 [D]; `ledge_front_from` / `ledge_back_from` 13 (the reference's 14); 70 frames then
  `helpless_after` with `fall_landing_lag` 38; touching down before the fall: `landing_lag` 42 on the move (the grounded version's
  frames 63-105) [D]: the house `land_out_of_move` path, no slot needed.
* "Immune to most windboxes": not modelled (a push always pushes); question 7 (a `Move::push_immune` window would be the general shape).

### Hand Slap: exists (scripts and data)

* Grounded: the down special slot (startup and the first loop) → a loop `ext` that `goto`s itself every 26 frames while presses are
  banked → an ending `ext` (13 frames to FAF). The script banks `special_tap` presses from the startup on, and a press in the loop's
  20-26 counts twice (the reference's quirk) [R]. Each slap in its own group, re-armed each loop (`rehit()`); four boxes 14% at 78 / 84,
  r 13, `targets` grounded, `hitlag` 50, `shieldstun` 58, `shield_damage` 107, `clanks` false; positions at y 5 and z ±6.1 / ±21.1 [D
  reading]. Not modelled: the counter oddity (`direct=f`); the "slap slip" edge cancel follows the engine's own edge rules.
* Aerial (`air_version` `ext`): the two claps in two groups, FAF 50, `landing_lag` 10, no clank; air physics [E: gap 8].

### The cargo hold: E7 (new, broad: a new holder state; explain to the owner before building), E1 knockback armor

**What exists**: the ordinary hold (a holder that cannot move, a victim pinned in front, a clock only while holding) and E6's command
hold (a holder in a move, a victim pinned at `hold_at` with no clock). Neither lets the holder walk, jump or fall with a victim who can
still mash out.

**The exact shape** (every default leaves existing content and fighters unchanged):

* **Content, on the move** (`Move`, `sim-core/src/moves.rs`): `carry: Option<Carry>` on the forward throw (default none: an ordinary
  throw) with `Carry { walk: u16 (percent of the walk speed and accelerations, 110), clock_num: u8, clock_den: u8 (11 / 6), jumps: u8
  (1), hold_at: (Fx, Fx) (the victim overhead [E]), armor: Option<Armor> (the knockback armor, conflict 2), throws: [u8; 4] (the `ext`
  slots of the forward, back, up and down cargo throws), ledge: bool (false [E: gap 4]) }`. The forward throw move itself is the
  15-frame lift (`intangible` 15, no hitbox), and its end enters the carry instead of ending the grab. Content: a `carry { ... }` block,
  `docs/CONTENT.md` lines, `Weapon::hash_into`, validation (the throw slots are `ext` moves with a `HIT_THROW` row).
* **State, on the fighter** (`Fighter`, `sim-core/src/state.rs`, each hashed with a checksum case): a new `FighterState::Carrying` for
  the holder (on the ground and in the air), the victim staying `Grabbed`; `carry_jumps: u8` (jumps left in this carry); and on the
  victim `grab_start: u16` (the clock's starting value, set by `grab::connect`), so that the grab time passed is `g = grab_start -
  grab_timer` (mashing included, as the reference's `g`).
* **Entering**: when the lift move ends, `grab_timer = hold_time(percent now) x 11 / 6 - g` (at least 1) and the holder enters
  `Carrying` with `carry_jumps = jumps`. (A pummel before the carry pauses the clock as today, so its frames are not in `g`, while its
  damage raises the carry's `p`: the reference's formula reads the same way [D].)
* **The clock** (`grab::update`): runs while the holder is in `Grabbing` **or `Carrying`**, with the same mash cuts; at 0
  `drop_grab` (the release lag and the victim's escape: gap 5, the ordinary release until then [E]).
* **The holder's movement** (`fighter::update`, a `carrying(f, p, ...)` arm): on the ground, the walk only (the stick's walk at `walk`
  percent of `walk_speed`, `walk_accel`, `walk_accel_stick`; no dash, run, crouch, shield, dodge or attack), turning with the stick;
  jump (if `carry_jumps` > 0: the ordinary jump squat and jump, spending one) staying `Carrying` in the air; walking off an edge stays
  `Carrying` and falls with the body's air physics (drift, fast fall [E: gap 3]); no air dodge, no midair jump beyond `carry_jumps`, no
  ledge grab unless `ledge`. Attack (with the stick's direction; attack alone forward) starts the matching cargo throw slot
  (`begin_attack`, which keeps `grab_with`): the throws are ordinary `Attack` moves with a `HIT_THROW` row, `hold_at` overhead until
  the release, `turns_around` on the back throw (frame 16), a one-frame motion on the down throw's frame 17 (vx 0, vy 0.1875 world
  [D]), `land_continue` if they land [E].
* **The victim's position** (`grab::pin`): in `Carrying` at `carry.hold_at` from the holder's feet (mirrored by facing), on the holder's
  platform while it stands, in the air with it (`held.platform` NONE). `grab::update`'s consistency test accepts a `Carrying` holder.
* **Armor** (E1 addition, below): `apply_hit` tests `carry.armor` when the defender is `Carrying` (the reference: from frame 16, after
  the lift's invincibility, "until he initiates a throw"; the cargo throws have none).
* **A hit over the armor**: the holder goes to `Hitstun` and the next `grab::update` drops the grab (exists); the victim takes the
  carry's interrupted row (4%, FKV 30) [R], not modelled (the house limit on interrupted rows).
* **Off the stage**: the pair falls; `check_ko` drops the grab of whoever leaves first (exists) [E: gap 13].
* **The view**: `Carrying` and a carried pose need clips (art agent); the bridge sends the state's name as for the others.
* **Rollback and the wire**: `GameState` only; `pftool net-fuzz` should stay at zero (run it). `step.rs`'s invariant test: a
  `Carrying` holder is a mutual grab, may be airborne, and its victim may be airborne.
* **Tests** (a shared `carry.rs` and the fighter's file): the carry's length at 0% straight out of the grab (165) and after 60 frames
  (105); mashing shortens it; walking at x1.1 (conflict 5); one jump; walking off the stage with the victim, who stays overhead; each
  cargo throw's release frame, damage, angle and FAF, the back throw's turn, the down throw's boost; the armor withstanding a weak hit
  and a strong hit breaking the hold; a KO during the carry; checksum cases for the three fields; a rollback replay through a carry
  off the stage; every other fighter's grabs unchanged (`grabs.rs`, `throw_combos.rs`, `base.pfc` equality).

### E1 addition: the knockback kind

`ARMOR_KNOCKBACK` built: `Armor::withstands(frame, damage, knockback)`; the knockback kind withstands a hit whose knockback (the value
`apply_hit` computes, after rage and before crouch cancelling: the reference's "before the one-on-one multiplier" is automatic here,
since the multiplier only touches the percent) is at most `threshold`; a field `subtract: bool` (default false) for the subtractive
form, which takes `threshold` off the knockback of a stronger hit (Yoshi's double jump later; the carry's kind is gap 11). The
validation refusal is removed. The carry reads it outside `S::Attack` (above). A bury hit ignores a subtractive armor (the reference's
rule). Tests in `multihit.rs` beside the existing armor test.

## Slots (`ext`, 16 a weapon since E17)

| Use | Slots |
| --- | --- |
| Giant Punch release grounded, aerial | 2 |
| Giant Punch full grounded, aerial | 2 |
| Giant Punch charge cancel | 1 |
| Headbutt aerial | 1 |
| Spinning Kong aerial (its early landing uses `landing_lag`, no slot) | 1 |
| Hand Slap loop, ending, aerial | 3 |
| Cargo throws forward, back, up, down | 4 |
| **Total** | **14 of 16** |

Two spare, for whatever gaps 6 and 7 show (an aerial charge with its own physics, a grounded Spinning Kong off-edge version). The jab
uses `Jab2`; the carry lift is the `FThrow` slot; the charge loops on the neutral special's own slot.

## Extension summary

| Id | Status | This port needs | Shape |
| --- | --- | --- | --- |
| E1 | built (super, damage); **addition** | super armor on four moves (exists); the carry's knockback armor | `ARMOR_KNOCKBACK` built with `subtract`; `withstands` sees the knockback; read in `Carrying` |
| E3c | built | no-clank boxes | as built |
| **E3g** | **new, small** (question 6) | "cannot rebound" on the smashes and Giant Punch | `Hitbox::rebounds` (default true) in `clank_attacks` |
| **E5** | **new, broad** | the Headbutt bury | `EFFECT_BURY`, Ruleset bury numbers, `FighterState::Buried`, hashed `bury_timer` / `bury_kb` / `bury_angle` / `bury_set` |
| E6 | built | `hold_at` for the cargo throws | as built |
| **E7** | **new, broad** | the cargo hold | `Move::carry: Carry`, `FighterState::Carrying`, hashed `carry_jumps`, `grab_start` |
| E11 | built, **one addition** (Cloud's) | Headbutt's vertical start speed | `start_speed_y` |
| E17 | built | 14 `ext` slots | as built |
| E19 | built | arm and head windows (a real limb here) | as built |
| E20 | not built (question 4) | three trip chances | a hitbox trip chance from the sim's RNG |
| E22b | proposed (Cloud's) | landing in the aerial Giant Punch | `Move::on_land` |
| E28 | proposed (Cloud's; question 5) | grounded Spinning Kong hit 1 against aerial targets | `ANGLE_VORTEX` |
| **E29** | **new, small** | Giant Punch's added damage over 110 frames | `Move::charge_add: ChargeAdd { full, add, floor }`, script `set_charge(n)` |

Not needed: E2 (no windboxes in this kit), E3a, E3b, E3d, E3e, E3f, E4, E8, E9, E10, E10c, E12, E13, E14, E15 (no wall jump), E16
beyond `set_charge`, E21, E22 (unless gap 7), E24 (`shieldstun` exists), E25, E26 (exists, used), E27.

Every new field defaults to today's behaviour, is hashed (content through `Weapon::hash_into` / `Ruleset` / `FighterParams::hash_into`,
state through `GameState::hash_into` with a checksum case each), appears in `docs/CONTENT.md`, and gets a test in its topic file; no
existing move or body uses them, so the other fighters stay unchanged (check `base.pfc` equality and the existing kit tests). The new
content fields are optional with defaults (as E3a's and E10c's were: no schema bump expected; if the owner prefers one, it follows
Cloud's schema 6). One `SIM_VERSION` bump per phase.

Shared limits this port exposes but does not fix (listed, not built): the throws' interrupted rows and thrower invincibility (the
carry's 4% row too); one get-up attack slot; the forward tilt's one-frame FAF difference by aim (unless the `end()` script is used);
the counter oddity of `direct=f`; trips (unless E20); the windbox immunity (unless question 7); the earlier shield's head intangibility
(a 7.0.0 shield hook, as Yoshi's).

## Suggested phases

1. **Body and every normal** (`FighterParams::hauler`, jab chain, the aimed forward tilt, up and down tilts, dash attack, smashes, the
   five aerials with the meteors, grabs, pummel, back / up / down throws, ledge and get-up attacks; class 7 Bracers registered in
   `CLASSES` and `roster.gd`, content re-export). Engine: nothing new (E3g only if the owner wants it now). The forward throw as a
   placeholder plain throw; specials as placeholders.
2. **The specials**: Giant Punch (E29, E22b if Cloud has not built it, E3g), Headbutt (E11 `start_speed_y`) with **E5 bury**, Spinning
   Kong (scripts; E28 or 367), Hand Slap (scripts). E5 is explained to the owner before it is built.
3. **The cargo hold**: **E7** and the E1 knockback kind, the lift, the four cargo throws; explained to the owner before it is built.
   Then the re-check of every iconic move in the hitbox view and against footage for the gaps.

(Phases 2 and 3 can swap: the carry is the bigger change and does not depend on the specials.)

## Questions for the owner

1. **E5's shape** (a new buried state with its own clock, mash-out, pop-out and stored knockback, on the reference's formula): approve
   before it is built? Include the stock-difference terms (+8 / +12 frames when the burier is behind, -18 / -22 when ahead), or even
   stocks only?
2. **E7's shape** (a new carrying state: the holder walks, jumps once and falls with the victim, the clock `x 11 / 6` minus the grab
   time, knockback armor, four cargo throws in `ext` slots): approve before it is built?
3. **The cargo hold's armor** (conflict 2): 16 or 80 units, subtractive or not? And its walk (conflict 5): x1.1 the walk or the
   parameter file's lift values? Footage settles both.
4. **Trips (E20)**: this kit has three (down tilt 40%, forward tilt down 15%, aerial Hand Slap 20%). Build E20 (a deterministic chance
   from the sim's RNG, a trip state) now, or keep trips a listed limit as for the Longsword's down tilt?
5. **Angle 366** on grounded Spinning Kong's first hit against aerial targets: reuse E28 if Cloud's phase builds it, else 367 as a
   labelled approximation?
6. **E3g "cannot rebound"** (the smashes and Giant Punch clank but do not rebound): build the small flag, or list it?
7. **Spinning Kong's windbox immunity** ("immune to most windboxes", no frames published): model it (a push-immune window on the
   armour frames [E]) or list it?
8. **Aerial Spinning Kong's gravity** (conflict 6): 0.07 as the gravity (low, sinking: proposed) or as a multiplier (a 48-unit rise)?
9. **Names**: Bracers / hauler / Hauler and the blurb above, or others?
10. **Footage** for the gaps (15 items in `REFERENCE_DATA.md`); the most important for 1:1: the Giant Punch charge with a frame counter
    and its cancels (gaps 1, 2, 6), the cargo hold's jump, ledge, armor and release (gaps 3-5, 11, conflict 2), the aerial Spinning
    Kong's height (conflict 6), the bury's pop-out and stored knockback (gaps 9, 10).
11. **Phase order**: the specials before the carry (as above), or the carry first?
12. **Owner decisions 34-36**: the brief mentions them, but `ROSTER_STATUS.md` ends at 33. Are there newer decisions to add?
