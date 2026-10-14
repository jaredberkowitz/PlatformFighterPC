# Reference data for the ninth moveset (the reference game's Yoshi)

Mechanical reference for the ninth port, collected 2026-10-13 for version **13.0.1**, before anything is built. Proposed in-game
names (not registered; see `PROJECT_AUDIT.md`): the **Clutch** class and weapon on the **bounder** body, built-in fighter **Bounder**.
The reference character's name is not used in the game. What the engine can and cannot do with this data is in `PROJECT_AUDIT.md`; the
format follows `docs/movesets/donkey_kong/REFERENCE_DATA.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Yoshi_(SSBU)/<move>` (28 pages: `Neutral_attack` and its `Hit_1` / `Hit_2`, the three tilts,
  `Dash_attack`, the three smashes, the five aerials, `Grab`, `Pummel`, the four throws, `Edge_attack`, `Floor_attack_(back|front|trip)`,
  `Neutral_special`, `Side_special`, `Up_special`, `Down_special`), read as **raw wikitext** (`index.php?title=...&action=raw`), so every
  hitbox value comes from a named template field (`damage`, `angle`, `af` angle type, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff`
  hitlag multiplier, `sdi`, `sd` shield damage, `effect`, `trip`, `setweight`, `rehit`, `clang`, `rebound`, `direct`, `hitbits`, `g=f` /
  `a=f`) and the angle-361 column shift cannot happen. The four specials' pages also give the move **parameters** (speeds, gravities,
  the egg's timer formula, the throw's hop formula). https://www.ssbwiki.com/Yoshi_(SSBU)
- **ssbwiki.com articles** `Yoshi_(SSBU)` (attributes template, moveset notes, update history), `Egg_Lay`, `Egg_Roll`, `Egg_Throw`,
  `Yoshi_Bomb`, **`List_of_armored_attacks_(SSBU)`** (the double jump's armor row), `Armor`, `Double_jump`, `Shield` (the egg shield
  section), `Dash` (current-game table), `Template:SSB4_to_SSBU_changelist/Yoshi` (changes from the earlier game, used only to understand
  the current behaviour) and `Template:UpdateList_(SSBU)/<version>` for every version that touched this character (2.0.0, 3.0.0, 3.1.0,
  4.0.0, 5.0.0, 7.0.0).
- **ultimateframedata.com/yoshi** (UFD: startup, total frames, landing lag, notes, active frames, dodges, jump air times, stats). UFD's
  "total frames" is FAF - 1. https://ultimateframedata.com/yoshi
- **dragdown.wiki** `SSBU/Yoshi/Data` (the parameter file and a frame-data table; "total" = FAF - 1) and `SSBU/Yoshi` (player notes: the
  double jump's armor window and the egg shield). https://dragdown.wiki/wiki/SSBU/Yoshi/Data
- **kuroganehammer.com**: the Yoshi page (https://kuroganehammer.com/Ultimate/Yoshi) returned **HTTP 404** on 2026-10-13 (the character index still links it;
  the archived copies are 404 too), so it could not be used. Every startup and FAF below is cross-checked between ssbwiki.com, UFD and
  dragdown instead (three sources, as for the other ports).

**Update history applying to 13.0.1** (ssbwiki.com update lists, every entry for this character): **2.0.0** forward air's ledge
lockout 71 → **68** frames; grab invincibility during the grab release and throw animations. **3.0.0** Egg Throw's shield damage 0 →
**-3**. **3.1.0** bug fix: Egg Lay no longer makes a victim caught in hitstun fast fall. **4.0.0** the forward smash's charge pushes
opponents away less (jostling, not modelled). **5.0.0** a grab can be buffered out of a roll or spot dodge (the engine's buffer already
allows it). **7.0.0** the shield's hitbox is bigger (global shield size, not modelled); **the head and legs are briefly intangible after
the shield goes up** (no frames published; not modelled: owner decision 5). No entry after 7.0.0. Every table below is the post-13.0.1
data. No data from the earlier games was used; where they differ (the earlier games' delayed double jump, the earlier Egg Throw's two
hits, the earlier egg's free air drift) it is called out.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift, a reading of a field);
**[E]** estimate. Hitbox **positions** on a bone other than `top` (`tail1` / `tail2` / `tail3` the tail, `head`, `mouth2` the tongue tip,
`legl`, `footl`) are [E]: the reference attaches them to its own skeleton, so ours go on our body where that bone is in the move's pose.
Offsets on `top` (the character's root at the feet: `zpos` forward, `ypos` up) convert directly [D]. A reference capsule (`zpos=7.0 to
10.0`) is two or three circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first, FAF as published: ssbwiki.com's "Interruptible" is the FAF); the sim stores
  them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius in world tenths = reference value x 1.25, rounded half up [D] (the house rule). Speeds
  and gravities: reference units a frame / 8 = world units a frame [D].
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; **361** is the reference's 361 angle ("angle 361"), **367**
  the autolink angle. H = hitlag multiplier (`ff`), SDI = SDI multiplier.
- **Angle type** (`af`): unset = 0 (by position, away from the hitbox's owner), 3 forward (the owner's facing), 4 backward.
- **Targets**: `a=f` cannot hit aerial targets (grounded only), `g=f` cannot hit grounded targets (aerial only) [R].
- **Shield damage**: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd` becomes
  `100 x (damage + sd) / damage`, rounded [D].
- `clang=f rebound=f`: never clanks, never rebounds (E3c, `Hitbox::clanks`) [R]. `rebound=f` alone: clanks, the owner does not rebound
  (E3g, planned for the Bracers). `direct=f`: an indirect hit (counter behaviour) [R], not modelled. `reflectable=t`, `absorbable=t`:
  the projectile can be reflected / absorbed [R].
- `trip=N`: an N% chance to trip a grounded target [R] (E20, approved for the Bracers by owner decision 39).
- `setweight=t`: knockback worked out at weight 100 (`Hitbox::set_weight`) [R]. `rehit=N` on a melee box: the same target can be hit
  again N frames after its last hit by it [R]. `effect=Rapid`: presentation only (the multi-hit's hit effect).

## Attributes [R]

ssbwiki.com attributes template, agreeing with UFD and the parameter file (dragdown):

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump (heights) | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **104** | 1.208 | 1.98 | 2.046 | 0.095 | **1.344** (the fastest in the game) | 0.03 + 0.068 | 0.005 | 0.08 | 1.29 / 2.064 | 36.09 / 14.43 / **51.56** | 3 | 1 |

Weight 104: a heavyweight that is one of the floatiest fighters (low gravity, slow fall) (ssbwiki.com attributes text) [R]. The double
jump's 51.56 is the second-highest in the game (`Double_jump`, current-game table) [R].

Parameter file (dragdown) [R]: `walk_accel_add` **0.105**, `walk_accel_mul` **0.168** (ours: `walk_accel` 0.105, `walk_accel_stick`
0.168); `walk_speed_max` 1.208; `run_accel_add` **0.044**, `run_accel_mul` **0.1056** (ours: `run_accel` 0.044, `run_accel_stick` 0.1056;
ssbwiki.com's Dash table agrees: base 0.044, additional 0.1056, max 0.1496); `ground_brake` **0.095**; `dash_speed` 1.98;
`run_speed_max` 2.046; initial dash **14 frames**, pivot dash 16 (ssbwiki.com Dash table); `jump_initial_y` 19.8495; `jump_y` 36.09,
`mini_jump_y` 14.43, `jump_aerial_y` 51.56; `jump_speed_x` 0.91, `jump_speed_x_mul` **0.7**, `jump_speed_x_max` **1.5**,
`jump_aerial_speed_x_mul` **1.2**; `air_accel_x_mul` 0.068, `air_accel_x_add` 0.03; `air_speed_x_stable` 1.344; `air_brake_x` 0.005;
`air_accel_y` 0.08; `air_speed_y_stable` 1.29; `air_brake_y` 0.01; `dive_speed_y` 2.064; `air_ground_speed_brake` 0.015;
`landing_frame_light` 1, `landing_frame` 3, **`landing_heavy_frame` 7** (so **6** frames of heavy landing lag here [D], by the house rule
`landing_heavy_frame - 1`; ssbwiki.com's "ellag 4" is the normal landing, `landing_frame` 3 + 1 [D reading]); `landing_attack_air_frame`
n / f / b / hi / lw **7 / 12 / 11 / 8 / 17** (agreeing with UFD and with ssbwiki.com's landing "interruptible" 8 / 13 / 12 / 9 / 18);
`cliff_jump_y` **38.045**, `cliff_jump_x_speed` 0.6; `jump_count_max` 2; `scale` **0.94**; `size` `fighter_size_m`; **`shield_radius`
8.0** (the egg shield, below; shields are global here); `jostle_front` 2.0, `jostle_back` 0.8, `jostle_weight` 6.4 (no jostling in the
engine). **No wall jump** (`wall_jump_type` False, `attach_wall_type` False); the wall-tech jump speeds are `passive_wall_jump_x_speed`
**1.3**, `passive_wall_jump_y_speed` **2.3** (ours: `wall_jump` 0, `wall_jump_vx` 1.3, `wall_jump_vy` 2.3 [D], the wall-tech jump only).
**Crawls** (`squat_walk_type` True; "Yoshi can crawl", ssbwiki.com) [R]: crawling is not modelled for anyone. No tether (`air_lasso_type`
False) [R]. Footstool (`tread_*`) not modelled for anyone. `damage_fly_top_air_accel_y` 0.076944 / `damage_fly_top_speed_y_stable` 1.8 (the
shared launch fall covers them).

Jump air times (UFD) [R]: short hop **38**, full hop **60**, short hop fast fall **26**, full hop fast fall **42**. The double jump's air
time is not published (gap 1).

Dodges (UFD total frames, so FAF one more; ssbwiki.com changelist FAFs agree) [R]: spot dodge **26** (FAF 27, intangible 3-17; UFD's
"21/26" first number is not explained: gap 2), forward roll **30** (FAF 31, 4-15), back roll **35** (FAF 36, 5-16). Neutral air dodge
**58** (FAF 59, intangible **3-30**, landing lag 10); directional down **79**, diagonal down 89, sideways **107**, diagonal up 116, up
**130** (intangible **3-21**; landing lag 11-19). (These are longer than most of the cast's [R].) The engine's three directional lengths
would be down / side / up 79 / 107 / 130 (no separate diagonals) [D].

Out of shield (UFD) [R, for checks]: neutral air 6 frames, up air 8, up smash 11, shield grab 18.

Ledge (ssbwiki.com edge attack page): ledge attack intangible 1-22 on the first grab, **1-18** on the second, **1-11** on the third, none
from the fourth (ours: `ledge_option_decay_2` 82, `ledge_option_decay_3` 50 [D]).

Body size [E]: scale 0.94, `fighter_size_m`; the up smash's foot reaches about 20 reference units up and the forward smash's head 14.4
forward at 6.5 up; the bounder body is fitted to those reaches [E].

## The double jump and its armor [R]

The kit's defining body trait: a very high double jump that shrugs off hits.

| Property | Value | Source / label |
| --- | --- | --- |
| Height | **51.56** reference units (`jump_aerial_y`), the second-highest double jump | parameter file, ssbwiki.com `Double_jump` [R] |
| Shape | **not delayed** in this game: "Yoshi had a delayed double jump in [the first three games]"; the double jump cancel is "impossible ... with Yoshi from [the previous game] onward, as [he does] not have delayed double jumps" | ssbwiki.com `Double_jump`, `Double_jump_cancel` [R]. So in 13.0.1 the double jump is an ordinary velocity jump to its height under the body's gravity [R reading] |
| Check | an ordinary jump of 51.56 under gravity 0.08 starts at about 2.91 units a frame and peaks about **36 frames** in; the armor's end on frame 44 is "slightly after the apex" (dragdown) | [D]: the two sources agree with a plain jump |
| Turning | "allows him to turn around in midair" with the double jump (stick held back), "a trait normally reserved for characters with multiple double jumps" | ssbwiki.com `Double_jump` [R] |
| Horizontal | `jump_aerial_speed_x_mul` 1.2 (the reference's double-jump horizontal multiplier) | parameter file [R]; the engine sets the double jump's drift from the stick for every body (a house rule) |
| **Armor window** | **frames 1-44** of the double jump | ssbwiki.com `List_of_armored_attacks_(SSBU)` [R], dragdown ("frames 1-44 of his jump, ending slightly after the apex") [R] |
| **Kind** | **subtractive knockback armor of 120 units**: "the attack's knockback will be reduced by 120 KB, with only the remainder being applied to Yoshi. The armor will only break if said remainder is more than 0" | ssbwiki.com armored-attacks list and `Yoshi_(SSBU)` moveset notes ("subtracts 120 units of knockback, with Yoshi not flinching at all if the result is negative, or otherwise only receiving the knockback difference") [R] |
| **Minimum launch** | "the minimum knockback an attack that breaks the armor can do is **60 KB**" | ssbwiki.com armored-attacks list [R]. Read as: a breaking hit launches with `max(k - 120, 60)` units [D reading]; dragdown says "a minimum of 60 frames of hitstun" with its own "research" flag (conflict 3) |
| Ends | "Armor is only available during the double jump animation, meaning that it goes away when Yoshi attacks or air dodges" | dragdown [R]; a special also leaves the jump animation [D reading] |
| Several hits | conflict 4: dragdown "each hit reducing this threshold" (a 120-unit pool) against ssbwiki.com (each hit's knockback compared with 120) and an earlier-game player guide ("knockback NEVER adds up") | conflict 4 |
| What it does not stop | grabs (they ignore all armor, ssbwiki.com `Armor`); the damage and the hitlag are always taken | [R] |
| Knockback | computed before the one-on-one multiplier; staleness and rage do count (ssbwiki.com `Armor`, knockback-based armor) | [R] |
| Earlier game | the previous game's armor lasted much longer (about 68 frames) and did not subtract ("duration significantly shorter, but ... knockback taken is reduced if the armor is broken", changelist) | used only to read the current rule [R] |

Example [R]: "it takes over 70% from [a fully charged blaster shot of another fighter] to break Yoshi's double jump armor" (ssbwiki.com
`Armor`).

## The egg shield (reference behaviour; owner decision 5 keeps a regular shield here)

| Property | Value | Source |
| --- | --- | --- |
| Shape | an egg around the whole body instead of a bubble; it **never shrinks** (it darkens as it weakens), so it cannot be shield-poked, and it cannot be angled | ssbwiki.com `Shield`, `Yoshi_(SSBU)`; dragdown [R] |
| Size | `shield_radius` 8.0 (fixed) | parameter file [R] |
| 7.0.0 | the shield hitbox is bigger; **the head and legs are briefly intangible after the shield goes up** (frames not published) | update list [R] |
| Otherwise | in the current game the shield "no longer has practical differences from the rest other than its fixed size" (shield drop 11 frames, universal) | ssbwiki.com `Shield`, UFD [R] |
| A weakened shield | "can still be hit easily, and as such, can break more easily than a normal shield" (it never shrinks out of reach of hits) | ssbwiki.com [R] |

**Owner decision 5**: this port gets a **regular shield** (no E13, no shield geometry). Since no shield in this engine has geometry (any
hit on a shielding fighter is blocked, and nothing shrinks), every shield here already behaves as the egg does in the one respect that
matters (no pokes); only the 7.0.0 head-and-legs intangibility is left out [D].

## Normals

All frame numbers [R] (ssbwiki.com timing tables; UFD and dragdown agree on every startup and FAF unless listed under "Conflicts").
Damage, angle, BK, KS, FKV, H, SDI [R]. Radius column: reference → world tenths [D]. Positions in reference units on the named bone.

### Jab [R]

Two kicks (7% total). Quickest loop: jab 1 on 3, jab 2 on 8-9; FAF 18 / 25 (from the start of the chain).

| Part | Frames | Continue | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Jab 1 | **3** | **6-30** (held on hit or tapped); **16** (held without hitting) | 18 | 0 | 3 | 361 (3) | 40 | 30 | 1.8 → 2 | top (7.0, 5.5) [D] | 1.5 | |
| | | | | 1 | 3 | 361 (3) | 35 | 28 | 1.8 → 2 | top (7.0, 8.5) [D] | 1.5 | |
| | | | | 2 | 3 | **180** (3) | 10 | 20 | 2.2 → 3 | top (7.0, 12.0) [D] | 1.5 | **fighters only**: pulls the target in (the far box) |
| | | | | 3 | 3 | 361 (3) | 10 | 20 | 2.2 → 3 | top (7.0, 12.0) [D] | 1.5 | the same box for objects (id 2 wins on fighters); not modelled |
| Jab 2 | **3-4** | | 20 | 0 / 1 | 4 | 361 (3) | 56 | 70 | 3.0 → 4 | top (7.0, 5.0 / 8.5) [D] | 2.0 | |
| | | | | 2 | 4 | 361 (3) | 56 | 70 | 4.0 → 5 | top (7.5, 13.3) [D] | 2.0 | |

The continuable window is jab 1's frames 6-30: attack **pressed** in it, or **held** after jab 1 hit, carries on into jab 2 from frame
6 (UFD "transitions to jab 2 as early as frame 6") [R]; attack held **without** a hit carries on from frame **16** [R; meaning per the
table's two rows]. Jab 2 from jab 1's frame 6 puts its 3-4 on 8-9 and its FAF 20 on 25: the quickest loop exactly [D]. No rapid jab, no
third hit [R]. The far box (ids 2 / 3) is meant for jab locks (dragdown notes) [R].

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt (each aim) | **5-7** | **30** | 0 | 8 | **88** (3) | 60 | 75 | 4.0 → 5 | tail1 [E] | can be aimed up / straight / down: the **same numbers** for every aim, only the tail's position changes [R] |
| | | | 1 | 8 | 85 (3) | 60 | 75 | 3.5 → 4 | tail1 (x 4.3, y -1.0) [E] | |
| | | | 2 | 8 | 85 (3) | 60 | 75 | 3.0 → 4 | tail2 (x 3.1 up / 4.5 straight / 3.0 down) [E] | the tail tip |
| Up tilt | **8-15** | 32 | 0 | 7 | 100 (0) | **72** | 45 | 4.0 → 5 | tail2 (x 4.5) [E] | sweeps from front to back; the front box is high (it misses most grounded targets in front) [R] |
| | | | 1 / 2 | 7 | 100 (0) | 72 | 45 | 4.5 / 2.5 → 6 / 3 | tail2 (x 1.2), tail1 [E] | |
| Down tilt | **8-10** (conflict 1) | 24 | 0 | **5** | 28 (0) | 67 | 30 | 3.5 → 4 | tail1 (x 1.0) [E] | **trip 30%** |
| | | | 1 | **4.5** | 28 (0) | 67 | 30 | 3.5 → 4 | tail2 (x 3.0) [E] | trip 30% |
| | | | 2 | **4** | 361 (0) | 67 | 30 | 3.0 → 4 | tail3 (x 3.5) [E] | trip 40% (the tip) |
| Dash attack, clean | **10-12** | **41** | 0 / 1 | **11** | 65 (3) | 90 | 45 | 5.0 / 3.5 → 6 / 4 | legl (x 6.1 / 2.0) [E] | a flying side kick |
| Dash attack, late | **13-20** | | 0 / 1 | **8** | 65 (3) | 90 | **44** | 4.0 / 2.5 → 5 / 3 | legl (x 6.1 / 2.0) [E] | |

The forward tilt's three aims have identical hit data (the reference's three tables differ only in tail offsets) [R], so its aims are
positions only. The down tilt's trips: ssbwiki.com's trivia notes that only the tip's 361 angle can trip without a slope, and with the
move's high base knockback only against a target with reduced knockback; in practice it does not trip [R]. dragdown lists the down tilt
as "low profile 7-10" [R] (not modelled: a crouch-height pose here [E]). No arm or body intangibility on the tilts or the dash attack [R].

### Smashes [R]

| Move | Frames | Charge hold | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash (each aim) | **14-16** | 7 (charges 7-8) | **53** | 0 | **15.5** | 361 (3) | 30 | **102** | 4.5 → 6 | top (11.0 up / 6.5 straight / 4.2 down, 8.5 / 9.5 / 8.5) [D] | the head: the sweetspot, wins over id 1 |
| | | | | 1 | **14** | 361 (3) | 30 | **97** | 4.8 → 6 | top (11.5 / 6.5 / 4.2, 12.4 / 14.4 / 13.0) [D] | the nose: the sourspot |
| Up smash, clean | **11-13** | 7 (charges 7-8) | 47 | 0 / 1 | **14** | 75 (0) | 38 | 99 | 5.0 / 3.5 → 6 / 4 | footl (y -2.5, z -2.0), legl (x 1.0, z -2.0) [E] | a bicycle kick; hits behind |
| Up smash, late | 14-16 | | | 0 / 1 | **12** | 75 (0) | 38 | 99 | as clean | as clean [E] | |
| Down smash, hit 1 (front) | **7-8** | 3 (charges 3-4) | **50** | 0 | **12** | 26 (3) | 51 | 75 | 3.5 → 4 | top (3.0, 7.0 to 10.0) [D] | sdi 1.0 |
| | | | | 1 | **10** | 26 (3) | 46 | 63 | 2.0 → 3 | top (2.0, 4.0 to 14.0) [D] | |
| Down smash, hit 2 (back) | **22-23** | | | 0 | 12 | 26 (**4**) | 51 | **83** | 3.5 → 4 | top (3.0, -7.0 to -10.0) [D] | |
| | | | | 1 | 10 | 26 (4) | **44** | **75** | 2.0 → 3 | top (2.0, -3.0 to -13.0) [D] | |

Intangibility: forward smash **head and nose 12-16** [R] (E19 high); up smash **the kicking foot 11-13** [R] (E19 limb). Charged smash
damage is the shared rule (up to 1.5x over 60 frames in the reference: dragdown's charged 23.25 / 21 forward, 21 / 18 up, 18 / 15 down
are x1.5) [R]; the engine's shared charge bonus is what every port uses (conflict 9). The down smash's two hits are separate swings (front,
then back); a target in front is hit by hit 1 only [D]. The forward smash is aimed as the forward tilt; the down smash's back hit sends
backward (type 4) [R].

### Aerials [R]

| Move | Frames | Autocancel | Landing lag | FAF | Phase / id | Dmg | Angle (type) | BK | KS | Radius | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Neutral air, clean | **3-4** | 1-2, 38+ | **7** | 45 | 0 | **10** | 361 (0) | 40 | 86 | 4.8 → 6 | top (4.3 to 3.0, 7.5 to -1.0) [D]: a capsule along the leg |
| Neutral air, mid | 5-11 | | | | 0 | **7** | 361 (0) | 30 | 100 | 3.6 → 5 | top (4.5 to 3.7, 5.0 to -1.5) [D] |
| Neutral air, late | 12-25 | | | | 0 | **5** | 361 (0) | 30 | 100 | 3.0 → 4 | top (4.8 to 4.0, 3.5 to -1.5) [D] |
| **Forward air** | **16-20** | 1-3, 40+ | **12** | 44 | 0 | **15** | 361 (3) | **0** | 90 | 4.2 → 5 | head (x 1.5) [E]: the head |
| | | | | | 1 | **14** | **270** (3) | 0 | 90 | 4.0 → 5 | head (x 7.0, y 1.5) [E]: **the meteor** (the tip of the snout); hits anyone |
| **Back air**, hits 1-2 | **11, 14** | 1-5, 44+ | **11** | 56 | 0 | **3.5** each | **367** (**4**) | 20 | 100 | 5.5 → 7 | top (5.0 to 3.0, -10.0) [D]; **set weight**; H **0.5**; SDI **0.5** |
| Back air, hit 3 | **18-19** | | | | 0 | **5.5** | **54** (4) | 30 | **184** | 8.0 → 10 | top (5.0, -11.0 to -9.0) [D]; H **1.5** |
| **Up air** | **5-6** | 1-4, 31+ | **8** | 37 | 0 / 1 | **12** | 90 (0) | 25 | 100 | 7.5 → 9 | tail2 (x 0 / 6.0) [E]: hits behind too, even grounded targets |
| **Down air**, hits 1-6 | **16, 18, 20, 22, 24, 26** | 1-13, 50+ | **17** | 57 | 0 | **2.3** | **367** (0) | 5 | **80** | 5.0 → 6 | top (1.0, 2.0) [D]: the middle; H **0.6**, SDI **1.8** |
| | | | | | 1 / 2 | **1.8** | 367 (0) | 5 | 80 | 3.5 → 4 | top (2.0, 6.5 / -2.5) [D]: the sides |
| Down air, hits 7-12 | **28, 30, 32, 34, 36, 38** | | | | 0 | **1.9** | 367 (0) | 5 | **100** | 5.0 → 6 | top (0.5, 2.0) [D]; H 0.6, SDI 1.8 |
| | | | | | 1 / 2 | **1.4** | 367 (0) | 5 | 100 | 3.5 → 4 | top (1.5, 6.5 / -2.5) [D] |
| Down air, hit 13 | **41** | | | | 0 | **2.8** | **80** (0) | **55** | **179** | 6.5 → 8 | top (0.5, 2.0) [D]; H **1.5** |
| | | | | | 1 / 2 | 2.8 | **40** (0) | 55 | 179 | 4.5 → 6 | top (1.5, 6.5 / -2.5) [D] |
| Down air, landing hit | landing frame **1** | | landing FAF **18** | | 0 | **1** | 361 (0) | 65 | 100 | 5.0 → 6 | top (2.5, -3.0 to 6.5) [D] |

Landing lag = `landing_attack_air_frame` (dragdown) = ssbwiki.com's landing "interruptible" - 1 = UFD's landing lag [D]. The down air's
landing hit comes out on its landing animation's frame 1 when the move lands outside its autocancel windows (the Needles' down air
pattern: `landing_move`) [R]. **The down air's total**: 6 x 2.3 + 6 x 1.9 + 2.8 = **28%** when every middle box connects [D] (ssbwiki.com:
"28% in total", "the most damaging aerial in the game") [R]; each kick is a separate one-frame hit two frames apart [R]. The back air's
first two hits are set-weight autolinks (they combo into each other and into hit 3); every box sends backward (type 4) [R]. The forward
air "enables ledge grabs from both sides starting on frame **68**" (2.0.0; no aerial ledge lockout in the engine: the house limit already
listed for the Greatsword) [R]. The neutral air's three phases are one hit [D].

### Grabs, pummel and throws [R]

The grab is a **tongue**: long reach, slow, with a box near the mouth on its first frames only [R]. It is an ordinary grab (it catches
grounded, standing targets as anyone's; no tether: `air_lasso_type` False) [R].

| Move | Frames | FAF | Data |
| --- | --- | --- | --- |
| Standing grab | **14-21** | 49 | early (14-15): id 0 r 3.5 → 4 on `mouth2` (x -2.0) **aerial targets only**, id 1 r 2.8 → 4 on `mouth2` (x 1.2) **grounded only**, id 2 r 5.5 → 7 top (6.0, 7.5) [D] (the mouth box, "only active for the first 2 frames"); late (16-21): ids 0 / 1 only. `mouth2` is the **tongue tip**: its reach is not in the table (gap 3) |
| Dash grab | **16-23** | 57 | ids 0 / 1 r 3.3 / 2.2 → 4 / 3, id 2 r 5.5 → 7 top (6.0, 9.0) [D] early |
| Pivot grab | **17-24** | 52 | as the standing grab; id 2 top (6.0, -8.0) (written in front: the engine turns a pivot grab round [D]) |
| Pummel | **1** | **7** | **1.3%**, 361 (3), BK 0, KS 100, **FKV 30**, r 5.7 → 7, head (x 1.6, z 3.2) [E], **H 2.1**, no clank. The reference's hit on frame 1 becomes frame 2 here, FAF 8 (the house shift) [D]; UFD "19 including 14 frames of hitlag" agrees |
| Forward throw | release **15** | **40** | **9%**, **45**, BK 55, KS 66. Invincible 1-15 |
| Back throw | turns round **20**, release **20** | **44** | **9%**, 45, BK 55, KS 66 (the same numbers as the forward throw). Invincible 1-20 |
| Up throw | release **13** | **44** | 5%, **90**, BK 65, KS **90**. Invincible 1-13 |
| Down throw | release **18** | **44** without the hitlag | 4%, **80**, BK **100**, KS 25. "During frame 17, the move inflicts **8 frames of hitlag** to both Yoshi and the opponent, during which the opponent is invincible"; the table is the timing without the hitlag. Invincible 1-18 |

No throw has a pre-hit [R]. Each throw's "interrupted" row (`kind=1`: 3%, 361, BK 40, KS 100) is not modelled (as for the other
ports); the thrower's invincibility rows (2.0.0: invincible during the grab release and throw animations) are not modelled (house
limit). The down throw's hitlag folds into the timeline as the Staff's and the Greatsword's down throws do: release on **26**, FAF **52**
[D] (UFD gives "25" for its startup: conflict 7).

### Ledge and floor [R]

| Move | Frames | Intangible | FAF | Data |
| --- | --- | --- | --- | --- |
| Ledge attack | **20-22** | 1-22 (1st grab; 1-18 2nd, 1-11 3rd, none from the 4th) | 56 | 9%, 45 (3), BK 90, KS 20, shield +1 (111 [D]), r 5.0 → 6, top (5.0, 12.5 to 1.0) [D], no clank |
| Get-up attack, face up (ssbwiki.com "back") | front **14-15**, behind **20-21** | 1-21 | 46 | 7%, 48 (0), BK 80, KS 48, shield +8 (214 [D]), r 5.0 → 6; top (5.0, 14.0 to 3.0) then (5.0, -15.0 to -4.0) [D]; no clank |
| Get-up attack, face down (ssbwiki.com "front") | front **14-15**, behind **24-25** | 1-25 | 46 | as face up; top (5.0, 11.5 to 3.0) then (5.0, -18.0 to -5.0) [D] |
| Get-up attack, trip | front **19-20**, behind **26-27** | 1-7 | 50 | 5%, 361, BK 60, KS 50, shield +8 (260 [D]), r 4.5 → 6; top (4.5, 12.0 to 3.5) then (4.5, -15.0 to -4.5) [D]; no clank |

One get-up attack slot exists (as for the other ports): the face-up one would be used.

## Specials

### Egg Lay (neutral special) [R]

The tongue again, as a **command grab** that also catches in the air: the catch is swallowed and laid as an **egg** that traps the
victim until it mashes out or the timer runs down.

**The tongue** (both versions):

| Part | Data |
| --- | --- |
| Boxes | early (frame **19**): id 0 r 4.0 → 5 on `mouth2` (x -0.9) **aerial only**, id 1 r 3.5 → 4 on `mouth2` (x -2.0) **grounded only**, id 2 r 5.9 → 7 top (7.0, 9.0) [D] (the mouth box, "active for only one frame"); late (**20-22**): ids 0 / 1 [R] |
| Whiff | FAF **42** (UFD total 41 on a whiff); animation 61 grounded, 51 aerial (ssbwiki.com table; the changelist says the aerial one became 61: conflict 6) [R] |
| Catch | the closest opponent in reach, **on the ground or in the air** ("it acts as an aerial grab"); a grab, so it is not stopped by a shield; it catches a victim in hitstun (the 3.1.0 fix is about such a catch) [R reading]; the general command-grab rules (not on a ledge, not intangible) [D] |
| Success | the victim is pulled in and swallowed, then the egg is laid: the laying part's **release on its frame 21**, **Yoshi intangible 7-29** of it, FAF **38** of it (animation 52) [R]; UFD: "total frames when successful is 62" (FAF 63), "egg appears on frame 33 when successful" (conflict 5: the two readings of the timeline) |
| Armor | "Yoshi has a few armor frames in this move from the second his tongue latches onto another character to the moment the egg is laid" (the move article, all games) [R text]; in 13.0.1 the table gives the laying part's **intangibility 7-29** [R]; the frames between the catch and the laying part are not published (gap 4) |
| Laid in the air | Yoshi's vertical speed becomes **1** when he lays the egg (0.125 world [D]) [R]; he cannot grab a ledge during the aerial version if it is not interrupted (changelist) [R] |
| The laying throw | **7%**, 361, BK 0, KS 100, H 0 (it launches nobody: the victim becomes the egg) [R]; its interrupted row 4%, 361, BK 0, KS 100 (Yoshi hit during the swallow) not modelled [R] |

**The egg** (ssbwiki.com neutral special page, "Parameters"; the move article) [R]:

| Property | Value |
| --- | --- |
| Duration | **`100 + 1.8 x p`** frames, **p** the victim's percent, **capped at 100%** [R]. The laying throw's own damage then cuts it like any hit (below): without mashing an egg lasts **86 to 266 frames** (84 to 264 with the one-on-one multiplier) [R]. 86 = 100 - 2 x 7, so **p is the percent before the laying throw's 7%** [D] |
| Mashing | **7 frames per stick input**, **12.6 per button input** ("though they cannot be registered as quickly") [R] |
| Hits | the egg is **open to attacks**; the victim **cannot be knocked back** while in it, takes **0.66x** damage, and each hit cuts the timer by **2 frames per percent** (conflict 2: of the damage before or after the 0.66x) [R] |
| Laid | the egg leaves at **0.7 horizontal, 2 vertical** (0.0875, 0.25 world [D]); which way horizontally is not published (gap 5) [R] |
| Size | starts at **0.45x** its full size and grows to full in **20 frames** [R]; the full egg's size is not published (gap 6) |
| Control inside | on the ground the victim can **shift left and right by mashing that way** (horizontal speed **0.3** a press) and **hop slightly by mashing diagonally up** (vertical speed **1**); in the air **no real control** (0.01 horizontal, 0.1 vertical) [R] (0.0375, 0.125, 0.00125, 0.0125 world [D]); "the opponent has no control in the air, giving it minor utility near the ledge at high percents" [R] |
| Physics | the egg falls and lands; its gravity, fall speed and friction are not published (gap 7) |
| Escape | the egg breaks: the victim leaves it with vertical speed **2.5** (0.3125 world [D]) and **7 frames of intangibility** [R]; what it can do on breaking out (act at once, or a fall) is not published (gap 8) |
| Who can be egged | nothing in the 13.0.1 sources excludes a fighter; the general command-grab rules apply [R reading] (gap 9: an egged fighter grabbed or egged again) |

### Egg Roll (side special) [R]

A small hop into an egg, then a roll along the ground that the stick steers; it hits by speed; one jump; it passes through shields; it
stalls at a ledge; special breaks it.

| Part | Data |
| --- | --- |
| Start | a hop: **vertical speed 2** on startup (0.25 world [D]), roll gravity **0.13**, fall speed **1.95** (0.01625, 0.24375 world [D]); "Yoshi always bounces once when using Egg Roll on the ground" (changelist) [R]; the roll itself is on from frame **21** [R] |
| Hitbox | **only once the egg touches the ground** (frame **32** on flat ground), **none while turning**, none in the air (an aerial Egg Roll has no hitbox until it lands) [R]; ids 0 (r 2.3 → 3, **aerial only**) and 1 (r 3.5 → 4, **grounded only**), top (5.6, 0) [D]; **10% + the roll speed** ("10 + roll speed"; 10 to about 13.8%), **80** (0), BK 70, KS 50, H 1.0, **rehit 32** [R]; UFD "10.8—12.8", dragdown 10 (the base) [R] |
| Speed | landing speed **0.9**; base roll speed **1.4 to 3.2** grounded (0.7 to 2.8 on ground-to-air transitions); grounded speed x**1.2** by the stick; aerial speed **0.4** by the stick; acceleration **0.05**; friction **0.07**; "the maximum reachable roll speed is of roughly 4"; "highly volatile and affected by various factors" [R] (world /8 [D]); the exact steering law is not published (gap 10) |
| On a hit | the roll speed drops by **1**; the roll's remaining time drops by **40 frames** per target hit [R] |
| Duration | **25 to 180 frames** of roll (from frame 21): special **cancels it from 25 frames in**; at 180 it ends by itself [R] |
| Ending | from the cancel (or the end): FAF **40** frames later (animation 44); the minimum whole move is FAF **85** [R]; on cancelling, the speed x**0.7** horizontal, x**0.5** vertical, ground friction **0.05** [R] |
| Jump | **one jump** per use, at **30 degrees** at the lowest [R]; its speed is not published (gap 11) |
| Bounces | off a wall: horizontal x**0.4**, vertical **1.7**; off the floor: horizontal x**1.3**, vertical **0.6x** its falling speed, at most **1** [R] (world /8 [D]) |
| Shields | "can move through shields"; -29 to -27 on shield (dragdown); no shield damage multiplier [R] |
| Ledge | "can stall at the ledge if the control stick is not held towards it, but does no damage when stationary" [R]; held toward it, it rolls off |
| Air | "not being able to break out until Yoshi lands" (special does not cancel in the air) [R reading]; "failing to grab the ledge" means it can catch a ledge in the air [R reading] (the frames: gap 12) |
| Helpless | **no** (since the previous game) [R] |
| Armor | none ("lack of armor or disjoints") [R] |

### Egg Throw (up special) [R]

An aimed, lobbed egg that bounces and bursts; in the air the first three throws each give a hop.

| Part | Data |
| --- | --- |
| Timing | the egg leaves on frame **16**; FAF **56** (animation 57) [R]; -30 on shield (UFD) |
| Egg | **6%**, **70** (3), BK 60, KS 50, r 2.5 → 3 (a capsule y -1 to 0) [D], **H 0.6**, shield **-3** (50 [D]), **reflectable**, cannot rebound, `direct=f` [R] |
| Burst | the egg **bursts on hitting an opponent or a surface**, or by itself after its **46-frame** life (move frames 62-64: a 3-frame burst; UFD "eggs explode automatically on frame 60": conflict 8); the burst: **6%**, 70 (3), BK 60, KS 50, r **6.5** → 8, H 0.6, shield -3 [R]. **One hit**: "eggs deal a single hit to opponents instead of two" (changelist), so the burst does not hit the egg's own victim again [R reading] |
| Speed | **`1.5 + 0.082 x f`** reference units a frame, **f** the frames special is **held** during the startup (at most **15**): 1.5 to 2.73 (0.1875 to 0.341 world [D]) [R] |
| Aim | the stick during the throwing animation: default angle **58 degrees**, range **90 to 18** (straight up to low forward), stick sensitivity **0.5** [R] |
| Flight | spawned at forward **5**, up **13** [R]; vertical speed multiplier **1.25** (on its launch's vertical part [D reading]); gravity **0.09**; fall speed **10** (0.01125, 1.25 world [D]) |
| Bounce | it bounces on the ground, **at most twice**, "retaining [its] horizontal momentum"; "maximum vertical speed to bounce off ground **1.6**"; speed x**0.8** after a bounce [R]; how 1.6 applies (a cap on the bounce, or the limit above which it bursts) is not stated (conflict 10) |
| Yoshi's move physics | gravity **0.07**, falling speed **1.29**, air speed x**0.9**, vertical speed x**0.8** on starting [R] (0.00875, 0.16125 world [D]) |
| **The hop** (air) | the vertical speed becomes **`min(2.5 x 0.8^n + 0.8 y, 2.6)`**, **y** Yoshi's vertical speed when the move starts and **n** the throws already made without landing; **at most 3 hops per air time** (n = 0, 1, 2); from a standstill 1.468 / 0.968 / 0.568 [R] (0.3125, the cap 0.325 world [D]); the frame the hop happens is not published (gap 13) |
| Ledge | "Yoshi can grab ledges during any portion of the move **as long as he is not moving upward**" [R] |
| Helpless | **never** [R] |
| Ground | no hop [R reading]; B-reverse and turnaround as the shared special rules [E] |

### Yoshi Bomb (down special) [R]

A ground pound: on the ground a hop (with a weak hit) into a fast drop; in the air a stall then the drop; stars burst out on landing.

| Part | id | Frames | Dmg | Angle (type) | BK / FKV | KS | Radius | Bone (y, z) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Grounded, the hop | 0 | **7** | **4** | 80 (3) | FKV **80** | 100 | 7.0 → 9 | top (7.0, 10.0) [D] | **grounded only**; H 0.5 |
| Grounded, the drop | 0 | **27** onward (until landing) | **15** | 80 (3) | 60 | 72 | 6.8 → 9 | top (5.5, 0) [D] | H 0.7; shield +10 (167 [D]) |
| Aerial, clean | 0 | **19** | **12** | 80 (3) | 70 | 68 | 6.4 → 8 | top (0, 0) [D] | H 0.7; shield +10 (183 [D]) |
| Aerial, late | 0 | **20** onward (until landing) | 12 | 80 (3) | 70 | 68 | 6.4 → 8 | top (5.5, 0) [D] | the same, the box higher |
| Landing stars | 0 (one each side) | landing **3-12** | **4** | 50 (3) | 50 | 40 | 4.0 → 5 | projectiles from (1, ±8) [R] | **H 0** (no hitlag), shield +6 (250 [D]), cannot rebound, `direct=f`, **reflectable, absorbable** |

| Rules | Data |
| --- | --- |
| Drop | **falling speed 4.8** (0.6 world [D]) [R]; "the falling portion lasts indefinitely until Yoshi lands" [R] |
| Grounded hop | the rise is the aerial version's x**1.4** ("grounded height multiplier") [R]; the rise's height in units is not published (gap 14) |
| Aerial stall | the aerial version is a "stall-then-fall" move; its physics on frames 1-18 are not published (gap 15) |
| Platforms | "always [drops through soft platforms] for a short distance to start the move" (**a 7-frame drop-through period during the fall**), and passes every soft platform **if down is held** [R] |
| Landing | the landing lasts **39** frames, FAF **40** [R] (UFD: landing 39, total 77 grounded on level ground) |
| Stars | two stars, at forward ±8, up 1; initial speed **0.6 horizontal, 2 vertical**, gravity **0.2**, falling speed **2** [R] (0.075, 0.25, 0.025, 0.25 world [D]); out on the landing's frames 3-12 (UFD "stars appear on frame 4 of landing": conflict 11) |
| Ledges | "Yoshi can grab ledges as soon as he starts to fall", **in front and behind** ("can sweetspot a ledge even if Yoshi is facing away from it", changelist) [R] |
| Shield | "the grounded version can break a full shield if it is not stale and all hits connect on shield" [R] |
| Helpless | no (the landing ends the move) [R] |

## Conflicts and how they are resolved

1. **Down tilt's active frames**: ssbwiki.com and UFD **8-10**, dragdown **7-9** (with "low profile 7-10"). Resolved to **8-10** (two
   sources; dragdown's table is a frame early on this move only).
2. **The egg's timer cut by hits**: the move page says the cut is twice the damage dealt "taking the egg's 0.66x damage multiplier into
   account" (twice the damage actually taken); the move article says "2 frames per damage dealt (prior to the damage reduction)".
   **Unresolved**; proposed **twice the damage actually taken** [E] (the move page is the parameter-based one); footage settles it (a
   10% hit on an egg with a frame count).
3. **A breaking hit on the double-jump armor**: ssbwiki.com "the minimum knockback an attack that breaks the armor can do is 60 KB";
   dragdown "a minimum of 60 frames of hitstun" (marked for research by dragdown itself). Resolved to **60 units of knockback** [R]
   (60 frames of hitstun would need 150 units); the launch is `max(k - 120, 60)` [D reading].
4. **Several hits on the double-jump armor**: dragdown "absorbs up to 120 units ... with each hit reducing this threshold" (a pool);
   ssbwiki.com compares each hit's knockback with 120; an earlier-game player guide: "knockback NEVER adds up". **Unresolved**;
   proposed **each hit on its own** [E: two of three statements, one of them from the earlier game]; footage settles it (a multi-hit
   move into a double jump).
5. **Egg Lay's success timeline**: the laying part's FAF 38 against UFD's total of 62 puts the laying part's start on move frame 25 and
   its release on 45 [D]; UFD's "egg appears on frame 33" does not fit that. Proposed: the catch on 19-22, the swallow until the laying
   part starts on **25** [E], the release on its 21, FAF 63 overall [R: UFD]; footage settles it.
6. **Egg Lay's animation lengths**: ssbwiki.com's table 61 grounded, 51 aerial; the changelist "the aerial version's animation is
   longer (43 frames → 61)". The FAF (42) is the same in every source; resolved: FAF 42 for both [R], the animation lengths only matter
   to the view.
7. **The down throw's release**: ssbwiki.com release 18 without its 8 frames of hitlag (26 with it, FAF 52 [D]); UFD's startup **25**.
   Resolved to the folded **26** [D] (the house rule for a throw's mid-throw hitlag); one frame of difference, footage settles it.
8. **The egg's life**: ssbwiki.com 46 frames (burst on move frames 62-64); UFD "explode automatically on frame 60". Resolved to **46
   frames** [R: the parameter], burst on 62 [D].
9. **Charged smash bonus**: dragdown's charged values are x1.5; the engine's shared smash charge (40% over 60 frames, every port) is
   kept, as for the other ports (a shared rule, not this port's).
10. **The egg's ground bounce**: "maximum vertical speed to bounce off ground 1.6" and "speed multiplier after bouncing 0.8" do not say
    whether 1.6 caps the bounce or is the fastest fall that still bounces (a faster one bursting). **Unresolved**; proposed: it bounces
    when falling at most 1.6 and bursts otherwise, its speed x0.8 both ways on a bounce [E]; built that way in phase 2 (sim 95; owner
    decision 60: the estimate stands, no footage needed). Earlier note: footage would settle it (an egg thrown high
    and one thrown low along the ground).
11. **The landing stars' first frame**: ssbwiki.com 3-12, UFD "frame 4 of landing". Resolved to **3** [R: the hitbox table] (UFD may
    count the frame they become visible).
12. **The roster note's "egg shield" and "air-jump curve" (`ROSTER_STATUS.md`)**: in 13.0.1 the shield is decided (owner decision 5),
    and the double jump is **not** a delayed jump with its own curve (it was, in the first three games): it is an ordinary jump to
    51.56 [R]. Corrected (see `PROJECT_AUDIT.md`: E12 shrinks to a turnaround flag).

## Gaps (needed for 1:1, not published)

1. The double jump's air time (to check the plain-jump reading against the reference).
2. UFD's spot dodge "21/26": what 21 is.
3. The tongue's reach: where the `mouth2` bone is on the grab's and Egg Lay's active frames (the grab's range in units).
4. Egg Lay: the frames between the catch and the laying part, and whether Yoshi is armored or intangible there.
5. The egg's horizontal direction when laid (in front or behind Yoshi).
6. The full egg's size (its hurtbox) and where it sits on the victim.
7. The egg's gravity, fall speed, ground friction and how it lands (does it bounce).
8. What the victim can do on breaking out (act at once, or a no-action window), and whether the 2.5 jump spends its midair jump.
9. Whether an egged fighter can be grabbed, egged again, or knocked out of the egg by a strong hit.
10. Egg Roll's steering law (how the stick sets the speed between 1.4 and 3.2, the turn and its length).
11. Egg Roll's jump speed (only its lowest angle, 30 degrees, is published).
12. Egg Roll's ledge grab in the air (from which frames, in front only or both sides).
13. Egg Throw's hop frame, and whether a ledge grab (not only landing) gives the three hops back.
14. Yoshi Bomb's grounded rise height (the hop) and the drop's start height.
15. Yoshi Bomb's aerial stall (frames 1-18): its vertical speed and drift.
16. The 7.0.0 shield intangibility frames (only needed if decision 5 is ever revisited).

Footage that would settle most of them (training mode, frame-stepped, hitbox view): a double jump from the ground with a frame counter;
a jab and a multi-hit move into a double jump at 0% and at 100% (conflicts 3, 4); the grab and Egg Lay at their furthest reach; an Egg
Lay at 0% and 100% with and without mashing, the egg hit by a 10% move, and the break-out (conflicts 2, 5, gaps 4-9); an Egg Roll on flat
ground turned at full speed, jumped, run into a wall, held to a ledge and rolled off (gaps 10-12); an aerial Egg Throw three times from a
full hop and an egg thrown high and low (gap 13, conflict 10); a grounded and an aerial Yoshi Bomb from a platform (gaps 14, 15).
