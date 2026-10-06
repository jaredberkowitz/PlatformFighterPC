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
