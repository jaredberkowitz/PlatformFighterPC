# Art review 9: four stages against the style targets (2026-10-18)

Stages: meadow, beach (harbour dock), city, grove; the canyon stays as it is. Match-camera shots:
`docs/screenshots/review9_four_stages.jpg` (meadow, beach, city, grove, top to bottom).

The targets, and what each one asks of us:
- **The cel-shaded sailing adventure (HD):** clean two-tone cel light with a strong sun, soft painted gradients in sky and sea, saturated
  but harmonised colour, bold simple shapes, toon water with white foam lines, cloud shadows moving over the land.
- **The papercraft adventure:** everything reads as folded paper and card: crisp flat-shaded facets, folds and creases, a faint paper grain,
  scenery built as layered cut-out cards, bright colour with strong value contrast.
- **The big crossover fighter:** polish and density: every surface has material detail, platforms have clear readable edges and trim,
  rich rim light and contact shadow, layered backgrounds with a lot of small believable things in them.

## Weak areas that cut across all four stages

1. **Mixed shape language.** Some props are crisp facets (the meadow's papercraft trees, the cottages); others are smooth soft balls and
   tubes (hay bales, bushes, the grove's roots and canopy, the distant hills, the beach's headland). The soft ones read as clay or dough,
   which is neither cel nor paper. *Fix:* one rule, flat-shaded facets with crisp folds, on every organic shape (hills, bushes, roots,
   canopy, rocks); rebuild the soft ones from low-poly faceted meshes (Blender script or code, like the tree kit).
2. **The lawn tops are one saturated lime.** The meadow and grove tops are the brightest, flattest colour on screen. The adventure's
   grass is yellow-green but shaped by strong shadow and moving cloud shadow. *Fix:* pull the lime back a step, deepen it toward the back
   edge, and turn on drifting cloud shadows over the stage tops (the system exists; it is off or weak on these stages).
3. **Floating platforms are plain slabs.** The crossover fighter's platforms have a readable lit top edge, trim and a material; ours are
   thin flat boxes (the meadow's yellow and red, the grove's boards). *Fix:* a platform kit per stage, modelled like the dock: planks
   with bevels and lashing on the meadow, a branch-and-plank shelf on the grove; a clear lit lip on every one.
4. **Weak cel light and cast shadow on the stage.** The sun barely shapes anything on the stage itself; platforms cast no shadow on the
   ground under them at match distance. *Fix:* a stronger key-to-fill ratio and crisper bands on the scenery (the adventure's look), and
   the platforms' shadows on the stage top (a projected or painted shadow under each platform).
5. **Backgrounds lack a layered-card read.** The far mountains and hills are soft blurred shapes. *Fix (papercraft):* far layers as cut
   card silhouettes with a thin lit edge and a cast shadow on the layer behind, the air colour between layers; the haze stays, the blur
   goes.

## Per stage

**Meadow**
- The stone wall's mortar shows the old red earth tile, and the stones are near-rectangular and evenly sized, so it reads as brick. *Fix:*
  a dark grey-brown mortar, rounder, more varied fieldstones, a few big ones, turf and moss tucked into the top course.
- The sky islands are pale, flat and distracting. *Fix:* remove them, or make them faceted cut-card silhouettes far back.
- The windmill sails and the fence are thin flat cards. *Fix:* sails with a lattice frame; fence rails with a lit top.

**Beach (harbour dock)**
- The deck reads busy: hundreds of short boards in many shades. *Fix:* longer boards, a narrower range of shade, a stronger lit lip along
  the deck's front edge and a dark gap under it.
- The cross braces read as a fence. *Fix:* fewer, heavier braces and thicker piles.
- The headland behind the town is a soft dough blob; the town's houses are near-identical. *Fix:* a faceted headland with a rocky shore,
  a few distinct buildings (a tower, a warehouse, a market awning).
- No harbour clutter on the deck edge. *Fix:* rope railing posts, a coiled rope, a net hung on the end, bollards.

**City**
- Everything is one violet. Value and hue contrast are low, so it reads as a monochrome illustration. *Fix:* warm accents (lit windows,
  neon, the water tower's red) pushed brighter; the far buildings darker and cooler; a sodium-orange glow from the street.
- The roof is a flat lilac sheet. *Fix:* a wet roof with soft reflections of the neon and windows.
- Skyline towers are flat boxes (unchanged since review 8). *Fix:* a lit edge on the corner facing the moon, darker side faces, and
  rooftop silhouettes (setbacks, tanks, antennas) already partly there.

**Grove**
- The new roots are smooth uniform tubes in pink-brown: they read as sausages or tentacles. *Fix:* faceted roots with flattened,
  buttress-like cross-sections, bark ridges, knuckles where they bend, and darker colour where they go under.
- The painted roots on the earth block now double the modelled ones. *Fix:* drop the painted roots from the grove's earth tile.
- The forest canopy below is soft muddy blobs. *Fix:* faceted crowns in three clear greens with crisp light and shade.
- The great trunk is pinkish and the same hue as the earth. *Fix:* a browner, darker bark, so the tree and the island separate.

## Suggested order

1. Cross-stage 1 (faceted shape language) and 2 (lawn colour, cloud shadows): the biggest change to the overall read, on every stage.
2. Grove roots, trunk and canopy (the newest and weakest piece).
3. Cross-stage 3 (platform kits) and 4 (stronger cel light, platform shadows).
4. Per-stage items, then cross-stage 5 (layered card backgrounds).
