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

**The owner picked D** (2026-10-20). Applied: `meadow.cfg` `ledge` (0.4, 0.62, 0.32), and the block's body behind the ledge now follows it
(`ledge` a step darker; it was hard-coded to the old dark green-grey).

**Result** (`pass2_meadow_cliff.png`: match camera, close-up and fighters in play; round 46): the cliff 0.17 → 0.36, in the sheet's
band, hue 182 → 133 (the land's green, not a teal void); the lawn's lit top still stands 0.28 above it; the fighters stay the clearest
thing. Stage fronts now: meadow 0.36, grove 0.48 to 0.53, city 0.41, beach 0.33 (pass 4), canyon 0.32 (held).

**Tried and reverted:** folding the ledge (columns alternately out and in, the sheet's rule 9) to split its planes into lit and shaded:
no measurable change (contrast 0.036 → 0.035) and a dark sliver at one crease. The meadow's key falls nearly straight onto the face, so
planes turned left and right light alike; a real split needs the face's planes tilted up and down (toward and away from a high key) or
a scenery rim term (backlog item 2, pass 6).

**Moved:** the beach's posts (0.16) go with the beach's wood mocks in pass 4: the deck's colour is the owner's pick there, and the posts
must sit a step under whatever it becomes.

**Checked.** `prop_check_test`, `art_pipeline_test`, `match_flow_test`, `editor_ui_test` (the editor's meadow stages use this ledge),
`replay_test` pass. `runner_test` failed 2 of 6 runs on "the way out" (the run shop's hint read three frames after the bell, before the
HUD updated, while the machine was under load): a timing race in the test, unrelated to stage art (the run mode is built from the city,
untouched here); passed on the other runs.

## Pass 3: the city's readability (2026-10-20)

**Defect** (`REFERENCE.md`, weakness 1). The look bench fails (the midnight body on the city's dark backdrop, edge 0.39); the near tower
layer is the darkest large area behind the fighters (0.22); the OPEN and SHOP signs (0.55) outshine both fighters (0.47 to 0.52); the
sky is a hot magenta (saturation 0.68) against blue-violet towers, where the reference's night is one muted violet family.

**Fixed** (`city.cfg`):
- The skyline's three shades lighter and a little redder: near 0.22 → 0.30, far 0.34 → 0.36 (hue 262 → 267, the lighting pulls them
  blue; the reference is 286 to 306).
- `sign_dim` 0.6 → 0.3: the signs 0.55 → 0.45 to 0.47, at or under the fighters, still legible.

**Mocked, for the owner** (`mocks/city_sky_options.png`, with the new towers and signs; the committed sky is unchanged, A):

| Option | `sky_horizon` / `sky_glow` / `sun_side` | Sky value, hue, sat |
|---|---|---|
| A, today | 0.42, 0.22, 0.46 / 0.95, 0.38, 0.6 at 0.45 / 0.4 | 0.43, 324, 0.67 |
| B, calmer pink | 0.42, 0.24, 0.48 / 0.8, 0.42, 0.62 at 0.32 / 0.25 | 0.37, 310, 0.59 |
| C, deep violet | 0.42, 0.26, 0.5 / 0.78, 0.44, 0.62 at 0.2 / 0.15 | 0.31, 300, 0.57 |
| D, grey violet | 0.38, 0.3, 0.46 / 0.66, 0.5, 0.62 at 0.2 / 0.15 | 0.31, 279, 0.48 (the reference: 0.25, 286, 0.36) |

The sky's pink is mostly `sun_side`, which tints the whole sky ahead with the glow's colour (the camera looks close to the horizon).

**The look bench, diagnosed, not fixed.** It still fails (0.39 → 0.39). Tried and reverted, each with no effect on that cell: the city's
edge light 0.55 → 0.65, its fill 0.48 → 0.62 (0.41), its ambient 0.55 → 0.75 (0.41). The causes:
- The fighters' edge light is scaled by each body's own colour (`PF_FIGHTER_CREAM` in `toon_light.gdshaderinc`, the character pass), so
  on the near-black midnight body it is about a third as strong; and the stage's lights multiply that same near-black colour.
- The bench's dark backdrop is synthetic: the mood's ambient darkened by 0.72 (0.23 here), not the stage. It builds every mood before
  rendering, so per-mood globals (edge, rim, ramp) are not compared per mood either.
- In the game the midnight fighter reads (`pass3_city.png`, bottom row: player 1 given the midnight body for the capture, reverted): it
  stands on the roof (0.40) against towers now at 0.30 or more, and its head reads better against the lighter towers.
Fixing the cell needs a fighter-side floor on the edge light for very dark bodies, or the bench measuring against each stage's real
darkest backdrop. Both are outside this pass; the owner's call.

**Checked.** Round 47 (city); `pass3_city.png` (match camera, a 2x close-up of the signs and towers, and the midnight fighter in play,
before and after); the run mode (`run_shot`, before and after: its own facades fill most of the frame, the far towers a touch lighter,
nothing breaks); `prop_check_test`, `art_pipeline_test`, `match_flow_test`, `runner_test`, `editor_ui_test`, `replay_test` pass.

**Left.** The sky, per the owner's pick. The roof (0.40) stays under the fighters by 0.06 or more; the reference makes its play pieces
the palest things, but lifting the roof would close that gap.

**The owner picked sky B** (calmer pink, 2026-10-20; reviewed on a flip/slider page). Applied in `city.cfg`: `sky_horizon` 0.42, 0.24,
0.48; `sky_glow` 0.8, 0.42, 0.62 at 0.32; `sun_side` 0.25. The sky 0.43 → 0.37, saturation 0.67 → 0.59, hue 324 → 310 (round 48,
`pass3_city_sky.png`). The run mode's sky calms the same way (checked with `run_shot`). Tests as above, all pass.

## Pass 4: the beach's wood (mocks, 2026-10-20)

**Defect** (`REFERENCE.md`, weakness 4): the deck bleached (0.76, saturation 0.28; the harbour reference's honey 0.55, saturation 0.60)
and the posts nearly black (0.16 to 0.23). **Cause:** `_dock_block` hard-coded the deck colour (0.86, 0.7, 0.5) and the post colour,
and lifts the play line by `worn_tint` 1.6. They are now the palette's `[backdrop]` `deck_wood`, `deck_worn` and `pile_wood` (read from
the theme's own palette by name: the blocks are built before `backdrop` sets `_palette`), today's values the defaults.

**The constraint:** the fighters measure 0.54 (orange) and 0.60 (yellow) on the deck, so a deck 0.05 from both sits under about 0.49 or
over about 0.65. Mocks (`mocks/beach_wood/`, the match camera and fighters in play, on the owner's review page):

| Option | `deck_wood` / `deck_worn` / `pile_wood` | Deck | Posts |
|---|---|---|---|
| A, today | 0.86, 0.7, 0.5 / 1.6, 1.45, 1.25 / 0.36, 0.26, 0.2 | 0.76, sat 0.28 | 0.23 |
| B, deep honey | 0.5, 0.32, 0.18 / 1.16, 1.1, 1.04 / 0.44, 0.29, 0.19 | 0.48, sat 0.48 | 0.25 |
| C, rich light honey | 0.92, 0.7, 0.42 / 1.35, 1.28, 1.15 / 0.5, 0.34, 0.22 | 0.75, sat 0.38 | 0.30 |
| D, mid honey | 0.8, 0.54, 0.3 / 1.25, 1.18, 1.1 / 0.55, 0.36, 0.22 | 0.67, sat 0.47 | 0.33 |

A first B (0.7, 0.46, 0.26) measured 0.61, the fighters' own value: the stage's light lifts the boards more than their colour suggests.
Recommended B, the reference's honey under the fighters.

**The owner picked B** (2026-10-20). Applied in `ocean.cfg`: `deck_wood` 0.5, 0.32, 0.18; `deck_worn` 1.16, 1.1, 1.04; `pile_wood`
0.44, 0.29, 0.19. Round 53, `pass4_beach_wood.png` (match camera, close-up of the deck and posts, fighters in play): the deck 0.76 →
0.48 (saturation 0.28 → 0.48, hue 41 → 27, the harbour reference's honey), the posts 0.23 → 0.25 and warm brown (hue 210 → 17), the
fighters 0.53 and 0.60, both at least 0.05 clear. The crates, barrels and the far jetty keep their paler wood: the props stand off the
deck and the jetty sits back. Only the beach theme builds a dock (and the editor's custom stages with that theme). Tests: all six pass.
