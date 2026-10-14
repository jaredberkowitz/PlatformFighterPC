# Reference data for the eleventh moveset (the reference game's Villager)

Mechanical reference for the eleventh port, collected 2026-10-14 for version **13.0.1** (owner decision 68), except the jab, which is the
**previous reference version's (1.1.x)** jab on that version's hit rules (owner decisions 66-67). In-game names (owner decision 65): class
and weapon **Pockets**, body and built-in fighter **Settler** (class 10). The reference character's name is not used in the game. What the
engine can and cannot do with this data is in `PROJECT_AUDIT.md`; what was built in `IMPLEMENTATION_REPORT.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Villager_(SSBU)/<move>` (29 pages: `Neutral_attack/Hit_1`, `Hit_2`, `Finisher`, the three tilts,
  `Dash_attack`, the three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(back|front|trip)`,
  `Neutral_special`, `Side_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), so every value comes
  from a named template field (`damage`, `angle`, `af`, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff`, `sdi`, `sd`, `trip`,
  `setweight`, `clang`, `rebound`, `direct`, `reflectable`, `a=f` / `g=f`, `effect`) and the angle-361 column shift cannot happen. There
  is **no up special page** for this character (checked with the wiki's page list, 2026-10-14). https://www.ssbwiki.com/Villager_(SSBU)
- **ssbwiki.com articles**: `Villager_(SSBU)` (moveset notes, the ridden rocket's damage), the previous-to-current changelist (https://www.ssbwiki.com/Template:SSB4_to_SSBU_changelist/Villager)
  (every change from the previous version), `Lloid_Rocket`, `Balloon_Trip_(move)`, `Dash` (13.0.1 table), `Item_throw`, and the
  mechanics articles `Hitstun`, `Hitstun_canceling`, `Hitlag`, `Knockback`, `Lock`.
- **The previous version (1.1.x)**: https://www.ssbwiki.com/Villager_(SSB4)/Neutral_attack/Hit_1 and `/Hit_2` (raw), https://www.ssbwiki.com/Villager_(SSB4) (moveset
  notes, update history 1.0.6 to 1.1.3: no jab change), **kuroganehammer.com/Smash4/Villager** ("Villager's Frame Data [1.1.6]").
- **ultimateframedata.com/villager** (UFD: startup, total = FAF - 1, landing lag, autocancel, stats, air times, dodges).
- **dragdown.wiki** `SSBU/Villager/Data` (the parameter file, raw).
- **kuroganehammer.com** 13.0.1 page: not used (the Ultimate pages returned 404 for the last two ports; the 1.1.6 page above did load).
- A forum thread on the previous version's jab combos (https://smashboards.com/threads/which-advanced-techniques-are-essential-to-villagers-metagame.436332/) for the combo claims (a player's
  calculation with the Kurogane calculator, not an official source).

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, formula, frame shift); **[E]** estimate. Positions on
bones other than `top` (`haver` / `havel` the held tool in the right / left hand, `have` a held item, `hip`, `handr` / `handl`, the
previous version's numbered hand bones 16 and 22) are [E] by nature. Offsets on `top` (the root at the feet: `zpos` forward, `ypos` up;
`xpos` is depth and is dropped in 2D) convert directly [D].

## Conventions

As `docs/movesets/link/REFERENCE_DATA.md`: reference frame numbers (frame 1 first; FAF as published), 1 world unit = 8 reference units,
radius in world tenths = reference x 1.25 rounded half up [D], damage before the 1.2 one-on-one multiplier, **361** the reference's 361
angle, `af` unset = angle type 0 (by position), 3 forward, 4 backward; `a=f` grounded targets only, `g=f` aerial only; shield damage
`sd` (additive) becomes `100 x (damage + sd) / damage` percent [D]. A projectile first hits on the frame after it appears.

## Attributes (13.0.1) [R unless marked]

| Value | Reference | Ours |
| --- | --- | --- |
| Weight | 92 | 92 |
| Walk | 1.092, accel 0.105 + 0.105 | `gu(1092)`, `gu(105)` + `gu(105)` |
| Initial dash | 1.815 for 13 frames (pivot 15) (ssbwiki `Dash`) | `gu(1815)`, 13 |
| Run | 1.397, accel 0.044 + 0.10978 | `gu(1397)`, `gu(44)` + `gu_fine(10978)` |
| Traction | 0.131 | `gu(131)` (doubled 262 for ground moves) |
| Air speed | 0.987, accel 0.01 + 0.05, air friction 0.01 | `su(987)`, `su(10)` + `su(50)`, `su(10)` |
| Gravity, fall, fast fall | 0.078, 1.32, 2.112 | as written |
| Jumps | full 32.5, short 15.69, double 32.5, one air jump; ledge jump 36.25 | as written |
| Air times | SH 40, FH 56, SHFF 28, FHFF 39 (UFD) | within one frame (tested) |
| Landings | light 1, heavy 8 (lag 7) | 2 (shared), 7 |
| Dodges | spot dodge FAF 26 (3-17); rolls 30 (4-15) / 35 (5-16); air dodge neutral 58 (3-30), directional down 81, side 97, up 129 (3-21) (UFD totals + 1) | as written; the diagonal ones are not separate |
| Wall jump | yes (`wall_jump_type`), 1.3 away / 2.3 up | `wall_jump 1` |
| Item throws | release 11 / 11 / 10 / 10 / 5 / 6 (forward, back, up, down, dash, drop; ground) (ssbwiki `Item_throw`) | as written |
| Size | `scale` 0.993, medium | the shared body size [E] |

## The jab: the previous version's (1.1.x) [R unless marked]

13.0.1's jab (a two-hit jab with a rapid jab and a finisher, +7 / +11 hitstun modifiers, 1% hits) is **not** used (owner decision 66).
The 1.1.x jab (no change in 1.0.6-1.1.6):

| Hit | Frames | FAF | Continuable | Boxes |
| --- | --- | --- | --- | --- |
| Jab 1 | 3-4 | 22 | from 7 | 3%, BK 10, KS 80: id 0 60 degrees r3.0 (hand bone -1.2), id 1 80 degrees r3.6 (hand bone +4.2) |
| Jab 2 | 3-4 | 24 | from 8 | 3%, BK 20, KS 120: the same boxes on the other hand |

No finisher; it repeats when held, faster when mashed (the moveset notes). SDI x1, hitlag x1, no `af`.
**Ours**: the boxes at 5.5 up, 6.8 and 12.2 ahead [E] (13.0.1's reach 6.5-11.0 at 5.5 up); a press from the continuable frame carries on
at once; held, the next hit starts at the move's end [E: the version's held rule is not published]; jab 2 carries on into jab 1.

### The previous version's hit rules (E40), each with its source

| Rule | 1.1.x | 13.0.1 | Source |
| --- | --- | --- | --- |
| Knockback formula | `((p/10 + p d/20) 200/(w+100) 1.4 + 18) s + b`, d with 0.3 of staleness | the same | ssbwiki `Knockback` (Variables) [R] |
| Launch speed, decay | 0.03 x KB, -0.051 a frame | the same | ssbwiki `Knockback` [R] |
| One-on-one damage | none (x1) | x1.2 (only the damage taken, never the knockback here: sim 97) | ssbwiki `Knockback`/`Damage`; house rule [R] |
| Hitlag | `floor(floor((d x 0.3846154 + 5) h e) c)`, crouch on the victim only | `floor(d x 0.65 + 6)`..., crouch on both | ssbwiki `Hitlag` [R] |
| Hitstun | `floor(KB x 0.4) - 1`, **+1 when tumbling** (or electric) | `floor(KB x 0.4) - 1` always | ssbwiki `Hitstun` [R] |
| Launch speed-up | none | "balloon" speed-up from a hitstun FAF of 31 (x1 to x6) | ssbwiki `Hitstun`, `Knockback` [R] |
| Gravity boost | `(g - 0.075) x 5` on every tumbling launch | none in 70-110 (the shared 1.8 fall there instead) | ssbwiki `Knockback` [R] |
| Vertical launch fall | the victim's own | homogenised to 1.8 for 70-110 degrees | ssbwiki `Knockback` [R] |
| Hitstun cancel | air dodge from 40 below 2.5, aerial from 45 below 2.0 | the same parameters | ssbwiki `Hitstun_canceling` [R] |
| Rage | x1 to x1.15 (35% to 150%) | x1 to x1.1 | ssbwiki `Knockback` (r), `Rage` [R] |
| Crouch cancel knockback | x0.85 | x0.85 | ssbwiki `Knockback` [R] |
| Hitstun modifiers | none | per move | ssbwiki `Hitstun` [R] (the old jab has none) |
| Locks | 3-lock limit, forced get-up | 2-lock limit, all get-up options | ssbwiki `Lock` [R]; **not relevant here**: the old jab's 60 / 80 degrees lift a lying fighter, so it never locks [D] |
| Non-tumble landing in hitstun | lands, hitstun carries on (+1 frame) | the same | ssbwiki `Hitstun` [R] (the engine's rule already) |

Ultimate's 4-frame minimum flinch is not in the engine for either version (no change). The shield hitlag (x0.67 here) is the same for
both [E: the previous version's shield hitlag rule (multipliers above 1 cut by 0.8 on a shield) is not modelled].

### What the old jab does (the combo claims)

- The previous version's moveset notes (the previous-version page above): jab 1 alone true-combos into the axe at kill percents against most
  floaty lightweights [R, qualitative].
- The forum thread (a player with the Kurogane calculator): about 10 frames of hitstun at 0% / 0% from jab 1 (not enough for the axe);
  from 130% a frame-perfect jab 1 > jab 2 > axe works against every character (the axe comes out on frame 6).
- The 13.0.1 changelist: the jab-to-axe kill confirm of the previous version is gone with 13.0.1's finisher.
- Ours (tests in `sim-core/tests/older_rules.rs`, the duelist at weight 90): jab 1 at 0% leaves 9 frames of hitstun
  (`floor(25.2 x 0.4) - 1`) [D]; the frame advantage after jab 1 > jab 2 is 0 to 30%, 2 at 30%, 5 at 50%, 10 at 80%, 15 at 100%, **20 at
  130%**, 24 at 150%; after jab 1 alone 6 at 130% (the table in `IMPLEMENTATION_REPORT.md`).

## Normals (13.0.1) [R unless marked]

Frames in the reference's numbering. Positions on non-`top` bones [E].

| Move | Frames | FAF | Hits |
| --- | --- | --- | --- |
| Forward tilt (umbrella, `haver`) | 8-11 | 34 | 9%, 45 (type 3), BK 80, KS 50; canopy r4.5, edges r2.5, shaft r2.0 |
| Up tilt (stick, `havel`) | hit 1 7-18, hit 2 22-25 | 48 | hit 1: 5%, set weight, hitlag x0.8, +2 hitstun on every box (glitch); grounded-only 108, BK 0, KS 100, FKV 75 (r3.8, r4.0); aerial-only 135, BK 35, KS 10 (r4.0). Hit 2: 6%, 80 (type 3), BK 50, KS 140, hitlag x1.2 (r5.0, r4.0). Head and arm intangible 7-25 (8.0.0) |
| Down tilt (weed) | 9-11 | 38 | trip 30 on both; close 12% 75 BK 50 KS 95 r5.0 at (3, 3); far 10% 70 BK 50 KS 95 r6.0 at z 10-2, grounded only |
| Dash attack (flowerpot, `have`, `direct=f`, reflectable) | clean 9-14, late 15-22 (on the ground), 15-68 (off a ledge) | 45 | 10% / 6%, 70 (type 3), BK 65, KS 60, r4.2 / 3.7, hitlag x0.6, SDI x0 |
| Forward smash (bowling ball, `direct=f`, reflectable) | charge 16-17; early 25-27, mid 28-30, off a ledge 31-139 | 50 | 15% / 17%, 45 (type 3), BK 70, KS 95, r6.0 / 3.7, SDI x0; full charge x1.2, held 60 frames at most |
| Up smash (firework) | charge 7-8; hit 1 12-13; hits 2-5 17-18, 21-22, 25-26, 29-30; hit 6 33-34 | 54 | hit 1 grounded only 3%, FKV 120 (BK 0 KS 100), hitlag x0.5, SDI x0.1, type 3, 87 (r4.0, z 0.5-2.5, y 5) and 115 (r6.0, z 6); hits 2-5 1.5%, set weight, FKV 30 (KS 0), SDI x0: 110 BK 55 at y 19, 185 BK 40 at y 25, r2.8; hit 6 4.6%, 90, BK 30, KS 220, r9.0 at y 24, hitlag x1.5. Hits 2-6 are projectiles (no hitlag for the user) |
| Down smash (shovel) | charge 3-4; hit 1 9 / 10-11; hit 2 29 / 30-31 | 50 | 60 degrees; clean 6% r4.0 at z ±13, late 3% r6.0 at z ±12, y 3, hitlag 0 late; grounded: bury, BK 30, KS 60 / 50; airborne: BK 50, KS 70 / 60; sd 0, SDI x0, `rebound=f`; hit 1 type 3, hit 2 type 0; `noprone` |
| Neutral air | 3-10, 11-23 | 36 | 9% / 5%, 361, BK 20, KS 100; hip r5.0 / 4.0, hands r3.6 / 3.0 |
| Forward air (slingshot pellet, reflectable) | 10-12, 13-18, 19-23 | 40 | 7 / 4 / 2.5%, 361 (type 3), BK 55 / 10 / 10, KS 100, r5.0 / 1.2 / 1.2, sd -3.5 / -2 / -1 |
| Back air (pellet behind) | 13-15, 16-21, 22-26 | 36 | 9 / 5 / 3%, the same, sd -4.5 / -2.5 / -1.5 |
| Up air (turnips: 1 / 2 / 3 at 30 / 30 / 40%) | 6-7, 8-21 | 43 | 85, BK 40, KS 100; 3: 13% r7.5 / 6% r4.8; 2: 10% r6.5 / 5% r4.5; 1: 8% r5.2 / 4% r3.8; at y 20 / 18 |
| Down air (turnips) | 11-12, 13-26 | 43 | clean: a 270 box (KS 80, BK 20 / 10 / 0, r3.2 / 3.0 / 2.7) inside a 45 box (BK 30 KS 100, r7.5 / 6.5 / 5.2); 13 / 10 / 8%; late 45, 6 / 5 / 4%, r4.8 / 4.5 / 3.8; below the feet |
| Grab | standing 14-16 (FAF 44), dash 16-18 (48), pivot 17-19 (45) | | r5.0 / 4.0 / 5.0 at y 5.5 along z 4-14 / 4-13 / 5-14 (grounded box) |
| Pummel | 3 | 7 | 1.3%, 361 (type 3), FKV 30 KS 100, r5.0 at (10, 7), hitlag x2.1, no clank |
| Forward throw | release 15 | 46 | 9%, 45, BK 108, KS 10, no hitlag |
| Back throw | turn and release 11 | 46 | 11%, 135, BK 46, KS 88 |
| Up throw | release 13 | 46 | 10%, 90, BK 114, KS 34, no hitlag |
| Down throw | 6 frames of hitlag on 17, release 18 | 36 (without the hitlag) | 6%, 70, BK 78, KS 60, no hitlag |
| Edge attack | 23-25, intangible 1-25 / 1-20 / 1-12 | 56 | 9%, 45 (type 3), BK 90, KS 20, r5.0, z 6.5 to -4, y 5, sd +1, no clank |
| Get-up (back / front) | 16-17 and 20-21, intangible 1-21 | 46 | 7%, 48, BK 80, KS 48, r5.0, sd +8, no clank (front: front then back) |
| Get-up (trip) | 19-20, 28-29, intangible 1-7 | 50 | 5%, 361, BK 60, KS 50, r4.5 (no slot: one get-up attack) |

Landing lag: neutral 8, forward / back 14, up / down 12 (interruptible 9 / 15 / 13 on the landing pages; the parameter file's
`landing_attack_air_frame_*`). Autocancel: neutral 1-2 / 27-, forward and back 1-2 / 30-, up 1-5 / 39-, down 1-10 / 36- (ssbwiki).

## Specials (13.0.1)

- **Side special: the rocket.** UFD: startup 52, total 35 (FAF 36), hit 52-116 (flying) and 117-176 (slowing), +27 on shield [R].
  ssbwiki raw: flying 7%, 60 (type 3), BK 65, KS 75, r3.3, hitlag x0.8, sd -3.5, reflectable; slowing 5%, 55, BK 60, KS 75, r3.0, sd
  -2.5; the explosion 12%, 68, BK 65, KS 90, r7.0, never clanks, sd -6; on a wall 12%, r6.5 at y 3.1 [R]. Ridden: 15.4% / 11% / 18%
  (ssbwiki `Villager_(SSBU)`) [R]. Behaviour (ssbwiki `Lloid_Rocket`, moveset notes) [R, qualitative]: held, the user rides it, the stick
  changes its speed, jumping off / a wall / the end of the flight leave the user helpless, a hit on a fighter bounces the user up and does
  not; one out at a time (a second use does nothing); it stops the user's momentum; it explodes on contact with a fighter or a wall,
  slows to a stop and explodes by itself after "a little less than the length of" a large stage. **Not published**: when it appears,
  when the hold is read, its speeds, the ridden speeds, the bounce and the hop (gaps 1-3).
- **Up special: the balloons.** UFD: landing lag 30 from the special fall, 20 with the balloons [R]. Behaviour (ssbwiki `Balloon_Trip_(move)`,
  moveset notes) [R, qualitative]: special pressed or held flaps up, nothing pressed sinks, the stick steers; after a few seconds the
  balloons let go (helpless); attack lets them go; they can be popped, one at a time, a popped pair leaving the user helpless with a
  little hop; hitting the user's body instead lets it use the move again; the helium recharges slowly (13.0.1: slower than before) and
  when low only one balloon comes out, for a shorter flight. **Not published**: the flight time, the flap, gravity, fall and air speeds,
  the balloons' size, place and hit points, the recharge rate and the one-balloon threshold (gaps 4-7).
- **Neutral special (Pocket), down special (Timber)**: researched for phase 2 (ssbwiki raw `Neutral_special`, `Down_special` are saved);
  not built.

## Conflicts

1. Down air autocancel: ssbwiki raw 1-10 / 36- vs UFD 1-13 / 39-. **Ours: ssbwiki** (the raw frame strip; the changelist's 1-7 → 1-10).
2. Spot dodge: UFD lists 20/25, the changelist FAF 26. **Ours: 26.**
3. The old jab's hitstun at 0%: the forum post says 10 frames; the version's formula gives 9 (it takes one off). **Ours: 9.**

## Gaps (unpublished; [E] in the build)

1. The rocket's appearance frame (20), the hold's frame (16), its flight speed (1.7 / frame) and slowing (to 0 over 60 frames, dropping).
2. The ridden speeds (1.7 + up to 0.6 by the stick), what the user does while waiting (stays put).
3. The ridden bounce (2.0 up for 6 frames, FAF 25), the hop off (1.5 up for 4, FAF 17), the crash (FAF 27).
4. The balloons' flight time (300 frames), recharge (6 / 16 a frame), one-balloon threshold (half).
5. The flight's physics: gravity 0.06, fall 1.0, flap 1.4 (two) / 0.9 (one), every 6 frames pressed / 12 held, take-off 1.6 for 3 frames.
6. The balloons' circle (radius 0.55 world units, 3.0 up) and hit points (any hit pops one).
7. The pop's hop (1.0 a frame).
8. The pot's and the ball's flights (from 8.0 / 8.8 up, landing on frames 23 / 31), the pellets' speed (2.8) and drop (0.08).
9. The old jab's held rule (at the move's end).
