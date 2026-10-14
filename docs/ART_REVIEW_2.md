# Art review 2: the gap to a competitive product (2026-10-14)

A second whole-game review, after the first one's work (`docs/ART_REVIEW.md`) landed. Animation and modelled weapons are left out, as
the owner asked: they come at the end. The bar is the shipping quality of **Rivals of Aether II** and **the reference platform fighter**, not
"good for an indie prototype".

Fresh captures (main at `15bb8ad`), all at 1280x720: every stage at play distance with the HUD, a close-up, a strong hit, a
fighter crop at play scale and a greyscale copy of the canyon. They are in `docs/screenshots/review2_*.png`. The comparison is made from
knowledge of the reference games, not side-by-side captures.

## Verdict

**We look like a charming, coherent indie game, but not yet like a commercial platform fighter.** The first review's fixes moved the
*world* forward a lot: the stages have ground, depth, life and built corners; effects and UI are close to the bar. The gap that is left
is mostly in the **fighters** and in the **value structure** that makes fighters pop off a stage. Both competitors spend their art budget
there first.

## Scorecard (1 to 5, against RoA II and the reference platform fighter)

| Area | First review | Now | In one line |
|---|---|---|---|
| UI, HUD and menus | 4 | **4** | Cohesive, clear, like the reference platform fighter; near the bar. |
| Combat effects | 3 | **3.5** | Painted bursts, smoke and the KO plume read well and feel like ours. |
| Palette and stage mood | 4 | **3.5** | Still lovely per stage, but every stage is as saturated as the fighters (see 2). |
| Backdrops (depth, life) | 3.5 | **3.5** | Lived-in and layered; busy, and it competes with play. |
| Stage construction | 2.5 | **3** | Built corners, skirts and shores help; still boxes with tiled walls. |
| Lighting and form | 3 | **3** | Fighters have a lit side now; stage geometry is still lit flat. |
| **Fighter rendering (surface, shading, line)** | 2 | **2.5** | The biggest gap left. |
| **Value hierarchy (fighters against stage)** | n/a | **2** | Newly found; the most fixable large gap. |

## The weakest areas, ranked by impact

### 1. Fighter rendering

`review2_fighters_play_scale.png`, `review2_closeup.png`.

At play scale our fighters read as **soft, streaky gradients with a wobbly line**. RoA II's and the reference platform fighter's read as **clean, designed
shapes**.

- **The shading is mushy.** The body goes from lit to shade through a soft ramp, warm-to-pink colour shifts and painted-shadow hue
  rotation, so an orange fighter shows bands of orange, salmon and pink streaks rather than one light shape and one shadow shape. RoA II
  is two (sometimes three) flat tones with a hard edge. The reference adventure is the same.
- **The ink line is noisy.** The screen-space outline doubles up, breaks and draws stray strokes inside the body (the chest and legs in
  the crop), which reads as unfinished. Competitors' fighter outlines are a clean, near-constant weight round the silhouette, with only
  deliberate interior lines.
- **There is no costume construction.** Bodies are bare blobs with a hat and a scarf. RoA II fighters are fully designed costumes
  (layered cloth, belts, trims, shapes that tell you who they are), and the reference platform fighter's fighters carry a lot of material detail. Our wardrobe now
  gives silhouettes, but the torso and limbs are still one smooth colour.
- **The fighters are small and low in contrast** for how much screen they get (next point).

### 2. Value hierarchy: fighters do not pop

`review2_canyon_grey.png`, `review2_stage0.png`, `review2_hit.png`.

In greyscale the canyon's fighters are **the same mid-grey as the sand and the wall behind them**, and in colour they are the same orange
as the stage. The meadow's lime lawn is as saturated as any fighter. The reference platform fighter and RoA II deliberately keep the stage **lower in saturation
and contrast** than the fighters, and give fighters a rim or value separation so they are the brightest, crispest thing on screen. This is
the cheapest large win available.

### 3. Stage construction: boxes and tiles

`review2_stage2.png`, `review2_stage3.png`.

- The fighting surface is still a rectangular slab. The new corners, skirts and shores dress the box; they do not change its silhouette.
- Wall textures still repeat visibly: the beach and the meadow share the masonry tile.
- Platforms are flat planks with no designed identity. RoA II's platforms are props that belong to the place (a branch, a sign, a
  crate stack). The reference platform fighter's are designed silhouettes.
- There is **no foreground layer**: nothing frames the play space in front of the stage. Both competitors use foreground elements
  (foliage, rocks, rails) at the bottom corners to sandwich the action in depth.

  This overlaps the deferred modelled-props work; a few foreground framing pieces can be built in code now.

### 4. Backdrop busyness

The backdrops are full of things (cottages, islands, windmill, cars, string lights) at similar contrast to the play space. The reference platform fighter keeps
backgrounds quieter and lower in contrast behind the fighting area, with detail concentrated away from it. Ours compete for the eye.
Fixing point 2 fixes much of this.

### 5. Lighting on stage geometry

The stage blocks, platforms and props are lit by the same soft ramp. There is no contact darkening where platforms meet the ground, no
rim on the stage silhouette, and no light falloff across the stage. RoA II paints light into its stage textures; the reference platform fighter lights richly.
Moderate impact, after 1 and 2.

## What is at or near the bar

- The **HUD and menus**: one coherent design system.
- **Effects**: the painted hit bursts and the knock-out plume have their own identity.
- **World-building**: the stages feel like places (farm, street, shore), which many fighters lack.
- **Stage mood**: five distinct palettes in one style.

## Recommended order of work (excluding animation, modelled weapons and modelled props)

1. **Value hierarchy pass, all stages** (fast, large). Lower the saturation and contrast of stage surfaces and backdrops a step against
   the fighters; give fighters a slightly brighter lit tone and a crisp rim so they are the most contrasty thing on screen; check every
   stage in greyscale.
2. **Fighter shading: crisp two-tone cel.** A hard light/shadow step on fighters (no soft ramp, no streaky hue banding), one clean shadow
   colour per fighter, and a broad soft highlight. Keep the form light.
3. **Fighter line: clean silhouette ink.** Constant-weight silhouette outline (the inverted-hull outline the props use) for fighters, and
   the screen-space interior lines weakened on fighters, so the noisy doubled strokes go.
4. **Costume construction** in the wardrobe: painted tops, belts, trims, gloves and boots as real pieces, so torsos and limbs have
   designed shapes, not one colour.
5. **Foreground framing** layer per stage (code-built rocks, foliage, rails at the bottom corners) and **platform identity** (each
   platform a themed prop).
6. **Break the tiling**: a distinct wall pattern per stage, plus large-scale variation over the wall.
7. **Stage lighting**: contact darkening where platforms and props meet the ground, a rim on the stage silhouette.

Deferred to the end, by the owner's call: modelled props, weapon models and animation. The fighters' base body itself is also a modelled
asset; if points 2 to 4 cannot bring the fighters to the bar, a remodel of the base body belongs in that end pass.
