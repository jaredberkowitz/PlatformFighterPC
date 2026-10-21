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
