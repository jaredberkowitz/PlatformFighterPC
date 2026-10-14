# Reference data for the thirteenth moveset

Mechanical reference for the thirteenth port, collected 2026-10-14 for version **13.0.1**. In-game names (owner decision 65): class and
weapon **Dagger**, body and built-in fighter **Phantom** (class 12). Neither the reference character, its moves nor its partner are named in
the game or in this repo's text; the moves are described ("the gun", "the curse bolt", "the guard", "the partner"). What the engine can
and cannot do with this data is in `PROJECT_AUDIT.md`; what was built in `IMPLEMENTATION_REPORT.md`.

## Sources and version

- **ssbwiki.com** per-move pages under the character's 13.0.1 page (`Neutral_attack/Hit_1` to `Hit_3`, the three tilts, `Dash_attack`, the
  three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(front|back|trip)`, `Side_special`,
  `Up_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), every value from a named template field
  (`id`, `part`, `rehit`, `damage`, `sd`, `angle`, `af`, `bk`, `ks`, `fkv`, `setweight`, `r`, `bn`, `ypos`, `zpos`, `ff`, `sdi`, `clang`,
  `rebound`, `direct`, `a=f` / `g=f`, `effect`, `hitbits`) and the timing tables, with the separate "without" / "with" (the partner)
  tables. There is **no neutral special page** for 13.0.1 (the wiki's page list, 2026-10-14).
- **ssbwiki.com articles**: the character's 13.0.1 page (attributes table, the moveset table with the partner's damages, update history
  3.0.1 to 13.0.0), the gauge's article (its technical data: the rates, the multipliers, the drain, the KO values, the player table), the
  partner's article, and the four special articles (the gun, the curse bolt, the hook / wings, the guard / counter / reflector).
- **ultimateframedata.com** (UFD: startup, total = FAF - 1, landing lag, autocancels, the partner's values in parentheses, the gun's six
  forms with and without the partner, the gauge notes, dodges).
- **dragdown.wiki** (the parameter file, raw: ground and air numbers, jumps, `shoot_dash_speed_f` / `_b`; its frame data, including the
  gun and the partner's versions; its "total" is the FAF).
- ssbwiki.com `Dash` (13.0.1 table: 10 frames) and `Item_throw` (the release frames: the common 8 / 11 / 12 / 7 / 5 / 6).
- A web search for the gun's ranges and the summon's frames found no published numbers (gaps 1-3).

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, formula, frame shift); **[E]** estimate. Positions on
bones other than `top` (`knife`, `armr`, `kneer`, `kneel`, `legr`, `legl`, `footr`, `footl`, `throw` (the partner's hand)) are [E] by
nature. Offsets on `top` (the root at the feet: `zpos` forward, `ypos` up) convert directly [D].

## Conventions

As `docs/movesets/villager/REFERENCE_DATA.md`: reference frame numbers (frame 1 first; FAF as published), 1 world unit = 8 reference units,
radius in world tenths = reference x 1.25 rounded half up [D], damage before the 1.2 one-on-one multiplier, **361** the reference's 361
angle, `af` unset = angle type 0 (by position), 1 by travel, 3 forward, 4 backward; `a=f` grounded targets only, `g=f` aerial only; `part=1`
the partner's own box; shield damage `sd` (additive) as `100 x (damage + sd) / damage` percent [D].

## Attributes (13.0.1) [R unless marked]

| Value | Reference | Ours |
| --- | --- | --- |
| Weight | 93 | 93 |
| Walk | 1.12, accel 0.2 + 0.11 | `gu(1120)`, `gu(200)` + `gu(110)` |
| Initial dash | 1.9 for 10 frames (pivot 12) | `gu(1900)`, 10 |
| Run | 2.06, accel 0.04 + 0.11 | `gu(2060)`, `gu(40)` + `gu(110)` |
| Traction | 0.13 | `gu(130)` (doubled 260 for ground moves) |
| Air speed | 1.1, accel 0.01 + 0.07, air friction 0.01 | `su(1100)`, `su(10)` + `su(70)`, `su(10)` |
| Gravity, fall, fast fall | 0.127, 1.63, 3.097 | as written |
| Jumps | full 32.5, short 14.2, double 34.0 (sideways x0.9), one air jump, squat 3; ledge jump 36.25 | as written; `air_jump_drift` 90 |
| Air times | SH 30, FH 45, SHFF 20, FHFF 30 (UFD) | within one frame (tested) |
| Landings | light 1, normal 3, heavy 4 (lag 3) | 2 (shared), 3 |
| Dodges | spot dodge 25 (3-17), rolls 30 (4-15) / 35 (5-16: not separate here), air dodge neutral 48 (3-30), directional down 68, side 84, up 101 (3-21) (UFD totals + 1; diagonals not separate) | as written |
| Wall jump | yes, 1.3 away / 2.7 up | `wall_jump 1` |
| Item throws | 8 / 11 / 12 / 7 / 5 / 6 | `ItemThrows::STANDARD` |
| Size | `scale` 0.98, a thinner, slightly shorter hurtbox | the shared body size [E] |

## The gauge and the partner [R unless marked]

| Rule | Reference | Ours |
| --- | --- | --- |
| Capacity | 100 units | `max` 100 |
| By itself | 0.00925 a frame (10,810 frames from empty); not on a revival platform | `per_frame` 0.00925 (606 / 65536 in the fixed point: 10,815 frames [D]) |
| Damage taken | 1.3 a percent actually taken (the 1v1 multiplier counted: 76.92% fills it) | `per_taken` 1.3 |
| Damage dealt | nothing (the sources list only time, damage taken, the guard and team events) | **owner decision 80 (sim 123)**: `per_dealt` 1.3, a house rule [E], not the reference's: the phantom's and its partner's hits on fighters, no stock-gap or crowd multiplier, not shields, not the curse's ticks, nothing while the partner is out |
| Stock gap | x1.625 one stock behind first place, x1.95 two or more (of 1.3) | `behind` 1250 / 1500 per mille |
| Players | 2: 1.0; 3: 1.0 / 1.1 / 1.1667; 4: 1.0 / 1.2 / 1.3333 by the gap (5-8 players not possible here) | `crowd` rows |
| The guard | `m x 0.4 x 6.7368 x damage` (the guard takes 0.4 of the hit): 28.6% fills it | `Move::gauge_gain` 6737 per mille on the guard's moves, `armor_damage` 400 |
| Full | the partner comes for 1800 frames (30 s); the gauge does not fill while it is out | `mode_frames` 1800 |
| Damage while out | 16 frames a percent (2-3 players; 10.2 with 4; since 7.0.0, was 12): 112.5% ends it | `mode_taken` 16 / 16 / 10.2 (rounded to the nearest frame [E]) |
| Hitlag | the clock runs in hitlag (UFD) | counted every frame |
| Stun | a stun (a broken shield; sleep and others not in this game) stops the clock | `halts` |
| KO | the gauge restarts at 20 units, 25 two stocks behind, 30 three or more (times the player row); the last stock empties it | `ko_keep` 20 / 25 / 30 |
| Specials | each changes while the partner is out; using them does not end it | `limit_version` on each special, `keeps` |
| Body | no stat changes (the partner adds boxes; "longer rolls" with it: gap 4) | no multipliers |
| Arrival / leaving | arrival can wait up to 31 frames for an uninterruptible animation; standing, an intangible summon animation; leaving, an uncancellable animation, invincible for most of it, the fall stopped | **not modelled: owner decision 81** (close to instant; a decision, not an estimate) |
| The partner's moves | its boxes (`part=1`) hit alongside the fighter's own (the total is the sum); the fighter's own boxes may change ("awakened": size, knockback, angle, hitlag), kept to the attack's end if the partner leaves during it; no partner on the pummel, throws, floor and edge attacks | `Hitbox::part`, `Hitbox::mode`, `Fighter::move_powered` |

## Normals [R unless marked]

Damage in %, `p:` the partner's own box (part 1). Positions on the knife, arms and legs are [E].

| Move | Frames, FAF | Without the partner | With the partner |
| --- | --- | --- | --- |
| Jab 1 | 4-5, FAF 24, continuable 7-24 | 2.0, 361 BK 32 KS 30 at 7.0 / 10.0 ahead, 7.5 up (r 2.0); far box 12.5-15.2 ahead 180 BK 32 KS 20 (fighters only); type 3, hitlag x1.6 | the same + p: 0.7 361 / 180, BK 33 KS 30, r 2.5, hitlag x1.8, no clank |
| Jab 2 | 3-4, FAF 24, continuable 11-24 | 1.5, 361 BK 25 KS 30 (r 2.0, 10.0 / 13.0-14.5 ahead, 8.0 up); 180 BK 30 KS 30 at 18.0 (r 2.5, fighters only) | + p: 0.7 361 / 180 BK 25 KS 30 |
| Jab 3 | early 3, late 4; FAF 35 | 4.0 361 BK 60 KS 80, hitlag x1.6, the late box at 15.0 no clank | hitlag x1.95, late box at 17.0; p: 3.1 361 BK 65 KS 135 hitlag x2 |
| Forward tilt | hit 1 8-9, hit 2 13-15 (13-19 with); FAF 35 | hit 1 3.0 set weight BK 0 KS 100: up 180 / fkv 10 and 35 / 25; straight 93 / 20, 40 / 20; down 93 / 10, 40 / 20. Hit 2 5.0 BK 45: up 45 KS 128, straight 35 KS 110, down 25 KS 105 | hit 2 BK 60, KS 121 / 106 / 106, r 4.0, hitlag x1.7, shieldstun x2; p: the same 5.0 |
| Up tilt | 8, 9, 10-19 (rehit 3), final 20-23; FAF 41 | 4.0 set weight (grounded 120 / fkv 150 etc., airborne 100 / 60, 115 / 60); 9: 130 / 20, 367 / 20, 133 / 0; loop 1.0 367 BK 20 KS 20 r 5.0 hitlag x0.3; final 1.0 90 BK 35 KS 270 | p: 0.7 367 BK 50 KS 20 r 6.0 on 11-19, electric; final 3.9 90 BK 50 KS 180 hitlag x1.3 electric |
| Down tilt | 8-9, 10-11, 12-14; FAF 38 | 6.0 BK 75 KS 66, 75 / 82 / 85, the knee r 2.6 | the root box r 3.5 (BK 55, hitlag x1.5); p: 7.0 82 BK 70 KS 95 / 98 / 99 |
| Dash attack | 6-7, 15-21; FAF 46 | hit 1 2.0 BK 0 KS 95 fkv (grounded 35 / 70, 40 / 80, 50 / 50; airborne 25 / 40, 10 / 30, 15 / 25); hit 2 6.0 361 BK 60 KS 85 | hit 2 6.0 361 BK 85 KS 82 r 5.0 hitlag x1.7 (three phases) + p: the same |
| Forward smash | 16, 17-18 (19 with); charge 5; FAF 48 | 14.0 361 BK 40 KS 98 hitlag x0.8 | 14.0 hitlag x1.2, BK 0 on 19; p: 8.0 BK 50 KS 151 |
| Up smash | 10, 11-14; charge 4; FAF 52 | 12.0 83 BK 43 KS 100 | hitlag x1.5, high box 12-14 (r 5.0, no clank); p: 5.0 83 BK 59 KS 181 no clank; arm intangible 10-14 (not modelled: limit) |
| Down smash | 12-13, 16-17; charge 5; FAF 52 (conflict 2) | 12.0 32 BK 30 KS 93 / 30 BK 35 KS 93 (hit 2 no clank), hitlag x0.8 | hit 1 BK 64 KS 84, hitlag x1.3, root boxes; p: 6.0 38 BK 44 KS 158 / 30 BK 40 KS 158 |
| Neutral air | 12-27; LL 8; AC 1-4, 48+; FAF 55 | the knife 7.0 46 BK 35 KS 93 r 2.8 (a circle [E]) | p: 4.0 46 BK 50 KS 139 the other half [E] |
| Forward air | 7-8, 12-14; LL 12; AC 1-3, 42+; FAF 48 | hit 1 2.0 set weight: airborne 367 BK 70 KS 25 (knee KS 5), grounded 76 / 80 BK 73 KS 50; hit 2 5.0 361 BK 46 KS 125 | hit 2 5.0 34 BK 48 KS 148 r 6.0 hitlag x1.9; p: 8.0 34 BK 58 KS 98 |
| Back air | 7, 8 (7-8 with); LL 9; AC 1-3, 30+; FAF 32 | 9.0 43 backward BK 54 KS 90 hitlag x0.8 | 9.0 35 BK 51 KS 102 r 5.0 hitlag x1.3; p: 7.0 BK 58 KS 114 / BK 76 KS 98 |
| Up air | 5-18 (rehit 4), 20-21; LL 14; AC 1-4, 40+; FAF 40 | 0.75 grounded 80 BK 20 KS 80, airborne 367 BK 30 KS 100, hitlag x0.7, SDI x1.2; final 3.0 80 BK 50 KS 155 | final r 5.0 hitlag x1.7; p: 7.0 80 BK 50 KS 126 |
| Down air | 13-16; LL 11; AC 1-3, 40+; FAF 47 | 8.0 38 BK 40 KS 100 hitlag x0.8 | 13-14 KS 60; 15-16 BK 50 KS 50 hitlag x1.5 no clank; p: 8.0 **270** BK 35 KS 96 (the meteor) |
| Grabs | 6-7 FAF 35; dash 8-9 FAF 43; pivot 9-10 FAF 38 | r 3.3 at 8.1 up, 4.5-9.0 ahead (grounded box) | no partner |
| Pummel | 1 (2 here), FAF 7 (8 here [D]) | 1.5 361 fkv 30 r 5.0 hitlag x2.2, no clank | no partner |
| Throws | F 9 / 30; B 14 / 40; U 16 / 38; D 23 / 41 | F 8.0 45 BK 65 KS 65; B 10.0 131 BK 70 KS 66; U 7.0 90 BK 70 KS 72; D 7.0 79 BK 55 KS 95; no hitlag | no partner |
| Ledge attack | 24-26, intangible 1-26, FAF 56 | 9.0 45 BK 90 KS 20 sd +1, no clank | no partner |
| Get-up (face up) | 15-16 behind, 24-25 in front, intangible 1-25, FAF 46 | 7.0 48 BK 80 KS 48 sd +8, no clank | no partner |

## Specials [R unless marked]

### Neutral special: the gun

Not projectiles: boxes in a line (they cannot be reflected or blocked by a passive shield; they pierce). Three bands by distance with
no published distances (gap 1).

| Form | Frames, FAF | Without | With the partner |
| --- | --- | --- | --- |
| Shot | 12; FAF 37 (UFD; dragdown 36: conflict 3); next shot / dodge / jump from 26, below / spiral from 14 | 5 / 3 / 1 %, BK 5 / 1 / 0, KS 75 / 50 / 0; hitlag 6 / 0 / 0 frames (x0.7 [D]) | three bullets 12 / 18 / 24, FAF 42, next from 34, below / spiral from 15; 6 / 3 / 1 % BK 60 KS 50 close, follow-ups 2 / 1.5 / 1 % |
| Dodge forward | shot 20, FAF 40, intangible 4-13, speed 1.52 | 6 / 4 / 2, BK 35 / 4 / 0, KS 67 / 45 / 0 | shots 20 / 26 / 32, FAF 46; 7 / 4 / 1.5 |
| Dodge back | shot 22, FAF 46, intangible 5-14, speed 1.216 | 5 / 3 / 1 | 22 / 28 / 34, FAF 50 |
| Jump (ground) | shots 10 / 17 / 24 / 31, FAF 60, LL 14, intangible 1-6 | 3.0 / 0.8, angles 365 x3 then 125 / 90 / 45, BK 10 / 20, KS 70 / 120 | + 13 / 20 / 27 at 1.0 / 0.5 |
| Below (air) | 7 / 21 / 35 / 44, FAF 49, LL 10, extendable | 4.0 / 1.2, 50 / 45, BK 25 / 29, KS 80 / 0 | + 14 / 28 at 1.6 / 0.8 (dragdown: 1 / 8 / 15 / 22 / 29: conflict 4) |
| Spiral (air) | 12, 15, 18, 24, 27, 30, 36, 39, 42, 48, 51, 54; FAF 70; LL 20 | 3.0 each, BK 5 KS 75 | 4.5 each |

Up to four dodges in a string; the dodges are staled like rolls [R] (sim 123: counted and their intangibility delayed as a roll's, not
stretched in length: estimate 17); buffered maneuvers come 4 frames later [R] (sim 123: a dodge or jump out of a shot); the gun stalls the
fighter in the air (stall numbers [E]).

### Side special: the curse bolt

| | Without | With the partner |
| --- | --- | --- |
| Frames | the bolt 16-42, FAF 53 (since 11.0.0) | 16-32, FAF 58 |
| Contact | 1.0 361 by travel, BK 35 KS 20, r 1.2, hitlag x0.6 | 1.0 90 by travel, BK 35 KS 20, r 1.5, hitlag x0.8 |
| Burst | a frame after contact, not on a shield: 2.0 75 forward BK 20 KS 210, r 8.5 at 1.5-3.5 up, 19 frames | on a fighter or the ground: 3 x 0.5 367 (BK 5 KS 100, r 4.0, 1.5-8.0 up, rehit 5, backward early then forward since 3.1.0) then 2.5 73 BK 105 KS 105 on 16-27 |
| Curse | 1% every 45 frames for 361 (8 ticks; 11% in all) | 1.5% every 40 for 321 (8 ticks; 17% in all) |
| Path | "travels diagonally downward" (speed, angle: gap 5) | "faster and farther" (gap 5) |

### Down special: the guard, and with the partner the counter and reflector

| | Values |
| --- | --- |
| Guard | super armor 3-19; held, up to 120 frames; FAF 53 unheld, 33 frames of lag after a held guard (UFD; ssbwiki's "FAF 31" is the backhand's: conflict 5); 0.4 of the damage; hitlag x1.5, hitstun x1.1 (UFD) |
| Backhand | let go within 33 frames of a hit: 2.4 50 fkv 82 BK 0 KS 100, r 12.0 at 10.0 up, 3.0 behind to 3.0 ahead, hitlag x1.25, SDI 0, no clank; hit 8-9, intangible 1-14, FAF 31 |
| Counter / reflector stance | intangible 3-4; counter / reflect 4-31; FAF 57 (since 11.0.0); melee countered at x1.6 (at least 12%, at most 50%); projectiles reflected (a "larger, higher" search box since 4.0.0) |
| Counterattack | 12.0 (the caught damage x1.6), 361 BK 80 KS 51, r 20.0 at 10.5 up, 5.0 behind to 10.0 ahead, hitlag x0.75, no clank; hit 5-7, intangible 1-8, FAF 39 |
| Reflection | reflects 1-29 at x1.6 damage, x1.9 speed, shots up to 50%; arm intangible 1-29; FAF 42 |

### Up special: the hook (phase 2, sim 123) and the wings

Sources (2026-10-14): ssbwiki.com's 13.0.1 up special page (raw hitbox and timing tables), its articles on the hook move and on tether
recoveries, the character page's update history (a cited post on the 4.0.0 change), ultimateframedata.com, dragdown.wiki.

| | Values |
| --- | --- |
| The hook (ground) | a grab box [R]: early (frame 20) id 0 r 3.0 on the `throw` bone, a capsule from (1.5, 1.5, 0.7) to (1.5, -15.0, -6.4) (16.4 long: the rope); ids 1 / 2 r 5.5 at (1.5, 1.5, 0.7) (the head); late (21-26) the head; FAF 60 (UFD 59 total) [R]. It grabs fighters in the air ("can grab opponents, dragging them towards" the fighter) and items [R]; "can be shielded despite its grab properties", shields or reflectors cancel it [R]; not on fighters in hitstun (phase 1 reading [R]) |
| The pull and throw | throw id 0: 11.0%, 78 degrees, BK 75, KS 40, FKV 0, release frame 6, FAF 30 (animation 41) [R] |
| The hook (air) | no grab ("the air version cannot grab") [R]; early (20) id 0 r 2.7 on `throw` (1.5, 2.0, 2.0) and id 1 r 2.7 on `top` from 22.0 to 38.0 up, 3.2 to 10.6 ahead; late (21-25) id 0; each 5.0%, sd +1.0, 55 degrees, BK 76, KS 56, hitlag x1.2, clang on, rebound off [R]; FAF 45 (UFD 44 total) [R]; "does not induce special fall" (UFD) [R]; a slight vertical boost on the first use in an air time, none on a second (ssbwiki, dragdown) [R]; the stick tilts the rise a little [R, not modelled: limit] |
| The tether | "tethers on frame 12" (UFD) [R]; automatically connects to an open edge in reach, "a large ledge grab range", "very high vertical reach", "low horizontal range" [R, no numbers]; three tethers before landing, then no catch [R]; down cancels the grapple [R]; a short wait before tethering again [R, no number]; tethers count toward the ledge grab limit and do not occupy the edge (several can hang) [R]; no wall latching (tethers "can only grab onto unoccupied edges as opposed to any wall") [R]; 4.0.0: `air_resist_hang` 0.27 to 0.24, `fighter_offset` 6 to 8 (an undocumented change cited on the character page) [R] |
| The wings (with the partner) | no hitbox; intangible 1-25 from the ground, 3-25 from the air; 98 frames; the special fall, landing lag 30; angled a little by the stick; "higher than most up specials" (distance: gap 6) |

## Conflicts (resolved as listed)

1. The owner's brief: the gauge fills "by damage taken and dealt". Every source lists damage taken (and the guard, time and team events)
   only: dealt gives nothing here (question 1).
2. Down smash FAF: UFD 51 total and dragdown 52 (FAF 52) against ssbwiki's interruptible 46: **52**.
3. The gun's FAF: UFD 36 total (FAF 37) against dragdown 36: **37** (the 25-frame shot rhythm 12 / 37 / 62 agrees).
4. The partner's shots below: UFD 7 / 14 / 21 / 28 / 35 / 44 against dragdown 1 / 8 / 15 / 22 / 29: UFD's frames, the follow-ups on 14
   and 28 [E: which of UFD's are follow-ups].
5. The guard's FAF: ssbwiki "FAF 31 normally" against UFD / dragdown 52 minimum: **53** (31 is the backhand's FAF).
6. The hook's pull: UFD lists an air row "grappling attack" with startup 5, total 28 and 11.0 damage, mixing the throw (11%) with the air
   hit (5%): ssbwiki's throw (release 6, FAF 30) and air hit (5%) are used.
7. The hook's air hit clanking: phase 1 wrote "no clank"; the raw table reads clang on, rebound off: **clanks, does not rebound**.

## Estimates [E] and gaps

1. **The gun's band distances** (kept as estimates: owner decision 82): close to 20 reference units, middle to 44, far to 72 from the muzzle at 4 ahead, 9.5 up, radius 2.5
   (only "about half the stage" is published). The held fire's rhythm (a shot each 36 frames) and the follow-up bullets' knockback.
2. **The partner's arrival**: the reference waits up to 31 frames for an uninterruptible animation and plays an intangible summon when
   standing; here the partner is there the frame the gauge fills (no animation, no intangibility).
3. **The partner's leaving**: an uncancellable, mostly invincible animation that stops the fall; not modelled.
4. **Longer rolls with the partner**: no numbers published; not modelled.
5. **The curse bolt's path**: 1.6 reference units a frame straight on the ground and 30 degrees down from the air; the partner's 3.0;
   the stall in the air (gravity 0.04, top fall 1.0).
6. **The wings' flight**: 2.2 a frame on 6-30, 1.4 on 31-45; the stick's 25 degrees; ledges in front from 10.
7. **Body boxes on bones**: every knife, arm, leg and partner-hand box position; the neutral air's circle (8 points, 9 units round the
   body's middle); the forward tilt's step forward and the forward smash's step (not modelled).
8. **Two hits on one frame** (the fighter's and the partner's): both damages; the partner's launch, applied second, stands.
9. **The guard's held limit**: 120 frames counted from the guard's start; the release window counted outside hitlag.
10. **The stance's bubbles**: the counter bubble r 10 at 10 up, the reflector search r 12 at 11 up, the reflection's field r 12 at 10 up.
11. **The gun's air stall**: vertical speed stopped, gravity 0.03, top fall 0.8.
12. **Angle 365** (built in sim 123: half the attacker's momentum, ssbwiki.com `Autolink_angle`): its grounded rule is not published (a
    grounded victim is launched the same, a launch that does not rise leaves it standing) [E]; the partner's three follow-up shots on the
    jump use it too [E: their angle is not published].
13. **The tether's reach** (sim 123): the ledge's corner up to 40.0 of the reference's units above the feet (the hook line's far end 38.0
    plus its 2.7 radius) and 0 to 16.0 in front [E]; the catch window to frame 25 (the head's last frame) [E].
14. **The tether's hang and reel**: hanging 8 frames, reeled to the ledge in 12 [E: no published frame counts]; the hang's speed loss
    0.24 a frame is the reference's `air_resist_hang` read as a drag [E: the reading]; its `fighter_offset` 8 is not used (the hang
    point is where the fighter caught the ledge). A ledge taken during the reel throws the fighter off in the knocked-off hop [E: an
    older game's "forced hop"]. The wait before another tether is the body's ledge regrab wait (30) [E].
15. **The hook's grab boxes** (ground): the head held at the line's far end on 21-26 [E: the reference's `throw` bone moves]; a shield or
    reflector stop lets the move run to its end [E]; the caught fighter held just in front (9.0 of the reference's units) during the pull
    [E: the reference drags it in].
16. **The air hook's rise**: 1.5 of the reference's units a frame set on frame 5 [E]; a landing during the hook has the body's normal
    landing lag [E: none published].
17. **The gun's dodge staling**: counted and its intangibility delayed as a roll's, its length not stretched (limit); the buffered
    delay (4 frames, [R]) applies out of a shot only, not between dodges (limit).
