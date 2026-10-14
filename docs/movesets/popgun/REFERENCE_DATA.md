# Reference data for the twelfth moveset (the reference game's Diddy Kong)

Mechanical reference for the twelfth port, collected 2026-10-14 for version **13.0.1** (owner decision 68 applies: everything 1:1 with
13.0.1; the older jab rule is not used here). In-game names (owner decision 65): class and weapon **Popgun**, body and built-in fighter
**Scamp** (class 11). The reference character's name is not used in the game. What the engine can and cannot do with this data is in
`PROJECT_AUDIT.md`; what was built in `IMPLEMENTATION_REPORT.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Diddy_Kong_(SSBU)/<move>` (28 pages: `Neutral_attack/Hit_1`, `Hit_2`, `Hit_3`, the three tilts,
  `Dash_attack`, the three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(back|front|trip)`,
  `Neutral_special`, `Up_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), so every value comes from
  a named template field (`damage`, `angle`, `af`, `bk`, `ks`, `fkv`, `setweight`, `r`, `bn`, `ypos`, `zpos`, `ff`, `sdi`, `sd`, `trip`,
  `clang`, `rebound`, `direct`, `a=f` / `g=f`, `effect`, `hitbits`) and the frame strips (`FrameStrip` rows: lag, hitbox, continuable,
  interruptible); the angle-361 column shift cannot happen. There is **no side special page** for 13.0.1 (the wiki's page list,
  2026-10-14). https://www.ssbwiki.com/Diddy_Kong_(SSBU)
- **ssbwiki.com articles**: `Diddy_Kong_(SSBU)` (attributes text, update history 2.0.0 to 11.0.0 via the `UpdateList (SSBU)` templates),
  the SSB4-to-13.0.1 changelist (https://www.ssbwiki.com/Template:SSB4_to_SSBU_changelist/Diddy_Kong), `Banana_Peel`,
  `Banana_Peel_(move)`, `Peanut_Popgun`, `Peanut`, `Monkey_Flip`, `Rocketbarrel_Boost`, `Slip`, `Tripping`, `Item_throw` (toss strength,
  the damage formula, the per-item factors, the per-character release frames), `Dash` (13.0.1 table), and
  `Lucario_(SSBU)/Floor_getups_(trip)` (the hard and soft trip timings, the only page that gives them).
- **ultimateframedata.com/diddy_kong** (UFD: startup, total = FAF - 1, landing lag, autocancel, the specials' notes, dodges, air times).
  Its neutral special numbers are from before 3.1.0 (121 / 124, 3.3-15%): superseded by the 3.1.0 notes.
- **dragdown.wiki** `SSBU/Diddy_Kong/Data` (the parameter file, raw; its frame data, including the side special's parts that ssbwiki.com
  has no page for).
- The previous version's side special page (`Diddy_Kong_(SSB4)/Side_special/Default`) was read for the kick's trip chance and the
  latch's parts; in phase 2 only the kick's and the latch attack's radii come from it (13.0.1 publishes none), labelled where used.
- **kuroganehammer.com**: its 13.0.1 page and API returned 404 (as for the last ports); not used.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, formula, frame shift); **[E]** estimate. Positions on
bones other than `top` (`handr` / `handl`, `armr` / `arml`, `shoulderr` / `shoulderl`, `kneer` / `kneel`, `legr`, `footr`, `waist`) are
[E] by nature. Offsets on `top` (the root at the feet: `zpos` forward, `ypos` up) convert directly [D].

## Conventions

As `docs/movesets/pockets/REFERENCE_DATA.md`: reference frame numbers (frame 1 first; FAF as published), 1 world unit = 8 reference units,
radius in world tenths = reference x 1.25 rounded half up [D], damage before the 1.2 one-on-one multiplier, **361** the reference's 361
angle, `af` unset = angle type 0 (by position), 1 by travel, 3 forward, 4 backward; `a=f` grounded targets only, `g=f` aerial only; shield
damage `sd` (additive) becomes `100 x (damage + sd) / damage` percent [D]. A projectile first hits on the frame after it appears.

## Attributes (13.0.1) [R unless marked]

| Value | Reference | Ours |
| --- | --- | --- |
| Weight | 90 | 90 |
| Walk | 1.313, accel 0.105 + 0.105 | `gu(1313)`, `gu(105)` + `gu(105)` |
| Initial dash | 2.09 for 10 frames (pivot 12) (ssbwiki `Dash`) | `gu(2090)`, 10 |
| Run | 2.006, accel 0.044 + 0.165 | `gu(2006)`, `gu(44)` + `gu(165)` |
| Traction | 0.093 | `gu(93)` (doubled 186 for ground moves) |
| Air speed | 0.924, accel 0 + 0.05, air friction 0.008 | `su(924)`, 0 + `su(50)`, `su(8)` |
| Gravity, fall, fast fall | 0.125, 1.75, 2.8 | as written |
| Jumps | full 41.21, short 19.9, double 41.21 (its sideways speed x0.9: `jump_aerial_speed_x_mul`), one air jump; ledge jump 40.605 | as written; `air_jump_drift` 90 |
| Air times | SH 36, FH 51, SHFF 26, FHFF 36 (UFD) | within one frame (tested) |
| Landings | light 1, heavy 7 (lag 6) | 2 (shared), 6 |
| Dodges | spot dodge FAF 24 (3-16: UFD; conflict 4); rolls 29 (4-14) / 34 (4-15, not separate here); air dodge neutral 45 (2-27), directional down 63, side 75, up 97 (2-19) (UFD totals + 1; the diagonals are not separate) | as written |
| Wall jump | yes (`wall_jump_type`), 1.3 away / 2.3 up | `wall_jump 1` |
| Item throws | release 7 / 9 / 11 / 6 / 4 / 5 (forward, back, up, down, dash, drop; ssbwiki `Item_throw`); every character's lengths are the shared 24 / 24 / 23 / 19 / 29 / 23 in 13.0.1 (the same page) | as written; the ruleset's lengths |
| Toss strength | 0.966667 (`throw_item_speed_mul`; `1 + (90 - 100) / 300`) | the engine's formula gives it [D] |
| Size | `scale` 1.0, `fighter_size_m` | the shared body size [E] |

## Normals [R unless marked]

Frame strips read with UFD and dragdown.wiki; where they disagree the majority is used and the conflict is listed.

| Move | Active | FAF | Hits (damage, angle, angle type, BK, KS, radius; [E] = position) | Notes |
| --- | --- | --- | --- | --- |
| Jab 1 | 3 | 20 | 2%, 361 (3), BK 25, KS 25, r1.8 at 4.0 / 7.0 ahead, 5.5 up; the far box 180 (3), BK 20, KS 15, r2.0 at 10.0 ahead (fighters only; the same box at 361 for objects not modelled); hitlag x1.6 | continuable 6-30 (into jab 2; past the FAF: decision 58); held, jab 1 again from 10 (the loop marker on 6-9) [D] |
| Jab 2 | 5 | 21 | 1.5%, 361 (3), BK 20, KS 25, r2.3 at 4.0 / r2.6 at 7.0; the far box 180 (3), BK 20, KS 15, r2.8 at 11.3; hitlag x1.2 | continuable 7-30 into jab 3 (UFD says 6: conflict 11) |
| Jab 3 | 5 | 35 | 4%, 361 (3), BK 60, KS 80: r2.6 at 4.0 / 6.0 up, r3.0 at 7.5 / 6.3, r3.8 at 12.0 / 6.8; hitlag x2 | the infinite is gone in 13.0.1 |
| Forward tilt | 10-11 / 12 / 13-16 | 37 | 35 (3), BK 60, KS 60: 10-11 the hands (r3.8) and two root boxes (r4.0 at 7.5 / 6.0, r3.0 at 1.5 / 6.0) 10%; 12 the hands 10%; 13-16 the hands 7% | angled up / down (dragdown): the hands placed per aim [E], the root boxes in every aim |
| Up tilt | 6-11 | 30 | 6%, 92 (0), BK 28, KS 150, the arm (two r4.0) and shoulder (r3.8) [E: over the head], SDI x1.5 | |
| Down tilt | 4-5 | 19 | 5.5%, BK 15, KS 75, type 0, trip 20%: 60 degrees r3.0 at 16.0 / 1.8, 50 degrees r3.5 at 8.5 / 3.0 | FAF 19 (UFD, dragdown) vs 20 (the strip): conflict 1 |
| Dash attack | 8-9 / 16-17 / 22-23 | 41 | hit 1 2%, set weight, BK 0, KS 100, hitlag x0.5: grounded 35 (FKV 60), aerial 10 (45) r3.5 swept 8.5 / 11.0 to 7.5 / 9.5, all 43 (60) r3.5 at 7.5 / 5.5; hit 2 2%, hitlag x0.8: grounded 25 (45), aerial 10 (45) r4.0 swept 8.0 / 11.5 to 7.0 / 10.0, all 40 (70) r4.0 at 7.0 / 6.5; hit 3 3%, 83, BK 80, KS 90, r5.5 at 8.0 ahead from 11.0 down to 6.0; every box type 3, never clanking | |
| Forward smash | 12 / 13 / 21-23 | 51 | charge on 4; hit 1 5% set weight, BK 0, KS 100, type 3: 12 the root box (r4.2 at 9.5 / 9.7, aerial only, 10, FKV 20), the arm 45 (35), the arm's grounded-only 80 (25), the shoulder 22 (42); 13 the root box at 9.2 / 13.5, 0 (20); hit 2 11% / 11% / 9%, 46 / 50 / 60, BK 40, KS 116, hitlag x1.2 [E: arm positions] | |
| Up smash | 5 / 6-8 / 12-14 / 15 / 19-23 | 54 | charge on 1; hits 1-2 2.5% set weight, BK 0, KS 100, type 0: 5 the arm grounded 110 (FKV 110), aerial 125 (30), the shoulder 95 (50); 6-8 the arm 130 (25) / 140 (15), the shoulder 95 (50); 12-14 the leg 105 (65), the knee 125 (45); 15 the leg 105 (65), the knee 115 (80); hit 3 6%, 85 (3), BK 55, KS 131, the arm r6.5 and shoulder r4.5 [E: limb positions] | |
| Down smash | 7-8 / 12-13 | 51 | charge on 2; 12% 33, BK 46, KS 77 (r3.9 at 13.8, r3.3 at 8.8, 3.7 up); 15% 35, BK 38, KS 76 (r3.9 at 10.5 behind, r3.3 at 5.5 behind); type 0 | FAF 51 (UFD, dragdown) vs 50 (the strip, the 8.0.0 note): conflict 2 |
| Neutral air | 8-20 | 49 | 6%, 80 (0), BK 60, KS 75, both arms and knees r3.5 [E: positions] | landing lag 8, autocancel 1-3 and from 27 |
| Forward air | 6-9 / 10-16 | 47 | 361 (3), BK 30, KS 86: clean 10% (the foot and the other knee, r5.0), late 8% (r2.7) [E: positions] | landing lag 16, autocancel 1-2 and from 26 |
| Back air | 5-8 | 31 | 9%, 361 (4), BK 30, KS 102, the knee r5.3 / r4.7 and the leg r4.3 [E] | landing lag 7, autocancel 1-2 and from 20 |
| Up air | 4-7 | 36 | 7%, 76 (0), BK 57, KS 89, the knee r5.0 / r4.5 [E] | landing lag 9, autocancel from 27; FAF 36 (the 3.1.0 note, the strip) vs 37 (UFD, dragdown): conflict 3 |
| Down air | 15 / 15-16 | 47 | 270 (0): 15 13% (r4.5 at 6.5 below, BK 25, KS 110) and 10% (r6.7 at 3.0 below, BK 20, KS 100); 16 the 13% box | landing lag 17, autocancel 1-14 and from 35 |
| Grab | 7-8 | 37 | r3.7, 4.0 to 7.8 ahead, 7.0 up (grounded targets; the aerial box r1.85 not modelled) | |
| Dash grab | 10-11 | 45 | r3.0, 4.0 to 9.5, 6.0 up | |
| Pivot grab | 11-12 | 40 | r3.7, 4.0 to 13.6 behind, 6.0 up | written in front (the engine turns a pivot grab round) |
| Pummel | 1 | 7 | 1%, 361 (3), FKV 30, KS 100, r5.0 at 9.0 / 8.0, hitlag x1.7, never clanks | ours on frame 2, FAF 8 [D] (a pummel cannot hit on its first frame) |
| Forward throw | 21 | 32 | 9%, 46, BK 69, KS 65, no hitlag | |
| Back throw | 18 | 52 | 12%, 42 behind, BK 60, KS 55 | the thrower's facing after it is not published (kept [E]) |
| Up throw | 17 / 19 | 27 | 1% on 17 (75 (3), BK 50, KS 120, r6.0 at 23.0 up, never clanking: here a hit on the held fighter), the throw 4%, 87, BK 70, KS 72, no hitlag | FAF 27 (the notes, UFD, dragdown) vs 28 (the strip): conflict 5 |
| Down throw | 22 | 32 | 7%, 106, BK 85, KS 59, no hitlag | |
| Ledge attack | 23-25 | 56 | 9%, +1 shield (111 [D]), 45 (3), BK 90, KS 20, r4.0 from 19.0 to 3.5 ahead, 5.0 up, never clanks; intangible 1-25 | |
| Get-up attack (face up) | 15-16 / 22-23 | 46 | 7%, +8 shield (214 [D]), 48 (0), BK 80, KS 48, r5.0, in front 12.0 to 5.0 then behind 11.0 to 5.0, 5.0 up, never clanks; intangible 1-23 | the face-down (15-22) and trip (19-28, 5%) get-up attacks have no slot (house limit) |

The thrown fighter's 3% hit on bystanders (every throw's `kind=1` row) and the thrower's invincibility are not modelled (house limits).

## Specials

### Down special: the banana peel [R unless marked]

- **The move** (ssbwiki raw timing table, UFD, dragdown): the peel appears on frame 4, a held item is thrown on 14, the peel on 20;
  interruptible 40 (FAF 40), animation 50. The peel is tossed over the shoulder at **105 degrees**, at **2.2** a frame (**3.036** for a
  smash input), times the scamp's **toss strength 0.966667** (ssbwiki `Banana_Peel_(move)`, "Item toss specifics"). With an item in the
  hand the move throws both, the held item first, at the same speed and arc.
- **One out**: one peel at a time; with one out the move makes none ("a new animation where he shrugs after the throwing motion, which
  has more ending lag": the previous version's description; Ultimate: "Only one peel can exist at a time"). **The wait**: after a toss
  the next peel can be made only **85 frames** later (12 frames, 61 in 8.0.0, + 24 in 11.0.0).
- **The item** (ssbwiki `Banana_Peel`, `Item_throw`): anyone can pick it up and throw it; **two uses** ("they will last for two slips and/or
  throws until they disappear"; "can hit opponents twice before disappearing, bouncing off in a high arc on the first hit"); **transcendent
  priority** (never clanks). Its throw factors: base damage modifier **+1**, speed damage multiplier **1.7**, throw speed multiplier
  **0.85**, falling speed **2.2**; the move's toss deals `1 + 1.7 x toss speed` (a drop: the base halved). No time limit is published (the
  sources describe removal by use only).
- **Thrown hit** (raw table): r3.0, 0% base, 361, BK 50, KS 0, type 1 (by travel); against grounded fighters the **slip** effect (`a=f`),
  against airborne ones a normal hit (`g=f`); `clang=f rebound=f`, reflectable.
- **Lying hit** (raw table "Grounded"): r3.0 at 3.0 below its root, 0%, the slip effect, grounded fighters only, **only leg and lower-body
  hurtboxes**, **not blockable** by a shield.
- **The slip** (ssbwiki `Slip`, `Tripping`, `Lucario_(SSBU)/Floor_getups_(trip)`): a slip always trips a grounded fighter whatever the
  knockback. A thrown peel's trip is a **hard trip** (hitlag, a push back, get-up options after 39 frames, a neutral get-up on frame 100,
  5 intangible frames: the engine's existing trip timings); walking into a lying peel is a **soft trip** (no hitlag, no push, options after
  29 frames, a neutral get-up on frame 90, 5 intangible frames). The owner (its last thrower) does not slip on it ("an opponent must
  acquire ownership and make it an opposing item by throwing it").

### Neutral special: the peanut popgun [R unless marked]

- **Timing** (13.0.1 = the 3.1.0 values; UFD's 121 / 124 are older): uncharged peanut on frame **16**; held, it charges to a **full charge
  on frame 94**, fires on release; still held, the gun **goes off on frame 97**. The charge cannot be kept; shield or an air dodge cancels
  it **from frame 14** (UFD). Total 48 for the uncharged shot (UFD: FAF 49).
- **The peanut** (raw table and 3.1.0): **4.3%** uncharged to **13.8%** full (3.1.0: "3.3%-15% → 4.3%-13.8%"), 70 (3), BK 40, KS 70, r2.4,
  hitlag x0.2, shield damage -1, `clang=f rebound=f direct=f`, reflectable, "No floor". A later version of the box is taller (-1.1 to 1.9
  up). Faster and flatter with the charge ("boost the speed and lower the angle of the shot": `Peanut_Popgun`). Peanuts bounce once on the
  ground "only aesthetic" (the changelist). **No healing** in 13.0.1 (only the Brawl-era peanuts could be eaten: `Peanut`). Thrown back as
  items in the reference (`Item_throw` lists a peanut's factors: -0.4, 1.5, 1, falling speed 2).
- **The muzzle hit**: UFD 2%, the changelist 3% ("gained a melee hitbox that deals 3%"); no angle or knockback published: gap 1.
- **The misfire** (raw table): 23%, 361 (0), BK 30, KS 92, r8.2 at 7.0 ahead / 8.5 up, hitlag x1.5, flame, `clang=f`; UFD's total 228
  for its old frame 123 (105 frames after the explosion) [D]; dragdown's 110. Overcharging stuns the scamp; no damage to him is published.

### Up special: the rocket barrels [R unless marked]

- **Charge** (raw timing table, UFD, the changelist): charges between frames 7-8; the launch is on frames **8 to 68** (60 frames of charge
  at most: "90 → 60"). The angle can be tilted with the stick while charging, more the longer the charge (`Rocketbarrel_Boost`: no
  numbers). In the air the charge falls much slower (3.1.0: "falls slower while charging (only applies to the first charge)").
- **Launch speed** (the changelist): initial **3.1**, **+0.019 a frame of charge**, **x1.05** from the air.
- **Flight hits** (raw table; frames after the launch from the uncharged 8): clean 8-9 **10%** 361, BK 40, KS 85, r4.0; late 10-16 **8%**
  80, BK 40, KS 90, r3.2; latest 17-35 **6%** 70, BK 40, KS 90, r2.8; a box each side of the waist (5.0 ahead and behind), flame,
  `clang=f rebound=f`; then 26 frames with no hit (the strip); dragdown total 62. **Landing lag 30** (UFD).
- **The crash** (UFD, `Rocketbarrel_Boost`): touching a solid surface during the boost sets the barrels off: **18%**, 70, BK 50, KS 70,
  **r20.0** round the root, flame, `direct=f`; total 57-59 on level ground; the scamp does not suffer its hitlag. "If Diddy hits a surface
  while using the move, he takes about 5% damage" (the article's general text, not version-specific): gap 2.
- **Hit while using it** (the article): the barrels fly off, irregularly moving, **unblockable** but reflectable, exploding on contact with
  the stage; the player steers them a little for a short time. Their hits (raw table "Flying"): **1%**, 361, BK 10, KS 100, r2.5 / r3.2,
  `direct=f`; their explosion the crash's 18%. "If Diddy is hit out of Rocketbarrel Boost with a weak move, he will become helpless."

### Side special: the monkey flip (phase 2, sim 119) [R unless marked]

Sources (re-read 2026-10-14 for phase 2): ultimateframedata.com (UFD) rows "Side B", "Attack", "Grab Success", "Grab Attack", "Grab
Jump"; dragdown.wiki `SSBU/Diddy_Kong` and `SSBU/Diddy_Kong/Data` rows "Flying", "Attack, Clean / Late", "Stick, Hit 1 / Throw",
"Stick Attack, Hit 1 / Throw"; ssbwiki.com `Monkey_Flip` (the descriptions; no 13.0.1 hitbox page exists); the previous version's raw
page `Diddy_Kong_(SSB4)/Side_special/Default` (radii and the trip chance only, labelled where used).

| Part | 13.0.1 data | Ours |
| --- | --- | --- |
| The leap | grab active 18-36 on the ground, 18-29 in the air (dragdown "18-36", "18-29 Aerial"; UFD 18 / 18-36); FAF 48 in the air, 62 on level ground (UFD totals 47 / 61, dragdown 48); landing lag 26 (UFD); "leaps forward and grabs whomever he contacts"; not helpless after it (ssbwiki); the changelist: startup 20 -> 18, the grab's range "3 / 2 -> 4.5 / 4.0" ahead | as written; a command grab (on the ground or in the air, shielding or not) |
| The kick | a second button press during the leap: clean 6-10 14% (inner leg) / 12% (foot), late 11-25 12% / 10%, 361, BK 80, KS 50; FAF 33 (dragdown total 33, UFD 32), landing lag 18 (UFD); it "prevents Diddy Kong from using Rocketbarrel Boost before he lands" (ssbwiki) | as written; radii 3.5 / 4.0 [R: the previous version's]; positions [E]; trip 0 (the previous version's 30%: not confirmed for 13.0.1, owner question) |
| The latch | an automatic scratch, 1%, 361, BK 0, KS 0 (dragdown "Stick, Hit 1", active 10-28 of a 46-frame animation); UFD "Grab Success 11 / 29 / 59..." | the ordinary hold clock with the shared mash-out (owner decision 74); the scratch on the latch's frames 11 and 29 of every 48 [E: our reading of UFD's list: 59 = 11 + 48] |
| Attack (latched) | a scratch 3%, 361, BK 60, KS 150 on 18-20 and a throw 7%, BK 50, KS 100 (a second row 5%, BK 0, KS 50) on 20 (dragdown "Stick Attack"); UFD 18 / 21, total 36 | the 3% on the held fighter on 18 (the house's pre-hit) and on others near it (radius 6.0 [E: the previous version's]); the 7% throw on 20; FAF 37; the 5% row not used (its role is not published) |
| Jump (latched) | a footstool-style kick, 2%, 361, BK 0, KS 50 (dragdown "Stick, Throw"); UFD 18 / 27, total 42; it "now immobilises the opponent for exactly 20 frames if the move does not cause tumble" (the changelist) | the kick on 18 as a throw with `fixed_stun` 20; FAF 43; the scamp jumps off at its full hop's speed [E] |

**Not published** (all [E] in ours): the leap's speed, arc and distance (launched on 13 at 2.2 of the reference's units a frame ahead
and 1.7 up, 2.8 ahead for a smash input, then the body's gravity, no air control); the grab boxes (two, radius 4.0, at 4.5 / 7.0 and
4.0 / 2.0 ahead / up); the kick's input window (from the launch to the end of the grab) and positions; the latch's ride point (on the
head, 0.9 of the held fighter's height up) and the held fighter's fall (its own gravity); the attack exit's backflip (1.0 back, 2.0 up
a frame) and the jump's speed; whether a hit clears the kick's up-special bar (here it does: the air flags); the smash-input leap being
further (the previous version's description); the ledges (in front once falling, from the launch; the kick from its frame 25).

### Gaps closed in phase 2 (sim 119)

- **The first charge only** falls slowly in the air (3.1.0: "only applies to the first charge"): owner decision 75, built.
- **The barrels cannot be shielded** (`Rocketbarrel_Boost`: "unblockable"): built.
- **The crash's damage to the scamp**: "If Diddy hits a surface while using the move, he takes about 5% damage" (the article's
  general text, not tied to 13.0.1; the 13.0.1 hitbox page lists no self-damage row): 5% [E], an owner question.

## Conflicts (resolved as listed)

1. Down tilt FAF: 19 (UFD total 18, dragdown 19) vs 20 (the strip): 19.
2. Down smash FAF: 51 (UFD, dragdown) vs 50 (the strip, the 8.0.0 note "52 → 50"): 51.
3. Up air FAF: 36 (the 3.1.0 note "37 → 36", the strip) vs 37 (UFD, dragdown): 36.
4. Spot dodge intangibility: 3-16 (UFD) vs 3-17 (dragdown) vs 3-14 (the changelist): 3-16.
5. Up throw FAF: 27 (the changelist, UFD, dragdown) vs 28 (the strip): 27.
6. The peanut's damage: 3.0% (the raw table's base), 3.3-15% (UFD, pre-3.1.0), 4.3-13.8% (the 3.1.0 notes): 4.3-13.8.
7. The popgun's timing: 121 / 124 (UFD, dragdown: pre-3.1.0) vs 94 / 97 (3.1.0): 94 / 97.
8. The muzzle hit: 2% (UFD) vs 3% (the changelist): not modelled (gap 1).
12. The latch's attack FAF: 37 (UFD's 36 total) vs dragdown.wiki's 21 (the throw frame's total): 37.
13. The latch's jump FAF: 43 (UFD's 42 total) vs dragdown.wiki's 46 (its "Stick" animation): 43.
14. The leap's FAF in the air: 48 (UFD's 47 total, dragdown 48): 48.
9. The misfire's length: 105 frames after the explosion (UFD's 228 less its 123) vs dragdown's 110: 105 [D].
10. The peel's wait: 61 (8.0.0) + 24 (11.0.0) = 85 = the move page's 85: 85.
11. Jab 2 into jab 3: from 7 (the strip, the changelist) vs 6 (UFD): 7.

## Estimates [E] and gaps

1. Positions of every box on a bone other than the root (the hands, arms, shoulders, knees, legs, waist): placed on our body in the
   move's pose.
2. The peel: gravity 0.1 (its falling speed 2.2 is [R]), sliding to a stop on landing (friction 0.06), never bouncing off the floor, a
   box 2.5 wide and 1.5 tall, picked up within 4.0; its bounce off a fighter (10% back, 2.0 up); the trap's place (at its foot, the
   reference's 3.0 below its root) and its kick after a slip (0.8 away, 1.5 up); the no-peel version's FAF 50 (the animation's 50); the
   item held before the move tossed on 14 and the peel made then (one hand here).
3. The peanut: its speeds and angles (1.6 at 35 degrees uncharged to 3.6 at 8 degrees full), gravity 0.08, life 80, the same shield damage
   percent at every charge (77%, from the uncharged hit); it leaves 6.0 ahead and 7.0 up.
4. The rocket: the charge's fall (gravity 0.02, fall 0.5), the stick's tilt while charging (0.75 a frame, at most 45), steering while flying
   (1.5 a frame, at most 80), the speed falling off evenly to nothing over the flight's 54 frames (only the launch speed is published),
   ledges from the launch, the ground crash only after the flight's first 3 frames; the barrels' flight (75 degrees at 1.5, gravity 0.04,
   a wobble, 40 frames of steering, 180 frames at most); the momentum stopped when charging starts in the air.
5. Gaps (not modelled): the popgun's muzzle hit (no angle, knockback or size published) and its taller later hit; a peanut caught and
   thrown back (no aerial catch for anyone). Closed in phase 2: the barrels unblockable, the crash's self damage (5% [E]), the slower
   charge-fall for the first charge of an air time only.
6. The monkey flip: see its section ("Not published").
