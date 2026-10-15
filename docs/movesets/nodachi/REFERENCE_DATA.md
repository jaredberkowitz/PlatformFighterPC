# Reference data for the fourteenth moveset

Mechanical reference for the fourteenth port, collected 2026-10-14 for version **13.0.1**. In-game names (owner decision 65): class and
weapon **Nodachi**, body and built-in fighter **Exile** (class 13). Neither the reference character nor its moves are named in the game or
in this repo's text; the moves are described ("the wing", "the blade dash", "the eight slashes", "the flare", "the shadow orbs", "the
barrier"). What the engine can and cannot do with this data is in `PROJECT_AUDIT.md`; what was built in `IMPLEMENTATION_REPORT.md`.

## Sources and version

- **ssbwiki.com** per-move pages under the character's 13.0.1 page (`Neutral_attack/Hit_1` to `Hit_3`, the three tilts, `Dash_attack`, the
  three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(front|back|trip)`, `Neutral_special`,
  `Side_special`, `Up_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), every value from a named template
  field (`id`, `rehit`, `damage`, `sd`, `angle`, `af`, `bk`, `ks`, `fkv`, `setweight`, `r`, `bn`, `xpos`, `ypos`, `zpos`, `ff`, `sdi`,
  `clang`, `rebound`, `direct`, `a=f` / `g=f`, `hitbits`, `trip`) and the timing tables; the hitbox animations of the forward and up smashes
  (ssbwiki.com's hitbox GIFs, decoded frame by frame) for where the blade is in those poses.
- **ssbwiki.com articles**: the character's 13.0.1 page (the moveset table and its descriptions of each special: the charge thresholds of
  the side special, the up special's hold rule, the counter's damage formula and cap), `Winged_Form` (the wing's whole rule set: the start
  table, the points, the multipliers, the smash armor), and the four special articles.
- **ultimateframedata.com** (UFD: startup, total = FAF - 1, landing lag, autocancels, the specials' notes: the flare's release frames,
  the charge cancel's 7 frames, the up special's ground totals).
- **kuroganehammer.com** (FAFs, the winged numbers: run 2.325, walk 1.035, fall 1.932 / 3.0912, gravity 0.11124, air acceleration 0.0885,
  three jumps; air times SH 27, FH 45 and winged 26 / 44; the up special's FAF 57 / 105; the counter's damage formula).
- **dragdown.wiki** (the parameter file, raw: ground and air numbers, jumps, landing frames; its frame data rows).
- ssbwiki.com `Dash` (13.0.1 table: 10 frames, the winged dash and run numbers) and `Item_throw` (the release frames: the common 8 / 11 /
  12 / 7 / 5 / 6).
- A forum thread with community frame data (smashboards.com "preliminary frame data", via a search summary; the page itself refused the
  fetch): the up special decided by special held on frame 17, the direction locked on 19 / 31; the giant flare's hit on about 130.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, formula, frame shift); **[E]** estimate. Positions on the
blade's bone (`swordl1`) and the dash's rotating root (`rot`) are [E] by nature. Offsets on `top` (the root at the feet: `zpos` forward,
`ypos` up) convert directly [D].

## Conventions

As `docs/movesets/dagger/REFERENCE_DATA.md`: reference frame numbers (frame 1 first; FAF as published, UFD's totals + 1), 1 world unit = 8
reference units, radius in world tenths = reference x 1.25 rounded half up [D], damage before the 1.2 one-on-one multiplier, **361** the
reference's 361 angle, `af` unset = angle type 0 (by position), 3 forward, 4 backward; `a=f` grounded targets only, `g=f` aerial only; shield
damage `sd` (additive) as `100 x (damage + sd) / damage` percent [D]. The reference's box ids are the priorities (the lowest wins), with
one listed exception (the forward smash's root box, below).

## Attributes (13.0.1) [R unless marked]

| Value | Reference | Ours |
| --- | --- | --- |
| Weight | 79 | 79 |
| Walk | 0.9, accel 0.02 + 0.08 | `gu(900)`, `gu(20)` + `gu(80)` |
| Initial dash | 1.92 for 10 frames (pivot 12) | `gu(1920)`, 10 |
| Run | 1.86, accel 0.03 + 0.09 | `gu(1860)`, `gu(30)` + `gu(90)` |
| Traction | 0.11 | `gu(110)` (doubled 220 for ground moves) |
| Air speed | 0.95, accel 0.01 + 0.07, air friction 0.008 | `su(950)`, `su(10)` + `su(70)`, `su(8)` |
| Gravity, fall, fast fall | 0.108, 1.84, 2.944 | as written |
| Jumps | full 30.5, short 10.0, double 34.0 (sideways x1.0), one air jump, squat 3; ledge jump 35.25 | as written; `air_jump_drift` 100 |
| Air times | SH 27, FH 45 (kuroganehammer.com) | within one frame (tested) |
| Landings | light 1, normal 3, heavy 3 (lag 2) | 2 (shared), 2 |
| Dodges | spot dodge 26 (3-17), rolls 30 (4-15) / 35 (5-16), air dodge neutral 45 (3-30), directional down 62, side 75, up 98 (3-21) (UFD totals + 1, matching kuroganehammer.com's FAFs; diagonals not separate) | as written |
| Wall jump | no (`wall_jump_type` false); wall tech jump 1.3 away / 2.4 up | `wall_jump 0` |
| Item throws | 8 / 11 / 12 / 7 / 5 / 6 | `ItemThrows::STANDARD` |
| Size | `scale` 1.0, a tall body | the shared body size [E] |

## The wing [R unless marked] (ssbwiki.com `Winged_Form`)

| Rule | Reference | Ours |
| --- | --- | --- |
| When | after a hit, by the fighter's percent against a threshold that depends on its stock difference with the opponent (capped at +-2) and the percent difference (its percent less the opponent's) before the hit | `Wing::rows` (one per stock difference), checked in `wing::on_hurt` after every hit, pummel and damage-over-time tick that raises the percent |
| Start table | -2: 30 above -70; 30 to 45 from -70 to -120; 60 to -320; 70 to 80 from -320 to -370; 80 below. -1: 40 above 40; 40 to 55 from 40 to -10; 70 to -210; 80 to 90 from -210 to -310; 90 below. 0: 80 above -100; 90 to 100 from -100 to -150; 100 below. +1: 85 above 10; 95 to 105 from 10 to -40; 105 below. +2: 90 above 120; 100 to 110 from 120 to 70; 110 below | points on each row's line (`WingRow`) |
| The table's formulas | two rows' published formulas contradict their own ranges (the -2 row's `70 - 0.2 x (320 - diff)` and the -1 row's `80 - 0.2 x (210 - diff)` give -58 and -4 at the range's start); the ranges and the page's worked examples (the opponent at 0 / 50 / 100 / 150 / 200%: 30 / 30 / 30 / 41.54 / 60 two stocks behind, ... 103.34 / 110 / 110 / 110 / 110 two ahead) agree with straight lines between the range ends | the lines [D]; every published example reproduced to 0.03% (tested) |
| Several opponents | the rules are for one-on-one | the opponent with the most stocks (the lowest player number on a tie) [E] |
| Points | 110 / 100 / 90 / 65 / 50 from two stocks behind to two ahead | `points` |
| Spent | 0.3 a percent of the hit's base damage (no staleness, no multipliers), 0.15 with several opponents; a KO -50, then the points move by the new stock difference's grant less the old one's (90 -> 40 -> 15 on a KO level on stocks) | `per_dealt`, `ko_points`; hits on fighters (not shields); the KO credited to the last fighter whose hit raised the victim's percent since it came back [E: the reference's credit rule is not published] |
| Gone | at 0 points or a KO; does not come back until the fighter is KO'd | `Fighter::wing_spent`, cleared by the respawn |
| Damage | x1.3 on every attack (not items, not the final smash); only the percent the hit adds counts for knockback ("a 10% move KOs 3% earlier") | `power` 130 on hits and pummels and projectiles; the knockback formula's damage term is the move's own |
| Body | walk 1.15 (both walk accelerations too), dash 1.2, run 1.25, run acceleration 1.2 (both parts), traction 1.3, air acceleration 1.15 (base) and 1.1 (stick), air speed 1.2, air friction 1.05, gravity 1.03, fall 1.05 (fast fall too: kuroganehammer.com's 3.0912) | the wing's percents (`FighterParams::winged`) |
| Jumps | a third jump | `jumps` 1 (given at once when the wing comes, taken when it goes) |
| Smash armor | damage armor of 20% from the charge point to the hit: forward smash 10-23, up smash 14-22, down smash 10-20 | `Move::armor` with `armor_mode` powered |
| Training mode | different rules (the opponent's percent, 65 to 90%; always 110 points) | not modelled (no training-mode rules in the sim) |
| Stamina mode | at 35% of the starting HP | not modelled (no stamina mode) |

## Normals [R unless marked]

Damage in %, angle, BK, KS, hitlag multiplier; frames are the reference's.

| Move | Frames | Hits |
| --- | --- | --- |
| Jab 1 | 5-7; FAF 29; continuable 10-28 (a press, or held after a hit), held without a hit 19 | 2.5, 361 (type 3), BK 25, KS 10 (the far box 180, KS 15, fighters only), x1.4, hitstun +2; root boxes at 6.0 (r 2.5), 9.25 and 12.5 (r 2.7) ahead, 8.0 up |
| Jab 2 | 5-7; FAF 34; continuable 10-30, held without a hit 19 | 2.0, 361 (type 3), BK 35 KS 20 (r 3.2 at 7.5, r 3.7 at 11.0), BK 25 KS 15 (r 3.7 at 14.5), x1.3, hitstun +3, 8.0 up |
| Jab 3 | 7-9; FAF 36 | 4.0, 361 (type 3), BK 70, KS 80, x1.5; r 3.8 at 5.8, r 4.8 at 11.4, 7.5 up |
| Forward tilt | 14-16; FAF 38; aimed up / straight / down | hilt 7.5 (KS 102, x0.6), blade 12 (KS 98, x1.2), tip 9 (KS 102, x0.6); 361 (type 3), BK 41; capsules from 6.0 ahead, r 2.7: straight 9.5 up to 12.5 / 23.0 / 31.0 ahead; up 12.5 up to 14.2 / 17.5 / 20.0 up at 11.5 / 21.5 / 29.5; down 8.5 up to 7.1 / 4.5 / 2.5 |
| Up tilt | 10; 13-15; 16-25; FAF 45 (conflict 1) | hit 1: 1.0, the reference's 368 with a position vector (id 0: [2, 45] over 5 frames; id 1: [0, 46] over 6), BK 0, KS 100, x0.4, grounded targets, no clank; r 4.5 at 5.0 and 9.5-10.0 ahead, 8.5 up. Hit 2 clean: tip 14 (90, BK 74, KS 80, x1.6, 46.5-48.0 up), blade 12.5 (91, BK 69, KS 75, x1.2, 40.0-43.7 and 29.0-38.0 up), hilt 11 (r 3.7, 21.0-25.0 up, x0.7); late 10 / 8.5 / 7 (96, BK 60, KS 73, x0.6; r 2.0 for tip and blade). The upper body (head, chest, arms) intangible 13-15 |
| Down tilt | 9-15; 16-22; FAF 43 | 8 (85, BK 84, KS 60) then 6 (75, BK 79, KS 60), type 3; r 2.0 / 1.2 from 12.0 to 2.0 ahead, 1.5 up; r 3.5 / 2.8 from 5.5 to 2.0, 2.0 up |
| Dash attack | 14-15; 16-19; FAF 47 | 13 (53, BK 67, KS 82, x1.3) then 7 (361, BK 66, KS 82); type 3; r 3.0 / 2.5 from 4.0 to 8.5 ahead and r 6.0 / 4.0 at 13.0, 8.0 up |
| Forward smash | 24-25; charge 9 (between 8-9); FAF 68; wing armor 10-23 | on the blade (r 4.0): hilt 13.5 (BK 54, KS 81, x0.7) at 0, middle 20 (BK 57, KS 78, x1.3) at 7.0 and 15.0, tip 15 (BK 57, KS 85, x0.9) at 21.5; root boxes 13.5 (r 5.8, 21.5 ahead, 8.5 up) and 20 (r 5.5, 16.5-21.5 ahead, 9.0 up); 361 (type 3). See conflict 2 |
| Up smash | 23; 24; 25; 26; 27-28; charge 13 (between 12-13); FAF 64; wing armor 14-22 | blade ids 2 / 0 / 1 (hilt / middle / tip) at 5.2 / 13.8 / 21.0 along it, r 4.5 / 5.7 / 4.3; 23: 92 (type 3) 10.5 / 15.5 / 11.5, BK 62 / 67 / 67, KS 79, x0.9 / 1.4 / 0.9; 24: 97 (type 3) 12 / 17 / 13; 25-28: 83 (type 4) same damage, x0.8 / 1.4 / 0.8; 26 also root boxes 97 (type 3) 17 (r 5.7, 34.0 up, 5.0 ahead to 5.0 behind) and 13 (r 4.3, 40.5 up, 8.5 to 8.5) |
| Down smash | 21-23 (ground contact) / 21-22 (none); charge 9; FAF 54; wing armor 10-20 | eruption 16.5 (63, BK 85, KS 68, r 4.6 at 23.5 ahead, 2.5 up, x1.3, sd +15) and 13.5 (r 8.8, 6.5-8.5 up, x0.7, sd +10, no clank); blade along the ground 10.5 / 11.5 / 13 (48, BK 80, KS 68, r 1.2, -2.0 to 8.0, 10.0 to 19.0, 20.0 to 24.0 ahead, x0.7 / 0.7 / 1.2); none rebound. Without ground contact the blade alone (r 1.7, BK 75, KS 63) |
| Neutral air | 9-10; FAF 34; landing 9; autocancel 1, 30- | 8.5, 60 (by position), BK 64, KS 72, r 12.5 at 10.0 up |
| Forward air | 13-14; 15-18; FAF 42; landing 14; autocancel 1-2, 39- | root 8.5 (r 4.0, 4.5-8.5 ahead, 12.5 up, x0.9), tip 11 (r 2.3, 20.0-23.5 along, x1.3), blade 10 (r 2.5, 12.5-17.0), base 10 (r 2.8, 0-9.5); late 5.5 / 8 / 7 / 7 (r 3.5 / 1.8 / 2.0 / 2.3); 361 (type 3), BK 39, KS 86. Sticks to a wall it stabs (three times before landing; jump 6.9 up; FAF 18 / 23 / 20) |
| Back air | 15-16; FAF 47; landing 16; autocancel 1-4, 46- | 42 (type 4): 9.5 (BK 35, KS 106, r 4.8, 6.0-11.3 behind, x0.8), 14.5 (BK 47, KS 91, r 6.0, 16.5-22.2, x1.3), 11.5 (BK 39, KS 104, r 3.7, 28.0-30.0, x0.8), 12.5-13.0 up |
| Up air | 16; 17; 18; 19-21; FAF 54; landing 21; autocancel 1-4, 52- | blade r 4.0: hilt 7 (x0.8) at 0.5, middle 11 (x1.1) at 7.0 and 14.0, tip 8.5 (x0.8) at 21.0; BK 55, KS 96; 16 all 106 (type 3); 17 hilt 74 (type 4); 18 hilt and first middle 74; 19-21 all 74 |
| Down air | 19-23; 24-39; FAF 61; landing 26; autocancel 1-4, 55- | tip (20.0 along, r 5.5, x1.3) 15: grounded targets 58 (BK 75, KS 65), aerial 270 (BK 20, KS 90); blade 10 (48, BK 58, KS 73, r 6.0, 11.0-19.5, x0.8); late blade 10 (r 4.0, 9.5-21.0); by position. Vertical speed 1 up on 1-18, 4.2 down from 19; sideways control off 1-46. Landing: 1 the shockwave 5 (361, BK 70, KS 70, r 2.5, 6.5 to -6.5, 2.7 up, no clank) and the blade 7 (53, BK 58, KS 73, r 2.0, 3.0-24.0 along); 2 the shockwave |
| Grabs | standing 7-8, FAF 37 (r 3.3, 4.7-8.7 ahead, 8.5 up); dash 10-11, FAF 45 (r 3.0, 4.7-12.0, 7.0 up); pivot 11-12, FAF 40 (r 3.3, 4.7-15.0 behind, 7.0 up) | grounded-target boxes (the aerial ones not modelled) |
| Pummel | 1; FAF 7 | 1.3, 361 (type 3), BK 0, KS 100, set knockback 30, x2.2, r 5.0 at 10.0 ahead, 10.0 up, no clank |
| Throws | forward hit 14 (3, 361, BK 60, KS 80), throw 15 (3, 28, BK 70, KS 68), FAF 35; back hit 18 (3, 361 type 4), throw 19 (3.5, 58 type 4, BK 60, KS 105), FAF 32; up hit 13 (4, 75, BK 50, KS 90), late collateral 14, throw 14 (3, 98, BK 70, KS 110), FAF 33; down throw 40 (4, 56, BK 80, KS 94), FAF 48 | the throws' own launch with no hitlag |
| Ledge attack | 24-26; FAF 56; intangible 1-26 (21 / 13 on the second / third grab) | 9, 45 (type 3), BK 90, KS 20, sd +1, r 5.0 from 10.5 ahead to 0.5 behind, 5.0 up, no clank |
| Get-up attacks | 19-20 behind, 24-25 ahead; FAF 46; intangible 1-25 (face up and face down alike) | 7, 48, BK 80, KS 48, sd +8, r 5.0, 12.5-5.0 behind then 12.0-5.0 ahead, 5.0 up, no clank. The trip get-up attack: 5, 361, BK 60, KS 50, 19-20 and 31-32, FAF 50 (no slot of its own) |

## Neutral special: the flare [R unless marked]

| Rule | Reference | Ours |
| --- | --- | --- |
| Charge | from frame 20 letting go releases it; shield or jump cancels the charge (7 frames), nothing kept | the charge move's script; the cancel slot (FAF 8) |
| Flare | let go before 31 frames held; the fireball 22 frames after the release, 22-95 of the release (UFD: startup 41, total 65 from the release; kuroganehammer.com: FAF 84) | as written |
| Its hit | 3%, 82 (type 3), BK 55, KS 15, set weight, x0.5, SDI 0, sd -2; r 3.0 / 3.5 / 4.0 / 4.5 from the release's 22 / 28 / 35 / 40 | phases by age (whole world tenths: 0.4 / 0.4 / 0.5 / 0.6 [D]) |
| Its burst | 3 frames after contact (its flight's end, a wall, a fighter), 4 frames: 10.5%, 62 (type 3), BK 77, KS 64, r 12.5, sd -5, no clank | a burst from age 3, life 7, on walls and at the end too |
| Its flight | "about two thirds of a big stage", slowly | 1.53 of the reference's units a frame [E] |
| Bigger flare | held 31 frames or more; the fireball 21-48 of the release (3.5%, r 4.0); FAF 73 from the release (104 from the start) | as written |
| Its burst | 10 frames after contact: 3.7% at 366 (the vortex), BK 0, KS 100, set knockback 50, set weight, r 19.3, x0, rehit 3, 2 frames; then set knockback 60, x1.5, 7 frames; then 9% at 58 (type 3), BK 84, KS 96, r 19.8, its own part (hits after the multi-hits) | phases from age 10 / 12 / 19; the last its own group |
| Its flight | faster than the flare, a little under half its distance | 1.86 a frame [E] |
| Giant flare | at full charge, by itself, past cancelling; the fireball 130-176 (6%, r 2.5, x0.7, sd -3); FAF 182 | the release on frame 108 [E: unpublished], so the fireball comes 22 frames after it as the flare's |
| Its burst | 7 frames after contact, three rings at once: 2 frames at 115 (set knockback 50 / 35 / 35), 16 frames at 366 (75 / 65 / 65), rehit 3, 3.7 / 2.7 / 2.0% at r 12.5 / 22.5 / 30.0; then 11 / 8 / 4% (54 / 52 / 52, BK 105 / 95 / 95, KS 115 / 115 / 95, r 13.0 / 22.5 / 30.5) | rings (E58) |
| Its flight | barely past arm's reach | 0.43 a frame [E] |
| Moving while charging | not published as such (the charge stands on the ground; in the air the fighter falls and drifts as usual) | air physics as usual in the air, standing on the ground [E: a question for the owner] |

## Up special: the blade dash and the eight slashes [R unless marked]

| Rule | Reference | Ours |
| --- | --- | --- |
| Which | special held at least 18 frames: the eight slashes (ssbwiki.com; the forum's "held on frame 17") | the switch on frame 18 |
| Aim | locked on 19 (the dash) / 31 (the slashes); neutral straight up; grounded versions do not leave edges | as written; on the ground not below level [E: as the brawler's up special] |
| Blade dash | one hit on 25-28: 7%, 107 (type 3), BK 70, KS 60, r 3.0 on the rotating root from 9.0 ahead to 4.0 behind, x0.5, shieldstun x0.5, sd -3, no clank | the capsule ahead of the body's middle as published (it does not turn with the dash) [E] |
| Its timing | ground FAF 57 (UFD total 56 "along the ground"); the animation 80, then the special fall; landing lag 27; it catches ledges in travel | as written; ledges in front and behind from frame 25 [E] |
| Its travel | "a fairly short distance", quick | 11 of the reference's units a frame on 25-28, a quarter kept after [E] |
| Started in the air | the startup stalls (not published in numbers) | half the sideways speed and none of the vertical kept on starting (`start_speed` 50, `start_speed_y` 0, sim 130, decision 97), then 0.9 a frame and a 0.0025 sink to frame 24 [E] |
| Into the ground | "its active frames end immediately on landing ... sliding with the remaining momentum" | an ordinary landing out of the move with 27 frames of lag |
| Eight slashes | hit 1 on 36: the middle box 361 (r 8.0, BK 0, KS 100) for everyone, then the reference's 368 for aerial (`g=f`) and grounded (`a=f`) targets at 1.0 (r 8.0), 10.0 and 20.5 (r 6.5) ahead; hits 2-7 on 38-48 every 2 frames (the same, r 5.0 / 5.0 / 6.5, and two boxes at 60, BK 90, KS 60, r 6.5 at 0 and 10.0); hit 8 on 50: 7% at 361 (type 3), BK 68, KS 104, r 10.0 at 6.0 and 18.0, x2.4, and 2.3% at 60 (BK 90, KS 60, r 8.5 at 2.0 behind, x0.5); hits 1-7 2.3%, SDI x0.5, sd -1, no clank | the 368 drag as the autolink angle (367) [E: the reference's 368 here has no published position vector]; the boxes ahead of the body's middle [E] |
| Its timing | ground FAF 105; animation 106, then the special fall; landing lag 56; no ledge until it ends; grounded it stops at the edge, slashing in place | as written |
| Its travel | "slightly farther than the blade dash" | 3.3 of the reference's units a frame on 35-50 [E] |
| Its landing hit | 2.3%, 60, BK 90, KS 60 on landing's frame 1 (kuroganehammer.com) | in its 56 frames of landing lag, landing during the move or in the special fall after it (sim 128); the box (radius 8.0 at 4.0 up, centred on the feet) [E] |

## Side special: the shadow orbs (sim 128) [R unless marked]

Sources: ssbwiki.com `Sephiroth_(SSBU)/Side_special` (raw: the hitbox tables and timing) and the move's article (the rules), the
character page's description (the counts by charge, "8 frames after releasing"), kuroganehammer.com (damage at no and full charge, the
orbs' travel 1-13 and explosion 14), ultimateframedata.com (startup 16-42, total 38-64); the hitbox animations at minimum and maximum
charge (ssbwiki.com's GIFs, decoded frame by frame) for the beam's speed and place.

| Rule | Reference | Ours |
| --- | --- | --- |
| Charge | held from frame 9 ("charges between 8-9") up to 26 frames; the beam 8 frames after the charge ends | the script holds frame 8 (the move's numbering) while special is held, `charge` < 26 |
| Timing | the beam on 16-22 uncharged, 42-48 at full; FAF 39 to 65 (UFD total 38-64 + 1) | as written (tested) |
| Beam boxes | id 0: 1.5%, 361 (type 3), BK 10, KS 30, r 2.0 from 0 to 8.0 behind the projectile's root, x0.5, fighters only; id 1: 2%, KS 100, same shape, not fighter-only (things that are not fighters); ids 2 / 3: r 3.5 from 1.5 to 6.5 behind, `g=f` (aerial targets); all no trip (`trip=-100`), reflectable, absorbable | two rings on the projectile (E58) at age 1: a lying capsule r 2.0 (centre 4.0 behind the head, half width 4.0) for everyone and one r 3.5 (half width 2.5) for fighters in the air; the 2% object boxes not modelled (no objects) |
| Beam damage | kuroganehammer.com: 1.611 uncharged, 4.5 at full: 1.5 + 3 x (charge + 1) / 27 [D] | `ChargedShot` 29/18 to 4.5 over 26 frames, `every_hit` (both rings) |
| Beam flight | 7 frames | life 8 (ages 1-7); 5.46 of the reference's units a frame uncharged, 8.16 at full (x1.5), its first hit 6.4 ahead and 8.8 up whatever the charge [E: from the hitbox animation, 17.3 pixels a unit] |
| Orbs placed | one uncharged, two from 17 frames of charge, three at full; up to five on one fighter; five active for one exile in all, reflected ones on itself included; none on a fighter with another exile's orbs, none while its orbs are attacking; a reflected beam places twice as many (5 at most) | `Orbs` count 1, more_at 17 / 26, reflected 2, per_target 5, per_owner 5 |
| Clock | 4 seconds from the hit; a later hit adds 0.5 to 3 seconds by its charge, never past 7.5 seconds from the first hit | wait 240, extend 30 to 180 [E: in proportion to the charge], cap 450 |
| Orbiting | circling the target; no hitbox against it; a bystander it touches takes 7%, 50 (`af=1`: by travel), BK 65, KS 75, r 1.5, x0.4, SDI x2, sd -3, no clank, rehit 45; each orb hits two at most and is used up | as written; the orbit 12 of the reference's units round the chest, a turn every 60 frames [E] |
| Strike | each orb travels 13 frames (a hitbox on 1-13, kuroganehammer.com) and explodes on 14: 6.5%, 53 (type 3), BK 50, KS 75, r 3.5, x0.8, sd -3, no clank; always launched away from the exile (or the beam's current owner) | as written; all the orbs on a fighter together [E] |
| Reflected, absorbed, pocketed | an orb can be reflected or absorbed as it attacks; pocketed, it comes out flying horizontally | a striking orb reflected or pocketed flies on straight as a shot with the strike's hit at the beam's uncharged speed for 60 frames [E] |
| Pierces projectiles | the beam goes through projectiles, cancelling weak ones | not modelled (no projectile-projectile collision in the engine) |

## Down special: the barrier (sim 128) [R unless marked]

Sources: ssbwiki.com `Sephiroth_(SSBU)/Down_special` (raw) and the move's article, kuroganehammer.com (the formulas, the break's FAF 41,
"counters 25% HP"), ultimateframedata.com ("Sephiroth does not suffer hitlag", intangible 1-11); the hitbox animations (miss, minimum and
maximum) decoded frame by frame for the catching capsule and the answer's centre.

| Rule | Reference | Ours |
| --- | --- | --- |
| Catch | 7-27 on a barrier in front; the body does not count; melee and projectiles (it stops them, it does not reflect them) | `Counter` 6..26 (the move's numbering), `bubble` a standing capsule 7.7 ahead, centre 11.0 up, half height 9.2, r 5.9 [E: read from the animation at 19.6 pixels a unit] |
| Not struck | the answer on 38-52 (1%, 366, set knockback 60, BK 40, KS 65, set weight, r 9.5, x0.8, SDI 0, rehit 5) and 53 (5%, 361 type 3, BK 66, KS 75, r 11.0, x0.7, sd -2); no clank; FAF 87 | a standing projectile from frame 37, life 17, piercing, rehit 5, the last its own group; centre 14 ahead, 10.5 up [E: from the animation] |
| Struck | answer on 7-21 / 22 (activation numbering), intangible 1-11, FAF 52; no hitlag for the exile | `BARRIER_ANSWER`, `Counter::still`; centre 19 ahead, 10.5 up [E: from both hit animations] |
| Its numbers | interpolated on the caught hit's damage D (0 to 18): multi-hits 2 + 2D/18 (sd -1 to -2, r 16.5 to 22.5), the last 6.5 + 11.5D/18 (BK 80 to 83, KS 70 to 83, r 18.0 to 24.0, x0.7 to x0.4, sd -3 to -9) | `Move::lerp` full 18; damage [R], the other fields read the same way [E] |
| Break | a hit of more than 25% (30% after the one-on-one multiplier) breaks it: no damage, a stagger, pushed slightly back; FAF 41 | `Counter::break_above` 25 (compared before the multiplier), `BARRIER_BREAK` FAF 41, 5 units back over 6 frames [E] |
| A projectile | the answer can be reflected or absorbed; 11.0.0 enlarged the last hit (9 -> 11, 16-22 -> 18-24) | the answer is a projectile (reflectable, pocketable); 13.0.1's sizes |
| In the air | not published separately | the same move with the body's air physics [E] |

## Phase 1's leftovers (sim 128) [R unless marked]

| Move | Reference | Ours |
| --- | --- | --- |
| Down smash without ground contact | ssbwiki.com: a ground test from 16.0 ahead and 4.5 up to 32.5 ahead and 2.5 down (`top`); with no ground the blade alone on 21-22 (10.5 / 11.5 / 13%, 48 type 3, BK 75, KS 63, r 1.7, same places, x0.7 / 0.7 / 1.2, no rebound); FAF 54 both | `Move::probe` on frame 21 (the move's 20), `DSMASH_OFF` |
| Forward air wall stick | ssbwiki.com: stabbing a wall sticks to it, three times before landing, not below the camera bounds, not on a wall that bars wall jumps, not with the ground less than 14 below; stuck: interruptible 18, 120 frames then letting go; jump (up, jump or attack): jump on 12, gravity x1.05, "initial vertical speed 6.9", FAF 23, ledges from 27; let go (away or down): FAF 20 | the forward air's script on 13-18 (`ground()`: a segment level with the blade 12.5 up out to 32 ahead [E]; a segment 14 down), marks 5-7; `WALL_STICK`, `WALL_JUMP` (a jump 6.9 units high [E: the speed read as a height]), `WALL_DROP`; the exile stays where it stuck [E: the "jostle distance 7" not modelled]; camera bounds and wall-jump bars not modelled (no such stage data) |

## Gaps and conflicts

1. **Up tilt FAF**: ssbwiki.com (interruptible 45, its frame strip) and kuroganehammer.com (45) against ultimateframedata.com (total 37:
   FAF 38) and dragdown.wiki (37). Taken: 45 (two sources, the frame strip's own count).
2. **The forward smash's 13.5% root box (id 1)** sits where the 20% blade boxes are (21.5 ahead, 8.5 up, r 5.8); with the reference's ids
   as priorities it would win there and the middle of the blade would deal 13.5%, against every published account (UFD "close / middle /
   tip 13.5 / 20 / 15", the wiki's description of the slash's middle sweet spot). It ranks last here [E]; the published id is recorded.
3. **The flare's burst after its flight's end**: the timing table's 98-101 after a fireball ending on 95 reads two frames; "three frames
   after contact" reads three. Taken: three after contact; the end-of-flight burst comes a frame later than the table [D].
4. **The wing's start formulas** (above): the ranges and examples taken over the two formulas that contradict them.
5. **Not published** (estimates, labelled [E] where used): the blade dash's and the slashes' speeds and distances, every flight speed of
   the flares, when the giant flare's charge is full, the startup's air stall of the up special, where the blade is in each pose (the
   forward and up smashes read from the hitbox animation), the KO credit rule, several opponents for the wing (decisions 87-91 keep
   these). Phase 2: the beam's speed and start (from the animation), the orbit, the extension's share of the charge, the orbs striking
   together, a loose orb's flight, the barrier's capsule and answer centres (from the animations), the non-damage fields of the lerp,
   the break's push, the slashes' landing box, the wall test, the wall jump's height.
6. **The orbs' numbering of the charge**: kuroganehammer.com's 1.611 uncharged means the reference counts one frame of charge for a tap;
   "two orbs from 17 frames" is read as 17 frames held on the charge frame (our `charge` 17).
7. **The wall stick jump's 6.9**: a vertical speed of 6.9 of the reference's units a frame would be far above any jump (the full hop
   starts near 2.6); read as a height [E], an owner question.
