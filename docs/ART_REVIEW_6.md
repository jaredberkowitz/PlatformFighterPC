# Art review 6: a game-wide rating against the competition

Captured from main at sim 128, art update 40, 1280x720 (`docs/screenshots/review6/`). Every area is rated 1 to 5 against the three
products the owner named as the competition:

- **The reference platform fighter** (the latest home-console entry): the bar for combat readability, animation weight, effects, UI.
- **A dimensional stylized indie platform fighter** (the second entry of the aether series): 3D cel characters with 2D-feeling motion,
  very strong animation and effects at a small team's budget, the closest comparable product.
- **PEAK**: the bar for the character look we chose (round heads, box torsos, chunky outfits, charm).

Scale: **5** matches the competition, **4** close (a player would not call it amateur), **3** decent but visibly behind, **2** reads as
unfinished, **1** placeholder.

| Area | Rating | Versus the competition |
|---|---|---|
| Stage environments (meadow, grove) | **4** | Painted, layered, lit; close to the stylized fighter's stages. |
| Stage environments (desert, beach, city) | **3.5** | Good composition; walls and fronts still busy; the beach's sea improved but flat at distance. |
| Lighting and rendering | **4** | Cel ramp, rim, bloom and per-stage moods hold up; fighters keep their shading. |
| Faces and expressions | **4** | Readable, charming, PEAK-like. A dark crack under the mouth on several fighters (a defect). |
| Character model and proportions | **3** | Clean, symmetric, PEAK proportions; still plain next to PEAK's (no shoulder or knee definition, small mitten hands at match scale), and a hard dark ring where the head meets the body. |
| Outfits and accessories | **3** | Belt, collar, boots, cargo shorts read; PEAK's clothing has thicker, puffier shapes and printed texture. The main menu's Play card shows a bare body (no outfit). |
| Character animation: idle, run, jump | **2.5** | Serviceable, generic; no secondary motion, little personality per class. Both competitors give every fighter a distinct idle and run. |
| Character animation: attacks | **2.5** | Now structured (chamber, launch, held strike, follow-through) and arms bend as a person's, but still stiff next to either fighter: the body rarely commits (no rolls through aerials, few big silhouette changes), and the aimed arm still reads mechanical. |
| Combat effects (sparks, trails, launch) | **2.5** | The swing arc is large and the hit spark small; launch trails thin; no strong-hit accents. Both fighters are a 5 here: layered, bright, short-lived, scaled to the hit. |
| Weapons and props | **3** | Clean hilts and blades; flat colours, no wear; the shield and staff plain. |
| Main menu | **4** | Bold, cohesive, live 3D. |
| Character select | **3** | Grid and panels good; the settings pills (stocks, time, stage) are plain grey bars unlike everything else. |
| In-match HUD | **3** | Percent and portrait read well; the "GAME!" call on a flat grey band; stocks tiny. |
| Camera and framing | **3** | Fighters small at match scale (about 10% of screen height); the reference fighter keeps them larger, the stylized fighter zooms tighter in 1v1. |
| Overall gameplay readability | **4** | Fighters stand out on every stage (look bench 0.87); effects don't hide them. |

**Overall: about 3.2.** Environments and rendering are near the competition; the characters in motion (animation and effects) are where
the game still reads as amateur, and they are what players watch most.

## Points of improvement, ranked by gain for the effort

1. **Combat effects** (2.5 → 4): hit sparks scaled to the hit with a bright core and a short shockwave ring; a thinner, faster-fading
   swing arc; a fuller launch trail and a strong-hit flash. The cheapest big gain; both competitors lean on effects for weight.
2. **Attack animation, class by class** (2.5 → 3.5): the owner's reference gifs as the bar. Aerials roll the body through the
   follow-through; each class's signature moves (the sword's smashes and airs, the kicks) get their own key poses instead of the
   shared short specs; the off hand aims in the wind-up.
3. **Idle, run and jump with character** (2.5 → 3.5): a distinct stance and run per class family (sword, brawler, heavy, quick), a
   squash on take-off and landing, a little secondary motion on the head and hat.
4. **Character model detail** (3 → 4): soften the head-to-body ring (a lighter, shorter chin line), fix the dark crack under the mouth,
   a little shoulder, elbow and knee definition, bigger mitten hands at match scale.
5. **Outfits** (3 → 4): fuller, puffier shirt and shorts shapes and a painted cloth texture, as PEAK's; dress the main menu's Play
   fighter.
6. **Camera** (3 → 4): a slightly tighter 1v1 framing, so the fighters (and all of the above) read larger.
7. **Select and HUD polish** (3 → 4): the settings as the kit's angular panels; the "GAME!" call on a ribbon; larger stocks.
8. **Stage fronts and the beach's distance** (3.5 → 4): the busy fronts recede further; the far sea calmer and paler.

Then the queued character art (the Settler's, Scamp's, Phantom's and Exile's clips and props) and the run mode's new pieces, which the
owner scheduled after this pass.

## Done: the quick fixes (item 4's defects and the menu's bare fighter)

`review6_quick_fixes.png` (before | after: the face close up, the main menu):
- **The dark crack under the mouth** was a slit in the mesh down the middle of the face: points of the remeshed body lay exactly on the
  mirror plane, so the cut left a strip of faces with no partner, and the outline behind showed through. The body is nudged a hair off
  the plane before the cut (`make_base_body.py`); the welded mesh has no open edges.
- **The hard ring at the neck**: the painted chin line drawn at 28% ink instead of 65%, and narrower (`skin.gdshader`).
- **The Phantom and the Exile had no outfits** (added after the outfit pass), so the menu's Play card (the last fighter played) showed
  a bare body: the Phantom a midnight body with shades, a cape and the full kit; the Exile a pale body with a scarf and the full kit
  (`roster.gd`).
- Checked: every Godot test; the look bench 0.87.

## Done: combat effects, first pass (item 1)

`review6_effects.png` (rows: a strong hit, a launch, a light hit; before | after):
- **Hit bursts a size up**, the light ones most (`effects.gd` `hit`: size 0.8 + 0.75 x strength, was 0.5 + 0.6): a jab was a speck at play scale.
- **A contact ring on every hit**: a quick small one on light and medium hits; a thicker shockwave on strong ones.
- **More sparks on light hits** (0.25 + 0.75 x strength).
- **The swing arc** lighter and shorter-lived (`ARC_OPACITY` 0.62, was 0.82; `ARC_FADE` 3, was 5): it was the most solid shape on screen at the hit.
- **A launch trail** (`LaunchTrail`): a painted ribbon (ink rim, pale-blue edge, white core) along the flight, widest at the fighter.
- **Launch streaks actually show**: they were gated on 0.5 units a frame, faster than nearly any launch, and on the fighter's own velocity
  (small in a launch) instead of the whole flight; now 0.2 on the flight.
- Checked: the fighter, match, motion, art and online tests; the look bench 0.87; frame time on a strong hit unchanged within noise
  (126 vs 130 fps on the RTX 4050 laptop).
- Next in effects: a strong-hit accent (a brief two-tone frame on the KO-capable hits) and per-class spark colours.

## The owner's two further plans, folded in (2026-10-14)

Two external polish plans (versus stages, from a meadow match; the single-player run's rooftop) agree with this review's ranking and add
specifics. The combined order, keeping the refine-before-replace rule, matched before/after captures and a frame-time check per step:

1. **Combat effects** (done, first pass above; next: a brief strong-hit accent, landing and dash dust in the same painted family,
   per-weapon spark colours, the arc's sorting checked against transparent props).
2. **The meadow platform's finish** (their priority two): the grass-to-soil lip as a few broad deliberate tongues, the soil wall as large
   forms with the stones as supporting detail, the corner blocks built like the rest, the top and the underside one designed object.
   Then the same rules for the other stages' fronts (this review's item 8).
3. **Attack animation, class by class** and **idles and runs per class family** (items 2 and 3), with landing compression and grounding.
4. **Character model detail** and **outfits** (items 4 and 5).
5. **Background cohesion**: value steps between the stage, near scenery and far hills; the windmill, houses, trees, fence, bales and
   islands checked for one geometry and outline language; distant things lower in contrast.
6. **Camera and off-screen feedback**: framing tested on long launches and recoveries (no gameplay change without the owner's say),
   player markers clear of the HUD and the screen edge, off-screen indicators checked.
7. **Select and HUD** (item 7), one panel system.
8. **The run's rooftop** (the second plan): the roof's top, fascia and underside told apart, tile seams lighter and less regular, the
   roof and the facade as different materials; the skyline's rooflines, setbacks and window depth varied, far buildings shifted toward
   the sky; the run HUD's hierarchy (distance first, combo second, the rest supporting) built from the same panel rules. The run reuses
   the city's art (docs/modes/RUNNER_PLAN.md section 10), so the city stage gains with it; the run's shadow crawl on pans is checked here.
9. Performance and regression across every stage, four fighters, supported resolutions.

## Done: the meadow block as one of its own islands (plan item 2, first pass)

The owner: the aim now is one cohesive style across every asset. The meadow's sky islands are earth with a turf cap, banded strata and a
keel; the stage was a different object: brick buttresses at the ends, a front dotted with two dozen stones, a row of rubble along its foot.
`review6_meadow_island.png` (before | after: match view, wide view):
- **No masonry on the earth islands** (the meadow and the grove): `_ledges_and_buttresses` skips them.
- **The soil's beds are the big forms** (`earth` pattern): one boulder, four middling stones and eight pebbles per tile, from 3, 7 and 16.
- **No rubble skirt** under the earth islands: the wall's earth carries straight on into the keel, as the sky islands' does.
- Checked: the art, match and fighter tests; the look bench.
- Next for cohesion: the grove's stepped underside is its collision shape (the sim's solid blocks), so it stays unless the owner wants
  the stage itself changed; its art could round the steps' corners; the desert's, beach's and city's fronts by the same rule
  (built where the stage is built, earth where it is earth).

## Done: the other stages' fronts by the same rule (built where built, rock where rock)

`review6_fronts.png` (before | after, wide): the desert's block is carved strata like the mesas round it, so its stone pillars go
(`_ledges_and_buttresses` skips `strata` too); the beach's sea wall is built, so it keeps its masonry, but in warm sandstone
(`seawall` stone 0.86/0.74/0.6, was cool grey 0.72/0.7/0.68), the one cold thing on a warm sand island. The city's rooftop already
matches its skyline and is unchanged. Checked: the art and match tests; the look bench. The owner asked to keep the meadow as it is.

## Done: fighters in the middle of the stage's depth (the owner, after the reference fighter)

The fighters stood 0.3 units behind the stage's front lip, so the floor read as a ledge they balanced on. The front now comes 2.0 toward
the camera (`stage_art.gd` `STAGE_FRONT`, used by `stage_view.gd` `FRONT_Z` and the undersides): floor in front of them, the platforms
centred on them (4 deep, 2 either side), the run mode's rooftops likewise (it shares the constant). No gameplay change: the fighters' own
plane is untouched. `review6_depth_centre.png` (before | after, the beach). Checked: the art, match, fighter, content and runner tests,
a run capture, the look bench.

## Done: select and HUD on one panel system (item 7)

`review6_ui.png` (before | after): the select screen's five settings (players, rules, stocks, time, stage) are dark kit cards with the
label in a quiet white and the value in the accent yellow (`ui_kit.gd` `Btn.setting`), as the menu's utility tiles are, not pale bars;
the "GAME!", "TIME!" and countdown calls sit on an angular ribbon in the title's style, sized to the word (`match_hud.gd` `_banner`), not
a flat grey band; the stocks a size up (1.3x). Checked: the match, controls, creator and select grid tests.

The owner scheduled the full animation overhaul for Saturday 2026-10-17 (plan in memory); until then the other items continue.

## Done: the rooftop's top, edge and facade told apart (item 8, first pass; the city stage and the run)

`review6_rooftop.png` (before | after: the run, the city close up): the roof's edge is a stone coping standing proud of the facade, lit
on top with a shaded face and a dark drip line under it; the roofing is a warm tar-grey a step apart from the violet facade; its seams
fall at uneven spacings. Checked: the art, match and runner tests; the look bench. Next here: the skyline's rooflines and window depth.
