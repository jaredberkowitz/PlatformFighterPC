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

# ---- Proportions (units; first read off the concept's front and side views, scaled to 2.2 tall, then made cuter on request: a bigger,
# rounder head, a shorter and plumper trunk, shorter thicker legs and arms, puffier feet, all within the same 2.2 so the game's fit of
# hats, camera and hurtboxes holds) -------------------------------------------------------------------------------------------------------
HEIGHT = 2.2
HEAD_CENTRE = Vector((0.0, 0.0, 1.725))
HEAD_RADII = Vector((0.475, 0.455, 0.465))
# The trunk as rings from the top (hidden inside the head) to the crotch: (height, half-width, half-depth, forward shift of the ring's
# middle). The shoulders tuck under the head (no neck) and are slim front to back there, so the head overhangs them all round and its
# outline reads as a whole circle, as on the sheet; the belly is the widest and pushes forward, with a little seat behind.
TRUNK = [
    (1.42, 0.13, 0.1, 0.0),
    (1.36, 0.235, 0.13, 0.0),
    (1.29, 0.26, 0.15, 0.01),
    (1.18, 0.28, 0.185, 0.02),
    (1.06, 0.3, 0.22, 0.04),
    (0.93, 0.318, 0.24, 0.058),
    (0.8, 0.315, 0.232, 0.048),
    (0.68, 0.295, 0.212, 0.016),
    (0.58, 0.245, 0.17, -0.005),
    (0.505, 0.13, 0.09, 0.0),
]
# Arms: shoulder (inside the trunk), elbow, wrist, each with its radius; hanging out at about 20 degrees, the hands ending at the top of
# the thigh. Parts must stand at least 2% of the height apart below the armpits and the crotch, or fit_body.py cannot tell them from the
# trunk.
# (The upper arm a little fuller than first built, as the concept's soft tube arms are: thin arms read as sticks at play size.)
# (Chunkier, after the owner's PEAK reference: thick upper arm and forearm that read at play size, a little longer so the hands swing
# clear of the belly. Was 0.117 / 0.103 / 0.088, ending at 0.585, 0.885.)
ARM = [((0.25, 0.0, 1.27), 0.15), ((0.47, 0.012, 1.05), 0.136), ((0.62, 0.025, 0.84), 0.12)]
# A mitten hand: palm and fingers in one rounded lozenge along the forearm, a thumb toward the front.
HAND_LENGTH = 0.2
HAND_RADII = (0.062, 0.1)   # thickness, width
# Legs: hip (inside the trunk), knee, ankle (inside the foot): short and thick.
LEG = [((0.17, 0.0, 0.57), 0.152), ((0.172, 0.0, 0.34), 0.143), ((0.176, 0.0, 0.13), 0.13)]
# Feet: wide puffy ovals, pushed forward, their soles on the ground.
FOOT_CENTRE = (0.18, -0.075, 0.085)
FOOT_RADII = (0.145, 0.225, 0.09)

VOXEL = 0.009          # remesh detail (units)
SOFTEN = 6             # smoothing steps where the parts meet
TARGET_TRIS = 6500     # the final budget: as few as keep the head and belly round under the outline
# How the skin is reduced to that budget: "quad" re-lays it as an even flow of quads (QuadriFlow), which keeps the normals smooth, so the
# fighters' hard cel step draws a clean light-and-shadow edge across the body; "decimate" collapses edges, which left long thin triangles
# whose normals made that edge ragged.
REDUCE = "quad"
RELAX = 4              # smoothing steps after the quad remesh (it snaps to the surface, leaving a slight ripple)


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


def mend_folds(obj):
    """The symmetric decimation can leave a fin on the mirror line: a point joined to only two others by a thin triangle and its back,
    on copies of its corners (one under the chin faced inward and showed as a speck of ink). Here copies are joined, the skin is cut into
    triangles, and any point with only two edges (in a closed skin of triangles every point has three or more) is deleted with its fin,
    so every edge has two faces."""
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=1e-5)
    bmesh.ops.triangulate(bm, faces=bm.faces[:], quad_method="BEAUTY", ngon_method="BEAUTY")
    fins = [v for v in bm.verts if len(v.link_edges) <= 2]
    if fins:
        bmesh.ops.delete(bm, geom=fins, context="VERTS")
    # Deleting a fin (or a fold the decimation left) can open a slit in the skin, which showed as a crack of background through the chin
    # and the belly: every open edge is closed again, and the patches cut into triangles.
    for _ in range(3):
        open_edges = [e for e in bm.edges if len(e.link_faces) < 2]
        if not open_edges:
            break
        bmesh.ops.holes_fill(bm, edges=open_edges, sides=0)
        bmesh.ops.triangulate(bm, faces=[f for f in bm.faces if len(f.verts) > 3])
    print("open edges left: %d" % len([e for e in bm.edges if len(e.link_faces) < 2]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.to_mesh(obj.data)
    bm.free()


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
    if REDUCE == "quad":
        bpy.ops.object.quadriflow_remesh(target_faces=TARGET_TRIS // 2, use_mesh_symmetry=False, use_preserve_sharp=False,
                                         use_preserve_boundary=False, smooth_normals=False, seed=1)
        relax = obj.modifiers.new("relax", "CORRECTIVE_SMOOTH")
        relax.iterations = RELAX
        relax.factor = 0.5
        relax.smooth_type = "LENGTH_WEIGHTED"
        relax.use_only_smooth = True
        bpy.ops.object.modifier_apply(modifier=relax.name)
    else:
        tris = sum(len(p.vertices) - 2 for p in obj.data.polygons)
        dec = obj.modifiers.new("reduce", "DECIMATE")
        dec.ratio = min(1.0, TARGET_TRIS / max(tris, 1))
        dec.use_symmetry = True
        dec.symmetry_axis = "X"
        bpy.ops.object.modifier_apply(modifier=dec.name)
    mend_folds(obj)
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
