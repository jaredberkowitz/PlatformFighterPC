# Implementation report: the fifth moveset (reference: the reference game's Captain Falcon)

In game: class 4 **Gauntlets** (weapon `gauntlets`), body `sprinter`, built-in fighter **Sprinter** (owner decision 10). Target version
13.0.1. Data: `REFERENCE_DATA.md`; what the engine had and needed: `PROJECT_AUDIT.md`.

**Status (sim 73): playable.** Every normal (phase 1) and every special (phase 2) is built on [R] data and tested. Not yet *verified*:
the specials' travel distances (the dash, the leap's rise, the slide and the dive) are the reference's animation, not published, so they
are [E]; a few readings are [E] (listed per special below).

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
  (grabs catch grounded fighters only); one get-up attack slot (face up); the neutral air dodge (42: a second source agreed in sim
  73, see phase 2); the wall jump's
  0.2 "enable" speed and passive wall speed are not modelled for anyone; jostling is not modelled for anyone.
* Shared, noted in passing: `combat::block`'s 0.8 was a hair under in fixed point, so a stun that should come out whole was one frame
  short (10% blocked gave 9, not 10) for every fighter. **Fixed in sim 67** (owner decision 17): see phase 2.

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

## Phase 2 (sims 67 to 71 on branch `falcon-phase2`, merged into `movesets` at sim 72): the specials

The four specials on reference data (`REFERENCE_DATA.md`, "Specials"), and the general pieces they needed. Each piece defaults to the old
behaviour, so no other fighter's moves changed, **except the shield stun fix**, which is meant to change everyone (owner decision 17).
Commits: `0b9511d` (shield stun), `0c3608b` (E1), `4f661de` (E11 additions), `68f5370` (sim 68, E21 and the side special), `1c32a2b` (sim
69, E6 and the up special), `472449d` (sim 70, E3c and the neutral special), `27f1134` (sim 71, E22, `wall` and the down special), merge
`c6177a3` (sim 72). The specials sit in `sim-core/src/moves_gauntlets.rs` (`raptor_boost`, `falcon_dive`, `falcon_punch`, `falcon_kick`)
and its scripts `sim-core/src/scripts/gauntlets_*.script`; tests in `sim-core/tests/gauntlets_specials.rs` (27 tests).

### Slots

| Slot | Move |
| --- | --- |
| `NSpecial` | the punch on the ground (its `air_version` is `ext1`) |
| `ext0` / `ext1` / `ext2` | the punch reversed on the ground / in the air / reversed in the air |
| `SideSpecial` | the dash on the ground (`air_version` `ext4`, `on_search` `ext3`) |
| `ext3` / `ext4` / `ext5` | the uppercut / the aerial dash (`on_search` `ext5`) / the overhand |
| `UpSpecial` | the leap (`catch_move` `ext6`) |
| `ext6` | the catch: explosion and throw |
| `DownSpecial` | the slide kick (`air_version` `ext7`, `off_edge` `ext9`) |
| `ext7` / `ext8` / `ext9` / `ext13` | the dive kick (`landing_move` `ext8`) / its landing hit / the slide over an edge / the wall crash |
| `ext10` to `ext12` | the rapid jab (phase 1) |
| `ext14`, `ext15` | free |

### The shield stun fix (sim 67, owner decision 17; every fighter)

`combat::shield_stun_frames` works the reference's `floor(0.8 x damage x type x hitbox multiplier + 2)` (ssbwiki.com, Shieldstun) in
integers over the damage's raw fixed-point value, with 32 raw units (about 0.0005 of a frame) of slack so the damage's own fixed-point
rounding cannot drop a whole result. Before, 0.8, 0.33 and 0.725 were each a hair under in 16.16 fixed point and **every stun that should
come out a whole number came out one frame short** (10% gave 9, not 10). The type multipliers are named (`STUN_NORMAL` 1, `STUN_AERIAL`
0.33, `STUN_SMASH` 0.725, `STUN_PROJECTILE` 0.29), and, as in the reference, the projectile 0.29 now applies only to a projectile hit whose
own multiplier is 1 (no shipped projectile has another, so nothing changed there). Test: `combat::tests::shield_stun_is_the_reference_formula_with_whole_results_whole`.

**Which fighters changed** (worked out at sim 73 from every shipped hitbox, old formula against new): a hit gains one frame of shield stun
only where `0.8 x damage x type x multiplier` is a whole number. At the base damage that is every 1.25% step for normal moves (2.5, 5, 10,
15, 20, 25% hits); but a hit also carries the stale-move factor, 1.05 on a fresh move, and with it the whole cases are rarer. The moves
whose stun changed:

* At the base damage (a move that is neither fresh nor staled, which in play means after some uses): Longsword jab and up tilt (5%,
  10%), down tilt, dash attack, Blade Flurry hits (2.5%, 5%); Claws forward tilt (5%), up tilt (10%), flash dash (15%, 20%), up special
  loop (2.5%), up special ending (5%); Needles dash attack (10%), Flash Step (5%); Embers down tilt (5%), rising punch (5%); Gauntlets forward
  and down tilt, ledge attack, uppercut and overhand (10%), dash attack (10% x1.5), jab 3 and the leap's explosion (5%), slide and dive
  kick (15%), the punch (25%); the Staff's down tilt (5%) and its placeholder specials (the claws').
* On a fresh hit (x1.05, the usual first use): **only the 25% hits**, which go from 22 to 23 frames: the Gauntlets' grounded punch and
  aerial reversed punch.

Every change is one frame more, toward the reference. The Gauntlets' tests were moved to the exact numbers (dash attack 14, a rapid jab
hit 6).

### General extensions

| Id | What | Where | Tests |
| --- | --- | --- | --- |
| E1 armor (sim 67, `0c3608b`) | `Move::armor: Option<Armor { start, end, kind, threshold }>`: on move frames `start..=end`, a hit deals its damage and hitlag but does not knock the fighter out of the move. `ARMOR_DAMAGE` (a hit of at most `threshold` percent, compared before the one-on-one multiplier: the reference's damage-based armor) and `ARMOR_SUPER` (every hit) are built; `ARMOR_KNOCKBACK` (knockback subtraction, for Yoshi) is reserved and refused by validation. Grabs ignore armor; a push still pushes. Content `armor { start end [kind] [threshold] }`, hashed in the content hash | `moves.rs`, `combat::apply_hit`, `sim-content` format and validation | `multihit::damage_armor_withstands_hits_up_to_its_threshold_and_super_armor_every_hit`, `gauntlets_specials::the_uppercut_withstands_a_hit_of_up_to_10_percent_on_frames_1_to_4` |
| E11 additions (`4f661de`) | `Move::air_blocks: Vec<AirBlock { from, to, gravity, max_fall, friction, drift, accel }>` (frame-ranged air physics; `accel` the reference's horizontal acceleration multiplier, x6 = 600), `fast_fall_from`, `turn_at: Option<(frame, stick)>` (a move's own turnaround, replacing the up-special turn window, the up-and-back start and the B-reverse window for it), `lands_from` (touching down earlier does not end the move). Content `air { ... }`, `fast_fall_from`, `turn_frame` / `turn_stick`, `lands_from` | `moves.rs`, `fighter.rs` | `edges_and_recovery::an_air_block_changes_the_air_physics_only_on_its_frames_and_can_multiply_the_acceleration`, `fast_fall_from_holds_the_fast_fall_back_until_its_frame`, `a_move_can_turn_only_on_its_own_frame`, `lands_from_lets_the_move_touch_down_early_and_carry_on` |
| E21 search boxes (sim 68, `68f5370`) | hit kind `HIT_SEARCH`: a box that only notices a fighter it touches (shields or not; intangible fighters are skipped like every hit), deals nothing and never clanks or competes with the move's hits; `Move::on_search` (an `ext` slot) starts at the beginning of the next frame keeping the speed, unless the searcher was hit that frame. `Fighter::next_move` (the switch waiting for the next frame), hashed. `Move::air_version`: a special started in the air uses that slot instead | `combat::resolve_hits`, `fighter.rs`, `state.rs` | `multihit::a_search_box_touches_without_hitting_or_clanking_and_switches_the_move_next_frame`, the state hash case `next_move` |
| E6 command grabs (sim 69, `1c32a2b`) | `Hitbox::command` on a `HIT_GRAB` box uses `grab::catchable` (on the ground or in the air, shielding or in hitstun; not on or climbing a ledge, not already held or holding, not grab-immune, not respawning); `Move::catch_move` (the `ext` slot the holder goes on with: the holder stays in `Attack` with its speed and air physics; the victim is `Grabbed` with no hold clock and no mash-out); `Move::hold_at` (content `hold_x` / `hold_y`: where the victim is pinned, in the air if the holder is) | `grab.rs`, `combat.rs`, `step.rs`, `moves.rs` | `grabs::a_command_grab_catches_airborne_shielding_and_hitstunned_fighters`, `a_command_grab_holds_its_catch_in_the_air_without_a_mash_out_and_throws_it` |
| E3c no-clank hits (sim 70, `472449d`) | `Hitbox::clanks` (default true; content `clanks false`): a hit that never clanks with attacks, nor with projectiles (`clank_projectile` reads it too: a projectile's transcendent priority); it trades instead. Also in sim 70: an aerial version (`air_version`) counts as its special for the turn and B-reverse windows | `combat::clank_attacks`, `clash`, `clank_projectile`, `fighter.rs` | `hit_feel_2::a_hit_that_never_clanks_trades_instead`, `gauntlets_specials::the_punch_never_clanks_with_a_grounded_attack` |
| E22 off-edge version (sim 71, `27f1134`) | `Move::off_edge`: a ground move that goes over an edge goes on as that slot on the same frame (its own FAF), keeping its hits so far; without it, a move with a motion goes on as itself and one without ends (as before) | `fighter.rs` | `gauntlets_specials::the_slide_kick_off_an_edge_goes_on_as_its_off_edge_version_and_ends_on_66` |
| E16 `wall` register (sim 71) | script register `wall`: 1 when the fighter's last movement step was stopped by a wall (`Fighter::touched_wall`, hashed; the script runs before the move moves, so it is last frame's step) | `sim-script/src/api.rs`, `scripting.rs`, `fighter.rs`, `state.rs`, `docs/CONTENT.md` | `scripted_moves::the_wall_register_says_the_last_step_was_stopped_by_a_wall`, the state hash case `touched_wall` |
| E26 on the specials (sim 73) | every special box carries its reference angle type (below) | `moves_gauntlets.rs` | `angle_types::the_older_movesets_carry_their_reference_angle_types` |

### Each special: verified against [R] / [D] and what is [E]

**Side special** (the dash into an uppercut or an overhand; `raptor_boost`):

* [R], tested: grounded search 10-25 on grounded and airborne fighters (shields too), FAF 72 on a miss; the uppercut (clean 5, late 6-7,
  10%, 85, BK 82, KS 30, +3 hitstun, hitlag x1.5, shield +4 = 140 [D]; FAF 28), its **damage armor on 1-4 against hits up to 10%**
  (conflict 2: 1-4 by two sources to one), the dash speed x0.18 on switching; the aerial search 19-35, ledges from 26 on either side,
  landing lag 22, the special fall after the 79-frame animation (air speed x0.7, landing lag 22); the overhand (clean 5, late 6; the 270
  meteor on airborne fighters, 60 degrees BK 60 KS 80 on grounded ones and from the arm box; hitlag x2, shield +2 = 120 [D]), the set
  momentum 1.5 back and 2.5 up on frame 6 hit or not, no landing and no ledge before 8, FAF 45, free afterwards. The clean uppercut boxes
  never clank; the late box clanks (the reference's `clang` rows). Angle types (sim 73): uppercut and overhand forward (type 3).
* [E]: the dash's speed (0.8 / 2.0 / 0.6 reference units a frame on frames 1-9 / 10-25 / 26-30 on the ground, about 42 units; 0.5 then
  1.8 forward and 0.3 down in the air): the reference's root motion, not published. The overhand's landing lag 21 is dragdown's alone
  (conflict 7). Box positions on the hand and arm.

**Up special** (the leaping command grab; `falcon_dive`):

* [R], tested: grab boxes on 14 (the second one grounded-only) and 15-30, command grabs catching airborne, grounded, shielding and
  hitstunned fighters; the turnaround only on 14 with the stick back past 0.25; x6 air acceleration before 14, air speed x0.95, gravity
  0.103 after the rise; fast fall from 50; ledges in front from 20, behind from 37; a miss ends in the special fall after frame 66 (air
  speed x0.7, landing lag 24); the catch: the 5% explosion on 2-3 on the held fighter **and on bystanders** (radius 9.6 → 12 at the body
  [D]), the 13% throw at 50 degrees (BK 85, KS 62) on 17, FAF 40 (conflict 3), then free (it can be used again). Angle type (sim 73): the
  explosion forward (type 3).
* [E]: the rise (3.4 reference units a frame slowing evenly to 0.4 over frames 14-31, about 34 units up and 11 forward, plus the drift the
  x6 acceleration built), the backflip after the throw, where the held fighter is pinned (at the fist, 0.9 ahead and 0.5 up); touching down
  during the catch carries it on (gap 5); the catch grabs no ledge (the reference's: in front from its frame 61).

**Neutral special** (the wound-up punch; `falcon_punch`):

* [R], tested: grounded hit 53-57, 25%, 361, BK 93, KS 59, FAF 104; the stick held back on 15 switches to the reversed version (turns on
  35, hit 62-66, 28% BK 85 KS 65 on the ground, 25% in the air, FAF 118 from the start); aerial 22% with the x1.15 boxes, the set 0.2
  upward speed on starting, x0.92 a frame for 19 frames with no air control meanwhile; the 35-degree boost read on 51 (60 reversed) with
  the stick up or down between 0.25 and 0.65; hitlag x1.5; **no clank** on every box. Angle types (sim 73): every box by position (no `af`
  in the reference's rows, the template's default 0).
* [E]: the boost's speed (1.0 reference units a frame along 35 degrees: not published, gap 2); the stick's "back" threshold 0.25 for the
  reversal; touching down during an aerial punch carries it on along the ground (gap 3); the grounded slide is not modelled (none
  published); box positions on the arm.

**Down special** (the slide kick and the dive kick; `falcon_kick`):

* [R], tested: the slide's three phases (clean 13-16 15% 52 BK 60 KS 88, mid 17-23 11% 52 BK 90 KS 60, late 24-33 7% 46 BK 80 KS 65;
  conflict 1 resolved to ssbwiki.com), FAF 69, **66 when it goes over an edge** (E22), the **wall crash on 14-36** (the `wall` register,
  a 53-frame crash), x0.8 speed once it hits; the dive (clean 14-18 15% 361 BK 80 KS 57 hitlag x1.2, mid 19-24 12% KS 46, late 25-27 9%
  KS 46; FAF 57, not helpless) and its **landing hit** on both sides (9%, 60, BK 80, KS 35 on 1-2, landing lag 35). Angle types (sim
  73): the slide, its off-edge version and the landing hit by position (no `af`), the dive forward (type 3).
* [E]: the slide's speed (2.4 reference units a frame through the kick, easing to a stop by 45: about 69 units in all) and the dive's
  (0.3 a frame forward on 1-13, then 2.1 forward and 2.1 down on 14-27: about 33 units forward and 29 down); the crash's push off the wall; "maximum speed decreases on hit 4"
  and the 0.12 ending friction (not modelled); positions on the leg; whether the aerial kick is limited per airtime (gap 4: not limited
  here).

### Sim 73 follow-ups (branch `cleanup-72`)

* **Angle types** for every special box, read from ssbwiki.com's raw wikitext (`Captain_Falcon_(SSBU)/<special>&action=raw`, the `af`
  field; a row without one is the template's default 0, "standard", by position): see each special above. Visible change: a target behind
  the fighter when a punch, slide or landing-hit box reaches it is now sent away from the fighter (backward) instead of forward.
* **The neutral air dodge** (owner decision 18): a second source agrees. ssbwiki.com's air dodge page (`Air_dodge`, the current game's
  table, raw wikitext) gives this character's neutral air dodge as 42 frames, intangible 3-30 (fully stale 6-27), and the directional
  ones 72 sideways, 94 up, 62 down, 85 diagonally up, 66 diagonally down, intangible 3-21: every number the same as UFD's. (That table
  carries a cleanup tag saying some values may be inaccurate.) dragdown's parameter page has the fields but no values. **42 kept**, now
  [R] from two sources. The other figure that made it look odd, Mario's 52, is also on both sites.

### Tests and checks run

The phase-2 run was cut off before its checks were recorded. At sim 73 (this branch): `cargo fmt --all`,
`cargo test --workspace` (all green), `cargo clippy --workspace --all-targets -- -D warnings` (clean), `pftool fuzz-rollback 1000`
(0 mismatches), `pftool net-fuzz 200` (0 mismatches), `pftool content-check content/base.pfc` (ok). **Not run**: the Godot tests.

### Training-mode checks (hitbox view)

Sprinter vs a duelist at 0%: (1) side special on the ground into a shielding duelist: the search box touching the shield switches to the
uppercut, which hits the shield; (2) aerial side special over a grounded and an airborne duelist: the meteor box only sends the airborne
one down; (3) up special under a duelist in the air and next to a shielding one: both are caught and exploded, a third fighter inside the
explosion radius is launched too; (4) down special toward a wall from close: the crash before frame 37, no crash after; off an edge, the
kick goes on in the air and ends sooner. Distances (dash, rise, slide, dive) should be compared with footage when the owner has some.

### For the art agent

Moves that want their own clip (they fall back to the shared archetypes now): the punch (wind-up 1-52, the fist out 53-57; the reversed
one turns on 35 and hits 62-66), the dash and its uppercut (search 10-25, the uppercut 5-7) and the aerial overhand (5-6), the leap (rise
14-30, the grab hand forward; the catch: hold, explosion 2-3, throw on 17, backflip), the slide kick (13-33, the leg forward along the
ground) and the dive kick (14-27, diagonal), its landing (1-2), the wall crash (53 frames).

## Owner decisions on this port (answered)

15 (rapid jab presses: the press that starts jab 1 counts; mashing keeps it going), 16 (E6 and E21 as hit kinds in `resolve_hits`), 17
(fix the shield stun rounding for everyone), 18 (keep the 42-frame neutral air dodge unless a second source disagrees: it agrees). See
`docs/movesets/ROSTER_STATUS.md`, "Owner decisions".

## Still open

* The root-motion distances (gap 1), the aerial punch's boost speed (gap 2), landing during the aerial punch (gap 3), per-airtime limits
  (gap 4), the catch reaching the ground or a ledge-hanging fighter (gap 5): footage or a published figure would settle them
  (owner decision 12: fit to published figures; none found so far).
* Limits kept from phase 1: the forward air's late-hit trip (E20), one get-up attack slot, the throws' interrupted rows and thrower
  invincibility, grabs on aerial targets (ordinary grabs).

## Sim 130: up specials halt their starting momentum

Owner decision 97 (sim 130, branch `upspecial-momentum`): up specials halt the momentum they start with, as the reference does. The leap (up special) now sets `start_speed_y` 0: its 13-frame startup no longer carries a jump's rise (it still falls by the body's gravity from rest, as before from a standstill) [E: reading; no startup multiplier is published]. Before, out of a jump the move rose higher than from a standstill at the same height; now the two match (`sim-core/tests/upspecial_momentum.rs`). The audit of every fighter and its sources: `docs/movesets/SHARED_MECHANICS.md`, "Starting momentum of specials".

## Sim 143: the leap's height (owner decision 102)

The owner: "Sprinter's up special should gain more height." The leap's rise was an [E] (34 units, ending on frame 31, with the body's
gravity in the startup: 25 units net in the air). It is now fitted to the reference's hitbox visualisation (`REFERENCE_DATA.md`, "Fitted
distances"): held still through the startup (no gravity on frames 0-12), up to a peak on frame 45 (53 reference units in the air, 62
from the ground), then the flip down to 15 / 18 units above the start as the special fall begins. Measured (`recovery_rise`): air
peak 52.6 (before 25.5), ground 62.5 (before 34.9). The stick drifts it again from frame 30 (x0.95, the move's air block). The rise's
horizontal drive is unchanged [E]. Tests: `recovery_rise::the_sprinters_leap_holds_still_then_rises_to_the_fitted_peak`,
`gauntlets_specials::the_leap_rises_and_cannot_fast_fall_before_frame_50`, `the_up_specials_frame_data_is_the_references`.
