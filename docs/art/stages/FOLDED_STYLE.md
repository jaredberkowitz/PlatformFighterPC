# One folded style for every background (plan, 2026-10-20)

The owner: the folded paper look on the meadow's trees and mountains looks good, but it is not applied to the whole background, nor
on the other stages. This plan carries it to every natural background element, stage by stage. The rule is already written down:
`STYLE_SHEET.md` section 9, rule 9, "Folded, not crumpled": a form folded like paper, so the key light falls wholly on one side of
every fold. The kit that builds it is `art/blender/fold_kit.py` (`pleated_tier`, `folded_dome`, `folded_peak`, `block_tier`).

**Scope.** Natural forms behind the play area: land, hills, mountains, trees, bushes, rocks, clouds. Built things (houses, the windmill,
towers, crates, the dock) keep straight, worn edges (section 4); the city's skyline is built and stays as it is. The play surfaces and
the fighters are out of scope.

## Why some folds show and some do not

The stages shade in cel bands (`lighting.gd` `ramp`; the meadow's: fully lit above 0.6 of the way toward the light, a half band from
0.43 to 0.55). Any plane turned roughly toward the key falls in the fully lit band, so a shallow fold lights both its sides alike and
vanishes. Pass 2 found this on the meadow's cliff: alternating its columns about 25 degrees changed nothing measurable. A fold reads only
when its shaded side turns far enough from the key to drop a band: the ranges' and pines' folds are steep (40 degrees and more), which is
why they read. Every folded element below needs folds that steep, or it needs to be lit from the side.

## Inventory (round 45 to 48 captures)

| Stage | Element | Built by | Now | Treatment |
|---|---|---|---|---|
| Meadow | Mountain ranges | `folded_range` model | folded | keep (the reference for the rest) |
| Meadow | Pines | `pine` model (`pleated_tier`) | folded | keep |
| Meadow | Rolling hill rows, the rise behind the fence | `_land_band` (code, `chunky` jitter) | smooth, soft gradients; the random jitter does not show | **fold**: ridges running down from each crest |
| Meadow | Hedgerow clumps in the valley | `_fields` (code, balls) | smooth beads | folded clumps (`folded_dome` style) |
| Meadow | Fields | `_fields` (code, flat) | flat patchwork | keep flat; fold only the ground they lie on |
| Grove | Far ranges | `folded_range` (rounded) | folded | keep |
| Grove | Purple and teal hill rows, the rise | `_land_band` (code) | smooth | **fold** (the same change as the meadow's) |
| Grove | Side tree crowns | `tree_crown` model | random low-poly facets (backlog item 3) | refold as `folded_dome` with steep pleats |
| Grove | Tree trunks (great tree, side trees) | `trunk_mesh` (code) | smooth tapered columns | ridged folds down the trunk (`trunk_mesh` already takes `folds`) |
| Grove | Bushes | `bush` model | folded | keep |
| Grove | Far tree line | `_tree_line` (code) | smooth blobs | folded domes or remove |
| Beach | Headland hills | `hill_dome` model | folded | keep |
| Beach | Islet with palms | `_ocean` (code, scaled spheres) | primitive mound | a folded `hill_dome` with a sand skirt |
| Beach | Quay rocks | `_rock` (code) | smooth blobs | folded rocks (`block_tier` or a faceted boulder) |
| Canyon | Dunes | `dune_range` model | folded | keep |
| Canyon | Mesas, buttes, the arch | `mesa`, `rock_arch` (`block_tier`) | stacked folded slabs | keep |
| Canyon | Desert rocks | `_rock` (code) | smooth pink blobs | folded rocks (as the beach's) |
| Canyon | Cacti | `cactus` model (`blob` arms) | smooth capsules | ribbed folds (a cactus is pleated in life) |
| All but the city | Clouds | `_clouds` (code, sphere puffs, `cloud.gdshader`) | smooth cotton | folded or faceted puffs: **mock first**, the biggest change of look |

## Order (shared first, biggest area first)

1. **Hill rows and rises** (`_land_band`): one function, the meadow's and the grove's largest background areas, and the editor's custom
   stages. Mock two fold depths.
2. **Clouds** (`_clouds`, `cloud.gdshader`): on four stages. Mock folded against the current soft look; the owner picks.
3. **Grove crowns and trunks**: refold the crown model, ridge the trunks.
4. **Small round things**: hedgerow clumps, rocks, the islet, cacti, the grove's far tree line.

Each step follows the refinement brief: before/after from the same camera, the greyscale bands, fighters in play, the tests, and a log
entry in `STAGE_ANALYSIS.md`.

## Step 1 mocks: hill folds (2026-10-20)

`_land_band` takes a palette key `[backdrop] hill_fold` (a share of the band's column width; off by default) and `hill_fold_clean`
(drops the old random jitter). Every other column stands proud along the surface's own normal, growing from the crest down. A first try
pushed the columns toward the camera, which slid the meadow's near-flat valley floor sideways and tilted nothing; along the normal they
fold on faces and floors alike. Options rendered on the meadow and the grove (`mocks/hill_folds/`, A to D: none, 0.35, 0.6, 0.6 clean),
reviewed on the owner's flip page. Recommended D. Known catch: deep folds move the land up to about 5 units, so the houses, trees and
the windmill must be re-seated on the folded ground before it ships.
