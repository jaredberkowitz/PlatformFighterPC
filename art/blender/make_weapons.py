"""The fighters' held weapons modelled by script (the animation overhaul, 2026-10-25: the owner asked for a modelled claw on the Claws
class's fists and for the Dagger's and Nodachi's weapons, in the same chunky, rounded style as the modelled gear, make_gear.py, and the
stage props). Each piece is a few shaped parts named by what they are made of, so the game colours them:
  metal*    the blade or claw steel           band*    the class's accent band (wrap, cuff)
  leather*  brown leather                     wood*    a wooden grip
Built in the hand's space as the game holds it: the weapon points along +X from the wrist (Blender X), the back of the hand is up
(Blender +Z, the game's +Y), and the hand's middle is at the origin. The game hangs it on the hand bone and turns +X along the forearm
(fighter_view.gd `_mount_weapon_models`).

Run:  blender --background --python art/blender/make_weapons.py [-- --only claw] [--preview <folder>]
Writes godot/models/weapons/<name>.glb (then `Godot --headless --path godot --import`). Keep each within the prop budget
(docs/ART_WORKFLOW.md: 3,000 triangles).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

OUT = "godot/models/weapons"


def material(name):
    m = bpy.data.materials.get(name)
    if m is None:
        m = bpy.data.materials.new(name)
    return m


def to_object(name, bm, smooth=True, sharp=50.0):
    me = bpy.data.meshes.new(name)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.normal_update()
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    obj.data.materials.append(material("m_" + name.split("_")[0]))
    for p in obj.data.polygons:
        p.use_smooth = smooth
    if smooth:
        obj.data.set_sharp_from_angle(angle=math.radians(sharp))
    return obj


def empty(name):
    e = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(e)
    return e


def tube(bm, points, radii, sides=10, flat=1.0, cap_end=True):
    """A tube along `points` (Vectors) with a radius per point, its cross-section squashed to `flat` across Z (a blade's edge), closed
    at the start and pointed or closed at the end."""
    rings = []
    for i, p in enumerate(points):
        a = points[min(i + 1, len(points) - 1)] - points[max(i - 1, 0)]
        a.normalize()
        side = a.cross(Vector((0.0, 0.0, 1.0)))
        if side.length < 1e-4:
            side = Vector((0.0, 1.0, 0.0))
        side.normalize()
        up = side.cross(a)
        ring = []
        for k in range(sides):
            t = math.tau * k / sides
            ring.append(bm.verts.new(p + side * math.cos(t) * radii[i] + up * math.sin(t) * radii[i] * flat))
        rings.append(ring)
    for r0, r1 in zip(rings, rings[1:]):
        for k in range(sides):
            bm.faces.new((r0[k], r0[(k + 1) % sides], r1[(k + 1) % sides], r1[k]))
    bm.faces.new(list(reversed(rings[0])))
    if cap_end:
        bm.faces.new(rings[-1])
    return rings


def rounded_box(bm, centre, size, bevel=0.35, segments=3):
    """A box with well rounded edges (a chunky plate)."""
    res = bmesh.ops.create_cube(bm, size=1.0)
    verts = res["verts"]
    for v in verts:
        v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2])) + centre
    bmesh.ops.bevel(bm, geom=list(bm.edges), offset=min(size) * bevel, segments=segments, affect="EDGES", profile=0.5)


def claw():
    """The Claws class's claw, one on each fist: a rounded steel plate over the knuckles, three thick curved claws out past the fist
    (stubby and rounded at the root, tapering to a blunt hook), and a wrap band round the wrist in the class colour. About 0.33 long
    past the knuckles, so the claw tips, not the fist, meet the hit."""
    root = empty("claw")
    bm = bmesh.new()
    # The knuckle plate, on the back of the hand.
    rounded_box(bm, Vector((0.07, 0.0, 0.095)), (0.2, 0.2, 0.06), bevel=0.45)
    to_object("metal_plate", bm).parent = root
    bm = bmesh.new()
    for y, lift, length in ((-0.072, 0.0, 0.9), (0.0, 0.012, 1.0), (0.072, 0.0, 0.9)):
        pts, radii = [], []
        n = 9
        for i in range(n):
            k = i / (n - 1)
            # Out along X, rising a little, then hooking down at the tip.
            x = 0.1 + 0.34 * length * k
            z = 0.075 + lift + 0.055 * math.sin(k * math.pi * 0.8) - 0.075 * k ** 3
            pts.append(Vector((x, y * (1.0 + 0.2 * k), z)))
            # (Chunky: thick and round at the root, tapering to a blunt rounded hook, not a needle.)
            radii.append(0.05 * (1.0 - 0.72 * k ** 1.5))
        tube(bm, pts, radii, sides=12, flat=0.8)
    to_object("metal_claws", bm).parent = root
    # The wrap at the wrist.
    bm = bmesh.new()
    pts = [Vector((-0.1, 0.0, 0.0)), Vector((-0.05, 0.0, 0.0)), Vector((0.0, 0.0, 0.0))]
    tube(bm, pts, [0.135, 0.145, 0.135], sides=20, flat=1.0)
    to_object("band_wrap", bm).parent = root
    return root


def _blade(bm, x0, x1, width, thick, curve=0.0, tip=0.25, steps=14):
    """A chunky blade along X from x0 to x1: a lens cross-section (thick spine, rounded edge) `width` tall in Z and `thick` across Y,
    bowed up by `curve` at the middle, narrowing to a rounded point over its last `tip` share."""
    rings = []
    sides = 10
    for i in range(steps + 1):
        k = i / steps
        x = x0 + (x1 - x0) * k
        z = curve * 4.0 * k * (1.0 - k) + curve * 0.6 * k
        w = width * (1.0 if k < 1.0 - tip else max(0.06, ((1.0 - k) / tip) ** 0.7))
        ring = []
        for j in range(sides):
            t = math.tau * j / sides
            ring.append(bm.verts.new((x, math.sin(t) * thick * 0.5 * (0.35 + 0.65 * abs(math.cos(t))), z + math.cos(t) * w * 0.5)))
        rings.append(ring)
    for r0, r1 in zip(rings, rings[1:]):
        for j in range(sides):
            bm.faces.new((r0[j], r0[(j + 1) % sides], r1[(j + 1) % sides], r1[j]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])


def _grip(bm, x0, x1, r, wraps=0):
    pts = [Vector((x0 + (x1 - x0) * k / 6.0, 0.0, 0.0)) for k in range(7)]
    radii = [r * (1.0 + (0.12 * math.sin(k * math.pi) if wraps else 0.0)) for k in range(7)]
    tube(bm, pts, radii, sides=12)


def dagger():
    """The Dagger class's knife, held in the weapon hand (hung on the hand bone like the claw, along the forearm's line): a short broad
    leaf blade with a thick spine and a rounded point, a round guard, a wrapped grip in the fist and a ball pommel. About 0.5 of blade
    past the fist, so the knife's point meets the jab's far hitboxes."""
    root = empty("dagger")
    bm = bmesh.new()
    _blade(bm, 0.1, 0.62, 0.16, 0.05, curve=0.0, tip=0.38)
    to_object("metal_blade", bm).parent = root
    bm = bmesh.new()
    rounded_box(bm, Vector((0.08, 0.0, 0.0)), (0.05, 0.12, 0.26), bevel=0.45)
    to_object("metal_guard", bm, sharp=60.0).parent = root
    bm = bmesh.new()
    _grip(bm, -0.16, 0.06, 0.045, wraps=1)
    to_object("band_grip", bm).parent = root
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=14, v_segments=10, radius=0.06)
    bmesh.ops.translate(bm, vec=Vector((-0.19, 0.0, 0.0)), verts=bm.verts)
    to_object("metal_pommel", bm).parent = root
    return root


def nodachi():
    """The Nodachi class's very long sword, laid out like the game's longsword so it is posed and stretched on the same weapon pivot
    (fighter_view.gd `blade_pivot`: the blade from 0.2 to 2.9 along X, the hand at 0): a long slim blade with a gentle upward curve and a
    chunky rounded spine, a round guard, and a long two-handed grip wrapped in the class colour with a cap."""
    root = empty("nodachi")
    bm = bmesh.new()
    _blade(bm, 0.18, 2.9, 0.15, 0.06, curve=0.08, tip=0.12, steps=24)
    to_object("metal_blade", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((0.1, 0.0, 0.0)), Vector((0.16, 0.0, 0.0))]
    tube(bm, pts, [0.16, 0.16], sides=20)
    to_object("metal_guard", bm).parent = root
    bm = bmesh.new()
    _grip(bm, -0.62, 0.1, 0.06, wraps=1)
    to_object("band_grip", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((-0.7, 0.0, 0.0)), Vector((-0.6, 0.0, 0.0))], [0.07, 0.075], sides=14)
    to_object("metal_cap", bm).parent = root
    return root


# ---- The pivot weapons (2026-10-28, the owner: the code-built weapons replaced with modelled ones). Laid out on the game's weapon pivot
# like the nodachi (fighter_view.gd `blade_pivot`: the hand at the origin, the blade or shaft along +X to 2.9, its width in Blender Z,
# the game's Y), so every pose, arc and reach that was fitted to the code-built pieces still fits. More part names:
#   gold*  the hilt fittings              edge*  a bright sharpened bevel (drawn unlit)
#   dark*  a fuller                       gem*   a glowing crystal (drawn unlit)
#   bronze* the greatsword's rivets


def _sections(bm, stations, sharp_faces=True):
    """Lofts closed cross-sections (lists of (y, z) in the plane square to X) through `stations` [(x, section)], capping both ends; a
    section that has shrunk to a point closes the end."""
    rings = []
    for x, sec in stations:
        rings.append([bm.verts.new((x, y, z)) for (y, z) in sec])
    n = len(stations[0][1])
    for r0, r1 in zip(rings, rings[1:]):
        for j in range(n):
            bm.faces.new((r0[j], r0[(j + 1) % n], r1[(j + 1) % n], r1[j]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)


def _torus(bm, centre, radius, minor, segments=32, sides=10, squash=1.0):
    """A closed ring in the X-Z plane (square to the game's camera) round `centre`, with no seam (a tube's caps left a notch)."""
    rings = []
    for i in range(segments):
        a = math.tau * i / segments
        out = Vector((math.cos(a), 0.0, math.sin(a)))
        rings.append([bm.verts.new(centre + out * (radius + minor * math.cos(t)) + Vector((0.0, minor * squash * math.sin(t), 0.0)))
                      for t in [math.tau * k / sides for k in range(sides)]])
    for i in range(segments):
        r0, r1 = rings[i], rings[(i + 1) % segments]
        for k in range(sides):
            bm.faces.new((r0[k], r1[k], r1[(k + 1) % sides], r0[(k + 1) % sides]))


# The blades' ends along X (round 8, the owner 2026-10-29: "the sword itself is a bit too long"). Fitted to the references' blade against
# their character's height: the swordsman's blade is about 0.6 of his height (ours ran 0.24 to 2.9, 1.2 times our 2.24-tall body), the
# broadsword reference's about 1.05 (ours was 1.2). The view's tips (fighter_view.gd BLADE_TIP_X_BY) follow these.
LONGSWORD_END = 1.64
GREATSWORD_END = 2.55


def longsword():
    """The Longsword's sword (also the Buckler's): a slim straight double-edged blade with a diamond section (a ridge down each face, so
    one bevel catches the light and the other falls into shade) and a long tapering point, a dark fuller down the middle, a gently swept
    crossguard with ball ends and a block where it meets the blade, a wrapped one-hand grip and a round pommel. The blade runs 0.24 to
    2.9 along X, 0.17 across, as the code-built one did."""
    root = empty("longsword")
    bm = bmesh.new()
    stations = []
    x0, x1 = 0.24, LONGSWORD_END
    tip_from = x1 - 0.58 * (x1 - x0) / 2.66
    for i in range(17):
        x = x0 + (x1 - x0) * i / 16
        k = 1.0 if x <= tip_from else max(0.0, (x1 - x) / (x1 - tip_from)) ** 0.85
        w, t, flat = 0.085 * k, 0.04 * k + 0.004, 0.011 * k
        stations.append((x, [(flat, w), (t, 0.0), (flat, -w), (-flat, -w), (-t, 0.0), (-flat, w)]))
    _sections(bm, stations)
    to_object("metal_blade", bm, sharp=25.0).parent = root
    bm = bmesh.new()
    _sections(bm, [(x, [(0.046, 0.022), (0.046, -0.022), (-0.046, -0.022), (-0.046, 0.022)]) for x in (0.3, tip_from - 0.15)])
    to_object("dark_fuller", bm, sharp=60.0).parent = root
    # The crossguard: a round bar across the blade (Z), its ends swept a little toward the point, with ball ends and a block in the middle.
    bm = bmesh.new()
    # (The block first: `rounded_box` bevels every edge already in the mesh.)
    rounded_box(bm, Vector((0.2, 0.0, 0.0)), (0.13, 0.13, 0.2), bevel=0.4, segments=2)
    pts = [Vector((0.17 + 0.05 * (abs(z) / 0.32) ** 2, 0.0, z)) for z in [-0.32 + 0.64 * k / 10 for k in range(11)]]
    tube(bm, pts, [0.045] * len(pts), sides=12)
    for z in (-0.33, 0.33):
        s = bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.072)
        bmesh.ops.translate(bm, vec=Vector((0.22, 0.0, z)), verts=s["verts"])
    to_object("gold_guard", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((-0.24 + 0.38 * k / 12, 0.0, 0.0)) for k in range(13)]
    tube(bm, pts, [0.062 + 0.008 * abs(math.sin(k * math.pi * 3 / 12)) for k in range(13)], sides=12)
    to_object("band_grip", bm).parent = root
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=14, v_segments=10, radius=0.085)
    bmesh.ops.translate(bm, vec=Vector((-0.3, 0.0, 0.0)), verts=bm.verts)
    to_object("gold_pommel", bm).parent = root
    return root


def greatsword():
    """The Greatsword's broad single-edged blade (the code-built one's outline: 0.2 to 2.9 along X, 0.5 across, the back straight and the
    edge cut in a long chisel to the point): thick at the spine and thin at the edge, the sharpened bevel its own part so it reads
    bright, a dark fuller near the back, a heavy bronze guard with two rivets, a long two-hand leather grip and a round pommel."""
    root = empty("greatsword")
    x0, x1 = 0.2, GREATSWORD_END
    chisel = x1 - 0.5
    body, bevel = [], []
    for i in range(19):
        x = x0 + (x1 - x0) * i / 18
        e = -0.25 if x <= chisel else -0.25 + 0.5 * (x - chisel) / (x1 - chisel)
        s = (0.25 - e) / 0.5
        top = e + 0.15 * s
        spine, mid, sharp_t = 0.07 * (0.35 + 0.65 * s), 0.055 * (0.35 + 0.65 * s), 0.008
        body.append((x, [(spine, 0.25), (mid, top), (-mid, top), (-spine, 0.25)]))
        bevel.append((x, [(mid, top), (sharp_t * s, e), (-sharp_t * s, e), (-mid, top)]))
    bm = bmesh.new()
    _sections(bm, body)
    to_object("metal_blade", bm, sharp=30.0).parent = root
    bm = bmesh.new()
    _sections(bm, bevel)
    to_object("edge_bevel", bm, sharp=30.0).parent = root
    bm = bmesh.new()
    _sections(bm, [(x, [(0.076, 0.135), (0.076, 0.065), (-0.076, 0.065), (-0.076, 0.135)]) for x in (0.36, chisel - 0.2)])
    to_object("dark_fuller", bm, sharp=60.0).parent = root
    bm = bmesh.new()
    rounded_box(bm, Vector((0.12, 0.0, 0.0)), (0.2, 0.24, 0.86), bevel=0.3, segments=3)
    to_object("gold_guard", bm).parent = root
    bm = bmesh.new()
    for z in (-0.3, 0.3):
        for sy in (-1.0, 1.0):
            r = bmesh.ops.create_uvsphere(bm, u_segments=10, v_segments=6, radius=0.05)
            for v in r["verts"]:
                v.co = Vector((0.12 + v.co.x, sy * (0.12 + 0.6 * abs(v.co.y)), z + v.co.z))
    to_object("bronze_rivets", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((-0.6 + 0.62 * k / 16, 0.0, 0.0)) for k in range(17)]
    tube(bm, pts, [0.07 + 0.01 * abs(math.sin(k * math.pi * 4 / 16)) for k in range(17)], sides=12)
    to_object("leather_grip", bm).parent = root
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=14, v_segments=10, radius=0.11)
    bmesh.ops.translate(bm, vec=Vector((-0.66, 0.0, 0.0)), verts=bm.verts)
    to_object("gold_pommel", bm).parent = root
    return root


def staff():
    """The Seer's staff (the code-built one's layout: held near its foot at the origin, its head at 2.6 along X): a pale wooden shaft
    tapering toward the head with a gold foot cap and a wrap where the hand holds it, a gold collar, and a round gold ring standing
    square to the camera (in X and Z) on two short prongs, a glowing crystal floating in it."""
    root = empty("staff")
    bm = bmesh.new()
    pts = [Vector((-0.34 + 2.66 * k / 16, 0.0, 0.0)) for k in range(17)]
    tube(bm, pts, [0.092 - 0.02 * k / 16 + 0.006 * math.sin(k * 1.7) for k in range(17)], sides=12)
    to_object("wood_shaft", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((-0.42, 0.0, 0.0)), Vector((-0.38, 0.0, 0.0)), Vector((-0.3, 0.0, 0.0))], [0.07, 0.1, 0.1], sides=14)
    tube(bm, [Vector((2.2, 0.0, 0.0)), Vector((2.26, 0.0, 0.0)), Vector((2.34, 0.0, 0.0)), Vector((2.38, 0.0, 0.0))],
         [0.1, 0.135, 0.135, 0.1], sides=16)
    to_object("gold_collar", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((-0.2 + 0.42 * k / 10, 0.0, 0.0)) for k in range(11)]
    tube(bm, pts, [0.1 + 0.008 * abs(math.sin(k * math.pi * 3 / 10)) for k in range(11)], sides=12)
    to_object("band_wrap", bm).parent = root
    # The ring (in X and Z, the plane facing the camera) and the two prongs holding it on the collar.
    bm = bmesh.new()
    _torus(bm, Vector((2.64, 0.0, 0.0)), 0.24, 0.042)
    for sz in (-1.0, 1.0):
        prong = [Vector((2.34 + 0.08 * k, 0.0, sz * (0.07 + 0.12 * math.sin(k * math.pi / 4)))) for k in range(4)]
        tube(bm, prong, [0.035, 0.033, 0.03, 0.03], sides=8)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    to_object("gold_ring", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((2.44, 0.0, 0.0)), Vector((2.64, 0.0, 0.0)), Vector((2.84, 0.0, 0.0))]
    rings = []
    for p, rad in zip(pts, [0.0, 0.13, 0.0]):
        rings.append([bm.verts.new(p + Vector((0.0, rad * math.cos(a), rad * math.sin(a)))) for a in [math.tau * k / 4 + 0.785 for k in range(4)]])
    for r0, r1 in zip(rings, rings[1:]):
        for j in range(4):
            bm.faces.new((r0[j], r0[(j + 1) % 4], r1[(j + 1) % 4], r1[j]))
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    to_object("gem_crystal", bm, smooth=False).parent = root
    return root


def round_shield():
    """The Buckler's round shield, worn on the other forearm facing the camera (the game's +Z, Blender -Y): a domed face of wooden planks
    0.42 across the radius, a steel rim, six rivets and a boss in the fighter's accent colour."""
    root = empty("round_shield")
    bm = bmesh.new()
    sides = 32
    rings = []
    for k in range(7):
        rr = 0.42 * k / 6
        dome = 0.07 * (1.0 - (rr / 0.42) ** 2)
        rings.append([bm.verts.new((rr * math.cos(math.tau * i / sides), -0.04 - dome, rr * math.sin(math.tau * i / sides))) for i in range(sides)])
    back = [bm.verts.new((0.42 * math.cos(math.tau * i / sides), 0.03, 0.42 * math.sin(math.tau * i / sides))) for i in range(sides)]
    for r0, r1 in zip(rings, rings[1:]):
        for i in range(sides):
            bm.faces.new((r0[i], r1[i], r1[(i + 1) % sides], r0[(i + 1) % sides]))
    for i in range(sides):
        bm.faces.new((rings[-1][i], back[i], back[(i + 1) % sides], rings[-1][(i + 1) % sides]))
    bm.faces.new(list(back))
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    to_object("wood_face", bm).parent = root
    bm = bmesh.new()
    _torus(bm, Vector((0.0, -0.02, 0.0)), 0.42, 0.045, segments=40)
    for k in range(6):
        a = math.tau * k / 6 + 0.26
        s = bmesh.ops.create_uvsphere(bm, u_segments=8, v_segments=6, radius=0.03)
        bmesh.ops.translate(bm, vec=Vector((0.34 * math.cos(a), -0.075, 0.34 * math.sin(a))), verts=s["verts"])
    to_object("metal_rim", bm).parent = root
    bm = bmesh.new()
    s = bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=10, radius=0.13)
    for v in s["verts"]:
        v.co = Vector((v.co.x, -0.1 + min(v.co.y, 0.0) * 0.55, v.co.z))
    to_object("band_boss", bm).parent = root
    return root


WEAPONS = {"claw": claw, "dagger": dagger, "nodachi": nodachi, "longsword": longsword, "greatsword": greatsword, "staff": staff,
           "round_shield": round_shield}


def export(root, path):
    bpy.ops.object.select_all(action="DESELECT")
    root.select_set(True)
    for c in root.children_recursive:
        c.select_set(True)
    bpy.ops.export_scene.gltf(filepath=os.path.abspath(path), use_selection=True, export_format="GLB", export_yup=True,
                              export_apply=True)


def preview(root, path):
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.show_object_outline = True
    scene.render.resolution_x = 480
    scene.render.resolution_y = 480
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    lo, hi = Vector((1e9,) * 3), Vector((-1e9,) * 3)
    for c in root.children_recursive:
        for v in c.data.vertices:
            lo = Vector(map(min, lo, v.co))
            hi = Vector(map(max, hi, v.co))
    cam_data.ortho_scale = max(1.1, (hi - lo).length * 1.05)
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    target = (lo + hi) * 0.5
    cam.location = target + Vector((0.9, -2.2, 1.3)) * max(1.0, (hi - lo).length)
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    only = opts["only"].split(",") if "only" in opts else list(WEAPONS)
    os.makedirs(OUT, exist_ok=True)
    for name in only:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        root = WEAPONS[name]()
        export(root, os.path.join(OUT, name + ".glb"))
        tris = sum(sum(len(p.vertices) - 2 for p in c.data.polygons) for c in root.children_recursive if c.type == "MESH")
        print("weapon %s: %d triangles" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
