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
- Treetop Isle is our take on the classic "floating island under a big tree" stage: original shapes, no face on the tree, no borrowed
  characters or names.
- Check a stage's art with `Godot --path godot -- --demo=stage --stage=N --noui --noecb --shots=<folder>`.
