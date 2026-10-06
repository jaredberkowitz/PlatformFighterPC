# Combat (Phase 3, first slice)

## What exists

- **Weapons choose movesets** (`sim-core/src/moves.rs`): a fighter has a `weapon` index; each weapon holds 13 moves.
  The **longsword** (duelist) has a tip (priority 0, hits harder) and a body (priority 1). The **claws** (brawler) are the
  same moves pulled in close, quicker and a little heavier, with no tip/body split. Frame data and numbers are placeholders.
- **Hitboxes and hurtboxes are circles.** Hitboxes come from move data (active on frames `start..=end`, offset from the feet,
  mirrored by facing). Hurtboxes are three circles stacked up the body, derived from the ECB. One hit per move per target;
  when several hitboxes overlap a target, the lowest priority number wins (that is what makes spacing matter).
- **Moves:** jab, forward/up/down tilt, dash attack, forward/up/down smash, neutral/forward/back/up/down aerial.
  *Smash vs tilt:* a stick flick within 4 frames of the attack press is a smash; a stick that was already held is a tilt.
  Attack out of a dash or run is the dash attack. Aerials follow the stick relative to facing.
- **Aerial landing lag** with autocancel windows (landing early or late in the move costs only the normal landing lag).
- **Damage and launch** follow the reference formulas: `KB = ((((p/10 + p*d/20) * 200/(w+100) * 1.4) + 18) * g/100) + b`,
  launch speed `KB * 0.03` reference units/frame (8 per world unit), hitstun `KB * 0.4 * ruleset multiplier` frames
  (multiplier 1.05), hitlag `floor(d/3 + 4)` frames for both fighters. Angle 361 is horizontal for weak grounded hits and
  44 degrees otherwise.
- **Hitlag freezes both fighters.** During it the victim can **DI** (the part of the stick perpendicular to the launch bends
  it up to 18 degrees, speed unchanged, applied on the frame hitlag ends) and **SDI** (a stick flick nudges 0.75 units).
- **Hitstun** with gravity and decaying launch speed. Landing in hitstun is a **tech** with a shield press just before
  (4 frames of lag), otherwise a placeholder knockdown (24 frames).
- **Shield** blocks: no damage, no launch, a little pushback. Intangible: respawn invulnerability, ledge invulnerability and
  the middle of an air dodge.
- **KO and respawn:** leaving the blast zone costs a stock, resets damage and respawns at the spawn point with 120 frames of
  invulnerability.
- **Ruleset** (`content::Ruleset`) holds the global combat numbers and is hashed with content, so both peers must agree.

## Not done yet

Shield damage and shield stun, grabs and throws, special moves, projectiles, wall bounces and real knockdown, get-up and
tech options, items, hitbox clanking and trades beyond both hitting, hit sparks and sounds by strength, and per-move
hitlag/hitstun overrides. See `docs/MOVESETS.md` for the plan.

## Training view

F3 shows hitboxes (red = tip or priority 0, orange = body) and hurtboxes (green). F6 adds 25% to player 2, F7 resets damage,
F8 stands the fighters face to face. The overlay lists damage, stocks, hitlag, hitstun and the current move.
Demos: `Godot --path godot -- --demo=combat --shots=<folder>` and `--demo=smash`.

## Tests

`sim-core/tests/combat.rs` (28 tests) pins hit timing, hitlag, tip vs body, reach, the knockback formula, launch direction,
percent and weight scaling, hitstun length, DI, SDI, shielding, invulnerability, trades, KO, tech, aerial landing lag and
move selection. The rollback fuzzer now includes random attacks and hits.

## Reference-based moves

These moves follow published Smash Ultimate frame data for two archetypes. Names and visuals stay original; only the
numbers (a game's mechanics) are matched. Frame numbering matches the reference: the tick the button is pressed is
frame 1, so a hit "on frame 6" lands on tick 6 and a move with first actionable frame 38 returns control on tick 38
(`move frame = frame - 1` internally). Damage is shown with the 1.2 multiplier the reference applies in one-on-one rules
(`Ruleset::damage_mult`).

| Move | Source data (reference numbering) |
| --- | --- |
| Swordfighter forward air | hits frame 6-8, 8% close / 11.5% tip, angle 361, base KB 40, growth 80, landing lag 10, autocancel from 36, FAF 38 |
| Swordfighter back air | hits 7-11, 9% / 12.5%, angle 361, KB 40, growth 85 / 94, lag 10, autocancel 1-2 and from 32, FAF 40, turns around, sends victims backward |
| Swordfighter neutral air | two hits: 6-7 (3.5% / 5%, angles 75-90, KB 45 / 35, growth 50) and 15-21 (7% / 9.5%, angle 361, KB 50 / 60, growth 90 / 100), lag 7, autocancel from 47, FAF 50 |
| Swordfighter forward tilt | hits 8-11, 9% / 12%, angle 361, KB 30 / 55, growth 70 / 85, FAF 34 |
| Swordfighter up special | intangible frames 1-5 in the air, hits from frame 5 (11% early tip, then 7%), angle 74, helpless after |
| Brawler forward air | hits 7-9, 9%, angle 60, KB 45, growth 85, lag 10, autocancel from 29, FAF 41 |
| Brawler neutral air | 12% on 7-9 (KB 30, growth 75), then 8% on 10-26 (KB 0, growth 100), lag 9, autocancel 1-6 and from 38, FAF 43 |
| Brawler forward tilt | two hits: frame 8 (5%, angle 60, KB 10, growth 70) and 9-10 (6%, angle 361, KB 55, growth 106), FAF 35 |
| Brawler blaster | bayonet on frames 15-19 (7%, angle 60, KB 80, growth 37); otherwise a shot on frame 16 that does 8% falling to 6% over its range (about two thirds of the stage), FAF 53 |

**Sources:** two community frame-data tables (ultimateframedata.com and kuroganehammer.com) cross-checked against each other,
plus SmashWiki for the blaster. Where they disagreed on a total frame count I used the kuroganehammer values.

**Estimates (not published in those sources):** hitbox positions and sizes; the up special's travel (about 44 reference
units straight up with a little forward drift, then its leftover speed) and landing lag; the blaster shot's speed (3 reference
units a frame), exact range (35 frames), knockback (a flinch) and muzzle position; the 8-to-6 percent damage falloff direction.
Everything else in the movesets (jab, other tilts, smashes, up and down air, specials not listed) is still placeholder.

## Weapon visuals

The blade is posed from the move data: it runs from the hand to the move's sweet-spot hitbox, its length is that distance,
it winds up from the opposite side, reaches the hitbox by the first active frame, holds through the active frames and recovers.
At rest it is held up and ready. While standing it is clamped above the floor. Facing left mirrors the pose.

## Tests

`sim-core/tests/reference_moves.rs` (25 tests) pins each hit frame, damage, launch angle, knockback, control-return frame,
autocancel window and landing lag above, plus the blaster's spawn frame, speed, range, falloff, flinch, bayonet and blocking,
and the up special's intangibility, rise, hit and helplessness. Demos: `--demo=marth_fair`, `marth_bair`, `marth_nair`,
`marth_dolphin`, `wolf_fair`, `wolf_nair`, `wolf_ftilt`, `wolf_blaster`, `low`.
