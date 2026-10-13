# Project audit for the ninth moveset (reference: the reference game's Yoshi)

What the engine needs to reproduce the reference game's Yoshi 1:1 (version 13.0.1), traced through the code at **sim version 83**
(content schema 5, branch `yoshi-research` from `movesets` at `5e7bcb3`). Nothing is built: this is research only. The data is in
`REFERENCE_DATA.md`; the engine inventory is `docs/movesets/SHARED_MECHANICS.md`; the extension ids are those of
`docs/movesets/ROSTER_STATUS.md`, `docs/movesets/cloud/PROJECT_AUDIT.md` (E3e, E22b, E28, the E11 `start_speed_y`) and
`docs/movesets/donkey_kong/PROJECT_AUDIT.md` (E3g, E5, E7, E29, the E1 knockback kind), with new ones numbered from E30. The owner
decisions in `ROSTER_STATUS.md` are 1-33 and 37-41 (34-36 are not in any committed copy). **Decision 5 applies throughout: a regular
shield, no E13.**

**Other branches that change what "exists"**: `greatsword-phase2` (sim 85, schema 6, not yet merged into `movesets`) has built E9 and
four general move rules this port can reuse: **`Move::start_speed_y`**, **`Move::on_land`** (E22b), `Move::ledge_drop` and
`Hitbox::half_height` (E3f). `bracers-phase1` is at the research commit (nothing of E1-knockback, E5, E7, E20, E29 or E3g is built yet).
Where this audit says "from Cloud's phase 2" or "from the Bracers", it means those branches, to be merged before this port's phase that
needs them.

## Proposed names (not registered)

| Thing | Name | Checked |
| --- | --- | --- |
| Class / weapon | **Clutch** (a clutch of eggs: the tongue that seals foes in shells, the lobbed shells, the rolling shell) | `content/blocklist.txt` has no terms (comments only); contains none of the `TRADEMARKS` terms in `sim-content/src/policy.rs` after its look-alike folding; not used anywhere in the code, content, `godot/scripts` or docs; not a reference fighter's name |
| Body | **bounder** (the very high second jump, the floaty heavyweight) | same |
| Built-in fighter | **Bounder** | same |
| Blurb | "Light on its feet and hard to launch, with a towering second jump that shrugs off blows: a long tongue that seals foes in shells, a rolling shell, lobbed shells that lift it higher, a crashing drop that scatters stars, and a flurry of kicks." | original wording |

Alternatives if the owner prefers: weapon "Shells", body "hatchling". Avoided on purpose: the reference's move names ("Egg Lay", "Egg
Roll", "Egg Throw" and the rest) and anything with the series' or the character's name (`yoshi` is a `TRADEMARKS` term). Class index 8
when registered (after the Bracers at 7; `CLASSES` becomes 9). This body has **no weapon object** (`Weapon::limbs` true, as the Claws,
the Gauntlets and the Bracers): the up smash's foot window (E19 limb) acts on a real limb here; the forward smash's head window is the
high circle.

## Iconic moves: what must match exactly

1. **The double jump and its armor** (a body trait, not a move). An ordinary jump to **51.56** reference units (the second-highest
   double jump: **not** a delayed jump with its own curve in 13.0.1, see conflict 12), a turnaround by holding back, and **subtractive
   knockback armor of 120 units on its frames 1-44**: a hit of at most 120 units leaves only damage and hitlag; a stronger one launches
   with `max(k - 120, 60)` units; an attack, an air dodge or a special ends it; grabs ignore it. **Engine**: the jump exists (a plain
   `air_jump_velocity` from `hop_velocity`, as every body); **the armor does not** (E1's knockback kind is reserved and refused by
   validation, and armor is read only in `S::Attack`) and **the turnaround does not** (the air jump sets the drift from the stick but
   never the facing). New: **E1 addition** (the knockback kind with `subtract` and a minimum launch, shared with the Bracers' carry) and
   **E33** (an air-jump armor and turnaround on the body; this replaces the roster's E12).
2. **Egg Lay** (neutral special). The tongue on 19-22 (a one-frame mouth box, then the tongue tip's aerial-only and grounded-only boxes),
   a **command grab** that catches on the ground and in the air; swallow; lay (release on the laying part's 21, Yoshi intangible 7-29,
   FAF 63 overall [E: conflict 5]); in the air Yoshi's vertical speed becomes 1 on laying. The **egg**: `100 + 1.8 x min(p, 100)` frames
   from the percent before the throw, cut by mashing (7 a stick input, 12.6 a button), cut by 2 frames a percent of hits taken (conflict
   2), hit for 0.66x damage with no knockback, laid at 0.7 / 2, growing from 0.45x to full in 20 frames, shifted by 0.3 / hopped by 1 on
   the ground (0.01 / 0.1 in the air), broken out of with a 2.5 jump and 7 intangible frames. **Engine**: the catch, the hold and the
   laying throw exist (E6: `Hitbox::command`, `catch_move`, `hold_at`; `HIT_THROW`); **the egg does not** (**E8**, broad: a new victim
   state, explained to the owner before it is built).
3. **Yoshi Bomb** (down special). Grounded: the hop's 4% FKV 80 grounded-only hit on 7, the drop from 27 (15%, shield +10), the fall
   at 4.8 until landing, a 7-frame soft-platform drop at the fall's start and all soft platforms while down is held, ledges both sides
   from the fall's start, the 39-frame landing (FAF 40) with two 4% stars on the landing's 3-12 (H 0, gravity 0.2, reflectable).
   Aerial: the stall, the 12% clean hit on 19 and late from 20. **Engine**: the hits, the endless fall (a `goto` loop), the landing move
   (`landing_move`), the ledge windows (E11) and the stars (projectiles with an arc) exist; missing: **a move's own soft-platform drop
   (new E30)** and **two projectiles on the same frame (new E3h)** (or the second star a frame later [E]).
4. **Down air** (the flutter kick). Twelve one-frame autolink kicks two frames apart (16-38: 2.3 / 1.8 then 1.9 / 1.4, KS 80 then 100,
   H 0.6, SDI 1.8), the 2.8% finisher on 41 (80 / 40, BK 55 KS 179), 28% in all, autocancel 1-13 / 50+, landing lag 17, the 1% landing
   hit on the landing's frame 1. **Engine**: data (hit groups, `Move::rehit` every 2 frames from 16 with the finisher's group never
   re-armed, 367 autolink, `sdi`, `hitlag`, `landing_move`).
5. **Egg Throw** (up special). The egg on 16 at `1.5 + 0.082 x f` (f the frames special is held, up to 15), aimed 18-90 degrees by the
   stick (default 58), its arc (x1.25 vertical, gravity 0.09), two bounces, the burst (6%, r 6.5, 3 frames) on a fighter, a surface or
   after 46 frames, **one hit** per target, H 0.6, shield -3; in the air **three hops per air time**, each `min(2.5 x 0.8^n + 0.8 y,
   2.6)`; the move's gravity 0.07; ledges only while not rising; never helpless. **Engine**: the throw and the hops are scripts
   (`spawn` with `sin` / `cos`, `add_vel`, `vy`, `air_flag`); the arc and bounces exist (sim 60 `ProjectileArc`); missing: **the burst
   (E3e, proposed for Cloud's phase 3)** with a rule that it does not hit the egg's own target again, two **E3b additions** (a vertical
   multiplier on a bounce, and bursting instead of disappearing when it lands too fast or runs out of bounces), and an **E11 addition**
   (`ledge_falling_only`).
6. **Back air** (two set-weight 367 autolinks on 11 and 14, the 5.5% KS 184 finisher on 18-19, all backward) and the **forward air**'s
   meteor snout (14% at 270, hits anyone). **Engine**: data.

Lesser points that must also be on [R]: the jab's 6-30 window and its held-without-a-hit 16; the forward tilt's three aims with the same
numbers; the up tilt's high front box; the down tilt's trips (E20 from the Bracers, owner decision 39); the forward smash's head
intangibility and sweetspot priority; the up smash's foot intangibility; the long, slow tongue grab; the heavy landing lag 6; the long
air dodges (58 / 79 / 107 / 130).

## How the moves flow (traced)

As in the earlier audits: `fighter::update` → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` / `start_special` →
`begin_attack` → `fighter::attack` each frame → (after every fighter moved) `combat::resolve_hits` → `grab::update` →
`combat::update_projectiles` / `spawn_projectiles` → ledge phase → `combat::check_ko` (`sim-core/src/step.rs`). Points that matter here:

- **The air jump** (`fighter::airborne`, sim-core/src/fighter.rs around line 913): `if f.air_jumps_left > 0 && f.take(JUMP)`, the
  vertical speed becomes `p.air_jump_velocity` (set per body from a height by `FighterParams::hop_velocity(gravity, height)`), the
  horizontal speed becomes the stick times `air_speed` (or 0 with the stick neutral), and `hop_boost`, `fast_fall`, `damage_fall`,
  `short_hop` are cleared. The state stays `S::Airborne` and nothing records that this airtime's jump is the air jump, how long ago it
  was, or which way the stick was; **the facing never changes**. Gravity is the body's from the next frame. So a jump "to 51.56 under
  gravity 0.08" is already exactly what `hop_velocity` builds (2.91 units a frame at the start, the peak on about frame 36 [D]); the
  reference's 13.0.1 double jump is that kind of jump (`REFERENCE_DATA.md`, conflict 12). **No velocity curve is needed.**
- **Armor** (`combat::apply_hit`, sim 67): after the damage is added and after a push has returned, `if def.state == S::Attack &&
  weapon.get(def.move_id).armor.is_some_and(|a| a.withstands(def.state_frame, hb.damage))` the victim takes only the hitlag.
  `ARMOR_SUPER` and `ARMOR_DAMAGE` are built; `ARMOR_KNOCKBACK` (2) is reserved and refused by `sim-content/src/lib.rs` validation
  ("knockback armor is not built yet"). The knockback is worked out a few lines earlier (`kb`, with rage, before crouch cancelling), so
  a knockback test can sit at the same place. A double-jumping fighter is in `S::Airborne`, not `S::Attack`: **armor that is not on a
  move needs its own test there** (the Bracers' carry has the same need in `Carrying`).
- **Grabs and command grabs** (`grab.rs`, E6 sim 69): `grab::can_catch` → `catchable` for a `Hitbox::command` box (airborne, grounded,
  shielding or in hitstun; not on a ledge, not held, not just released, not respawning; intangible fighters are skipped earlier in
  `resolve_hits`). `grab::connect` with a `Move::catch_move` keeps the holder in `S::Attack` (`next_move` = the catch move, next frame)
  and the victim `Grabbed` **with no clock and no mash-out**; `grab::pin` holds the victim at the move's `hold_at` (forward, up) from the
  holder's feet, in the air if the holder is. A `HIT_THROW` row (`resolve_hits` around line 439) frees the victim (`free_held`) and
  calls `apply_hit`, which launches it. **This is the whole Egg Lay up to the egg**: the tongue boxes (`targets` aerial / grounded,
  `command`), the swallow and laying move as the `catch_move` with `hold_at` in the mouth, and the laying throw as its `HIT_THROW` row.
  What the throw does to the victim (an egg instead of a launch) is E8.
- **Ordinary grab**: the tongue grab is an ordinary `HIT_GRAB` move with long boxes; the victim is held `Ruleset::grab_distance` in
  front (the reference holds it in the mouth: the same place for the sim) [D]. No tether needed (`air_lasso_type` False in the reference).
- **Projectiles** (`combat::spawn_projectiles`, `update_projectiles`): one spawn request per fighter per frame
  (`Fighter::spawn_request: bool`); a `ProjectileArc` (sim 60) gives gravity, a maximum fall, `bounces` off platform tops with the
  bounce's vertical speed `(-vy).clamp(bounce_min, bounce_max)` and the horizontal speed kept by `bounce_keep` percent; **after its last
  bounce a projectile that lands simply disappears**; a hit uses it up unless `pierce` (E3a); walls and the blast zone remove it. There
  is no burst (E3e is designed in the Cloud audit, built in Cloud's phase 3).
- **Scripts** (`sim-script/src/api.rs`): registers `frame facing x y vx vy percent stick_x stick_y grounded attack attack_tap special
  special_tap shield charge hit stored jump presses releases wall flick locked lock_x lock_y` (plus `boost`, `boosting`, `grab` on
  `greatsword-phase2`); functions `abs min max clamp sign floor sqrt sin cos set_vel add_vel spawn intangible end turn rehit stall goto
  store air_flag set_air_flag spawn_scaled spawn_at lock`; 4 fighter vars, 2 projectile vars. A script can aim a throw with `sin` /
  `cos`, count the frames special is held, read `vy` and change it with `add_vel`, and count hops in `air_flag`s. It **cannot**: turn
  the move's hitboxes off for a frame, know it is at an edge, set a hit's damage from its speed (the Bracers' E29 adds `set_charge`),
  or drop through a platform.
- **Moves and the ground** (`fighter::attack` around line 1944): a motion or `set_vel` along the ground follows the ground and leaves it
  at an edge (`platform = NONE`, an `off_edge` version if any); a move without a motion slides to a stop and, off an edge, ends unless it
  has `off_edge`. Nothing stops a moving fighter at an edge. `land_continue` lets a move touch down and carry on; `landing_move` starts
  an `ext` move on a non-autocancelled landing; `lands_from` and `fast_fall_from` exist (E11).
- **Soft platforms**: a fighter passes through one only while `Fighter::platform_ignore > 0`, which only the shield drop and the crouch
  drop set (`FighterParams::platform_ignore_frames` 8). Holding down in the air does not pass through. No move can set it.
- **Ledges during a move**: `ledge_front_from` / `ledge_back_from` (sim 58) catch from a frame, in front and behind, rising or not.
  There is no "only while not rising" rule (`SHARED_MECHANICS.md`: "no only-while-falling rule").
- **Hurtboxes** (`combat::hurtboxes`): four circles from the body's ECB; a crouch lowers them. An egged fighter would need its own
  placement (E8).
- **Shields** (`fighter::shield`, `combat::block`): global numbers, no geometry; any hit on a shielding fighter is blocked. Decision 5:
  nothing changes.
- **Fighter states** (`state.rs` `FighterState`, hashed as a `u8`): no encased state. `step.rs`'s `random_play_keeps_invariants`
  checks grab pairs; an egged fighter is not a grab pair (the throw has freed it) but must be added to the states the test accepts. The
  view maps state names to clips (`godot/scripts/anim_library.gd`): one new state needs an entry (the art agent's).

## Normals: what exists

| Needed | Exists? | Decision |
| --- | --- | --- |
| Body: walk 1.208 with accel 0.105 + 0.168, run 2.046 with 0.044 + 0.1056, initial dash 1.98 for 14 frames, traction 0.095, air 1.344 with 0.03 + 0.068, air friction 0.005, gravity 0.08, fall 1.29 / 2.064, hops 36.09 / 14.43 / 51.56, ledge jump 38.045, heavy landing 6, weight 104, dodges (UFD: spot 26, rolls 30 / 35, air dodge 58 and 79 / 107 / 130, intangible from 3), ledge decay 82 / 50 | yes (`FighterParams`) | `FighterParams::bounder`, as `champion` / `sprinter`; jump air times 38 / 60 / 26 / 42 checked by test; no wall jump (`wall_jump` 0), the wall-tech jump 1.3 / 2.3; ECB from scale 0.94 and the smashes' reach [E] |
| The double jump's armor and turnaround | **no** | **E1 addition + E33** (below) |
| Jab 1 → 2 on the 6-30 window, held without a hit from 16 | yes (`next`, `next_from`, `next_window`; a one-line script for the held rule, as the Greatsword's held repeat) | data plus a jab-1 script: `if attack && hit == 0 && frame == 15 { goto(jab 2) }` [D]; the 180-degree fighters-only far box as the Greatsword's |
| Forward tilt and smash aimed up / straight / down | yes (`Hitbox::aim`) | data; the tilt's aims differ only in position |
| Down tilt trips (30 / 30 / 40%) | **not yet** (E20, approved: owner decision 39, built with the Bracers) | data once E20 exists; ssbwiki.com notes they almost never trip in practice |
| Forward smash head and nose 12-16, up smash foot 11-13 | yes (E19 `PART_HIGH`, `PART_LIMB`; a real limb on this body) | data |
| Aerials: the neutral air's three phases, the forward air's meteor snout, the back air's set-weight autolinks and backward types, the up air, the down air's twelve kicks and finisher, landing lags 7 / 12 / 11 / 8 / 17, autocancels | yes | data; the down air's kicks: `Move::rehit = Some((15, 2))` re-arming the kick groups only (sim frames), the middle box at priority over the sides, rows split at frame 28 for KS 80 → 100, the finisher in its own group; its landing hit through `landing_move` |
| Forward air "ledges from 68" | no (no aerial ledge lockout) | the house limit (as the Greatsword's forward air) |
| Grabs (standing 14-21, dash 16-23, pivot 17-24) with the early mouth box and the tongue tip's aerial / grounded boxes | yes (`HIT_GRAB` rows by frame, `targets`) | data; the tongue's reach is [E] (gap 3) |
| Pummel (frame 1 → 2, FAF 8, H 2.1) | yes (the house shift) | data |
| Throws (forward 15 / 40, back turns and releases 20 / 44, up 13 / 44, down 18 + 8 hitlag → 26 / 52) | yes (`HIT_THROW`, `turns_around`, the folded hitlag) | data |
| Ledge attack, get-up attacks (face up / face down / trip) | yes (one get-up slot) | the face-up one; the other two are the known limit |
| No-clank boxes (pummel, ledge and get-up attacks) | yes (E3c) | data |

## Specials: what exists, and the exact shape of each needed extension

### The double jump: E1 knockback addition, E33 (air-jump armor and turnaround)

* **E1 addition (shared with the Bracers' carry; designed in the DK audit)**: `ARMOR_KNOCKBACK` built; `Armor::withstands(frame,
  damage, knockback)`; the knockback kind withstands a hit whose knockback (the `kb` `apply_hit` computes, after rage and set
  knockback, before crouch cancelling) is at most `threshold`; with **`subtract: bool`** (default false) a stronger hit takes `kb -
  threshold`; with **`min_kb: Fx`** (default 0, new here) the subtracted launch is at least `min_kb` (the reference's "60 KB minimum").
  The validation refusal is removed. Hitlag and damage are always taken; a push still pushes; grabs never reach it.
* **E33 (new, small): the air jump's own rules on the body.** Shape: `FighterParams::air_jump_armor: Option<Armor>` (default none:
  every existing body unchanged; Yoshi: `{ start 0, end 43, kind ARMOR_KNOCKBACK, threshold 120, subtract true, min_kb 60 }`, sim frames
  of the jump) and `FighterParams::air_jump_turns: bool` (default false). State: a hashed **`Fighter::air_jump_frame: u8`** (frames
  since the air jump, 0 = not in one; set to 1 by the air jump in `fighter::airborne`, counted up each `Airborne` frame outside hitlag,
  saturating, and cleared to 0 by anything that leaves `Airborne`: an attack, a special, an air dodge, a landing, a ledge grab, a hit
  that launches). In `apply_hit`, beside the move armor: `if def.state == S::Airborne && def.air_jump_frame > 0` and
  `params.air_jump_armor` withstands `(air_jump_frame - 1, damage, kb)` → hitlag only, the jump carries on (speed kept); else, if it is
  subtractive and the hit breaks it, `kb` becomes `max(kb - threshold, min_kb)` and the hit goes on as usual (launch, hitstun from the
  reduced knockback). With `air_jump_turns`, an air jump with the stick held back (`stick_x` against the facing past the walk threshold
  [E]) turns the fighter round. Conflict 4 (a 120-unit pool or each hit alone): each hit alone [E]; a pool would add one more hashed
  `Fighter` field (the knockback absorbed this jump). Content: `air_jump_armor { start end kind threshold subtract min_kb }` and
  `air_jump_turns` on the body, `docs/CONTENT.md` lines, `FighterParams::hash_into`, validation (as for move armor). Tests (in a shared
  `air_jump.rs` and the fighter's file): a 100-unit hit on frame 1 and on frame 44 of the double jump leaves only damage and hitlag and
  the jump reaches its height; on frame 45 it launches; a 150-unit hit launches at 60 (the minimum), a 200-unit hit at 80; an aerial
  started during the jump ends the armor; a grab still catches; a full hop (not an air jump) has no armor; holding back turns the
  fighter round; a checksum case for `air_jump_frame`; every other body's jump and every kit test unchanged.
* **Why not E12 (the roster's "air-jump velocity curve")**: the reference's 13.0.1 double jump is not delayed (ssbwiki.com `Double_jump`:
  the delayed jump was the first three games'; the double-jump cancel is impossible for this character since the previous game), and
  the armor's frame 44 sits just after a plain jump's apex (frame 36). `hop_velocity` already builds that jump. **E12 is not needed**;
  if the double jump's air time (gap 1) turns out not to match a plain jump, E12 comes back as a per-body curve.

### Egg Lay: data, E6 (exists), E8 (new, broad)

* **Slots**: the neutral special (the grounded tongue) and its `air_version` (`ext`); the catch move for each (`ext`, grounded and
  aerial): the swallow and the laying part as one move, `hold_at` in the mouth [E] until the release, the laying throw row on its
  release frame, `intangible_from` / `intangible` for the laying part's 7-29, FAF 63 overall [E: conflict 5]; the aerial one's
  `set_vel(0, 0.125)` on laying (a script line) and no ledge grab. The tongue rows: frame 19 the three boxes (`command`, targets aerial /
  grounded / all), 20-22 the two tongue-tip boxes; whiff FAF 42.
* **The victim during the swallow**: `Grabbed`, no clock (E6). Hidden in the view (the bridge needs to say so: an art-agent item, and a
  bridge field for "held inside": `Move::hold_hidden` or a view rule [E]).
* **E8 (new, broad: a new victim state; explain to the owner before building).** Shape (every default leaves existing content and
  fighters unchanged):
  * **Content**: a hit effect **`EFFECT_ENCASE`** (= 3, after the Bracers' `EFFECT_BURY` 2; content `effect encase`) on a `HIT_THROW`
    row, and the egg's numbers on the throwing move: `Move::encase: Option<Encase>` with `Encase { base: u16 (100), per_percent: Fx
    (1.8), percent_cap: u16 (100), mash_stick: Fx (7), mash_button: Fx (12.6), hit_cut: Fx (2 frames a percent), damage_taken: u8 (66
    percent), launch: (Fx, Fx) (0.0875 forward [E: gap 5], 0.25 up), grow_from: u8 (45 percent), grow_frames: u8 (20), shift: (Fx, Fx)
    ground (0.0375, 0.125) and air (0.00125, 0.0125), exit_vy: Fx (0.3125), exit_intangible: u8 (7) }` (world units [D]). Validation:
    an `encase` row needs the move to have `encase` (and the reverse). `docs/CONTENT.md` lines; `Weapon::hash_into`.
  * **State**: a new **`FighterState::Encased`**; hashed `Fighter` fields **`encase_timer: Fx`** (fixed point, so 12.6 a press is exact),
    **`encase_age: u8`** (for the growth), **`encase_by: i8`** (the encaser's slot, to read its move's `encase` each frame; NONE when
    free) and **`encase_move: u8`** (the move whose numbers apply). Each in `Fighter::hash_into` with a checksum case in `state.rs`'s
    tests.
  * **Entering** (`resolve_hits`, the `HIT_THROW` arm): an `EFFECT_ENCASE` throw frees the victim as now, adds its damage, then
    instead of `apply_hit`'s launch sets `Encased`, `encase_timer = base + per_percent x min(percent before the throw, percent_cap)`
    minus `hit_cut x` the throw's damage (the reference's 86-266), `encase_age = 0`, the velocity `launch` mirrored by the encaser's
    facing, no hitstun. (The throw's hitlag as usual.)
  * **While encased** (`fighter::update`, an `encased(f, p, ...)` arm): no action; gravity and fall speed the victim's own [E: gap 7];
    on the ground the body's friction [E]; a stick flick sideways adds `shift` in that direction (ground or air numbers), a flick
    diagonally up on the ground adds the hop; outside hitlag the timer drops by 1, by `mash_stick` a stick flick and `mash_button` a
    button press (the `grab::update` reading, shared as a helper); at 0 the break-out: `vel.y = exit_vy`, `intangible = exit_intangible`,
    state `Airborne` (its midair jump kept [E: gap 8]). No ledge grab, no tech; landing keeps it `Encased`.
  * **Hit while encased** (`apply_hit`): the damage times `damage_taken` / 100 is added; hitlag as usual; **no knockback, no hitstun,
    no state change**; the timer drops by `hit_cut x` the damage taken (conflict 2: taken after the 0.66x [E]); a push still pushes
    [E]. Grabs and command grabs refuse an encased fighter [E: gap 9] (`grabbable` / `catchable`).
  * **Hurtbox** (`combat::hurtboxes`): for `Encased`, one circle (the egg) of the body's height scaled from `grow_from` to 100% over
    `grow_frames` [E: gap 6]; the other three circles at the same place.
  * **Off the stage**: the egg falls; the blast zone knocks it out as anyone (exists).
  * **Rollback and the wire**: everything is `GameState`; inputs only on the wire (`pftool net-fuzz` should stay at zero; run it).
  * **Tests** (a shared `encase.rs` and the fighter's file): the egg's length at 0% and at 100% and 150% (the cap) against the formula
    (86 / 266 / 266 without mashing, the throw's own cut included); mashing cuts 7 a flick and 12.6 a press; a hit deals 0.66x, does not
    launch and cuts the timer; the shift and the hop on the ground, nearly nothing in the air; the break-out's speed and its 7
    intangible frames; no grab on an egg; an egg that falls off the stage is knocked out; checksum cases for the four fields; a rollback
    replay through an egg; every other fighter unchanged (`base.pfc` equality, the kit tests).
* **Why not reuse the Bracers' E5 (bury)**: both are "a victim held in place by a timer it mashes against", and they should share the
  mash-reading helper and the "stored state with a timer" pattern, but the egg moves (laid with a speed, falls, is shifted and hopped,
  is knocked out off the stage), changes the damage taken, never launches, and gives intangibility on the way out, where the bury
  pins in place, stores knockback and pops out with lag and no intangibility. One state with options would be a forced union; two
  states sharing a helper is the smaller change [E: the owner's call].

### Egg Roll: scripts, E29 (from the Bracers), new E31 and E32

* **Slots**: the side special (the hop into the egg and the roll, one move: 21 + 180 frames fit in a move's 255) and the ending (`ext`,
  40 frames to FAF after the cancel or the 180). An aerial start uses the same move (the script reads `grounded`) [E: whether the
  aerial start has the hop: gap].
* **The script** (4 vars: speed, roll frames left, jump used, the frame of the last hit): the hop `set_vel(0, 0.25)` on frame 1, the
  roll gravity and fall by `air_gravity` / `air_max_fall` (E11) [D]; from 21 each frame a `set_vel` from the stick and the speed
  (accel 0.05, friction 0.07, the base 1.4-3.2 and x1.2 by the stick [E: gap 10]); `land_continue`; on a wall (`wall`) the bounce
  (x0.4, vertical 1.7); a landing after the jump, the floor bounce (x1.3, 0.6x the fall, at most 1); jump once (`jump`, at least 30
  degrees [E: gap 11]); on `hit`, the speed -1 and the frames left -40, and `rehit()` 32 frames later (exact one-on-one; with more
  fighters, all targets re-arm together [D]); special after 25 roll frames on the ground → `goto(ending)` with the speed x0.7 / x0.5; at
  180 → `goto(ending)`; ledges in the air [E: gap 12]; never helpless.
* **The damage by speed: E29 (the Bracers' additive charge damage) reused.** The roll's box: `charge_add { full 40, add 4, floor false
  }` and each frame `set_charge(speed in tenths of a reference unit)`, so the damage is `10 + speed` up to 14 [D]; the knockback reads
  that damage as for any hit. No change to E29's shape.
* **E31 (new, small): a script turns the move's hitboxes off for a frame.** The roll has no box in the air and while it turns, and its
  boxes come and go with the ground. Shape: a script function `quiet()` that sets a hashed `Fighter::quiet: bool` for the current frame
  (cleared at the start of each frame); `combat::active_hitboxes` returns nothing for a fighter with `quiet` set. A whitelist entry, its
  test in `scripted_moves.rs` and the doc line. (Without it the roll would need three slots switched by `goto`, which clears `hit_mask`
  and breaks the 32-frame rehit.)
* **E32 (new, small): an edge register.** The roll "stalls at the ledge if the stick is not held towards it". Shape: a fighter register
  **`edge`**: 1 if the fighter stands on a platform and its next ground step at its current speed would leave it on the side it is moving
  (from `collision::platform` and the speed, the same test the motion code makes after moving), else 0. The script then holds the speed
  at 0 at the edge unless the stick points that way. Whitelist entry, test, doc line.
* "Moves through shields": attacking fighters are never moved by the body push (`push.rs`) [R behaviour exists]; the shielding fighter
  gives way instead of the egg [D: close; the reference passes through without moving it: a listed limit].

### Egg Throw: scripts, E3e (from Cloud's phase 3) with one rule, two E3b additions, an E11 addition

* **One slot** (the up special; the script reads `grounded` for the hop): on frames 1-15 it counts the frames special is held (a var);
  on 16 it reads the stick into an angle between 18 and 90 (default 58 with the stick neutral, sensitivity 0.5 [E: how the 0.5 maps])
  and `spawn(5/8, 13/8, speed x cos, speed x sin x 1.25)` with `speed = 0.1875 + 0.01025 x f` [D]. The hop in the air: `n` from
  `air_flag(0..2)` (three flags, set one per throw), `add_vel(0, min(0.3125 x 0.8^n + 0.8 x vy, 0.325) - vy)` on the hop frame [E: gap
  13], nothing at `n = 3`. The move's air physics: `air_gravity` 0.00875, `air_max_fall` 0.16125, `air_drift` 90 [D] (E11, exists), and
  **`start_speed_y` 80** (E11 addition from Cloud's phase 2). Never helpless. Ledges: **E11 addition `ledge_falling_only: bool`**
  (default false): with it, the move's ledge windows catch only while the vertical speed is not positive.
* **The egg** (the move's `ProjectileSpawn`): the 6% box (r 3, H 60, `shield_damage` 50, reflectable by the shared reflectors, `clanks`
  as usual: `rebound=f` is E3g); `arc { gravity 0.01125, max_fall 1.25, bounces 2 }`; `life` 46.
* **E3b additions (small)**: `ProjectileArc::bounce_keep_y: u8` (default 100: today's) multiplying the bounce's vertical speed after the
  clamp, and `ProjectileArc::bounce_max_fall: Fx` (default 0: none) above which a landing does not bounce (conflict 10). Defaults keep
  the Embers' fireball unchanged.
* **E3e (designed for Cloud: `ProjectileSpawn::burst: Option<Burst { life, rehit, on_wall, phases }>`, hashed
  `Projectile::bursting`)**, needed here with two more triggers and one rule: `on_ground` (a landing it does not bounce from, or after
  its last bounce, bursts there instead of disappearing), `on_expire` (the end of its life bursts it), and **`keep_hits: bool`** (the
  burst keeps the projectile's `hit_mask`, so the fighter the egg hit is not hit again by the burst: the reference's single hit). The
  egg's burst: life 3, one phase (6%, r 8). If Cloud's phase 3 builds E3e first, these are three optional fields on it (defaults off).
  Tests: an egg on a fighter deals 6% once and bursts on a second fighter beside it; an egg on the ground with a slow fall bounces
  twice then bursts; a fast one bursts at once; an egg in the air bursts at 46 frames.

### Yoshi Bomb: data and scripts, new E30, new E3h

* **Slots**: the down special (grounded: the hop with its 4% box on 7, the rise [E: gap 14], from 27 the drop at 0.6 with the 15% box,
  a `goto` loop of its own falling slot until landing), its `air_version` (`ext`: the stall [E: gap 15], the 12% clean box on 19, the
  late box from 20 in the falling slot), the falling slot (`ext`, shared, re-entered by `goto` before 255 frames), the landing (`ext`
  through `landing_move`, 39 frames, FAF 40). `land_continue` false, `landing_lag` 39 on the falling moves [D]. Ledges:
  `ledge_front_from` / `ledge_back_from` at the fall's first frame (both sides: the 13.0.1 change) [R]. The boxes' `shield_damage` 167 /
  183 [D].
* **E30 (new, small): a move's own soft-platform drop.** Shape: `Move::drop_through: Option<DropThrough { from: u8, frames: u8,
  while_down: bool }>` (default none): on move frame `from` the fighter's `platform_ignore` is set to `frames` (the reference's 7), and
  with `while_down` it is kept at 1 every frame the stick is held down past the crouch threshold. Content: `drop_through { from frames
  while_down }`, `docs/CONTENT.md`, `Weapon::hash_into`. Tests: a drop from a soft platform passes it in the first 7 frames and lands on
  the next one, holding down passes every soft platform to the ground, solid ground always stops it.
* **The stars**: the landing move's projectile (`arc { angle 73, gravity 0.025, max_fall 0.25 }`, speed 0.261 [D: 0.075 forward and
  0.25 up], `life` 10, the 4% box r 5 at 50 (3), H 0, `shield_damage` 250), one each way: `spawn(1, 1/8, 0.075, 0.25)` and
  `spawn(-1, 1/8, -0.075, 0.25)` on the landing's frame 3.
* **E3h (new, small): two projectiles on one frame.** Today a fighter has one `spawn_request` (a bool and one set of `spawn_*` fields)
  a frame, so the second star would come a frame late (the landing's 4: UFD's frame [E], conflict 11). Shape: a second request slot
  (`spawn_request: u8` counting 0-2 and a second set of `spawn_pos` / `spawn_vel` / `spawn_kb_add` / `spawn_size`), each in
  `Fighter::hash_into` with a checksum case; `spawn_projectiles` serves both in order. Or a listed one-frame limit (question 6).

## Slots (`ext`, 16 a weapon since E17)

| Use | Slots |
| --- | --- |
| Down air landing hit | 1 |
| Egg Lay aerial tongue | 1 |
| Egg Lay swallow and lay, grounded and aerial | 2 |
| Egg Roll ending | 1 |
| Yoshi Bomb aerial start, falling, landing | 3 |
| **Total** | **8 of 16** |

Eight spare (an Egg Roll air start or turn slot if gaps 10-12 need one; an Egg Throw air version if the hop needs its own physics). The
jab uses `Jab2`; Egg Throw is one slot; the Egg Roll's roll is its side special's own slot.

## Extension summary

| Id | Status | This port needs | Shape |
| --- | --- | --- | --- |
| E1 | built (super, damage); **addition** (designed with the Bracers) | the double jump's subtractive knockback armor | `ARMOR_KNOCKBACK` with `subtract` and **`min_kb`** (new here); `withstands` sees the knockback |
| E3b | built (sim 60); **two additions** | the egg's bounces | `bounce_keep_y`, `bounce_max_fall` |
| E3c | built | no-clank boxes | as built |
| E3e | designed (Cloud phase 3); **three options** | the egg's burst | `on_ground`, `on_expire`, `keep_hits` |
| E3g | approved (owner decision 40, the Bracers) | the egg and stars "cannot rebound" | as designed |
| **E3h** | **new, small** (question 6) | two landing stars on one frame | a second spawn request |
| E6 | built | Egg Lay's catch, hold and throw | as built |
| **E8** | **new, broad** | the egg | `EFFECT_ENCASE`, `Move::encase: Encase`, `FighterState::Encased`, hashed `encase_timer` / `encase_age` / `encase_by` / `encase_move` |
| E11 | built; **one addition here**, one from Cloud's phase 2 | Egg Throw's ledges, its start | `ledge_falling_only`; `start_speed_y` |
| E12 | **not needed** (conflict 12) | | replaced by E33 |
| E17 | built | 8 `ext` slots | as built |
| E19 | built | forward smash head, up smash foot | as built |
| E20 | approved (owner decision 39, the Bracers) | the down tilt's trips | as designed there |
| E29 | designed (the Bracers) | Egg Roll's damage by speed | as designed (`charge_add`, `set_charge`) |
| **E30** | **new, small** | Yoshi Bomb's platform drop | `Move::drop_through { from, frames, while_down }` |
| **E31** | **new, small** | Egg Roll's box only on the ground and not while turning | script `quiet()`, hashed `Fighter::quiet` |
| **E32** | **new, small** | Egg Roll's ledge stall | fighter register `edge` |
| **E33** | **new, small** | the double jump's armor and turnaround | `FighterParams::air_jump_armor`, `air_jump_turns`, hashed `Fighter::air_jump_frame` |

Not needed: E2, E3a (the egg is used up by its hit), E3d, E3f, E4, E5 (but E8 shares its mash helper), E7, E9, E10, E10c, E13
(decision 5), E14, E15 (no wall jump), E21, E22, E22b (no aerial move here lands into a grounded version), E24, E25, E26 (exists, used),
E27, E28.

Every new field defaults to today's behaviour, is hashed (content through `Weapon::hash_into` / `FighterParams::hash_into`, state
through `GameState::hash_into` with a checksum case each), appears in `docs/CONTENT.md`, and gets a test in its topic file; no existing
move or body uses them, so the other fighters stay unchanged (check `base.pfc` equality and the existing kit tests). The new content
fields are optional with defaults (no schema bump expected; if the owner prefers one, it follows Cloud's schema 6). One `SIM_VERSION`
bump per phase.

Shared limits this port exposes but does not fix (listed, not built): the throws' interrupted rows and thrower invincibility (Egg Lay's
4% interrupted row too); one get-up attack slot; no aerial ledge lockout (the forward air's 68); no crawl; the counter oddity of
`direct=f`; the 7.0.0 shield intangibility (decision 5); the double jump's 1.2 horizontal multiplier (every body's air jump drifts by
the stick times its air speed); the forward smash's charge push (jostling).

## Suggested phases

1. **Body and every normal** (`FighterParams::bounder` with the jump heights, jab chain, the aimed forward tilt and smash, up and down
   tilts, dash attack, the smashes with their E19 windows, the five aerials with the down air's kicks and landing hit, the tongue grabs,
   pummel, throws with the down throw's folded hitlag, ledge and get-up attacks; class 8 Clutch registered in `CLASSES` and `roster.gd`,
   content re-export). Engine: **E33 and the E1 knockback kind** here, since the double jump is the body's defining trait and belongs
   with the body (if the Bracers' phase 3 has not built the E1 kind yet, this phase builds it as designed there). Specials as
   placeholders. The down tilt's trips once E20 exists.
2. **Egg Throw and Yoshi Bomb**: the throw script and hops, the egg (E3b additions, E3e with the three options, E11
   `ledge_falling_only`, `start_speed_y` from Cloud's phase 2), the bomb (E30, E3h or the one-frame limit).
3. **Egg Lay and Egg Roll**: **E8** (explained to the owner before it is built), the catch and lay moves; the roll (E29 from the
   Bracers, E31, E32). Then the re-check of every iconic move in the hitbox view and against footage for the gaps.

(Phase 3 is the biggest; Egg Roll can move to phase 2 if E8 waits for the owner.)

## Questions for the owner

1. **E8's shape** (a new encased state with its own clock, mash-out, 0.66x damage, no knockback, shift and hop, a break-out jump with
   intangibility; the egg's numbers on the move): approve before it is built? And kept separate from the Bracers' bury (sharing the mash
   helper), as proposed?
2. **E33 and the double jump**: an ordinary jump with the armor and the turnaround (no velocity curve: the 13.0.1 jump is not delayed),
   in place of the roster's E12. Agreed? And for conflict 4: each hit compared with 120 on its own (proposed), or a 120-unit pool?
3. **The egg's open questions** (gaps 5-9): which way the egg is laid, its size, its fall, what the victim can do on breaking out,
   whether it can be grabbed. Defaults proposed in the E8 shape ([E]); footage would settle them.
4. **E31 and E32** (a script `quiet()` and an `edge` register) for Egg Roll: build them, or accept slot switching and a roll that does
   not stall at ledges?
5. **E30** (a move's own soft-platform drop) for Yoshi Bomb: build it, or list "does not drop through platforms"?
6. **E3h** (two projectiles on one frame) for the landing stars, or the second star a frame later as a listed one-frame limit?
7. **Names**: Clutch / bounder / Bounder and the blurb above, or others?
8. **Order of the phases**: the double jump with the body in phase 1 (proposed), the projectile specials before the egg?
9. **Footage** for the gaps (16 items in `REFERENCE_DATA.md`); the most important for 1:1: the double jump's air time and a multi-hit
   into it (gap 1, conflict 4), the egg at 0% and 100% hit and mashed (conflict 2, gaps 5-9), the Egg Roll's steering and jump (gaps
   10-11), an aerial Egg Throw three times (gap 13, conflict 10), Yoshi Bomb's rise and stall (gaps 14-15).
10. **Owner decisions 34-36**: `ROSTER_STATUS.md` jumps from 33 to 37. Are there decisions to add?
