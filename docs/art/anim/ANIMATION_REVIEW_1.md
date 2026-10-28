# Animation review 1 (2026-10-27): poses, fluidity and weight against the references

The owner: keep refining the animations and build any missing animation sets, reviewed the way the stages are (a numbered review, every
item scored against the references, the weakest ranked with a fix, mocks for design choices, the owner picks, ship, repeat). Every item
is scored on three axes and overall (5 = it could sit beside the reference fighter's motion):

- **Poses:** key poses next to the reference's at matching frames: silhouette and line of action, hip, chest and head orientation, limb
  placement and extension, stance width.
- **Fluidity:** spacing and arcs, overlap and follow-through, hand-offs between clips, measured as per-frame jumps from the trace
  (`PF_HEAD_TRACE`, `tools/art/review/head_leg_report.py`): a head turning over 9 degrees, a knee bend changing over 30 degrees, a foot
  moving over 0.3 units on the ground (0.6 in the air) in one frame.
- **Weight:** timing per phase against the reference (`tools/art/review/timing_report.py`: hold and recovery after the last active frame,
  the wind-up's half-way frame `w50`), measured on ours from `PF_MOTION_TRACE` and on the reference's hitbox animations the same way in
  the browser (silhouette distance from the last frame and change per frame); centre-of-mass drop and settle; squash on landing; heavy
  against light.

**Evidence** (scored on the build at 9a18a2b8, the round 7b install; F:\anim_r8 holds every capture):
- each family's representative in the move gallery at 60 frames a second, both facings, every third frame: Longsword (sword, against the
  swordsman), Claws (brawler, against the wolf), Needles (quick, against the ninja), Clutch (heavy, against the dinosaur), Embers (caster,
  against the plumber), Settler (tool, against the villager);
- the locomotion run (walk, dash, run, stop, hop, landing, shield, roll) per family, and the shared demos (launch, ledge, grab, shield
  break, roll, walk-off, items);
- the computer-against-computer matches of round 7 (eight, 34,400 frames) for live play at the match camera;
- the owner's combo video at match speed (the swordsman's runs, landings and shields).

**Sheets** (`docs/art/anim/review1/`): one per family (`sword.jpg`, `brawler.jpg`, `quick.jpg`, `heavy.jpg`, `caster.jpg`, `tool.jpg`:
locomotion, ground attacks, aerials both facings, specials) and `shared_states.jpg` (launch, ledge, grab, shield break, roll, walk-off).
**The side-by-side strips with the reference** (ours above, the reference's hitbox animation below, the same frames 1-34 of the forward
tilt, forward smash, forward air and up air) are kept on F:, not in the repository, because they hold the reference game's character
models (CLAUDE.md: no Nintendo assets in the repository): `F:\anim_r8\sheets\<family>_vs_ref.jpg`, and for the weight fixes
`F:\anim_r8\sheets\r8_g2_with_reference.jpg`. The reference's timing numbers are in this document.

## 1. Missing sets: every state and moment, and what plays

States are sim-core's `FighterState` (35); moments are the presentation ones the game has. The game has **no** entrance, taunt, teeter,
crawl or star KO (no state or moment), so none are listed as missing.

| State or moment | Before this round | Kind | Now |
|---|---|---|---|
| Idle (and the long idle's fidget) | each family's own idle and fidget | dedicated per family | same |
| Walk, Run | each family's own | dedicated per family | same |
| Dash | one shared dash | shared | same |
| Turn (standing) | the idle while the body yaws round (no clip) | code only | same; scored under Turnaround |
| Turn out of a run (skid), run stop | `skid`, `run_stop` | shared | same |
| Dash dance | the dash restarted off the foot that is down | shared | same |
| Crouch | `crouch` | shared | same |
| JumpSquat, Landing, WaveLand | `jumpsquat`, `land`; landing lag out of an aerial is deeper and pitched (code) | shared | same |
| Airborne (rise, tuck, open, fall), midair jump, platform drop | `jump`, `fall_start`, `fall`, `air_jump` (flip), `platform_drop` | shared | same |
| **Helpless (special fall)** | the jump's arc by velocity: a fighter still in control | **stand-in** | **`helpless`**: limp, hunched, arms loosely up, legs hanging (group 1) |
| AirDodge, Roll, SpotDodge | `air_dodge`, `roll`, `spot_dodge` by progress | shared | same |
| Shield, ShieldDrop | `shield` | shared | same |
| ShieldRelease | the idle | stand-in (acceptable: a 1-2 frame lag) | same |
| **ShieldBreak** | the hit reaction's flail, held | **stand-in** | **`dizzy`** once grounded: a slow dazed sway (group 1) |
| LedgeHang, LedgeGetUp (climb, roll), LedgeAttack, LedgeJump | `ledge`, `ledge_climb`, `roll`, climb then `attack_low`, the flip | shared | same |
| Grabbing, pummel | `grab` | shared | same |
| Throws (four) | one `throw` clip for forward, back, up and down | **shared, one for four** | same (ranked item 5) |
| Grabbed | `hurt` | stand-in (reads as held) | same |
| Hitstun (flinch, heavy, upward, launch, tumble) | `hurt`, `hurt_heavy`, `hurt_up`, `tumble`, recoil by strength, the launch pose | dedicated | same (tumble settles out since round 7) |
| Knockdown, trip, GetUp, tech in place, missed tech | `knockdown`, `crouch` (trip), `getup`, `tech`, knockdown | dedicated | same |
| Get-up attack, trip attack | `attack_swing` | shared | same |
| **WallTech** | the shield | **stand-in** | **the tech's slap, by progress** (group 1) |
| Rebound (clank) | `hurt` | stand-in | same |
| Respawn platform | the idle | shared | same |
| **Buried** | the idle, under the ground | **stand-in** | **`buried`**: both arms pushing in turn (group 1; little of it shows, see the list) |
| Carrying | the up air held at its top | shared | same |
| Encased | hidden in the egg | none needed | same |
| **ItemPickup** | the idle | **missing** | **`item_pick`**: a dip and a reach to the floor (group 1) |
| **ItemThrow** | the idle, or the fall | **missing** | **`item_throw`**: the free hand drawn back and whipped through (group 1; for armed classes the free arm is mostly hidden behind the body, see the list) |
| **Tether** | the fall | **missing** | **the ledge hang's arms-up hold** (group 1) |
| Victory (results) | `victory_a/b/c` by class | dedicated | same |
| KO | the launch, smoke and blast (effects) | dedicated | same |

**Specials, per class** (dedicated, shared archetype, or stand-in for the reference's motion):

| Class (family) | Dedicated | Shared archetype that fits | Stand-in (the reference moves otherwise) |
|---|---|---|---|
| Longsword (sword) | | up special (rising cut) | neutral special (a charged thrust plays the smash), side special (a four-hit combo plays one lunge), down special (a parry stance plays the low strike) |
| Claws (brawler) | neutral special (blaster), up special (corkscrew, code) | side special (a dash slash) | down special (a reflector gesture plays a kick) |
| Needles (quick) | | down special (flip kick) | neutral special (aiming and throwing needles: stands), side special (the chain: stands), up special (the vanish: a low strike) |
| Embers (caster) | | all four (fireball, cape, rising punch, pump) | |
| Gauntlets (brawler) | neutral special (the big punch), down special (the dropping kick) | side special (lunge) | up special (a leaping grab plays the rising strike) |
| Staff (sword stance) | | | all four (a spinning shield, a cast fireball, a vanish, a summon play the smash, lunge, rising strike, low strike) |
| Greatsword (sword stance) | down special (counter crouch) | side special (lunge) | neutral special (a plunge into the ground), up special (a leap and plunge) |
| Bracers (brawler) | neutral (charged punch), up (spinning arms), down (hand slap) | | side special (headbutt plays the lunge) |
| Clutch (heavy) | neutral (tongue headbutt), down (ground pound) | up special (throw) | side special (rolling in an egg plays the lunge) |
| Buckler (sword stance) | up special (spin) | | neutral (bow), side (boomerang), down (pulling a bomb) |
| Settler (tool) | all four (pocket, rocket, balloons, planting) | | |
| Popgun (tool) | all four (pass 31) | | |
| Dagger (quick) | | | its specials play the blaster pose |
| Nodachi (sword stance) | | | all four play the shared archetypes |

None of the stand-ins needs sim data: each is a clip and a mapping. The one missing piece of data is which item throw is being thrown
(up, down, forward, dash): the bridge does not expose `Fighter::item_throw`, so `item_throw` plays one overhand throw for all. A getter in
`godot-bridge` (not the sim) would let it pick; noted for the moveset porting agent.

## 2. Scorecard

P = poses, F = fluidity, W = weight, O = overall (their mean). **Bold**: the weakest axis. "After" is the score once this round's groups
1-3 shipped (only items that moved).

| # | Item | P | F | W | O | Weakest | After |
|---|---|---|---|---|---|---|---|
| 1 | Sword locomotion | 3 | 4 | 3 | 3.3 | poses, weight | |
| 2 | Sword ground attacks | 3 | 3 | 3 | 3.0 | poses | |
| 3 | Sword aerials | 3 | 3 | **2** | 2.7 | weight | W 3, O 3.0 |
| 4 | Sword specials | **2** | 3 | 3 | 2.7 | poses | |
| 5 | Sword defence | 3 | 4 | 3 | 3.3 | poses | |
| 6 | Brawler locomotion | 3 | 4 | 3 | 3.3 | poses | |
| 7 | Brawler ground attacks | **2** | 3 | 4 | 3.0 | poses | |
| 8 | Brawler aerials | 3 | 3 | 4 | 3.3 | poses, fluidity | |
| 9 | Brawler specials | **2** | 3 | 3 | 2.7 | poses | |
| 10 | Brawler defence | 3 | 4 | 3 | 3.3 | poses | |
| 11 | Quick locomotion | 3 | 3 | 3 | 3.0 | fluidity | |
| 12 | Quick ground attacks | **2** | 3 | 3 | 2.7 | poses | |
| 13 | Quick aerials | 3 | 3 | **2** | 2.7 | weight | W 3, O 3.0 |
| 14 | Quick specials | **2** | 3 | **2** | 2.3 | poses, weight | |
| 15 | Quick defence | 3 | 4 | 3 | 3.3 | poses | |
| 16 | Heavy locomotion | 3 | 3 | 4 | 3.3 | poses, fluidity | |
| 17 | Heavy ground attacks | **2** | 3 | 3 | 2.7 | poses | |
| 18 | Heavy aerials | **2** | 3 | 3 | 2.7 | poses | |
| 19 | Heavy specials | **2** | 3 | 3 | 2.7 | poses | P 3, O 3.0 |
| 20 | Heavy defence | 3 | 4 | 3 | 3.3 | poses | |
| 21 | Caster locomotion | 3 | 4 | 3 | 3.3 | poses | |
| 22 | Caster ground attacks | 3 | 3 | 3 | 3.0 | poses, fluidity | |
| 23 | Caster aerials | 3 | 3 | 3 | 3.0 | weight | W 4, O 3.3 |
| 24 | Caster specials | 3 | 3 | 3 | 3.0 | poses | |
| 25 | Caster defence | 3 | 4 | 3 | 3.3 | poses | |
| 26 | Tool locomotion | 3 | 4 | 3 | 3.3 | poses | |
| 27 | Tool ground attacks | 3 | 3 | 3 | 3.0 | weight | |
| 28 | Tool aerials | 3 | 3 | 3 | 3.0 | poses | |
| 29 | Tool specials | 4 | 3 | 3 | 3.3 | fluidity, weight | |
| 30 | Tool defence | 3 | 4 | 3 | 3.3 | poses | |
| 31 | Reactions (hurt, launch, tumble, knockdown, tech, get-up) | 3 | 4 | 4 | 3.7 | poses | |
| 32 | Ledge (grab, hang, climb, roll, attack, jump) | 3 | 3 | 3 | 3.0 | fluidity | |
| 33 | Grabs and throws | **2** | 3 | 3 | 2.7 | poses | |
| 34 | Special states (special fall, items, shield break, tether, wall tech, buried) | **1** | 3 | **2** | 2.0 | poses | P 2, W 3, O 2.7 |
| 35 | Turnaround (standing turn, dash dance, the head's swing) | 3 | 3 | 3 | 3.0 | poses, fluidity | |
| 36 | The Clutch's down special knee | **2** | 3 | 3 | 2.7 | poses | P 4, O 3.3 |
| 37 | Stance width and knee bend | **2** | 4 | 3 | 3.0 | poses | (mock, awaiting the owner) |

**Mean: 3.00 before this round's groups, 3.07 after.** By axis: poses 2.68 (2.78 after), fluidity 3.32, weight 3.00 (3.11 after).
**Poses is the weakest axis in 26 of 37 items.** Nothing scores 5; the reactions are the best item (3.7).

### What the scores rest on

**Poses (against the reference strips at the same frames):**
- Our grounded strikes stand nearly upright where the references drive the whole body along the strike. The swordsman's forward smash
  ends nearly horizontal over a deep front knee (frames 13-28); ours keeps the chest up over a bent lead knee. The wolf's forward tilt
  and the dinosaur's forward smash are low and horizontal; ours are upright with the arm or leg doing the work. The ninja's forward tilt
  is a tall, straight high kick on one leg; ours is a lower knee-up kick. This is the line of action (ranked item 4, a mock).
- Aerials: the references curl the whole body round the swing (forward air) or flip (up air). Ours stay upright; the flips were turned
  down on purpose (the owner, 2026-10-13: "our own solutions where the reference flips the whole body"), so they score 3, not lower.
- Specials: 27 of the 56 (14 classes, 4 each) play a shared archetype that moves differently from the reference (the table above), and the Needles'
  needle volley and chain stand still.
- Stance: the idle bends the lead knee 90-100 degrees with the feet about 1.0 apart in depth; the references bend 20-40 degrees and
  stand about hip-wide (ranked item 7, a mock).
- Throws: one clip for all four throws, where the references' back throw turns round and the up throw lifts.

**Fluidity (per 100 frames: head jolts, knee pops, foot pops):**

| Family | Moves (gallery, both facings) | Locomotion |
|---|---|---|
| Sword | 5.8 | 3.0 |
| Brawler | 9.7 | 3.3 |
| Quick | 9.2 | 5.9 |
| Heavy | 9.1 | 5.3 |
| Caster | 10.4 | 2.8 |
| Tool | 8.1 | 1.8 |

Shared demos: launch 1.9, ledge 5.8, grab 3.2, shield break 1.2, roll 1.2, items 2.1, walk-off 10.6. Most of the moves' pops are the
strike's snap (the whole wind-up-to-hit turn in one frame, the owner's snap/sweep A/B, undecided) and the gallery's air placement; the
follow-through after the last hit reads 0-2 frames everywhere where the references carry on 1-10 (the swordsman's up air 10, the
ninja's forward smash 8). Round 7 removed the head's jolts at hand-offs (now 7-10 per family, all at deliberate spins or turnarounds).

**Weight (hold / recovery frames after the last hit, ours against the reference):**

| Family (reference) | Mean error: hold | recovery | wind-up w50 | Worst moves before |
|---|---|---|---|---|
| Sword (swordsman) | 5.1 frames | 3.8 | 0.9 | forward air 19/5 against 11/15, up air 18/13 against 8/17, back air 9/13 against 17/5, jab 13/3 against 6/5 |
| Brawler (wolf) | 2.6 | 2.6 | 2.2 | neutral air 10/3 against 4/9, forward smash wind-up w50 6 against 18 |
| Quick (ninja) | 6.5 | 6.5 | 0.5 | up air 33/3 against 2/26, neutral air 12/4 against 2/14, down smash 19/8 against 6/22 |
| Heavy (dinosaur) | 4.5 | 2.7 | 1.2 | forward air 5/9 against 15/6, up smash 21/6 against 5/5 |
| Caster (plumber) | 4.1 | 3.2 | 0.6 | forward smash 18/6 against 11/15, up air 16/4 against 7/10, forward air 23/7 against 34/2 |
| Tool (villager) | 3.3 | 2.1 | 1.1 | down tilt 13/7 against 20/4, jab 13/2 against 6/9 |

The heavy family lands deeper and settles slower than the light ones (round 2 pass 33, round 4 pass 45), and the hit's freeze holds every
layer (round 4 pass 48); the landings squash on impact. What weight lacks is the follow-through's carry and, until this round, the right
hold and float home per family.

## 3. The weakest items, ranked, with their fixes

| Rank | Item | Weakest axis | Fix | Status |
|---|---|---|---|---|
| 1 | Special states (34) | poses | The states that played another state's clip get their own: special fall, item pick-up and throw, shield-break daze, buried, tether, wall tech | **Shipped, group 1.** Still weak: on this body (short arms, big head) raised arms hide behind the head, so the special fall and the armed classes' item throw read only partly; buried shows only the head |
| 2 | Quick specials (14) | poses, weight | The Needles' own needle volley (aim and throw), chain swing and vanish clips | Open (next) |
| 3 | Quick and sword aerials' weight (13, 3) | weight | Per-family end-lag shapes fitted to the reference's hold and recovery | **Shipped, group 2** (and the heavy's and caster's) |
| 4 | Ground attacks' line of action (7, 12, 17, 2) | poses | Lean the whole body into level strikes (the whole-body lean and a chest bend toward the hit) | **Mock** (`review1/mock_line_of_action.jpg`), awaiting the owner |
| 5 | Grabs and throws (33) | poses | A clip per throw (forward push, back lift-and-toss, up lift, down slam), mapped by the throw's name | **Tried and reverted.** Built and captured all four (`--demo=grab --throw=fwd/back/up/down`): at the match camera the release is under the hit burst and the raised arms hide behind the head, so before and after barely differ (`F:nim_r8\sheets
8_throws.jpg`). Next: carry the thrown fighter's motion in the thrower's whole body (a step and turn) rather than the arms |
| 6 | Clutch down special knee (36) | poses | Knees side by side when both thighs are forward | **Shipped, group 3** (every tuck too) |
| 7 | Stance width and knee bend (37) | poses | A shallower, narrower stance | **Mock** (`review1/mock_stance.jpg`), awaiting the owner |
| 8 | Sword, brawler and Staff/Buckler/Nodachi specials (4, 9) | poses | Own clips for the counter, the charged thrust, the reflector, the bow, boomerang and bomb pull | Open |
| 9 | Turnaround (35) | poses, fluidity | A short pivot clip for the standing turn, the head leading | Open |

## 4. What shipped this round (round 8 groups 1-3)

Logged in `docs/ANIMATION.md` section 8 (round 8). Images in `docs/screenshots/anim/`; the side-by-sides with the reference on F:.

- **Group 1, the missing sets** (`r8_g1_missing_sets.jpg`, commit c4a1e593): `helpless`, `item_pick`, `item_throw`, `dizzy`, `buried`
  clips in the rig; the tether hangs; the wall tech slaps. `PF_R8_OFF=sets` plays them as before. Face cover 0% in each
  (`face_cover_probe.gd`).
- **Group 2, weight** (`r8_g2_end_lag_weight.jpg`, the reference alongside in `F:\anim_r8\sheets\r8_g2_with_reference.jpg`, commit
  c4a1e593): `CLASS_END_SHAPES` for the Longsword, Needles, Clutch and Embers. Mean hold / recovery error in frames: sword 5.1/3.8 to
  4.1/2.9; quick 6.5/6.5 to 4.3/4.2; heavy 4.5/2.7 to 2.4/2.8; caster 4.1/3.2 to 2.5/1.7.
- **Group 3, knees in tucks** (`r8_g3_tuck_knees.jpg`, commit 00dedc07): the Clutch's down special knees -79/69 degrees off the hips to
  4/2; its forward air 15 to 8.

## 5. Mocks (both decided: the owner picked C for each, 2026-10-28)

1. **Decided (owner picked C, 2026-10-28; built in round 8 group 6).** **Line of action in ground strikes** (`review1/mock_line_of_action.jpg`): the Longsword's, Claws' and Clutch's forward tilt and
   forward smash, frames 7-25.
   - A: today (the whole body leans 35% of the strike's swing, about 4 degrees for a level strike).
   - B: the full lean (about 13 degrees) and the chest bent 10 more toward the hit.
   - C: a deep lean (about 20 degrees) and the chest bent 24 more: nearest the references' horizontal lunges; the head comes down and
     the face limiter lifts it.
2. **Decided (owner picked C, 2026-10-28; built in round 8 group 6).** **Stance width and knee bend** (`review1/mock_stance.jpg`): the stage idle at the match camera, two pairs of fighters.
   - A: today (the low stance of passes 1 and 18: the lead knee 90-100 degrees, the feet about 1.0 apart in depth).
   - B: knees at 72% of today's and the feet 60% as wide.
   - C: knees at half (about 45-50 degrees) and the feet 35% as wide: nearest the references.

## 6. Scores after round 8's groups 4-6 (2026-10-28)

| # | Item | P | F | W | O | Was |
|---|---|---|---|---|---|---|
| 1 | Sword locomotion (the low carry, stance C) | 4 | 4 | 3 | 3.7 | 3.3 |
| 4 | Sword specials (the counter, group 5) | 3 | 3 | 3 | 3.0 | 2.7 |
| 7 | Brawler ground attacks (line of action C) | 3 | 3 | 4 | 3.3 | 3.0 |
| 9 | Brawler specials (the reflector, group 5) | 3 | 3 | 3 | 3.0 | 2.7 |
| 14 | Quick specials (the Needles' own, group 4) | 3 | 3 | 3 | 3.0 | 2.3 |
| 17 | Heavy ground attacks (line of action C) | 3 | 3 | 3 | 3.0 | 2.7 |
| 37 | Stance width and knee bend (stance C) | 4 | 4 | 3 | 3.7 | 3.0 |

**Mean 3.15** (3.07 after groups 1-3, 3.00 before the round). By axis: poses 3.00, fluidity 3.32, weight 3.14. Weapons held out of the
way (group 6) also lift the sword-stance classes' locomotion and defence reads; their items stay 3 on the family representative.

## 7. After round 8's groups 8-9 (2026-10-28)

| # | Item | P | F | W | O | Was |
|---|---|---|---|---|---|---|
| 35 | Turnaround (the standing pivot, the head leading) | 4 | 4 | 3 | 3.7 | 3.0 |

Group 8 (a raised sword hand kept in front of the face) is a fix to face cover and the hilt's read rather than an item's axis; it holds
the sword-stance items where they were. **Mean 3.17** (poses 3.03, fluidity 3.35, weight 3.14).


## 8. After round 8's group 10 (2026-10-28)

| # | Item | P | F | W | O | Was |
|---|---|---|---|---|---|---|
| 4 | Sword specials (the counter's raised blade, the Wanderer's bow, boomerang and bomb) | 4 | 3 | 3 | 3.3 | 3.0 |

**Mean 3.18** (poses 3.05, fluidity 3.35, weight 3.14).
