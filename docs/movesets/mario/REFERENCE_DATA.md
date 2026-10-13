# Reference data for the Embers moveset (the reference game's Mario)

Mechanical reference for the fourth moveset (phase 1: body and normals; phase 2: the specials and the wall jump). In the game it is the **tumbler** body with the **Embers** weapon (class 3, built-in
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
x speed 0.2: built in phase 2, see "Wall jump" below), aerial landing lags 6 / 17 / 6 / 6 / 15 (n / f / b / up / down, agreeing with UFD).

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

UFD lists the down air's loop at 1.3%; SmashWiki's hitbox table and moveset summary and dragdown.wiki's data page say 1.4%.
**Conflict**, resolved to 1.4 (two sources of three, and the named-field hitbox table). The UFD up-air row gives no early autocancel; SmashWiki gives frame 1, which is used.

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

## Specials (phase 2, built: sims 58 to 64)

Sources: SmashWiki `Mario_(SSBU)/Neutral_special`, `/Side_special`, `/Up_special`, `/Down_special` (raw wikitext: hitbox rows,
timing tables and the parameter tables), cross-checked with dragdown.wiki `SSBU/Mario/Data` (hit data and notes) and UFD. All
frame numbers are the reference's.

### Up special: the rising punch (Super Jump Punch)

| Item | Value | Label |
| --- | --- | --- |
| Hit 1 | frames 3-6, 5%, set knockback 160 / 150 / 150 at 60 / 86 / 100 degrees, KS 100, BK 0, set weight, SDI 0, radius 2.5 / 4.0 / 4.0, top (0, 6.5, 0 / 5.0 / 9.2); the 60-degree box at the body does not hit grounded fighters (`g=f`) | [R]; positions and radii [D] |
| Hits 2-6 | frames 7-16, re-armed every 2 frames (on 7, 9, 11, 13, 15), 0.6%, set knockback 180 / 170 / 110 / 110 at 60 / 92 / 60 / 92, set weight, SDI 0, radius 3.0 / 3.8 / 3.0 / 3.8, top (0, 6.5 / 11.5, 2.5 / 8.0) | [R]; [D] |
| Hit 7 | frames 17-18, 3%, 60 degrees, BK 50, KS 145, radius 9, top (0, 11.5, 8.0 / 2.5) | [R]; [D] |
| Total | 11% when every hit connects | [R] (SmashWiki: "up to 11.0% total"); checked by a test |
| Intangible | 3-6 (from the ground and the air) | [R] |
| Animation, landing | 37 frames, then the special fall; landing lag 30 (interruptible 31) | [R] |
| Special fall | air speed x0.6 | [R] |
| Ledges | either side from frame 14 | [R] |
| Reverse | stick back on frame 3 (sensitivity 0.3); here the shared up-special turn window (frames 1-4, any back tilt) | [R] / [E] (shared window) |
| Angling | sensitivity 0.5, at most 12 degrees; read on frame 3 here, evenly from 0.5 of the stick to the rim | [R]; the frame and the reading [E] |
| Height multipliers | x1.1 from the ground, x1.08 from the air | [R] |
| Startup momentum | x0.67 in the air | [R] (read as the horizontal speed kept) |
| Rise | **not published** (the animation's root motion): 40 reference units before the multipliers (44 from the ground), about 11 forward, over frames 3-20 | **[E]** (no published height or recovery-distance figure was found: open) |

The reference rule that set-knockback hits keep traditional launch physics (SmashWiki, Knockback: "Moves with set knockback retain
traditional knockback physics") is what lets the loop carry its target: without it the launch speed-up sent the target out of the next
hit. It applies to every set-knockback hit in the game (sim 59).

### Neutral special: the fireball

| Item | Value | Label |
| --- | --- | --- |
| Hit by age | clean (frames 17-21) 5%, BK 35, KS 20, radius 2.4; late (22-46) 5%, BK 28, KS 15, radius 2.2; latest (47-86) 4%, BK 22, KS 10, radius 2.0; 361 degrees, hitlag x0.6 | [R] |
| Shield damage | -2.5 / -2.5 / -2.0 (additive) | [R]; as our percent 50 / 50 / 50 [D] |
| Timing | out on frame 17 (it appears at the end of 16 and hits from the next frame), FAF 50, animation 63 | [R] |
| Flight | life 70 frames, 1.4 at -10 degrees, gravity 0.1 | [R] |
| Bounces | off the ground: vertical speed 1.4 to 1.6, speed x0.97, 7 bounces then gone; walls under 60 degrees | [R]; the bounce speed read as the falling speed held between the two [D]; "7 bounces before disappearing" read as removed on the 8th touch [D]; no wall bounce here (the game's walls are upright) |
| Spawn point | (0.8, 0.9) world units from the feet | [E] |
| Air control | full (no motion) | [R] |

### Side special: the cape

| Item | Value | Label |
| --- | --- | --- |
| Swing | frames 12-14, 7%, 110 degrees, set knockback 80, KS 100, BK 0, radius 7.5 / 5.0 at top (0, 6.7, 9.7 / 5.0), shield damage +4, the "Reverse" effect, no clank | [R]; positions [D]; shield damage as 157 percent [D] |
| Reflector | frames 9-20, offsets (0, 6.5, 8), size 6.5, damage x1.5, speed x1, lifetime x0.75, damage threshold 50% | [R]; the threshold read as "shots above 50% pass" [E] |
| Detector | frames 6-8, radius 8 along 2.5 to 8 ahead at 6.5 up; skips the move to frame 9 | [R]; one circle covering the capsule [D] |
| Reflected shot's vertical speed | turned round | **[E]** (not stated by the sources) |
| Aerial | horizontal speed / 2.5, air friction 0.0025, +0.3 vertical speed, gravity 0.031, fall speed 1 | [R] |
| The lift | once per airtime, on frame 2, from a stopped fall (`max(vy, 0) + 0.3`) | the once-per-airtime rule [R] (the player guides); the frame and the stopped fall [E] |
| Ledges | in front from frame 2, behind from 16 | [R] |
| FAF | 36 (animation 58) | [R] |

### Down special: the water pump (F.L.U.D.D.)

| Item | Value | Label |
| --- | --- | --- |
| Wind-up | 19 frames, then charging (charges between 19-20) | [R] |
| Full charge | **80 frames of charge** (SmashWiki's parameter "total charge time"), counted from the charge's start: with the wind-up that is the 100 frames UFD and dragdown.wiki ("100F to fully charge") give | [R] SmashWiki + [D] reconciliation; **default chosen** (owner's rule: a third source was found and agrees once the wind-up is counted) |
| Cancels | shield or jump keeps the charge with no lag; special shoots | [R] |
| At full charge | keeps itself (stored) and the move ends | **[E]** (the sources say a fully charged pump fires on the next use, not whether the stance holds) |
| Shot | recoil 0.02 a charge frame; 7 pumps every 5 frames (2-13, 7-18, ... 32-43 from the release); FAF 49 from the release | [R] |
| Pump | 0%, 55 degrees, set knockback 70, KS 100, radius 4, flinchless (a push here); life 12; speed 4.3 + up to 4.3; gravity 0.17, fall 3; size x0.8 to x1.2; base knockback up to +55 with the charge | [R]; the push as an added velocity [E] |
| Aim | default 12 degrees, up to +50 / -18, 1.9 degrees a frame | [R]; the stick threshold (0.3) [E] |
| Spawn point | (1.0, 0.8) world units from the feet | [E] |
| Pierce | each pump is removed by its first hit | [E] |

### Wall jump

From the parameter file (dragdown.wiki): `wall_jump_type` true, `passive_wall_jump_x_speed` 1.3, `passive_wall_jump_y_speed` 2.3 [R].
"Passive" is the reference's word for a tech, so those are its wall-tech jump speeds; they are used for the wall jump too [E]. Each
wall jump without touching the ground keeps 0.96 of the last (SmashWiki, Wall jump) [R]. `wall_jump_enable_x_speed` 0.2 is not
modelled (its meaning is not published). The wall-contact range (1/8 unit) is [E].

## Defaults chosen for the phase-1 questions (owner's defaults, 2026-10-13)

1. **F.L.U.D.D.'s full charge**: resolved as above (80 frames of charge, 100 from the input) [R + D]. Not open any more.
2. **Rise heights** (the rising punch, the cape's lift, the Longsword's and Claws' up specials): no published height or recovery
   distance was found (searched SmashWiki, dragdown, UFD and the web; the owner records no footage for now). The rising punch's rise
   is [E] (above); the Longsword keeps its earlier estimate (about 44 reference units); the Claws' flight is built from its published
   dash parameters, so it is [D]. **Open**.
3. **Angle threshold** for the angled forward tilt and smash: kept at 0.25 [E].
4. **Pummels**: re-checked (SmashWiki): the Needles' (1%, hitlag x1.7, FAF 6) and the Claws' and Longsword's (1.3%, set knockback 30,
   hitlag x2.1, FAF 7) were too long by their hitlag; fixed (sim 63), and a pummel now uses its own hitlag multiplier.
5. **Projectile pool**: raised to 16 (sim 62).
6. **Get-up attack variants**: wait for E17; still one slot (face up).

## Conflicts

* Down air loop: UFD 1.3%, SmashWiki 1.4% (hitbox table and moveset summary), dragdown.wiki 1.4% ("Multihits"). **Resolved to 1.4**
  (two of three, and the named-field hitbox table).
* Full charge of the pump: see above (no conflict once the wind-up is counted).

## Unresolved

1. The rising punch's rise, the cape's lift and the Longsword's up-special rise are animation root motion: the published numbers are
   multipliers on it. No published height or recovery distance was found; footage (frame-stepped in training mode) would settle them.
2. The angled forward tilt / smash stick threshold is not published: ours is 0.25 of the stick, short of the up / down tilt line [E].
3. Throw intangibility for the thrower ("Invincibility 1-13" etc.): which fighter it applies to is not stated on the pages read.
4. Whether a fully charged pump holds its stance or keeps itself at once [E: keeps itself]; whether a reflector turns a shot's
   vertical speed round [E: the cape does].
