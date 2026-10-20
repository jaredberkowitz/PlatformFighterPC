# Stage refinement log

One entry per pass (`docs/STAGE_REFINEMENT_AGENT.md` Phase 2). The audit, measured table and order are in `REFERENCE.md`. Values are
greyscale luma at the match camera (0 black, 1 white), measured over the same regions as the audit.

## Pass 1: the depth curve (2026-10-20)

**Defect.** On the meadow, grove and beach the far land read darker than the mid-ground in front of it, and further under the sky than
the style sheet's 0.15: the meadow's ranges 0.53 under a 0.71 sky, the beach's headland 0.58 under 0.76 (`REFERENCE.md`, weakness 3;
`STAGE_BACKLOG.md` item 1).

**Cause.** The `[air]` curve (`StageArt._air`) reached only the props that called it (trees, houses, the windmill). The land bands
(`_land_band`: the hill rows and rises), the modelled ranges (`ridge`, a spec's `land_colour`) and the harbour's headland hills took
their palette colours untouched, at any depth, and the curve stopped growing at `end` (100 to 140 units), in front of the ranges.

**Fix.**
- `_air_amount(z)` and `_air_colour(c, z)` give any colour the same air as `_air` gives a node; past `end` the lift keeps growing to a
  palette's new `far_lift` at `far`, so each row behind is a step lighter toward the horizon.
- Land bands and the harbour's headland take it at their depth. The modelled ranges take only the lift, toward a light tint of their own
  hue: lifted toward the pale haze itself (and greyed by `desat`), the meadow's teal fell from 0.47 to 0.34 saturation and read as milky
  haze, which the style sheet's rule 6 forbids (the camp reference keeps 0.48).
- `far`/`far_lift`: meadow 170/0.5, grove 165/0.45, ocean 150/0.3. The canyon and city have none: the canyon's distance is already
  lighter than its sky (and the owner likes it as it is), the city is pass 3.

**Result** (before → after; `pass1_depth.png`, match camera and close-ups, red strip before, green after; round 45):

| Stage | Distance | Sky | Under the sky | Saturation |
|---|---|---|---|---|
| Meadow (ranges) | 0.53 → 0.59 | 0.71 | 0.18 → 0.12 | 0.47 → 0.43 |
| Beach (headland) | 0.58 → 0.64 | 0.76 | 0.18 → 0.12 | 0.44 → 0.30 (the harbour reference: 0.33) |
| Canyon (dunes) | 0.74 → 0.74 | 0.63 | unchanged | 0.38 → 0.37 |
| Grove (far hill row) | 0.48 → 0.50 | 0.73 | 0.25 → 0.23 | 0.36 → 0.34 |

The meadow's near range lost its darkest shade planes and now sits behind the valley; the beach's headland steps back behind its town.

**Checked.** Review round 45 for all five stages (`docs/gauntlet/round45/`); fighters in play on the meadow and beach (they stay the
clearest thing); the look bench is unaffected (it uses only the moods); `prop_check_test`, `art_pipeline_test`, `match_flow_test`,
`runner_test`, `editor_ui_test`, `replay_test` all pass (the editor's custom stages use the same themes). Frame time: per-frame GPU and
render CPU unchanged (meadow 2.48 / 1.27 ms, beach 2.66 / 1.46 ms; draw calls equal); the wall-clock average was noisy during the run
(other load on the machine), not a cost of this colour-only change.

**Left.**
- The grove's purple far hill row (`hills_far`) stays dark on purpose: the owner and the judge's round 39 set it darker so the island
  pops. The grove's depth fix is to darken and cool what sits round the island (its mid-ground plain, 0.58, equals the lawn), in the
  grove pass, not to lift the distance.
- The meadow's valley (0.64) is still a touch lighter than its near range (0.59). Pushing the ranges further washed them; the step reads
  through hue (warm valley, cool teal ranges), as in the camp reference, whose mountains (0.42) sit well under its ground (0.63).
- The skies sit under the sheet's 0.75 band (meadow 0.71), but the owner's references are deeper still (the camp 0.58, the harbour
  0.42): the band, not the skies, may need revising. Not changed.
- The city's tower layers: pass 3, with the same curve.

## Pass 2: stage fronts, the meadow cliff (mocks, 2026-10-20)

**Defect.** The meadow's cliff under the lawn is 0.17, a near-black slab across a quarter of the frame, against the sheet's 0.35 to 0.55
for a stage front (`REFERENCE.md`, weakness 2). Its colour is `meadow.cfg` `ledge` (0.2, 0.3, 0.25), the camp reference's ledge face,
which there is a thin strip.

**Mocks** (a scratch palette swap, reverted; match camera `mocks/cliff_options.png`, close-up `mocks/cliff_options_close.png`; order
top-left, top-right, bottom-left, bottom-right):

| Option | `ledge` | Measured front | Hue, sat | Reads as |
|---|---|---|---|---|
| A, today | 0.2, 0.3, 0.25 | 0.17 | 182, 0.31 | a dark teal void |
| B, deep moss | 0.3, 0.44, 0.3 | 0.25 | 153, 0.33 | the reference's dark green-grey, out of the void, still under the band |
| C, warm earth | 0.66, 0.44, 0.26 | 0.35 | 26, 0.42 | soil under turf; the strongest split from the lawn by hue, ties to the wood and bales |
| D, light moss | 0.4, 0.62, 0.32 | 0.36 | 133, 0.38 | the land's own green in band; the lit top still pops (lawn 0.64) |

Two first tries were dropped: a warm grey stone (0.5, 0.44, 0.36) came out concrete grey (sat 0.08) under the stage's cool fill, and a
light sage (0.42, 0.54, 0.44) went the pale blue-grey the judge's round 39 rejected. Recommended: D. Seen in every option: the facets
barely show (contrast 0.03); once a colour is picked, the next step is a stronger lit/shaded split between the planes.
