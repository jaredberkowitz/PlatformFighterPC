# Visual polish: audit, plan and change report (2026-10-14)

The owner's visual polish iteration plan, merged with the open steps of `docs/ART_REVIEW_2.md`. The rule is to **iterate on what exists**:
fix a parameter, then a shader, then geometry, and remodel or replace only what smaller changes cannot fix. Each change is one variable,
compared before and after from the same camera, and kept only if it helps. Animation, weapon models and modelled props stay deferred to
the end (the owner's call); everything else in the plan is in scope, and anything skipped is listed under "Skipped, and why".

## Baseline (phase 1)

- **Renderer**: Godot 4.7.1, Vulkan Forward+, MSAA. One shared cel light (`shaders/toon_light.gdshaderinc`) for fighters and the stage, a
  post pass for ink, haze, the value hierarchy and paper (`shaders/post.gdshader`), per-stage moods and shading profiles
  (`scripts/lighting.gd`).
- **Lights**: a key with soft-edged cast shadows (two splits, 70 m), a cool fill from behind, flat ambient. No SSAO or SSIL; contact
  darkening comes from `stage_ao.gdshaderinc` and painted contact patches.
- **Fighters**: one skinned base body on a shared skeleton (`art/blender/make_base_body.py` builds it, `fit_body.py` measures it,
  `make_rigged_blob.py` rigs it into `godot/models/base_rig.glb`), clothes painted on by region (`shaders/skin.gdshader`), a 2D face,
  wardrobe pieces from `Loadout`, an inverted-hull outline plus the post pass's ink.
- **Stages**: built in code (`scripts/stage_art.gd`): blocks with a patterned front, grass or sand caps, props, backdrop layers.
- **Captures**: 1280x720. `--demo=stage --stage=N --match --noecb --chars=0,6` (play distance with the HUD) and
  `--demo=stage --stage=N --noui --noecb --chars=2,9 --cam=7` (close-up), every stage, plus `tests/lineup_shot.gd`.

### Found while capturing

- **The collision-box outlines were in every review shot.** With `--noecb`, the outlines were never updated, so all four stood visible
  at the stage's origin (the gold diamond beside the second fighter in earlier reviews). They are hidden until shown now. Shots only;
  play was never affected.

## Ranked opportunities

| # | Opportunity | Responsible files | Without remodelling? |
|---|---|---|---|
| 1 | **Fighter shading** is a soft three-band ramp with strong hue rotation: orange fighters showed orange, salmon and pink streaks. | `toon_light.gdshaderinc` | Yes (done, pass 52) |
| 2 | **The light/shadow edge on fighters is ragged**: the base body's decimation left long thin triangles whose normals jag the edge. | `make_base_body.py` | Needs a topology change (done, pass 52) |
| 3 | **The meadow platform**: lawn patches read as overlaid rectangles, the earth front repeats the same rounded panels, the lip is a hard line. | `stage_art.gd`, `toon_common.gdshaderinc` | Yes |
| 4 | **Hero props are flat**: the hay bales' checker reads as a decal, the barrel has no rim or bands, the fence is thin, bright and busy behind fighters, the windmill's sails are paper-thin. | `stage_art.gd` | Yes (geometry built in code) |
| 5 | **Ink line noise** inside fighters (doubled strokes, dashes between the legs) and contact/grounding of feet. | `post.gdshader`, `outline.gdshader`, `fighter_view.gd` | Yes |

After these: material families written down as standards (phase 6), the backdrop hierarchy and the floating islands (phase 7), motion
checks (phase 8), and a final grading and performance pass (phase 9).

**Is a character model change warranted?** Yes, one: the topology (item 2). The proportions and silhouette follow the concept sheet and
read well; the faces, wardrobe and moves depend on them, so they stay. Hands are the next candidate (mitten shapes read lumpy close up).

## Change report

### Pass 52: two-tone fighters and a clean body surface

- **Shading** (`toon_light.gdshaderinc`): fighters drop the stage's three-band ramp for a hard step between two flat tones
  (`PF_FIGHTER_STEP` 0.62, anti-aliased over `PF_FIGHTER_EDGE` 0.02). Their shadow takes a third of the stage's hue turn
  (`PF_FIGHTER_HUE`), so an orange shades deeper orange, not pink. The shadow tone is a step deeper (`PF_FIGHTER_SHADE` 0.8, which
  replaces `PF_FIGHTER_LIFT`), because the two tones need a clear value step to read as shapes at play size. The stage keeps its ramp.
  Tried and rejected: the step at 0.5 (the shadow was a sliver) and at 0.8 (too much of the face in shade).
- **Body topology** (`make_base_body.py`, `REDUCE = "quad"`): the body is reduced with a QuadriFlow quad remesh plus four smoothing
  steps instead of edge-collapse decimation. Same budget (6,386 triangles against 6,498), joints within a millimetre (refitted, rigged
  again; hats, glasses and weapons fit as before). Under the hard step the light/shadow edge is now one smooth curve.
- **Images**: `docs/screenshots/polish1_twotone_city.png`, `polish1_twotone_meadow.png` (before left, after right),
  `polish1_body_terminator.png` (step only on the left, step plus the new topology on the right), `polish1_lineup.png`.
- **Checks run**: fighter view, animation library and art pipeline tests pass; the look bench's worst silhouette holds at 0.89 (needs
  0.85); close-ups on the meadow and the night city, and the lineup.
- **Still open**: on the meadow, the key sits near the camera, so fighters show only a narrow shadow side there (per-stage mood, left
  as is for now); the fists' shading shows their own facets.

### Pass 53: one clean silhouette line on fighters (step 3 of docs/ART_REVIEW_2.md)

- **Problem**: a fighter's silhouette had two lines, the inverted-hull outline just outside it and the post pass's screen-space ink
  just inside it. Together they made a thick, dotted, wavering stroke at play size. Dropping the post line on fighters outright left
  blades and the shield rim with no line, and the hull alone was faint up close.
- **Fix**: the hull outline writes its own object id (`HULL_ID` 0.7; `outline.gdshader` is now emission over black so it can write
  one), and the post pass draws no line on or beside a hull line, only where a silhouette has none. The hull's minimum width is a
  little higher (0.02 to 0.028 world units) so it holds up close.
- **Images**: `docs/screenshots/polish2_silhouette_line.png` (before left, after right, play distance, 4x).
- **Checks run**: all five stages at play distance and close up; the look bench's worst silhouette 0.89 to 0.90. Performance (the KO
  demo, warm shader cache, alternated runs): 383 fps against 391 before, GPU 1.71 ms against 1.66 ms. The first run after a shader
  change hitches while the cache rebuilds; that is not a cost in play.

### Pass 54: the meadow platform (phase 4; step 6 of docs/ART_REVIEW_2.md for the meadow)

- **The front**: the meadow and the beach shared one masonry tile, a grid of rounded panels. The meadow now has its own `earth` pattern
  (`stage_art.gd`): broad beds of farm soil with soft waving seams, half-buried stones of very different sizes (a few boulders, more
  middling stones, many pebbles; lit tops, shade under them), fine roots hanging from the turf, grit. It is drawn at 512 px and laid
  every 12 units, so no repeat shows across the stage. The beach keeps the masonry, which suits a sea wall. First try had a boulder the
  size of a fighter right under the fighting; the boulders are smaller and closer in value to the soil now.
- **The lawn's patches**: long, hard-sided darker rectangles on the lawn were the platforms' cast shadows (confirmed by switching the
  stage AO off: they stayed). A plank three units up cast a shadow as crisp as a fighter's foot. The key light's angular size is now 5
  degrees (from 0.35), so a shadow softens with its distance from what casts it: contact shadows stay crisp, the platforms' shade
  becomes a soft pool. (2 degrees changed nothing visible; 10 lost the platforms' shade altogether.)
- **The stage AO under platforms** (`stage_ao.gdshaderinc`, found on the way): the baked map is flat (across and up only), so on the
  ground it shaded the whole depth of the lawn under a platform, back to front. Ground (a surface facing up) now takes it only under
  the platforms' depth (`pf_ao_depth`, from `StageArt.PLATFORM_DEPTH`), fading over a unit. Fighters and walls are unchanged.
- **Images**: `docs/screenshots/polish3_meadow_platform.png` (the platform and front, before left), `polish3_lawn_shade.png` (the lawn).
- **Checks run**: every stage at play distance and close up; the look bench (0.90); match flow, fighter view and art pipeline tests.
  Performance could not be measured cleanly this pass (the machine was busy: the GPU time swung between 1.8 and 5.5 ms with either
  shadow setting); to be measured again.

### Pass 55: the meadow's hero props (phase 5, first of each group)

- **Hay bales** (`_hay_bale`): were 12-sided drums with the straw hat's weave wrapped over them, ends included, so they read as
  checkered boxes. Now a lathe (`_lathe`, a new helper): a drum whose rim rounds over into the ends, strands of straw running round
  the body and a rolled spiral painted on each end (`_straw_texture`, drawn in code), and two thin bands of twine.
- **Barrel** (`_barrel`): a straight cylinder with flat bands. Now a bulging lathe of painted staves (`_stave_texture`: twelve boards,
  dark seams, lit edges, grain), iron hoops sized to the bulge where they sit, a rim of staves standing above an inset, boarded lid.
- **Windmill** (`_windmill`): the canvas sat in front of the lattice, so the sails read as four flat white boards. The canvas is now
  stretched behind the frame (rails and slats over it, as a common sail is built), with a leading board on the other side of each
  stock, and the hub is a drum with an iron nose on a windshaft that comes out of a housing on the tower, not a disc floating in
  front of it.
- **Fence**: sturdier posts of slightly different heights, each leaning a touch, and rails set bay by bay, each a little off level,
  instead of two perfect boards the length of the stage. Kept as quiet as before behind the fighters. (It draws from the meadow's
  random sequence, so the flowers placed after it moved.)
- **The earth front's tiling** (found while checking): each soil bed's seam started at x 0 without its wave (a step at every repeat),
  and the beds had straight bottoms with bare bands between them. Seams start on the wave now, each bed runs down under the next, the
  last one wrapped to the top, and the beds' shades are closer together. Checked by drawing the tile 2x2.
- **Images**: `docs/screenshots/polish4_bales.png`, `polish4_windmill.png`, `polish4_meadow_props.png` (before left).
- **Checks run**: the meadow close up and at play distance (1080p), match flow and art pipeline tests.

### Pass 56: trees, sky islands, turf edges (phase 5, the rest)

- **Trees** (`_round_tree`): every crown was the same big mass with a smaller one to its right. Now one of three shapes (a side lobe
  either way, a tall stacked crown, a wide crown with two shoulders), chosen from the tree's own position so nothing else moves.
- **Sky islands** (`_island_detail`): the turf over the edge was seven equal balls in a row (a bead necklace). Now nine tongues of
  different widths and lengths, overlapping and sunk into the lip. The roots, dark sticks that read as posts, are thin, tapering and
  bent, in two lengths each, closer to the earth's colour.
- **The stage's own lip** (grove, and any grass cap without moss): the same bead row, under the fighters. Each tongue now has its own
  width and hang (seeded from its place, so the layout holds).
- **Images**: `docs/screenshots/polish5_islands.png`, `polish5_grove_lip.png` (before left).
- **Checks run**: the meadow and the grove at 1080p, match flow test.

### Pass 57: background hierarchy (phase 7)

Every stage checked in greyscale at play distance.

- **The value hierarchy was off in every match** (found here): `Lighting.style()` merged a stage's profile over the default one key at a
  time, so a stage with its own `finish` (all five do) replaced the whole finish and lost the default's `quiet` (pass 50's quieter
  stage) and its vignette. The finish now merges value by value. With it on, the fighters are the brightest, crispest thing on every
  stage and each stage keeps its mood.
- **The city's skyline** was a field of bright windows right behind the fighters. Lit windows are fewer on the near layers
  (`FACADE_LIT_SHARE` 0.24, 0.28, 0.36 from 0.42) and a little dimmer (`FACADE_LIT` 2.4, 1.9, 1.5 from 3.0, 2.3, 1.7).
- **The canyon**: its centre butte matched the stage top in value. With the hierarchy back on it settles back, so nothing else changed.
- **Images**: `docs/screenshots/polish6_value_hierarchy.png` (meadow, beach, city; before left).
- **Checks run**: all five stages before and after; the look bench; match flow.

### Pass 58: material standards (phase 6) and motion (phase 8)

- **Material standards** are written down in `docs/VISUAL_STYLE.md` ("Material standards"): which family each kind of surface uses,
  its light response, colour variation and line, the same on bright and dark stages. The new hay bales and barrel had bypassed the
  families; they now take the straw (grass) and wood responses (`Materials.as_family`).
- **Motion** (the combat, smash and kicks demos): hits, launches, trails and feet on the ground read. The gap: a jumping fighter's drop
  shadow had faded to nothing a few units up (it faded over half its 10-unit reach), so where it would land was not marked. It reaches
  14 units now and fades only over the last fifth (`SHADOW_DEPTH`, `SHADOW_FADE`).
- **Images**: `docs/screenshots/polish7_drop_shadow.png` (a neutral air, before left).
- **Checks run**: the motion demos; the fighter view test.

### Pass 59: effects to the knock-out blast's standard

The owner's call: the knock-out blast (layered painted smoke with an ink silhouette, a deep body, a lit top and a white-hot core) is the
standard for every effect.

- **Swing arcs** (`fighter_view.gd` `_draw_arc`, `_arc_band`): were a soft gradient crescent fading to nothing on the inside, hard to see
  on pale skies and over a fighter of the trail's colour. Now painted bands side by side: an ink edge, a lit band, the body in a
  deeper version of the class colour, an ink edge along a scalloped (brushed) inside, and a hot core near the head. They are
  see-through (`ARC_OPACITY` 0.82, a new `opacity` on the effect shader: painted effects spend their alpha on the ragged breakup) and
  still break up in painted chunks as they fade. Tried and dropped: paint flecks off the tail (they read as a chain of beads), and
  fully solid (it read as a shape, not a trail).
- **Punch and kick smears** (`smear.gd`): the same bands, with a rounded head at the fist or foot.
- **Hit bursts** (`effects.gd`): their body colours sat above the effects' glow threshold, so the whole burst bloomed to white and its
  ink and colour were lost. The flare and body are under it now, and only the core glows. The strong hit's speed lines (white slivers)
  are inked and in the hit's colour, so a hit reads as the kind that landed it (a blue blade, violet claws, a warm blow).
- **Images**: `docs/screenshots/polish8_trails.png` (a sword arc and a claw hit, before left).
- **Checks run**: sword, claw and maul effect demos on the meadow and the city; the fighter view test.

### Pass 60: the backdrop to the stage's finish (the meadow first)

The owner's note: the stage and the props by it now look clearly higher quality than the scenery behind, and the styles differ. The
backdrop is brought to the same finish: painted surfaces (tiles with detail, as on the stage), built construction, ink lines that hold.

- **Ink on far scenery**: the outline's width was capped at 0.11 world units, so beyond about 50 units every backdrop object lost its
  line and read as a different, line-less style. The cap is 0.2 (`max_width`), so the line keeps a steady weight on screen out to the
  hills.
- **Two new painted tiles**: `plaster.svg` (soft patches, hairline cracks, chips: whitewashed walls) and `shingles.svg` (rounded
  shingles in rows, a dark gap and a lit edge on each), as `Materials.plaster` and `Materials.shingles`.
- **Cottages, the barn, the windmill**: plastered walls, shingled roofs, stone footings, eave boards, framed doors and windows with
  shutters, the barn's corner boards. Built like the props by the stage instead of boxes in flat colour.
- **The valley's fields**: were flat colour quads ruled off by hard green lines. Now each field is painted with a new `field.svg` tile
  (crop rows with tufts, soil detail over it) in its own colour, and the hedgerows are rows of leafy bushes of different sizes
  (foliage material, inked), along and between the fields, over a thin shade line. Tried and dropped: the grass tile on the fields
  (its big soft blobs read as smudges at that size) and evenly spaced bushes (a bead chain).
- **The beach**: the distant isles were flat-colour ellipsoids. Their sand is the stage's sand, their green leafy foliage, the hut on the
  middle one plastered and shingled. (The haze softens all of it at that distance; the ink and the materials now match the stage.)
- **Images**: `docs/screenshots/polish9_meadow_backdrop.png` (the whole meadow from wide, before top).
- **Checks run**: the meadow wide and at play distance; art pipeline and match flow tests.

### Performance after passes 52 to 56 and character pass 1

The KO demo at 1280x720, warm cache: GPU 1.89 ms against 1.66 to 1.71 ms before (soft shadows, lathe props, the earth tile), about 255
fps. Runs while the machine was busy (OneDrive uploading the build) fell to about 50 fps with the GPU still under 5 ms. That is outside
contention, not the game.

## Skipped, and why

Nothing yet.

### Pass 61: the Wanderer's sword wave

`main.gd` `_draw_blade_wave` (class 9, forward smash, sim 102): a horizontal blade of light 1.75 long, pointed at both ends and thickest
toward the front, painted in the effect standard's layers (ink, the longsword's blue, a pale band, a hot core), its edges flickering on
twos. `--demo=blade_wave`. Its size follows the charge (x0.8 to x1.1) through the bridge's `projectile_scale`.
Image: `docs/screenshots/polish10_blade_wave.png`. Also: the three new stage tiles' `.import` files use the `keep` importer like the
other tiles (the art pipeline test asks for it).

### Items: a placeholder look (sim 111)

Until the modelled items (scheduled near the end), every item slot is drawn so players can see it: the bomb a dark navy ball with a
steel cap, a fuse and a spark that flashes red for its last 45 frames, bursting in the painted fire burst; the arrow a shaft with a
steel head and red fletching. Held items sit at the holder's hand side. `main.gd` `_build_items`, `_update_items`; `--demo=bomb`.
A flying arrow points along its velocity (`item_vel`), a lodged one at the angle it flew in at (`item_angle`, sim 113; 35 degrees down the way it flew if the sim gives none), and a thrown bomb spins about its middle by the frames since the throw (`item_flight`).

### The Bounder's egg and thrown ball

- **The egg** (the tongue and lay, the sim's `Encased`; `fighter_view.gd` `_egg`, radius from `fighter_egg_radius`): a cream egg with
  green spots standing on the fighter's feet, the fighter hidden inside, rocking as it struggles, breaking into shell pieces
  (`Particles.shell_pieces`) as it jumps out. `--demo=ball`.
- **The up special's thrown egg is a basketball** (the owner's call; `main.gd` `_basketball`): orange rubber, inked, with its seams,
  spinning the way it flies, popping in a warm painted burst. `--demo=basketball`.
- Thrown things and items now carry an object id in the fighters' range (`PLAY_OBJECT_ID`), so the stage's quieting no longer washes
  their colour out.
- Image: `docs/screenshots/polish11_ball_and_egg.png`.

### Pass 62: from vector art to a lit world (the owner's "not a Flash game" brief, toward a high-definition cel-shaded adventure)

What read as cheap was not detail but light: every stage lit evenly from near the camera (cast shadows fell straight back, out of
sight), one even dark stroke round everything, large surfaces in one flat value, white specks on a blue sky reading as snow. The pass:

- **The sun from the side and a little behind** (`Lighting.KEY_SIDE` -28 to -80). Props show a lit side and a shaded side, cast shadows
  fall across the fighting surface where they are seen, and the stage's front wall turns into its painted shade, so the lit play
  surface leads and the mass under it settles back. The canyon's light comes from its painted sun, now moved to the left of the sky
  so the two agree (backlighting from the right cost the darkest body colour its silhouette in the style bench, 0.84 against 0.85).
- **A painted world, inked fighters**: the scenery's line is a soft violet-grey (`outline.gdshader` `scenery_color`) and the screen-space
  ink on the stage and scenery is at 0.4 (was 0.72); fighters keep full dark ink, the clearest things on screen.
- **Cloud shadows** (`pf_clouds`, per stage in `Lighting.STYLES`): soft patches of shade drifting over everything facing up (the lawn,
  the sand, the props' tops), painted in the surface's own shadow colour; the grove's are small and slow, a canopy's dappled light.
  Not on fighters (a passing shade read as the fighter dimming), not on walls (they showed as slanted stripes), none at night.
- **A soft bloom** (`post.gdshader` `bloom`, from the screen's small mip levels): the sky, clouds and sunlit whites bleed a little light,
  the glow of the high-definition remaster of a cel-shaded classic; fighters take half.
- **The meadow's earth** a richer, darker brown (it was a pale sand that competed with the lawn); **pollen** golden, smaller, fewer.
- Tried and dropped: a stronger value and saturation finish (the lawn went neon: the cheap look, not the rich one), stronger far haze
  (milky, no gain at these distances), a glazed gloss on the fighters (a pale blotch on the hair; not in the style).
- Images: `docs/screenshots/polish12_light_before_after.png` (the meadow, before above), `polish12_canyon.png`, `polish12_stages.png`.

### Pass 63: the sky to the stage's fidelity (the owner: "the background clouds and sun can look a bit flat")

- **Clouds** (`shaders/cloud.gdshader`, `StageArt._clouds`): lit by hand from the stage's sun (`pf_sun_dir`, set from the key light):
  three flat tones (sunlit white drawn a little over white so the tone curve leaves it white, a soft mid, a clear lavender-blue shade),
  a belly darkening to the flat base, a silver lining round the edge (strongest against the light), lumpy edges from a slow noise. Three
  kinds so the sky is not one stamp repeated: domed heaps with smaller puffs piled on the crown, long low banks, tall towers.
- **The sun** (`shaders/sun.gdshader`): one card in place of the stepped discs and halos: a hot core that reaches the bloom, a gold disc
  deepening to an orange rim, a soft corona and faint rays turning slowly. The rose and lilac cloud bands still cross its lower half.
- **The sea's horizon**: the flat white cloud-bank card is now a bank of modelled cumulus sitting on the horizon behind the far coast.
- **Ridges** take a lit flank and a shaded flank from the sun (`painted_ridge.gdshader` `flank`), on the canyon's mesas. Tried on the
  meadow's far range and dropped: on a pale, hazy silhouette it read as a grey band over blue stripes; a flat hazy far range is the
  convention of the style anyway.
- Images: `docs/screenshots/polish13_sky_before_after.png` (the meadow wide, before above), `polish13_sun.png`, `polish13_horizon.png`.
