# Shared combat mechanics: inventory

What the engine can already do for a moveset, where it lives, who uses it, its knobs, its limits and the tests that protect it. Read this
before building anything for a new port: if a mechanic is here, reuse it; if it is close, extend it here (and update this file); only if
it is missing, build it, generally, and add a row.

Built from the code at sim version 55 (content schema 3), October 2026. "Sword" is the Longsword (class 0, duelist body), "Claws" class 1
(brawler body), "Needles" class 2 (rogue body). File paths are from the repo root. Test names are the `#[test]` function names.

How a move flows (traced): `fighter::update` → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` / `start_special` →
`begin_attack` (state `Attack`, `state_frame` 0, `hit_mask` 0, `charge` 0, script vars cleared on entering `Attack`) → each frame
`fighter::attack` (turn windows, smash charge, rehit, projectile request, script, motion or air physics or ground friction, end of move,
jab chain, ledge grab) → after every fighter has moved, `combat::resolve_hits` (collect, clank, apply), `grab::update`,
`combat::update_projectiles`, `combat::spawn_projectiles` (`sim-core/src/step.rs`).

## 1. Move data (`sim-core/src/moves.rs`)

| Mechanic | Where | Parameters | Used by | Limits | Tests |
| --- | --- | --- | --- | --- | --- |
| Move slots | `MoveId` (39 slots) | jab, 3 tilts, dash attack, 3 smashes, 5 aerials, 4 specials, `Jab2`/`Jab3`, grab, dash grab, pivot grab, pummel, 4 throws, ledge attack, get-up attack, `Ext0..Ext9` | all | **Only 10 `ext` slots** for every follow-up (multi-part specials, ground/air variants, counter answers, landing hits). One move per special direction; ground and air variants must branch by script (`grounded` + `goto`). No slots for the reference's separate get-up attacks by facing, ledge attacks by percent, or item throws | `reference_moves::every_special_slot_starts_a_move_and_the_follow_up_slots_cannot_be_started_by_a_button` |
| Reference frame numbering | `ref_move(faf, landing_lag, ac_before, ac_after, rows)`, `row`, `r(...)` | reference frame 1 = move frame 0; `total_frames = FAF - 2` | every reference move | `total_frames`, hitbox frames and autocancel fields are `u8` (a move is at most 255 frames before a script stall or `goto`) | `reference_moves::moves_return_control_on_their_first_actionable_frame`, `needles::the_priority_moves_have_the_references_frame_data` |
| Circular hitboxes | `Hitbox` | `start..=end`, `x`/`y` from the feet (mirrored by facing), `radius`, damage, angle, BK, KS | all | Circles only (no capsules; the reference interpolates hitboxes between frames, here a swing is one row per frame). Positions are always [E] | `combat::a_jab_hits_on_its_first_active_frame_and_not_before`, `out_of_range_attacks_miss` |
| Priority (hitbox id) | `Hitbox::priority` | lowest wins when several overlap one target | Sword tip/blade/arm (`tipper_last`), all | per move only | `combat::the_tip_of_the_sword_hits_harder_than_the_hilt` |
| Hit groups | `Hitbox::group` (0 to 7), `Fighter::hit_mask: u32` | each group hits a target once per move (or once per rehit) | Sword nair, dsmash; Needles uair, smashes; Claws ftilt, usmash | 8 groups per move | `multihit::a_move_can_have_more_than_two_separate_hits`, `reference_moves::neutral_air_marks_both_hit_groups_once_each` |
| Rehit rhythm | `Move::rehit = Some((start, every))`, `Move::rehit_groups` (bit mask) | re-arms the chosen groups every `every` frames from `start` | Claws up special (fire), Sword low Dancing Blade finisher, Needles rapid jab and up air (loop re-armed, final hit not) | one rhythm per move; scripts can re-arm everything with `rehit()` | `multihit::the_rehit_rhythm_re_arms_the_loop_but_never_the_final_hit` |
| Sweet / sour spots | separate rows with priorities and damage | | Sword tipper, Claws bair, Needles | | `combat::the_tip_of_the_sword_hits_harder_than_the_hilt`, `kits::*tip*` |
| Hitlag multiplier | `Hitbox::hitlag` (percent, 100 normal, 0 none) | | Sword aerials, Needles throughout | ignored on a shield when it would shorten hitlag | `hit_feel_2::a_hitbox_can_set_its_own_hitlag` |
| SDI multiplier | `Hitbox::sdi` (percent) | carried to the victim for its hitlag (`Fighter::sdi_mult`) | Needles uair, rapid jab and others | | `multihit::a_hits_sdi_multiplier_scales_the_victims_nudge` |
| Grounded-only / aerial-only hitboxes | `Hitbox::targets` (`TARGETS_ALL`, `TARGETS_GROUNDED`, `TARGETS_AERIAL`) | | Needles uair, dsmash | | `multihit::grounded_only_and_aerial_only_hitboxes_pick_their_targets` |
| Set knockback | `Hitbox::fixed_kb` (the reference's FKV; knockback as if at that percent) | `combat::fixed_knockback` | Needles utilt, dsmash, rapid jab, line | | `multihit::set_knockback_is_the_same_at_any_percent` |
| Extra hitstun | `Hitbox::hitstun_add` | frames added (only when the hit has hitstun) | Needles | | `multihit::a_hitstun_modifier_adds_frames` |
| Shield damage multiplier | `Hitbox::shield_damage` (percent) | | Sword Shield Breaker (200) | no per-hitbox shieldstun modifier; no "unblockable" flag | `kits::shield_breaker_does_double_damage_to_a_shield_and_a_full_charge_breaks_it`, `only_shield_breaker_has_extra_shield_damage` |
| Electric effect | `Hitbox::effect = EFFECT_ELECTRIC` | 1.5x hitlag, two automatic SDI drifts | **no move in the roster** | only normal and electric; no fire/darkness/etc. (presentation only in the reference, except hitlag multipliers) | `hit_feel_2::an_electric_hit_lasts_half_as_long_again_and_drifts_the_victim_with_the_stick` |
| 361 angle | `combat::launch_angle` | the reference's: grounded 0 up to 60 KB rising to 38 at 88; airborne 38 | all | | `combat::the_launch_follows_the_knockback_formula`, formula tests in `combat.rs` |
| Autolink 367 | `ANGLE_AUTOLINK`, `fighter::autolink` | attacker velocity plus a fifth of the way to the hitbox, capped (`autolink_max_speed`, `autolink_max_fall`); grounded victim sent at 80 | Needles uair | | `multihit::autolink_sends_an_airborne_victim_with_the_attacker_and_toward_the_hitbox`, `survival_di_during_hitlag_changes_an_autolink_pull`, `needles::up_air_*` |
| Autolink lead | `Hitbox::link_ahead` (ours) | the pull aims this far in front | Needles uair | | `needles::a_drag_down_up_air_lands_its_target_in_front` |
| Pull angle 368 (ours, not the reference's) | `ANGLE_PULL`, `fighter::pull_frame` | `pull_rate`, `pull_max_speed`, `pull_gap` (Ruleset) | Needles line (side special) | not a reference mechanic; do not use it for a fidelity port | `needles::the_line_reels_a_hit_fighter_in_to_just_in_front` and the two after it |
| Other reference angles | none | | | **Not implemented: 363 (sent toward the hitbox's centre), 365/366 (vector pulls), and the reference's "angle flipper" options** (the engine only flips a hitbox behind the attacker to send backward). Only 361, 367, 368 are special | |
| Aerial landing lag and autocancel | `Move::landing_lag`, `autocancel_before`, `autocancel_after`; `fighter::land_out_of_move` | | every aerial; specials landing in the air also go through it | | `combat::landing_in_the_middle_of_an_aerial_costs_its_landing_lag_but_early_landings_autocancel`, `reference_moves::aerials_autocancel_in_their_reference_windows_and_otherwise_cost_their_landing_lag` |
| Landing hits / landing moves | `Move::landing_move` (an `ext` slot started on a non-autocancelled landing; its length is the landing lag) | | Needles dair (landing hit), Needle Storm charge (carries the charge on landing) | the landing move starts only outside the autocancel windows | `multihit::a_landing_hit_comes_out_on_a_real_landing_and_only_once`, `needles::down_air_*` |
| Scripted motion | `Move::motion` (data) or script `set_vel` | velocity override while active; ground motion follows the ground, leaves at the edge | Claws flash and fire, Needles Vanish, Bouncing Fish, Sword Dancing Blade steps | motion is a velocity, not a published position curve: recovery distances must be fitted [D] | `scripted_moves::a_script_can_move_the_fighter_like_a_data_motion`, `wild_scripts_cannot_push_fighters_into_the_stage` |
| Intangibility | `Move::intangible` (from frame 0), script `intangible(n)` | whole-body only | Sword up special, ledge/get-up attacks, Claws reflector, Needles Vanish | **no partial (per-hurtbox) intangibility** (the reference's head/arm intangibility on up smashes); no invincibility vs intangibility distinction | `reference_moves::up_special_is_intangible_for_its_first_five_frames`, `kits::the_reflector_is_intangible_on_frames_5_to_8_only` |
| Helpless after a special | `Move::helpless_after`, `fighter::enter_helpless` | drift `Ruleset::helpless_drift` (one global value), landing lag `FighterParams::helpless_landing_lag` (one per body, 20 for all) | Sword up special, Claws flash and fire, Needles Vanish | **The reference sets special-fall landing lag and drift per move; here both are per body / global.** `helpless_after` is fixed per move (a script must `goto` another slot to avoid it on hit) | `reference_moves::up_special_hits_on_frame_5_for_11_percent_then_ends_helpless`, `needles::vanish_*` |
| Ledge grab during a move | `Move::grabs_ledge` | any frame of the move | up specials, Bouncing Fish | no per-frame window (the reference only allows it from a given frame, and some moves only while falling) | `reference_moves::an_up_special_grabs_the_ledge_in_the_middle_of_the_move` and the three after it |
| Turnarounds | `Move::turns_around`; up-special reversal (`special_turn_window`); B-reverse (`special_reverse_window`, once per special); script `turn()` | | Sword bair, up specials, all neutral/side specials | | `reference_moves::back_air_sends_the_victim_backward_and_turns_the_attacker_around`, `kits::fire_wolf_*` |
| Smash charge | `Move::charge_at`, `Ruleset::charge_frames` (60), `charge_bonus_percent` (40), `Move::charge_bonus` | | every smash; Shield Breaker uses a 170 bonus | | `reference_moves::holding_attack_charges_a_smash_attack_for_up_to_40_percent_more_damage` and the three after it |
| Jab chains | `Move::next`, `next_window`; slots `Jab2`, `Jab3`; scripted chains | | Sword (2 hits), Claws (3), Needles (2 + rapid jab + finisher, scripts) | | `reference_moves::pressing_attack_again_chains_the_jab_into_hits_two_and_three`, `needles::mashing_attack_goes_jab_jab_rapid_jab_then_the_finisher` |
| Rapid jab loop and finisher | script `goto` loop on an `ext` slot with `rehit` | | Needles | cannot see presses made during hitlag (documented limit) | `needles::holding_attack_through_the_hits_runs_the_rapid_jab_until_let_go` |
| Limb hurtboxes | `Weapon::limbs` | the attacking limb is hittable | Claws, Needles | one limb circle | `combat::the_claws_have_much_shorter_reach_than_the_sword` |
| Hitbox scale | `FighterParams::hitbox_scale` | scales hitboxes, projectile muzzles and sizes, reflectors | created fighters | built-ins use 1.0 | `sim-content/tests/recipes.rs::active_hitboxes_are_scaled_in_size_and_position` |

## 2. Hit resolution and hit feel (`sim-core/src/combat.rs`, `fighter.rs`)

| Mechanic | Where | Parameters | Limits | Tests |
| --- | --- | --- | --- | --- |
| Knockback formula | `combat::knockback` | reference formula, weight, 1.2 one-on-one damage multiplier (`Ruleset::damage_mult`) | | `combat::the_launch_follows_the_knockback_formula`, `a_heavier_target_is_launched_less` |
| Hitlag | `hitlag_frames` | `floor(d * 0.65 + 6)`, cap 30, shield x0.67, crouch cancel | | `hit_feel::hitlag_is_the_reference_formula_for_both_fighters` |
| Hitstun, tumble, launch speed-up, gravity boost, launch fall | `hitstun_frames`, `apply_launch`, `launch_speedup` | `balloon_*`, `gravity_boost*`, `vertical_launch_*` | | `launch_reference::*`, `hit_feel_2::a_strong_launch_is_sped_up_and_its_hitstun_is_shorter` |
| DI, LSI, SDI, ASDI, SDI growth | `apply_launch`, `sdi`, `asdi` | `di_degrees` 10, `lsi_*`, `sdi_interval`, `sdi_combo_*` | | `combat::directional_influence_*`, `hit_feel::survival_di_is_short_and_paced`, `hit_feel_2::survival_di_goes_further_late_in_a_long_combo` |
| Rage, stale moves, short-hop aerial damage, crouch cancel | `rage_multiplier`, `stale_factor`, `apply_hit` | Ruleset | | `hit_feel::*` |
| Hitstun cancel (air dodge / aerial) | `hitstun` | `hitstun_*_cancel`, `*_cancel_speed` | | `launch_reference::hitstun_can_be_cancelled_only_once_the_launch_has_slowed` |
| Light hits land standing; tumble lands in knockdown; techs; missed techs; wall/ceiling techs; bounces | `hitstun`, `tumble_landing`, `wall_tech` | `tech_*`, `knockdown_*`, `bounce_keep`, `ground_bounce_speed` | no wall jump | `knockdown::*`, `needles::a_light_hit_lands_on_its_feet_and_a_tumble_lies_down`, `hit_feel_2::a_tumble_into_a_wall_bounces_off_and_a_tech_holds_on` |
| Clanks and rebounds | `clank_attacks`, `clank_projectile`, `start_rebound` | `clank_range` 9%, `rebound_cap` | grounded attacks only; no transcendent-priority flag | `hit_feel_2::two_jabs_that_meet_clank_and_both_fighters_rebound`, `a_much_stronger_attack_wins_a_clank_and_carries_on` |
| Trades | `resolve_hits` collects before applying | ties to the lower player index | | `combat::two_fighters_hitting_each_other_on_the_same_frame_both_take_damage` |
| Victim state on hit | `apply_hit` | | **No armor of any kind** (super armor, heavy armor by knockback threshold, damage-threshold armor); **no windbox** (a push without damage or hitstun); **no bury**, **no trip**, **no hit that turns the victim around** (a reversal), **no flinchless hits**, **no stun/sleep/freeze**, **no damage-over-time** | |

## 3. Shields, dodges, grabs (`fighter.rs`, `combat.rs::block`, `grab.rs`)

| Mechanic | Where | Parameters | Limits | Tests |
| --- | --- | --- | --- | --- |
| Shield health, stun, pushback, perfect shield, break | `fighter::shield`, `combat::block`, `break_shield` | `Ruleset::shield_*`, `perfect_shield_*` (Smash-4 style perfect shield, kept by the owner's choice) | **Shield numbers are global (`Ruleset`), not per fighter.** No shield bubble geometry: any hit on a shielding fighter is blocked (no shield pokes, no shield tilt), so a different shield model (an egg shield) has nothing to differ from yet. Shield drop lag not modelled | `shields::*` |
| Rolls, spot dodge, air dodges | `roll`, `spot_dodge`, `air_dodge` | per body (`roll_*`, `spot_*`, `air_dodge_*`) | | `shields::*`, `movement.rs` |
| Grabs | `HIT_GRAB` hitboxes, `grab::grabbable`, `grab::connect` | ignore shields; grab parry | **Only grounded, standing targets can be caught**; one hold behaviour for everyone: the victim is pinned `grab_distance` in front of a holder that cannot move (`Grabbing` does nothing but pummel and throw). No command grabs on airborne targets, no grab that leads into a special's own sequence | `grabs::*`, `hit_feel_2::two_fighters_grabbing_each_other_at_once_both_let_go` |
| Hold time and mash out | `grab::hold_time`, `grab::update` | `grab_base_frames` 90, `grab_percent_tenths` 17, `grab_mash_*`, `grab_release_lag`, `grab_immunity`, `grab_distance` | global Ruleset values; the reference's per-character grab release differences are not modelled | `grabs::the_time_held_is_90_frames_plus_1_7_per_percent`, `mashing_buttons_breaks_free_much_sooner` |
| Pummel | `HIT_PUMMEL` | | | `grabs::attack_pummels_*`, `kits::the_sword_pummel_does_1_3_percent` |
| Throws (single and hit-then-throw) | `HIT_THROW`, `throw_after_hit` | | the thrower stays where it is (no throw that moves the pair, no cargo) | `grabs::the_throws_land_on_their_frames_for_their_damage`, `throw_combos::*` |
| Pivot grab | `MoveId::PivotGrab` | falls back to dash grab | | `kits::a_pivot_grab_turns_around_out_of_a_dash_and_catches_behind` and the two after it |

## 4. Projectiles, reflectors, counters (`combat.rs`)

| Mechanic | Where | Parameters | Used by | Limits | Tests |
| --- | --- | --- | --- | --- | --- |
| Projectile pool | `GameState::projectiles`, `MAX_PROJECTILES` = 8 (shared by every fighter) | `ProjectileSpawn { frame, x, y, speed, life, hitbox, end_damage }`, one spawn spec per move | Claws blaster, Needles needles and line | **One hitbox per projectile; it is destroyed by its first hit** (no multi-hit, no piercing, no per-target hit memory). **Dies in a solid wall or on `life`** (no bounce, no landing, no resting on the ground). `life` is `u8` (at most 255 frames). No per-owner count limit (one bomb, one boomerang at a time). No projectile that spawns another (an explosion). The owner's later moves cannot act on a live projectile (no remote detonation). Damage falls off linearly with age | `reference_moves::the_blaster_*`, `a_shot_*`, `the_owner_is_never_hit_by_their_own_shot` |
| Projectile scripts | `Move::projectile_script`, `scripting::run_projectile` | registers `px py pvx pvy age life owner_x owner_y target_x target_y has_target`; `set_vel`, `kill`; 2 vars | Needles | can steer (gravity, homing, return to owner) but cannot read the owner's buttons, cannot change its own hitbox, cannot detect the ground except through position | `scripted_moves::a_projectile_script_can_home_in_on_the_nearest_enemy`, `a_projectile_script_can_remove_the_projectile` |
| Script-fired projectiles | fighter `spawn(fx, up, vx, vy)` | several per move from one spec | Needle Storm (up to six) | same spec for each | `scripted_moves::a_script_can_fire_a_projectile_with_its_own_velocity` |
| Projectile clanks | `clank_projectile` | `clank_range` | | | `hit_feel_2` |
| Reflector | `Move::reflector` (`Reflector { start, end, x, y, radius, damage_percent, speed_percent }`) | turns the shot round, new owner, more damage | Claws down special | **only the horizontal speed is reversed**; a field attached to the fighter's move (no barrier left in the world); no absorb; no reflect limit (the reference's x1.5 cap rules) | `reference_moves::the_reflector_turns_a_blaster_shot_around_for_one_and_a_half_times_the_damage` |
| Counter | `Move::counter` (`start`, `end`, `then`, `percent`, `min_damage`), `counter_strike` | catches melee and projectile hits, faces the attacker, answers with an `ext` move | Sword down special | one answer move; the answer always deals caught damage x percent (the reference's counters also have fixed-damage answers) | `kits::a_hit_inside_the_counter_window_is_caught_and_answered_with_interest` and the four after it |

## 5. Special-move scripting (`sim-script`, `sim-core/src/scripting.rs`, `sim-core/src/scripts/`)

| Ability | Whitelist | Used by | Limits | Tests |
| --- | --- | --- | --- | --- |
| Read: `frame facing x y vx vy percent stick_x stick_y grounded attack attack_tap special special_tap shield charge hit stored jump` | `FIGHTER_REGISTERS` | all scripted moves | **no `grab` button, no opponent position, no "was hit / is blocked / hit a shield" distinction (`hit` is any hit), no airtime/once-per-airtime flags, no ledge or stage queries** | `scripted_moves::a_script_reads_the_stick_relative_to_facing`, `a_script_sees_the_jump_button` |
| `set_vel`, `add_vel` | | Claws flash/fire, Needles Vanish/Fish, Dancing Blade | | `scripted_moves::a_script_can_move_the_fighter_like_a_data_motion` |
| `spawn` | | Needle Storm | | `scripted_moves::a_script_can_fire_a_projectile_with_its_own_velocity` |
| `intangible(n)` | | Claws reflector, Vanish | whole body | `kits::the_reflector_is_intangible_on_frames_5_to_8_only` |
| `end`, `turn`, `rehit`, `goto(slot)` | | Dancing Blade, Needle Storm, Bouncing Fish, rapid jab | `goto` keeps the script vars (one attack), clears `hit_mask` and `charge` | `scripted_moves::goto_switches_to_another_move`, `end_finishes_the_move_early` |
| `stall()` | | Shield Breaker, smash-like holds | at most `Ruleset::charge_frames` (60) per move; a `goto` loop restarts the count (Needle Storm does this) | `scripted_moves::stall_holds_the_move_but_never_forever` |
| `store(n)` / `stored` | `Fighter::stored: u8`, hashed | Needle Storm | **one value per fighter, shared by every move**, 0 to 255, set only by a script during a move, cleared by a KO. Not a gauge: nothing outside a move script can change it (no gain on damage dealt or taken, no timer) | `scripted_moves::a_stored_charge_outlasts_the_move_and_a_ko`, `needles::shield_or_jump_stores_*` |
| Script variables | `Fighter::vars` (4), `Projectile::vars` (2), hashed | | **4 per fighter, 2 per projectile**; cleared when a new attack starts | `scripted_moves::variables_persist_through_a_move_and_reset_for_the_next`, `projectile_variables_count_between_frames` |
| Determinism | bounded VM, integer only | | | `scripted_moves::scripted_content_is_deterministic_and_rolls_back`, `scripts_actually_ran_in_the_fuzz` |

Adding a script ability: an entry in `sim-script/src/api.rs`, its implementation in `sim-core/src/scripting.rs`, a test in
`sim-core/tests/scripted_moves.rs`, and the doc line in `docs/CONTENT.md` (a test checks the doc lists every entry).

## 6. Bodies (`sim-core/src/content.rs`, `FighterParams`)

| Mechanic | Parameters | Used by | Limits | Tests |
| --- | --- | --- | --- | --- |
| Ground movement from the parameter file | `walk_*`, `run_*`, `dash_*`, `ground_friction`, `dash_attack_friction`, `dash_frames`, `heavy_landing_lag` | all three | | `ground_attributes::every_fighter_has_its_reference_characters_ground_numbers` |
| Air movement and jumps | `air_*`, `gravity`, fall speeds, `full_hop` / `hop_velocity` from reference heights, `air_jumps`, `air_jump_velocity` | all three | **Every air jump is a plain velocity plus gravity**: no per-character jump curve (a slow-rising, then accelerating air jump), no air jump armor, no momentum-cancelling jump differences | `needles::the_rogues_jumps_have_the_references_air_times`, `reference_movement.rs` |
| Ledges, rolls, air dodges | per body | | | `edges_and_recovery.rs`, `movement.rs` |
| Weight | `weight` | | | `combat::a_heavier_target_is_launched_less` |
| Runtime stat changes | none | | **Bodies are immutable content: nothing can change a fighter's speed, jump or weight during a match** (needed for a powered-up mode) | |
| Wall jump, crawl, wall cling, glide, footstool | none | | not modelled | |

## 7. Protecting tests outside sim-core

* `sim-content/tests/bundle.rs::the_hand_coded_roster_round_trips_exactly`: `content/base.pfc` equals the Rust builders.
* `sim-content/tests/recipes.rs::every_recipe_is_valid_content`, `creating_a_character_never_changes_the_other_fighters`.
* `sim-core/src/state.rs` checksum tests: each `GameState` field is in the hash (one case per field; add one for every new field).
* `sim-core/src/step.rs::random_play_keeps_invariants`, `restore_then_replay_matches_straight_run`; `pftool fuzz-rollback`, `pftool net-fuzz`.
