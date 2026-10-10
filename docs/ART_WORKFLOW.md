# Art workflow

How a piece of art goes from an idea to the game, so everything we add is consistent with the style (`ART_DIRECTION.md`), original, and
within budget. The tools are in place for every step; each step also has a by-hand route.

```
 1 Brief        what it is, its silhouette, its colours, its budget          (this page: style sheet, budgets)
 2 Concept      a front view (and side and back for a character)             (drawn, commissioned, or an image generator + the prompt kit)
 3 Generate     a raw 3D model                                                (tools/art/gen3d.py: Meshy or Tripo; or Blender by hand; or CC0)
 4 Clean up     scaled, within budget, flat cel colours, no baked shading    (art/blender/import_generated.py)
 5 Integrate    in the game                                                   (godot/models/props + Toonify, backdrop layers, loadout)
 6 Review       seen in the game, in every stage's light                     (demo shots, the checklist below)
```

## 1. Style sheet

Every asset follows these, whatever made it:

- **Shapes**: chunky, rounded and simple, readable as a silhouette at match distance (the camera is 19 to 85 units away; a fighter is
  2.2 tall). Big shapes first; no fine detail that turns to noise when small.
- **Colour**: flat colours, 2 to 6 per asset, fairly saturated, mid to light values. **No shading painted in**: the game paints the
  shadows (each stage turns them toward its own hue, `lighting.gd` `MOODS`), draws the rim light and the ink lines. A colour that is
  only a darker version of another is a mistake (the clean-up step merges those).
- **Outlines** come from the game (the cel material's ink hull and the post-process), never from the model or its texture.
- **Faces** are 2D drawings (`godot/art/faces/*.svg`) laid on the head. A generated character's head must be **blank**.
- **Originality**: the reference image is a style target only. Never feed another game's characters, screenshots or logos to a
  generator, never ask for a "character from X" or "in the style of X game", and keep names clear of the blocked list (`Roster.name_problem`).
  Everything we ship must be ours.

**Budgets** (enforced by the clean-up step; `KINDS` in `import_generated.py`):

| Kind | Height (world units) | Triangles | Colours | Where it goes |
| --- | --- | --- | --- | --- |
| accessory (hat, glasses) | 0.6 | 1,500 | 4 | a loadout slot (each catalog item adds at most 12 meshes, `loadout_test.gd`) |
| prop (barrel, crate, sign) | 1.2 | 3,000 | 6 | on or near the stage |
| scenery (tree, rock, building) | 6 | 2,500 | 6 | behind the stage (far things can be cheaper still) |
| fighter body | 2.2 | 8,000 | 8 | the rig (see Characters) |

## 2. Concept art

A clean **front view** on a plain background is what image-to-3D needs; a character also wants side and back views (a turnaround). It
can be drawn (rough is fine if the shapes and colours are clear), commissioned, or made with an image generator. Prompt kit for image
generators (fill in the brackets; keep the rest):

- **Character turnaround**: `character turnaround sheet, front view, side view and back view, A-pose, a round chubby blob creature whose
  big round head is merged with its body, short stubby arms with mitten hands, short legs with chunky shoes, wearing [outfit], [2 to 4
  colours], flat colours, soft cel shading, clean thick outlines, blank head with no face, plain white background, full body, centred,
  orthographic, game asset`
- **Accessory or prop**: `a single [object], chunky stylized cartoon proportions, [2 to 4 colours], flat colours, simple shapes, no
  texture detail, plain white background, three-quarter view, centred, game asset`
- **Backdrop layer** (one layer at a time: far, middle, near): `wide panoramic painted background layer of [subject], storybook
  gouache style, soft shapes, limited palette of [the stage's sky and ground colours], no characters, no text, [transparent / plain
  sky-coloured] background`

**Preparing a turnaround sheet**: `art/blender/prepare_concept.py` cuts the views out of a sheet listed in a small views file (see
`art/concept/base_body_views.json`), keeps only the figure (the grey ground, guide lines, labels and bits of neighbouring views turn
white), puts each view on a white square, and with `blank_face` also writes a version with the face painted out in the skin colour (the
game draws faces itself). Example: the base body sheet gives `art/concept/base_body_turnaround_{front,front_blank,side,side_blank,back}.png`.

```bash
"C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/prepare_concept.py -- --in art/concept/base_body_turnaround.jpg --views art/concept/base_body_views.json
```

## 3. Generate a model

**Free**: `tools/art/hf_gen.py` uses public Hugging Face demos of open models: **FLUX.1-schnell** (Apache-2.0) for a picture from a
prompt (wrapped in the prop style wording: straight-on, eye level, upright, no shadow), and for the model **TRELLIS** (`--model trellis`,
the default) or **TRELLIS.2** (`--model trellis2`), both Microsoft's and MIT-licensed, or **Stable Fast 3D** (`--model sf3d`, Stability AI
Community Licence: free under $1M a year in revenue, commercial use after a free registration). Log in once with `hf auth login` (a free
account's read token). Measured limits (October 2026): the demos share each account's free GPU time (ZeroGPU, a few minutes a day), and
TRELLIS and TRELLIS.2 each ask for 120 seconds, which is only granted with plenty left, so a free account makes about **one model a day**
(Hugging Face PRO gives 25 minutes a day, about a dozen). Stable Fast 3D fits easily but its models are lumpy blobs that lose the details;
it is not good enough for this style. Before generating, cut the picture out of its ground and shadow
(`art/blender/prepare_concept.py -- --in picture.png --cutout out.png`), or the shadow becomes a disc under the model. Pictures are kept in
`art/concept/` (props in `art/concept/props/`) as the record of where each model came from.

```bash
python tools/art/hf_gen.py text-image "a wooden barrel with iron bands" --name barrel
"C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/prepare_concept.py -- --in art/generated/barrel/concept.png --cutout art/concept/props/barrel.png
python tools/art/hf_gen.py image-3d art/concept/props/barrel.png --name barrel
python tools/art/hf_gen.py image-3d art/concept/base_body_turnaround_front_blank.png --name base_body --model trellis2
```

**Paid**: `tools/art/gen3d.py` sends a concept picture (or a text prompt) to **Meshy** or **Tripo** and downloads the raw model to
`art/generated/<name>/raw.glb` (raw models are not committed; see `.gitignore`).

1. Make an account with the service and create an API key on its account page (both need a paid plan for API keys: Meshy's docs
   say free accounts only get a key for its web playground, and Tripo's free tier has no API access).
2. Put the key in the environment, never in a file in the repository: in PowerShell, `setx MESHY_API_KEY "<key>"` (or
   `TRIPO_API_KEY`), then open a new terminal.
3. Run it. Without `--yes` it only prints the request (every request spends credits):

```bash
python tools/art/gen3d.py meshy-image concept/straw_hat_front.png --name straw_hat --polycount 1500
python tools/art/gen3d.py meshy-image concept/straw_hat_front.png --name straw_hat --polycount 1500 --yes
python tools/art/gen3d.py meshy-text "a round wooden barrel with iron bands, chunky cartoon proportions" --name barrel --refine --yes
python tools/art/gen3d.py tripo-image concept/straw_hat_front.png --name straw_hat --yes
```

Meshy follows its documented API (image-to-3d v1, text-to-3d v2: a shape preview, then `--refine` paints it). Tripo follows its v3 API,
whose result fields are only partly documented: the client takes the first `.glb` link in the finished task (and saves the whole task as
`task.json`), so check the first Tripo run.

**Other routes**: model it by hand in Blender (the fighters' bodies are built by `art/blender/make_rigged_blob.py`); use **CC0
libraries** (Kenney, Quaternius, Poly Haven), which go through the same clean-up; or the **Blender MCP** add-on, which lets Claude drive
Blender directly and reach Hyper3D Rodin generation, Poly Haven and Sketchfab from inside it (installed by the developer: a Blender
add-on plus an MCP server entry for Claude Code).

## 4. Clean up

```bash
"C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/import_generated.py -- --in art/generated/straw_hat/raw.glb --name straw_hat --kind accessory
```

It imports the model (`.glb`, `.gltf`, `.fbx`, `.obj`), joins it into one mesh, turns it (`--turn` degrees) and scales it to its kind's
height standing on the ground, welds split vertices and reduces it to the budget, and **flattens its colours**: the texture is sampled
all over, the colours are grouped by hue and strength more than by lightness (shading painted into a generated texture changes
mostly lightness), near twins are merged, and each flat colour is taken from the lighter part of its group. With a texture it keeps the
texture but repaints every texel in the nearest flat colour (`--detail texture`, 256 pixels: spots, badges and stripes stay crisp);
without one (or `--detail flat`) each face gets one plain colour. It writes `godot/models/props/<name>.glb` and a report beside it
(`<name>.json`: triangles before and after, the colours, the source). Override any budget with `--height`, `--tris`, `--colors`.

After adding files under `godot/`, run `Godot --headless --path godot --import` so Godot imports them, and commit the `.import` files.

## 5. Into the game

- **Props and scenery**: `Toonify.load_prop("barrel")` (`godot/scripts/toonify.gd`) gives the model with its plain materials swapped
  for the cel material of the same colour (texture kept), with the ink outline; place it from the theme's builder in `stage_art.gd`.
  `Toonify.apply(node)` does the same for any imported scene.
- **Painted backdrop layers**: put the pictures (PNG, with transparency where the layer ends) in `godot/art/backdrops/<theme>/` with a
  `layers.json`: `[{"image": "far_hills.png", "z": -110, "y": -8, "width": 260}, ...]` (`x` and `"cutout": true` for hard-edged
  transparency are optional). Each becomes a flat card behind the stage; the camera's perspective gives the parallax. The theme's 3D
  scenery stays unless it is removed from its builder.
- **Accessories**: the loadout catalog (`loadout.gd`) builds its pieces in code today; a cleaned accessory model hangs from the same
  socket (head or face) with `Toonify` and the fighter's object id (`FighterView._tag_object`). Keep within the mesh budget.
- **Lighting and shading**: a stage's look is its theme's mood in `lighting.gd` (`MOODS`: key and fill light, ambient, the hue shadows
  turn toward, the rim colour) plus its fog layers (`MIST` in `stage_art.gd`); the baked soft shading comes from the stage's shape by
  itself.

### Characters

Fighters are animated by poses on a shared skeleton (`make_rigged_blob.py`: the clips, the IK and the 2D face all depend on it), so a new
body is fitted to that skeleton rather than bringing its own. The base body (from `art/concept/base_body_turnaround.jpg`) was made this way:

1. **Generate** a full body, A-pose or arms down, blank head if possible (it was made with Tripo from the sheet; Tripo built the whole sheet
   as 3D, so the main figure was picked out in the next step).
2. **Clean it up** as one closed, low skin in one colour (the face and any patches go; the game draws the face and paints the colour):
   ```bash
   "C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/import_generated.py -- --in <sheet.glb> --name base_body --kind fighter --keep biggest --remesh 0.02 --out art/models/base_body/base_body.glb
   ```
   (`--keep biggest` keeps the largest figure after welding the seams; `--remesh` rebuilds the surface; fighters are kept to 3,500
   triangles: smooth shading and the ink outline carry the look.)
3. **Measure it** for the skeleton (`art/blender/fit_body.py`: points spread over the surface are sliced; the neck is the narrowest
   trunk, the legs are followed up from the feet to the crotch, the arms down from the armpits to the fingertips):
   ```bash
   "C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/fit_body.py -- --in art/models/base_body/base_body.glb --out art/models/base_body/joints.json
   ```
4. **Rig it** with every clip (`make_rigged_blob.py --body`: the arms are filled out front to back, the generated hands give way to the game's
   glove fists, a face shell is laid on the head, the skin is weighted automatically; writes `godot/models/base_rig.glb` and
   `base_rig.json`, the head, shoulder and wrist the game fits hats, glasses and the weapon arm to):
   ```bash
   "C:/Program Files/Blender Foundation/Blender 5.2/blender.exe" --background --python art/blender/make_rigged_blob.py -- --body art/models/base_body/base_body.glb --joints art/models/base_body/joints.json
   ```
5. **Try it** in the game with the `--base-body` launch option (`FighterView.use_base_body`).

Still to do for the base body: its clothes (the shirt, shorts and shoes the blob wears come from separate parts and limb bands; the base
body needs them painted on by region or made as fitted pieces), the neckwear's fit, and the licence of the free-plan model (see
`THIRD_PARTY.md`).

## 6. Review

- **See it in the game**, in every stage's light: `--demo=stage --stage=N --noui --shots=<dir>` (stages), `tests/lineup_shot.gd` and
  `tests/anim_sheet.gd` (characters), `--demo=fx_<sword|claws|maul>_<move> --noui --shots=<dir>` (effects, steady camera).
- **Checklist**: the silhouette reads at match distance; colour count and triangles within budget (the report); no baked shading left
  (a shaded side darker than the game's own shadow is baked); outlines only round the outside and real edges; it sits in each stage's
  mood (shadows turned to the stage's hue still read); nothing clips or floats; the name and the design are original.
- Tests: `godot/tests/art_pipeline_test.gd` (materials swapped to cel ones in the same colour, textures kept, absent props and layers
  are simply absent).
