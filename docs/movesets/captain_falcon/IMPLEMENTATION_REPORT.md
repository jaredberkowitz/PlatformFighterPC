# Implementation report: the fifth moveset (reference: the reference game's Captain Falcon)

In game: class 4 **Gauntlets** (weapon `gauntlets`), body `sprinter`, built-in fighter **Sprinter** (owner decision 10). Target version
13.0.1. Data: `REFERENCE_DATA.md`; what the engine had and needed: `PROJECT_AUDIT.md`.

## Phase 1 (sim 66, content schema 5): the body and every normal

Branch `falcon-phase1` (from `movesets`, with `movesets` merged back in before the last commit; movesets was at sim 65, so this is 66).

### Built

* **Body** `FighterParams::sprinter` (`sim-core/src/content.rs`): every attribute and parameter-file value in `REFERENCE_DATA.md`
  "Attributes": weight 104; walk 0.987, initial dash 1.98, run 2.552, traction 0.105; walk acceleration 0.105 + 0.158, run 0.11 + 0.066;
  a **15-frame initial dash**; heavy landing 3 frames of lag; air speed 1.218, air acceleration 0.01 + 0.065, air friction 0.0075,
  gravity 0.12, fall 1.865 / fast fall 2.984; full hop 37.31, short hop 17.98, double jump 37.31, ledge jump 38.655; air dodges 42
  (3-30) and 62 / 72 / 94 (3-21); rolls 30 / 35, spot dodge 26; **wall jump** (1.4 away, 3.1 up; it also takes the jump button, the
  `movesets` rule, since `wall_jump` is set). Taller hurtboxes (`ecb_height` 2.6) [E].
* **Every normal** (`sim-core/src/moves_gauntlets.rs`, a child module of `moves.rs`), all on [R] frames, damage, angles, BK, KS, set
  knockback, hitlag / SDI / shieldstun multipliers, hitstun modifiers, shield damage and target filters; root-bone offsets converted
  [D]; other bones placed on our body [E]:
  * Jab 1 (hit 3, FAF 18, the far box pulls), jab 2 (hit 5, +3 hitstun, FAF 19), jab 3, the knee (6-7, FAF 33). Jab 1 continues into
    jab 2 from frame 5; **holding attack repeats jab 1 from frame 7**.
  * **The rapid jab's entry rule**: from jab 2, on frames 7-9, the rapid jab once attack has been **pressed 3 times and released twice
    since jab 1 began, presses during hitlag included**; otherwise a press made since jab 2 began gives jab 3 from frame 8; the rapid jab
    wins where both are open. The first cycle (`ext10`) is frames 1-20 with hits on 5, 7 ... 19; later cycles (`ext11`) are frames 3-20
    (the loop points 2-3 and 19-20, [D]: a 17-frame cycle); 0.6% x8 a cycle, hitlag x0.5, SDI x0.4, +2 hitstun, shieldstun x9 on both
    boxes. After the first cycle, letting go goes to the finisher (`ext12`: 3%, 45, BK 110, KS 55, hit 6-7, FAF 50).
    The **quickest loop** is reproduced tick for tick: jab 1 hit 3, jab 2 hit 9, rapid hits 15 ... 29, finisher 35-36, control back on 79.
  * Forward tilt (7-10, FAF 30) straight / up / down, +1% angled. Up tilt (the axe kick, 14-17, 285 degrees, grounded-only heel 35/100
    and aerial-only 22/90 with hitlag x1.4), **leg intangible 14-17**. Down tilt (11-12). Dash attack (clean 7-9 10%, late 10-16 6%,
    hitlag x1.25, shield +1, **shieldstun x1.5**).
  * Forward smash (19-22, charge 11) straight 19% / 43, up 20% / 45, down 20% / 38 KS 87. Up smash (hit 1 22-23 with its set-knockback
    boxes, hit 2 28-29; charge 8; **each kicking leg intangible while its kick is out**). Down smash (front 19-20, back 29-30, charge 11).
  * Neutral air (two hits, set weight and grounded / aerial split on hit 1). **Forward air, the knee: the 22% electric sweetspot on
    frame 14 only**, the 6% hip box the same frame, the 3% late hit 15-30. Back air, up air (the 30-degree late kick), **down air with the
    meteor box below the feet winning where both reach**. Every landing lag and autocancel window.
  * Grabs (standing 6-7 / 36, dash 9-10 / 44, pivot 10-11 / 39), pummel (1.3%, [D] one frame later, as for Mario), forward / back / up
    throws as hit-then-throw (3.5% / 3.5% / 4% on the held fighter, then the throw), down throw (6%, 68, BK 65, KS 70, release 21, FAF 40),
    the back throw ending turned round. Ledge attack (22-24, intangible 1-24, FAF 56), get-up attack (face up, 14-15 and 20-21).
* **Specials**: the claws' placeholders (as the Embers had in phase 1). The rapid jab uses `ext10` to `ext12`, so the claws' `ext0` to
  `ext2` placeholders still work.
* **Registered**: `CLASSES` 5 and `Recipe::params` (`sim-content/src/recipe.rs`), `Content::placeholder` (fighter `sprinter`, weapon
  `gauntlets`), `godot/scripts/roster.gd` (`CLASS_NAMES`, a blurb, the built-in Sprinter), `content/base.pfc` re-exported. Tests that
  count classes or fighters updated (sim-content doc / bundle / recipes; godot `content_e2e`, `creator_flow_test` [0..4],
  `editor_api_test`, `online_fighters_test`). The editor and bundle tests used "sprinter" as the name of a made-up fighter; they now use
  "racer".

### General extensions (each defaults to the old behaviour; no other fighter changed)

| Id | What | Where | Tests |
| --- | --- | --- | --- |
| E17 | 16 `ext` slots: `Ext10..Ext15` = 39..44, after `PivotGrab`, so no older slot or `goto` number moved. **Schema 5** with a migration 4 to 5 that has nothing to rewrite (a schema-4 weapon leaves the new slots empty; the bump makes an older build refuse a file using them) | `moves.rs` `MoveId`, `sim-content/src/bundle.rs` | `bundle::a_schema_4_file_from_before_sixteen_ext_slots_still_loads_unchanged`, `moves::move_ids_round_trip` |
| E19 | Per-limb intangibility: `Move::part_intangible: Vec<PartIntangible { start, end, parts }>` naming hurtbox circles (`PART_LOW`, `PART_MIDDLE`, `PART_HIGH`, `PART_LIMB`); `combat::intangible_parts`, applied to melee, grabs and projectiles. Content `part_intangible { start end low middle high limb }`, validated, hashed | `moves.rs`, `combat.rs`, `format.rs`, `lib.rs` | `gauntlets::the_up_tilts_and_up_smashs_kicking_leg_cannot_be_hit_while_the_kick_is_out` (the leg misses, the body is hit, and with the window removed the leg is hit) |
| E24 | Per-hitbox shieldstun: `Hitbox::shieldstun` percent (default 100) multiplying the damage part of `combat::block`'s stun | `moves.rs`, `combat.rs`, `format.rs` | `gauntlets::the_dash_attack_and_rapid_jab_stun_a_shield_longer` |
| E16 (narrow) | Script registers `presses` / `releases`: attack presses and releases since the attack began (the press that started it is 1; an attack started by another button starts at 0), counted in `fighter::update` **before the hitlag return**; `Fighter::presses` / `releases`, hashed | `state.rs`, `fighter.rs`, `sim-script/src/api.rs`, `scripting.rs`, `docs/CONTENT.md` | `scripted_moves::presses_and_releases_count_the_attack_button_since_the_attack_began_hitlag_included`, `gauntlets::presses_during_hitlag_count_toward_the_rapid_jab`, the state hash cases `presses` / `releases` |

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/gauntlets.rs` (22 tests): every normal's windows, FAF, landing lag and autocancel in the data; first-hit frames and FAF
by running each move; damage / angle / BK / KS of every phase and the multipliers; the jab chain, held repeat, rapid jab entry (quickest
loop timeline, the jab-3 path, holding gives no rapid jab), presses in hitlag, the loop (17-frame cycle) and finisher on letting go,
mashing keeps it going; the knee only on frame 14; the down air meteor sends straight down; E19 and E24; grab on frame 6; throws' hit and
release frames and damage, the back throw behind and turning round; pummel; aerial landing lag and autocancels; jump air times (34 /
48 / 24 / 34 within a frame); body attributes; the new fields in the content hash; random play deterministic and rolling back.
Also `ground_attributes.rs` (the sprinter's ground numbers, distinct from everyone's), `knockdown.rs` (its shared get-up / ledge test
now covers the first four weapons; the gauntlets' own numbers are in `gauntlets.rs`).

### Estimated or limited [E]

* Hitbox positions on bones other than the root; the body's height (2.6). Check in the hitbox view (scenario below).
* The rapid jab's "letting go": not held and not pressed for 8 frames (so mashing keeps it going). The reference's exact mash rule is not
  published.
* The rapid jab cycle read as frames 3-20 (17 frames) from the loop points "2-3 and 19-20".
* `presses` counts the press that started jab 1 as the first [D: the reading that makes both "3 presses" and "2 releases" binding].
  With it, the reference's quickest jab 3 (hit 17-18) cannot be input (any third press before frame 9 also makes the second release);
  jab 3 comes from a press after the rapid window or with fewer releases. Flagged for the owner.
* **Limits**: the forward air's late-hit 20% trip (E20, not built); the down tilt's type-0 box at the body sends by our "behind sends
  backward" rule, not by the target's side; the throws' hit on the held fighter is one frame (the reference's 11-12 / 12-13 boxes);
  the throws' interrupted rows and the thrower's invincibility are not modelled; the grabs' thin aerial-target boxes are not modelled
  (grabs catch grounded fighters only); one get-up attack slot (face up); the neutral air dodge (42) is single-source; the wall jump's
  0.2 "enable" speed and passive wall speed are not modelled for anyone; jostling is not modelled for anyone.
* Shared, noted in passing: `combat::block`'s 0.8 is a hair under in fixed point, so a stun that should come out whole is one frame short
  (10% blocked gives 9, not 10) for every fighter. Not changed here (it would change every fighter).

### Training-mode check (hitbox view)

Sprinter vs a standing duelist at 0%: (1) forward air from a short hop, the knee box (radius 0.47) at the front of the bent knee on
frame 14 only, the hip box behind it; (2) up tilt, the heel box coming down in front at chest height on frames 14-17, the leg hurtbox not
drawn as hittable (the bridge does not yet show intangible parts: see "For the art agent"); (3) up smash's two kicks over the head at 3.6
units; (4) down air, the meteor box below the feet. Adjust positions if they sit off the limbs in the clips.

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace` (all green, after the `movesets` merge too); `cargo clippy --workspace --all-targets -- -D
warnings` (clean); `pftool fuzz-rollback` (1000 runs, 0 mismatches); `pftool net-fuzz` (200 runs, 0 mismatches); `pftool content-check
content/base.pfc` (ok). **Not run**: the Godot tests (`godot/tests/*.gd`) — they were updated for the fifth class but need the editor
and a built bridge.

### For the art agent

* An emblem for class 4 (Gauntlets) in `godot/ui/match_hud.gd`, and a default look for the built-in Sprinter (it uses
  `Loadout.default_for(4)`, the same colour as the Duelist).
* Moves that want their own clip (they fall back to the shared archetypes now): the knee (forward air: knee driven forward, first
  active frame 14, one frame; the late hit through 30); the axe-kick up tilt (heel raised and brought down in front, active 14-17); the
  up smash's two overhead kicks (22-23, 28-29); the down air stomp (both feet straight down, 16-18); the jab knee (jab 3, 6-7); the rapid
  jab (both fists, a hit every 2 frames from 5, 17-frame cycle) and its rising backhand finisher (6-7); the dash attack shoulder (7-16);
  the forward smash elbow (19-22, three heights); the back throw (turns round on frame 13).
* Optional: `godot-bridge` could expose `combat::intangible_parts` so the hitbox view draws an intangible limb differently.

## Phase 2 plan: the specials (needs the owner's go for the broad pieces)

Slots: the rapid jab holds `ext10` to `ext12`; the specials get `ext0` to `ext9` and `ext13` to `ext15` (13), exactly the audit's count
(Falcon Punch 3, Raptor Boost 3, Falcon Dive 1, Falcon Kick 4, plus 2 spare; the rapid jab's 2 were counted in its 13).

### Extensions

* **E1 armor** (owner decision 14): `Move::armor: Option<Armor { start, end, kind, threshold }>` with `ARMOR_SUPER` / `ARMOR_DAMAGE` /
  `ARMOR_KNOCKBACK` defined and only `ARMOR_DAMAGE` applied now. In `combat::apply_hit`, after damage and hitlag, before the hitstun: an
  attacker in `Attack` inside the window and a hit whose damage (after `damage_mult`) is at most `threshold x damage_mult` leaves only
  damage and hitlag; grabs ignore armor. Content `armor { start end kind threshold }`; hashed; tests in `multihit.rs` (9% does not
  interrupt, 11% does, a grab catches). Raptor Boost: window 1-4 (conflict 2), threshold 10.
* **E6 command grabs**: `Hitbox::command` on a `HIT_GRAB` box uses `grab::catchable` (airborne or grounded, in hitstun, shielding; not
  intangible, held or grab-immune); `Move::catch_move` (an `ext` slot started on the catch instead of `Grabbing`; the holder stays in
  `Attack` with its velocity and air physics; the victim is `Grabbed` with no hold timer); `Move::hold_at` (where the victim is pinned,
  not snapped to a platform in the air). Tests in `grabs.rs`.
* **E21 search boxes**: hitbox kind `HIT_SEARCH` (never clanks, deals nothing, sees shields) and `Move::on_search` (the slot started at
  the end of the frame of contact, keeping velocity). Raptor Boost's dash. Open: intangible targets (gap 6).
* **E22 situation change keeping the frame**: `Move::off_edge` / `Move::on_land` (switch slot at the same `state_frame`, hit mask and
  charge kept). Falcon Kick off an edge (FAF 69 to 66); possibly the aerial Falcon Punch on landing (gap 3).
* **E3c no-clank melee**: `Hitbox::clanks` (default true), read by `combat::clash` / `clank_attacks`; Falcon Punch, Raptor Boost, ledge and
  get-up attacks; later transcendent projectiles.
* **E11 additions** (on the per-move rules `movesets` built): `fast_fall_from` (Falcon Dive from 50); a framed air block with
  `air_accel_percent` (`air_from`, `air_to`: Falcon Dive's x6 acceleration before frame 14 and gravity 0.103 after the rise; it could
  replace Falcon Punch's x0.92 script); `turn_at: Option<(frame, stick)>` overriding `special_turn_window` (Falcon Dive turns only on 14 at
  0.25); `lands_from` (Raptor Boost's aerial hit ignores landings before 8).
* **E16 wall register** for Falcon Kick's wall crash (14-36): `wall` (1 when the last move step hit a wall).
* **E15**: done in phase 1 (1.4 / 3.1, the jump-button wall jump included).

### Each special

* **Falcon Punch** (neutral): grounded (hit 53-57, FAF 104, 25%), aerial (22%, x1.15 boxes) and the two reversed versions (turn on 35,
  hit 62-66, FAF 118; 28% BK 85 KS 65 grounded, 25% aerial), reached by script on frame 15 with the stick back; B-reverse shared; aerial:
  0.2 up on start, x0.92 a frame for 19 frames, the 35-degree stick nudge on 51 / 60 (speed not published: fitted [E] per owner decision
  12, or asked); no clank (E3c).
* **Raptor Boost** (side): grounded search 10-25 (E21) into the uppercut (armor 1-4, E1; 10%, 85 degrees, +3 hitstun; FAF 28; the dash
  x0.18 on contact), FAF 72 on a miss; aerial search 19-35, ledges from 26, into the overhand (meteor on aerial targets, 60 on grounded,
  set momentum 1.5 back / 2.5 up on frame 6, no landing or ledge before 8; FAF 45, not helpless); helpless only on a miss (x0.7, landing
  lag 22). Dash speed fitted [D/E] to a published distance (owner decision 12).
* **Falcon Dive** (up): command grab boxes 14 and 15-30 (E6), catch move: 5% on 2-3 that also hits bystanders, 13% throw at 50 on 17,
  FAF 40, free afterwards (can use it again); miss: helpless x0.7, landing lag 24; ledges front 20 / back 37 (61 in the catch); fast fall
  from 50, turn only on 14, x6 air acceleration, x0.95 air speed, gravity 0.103 (E11 additions). Rise fitted to a published height.
* **Falcon Kick** (down): grounded slide with three phases (15 / 11 / 7%, FAF 69, 66 off an edge via E22), wall crash 14-36 (the `wall`
  register), on-hit slowdown; aerial dive (15 / 12 / 9%, FAF 57) with the 9% landing hit on both sides (`landing_move`, 35 frames).

## Questions for the owner

1. **Rapid jab presses**: should the press that starts jab 1 count toward "pressed 3 times" (built; it makes "released twice"
   meaningful but makes the published quickest jab 3 impossible to input), or not (then four presses make a rapid jab)?
2. **Rapid jab "let go"**: built as "not held and not pressed for 8 frames" so mashing keeps it going. Keep, or end on the first frame
   attack is up?
3. **Phase 2 broad pieces**: E6 (command grabs with their own catch sequence) and E21 (search boxes) are new hit kinds in
   `resolve_hits`; OK to build them as sketched above?
4. **Shield stun rounding**: fix `combat::block`'s 0.8 so whole-number stuns are exact (changes every fighter's stun by up to a frame,
   toward the reference)?
5. The neutral air dodge's 42 frames is from one source (Mario's on the same site is 52). Keep until checked?
