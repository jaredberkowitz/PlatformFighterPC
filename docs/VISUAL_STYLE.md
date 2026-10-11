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
profile** (`Lighting.STYLES`: ramp, deep shade, edge light, painted gradient), applied together by `Lighting.stage_mood`. A new stage picks
a theme and inherits all of it; it overrides only what it needs.

## 4. Atmosphere and post

The post pass adds distance haze toward the stage's haze colour (from 4 units behind the stage plane, so the nearer scenery layers already
separate from the farther ones, up to 36% far back; the stage and its props are untouched), the pastel finish (fighters get half, so they stay the most vivid
thing), a paper tone and grain, a light vignette (0.16: dark corners hide fighters near the edge) and the stage's grade. The grade is
restrained: fighters take a third of it, so a white glove stays white on a dusk or night stage, and no stage washes everything in one hue
(the city's purple stays in its sky and buildings; its roof is pale stone). Glow is kept for
effects (`Lighting.FX_GLOW`). Depth of field is only for presentation: the stage softens behind the results screen (`RESULTS_BLUR`).

## 5. Combat effects

`scripts/effects.gd` (drawn bursts by weapon, impacts, rings, the DI line) and `scripts/particles.gd` (sparks, dust, embers, knock-out
confetti) share conventions: bigger and longer for stronger hits, drawn on twos like hand-drawn effects, warmed up at start so the first
hit does not stutter. The Effects setting thins particles evenly (`Burst.density`).

## 6. Comfort and scalability

The Controls screen's Display page (`scripts/visual_settings.gd`, saved in `user://display.cfg`): **Screen shake** (off, low, full: scales
the shake, roll and punch-in), **Flashes** (off, soft, full: the full-screen flash; the finishing blow's two-tone frames only at full),
**Effects** (low, medium, full: particle counts) and **Quality** (fast: no shadows or anti-aliasing; balanced: 2x
anti-aliasing; best: 4x). There is no screen-space ambient occlusion: under the cel light it made no visible difference, so contact shade
is painted (see section 1).

## 7. Checking it: the look bench

`godot/tests/look_bench.gd` (needs a window): the four default looks plus the darkest and palest body colours, under every stage mood,
against its bright haze and a dark backdrop. It measures how much of each silhouette stands clear of the backdrop and fails below 85%;
it writes `look_bench.png` and `look_bench_misses.png` (failures in red). Before the shading ramp and edge light, fighters on dark night
and city backdrops were nearly invisible at their edges (2%); now the worst case is 89%.

Run: `Godot --path godot --script res://tests/look_bench.gd -- --out=<folder>`
