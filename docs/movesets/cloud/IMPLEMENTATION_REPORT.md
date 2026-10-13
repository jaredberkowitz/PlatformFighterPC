# Implementation report: the seventh moveset (reference: the reference game's Cloud)

In game: class 6 **Greatsword** (weapon `greatsword`), body `champion` (`FighterParams::champion`), built-in fighter **Champion**
(owner decision 30; the research's proposed "vanguard" and "Surge" are not used). The gauge is called **Boost** for now. Target version
13.0.1. Data: `REFERENCE_DATA.md`; what the engine had and needs: `PROJECT_AUDIT.md`.

## Phase 1 (sim 83, content schema 5 unchanged): the body and every normal

Branch `greatsword-phase1`, from `movesets` (taken at 9c37fb2, then merged up to 0eada54, sim 82). One commit on top of `movesets`. The
sim numbers 79 to 82 belong to the engine session (grab and landing dash buffers) and the merge; this phase is **83**.

No engine change: every normal is data on mechanics that already exist (and one one-line script).

### Built

* **Body** `FighterParams::champion` (`sim-core/src/content.rs`), every value from `REFERENCE_DATA.md` "Attributes" [R]: weight 100;
  walk 1.155, initial dash 2.145, run 2.167, traction 0.106 (`ground_friction` twice it, `dash_attack_friction` it); walk acceleration
  0 + 0.1575, run acceleration 0.011 + 0.099; a 10-frame initial dash; heavy landing **3** frames of lag (the parameter file's
  `landing_heavy_frame` 4, the house rule); air speed 1.155, air acceleration 0.01 + 0.06, air friction 0.009, gravity 0.098, fall 1.68 /
  fast fall 2.688; full hop 32.5, short hop 17, double jump 32.5, ledge jump 36.25 (drift 0.6, the shared value); air dodges 47 (3-30)
  and 64 / 79 / 104 (3-21); rolls 29 / 34 and spot dodge 25 (the shared ones, as for every body); **a wall jump** (the parameter file's
  `wall_jump_type`) at 1.3 away, 2.4 up. The shared body height [E] (its attacks reach where a shared-height body's would).
* **Every normal** (`sim-core/src/moves_greatsword.rs`, a child module of `moves.rs`), every row read from ssbwiki.com's **raw**
  hitbox tables (re-fetched 2026-10-13 and compared with `REFERENCE_DATA.md`: they agree), on [R] frames, damage, angles and angle types,
  BK, KS, set knockback and set weight, hitlag and SDI multipliers, shield damage (`sd` converted to a percent [D]), target filters
  (`a=f` grounded only, `g=f` aerial only) and hit groups. Offsets on the root bone are converted by `u` (x 1.25, rounded half away from
  zero) [D]; reference capsules are circles along them (`along`) [D]; boxes on the sword bone (`haver`) are placed with `blade` (a grip
  point and a blade direction per pose [E], the distance along the blade [R]); boxes on the leg bones are placed on our body [E].
  `Weapon::limbs` is false (a held sword).
  * **Jab**: jab 1 (kicks, 4-5, 2.5%, 361 / the 180-degree pulling tip, hitlag x1.5, FAF 30), jab 2 (5-6, 2%, x1.2, FAF 33), jab 3 (the
    sword, 6-7, 3.5%, 30 degrees, BK 75, KS 50, x2, FAF 38), every box angle type 3. Continuable windows **8-30** and **9-30**
    (`moves::chain`, the input buffer's 9 frames). **Holding attack repeats jab 1 from its frame 16** (the reference's "consecutive jab
    frame"; `scripts/greatsword_jab_1.script`, the Embers' pattern). The quickest loop runs tick for tick: hits 4-5, 12-13, 21-22,
    control back on 30 / 40 / 53.
  * **Forward tilt** 9-10 (11%, 361, BK 20, KS 100, FAF 36). **Up tilt** 6-8 / 9 (8%, 95, BK 46, KS 100, by position; the box over the
    head early only, FAF 33). **Down tilt** 7-8 / 9-17 (the leg 7%, the body box 6% early, 80 degrees forward, BK 80, KS 75, FAF 41),
    a slide on 6-17 [E]. **Dash attack** clean 9-13 (11%, 60, BK 100, KS 70 since 7.0.0), late 14-18 (8%, 65, KS 50), shield +1, FAF 46.
  * **Forward smash**: three single-frame hits in three hit groups on **19, 24 and 28**: hit 1 (3%, set weight, x1.5, SDI 0) with
    grounded-only boxes (60 / FKV 25 near, 170 / FKV 10 at the tip) and aerial-only boxes (0 and 180, FKV 10); hit 2 (set weight, SDI
    x0.5: 3% at 15 / FKV 80 since 7.0.0, 2% at 15 / FKV 50, 4% at 170 / FKV 10); hit 3 (13% / 12%, 361, BK 42, KS 115, x1.8). Charges on
    5 (ssbwiki.com). FAF 67 (conflict 7). **Up smash**: clean 12 (13%, KS 105), mid 13-14 with the hilt (12%, KS 96) winning on priority
    and the box behind the head (11%, KS 90), late 15-16 (8%, 60, by position); charges on 6 (conflict 3 resolved to ssbwiki.com); FAF 46.
    **Down smash**: hit 1 on 8-9 (3%, set weight, SDI x0.5; aerial-only 170 / 155, grounded-only 160 / 150, FKV 90 / 85 since 3.1.0),
    hit 2 on 21-23 behind (11%, 35 **backward**, BK 35, KS 92, x1.4); charges on 2; FAF 50.
  * **Neutral air** 5-15 (8%, 40, BK 25, KS 100, by position; the sword's circle as four [E] parts of one hit), landing lag 9,
    autocancel 1 / 31+, FAF 39. **Forward air** clean 18-19 (the tip a **270-degree meteor on aerial targets only**, 14%, BK 20, KS 90;
    the same tip 14% at 60 on grounded ones; the arc and the blade 13% at 45; x1.4), late 20-25 (11%), landing lag 11, autocancel 1-2 /
    43+, FAF 54. **Back air** 11 / 12 (13%, 361 **backward**, BK 20, KS 95), landing lag 8, autocancel 1-4 / 37+, FAF 43. **Up air**
    **one hit in three phases** (13.0.1; not a multi-hit): 11% on 8 (the blade flat overhead, three columns) and 9 (on the blade), 8% on
    10-23, 80 degrees, BK 42, KS 80, landing lag 9, autocancel 1 / 32+, FAF 45. **Down air**: clean 11-13 (the tip a **270-degree meteor
    on aerial targets only**, 15%, BK 10, KS 100, x1.5; 13% at 60 along the blade), late 14-39 (8% at 65, BK 80, KS 55), landing lag 16,
    autocancel 1-4 / 46+, FAF 65.
  * **Grabs** (standing 9-10 / 37, dash 12-13 / 45, pivot 13-14 / 40, the grounded-target box). **Pummel** (1.3%, FKV 30, x2.1, one
    frame later than the reference [D], no clank). **Throws**, hit then throw: forward (kick on 7, 4%, x1.5; throw on 9, 3% at 49, BK 48,
    KS 125; FAF 36), back (kick on 13, 3%, x1.5; throw behind on 15, 3% at 38, BK 48, KS 110; ends turned round; FAF 32), up (kick on 8,
    2%; slash on 13, 2.5%, x1.2; throw on 15, 4% at 80, BK 78, KS 97; FAF 51), down (7% at 122, sent behind, BK 40, KS 110; the
    reference's 10 frames of hitlag on its frame 17 folded in: release **28**, FAF **54** [D], as the Staff's down throw). The hits on
    the held fighter never clank.
  * **Ledge attack** 20-22 (10%, 45 forward, BK 90, KS 20, shield +1, intangible through 22, no clank, FAF 56). **Get-up attack** face up
    (ssbwiki.com's "back" floor attack): behind 20-21, in front 24-25 (7%, 48, BK 80, KS 48, shield +8, by position, intangible through
    25, no clank, FAF 46).
  * **No-clank boxes** (E3c): exactly the raw tables' `clang=f` rows: the pummel, the throws' hits, the ledge attack and the get-up
    attacks (checked: no other normal has `clang=f`).
* **Specials**: the claws' placeholders (as the Embers, Gauntlets and Staff had in phase 1). No `ext` slot is used by the normals; the
  claws' placeholders use `ext0` to `ext2`.
* **Registered**: `CLASSES` 7 and `Recipe::params` (`sim-content/src/recipe.rs`), `Content::placeholder` (fighter `champion`, weapon
  `greatsword`), `godot/scripts/roster.gd` (`CLASS_NAMES` "Greatsword", the audit's blurb with "boost" for the gauge, the built-in
  Champion with `Loadout.default_for(6)`), `content/base.pfc` re-exported (sim 83). Tests that count classes or fighters updated:
  sim-content `doc.rs`, `recipe.rs`, `tests/bundle.rs`, `tests/recipes.rs` (`MADE` 7); netplay `tests/fighters.rs` (class 7 is now
  the unknown one); godot `content_e2e.gd`, `creator_flow_test.gd` ([0..6]), `editor_api_test.gd` (the made fighter is index 7),
  `online_fighters_test.gd` (nine and seven fighters). `select_grid_test.gd` does not count the built-ins (unchanged).
  `ground_attributes.rs`: the champion's ground numbers added; the "no two fighters" check now allows a shared walk speed (the tumbler
  and the champion both walk at the reference's 1.155) but still not a shared dash or run speed.

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/greatsword.rs` (22 tests): every normal's windows, FAF, landing lag and autocancel in the data, charge frames, the
continuable windows, no rehit anywhere; first-hit frames (15 moves) and FAF (every move but the throws and ledge attack) by running each
move; damage / angle / BK / KS of 45 boxes and every phase's multipliers, set knockback and set weight, targets, hit groups, shield
damage and angle types; the no-clank set; the quickest jab chain tick for tick (4-5, 12-13, 21-22, FAF 30 / 40 / 53), an early press
waiting for the window, the held repeat on 1, 16, 31, 46; the up air as one 11% hit (8% when only the late phase reaches); the forward
smash's hits on 19, 24, 28; the down smash in front on 8 and behind on 21; the forward and down air meteors straight down on aerial
targets and the down air's late hit upward; the back air and the down smash's second hit backward; the down tilt's slide staying on the
stage and its recorded leg window; the standing grab on frame 9; each throw's hit and release frames and total damage (7 / 6 / 8.5 / 7%),
the back throw behind with the thrower turned, the down throw behind; the pummel; aerial landing lag and autocancels; jump air times
(37 / 49 / 26 / 34 within a frame); the body's attributes; registration as class 6 with the claws' specials; random play deterministic
and rolling back. Also `angle_types.rs` (17 Greatsword moves added: 92 checked), `ground_attributes.rs` (the champion), and the unit
test `moves::every_weapon_has_every_move_slot` (now includes the greatsword).

### Estimated or limited [E]

* **Positions on non-root bones** [E]: every sword (`haver`) box (the up tilt, dash attack, up smash, the four parts of the neutral air,
  the forward air's blade, the up air from frame 9, the down air) and the leg boxes (the down tilt's knee, the throws' kicks, which act
  on the held fighter wherever they are). The grip point and blade direction of each pose are listed in the code comments; the distance
  along the blade is the reference's.
* **The neutral air's circle**: the reference is one set of boxes on the moving sword for 5-15; here four fixed poses (above 5-7, in
  front 8-10, below 11-13, behind 14-15) in one hit group [E].
* **The down tilt's slide**: 12 reference units over its frames 6-17 [E: the root motion is not published, gap 13]. Its back-leg
  intangibility (7-17) is recorded on the limb bit and **acts on nothing** (a held sword's fourth circle is the middle one; the low
  circle is the whole lower body, so it was not used: the same listed limit as the Staff's arms, owner decision 26).
* Not modelled (house limits, as for every port): jab 1's objects-only box (id 3), the grabs' thin aerial-target boxes, the throws'
  "interrupted" rows and the thrower's invincibility, the down throw's held-fighter invincibility during the folded hitlag, the
  forward air's "ledges from both sides from frame 78" (no aerial ledge lockout), the face-down and trip get-up attacks (one slot; the
  face-down one differs from face up by half a unit, the trip one is 5% at 361 with intangibility 1-7 and FAF 50), the down tilt's
  impossible trip.

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace` (938 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` (clean);
`pftool fuzz-rollback 1000` (0 mismatches); `pftool net-fuzz 200` (0 mismatches); `pftool content-check content/base.pfc` (ok); the
content re-export equals the builders (`base.pfc` equality test). **Not run**: the Godot tests (`godot/tests/*.gd`): updated for the
seventh class, but they need the editor and a built bridge. No new state, so no checksum case is needed in this phase; every other
fighter's kit tests are green and their content is unchanged (only the appended fighter and weapon change `base.pfc`).

### Training-mode check (hitbox view)

Champion vs a standing duelist at 0%: (1) the forward smash: three flashes on 19, 24 and 28 along the blade 0.8 to 2.0 units ahead at
1.1 up, the target pulled in and finished; (2) the up smash: the blade in front and up on 12, overhead on 13-14 with the hilt box at
the hands, behind on 15-16; (3) the neutral air's four poses round the body; (4) the forward air's tip 1.8 ahead at 0.9 up; (5) the down
air's tip 0.3 to 0.4 below the feet; (6) the up air's three columns over the head on 8. Move the [E] blade poses to the sword in the
clips once they exist.

## Phase 2 (sim 85, content schema 6): in progress

Branch `greatsword-phase2` (from `movesets` at a88c09f). Sim 84 is reserved for the engine session.

* **Milestone 1, engine (committed)**: E9, the gauge (`content::Gauge` on `FighterParams::gauge`, schema 6, `sim-core/src/gauge.rs`,
  hashed `Fighter::gauge` / `gauge_mode` / `gauge_drain`, `Move::limit_version` / `gauge_drain`, `FighterParams::powered`); E16
  (`boost`, `boosting`, `grab`, `boost_add`); E11 (`Move::start_speed_y`, `Move::ledge_drop` with hashed `Fighter::move_y0`); E22b
  (`Move::on_land`); E3f (`Hitbox::half_height`). Tests: `sim-core/tests/gauge.rs`, `move_rules.rs`, `scripted_moves.rs`,
  `sim-content/tests/bundle.rs`. The specials follow (milestone 2). The full write-up replaces this section when the phase ends.

## For the art agent

* An emblem for class 6 (Greatsword) in `godot/ui/match_hud.gd`, a look for the built-in Champion (it uses `Loadout.default_for(6)`,
  the same colours and hat as the Needles' Rogue), and a sword model (huge, two-handed: the boxes reach about 2 world units along the blade from
  the grip).
* Moves that want their own clip (they fall back to the shared archetypes now; first active frame, active span, pose): jab 1 (a front
  kick, 4-5, at chest height 0.9 to 1.75 ahead), jab 2 (a second kick, 5-6, 0.4 to 1.4 ahead), jab 3 (a horizontal sword slash, 6-7, 1.0
  to 2.0 ahead); forward tilt (a wide horizontal slash, 9-10, 0.9 to 1.8 ahead at 1.1 up); up tilt (the sword swept from behind up over
  the head to the front, 6-9); down tilt (a sliding low kick, 7-17, the leg 0.75 to 1.1 ahead at 0.25 up, sliding 1.5 forward); dash
  attack (a lunging slash forward and slightly up, 9-18, the blade 1.6 to 2.4 ahead); forward smash (a stepping three-part blade
  thrust or slash, hits on 19, 24, 28, the blade level at 1.1 up from 0.8 to 2.0 ahead); up smash (an arc from in front over the head to
  behind, 12-16); down smash (a low sweep in front on 8-9, then behind on 21-23, at 0.5 up out to 1.75 either side); neutral air (a full
  vertical circle round the body, 5-15); forward air (an overhead downward chop in front, 18-25, the tip 1.8 ahead at 0.9 up on 18);
  back air (a backward slash rising from 1.1 to 2.0 up, 11-12, 0.9 to 1.9 behind); up air (the blade flat overhead on 8, then pointed
  up and forward, lingering to 23); down air (the sword driven straight down below the feet, 11-39); the throws (forward and back
  kicks, the up throw's kick and rising slash, a down throw that pins the target behind); ledge attack (a sweep 20-22) and get-up attack
  (behind 20-21, in front 24-25).
* The Boost gauge's display (above the percent, and over the fighter while charging) comes in phase 2; the bridge will expose it like
  the percent, presentation only.

## Phase 2 and 3 plan

Every piece below defaults to today's behaviour for every other fighter, is hashed (content through `Weapon::hash_into` /
`FighterParams::hash_into`, state through `GameState::hash_into` with a checksum case each), appears in `docs/CONTENT.md`, and gets its
own test. One `SIM_VERSION` bump per phase.

### Phase 2: the normal specials and the gauge

1. **E9, the Boost gauge** (approved, owner decision 31; the shape is `PROJECT_AUDIT.md`'s "E9"), built general:
   * Content on the body: `FighterParams::gauge: Gauge { max, per_dealt, per_taken, mode_frames, walk, dash, run, traction, air_speed,
     air_accel, air_friction, gravity, fall }` (`max` 0 = no gauge, every existing body). The champion: 100, 0.667, 1, 900, x1.15
     walk (speed and both accelerations), x1.1 dash and run, x1.15 traction (every traction field), x1.2 air speed and acceleration,
     x1.15 air friction, x1.1 gravity and both fall speeds; **jump launch speeds unchanged**. **Schema 6** with a migration that adds
     nothing (an absent `gauge { }` block is `max 0`).
   * Content on the move: `Move::limit_version: Option<u8>` (the slot used while the mode runs) and `Move::gauge_drain: bool`.
   * State on the fighter, hashed: `gauge: Fx`, `gauge_mode: u16`, `gauge_drain: bool`, all reset by `respawn` (owner decision 9: a
     stock loss empties the gauge and ends the mode; a test pins it).
   * Gain in one helper called where the percent rises (`combat::apply_hit`, `combat::pummel`, the grab parry's 1%): the attacker gains
     `per_dealt x` **the damage actually applied** (after staleness and the 1v1 multiplier, owner decision 32); the victim gains
     `per_taken x` it, or loses it while in a `gauge_drain` move or held out of one; nothing from shields or counters. Filling starts the
     mode (`gauge_mode = 900`); while it runs nothing more is gained or drained. The timer counts every frame (gap 12 [E]).
   * Spending in `fighter::start_special` before the `air_version` swap: a move with a `limit_version` while the mode runs uses that
     slot and empties the gauge at once (even if the move misses). Follow-ups by `goto` never spend.
   * Runtime multipliers: `FighterParams::powered(&self, f) -> FighterParams` used by `step.rs` for `fighter::update` and the ledge
     phase; `combat.rs` keeps the plain body.
   * Bridge: the gauge and the mode as read-only values for the HUD (the art agent draws them).
2. **E16 additions**: registers `boost` (the gauge, 0 to 100), `boosting` (1 while the powered mode runs), `grab` (the grab
   button held); the function
   `boost_add(n)`. Each a whitelist entry in `sim-script/src/api.rs`, implemented in `sim-core/src/scripting.rs`, a test in
   `scripted_moves.rs`, a doc line.
3. **E11 additions**: `Move::start_speed_y` (percent of vertical speed kept on an aerial start; the beam's 65, Finishing Touch's 40,
   Limit Cross Slash's 0) and `Move::ledge_drop: Option<(Fx, Fx)>` with a hashed `Fighter::move_y0` (Climhazzard part 2: ledges in front
   and behind only after a fall distance, [E] gap 7).
4. **E22b**: `Move::on_land: Option<u8>` (an aerial move touching the ground switches to that slot on the same frame, keeping its frame:
   the aerial Blade Beam into the grounded one, the aerial Finishing Touch).
5. **E3f** (owner decision 33): `Hitbox::half_height` (a vertical capsule, tested with the reflector's capsule code), read for
   projectiles: the beam's two stacked circles as one 12-unit-tall wall (Limit: 19).
6. **The specials**:
   * **Blade Beam** (neutral special, and its aerial `air_version`): the grounded-only windbox on 16-17 (a push, FKV 40), the beam from
     18 (8% / 6.4% clean to 35, 6% / 4.8% late to 67, angle type 1, BK 38, KS 60, shield -4 / -3.2), speed 0.25 / 0.2 world, life 50; the
     air's fall break (x0.4, x0.65 vertical, gravity 0.06), no ledge grab; landing before the beam is fired makes it the grounded beam
     (E22b).
   * **Cross Slash** (side special, `ext` hit 2 and part 3): the Dancing Blade pattern (`hit`, `special_tap`, `goto`): hit 1 (10-11,
     4%, FAF 41) goes on to hit 2 only on hit (shields count) with special from frame 10; hit 2 (its 2-3, 3%, FAF 32) to part 3 from its
     14; part 3 (its 2-3, 11-12, 25-26; 3 / 3 / 6%; FAF 56). Box sizes and positions [E] (gap 1: no published table); hit 1's 366 box
     on E28 (phase 3) or the 367 until then.
   * **Climhazzard** (up special, `ext` part 2 and its landing hit): the stab on 7, the rising slash 10-15 (set weight, SDI 0, H 2.5),
     the rise fitted to the published description (owner decision 12, [E]) with x1 / x1.1 height and x0.6 horizontal speed; the second
     press in 20-26 into part 2; ledges from 24; the special fall at x0.6 with 20 frames of landing lag. Part 2: straight down at 0.525
     world a frame, the aerial-only 262-degree meteor and the 45-degree box from its frame 12, no clank, the landing hit (3.5%, 64, BK
     64, KS 128) with 26 frames of lag, ledges after a fall (E11 `ledge_drop`).
   * **Limit Charge** (down special, with its cancel in an `ext` slot): `boost_add(0.3)` a frame (334 frames from empty), the Needle
     Storm stall loop, cancels by special, shield, jump, grab, a roll and the air dodge with FAF 8, `gauge_drain` on the charge and the
     cancel; the start frame (gap 9), the end on a full gauge (gap 10) and the air physics (gap 11) [E] until footage.
   * With phase 2 done the gauge fills, the mode runs and expires, the multipliers apply and a KO resets it; no `limit_version` is set
     yet, so a special in the mode is its normal version and **does not spend** the gauge until phase 3 (the spend is gated on a
     `limit_version` existing).

### Phase 3: the Limit versions

1. **E3e**: `ProjectileSpawn::burst: Option<Burst { life, rehit, on_wall, phases }>` and a hashed `Projectile::bursting`: the beam stops
   at its first fighter (or a wall with `on_wall`), turns piercing and runs its burst phases (E3a's machinery).
2. **E28** (owner decision 33): angle 366 (`ANGLE_VORTEX`), the reference's vortex: the victim sent the way the attacker moves plus a
   pull to the hitbox centre for 5 frames, without the 367's speed cut or grounded rule; a hashed `Fighter::link_kind` and a 5-frame
   count beside the autolink fields.
3. **The Limit specials**, each its own move on the `limit_version` routing (12 of 16 `ext` slots in all, `PROJECT_AUDIT.md` "Slots"):
   * **Limit Blade Beam** (grounded and aerial): intangible 10-17, a no-clank beam from 16 (6% / 4.8%, speed 0.369 / 0.3, life 47,
     r 7 tenths in a 19-unit capsule) that bursts on its first fighter or a wall into five 2% / 1.6% hits every 6 frames at angle 366
     (FKV 40) and a 3% / 2.4% finisher (48, BK 50, KS 226): 19% on the first target, 13% behind it; FAF 61.
   * **Limit Cross Slash**: all five hits by themselves (UFD's frames 10-11 / 15-16 / 21-22 / 28 / 38-39 and damages 5 / 5 / 3 / 3 / 10,
     conflict 1 [E]), intangible 6-11, autolink 367 on hits 1-2, FAF 61, in the air no momentum (`start_speed` 0, `start_speed_y` 0,
     gravity 0).
   * **Limit Climhazzard**: 6% then 7% (KS 142, then 110) hits, intangible 5-12, height x1.6 / x1.85, x0.67 horizontal, the second press
     in 33-39 into the same part 2, ledges throughout, special fall x0.8 with 20 frames of lag.
   * **Finishing Touch** (down special's Limit version, grounded and aerial): 1% hits at 80 (BK 60, KS 431 / 406 grounded, 371 / 349
     aerial; hitlag x2) on 12 / 13 / 14 and 14 / 15 / 17, one hit group, the both-sides windbox (FKV 80, KS 100 / 86) on 22-26 / 25-29,
     FAF 85 / 88, the aerial hover (x0.4 both ways, gravity 0.03, max fall 1.0), E22b into the grounded one on landing.
4. Then the iconic-move re-check in the hitbox view and against footage for the open gaps (`REFERENCE_DATA.md` gaps 1-14).

## Questions for the owner

1. **The up tilt's sweep direction**: the reference data describes the sword sweeping from behind over the head; its early boxes are
   placed up and behind and the late one up and in front [E]. If footage shows the opposite way round, it is a two-line change.
2. **The down tilt's slide distance** (12 reference units [E]) and its back-leg intangibility (left as a listed limit, like the Staff's
   arms): keep both, or record footage for the slide?
3. **The get-up attack slot**: face up is built; the trip version differs (5% at 361, intangible 1-7, FAF 50). A second get-up slot
   would be a small shared extension for every fighter; wanted?
4. **Phase 2 spend rule**: until phase 3 builds the Limit moves, should a special used with a full gauge keep the gauge (as planned
   above) or spend it with no Limit effect?
5. The gauge's in-game name ("Boost" for now) and the register names that follow it.
