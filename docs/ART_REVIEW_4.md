# Art review 4: alpha to beta (2026-10-14)

Main at sim 115, art update 23. Captured at 1080p: all five stages at play distance with the HUD, a strong hit, a knock-out, the main
menu and fighter select. The owner called the art "alpha"; the goal is beta: more detail, more consistency between UI, background and
foreground, higher-quality rendering, closer to a cel-shaded HD remaster.

## Where we are (honest)

- **At beta quality already**: the light (side key, cel ramps, cloud shadows, bloom), the skies (lit cumulus, the glowing sun), the stage
  play surfaces, the effects (the knock-out blast is the standard), the menus' world and spotlit cards, the HUD cards.
- **Alpha**: the background props and buildings (the owner's point, and I agree), the stage front walls, the weapons, the default
  fighters' bare bodies. These are what a player's eye still reads as "simple shapes".

## Weak points, ranked by how much of the screen they cover

| # | Weak point | Seen in | Plan |
|---|---|---|---|
| 1 | **Background buildings and props are primitive shapes**: cottages were boxes with gable wedges poking past the roof, the windmill floated over its hill, the sky islands were faceted inverted pyramids, trees one blob crown. | Meadow, grove. | **Started (pass 64, below).** Next: the city's skyline (flat boxes with one window grid each: cornices exist, but no balconies, signs, rooftop clutter), the beach's palms (stacked cylinders), umbrella and hut, the canyon's cacti and posts. |
| 2 | **Stage front walls**: the beach's sea wall reads as big soft pillows, the canyon's strata repeat in even bands, the city's front is one window grid. They fill the bottom third of every frame. | Beach, canyon, city. | A detail pass per wall: smaller stones with mortar and barnacles at the waterline on the beach; broken strata and overhangs on the canyon; varied windows, AC units, pipes on the city. |
| 3 | **Weapons are placeholder blocks** (the greatsword a flat slab, the longsword a box blade). They are in nearly every frame and next to the most polished thing on screen, the fighter. | Every match. | I'd move the weapons pass up now: modelled blades with a bevel, a fuller, a wrapped grip and a guard, in the metal response. |
| 4 | **Default fighters are bare bodies**: the built-ins in a match show a plain colour body; the outfits exist in the loadout but the defaults barely use them. | Every match. | Give each built-in a signature outfit from the existing catalog (shirt, sash, boots) so the roster reads as characters, not colour swaps. |
| 5 | **Contact and grounding**: some props still sit on the ground without a contact shade (hay bales, barrels on the beach, crates). | All stages. | `_contact` under every prop that touches ground (a sweep). |
| 6 | **Rendering**: thin far lines shimmer (fence rails, sail lattice) at 1080p; no ambient occlusion in creases. | Meadow, city. | Raise MSAA or add a light FXAA/TAA option on the Best tier; a baked crease darkening on the built props. |
| 7 | **UI consistency**: the HUD's player tags (P1/P2) are larger and heavier than the rest of the in-match UI; the "FREE PLAY" banner and the portrait cards use different corner treatments. | Every match. | One pass over the in-match UI with the menu's card language. |

## Pass 64 (this round): background buildings, the meadow first

- **Cottages** (`StageArt._cottage`): half-timbered walls (corner posts, beams, braces), a deep roof overhanging gables and eaves with a
  fascia and a ridge beam, real gable triangles (`_gable`) with a lit round attic window, a door on a stone step with a lantern, a framed
  shuttered window with a flower box, shrubs at the foot, a contact shade, a capped chimney.
- **The windmill** stands in a grassy mound with its shade (it hovered over the hill).
- **Sky islands**: rounded, bulging rock keels in more rings (were faceted pyramids), a second strata band, vines hanging over the lip.
- **Trees** (`_round_tree`, every stage that uses it): four more leaf clumps round each crown and two branches forking into it.
- Images: `docs/screenshots/review4_*.png`.

## Pass 65: the beach's props and the weapons' hilts

- **Palms** (`StageArt._palm`): a trunk of overlapping scales (each ring wider at its top), turned along the trunk's curve and narrowing
  to the crown, a darker joint at each, a flared foot and a contact shade (was a stack of equal drums).
- **The beach umbrella** (`_umbrella`, `_canopy_mesh`): a striped canopy in eight panels, its colour and white, a scalloped hem and a knob
  (was one flat cone).
- **The longsword's hilt**: a round crossguard with ball ends and a block, a round grip in three bands, a gold pommel (was a box guard on
  a box grip). **The greatsword**: a bright bevel along its edge and chisel point, rivets in its guard.
- Images: `review4_beach_props.png`, `review4_sword_hilt.png`.

## Pass 66: the sea wall and the roster's outfits

- **The beach's sea wall** (`seawall` in `_soil_material`): smaller stones (about half the size), corners only just softened, a lit
  bevel along each stone's top and left and a shaded one along its bottom and right, darker mortar, fine pits in the stone and a cooler
  grey in the sandstone; it read as a wall of cushions. `review4_seawall.png`.
- **Every built-in fighter wears a signature outfit** from the existing catalog (`Roster.builtins`): shirts, shorts, gloves or the full
  kit, chosen to its reference (the Tumbler a cap with the full kit, the Settler a straw hat and flower shirt, the Scamp an aviator cap
  and a striped shirt). They were bare colour bodies under their hats. Presentation only; the roster tests pass. `review4_outfits.png`.

## Pass 67: the city's front, the player tags; two review items revisited

- **The city's front** (`facade` in `_soil_material`): a brick apartment wall, four storeys a repeat: brick courses, a stone ledge per
  floor, four tall framed windows a floor with a cross bar (a third lit, curtains drawn to the sides), flower pots on some sills, an air
  conditioner, striped awnings, a drainpipe at the seam (was a flat grid of narrow windows on panels). `review4_city_front.png`.
- **Player tags** a fifth smaller with a thinner outline, so they sit with the rest of the in-match UI instead of shouting over it.
- **Item 5 (contact)**: on checking, every stage prop already sits in its contact patch (`_props` `patches`); the review overstated it.
- **Item 6 (shimmer)**: the Best tier already runs 4x MSAA with SMAA; temporal AA would smear the ink lines. Left as is.
- **The canyon's front** was checked and kept: its broken, waving strata read well at play distance.

Next rounds, in order: 1 (the other stages' props), 3 (weapons), 2 (front walls), 4, 5, 6, 7.
