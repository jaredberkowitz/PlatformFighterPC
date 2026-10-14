# Art review: where we stand against our references (2026-10-13)

An art director's whole-game review of the current build (main at `8edcfd5`), against the inspirations in the ship-polish brief and the
genre's market leaders. The game was captured fresh at play distance on all five stages with the HUD, plus a close-up, a heavy hit and a
knock-out (`docs/screenshots/review_*.png`).

The comparisons are made from knowledge of the reference games, not side-by-side captures: no reference footage was put in the repo.

## The references, and what each one is the bar for

| Reference | What it sets the bar for |
|---|---|
| **The reference cel-shaded adventure (HD)** | Cel lighting with hard, confident shadow shapes; graphic, readable silhouettes; bespoke, hand-shaped props; crisp distance (atmosphere is colour, not blur); big expressive effects (smoke balls, flame). |
| **PEAK** | Appeal from one simple shared body: the charm comes from proportion, expression and physical animation, with identity carried by outfits and colour. |
| **The reference platform fighter** | Fighter readability and identity at small size; impact: hit sparks, hitlag, the KO blast; stage silhouettes designed for play; a polished, cohesive UI. |
| **Rivals of Aether II** | A stylized 3D fighter at our budget: strong, varied silhouettes; chunky painterly stages with sculpted edges; clean, punchy effects; consistent cel shading. |

## Scorecard

Rated against those bars, 1 to 5, at play distance.

| Area | Score | In one line |
|---|---|---|
| Palette and cohesion | **4** | Saturated, warm and consistent; every stage has its own mood and they still look like one game. |
| UI, HUD and menus | **4** | Clear, like the reference platform fighter hierarchy; one card language across screens since this week. |
| Stage backdrops (layout and depth) | **3.5** | Lived-in and layered (farms, streets, valleys); real ground under everything now. |
| Lighting and form | **3** | Fighters now show a lit and a shadow side; the stages are still lit softly and evenly, without the reference adventure's hard shadow shapes. |
| Combat effects | **3** | Painted hits, smoke and arcs read well; the knock-out lacks a signature moment. |
| Stage construction (the fighting surface) | **2.5** | Every stage is a rectangular slab with a tiled front. |
| Prop and backdrop asset fidelity | **2.5** | Charming but built from primitives (balls, cylinders, boxes); it reads as blockout next to the reference adventure's hand-shaped props. |
| **Fighters: identity and surface** | **2** | One body in different colours and hats, with a blotchy skin surface; the weakest area, and the most important one. |

## The weakest areas, ranked by impact

### 1. Fighter identity and surface (the biggest gap)

Fighters are what a player looks at, so this matters most. `review_meadow.png`, `review_closeup.png`.

- **Silhouette.** Every fighter is the same round blob body. At play distance the Duelist, the Brawler and the Rogue differ only by colour
  and a hat. The reference platform fighter and Rivals II give every fighter a silhouette you can read in black. PEAK proves a shared body *can* work, but it carries
  identity through big costume shapes (packs, hoods, scarves) and strong animation, which we do not have yet. The Hauler's forearms are the
  only real silhouette change.
- **Skin surface.** The painted brush texture on the body reads as blotches and dirt at play size (look at the orange fighter's chest and
  arms in `review_closeup.png`), and the sheen makes the skin look like plastic. The reference adventure's skin is flat colour, a hard shadow shape and
  nothing else. At this size, less surface texture would read as higher quality.
- **Weapons.** The Seer's staff reads as a giant cotton swab (a plain stick with a white ball). The longsword and the greatsword are flat,
  untextured blades with no edge highlight or detail. Rivals II weapons are chunky and graphic, with a coloured accent and a lit edge.
- **Faces** are good up close but vanish at play distance. That is fine and matches the genre.

### 2. Stage construction: the fighting surface

`review_beach.png`, `review_meadow.png`, `review_city.png`.

- Four of the five stages are the same shape: a flat-topped rectangular slab with a vertical front and a square-cornered silhouette (the
  grove's stepped underside is the exception). The reference platform fighter and Rivals II stages are *designed shapes*: tapered or sculpted undersides, lips,
  overhangs and rounded ledges that frame the play space and read instantly.
- The front is one texture tiled across a big flat wall. The beach's masonry is the meadow's, recoloured, and its repeat is obvious. A big
  flat wall is also the largest single thing on screen, so it costs the most when it looks generic.
- Ledges have no visual treatment. They are a core gameplay landmark, and the references mark them with a lip, a corner stone or a
  change of material.

### 3. Backdrop asset fidelity and atmosphere

`review_canyon.png`, `review_grove.png`, `review_beach.png`.

- Backdrop props are assembled in code from primitives. At a distance this works, and it is charming, but up close (the windmill, the
  cottages, the trees) it reads as greybox next to the reference adventure's hand-shaped props: no silhouette breakup, no tapering or bevels, the
  same sphere crown on every tree.
- Atmosphere sometimes turns to mud rather than colour:
  - the canyon's orange-pink haze flattens the mesas into one low-contrast mass;
  - the grove's lower canopy dissolves into a pale wash;
  - the beach's horizon has a soft, smeared band.

  the reference adventure keeps the distance crisp and lets colour, not blur, do the receding.

### 4. Combat effects: the signature moments

`review_hit.png`, `review_ko.png`.

- The everyday effects (painted hit bursts, smoke rings, sword arcs) hold up.
- The **knock-out** is the weak moment. The painted beam reads as a flat striped banner (a flag) laid across the screen, not a blast,
  and it has no impact frame.
- the reference platform fighter's KO, a coloured explosion column with a freeze, a flash and a camera kick, is the moment players remember. Rivals II's is a
  clean, punchy burst. Ours should be the most spectacular effect in the game; today it is the least convincing.
- Hit sparks on light hits are small next to the reference platform fighter's, so medium hits could stand to be bigger.

### 5. Lighting drama (moderate)

- The new form light gives fighters volume, and the city's light pools help.
- Stages are still lit softly and evenly. The reference adventure's look comes from **hard, graphic cast-shadow shapes**: trees, eaves and platforms
  throwing crisp dark shapes on the ground. Our platform shadows on the floor are soft and faint.

## What is already at the bar (keep it)

- **Colour and mood.** Five distinct, saturated palettes that still read as one game.
- **Readability.** Fighters stand out on every stage (look bench 0.89); the HUD is clear.
- **The lived-in backdrops.** Farms, streets, a stilt hut, chimney smoke: more world-building than most indie fighters show.
- **The menus**, since the redesign.

## Recommended order of work

1. **Fighter surface (quick, high impact).** Drop or soften the brush texture on skin, cut the sheen, and rely on the flat colour plus the
   form light and a hard shadow step. Restyle the weapons: a graphic blade with a lit edge and a coloured fuller, and a staff head with a
   designed shape (a crystal, a ring or a crook) instead of a ball.
2. **Fighter identity (larger).** Per-class silhouette pieces in the cosmetic layer: pauldrons, a cape or a scarf that streams, packs,
   big gloves, hair shapes, so each built-in fighter reads in black. This is the PEAK route and keeps the shared body. It sits partly in
   animation, which is scheduled for later.
3. **Stage silhouettes and ledges.** Sculpt each stage's slab: taper or carve the undersides, round or chip the top corners, give the
   ledges a lip or corner stones, and break the front with two or three large features (an arch, a root, a buttress) so the tiled wall is
   not the whole face.
4. **The knock-out moment.** Rebuild it as a volumetric painted blast (a column of layered smoke and colour with a hot core) plus an impact
   freeze-flash; scale medium hit sparks up.
5. **Hard cast shadows.** A crisper, darker shadow ramp for cast shadows on the stage, so platforms and props throw graphic shapes.
6. **Backdrop atmosphere and assets.** Pull the canyon's haze back, firm up the grove's canopy and the beach's horizon; later, replace
   the most visible primitive props (trees, the windmill, cottages) with authored meshes (a Blender script or TRELLIS, asked first).
