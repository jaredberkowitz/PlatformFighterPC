# Implementation report: the tenth moveset (reference: the reference game's Link, adult)

In game: class 9 **Buckler** (weapon `buckler`), body `wanderer` (`FighterParams::wanderer`), built-in fighter **Wanderer** (owner
decision 53). Target version 13.0.1. Data: `REFERENCE_DATA.md`; what the engine had and needed: `PROJECT_AUDIT.md`. Owner decisions that
shape this phase: 2 (the spin: grounded single charged hit, aerial multi-hit), 55 (E34 position vectors), 56 (E37 for every fighter),
57 (the forward air's drag as the reference data, no change); the data conflicts take the research's proposed values.

## Phase 1 (sim 91, content schema 6 unchanged)

Branch `buckler-phase1`, from `movesets` at 88adf03, merged up to 9385f20 (sim 89, the Clutch registered as class 8). Sim 90 is Donkey
Kong's phase 2 on its own branch; this phase is **91**. Commits: 24cfee7 (the engine), 4569a8b (merge of `movesets`), f7f7233 (the body,
the moves and the registration), 0137366 (the tests), then the docs.

### Milestone 1: the engine (all general; every default is today's behaviour)

* **E34, position vectors** (decision 55): `Hitbox::posvec_x`, `posvec_y`, `posvec_frames` (content fields of the same names). A hit with
  `posvec_frames` above 0 sets `link_kind = LINK_POSVEC`, the hashed `Fighter::posvec` (frames of carry left) and `link_point` = the
  attacker's feet where it hit plus the offsets by its facing (`combat::apply_hit`). At the launch (`fighter::posvec_launch`) the
  victim's feet move straight to the point at the speed that covers the distance in that many frames, with no gravity and no
  knockback decay (`knockback_frame`), and stop there; the angle and knockback still give the hitstun. Scaled like the hitbox. The
  reference's "the launch speed then drops off quickly" is taken as a stop [E].
* **E36, a follow-up keeps the charge**: `Move::keep_charge`; `fighter::follow_up` starts a `next` move or a script's `goto` and keeps
  `Fighter::charge` when the move reached has the flag.
* **E11 addition**: `Move::fall_speed` and `Move::fall_accel` (the special fall's own air speed and stick acceleration, world units:
  they replace what `fall_drift` gives; the base acceleration is then the body's), and `AirBlock::speed` and `AirBlock::accel_stick`
  (the same for a block of move frames). The reading that the reference's "additional air acceleration" is the stick-scaled part of the
  acceleration is [E] (gap 4).
* **E14, the passive shield**: `FighterParams::passive_shield: Option<PassiveShield { x, y, crouch_y, half_height, radius }>` (a
  `passive_shield { ... }` block in a fighter). In `combat::update_projectiles`, before a projectile's hit on a fighter that stands
  (`Idle`), walks or crouches on the ground: a shot in front of it, not moving away from its front, whose box touches the capsule, is
  blocked (`passive_shield_blocks` / `passive_block`): no damage, no launch, the hit's shield hitlag with shield SDI (the hashed
  `Fighter::shield_sdi`, cleared when the hitlag ends), a raised shield's pushback for that hit (no shield damage or stun), and the shot
  ends as on a shield (a piercing one goes on; a burst is never blocked). Melee hits are never blocked (the reference blocks only
  reflectable hitboxes).
* **E37, up special and up smash out of a shield** (decision 56, **every fighter**): in `fighter::shield`, after the shield stun and
  before the jump, special with the stick up starts the grounded up special (`start_special`), and attack with the stick (or the
  attack stick) up starts the up smash, on the same frame with no jump squat. Attack with the stick neutral is still the shield grab.
  The reference also lets either interrupt any jump squat; that general rule is not built (question 2).
* **E16 addition**: the `fast_fall` register (1 while the fighter is fast falling); whitelist entry, `scripting.rs`, test
  `scripted_moves::the_fast_fall_register_reads_a_fast_fall_during_the_move`, doc line in `docs/CONTENT.md`. Register id 30 (29 is kept
  for the Bracers' `aimed` on `bracers-phase2`).
* Hashed state with checksum cases: `Fighter::posvec`, `Fighter::shield_sdi`. Content fields hashed (`Hitbox`, `Move`, `AirBlock`,
  `FighterParams`), read and written by `sim-content/src/format.rs`, validated (no negative speeds; a position vector needs frames), and
  documented in `docs/CONTENT.md` ("Still schema 6 (sim v91)").

### Milestone 2: the body, the sword normals and the spin (`sim-core/src/moves_buckler.rs`, scripts `buckler_*.script`)

* **Body** `FighterParams::wanderer` (`sim-core/src/content.rs`), every value [R] from `REFERENCE_DATA.md` "Attributes" (conflict 1
  resolved to the parameter file): weight 104; walk 1.247, initial dash 1.98, run 1.534, traction 0.113 (`ground_friction` twice it);
  walk acceleration 0.105 + 0.21, run acceleration 0.044 + 0.13167; a 12-frame initial dash; heavy landing 3 frames of lag; air speed
  0.924, air acceleration 0.01 + 0.04, air friction 0.0038, gravity 0.096, fall 1.6 / fast fall 3.04; full hop 27.8, short hop 13.38,
  double jump 29.0, ledge jump 33.9; air dodges 49 (3-30) and 69 / 82 / 108 (3-21); rolls 30 / 35, spot dodge 26; no wall jump, the
  wall-tech jump 1.3 / 2.6. The shared body size [E]. The passive shield `FighterParams::wanderer_shield` [E: gap 2]: 0.6 ahead, 1.0
  up (0.4 crouching), half height 0.35, radius 0.45 (its front reaches as far as the guard held out, so a shot meets it no later than
  the body).
* **Normals**, every row from ssbwiki.com's raw tables via `REFERENCE_DATA.md`, on [R] frames, damage, angles and angle types, BK, KS,
  set knockback and set weight, hitlag and SDI multipliers, hitstun modifiers, shield damage, shieldstun, trips, target filters, hit
  groups; root-bone offsets converted [D]; boxes on `sword2` (a pose per window: grip and blade direction [E], the distance along the
  bone [R] past a guard 4 units from the hand [E]), `armr` and `colonells` (the forearm half way to the chest, the hand at the grip
  [E]), and the leg bones [E].
  * **Jab** 7-8 / 5-6 / 6-7 (FAF 24 / 26 (conflict 2) / 35), the far boxes at 180 pulling a fighter in; continuable windows 10 and 8 (a
    press waits in the buffer), held after a hit from 10 / 8, held without one from 14 / 13 (`buckler_jab_1.script`, `_2`).
  * **Tilts**: forward tilt 15 (the blade, arm and hand high) and 16-19 (the tip with them), FAF 40; up tilt 8-12 swept from behind to
    in front (three poses [E]), the tip 95 / KS 103, by position, FAF 36; down tilt 10-11 (13.0.1: 85 degrees), the low capsule, never
    clanks, FAF 31. **Dash attack** 20-23 (tip 14% at 45 with hitlag x1.2, blade 13%, arm 12%; shieldstun x1.4), FAF 57.
  * **Forward smash**: hit 1 on 17-18 (the tip 14% at 361, the blade and body 7% at 69 / 60, KS 12), charges on 9, FAF 51; attack pressed
    on 23-36 (from 14 in the buffer, hitlag presses counted: `presses`) starts hit 2 (`ext1`, 10-11, 13% / 12%, 48 degrees, hitlag x1.5,
    FAF 61), which **keeps hit 1's charge** (E36). **Up smash**: hits on 10-13 / 14-15 (4%), 25-28 / 29-30 (3%), 41-45 (11 / 11 / 10 /
    9%, KS 101), the first two set knockback with set weight and SDI x0.5, grounded-only (107, FKV 28) and aerial-only (115, FKV 38 /
    28; late tip 135, FKV 45 / 40, no +3) boxes, +3 hitstun on every other box, the last hit never clanks, FAF 78. **Down smash** 12-13
    in front (16 / 17 / 16 / 14%) and 24-25 behind (12 / 12 / 11 / 10% at 30), trip 30 everywhere, FAF 57 (conflict 4).
  * **Aerials**: neutral air (7-8 / 9-31, landing 6, autocancel 1-3 / 36+), forward air (16-17 with grounded-only boxes and the aerial
    367 drag on the outer two, decision 57; 24-25 KS 120; landing 11), back air (6-8 set-weight kicks FKV 50 / 36, 15-17; landing 6),
    up air (one long box 11-40, 15% then 13%; landing 14; autocancel 1-10 / 43+), down air (14-19 the aerial-only 270 meteor and the
    grounded 60 box, 20-64 15%; landing 19; autocancel 1-9 / 65+).
  * **The down air's bounce**: a hit when not fast falling (`fast_fall` register) switches to `ext2` (`buckler_dair.script`), which
    sets the vertical speed to 1.33 of the reference's units on its first moving frame and brings the box back 30 frames after the hit
    (11%, 65, BK 50, KS 80) until the down air's frame 64 [E]; then `ext3`, the down air's frames 65-79 (every landing autocancels).
    Another hit bounces again. The switch costs one frame without movement (a `goto`) [D].
  * **Grabs** (a hand grab: 6-7 / 9-10 / 10-11, FAF 35 / 43 / 38), **pummel** (1.3%, FKV 30, x2.1), **throws** hit then throw:
    forward 12 / 13 (3 + 2.5%, 33 degrees), back 12 / 14 (130 degrees), up 26 / 28 (5 + 2%, KS 230), down 22 / 24 (3 + 3%).
  * **Ledge attack** 20-22 (9%, intangible to 22, FAF 56) and **get-up attack** 17-18 behind / 21-22 in front (7%, shield +8,
    intangible to 22, FAF 46).
* **The spin (up special), both versions** (decision 2):
  * **Grounded** (`UpSpecial`): **one hit group** for the whole move, in four phases (clean 7-8: 14% blade / 11.2% tip at 45, BK 60,
    KS 88, hitlag x1.4 on the blade; mid 9-13: 12 / 9.6% at 43, BK 52; late 14-23: 9 / 7.2%, BK 48, KS 84; latest 24-39: 7 / 5.6% at
    361, BK 20, KS 82, the only phase that clanks); by position. Holding special holds it on frame 3 (`buckler_spin.script`: a
    `stall()`), up to 60 frames, and the hit deals `1 + 0.6 x charge / 60` (`charge_bonus` 60: exactly the reference's formula). FAF 77.
    Out of a shield it comes out at once (E37) and hits on frame 7.
  * **Aerial** (`ext0`, the up special's `air_version`): the eight spin hits (front 8-9, 16-17, 22-23, 31-32: 4 / 2 / 2 / 2% at 367,
    367, 83, 69 with set weight and FKV 107 / 107 / 71 / 62; back 12-13, 19-20, 26-27, 38-39: 2% at 145 / 145 / 150 / 160, FKV 95, no
    set weight, angle type backward, each a **position vector to [10, 20] over 10 frames**), SDI x0.3, one hit group re-armed by the
    script before each hit; the finisher on 47 / 48 / 49 (4%, 361, BK 30, KS 200, hitlag x2) in its own group. Every box never clanks.
    The rise: vertical speed 2.4 (0.3 world) set on the script's first frame (move frame 1 [E: gap 4]), gravity 0.096 x 0.55
    throughout; air speed 0.462 and stick acceleration 0.22 to move frame 46, then 0.8 and 0.01 (reference frame 48 on); ledges in front
    from 15, behind from 34; landing lag 30; animation 91, then the special fall at air speed 0.84 (x0.909) and stick acceleration
    0.025 (E11), landing lag 30.
* **Placeholders**: the neutral, side and down specials are the claws' (as every port's phase 1). Every `ext` slot is emptied first
  (the claws' up special used ext0-2); the Buckler uses ext0-3.
* **Registered**: `CLASSES` 10 and `Recipe::params` (`sim-content/src/recipe.rs`), `Content::placeholder` (fighter `wanderer`, weapon
  `buckler`), `godot/scripts/roster.gd` (`CLASS_NAMES` "Buckler", the audit's blurb, the built-in Wanderer with `Loadout.default_for(9)`),
  `content/base.pfc` re-exported (sim 91). Count tests updated: sim-content `doc.rs`, `lib.rs`, `recipe.rs`, `tests/bundle.rs`,
  `tests/recipes.rs` (`MADE` 10); netplay `tests/fighters.rs` (class 10 is the unknown one); godot `content_e2e.gd`,
  `creator_flow_test.gd`, `editor_api_test.gd`, `online_fighters_test.gd`. `angle_types.rs` (20 Buckler moves: 145 checked),
  `ground_attributes.rs` (the wanderer), `moves::every_weapon_has_every_move_slot` (the buckler).

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/buckler.rs` (30 tests): every normal's windows, FAF, landing lag and autocancels in the data; first-hit frames of 18
moves (both spins included) and the FAF of every move but the throws and ledge attack by running them; damage / angle / BK / KS of 52
boxes; hitlag, set knockback, set weight, SDI, hitstun modifiers, targets, trips, shield damage; the no-clank set (moves and the spin);
the quickest jab chain tick for tick (7-8, 14-15, 22-23, control back on 24 / 35 / 51) and both held rules; the forward smash's
second hit on a press in its window, not before, and charged x1.4 by hit 1's full charge (E36); the down air's bounce (1.33, its box
30 frames after the hit, none when fast falling) and the after-bounce 11%; the hand grab on frame 6; each throw's hit and release
frames and totals (5.5 / 5.5 / 7 / 6%); the pummel; landing lag and autocancels (the aerial spin's 30 included); jump air times (33 /
45 / 22 / 30 within a frame); the body's attributes; **the grounded spin** as one hit in its four phases with the 0.8 tip, 14% in front
on frame 7, 12% behind on frame 9 [E: sweep], charged x(1 + 0.6 x 29 / 60) after 29 held frames and 22.4% at the 60-frame cap; **the
aerial spin**'s eight hits and finisher (22% in all), its rise against the [D] height from its parameters (about 6.8 world units, 54.5
of the reference's) and peak frame, the special fall at frame 91 with its own speed (x0.909) and acceleration (0.025), a back hit
carrying its target to [10, 20] in 10 frames (E34), a target in front carried through the four front hits and the finisher (14%) on
frames 8 / 16 / 22 / 31 / 47; **the passive shield** stopping the blaster from in front standing, walking and crouching (no damage,
the block's hitlag, pushed back, the shot gone) but not from behind, not a small shot at head height, not during an attack or a dash,
and not for a body without it; **E37** for classes 0, 1, 3, 6 and 9 (the grounded up special and the up smash on the same frame out of a
shield; the shield grab and the jump unchanged) and the grounded spin out of a shield hitting on frame 7; the new content fields in the
content hash; registration; random play deterministic and rolling back. Also `scripted_moves.rs` (`fast_fall`), `angle_types.rs`,
`ground_attributes.rs`, the `state.rs` checksum cases (`posvec`, `shield_sdi`).

### Estimated or limited [E]

* **Positions on non-root bones** [E]: every `sword2`, `sword1`, `armr`, `colonells`, knee and hip box (the pose per window is in the
  code); the up smash's overhead sword; the down air's boxes at the feet (root offsets, [D]).
* **The grounded spin's sweep** (gap 3) [E]: the blade in front on 7-8, behind on 9-10, in front on 11-12 and so on, at hip height. Only
  "the front hit is on 7" is published.
* **The aerial spin**: the rise starts on move frame 1 (gap 4); the stick-acceleration reading of 0.22 / 0.01 / 0.025 (gap 4); a
  position vector's drop-off after its 10 frames taken as a stop; jostling off on 26-49 not modelled (no jostling in the air here).
  Whether a target in front also takes the back hits depends on where the boxes are (the reference's back hits are behind it): here
  it takes the front hits and the finisher.
* **The passive shield**: its size and place (gap 2) and its push (gap 1, a raised shield's pushback for the hit is used); "holding an
  item turns it off" waits for E4.
* **The down air** after a bounce: its box lasts to the down air's frame 64 [E]; the switch costs one frame of movement [D].
* **E37** is only straight out of a shield; the reference's up smash / up special interrupting any jump squat is not built.
* Not modelled (house limits): jab and forward smash's objects-only twin boxes, the grabs' thin aerial-target boxes, the throws'
  interrupted rows and the thrower's invincibility, the down throw's edge push, the face-down and trip get-up attacks (one slot),
  conflict 3 (jab 1 into 2 from 10, UFD's 12).

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace` (1098 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` (clean);
`pftool fuzz-rollback 500` (0 mismatches; 225 wanderers drawn), `pftool net-fuzz 200` (0 mismatches; 45 wanderers); the content
re-export equals the builders (`base.pfc` equality test). **Not run**: the Godot tests (`godot/tests/*.gd`), updated for the tenth
class, need the editor and a built bridge. Other fighters: every kit test is green and their content is unchanged except E37, which
changes every fighter's shield options by design (decision 56).

### Training-mode checks (hitbox view)

Wanderer vs a standing duelist at 0%: (1) the grounded spin tapped, the blade in front on 7 and behind on 9; held, the hold on frame 3
and the hit 1.6x at full charge; (2) the aerial spin next to an airborne target: the front hits carry it up, a target behind is thrown
round to the front-top point; (3) the forward smash pressed again on 23-36 for the second slash; (4) the down air on a standing target
without and with a fast fall (bounce / no bounce); (5) a blaster shot at the standing, walking, crouching and dashing wanderer.

### For the art agent

New moves fall back to the shared archetypes in `godot/scripts/anim_library.gd`. These want their own clips (reference frames; boxes
as in `moves_buckler.rs`):

* **Grounded spin** (up special): a crouched wind-up held on frame 3 while charging, then one fast horizontal sword circle at hip
  height (in front on 7-8, behind on 9-10 in our boxes), the blade's tip about 2 world units out; FAF 77.
* **Aerial spin** (`ext0`): a rising whirl, the sword horizontal, about 6.8 world units up over 45 frames, front hits on 8, 16, 22,
  31, back hits on 12, 19, 26, 38, a final outward slash on 47-49; then the special fall pose.
* **Forward smash** two-part: a downward slash on 17-18 and, on the press, a rising slash on its 10-11.
* **Down air**: the sword thrust straight down under the feet from 14 to 64; the pogo bounce up.
* **Up smash**: three slashes overhead (10, 25, 41). **Up air**: a long stab straight up (11-40). **Jab 3**: a thrust on 6-7.
* **Passive shield**: a short guard flinch on a block (the fighter's shield arm forward); the shield itself on the left arm in front,
  at chest height (lower when crouching).

## Phase 1, after the merges (sim 96)

`buckler-phase1` merged `movesets` at 4f346dd (sim 94, the Bracers' phase 2) and again at 81d5f50 (sim 95, the Clutch's phase 2),
keeping both sides; the script registers are `aimed` 29 and `fast_fall` 30, the functions `set_charge` 50 (no clash). This branch
takes **sim 96**. Owner defaults (decisions 61-62, `ROSTER_STATUS.md`):

* **Decision 61**: the aerial spin keeps the reference's layout, a target in front taking the four front hits and the finisher (5 of
  9) [E until footage].
* **Decision 62, built**: the up special and up smash also cancel a **jump squat**, for every fighter (`fighter::up_out_of_ground`,
  shared with the shield version): special with the stick up, or an up smash input (attack with the stick flicked up, the strong
  button with the stick up, or the attack stick up), pressed after the jump's own frame, starts the grounded move. A press made with
  the jump itself stays the short-hop aerial (the house's short-hop input, `reference_movement` tests), and attack with the stick
  merely held up stays the short-hop up air. Tests: `buckler::up_special_and_up_smash_cancel_a_jump_squat_for_every_class` (classes
  0, 1, 3, 6, 8 and 9, on each of the three squat frames).
* **Decision 58 on the Buckler's jabs**: both continuable windows run to 35, past the FAFs 24 and 26 (`late_chain`): a press after a
  jab ends, inside its window, starts the next hit (`buckler::a_press_after_the_jabs_faf_inside_its_window_carries_on`).

Checks after the merges: `cargo fmt --all`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`pftool fuzz-rollback 500` and `net-fuzz 200` (results in the commit message and the porter's report).

## Phase 2 (sim 102, content schema 6 unchanged): the bow, the boomerang and the sword wave

Branch `buckler-phase2`, from `movesets` at 9f2383b (sim 96); the work commit f83aa12, then a merge of `movesets` at 238868c (sim 97
the knockback-reference fixes, 98 the Bracers' carry, 100 the platform-drop rule, 101 the wall-jump input rule). Sim **102**.

### Milestone 1: the engine (all general; every default is today's behaviour)

* **E35, charge-scaled shots**: `ProjectileSpawn::charged: Option<ChargedShot { full, snap, damage_to, speed_to, life_to, size_from,
  size_to, floor }>` and the hashed `Projectile::damage`. As a shot appears, `c` = the firing fighter's `charge` (a smash charge, or a
  script's `set_charge` carried into the firing move by `keep_charge`), capped at `full` and snapped to `full` from `snap` on; damage,
  speed, life and size go in proportion from the spec's own values to the `_to` values (damage floored with `floor`). A script's
  `spawn` is fired as at no charge and its velocity scaled by the same speed ratio. A charged shot has no damage falloff (validated:
  `end_damage` equals the damage); phases keep their own damage. `ProjectileArc::drag` slows a shot along its flight, to a stop.
* **E3b, lodging**: `ProjectileArc::lodge` (frames) and `lodge_angle` (degrees, 0 any): an arcing shot meeting a platform top or a
  solid wall at that angle or steeper sticks there (`Projectile::rest = REST_LODGED`): it stops, stops hitting, has no script, and lasts
  `lodge` frames; meeting one more shallowly it is gone [E: gap 8's reading]. Not with `bounces`.
* **E3d, first part (the tool life rules)**: `ProjectileSpawn::tool: Tool { kind, limit, catch_from, catch_move, on_clank }`
  (`Tool::NONE` off), hashed `Projectile::kind`; the fighter register **`owned`** (id 31: live shots the fighter fired of the move's
  tool kind, or of the move itself with kind 0, as the frame began; `scripting::owned`); a spawn past `limit` does not happen. From age
  `catch_from` a shot still its firer's that touches its firer's body is removed, and a free firer (standing, walking, running,
  dashing, turning, crouching, airborne; not in hitlag) starts `catch_move`; a dodging one (roll, spot dodge, air dodge) lets it
  pass; a busy one (attacking, shielding, hurt) catches it without the move (`combat::catch_by_owner`). `on_clank: Option<Fall>`: a
  clank with an attack turns the shot into a falling, harmless one (`REST_FALLEN`: its life, speed back and up, gravity, top fall
  speed; it lies on a platform it lands on). **`Move::smash_version`**: a special started with a smash-style input (the `flick` rule)
  uses that slot instead (after `air_version`); B-reverse and the special turn windows treat it as its special.
* **E38 (new, small), the lying capsule**: `Hitbox::half_width` (content `half_width`): the box reaches that far ahead and behind its
  centre too (a rounded rectangle with `half_height`); `combat::shapes_touch` replaces `capsules_touch` (exact for these shapes) in
  hurtbox, counter bubble, passive shield, reflector field / body / detector tests. Scaled by `hitbox_scale`, not by a shot's charged
  size (the reference scales a shot's radius, not its capsule's ends).
* Reflected shots already hit their thrower (a reflection makes the reflector the owner, and only the owner is skipped), so
  **`hits_owner` is not needed** for the reflected boomerang; it stays on the phase-3 list for the bomb's self damage.
* Hashed and checked: checksum cases `projectile_damage`, `projectile_kind`, `projectile_rest` (`state.rs`); every new content field
  in `Weapon::hash_into` (`move_rules::the_tool_fields_are_in_the_content_hash`), read and written by `sim-content/src/format.rs`,
  validated (drag not negative, `lodge_angle` at most 90; a charged shot needs `full`, a `life_to`, sizes 10-1000 and no falloff; a
  clank fall needs a life; a `catch_move` needs a `catch_from` and a real move; a catch move may be frames only), round trip
  (`sim-content` `the_tool_fields_round_trip_and_are_checked`), documented in `docs/CONTENT.md` ("Still schema 6 (sim v102)").
  Read-only bridge accessor `projectile_rest(i)` (0 flying, 1 lodged, 2 fallen, -1 empty) for the view.

### Milestone 2: the moves (`sim-core/src/moves_buckler.rs` `tools`, scripts `buckler_bow`, `buckler_boomerang_*`, `buckler_fsmash`)

* **The bow** (neutral special): the draw (move frames 0-14, the reference's 1-15), then the hold from frame 16 while special is
  held, up to 180 frames; on the first frame special is up (frame 16 for a tap), or after 180 held frames, the script sets the charge
  to the frames held, fires the arrow and goes on as the release (`ext4`, FAF 30, `keep_charge`): FAF 45 for a tap, 81 after 36 held
  frames [R]. The arrow (E35): 2.5 degrees up, 2.6 → 10 reference units a frame (0.325 → 1.25), full at 36 charge frames (the snap),
  floor(4 + 8c / 60)% = 4 → 12, 46 frames, gravity 0.054, angle 361 type forward, BK 10, KS 71, radius 1.35 → 2, hitlag x0.8, not
  piercing [R]; lodges for 180 frames at 9 degrees or steeper (E3b). In the air: the body's air physics, landing carries the move on
  (gap 7) [E]. Spawn point 0.8 ahead, 1.2 up [E].
* **The boomerang** (side special): the throw (FAF 46; out on frame 27, aimed by the stick up to ±26 degrees, a 0.1 dead zone [E]) at
  2.4 (life 125) or, with a smash input (`smash_version` → `ext5`), 3.1 (life 167) with x1.2 on the clean and late hits [R]; one at
  a time (tool kind 1, limit 1): with one out, the throw goes on as the failed throw (`ext6`, FAF 60 ground / 46 air [R], from the
  throw's frame 2 [D]). The flight (projectile scripts `buckler_boomerang_flight` / `_far_flight`): slows 0.047 a frame to 0.3 along
  its throw (reaching it on age 45 / 60, exactly where the returning hit starts [D]); then it turns to face its thrower (1.1 above the
  feet [E]), speeds up to 3 at the rate that covers the turn-back distance 16 / 24 (0.0348 / 0.0232 world units a frame) [E: gap 6,
  the reading] and homes at up to 0.75 degrees a frame [R]. Hits by age: clean 8% (70, BK 80) ages 0-9, late 6% (65, BK 60) from 10,
  returning 3% (70, BK 50) from 45 / 60, all KS 40, type 1 (by travel), shield damage 50% (the reference's -4 / -3 / -1.5 [D]), radius
  4.0 / 3.6 → 5 [R]; not piercing. Caught from the returning age on (the catch, `ext7`, FAF 10 [R]); a clank drops it (60 frames,
  -0.15 / +1.25, gravity 0.09, fall 1.4 [R]), still counted as out. It bounces off any ground it meets [E: the reference's 16- and
  30-degree rules not modelled]. Spawn point 1.0 ahead, 1.1 up [E].
* **The sword wave**: the forward smash fires it on frame 17 only at exactly 0% (the `percent` register): 11 ahead, 6 up (1.375,
  0.75 [D]); E35 from the smash's charge: 5 → 13%, 2.4 → 3.4 (0.3 → 0.425), life 32 → 40, size x0.8 → x1.1; drag 0.01 (0.00125);
  angle 361 type forward, BK 45, KS 15, radius 1.8 → 2 on a 14-unit lying capsule (half 0.875, E38) [R]; no falloff (gap 9).
* Slots: ext4 bow release, ext5 far boomerang, ext6 failed throw, ext7 catch (8 of 16 used). The down special stays the claws'
  placeholder until phase 3.

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/buckler_specials.rs` (20 tests): the tapped arrow on frame 16 at 0.325, 2.5 degrees up, 4%, 46 frames, control on 45;
the charge scaling at 0 / 20 / 35 / 36 / 90 held frames (damage 4 / 6 / 8 / 12 / 12, the speed in proportion and full from 36),
FAF 30 after the release, the release keeping the charge; auto-fire after 180 frames; an arrow's hit (4% and 12%, used up), its
angle data; the arrow lodging in the stage (180 frames, on the top, harmless to a fighter on it, gone after); shallow (under 9
degrees) and steep landings; the bow drawn in the air landing and carrying on; the wave only at 0%, on frame 17, at 0 / 30 / 60
charge frames (5 / 9 / 13%, speed 0.3 / 0.362 / 0.425, life 32 / 36 / 40, size 80 / 95 / 110), its drag, its 5% hit at range and its
lying capsule reaching a target two or more frames sooner than its circle; the boomerang on frame 27, aimed straight / +26 / -26
degrees, life 125, control on 46; the smash input's 3.1 and 167; its slowest speed on age 45, turning back on 45-47, out about 7.6
units [D], caught and the catch move's FAF 10; the failed throw (FAF 60, nothing thrown); its hits by age (both throws) and live
(8% close, 6% further); the returning hit sending toward the thrower; a clank knocking it down (up and back, 60 frames, lying on the
stage, still out: a throw fails; harmless); a reflected one hitting its thrower for its clean hit and not caught; a shielding thrower
catching it without the catch move; resimulation from a snapshot. Also `scripted_moves::the_owned_register_counts_the_fighters_shots_out_and_a_limit_stops_more`,
`move_rules::the_tool_fields_are_in_the_content_hash`, the sim-content round trip, the `state.rs` checksum cases, and the phase-1
`buckler.rs` (32: its registration and slot tests updated for the new specials; the forward smash's two-hit test runs at 1% so the
wave stays out of it).

### Estimated or limited [E]

* Spawn points of the arrow (0.8, 1.2) and the boomerang (1.0, 1.1), the boomerang's homing point (1.1 above the feet).
* The boomerang's turn back (gap 6): the published "distance spent turning back" read as the distance over which it reaches its
  return speed, the turn itself made at once; the stick's 0.1 "sensitivity" read as a dead zone, the stick read on frame 27; it
  bounces off every ground (the reference's 16-degree minimum and 30-degree pass-through rules are not modelled); a dodge lets it fly
  on in its current heading but it then homes back (the reference's "continues toward the direction they dodged in" is not modelled).
* A reflected boomerang belongs to the reflector and homes on it (it is not caught by anyone); unpublished.
* The bow in the air (gap 7): the body's air physics, landing carries it on. Arrow lodging (gap 8): every platform top (soft ones too)
  and every solid block face; the 9-degree reading; a shallow arrow is removed.
* The house limit that a shot hits from the frame after it appears (the wave's and arrow's first frame).

### Tests and checks run

See the last commit message and the porter's report: `cargo fmt --all`, `cargo test --workspace`, `cargo clippy --workspace
--all-targets -- -D warnings`, `pftool fuzz-rollback 500`, `pftool net-fuzz 200`, the content re-export. **Not run**: the Godot
tests (they need the editor and a built bridge).

### Training-mode checks (hitbox view)

Wanderer vs a standing duelist: (1) tap the bow (an arrow on 16, a slow arc, lodged in the floor ahead) and hold it a second (a fast
flat arrow); (2) tilt and smash boomerangs, aimed up and down, standing still to catch them, and a second throw while one is out;
(3) jab a returning boomerang with the duelist (it drops); (4) forward smash at 0% (a wave) and at 1% (none).

### For the art agent

* **Bow**: a draw on frames 1-15, a held aim pose (it can last 3 seconds), the release recoil (30 frames); the arrow (a thin shaft,
  slightly arcing) and a lodged arrow lying at its angle in the floor or a wall (`projectile_rest` = 1) for 3 seconds.
* **Boomerang**: a throw (out on 27, FAF 46), the spinning boomerang (22 / 26 degrees a frame spin in the reference), the catch
  (FAF 10), the failed throw (an empty-handed throw), a knocked-down boomerang tumbling and lying (`projectile_rest` = 2).
* **Sword wave**: a horizontal crescent of light 1.75 units long, growing x0.8 → x1.1 with the charge, fired with the forward smash's
  slash at 0%.

## Phase 3 plan: items and the bomb

* **E4, held items** (decision 54: anyone can pick up the lodged arrows and the bomb and throw them; an opponent holding the bomb stops
  its detonation). Its shape, for the owner before it is built (a broad change, with a content schema bump):
  * **State**: a small hashed pool `GameState::items` (like the projectile pool: a fixed array, each slot an `Item { active, kind,
    owner, holder, pos, vel, age, life, hp, rest }`) and `Fighter::holding` (the item slot it holds, or none). An item is not a
    projectile: it can lie on the ground, be held, be thrown (then it hits like a projectile), take hits (the bomb's 50 HP) and bounce.
  * **Content**: an `ItemSpec` per item kind on the weapon that makes it (gravity, fall, friction, bounces, size, pick-up box, throw
    speed multiplier, the thrown hit and its damage formula `base + speed x k`, what a shield does to it, fuse / HP), with defaults
    that leave every fighter unchanged.
  * **Universal actions** (every fighter, the 13.0.1 rules: `REFERENCE_DATA.md` "Remote Bomb" throw rows and conflict 9): attack near
    a lying item picks it up (grounded; the aerial catch of a thrown item is gap 14); while holding, attack throws it forward / back /
    up / down by the stick (smash throws with a flick, the dash throw out of a dash), grab or a shield drops it, and an aerial attack
    is replaced by the throw (the "Z-drop" with grab). Item throw frames are fighter-wide (released 7 / 9 / 11 / 6 / 4 / 5, totals 24 /
    24 / 23 / 19 / 29 / 23), so they are actions, not weapon slots.
  * **Script surface**: a `holding` register (the kind held, 0 none) and `detonate()` (the caller's live tools of a kind burst: E3e's
    burst for the explosion); the bridge gets read-only accessors for the view.
  * Tests: pick-up / throw / drop frames, the thrown hits, the bomb's fuse, HP, bounces and self damage, rollback and checksum cases,
    other fighters unchanged unless they pick an item up.
* **The Remote Bomb**: E3d's long life (u16 for its 1800-frame fuse, or on the item), `hits_owner` (the explosion's self damage) and
  `detonate()`; the pull (refused while holding an item), the detonation move (12 frames), HP 50, bounces.
* **The arrow pick-up and the double arrow** (a lodged arrow becomes an arrow item; holding one, the bow fires two arrows 3 frames
  apart, x0.8 damage floored, BK 25 KS 110, lodging 120 / 30), and the passive shield off while holding an item.
* Then the re-check of every iconic move in the hitbox view.

## Questions for the owner

(Questions 1 and 2 were answered by owner decisions 61 and 62; question 3 stays open.)

1. **The aerial spin and a target in front**: with the reference's boxes (front hits in front, back hits behind), a target in front
   takes the four front hits and the finisher (14%); a target caught behind is carried round to the front. If footage shows a target
   in front taking all nine hits, the back boxes need to reach round further [E]: footage of the reference's hitbox view would settle it.
2. **E37's jump-squat cancel**: the reference lets an up smash or up special interrupt any jump squat (with or without a shield). Only
   the straight-out-of-shield start is built. Add the general jump-squat cancel too (it changes tap-jump and buffered inputs for every
   fighter)?
3. **Footage** for gaps 1-5 (the shield's push and coverage, the grounded spin's sweep, the aerial spin's acceleration reading and
   height) and conflicts 2-4 (jab 2's and the down smash's FAF, jab 1 into 2 from 10 or 12).
4. **The boomerang's turn back** (gap 6, phase 2): built as "it turns to face its thrower once it is at its slowest and speeds up to 3
   over the published turn-back distance (16 / 24)". Footage of the reference's boomerang path (a tilt and a smash throw, standing
   still) would confirm the turn's shape and timing.
5. **Arrows lodging** (gap 8): built on every platform top (soft ones too) and every solid face, at 9 degrees or steeper; a shallower
   arrow vanishes. Footage of a full-charge arrow skimming the stage would settle the shallow case.
6. **E4** (phase 3): the shape above (an item pool, `ItemSpec`, universal pick-up / throw / drop actions for every fighter, a schema
   bump). Build it this way?
