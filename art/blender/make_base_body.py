"""Builds the game's base body from numbers, after the concept sheet (art/concept/base_body_turnaround.jpg): a big round head sitting
straight on a small pear-shaped trunk (no neck), soft tube arms reaching mid-thigh with mitten hands, short thick legs and wide flat oval
feet. Every part is a simple solid (a ball, a lathed trunk, limbs swept as a chain of balls); they are joined into one seamless skin by
voxel remeshing, softened where they meet, and reduced to a low-poly mesh for the cel shader.

    blender --background --python art/blender/make_base_body.py -- --out art/models/base_body/base_body.glb [--preview <png>]

Then measure it and rig it (docs/ART_WORKFLOW.md, Characters):

    blender --background --python art/blender/fit_body.py -- --in art/models/base_body/base_body.glb --out art/models/base_body/joints.json
    blender --background --python art/blender/make_rigged_blob.py -- --body art/models/base_body/base_body.glb --joints art/models/base_body/joints.json --neck-drop 0

The body stands on the ground (z 0), faces -Y (Blender's front) and is 2.2 units tall, the height of the body it replaces, so the
game's fit (hats, camera, hurtbox look) is unchanged. All of it is original and ours: no generated mesh is involved.
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

# ---- Proportions (units; read off the concept's front and side views, scaled to 2.2 tall) -------------------------------------------
HEIGHT = 2.2
HEAD_CENTRE = Vector((0.0, 0.0, 1.795))
HEAD_RADII = Vector((0.4, 0.385, 0.39))
# The trunk as rings from the top (hidden inside the head) to the crotch: (height, half-width, half-depth, forward shift of the ring's
# middle). The shoulders tuck under the head (no neck) and are slim front to back there, so the head overhangs them all round and its
# outline reads as a whole circle, as on the sheet; the belly is the widest and pushes forward, with a little seat behind.
TRUNK = [
    (1.56, 0.12, 0.1, 0.0),
    (1.48, 0.215, 0.12, 0.0),
    (1.40, 0.232, 0.135, 0.01),
    (1.27, 0.232, 0.162, 0.015),
    (1.13, 0.245, 0.19, 0.03),
    (1.00, 0.27, 0.21, 0.048),
    (0.89, 0.272, 0.205, 0.04),
    (0.80, 0.255, 0.19, 0.012),
    (0.73, 0.21, 0.15, -0.005),
    (0.67, 0.12, 0.08, 0.0),
]
# Arms: shoulder (inside the trunk), elbow, wrist, each with its radius; hanging out at about 20 degrees, the hands ending at the top of
# the thigh (the sheet's reach to mid-thigh read as too long in play). Parts must stand at least 2% of the height apart below the armpits
# and the crotch, or fit_body.py cannot tell them from the trunk.
ARM = [((0.22, 0.0, 1.40), 0.088), ((0.42, 0.012, 1.13), 0.078), ((0.535, 0.025, 0.87), 0.068)]
# A mitten hand: palm and fingers in one rounded lozenge along the forearm, a thumb toward the front.
HAND_LENGTH = 0.22
HAND_RADII = (0.055, 0.092)   # thickness, width
# Legs: hip (inside the trunk), knee, ankle (inside the foot).
LEG = [((0.15, 0.0, 0.82), 0.113), ((0.152, 0.0, 0.45), 0.103), ((0.158, 0.0, 0.12), 0.096)]
# Feet: wide puffy ovals, pushed forward, their soles on the ground.
FOOT_CENTRE = (0.172, -0.07, 0.075)
FOOT_RADII = (0.13, 0.21, 0.08)

VOXEL = 0.009          # remesh detail (units)
SOFTEN = 6             # smoothing steps where the parts meet
TARGET_TRIS = 6500     # the final budget: as few as keep the head and belly round under the outline


def ellipsoid(bm, centre, radii, rot=None, segments=28, rings=18):
    m = Matrix.Diagonal((radii[0], radii[1], radii[2], 1.0))
    if rot is not None:
        m = rot.to_4x4() @ m
    m = Matrix.Translation(Vector(centre)) @ m
    bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=1.0, matrix=m)


def sweep(bm, points, step=0.012):
    """A limb: balls along the polyline `points` [(position, radius)], each blended to the next, so the remesh sees one smooth tube."""
    for (a, ra), (b, rb) in zip(points, points[1:]):
        a, b = Vector(a), Vector(b)
        n = max(2, int((b - a).length / step))
        for k in range(n + 1):
            t = k / n
            r = ra + (rb - ra) * t
            ellipsoid(bm, a.lerp(b, t), (r, r, r), segments=14, rings=10)


def trunk(bm, segments=40):
    rings = []
    for z, a, b, fwd in TRUNK:
        ring = []
        for i in range(segments):
            t = 2.0 * math.pi * i / segments
            # (Superellipse-ish: a little squarer than an ellipse across the shoulders and hips, rounder at the belly.)
            ring.append(bm.verts.new((a * math.cos(t), -fwd + b * math.sin(t), z)))
        rings.append(ring)
    for r0, r1 in zip(rings, rings[1:]):
        for i in range(segments):
            j = (i + 1) % segments
            bm.faces.new((r0[i], r0[j], r1[j], r1[i]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])


def build(mirror_x):
    bm = bmesh.new()
    ellipsoid(bm, HEAD_CENTRE, HEAD_RADII, segments=40, rings=28)
    trunk(bm)
    for sx in (-1.0, 1.0):
        arm = [((sx * p[0], p[1], p[2]), r) for p, r in ARM]
        sweep(bm, arm)
        # The hand: a flattened lozenge continuing the forearm, its flat side toward the body.
        wrist = Vector(arm[-1][0])
        along = (wrist - Vector(arm[-2][0])).normalized()
        centre = wrist + along * (HAND_LENGTH * 0.5)
        rot = Vector((0.0, 0.0, -1.0)).rotation_difference(along).to_matrix()
        ellipsoid(bm, centre, (HAND_RADII[1], HAND_RADII[0], HAND_LENGTH * 0.5), rot=rot, segments=20, rings=14)
        # The thumb, toward the front.
        ellipsoid(bm, wrist + along * 0.06 + Vector((0.0, -0.045, 0.0)), (0.03, 0.03, 0.05), rot=rot, segments=12, rings=8)
        leg = [((sx * p[0], p[1], p[2]), r) for p, r in LEG]
        sweep(bm, leg)
        ellipsoid(bm, (sx * FOOT_CENTRE[0], FOOT_CENTRE[1], FOOT_CENTRE[2]), FOOT_RADII, segments=28, rings=16)
    me = bpy.data.meshes.new("BaseBody")
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new("BaseBody", me)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def finish(obj):
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    # One skin: the parts' volumes joined.
    rm = obj.modifiers.new("remesh", "REMESH")
    rm.mode = "VOXEL"
    rm.voxel_size = VOXEL
    bpy.ops.object.modifier_apply(modifier=rm.name)
    # Softened where the parts meet (armpits, hips, the feet's tops), keeping the head's crease on the shoulders readable.
    sm = obj.modifiers.new("soften", "CORRECTIVE_SMOOTH")
    sm.iterations = SOFTEN
    sm.factor = 0.5
    sm.smooth_type = "LENGTH_WEIGHTED"
    sm.use_only_smooth = True
    bpy.ops.object.modifier_apply(modifier=sm.name)
    tris = sum(len(p.vertices) - 2 for p in obj.data.polygons)
    dec = obj.modifiers.new("reduce", "DECIMATE")
    dec.ratio = min(1.0, TARGET_TRIS / max(tris, 1))
    dec.use_symmetry = True
    dec.symmetry_axis = "X"
    bpy.ops.object.modifier_apply(modifier=dec.name)
    # Soles flat on the ground.
    for v in obj.data.vertices:
        if v.co.z < 0.012:
            v.co.z = max(v.co.z, 0.0) * 0.3
    for p in obj.data.polygons:
        p.use_smooth = True
    mat = bpy.data.materials.new("Skin")
    mat.diffuse_color = (0.98, 0.58, 0.18, 1.0)
    obj.data.materials.append(mat)
    return sum(len(p.vertices) - 2 for p in obj.data.polygons)


def preview(obj, path):
    """Front and side views, flat lit, for comparing against the concept sheet."""
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_object_outline = True
    scene.render.resolution_x = 640
    scene.render.resolution_y = 720
    scene.render.film_transparent = False
    world = bpy.data.worlds.new("w")
    scene.world = world
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = 2.6
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    base, ext = os.path.splitext(path)
    for name, loc, rot in (("front", (0.0, -6.0, 1.1), (math.pi / 2, 0.0, 0.0)),
                           ("side", (6.0, 0.0, 1.1), (math.pi / 2, 0.0, math.pi / 2))):
        cam.location = loc
        cam.rotation_euler = rot
        scene.render.filepath = "%s_%s%s" % (base, name, ext)
        bpy.ops.render.render(write_still=True)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    out = opts.get("out", "art/models/base_body/base_body.glb")
    bpy.ops.wm.read_factory_settings(use_empty=True)
    obj = build(True)
    tris = finish(obj)
    lo = min(v.co.z for v in obj.data.vertices)
    hi = max(v.co.z for v in obj.data.vertices)
    print("base body: %d triangles, %.3f tall" % (tris, hi - lo))
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=os.path.abspath(out), use_selection=True, export_format="GLB", export_yup=True)
    if "preview" in opts:
        preview(obj, os.path.abspath(opts["preview"]))


main()
