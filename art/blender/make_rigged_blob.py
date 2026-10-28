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
    # (Faces turned outward on the hands: the glove was built inside out, so the game drew its ink hull over the palm as a dark oval,
    # the body rebuild, 2026-10-18.)
    if name.startswith(("Hand", "Mitt", "Fist")):
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
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
# How far round the head the face shell reaches, as the head's depth (0 is its side, -1 its front, positive past the side). The drawing
# slides toward the camera on a turned head (face.gdshader `face_slide`, up to 0.5 rad), which carries the near eye's outer edge to
# about 78 degrees round; a shell cut at 72 (its rim sunk under the skin to about 65) cut that eye's white off square. Past the side,
# the straight-on texture map folds back, but within 0.1 of the depth the fold stays outside the drawing.
FACE_WRAP = 0.1
# How the face shell's depth is kept in its colour (8 bits a channel in the game): the lowest depth and the span, in head radii.
FACE_DEPTH_CODE = (-0.25, 1.5)


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


def _hand_frame(x):
    """The `x` side's wrist and the hand's own axes there: `down` straight on along the forearm (so the wrist never bends), `front` the
    character's front squared to it (the thumb's side), `out` away from the body (the back of the hand; the palm faces in)."""
    w = Vector((x * BLOB_WRIST[0], BLOB_WRIST[1], BLOB_WRIST[2]))
    down = (w - Vector((x * 0.66, -0.02, 0.95))).normalized()
    front = Vector((0.0, -1.0, 0.0))
    front = (front - down * front.dot(down)).normalized()
    out = front.cross(down)
    if out.x * x < 0.0:
        out = -out
    return w, down, front, out


def _at(x, d, f, o):
    """The point `d` down the forearm's line from the `x` side's wrist, `f` toward the front and `o` outward (hand units, grown by
    HAND_SCALE)."""
    w, down, front, out = _hand_frame(x)
    return w + (down * d + front * f + out * o) * HAND_SCALE


def _mitt_block(bm, x, d, radii, squareness=0.78, curl=0.0, segments=32, rings=22):
    """A soft rounded block (a ball pushed toward a box, `squareness` < 1) centred `d` down the forearm's line on the `x` side, with half
    sizes `radii` (down, front, out); `curl` bends its far end in toward the palm. Returns its verts."""
    w, down, front, out = _hand_frame(x)
    res = bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=1.0, calc_uvs=True)
    for v in res["verts"]:
        c = [math.copysign(abs(q) ** squareness, q) for q in v.co]
        t = (1.0 - c[2]) / 2.0    # 0 at the wrist end, 1 at the far end
        p = down * (d + radii[0] * -c[2]) + front * (radii[1] * c[1]) + out * (radii[2] * c[0] - curl * t * t)
        v.co = w + p * HAND_SCALE
    return res["verts"]


def _crease(verts, x, d, width, depth, side=1.0):
    """A faint line pressed into a block across the hand at `d` down it, on the back (`side` 1) or the palm (`side` -1)."""
    w, down, front, out = _hand_frame(x)
    for v in verts:
        rel = (v.co - w) / HAND_SCALE
        o = rel.dot(out) * side
        if o <= 0.0:
            continue
        k = math.exp(-((rel.dot(down) - d) / width) ** 2) * min(1.0, o / 0.04)
        v.co -= out * side * depth * k * HAND_SCALE


def _fingers(verts, x, d_from, centre_d, depth, width=0.012, splits=(-0.052, 0.0, 0.052)):
    """Grooves pressed in along the hand from `d_from` down it to its end, between four fingers held together (at `splits` across its
    front), the surface drawn in toward the block's middle (`centre_d` down the forearm's line), deepest at the fingertips, so the end of
    the hand reads as four fingertips, not one paddle (the body rebuild, 2026-10-18, the owner after the reference cast's hands)."""
    w, down, front, out = _hand_frame(x)
    for v in verts:
        rel = (v.co - w) / HAND_SCALE
        dd = rel.dot(down)
        if dd <= d_from:
            continue
        along = min(1.0, (dd - d_from) / 0.04)
        fr = rel.dot(front)
        k = max(math.exp(-((fr - s0) / width) ** 2) for s0 in splits) * along
        if k < 0.01:
            continue
        towards = (down * centre_d - rel)
        towards -= front * towards.dot(front)
        if towards.length > 1e-6:
            v.co += towards.normalized() * depth * k * HAND_SCALE


# The hands' resolution (fist and mitten below): halved in the cloth clean-up, 2026-10-19. Each hand was about 4,700 to 5,300 triangles,
# nearly the whole body's 5,000, for a shape about 30 pixels across in a match; the hand sheet (tests/hand_sheet.gd) showed no new facets.


def fist(bm, x):
    """A closed fist on the `x` side, straight on from the forearm (the owner, 2026-10-19: a ball with a thumb on its side did not read as a
    fist): a squarer palm block, four fingers each a thick roll curling from its knuckle over the end of the hand and back under the
    palm, set side by side so the curled fingers read as a stack of rolls, and the thumb folded across the front of the curled index and
    middle fingers. About the forearm's width and a little more."""
    _mitt_block(bm, x, 0.095, (0.095, 0.115, 0.085), squareness=0.5, segments=26, rings=18)
    for k, f0 in enumerate((0.07, 0.024, -0.022, -0.066)):
        r = 0.036 if k < 3 else 0.031
        reach = 1.0 if k in (1, 2) else 0.94
        tube(bm, [_at(x, 0.15, f0, 0.06), _at(x, 0.2 * reach + 0.01, f0, 0.035), _at(x, 0.215 * reach + 0.005, f0, -0.01),
                  _at(x, 0.19 * reach, f0, -0.06), _at(x, 0.15, f0, -0.075)],
             [r, r, r, r, r * 0.9], segments=10, per_span=3)
    # The thumb: from the heel of the hand on its front side, folded across the curled fingers' middle joints.
    tube(bm, [_at(x, 0.05, 0.115, -0.03), _at(x, 0.1, 0.12, -0.075), _at(x, 0.155, 0.075, -0.1), _at(x, 0.175, 0.02, -0.1)],
         [0.042, 0.04, 0.036, 0.03], segments=10, per_span=3)


def mitten(bm, x):
    """An open hand on the `x` side, built round the same wrist as `fist`, straight on from the forearm, after the owner's reference cast's
    soft chunky hands (the character judge's round 12: a flat paddle with grooves read as a cut-out): a palm pad with real thickness, four
    separate fingers growing out of it that taper to rounded tips, each curled a little toward the palm and fanned at its own angle and
    length, and a fat thumb set low on the heel of the palm, turned forward."""
    # (The hand pass, 2026-10-19, the owner's references: their open hands show four clearly separate fingers and a thumb standing off
    # the hand; ours were a mitten, the fingers fat tubes overlapping one another and mostly buried in the palm. The palm is shorter,
    # the fingers slimmer, set apart so a gap shows between each, and longer out of the palm; the hand's whole length is about as before.)
    _mitt_block(bm, x, 0.085, (0.088, 0.112, 0.074), squareness=0.82, curl=0.02, segments=28, rings=18)
    # (front offset across the hand, length, spread) per finger: index to little.
    # (All four out, evenly curled, round 19: unevenly curled, from the front the hand read as pointing.)
    # (Relaxed since the idle-hand pass, 2026-10-20, the owner's pick of three mocks: fanned apart, slim and curled at the tips, the open
    # hand read as a claw at rest. The fingers lie close together, a little fuller and shorter, curled further in toward the palm.)
    for k_f, (f0, ln, spread) in enumerate(((0.07, 0.92, 0.014), (0.024, 1.0, 0.004), (-0.022, 0.97, -0.004), (-0.066, 0.84, -0.014))):
        pts, radii = [], []
        for k, t in enumerate((0.0, 0.45, 0.8, 1.0)):
            d = 0.115 + 0.1 * ln * t
            f = f0 + spread * t
            o = -0.004 - 0.045 * t * t      # curling toward the palm, evenly
            pts.append(_at(x, d, f, o))
            radii.append((0.038, 0.037, 0.034, 0.03)[k])
        tube(bm, pts, radii, segments=10, per_span=3)
    # The thumb: thicker than a finger, set low on the heel of the palm and turned well out to the front, so it reads as its own digit.
    # (Closer in to the hand since the idle-hand pass, turned a little toward the palm.)
    tube(bm, [_at(x, 0.035, 0.08, -0.02), _at(x, 0.075, 0.12, -0.045), _at(x, 0.105, 0.14, -0.06), _at(x, 0.125, 0.145, -0.07)],
         [0.042, 0.04, 0.036, 0.03], segments=12, per_span=3)


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


def _attack_clip(name):
    return name.startswith(("attack", "sword_", "kick_", "gaunt_", "body_"))


def pose(rig, bone, fwd=0.0, out=0.0, twist=0.0, lift=0.0):
    """Sets a bone's pose from forward / outward / twist degrees (and an offset for the hips: upward on the base body; on the blob, whose
    clips were tuned with it, along the hip bone's own z, which is backward and forward)."""
    fwd, out, twist, lift = fwd * AMP, out * AMP, twist * AMP, lift * AMP
    # (On the base body the game aims the arm nearest the camera, the left facing right, at the hit: weapon or fist. An attack clip's
    # posed arms go to the other sides, so the free arm's counterbalance is the far arm and the near one is left to the game. The game
    # mirrors every clip facing left. Art review 5.)
    if BODY and CLIP_NOW and _attack_clip(CLIP_NOW) and bone.startswith(("armU.", "armL.", "hand.")):
        bone = bone[:-1] + ("R" if bone.endswith("L") else "L")
    # (Exaggeration multiplies angles, so an arm already raised high would be pushed right round behind the body, and an elbow folded
    # through itself: the shoulder and the elbow stay within their reach.)
    if BODY and bone.startswith("armU."):
        fwd = clampf(fwd, -100.0, 172.0)
    if BODY and bone.startswith("armL."):
        fwd = clampf(fwd, 0.0, 145.0)   # (a hinge: folds forward only, never past straight)
    # (A planted move's thigh stays in a lunge's range: exaggerated, a lunge's front thigh rose past level and folded the belly over it.
    # Art review 5. Kicks and aerials keep their reach.)
    # (Not the runs, round 3: their feet are solved onto authored paths (`run_style`), and the knee drive and the heel kick need the range.)
    if BODY and bone.startswith("thigh.") and CLIP_NOW and not CLIP_NOW.startswith(("kick", "run")) and grounded(CLIP_NOW):
        fwd = clampf(fwd, -50.0, 75.0)
    # (In an attack the arms stay in the fight, as the reference fighters' do: an elbow never flung straight, an arm never thrown far behind
    # the body. The owner: arms bent and flailed in ways out of place in combat. Art review 5.)
    # (Tighter (the owner, 2026-10-17, after the reference's sword swings and forward airs): the arm the clip poses, the free one, stays
    # tucked in close to the body, the elbow well bent and the fist by the chest, never flung out wide or back.)
    if BODY and CLIP_NOW and CLIP_NOW.startswith(("attack", "sword_", "kick_", "gaunt_", "body_")):
        if bone.startswith("armU."):
            fwd = clampf(fwd, FREE_ARM_FWD[0], FREE_ARM_FWD[1])
            out = clampf(out, 0.0, FREE_ARM_OUT)
        if bone.startswith("armL."):
            fwd = max(fwd, FREE_ELBOW)
    # Wrists held nearly straight (the owner, art review 5: bent wrists and open, flapping hands do not read as combat; a fist in line
    # with the forearm does).
    if BODY and bone.startswith("hand."):
        fwd, out, twist = fwd * WRIST_KEEP, out * WRIST_KEEP, twist * WRIST_KEEP
    # The base body's head is big: tipped back more than a little it shows its chin and hides the drawn face, the half of every read.
    if BODY and bone == "head" and CLIP_NOW not in HEAD_FREE_CLIPS:
        if HEAD_LIMIT_NOW is not None:
            fwd = max(fwd, HEAD_LIMIT_NOW)
        else:
            fwd = max(fwd, HEAD_BACK_IDLE if CLIP_NOW.startswith(("idle", "walk")) else HEAD_BACK_LIMIT)
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
# The free arm in an attack (see `pose`): the upper arm's swing range (degrees, back to front), how far out to the side, the elbow's least bend.
FREE_ARM_FWD = (10.0, 80.0)
FREE_ARM_OUT = 24.0
FREE_ELBOW = 90.0
HEAD_FREE_CLIPS = ("knockdown", "getup", "ledge", "ledge_climb")
CLIP_NOW = ""
# A head limit for the key being posed (the attacks' stance keys take the idle's, `_stance_rest`), or None for the clip's own.
HEAD_LIMIT_NOW = None


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
# How far an attack's chamber (wind-up) key is pushed past its pose. (Animation overhaul pass 3: the game now holds the coil for a third of
# the startup, so it is pushed deeper, 1.22 to 1.34, and reads as a wound-up shape; the `move` clips' wind-up took 1.12.)
CHAMBER_AMP = 1.34
# The knee bend (degrees) a grounded move's legs keep under its pose (the back leg 0.7 of it): pass 4, after the stance pass.
KNEE_SOFT = 22.0
WRIST_KEEP = 0.3   # how much of a clip's wrist bend the base body keeps

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
        # (Animation overhaul pass 4, against the reference's frames: it crouches into the coil, then cuts from a lunge about twice as
        # deep as ours was, the back leg long behind and the front knee well bent, and holds it. Was stance (18, -14) and (36, -28).)
        {"hips": -0.08, "spine": (6.0, -28.0), "head": (-2.0, 14.0), "stance": (24.0, -18.0), "shin.L": (-48.0,), "shin.R": (-34.0,),
         "armU.L": (-10.0, 40.0), "armL.L": (30.0,)},
        {"hips": -0.04, "spine": (18.0, 28.0), "head": (-6.0, -12.0), "stance": (50.0, -40.0), "shin.L": (-56.0,), "shin.R": (-4.0,),
         "armU.L": (-30.0, 50.0), "armL.L": (20.0,)},
        {"spine": (6.0, 10.0), "stance": (30.0, -24.0), "shin.L": (-36.0,)}),
    # Fencer's forward smash: the blade raised high with the weight back, then one committed cut down in front into a deep lunge, the
    # front knee well bent, the back leg straight, the free arm thrown back.
    "sword_fsmash": (
        # (Round 2, judged frame by frame at real speed: the coil stood up on both legs drawn in under the body, (24, 18) with the back shin
        # at -40, so the wind-up floated up out of the stance; the reference sits back onto its bent back leg with the front foot reaching
        # ahead. Now the front leg reaches forward and the back knee loads.)
        {"hips": -0.02, "spine": (-16.0, -40.0), "head": (10.0, 20.0), "stance": (14.0, -14.0), "shin.L": (-8.0,), "shin.R": (-26.0,),
         "armU.L": (20.0, 40.0)},
        # (As the reference's: a lunge so deep the back leg lies nearly flat behind and the body drops low over the bent front knee, the
        # head kept up to the camera.)
        # (The torso leans in about 25 degrees, not folded over: at 30 before the exaggeration it bent nearly double and took the head
        # down to the blade. Art review 5, against the reference's frames.)
        # (A lunge, not a kneel: the hips dropped 0.24 before the exaggeration sat the short-legged body down on both knees, and the code
        # places the sword hand from the feet, so the head sank to the hand. Art review 5.)
        {"hips": -0.1, "spine": (14.0, 30.0), "head": (-20.0, -14.0), "stance": (64.0, -48.0), "shin.L": (-54.0,), "shin.R": (-2.0,),
         "armU.L": (-50.0, 40.0), "armL.L": (10.0,)},
        {"hips": -0.06, "spine": (10.0, 12.0), "head": (-10.0, 0.0), "stance": (44.0, -34.0), "shin.L": (-40.0,)}),
    # Fencer's down tilt: a low lunging thrust, down on the bent front leg with the back leg stretched long behind.
    "sword_dtilt": (
        {"hips": -0.2, "spine": (20.0, -10.0), "head": (-10.0, 0.0), "stance": (50.0, 26.0), "shin.L": (-90.0,), "shin.R": (-100.0,)},
        # (The back kept long and the head up: at 34 the torso folded over so far that the camera saw the top of the head. Art review 5.)
        {"hips": -0.28, "spine": (18.0, 10.0), "head": (-30.0, 0.0), "stance": (70.0, -30.0), "shin.L": (-92.0,), "shin.R": (-20.0,),
         "armU.L": (-30.0, 30.0)},
        {"hips": -0.16, "spine": (22.0, 0.0), "stance": (48.0, -8.0), "shin.L": (-70.0,)}),
    # The claws' forward tilt: both claws cocked across the chest, then the fighter lunges and throws them out forward and wide (a
    # two-handed outward slash; the game aims the weapon hand, this the other).
    "attack_claws": (
        {"spine": (8.0, -10.0), "head": (0.0, 6.0), "stance": (16.0, -16.0), "armU.L": (60.0, -24.0), "armL.L": (100.0,)},
        # (The reference's is a low lunge, the body nearly level over the bent front leg, one claw thrust far out ahead.)
        {"hips": -0.12, "spine": (26.0, 12.0), "head": (-8.0, -4.0), "stance": (62.0, -50.0), "shin.L": (-72.0,), "shin.R": (-2.0,),
         "armU.L": (-40.0, 34.0), "armL.L": (20.0,)},
        {"hips": -0.08, "spine": (14.0, 4.0), "stance": (30.0, -24.0), "shin.L": (-30.0,)}),
    # The Dagger's forward smash, after its reference's (animation overhaul pass 11): leaned right back on the back leg with the knife
    # cocked up behind the head (the game raises the knife hand, `OVERHEAD_CHAMBER`), then cut down in front as the body dives into a
    # lunge so low and long that the torso is pitched well forward over the bent front knee, the back leg stretched flat behind; the head
    # is kept up toward the camera and the hit.
    "attack_knife_smash": (
        {"hips": -0.04, "spine": (-22.0, -34.0), "head": (10.0, 18.0), "stance": (20.0, 24.0), "shin.L": (-14.0,), "shin.R": (-60.0,),
         "armU.L": (60.0, 20.0), "armL.L": (100.0,)},
        {"hips": -0.2, "spine": (24.0, 26.0), "head": (-8.0, -12.0), "stance": (74.0, -58.0), "shin.L": (-74.0,), "shin.R": (-2.0,),
         "armU.L": (-30.0, 30.0), "armL.L": (30.0,)},
        {"hips": -0.1, "spine": (18.0, 12.0), "head": (-10.0, 0.0), "stance": (46.0, -36.0), "shin.L": (-44.0,)}),
    # The rusher's charged punch (the Gauntlets' neutral special, after its reference's; animation overhaul pass 12): over the long wind-up
    # the body turns right away from the foe and sinks onto the bent back leg, the free arm thrown out ahead to aim while the game pulls
    # the punching fist far back behind the hip (`DEEP_CHAMBER`), then everything unwinds at once into a huge lunge, the shoulders turned
    # through past square and the back leg flung long behind.
    "gaunt_punch": (
        {"hips": -0.14, "spine": (-4.0, -58.0), "head": (6.0, 30.0), "stance": (26.0, 30.0), "shin.L": (-30.0,), "shin.R": (-74.0,),
         "armU.L": (80.0, 10.0), "armL.L": (90.0,)},
        {"hips": -0.16, "spine": (26.0, 48.0), "head": (-8.0, -26.0), "stance": (72.0, -54.0), "shin.L": (-66.0,), "shin.R": (-2.0,),
         "armU.L": (-10.0, 24.0), "armL.L": (100.0,)},
        {"hips": -0.08, "spine": (12.0, 18.0), "stance": (40.0, -30.0), "shin.L": (-36.0,)}),
    # Round 2's own clips for the last stand-in moves (names without an attack prefix, so the arms take no attack clamps or exaggeration
    # and both are the clip's: AnimLibrary.OWN_ARMS). The Hauler's hand slap: both arms raised up behind the head, then both palms slammed
    # down onto the floor in front, bent over deep knees (each hit of the loop strikes again, `strike_progress`).
    "slap_ground": (
        {"hips": -0.06, "spine": (-10.0, 0.0), "head": (8.0, 0.0), "stance": (30.0, -26.0), "shin.L": (-50.0,), "shin.R": (-40.0,),
         "arms": (165.0, 10.0), "armL.L": (40.0,), "armL.R": (40.0,)},
        {"hips": -0.24, "spine": (44.0, 0.0), "head": (-26.0, 0.0), "stance": (44.0, -34.0), "shin.L": (-80.0,), "shin.R": (-60.0,),
         "arms": (36.0, 14.0), "armL.L": (15.0,), "armL.R": (15.0,)},
        {"spine": (10.0, 0.0), "arms": (20.0, 14.0)}),
    # The Scamp's monkey flip: crouched for the leap, then diving forward with both hands reaching out ahead to grab (the game turns the
    # body over in a forward flip, fighter_view.gd FLIP_MOVES).
    "leap_grab": (
        {"spine": (30.0, 0.0), "head": (-10.0, 0.0), "stance": (70.0, 60.0), "shin.L": (-110.0,), "shin.R": (-110.0,),
         "arms": (-40.0, 20.0), "armL.L": (20.0,), "armL.R": (20.0,)},
        {"spine": (40.0, 0.0), "head": (-30.0, 0.0), "stance": (-10.0, -30.0), "shin.L": (-20.0,), "shin.R": (-30.0,),
         "arms": (95.0, 12.0), "armL.L": (15.0,), "armL.R": (15.0,)},
        {"spine": (10.0, 0.0), "stance": (30.0, 20.0), "shin.L": (-60.0,), "shin.R": (-60.0,), "arms": (40.0, 14.0)}),
    # The Scamp's latch: sitting astride the caught fighter's head, knees out round it, both hands raised, then raking down to scratch.
    "latch_sit": (
        {"spine": (-6.0, 0.0), "head": (4.0, 0.0), "stance": (84.0, 80.0), "shin.L": (-100.0,), "shin.R": (-100.0,),
         "arms": (120.0, 18.0), "armL.L": (50.0,), "armL.R": (50.0,)},
        {"spine": (30.0, 0.0), "head": (-16.0, 0.0), "stance": (84.0, 80.0), "shin.L": (-100.0,), "shin.R": (-100.0,),
         "arms": (30.0, 16.0), "armL.L": (40.0,), "armL.R": (40.0,)},
        {"spine": (6.0, 0.0), "stance": (84.0, 80.0), "shin.L": (-100.0,), "shin.R": (-100.0,), "arms": (50.0, 16.0)}),
    # The Scamp's rocket barrels: the charge crouched low and leaning in, gripping the straps at the chest (the game holds it, HELD_COIL);
    # the flight stretched long, the arms swept back and the legs trailing together (the game pitches it along its path, FLIGHT_MOVES).
    "rocket_fly": (
        {"hips": -0.1, "spine": (26.0, 0.0), "head": (-10.0, 0.0), "stance": (50.0, 40.0), "shin.L": (-90.0,), "shin.R": (-90.0,),
         "arms": (30.0, 20.0), "armL.L": (110.0,), "armL.R": (110.0,)},
        {"spine": (6.0, 0.0), "head": (-8.0, 0.0), "stance": (-6.0, -12.0), "shin.L": (-10.0,), "shin.R": (-14.0,),
         "arms": (-50.0, 24.0), "armL.L": (10.0,), "armL.R": (10.0,)},
        {"spine": (8.0, 0.0), "stance": (-4.0, -10.0), "shin.L": (-12.0,), "shin.R": (-14.0,), "arms": (-40.0, 24.0)}),
    # The Hauler's giant punch charge (round 2; the game holds it in its wind-up pose for the whole charge, fighter_view.gd HELD_COIL, and
    # swings the punching arm round and round on top, WINDMILL): planted wide and low, leaning back a touch, the chest left open to the
    # camera so the swinging arm reads beside it, the other fist low in front. (The values are small: the chamber key is exaggerated
    # about 2.3 times, `CHAMBER_AMP` and the attacks' `AMP`.)
    "gaunt_charge": (
        {"hips": -0.06, "spine": (-3.0, -6.0), "head": (-2.0, 4.0), "stance": (18.0, -16.0), "shin.L": (-22.0,), "shin.R": (-18.0,),
         "armU.L": (14.0, 22.0), "armL.L": (95.0,)},
        {"hips": -0.06, "spine": (-3.0, -6.0), "head": (-2.0, 4.0), "stance": (18.0, -16.0), "shin.L": (-22.0,), "shin.R": (-18.0,),
         "armU.L": (14.0, 22.0), "armL.L": (95.0,)},
        {"hips": -0.12, "spine": (4.0, 0.0), "stance": (30.0, -26.0), "shin.L": (-40.0,)}),
    # The bounder's drop (its down special, after its reference's; pass 13): crouched for the hop, then sat down in the air bottom first,
    # the legs held out level in front and the body leaning back a little over them, the arms out to the sides (raised, the near one
    # crossed the face), so it falls seat first onto the
    # target; the body is the weapon (`body_`: no limb reaches for the hit).
    "body_drop": (
        {"hips": -0.1, "spine": (24.0, 0.0), "head": (6.0, 0.0), "stance": (40.0, 36.0), "shin.L": (-90.0,), "shin.R": (-86.0,),
         "arms": (-20.0, 24.0)},
        {"spine": (-14.0, 0.0), "head": (10.0, 0.0), "stance": (88.0, 82.0), "shin.L": (-16.0,), "shin.R": (-22.0,),
         "foot.L": (-20.0,), "foot.R": (-20.0,), "arms": (16.0, 40.0)},
        {"spine": (-8.0, 0.0), "stance": (80.0, 76.0), "shin.L": (-30.0,), "shin.R": (-34.0,), "arms": (14.0, 34.0)}),
    # Forward air (a crescent cut, or a claw raked, down through the space in front): the weapon raised with the shoulders turned back and
    # the knees drawn up, then the torso leans over the swing and turns into it, the front knee up and the back leg trailing.
    # Upright enough that the face stays toward the hit.
    # (Compact then long: the wind-up tucks both knees high and cocks the free arm, the strike opens out, the front knee driven up and the
    # back leg kicked long behind, so the silhouette changes shape, not just angle.)
    # (After the reference's forward air, the owner's gold standard, 2026-10-17: leaning back with the knees tucked as the fist or blade
    # cocks up behind the head (the game sweeps the arm, `_overhead_angle`), then the whole body curls forward over the strike with the
    # back leg trailing, and stays curled through the follow-through, so the swing has weight. The free arm stays tucked.)
    "attack_fair": (
        {"spine": (-14.0, -20.0), "head": (-6.0, 10.0), "stance": (56.0, 44.0), "shin.L": (-100.0,), "shin.R": (-92.0,)},
        {"spine": (34.0, 24.0), "head": (10.0, -10.0), "stance": (44.0, -30.0), "shin.L": (-90.0,), "shin.R": (-26.0,)},
        {"spine": (24.0, 8.0), "stance": (42.0, 22.0), "shin.L": (-84.0,), "shin.R": (-64.0,)}),
    # Up air (an overhead crescent from front to back): tucked, then the body stretches up tall under the swing, the legs reaching down.
    # (Pose review 2026-10-17: up moves reach from the shoulders with the chest lifted and the hips over the feet; the round body arched
    # back with its belly forward read as inhaling, not striking.)
    "attack_uair": (
        {"spine": (12.0, -8.0), "head": (6.0, 0.0), "stance": (34.0, 30.0), "shin.L": (-80.0,), "shin.R": (-70.0,), "armU.L": (24.0, 20.0)},
        {"spine": (4.0, 10.0), "head": (-2.0, 0.0), "stance": (14.0, -16.0), "shin.L": (-30.0,), "shin.R": (-20.0,),
         "armU.L": (-16.0, 44.0), "armL.L": (20.0,)},
        {"spine": (2.0, 0.0), "stance": (24.0, 16.0), "shin.L": (-50.0,), "shin.R": (-40.0,)}),
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
        {"spine": (0.0, 0.0), "head": (-4.0, 0.0), "stance": (0.0, 4.0), "shin.R": (-4.0,), "armU.L": (-20.0, 50.0)},
        {"spine": (2.0, 0.0), "stance": (4.0, 4.0)}),
    # A headbutt straight up (the all-rounder's and the dinosaur's up smash, the ape's up air): crouched low with the head ducked and the
    # arms drawn down and back, then the whole body springs up through the legs, upright, the arms flung down behind, so the head
    # leads the hit. (`body_` clips: no arm reaches for the hitbox; the body is the weapon.)
    "body_headbutt": (
        {"hips": -0.16, "spine": (34.0, 0.0), "head": (14.0, 0.0), "stance": (30.0, 26.0), "shin.L": (-70.0,), "shin.R": (-66.0,),
         "arms": (-30.0, 26.0)},
        {"hips": 0.0, "spine": (2.0, 4.0), "head": (2.0, 0.0), "stance": (10.0, -8.0), "shin.L": (-14.0,), "shin.R": (-10.0,),
         "arms": (-54.0, 34.0)},
        {"spine": (2.0, 0.0), "stance": (8.0, -4.0)}),
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
    # ---- Round 8: the Needles' own specials (they played the blaster, the swing and the low strike; ANIMATION_REVIEW_1.md item 14) ----
    # Needle Volley: coiled low and turned away, the near hand drawn back behind the hip with the needles, the far hand guarding in front
    # (the charge holds this coil); the throw whips the near arm through flat and out in front as the chest turns into it.
    "needle_volley": (
        {"hips": -0.12, "spine": (22.0, -26.0), "head": (-6.0, 16.0), "stance": (42.0, -26.0), "armU.L": (-34.0, 22.0),
         "armL.L": (64.0,), "armU.R": (62.0, 12.0), "armL.R": (70.0,)},
        {"hips": -0.08, "spine": (18.0, 26.0), "head": (-10.0, -10.0), "stance": (54.0, -36.0), "armU.L": (94.0, 6.0), "armL.L": (8.0,),
         "armU.R": (-24.0, 26.0), "armL.R": (30.0,)},
        {"spine": (10.0, 0.0), "stance": (30.0, -20.0)}),
    # The pulling line: the line swung back low behind on a step back, then cast forward and up underhand at full stretch, the body
    # leaning after it.
    "line_cast": (
        {"hips": -0.08, "spine": (8.0, -30.0), "head": (-4.0, 18.0), "stance": (24.0, -30.0), "armU.L": (-56.0, 26.0), "armL.L": (14.0,),
         "armU.R": (56.0, 16.0), "armL.R": (80.0,)},
        {"hips": -0.04, "spine": (20.0, 24.0), "head": (-8.0, -8.0), "stance": (52.0, -30.0), "armU.L": (112.0, 4.0), "armL.L": (6.0,),
         "armU.R": (-30.0, 24.0), "armL.R": (24.0,)},
        {"spine": (8.0, 0.0), "stance": (28.0, -16.0)}),
    # (Round 8, the stand-in specials.) The Longsword's counter (it played the low strike): settled back onto the rear foot, chest open,
    # the free hand drawn back, the blade (the game's) held in front. The Claws' reflector (it played a kick): down over bent knees, the
    # near palm thrust out low in front, the far arm back.
    "sword_counter": (
        {"hips": -0.08, "spine": (8.0, -10.0), "head": (-6.0, 6.0), "stance": (30.0, -20.0)},
        {"hips": -0.16, "spine": (-6.0, -22.0), "head": (-8.0, 12.0), "stance": (34.0, -34.0), "shin.L": (-56.0,), "shin.R": (-70.0,),
         "armU.L": (20.0, 24.0)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    "reflector": (
        {"hips": -0.1, "spine": (16.0, -14.0), "head": (-6.0, 8.0), "stance": (40.0, -24.0), "armU.L": (30.0, 20.0), "armL.L": (96.0,),
         "armU.R": (24.0, 20.0), "armL.R": (84.0,)},
        {"hips": -0.18, "spine": (24.0, 10.0), "head": (-12.0, 0.0), "stance": (56.0, -30.0), "armU.L": (66.0, -2.0), "armL.L": (10.0,),
         "hand.L": (-30.0,), "armU.R": (-20.0, 28.0), "armL.R": (30.0,)},
        {"spine": (10.0, 0.0), "stance": (30.0, -20.0)}),
    # (Round 8, the Wanderer's specials: the sword stays low in the near hand, the free (far) hand works, as the adventurer reference
    # does with his shield hand.) The bow: the far arm held out straight ahead at shoulder height, the chest squared behind it, the head
    # down the line; the shot's recoil eases the body back.
    "buckler_bow": (
        {"hips": -0.06, "spine": (8.0, -18.0), "head": (-6.0, 14.0), "stance": (34.0, -30.0), "armU.R": (92.0, 4.0), "armL.R": (6.0,)},
        {"hips": -0.04, "spine": (-6.0, -14.0), "head": (-4.0, 12.0), "stance": (30.0, -36.0), "armU.R": (88.0, 6.0), "armL.R": (10.0,)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The boomerang: the far hand cocked back behind the shoulder on a step back, then flung through flat and out ahead.
    "buckler_throw": (
        {"hips": -0.08, "spine": (6.0, -28.0), "head": (-6.0, 16.0), "stance": (26.0, -32.0), "armU.R": (-46.0, 30.0), "armL.R": (64.0,)},
        {"hips": -0.06, "spine": (22.0, 22.0), "head": (-10.0, -8.0), "stance": (50.0, -30.0), "armU.R": (96.0, 4.0), "armL.R": (8.0,)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The bomb: the far hand reached back behind the hip, then brought up in front holding it.
    "buckler_bomb": (
        {"hips": -0.12, "spine": (18.0, -26.0), "head": (-8.0, 14.0), "stance": (34.0, -26.0), "armU.R": (-52.0, 22.0), "armL.R": (28.0,)},
        {"hips": -0.04, "spine": (6.0, 10.0), "head": (-8.0, 0.0), "stance": (28.0, -22.0), "armU.R": (72.0, 10.0), "armL.R": (70.0,)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # (Round 8, the Seer's, the Champion's and the Exile's specials: the body only (and the Exile's free hand); the weapon is held by
    # the game through each move, `HELD_BLADE` in fighter_view.gd.) The Seer's reticle: upright and leaning a little back, the chest
    # turned to the front behind the staff pointed ahead.
    "staff_point": (
        {"hips": -0.04, "spine": (-4.0, -16.0), "head": (-4.0, 10.0), "stance": (30.0, -26.0)},
        {"hips": -0.02, "spine": (-8.0, -10.0), "head": (-2.0, 8.0), "stance": (32.0, -30.0)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The Seer's explosive flame: turned away as the staff goes up, then leaning into the thrust as it points ahead.
    "staff_flame": (
        {"hips": -0.06, "spine": (6.0, -40.0), "head": (-6.0, 22.0), "stance": (26.0, -30.0)},
        {"hips": -0.06, "spine": (14.0, 18.0), "head": (-6.0, -6.0), "stance": (46.0, -32.0)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The Seer's counter and barrier: standing tall behind the upright staff, the chest square.
    "staff_guard": (
        {"hips": -0.02, "spine": (-2.0, -12.0), "head": (-4.0, 8.0), "stance": (28.0, -22.0)},
        {"hips": 0.0, "spine": (-6.0, -20.0), "head": (-2.0, 10.0), "stance": (26.0, -24.0)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The Champion's wave: crouched over the blade dragged low behind, then rising tall as it sweeps up through the front to overhead.
    "great_wave": (
        {"hips": -0.16, "spine": (24.0, -30.0), "head": (-10.0, 16.0), "stance": (44.0, -34.0)},
        {"hips": -0.04, "spine": (-10.0, 16.0), "head": (2.0, -4.0), "stance": (40.0, -30.0)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The Champion's rising slash: crouched, then stretched tall as the blade comes up, the back leg trailing.
    "great_rise": (
        {"hips": -0.14, "spine": (20.0, -20.0), "head": (-8.0, 12.0), "stance": (40.0, -30.0)},
        {"hips": 0.0, "spine": (-12.0, 10.0), "head": (6.0, 0.0), "stance": (10.0, -40.0)},
        {"spine": (0.0, 0.0), "stance": (20.0, -30.0)}),
    # The Exile's flare: the free hand held out ahead, palm open, the long blade kept low in the other.
    "nodachi_flare": (
        {"hips": -0.04, "spine": (2.0, -24.0), "head": (-4.0, 14.0), "stance": (28.0, -24.0), "armU.R": (60.0, 10.0), "armL.R": (30.0,)},
        {"hips": -0.02, "spine": (-6.0, -18.0), "head": (-2.0, 12.0), "stance": (30.0, -28.0), "armU.R": (88.0, 4.0), "armL.R": (6.0,)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The Exile's shadow orbs: the free hand drawn back, then swung up and out ahead.
    "nodachi_orbs": (
        {"hips": -0.06, "spine": (6.0, -30.0), "head": (-6.0, 16.0), "stance": (28.0, -30.0), "armU.R": (-40.0, 30.0), "armL.R": (50.0,)},
        {"hips": -0.04, "spine": (10.0, 16.0), "head": (-6.0, -4.0), "stance": (40.0, -30.0), "armU.R": (110.0, 6.0), "armL.R": (10.0,)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # The Exile's barrier: tall behind the blade held upright, then stepping into the cut ahead.
    "nodachi_barrier": (
        {"hips": -0.02, "spine": (-2.0, -14.0), "head": (-4.0, 8.0), "stance": (28.0, -22.0)},
        {"hips": -0.06, "spine": (10.0, 14.0), "head": (-6.0, -2.0), "stance": (40.0, -28.0)},
        {"spine": (8.0, 0.0), "stance": (30.0, -20.0)}),
    # Flash Step: gathered into a deep crouch with the arms crossed over the chest, then bursting open (the explosion) tall, arms and
    # chest flung wide; on its reappearing hit the same burst.
    "vanish": (
        {"hips": -0.2, "spine": (32.0, 0.0), "head": (-14.0, 0.0), "stance": (62.0, -34.0), "armU.L": (58.0, -6.0), "armL.L": (112.0,),
         "armU.R": (62.0, -6.0), "armL.R": (112.0,)},
        {"hips": 0.04, "spine": (-10.0, 0.0), "head": (-6.0, 0.0), "stance": (14.0, -12.0), "armU.L": (-36.0, 44.0), "armL.L": (10.0,),
         "armU.R": (-30.0, 44.0), "armL.R": (10.0,)},
        {"spine": (10.0, 0.0), "stance": (30.0, -20.0)}),
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
    # Floating under the balloons, the legs hanging and paddling a little apart. (Round 3, the owner's pick: a re-posed hold. Both hands
    # up beside the head put the near one in front of the face, side-on, for the whole float; now the far hand holds the strings high
    # over the head, a little behind it, and the near hand grips them lower, in front of the chest.)
    "tool_float": (
        {"spine": (-4.0, 0.0), "head": (-6.0, 0.0), "stance": (20.0, -10.0), "shin.L": (-40.0,), "shin.R": (-20.0,), "arms": (40.0, 4.0),
         "armU.R": (92.0, -4.0), "armL.R": (20.0,), "armL.L": (60.0,)},
        {"spine": (-8.0, 0.0), "head": (-10.0, 0.0), "stance": (-6.0, 24.0), "shin.L": (-20.0,), "shin.R": (-50.0,), "arms": (36.0, 2.0),
         "armU.R": (104.0, -6.0), "armL.R": (14.0,), "armL.L": (64.0,)},
        {"spine": (-6.0, 0.0), "head": (-8.0, 0.0), "stance": (10.0, 10.0), "shin.L": (-30.0,), "shin.R": (-30.0,), "arms": (36.0, 2.0),
         "armU.R": (102.0, -6.0), "armL.R": (16.0,), "armL.L": (62.0,)}),
    # Fencer's up tilt: the blade sweeps overhead from front to back while the body stands tall under it.
    # (Pose review 2026-10-17: up moves reach from the shoulders with the chest lifted and the hips over the feet; the round body arched
    # back with its belly forward read as inhaling, not striking.)
    "sword_utilt": (
        {"hips": -0.06, "spine": (14.0, 10.0), "head": (6.0, 0.0), "stance": (12.0, -10.0), "shin.L": (-20.0,)},
        {"hips": 0.0, "spine": (4.0, -8.0), "head": (-4.0, 0.0), "stance": (14.0, -18.0), "armU.L": (-20.0, 30.0)},
        {"spine": (2.0, 0.0), "stance": (8.0, -8.0)}),
}


# On the ground the base body stands on its feet: whatever the legs are posed to, the hips are raised or lowered so the lower foot is on
# the ground. In the standing moves (STANDING and BODY_MOVES) both feet stay down: the hips sink until the foot of the wider-spread leg
# touches, and the other leg is raised to put its foot down too (a lunge: the front knee lifted and bent, the back leg long), the soles
# level. The walk and run lift and roll their own feet, and the run and dash leave the ground between steps (STRIDING). The blob's stubby
# legs never needed any of it.
def grounded(name):
    """Whether a clip is played on the ground (the feet planted), not in the air, hit, rolling, lying down or on a ledge."""
    if name in ("jump", "fall", "fall_start", "tumble", "air_jump", "air_dodge", "hurt", "roll", "knockdown", "getup", "ledge", "ledge_climb", "kick_dash", "body_drop",
                "hurt_heavy", "hurt_up", "platform_drop", "gaunt_knee", "gaunt_stomp", "tool_ride", "tool_float",
                # (Aerials whose names do not end in "air": round 2, so their rest keys do not take the battle stance.)
                "sword_rise", "sword_plunge", "kick_double", "leap_grab", "latch_sit", "rocket_fly",
                # (Round 8: the special fall, and buried, whose legs are under the ground.)
                "helpless", "buried"):
        return False
    return not name.endswith("air")


STANDING = {"item_pick", "item_throw", "dizzy", "idle", "idle_brawler", "idle_quick", "idle_heavy", "idle_caster", "idle_tool", "crouch", "jumpsquat", "land", "spot_dodge", "shield", "skid", "grab", "throw", "blaster", "sword_usmash", "sword_dsmash",
            "tech", "run_stop", "idle_look", "idle_look_brawler", "idle_look_quick", "idle_look_heavy", "idle_look_caster", "idle_look_tool"}


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
STRIDING = {"run", "dash", "run_quick", "run_heavy", "run_caster", "run_tool", "run_brawler", "walk", "walk_brawler", "walk_quick", "walk_heavy",
             "walk_caster", "walk_tool"}


# A grounded attack's coil (its key 19, or 21 for the older `move` clips) never stands the hips higher than this (the battle stance's are at
# 0.63-0.65): a wind-up that rose onto straight legs read as floating up before the strike, where the reference loads its knees.
# (Round 2: the Longsword's forward smash coiled at 0.80, the Settler's chop at 0.87.)
COIL_KEYS = (19, 21)
COIL_CAPPED = ("attack", "sword_", "kick_", "gaunt_", "tool_", "body_")
COIL_HIPS_MAX = 0.82   # (0.64 until round 8's stance C, the owner's pick: the stance's hips stand at 0.85-0.93 now; a coil loads the knees a little below it)


def _sink_to(rig, top):
    """Lowers the hips to `top` if they are higher, bending the legs to keep both feet on the ground (`_lift_onto`)."""
    bpy.context.view_layer.update()
    over = rig.pose.bones["hips"].head.z - top
    if over <= 0.0:
        return
    ground = rig.pose.bones["foot.L"].bone.head_local.z
    rig.pose.bones["hips"].location.y -= over
    for side in "LR":
        _lift_onto(rig, side, ground)
        _level_sole(rig, side)


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


def _human_arms(spec):
    """An arm as a person holds one (the owner's rule, art review 5): swung behind the body the arm is nearly straight, opened back for
    balance, as the reference fighters' counterbalancing arms are; held in front it may bend. (A forced bend on an arm thrown back
    folded the forearm forward round the body: the arm read as going behind and in front at once.)"""
    if not BODY:
        return spec
    out = dict(spec)
    arms = spec.get("arms", (8.0, 14.0))
    for side in "LR":
        up = spec.get("armU." + side, arms)[0]
        key = "armL." + side
        bend = spec[key][0] if key in spec else 30.0
        if up < -8.0:
            # Behind: nearly straight, the more so the further back.
            bend = min(bend, 20.0 * max(0.0, 1.0 + (up + 8.0) / 60.0) + 6.0)
        elif up > 30.0:
            # Out in front: a soft bend at least, never locked.
            bend = max(bend, 15.0)
        out[key] = (bend,)
    return out


def _spec_pose(spec):
    """A pose from the short form: {"hips": lift, "spine": (fwd, twist), "head": (fwd, twist), "stance": (lead, back), "arms": (fwd,
    out), bone: (fwd, out)...}; the free arm and legs not named rest."""
    spec = _human_arms(spec)

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
        # (Animation overhaul pass 4: on the ground the knees are never locked. A grounded move's legs keep a soft bend under whatever
        # the pose says (`KNEE_SOFT`), so `plant` sinks the hips and the moves stand as low as the battle stance does; the legs a pose
        # names itself keep their own bend.)
        soft = KNEE_SOFT if BODY and CLIP_NOW and grounded(CLIP_NOW) else 0.0
        lead, back = spec.get("stance", (0.0, 0.0)) if ("stance" in spec or soft) else (None, None)
        if lead is not None:
            # The further the legs step apart front to back, the more the feet come in toward one line (as a fencer's): seen turned three
            # quarters to the camera, feet set wide apart sideways would hide the step.
            # (Round 8: only a step, the legs apart front to back, draws the feet in. Both thighs forward (a tuck, a seat, a ground pound)
            # drew both knees in across the body by up to 30 degrees: the knees crossed in every aerial tuck and the Clutch's down special
            # turned its knees sideways. The reference fighters tuck with the knees side by side, pointing ahead.)
            split = lead * back < 0.0
            pose(rig, "thigh.L", fwd=lead, out=5.0 - 0.4 * abs(lead) if split else 6.0)
            pose(rig, "shin.L", fwd=-8.0 - max(0.0, -lead) * 0.5 - soft)
            pose(rig, "thigh.R", fwd=back, out=5.0 - 0.4 * abs(back) if split else 8.0)
            pose(rig, "shin.R", fwd=-8.0 - max(0.0, -back) * 0.5 - soft * 0.7)
        for bone, v in spec.items():
            if "." in bone:
                pose(rig, bone, fwd=v[0], out=v[1] if len(v) > 1 else 0.0)
    return f


def _stance_rest(rest, k):
    """An attack's rest key (its first and last frames) on the ground: the move's rest pose on top of the battle stance's legs and lean
    (`STANCE_*`, the sword family's idle without its bob), so a move starts from and settles back into the stance instead of rising to an
    upright stand and sinking again when the idle takes over. (Animation overhaul pass 4.)
    (Round 2: the stance is divided by the clip's exaggeration (`AMP`), which `pose` multiplies back, so the key is the idle's stance
    exactly. Exaggerated, the rest keys squatted the hips to 0.32-0.36 against the idle's 0.65: every move dropped into a squat on its
    first frame and stood back up out of one at its end. The head counters the lean as the idle's does, `STANCE_HEAD`.)"""
    return _stance_over(_spec_held(rest, k), rest, k)


def _stance_over(base, rest, k, legs_only=False):
    """`base` (a pose function) with the battle stance's legs, lean and head put back on top on the ground (see `_stance_rest`); `rest`'s
    spine and head (scaled by `k`) ride on the stance's. With `legs_only`, only the legs (the pose keeps its own chest and head)."""
    def f(rig):
        global HEAD_LIMIT_NOW
        base(rig)
        if BODY and CLIP_NOW and grounded(CLIP_NOW) and legs_only:
            a = AMP
            pose(rig, "thigh.L", fwd=STANCE_LEAD / a, out=STANCE_WIDE / a)
            pose(rig, "shin.L", fwd=STANCE_LEAD_SHIN / a)
            pose(rig, "thigh.R", fwd=STANCE_BACK / a, out=(STANCE_WIDE + STANCE_WIDE_BACK) / a)
            pose(rig, "shin.R", fwd=STANCE_BACK_SHIN / a)
        elif BODY and CLIP_NOW and grounded(CLIP_NOW):
            a = AMP
            pose(rig, "thigh.L", fwd=STANCE_LEAD / a, out=STANCE_WIDE / a)
            pose(rig, "shin.L", fwd=STANCE_LEAD_SHIN / a)
            pose(rig, "foot.L", fwd=6.0 / a, out=STANCE_FOOT_OUT / a)
            pose(rig, "thigh.R", fwd=STANCE_BACK / a, out=(STANCE_WIDE + STANCE_WIDE_BACK) / a)
            pose(rig, "shin.R", fwd=STANCE_BACK_SHIN / a)
            pose(rig, "foot.R", fwd=18.0 * STANCE_K / a, out=STANCE_FOOT_OUT / a)
            sp = rest.get("spine", (0.0, 0.0))
            hd = rest.get("head", (0.0, 0.0))
            pose(rig, "spine", fwd=(STANCE_LEAN + sp[0] * k * a) / a, twist=(8.0 + sp[1] * k * a) / a, out=STANCE_SIDE / a)
            HEAD_LIMIT_NOW = HEAD_BACK_IDLE
            pose(rig, "head", fwd=(STANCE_HEAD + hd[0] * k * a) / a, twist=hd[1] * k)
            HEAD_LIMIT_NOW = None
    return f


def _launch_spec(wind, strike):
    """The launch between the wind-up and the strike (the owner's swing notes, art review 5): the hips and legs already in the strike's
    place and the torso half way there while the arms and head are still wound back, so the swing comes from the hips through the torso
    to the shoulders, a chain, rather than the limbs rotating into place together."""
    out = {}
    for key in set(wind) | set(strike):
        if key in ("hips", "stance") or key.startswith(("thigh.", "shin.", "foot.")):
            out[key] = strike.get(key, wind.get(key))
        elif key == "spine":
            a = wind.get(key, (0.0, 0.0))
            b = strike.get(key, (0.0, 0.0))
            out[key] = tuple(x * 0.4 + y * 0.6 for x, y in zip(a, b))
        elif key in wind:
            out[key] = wind[key]
    return out


def _chamber_spec(wind, aerial):
    """The wind-up as a compact extreme: in the air the knees drawn up to the chest (both shins folded, the thighs up), on the ground the
    free arm drawn in, elbow bent, so the strike's extension reads by contrast."""
    out = dict(wind)
    if aerial:
        out["stance"] = (max(out.get("stance", (0.0, 0.0))[0], 60.0), max(out.get("stance", (0.0, 0.0))[1], 50.0))
        out["shin.L"] = (min(out.get("shin.L", (0.0,))[0], -100.0),)
        out["shin.R"] = (min(out.get("shin.R", (0.0,))[0], -95.0),)
    out["armL.L"] = (max(out.get("armL.L", (0.0,))[0], 90.0),)
    return out


def _follow_spec(strike):
    """The follow-through: the torso turns on past the target (its twist further) while the head lags behind it."""
    out = dict(strike)
    if "spine" in out:
        out["spine"] = (out["spine"][0], out["spine"][1] * 1.3)
    if "head" in out:
        out["head"] = (out["head"][0] * 0.7, out["head"][1] * 0.6)
    return out


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
    elif name in ("jump", "fall", "fall_start", "tumble", "hurt", "crouch", "jumpsquat", "land", "shield", "roll", "spot_dodge", "air_dodge", "knockdown"):
        AMP = 1.25
    else:
        AMP = 1.0
    if BODY:
        AMP = 1.0 + (AMP - 1.0) * BODY_AMP
    # (A move with no poses of its own, such as the claws' lunging slash, uses its BODY_MOVES poses on the blob too.)
    if name in BODY_MOVES and (BODY or not poses):
        wind, strike, rest = BODY_MOVES[name]
        # (Six poses after the owner's swing notes: settle from rest, anticipation pushed past the wind-up, the launch led by the hips,
        # the strike, the follow-through past it, the recovery, back to rest. The launch-to-strike gap is short, so the blade accelerates.)
        # (And from the reference fighters' forward and neutral airs and forward tilt (the owner's gifs): the chamber is a hard, compact
        # extreme (aerials tuck their knees to the chest), the strike snaps in and is held through the active frames, the follow-through
        # carries on, then the body eases home.)
        chamber = _chamber_spec(wind, not grounded(name))
        poses = {0: _stance_rest(rest, 0.0), 19: _amped_pose(_spec_pose(chamber), CHAMBER_AMP), 29: _spec_pose(_launch_spec(chamber, strike)),
                 32: _spec_pose(strike), 36: _spec_pose(strike), 40: _amped_pose(_spec_pose(_follow_spec(strike)), 1.1),
                 47: _stance_rest(rest, 1.0), 60: _stance_rest(rest, 0.0)}
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
            if frame in COIL_KEYS and name.startswith(COIL_CAPPED):
                _sink_to(rig, COIL_HIPS_MAX)
            if os.environ.get("HIPS_DEBUG"):
                bpy.context.view_layer.update()
                print("HIPS %s %d %.3f" % (name, frame, rig.pose.bones["hips"].head.z))
        key_all(rig, frame)
    action.frame_range = (0, max(poses))


def _ankle_at(rig, side, thigh, shin, out):
    pose(rig, "thigh." + side, fwd=thigh, out=out)
    pose(rig, "shin." + side, fwd=shin)
    return _ankle(rig, side).copy()   # (a live view of the bone otherwise: the next pose would change it)


def foot_to(rig, side, ahead, height, width):
    """Bends a leg (the thigh forward and out, and the knee, about the hips as they are posed now) so its ankle is `ahead` units ahead of
    the hip joint's rest position (the body's forward axis, -y), `height` units above the ground and `width` units further out to the side
    than the hip: a solve by Newton's method on the rig itself, so the hips' lift, twist and roll are allowed for and a planted foot stays
    put while they move. The knee only bends forward. Returns the largest miss (units)."""
    hip = rig.pose.bones["thigh." + side].bone.head_local
    ground = rig.pose.bones["foot.L"].bone.head_local.z
    outward = -1.0 if hip.x < 0.0 else 1.0
    goal = Vector((hip.x + outward * width, hip.y - ahead, ground + height))

    def err(v):
        return goal - _ankle_at(rig, side, v[0], v[1], v[2])

    v = Vector((20.0, -40.0, 6.0))
    e = err(v)
    for _ in range(12):
        if e.length < 0.001:
            break
        h = 0.5
        cols = []
        for n in range(3):
            d = v.copy()
            d[n] += h
            cols.append((err(d) - e) / h)
        jac = Matrix((cols[0], cols[1], cols[2])).transposed()
        try:
            step = jac.inverted() @ e
        except ValueError:
            break
        # (err(v + d) ~ e + J d, so d = -J^-1 e.)
        big = max(abs(x) for x in step)
        if big > 15.0:
            step *= 15.0 / big
        v -= step
        v[1] = min(-1.0, v[1])
        v[2] = clampf(v[2], -10.0, 40.0)
        e = err(v)
    if os.environ.get("FOOT_DEBUG") and e.length > 0.03:
        print("FOOT %s %s goal ahead %.3f height %.3f -> thigh %.1f shin %.1f out %.1f miss %.3f" % (CLIP_NOW, side, ahead, height, v[0], v[1], v[2], e.length))
    return e.length


# Each family's walk on the base body (animation overhaul round 2; the one shared swaggering walk slid about a third of its speed and
# stood up out of every stance). Each foot's path is authored, not its angles (`foot_to`): while it is down it moves back under the body
# in a straight line at an even pace, so it can stay planted, then it swings through, lifted, to land ahead again. The pace is set by the
# clip's length: the game plays a walk once a cycle every `WALK_TEMPO` (0.055) units of ground at any walking speed (fighter_view.gd), so
# a cycle covers `frames` x 0.055 units, and the foot's sweep while down (`sweep`, over the share `down` of the cycle) matches that ground
# seen through the body's three-quarter turn (`WALK_TURN_SHARE`: the share of a step along the body that shows as ground covered, about
# sin 60 for the game's turn then; 0.96 since the side-on turn of round 2, re-measured in round 3 with tests/motion_probe.gd). The
# short legs cannot sweep far, so the quicker walks have a moment with both feet off the floor (a bounce) rather than sliding.
#   hips: the hips' height (the rest stand is 0.94); lift: the swinging foot's highest; bob: the hips' rise and fall each step;
#   ahead: where the sweep is centred, ahead of the hip; lean, side, head: the chest's lean and side bend and the head's counter-tilt (as
#   the family's idle); twist: the hips and shoulders counter-turning; sway: the hips rolling side to side; wide: each foot set out from its hip (units);
#   arms: (upper arm forward, out, elbow) for the near and far arm, and how far they swing.
WALK_TEMPO = 0.055
WALK_TURN_SHARE = 0.96
WALK_STYLES = {
    # The sword family: a guarded advance in the battle stance, the free hand up, small arm swings.
    "": dict(frames=34, hips=0.74, down=0.42, lift=0.15, bob=0.025, ahead=0.02, lean=26.0, side=20.0, head=-19.0, twist=6.0,
             sway=0.0, wide=0.02, arm_l=(28.0, 30.0, 100.0), arm_r=(24.0, 28.0, 108.0), swing=10.0),
    # The brawlers: a hunched prowl, both fists up in the guard, the shoulders rolling.
    "brawler": dict(frames=35, hips=0.71, down=0.40, lift=0.14, bob=0.03, ahead=0.02, lean=30.0, side=18.0, head=-16.0,
                    twist=9.0, sway=0.0, wide=0.04, arm_l=(52.0, 20.0, 96.0), arm_r=(40.0, 26.0, 108.0), swing=8.0),
    # The quick fighters: low and springy, long light steps with a bounce between them, the lead hand out.
    "quick": dict(frames=38, hips=0.72, down=0.32, lift=0.16, bob=0.05, ahead=0.0, lean=28.0, side=14.0, head=-16.0,
                  twist=8.0, sway=0.0, wide=0.02, arm_l=(64.0, 10.0, 40.0), arm_r=(-14.0, 24.0, 100.0), swing=14.0),
    # The heavy one: wide, planted steps, the hips rolling side to side and the arms swinging big and slow.
    "heavy": dict(frames=33, hips=0.70, down=0.46, lift=0.12, bob=0.035, ahead=0.0, lean=18.0, side=12.0, head=-8.0,
                  twist=8.0, sway=5.0, wide=0.1, arm_l=(10.0, 46.0, 46.0), arm_r=(6.0, 46.0, 50.0), swing=24.0),
    # The caster: upright and neat, small even steps, the lead palm open and the other hand held in.
    "caster": dict(frames=35, hips=0.80, down=0.36, lift=0.13, bob=0.02, ahead=0.02, lean=8.0, side=6.0, head=-8.0,
                   twist=5.0, sway=0.0, wide=0.0, arm_l=(66.0, 10.0, 30.0), arm_r=(-16.0, 36.0, 104.0), swing=6.0),
    # The tool users: a loose, bouncy stroll, the arms swinging free.
    "tool": dict(frames=37, hips=0.79, down=0.34, lift=0.16, bob=0.04, ahead=0.02, lean=6.0, side=5.0, head=-4.0, twist=10.0,
                 sway=0.0, wide=0.01, arm_l=(4.0, 14.0, 34.0), arm_r=(2.0, 16.0, 30.0), swing=34.0),
}
for _st in WALK_STYLES.values():
    _st["sweep"] = _st["down"] * _st["frames"] * WALK_TEMPO / WALK_TURN_SHARE


def walk_style(phase, st):
    """One frame of a family's walk (`WALK_STYLES`) at `phase` (0..1). The left foot lands at phase 0, the right at 0.5."""
    def f(rig):
        rest_hips = rig.pose.bones["hips"].bone.head_local.z
        lowest = math.cos((phase - 0.06) * 4.0 * math.pi)      # 1 just after each landing
        roll = math.sin(phase * 2.0 * math.pi)
        pose(rig, "hips", lift=st["hips"] - rest_hips - st["bob"] * lowest, twist=-st["twist"] * roll, out=st["sway"] * roll)
        breath = math.cos((phase - 0.12) * 4.0 * math.pi)
        pose(rig, "spine", fwd=st["lean"] + 2.0 * breath, twist=8.0 + st["twist"] * 1.3 * roll, out=st["side"])
        pose(rig, "head", fwd=st["head"] - 2.0 * breath, twist=-4.0 - st["twist"] * 0.8 * roll)
        for side, arm in (("L", st["arm_l"]), ("R", st["arm_r"])):
            # Each arm swings with the other leg: back as its own leg lands in front.
            swing = -math.cos((phase + (0.0 if side == "L" else 0.5)) * 2.0 * math.pi)
            far = FAR_ARM_FWD if side == "R" else 0.0
            pose(rig, "armU." + side, fwd=arm[0] + far + st["swing"] * swing, out=arm[1] - (FAR_ARM_IN if side == "R" else 0.0))
            pose(rig, "armL." + side, fwd=arm[2] + 0.3 * st["swing"] * max(0.0, swing))
        down, sweep = st["down"], st["sweep"]
        for side, offset in (("L", 0.0), ("R", 0.5)):
            p = (phase + offset) % 1.0
            if p < down:
                u = p / down
                ahead = st["ahead"] + sweep * (0.5 - u)
                height = 0.0
                toe = 0.0
            else:
                u = (p - down) / (1.0 - down)
                # (A Hermite curve whose ends move at the stance's pace: the foot leaves the floor still going back and lands already
                # going back with the ground, so it neither drags at the lift nor skids on landing.)
                m = -sweep * (1.0 - down) / down
                h00, h10, h01, h11 = 2 * u ** 3 - 3 * u * u + 1, u ** 3 - 2 * u * u + u, -2 * u ** 3 + 3 * u * u, u ** 3 - u * u
                ahead = st["ahead"] + h00 * (-0.5 * sweep) + h10 * m + h01 * (0.5 * sweep) + h11 * m
                height = st["lift"] * math.sin(math.pi * min(1.0, u ** 0.8))
                # (The toe leaves last and the heel reaches for the floor.)
                toe = -24.0 * math.sin(math.pi * u * 2.0) if u < 0.5 else 14.0 * math.sin(math.pi * (u - 0.5) * 2.0)
            miss = foot_to(rig, side, ahead, height, st["wide"])
            if miss > 0.01:
                print("walk %s: the %s foot misses by %.3f at %.2f" % (CLIP_NOW, side, miss, phase))
            _level_sole(rig, side)
            rig.pose.bones["foot." + side].rotation_euler.x += math.radians(-toe)
    return f


# Each family's run on the base body (animation round 3; the runs were posed as leg angles, so a planted foot slid 0.4-0.8 of the ground
# speed, and the sword and brawler families shared one run). As the walks (`walk_style`), each foot's path is authored: while down it moves
# straight back under the body at the ground's pace, then it kicks up behind, the knee drives through and it reaches ahead to land. The
# game plays a run once a cycle every `frames` x `RUN_TEMPO` units of ground (fighter_view.gd `RUN_TEMPO`), so a cycle covers that much,
# and the sweep while down matches it seen through the body's side-on turn (`RUN_TURN_SHARE`). A foot is down only `down` of the cycle,
# with a flight between the steps.
#   hips: the hips' height; down: the share of the cycle a foot is down; lift: the swinging foot's highest; kick: how far back the heel
#   kicks up; bob: the hips' drop on each landing; lean, side, head: the chest's lean and side bend and the head's counter-tilt; twist: the
#   hips and shoulders counter-turning; sway: the hips rolling; wide: each foot set out from its hip; arm_l, arm_r: (upper arm forward,
#   out, elbow) for the near (weapon) and far arm; swing_l, swing_r: how far each pumps.
RUN_TEMPO = 0.17
RUN_TURN_SHARE = 0.98
RUN_STYLES = {
    # The sword family: leaning in, the blade carried low behind in the near hand, the far fist pumping.
    "": dict(frames=21, hips=0.68, down=0.25, lift=0.3, kick=0.16, bob=0.07, lean=26.0, side=8.0, head=-14.0, twist=10.0, sway=0.0,
             wide=0.03, arm_l=(-12.0, 26.0, 40.0), arm_r=(14.0, 22.0, 96.0), swing_l=18.0, swing_r=48.0),
    # The brawlers: hunched low, both fists up and pumping short and hard.
    "brawler": dict(frames=21, hips=0.64, down=0.25, lift=0.28, kick=0.14, bob=0.08, lean=34.0, side=10.0, head=-18.0, twist=12.0,
                    sway=0.0, wide=0.04, arm_l=(34.0, 22.0, 104.0), arm_r=(36.0, 20.0, 108.0), swing_l=30.0, swing_r=34.0),
    # The quick fighters: very low and long, the arms swept back.
    "quick": dict(frames=23, hips=0.62, down=0.24, lift=0.32, kick=0.2, bob=0.05, lean=40.0, side=6.0, head=-20.0, twist=8.0, sway=0.0,
                  wide=0.02, arm_l=(-44.0, 18.0, 24.0), arm_r=(-40.0, 20.0, 28.0), swing_l=12.0, swing_r=12.0),
    # The heavy one (round 4: it read like the others): lower, longer strides at a slower cadence (as long as the short legs reach), a
    # long, heavy contact with the weight dropping onto each step after the foot is down (`sink`), the feet kept low, the body rocking
    # side to side and the arms swinging wide. (fighter_view.gd squashes it harder on each footfall, `STEP_SQUASH`.)
    "heavy": dict(frames=22, hips=0.61, down=0.32, lift=0.16, kick=0.04, bob=0.12, lean=14.0, side=8.0, head=-7.0, twist=16.0, sway=9.0,
                  wide=0.1, arm_l=(4.0, 44.0, 60.0), arm_r=(6.0, 44.0, 64.0), swing_l=44.0, swing_r=44.0, sink=0.03),
    # The caster: neat and upright, the arms held in.
    "caster": dict(frames=21, hips=0.72, down=0.25, lift=0.24, kick=0.12, bob=0.05, lean=18.0, side=6.0, head=-10.0, twist=8.0, sway=0.0,
                   wide=0.0, arm_l=(22.0, 16.0, 84.0), arm_r=(18.0, 18.0, 88.0), swing_l=22.0, swing_r=22.0),
    # The tool users: a loose, bouncy jog, the arms swinging free.
    "tool": dict(frames=21, hips=0.7, down=0.25, lift=0.28, kick=0.14, bob=0.09, lean=16.0, side=5.0, head=-8.0, twist=12.0, sway=0.0,
                 wide=0.02, arm_l=(6.0, 24.0, 62.0), arm_r=(8.0, 24.0, 66.0), swing_l=52.0, swing_r=52.0),
}
for _st in RUN_STYLES.values():
    _st["sweep"] = _st["down"] * _st["frames"] * RUN_TEMPO / RUN_TURN_SHARE


def run_style(phase, st):
    """One frame of a family's run (`RUN_STYLES`) at `phase` (0..1). The left foot lands at phase 0, the right at 0.5."""
    def f(rig):
        rest_hips = rig.pose.bones["hips"].bone.head_local.z
        # 1 just after each landing, -1 in the flight between; a heavy run (`sink`) lands later in its drop, so the weight comes down
        # onto the step after the foot is planted.
        lowest = math.cos((phase - 0.05 - st.get("sink", 0.0)) * 4.0 * math.pi)
        roll = math.sin(phase * 2.0 * math.pi)
        pose(rig, "hips", lift=st["hips"] - rest_hips - st["bob"] * lowest, twist=-st["twist"] * roll, out=st["sway"] * roll)
        pose(rig, "spine", fwd=st["lean"] + 4.0 * lowest, twist=8.0 + st["twist"] * 1.4 * roll, out=st["side"])
        pose(rig, "head", fwd=st["head"] - 4.0 * lowest, twist=-4.0 - st["twist"] * 0.9 * roll)
        for side, arm, amp in (("L", st["arm_l"], st["swing_l"]), ("R", st["arm_r"], st["swing_r"])):
            # Each arm pumps with the other leg: back as its own leg lands in front, the elbow closing as it comes forward.
            swing = -math.cos((phase + (0.0 if side == "L" else 0.5)) * 2.0 * math.pi)
            far = FAR_ARM_FWD * 0.8 if side == "R" else 0.0
            pose(rig, "armU." + side, fwd=arm[0] + far + amp * swing, out=arm[1] - (FAR_ARM_IN if side == "R" else 0.0))
            pose(rig, "armL." + side, fwd=arm[2] + 0.35 * amp * max(0.0, swing))
        down, sweep = st["down"], st["sweep"]
        for side, offset in (("L", 0.0), ("R", 0.5)):
            p = (phase + offset) % 1.0
            if p < down:
                u = p / down
                ahead = sweep * (0.5 - u)
                height = 0.0
                toe = -18.0 * max(0.0, u - 0.6) / 0.4
            else:
                u = (p - down) / (1.0 - down)
                # (Leaving still going back and landing already going back, as the walk's foot; the heel kicks up behind early in the
                # swing, then the knee drives through and the foot reaches ahead.)
                # (It reaches a little past its landing point and paws back onto it, so it touches down already moving back with the
                # ground; both ends of the lift are steep, so it is never skimming the floor.)
                m0, m1 = -sweep * 0.35, -sweep * 1.6
                h00, h10, h01, h11 = 2 * u ** 3 - 3 * u * u + 1, u ** 3 - 2 * u * u + u, -2 * u ** 3 + 3 * u * u, u ** 3 - u * u
                ahead = h00 * (-0.5 * sweep) + h10 * m0 + h01 * (0.5 * sweep) + h11 * m1
                ahead -= st["kick"] * math.sin(math.pi * min(1.0, u / 0.55))
                height = st["lift"] * math.sin(math.pi * u) ** 0.6
                toe = -30.0 * math.sin(math.pi * u * 2.0) if u < 0.5 else 16.0 * math.sin(math.pi * (u - 0.5) * 2.0)
            miss = foot_to(rig, side, ahead, height, st["wide"])
            if miss > 0.01:
                print("run %s: the %s foot misses by %.3f at %.2f" % (CLIP_NOW, side, miss, phase))
            _level_sole(rig, side)
            rig.pose.bones["foot." + side].rotation_euler.x += math.radians(-toe)
    return f


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
            # (Round 2, with the side-on turn: the far arm's swing centred further forward, so it shows past the chest.)
            bias = FAR_ARM_FWD * 0.8 if side == "R" else 0.0
            pose(rig, "armU." + side, fwd=arms * (-62.0 * swing + 8.0) + bias, out=28.0 + 10.0 * forward - (FAR_ARM_IN if side == "R" else 0.0))
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
    # (`--out <glb>`: write the rig somewhere else, for trying a body out before it replaces the game's.)
    if "--out" in sys.argv:
        OUT = os.path.abspath(sys.argv[sys.argv.index("--out") + 1])
## How big the fists are against the blob's (the concept's mitten hands are small: about a quarter of the head across).
FIST_SCALE = 0.74
# The glove's fist (it is built bigger and rounder than the bare hand, with knuckles and a cuff): smaller, or a gloved fighter carried two
# white balloons.
GLOVE_SCALE = 0.74
GLOVE_GROW = 1.0     # the base body's glove against its bare hand (1.1 until the aviator scout pilot, 2026-10-19: the reference's gloves are slim, ours read puffy)
## The wrist radius FIST_SCALE and FIST_CLOSED are set for (the cage body's, 2026-10-15); a body with thicker wrists gets bigger hands.
HAND_WRIST = 0.072   # (0.082 until the character judge's round 19: the slimmer arm tapers into the hand, which keeps its size)
FIST_CLOSED = 0.74   # the closed fist (`fist`, built to the open hand's size) against the blob's
## How far the head is lowered onto the shoulders: the old generated body had a short neck between them, which the character has none of.
## The scripted body (make_base_body.py) has no neck, so it is built with `--neck-drop 0`.
NECK_DROP = float(sys.argv[sys.argv.index("--neck-drop") + 1]) if "--neck-drop" in sys.argv else 0.07


ARM_BLEND = 0.2          # the arm's own radius for this (as near_arm's)
ARM_BLEND_FADE = 0.14    # beyond it the arm's pull on the trunk fades out over this much
SHOULDER_HOLD = 0.08     # within this distance of the shoulder joint the trunk keeps its arm weights
SHOULDER_FADE = 0.1     # and over this much further they fade to none


def near_arm(co, J, radius=0.2):
    """Whether a rest-pose point is on an arm: within `radius` of the line from that side's shoulder joint to its wrist, below the
    shoulder. (Being inside the shoulder joint's x is not enough to call a point the trunk's: the inside of the upper arm is too, and
    taken for the trunk it stayed behind when the arm moved, a crumpled web of spikes under the elbow.)"""
    side = "R" if co.x > 0.0 else "L"
    a, b = J["armU." + side], J["hand." + side]
    if co.z > a.z + 0.05:
        return False
    ab = b - a
    t = max(0.0, min(1.0, (co - a).dot(ab) / ab.length_squared))
    return (co - (a + ab * t)).length < radius


ANALYTIC_WEIGHTS = True
ARM_ROOT_BLEND = 0.15     # over how much of the arm's start (inward of the shoulder joint, toward the trunk) the arm takes over
ELBOW_BLEND = 0.09       # half the length over which the upper arm hands over to the forearm
HIP_BLEND = 0.15         # over how much height above the crotch the trunk hands over to each thigh
KNEE_BLEND = 0.06
ANKLE_BLEND = 0.04
NECK_BLEND = 0.05
WAIST_BLEND = 0.16        # half the height over which the hips hand over to the spine


def _smooth01(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a))) if b != a else (1.0 if x >= a else 0.0)
    return t * t * (3.0 - 2.0 * t)


def analytic_weights(skin, J):
    """The skin's bone weights from the skeleton alone (see the call). Each point: how much it belongs to an arm (by its distance along
    and from the arm's line, from the shoulder joint out), to a leg (by height below the crotch, on its side), the rest to the trunk (hips
    to spine by height, the head above the neck); within a limb, the upper and lower bones blend at the joint. Symmetric by construction."""
    groups = {}
    for name in ("hips", "spine", "head") + tuple(b + "." + s for s in "LR" for b in ("armU", "armL", "thigh", "shin", "foot")):
        groups[name] = skin.vertex_groups.get(name) or skin.vertex_groups.new(name=name)
    every = list(range(len(skin.data.vertices)))
    for g in skin.vertex_groups:
        g.remove(every)
    crotch = FIT["crotch"]
    neck = FIT["neck"]
    hips_z = J["hips"].z
    spine_z = J["spine"].z
    for v in skin.data.vertices:
        co = v.co
        side = "R" if co.x > 0.0 else "L"
        sh, el, wr = J["armU." + side], J["armL." + side], J["hand." + side]
        # The arm: along the shoulder-wrist line, from a little inside the shoulder joint, and near the line.
        axis = wr - sh
        t = (co - sh).dot(axis) / axis.length_squared
        closest = sh + axis * max(0.0, min(1.0, t))
        near = (co - closest).length
        reach = t * axis.length
        w_arm = _smooth01(-ARM_ROOT_BLEND, 0.02, reach) * (1.0 - _smooth01(ARM_RADIUS * 0.9, ARM_RADIUS * 1.15, near))
        if co.z > sh.z + 0.12:
            w_arm = 0.0
        # The leg: below the crotch on its side.
        th, kn, an = J["thigh." + side], J["shin." + side], J["foot." + side]
        # (By height, and on the inner side of the thigh lower still, so the crotch's middle stays with the hips while the inside of each
        # thigh goes with its leg: given to the hips by distance from the middle alone, it stretched into a sheet between the legs in a
        # wide stance.)
        inner = 1.0 - _smooth01(0.0, abs(th.x), abs(co.x))
        top = crotch + HIP_BLEND * (1.0 - 0.7 * inner)
        w_leg = _smooth01(top, top - HIP_BLEND * 0.8, co.z) * _smooth01(0.0, 0.015, abs(co.x))
        w_leg = min(w_leg, 1.0 - w_arm)
        w_trunk = max(0.0, 1.0 - w_arm - w_leg)
        out = {}
        if w_arm > 0.0:
            elbow_at = (el - sh).dot(axis) / axis.length
            lower = _smooth01(elbow_at - ELBOW_BLEND, elbow_at + ELBOW_BLEND, reach)
            out["armU." + side] = w_arm * (1.0 - lower)
            out["armL." + side] = w_arm * lower
        if w_leg > 0.0:
            shin = _smooth01(kn.z + KNEE_BLEND, kn.z - KNEE_BLEND, co.z)
            foot = _smooth01(an.z + ANKLE_BLEND, an.z - ANKLE_BLEND, co.z)
            out["thigh." + side] = w_leg * (1.0 - shin)
            out["shin." + side] = w_leg * shin * (1.0 - foot)
            out["foot." + side] = w_leg * foot
        if w_trunk > 0.0:
            head = _smooth01(neck - NECK_BLEND, neck + NECK_BLEND, co.z)
            mid = (hips_z + spine_z) / 2.0 + 0.08
            spine = _smooth01(mid - WAIST_BLEND, mid + WAIST_BLEND, co.z)
            out["head"] = w_trunk * head
            out["spine"] = w_trunk * (1.0 - head) * spine
            out["hips"] = w_trunk * (1.0 - head) * (1.0 - spine)
        for name, w in out.items():
            if w > 1e-4:
                groups[name].add([v.index], w, "REPLACE")


def release_trunk(skin, J):
    """Fades the arm bones' weights out of the trunk (the points inside the shoulder joints' line) with distance from the shoulder joint,
    giving what they held to the spine, so a raised arm lifts the shoulder's round and not a web of the torso's side."""
    names = {g.index: g.name for g in skin.vertex_groups}
    spine = skin.vertex_groups["spine"]
    for v in skin.data.vertices:
        side = "R" if v.co.x > 0.0 else "L"
        joint = J["armU." + side]
        if abs(v.co.x) >= abs(joint.x) or v.co.z < J["hips"].z or near_arm(v.co, J):
            continue
        d = (v.co - joint).length
        keep = max(0.0, min(1.0, (SHOULDER_HOLD + SHOULDER_FADE - d) / SHOULDER_FADE))
        # And with distance from the arm itself: the trunk's skin just beside the arm follows it part way, fading out over
        # ARM_BLEND_FADE, so the armpit stretches evenly over a hand's width, not in a sheet from one ring of points (the owner: the arms
        # bent and contorted strangely; a swing back pulled a web from the torso's side).
        a, b = J["armU." + side], J["hand." + side]
        ab = b - a
        t = max(0.0, min(1.0, (v.co - a).dot(ab) / ab.length_squared))
        axis = (v.co - (a + ab * t)).length
        keep = max(keep, max(0.0, min(1.0, (ARM_BLEND + ARM_BLEND_FADE - axis) / ARM_BLEND_FADE)) * 0.85)
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
    # First the forearm is sliced through along the cut's plane, so the points beyond it go and the edge left is one straight ring
    # (deleting whole points alone left a staircase of quads round the wrist, a jagged collar the hand's plug did not cover).
    for side, sign in (("L", -1.0), ("R", 1.0)):
        s, e, w = J["armU." + side], J["armL." + side], J["hand." + side]
        length = (w - s).length
        fore = w - e
        plane = w + fore.normalized() * 0.03 * length
        near = [v for v in bm.verts if v.co.x * sign >= abs(s.x) * 0.7 and (v.co - e).dot(fore) / fore.length_squared > 0.5
                and (v.co - w).length < 0.35]
        nearset = set(near)
        edges = [ed for ed in bm.edges if ed.verts[0] in nearset and ed.verts[1] in nearset]
        faces = [f for f in bm.faces if all(v in nearset for v in f.verts)]
        bmesh.ops.bisect_plane(bm, geom=near + edges + faces, plane_co=plane, plane_no=fore.normalized(), dist=1e-5)
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
            if t > 0.9 and tf > 1.0 + 0.03 * length / fore.length + 1e-4:
                cut.append(v)
                break
            # (How thick the forearm is near the wrist, for the cuffs and the hand's plug: measured over the last stretch only, as the
            # cage body's forearm tapers and a plug as wide as its middle stood out of the wrist in a jagged ring.)
            if 0.88 < tf <= 1.0:
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
    wrist_q = {side: (sorted(rs)[len(rs) // 4] if rs else HAND_WRIST) for side, rs in wrist_r.items()}
    for side, x in (("L", -1.0), ("R", 1.0)):
        wrist_blob = Vector((x * BLOB_WRIST[0], BLOB_WRIST[1], BLOB_WRIST[2]))
        fore_blob = (wrist_blob - Vector((x * 0.66, -0.02, 0.95))).normalized()
        fore = (J["hand." + side] - J["armL." + side]).normalized()
        turn = fore_blob.rotation_difference(fore).to_matrix()
        # The wrist's radius (the lower quartile of the forearm's last stretch: inside the skin all round), for the wrist plug; and the
        # hands grow with the two wrists' mean, so a thicker or thinner body keeps hands in proportion to its arms (both the same size).
        rw = wrist_q[side]
        grow = (wrist_q["L"] + wrist_q["R"]) / 2.0 / HAND_WRIST
        print("hand", side, "wrist radius", round(rw, 4), "grow", round(grow, 3))
        # The wrist plug (below): sunk inside the forearm up to the skin's cut and only past it as wide as the wrist, so it closes the cut
        # without crossing the skin (level with it, the faceted skin and the round plug crossed in a zigzag ring round the wrist).
        cut_d = 0.03 * (J["hand." + side] - J["armU." + side]).length
        wrist_at = J["hand." + side]

        def plug(bm):
            tube(bm, [wrist_at - fore * 0.07, wrist_at + fore * (cut_d - 0.005), wrist_at + fore * (cut_d + 0.03)],
                 [rw * 0.86, rw * 0.9, rw * 1.04], segments=18, per_span=3)
        # (The glove is the bare hand's shape a size up, not the blob's clenched ball: gloved fighters carried two white balloons.)
        bm = bmesh.new()
        mitten(bm, x)
        for v in bm.verts:
            v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * FIST_SCALE * GLOVE_GROW * grow
        parts["Hand." + side] = to_object("Hand." + side, bm, "hand." + side, rig)
        # The bare hand (no gloves in the loadout): the game shows this one instead, in the body's colour.
        bm = bmesh.new()
        mitten(bm, x)
        for v in bm.verts:
            v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * FIST_SCALE * grow
        # A wrist plug: the forearm skin is cut open at the wrist and the hand's rounded top did not fill the cut, so its inside showed as a
        # dark ring (a bracelet). A short round plug the width of the wrist closes it, running a little up into the forearm.
        plug(bm)
        parts["Mitt." + side] = to_object("Mitt." + side, bm, "hand." + side, rig)
        # (No ink where the hand goes into the forearm: the hull of its rounded top showed as a dark ring round the wrist, a bracelet. The
        # vertex colour's red is how much outline a point gets, shaders/outline.gdshader.)
        mitt = parts["Mitt." + side]
        ink = mitt.data.color_attributes.new("outline", "FLOAT_COLOR", "POINT")
        for v in mitt.data.vertices:
            up = (v.co - wrist_at).dot(-fore)
            # (Over a longer stretch, the character pass, 2026-10-19: the line came back just past the wrist as a ring.)
            w = min(1.0, max(0.0, (-0.05 - up) / 0.09))
            ink.data[v.index].color = (w, 1.0, 1.0, 1.0)
        mitt.data.color_attributes.active_color = ink
        # Closed fists to match, bare and gloved (`FistBare`, `FistGlove`): the game shows them on a hand that grips a weapon and on
        # an unarmed fighter's hands while it attacks (an open hand beside a sword's hilt read as not holding it, a punch as a slap).
        for name, k in (("FistBare." + side, FIST_CLOSED), ("FistGlove." + side, FIST_CLOSED * GLOVE_GROW)):
            bm = bmesh.new()
            fist(bm, x)
            for v in bm.verts:
                v.co = J["hand." + side] + turn @ (v.co - wrist_blob) * k * grow
            if name.startswith("FistBare"):
                plug(bm)
            parts[name] = to_object(name, bm, "hand." + side, rig)
            if name.startswith("FistBare"):
                f = parts[name]
                ink = f.data.color_attributes.new("outline", "FLOAT_COLOR", "POINT")
                for v in f.data.vertices:
                    up = (v.co - wrist_at).dot(-fore)
                    ink.data[v.index].color = (min(1.0, max(0.0, (-0.05 - up) / 0.09)), 1.0, 1.0, 1.0)
                f.data.color_attributes.active_color = ink
        radii_at_wrist = sorted(wrist_r[side])
        r = radii_at_wrist[len(radii_at_wrist) // 2] if radii_at_wrist else 0.08
        bm = bmesh.new()
        # (A longer flared cuff, the character judge's round 1: a short one left a ring of skin showing at the wrist.)
        bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=24, radius1=r * 1.2, radius2=r * 1.5, depth=0.08)
        bmesh.ops.transform(bm, matrix=Vector((0.0, 0.0, 1.0)).rotation_difference(fore).to_matrix().to_4x4(), verts=bm.verts)
        # (Short, so a clear stretch of forearm shows between it and a short sleeve, as on the reference's scouts, the character judge's
        # round 4; right up against the glove, round 12: set back down the forearm it floated apart from the hand when the wrist bent.)
        bmesh.ops.translate(bm, vec=J["hand." + side] + fore * 0.005, verts=bm.verts)
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
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.y > centre.y + FACE_WRAP * radii.y], context="VERTS")
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
    # How far forward each point of the shell lies (in head radii, as the texture map's x), in its colour's green (FACE_DEPTH_CODE; blue 0
    # marks it there): face.gdshader slides the drawing round the head by turning each point about the head's upright axis and
    # projecting it straight on again. Worked out from a ball instead, the head's sides, wider than the ball, all took one angle and
    # showed a stretched sliver of the drawing once the shell wrapped round them (FACE_WRAP).
    f = parts["Face"]
    depth = f.data.color_attributes.new("depth", "FLOAT_COLOR", "POINT")
    lo_d, span_d = FACE_DEPTH_CODE
    for v in f.data.vertices:
        d = (centre.y - v.co.y) / radii.x
        depth.data[v.index].color = (1.0, min(1.0, max(0.0, (d - lo_d) / span_d)), 0.0, 1.0)
    f.data.color_attributes.active_color = depth

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
    trunk_ids = [v.index for v in skin.data.vertices
                 if abs(v.co.x) < abs(J["armU.R"].x) * 0.72 and v.co.z > J["hips"].z
                 and not near_arm(v.co, J, ARM_BLEND + ARM_BLEND_FADE)]
    if ANALYTIC_WEIGHTS:
        # (Art review 5: the automatic weights, and every patch laid over them, left the chest partly following the arms, so on a swing the
        # torso stretched and puffed out as if breathing in. The rebuilt body is a clean, symmetric shape, so its weights are worked out
        # from the skeleton instead: the trunk follows the spine and hips alone, each limb takes over within a short band at its root, and
        # every joint blends over the same short stretch on both sides.)
        analytic_weights(skin, J)
    else:
        smooth_weights(skin, [g + "." + side for side in "LR" for g in ("armU", "armL")], 0.5, 12, trunk_ids)
        # And on the trunk the arm's pull fades with distance from the shoulder joint: the round of the shoulder rises with a raised arm,
        # the chest and the sides under it stay with the spine (the owner, 2026-10-14: the arms stretched the body and did not read as
        # shoulders).
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
        # (Tall, to just under the knee, the character judge's round 1 after the owner's reference: thick socks give the legs structure.)
        "sock_top": round(ankle + SOCK_HEIGHT, 4),
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
    # The collar: one band round the neck, folded over along its top edge, narrow at the back and widening into two points at the
    # front, laid on the shirt by walking down its surface from the neckline (so it lies on the chest in front and on the shoulders at
    # the sides alike). (Two flat triangles on the chest until the shirt pass, 2026-10-20: the owner, "it just reads as triangles on the
    # shirt"; nothing joined them round the neck.)
    ring = [v.co for v in sbm.verts if abs(v.co.z - neck_line) < 0.04]
    yc = sum(c.y for c in ring) / len(ring) if ring else 0.0

    def outward(p, n):
        axis = Vector((0.0, yc, p.z))
        return n if n.dot(p - axis) >= 0.0 else -n

    def walk(theta, length, steps):
        # From the neckline at this angle round the neck (0 the front), down the cloth's slope in equal steps.
        d = Vector((math.sin(theta), -math.cos(theta), 0.0))
        hit, _, _, _ = tree.ray_cast(Vector((0.0, yc, neck_line - 0.012)), d, 2.0)
        # (Only a hit close round the neck: wide of it, the ray can miss the cloth there and find a far part of the shirt.)
        if hit is None or (hit - Vector((0.0, yc, hit.z))).length > COLLAR_REACH:
            return None
        pts = [hit]
        step = length / max(steps, 1)
        for _ in range(steps):
            loc, nrm, _, _ = tree.find_nearest(pts[-1])
            nrm = outward(loc, nrm)
            t = Vector((0.0, 0.0, -1.0)) + nrm * nrm.z
            if t.length < 1e-4:
                t = d.copy()
            t.normalize()
            loc2, _, _, _ = tree.find_nearest(pts[-1] + t * step)
            # (A step that jumps has snapped to another part of the cloth: the column is left out.)
            if (loc2 - pts[-1]).length > 2.5 * step + 0.004:
                return None
            pts.append(loc2)
        return pts

    def width(theta):
        for (t0, w0), (t1, w1) in zip(COLLAR_SHAPE, COLLAR_SHAPE[1:]):
            if t0 <= theta <= t1:
                return w0 + (w1 - w0) * (theta - t0) / (t1 - t0)
        return COLLAR_SHAPE[-1][1]

    def collar(out_by, n_t=72, n_d=8):
        bm = bmesh.new()
        t_open = COLLAR_SHAPE[0][0]
        for sx in (-1.0, 1.0):
            cols = []
            for i in range(n_t + 1):
                th = t_open + (math.pi - 0.01 - t_open) * (i / n_t) ** 1.4
                cols.append(walk(th * sx, width(th), n_d))
            # A column left out is blended from its neighbours either side (and laid on the cloth below with the rest), so the band
            # runs on unbroken; joined straight across, a gap at the shoulder made one long face cutting through the shirt.
            good = [i for i, c in enumerate(cols) if c is not None]
            if not good:
                continue
            for i, c in enumerate(cols):
                if c is None:
                    lo = max([g for g in good if g < i], default=good[0])
                    hi = min([g for g in good if g > i], default=good[-1])
                    f = 0.0 if hi == lo else (i - lo) / (hi - lo)
                    cols[i] = [p.lerp(q, f) for p, q in zip(cols[lo], cols[hi])]
            # Smoothed along the band (each row averaged with its neighbours, then laid back on the cloth), so the free edge runs in one
            # clean curve instead of a row of small steps.
            for _ in range(COLLAR_SMOOTH):
                cols = [cols[0]] + [[(a + 2.0 * b + c) / 4.0 for a, b, c in zip(cols[k - 1], cols[k], cols[k + 1])]
                                    for k in range(1, len(cols) - 1)] + [cols[-1]]
            grid = {}
            for i, pts in enumerate(cols):
                for j, p in enumerate(pts):
                    loc, nrm, _, _ = tree.find_nearest(p)
                    nrm = outward(loc, nrm)
                    k = j / n_d
                    # The fold along the top stands off the shirt and rounds over; the free edge lifts a little, as a pressed collar does.
                    lift = out_by + COLLAR_FOLD * (1.0 - k) ** 2 + COLLAR_EDGE_LIFT * k ** 3
                    grid[i, j] = bm.verts.new(loc + nrm * lift)
            for (i, j), v in list(grid.items()):
                a, b, c = grid.get((i + 1, j)), grid.get((i, j + 1)), grid.get((i + 1, j + 1))
                if a and b and c:
                    bm.faces.new((v, a, c, b) if sx > 0 else (v, b, c, a))
        bmesh.ops.dissolve_degenerate(bm, dist=1e-5, edges=list(bm.edges))
        return bm

    # (One sheet, drawn from both sides as all the cloth is: thickened, the tiny faces at its points threw a few corners metres off.)
    out["ShirtCollar"] = finish("ShirtCollar", collar(0.012), 0.0)
    # The holiday shirt's camp collar: the same band, in the shirt's print (fighter_view.gd), standing a hair further off.
    # (Two squared cream lapels until the shirt pass, 2026-10-20: they read as a white slab across the shoulders.)
    out["ShirtCampCollar"] = finish("ShirtCampCollar", collar(0.016), 0.0)
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
        # (Smaller and flatter, the button pass, 2026-10-19: at 0.028 the three pale balls were the biggest detail on the chest.)
        bmesh.ops.create_uvsphere(b, u_segments=12, v_segments=8, radius=0.019)
        bmesh.ops.scale(b, vec=(1.0, 0.35, 1.0), verts=b.verts)
        bmesh.ops.translate(b, vec=co, verts=b.verts)
        merge(bm, b)
    out["Buttons"] = finish("Buttons", bm, 0.0)
    sbm.free()
    return out


# ---- Clothes: garments cut from the skin ------------------------------------------------------------------------------------------------
# (Puffier, art review 6: PEAK's shirts and shorts are thick soft garments standing off the body; ours hugged it like paint.)
# The shirt's collar (shirt_front): its width down the cloth (head-relative units) at each angle round the neck from the front (radians),
# so narrow at the back and widening into a point either side of the throat; how far from the neck's middle the neckline is looked for;
# how many smoothing passes along the band; how far the fold along its top and its free edge stand off the shirt.
COLLAR_SHAPE = [(0.1, 0.03), (0.42, 0.26), (0.9, 0.11), (1.6, 0.075), (math.pi, 0.065)]
COLLAR_REACH = 0.36
COLLAR_SMOOTH = 3
COLLAR_FOLD = 0.014
COLLAR_EDGE_LIFT = 0.01
SHIRT_LIFT = 0.085      # how far off the skin the shirt sits (0.065 until the body rebuild: it read painted on; the reference's shirts hang loose)
# Where the waist is, as a share of the way from the hips up to the neck. (The owner, 2026-10-14: PEAK's figures have a waistline; ours
# had the shorts' top almost at the crotch, 0.12, so the shirt ran in one straight box to the hips. Now the shorts come up to a belt at the
# body's narrowest, and the shirt is tucked in under it.)
WAIST_SHARE = 0.25
BELT_HALF = 0.045       # half the belt's height
BELT_LIFT = 0.085       # how far off the skin the belt sits (over the shirt's tucked hem and the shorts' waistband)
BLOUSE = 0.02          # how far the shirt puffs out just above the belt, where it is tucked in
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
SHORTS_KNEE_CLEAR = 0.02   # how far above the top of the knee's bend (the knee joint plus KNEE_BLEND) the shorts end
CUFF_LIFT = 0.072   # (outside the shorts' flared hem, the character judge's round 13: at 0.06 the shorts poked through it in teeth when a leg bent)
# The sleeves' trim (the aviator scout pilot, 2026-10-19, the owner's reference: a coloured band round each sleeve's end, standing proud),
# this share of the upper arm long and lifted this far off the skin.
SLEEVE_TRIM = 0.2
# How far round the shoulder joint the shirt takes the upper arm's bone, and how much of it at most.
SHOULDER_BLEND = 0.2
SHOULDER_SHARE = 0.55   # (the character judge's round 18: more lifted the shirt's shoulder into a flat shelf with the arm)
# Tall thick ribbed socks (the character judge's round 1): how far up from the ankle, how far off the skin, and their turned-down top.
SOCK_HEIGHT = 0.375   # (just under the knee-length shorts, the character judge's round 12: higher, the turn-down poked through them in a bend)
SOCK_LIFT = 0.03
SOCK_TOP_BAND = 0.045
SOCK_TOP_LIFT = 0.045
SLEEVE_TRIM_LIFT = 0.11   # (about 1.15 times the arm round, the character judge's round 18)
SLEEVE_SNUG = 0.55      # the sleeve's opening hugs the arm (this share of the shirt's distance), so no dark gap shows inside it
SHORTS_TOP = -0.002   # the shorts reach just under the shirt's hem (below it a little, they could poke through)
SHORTS_LIFT = 0.04     # the shorts sit a little closer in, so the shirt's hem lies over their waistband
CLOTH_THICKNESS = 0.0     # none: an inner layer crossed the outer one where the cloth folds (the game draws cloth two-sided)
SLEEVE = 0.85  # (a short puffed cap, the character judge's round 19 and the owner: about a fifth of shoulder to fingertip)
                       # shoulder, so 0.55 left only a cap over it: PEAK's short sleeves reach most of the way to the elbow)
HEM_FLARE = 0.035
# The body rebuild (the owner: the clothes read painted on beside the reference cast): the sleeves stand off the arm and open a little at
# the hem, the shorts' legs flare wide, as the reference's cloth hangs.
SLEEVE_FLARE = 0.02   # (0.05, puffed, the owner's choice in round 19, until the hem pass: the sleeves stood out as flat wings)
# How far off the arm a sleeve sits, as a share of the shirt's lift, over its length and at its end: a solid tube about 1.4 times the
# arm's width, as the references' short sleeves (the hem pass, 2026-10-19; the whole lift made them a shelf twice the arm's width).
SLEEVE_SIT = (0.6, 0.45)
SHORTS_FLARE = 0.075
ARM_RADIUS = 0.15   # (the slimmer arm, the character judge's round 19; 0.19 for the thicker sausage arm)     # how far from the arm's axis a point still counts as the arm (the cage body's arm is at most 0.125 round and hangs with
                       # the torso's side 0.16 from its axis, so this sits between them; at 0.2, made for the generated figure, the torso's
                       # sides took arm weights and tore when the arms moved)
HEM_SMOOTH = 30
TORSO_SLICE = 0.01       # the height of a slice of the torso that shares one blend of the trunk's bones
TORSO_SLICE_SPREAD = 4   # how many slices either side are averaged in       # how many smoothing steps each hem gets along its loop
WEIGHT_BLUR = 0.06     # how far round each cloth point the skin's bone weights are averaged
GARMENT_SMOOTH = 14     # how many smoothing steps the cloth gets (it drapes over the body's small bumps)
GARMENT_WEIGHT_SMOOTH = 24
# (0.07 until the armpit pass, 2026-10-20: with the arm swung forward, the shirt's cloth folds under the arm further out than that, and
# its outline drew the fold as a black arc across the side. The match's ink pass still draws the silhouette, and a light line for the fold.)
ARMPIT_NO_INK = 0.2          # how far round each armpit the shirt has no ink outline   # how many smoothing steps the cloth's bone weights get        # how much further out a hem stands than the rest of the garment (a soft, cartoony flare)


# The hem pass (2026-10-19, the owner's references: their hems are thick and clean, ours read as torn paper): the bands that end a garment
# get a round roll along each edge, this thick (radius), so the edge reads as cloth with body rather than a cut sheet.
HEM_ROLL = {"SockTop": 0.014, "BootCuff": 0.014}
# The shorts' cuffs and the sleeves' trim are built on the garment's own hem rather than cut from the skin and lifted: a solid band (a
# rounded section this tall and this thick) round each leg opening and sleeve end. (Cut from the skin, each sat at its own distance from
# the body while the garment's flared hem sat at another, so the two crossed: the cuff showed only as a ragged fringe under the hem, and
# the sleeve end as a loose flap.)
CUFF_BAND = (0.06, 0.024)
TRIM_BAND = (0.05, 0.022)
BAND_STEP = 0.025       # the spacing of a band's sections along the hem
HEM_ROLL_SEGMENTS = 8


def hem_rolls(bm, deform, radius):
    """A tube along every open edge of the garment in `bm`, centred on the edge and lifted half its radius off the cloth's outer side,
    each ring taking the bone weights of the edge point it is built round (so it moves exactly as the edge does)."""
    bm.normal_update()
    nbrs = {}
    for e in bm.edges:
        if e.is_boundary:
            a, b = e.verts
            nbrs.setdefault(a, []).append(b)
            nbrs.setdefault(b, []).append(a)
    seen = set()
    new_faces = []
    for start in list(nbrs):
        if start in seen or len(nbrs[start]) != 2:
            continue
        loop = [start]
        seen.add(start)
        prev, cur = None, start
        while True:
            nxt = [n for n in nbrs[cur] if n is not prev]
            if not nxt or len(nbrs[cur]) != 2:
                loop = None
                break
            prev, cur = cur, nxt[0]
            if cur is start:
                break
            if cur in seen:
                loop = None
                break
            seen.add(cur)
            loop.append(cur)
        if not loop or len(loop) < 6:
            continue
        # (Sections every BAND_STEP along the edge, not at every one of its points: thousands of triangles otherwise.)
        picked, run = [loop[0]], 0.0
        for p0, p1 in zip(loop, loop[1:]):
            run += (p1.co - p0.co).length
            if run >= BAND_STEP:
                picked.append(p1)
                run = 0.0
        if len(picked) < 6:
            continue
        loop = picked
        rings = []
        n_loop = len(loop)
        for i, v in enumerate(loop):
            t = (loop[(i + 1) % n_loop].co - loop[i - 1].co).normalized()
            nrm = v.normal.normalized()
            across = t.cross(nrm).normalized()
            # (Pointing off the cloth, away from the faces at this edge.)
            inward = sum((f.calc_center_median() for f in v.link_faces), Vector()) / max(1, len(v.link_faces)) - v.co
            if across.dot(inward) > 0.0:
                across = -across
            centre = v.co + nrm * radius * 0.5
            ring = []
            for k in range(HEM_ROLL_SEGMENTS):
                a = 2.0 * math.pi * k / HEM_ROLL_SEGMENTS
                nv = bm.verts.new(centre + (nrm * math.cos(a) + across * math.sin(a)) * radius)
                for g, w in v[deform].items():
                    nv[deform][g] = w
                ring.append(nv)
            rings.append(ring)
        for i in range(n_loop):
            r0, r1 = rings[i], rings[(i + 1) % n_loop]
            for k in range(HEM_ROLL_SEGMENTS):
                k1 = (k + 1) % HEM_ROLL_SEGMENTS
                new_faces.append(bm.faces.new((r0[k], r0[k1], r1[k1], r1[k])))
    if new_faces:
        bmesh.ops.recalc_face_normals(bm, faces=new_faces)
        for f in new_faces:
            f.smooth = True


def _boundary_loops(bm):
    """The garment's open edges as ordered, closed loops of verts."""
    nbrs = {}
    for e in bm.edges:
        if e.is_boundary:
            a, b = e.verts
            nbrs.setdefault(a, []).append(b)
            nbrs.setdefault(b, []).append(a)
    seen, loops = set(), []
    for start in list(nbrs):
        if start in seen or len(nbrs[start]) != 2:
            continue
        loop, prev, cur = [start], None, start
        seen.add(start)
        while True:
            nxt = [n for n in nbrs[cur] if n is not prev]
            if not nxt or len(nbrs[cur]) != 2:
                loop = None
                break
            prev, cur = cur, nxt[0]
            if cur is start:
                break
            if cur in seen:
                loop = None
                break
            seen.add(cur)
            loop.append(cur)
        if loop and len(loop) >= 6:
            loops.append(loop)
    return loops


def band_on_hems(garment, skin, name, keep, height, thickness):
    """A solid band round each open edge of `garment` that `keep(loop)` accepts: a rounded section `height` tall (from just below the
    edge up over the cloth) and `thickness` deep (its outer face standing off the cloth), every section taking the bone weights of the
    hem point it is built on, so the band moves exactly as the hem does. Returns the new object (a copy of the skin's object, for its
    bone groups and armature)."""
    src = bmesh.new()
    src.from_mesh(garment.data)
    src.normal_update()
    sdef = src.verts.layers.deform.verify()
    bm = bmesh.new()
    deform = bm.verts.layers.deform.verify()
    faces = []
    seg = 12
    for loop in _boundary_loops(src):
        if not keep(loop):
            continue
        picked, run = [loop[0]], 0.0
        for a, b in zip(loop, loop[1:]):
            run += (b.co - a.co).length
            if run >= BAND_STEP:
                picked.append(b)
                run = 0.0
        n = len(picked)
        if n < 6:
            continue
        # The band's frame from the whole opening, not each hem point's own normal (the hem follows every small bump, and a band laid on
        # it wobbled in and out, so the garment showed through it in teeth): the opening's middle and axis, and round them each point's
        # direction and distance, smoothed along the loop.
        c = sum((v.co for v in picked), Vector()) / n
        axis = Vector()
        for i in range(n):
            p0, p1 = picked[i].co - c, picked[(i + 1) % n].co - c
            axis += p0.cross(p1)
        axis.normalize()
        inward = sum((f.calc_center_median() - v.co for v in picked for f in v.link_faces), Vector())
        if axis.dot(inward) < 0.0:
            axis = -axis
        radial = []
        for i in range(n):
            q = sum((picked[(i + k) % n].co for k in range(-2, 3)), Vector()) / 5.0 - c
            radial.append((q - axis * q.dot(axis), q.dot(axis)))
        rings = []
        for i, v in enumerate(picked):
            r = max(sum(radial[(i + k) % n][0].length for k in range(-2, 3)) / 5.0, radial[i][0].length)
            nrm = radial[i][0].normalized()
            up = axis
            base = c + axis * radial[i][1] + nrm * r
            centre = base + up * (height * 0.5 - 0.012) + nrm * thickness * 0.55
            ring = []
            for k in range(seg):
                a = 2.0 * math.pi * k / seg
                cu, sw = math.cos(a), math.sin(a)
                # (A rounded rectangle, not an ellipse: a cuff is a flat band with soft edges.)
                cu = math.copysign(abs(cu) ** 0.5, cu)
                sw = math.copysign(abs(sw) ** 0.5, sw)
                nv = bm.verts.new(centre + up * (height * 0.5 * sw) + nrm * (thickness * 0.5 * cu))
                for g, w in v[sdef].items():
                    nv[deform][g] = w
                ring.append(nv)
            rings.append(ring)
        for i in range(n):
            r0, r1 = rings[i], rings[(i + 1) % n]
            for k in range(seg):
                k1 = (k + 1) % seg
                faces.append(bm.faces.new((r0[k], r0[k1], r1[k1], r1[k])))
    src.free()
    if faces:
        bmesh.ops.recalc_face_normals(bm, faces=faces)
        for f in faces:
            f.smooth = True
    obj = skin.copy()
    # (A copy of the skin's mesh, its geometry then replaced: the bone groups' names live on the mesh, and a new empty mesh had none,
    # so the band's weights were dropped and it stayed behind when the limb moved.)
    obj.data = skin.data.copy()
    obj.data.name = name
    obj.name = name
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(obj.data)
    bm.free()
    for layer in list(obj.data.color_attributes):
        obj.data.color_attributes.remove(layer)
    return obj


def build_garments(skin, J, neck):
    """A shirt (from the waist to the neck, short sleeves) and shorts (from the waist to above the knee) for the base body. Each is a copy of
    the skin cut along its hems (planes, so the edges are straight however the mesh is laid out), keeping the faces in the garment's region,
    pushed out along the surface (further near a hem, so it flares a little) and given thickness. The copies keep the skin's bone weights,
    so the clothes move exactly with the body. The game colours them (by name: "Shirt", "Shorts")."""
    hips = J["hips"].z
    crotch = FIT["crotch"]
    knee = (J["shin.L"].z + J["shin.R"].z) / 2.0
    waist = hips + WAIST_SHARE * (neck - hips)
    # (Longer, half way to the knee, the body rebuild: the reference's shorts are long and wide.)
    # (Down to the knee, overlapping the socks' tops in a crouch, the character judge's round 9: a band of skin showed above them.)
    # (Just above the knee's bend since the cloth clean-up, the owner's choice, 2026-10-19: down to it, the cuff lay over the joint and
    # the skin folded out through it in the stance's slight bend, a band of bare leg and white teeth under the shorts. A little bare knee
    # shows above the socks now, as on the reference cast.)
    shorts_end = knee + KNEE_BLEND + SHORTS_KNEE_CLEAR
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

    def in_toe(co):
        # The shoe's toe cap: the front of the foot, under the instep (the character judge's round 11: the shoes read as rounded lumps).
        return co.z < boot_top - 0.06 and co.y < -0.12 and co.z > SOLE_HEIGHT

    def in_sole(co):
        return co.z < SOLE_HEIGHT

    def in_cuff(co):
        return not on_arm(co) and shorts_end < co.z < cuff_top and co.z < crotch

    sock_top = ankle + SOCK_HEIGHT

    def in_sock(co):
        return not on_arm(co) and boot_top - 0.02 < co.z < sock_top - SOCK_TOP_BAND

    def in_sock_top(co):
        return not on_arm(co) and sock_top - SOCK_TOP_BAND < co.z < sock_top

    def in_sleeve_trim(co):
        return co.z < neck_line and on_arm(co) and SLEEVE - SLEEVE_TRIM < sleeve_t(co) < SLEEVE

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
            (Vector((0, 0, cuff_top)), Vector((0, 0, 1))), (Vector((0, 0, boot_top - BOOT_CUFF)), Vector((0, 0, 1))),
            (Vector((0, 0, sock_top)), Vector((0, 0, 1))), (Vector((0, 0, sock_top - SOCK_TOP_BAND)), Vector((0, 0, 1))),
            (Vector((0, 0, boot_top - 0.02)), Vector((0, 0, 1))),
            (Vector((0, -0.12, 0)), Vector((0, 1, 0))), (Vector((0, 0, boot_top - 0.06)), Vector((0, 0, 1)))]
    for side in "LR":
        a, b = arm[side]
        cuts.append((a + (b - a) * SLEEVE, (b - a).normalized()))
        cuts.append((a + (b - a) * (SLEEVE - SLEEVE_TRIM), (b - a).normalized()))
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
                                    ("BootSole", in_sole, belt_hem, SOLE_LIFT), ("BootCuff", in_boot_cuff, belt_hem, BOOT_CUFF_LIFT),
                                    ("BootToe", in_toe, belt_hem, BOOT_LIFT + 0.014),
                                    ("Sock", in_sock, belt_hem, SOCK_LIFT), ("SockTop", in_sock_top, belt_hem, SOCK_TOP_LIFT)):
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
        # Where each point lay on the skin, before the cloth is smoothed and pushed out: whether it is sleeve or body, and whose weights
        # it takes, are told from there. (Told from the pushed-out cloth, a puffed sleeve's sides lay further from the arm's line than
        # ARM_RADIUS, so they took the torso's bones and stayed behind when the arm moved, tearing the sleeve; their hem points missed the
        # sleeve's hem smoothing and stood out as spikes.)
        src = {v: v.co.copy() for v in bm.verts}
        sleeve_pts = {v for v in bm.verts if on_arm(src[v])}
        # Cloth drapes over the body's small bumps (the belly, the chest) rather than following each one: smooth the surface, the hems
        # held where they are, then push any point the smoothing pulled toward the body back out to the garment's distance from it.
        hems = {v for v in bm.verts if v.is_boundary}
        inner = [v for v in bm.verts if v not in hems]
        # (The shorts twice over, 2026-10-18: over the hips and the crotch the cloth kept the body's lumps and shaded in blotches.)
        for _ in range(GARMENT_SMOOTH * (3 if name == "Shorts" else 1)):
            bmesh.ops.smooth_vert(bm, verts=inner, factor=0.5, use_axis_x=True, use_axis_y=True, use_axis_z=True)
        bm.normal_update()
        for v in bm.verts:
            flare = HEM_FLARE * (1.0 - min(1.0, hem(v.co) / 0.06))
            # The shorts' waistband hugs the body (two thirds of their distance), well inside the shirt's hem over it.
            snug = 0.67 if name == "Shorts" and v.co.z > waist - 0.05 else 1.0
            if name == "Shorts":
                flare = SHORTS_FLARE * (1.0 - min(1.0, hem(v.co) / 0.16)) ** 1.5
                # None on the inside of each leg, facing the other one: with the hem above the knee, where the thighs are close, the
                # two leg openings flared into each other and crossed between the legs.
                if v.co.z < crotch:
                    outward = v.normal.x * (1.0 if v.co.x > 0.0 else -1.0)
                    flare *= max(0.0, min(1.0, (outward + 0.6) / 0.8))
            if name == "Shirt" and v not in sleeve_pts and v.co.z > neck_line - 0.06:
                flare = 0.0
            elif name == "Shirt" and v not in sleeve_pts:
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
                near_end = 1.0 - min(1.0, abs(sleeve_t(v.co) - SLEEVE) * (b - a).length / 0.14)
                if abs(v.co.z - neck_line) > 0.06:
                    # (Standing off the arm and opening at the hem, the body rebuild; a snug band read as a painted sleeve.)
                    # (Not too open: the arm's ink hull showed through the gap as a dark patch under the arm.)
                    flare = SLEEVE_FLARE * near_end
                    snug = SLEEVE_SIT[0] + (SLEEVE_SIT[1] - SLEEVE_SIT[0]) * near_end
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
        sleeve_hem = hems & sleeve_pts
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
                  "ShortsCuff": [shorts_end, cuff_top], "SleeveTrim": [], "Sock": [boot_top - 0.02, sock_top - SOCK_TOP_BAND],
                  "SockTop": [sock_top - SOCK_TOP_BAND, sock_top], "BootToe": []}[name]
        for v in hems:
            if v in sleeve_pts or not levels:
                continue
            nearest = min(levels, key=lambda z: abs(v.co.z - z))
            if abs(v.co.z - nearest) < 0.04:
                # (Level since the hem pass, 2026-10-19: the round 14 waver, meant against a machined look, read as torn paper on a
                # thin edge; the rolled hems below give the edge its softness.)
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
            # (From the point's place on the skin: see `src`.)
            co = Vector((-src[v].x, src[v].y, src[v].z)) if left else src[v]
            # The skin's weights blurred over a few centimetres round the point (Blender's automatic weights alternate between bones
            # from one point to the next, which made a hem zigzag in a pose); the sleeves keep the nearest point's.
            if v in sleeve_pts:
                _, i, _ = skin_points.find(co)
                near = [(skin.data.vertices[i], 1.0)]
            else:
                found = torso_points.find_range(co, WEIGHT_BLUR)
                if not found:
                    found = [torso_points.find(co)]
                near = [(skin.data.vertices[i], max(0.0, 1.0 - d / WEIGHT_BLUR) + 1e-4) for _, i, d in found]
            blend = {}
            even = None if v in sleeve_pts or co.z < crotch else trunk_weights(co.z)
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
            if name == "Shirt" and v not in sleeve_pts:
                # (The shirt's shoulder follows the upper arm, the character judge's round 12: tied to the torso alone, the sleeve stayed a
                # fixed box and the arm swung out of it as from a doll's socket.) Over the shoulder cap a share of the upper arm's bone,
                # most at the armhole, none by the collar and the chest.
                sh = J["armU." + ("L" if co.x < 0.0 else "R")]
                dist = (Vector((abs(co.x), co.y, co.z)) - Vector((abs(sh.x), sh.y, sh.z))).length
                t = max(0.0, 1.0 - dist / SHOULDER_BLEND) * SHOULDER_SHARE
                # (Over the shoulder only, none under the arm, the character judge's round 19: the armpit's cloth taking the arm's bone
                # tore into spikes when the arm rose.)
                t *= max(0.0, min(1.0, (co.z - sh.z + 0.1) / 0.08))
                arm_g = index.get("armU." + ("L" if v.co.x < 0.0 else "R"))
                if t > 0.0 and arm_g is not None:
                    for k in list(v[deform].keys()):
                        v[deform][k] = v[deform][k] * (1.0 - t)
                    v[deform][arm_g] = v[deform].get(arm_g, 0.0) + t
            if name in ("Shorts", "ShortsCuff") and v.co.z < crotch:
                # Below the crotch each leg of the shorts follows its own thigh, fully at the hem: the leg openings keep their place on
                # the thighs in any pose rather than riding up or standing off like a skirt.
                t = min(1.0, (crotch - v.co.z) / max(crotch - shorts_end, 0.01) * 1.4)
                thigh = index.get("thigh." + ("L" if v.co.x < 0.0 else "R"))
                if thigh is not None:
                    for k in list(v[deform].keys()):
                        v[deform][k] = v[deform][k] * (1.0 - t)
                    v[deform][thigh] = v[deform].get(thigh, 0.0) + t
        # A rolled hem along both edges of each band (the cuffs, the sleeve's trim, the socks' turn-down): see `hem_rolls`.
        if name in HEM_ROLL:
            hem_rolls(bm, deform, HEM_ROLL[name])
        # The sleeves' weights as copied from the skin under them, kept to put back after the shirt's weights are smoothed below.
        bm.verts.index_update()
        sleeve_w = {v.index: dict(v[deform]) for v in sleeve_pts}
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
            # The sleeves bend exactly as the arm's skin under them: the smoothing and `release_trunk` above (both told sleeve from body by
            # the pushed-out cloth) gave the puffed sleeve's underside to the spine, so a raised arm left it behind in torn white sheets.
            groups = list(obj.vertex_groups)
            for i, w in sleeve_w.items():
                for g in groups:
                    g.remove([i])
                for gi, x in w.items():
                    groups[gi].add([i], x, "REPLACE")
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
    # The cuffs and the sleeves' trim, on the finished hems (see CUFF_BAND).
    out["ShortsCuff"] = band_on_hems(out["Shorts"], skin, "ShortsCuff",
                                     lambda loop: sum(v.co.z for v in loop) / len(loop) < crotch, *CUFF_BAND)
    out["SleeveTrim"] = band_on_hems(out["Shirt"], skin, "SleeveTrim",
                                     lambda loop: abs(sum(v.co.x for v in loop) / len(loop)) > 0.3, *TRIM_BAND)
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


# The base body's battle stance (the sword family's idle, `idle`): the front and back thigh and shin angles before `plant`, how far the
# legs open, and the chest's lean into the stance. (Animation overhaul, stance pass: was 32/-40 front and -14/-10 back, a near-straight
# stand with the hips 0.09 down.)
# (Round 2, 2026-10-25: at the match camera the head still sat straight over the hips. The chest goes further down and forward (lean 14
# to 26, side bend 12 to 20) over deeper knees (lead 44/-72 to 50/-82, back -20/-46 to -24/-54), so the head is out over the front knee.)
# (Round 8, the owner's pick C (2026-10-28) after the stance mock: the knees bent half as far and the feet about a third as far apart in
# depth, nearer the reference fighters' stances (knees 20-40 degrees, feet about hip-wide). Was 50/-82, -24/-54 and 19: the lead knee bent
# 90-100 degrees, the feet about 1.0 apart in depth. STANCE_K and STANCE_W scale every family's style (`IDLE_STYLES`) the same way.)
STANCE_K, STANCE_W = 0.5, 0.35
STANCE_LEAD, STANCE_LEAD_SHIN = 50.0 * STANCE_K, -82.0 * STANCE_K
STANCE_BACK, STANCE_BACK_SHIN = -24.0 * STANCE_K, -54.0 * STANCE_K
STANCE_WIDE = 19.0 * STANCE_W
# The back thigh's extra turn out, and the feet's counter-turn, scaled with the width.
STANCE_WIDE_BACK = 4.0 * STANCE_W
STANCE_FOOT_OUT = -10.0 * STANCE_W
STANCE_LEAN = 26.0
# The chest's bend toward the way the fighter faces (degrees): the game turns the body most of the way to the camera, so a forward bend
# alone leans mostly toward the camera and hardly reads; this bend shows the lean on screen.
STANCE_SIDE = 20.0
# In an idle the head may tip back this far against the chest (the general limit is `HEAD_BACK_LIMIT`), so the leaned stance keeps the face
# level toward the camera instead of looking at the floor and showing the hat's crown.
HEAD_BACK_IDLE = -24.0
# The sword stance's head against its chest (the idle's, at the bottom of its bob), for the attacks' stance keys.
STANCE_HEAD = -19.0
# The far arm (the right one in a clip; the game mirrors facing left) carried this much further forward and in (degrees) in the idle, walk
# and run, so its hand shows past the chest once the body stands side-on (round 2, the owner's pick D).
FAR_ARM_FWD, FAR_ARM_IN = 18.0, 8.0
# Idle loops are this long (frames): two bobs, one breath and one weight shift a loop.
IDLE_FRAMES = 96


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
            if BODY:
                # (Animation overhaul, stance pass: a low battle stance, not a stand. Both knees bent, the back one too, turned out so it
                # stays behind the front leg rather than crossing it; the hips sink about a tenth of the body's height. `plant` sets the
                # hips from the feet, so the bob and the weight shift are knee bends: both knees flex together on each bob, and over the
                # loop the weight rocks onto the front foot and back.)
                shift = math.sin(k * 2.0 * math.pi + 0.9)
                pose(rig, "hips", twist=-6.0 + 1.5 * breath)
                pose(rig, "thigh.L", fwd=STANCE_LEAD + (4.0 * down + 4.0 * shift) * STANCE_K, out=STANCE_WIDE)
                pose(rig, "shin.L", fwd=STANCE_LEAD_SHIN - (12.0 * down + 8.0 * shift) * STANCE_K)
                pose(rig, "foot.L", fwd=6.0, out=STANCE_FOOT_OUT)
                pose(rig, "thigh.R", fwd=STANCE_BACK + (3.0 * down - 3.0 * shift) * STANCE_K, out=STANCE_WIDE + STANCE_WIDE_BACK)
                pose(rig, "shin.R", fwd=STANCE_BACK_SHIN - (12.0 * down - 8.0 * shift) * STANCE_K)
                pose(rig, "foot.R", fwd=18.0 * STANCE_K, out=STANCE_FOOT_OUT)
            else:
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
            pose(rig, "spine", fwd=(STANCE_LEAN if BODY else 9.0) + 4.0 * down * flex - 3.0 * breath, twist=8.0 + 2.0 * b + 2.0 * breath,
                 out=STANCE_SIDE if BODY else 0.0)
            pose(rig, "head", fwd=(-19.0 if BODY else -7.0) + 4.0 * down - (2.5 * (0.5 - 0.5 * settle) if BODY else 0.0), twist=-4.0 - 1.5 * breath)
            # Lead fist (far arm) out in front, rear fist guarding the chin; both ride the bounce a beat behind.
            lag = math.cos(k * 4.0 * math.pi - 0.8)
            swing = 1.8 if BODY else 1.0
            # (The base body's arms are thick and fully jointed: its elbows bend less, the fists held a little lower.)
            elbow = 0.72 if BODY else 1.0
            # (After the owner's PEAK reference: the chunky arms held out clear of the body, so both read against the background, not
            # tucked into a guard in front of the belly where the body and the weapon hid them; fists still a little up and forward.)
            # (Forward more than out, 2026-10-14: held mostly out to the side, the rear arm pointed backward once the chest turns to the
            # camera, and its forearm folded down behind the body; the owner saw it bend behind him.)
            # (A guard, art review 5: the owner saw the unarmed fighters' arms extended as if going in for a hug, the elbows barely bent.
            # The upper arms now hang forward and a little out, the elbows bent hard so the forearms stand up, the fists in front of the
            # chest, the lead one a little further out and higher: ready to fight, both arms still clear of the body's outline.)
            if BODY:
                # (The elbows down and out to the side, so the upper arm shows from the camera: pointed forward it pointed at the camera
                # with the chest turned, foreshortened to nothing, and the fist looked stuck to the shoulder; the bend about a right angle,
                # not so tight that the elbow's volume squeezes. The owner: the bend read disjointed.)
                pose(rig, "armU.L", fwd=28.0 + 4.0 * lag * swing, out=30.0)
                pose(rig, "armL.L", fwd=104.0 - 6.0 * lag * swing)
                pose(rig, "hand.L", fwd=-16.0)
                # (Round 2, with the side-on turn: the far arm carried further forward and in, so its fist shows past the chest.)
                pose(rig, "armU.R", fwd=24.0 + FAR_ARM_FWD + 4.0 * lag * swing, out=28.0 - FAR_ARM_IN)
                pose(rig, "armL.R", fwd=110.0 - 6.0 * lag * swing)
                pose(rig, "hand.R", fwd=-18.0)
            else:
                pose(rig, "armU.L", fwd=40.0 + 5.0 * lag * swing, out=24.0)
                pose(rig, "armL.L", fwd=(64.0 - 6.0 * lag * swing) * elbow)
                pose(rig, "hand.L", fwd=-12.0)
                pose(rig, "armU.R", fwd=36.0 + 5.0 * lag * swing, out=22.0)
                pose(rig, "armL.R", fwd=(70.0 - 6.0 * lag * swing) * elbow)
                pose(rig, "hand.R", fwd=-14.0)
        return f

    clip(rig, "idle", IDLE_FRAMES, {i: idle(i / IDLE_FRAMES) for i in range(0, IDLE_FRAMES + 1, 4)})

    # Each unarmed family's own stance on the base body (the pose review: nine unarmed classes stood alike), the brawlers keeping the bouncy
    # guard above: the quick fighters low and coiled with the lead hand out, the heavy one wide and planted with its arms low and slow, the
    # caster upright with an open lead palm and the other hand at the hip, the tool users relaxed with their hands low. Arms are
    # (forward, out, elbow); the bob is a share of the guard's. Mapped by class in anim_library.gd (CLASS_FAMILY).
    IDLE_STYLES = {
        # (Stance pass: every family's knees bent, the back knee too, so each stands low and grounded; the styles differ in depth, width,
        # lean and the hands.)
        # (Round 2, with the side-on turn: each far arm (`arm_r`) carried further forward so its hand shows past the chest.)
        # (Round 2, judged at the match camera: the heavy, caster and tool fighters lean a step further toward the foe, and each head tips
        # back against its chest's lean (`HEAD_BACK_IDLE`) so the face stays level rather than looking at the floor.)
        # The brawlers (Claws, Gauntlets, Bracers): hunched forward over bent knees, both fists up in front of the chest, the lead one
        # further out (the reference claws fighter's crouch).
        "brawler": dict(hips=0.0, bob=0.55, lead=50.0, back=-24.0, shin_l=-84.0, shin_r=-54.0, spine=30.0, side=18.0, head=-16.0, wide=21.0,
                        arm_l=(52.0, 20.0, 96.0), arm_r=(58.0, 18.0, 100.0)),
        "quick": dict(hips=-0.2, bob=0.6, lead=62.0, back=-30.0, shin_l=-90.0, shin_r=-60.0, spine=28.0, side=14.0, head=-16.0, wide=18.0,
                      arm_l=(64.0, 10.0, 36.0), arm_r=(10.0, 18.0, 96.0)),
        "heavy": dict(hips=-0.14, bob=0.35, lead=34.0, back=-20.0, shin_l=-60.0, shin_r=-50.0, spine=18.0, side=12.0, head=-8.0, wide=30.0,
                      arm_l=(10.0, 50.0, 46.0), arm_r=(24.0, 40.0, 50.0)),
        "caster": dict(hips=0.0, bob=0.45, lead=28.0, back=-14.0, shin_l=-48.0, shin_r=-34.0, spine=8.0, side=6.0, head=-8.0, wide=17.0,
                       arm_l=(72.0, 10.0, 28.0), arm_r=(18.0, 24.0, 100.0)),
        "tool": dict(hips=0.0, bob=0.35, lead=22.0, back=-12.0, shin_l=-40.0, shin_r=-30.0, spine=6.0, side=5.0, head=-4.0, wide=16.0,
                     arm_l=(2.0, 12.0, 22.0), arm_r=(32.0, 10.0, 28.0)),
    }

    def styled_idle(k, st):
        def f(rig):
            b = math.cos(k * 4.0 * math.pi)
            down = 0.5 - 0.5 * b
            breath = math.sin(k * 2.0 * math.pi)
            # (Stance pass: the weight rocks onto the front foot and back once a loop, and both knees flex on the bob, since `plant` sets
            # the hips from the feet.)
            shift = math.sin(k * 2.0 * math.pi + 0.9)
            bob = st["bob"] * 2.0
            wide = st.get("wide", 15.0) * STANCE_W
            pose(rig, "hips", lift=st["hips"] - 0.03 * down * st["bob"], twist=-6.0 + 1.5 * breath)
            sk = STANCE_K
            pose(rig, "thigh.L", fwd=(st["lead"] + 4.0 * down * bob + 4.0 * shift) * sk, out=wide)
            pose(rig, "shin.L", fwd=(st["shin_l"] - 10.0 * down * bob - 8.0 * shift) * sk)
            pose(rig, "foot.L", fwd=6.0, out=STANCE_FOOT_OUT)
            pose(rig, "thigh.R", fwd=(st["back"] + 3.0 * down * bob - 3.0 * shift) * sk, out=wide + STANCE_WIDE_BACK)
            pose(rig, "shin.R", fwd=(st["shin_r"] - 10.0 * down * bob + 8.0 * shift) * sk)
            pose(rig, "foot.R", fwd=14.0 * sk, out=STANCE_FOOT_OUT)
            pose(rig, "spine", fwd=st["spine"] + 3.0 * down * bob - 3.0 * breath, twist=8.0 + 2.0 * breath, out=st.get("side", 0.0))
            pose(rig, "head", fwd=st["head"] + 3.0 * down, twist=-4.0 - 1.5 * breath)
            lag = math.cos(k * 4.0 * math.pi - 0.8)
            for side, arm in (("L", st["arm_l"]), ("R", st["arm_r"])):
                pose(rig, "armU." + side, fwd=arm[0] + 3.0 * lag - 2.0 * breath, out=arm[1])
                pose(rig, "armL." + side, fwd=arm[2] - 4.0 * lag)
        return f

    if BODY:
        for fam, st in IDLE_STYLES.items():
            clip(rig, "idle_" + fam, IDLE_FRAMES, {i: styled_idle(i / IDLE_FRAMES, st) for i in range(0, IDLE_FRAMES + 1, 4)})

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

    if BODY:
        # (Round 2: each family's own walk, its feet planted; `walk_style`.)
        for fam, st in WALK_STYLES.items():
            n = st["frames"]
            clip(rig, "walk_" + fam if fam else "walk", n, {i: walk_style(i / n, st) for i in range(0, n + 1)})
    else:
        clip(rig, "walk", 40, {i: walk(i / 40.0) for i in range(0, 41, 2)})
    # The run: a key every other frame so the snappy gait survives interpolation.
    if not BODY:
        clip(rig, "run", 20, {i: run_pose(i / 20.0) for i in range(0, 21)})
    # Each unarmed family's own run on the base body (with its idle, `IDLE_STYLES`): the quick fighters low and long with small arm
    # pumps, the heavy one upright and short-strided with big swings and a heavier bounce, the caster neat with its arms in, the tool
    # users a loose, bouncy jog.
    if BODY:
        # (Round 3: each family's run with its feet authored, `run_style`; the sword family's is the shared `run`, and the brawlers have
        # their own. The angle-keyed `run` above stays for the old blob rigs.)
        for fam, st in RUN_STYLES.items():
            n = st["frames"]
            clip(rig, "run_" + fam if fam else "run", n, {i: run_style(i / n, st) for i in range(0, n + 1)})
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
    # The base body's air arc (animation overhaul pass 2). The near leg is the right one and the near arm the left one once the game turns
    # the body and chest to the camera (see `jump`).
    def air_stretch(rig):
        # Off the floor: long and straight, toes pointed down, the far fist thrown up overhead and the near arm swept back.
        for side, fwd in (("L", -8.0), ("R", 10.0)):
            pose(rig, "thigh." + side, fwd=fwd, out=10.0)
            pose(rig, "shin." + side, fwd=-10.0)
            pose(rig, "foot." + side, fwd=-34.0)
        pose(rig, "armU.R", fwd=150.0, out=22.0)
        pose(rig, "armL.R", fwd=24.0)
        pose(rig, "armU.L", fwd=-40.0, out=34.0)
        pose(rig, "armL.L", fwd=30.0)
        pose(rig, "spine", fwd=-4.0, twist=4.0)
        pose(rig, "head", fwd=-4.0)

    def air_tuck(rig):
        # The top of the jump: both knees pulled up to the chest, the fists drawn in front of it, the back rounded.
        pose(rig, "thigh.R", fwd=82.0, out=12.0)
        pose(rig, "shin.R", fwd=-112.0)
        pose(rig, "foot.R", fwd=18.0)
        pose(rig, "thigh.L", fwd=64.0, out=16.0)
        pose(rig, "shin.L", fwd=-104.0)
        pose(rig, "foot.L", fwd=10.0)
        pose(rig, "armU.R", fwd=56.0, out=26.0)
        pose(rig, "armL.R", fwd=96.0)
        # (Round 2, side-on: the near fist drawn in at the chest, not up at the chin, where it covered the mouth.)
        pose(rig, "armU.L", fwd=22.0, out=20.0)
        pose(rig, "armL.L", fwd=80.0)
        pose(rig, "spine", fwd=18.0, twist=6.0)
        pose(rig, "head", fwd=-4.0)

    def air_open(rig):
        # Opening out on the way down: the legs come apart and reach toward the floor (the near knee still soft in front, the far leg
        # behind), the arms open out to the sides for balance.
        pose(rig, "thigh.R", fwd=40.0, out=10.0)
        pose(rig, "shin.R", fwd=-58.0)
        pose(rig, "foot.R", fwd=8.0)
        pose(rig, "thigh.L", fwd=-16.0, out=16.0)
        pose(rig, "shin.L", fwd=-30.0)
        pose(rig, "foot.L", fwd=-18.0)
        # (Round 2, with the side-on turn: "out to the sides" now points the near arm at the camera, its fist over the face (every
        # unarmed class, every fall). The near arm (L) opens forward and low along the stage, the far one back and up behind the body.)
        pose(rig, "armU.L", fwd=30.0, out=22.0)
        pose(rig, "armL.L", fwd=30.0)
        pose(rig, "armU.R", fwd=-30.0, out=50.0)
        pose(rig, "armL.R", fwd=30.0)
        pose(rig, "spine", fwd=6.0, twist=4.0)
        pose(rig, "head", fwd=-2.0)

    def air_fast(k):
        # Falling at full speed: legs long and apart, reaching down, the arms raised high out to the sides and paddling a little.
        def f(rig):
            w = math.sin(k * 2 * math.pi)
            pose(rig, "thigh.R", fwd=22.0 + 5.0 * w, out=10.0)
            pose(rig, "shin.R", fwd=-28.0 - 6.0 * w)
            pose(rig, "foot.R", fwd=-6.0)
            pose(rig, "thigh.L", fwd=-18.0 - 5.0 * w, out=16.0)
            pose(rig, "shin.L", fwd=-16.0 + 5.0 * w)
            pose(rig, "foot.L", fwd=-24.0)
            # (Round 2, side-on: the near arm reaches forward and a little up along the stage, the far one raised high behind the head,
            # both paddling; raised out to the sides, the near fist sat over the face.)
            pose(rig, "armU.L", fwd=34.0 + 8.0 * w, out=24.0)
            pose(rig, "armL.L", fwd=30.0)
            pose(rig, "armU.R", fwd=130.0 - 8.0 * w, out=30.0)
            pose(rig, "armL.R", fwd=34.0)
            pose(rig, "spine", fwd=-2.0, twist=4.0 * w)
            pose(rig, "head", fwd=-4.0)
        return f

    def air_between(a, b):
        # Half way from pose `a` to pose `b` (a key between two, so the knee-up eases into the tuck rather than sweeping straight across).
        def f(rig):
            a(rig)
            first = {pb.name: tuple(pb.rotation_euler) for pb in rig.pose.bones}
            reset(rig)
            b(rig)
            for pb in rig.pose.bones:
                r = first[pb.name]
                pb.rotation_euler = tuple(0.5 * (x + y) for x, y in zip(r, pb.rotation_euler))
        return f

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
        # (Animation overhaul, the air arc: the game plays the rise by how fast the fighter still climbs, not by time, so this clip runs
        # from the take-off (0) to the top of the jump (30): stretched long off the floor, the near knee driven up, then both knees pulled
        # in to a tuck at the top. `fall_start` opens out of the tuck on the way down. It was one pose, the knee up, eased a little.)
        clip(rig, "jump", 30, {0: amped(air_stretch, 1.05), 9: amped(jump(1.0), 1.08), 19: air_between(jump(1.0), air_tuck),
                               30: air_tuck})
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

    if BODY:
        # The way down (the air arc): played by how fast the fighter falls, from the tuck at the top (0), the legs opening and reaching
        # for the floor as it speeds up (12), to the long open pose at full speed (30), where `fall` takes over and paddles.
        clip(rig, "fall_start", 30, {0: air_tuck, 12: air_open, 30: air_fast(0.0)})
        clip(rig, "fall", 30, {0: air_fast(0.0), 8: air_fast(0.25), 15: air_fast(0.5), 23: air_fast(0.75), 30: air_fast(1.0)})
    else:
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

    # (Round 2: the crouch starts from the battle stance's legs; it started standing 0.17 taller than the idle, so a jump rose before it sank.)
    clip(rig, "jumpsquat", 4, {0: _stance_over(jumpsquat(0.3), {}, 0.0, True), 4: jumpsquat(1.0)})

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

    # (Round 2: it settles into the battle stance's legs; it stood 0.17 taller than the idle at its end, then sank into the idle.)
    clip(rig, "land", 14, {0: land(1.0), 3: amped(land(1.0), 1.06), 8: land(0.45, rebound=1.0), 14: _stance_over(land(0.3), {}, 0.0, True)})

    def shield(rig):
        pose(rig, "hips", lift=-0.08)
        for side in "LR":
            pose(rig, "thigh." + side, fwd=26.0, out=8.0)
            pose(rig, "shin." + side, fwd=-40.0)
            # (Round 2, side-on: braced with the fists in at the chest; held at 62 the forearms stood in front of the face.)
            pose(rig, "armU." + side, fwd=(16.0 if side == "L" else 30.0), out=16.0)
            pose(rig, "armL." + side, fwd=92.0)
        pose(rig, "spine", fwd=12.0)

    # (Round 2: on the battle stance's legs; it stood up 0.23 out of the idle to shield and sank back on dropping it.)
    clip(rig, "shield", 8, {0: _stance_over(shield, {}, 0.0, True), 8: _stance_over(shield, {}, 0.0, True)})

    def hurt(rig):
        for side in "LR":
            pose(rig, "armU." + side, fwd=-20.0, out=78.0)
            pose(rig, "armL." + side, fwd=-10.0)
            pose(rig, "thigh." + side, fwd=-10.0, out=14.0)
            pose(rig, "shin." + side, fwd=-26.0)
        pose(rig, "spine", fwd=-18.0, twist=-8.0)
        pose(rig, "head", fwd=-14.0)

    # The base body's hit reaction: on the hit (held through the hitlag freeze) the chest is thrown back and twisted off the blow, the head
    # snapped back, the arms trailing, the knees giving; then over the hitstun it slackens into a looser reel, so a long stun does not
    # read as one frozen pose. The game's flinch, tumble and launch poses
    # go on top for stronger hits.
    def hurt_snap(rig):
        # (Pose review 2026-10-17: a recoil, not arms thrown out wide. The chest and head snap back off the blow, both arms trail loosely
        # down and behind the body it leaves them with, and the knees give, the near one buckling.)
        pose(rig, "armU.L", fwd=-30.0, out=30.0)
        pose(rig, "armL.L", fwd=34.0)
        pose(rig, "armU.R", fwd=-22.0, out=26.0)
        pose(rig, "armL.R", fwd=40.0)
        pose(rig, "thigh.R", fwd=26.0, out=10.0)
        pose(rig, "shin.R", fwd=-56.0)
        pose(rig, "thigh.L", fwd=-4.0, out=12.0)
        pose(rig, "shin.L", fwd=-34.0)
        # (Not so far back or so twisted that the face turns away: the hurt face is half the read.)
        pose(rig, "spine", fwd=-22.0, twist=-6.0)
        pose(rig, "head", fwd=-6.0, twist=4.0)

    def hurt_reel(rig):
        # Slackening over the hitstun: the chest coming forward again, the arms hanging, the knees still soft.
        pose(rig, "armU.L", fwd=-8.0, out=22.0)
        pose(rig, "armL.L", fwd=40.0)
        pose(rig, "armU.R", fwd=-4.0, out=20.0)
        pose(rig, "armL.R", fwd=46.0)
        pose(rig, "thigh.R", fwd=16.0, out=10.0)
        pose(rig, "shin.R", fwd=-40.0)
        pose(rig, "thigh.L", fwd=-2.0, out=12.0)
        pose(rig, "shin.L", fwd=-28.0)
        pose(rig, "spine", fwd=-8.0, twist=-2.0)
        pose(rig, "head", fwd=0.0, twist=2.0)

    # (Animation overhaul pass 8, reactions: a long hitstun no longer freezes on the reel. After it the body keeps reeling: the arms swing
    # loosely in turn and the knees give and recover, slowing, so a stun of any length stays alive.)
    def hurt_flail(k):
        def f(rig):
            hurt_reel(rig)
            w = math.sin(k * 2.0 * math.pi)
            pose(rig, "armU.L", fwd=-8.0 + 22.0 * w, out=26.0 + 8.0 * w)
            pose(rig, "armL.L", fwd=40.0 + 14.0 * w)
            pose(rig, "armU.R", fwd=-4.0 - 22.0 * w, out=24.0 - 8.0 * w)
            pose(rig, "armL.R", fwd=46.0 - 14.0 * w)
            pose(rig, "thigh.R", fwd=16.0 + 10.0 * w, out=10.0)
            pose(rig, "shin.R", fwd=-40.0 - 14.0 * w)
            pose(rig, "spine", fwd=-8.0 - 5.0 * w, twist=-2.0 + 4.0 * w)
        return f

    # The tumble (a launch slowing down, spinning head over heels): curled round into a ball rather than spun stiff, the arms and legs
    # flailing in turn, so the spin reads as a tumble, not a statue rotating.
    def tumble_pose(k):
        def f(rig):
            w = math.sin(k * 2.0 * math.pi)
            v = math.cos(k * 2.0 * math.pi)
            pose(rig, "spine", fwd=22.0 + 6.0 * w)
            pose(rig, "head", fwd=-4.0)
            pose(rig, "thigh.R", fwd=62.0 + 22.0 * w, out=14.0)
            pose(rig, "shin.R", fwd=-84.0 - 18.0 * v)
            pose(rig, "thigh.L", fwd=44.0 - 22.0 * w, out=18.0)
            pose(rig, "shin.L", fwd=-70.0 + 18.0 * v)
            pose(rig, "armU.L", fwd=40.0 + 40.0 * v, out=46.0)
            pose(rig, "armL.L", fwd=50.0)
            pose(rig, "armU.R", fwd=30.0 - 40.0 * v, out=50.0)
            pose(rig, "armL.R", fwd=56.0)
        return f

    if BODY:
        clip(rig, "hurt", 66, {0: hurt_snap, 4: amped(hurt_snap, 1.05), 18: hurt_reel, 30: hurt_flail(0.25), 42: hurt_flail(0.75),
                               54: hurt_flail(0.25), 66: hurt_reel})
        clip(rig, "tumble", 32, {i: tumble_pose(i / 32.0) for i in range(0, 33, 4)})
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

    clip(rig, "attack_kick", 60, {0: _stance_over(kick(2), {}, 0.0, True), 21: amped(kick(0), 1.12), 33: kick(1), 38: amped(kick(1), 1.1), 45: kick(1),
                                  60: _stance_over(kick(2), {}, 0.0, True)})

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
        # (Round 2: the rest and recovery keys stand on the battle stance, `_stance_over`, as the six-pose clips' do; standing tall, these
        # moves rose about 0.3 out of the idle on their first frame and sank back at their end.)
        clip(rig, name, 60, {0: _stance_over(hold(0), {}, 0.0), 21: amped(wind, CHAMBER_AMP), 33: strike, 38: amped(strike, 1.12),
                             45: _stance_over(hold(1), {}, 1.0), 60: _stance_over(hold(0), {}, 0.0)})

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
    # The Dagger's diving forward smash (posed in BODY_MOVES).
    clip(rig, "attack_knife_smash", 60, {})
    # The rusher's charged punch (posed in BODY_MOVES).
    clip(rig, "gaunt_punch", 60, {})
    # The Hauler's giant punch charge (posed in BODY_MOVES; round 2).
    clip(rig, "gaunt_charge", 60, {})
    # The last stand-in moves' own clips (posed in BODY_MOVES; round 2).
    for nm in ("slap_ground", "leap_grab", "latch_sit", "rocket_fly"):
        clip(rig, nm, 60, {})
    # The bounder's drop (posed in BODY_MOVES).
    clip(rig, "body_drop", 60, {})

    # Spinning with both arms out (pass 13: the all-rounder's down air, the Hauler's spins; the game turns the whole body, fighter_view.gd
    # `SPIN_MOVES`, and leaves both arms to the clip, AnimLibrary.OWN_ARMS): the arms held out level, elbows soft, the body
    # upright; on the ground the feet set apart, in the air the knees drawn up a little. The arms lift as the spin builds and settle.
    def spin_pose(air, k):
        def f(rig):
            # (Spread along the line of the fight, one arm ahead and one behind: held out to the sides of the turned body, the near arm
            # pointed at the camera and its fist sat in front of the face.)
            for side, sign in (("L", 1.0), ("R", -1.0)):
                pose(rig, "armU." + side, fwd=sign * (50.0 + 34.0 * k), out=22.0)
                pose(rig, "armL." + side, fwd=24.0 - 10.0 * k)
                pose(rig, "hand." + side, fwd=0.0)
            pose(rig, "spine", fwd=4.0 - 4.0 * k)
            pose(rig, "head", fwd=2.0)
            if air:
                stance(rig, 30.0 + 14.0 * k, 22.0 + 10.0 * k)
                pose(rig, "shin.L", fwd=-60.0 - 10.0 * k)
                pose(rig, "shin.R", fwd=-56.0 - 10.0 * k)
            else:
                pose(rig, "hips", lift=-0.06 * k)
                stance(rig, 14.0, -12.0)
                pose(rig, "thigh.L", fwd=14.0, out=14.0)
                pose(rig, "thigh.R", fwd=-12.0, out=14.0)
        return f
    for air, nm in ((False, "spin_arms"), (True, "spin_arms_air")):
        clip(rig, nm, 60, {0: spin_pose(air, 0.0), 21: spin_pose(air, 0.8), 33: spin_pose(air, 1.0), 45: spin_pose(air, 1.0), 60: spin_pose(air, 0.0)})
    # The Settler's tool clips (posed in BODY_MOVES).
    for tool in ("tool_reach", "tool_toss", "tool_dig", "tool_pour", "tool_chop", "tool_heave", "tool_raise", "tool_trip", "tool_shrug", "tool_ride", "tool_float"):
        clip(rig, tool, 60, {})
    # (Round 8: the Needles' own specials, posed in BODY_MOVES.)
    if BODY:
        for nm in ("needle_volley", "line_cast", "vanish", "sword_counter", "reflector", "buckler_bow", "buckler_throw", "buckler_bomb",
                   "staff_point", "staff_flame", "staff_guard", "great_wave", "great_rise", "nodachi_flare", "nodachi_orbs",
                   "nodachi_barrier"):
            clip(rig, nm, 60, {})
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
        spec = _human_arms(spec)

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
    # Up tilt: the blade sweeps overhead from front to back; the body stands tall under it.
    move_of("sword_utilt",
            {"hips": -0.06, "spine": (16.0, 8.0), "head": (8.0, 0.0), "stance": (14.0, -10.0)},
            {"hips": 0.02, "spine": (4.0, -6.0), "head": (-4.0, 0.0), "stance": (16.0, -18.0), "arms": (-20.0, 24.0)},
            {"spine": (2.0, 0.0), "stance": (8.0, -8.0)})
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
    # Up smash: crouch, then rise onto planted feet as the blade goes straight up overhead (the arm and blade reach, not the body: the
    # owner, 2026-10-17).
    move_of("sword_usmash",
            {"hips": -0.16, "spine": (18.0, 0.0), "head": (8.0, 0.0), "stance": (34.0, 26.0), "shin.L": (-60.0,), "shin.R": (-60.0,)},
            {"hips": -0.04, "spine": (4.0, 0.0), "head": (-4.0, 0.0), "stance": (14.0, -12.0), "arms": (-24.0, 30.0)},
            {"spine": (2.0, 0.0), "stance": (6.0, 4.0)})
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
    # long beneath the blade as it is driven straight up overhead, the legs hanging together, the face kept to the camera.
    move_of("sword_rise",
            {"spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (60.0, 54.0), "shin.L": (-100.0,), "shin.R": (-96.0,), "arms": (20.0, 20.0)},
            {"spine": (2.0, 0.0), "head": (-4.0, 0.0), "stance": (-6.0, 4.0), "shin.L": (-14.0,), "shin.R": (-20.0,), "arms": (-24.0, 30.0)},
            {"spine": (2.0, 0.0), "stance": (16.0, 20.0), "shin.L": (-40.0,), "shin.R": (-46.0,)})
    # The claws fighter's neutral air, after its reference: a held double kick, the body leaned back a little (not lying on its back: the
    # pose review, head below the hips) and both legs shot out straight
    # ahead together, the arms flung back for balance; the wind-up curls both knees up to the chest.
    move_of("kick_double",
            {"spine": (16.0, 0.0), "head": (6.0, 0.0), "stance": (70.0, 66.0), "shin.L": (-116.0,), "shin.R": (-110.0,), "arms": (30.0, 22.0)},
            {"spine": (-12.0, 0.0), "head": (6.0, 0.0), "stance": (96.0, 80.0), "shin.L": (-4.0,), "shin.R": (-16.0,),
             "foot.L": (-30.0,), "foot.R": (-24.0,), "arms": (-40.0, 64.0)},
            {"spine": (-6.0, 0.0), "stance": (76.0, 66.0), "shin.L": (-30.0,), "shin.R": (-40.0,), "arms": (-16.0, 50.0)})
    # Back air: lean forward and drive the heel back (not so far forward, nor twisted, that the big head shows its crown).
    move_of("kick_bair",
            {"spine": (24.0, 10.0), "head": (10.0, 20.0), "stance": (60.0, 50.0), "shin.L": (-90.0,), "shin.R": (-90.0,), "arms": (40.0, 20.0)},
            {"spine": (14.0, -10.0), "head": (-6.0, 10.0), "stance": (50.0, -20.0), "shin.L": (-80.0,), "arms": (60.0, 30.0)},
            {"spine": (10.0, 0.0), "stance": (30.0, 0.0)})
    # Up air: a flip kick, leaning back a little as the leg goes up (the game adds the counter-lean; the head stays level).
    move_of("kick_uair",
            {"spine": (24.0, 0.0), "stance": (60.0, 60.0), "shin.L": (-100.0,), "shin.R": (-100.0,), "arms": (20.0, 20.0)},
            {"spine": (-6.0, 0.0), "head": (-2.0, 0.0), "stance": (10.0, 30.0), "arms": (-30.0, 50.0)},
            {"spine": (-2.0, 0.0), "stance": (20.0, 20.0)})
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
    # A side kick (animation overhaul pass 11, after the references' side kicks: the quick fighter's second forward smash kick, the
    # all-rounder's and the rusher's forward tilts): the kicking knee chambered high and tucked with the body turned away, then the
    # leg driven straight out level (the game aims the foot at the hit) as the body leans right back away from it over the bent standing
    # leg, the arms flung back and out for balance, so the whole silhouette is one long line from the head to the foot. (Was the stretch
    # kick straight up, `kick_up`, upright, which read as a toe poke at waist height.)
    move_of("kick_side",
            {"hips": -0.08, "spine": (14.0, -22.0), "head": (2.0, 14.0), "stance": (74.0, -12.0), "shin.L": (-118.0,), "shin.R": (-34.0,),
             "arms": (30.0, 24.0)},
            {"hips": -0.06, "spine": (-30.0, 22.0), "head": (20.0, -10.0), "stance": (84.0, -8.0), "shin.L": (-4.0,), "shin.R": (-22.0,),
             "arms": (-22.0, 56.0)},
            {"spine": (-8.0, 6.0), "stance": (20.0, -8.0), "shin.R": (-20.0,)})
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
    # The dash grab (the playtest notes, 2026-10-19: it played the standing grab, so it did not read): a lunge out of the run, the hips
    # dropped and the body pitched forward over a long front step, the back leg stretched out behind, both arms thrown out low to catch.
    move_of("body_dash_grab",
            {"hips": -0.1, "spine": (24.0, 0.0), "head": (-10.0, 0.0), "stance": (40.0, -30.0), "shin.L": (-30.0,), "arms": (48.0, 14.0)},
            {"hips": -0.16, "spine": (34.0, 0.0), "head": (-16.0, 0.0), "stance": (62.0, -44.0), "shin.L": (-46.0,), "shin.R": (-6.0,),
             "arms": (84.0, 8.0), "armL.L": (8.0,), "armL.R": (8.0,)},
            {"hips": -0.1, "spine": (22.0, 0.0), "stance": (42.0, -30.0), "shin.L": (-30.0,), "arms": (70.0, 12.0)})
    # The pivot grab (the same notes: it showed no turn): the game spins the body round from the way it was running (fighter_view.gd
    # `PIVOT_SPIN`); the clip winds the torso against the spin and whips it round into the reach, the arms swinging out wide and in.
    move_of("body_pivot_grab",
            {"hips": -0.06, "spine": (8.0, 36.0), "head": (0.0, 24.0), "stance": (22.0, -20.0), "arms": (30.0, 44.0)},
            {"hips": -0.12, "spine": (24.0, -12.0), "head": (-10.0, -6.0), "stance": (40.0, -30.0), "shin.L": (-30.0,),
             "arms": (84.0, 10.0), "armL.L": (12.0,), "armL.R": (12.0,)},
            {"hips": -0.08, "spine": (16.0, -4.0), "stance": (30.0, -22.0), "arms": (68.0, 12.0)})
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

    # (Round 2: the stance's legs, as the moves' rest keys.)
    clip(rig, "grab", 8, {0: _stance_over(grab_pose, {}, 0.0, True), 8: _stance_over(grab_pose, {}, 0.0, True)})

    def throw_wind(rig):
        grab_pose(rig)
        pose(rig, "spine", fwd=-4.0, twist=-26.0)

    def throw_strike(rig):
        grab_pose(rig)
        pose(rig, "spine", fwd=22.0, twist=30.0)
        pose(rig, "head", fwd=-12.0)

    clip(rig, "throw", 60, {0: _stance_over(grab_pose, {}, 0.0, True), 21: amped(throw_wind, 1.12), 33: throw_strike, 38: amped(throw_strike, 1.1), 45: throw_strike,
                             60: _stance_over(grab_pose, {}, 0.0, True)})

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

        clip(rig, "spot_dodge", 30, {0: _stance_over(duck(0.2), {}, 0.0, True), 5: amped(duck(1.0), 1.05), 22: duck(1.0), 30: _stance_over(duck(0.2), {}, 0.0, True)})

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
        # (Pose review 2026-10-17: the arms trail bent and close, not thrown straight out to the sides.)
        def heavy_fold(rig):
            pose(rig, "spine", fwd=30.0, twist=-4.0)
            pose(rig, "head", fwd=-8.0, twist=4.0)
            pose(rig, "armU.L", fwd=70.0, out=22.0)
            pose(rig, "armL.L", fwd=40.0)
            pose(rig, "armU.R", fwd=50.0, out=28.0)
            pose(rig, "armL.R", fwd=46.0)
            pose(rig, "thigh.L", fwd=52.0, out=14.0)
            pose(rig, "shin.L", fwd=-30.0)
            pose(rig, "thigh.R", fwd=30.0, out=16.0)
            pose(rig, "shin.R", fwd=-56.0)

        def heavy_drift(rig):
            pose(rig, "spine", fwd=18.0, twist=-2.0)
            pose(rig, "head", fwd=-4.0, twist=2.0)
            pose(rig, "armU.L", fwd=50.0, out=24.0)
            pose(rig, "armL.L", fwd=40.0)
            pose(rig, "armU.R", fwd=30.0, out=30.0)
            pose(rig, "armL.R", fwd=44.0)
            pose(rig, "thigh.L", fwd=36.0, out=14.0)
            pose(rig, "shin.L", fwd=-44.0)
            pose(rig, "thigh.R", fwd=14.0, out=16.0)
            pose(rig, "shin.R", fwd=-60.0)

        # (Pass 8: past the drift the limbs keep trailing and swinging a little, rather than holding still through a long flight.)
        def heavy_sway(k):
            def f(rig):
                heavy_drift(rig)
                w = math.sin(k * 2.0 * math.pi)
                pose(rig, "armU.L", fwd=50.0 + 16.0 * w, out=24.0)
                pose(rig, "armU.R", fwd=30.0 - 16.0 * w, out=30.0)
                pose(rig, "thigh.L", fwd=36.0 - 10.0 * w, out=14.0)
                pose(rig, "thigh.R", fwd=14.0 + 10.0 * w, out=16.0)
            return f

        clip(rig, "hurt_heavy", 60, {0: heavy_fold, 4: amped(heavy_fold, 1.06), 24: heavy_drift, 36: heavy_sway(0.25), 48: heavy_sway(0.75),
                                     60: heavy_drift})

        # The same, sent upward: the chest arched up off the blow, the arms and legs flung down and out below the body, the face up.
        def up_arch(rig):
            pose(rig, "spine", fwd=-26.0)
            pose(rig, "head", fwd=4.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=-24.0, out=24.0)
                pose(rig, "armL." + side, fwd=30.0)
                pose(rig, "thigh." + side, fwd=-18.0, out=18.0)
                pose(rig, "shin." + side, fwd=-38.0)
                pose(rig, "foot." + side, fwd=-20.0)

        def up_drift(rig):
            pose(rig, "spine", fwd=-14.0)
            pose(rig, "head", fwd=2.0)
            for side in "LR":
                pose(rig, "armU." + side, fwd=4.0, out=28.0)
                pose(rig, "armL." + side, fwd=40.0)
                pose(rig, "thigh." + side, fwd=6.0, out=16.0)
                pose(rig, "shin." + side, fwd=-46.0)

        def up_sway(k):
            def f(rig):
                up_drift(rig)
                w = math.sin(k * 2.0 * math.pi)
                for side, sign in (("L", 1.0), ("R", -1.0)):
                    pose(rig, "armU." + side, fwd=4.0 + 18.0 * w * sign, out=28.0)
                    pose(rig, "thigh." + side, fwd=6.0 - 12.0 * w * sign, out=16.0)
            return f

        clip(rig, "hurt_up", 60, {0: up_arch, 4: amped(up_arch, 1.06), 24: up_drift, 36: up_sway(0.25), 48: up_sway(0.75), 60: up_drift})

        # A rare fidget after a long stillness (the game plays it now and then in a long idle): from the stance, a glance off toward the
        # camera side and back, the weight shuffled from foot to foot, a roll of the shoulders, back into the stance.
        # (Animation overhaul step 9: on each family's own stance, `idle_look_<family>`; the one fidget was on the sword stance, so a
        # brawler's or a quick fighter's fidget popped from its own crouch into the sword guard and back. The sword stance's lean and side
        # bend are kept through the glance too: it straightened the chest to 8 degrees and dropped the side bend.)
        def glance_on(base, spine_fwd, side, head_fwd, hips):
            def make(turn, shift):
                def f(rig):
                    base(rig)
                    pose(rig, "hips", lift=hips, twist=-6.0 + 4.0 * shift)
                    pose(rig, "spine", fwd=spine_fwd, twist=8.0 + 10.0 * turn, out=side)
                    pose(rig, "head", fwd=head_fwd, twist=-4.0 + 26.0 * turn)
                return f
            return make

        def look_clip(name, base, glance):
            clip(rig, name, 72, {0: base, 12: glance(1.0, 0.5), 30: glance(1.0, 1.0), 42: glance(-0.6, 0.0), 60: glance(-0.6, -0.5),
                                 72: base})

        look_clip("idle_look", idle(0.0), glance_on(idle(0.0), STANCE_LEAN, STANCE_SIDE, -16.0, -0.06))
        for fam, st in IDLE_STYLES.items():
            look_clip("idle_look_" + fam, styled_idle(0.0, st),
                      glance_on(styled_idle(0.0, st), st["spine"], st.get("side", 0.0), st["head"], st["hips"]))
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

    if BODY:
        # ---- Round 8: the states that played another state's clip (docs/art/anim/ANIMATION_REVIEW_1.md, the missing sets) ----
        # Special fall (`Helpless`, after an up special): spent and limp, as the reference fighters fall after a recovery move: the body
        # a little hunched, both arms raised loosely (the near one forward along the stage, the far one up behind the head) and slowly
        # flailing, the legs hanging apart with soft knees and the toes down. A loop; it played the jump's arc before, which reads as a
        # fighter still in control.
        def helpless(k):
            def f(rig):
                w = math.sin(k * 2.0 * math.pi)
                v = math.sin(k * 2.0 * math.pi + 1.3)
                pose(rig, "spine", fwd=14.0 + 3.0 * v, twist=5.0 * w)
                pose(rig, "head", fwd=-10.0 + 3.0 * v)
                # (Raised well up so they read at the match camera: at 78 the near fist sat at the chest.)
                pose(rig, "armU.L", fwd=118.0 + 16.0 * w, out=14.0)
                pose(rig, "armL.L", fwd=30.0 + 12.0 * v)
                pose(rig, "hand.L", fwd=10.0)
                pose(rig, "armU.R", fwd=164.0 - 16.0 * w, out=24.0)
                pose(rig, "armL.R", fwd=24.0 - 10.0 * v)
                pose(rig, "hand.R", fwd=10.0)
                pose(rig, "thigh.L", fwd=24.0 + 6.0 * v, out=8.0)
                pose(rig, "shin.L", fwd=-36.0 - 6.0 * v)
                pose(rig, "foot.L", fwd=-30.0)
                pose(rig, "thigh.R", fwd=-10.0 - 6.0 * v, out=12.0)
                pose(rig, "shin.R", fwd=-28.0 + 6.0 * v)
                pose(rig, "foot.R", fwd=-34.0)
            return f

        clip(rig, "helpless", 48, {i: helpless(i / 48.0) for i in range(0, 49, 6)})

        # Picking an item up (`ItemPickup`, 8 frames, played by progress): a dip onto bent knees, the near hand reaching down in front to
        # the floor and coming back up with it. It stood in the idle.
        def item_reach(rig):
            pose(rig, "hips", lift=-0.16)
            pose(rig, "thigh.L", fwd=66.0, out=10.0)
            pose(rig, "shin.L", fwd=-104.0)
            pose(rig, "foot.L", fwd=30.0)
            pose(rig, "thigh.R", fwd=-6.0, out=14.0)
            pose(rig, "shin.R", fwd=-70.0)
            pose(rig, "foot.R", fwd=26.0)
            pose(rig, "spine", fwd=38.0, out=STANCE_SIDE * 0.5)
            pose(rig, "head", fwd=-22.0)
            pose(rig, "armU.L", fwd=58.0, out=12.0)
            pose(rig, "armL.L", fwd=12.0)
            pose(rig, "armU.R", fwd=20.0, out=26.0)
            pose(rig, "armL.R", fwd=70.0)

        clip(rig, "item_pick", 12, {0: idle(0.0), 5: item_reach, 7: item_reach, 12: idle(0.0)})

        # Throwing an item (`ItemThrow`, 19-29 frames, played by progress): the far hand (the near one holds an armed class's weapon, as the
        # reference's swordsman throws with his free hand) drawn back and up behind the shoulder over a step back, then whipped through
        # forward over the front foot as the chest turns into it, and back to the stance. It stood in the idle (on the ground) or fell (in
        # the air), the item leaving an empty hand.
        def item_wind(rig):
            pose(rig, "hips", twist=-14.0)
            pose(rig, "thigh.L", fwd=STANCE_LEAD * 0.8, out=STANCE_WIDE)
            pose(rig, "shin.L", fwd=STANCE_LEAD_SHIN * 0.8)
            pose(rig, "thigh.R", fwd=STANCE_BACK * 1.2, out=STANCE_WIDE + 4.0)
            pose(rig, "shin.R", fwd=STANCE_BACK_SHIN * 1.1)
            pose(rig, "spine", fwd=4.0, twist=-26.0, out=-6.0)
            pose(rig, "head", fwd=-8.0, twist=18.0)
            pose(rig, "armU.R", fwd=168.0, out=14.0)
            pose(rig, "armL.R", fwd=70.0)
            pose(rig, "hand.R", fwd=-20.0)
            pose(rig, "armU.L", fwd=40.0, out=30.0)
            pose(rig, "armL.L", fwd=80.0)

        def item_release(rig):
            pose(rig, "hips", twist=10.0)
            pose(rig, "thigh.L", fwd=STANCE_LEAD * 1.1, out=STANCE_WIDE)
            pose(rig, "shin.L", fwd=STANCE_LEAD_SHIN * 1.05)
            pose(rig, "thigh.R", fwd=STANCE_BACK * 1.4, out=STANCE_WIDE + 4.0)
            pose(rig, "shin.R", fwd=STANCE_BACK_SHIN * 0.8)
            pose(rig, "spine", fwd=30.0, twist=20.0, out=STANCE_SIDE)
            pose(rig, "head", fwd=-18.0, twist=-10.0)
            pose(rig, "armU.R", fwd=78.0, out=6.0)
            pose(rig, "armL.R", fwd=8.0)
            pose(rig, "hand.R", fwd=14.0)
            pose(rig, "armU.L", fwd=10.0, out=34.0)
            pose(rig, "armL.L", fwd=60.0)

        clip(rig, "item_throw", 24, {0: idle(0.0), 7: item_wind, 9: amped(item_wind, 1.06), 12: item_release, 15: item_release,
                                     24: idle(0.0)})

        # Shield break, once down on the ground (`ShieldBreak`): dazed, swaying in a slow circle over soft knees, the arms hanging loose
        # and swinging a beat behind, the head lolling with the sway. A loop; it played the hit reaction's flail, held.
        def dizzy(k):
            def f(rig):
                a = k * 2.0 * math.pi
                sx, sy = math.sin(a), math.cos(a)
                pose(rig, "hips", lift=-0.06, twist=6.0 * sx)
                pose(rig, "thigh.L", fwd=26.0 + 6.0 * sy, out=12.0)
                pose(rig, "shin.L", fwd=-44.0 - 8.0 * sy)
                pose(rig, "foot.L", fwd=12.0)
                pose(rig, "thigh.R", fwd=-6.0 - 6.0 * sy, out=16.0)
                pose(rig, "shin.R", fwd=-30.0 + 8.0 * sy)
                pose(rig, "foot.R", fwd=10.0)
                pose(rig, "spine", fwd=16.0 + 6.0 * sy, out=9.0 * sx, twist=6.0 * sx)
                pose(rig, "head", fwd=-6.0 + 5.0 * math.cos(a - 0.8), out=7.0 * math.sin(a - 0.8))
                lag = math.sin(a - 1.0)
                pose(rig, "armU.L", fwd=10.0 + 10.0 * lag, out=12.0)
                pose(rig, "armL.L", fwd=24.0)
                pose(rig, "armU.R", fwd=6.0 - 10.0 * lag, out=16.0)
                pose(rig, "armL.R", fwd=20.0)
            return f

        clip(rig, "dizzy", 80, {i: dizzy(i / 80.0) for i in range(0, 81, 10)})

        # Buried (`Buried`, the body sunk to the waist by the game): both arms raised and pushing at the ground round it in turn, the
        # chest straining side to side. A loop; it stood in the idle under the ground.
        def buried(k):
            def f(rig):
                a = k * 2.0 * math.pi
                pose(rig, "spine", fwd=6.0, out=8.0 * math.sin(a), twist=10.0 * math.sin(a))
                pose(rig, "head", fwd=-10.0)
                # (Raised high: sunk to the waist, arms held lower stayed under the ground.)
                pose(rig, "armU.L", fwd=150.0 + 18.0 * math.sin(a), out=34.0)
                pose(rig, "armL.L", fwd=40.0 - 24.0 * math.sin(a))
                pose(rig, "armU.R", fwd=150.0 - 18.0 * math.sin(a), out=34.0)
                pose(rig, "armL.R", fwd=40.0 + 24.0 * math.sin(a))
                for side in "LR":
                    pose(rig, "thigh." + side, fwd=10.0, out=8.0)
                    pose(rig, "shin." + side, fwd=-14.0)
            return f

        clip(rig, "buried", 36, {i: buried(i / 36.0) for i in range(0, 37, 6)})

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
