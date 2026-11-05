# Animation review 3 (2026-11-04): weight, readability and accuracy at the match camera

The owner, 2026-11-04: another full pass, "trying to get them to feel a bit less weightless, more readable, and more accurate to the
reference's animations". This review sets the order of that pass (round 10). It builds on round 9 (`docs/ANIMATION.md`, groups 1-23;
review 2, `docs/art/anim/ANIMATION_REVIEW_2.md`, mean 3.48), and judges the installed core10 (sim 149, 538daa3b).

**Method.** Every class's gallery in both facings at the match camera's scale (a fighter about a sixth of the screen), filmed at 60 frames a
second and read frame by frame; four computer matches at the real match camera (Longsword against Claws, Needles against Clutch, Embers
against Pockets, Staff against Gauntlets); every class's gallery traced (`PF_ANIM_TRACE`, `PF_MOTION_TRACE`) and measured: the timing
against the references (68 moves), which limb strikes and where its root lies at the hit (new: `F:\anim_core\t3\cross.py`), and how far the
torso travels after the last hit along the way it launched (new: `bodyft.py`). The references are the same as review 2's (each family's
reference fighter: the swordsman, the wolf, the ninja, the dinosaur, the plumber, the villager; the frame-data site's hitbox clips; the
owner's gifs and match videos). Reference footage and side-by-sides stay on F:; the images here show ours only.

**Axes, 1 to 5** (5 = it could sit beside the reference's motion): **W**eight (anticipation, a kinetic chain, impact, follow-through and
settle, planted feet, knees absorbing, heavier classes heavier); **R**eadability (a clear silhouette and line of action at match size, the
striking limb visible and clear of the torso at the hit, key poses held long enough to register); **A**ccuracy (key poses and timing
against the reference).

## 1. Scores (core10)

| Animation | W | R | A | Mean | What holds it back |
|---|---|---|---|---|---|
| Idle | 4 | 4 | 4 | 4.0 | the feet stay about two thirds of the references' width |
| Walk | 3 | 3 | 3 | 3.0 | a fast walk's foot leaves the floor between steps; the slowest tilt plays at a quarter speed |
| Dash, run | 4 | 4 | 4 | 4.0 | |
| Turn, dash dance | 3 | 4 | 3 | 3.3 | |
| Jump squat, jump, fall | 3 | 4 | 4 | 3.7 | |
| Landing | 4 | 4 | 4 | 4.0 | |
| Shield | 3 | 4 | 3 | 3.3 | |
| Rolls, spot dodge | 4 | 4 | 4 | 4.0 | |
| Air dodge | 3 | 4 | 4 | 3.7 | |
| Jabs | 3 | 3 | 3 | 3.0 | the torso stops on the strike (cause 2) |
| Tilts | 3 | 3 | 3 | 3.0 | cause 2; the free arm hangs (cause 5) |
| Smashes | 3 | 3 | 3 | 3.0 | cause 2; no settle (cause 3); heavies strike as lightly as the rest (cause 4) |
| Aerials (neutral, forward, up, down) | 3 | 3 | 3 | 3.0 | cause 2 |
| Back airs | 3 | 2 | 2 | 2.3 | the kicking leg crosses the other at the hit (cause 1) |
| Specials | 3 | 3 | 3 | 3.0 | cause 2 |
| Grab, pummel, throws | 3 | 3 | 3 | 3.0 | |
| Light hurt | 4 | 4 | 4 | 4.0 | one set of flinch poses for every family (cause 8) |
| Heavy hurt, launch, tumble | 4 | 4 | 4 | 4.0 | at the match camera the hit effects cover the first ~10 frames (effects, not animation) |
| Knockdown, tech, get-ups | 3 | 3 | 3 | 3.0 | sim 150's 26-frame tech in place and 29/21-frame stand-ups: first knots, not measured (cause 6) |
| Ledge (grab, hang, climb, roll, jump, attack) | 3 | 3 | 3 | 3.0 | the hang is still (cause 9) |
| Shield break, dizzy | 3 | 3 | 2 | 2.7 | not reworked (cause 10) |
| Victory poses | 3 | 3 | 3 | 3.0 | not reworked (cause 10) |

**Mean (22 rows): 3.30.** By family, attacks only: sword 3.0, brawler 3.0, quick 3.0, heavy 2.7, caster 3.0, tool 3.0, staff 3.0. The shared
clips (idle to air dodge) sit at 3.0-4.0 after round 9; every attack row sits at 3.0 or under, and they share causes.

## 2. Measured at core10

- **Timing** (mean frames off the references, 68 moves; follow-through / hold / recovery / wind-up): 2.40 / 1.40 / 1.65 / 1.13. By family:
  sword 3.1 / 1.1 / 1.4 / 1.1 (the up air's somersault, the tucked curl decision), brawler 1.2 / 0.6 / 1.2 / 2.1, quick 2.9 / 3.2 / 2.9 / 0.3
  (the up air's measure error), heavy 3.8 / 1.2 / 2.0 / 1.3 (the up smash's tip back, the owner's rule), caster 2.4 / 1.5 / 1.2 / 0.7, tool
  0.9 / 0.9 / 1.1 / 1.2. Most references measure a follow-through of 1-2 frames by this rule too: the follow-through error is a few
  outliers, not the shared stop (cause 2), which the rule does not see.
- **Eyes or mouth covered** 7 frames; **striking hands hidden** 1,306; **limbs reading reversed** 2,693; **end-lag pops** of 0.5 units or
  more 7.
- **Limbs crossing at the hit** (new): the striking limb whose root (hip or shoulder) lies more than 0.15 units on the far side of the other
  limb's root from the strike: 22 move-facings over every gallery. Every kick back air (the Claws, Embers, Staff, Bracers, Clutch, Wanderer
  and Scamp) and the Greatsword's back air, both facings, the hip 0.22-0.31 units on the wrong side; the Needles' down smash's back kick;
  the Bracers' down special; the Nodachi's up tilt (its arms, 0.16-0.17).
- **The torso after the hit** (new): how far the hips, chest and head go on past their strike pose, along the way they launched, in the
  10 frames after the last hit, as a share of the launch: median by family 0.13-0.37. The striking limb or blade carries on (its travel
  in the 8 frames after the hit 0.2-0.5 of its launch's); the torso stops with the strike.

## 3. The ten shared causes, ranked by gain for cost

1. **The kicking leg crosses the other at the hit (the owner's note of 2026-11-04).** *Readability, back airs.* `fighter_view.gd`
   `_aim_leg` kicks behind with the far leg ("a kick behind uses the far leg"), written when the near leg was the front one. Since stance C
   (round 9) the near foot is the back one, and for a hit behind the body swings round through the camera (`BACK_ATTACK_TURN`, 115
   degrees), which brings the far hip to the front; the clips (`kick_bair`, `attack_bair`) tuck the near leg forward and trail the far one
   back, so the legs scissor and the kick leaves from behind the tucked leg. **Fix:** the limb on the strike's side strikes: a kick behind
   uses the near leg (behind in the stance and after the turn, and in front of the body in depth, so in plain view), the clips' tuck goes on
   the far leg; both facings (the `@m` clips swap with it). Low cost (one function, two clip specs, a rig rebuild).
   Image: `docs/screenshots/anim/r10_audit_back_kicks.jpg` (each class's back air around its hit, facing right then left).
2. **The torso stops on the strike and never turns through.** *Weight, every attack.* The shared six-pose clip keys the strike twice
   (`make_rigged_blob.py` `clip()`: frames 32 and 36 the same pose), so the body comes to rest on the hit and, through the keys' ease,
   slows into it; its follow-through key (40) is the strike plus 2 degrees of bend, 1.3 times the twist and 0.03 of sink
   (`_follow_spec`), played over one frame and a two-frame ease (`follow_k`); then the hold creeps back. The references, and the owner's
   guide, turn the torso on past the target with the weight onto the lead leg while the limb carries on. **Fix:** in the shared builder,
   the strike flows into a real follow-through: the second strike key moves on, the follow key carries the chest's twist and bend past the
   strike by a share of the launch's swing, the hips over the lead foot, the head lagging; each family's carry span; then the hold and
   recovery refitted. Medium cost (one builder, a rig rebuild, the refit). Three parameter tries in round 9 moved only the limb or spread
   the old travel thinner; this changes the keys every attack shares.
3. **No settle into the stance.** *Weight.* The recovery eases straight home (`home_k`), then the hand-off eases into the idle: the knees
   never absorb the body's arrival. **Fix:** a settle at the end of a grounded recovery, the hips dipping and coming back up over a few
   frames, by the move's strength. Low cost (view only).
4. **Heavy fighters strike like light ones.** *Weight.* One coil, snap and carry for every class (`COIL_AT`, `SNAP_AT`, `CARRY_FRAMES`).
   **Fix:** after 2 and 3, a per-family scale: the heavy family holds its coil longer, carries further and settles deeper; the quick
   family snaps and settles shorter. Low cost.
5. **The free arm hangs out of the strike.** *Weight and readability.* The attack clips clamp the free arm (`FREE_ARM_FWD` 10-80 degrees,
   `FREE_ARM_OUT` 24), so in most strikes it neither reaches toward the target in the wind-up nor swings back against the strike at the
   impact (the owner's bar: the plumber's forward air, the off hand aiming while the fist cocks back). **Fix:** the free arm opposite the
   strike in the shared archetypes' keys, within the face and no-arm-behind-the-body rules. Medium cost.
6. **Tech in place and stand-ups on sim 150.** *Accuracy.* The tech's knots (slapped down by frame 4, up by 10, in the stance by 16, then
   the idle) are the engine session's first timing, not measured; the stand-ups' 29 and 21 frames are not checked against the references.
   **Fix:** retime both against the reference's tech in place and get-ups. Low cost.
7. **The light flinch is one set of poses.** *Accuracy.* Every family flinches alike, by the hit's strength only (round 9 group 20).
   **Fix:** a family scale on the recoil (heavies give less, quick fighters snap back further). Low to medium cost.
8. **The ledge hang is still.** *Weight and accuracy.* The references hang from one hand and stay alive, the free arm swinging out and the
   legs kicking every two seconds or so. **Fix:** mock two or three options for the owner (side-on to the wall a swing goes toward the camera
   or into the wall). Medium cost, an owner pick.
9. **Striking hands hidden by the torso** (1,306 frames): the far fist in two-handed and alternating strikes and the big head. **Fix:** per
   layer, by move; medium to high cost, after the above.
10. **Not reworked since round 8:** the shield break and dizzy, and the victory poses. **Fix:** against the references. Medium cost; less
    seen than everything above.

## 4. Order of round 10

1. Cause 1 (back kicks, and any strike behind), with the before/after at the match camera.
2. Cause 2 (the torso's follow-through), with cause 3 (the settle) and the refit; then an install.
3. Cause 6 (tech in place, stand-ups) on sim 151.
4. Causes 4 and 5 (heavy families, the free arm), then 7, 8 (mock), 9, 10.

Limits held throughout: hold at most 1.56, recovery at most 1.65, eyes or mouth covered at most 7, hidden hands, reversed limbs and pops
not rising.

## 5. Round 10 so far

- **Group 1 (cause 1, done):** back kicks with the near leg, the back one; crossings at the hit past 0.15 units 22 to 0. Back airs
  readability 2 to 4, accuracy 2 to 3 (mean 2.3 to 3.3).
- **Group 2 (found in these traces, done):** the wind-up's crouch was the clip's full chamber whatever the startup: 53 of 134 grounded
  moves dropped the hips or head 0.25 units or more in one frame of their wind-up (16 of 0.4 or more). Each wind-up now goes as deep as its
  frames allow: 32 (5). Jabs and tilts weight 3 to 4 where the jolt was the weak point (the jab, tilt and smash rows' weight 3 to 3.3).
- **Cause 2 corrected:** tried and reverted (`docs/ANIMATION.md`): the drawn torso does carry on through the hit, by the view's lunge and
  lean layers, for about five frames; the clip's held strike key did not show. The stop the audit read in the clip is not on screen.
- **Group 3 (found at r10a's install, done):** the Scamp's popgun aimed at the foe through the draw and the shot (it pointed at the
  floor, and the shot swung the gun over the head); the charge's glow, tremble and squint only while the charge builds (they ran through
  the whole shot, and every charged smash's swing). The Scamp's eyes or mouth covered, hidden hands and reversed limbs as at r10a; every
  other class's poses identical. Left: the misfire (extra move 4, the gun going off in the hands) still plays the fallback swing.
- **Group 4 (done):** the recovery refitted: timing 2.35 / 1.43 / 1.65 / 1.18 (the recovery back to the 1.65 limit). The Clutch's up
  air's new end-lag pop judged real (the near fist crosses the head's middle in a frame) and left for cause 9 (`docs/ANIMATION.md`).
- **Group 5 (cause 3, done):** the settle: the hips dip and rise with the knees taking the weight as a grounded move comes home (a jab
  barely, a smash clearly, feet planted), refitted: timing 2.35 / 1.50 / 1.63 / 1.18, pops 8, eyes or mouth covered 7, hidden hands 1,310,
  reversed limbs 2,841. The smashes' and tilts' weight 3 to 3.5 where the glide home was the weak point. Group 2 checked at real speed
  against core10: the jolt is gone, nothing lost.
- **Group 6 (the coordinator's item 1, done): the grip pass.** The Settler's spade turns its face to the camera at the dig; its turnips
  stand up through the up air's hits (they pointed back at the chest); its slingshot points along its shots, at where the pellet starts
  (the forward air's pointed at the floor, the back air's at the camera); the Wanderer's shield is carried low while the bow is out (it
  covered the bow); the Scamp's barrels show at both sides of its waist in flight (they were hidden behind the body). Eyes or mouth
  covered and hidden hands unchanged; no body bone moves. Left: the turnips beside the head rather than over it (an owner pick), the
  down smash's spade held in the ground between the digs, the back air's tucked kick pose (`docs/ANIMATION.md`).
- **Group 7 (cause 6, done): the tech in place and the stand-ups retimed** against the reference's, filmed at 60 frames a second. The tech
  stays low through 19 and stands at 23 without the over-stretched pop (it stood tall by 10). The stand-ups sit up by 5, reach their
  feet by 10 and stand at 24 (8, 15 and 29 before), then breathe in the fighter's own stance. Knockdown, tech and get-ups: accuracy 3 to
  4 (mean 3.0 to 3.3). Left: the reference's roll back over the shoulders in its tech (no clip for it).
