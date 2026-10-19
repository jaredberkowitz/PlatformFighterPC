# Footage comparison: movement against the reference (19 October 2026)

The owner asked for our movement to be checked against reference footage, "the way they move should line up". This is the first
pass, on the two training-mode combo videos with a measuring grid (the Wolf and Marth "bread and butter combos" clips,
`docs/movesets` references; downloaded with the owner's permission and deleted after measuring).

## Method

Frames at the clips' full 30 per second (one video frame = two game frames), scaled to 960x540, read by a small tracker
(scratch script): the floor line, the grid's scale in every frame (the camera zooms and pans: the period of the grid's unit lines),
the red vertical line (world x = 0) for horizontal positions, and the fighter's outline (its lowest point, its centre). Our side is
the simulation itself, frame by frame, with the same inputs. Units are the reference's (our world unit = 8 of them); one grid unit
reads as about 9.4-10 reference units (the opponent stands about 1.8 grid units tall, and both fighters' short hops agree with
about 9.4).

## Results

| What | Reference (footage) | Ours | Verdict |
|---|---|---|---|
| Jump squat | 3 frames (seen as 1-2 video frames of crouch) | 3 | Matches |
| Claws (Wolf) short hop, height after lift-off (2-frame steps) | 3.9, 7.3, 9.8, 11.2, 13.5, 15.2, 16.5, 16.7 | 3.7, 7.0, 9.7, 11.8, 13.5, 14.6, 15.3, 15.4 | Same shape and timing; footage peak 8% higher (scale or pose) |
| Claws short hop, frames to the peak | about 16 | 15 | Matches (within one video frame) |
| Longsword (Marth) short hop, height after lift-off | 1.2, 3.4, 5.6, 8.2, 10.9, 12.9, 14.4, 15.8, 16.7 | 1.5, 4.4, 6.9, 9.1, 11.0, 12.6, 14.0, 15.0, 15.7 | Same shape and timing; footage peak 6% higher |
| Longsword short hop, frames to the peak | about 18 | 20 | Matches (within one video frame) |
| Claws air drift | builds about 0.09 a frame to about 1.2-1.3 | 0.09 a frame to 1.28 (published) | Consistent |

Both fighters' footage peaks are 6-8% above ours by the same share, which points at the grid's scale (about 9.4 reference units
per grid unit, not 10) rather than at the jumps; the outline's lowest point also moves with the pose (legs tucked at the top of a
jump), about ±0.1-0.2 grid units.

## Not measurable from these clips

- **The full hop.** Every jump in both clips is a short hop into an aerial (often fast-fallen), so there is no clean full hop. Ours
  opens with the reference's published "initial height" (`hop_burst_*`: 55% of the height in the first 4 frames, then the arc); a
  clean full hop on video would confirm it. This is the check that matters most.
- **Dash and run speed.** The only dash from a standstill is a dash-dance (out, back, jump), so the speeds are rough; no clean run.
- **Rolls, spot dodges, air dodges, wavedashes, fast falls.** Not shown cleanly (or not at all).

## What would finish it

A short training-mode recording, the camera fixed, at 60 frames a second if possible, for each of Wolf and Marth: three full hops,
three short hops, a full hop with the stick held forward and one held back out of a run, a dash from standing held into a run for
two seconds, a forward and a back roll, a spot dodge and a fast-fallen full hop. The same tracker and probe then give a frame-by-frame
overlay of each.
