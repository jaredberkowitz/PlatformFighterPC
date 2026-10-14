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

## Pass 68: chunkier fighters, arms that read (the owner's PEAK reference)

The owner's note: more of the arms visible, chunky limbs as PEAK's, proportions and turns adjusted, kept animated and readable. (PEAK is
a close third-person 3D game; ours is a side-on 2.5D view, so only the proportions and the arm read were taken, not its camera.)

- **The base body** (`make_base_body.py`): arms about 30% thicker and a little longer (`ARM` radii 0.15 / 0.136 / 0.12, the wrist at
  0.62, 0.84), legs about 12% thicker (`LEG` 0.152 / 0.143 / 0.13); bigger mitten hands (`HAND_SCALE` 1.24, `FIST_SCALE` 0.82). Body, joints
  and rig rebuilt; garments follow the body.
- **The idle stance** (`make_rigged_blob.py` `idle`): both chunky arms held out clear of the body, fists a little up and forward (they were
  tucked into a guard in front of the belly, where the body and the weapon hid them).
- **The turn** (`fighter_view.gd`): the body 60 degrees off straight-on (was 70) and the chest 24 (was 35), so both arms and more of the
  chest read while moves still play across the stage.
- Checks: the art, fighter view, motion, animation and creator tests pass; the look bench 0.87 (needs 0.85).
- Images: `review4_chunky_before_after.png` (before above), `review4_chunky_lineup.png`.

## Pass 69: the body rebuilt to the owner's PEAK references

Feedback across several looks: the arms read as sleeves, the body as tubes or a gourd, the head melting into the chest, little torso. The
proportions were changed rather than the old ones forced into the style (`make_base_body.py`, then `fit_body.py`, `make_rigged_blob.py`):

- **Head and torso**: the head a 0.44 ball centred 1.78 up on a short narrow neck (was 0.475, 1.725 and no neck), over a real boxy torso
  as wide as the head and about as tall, square high shoulders, a squarer cross-section (`TRUNK_SQUARE`). Height kept at 2.2.
- **Anatomy as soft forms on one skin** (`ANATOMY`, `SHOULDER_CAP`): deltoids rounding the box into the arm (no sleeve seam), the point
  of the elbow and the forearm's swell, kneecaps and calves, ankles narrowing into a foot with a heel and a toe box; arms and legs shaped
  along their length (full, a joint, filling out again).
- **Hands with fingers** (`mitten`): a palm, four stubby fingers curling in (the middle two longest), a thumb in front.
- **One continuous skin**: QuadriFlow is retried on a coarser voxel skin when it refuses (it had silently left 237,000 triangles, which
  shaded lumpy); the head's ball is weighted wholly to the head bone (the chest's twist stretched its underside); the arms' weights are
  spread further so elbows and armpits bend round. A thin chin line is painted where the ball of the head sits on the body
  (`skin.gdshader` `head_ball`, from `base_rig.json`): the two-tone light draws no line at a soft crease.
- Checks: all the art, fighter, motion, animation, creator, online and content tests pass; the look bench 0.91 (was 0.87).
  `review4_peak_body.png`.

- **The arm's join** (follow-up, the owner: "the arms look like sleeves"): the deltoid ball and the forearm swell are gone (each step in
  width read as a cuff); the shoulder cap lies along the arm's first stretch so the shoulder's top line runs on into the arm, and the arm
  tapers evenly from shoulder to wrist with only the point of the elbow behind. `review4_peak_body_arms.png`.

- **Wrists, gloves, collar, dark bodies**: a wrist plug in the bare hand closes the cut forearm (its inside showed as a dark ring, a
  bracelet), the hand's outline left out where it enters the forearm; the glove's fist on its own, smaller scale (`GLOVE_SCALE` 0.74:
  gloved fighters carried two white balloons); a shirt collar with a button placket (`_shirt_collar`, after PEAK's scout shirts); a very
  dark body (midnight) gets a lighter line in its own hue, so it keeps a silhouette at dusk (look bench 0.87). The review tool takes
  `--shirt=N --gear=N` (`tests/turnaround_shot.gd`). `review4_outfit_turnaround.png`, `review4_wrists.png`.

Next rounds, in order: 1 (the other stages' props), 3 (weapons), 2 (front walls), 4, 5, 6, 7.

## Pass 70: the clothing pass, a waistline, and shoulders that hold in motion

The owner: clothing quality uneven, the model still reads amateurish; PEAK's figures have a waistline; after a playtest, the arms stretch
and do not read as shoulders.

- **A waistline** (`make_rigged_blob.py` `WAIST_SHARE` 0.25, was 0.12; `make_base_body.py` `TRUNK`): the shorts come up to the body's
  narrowest, where the chest now narrows a little and the hips fill out again under it. A **belt** (`Belt`, a band cut from the skin like
  the other garments) with a brass **buckle** (`Buckle`), coloured by `fighter_view.gd` (`BELT_LEATHER`, `BELT_BRASS`); the shirt is
  tucked under it and puffs a little above it (`BLOUSE`).
- **Prints laid straight**: each garment's texture maps are rewritten from its own finished shape (they were the skin's, from before the
  cloth was pushed out and smoothed, so the stripes came out wavy and torn).
- **Collar, placket and buttons modelled on the shirt** (`shirt_front`: `ShirtCollar`, `ShirtPlacket`, `Buttons`), each point set on the
  shirt's own front, so they sit on it in any body: two pointed flaps apart in a V, a shade darker than a plain shirt, white on a print.
  The game's boxes (`_shirt_collar`) stand in only on an older rig; theirs read as a bow tie and sank into the new shirt.
- **Hems**: the sleeves' openings hug the arm (`SLEEVE_SNUG`; no dark inside showing) and are smoothed along their loops (spikes); the
  neckline does not flare (spikes round the throat).
- **Shoulders** (the owner's playtest): the arm had fused to the torso's side down to the elbow, and the arm weights reached over the
  trunk, so a swinging arm dragged the side out in a web like a poncho. The arms hang a little further out over a trunk narrowed under them
  (a real armpit); the trunk gains no arm weight when it is spread (`smooth_weights` `no_gain`), and the arm's pull on the trunk fades
  with distance from the shoulder joint (`release_trunk`, `SHOULDER_HOLD` 0.08, `SHOULDER_FADE` 0.1), on the skin and the shirt. The
  tucked part of the shirt moves with the belt's bones (`TUCK_SPAN`): it swung out under the belt like a skirt in the run's lean.
- The elbow's point sits closer in (the outline showed in the crease beside it). `tests/anim_sheet.gd` takes `--shirt=N`.
- Checks: the fighter, art, loadout, motion, animation, creator and online tests pass; the look bench 0.88.
  `review4_clothing_pass.png` (four shirts), `review4_run_shoulders.png` (the run before, after).

Still open: the hands (the owner: still chunky and odd), shorts' detail (pockets, a hem), shoes.
