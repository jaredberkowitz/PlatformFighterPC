# Reference data for the Embers moveset (the reference game's Mario)

Mechanical reference for the fourth moveset. In the game it is the **tumbler** body with the **Embers** weapon (class 3, built-in
fighter "Tumbler"); the reference character's name is not used in the game. Collected 2026-10-13 for version **13.0.1**.

## Sources and version

- **SmashWiki** per-move pages `Mario_(SSBU)/<move>` and `Mario_(SSBU)/Neutral_attack/Hit_1..3`, `/Floor_attack_(front|back|trip)`,
  `/Edge_attack`, read as **raw wikitext** (`index.php?title=...&action=raw`). The hitbox rows are templates with named fields
  (`damage`, `angle`, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff` hitlag multiplier, `sdi`, `sd` shield damage, `setweight`,
  `trip`), so the 361-icon column shift that garbles rendered tables cannot happen. Timing tables (hitbox frames, interruptible,
  autocancel, landing) from the same pages. `Mario_(SSBU)` for the attributes table and the moveset summary.
  https://www.ssbwiki.com/Mario_(SSBU)
- **ultimateframedata.com/mario** (startup, total frames, landing lag, autocancel, active frames; jump air times, dodges, stats).
  UFD's "total frames" is FAF - 1. https://ultimateframedata.com/mario
- **dragdown.wiki `SSBU/Mario/Data`** (the parameter file: walk and run acceleration, traction, jump heights, landing frames, cliff
  jump, wall jump numbers). https://dragdown.wiki/wiki/SSBU/Mario/Data
- **SmashWiki, Dash** (Ultimate table: initial dash 1.936, run 1.76, 10 dash frames, run acceleration 0.044 + 0.10978).
  https://www.ssbwiki.com/Dash
- **kuroganehammer.com** could not be read (the page came back empty), so it was not used as a cross-check this run.

Update history applying to 13.0.1 (SmashWiki): 2.0.0 the dash attack's angles 60 / 58 to 50 / 48; 3.0.0 Fireball shield damage 0 to
-2.5 / -2.0 (additive); 4.0.0 and 8.0.0 touch only the final smash. No data from the 3DS/Wii U game or earlier was used.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift the engine needs); **[E]**
estimate. Hitbox **positions** on a bone other than `top` are [E] (the reference attaches them to its own skeleton; ours are placed
on our body where that bone is in the move's pose). Offsets on `top` (the character's root at the feet, `zpos` forward, `ypos` up)
are converted directly [D] (our body is about the reference's height). A reference "capsule" (`zpos=4.0 to 8.7`) is two or three
circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first); the sim stores them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius and position in world tenths = reference value x 1.25, rounded to a tenth [D].
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; 361 is the reference's 361 angle.
- Shield damage: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd` becomes
  `100 x (damage + sd) / damage` [D] (dash attack 113 / 117, ledge attack 111, get-up attack 214).

## Attributes [R]

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 98 | 1.155 | 1.936 | 1.76 | 0.102 | 1.208 | 0.01 + 0.07 | 0.015 | 0.087 | 1.5 / 2.4 | 36.33 / 17.54 / 36.33 | 3 | 1 |

Parameter file (dragdown) [R]: walk acceleration 0.105 + 0.105 (by the stick), run acceleration 0.044 + 0.10978, initial dash 10
frames (SmashWiki Dash table), `landing_heavy_frame` 4 (so 3 frames of lag here, [D]), light landing 3 frames (universal),
`cliff_jump_y` 38.165 and `cliff_jump_x_speed` 0.6 (the ledge jump), `wall_jump_type` true (passive wall jump x 1.3, y 2.3, enable at
x speed 0.2: **not built**, E15), aerial landing lags 6 / 17 / 6 / 6 / 15 (n / f / b / up / down, agreeing with UFD).

Jump air times (UFD) [R]: short hop 40, full hop 56, short hop fast fall 28, full hop fast fall 39 (the tests check ours within one
frame). Dodges (UFD) [R]: spot dodge 25 (intangible 3-17; UFD's "20/25" second value is the length), forward roll 29 (4-15), back
roll 34 (5-16: not separate here, it uses 4-15 [D]), neutral air dodge 52 (3-29), directional 71 down / 87 sideways / 116 up
(3-21; UFD also lists 77 diagonally down and 102 diagonally up, which our angle interpolation approximates [D]), air dodge landing lag
10 / 11-19. Our shared values already equal the reference's for everything but the directional down (69 to 71) and side (85 to 87).

## Normals

All frame numbers [R] (SmashWiki timing tables; UFD agrees on every startup and FAF). Damage, angle, BK, KS, FKV, hitlag (H), SDI
[R]. Radius column: reference → world tenths [D].

### Jab [R]

| Hit | Frames | FAF | Continuable | id | Dmg | Angle | BK | KS | Radius | Bone (offset) | H |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 2-3 | 20 | 5-24 (repeat on 7 when held) | 0 / 1 | 2.2 | 361 | 20 | 30 | 1.2 / 1.5 → 2 / 2 | top (0, 6.3, 6.0 / 8.7) [D] | 1.8 |
| 1 | | | | 2 | 2.2 | 180 | 15 | 20 | 1.8 → 2 | top (0, 6.3, 12.0), fighters only [D] | 1.8 |
| 2 | 2-3 | 22 | 6-24 | 0 / 1 / 2 | 1.7 | 361 | 20 / 20 / 15 | 30 / 30 / 15 | 3.0 / 1.5 / 4.0 → 4 / 2 / 5 | shoulderr (1.4), bust, armr (4.2) [E] | 1.2 |
| 3 | 3-4 | 34 | | 0 / 1 / 2 | 4.0 | 361 | 60 | 80 | 4.3 / 2.5 / 2.6 → 5 / 3 / 3 | top (0, 9.0→6.0, 13.0→11.0) and (0, 8.0, 7.0) [D]; footr [E] | 2.0 |

Jab 1's id 3 (the same box for objects) is not modelled. "Consecutive jab frame 7" (SmashWiki Hit 1) is read as: holding attack
repeats jab 1 from its frame 7 [D, the reading of the field]. Quickest loop 2-3 / 6-7 / 12-13 [R] is what the tests check.

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle | BK | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt | 5-7 | 26 | 0 / 1 / 2 | 7.0 | 361 | 55 | 70 | 3.8 / 3.0 / 2.0 → 5 / 4 / 3 | kneer (4.6), legr (1.2), hip (1.1, -2.0) [E] | angled up / straight / down: same hit, the pose changes [E] |
| Up tilt | 5-11 | 30 | 0 / 1 / 2 | 5.5 | 96 | 28 | 130 | 3.5 / 4.2 / 5.0 → 4 / 5 / 6 | head (-0.5, -0.8), arml (0), arml (4.0) [E] | the fist sweeps front, over, behind (three phases [E]) |
| Down tilt | 5-7 | 28 | 0 / 1 | 7.0 / 5.0 | 80 | 35 | 80 | 3.2 / 4.2 → 4 / 5 | kneel (-1.0), toel [E] | trip 40%, which can never happen at this angle (SmashWiki); trips not built |
| Dash attack | clean 6-9, late 10-25 | 38 | 0 | 8.0 / 6.0 | 50 / 48 | 100 / 100 | 43 / 30 | 3.5 / 2.7 → 4 / 3 | top (0, 1.5, 5.4 / 4.9) [D] | H 1.25, shield +1.0 |

### Smashes [R]

| Move | Frames | Charge hold | FAF | id | Dmg | Angle | BK | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash | 15-17 | 5 | 48 | 0 (arm) | 14.7 | 361 | 25 | 105 | 2.0 → 3 | arml (-1.0→-3.0, 0.7→1.0) [E] | angled up, straight, down |
| | | | | 1 (explosion) | 17.8 | 361 | 25 | 101 up / 99 straight / 100 down | 5.0 → 6 | arml (5.4, 0, -1.0) [E] | the arm is id 0, so it wins where both reach (SmashWiki: the sweetspot is out-prioritised by the sourspot) |
| Up smash | 9-12 | 6 | 40 | 0 / 1 | 14.0 | 83 | 32 | 94 | 5.0 / 4.0 → 6 / 5 | head (2.5, 1.1), bust (1.0) [E] | head intangible 9-12 (**not modelled**: partial intangibility, E19); no jostling 7-19 |
| Down smash | front 5-6, back 14 | 2 | 44 | 0 / 1 | 10.0 front / 12.0 back | 32 / 30 | 30 | 100 | 4.0 / 3.3 → 5 / 4 | top (0, 3.6, ±12.5 / ±7.0) and (-11.5 / -6.0) [D] | |

Charged smash damage is the shared rule (up to 1.4x over 60 frames).

### Aerials [R]

| Move | Frames | Autocancel | Landing lag | FAF | Phase / id | Dmg | Angle | BK | KS | Radius | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Neutral air | clean 3-5, late 6-27 | 1-2, 39+ | 6 | 46 | clean 0 / 1 | 8.0 | 361 | 20 | 100 | 4.0 → 5 | kneer / kneel (0.8) [E] |
| | | | | | late 0 / 1 | 5.0 | 361 | 13 | 90 | 2.8 → 4 | legr / kneel (0.8) [E] |
| Forward air | early 16, clean 17-20, late 21 | 1-2, 43+ | 17 | 60 | early 0 | 12.0 | 361 | 30 | 80 | 3.0 → 4 | arml (3.2) [E] |
| | | | | | clean 0 / 1 | 14.0 | **280** (meteor) | 32 | 78 | 4.0 / 4.6 → 5 / 6 | shoulderl, arml (3.4) [E] |
| | | | | | late 0 / 1 | 10.0 | 361 | 20 | 80 | 4.0 / 4.6 → 5 / 6 | |
| Back air | clean 6-7, late 8-10 | 1-5, 19+ | 6 | 34 | clean 0 / 1 | 10.5 | 361 (backward) | 12 | 106 | 4.5 / 5.9 → 6 / 7 | kneer (4.8), legr (1.6) [E] |
| | | | | | late 0 / 1 | 7.0 | 361 (backward) | 7 | 90 | 4.5 / 5.3 → 6 / 7 | |
| Up air | 4-7 | 1, 17+ | 6 | 31 | 0 / 1 | 7.0 | 75 | 10 | 135 | 4.4 / 5.5 → 6 / 7 | legr (1.2), kneer (3.8) [E]; sweeps front to back (two phases [E]) |
| Down air | loop 5, 7, 9, 11, 13; final 23 | 1-4, 33+ | 15 | 38 | loop 0 / 1 | 1.4 | 94 | 50 / 25 | 15 | 4.0 / 7.0 → 5 / 9 | top (0, -0.5 / 6.0) [D]; **set weight**; SDI 0.8 |
| | | | | | final 0 | 5.5 | 75 | 80 | 100 | 11.0 → 14 | top (0, 6.8) [D]; SDI 0.8 |
| | landing 1-2 | | (15) | landing 16 | landing 0 / 1 | 2.0 | 361 | 0, FKV 60 | 100 | 4.3 → 5 | top (0, 3.2, ±4.0) [D] |

UFD lists the down air's loop at 1.3%; SmashWiki's hitbox table and moveset summary say 1.4%. **Conflict**, resolved to 1.4 (the
named-field hitbox table). The UFD up-air row gives no early autocancel; SmashWiki gives frame 1, which is used.

### Grabs and throws [R]

| Move | Frames | FAF | Data |
| --- | --- | --- | --- |
| Standing grab | 6-7 | 35 | grab box r 3.3 → 4, top (0, 6.6, 4.0 to 8.7) [D]: circles at 0.5 and 1.1 units ahead, 0.8 up |
| Dash grab | 9-10 | 43 | r 2.6 → 3, (0, 6.6, 4.0 to 10.4) [D] |
| Pivot grab | 10-11 | 37 | r 3.3 → 4, (0, 6.6, -4.0 to -14.7) [D] (the engine turns a pivot grab round, so it is written in front) |
| Pummel | 1 (ours 2) | 7 (ours 8) | 1.3%, 361, BK 0, KS 100, FKV 30, H 2.1, top (0, 10.0, 10.0) [D]. Our pummel cannot hit on its first frame, so the hit and the FAF are one frame later, keeping the 6 frames after the hit [D]. UFD's "total 19 including 14 frames of hitlag" agrees with 5-6 frames outside hitlag |
| Forward throw | release 13 | 28 | 8%, 45, BK 60, KS 65 |
| Back throw | release 44 (turns round on 40) | 60 | 11%, 45 behind, BK 70, KS 66; collateral hit on others while spinning 13-39: 8%, 50, BK 100, KS 70, r 4 (positions [E]) |
| Up throw | release 18 | 40 | 7%, 90, BK 70, KS 72 |
| Down throw | release 18 | 40 | 5%, 68, BK 40, KS 90 |

Each throw table also has a second row (`kind=1`: 3%, 361, BK 40-60, KS 100), the reference's hit used when the throw is interrupted;
not modelled. The throws' "Invincibility 1-N" rows (the thrower's intangibility through the release) are **not modelled** (open).

### Ledge and floor [R]

| Move | Frames | Intangible | FAF | Data |
| --- | --- | --- | --- | --- |
| Ledge attack | 24-26 | 1-26 (1st grab; 1-21 2nd, 1-13 3rd: the shared decay) | 56 | 9%, 45, BK 90, KS 20, shield +1, top (0, 5.0, 12.0 to 2.0) [D] |
| Get-up attack, face up | behind 16-17, front 21-22 | 1-22 | 46 | 7%, 48, BK 80, KS 48, shield +8, top (0, 5.0, ∓12.0 to ∓5.0) [D] |
| Get-up attack, face down | behind 19-20, front 24-25 | 1-25 | 46 | as face up (BK 80, KS 48, angle 48) — **not separate here** (one get-up attack slot) |
| Get-up attack, trip | behind 19-20, front 31-32 | 1-7 | 50 | 5%, 361, BK 60, KS 50 — **not separate here** |

## Specials (phase 2: data collected, not built)

| Move | Data [R] |
| --- | --- |
| Fireball | spawned on 17; hitbox phases by age: clean 17-21 5% BK 35 KS 20 r 2.4, mid 22-46 5% BK 28 KS 15 r 2.2, late 47-86 4% BK 22 KS 10 r 2.0; 361 angle, H 0.6, shield damage -2.5 / -2.5 / -2.0; FAF 50. Parameters: lifetime 70, initial speed 1.4 at -10 degrees, gravity 0.1, speed x0.97 after a bounce, vertical speed after a bounce 1.4 to 1.6, 7 bounces then gone, bounces off walls up to 60 degrees, does not pierce. Mario keeps full air control during it |
| Cape | hit 12-14: 7%, 110 degrees, FKV 80, KS 100, r 7.5 / 5.0, effect "Reverse" (turns the victim round), shield +4.0; reflector 9-20 at (0, 6.5, 8), size 6.5, x1.5 damage, x1 speed, x0.75 lifetime, damage threshold 50%; a projectile detector 6-8 that skips the move to frame 9; FAF 36. In the air: horizontal speed / 2.5, air friction 0.0025, +0.3 vertical speed (the fall break), gravity 0.031, fall speed 1; ledge grab from the front from frame 2, from behind from 16 |
| Super Jump Punch | hit 1 frames 3-6: 5%, FKV 160 / 150 / 150 at 60 / 86 / 100 degrees, KS 100, SDI 0, set weight; hits 2-6 frames 7-16 (rehit every 2): 0.6%, FKV 180 / 170 / 110 / 110 at 60 / 92 / 60 / 92, SDI 0, set weight; hit 7 frames 17-18: 3%, 60, BK 50, KS 145, r 9. Intangible 3-6. B-reverse on frame 3 (stick 0.3); angled by the stick (sensitivity 0.5, up to 12 degrees); height x1.1 grounded, x1.08 aerial; air speed x0.67 on startup; helpless air speed x0.6; **landing lag 30**; ledge grab from both sides from frame 14; animation 37. **The rise itself is the animation's root motion, not a published number**: see the phase-2 questions |
| F.L.U.D.D. | charge entered in 19 frames (shield-cancellable from 20), loops on 19-20; **total charge 80 frames (SmashWiki parameter) or 100 (UFD note): conflict**; heavier shot from 70 frames of charge; 7 pumps, one every 5 frames (2-13, 7-18, ... 32-43 from release); FAF 49 from release; recoil 0.02 per charge frame on release. Pump: 0%, 55 degrees, FKV 70, KS 100, r 4.0, flinchless, BK rising up to +55 by charge; lifetime 12, speed 4.3 + up to 4.3, gravity 0.17, fall 3, size x0.8 to x1.2; aimed from 12 degrees, up to +50 / -18 at 1.9 degrees a frame. Charge-cancel with no lag |

## Unresolved

1. F.L.U.D.D.'s full charge: 80 (SmashWiki parameter table) against 100 frames (UFD). Needs a third source or footage.
2. Super Jump Punch, Cape's rise and the Longsword's / Claws' up special travel are animation root motion: the published numbers are
   multipliers on it. A measured rise height (footage frame-stepped, or a published recovery-distance table) is needed for 1:1.
3. The angled forward tilt / smash stick threshold is not published: ours is 0.25 of the stick, short of the up / down tilt line [E].
4. Throw intangibility for the thrower ("Invincibility 1-13" etc.): which fighter it applies to is not stated on the pages read.
