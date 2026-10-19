# Stage style sheet (draft, 2026-10-19, for the owner's approval)

One rule set every stage element is checked against, so the stages read as one game drawn by one hand. It sits on top of
`docs/ART_DIRECTION.md` (the attention hierarchy, palette and lessons) and replaces nothing there; where the two disagree, this sheet is
the newer decision. The gauntlet judge scores against this sheet first (`docs/gauntlet/JUDGE.md`).

**In one line:** soft-lit, chunky, hand-made places under one warm sun: big calm shapes, crisp lit edges, and detail only where the
fighters are.

**The look:** the current cel look. The paper look (F10) is retired (the owner, 2026-10-19: the current look is marginally better).

**The colour, light and depth target (the owner, 2026-10-19):** the papercraft adventure's blue-mountain camp, without its paper
aesthetic. High-key and clean: bright saturated lime ground, warm wood and sand, cool saturated blue mountains layered lighter toward the
peaks, a clear blue sky. The foreground had drifted dark and muddy under value rules that kept the ground darker than the fighters;
the fighters read by their outlines, edge light and colour, and may stand on ground lighter than they are.

## 1. Layers, values and edges

Measured in greyscale at the match camera (0 black, 1 white). Every stage has these layers, front to back:

| Layer | What | Value | Saturation | Detail | Edges |
|---|---|---|---|---|---|
| Fighters | the players | their full range | highest on screen | their own | full ink outline |
| Play surface | the tops, platforms, props on the tops | 0.45 to 0.72, at least 0.05 from every fighter, lighter or darker | rich and bright: vivid ground | main features at least 8 px | soft ink, a lit rim on top edges |
| Stage front | the wall, bank, facade or deck under the top | a step under the top (0.35 to 0.55 on a bright stage), falling into shade downward | a step under the top | calmer than the top | soft ink |
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


## 9. The rules the meadow rebuild proved (2026-10-19), for every stage

The meadow, rebuilt against the owner's blue-mountain reference, is the template. Each stage keeps its own palette, place and
personality; these rules are what makes them one game:

1. **Form over texture.** Quality comes from modelled shapes and light, not surface tiles. Big faceted planes, flat-shaded, each one
   clean tone: mountains, hills, ledges, rocks, pines. Texture only where it earns its place (wood grain, brick, roof sheets) and kept
   faint; no texture on large natural surfaces (lawns, hillsides, cliffs).
2. **One construction language.** The stage front is built like the land behind it (the meadow's faceted rock ledge matches its
   faceted hills). Nothing in view drawn in a different technique (painted cobbles, cut cards, primitives).
3. **Clean, bright play surface.** The walkable top is the stage's brightest clean colour, untextured, shaded by the light; a calm path
   or worn line in a warm pale tone where the fighters stand. The ground may be lighter than the fighters; they read by outline and colour.
4. **Dark, simple front.** The face under the top is a deep version of the land's colour, plain and recessive, so the lit top pops.
5. **Shade is a deeper version of each colour** (a small hue turn only), crisp; never grey, olive or violet wash.
6. **Clear air.** Distance reads through cooler, lighter, simpler shapes, not through milky haze or fog bands. Mist only where the
   place has it (the city's night haze).
7. **Measure against the reference.** Each stage's target image gets a measured colour table (`STAGE_REFERENCES.md`); the judge diffs
   the capture against it and the table (`JUDGE.md`, round 35 on).
8. **Mid-ground props unified:** no ink, colours a step under the play area, simple shapes (pines as stacked faceted cones).
9. **Folded, not crumpled** (the owner, 2026-10-19, round 37). The reference's quality comes from forms folded like paper: a pine is
   stacked skirts pleated in and out, so the key light falls on one side of every fold; a mountain is ridges running straight down from
   each summit with a crease between each pair, so every flank is a few big planes wholly lit or wholly in shade, with spur summits and
   shoulders so no two peaks are the same pyramid. Random facets (a jittered grid, lumpy icospheres) read as low-poly noise, not as
   this. Background trees grow in belts at several depths and in a few greens, never one scattered grey-green clump. Far scenery stays
   crisp (the play blur is 1 px, not 3).
