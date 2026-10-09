"""Builds the rigged blob fighter (arms, legs, a skeleton and a set of animation clips) and exports godot/models/blob_rig.glb.

Run:  blender --background --python art/blender/make_rigged_blob.py            (blob_rig.glb)
      blender --background --python art/blender/make_rigged_blob.py -- --long   (blob_rig_long.glb: the brawler's longer limbs)

The character is original: a big round head, a squat dumpling torso, stubby capsule limbs, mitten hands and chunky shoes, in the spirit of
docs/ART_DIRECTION.md. The game paints the flat colours (by part name), draws the face and accessories itself, and plays the clips below.

Axes while modelling (Blender, Z up): the character faces -Y; its left is -X. Every bone points up (+Z) from its joint, so a bone's local X is
the world X (a swing forwards and backwards) and the bone's own Y is the vertical axis (a twist). `pose()` takes angles in the terms a person
would use (forward, outward, twist) and does the sign bookkeeping once.

Parts are rigid (each is weighted fully to one bone), like a vinyl toy: round, readable, cheap, and the squash and lean the game adds on top
read well on it.
"""

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


def ball(bm, radius, centre, scale=(1.0, 1.0, 1.0), segments=32, rings=20, uvs=False):
    """A ball. With `uvs` it is unwrapped like a globe (u around, v from the bottom up), for cloth patterns."""
    if uvs:
        bm.loops.layers.uv.verify()
    res = bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=radius, calc_uvs=uvs)
    for v in res["verts"]:
        v.co.x *= scale[0]
        v.co.y *= scale[1]
        v.co.z *= scale[2]
        v.co += Vector(centre)
    return res["verts"]


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


def build_meshes(rig):
    parts = {}

    bm = bmesh.new()
    ball(bm, 0.56, (0, 0, 0.98), (1.0, 0.9, 0.75), uvs=True)
    for v in bm.verts:
        t = -(v.co.z - 0.98) / 0.42
        v.co.x *= 1.0 + 0.08 * t
        v.co.y *= 1.0 + 0.08 * t
    parts["Body"] = to_object("Body", bm, "spine", rig)

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

    for side, x in (("L", -1.0), ("R", 1.0)):
        bm = bmesh.new()
        capsule(bm, (x * 0.5, 0.0, 1.15), 0.13, (x * 0.66, -0.02, 0.95), 0.115)
        parts["ArmU." + side] = to_object("ArmU." + side, bm, "armU." + side, rig)

        # A short shirt sleeve: a wider capsule over the top of the upper arm, open at the bottom, so the arm comes out of it.
        bm = bmesh.new()
        capsule(bm, (x * 0.48, 0.0, 1.18), 0.17, (x * 0.62, -0.015, 1.02), 0.16)
        cut = [v for v in bm.verts if (v.co - Vector((x * 0.62, -0.015, 1.02))).dot(Vector((x * 0.14, -0.015, -0.16)).normalized()) > 0.02]
        bmesh.ops.delete(bm, geom=cut, context="VERTS")
        parts["Sleeve." + side] = to_object("Sleeve." + side, bm, "armU." + side, rig)

        bm = bmesh.new()
        capsule(bm, (x * 0.66, -0.02, 0.95), 0.115, (x * 0.74, -0.04, 0.8), 0.105)
        parts["ArmL." + side] = to_object("ArmL." + side, bm, "armL." + side, rig)

        # A mitten: a round palm with a small thumb on the inner side.
        bm = bmesh.new()
        ball(bm, 0.2, (x * 0.74, -0.04, 0.72), segments=28, rings=18)
        ball(bm, 0.08, (x * 0.74 - x * 0.15, -0.09, 0.76), (0.9, 1.3, 1.0), segments=14, rings=10)
        parts["Hand." + side] = to_object("Hand." + side, bm, "hand." + side, rig)

        # A rolled glove cuff just above the mitten: a flat disc square to the forearm, wider than the arm.
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=24, radius1=0.15, radius2=0.165, depth=0.08)
        elbow = Vector((x * 0.66, -0.02, 0.95))
        wrist = Vector((x * 0.74, -0.04, 0.8))
        axis = (wrist - elbow).normalized()
        rot = Vector((0.0, 0.0, 1.0)).rotation_difference(axis).to_matrix().to_4x4()
        bmesh.ops.transform(bm, matrix=rot, verts=bm.verts)
        bmesh.ops.translate(bm, vec=elbow + (wrist - elbow) * 0.3, verts=bm.verts)
        parts["Cuff." + side] = to_object("Cuff." + side, bm, "hand." + side, rig)

        bm = bmesh.new()
        capsule(bm, (x * 0.26, 0.0, 0.64), 0.14, (x * 0.26, 0.0, 0.42), 0.125)
        parts["Thigh." + side] = to_object("Thigh." + side, bm, "thigh." + side, rig)

        bm = bmesh.new()
        capsule(bm, (x * 0.26, 0.0, 0.42), 0.125, (x * 0.26, 0.0, 0.26), 0.12)
        parts["Shin." + side] = to_object("Shin." + side, bm, "shin." + side, rig)

        # A chunky shoe: a flat sole and a rounded toe cap.
        bm = bmesh.new()
        ball(bm, 0.22, (x * 0.26, -0.05, 0.17), (1.0, 1.35, 0.8), segments=28, rings=18)
        for v in bm.verts:
            if v.co.z < 0.08:
                v.co.z = 0.08 + (v.co.z - 0.08) * 0.15
        parts["Foot." + side] = to_object("Foot." + side, bm, "foot." + side, rig)

        # A strap across the top of the shoe (in the outfit colour): a band of the shoe's own shape, a hair bigger. The ball is built with
        # its rings running across the shoe (poles front and back), so the band's edges are clean.
        bm = bmesh.new()
        res = bmesh.ops.create_uvsphere(bm, u_segments=28, v_segments=28, radius=0.22 * 1.05)
        bmesh.ops.rotate(bm, cent=(0, 0, 0), matrix=Matrix.Rotation(math.pi / 2.0, 3, "X"), verts=res["verts"])
        for v in res["verts"]:
            v.co.y *= 1.35
            v.co.z *= 0.8
            v.co += Vector((x * 0.26, -0.05, 0.17))
            if v.co.z < 0.08:
                v.co.z = 0.08 + (v.co.z - 0.08) * 0.15
        off = [v for v in bm.verts if not (-0.21 < v.co.y < -0.1) or v.co.z < 0.13]
        bmesh.ops.delete(bm, geom=off, context="VERTS")
        parts["Strap." + side] = to_object("Strap." + side, bm, "foot." + side, rig)

        # Clothes: a sole under each shoe, and the shorts' legs over the top of each thigh.
        bm = bmesh.new()
        ball(bm, 0.215, (x * 0.26, -0.05, 0.085), (1.02, 1.37, 0.18), segments=28, rings=10)
        parts["Sole." + side] = to_object("Sole." + side, bm, "foot." + side, rig)
        bm = bmesh.new()
        capsule(bm, (x * 0.26, 0.0, 0.66), 0.165, (x * 0.26, 0.0, 0.52), 0.155)
        parts["ShortsLeg." + side] = to_object("ShortsLeg." + side, bm, "thigh." + side, rig)

    # The shorts: the bottom of the body, a little bigger than it, cut off at the waist.
    bm = bmesh.new()
    ball(bm, 0.585, (0, 0, 0.98), (1.0, 0.9, 0.75))
    for v in bm.verts:
        t = -(v.co.z - 0.98) / 0.42
        v.co.x *= 1.0 + 0.08 * t
        v.co.y *= 1.0 + 0.08 * t
    waist = [v for v in bm.verts if v.co.z > 0.8]
    bmesh.ops.delete(bm, geom=waist, context="VERTS")
    parts["Shorts"] = to_object("Shorts", bm, "hips", rig)

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
    """Sets a bone's pose from forward / outward / twist degrees (and a vertical offset for the hips)."""
    fwd, out, twist, lift = fwd * AMP, out * AMP, twist * AMP, lift * AMP
    pb = rig.pose.bones[bone]
    pb.rotation_mode = "XYZ"
    limb = bone.split(".")[0] in LIMBS
    left = bone.endswith(".L")
    x = -fwd if limb else fwd
    z = (-out if left else out)
    pb.rotation_euler = (math.radians(x), math.radians(twist), math.radians(z))
    if bone == "hips":
        pb.location = (0.0, 0.0, lift)


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


def clip(rig, name, frames, poses):
    """`poses` maps a frame to a function that sets the pose; the clip is keyed at each of those frames."""
    global AMP
    if name.startswith("attack") or name in ("grab", "throw"):
        AMP = 1.65
    elif name.startswith(("sword_", "kick_")) or name == "blaster":
        AMP = 1.5
    elif name in ("walk", "run", "dash", "skid"):
        AMP = 1.0
    elif name in ("jump", "fall", "hurt", "crouch", "shield", "roll", "knockdown"):
        AMP = 1.25
    else:
        AMP = 1.0
    action = bpy.data.actions.new(name)
    action.use_fake_user = True
    if rig.animation_data is None:
        rig.animation_data_create()
    rig.animation_data.action = action
    for frame in sorted(poses):
        reset(rig)
        poses[frame](rig)
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
                if name.startswith(("Thigh", "Shin")):
                    co.z = leg_z(co.z)
                elif name.startswith(("Foot", "Sole", "Strap")):
                    pass
                elif name.startswith("ShortsLeg"):
                    co.z = leg_z(co.z)
                elif name.startswith(("ArmU", "ArmL", "Sleeve")):
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
    parts = build_meshes(rig)
    if LONG:
        lengthen_limbs(rig, parts)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="POSE")
    bpy.context.scene.render.fps = FPS

    # Standing, breathing.
    def idle(k):
        def f(rig):
            b = math.sin(k * 2 * math.pi)
            pose(rig, "hips", lift=0.015 * b)
            pose(rig, "spine", fwd=1.5 * b)
            pose(rig, "head", fwd=-1.0 * b)
            for side in "LR":
                pose(rig, "armU." + side, fwd=3.0 * b, out=10.0)
                pose(rig, "armL." + side, fwd=8.0)
                pose(rig, "thigh." + side, out=0.0)
        return f

    clip(rig, "idle", 90, {0: idle(0.0), 22: idle(0.25), 45: idle(0.5), 67: idle(0.75), 90: idle(1.0)})

    n = 8
    clip(rig, "walk", 40, {int(40 * i / n): walk_pose(i / n, 34.0, 46.0, 28.0, 5.0, 0.05, 26.0, 5.0) for i in range(n + 1)})
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

    def jump(rig):
        for side in "LR":
            pose(rig, "armU." + side, fwd=20.0, out=55.0)
            pose(rig, "armL." + side, fwd=25.0)
            pose(rig, "thigh." + side, fwd=26.0 if side == "L" else 12.0)
            pose(rig, "shin." + side, fwd=-46.0 if side == "L" else -30.0)
        pose(rig, "spine", fwd=-4.0)
        pose(rig, "head", fwd=4.0)

    clip(rig, "jump", 12, {0: jump, 12: jump})

    def fall(k):
        def f(rig):
            w = math.sin(k * 2 * math.pi)
            for side, sign in (("L", 1.0), ("R", -1.0)):
                pose(rig, "armU." + side, fwd=-10.0 + 6.0 * w * sign, out=70.0 + 8.0 * w)
                pose(rig, "armL." + side, fwd=14.0)
                pose(rig, "thigh." + side, fwd=8.0 * sign * w + 6.0, out=6.0)
                pose(rig, "shin." + side, fwd=-14.0 - 6.0 * w * sign)
            pose(rig, "spine", fwd=3.0)
        return f

    clip(rig, "fall", 30, {0: fall(0.0), 8: fall(0.25), 15: fall(0.5), 23: fall(0.75), 30: fall(1.0)})

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
            {"hips": -0.2, "spine": (30.0, -10.0), "head": (-10.0, 0.0), "stance": (64.0, 30.0), "shin.L": (-96.0,), "shin.R": (-100.0,)},
            {"hips": -0.22, "spine": (40.0, 12.0), "head": (-18.0, 0.0), "stance": (78.0, 18.0), "shin.L": (-60.0,), "shin.R": (-104.0,)},
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

    # Claws fighter. The kicking leg is aimed by the game; these set the torso, arms and the other leg.
    # Neutral air: a spinning kick, tucked, then opened out and turning.
    move_of("kick_nair",
            {"spine": (22.0, -20.0), "stance": (60.0, 64.0), "shin.L": (-90.0,), "shin.R": (-90.0,), "arms": (30.0, 30.0)},
            {"spine": (-10.0, 60.0), "head": (-6.0, -20.0), "stance": (20.0, -30.0), "arms": (-10.0, 60.0)},
            {"spine": (0.0, 20.0), "stance": (20.0, 10.0)})
    # Back air: lean forward and drive the heel back.
    move_of("kick_bair",
            {"spine": (24.0, 10.0), "head": (10.0, 20.0), "stance": (60.0, 50.0), "shin.L": (-90.0,), "shin.R": (-90.0,), "arms": (40.0, 20.0)},
            {"spine": (42.0, -24.0), "head": (-24.0, 34.0), "stance": (50.0, -20.0), "shin.L": (-80.0,), "arms": (60.0, 30.0)},
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
    move_of("kick_up",
            {"hips": -0.06, "spine": (12.0, 0.0), "stance": (10.0, -16.0), "arms": (10.0, 24.0)},
            {"hips": -0.02, "spine": (34.0, 0.0), "head": (14.0, 0.0), "stance": (-6.0, 0.0), "arms": (40.0, 30.0)},
            {"spine": (12.0, 0.0), "stance": (4.0, -6.0)})
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
