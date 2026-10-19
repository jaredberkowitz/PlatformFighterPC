# Stage style sheet (draft, 2026-10-19, for the owner's approval)

One rule set every stage element is checked against, so the stages read as one game drawn by one hand. It sits on top of
`docs/ART_DIRECTION.md` (the attention hierarchy, palette and lessons) and replaces nothing there; where the two disagree, this sheet is
the newer decision. The gauntlet judge scores against this sheet first (`docs/gauntlet/JUDGE.md`).

**In one line:** soft-lit, chunky, hand-made places under one warm sun: big calm shapes, crisp lit edges, and detail only where the
fighters are.

**The look:** the current cel look. The paper look (F10) is retired (the owner, 2026-10-19: the current look is marginally better).

## 1. Layers, values and edges

Measured in greyscale at the match camera (0 black, 1 white). Every stage has these layers, front to back:

| Layer | What | Value | Saturation | Detail | Edges |
|---|---|---|---|---|---|
| Fighters | the players | their full range | highest on screen | their own | full ink outline |
| Play surface | the tops, platforms, props on the tops | 0.40 to 0.55, and at least 0.05 from every fighter | rich, a step under the fighters | main features at least 8 px | soft ink, a lit rim on top edges |
| Stage front | the wall, bank, facade or deck under the top | 0.25 to 0.45, falling into shade downward | a step under the top | calmer than the top | soft ink |
| Mid-ground | towns, trees, fields, near hills | 0.45 to 0.70 | medium, cooler than the front | features at least 8 px, fewer than near | no ink |
| Distance | ranges, far skyline, far hills | within 0.15 of the sky behind it | low, cool | big shapes only | no ink |
| Sky | | 0.75 to 0.95 (night: its own darks) | low | clouds only | none |

A layer never outshines the one in front of it in contrast, saturation or detail. Signs, lamps and glows stay below the fighters.

## 2. Colour

- Each object keeps its own hue; the stage's light bends it and never replaces it.
- Light warm (peach, cream, amber), shade cool (teal, violet, indigo), as `lighting.gd` sets per stage.
- Warm foreground, cool background: distance reads cooler and lighter, never only paler.
- The ground stays colourful (`ART_DIRECTION.md`, lesson 1). Calm it by lowering its contrast and noise, never its saturation.
  (Several gauntlet rounds muted the meadow's lawn and fields; the pilot restores their colour.)

| Stage | Warm near | Accents | Cool far | Sky |
|---|---|---|---|---|
| Meadow | lawn green, honey wood, warm grey stone, straw gold | red roofs, flowers | teal ranges, blue-green hills | cream to blue |
| Grove | spring green, terracotta earth, honey planks | pink mushrooms, string lights | lilac hills, deep green forest | pink to lilac |
| Beach | honey deck, sand, weathered wood | red life ring, sail stripes | turquoise sea, pastel town, blue-green headland | clear blue |
| City (night) | warm terracotta roof and brick, amber windows | coral girders, neon | violet and indigo towers | pink horizon to indigo |
| Canyon | held as it is | | | |

## 3. Light

- One warm key, one cool fill, coloured shade (`pf_shadow_hue`).
- Soft occlusion in every crevice; a contact shadow under every prop.
- Shading rolls off softly across each organic form (grass, crowns, rock, earth): no hard two-tone splits. Designed low-poly props may
  keep their flat facets.
- Every top edge that faces the key catches a lit rim.

## 4. Shape

- Modelled (Blender) over code-built primitives.
- Every edge that catches the light is rounded or bevelled; natural edges are irregular, built ones straight but worn.
- No ruler lines on natural things, nothing floating, nothing primitive (a bare box, sphere or cylinder) in view.

## 5. Surface

- One method: tiles rendered from modelled geometry (`art/blender/make_tiles.py`), a greyscale value tile times the object's colour.
- Main features at least 8 px at the match camera (about 0.17 world units at the standard framing); anything finer averages to tone.
- The tile's value range is set by its layer: the most contrast near, the least far.
- No photographic textures, no noise for its own sake. The hand-drawn SVG tiles are replaced as each surface is reached.

## 6. Detail and clarity

- Detail goes to the play surface and its props. The stage front is calmer than the top; far layers are simple big shapes.
- Refine, don't add: improve or remove what is there before anything new.
- Every element either follows this sheet or is fixed or removed; no third option.

## 7. How a change is checked

1. Greyscale: each fighter at least 0.05 from what it stands on.
2. A contact sheet of all five stages side by side: the change must not make one stage stand apart from the others.
3. The judge's outlier list: what breaks the sheet, removed or unified before anything new is added.

## 8. The meadow pilot

The meadow goes first; the others follow once its result is approved. Its checklist against this sheet:

- Lawn: its colour restored (it was muted), its texture calmer; no dark speckle.
- Wall: the busiest thing in the frame. Calmer: fewer value steps, stones within 0.30 to 0.40, the lichen subtle.
- Platforms: the boards are on the sheet; the log-end supports and straps still built from primitives.
- Fence, bales, barrel, sign: a contact shadow under each, a lit top edge.
- Valley: the fields and farm sit in the mid-ground band; the village houses' colours a step under the play area's.
- Distance: the ranges are on the sheet; keep them.
