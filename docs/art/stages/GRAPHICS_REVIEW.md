# Graphics review (2026-10-22): what every stage is missing

The owner, after the ground pilots: "we are redesigning [the background ground] rather than making graphical improvements. We should take
a step back, do an analysis, and figure out what every stage is missing." So this review leaves layout alone. It looks only at
rendering quality: shadows, light, atmosphere, materials, sky and water. Evidence: every stage as it ships (the grove with its new
terraces) at the match camera and wide, close crops of each background (`review5_crops.jpg`), measured regions, and one test (below).

The meadow's shelf pilot is parked unshipped; the grove's terraces stay shipped.

## Game-wide gaps (one system each, every stage gains)

### 1. Nothing in the background is grounded (the biggest gap)

The windmill, the cottages, the pines, the torii and the side trees, the trestle and water tower, the buttes, the beach's houses and
boats: none casts a shadow you can see, so each sits on the land like a cut-out laid on top. The references anchor everything: a crisp
drop shadow under every tree and house, and shade where things meet the ground.

- The sun's shadows stop 70 units from the camera (`Lighting.apply`, `directional_shadow_max_distance`), and the background starts at
  about 75.
- **Test:** with the range at 320 and four splits (a temporary `--shadow-far` flag, now removed), the frame time did not change (225 fps
  against 230 on the meadow), but the shadows **barely read**. The buttes shade their own faces; trees and houses drop almost nothing
  visible on the ground. Cast shadows are kept light on purpose (`shadow_amount` 0.8 of a light ramp), and the air lifts the far
  ground toward the haze, which washes out what is left.
- **The fix is a system, not a setting:** bake the sun's shadow into the background, the way `light_stage.py` already bakes contact
  shade into the modelled pieces. Extend it from occlusion to the key light's own direction per stage. Every tree, house, gate and butte
  then lays a crisp, cool shadow on the land (the canyon's buttes long sunset shadows, the grove's trees on the terraces), at no
  runtime cost. Add a contact darkening under each placed prop.

### 2. The air washes the middle distance grey

The haze lifts and greys everything past the play area, so the middle distance loses its colour and contrast together.
- Saturation of the middle distance at the match camera: beach town **0.12**, grove terraces **0.17**, meadow hills **0.21**, canyon
  middle 0.38 (one pink value across the whole band).
- The references keep the middle distance clear and coloured (the papercraft camp's grass reads at about 0.7 saturation), and haze only
  the far ranges.
- **Fix:** retune the air per stage. Start it later, lift less, keep the hue (`[air]` in the palettes and the post pass's
  `haze_*`). It's a settings pass on one system, cheap, and every stage gains.

### 3. Background surfaces are one flat tone per face

Big faces in a single colour: the hills, terrace tops, house walls, butte faces, the far land.
- The near pieces have printed paper, lit bevels and contact shade; at background scale the print is too fine to read, and the lit
  bevel only catches edges.
- The references' backgrounds are simple but never flat. Each face carries a soft gradient (lit toward its top or the sun, cooler
  toward its foot), and the grass has light and dark patches.
- **Fix:** a background material layer in the toon shader. Give each face a gentle gradient by height and facing (the shader's
  `pf_paint` warm-top / cool-foot exists but is too weak to see), and add large-scale grass and stone mottling at a scale that reads
  from the camera.

### 4. Plain skies (smaller)

Smooth gradients (the city's a flat magenta). The references have a soft horizon glow and sometimes a band of far haze or cloud layers.
Lower gain than 1 to 3.

## Per stage

| Stage | What it is missing (graphics, not layout) |
|---|---|
| **Meadow** | Shadows under the windmill, cottages, pines and haystacks; the middle distance is a pale band of field patches at one value (saturation 0.21); the grass lacks the reference's lime and its soft light-and-dark patches. |
| **Grove** | The side trees, terrace trees, gates and the tree village cast nothing on the terraces; the terrace tops are one flat green; the abyss is one flat violet with no glow or depth (the reference's misty violet light, soft shafts through the canopy); the middle distance greyed (0.17). |
| **Beach** | The water is one bright cyan with stripe-like ripple lines (the reference: clear turquoise, lighter and see-through near the pier, soft painted highlights, darker with depth); the town across the bay is washed grey (0.12); boats and houses cast no shadow; the late-afternoon light has no long shadows. |
| **Canyon** | The buttes and the arch cast no long sunset shadows across the floor (a sunset's strongest cue); the middle distance is one pink value under the haze; the floor's sheets are flat. |
| **City** | The towers are flat slabs (windows painted on, no depth to the facades, no shade where buildings meet); the lit windows don't glow; the sky is a flat magenta. |

## Recommended order

1. **Baked sun shadows and contact for the background** (gap 1): one system, every stage, the largest gain, no runtime cost. Pilot it on
   the canyon (long sunset shadows) and the grove (trees on the terraces), then roll it out.
2. **The air retune** (gap 2): a settings pass per stage, quick.
3. **The background material layer** (gap 3): a shader change; pilot on the meadow's hills and the grove's terraces.
4. **Per stage:** the beach's water shader, then the city's facades and window glow.
5. Skies.

## Pilot: grounding the background (canyon and grove, 2026-10-22)

**What I found first.** The game's realtime sun shadows never reach the background. That's not only the 70-unit range. Out to 260 units,
with four splits, even a plain white test box standing on a grove terrace casts nothing on a plain white test plane beside it, while the
same pair on the play area does. The shadow map does reach that far (with the bias lowered, acne appears on the terraces), but the
casters don't land in it. Finding out why is engine-level digging; it may be worth doing later, because realtime shadows would be the
crispest answer. Meanwhile:

1. **A bug that greyed every background top** (a game-wide fix, shipped in this pilot). The faceted shading rebuilds each face's normal
   from screen derivatives, and on flat, up-facing faces that normal came out turned over, so every grass, sand and terrace top in the
   background was lit as an underside, in its shade colour. That's a large part of the greyed middle distance. Flat tops now keep their
   own normal (`toon_common.gdshaderinc`, `facets` skips faces whose normal points straight up); nothing else changes. The grove's
   terraces read lime instead of grey-green, and the canyon's floor warmer.
2. **The grove's terrace trees were missing in the game.** Re-lighting re-exported the model without its `treeslot` empties.
   `light_stage.py` now writes them back.
3. **Baked sun shadows** (`light_stage.py --sun x,y,z --sun-pieces ... --sun-strength --sun-grid`): the chosen background pieces get
   their up-facing faces cut to a grid (3 to 3.5 units), each vertex casts five rays toward the stage's sun past everything on the
   stage (not the far ranges or the sky), and what is hidden is darkened (strength 0.6, deeper than the contact shade's floor). Those
   pieces also keep their contact shade at full strength into the distance. Canyon: sun travel (0.797, -0.342, 0.498), the low sunset
   sun; grove: (0.806, -0.53, 0.262).

**Result** (`review5/sun_pilot.jpg`, today left, the pilot right; `review5/sun_arch.jpg` close): the arch's legs and the buttes lay soft
shadows across the canyon floor toward the camera, and the trees darken the terraces under and beside them. But at vertex resolution
the shadows are soft smudges, not the reference's crisp drop shadows, and the distance's air lifts them further. Cost: the grove's
ground is 6.8 MB (was about 2) and the canyon's 4.2 MB (was 1.3); frame time unchanged (canyon 297 fps, grove 249 fps, GPU 2.6 to
2.7 ms).

**Recommendation:** keep the pilot (the shader fix and the restored trees are clear gains; the baked shadows a modest one). For crisp
shadows, either dig out why the engine's shadows stop at the play area, or bake the shadows into a texture rather than vertices.

## Realtime background shadows (2026-10-22, the owner: "dig into why the engine's shadows stop at the play area")

**Why they stopped.** Not the engine and not the range alone. Three causes:
1. **Far casters stretched the shadow map.** The ranges, dune flats, mist and abyss sheets are hundreds of units across and cast
   shadows. In the sun's shadow map they stretched each split's depth range so far that every shadow sharing a split was lost, the
   play area's included. `StageView.trim_shadow_casters` now turns casting off for anything wider than 200 units or further back than
   180 (it still receives). This was found by elimination: with those turned off, test boxes on the terraces cast at once.
2. **The shading washed out what was left.** The scenery's shade is a light pastel (the paper look), and the ambient light is the same
   in sun and shadow, so a full cast shadow read only about a sixth darker. The toon light now multiplies a lit face that something
   shadows by the stage's `cast` tone (`pf_cast_shade`), only behind the play area (more than 6 to 14 units back, never on fighters),
   fading out from 110 to 150 behind, where the map is coarse.
3. **A 5-degree sun** blurred every far shadow to a smudge. Stages with background shadows use 1 degree.

Per stage, in `Lighting.STYLES` (`sun_shadow`: [reach, splits, sun size]; `cast`: tone): canyon, grove, meadow and city reach 260
units in 4 splits (the first split as far as the old 70, so the play area keeps its detail), the beach 190 (beyond it, the headland's
flats shadowed each other in stripes). Tones are cool violets and blues against each stage's warm light.

The vertex-baked sun from the pilot (`light_stage.py --sun`) is no longer used: the canyon and grove grounds were rebuilt and re-lit
without it (1.3 and 3.1 MB again, from 4.2 and 6.8). The tool keeps the option.

**Result** (`review5/shadows_built.jpg`, `review5/shadows_close.jpg`; before = the review's captures): the canyon's arch and buttes lay
crisp, long sunset shadows across the floor; the grove's trees and gates shadow the terraces; the city's scaffolds and people shadow
the street; the beach's town is shaded; the meadow changes least (the sun is high and the land falls away from it). Cost: GPU about
0.4 ms on the meadow (4.3 against 3.9 at the same moment; the machine was running slower than earlier in the day, so absolute frame
rates aren't comparable with earlier numbers). Tests pass.

## The air retune (2026-10-22)

Middle distance (75 to 110 units from the match camera) measured with the camera's depth (`--depth`): saturation meadow 0.18, beach
0.29, grove 0.29 (canyon 0.49 and city 0.53 were already coloured and are unchanged). Two hazes stacked there: each mesh's colour lifted
toward the palette's pale haze at build time (`[air]`), and the finishing pass's haze from 14 behind the stage. On the meadow a third:
its hills and fields kept only 72% of their own saturation (`StageBake._calm`).
- Meadow, beach, grove `[air]`: start, end and far 40 further back, a third of the lift (the far ranges keep their far_lift).
- Their finishing haze from 45 behind the stage, at most 0.1 (meadow, beach; were 0.24) and 0.04 (grove; was 0.08): `haze_start` in
  `Lighting.STYLES` finish.
- The meadow keeps all its fields' and hills' saturation (`calm=1.0` in meadow.cfg), as the reference's lime grass.

Result: middle distance saturation meadow 0.18 to 0.29, beach 0.29 to 0.37, grove 0.29 to 0.35 (`review5/air_built.jpg`). The far
ranges stay lighter and cooler, so depth still reads.
