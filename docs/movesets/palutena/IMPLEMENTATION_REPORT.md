# Implementation report: the sixth moveset (reference: the reference game's Palutena)

In game: class 5 **Staff** (weapon `staff`), body `seer`, built-in fighter **Seer** (owner decision 19). Target version 13.0.1. Data:
`REFERENCE_DATA.md`; what the engine had and needed: `PROJECT_AUDIT.md`.

## Phase 1 (sim 67, content schema 5 unchanged): the body and every normal

Branch `seer-phase1` (from `movesets` at `02a505f`, sim 66). The parallel Gauntlets phase 2 (E1, E3c, E6, E11 additions, E21, E22, the
`wall` register, the shield-stun rounding fix) is on another branch and was not touched; both bump to 67, to be resolved at merge.

### Built

* **Body** `FighterParams::seer` (`sim-core/src/content.rs`): every attribute and parameter-file value in `REFERENCE_DATA.md`
  "Attributes": weight 91; walk 1.271, initial dash 2.035, run 2.077, traction 0.106; walk acceleration 0.105 + 0.21, run 0.044 +
  0.12991; a 10-frame initial dash; **heavy landing 6 frames of lag** (the parameter file's `landing_heavy_frame` 7, owner decision
  22); air speed 1.0, air acceleration 0.01 + 0.105, air friction 0.01, gravity 0.12, fall 1.55 / fast fall 2.48; full hop 35.9, short hop
  17.3, double jump 35.9, ledge jump 37.95 (drift 0.6); air dodges 49 (3-30) and 70 / 82 / 105 (3-21), rolls 29 / 34, spot dodge 25
  (UFD's numbers, as for every body); **no wall jump** (the parameter file), the wall-tech jump 1.3 away, 2.2 up. Taller hurtboxes
  (`ecb_height` 2.6, as the sprinter) [E].
* **Every normal** (`sim-core/src/moves_staff.rs`, a child module of `moves.rs`), on [R] frames, damage, angles and angle types, BK, KS,
  set knockback and set weight, hitlag / SDI / shieldstun multipliers, hitstun modifiers, shield damage and target filters. Offsets on
  the root bone are converted by one helper (`u`: x 1.25, rounded half away from zero) [D]; reference capsules are circles along them
  (`along`) [D]; boxes on the staff bone are placed on our body [E]. `Weapon::limbs` is false (a held staff and shield).
  * **Jab 1** (8-9, 3%, +4 hitstun, hitlag x1.6, the far box pulls at 180, FAF 25) and **no jab 2 or 3**. Its script goes into the rapid
    jab on frames **10-24** once attack has been **pressed and let go twice since jab 1 began** (`presses` / `releases`, the press that
    started it counted, presses in hitlag counted: the Gauntlets' rule), or on **frame 13 with attack held when jab 1 has hit**.
  * **Rapid jab**: hits on its 4, 7 ... 22 (seven a cycle), 0.6%, 361, BK 10, KS 20, hitlag x0.5, SDI x0.4, **shieldstun x7**; the first
    cycle (`ext10`) frames 1-23, later cycles (`ext11`) frames 2-23 [D: the loop points read as the Gauntlets' were]; after one cycle,
    letting go gives the **finisher** (`ext12`: 3.5%, 361, BK 55, KS 140, hit 3-4, FAF 40). The **quickest loop is reproduced tick for
    tick**: jab 1 hit 8-9, rapid jab from 10 with hits 13, 16 ... 31, finisher from 33 with its hit on 35-36, control back on 72.
  * **Forward tilt**: hit 1 on 14-23 in three phases of set knockback with set weight (head 90 / 285 / 20 degrees, FKV 25 / 25 / 35; tail
    45 / 100 / 85, FKV 45 / 40 / 30; hitlag x0.8, SDI 0), hit 2 on 24-35 (7%, 40, BK 78, KS 80), FAF 60. **Up tilt**: five 1.4% loop hits
    every 3 frames from 8 (autolink top box, 130 and 190 degree sweeps, set knockback 20 / 50 / 30) and the 4.5% last hit on 24-25 (85,
    BK 75, KS 136), FAF 56. **Down tilt**: 8.5% on grounded targets, 5% on aerial ones, 14 and 15-27 (the staff along the floor from 15),
    FAF 40. **Dash attack**: clean 6-9 (11%, 70 / 60), late 10-17 (7%), FAF 41 (11.0.0), **head and torso intangible 5-6**.
  * **Forward smash** (18-20, 16% inner wing wins over 13% tips; then **the wind as pushes re-applied every frame**, FKV 60 on 21-25 and
    35 on 26-31; charge 10; FAF 64). **Up smash**: the pillar of light, three columns up to 21 / 44 / 65 reference units (16 / 12 / 9%,
    late 13 / 9 / 7%), charge 12, FAF 64. **Down smash**: 15% on 17 (60 degrees, BK 52), 13% on 18-19 (28), the wind on 20-29 (FKV 55),
    both sides by position; charge 13; FAF 69.
  * **Neutral air**: six 1.4% loop hits every 4 frames from 5 (autolink on aerial targets, set-knockback 100 / 95 degree pop-ups on
    grounded ones), **the late loop 24-27 with set knockback 65 and -13 hitstun on every box**, the 5.1% last hit on 29-30 **always sent
    forward** (angle type 3), landing lag 12, autocancel 1-3 / 40+, FAF 52. **Forward air** 9-11 (10%, 45), landing lag 9, FAF 45.
    **Back air** 8-10 (12%, 361 **backward**, BK 30, KS 95), **head and torso intangible on 7**, landing lag 10, FAF 50. **Up air**: five
    1% loop hits every 3 frames from 10 (105-degree set knockback on grounded targets, autolink on aerial ones) and the 5% last hit on
    24-25 (88, BK 50, KS 152), **every box backward** (type 4), landing lag 14, FAF 52. **Down air** 10-11: the meteor on aerial targets,
    55 degrees backward on grounded ones, landing lag 11, FAF 59.
  * **Grabs** (standing 7-8 / 40 with the 11.0.0 reach, dash 10-11 / 48, pivot 11-12 / 43), **pummel** (1.3%, one frame later [D]),
    plain **throws**: forward 9% on 20 (FAF 45), back 10% at 361 behind on 20 and turning round (FAF 55), up 8% at 92 on 17 (FAF 55),
    down 5% at 73 (BK 90, KS 30) **released on 33 with FAF 54**: the reference's release 25 and FAF 46 plus its 8 frames of hitlag on 24,
    folded into the timeline since nobody is frozen here [D].
  * **Ledge attack** (23-25, 9%, 45 forward on both sides, intangible to 25), **get-up attack** face up (behind 16-17, front 25-26, 7%,
    48, intangible to 26).
* **Specials**: the claws' placeholders (as the Embers and Gauntlets had in phase 1). The rapid jab uses `ext10` to `ext12`.
* **Registered**: `CLASSES` 6 and `Recipe::params` (`sim-content/src/recipe.rs`), `Content::placeholder` (fighter `seer`, weapon
  `staff`), `godot/scripts/roster.gd` (`CLASS_NAMES` "Staff", the audit's blurb adapted, the built-in Seer), `content/base.pfc`
  re-exported. Tests that count classes or fighters updated: sim-content `doc`, `recipe`, `tests/bundle.rs`, `tests/recipes.rs`
  (`MADE` 6); netplay `tests/fighters.rs` (class 6 is now the unknown one); godot `content_e2e`, `creator_flow_test` ([0..5]),
  `editor_api_test` (the made fighter is now index 6), `online_fighters_test` (eight and six fighters).

### General extensions (each defaults to the old behaviour)

| Id | What | Where | Tests |
| --- | --- | --- | --- |
| E26 | **Angle types**: `Hitbox::angle_type` = `ANGLE_BY_BOX` (0, the default: a box behind the attacker sends backward, as before), `ANGLE_FORWARD` (the reference's type 3), `ANGLE_BACKWARD` (4), `ANGLE_BY_POSITION` (0: "away from the attacker based on the target's position", from ssbwiki.com's hitbox template, so from the attacker's position, not the box's). `combat::launch_side`, used for melee hits and projectile hits (a projectile's "forward" is its direction of travel; by position is from its centre). Content `angle_type box / forward / backward / position`, hashed | `moves.rs`, `combat.rs`, `format.rs`, `docs/CONTENT.md` | `multihit::angle_types_choose_which_way_forward_is`, `staff::the_neutral_airs_last_hit_sends_forward_and_the_up_air_backward_on_both_sides`, `staff::the_new_fields_are_in_the_content_hash` |
| E27 | **Signed hitstun modifier**: `Hitbox::hitstun_add: i16` (was `u8`); a launched victim keeps at least one frame [E: the reference floors at 0 and still launches; 0 here would turn the launch into a flinch]. Content accepts negative values; no schema change (old values mean the same) | `moves.rs`, `combat.rs` | `multihit::a_hitstun_modifier_adds_frames` (now also -13 and the floor), `staff::the_neutral_airs_late_loop_takes_13_frames_of_hitstun_off` |

### The older fighters' angle types (owner decision 21): what changed

Read from ssbwiki.com's raw hitbox tables (the `af` field of every row, 13.0.1) for every move of the five older weapons that has a box
reaching the fighter's centre or behind it (the only boxes where the type can change the launch side; boxes wholly in front keep the
default, which equals types 0 and 3 there). Each is in `sim-core/tests/angle_types.rs`.

| Weapon | By position (type 0) | Forward (3) | Backward (4) |
| --- | --- | --- | --- |
| Longsword | up tilt, up smash, down smash, neutral air, up air, down air, get-up attack | forward smash, forward air | back air (the reference's is type 3, but its fighter has turned round when it hits: backward from the starting facing) |
| Claws | up tilt, down tilt, up smash, down smash, neutral air, up air, down air, the up special's wind-up hit, the reflector's hit, get-up attack (inherited by the Needles) | the up special's last hits (`ext0`, `ext1`) | back air |
| Needles | up tilt hit 2, down tilt, up smash, neutral air, the up air's aerial loop box and final hit, down air, up special | forward tilt, up tilt hit 1, the down smash's front boxes, forward air, the up air's grounded loop box, the down air's landing hit, both down-special kicks | back air, the down smash's back boxes |
| Embers | up tilt, up smash, down smash, neutral air, up air, down air and its landing hit, the back throw's hits on bystanders, ledge attack, get-up attack | jab 2, forward tilt, forward air, up special | back air |
| Gauntlets | down tilt, the up smash's first-hit boxes 0-2, down smash, up air, down air, get-up attack (and the claws' placeholder specials as above) | forward tilt, the up smash's box behind the head and its second hit, neutral air, forward air, ledge attack | back air |

**What this changes in play**: only the type-0 boxes that straddle the fighter or sit behind it while a target is on the other side:
a target in front of the Embers' up smash's early boxes (behind the head) is now sent forward, not back over the head (tested:
`angle_types::a_target_in_front_of_the_embers_up_smash_is_now_sent_forward`); the same for the up tilts, up airs and neutral airs
sweeping across the top, the down smashes' and get-up attacks' boxes near the middle, and the Gauntlets' down tilt box at the body (the
limit listed in its report, now fixed). Every type 3 and type 4 box set above already sat on the side that sent that way, so they change
nothing now. Not changed: the Claws' up special loop is the reference's **type 1 (speed-based: from the direction of travel)**, which is
not built; it keeps the default (the known limit in its row). The Needles' dash attack is the earlier game's (owner choice) and keeps
the default. Every existing kit test stayed green.

### Verified against [R] / [D] (by tests that ran)

`sim-core/tests/staff.rs` (22 tests): every normal's windows, FAF, landing lag and autocancel in the data, charge frames, rehit
rhythms; first-hit frames and FAF by running each move; damage / angle / BK / KS of every phase and the multipliers, set knockback,
targets, shield damage and angle types; the quickest rapid jab tick for tick, one press or a held button without a hit gives jab 1
alone, held on a hit goes in on frame 13, the 22-frame loop while held and the finisher on letting go (mashing keeps it going), the
rapid jab's x7 shieldstun; the up tilt's, neutral air's and up air's loop counts and last hits on a held target (11.5%, 13.5%, 10%);
the late neutral air's -13 hitstun; the neutral air's last hit forward and the up air backward on both sides, the back air backward;
the forward smash's wind pushing on each of frames 21-31 with no hitstun; the dash attack's and back air's head-and-torso
intangibility; grab on frame 7, throws' release frames and damage, back throw behind and turning round, pummel; aerial landing lag and
autocancels; jump air times (34 / 50 / 24 / 35 within a frame); body attributes; the new fields in the content hash; random play
deterministic and rolling back. Also `angle_types.rs` (2), `multihit.rs` (the two E26 / E27 tests), `ground_attributes.rs` (the
seer's ground numbers; the "no two share a traction" check dropped: the rogue and the seer both have the reference's 0.106).

### Estimated or limited [E]

* Hitbox positions on the staff bone (forward tilt, down tilt, neutral air, the arm boxes) and the body's height (2.6).
* The rapid jab's cycle reading (frames 2-23, the loop points "1-2 and 23-24" read as the Gauntlets' "2-3 and 19-20") and its "let go"
  rule (not held and not pressed for 8 frames), the Gauntlets' open question 2.
* The hitstun floor of 1 frame (E27).
* (Done in sim 73 on the cleanup branch: the rapid jab, finisher, dash attack, smashes, down air, pummel, ledge and get-up attacks
  carry `clanks = false`, the reference's `clang=f`.)
* The **arm intangibility windows** (forward tilt 14-35, dash attack 5-16, back air 7-10) are recorded but act on nothing: with a held
  weapon the fourth hurtbox circle is the middle one, so the arm is inside the body's circles. The head-and-torso windows work.
* The down throw's 8 frames of hitlag are folded into its timeline (release 33, FAF 54); the held fighter's invincibility meanwhile and
  the thrower's throw invincibility are not modelled (house limit).
* Not modelled (house limits): the jab's fourth box (objects only), the grabs' thin aerial-target boxes, the throws' interrupted rows,
  the face-down and trip get-up attacks (one slot), jab lock, jostling.

### Training-mode check (hitbox view)

Seer vs a standing duelist at 0%: (1) forward tilt, the staff's head and tail in front at chest height turning through 14-35; (2) up
tilt and up air, the sweep reaching 1.3 units in front to 0.6 behind over the head; (3) the up smash pillar 1.2 units ahead, from the
floor to 8 units up; (4) neutral air, the staff's ends 0.7 units either side of the chest; (5) a target behind the neutral air's last
hit flies forward over the seer. Adjust the staff positions if they sit off the staff in the clips.

### Tests and checks run

`cargo fmt --all`; `cargo test --workspace` (all green); `cargo clippy --workspace --all-targets -- -D warnings` (clean); `pftool
fuzz-rollback 1000` (0 mismatches); `pftool net-fuzz 200` (0 mismatches); `pftool content-check content/base.pfc` (ok). **Not run**:
the Godot tests (`godot/tests/*.gd`): updated for the sixth class, but they need the editor and a built bridge.

### For the art agent

* An emblem for class 5 (Staff) in `godot/ui/match_hud.gd`, and a default look for the built-in Seer (it uses `Loadout.default_for(5)`,
  the same as the Brawler's).
* Moves that want their own clip (they fall back to the shared archetypes now): the jab (staff thrust, 8-9) and the rapid jab (a
  flurry of light in front, a hit every 3 frames from 4, 22-frame cycle) and its burst finisher (3-4); the forward tilt (staff spun in
  front, 14-35); the up tilt and up air (staff spun overhead, 8-25 and 10-25); the down tilt (staff swept along the floor, 14-27); the
  dash attack (shield lunge, 6-17); the forward smash (wings of light, 18-20, then the wind to 31); the up smash (a pillar of light 1.2
  units ahead, 18-35); the down smash (a burst on both sides, 17-19, then wind to 29); the neutral air (staff spun round the body, 5-30);
  the forward air (a palm pulse, 9-11); the back air (shield bash behind, 8-10); the down air (staff driven down, 10-11); the back throw
  (turns round on 20).

## Phase 2 (sims 74 onward, branch `seer-phase2`): the specials

Branch `seer-phase2` from `movesets` at `17e91f7` (sim 72, after the Gauntlets' phase 2). A parallel cleanup branch takes sim 73; this
one starts at 74 and the two are reconciled at merge. Built in the owner's order, each milestone tested and committed before the next.
Tests: `sim-core/tests/staff_specials.rs`.

### Milestone 1 (sim 74): Warp (up special)

`moves_staff.rs` `warp`, script `scripts/staff_warp.script`. No new engine field: the E11 addition the plan called
`land_continue_to` already exists as **`Move::lands_from`** (built with the Gauntlets' E11 additions: "touching down before this move
frame does not end the move"), which is exactly that frame. The claws' follow-up slots `ext0` to `ext2` are cleared.

| Part | Built | Label |
| --- | --- | --- |
| Startup 1-15 | an air block on move frames 0-14: gravity 0.03, top fall 0.75 (reference units) | [R] |
| Direction | read on frame 16 (move 15) from the stick, normalised; under 0.2 of its travel it is straight up; the tilt does not change the distance | [R] frame; [E] neutral = up, magnitude ignored, any angle (not snapped), facing unchanged |
| Travel | 17 frames (16-32) at 3.5 reference units (0.4375 world) along it: 7.4375 world units | [R] / [D] |
| Intangible | 16-32 (`intangible_from` 15, `intangible` 32) | [R] |
| Against the stage | a motion: it follows the ground if it meets it (never sinks in; glides on, leaves at an edge), stops at walls and ceilings; touching down before frame 33 carries on (`lands_from` 32) | [E] gap 9 |
| Reappearance | frame 33 at half speed once, then the body's own air physics (in the air) or the ground's friction | [R] x0.5; [E] gap 10 for the physics |
| End | FAF 68; ending in the air: helpless, landing lag 22; landing during the reappearance: 22 | [R] |
| Edge-cancel | a grounded reappearance sliding off an edge ends in an ordinary fall (the shared rule in `slide_on_platform`) | [R] behaviour |
| Ledges | front and back from frame 33 (`ledge_front_from` / `ledge_back_from` 32) | [E] gap 8 ("more easily" since 2.0.0, no numbers) |

Tests (all run): `the_warps_data_is_the_references`, `the_warp_travels_59_5_reference_units_straight_up_intangible_on_16_to_32` (the
exact distance, the half-speed frame, intangibility frame by frame), `the_stick_on_frame_16_aims_the_warp_and_later_changes_do_not`
(45 degrees, neutral, a light tilt the full distance), `an_aerial_warp_ends_helpless_and_lands_with_22_frames`,
`landing_during_the_reappearance_costs_22_frames`, `a_warp_into_the_ground_glides_along_it_and_reappears_standing`,
`a_grounded_warp_along_the_stage_ends_standing_and_one_sliding_off_an_edge_is_cancelled`,
`the_warp_catches_a_ledge_from_the_reappearance_on`. Workspace tests, clippy, fmt, `fuzz-rollback 1000` and `net-fuzz 200` at 0
mismatches.

### Milestone 2 (sim 75): Counter / Reflect Barrier (down special)

`moves_staff.rs` `counter_and_barrier`: the stance (down special), the counterattack (`ext0`), the reflection (`ext1`).

**Engine (E10c and two small additions; each defaults to the old behaviour, no schema change: older content means the same):**

| Field | What | Where |
| --- | --- | --- |
| `Counter::melee_only` | a projectile's hit is not caught (it lands as usual) | `combat::catch_with_counter` (told whether the hit is melee) |
| `Counter::max_damage` | the answer's damage is at most this (0: no cap) | `catch_with_counter` |
| (rule) | a counter whose window is open catches **through its own intangibility** (the reference's counters do: their intangibility and catch windows overlap). Only a fighter in an open counter window is affected, so no existing fighter changes (the Longsword's stance has no intangibility) | `combat::counter_open`, `resolve_hits`, `update_projectiles` |
| `Reflector::detect_then` | a shot touching the detector **starts that move at once** (with its intangibility), so its reflector meets the shot on the same frame; the detector then works on its whole window | `combat::reflector_detects` |
| `Reflector::half_height` | the field as a standing capsule (the barrier) instead of a circle | `combat::reflector_touching`, `overlaps_upright` |
| `Reflector::body_*` | a second circle with the same numbers on its own frames (the body reflector beside the barrier) | `reflector_touching` |
| `Move::air_max_rise` | the upward speed is held at or under it while the move runs without a motion in the air (the full hop's opening is cut too) | `fighter::attack` |

Content: `counter { max_damage melee_only }`, `reflector { detect_then half_height body_start body_end body_x body_y body_radius }`,
`air_max_rise` (`docs/CONTENT.md`); validation: `detect_then` must be a real move, `max_damage` not under `min_damage`.

| Part | Built | Label |
| --- | --- | --- |
| Stance | FAF 71; intangible 6-12; counter 7-34 (130%, min 10, max 50, melee only, answer `ext0`); the projectile search box 7-34 (radius 12.3 reference units on the chest: 1.5 at 0.2 ahead, 1.3 up) starting `ext1` | [R]; positions [D] |
| Stance air | x0.33 on starting, friction 0.0025, gravity 0.04, top fall 1.2, top rise 0.8; touching down carries on | [R]; landing [E] gap 4 |
| Counterattack | hit 5-7 (decision 22), intangible 1-5 (the majority), FAF 37; id 0 BK 50 / KS 70 (r 7.0, 10.5 up, 13.0-14.5 ahead) over id 1 BK 30 / KS 65 (r 9.0, 9.0-20.0 ahead); 361 forward, x1.5 hitlag, no clank; the caught damage; turned toward whoever struck; the caught hit's hitlag only before it (conflict 3) | [R]; positions [D]; the freeze [E] |
| Reflection | FAF 35; body reflector 1-4 (r 11.5 at 10 up, 1.5 ahead); the barrier a capsule r 3.8 from 2 to 21 up, 10 ahead, to the end of the move; x1.4 damage, x1.25 speed, x1 life, limit 50; the push on 2-11 (set knockback 50, set weight, 361 forward, re-applied every frame as the smashes' wind) | [R]; positions [D]; **barrier ending with the move: owner-chosen approximation (decision 20)** |
| Both answers' air | the stance's numbers | [E] |
| Reflected shot | horizontal speed turned round only | [E] gap 12 |
| Counter bubble | the reference catches on its own bubble (r 11.5 at 10 up); here the body's hurtboxes | [E] |

Tests (all run): `the_counter_and_barriers_data_is_the_references`, `a_melee_hit_in_the_window_is_countered_for_1_3_times_between_10_and_50`
(a jab answered with the 10% floor and the counterattack dealing it; a 45% hit answered with 50), `the_counter_catches_through_its_own_intangibility`,
`a_shot_touching_the_search_box_starts_the_reflection_and_is_never_countered` (the reflection on the touching frame, the shot the
seer's at x1.4 power and x1.25 speed), `a_shot_outside_the_window_hits` (before and after), `the_reflections_barrier_reflects_shots_at_its_height_and_not_above`,
`a_melee_hit_during_the_reflection_is_taken` (the two answers never both), `the_barriers_push_moves_a_fighter_without_hitstun`,
`the_stance_holds_the_rise_at_0_8_and_keeps_a_third_of_the_speed`, `the_down_specials_new_fields_are_in_the_content_hash`. Workspace
tests, clippy, fmt, `fuzz-rollback 1000`, `net-fuzz 200` at 0 mismatches.

### Milestone 3 (sim 76): Explosive Flame (side special)

`moves_staff.rs` `explosive_flame`, script `scripts/staff_flame.script`.

**Engine (E3a as the audit gives it, the `flick` register, one new script function; defaults leave every older projectile as it
was, no schema change):**

| Field | What | Where |
| --- | --- | --- |
| `ProjectileSpawn::pierce` | not used up by a hit: it goes on through every fighter it touches to the end of its life | `combat::update_projectiles` |
| `ProjectileSpawn::rehit` | a fighter already hit by a group is hit by it again this many frames after its last hit on that fighter (per target) | `update_projectiles` |
| `ProjectileSpawn::phases` | up to **8** (`MAX_PHASES`, was 2); a phase of radius 0 is a gap (no hit); `moves::phase_list` builds them | `moves.rs`, `format.rs` (`phase` blocks up to 8) |
| `Projectile::hit_mask`, `hit_age[4]`, `facing` (hashed, checksum cases) | who it hit by group (`combat::hit_bit` layout), its age at its last hit on each fighter, its owner's facing at spawn (turned round by a reflection, which also clears its hits): "forward" for a projectile standing still (E26 by position, forward and backward then work for it) | `state.rs`, `combat.rs` |
| `Fighter::special_flick` (hashed, checksum case) and the **`flick`** register | the special was input like a smash attack: the stick flicked (sideways, up or down, within the smash-flick buffer) or the strong button held; set when a special starts | `fighter::start_special`, `scripting.rs`, `sim-script/src/api.rs` |
| **`spawn_at(x, y, speed_forward, speed_up)`** and `Fighter::spawn_fixed` (hashed, checksum case) | `spawn` at an absolute point (a `spawn` offset is applied after the fighter's move that frame, so a falling seer's explosion would land low); found while testing the aerial version | `scripting.rs`, `combat::spawn_projectiles` |

E3c for projectiles already existed (`Hitbox::clanks` false skips `clank_projectile`) and E26's by-position side for a projectile
already measured from its centre; both are used as built.

| Part | Built | Label |
| --- | --- | --- |
| Point | where the seer stood as the move began, 45 reference units ahead (75 with `flick`), 10 up | [R] ahead; [E] height (gap 5); [E] a B-reverse turns the point round |
| Made | on frame 27 (move 26) by `spawn_at`, whatever the seer did since (falling, carried) | [R] |
| Loop | 1.5%, 160 by position from its centre (BK 50, KS 100, SDI x0.4, shield 53%), on ages 1-28 (frames 28-55), per target every 5 frames (28, 33, 38, 43, 48, 53 on a fighter inside throughout); radius 6 / 8 / 9 / 11 / 12 / 14 / 15 tenths from frames 28 / 32 / 36 / 40 / 44 / 48 / 52; nothing on 56 | [R] / [D] |
| Last hit | frame 57 (age 30, group 1): 5.5%, 84 the seer's way, BK 60, KS 141, radius 19 tenths, hitlag x1.5, shield 51%; life 31 | [R] |
| No clank, reflectable | `clanks` false on every box; a reflector takes it over (it stays put) | [R] |
| Move | FAF 72; x0.8 horizontal speed in the air; the body's air physics; touching down carries on | [R]; [E] gaps 4, 6 |
| Hitlag | the victim's hitlag does not freeze the explosion (its rhythm is by its own age) | [E] |

Tests (all run): `the_explosive_flames_data_is_the_references`, `the_flame_appears_on_frame_27_45_units_ahead_or_75_with_a_flick`,
`the_flame_stays_where_the_seer_stood_on_frame_1` (falling), `a_shielding_fighter_inside_it_is_hit_on_28_33_38_43_48_53_and_57`,
`a_fighter_walking_in_late_is_hit_every_5_frames_from_its_first_hit` (30, 35 ... 55, 57), `the_flame_pierces_and_pulls_each_target_back_toward_its_centre`
(two fighters hit on one frame, sent at 160 and 20), `the_flame_never_clanks_and_its_new_fields_are_hashed`; `scripted_moves::a_script_can_fire_a_projectile_at_a_fixed_point_while_the_fighter_moves`;
checksum cases in `state.rs`. Workspace tests, clippy, fmt, `fuzz-rollback 1000`, `net-fuzz 200` at 0 mismatches (note: `pftool`'s fuzzers
play classes 0 and 1 only; the seer's rollback is covered by `staff.rs` `random_play_with_the_seer_is_deterministic_and_rolls_back` and the
specials' own rollback test).

### Milestone 4 (sim 77): Autoreticle (neutral special)

`moves_staff.rs` `autoreticle`, script `scripts/staff_reticle.script`.

**Engine (the E16 target lock):**

| Piece | What | Where |
| --- | --- | --- |
| **`lock(range)`** and the registers **`locked`, `lock_x`, `lock_y`** | once this attack has a lock it returns 1; otherwise it looks for the nearest other fighter (lowest index on a tie) that can be targeted (active, stocks left, not intangible), in front (`(target - self) x facing > 0`), chest within `range` of the fighter's chest, with no solid block across the line between the chests, and keeps that chest as the lock point | `scripting.rs` (`FighterHost::lock`), `sim-script/src/api.rs` |
| `Fighter::lock` (Vec2), `Fighter::locked` (hashed, checksum cases) | the lock point; cleared when a new attack begins (`begin_attack`), kept through `goto` | `state.rs`, `fighter.rs` |
| `scripting::World` | what a move script sees of the others: every fighter's chest height and, for those that can be targeted, its chest; a snapshot taken at the start of each frame before anyone moves (deterministic; a fighter that moves this frame is seen where it began it) | `step.rs`, passed through `fighter::update` / `attack` to `run_fighter` |
| `collision::segment_blocked(stage, a, b)` | whether a solid block lies across a segment (its inside: touching an edge is not crossing), slab clipping in fixed point; pass-through platforms never block | `collision.rs` |
| `combat::chest_height` | the middle hurtbox circle's height (the "chest" of every body) | `combat.rs` |

| Part | Built | Label |
| --- | --- | --- |
| Search | frames 7-14 (move 6-13): `lock(15)`: 120 reference units chest to chest, in front, not intangible, line of sight | [R] rules; [E] the "in front" region (a half-plane), chest to chest instead of the search circle against the hurtbox, pass-through platforms not blocking, the nearest wins (gap 2) |
| Reticle | stays where the target's chest was | [R] |
| Shots | with a lock, made on 23 / 29 / 35 (move 22 / 28 / 34, hitting from the next frame) at the staff (1.0 ahead, 1.3 up) straight at the reticle at 0.75 world units a frame (6 reference), life 25; 3.5%, 361 forward, BK 40, KS 41, radius 3 tenths, shield 51%; not piercing, reflectable, they clank | [R] numbers; [E] the muzzle and the straight path, carrying on past the point (gap 1) |
| No lock | nothing comes out; the move plays out to FAF 68 | [R] |
| Air | x0.6 horizontal speed on starting; the body's physics (the descent stall has no published numbers); touching down carries on | [R]; [E] gaps 3, 4 |

Tests (all run): `the_reticles_data_is_the_references`, `a_locked_reticle_fires_three_shots_on_23_29_35_at_the_point`,
`without_a_lock_the_reticle_fires_nothing`, `the_reticle_stays_put_when_the_target_moves`, `a_reticle_shot_hits_for_3_5_percent`,
`the_specials_are_deterministic_and_roll_back` (every special in a row, replayed straight and from a snapshot mid-move);
`scripted_moves::lock_finds_the_nearest_opponent_in_front_and_keeps_its_chest`, `lock_ignores_fighters_behind_out_of_range_intangible_or_behind_a_block`,
`segment_blocked_is_a_line_of_sight_through_solid_blocks_only`; checksum cases `lock`, `locked`. Workspace tests, clippy, fmt,
`fuzz-rollback 1000`, `net-fuzz 200` (0 mismatches), `content-check content/base.pfc` (ok).

### Phase 2 summary

**Slots**: `ext0` the counterattack, `ext1` the reflection; `ext10`-`ext12` the rapid jab (phase 1). The warp, flame and reticle are
one move each. **Commits** on `seer-phase2`: sim 74 warp (`b1ed63f`), sim 75 counter / barrier (`9e90b5e`), sim 76 flame (`a22f7dd`),
the merge of `movesets` with the sim 73 cleanup (`f90ba8c`), sim 77 reticle. **Schema**: unchanged (5): every new field is optional
with a default meaning the old behaviour. `content/base.pfc` re-exported at every milestone.

**Verified against [R] / [D] by tests that ran**: every special's frame data (startup, active frames, FAF, intangibility, landing
lag), the warp's distance, direction and endings, the counter's ratio, floor and cap and its melee-only rule, the reflection on the
touching frame with its multipliers, the barrier's height, the push without hitstun, the stance's air numbers, the flame's placement,
timing, rhythm per target, piercing, growth and pull toward the centre, the reticle's search rules, shot frames, speed and damage;
determinism and rollback of all of it.

**Estimated [E] (listed per milestone above)**: the warp's neutral direction, ledge windows, stage contact and reappearance physics;
the counter's catch on the hurtboxes (not the bubble) and its freeze (conflict 3); both answers' air physics; the reflected shot's
vertical speed; the flame's height and its B-reverse; the reticle's "in front", muzzle and path; every aerial special's landing
(gap 4) and the reticle's descent stall (gap 3).

**Owner-chosen approximation**: the barrier ends with the reflection move (decision 20; no E25).

**Remaining limits**: the barrier's ten frames past FAF 35 and staying when the seer is hit; a push (windbox) is caught by any open
counter like a hit (shared, unchanged); `pftool`'s fuzzers play classes 0 and 1 only (the seer's rollback is covered by its own tests);
the Godot tests were not run (no editor here).

### For the art agent (phase 2)

Moves that want their own clip (they fall back to the shared archetypes now):

* **Up special, warp** (`up_special`): a vanish on frame 16, the fighter **hidden while intangible (16-32)**, reappearing on 33 and a
  35-frame reappearance; then the special fall.
* **Down special, stance** (`down_special`): guard pose 1-70, the counter window 7-34. **Counterattack** (`ext0`): a strike in front on
  5-7 (boxes 10.5 reference units up, 9 to 20 ahead), FAF 37. **Reflection** (`ext1`): a barrier of light 1.3 world units ahead from
  0.2 to 2.6 up, frames 1-33 (the push 2-11), FAF 35.
* **Side special, explosive flame** (`side_special`): a point marked at frame 1 5.6 (9.4 with a flick) world units ahead, 1.25 up; the
  explosion (a projectile of this move) growing from radius 0.6 to 1.5 on frames 28-55 and bursting to 1.9 on 57; the seer's pose
  plays to FAF 72.
* **Neutral special, reticle** (`neutral_special`): the staff raised on 7-14 (the search), a reticle at the lock point while it
  lasts, three shots from the staff tip (1.0 ahead, 1.3 up) on 23, 29, 35; FAF 68.

## Phase 2 plan: the specials (as written before building)

Slots: the rapid jab holds `ext10` to `ext12`; the claws' placeholders in `ext0` to `ext2` go. The specials need two: the
counterattack (`ext0`) and the reflection (`ext1`).

* **Warp** (up special, one move, no slot): E11's `air_gravity` 0.03 / `air_max_fall` 0.75 for the startup; a script that on move frame
  15 reads the stick (neutral straight up [E], magnitude ignored [E]), calls `intangible(17)` and `set_vel`s 0.4375 world units a frame
  for 17 frames; on 32 half of it once; FAF 68, `helpless_after`, `fall_landing_lag` 22. **E11 addition `land_continue_to: u8`** (0 never,
  255 the whole move, the bool's meaning; added beside `land_continue` or as a schema bump with a migration `true` → 255): the travel
  carries on along the ground to frame 32, the aerial reappearance then lands with its 22 frames. The grounded reappearance's
  edge-cancel already works (`slide_on_platform` ends the move into `Airborne`). Ledge windows wait for footage (gap 8); the view hides
  the fighter while intangible in this move (art).
* **Explosive Flame** (side special, no slot): the script remembers the frame-1 position in two vars and spawns on move frame 26 with
  the offset that cancels the movement since (45 reference units ahead, 75 with a flick: the **E16 `flick` register**, recorded at
  `start_special` into hashed `Fighter::special_flick` from the dash-flick test); `start_speed` 80. **E3a exactly as the audit gives it**:
  `ProjectileSpawn::pierce`, `ProjectileSpawn::rehit` (per-target, from hashed `Projectile::hit_mask` and `hit_age[MAX_FIGHTERS]`),
  `phases` grown to 8 (radius 0 = none), hashed `Projectile::facing` (the owner's facing at spawn: used by E26's forward / backward and by
  a stationary projectile's by-box side). The flame: ages 1 / 5 / 9 / 13 / 17 / 21 / 25 radius 6 / 8 / 9 / 11 / 12 / 14 / 15 tenths (1.5%,
  160 **by position from the flame's centre** with E26: the trap), age 29 none, age 30 the 5.5% last hit (84 forward, BK 60, KS 141,
  hitlag x1.5), life 31; no clank (E3c read in `clank_projectile`). Checksum cases for the three projectile fields and `special_flick`.
* **Autoreticle** (neutral special, no slot): **E16 `lock(range)`** (the nearest active opponent with stocks, not intangible, in front,
  chest within range of the chest, and **`collision::segment_blocked(stage, a, b, pass_through)`** clear between the chests); hashed
  `Fighter::lock` / `locked`, registers `lock_x`, `lock_y`, `locked`, cleared by `begin_attack`. Script: `lock(15)` on move frames 6-13;
  if locked, three `spawn`s on 22 / 28 / 34 straight at the fixed point at 0.75 a frame [E, gap 1]; shot 3.5%, 361 forward, BK 40, KS 41,
  shield 51%, life 25; `start_speed` 60. Whitelist entries + `scripted_moves.rs` tests (nearest in front; ignores one behind, out of
  range, intangible, behind a block; the lock stays put) + doc lines.
* **Counter / Reflect Barrier** (down special, two slots): **E10c**: `Counter::melee_only` (projectile hits not caught),
  `Counter::max_damage` (50), `Reflector::detect_then: Option<u8>` (a shot touching the detector starts that move). The stance: counter
  7-34 (130%, min 10, max 50, melee only, answer `ext0`), a detector-only reflector 7-34 (r 15 tenths at 1.25 / 0.19 [D]) with
  `detect_then` `ext1`, intangible 6-12, air physics `start_speed` 33, `air_friction` 0.0025, `air_gravity` 0.04, `air_max_fall` 1.2 and
  an **E11 addition `air_max_rise`** (0.8). The **counterattack** (`ext0`): hit **on 5-7** (owner decision 22), intangible 1-5 (the
  majority), FAF 37, id 0 BK 50 / KS 70 and id 1 BK 30 / KS 65, 361 forward, hitlag x1.5, `counter_strike`. The **reflection** (`ext1`):
  the body reflector on 1-4 and **the barrier kept on the move** (owner decision 20): a reflector from frame 1 to the end of the move,
  in front at (1.375, 1.19), x1.4 damage, x1.25 speed, limit 50, with the wind (a push, set weight, FKV 50, on 2-11) and FAF 35.
  **Recorded approximation**: the reference's barrier outlives the FAF by ten frames and stays when she is hit; here it ends with the
  move. Tests: melee countered for 1.3x within 10-50; a shot reflected by the new move and not countered; the two never both.
* (Merged: the Gauntlets' phase 2 is in, and the no-clank boxes were set in sim 73.)

## Questions for the owner

Phase 2 (new):

1. **Footage for the [E] readings**: Warp's ledge windows (front / back, from which frame) and what the travel does against the ground,
   walls and pass-through platforms; the explosive flame's height; how the reticle's shots fly (straight from the staff, or steering)
   and its "in front" region; the counter's freeze before the strike (conflict 3). Each is one constant or one line to change.
2. **Counters and pushes**: any open counter (the Longsword's too) catches a push (windbox) like a hit; the reference's counters
   ignore flinchless hits. Change it for every counter (it changes the Longsword's corner case), or leave it?
3. **Counter bubble**: the reference catches on its own bubble (radius 11.5 at 10 up); ours on the body's hurtboxes. Add a bubble
   (`Counter::bubble`) or leave it?

Phase 1:

1. **Angle type 1 (speed-based)**: the Claws' up special loop is the reference's type 1 (from the direction of travel); its row already
   lists "angles from the facing" as a limit. Add it as a fifth angle type (small: the attacker's velocity sign), or leave it?
2. **Hitstun floor**: a negative modifier leaves at least one frame (0 would make the launch a flinch here). Keep?
3. **Rapid jab**: the cycle read as frames 2-23 (as the Gauntlets' 3-20) and "let go" as 8 quiet frames, the same open question as the
   Gauntlets' 2. Keep both the same way?
4. **Arm intangibility with a held staff**: the forward tilt's, dash attack's and back air's arm windows have no hurtbox to act on.
   Leave it (the arm is inside the body's circles), or give held-weapon bodies an arm circle?
5. **Two older data slips seen while reading the tables** (not changed, outside this phase): the Longsword's and the Claws' get-up
   attacks use angle 361 where the reference's are 48 (Marth's and Wolf's floor attacks, ssbwiki.com); fix them?
6. **Footage**: the open behaviours for phase 2 are the audit's (Warp's ledge windows and stage contact, Autoreticle's shot path and
   "in front" rule, the counter's freeze before the strike).
