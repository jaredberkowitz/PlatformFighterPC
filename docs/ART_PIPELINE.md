# Art pipeline (Blender to Godot)

Blender 5.2 is installed (`winget install BlenderFoundation.Blender`; `C:\Program Files\Blender Foundation\Blender 5.2\blender.exe`).
Models are **made by script** so they are reproducible, reviewable in git and original. Style target: `docs/ART_DIRECTION.md`.

## Blob parts

`art/blender/make_blob_parts.py` builds the fighter's body parts (Body as a pear, Head, FootL/FootR with a flat sole and toe cap, HandL/HandR
mittens with a thumb) and exports `godot/models/blob_parts.glb`:

```
"C:\Program Files\Blender Foundation\Blender 5.2\blender.exe" --background --python art/blender/make_blob_parts.py
```

Each part is modelled at the size and centre the game used for its old sphere, so `FighterView` (`godot/scripts/fighter_view.gd`,
`blob_parts()`) swaps the mesh in place and only chooses the colour. The glTF is read at run time (no editor import needed); if the file is
missing or damaged the fighter falls back to plain spheres. The face, hat, glasses, neckwear, sash and weapon are still built in code.

## Rules

* Cosmetic only: gameplay hurtboxes come from content, never from a mesh (`CLAUDE.md`).
* Original shapes only; keep the poly budget small (the whole set is about 140 KB).
* Change a shape: edit the script, rerun Blender, commit both the script and the `.glb`.

## Next steps for art

1. Move the hat, glasses, neckwear and sash into the Blender script as meshes with sockets, then drop the code-built versions.
2. A rigged blob (hips, spine, head, arms, legs) with the same part names so poses come from animation clips instead of the squash,
   lean and blade code in `fighter_view.gd`.
3. A face atlas texture and expression set for the head.
4. Stage backdrops, hit effects and a shield bubble.
