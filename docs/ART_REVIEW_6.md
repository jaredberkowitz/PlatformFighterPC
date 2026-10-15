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
