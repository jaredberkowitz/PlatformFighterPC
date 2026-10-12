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
moves the victim and so changes the pull, which is how a looping hit is escaped [D].

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

## The rest of the kit (timing [R] from UFD / SmashWiki; hitboxes from SmashWiki where parsed)

| Move | Frames (active / FAF) | Damage | Notes |
| --- | --- | --- | --- |
| Jab 1, 2 | 2 (FAF 18), 3 (FAF 18) | 2.0, 1.6 | chains on 4 / 5; rapid jab 0.3 a hit, finisher 2.0 (5-6, FAF 36) |
| Dash attack | clean 5-6, late 7-8 (FAF 35) | 7.0 / 5.0 | 60 BK70 KS90, H 1.3; late 361 BK20 KS100 |
| Forward smash | 12, 20-21 (FAF 45); charge on 3 | 5.0 / 8.0 | hit 1 FKV 47/47/30 (35/45/80), hit 2 361 BK60 KS109 |
| Up smash | 11, 14-15 (FAF 53); charge on 6; arms intangible 8-15 | 15.0 / 11.0 | 93 BK52 KS88 H1.3; 50 BK38 KS100 |
| Down smash | 8-9, 15-16 (FAF 49); charge on 1 | 4.0 / 6.0 | hit 1 FKV 35; hit 2 43 BK35 KS165 both sides |
| Grab / dash / pivot | 6-7 / 9-10 / 11-12 (FAF 37 / 45 / 40) | | |
| Throws | release f 20/22 (F), 15/17 (B), 19/21 (U), 26-29/30 (D) | 5+2, 5+2, 3+3, 3+3 | throw angles and knockback still to parse |
| Needle Storm | 4-16 per needle (+7 to charge); full charge 86 frames | 1.5 / 0.8 each | projectiles: later phase |
| Burst Grenade | 13 throw, explodes 67-88 | 1.0 windboxes / 12.6 | later phase |
| Vanish | 36-38, 55-57 (FAF 96); intangible 14-60 ground, 19-55 air | 12.0 / 5.0 | later phase |
| Bouncing Fish | 18 / 26 | 11.0 | later phase |

These are filled in after the priority moves; until then the slots use the reference timing with simple hitboxes, each labelled in
`moves.rs`, and the specials are plain stand-ins (see `PROJECT_AUDIT.md`, plan step 4).
