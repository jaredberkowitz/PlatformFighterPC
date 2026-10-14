# Art review 5: the visual polish roadmap

The owner's visual polish roadmap (2026-10-14) combined with what the earlier reviews built. The roadmap's rule is ours too: **refine
before replacing**. Tune a parameter before a shader, a shader before a mesh, and change one major thing at a time against matched
captures. The rendering and material guide its Phase One asks for already exists (`docs/VISUAL_STYLE.md`: one cel material family,
outline rules, stage profiles, the effect style guide, the look bench); this review keeps it current instead of writing a second one.

References are named by role here, as in every doc: the reference platform fighter (readability, UI hierarchy, effects), the reference
cel-shaded adventure (painted water and shapes), PEAK (tactile, chunky characters), and a dimensional stylized platform fighter.

## Phase Zero: the baseline

Captured at 1280x720 from main at sim 123, art update 33. Kept at 60% in `docs/screenshots/baseline5/`:

- `menu.png`, `select.png`, `hud_game.png`: the main menu, character select, the HUD at the end of a match.
- `stage0..4.png`: every stage at match scale with the HUD (meadow, sunset desert, beach, city at night, grove); `close0..4.png`
  the same at close range (`--cam=7`).
- `combat_fx.png`: a forward smash on the beach (hit, launch, flight) and a sword tilt on the meadow (hit, hitlag, launch).

Test setup, repeatable: `main.tscn -- --demo=stage --stage=N --match --noecb --chars=0,6 --shots=<dir>` (wide), the same with
`--noui --chars=2,9 --cam=7` (close); `--demo=smash` / `--demo=combat` for effects; `menu.tscn -- --shot=<png> [--keys=ENTER]` for
the screens; `tests/hud_shot.gd`. (The demos use the classes' plain looks; real matches dress the built-ins in their outfits.)

Renderer: Godot 4.7 Forward+ (Vulkan). Light: one key light per stage profile (`scripts/lighting.gd`) through the cel ramp
(`shaders/toon_light.gdshaderinc`), cloud shadows (`pf_clouds`), stage AO under props. Outlines: inverted hulls on fighters
(`outline.gdshader`) and an ink pass on scenery in the finishing pass (`post.gdshader`, which also grades, hazes and blooms). Water,
sky, clouds and sun have their own shaders (`water`, `sky`, `cloud`, `sun.gdshader`).

## The five highest-value opportunities, ranked

### 1. The sea (the beach stage; the biggest single surface in any stage)
- **Problem**: one saturated cyan from shore to horizon; many thin, evenly spaced white dashes all the same length; no broad wave
  forms, so the eye reads a pattern, not water. Behind the clouds a flat, see-through lens shape cuts across the bank of cumulus on the
  horizon (it reads as a rendering fault).
- **Cause**: shader tuning, not geometry (`shaders/water.gdshader`: bands, streaks, crests, glints and foam all exist, but the crest
  rows are dense and uniform, the colour bands are too close in value); the lens is a cloud of the long low kind in the horizon bank
  (`stage_art.gd` `_ocean` → `_clouds(... [1, 0, 1], 1.9)`), flattened so far it is nearly all soft edge.
- **First change**: broad, slow light and dark swaths under the detail; fewer, chunkier crests of varied length; the distance
  calmer and paler; the flat bank cloud replaced by the heaped kinds. Then the props (lighthouse, jetty, palms, islands) per the
  roadmap's 5.2.
- **Success**: at match scale the sea reads as water moving one way, not a pattern; fighters stay the strongest thing on it; matched
  before/after of `stage2` and `close2`; the motion is calm enough not to pull the eye during a fight.

### 2. Stances and the shared animations
- **Problem** (the owner): the hand-to-hand fighters' idle has the arms extended, as if going for a hug, not a guard; the other idles
  are better but every one needs work. Swings and kicks read, but stances are where the fighters spend most of their time.
- **Cause**: animation (`art/blender/make_rigged_blob.py` `idle` and the per-class stances; `scripts/anim_library.gd`). The arms were
  held out on purpose so they read against the body (pass 68); that went too far for the unarmed classes.
- **First change**: per-class guard stances: fists up in front of the chest for the unarmed classes (elbows in, forearms up), the
  weapon classes' blade or tool held ready; arms still clear of the body's silhouette from the gameplay camera.
- **Success**: each class's stance reads as ready to fight in a still frame at match scale, from both facings; no arm through the body.

### 3. Combat effects
- **Problem**: the sword's swing arc is a large, nearly opaque pale crescent that covers both fighters at the moment of the hit (the
  moment that most needs to read); hit sparks are small next to it; the launch trail is a thin line.
- **Cause**: effect design (`scripts/fighter_view.gd` swing arcs, `shaders/fx.gdshader`, the effect style guide in `VISUAL_STYLE.md`
  section 5).
- **First change**: the arc thinner and see-through behind its leading edge, fading from the tip back; impact sparks scaled with the
  hit's strength; a fuller launch trail on strong hits.
- **Success**: in `--demo=smash` and `--demo=combat` both fighters' silhouettes stay visible through the hit frame; strong and light
  hits are told apart by the effect alone.

### 4. UI consistency
- **Problem**: the menu's cards are strong, but character select's settings (Players, Rules, Stocks, Time, Stage) are flat grey pills
  unlike every other panel; the "GAME!" call sits on a flat translucent grey band; the stock icons are small for their importance.
- **Cause**: UI (`ui/select.gd`, `ui/ui_kit.gd`, the HUD in `scripts/main.gd` / `ui/`).
- **First change**: the settings as the kit's angular dark panels with a coloured value; the "GAME!" call on an angular ribbon in the
  title's style; stocks a size up. A short UI rules section added to `docs/MENUS.md` (type scale, panel shapes, button states).
- **Success**: every panel on the menu, select and HUD comes from the same few kit pieces; focus is visible on every control from the
  keyboard and a pad.

### 5. Fighters' value and form at close range
- **Problem**: bright bodies (yellow, orange) glow at close range: bloom and the rim light flatten them toward one colour, so the cel
  shading that gives them volume nearly disappears; at match scale they hold up.
- **Cause**: lighting and post (`post.gdshader` `bloom_fighter`, the rim in `toon_light.gdshaderinc`, `lighting.gd` per stage).
- **First change**: less bloom on fighters and a rim that scales down on light bodies, checked across all body colours on every stage.
- **Success**: a visible light/shadow split on every body colour in `close0..4`; the look bench still passes.

### Also noted (lower)
- **Stage fronts**: the beach's brick wall and the desert's strata fill the bottom third at match scale with strong, busy pattern;
  they should recede (roadmap Phase Six). The meadow and the grove handle this better.
- **Depth**: the grove's foreground bushes, bottom left, are pale and soft and read as fog more than as near leaves.

## The plan (the roadmap's order, adjusted to the audit)

1. Baseline (done, above).
2. Rendering standard: `VISUAL_STYLE.md` kept current with each step below (it already holds the material, outline and effect rules).
3. **The sea and the beach's props** (opportunity 1).
4. **Fighters' shading** (opportunity 5), then **stances and shared animation** (opportunity 2), then the Phantom's clips (deferred
   by the owner until after this pass).
5. **UI design rules and the select and HUD fixes** (opportunity 4).
6. **Combat effects** (opportunity 3), checked in real matches on bright, water and dark stages.
7. **Stage fronts and depth** (the lower notes).
8. Performance, resolutions, pad navigation, and every stage again against this baseline.

Each step: a matched before/after, a check in motion, the tests and the look bench, then the desktop build.

## Step 3: the sea

**First change (landed)**, `review5_sea.png` (left the baseline, right after; match scale above, close below), `review5_sky_lens.png`:
- **The lens across the sky** was the painted far coast (`stage_art.gd` `RIDGES["ocean"]`): its foot sat under the sea, so only a thin
  faded hump showed, at cloud height (confirmed by tinting it red). Removed: the horizon is the modelled cloud bank and the distant
  islands. (Not the clouds' flat bases, the first suspect; that change was tried, made no difference and was reverted.)
- **The water** (`water.gdshader`): broad slow swaths, a little deeper and a little lighter, stretched along the shore under all the
  detail; a soft lit face behind each crest row, so the rows read as swells rolling in; crest rows further apart, each crest of its own
  length and thicker in its middle; the drifting streaks sparser. Colours, speeds, foam and the Fast tier unchanged.
- **Checked**: matched captures; the look bench 0.87 (unchanged; the ocean rows hold); the art, fighter and match tests pass. The demo's
  checksum is the same before and after (no gameplay change). Beach frame time: GPU 1.60 → 1.71 ms (RTX 4050 laptop, `--perf`).
- **Capture note**: the first baseline was taken with a stale debug bridge that could not read the roster (its overlay said so); it was
  retaken. Two runs of the same build give pixel-identical frames, so before/after pairs are exact. (A crop tool that kept the alpha
  channel made one copy look washed out; the tools in use now drop it.)

**Next in this step**: the beach's props (the lighthouse's light and shadow, the jetty's supports and contact shadows, the palms'
fronds, the islands' layers), then the sea wall's front receding (step 7).
