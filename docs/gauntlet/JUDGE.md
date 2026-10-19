# Gauntlet loop: the art judge's brief

A separate agent judges the stages each round, so the one making the art is not the one grading it. Same brief every round, so scores
compare across rounds.

**Benchmarks** (docs/art/benchmarks/):
- `environment_godot_reference.png`: another small team's Godot game: believable, consistent, soft-lit environments (the owner's
  environment benchmark, 2026-10-18).
- `papercraft_reference.webp`: the papercraft adventure (the style benchmark): soft high-key light, pale large surfaces with saturated
  accents, no ink on scenery, crisp lit edges.
- The big crossover fighter's stages (from memory): a clear readable play area framed by a calm, layered background.
- Per stage: `STAGE_REFERENCES.md` (the owner's harbour and city references, described, with the images when added). Judge the beach
  against the harbour reference and the city against the city reference first, then the general benchmarks.

**The style sheet comes first (the owner, 2026-10-19):** `docs/art/STYLE_SHEET.md` is the rule set every element is checked against
(layers and their value bands, colour, light, shape, surface, detail), and the mood board shows what already meets it. From round 33 the
judge's main job is consistency: find what breaks the sheet and say whether to fix, unify or remove it. A stage cannot score 4 in
Cohesion while any element in view breaks the sheet.

**What is judged:** the round's match-camera captures of each stage, `docs/gauntlet/round<N>/stage<k>.png` (0 meadow, 1 canyon,
2 beach, 3 city, 4 grove). The fighters, UI and effects are not judged here. **The canyon is scored but not ranked as a gap:** the
owner keeps it as it is for now.

**Scores, 1 to 5 against the benchmarks** (5 = on par), per stage. **The bar for a 5 (the owner, 2026-10-18):** only when a capture is
hard to tell apart from the reference images in quality, fidelity and readability, at a similar level. One consistent, unified style;
anything amateur-ish, cheap-looking or "Flash game"-like caps the score. Do not round up.
1. Believability: do surfaces read as real materials (grass, dirt, stone, wood) with natural variation, or as flat fills and primitives?
2. Cohesion: one consistent style across every element, no piece that looks like it came from another game.
3. Lighting: soft, unified light; believable shadow and contact; colour of light and shade.
4. Depth and framing: foreground, play area, mid-ground and distance read as layers; the background frames the fight.
5. Ground detail: the scale cues the benchmark has (grass clumps, flowers, pebbles, worn paths) at a calm density.
6. Clarity: the play area and fighters stay the clearest thing; nothing distracts.

**Output (from round 33):**
1. The score table per stage (same six categories, same bar).
2. **Outliers:** every element in the judged stage(s) that breaks the style sheet, each with the rule it breaks (cite the section), and
   one of: *fix* (how), *unify* (with what), or *remove*. Ranked by how much it hurts the whole frame. Removing is as good an answer as
   fixing.
3. **Measured checks** where the captures allow: each layer's value against its band in section 1 (from a greyscale read), the fighters'
   separation from the surface under them.
4. **Contact sheet:** with all five stages side by side, name any stage or element that reads as from a different game.
5. The **three actions** that would move the judged stage(s) most toward the sheet, in order. No new elements unless the sheet asks for
   one. The owner's rules bind the fixes: refine and improve what exists, add no new elements; prefer modelled
(Blender) assets over code-built primitives; keep the play area's collision and the camera as they are.

**Log:** each round's verdict goes in `docs/gauntlet/LOG.md` with the round's commit, so progress is measured.
