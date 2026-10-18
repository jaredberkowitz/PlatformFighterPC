# Art direction

Reference: [`art/reference_character_style.webp`](art/reference_character_style.webp). This is a *style* target only;
every character, accessory and name we ship must be original (plan section 2).

## What the reference shows

- **Body**: simple, round, near-featureless "blob" bodies and heads in one flat saturated colour each
  (yellow, orange, blue, pink). Short stubby limbs, mitten hands, chunky feet. Very readable silhouettes.
- **Face**: a 2D hand-drawn face with thick dark outlines applied to a 3D head. Heavy-lidded, deadpan
  expressions (half-closed eyes, slack mouth, drool, frown). The face reads as a *texture*, not geometry.
- **Costume layers**: uniform shirt/shorts, a diagonal sash with badges, socks and shoes. All simple shapes
  with soft cel-style shading and almost no texture detail.
- **Accessories**: hats (sailor cap, safari/straw hat, aviator cap with goggles), sunglasses, neckerchief.
  Each is a self-contained mesh sitting on the head or body.
- **Rendering**: soft cel shading, warm rim light, flat colours, clean outlines. Low poly count.

## How it maps onto the plan

| Reference trait | Plan section | Implementation note |
| --- | --- | --- |
| Face drawn on a round head | 7.3 expressions | Face texture atlas on the head mesh; hurt/idle/taunt faces swapped by cosmetic timeline events |
| Hats, glasses, neckerchief | 7.3 accessories | Mesh + socket (`head`, `face`) + optional recolour slots |
| Sash and badges | 7.3 accessories | `back`/`hips` sockets or a torso overlay texture |
| Flat colour per character | 7.3 recolour slots | One base-colour slot per body type, palette swaps for teams |
| Round, simple bodies | 7.4 body types | Start with blob body types: small/fast, medium, heavy. Same bone and socket names |
| Readable silhouettes | Design pillars | Also supports the competitive-readability mode (team-coloured outlines) |

## Production rules

How new art is made, cleaned up and brought in (concept, generation, clean-up, budgets, review): `ART_WORKFLOW.md`.


- Gameplay hurtboxes come from character data, never the mesh. Round bodies make capsule placeholders
  a close visual match, so placeholder fighters can already be blob-shaped.
- Keep budgets (plan 7.3) tight: the style needs few polygons, and tiny textures for faces.
- Don't copy the reference's specific characters, outfits or badge designs. Build original ones in the same spirit.

## Stages

Stage art is built in code from the stage's geometry and its **look** (`StageLook`: a backdrop name and optional sky colours, presentation only,
not in the content hash), in `godot/scripts/stage_art.gd`:

- **Blocks**: patterned soil (rows of diamonds in two shades) with a thick grass cap that rolls over the front edge and hangs in tufts, on any
  block whose top is not covered by another. Stacked blocks make an island that narrows underneath (Treetop Isle), so the underside you can
  bump into is the shape you see.
- **Platforms**: rounded wooden planks, a light board on a darker beam, with seams.
- **Backdrops**: *meadow* (rolling hills, a turning windmill, round trees, a fence and flowers behind the fighters), *grove* (a giant cartoon tree behind a floating island, bushes and flowers on the island, two
  smaller trees far behind, pastel hills, a pink-to-blue sky and twinkling glints in the canopy), *sunset* (a canyon at dusk: a low sun,
  striped mesas, a rock arch, cacti), *night* (a big moon and stars), *ocean* (a sandy island on a swelling sea, distant islands, a lighthouse,
  a sailboat, palms) and *city* (a rooftop in front of a lit skyline, a water tower, a blinking antenna). Each theme also sets the block
  pattern (quilted diamonds, sandstone strata, cobbles, or a building facade with lit windows), what tops a block (grass, sand or a rooftop
  ledge) and the platforms (wooden planks or riveted steel girders). The built-in stages use: Meadow *meadow*, Triple Tier *sunset*, Flat
  Island *ocean*, Skyline *city*, Treetop Isle *grove*.
- **Under the stage** (`StageArt.underside`): no stage is a brick floating in the air. Under the lowest block hangs what the place is
  made of: an earth keel with rocks and roots (meadow, grove), the mesa's column (sunset), a rock shelf in the sea (ocean), the building
  carrying on down to the street (city). It is set back `UNDER_RECESS` behind the blocks' front, so fighters pass in front of it and the
  block's bottom edge (inked by the depth step) stays the true edge of the solid ground. A bank of haze in the horizon's colour
  (`_under_haze`, behind the fighters) swallows its lower part. Block fronts are calm (soft mortar and ink, large repeats) and fall
  further into cool shade below the lit rim, so the wall never outshouts the fighters.
- Treetop Isle is our take on the classic "floating island under a big tree" stage: original shapes, no face on the tree, no borrowed
  characters or names.
- Surfaces and scenery since the graphics fidelity pass (`docs/GRAPHICS_FIDELITY_PLAN.md`, results in `docs/GRAPHICS_FIDELITY_LOG.md`): the
  skyline's windows are painted by one facade shader and glow; stage surfaces carry painted wear at two scales; tree crowns are leaf
  cards over solid cores that move in the wind, with grass tufts behind the fighters; the sea has bands, streaks, glints and foam; roofs
  and floors take decals (stains, scuffs from heavy hits). All of it is drawn in the same hand-drawn style as the tiles: new tiles are
  SVGs in `godot/art/stage/`.
- Check a stage's art with `Godot --path godot -- --demo=stage --stage=N --noui --noecb --shots=<folder>` (the still at frame 40 and a
  second one at frame 100, for anything that moves), and judge a change with `godot/tests/shot_stats.gd` and the look bench.

## The rules every screen follows

The game should look like one artist drew the fighters, the stage, the light, the effects and the HUD together. The references are for
qualities, not content: decisive cel shadow shapes and confident silhouettes (*the reference cel-shaded adventure*), simple playful bodies (*PEAK*), warm
light against cool atmospheric shadow (*Valheim*). Never copy their characters, places or designs.

**Attention hierarchy.** Three levels, in this order; a change that lifts a lower level above a higher one is wrong even if it looks good
alone.

1. *Primary*: fighters, attacks, hit effects, damage. The most contrast, the clearest edges (full hull outline, edge light, rim boost),
   the truest colours (they take only half the pastel finish and a third of the grade, `post.gdshader`).
2. *Secondary*: the stage, platforms and nearby props. Clear edges where you can stand (the lit trim along every platform), moderate
   detail, colours a step calmer than the fighters (the city's girders are coral, not red).
3. *Tertiary*: buildings, skyline, sky, decorative lights. Lower contrast and less detail with distance (haze, far blur, the far layer has
   windows only); lit windows in clusters, not an even grid; only the nearest layer's windows bloom, and softly.

**Palette.** Each object keeps its own colour; the stage's mood bends it, it never replaces it. Light leans warm (peach, amber, ivory:
the key light and the painted gradient's tops), shadow leans cool (violet, indigo, plum: `pf_shadow_hue`, the painted shadow), with
pastel accents (mint, seafoam, pale cyan, dusty pink) for small things. Backgrounds use muted plum, indigo, mauve and blue-grey. The
player colours and the effects' colours stay distinct from all of these. All of it is set per stage in `scripts/lighting.gd` (`MOODS`,
`STYLES`), never as a tint over everything.

**Material families** (all one cel material family, `VISUAL_STYLE.md`; responses from `FighterView.RESPONSES` and the shared light):
skin soft; cloth matte; rubber (gloves, soles) a soft broad sheen; metal (the blade) a crisp glint band and stronger rim, never a
mirror; stone and concrete broad, low-frequency painted variation (no noisy texture); emissive windows and signs bright in colour, glow
restrained; distant buildings flat-shaded and simple.

**What we do not add**: photographic textures, realistic wear everywhere, screen-space reflections, heavy depth of field in play, motion
blur, film grain beyond the existing paper grain, chromatic aberration.

## Lessons from the owner's style references (2026-10-17)

The owner shared frames from a cel-shaded island adventure and from a papercraft adventure (forest, village and camp scenes). Names
stay out of the repo; these are the lessons, which every stage, prop and the menu backdrop follow.

1. **Quality comes from composition, not from muting.** The papercraft camp is bright: saturated lime grass, warm sand paths, and
   it still reads as premium because every colour has a job. Vivid ground sits against cool teal mountains, deep pine-green trees
   and warm wood. Don't desaturate the ground to look "grown up"; make the neighbouring colours contrast with it. (A muted-ground
   try was reverted.)
2. **A temperature plan per stage.** Warm foreground (ground, wood, sand), cool background (mountains teal or blue-green, far trees
   deep blue-green). Distance shows as cooler and lighter, not just paler.
3. **Low-poly done with intent.** Flat-shaded facets are fine and even striking when the shapes are designed: trees as stacked tiers
   with long drooping panels, stumps with stepped rings and folded bark, rocks as a few big chunky planes with a light top face.
   Each facet one tone; the light reads in crisp planes, not smooth plastic gradients.
4. **Clear silhouettes, toothed or folded edges.** Leaves end in points, paper folds, bark in planes; outlines are never a smooth
   ball. Small props (stumps, rocks, fence posts) have a clear lit top and a darker side, so they read from the match camera.
5. **Layered mountains.** Mountains are built from overlapping paper-like layers, each band a step lighter toward the peak, with
   crisp lighter edges. They are a backdrop shape that frames the stage, cool against the warm play area.
6. **Restraint in detail.** Detail goes where the eye lands (the stage, nearby props); far layers stay simple big shapes. Raise the
   quality of what is there before adding anything new (the owner's rule).
7. **Light with a direction.** One clear sun side and shadow side on everything, shadows coloured (cool teal-blue), never grey.

Applied so far: tiered faceted tree crowns and trunks (`art/blender/make_props.py` `tree_crown_*`, `tree_trunk`), the meadow's mountain
flanks lit and turned teal. Next candidates: stumps, rocks and cottages in the same faceted language; the mountains' layered bands.

## Depth and dimension rules, every stage (owner's outside brief, 2026-10-17)

Applied to all five stages, each tuned to its own palette and light:
- **Depth separation:** the play area is the most saturated and sharpest layer; far layers sit back in the stage's own air colour
  (painted ridges' `haze`, the finishing pass's `haze_max`). The finishing pass greys scenery only a little (`quiet` 0.0-0.12 by stage,
  was 0.25): greying flattened the mid distance.
- **Painted surfaces:** walkable tops take large tonal patches, a deeper and a sunlit tone (`patch_amount` in toon_common, a theme's
  `grass_patch`; roofs 0.25), instead of one flat sheet.
- **Terrain as a solid:** the block front falls into cool shade over the face you see (`front_fall`/`front_span` per pattern).
- **Layered mountains:** ridges cut into flat paper bands with lit lips and a deep foot (`layers`, `foot_dark`, `flank_from`).
- **One language for plants:** faceted tiered crowns and pointed turf, never round blobs; distant plants simpler than near ones.
- Never make the stage more complicated than the fighters.
