# Character model and cel shading: audit and change report (2026-10-14)

The owner's character polish plan: bring the shared fighter closer to the concept sheet (`art/concept/base_body_turnaround.jpg`), a
pastel, soft, rounded cartoon with clean cel shading, by iterating on what exists. Done before any weapon or animation work, as asked.
Gameplay is untouched: no sim, hitbox or moveset change.

## The pipeline (audit)

- **Body**: one skinned mesh, built from numbers in `art/blender/make_base_body.py` (head, trunk rings, limb chains, feet), joined by a
  voxel remesh and re-laid as even quads (pass 52), measured by `fit_body.py` (`art/models/base_body/joints.json`) and rigged by
  `make_rigged_blob.py`, which adds the glove fists (or bare mittens) and the face shell and writes `godot/models/base_rig.glb` and
  `base_rig.json` (head, shoulder and wrist the game fits hats, glasses and the weapon arm to). Every fighter and every moveset shares it.
- **Materials**: `shaders/skin.gdshader` (clothes painted on by region), `limb.gdshader`, `face.gdshader` (the 2D face on its shell),
  `toon.gdshader` for worn pieces, all through the shared cel light `toon_light.gdshaderinc`. Body colours: `Loadout.PALETTE`.
- **Line**: an inverted-hull outline per mesh (`outline.gdshader`), tinted with the fighter's own colour darkened (`fighter_view.gd`,
  `OUTLINE_TINT*`, `FIGHTER_OUTLINE_WEIGHT`); the post pass adds no second line on it (pass 53).
- **Attachments**: weapons follow the wrist bone and are placed in `fighter_view.gd`; hats and glasses fit to `base_rig.json`.
- **Review tools**: `tests/turnaround_shot.gd` (new: front, three-quarter, side, back, as the sheet; `--bare` for the body alone,
  `--neutral` for shape only), `tests/lineup_shot.gd`, `tests/anim_sheet.gd`, `tests/look_bench.gd`, `--demo=stage`.

## Ranked differences from the concept

| # | Difference | Cause | Fix by |
|---|---|---|---|
| 1 | **Glossy, toy-like surface**: a wide white rim and a diagonal streak of light across the head and body read as plastic. | Shader: the rim's width (0.55 of the edge) and the white edge light, on a fighter. | Shader (done) |
| 2 | **Shading too hard and too dark** for the concept's gentle gradients. | Shader: pass 52's step (edge 0.02, shadow 0.8) was tuned against the old mushy look. | Shader (done) |
| 3 | **Colours candy-bright** (orange about 95% saturated against the concept's 70%). | Material: `Loadout.PALETTE`. | Material (done) |
| 4 | **Hands small and arms thin**: the concept's arms are soft tubes with broad hands; ours read as sticks with knobs at play size. | Geometry: `ARM` radii, the fist and mitten size. | Geometry (done) |
| 5 | **Heavy outline** against the concept's thin warm-brown line. | Line: weight 0.82, tint darkened 64%. | Line (done) |

Checked and left as they are:
- **Head-to-body ratio and torso**: the neutral turnaround matches the sheet's (head about 0.43 of the height on both). No change.
- **Feet**: broad and flat as on the sheet.
- **The face**: drawn on a shell that follows the head's curve. It reads at play distance (the readability bench).
- **The concept's open five-fingered hand** stays a mitten. It is the sheet's detail call-out, and at play size the fist or mitten is
  the readable shape (the plan says to avoid realistic fingers).

## Changes (character pass 1)

1. **Cream rim** (`toon_light.gdshaderinc`): on fighters the rim is a thin line at the very edge (`PF_FIGHTER_RIM_WIDTH` 0.3, from
   0.55), and the edge light takes the fighter's own colour (`PF_FIGHTER_CREAM`). The highlight reads creamy and warm, not white glare.
2. **Gentler two-tone**: the step's edge is wider (`PF_FIGHTER_EDGE` 0.07) and the shadow lighter (`PF_FIGHTER_SHADE` 0.88). Still two
   clear shapes, but with the concept's soft turn rather than a cut.
3. **Pastel palette** (`loadout.gd`): the bright body colours carry a little more cream (same hues). Neutrals are unchanged.
4. **Hands and arms** (`make_rigged_blob.py` `HAND_SCALE` 1.12, grown round the wrist so grips stay put; `make_base_body.py` `ARM`
   radii about 10% fuller). Rebuilt, refitted (the joints moved under 2 cm) and rigged again.
5. **Line** (`fighter_view.gd`): weight 0.76 (from 0.82), tint darkened 56% (from 64%): a thinner, warmer brown line.

Each was compared on its own (turnarounds in the same camera, close-ups on the meadow and the night city, play distance).

- **Images**: `docs/screenshots/char1_turnaround.png` (before top, after bottom), `char1_shape_neutral.png` (shape only: hands and
  arms), `char1_lineup.png`.
- **Checks run**:
  - fighter view, animation library, art pipeline, loadout and match flow tests pass;
  - the look bench holds at 0.89 (needs 0.85);
  - animation sheets for run, jump, the sword's forward smash, the greatsword's forward smash, the Wanderer's forward tilt (sword and
    shield), the hit reaction and the Hauler's jab: grips stay in the fists, the shield on the forearm, nothing clips.

## Character pass 2: faces and accessories (phase 6)

Every hat, pair of glasses and neckwear checked on the lineup (`tests/lineup_shot.gd`) and the turnaround (`--hat=N`, `--neck=N`,
`--glasses=N` added to `tests/turnaround_shot.gd`). Faces follow the head and read; glasses and hats sit right.

- **The hood (the Rogue's) read as a diver's helmet**: it was a closed ball over the head, and the face, drawn on top, showed as a disc
  in a dark ring. Rebuilt:
  - an open shell (`_open_shell`) whose opening is shaped like a hood's (`_hood_edge`): snug over the brow, wider at the cheeks, down
    past the chin;
  - a rolled lip in a lighter shade that follows that edge exactly (`_shell_lip`, built from the same edge, so it lines up at any scale);
  - a flared collar of the same cloth over the shoulders, dipping in front;
  - the point behind kept.

  Seen from both sides like the clothes. Tried and dropped on the way: a round opening with a cream rim (a porthole), and a torus
  cowl (an inflatable ring).
- **Images**: `docs/screenshots/char2_hood.png` (the gear lineup before, top, and after).
- **Checks run**: fighter view and loadout tests; the gear lineups; the turnaround with the hood.

## Still open

- Elbow and knee bends (deformation): to look at in the motion pass, which belongs with the animation work at the end.
- The fist's thumb shows a small facet close up.
