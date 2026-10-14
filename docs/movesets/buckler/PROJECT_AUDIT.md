# Project audit for the tenth moveset (reference: the reference game's Link, adult)

What the engine needs to reproduce the reference game's adult Link 1:1 (version 13.0.1), traced through the code at **sim version 87**
(content schema 6, branch `link-research` from `origin/main` at `64147b0`, which contains `5b2bb84`). Nothing is built: this is research
only. The data is in `REFERENCE_DATA.md`; the engine inventory is `docs/movesets/SHARED_MECHANICS.md`; the extension ids are those of
`docs/movesets/ROSTER_STATUS.md` and the earlier audits (E29 from the Bracers, E30-E33 from Yoshi), with new ones numbered from **E34**.
Owner decisions 1-52 in `ROSTER_STATUS.md` apply; the ones that shape this port are **2** (Spin Attack: grounded single charged hit,
aerial multi-hit), **3** (an item system will be built; whether others can pick up his bombs and arrows is **undecided**), **4** (the
forward air drags on its first hit; research recorded in `REFERENCE_DATA.md`) and **36** (one get-up attack slot).

## Proposed names (not registered)

| Thing | Name | Checked |
| --- | --- | --- |
| Class / weapon | **Buckler** (the small shield that stops shots, the kit's one passive trait; the sword, bow, boomerang and bombs hang off it) | `content/blocklist.txt` has no terms (comments only); contains none of the `TRADEMARKS` terms in `sim-content/src/policy.rs` after its look-alike folding; not used anywhere in the code, content, `godot/scripts` or docs (searched 2026-10-13); not a reference fighter's name |
| Body | **wanderer** (a travelling swordsman with a pack of tools; low jumps, heavy) | same |
| Built-in fighter | **Wanderer** | same |
| Blurb | "A sword, a shield and a pack of tools: a blade with a long reach, a shield that stops shots while standing, a bow that leaves arrows to pick up, a boomerang that comes back, a bomb set off from afar, and a whirling spin that lifts it home." | original wording |

Alternatives if the owner prefers: weapon "Quiver", body "ranger" (both free). Avoided on purpose: the reference's move names and
anything with the reference series' or the character's name (its series name is a `TRADEMARKS` term), and "wayfarer" (a product
trademark). Class index **9** when registered if the Clutch (Yoshi) registers first as 8; `CLASSES` is 8 today (0-7). This body holds a
weapon (`Weapon::limbs` false, as the Longsword and the Greatsword).

## Iconic moves: what must match exactly

1. **Spin Attack, grounded** (up special on the ground). **A single hit** (one hit group for the whole move) in four timing phases:
   clean 7-8 (14% blade / 11.2% tip, 45, BK 60 KS 88, H 1.4 on the blade), mid 9-13 (12 / 9.6, 43, 52 / 88), late 14-23 (9 / 7.2, 43,
   48 / 84), latest 24-39 (7 / 5.6, 361, 20 / 82); the tip box 0.8x; `clang=f` except the latest phase; charge held on frame 3-4 while
   special is held, up to 60 frames, damage **x(1 + 0.6 x charge / 60)**; FAF 77 (74 from release); 7-frame out-of-shield use (front
   only). **Engine**: the phases are rows on one group (exists); the charge is a script `stall()` while `special` is held, which
   counts `Fighter::charge` up to `Ruleset::charge_frames` 60 and the hit then gets `charge_bonus` 60 → exactly `1 + 0.6 x c / 60`
   (`combat::resolve_hits`, the smash-charge code; `charge_at` itself only holds on the **attack** button, so the script is needed)
   [D]. Missing: the sweep's positions (gap 3, [E]) and **the out-of-shield start** (new **E37**, shared: today a shield only jumps,
   grabs, rolls or spot dodges, and a jump squat never starts a grounded up special, so out of shield gives the aerial spin 3 frames
   late).
2. **Spin Attack, aerial** (the up special's `air_version`). Eight spin hits alternating front (8-9, 16-17, 22-23, 31-32) and back
   (12-13, 19-20, 26-27, 38-39), then a 4% finisher on 47-49 (361, BK 30, **KS 200**, H 2.0); front 1-2 **367 set weight FKV 107**,
   front 3 83 / FKV 71, front 4 69 / FKV 62; back hits 145 / 145 / 150 / 160 FKV 95, **not** set weight (4.0.0), each with a **position
   vector [10, 20] over 10 frames**; SDI 0.3 on the spin hits; every box `clang=f`; rise: **vertical speed 2.4, gravity x0.55**, air
   speed x0.5 and added acceleration 0.22 while rising, from frame 48 air speed 0.8 and added 0.01; special fall at air speed x0.909,
   added 0.025; **ledges in front from 15, behind from 34**; **landing lag 30**; helpless after. **Engine**: the hits (groups, 367,
   angle types 3 / 4, `sdi`, `hitlag`, set weight), the ledge windows (`ledge_front_from` 14 / `ledge_back_from` 33 in move frames),
   the landing lag (`landing_lag`, `fall_landing_lag` 30), frame-ranged air physics (`air_blocks`: gravity, drift, `accel`) and
   helplessness exist. Missing: **the position vector (new E34)** (without it a target hit behind is not carried into the next front
   hit, and the move drops people: the one thing that makes it a multi-hit), and **separate special-fall speed and acceleration
   (E11 addition)** (`fall_drift` is one percent for both; the reference gives x0.909 speed and 0.025 acceleration). The rise speed is
   set on the first frame by a script `set_vel` (exists) [E: which frame, gap 4].
3. **Forward air** (decision 4). Hit 1 on 16-17: against **aerial targets** the outer and middle boxes are **367 set weight** (BK 25 KS
   30): the reference already drags a target along with a falling Link into hit 2 (24-25, 10%, 44, BK 43 **KS 120**); the inner aerial
   box is 38, the grounded boxes 43 / 47. **Engine**: data (`Hitbox::targets`, 367, set weight). If the owner wants the inner box or the
   grounded boxes to drag too, that is an owner-chosen change (question 2).
4. **Down air**: the 18% **aerial-only meteor** (270) / grounded 60 on 14-19, 15% late to 64; the **bounce** on a hit when not fast
   falling (vertical speed 1.33) with the 11% box (65, BK 50) back after 30 frames; autocancel 1-9 / 65+; landing lag 19. **Engine**:
   the hits are data; the bounce is a script on the Flip Kick pattern (`hit`, `set_vel`, `goto` to an after-bounce slot carrying the
   frame count in a var) but **needs a `fast_fall` register** (E16 addition: the bounce happens only without a fast fall).
5. **Remote Bomb**: pulled as a **held item**, thrown in every item direction or dropped, bounces off fighters and terrain, knocked about
   by hits (HP 50), **detonated by pressing down special again** (explosion 12 frames into the detonation move, 7% angle 52 BK 60 KS
   115 radius 14), **30-second fuse**, **one at a time**, **damages Link too**, not while helpless or while someone else holds it.
   **Engine**: none of it (**E4** items, **E3d** detonation / self damage / long life).
6. **Boomerang**: out (clean 8% / late 6%, 70 / 65, by travel) and back (3%, toward Link), x1.2 and a longer flight with a smash input,
   aimed ±26 degrees, decelerating then homing back, **caught** (a catch move), **one at a time** (a failed throw otherwise), falls when
   it clashes. **Engine**: the hits by age are E3a phases (exist) with angle type 1 (exists for projectiles); the path is a projectile
   script; missing **E3d** (per-owner limit, owner catch, falling on a clank).
7. **Bow**: charge from 16, full at 36 charge frames (frame 52), auto-fire after 180 frames, arrow damage **floor(4 + 8c / 60)** and speed
   2.6 → 10, gravity 0.054, 46 frames of flight, **lodging** for 180 frames, **pick-up** and the **double arrow**. **Engine**: the hold is
   a `stall()` / `goto` loop (exists); missing **E35** (a projectile's damage, speed, life and size from the charge), **E3b lodge**, and
   E4 for the pick-up and double arrow.
8. **Hylian Shield**: standing, walking or crouching, a reflectable hit meeting the shield in front is blocked: no damage, a push, hitlag
   with shield SDI (4.0.0); off while holding an item. **Engine**: none (**E14**).

Lesser points that must also be on [R]: the jab's continuable windows and jab 1's 13.0.1 frames (7-8, FAF 24); the far jab box at
angle 180 for jab locks; forward tilt's early frame 15; the dash attack's x1.4 shieldstun (E24, exists); the forward smash's tip and its
**two-hit chain keeping hit 1's charge** (new **E36**); the **Sword Beam at 0%** (E35); up smash's aerial / grounded boxes and +3
hitstun (data); down smash trip 30 (E20 data, never trips); up air's single long box; the throws' kicks; the long, quick, short-ranged
hand grab (no tether).

## How the moves flow (traced)

As in the earlier audits: `fighter::update` → `ground` / `airborne` → `start_ground_attack` / `start_air_attack` / `start_special` →
`begin_attack` → `fighter::attack` each frame → (after every fighter moved) `combat::resolve_hits` → `grab::update` →
`combat::update_projectiles` / `spawn_projectiles` → ledge phase → `combat::check_ko` (`sim-core/src/step.rs`). Points that matter here:

- **Charge** (`fighter::attack`, around line 1918): `charge_at` holds the move frame while **attack** is held and `f.charge <
  rules.charge_frames` (60). A script's `stall()` does the same for any button (around line 1961), also counting `f.charge` and capped
  at 60 per move (a `goto` starts the count again, the Needle Volley pattern). In `resolve_hits` (around line 430) a hit's damage is
  multiplied by `1 + charge x bonus / (100 x 60)` with the move's `charge_bonus` (or the ruleset's 40): with bonus 60 that is the Spin
  Attack's formula exactly [D]. Projectiles never see the charge (only `kb_add` and `size` through `spawn_scaled`).
- **`begin_attack`** (`fighter.rs` line 1698) sets `charge = 0` for every new move, a `next` or a `goto` included: the forward smash's
  hit 2 would lose hit 1's charge (E36).
- **Specials out of a shield** (`fighter::shield`, around line 1210): jump, attack / grab (a shield grab), a roll or a spot dodge; and
  `jump_squat` (line 684) only takes off (an attack makes it a short-hop aerial). So no up special (and no up smash) comes straight out
  of a shield; a jump-cancelled up special is the aerial one after the 3-frame squat (E37).
- **Autolink** (`combat::apply_hit` around line 1049, `fighter::autolink` line 2600): a 367 hit stores the attacker's velocity
  (`link_vel`) and the box's centre plus `link_ahead` (`link_point`); at launch the victim moves at `link_vel + (link_point -
  middle) / 5`, capped by `autolink_max_speed` / `autolink_max_fall`. `link_kind` (hashed) already tells 367 from 366 (the vortex, with
  a hashed frame count `vortex`): a third kind is where a position vector fits (E34).
- **Projectiles** (`combat::spawn_projectiles` line 1169, `update_projectiles` line 1236): one spawn request per fighter per frame; the
  hit loop **skips the owner** (`d == owner`), so a fighter's own projectile can never hit it (no self damage) unless a reflector has made
  someone else the owner; `life` and `age` are **u8** (255 frames at most); there is **no per-owner count**, no owner catch, no command
  from the owner, no projectile-against-projectile contact; a projectile that touches a solid wall is removed (or bursts with E3e
  `on_wall`); an arcing one that lands after its last bounce is removed (nothing lies on the ground); a clank with an attack removes it.
  `ProjectileSpawn::burst` (E3e, sim 87) turns a projectile into a standing piercing burst with its own phases (the explosion shape the
  bomb needs). Phases by age (E3a, eight) give the boomerang's clean / late / returning hits by its age.
- **Projectile scripts** (`scripting::run_projectile`): registers `px py pvx pvy age life owner_x owner_y target_x target_y has_target`,
  functions `set_vel`, `kill`, the math set (with `sin`, `cos`, `sqrt`), 2 vars. Enough for the boomerang's deceleration, turn-back and
  homing toward `owner_x / owner_y` [D]; not enough to know the owner's buttons (detonation) or to be caught.
- **Hits on a shielding fighter** (`combat::block`, line 1099): stun from damage, shield damage, a push `(stun + 1) x 0.09` reference
  units capped at 1.3, hitlag with shield SDI. The Hylian Shield's block can reuse this push and hitlag [E: the reference's push for the
  passive shield is not published, gap 1].
- **Reflectors** (`update_projectiles` around line 1316): every projectile is reflectable here (no `reflectable` flag), which matches
  every projectile in this kit [R]. The passive shield in the reference blocks only reflectable hits: the same set here.
- **Helpless** (`fighter::enter_helpless`): `fall_drift` is one percent applied to the body's air speed **and** acceleration;
  `fall_landing_lag` per move. A helpless fighter cannot start specials, so "cannot detonate while helpless" comes for free [D].
- **No items**: nothing in `sim-core` holds, picks up or throws an object; `GameState` has fighters, ledges, projectiles and script vars.

## Normals: what exists

Every normal is data on existing fields, as for the earlier ports:

| Move | Fields | Notes |
| --- | --- | --- |
| Jab 1-3 | `Jab2`, `Jab3`, `next`, `next_from` (10 / 8), `next_window` | the held repeat rule (14 / 13 without a hit) as the Embers' and Greatsword's jabs; the objects-only twin boxes are dropped (no items until E4) |
| Tilts, dash attack | rows, `angle_type`, `clanks` false on arm / hilt boxes, `shieldstun` 140 on the dash attack | forward tilt's early frame is its own row on 15 |
| Forward smash | `charge_at` (hold frame 9 → move frame 8), hit 1 rows (tip 14 / blade 7), `next` with `next_from` 22 (ref 23) to an `ext` hit 2 | **E36**: hit 2 keeps hit 1's charge; the Sword Beam needs E35 and a script (only at 0%: the `percent` register exists) |
| Up smash | rows with `targets` aerial / grounded, `fixed_kb` + `set_weight`, `sdi` 50, `hitstun_add` 3 (except the late tip's aerial box) | exists |
| Down smash | rows, `trip` 30 (E20) | never trips at 78 / 30 degrees (the reference too) |
| Aerials | `landing_lag`, `autocancel_*`, rows, `targets`, set weight, 367 | the down air's bounce: a script and the `fast_fall` register (E16 addition) |
| Grab, dash, pivot | `HIT_GRAB` rows | the aerial-only thin boxes are the house limit (grabs catch grounded targets) |
| Pummel, throws | `HIT_PUMMEL` / `HIT_THROW` hit-then-throw | the down throw's edge push (2.5 up and away) is not modelled [listed]; interrupted rows and thrower invincibility are the house limits |
| Ledge, get-up | one get-up slot (decision 36) | front and back get-ups share every number |

## Specials: what exists, and the exact shape of each needed extension

### Spin Attack: data, a script, E34 (new), an E11 addition, E37 (new, shared)

- **Grounded** (`UpSpecial`): one group, the four phases as rows (three boxes each: two blade boxes, the 0.8x tip), `clanks` false on
  the first three phases; script: on move frame 2 (ref 3) `stall()` while `special` is held (the cap 60 is the reference's 60);
  `charge_bonus` 60; FAF 77. The sweep's box positions round the body are [E] (gap 3).
- **Aerial** (`ext`, the up special's `air_version`): nine hits (eight spin hits and the finisher) but **eight hit groups a move**.
  The spin hits never overlap in time, so they need not each have their own group: the four front hits in group 0 and the four back
  hits in group 1, both re-armed by a script `rehit()` just before each next hit [D] (the regular `Move::rehit` rhythm does not fit the
  uneven gaps 4 / 3 / 3 / 4 / 5 / 7 frames), and the finisher's three one-frame rows in group 2, never re-armed. Script frame 0:
  `set_vel(kept vx, 0.3)` (2.4 / 8) [E: frame, gap 4]; `air_blocks`: frames 0-46 gravity **0.0066** world a frame (0.096 x 0.55 / 8
  [D]), drift 50, accel from the 0.22 reading [E]; 47-90 drift 87 (0.8 / 0.924) [D, if 0.8 is absolute], accel from the 0.01 reading
  [E]; `ledge_front_from` 14, `ledge_back_from` 33; `landing_lag` 30; `helpless_after`; `fall_landing_lag` 30.
- **E34 (new, small to moderate): position vectors.** The reference's per-hitbox "position vector": the victim is sent toward a point
  relative to the attacker's root (x forward, y up), at the speed that covers the distance in N frames, then its launch speed drops off
  quickly (ssbwiki.com `Autolink_angle`). Shape: `Hitbox::pos_vec: Option<PosVec { x: Fx, y: Fx, frames: u8 }>` (content: `pos_vec {
  x y frames }` on a box, default none); in `apply_hit` a hit with one sets `link = true`, `link_kind = LINK_POSVEC` (new constant),
  `link_point = attacker root + (x x facing, y)`, and a hashed count (reuse `Fighter::vortex` as a general "link frames" count, or a new
  hashed `link_frames: u8`); at launch the victim moves at `(link_point - middle) / frames` for `frames` frames, then keeps a short
  decaying remainder [E: the drop-off rate is not published, gap]. The angle and knockback still set the hitstun. Validation: `frames >
  0`. Tests: a back hit behind the attacker ends `frames` frames later at the point (± a unit); hashed state and content hash cases; no
  other move changes (`base.pfc` equality). Who else would use it: several reference moves with multi-hit sweeps (none in the built
  kits; listed for later).
- **E11 addition (small): `Move::fall_accel`** (percent of the body's air acceleration in the special fall after this move; 0 = the same
  as `fall_drift`, today's behaviour). The spin: `fall_drift` 91 (x0.909), `fall_accel` 70 ((0.01 + 0.025) / 0.05 under the "replace the
  added part" reading [E: gap 4]).
- **E37 (new, shared, owner decision): up special and up smash out of a shield.** The reference lets a shielding fighter start its up
  special or up smash at once (UFD's out-of-shield list counts them from the shield with no jump squat: this body's up special 7 =
  its startup). Shape: in `fighter::shield`, special with the stick up starts the grounded up special, attack with the stick up (or the
  attack stick up) the up smash, before the jump check; or the same during a jump squat that came out of a shield. **It changes every
  fighter** (each gets a faster out-of-shield option): question 4.

### Bow and Arrows: scripts, E35 (new), E3b lodge, then E4 for the pick-up

- **The charge** (`NSpecial`): script: frames to 15 as published, then `stall()` while `special` is held, counting the frames in a var;
  a `goto` to itself every 60 frames up to 180 (the stall cap); on release or at 180 frames `goto(fire)` with the charge kept in a var.
- **The fire** (`ext`): its projectile on frame 1 (the release) with the charge.
- **E35 (new, small): charge-scaled projectiles.** Shape: `ProjectileSpawn::charged: Option<ChargedShot { full: u8, snap: u8,
  damage_to: Fx, speed_to: Fx, life_to: u8, size_to: u16, floor: bool }>`, and the hashed `Projectile::damage: Fx` set at spawn. With
  `c` = the firing fighter's `charge` (or a script-set charge: E29's `set_charge` from the Bracers' design, or a new `spawn` argument),
  `c` becomes `full` once `c >= snap` (the bow's 36 → 60), and the shot's damage, speed, life and size are interpolated from the spec's
  own values to the `_to` values by `c / full` (damage floored if `floor`). Bow: full 60, snap 36, 4 → 12 floored, 2.6 → 10 (/ 8 world),
  life 46. Sword Beam: full 60, no snap, 5 → 13, 2.4 → 3.4, life 32 → 40, size 80 → 110, plus a **drag** (0.01 a frame: a
  `ProjectileArc::drag` field, small, or a projectile script `set_vel`). Tests: damage and speed at 0 / 35 / 36 / 60; the beam only at
  0%; hashed `damage`; content hash.
- **E3b addition (small): lodging.** `ProjectileArc::lodge: u8` (frames; 0 = none): an arcing projectile meeting the ground (or a wall)
  stops there, stops hitting and lives `lodge` more frames (180). Without E4 a lodged arrow just lies there and fades; with E4 it becomes
  an **arrow item** (pick-up, throw, the double arrow). The "9-degree" rule [E: gap 8]. Life past 255 is not needed here (46 + 180).
- **Double arrow** (`ext`, phase 3): when the fighter holds an arrow item (an E4 `holding` register), the release fires two arrows 3
  frames apart (0.9x speed, 2.3 degrees, x0.8 damage floored, BK 25 KS 110) and uses the held arrow up.

### Boomerang: a projectile script, E3a phases (exist), E3d (new pieces)

- **The throw** (`SideSpecial`): `flick` (exists) chooses the smash version: `goto` an `ext` slot with its own projectile (3.1, life
  167, its phase ages 37 / 87). The projectile on frame 27, angled by the stick (±26 degrees, `spawn` with `sin` / `cos`).
- **The path** (projectile script): decelerate 0.047 to a minimum 0.3, then turn back and home on `owner_x / owner_y` at up to 3 with a
  0.75-degree-a-frame turn [E: gap 6]. Its hits are E3a phases by age (clean / late / returning) with angle type 1 (exists).
- **E3d (new, moderate): projectile lifecycle**, shared by the boomerang and the bomb:
  - `ProjectileSpawn::limit: u8`: at most this many live projectiles of this move per owner (1); a fighter register `owned` (live
    projectiles of the current move) so the move can branch to the **failed throw** (`ext`, FAF 60 / 46).
  - `ProjectileSpawn::catch: Option<u8>`: once the projectile is past a set age (its return), touching its owner removes it and, if the
    owner is free (standing, walking, running, airborne, not attacking or defending), starts that move (the catch, `ext`, FAF 10).
  - `ProjectileSpawn::on_clank: Option<Fall { life, vx, vy, gravity, max_fall }>`: clanking with an attack (today: removal) turns it into a
    harmless falling object for `life` frames (the boomerang's 60 / -0.15 / 1.25 / 0.09 / 1.4).
  - `Projectile::life` / `age` to **u16** (the bomb's 1800 frames), or a separate long life; `hits_owner: bool` (self damage: the owner
    is not skipped by that projectile's hits); a fighter function `detonate()` (E16) that makes the caller's live projectiles (and, with
    E4, items) of a given kind burst at once through E3e's `start_burst` (the explosion is the burst's phases: 7%, 52, BK 60 KS 115,
    radius 14, 2 frames, `hits_owner`).
  - Hashed: the new `Projectile` fields; content: the new spec fields with defaults (no move changes); `docs/CONTENT.md` lines; the
    whitelist entries with their tests in `scripted_moves.rs`.

### Remote Bomb: E4 (new, broad), E3d, scripts

- **The pull** (`DownSpecial`): if the fighter already owns a live bomb, `goto(detonate)`; if it holds any item, nothing (the reference
  refuses the pull); else on frame 17 an item **bomb** appears in its hand (E4).
- **The detonation** (`ext`, FAF 40): on frame 12 `detonate()` (E3d). Not while helpless (free); not while another fighter holds it
  (an E4 check on the item's holder).
- **The bomb item** (E4): gravity 0.07, fall 2.3, friction 0.01, max horizontal 4, bounces (floor / wall / ceiling / fighters), a 1%
  type-1 hit when thrown, bounces off shields, HP 50 and a hurtbox of 4 (launched at 1-3.2 by 1-15%), fuse 1800 (flash the last 45).

### E4: held items (new, broad; explained to the owner before it is built)

The roster's E4 sketch, made exact for this kit. Items are a **new kind of thing in `GameState`**, separate from projectiles (a
projectile is a hit in flight; an item is an object that can lie, be held, be thrown and be hit):

- **State** (hashed, each field in `hash_into` with a checksum case): `GameState::items: [Item; MAX_ITEMS]` (8 [E]); `Item { active,
  kind (the spec: weapon + move + item index), owner (who made it), holder (fighter or NONE), thrower (credit for a thrown hit), pos,
  vel, facing, age: u16, life: u16, hp: Fx, state (held / flying / resting / lodged / fading), bounces, hit_mask, hit_age }`;
  `Fighter::holding: i8` (the item held, NONE).
- **Content** (`ItemSpec` on the move that makes it, defaults leave every weapon as today): collision half-lengths, pick-up box, gravity,
  fall, friction, horizontal cap, bounce multipliers (floor / wall / ceiling / fighter, the +1 up), throw speed multiplier, the item
  damage formula (base, base modifier -0.4 / 0, speed factor 0.8 / 0), its thrown hitbox, what a shield does (vanish / bounce), HP and
  hurtbox size, fuse and flash, explosion (an E3e burst), **`owner_only: bool`** (the undecided rule, below). Per body: item throw
  frames (this body: release 7 / 9 / 11 / 6 / 4 / 5; totals 24 / 24 / 23 / 19 / 29 / 23, the 13.0.1 rule for everyone), toss strength
  `1 + (weight - 100) / 300` (from the weight, no new field), pick-up reach (`item_range_*`).
- **Actions** (in `fighter.rs`, universal like the grab): pick up (attack or grab standing or walking over an item in reach; in the air,
  an air catch); throw with grab (or attack, for a thrown-type item) in four directions and the dash throw, a smash throw on a flick;
  the grounded drop and the aerial drop (Z-drop) on grab with no direction; the throw speeds and angles of `Item_throw`
  (`REFERENCE_DATA.md`). While holding: the Hylian Shield is off (E14), specials that refuse a held item refuse (the bomb pull), the
  double arrow is offered.
- **Physics and hits** (`combat`): a thrown item hits fighters with its formula damage (and bounces off them), is blocked or bounces off
  shields, can be hit (HP, launch), clanks [E], bounces on terrain, burns its fuse, explodes as a burst.
- **The undecided rule (decision 3)**: with `owner_only` **false** (the reference) anyone can pick up a lodged arrow or the bomb, throw
  it back, and hold the bomb to stop its owner detonating it; with `owner_only` **true** only the maker can pick it up (others still hit
  it about and are hit by it). One flag, one code path; the choice is per item spec, so it can differ between the arrow and the bomb.
- **Cost and reach**: the largest new system since the gauge (state, content, fighter actions, combat, a view for the Godot side via a
  read-only bridge accessor, `pftool net-fuzz` at zero mismatches, a `SCHEMA_VERSION` bump with a migration for the item blocks). It is
  built general (decision 3 says an item system is coming anyway), so stage items can reuse it later.

### E14: the Hylian Shield (new, small)

`FighterParams::passive_shield: Option<PassiveShield { x: Fx, y: Fx, half_height: Fx, radius: Fx, crouch_y: Fx }>` (default none: every
other body unchanged). In `update_projectiles`' hit loop, before `apply_hit` on fighter `d`: if `d`'s body has one, `d` is in `Idle`,
`Walk` or `Crouch` [R] (not holding an item, E4), the projectile is moving toward `d`'s front (`pvx x facing < 0`, or a standing shot
on its front side) and its box touches the shield capsule (in front at chest height; lower in a crouch) [E: gap 2], then the shot is
**blocked**: no damage, the fighter gets the shield's hitlag (shield SDI on) and the shield push from `combat::block`'s formula [E: gap
1] with no shield damage or stun, and the projectile ends as against a shield (removed; a piercing one goes on) [E]. A shot reaching the
head or feet past the capsule hits as today. Tests: a blaster shot from the front at chest height is blocked standing, walking and
crouching (crouch: a shot at head height hits); from behind it hits; dashing it hits; a push and hitlag happen; hashed: no new state (the
existing hitlag and velocity carry it); content hash case.

### E36: a jab-style follow-up keeps the charge (new, small)

`Move::keep_charge: bool` on the follow-up (default false): `begin_attack` from a `next` into a move with it keeps `f.charge`. The
forward smash's hit 2 (`ChargedSmashDmgSSBU` on its rows, [R]). Test: a fully charged hit 1 then hit 2 deals 13 x 1.4.

### E16 additions

Registers `fast_fall` (the down air bounce), `owned` (E3d), `holding` (E4: the kind held, 0 none); function `detonate()` (E3d). Each:
a whitelist entry, a test, the doc line.

## Slots (`ext`, 16 a weapon since E17)

| Use | Slots |
| --- | --- |
| Forward smash hit 2 | 1 |
| Down air after the bounce | 1 |
| Aerial Spin Attack | 1 |
| Bow fire; double-arrow fire | 2 |
| Boomerang smash throw; failed throw; catch | 3 |
| Remote Bomb detonation | 1 |
| **Total** | **9 of 16** |

Seven spare (an aerial bow if gap 7 shows its own frames, an aerial detonation or pull if gap 10 does). Item throws are universal
actions (E4), not weapon slots.

## Extension summary

| Id | Status | This port needs | Shape |
| --- | --- | --- | --- |
| E3a | built (sim 76) | the boomerang's hits by age | as built |
| E3b | built (sim 60); **addition** | arrows lodging | `ProjectileArc::lodge` |
| E3c | built | no-clank boxes | as built |
| **E3d** | **designed here** (roster sketch) | boomerang and bomb lifecycle | `limit`, `catch`, `on_clank`, u16 life, `hits_owner`, `detonate()`, `owned` |
| E3e | built (sim 87) | the bomb's explosion | as built (a burst started by `detonate`) |
| **E4** | **designed here** (broad) | the bomb, arrow pick-up, double arrow | `GameState::items`, `ItemSpec` with `owner_only`, `Fighter::holding`, item actions |
| E11 | built; **addition** | the spin's special fall | `fall_accel` |
| **E14** | **designed here** (small) | the Hylian Shield | `FighterParams::passive_shield` |
| E16 | built; **additions** | bounce, bombs, items | `fast_fall`, `owned`, `holding`, `detonate()` |
| E17 | built | 9 `ext` slots | as built |
| E20 | built | down smash's trip data | as built |
| E24 | built | dash attack shieldstun | as built |
| E26 | built | angle types incl. type 1 on projectiles | as built |
| E29 | designed (the Bracers) | optional: `set_charge` to hand the bow's charge to the fire move | as designed |
| **E34** | **new** | the aerial spin's back hits | `Hitbox::pos_vec { x, y, frames }`, `LINK_POSVEC` |
| **E35** | **new, small** | arrows and the Sword Beam | `ProjectileSpawn::charged`, hashed `Projectile::damage`, `ProjectileArc::drag` |
| **E36** | **new, small** | forward smash hit 2's charge | `Move::keep_charge` |
| **E37** | **new, shared (owner decision)** | out-of-shield Spin Attack | up special / up smash from a shield |

Not needed: E1, E2 (no wind box anywhere in the 13.0.1 kit), E5-E10c, E12, E13, E15 (no wall jump), E19, E21, E22, E22b, E25, E27, E28,
E30-E33.

Every new field defaults to today's behaviour, is hashed (content through `Weapon::hash_into` / `FighterParams::hash_into`, state
through `GameState::hash_into` with a checksum case each), appears in `docs/CONTENT.md`, and gets a test in its topic file; no existing
move or body uses them, so the other fighters stay unchanged (check `base.pfc` equality and the existing kit tests), **except E37**,
which changes every fighter by design. One `SIM_VERSION` bump per phase.

Shared limits this port exposes but does not fix (listed): the grabs' aerial-only boxes; the throws' interrupted rows and thrower
invincibility; one get-up attack slot (decision 36); the jab's objects-only boxes (until E4); the down throw's edge push; the bomb
exploding on fire (no fire effect); no projectile-against-projectile contact.

## Suggested phases

1. **Body, every sword normal and Spin Attack**: `FighterParams::wanderer` (the parameter file's numbers, initial dash 12, heavy landing
   3, wall-tech jump only), the jab chain, tilts, dash attack, the smashes (E36 for hit 2), the five aerials (the forward air's drag as
   the reference, the down air's bounce with `fast_fall`), grabs, pummel, throws, ledge and get-up attacks; **both Spin Attacks** with
   **E34** and the E11 `fall_accel`; **E14** (a body trait, testable with the existing blasters and needles); **E37** if the owner
   approves. Class registered (`CLASSES`, `roster.gd`), content re-export. Neutral, side and down specials as placeholders.
2. **Bow and Boomerang, and the Sword Beam**: E35, the E3b lodge (arrows lie and fade), E3d's `limit`, `catch`, `on_clank`, `owned`.
3. **Items and Remote Bomb**: E4 (after the owner's answer on `owner_only`), E3d's long life, `hits_owner`, `detonate()`; the arrow
   pick-up and the double arrow. Then the re-check of every iconic move in the hitbox view.

## Questions for the owner

1. **Items (decision 3)**: build E4 as above, general for later stage items? And for this kit: **can other fighters pick up his lodged
   arrows and his bomb** (the reference: yes; they can throw them back and hold the bomb so he cannot set it off), or **only he** (his
   tools stay his; others can still knock the bomb about)? The answer can differ for arrows and bombs (`owner_only` per item).
2. **The forward air (decision 4)**: the reference's hit 1 already drags airborne targets with its outer two boxes (367). Keep it
   exactly (proposed), or make the inner aerial box and / or the grounded boxes drag too (an owner-chosen change, recorded as such)?
3. **E34** (position vectors) for the aerial spin's back hits: build it as shaped (proposed: without it the spin drops targets hit
   behind)?
4. **E37** (up special and up smash straight out of a shield): it changes every fighter (each gains the reference's out-of-shield up
   special and up smash). Build it, or keep shields as they are and list the grounded spin out of shield as a limit?
5. **Names**: Buckler / wanderer / Wanderer and the blurb, or others?
6. **Phases**: Spin Attack and the Hylian Shield in phase 1, items last (proposed)?
7. **Footage** for the gaps (14 in `REFERENCE_DATA.md`); the most important for 1:1: the grounded spin's sweep (gap 3), the aerial
   spin's height and drift (gaps 4-5), the Hylian Shield's push and coverage (gaps 1-2), the boomerang's path (gap 6), and the jab 2 /
   down smash FAFs (conflicts 2, 4).
