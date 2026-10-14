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
from mathutils import Matrix, Vector, bvhtree, kdtree

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


def match_skin(obj, skin):
    """Gives `obj` (lying on `skin`) the skin's bone weights and normals at the nearest point, so it moves and shades as the skin under it."""
    obj.vertex_groups.clear()
    for g in skin.vertex_groups:
        obj.vertex_groups.new(name=g.name)
    dt = obj.modifiers.new("Match skin", "DATA_TRANSFER")
    dt.object = skin
    dt.use_vert_data = True
    dt.data_types_verts = {"VGROUP_WEIGHTS"}
    dt.vert_mapping = "POLYINTERP_NEAREST"
    dt.layers_vgroup_select_src = "ALL"
    dt.layers_vgroup_select_dst = "NAME"
    dt.use_loop_data = True
    dt.data_types_loops = {"CUSTOM_NORMAL"}
    dt.loop_mapping = "POLYINTERP_NEAREST"
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    # (Applied first in the stack, before the armature.)
    bpy.ops.object.modifier_move_to_index(modifier=dt.name, index=0)
    bpy.ops.object.modifier_apply(modifier=dt.name)


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
# How far outside the base body's skin its face shell sits (where the skin bulges past the shell's ball).
FACE_CLEARANCE = 0.006
# How far under the skin the face shell's rim is sunk (so its edge never shows).
FACE_RIM_SINK = 0.01
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


## How big the hands are against the first model's, grown round the wrist (so the grip and anything held stay where they were): the
## concept's hands are broad, soft mittens, and at play size a small fist read as a knob on the end of the arm.
HAND_SCALE = 1.24


def _hand(x, p):
    """Point `p` of a hand built at the first model's size, on the `x` side, grown by HAND_SCALE round that side's wrist."""
    w = Vector((x * BLOB_WRIST[0], BLOB_WRIST[1], BLOB_WRIST[2]))
    return w + (Vector(p) - w) * HAND_SCALE


def fist(bm, x):
    """A fist on the `x` side (-1 left, 1 right) at the blob's wrist: a round glove clenched, the four curled fingers a row of knuckle
    bumps underneath, the thumb wrapped across the front."""
    k = HAND_SCALE
    ball(bm, 0.205 * k, _hand(x, (x * 0.775, -0.04, 0.66)), (0.9, 1.0, 0.92), segments=32, rings=20)
    for fy in (-0.135, -0.045, 0.045, 0.135):
        ball(bm, 0.082 * k, _hand(x, (x * 0.8, -0.04 + fy, 0.53)), (1.0, 0.95, 0.9), segments=16, rings=10)
    tube(bm, [_hand(x, (x * 0.69, -0.17, 0.68)), _hand(x, (x * 0.72, -0.24, 0.6)), _hand(x, (x * 0.8, -0.22, 0.55))],
         [0.07 * k, 0.066 * k, 0.06 * k], segments=14, per_span=4)


def mitten(bm, x):
    """A bare hand on the `x` side, built round the same wrist as `fist`: the concept's mitten, a soft rounded paddle a little flatter
    side to side than deep, closed into a loose curl, with a stubby thumb in front. No knuckle row: that is the glove's clench, and on a
    bare hand it read as a glove in the body's colour."""
    # (After the owner's PEAK references, 2026-10-14: a hand with fingers. A broad palm a little flatter side to side than deep, four
    # stubby fingers hanging from it side by side (the middle two longest), each curled gently in toward the palm, and a thumb standing
    # out in front: it reads as a hand at play size, where a ball or a paddle read as a knob.)
    # (Pass 71, the owner: the hands still read chunky and odd. They were as wide as the legs and ran straight on from the forearm, a club
    # at the end of the arm. Now a narrow wrist, a palm flat side to side (the camera sees its back, the fingers hanging from it), four
    # rounded fingers that part a little toward their tips and curl in toward the body, and a shorter thumb in front.)
    k = HAND_SCALE
    tube(bm, [_hand(x, (x * 0.745, -0.04, 0.8)), _hand(x, (x * 0.765, -0.045, 0.715))], [0.072 * k, 0.08 * k], segments=16, per_span=3)
    ball(bm, 0.128 * k, _hand(x, (x * 0.775, -0.05, 0.645)), (0.64, 1.0, 0.98), segments=32, rings=20)
    for fy, length in ((-0.092, 0.1), (-0.031, 0.122), (0.031, 0.116), (0.09, 0.094)):
        top = _hand(x, (x * 0.78, -0.05 + fy, 0.58))
        mid = _hand(x, (x * 0.775, -0.05 + fy * 1.12, 0.58 - length * 0.6))
        tip = _hand(x, (x * 0.74, -0.06 + fy * 1.22, 0.58 - length))
        tube(bm, [top, mid, tip], [0.04 * k, 0.038 * k, 0.033 * k], segments=12, per_span=4)
        ball(bm, 0.033 * k, tip, segments=12, rings=8)
    tube(bm, [_hand(x, (x * 0.75, -0.13, 0.68)), _hand(x, (x * 0.74, -0.19, 0.63)), _hand(x, (x * 0.745, -0.205, 0.575))],
         [0.05 * k, 0.045 * k, 0.04 * k], segments=14, per_span=5)
    ball(bm, 0.04 * k, _hand(x, (x * 0.745, -0.205, 0.575)), segments=12, rings=8)


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


def clampf(v, lo, hi):
    return max(lo, min(hi, v))


def pose(rig, bone, fwd=0.0, out=0.0, twist=0.0, lift=0.0):
    """Sets a bone's pose from forward / outward / twist degrees (and an offset for the hips: upward on the base body; on the blob, whose
    clips were tuned with it, along the hip bone's own z, which is backward and forward)."""
    fwd, out, twist, lift = fwd * AMP, out * AMP, twist * AMP, lift * AMP
    # (Exaggeration multiplies angles, so an arm already raised high would be pushed right round behind the body, and an elbow folded
    # through itself: the shoulder and the elbow stay within their reach.)
    if BODY and bone.startswith("armU."):
        fwd = clampf(fwd, -100.0, 172.0)
    if BODY and bone.startswith("armL."):
        fwd = clampf(fwd, -5.0, 145.0)
    # The base body's head is big: tipped back more than a little it shows its chin and hides the drawn face, the half of every read.
    if BODY and bone == "head" and CLIP_NOW not in HEAD_FREE_CLIPS:
        fwd = max(fwd, HEAD_BACK_LIMIT)
    pb = rig.pose.bones[bone]
    pb.rotation_mode = "XYZ"
    limb = bone.split(".")[0] in LIMBS
    left = bone.endswith(".L")
    x = -fwd if limb else fwd
    z = (-out if left else out)
    pb.rotation_euler = (math.radians(x), math.radians(twist), math.radians(z))
    if bone == "hips":
        pb.location = (0.0, lift, 0.0) if BODY else (0.0, 0.0, lift)


# How far back (degrees) the base body's head may tip, and the clips exempt (lying down or hanging, where the head lifts off the floor or
# looks up the wall).
HEAD_BACK_LIMIT = -8.0
HEAD_FREE_CLIPS = ("knockdown", "getup", "ledge", "ledge_climb")
CLIP_NOW = ""


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
BODY_AMP = 0.85

BODY_MOVES = {
    # A straight punch (or a quick slash): the guard hand stays up at the chin, the shoulders snap round and the front foot steps in.
    # (The shoulders load further back and snap further through than a turn of the chest alone, and the step lands deeper, so even a
    # 4-frame jab reads as the whole body behind the fist at the match camera's distance.)
    "attack_jab": (
        {"hips": -0.05, "spine": (0.0, -30.0), "head": (2.0, 16.0), "stance": (16.0, -18.0), "shin.L": (-26.0,),
         "armU.L": (52.0, 18.0), "armL.L": (106.0,)},
        {"hips": -0.08, "spine": (12.0, 34.0), "head": (-6.0, -20.0), "stance": (34.0, -30.0), "shin.L": (-34.0,), "shin.R": (-4.0,),
         "armU.L": (24.0, 14.0), "armL.L": (112.0,)},
        {"spine": (2.0, 8.0), "stance": (10.0, -8.0)}),
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
    # Fencer's forward tilt: a fast rising cut, stepping in and leaning forward into it as the blade climbs, the free arm opening back for
    # balance.
    "sword_ftilt": (
        {"hips": -0.08, "spine": (12.0, -28.0), "head": (6.0, 14.0), "stance": (18.0, -14.0), "shin.L": (-30.0,), "armU.L": (-10.0, 40.0),
         "armL.L": (30.0,)},
        {"hips": -0.04, "spine": (10.0, 28.0), "head": (-2.0, -12.0), "stance": (36.0, -28.0), "shin.L": (-26.0,), "shin.R": (-4.0,),
         "armU.L": (-30.0, 50.0), "armL.L": (20.0,)},
        {"spine": (2.0, 10.0), "stance": (14.0, -10.0)}),
    # Fencer's forward smash: the blade raised high with the weight back, then one committed cut down in front into a deep lunge, the
    # front knee well bent, the back leg straight, the free arm thrown back.
    "sword_fsmash": (
        {"hips": -0.02, "spine": (-16.0, -40.0), "head": (10.0, 20.0), "stance": (24.0, 18.0), "shin.R": (-40.0,), "armU.L": (20.0, 40.0)},
        # (As the reference's: a lunge so deep the back leg lies nearly flat behind and the body drops low over the bent front knee, the
        # head kept up to the camera.)
        {"hips": -0.24, "spine": (30.0, 30.0), "head": (-14.0, -14.0), "stance": (70.0, -60.0), "shin.L": (-76.0,), "shin.R": (-2.0,),
         "armU.L": (-50.0, 40.0), "armL.L": (10.0,)},
        {"hips": -0.14, "spine": (18.0, 12.0), "stance": (48.0, -40.0), "shin.L": (-56.0,)}),
    # Fencer's down tilt: a low lunging thrust, down on the bent front leg with the back leg stretched long behind.
    "sword_dtilt": (
        {"hips": -0.2, "spine": (20.0, -10.0), "head": (-10.0, 0.0), "stance": (50.0, 26.0), "shin.L": (-90.0,), "shin.R": (-100.0,)},
        {"hips": -0.28, "spine": (34.0, 10.0), "head": (-24.0, 0.0), "stance": (70.0, -30.0), "shin.L": (-92.0,), "shin.R": (-20.0,),
         "armU.L": (-30.0, 30.0)},
        {"hips": -0.16, "spine": (22.0, 0.0), "stance": (48.0, -8.0), "shin.L": (-70.0,)}),
    # The claws' forward tilt: both claws cocked across the chest, then the fighter lunges and throws them out forward and wide (a
    # two-handed outward slash; the game aims the weapon hand, this the other).
    "attack_claws": (
        {"spine": (8.0, -10.0), "head": (0.0, 6.0), "stance": (16.0, -16.0), "armU.L": (60.0, -24.0), "armL.L": (100.0,)},
        # (The reference's is a low lunge, the body nearly level over the bent front leg, one claw thrust far out ahead.)
        {"hips": -0.2, "spine": (44.0, 12.0), "head": (-18.0, -4.0), "stance": (62.0, -50.0), "shin.L": (-72.0,), "shin.R": (-2.0,),
         "armU.L": (-40.0, 34.0), "armL.L": (20.0,)},
        {"hips": -0.08, "spine": (14.0, 4.0), "stance": (30.0, -24.0), "shin.L": (-30.0,)}),
    # Forward air (a crescent cut, or a claw raked, down through the space in front): the weapon raised with the shoulders turned back and
    # the knees drawn up, then the torso leans over the swing and turns into it, the front knee up and the back leg trailing, the free arm
    # flung back. Upright enough that the face stays toward the hit.
    # (Compact then long: the wind-up tucks both knees high and cocks the free arm, the strike opens out, the front knee driven up and the
    # back leg kicked long behind with the free arm flung back, so the silhouette changes shape, not just angle.)
    "attack_fair": (
        {"spine": (-20.0, -28.0), "head": (-8.0, 12.0), "stance": (52.0, 40.0), "shin.L": (-96.0,), "shin.R": (-88.0,),
         "armU.L": (40.0, 30.0), "armL.L": (50.0,)},
        {"spine": (20.0, 30.0), "head": (4.0, -14.0), "stance": (64.0, -24.0), "shin.L": (-100.0,), "shin.R": (-16.0,),
         "armU.L": (-56.0, 40.0), "armL.L": (20.0,)},
        {"spine": (6.0, 8.0), "stance": (34.0, 14.0), "shin.L": (-56.0,), "shin.R": (-46.0,)}),
    # Up air (an overhead crescent from front to back): tucked, then the body arches back under the swing with the head thrown back to
    # watch it, the legs stretching down.
    "attack_uair": (
        {"spine": (12.0, -8.0), "head": (6.0, 0.0), "stance": (34.0, 30.0), "shin.L": (-80.0,), "shin.R": (-70.0,), "armU.L": (24.0, 20.0)},
        {"spine": (-16.0, 10.0), "head": (-10.0, 0.0), "stance": (14.0, -16.0), "shin.L": (-30.0,), "shin.R": (-20.0,),
         "armU.L": (-16.0, 44.0), "armL.L": (20.0,)},
        {"spine": (-6.0, 0.0), "stance": (24.0, 16.0), "shin.L": (-50.0,), "shin.R": (-40.0,)}),
    # Back air (a rising crescent behind): the shoulders turn toward the front, then twist right round so the chest faces behind as the
    # swing goes up behind, the head looking back over the shoulder, the front knee tucked and the back leg reaching out behind.
    "attack_bair": (
        {"spine": (10.0, 26.0), "head": (-4.0, -10.0), "stance": (30.0, 10.0), "shin.L": (-70.0,), "armU.L": (30.0, 20.0)},
        {"spine": (-12.0, -52.0), "head": (6.0, -18.0), "stance": (50.0, -26.0), "shin.L": (-90.0,), "shin.R": (-20.0,),
         "armU.L": (40.0, 30.0), "armL.L": (40.0,)},
        {"spine": (-4.0, -16.0), "stance": (24.0, 0.0), "shin.L": (-40.0,)}),
    # Down air (both claws, or the weapon, brought down through the space below): both arms raised overhead with the body leaning back and
    # the knees drawn up, then the torso curls over and both arms sweep down in front of the knees, the head looking down at the hit.
    "attack_dair": (
        {"spine": (-12.0, 0.0), "head": (-8.0, 0.0), "stance": (50.0, 46.0), "shin.L": (-96.0,), "shin.R": (-96.0,),
         "armU.L": (150.0, 24.0), "armL.L": (30.0,)},
        {"spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (62.0, 54.0), "shin.L": (-110.0,), "shin.R": (-110.0,),
         "armU.L": (24.0, 16.0), "armL.L": (6.0,)},
        {"spine": (10.0, 0.0), "stance": (30.0, 26.0), "shin.L": (-60.0,), "shin.R": (-60.0,)}),
    # Neutral air (an inward cut, then the blade carried right round the body as the game flips it over backward): drawn up compact for
    # the spin, the free arm out.
    "attack_nair": (
        {"spine": (8.0, -18.0), "head": (0.0, 8.0), "stance": (30.0, 24.0), "shin.L": (-60.0,), "shin.R": (-50.0,), "armU.L": (20.0, 20.0)},
        {"spine": (10.0, 6.0), "head": (2.0, -4.0), "stance": (56.0, 50.0), "shin.L": (-100.0,), "shin.R": (-96.0,),
         "armU.L": (-10.0, 56.0), "armL.L": (30.0,)},
        {"spine": (4.0, 0.0), "stance": (26.0, 20.0), "shin.L": (-50.0,), "shin.R": (-40.0,)}),
    # Fencer's down smash: down low, kneeling on the back knee with the front one bent, sweeping in front and then behind, the torso turned
    # into each sweep (the game aims the blade).
    "sword_dsmash": (
        {"spine": (16.0, -20.0), "head": (-6.0, 10.0), "stance": (24.0, -6.0), "shin.L": (-40.0,), "shin.R": (-50.0,)},
        {"spine": (26.0, 30.0), "head": (-12.0, -10.0), "stance": (34.0, -8.0), "shin.L": (-46.0,), "shin.R": (-70.0,),
         "armU.L": (-20.0, 50.0), "armL.L": (20.0,)},
        {"spine": (10.0, 8.0), "stance": (20.0, -6.0), "shin.L": (-30.0,), "shin.R": (-30.0,)}),
    # The claws' down tilt: a low sweeping kick from a crouch on the back leg (the game aims the kicking front leg), the torso over it.
    "kick_low": (
        {"spine": (20.0, -10.0), "stance": (30.0, 30.0), "shin.L": (-80.0,), "shin.R": (-70.0,), "armU.L": (30.0, 30.0)},
        {"spine": (26.0, 16.0), "head": (-10.0, 0.0), "stance": (20.0, 44.0), "shin.R": (-90.0,), "armU.L": (40.0, 40.0)},
        {"spine": (12.0, 0.0), "stance": (24.0, 24.0), "shin.L": (-40.0,), "shin.R": (-40.0,)}),
    # The claws' up tilt: a stretch kick straight up from the standing leg (the game aims the kicking one), upright and leaning back only a
    # little, the arms out for balance.
    "kick_up": (
        {"spine": (10.0, 0.0), "head": (4.0, 0.0), "stance": (10.0, 6.0), "shin.L": (-50.0,), "shin.R": (-20.0,), "armU.L": (20.0, 20.0)},
        {"spine": (-8.0, 0.0), "head": (-12.0, 0.0), "stance": (0.0, 4.0), "shin.R": (-4.0,), "armU.L": (-20.0, 50.0)},
        {"spine": (-2.0, 0.0), "stance": (4.0, 4.0)}),
    # A headbutt straight up (the all-rounder's and the dinosaur's up smash, the ape's up air): crouched low with the head ducked and the
    # arms drawn down and back, then the whole body springs up through the legs and arches back, the arms flung down behind, so the head
    # leads the hit. (`body_` clips: no arm reaches for the hitbox; the body is the weapon.)
    "body_headbutt": (
        {"hips": -0.16, "spine": (34.0, 0.0), "head": (14.0, 0.0), "stance": (30.0, 26.0), "shin.L": (-70.0,), "shin.R": (-66.0,),
         "arms": (-30.0, 26.0)},
        {"hips": 0.1, "spine": (-20.0, 4.0), "head": (-8.0, 0.0), "stance": (6.0, -4.0), "shin.L": (-6.0,), "shin.R": (-4.0,),
         "arms": (-54.0, 34.0)},
        {"spine": (-6.0, 0.0), "stance": (8.0, -4.0)}),
    # A headbutt thrown forward (the dinosaur's forward smash and forward air): rocked back with the head drawn up, then the body lunges
    # head first over the bent front leg, the back leg long behind and both arms swept back.
    "body_headbutt_fwd": (
        {"hips": -0.06, "spine": (-14.0, -18.0), "head": (6.0, 10.0), "stance": (14.0, 26.0), "shin.R": (-44.0,), "arms": (-24.0, 24.0)},
        {"hips": -0.1, "spine": (44.0, 10.0), "head": (8.0, -6.0), "stance": (50.0, -34.0), "shin.L": (-46.0,), "shin.R": (-2.0,),
         "arms": (-62.0, 30.0)},
        {"spine": (14.0, 4.0), "stance": (26.0, -18.0), "shin.L": (-28.0,)}),
    # ---- The Settler's tools (after its reference's moves; the game aims the tool hand at each hit, these set the body and other arm) ----
    # The pocket: a quick lean in with the hand held open out in front, the other arm back, then straightening as it tucks the catch away.
    "tool_reach": (
        {"hips": -0.04, "spine": (-6.0, -14.0), "head": (2.0, 8.0), "stance": (12.0, -8.0), "arms": (10.0, 20.0)},
        {"hips": -0.1, "spine": (30.0, 16.0), "head": (-14.0, -8.0), "stance": (44.0, -32.0), "shin.L": (-46.0,), "armU.L": (-36.0, 30.0)},
        {"spine": (6.0, 4.0), "stance": (14.0, -10.0)}),
    # The pocket's release: an underhand toss, the arm swung from low behind up out in front, the body rising onto the front foot.
    "tool_toss": (
        {"hips": -0.1, "spine": (16.0, -26.0), "head": (0.0, 14.0), "stance": (18.0, 10.0), "shin.R": (-40.0,), "arms": (-20.0, 20.0)},
        {"hips": 0.02, "spine": (-8.0, 22.0), "head": (-8.0, -10.0), "stance": (30.0, -26.0), "shin.R": (-6.0,), "armU.L": (-30.0, 34.0)},
        {"spine": (0.0, 6.0), "stance": (12.0, -8.0)}),
    # Digging (the planting): bent right over with both hands low in front driving the shovel in, then levering it back.
    "tool_dig": (
        {"hips": -0.06, "spine": (10.0, -10.0), "head": (4.0, 6.0), "stance": (24.0, -14.0), "arms": (36.0, 10.0)},
        {"hips": -0.16, "spine": (46.0, 6.0), "head": (-22.0, 0.0), "stance": (40.0, -26.0), "shin.L": (-50.0,), "arms": (42.0, 6.0)},
        {"hips": -0.04, "spine": (14.0, 0.0), "stance": (18.0, -12.0)}),
    # Pouring (the watering can): the can held out in front, tipped, the body leaning in over it and the free hand steadying it.
    "tool_pour": (
        {"spine": (0.0, -8.0), "head": (0.0, 4.0), "stance": (12.0, -10.0), "arms": (40.0, 14.0)},
        {"hips": -0.06, "spine": (20.0, 6.0), "head": (-10.0, 0.0), "stance": (28.0, -18.0), "armU.L": (44.0, -4.0), "armL.L": (20.0,)},
        {"spine": (6.0, 0.0), "stance": (14.0, -10.0)}),
    # A two-handed overhead chop (the axe, the umbrella's swing): both arms raised high with the body arched back on the back foot, then
    # brought down hard in front as the body folds over the front knee.
    "tool_chop": (
        {"hips": -0.02, "spine": (-20.0, -10.0), "head": (6.0, 6.0), "stance": (16.0, 14.0), "shin.R": (-30.0,), "arms": (84.0, 12.0),
         "armL.L": (30.0,)},
        {"hips": -0.16, "spine": (40.0, 8.0), "head": (-18.0, -4.0), "stance": (48.0, -32.0), "shin.L": (-56.0,), "arms": (54.0, 10.0),
         "armL.L": (10.0,)},
        {"hips": -0.06, "spine": (14.0, 2.0), "stance": (22.0, -14.0)}),
    # A heavy two-handed heave (the bowling ball): the ball lifted up to the chest with the body leaning back, then pushed out and down in
    # front as the body pitches forward after it.
    "tool_heave": (
        {"hips": -0.08, "spine": (-16.0, 0.0), "head": (6.0, 0.0), "stance": (20.0, 16.0), "shin.L": (-30.0,), "shin.R": (-30.0,),
         "arms": (46.0, 6.0), "armL.L": (64.0,)},
        {"hips": -0.16, "spine": (38.0, 4.0), "head": (-18.0, 0.0), "stance": (40.0, -24.0), "shin.L": (-56.0,), "arms": (28.0, 8.0),
         "armL.L": (12.0,)},
        {"hips": -0.08, "spine": (16.0, 0.0), "stance": (20.0, -12.0)}),
    # (The cute body's shoulders sit low and its arms are short against its big head: raised as high as they go, the hands come up beside
    # the head, which is the most this body can show. Values here are before the clip's exaggeration, about 1.6 times, and the shoulder
    # is clamped at 172 degrees.)
    # Both arms raised straight up overhead (the firework, the rocket pulled out): a crouch, then stretched tall on the toes.
    "tool_raise": (
        {"hips": -0.12, "spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (26.0, 20.0), "shin.L": (-50.0,), "shin.R": (-50.0,),
         "arms": (40.0, 16.0)},
        {"hips": 0.06, "spine": (-10.0, 0.0), "head": (-6.0, 0.0), "stance": (6.0, -6.0), "arms": (110.0, 10.0), "armL.L": (6.0,), "armL.R": (6.0,)},
        {"spine": (-4.0, 0.0), "stance": (8.0, -4.0)}),
    # The trip (the dash attack): a stumble, then a flop forward flat out over the front foot, the arms flung out ahead holding the pot.
    "tool_trip": (
        {"hips": -0.04, "spine": (14.0, 0.0), "head": (0.0, 0.0), "stance": (30.0, -30.0), "arms": (30.0, 30.0)},
        {"hips": -0.3, "spine": (60.0, 0.0), "head": (-30.0, 0.0), "stance": (60.0, -56.0), "shin.L": (-80.0,), "shin.R": (-30.0,),
         "arms": (150.0, 20.0)},
        {"hips": -0.12, "spine": (24.0, 0.0), "stance": (30.0, -20.0)}),
    # A shrug (nothing caught, the planting failed): the shoulders up, both forearms turned out palms up, the head tipped to one side.
    "tool_shrug": (
        {"spine": (0.0, 0.0), "head": (0.0, 0.0), "stance": (8.0, -4.0), "arms": (10.0, 20.0)},
        {"hips": 0.02, "spine": (-6.0, 0.0), "head": (-4.0, 16.0), "stance": (8.0, -4.0), "arms": (16.0, 34.0), "armL.L": (60.0,),
         "armL.R": (60.0,)},
        {"spine": (-2.0, 0.0), "stance": (8.0, -4.0)}),
    # Riding the rocket (in the air): crouched forward low over it, both hands gripping it in front, the knees drawn up astride it.
    "tool_ride": (
        {"spine": (20.0, 0.0), "head": (-6.0, 0.0), "stance": (60.0, 40.0), "shin.L": (-90.0,), "shin.R": (-90.0,), "arms": (60.0, 10.0)},
        {"spine": (44.0, 0.0), "head": (-26.0, 0.0), "stance": (80.0, 64.0), "shin.L": (-110.0,), "shin.R": (-104.0,), "arms": (48.0, 8.0),
         "armL.L": (20.0,)},
        {"spine": (36.0, 0.0), "head": (-20.0, 0.0), "stance": (74.0, 60.0), "shin.L": (-104.0,), "shin.R": (-100.0,), "arms": (44.0, 8.0)}),
    # Floating under the balloons: both hands up holding the strings, the legs hanging and paddling a little apart.
    "tool_float": (
        {"spine": (-4.0, 0.0), "head": (-6.0, 0.0), "stance": (20.0, -10.0), "shin.L": (-40.0,), "shin.R": (-20.0,), "arms": (104.0, 12.0),
         "armL.L": (30.0,)},
        {"spine": (-8.0, 0.0), "head": (-10.0, 0.0), "stance": (-6.0, 24.0), "shin.L": (-20.0,), "shin.R": (-50.0,), "arms": (110.0, 12.0),
         "armL.L": (24.0,)},
        {"spine": (-6.0, 0.0), "head": (-8.0, 0.0), "stance": (10.0, 10.0), "shin.L": (-30.0,), "shin.R": (-30.0,), "arms": (106.0, 12.0)}),
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
    if name in ("jump", "fall", "air_jump", "air_dodge", "hurt", "roll", "knockdown", "getup", "ledge", "ledge_climb", "kick_dash",
                "hurt_heavy", "hurt_up", "platform_drop", "gaunt_knee", "gaunt_stomp", "tool_ride", "tool_float"):
        return False
    return not name.endswith("air")


STANDING = {"idle", "crouch", "jumpsquat", "land", "spot_dodge", "shield", "skid", "grab", "throw", "blaster", "sword_usmash", "sword_dsmash",
            "tech", "run_stop", "idle_look"}


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
    global AMP, CLIP_NOW
    CLIP_NOW = name
    if name.startswith("attack") or name in ("grab", "throw"):
        AMP = 1.85
    elif name.startswith(("sword_", "kick_", "gaunt_", "tool_", "body_")) or name == "blaster":
        AMP = 1.75
    elif name in ("walk", "run", "dash", "skid"):
        AMP = 1.0
    elif name in ("jump", "fall", "hurt", "crouch", "jumpsquat", "land", "shield", "roll", "spot_dodge", "air_dodge", "knockdown"):
        AMP = 1.25
    else:
        AMP = 1.0
    if BODY:
        AMP = 1.0 + (AMP - 1.0) * BODY_AMP
    # (A move with no poses of its own, such as the claws' lunging slash, uses its BODY_MOVES poses on the blob too.)
    if name in BODY_MOVES and (BODY or not poses):
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
# The foot is down only briefly (contact to push-off is 0.18 of the cycle) and the rest is flight: the stubby legs can only sweep so far,
# so a short stance is what keeps the planted foot from sliding at the fighters' running speeds (motion plan phase 4, measured by
# godot/tests/motion_probe.gd).
RUN_THIGH = [(0.0, 42.0), (0.06, 24.0), (0.18, -40.0), (0.32, -56.0), (0.5, -20.0), (0.74, 82.0), (0.88, 70.0)]
RUN_SHIN = [(0.0, -12.0), (0.06, -40.0), (0.18, -14.0), (0.32, -70.0), (0.5, -128.0), (0.74, -112.0), (0.88, -34.0)]
RUN_FOOT = [(0.0, 12.0), (0.06, -4.0), (0.18, -26.0), (0.32, -44.0), (0.5, -30.0), (0.74, 24.0), (0.88, 20.0)]


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
            # (Held further out from the body, after the owner's PEAK reference: the chunky arms pump clear of the trunk and read.)
            pose(rig, "armU." + side, fwd=arms * (-62.0 * swing + 8.0), out=28.0 + 10.0 * forward)
            pose(rig, "armL." + side, fwd=arms * (62.0 + 46.0 * forward))
            pose(rig, "hand." + side, fwd=10.0 * swing)
        # Two steps a cycle: lowest just after each landing (0.06, 0.56), highest in the flight between them.
        step = math.cos((phase - 0.31) * 4.0 * math.pi)
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
## How big the fists are against the blob's (the concept's mitten hands are small: about a quarter of the head across).
FIST_SCALE = 0.9
# The glove's fist (it is built bigger and rounder than the bare hand, with knuckles and a cuff): smaller, or a gloved fighter carried two
# white balloons.
GLOVE_SCALE = 0.74
GLOVE_GROW = 1.1     # the base body's glove against its bare hand
FIST_CLOSED = 0.66   # the base body's closed fist (the blob's clenched glove) against the blob's
## How far the head is lowered onto the shoulders: the old generated body had a short neck between them, which the character has none of.
## The scripted body (make_base_body.py) has no neck, so it is built with `--neck-drop 0`.
NECK_DROP = float(sys.argv[sys.argv.index("--neck-drop") + 1]) if "--neck-drop" in sys.argv else 0.07


SHOULDER_HOLD = 0.08     # within this distance of the shoulder joint the trunk keeps its arm weights
SHOULDER_FADE = 0.1     # and over this much further they fade to none


def release_trunk(skin, J):
    """Fades the arm bones' weights out of the trunk (the points inside the shoulder joints' line) with distance from the shoulder joint,
    giving what they held to the spine, so a raised arm lifts the shoulder's round and not a web of the torso's side."""
    names = {g.index: g.name for g in skin.vertex_groups}
    spine = skin.vertex_groups["spine"]
    for v in skin.data.vertices:
        side = "R" if v.co.x > 0.0 else "L"
        joint = J["armU." + side]
        if abs(v.co.x) >= abs(joint.x) or v.co.z < J["hips"].z:
            continue
        d = (v.co - joint).length
        keep = max(0.0, min(1.0, (SHOULDER_HOLD + SHOULDER_FADE - d) / SHOULDER_FADE))
        if keep >= 1.0:
            continue
        freed = 0.0
        for g in v.groups:
            if names[g.group].startswith(("armU", "armL", "hand")):
                freed += g.weight * (1.0 - keep)
                g.weight *= keep
        if freed > 0.0:
            spine.add([v.index], next((g.weight for g in v.groups if g.group == spine.index), 0.0) + freed, "REPLACE")


def smooth_weights(obj, groups, factor=0.5, repeat=6, no_gain=None):
    """Spreads the named vertex groups' weights along the mesh (each point toward its neighbours' average, `repeat` times), then puts
    every point's weights back to a sum of one. Points in `no_gain` (indices) keep at most the weight they had in those groups."""
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
        before = list(w)
        for _ in range(repeat):
            w[:] = [w[i] + factor * (sum(w[j] for j in near[i]) / len(near[i]) - w[i]) if near[i] else w[i] for i in range(n)]
        for i in no_gain or ():
            w[i] = min(w[i], before[i])
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
            e = J["armL." + side]
            fore = J["hand." + side] - e
            tf = (v.co - e).dot(fore) / fore.length_squared
            # (Square across the forearm, not the shoulder-to-wrist line: the arm bends at the elbow, and a cut across that line reached
            # up one side of the forearm in a notch whose hole showed the outline's black inside.)
            if t > 0.9 and tf > 1.0 + 0.03 * length / fore.length:
                cut.append(v)
                break
            # (How thick the forearm is near the wrist, for the cuffs.)
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
        # (The glove is the bare hand's shape a size up, not the blob's clenched ball: gloved fighters carried two white balloons.)
        bm = bmesh.new()
        mitten(bm, x)
        for v in bm.verts:
            v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * FIST_SCALE * GLOVE_GROW
        parts["Hand." + side] = to_object("Hand." + side, bm, "hand." + side, rig)
        # The bare hand (no gloves in the loadout): the game shows this one instead, in the body's colour.
        bm = bmesh.new()
        mitten(bm, x)
        for v in bm.verts:
            v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * FIST_SCALE
        # A wrist plug: the forearm skin is cut open at the wrist and the hand's rounded top did not fill the cut, so its inside showed as a
        # dark ring (a bracelet). A short round plug the width of the wrist closes it, running a little up into the forearm.
        rw = sorted(wrist_r[side])
        rw = rw[len(rw) // 2] if rw else 0.08
        tube(bm, [J["hand." + side] - fore * 0.07, J["hand." + side] + fore * 0.05], [rw * 1.0, rw * 1.04], segments=18, per_span=3)
        parts["Mitt." + side] = to_object("Mitt." + side, bm, "hand." + side, rig)
        # (No ink where the hand goes into the forearm: the hull of its rounded top showed as a dark ring round the wrist, a bracelet. The
        # vertex colour's red is how much outline a point gets, shaders/outline.gdshader.)
        mitt = parts["Mitt." + side]
        ink = mitt.data.color_attributes.new("outline", "FLOAT_COLOR", "POINT")
        wrist_at = J["hand." + side]
        for v in mitt.data.vertices:
            up = (v.co - wrist_at).dot(-fore)
            w = min(1.0, max(0.0, (-0.02 - up) / 0.06))
            ink.data[v.index].color = (w, 1.0, 1.0, 1.0)
        mitt.data.color_attributes.active_color = ink
        # Closed fists to match, bare and gloved (`FistBare`, `FistGlove`): the game shows them on a hand that grips a weapon and on
        # an unarmed fighter's hands while it attacks (an open hand beside a sword's hilt read as not holding it, a punch as a slap).
        for name, k in (("FistBare." + side, FIST_CLOSED), ("FistGlove." + side, FIST_CLOSED * GLOVE_GROW)):
            bm = bmesh.new()
            fist(bm, x)
            for v in bm.verts:
                v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * k
            if name.startswith("FistBare"):
                tube(bm, [J["hand." + side] - fore * 0.07, J["hand." + side] + fore * 0.05], [rw * 1.0, rw * 1.04], segments=18, per_span=3)
            parts[name] = to_object(name, bm, "hand." + side, rig)
            if name.startswith("FistBare"):
                f = parts[name]
                ink = f.data.color_attributes.new("outline", "FLOAT_COLOR", "POINT")
                for v in f.data.vertices:
                    up = (v.co - wrist_at).dot(-fore)
                    ink.data[v.index].color = (min(1.0, max(0.0, (-0.02 - up) / 0.06)), 1.0, 1.0, 1.0)
                f.data.color_attributes.active_color = ink
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
    # The head is not quite a ball: toward the jaw the skin bulges past it and would hide the lower face (a mouth moved down, big eyes).
    # Each point of the shell is pushed out, along the line from the head's middle, until it is just outside the skin.
    skin_bm = bmesh.new()
    skin_bm.from_mesh(skin.data)
    skin_tree = bvhtree.BVHTree.FromBMesh(skin_bm)
    # The shell's rim sinks just under the skin (the next ring in, level with it), so the head itself hides the shell's edge: no step in
    # depth there for the ink pass to draw as a line round the face.
    rim = {v for v in bm.verts if v.is_boundary}
    near_rim = {e.other_vert(v) for v in rim for e in v.link_edges} - rim
    for v in bm.verts:
        out = v.co - centre
        hit = skin_tree.ray_cast(centre, out.normalized(), out.length * 1.5)
        if hit[0] is not None:
            skin_at = (hit[0] - centre).length
            if v in rim:
                v.co = centre + out.normalized() * (skin_at - FACE_RIM_SINK)
            elif v in near_rim:
                v.co = centre + out.normalized() * skin_at
            elif skin_at + FACE_CLEARANCE > out.length:
                v.co = centre + out.normalized() * (skin_at + FACE_CLEARANCE)
    skin_bm.free()
    face_size = 1.75 * radii.x
    # The drawing is laid out about the middle of the round head, as on the concept sheet (eyes above it, the mouth below). The head's
    # points are only counted from just above the neck, so the middle of their span is well above the ball's: the ball's middle is its
    # crown less its radius.
    face_centre = hi.z - radii.x
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
    # (Spread further, and the chest with them, so the armpit and the shoulder bend round with the arm and the elbow does not twist into a
    # pinch: the owner saw the arms twist strangely in play.)
    # (But the trunk itself gains none: spread onto the chest and back, the arm's weight dragged the torso's skin up with a raised arm
    # into a web like a poncho under each arm, all through the run.)
    trunk_ids = [v.index for v in skin.data.vertices if abs(v.co.x) < abs(J["armU.R"].x) * 0.72 and v.co.z > J["hips"].z]
    smooth_weights(skin, [g + "." + side for side in "LR" for g in ("armU", "armL")], 0.5, 12, trunk_ids)
    # And on the trunk the arm's pull fades with distance from the shoulder joint: the round of the shoulder rises with a raised arm, the
    # chest and the sides under it stay with the spine (the owner, 2026-10-14: the arms stretched the body and did not read as shoulders).
    release_trunk(skin, J)
    # The ball of the head belongs wholly to the head bone (the owner saw the head melt into the chest, a gourd): the automatic weights
    # gave its underside partly to the spine, which the game twists toward the camera apart from the head, so the chin stretched into the
    # shoulders. Everything inside the head's ball, down to a little above the neck, takes the head's weight alone.
    hc = Vector(FIT["head_centre"])
    hr = float(FIT["head_radius"])
    hg = skin.vertex_groups.get("head")
    if hg is not None:
        for v in skin.data.vertices:
            if (v.co - hc).length < hr * 1.04 and v.co.z > FIT["neck"] + 0.02:
                for g in v.groups:
                    if g.group != hg.index:
                        skin.vertex_groups[g.group].remove([v.index])
                hg.add([v.index], 1.0, "REPLACE")

    # The face shell bends and shades with the skin it lies on: it takes the skin's own weights (the jaw and cheeks partly follow the
    # chest, which the game twists toward the camera) and its normals. Bound rigidly to the head bone, it slid into the skin on one side
    # when the chest twisted, and its ball normals shaded it a little darker than the skin there: a jagged seam down the face of a
    # fighter facing left.
    match_skin(parts["Face"], skin)

    # The clothes, modelled: a shirt and a pair of shorts cut from the skin itself (so they fit it and bend with it exactly, weights and
    # all), lifted off it and given a little thickness, with clean straight hems.
    parts.update(build_garments(skin, J, neck))

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
        "waist": round(hips + WAIST_SHARE * (neck - hips), 4),
        "shorts_end": round(crotch - 0.42 * (crotch - knee), 4),
        "sock_top": round(ankle + 0.19, 4),
        "shoe_top": round(ankle + 0.03, 4),
        "sole_top": round(0.035, 4),
        "shoulder_l": [round(J["armU.L"].x, 4), round(J["armU.L"].z, 4)], "elbow_l": [round(J["armL.L"].x, 4), round(J["armL.L"].z, 4)],
        "shoulder_r": [round(J["armU.R"].x, 4), round(J["armU.R"].z, 4)], "elbow_r": [round(J["armL.R"].x, 4), round(J["armL.R"].z, 4)],
        "torso_half_width": round(abs(J["armU.R"].x) * 0.72, 4),
        # (The garments' own measures, so the game's skin shader hides the same skin the sleeves cover.)
        "sleeve": SLEEVE, "arm_radius": ARM_RADIUS,
    }
    # The torso as the ellipsoid the game's neckwear is made round (fighter_view.gd fits its own design ellipsoid onto this one): from the
    # hips to where the head meets the shoulders, as wide as the trunk under the arms and as deep as its middle.
    trunk = [v.co for v in skin.data.vertices if hips < v.co.z < neck and abs(v.co.x) < 0.2]
    mid_y = (max(c.y for c in trunk) + min(c.y for c in trunk)) / 2.0
    depths = sorted(abs(c.y - mid_y) for c in trunk)
    torso = {"centre": godot(Vector((0.0, mid_y, (hips + neck) / 2.0))),
             "radii": [round(abs(J["armU.R"].x) * 0.75, 4), round((neck - hips) / 2.0, 4), round(depths[int(len(depths) * 0.9)], 4)]}
    # Where the face drawing is on the head (the middle of its square, as a height, and the square's size), for fitting glasses to its
    # eyes.
    info = {"head_ball": [round(FIT["head_centre"][2], 4), round(FIT["head_centre"][1], 4), round(FIT["head_radius"], 4)],
            "head_centre": godot(centre), "head_radii": [round(radii.x, 4), round(radii.z, 4), round(radii.y, 4)], "torso": torso,
            "face": [round(face_centre, 4), round(face_size, 4)],
            "neck": round(neck, 4), "shoulder": godot(J["armU.R"]), "wrist": godot(J["hand.R"]), "hips": round(J["hips"].z, 4),
            "height": round(FIT["height"], 4), "clothes": clothes}
    with open(OUT.replace(".glb", ".json"), "w") as f:
        json.dump(info, f, indent=1)
    return parts


def shorts_pockets(shorts, skin, waist, crotch):
    """A pocket flap on each side of the shorts, under the belt, laid on their finished surface and moving as the thigh under it does:
    cargo shorts, as PEAK's scouts wear, where ours were a plain tube."""
    sbm = bmesh.new()
    sbm.from_mesh(shorts.data)
    sbm.normal_update()
    tree = bvhtree.BVHTree.FromBMesh(sbm)
    out_bm = bmesh.new()
    top = waist - BELT_HALF - 0.025
    bottom = top - POCKET_HEIGHT
    for sx in (-1.0, 1.0):
        grid = {}
        n = 8
        for i in range(n + 1):
            for j in range(n + 1):
                z = top + (bottom - top) * i / n
                y = POCKET_FRONT + (POCKET_BACK - POCKET_FRONT) * j / n
                # (The flap's bottom corners rounded off.)
                u = abs(j / n - 0.5) * 2.0
                if i == n and u > 0.6:
                    z += 0.02 * (u - 0.6) / 0.4
                hit, normal, _, _ = tree.ray_cast(Vector((sx * 2.0, y, z)), Vector((-sx, 0.0, 0.0)))
                if hit is None:
                    continue
                if normal.x * sx < 0.0:
                    normal = -normal
                grid[i, j] = out_bm.verts.new(hit + normal * POCKET_LIFT)
        for (i, j), v in list(grid.items()):
            a, b, c = grid.get((i + 1, j)), grid.get((i, j + 1)), grid.get((i + 1, j + 1))
            if a and b and c:
                out_bm.faces.new((v, a, c, b) if sx > 0 else (v, b, c, a))
    out_bm.normal_update()
    bmesh.ops.solidify(out_bm, geom=list(out_bm.faces), thickness=0.012)
    bmesh.ops.recalc_face_normals(out_bm, faces=list(out_bm.faces))
    obj = shorts.copy()
    obj.data = bpy.data.meshes.new("ShortsPocket")
    obj.name = "ShortsPocket"
    bpy.context.collection.objects.link(obj)
    out_bm.to_mesh(obj.data)
    out_bm.free()
    for poly in obj.data.polygons:
        poly.use_smooth = True
    for layer in list(obj.data.color_attributes):
        obj.data.color_attributes.remove(layer)
    # Each point takes the shorts' own weights at the nearest point of the shorts (the thigh below the crotch, the hips above).
    names = [g.name for g in shorts.vertex_groups]
    near = kdtree.KDTree(len(shorts.data.vertices))
    for v in shorts.data.vertices:
        near.insert(v.co, v.index)
    near.balance()
    for v in obj.data.vertices:
        _, i, _ = near.find(v.co)
        for g in shorts.data.vertices[i].groups:
            group = obj.vertex_groups.get(names[g.group]) or obj.vertex_groups.new(name=names[g.group])
            group.add([v.index], g.weight, "REPLACE")
    sbm.free()
    return {"ShortsPocket": obj}


POCKET_HEIGHT = 0.13
POCKET_FRONT = -0.1
POCKET_BACK = 0.07
POCKET_LIFT = 0.006


def shirt_front(shirt, skin, neck_line, waist, trunk_weights):
    """The front of the shirt, laid on its finished surface (so they sit on it in any body): a collar of two pointed flaps folded down
    from the throat, apart in a V, a button placket from the V to the belt, and three buttons. (The owner's PEAK references, 2026-10-14:
    their scout shirts have a collar and a placket. The game's boxes for them sat inside the cloth once the shirt changed shape.)"""
    sbm = bmesh.new()
    sbm.from_mesh(shirt.data)
    sbm.normal_update()
    tree = bvhtree.BVHTree.FromBMesh(sbm)
    names = [g.name for g in skin.vertex_groups]

    def on_front(x, z, out_by):
        hit, normal, _, _ = tree.ray_cast(Vector((x, -3.0, z)), Vector((0.0, 1.0, 0.0)))
        if hit is None:
            return None
        if normal.y > 0.0:
            normal = -normal
        return hit + normal * out_by

    def finish(name, bm, thickness, smooth=True):
        obj = shirt.copy()
        obj.data = bpy.data.meshes.new(name)
        obj.name = name
        bpy.context.collection.objects.link(obj)
        if thickness > 0.0:
            bm.normal_update()
            bmesh.ops.solidify(bm, geom=list(bm.faces), thickness=thickness)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        bm.to_mesh(obj.data)
        bm.free()
        for poly in obj.data.polygons:
            poly.use_smooth = smooth
        for layer in list(obj.data.color_attributes):
            obj.data.color_attributes.remove(layer)
        for layer, uv_of in (("rest", lambda co: (co.x, co.z)), ("rest_depth", lambda co: (co.y, 0.0))):
            if layer not in obj.data.uv_layers:
                obj.data.uv_layers.new(name=layer)
            data = obj.data.uv_layers[layer].data
            for loop in obj.data.loops:
                data[loop.index].uv = uv_of(obj.data.vertices[loop.vertex_index].co)
        for v in obj.data.vertices:
            for gi, w in (trunk_weights(v.co.z) or {}).items():
                group = obj.vertex_groups.get(names[gi]) or obj.vertex_groups.new(name=names[gi])
                group.add([v.index], w, "REPLACE")
        return obj

    def patch(corners, n, out_by, lift_edge=0.0):
        # A triangle (three corners, x and height) or a quad (four), as a grid of points each set on the shirt's front.
        bm = bmesh.new()
        grid = {}
        quad = len(corners) == 4
        for i in range(n + 1):
            for j in range(n + 1 - (0 if quad else i)):
                u, w = i / n, j / n
                if quad:
                    a = corners[0].lerp(corners[1], w)
                    b = corners[3].lerp(corners[2], w)
                    p = a.lerp(b, u)
                    edge = 0.0
                else:
                    p = corners[0] * (1.0 - u - w) + corners[1] * w + corners[2] * u
                    edge = max(u, w)
                co = on_front(p.x, p.y, out_by + lift_edge * edge)
                if co is not None:
                    grid[i, j] = bm.verts.new(co)
        for (i, j), v in list(grid.items()):
            a, b, c = grid.get((i + 1, j)), grid.get((i, j + 1)), grid.get((i + 1, j + 1))
            if quad:
                if a and b and c:
                    bm.faces.new((v, a, c, b))
            else:
                if a and b:
                    bm.faces.new((v, a, b))
                if a and b and c:
                    bm.faces.new((a, c, b))
        return bm

    def merge(into, part):
        tmp = bpy.data.meshes.new("tmp")
        part.to_mesh(tmp)
        part.free()
        into.from_mesh(tmp)
        bpy.data.meshes.remove(tmp)

    out = {}
    bm = bmesh.new()
    for sx in (-1.0, 1.0):
        # Inner corner at the throat, outer corner out along the shoulder line, the point down and out over the chest; the free edges
        # stand a little further off, as a folded collar does.
        tri = [Vector((sx * 0.03, neck_line - 0.1)), Vector((sx * 0.24, neck_line - 0.05)), Vector((sx * 0.15, neck_line - 0.3))]
        merge(bm, patch(tri, 8, 0.014, 0.016))
    out["ShirtCollar"] = finish("ShirtCollar", bm, 0.014)
    top, bottom = neck_line - 0.2, waist + BELT_HALF + 0.005
    plk = patch([Vector((-0.032, top)), Vector((0.032, top)), Vector((0.032, bottom)), Vector((-0.032, bottom))], 12, 0.006)
    out["ShirtPlacket"] = finish("ShirtPlacket", plk, 0.01)
    bm = bmesh.new()
    for k in range(3):
        z = top - 0.04 - k * (top - bottom - 0.08) / 2.0
        co = on_front(0.0, z, 0.02)
        if co is None:
            continue
        b = bmesh.new()
        bmesh.ops.create_uvsphere(b, u_segments=12, v_segments=8, radius=0.028)
        bmesh.ops.scale(b, vec=(1.0, 0.45, 1.0), verts=b.verts)
        bmesh.ops.translate(b, vec=co, verts=b.verts)
        merge(bm, b)
    out["Buttons"] = finish("Buttons", bm, 0.0)
    sbm.free()
    return out


# ---- Clothes: garments cut from the skin ------------------------------------------------------------------------------------------------
SHIRT_LIFT = 0.04      # how far off the skin the shirt sits
# Where the waist is, as a share of the way from the hips up to the neck. (The owner, 2026-10-14: PEAK's figures have a waistline; ours
# had the shorts' top almost at the crotch, 0.12, so the shirt ran in one straight box to the hips. Now the shorts come up to a belt at the
# body's narrowest, and the shirt is tucked in under it.)
WAIST_SHARE = 0.25
BELT_HALF = 0.045       # half the belt's height
BELT_LIFT = 0.056       # how far off the skin the belt sits (over the shirt's tucked hem and the shorts' waistband)
BLOUSE = 0.008          # how far the shirt puffs out just above the belt, where it is tucked in
TUCK_SPAN = 0.12        # how far above the belt the shirt blends into moving as the belt does
BLOUSE_SPAN = 0.1      # over how much height above the belt
# Boots (the owner's PEAK references: chunky hiking boots on thick soles, where ours had shoes painted on the feet): an upper cut from the
# foot up a little past the ankle, and a sole that stands out round it like a rim.
BOOT_ABOVE_ANKLE = 0.1
BOOT_LIFT = 0.03
SOLE_HEIGHT = 0.08
SOLE_LIFT = 0.062
BOOT_CUFF = 0.04        # a padded collar round the boot's top
BOOT_CUFF_LIFT = 0.046
# The shorts' rolled cuffs, a band round each leg opening standing a little proud of the cloth.
CUFF_HEIGHT = 0.06
CUFF_LIFT = 0.034
SLEEVE_SNUG = 0.55      # the sleeve's opening hugs the arm (this share of the shirt's distance), so no dark gap shows inside it
SHORTS_TOP = -0.002   # the shorts reach just under the shirt's hem (below it a little, they could poke through)
SHORTS_LIFT = 0.018     # the shorts sit a little closer in, so the shirt's hem lies over their waistband
CLOTH_THICKNESS = 0.0     # none: an inner layer crossed the outer one where the cloth folds (the game draws cloth two-sided)
SLEEVE = 0.85           # how far down the upper arm the sleeves reach (0 at the shoulder joint, 1 at the elbow; the joint sits low in the
                       # shoulder, so 0.55 left only a cap over it: PEAK's short sleeves reach most of the way to the elbow)
HEM_FLARE = 0.02
ARM_RADIUS = 0.2       # how far from the arm's axis a point still counts as the arm (the base body's arm is about 0.16 thick; at 0.14 a strip down
                       # its outer side was taken for the torso's shirt, slivers and a slit by the sleeve)
HEM_SMOOTH = 30
TORSO_SLICE = 0.01       # the height of a slice of the torso that shares one blend of the trunk's bones
TORSO_SLICE_SPREAD = 4   # how many slices either side are averaged in       # how many smoothing steps each hem gets along its loop
WEIGHT_BLUR = 0.06     # how far round each cloth point the skin's bone weights are averaged
GARMENT_SMOOTH = 14     # how many smoothing steps the cloth gets (it drapes over the body's small bumps)
GARMENT_WEIGHT_SMOOTH = 24
ARMPIT_NO_INK = 0.07          # how far round each armpit the shirt has no ink outline   # how many smoothing steps the cloth's bone weights get        # how much further out a hem stands than the rest of the garment (a soft, cartoony flare)


def build_garments(skin, J, neck):
    """A shirt (from the waist to the neck, short sleeves) and shorts (from the waist to above the knee) for the base body. Each is a copy of
    the skin cut along its hems (planes, so the edges are straight however the mesh is laid out), keeping the faces in the garment's region,
    pushed out along the surface (further near a hem, so it flares a little) and given thickness. The copies keep the skin's bone weights,
    so the clothes move exactly with the body. The game colours them (by name: "Shirt", "Shorts")."""
    hips = J["hips"].z
    crotch = FIT["crotch"]
    knee = (J["shin.L"].z + J["shin.R"].z) / 2.0
    waist = hips + WAIST_SHARE * (neck - hips)
    shorts_end = crotch - 0.42 * (crotch - knee)
    neck_line = neck - 0.015
    ankle = (J["foot.L"].z + J["foot.R"].z) / 2.0
    boot_top = ankle + BOOT_ABOVE_ANKLE
    cuff_top = shorts_end + CUFF_HEIGHT
    half_width = abs(J["armU.R"].x) * 0.72
    arm = {}
    for side in "LR":
        a = Vector((J["armU." + side].x, 0.0, J["armU." + side].z))
        b = Vector((J["armL." + side].x, 0.0, J["armL." + side].z))
        arm[side] = (a, b)

    wrist = {side: Vector((J["hand." + side].x, 0.0, J["hand." + side].z)) for side in "LR"}

    def on_arm(co):
        # Close to the arm's own axis (shoulder to wrist): the body is about as wide at the hips as the arms' inner edge, so being far from
        # the middle alone took the hips' sides for arms (they missed the smoothing and the straight hems, and stuck out as spikes).
        if co.z >= neck or abs(co.x) < 0.15:
            return False
        side = "L" if co.x < 0.0 else "R"
        a = arm[side][0]
        b = wrist[side]
        p = Vector((co.x, 0.0, co.z))
        t = max(0.0, min(1.0, (p - a).dot(b - a) / (b - a).length_squared))
        return (p - (a + (b - a) * t)).length < ARM_RADIUS

    def sleeve_t(co):
        a, b = arm["L" if co.x < 0.0 else "R"]
        ab = b - a
        return (Vector((co.x, 0.0, co.z)) - a).dot(ab) / ab.length_squared

    def in_shirt(co):
        if co.z >= neck_line:
            return False
        if on_arm(co):
            return sleeve_t(co) < SLEEVE
        return co.z > waist - 0.012      # just below the waist, over the shorts' waistband

    def in_shorts(co):
        return not on_arm(co) and shorts_end < co.z <= waist + SHORTS_TOP

    def shirt_hem(co):
        # Distance to the nearest of the shirt's edges (the waist hem, the neckline, a sleeve end).
        d = min(abs(co.z - (waist - 0.012)), abs(co.z - neck_line))
        if on_arm(co):
            a, b = arm["L" if co.x < 0.0 else "R"]
            d = min(d, abs(sleeve_t(co) - SLEEVE) * (b - a).length)
        return d

    def in_boot(co):
        return co.z < boot_top

    def in_boot_cuff(co):
        return boot_top - BOOT_CUFF < co.z < boot_top

    def in_sole(co):
        return co.z < SOLE_HEIGHT

    def in_cuff(co):
        return not on_arm(co) and shorts_end < co.z < cuff_top and co.z < crotch

    def boot_hem(co):
        return abs(co.z - boot_top)

    def in_belt(co):
        return not on_arm(co) and abs(co.z - waist) < BELT_HALF

    def belt_hem(co):
        return 1.0      # (no flare: a belt is a stiff band)

    def shorts_hem(co):
        # Only the leg openings flare: the waistband stays close in, under the shirt's hem.
        return abs(co.z - shorts_end)

    cuts = [(Vector((0, 0, waist - 0.012)), Vector((0, 0, 1))), (Vector((0, 0, neck_line)), Vector((0, 0, 1))),
            (Vector((0, 0, shorts_end)), Vector((0, 0, 1))), (Vector((0, 0, waist + SHORTS_TOP)), Vector((0, 0, 1))),
            (Vector((0, 0, waist - BELT_HALF)), Vector((0, 0, 1))), (Vector((0, 0, waist + BELT_HALF)), Vector((0, 0, 1))),
            (Vector((0, 0, boot_top)), Vector((0, 0, 1))), (Vector((0, 0, SOLE_HEIGHT)), Vector((0, 0, 1))),
            (Vector((0, 0, cuff_top)), Vector((0, 0, 1))), (Vector((0, 0, boot_top - BOOT_CUFF)), Vector((0, 0, 1)))]
    for side in "LR":
        a, b = arm[side]
        cuts.append((a + (b - a) * SLEEVE, (b - a).normalized()))
    skin_points = kdtree.KDTree(len(skin.data.vertices))
    for v in skin.data.vertices:
        skin_points.insert(v.co, v.index)
    skin_points.balance()
    # The torso's skin alone, for the cloth over the torso to take its weights from: in the rest pose the arms hang right beside the hips,
    # and hem points that took some of an arm's weights were dragged up in teeth when the fists came up.
    arm_groups = {g.index for g in skin.vertex_groups if g.name.startswith(("armU", "armL", "hand"))}
    torso_ids = [v.index for v in skin.data.vertices if not on_arm(v.co) and not any(g.group in arm_groups and g.weight > 0.05 for g in v.groups)]
    torso_points = kdtree.KDTree(len(torso_ids))
    for i in torso_ids:
        torso_points.insert(skin.data.vertices[i].co, i)
    torso_points.balance()
    # The torso's own bones by height: the average of the hips, spine and neck weights over each slice of the torso's skin (the thighs and
    # arms left out: in the wide stance hem points that followed a thigh a little crumpled into teeth). Torso cloth takes these.
    trunk = {g.index: g.name for g in skin.vertex_groups if g.name in ("hips", "spine", "neck")}
    sums = {}
    for i in torso_ids:
        v = skin.data.vertices[i]
        b = int(round(v.co.z / TORSO_SLICE))
        acc = sums.setdefault(b, {})
        for g in v.groups:
            if g.group in trunk:
                acc[g.group] = acc.get(g.group, 0.0) + g.weight
    trunk_slices = {}
    for b in sums:
        total = {}
        for k in range(b - TORSO_SLICE_SPREAD, b + TORSO_SLICE_SPREAD + 1):
            for gi, w in sums.get(k, {}).items():
                total[gi] = total.get(gi, 0.0) + w
        whole = sum(total.values())
        if whole > 0.0:
            trunk_slices[b] = {gi: w / whole for gi, w in total.items()}

    def trunk_weights(z):
        if not trunk_slices:
            return None
        return trunk_slices[min(trunk_slices, key=lambda k: abs(k - z / TORSO_SLICE))]

    skin_bm = bmesh.new()
    skin_bm.from_mesh(skin.data)
    skin_bm.normal_update()
    skin_tree = bvhtree.BVHTree.FromBMesh(skin_bm)
    out = {}
    for name, inside, hem, lift in (("Shirt", in_shirt, shirt_hem, SHIRT_LIFT), ("Shorts", in_shorts, shorts_hem, SHORTS_LIFT),
                                    ("Belt", in_belt, belt_hem, BELT_LIFT), ("Boot", in_boot, boot_hem, BOOT_LIFT),
                                    ("BootSole", in_sole, belt_hem, SOLE_LIFT), ("BootCuff", in_boot_cuff, belt_hem, BOOT_CUFF_LIFT), ("ShortsCuff", in_cuff, belt_hem, CUFF_LIFT)):
        obj = skin.copy()
        obj.data = skin.data.copy()
        obj.name = name
        obj.data.name = name
        bpy.context.collection.objects.link(obj)
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        for co, normal in cuts:
            geom = list(bm.verts) + list(bm.edges) + list(bm.faces)
            bmesh.ops.bisect_plane(bm, geom=geom, plane_co=co, plane_no=normal)
        drop = [f for f in bm.faces if not inside(f.calc_center_median())]
        bmesh.ops.delete(bm, geom=drop, context="FACES")
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
        # (The cuts leave slivers; merged away, they cannot throw a point far off when the surface is pushed out.)
        bmesh.ops.dissolve_degenerate(bm, dist=0.002, edges=bm.edges)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.002)
        # The body mesh is not quite symmetric, so a copy of it has two different legs and sides: keep the right half and mirror it.
        bmesh.ops.bisect_plane(bm, geom=list(bm.verts) + list(bm.edges) + list(bm.faces), plane_co=(0, 0, 0), plane_no=(1, 0, 0),
                               clear_inner=True)
        bmesh.ops.symmetrize(bm, input=list(bm.verts) + list(bm.edges) + list(bm.faces), direction="X", dist=0.001)
        if min(v.co.x for v in bm.verts) > -0.05:
            raise RuntimeError(name + ": mirroring kept only one side")
        # Cloth drapes over the body's small bumps (the belly, the chest) rather than following each one: smooth the surface, the hems
        # held where they are, then push any point the smoothing pulled toward the body back out to the garment's distance from it.
        hems = {v for v in bm.verts if v.is_boundary}
        inner = [v for v in bm.verts if v not in hems]
        for _ in range(GARMENT_SMOOTH):
            bmesh.ops.smooth_vert(bm, verts=inner, factor=0.5, use_axis_x=True, use_axis_y=True, use_axis_z=True)
        bm.normal_update()
        for v in bm.verts:
            flare = HEM_FLARE * (1.0 - min(1.0, hem(v.co) / 0.06))
            # The shorts' waistband hugs the body (two thirds of their distance), well inside the shirt's hem over it.
            snug = 0.67 if name == "Shorts" and v.co.z > waist - 0.05 else 1.0
            if name == "Shirt" and not on_arm(v.co) and v.co.z > neck_line - 0.06:
                flare = 0.0
            elif name == "Shirt" and not on_arm(v.co):
                if v.co.z < waist + BELT_HALF:
                    # Tucked in under the belt: close in, no flare.
                    snug, flare = 0.8, 0.0
                else:
                    # Puffed a little just above the belt, where the tucked cloth gathers.
                    t = (v.co.z - waist - BELT_HALF) / BLOUSE_SPAN
                    if t < 1.0:
                        flare += BLOUSE * math.sin(math.pi * max(0.0, t))
            elif name == "Shirt":
                # The sleeve's opening hugs the arm, a clean band, not a flared cone with the dark inside showing.
                a, b = arm["L" if v.co.x < 0.0 else "R"]
                near_end = 1.0 - min(1.0, abs(sleeve_t(v.co) - SLEEVE) * (b - a).length / 0.08)
                if abs(v.co.z - neck_line) > 0.06:
                    flare = 0.0
                    snug = 1.0 - (1.0 - SLEEVE_SNUG) * near_end
            v.co = v.co + v.normal * (lift * snug + flare)
            if name in ("Boot", "BootSole"):
                v.co.z = max(v.co.z, -0.004 if name == "BootSole" else 0.0)
        for v in bm.verts:
            hit = skin_tree.find_nearest(v.co)
            if hit[0] is None:
                continue
            away = (v.co - hit[0]).dot(hit[1])
            least = lift * (0.5 if name == "Shirt" else 0.8)
            if away < least:
                v.co = v.co + hit[1] * (least - away)
        # The hems themselves: held still while the cloth was smoothed, they kept every bump of the body's surface, which the flare then
        # turned into a torn-looking edge. Each hem (not the sleeves') is smoothed along its own loop, across but not up or down.
        loop = {v: [e.other_vert(v) for e in v.link_edges if e.is_boundary] for v in hems}
        sleeve_hem = {v for v in hems if on_arm(v.co)}
        for _ in range(HEM_SMOOTH):
            moved = {}
            for v, nbrs in loop.items():
                if len(nbrs) == 2:
                    mid = (nbrs[0].co + nbrs[1].co) * 0.5
                    # (A sleeve's hem is smoothed in every direction: it ends square across the arm, and its points stood out as spikes.)
                    z = v.co.z + (mid.z - v.co.z) * 0.5 if v in sleeve_hem else v.co.z
                    moved[v] = Vector((v.co.x + (mid.x - v.co.x) * 0.5, v.co.y + (mid.y - v.co.y) * 0.5, z))
            for v, co in moved.items():
                v.co = co
        # Straight hems: the edges were cut level, but pushing the cloth out along a bumpy surface's normals left them wavy (like torn
        # paper where the shirt meets the shorts). Every edge point near a level cut goes back to its height.
        levels = {"Shirt": [waist - 0.012, neck_line], "Shorts": [waist + SHORTS_TOP, shorts_end],
                  "Belt": [waist - BELT_HALF, waist + BELT_HALF], "Boot": [boot_top], "BootSole": [SOLE_HEIGHT],
                  "BootCuff": [boot_top - BOOT_CUFF, boot_top],
                  "ShortsCuff": [shorts_end, cuff_top]}[name]
        for v in hems:
            if on_arm(v.co):
                continue
            nearest = min(levels, key=lambda z: abs(v.co.z - z))
            if abs(v.co.z - nearest) < 0.04:
                v.co.z = nearest
        # Every point takes the bone weights of the nearest point of the skin (the cuts and the mirroring made new points, and an
        # unweighted point would be left behind when the body moves).
        # The left half copies the right half's weights, left and right swapped, so both sides of the garment move alike (the body's own
        # weights are not quite symmetric either).
        deform = bm.verts.layers.deform.verify()
        names = [g.name for g in skin.vertex_groups]
        index = {n: k for k, n in enumerate(names)}
        for v in bm.verts:
            left = v.co.x < 0.0
            co = Vector((-v.co.x, v.co.y, v.co.z)) if left else v.co
            # The skin's weights blurred over a few centimetres round the point (Blender's automatic weights alternate between bones
            # from one point to the next, which made a hem zigzag in a pose); the sleeves keep the nearest point's.
            if on_arm(co):
                _, i, _ = skin_points.find(co)
                near = [(skin.data.vertices[i], 1.0)]
            else:
                found = torso_points.find_range(co, WEIGHT_BLUR)
                if not found:
                    found = [torso_points.find(co)]
                near = [(skin.data.vertices[i], max(0.0, 1.0 - d / WEIGHT_BLUR) + 1e-4) for _, i, d in found]
            blend = {}
            even = None if on_arm(co) or co.z < crotch else trunk_weights(co.z)
            if even is not None:
                blend = dict(even)
                near = []
            for sv, k in near:
                for g in sv.groups:
                    blend[g.group] = blend.get(g.group, 0.0) + g.weight * k
            whole = sum(blend.values())
            v[deform].clear()
            for gi, w in blend.items():
                if whole <= 0.0 or w / whole < 0.01:
                    continue
                group = gi
                if left:
                    n = names[gi]
                    swapped = n[:-2] + (".L" if n.endswith(".R") else ".R") if n.endswith((".L", ".R")) else n
                    group = index.get(swapped, gi)
                v[deform][group] = w / whole
            if name in ("Shorts", "ShortsCuff") and v.co.z < crotch:
                # Below the crotch each leg of the shorts follows its own thigh, fully at the hem: the leg openings keep their place on
                # the thighs in any pose rather than riding up or standing off like a skirt.
                t = min(1.0, (crotch - v.co.z) / max(crotch - shorts_end, 0.01) * 1.4)
                thigh = index.get("thigh." + ("L" if v.co.x < 0.0 else "R"))
                if thigh is not None:
                    for k in list(v[deform].keys()):
                        v[deform][k] = v[deform][k] * (1.0 - t)
                    v[deform][thigh] = v[deform].get(thigh, 0.0) + t
        bm.to_mesh(obj.data)
        bm.free()
        for poly in obj.data.polygons:
            poly.use_smooth = True
        # No thickness: an inner layer crossed the outer one wherever the cloth folds (the game draws the cloth on both sides instead).
        bpy.context.view_layer.objects.active = obj
        sol = None if CLOTH_THICKNESS <= 0.0 else obj.modifiers.new("Thickness", "SOLIDIFY")
        if sol is not None:
            sol.thickness = CLOTH_THICKNESS
            sol.offset = -1.0
            sol.use_even_offset = False
            # (The solidify goes first, under the armature, so the clothes are posed with the body.)
            while obj.modifiers.find("Thickness") > 0:
                bpy.ops.object.modifier_move_up(modifier="Thickness")
            bpy.ops.object.modifier_apply(modifier="Thickness")
        # The garment's own rest position as its texture maps (the skin's were copied with it, but the cloth has since been pushed out
        # and smoothed, so a print laid by them wobbled: the stripes came out wavy and torn).
        for layer, uv_of in (("rest", lambda co: (co.x, co.z)), ("rest_depth", lambda co: (co.y, 0.0))):
            if layer in obj.data.uv_layers:
                data = obj.data.uv_layers[layer].data
                for loop in obj.data.loops:
                    data[loop.index].uv = uv_of(obj.data.vertices[loop.vertex_index].co)
        # The shirt's arm and spine weights spread smoothly over it, so its sides stretch evenly when the fists come up rather than
        # bunching where the skin's weights change quickly. Its hips weights are left as the skin's, so the hem moves with the shorts'
        # waistband; the shorts keep the skin's own weights, so each leg follows its own thigh.
        if name == "Shirt":
            smooth_weights(obj, [g for g in ("armU.L", "armU.R", "spine") if g in obj.vertex_groups], 0.5, GARMENT_WEIGHT_SMOOTH)
            release_trunk(obj, J)
            # The part tucked under the belt, and a little above it, takes the belt's own bones (the trunk's blend at that height): with
            # the spine's weights spread down to it, the hem swung out from under the belt like a skirt whenever the body leaned.
            tuck_top = waist + BELT_HALF + TUCK_SPAN
            names = [g.name for g in obj.vertex_groups]
            skin_names = [g.name for g in skin.vertex_groups]
            for v in obj.data.vertices:
                if v.co.z >= tuck_top or on_arm(v.co):
                    continue
                t = min(1.0, (tuck_top - v.co.z) / TUCK_SPAN)
                belt_w = {skin_names[gi]: w for gi, w in (trunk_weights(min(v.co.z, waist)) or {}).items()}
                now = {names[g.group]: g.weight for g in v.groups}
                for n in set(now) | set(belt_w):
                    w = now.get(n, 0.0) * (1.0 - t) + belt_w.get(n, 0.0) * t
                    group = obj.vertex_groups.get(n) or obj.vertex_groups.new(name=n)
                    if w > 1e-4:
                        group.add([v.index], w, "REPLACE")
                    elif n in now:
                        group.remove([v.index])
            # Where the sleeve meets the side the cloth pinches when the elbows come in; the ink outline is left out there (the vertex
            # colour's red is how much outline a point gets: shaders/outline.gdshader), so the folds do not turn into black scratches.
            ink = obj.data.color_attributes.new("outline", "FLOAT_COLOR", "POINT")
            for v in obj.data.vertices:
                side = 1.0 if v.co.x > 0.0 else -1.0
                pit = Vector((side * half_width, J["armU." + ("R" if side > 0 else "L")].z - 0.12))
                d = (Vector((v.co.x, v.co.z)) - pit).length
                w = min(1.0, max(0.0, (d - ARMPIT_NO_INK) / 0.08))
                ink.data[v.index].color = (w, 1.0, 1.0, 1.0)
            obj.data.color_attributes.active_color = ink
        out[name] = obj
    # The belt's buckle: a rounded brass plate on the belt's front, moving as the belt does there (the trunk's bones at the waist).
    belt = out["Belt"]
    front = min((v.co.y for v in belt.data.vertices if abs(v.co.x) < 0.04), default=-0.25)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=(0.16, 0.035, 0.115), verts=bm.verts)
    bmesh.ops.bevel(bm, geom=list(bm.edges), offset=0.014, segments=3, affect="EDGES", profile=0.5)
    bmesh.ops.translate(bm, vec=(0.0, front - 0.006, waist), verts=bm.verts)
    buckle = belt.copy()
    buckle.data = bpy.data.meshes.new("Buckle")
    buckle.name = "Buckle"
    bpy.context.collection.objects.link(buckle)
    bm.to_mesh(buckle.data)
    bm.free()
    for poly in buckle.data.polygons:
        poly.use_smooth = False
    for layer in list(buckle.data.color_attributes):
        buckle.data.color_attributes.remove(layer)
    every = list(range(len(buckle.data.vertices)))
    names = [g.name for g in skin.vertex_groups]
    for gi, w in (trunk_weights(waist) or {}).items():
        group = buckle.vertex_groups.get(names[gi]) or buckle.vertex_groups.new(name=names[gi])
        group.add(every, w, "REPLACE")
    out["Buckle"] = buckle
    out.update(shirt_front(out["Shirt"], skin, neck_line, waist, trunk_weights))
    out.update(shorts_pockets(out["Shorts"], skin, waist, crotch))
    # The skin's marks for the game's outline (shaders/outline.gdshader): green is how much line a point gets under the shirt, blue under
    # the shorts (none deep under a garment, all of it near a hem), red always all.
    cover = skin.data.color_attributes.new("outline", "FLOAT_COLOR", "POINT")
    for v in skin.data.vertices:
        g = 1.0 - min(1.0, max(0.0, (shirt_hem(v.co) - 0.02) / 0.03)) if in_shirt(v.co) else 1.0
        if in_shirt(v.co) and on_arm(v.co):
            # (None under a sleeve, to its very hem: the arm's line showed through the opening as a black wedge; the sleeve's own edge
            # draws the line there.)
            g = 0.0
        b = 1.0 - min(1.0, max(0.0, (shorts_hem(v.co) - 0.02) / 0.03)) if in_shorts(v.co) else 1.0
        cover.data[v.index].color = (1.0, g, b, 1.0)
    skin.data.color_attributes.active_color = cover
    return out


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
            # A slower breath under the bounce (one a cycle): the chest rises and turns a touch, so the loop never reads as one repeated
            # step.
            breath = math.sin(k * 2.0 * math.pi)
            # On the base body the feet are planted (`plant`), which takes over the hips' own drop, so the bob has to come from the knees:
            # they flex twice as far.
            flex = 2.0 if BODY else 1.0
            pose(rig, "hips", lift=-0.06 - 0.04 * down, twist=-6.0 + 1.5 * breath)
            pose(rig, "thigh.L", fwd=32.0 + 6.0 * down * flex, out=15.0)
            pose(rig, "shin.L", fwd=-40.0 - 14.0 * down * flex)
            pose(rig, "foot.L", fwd=6.0 + 6.0 * down, out=-10.0)
            # The back leg stays nearly straight (a bent back knee would cross the front leg on screen).
            pose(rig, "thigh.R", fwd=-14.0 + 3.0 * down * flex, out=15.0)
            pose(rig, "shin.R", fwd=-10.0 - 6.0 * down * flex)
            pose(rig, "foot.R", fwd=18.0, out=-10.0)
            # The chest sinks into each bob; the head keeps nearly level over it, settling a beat late.
            settle = math.cos(k * 4.0 * math.pi - 0.6)
            pose(rig, "spine", fwd=9.0 + 4.0 * down * flex - 2.0 * breath, twist=8.0 + 2.0 * b + 2.0 * breath)
            pose(rig, "head", fwd=-7.0 + 4.0 * down - (2.5 * (0.5 - 0.5 * settle) if BODY else 0.0), twist=-4.0 - 1.5 * breath)
            # Lead fist (far arm) out in front, rear fist guarding the chin; both ride the bounce a beat behind.
            lag = math.cos(k * 4.0 * math.pi - 0.8)
            swing = 1.8 if BODY else 1.0
            # (The base body's arms are thick and fully jointed: its elbows bend less, the fists held a little lower.)
            elbow = 0.72 if BODY else 1.0
            # (After the owner's PEAK reference: the chunky arms held out clear of the body, so both read against the background, not
            # tucked into a guard in front of the belly where the body and the weapon hid them; fists still a little up and forward.)
            # (Forward more than out, 2026-10-14: held mostly out to the side, the rear arm pointed backward once the chest turns to the
            # camera, and its forearm folded down behind the body; the owner saw it bend behind him.)
            pose(rig, "armU.L", fwd=40.0 + 5.0 * lag * swing, out=24.0)
            pose(rig, "armL.L", fwd=(64.0 - 6.0 * lag * swing) * elbow)
            pose(rig, "hand.L", fwd=-12.0)
            pose(rig, "armU.R", fwd=36.0 + 5.0 * lag * swing, out=22.0)
            pose(rig, "armL.R", fwd=(70.0 - 6.0 * lag * swing) * elbow)
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
    clip(rig, "run", 20, {i: run_pose(i / 20.0) for i in range(0, 21)})
    # The initial dash: the same stride, launched lower and further forward, arms flung harder.
    clip(rig, "dash", 16, {i: run_pose(i / 16.0, lean=32.0, stride=1.1, arms=1.15, bounce=0.06) for i in range(0, 17)})

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
            if BODY:
                # The far fist is thrown up overhead and the near arm (the left, nearest the camera once the game turns the chest to it)
                # swings down and back, so the arms make one long diagonal through the body and neither crosses the face: the base body's
                # thick near arm raised in front hid it.
                pose(rig, "armU.R", fwd=30.0 + 110.0 * tuck, out=24.0 + 6.0 * tuck)
                pose(rig, "armL.R", fwd=40.0 - 12.0 * tuck)
                pose(rig, "armU.L", fwd=-20.0 - 34.0 * tuck, out=36.0)
                pose(rig, "armL.L", fwd=24.0)
            else:
                pose(rig, "armU.L", fwd=40.0 + 56.0 * tuck, out=24.0)
                pose(rig, "armL.L", fwd=40.0)
                pose(rig, "armU.R", fwd=-20.0 - 24.0 * tuck, out=42.0)
                pose(rig, "armL.R", fwd=30.0)
            pose(rig, "spine", fwd=4.0 - 10.0 * tuck, twist=6.0 * tuck)
            pose(rig, "head", fwd=-6.0 * tuck)
        return f

    if BODY:
        # Up out of the crouch: stretched long on the push-off, the knee driven up, then easing open toward the top of the arc (where the
        # fall takes over), so the rise keeps moving instead of freezing in one pose.
        clip(rig, "jump", 30, {0: amped(jump(0.0), 1.1), 5: amped(jump(1.0), 1.08), 14: jump(0.85), 30: jump(0.62)})
    else:
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

    # The jump's take-off crouch (anticipation, a few frames): the knees load deep, the chest leans into it and both arms swing down and
    # back, so the jump that follows throws them up through the push-off.
    def jumpsquat(k):
        def f(rig):
            pose(rig, "hips", lift=-0.08 - 0.14 * k)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=24.0 + 42.0 * k, out=8.0)
                pose(rig, "shin." + side, fwd=-36.0 - 66.0 * k)
                pose(rig, "foot." + side, fwd=12.0 + 20.0 * k)
                pose(rig, "armU." + side, fwd=10.0 - 46.0 * k, out=18.0 + 6.0 * k)
                pose(rig, "armL." + side, fwd=30.0 - 10.0 * k)
            pose(rig, "spine", fwd=8.0 + 18.0 * k)
            pose(rig, "head", fwd=-4.0 - 12.0 * k)
        return f

    clip(rig, "jumpsquat", 4, {0: jumpsquat(0.3), 4: jumpsquat(1.0)})

    # Landing: the weight drops onto bent knees, the chest pitched forward and the arms flung out and down for balance, then a small
    # rebound and a settle back up toward standing (the game adds its squash on top, deeper after an aerial).
    def land(k, rebound=0.0):
        def f(rig):
            pose(rig, "hips", lift=-0.24 * k)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=20.0 + 50.0 * k, out=10.0)
                pose(rig, "shin." + side, fwd=-30.0 - 78.0 * k)
                pose(rig, "foot." + side, fwd=10.0 + 26.0 * k)
                pose(rig, "armU." + side, fwd=20.0 + 16.0 * k - 10.0 * rebound, out=24.0 + 34.0 * k)
                pose(rig, "armL." + side, fwd=26.0)
            pose(rig, "spine", fwd=6.0 + 22.0 * k - 4.0 * rebound)
            pose(rig, "head", fwd=-4.0 - 10.0 * k + 6.0 * rebound)
        return f

    clip(rig, "land", 14, {0: land(1.0), 3: amped(land(1.0), 1.06), 8: land(0.45, rebound=1.0), 14: land(0.3)})

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

    # The base body's hit reaction: on the hit (held through the hitlag freeze) the chest is thrown back and twisted off the blow, the head
    # snapped back, the arms flung forward and out as the body leaves them behind, one knee jerked up; then over the hitstun it slackens
    # into a looser reel, the arms dropping, so a long stun does not read as one frozen pose. The game's flinch, tumble and launch poses
    # go on top for stronger hits.
    def hurt_snap(rig):
        pose(rig, "armU.L", fwd=46.0, out=64.0)
        pose(rig, "armL.L", fwd=20.0)
        pose(rig, "armU.R", fwd=14.0, out=84.0)
        pose(rig, "armL.R", fwd=10.0)
        pose(rig, "thigh.R", fwd=38.0, out=12.0)
        pose(rig, "shin.R", fwd=-62.0)
        pose(rig, "thigh.L", fwd=-16.0, out=14.0)
        pose(rig, "shin.L", fwd=-20.0)
        # (Not so far back or so twisted that the face turns away: the hurt face is half the read.)
        pose(rig, "spine", fwd=-18.0, twist=-4.0)
        pose(rig, "head", fwd=2.0, twist=4.0)

    def hurt_reel(rig):
        pose(rig, "armU.L", fwd=20.0, out=58.0)
        pose(rig, "armL.L", fwd=30.0)
        pose(rig, "armU.R", fwd=-6.0, out=66.0)
        pose(rig, "armL.R", fwd=24.0)
        pose(rig, "thigh.R", fwd=18.0, out=12.0)
        pose(rig, "shin.R", fwd=-40.0)
        pose(rig, "thigh.L", fwd=-10.0, out=14.0)
        pose(rig, "shin.L", fwd=-22.0)
        pose(rig, "spine", fwd=-12.0, twist=-2.0)
        pose(rig, "head", fwd=0.0, twist=2.0)

    if BODY:
        clip(rig, "hurt", 18, {0: hurt_snap, 4: amped(hurt_snap, 1.05), 18: hurt_reel})
    else:
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
    # The claws' forward tilt, a two-handed lunging slash (posed in BODY_MOVES).
    clip(rig, "attack_claws", 60, {})
    # The Settler's tool clips (posed in BODY_MOVES).
    for tool in ("tool_reach", "tool_toss", "tool_dig", "tool_pour", "tool_chop", "tool_heave", "tool_raise", "tool_trip", "tool_shrug", "tool_ride", "tool_float"):
        clip(rig, tool, 60, {})
    # Headbutts, up and forward (posed in BODY_MOVES; the body is the weapon).
    clip(rig, "body_headbutt", 60, {})
    clip(rig, "body_headbutt_fwd", 60, {})

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
    # (After the reference's neutral airs: a held kung-fu kick, not a split. The front leg (thigh.L, the near leg facing right) shoots out
    # straight ahead, the other knee stays drawn up with its foot tucked under the seat, the body leans back a touch against the kick
    # and the arms are flung out wide for balance; the wind-up curls both knees up.)
    move_of("kick_nair",
            {"spine": (18.0, 0.0), "head": (6.0, 0.0), "stance": (64.0, 60.0), "shin.L": (-110.0,), "shin.R": (-104.0,), "arms": (34.0, 22.0)},
            {"spine": (-10.0, 0.0), "head": (2.0, 0.0), "stance": (84.0, 58.0), "shin.L": (-4.0,), "shin.R": (-122.0,),
             "foot.L": (-30.0,), "foot.R": (34.0,), "arms": (-18.0, 76.0)},
            {"spine": (-4.0, 0.0), "stance": (70.0, 56.0), "shin.L": (-20.0,), "shin.R": (-116.0,), "arms": (-8.0, 60.0)})
    # A plunging thrust (the Wanderer's and the Champion's down airs, after their references): the blade raised in both hands with the
    # knees tucked, then both knees drawn right up under the chin as the blade is driven straight down below, the body upright over it
    # and the head looking down along the blade.
    move_of("sword_plunge",
            {"spine": (-6.0, 0.0), "head": (-4.0, 0.0), "stance": (50.0, 40.0), "shin.L": (-90.0,), "shin.R": (-80.0,), "arms": (30.0, 24.0)},
            {"spine": (14.0, 0.0), "head": (18.0, 0.0), "stance": (92.0, 84.0), "shin.L": (-130.0,), "shin.R": (-124.0,), "arms": (10.0, 20.0)},
            {"spine": (8.0, 0.0), "head": (8.0, 0.0), "stance": (70.0, 60.0), "shin.L": (-110.0,), "shin.R": (-100.0,)})
    # An upward thrust (the Wanderer's up air): gathered with the knees up and the blade drawn down by the hip, then the body stretches out
    # long beneath the blade as it is driven straight up overhead, the legs hanging together and the head tipped back to follow it.
    move_of("sword_rise",
            {"spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (60.0, 54.0), "shin.L": (-100.0,), "shin.R": (-96.0,), "arms": (20.0, 20.0)},
            {"spine": (-12.0, 0.0), "head": (-8.0, 0.0), "stance": (-6.0, 4.0), "shin.L": (-14.0,), "shin.R": (-20.0,), "arms": (-24.0, 30.0)},
            {"spine": (-4.0, 0.0), "stance": (16.0, 20.0), "shin.L": (-40.0,), "shin.R": (-46.0,)})
    # The claws fighter's neutral air, after its reference: a held double kick, the body leaned far back and both legs shot out straight
    # ahead together, the arms flung back for balance; the wind-up curls both knees up to the chest.
    move_of("kick_double",
            {"spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (70.0, 66.0), "shin.L": (-116.0,), "shin.R": (-110.0,), "arms": (30.0, 22.0)},
            {"spine": (-34.0, 0.0), "head": (12.0, 0.0), "stance": (96.0, 80.0), "shin.L": (-4.0,), "shin.R": (-16.0,),
             "foot.L": (-30.0,), "foot.R": (-24.0,), "arms": (-40.0, 64.0)},
            {"spine": (-14.0, 0.0), "stance": (76.0, 66.0), "shin.L": (-30.0,), "shin.R": (-40.0,), "arms": (-16.0, 50.0)})
    # Back air: lean forward and drive the heel back (not so far forward, nor twisted, that the big head shows its crown).
    move_of("kick_bair",
            {"spine": (24.0, 10.0), "head": (10.0, 20.0), "stance": (60.0, 50.0), "shin.L": (-90.0,), "shin.R": (-90.0,), "arms": (40.0, 20.0)},
            {"spine": (14.0, -10.0), "head": (-6.0, 10.0), "stance": (50.0, -20.0), "shin.L": (-80.0,), "arms": (60.0, 30.0)},
            {"spine": (10.0, 0.0), "stance": (30.0, 0.0)})
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
    # Gauntlets (class 4), after the reference's moves. thigh.L is the front leg (nearest the camera facing right).
    # The knee (forward air, jab 3): curled up tight, then the front knee driven forward and up, the shin folded hard under it so the knee
    # leads; the back leg trails long behind for balance, the body leans back over it and both fists are thrown back.
    move_of("gaunt_knee",
            {"spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (60.0, 40.0), "shin.L": (-120.0,), "shin.R": (-100.0,), "arms": (40.0, 22.0)},
            {"spine": (-14.0, 0.0), "head": (2.0, 0.0), "stance": (112.0, -34.0), "shin.L": (-140.0,), "shin.R": (-36.0,),
             "foot.L": (40.0,), "foot.R": (-30.0,), "arms": (-52.0, 40.0)},
            {"spine": (-4.0, 0.0), "stance": (80.0, -10.0), "shin.L": (-120.0,), "shin.R": (-50.0,), "arms": (-20.0, 34.0)})
    # The stomp (down air): both knees pulled up to the chest, then both legs driven straight down together, the feet flat, the body
    # pitched forward over them and the arms flung up and out (the recoil).
    move_of("gaunt_stomp",
            {"spine": (14.0, 0.0), "head": (-4.0, 0.0), "stance": (86.0, 86.0), "shin.L": (-124.0,), "shin.R": (-124.0,), "arms": (40.0, 26.0)},
            {"spine": (18.0, 0.0), "head": (-6.0, 0.0), "stance": (-6.0, -6.0), "shin.L": (-2.0,), "shin.R": (-2.0,),
             "foot.L": (-20.0,), "foot.R": (-20.0,), "arms": (-60.0, 70.0)},
            {"spine": (10.0, 0.0), "stance": (20.0, 20.0), "shin.L": (-30.0,), "shin.R": (-30.0,), "arms": (-20.0, 60.0)})
    # The axe kick (up tilt): the weight rocks back onto the planted back leg, the front leg swings straight up overhead (the game aims
    # the foot at the heel's hitbox and brings it down in front), the arms spread wide; the body leans back under the raised leg.
    move_of("kick_axe",
            {"hips": -0.06, "spine": (12.0, 0.0), "stance": (20.0, -8.0), "shin.L": (-40.0,), "shin.R": (-20.0,), "arms": (30.0, 30.0)},
            {"hips": 0.02, "spine": (-16.0, 0.0), "head": (4.0, 0.0), "stance": (150.0, -6.0), "shin.L": (-4.0,), "shin.R": (-6.0,),
             "arms": (-30.0, 74.0)},
            {"hips": -0.08, "spine": (18.0, 0.0), "stance": (40.0, -10.0), "shin.L": (-30.0,), "arms": (10.0, 50.0)})
    # The shoulder charge (dash attack): dropped low and turned side-on, the lead shoulder driven forward with the fists tucked to the
    # chest, the back leg pushing off long behind.
    move_of("gaunt_shoulder",
            {"hips": -0.12, "spine": (24.0, -20.0), "head": (-10.0, 10.0), "stance": (44.0, -20.0), "shin.L": (-70.0,), "arms": (50.0, 10.0)},
            {"hips": -0.16, "spine": (34.0, 36.0), "head": (-18.0, -24.0), "stance": (58.0, -52.0), "shin.L": (-80.0,), "shin.R": (-6.0,),
             "armL.L": (120.0,), "armL.R": (120.0,), "arms": (64.0, 6.0)},
            {"hips": -0.08, "spine": (18.0, 14.0), "stance": (36.0, -28.0), "arms": (40.0, 16.0)})
    # The elbow (forward smash): wound right round, the weight on the back leg and the elbow drawn back; then a big step in, the hips
    # and shoulders turning through and the elbow driven ahead at chest height, the other arm whipped back.
    move_of("gaunt_elbow",
            {"hips": -0.1, "spine": (-6.0, -44.0), "head": (4.0, 30.0), "stance": (16.0, -16.0), "shin.R": (-40.0,), "arms": (-30.0, 30.0)},
            {"hips": -0.14, "spine": (26.0, 46.0), "head": (-14.0, -30.0), "stance": (64.0, -46.0), "shin.L": (-60.0,), "shin.R": (-4.0,),
             "armL.R": (130.0,), "arms": (-40.0, 40.0)},
            {"hips": -0.08, "spine": (12.0, 20.0), "stance": (40.0, -24.0), "arms": (10.0, 26.0)})
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

    if BODY:
        # The dodges, played by progress through the simulation's own frames (the game spins the roll and steps the spot dodge
        # back into the stage). A roll: a quick crouch into a tight ball for the somersault, then opening out onto the feet.
        def roll_launch(rig):
            pose(rig, "hips", lift=-0.16)
            pose(rig, "spine", fwd=34.0)
            pose(rig, "head", fwd=-10.0)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=56.0, out=6.0)
                pose(rig, "shin." + side, fwd=-90.0)
                pose(rig, "armU." + side, fwd=60.0, out=10.0)
                pose(rig, "armL." + side, fwd=60.0)

        def roll_land(rig):
            pose(rig, "hips", lift=-0.12)
            pose(rig, "spine", fwd=16.0)
            pose(rig, "head", fwd=-8.0)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=46.0, out=8.0)
                pose(rig, "shin." + side, fwd=-70.0)
                pose(rig, "armU." + side, fwd=30.0, out=24.0)
                pose(rig, "armL." + side, fwd=50.0)

        clip(rig, "roll", 30, {0: roll_launch, 4: roll_pose, 21: roll_pose, 27: roll_land, 30: roll_land})

        # A spot dodge: a quick duck and tuck in place, arms drawn in to the chest and the knees bent, held while it is safe, then
        # back up (the game turns the chest away and steps the body back into the stage).
        def duck(k):
            def f(rig):
                pose(rig, "hips", lift=-0.18 * k)
                pose(rig, "spine", fwd=6.0 + 14.0 * k)
                pose(rig, "head", fwd=-4.0 - 6.0 * k)
                for side in "LR":
                    pose(rig, "thigh." + side, fwd=22.0 + 34.0 * k, out=8.0)
                    pose(rig, "shin." + side, fwd=-30.0 - 50.0 * k)
                    pose(rig, "armU." + side, fwd=30.0 + 34.0 * k, out=20.0 - 14.0 * k)
                    pose(rig, "armL." + side, fwd=60.0 + 60.0 * k)
            return f

        clip(rig, "spot_dodge", 30, {0: duck(0.2), 5: amped(duck(1.0), 1.05), 22: duck(1.0), 30: duck(0.2)})

        # An air dodge: a flinch into a tight curl, knees up and arms crossed before the face (not over it: the hands stay under the
        # chin), held through the dodge, then opening into the fall.
        def curl(k):
            def f(rig):
                pose(rig, "spine", fwd=4.0 + 18.0 * k)
                pose(rig, "head", fwd=-4.0 - 4.0 * k)
                for side in "LR":
                    pose(rig, "thigh." + side, fwd=20.0 + 50.0 * k, out=10.0)
                    pose(rig, "shin." + side, fwd=-30.0 - 66.0 * k)
                    pose(rig, "armU." + side, fwd=30.0 + 34.0 * k, out=30.0 - 34.0 * k)
                    pose(rig, "armL." + side, fwd=50.0 + 60.0 * k)
            return f

        clip(rig, "air_dodge", 30, {0: curl(0.3), 4: amped(curl(1.0), 1.06), 20: curl(1.0), 30: fall(0.0)})
    else:
        clip(rig, "roll", 8, {0: roll_pose, 8: roll_pose})

    def down_pose(rig):
        for side in "LR":
            pose(rig, "armU." + side, fwd=-12.0, out=48.0)
            pose(rig, "armL." + side, fwd=6.0)
            pose(rig, "thigh." + side, fwd=-6.0, out=16.0)
            pose(rig, "shin." + side, fwd=-8.0)
        pose(rig, "head", fwd=-8.0)

    if BODY:
        # Knocked down (the game lays the body face down with its lean): the slam arches it, head and heels up and the arms flung out
        # ahead, a small bounce, then it lies flat.
        def slam(rig):
            pose(rig, "spine", fwd=-20.0)
            pose(rig, "head", fwd=-22.0)
            # (Lying face down, "ahead along the floor" is overhead for the arms: raised forward they went into the floor.)
            for side in "LR":
                pose(rig, "armU." + side, fwd=172.0, out=34.0)
                pose(rig, "armL." + side, fwd=-14.0)
                pose(rig, "thigh." + side, fwd=-22.0, out=12.0)
                pose(rig, "shin." + side, fwd=-64.0)

        def flat(rig):
            pose(rig, "spine", fwd=0.0)
            pose(rig, "head", fwd=6.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=166.0, out=52.0)
                pose(rig, "armL." + side, fwd=-8.0)
                pose(rig, "thigh." + side, fwd=-6.0, out=14.0)
                pose(rig, "shin." + side, fwd=-12.0)

        clip(rig, "knockdown", 14, {0: slam, 4: amped(slam, 1.1), 9: flat, 14: flat})

        # Standing up from it (played by progress; the game swings the body upright over its first half): hands planted and pushing
        # up, the knees drawn under, a crouch, then standing.
        def push_up(rig):
            pose(rig, "spine", fwd=24.0)
            pose(rig, "head", fwd=-16.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=72.0, out=18.0)
                pose(rig, "armL." + side, fwd=16.0)
                pose(rig, "thigh." + side, fwd=92.0, out=10.0)
                pose(rig, "shin." + side, fwd=-116.0)

        def rise(rig):
            pose(rig, "hips", lift=-0.16)
            pose(rig, "spine", fwd=22.0)
            pose(rig, "head", fwd=-10.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=34.0, out=22.0)
                pose(rig, "armL." + side, fwd=40.0)
                pose(rig, "thigh." + side, fwd=66.0, out=8.0)
                pose(rig, "shin." + side, fwd=-98.0)

        clip(rig, "getup", 30, {0: flat, 9: push_up, 18: rise, 30: idle(0.0)})

        # Motion polish phase 3 (docs/MOTION_POLISH_PLAN.md): the clips the inventory was missing.

        # A tech in place (played by progress over its short landing lag): slapped down low with both hands to the floor, then popped
        # straight up into the stance, a little over-stretched on the way.
        def tech_slap(rig):
            pose(rig, "hips", lift=-0.22)
            pose(rig, "spine", fwd=34.0)
            pose(rig, "head", fwd=-22.0)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=70.0, out=16.0)
                pose(rig, "shin." + side, fwd=-104.0)
                pose(rig, "foot." + side, fwd=30.0)
                pose(rig, "armU." + side, fwd=22.0, out=46.0)
                pose(rig, "armL." + side, fwd=8.0)

        def tech_pop(rig):
            pose(rig, "hips", lift=0.02)
            pose(rig, "spine", fwd=-6.0)
            pose(rig, "head", fwd=-2.0)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=8.0, out=10.0)
                pose(rig, "shin." + side, fwd=-10.0)
                pose(rig, "armU." + side, fwd=40.0, out=58.0)
                pose(rig, "armL." + side, fwd=50.0)

        clip(rig, "tech", 12, {0: tech_slap, 3: amped(tech_slap, 1.06), 7: tech_pop, 12: idle(0.0)})

        # Dropping through a platform (by progress over the first frames of the drop): a hop down out of a crouch, tipping forward, the
        # knees tucked and the arms thrown up as the body falls away, then opening into the fall.
        def drop_tip(rig):
            pose(rig, "spine", fwd=26.0)
            pose(rig, "head", fwd=-14.0)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=60.0, out=10.0)
                pose(rig, "shin." + side, fwd=-92.0)
                pose(rig, "foot." + side, fwd=24.0)
                pose(rig, "armU." + side, fwd=20.0, out=30.0)
                pose(rig, "armL." + side, fwd=30.0)

        def drop_away(rig):
            pose(rig, "spine", fwd=8.0)
            pose(rig, "head", fwd=-6.0)
            for side, sign in (("L", 1.0), ("R", -1.0)):
                pose(rig, "thigh." + side, fwd=34.0 + 10.0 * sign, out=12.0)
                pose(rig, "shin." + side, fwd=-70.0)
                pose(rig, "armU." + side, fwd=96.0, out=58.0)
                pose(rig, "armL." + side, fwd=20.0)

        clip(rig, "platform_drop", 20, {0: drop_tip, 8: drop_away, 20: fall(0.0)})

        # A run stopping: braced and leaning back against the slide, the front heel dug in and the arms flung back, a rebound forward
        # over the feet, then settling into the stance.
        def stop_brace(rig):
            pose(rig, "hips", lift=-0.12)
            pose(rig, "spine", fwd=-18.0)
            pose(rig, "head", fwd=10.0)
            pose(rig, "thigh.L", fwd=44.0, out=8.0)
            pose(rig, "shin.L", fwd=-8.0)
            pose(rig, "foot.L", fwd=-18.0)
            pose(rig, "thigh.R", fwd=-16.0, out=8.0)
            pose(rig, "shin.R", fwd=-62.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=-46.0, out=56.0)
                pose(rig, "armL." + side, fwd=24.0)

        def stop_rebound(rig):
            pose(rig, "hips", lift=-0.08)
            pose(rig, "spine", fwd=20.0)
            pose(rig, "head", fwd=-12.0)
            for side in "LR":
                pose(rig, "thigh." + side, fwd=34.0, out=12.0)
                pose(rig, "shin." + side, fwd=-50.0)
                pose(rig, "armU." + side, fwd=46.0, out=34.0)
                pose(rig, "armL." + side, fwd=60.0)

        clip(rig, "run_stop", 20, {0: stop_brace, 4: amped(stop_brace, 1.08), 11: stop_rebound, 20: idle(0.0)})

        # Hitstun from a hit strong enough to send the fighter flying but not to tumble: folded round the blow (the game turns it to
        # face the hit), the arms and legs left trailing toward the hitter, the head pitched down over it but still showing the face.
        def heavy_fold(rig):
            pose(rig, "spine", fwd=30.0, twist=-4.0)
            pose(rig, "head", fwd=-8.0, twist=4.0)
            pose(rig, "armU.L", fwd=84.0, out=40.0)
            pose(rig, "armL.L", fwd=12.0)
            pose(rig, "armU.R", fwd=62.0, out=62.0)
            pose(rig, "armL.R", fwd=20.0)
            pose(rig, "thigh.L", fwd=52.0, out=14.0)
            pose(rig, "shin.L", fwd=-30.0)
            pose(rig, "thigh.R", fwd=30.0, out=16.0)
            pose(rig, "shin.R", fwd=-56.0)

        def heavy_drift(rig):
            pose(rig, "spine", fwd=18.0, twist=-2.0)
            pose(rig, "head", fwd=-4.0, twist=2.0)
            pose(rig, "armU.L", fwd=60.0, out=56.0)
            pose(rig, "armL.L", fwd=24.0)
            pose(rig, "armU.R", fwd=40.0, out=70.0)
            pose(rig, "armL.R", fwd=26.0)
            pose(rig, "thigh.L", fwd=36.0, out=14.0)
            pose(rig, "shin.L", fwd=-44.0)
            pose(rig, "thigh.R", fwd=14.0, out=16.0)
            pose(rig, "shin.R", fwd=-60.0)

        clip(rig, "hurt_heavy", 24, {0: heavy_fold, 4: amped(heavy_fold, 1.06), 24: heavy_drift})

        # The same, sent upward: the chest arched up off the blow, the arms and legs flung down and out below the body, the face up.
        def up_arch(rig):
            pose(rig, "spine", fwd=-26.0)
            pose(rig, "head", fwd=4.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=-24.0, out=40.0)
                pose(rig, "armL." + side, fwd=14.0)
                pose(rig, "thigh." + side, fwd=-18.0, out=18.0)
                pose(rig, "shin." + side, fwd=-38.0)
                pose(rig, "foot." + side, fwd=-20.0)

        def up_drift(rig):
            pose(rig, "spine", fwd=-14.0)
            pose(rig, "head", fwd=2.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=4.0, out=58.0)
                pose(rig, "armL." + side, fwd=26.0)
                pose(rig, "thigh." + side, fwd=6.0, out=16.0)
                pose(rig, "shin." + side, fwd=-46.0)

        clip(rig, "hurt_up", 24, {0: up_arch, 4: amped(up_arch, 1.06), 24: up_drift})

        # A rare fidget after a long stillness (the game plays it now and then in a long idle): from the stance, a glance off toward the
        # camera side and back, the weight shuffled from foot to foot, a roll of the shoulders, back into the stance.
        def glance(turn, shift):
            def f(rig):
                idle(0.0)(rig)
                pose(rig, "hips", lift=-0.06, twist=-6.0 + 4.0 * shift)
                pose(rig, "spine", fwd=8.0, twist=8.0 + 10.0 * turn)
                pose(rig, "head", fwd=-4.0, twist=-4.0 + 26.0 * turn)
            return f

        clip(rig, "idle_look", 72, {0: idle(0.0), 12: glance(1.0, 0.5), 30: glance(1.0, 1.0), 42: glance(-0.6, 0.0),
                                    60: glance(-0.6, -0.5), 72: idle(0.0)})
    else:
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
        export_vertex_color="ACTIVE",
    )
    print("wrote", OUT, "clips:", [a.name for a in bpy.data.actions])


main()
