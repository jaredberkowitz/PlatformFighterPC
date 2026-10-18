# Gauntlet loop: the art judge's brief

A separate agent judges the stages each round, so the one making the art is not the one grading it. Same brief every round, so scores
compare across rounds.

**Benchmarks** (docs/art/benchmarks/):
- `environment_godot_reference.png`: another small team's Godot game: believable, consistent, soft-lit environments (the owner's
  environment benchmark, 2026-10-18).
- `papercraft_reference.webp`: the papercraft adventure (the style benchmark): soft high-key light, pale large surfaces with saturated
  accents, no ink on scenery, crisp lit edges.
- The big crossover fighter's stages (from memory): a clear readable play area framed by a calm, layered background.

**What is judged:** the round's match-camera captures of each stage, `docs/gauntlet/round<N>/stage<k>.png` (0 meadow, 1 canyon,
2 beach, 3 city, 4 grove). The fighters, UI and effects are not judged here.

**Scores, 1 to 5 against the benchmarks** (5 = on par), per stage:
1. Believability: do surfaces read as real materials (grass, dirt, stone, wood) with natural variation, or as flat fills and primitives?
2. Cohesion: one consistent style across every element, no piece that looks like it came from another game.
3. Lighting: soft, unified light; believable shadow and contact; colour of light and shade.
4. Depth and framing: foreground, play area, mid-ground and distance read as layers; the background frames the fight.
5. Ground detail: the scale cues the benchmark has (grass clumps, flowers, pebbles, worn paths) at a calm density.
6. Clarity: the play area and fighters stay the clearest thing; nothing distracts.

**Output:** a table of scores per stage; the two biggest gaps across the game (not per stage), each with what is wrong, where (stage and
element), and a concrete fix. The owner's rules bind the fixes: refine and improve what exists, add no new elements; prefer modelled
(Blender) assets over code-built primitives; keep the play area's collision and the camera as they are.

**Log:** each round's verdict goes in `docs/gauntlet/LOG.md` with the round's commit, so progress is measured.
