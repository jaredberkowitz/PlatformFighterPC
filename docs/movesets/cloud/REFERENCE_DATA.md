# Reference data for the seventh moveset (the reference game's Cloud)

Mechanical reference for the seventh port, collected 2026-10-13 for version **13.0.1**, before anything is built. Proposed in-game
names (not registered; see `PROJECT_AUDIT.md`): the **Greatsword** class and weapon on the **vanguard** body, built-in fighter
**Vanguard**, with a **Surge** gauge for the reference's Limit Gauge. The reference character's name is not used in the game. What the
engine can and cannot do with this data is in `PROJECT_AUDIT.md`; the format follows `docs/movesets/palutena/REFERENCE_DATA.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Cloud_(SSBU)/<move>` (27 pages: `Neutral_attack` and its three hits, the three tilts, `Dash_attack`,
  the three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(back|front|trip)`,
  `Neutral_special`, `Up_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), so every hitbox value
  comes from a named template field (`damage`, `angle`, `af` angle type, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff` hitlag
  multiplier, `sdi`, `sd` shield damage, `rehit`, `setweight`, `flinchless`, `g=f` / `a=f`, `clang`, `hitbits`, `trip`, `part`) and the
  angle-361 column shift cannot happen. The specials' pages also give the move **parameters** (speeds, multipliers, lifetimes, the gauge's
  rates). **There is no `Side_special` page**: ssbwiki.com has no hitbox table for Cross Slash (checked with the wiki's page list and a
  full-text search), so Cross Slash comes from dragdown.wiki and ultimateframedata.com only. https://www.ssbwiki.com/Cloud_(SSBU)
- **ssbwiki.com articles** `Cloud_(SSBU)` (attributes, moveset summary, update history), `Limit_(status)` (the gauge, the Limit Break
  stat table), `Limit_Charge_/_Finishing_Touch`, `Blade_Beam`, `Cross_Slash`, `Climhazzard`, `Dash` (current-game table),
  `Autolink_angle` (what angle 366 does), and `Template:UpdateList_(SSBU)/<version>` for 2.0.0, 3.0.0, 3.1.0, 7.0.0, 10.1.0, 13.0.1.
- **ultimateframedata.com/cloud** (UFD: startup, total frames, landing lag, autocancel, active frames, notes; dodges, jump air times,
  stats including the Limit values). UFD's "total frames" is FAF - 1. https://ultimateframedata.com/cloud
- **dragdown.wiki `SSBU/Cloud/Data`** (the parameter file and a second frame-data table, the only per-hit Cross Slash data found).
  https://dragdown.wiki/wiki/SSBU/Cloud/Data
- **kuroganehammer.com**: `kuroganehammer.com/Ultimate/Cloud` returns **404** (2026-10-13, also with other spellings); not used. (A
  search confirms the site's data for this game was superseded by UFD.)

**Update history applying to 13.0.1** (ssbwiki.com update lists, every entry for this character): **2.0.0** ledges caught sooner after
the forward air (no number; the move page says ledges from both sides from frame 78); Finishing Touch's early box reaches lower (Y
20-16 → 20-11, it now hits fighters lying down). **3.0.0** Blade Beam shield damage 0 → -4 / -3 (grounded clean / late), -3.2 / -2.4
(aerial). **3.1.0** down smash hit 1's farthest box FKV 120 → 85, angles 165 / 176 → 150 / 155. **7.0.0** shield size up (shields are
global here: not modelled); dash attack KS 60 → **70**; forward smash hit 2's set knockback 90 → **80** on one box; up smash startup
15 → **12**, FAF 49 → **46**, its 13% boxes KS 96 → **105**; **Climhazzard's rising hit snaps to the ledge earlier, frame 28 → 24**;
**Limit Charge 0.25 → 0.3 units a frame** (full charge 400 → 335 frames). **10.1.0** a cosmetic alternate super move only. **13.0.1**
**Finishing Touch startup grounded / aerial 14 / 16 → 12 / 14, FAF 87 / 90 → 85 / 88**. Every table below is the post-13.0.1 data.
(The up smash page files its change under 3.0.0; the update list puts it in 7.0.0. Either way it predates 13.0.1.) No data from the
earlier game was used; where the earlier game's behaviour differs (the Limit Break lasting until a KO, Limit Charge filling on a hit,
the instant Limit cancel, Finishing Touch ignoring weight, teching at Climhazzard's peak) it is called out.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift, a reading of a field);
**[E]** estimate. Hitbox **positions** on a bone other than `top` (`haver`, the sword's grip; `kneer`, `footl`, `footr`) are [E]: the
reference attaches them to its own skeleton, so ours go on our body where that bone is in the move's pose. Offsets on `top` (the
character's root at the feet: `zpos` forward, `ypos` up) convert directly [D]. A reference capsule (`zpos=16.0 to 6.5`) is two or three
circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first, FAF as published: ssbwiki.com's "Interruptible" is the FAF); the sim stores
  them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius and position in world tenths = reference value x 1.25, rounded half up [D] (the house
  rule). Speeds: reference units a frame / 8 = world units a frame [D].
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; **361** is the reference's 361 angle ("angle 361"), **367**
  the autolink angle, **366** the reference's "vortex" autolink (not built: see the audit, E28). H = hitlag multiplier (`ff`), SDI = SDI
  multiplier.
- **Angle type** (`af`): unset = 0 (by position, away from the hitbox's owner), 1 by travel, 3 forward (the owner's facing), 4 backward.
- **Targets**: `a=f` cannot hit aerial targets (grounded only), `g=f` cannot hit grounded targets (aerial only) [R].
- **Shield damage**: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd` becomes
  `100 x (damage + sd) / damage`, rounded [D].
- `clang=f rebound=f`: never clanks, never rebounds (E3c, `Hitbox::clanks`) [R]. `flinchless=t`: the reference's windbox, ours
  `Hitbox::push` (E2) [D]. `setweight=t`: set weight (`Hitbox::set_weight`) [R].

## Attributes [R]

ssbwiki.com attributes table, agreeing with UFD and the parameter file (dragdown):

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 100 | 1.155 | 2.145 | 2.167 | 0.106 | 1.155 | 0.01 + 0.06 | 0.009 | 0.098 | 1.68 / 2.688 | 32.5 / 17.0 / 32.5 | 3 | 1 |

Parameter file (dragdown) [R]: `walk_accel_add` **0.0**, `walk_accel_mul` **0.1575** (ours: `walk_accel` 0, `walk_accel_stick`
0.1575); `run_accel_add` 0.011, `run_accel_mul` 0.099 (ours: `run_accel` 0.011, `run_accel_stick` 0.099; ssbwiki.com's Dash table
agrees: base 0.011, additional 0.099, max 0.11); `ground_brake` 0.106; initial dash **10 frames**, pivot dash 12 (ssbwiki.com Dash
table); `jump_initial_y` 17.875; `jump_speed_x` 0.95, `jump_speed_x_mul` 0.9, `jump_speed_x_max` 1.2, `jump_aerial_speed_x_mul` 1.0;
`air_brake_y` 0.009; `air_ground_speed_brake` 0.027; `landing_frame_light` 1, `landing_frame` 3, **`landing_heavy_frame` 4** (so **3**
frames of heavy landing lag here [D], by the house rule `landing_heavy_frame - 1`; ssbwiki.com's attributes template also gives 4);
`landing_attack_air_frame` n / f / b / hi / lw **9 / 11 / 8 / 9 / 16** (agreeing with UFD and with ssbwiki.com's landing timings, which
give interruptible 10 / 12 / 9 / 10 / 17); `cliff_jump_y` **36.25**, `cliff_jump_x_speed` 0.6 (the ledge jump); `jump_count_max` 2;
`scale` 0.98; `shield_radius` 12.9 (shields are global here, not used); `jostle_front` 1.8, `jostle_back` 0.5 (no jostling in the engine);
**a wall jump** (`wall_jump_type` True, `attach_wall_type` False; ssbwiki.com's moveset summary: "Cloud can wall jump"), with the
wall-tech and wall jump numbers `passive_wall_jump_x_speed` **1.3**, `passive_wall_jump_y_speed` **2.4**, `wall_jump_enable_x_speed`
0.2, `passive_wall_x_speed` 0.5 (ours: `wall_jump` 1, `wall_jump_vx` 1.3, `wall_jump_vy` 2.4 [D], as the other wall-jumping bodies).
Footstool (`tread_*`) not modelled for anyone. `damage_fly_top_air_accel_y` 0.0756 / `damage_fly_top_speed_y_stable` 1.8 (the
reference's launch-fall values; the engine's shared launch fall covers them).

Jump air times (UFD) [R]: short hop **37**, full hop **49**, short hop fast fall **26**, full hop fast fall **34**.

Dodges (UFD total frames, so FAF one more) [R]: spot dodge "20/25" (FAF 26, intangible 3-17), forward roll 29 (FAF 30, 4-15), back roll
34 (FAF 35, 5-16): the shared values. Neutral air dodge **47** (intangible 3-30, landing lag 10); directional down **64**, diagonal down
**72**, sideways **79**, diagonal up **94**, up **104** (intangible 3-21; landing lag 11-19). The body takes UFD's numbers as every other
body does (the engine's dodge lengths count as UFD's); the engine has three directional lengths (down / side / up: 64 / 79 / 104) and
no separate diagonals, as for the other bodies [D].

Body size [E]: "a middleweight swordsman" (ssbwiki.com), model scale 0.98; the up air's top box is 26 reference units up and the
forward air's sweetspot 14.5 forward at 7 up; our vanguard body is the shared height, fitted to those reaches.

### The Limit gauge (the reference's "Limit"; ours "Surge") [R]

Sources: ssbwiki.com `Limit_(status)` and `Limit_Charge_/_Finishing_Touch` (text and the parameter table), `Cloud_(SSBU)` moveset
summary, UFD's Limit Charge row and stats.

| Property | Value | Source / label |
| --- | --- | --- |
| Gauge size | **100 units** | ssbwiki.com, UFD ("a 100 unit cap") [R] |
| Gain from damage dealt | **0.667 units per percent** ("Gauge fill per damage inflicted 0.667"); 150% dealt fills it | parameter table [R] |
| Gain from damage taken | **1 unit per percent** ("Gauge fill per damage taken 1"); 100% taken fills it | parameter table [R] |
| What does not count | damage to **shields**, with items, to non-player targets | ssbwiki.com [R] |
| Gain from Limit Charge | **0.3 units a frame** since 7.0.0 (0.25 before): 334 frames from empty (ssbwiki.com "334", the moveset summary and the patch note "335") | parameter table, UFD [R]; 100 / 0.3 = 333.3, so the 334th frame fills it [D] |
| Hit while charging | taking damage **during Limit Charge or its cancel lag** **takes away** the units it would have added (1 a percent) instead of adding them; **grabbed** out of either, every later pummel, throw and collateral hit drains too, until released (not for command grabs, except one other fighter's) | ssbwiki.com [R] |
| Full gauge | **Limit Break**: the stat multipliers below, and the **next special used is its Limit version** (Limit Charge becomes Finishing Touch) | [R] |
| Spending | using **any** special in Limit Break empties the gauge and ends the status, "even if the move misses or is interrupted" | moveset summary [R] |
| Expiry | **900 frames** ("Limit Break duration 900 frames"; "15 seconds") after it fills, if no special is used, the gauge empties; no visual countdown | parameter table [R] |
| Stock loss | **the gauge (any partial charge too) and an active Limit Break reset to empty** | **owner decision 9**. Evidence: ssbwiki.com says the earlier game's Limit Break lasted "until Cloud expends the gauge or gets KO'd", and the 13.0.1 game "also" (in addition) added the expiry; no source gives the partial gauge's fate on a KO [R, owner] |
| Gauge display | above the percent at all times, and above the fighter while charging (60 frames after a cancel) | presentation [R] |

**Limit Break stat multipliers** (ssbwiki.com `Limit_(status)` table, UFD's Limit values, ssbwiki.com Dash table) [R]:

| Stat | Base | Multiplier | In Limit Break |
| --- | --- | --- | --- |
| Walk acceleration | 0.1575 | x1.15 | 0.181125 |
| Walk speed | 1.155 | x1.15 | 1.32825 |
| Initial dash speed | 2.145 | x1.1 | 2.3595 |
| Run acceleration (base / additional) | 0.011 / 0.099 | x1.1 | 0.0121 / 0.1089 (max 0.121) |
| Run speed | 2.167 | x1.1 | 2.3837 |
| Traction | 0.106 | x1.15 | 0.1219 |
| Air acceleration (total) | 0.07 | x1.2 | 0.084 (the table gives the total; x1.2 on both parts [D]) |
| Air speed | 1.155 | x1.2 | 1.386 |
| Air friction | 0.009 | x1.15 | 0.01035 |
| Gravity | 0.098 | x1.1 | 0.1078 |
| Fall speed | 1.68 | x1.1 | 1.848 |
| Fast fall speed | 2.688 | x1.1 | 2.9568 (UFD; not in ssbwiki.com's table) |

The jumps' launch speeds do not change: with the higher gravity "his jumps [are] slightly lower" (ssbwiki.com attributes text) [R
reading]. So the multiplier applies to gravity only, never to the jump heights or initial speeds.

## Normals

All frame numbers [R] (ssbwiki.com timing tables; UFD and dragdown agree on every startup and FAF unless listed under "Conflicts").
Damage, angle, BK, KS, FKV, H, SDI [R]. Radius column: reference → world tenths [D]. Positions in reference units on the named bone.

### Jab [R]

Three hits: two kicks and a sword slash (8% total). Quickest loop: jab 1 on 4-5, jab 2 on 12-13, jab 3 on 21-22; FAF 30 / 40 / 53 (each
from the start of the chain).

| Part | Frames | Continue | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Jab 1 | 4-5 | **8-30** (consecutive jab 1 from 16) | 30 | 0 / 1 | 2.5 | 361 (3) | 35 | 25 | 2.0 → 3 | top (10.0, 7.0 / 10.5) [D] | 1.5 | |
| | | | | 2 | 2.5 | **180** (3) | 20 | 15 | 2.3 → 3 | top (10.0, 14.0) [D] | 1.5 | fighters only: pulls the target in |
| | | | | 3 | 2.5 | 361 (3) | 20 | 15 | 2.3 → 3 | top (10.0, 14.0) [D] | 1.5 | the same box for objects (id 2 wins on fighters); not modelled |
| Jab 2 | 5-6 | **9-30** | 33 | 0 / 1 | 2.0 | 361 (3) | 35 | 25 | 3.0 → 4 | top (9.5, 3.0 / 7.0) [D] | 1.2 | |
| | | | | 2 | 2.0 | 361 (3) | 25 | 20 | 3.5 → 4 | top (9.5, 11.0) [D] | 1.2 | |
| Jab 3 | 6-7 | | 38 | 0 | 3.5 | 30 (3) | 75 | 50 | 4.5 → 6 | top (9.0, 16.0 to 8.0) [D] | 2.0 | the sword |

The continuable windows are jab 1's frames 8-30 and jab 2's 9-30 (attack pressed in them carries on at once from the window's first
frame: the Embers' `next_from`) [R]. "Consecutive jab frame 16": jab 1 repeated (attack pressed again with no follow-up) starts again
from its frame 16 [R; meaning per ssbwiki.com's frame strip].

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt | 9-10 | 36 | 0 | 11 | 361 (3) | 20 | 100 | 6.0 → 8 | top (9.0, 14.5 to 7.0) [D] | H 1.0 |
| Up tilt, early | 6-8 | 33 | 0 / 1 / 2 | 8 | 95 (0) | 46 | 100 | 5.6 / 5.0 / 6.0 → 7 / 6 / 8 | haver (12.0 / 8.5 / 3.0) [E] | the sword sweeping from behind over the head |
| | | | 3 | 8 | 95 (0) | 46 | 100 | 4.5 → 6 | top (21.0 to 16.0, 0) [D] | above the head, early only |
| Up tilt, late | 9 | | 0 / 1 / 2 | 8 | 95 (0) | 46 | 100 | as early | haver [E] | |
| Down tilt, early | 7-8 | 41 | 0 | **7** | 80 (3) | 80 | 75 | 3.6 → 5 | kneer (x 5.0 to 1.5, y -1.0, z 1.5) [E] | the leg; trip chance 20 (cannot happen at an 80-degree launch: ssbwiki.com trivia) |
| | | | 1 | **6** | 80 (3) | 80 | 75 | 2.5 → 3 | top (2.5, -2.6) [D] | the body |
| Down tilt, late | 9-17 | | 0 | 7 | 80 (3) | 80 | 75 | 3.6 → 5 | kneer [E] | **left (back) leg intangible 7-17** (partial: E19) |
| Dash attack, clean | 9-13 | 46 | 0 / 1 / 2 | 11 | 60 (3) | 100 | **70** (7.0.0) | 5.0 / 4.0 / 3.0 → 6 / 5 / 4 | haver (14.0 / 8.0) [E], top (8.0, 3.0) [D] | shield +1 (109 [D]) |
| Dash attack, late | 14-18 | | 0 / 1 / 2 | 8 | 65 (3) | 100 | 50 | 4.0 / 3.3 / 2.8 → 5 / 4 / 4 | as clean | shield +1 (113 [D]) |

The down tilt is a slide forward (the reference's root motion; distance not published [E]). Up tilt boxes have no angle type (0, by
position): a target behind is sent backward [D].

### Smashes [R]

| Move | Frames | Charge hold | FAF | id | Dmg | Angle (type) | BK / FKV | KS | Radius | Bone (y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash hit 1 | **19** | 5 (charges 5-6) | 67 | 0 | 3 | 60 (3) | FKV 25 (BK 30) | 30 | 4.5 → 6 | top (9.0, 10.0 to 6.5) [D] | **grounded only**; set weight; H 1.5; SDI 0 |
| | | | | 1 | 3 | **170** (3) | FKV 10 | 100 | 4.5 → 6 | top (9.0, 16.0 to 6.5) [D] | grounded only; set weight; pulls in |
| | | | | 2 | 3 | 0 (3) | FKV 10 | 100 | 4.5 → 6 | top (9.0, 10.0 to 6.5) [D] | **aerial only**; set weight |
| | | | | 3 | 3 | 180 (3) | FKV 10 | 100 | 4.5 → 6 | top (9.0, 16.0 to 6.5) [D] | aerial only; set weight |
| Forward smash hit 2 | **24** | | | 0 | 3 | 15 (3) | FKV **80** (7.0.0) | 100 | 2.5 → 3 | top (9.0, 5.0) [D] | set weight; H 1.5; SDI 0.5 |
| | | | | 1 | 2 | 15 (3) | FKV 50 | 100 | 5.2 → 7 | top (9.0, 10.0 to 6.5) [D] | set weight |
| | | | | 2 | 4 | 170 (3) | FKV 10 | 100 | 5.2 → 7 | top (9.0, 16.0 to 6.5) [D] | set weight (the tip pulls back in) |
| Forward smash hit 3 | **28** | | | 0 | **13** | 361 (3) | 42 | 115 | 5.5 → 7 | top (9.0, 10.0 to 6.5) [D] | H 1.8 |
| | | | | 1 | 12 | 361 (3) | 42 | 115 | 5.5 → 7 | top (9.0, 16.0 to 6.5) [D] | the tip, slightly weaker |
| Up smash, clean | **12** | 6 (charges 6-7; conflict 3) | 46 | 1 / 2 | 13 | 83 (3) | 40 | **105** (7.0.0) | 4.0 → 5 | haver (9.0 / 13.0) [E] | H 1.2 |
| Up smash, mid | 13-14 | | | 0 | 12 | 83 (3) | 40 | 96 | 3.3 → 4 | haver (2.5) [E] | the hilt; lower id, so it wins where both reach |
| | | | | 1 / 2 | 13 | 83 (3) | 40 | 105 | 4.0 → 5 | haver (9.0 / 13.0) [E] | |
| | | | | 3 | 11 | 83 (3) | 40 | 90 | 3.5 → 4 | top (15.0 to 8.0, -5.0) [D] | behind |
| Up smash, late | 15-16 | | | 0 / 1 / 2 | 8 | 60 (0) | 32 | 100 | 3.0 → 4 | haver (2.5 / 7.0 / 11.0) [E] | H 1.0 |
| Down smash hit 1 | 8-9 | 2 (charges 2-3) | 50 | 0 / 1 | 3 | 170 / **155** (3) | FKV 90 / **85** (3.1.0) | 97 | 2.5 → 3 | top (4.0, 8.0 to 5.0 / 13.5 to 5.0) [D] | **aerial only**; set weight; SDI 0.5 |
| | | | | 2 / 3 | 3 | 160 / **150** (3) | FKV 90 / 85 | 97 | 3.5 → 4 | as 0 / 1 | **grounded only**; set weight |
| Down smash hit 2 | 21-23 | | | 0 | 11 | 35 (**4: backward**) | 35 | 92 | 4.4 → 6 | top (4.8, -14.0 to -1.0) [D] | behind; H 1.4 |

Charged smash damage is the shared rule (up to 1.4x over 60 frames), agreeing with dragdown's charged values (4.5 / 3 / 6 / 19.5 / 18
forward, 19.5 / 18 / 16.5 / 12 up, 4.5 / 16.5 down) [R]. Every forward smash hit is a single frame: a well-timed roll avoids them all
(ssbwiki.com) [R].

### Aerials [R]

| Move | Frames | Autocancel | Landing lag | FAF | Phase / id | Dmg | Angle (type) | BK | KS | Radius | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Neutral air | 5-15 | 1, 31+ | **9** | 39 | 0 / 1 / 2 | 8 | 40 (0) | 25 | 100 | 3.8 → 5 | haver (x 3.0; y 12.0 / 8.0 / 4.0) [E]; autocancels from a short hop |
| **Forward air**, clean | 18-19 | 1-2, 43+ | **11** | 54 | 0 | **14** | **270** (3) | 20 | 90 | 3.3 → 4 | top (7.5 to 6.5, 14.5 to 14.0) [D]; **aerial only** (the meteor sweetspot); H 1.4 |
| | | | | | 1 | 14 | 60 (3) | 20 | 100 | 3.3 → 4 | same place; **grounded only** |
| | | | | | 2 | 13 | 45 (3) | 30 | 83 | 7.8 → 10 | top (9.5 to 14.5, 14.0 to 14.6) [D] |
| | | | | | 3 | 13 | 45 (3) | 30 | 83 | 4.0 → 5 | haver (12.0 to 4.0, z -2.0) [E] |
| Forward air, late | 20-25 | | | | 3 | 11 | 45 (3) | 30 | 83 | 3.0 → 4 | haver [E]; H 1.0. "Ledges from both sides from frame 78" (2.0.0) |
| Back air | 11 (early), 12 (late) | 1-4, 37+ | **8** | 43 | 0 | **13** | 361 (**4: backward**) | 20 | 95 | 7.0 → 9 | top (8.5, -15.0 to -7.0) on 11, top (16.0, same) on 12 [D] |
| **Up air**, clean early | **8** | 1, 32+ | **9** | 45 | 0 / 1 / 2 | **11** | 80 (0) | 42 | 80 | 4.2 → 5 | top (24.0 / 25.0 / 26.0 to 17.5; z 0 / 4.5 / 9.0) [D]: the blade flat overhead |
| Up air, clean late | 9 | | | | 0 / 1 / 2 | 11 | 80 (0) | 42 | 80 | 4.0 → 5 | haver (4.0 / 9.0 / 14.0) [E] |
| Up air, late | 10-23 | | | | 0 / 1 / 2 | **8** | 80 (0) | 42 | 80 | 3.0 → 4 | haver (4.0 / 9.0 / 14.0) [E]; H 1.0 |
| **Down air**, clean | 11-13 | 1-4, 46+ | **16** | 65 | 0 | **15** | **270** (0) | **10** | **100** | 3.6 → 5 | haver (11.0 to 12.5) [E]; **aerial only** (the meteor); H 1.5 |
| | | | | | 1 | 13 | 60 (0) | 30 | 105 | 4.5 → 6 | haver (7.0 to 13.0) [E]; H 1.2 |
| Down air, late | 14-39 | | | | 1 | 8 | 65 (0) | 80 | 55 | 3.5 → 4 | haver (9.0 to 11.0) [E]; H 1.2 |

Landing lag = `landing_attack_air_frame` (dragdown) = ssbwiki.com's landing "interruptible" - 1 = UFD's landing lag [D].

**The up air is a single hit, not a multi-hit** in 13.0.1: three clean boxes on frame 8 (on the root, over the head), the same strength
moved to the blade on 9, then a weaker late hit 10-23 (a "sex kick": strong early, lingering weak) [R]. There is no `rehit` and no
second hit group. (The task brief's "up air's multi-hit" does not match the reference data; recorded under Conflicts.)

**The down air's meteor** is the id 0 box straight down on 11-13, aerial targets only, 15%, angle 270, BK 10, KS 100; on the same
frames a grounded target (or one off the sweetspot) takes id 1, 13% at 60 degrees; from 14 to 39 the late 8% hit at 65 [R].

### Grabs, pummel and throws [R]

| Move | Frames | FAF | Data |
| --- | --- | --- | --- |
| Standing grab | 9-10 | 37 | id 0 r 3.3 → 4, top (6.6, 4.0 to 8.7) [D], grounded targets only; id 1 r 1.65 → 2, (6.6, 2.35 to 10.35), aerial only |
| Dash grab | 12-13 | 45 | id 0 r 2.6 → 3, top (6.6, 4.0 to 10.4); id 1 r 1.3 → 2, (6.6, 2.7 to 11.7) |
| Pivot grab | 13-14 | 40 | id 0 r 3.3 → 4, top (6.6, -4.0 to -14.7); id 1 r 1.65 → 2, (6.6, -2.35 to -16.35) (written in front: the engine turns a pivot grab round [D]) |
| Pummel | 1 (ours 2) | 7 (ours 8) | 1.3%, 361 (3), BK 0, KS 100, **FKV 30**, r 5.0 → 6, top (10.0, 8.0) [D], H **2.1**, no clank. The engine's pummel cannot hit on its first frame, so the hit and FAF are one frame later [D] (the house rule). UFD's "18 including 12 frames of hitlag" agrees |
| Forward throw | hit 7-8, release 9 | 36 | **hit**: 4%, 45 (3), BK 60, KS 120, r 4.0 → 5, footl [E], H 1.5, no clank; **throw**: 3%, 49, BK 48, KS 125. Thrower invincible 1-9 |
| Back throw | hit 13-14, turns round 15, release 15 | 32 | **hit**: 3%, 45 (**4**), BK 48, KS 90, r 4.0 → 5, footr [E], H 1.5, no clank; **throw**: 3%, 38, BK 48, KS 110. Invincible 1-15 |
| Up throw | hits 8-9, 13-14, release 15 | 51 | **hit 1**: 2%, 50 (3), BK 80, KS 150, r 4.0 → 5, footr [E]; **hit 2**: 2.5%, 50 (3), BK 80, KS 150, r 4.0 → 5, top (21.5, 7.5) [D], H 1.2; **throw**: 4%, 80, BK 78, KS 97. Invincible 1-15 |
| Down throw | release 18 | 44 | **throw**: 7%, **122**, BK 40, KS 110 (sends behind). **On frame 17, 10 frames of hitlag for both fighters** (the held fighter invincible meanwhile); the frames above are without that hitlag, so in real time the release is 28 and the FAF 54 [D]. Invincible 1-18 |

The forward, back and up throws hit the held fighter before the throw (the engine's hit-then-throw pattern: `throw_after_hit`, a
`HIT_PUMMEL` pre-hit, as the Claws' throws) [R]. Each also has the "interrupted" row (`kind=1`: 3%, 361, BK 60 or 40, KS 100), not
modelled (as for the other ports). The thrower's invincibility rows are not modelled (house limit).

### Ledge and floor [R]

| Move | Frames | Intangible | FAF | Data |
| --- | --- | --- | --- | --- |
| Ledge attack | 20-22 | 1-22 (1st grab; 1-18 2nd, 1-11 3rd, none from the 4th: the shared decay) | 56 | 10%, 45 (3), BK 90, KS 20, shield +1 (110 [D]), r 5.0 → 6, top (5.0, 17.0 to -1.0) [D], no clank |
| Get-up attack, face up (ssbwiki.com "back") | behind 20-21, front 24-25 | 1-25 | 46 | 7%, 48 (0), BK 80, KS 48, shield +8 (214 [D]), r 4.5 → 6; top (4.5, -18.5 to -5.5) then (4.5, 18.5 to 6.0) [D]; no clank |
| Get-up attack, face down (ssbwiki.com "front") | behind 20-21, front 24-25 | 1-25 | 46 | as face up; top (4.5, -18.5 to -6.5) then (4.5, 19.0 to 6.5) [D] |
| Get-up attack, trip | behind 20-21, front 24-25 | 1-7 | 50 | 5%, 361, BK 60, KS 50, shield +8 (260 [D]), r 4.5 → 6; top (4.5, -18.5 to -5.5) then (4.5, 19.0 to 6.0) [D] |

dragdown's "active 20-25" is the first to last active frame of the same two hits [D]. One get-up attack slot exists (as for the other
ports): the face-up one would be used.

## Specials

Every special has a **normal** and a **Limit** version; the Limit version is used instead when the gauge is full, and using it empties
the gauge. Limit Charge's Limit version is a different move, **Finishing Touch**.

### Blade Beam (neutral special) [R]

A slow, long-range projectile from a rising slash. The slash itself has no hitbox; a **flinchless body windbox** pushes grounded fighters
next to Cloud away on 16-17.

| Part | Frames | Data |
| --- | --- | --- |
| Body windbox | **16-17** | 0%, 361 (3), FKV **40**, BK 0, KS 100, r 3.0 → 4, top (6.5, 5.5 to 4.5) [D], **grounded fighters only**, flinchless (push), no hitlag, SDI 0, no clank |
| Beam, clean | **18-35** | grounded **8%** / aerial **6.4%**, 55 (**1: by travel**), BK 38, KS 60, H 0.5; shield **-4 / -3.2** (50% [D]); two circles r 3.0 → 4: top (11.0 to 5.0, -2.1 to 0) and (1.7 to 5.0, -0.8 to 0) on the beam [D]: a beam about 12 reference units tall; reflectable, absorbable |
| Beam, late | **36-67** (conflict 4) | grounded **6%** / aerial **4.8%**, same angle and knockback, r 2.7 → 3; shield -3 / -2.4 (50% [D]) |
| Projectile | spawned at **[12, 2]** (forward, up) [R] (→ 1.5, 0.25 world units [D]) on frame 17 [D: the fireball rule, so it hits from 18] | speed **2** grounded / **1.6** aerial reference units a frame (→ 0.25 / 0.2 world [D]); **lifetime 50** (→ 100 / 80 reference units of travel, 12.5 / 10 world units [D]); gone on its first hit; **follows the ground's contour, and carries on at the ground's angle past an edge** (ssbwiki.com; on this game's flat stages: straight on [D]) |
| End | FAF **54** grounded, **61** aerial (animation 70 / 82) | |
| Movement | | grounded speed **x0.2**, air speed **x0.4**, **vertical speed x0.65**, gravity **0.06** (a fall break, "he will stall slightly"); "slight horizontal manoeuvrability" in the air; **no ledge grab until the move ends** [R] |

Whether the grounded or aerial beam comes out is decided when it is fired: "if he lands before firing the beam, [it] counts as ... the
grounded version" [R].

**Limit Blade Beam** [R]:

| Part | Frames | Data |
| --- | --- | --- |
| Intangible | **10-17** (whole body) | |
| Beam | **16-62** (conflict 4) | grounded **6%** / aerial **4.8%**, 55 (1), BK 38, KS 60; two circles r **5.5 → 7**: top (13.5 to 5.0, -7.2 to -3.0) and (0.0 to 5.0, -5.7 to -3.0) [D] (about 19 units tall); **no clank** (transcendent); reflectable, absorbable. Speed **2.95** grounded / **2.4** aerial (→ 0.369 / 0.3 world [D]), lifetime **47** |
| Burst, loop | its frames **1-28**, **rehit every 6** (hits on 1, 7, 13, 19, 25 [D]) | grounded **2%** / aerial **1.6%**, angle **366** (3), FKV **40**, BK 0, KS 100, r **14.0 → 18**, at the beam's point (6.0 up) [D], H 0.5, no clank |
| Burst, final | its **29-30** | grounded **3%** / aerial **2.4%**, **48** (3), BK **50**, KS **226**, r 14.0 → 18, a separate hit (part 1), no clank |
| End | FAF **61** (animation 82; one timing for both versions) | movement as the normal version (the parameter page: "the Limit Break version shares the same values") |

How the two parts relate (ssbwiki.com `Blade_Beam`, `Limit_(status)`, moveset summary) [R]: the beam **stops at the first fighter it
hits** and **bursts there**; the burst's loop and final hit catch that fighter and everyone inside it. "**19% on the first target and 13%
on any targets immediately behind**": 6 + 5 x 2 + 3 = 19 and 5 x 2 + 3 = 13, exactly [D] (aerial 15 / 10: 4.8 + 5 x 1.6 + 2.4 = 15.2 and
10.4, rounded in the text [D]). **On a wall** the Limit beam does not vanish: it "acts as if hitting a player" (bursts there: a stage
spike under the ledge) [R]. Angle 366 pulls the loop's victims toward the burst's centre (ssbwiki.com `Autolink_angle`: it sends in the
attacker's direction of movement plus a pull to the hitbox centre for 5 frames, without reducing the launch speed; the burst does not
move, so it is the pull alone [D]). UFD's "first / multi / final" and "Multihits: 1-28 (rehit 6) / 29-30" agree.

### Cross Slash (side special) [R, partly]

A natural combo of three parts drawing a kanji: **hit 1**, then **hit 2** if special is pressed again **and hit 1 connected**, then
**part 3** (three automatic slashes: hits 3, 4, 5) if special is pressed again and hit 2 connected. "Cloud will only perform the first
slash repeatedly if the move whiffs regardless of what the player does, and if the opponent escapes the second hit, the third step
cannot be used" (ssbwiki.com `Cross_Slash`). A shielded hit counts as connecting: "if any part of the move hits a shield ... especially
after the final part" it is punishable, so the parts follow on shield too [R reading]. The animation differs by facing (it writes a real
character, so it is not mirrored): facing left it starts lower, facing right higher [R]; no numbers are published for the difference
[gap].

**No ssbwiki.com hitbox table exists for this move** (see Sources). Damage, angles and knockback below are dragdown's per-hit rows; the
frames are UFD's and dragdown's; **radii and positions are not published** [gap 1: E for every box].

| Part | Frames | Follow-up window | FAF | Dmg | Angle | BK / FKV | KS | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Hit 1 | **10-11** | from **10** on a hit (UFD: "transitions to next slash as early as frame 10 if you hit"); end [gap 2] | **41** | **4** | 75 / 112 / **366** (three ids) | 30 / 32 | 35 / 40 | dragdown, UFD |
| Hit 2 | its **2-3** | from its **14** on a hit (UFD) | its **32** | **3** | 95 / 60 | 33 / 43 | 25 | dragdown, UFD |
| Part 3, hit 3 | its **2-3** | | its **56** | **3** | **280** / 70 | FKV **20** | 100 | dragdown, UFD |
| Part 3, hit 4 | its **11-12** | | | **3** | 95 / 110 | FKV **37** | 100 | dragdown, UFD |
| Part 3, hit 5 | its **25-26** | | | **6** | **50** | 30 | **140** | dragdown, UFD |

Total 4 + 3 + 3 + 3 + 6 = **19%** [R: ssbwiki.com "19% total"]. "A small pause before the third segment" [R]: part 3's first hit is on
its frame 2, so the pause is hit 2's window opening on 14 [D]. Hit 1 "steps forward" on the ground [R]; the distance is not published
[E]. Its aerial behaviour (falling or held) is not published [gap 3].

**Limit Cross Slash**: all five hits in one move, **regardless of hitting** and with no further input; **intangible 6-11** (UFD,
"invulnerable"; ssbwiki.com: "intangibility to Cloud's body on startup"); FAF **61**; in the air "it halts all momentum until the move is
finished" [R]; high shield damage (the last hit "can sometimes break a middleweight's shield" after an aerial) [R, no number: gap 4].

| Hit | Frames (UFD) | Frames (dragdown) | Dmg | Angle | BK / FKV | KS |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 10-11 | 10-17 (hits 1-2 together) | **5** | **367** (autolink) | FKV **80** | 100 |
| 2 | 15-16 | | 5 (UFD) / 3 (ssbwiki.com) | 367 | FKV 80 | 100 |
| 3 | 21-22 | 21-23 | 3 | 280 / 70 | FKV **14 / 21** | 100 |
| 4 | **28** | 28-29 | 3 | 95 / 110 | FKV **26** | 100 |
| 5 | 38-39 | **39** | **10** | 50 | 30 | **147** |

The total is a conflict (conflict 1): ssbwiki.com's articles say **27%**, its moveset summary lists 5 + 3 + 3 + 3 + 10 = 24, UFD's rows give
5 + 5 + 3 + 3 + 10 = 26. "The final hit's hitbox also becoming slightly larger" [R, no number].

### Climhazzard (up special) [R]

**Part 1 (the rise)**: a stab, then a rising slash; at the top, special pressed again in the **continuable window** starts **part 2**,
the falling slash. Without the second press Cloud falls helpless.

| Part | Frames | id | Dmg | Angle (type) | BK / FKV | KS | Radius | Bone (y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Hit 1 (stab) | **7** | 0 | 3 | 40 (3) | FKV **80** | 100 | 2.0 → 3 | top (7.0, 3.0) [D] | **H 2.5, SDI 0** (every box of the move: "completely immune to SDI") |
| | | 1 | 3 | 78 (3) | FKV 80 | 100 | 5.0 → 6 | top (7.0, 5.0 to 6.5) [D] | |
| | | 2 | 3 | 102 (3) | FKV 80 | 100 | 3.5 → 4 | top (7.0, 15.0 to 6.5) [D] | pulls back toward the body |
| Hit 2, early | **10** | 0 / 1 | 4 | 94 / 96 (3) | FKV **121** | 100 | 5.0 → 6 | haver (6.0 / 14.5 to 6.0) [E] | **set weight**; H 1.2; a separate hit (part 1) |
| Hit 2, mid | **11** | 0 / 1 | 4 | 94 / 96 (3) | FKV 121 | 100 | 5.0 → 6 | haver [E] | set weight |
| | | 2 | 4 | 90 (3) | 98 | 40 | 3.5 → 4 | haver (14.5 to 6.0, z -10.0) [E] | set weight; behind |
| Hit 2, late | **12-15** | 0 | 4 | 85 (3) | 50 | 50 | 5.0 → 6 | haver [E] | set weight |
| | | 1 | 4 | 96 (3) | 50 | 50 | 3.5 → 4 | haver [E] | set weight |
| | | 2 | 4 | 90 (3) | 70 | 40 | 3.5 → 4 | haver (z -12.0) [E] | set weight; behind |

| Part 1 timing and rules | Data |
| --- | --- |
| Continuable (second press) | **20-26** [R] |
| Animation | 57 frames, then the special fall (dragdown "total 66" [conflict 5]) |
| Height | grounded height multiplier **1**, aerial **1.1** (parameters) [R]; the height itself is the animation's root motion, **not published** in units (ssbwiki.com: about the height of the top platform of the reference's standard three-platform stage) [E, owner decision 12: fit to that description] |
| Horizontal | speed **x0.6** on starting; "completely stops Cloud's forward momentum ... almost no horizontal momentum" [R] |
| Ledges | the rising hit **snaps to the ledge from frame 24** since 7.0.0 (28 before) [R]; "can only grab ledges during the initial thrust, or when [he] reaches its peak" (ssbwiki.com) [R, the thrust's window not published: gap 6] |
| Special fall | air speed **x0.6** (parameter "air speed multiplier during special fall") [R]; **landing lag 20** [R] |
| Earlier game only | the "cannot tech when hit at the peak" rule is the earlier game's only (ssbwiki.com) [R]: **not** in 13.0.1 |

**Part 2 (the descent)**:

| Part | Frames | Data |
| --- | --- | --- |
| Falling slash | its **12-61** (quickest loop: 31-44 when starting and landing at the same height) | **fall speed 4.2** reference units a frame (→ 0.525 world [D]), straight down: "any horizontal momentum is utterly cancelled" [R]. id 0: **4.5%, 262** (0), FKV **110**, KS 100, r 2.0 → 3, top (2.0, 10.4 to -1.0) [D], **aerial targets only** (the meteor); id 1: **4.5%, 45** (0), BK 50, KS 80, r 4.0 → 5, top (4.6 to 4.2, 9.4 to -1.0) [D] (anyone; "if it lands behind Cloud or anywhere not directly below the sword, it will instead send opponents at a diagonal angle"); H 1.3; **no clank** |
| Landing hit | landing frames **1-3** | **3.5%, 64** (0), BK **64**, KS **128**, r 8.0 → 10, top (8.0, 8.0 to 0) [D], H 1.3, no clank; **FAF 27** (landing lag **26** [D]; UFD 26; the parameter "landing lag frames 40" is the animation's length) |
| Not cancellable | "the descending slash cannot be canceled" [R] |
| Ledges | **not until he has descended "a distance slightly less than Climhazzard's normal vertical distance"** in front, and "slightly less than Climhazzard's Limit Break upward distance" for a ledge behind; at any lower height the ledge is missed (a self-destruct) [R, no numbers: gap 7] |
| Helpless | after 61 frames (a full second) of falling it becomes the ordinary helpless fall, at the same speed (ssbwiki.com trivia) [R] |

Total when every hit connects: 3 + 4 + 4.5 + 3.5 = **15%** ("15% base", ssbwiki.com `Up_special`) [R].

**The roster note's "descending meteor sweetspot only after a fall distance" is a misreading**: the meteor box is out from part 2's
frame 12; what waits for a fall distance is the **ledge grab** [R].

**Limit Climhazzard** (part 1 only; part 2 "can still be activated, but it does not have any enhancements") [R]:

| Part | Frames | id | Dmg | Angle (type) | BK / FKV | KS | Other |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Hit 1 | **7-9** | 0 / 1 / 2 | **6** | 40 / 78 / 102 (3) | FKV 80 | 100 | as normal, **H 2.5**, **no SDI multiplier** (normal SDI: "notably susceptible to SDI") |
| Hit 2, clean | **10** | 0 / 1 / 2 | **7** | 88 (3) | **50** | **142** | haver / top; H 2.5; a separate hit |
| Hit 2, late | **11-16** | 0 / 1 | 7 | 88 (3) | 50 | **110** | |
| | | 2 | 7 | 90 (3) | 98 | 40 | behind |

| Rule | Data |
| --- | --- |
| Intangible | **5-12** |
| Continuable | **33-39** (into the ordinary part 2) |
| Animation | 69 (dragdown total 69) |
| Height | grounded height multiplier **1.6**, aerial **1.85** (ssbwiki.com's text "1.5 times higher" is a rounded description) [R] |
| Horizontal | speed **x0.67** on starting; "significant horizontal manoeuvrability while rising", "able to carry Cloud's horizontal momentum as he falls" [R, the air control not published: gap 8] |
| Ledges | "allow[s] grabbing the ledge while it is in progress": **from any frame** [R] (both sides [E]) |
| Special fall | air speed **x0.8**, landing lag **20** [R] |

### Limit Charge (down special) [R]

| Part | Frames | Data |
| --- | --- | --- |
| Charging | held by itself (an auto-charge: it goes on until cancelled) | **0.3 units a frame** into the gauge; the gauge shown above Cloud; the frame charging starts on is **not published** [gap 9] |
| Cancel | **FAF 8** (7 frames of lag; animation 21) | by **special** again, **shield**, **roll** (the stick sideways), **spot dodge**, **jump**, **air dodge**, **grab** (ssbwiki.com moveset summary and `Limit_Charge` article); "even shielding and dodging out of Limit Charge have the same delay" [R]. UFD: "takes 7 frames to cancel" |
| Hit while charging | the hit's damage **drains** the gauge 1 unit a percent instead of filling it; the same during the cancel's 7 frames; grabbed, every pummel and throw drains until he is released (see the gauge table) [R] |
| Full gauge | the line "limits are meant to be broken" when filled by charging [R]; whether charging **stops by itself** when the gauge fills is **not published** [gap 10] |
| In the air | usable in the air (an air dodge cancels it) [R]; its air physics are **not published** [gap 11] |

The earlier game's "instant Limit cancel" (an attack straight out of the charge) **was removed** in 13.0.1 [R].

### Finishing Touch (down special in Limit Break) [R]

A whirlwind slash for 1% with huge knockback; the gauge is spent by attempting it. Since **13.0.1**: startup 12 grounded / 14 aerial, FAF
85 / 88.

| Part | Grounded frames | Aerial frames | Data |
| --- | --- | --- | --- |
| Early hit | (never comes out: "because of a frame speed multiplier, the early hit never actually comes out") | **14** | 1%, **80** (0), BK **60**, KS **431** grounded / **371** aerial, r 10.0 → 13, top (10.0, 13.0 to 9.0) [D], **H 2.0**, clanks |
| Mid hit | **12** | **15** | 1%, 80, BK 60, KS 431 / 371, r 8.0 → 10, top (20.0 to 11.0 (2.0.0), 9.0 to 10.0) [D] |
| Late hit | **13** | **17** | grounded: 1%, 80, BK 60, **KS 406**, r 7.0 → 9, top (22.0, -2.0 to -8.0) [D] (behind); aerial: KS **349**, r 8.0 → 10, top (16.0 to 10.0, -11.5 to -11.0) [D] |
| Latest hit (grounded only) | **14** | | 1%, 80, BK 60, KS 406, r 8.0 → 10, top (16.0 to 10.0, -11.5 to -11.0) [D] (behind) |
| Windbox | **22-26** | **25-29** | 0%, 361 (0: by position, both sides), **FKV 80**, BK 0, KS **100** grounded / **86** aerial, r 13.0 → 16, top (15.0, 16.0 to -16.0) [D], flinchless (push), no hitlag, no clank |
| End | FAF **85** | FAF **88** | animation 115 / 117 |
| Air movement | | | horizontal and vertical speed **x0.4** on starting, gravity **0.03**, max fall **1.0** ("briefly hover in place until the move ends") [R] |

One hit group across the phases (a target is hit once) [D]. Hits behind Cloud are weaker (KS 406 / 349: "KOing about 10% later"); the
aerial version weaker still [R]. **It no longer ignores weight** in 13.0.1 (no `setweight`) [R]. It can be shielded, countered and
armoured [R]. UFD: "12 (13-14) / 22-26 // 14-15 (17) / 25-29", FAF 85 / 88 (total 84 / 87) agree.

## As built in phase 2 (sim 85): fits and readings

* **Climhazzard's rise** [E, owner decision 12]: 5.5 reference units a frame on 8-15, 4.0 on 16-19, 1.5 on 20-23, still on 24 (60 by
  frame 19, 66 at the peak; x1.1 from the air), straight up. Fitted to ssbwiki.com's **quickest loop** (rows re-read 2026-10-13: the
  falling hit 31-44, the landing hit 45-47, interruptible 71, animation 84): the falling slash, falling 4.2 a frame from its frame 12
  (absolute 31 with the second press on 20), lands on 45 from 60 units up. The test `the_quickest_rise_and_fall_lands_on_45_and_is_free_on_71`
  reproduces 20 / 45 / 71 exactly. The falling slash holds still on its frames 1-11 [E].
* **Climhazzard part 2's ledge distances** [E, gap 7]: 56 units below its start for a ledge in front (just under the 60 rise), 92
  behind (just under the powered rise, 1.6 x 60 = 96).
* **Blade Beam's capsule** [D]: the two circles (1.7 to 11.0 up on the beam) are one standing capsule centred 6.35 up, 4.65 above
  and below; spawned 8.35 up (2 + 6.35) and 12 ahead; lifetime 51 here for the reference's 50 (it hits from 18 to 67, the house
  projectile rule); its slight slant (-2.1 / -0.8 units) is not modelled. The windbox is on both the grounded and the aerial beam
  [R reading]. Landing after the beam fired carries the rest of the move on the ground [E].
* **Cross Slash** [E, gap 1]: three / two / two boxes per hit along the blade in front, angle type 3; the 366 box is the 367 autolink
  until E28 (conflict 2); a press any time during a part counts once the part has connected (gap 2: the window runs to the part's
  end); the ground step 0.12 world a frame on 3-8 [E]; in the air the body's air physics (gap 3).
* **Limit Charge** [E, gaps 9-11]: gains from the move's second frame (full on 335 after 334 frames of charging); a full gauge ends
  it into the cancel; in the air the body's air physics, and a landing carries it on.
* **The gauge** [R / E]: gain from the damage actually added (owner decision 32); the 900-frame timer counts during hitlag (gap 12 [E]);
  a reflected projectile's damage is credited to the reflector (its new owner).

## As built in phase 3 (sim 87): fits and readings

* **Limit Blade Beam** [R / D]: the beam spawns on frame 15 (move frame 14) 12 ahead and 8.75 up (2 + the capsule's centre) [D, as the
  normal beam], lifetime 48 here for the reference's 47 (hits 16-62, the house projectile rule); its two circles (0 to 13.5 up on the
  beam) are one standing capsule, centre 6.75, 6.75 above and below [D]; hitlag x1 and shield damage x1 (no multiplier is listed for
  the Limit beam [R: absent]). The burst stands 0.75 below the capsule's centre (the reference's 6.0 up on the beam) [D]; its age
  restarts at 0, so the loop hits on its frames 1, 7, 13, 19, 25 and the final hit on 29 [D]; the final hit's hitlag x1 (only the loop's
  0.5 is listed). It bursts on a **shield** too (it "acts as if hitting a player") [R reading]; a burst is **not reflected** [E: whether
  the reference's is, unpublished]. No windbox on the Limit beam (none listed) [R: absent].
* **The vortex 366** [R / E]: launch along the attacker's movement at the knockback's speed, no cap, no grounded rule [R]; the 5-frame
  pull is spread evenly so the victim's middle reaches the hitbox centre on the fifth frame, its own fall held meanwhile [E: the
  reference's pull curve is not published]; DI does not bend it [E]. The normal Cross Slash's nearest box is the 366 one [E: conflict 2].
* **Limit Cross Slash** [E: conflict 1]: UFD's frames and damages (26% in all); boxes are the normal Cross Slash's, the last two a little
  larger (radius 7.0 / 6.0 for 6.0 / 5.0) ["slightly larger" R, no number]; set knockback per the dragdown rows; no shield-damage
  multiplier (gap 4); in the air `start_speed` 0, `start_speed_y` 0, gravity 0 and no drift for the whole move [R reading]; no ground
  step [E].
* **Limit Climhazzard** [R / E]: the fitted rise (owner decision 42) at x1.6 / x1.85 [R multipliers]; air control while rising at the
  body's full air speed (`motion_drift` 100) [E: gap 8]; the late rising hit's hitlag x2.5 like the clean one [E: only the clean one's is
  listed]; no set weight [R: none listed]; the box positions are the normal rise's [E].
* **Finishing Touch** [R / D]: positions on the root bone as capsules of circles about 4 reference units apart [D]; one hit group [D];
  landing during the aerial one switches to the grounded one keeping its frame (E22b) [E].
* **Spending**: every special has a powered version, so any special used in Limit Break empties the gauge at once, hit or miss [R].
* Owner decisions 42-44 (recorded in `ROSTER_STATUS.md`): the fitted rise stays [E]; Limit Charge fills from its second frame and ends
  itself on a full gauge [E]; the 900-frame timer counts during hitlag [E].

## Conflicts and how they are resolved

1. **Limit Cross Slash damage and frames**: ssbwiki.com's articles say 27% in total; its moveset summary lists 5 / 3 / 3 / 3 / 10 (24);
   UFD gives 5 / 5 / 3 / 3 / 10 (26). Frames: UFD 10-11 / 15-16 / 21-22 / 28 / 38-39, dragdown 10-17 (hits 1-2) / 21-23 / 28-29 / 39.
   **Unresolved**: no hitbox table exists. Proposed: UFD's frames and damages (it lists two separate hits 1-2 with their own frames,
   which matches "five strikes"), 26% [E]; footage (training mode, a damage readout per hit) settles it.
2. **Cross Slash hit 1's "366"**: dragdown lists three angles 75 / 112 / 366 but two BK and two KS values; which box carries 366 is not
   given. **Unresolved**; built (sim 87): the box nearest the body carries 366, the E28 vortex [E].
3. **Up smash charge hold**: ssbwiki.com "charges between 6-7", UFD "charge hold is frame 5" (UFD gives 5 for the forward smash too,
   where ssbwiki.com says 5-6). Resolved to **ssbwiki.com's first frame** (5 / 6 / 2), as the earlier ports took ssbwiki.com's.
4. **Beam active frames**: ssbwiki.com 18-35 / 36-68 (normal) and 16-63 (Limit); UFD 18-35 / 36-67 and 16-62. The lifetimes 50 and 47
   from spawn give **67** and **62** [D]: resolved to UFD.
5. **Climhazzard part 1's length**: ssbwiki.com animation 57 (Limit 69), dragdown "total" 66 (Limit 69). Resolved to **57 / 69** then the
   special fall [D] (dragdown's 66 has no stated meaning).
6. **Finishing Touch's active frames**: dragdown 12 / 13-21 (grounded) and 14-16 / 17-24 (aerial); ssbwiki.com and UFD 12 / 13 / 14 and
   14 / 15 / 17. ssbwiki.com's trivia explains it: between hits the box stays with its hit bits off and radius 0.1, which dragdown
   counted. Resolved to **ssbwiki.com and UFD**.
7. **Forward smash FAF**: dragdown "total 46" (a copy of the up smash's), ssbwiki.com 67, UFD 66 (FAF 67). Resolved to **67**.
8. **Gauge gain basis**: 0.667 a percent dealt and 1 a percent taken, but whether "percent" is the hit's base damage or the damage shown
   after the one-on-one 1.2 multiplier (and stale moves) is not stated. Proposed: the **damage actually added to the victim's percent**
   (after the multiplier and staleness) [E]; footage (a fresh 10% hit and the gauge's fill) settles it.
9. **The task brief's "up air's multi-hit"**: the 13.0.1 up air is one hit in three phases (clean early, clean late, late) [R]; it is
   built as that.
10. **The roster note's Climhazzard points**: "the descending meteor's sweetspot only after a fall distance" is the ledge rule (above);
    "tech at the peak" is the earlier game's; "landing lag 26" is part 2's landing (part 1's is 20). Corrected here.
11. **Limit Charge cancel delay**: the moveset summary says "an 8 frame delay"; the move page and UFD give 7 frames of lag, FAF 8: the same
    thing counted two ways [D].

## Gaps (needed for 1:1, not published)

1. Cross Slash's radii and positions for every hit (normal and Limit), and the left / right animation difference.
2. Cross Slash's follow-up window ends (hit 1 → 2, hit 2 → 3), and whether a press before the hit lands (buffered) counts.
3. Cross Slash in the air: falling, held, or slowed; landing during it.
4. Limit Cross Slash's shield damage.
5. Climhazzard's rise height in units (normal and Limit) and the rise's shape over frames (the reference's root motion).
6. The "initial thrust" ledge window of Climhazzard part 1 (before frame 24).
7. The fall distances before Climhazzard part 2 can catch a ledge (front and back).
8. Limit Climhazzard's air control while rising and its momentum in the fall.
9. The frame Limit Charge starts adding to the gauge.
10. Whether Limit Charge ends by itself when the gauge fills.
11. Limit Charge's air physics, and landing during it.
12. Whether the 900-frame expiry counts during hitlag and other freezes.
13. Down tilt's slide distance and Cross Slash hit 1's step.
14. Blade Beam's horizontal air control (its "slight manoeuvrability": an air speed or acceleration multiplier is not given beyond the
    startup x0.4).

Footage that would settle most of them (training mode, frame-stepped, hitbox view, the gauge visible): Cross Slash whiffed, on hit and on
shield, pressing at several frames, facing both ways, grounded and from a short hop; Limit Cross Slash with a damage readout per hit;
Climhazzard from the ground and from the air next to a ledge, with and without the second press, normal and Limit; Limit Charge from
empty to full (does it end?), cancelled each way, and hit during the charge; a 10% hit dealt and taken with the gauge shown.
