# Reference data for the sixth moveset (the reference game's Palutena)

Mechanical reference for the sixth port, collected 2026-10-13 for version **13.0.1**, before anything was built. Proposed in-game
names (registered in sim 67, owner decision 19): the **Staff** class and weapon (`staff`) on the **seer** body, built-in fighter **Seer**. The
reference character's name is not used in the game. What the engine can and cannot do with this data is in `PROJECT_AUDIT.md`; the
format follows `docs/movesets/captain_falcon/REFERENCE_DATA.md`, `docs/movesets/mario/REFERENCE_DATA.md` and `docs/sheik/REFERENCE_DATA.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Palutena_(SSBU)/<move>` (29 pages: `Neutral_attack`, `/Hit_1`, `/Infinite`, the three tilts,
  `Dash_attack`, the three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(back|front|trip)`,
  the four specials), read as **raw wikitext** (`index.php?title=...&action=raw`), so every hitbox value comes from a named template
  field (`damage`, `angle`, `af` angle type, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff` hitlag multiplier, `sdi`, `sd` shield
  damage, `rehit`, `setweight`, `flinchless`, `g=f` / `a=f`, `clang`, `hitbits`, `reflectable`) and the angle-361 column shift cannot
  happen. The specials' pages also give the move **parameters** (speeds, offsets, multipliers, lifetimes) from the move files.
  `Palutena_(SSBU)` for the attributes table, the moveset summary and the update history; `Template:UpdateList_(SSBU)/<version>` for the
  per-version change lists. https://www.ssbwiki.com/Palutena_(SSBU)
- **ssbwiki.com move articles** `Autoreticle`, `Explosive_Flame`, `Warp`, `Counter_(Palutena)`, `Reflect_Barrier`, and the general
  pages `Counterattack`, `Ledge-canceling` and `Dash` (current-game table), for behaviour the tables do not show.
- **ultimateframedata.com/palutena** (startup, total frames, landing lag, autocancel, active frames, notes; dodges, jump air times,
  stats). UFD's "total frames" is FAF - 1. https://ultimateframedata.com/palutena
- **dragdown.wiki `SSBU/Palutena/Data`** (the parameter file and a second frame-data table). https://dragdown.wiki/wiki/SSBU/Palutena/Data
- **kuroganehammer.com/Ultimate/Palutena**: answered this time, but its page is labelled **version 5.0.0**, so it is used only as a
  cross-check of values that did not change after 5.0.0 (it still shows the pre-7.0.0 up tilt and neutral air values and the pre-11.0.0
  dash attack and forward air: consistent with the patch notes, which confirms the rest). https://kuroganehammer.com/Ultimate/Palutena

**Update history applying to 13.0.1** (ssbwiki.com, every entry for this character): **2.0.0** neutral air hitlag 1x / 2x (loop / last)
→ 0.5x / 1.5x; "can grab ledges more easily during Warp" (no numbers published). **3.0.0** Autoreticle shield damage 0 → -1.7; Explosive
Flame 0 → -0.7 (loop) / -2.7 (last); the "Super Warp Glitch" removed. **7.0.0** shield size up (shields are global here: not modelled);
up tilt last hit 2.5% BK 60 KS 195 → **4.5% BK 75 KS 136**; down smash sweetspot BK 40 → **52**; neutral air last hit BK 30 KS 180 →
**35 / 155**, its aerial-only box 11.5 → **9.8**; neutral air's late loop hits get **-13 hitstun**; down throw KS 25 → **30**. **8.0.0**
the super move only (not ported). **11.0.0** dash attack FAF 38 → **41**; forward air landing lag 7 → **9**; standing grab reach (Z2
11.4 → **9.4** grounded, 13.2 → **11.2** aerial). Nothing after 11.0.0. Every table below is the post-11.0.0 data the pages show
(the moveset summary on the main page still lists the old 2.5% up tilt hit; the move page and the patch note agree on 4.5%). No data
from the 3DS/Wii U game or earlier was used; where those games' behaviour differs (the moving Reflect Barrier) it is called out.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift, a reading of a field);
**[E]** estimate. Hitbox **positions** on a bone other than `top` (`stick`, the staff; `bust`, the chest) are [E]: the reference attaches
them to its own skeleton, so ours go on our body where that bone is in the move's pose. Offsets on `top` (the character's root at the
feet: `zpos` forward, `ypos` up) convert directly [D]. A reference "capsule" (`zpos=8.0 to 13.0`) is two or three circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first, FAF as published); the sim stores them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius and position in world tenths = reference value x 1.25, rounded half up [D] (the
  house rule). Speeds: reference units a frame / 8 = world units a frame [D].
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; **361** is the reference's 361 angle ("angle 361"), **367**
  the autolink angle. H = hitlag multiplier (`ff`), SDI = SDI multiplier.
- **Angle type** (`af`): 0 (unset) by position (away from the hitbox's owner), 3 forward (the owner's facing), 4 backward. This port
  has many type 3 and 4 boxes that reach both sides of the body (the spinning staff) and a stationary projectile whose type-0 loop hits
  must send *toward its own centre*; see the audit (E27).
- **Targets**: `a=f` cannot hit aerial targets (grounded only), `g=f` cannot hit grounded targets (aerial only) [R].
- **Shield damage**: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd`
  becomes `100 x (damage + sd) / damage`, rounded [D]. Negative `sd` (the projectiles) gives less than 100.
- `clang=f rebound=f`: never clanks, never rebounds [R]. `flinchless=t` (windboxes): the reference's flinchless hit, ours `Hitbox::push`
  (E2) [D].
- `rehit=N`: the hitbox hits the same target again every N frames while it is out (ours `Move::rehit`) [R].

## Attributes [R]

ssbwiki.com attributes table, agreeing with UFD, the parameter file and kuroganehammer.com:

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 91 | 1.271 | 2.035 | 2.077 | 0.106 | 1.0 | 0.01 + 0.105 | 0.01 | 0.12 | 1.55 / 2.48 | 35.9 / 17.3 / 35.9 | 3 | 1 |

Parameter file (dragdown) [R]: `walk_accel_add` 0.105, `walk_accel_mul` 0.21 (ours: `walk_accel` 0.105, `walk_accel_stick` 0.21);
`run_accel_add` 0.044, `run_accel_mul` 0.12991 (ours: `run_accel` 0.044, `run_accel_stick` 0.12991; ssbwiki.com's Dash table agrees:
base 0.044, additional 0.12991, max 0.17391); `ground_brake` 0.106; initial dash **10 frames**, pivot dash 12 (ssbwiki.com Dash table);
`jump_initial_y` 19.745; `jump_speed_x` 0.72, `jump_speed_x_mul` 0.7, `jump_speed_x_max` 1.1, `jump_aerial_speed_x_mul` 0.9;
`air_brake_y` 0.01; `air_ground_speed_brake` 0.03; `landing_frame_light` 1, `landing_frame` 3, **`landing_heavy_frame` 7** (so **6** frames
of heavy landing lag here [D], by the house rule `landing_heavy_frame - 1`; conflict 1); `landing_attack_air_frame` n / f / b / hi / lw
**12 / 9 / 10 / 14 / 11** (agreeing with UFD and ssbwiki.com's landing timings, which give interruptible 13 / 10 / 11 / 15 / 12);
`cliff_jump_y` 37.95, `cliff_jump_x_speed` 0.6 (the ledge jump); `jump_count_max` 2; `scale` 0.95; `shield_radius` 13.7 (shields are
global here, not used); `jostle_front` 0.8, `jostle_back` 1.2 (no jostling in the engine); **no wall jump** (`wall_jump_type` False,
`attach_wall_type` False); the wall-tech jump numbers `passive_wall_jump_x_speed` 1.3, `passive_wall_jump_y_speed` 2.2,
`wall_jump_enable_x_speed` 0.2, `passive_wall_x_speed` 0.5 (ours: `wall_jump` 0, `wall_jump_vx` 1.3, `wall_jump_vy` 2.2, which since sim
64 are also the wall-tech jump [D]). Footstool (`tread_*`) not modelled for anyone.

Jump air times (UFD; kuroganehammer.com agrees on SH / FH) [R]: short hop **34**, full hop **50**, short hop fast fall **24**, full hop
fast fall **35**.

Dodges (UFD total frames, so FAF one more; kuroganehammer.com gives the FAF and agrees) [R]: spot dodge FAF 26 (intangible 3-17), forward
roll FAF 30 (4-15), back roll FAF 35 (5-16): the shared values. Neutral air dodge FAF **50** (intangible 3-30, landing lag 10);
directional down **71**, diagonal down **78**, sideways **83**, diagonal up **94**, up **106** (intangible 3-21; landing lag 11-19).
UFD's own numbers (re-read 2026-10-13, as every other body's fields take them): spot dodge "20/25", forward roll 29, back roll 34,
neutral air dodge 49 (landing lag 10), down 70, diagonally down 77, sideways 82, diagonally up 93, up 105: the body uses these (the
engine's dodge lengths count as UFD's do, as for the tumbler and the sprinter) [R].

Body size [E]: the reference model is tall (ssbwiki.com: "a tall middleweight"; the up tilt's top box is 23 units up and the counter's
bubble is centred 10 up); our seer body needs a taller hurtbox than the tumbler's (`ecb_height`), fitted to those heights.

## Normals

All frame numbers [R] (ssbwiki.com timing tables; UFD, dragdown and kuroganehammer.com agree on every startup and FAF unless listed
under "Conflicts"). Damage, angle, BK, KS, FKV, H, SDI [R]. Radius column: reference → world tenths [D]. Positions in reference units
on the named bone.

### Jab [R]

Palutena's jab is **one hit, then the rapid jab** (no second or third jab).

| Part | Frames | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (offset y, z) | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Jab 1 | 8-9 | 25 | 0 | 3.0 | 361 (3) | 25 | 20 | 2.2 → 3 | top (8.5, 5.0) [D] | 1.6 | **+4 hitstun** (every id) |
| | | | 1 | 3.0 | 361 (3) | 25 | 20 | 2.2 → 3 | top (8.5, 8.0) [D] | 1.6 | |
| | | | 2 | 3.0 | **180** (3) | 20 | 20 | 2.5 → 3 | top (8.5, 12.0) [D] | 1.6 | fighters only: pulls the target in |
| | | | 3 | 3.0 | 361 (3) | 20 | 20 | 2.5 → 3 | top (8.5, 12.0) [D] | 1.6 | the same box for objects (id 2 wins on fighters); not modelled |
| Rapid jab loop | its 4, 7, 10, 13, 16, 19, 22 (7 a cycle), loop points 1-2 and 23-24 | | 0 | 0.6 | 361 (3) | 10 | 20 | 6.0 → 8 | top (8.5, 8.0 to 13.0) [D] | 0.5 | SDI 0.4, **shieldstun x7**, no clank |
| Rapid jab finisher | its 3-4 | its **40** (animation 49) | 0 / 1 | 3.5 | 361 (3) | 55 | 140 | 6.0 → 8 | top (9.0, 10.0) and (9.0, 16.0) [D] | 2.0 | no clank |

**Entry rule** [R] (ssbwiki.com `Hit_1`): jab 1's continuation window is **10-24** when attack is pressed repeatedly: "the move
transitions into the rapid jab if the button has been pressed and released at least 2 times since its initiation"; and **held on hit:
13** (holding attack, jab 1 having connected, goes into the rapid jab on frame 13). Jab 1 has no other follow-up. **Exit rule** [R]:
"after one full loop, releasing the attack button at any point" goes to the finisher.

**Quickest loop from jab 1's frame 1** [R]: jab 1 on 8-9; rapid jab hits on 13, 16, 19, 22, 25, 28, 31; finisher 35-36; FAF 25 (jab 1
alone) / 72 (finisher). The tests should check exactly this. (dragdown's 2, 5, 8 ... and UFD's 4/7/10 are the rapid jab's own
numbering: consistent [D].)

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle (type) | BK / FKV | KS | Radius | Bone (offset y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt hit 1, early | 14-16 | 60 | 1 | 6 | 90 (3) | 0 / **FKV 25** | 100 | 4.1 → 5 | stick (5.5, 0) [E] | **set weight**; H 0.8; SDI 0; shield +1 (117 [D]); no rebound |
| | | | 0 | 6 | 45 (3) | 0 / FKV 45 | 100 | 3.0 → 4 | stick (-5.5, 0) [E] | set weight; as above |
| Forward tilt hit 1, mid | 17-20 | | 1 | 6 | **285** (3) | 0 / FKV 25 | 100 | 4.1 → 5 | stick (5.5) [E] | set weight; a set-knockback meteor angle (kuroganehammer.com and dragdown agree) |
| | | | 0 | 6 | 100 (3) | 0 / FKV 40 | 100 | 3.0 → 4 | stick (-5.5) [E] | set weight |
| Forward tilt hit 1, late | 21-23 | | 1 | 6 | 20 (3) | 0 / FKV 35 | 100 | 4.1 → 5 | stick (5.5) [E] | set weight |
| | | | 0 | 6 | 85 (3) | 0 / FKV 30 | 100 | 3.0 → 4 | stick (-5.5) [E] | set weight |
| Forward tilt hit 2 | 24-35 | | 1 / 0 | 7 | 40 (3) | 78 | 80 | 4.8 / 3.9 → 6 / 5 | stick (5.5 / -5.5) [E] | a separate hit (part 1); H 1.0; shield +1 (114 [D]). **Lower (right) arm intangible 14-35** (partial: E19) |
| Up tilt, loop | 8-22, **rehit every 3** (hits on 8, 11, 14, 17, 20) | 56 | 0 | 1.4 | **367** (autolink) | 0 / FKV 20 | 100 | 3.0 → 4 | top (18.0, 4.0) [D] | H 0.5, SDI 0.3, shield +1 (171 [D]) |
| | | | 1 | 1.4 | 130 | 0 / FKV 50 | 100 | 2.0 → 3 | top (15.0, 10.5 to -5.0) [D] | |
| | | | 2 | 1.4 | 190 | 0 / FKV 30 | 100 | 2.0 → 3 | top (20.0, 10.5 to -5.0) [D] | |
| Up tilt, last hit | 24-25 | | 0 / 1 | **4.5** | 85 | **75** | **136** | 3.0 → 4 | top (23.0 / 17.0, 10.5 to -5.0) [D] | H 1.3, shield +1 (122 [D]); total 11.5% |
| Down tilt, early | 14 | 40 | 0 | **8.5** | 35 | 40 | 100 | 2.7 → 3 | stick (8.0 to -7.0) [E] | **grounded targets only**; shield +1 (112 [D]) |
| | | | 1 | **5** | 75 | 38 | 100 | 2.7 → 3 | stick (8.0 to -7.0) [E] | **aerial targets only**; shield +1 (120 [D]) |
| Down tilt, late | 15-27 | | 0 / 1 | as early | | | | as early | | as early |
| | | | 2 | 8.5 | 35 | 40 | 100 | 2.5 → 3 | top (1.0 to 2.0, 19.5 to 9.8) [D] | grounded only: the staff along the floor |
| | | | 3 | 5 | 75 | 38 | 100 | 2.5 → 3 | top (1.0 to 2.0, 19.5 to 9.8) [D] | aerial only |
| Dash attack, clean | 6-9 | **41** (11.0.0) | 0 | 11 | 70 (3) | 71 | 76 | 4.5 → 6 | top (9.0 to 11.0, 13.0) [D] | shield +3 (127 [D]); no clank |
| | | | 1 | 11 | 60 (3) | 71 | 76 | 3.5 → 4 | top (10.0, 5.0 to 9.0) [D] | |
| Dash attack, late | 10-17 | | 0 | 7 | 60 (3) | 61 | 56 | 2.5 → 3 | top (9.0 to 12.0, 10.0) [D] | shield +3 (143 [D]). **Head and torso invincible 5-6, left arm invincible 5-16** (partial: E19) |

Forward tilt hit 1 has two ids (the staff's head and tail, `stick` ±5.5) in one hit group across its three phases; hit 2 is its own
group (part 1). The phases are the reference's "early / mid / late" rows, all set knockback with set weight, so they combo the same into
hit 2 at any percent and weight [D]. kuroganehammer.com notes "right arm intangibility 14-35" for the same window [R].

### Smashes [R]

| Move | Frames | Charge hold | FAF | id | Dmg | Angle (type) | BK / FKV | KS | Radius | Bone (offset y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash | 18-20 | 10 | 64 | 0 | **16** | 361 (3) | 40 | 100 | 5.1 → 6 | top (11.0 → 9.9, 6.0 → 11.9) [D] | the inner wing; wins where both reach; shield +1 (106 [D]); no clank |
| | | | | 1 | 13 | 361 (3) | 40 | 102 | 5.1 → 6 | top (11.0 → 8.5, 6.0 → 18.5) [D] | the wing tips; shield +1 (108 [D]) |
| Forward smash windbox, early | 21-25 | | | 0 | 0 | 361 (3) | FKV **60** | 100 | 12.0 → 15 | top (10.0, 8.0 to 25.0) [D] | **flinchless push** (E2), no hitlag (`ff` 0) |
| Forward smash windbox, late | 26-31 | | | 0 | 0 | 361 (3) | FKV **35** | 100 | 12.0 → 15 | top (10.0, 8.0 to 40.0) [D] | push |
| Up smash, clean | 18-29 | 12 | 64 | 0 / 1 / 2 | **16 / 12 / 9** | 88 (3) | 53 / 58 / 62 | 84 / 89 / 92 | 4.5 / 2.8 / 2.8 → 6 / 4 / 4 | top (21.0 / 44.0 / 65.0 down to 4.0, z 9.7) [D] | a pillar of light in front: three columns from the floor up; shield 0; no clank |
| Up smash, late | 30-35 | | | 0 / 1 / 2 | 13 / 9 / 7 | 88 (3) | 53 / 58 / 62 | 84 / 89 / 92 | 2.8 → 4 (all) | top (22.0 / 44.0 / 65.0 down to 2.0 / 4.0 / 4.0, z 9.7) [D] | |
| Down smash, clean | 17 | 13 | 69 | 0 | **15** | 60 | **52** (7.0.0) | 103 | 3.7 → 5 | top (9.2, 7.3 to -7.3) [D]: both sides | H 1.4; no clank; angle by position |
| Down smash, late | 18-19 | | | 0 | 13 | 28 | 30 | 90 | 6.0 → 8 | top (4.0, 14.0 to -14.0) [D] | H 0.85 |
| Down smash windbox | 20-29 | | | 0 | 0 | 361 | FKV **55** | 100 | 9.0 → 11 | top (4.0, 24.0 to -24.0) [D] | push, both sides |

Charge hold frames: ssbwiki.com "charges between 10-11 / 12-13 / 13-14", UFD "charge hold is frame 10 / 12 / 13" [R]: `charge_at` on
the reference's frame 10 / 12 / 13 [D]. Charged smash damage is the shared rule (up to 1.4x over 60 frames), agreeing with dragdown's
charged values (24 / 19.5 forward, 24 / 18 / 13.5 up, 22.5 / 19.5 down) [R].

### Aerials [R]

| Move | Frames | Autocancel | Landing lag | FAF | Phase / id | Dmg | Angle (type) | BK / FKV | KS | Radius | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Neutral air** | loop 5-27, **rehit every 4** (hits on 5, 9, 13, 17, 21, 25); last hit 29-30 | 1-3, 40+ | **12** | 52 | early (5-6) 0 / 1 | 1.4 | **367** | 0 / 0 | 100 | 4.2 → 5 | stick (5.2 / -5.4) [E]; **aerial targets only**; H 0.5, SDI 0.3, shield +1 (171 [D]) |
| | | | | | early 2 | 1.4 | 100 | 0 / **FKV 65** | 100 | 4.2 → 5 | stick (5.2) [E]; **grounded targets only** |
| | | | | | early 3 | 1.4 | **95** | 0 / **FKV 80** | 100 | 4.2 → 5 | stick (-5.4) [E]; grounded only (the stronger frames-5-6 pop-up: tumbles light fighters) |
| | | | | | mid (7-23) 0 / 1 | 1.4 | 367 | 0 / 0 | 100 | 4.2 → 5 | aerial only |
| | | | | | mid 2 / 3 | 1.4 | 100 | 0 / FKV 65 | 100 | 4.2 → 5 | grounded only |
| | | | | | late (24-27) 0 / 1 | 1.4 | 367 | 0 / **FKV 65** | 100 | 4.2 → 5 | aerial only; SDI **0**; **-13 hitstun** (7.0.0) |
| | | | | | late 2 / 3 | 1.4 | 100 | 0 / FKV 65 | 100 | 4.2 → 5 | grounded only; the scripts give -13 only to ids 0 and 1, but a known engine glitch applies it to ids 2 and 3 too, so **all four get -13** [R] |
| | | | | | last hit 0 | **5.1** | 55 (**3: always forward**) | **35** | **155** | **9.8 → 12** | stick (0, 0) [E]; aerial only; H 1.5; shield +3 (159 [D]) |
| | | | | | last hit 1 | 5.1 | 55 (3) | 35 | 155 | **11.5 → 14** | stick (0, 0) [E]; grounded only |
| Forward air | 9-11 | 1-3, 24+ | **9** (11.0.0) | 45 | 0 | 10 | 45 (3) | 31 | 92 | 5.5 → 7 | top (6.9, 9.0 to 15.0) [D]; shield +3 (130 [D]) |
| **Back air** | 8-10 | 1-2, 35+ | 10 | 50 | 0 | **12** | 361 (**4: backward**) | 30 | 95 | 7.0 → 9 | top (10.2, -14.0) [D]; H 1.2; shield +3 (125 [D]). **Head and torso invincible 7, left arm 7-10** (partial: E19) |
| **Up air** | loop 10-22, **rehit every 3** (hits on 10, 13, 16, 19, 22); last hit 24-25 | 1-2, 42+ | 14 | 52 | loop 0 | 1.0 | 105 (4) | 0 / FKV 40 | 100 | 4.0 → 5 | top (20.0, 6.0 to -6.0) [D]; **grounded only**; H 0.6, SDI 0.7 |
| | | | | | loop 1 | 1.0 | **367** (4) | 0 / FKV 20 | 100 | 4.0 → 5 | top (22.8, 0) [D]; **aerial only** |
| | | | | | loop 2 / 3 | 1.0 | 367 (4) | 0 / FKV 20 | 100 | 3.0 → 4 | top (18.0 to 22.0, ±4.0 to ±9.0) [D]; aerial only |
| | | | | | last hit 0 | **5.0** | 88 (4) | **50** | **152** | 6.0 → 8 | top (23.0, 0) [D]; H 1.5, SDI 0.7; total 10% |
| | | | | | last hit 1 / 2 | 5.0 | 88 (4) | 50 | 152 | 4.0 → 5 | top (18.0 to 23.0, ±4.0 to ±12.0) [D] |
| Down air | 10-11 | 1-2, 48+ | 11 | 59 | 0 | 11 | **270** (meteor) | 20 | 100 | 4.2 → 5 | top (-3.7 to -5.0, 0.5) [D]; **aerial only**; H 1.5; no clank |
| | | | | | 1 | 11 | 55 (4) | 30 | 100 | 6.3 → 8 | top (-5.0, 1.5 to -0.5) [D]; **grounded only**; H 1.5 |

Landing lag here = `landing_attack_air_frame` (dragdown) = ssbwiki.com's landing "interruptible" - 1 = UFD's landing lag [D].
The neutral air's autocancel at 40+ means it autocancels from a full hop (ssbwiki.com) [R]. The up air's last-hit `rehit=3` cannot
hit twice (two frames) [R]. Down air disables jostling on 8-11 (no jostling here).

**What the neutral air must do** (ssbwiki.com's description, consistent with the table) [R]: the aerial loop hits at the autolink angle
with BK 0 / KS 100 pull an airborne target along and, at higher percent and rage, carry it down (drag-down); on 24-27 they switch to set
knockback with -13 hitstun so a late drag-down gives no advantage; grounded targets are popped up by set knockback (weight-dependent);
the **last hit always sends the way Palutena faces** (angle type 3), so holding forward keeps the target in front.

### Grabs, pummel and throws [R]

| Move | Frames | FAF | Data |
| --- | --- | --- | --- |
| Standing grab | 7-8 | 40 | id 0 r 3.6 → 5, top (8.0, 4.0 to **9.4**) [D] (11.0.0), grounded targets only; id 1 r 1.8 → 2, (8.0, 2.2 to 11.2), aerial targets only |
| Dash grab | 10-11 | 48 | id 0 r 2.9 → 4, top (8.0, 4.0 to 13.1); id 1 r 1.45 → 2, (8.0, 2.55 to 14.55) |
| Pivot grab | 11-12 | 43 | id 0 r 3.6 → 5, top (8.0, -4.0 to -18.9); id 1 r 1.8 → 2, (8.0, -2.2 to -20.7) (written in front: the engine turns a pivot grab round [D]) |
| Pummel | 1 (ours 2) | 7 (ours 8) | 1.3%, 361 (3), BK 0, KS 100, **FKV 30**, r 5.0 → 6, top (9.0, 10.0) [D], H **2.1**, no clank. The engine's pummel cannot hit on its first frame, so the hit and FAF are one frame later [D] (the house rule). UFD's "19 including 13 frames of hitlag" agrees |
| Forward throw | release 20 | 45 | **9%, 45, BK 90, KS 30**. Thrower invincible 1-20 |
| Back throw | turns round 20, release 20 | 55 | **10%, 361, BK 65, KS 70** (the KO throw; angle 361 resolved by the shared rule, behind the original facing [D]). Invincible 1-20 |
| Up throw | release 17 | 55 | **8%, 92, BK 90, KS 60**. Invincible 1-17 |
| Down throw | release 25 | 46 | **5%, 73, BK 90, KS 30** (7.0.0). **On frame 24, 8 frames of hitlag for both fighters** (the held fighter invincible meanwhile); the frames above are without that hitlag, so in real time the release is 33 and the FAF 54 [D]. Invincible 1-25 |

Palutena's throws have **no hit on the held fighter before the throw** (plain throws: one throw row each, no `HIT_PUMMEL` pre-hit) [R].
Each also has the "interrupted" row (`kind=1`: 3%, 361, BK 60, KS 100), not modelled (as for Mario and Falcon). The thrower's invincibility
rows are not modelled (house limit).

### Ledge and floor [R]

| Move | Frames | Intangible | FAF | Data |
| --- | --- | --- | --- | --- |
| Ledge attack | 23-25 | 1-25 (1st grab; 1-20 2nd, 1-12 3rd, none from the 4th: the shared decay) | 56 | 9%, 45 (3), BK 90, KS 20, shield +1 (111 [D]), r 5.0 → 6, top (4.5, 9.2 to -5.2) [D] (both sides), no clank |
| Get-up attack, face up (ssbwiki.com "back") | behind 16-17, front 25-26 | 1-26 | 46 | 7%, 48, BK 80, KS 48, shield +8 (214 [D]), r 5.0 → 6; top (5.0, -15.0 to -4.0) then (5.0, 15.0 to 4.0) [D]; no clank |
| Get-up attack, face down (ssbwiki.com "front") | behind 14-15, front 24-25 | 1-25 | 46 | as face up; top (5.0, -13.0 to -4.0) then (5.0, 13.0 to 4.0) [D] |
| Get-up attack, trip | front 13-14, behind 24-25 | 1-7 | 50 | 5%, 361, BK 60, KS 50, shield +8 (260 [D]), r 4.5 → 6; top (4.5, 13.0 to 4.0) then (4.5, -13.0 to -4.0) [D] |

dragdown's "active 16-26" / "14-25" / "13-25" are the first to last active frame of the same two hits [D]. One get-up attack slot exists
(as for Mario and Falcon): the face-up one would be used.

## Specials

### Autoreticle (neutral special) [R]

| Part | Frames | Data |
| --- | --- | --- |
| Reticle search | **7-14** | a search box of **radius 120** (→ 150 tenths, 15 world units [D]) on `bust` (the chest) [R]. It locks onto an opponent inside it who is **in front** of Palutena, **not intangible or invincible**, with **no obstacle (platforms, items) in the way** (ssbwiki.com move article and moveset summary); if none is found, the animation plays and **no reticle and no shots** appear [R] |
| Reticle | from the lock | placed on the target's chest (`reticle target bone: bust`); **"once the reticle locks on, it stays in place"** (a fixed point in the stage) [R]. "Reticle x/y offset [-0.4, 0.4]" [R, meaning not stated: conflict 6] |
| Shots | spawned after **23 frames** and then every **6** (parameters); hitboxes **24-46, 30-52, 36-58** (UFD: 24 / 30 / 36) | each: **3.5%, 361 (3), BK 40, KS 41**, r 2.3 → 3, shield damage **-1.7** (51 [D]); **speed 6** (→ 0.75 world units a frame [D]), **lifetime 25**, not piercing (gone on its first hit), reflectable, absorbable; fired at the reticle ("fires three energy blasts at that location") |
| End | FAF **68** | animation 67 grounded, **74 aerial** (the aerial FAF is also 68: the last 7 frames are interruptible) |
| Movement | | **horizontal speed x0.6** when used (parameter). In the air it "briefly stalls her descent" (moveset summary): the stall's numbers are **not published** [gap] |

Not stated: how the shots fly (straight at the reticle point from the staff, or steering toward it), whether they stop at the point
or carry on (a lifetime of 25 at speed 6 covers 150 reference units, longer than the 120 search radius); which opponent is picked when
two are in range (a secondary source, Smashpedia, says "the nearest"); the exact "in front" rule (ssbwiki.com's range image, from the
earlier game, shows a region in front); whether pass-through platforms count as obstacles; landing during the aerial version
[gaps 1-4]. The shots **can lock** (hit a fighter lying on the floor, keeping it down) per the moveset summary [R]; **nothing in the
data trips** (no `trip` field on any hitbox of the kit).

### Explosive Flame (side special) [R]

A stationary explosion summoned at a distance; it is a projectile (`reflectable=t`), not a hitbox on Palutena.

| Part | Frames | Data |
| --- | --- | --- |
| Placement | **frame 1** (UFD: "area of explosion dictated on frame 1") | **x offset 45** in front (→ 5.625 world units [D]), **75 with a stick flick** ("if inputted like a smash attack": → 9.375 units [D]). The point is fixed where Palutena was **when the button was pressed** (a flash marks it); if she moves (falling, a moving platform) the explosion still appears there (ssbwiki.com move article). The height of the point is **not published** [gap 5]. "Unable to hit anyone directly in front of her" [R] |
| Loop hits | **28-55, rehit every 5** (hits on 28, 33, 38, 43, 48, 53 against a target inside all the time; UFD lists exactly those) | **1.5%, 160 (type 0: by position), BK 50, KS 100**, SDI 0.4, shield **-0.7** (53 [D]); radius **grows**: 4.8 → 6 (from 28), 6.0 → 8 (32), 7.2 → 9 (36), 8.4 → 11 (40), 9.6 → 12 (44), 10.8 → 14 (48), 12.0 → 15 (52); top (0, 0) of the flame [D]; no clank; reflectable |
| Last hit | **57** | **5.5%, 84 (3: the flame's facing), BK 60, KS 141**, r **15.5 → 19**, H **1.5**, SDI 0.4, shield **-2.7** (51 [D]); a separate hit (part 1): it hits a target the loop already hit. Total when everything connects: 6 x 1.5 + 5.5 = **14.5%** [D] |
| End | FAF **72** (animation 99) | |
| Movement | | **horizontal speed x0.8** on starting (parameter) |

The loop's angle 160 by position means a target on the far side of the centre is sent back toward the centre, and one on the near side
toward Palutena and the centre: the loop **holds targets in the middle of the explosion** until the last hit (ssbwiki.com: "its loop hits
reliably trapping the opponent") [D, from the angle and angle type]. Not stated: the aerial version's gravity (the parameter page lists
only the startup speed multiplier), landing during it, and whether the explosion is cancelled if Palutena is hit before frame 28 [gaps
5-7].

### Warp (up special) [R]

Every number below is a published parameter or timing; only the stage-collision behaviour and the ledge windows are not.

| Part | Frames | Data |
| --- | --- | --- |
| Startup | 1-15 | gravity **0.03**, maximum fall **0.75** (parameters; the body's are 0.12 and 1.55) |
| Teleport | **16-32** (17 frames: "teleport duration 17 frames") | direction **decided on frame 16** from the stick; **base speed 3.5** (→ 0.4375 world units a frame [D]), so **59.5 reference units = 7.44 world units** of travel [D]; **intangible 16-32** (UFD and kuroganehammer.com agree) and invisible; **no hitbox** |
| Reappearance | 33-67 (animation 35 frames from reappearing: "67 (35)") | speed **x0.5** on reappearing (→ 1.75 reference units a frame along the warp [D]); **maximum horizontal speed 2.7** after the teleport; then **helpless** in the air. Grounded and aerial versions have the same timing |
| On the ground | | the reappearance on the ground ends after its 35 frames (UFD "67 total frames when ending on the ground" → FAF 68 [D]). **Edge-cancel**: sliding off a ledge **during the grounded reappearance** cancels it (ssbwiki.com `Ledge-canceling`: "applies when reappearing on the ground"); she is then in an ordinary fall and **not helpless** (secondary guides: "you will have no landing recovery") |
| Landing | | **landing lag 22** (parameter, UFD and dragdown agree; the "landing" animation is 22 frames) |
| Ledges | | "can grab ledges more easily during Warp" since **2.0.0**; the windows are **not published** [gap 8] |

Not stated: whether the stick's magnitude changes the distance (the single "base speed" parameter suggests not [E]), the neutral-stick
direction (straight up, as players describe it [E]), whether the direction is any angle or snapped, whether she turns to face the warp,
what the teleport does against the ground (slide along it, stop, or land), walls, ceilings and pass-through platforms (secondary source:
"can't pass through solid ground") [gap 9], and the air control during the reappearance and the special fall (the moveset summary's "she
also falls faster in this state" has no number) [gap 10].

### Down special: Counter and Reflect Barrier [R]

**One move with two separate answers** (ssbwiki.com: "Counter activates against melee attacks, whereas Reflect Barrier activates against
reflectable projectiles at either close range or from a distance"; the reference's in-game tip: "it is automatic, but both can't happen
at once"; ssbwiki.com `Counterattack`: of the four counter-reflectors in the game only another fighter's "acts as a counterattack and
reflector simultaneously"). A melee hit inside the window starts the **counterattack**; a projectile touching the **projectile search box**
starts the **reflection**. The counter does not answer projectiles; the reflection does not strike.

| Part | Frames | Data |
| --- | --- | --- |
| Stance | **counter 7-34**, **projectile search box 7-34**, **intangible 6-12** | FAF **71** (animation 70 grounded, 79 aerial). Search box r **12.3 → 15**, top (10.0, 1.5) [D]. "Counter bubble" r 11.5 at (0, 10, 1.5) [R] (where hits are caught). Air movement: horizontal speed **x0.33** on starting, **maximum vertical speed 0.8**, air friction **0.0025**, gravity **0.04**, maximum fall **1.2** (parameters) |
| Counterattack | hitbox **5-7** (conflict 2), intangible **1-6** (conflict 2) | animation **37** (UFD and kuroganehammer.com FAF 37). True damage **1.3x the countered hit, at least 10%, at most 50%** (both ids): id 0 361 (3), **BK 50, KS 70**, r 7.0 → 9, top (10.5, 13.0 to 14.5) [D]; id 1 361 (3), **BK 30, KS 65**, r 9.0 → 11, top (10.5, 9.0 to 20.0) [D]; H **1.5**; no clank. Ignores armor (except a double-jump armor) [R]. "Counterattack frame from hit 16" (parameter; meaning not stated: conflict 3) |
| Reflection | **body reflector 1-4**, **reflect barrier 1-44**, **barrier windbox 2-11** | FAF **35** (animation 50 grounded, 59 aerial). **Body reflector**: r 11.5 → 14 at top (10, 1.5) [D], damage **x1.4**, speed **x1.25**, lifetime **x1**, damage limit **50**. **Reflect barrier**: a separate object created at **[11, 9.5]** (forward, up) [R], **lifetime 45 frames** (UFD: "barrier lasts until frame 45 for additional reflects"), a capsule r **3.8 → 5** from 11.5 above to 7.5 below its point, 1 behind it (a wall about 19 reference units tall: from 2 to 21 above the feet, centred 10 in front [D]), damage x1.4, speed x1.25, lifetime x1, limit 50. **It does not move** in this game ("Unlike in the earlier game the barrier no longer moves at all, remaining stationary in front of her until the move's conclusion") [R]. **Windbox**: 0%, 361 (3), FKV **50**, **set weight**, no hitlag, flinchless (a push, E2), r 5.0 → 6, (8.5 to -4.5 up, 0) on the barrier [D/E], not items |

Notes: the barrier outlives the reflection's FAF (35) by ten frames: if Palutena acts at 35, the barrier stays and keeps reflecting
until frame 44 / 45 [R, UFD]. The earlier game's barrier moved forward and pushed; that is **not** this game's behaviour (and the task
brief's "a reflector that moves forward" describes the earlier game). Not stated: whether a projectile from behind is reflected (the
search box and body reflector are centred on her, the barrier in front), whether she turns round for it, what reverses on a reflected
shot (horizontal only, or vertical too), what happens to the barrier if Palutena is hit, and landing during the aerial versions [gaps
11-13].

## Conflicts and how they are resolved

1. **Heavy landing lag**: the parameter file (dragdown) `landing_heavy_frame` **7** (6 frames here by the house rule); kuroganehammer.com
   (5.0.0) "hard landing lag 4". ssbwiki.com has no table. Resolved to the **parameter file** (it is the source the house rule and every
   earlier port used); recheck in footage (a fall onto the stage with no landing input).
2. **Counterattack frames**: hitbox **5-7** (ssbwiki.com, dragdown) or **4-6** (UFD, kuroganehammer.com); intangibility **1-6**
   (ssbwiki.com) or **1-5** (UFD, dragdown, kuroganehammer.com). Two-two on the hitbox, three-one on the intangibility. A one-frame origin
   difference (counting the catch frame or not) explains both. **Unresolved**: use ssbwiki.com's 5-7 and the majority's 1-5 until footage;
   a test can only pin what we choose.
3. **"Counterattack frame from hit 16"** (ssbwiki.com parameter, no explanation, no other page has it): either a 16-frame pause after the
   catch before the counterattack runs, or the frame of the stance animation the counterattack replaces. UFD's note "invulnerable on 1-5
   in addition to counter freeze frames" suggests a freeze. **Unresolved**; the engine's catch today freezes the counterer for the caught
   hit's hitlag only.
4. **Counterattack FAF**: UFD and kuroganehammer.com 37; ssbwiki.com gives only the animation length 37 (which would make the FAF 38 if
   nothing is interruptible earlier). Resolved to **37**.
5. **Explosive Flame last hit**: ssbwiki.com 57, UFD 57, kuroganehammer.com 57; dragdown 60-61 (and its loop 28-57). Resolved to **57**
   (three to one).
6. **Autoreticle**: search frames 7-14 (ssbwiki.com, UFD) or 8-14 (kuroganehammer.com, older); shot hitboxes 24-46 (ssbwiki.com) or
   24-48 (kuroganehammer.com). Resolved to ssbwiki.com. The "reticle x/y offset [-0.4, 0.4]" parameter has no stated meaning (a range
   of random offset, or an offset limit): **unresolved**, left out (a random offset would come from the sim's RNG if footage shows one).
7. **Shot lifetime vs active frames**: lifetime 25 but hitboxes of 23 frames (24-46). Read as: spawned on frame 23 ("time before bullet
   generation 23"), hitting from the next frame, and the last frame or two of life without a hit [D]; the engine's projectile also hits
   from the frame after it appears.
8. **Moveset summary vs move page** (up tilt last hit 2.5% vs 4.5%; neutral air last hit "13.4% total"): the summary is stale; the move
   page, the patch note and dragdown agree on 4.5% and the per-hit data adds to 13.5% (6 x 1.4 + 5.1) [D].
9. **Reflect Barrier moving forward** (the task brief, and the earlier game): ssbwiki.com says it is stationary in this game. Resolved
   to stationary.

## Gaps (needed for 1:1, not published)

1. How Autoreticle's shots fly to the reticle (straight from the staff, or steering), and whether they stop at it.
2. Autoreticle's exact "in front" region (an angle, or a half-plane), the choice between two targets, and whether pass-through platforms
   block the line of sight.
3. The aerial Autoreticle's descent stall (speeds), and whether it is once per airtime.
4. Landing during the aerial versions of every special (Autoreticle, Explosive Flame, the counter stance, the counterattack, the
   reflection): the reference's specials with separate grounded and aerial animations usually switch to the other version keeping the
   frame [E], not confirmed for these.
5. The height of Explosive Flame's point relative to Palutena's feet.
6. The aerial Explosive Flame's gravity and drift.
7. Whether Explosive Flame is cancelled when Palutena is hit before it explodes (it is placed on frame 1, a separate object).
8. Warp's ledge-grab windows (front and back, which part of the move), as improved in 2.0.0.
9. Warp against the stage: sliding along the ground, stopping at walls and ceilings, passing pass-through platforms going down.
10. Air control during Warp's reappearance and the special fall after it.
11. Reflect Barrier against shots from behind; whether she turns.
12. Which speed components a reflected shot reverses (the Embers' cape reverses both, the Claws' only the horizontal: owner decision
    2026-10-13 that those two differ).
13. What happens to the barrier when Palutena is hit or KO'd.

Footage that would settle most of them (training mode, frame-stepped, hitbox view): Autoreticle at several distances and heights, with a
platform between; Explosive Flame from a short hop and while falling; Warp into the stage at 45 degrees down, into a wall, down through a
platform, and at the ledge; the counter against a melee hit (count the frames from the catch to the strike) and against a projectile from
in front and behind; the reflection interrupted by a hit.
