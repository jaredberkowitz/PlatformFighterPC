# Body proportion study (2026-10-18)

The owner: the fighters read cheap and childishly proportioned beside the reference cast (the PEAK-style T-pose sheet the owner shared,
and the crossover fighter's plumber). Measured from the reference sheet (1920x1080, the yellow chef without its hat, the orange scout)
and from our bare body's front turnaround (`tests/turnaround_shot.gd --bare`), as shares of the standing height:

| Measure | Reference | Ours before | Ours after the rebuild |
|---|---|---|---|
| Head height | 33 to 37% | 43% (scaled-up head) | 34% |
| Head width / height | 0.41 (a wide oval, up to 1.25 times as wide as tall) | 0.43 (a ball) | 0.41 (widened 1.2 across) |
| Torso, neck to crotch | about 32% | 30% | 31% |
| Legs, crotch to floor | about 35% | 27% | 36% |
| Hips / head width | 0.84 | 0.67 | 0.85 |
| Leg thickness / head width | 0.33 | 0.33 | 0.35 |
| Gap between the legs / head width | about 0.2 | 0.09 | 0.2 |

What read as childish was toddler proportions: a huge head on short, close-set legs and a narrow body. The reference has a big, wide head
on a sturdy block of a torso and longer legs set well apart.

The rebuild (art/blender/make_cage_body.py, about 2.34 tall): the crotch raised from 0.62 to 0.80, the torso a wide block from 0.88 to
1.52 (as wide at the hips as across the shoulders), a wide neck, the head modelled at the study's height and widened in the game
(`HEAD_SIZE` in scripts/fighter_view.gd), the arms set out and down to the new shoulders, the legs longer and further apart. Same
skeleton and joint names, refitted (fit_body.py), every clip rebuilt (make_rigged_blob.py), so every move still works.

Next, from the same reference: the clothes as their own layered pieces (sleeved shirts and collars standing off the body, flared shorts,
the sash), where the reference's detail lives; and hands with fingers.
