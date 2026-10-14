# Reference data for the tenth moveset (the reference game's Link, adult)

Mechanical reference for the tenth port (the adult Link; not Young Link or Toon Link), collected 2026-10-13 for version **13.0.1**,
before anything is built. Proposed in-game names (not registered; see `PROJECT_AUDIT.md`): the **Buckler** class and weapon on the
**wanderer** body, built-in fighter **Wanderer**. The reference character's name is not used in the game. What the engine can and cannot
do with this data is in `PROJECT_AUDIT.md`; the format follows `docs/movesets/yoshi/REFERENCE_DATA.md`.

## Sources and version

- **ssbwiki.com** per-move pages `Link_(SSBU)/<move>` (31 pages: `Neutral_attack` and its `Hit_1` / `Hit_2` / `Hit_3`, the three tilts,
  `Dash_attack`, `Forward_smash` and its `Hit_1` / `Hit_2`, `Up_smash`, `Down_smash`, the five aerials, `Grab`, `Pummel`, the four
  throws, `Edge_attack`, `Floor_attack_(back|front|trip)`, `Neutral_special`, `Side_special`, `Up_special`, `Down_special`), read as
  **raw wikitext** (`index.php?title=...&action=raw`), so every hitbox value comes from a named template field (`damage`, `angle`, `af`
  angle type, `bk`, `ks`, `fkv`, `r`, `bn`, `xpos/ypos/zpos`, `ff` hitlag multiplier, `sdi`, `sd` shield damage, `trip`, `setweight`,
  `clang`, `rebound`, `direct`, `hitbits`, `reflectable`, `g=f` / `a=f`) and the angle-361 column shift cannot happen. The four specials'
  pages also give the move **parameters** (arrow, boomerang, bomb and spin physics). https://www.ssbwiki.com/Link_(SSBU)
- **ssbwiki.com articles**: `Link_(SSBU)` (attributes template, moveset notes, update history), `Passive_shield` (the Hylian Shield),
  `Bow_and_Arrows`, `Boomerang`, `Remote_Bomb`, `Spin_Attack`, `Item_throw` (throw strengths, frames and the item damage formula),
  `Autolink_angle` (position vectors), `Dash` (current-game table) and `Template:UpdateList_(SSBU)/<version>` for every version that
  touched this character (1.1.0, 1.2.0, 2.0.0, 3.0.0, 4.0.0, 7.0.0, 9.0.0, 9.0.1, 11.0.0, 13.0.1).
- **ultimateframedata.com/link** (UFD: startup, total frames, landing lag, notes, active frames, dodges, jump air times, stats). UFD's
  "total frames" is FAF - 1. https://ultimateframedata.com/link
- **dragdown.wiki** `SSBU/Link/Data` (the parameter file and a frame-data table; "total" = FAF - 1), read as raw wikitext.
  https://dragdown.wiki/wiki/SSBU/Link/Data
- **kuroganehammer.com**: the Link page (https://kuroganehammer.com/Ultimate/Link) returned **HTTP 404** on 2026-10-13, as for the
  Yoshi port, so it could not be used. Every startup and FAF below is cross-checked between ssbwiki.com, UFD and dragdown instead.

**Naming note**: the reference's 13.0.1 neutral special is "Bow and Arrows" (a travel bow whose arrows lodge and can be picked up), not
the earlier game's "Hero's Bow"; UFD and dragdown still label it "Hero's Bow". The data below is the 13.0.1 move.

**Update history applying to 13.0.1** (ssbwiki.com update lists, every entry for this character): **1.1.0** crouching while holding an
item adjusted (presentation). **1.2.0** and **4.0.0** "Invisible Bomb" glitch fixes (no gameplay value). **2.0.0** up air autocancels
earlier (47 → **43**); arrows travel longer (lifetime 40 → **46** frames). **3.0.0** Boomerang's shield damage 0 → **-4.5 / -3 / -1.5**
(the move page's raw table says **-4** for the clean hit: conflict 6). **4.0.0** **shield SDI when a projectile hits the Hylian Shield**;
get-up timing made universal; **aerial Spin Attack's back hits no longer use set weight** (their position vector lasted too long when
interrupted); the spin's autolink knockback reduced. **7.0.0** shield size increased (global here). **9.0.0** up smash's first two hits:
grounded-only boxes 105 → **107** degrees and set knockback 39 → **28**; every up smash box gains a **+3 hitstun modifier**. **9.0.1**
falling through the stage after breaking another fighter's blocks fixed (no value). **11.0.0** crumple vulnerability made universal.
**13.0.1** jab 1 startup 8 → **7**, FAF 25 → **24**; down tilt angle 80 → **85**; up smash final hit KS 96 → **101**. Every table below
is the post-13.0.1 data. No data from the earlier games was used.

**Labels**: **[R]** published reference value; **[D]** derived (unit conversion, a formula, a frame shift, a reading of a field);
**[E]** estimate. Hitbox **positions** on a bone other than `top` (`sword2` / `sword1` the blade, `armr` the sword arm, `colonells` a
bone at the sword hand / hilt [E: reading], `kneer` / `kneel` the knees, `hip`) are [E]: the reference attaches them to its own
skeleton, so ours go on our body where that bone is in the move's pose. Offsets on `top` (the root at the feet: `zpos` forward, `ypos`
up) convert directly [D]. A reference capsule (`zpos=14.5 to 8.0`) is two or three circles along it [D].

## Conventions

- Frames are the reference's (frame 1 is the move's first, FAF as published: ssbwiki.com's "Interruptible" is the FAF); the sim stores
  them minus one (`ref_move` / `row`).
- Units: 1 world unit = 8 reference units. Radius in world tenths = reference value x 1.25, rounded half up [D] (the house rule). Speeds
  and gravities: reference units a frame / 8 = world units a frame [D].
- Damage in percent before the 1.2 one-on-one multiplier. Angles in degrees; **361** is the reference's 361 angle ("angle 361"), **367**
  the autolink angle. H = hitlag multiplier (`ff`), SDI = SDI multiplier.
- **Angle type** (`af`): unset = 0 (by position), 1 (by the direction of travel), 3 forward (the owner's facing), 4 backward.
- **Targets**: `a=f` cannot hit aerial targets (grounded only), `g=f` cannot hit grounded targets (aerial only) [R].
- **Shield damage**: the reference adds `sd` to the damage before the shield multiplier; ours is a percent of the damage, so `sd` becomes
  `100 x (damage + sd) / damage`, rounded [D].
- `clang=f rebound=f`: never clanks, never rebounds (E3c, `Hitbox::clanks`) [R]. `direct=f`: an indirect hit [R], not modelled.
  `reflectable`, `absorbable`: the projectile can be reflected / absorbed [R] (here every projectile is reflectable; see the audit).
- `trip=N`: an N% trip chance (E20) [R]. `setweight=t`: knockback at weight 100 (`Hitbox::set_weight`) [R]. FKV = set knockback.
- `hitbits=Fighter only`: the box hits fighters only; its twin with the same shape hits objects (items) [R]. Here only the fighter box
  is modelled until items exist (E4).

## Attributes [R]

ssbwiki.com attributes template, UFD stats and the parameter file (dragdown); walk speed is a conflict (1):

| Weight | Walk | Initial dash | Run | Traction | Air speed | Air accel (base + additional) | Air friction | Gravity | Fall / fast fall | Jump / short hop / double jump (heights) | Jump squat | Air jumps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **104** | **1.247** (conflict 1) | 1.98 | 1.534 | 0.113 | 0.924 | 0.01 + 0.04 | 0.0038 | 0.096 | 1.6 / 3.04 | **27.8 / 13.38 / 29.0** (among the lowest jumps) | 3 | 1 |

Parameter file (dragdown) [R]: `walk_accel_add` **0.105**, `walk_accel_mul` **0.21** (ours: `walk_accel` 0.105, `walk_accel_stick`
0.21); `walk_speed_max` 1.247; `run_accel_add` **0.044**, `run_accel_mul` **0.13167** (ssbwiki.com's Dash table agrees: 0.044 +
0.13167, max 0.17567); `ground_brake` **0.113**; `dash_speed` 1.98; `run_speed_max` 1.534; initial dash **12 frames**, pivot dash
**14** (ssbwiki.com Dash table); `jump_initial_y` 15.29; `jump_y` 27.8, `mini_jump_y` 13.38, `jump_aerial_y` 29.0; `jump_speed_x` 0.95,
`jump_speed_x_mul` 0.8, `jump_speed_x_max` 1.2, `jump_aerial_speed_x_mul` 1.0; `air_accel_x_mul` 0.04, `air_accel_x_add` 0.01;
`air_speed_x_stable` 0.924; `air_brake_x` 0.0038; `air_accel_y` 0.096; `air_speed_y_stable` 1.6; `air_brake_y` 0.004; `dive_speed_y`
3.04; `air_ground_speed_brake` 0.0114; `landing_frame_light` 1, `landing_frame` 4, **`landing_heavy_frame` 4** (so **3** frames of
heavy landing lag here [D], by the house rule `landing_heavy_frame - 1`; ssbwiki.com's "ellag 5" is `landing_frame` 4 + 1 [D
reading]); `landing_attack_air_frame` n / f / b / hi / lw **6 / 11 / 6 / 14 / 19** (agreeing with UFD and with ssbwiki.com's landing
"interruptible" 7 / 12 / 7 / 15 / 20); `cliff_jump_y` **33.9**, `cliff_jump_x_speed` 0.6; `jump_count_max` 2; `scale` **1.0**; `size`
`fighter_size_m`; `shield_radius` 10.9 (global here); `jostle_front` 1.8, `jostle_back` 1.0; **no wall jump** (`wall_jump_type`
False, `attach_wall_type` False); the wall-tech jump speeds `passive_wall_jump_x_speed` **1.3**, `passive_wall_jump_y_speed` **2.6**
(ours: `wall_jump` 0, `wall_jump_vx` 1.3, `wall_jump_vy` 2.6 [D], the wall-tech jump only); **no tether** (`air_lasso_type` False: the
grab is a plain hand grab) [R]; no crawl (`squat_walk_type` False). Item handling [R] (used only if E4 is built): `throw_item_speed_mul`
**1.01333** (= the item toss strength `1 + (weight - 100) / 300`, ssbwiki.com `Item_throw`), `throw_item_power_mul` 1.0,
`item_lift_*` (carrying heavy items), `shoot_walk_*` / `shoot_dash_speed_f` 1.584 / `_b` 1.2672 (moving while firing a shooting item),
`item_range_front` 13.5 (max 17.5), `item_range_back` 6.78571, `item_range_h` 12.75, `item_range_air_front` 10.125, `item_range_air_back`
6.78571, `item_range_air_h` 17.0 (the pick-up reach), `backshield_offset_y` **7.0** (where the shield sits on the back while an item is
held).

Jump air times (UFD) [R]: short hop **33**, full hop **45**, short hop fast fall **22**, full hop fast fall **30**.

Dodges (UFD total frames, so FAF one more) [R]: spot dodge **26** (FAF 27, intangible 3-17; UFD's "21/26" first number unexplained, as
for every port), forward roll **30** (FAF 31, 4-15), back roll **35** (FAF 36, 5-16). Neutral air dodge **49** (FAF 50, intangible
**3-30**, landing lag 10); directional down **69**, diagonal down 74, sideways **82**, diagonal up 95, up **108** (intangible **3-21**;
landing lag 11-19). The engine's three directional lengths would be down / side / up 69 / 82 / 108 (no separate diagonals) [D].

Out of shield (UFD) [R, for checks]: **up special 7 frames (the front hit only)**, back air 9, neutral air or up smash 10, shield grab 10.

Ledge (ssbwiki.com edge attack page): ledge attack intangible 1-22 on the first grab, 1-18 on the second, 1-11 on the third, none from
the fourth (the universal decay, already the engine's) [R].

Body size [E]: scale 1.0, `fighter_size_m`. The sword's reach: the jab's far box at 18-18.5 forward, the down tilt's tip at 17.5, the
grounded spin's tip `sword2` x 8 (the blade is about 8-9 reference units past `sword2`'s base) [R]; the body and blade lengths are
fitted to these [E].

## The Hylian Shield (passive projectile block) [R]

| Property | Value | Source / label |
| --- | --- | --- |
| States | **standing still, walking or crouching** | ssbwiki.com `Link_(SSBU)` moveset note and `Passive_shield` [R]. Other states (dash, run, turn, jump squat, landing, any attack, the air) do not block [R reading: the list is exhaustive in both articles] |
| What it blocks | any hitbox that is **reflectable** and not unblockable, whether or not it is on a projectile ("it does not actually matter whether the hitbox is attached to a projectile") | `Passive_shield` [R] |
| Front check | the projectile must **hit the shield itself** (the front of the body, where the shield is held); a hit on the back, head or feet goes through | `Passive_shield` [R]. Crouching moves the shield down to cover the legs, but then a shot at face height reaches the head [R] |
| Effect | **no damage**; the wielder is **pushed back**, "dependent on how much damage and knockback it would ordinarily do"; it **never breaks** and blocks any number of projectiles of any power; a defensive stance plays (presentation) | `Passive_shield` [R]. **The push's formula is not published** (gap 1) |
| The projectile | is stopped as by a shield [E: not stated; an arrow item "vanishes" when shielded and the bomb "bounces off", so each projectile's own shield behaviour applies] | [E] |
| Shield SDI | **since 4.0.0** the wielder can shield-SDI while in the hitlag of a blocked projectile | update list 4.0.0 [R]; so the block has hitlag [D reading] |
| Holding an item | **disables it** (the shield is moved to the back, `backshield_offset_y` 7.0) | `Passive_shield` [R] |
| Shape and height | not published (gap 2): it is the shield's model on the left arm, in front at roughly chest height [E] | [E] |

## The sword: tip, middle and hilt

The reference has **no single "tipper" rule** for this sword; each move sets its own boxes (`sword2` along the blade, `armr` and
`colonells` at the arm and hilt, sometimes a `top` box on the body). Where the boxes differ, it is listed per move [R]:

- **Tip stronger**: dash attack (tip 14 / blade 13 / arm 12), forward smash hit 1 (**tip 14 at angle 361, everything else 7 at 69 /
  60** so that the non-tip hit combos into hit 2), down smash front hit (tip 17 / blade and arm 16 / hilt 14), up smash final hit (tip
  11 at 80 / blade 11 at 90 / arm 10 / body 9), forward air hit 1 (outer and middle boxes 367 to aerial targets, the inner one 38),
  up throw's bystander box.
- **Tip weaker** (a sourspot at the very end): the grounded Spin Attack (tip boxes deal **0.8x**: 11.2 / 9.6 / 7.2 / 5.6), up tilt
  (tip 95 degrees KS 103 vs blade 85 KS 111).
- **Hilt or body weaker**: forward smash hit 2 (blade 13, arm and body 12), down smash back hit (blade 12, arm 11, hilt 10), neutral air
  (foot 11, leg 9).
- **No difference** (one value on every box): jab 1-3, forward tilt, down tilt, up air, down air, back air (kicks), the aerial Spin
  Attack.

The arm and hilt boxes (`armr`, `colonells`) carry `clang=f rebound=f` on several moves (forward and up tilt, dash attack, up smash
final hit) [R]: the body part does not clank, the blade does.

## Normals

All frame numbers [R] (ssbwiki.com timing tables; UFD and dragdown agree on every startup and FAF unless listed under "Conflicts").
Damage, angle, BK, KS, FKV, H, SDI [R]. Radius column: reference → world tenths [D]. Positions in reference units on the named bone
(top: (y up, z forward)).

### Jab [R]

Three slashes (10% total). Quickest loop: jab 1 on 7-8, jab 2 on 14-15, jab 3 on 22-23; FAF 24 / 34 / 51 (from the start of the chain).

| Part | Frames | Continue | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone (y, z) | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Jab 1 | **7-8** (13.0.1) | **10-35** (held on hit or tapped); **14** (held without hitting) | **24** (13.0.1) | 0 | 3 | 361 (3) | 25 | 25 | 1.8 → 2 | top (8.0, 10.0 → 7.0) | 1.6 | |
| | | | | 1 | 3 | 361 (3) | 25 | 25 | 1.8 → 2 | top (8.0, 14.0 → 7.0) | 1.6 | |
| | | | | 2 | 3 | **180** (3) | 20 | 15 | 1.8 → 2 | top (8.0, 18.0 → 7.0) | 1.6 | **fighters only**: the far box pulls the target in (jab lock) |
| | | | | 3 | 3 | 361 (3) | 20 | 15 | 1.8 → 2 | top (8.0, 18.0 → 7.0) | 1.6 | the same box for objects; not modelled |
| Jab 2 | **5-6** | **8-35** (held on hit or tapped); **13** (held without hitting) | **25** (conflict 2: UFD / dragdown 26) | 0 | 3 | 361 (3) | 25 | 25 | 2.8 → 4 | top (8.5, 9.5 → 8.5) | 1.2 | |
| | | | | 1 | 3 | 361 (3) | 25 | 25 | 3.5 → 4 | top (8.5, 14.5 → 8.5) | 1.2 | |
| | | | | 2 | 3 | **180** (3) | 20 | 15 | 3.5 → 4 | top (8.5, 18.5 → 8.5) | 1.2 | fighters only |
| | | | | 3 | 3 | 361 (3) | 20 | 15 | 3.5 → 4 | top (8.5, 18.5 → 8.5) | 1.2 | objects; not modelled |
| Jab 3 | **6-7** | | **35** | 0 / 1 | 4 | **30** (3) | 70 | 70 | 4.5 → 6 | top (8.0, 10.0 / 15.0) | 2.0 | the semi-spike |
| | | | | 2 | 4 | 30 (3) | 70 | 70 | 3.8 → 5 | sword1 (x 11.0, z -1.5) [E] | 2.0 | |

The continuable windows follow the house reading of the two rows (as the Yoshi and Embers ports): attack **pressed** in the window, or
**held** after a hit, carries on from the window's first frame; held **without** a hit from the second number [R; meaning per the
table's two rows]. UFD gives jab 1 → 2 "as early as frame 12" (conflict 3) and jab 2 → 3 "as early as 8" (agrees). No rapid jab [R].

### Tilts, dash attack [R]

| Move | Frames | FAF | id | Dmg | Angle (type) | BK | KS | Radius | Bone | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward tilt | early **15**, clean **16-19** | **40** | 0 (16-19 only) | 13 | 361 (3) | 55 | 82 | 3.5 → 4 | sword2 (x 7.8, z -2.5) [E] | 1.0 | the far blade box comes out on 16; frame 15 "hits targets above him" (UFD) |
| | | | 1 | 13 | 361 (3) | 55 | 82 | 4.0 → 5 | sword2 (x 1.8) [E] | 1.0 | |
| | | | 2 | 13 | 361 (3) | 55 | 82 | 2.9 → 4 | armr [E] | 1.0 | `clang=f rebound=f` |
| | | | 3 | 13 | 361 (3) | 55 | 82 | 2.1 → 3 | colonells [E] | 1.0 | `clang=f rebound=f` |
| Up tilt | **8-12** | **36** | 0 | 11 | **95** (0) | 30 | **103** | 3.4 → 4 | sword2 (x 8.4) [E] | 0.9 | the tip |
| | | | 1 | 11 | 85 (0) | 30 | 111 | 4.3 → 5 | sword2 (x 2.0) [E] | 0.9 | |
| | | | 2 | 11 | 85 (0) | 30 | 105 | 2.7 → 3 | armr [E] | 0.9 | `clang=f rebound=f` |
| | | | 3 | 11 | 85 (0) | 30 | 104 | 2.16 → 3 | colonells [E] | 0.9 | `clang=f rebound=f` |
| Down tilt | **10-11** | **31** | 0 | 9 | **85** (3) (13.0.1; 80 before) | **90** | **30** | 2.0 → 3 | top (1.3 → 5.0, 17.5 → 5.5) | 1.0 | `clang=f rebound=f`; a long low capsule |
| Dash attack | **20-23** | **57** | 0 | **14** | 45 (3) | 70 | 85 | 3.2 → 4 | sword2 (x 8.5) [E] | 1.2 | the tip; **shieldstun x1.4** (every box) |
| | | | 1 | 13 | 50 (3) | 85 | 85 | 3.5 → 4 | sword2 (x 3.0) [E] | 1.0 | |
| | | | 2 | 12 | 55 (3) | 85 | 77 | 3.0 → 4 | armr [E] | 1.0 | `clang=f rebound=f` |

The up tilt's boxes have no angle type (0, by position): it sweeps from behind over the head [R].

### Smashes [R]

Smash charge: the shared rule (60 frames, x1.4). Charge hold frames: forward smash **9-10**, up smash **5-6**, down smash **4-5** [R].

| Move | Frames | FAF | id | Dmg | Angle (type) | BK | KS | FKV | Radius | Bone | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Forward smash hit 1 | **17-18** | **51** | 0 | 7 | **69** (3) | 45 | **12** | | 4.3 → 5 | sword2 (x 2.7, y -2.0) [E] | 1.0 | the blade: a weak lift into hit 2 |
| | | | 1 | **14** | **361** (3) | 30 | 100 | | 3.5 → 4 | sword2 (x 8.5, y -2.0) [E] | 1.0 | **the tip** |
| | | | 2 | 7 | 60 (3) | 45 | 12 | | 3.2 → 4 | top (8.5, 6.0 → 3.5) | 1.0 | the body |
| Forward smash hit 2 (attack pressed on **23-36**: continuable) | **10-11** of its own move | **61** | 0 / 1 | 13 | 48 (3) | 85 | 89 | | 4.5 → 6 | sword2 (x 7.0 / 1.5, y 2.3) [E] | 1.5 | **charged by hit 1's charge** (`ChargedSmashDmgSSBU` on hit 2's rows) [R] |
| | | | 2 | 12 | 48 (3) | 85 | 89 | | 3.5 → 4 | armr (x 1.0) [E] | 1.5 | |
| | | | 3 | 12 | 48 (3) | 85 | 89 | | 3.5 → 4 | top (9.0, 3.0) | 1.5 | `clang=f rebound=f` |
| Up smash hit 1 | clean **10-13**, late **14-15** | **78** | 0 | 4 | 105 (0) | 0 | 100 | **55** | 3.5 → 4 | colonells [E] | 1.0 | set weight, SDI 0.5, **hitstun +3** (9.0.0) |
| | | | 1 / 3 | 4 | 115 (0) | 0 | 100 | **38** / 28 (late id 3: **135**, FKV 45, **no +3**) | 4.8 → 6 / 3.0 → 4 | sword2 (x 2.5 / 8.5) [E] | 1.0 | **aerial targets only** (`g=f`); set weight, SDI 0.5, +3 |
| | | | 2 / 4 | 4 | **107** (0) | 0 | 100 | **28** (9.0.0) | 4.8 → 6 / 3.0 → 4 | sword2 (x 2.5 / 8.5) [E] | 1.0 | **grounded targets only** (`a=f`); set weight, SDI 0.5, +3 |
| Up smash hit 2 | clean **25-28**, late **29-30** | | the same five boxes at **3%** | | | | | as hit 1 (late id 3: 135, **FKV 40**, no +3) | | | | | |
| Up smash hit 3 | **41-45** | | 0 | **11** | **80** (0) | 60 | **101** (13.0.1) | | 4.3 → 5 | sword2 (x 7.0) [E] | 1.0 | the tip; `clang=f rebound=f` (every box of hit 3) |
| | | | 1 | 11 | 90 (0) | 60 | 101 | | 4.3 → 5 | sword2 (x 1.0) [E] | 1.0 | |
| | | | 2 | 10 | 90 (0) | 60 | 101 | | 3.0 → 4 | armr [E] | 1.0 | |
| | | | 3 | 9 | 90 (0) | 60 | 101 | | 5.2 → 7 | top (11.0, -0.5) | 1.0 | |
| Down smash front | **12-13** | **55** (conflict 4: UFD / dragdown 57) | 0 | 16 | 78 (0) | 40 | 88 | | 3.5 → 4 | sword2 (x 1.5) [E] | 1.0 | **trip 30** on every box (never trips at these angles: ssbwiki.com trivia) |
| | | | 1 | **17** | 78 (0) | 40 | 88 | | 3.6 → 5 | sword2 (x 7.5) [E] | 1.0 | the tip |
| | | | 2 | 16 | 78 (0) | 40 | 88 | | 3.4 → 4 | armr [E] | 1.0 | |
| | | | 3 | 14 | 78 (0) | 40 | 88 | | 3.0 → 4 | colonells [E] | 1.0 | |
| Down smash back | **24-25** | | 0 / 1 | 12 | **30** (0) | **80** | 63 | | 3.5 → 4 / 3.6 → 5 | sword2 [E] | 1.0 | the semi-spike; trip 30 |
| | | | 2 | 11 | 30 (0) | 80 | 63 | | 3.4 → 4 | armr [E] | 1.0 | |
| | | | 3 | 10 | 30 (0) | 80 | 63 | | 3.0 → 4 | colonells [E] | 1.0 | |

The up smash's hits 1 and 2 have different boxes for aerial and grounded targets (`g=f` / `a=f`) and their late phase changes only the
tip's aerial box [R]. Its 13.0.1 FAF 78 agrees in all three sources.

**The Sword Beam** (forward smash hit 1, **only when the wielder is at exactly 0.0%**; both ssbwiki.com and UFD) [R]:

| Property | Value |
| --- | --- |
| Appears | frame **17** (with hit 1), at offset **[11, 6]** (forward 11, up 6) |
| Active | **17-48** at no charge, **17-56** at full charge (lifetime **32 → 40** frames) |
| Damage | **5 + 8 x charge / 60** (5% to 13%), no falloff stated |
| Hit | angle 361 (3), BK 45, KS 15, radius 1.8 (→ 2) scaled by the size multiplier, a capsule z -7.0 → 7.0 (a 14-unit horizontal bolt), H 1.0, `direct=f`, **reflectable, absorbable** |
| Size multiplier | **0.8 → 1.1** |
| Speed | **2.4 → 3.4** (initial), friction (deceleration) **0.01** a frame |
| Scaling | "all values except offsets and friction scale linearly from no charge to full charge" |

### Aerials [R]

| Move | Frames | Autocancel | FAF | Landing lag | id | Dmg | Angle (type) | BK | KS | FKV | Radius | Bone | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Neutral air | clean **7-8**, late **9-31** | **1-3, 36+** | **39** | **6** | 0 | **11** | 361 | 22 | 100 | | 5.0 → 6 | kneer (x 3.5) [E] | 1.0 | the foot |
| | | | | | 1 / 2 | 9 | 361 | 22 | 100 | | 5.0 → 6 / 3.0 → 4 | kneer (x -2.0) / kneel (x 1.5) [E] | 1.0 | |
| | late | | | | 0-2 | 6 | 361 | **15** | 100 | | 4.5 → 6 / 4.5 → 6 / 3.0 → 4 | as above | 1.0 | |
| Forward air hit 1 | **16-17** | **53+** (after the FAF: never in a short hop) | **52** | **11** | 0 | 8 | **43** (3) | 31 | 28 | | 4.0 → 5 | top (11.5, 18.0) | 0.8 | **grounded targets only**; set weight |
| | | | | | 1 | 8 | **367** (3) | 25 | 30 | | 4.0 → 5 | top (11.5, 18.0) | 0.8 | **aerial targets only**; set weight |
| | | | | | 2 | 8 | 47 (3) | 33 | 30 | | 6.5 → 8 | top (11.5, 11.5) | 0.8 | grounded only; set weight; `clang=f rebound=f` |
| | | | | | 3 | 8 | **367** (3) | 25 | 30 | | 6.5 → 8 | top (11.5, 11.5) | 0.8 | aerial only; set weight; `clang=f rebound=f` |
| | | | | | 4 | 8 | 47 (3) | 33 | 28 | | 7.0 → 9 | top (11.5, 6.5) | 0.8 | grounded only; set weight; `clang=f rebound=f` |
| | | | | | 5 | 8 | **38** (3) | 26 | 26 | | 7.0 → 9 | top (11.5, 6.5) | 0.8 | aerial only; set weight; `clang=f rebound=f` |
| Forward air hit 2 | **24-25** | | | | 0 | 10 | 44 (3) | 43 | **120** | | 4.5 → 6 | top (11.0, 17.5) | 1.2 | |
| | | | | | 1 | 10 | 44 (3) | 43 | 120 | | 6.5 → 8 | top (11.0, 11.5) | 1.2 | |
| | | | | | 2 | 10 | 44 (3) | 43 | 120 | | 6.8 → 9 | top (11.0, 6.7) | 1.2 | `clang=f rebound=f` |
| Back air hit 1 | **6-8** | **29+** | **31** | **6** | 0 | 5 | **73** (4) | 0 | 100 | **50** | 4.0 → 5 | kneer (x 5.0) [E] | 0.8 | grounded only; set weight |
| | | | | | 1 | 5 | 70 (4) | 0 | 100 | **36** | 4.0 → 5 | kneer (x 5.0) [E] | 0.8 | aerial only; set weight |
| | | | | | 2 / 4 | 5 | 70 (4) | 0 | 100 | 50 | 4.5 → 6 / 5.0 → 6 | kneer (x -0.5) / hip [E] | 0.8 | grounded only; set weight |
| | | | | | 3 / 5 | 5 | 66 (4) | 0 | 100 | 36 | 4.5 → 6 / 5.0 → 6 | kneer (x -0.5) / hip [E] | 0.8 | aerial only; set weight |
| Back air hit 2 | **15-17** | | | | 0-2 | 7 | 361 (4) | 45 | 98 | | 4.5 → 6 / 5.0 → 6 / 5.5 → 7 | kneel (x 5.0 / -0.5) / hip [E] | 1.0 | |
| Up air | clean **11-13**, late **14-40** | **1-10, 43+** (2.0.0) | **60** | **14** | 0 | **15** | 85 (0) | 23 | 93 | | 4.5 → 6 | sword2, a capsule x 6.9 → -1.5 [E] | 1.2 | the long stab |
| | late | | | | 0 | 13 | 85 (0) | 18 | 88 | | 4.5 → 6 | as above | 1.2 | |
| Down air | clean **14-19**, late **20-64** | **1-9, 65+** | **80** | **19** | 0 | **18** | **270** (0) | 30 | 80 | | 4.0 → 5 | top (1.0, 0.0), x 1.5 | 1.2 | **aerial targets only**: the meteor |
| | | | | | 1 | 18 | 60 (0) | 30 | 78 | | 4.0 → 5 | as above | 1.2 | **grounded targets only** |
| | late | | | | 0 | 15 | 60 (0) | 30 | 80 | | 4.2 → 5 | top (1.5 → 4.0), x 1.0 | 1.2 | |
| | after a bounce | | | | 0 | **11** | **65** (0) | **50** | 80 | | 4.3 → 5 | top (1.0, 0.5), x 1.0 | 1.2 | the second (and third) hit after bouncing |

**Down air bounce** [R]: "upon hitting a target **without fast falling**, Link bounces off them, with his **vertical momentum being set
to 1.33**, and the hitbox reappears **after 30 frames**" (the after-bounce hit above); the box can reappear at most twice (three hits),
but the 30-frame gaps outlast the move, so the limit never matters (ssbwiki.com trivia). UFD agrees ("can hit same target once again
after 30 frames due to pogo effect").

**Up air's long hitbox** [R]: one box (a capsule along the blade) out from 11 to 40, 15% on 11-13 and 13% after; autocancel from 43
(2.0.0). UFD's total 59 agrees with FAF 60 (ssbwiki.com's "animation length 59" is one frame shorter than its own FAF: a table
quirk) [D].

### The forward air's first hit drags (owner decision 4) [R]

Owner decision 4 asks that the forward air's first hit drag the target down so the second hit connects, "if the reference's first hit
does not already do this". **The reference already does, on aerial targets**: hit 1's two outer boxes against **aerial** targets
(ids 1 and 3, the tip and middle of the swing) are **angle 367 (autolink), set weight, BK 25 KS 30** [R]. The autolink sends the
target with the attacker's own movement plus a pull toward the box, so a falling Link carries an airborne target down with him into
hit 2 (24-25) [R, ssbwiki.com `Autolink_angle`]. Two details to keep exactly:

- The **inner box against aerial targets** (id 5, the part of the swing nearest the body) is **not** an autolink: angle **38**, set
  weight, BK 26 KS 26 [R] (a weak, low push that still leaves the target close).
- Against **grounded** targets hit 1 is never an autolink: angles **43 / 47 / 47**, set weight, BK 31-33, KS 28-30 [R] (a small lift).

So decision 4 is met **by the reference data itself** on the outer two-thirds of the swing; making the inner aerial box (or the grounded
boxes) drag as well would be an **owner-chosen change**, to be recorded as such if wanted (question 2 in the audit).

### Grabs, pummel and throws [R]

| Move | Frames | FAF | Boxes / hits | Other |
| --- | --- | --- | --- | --- |
| Standing grab | **6-7** | **35** | id 0 r 3.3 (→ 4) top (8.0, 4.0 → 7.7) **grounded only**; id 1 r 1.65 (→ 2) (8.0, 2.35 → 9.35) **aerial only** | **a hand grab, no tether** (`air_lasso_type` False): the third-shortest grab in the game, but fast [R] |
| Dash grab | **9-10** | **43** | id 0 r 2.6 (→ 3) (7.0, 4.0 → 9.4) grounded; id 1 r 1.3 (→ 2) (7.0, 2.7 → 10.7) aerial | |
| Pivot grab | **10-11** | **38** | id 0 r 3.3 (→ 4) (7.0, -4.0 → -13.2) grounded; id 1 r 1.65 (→ 2) (7.0, -2.35 → -14.85) aerial | |
| Pummel | **1** | **7** | 1.3%, 361 (3), BK 0, KS 100, **FKV 30**, r 4.8, top (7.0, 7.0), **H 2.1**, `clang=f rebound=f` | UFD's total 18 "includes 13 frames of hitlag" [R] |
| Forward throw | hit **12-13**, release **13** | **38** | hit: 3%, 361 (3), BK 55, KS 55, kneer r 3.8, `clang=f`; throw: **2.5%**, angle **33**, BK 75, KS 85 | thrower invincible 1-13 |
| Back throw | hit **12-13**, release **14** | **39** (conflict 5: dragdown 35) | hit: 3%, 361 (**4**), BK 55, KS 55, kneer r 3.5; throw: 2.5%, angle **130**, BK 67, KS 110 | invincible 1-14 |
| Up throw | hit **26-29**, release **28** | **50** | hit (bystanders; the thrown fighter "is always hit by the sweetspot"): 5%, angle **20** (3), BK 50, KS 100, sword2 r 3.8 at x 6.8 and x 2.0; throw: **2%**, angle **90**, BK 24, **KS 230** | invincible 1-28; ssbwiki.com's moveset text gives the base box 4% (conflict 7) |
| Down throw | hit **22-23**, release **24** | **49** | hit: 3%, 361 (0), BK 60, KS 150, top (2.4, -8.4) r 4.3; throw: **3%**, angle **83**, BK 60, KS 125 | invincible 1-24; "if used at an edge, the opponent is pushed 2.5 units up and away" [R] |

Throws' interrupted rows (`kind=1`: 3%, 361, BK 40, KS 100) are the shared limit of every port. Totals: forward 5.5, back 5.5, up 7
(the thrown fighter takes the 5% box and the 2% throw [R reading of UFD "5.0/2.0"]), down 6 [R].

### Ledge and floor [R]

| Move | Frames | FAF | Hit | Other |
| --- | --- | --- | --- | --- |
| Ledge attack | **20-22** | **56** | 9%, angle **45** (3), BK 90, KS 20, r 5.0 (→ 6), top (5.0, 15.5 → -4.0), `clang=f rebound=f` | intangible 1-22 (first grab) |
| Get-up attack (back) | hits **17-18** (behind) and **21-22** (in front) | **46** | 7% each, angle **48** (0), BK 80, KS 48, r 5.0 (→ 6), shield **+8** (→ 214%), `clang=f rebound=f` | intangible 1-22 |
| Get-up attack (front) | **17-18** (behind), **21-22** (in front) | **46** | the same numbers (zpos -15 → -4 then 17 → 4) | intangible 1-22. One get-up attack slot here (owner decision 36): the two share every number but the reach, so one slot loses nothing [D] |
| Trip attack | **19-20** (in front), **28-29** (behind) | **50** | 5% each, angle 361 (0), BK 60, KS 50, r 5.0, shield **+8** (→ 260%), `clang=f rebound=f` | intangible 1-7; not modelled (the knockdown's get-ups are used, as every port) |

## Specials

### Bow and Arrows (neutral special) [R]

| Property | Value | Source |
| --- | --- | --- |
| Charge | holds on frame **15-16** ("charges between 15-16"); **full at 36 charge frames** ("with at least 36 charge frames, the move immediately skips to full charge"), i.e. full on frame **52** (UFD "reaches full charge on frame 52") | ssbwiki.com, UFD [R] |
| Release | the arrow appears on the frame of release: **16-61** without charge (**1-46** from release); FAF **45** (30 from release); animation 66 (51) | ssbwiki.com [R]; UFD total 44 → 80 [R] (80 = 44 + 36) |
| Auto-fire | **maximum hold 180 frames** (3 seconds), then it fires by itself | ssbwiki.com parameters [R] |
| Charge used for scaling | "the charge time of 60 frames is only used for the arrows to scale in damage and speed": c from 0 to 60, and **c = 60 once 36 frames are reached** | [R]; so the scale jumps from 35/60 to full on the 36th frame [D] |
| Damage | **floor(4 + 8 x c / 60)**: 4% to 12% | [R] |
| Speed | **2.6 → 10** (uncharged → full), linear in c [R reading]; lifetime **46** frames in flight (2.0.0) | [R] |
| Flight | fired at **2.5 degrees** upward, **gravity 0.054** | [R] |
| Hit (single) | angle 361 (3), **BK 10, KS 71**, r 1.35 (→ 2), H 0.8, `direct=f`, reflectable; **does not pierce** | [R] |
| Double arrow | with an **arrow item in hand**, the move fires **two arrows**: the second **3 frames** after the first, at **0.9x** speed and **2.3 degrees**; each deals **floor(0.8 x the single damage)** (6% to 18% for the pair); their hit is **BK 25, KS 110** | ssbwiki.com move page [R]; the `Bow_and_Arrows` article says **0.75x** (conflict 8) |
| Lodging | an arrow that hits **the ground or a wall** sticks there: **180 frames** (a single arrow; the 3 seconds UFD gives), **120 / 30** frames (the first / second of a double shot); "minimum stick angle 9 degrees" (read as: it sticks only if it meets the surface at 9 degrees or more [E: reading]); an arrow in a fighter shows for 5 frames (head) or 30 (elsewhere) (presentation) | [R] |
| As an item | a lodged arrow can be **picked up by Link or any other fighter** ("picked up and thrown by Link or other characters as an item") and thrown; it **disappears on hitting a surface or a target**; shielded: vanishes | ssbwiki.com `Bow_and_Arrows` article [R] |
| Thrown arrow | item damage **1 - 0.4 + speed x 0.8** (2.1% to 3.9%: the moveset table); hit angle 361 (3), BK 10, KS 66, r 1.35 (a rotated version r 1.2); item lifetime 40 frames thrown; gravity 0.053, fall 2.3, friction 0.0025 air / 0.03 ground; throw speed multiplier 1; collision half-lengths 3.5 x 3; pick-up box 7 x 6 | [R] |

**Aerial use**: the same frames; the reference gives no separate aerial table (gap 7: landing during the bow, air physics while
charging).

### Boomerang (side special) [R]

| Property | Value | Source |
| --- | --- | --- |
| Throw | the boomerang appears on frame **27**; Link's FAF **46**; animation 63 | ssbwiki.com, UFD (total 45) [R] |
| Aim | the stick angles it up or down: "vertical angle deviation **26 degrees**", "stick sensitivity to angle vertically 0.1" | [R]; the sensitivity's meaning is [E] (a dead zone of 0.1 is the reading) |
| Smash input | a smash-style input throws it **further and harder**: speed **3.1** (tilt **2.4**), lifetime **167** frames (tilt **125**; "coded as 171 / 129"), clean and late hits **x1.2** (9.6 / 7.2) | [R] |
| Flight | decelerates **0.047** a frame to a **minimum speed 0.3**; "distance spent turning back" **16** (tilt) / **24** (smash); returns at up to **3** a frame, turning toward Link by up to **0.75 degrees a frame** (homing); spins 22 / 26 degrees a frame (presentation) | [R]; the turn-back's exact path is [E: reading] |
| Hits (tilt) | **clean 27-36**: 8%, angle **70**, type **1** (by travel), **BK 80**, KS 40, r 4.0 (→ 5), shield -4 (→ 50%); **late 37-71**: 6%, **65**, type 1, BK 60, KS 40, r 3.6 (→ 5), shield -3 (→ 50%); **returning 72-151**: 3%, 70, type 1, **BK 50**, KS 40, r 3.6, shield -1.5 (→ 50%); H 1.0, `direct=f`, reflectable; **does not pierce** | [R] (frames from the move's start) |
| Hits (smash) | clean **27-36**, late **37-86**, returning **87-192** | [R] |
| Direction | type 1 sends **the way the boomerang is moving**: up-and-forward on the way out, **toward Link** on the way back | [R] (`af=1`); ssbwiki.com moveset text agrees |
| One at a time | "Link can only have one boomerang on screen at any time; trying the attack again while a boomerang is out only results in Link doing his boomerang throwing animation without actually throwing anything": the **failed throw**, FAF **60** grounded / **46** aerial (animation 63) | ssbwiki.com `Boomerang`, move page [R] |
| Catch | Link must **catch it as it returns** (it does not vanish on its own when it reaches him unless caught); catching removes its hitboxes; the **catch animation** (FAF **10**, animation 31; UFD "9-frame animation") plays only if Link is not attacking or defending | [R] |
| Dodged | "if the player dodges the boomerang as it comes back to them, it will continue flying and curving towards the direction they dodged in" | ssbwiki.com `Boomerang` (series text) [R, not version-specific] |
| Ground | it bounces off the ground at a minimum of **16 degrees**, or goes **through** the floor if its angle to it differs by less than **30 degrees**; ground collision range -0.5 | [R]; reading [E] |
| Clash | after **clashing with an attack** it falls: lifetime **60**, speed **-0.15 / +1.25**, gravity 0.09, fall 1.4 (rotation x0.2), no hit | [R] ("knocked down by another projectile" in the roster notes is the same clash read for projectiles [E]) |
| Reflected | a reflector turns it round under the reflector's owner, so it can hit Link | [R reading of `reflectable`] |
| Wind | **no wind box** on any 13.0.1 box (the raw tables list none) | [R]; the roster's "verify the Boomerang's return wind" is answered: none |

### Remote Bomb (down special) [R]

| Property | Value | Source |
| --- | --- | --- |
| Pull | the bomb appears in Link's hand on frame **17**; FAF **40**; animation 69. **It cannot be pulled while Link holds an item** | ssbwiki.com move page, UFD (39) [R] |
| Hold | a **held item**: Link carries it and can throw it like any item (forward / back / up / down, smash throws, dash throw, a grounded drop, the aerial drop "Z-drop"); up or down with the attack stick (the C-stick) | ssbwiki.com `Remote_Bomb`, `Item_throw` [R] |
| Throw frames (this body's item throws) | released on forward **7**, back **9**, up **11**, down **6**, dash **4**, drop **5**; totals **24 / 24 / 23 / 19 / 29 / 23** (the 13.0.1 rule: "every character 24 total frames for forward and back, 23 for up and the grounded drop, 19 for down, 29 for the dash throw") | ssbwiki.com `Item_throw` [R]; its table's older totals (20 / 20 / 20 / 18 / 18 / 23) are the earlier game's (conflict 9) |
| Throw speeds (light item) | forward / back **2.6** (smash 3.64) at 17 degrees (smash 11); up **3.1** (4.1) at 90; down **3.3** (4.62) at -90; dash **3.4** at 12; drop **2.1** at 100 (grounded); aerial drop: no speed | `Item_throw` [R]; each x this body's toss strength **1.01333** and x the bomb's throw speed multiplier **0.6264** ("coded as 0.72 for the item, then multiplied by 0.87"); the horizontal part gets the multiplier twice (`sqrt((t cos a m^2)^2 + (t sin a m)^2)`) [R] |
| Thrown hit | **1%** flat (the bomb's item damage factors are 0 and 0: no speed bonus), angle 361, type **1**, BK 10, KS 100, r 2.0 (→ 3), H 0.6, SDI 1.0, `hitbits=No floor` [R; meaning unclear: gap 12], reflectable; **bounces off fighters** (x0.2 horizontal, x0.2 vertical **+1** up) | [R] |
| Physics | gravity **0.07**, fall **2.3**, friction 0.01, max horizontal speed **4**; bounces off floor (x0.9 / x0.7), walls (x0.7 / x0.6, +1), ceiling (x0.9 / x0.6); bounces only above speed 0.4; speeds after a bounce clamped 0.05-3 horizontal, 0.1-2.3 vertical; size x0.81; collision half-lengths 2 x 2; pick-up box 4 x 4 | [R] |
| Shielded | **bounces off** a shield | [R] |
| Knocked about | it has a **hurtbox (size 4)** and **50 HP**; an attack launches it at **1 to 3.2** speed for **1% to 15%** damage taken, with hitlag x0.5 | [R]; what happens at 0 HP is not stated (gap 11) |
| Detonate | **pressing down special again** while a bomb exists: the detonation move, explosion on **12-13** ("explosion delay when detonated 12 frames"), FAF **40**, animation 60. A bomb in Link's own hand explodes there [R reading] | [R] |
| Cannot detonate | while Link is **helpless** (after the aerial Spin Attack, for instance), or while **another fighter holds it** (or a pocketing move holds it) | ssbwiki.com move page and `Remote_Bomb` [R] |
| Fuse | **1800 frames** (30 s) maximum, flashing for the last **45**; it explodes **at once if hit by fire** | [R] |
| One at a time | down special with a bomb out detonates it instead of pulling another [R reading of "using the move again has Link detonate"] | [R] |
| Explosion | **7%**, angle **52** (0), **BK 60, KS 115**, **r 14.0** (→ 18), H 0.8, SDI 1.0, `direct=f`, `clang=f rebound=f`; **it damages Link too** (UFD "can self-damage"; ssbwiki.com: Link "can be harmed by his own bomb's explosion" and uses it to recover without becoming helpless) | [R] |
| Item throws by anyone | the bomb "can be grabbed as an item" by other fighters; the owner cannot detonate it while they hold it | `Remote_Bomb` [R] |

The bomb is an **item**, not a projectile, for its whole life (pulled, held, thrown, lying on the stage, knocked about, picked up), and
only its explosion is an attack.

### Spin Attack (up special) [R]

The kit's iconic move: two different moves on the ground and in the air (owner decision 2: match the reference).

**Grounded: a single charged hit in four timing phases.** One hit per target for the whole move (the four phases are the same hit
getting weaker over time, not four hits) [R].

| Phase | Frames (from release) | id | Dmg | Angle | BK | KS | Radius | Bone | H | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Clean | **7-8** (4-5) | 0 / 1 | **14** | **45** | **60** | **88** | 4.0 → 5 / 3.5 → 4 | sword2 (x -2.0 / 2.5, z 0 / 1.0) [E] | **1.4** | `clang=f rebound=f` |
| | | 2 | **11.2** (tip x0.8) | 45 | 60 | 88 | 2.8 → 4 | sword2 (x 8.0, z 1.5) [E] | 1.0 | the tip, weaker |
| Mid | **9-13** (6-10) | 0 / 1 | 12 | 43 | 52 | 88 | as above | | 1.0 | |
| | | 2 | 9.6 | 43 | 52 | 88 | | | 1.0 | |
| Late | **14-23** (11-20) | 0 / 1 | 9 | 43 | 48 | 84 | | | 1.0 | |
| | | 2 | 7.2 | 43 | 48 | 84 | | | 1.0 | |
| Latest | **24-39** (21-36) | 0 / 1 | 7 | **361** | **20** | **82** | | | 1.0 | no `clang=f` on the latest phase |
| | | 2 | 5.6 | 361 | 20 | 82 | | | 1.0 | |

| Property | Value | Source |
| --- | --- | --- |
| Charge | holds **between frames 3 and 4** while special is held, up to **60 frames** | ssbwiki.com [R] |
| Damage multiplier | **1 + 0.6 x charge / 60** (to 1.6x: 22.4% clean) | ssbwiki.com [R] |
| FAF | **77** (74 from release); animation 94 | ssbwiki.com, UFD (76), dragdown (76) [R] |
| Startup | 7 is **the front hit only** (UFD): the blade sweeps round, so behind Link the phases start later | UFD [R]; where the sweep is each frame is not published (gap 3) |
| Movement | none stated (a standing spin) | [E] |
| Out of shield | 7 frames (front only) | UFD [R] |

**Aerial: eight spin hits and a finisher, rising.** Each part is its own hit (the front and back pairs alternate) [R]:

| Part | Frames | Dmg | Angle (type) | BK | KS | FKV | Set weight | Radius | Position (top: y, z) | H | SDI |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Front 1 | **8-9** | **4** | **367** (3) | 0 | 100 | **107** | yes | 6.0 → 8 | (10.0, 14.5 → 8.0) | 1.0 | 0.3 |
| Back 1 | **12-13** | 2 | **145** (4) | 0 | 100 | **95** | **no** (4.0.0) | 5.0 → 6 | (11.5 → 11.0, -12.0 → -9.0) | 1.0 | 0.3 |
| Front 2 | **16-17** | 2 | **367** (3) | 0 | 100 | 107 | yes | 6.0 → 8 | (6.0 → 7.5, 15.0 → 8.0) | 1.0 | 0.3 |
| Back 2 | **19-20** | 2 | 145 (4) | 0 | 100 | 95 | no | 5.0 → 6 | (11.5 → 11.0, -12.5 → -9.0) | 1.0 | 0.3 |
| Front 3 | **22-23** | 2 | **83** (3) | 0 | 100 | **71** | yes | 6.0 → 8 | (10.0 → 10.5, 15.0 → 8.0) | 1.0 | 0.3 |
| Back 3 | **26-27** | 2 | **150** (4) | 0 | 100 | 95 | no | 5.0 → 6 | sword2 (x 2.0 → 5.5) [E] | 1.0 | 0.3 |
| Front 4 | **31-32** | 2 | **69** (3) | 0 | 100 | **62** | yes | 7.0 → 9 | (14.0 → 12.5, 12.5 → 8.0) | 1.0 | 0.3 |
| Back 4 | **38-39** | 2 | **160** (4) | 0 | 100 | 95 | no | 5.0 → 6 | sword2 (x 2.0 → 5.5) [E] | 1.0 | 0.3 |
| Final (early / mid / late) | **47 / 48 / 49** | **4** | **361** (3) | **30** | **200** | | no | 7.5 → 9 | (7.5 → 9.5, 14.0 → 7.5) / (13.0 → 12.5, 15.5 → 7.5) / (19.0 → 14.0, 12.5 → 7.5) | **2.0** | 1.0 |

Every aerial box is `clang=f rebound=f` [R]. Total 22% [D]. **All back hits use a position vector with offsets [10, 20] for 10
frames** [R]: the target is sent toward a point **10 units in front of and 20 above Link's root**, at a speed that covers the distance in
10 frames, after which the launch speed drops off quickly (ssbwiki.com `Autolink_angle`, "position vectors") [R]. This is what carries a
target caught behind Link round into the next front hit; the front hits do the same with the autolink angle. Since 4.0.0 the back hits
no longer use set weight, and the spin's autolink knockback is reduced (it used to launch hard when the move was interrupted) [R].

| Property | Value | Source |
| --- | --- | --- |
| Rise | **initial vertical speed 2.4**; **gravity x0.55** | ssbwiki.com parameters [R] |
| Air control | rising: **air speed x0.5**, additional air acceleration **0.22**; ending (**from frame 48**): air speed **0.8**, additional acceleration 0.01; special fall: **air speed x0.909**, additional acceleration **0.025** | [R]; whether 0.22 / 0.01 / 0.025 replace the body's additional acceleration (0.04) or multiply it is not stated (gap 4); replacing is the reading [E] |
| Rise height | with 2.4 a frame and 0.096 x 0.55 = 0.0528 gravity, the rise lasts about **45 frames** and climbs about **54.5 reference units** (6.8 world units) before the ending physics [D from the [R] parameters, if the initial speed applies on frame 1 and the gravity multiplier holds through the rise] | [D] |
| Ledges | grabs a ledge **in front from frame 15**, **behind from frame 34** | ssbwiki.com [R] |
| Jostling | off on frames 26-49 (no jostling in the air here) | [R] |
| Animation | **91** frames (dragdown total 92), then the special fall | [R] |
| Landing | **landing lag 30** (the landing animation interruptible on 31) | ssbwiki.com, dragdown [R] |
| After | **helpless** (special fall) to the ground or a ledge | [R] |

## Conflicts and how they are resolved

1. **Walk speed**: the parameter file (dragdown) and UFD **1.247**; ssbwiki.com's attributes template **1.237**. Resolved to **1.247**
   (the parameter file, two sources).
2. **Jab 2's FAF**: ssbwiki.com **25**; UFD and dragdown total 25 → FAF **26**. **Unresolved**; proposed **26** [E: two sources; their
   "total" is FAF - 1 on every other move here], footage settles it.
3. **Jab 1 → jab 2's earliest frame**: ssbwiki.com window **10-35**; UFD "as early as frame 12". Proposed **10** (the raw table's named
   window) [E]; footage settles it.
4. **Down smash's FAF**: ssbwiki.com **55**; UFD and dragdown total 56 → FAF **57**. **Unresolved**; proposed **57** [E: two sources],
   footage settles it.
5. **Back throw's FAF**: ssbwiki.com **39**, UFD total 38 → **39**; dragdown total 34. Resolved to **39** (two sources).
6. **Boomerang's clean shield damage**: the update list says 0 → **-4.5**; the move page's raw table **-4.0**. Resolved to the raw
   table's **-4** (the named field) [R]; both give about half the damage on a shield.
7. **Up throw's base box**: the raw table gives both sword boxes **5%**; the moveset text gives "4% (hit 1, tip)" for the base box on
   bystanders. Resolved to the raw table (5%) [R]; only bystanders see it.
8. **Double arrow damage**: the move page's parameters **x0.8** each, floored (6% to 18% for two); the `Bow_and_Arrows` article
   **0.75x** each (1.5x in total). Resolved to **x0.8, floored** (the parameter table; its 6-18% totals agree with the moveset table's
   "6% / 18% (two arrows)") [R].
9. **Item throw totals**: the `Item_throw` table lists this body's older totals 20 / 20 / 20 / 18 / 18 / 23; the same page says 13.0.1
   gives every fighter 24 / 24 / 23 / 19 / 29 / 23. Resolved to the **13.0.1 rule** (the release frames 7 / 9 / 11 / 6 / 4 / 5 stand)
   [R reading].
10. **Forward smash hit 2's earliest frame**: ssbwiki.com continuable **23-36**; UFD "as early as frame 22". Proposed **23** (the raw
    table) [E].
11. **Aerial Spin Attack's length**: ssbwiki.com animation **91**, dragdown total **92**. Not a gameplay difference (it ends in the
    special fall either way); 91 is used [R].

## Gaps (needed for 1:1, not published)

1. **The Hylian Shield's push**: how far a blocked projectile pushes the wielder (the articles say only "dependent on damage and
   knockback"), and its hitlag.
2. **The Hylian Shield's shape**: where the block area is (height, width) standing, walking and crouching; whether a turn or a landing
   frame counts as standing.
3. **The grounded Spin Attack's sweep**: where the blade is on each active frame (the boxes ride the `sword2` bone round the body; only
   "the front hit is on 7" is published). Needed to know when a target behind is hit.
4. **The aerial Spin Attack's acceleration numbers**: whether 0.22 / 0.01 / 0.025 replace or multiply the body's additional air
   acceleration; on which frame the 2.4 rise speed is set; whether horizontal momentum is kept at the start.
5. **The aerial Spin Attack's height**: the [D] 54.5 units above is from the parameters; a measured height would confirm it.
6. **The Boomerang's return path**: how "distance spent turning back" (16 / 24) turns into a path, and what the stick's 0.1
   "sensitivity" means.
7. **The bow in the air**: air physics while charging, and what landing does to it.
8. **Arrow lodging rules**: the "minimum stick angle 9 degrees" reading; whether an arrow lodges on a platform's top only or also under
   it; whether lodged arrows can be picked up in the air.
9. **The Sword Beam's falloff**: whether its damage changes over its flight (none is listed).
10. **The bomb's pull and detonation in the air**: air physics during the two moves.
11. **The bomb at 0 HP**: whether a bomb whose 50 HP is used up explodes or breaks.
12. **`hitbits=No floor`** on the thrown bomb: its meaning (presumably it does not hit floor-type objects; no effect on fighters [E]).
13. **The double arrow's lodging**: why the first lodges for 120 and the second for 30 frames (both published; only the reason is
    missing).
14. **The item pick-up rules** (attack near an item on the ground, catching an item in the air): universal rules, not this
    character's; needed when E4 is built.
