# Reference data for the Needles moveset (the reference game's Sheik)

Mechanical reference for the third moveset (in-game: the **rogue** body with the **Needles** weapon; the reference character's name is
not used in the game). Collected 2026-10-11.

## Sources and version

- **SmashWiki** per-move pages, `Sheik_(SSBU)/<move>` (hitbox tables: id, damage, angle, angle type, BK, KS, FKV, radius, bone, offset,
  hitlag (H) and SDI multipliers, trip, grounded / aerial flags, hitstun modifiers; timing bars), and `Sheik_(SSBU)` (attributes). The
  wiki tracks the final game version (13.0.x); its update histories note the changes that apply (for example 9.0.0: up tilt's first
  hit +2 hitstun).
- **ultimateframedata.com/sheik** (startup, active frames, total frames, landing lag, autocancel windows, shield stun; jump durations).
- **SmashWiki, Autolink angle** (the 367 angle's behaviour).

Where the two agree they are recorded once. Conflicts: none in the frame data (UFD "total frames" is the wiki's "interruptible" minus
one: the first actionable frame (FAF) is the wiki's number). Smash 4 data was not used.

**Labels**: **[R]** reference value as published; **[D]** derived from reference values (unit conversion, a formula); **[E]** estimate
(not published, or not representable): hitbox *positions* are estimates throughout, because the reference attaches hitboxes to bones
of its own skeleton (offsets in its units along bones we do not have); our positions are placed on our body where those bones are in
the move's pose. Radii are converted exactly.

## Conventions

- Frames are the reference's (frame 1 is the move's first); the sim stores them minus one (`moves.rs` `ref_move` / `row`).
- Units: 1 world unit = 8 reference units (`FighterParams::su`). Radius (world tenths) = reference radius x 1.25 **[D]**.
- Angles: degrees, 0 forward, 90 up; 361 Sakurai angle; 367 autolink (below). "Backward" angle type: sends away behind the fighter (our
  hitboxes behind the body send that way).
- BK base knockback, KS knockback growth, FKV set (fixed) knockback, H hitlag multiplier, SDI multiplier.

## Attributes [R] (SmashWiki attributes table; UFD stats)

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 78 | 1.47 | 2.178 | 2.42 | 0.106 | 1.155 | 0.01 + 0.08 | 0.015 | 0.15 | 1.75 / 2.8 | 39 / 18.75 / 40 | 3 | 1 |

Jump durations (UFD) [R]: short hop 32 frames, full hop 47, short hop fast fall 23, full hop fast fall 33 (tests check ours against
these). Air dodges (UFD) [R]: neutral 44 frames (intangible 2-26); directional down 63, side 74, up 93 (intangible 2-19).
Rolls and spot dodge (UFD) [R]: forward roll 26 (intangible 4-12), back roll 32 (4-14), spot dodge 18/23 (3-14); **not applied yet**
(the rogue keeps the shared roll and spot dodge timings) [E]. Ledge jump height: not published [E: the duelist's].

## Autolink angle 367 [R], and how it is implemented [D]

SmashWiki: the victim's launch speed is set to the attacker's momentum plus 20% of the distance from the victim to the hitbox,
computed when hitlag ends; the speed is capped at 3 (the speed of 100 knockback) and the vertical speed at no lower than -1; hitstun
still comes from the numeric knockback; against a grounded victim the angle is 80 with the normal launch. Ours: exactly that, in
world units (3 / 8 and -1 / 8 a frame), with the launch then decaying like any launch. The 5-frame delay before the cap applies is
not modelled (the cap applies from the start) [E]. DI does not bend an autolink launch (it has no angle to bend); SDI during hitlag
moves the victim and so changes the pull, which is how a looping hit is escaped [D]. Our SDI step (a quarter of a world unit, times
the hitbox's SDI multiplier) is the game-wide value and was not tuned against the reference's; the tests show an up-air SDI nudge moving
the victim, not a measured escape rate [E]. The natural escape that is tested is the attacker fast-falling out from under the pull.

## Priority moves

### Forward tilt [R] (timing 5-8, FAF 25)

| id | Dmg | Angle | BK | KS | Radius (ref → tenths) | Bone (offset) | H | SDI |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 3.0 | 84 | 20 | 180 | 3.8 → 5 | legr (0.7) | 0.5 | 1.0 |
| 1 | 3.0 | 86 | 20 | 180 | 3.0 → 4 | kneer (0.4) | 0.5 | 1.0 |
| 2 | 3.0 | 78 | 20 | 188 | 4.0 → 5 | kneer (4.3, -0.2) | 0.5 | 1.0 |

A rising front kick; the low angles' vertical launch and low hitlag make it a combo starter. Positions [E]: thigh, knee, foot, the foot
at chest height in front.

### Up tilt [R] (hit 1 frames 5-8, hit 2 frames 16-23, FAF 33)

| Hit | id | Dmg | Angle | BK | KS | FKV | Radius | Bone | H | SDI | Hitstun |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 0 | 3.0 | 0 | 0 | 80 | 30 | 3.5 → 4 | legr (4.0) | 1.0 | 0.5 | +2 |
| 1 | 1 | 3.0 | 330 | 0 | 80 | 28 | 5.5 → 7 | kneer (2.5) | 1.0 | 0.5 | +2 |
| 1 | 2 | 3.0 | 75 | 0 | 80 | 40 | 4.5 → 6 | legr (0.4) | 1.0 | 0.5 | +2 |
| 2 | 0 | 4.0 | 60 | 25 | 147 | 0 | 4.0 → 5 | legr (0.4) | 1.0 | 1.0 | |
| 2 | 1 | 4.0 | 60 | 25 | 147 | 0 | 4.0 → 5 | legr (5.0) | 1.0 | 1.0 | |
| 2 | 2 | 4.0 | 60 | 25 | 147 | 0 | 5.0 → 6 | kneer (4.0) | 1.0 | 1.0 | |

Hit 1 is set knockback (FKV) pulling the victim in (angles 0 and 330 toward the leg) so hit 2 (the leg brought up overhead) connects;
two hit groups. Positions [E].

### Down tilt [R] (5-6, FAF 27)

| id | Dmg | Angle | BK | KS | Radius | Bone | Trip |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 4.5 | 70 | 37 | 100 | 3.2 → 4 | legr (0) | 40% |
| 1 | 4.5 | 70 | 37 | 100 | 3.2 → 4 | legr (3.0, -0.6) | 40% |
| 2 | 4.5 | 90 | 37 | 100 | 4.2 → 5 | kneer (4.0, -0.2) | 40% |

The trip can never happen (the angles are too high, SmashWiki); tripping is not implemented. Positions [E]: a low kick along the floor,
the toe (id 2, the 90-degree "tipper") furthest out.

### Neutral air [R] (clean 3-6, late 7-30; autocancel 1-2 and 31+; landing lag 6; FAF 50)

| Phase | id | Dmg | Angle | BK | KS | Radius | Bone |
| --- | --- | --- | --- | --- | --- | --- | --- |
| clean | 0 | 6.0 | 361 | 30 | 100 | 3.7 → 5 | kneel (2.1) |
| clean | 1 | 6.0 | 361 | 30 | 100 | 3.0 → 4 | kneel (-1.0) |
| clean | 2 | 5.0 | 361 | 30 | 100 | 3.0 → 4 | hip (-1.0) |
| late | 0 | 4.0 | 361 | 30 | 100 | 3.4 → 4 | kneel (1.9) |
| late | 1 | 4.0 | 361 | 30 | 100 | 2.7 → 3 | kneel (-1.0) |
| late | 2 | 3.0 | 361 | 30 | 100 | 1.5 → 2 | hip (-1.0) |

One hit per target (clean replaces late in one group). A held flying kick. Positions [E].

### Forward air [R] (early 5-6, late 7; autocancel 1-4 and 11+; landing lag 5; FAF 35)

| Phase | id | Dmg | Angle | BK | KS | Radius | Bone (offset) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| early | 0 | 3.8 | 50 | 44 | 127 | 5.0 → 6 | shoulderl (0) |
| early | 1 | 4.5 | 50 | 43 | 127 | 5.0 → 6 | arml (2.5) |
| late | 0 | 3.8 | 50 | 44 | 127 | 5.0 → 6 | shoulderl (0, -0.5, 1.0) |
| late | 1 | 4.5 | 50 | 43 | 127 | 5.0 → 6 | arml (2.5, -1.0, 1.5) |

A downward chop; the far (arm) hitbox deals more. Its ledge-grab-from-both-sides property (frame 59) is beyond the move's end and not
modelled. Positions [E].

### Back air [R] (clean 4-6, late 7-14; autocancel 1-3 and 31+; landing lag 7; FAF 38)

| Phase | id | Dmg | Angle (type) | BK | KS | Radius | Bone |
| --- | --- | --- | --- | --- | --- | --- | --- |
| clean | 0 | 7.5 | 361 (backward) | 32 | 98 | 3.8 → 5 | hip |
| clean | 1 | 7.5 | 361 (backward) | 32 | 98 | 2.0 → 3 | kneer (-0.5) |
| clean | 2 | 9.5 | 361 (backward) | 30 | 98 | 4.2 → 5 | kneer (3.8, -0.2) |
| late | 0 | 6.0 | 361 (backward) | 10 | 101 | 4.0 → 5 | hip |
| late | 1 | 6.0 | 361 (backward) | 10 | 101 | 3.3 → 4 | kneer (-0.5) |
| late | 2 | 6.0 | 361 (backward) | 10 | 101 | 2.8 → 4 | kneer (3.0, -0.2) |

A back kick; the foot (id 2) is the sweet spot. Facing either way: positions are relative to facing, and every box sits behind the
body, so the launch goes away behind. Positions [E].

### Up air [R] (multihits 4-15, rehit every 4; final hit 23-24; autocancel 1-3 and 44+; landing lag 13; FAF 44)

| Phase | id | Dmg | Angle | BK | KS | Radius | Bone | H | SDI | Targets |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| loop | 0 | 1.0 | 80 | 20 | 80 | 4.5 → 6 | footr | 0.7 | 1.2 | grounded only |
| loop | 1 | 1.0 | 367 | 30 | 100 | 4.5 → 6 | footr | 0.7 | 1.2 | aerial only |
| final | 0 | 4.0 | 80 | 50 | 138 | 7.0 → 9 | kneel (5.0 to 0) | 2.0 | 1.0 | both |

The loop (one group, re-armed every 4 frames: hits on frames 4, 8 and 12) uses the autolink angle against airborne targets, so a
falling attacker (a fast fall) drags the target down with it; the final hit is its own group and is never re-armed (no double final
hit). Total 7% when every hit lands. Positions [E]: the loop at the feet kicking overhead, the final hit on the knee above the head.

### Down air [R] (clean 15-18, late 19-33; autocancel 1-2 and 53+; landing lag 22; FAF 55; landing hit on landing frames 1-2)

| Phase | id | Dmg | Angle | BK | KS | Radius | Bone (offset) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| clean | 0 | 10.0 | 280 (meteor) | 30 | 90 | 5.9 → 7 | top |
| late | 0 | 10.0 | 56 | 55 | 78 | 5.8 → 7 | top |
| landing | 0 | 2.0 | 48 (forward) | 77 | 50 | 6.0 → 8 | top (0, 6.0) |

Motion [R]: "sets vertical momentum to 2 on frames 1-14, then horizontal 1.2 and vertical -6 from frame 15 onward; manual horizontal
movement disabled 1-47". Ours [D/E]: frames 1-14 rise from 2 a frame slowing under gravity (0.15) with no drift (the reference's wording
is ambiguous between a held speed and an initial one; a held 2 would rise 28 units, far more than the move shows); from 15 on a dive at
1.2 forward and 6 down a frame until landing or the move's end. The landing hit is its own short move (the landing lag, 22 frames, FAF
23) started only by a real landing outside the autocancel windows; its hitbox is out on its frames 1-2 and cannot repeat.

## The rest of the kit (built in phase 2, sim 50)

Sources as above, plus SmashWiki's `Sheik_(SSBU)/Neutral_attack/Hit_1`, `/Hit_2` and `/Infinite` (the jab's per-hit tables),
`Sheik_(SSBU)/Pummel`, and, for the dash attack only, the earlier game's `Sheik_(SSBM)/Dash_attack` (asked for by name).

| Move | Built as | Labels |
| --- | --- | --- |
| Jab 1 | frame 2, 2%, FAF 18, H 1.6; close Sakurai angle BK 35 / 30 KS 30, tip 180 degrees BK 20 KS 15 (the tip's fighter-only box; its twin for objects is not modelled) | [R]; positions [E] |
| Jab 1 to 2 | a press (or attack held through the hit) carries on from frame 4 | [R] (a script) |
| Jab 2 | frame 3, 1.6%, FAF 18, H 1.2; Sakurai angle BK 35 / 35 / 25, KS 25 / 25 / 20 | [R] |
| Jab 2 to rapid jab | from frame 5, after two presses since jab 1 (the reference: three; presses during hitlag reach no script) or attack held as it hits | [D] |
| Rapid jab | stabs on loop frames 5-13 every 2 frames, 0.3%, Sakurai angle BK 12 KS 15, +2 hitstun, H 0.5, SDI 0.4; loops while attack is held or pressed within 10 frames | values [R]; the loop is 14 frames (the reference's 12) [D] |
| Finisher | frames 5-6, 2%, 40 degrees, BK 60, KS 132, H 2.0, FAF 36 | [R] |
| Dash attack (the earlier game's) | clean frame 6: 10%, 80 degrees, BK 34, KS 100; late 7-12: 7%, Sakurai angle, BK 15, KS 100; FAF 36; radii 3.9 / 3.1 of its units | [R] (earlier game); positions [E]; its knockback runs through our (the reference's) formula [D] |
| Forward smash | hit 1 frame 12: 5%, FKV 47 / 47 / 30 at 35 / 45 / 80 degrees, KS 100, SDI 0.5; hit 2 frames 20-21: 8%, Sakurai angle, BK 60, KS 109; charge hold frame 3; FAF 45 | [R] |
| Up smash | hit 1 frame 11: 15%, 93 degrees, BK 52, KS 88, H 1.3, SDI 1.5; hit 2 frames 14-15: 11%, 50 degrees, BK 38, KS 100, both sides; charge hold frame 6; FAF 53 | [R]; arm intangibility 8-15 not modelled [E] |
| Down smash | hit 1 frames 8-9: 4%, FKV 35, KS 100; 140 degrees in front (200 against airborne), 165 behind; hit 2 frames 15-16: 6%, 43 degrees, BK 35, KS 165, both sides; FAF 49 | [R]; charge hold on frame 2, not 1 [D] |
| Grabs | standing 6-7 (FAF 37), dash 9-10 (FAF 45), pivot 11-12 (FAF 40) | [R]; reach [E] |
| Pummel | 1%, FKV 30, H 1.7, on frame 2 (a hit cannot start on frame 1), FAF 16 | [R] / [D] |
| Forward throw | 5% hit frame 20, release 22: 2%, 72 degrees, BK 80, KS 79; FAF 40 | [R] |
| Back throw | 5% hit frame 15, release 17: 2%, BK 90, KS 50, 120 degrees from the front (60 behind); FAF 36 | [R] / [D] |
| Up throw | 3% hit frame 19, release 21: 3%, 85 degrees, BK 110, KS 62; FAF 50 | [R] |
| Down throw | 3% hit frame 26, release 30: 3%, 77 degrees, BK 95, KS 60; FAF 54 | [R] |
| Rolls, spot dodge | forward roll 26 frames, back roll 32, intangible 4-12 (the back roll's 4-14 is not separate); spot dodge 23, intangible 3-14 | [R] / [D]; roll distance [E] |
| Ledge attack, get-up attack | 9% / 7% (the reference's damage), the claws' frames | damage [R], timing [E] |

Two general rules changed with this phase, because the jab and the up air's drag-down depend on them (sim 50, `docs/COMBAT.md`): the
Sakurai angle is the reference's (0 degrees on the ground up to 60 knockback, 38 from 88 and in the air; it was the older game's 0 / 44 at
32), grounded slides are slowed by traction, and a non-tumble hitstun lands on its feet (no knockdown, no tech).

### Still to build

| Move | Frames (active / FAF) | Damage |
| --- | --- | --- |
| Needle Storm | 4-16 per needle (+7 to charge); full charge 86 frames | 1.5 / 0.8 each |
| Burst Grenade | 13 throw, explodes 67-88 | 1.0 windboxes / 12.6 |
| Vanish | 36-38, 55-57 (FAF 96); intangible 14-60 ground, 19-55 air | 12.0 / 5.0 |
| Bouncing Fish | 18 / 26 | 11.0 |

Until then the specials are the claws kit's stand-ins.
