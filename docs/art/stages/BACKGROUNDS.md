# The backgrounds audit (2026-10-21)

The owner asked for the biggest gains left in the backgrounds (props, the grounds, the skies), and noted the background floors and grounds
read as needing work. Audited from fresh captures of every stage at the match camera and wide, against the papercraft references
(docs/art/stages/LOOK.md).

## Ranked

1. **The grounds (the meadow, the grove, the canyon).** The meadow's land behind the stage is horizontal mint stripes edge to edge, the
   same shape and nearly the same tone, with nothing on them; at the wide camera a striped blanket. The grove's terraces either side are
   flat teal-green stripes and the abyss under the island one dark band; the canyon's floor one plain sand sheet with a few bushes. The
   references lay ground as two or three greens in overlapping cut shapes with organic edges, pale paths as separate sheets on top, small
   things dotted over them, soft shade where a sheet lies on another. Rebuild as paper sheets: patchwork fields with hedgerows and lanes
   on the meadow; terraces with draped grass edges over rock on the grove; dune sheets, scrub, stones and a dry wash on the canyon (the
   LOOK.md lesson 3, the draped grass sheet, with it).
2. **The clouds (every stage).** Faceted low-poly puffs that read as crystal, against the cut-card smoke and the references' flat, thick
   card clouds: rebuild as layered, scalloped card cut-outs. Cheap, and they fill much of every frame.
3. **The canyon's buttes and arch.** Stacks of rounded discs that read as pancakes; the LOOK.md lesson 4, chunky cardboard blocks of a few
   big bevelled planes, stepped ledges, warm stone. They fill most of the canyon's middle distance.
4. **Shade and detail in the middle distance.** Nothing behind the play area casts shade on the ground (the sheet shadow fades out by 50
   units behind it), and the middle distance has no small detail: carry the shadow over the near grounds, and dot a few paper pieces
   (bushes, stones, flowers) on them.
5. **The skies.** Plain gradients; a faint paper texture or layered card bands at the horizon. The smallest of the five.

## Order

The grounds first, on the meadow (the most ground in view): mocks A patchwork (cut sheets in a few greens), B with lanes and hedgerows,
C with scattered detail and soft shade; then the grove and the canyon. The clouds alongside, as a quick win on every stage.

## Round 1 mocks: the meadow's ground and card clouds (2026-10-21)

- **The meadow's ground** (`make_meadow_ground.py --variant PA|PB|PC`, B's pines and hill kept; `--ground-mock=` and `--bush-mock=`):
  the land a smooth sheet laid over with fields of card, rounded oblongs along the contour, each lifted over the land with its cut edge
  showing, in the palette's greens and a little wheat, the land's own green between them; **PB** adds two lanes winding down the rise and
  hedgerows rolled along some fields' lower edges; **PC** adds bushes, stones and flowers with soft shade under them and a deeper crease
  bake. (First try: round blobs of every size overlapped into camouflage; lanes ran off the frame.) The new families `lane`, `card_lane`,
  `shade`, `stone`, `flower_a/b` are in `StageBake._ground_colours`. About 75,000 to 150,000 triangles: to trim before shipping.
- **Card clouds** (`--cloud-style=4`, `StageArt._card_cloud`): flat cumulus cut from thick card, a scalloped crown on a gently bellied
  base, a deeper card behind each, still lit by the cloud shader (smooth, not its flat planes: those shaded each triangle of the face on its
  own, as crinkled shards). The tower kind lower with four bumps (three read as a box).
- Options on the Stage Look Review and `mocks/look/` (`mg_*`, `cl_*`). Recommended: ground B, card clouds.

## Round 1 built (2026-10-21): meadow ground B and card clouds

- **The meadow's ground B** is in the game (`make_meadow_ground.py --variant PB` into `meadow_ground.glb` and `meadow_pines.glb`), trimmed to
  about 76,000 triangles (the fields' rows and the hedgerows' segments cut to what the match camera resolves); the contact shade re-baked
  over it (`light_stage.py` on the front, the play area and the ground). 120 fps in the match, the GPU at 2.6 ms.
- **Card clouds** on every stage with clouds: `cloud_style=4` in the grove, meadow, ocean and sunset palettes (the city has none).
- Before and after: `ground_clouds_built.jpg`.

## Round 2 mocks: the grove's land and the canyon's floor (2026-10-21)

- **The grove** (`make_grove_paper.py --variant FA|FB|FC`, mocked with `--ground-mock=` and no bush): the striped ledges broken into
  floating islands at varied heights, as the grove's reference floats its land, each a grass sheet whose lip drapes over its edge above a
  stone underside of stepped card plates narrowing to a point, over two sheets of pale violet mist in place of the dark band; the side
  trees keep an island each. **FB** hangs moss down the stone and dots the tops with tufts, flowers and stones; **FC** adds the far
  forest, rows of giant trunks fading into the violet. (First try: the undersides alternated dark teal and grey plates, stripes again; one
  violet stone, `isle_rock`, sits in the grove's air. Pale stone read ghostly.) The old ledge's bushes go with it. About 23,000 to 25,000
  triangles. New families `mist`, `card_mist`, `tuft`, `canopy`, `isle_rock`, `isle_moss`.
- **The canyon** (`make_canyon_paper.py --variant DA|DB|DC`, A's buttes): the floor laid over with long cut sheets of sand in three tones
  round the desert floor's, lifted a step each with the cut edge showing, wind ripples as pale strips on some; **DB** adds a dry wash
  winding in from the far right through the gap in the near buttes (first routed past x 60, off the frame), pebbles on its banks; **DC**
  adds paper scrub and stones with soft shade under them, seated on the topmost sheet, and hides the code-built scatter (`floor_scatter`
  meta, hidden by `StageBake.place_ground` when the ground has `scatter*` pieces). About 22,000 to 25,000 triangles with the buttes. New
  families `sand_a/b/c` and their cut edges, `ripple`, `wash`, `card_wash`, `pebble`, `scrub`, `sand_shade`.
- Options on the Stage Look Review and `mocks/look/` (`gl_*`, `cf_*`). Recommended: grove B, canyon C.

## Round 2 built (2026-10-21): grove B and canyon C

- **The grove's floating ledges with moss and tufts** (`make_grove_paper.py --variant FB` into `grove_paper.glb`, 22,900 triangles); the
  grove's ground has no bushes now (`GROUNDS`, the bush path empty: they stood on the old ledge). `grove_bush.glb` stays on disk, unused.
- **The canyon's floor with the dry wash, paper scrub and stones** (`make_canyon_paper.py --variant DC` into `canyon_paper.glb`, 25,400
  triangles with the buttes); the code-built scrub and pebbles hidden under it.
- Both stages re-lit (`--light-export`, then `light_stage.py` on the front, the play area and the ground, curve and reach 2.5). The canyon
  at 290 fps, the GPU at 2.6 ms.
- Before and after, wide: `lands_built.jpg`.

## Round 3 mocks: the canyon's buttes and the light pass (2026-10-21)

Review 4's items 3 and 4 (docs/art/stages/FIDELITY_REVIEW_4.md).
- **Cardboard buttes** (`make_canyon_paper.py --variant KA|KB|KC`, on DC's floor): **KA** every butte two or three tiers of angular blocks
  (five to seven straight sides at uneven angles) with bevelled tops, stepping in as they rise, each tier banded in the strata, on a low
  skirt block; the arch's legs blocks, its span three bevelled beams. **KB** slab courses, as the stage's front is built (in the distance
  they read as brick walls or crates). **KC** KA with notched crowns and fallen blocks round the feet. Each block's faces are turned out
  from its own middle (`OWN`), not the butte's axis. About 20,000 triangles with the floor. Recommended: KC.
- **The light pass** (`--light-mock=A|B|C`, `Lighting.light_mock`): **A** form on the ground, every play top falling into a cool shade
  toward its front edge (`lawn_fall` in the toon shader, set by `StageBake.place_play`; the meadow's lawn 0.78 at the back to 0.63 at the
  front, was 0.79 to 0.70) and the day stages' sun 8 degrees lower; **B** A with coloured distance (the palettes' air greys a fifth as
  much, the post haze's `haze_desat` 1.3 to 0.35, its colour more saturated: the meadow's distance saturation 0.26 to 0.39); **C** B with
  the beach at late afternoon (`Lighting.LATE_OCEAN` and `LATE_OCEAN_SKY`; the sea keeps its midday cyan in the mock). Recommended: B, and
  C if the beach should have a time of day.
- Options on the Stage Look Review and `mocks/look/` (`bt_*`, `li_*`).

## Round 3 built (2026-10-21): buttes C and the light pass B with C's beach

- **The canyon's buttes and arch in cardboard blocks with notched crowns and fallen rock** (`make_canyon_paper.py --variant KC` into
  `canyon_paper.glb`, 20,600 triangles with the floor); the canyon re-lit.
- **The light pass**, the mock's values written into the data (the `--light-mock` flag is gone):
  - form on the ground: every play top falls into a cool shade toward its front edge (`lawn_fall` 0.9 over 6 units, set by
    `StageBake.place_play` on the lawn, top, path and fringe families); the meadow's and the grove's sun 8 degrees lower
    (`Lighting.MOODS`);
  - coloured distance: every mood's haze a clearer colour (its saturation half again), the post haze greying less (`haze_desat` 0.35 in
    `STYLE_DEFAULT.finish`, was the shader's 1.3), each palette's `[air] desat` a fifth of what it was;
  - the beach at late afternoon: `MOODS.ocean` (a low warm key, cool blue shade, a peach haze and grade) and `ocean.cfg`'s sky (a peach
    horizon, a warm glow, the sun lower).
- Before and after (meadow, beach wide, grove, canyon wide): `buttes_light_built.jpg`.

## A new approach (2026-10-21): structure, not pattern

The owner: the grove's floating islands made little progress; focus on the background ground of the four stages that aren't the city.
Next to the references, the reason is the same on all four. For three rounds we changed the pattern on the land (stripes, patchwork
fields, floating islands, sand sheets) but not its structure:
- **Thin sheets, not masses.** Our land is flat sheets, lit alike and lifted by the haze, all in one band of mid values. The references'
  ground is mass: a thick top with an overhanging lip, rock or earth under it turning away into shade, dark gaps between pieces. At
  background distance their light-over-dark is what reads; a pattern on a flat sheet blurs away.
- **Many small, repeated things** (fields, islands, sand sheets by the dozen) turn into texture. The references use a few big pieces.
- **Detail spread evenly.** The references put it where the eye lands, on the lips (fringed grass, ferns, hanging vines), and keep the
  faces simple and dark.

**Pilot on the grove** (`make_grove_paper.py --variant H`): each ledge broken into chunks 22 to 40 wide at their own heights and depths
with dark gaps between (the chunks under the side trees keep the trees' footing, nothing right behind the fight); a thick grass top
overhanging in a scalloped lip, ferns and vines hanging from it, tufts on the top; under it dark teal cardboard blocks (`cliff`,
`cliff_b`) with faceted fronts, each lower block stepping back into a deep violet abyss (`abyss`, `abyss_far`), which swallows the
rock's feet; occlusion baked at 5 units, not 2, so every lip shades the rock under it. 37,400 triangles. (First try: blocks of three
hues, beige among them, and a full-length wall each read as a quilt of boxes; one ledge per band still read as stripes.)

If the owner takes the direction: the meadow's land as two or three big overlapping hill masses (grass draped over earth banks, soil at
the cut edges, hedgerows on the crests, shaded sides); the beach's town hill and islands as grass draped over pale sandstone blocks; the
canyon's floor as a few big dune masses with shaded lee sides and low rock steps at its edges.

**Pilot 2, after the owner's notes** (the broken ledges didn't read as believable: trees floated, every chunk faced the same way, some
trees were too short). The land is connected: terraces as plan outlines (`side_tiers`, `back_tier`) round the chasm the island floats
in, T0 and T1 either side and T2 right across the back, each tier's front rising from the top of the one before it (`ground_under`).
Their fronts wander out into promontories (`bumps`) and their inner ends turn back into the chasm, so the cliffs face every way. Rock is
laid along whole runs of showing edge (`rock_run`; per two-unit outline segment it read as a wall of little cubes), set back under the
lip. Pines are seated on the tier they stand on, clear of every edge and of higher tiers, bigger (3 to 6.5) and fewer; the far tree
village stands on T2 at -1, as before. Tufts over the tops. About 60,000 triangles. (A sign slip in the outline's winding first put the
rock and the lips on the tiers' hidden back edges.)

**Pilot 3, the owner's ask: the terraces in a half-circle round the stage** (`arc_tiers`, `arc_band`). Each tier is a band of a
half-circle centred on the stage: its front the inner arc wandering in toward the stage in promontories, its back the outer arc under
the face of the tier behind. T0 (radius 46 to 72) and T1 (72 to 100) come in halves meeting straight behind the great tree, at the old
heights (-9.3 and -9.0, -5.0 and -4.5); T2 (100 to 136, -1) is whole. Each tier's arc runs a little further round than the one in front
(166 to 374 degrees for T0, 156 to 384 for T2), so its ends wrap past it beside the stage. The gates (radius 61 and 65), the side trees
and the far tree village (110 to 121) keep their footing. Pines are kept off the near ends (z above -15), where they would crowd the
frame. About 61,000 triangles.

**Pilot 4, the owner's notes on the trees** ("all a somewhat consistent, believable and large size"; refine each stage on its own
unless a game-wide system gives a real gain). The reference's forest is one kind of giant tree. The terraces now ask the game to stand
the grove's own tree (`grove_tree.glb`, the side trees' model) at the side trees' size, 0.55 to 0.7 of the great tree: `treeslot` empties
in the ground (`tree_slot`), turned into `StageArt.tree_slots` by `StageBake.place_ground` before `place_trees` runs. They replace the
paper disc pines, which read as shrubs. Three a side on T0 and T1 and six on T2, each 15 from the side trees, the village's trees and
one another, its canopy clear of every edge, sunk 0.6 as the side trees are. The cliffs have wider, taller blocks (4 to 7.5 by 3 to 5),
moss on the ledges each block leaves as it steps back, and flowers among the tufts. About 44,000 triangles in the ground; the grove at
183 fps with the GPU at 2.7 ms (2.35 ms today).

**Built (2026-10-21): grove pilot 4** (`make_grove_paper.py --variant H` into `grove_paper.glb`, the grove re-lit; the terrace trees reach
the game through `treeslot` empties). Before and after: `grove_terraces_built.jpg`.

## The meadow's ground from its own reference (2026-10-21)

The owner: ship the grove, then the next stage's ground with its own reference. The meadow's is the papercraft camp
(`docs/art/benchmarks/papercraft_mountains_reference.png`): a few broad shelves of bright lime grass, each ending in a crisp dark ledge
under a grass lip; pale paths across them; belts of big dark pines behind, taller than the buildings. Ours: pale rounded fields on one
smooth slope, small pine clusters.

**Pilot** (`make_meadow_ground.py --variant MH`, `--ground-mock` and `--bush-mock`): the rise as five shelves (`shelved_band`, drops 0,
4, 11, 22, 36, 52 under the crest, flat lime tops, `shelf`), each ending in a dark green-grey ledge (`ledge`, the reference's 0.2, 0.31,
0.24) under a scalloped lip (`shelf_lip`); the ledges wander out in promontories (`ledge_offsets`), and how much of each step a ledge takes
varies slowly along x (`share`), so in places two shelves run together and no ledge crosses the frame as a stripe (first try: seven
shelves edge to edge read as stripes again; a faster variation pulled the shelves into steep folded ramps; alternating greens read as
panels). The lanes cross the shelves and step down each ledge on stairs. Pine belts along the near hills (`forest_belts`, one size, about
15 tall); the clusters at 1.2. Tufts and flowers over the shelves. The far and near hills plain card (B's layers). About 44,000 triangles.

## The meadow's ground tidied, and the first detail pass (2026-10-22, mocks)

The owner, on the shipped meadow (PB): "the ground ... reads a bit muddy, unprofessional, not consistently shaped"; and: begin adding
more detail to the background trees, lights, islands and buildings, one stage at a time.

Causes: free-form field blobs of every size; cream cut edges, which read as grubby outlines at a distance; a deep crease bake.
`make_meadow_ground.py --variant QA|QB|QC` (`quilt_band`): **QA** tidy quilt (every field the same rounded rectangle in even rows,
gaps a deeper green `qgap`, three greens and wheat `qf_a/b/c`, `wheat`, hedgerows along the rows `qhedge`, cut edges a deeper green
`qcard`, occlusion at 1.5); **QB** half as many rows, fields twice as wide; **QC** a lawn with fields only round the farm and along
the lanes. The hills behind take light and dark field tones (`hill_near_l/_d`, `hill_far_l/_d`). (First tries: fields on the land's
own green vanished; the farm and lane exclusions emptied the left side.) Recommended: QA.

Detail, first piece: `--pine D` (`pine_detailed`): eight skirts with scalloped hems, a slight lean, a trunk with roots (a pleated-tier
try read as shards). Mocks on the Stage Look Review (`mq_*`). The proposed order per stage is on the page.

**Built (2026-10-22): meadow ground QA with the detailed pines** (`make_meadow_ground.py --variant QA --pine D` into `meadow_ground.glb`
and `meadow_pines.glb`, the meadow re-lit). Before and after: `review5/meadow_quilt_built.jpg`.

## The meadow's detail pass (2026-10-22, mocks)

The owner: continue the meadow's detail pass (windmill, cottages, haystacks). Mocked behind flags:
- **Windmill** (`--detail-mock`, `StageArt._windmill`): finer lattice sails, every other one reefed; a stone door surround and step, a
  lantern, a barrel and sacks; the base in two courses of stone blocks.
- **Houses** (`make_paper_houses_v2.py --shape B --detail 1`, `--house-mock=`): shutters, side windows, door lanterns, picket fences with
  stone paths, flower pots; the barn's loft hay, side windows and weathervane.
- **Haystacks** (`make_world.py --theme meadow --variant BC --detail 1`, `--world-mock=`): straw strands, a frayed hem, crown wisps.
Before and after: `mocks/look/md_detail.jpg`. Known issues: the door surround squares off the arched door; a fence runs into the slope.

**Built (2026-10-22): the meadow's detail pass.** The windmill's detail is on by default (`--no-detail` compares); its door surround is
now jambs and seven voussoirs round the arch. The house kit (`paper_houses.glb`, meadow and beach) and the meadow's world are rebuilt
with the detail (now each script's default). The fence moved close before the front (0.85, was 1.6) with deeper pickets, so it no longer
runs into a slope. Two lessons: the door lanterns are plain warm paper (`lamp`), not the lit `glow`; and each house's pieces are joined
by material at export, since a town of separately meshed shutters and pickets overflowed the per-instance shader buffer and painted
stray glows and blue edges on the beach's tower, warehouse and boats. 243 fps on the meadow (GPU 2.25 ms), 239 on the beach. Before and
after: `review5/meadow_detail_built.jpg`.

## The grove's detail pass (2026-10-22, mock)

`make_world.py` (detail on by default, `--detail 0` without; mocked with `--world-mock=`): the tree village's huts get plank battens, a
framed window with red shutters, a second thatch fringe, a post-and-rope rail round each deck with a lantern, and a ladder down each
trunk; the far great trees roots and branch stubs; the gates a rope with paper zigzags and stone plinths. `make_world.py` now joins its
still pieces by material and depth band before the bake (the per-instance shader buffer). Before and after: `mocks/look/gd_detail.jpg`.

## Blender models over code (2026-10-22, built)

The owner: worried about quality, switch the art to Blender models wherever possible; the forest was a few sporadic trees. Worked from
`MODEL_AUDIT.md`: the grove's forest and the meadow's pine belts; the windmill as one model with its sails turning as a separate node;
the beach's bell tower, warehouse, market stalls and stilt hut with its jetty in the house kit; the city's near towers modelled from
the game's own layout with real window frames, sills, lintels, ledges and pilasters under the same facade shader. Each keeps its code
build as the fallback. Before and after: `review5/windmill_model.png`, `harbour_models.png`, `city_towers_model.png`.

## The meadow's farmland (2026-10-22, mock)

The owner: the meadow's ground still feels cheap next to the grove; the plots sloppy, the road not connected. At the wide camera: the
fields are rounded stickers on one flat green, the two lanes start at the crest and stop at the bottom, and there is no structure (the
grove has connected masses, lit tops over dark sides, a dense fill). `make_meadow_ground.py --variant FA|FB|FC` (mocked with
`--ground-mock=`, `--bush-mock=`): FA hedged fields tiling the slope edge to edge (contour rows, boundaries down the slope, bushy
hedgerows, crops in rows mostly round the farm, one connected lane network with a spur into the farmyard and hedges along the lanes);
FB FA as shallow terraces (flatter shelves, short grass banks; the farm on its own wide shelf); FC FB with pines along the hedgerows
and a copse behind the farm. Earth-brown banks read muddy, so the banks are a deep grass green. On the Stage Look Review ("meadowfarm",
`mf_*`). Mocks are unlit and about 240,000 triangles; the ship bakes the contact shade and trims.

**Built (2026-10-22): the meadow's farmland C** (the owner's pick): terraced hedged fields with pines along the hedgerows and a copse
behind the farm, the lanes connected. The hedgerows' bushes and the lanes' hedges were thinned to keep the band near 250,000 triangles
(the old ground 118,000): 275 fps, GPU 2.37 ms against 2.31 before. Re-lit. Before and after (wide, then the match camera):
`review5/meadow_farmland_built.jpg`.
