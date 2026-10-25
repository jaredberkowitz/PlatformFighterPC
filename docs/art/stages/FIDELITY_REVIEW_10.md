# Stage fidelity review 10 (2026-10-25, after review 9 and the contact-shade round): where we stand

The owner: "Do another fidelity review of all five stages (review 10) to see where we stand and what still blocks 5/5." The method is
the same: eight categories scored 1 to 5 (5 = a still could sit beside the references), review 3's regions, the same references, the
fighters' readability measured. The judge is the hand that built the work, so read the scores as "the gaps this review can see".

**Evidence:** art/stage-refine 7bccc74 (review 9's six items, the deeper contact shade on every stage, the canyon's far clouds), at the
match camera and wide (`review10/fidelity10_sheet.jpg`: each row a stage, match camera left, wide right: meadow, grove, beach, canyon,
city), and close crops (`review10/review10_crops.jpg`, left to right and down: the meadow's hill under the wall wide, the canyon's floor,
the grove's abyss, the city's middle towers, the canyon's sky, the city's facade).

## Scorecard

| Stage | Light | Surfaces | Play area and front | Background land | Sky | World and life | Cohesion | Readability | Mean | Review 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Beach | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Canyon | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 3.9 |
| City | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Grove | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| Meadow | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | **4.0** | 4.0 |
| **Mean** | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | 4.0 | **4.0** | 3.98 |

- **Every column is a clear 4 on every stage, and nothing is a defect.** The canyon's step back is closed (sandstone in uneven beds).
  Review 9's list did what it set out to: the meadow's wall sits back, the beach's targets and border are gone, the grove is one stone
  place, the city's street reads as a road, and the contact shade is deeper everywhere.
- **Readability holds.** P1's value gap to the ground: meadow 0.042, canyon 0.040, beach 0.171, city 0.126, grove 0.049; the grove and
  canyon are the narrowest, carried by the fighters' edge light and hue.
- **No column is a 5.** What's left is not wrong; it's finish that the references have and we don't.

## What still blocks 5

| Gap | What it is | Where |
|---|---|---|
| **Painted variation in the surfaces** | The pattern behind the light and surfaces columns. Most surfaces are one flat colour times a grey print: the fronts, the tops, the terraces, the rock. The references paint broad value and hue drift across a surface (warmer and lighter where it turns to the light, cooler and deeper as it turns away and goes down, a colour shift across a wall). Reviews 8 and 9 showed the light can't add this: key, shade colour and sun angle moved the regions within 0.01. It has to be in the surfaces. | every stage, the fronts and tops most |
| **Plain lower thirds at the wide camera** | The meadow's hill under the wall is a plain green faceted mound with a few stray white specks. The grove's abyss is a flat purple band with one wavy mist line. The canyon's floor is broad sand with faint ripples. Each is a third of the wide frame with the least finish in it. | meadow, grove, canyon |
| **The canyon's front at the wide camera** | Better (sandstone), but a third of the frame is still a block of horizontal layers. Painted variation (above) is the likely fix: a face that darkens and cools as it goes down reads as rock mass, not a stack. | canyon |
| **The fighters** | Half of every still. Their finish is the character and animation sessions' work, not this one's. | all |

## The order

1. **A painted-variation pilot on one stage (the meadow):** broad value and hue drift on its front and top in the existing paint
   (`toon_common`'s world-space paint, the palette), not a new element: warmer and lighter toward the light at the top, deeper and cooler
   down the front, a slow colour wander along it. Measured against today. If it reads, take it stage by stage (the owner's rule).
2. **The meadow's hill under the wall:** the lawn's own print and finish on it, and the stray specks sized or removed.
3. **The grove's abyss:** depth in its mist, darker toward the bottom, the second mist layer reading behind the first.
4. **The canyon's floor and front at the wide camera:** after item 1, the same painted drift on the front, and broader shading on the
   floor's sand sheets.

Items 2 to 4 are refinements of what exists. Item 1 is the one with a chance of moving surfaces and light toward 5. Without it the
honest ceiling for this look is a strong 4.

## Item 1, painted variation, the meadow pilot (2026-10-25, built on the meadow)

`toon_common.gdshaderinc` `paint_drift` [value patches, warm-cool wander, size, tops warm and walls cool] in the fragment, on the
scenery only (not fighters): broad world-space `brush_noise` patches of value, a slow hue wander between a cool and a warm tint, tops
warmer and walls cooler. `StageBake._paint_drift` sets it on the modelled front and play area from the palette's [backdrop] `paint_drift`
(`--paint-drift=A|B|C`; a fifth value 1 keeps it to the play area). A first pass at [0.7, 0.5, 8, 0.6] / [1.0, 0.85, 6, 1.0] moved
nothing visible; retuned to A [1.3, 0.8, 14, 0.5] and B [2.0, 1.2, 18, 0.7]. Measured at the match camera:

- **The lawn:** contrast 0.073 to 0.096 (A and C), the first measurable surface gain since review 8.
- **The front:** under A value 0.45 to 0.42 and saturation 0.29 to 0.26 (B 0.39, 0.24): its small stones and logs already vary, so the
  drift only darkens and greys it. C leaves the front out.
- **Readability:** P1's gap 0.038 today, 0.039 (A), 0.036 (C).

Note: the shader change recompiles every scenery material on first load (a first capture took about three minutes). On the Stage Look
Review ("paintdrift11", `pd11_*`); recommended C, then stage by stage on the other play areas.

Built C on the meadow (the owner's pick): meadow.cfg [backdrop] `paint_drift=[1.3, 0.8, 14.0, 0.5, 1]`, the play area only; the front
unchanged (value 0.45, saturation 0.29, contrast 0.101). The lawn measured 0.085 in this capture (0.096 in the mock; the drifting cloud
shade moves it). Before and after at the match camera: `review10/meadow_paint_built.jpg`. All seven tests pass. Next: the other four
play areas, stage by stage.

## Item 1 on the other play areas (2026-10-25, mock)

The meadow's setting, play area only (`--paint-drift=C`), on each of the other stages at the match camera:

| Stage | Play top | P1's gap | Verdict |
|---|---|---|---|
| Grove | contrast 0.094 to 0.105 | 0.047 to 0.064 | gains (and widens the narrowest gap of the five) |
| Canyon | value 0.62 to 0.57, contrast 0.101 to 0.087 | 0.040 to 0.053 | worse: the sand already has drifts and cloud shade |
| Beach | contrast 0.131 to 0.128 | 0.173 to 0.187 | no gain: the boards already vary |
| City | value 0.54 to 0.52, contrast 0.133 to 0.129 | 0.133 to 0.149 | no gain: the roof's tar patches already do this |

It works where a top was one flat colour (the meadow's and the grove's lawns), not where the surface already varies. On the Stage Look
Review ("paintdrift12", `pd12_*`); recommended the grove only.
