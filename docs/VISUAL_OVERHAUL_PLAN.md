# Claude Code Plan: Visual Fidelity Overhaul for the Godot Platform Fighter

## Objective

Upgrade the game's visual fidelity toward the polish, readability, and presentation quality of **the reference game** (the platform fighter the project measures itself against), while preserving the existing **cel-shaded, PEAK-inspired art direction**.

This is a rendering, materials, environment, effects, and presentation overhaul—not a wholesale art-style replacement. Keep the current game architecture and assets wherever practical. Prefer incremental, measurable improvements over large rewrites.

## Core Principles

1. **Preserve the visual identity.** Keep the stylized proportions, outlines, saturated colors, and nighttime rooftop atmosphere.
2. **Prioritize gameplay readability.** Fighters, attacks, platform edges, damage feedback, and HUD information must remain easy to read during fast combat.
3. **Improve cohesion, not detail for its own sake.** Use consistent lighting, materials, color grading, and visual hierarchy across the frame.
4. **Reuse shared systems.** The game uses a common base character model with interchangeable movesets. Character rendering improvements must benefit every moveset without requiring duplicated assets.
5. **Protect gameplay behavior.** Do not alter combat logic, input, collision, platform positions, character physics, or moveset behavior as part of visual changes unless explicitly required and approved.
6. **Work incrementally.** Inspect the project first, identify existing systems, make focused changes, and validate each phase before proceeding.
7. **Respect performance.** Profile the current baseline and avoid expensive effects unless they provide a clear visual benefit.

---

## Phase 0 — Project Audit and Baseline Capture

### Tasks

- Inspect the Godot project structure, current renderer, active environment settings, camera setup, materials, shaders, lights, shadows, VFX, and HUD implementation.
- Identify existing toon/cel-shading and outline systems before proposing replacements.
- Locate the main gameplay scene and the rooftop stage assets.
- Identify how the shared character model, skins/material variants, and interchangeable movesets are assembled.
- Determine the project's Godot version and renderer (Forward+, Mobile, or Compatibility).
- Record current performance at the target resolution and with the expected number of fighters and effects.
- Capture a baseline screenshot from a fixed camera position with both fighters visible.
- Record any existing visual settings and relevant performance metrics so results can be compared.

### Constraints

- Do not rewrite the renderer or replace working shaders before understanding their purpose.
- Do not introduce new plugins or dependencies without a clear need.
- Do not modify gameplay code during the audit.

### Deliverables

- A short audit report listing relevant scenes, scripts, shaders, materials, and settings.
- A prioritized implementation plan that maps recommendations to actual project files.
- A baseline screenshot and performance measurement, if the project can be run.

### Acceptance Criteria

- The existing visual pipeline and key dependencies are understood.
- All planned changes identify the systems/files they affect.
- Baseline visuals and performance are recorded, or any blocker is documented.

---

## Phase 1 — Cohesive Lighting, Shading, and Post-Processing

**Priority: Very high.** Establish the visual foundation before polishing individual assets.

### Tasks

#### 1.1 World and color management

- Review `WorldEnvironment` settings and establish a consistent environment setup.
- Tune ambient/background contribution so shadowed surfaces retain stylized color and detail without appearing washed out.
- Test available tonemapping options in the project's Godot version and renderer. Compare results using the existing purple nighttime palette; do not assume one mode is universally best.
- Apply restrained color grading to unify the rooftop, characters, and skyline while keeping player colors, attack effects, and HUD elements distinct.
- Keep emissive elements controlled. Avoid globally increasing exposure simply to make lights look brighter.

#### 1.2 Primary lighting

- Establish a clear key-light direction that gives characters and stage geometry readable form.
- Use ambient/fill illumination to retain detail in shadowed areas.
- Add subtle silhouette/rim separation where appropriate, without creating an artificial glowing outline around every object.
- Use local lights selectively for prominent fixtures and environmental accents. Avoid adding a large number of dynamic lights without profiling.
- Tune light and shadow settings for the actual camera distance and target resolution.

#### 1.3 Shadows and contact

- Improve the visual grounding of fighters, crates, the water tower, and other stage props with consistent contact shadows.
- Tune shadow softness and resolution to avoid jagged, flickering, or excessively dark shadows.
- Check for shadow acne, detached shadows, and halos around characters.
- Use baked, fake, or shader-based contact shading where it offers a better performance/quality tradeoff than additional real-time lights.

#### 1.4 Post-processing

- Keep glow/bloom subtle and reserve it for bright emissive fixtures and selected combat effects.
- Compare available anti-aliasing options in the current renderer. Prioritize stable, clean silhouettes during movement.
- Avoid heavy depth of field, motion blur, film grain, chromatic aberration, or other cinematic effects that reduce fighting-game readability.
- Test at the target resolution rather than judging only from editor previews.

### Acceptance Criteria

- Characters, props, and platforms appear to belong to the same lighting environment.
- The scene retains the intended cel-shaded look and saturated palette.
- Silhouettes remain crisp and readable during movement.
- Lighting improvements do not produce excessive glow, crushed shadows, or washed-out colors.
- Performance remains within the agreed budget.

---

## Phase 2 — Shared Character Rendering and Materials

**Priority: Very high.** Implement improvements once in the shared character pipeline so all movesets benefit.

### Tasks

#### 2.1 Audit the current character material pipeline

- Determine whether the existing look is produced by `StandardMaterial3D`, `ShaderMaterial`, custom toon shading, inverted-hull outlines, post-process outlines, or a combination.
- Preserve working systems when possible. Improve them incrementally rather than layering a second outline system on top.
- Verify how player color variants and cosmetic/material variants are assigned.

#### 2.2 Improve form and material response

- Improve shading gradients and highlights so rounded forms read clearly without abandoning the stylized look.
- Give different material categories distinct responses where supported by the current assets: skin, fabric, rubber, plastic, metal, and accessories.
- Tune roughness and specular response carefully. Avoid making all surfaces glossy or photorealistic.
- Add restrained material variation only where it remains visible at gameplay scale.
- Improve facial and small-feature readability if the existing model and UVs support it; avoid detail that disappears at normal camera distance.

#### 2.3 Improve outlines and separation

- Review outline thickness at the target resolution. Avoid outlines that appear too thick at close range or disappear at normal gameplay distance.
- If practical within the current shader architecture, make outlines responsive to viewing distance, material, or surface orientation.
- Use subtle lighting-based edge separation in addition to outlines rather than making every edge equally dark.
- Ensure player one and player two remain visually distinguishable against all stage layers and effects.

#### 2.4 Validate across animations and movesets

- Check idle, run, jump, fall, attack, hurt, launch, landing, and recovery poses.
- Ensure materials and outlines remain stable during deformation and fast movement.
- Verify the shared rendering changes apply to every moveset and do not require duplicate character scenes or per-moveset material copies.

### Acceptance Criteria

- Character shapes read more clearly through shading and controlled highlights.
- Material differences are perceptible but still consistent with the game's stylized art direction.
- Outlines remain stable and attractive at actual gameplay resolution.
- All existing movesets and animation states render correctly.
- No gameplay behavior is changed.

---

## Phase 3 — Rooftop, Platforms, and Prop Materials

**Priority: High.** Improve the stage's sense of physical depth while preserving its current layout and collision behavior.

### Tasks

#### 3.1 Main rooftop surface

- Improve the main floor's material response so it does not read as one uniformly colored plane.
- Add restrained tile seams, concrete variation, subtle roughness variation, or other surface detail appropriate to the current art style.
- Avoid high-frequency texture noise that becomes distracting or shimmers during camera movement.
- Improve the front edge and underside so the platform has clear thickness and readable construction.
- Preserve all existing collision geometry and gameplay dimensions.

#### 3.2 Floating platforms

- Distinguish top, side, trim, and underside materials.
- Add modest bevels or edge highlights where geometry permits and where they improve silhouette readability.
- Refine the red surface with controlled shading and highlights instead of uniform brightness.
- Ensure visible platform edges remain aligned with collision shapes.

#### 3.3 Props

- Improve material separation among the water tower, crates, vent, lamp post, and other rooftop objects.
- Add a restrained amount of detail—such as seams, supports, fasteners, or edge wear—only where it helps establish scale and material.
- Use contact shadows to ground props.
- Keep decorative details from resembling platforms or obscuring fighters.

### Acceptance Criteria

- The rooftop and floating platforms have clearer volume and material separation.
- Platform boundaries remain obvious during combat.
- Existing stage layout, collision shapes, and gameplay dimensions are unchanged.
- Added detail does not create distracting visual noise.

---

## Phase 4 — Background Depth and Environmental Life

**Priority: High.** Keep the stylized city, but make it feel deeper, less repetitive, and more intentional.

### Tasks

#### 4.1 Layer the skyline

- Organize the environment into foreground/gameplay, midground buildings, and distant skyline layers.
- Use simplified silhouettes and lower contrast for distant buildings.
- Apply subtle atmospheric separation through color and contrast rather than heavy fog.
- Add parallax only if it remains stable and does not distract from combat.

#### 4.2 Improve building variation

- Reduce the impression of repeated, identical window grids.
- Vary window sizes, spacing, density, brightness, and color in a controlled way.
- Add a few distinct rooftop silhouettes, antennas, water tanks, signs, or architectural features.
- Keep background geometry and patterns visually subordinate to the fighters.

#### 4.3 Emissive elements and animation

- Use emissive materials for selected windows, signs, and lamps.
- Keep bloom limited; emissive materials should not all produce visible halos.
- Add subtle animation such as occasional window flicker, sign animation, or slow distant movement only where inexpensive and appropriate.
- Avoid numerous real-time lights attached to individual windows; use emissive materials or shader variation instead.

### Acceptance Criteria

- Foreground, midground, and distant buildings have visibly different levels of contrast and detail.
- Window patterns feel varied rather than mechanically repeated.
- Background animation is subtle and does not distract from the fight.
- Environment additions do not meaningfully impair performance.

---

## Phase 5 — Combat Visual Effects and Feedback

**Priority: High after the rendering foundation.** Improve the clarity and impact of actions without changing the underlying combat mechanics.

### Tasks

- Audit the current effects system and reuse it where practical.
- Improve attack trails, hit sparks, landing dust, jump/fall effects, and other existing combat effects.
- Establish a consistent shape language, color palette, duration, and intensity for weak, medium, and strong impacts.
- Make effects directional and brief enough to communicate attack direction and timing.
- Add small, controlled character hit flashes where appropriate.
- Evaluate hit-stop and screen shake only if these are already supported or can be introduced as strictly visual feedback without changing combat timing or physics.
- Keep particles and flashes from obscuring fighters, platform edges, or important hitbox/collision information.
- Ensure effects clean themselves up and do not accumulate during long matches.
- Profile scenes with several simultaneous effects.

### Acceptance Criteria

- Weak and strong attacks are visually distinguishable.
- Impact direction and timing are easy to read.
- Effects are visually consistent with the cel-shaded world.
- Existing damage, knockback, hitbox, and timing behavior is unchanged.
- Effects do not obscure gameplay or create resource leaks.

---

## Phase 6 — HUD and Screen Presentation

**Priority: Medium-high.** Preserve the existing information hierarchy and improve consistency.

### Tasks

- Keep damage percentage, stock count, player identity, and match timer immediately legible.
- Standardize typography, weights, spacing, margins, and alignment across HUD elements.
- Refine panel gradients, borders, bevels, and highlights without adding unnecessary decoration.
- Improve portrait framing and consistency if portraits are generated from character models.
- Add brief, restrained damage-number animation or color feedback when damage changes.
- Keep the HUD rendered independently from world lighting and world post-processing so it stays crisp.
- Check safe margins and clipping at supported aspect ratios and resolutions.
- Ensure accessibility and player-color distinction remain intact.

### Acceptance Criteria

- The HUD has consistent typography, spacing, and visual materials.
- Critical information remains readable at the target resolution.
- HUD elements do not clip at supported aspect ratios.
- The interface remains visually distinct from the 3D scene.

---

## Phase 7 — Performance, Regression Testing, and Final Polish

### Tasks

- Compare the updated scene against the baseline screenshot using the same camera, resolution, scene state, and lighting conditions.
- Profile CPU and GPU time, frame rate, memory use, shadow cost, transparent overdraw, and particle load where available.
- Test the expected maximum number of fighters and simultaneous effects.
- Check visual stability during fast camera movement, jumping, launches, and screen-edge interactions.
- Test all movesets and animation states for rendering regressions.
- Check for shader compilation hitches and effects that persist after a match ends.
- Provide scalable quality options if needed, such as shadow quality, effects density, and background detail.
- Document any compromises made for performance.

### Acceptance Criteria

- The updated scene meets the project's target frame rate and performance budget.
- No combat, input, physics, collision, or moveset regressions are introduced.
- The game remains readable during busy combat.
- Before/after comparisons show improvements in lighting, material definition, stage depth, and overall cohesion.
- All known limitations are documented.

---

## Implementation Rules for Claude Code

1. **Inspect before editing.** Find the relevant scenes, scripts, materials, shaders, and project settings before making changes.
2. **Map each change to actual project files.** Do not assume filenames, node names, renderer settings, or shader architecture.
3. **Make a small change set per phase.** Avoid broad, unrelated rewrites.
4. **Preserve current gameplay behavior.** Do not alter movesets, damage, knockback, hitboxes, physics, inputs, collision, platform dimensions, or match flow as part of this task.
5. **Reuse existing resources.** Avoid duplicate materials, redundant shaders, unnecessary plugins, and parallel systems that solve the same problem.
6. **Use project-compatible features.** Verify API and shader syntax against the project's actual Godot version and renderer.
7. **Use measurable before/after comparisons.** Capture screenshots under the same conditions and note performance changes.
8. **Validate each phase before proceeding.** Run the project or relevant tests when possible and report blockers clearly.
9. **Do not fabricate test results.** Distinguish verified behavior from code inspection or assumptions.
10. **Keep changes reversible.** Prefer focused edits and preserve a clear summary of modified files.

## Recommended Execution Order

1. Phase 0 — Project audit and baseline.
2. Phase 1 — Lighting, shading, and post-processing.
3. Phase 2 — Shared character rendering and materials.
4. Phase 3 — Rooftop, platforms, and props.
5. Phase 4 — Background depth.
6. Phase 5 — Combat VFX.
7. Phase 6 — HUD.
8. Phase 7 — Performance and regression validation.

Complete and validate each phase before moving to the next. If the project structure or current rendering pipeline makes a proposed change inappropriate, explain the issue and propose a compatible alternative instead of forcing the original implementation.

## Definition of Done

The overhaul is complete when the game retains its distinctive PEAK-inspired cel-shaded identity while presenting more polished character shading, cohesive lighting, dimensional stage materials, a layered city background, readable combat effects, a consistent HUD, and stable performance—with existing gameplay behavior preserved.
