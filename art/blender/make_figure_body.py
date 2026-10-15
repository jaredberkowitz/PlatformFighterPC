"""The fighters' base body from the owner's generated figure (art review 6, 2026-10-15: the owner chose it as the new base model).

`art/generated/base_figure/figure_sheet.glb` holds three views of one character side by side (a generated model sheet, the face painted
in its texture). The middle figure is cut out by its band along the row, turned to face -Y, stood on the ground at the game's 2.2 height
and passed through make_base_body.py's own finish(): one voxel skin (the sheet is hundreds of loose fragments), QuadriFlow, the right
half mirrored, a volume-keeping smooth. The painted face is dropped: the game draws its own faces with expressions. Then, as before:
fit_body.py, make_rigged_blob.py.

Run:
    blender --background --python art/blender/make_figure_body.py -- art/generated/base_figure/figure_sheet.glb art/models/base_body/base_body.glb -0.27 0.215
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

argv = sys.argv[sys.argv.index("--") + 1:]
src, out, ymin, ymax = argv[0], argv[1], float(argv[2]), float(argv[3])
REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
code = open(os.path.join(REPO, "art/blender/make_base_body.py"), encoding="utf-8").read()
code = code.replace("\nmain()\n", "\n")
g = {"__name__": "base_body", "__file__": os.path.join(REPO, "art/blender/make_base_body.py")}
exec(compile(code, "make_base_body.py", "exec"), g)

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=src)
o = [x for x in bpy.context.scene.objects if x.type == "MESH"][0]
bpy.context.view_layer.objects.active = o
o.select_set(True)
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
bm = bmesh.new()
bm.from_mesh(o.data)
# The middle figure only.
bmesh.ops.delete(bm, geom=[v for v in bm.verts if not (ymin < v.co.y < ymax)], context="VERTS")
# It faces +x: turn it to face -y (Blender's front here), stand it on the ground, 2.2 tall, centred.
bmesh.ops.rotate(bm, verts=bm.verts, cent=(0, 0, 0), matrix=Matrix.Rotation(-math.pi / 2, 3, "Z"))
lo = Vector((min(v.co.x for v in bm.verts), min(v.co.y for v in bm.verts), min(v.co.z for v in bm.verts)))
hi = Vector((max(v.co.x for v in bm.verts), max(v.co.y for v in bm.verts), max(v.co.z for v in bm.verts)))
k = 2.2 / (hi.z - lo.z)
for v in bm.verts:
    v.co = Vector(((v.co.x - (lo.x + hi.x) / 2) * k, (v.co.y - (lo.y + hi.y) / 2) * k, (v.co.z - lo.z) * k))
bm.to_mesh(o.data)
bm.free()
o.data.materials.clear()
o.name = "BaseBody"
o.data.name = "BaseBody"
tris = g["finish"](o)
print("EXT BODY", tris)
bpy.ops.object.select_all(action="DESELECT")
o.select_set(True)
bpy.ops.export_scene.gltf(filepath=os.path.abspath(out), use_selection=True, export_format="GLB", export_yup=True)
