# Stages more their own: the style pass (2026-11-02)

The owner, after fidelity review 17: "do you have any recommendations on making them feel a bit more unique, stylized?" Then: "Please
continue with what you mentioned, starting with stepped skies."

**The diagnosis:** all five stages share one paper finish (the same fibre, print, cut edges, card clouds and folded trees). So they read as
one style in five palettes. The papercraft reference makes each place feel different by giving it its own craft material, not only its
own colours.

**The rules every step keeps:**
- Refine what is there; add nothing. No animals.
- The play areas stay calm and readable.
- The camera stays accurate to the reference. The grove stays violet.

## The plan

Across every stage:
1. **Stepped paper skies.** Each sky cut into flat paper layers with lit cut edges, like a diorama's backdrop, in place of a smooth
   gradient.
2. **Scenery on twos.** The windmill's sails, waves and smoke step at half the frame rate, like stop-motion. The fighters stay smooth.
3. **Each stage's paper colour on its cut edges.** White card for the meadow, kraft for the canyon, a blue-grey board for the city.

One craft per stage, applied to what is already there:
- **Meadow, patchwork and stitch:** the fields as fabric patches with stitched seams, the hedgerows as rows of stitching.
- **Canyon, corrugated cardboard:** the strata and mesas show their flutes on their cut edges; the sun's rays as cut strips.
- **Beach, origami:** the waves and sails as crisp folded paper; the foam as torn paper.
- **City, pop-up book:** the buildings as hinged pop-up layers, windows printed in halftone ink, the neon as coloured cellophane.
- **Grove, washi:** the canopy's edges and the lanterns in translucent paper glowing through; the mist as layers of tissue.

Order: the stepped skies first (every stage, no remodelling), then the canyon's cardboard (the weakest stage).

## Step 1: stepped paper skies (mock, 2026-11-02)

`sky.gdshader` `steps`: the sky over the horizon cut into layers. Each layer takes the sky's own colour (bands, glow, the sun's side and
the horizon haze) at its height, so a layer is flat up and down but still warmed across by the glow.
- The cuts sit closer together low down, where the cameras see most of the sky.
- They wander as if cut by hand, by a share of the gap under each.
- Each cut is a pixel soft, so it doesn't step at 1080 lines.
- Each layer has a light along its own cut edge and the soft shadow of the nearer layer under it.
- The cut-paper stratus and the card clouds stay in front.

`--sky-steps=T|A|B|C`, or the palette's `sky_steps` [layers, wander, cut edge, shadow, paper]:

| Option | What changes |
|---|---|
| Today | A smooth painted gradient |
| A, flat layers | Six layers, near-straight cuts, a light along each cut |
| B, cut layers | A, with the cuts wandering as if cut by hand and each nearer layer's soft shadow on the one behind |
| C, paper layers | B, with a paper fibre and a slight tone of its own on each sheet |

The play areas don't change, so the fighters' numbers hold. On the Stage Look Review ("skysteps65", `sk65_*`).

Recommended B: the wandering cuts and their shadows read as layers of cut paper, where A's near-straight cuts read as stripes. C looks the
same as B at normal size; its fibre only shows up close.
- It shows most on the canyon's sunset and the beach. The city's towers and the grove's trees hide most of their skies.
- The canyon's soft sun is a separate piece and stays smooth. A cut-paper sun would be a next step.

**Built B** (the owner's pick, 2026-11-02):
- `sky_steps=[6, 0.8, 0.7, 0.6, 0.0]` in each stage's palette (meadow, sunset, ocean, city, grove); `--sky-steps=T` brings back the smooth
  sky.
- The shipped skies measure the same as the mock's B.
- Before and after, the five stages at the wide camera (rows meadow, canyon, beach, city, grove; before left): `style/skies_built.jpg`.

All seven tests pass.

## Step 2: the canyon's cardboard (mock, 2026-11-03)

toon_common `corrugate` [board height, flute period, fluting, hollows]: what faces sideways shows the cut edge of stacked boards, counted
down from the front's top. Each board has a pale liner at its top and bottom and a fine seam to the next. Between the liners is the
fluting: a light wave from liner to liner, with the arches either side of it hollow and in shade. It is on the canyon's front beds and its
platforms' sides only. `--cardboard=T|A|B|C`, or the palette's [backdrop] `cardboard` (StageBake.CARDBOARD_TRY):

| Option | What changes |
|---|---|
| Today | The natural rock beds |
| A, light | Boards with fluting, the hollows a soft shade |
| B, deep | The hollows deep, as a card's cut edge shows them |
| C, bold | B, with boards half as tall again and broader flutes |

A first try with a light wave and pale hollows read as a zigzag print. The arches were then drawn hollow and in shade.

**Honest read: none of these reads as cardboard.** Flutes need straight, flat boards to read as card. On the front's eroded, wandering beds
(review 17's natural strata), they read as a woven chevron pattern, like a blanket, worst at the wide camera. Recommended: keep today's
front. On the Stage Look Review ("cardboard66", `cb66_*`).

Proposed instead for the canyon's own craft: **torn paper**. Each bed's edge torn, with the paper's pale core showing along the tear. It
suits rock shapes, and it is a classic papercraft way to draw strata.
