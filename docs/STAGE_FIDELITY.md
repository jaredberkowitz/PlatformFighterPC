# Stage visual fidelity: audit and change report (2026-10-14)

The owner's stage fidelity plan, run on top of `docs/VISUAL_POLISH.md` (which already covers much of it). Refine what exists; one change
at a time, compared from the same camera; keep only what clearly helps.

## Audit: the five largest opportunities

| # | Opportunity | Cause | Resources | Status |
|---|---|---|---|---|
| 1 | **The main platform reads as layers**: grass top, soil wall, a row of equal pale boulders, then a keel in a different material. | Geometry and material: the keel was a separate stone material, the skirt a row of near-equal lighter rocks standing proud. | `stage_art.gd` `underside` | Done (pass 1) |
| 2 | **Distant foliage as repeated blobs** (the grove's canopy below the island, the meadow's far tree line). | Composition and geometry: equal round puffs in rows. | `stage_art.gd` `_canopy` | Next |
| 3 | **Backdrop finish below the stage's** (flat colour, no ink beyond 50 units, boxes). | Material and outline. | `outline.gdshader`, `_cottage`, `_fields`... | Done (`VISUAL_POLISH.md` pass 60) |
| 4 | **Material families and contact shadows**. | Material. | `materials.gd`, `VISUAL_STYLE.md` | Done (passes 55 to 58) |
| 5 | **Fighter readability and grounding**. | Shading, line, drop shadow. | `toon_light`, `outline`, `fighter_view.gd` | Done (passes 52, 53, 58; `CHARACTER_POLISH.md`) |

Not doing: a camera change (the plan says not without authorization; current framing passes the readability bench) and global colour
grading (each stage keeps its own finish).

## Pass 1: the platform as one body of earth (meadow; every grassy floating island)

- The keel is now the wall's own earth carried down (the soil material falling into shade below the light, `_under_material` at
  0.85), as the canyon's mesa and the city's tower already were, so the soil face and the underside are one surface.
- The rocks along the wall's foot are the earth's darker colour, of very different sizes, in clusters with gaps, and sunk into the
  earth rather than standing proud in a row.
- The stones bedded in the keel are in the earth's colours, not pale lavender, and half sunk.
- The haze under the island is lighter (it washed the keel paler than the wall, backwards for an underside).
- **Images**: `docs/screenshots/stage1_platform_underside.png` (before left).
