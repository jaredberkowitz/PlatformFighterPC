# Reference data for the fifth moveset (the reference game's Captain Falcon)

Mechanical reference for the fifth port, collected 2026-10-13 for version **13.0.1**, before anything was built. In game: the
**Gauntlets** weapon on the **sprinter** body, built-in fighter **Sprinter** (owner decision 10; class 4). Updated at sim 73 with the
specials' angle types, the second air dodge source and the fitted distances (what was built and tested: `IMPLEMENTATION_REPORT.md`). The reference character's name is not used in the game. What the engine can and cannot do with this data is in
`PROJECT_AUDIT.md`; the format follows `docs/movesets/embers/REFERENCE_DATA.md` and `docs/movesets/needles/REFERENCE_DATA.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Captain_Falcon_(SSBU)/<move>` (31 pages: `Neutral_attack` and `/Hit_1`, `/Hit_2`, `/Hit_3`,
  `/Infinite`; the tilts, smashes, aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(front|back|trip)`, the four
  specials), read as **raw wikitext** (`index.php?title=...&action=raw`), so every hitbox value comes from a named template field
  (`damage`, `angle`, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff` hitlag multiplier, `sdi`, `sd` shield damage, `setweight`,
  `trip`, `af` angle type, `g=f` / `a=f` grounded / aerial targets, `clang`, `rebound`) and the 361-icon column shift cannot happen.
  `Captain_Falcon_(SSBU)` for the attributes table, the moveset summary and the update history; `Template:UpdateList_(SSBU)/<version>`
  for the per-version change lists; the wiki's hitbox-row template for what each field means. https://www.ssbwiki.com/Captain_Falcon_(SSBU)
- **ultimateframedata.com/captain_falcon** (startup, total frames, landing lag, autocancel, active frames, notes; jump air times,
  dodges, stats). UFD's "total frames" is FAF - 1. https://ultimateframedata.com/captain_falcon
- **dragdown.wiki `SSBU/Captain_Falcon/Data`** (the parameter file and a second frame-data table). https://dragdown.wiki/wiki/SSBU/Captain_Falcon/Data
- **ssbwiki.com, Dash** (the current-game table: initial dash, run, dash frames, pivot dash frames, run accelerations). https://www.ssbwiki.com/Dash
- **ssbwiki.com move articles** `Falcon_Dive`, `Raptor_Boost`, `Falcon_Kick`, `Falcon_Punch`, `Knee_Smash` (the sections on the current game, for
  behaviour the tables do not show).
- **kuroganehammer.com** came back empty (as in the Mario run), so it was not used as a cross-check.

Update history applying to 13.0.1 (ssbwiki.com, every Captain Falcon entry): **3.1.0** forward tilt BK 50 → 60; up tilt startup 17 → 14
(FAF 40 → 37); forward air sweetspot 3.0 → 3.75; back air 25/94 → 27/97; Raptor Boost uppercut startup 6 → 5, armor threshold 8% →
10%, grounded hit +3 hitstun; Falcon Kick clean hit longer (aerial 16-18 → 16-20). **7.0.0** shield x1.05 (not modelled: shields are
global); jab 1's pulling hitbox gets higher priority. **8.0.0** dash attack FAF 38 → 35 and shieldstun multiplier 1.875 → 1.5; down tilt
KS 80 → 86; up smash hit 2 KS 72/74/80 → 80/81/88; Raptor Boost uppercut angle 76 → 85, KS 52 → 30, inner box stretched (Z 5.5 → 5.5 to
12.5), the overhand's meteor box given top priority; Falcon Kick startup 15/16 → 13/14 (FAF 71 → 69 ground, 68 → 66 off edge, 59 → 57
aerial), clean KS 84 → 88 ground, 52 → 57 aerial. **9.0.0** up smash hit 1's 12% boxes swap priority and lose knockback (30/8 base, 20/10
set → 5/0 base, 10/0 set). **11.0.0** Falcon Dive's throw FAF 55 → 24 (see conflict 3). Every table below is the post-11.0.0 data the
pages show. No data from the 3DS/Wii U game or earlier was used.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift, a reading of a field);
**[E]** estimate. Hitbox **positions** on a bone other than `top` are [E] (the reference attaches them to its own skeleton; ours go on our
body where that bone is in the move's pose). Offsets on `top` (the character's root at the feet, `zpos` forward, `ypos` up) convert
directly [D]. A reference "capsule" (`zpos=4.0 to 9.7`) is two or three circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first, FAF as published); the sim stores them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius and position in world tenths = reference value x 1.25, rounded half up to a whole tenth
  [D] (the Mario port's rule).
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; **361** is the reference's 361 angle (named "angle 361"
  here). H = hitlag multiplier (`ff`), SDI = SDI multiplier.
- **Angle type** (`af`, the wiki's hitbox-row template): 0 standard (away from the attacker by the target's position; **the template's
  default when a row has no `af`**, checked in `Template:UltimateHitboxTableRow`), 1 speed-based (the attacker's direction of movement),
  3 forward (the attacker's facing), 4 backward (opposite the attacker's facing). Built as E26 (`Hitbox::angle_type`, sim 67; type 1
  since sim 73): every normal with a box at the body or behind it carries its type since sim 67, every special box since sim 73 (the
  specials' types are in each special's section below).
- **Targets**: `a=f` cannot hit aerial targets (grounded only), `g=f` cannot hit grounded targets (aerial only) [R].
- Shield damage: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd` becomes
  `100 x (damage + sd) / damage` [D].
- `clang=f rebound=f`: the hitbox never clanks with other attacks and its owner never rebounds [R].

## Attributes [R]

ssbwiki.com attributes table, agreeing with the UFD stats and the parameter file:

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 104 | 0.987 | 1.98 | 2.552 | 0.105 | 1.218 | 0.01 + 0.065 | 0.0075 | 0.12 | 1.865 / 2.984 | 37.31 / 17.98 / 37.31 | 3 | 1 |

Parameter file (dragdown) [R]: `walk_accel_add` 0.105, `walk_accel_mul` 0.158 (ours: `walk_accel` 0.105, `walk_accel_stick` 0.158);
`run_accel_add` 0.11, `run_accel_mul` 0.066 (ours: `run_accel` 0.11, `run_accel_stick` 0.066; ssbwiki.com's Dash table agrees: base 0.11,
additional 0.066, max 0.176); `ground_brake` 0.105; initial dash **15 frames**, pivot dash 17 (ssbwiki.com Dash table); `jump_initial_y`
20.5205; `jump_speed_x` 0.9, `jump_speed_x_mul` 0.75, `jump_speed_x_max` 2.1, `jump_aerial_speed_x_mul` 0.9; `air_brake_y` 0.008;
`landing_heavy_frame` 4 (so 3 frames of lag here [D]); `landing_attack_air_frame` n / f / b / hi / lw 7 / 18 / 10 / 10 / 12 (agreeing
with UFD and ssbwiki.com); `cliff_jump_y` 38.655, `cliff_jump_x_speed` 0.6 (the ledge jump); `jump_count_max` 2; `scale` 1.03;
`shield_radius` 11.1 (shields are global here, not used); `jostle_front` 1.8, `jostle_back` 0.6 (no jostling in our engine, see the
audit); **wall jump**: `wall_jump_type` true, `passive_wall_jump_x_speed` 1.4, `passive_wall_jump_y_speed` 3.1,
`wall_jump_enable_x_speed` 0.2, `passive_wall_x_speed` 0.5 (not built: E15). Footstool (`tread_*`) not modelled for anyone.

Jump air times (UFD) [R]: short hop 34, full hop 48, short hop fast fall 24, full hop fast fall 34.

Dodges (UFD) [R]: spot dodge 26 (UFD "21/26"; intangible 3-17), forward roll 30 (4-15), back roll 35 (5-16), neutral air dodge **42**
(3-30), directional down 62, diagonal down 66, sideways 72, diagonal up 85, up 94 (3-21), air dodge landing lag 10 / 11-19. **Second
source (sim 73, owner decision 18)**: ssbwiki.com's `Air_dodge` page (raw wikitext, the current game's table) gives the same: neutral
42, intangible 3-30 (6-27 fully stale); sideways 72, up 94, down 62, diagonally up 85, diagonally down 66, intangible 3-21 (6-20 fully
stale). That table has a cleanup tag ("some values may be inaccurate"); dragdown's page has the air dodge fields but no values. Mario's
52 is also in both. **42 is kept, [R] from two sources**.

Body size [E]: the reference model is tall (its `top`-bone hitboxes reach 29 units up on the up smash, 10-11 on the jab); our sprinter
body needs a taller hurtbox than the tumbler's (`ecb_height`), fitted to those heights.

## Normals

All frame numbers [R] (ssbwiki.com timing tables; UFD and dragdown agree on every startup and FAF unless listed under "Conflicts").
Damage, angle, BK, KS, FKV, H, SDI [R]. Radius column: reference → world tenths [D].

### Jab [R]

| Hit | Frames | FAF | Continuable | id | Dmg | Angle | BK | KS | Radius | Bone (offset) | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 3 | 18 | 5-30 (consecutive jab frame 7) | 0 | 1.5 | 361 | 20 | 25 | 2.0 → 3 | top (0, 10.0, 6.0) [D] | 1.6 | |
| | | | | 1 | 1.5 | 180 | 20 | 20 | 2.5 → 3 | top (0, 10.0, 12.0), fighters only [D] | 1.6 | pulls the target in (7.0.0 priority) |
| | | | | 2 | 1.5 | 361 | 20 | 25 | 2.3 → 3 | top (0, 10.0, 8.8) [D] | 1.6 | |
| | | | | 3 | 1.5 | 361 | 20 | 20 | 2.5 → 3 | top (0, 10.0, 12.0) [D] | 1.6 | the same box as id 1 for objects; not modelled |
| 2 | 5 | 19 | rapid jab 7-9, jab 3 8-30 | 0 / 1 / 2 | 1.5 | 361 | 20 / 20 / 20 | 25 / 25 / 20 | 2.5 / 3.0 / 3.6 → 3 / 4 / 5 | top (0, 10.0, 6.5 / 10.0 / 14.0) [D] | 1.2 | **+3 hitstun** |
| 3 (the knee) | 6-7 | 33 | | 0 / 1 | 5.0 | 361 | 70 | 100 | 5.5 / 3.8 → 7 / 5 | top (0, 11.0, 12.0) and (0, 10.0, 6.5) [D] | 2.0 | |

**Rapid jab** (`/Infinite`) [R]: entered from jab 2 when, since jab 1 started and before the end of jab 2's rapid window (7-9), attack has
been **pressed at least 3 times and released at least 2 times**; otherwise a press in 8-30 gives jab 3. The rapid window wins where both
are open. Loop: hits on its frames 5, 7, 9, 11, 13, 15, 17, 19 (eight per cycle), loop points 2-3 and 19-20 (the cycle returns from 19-20
to 2-3); **after one full cycle, releasing attack at any point** goes to the finisher.

| Part | id | Dmg | Angle | BK | KS | Radius | Bone (offset) | H | SDI | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Loop | 1 | 0.6 | 361 | 8 | 15 | 2.25 → 3 | shoulderr (7.5 → 2.0, 0.8 → -1.0, -0.5 → 0) [E] | 0.5 | 0.4 | +2 hitstun, shieldstun x9 |
| Loop | 2 | 0.6 | 361 | 8 | 15 | 6.5 → 8 | top (0, 8.0, 15.0 → 12.0) [D] | 0.5 | 0.4 | the scripts set the modifiers only on id 1, but a known engine glitch applies them to id 2 too, so **both** get +2 / x9 [R] |
| Finisher (hit 6-7, FAF 50) | 0 / 1 | 3.0 | 45 | 110 | 55 | 4.5 → 6 | top (0, 13.0 → 5.5, 7.0 / 13.5) [D] | 2.0 | | |

Quickest loop (from jab 1's frame 1) [R]: jab 1 on 3, held repeat 7, jab 2 on 9, jab 3 on 17-18; rapid jab 15, 17 ... 29; finisher 35-36;
FAF 18 / 23 / 44 / 79. That is what the tests should check.

"Consecutive jab frame 7" is read as in the Mario port: **holding attack repeats jab 1 from frame 7** [D, the reading of the field].
dragdown lists the loop hits as 3, 5 ... 15 (a different frame origin); the ssbwiki.com numbering is used [D].

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle | BK | KS | Radius | Bone (offset) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt, straight | 7-10 | 30 | 0 / 1 / 2 | 8 / 8 / **9** | 361 | 60 | 58 | 4.3 / 4.3 / 5.3 → 5 / 5 / 7 | legr (3.5), hip (0), kneer (5.5, -1.0, 0.5) [E] | id 2 is the foot (the sweetspot) |
| Forward tilt, up or down | 7-10 | 30 | 0 / 1 / 2 | 9 / 9 / **10** | 361 | 60 | 58 | as straight | as straight, in the angled pose [E] | angling adds 1% to every box and nothing else (UFD: "damage and shieldstun +1 when angled") |
| Up tilt (axe kick) | 14-17 | 37 | 0 | 11 | **285** | 35 | 100 | 6.0 → 8 | kneer (7.0, -1.0, 0.5) [E] | **grounded targets only**; H 1.0 |
| | | | 1 | 11 | 285 | 22 | 90 | 3.5 → 4 | kneer (7.0, -1.0, 0.5) [E] | **aerial targets only**; H 1.4 |
| | | | 2 / 3 | 11 | 285 | 35 | 100 | 4.0 / 3.0 → 5 / 4 | legr (5.0 / 0) [E] | |
| | | | | | | | | | | **lower leg intangible 14-17** (partial: E19); angle type 0 |
| Down tilt | 11-12 | 35 | 0 / 1 | 10 | 25 | 38 | 86 | 4.8 → 6 | kneel (6.5, -1.0), legl (3.5) [E] | trip 50% (only possible on slopes at this angle: ssbwiki.com trivia) |
| | | | 2 | 10 | 155 | 38 | 86 | 3.5 → 4 | top (0, 3.5, -1.0) [D] | angle type 0: a target behind is sent the other way |
| Dash attack | clean 7-9, late 10-16 | 35 | 0 | 10 / 6 | 65 | 90 | 67 | 6.2 / 4.8 → 8 / 6 | top (0, 10.0 → 8.0 / 10.0 → 6.0, 12.0) [D] | H 1.25; shield +1.0 (110 / 117 [D]); **shieldstun x1.5** |

### Smashes [R]

| Move | Frames | Charge hold | FAF | id | Dmg | Angle | BK | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash, up | 19-22 | 11 | 60 | 0 / 1 | 20 | 45 | 36 | 94 | 3.8 / 3.2 → 5 / 4 | arml (2.2, 0.7, 0.5), shoulderl [E] | flame (presentation only) |
| Forward smash, straight | | | | 0 / 1 | 19 | 43 | 36 | 94 | | | |
| Forward smash, down | | | | 0 / 1 | 20 | 38 | 36 | **87** | | | |
| Up smash, hit 1 (early 22, late 23) | 22-23 | 8 | 46 | 0 | 7 | 110 | 0, FKV 150 | 90 | 5.7 → 7 | top (0, 7.5, 10.0) [D] | the "scoop" in front |
| | | | | 1 (early only, 22) | 7 | 96 | 0, FKV 80 | 90 | 6.0 → 8 | top (0, 17.0, 5.0) [D] | |
| | | | | 2 | 12 | 80 | 0 | 90 | 4.8 → 6 | top (0, 28.0, 5.0) [D] | a moving box (offsets to (5, 29) over 7.5 frames) [R]; ids 2 and 3 swapped priority in 9.0.0 |
| | | | | 3 | 12 | 80 | 5, FKV 10 | 90 | 4.8 → 6 | top (0, 21.0, 1.5) [D] | angle type 3 |
| Up smash, hit 2 | 28-29 | | | 0 / 1 / 2 | 14 / 14 / 13 | 83 / 85 / 70 | 70 | 80 / 81 / 88 | 6.0 / 5.5 / 5.0 → 8 / 7 / 6 | top (0, 29.0 / 21.0 / 16.0, 5.5 / 2.5 / 0.5) [D] | right lower leg intangible 22-23, left lower leg 28-29 (E19) |
| Down smash | front 19-20, back 29-30 | 11 | 49 | front 0 / 1 / 2 | 14 | 28 | 30 | 90 | 4.5 / 4.2 / 3.7 → 6 / 5 / 5 | kneer (4.9, -0.9), kneer, legr [E] | |
| | | | | back 0 / 1 / 2 | 18 | 28 (backward) | 20 | 84 | as front | the same bones, behind [E] | a separate hit (part 1) |

Charged smash damage is the shared rule (up to 1.4x over 60 frames). The forward smash's charge hold is on frame 11 (ssbwiki.com "charges
between 11-12", UFD "charge hold is frame 11"); up smash 8, down smash 11 [R].

### Aerials [R]

| Move | Frames | Autocancel | Landing lag | FAF | Phase / id | Dmg | Angle | BK | KS | Radius | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Neutral air | hit 1 7-8, hit 2 13-15 | 1-3, 32+ | 7 | 40 | hit 1, 0 | 4 | 74 | 50 | 25 | 6.0 → 8 | legr (3.2) [E]; aerial targets only; **set weight** (all of hit 1) |
| | | | | | hit 1, 1 | 4 | 68 | 65 | 75 | 6.0 → 8 | legr (3.2); grounded targets only; SDI 1.4 |
| | | | | | hit 1, 2 | 4 | 80 | 50 | 25 | 4.8 → 6 | kneer (4.2, -0.7); aerial only |
| | | | | | hit 1, 3 | 4 | 74 | 65 | 75 | 4.8 → 6 | kneer (4.2, -0.7); grounded only; SDI 1.4 |
| | | | | | hit 1, 4 | 4 | 68 | 50 | 25 | 3.5 → 4 | hip; everyone |
| | | | | | hit 2, 0 / 1 / 2 | 6 | 361 | 45 | 120 | 5.2 / 6.0 / 3.8 → 7 / 8 / 5 | kneel (4.4, -0.7), legl (3.2), hip [E] |
| **Forward air (the knee)** | **clean 14 only**, late 15-30 | 1-4, 42+ | 18 | 46 | clean 0 (sweetspot) | **22** | **32** | 30 | 81 | **3.75 → 5** | legl (4.4, -0.2 → 0.3, -1.0) [E]; **electric** (1.5x hitlag); H 1.2 |
| | | | | | clean 1 (sourspot) | 6 | 361 | 35 | 80 | 3.0 → 4 | top (0, 6.0, 0) [D]; H 1.2 |
| | | | | | late 0 / 1 | 3 | 361 | 35 | 80 | 4.7 / 3.0 → 6 / 4 | legl (4.4, -0.2, -1.0) [E], top (0, 6.0, 0) [D]; trip 20% |
| Back air | clean 10-11, late 12-15 | 1-4, 19+ | 10 | 36 | clean 0 / 1 / 2 | 13 | 361 (backward, type 4) | 27 | 97 | 5.5 / 4.0 / 4.0 → 7 / 5 / 5 | arml (4.0), shoulderl (3.2), shoulderl (-0.8) [E] |
| | | | | | late 0 / 1 / 2 | 8 | 361 (backward) | 27 | 97 | as clean | |
| Up air | clean 7-10, late 11-12 | 24+ (none early) | 10 | 32 | clean 0 / 1 | 10 / 9 | 66 | 20 | 96 | 4.5 / 5.0 → 6 / 6 | legl (3.2, 2.1), kneel (6.2, 0.9, -0.4) [E] |
| | | | | | late 0 / 1 | 9 / 8 | **30** | 12 | 80 | as clean | the low-angle "late" kick |
| **Down air** | 16-18 | 1-3, 39+ | 12 | 45 | 0 (sweetspot) | 14 | **270** (meteor) | 10 | 100 | 5.9 → 7 | top (0, -5.2, 0.8) [D]: below the feet; id 0 wins where both reach |
| | | | | | 1 (sourspot) | 14 | 361 | 40 | 100 | 6.0 → 8 | top (0, 1.0, 0.8) [D] |

The forward air's clean sweetspot is the reference's 1-frame window (the 3.1.0 enlargement to 3.75 included). Its sourspot on `top` sits at
the hips. The knee's Special Zoom parameters exist but are unused in game [R] (presentation only anyway). The down air disables jostling
4-18 (no jostling here).

### Grabs, pummel and throws [R]

| Move | Frames | FAF | Data |
| --- | --- | --- | --- |
| Standing grab | 6-7 | 36 | id 0 r 3.3 → 4, top (0, 8.0, 4.0 to 9.7) [D], grounded targets only; id 1 r 1.65 → 2, (0, 8.0, 2.35 to 11.35), aerial targets only |
| Dash grab | 9-10 | 44 | id 0 r 2.6 → 3, (0, 8.0, 4.0 to 11.6); id 1 r 1.3 → 2, (0, 8.0, 2.7 to 12.9) |
| Pivot grab | 10-11 | 39 | id 0 r 3.3 → 4, (0, 8.0, -4.0 to -16.2); id 1 r 1.65 → 2, (-2.35 to -17.85) (written in front: the engine turns a pivot grab round [D]) |
| Pummel | 1 (ours 2) | 7 (ours 8) | 1.3%, 361, BK 0, KS 100, FKV 30, r 6.2 → 8, top (0, 10.0, 12.0) [D], H 2.1. The engine's pummel cannot hit on its first frame, so the hit and FAF are one frame later [D] (Mario's rule). UFD's "19 including 13 frames of hitlag" agrees |
| Forward throw | hit on the held fighter 11-12, release 13 | 33 | hit 3.5%, 80, BK 70, KS 100, H 1.5 (r 4.8 on arml [E]); throw 4%, **45**, BK 55, KS 120. Thrower invincible 1-13 |
| Back throw | hit 12, turns round 13, release 13 | 46 | hit 3.5%, 110, BK 70, KS 100, H 0.6; throw 4%, **135**, BK 60, KS 130 (an angle past 90 sends behind the thrower's original facing: backward and up [D]). Invincible 1-13 |
| Up throw | hit 12-13, release 14 | 38 | hit 4%, 80, BK 60, KS 100, H 1.6; throw 5%, 84, BK 68, KS 100. Invincible 1-14 |
| Down throw | release 21 | 40 | 6%, 68, BK 65, KS 70. Invincible 1-21 |

Each throw also has a second row (`kind=1`: 3%, 361, BK 60, KS 100), used when the throw is interrupted; not modelled. The throws' hit on
the held fighter is the "hit then throw" pattern (damage and hitlag, the victim stays held). The thrower's invincibility rows are not
modelled (as for Mario).

### Ledge and floor [R]

| Move | Frames | Intangible | FAF | Data |
| --- | --- | --- | --- | --- |
| Ledge attack | 22-24 | 1-24 (1st grab; 1-19 2nd, 1-12 3rd, none from the 4th: the shared decay) | 56 | 10%, 45, BK 90, KS 20, shield +1 (110 [D]), r 5.0 → 6, top (0, 5.0, 12.5 to 3.5) [D] |
| Get-up attack, face up (ssbwiki.com "back") | 14-15 and 20-21 | 1-21 | 46 | 6%, **48** (dragdown: 361, conflict 5), BK 80, KS 48, shield +8 (233 [D]); one box sweeping top (0, 9.0, 12.0 to -12.0): both sides each time [D] |
| Get-up attack, face down (ssbwiki.com "front") | behind 14-15, front 24-25 | 1-25 | 46 | 7%, 48, BK 80, KS 48, shield +8 (214 [D]); top (0, 5.0, -16.0 to -2.0) then (0, 5.0, 15.0 to 4.0) |
| Get-up attack, trip | front 18-19, behind 23-24 | 1-7 | 50 | 5%, 361, BK 60, KS 50, shield +8 (260 [D]); (0, 5.0, 10.5 to 4.5) then (0, 5.0, -9.5 to -4.0). ssbwiki.com gives interruptible 50 with an animation of 49 (the FAF is the animation end + 1) |

One get-up attack slot exists (as for Mario): the face-up one would be used.

## Specials

### Falcon Punch (neutral special) [R]

Four versions, chosen by the situation when it starts and by the stick on frame 15:

| Version | Hit | FAF | Dmg | Angle | BK | KS | Radii (ids 0 / 1 / 2) | Bones |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Grounded | 53-57 | 104 | 25 | 361 | 93 | 59 | 4.5 / 2.5 / 4.0 → 6 / 3 / 5 | armr (-1.0), shoulderr (-2.5), armr (4.2) [E] |
| Grounded, reversed | 62-66 | 118 | **28** | 361 | **85** | **65** | as grounded | |
| Aerial | 53-57 | 104 | 22 | 361 | 93 | 59 | 5.175 / 2.875 / 4.6 → 6 / 4 / 6 (x1.15) | as grounded |
| Aerial, reversed | 62-66 | 118 | 25 | 361 | 93 | 59 | as aerial | |

Every box: H 1.5, flame (presentation), **no clank and no rebound** (`clang=f rebound=f`), Special Zoom on hit (presentation). **Angle
type**: no row has an `af`, so 0, by position [R] (raw wikitext, re-read 2026-10-13).

- **Turnaround** [R]: the move "can be reversed on frame 15" (stick held backward then): it changes to the reversed version from
  that version's frame 15 ("transition start 15"), Falcon turns round on frame 35, the hit is 62-66 and the FAF 118 (counted from the
  start of the original move). Frames 1-14 are shared. The usual B-reverse at the start also applies (the shared rule).
- **Aerial movement** [R]: on starting, a set 0.2 upward speed; "momentum forward 1"; the horizontal speed is multiplied by 0.92 each frame
  for 19 frames, then the normal air friction applies. On frame 51 (normal) or 60 (reversed) the stick is read: held up or down between
  0.25 and 0.65 of its travel (the stick sensitivity), Falcon gets a small boost travelling at **35 degrees** up or down. The boost's speed
  is **not published** [gap].
- **Animation lengths** [R]: grounded 115, aerial 124; reversed grounded 123, aerial 133 (after the FAF the animation keeps going unless
  interrupted; matters only for ledge grabs and fast falls).
- **Landing during the aerial version**: not stated on the pages read [gap: footage]. **Grounded slide**: not stated [gap]; the move
  article says the punch moves Falcon forward less than in the previous game.

### Raptor Boost (side special) [R]

A dash that searches for a target; touching one switches to the hit move.

| Part | Frames | FAF | Data |
| --- | --- | --- | --- |
| Grounded dash | search 10-25 | **72 on a miss** (animation 89) | search boxes (0% "inert", never clank, indirect): id 0 r 4.0 → 5, top (0, 9.0 → 5.0, 8.8) [D], grounded targets only; id 1 r 4.0 → 5, top (0, 12.0 → 5.0, 8.8), aerial targets only. Jostling off 7-25. "Speed multiplier per frame upon activation (ground) 0.18" [R] (read: on switching to the hit, the dash speed is multiplied by 0.18 [D]) |
| Grounded hit | **damage-based armor 1-5, threshold 10%** (12% in one-on-one, i.e. after the 1.2 multiplier); clean 5, late 6-7 | **28** | clean id 0: 10%, **85**, BK 82, KS 30, r 6.0 → 8, handl (0, -2.0, -1.0) [E]; clean id 1: same, r 5.5 → 7, top (0, 9.0, 5.5 to 12.5) [D]; late id 0: same, r 4.5 → 6, handl [E]. All: **+3 hitstun**, H 1.5, shield +4 (140 [D]), no clank (clean), angle type 3. Jostling off 1-7 |
| Aerial dash | search 19-35 | animation 79, then **helpless** | search box r 4.1 → 5, top (0, 12.0 → 3.0, 11.0) [D], everyone. **Ledge grabs from both sides from frame 26**. Jostling off throughout. Landing during the dash: **landing lag 22** (ssbwiki.com landing table: interruptible 23) |
| Aerial hit | clean 5, late 6 | **45** | clean id 0: 10%, **270** (meteor), BK 60, KS 70, r 8.0 → 10, arml (8.0, 0, 0.4) [E], **aerial targets only** (top priority since 8.0.0); id 1: 10%, 60, BK 60, KS 80, same box, **grounded targets only**; id 2: 10%, 60, BK 60, KS 80, r 2.5 → 3, arml (-2.0 to -5.0) [E]. Late: ids 0 / 1 as clean; id 2 r 4.0 → 5, top (0, 8.0 → 1.0, 5.5 → 3.0) [D]. All H 2.0, shield +2 (120 [D]), angle type 3, rebound off. **On frame 6, a set momentum of 1.5 backward and 2.5 upward** (reference units a frame) whether or not anything is hit; **no ledge grab and no landing before frame 8**. **Not helpless afterwards** |
| Helpless after an aerial miss | | | air speed x0.7; **special-fall landing lag 22** |

**Angle types** [R] (raw wikitext): every uppercut and overhand row is `af=3` (forward); the grounded search boxes are `af=3` too and
the aerial one has none (they launch nobody). The late uppercut box has no `clang=f` (it clanks); the clean ones do not clank.

The search box "does not possess a damaging hitbox unless it connects" and the move has "more ending lag when it connects with an
opponent or a shield" (ssbwiki.com move article): **the search also triggers on a shielding target** [R]. The 3.1.0 frame speed multiplier
(0.8x on frames 1-7) is already inside the published frame numbers.

**Not published**: the dash's speed and distance (animation root motion), grounded and aerial [gap: footage].

### Falcon Dive (up special) [R]

A rising command grab with no damaging hitbox of its own.

| Part | Frames | FAF | Data |
| --- | --- | --- | --- |
| Rise | early grab 14, late grab 15-30 | animation 66, then **helpless** | early: id 0 r 5.0 → 6, top (0, 13.0, 7.0) [D]; id 1 r 5.5 → 7, top (0, 8.8, 13.7) [D], **grounded targets only**. Late: id 0 r 5.0 → 6, top (0, 15.0, 6.0); id 1 r 5.0 → 6, top (0, 11.0, 6.0). Catches **airborne and grounded** targets, through shields (out of shield use), and fighters in hitstun (up air into Falcon Dive is a listed combo) |
| | | | **ledge grab from the front from frame 20, from behind from 37**; fast fall allowed from frame 50; **B-reverse: stick held back on frame 14** (sensitivity 0.25), and only then |
| | | | movement: horizontal acceleration x1.1 (ground) / **x6 (air)**, air speed x0.95, gravity after detaching 0.103 (the body's is 0.12). The x6 air acceleration before frame 14 is the "Tokyo drift" technique (forward or back input before the grab comes out greatly changes the horizontal reach) |
| Helpless after a miss | | | air speed x0.7; **special-fall landing lag 24** (ssbwiki.com landing table: interruptible 25; UFD and dragdown 24) |
| Catch (after a grab) | hit 1 on 2-3, throw release 17 | **40** (animation 71) | hit 1: 5%, 361, BK 50, KS 100, r **9.6 → 12** on `rot` (the body's centre) [D], flame: it damages the held fighter and **launches bystanders** too (ssbwiki.com move article: in this game "a successful grab now also deals knockback to bystanders"). Throw: **13%, 50, BK 85, KS 62**. Interrupted-throw row (kind 1): 8%, 0, BK 100, KS 10 (not modelled). **Ledge grab from the front from frame 61** of the catch. Afterwards Falcon "flips backward" and is **not helpless**: he can act, jump if he still has a jump, and use Falcon Dive again |

**Angle types** [R] (raw wikitext): the explosion (hit 1) is `af=3` (forward); the grab boxes and the throw rows have none (a grab and a
throw launch from the throw's own angle).

The rise itself and the backward flip after a catch are the animation's root motion: **not published** [gap: footage]. Where the held
fighter is carried during the catch is not published [E: at the fist, about (0, 13, 7) on `top`].

### Falcon Kick (down special) [R]

| Part | Frames | FAF | Data |
| --- | --- | --- | --- |
| Grounded | clean 13-16, mid 17-23, late 24-33 | **69** on the ground (animation 83), **66** if it goes off an edge (animation 66) | clean: 15%, **52**, BK 60, KS 88, r 3.8 → 5; mid: 11%, 52, BK 90, KS 60, r 3.2 → 4; late: 7%, **46**, BK 80, KS 65, r 3.2 → 4; all along legr (10.0 to 2.0) [E], H 1.0, flame. Jostling off 13-33. **Crashes into walls on 14-36** (a 53-frame wall-crash animation). On hit: speed x0.8 and "maximum speed decreases on hit 4"; friction on ending 0.12 |
| Aerial (a diagonal dive) | clean 14-18, mid 19-24, late 25-27 | **57** | clean: 15%, 361, BK 80, KS 57, r 5.76 → 7, kneel (4.0) [E], H 1.2; mid: 12%, 361, BK 80, KS 46, r 5.0 → 6, kneel (4.0 to -2.0), H 1.0; late: 9%, same as mid. Angle type 3. Not helpless afterwards |
| Landing (aerial version touching down) | hits 1-2 | **36** (landing lag 35) | 9%, 60, BK 80, KS 35, r 4.5 → 6, top (0, 2.0, 7.2 to -7.2) [D]: both sides |

**Angle types** [R] (raw wikitext): the grounded rows and the landing row have no `af` (0, by position); the aerial rows are `af=3`
(forward).

UFD: "Total frames is 68 on the ground or 65 if you go over an edge" (FAF 69 / 66), aerial 56 (57), landing lag 35 with the landing hit
on frame 1; dragdown agrees on every frame. **Not published**: the slide's and the dive's speed and distance (root motion) [gap].

## Conflicts and how they are resolved

1. **Falcon Kick (grounded) angle**: ssbwiki.com's named-field table gives **52 / 52 / 46** (clean / mid / late); dragdown gives "68/65".
   Resolved to ssbwiki.com (named template fields, and its BK / KS match dragdown's exactly, so dragdown's angle column is the odd one out).
   A third source or footage would settle it.
2. **Raptor Boost armor window**: ssbwiki.com "frames 1-5"; UFD "frames 1-4" (in its note), dragdown "armor 1-4". Two to one for **1-4**;
   ssbwiki.com's move article also says "1-5". Plausibly the armor is set on frame 1 and removed on frame 5, which one source counts
   inclusively. **Unresolved**: use 1-4 until footage (the clean hit is on 5, so the difference only matters for a trade on frame 5).
3. **Falcon Dive's catch FAF**: ssbwiki.com's catch timing says interruptible **40** (animation 71); dragdown's catch total 39 (FAF 40);
   the 11.0.0 patch note says "FAF 55 → 24"; UFD says "54 frames endlag when successful". Resolved to **40 from the start of the catch**:
   the patch note's 24 is counted from the throw release (17 + 23 = 40) [D], and UFD's 54 is the pre-11.0.0 value (FAF 55 - 1).
4. **Down throw FAF**: ssbwiki.com 40, UFD 39 (FAF 40), dragdown 33 (FAF 34). Resolved to **40**.
5. **Face-up get-up attack angle**: ssbwiki.com 48, dragdown 361. ssbwiki.com used (the other two get-up attacks' rows agree between
   the sources). Supporting it (sim 73): ssbwiki.com's raw tables give 48 for the swordfighter's, Wolf's and Sheik's get-up attacks too,
   which the older ports had written as 361 (fixed in sim 73, owner decision 27).
6. **Falcon Dive throw values**: dragdown lists the throw as 8%, 0 degrees, BK 100, KS 10, which is ssbwiki.com's *interrupted* row
   (kind 1); the real throw is 13%, 50, BK 85, KS 62 (ssbwiki.com, and the moveset summary's "13% (throw)"). Resolved to ssbwiki.com.
7. **Raptor Boost aerial hit landing lag**: dragdown alone gives 21. ssbwiki.com only says no landing before frame 8. **Unresolved** [gap].
8. **Rapid jab hit frames**: ssbwiki.com 5, 7 ... 19 within the loop; dragdown 3, 5 ... 15. A frame-origin difference; ssbwiki.com's quickest
   loop table (15 ... 29 from jab 1) is consistent with its own numbering, so it is used.
9. **Neutral air dodge**: UFD 42 frames for this character against 52 for Mario on the same site. **Resolved (sim 73)**: ssbwiki.com's
   air dodge table agrees on 42 (and on every directional figure), so 42 stands [R].

## Fitted distances [E] (built in sims 68-71)

None of the specials' travel is published (gap 1), and no published distance or height was found to fit them to (owner decision 12), so
they are estimates chosen to look and reach about right, in reference units (world units x 8):

| Move | Fitted motion | Roughly |
| --- | --- | --- |
| Side special, grounded dash | 0.8 a frame on frames 1-9, 2.0 on 10-25, 0.6 on 26-30 (the uppercut then cuts the speed to x0.18 [R]) | 42 units forward |
| Side special, aerial dash | 0.5 forward on 1-9, then 1.8 forward and 0.3 down on 10-35 | 51 forward, 8 down |
| Up special, the rise | from frame 14 the speed built by the x6 acceleration [R] plus 0.6 forward; upward 3.4 a frame slowing evenly to 0.4 on 31; then gravity 0.103 [R] | 34 up, 11 forward (plus the drift) |
| Up special, the backflip after the throw | held still to frame 17, then 0.8 back and 1.5 up slowing by 0.16 a frame to 24 | 6 back, 7 up |
| Neutral special, the aerial boost | 1.0 a frame along 35 degrees up or down (the angle and the stick range are [R]) | (gap 2) |
| Down special, the slide | 0.4 a frame to 12, 2.4 on 13-33, easing to a stop by 45; x0.8 once it hits [R] | 69 units |
| Down special, the dive | 0.3 forward on 1-13, then 2.1 forward and 2.1 down on 14-27, then the body's air physics | 33 forward, 29 down |
| Down special, the wall crash | pushed 0.4 a frame back for 6 frames, then still | 2.4 back |

Their code: `sim-core/src/moves_gauntlets.rs` (`motion` blocks) and `sim-core/src/scripts/gauntlets_*.script`. Footage would replace them.

## Gaps (needed for 1:1, not published)

1. Root motion: the Raptor Boost dash (ground and air), the Falcon Dive rise and the post-catch flip, the Falcon Kick slide and dive,
   the Falcon Punch's grounded slide. Needed as measured distances (the same request as Mario's question 2).
2. The aerial Falcon Punch's vertical boost speed (the angle, 35 degrees, and the stick range are published).
3. What the aerial Falcon Punch does on landing (keeps going as the grounded version, or ends in landing lag).
4. Whether aerial Raptor Boost and aerial Falcon Kick can be used more than once per airtime (nothing on the pages read says they are
   limited).
5. Whether the Falcon Dive catch continues if Falcon reaches the ground mid-catch, and whether it can catch a ledge-hanging fighter
   once their ledge intangibility is over.
6. Whether Raptor Boost's search ignores intangible targets.
