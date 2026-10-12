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
