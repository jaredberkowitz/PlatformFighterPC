"""Builds the rigged blob fighter (arms, legs, a skeleton and a set of animation clips) and exports godot/models/blob_rig.glb.

Run:  blender --background --python art/blender/make_rigged_blob.py            (blob_rig.glb)
      blender --background --python art/blender/make_rigged_blob.py -- --long   (blob_rig_long.glb: the brawler's longer limbs)
      blender --background --python art/blender/make_rigged_blob.py -- --body art/models/base_body/base_body.glb           --joints art/models/base_body/joints.json                              (base_rig.glb: the generated base body)

The character is original: a big round head, a squat dumpling torso, stubby capsule limbs, mitten hands and chunky shoes, in the spirit of
docs/ART_DIRECTION.md. The game paints the flat colours (by part name), draws the face and accessories itself, and plays the clips below.

Axes while modelling (Blender, Z up): the character faces -Y; its left is -X. Every bone points up (+Z) from its joint, so a bone's local X is
the world X (a swing forwards and backwards) and the bone's own Y is the vertical axis (a twist). `pose()` takes angles in the terms a person
would use (forward, outward, twist) and does the sign bookkeeping once.

Parts are rigid (each is weighted fully to one bone), like a vinyl toy: round, readable, cheap, and the squash and lean the game adds on top
read well on it.
"""

import json
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

OUT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "godot", "models", "blob_rig.glb"))
FPS = 60

# ---- Skeleton: name -> (joint position, parent) --------------------------------------------------------------------------------------
JOINTS = {
    "root": ((0.0, 0.0, 0.0), None),
    "hips": ((0.0, 0.0, 0.6), "root"),
    "spine": ((0.0, 0.0, 0.75), "hips"),
    "head": ((0.0, 0.0, 1.2), "spine"),
}
for _s, _x in (("L", -1.0), ("R", 1.0)):
    JOINTS["armU." + _s] = ((_x * 0.5, 0.0, 1.15), "spine")
    JOINTS["armL." + _s] = ((_x * 0.66, -0.02, 0.95), "armU." + _s)
    JOINTS["hand." + _s] = ((_x * 0.74, -0.04, 0.78), "armL." + _s)
    JOINTS["thigh." + _s] = ((_x * 0.26, 0.0, 0.6), "hips")
    JOINTS["shin." + _s] = ((_x * 0.26, 0.0, 0.4), "thigh." + _s)
    JOINTS["foot." + _s] = ((_x * 0.26, -0.05, 0.22), "shin." + _s)

LIMBS = ("armU", "armL", "hand", "thigh", "shin", "foot")


def clear() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def make_armature():
    arm = bpy.data.armatures.new("Rig")
    obj = bpy.data.objects.new("Rig", arm)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode="EDIT")
    bones = {}
    for name, (pos, parent) in JOINTS.items():
        b = arm.edit_bones.new(name)
        b.head = Vector(pos)
        b.tail = Vector(pos) + Vector((0.0, 0.0, 0.12))
        bones[name] = b
    for name, (pos, parent) in JOINTS.items():
        if parent:
            bones[name].parent = bones[parent]
            bones[name].use_connect = False
    bpy.ops.object.mode_set(mode="OBJECT")
    return obj


# ---- Meshes ------------------------------------------------------------------------------------------------------------------------
def to_object(name, bm, bone, rig):
    mesh = bpy.data.meshes.new(name)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.normal_update()
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    group = obj.vertex_groups.new(name=bone)
    group.add(list(range(len(mesh.vertices))), 1.0, "REPLACE")
    mod = obj.modifiers.new("Armature", "ARMATURE")
    mod.object = rig
    obj.parent = rig
    return obj


def ball(bm, radius, centre, scale=(1.0, 1.0, 1.0), segments=32, rings=20, uvs=True):
    """A ball, unwrapped like a globe (u around, v from the bottom up) for cloth patterns and the painted brush grain."""
    if uvs:
        bm.loops.layers.uv.verify()
    res = bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=radius, calc_uvs=uvs)
    for v in res["verts"]:
        v.co.x *= scale[0]
        v.co.y *= scale[1]
        v.co.z *= scale[2]
        v.co += Vector(centre)
    return res["verts"]


def tube(bm, points, radii, segments=22, per_span=8, bulge=None):
    """A smooth limb: one continuous tube through `points` (a joint chain) with the radius running through `radii`, round caps at both
    ends. `bulge(t, span)` can add to the radius along a span (a calf, a forearm). Returns nothing; the verts are added to `bm`."""
    pts = [Vector(q) for q in points]
    rings = []
    # Samples along the chain (centre, tangent, radius).
    samples = []
    for i in range(len(pts) - 1):
        a, b = pts[i], pts[i + 1]
        for k in range(per_span + (1 if i == len(pts) - 2 else 0)):
            t = k / per_span
            c = a.lerp(b, t)
            r = radii[i] + (radii[i + 1] - radii[i]) * t
            if bulge:
                r += bulge(t, i)
            samples.append((c, (b - a).normalized(), r))
    # Round caps: a few extra rings shrinking to a point past each end.
    cap = 5
    start = [(samples[0][0] - samples[0][1] * samples[0][2] * math.sin(math.pi / 2 * (1 - j / cap)), samples[0][1],
              samples[0][2] * math.cos(math.pi / 2 * (1 - j / cap))) for j in range(cap)]
    end = [(samples[-1][0] + samples[-1][1] * samples[-1][2] * math.sin(math.pi / 2 * j / cap), samples[-1][1],
            samples[-1][2] * math.cos(math.pi / 2 * j / cap)) for j in range(1, cap + 1)]
    chain = start + samples + end
    # How far along the joint chain each ring is (0 at the first joint, 1 at the last; the caps run a little past), for the UVs.
    lengths = [(pts[i + 1] - pts[i]).length for i in range(len(pts) - 1)]
    total = sum(lengths)

    def along(c):
        best, best_s, acc = None, 0.0, 0.0
        for i in range(len(pts) - 1):
            d = pts[i + 1] - pts[i]
            t = (c - pts[i]).dot(d) / d.length_squared
            tc = max(0.0, min(1.0, t))
            dist = (pts[i] + d * tc - c).length
            open_end = (i == 0 and t < 0) or (i == len(pts) - 2 and t > 1)
            sv = acc + (t if open_end else tc) * lengths[i]
            if best is None or dist < best - 1e-6:
                best, best_s = dist, sv
            acc += lengths[i]
        return best_s / total
    ring_v = [along(c) for c, _, _ in chain]
    up = Vector((0.0, 1.0, 0.0))
    for c, t, r in chain:
        side = t.cross(up)
        if side.length < 1e-4:
            side = t.cross(Vector((1.0, 0.0, 0.0)))
        side.normalize()
        other = t.cross(side).normalized()
        ring = []
        for k in range(segments):
            a = 2 * math.pi * k / segments
            ring.append(bm.verts.new(c + (side * math.cos(a) + other * math.sin(a)) * max(r, 0.002)))
        rings.append(ring)
    uv = bm.loops.layers.uv.verify()
    for i in range(len(rings) - 1):
        for k in range(segments):
            k1 = (k + 1) % segments
            face = bm.faces.new((rings[i][k], rings[i][k1], rings[i + 1][k1], rings[i + 1][k]))
            # u round the limb, v along it (1 - v, so the game reads 0 at the shoulder or hip and 1 at the wrist or ankle).
            for loop, (ri, kk) in zip(face.loops, ((i, k), (i, k + 1), (i + 1, k + 1), (i + 1, k))):
                loop[uv].uv = (kk / segments, 1.0 - ring_v[ri])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)


def chain_weights(points, bones, blend):
    """Smooth skinning along a joint chain: a vertex goes to the bone of the span it is nearest, blending into the next bone over `blend`
    either side of each joint, so a bent elbow or knee stays round."""
    pts = [Vector(q) for q in points]
    lengths = [(pts[i + 1] - pts[i]).length for i in range(len(pts) - 1)]
    joints = [sum(lengths[:i + 1]) for i in range(len(lengths) - 1)]

    def arclen(co):
        best, best_s = None, 0.0
        acc = 0.0
        for i in range(len(pts) - 1):
            a, b = pts[i], pts[i + 1]
            d = b - a
            t = max(0.0, min(1.0, (co - a).dot(d) / d.length_squared))
            dist = (a + d * t - co).length
            if best is None or dist < best:
                best, best_s = dist, acc + t * lengths[i]
            acc += lengths[i]
        return best_s

    def weights(co):
        s = arclen(co)
        w = {bones[0]: 1.0}
        for j, sj in enumerate(joints):
            k = (s - (sj - blend)) / (2 * blend)
            k = max(0.0, min(1.0, k))
            k = k * k * (3 - 2 * k)
            if k <= 0.0:
                break
            w = {b: v * (1 - k) for b, v in w.items()}
            w[bones[j + 1]] = w.get(bones[j + 1], 0.0) + k
        return w
    return weights


def to_object_weighted(name, bm, weights, rig):
    """Like `to_object`, but each vertex is weighted to several bones by `weights(co)` ({bone: weight})."""
    mesh = bpy.data.meshes.new(name)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.normal_update()
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    groups = {}
    for v in mesh.vertices:
        for bone, w in weights(v.co).items():
            if w <= 0.0:
                continue
            if bone not in groups:
                groups[bone] = obj.vertex_groups.new(name=bone)
            groups[bone].add([v.index], w, "REPLACE")
    mod = obj.modifiers.new("Armature", "ARMATURE")
    mod.object = rig
    obj.parent = rig
    return obj


def capsule(bm, p0, r0, p1, r1):
    """A tapered capsule between two points: the convex hull of a ball at each end."""
    before = set(bm.verts)
    ball(bm, r0, p0, segments=20, rings=12)
    ball(bm, r1, p1, segments=20, rings=12)
    new = [v for v in bm.verts if v not in before]
    hull = bmesh.ops.convex_hull(bm, input=new, use_existing_faces=False)
    bmesh.ops.delete(bm, geom=hull["geom_interior"] + hull["geom_unused"], context="VERTS")


## The square of the head front the face texture covers (Blender units, centred on the head's middle line).
FACE_SIZE = 1.155
FACE_CENTRE = 1.585


# Clothing skinned across the joints it covers, so it bends with the body instead of slicing through the next piece: the shirt and the
# shorts blend from the hips to the spine across the waist (both the same way, so their edges stay together), the shorts' legs from the
# hips into the thighs, and the sleeves from the chest into the upper arms at the shoulder.
def _smooth(a, b, v):
    k = max(0.0, min(1.0, (v - a) / (b - a)))
    return k * k * (3 - 2 * k)


def waist_weights(co):
    k = _smooth(0.74, 0.92, co.z)
    return {"hips": 1.0 - k, "spine": k}


## How much thicker the limbs are than the first model's.
LIMB = 1.32


def fist(bm, x):
    """A fist on the `x` side (-1 left, 1 right) at the blob's wrist: a round glove clenched, the four curled fingers a row of knuckle
    bumps underneath, the thumb wrapped across the front."""
    palm = Vector((x * 0.775, -0.04, 0.66))
    ball(bm, 0.205, palm, (0.9, 1.0, 0.92), segments=32, rings=20)
    for fy in (-0.135, -0.045, 0.045, 0.135):
        ball(bm, 0.082, (x * 0.8, -0.04 + fy, 0.53), (1.0, 0.95, 0.9), segments=16, rings=10)
    tube(bm, [(x * 0.69, -0.17, 0.68), (x * 0.72, -0.24, 0.6), (x * 0.8, -0.22, 0.55)], [0.07, 0.066, 0.06], segments=14,
         per_span=4)


## The blob's wrist, which `fist` and `cuff` are built round.
BLOB_WRIST = (0.74, -0.04, 0.78)


def cuff(bm, x):
    """A rolled glove cuff just above the mitten on the `x` side: a flat disc square to the forearm, wider than the arm."""
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=24, radius1=0.17 * LIMB, radius2=0.19 * LIMB, depth=0.1)
    elbow = Vector((x * 0.66, -0.02, 0.95))
    wrist = Vector((x * 0.74, -0.04, 0.8))
    axis = (wrist - elbow).normalized()
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(axis).to_matrix().to_4x4()
    bmesh.ops.transform(bm, matrix=rot, verts=bm.verts)
    bmesh.ops.translate(bm, vec=elbow + (wrist - elbow) * 0.62, verts=bm.verts)


def build_meshes(rig):
    parts = {}

    bm = bmesh.new()
    ball(bm, 0.56, (0, 0, 0.98), (1.0, 0.9, 0.75), uvs=True)
    for v in bm.verts:
        t = -(v.co.z - 0.98) / 0.42
        v.co.x *= 1.0 + 0.08 * t
        v.co.y *= 1.0 + 0.08 * t
    parts["Body"] = to_object_weighted("Body", bm, waist_weights, rig)

    bm = bmesh.new()
    ball(bm, 0.66, (0, 0, 1.56), (1.0, 0.97, 0.94), segments=36, rings=24)
    for v in bm.verts:
        t = -(v.co.z - 1.56) / 0.66
        if t > 0:
            v.co.x *= 1.0 + 0.04 * t
            v.co.y *= 1.0 + 0.04 * t
    parts["Head"] = to_object("Head", bm, "head", rig)

    # The face: a shell just outside the front of the head that carries the drawn face texture (godot/art/faces/*.svg). Its UVs are a
    # straight-on projection of a FACE_SIZE square centred at FACE_CENTRE, so the artwork lands on the head the way it is drawn.
    bm = bmesh.new()
    ball(bm, 0.66 * 1.025, (0, 0, 1.56), (1.0, 0.97, 0.94), segments=48, rings=32)
    for v in bm.verts:
        t = -(v.co.z - 1.56) / 0.66
        if t > 0:
            v.co.x *= 1.0 + 0.04 * t
            v.co.y *= 1.0 + 0.04 * t
    back = [v for v in bm.verts if v.co.y > -0.2]
    bmesh.ops.delete(bm, geom=back, context="VERTS")
    uv = bm.loops.layers.uv.verify()
    for face in bm.faces:
        for loop in face.loops:
            co = loop.vert.co
            loop[uv].uv = (0.5 + co.x / FACE_SIZE, 0.5 + (co.z - FACE_CENTRE) / FACE_SIZE)
    parts["Face"] = to_object("Face", bm, "head", rig)

    # Limbs are chunky (LIMB times the first model's thickness) so arms and legs read at play distance, with big cartoon gloves and shoes,
    # and a ball at each elbow and knee so a bent joint stays round instead of pinching.
    T = LIMB
    for side, x in (("L", -1.0), ("R", 1.0)):
        # The arm: one smoothly skinned tube from the shoulder through the elbow to the wrist (a little fuller at the upper arm and the
        # forearm), so it bends like an arm instead of two capsules hinging. The shirt sleeve is a band the game paints on it from the
        # shoulder (godot/shaders/limb.gdshader), so nothing can clip through it.
        arm_pts = [(x * 0.47, 0.0, 1.18), (x * 0.66, -0.02, 0.95), (x * 0.75, -0.04, 0.76)]
        bm = bmesh.new()
        tube(bm, arm_pts, [0.168, 0.15, 0.135], bulge=lambda t, i: 0.012 * math.sin(math.pi * t))
        parts["Arm." + side] = to_object_weighted("Arm." + side, bm, chain_weights(arm_pts, ["armU." + side, "armL." + side], 0.06), rig)

        bm = bmesh.new()
        fist(bm, x)
        parts["Hand." + side] = to_object("Hand." + side, bm, "hand." + side, rig)
        bm = bmesh.new()
        cuff(bm, x)
        parts["Cuff." + side] = to_object("Cuff." + side, bm, "hand." + side, rig)

        # The leg: one smoothly skinned tube from the hip through the knee into the shoe, with a little calf; the shorts' leg and the sock
        # are bands the game paints on it, so they bend with it and never clip.
        leg_pts = [(x * 0.26, 0.0, 0.66), (x * 0.26, 0.0, 0.42), (x * 0.26, -0.01, 0.22)]
        leg_w = chain_weights(leg_pts, ["thigh." + side, "shin." + side], 0.06)
        calf = lambda t, i: 0.014 * math.sin(math.pi * min(1.0, t * 1.6)) if i == 1 else 0.0
        bm = bmesh.new()
        tube(bm, leg_pts, [0.185, 0.165, 0.155], bulge=calf)
        parts["Leg." + side] = to_object_weighted("Leg." + side, bm, leg_w, rig)

        # A chunky shoe: a flat sole and a rounded toe cap.
        bm = bmesh.new()
        ball(bm, 0.26, (x * 0.26, -0.07, 0.17), (1.0, 1.35, 0.8), segments=28, rings=18)
        for v in bm.verts:
            if v.co.z < 0.08:
                v.co.z = 0.08 + (v.co.z - 0.08) * 0.15
        parts["Foot." + side] = to_object("Foot." + side, bm, "foot." + side, rig)

        # A strap across the top of the shoe (in the outfit colour): a band of the shoe's own shape, a hair bigger. The ball is built with
        # its rings running across the shoe (poles front and back), so the band's edges are clean.
        bm = bmesh.new()
        res = bmesh.ops.create_uvsphere(bm, u_segments=28, v_segments=28, radius=0.26 * 1.05)
        bmesh.ops.rotate(bm, cent=(0, 0, 0), matrix=Matrix.Rotation(math.pi / 2.0, 3, "X"), verts=res["verts"])
        for v in res["verts"]:
            v.co.y *= 1.35
            v.co.z *= 0.8
            v.co += Vector((x * 0.26, -0.07, 0.17))
            if v.co.z < 0.08:
                v.co.z = 0.08 + (v.co.z - 0.08) * 0.15
        off = [v for v in bm.verts if not (-0.27 < v.co.y < -0.13) or v.co.z < 0.14]
        bmesh.ops.delete(bm, geom=off, context="VERTS")
        parts["Strap." + side] = to_object("Strap." + side, bm, "foot." + side, rig)

        # A sole under each shoe.
        bm = bmesh.new()
        ball(bm, 0.255, (x * 0.26, -0.07, 0.085), (1.02, 1.37, 0.18), segments=28, rings=10)
        parts["Sole." + side] = to_object("Sole." + side, bm, "foot." + side, rig)

    # The shorts: the bottom of the body, a little bigger than it, cut off at the waist.
    bm = bmesh.new()
    ball(bm, 0.585, (0, 0, 0.98), (1.0, 0.9, 0.75))
    for v in bm.verts:
        t = -(v.co.z - 0.98) / 0.42
        v.co.x *= 1.0 + 0.08 * t
        v.co.y *= 1.0 + 0.08 * t
    waist = [v for v in bm.verts if v.co.z > 0.8]
    bmesh.ops.delete(bm, geom=waist, context="VERTS")
    parts["Shorts"] = to_object_weighted("Shorts", bm, waist_weights, rig)

    # A shirt collar: a soft ring where the head meets the body.
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=False, cap_tris=False, segments=28, radius1=0.4, radius2=0.33, depth=0.12)
    bmesh.ops.translate(bm, vec=(0, 0, 1.26), verts=bm.verts)
    parts["Collar"] = to_object("Collar", bm, "spine", rig)
    return parts


# ---- Animation ---------------------------------------------------------------------------------------------------------------------
# How much the clip being built is exaggerated. Moves are played big so a player can read what is happening from far away.
AMP = 1.0


def pose(rig, bone, fwd=0.0, out=0.0, twist=0.0, lift=0.0):
    """Sets a bone's pose from forward / outward / twist degrees (and an offset for the hips: upward on the base body; on the blob, whose
    clips were tuned with it, along the hip bone's own z, which is backward and forward)."""
    fwd, out, twist, lift = fwd * AMP, out * AMP, twist * AMP, lift * AMP
    pb = rig.pose.bones[bone]
    pb.rotation_mode = "XYZ"
    limb = bone.split(".")[0] in LIMBS
    left = bone.endswith(".L")
    x = -fwd if limb else fwd
    z = (-out if left else out)
    pb.rotation_euler = (math.radians(x), math.radians(twist), math.radians(z))
    if bone == "hips":
        pb.location = (0.0, lift, 0.0) if BODY else (0.0, 0.0, lift)


def key_all(rig, frame):
    for pb in rig.pose.bones:
        pb.keyframe_insert("rotation_euler", frame=frame)
        if pb.name == "hips":
            pb.keyframe_insert("location", frame=frame)


def reset(rig):
    for pb in rig.pose.bones:
        pb.rotation_mode = "XYZ"
        pb.rotation_euler = (0, 0, 0)
        pb.location = (0, 0, 0)


BODY = "--body" in sys.argv


# ---- The base body's moves --------------------------------------------------------------------------------------------------------------
# The blob had no limbs to speak of, so its moves were posed big and then exaggerated further (`clip`'s AMP) to read at all. The base body
# has real arms and legs, so its moves are played closer to life (BODY_AMP pulls the exaggeration most of the way back to 1) and the key
# moves are posed again so the body drives the hit, in the manner of the reference game's fighters of these builds (a stocky three-heads-
# tall brawler, a fencer, a wolfish striker): the shoulders and hips wind back, then turn through into the strike as the weight steps onto
# the front foot, the back leg straightening behind and the free arm pulled back for balance. Each entry is (wind-up, strike, rest) in the
# `pose_spec` short form, on the shared move timeline.
BODY_AMP = 0.3

BODY_MOVES = {
    # A straight punch (or a quick slash): the guard hand stays up at the chin, the shoulders snap round and the front foot steps in.
    "attack_jab": (
        {"hips": -0.04, "spine": (2.0, -22.0), "head": (0.0, 12.0), "stance": (14.0, -16.0), "shin.L": (-24.0,),
         "armU.L": (46.0, 18.0), "armL.L": (100.0,)},
        {"hips": -0.06, "spine": (8.0, 26.0), "head": (-4.0, -16.0), "stance": (28.0, -26.0), "shin.L": (-30.0,), "shin.R": (-4.0,),
         "armU.L": (30.0, 16.0), "armL.L": (110.0,)},
        {"spine": (2.0, 6.0), "stance": (8.0, -6.0)}),
    # A swinging strike across the body (a claw swipe, a backhanded cut): the torso turns right round through it.
    "attack_swing": (
        {"hips": -0.06, "spine": (-6.0, -32.0), "head": (4.0, 16.0), "stance": (16.0, -20.0), "shin.L": (-20.0,),
         "armU.L": (30.0, 24.0), "armL.L": (80.0,)},
        {"hips": -0.08, "spine": (12.0, 34.0), "head": (-6.0, -18.0), "stance": (34.0, -30.0), "shin.L": (-34.0,), "shin.R": (-6.0,),
         "armU.L": (-20.0, 30.0), "armL.L": (40.0,)},
        {"spine": (4.0, 10.0), "stance": (12.0, -10.0)}),
    # A low strike from a crouch.
    "attack_low": (
        {"hips": -0.16, "spine": (18.0, -14.0), "stance": (50.0, 20.0), "shin.L": (-70.0,), "shin.R": (-90.0,)},
        {"hips": -0.22, "spine": (30.0, 16.0), "head": (-14.0, -8.0), "stance": (66.0, 10.0), "shin.L": (-60.0,), "shin.R": (-110.0,),
         "armU.L": (10.0, 40.0)},
        {"hips": -0.1, "spine": (14.0, 4.0), "stance": (36.0, 10.0)}),
    # A smash: the weight rocks back onto the bent back leg with the shoulders turned away, then the whole body is thrown into a deep
    # lunge, the front knee bent and the back leg straight behind (as the stocky brawler's palm thrust).
    "attack_smash": (
        {"hips": -0.08, "spine": (-10.0, -38.0), "head": (8.0, 20.0), "stance": (22.0, 28.0), "shin.L": (-16.0,), "shin.R": (-56.0,),
         "armU.L": (50.0, 20.0), "armL.L": (60.0,)},
        {"spine": (22.0, 34.0), "head": (-10.0, -18.0), "stance": (46.0, -32.0), "shin.L": (-40.0,), "shin.R": (-2.0,),
         "armU.L": (-36.0, 26.0), "armL.L": (20.0,)},
        {"hips": -0.05, "spine": (10.0, 12.0), "stance": (28.0, -22.0), "shin.L": (-24.0,)}),
    # A running lunge: gather, then stretch out long over the front foot.
    "attack_lunge": (
        {"hips": -0.12, "spine": (-8.0, -10.0), "head": (8.0, 0.0), "stance": (20.0, 26.0), "shin.R": (-60.0,), "armU.L": (-20.0, 22.0)},
        {"spine": (26.0, 10.0), "head": (-18.0, 0.0), "stance": (50.0, -36.0), "shin.L": (-40.0,), "shin.R": (-2.0,),
         "armU.L": (-40.0, 30.0), "armL.L": (20.0,)},
        {"spine": (12.0, 0.0), "stance": (28.0, -22.0)}),
    # Fencer's forward tilt: a rising diagonal cut, stepping in and rising onto the front foot as the blade climbs, the free arm opening
    # back for balance.
    "sword_ftilt": (
        {"hips": -0.08, "spine": (12.0, -28.0), "head": (6.0, 14.0), "stance": (18.0, -14.0), "shin.L": (-30.0,), "armU.L": (-10.0, 40.0),
         "armL.L": (30.0,)},
        {"hips": -0.04, "spine": (-4.0, 30.0), "head": (-8.0, -14.0), "stance": (36.0, -28.0), "shin.L": (-26.0,), "shin.R": (-4.0,),
         "armU.L": (-30.0, 50.0), "armL.L": (20.0,)},
        {"spine": (2.0, 10.0), "stance": (14.0, -10.0)}),
    # Fencer's forward smash: the blade raised high with the weight back, then one committed cut down in front into a deep lunge, the
    # front knee well bent, the back leg straight, the free arm thrown back.
    "sword_fsmash": (
        {"hips": -0.02, "spine": (-16.0, -40.0), "head": (10.0, 20.0), "stance": (24.0, 18.0), "shin.R": (-40.0,), "armU.L": (20.0, 40.0)},
        {"spine": (22.0, 30.0), "head": (-6.0, -14.0), "stance": (48.0, -34.0), "shin.L": (-44.0,), "shin.R": (-2.0,),
         "armU.L": (-50.0, 40.0), "armL.L": (10.0,)},
        {"hips": -0.08, "spine": (14.0, 12.0), "stance": (34.0, -28.0), "shin.L": (-36.0,)}),
    # Fencer's down tilt: a low lunging thrust, down on the bent front leg with the back leg stretched long behind.
    "sword_dtilt": (
        {"hips": -0.2, "spine": (20.0, -10.0), "head": (-10.0, 0.0), "stance": (50.0, 26.0), "shin.L": (-90.0,), "shin.R": (-100.0,)},
        {"hips": -0.28, "spine": (34.0, 10.0), "head": (-24.0, 0.0), "stance": (70.0, -30.0), "shin.L": (-92.0,), "shin.R": (-20.0,),
         "armU.L": (-30.0, 30.0)},
        {"hips": -0.16, "spine": (22.0, 0.0), "stance": (48.0, -8.0), "shin.L": (-70.0,)}),
    # Fencer's up tilt: the blade sweeps overhead from front to back while the body stands tall and arches back a little under it.
    "sword_utilt": (
        {"hips": -0.06, "spine": (14.0, 10.0), "head": (6.0, 0.0), "stance": (12.0, -10.0), "shin.L": (-20.0,)},
        {"hips": 0.0, "spine": (-20.0, -8.0), "head": (-18.0, 0.0), "stance": (14.0, -18.0), "armU.L": (-20.0, 30.0)},
        {"spine": (-6.0, 0.0), "stance": (8.0, -8.0)}),
}


# On the ground the base body stands on its feet: whatever the legs are posed to, the hips are raised or lowered so the lower foot is on
# the ground. In the standing moves (STANDING and BODY_MOVES) both feet stay down: the hips sink until the foot of the wider-spread leg
# touches, and the other leg is raised to put its foot down too (a lunge: the front knee lifted and bent, the back leg long), the soles
# level. The walk and run lift and roll their own feet, and the run and dash leave the ground between steps (STRIDING). The blob's stubby
# legs never needed any of it.
def grounded(name):
    """Whether a clip is played on the ground (the feet planted), not in the air, hit, rolling, lying down or on a ledge."""
    if name in ("jump", "fall", "air_jump", "hurt", "roll", "knockdown", "ledge", "ledge_climb", "kick_dash"):
        return False
    return not name.endswith("air")


STANDING = {"idle", "crouch", "shield", "skid", "grab", "throw", "blaster", "sword_usmash", "sword_dsmash"}


def _ankle(rig, side):
    bpy.context.view_layer.update()
    return rig.pose.bones["foot." + side].head


def _level_sole(rig, side):
    # A bone's x turn adds along the leg (every bone rests pointing up), so the foot undoes the thigh and shin's.
    foot = rig.pose.bones["foot." + side]
    foot.rotation_euler.x = -(rig.pose.bones["thigh." + side].rotation_euler.x + rig.pose.bones["shin." + side].rotation_euler.x)


def _bring_down(rig, side, ground):
    """Turns the leg about the hip (toward hanging straight down) until its foot is on the ground; straightens the knee if it is short."""
    thigh = rig.pose.bones["thigh." + side]
    if _ankle(rig, side).z - ground < 0.004:
        return
    # Which way down is: a small turn each way.
    x0 = thigh.rotation_euler.x
    thigh.rotation_euler.x = x0 + 0.02
    up = _ankle(rig, side).z
    thigh.rotation_euler.x = x0 - 0.02
    step = 0.02 if up < _ankle(rig, side).z else -0.02
    lo, hi = x0, None
    best = (_ankle(rig, side).z, x0)
    x = x0
    for _ in range(80):
        x += step
        thigh.rotation_euler.x = x
        z = _ankle(rig, side).z
        if z <= ground:
            hi = x
            break
        if z > best[0] + 1e-5:
            break   # past straight down: the leg is too short as bent
        best = (z, x)
        lo = x
    if hi is None:
        thigh.rotation_euler.x = best[1]
        shin = rig.pose.bones["shin." + side]
        s_lo, s_hi = shin.rotation_euler.x, 0.0
        shin.rotation_euler.x = s_hi
        if _ankle(rig, side).z > ground:
            return
        for _ in range(24):
            shin.rotation_euler.x = (s_lo + s_hi) / 2.0
            if _ankle(rig, side).z > ground:
                s_lo = shin.rotation_euler.x
            else:
                s_hi = shin.rotation_euler.x
        shin.rotation_euler.x = s_hi
        return
    for _ in range(24):
        thigh.rotation_euler.x = (lo + hi) / 2.0
        if _ankle(rig, side).z > ground:
            lo = thigh.rotation_euler.x
        else:
            hi = thigh.rotation_euler.x
    thigh.rotation_euler.x = hi


def _lift_onto(rig, side, ground):
    """Raises a foot that is below the ground onto it: a leg in front lifts its knee (the thigh turns up, the shin keeps its slant, so the
    foot stays out in front: a lunge), a leg behind swings further back. Falls back on bending the knee."""
    thigh = rig.pose.bones["thigh." + side]
    shin = rig.pose.bones["shin." + side]
    if _ankle(rig, side).z >= ground - 0.004:
        return
    front = thigh.rotation_euler.x <= 0.0     # (a limb's x turn is minus its forward angle)
    t0, s0 = thigh.rotation_euler.x, shin.rotation_euler.x

    def turn(k):
        thigh.rotation_euler.x = t0 - k if front else t0 + k
        shin.rotation_euler.x = s0 + k if front else s0

    lo, hi = 0.0, None
    k = 0.0
    while k < math.radians(70.0):
        k += math.radians(3.0)
        turn(k)
        if _ankle(rig, side).z >= ground:
            hi = k
            break
        lo = k
    if hi is None:
        turn(0.0)
        _bend_up(rig, side, ground)
        return
    for _ in range(24):
        turn((lo + hi) / 2.0)
        if _ankle(rig, side).z < ground:
            lo = (lo + hi) / 2.0
        else:
            hi = (lo + hi) / 2.0
    turn(hi)


def _bend_up(rig, side, ground):
    """Bends the knee until a foot below the ground is on it."""
    shin = rig.pose.bones["shin." + side]
    if _ankle(rig, side).z >= ground - 0.004:
        return
    lo = shin.rotation_euler.x
    limit = lo + math.radians(150.0)
    hi = None
    x = lo
    while x < limit:
        x = min(x + math.radians(3.0), limit)
        shin.rotation_euler.x = x
        if _ankle(rig, side).z >= ground:
            hi = x
            break
        lo = x
    if hi is None:
        return
    for _ in range(24):
        shin.rotation_euler.x = (lo + hi) / 2.0
        if _ankle(rig, side).z < ground:
            lo = shin.rotation_euler.x
        else:
            hi = shin.rotation_euler.x
    shin.rotation_euler.x = hi


## Clips with a moment in the air between steps: their feet are only kept from going through the ground.
STRIDING = {"run", "dash"}


def plant(rig, both, lift_only=False):
    """Puts the lower foot on the ground by raising or lowering the hips (with `lift_only`, only raising them, if it is below); with
    `both`, sinks to the higher foot and raises the other leg to put that foot down too."""
    ground = rig.pose.bones["foot.L"].bone.head_local.z
    hips = rig.pose.bones["hips"]
    feet = [_ankle(rig, side).z for side in "LR"]
    hips.location.y += max(0.0, ground - min(feet)) if lift_only else ground - (max(feet) if both else min(feet))
    if both:
        for side in "LR":
            _lift_onto(rig, side, ground)
            _bring_down(rig, side, ground)
            _level_sole(rig, side)


def _spec_pose(spec):
    """A pose from the short form: {"hips": lift, "spine": (fwd, twist), "head": (fwd, twist), "stance": (lead, back), "arms": (fwd,
    out), bone: (fwd, out)...}; the free arm and legs not named rest."""
    def f(rig):
        arms = spec.get("arms", (8.0, 14.0))
        for side in "LR":
            pose(rig, "armU." + side, fwd=arms[0], out=arms[1])
            pose(rig, "armL." + side, fwd=30.0)
        if "hips" in spec:
            pose(rig, "hips", lift=spec["hips"])
        if "spine" in spec:
            pose(rig, "spine", fwd=spec["spine"][0], twist=spec["spine"][1])
        if "head" in spec:
            pose(rig, "head", fwd=spec["head"][0], twist=spec["head"][1])
        if "stance" in spec:
            lead, back = spec["stance"]
            # The further the legs step apart front to back, the more the feet come in toward one line (as a fencer's): seen turned three
            # quarters to the camera, feet set wide apart sideways would hide the step.
            pose(rig, "thigh.L", fwd=lead, out=5.0 - 0.4 * abs(lead))
            pose(rig, "shin.L", fwd=-8.0 - max(0.0, -lead) * 0.5)
            pose(rig, "thigh.R", fwd=back, out=5.0 - 0.4 * abs(back))
            pose(rig, "shin.R", fwd=-8.0 - max(0.0, -back) * 0.5)
        for bone, v in spec.items():
            if "." in bone:
                pose(rig, bone, fwd=v[0], out=v[1] if len(v) > 1 else 0.0)
    return f


def _spec_held(spec, k):
    scaled = {key: (tuple(x * k for x in v) if isinstance(v, tuple) else v * k) for key, v in spec.items()}
    return _spec_pose(scaled)


def _amped_pose(fn, k):
    def f(rig):
        global AMP
        keep = AMP
        AMP = keep * k
        fn(rig)
        AMP = keep
    return f


def clip(rig, name, frames, poses):
    """`poses` maps a frame to a function that sets the pose; the clip is keyed at each of those frames. For the base body (--body) the
    exaggeration is pulled back toward life and its own poses replace the listed moves' (BODY_MOVES)."""
    global AMP
    if name.startswith("attack") or name in ("grab", "throw"):
        AMP = 1.85
    elif name.startswith(("sword_", "kick_")) or name == "blaster":
        AMP = 1.75
    elif name in ("walk", "run", "dash", "skid"):
        AMP = 1.0
    elif name in ("jump", "fall", "hurt", "crouch", "shield", "roll", "knockdown"):
        AMP = 1.25
    else:
        AMP = 1.0
    if BODY:
        AMP = 1.0 + (AMP - 1.0) * BODY_AMP
        if name in BODY_MOVES:
            wind, strike, rest = BODY_MOVES[name]
            poses = {0: _spec_held(rest, 0.0), 21: _amped_pose(_spec_pose(wind), 1.12), 33: _spec_pose(strike),
                     38: _amped_pose(_spec_pose(strike), 1.12), 45: _spec_held(rest, 1.0), 60: _spec_held(rest, 0.0)}
    action = bpy.data.actions.new(name)
    action.use_fake_user = True
    if rig.animation_data is None:
        rig.animation_data_create()
    rig.animation_data.action = action
    for frame in sorted(poses):
        reset(rig)
        poses[frame](rig)
        if BODY and grounded(name):
            plant(rig, name in BODY_MOVES or name in STANDING, name in STRIDING)
        key_all(rig, frame)
    action.frame_range = (0, max(poses))


def walk_pose(phase, stride, bend, arm, lean, bob, elbow, twist):
    """One frame of a walk, run or dash cycle at `phase` (0..1).

    Legs: the leg swinging forward lifts its knee (the recovery), the planted leg is nearly straight. Arms: opposite the legs, with the
    elbow bent more as the arm comes forward (a pumping run). Body: leans into the run, bounces twice a cycle (once per step) and the
    shoulders counter-twist against the hips; the head stays level-ish.
    """
    s = math.sin(phase * 2 * math.pi)
    c = math.cos(phase * 2 * math.pi)

    def f(rig):
        for side, sign in (("L", 1.0), ("R", -1.0)):
            swing = s * sign
            lift = max(0.0, c * sign)  # this leg is coming forward: the knee lifts
            pose(rig, "thigh." + side, fwd=stride * swing)
            pose(rig, "shin." + side, fwd=-bend * lift - 5.0 - 0.1 * bend * max(0.0, -swing))
            pose(rig, "foot." + side, fwd=22.0 * lift - 0.25 * stride * swing)
            forward = max(0.0, -swing)  # the arm that is forward
            pose(rig, "armU." + side, fwd=-arm * swing, out=6.0)
            pose(rig, "armL." + side, fwd=elbow * (0.55 + 0.45 * forward))
        pose(rig, "hips", lift=bob * abs(c) - bob * 0.5, twist=-twist * s)
        pose(rig, "spine", fwd=lean, twist=twist * 1.4 * s)
        pose(rig, "head", fwd=-lean * 0.8, twist=-twist * 1.2 * s)

    return f


def periodic(table, t):
    """Linear interpolation through `table` ([(phase, value), ...], phases in 0..1, sorted) around a loop, at phase `t`."""
    t %= 1.0
    for i, (p0, v0) in enumerate(table):
        p1, v1 = table[(i + 1) % len(table)]
        if p1 <= p0:
            p1 += 1.0
        tt = t if t >= p0 else t + 1.0
        if p0 <= tt <= p1:
            k = (tt - p0) / (p1 - p0)
            k = k * k * (3.0 - 2.0 * k)  # ease in and out between the key poses
            return v0 + (v1 - v0) * k
    return table[0][1]


# One leg through a running stride, by its own phase: contact (heel down in front), down (the weight lands, knee bends), push off (leg
# straight behind), the heel kicks right up behind, then the knee drives high in front and the leg reaches out for the next contact.
RUN_THIGH = [(0.0, 42.0), (0.12, 22.0), (0.32, -38.0), (0.46, -52.0), (0.6, -18.0), (0.78, 82.0), (0.9, 70.0)]
RUN_SHIN = [(0.0, -12.0), (0.12, -40.0), (0.32, -14.0), (0.46, -70.0), (0.6, -128.0), (0.78, -112.0), (0.9, -34.0)]
RUN_FOOT = [(0.0, 12.0), (0.12, -4.0), (0.32, -26.0), (0.46, -44.0), (0.6, -30.0), (0.78, 24.0), (0.9, 20.0)]


def run_pose(phase, lean=24.0, stride=1.0, arms=1.0, bounce=0.085):
    """A big, bouncy cartoon run (think of a plumber's sprint): a strong lean, high knees and heels kicked up behind, arms pumping wide
    with bent elbows opposite the legs, the body dropping as each foot lands and springing up between steps, shoulders twisting against
    the hips and the head nodding with every step."""
    def f(rig):
        for side, offset in (("L", 0.0), ("R", 0.5)):
            leg = phase + offset
            pose(rig, "thigh." + side, fwd=stride * periodic(RUN_THIGH, leg), out=4.0)
            pose(rig, "shin." + side, fwd=periodic(RUN_SHIN, leg))
            pose(rig, "foot." + side, fwd=periodic(RUN_FOOT, leg))
            # The arm swings with the other leg: forward as that leg drives forward.
            other = leg + 0.5
            swing = math.cos(other * 2.0 * math.pi)  # +1 when the other leg lands in front
            forward = max(0.0, swing)
            pose(rig, "armU." + side, fwd=arms * (-58.0 * swing + 8.0), out=16.0 + 10.0 * forward)
            pose(rig, "armL." + side, fwd=arms * (62.0 + 46.0 * forward))
            pose(rig, "hand." + side, fwd=10.0 * swing)
        # Two steps a cycle: lowest just after each landing (0.12, 0.62), highest in the flight between them.
        step = math.cos((phase - 0.37) * 4.0 * math.pi)
        pose(rig, "hips", lift=bounce * step - bounce * 0.3, twist=-11.0 * math.sin(phase * 2.0 * math.pi))
        pose(rig, "spine", fwd=lean - 5.0 * step, twist=15.0 * math.sin(phase * 2.0 * math.pi))
        pose(rig, "head", fwd=-lean * 0.45 + 7.0 * step, twist=-9.0 * math.sin(phase * 2.0 * math.pi))
    return f


def amped(fn, k):
    """`fn`'s pose pushed `k` times further (for anticipation and overshoot keys)."""
    def f(rig):
        global AMP
        keep = AMP
        AMP = keep * k
        fn(rig)
        AMP = keep
    return f


# ---- A generated body (--body): one skinned mesh in place of the built parts -------------------------------------------------------------
# `-- --body <body.glb> --joints <joints.json>` builds the same skeleton and clips round a body made elsewhere (generated, then cleaned by
# import_generated.py --kind fighter and measured by fit_body.py, which gives the joints): its head is lowered onto its shoulders, its hands
# are cut off at the wrists for the game's glove fists (cuffs fitted to the forearms), a face shell is laid on the head for the drawn faces,
# and the skin is weighted to the skeleton automatically, the arms' weights smoothed so the elbows bend round. Writes godot/models/base_rig.glb, and base_rig.json beside it with the head and
# shoulders for the game (fighter_view.gd).
BODY = sys.argv[sys.argv.index("--body") + 1] if "--body" in sys.argv else None
FIT = None
if BODY:
    with open(sys.argv[sys.argv.index("--joints") + 1]) as _f:
        FIT = json.load(_f)
    for _name, _pos in FIT["joints"].items():
        if _name in JOINTS:
            JOINTS[_name] = (tuple(_pos), JOINTS[_name][1])
    OUT = OUT.replace("blob_rig.glb", "base_rig.glb")
## How big the glove fists are against the blob's.
FIST_SCALE = 0.85
## How far the head is lowered onto the shoulders: the generated body has a short neck between them, which the character has none of
## (measured on this body: the head's underside sits this far above the top of the shoulders).
NECK_DROP = 0.07


def smooth_weights(obj, groups, factor=0.5, repeat=6):
    """Spreads the named vertex groups' weights along the mesh (each point toward its neighbours' average, `repeat` times), then puts
    every point's weights back to a sum of one."""
    me = obj.data
    n = len(me.vertices)
    near = [[] for _ in range(n)]
    for e in me.edges:
        a, b = e.vertices
        near[a].append(b)
        near[b].append(a)
    weights = {g.index: [0.0] * n for g in obj.vertex_groups}
    for v in me.vertices:
        for g in v.groups:
            weights[g.group][v.index] = g.weight
    for name in groups:
        w = weights[obj.vertex_groups[name].index]
        for _ in range(repeat):
            w[:] = [w[i] + factor * (sum(w[j] for j in near[i]) / len(near[i]) - w[i]) if near[i] else w[i] for i in range(n)]
    for i in range(n):
        total = sum(w[i] for w in weights.values())
        if total > 0.0:
            for w in weights.values():
                w[i] /= total
    for g in obj.vertex_groups:
        w = weights[g.index]
        g.remove(list(range(n)))
        for i in range(n):
            if w[i] > 1e-4:
                g.add([i], w[i], "REPLACE")


def build_body_meshes(rig):
    parts = {}
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.abspath(BODY))
    meshes = [o for o in bpy.data.objects if o not in before and o.type == "MESH"]
    for o in [o for o in bpy.data.objects if o not in before and o.type != "MESH"]:
        bpy.data.objects.remove(o, do_unlink=True)
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.parent = None
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    skin = bpy.context.view_layer.objects.active
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    skin.name = "Skin"
    skin.data.name = "Skin"
    skin.data.materials.clear()
    J = {k: Vector(v) for k, v in FIT["joints"].items()}
    # No neck: the head is lowered onto the shoulders, the neck between them squeezed down to nothing (`neck` is where they now meet).
    neck = FIT["neck"] - NECK_DROP / 2.0
    for v in skin.data.vertices:
        v.co.z -= NECK_DROP * _smooth(FIT["neck"] - 0.05, FIT["neck"] + 0.05, v.co.z)

    # The generated hands cut off past the wrist (the glove fists take their place).
    bm = bmesh.new()
    bm.from_mesh(skin.data)
    cut = []
    wrist_r = {"L": [], "R": []}
    for v in bm.verts:
        for side, sign in (("L", -1.0), ("R", 1.0)):
            s, w = J["armU." + side], J["hand." + side]
            if v.co.x * sign < abs(s.x) * 0.7:
                continue
            axis = w - s
            length = axis.length
            d = axis / length
            t = (v.co - s).dot(d) / length
            closest = s + d * max(0.0, min(1.0, t)) * length
            off = v.co - closest
            if off.length > 0.25:
                continue
            if t > 1.03:
                cut.append(v)
                break
            # (How thick the forearm is near the wrist, for the cuffs.)
            e = J["armL." + side]
            fore = J["hand." + side] - e
            tf = (v.co - e).dot(fore) / fore.length_squared
            if 0.7 < tf <= 1.0:
                wrist_r[side].append((v.co - e - fore * tf).length)
            break
    bmesh.ops.delete(bm, geom=cut, context="VERTS")
    bm.to_mesh(skin.data)
    bm.free()
    for poly in skin.data.polygons:
        poly.use_smooth = True
    # The rest position of every point, kept in two texture maps (x and height in the first, depth in the second), so the game's skin
    # shader can paint the clothes by region (shaders/skin.gdshader) on the body as it moves.
    me = skin.data
    rest_a = me.uv_layers.new(name="rest")
    rest_b = me.uv_layers.new(name="rest_depth")
    for loop in me.loops:
        co = me.vertices[loop.vertex_index].co
        rest_a.data[loop.index].uv = (co.x, co.z)
        rest_b.data[loop.index].uv = (co.y, 0.0)
    parts["Skin"] = skin

    # Glove fists, built as the blob's and moved to this body's wrists, turned to its forearms; and cuffs fitted to its forearms, a band a
    # little wider than the wrist where the glove ends.
    for side, x in (("L", -1.0), ("R", 1.0)):
        wrist_blob = Vector((x * BLOB_WRIST[0], BLOB_WRIST[1], BLOB_WRIST[2]))
        fore_blob = (wrist_blob - Vector((x * 0.66, -0.02, 0.95))).normalized()
        fore = (J["hand." + side] - J["armL." + side]).normalized()
        turn = fore_blob.rotation_difference(fore).to_matrix()
        bm = bmesh.new()
        fist(bm, x)
        for v in bm.verts:
            v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * FIST_SCALE
        parts["Hand." + side] = to_object("Hand." + side, bm, "hand." + side, rig)
        radii_at_wrist = sorted(wrist_r[side])
        r = radii_at_wrist[len(radii_at_wrist) // 2] if radii_at_wrist else 0.08
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=24, radius1=r * 1.2, radius2=r * 1.38, depth=0.075)
        bmesh.ops.transform(bm, matrix=Vector((0.0, 0.0, 1.0)).rotation_difference(fore).to_matrix().to_4x4(), verts=bm.verts)
        bmesh.ops.translate(bm, vec=J["hand." + side] - fore * 0.055, verts=bm.verts)
        parts["Cuff." + side] = to_object("Cuff." + side, bm, "hand." + side, rig)

    # The face shell, as the blob's: a ball fitted to the head, its front only, the face drawing projected straight on.
    head = [v.co for v in skin.data.vertices if v.co.z > FIT["neck"] + 0.05 - NECK_DROP]
    lo = Vector((min(c.x for c in head), min(c.y for c in head), min(c.z for c in head)))
    hi = Vector((max(c.x for c in head), max(c.y for c in head), max(c.z for c in head)))
    centre = (lo + hi) / 2.0
    radii = (hi - lo) / 2.0
    bm = bmesh.new()
    ball(bm, 1.0, (0, 0, 0), segments=48, rings=32)
    for v in bm.verts:
        v.co = Vector((centre.x + v.co.x * radii.x * 1.03, centre.y + v.co.y * radii.y * 1.03, centre.z + v.co.z * radii.z * 1.03))
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.y > centre.y - 0.3 * radii.y], context="VERTS")
    face_size = 1.75 * radii.x
    face_centre = centre.z + 0.04 * radii.z
    uv = bm.loops.layers.uv.verify()
    for f in bm.faces:
        for loop in f.loops:
            co = loop.vert.co
            loop[uv].uv = (0.5 + co.x / face_size, 0.5 + (co.z - face_centre) / face_size)
    parts["Face"] = to_object("Face", bm, "head", rig)

    # The skin, weighted to the skeleton by Blender's automatic weights.
    bpy.ops.object.select_all(action="DESELECT")
    skin.select_set(True)
    rig.select_set(True)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    # Softer elbows and shoulders: the automatic weights change over from one arm bone to the next in too short a stretch, which pinches
    # the arm where it bends; spread out, it bends round.
    smooth_weights(skin, [g + "." + side for side in "LR" for g in ("armU", "armL")])

    # What the game needs to fit hats, glasses, the weapon arm and the neckwear (Godot's axes: x, z, -y).
    def godot(v):
        return [round(v.x, 4), round(v.z, 4), round(-v.y, 4)]
    # And where the clothes go, in the rest pose (Blender's axes, as the skin's texture maps hold them): the shirt from the waist to the
    # neck and down the arms to the sleeve's end, the shorts from the waist to above the knee, socks up from the ankle, shoes below it.
    hips = J["hips"].z
    crotch = FIT["crotch"]
    knee = (J["shin.L"].z + J["shin.R"].z) / 2.0
    ankle = (J["foot.L"].z + J["foot.R"].z) / 2.0
    clothes = {
        "neck": round(neck - 0.015, 4),
        "waist": round(hips + 0.12 * (neck - hips), 4),
        "shorts_end": round(crotch - 0.42 * (crotch - knee), 4),
        "sock_top": round(ankle + 0.19, 4),
        "shoe_top": round(ankle + 0.03, 4),
        "sole_top": round(0.035, 4),
        "shoulder_l": [round(J["armU.L"].x, 4), round(J["armU.L"].z, 4)], "elbow_l": [round(J["armL.L"].x, 4), round(J["armL.L"].z, 4)],
        "shoulder_r": [round(J["armU.R"].x, 4), round(J["armU.R"].z, 4)], "elbow_r": [round(J["armL.R"].x, 4), round(J["armL.R"].z, 4)],
        "torso_half_width": round(abs(J["armU.R"].x) * 0.72, 4),
    }
    info = {"head_centre": godot(centre), "head_radii": [round(radii.x, 4), round(radii.z, 4), round(radii.y, 4)],
            "neck": round(neck, 4), "shoulder": godot(J["armU.R"]), "wrist": godot(J["hand.R"]), "hips": round(J["hips"].z, 4),
            "height": round(FIT["height"], 4), "clothes": clothes}
    with open(OUT.replace(".glb", ".json"), "w") as f:
        json.dump(info, f, indent=1)
    return parts


# ---- The long-limbed variant (the brawler): legs and arms stretched, the rest of the body lifted to match -------------------------------
LONG = "--long" in sys.argv
KL = 1.5      # legs, between the ankle and the hip
KA = 1.4      # arms, from the shoulder out
ANKLE_Z = 0.22
HIP_Z = 0.6
SHIFT = (HIP_Z - ANKLE_Z) * (KL - 1.0) if LONG else 0.0   # how far everything above the hip moves up
if LONG:
    OUT = OUT.replace("blob_rig.glb", "blob_rig_long.glb")


def leg_z(z):
    return ANKLE_Z + (z - ANKLE_Z) * KL if z > ANKLE_Z else z


def lengthen_limbs(rig, parts):
    """Stretches the legs and arms in place (rest pose) and raises the body, head and arms with the longer legs."""
    for side, x in (("L", -1.0), ("R", 1.0)):
        shoulder = Vector((x * 0.5, 0.0, 1.15 + SHIFT))
        wrist = Vector((x * 0.74, -0.04, 0.78 + SHIFT))
        along = (wrist - shoulder).normalized()
        for name, obj in parts.items():
            if not name.endswith("." + side):
                continue
            for v in obj.data.vertices:
                co = v.co.copy()
                if name.startswith(("Thigh", "Shin", "Knee", "Leg")):
                    co.z = leg_z(co.z)
                elif name.startswith(("Foot", "Sole", "Strap")):
                    pass
                elif name.startswith("ShortsLeg"):
                    co.z = leg_z(co.z)
                elif name.startswith(("Arm", "Sleeve", "Elbow")):
                    co.z += SHIFT
                    co += along * ((co - shoulder).dot(along) * (KA - 1.0))
                elif name.startswith(("Hand", "Cuff")):
                    co.z += SHIFT
                    co += (Vector((x * 0.74, -0.04, 0.72 + SHIFT)) - shoulder) * (KA - 1.0)
                else:
                    co.z += SHIFT
                v.co = co
    for name in ("Body", "Head", "Face", "Shorts", "Collar"):
        for v in parts[name].data.vertices:
            v.co.z += SHIFT
    # bones
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    for eb in rig.data.edit_bones:
        head = eb.head.copy()
        base = eb.name.split(".")[0]
        if base in ("thigh", "shin"):
            head.z = leg_z(head.z)
        elif base == "foot" or base == "root":
            pass
        elif base in ("armL", "hand"):
            x = -1.0 if eb.name.endswith(".L") else 1.0
            shoulder = Vector((x * 0.5, 0.0, 1.15 + SHIFT))
            head.z += SHIFT
            head = shoulder + (head - shoulder) * KA
        else:
            head.z += SHIFT
        eb.head = head
        eb.tail = head + Vector((0.0, 0.0, 0.12))
    bpy.ops.object.mode_set(mode="OBJECT")
    # the armature keeps its rest pose; meshes were edited directly


def main() -> None:
    clear()
    rig = make_armature()
    parts = build_body_meshes(rig) if BODY else build_meshes(rig)
    if LONG:
        lengthen_limbs(rig, parts)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="POSE")
    bpy.context.scene.render.fps = FPS

    # Standing ready to fight (in the manner of a plumber's bouncy stance): feet apart with the far foot forward, knees bent, weight
    # bobbing down and up twice a cycle, fists up in front of the chest with the lead fist further out, the chest leaning in a little and
    # turned with the lead shoulder, the head nodding with the bounce. The game turns the chest and head toward the camera on top.
    def idle(k):
        def f(rig):
            b = math.cos(k * 4.0 * math.pi)       # 1 at the top of a bob, -1 at the bottom
            down = 0.5 - 0.5 * b
            pose(rig, "hips", lift=-0.06 - 0.04 * down, twist=-6.0)
            pose(rig, "thigh.L", fwd=32.0 + 6.0 * down, out=15.0)
            pose(rig, "shin.L", fwd=-40.0 - 14.0 * down)
            pose(rig, "foot.L", fwd=6.0 + 6.0 * down, out=-10.0)
            # The back leg stays nearly straight (a bent back knee would cross the front leg on screen).
            pose(rig, "thigh.R", fwd=-14.0 + 3.0 * down, out=15.0)
            pose(rig, "shin.R", fwd=-10.0 - 6.0 * down)
            pose(rig, "foot.R", fwd=18.0, out=-10.0)
            pose(rig, "spine", fwd=9.0 + 4.0 * down, twist=8.0 + 2.0 * b)
            pose(rig, "head", fwd=-7.0 + 4.0 * down, twist=-4.0)
            # Lead fist (far arm) out in front, rear fist guarding the chin; both ride the bounce a beat behind.
            lag = math.cos(k * 4.0 * math.pi - 0.8)
            # (The base body's arms are thick and fully jointed: its elbows bend less, the fists held a little lower.)
            elbow = 0.72 if BODY else 1.0
            pose(rig, "armU.L", fwd=52.0 + 5.0 * lag, out=16.0)
            pose(rig, "armL.L", fwd=(92.0 - 6.0 * lag) * elbow)
            pose(rig, "hand.L", fwd=-12.0)
            pose(rig, "armU.R", fwd=34.0 + 5.0 * lag, out=22.0)
            pose(rig, "armL.R", fwd=(112.0 - 6.0 * lag) * elbow)
            pose(rig, "hand.R", fwd=-14.0)
        return f

    clip(rig, "idle", 48, {i: idle(i / 48.0) for i in range(0, 49, 4)})

    # A bouncy, swaggering walk: knees lifting, a springy bob on every step, arms swinging wide with loose bent elbows, a little lean.
    # A big, bouncy, swaggering walk (in the manner of a plumber's cheerful stride): long steps with the knee lifted high as the leg
    # comes through, a springy bob on every step, fists swinging wide and high opposite the legs, the shoulders twisting against the hips
    # and the head nodding with each step.
    WALK_THIGH = [(0.0, 44.0), (0.12, 26.0), (0.32, -12.0), (0.48, -40.0), (0.62, -12.0), (0.78, 56.0), (0.9, 52.0)]
    WALK_SHIN = [(0.0, -6.0), (0.12, -30.0), (0.32, -10.0), (0.48, -22.0), (0.62, -90.0), (0.78, -80.0), (0.9, -18.0)]
    WALK_FOOT = [(0.0, 18.0), (0.12, -2.0), (0.32, -12.0), (0.48, -34.0), (0.62, -22.0), (0.78, 24.0), (0.9, 22.0)]

    def walk(phase):
        def f(rig):
            for side, offset in (("L", 0.0), ("R", 0.5)):
                leg = phase + offset
                pose(rig, "thigh." + side, fwd=periodic(WALK_THIGH, leg), out=5.0)
                pose(rig, "shin." + side, fwd=periodic(WALK_SHIN, leg))
                pose(rig, "foot." + side, fwd=periodic(WALK_FOOT, leg))
                swing = math.cos((leg + 0.5) * 2.0 * math.pi)
                forward = max(0.0, swing)
                pose(rig, "armU." + side, fwd=-60.0 * swing + 12.0, out=20.0 + 10.0 * forward)
                pose(rig, "armL." + side, fwd=50.0 + 40.0 * forward)
                pose(rig, "hand." + side, fwd=-10.0)
            step = math.cos((phase - 0.4) * 4.0 * math.pi)
            pose(rig, "hips", lift=0.08 * step - 0.045, twist=-13.0 * math.sin(phase * 2.0 * math.pi))
            pose(rig, "spine", fwd=8.0 - 5.0 * step, twist=17.0 * math.sin(phase * 2.0 * math.pi))
            pose(rig, "head", fwd=-4.0 + 8.0 * step, twist=-9.0 * math.sin(phase * 2.0 * math.pi))
        return f

    clip(rig, "walk", 40, {i: walk(i / 40.0) for i in range(0, 41, 2)})
    # The run: a key every other frame so the snappy gait survives interpolation.
    clip(rig, "run", 20, {i: run_pose(i / 20.0) for i in range(0, 21, 2)})
    # The initial dash: the same stride, launched lower and further forward, arms flung harder.
    clip(rig, "dash", 16, {i: run_pose(i / 16.0, lean=32.0, stride=1.1, arms=1.15, bounce=0.06) for i in range(0, 17, 2)})

    # A skid: braking out of a run to turn around. Leaning back hard on a planted front heel, the back knee bent low, arms flung back and
    # up for balance; a little wobble so it is alive.
    def skid(k):
        def f(rig):
            w = math.sin(k * 2.0 * math.pi)
            pose(rig, "hips", lift=-0.12)
            pose(rig, "spine", fwd=-22.0 + 3.0 * w, twist=6.0 * w)
            pose(rig, "head", fwd=10.0)
            pose(rig, "thigh.L", fwd=52.0, out=6.0)
            pose(rig, "shin.L", fwd=-6.0)
            pose(rig, "foot.L", fwd=34.0)
            pose(rig, "thigh.R", fwd=-8.0, out=6.0)
            pose(rig, "shin.R", fwd=-78.0)
            for side, sign in (("L", 1.0), ("R", -1.0)):
                pose(rig, "armU." + side, fwd=-70.0 + 14.0 * w * sign, out=60.0)
                pose(rig, "armL." + side, fwd=20.0)
        return f

    clip(rig, "skid", 12, {0: skid(0.0), 3: skid(0.25), 6: skid(0.5), 9: skid(0.75), 12: skid(1.0)})

    # The jump (rising): pushing off with both legs straight, then the front knee driven up high and the back leg trailing down and
    # behind, the lead fist thrown up and the other arm swung back (the reference game's jumping pose), held as it rises.
    def jump(k):
        def f(rig):
            tuck = k
            # The near leg (the right, nearest the game's camera on the turned body) drives its knee up in front; the far leg hangs long
            # below it, toes pointed, so both read: one high, one low.
            pose(rig, "thigh.R", fwd=6.0 + 66.0 * tuck, out=8.0)
            pose(rig, "shin.R", fwd=-6.0 - 84.0 * tuck)
            pose(rig, "foot.R", fwd=20.0 * tuck)
            pose(rig, "thigh.L", fwd=-6.0 - 10.0 * tuck, out=14.0)
            pose(rig, "shin.L", fwd=-8.0 - 6.0 * tuck)
            pose(rig, "foot.L", fwd=-36.0 * tuck)
            pose(rig, "armU.L", fwd=40.0 + 56.0 * tuck, out=24.0)
            pose(rig, "armL.L", fwd=40.0)
            pose(rig, "armU.R", fwd=-20.0 - 24.0 * tuck, out=42.0)
            pose(rig, "armL.R", fwd=30.0)
            pose(rig, "spine", fwd=4.0 - 10.0 * tuck, twist=6.0 * tuck)
            pose(rig, "head", fwd=-6.0 * tuck)
        return f

    clip(rig, "jump", 14, {0: jump(0.0), 5: jump(1.0), 14: jump(0.85)})

    # Falling: legs apart (front knee bent forward, back leg hanging behind), arms raised out to the sides and paddling a little.
    def fall(k):
        def f(rig):
            w = math.sin(k * 2 * math.pi)
            pose(rig, "thigh.R", fwd=52.0 + 6.0 * w, out=8.0)
            pose(rig, "shin.R", fwd=-66.0 - 8.0 * w)
            pose(rig, "thigh.L", fwd=-12.0 - 6.0 * w, out=14.0)
            pose(rig, "shin.L", fwd=-18.0 + 6.0 * w)
            pose(rig, "foot.L", fwd=-24.0)
            for side, sign in (("L", 1.0), ("R", -1.0)):
                pose(rig, "armU." + side, fwd=6.0 + 8.0 * w * sign, out=72.0 + 10.0 * w * sign)
                pose(rig, "armL." + side, fwd=30.0)
            pose(rig, "spine", fwd=4.0, twist=4.0 * w)
            pose(rig, "head", fwd=-4.0)
        return f

    clip(rig, "fall", 30, {0: fall(0.0), 8: fall(0.25), 15: fall(0.5), 23: fall(0.75), 30: fall(1.0)})

    # The midair jump: curled into a ball (knees to the chest, fists in) for the front flip the game spins it through, then opening out
    # into the rising pose.
    def ball_pose(rig):
        for side in "LR":
            pose(rig, "thigh." + side, fwd=86.0, out=12.0)
            pose(rig, "shin." + side, fwd=-112.0)
            pose(rig, "armU." + side, fwd=60.0, out=24.0)
            pose(rig, "armL." + side, fwd=96.0)
        pose(rig, "spine", fwd=26.0)
        pose(rig, "head", fwd=8.0)

    clip(rig, "air_jump", 26, {0: jump(0.4), 4: ball_pose, 16: ball_pose, 26: jump(1.0)})

    def crouch(rig):
        pose(rig, "hips", lift=-0.2)
        for side in "LR":
            pose(rig, "thigh." + side, fwd=62.0, out=6.0)
            pose(rig, "shin." + side, fwd=-96.0)
            pose(rig, "foot." + side, fwd=30.0)
            pose(rig, "armU." + side, fwd=26.0, out=14.0)
            pose(rig, "armL." + side, fwd=30.0)
        pose(rig, "spine", fwd=14.0)
        pose(rig, "head", fwd=-8.0)

    clip(rig, "crouch", 8, {0: crouch, 8: crouch})

    def shield(rig):
        pose(rig, "hips", lift=-0.08)
        for side in "LR":
            pose(rig, "thigh." + side, fwd=26.0, out=8.0)
            pose(rig, "shin." + side, fwd=-40.0)
            pose(rig, "armU." + side, fwd=62.0, out=24.0)
            pose(rig, "armL." + side, fwd=96.0)
        pose(rig, "spine", fwd=8.0)

    clip(rig, "shield", 8, {0: shield, 8: shield})

    def hurt(rig):
        for side in "LR":
            pose(rig, "armU." + side, fwd=-20.0, out=78.0)
            pose(rig, "armL." + side, fwd=-10.0)
            pose(rig, "thigh." + side, fwd=-10.0, out=14.0)
            pose(rig, "shin." + side, fwd=-26.0)
        pose(rig, "spine", fwd=-18.0, twist=-8.0)
        pose(rig, "head", fwd=-14.0)

    clip(rig, "hurt", 8, {0: hurt, 8: hurt})

    # Attacks are played by progress (0 to 1 over the move), so the swing lands on the move's active frames: wind up (to 0.35),
    # strike (to 0.55), hold, recover.
    def swing(strike, wind, out):
        def f_factory(arm_fwd, fore, spine_fwd, twist, step):
            def f(rig):
                pose(rig, "armU.R", fwd=arm_fwd, out=out)
                pose(rig, "armL.R", fwd=fore)
                pose(rig, "armU.L", fwd=-arm_fwd * 0.35, out=14.0)
                pose(rig, "armL.L", fwd=30.0)
                pose(rig, "spine", fwd=spine_fwd, twist=twist)
                pose(rig, "head", fwd=-spine_fwd * 0.5, twist=-twist * 0.5)
                pose(rig, "thigh.L", fwd=step, out=4.0)
                pose(rig, "thigh.R", fwd=-step * 0.6, out=4.0)
                pose(rig, "shin.L", fwd=-8.0 - max(0.0, -step) * 0.4)
                pose(rig, "shin.R", fwd=-8.0 - max(0.0, step) * 0.4)
            return f
        return {
            0: f_factory(10.0, 40.0, 2.0, 0.0, 0.0),
            21: amped(f_factory(wind, 70.0, -8.0, -22.0, -14.0), 1.12),
            33: f_factory(strike, 8.0, 14.0, 26.0, 22.0),
            38: amped(f_factory(strike, 8.0, 14.0, 26.0, 22.0), 1.12),
            45: f_factory(strike - 10.0, 12.0, 12.0, 20.0, 20.0),
            60: f_factory(10.0, 40.0, 2.0, 0.0, 0.0),
        }

    clip(rig, "attack_swing", 60, swing(strike=105.0, wind=-70.0, out=18.0))
    clip(rig, "attack_low", 60, swing(strike=75.0, wind=-50.0, out=30.0))

    def kick(phase):
        def f(rig):
            pose(rig, "hips", lift=-0.02)
            if phase == 0:
                pose(rig, "thigh.R", fwd=-24.0)
                pose(rig, "shin.R", fwd=-60.0)
                pose(rig, "spine", fwd=-4.0)
            elif phase == 1:
                pose(rig, "thigh.R", fwd=88.0)
                pose(rig, "shin.R", fwd=-6.0)
                pose(rig, "foot.R", fwd=-20.0)
                pose(rig, "spine", fwd=-14.0)
                pose(rig, "thigh.L", fwd=-4.0)
            else:
                pose(rig, "thigh.R", fwd=30.0)
                pose(rig, "shin.R", fwd=-30.0)
            for side, sign in (("L", 1.0), ("R", -1.0)):
                pose(rig, "armU." + side, fwd=-30.0 * sign if phase == 1 else 10.0, out=30.0)
                pose(rig, "armL." + side, fwd=30.0)
        return f

    clip(rig, "attack_kick", 60, {0: kick(2), 21: amped(kick(0), 1.12), 33: kick(1), 38: amped(kick(1), 1.1), 45: kick(1), 60: kick(2)})

    # ---- Aerials, smashes and the rest. The weapon arm is aimed by the game (it follows the blade), so these set the body: torso, head,
    # legs and the other arm. Move clips share one timeline: wind-up to frame 21, strike at 33, hold to 45, recover by 60.
    def rest_arms(rig, fwd=8.0, out=14.0):
        for side in "LR":
            pose(rig, "armU." + side, fwd=fwd, out=out)
            pose(rig, "armL." + side, fwd=30.0)

    def stance(rig, lead=0.0, back=0.0):
        pose(rig, "thigh.L", fwd=lead, out=5.0)
        pose(rig, "shin.L", fwd=-8.0 - max(0.0, -lead) * 0.5)
        pose(rig, "thigh.R", fwd=back, out=5.0)
        pose(rig, "shin.R", fwd=-8.0 - max(0.0, -back) * 0.5)

    def move(name, wind, strike, hold):
        # Wind-up pushed a little past its pose (anticipation), the strike, an overshoot past it (follow-through), then the settle.
        clip(rig, name, 60, {0: hold(0), 21: amped(wind, 1.12), 33: strike, 38: amped(strike, 1.12), 45: hold(1), 60: hold(0)})

    # Forward air: the reference pose is a mid-air crouch, knees pulled up high, the attacking arm reaching up and forward and then brought
    # down through the sweep in front while the torso curls over it. Lean back with the arm cocked, then curl forward with the knees tucked.
    def fair_wind(rig):
        rest_arms(rig, -20.0)
        pose(rig, "hips", lift=0.02)
        pose(rig, "spine", fwd=-18.0, twist=-20.0)
        pose(rig, "head", fwd=6.0)
        stance(rig, lead=40.0, back=46.0)
        pose(rig, "shin.L", fwd=-70.0)
        pose(rig, "shin.R", fwd=-80.0)

    def fair_strike(rig):
        rest_arms(rig, 30.0)
        pose(rig, "hips", lift=-0.04)
        pose(rig, "spine", fwd=34.0, twist=24.0)
        pose(rig, "head", fwd=-14.0)
        stance(rig, lead=66.0, back=58.0)
        pose(rig, "shin.L", fwd=-96.0)
        pose(rig, "shin.R", fwd=-104.0)

    def fair_hold(k):
        def f(rig):
            rest_arms(rig, 14.0 * k)
            pose(rig, "spine", fwd=16.0 * k, twist=8.0 * k)
            stance(rig, lead=44.0 * k, back=40.0 * k)
            pose(rig, "shin.L", fwd=-8.0 - 70.0 * k)
            pose(rig, "shin.R", fwd=-8.0 - 76.0 * k)
        return f

    move("attack_fair", fair_wind, fair_strike, fair_hold)

    # Back air: the body turns away and the sweep goes behind.
    def bair_wind(rig):
        rest_arms(rig, 20.0)
        pose(rig, "spine", fwd=12.0, twist=24.0)
        pose(rig, "head", fwd=-4.0, twist=-14.0)
        stance(rig, lead=20.0, back=-10.0)

    # Back air: the body turns away; the front knee comes up tucked and the back leg trails down and behind, as the swing goes behind.
    def bair_strike(rig):
        rest_arms(rig, -30.0)
        pose(rig, "hips", lift=-0.04)
        pose(rig, "spine", fwd=-18.0, twist=-40.0)
        pose(rig, "head", fwd=8.0, twist=26.0)
        stance(rig, lead=62.0, back=-30.0)
        pose(rig, "shin.L", fwd=-96.0)
        pose(rig, "shin.R", fwd=-24.0)

    def bair_hold(k):
        def f(rig):
            rest_arms(rig, -12.0 * k)
            pose(rig, "spine", fwd=-8.0 * k, twist=-14.0 * k)
            stance(rig, lead=-6.0 * k, back=20.0 * k)
        return f

    move("attack_bair", bair_wind, bair_strike, bair_hold)

    # Neutral air: a spin, legs flung out.
    def nair_wind(rig):
        rest_arms(rig, 10.0, 40.0)
        pose(rig, "spine", twist=34.0)
        for side in "LR":
            pose(rig, "thigh." + side, fwd=14.0, out=12.0)
            pose(rig, "shin." + side, fwd=-30.0)

    def nair_strike(rig):
        rest_arms(rig, 10.0, 60.0)
        pose(rig, "spine", twist=-40.0, fwd=6.0)
        for side in "LR":
            pose(rig, "thigh." + side, fwd=18.0, out=34.0)
            pose(rig, "shin." + side, fwd=-12.0)

    def nair_hold(k):
        def f(rig):
            rest_arms(rig, 10.0, 24.0 * k)
            pose(rig, "spine", twist=-12.0 * k)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=10.0 * k, out=12.0 * k)
                pose(rig, "shin." + side, fwd=-20.0 * k)
        return f

    move("attack_nair", nair_wind, nair_strike, nair_hold)

    # Up air: lean back, look up, legs up.
    def uair_wind(rig):
        rest_arms(rig, 0.0)
        pose(rig, "spine", fwd=8.0)
        stance(rig, lead=14.0, back=14.0)

    def uair_strike(rig):
        rest_arms(rig, -40.0, 24.0)
        pose(rig, "spine", fwd=-26.0, twist=14.0)
        pose(rig, "head", fwd=-20.0)
        stance(rig, lead=36.0, back=48.0)

    def uair_hold(k):
        def f(rig):
            rest_arms(rig, -14.0 * k)
            pose(rig, "spine", fwd=-10.0 * k)
            pose(rig, "head", fwd=-8.0 * k)
            stance(rig, lead=20.0 * k, back=20.0 * k)
        return f

    move("attack_uair", uair_wind, uair_strike, uair_hold)

    # Down air: curl up, then stab downward.
    def dair_wind(rig):
        rest_arms(rig, 20.0, 20.0)
        pose(rig, "hips", lift=0.04)
        pose(rig, "spine", fwd=-8.0)
        stance(rig, lead=60.0, back=60.0)
        for side in "LR":
            pose(rig, "shin." + side, fwd=-80.0)

    def dair_strike(rig):
        rest_arms(rig, 50.0, 10.0)
        pose(rig, "spine", fwd=22.0)
        pose(rig, "head", fwd=-14.0)
        stance(rig, lead=-6.0, back=4.0)

    def dair_hold(k):
        def f(rig):
            rest_arms(rig, 20.0 * k)
            pose(rig, "spine", fwd=10.0 * k)
            stance(rig, lead=-2.0 * k, back=2.0 * k)
        return f

    move("attack_dair", dair_wind, dair_strike, dair_hold)

    # Smashes: plant the legs, wind the torso back, and throw it through.
    def smash_wind(rig):
        rest_arms(rig, -30.0, 20.0)
        pose(rig, "hips", lift=-0.12)
        pose(rig, "spine", fwd=-12.0, twist=-34.0)
        pose(rig, "head", twist=18.0)
        stance(rig, lead=36.0, back=-30.0)
        pose(rig, "shin.L", fwd=-50.0)

    def smash_strike(rig):
        rest_arms(rig, 40.0, 12.0)
        pose(rig, "hips", lift=-0.08)
        pose(rig, "spine", fwd=28.0, twist=36.0)
        pose(rig, "head", fwd=-10.0, twist=-18.0)
        stance(rig, lead=48.0, back=-40.0)
        pose(rig, "shin.L", fwd=-30.0)

    def smash_hold(k):
        def f(rig):
            rest_arms(rig, 14.0 * k)
            pose(rig, "hips", lift=-0.04 * k)
            pose(rig, "spine", fwd=12.0 * k, twist=14.0 * k)
            stance(rig, lead=26.0 * k, back=-20.0 * k)
        return f

    move("attack_smash", smash_wind, smash_strike, smash_hold)

    # Jab: a quick straight strike. A small step, the shoulders turn back and snap forward; the move is short so the pose is held at the end.
    def jab_wind(rig):
        rest_arms(rig, -10.0, 16.0)
        pose(rig, "spine", fwd=-4.0, twist=-26.0)
        pose(rig, "head", twist=14.0)
        stance(rig, lead=14.0, back=-14.0)

    def jab_strike(rig):
        rest_arms(rig, 26.0, 10.0)
        pose(rig, "hips", lift=-0.03)
        pose(rig, "spine", fwd=10.0, twist=30.0)
        pose(rig, "head", fwd=-4.0, twist=-14.0)
        stance(rig, lead=30.0, back=-26.0)

    def jab_hold(k):
        def f(rig):
            rest_arms(rig, 8.0 * k)
            pose(rig, "spine", fwd=4.0 * k, twist=10.0 * k)
            stance(rig, lead=10.0 * k, back=-8.0 * k)
        return f

    move("attack_jab", jab_wind, jab_strike, jab_hold)

    # Lunge (dash attack, side special): crouch back, then the whole body stretches forward.
    def lunge_wind(rig):
        rest_arms(rig, -24.0, 22.0)
        pose(rig, "hips", lift=-0.14)
        pose(rig, "spine", fwd=-14.0)
        pose(rig, "head", fwd=8.0)
        stance(rig, lead=20.0, back=-36.0)
        pose(rig, "shin.R", fwd=-60.0)

    def lunge_strike(rig):
        rest_arms(rig, 50.0, 8.0)
        pose(rig, "hips", lift=-0.1)
        pose(rig, "spine", fwd=40.0)
        pose(rig, "head", fwd=-24.0)
        stance(rig, lead=62.0, back=-52.0)
        pose(rig, "shin.L", fwd=-16.0)

    def lunge_hold(k):
        def f(rig):
            rest_arms(rig, 16.0 * k)
            pose(rig, "hips", lift=-0.05 * k)
            pose(rig, "spine", fwd=16.0 * k)
            stance(rig, lead=30.0 * k, back=-24.0 * k)
        return f

    move("attack_lunge", lunge_wind, lunge_strike, lunge_hold)

    # ---- One clip per move (third pass) ----------------------------------------------------------------------------------------------
    # Each move gets its own body: where the weight goes, how the torso turns and what the free arm and legs do. The weapon arm is still
    # aimed by the game at the move's live hitbox, and for the claws fighter's kicks the kicking leg is too, so the swing and the kick
    # always land where the hit is; these clips carry the rest of the body through the move. Poses are keyed on the shared move timeline
    # (wind-up by frame 21, strike at 33, hold to 45, recover by 60), which the game stretches to each move's own frame data.
    def body(spec):
        """A pose from a short spec: {"arms": (fwd, out), "hips": lift, "spine": (fwd, twist), "head": (fwd, twist),
        "stance": (lead, back), bone: (fwd, out)...}."""
        def f(rig):
            arms = spec.get("arms", (8.0, 14.0))
            rest_arms(rig, arms[0], arms[1])
            if "hips" in spec:
                pose(rig, "hips", lift=spec["hips"])
            if "spine" in spec:
                pose(rig, "spine", fwd=spec["spine"][0], twist=spec["spine"][1])
            if "head" in spec:
                pose(rig, "head", fwd=spec["head"][0], twist=spec["head"][1])
            if "stance" in spec:
                stance(rig, spec["stance"][0], spec["stance"][1])
            for bone, v in spec.items():
                if "." in bone:
                    pose(rig, bone, fwd=v[0], out=v[1] if len(v) > 1 else 0.0)
        return f

    def held(spec):
        def make(k):
            scaled = {}
            for key, v in spec.items():
                if isinstance(v, tuple):
                    scaled[key] = tuple(x * k for x in v)
                else:
                    scaled[key] = v * k
            return body(scaled)
        return make

    def move_of(name, wind, strike, rest):
        move(name, body(wind), body(strike), held(rest))

    # Sword fighter.
    # Forward tilt: a rising diagonal cut. Crouch into it turned away, then rise and turn into the blade as it climbs.
    move_of("sword_ftilt",
            {"hips": -0.08, "spine": (14.0, -24.0), "head": (6.0, 10.0), "stance": (22.0, -22.0), "arms": (-10.0, 20.0)},
            {"hips": -0.02, "spine": (-10.0, 26.0), "head": (-8.0, -12.0), "stance": (34.0, -26.0), "arms": (24.0, 12.0)},
            {"spine": (4.0, 10.0), "stance": (16.0, -12.0)})
    # Up tilt: the blade sweeps overhead from front to back; the body arches back under it.
    move_of("sword_utilt",
            {"hips": -0.06, "spine": (16.0, 8.0), "head": (8.0, 0.0), "stance": (14.0, -10.0)},
            {"hips": 0.02, "spine": (-26.0, -6.0), "head": (-18.0, 0.0), "stance": (16.0, -18.0), "arms": (-20.0, 24.0)},
            {"spine": (-8.0, 0.0), "stance": (8.0, -8.0)})
    # Down tilt: a low crouching thrust.
    move_of("sword_dtilt",
            {"hips": -0.2, "spine": (20.0, -10.0), "head": (-12.0, 0.0), "stance": (64.0, 30.0), "shin.L": (-96.0,), "shin.R": (-100.0,)},
            {"hips": -0.22, "spine": (26.0, 12.0), "head": (-20.0, 0.0), "stance": (78.0, 18.0), "shin.L": (-60.0,), "shin.R": (-104.0,)},
            {"hips": -0.12, "spine": (20.0, 0.0), "stance": (40.0, 16.0)})
    # Forward smash: the blade raised high behind, then brought down in front in one big committed swing.
    move_of("sword_fsmash",
            {"hips": -0.06, "spine": (-18.0, -42.0), "head": (10.0, 20.0), "stance": (28.0, -34.0), "arms": (-30.0, 22.0)},
            {"hips": -0.14, "spine": (28.0, 30.0), "head": (-8.0, -16.0), "stance": (58.0, -46.0), "shin.L": (-34.0,), "arms": (40.0, 10.0)},
            {"hips": -0.06, "spine": (16.0, 14.0), "stance": (30.0, -24.0)})
    # Up smash: crouch, then spring up stretched tall as the blade goes straight up.
    move_of("sword_usmash",
            {"hips": -0.16, "spine": (18.0, 0.0), "head": (8.0, 0.0), "stance": (34.0, 26.0), "shin.L": (-60.0,), "shin.R": (-60.0,)},
            {"hips": 0.1, "spine": (-12.0, 0.0), "head": (-22.0, 0.0), "stance": (-4.0, -8.0), "arms": (-24.0, 30.0)},
            {"spine": (-4.0, 0.0), "stance": (6.0, 4.0)})
    # Down smash: a low sweep in front, then behind, in a wide crouch.
    move_of("sword_dsmash",
            {"hips": -0.12, "spine": (22.0, -20.0), "stance": (40.0, -36.0)},
            {"hips": -0.2, "spine": (34.0, 34.0), "head": (-12.0, -10.0), "stance": (56.0, -56.0), "thigh.L": (56.0, 22.0), "thigh.R": (-56.0, 22.0)},
            {"hips": -0.1, "spine": (16.0, 10.0), "stance": (30.0, -30.0)})

    # Down air: the blade sweeps a crescent underneath, front to back (the game aims the arm along it). Gather with the knees tucked and
    # the sword raised ahead, then open the legs wide (front knee up, back leg kicked out behind) with the torso upright and the head
    # looking down past the swing, the free arm flung out behind for balance.
    move_of("sword_dair",
            {"spine": (2.0, 0.0), "head": (6.0, 0.0), "stance": (44.0, 30.0), "shin.L": (-90.0,), "shin.R": (-80.0,), "arms": (16.0, 26.0)},
            {"spine": (-8.0, -12.0), "head": (20.0, 0.0), "stance": (46.0, -40.0), "shin.L": (-84.0,), "shin.R": (-22.0,),
             "arms": (-26.0, 52.0)},
            {"spine": (4.0, -4.0), "head": (8.0, 0.0), "stance": (34.0, 6.0)})

    # Claws fighter. The kicking leg is aimed by the game; these set the torso, arms and the other leg.
    # Neutral air: a spinning kick, tucked, then opened out and turning.
    # Neutral air: a split kick held out (the game aims both legs, one ahead and one behind). Gather tucked, then open the hips wide,
    # the torso upright and leaning back a touch over the split, arms flung out for balance; it holds through the long late hit.
    move_of("kick_nair",
            {"spine": (18.0, 0.0), "head": (6.0, 0.0), "stance": (60.0, 60.0), "shin.L": (-100.0,), "shin.R": (-100.0,), "arms": (30.0, 26.0)},
            {"spine": (-8.0, 0.0), "head": (-6.0, 0.0), "arms": (-12.0, 70.0)},
            {"spine": (-4.0, 0.0), "arms": (-6.0, 50.0)})
    # Back air: lean forward and drive the heel back.
    move_of("kick_bair",
            {"spine": (24.0, 10.0), "head": (10.0, 20.0), "stance": (60.0, 50.0), "shin.L": (-90.0,), "shin.R": (-90.0,), "arms": (40.0, 20.0)},
            {"spine": (32.0, -24.0), "head": (-20.0, 34.0), "stance": (50.0, -20.0), "shin.L": (-80.0,), "arms": (60.0, 30.0)},
            {"spine": (18.0, 0.0), "stance": (30.0, 0.0)})
    # Up air: a flip kick, arching back as the leg goes up.
    move_of("kick_uair",
            {"spine": (24.0, 0.0), "stance": (60.0, 60.0), "shin.L": (-100.0,), "shin.R": (-100.0,), "arms": (20.0, 20.0)},
            {"spine": (-38.0, 0.0), "head": (-30.0, 0.0), "stance": (10.0, 30.0), "arms": (-30.0, 50.0)},
            {"spine": (-10.0, 0.0), "stance": (20.0, 20.0)})
    # Down air: knees up, then a stomp straight down.
    move_of("kick_dair",
            {"spine": (10.0, 0.0), "stance": (72.0, 72.0), "shin.L": (-100.0,), "shin.R": (-100.0,), "arms": (-10.0, 40.0)},
            {"spine": (14.0, 0.0), "head": (-16.0, 0.0), "stance": (-4.0, 30.0), "shin.R": (-70.0,), "arms": (-30.0, 55.0)},
            {"spine": (6.0, 0.0), "stance": (20.0, 30.0)})
    # Up tilt: an overhead kick from behind (the body leans forward as the leg arcs over).
    # Up tilt: a high kick straight up. Dip and gather, then lean back on the standing leg as the kick goes overhead, arms thrown out
    # behind for balance and the head tipped back to watch the foot (the game bends the body further back the higher the kick reaches).
    move_of("kick_up",
            {"hips": -0.08, "spine": (14.0, 0.0), "head": (4.0, 0.0), "stance": (8.0, 22.0), "shin.R": (-40.0,), "arms": (20.0, 20.0)},
            {"hips": -0.04, "spine": (-14.0, 0.0), "head": (-16.0, 0.0), "stance": (0.0, 14.0), "shin.R": (-22.0,),
             "arms": (-34.0, 58.0)},
            {"spine": (-4.0, 0.0), "stance": (4.0, 8.0)})
    # Down tilt: a low sweep from a crouch.
    move_of("kick_low",
            {"hips": -0.2, "spine": (26.0, -14.0), "stance": (60.0, 40.0), "shin.L": (-100.0,), "shin.R": (-100.0,), "arms": (30.0, 30.0)},
            {"hips": -0.24, "spine": (34.0, 20.0), "head": (-12.0, 0.0), "stance": (70.0, 50.0), "shin.R": (-110.0,), "arms": (50.0, 40.0)},
            {"hips": -0.12, "spine": (16.0, 0.0), "stance": (40.0, 30.0)})
    # Dash attack: a flying kick.
    move_of("kick_dash",
            {"hips": -0.04, "spine": (20.0, 0.0), "stance": (30.0, -30.0), "arms": (30.0, 20.0)},
            {"hips": 0.08, "spine": (-14.0, 0.0), "head": (-10.0, 0.0), "stance": (10.0, -50.0), "shin.R": (-70.0,), "arms": (-20.0, 40.0)},
            {"spine": (4.0, 0.0), "stance": (20.0, -20.0)})
    # Blaster: draw and fire, arm out level, braced.
    move_of("blaster",
            {"spine": (6.0, -20.0), "head": (0.0, 14.0), "stance": (20.0, -20.0)},
            {"hips": -0.04, "spine": (-4.0, 30.0), "head": (0.0, -20.0), "stance": (30.0, -26.0), "armU.R": (88.0, 6.0), "armL.R": (4.0,)},
            {"spine": (0.0, 14.0), "stance": (16.0, -12.0)})

    # ---- Victory poses (looping, for the results screen) -------------------------------------------------------------------------------
    # Weapon raised high, the other hand on the hip, chest out; a slow proud breath.
    def raised(breath):
        return body({"arms": (8.0, 14.0), "hips": 0.02 * breath, "spine": (-10.0 - 4.0 * breath, 0.0), "head": (-12.0, 8.0),
                     "stance": (12.0, -12.0), "armU.R": (168.0, 12.0), "armL.R": (8.0,), "armU.L": (-14.0, 42.0), "armL.L": (84.0,)})
    clip(rig, "victory_a", 60, {0: raised(0.0), 30: raised(1.0), 60: raised(0.0)})

    # A fist pump: the punch goes up twice a second, knees dipping with it; the other arm braced.
    def pump(up):
        return body({"arms": (8.0, 14.0), "hips": -0.06 * up, "spine": (-6.0 * up, 14.0 * up), "head": (-14.0 * up, 0.0),
                     "stance": (16.0, -14.0), "armU.R": (60.0 + 95.0 * up, 18.0), "armL.R": (100.0 - 85.0 * up,),
                     "armU.L": (20.0, 30.0), "armL.L": (90.0,)})
    clip(rig, "victory_b", 60, {0: pump(0.0), 15: pump(1.0), 30: pump(0.0), 45: pump(1.0), 60: pump(0.0)})

    # A cheering hop: both arms thrown up, a little jump with the knees tucked at the top.
    def cheer(k):
        return body({"arms": (150.0 + 14.0 * k, 26.0), "hips": 0.28 * k, "spine": (-8.0, 0.0), "head": (-16.0, 0.0),
                     "stance": (10.0 + 40.0 * k, 10.0 + 40.0 * k), "shin.L": (-10.0 - 60.0 * k,), "shin.R": (-10.0 - 60.0 * k,),
                     "armL.L": (10.0,), "armL.R": (10.0,)})
    clip(rig, "victory_c", 60, {0: cheer(0.0), 12: cheer(0.6), 22: cheer(1.0), 34: cheer(0.4), 44: cheer(0.0), 60: cheer(0.0)})

    # Grabs and throws.
    def grab_pose(rig):
        pose(rig, "hips", lift=-0.06)
        pose(rig, "spine", fwd=12.0)
        pose(rig, "head", fwd=-6.0)
        for side in "LR":
            pose(rig, "armU." + side, fwd=82.0, out=10.0)
            pose(rig, "armL." + side, fwd=14.0)
            pose(rig, "thigh." + side, fwd=22.0)
            pose(rig, "shin." + side, fwd=-30.0)

    clip(rig, "grab", 8, {0: grab_pose, 8: grab_pose})

    def throw_wind(rig):
        grab_pose(rig)
        pose(rig, "spine", fwd=-4.0, twist=-26.0)

    def throw_strike(rig):
        grab_pose(rig)
        pose(rig, "spine", fwd=22.0, twist=30.0)
        pose(rig, "head", fwd=-12.0)

    clip(rig, "throw", 60, {0: grab_pose, 21: amped(throw_wind, 1.12), 33: throw_strike, 38: amped(throw_strike, 1.1), 45: throw_strike,
                             60: grab_pose})

    # Rolls and dodges: curled up. A knockdown: flat and limp. A ledge hang: arms up, legs dangling.
    def roll_pose(rig):
        pose(rig, "hips", lift=-0.22)
        pose(rig, "spine", fwd=46.0)
        pose(rig, "head", fwd=-18.0)
        for side in "LR":
            pose(rig, "thigh." + side, fwd=74.0, out=6.0)
            pose(rig, "shin." + side, fwd=-112.0)
            pose(rig, "armU." + side, fwd=42.0, out=8.0)
            pose(rig, "armL." + side, fwd=88.0)

    clip(rig, "roll", 8, {0: roll_pose, 8: roll_pose})

    def down_pose(rig):
        for side in "LR":
            pose(rig, "armU." + side, fwd=-12.0, out=48.0)
            pose(rig, "armL." + side, fwd=6.0)
            pose(rig, "thigh." + side, fwd=-6.0, out=16.0)
            pose(rig, "shin." + side, fwd=-8.0)
        pose(rig, "head", fwd=-8.0)

    clip(rig, "knockdown", 8, {0: down_pose, 8: down_pose})

    def ledge_pose(k):
        def f(rig):
            pose(rig, "hips", lift=-0.1)
            pose(rig, "spine", fwd=-4.0 + 2.0 * k)
            for side in "LR":
                pose(rig, "armU." + side, fwd=-168.0, out=8.0)
                pose(rig, "armL." + side, fwd=-6.0)
                pose(rig, "thigh." + side, fwd=-10.0 + 6.0 * k * (1 if side == "L" else -1), out=6.0)
                pose(rig, "shin." + side, fwd=-20.0)
        return f

    clip(rig, "ledge", 60, {0: ledge_pose(0.0), 30: ledge_pose(1.0), 60: ledge_pose(0.0)})

    # Climbing up from a ledge (played by progress over the get-up): hanging, then the arms push down on the edge as one knee comes up onto
    # it, a crouch on the edge, and standing.
    def climb(k):
        def f(rig):
            if k == 0:
                ledge_pose(0.0)(rig)
                return
            if k == 1:
                for side in "LR":
                    pose(rig, "armU." + side, fwd=-40.0, out=30.0)
                    pose(rig, "armL." + side, fwd=60.0)
                pose(rig, "spine", fwd=34.0)
                pose(rig, "head", fwd=-16.0)
                pose(rig, "thigh.L", fwd=96.0, out=10.0)
                pose(rig, "shin.L", fwd=-110.0)
                pose(rig, "thigh.R", fwd=-20.0, out=8.0)
                pose(rig, "shin.R", fwd=-30.0)
                return
            if k == 2:
                pose(rig, "hips", lift=-0.18)
                for side in "LR":
                    pose(rig, "thigh." + side, fwd=66.0, out=8.0)
                    pose(rig, "shin." + side, fwd=-100.0)
                    pose(rig, "armU." + side, fwd=24.0, out=20.0)
                    pose(rig, "armL." + side, fwd=40.0)
                pose(rig, "spine", fwd=24.0)
                pose(rig, "head", fwd=-12.0)
                return
            for side in "LR":
                pose(rig, "armU." + side, fwd=8.0, out=14.0)
                pose(rig, "armL." + side, fwd=30.0)
        return f

    clip(rig, "ledge_climb", 30, {0: climb(0), 10: climb(1), 20: climb(2), 30: climb(3)})

    bpy.ops.object.mode_set(mode="OBJECT")
    reset(rig)
    bpy.ops.object.select_all(action="SELECT")
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=OUT,
        export_format="GLB",
        export_materials="NONE",
        export_yup=True,
        export_animations=True,
        export_animation_mode="ACTIONS",
        export_skins=True,
        export_apply=False,
    )
    print("wrote", OUT, "clips:", [a.name for a in bpy.data.actions])


main()
