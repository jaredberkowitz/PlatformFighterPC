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
| Swordfighter neutral air | two hits: 6-11 (3.5% / 5%, angles 75-90, KB 45 / 35, growth 50) and 14-28 (7% / 9.5%, angle 361, KB 50 / 60, growth 90 / 100), lag 7, autocancel from 47, FAF 50. **Deliberately longer than the reference** (which is 6-7 and 15-21) so the spin stays active (sim v24) |
| Swordfighter forward tilt | hits 8-11, 9% / 12%, angle 361, KB 30 / 55, growth 70 / 85, FAF 34 |
| Swordfighter up special | intangible frames 1-5 in the air, hits from frame 5 (11% early tip, then 7%), angle 74, helpless after |
| Swordfighter up tilt | one hit in three phases: frame 6 (6% tip, 5% arm and body, angle 100, KB 65, growth 100), 7-8 (10% tipper, 6% sour, 5%), 9-12 (same damage, angle 85, KB 52), FAF 34 |
| Swordfighter down tilt | frames 7-8, 7% close / 10% tip, angle 30, KB 40 / 50, growth 40, FAF 24 (the reference game's 35% trip chance is not implemented) |
| Swordfighter forward smash | frames 10-13, 13% / 18% tip, angle 361, KB 48 / 80, growth 75 / 80, FAF 52, charges |
| Swordfighter up smash | frames 13-17 overhead, 13% / 17% tip, angle 89, KB 45 / 40, growth 90 / 95, FAF 59, charges. The reference 3% launcher (pulls grounded targets in) is not implemented |
| Swordfighter down smash | front hit frames 6-7 (8% / 12% tip, KB 60 / 50, growth 88), back hit frames 21-23 (12% / 17% tip, KB 40 / 50, growth 88 / 92), angle 361, FAF 56, charges |
| Swordfighter up air | frames 5-9, 9.5% / 13% tip, angle 80 / 90, KB 40, growth 80 / 84, landing lag 8, autocancel 1-2 and from 38, FAF 46 |
| Brawler forward air | hits 7-9, 9%, angle 60, KB 45, growth 85, lag 10, autocancel from 29, FAF 41 |
| Brawler neutral air | 12% on 7-11 (KB 30, growth 75), then 8% on 12-34 (KB 0, growth 100), lag 9, autocancel 1-6 and from 38, FAF 43. **Deliberately longer than the reference** (7-9 and 10-26) (sim v24) |
| Brawler forward tilt | two hits: frame 8 (5%, angle 60, KB 10, growth 70) and 9-10 (6%, angle 361, KB 55, growth 106), FAF 35 |
| Brawler up tilt | an overhead kick, frames 7-11 (10% foot only on 7-8, 8% / 9% / 10% along the leg), angle 80, KB 30, growth 115-120, FAF 36 |
| Brawler down tilt | a low kick, frames 5-6, 6%, angle 361, KB 25, growth 100, FAF 28 |
| Brawler jab | three claw hits; 1 and 2: frame 4, 2%, angle 361, FAF 22; 3: frame 4, 4%, angle 55, growth 176, FAF 35. Pressing attack in the last 12 frames of a hit continues the combo |
| Brawler dash attack | flying kick, frames 11-14 (11%, angles 80 / 50 / 361) then 15-18 (8%, growth 60), FAF 38 |
| Brawler up air | frames 7-9, 12%, angle 80, KB 30, growth 85, lag 10, autocancel 1-3 and from 31, FAF 39 |
| Brawler back air | frames 13-15, 15% / 13% / 11% from the foot in, angle 361, KB 37, growth 96, lag 15, autocancel 1-7 and from 19, FAF 45, does not turn around |
| Brawler down air | frames 16-17, 15% (spike, angle 270) and 13%, KB 6, growth 90, lag 19, autocancel 1-4 and from 36, FAF 54 |
| Brawler forward smash | frames 20-23, 15%, angle 361, KB 30, growth 106, FAF 42, charges |
| Brawler up smash | two hits: 13-15 (6%, angles 110 / 125, KB 70-80, growth 15) and 20-23 (12%, angle 95, KB 85, growth 65), FAF 48, charges |
| Brawler down smash | front hit 14-15 (16% / 14%, angles 30-35), back hit 21-22 (14% / 12%), FAF 44, charges |
| Brawler side special (flash) | 19 frame wind-up, a dash of about 7 world units (3% on the way), ending in a 20% spike and a 15% hit around it; helpless in the air |
| Brawler up special (fire) | 18 frame wind-up, a rising flame kick: five hits (4%, 2.5% x 3, 6% launching), helpless after, grabs the ledge mid-move |
| Brawler down special (reflector) | reflecting field frames 9-21, turns projectiles around for 1.5x damage, 4% hit on contact, FAF 31 |
| Brawler blaster | bayonet on frames 15-19 (7%, angle 60, KB 80, growth 37); otherwise a shot on frame 16 that does 8% falling to 6% over its range (about two thirds of the stage), FAF 53 |

**Sources:** two community frame-data tables (ultimateframedata.com and kuroganehammer.com) cross-checked against each other,
plus SmashWiki for the blaster. Where they disagreed on a total frame count I used the kuroganehammer values.

**Estimates (not published in those sources):** hitbox positions and sizes; the up special's travel (about 44 reference
units straight up with a little forward drift, then its leftover speed) and landing lag; the blaster shot's speed (3 reference
units a frame), exact range (35 frames), knockback (a flinch) and muzzle position; the 8-to-6 percent damage falloff direction.
The up tilt's three phases are placed as an arc in front of, above and behind the fighter. Everything else in the movesets
(the swordfighter's jab, dash attack, down air, grabs and specials) was placeholder until the kit completion below.
For the brawler, Fire Wolf, Wolf Flash and the jab pages were not in the sources: the Fire Wolf travel (5.6 up, 2.6 forward) and its
drag hits' knockback, the Wolf Flash distance (7 world units), its ending hit's knockback and the total lengths, the reflector's
frames, size and reflected speed, and every hitbox position are estimates. (Angling both specials and the reflector's
intangibility are done, see "Kit completion".)

Two sources disagreed on some values, so: damage numbers come from ultimateframedata and kuroganehammer, which agree; the
down smash tipper's first-hit base knockback is 50 (kuroganehammer) rather than 57 (SmashWiki); the up air's first actionable
frame is 46 and landing lag 8 (the 24-frame landing animation is not the lag).

### Charging smash attacks

A smash attack holds on its charge frame while the attack button stays held, up to 60 frames (`Ruleset::charge_frames`),
then deals up to 40% more damage (`charge_bonus_percent`), which also raises knockback. Marth-style charge frames: forward
smash frame 2, up and down smash frame 4. The fighter trembles and a glow grows while charging. A smash is a flick of the
stick plus attack, or the strong key (`I`) plus a direction.

### Up specials grab the ledge mid-move

`Move::grabs_ledge`: while the up special is in progress (rising or not) the fighter grabs a ledge that is inside its grab
box, so a recovery that reaches the ledge is forgiving. The regrab cooldown still applies.

## Weapon visuals

The blade is posed from the move data: it runs from the hand to the move's sweet-spot hitbox, its length is that distance,
it winds up from the opposite side, reaches the hitbox by the first active frame, holds through the active frames and recovers.
At rest it is held up and ready. While standing it is clamped above the floor. Facing left mirrors the pose.

## Tests

`sim-core/tests/reference_moves.rs` (about 50 tests) pins each hit frame, damage, launch angle, knockback, control-return frame,
autocancel window and landing lag above, plus the blaster's spawn frame, speed, range, falloff, flinch, bayonet and blocking,
and the up special's intangibility, rise, hit and helplessness. Demos: `--demo=marth_fair`, `marth_bair`, `marth_nair`,
`marth_dolphin`, `marth_utilt`, `marth_dtilt`, `marth_fsmash` (charged), `marth_usmash`, `marth_dsmash`, `marth_uair`,
`marth_upb_ledge`, `wolf_fair`, `wolf_nair`, `wolf_ftilt`, `wolf_utilt`, `wolf_dtilt`, `wolf_blaster`, `low`.

## Engine pieces added for the brawler kit

- **Jab chains:** `Move::next` / `next_window` (the jab hits are `MoveId::Jab2` and `Jab3`, started only by the previous hit).
- **Multi-hit moves:** `Move::rehit = (start, every)` lets a move's hits land again every N frames (Fire Wolf).
- **Reflector:** `Move::reflector`; a projectile that touches an active reflector turns around, is owned by the reflecting
  fighter and deals `damage_percent` of its damage. Projectiles now carry `owner` (who is credited), `origin` (whose weapon holds
  the hit data) and `power`.
- **Ground motion:** a scripted dash along the ground (`Motion` with no vertical speed) no longer counts as landing and leaves the
  ground when it passes the edge.
- Tests: `sim-core/tests/reference_moves.rs`. Demos: `wolf_jab`, `wolf_dashattack`, `wolf_uair`, `wolf_bair`, `wolf_dair`,
  `wolf_fsmash`, `wolf_usmash`, `wolf_dsmash`, `wolf_flash`, `wolf_firewolf`.

## Shields, rolls and spot dodge (sim v15)

Numbers follow the reference game's published shield data (SmashWiki: Shield, Shield stun, Shield break); all of them are
`Ruleset` values (`shield_*`, `perfect_shield_*`) so they can be tuned.

| Rule | Value |
| --- | --- |
| Shield health | 50. Drains 0.15 a frame while the shield is up, refills 0.08 a frame while it is not |
| Damage to the shield | the hit's damage times the 1.2 damage multiplier |
| Shield stun | `floor(0.8 * damage * type + 2)` frames, type = smash 0.725, aerial 0.33, projectile 0.29, everything else 1; capped at 60. The shield stays up and the fighter cannot act, release or roll until it runs out |
| Pushback | `(stun + 1) * 0.09` reference units a frame, at most 1.3 |
| Perfect shield | a hit within 5 frames of the shield going up: no shield damage, 3 frames less stun, 40% of the pushback |
| Shield break | when health reaches 0 the fighter hops up and is stunned for `400 - percent` frames (at least 120); every button press takes 4 more frames off. It comes back with 75% shield health. Being hit during the stun also ends it |

**Rolls and spot dodge** (out of a shield): flick sideways to roll that way (about 31 frames, roughly 2.8 world units, intangible on
frames 4-19), hard down to spot dodge on solid ground (25 frames, intangible on 3-20); down on a pass-through platform still
drops through. These frame counts and the distance are estimates (`roll_*`, `spot_*` in `FighterParams`).

**Keyboard:** while the shield key is held every direction press is full strength, so a single tap rolls and a single S spot dodges.
Tests: `sim-core/tests/shields.rs`. Demos: `shield_block`, `shield_break`, `roll`. The bubble shrinks and turns red as health drops.
Not implemented: shield drop lag (11 frames in the reference), shield tilt, and the shield only blocking hits that touch the
bubble (any hit on a shielding fighter is blocked).

## Grabs, pummels and throws (sim v16)

A grab is a move (`MoveId::Grab`, `DashGrab`) whose hitbox has `kind = HIT_GRAB`: instead of damage it catches a fighter that is
standing on something (not airborne, not in hitstun, not hanging, not recently released). **Grabs ignore shields.** Inputs: the grab
button (`U` / `M`), attack or grab out of a shield (shield grab), and grab while dashing or running (dash grab). Grabbed fighters
are pinned 1.3 units in front of the holder, facing it (`grab.rs`).

| Rule | Value |
| --- | --- |
| Standing grab | hits on frame 7 (dash grab frame 8), first actionable frame 30 on a miss (dash grab 38). Reach is an estimate |
| Time held | `90 + 1.7 * percent` frames (at least 19). Each button press takes 14 frames off, each stick flick 8 |
| Breaking free | the holder is stuck in a 25 frame release; the released fighter cannot be grabbed for 60 frames |
| Pummel | attack while holding: 1.3% on frame 4, 22 frames, repeatable until the hold runs out |
| Throws | a stick direction while holding: forward 9% (release frame 11, FAF 33), back 11% (24, 48), up 7% (27, 46), down 8.5% (26, 41) |
| Interruptions | hitting the holder, hitting the held fighter or knocking either out frees both |

Throw angles and knockback are partly estimates (forward 45 degrees, base 55, growth 57; back 50 / 40 / 150; up 80 / 75 / 110; down
361 / 50 / 65); the reference sources disagree on the back throw's damage (8% or 11%) and do not give the throws' angles. The
swordfighter's grab and throws now have reference frames and damage (see "Kit completion"). Not implemented: pivot grab, cargo carries, grab release lag differences
between characters, and the throws' two-part damage (a hit while being held, then the throw).

The keyboard: throw with a direction held for a few frames (W for up, S for down, A / D forward or back), pummel with attack.
Tests: `sim-core/tests/grabs.rs` (21 tests) plus a mutual-grab invariant in the random play test; the fuzzer now sends grab and
strong-attack buttons too. Demos: `grab`, `shield_grab`.

## Dash attack and Wolf visuals (follow-up)

- A dash attack now also comes out in the few frames after letting go of the stick while still sliding at dash speed, instead of
  turning into a jab (a held direction still gives a tilt or smash). Dash attacks keep their speed on the gentler run deceleration,
  so Wolf's slides about 4 units instead of stopping in 12 frames.
- The brawler no longer draws a sword on its kicks, specials, grabs and throws. Wolf Flash and Fire Wolf show a flame around the body
  and Fire Wolf spins. These are stand-in visuals; real effects come with the art pass.

## Phase 3 completion: techs, knockdown, ledge attack, training mode (sim v18)

- **Tech:** a shield press up to 5 frames before landing in hitstun. In place: 4 frames of lag and 18 intangible frames. With the stick
  flicked sideways: a tech roll (the shield roll, intangible). No tech: the fighter lies in a **knockdown** (vulnerable).
- **Knockdown get-ups** after lying for at least 24 frames (it stands by itself at 90): **stick up or jump** stands (26 frames, the first
  15 intangible), **attack** is a get-up attack (hits both sides, intangible for 10 frames), a **sideways flick** is a get-up roll.
  `Ruleset`: `knockdown_lag`, `knockdown_max`, `getup_frames`, `getup_intangible`, `tech_invuln`.
- **Ledge attack** now has hitboxes (`MoveId::LedgeAttack`) and the get-up attack is `MoveId::GetUpAttack`; both have placeholder numbers
  that need the reference data.
- **Training mode** (F9 toggles the path): while a fighter is in hitlag or hitstun the view draws where it will fly, as a white line
  with no DI and a yellow line with the stick held as it is now. It is computed by running a copy of the sim (`SimRunner::predict_path`),
  so it includes gravity, DI, walls and landing. The overlay also shows a **combo counter** (hits and damage while the fighter stays in
  hitstun or a grab) and the **launch readout** (knockback, angle and whether it is a tumble). Ledge invulnerability counters, frame
  stepping, the stick display and the wavedash cone were already there.
- Tests: `sim-core/tests/knockdown.rs`. These cover the Phase 3 exit criteria's pieces (combos, DI, edge guarding with ledge options)
  in training mode; whether they are *fun* needs playtesting.

## Kit completion: the rest of both characters (sim v20)

Frame data and damage come from the community tables (ultimateframedata and SmashWiki, fetched 2026-10-08); the fetch tool read their
tables with unlabeled columns, so I only used numbers that were stated as text and left alone the Wolf numbers the earlier session
had already cross-checked. Hitbox positions, sizes, most throw and early-hit knockback, and landing lags are estimates.

**New engine features**
- **Ten script follow-up slots** `ext0` to `ext9` (`MoveId::Ext0..Ext9`, indices 28 to 37). A button never starts one; a script's
  `goto` or a counter does. Used for Dancing Blade's later hits and Counter's answer.
- **Counter stance** (`Move::counter`, `counter_strike`): while the stance's window is open a hit that lands (melee or projectile) is
  cancelled, both fighters freeze for its hitlag, the countering fighter turns toward the attacker and switches to the `then` move,
  whose hitboxes deal `max(min_damage, percent / 100 x caught damage)`. Tests: `sim-core/tests/kits.rs`.
- **Per-move charge bonus** (`Move::charge_bonus`): a move can scale more than the ruleset's smash-attack 40% at full charge.
- **Ledge attack intangibility**: the ledge attack is intangible for the move's `intangible` frames; `ledge_attack_frames` is now 54.
- Ext slots use air physics when airborne like specials; ground motion that points down no longer sinks into the floor.

**Sword character**
| Move | Numbers |
| --- | --- |
| Jab | hit 1 frames 5-6, 3% (5% tip), FAF 25; hit 2 frames 4-5, 4% (6%), FAF 28; ends there |
| Dash attack | frames 13-16, 9 / 10 / 13% (tip knockback 93 base, 58 growth), FAF 49 |
| Down air | frames 9-13, 12-14%; frame 11 only, the tip is a 15% meteor; FAF 59 |
| Grab / dash grab / pummel | frames 6-7 (FAF 34), 9-10 (FAF 42), 1.3% |
| Throws | forward 4% (release 18, FAF 34), back 4% (19, 44), up 5% (13, 44), down 4% (20, 46) |
| Ledge attack | 9%, angle 45, base 90 / growth 20, frames 24-26, intangible to 26, FAF 56 (the brawler's published numbers; the sword's are not in the sources) |
| Shield Breaker | neutral special: hold special from frame 19 (at most 60 frames), thrust 8 frames after release, 8% (9% tip) to about 24% charged, 39 frames from release |
| Dancing Blade | side special: hit 1 on frame 9 (FAF 39, ends at 29 in the air); each press of special before the end continues; stick up gives the rising hits; hits 2.5-3%, 3-4%; finishers 4-6% (straight), 5-7% (rising), 2% x4 then 4-5% (low) |
| Counter | down special: window frames 6-27, answer 1.2x (at least 8%), counter-attack on frame 4, FAF 64 unused |

**Claws character**
- Wolf Flash and Fire Wolf are aimed with the stick (scripts): Flash up or down a little at the start of the dash; Fire Wolf in any
  direction at the end of its wind-up (neutral aims up and a little forward; backward turns him around first).
- The reflector is intangible on frames 5-8 (a script).
- Ledge attack as above. Everything else was already built from the earlier cross-checked data.

**On the keyboard (K is special):** Shield Breaker is K alone (hold it, release to thrust); Dancing Blade is D + K, then tap K again for
each further hit (hold W before a tap for the rising hits, S for the low ones); Counter is S + K and then the other fighter has to hit
you; Fire Wolf is W + K and the direction you hold when the wind-up ends is the direction he flies; Wolf Flash is D + K, with W or S
held at the start of the dash to angle it.

**Follow-up (sim v21): steps, shield damage, pivot grabs**
- **Dancing Blade steps forward** on the ground: about a unit on hit 1 (frames 3-8) and a short step into each later hit, then it stops. In
  the air the script leaves physics alone so the fall is not frozen. The distances are estimates.
- **Shield Breaker does double damage to shields** (`Hitbox::shield_damage`, a percent, default 100; Shield Breaker's hits use 200). An
  uncharged one takes about 19-22 off a 50 point shield and a full charge breaks it. The multiplier is an estimate: the sources used give none.
- **Pivot grab** (`MoveId::PivotGrab`): grab while turning around out of a dash or run, either by flicking the other way together with grab
  or by pressing grab during the skid while still sliding. The fighter ends up facing the new way and keeps sliding. Sword: hits on frames
  10-11, FAF 37 (reference). Claws: frames 11-12, FAF 33 (an estimate, four frames after the standing grab as the sword's is). A weapon
  without one falls back to its dash grab, and turning from a standstill is an ordinary grab. Keyboard: dash with D, then A and the grab key.

**Not done:** real animations (they wait for real models), and Shield Breaker's armour-free but otherwise unmodelled shield properties.

## Hitbox scale (sim v22)

`FighterParams::hitbox_scale` multiplies the size and position (from the feet, forward-relative) of every hitbox the fighter's moves
use, the position and size of projectiles the fighter fires, and the reflector's field. `combat::active_hitboxes` takes the scale and
returns already-scaled hitboxes. Damage, knockback, timing and scripted motion are unchanged, and a script's `spawn(x, y, ...)` offsets
are used as written. The created-fighter recipes set it to the body size (0.68 to 1.32); every built-in fighter uses 1.0, so nothing
changes for them.
