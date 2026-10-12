# Graphics fidelity: results log

Measurements for `GRAPHICS_FIDELITY_PLAN.md`, phase by phase. Stills: `docs/screenshots/fidelity_<phase>_<stage>.png` (1280 x 720, the
stage demo: `--demo=stage --stage=N --noui --noecb`). Numbers: `godot/tests/shot_stats.gd` (luminance percentiles, spread p98 - p2, mean
saturation, share of pixels above 0.9) and the stage demo's frame-time probe (`--perf`, now on any demo, with a 99th percentile).
Machine: the development PC, Balanced tier (the default), vsync off.

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
