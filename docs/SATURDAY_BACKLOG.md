# Saturday backlog (2026-10-17, after the usage reset)

## THE PLAN: Saturday's art overhaul, in order (the art session; the detail for each step is in the sections below)

Ordered by dependency: tools and the body come first because everything after is built on them; poses before UI because the
select and results screens show the posed fighters; skies and textures share the Material Maker set-up. Ship a build at the end of
each phase (a playtest checkpoint for the owner), and keep the refinement loop's rules: matched before/after, tests, the look bench.

**Before the session starts (the owner, 2 minutes):** open Blender (so the Blender connection loads) and Material Maker once.

**Phase 0: start-up (short)** DONE 2026-10-17: the build was already shipped (sim 136 has round 42 and the Run icon); the four
Material Maker exports are in `godot/art/materials/mm_exports/` (2048px, to shrink when wired; their `.tres` are Godot 3 format, unused);
Cascadeur and ArmorPaint skipped (owner): pose in Blender, paint with Blender's texture paint.
1. Ship one build of what is committed but not installed (round 42's sun clouds, the menu's Run icon).
2. Re-walk the owner through the Material Maker hand-export (Tools section); the owner exports dry_earth, rock, stylized_wall, wood.
3. Owner's go/no-go on downloads to evaluate: ArmorPaint, Cascadeur (Tooling review section). Ask, name the file, source, size.

**Phase 1: the body (blocks the poses)** DONE: `body-cage` was already merged (art 50). Note: a stale debug DLL in the repo freezes
every demo fighter in its rest pose (content newer than the bridge); rebuild `cargo build -p godot-bridge` after other sessions' sim work.
4. Review and merge the remodelled body and mitten hands (branch `body-cage`, Body section), test and look bench, build.

**Phase 2: poses and animation (the main job; Animation overhaul section, docs/POSE_REVIEW.md)**
5. The fix order from the pose review: arched-back up moves, locked straight arms, flung free arm; then hurt and launch; then
   per-family pose sets; then the smaller items. Cascadeur for key posing if approved; AnimationTree for locomotion blends (no root
   motion). Before/after with `--demo=poses` per class, both facings. Build per class family.

**Phase 3: materials and skies (Tools, Skies, refinement sections)**
6. (Started 2026-10-17: `Materials` prefers `art/stage/png/<tile>.png` over the SVG, greyscale value tiles tinted by the stage colour;
   `tools/art/mm_tile.gd` converts an export (value or detail mode, banded for a painted read). Done: stone <- rock, detail_soil <-
   dry_earth; the canyon sun's flat bands are now two cumulus banks in the sunset's pinks. Next: stylized_wall for the seawall/city
   walls, a bark and a palm trunk worth keeping, one cloud pass across the other stages.) Wire a PNG tile path into `Materials._make`; bring in the owner's exports: soil, bark (realistic, no streaks), stone; a palm-trunk
   texture. Then the painted skies (canyon sun and clouds first, then one cloud set for every stage). Build.
7. The pain-point list (city window grid, beach clouds and wall, stage block ends, the clothes' doubled outline, canyon slash marks)
   and the ink painted into textures.

**Phase 4: the one large UI pass (UI redesign section)**
8. Design system first (type, palette, panels, motion timings), then motion and transitions, the character select redesign, the title
   screen, the results screen (mend `--demo=results` first). Phantom Camera for menu/results moves if approved. Build.

**Phase 5: rendering trials (Tooling review section)**
9. ACES vs Filmic per stage, a light scenery SSAO A/B; keep only what wins on the look bench and the owner's eye.

Not the art session's (other sessions, in parallel): Run mode R3, the versus gaps.

---

The detail behind each step follows.

## Builds
- [ ] Ship a build with everything since art 49 (canyon haze, city windows, pose atlas tool, anything below): export, desktop folder,
      zip, GitHub release (docs/STATUS.md and the desktop-build memory for the steps).

## Run mode review (owner: "lacks the verticality and depth of Punch Quest")
- [ ] Read `docs/modes/RUNNER_REVIEW.md` with the owner. Its proposed order is R2d verticality (three decks, vertical rooms, sure and
      skilled routes), R2e vertical combat and the node map, then R3, then R4a moving platforms and set pieces. Settle its four
      questions and re-plan before starting R3.

## Run mode R3 (shops and items; modes-online agent, can run alongside the animation work)
- [ ] Build R3 from `docs/modes/RUNNER_R3_PLAN.md`: 18 items (stats, properties, consumables), filled pedestals, cards, the reroll
      bell, the HUD strip, and grunts teching from tier 3. Ask the moveset session for the SIM_VERSION, keep versus timing within 2%,
      then land and release.

## Versus gaps (docs/REFERENCE_GAPS.md; done 15 Oct: D3 jab locks 132, O1 perfect pivots 133, D1 shield tilt/pokes 134, O2 footstools 135)
- [x] **O2 footstool** (done, sim 135; notes kept for tuning the [E] numbers). Research so far (the reference, ssbwiki "Footstool jump"):
  - jump while above an opponent's head bounces off it, doing no damage;
  - 2 to 6 in a row depending on fall speed, each about half the last rebound; it can be short hopped;
  - not within 16 frames after attacking;
  - a grounded victim is briefly frozen;
  - an airborne victim tumbles a set distance and can tech the landing;
  - a fighter on the ledge can be footstooled but doesn't fall;
  - it can't cancel a move in progress;
  - a fall caused by one is a KO credited to the footstooler.

  Frame counts aren't published: find them, or mark them [E]. Needs new victim states and a version from the moveset session.
- [ ] O3: down-special B-reverse is per character in the reference (some down specials only, 4-frame window): give the moveset
      session a per-move `reversible` flag to fill in per port. R2 moving platforms later, with stage art.
- [ ] Playtest the four new mechanics on a controller: the drawn shield tilt matches the sim's, the perfect pivot's 1-frame window,
      footstool feel.

## Animation overhaul (the main job; start from docs/POSE_REVIEW.md)
- [x] Up moves arch the back with the belly forward ("inhaling"): reach from the shoulders, hips over the feet (2026-10-17, clips and
      the game's overhead lean/look; `docs/screenshots/poses/sat_up_moves_before_after.jpg`).
- [x] Owner's rule (2026-10-17): the limb or weapon extends to the hit, never the body (no whole-body strike stretch or backward tip on
      the base body; the sword arm reaches most of its length).
- [ ] High kicks (owner refs: Wolf up tilt, Sheik forward tilt; back straight, standing foot planted, leg extended up): angled up and
      forward now (HIGH_KICK 50), but the base body's short legs only reach chest height, so it still reads weakly. Options: a longer
      kick stretch, clip-authored up kicks, or longer legs on the body (owner's call).
- [ ] Sword swings (owner refs: Marth fair, Lucina bair): big arcing swings, no odd arm bends, the free arm tight to the body; back
      attacks turn the torso to face behind for the swing.
- [ ] Forward airs (owner's gold standard: Mario fair; also Wolf fair): large arcing swings with weight, a fist fighter's an arced
      punch or claw; the body adjusts to sell it.
- [ ] Locked straight arms at shoulder height in strikes, follow-throughs and unarmed idles ("hug"): elbow never fully straight,
      follow-through pulls back toward the chest.
- [ ] Free arm flung out sideways (T-pose) in unarmed fairs/fsmashes: guard the chin or chamber at the hip.
- [x] Hurt pose: a real recoil (head and chest snap back from the hit, arms trail, knees give), not the sword across the face.
- [x] Launch pose: much less stretch; curled or spread, limbs and weapon trailing the flight.
- [~] Per-family pose sets: each unarmed family has its own idle (2026-10-17: brawler guard; quick Needles/Dagger/Nodachi; heavy
      Clutch; caster Embers; tool Pockets/Popgun) and its own run (AnimLibrary.CLASS_FAMILY, FAMILY_CLIPS). Still shared: walk and most attacks;
      Dagger and Nodachi show no weapon (queued weapon art).
- [x] Strike frames upside down: the claws lunge keeps the head up, the double kick and up air lean back less, the air jackknife on
      front kicks is 6 on the base body (2026-10-17).
- [ ] Strike frames upside down on flips; one static jump pose (tuck at the apex); shield arm crossing the chest; hands shrinking on
      grips and sideways wrists; crossed-knee idle stances.
- [ ] Locomotion per family (idle, walk, run, dash, jump/fall/land, shield, dodge), then reactions (ledge, get-up, tumble).
- [ ] Before/after every fix with `--demo=poses --gallery-class=N` (both facings); all Godot tests and the look bench.

## Bugs
- [x] Run slides facing left: fixed 2026-10-15 (the shared animation library; a second fighter's build switched the `@m` clips' loop
      off). Regression demo: `--demo=dashleft`.

## Body
- [x] Review and merge the remodelled body and mitten hands (already in main).
- [ ] Open Blender before the session starts so the Blender connection loads (it timed out this time).

## Graphics refinement loop (screenshot every stage, fix the two weakest, repeat)
- [x] Grove: the pale corner shapes (the canopy's top rows; deepened, round 1).
- [ ] More rounds of the loop across all five stages.
- [ ] Canyon platforms: three dark claw-like slashes on the board's top near its end (close camera, `--demo=stage --stage=1 --cam=7`).
      Not a named mark in stage_art.gd; probably the wood tile's grain (`art/stage/wood*`) or a scuff (`scuff.svg`). Find with `--pick`.
- [x] Run HUD fighter plate in the match HUD's dark panel (round 15); the rest of the run HUD to check with the modes-online agent.
- [x] Controls screen hierarchy (round 12).
- [ ] Player-visible text says "strong attack", never "smash" (README, How to play.txt, in-game text); the owner chose "strong attack"
      (2026-10-15). Internal move ids (`fsmash` etc.) later, with the moveset session.
- [ ] Ink painted into the stage textures (stone and plank edges, trims, strata ledges) eased toward the Wind Waker HD reference, where
      colour and value carry the scenery; the post-pass and outline-shell levers made no visible difference (art review 6 round 22).
- [ ] Mend `--demo=results` (it stops in the match) and review the results screen.
- [ ] In-motion checks for the refinement rounds (the outside brief): an attack with its hit effect, both fighters airborne, fighters
      spread so the camera pulls out; look for outline flicker, shadow crawl, effects hiding fighters.
- [ ] The run mode's own city windows (`run_stage.gd` facade values) to match the stage's calmer ones (check with the modes-online agent).

## Pain points found 2026-10-15 (end of the refinement rounds; judged against the competitors, not yet fixed)
- [x] (Sun: round 39; streaks: soft clouds, round 42.) Canyon: the sun is a huge bright disc right behind the stage, and the thin horizontal cloud streaks cut across it as flat lines;
      a smaller sun set lower to one side, the streaks as soft painted clouds.
- [ ] City: the building under the stage is one evenly repeated window grid (the most regular surface left in the game); vary bays,
      a few lit/curtained windows, a ledge or sign. The far billboard: neon lettering since round 41.
- [ ] Beach: the sea wall's stones are still large blocks at play distance; the sky's clouds are flat white shapes with no light side.
- [x] (Six, lower: round 40.) Meadow: the floating islands crowd the top edge of the frame (several clipped by it); fewer, placed to frame the stage.
- [ ] Grove: still the palest stage under its pink light; the light shafts and pink grade could go further if the owner wants it
      nearer the meadow (owner chose to keep the pink-purple mood).
- [ ] All stages: the stage blocks' sides are flat cut-offs at the left and right ends (no end cap or rounded edge); the reference
      stages finish their ends.
- [ ] Fighters: the clothes' outline sits close to the body outline, doubling the line at the waist and cuffs at play size.

## UI redesign toward shippable: ONE large UI pass doing all five (owner, 2026-10-15), on Saturday or right after the work above
- [ ] 1. Motion: animated screen transitions (a wipe in the ink-panel style), tiles that scale/brighten on focus, an animated menu
      backdrop. The biggest "shipped game" signal in the competitors' menus.
- [ ] 2. Character select redesign: a big slot per player along the bottom (posed render in the player colour, name, costume,
      sliding in on pick), the roster grid above.
- [ ] 3. Title screen: logo over the stage scene, a posing fighter, "press any button".
- [ ] 4. Results screen: winner's victory pose, banner, a few stats (KOs, damage dealt), loser dimmed (mend `--demo=results` first).
- [ ] 5. One written UI design system: type scale, palette, panel rules, motion timings, so every screen stays consistent.

## Skies (owner, 2026-10-15: the canyon's sun and clouds still look cheap)
- [ ] A painted sunset for the canyon: the sun as a soft painted disc with a proper glow falloff, and clouds as painted shapes with a lit
      underside and soft edges (a cloud texture or card set, from Material Maker or painted SVG cards), replacing the squashed spheres
      in `_painted_sun` (`stage_art.gd`) and the flat `_clouds`. Then the same cloud set for the beach and meadow skies, so every stage's
      sky shares one painted style.

## Tooling and rendering pipeline review (owner, 2026-10-15: evaluate only; nothing downloaded or built yet)
Current state (checked in code): Filmic tonemap, glow on, SSAO off, depth of field only behind the results screen, no LightmapGI or
VoxelGI or SDFGI (the stage's soft light is our own 2D bake, `stage_light.gd`), a custom post pass (`post.gdshader`) for ink lines,
haze and grade. The sim is our own deterministic Rust (rollback), so engine physics never drives gameplay.
Content tools:
- [ ] **ArmorPaint** (paint textures straight onto models, open source): a strong fit for hand-painted props and the base body's skin
      and clothes once the Blender models are final. Pairs with Material Maker (tileable surfaces). Evaluate on one prop.
- [ ] **Poly Haven / ambientCG** (free CC0 textures, HDRIs, models): photoreal, so not for direct use in a cel style; useful as height
      and pattern sources fed through Material Maker and repainted, and Poly Haven HDRIs for lighting reference only.
- [ ] **Animation**: Mixamo (free, needs an Adobe account the owner would create) gives humanoid mocap that does not fit our chunky
      proportions or a fighter's snappy keys; Cascadeur has a free tier (physics-assisted posing, good for exactly our key-pose
      overhaul). Recommendation: Cascadeur for authoring key poses, exported to Blender/Godot. Godot's AnimationTree (blend spaces,
      state machine) is worth adopting for locomotion blends in the overhaul; root motion is NOT (the sim owns movement, rollback).
- [ ] **VFX**: GPU particles plus our painted FX shaders already fit the style; real-time volumetrics (volumetric fog) are costly and
      fight the cel look; use only as a soft light-shaft card if anywhere.
Engine rendering:
- [ ] **ACES tonemap**: test against Filmic per stage (ACES deepens contrast and saturation, which round 19's Wind Waker notes want;
      it may shift the fighters' colours, so check the look bench).
- [ ] **SSAO**: off on purpose (a cel style reads flat-shaded); try a very light SSAO only on scenery contact (low intensity), A/B.
- [ ] **SDFGI**: real-time GI for large dynamic worlds; heavy and soft, it would fight the cel ramp. Not recommended.
- [ ] **LightmapGI**: not used; stages are built in code at runtime, so baking needs a pipeline change; our 2D bake covers it. Skip.
- [ ] **Depth of field**: a subtle far blur already exists for the Best quality tier; keep, maybe stronger on menus and results.
- [ ] **Motion blur**: hurts readability in a fighter; skip (the competitors do not use it in play). Short smear frames on fast moves
      fit the style better.
- [ ] **Bloom**: in use, tuned; keep.
- [ ] **Jolt Physics**: only matters for cosmetic physics (cloth, debris, props); gameplay is our sim. Low priority.
- [ ] **Phantom Camera** (Godot addon): smooth framing/transitions; useful for menus, results and cinematic KO shots, not match
      framing (owner's camera rules, and the camera follows sim data). Evaluate for the UI pass's transitions.

## Tools
- [ ] Material Maker 1.7 is at `C:/Users/jared/Tools/MaterialMaker/app/material_maker_1_7_windows/material_maker.exe` (checksum
      verified). Batch export (`--export-material --target ... -o <dir> <file.ptex>`) ran but wrote nothing ("Cannot open
      user://export_targets"). Still nothing after a first windowed launch, with the exact target name (`Godot/Godot 4 Standard`, from
      nodes/material.mmg) and directory or file-prefix outputs: batch export looks broken in 1.7 on Windows. Next: export from the app
      (File > Export) by hand, or check its issue tracker / an older release. First use: painted bark and soil textures to replace the
      hand-written SVG tiles (`tools/art/make_tiles.py`).
- [ ] godot-ai MCP: configured at 127.0.0.1:8000 but nothing on this machine provides it; reinstall from its source if wanted
      (`--pick=x,y` in main.gd covers finding what draws a pixel).

- [ ] **Re-instruct the owner on the Material Maker export first thing Saturday** (open each example, File > Export > Godot/Godot 4
      Standard, save to `art/materials/mm_exports/<name>/`). Material Maker exports, by hand (owner, 2026-10-15): the owner exports `dry_earth`, `rock`, `stylized_wall` and `wood`
      (examples folder) with File > Export > Godot/Godot 4 Standard into `art/materials/mm_exports/<name>/`. Then: review them in game
      against the style; wire a PNG-tile path into `Materials._make` (it only reads the SVG tiles now), tinting through the albedo's
      value so stages keep their own colours; start with soil (dry_earth) and bark (wood/rock), compared before/after per stage. Also a palm-trunk texture (horizontal
      ring scars, no vertical streaks; the palms are plain since round 37) and tree bark the owner calls realistic, not streaky.

## Art review 6 leftovers
- [ ] Combat effects second pass: brief strong-hit accent, landing and dash dust in the same painted family, per-weapon spark colours.
- [ ] Camera and off-screen feedback checks (no camera behaviour change without the owner).
- [ ] Performance and regression pass: every stage, four fighters, supported resolutions.
- [ ] Run mode shadow crawl on camera pans, if still present.

## Beta gaps against the competition (Smash Ultimate, Rivals of Aether II, PEAK), 2026-10-15
Already in place: knock-out blasts, the final hit's slow-motion close-up, results screen with victory poses, damage that heats in
colour, music, alternate looks. What the competitors have that a beta of ours still lacks, art and presentation only:
- [ ] **Readable roster at a glance**, within the one body (owner, 2026-10-15: one shared model; variety comes from the character
      creator's customisation, never body variants). Make the creator's options (headwear, outfits, colours, accessories) bold enough
      that two fighters read apart in silhouette and colour at play distance.
- [ ] **Match entrances.** Each fighter arrives at the start (a drop-in, a dash in, a pose) rather than appearing in place.
- [ ] **Weapon art for the queued classes** (Dagger, Nodachi) and per-weapon hit sparks.
- [ ] **Living stages.** The competitors' backgrounds move (crowds, traffic, weather, wildlife); ours have a little (windmill, birds,
      boats). One or two signature moving things per stage, kept out of the fighting plane.
- [ ] **Off-screen fighter indicator** (the magnifier bubble at the screen edge): verify ours reads at a glance and stays clear of the HUD.
- [ ] **Model quality** (the PEAK bar): the body remodel and the pose overhaul above are the biggest single gap.
