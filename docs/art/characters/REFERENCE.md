# Character reference table (measured, 2026-10-19)

Proportions of the owner's reference cast, measured from the images, against our base body measured the same way. The owner chose the
three-character T-pose lineup (`docs/art/reference_character_art/peak-character-models.jpeg`) as the proportion target.

**Method.** `godot/tests/silhouette.gd` prints, row by row, the left and right edge of each figure against the flat background. Every
measure is given in **head widths** (the head's widest span), so images of any size compare directly. Ours is the default look 0
without its hat, in the T-pose (`tests/turnaround_shot.gd --tpose --hat=0`), front panel. Both cameras are a little above the figure
and slightly perspective; the numbers are good to about ±3%.

```
Godot --headless --path godot --script res://tests/silhouette.gd -- --img=docs/art/reference_character_art/peak-character-models.jpeg --bands=60:660,662:1230,1232:1860 --step=10 --tol=0.13
```

## Widths (head widths)

| Measure | Where (reference rows, px) | Chef | Crab | Cap | **Reference** | Teal (2nd ref) | **Ours** | Gap |
|---|---|---|---|---|---|---|---|---|
| Head width (px) | widest row, 440–450 | 280 | (hood) | 279 | **1.00** | 84 px | **1.00** (180 px) | |
| Torso just under the arms | first clean row under the arms, 640 | 0.72 | 0.63 | 0.68 | **0.68** | 0.60 | **0.82** | ours 20% wider |
| Waist (narrowest) | 690–700 | 0.58 | 0.59 | 0.59 | **0.59** | 0.56 | **0.74** | ours 25% wider |
| Shorts' hem | 790 | 0.83 | 0.79 | 0.82 | **0.81** | 0.79 | **0.82** | even |
| Legs, outer span | 830–860 | 0.77 | 0.75 | 0.77 | **0.76** | 0.77 | **0.74** | even |
| Shoulders with sleeves | 570 | 1.24 | 1.18 | 1.26 | **1.23** | — | **1.23** | even |
| Arm span, T-pose | 600 | 2.04 | 2.00 | 2.05 | **2.03** | 2.12 | **2.27** | ours 12% longer |
| Arm thickness (with hand) | 580–630 | 0.19 | 0.19 | 0.19 | **0.19** | — | **0.21** | close |
| Feet, outer span | 970 | 0.97 | (shadow) | 0.95 | **0.96** | — | **0.84** | ours 13% narrower |

**Head ÷ torso under the arms: reference 1.48 (teal 1.65), ours 1.22.** (An earlier reading by eye, "about 1.1", counted the puffed
sleeves as torso; measured, the reference's torso is narrower than ours, not wider.)

## Heights (head widths, from the chin, where the head meets the collar)

| Measure | Reference (cap figure) | Ours | Gap |
|---|---|---|---|
| Chin to sole | **1.62** | **1.42** | ours 12% shorter |
| Chin to the arms' line | 0.21 | 0.18 | close |
| Chin to the shorts' hem | 0.89 | 0.83 | ours 7% shorter |
| Shorts' hem to sole (leg showing) | **0.73** | **0.59** | ours 19% shorter (our hem is just above the knee) |
| Whole figure, head top to sole | ≈2.5 (hat hides the top; teal 2.55) | 2.2 | |

## What the gaps mean for the body
1. **Torso narrower**: about 0.82 → 0.68 head widths under the arms, the waist 0.74 → 0.59. The shirt and shorts are cut from the body,
   so they follow.
2. **Body taller below the head**: chin to sole 1.42 → 1.62, mostly the legs (about a fifth longer), the torso a little.
3. **Arms shorter**: the span 2.27 → 2.03; most of it comes back from the narrower shoulders, the rest from the arm itself.
4. **Feet a little broader**: 0.84 → 0.96.
5. Unchanged: the head, the shorts' and legs' widths, the shoulder line with its sleeves, the arm's thickness.

## After the refit (2026-10-19, `make_cage_body.py`, measured the same way)

| Measure (head widths) | Reference | Before | **After** |
|---|---|---|---|
| Torso just under the arms | 0.68 | 0.82 | **0.68** |
| Waist | 0.59 | 0.74 | **0.63** |
| Arm span, T-pose | 2.03 | 2.27 | **1.99** |
| Shorts' hem | 0.81 | 0.82 | **0.83** |
| Legs, outer span | 0.76 | 0.74 | **0.74** |
| Feet, outer span | 0.96 | 0.84 | **0.93** |
| Chin to sole | 1.62 | 1.42 | **1.60** |
| Head ÷ torso under the arms | 1.48 | 1.22 | **1.46** |

The body is then scaled as a whole back to its old height (2.24, `BODY_HEIGHT`): the fighters' hurtboxes and collision box are content,
sized to that height, and the refit's targets are shares of the head. So in the world the body is as tall as before and the head about
9% smaller; against the body, the proportions are the lineup's.

