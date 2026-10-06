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

- Gameplay hurtboxes come from character data, never the mesh. Round bodies make capsule placeholders
  a close visual match, so placeholder fighters can already be blob-shaped.
- Keep budgets (plan 7.3) tight: the style needs few polygons, and tiny textures for faces.
- Don't copy the reference's specific characters, outfits or badge designs. Build original ones in the same spirit.
