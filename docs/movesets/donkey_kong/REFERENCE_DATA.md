# Reference data for the eighth moveset (the reference game's Donkey Kong)

Mechanical reference for the eighth port, collected 2026-10-13 for version **13.0.1**, before anything is built. Proposed in-game
names (not registered; see `PROJECT_AUDIT.md`): the **Bracers** class and weapon on the **hauler** body, built-in fighter **Hauler**.
The reference character's name is not used in the game. What the engine can and cannot do with this data is in `PROJECT_AUDIT.md`; the
format follows `docs/movesets/cloud/REFERENCE_DATA.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Donkey_Kong_(SSBU)/<move>` (28 pages: `Neutral_attack` and its `Hit_1` / `Hit_2`, the three tilts,
  `Dash_attack`, the three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(back|front|trip)`,
  `Neutral_special`, `Side_special`, `Up_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), so every
  hitbox value comes from a named template field (`damage`, `angle`, `af` angle type, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff`
  hitlag multiplier, `sdi`, `sd` shield damage, `effect`, `trip`, `clang`, `rebound`, `direct`, `hitbits`, `g=f` / `a=f`) and the angle-361
  column shift cannot happen. The specials' pages also give the move **parameters** (speeds, multipliers, the charge time). The forward
  throw page holds the **carry** (cargo) data: the four cargo throws, the carry's timer formula, mashing and the walk multiplier.
  https://www.ssbwiki.com/Donkey_Kong_(SSBU)
- **ssbwiki.com articles** `Donkey_Kong_(SSBU)` (attributes template, moveset summary, update history), `Giant_Punch`, `Headbutt`,
  `Spinning_Kong`, `Hand_Slap`, **`Bury`** and **`Buried`** (the bury effect and the buried state), **`Armor`** (super, damage-based and
  knockback-based armor), `Forward_throw`, `Grab`, `Grab_release`, `Dash` (current-game table), `Template:SSB4_to_SSBU_changelist/Donkey_Kong`
  (changes from the earlier game, used only to understand the current behaviour) and `Template:UpdateList_(SSBU)/<version>` for every
  version that touched this character (1.2.0, 2.0.0, 3.0.0, 3.1.0, 4.0.0, 6.0.0, 7.0.0, 9.0.0, 13.0.1).
- **ultimateframedata.com/donkey_kong** (UFD: startup, total frames, landing lag, notes, active frames, dodges, jump air times, stats). UFD's
  "total frames" is FAF - 1. https://ultimateframedata.com/donkey_kong
- **dragdown.wiki** `SSBU/Donkey_Kong/Data` (the parameter file and a frame-data table; "total" = FAF - 1) and `SSBU/Donkey_Kong` (player
  notes: armor kinds, the carry's armor, ledge windows). https://dragdown.wiki/wiki/SSBU/Donkey_Kong/Data
- **kuroganehammer.com/Ultimate/Donkey Kong** (labelled "[13.0.1]": FAF, damage, angle, BK/FKV, KS, landing lag, autocancel, dodges;
  the URL needs the space, `Donkey%20Kong`). https://kuroganehammer.com/Ultimate/Donkey%20Kong

**Update history applying to 13.0.1** (ssbwiki.com update lists, every entry for this character): **1.2.0** "changes currently unknown".
**2.0.0** forward air autocancels from 56 (was 63). **3.0.0** Giant Punch's range 5 → 6.5 (the full-charge mid-arm box). **3.1.0** the
full-charge punch's super armor lasts longer, and a bug fix: landing on frames 12-17 of a full-charge punch no longer loses the armor.
**4.0.0** sleep wake-up lag (shared, not modelled). **6.0.0** forward tilt KS 55 → **78**; neutral air 11 / 8 → **12 / 9**, boxes 4.5 / 4.5 /
5.5 → **5 / 5 / 5.7**, a fourth clean box on its first frame at y 8, z 15; **Giant Punch full charge on frame 131 → 121** (charge 120 → 110
frames), full-charge BK 20 → **35**, KS 78 → **73**; grounded Spinning Kong armor 8-17 → **5-17**, loop boxes against aerial targets at
**366** (were 80), last hit BK 40 → **60**, KS 170 → **157**; grounded Hand Slap startup 15 → **12**, FAF 58 → **47**. **7.0.0** shield size
(global here, not modelled); the head briefly intangible as the shield goes up and down (not modelled: a shield hook, as Yoshi's). **9.0.0**
grounded Headbutt's second box against grounded targets (y 5 → **13**). **13.0.1** jab 2 active 5-6 → 5-7 and FAF 35 → **32** (conflict 1);
down tilt **+5 hitstun** on every box; **full-charge Giant Punch super armor 11-20 → 9-20 grounded, 17-20 → 15-20 aerial**. Every table
below is the post-13.0.1 data. No data from the earlier games was used; where they differ (the earlier game's helpless aerial Giant
Punch, the earlier bury formula, stage-based Giant Punch charge levels) it is called out.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift, a reading of a field);
**[E]** estimate. Hitbox **positions** on a bone other than `top` (`arml` / `armr` the forearms, `handl` / `handr`, `shoulderl` / `shoulderr`,
`hip`, `kneel`, `footl`, `head`, `waist`) are [E]: the reference attaches them to its own skeleton, so ours go on our body where that bone
is in the move's pose. Offsets on `top` (the character's root at the feet: `zpos` forward, `ypos` up) convert directly [D]. A reference
capsule (`zpos=19.0 to 7.0`) is two or three circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first, FAF as published: ssbwiki.com's "Interruptible" is the FAF); the sim stores
  them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius in world tenths = reference value x 1.25, rounded half up [D] (the house rule). Speeds:
  reference units a frame / 8 = world units a frame [D].
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; **361** is the reference's 361 angle ("angle 361"), **367**
  the autolink angle, **366** the reference's "vortex" autolink (not built: E28 in `docs/movesets/cloud/PROJECT_AUDIT.md`). H = hitlag
  multiplier (`ff`), SDI = SDI multiplier.
- **Angle type** (`af`): unset = 0 (by position, away from the hitbox's owner), 3 forward (the owner's facing), 4 backward.
- **Targets**: `a=f` cannot hit aerial targets (grounded only), `g=f` cannot hit grounded targets (aerial only) [R].
- **Shield damage**: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd` becomes
  `100 x (damage + sd) / damage`, rounded [D].
- `clang=f rebound=f`: never clanks, never rebounds (E3c, `Hitbox::clanks`) [R]. **`rebound=f` alone**: the hit clanks, but its owner does
  not rebound (UFD's "Cannot rebound"); the engine has no such flag (E3g in the audit). `direct=f`: an indirect hit (the reference's
  flag for hits that do not count as direct contact; it changes some counters' behaviour) [R], not modelled.
- `trip=N`: an N% chance to trip a grounded target [R]; not modelled anywhere in the engine (E20).
- `effect=Bury`: the bury effect (section "Bury"); `effect=Rapid`: presentation only (the multi-hit's hit effect).

## Attributes [R]

ssbwiki.com attributes template, agreeing with UFD, kuroganehammer and the parameter file (dragdown):

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **127** | 1.365 | 2.09 | 1.873 | 0.123 | 1.208 | 0.01 + 0.05 | 0.015 | 0.085 | 1.63 / 2.608 | 34.0 / 17.3 / 35.5 | 3 | 1 |

Weight 127: third-heaviest in the reference game, the same as another super-heavyweight there (ssbwiki.com) [R].

Parameter file (dragdown) [R]: `walk_accel_add` **0.105**, `walk_accel_mul` **0.105** (ours: `walk_accel` 0.105, `walk_accel_stick` 0.105);
`walk_speed_max` 1.365; `run_accel_add` **0.044**, `run_accel_mul` **0.07337** (ours: `run_accel` 0.044, `run_accel_stick` 0.07337;
ssbwiki.com's Dash table agrees: base 0.044, additional 0.07337, max 0.11737); `ground_brake` **0.123**; `dash_speed` 2.09; `run_speed_max`
1.873; initial dash **15 frames**, pivot dash 17 (ssbwiki.com Dash table); `jump_initial_y` 18.7; `jump_speed_x` 0.95, `jump_speed_x_mul`
**0.8**, `jump_speed_x_max` **1.6**, `jump_aerial_speed_x_mul` 1.0; `air_accel_x_mul` 0.05, `air_accel_x_add` 0.01; `air_brake_x` 0.015;
`air_brake_y` 0.015; `air_ground_speed_brake` 0.045; `landing_frame_light` 1, `landing_frame` 4, **`landing_heavy_frame` 4** (so **3**
frames of heavy landing lag here [D], by the house rule `landing_heavy_frame - 1`; kuroganehammer's "hard landing lag 5" counts the FAF
and its "soft 2" the light landing: conflict 9); `landing_attack_air_frame` n / f / b / hi / lw **10 / 17 / 11 / 15 / 14** (agreeing with
UFD, kuroganehammer and ssbwiki.com's landing "interruptible" 11 / 18 / 12 / 16 / 15); `cliff_jump_y` **37.0**, `cliff_jump_x_speed` 0.6;
`jump_count_max` 2; `scale` 1.0; `size` `fighter_size_l` (a large fighter); `shield_radius` 15.4 (shields are global here, not used);
`jostle_front` 4.2, `jostle_back` 2.0, `jostle_weight` 8.2 (no jostling in the engine). **No wall jump** (`wall_jump_type` False,
`attach_wall_type` False; kuroganehammer "Wall Jump: No"); the wall-tech jump speeds are `passive_wall_jump_x_speed` **1.3**,
`passive_wall_jump_y_speed` **2.7** (ours: `wall_jump` 0, `wall_jump_vx` 1.3, `wall_jump_vy` 2.7 [D], the wall-tech jump only). Footstool
(`tread_*`) not modelled for anyone. `damage_fly_top_air_accel_y` 0.084672 / `damage_fly_top_speed_y_stable` 1.8 (the shared launch
fall covers them).

**Carrying** (parameter file, the reference's heavy-item lift, which ssbwiki.com says the carry shares) [R]: `item_lift_accel_mul`
0.0976923, `item_lift_accel_add` 0.03, `item_lift_speed_max` **1.3246**. ssbwiki.com's forward-throw page instead says "while carrying an
opponent or heavy item, Donkey Kong's walk speed and acceleration are increased by 1.1x" (conflict 5).

Jump air times (UFD, kuroganehammer) [R]: short hop **40**, full hop **53**, short hop fast fall **27**, full hop fast fall **37**.

Dodges (UFD total frames, so FAF one more; kuroganehammer's FAFs agree) [R]: spot dodge **26** (FAF 27, intangible 3-17), forward roll
**30** (FAF 31, 4-15), back roll **35** (FAF 36, 5-16): each one frame longer than the shared values. Neutral air dodge **48** (FAF 49,
intangible **4-32**, landing lag 10); directional down **66**, diagonal down 73, sideways **82**, diagonal up 88, up **109** (intangible
**4-22**; landing lag 11-19). Note the **frame-4** air dodge (most of the cast's starts on 3: "a frame 4 air dodge", ssbwiki.com) [R]. The
body takes UFD's numbers as every other body does; the engine's three directional lengths are down / side / up 66 / 82 / 109 (no separate
diagonals) [D].

Out of shield (UFD) [R, for checks]: aerial up special 7 frames, up air 9, back air 10, shield grab 12.

Ledge (ssbwiki.com edge attack page; parameter file): ledge attack intangible 1-25 on the first grab, **1-20** on the second, **1-12** on
the third, none from the fourth (ours: `ledge_option_decay_2` 80, `ledge_option_decay_3` 48 [D]).

Body size [E]: "one of the largest hurtboxes ... even bigger than his model would suggest" (ssbwiki.com), `fighter_size_l`, scale 1.0;
the up smash's top box is 28 reference units up and the forward smash's tip 24 forward at 6.4 up; our hauler body is fitted to those
reaches (a taller, wider ECB than the shared one) [E].

## Normals

All frame numbers [R] (ssbwiki.com timing tables; UFD, dragdown and kuroganehammer agree on every startup and FAF unless listed under
"Conflicts"). Damage, angle, BK, KS, FKV, H, SDI [R]. Radius column: reference → world tenths [D]. Positions in reference units on the
named bone.

### Jab [R]

Two hits, a hook and an uppercut (10% total). Quickest loop: jab 1 on 5-6, jab 2 on 11-13; FAF 25 / 39 (from the start of the chain).

| Part | Frames | Continue | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Jab 1 | 5-6 | **8-28** | 25 | 0 | 4 | 361 (3) | 25 | 35 | 6.0 → 8 | top (11.5, 5.0) [D] | 1.5 | |
| | | | | 1 | 4 | 361 (3) | 20 | 30 | 4.5 → 6 | top (10.5, 11.0) [D] | 1.5 | |
| | | | | 2 | 4 | **180** (3) | 20 | 20 | 4.5 → 6 | handl (y 2.0) [E] | 1.5 | **fighters only**: pulls the target in |
| | | | | 3 | 4 | 361 (3) | 20 | 20 | 4.5 → 6 | handl (y 2.0) [E] | 1.5 | the same box for objects (id 2 wins on fighters); not modelled |
| Jab 2 | **4-6** (conflict 1) | | **32** (13.0.1) | 0 | 6 | 70 (3) | 40 | 100 | 7.0 → 9 | handr (1.0, 1.5, -2.0) [E] | 1.8 | |
| | | | | 1 | 6 | 70 (3) | 40 | 100 | 5.5 → 7 | top (9.5, 7.0) [D] | 1.8 | |
| | | | | 2 | 6 | 70 (3) | 40 | 100 | 5.0 → 6 | top (15.0, 5.0) [D] | 1.8 | |

The continuable window is jab 1's frames 8-28 (attack pressed in it carries on into jab 2 from frame 8: the Embers' `next_from`) [R];
UFD "transitions to Jab 2 as early as frame 8" [R]. No rapid jab and no third hit: jab 2 ends the chain [R]. Jab 2 starting on jab 1's
frame 8 puts its 4-6 on 11-13 and its FAF 32 on 39, exactly the quickest loop [D].

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt (straight) | 7-9 | **35** | 0 / 1 / 2 | **8** | 361 (3) | 50 | **78** (6.0.0) | 5.5 / 5.0 / 3.5 → 7 / 6 / 4 | arml (x 10.0 / 1.5), shoulderl (x -0.5) [E] | **lower arm intangible 6-9** (E19 limb) |
| Forward tilt (up / down) | 7-9 | **36** | 0 / 1 / 2 | **9** | 361 (3) | 50 | 78 | as straight | as straight [E] | the angled versions last one frame longer ("an oversight", ssbwiki.com); the down version has **trip 15%** |
| Up tilt | 5-11 | 39 | 0 | **8** | 100 (0) | 40 | **105** | 4.8 → 6 | shoulderr (x 2.7) [E] | **lower arm intangible 5-11** |
| | | | 1 | **9** | 100 (0) | 40 | **110** | 4.0 → 5 | armr (x 3.8) [E] | the forearm |
| | | | 2 | **10** | 100 (0) | 40 | **115** | 5.3 → 7 | armr (x 1.0) [E] | the fist: the strongest ("sweetspotted up tilt") |
| Down tilt | 6-7 | 25 | 0 / 1 / 2 | 6 | 361 (0) | 10 | 80 | 4.4 / 4.4 / 5.5 → 6 / 6 / 7 | shoulderl (x 2.3), arml (x 1.0 / 8.5) [E] | **+5 hitstun** (13.0.1), **trip 40%**; **whole arm intangible 5-7** |
| Dash attack, clean | 9-12 | 35 | 0 | **12** | 53 (3) | **85** | 46 | 7.0 → 9 | top (8.0, 8.0) [D] | H 1.25; a rolling attack |
| Dash attack, late | 13-24 | | 0 | **9** | 45 (3) | 60 | 60 | 5.5 → 7 | top (8.0, 5.2) [D] | H 1.25 |

Up tilt boxes have no angle type (by position): a target behind is sent backward [D]. The down tilt's arm intangibility covers the whole
attacking arm ("his only tilt to grant full intangibility to his attacking arm", ssbwiki.com trivia) [R]; the forward and up tilts' only
the lower arm [R]. Dash attack: "can catch items on frames 1-5" and "cancel the first 3 frames into a dash grab" (ssbwiki.com) [R]: items
do not exist; the dash-grab cancel is the engine's general rule to check.

### Smashes [R]

| Move | Frames | Charge hold | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash | **22-23** | 14 (charges 14-15) | 55 | 0 | **22** | 46 (3) | 21 | 88 | 5.7 → 7 | top (6.4, 24.0) [D] | the fist (far); shield +1 (105 [D]); H 1.4; **cannot rebound** |
| | | | | 1 | **21** | 361 (3) | 18 | 86 | 3.7 → 5 | top (6.4, 18.0 to 10.0) [D] | the arms (close); shield +1 (105); H 1.4 |
| Up smash | **14-15** | 6 (charges 6-7) | 50 | 0 | **19** | 90 (0) | 40 | 84 | 8.8 → 11 | top (28.0, 0) [D] | overhead clap; shield +4 (121 [D]); cannot rebound |
| | | | | 1 | 19 | 90 (0) | 40 | 84 | 7.0 → 9 | top (18.0, 5.0 to -5.0) [D] | |
| Down smash, clean | **11-12** | 2 (charges 2-3) | 56 | 0 / 1 | **17** | 70 (0) | 35 | 88 | 4.0 → 5 | armr / arml (x 5.5) [E] | the fists, both sides; shield +1 (106); cannot rebound |
| | | | | 2 / 3 | 14 | 82 (0) | 35 | 88 | 4.0 → 5 | shoulderr / shoulderl (x 2.0) [E] | shield +1 (107) |
| Down smash, late | 13-14 | | | 0 / 1 | **18** | 55 (0) | 30 | 88 | 6.0 → 8 | top (4.5, 19.0) / top (4.5, -12.0) [D] | front and back |
| | | | | 2 / 3 | 14 | 98 (0) | 35 | 88 | 4.5 → 6 | shoulderr / shoulderl (x 2.0) [E] | |

Intangibility: forward smash **head and lower arms 20-26**, up smash **head and lower arms 12-15**, down smash **lower arms 11-14** [R]
(E19 high + limb). Charged smash damage is the shared rule (up to 1.4x over 60 frames), agreeing with dragdown's charged values (30.8 /
29.4 forward, 26.6 up, 23.8 / 19.6 / 25.2 down) [R]. The down smash's two phases are one hit (a target is hit once) [D].

### Aerials [R]

| Move | Frames | Autocancel | Landing lag | FAF | Phase / id | Dmg | Angle (type) | BK | KS | Radius | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Neutral air, clean early | **10** | 1-9, 27+ | **10** | 39 | 0 / 1 | **12** (6.0.0) | 361 (0) | 20 | 100 | 5.0 → 6 | armr / arml (x 5.0) [E]: both arms out |
| | | | | | 2 | 12 | 361 (0) | 20 | 100 | 5.7 → 7 | hip (1.6, 3.1) [E] |
| | | | | | 3 | 12 | 361 (0) | 20 | 100 | 5.0 → 6 | top (8.0, 15.0) [D] (6.0.0: first frame only) |
| Neutral air, clean late | 11-13 | | | | 0 / 1 / 2 | 12 | 361 (0) | 20 | 100 | 5.0 / 5.0 / 5.7 → 6 / 6 / 7 | as early |
| Neutral air, late | 14-26 | | | | 0 / 1 / 2 | **9** | 361 (0) | 10 | 100 | as clean | |
| **Forward air**, clean | **18-20** | 56+ (2.0.0) | **17** | 56 | 0 | **16** | 361 (3) | 20 | 85 | 5.5 → 7 | arml (8.0, 1.1, 2.0) [E] |
| | | | | | 1 | 16 | 361 (3) | 20 | 85 | 4.5 → 6 | arml (3.0, 2.0, 3.0) [E] |
| Forward air, late | **21-23** | | | | 0 | **15** | **270** (3) | 30 | 100 | 5.5 → 7 | arml (10.0, 4.0, 3.7) [E]: **the meteor** (the fist, late); H 1.5; hits anyone |
| | | | | | 1 | 13 | 361 (3) | 30 | 100 | 5.0 → 6 | arml (3.0, 2.0, 3.0) [E]; H 1.5 |
| **Back air**, clean | **7-8** | 1-6, 31+ | **11** | 32 | 0 | **13** | 361 (**4**) | 20 | 97 | 7.0 → 9 | footl (1.0, -2.0) [E] |
| | | | | | 1 / 2 | 13 | 361 (4) | 20 | 97 | 5.0 → 6 | kneel (-1.0), hip (-1.0) [E] |
| Back air, late | 9-16 | | | | 0 | **8** | 361 (4) | 10 | 100 | 6.0 → 8 | footl [E] |
| | | | | | 1 / 2 | 8 | 361 (4) | 10 | 100 | 4.0 → 5 | kneel, hip [E] |
| **Up air** | **6-10** | 1-5, 26+ | **15** | 38 | 0 | **13** | 90 (0) | 32 | 90 | 7.5 → 9 | head (2.5, 2.5) [E]: an arcing headbutt; **head intangible 5-10** |
| **Down air** | **14-16** | 1-2, 50+ | **14** | 55 | 0 | **16** | **270** (0) | 30 | 90 | 7.0 → 9 | top (-5.0, 1.0) [D]: **the meteor** (the feet, below); hits anyone |
| | | | | | 1 | 13 | 361 (0) | 20 | 90 | 5.8 → 7 | top (6.0, 0) [D]: the leg, at the body |

Landing lag = `landing_attack_air_frame` (dragdown) = ssbwiki.com's landing "interruptible" - 1 = UFD's landing lag [D]. The forward
air's meteor is its **late** hit's fist (not a clean sweetspot), and it hits grounded targets too (no `g=f`) [R]; the down air's meteor is
the box below the feet on all three active frames, also with no target restriction [R]. The back air sends backward always (angle type
4: "Bair will always launch opponents in the opposite direction DK is facing") [R]. The neutral air's clean late phase drops the far
box [R].

### Grabs, pummel and throws [R]

| Move | Frames | FAF | Data |
| --- | --- | --- | --- |
| Standing grab | 8-10 | 39 | id 0 r 5.8 → 7, top (7.0, 4.0 to 13.7) [D], grounded targets only; id 1 r 2.9 → 4, (7.0, 1.1 to 16.6), aerial only |
| Dash grab | 11-13 | 47 | id 0 r 4.6 → 6, top (7.0, 4.0 to 15.9); id 1 r 2.3 → 3, (7.0, 1.7 to 18.2) |
| Pivot grab | 12-14 | 42 | id 0 r 5.8 → 7, top (7.0, -4.0 to -21.5); id 1 r 2.9 → 4, (7.0, -1.1 to -24.4) (written in front: the engine turns a pivot grab round [D]) |
| Pummel | **2** | **8** | **1.6%**, 361 (3), BK 0, KS 100, **FKV 30**, r 6.0 → 8, top (13.0, 18.0) [D], **H 2.3**, no clank. The hit is on frame 2, so the engine's "pummel cannot hit on its first frame" house rule needs no shift: frame 2, FAF 8 exactly [D]. UFD's "21 including 15 frames of hitlag" agrees |
| Forward throw = **Kong Karry** (the carry) | carry animation **15** frames, **invincible 1-15** | | puts him into the **cargo hold** (next section); there is no plain forward throw |
| Back throw | turns round **15**, release **15** | **40** | 11%, **40**, BK 60, KS 65. Invincible 1-15 |
| Up throw | release **14** | **44** | 9%, **90**, BK 70, KS 50. Invincible 1-14 |
| Down throw | release **19** | **55** | 7%, 361, BK 60, KS 50 (sends forward along the ground). Invincible 1-19 |

No throw has a pre-hit [R]. Each also has the "interrupted" row (`kind=1`: 3%, 361, BK 40, KS 100), not modelled (as for the other
ports); the thrower's invincibility rows are not modelled (house limit). The changelist's "throws are no longer weight-dependent" is
about their **timing** (the earlier game slowed some throws on heavy targets); the release frames above are fixed for every target, and
no throw row has `setweight`, so the knockback uses the victim's weight as usual [R reading] (conflict 8).

### The cargo hold (Kong Karry) [R]

| Property | Value | Source / label |
| --- | --- | --- |
| Entry | the forward throw: grab, then the stick forward; a **15-frame** lift animation, **invincible 1-15** | ssbwiki.com forward throw page, dragdown ("full invincibility 1~15") [R] |
| While carrying | "can move, jump, and walk off the stage while having the opponent in his hands, acting like he usually would outside of a typical grab (**aside from being unable to use his other moves and having only one jump**)" | ssbwiki.com moveset summary [R] |
| Walk | **walk speed and walk acceleration x1.1** while carrying (1.365 → 1.5015; 0.105 + 0.105 → 0.1155 + 0.1155 [D]); **or** the parameter file's heavy-item lift values (speed 1.3246, accel 0.03 + 0.0976923) | conflict 5 |
| Run, dash, shield, dodges, attacks | not available (only walking, jumping and the four cargo throws) | ssbwiki.com [R reading of "unable to use his other moves"] |
| Jump | **one jump** in all ("only one jump"): the ground jump, or a midair jump after walking off [R text; which of the two is not stated: gap 3]; jump height while carrying not published (gap 3) | ssbwiki.com [R] |
| Offstage | walking off the stage keeps the carry; the victim is taken along ("walk off the edge, pull back and throw them against the underside" for a stage spike) | ssbwiki.com [R] |
| Ledges | whether he can grab a ledge while carrying is **not published** (gap 4) | |
| Armor | after the 15 invincible frames, **knockback-based armor** "until he initiates a throw" (dragdown: "full body knockback armor 16+"); the threshold is a conflict: **16 units** (ssbwiki.com: "resist up to 16 Units of Knockback") or **80 units** (dragdown: "able to tank up to 80 total knockback units (i.e. any move that induces tumble)") | conflict 2 |
| Cargo throws' invincibility | none: "the actual cargo throws do not have invulnerability" (dragdown) | [R] |
| Duration | **`((90 + 1.7p) x 11 / 6) - g`** frames, **p** the victim's percent when the carry starts and **g** the grab time already passed (button mashing included). At 0% straight out of a grab: 165 frames; after 60 frames of grab: 105 | ssbwiki.com forward throw page and moveset summary [R]; the examples check: 90 x 11 / 6 = 165, 165 - 60 = 105 [D] |
| Mash | as an ordinary grab: **8 frames per stick input, 14.4 per button input** (buttons cannot register as quickly); mashing during the grab "transfers over" (it is in **g**) | ssbwiki.com [R], dragdown [R] |
| Escape | the victim breaks free (a grab release); the cargo release's own lag and the victim's escape motion in 13.0.1 are **not published** (gap 5) | |
| Interrupted | DK hit out of the carry (over the armor): the carry ends; the carry's `kind=1` row is **4%, 361, BK 0, KS 100, FKV 30** (the hit the victim takes when the carry is interrupted [D reading]) | ssbwiki.com forward throw page [R] |
| Throw input | a direction **and the attack button** ("by holding a direction and pressing the attack button"); attack alone is the forward cargo throw ("(Forward +) Attack", dragdown) | ssbwiki.com, dragdown [R] |

**The four cargo throws** [R] (ssbwiki.com forward throw page; UFD, dragdown and kuroganehammer agree on every release and FAF):

| Cargo throw | Release | FAF | Dmg | Angle | BK | KS | Other |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Forward | **15** | **46** | 12 | **60** | 90 | 46 | a high forward toss |
| Back | turns round **16**, release **16** | **43** | **13** | **72** | 85 | 52 | the strongest; "changes direction on frame 16" |
| Up | **15** | **30** | 12 | **92** | 65 | 53 | the combo throw (into up air); the shortest FAF |
| Down | **17** | **45** | 11 | **46** | 70 | 45 | a bear hug; **on frame 17 his horizontal speed stops and his vertical speed is set to 1.5** (→ 0.1875 world [D]): the "slight boost upwards" in the air |

Each cargo throw's `kind=1` row: 3%, 361, BK 40, KS 100 (interrupted; not modelled). Usable in the air (the cargo throws offstage, the
down throw's boost) [R].

### Ledge and floor [R]

| Move | Frames | Intangible | FAF | Data |
| --- | --- | --- | --- | --- |
| Ledge attack | **23-25** | 1-25 (1st grab; 1-20 2nd, 1-12 3rd, none from the 4th) | 56 | 10%, 45 (3), BK 90, KS 20, shield +1 (110 [D]), r 5.5 → 7, top (5.5, 12.0 to -5.5) [D], no clank |
| Get-up attack, face up (ssbwiki.com "back") | behind **18-19**, front **23-24** | 1-24 | 46 | 7%, 48 (0), BK 80, KS 48, shield +8 (214 [D]), r 6.5 → 8; top (6.5, -19.0 to -4.5) then (6.5, 19.0 to 5.5) [D]; H 1.0, no clank |
| Get-up attack, face down (ssbwiki.com "front") | front **18-19**, behind **23-24** | 1-24 | 46 | as face up; top (6.5, 19.0 to 3.5) then (6.5, -19.0 to -3.5) [D] |
| Get-up attack, trip | front **19-20**, behind **24-25** | 1-7 | 50 | 5%, 361, BK 60, KS 50, shield +8 (260 [D]), r 6.5 → 8; top (6.5, 19.0 to 3.5) then (6.5, -18.0 to -3.0) [D]; no clank (animation 49 < FAF 50: as published) |

One get-up attack slot exists (as for the other ports): the face-up one would be used.

## Specials

### Giant Punch (neutral special) [R]

A wind-up punch that **charges by itself** once started; special again releases it; it can be cancelled and the charge **kept**; at
full charge it is a different, faster, **super-armoured** punch.

| Part | Data |
| --- | --- |
| Start | **7 frames** to enter the charge loop ("Entering charge window takes 7 frames", UFD); ssbwiki.com "charges between **7-8**" (the loop points) [R] |
| Charge | **110 frames** to full since 6.0.0 (120 before) [R]; full on move frame **121** (6.0.0 patch note "frame 131 → 121") [R]; UFD "takes 111 frames to reach full charge" (conflict 3). In 13.0.1 the charge is **continuous**, not in stages: "power will increase by a given amount every 6.67 frames", 18% over 110 frames (ssbwiki.com `Giant_Punch`) [R]. The wind-up animation speeds up by 0.009 a frame (presentation) |
| Release (special pressed while charging) | the uncharged punch with the charge's bonus: hitbox on the release's **19-20** (26-27 with the shortest charge), **FAF 63 from the release** (70 overall), animation 75 from the release; **lower arm intangible on the release's 4-20** (11-27 overall) [R]. The punch "cannot rebound" (`rebound=f`) |
| Damage by charge | **`base + 18 x (charge frames / 110)`, rounded down** (ssbwiki.com): grounded 10 → up to 27 (UFD "10.0—27.0"), aerial 6 → up to 23 [D]; the knockback stays the uncharged rows' (BK 45 KS 70 grounded, BK 60 KS 65 aerial) [R] |
| Full charge | at 110 frames the charging ends by itself and the punch is **stored** at full ("after ten swings, Donkey Kong will drop his fist ... At this point the button only needs to be pressed once": the move article) [R]; the stored full punch comes out on the next neutral special press |
| Cancel | **shield** (into shield), **roll**, **spot dodge** and **jump** cancel the charge and **keep it**, each after **4 frames** of lag ("charge-canceling incurs 4 frames of lag before the selected action", ssbwiki.com; "4 to cancel with shield", UFD; the changelist: shield 8 → 4 frames, jump and spot dodge "with the same amount of lag", rolls now with this lag too) [R]; in the air an **air dodge** cancels (the move article; ssbwiki.com) [R] |
| Hit while charging | "If Donkey Kong suffers knockback while charging, he will lose all of his charge" (the move article) [R] |
| KO | the charge is lost on a KO (dragdown: after a KO he charges again from the revival platform) [R reading] |
| Turnaround | the punch "can be turned around, b-reversed, and wavebounced"; **the charge cannot be b-reversed, but can be turned around** (the reference's turnaround special at the start; dragdown) [R]; the frames are the shared special-turn windows [E] |
| Air | usable in the air; **no helplessness** after the aerial punch [R]; its air physics are not published (gap 6). Landing during an aerial punch: "if DK starts the move in the air, then lands on the ground, the grounded super armor window will take over" (dragdown), so landing switches the move to its grounded version, keeping its frame [R reading] |

**Uncharged punch boxes** (the release; damage rises with charge as above) [R]:

| Version | id | Dmg (uncharged) | Angle | BK | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Grounded | 0 | 10 | 361 (0) | 45 | 70 | 5.5 → 7 | arml (x 8.0) [E] | the fist; cannot rebound |
| | 1 | 10 | 361 (0) | 45 | 70 | 4.0 → 5 | arml (x -1.0) [E] | |
| | 2 | 10 | 361 (0) | 45 | 70 | 3.0 → 4 | shoulderl (x -1.5) [E] | |
| Aerial | 0 / 1 / 2 | 6 | 361 (0) | 60 | 65 | as grounded | as grounded [E] | |

**Full-charge punch** (its own animation) [R]:

| Part | Grounded | Aerial |
| --- | --- | --- |
| Hitbox | **19-20** | 19-20 |
| **Super armor** | **9-20** (13.0.1; 11-20 before) | **15-20** (13.0.1; 17-20 before) |
| FAF | **48** (animation 75) | 48 |
| Boxes | ids 0 / 1 / 2 **28%**, id 3 **18%** (the waist), 361, **BK 35, KS 73**; r 5.0 / 6.5 / 4.0 / 3.0 → 6 / 8 / 5 / 4; arml (x 8.0), arml (x -1.0, z 1.5), shoulderl (x -1.5), waist (x 1.0) [E] | ids 0-3 **25%**, 361, **BK 40, KS 72**, same sizes and bones |
| Other | cannot rebound; the "Special Zoom" on a hit (presentation, not modelled) | |

The full-charge punch is "overall faster" than a charged release (FAF 48 against 63 from the release) [R]. "Is DK's only armored move
where the armor overlaps his active frames" (dragdown) [R]. The super armor is **super** (every hit), not knockback-based ("Has super
armor, not heavy armor, allowing him to tank any move", dragdown) [R].

### Headbutt (side special) [R]

A downward headbutt: on the ground its bury boxes **bury** grounded targets; its upper box hits only airborne ones; in the air its bury
box hits anyone, and on an airborne target the bury effect does nothing, so the 270-degree box is a **meteor**.

| Part | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | Targets | Effect | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Grounded | 0 | 10 | 270 (0) | 40 | 40 | 5.0 → 6 | top (5.0, 19.0 to 7.0) [D] | **grounded only** | **bury** | shield **+25** (350 [D]); H 1.0; SDI 1.0 |
| | 1 | 10 | 270 (0) | 40 | 40 | 3.0 → 4 | top (**13.0**, 18.0 to 9.0) [D] | grounded only | bury | 9.0.0's higher box (tall grounded targets) |
| | 2 | **8** | 70 (0) | 15 | 40 | 10.0 → 13 | top (11.0, 14.0) [D] | **aerial only** | | shield +25 (413 [D]) |
| Aerial | 0 | 10 | 270 (0) | **30** | 40 | 6.5 → 8 | top (5.0, 8.8 to 1.4) [D] | anyone | bury | grounded targets buried, airborne ones meteored |
| | 2 | 8 | 70 (0) | 15 | 40 | 10.0 → 13 | top (11.0, 14.0) [D] | aerial only | | |

| Timing and rules | Data |
| --- | --- |
| Hitbox | **20-21** (both versions) |
| **Super armor** | **5-14** (both versions; UFD "on the ground and in the air") [R]; "a significant vulnerable gap in between his super armor and the active hitbox (5 frames)" (dragdown) [R]. Super, not knockback-based (dragdown) [R] |
| FAF | **63** (animation 69) [R]; -32 on shield |
| Movement | horizontal speed x**0.5** grounded / x**0.67** aerial on starting; vertical speed x**0** grounded / x**0.5** aerial on starting; **gravity multiplier 0.5** (0.0425 [D]) ["the aerial version stalls less (gravity multiplier: 0.3 → 0.5)", changelist] [R] |
| Ledges | "can snap on to the ledge during the headbutt's animation **after frame 2**" (ssbwiki.com overview) vs "DK can grab the ledge **on every frame** of this move's animation" (dragdown) (conflict 4); "if Headbutt hits the opponent as DK grabs the ledge, its ending lag is cut" [R] |
| Shield damage | "tremendous": +25, "not a one-hit shield break" [R] |
| Total | 10% |

### Spinning Kong (up special) [R]

**Grounded**: a spinning clothesline that travels along the ground, armoured during the startup; seven hits.

| Part | Frames | id | Dmg | Angle (type) | BK / FKV | KS | Radius | Bone (y, z) | Targets / other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Hit 1 | **19** | 0 | 5 | 95 (0) | FKV **90** | 30 | 6.0 → 8 | armr (6.5, -1.0, -1.0) [E] | **grounded only**; shield +1 (120 [D]) |
| | | 1 / 2 | 5 | 80 (0) | FKV 90 | 30 | 5.0 / 4.0 → 6 / 5 | armr (x 0), shoulderr [E] | grounded only |
| | | 3 / 4 / 5 | 5 | **366** (0) | FKV 90 | 30 | 6.0 / 5.0 / 4.0 → 8 / 6 / 5 | as ids 0 / 1 / 2 [E] | **aerial only** (6.0.0: was 80) |
| Hits 2-6 (loop) | **25-27, 32-33, 40-42, 49-50, 55-58** | 0 | **1.4** | **367** (3) | FKV **90** (hit 3: **80**) | 55 | 4.0 → 5 | top (7.0 to 12.0, 25.0) [D] | the far arm |
| | | 1 | 1.4 | 20 (3) | FKV 65 | 100 | 5.0 → 6 | top (12.0, 12.5) [D] | |
| | | 2 | **3.5** | 60 (**4**) | BK 35 | 100 | 5.7 → 7 | top (13.5, -2.5) [D] | behind: the "ejection hitbox behind DK" |
| Final hit | **62** | 0 | **4** | 361 (0) | BK **60** | **157** | 8.0 → 10 | top (11.0, 20.0 to -4.0) [D] | **H 2.0** |

H 0.8 on hits 2-6 [R]. Total 5 + 5 x 1.4 + 4 = **16%** [D] (the changelist's "16% (grounded)" agrees).

| Grounded rules | Data |
| --- | --- |
| **Super armor** | **5-17** (6.0.0; was 8-17) [R]; "a vulnerable gap in between his super armor and the active hitbox (2 frames)" (dragdown); super, not knockback-based [R] |
| Lower arm intangible | **19-24** [R] |
| FAF | **105** (animation 114) [R] |
| Movement | the reference's move parameters: **ground acceleration 0.034**, **ground friction 0.015**, **maximum ground speed 1.1**, **ground friction after the last hit 0.04** [R] (→ 0.00425, 0.001875, 0.1375, 0.005 world [D]); steered by the stick ("if used near ledge, always hold in", dragdown) [R reading: the acceleration follows the stick] |
| Windboxes | "immune to most windboxes during the move" (ssbwiki.com; no frames given) [R text] |
| Off an edge | not published (gap 7) |

**Aerial**: a rising, drifting spin with fourteen hits; ends helpless.

| Part | Frames | id | Dmg | Angle | BK | KS | Radius | Bone (y, z) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Hit 1 | **4-6** | 0 / 1 | 5 | 85 (0) | 50 | 10 | 7.7 → 10 | top (12.0, -7.0) / top (12.0, 4.0) [D] |
| Hits 2-13 | **10-12, 13-14, 15-17, 18-19, 20-21, 22-24, 25-26, 27-29, 30-31, 32-33, 34-36, 37-38** | 0 / 1 | **1** | **367** (0) | 55 | 20 | 5.0 → 6 | arml / armr (x 6.0) [E] |
| | | 2 / 3 | 1 | 367 (0) | 55 | 20 | 3.7 → 5 | arml / armr (x -4.0) [E] |
| Final hit | **39-43** | 0-3 | **2** | 361 (0) | **72** | **80** | 5.0 / 3.7 → 6 / 5 | as hits 2-13 [E] |

Shield +1 on every aerial box (200 / 150 / 120 [D]). Total 5 + 12 + 2 = **19%** [R]. Each of hits 2-13 is a separate hit on its own
frames: a fresh hit at the start of each listed span ("hits once every 1, 2, or 3 frames, seems inconsistent", UFD) [R].

| Aerial rules | Data |
| --- | --- |
| Lower arms intangible | **10-38** [R] (no whole-body intangibility in 13.0.1: "the aerial version no longer grants intangibility on frames 3-6") |
| Animation | **70** frames, then the special fall [R]; UFD "total" not given, landing lag column **38** |
| Movement | **initial vertical speed 0.9**, **air acceleration 0.05**, **maximum air speed 1.7**, **gravity multiplier 0.07** [R] (→ 0.1125, 0.00625, 0.2125 world [D]); "good horizontal distance, but very little vertical distance" [R]. Whether 0.07 multiplies the body's gravity (0.085 x 0.07 = 0.00595) or is the gravity itself is not stated (conflict 6) |
| Ledges | **both sides from frame 14** [R] |
| Landing during the move | "If Donkey Kong lands from the aerial version before undergoing helplessness, he transitions to **frame 63 of the grounded version**" (so 42 more frames to the grounded FAF 105) [R] |
| Special fall | landing lag **38** ("Helplessness landing lag 38 frames"; the landing animation's interruptible 39) [R]; the special fall's drift is not published (the shared drift [E]) |

### Hand Slap (down special) [R]

**Grounded**: slaps the ground on both sides; **hits only grounded targets**; repeats while special is pressed. **It does not bury**
(no `effect=Bury`: launches at 78 / 84) [R].

| Part | Data |
| --- | --- |
| Boxes (every slap) | four boxes, ids 0-3: **14%**, angles **78 / 84 / 84 / 78** (0), **BK 85, KS 30**, r **10.3** → 13, on the root; **grounded only** (`a=f`); **H 0.5**; shield +1 (107 [D]); **shieldstun x0.58**; **no clank** (`clang=f rebound=f`); `direct=f` |
| Box positions | "distance between grounded hitboxes **15u**", "grounded hitbox offsets from Donkey Kong **[0, 5, 6.1]**" [R]: read as y 5 and the inner boxes at z ±6.1, the outer at ±21.1 (the outer pair at 78, the inner at 84: "inner hitboxes launch more vertically", dragdown) [D reading] |
| Timing | loop points **8-9** and **34-35** (a **26-frame** loop); slaps on **12-13** and **23-24** (the loop's 4-5 and 15-16); the ending: FAF **47** (13 into the ending), animation 69 (35) [R] |
| Repeats | "can register subsequent button presses at any point during the startup or loop portions to repeat the slaps. During frames **20-26 of the loop** portion, the slaps are repeated twice even if the button is only pressed one more time" [R]; "each additional rep takes another 26 frames ... maximum reps is indefinite" (dragdown) [R] |
| Rehit | a target is hit by each slap: two hits a loop (the 12-13 and 23-24 slaps are separate hits) [R reading of "two hits", dragdown "Ground First Hit / Ground Second Hit"] |
| Counters | "will trigger every counter except [two other fighters'] even if they do it in the air" (an oddity of `direct=f`) [R], not modelled |
| Edge cancel | "can be edge canceled ... by holding forward with sufficient momentum near an edge" [R]: the engine's edge behaviour to check |

**Aerial**: two claps, the second a meteor; 10 frames of landing lag.

| Part | Frames | Dmg | Angle | BK / FKV | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Hit 1 | **19-21** | 5 | **90** (0) | FKV **36** | 100 | 7.0 → 9 | handr (4.0, 0, 2.0) [E] | trip 20; no clank |
| Hit 2 | **28-30** | 6 | **270** (0) | BK 30 | 100 | 8.0 → 10 | handl (4.0, 0, -2.0) [E] | trip 20 (only on a grounded target, "only possible if hit 1 missed"); no clank; "the entire swing of the second hit is a spike" |

| Aerial rules | Data |
| --- | --- |
| FAF | **50** (animation 69) [R] |
| Landing lag | **10** frames ("Landing lag 10 frames"; the landing animation 10) [R] |
| Air physics | not published (gap 8); B-reversible and wavebounceable (the shared rules) [R] |
| Shield | -3 on shield at best [R] |

## Bury (the reference's general rules) [R]

Sources: ssbwiki.com `Bury` (the effect) and `Buried` (the state), current-game sections. This port's only bury move is Headbutt.

| Rule | Value |
| --- | --- |
| Who | a fighter **on the ground** hit by a bury hitbox is buried; one **lying down** (a knockdown) **cannot** be buried (except after one other fighter's throw); one that **tripped can**. A fighter that cannot be buried takes the hit's knockback as usual [R] |
| Weight | bury hitboxes are **weight-independent** (knockback worked at weight 100) and **ignore subtractive knockback-based armor** [R] |
| Grabs | a buried fighter **cannot be grabbed** [R] |
| Duration | **`10 + 30h + 15(3 - r) + 0.5p + 1.5k`**, rounded **up**; with even stocks (h = r = 0) **`55 + 0.5p + 1.5k`** [R]. **p**: the victim's percent **after** the hit, with the 1.2 one-on-one multiplier; **k**: the hit's knockback at weight 100. **h** (the user behind): 0.25 one stock behind, 0.375 two or more (+8 / +12 frames [D]); **r** (the user ahead): 1.5 one stock ahead, 1.8 two or more (-18 / -22 frames [D]) [R] |
| Mash | **8 frames per stick input, 14.4 per button input** (buttons cannot be entered as quickly) [R] |
| End | the buried fighter **jumps out automatically** with **12 frames** of ending lag; **no invincibility** on escape (bury moves are combo starters) [R]. The pop-out's speed and height are not published (gap 9) |
| Hit while buried | a hit dealing **at least 90 units** of knockback **unburies** the victim and launches it with the **full** knockback [R]. A weaker hit does not unbury: its damage applies and its **knockback is stored and applied when the bury ends** (jabs, meteors, even other status effects) [R]. Several weaker hits: which knockback is kept is not published (gap 10) |
| Hurtbox | buried upright (Headbutt's pose); the visible part is the upper body; the hurtboxes move with the pose [R]; the exact boxes are not published [E] |
| Visual cues | at 30% of the duration small dirt splashes, at 60% a yellow flash (presentation) [R] |
| Platforms | the bury effect works on soft platforms too (only the "plunge" variant drops through) [R] |
| Removed glitch | a stored-knockback tech glitch was patched out in 13.0.0 [R]: nothing to model |

Headbutt check [D]: 10% at 0% (12 after the multiplier), k = the grounded box's knockback at weight 100 with BK 40 KS 40: the bury lasts
`ceil(55 + 6 + 1.5k)` frames before mashing.

## Armor (the reference's general rules) [R]

Source: ssbwiki.com `Armor`, dragdown's notes.

| Kind | Rule | This kit |
| --- | --- | --- |
| **Super armor** | no flinch and **no knockback** from any harmful hit; **damage and hitlag** are still taken; **flinchless hitboxes (windboxes) still move** the fighter; **grabs ignore all armor** [R] | Giant Punch full charge (9-20 / 15-20), Headbutt (5-14), grounded Spinning Kong (5-17) |
| **Damage-based armor** | no flinch unless the hit's damage (before the one-on-one multiplier) exceeds a threshold; with several hits, `(sum of earlier base damage x 1.2) + this hit's base damage > threshold` breaks it [R] | none (built for another port: `ARMOR_DAMAGE`) |
| **Knockback-based armor** ("heavy armor") | no flinch unless the knockback reaches a threshold in units. **Subtractive**: the threshold is taken off the knockback and the rest is taken; **non-subtractive**: full knockback once the threshold is passed. Knockback is computed before the one-on-one multiplier, so the multiplier does not affect it; staleness and rage do [R] | the cargo hold (conflict 2: threshold 16 or 80; subtractive or not is not stated: gap 11) |

## Conflicts and how they are resolved

1. **Jab 2's active frames**: the hit page's table says **4-6**, FAF 32; the 13.0.1 patch note says "5-6 → 5-7" and "FAF 35 → 32". UFD,
   dragdown and kuroganehammer all give **4-6**, and the quickest loop (11-13 from jab 1's frame 8) needs 4-6 [D]. Resolved to **4-6**
   (the patch note counts from a different first frame).
2. **The cargo hold's knockback armor**: ssbwiki.com "resist up to **16** units of knockback"; dragdown "tank up to **80** total knockback
   units (i.e. any move that induces tumble)", and dragdown's frame table "full body knockback armor **16+**" (from frame 16). The 16 in
   ssbwiki.com's text may be the frame misread as a threshold. **Unresolved**; proposed **80** (two dragdown statements agree with the
   frame-16 start, and 80 is the tumble threshold) [E]. Footage settles it (a jab and a tilt into the cargo hold).
3. **When the charge is full**: 110 frames of charging (ssbwiki.com, the patch note), full on move frame **121** (patch note), UFD
   "**111** frames to reach full charge", and the charge loop entered on 7-8 (110 + 8 = 118, not 121). Resolved: **110 frames of charge
   counted from the loop's start** [R], the frame it becomes full [E] until footage; the damage formula only needs the charge frames.
4. **Headbutt's ledge window**: "after frame 2" (ssbwiki.com) vs "every frame" (dragdown). Proposed **from frame 2** [E: the stricter
   statement]; footage settles it.
5. **The carry's walk**: ssbwiki.com "walk speed and acceleration x1.1" against the parameter file's heavy-item lift values (speed 1.3246,
   accel 0.03 + 0.0976923, which are slower than his walk). Proposed: **x1.1 of the walk** [R, the move page], the lift values recorded;
   footage (a timed walk while carrying) settles it.
6. **Aerial Spinning Kong's gravity**: "gravity multiplier 0.07" (a multiplier of 0.085, so 0.00595, would make the 70-frame spin rise
   about 48 reference units: more than a full hop, against "very little vertical distance"; as the gravity itself, 0.07 makes it peak
   after 13 frames about 6 units up and then sink). **Unresolved**; proposed **0.07 as the gravity** [E] (it matches "very little vertical
   distance ... prone to edgeguarding") pending footage, with the fall capped at the body's fall speed [E].
7. **Spinning Kong's grounded totals**: dragdown "total 104" (FAF 105) agrees with ssbwiki.com; UFD "104" too. No conflict (noted because
   kuroganehammer lists FAF 105 only on hit 1).
8. **Throw weight dependence**: the changelist says the throws "are no longer weight-dependent" (relative to the earlier game). Read as
   their timing (fixed release frames for every target, which is what the engine does); none of the throw rows has `setweight`, so the
   knockback uses the victim's weight. Resolved: the rows as published, **no set weight** [R reading]; footage on a light and a heavy
   target settles it.
9. **Heavy landing lag**: the parameter file's `landing_heavy_frame` 4 → 3 frames by the house rule; kuroganehammer's "hard landing lag 5"
   counts differently (its "soft landing lag 2" is `landing_frame_light` 1 + 1). Resolved to the house rule (3), as for every body.
10. **Pummel timing**: UFD "2 / 21 including 15 frames of hitlag" vs ssbwiki.com hit 2, FAF 8. The same thing (21 - 15 + 2 = 8 with
    the frame counting) [D].
11. **The roster note's "Hand Slap: burying?"**: in 13.0.1 the grounded Hand Slap **does not bury** (no bury effect, 78 / 84 degree
    launches) [R]. Corrected.

## Gaps (needed for 1:1, not published)

1. Giant Punch's charge-loop frame mapping (which move frame each of the 110 charge frames is), and whether the release's frame 1 is the
   frame special is pressed.
2. Giant Punch's turnaround window for the charge (start) and the punch.
3. The cargo hold's jump: ground jump only or a midair jump after walking off, its height and air speed; fast fall; air drift.
4. Whether the cargo hold can grab a ledge (and what happens to the victim if it does).
5. The cargo release: DK's lag and the victim's escape motion when the timer runs out or the victim mashes free (13.0.1).
6. Giant Punch's air physics (charge and punch) and its landing during the charge.
7. Grounded Spinning Kong going off an edge (does it carry on as the aerial version, end, or stop at the edge?).
8. Aerial Hand Slap's air physics.
9. The bury pop-out's speed and height, and the stored knockback's application when the bury ends.
10. Several weak hits on a buried fighter: whether stored knockback adds up or the last one is kept.
11. Whether the cargo hold's knockback armor is subtractive.
12. The aerial Spinning Kong's special-fall drift; whether the 70-frame spin keeps the move's air acceleration until the fall.
13. Who is knocked out first when both fall past the blast line during a carry (and whether the carry lets go first).
14. Grounded Spinning Kong's "immune to most windboxes": the frames and which pushes.
15. The body's hurtbox size (the reference's is "bigger than his model would suggest"): no numbers.

Footage that would settle most of them (training mode, frame-stepped, hitbox view): a Giant Punch charged from empty with a frame
counter, cancelled each way, and hit during the charge; the cargo hold walked a timed distance, jumped from the ground and after walking
off, next to a ledge, hit by a jab and a tilt, and mashed out at 0%; the aerial Spinning Kong from a ledge (height over time) and the
grounded one run off an edge; Headbutt next to a ledge from frame 1; a Headbutt bury at 0% and at 100%, hit while buried by a jab and by
a smash; the aerial Giant Punch and Hand Slap from a short hop.
