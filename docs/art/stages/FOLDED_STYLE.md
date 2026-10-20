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

## Step 1 done: hill folds, the owner's pick D (2026-10-20)

Applied to the meadow and the grove (`hill_fold=0.6`, `hill_fold_clean=true`). One function, `_fold_offset`/`_fold_at`, gives the fold
at any point of a band, so what is laid on the land follows it:
- the meadow's fields, hedgerows and their bushes are laid through `_fields`' `at`, which adds the fold;
- the farm's barn and farmhouse each stand on the highest point of the folded ground across their footprint (the farmhouse sank into a
  fold when seated with the barn);
- the folds start a few units under each crest (smoothstep 1.2 to 5), so the houses, the windmill and the trees on the crests keep
  their feet in view (folds at the crest stood up in front of the meadow's left cottage);
- they fade out where a band's face turns into a wall (the grove's rise under the island hung in long vertical stripes like a curtain).

Measured (round 49): the meadow's valley 0.63 → 0.61 against its ranges' 0.59 (the depth step nearly even now), its contrast 0.048 →
0.071 (the folds, still calm); the grove's plain 0.58 → 0.53, now under its lawn (0.56), so the island stands out more. Frame time: no
cost (same draw calls and primitives; an A/B of folds off and on under the same load gave the same GPU time, 4.7 ms). Before/after:
`fold1_hills.png` (match camera, close-ups, and the wide `--cam=60` view of both stages). Tests: all six pass.

## Step 2 mocks: clouds (2026-10-20)

`_clouds` takes a palette key `[backdrop] cloud_style` (off by default): 1 faceted (low-poly puffs), 2 folded (`_pleated_puff`: each
puff pleated from its crown down, the folds deepest round its lower half), 3 folded with every other puff left out and the rest larger
(the same random numbers, so the layout stays). `cloud.gdshader` gains `flat_planes` (each panel lit as one plane, the heap bending it
less). The first pleats (0.24 deep, the heap at 0.55) lit both sides of every fold alike in the shader's bright tone; at 0.42 and 0.25
they read. Mocks on the meadow, grove, beach and canyon, match camera, in `mocks/clouds/` (A to D), on the owner's review page.
Recommended C.

## Step 2 done: clouds, the owner's pick D (2026-10-20)

`cloud_style=3` on the meadow, grove, beach (ocean) and canyon (sunset): folded puffs, every other one left out and the rest larger.
Before/after `fold2_clouds.png` (beach whole and close up, grove, canyon; round 50). Cheaper too: the beach 0.90 M → 0.83 M primitives,
draw calls 921 → 918. Tests: all six pass.

## Step 3: the grove's crowns and trunks (2026-10-20)

- **Crowns** (`tree_crown_a/b/c`, rebuilt with `make_props.py`): `folded_dome` gains `stagger`; the crowns now run each fold unbroken from
  the rim to the cap (`stagger=False`) and deeper (rim 0.44, middle 0.3, top 0.14). Staggered band by band, the folds met in a diamond
  pattern of small facets and read as crumpled low-poly; now each dome carries long lit and shaded pleats like the pines. Bushes and
  moss keep the staggered fold. 300 to 372 triangles each, within the prop budget.
- **Trunks** (`trunk_mesh` `folds`, the grove's `_tree`): seven folds (was nine) and creases at 0.6 of the ridges' reach (was 0.76), so
  each plane turns far enough to leave the lit cel band; the great trunk and the side trees read folded at the match camera.

Before/after `fold3_grove_trees.png` (round 51, and close-ups of a side tree). Contrast in the trunk's region 0.072 → 0.077, a side
crown's 0.089 → 0.096 (values unchanged). Tests: all six pass.
