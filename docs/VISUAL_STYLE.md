# Visual style: the rendering framework

How the game is drawn, so every new fighter, stage and screen inherits the same look. Presentation only: none of this reaches the
simulation. Sister documents: `ART_DIRECTION.md` (what things look like), `ART_PIPELINE.md` (how models and faces are made),
`MENUS.md` (the menus' painted style), `COMBAT.md` (hit feedback).

## 1. One cel material family

Every character and stage surface uses the soft cel material (`shaders/toon.gdshader`, the shared body in `toon_common.gdshaderinc`, the
light in `toon_light.gdshaderinc`); variants only add to it (`toon_cloth.gdshader` draws both sides for garments, `skin.gdshader` paints
socks and shoes, `limb.gdshader`, `face.gdshader`). Lighting is not physical:

- **The shading ramp** (`pf_ramp`): light steps through the stage's bands, lit, a midtone and a painted shadow, read from a 256-step lookup
  drawn from stops (`Lighting.STYLES[...].ramp`), so the transition is art-directed rather than one threshold.
- **Painted shadow**: the shadow colour is the surface's own colour turned toward the stage's shadow hue, a little darker and richer;
  pale colours take a cool tint instead of going grey (`pf_painted_shadow`).
- **Deep shade**: where a cast shadow falls on a side already turned from the light, the shade deepens (`pf_deep_shade`).
- **Rim and edge light**: a rim in the sky's colour on the lit side; fighters also get a pale edge light all round (`pf_edge_light`,
  strong on dark stages) and keep their shadow side a little lighter (`PF_FIGHTER_LIFT`), so no fighter sinks into a background.
- **Painted variation**: tops lean warm and undersides cool (`pf_paint_*`; fighters take a third), and world-laid stage textures drift
  slowly in lightness and hue so large areas are never one flat tile.
- **Solid mass**: a stage block's front falls gradually into a cool shade below its top edge (`front_fall`, `front_span`; each block
  passes its top as the `front_top` instance uniform), so the ground reads as one solid body with a lit rim, not a flat repeated pattern.
- **Contact shading**: every prop and piece of furniture standing on the stage sits in a soft dark patch (`StageArt._contact`). Screen-
  space ambient occlusion does not show under this cel light, and the baked stage shading (`scripts/stage_light.gd`) only knows the blocks
  and platforms, so this is what keeps props from floating.

## 2. Outlines

- **Fighters**: an inverted hull (`shaders/outline.gdshader`), a constant width on screen, tinted with a little of the fighter's own colour
  (`outline_tint`). Garments use a thinner hull set further back so folds do not show through; masks in vertex colours leave the line out
  where cloth covers skin and round the armpits.
- **Stage and scenery**: the screen-space pass (`shaders/post.gdshader`) draws lines where objects meet (object id) and where depth jumps,
  creases on scenery only, thinner and fainter with distance. Lines inside one fighter are lighter than its silhouette (`inner_lines`).
- Effects, the shield and numbers are drawn after the outline pass, so they stay clean.

## 3. Stage profiles

Each stage theme has a **mood** (`Lighting.MOODS`: key and fill light, ambient, shadow hue, rim, colour grade, haze) and a **shading
profile** (`Lighting.STYLES`: ramp, deep shade, edge light, painted gradient, glow strength and the finishing pass's value range,
`finish`), applied together by `Lighting.stage_mood`. A new stage picks a theme and inherits all of it; it overrides only what it needs.

The value range per theme (`finish`, graphics phase 2): `desat` (eases colour toward grey, 0 by default), `lift` (lifts the darks toward
the haze, 0.03), `vignette` (0.16), `haze_max` (the far scenery's most haze, 0.22) and `s_curve` (a gentle S-curve on the 0..1 range, 0.22;
lit windows above white keep their glow); fighters take half of the pastel and the curve. The city curves more (0.48) and lifts less (0.02)
for a deep night. `glow` sets the bloom (0.75 by default, 0.95 night, 1.1 city).

### Stage surfaces and scenery (graphics fidelity pass)

- **Facades** (`shaders/facade.gdshader`): the city's backdrop buildings, one mesh and one material per layer, windows painted on in fitted
  bays (frames, sills, parapets, floor bands, rooftop boxes), lit ones HDR (3.4 / 2.5 / 1.8 times white from the near layer back) so the
  near ones bloom. Light sources get halos (`StageArt._halo`, see-through cards drawn after the finishing pass).
- **Painted detail** (`detail_*` in `toon_common.gdshaderinc`): world-laid stage surfaces only, two scales of a hand-drawn grey tile
  (`godot/art/stage/detail_concrete.svg`, `detail_soil.svg`, `detail_stone.svg`) varying the lightness and a little the hue; off unless a
  material sets it (`Materials.TILE_DETAIL`, the block fronts in `StageArt._soil_material`).
- **Foliage** (`shaders/toon_foliage.gdshader`): leaf-cluster cards (`leaf_cluster.svg`, `grass_tuft.svg`) over solid cores, alpha to
  coverage, normals bent from each leaf mass's middle so crowns shade as volumes, light through the leaves (`PF_TRANSLUCENT` in
  `toon_light.gdshaderinc`) and wind (`pf_wind`, a shader global). `StageArt._leaf_cards`, `_tufts` (Balanced and Best only).
- **Water** (`shaders/water.gdshader`): banded colour, drifting streaks, swell, glints and depth-texture foam where things stand in it; drawn
  after the finishing pass at priority -50, so it hazes itself; Fast keeps the bands only.
- **Weathering**: decals for the city roof's stains and puddle (`roof_detail.svg`) and for scuffs left by heavy hits (`scuff.svg`, a pool
  of six in `main.gd`, cleared each match, none on Fast).

### Characters and stage refinements (visual overhaul, `docs/VISUAL_OVERHAUL_LOG.md`)

- **Material responses** (`FighterView.RESPONSES`, `respond()`): skin the stage's own crisp step (a clear lit side and a cool shaded side, as
  a cel-shaded adventure draws bodies; it was a softer, airbrushed ramp before the ship-polish pass), gloves and shoes rubbery (a firmer step, a soft
  sheen), cloth matte, the blade and guard metal (`metal` in `toon_light.gdshaderinc`: a crisp glint band and a stronger rim). The sword's
  blade is a bevelled, pointed shape (`_blade_shape`), so its two-tone bevels read as steel.
- **Skyline buildings**: each has its own pattern (bay, pane, lit share, shade; one in five office strips); a few windows switch on and
  off; lit neon signs and two rooftop billboards (a sign face kind in `facade.gdshader`); the far layer windows only.
- **Stage detail**: legible roof pavers, three-part girders with a lit top edge, built props (the water tower's hoops and ladder, framed
  crates, the air conditioner's grille).
- **Hits**: three tiers (a spark; a spark and a short dart; a burst, ring and long spear), the strongest kept from covering its target.
- **Player tags**: in a match, "P1" over a pointer in the player's colour, one size on screen at any zoom (`FighterView.TAG_PIXEL`), so a
  fighter far off is still easy to find.
- **HUD**: cards 4% above the bottom, a white flash on the damage number when hit, scaled to fit four players.

Shader inventory, stage side: `toon.gdshader` / `toon_common` / `toon_light` / `brush` / `stage_ao` (surfaces), `facade.gdshader`,
`toon_foliage.gdshader`, `water.gdshader`, `painted_ridge.gdshader`, `fx.gdshader` (halos and effects), `post.gdshader` (finishing pass).

## 4. Atmosphere and post

**Clear and vivid, not milky** (ship-polish pass): far scenery hazes toward each stage's own saturated haze colour (`MOODS[...].haze`),
never toward white; the paper tint is light (0.04); colours are not eased toward grey by default; the mist layers between scenery
layers are thin; the sky's bands sit low (`mid_at` 0.035, `top_at` 0.13 in `sky.gdshader`) because the match camera only sees the
lowest few degrees, so each stage's rich top colour shows. Clouds are cel-lit cumulus (`StageArt._clouds`): a warm lit top and a
shaded underside in the stage's shadow colour, on a flat base. Targets: the clarity of a cel-shaded adventure, the painted skies of the
concept art, the readability of a modern platform fighter.

The post pass adds distance haze toward the stage's haze colour (from 4 units behind the stage plane, so the nearer scenery layers already
separate from the farther ones, up to 36% far back; the stage and its props are untouched), the pastel finish (fighters get half, so they stay the most vivid
thing), a paper tone and grain, a light vignette (0.16: dark corners hide fighters near the edge) and the stage's grade. The grade is
restrained: fighters take a third of it, so a white glove stays white on a dusk or night stage, and no stage washes everything in one hue
(the city's purple stays in its sky and buildings; its roof is pale stone). Glow is kept for
effects (`Lighting.FX_GLOW`) and HDR lights (the city's windows). Far scenery is softly out of focus in play on the Best tier (`far_blur`
in the finishing pass, 2.6 pixels at most from 14 units behind the stage plane): done there rather than by the camera so the effects, name
tags and halos drawn after it stay sharp, and depth-aware so the fighters never bleed into it. The results screen adds the camera's own
depth of field (`RESULTS_BLUR`).

## 5. Combat effects

`scripts/effects.gd` (drawn bursts by weapon, impacts, rings, the DI line) and `scripts/particles.gd` (sparks, dust, embers, knock-out
confetti) share conventions: bigger and longer for stronger hits, drawn on twos like hand-drawn effects, warmed up at start so the first
hit does not stutter. The Effects setting thins particles evenly (`Burst.density`).

## 6. Comfort and scalability

The Controls screen's Display page (`scripts/visual_settings.gd`, saved in `user://display.cfg`): **Screen shake** (off, low, full: scales
the shake, roll and punch-in), **Flashes** (off, soft, full: the full-screen flash; the finishing blow's two-tone frames only at full),
**Effects** (low, medium, full: particle counts) and **Quality** (fast: no shadows, anti-aliasing, glow, grass tufts or hit scuffs, and
the sea's bands only; balanced: 2x MSAA plus SMAA; best: 4x MSAA plus SMAA and the far blur). SMAA smooths what MSAA cannot reach: the
finishing pass's ink lines and the leaves' cut-outs (FXAA was compared and softened every drawn detail). The tone mapper is Filmic
(ACES and AgX compared: ACES blows the day stages' pale surfaces to white, AgX greys the palette). There is no screen-space ambient occlusion: under the cel light it made no visible difference, so contact shade
is painted (see section 1).

## 7. Checking it: the look bench

`godot/tests/look_bench.gd` (needs a window): the four default looks plus the darkest and palest body colours, under every stage mood,
against its bright haze and a dark backdrop. It measures how much of each silhouette stands clear of the backdrop and fails below 85%;
it writes `look_bench.png` and `look_bench_misses.png` (failures in red). Before the shading ramp and edge light, fighters on dark night
and city backdrops were nearly invisible at their edges (2%); now the worst case is 89%.

Run: `Godot --path godot --script res://tests/look_bench.gd -- --out=<folder>`
