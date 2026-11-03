# Animation review 2 (2026-11-02): the shared animations against the references

The owner, 2026-10-30: an animations pass at maximum effort, the re-usable animations (idle, locomotion, air, defence, ledge, reactions)
as close to the reference as possible. This review scores every clip in that scope on the same three axes and the same 1 to 5 scale as
review 1 (5 = it could sit beside the reference fighter's motion): **P** poses, **F** fluidity (spacing, arcs, hand-offs, per-frame
jumps), **W** weight (timing per phase, the centre of mass, squash and settle). "Before" is the round 8 hand-over build (8r, c1c0270b);
"now" is core4 (sim 147, round 9 groups 1-7). Review 1's scores were given per family and category, not per clip, so they are not a
clean baseline; the "before" column here re-scores 8r per clip, from the same captures.

**Method.** Each clip decoded from reference footage at 60 frames a second, frame by frame, for at least two reference fighters where
footage exists (compilations of every fighter's idle, spot dodge, air dodge, run, dash dance, crouch, ledge, dizzy and reel; training
grids for jumps and runs; the owner's match videos), and ours measured the same way with `PF_ANIM_TRACE` and
`tools/art/review/clip_report.py`. The footage and the side-by-sides stay on F: (they show the reference game's characters); the images
in this repository show ours only (`docs/screenshots/anim/r9_*.jpg`). The family representatives: Longsword (sword, against the
swordsman), Claws (brawler, against the wolf), Needles (quick, against the ninja), Clutch (heavy, against the dinosaur), Embers (caster,
against the plumber), Pockets and Popgun (tool, against the villager). Our body has a big head and short arms: the aim is the same read,
not the same joint angles.

**The stance mock (Decided: owner picked C, 2026-10-30):** `docs/art/anim/review2/mock_stance_width.jpg`. The feet set apart along the
stage at the reference's width, the knees at each family's own bend. It is the default, and every grounded clip stands on its feet.

## 1. Scores

| Clip | Before P / F / W | Before | Now P / F / W | Now | Reference |
|---|---|---|---|---|---|
| Idle | 2 / 3 / 2 | 2.3 | 4 / 4 / 4 | 4.0 | yes |
| Long idle (fidget) | 3 / 3 / 2 | 2.7 | 3 / 4 / 3 | 3.3 | yes |
| Walk | 3 / 2 / 2 | 2.3 | 3 / 4 / 3 | 3.3 | yes |
| Run | 2 / 2 / 2 | 2.0 | 4 / 4 / 4 | 4.0 | yes |
| Dash | 3 / 3 / 2 | 2.7 | 4 / 3 / 4 | 3.7 | yes |
| Dash dance | 3 / 3 / 3 | 3.0 | 4 / 4 / 3 | 3.7 | yes |
| Run stop, skid | 3 / 3 / 3 | 3.0 | 3 / 4 / 3 | 3.3 | no, by craft |
| Standing turn | 3 / 3 / 3 | 3.0 | 3 / 4 / 3 | 3.3 | yes |
| Crouch (enter, hold, exit) | 2 / 3 / 3 | 2.7 | 4 / 4 / 3 | 3.7 | yes |
| Jump squat, take-off | 3 / 3 / 3 | 3.0 | 4 / 4 / 3 | 3.7 | yes |
| Full and short hop (rise, apex, fall) | 3 / 3 / 2 | 2.7 | 4 / 4 / 3 | 3.7 | yes |
| Midair jump | 3 / 4 / 3 | 3.3 | 3 / 4 / 3 | 3.3 | yes |
| Fast fall | 3 / 3 / 3 | 3.0 | 3 / 4 / 3 | 3.3 | yes |
| Landing (light, heavy, aerial lag) | 3 / 3 / 3 | 3.0 | 4 / 4 / 4 | 4.0 | yes |
| Platform drop, walk-off | 3 / 3 / 3 | 3.0 | 3 / 3 / 3 | 3.0 | no, by craft |
| Shield (raise, hold, drop) | 3 / 3 / 3 | 3.0 | 3 / 4 / 3 | 3.3 | partly |
| Forward roll | 2 / 3 / 3 | 2.7 | 4 / 4 / 4 | 4.0 | one fighter |
| Back roll | 2 / 3 / 3 | 2.7 | 4 / 4 / 3 | 3.7 | one fighter |
| Spot dodge | 2 / 3 / 3 | 2.7 | 4 / 4 / 3 | 3.7 | yes |
| Air dodge | 3 / 3 / 3 | 3.0 | 4 / 4 / 3 | 3.7 | yes |
| Ledge grab and hang | 2 / 2 / 2 | 2.0 | 3 / 4 / 3 | 3.3 | yes |
| Ledge climb | 3 / 3 / 3 | 3.0 | 3 / 4 / 3 | 3.3 | yes |
| Ledge roll | 2 / 3 / 3 | 2.7 | 4 / 4 / 3 | 3.7 | two fighters |
| Ledge jump, ledge attack's body | 3 / 3 / 3 | 3.0 | 3 / 3 / 3 | 3.0 | yes |
| Flinch (light hit, standing) | 3 / 3 / 3 | 3.0 | 3 / 4 / 4 | 3.7 | no close footage |
| Heavy hit, upward hit | 3 / 3 / 3 | 3.0 | 3 / 4 / 3 | 3.3 | no close footage |
| Launch, tumble | 3 / 4 / 4 | 3.7 | 4 / 4 / 4 | 4.0 | match camera only |
| Knockdown, get-up | 3 / 3 / 3 | 3.0 | 4 / 4 / 3 | 3.7 | partly |
| Tech in place, tech roll, missed tech | 3 / 3 / 3 | 3.0 | 3 / 3 / 3 | 3.0 | no, by craft |
| Shield break, dizzy | 2 / 3 / 3 | 2.7 | 2 / 3 / 3 | 2.7 | yes |
| Helpless (special fall) | 3 / 3 / 3 | 3.0 | 3 / 3 / 3 | 3.0 | yes |
| Grabbed | 2 / 3 / 3 | 2.7 | 3 / 3 / 3 | 3.0 | partly |
| Respawn | 3 / 3 / 3 | 3.0 | 3 / 3 / 3 | 3.0 | no, by craft |
| Victory poses | 3 / 3 / 3 | 3.0 | 3 / 3 / 3 | 3.0 | yes, not reworked |

**Mean (34 clips): before 2.84, now 3.40** at core4; **3.45** with group 8 (the reactions, 2026-11-03: the flinch, heavy hit, launch and
knockdown rows; `docs/ANIMATION.md` group 8). The tech, shield break, dizzy, helpless, grabbed, respawn and victory poses were not
reworked in round 9 and keep their scores.

## 2. Measured, ours against the references

**Idle** (share of the height; hips up and down / along the stage / feet apart / period in frames). Reference: swordsman 3.7% / 6.5% /
0.26 / about 60; wolf 3.6% / 1.8% / 0.39 / about 75; ninja 4.5% / 5.5% / 0.48 / 46; dinosaur 10% / 3% / 0.19 / 30; plumber 10% /
7.6% / 0.25 / 27; villager 5.5% / about 3% / 0.27 / about 60. Ours, 8r to now: sword 0.9% / 0 / 0.10 / 96 to 2.7% / 6.2% / 0.16 / 59;
brawler 0.8% / 0 / 0.09 / 96 to 3.6% / 2.6% / 0.24 / 80; quick 1.0% / 0 / 0.14 / 96 to 3.5% / 5.3% / 0.27 / 44; heavy 0.5% / 0 / 0.07 /
96 to 8.8% / 2.6% / 0.13 / 30; caster 0.2% / 0 / 0.02 / 47 to 9.5% / 6.1% / 0.19 / 27; tool 0.1% / 0 / 0.00 / 48 to 4.5% / 2.6% / 0.18 /
59. **Still differs:** the feet stay short of the references' width (about two thirds): on these short legs a wider step reads as a
lunge.

**Run.** Cadence (a full cycle): references 26 (swordsman, wolf), 25 (plumber), 22 (ninja), 30 (dinosaur), 27 (villager); ours was 14
at the same speed, now 26, 21, 27, 31, 22, 22 by family. Hips to neck from upright: references about 50, 60, 68, 52, 40, 18; ours 8r
32 (sword), now sword 46, brawler 56, quick 52, caster 48, heavy 37, tool 15. Knees at the stride's widest, front / rear: 8r 75 / 59, now
17-26 / 14-23 (references about 15-25). Feet apart at the widest: 0.53 to 0.66 of the height (reference about 0.72). Quick family
flight: both feet off the floor 23% of the cycle, the lower foot 0.024H up on average and 0.035H at most; the ninja about 30%, 0.04H,
0.055H. **Still differs:** our hips to neck is 0.28 of the height against 0.30-0.38, so at the same angle the neck is less far ahead;
the quick run leans less than the ninja's (kept lower after the art director's note that it read as flying).

**Walk.** 8r cycled every 9.5 frames at full tilt; now about 17 (references 26-30 at walking speed). **Still differs:** a fast walk's
foot leaves the floor between steps.

**Jump and landing.** References: the take-off stretched long, legs trailing; each family's own arms; a curl or a drawn knee at the
top; the fall opening; the landing bottoming out by its third frame at about a fifth of the height. Ours now: `jump`, `fall_start`,
`fall`, `jumpsquat` and `land` per family; the landing bottoms out by frame 4 at about 0.2H and comes up by the frames since touching
down (quick 9, heavy 16).

**Forward and back roll.** The plumber's rolls (the only training-mode roll footage): the head at the floor upside down at 38% of the
roll, round again by 65%, a long low crawl up. Ours: the hips' lowest 0.26H (reference about 0.2H), the head at the floor, the spin
over 12-66% of the roll. Sim 147: the forward roll ends turned round, as the reference's does (the reference's scripts turn it on frame
20, 19 for four bodies); drawn as a turn through the camera in the stand-up, the largest bone jump around the roll's end 0.54-0.74 to
0.11 (Needles 0.36, Dagger 0.25). Stale dodges played by their real length: frames held at the end pose 0-9 to 0 (rolls), 8 to 0
(spot dodges).

**Spot dodge.** References snap into the dodge by the third frame and hold, each its own (tall and opened, a deep squat, a low lunge,
flat over the knees, a duck, a bow); none steps back in depth. Ours now per family, by frame 3, the step into depth 0.7 to 0.2.

**Ledge.** The ledge roll (frame-data clips of two fighters): up onto the edge by about a fifth, then the somersault low along the
floor. Ours now does that (it spun all the way up from the hang). The hang: from the far hand, drawn in to the wall, legs dangling (both
arms were posed straight out behind, stopped by the shoulder's limit).

**Crouch.** References fold right over bent knees, hips about a fifth of the height up; ours were at a third, now about a fifth.

**Hand-offs.** Every clip change eases over sim frames from the last pose (`_ease_handoff`): before it, the jump's take-off moved a
hand 1.0 units in a frame, shield to roll 0.44, out of a run 0.52. The carried weapon's hand comes back over 6 frames.

**Across the moves (the shared body under every attack).** Face cover, every gallery both facings, 8r / core2 / now: the old circle 110
/ 116 / 62; the drawn eyes or mouth covered (new measure, `PF_FACE_COVER=features`) 64 / not measured / 12. A strike's chest flipped
180 degrees for 5-15 frames in the unarmed classes' jabs, smashes and specials (95 frames over seven galleries), fixed. Striking hands
hidden by the body 2,470 / about 2,140 / 1,511 (1,354 before the character work's rebuilt rig, whose 1-3 mm joint shifts moved a
borderline count, mostly the Buckler's down air, visible by eye). Elbows reading reversed on screen (`PF_BEND` `scr`): core1 4,540
(elbow 3,180, knee 1,360), group 5 3,243 (elbow 1,941, knee 1,302). Attack timing, mean error in frames over 68 moves
(follow-through / hold / recovery / wind-up): 8r 2.29 / 1.56 / 1.81 / 1.15, now 2.34 / 1.62 / 2.03 / 1.13 (the last 0.09 of hold and
0.06 of recovery from the rebuilt rig flipping one frame between hold and recovery in 14 moves; to be refitted with the attack-weight
pass).

## 3. What still differs, and why

- **Width and reach:** short legs and arms. Stances, strides and the run's spread stop at about two thirds of the references' spans,
  because wider read as lunges on this body.
- **The run's lean:** the torso is shorter than the references', so the neck is less far ahead at the same angle; the head rule is not
  the cap (measured: tipping the head 40 instead of 18 would add 0.045 of the height).
- **Rolls:** the hips stay about 0.06H higher than the plumber's at the bottom of the roll; the head reaches the floor.
- **Reactions (group 8):** the launch and the tumble turn in the plane of the stage with the face toward the camera all the way round,
  the damage fall flails, a knockdown lies on its back. Still differs: the slam onto the floor is the body tipping over from upright in
  about 6 frames, where the references arrive nearly flat from the flight; the flinch keeps the pose review's poses (a compact recoil).
- **Elbows:** 1,941 elbow-frames still read reversed on screen at group 5 (the knockdowns and the overhead moves to judge by eye).

## 4. No adequate reference: judged by craft

Run stop, skid and run turn; rolls of any fighter but the plumber; tech in place, tech rolls and the missed tech; knockdown and get-up
close up; light and heavy hits taken standing, close up; platform drop; walk-off; respawn. The footage that would settle them: a
training-mode recording at a close camera, for each family's reference fighter, of a run stopped and turned, both rolls, a tech in
place and both ways, a missed tech and each get-up, a light and a heavy hit taken standing and launched, a drop through a platform, a
walk off an edge, and a respawn.

## 5. Next, in the order set on 2026-11-02

1. Hit reactions (flinch, heavy, upward, launch, tumble, knockdown, get-up, tech, missed tech, dizzy, helpless, grabbed, respawn),
   judged at the match camera: limbs clear of the body.
2. Weight in attacks (wind-up, impact, settle against the references at real speed; refit the end shapes) and the remaining elbows
   (the knockdowns, the overhead moves).
3. The rest of shield and ledge (shield raise and drop, ledge grab, climb, jump).
4. Victory poses.
