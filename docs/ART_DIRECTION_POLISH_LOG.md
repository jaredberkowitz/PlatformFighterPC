# Art-direction polish: results log

A pass against an outside plan ("cohesive stylized art-direction polish": Wind Waker shading, PEAK characters, Valheim colour). Most of
that plan had already been done by the graphics fidelity pass (`GRAPHICS_FIDELITY_LOG.md`) and the visual overhaul
(`VISUAL_OVERHAUL_LOG.md`), so this pass only did what was still missing. Stills: `docs/screenshots/polish_*_{before,after}.png`
(Skyline with the match HUD, two players and four).

## Skipped as already done

| Plan phase | Where it is already met |
| --- | --- |
| 0 Audit, baseline, test scene | `VISUAL_OVERHAUL_AUDIT.md`; the repeatable set-up is `--demo=stage --stage=N --match`, `--demo=cpu4 --players=4 --perf`, `tests/look_bench.gd` (every look under every mood) and `tests/shot_stats.gd` |
| 2 Cel shading, highlights, rim, outlines | Ramp per stage, painted cool shadows, rim and edge light (`VISUAL_STYLE.md`); material responses for skin, cloth, rubber, metal and outlines constant on screen (overhaul phase 2) |
| 3 Lighting and post | Tone mappers compared (Filmic kept), SMAA, shadows checked, glow only above the threshold (overhaul phase 1) |
| 4 Rooftop, platforms, props | Paver seams, three-part girders with a lit edge, built props, contact shade (overhaul phase 3) |
| 5.1 Depth layers | Three skyline layers, haze, far blur (fidelity pass) |
| 7 Effects | Three hit tiers, effects that free themselves (overhaul phase 5) |
| 8 Performance | Measured below |

## Done

* **Art-direction rules written down** (`ART_DIRECTION.md`, "The rules every screen follows"): the attention hierarchy, the palette
  relationships and the material families, mapped to where each is set. No new constants were needed: the parameters exist.
* **Platforms calmer** (`stage_art.gd`, the city theme): the girders were saturated red and drew the eye as much as the fighters; they are
  coral (top `#d67a69`, web `#804245`), still clearly a platform, with the lit trim along the edge unchanged. Collision untouched.
* **Skyline less repetitive and quieter** (`facade.gdshader`, `stage_art.gd`):
  * each building now has its own muted hue (plum, indigo, mauve or blue-grey) as well as its own shade;
  * lit windows come in clusters (each block of a few bays and two floors has its own occupancy), so the face has rhythm rather than an
    even salting;
  * office strips are rarer (one building in eight, from one in five) and broken by a pier every three to five bays and a plain service
    floor every sixth, so no face is one stack of identical bands;
  * the walls of the nearer layers are a little lighter and the windows a little dimmer (lit energy 3.0 / 2.3 / 1.7 from 3.4 / 2.5 / 1.8):
    only the nearest layer blooms, and softly.
* **The roof block's wall** (the facade under the fighters): its pattern repeats every 6.6 units instead of 3.3 (sixteen windows a repeat,
  not eight) and about a third of its windows are lit (from a half), in a softer warm, so it is no longer a busy stamped grid.
* **HUD**: "FREE PLAY" is said once (the clock's header, or a small chip at the top with no clock) instead of on all four cards, and the
  portrait backings are the player colours a little muted (22% toward a slate), so four cards read as one calm strip and the damage
  numbers lead.

## Checks

* Match flow, art pipeline and fighter view tests passed; look bench PASSED, worst 0.93 (unchanged).
* Busy fight (`--demo=cpu4 --players=4 --stage=3 --perf`, Best): 4.12 ms average, p99 13.3 ms, GPU 1.28 ms, 531 draw calls (the overhaul's
  run: 5.2 to 5.6 ms, GPU 1.4 to 1.5 ms, 551). The shader changes cost nothing measurable.
* Nothing touches the simulation, collision, input or timing.

## Left for later

* The fighters' own saturation was not reduced: the look bench and the four-player still show them leading already, and their colours
  are the players' identity. Revisit only if a playtest says they clash.
* Judge the skyline's clusters and the coral girders in motion and on the other display sizes; the stills are at 1280 x 720.
