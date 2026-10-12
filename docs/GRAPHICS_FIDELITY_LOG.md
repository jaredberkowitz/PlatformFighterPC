# Graphics fidelity: results log

Measurements for `GRAPHICS_FIDELITY_PLAN.md`, phase by phase. Stills: `docs/screenshots/fidelity_<phase>_<stage>.png` (1280 x 720, the
stage demo: `--demo=stage --stage=N --noui --noecb`). Numbers: `godot/tests/shot_stats.gd` (luminance percentiles, spread p98 - p2, mean
saturation, share of pixels above 0.9) and the stage demo's frame-time probe (`--perf`, now on any demo, with a 99th percentile).
Machine: the development PC, the Best tier (the default), vsync off. Frame-time runs on this machine now and then stall at about 60 fps
with many frames over 25 ms whatever the stage (it happened on the very first baseline run too); those runs are discarded and rerun.

## Phase 0: baseline

| Stage | p2 | p50 | p98 | spread | saturation | bright |
| --- | --- | --- | --- | --- | --- | --- |
| Meadow (0) | 0.336 | 0.772 | 0.871 | 0.535 | 0.188 | 0.0067 |
| Triple Tier (1) | 0.306 | 0.654 | 0.798 | 0.492 | 0.358 | 0.0000 |
| Flat Island (2) | 0.466 | 0.760 | 0.934 | 0.468 | 0.181 | 0.1051 |
| Skyline (3) | 0.127 | 0.287 | 0.749 | 0.622 | 0.447 | 0.0001 |
| Treetop Isle (4) | 0.434 | 0.753 | 0.853 | 0.420 | 0.212 | 0.0013 |

Frame time (warm runs, 10 s each): Skyline 1.37-1.71 ms average, p99 1.67 ms (one run 15 ms with a cold shader cache), GPU 1.01 ms,
269 draw calls, 196k primitives; Treetop Isle 1.70 ms average, p99 2.04 ms, GPU 1.32 ms, 440 draw calls, 265k primitives.

Look bench: PASSED, worst silhouette 0.93 (needs 0.85); every mood 0.93 to 1.00 on the edge test.

## Phase 1: Skyline facades and emissive light

* The backdrop buildings are one mesh per layer (`StageArt._skyline_layer`, three draw calls) drawn by `shaders/facade.gdshader`: windows
  in fitted bays with frames and sills, a parapet cap, floor bands every fourth floor, darker toward the street, per-building seeds for
  which windows are lit and in what colour (mostly warm, some pale blue, a rare magenta), rooftop vents, tanks on legs and antennas. The
  old per-window boxes are gone (they were merged into a few meshes already, so draw calls barely move: 269 to 267; primitives 196k to
  184k).
* Lit windows are HDR (3.4 / 2.5 / 1.8 times white from the near layer back) so the near ones bloom softly. A first try at 4.2 times a
  pale yellow tone-mapped to plain white, so the colours are saturated and the energy lower. A bug worth recording: the seed came from an
  interpolated vertex colour, whose last bits wobble across a face, and the window hash turned that into stripes; the seed is now rounded.
* Halos (see-through cards, `fx.gdshader`, drawn after the finishing pass) on the string lights, the lantern posts and the antenna's
  blinker (which blinks with it). `glow` is a per-theme style value (`Lighting.STYLES`: 0.75 by default, 0.95 night, 1.1 city), and the
  Fast tier turns glow off.
* The roof has decals from a new tile, `godot/art/stage/roof_detail.svg` (a water stain, a puddle, a tar patch, grime); they are
  subtle at the play camera's shallow angle.

Checks: fighters in front of lit windows do not bloom (the white-shirted fighter in `fidelity_1_skyline.png`); look bench PASSED, worst
0.93 (unchanged); Skyline 1.58 ms average, p99 2.08 ms, GPU 1.09 ms (+0.08 ms). The other four stages are unchanged apart from the
lantern's glow on Triple Tier.

## Phase 2: value range and grade

* `pastel_desat`, `pastel_lift`, `vignette` and `haze_max` moved from the post shader's defaults into each theme's style
  (`Lighting.STYLES`, `finish`), pushed with the grade in `stage_mood`, with a new gentle S-curve (`s_curve`, the 0..1 range only, so lit
  windows keep their glow; fighters take half). Moved at today's values first: a whole-frame pixel diff is no use here (two renders of
  the same build differ on 19-29% of pixels because clouds, sails and idle poses move with real time), so the check was the shot
  statistics, which matched to the third decimal (Meadow spread 0.535 / 0.537, Skyline 0.747 / 0.747, Flat Island 0.474 / 0.475).
* Tuned: by default desat 0.07 (was 0.10), lift 0.06 (0.13), haze ceiling 0.30 (0.36), S-curve 0.2; the city lift 0.04, S-curve 0.3.

| Stage | p2 | p50 | p98 | spread (baseline) | saturation (baseline) |
| --- | --- | --- | --- | --- | --- |
| Meadow | 0.279 | 0.771 | 0.870 | 0.590 (0.535) | 0.240 (0.188) |
| Triple Tier | 0.270 | 0.627 | 0.799 | 0.530 (0.492) | 0.426 (0.358) |
| Flat Island | 0.391 | 0.753 | 0.934 | 0.543 (0.468) | 0.200 (0.181) |
| Skyline | 0.097 | 0.233 | 0.877 | 0.780 (0.622: +25.4%, the target) | 0.488 (0.447) |
| Treetop Isle | 0.342 | 0.751 | 0.856 | 0.514 (0.420) | 0.259 (0.212) |

Look bench PASSED, worst 0.93 (unchanged). The optional colour-grade LUT (step 3) was skipped: the per-theme finish values cover what it
would have been used for here, and it adds a texture format question for no visible gain yet.

## Phase 3: depth

* Far-only depth of field in play, the Best tier only. Tried first as the plan says, with the camera's `CameraAttributesPractical`: it
  blurred the name tags, the light halos and would blur any effect drawn over the sky, because see-through things write no depth and
  take the blur of whatever is behind them. So the blur is in the finishing pass instead (`far_blur` in `post.gdshader`, 2.6 pixels at
  1080 lines at most, from 14 units behind the stage plane over 30): it runs before the effects, tags and halos are drawn, so they stay
  sharp, and it gathers only from pixels that are far themselves (ten taps on a spiral), so the fighters do not bleed into it. Ink lines
  on the blurred scenery fade with it. Fast and Balanced keep `far_blur` at 0. The results screen's own camera blur is unchanged; a
  rematch clears it as before.
* Checked: the stage plane, the fighters and the tags stay sharp in the wide stills and in a close-up
  (`fidelity_3_skyline_closeup.png`); Skyline GPU 1.04 ms (1.09 before), Treetop Isle 1.36 ms (1.32); look bench PASSED, worst 0.93.
* Skipped: the optional foreground framing (cables and leaves at the frame's edges). The blur alone gives the depth, and framing pieces
  over the outer 8% of the screen risk hiding a fighter near the blast lines for little gain.
