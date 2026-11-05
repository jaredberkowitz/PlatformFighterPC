"""The Settler's tools, held in the hand for its moves, modelled by script in the chunky style of the weapons (make_weapons.py) and
the match props (make_match_props.py). The art director's props pass, 2026-11-04: the Settler (the Pockets class) swung an umbrella,
dug with a shovel, chopped with an axe, watered, fired a slingshot and a firework, held turnips and floated on balloons with empty
hands. Each tool is a few shaped parts named by what they are made of (coloured by godot/scripts/held_props.gd):
  wood*  metal*  dark*  band*  paint*  rim*  pith*  petal*  leaf*  glass*  leather*

Run:  blender --background --python art/blender/make_tools.py [-- --only umbrella] [--preview <folder>]
Writes godot/models/tools/<name>.glb (then `Godot --headless --path godot --import`). Keep each within the prop budget (3,000
triangles).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

OUT = "godot/models/tools"


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


def tube(bm, points, radii, sides=10, flat=1.0, cap_start=True, cap_end=True, axis_hint=Vector((0.0, 0.0, 1.0))):
    """A tube along `points` (Vectors) with a radius per point, its cross-section squashed to `flat` across the hint axis."""
    rings = []
    side = None
    for i, p in enumerate(points):
        a = points[min(i + 1, len(points) - 1)] - points[max(i - 1, 0)]
        a.normalize()
        if side is None:
            side = a.cross(axis_hint)
            if side.length < 1e-3:
                side = a.cross(Vector((0.0, 1.0, 0.0)) if abs(a.y) < 0.9 else Vector((1.0, 0.0, 0.0)))
        else:
            # Carried along the path (parallel transport), so the rings never flip or twist where the path bends.
            side = side - a * side.dot(a)
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
    if cap_start:
        bm.faces.new(list(reversed(rings[0])))
    if cap_end:
        bm.faces.new(rings[-1])
    return rings


def blob(bm, centre, radii, u=16, v=10):
    """A squashed sphere (`radii` along X, Y, Z) at `centre`."""
    s = bmesh.ops.create_uvsphere(bm, u_segments=u, v_segments=v, radius=1.0)
    for vert in s["verts"]:
        vert.co = Vector((vert.co.x * radii[0], vert.co.y * radii[1], vert.co.z * radii[2])) + centre
    return s["verts"]


def transform(verts, m):
    for v in verts:
        v.co = m @ v.co


def rounded_box(bm, centre, size, bevel=0.35, segments=3):
    res = bmesh.ops.create_cube(bm, size=1.0)
    for v in res["verts"]:
        v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2])) + centre
    bmesh.ops.bevel(bm, geom=[e for e in bm.edges if e.verts[0] in res["verts"]], offset=min(size) * bevel, segments=segments,
                    affect="EDGES", profile=0.5)
    return res["verts"]


def box_part(name, size, m, bevel=0.3, segments=2, parent=None):
    """A bevelled box `size` as its own object, placed by matrix `m` (a bevel replaces the box's verts, so it is built alone and
    moved whole)."""
    bm = bmesh.new()
    rounded_box(bm, Vector((0.0, 0.0, 0.0)), size, bevel=bevel, segments=segments)
    transform(bm.verts, m)
    obj = to_object(name, bm)
    if parent is not None:
        obj.parent = parent
    return obj


def solidify(bm, faces, thick):
    """Gives `faces` a thickness (bmesh's solidify offsets only with the faces' verts and edges given too and the normals up to date)."""
    geom = set()
    for f in faces:
        geom.add(f)
        geom.update(f.verts)
        geom.update(f.edges)
    bm.normal_update()
    bmesh.ops.solidify(bm, geom=list(geom), thickness=thick)


def ribbon(bm, pts, widths, across, thick):
    """A strip along `pts` (Vectors), `widths` wide across the unit vector `across` at each point, given `thick` thickness (a flap of
    peel, a flat boomerang): the width and the thickness are free of each other, unlike a squashed tube's."""
    left = [bm.verts.new(p - across * (w * 0.5)) for p, w in zip(pts, widths)]
    right = [bm.verts.new(p + across * (w * 0.5)) for p, w in zip(pts, widths)]
    faces = [bm.faces.new((left[i], right[i], right[i + 1], left[i + 1])) for i in range(len(pts) - 1)]
    solidify(bm, faces, thick)


def lathe(bm, profile, sides=16, cap_bottom=True, cap_top=True):
    """Turns a profile [(radius, z)] about Z (a pot, a cork standing up)."""
    rings = []
    for r, z in profile:
        rings.append([bm.verts.new((r * math.cos(math.tau * k / sides), r * math.sin(math.tau * k / sides), z)) for k in range(sides)])
    for r0, r1 in zip(rings, rings[1:]):
        for k in range(sides):
            bm.faces.new((r0[k], r0[(k + 1) % sides], r1[(k + 1) % sides], r1[k]))
    if cap_bottom:
        bm.faces.new(list(reversed(rings[0])))
    if cap_top:
        bm.faces.new(rings[-1])
    return rings


def petal(bm, root, direction, length, width, thick, lift=0.0, curl=0.0, steps=6, sides=8):
    """A flat, pointed leaf or peel flap from `root` along `direction` (in the XY plane unless `lift` tips it), widest a third along,
    its tip curling up by `curl`."""
    d = direction.normalized()
    pts, radii = [], []
    for i in range(steps + 1):
        t = i / steps
        p = root + d * length * t + Vector((0.0, 0.0, lift * t + curl * t * t * t))
        pts.append(p)
        radii.append(max(width * 0.5 * math.sin(math.pi * min(1.0, t * 1.15 + 0.06)) ** 0.8, 0.004))
    # Flat across the up axis: a leaf lies in the plane of its direction and the side axis.
    return tube(bm, pts, radii, sides=sides, flat=thick / max(width * 0.5, 1e-3), axis_hint=Vector((0.0, 0.0, 1.0)))


# ---- The Settler's tools (the Pockets class) ----------------------------------------------------------------------------------------
# Built in the hand's space as the game holds them, as the weapons are (make_weapons.py): the tool runs along +X out from the fist (the
# forearm's line), the back of the hand is +Z (the game's +Y), the hand's middle at the origin. A grip pass with the animator may turn
# a tool in the hand; the parts stay named by what they are made of.


def _sections(bm, stations):
    """Lofts closed cross-sections [(y, z)] through stations [(x, section)], capping both ends."""
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


def umbrella():
    """The forward tilt's umbrella, open: a wooden shaft with a crook at the fist's end, and a domed canopy of eight panels, red and cream
    in turn, opening forward at the far end, with a little tip."""
    root = empty("umbrella")
    bm = bmesh.new()
    tube(bm, [Vector((-0.12, 0.0, 0.0)), Vector((0.95, 0.0, 0.0))], [0.022, 0.02], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    # The crook: a short hook behind the fist.
    pts = [Vector((-0.12, 0.0, 0.0)), Vector((-0.2, 0.0, -0.02)), Vector((-0.25, 0.0, -0.08)), Vector((-0.22, 0.0, -0.14)),
           Vector((-0.16, 0.0, -0.15))]
    tube(bm, pts, [0.024] * 5, sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("wood_shaft", bm).parent = root
    # The canopy: a dome about the shaft (+X), radius 0.5, its rim at x 0.62 and its top at 0.9, scalloped between the ribs.
    panels = 8
    for k in range(panels):
        bm = bmesh.new()
        a0, a1 = math.tau * k / panels, math.tau * (k + 1) / panels
        rings = []
        for i in range(7):
            t = i / 6.0
            r = 0.5 * math.sin(math.pi * 0.5 * (1.0 - t)) + 0.0001
            x = 0.62 + 0.28 * math.sin(math.pi * 0.5 * t)
            row = []
            for j in range(5):
                a = a0 + (a1 - a0) * j / 4.0
                sag = 1.0 - 0.06 * math.sin(math.pi * j / 4.0) * (1.0 - t)
                row.append(bm.verts.new((x, r * sag * math.cos(a), r * sag * math.sin(a))))
            rings.append(row)
        for r0, r1 in zip(rings, rings[1:]):
            for j in range(4):
                bm.faces.new((r0[j], r0[j + 1], r1[j + 1], r1[j]))
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        solidify(bm, list(bm.faces), 0.02)
        to_object(("band_panel%d" if k % 2 == 0 else "paint_panel%d") % k, bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.9, 0.0, 0.0)), Vector((1.0, 0.0, 0.0))], [0.028, 0.012], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("dark_tip", bm).parent = root
    return root


def shovel():
    """The shovel (the down smash and the planting): a long wooden handle from the fist, a dark collar, and a pointed steel spade at the
    far end, flat to the back of the hand."""
    root = empty("shovel")
    bm = bmesh.new()
    tube(bm, [Vector((-0.12, 0.0, 0.0)), Vector((0.82, 0.0, 0.0))], [0.026, 0.024], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("wood_handle", bm).parent = root
    bm = bmesh.new()
    # The spade: a rounded, pointed blade lofted along +X, thin across Z (the back of the hand) and wide across Y.
    stations = []
    for i in range(9):
        t = i / 8.0
        x = 0.8 + 0.44 * t
        half_w = max(0.18 * math.sin(math.pi * min(1.0, 0.35 + t * 0.75)) ** 0.7, 0.014) if t < 0.95 else 0.014
        th = 0.016
        stations.append((x, [(th, half_w), (-th, half_w), (-th, -half_w), (th, -half_w)]))
    _sections(bm, stations)
    to_object("metal_spade", bm, sharp=40.0).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.76, 0.0, 0.0)), Vector((0.84, 0.0, 0.0))], [0.04, 0.038], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("dark_collar", bm).parent = root
    return root


def axe():
    """The axe (felling the tree): a wooden handle with a steel head at its far end, the cutting edge on the palm's side."""
    root = empty("axe")
    bm = bmesh.new()
    tube(bm, [Vector((-0.12, 0.0, 0.0)), Vector((0.3, 0.0, 0.01)), Vector((0.66, 0.0, 0.0))], [0.03, 0.027, 0.03], sides=8,
         axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("wood_handle", bm).parent = root
    bm = bmesh.new()
    # The head: a wedge from the handle's end out toward -Z (the palm's side), widening along X to a broad edge.
    rings = []
    for i in range(7):
        t = i / 6.0
        z = 0.04 - 0.27 * t
        half_x = 0.055 + 0.075 * t * t
        th = 0.035 * (1.0 - 0.8 * t) + 0.006
        rings.append([bm.verts.new((0.6 + dx, dy, z)) for (dx, dy) in ((half_x, th), (half_x, -th), (-half_x, -th), (-half_x, th))])
    for r0, r1 in zip(rings, rings[1:]):
        for j in range(4):
            bm.faces.new((r0[j], r0[(j + 1) % 4], r1[(j + 1) % 4], r1[j]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    to_object("metal_head", bm, sharp=35.0).parent = root
    bm = bmesh.new()
    blob(bm, Vector((0.6, 0.0, 0.05)), (0.07, 0.045, 0.05), u=10, v=6)
    to_object("metal_poll", bm).parent = root
    return root


def watering_can():
    """The watering can (the planting's second step): a round green can under the fist, its handle arching over the top into the hand,
    a long spout reaching forward and up to a rose."""
    root = empty("watering_can")
    bm = bmesh.new()
    lathe(bm, [(0.0, -0.12), (0.12, -0.12), (0.14, -0.08), (0.14, 0.06), (0.11, 0.1), (0.0, 0.1)], sides=16)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    transform(bm.verts, Matrix.Translation(Vector((0.2, 0.0, -0.2))))
    to_object("paint_can", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.06, 0.0, -0.12)), Vector((0.02, 0.0, -0.02)), Vector((0.08, 0.0, 0.03)), Vector((0.2, 0.0, 0.03)),
              Vector((0.3, 0.0, -0.04)), Vector((0.32, 0.0, -0.1))], [0.022] * 6, sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    tube(bm, [Vector((0.32, 0.0, -0.22)), Vector((0.5, 0.0, -0.12)), Vector((0.64, 0.0, 0.0))], [0.03, 0.022, 0.018], sides=8,
         axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("paint_spout", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.0, 0.0), (0.02, 0.0), (0.05, 0.05), (0.0, 0.055)], sides=10)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    transform(bm.verts, Matrix.Translation(Vector((0.63, 0.0, -0.01))) @ Matrix.Rotation(math.radians(55.0), 4, "Y"))
    to_object("metal_rose", bm).parent = root
    return root


def slingshot():
    """The slingshot (the forward and back airs): a wooden Y, its fork forward from the fist, a red band strung between the tips."""
    root = empty("slingshot")
    bm = bmesh.new()
    tube(bm, [Vector((-0.08, 0.0, 0.0)), Vector((0.14, 0.0, 0.0))], [0.03, 0.028], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    for side in (-1.0, 1.0):
        tube(bm, [Vector((0.13, 0.0, 0.0)), Vector((0.2, 0.0, side * 0.06)), Vector((0.3, 0.0, side * 0.1))], [0.026, 0.024, 0.022],
             sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("wood_fork", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.3, 0.0, 0.1)), Vector((0.2, 0.0, 0.03)), Vector((0.16, 0.0, 0.0)), Vector((0.2, 0.0, -0.03)),
              Vector((0.3, 0.0, -0.1))], [0.012] * 5, sides=6, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("band_rubber", bm).parent = root
    # (Half as big again as a real one: at its true size it was lost in the fist at match size.)
    for c in root.children:
        c.scale = (1.6, 1.6, 1.6)
    return root


def firework():
    """The up smash's firework, held up: a thin stick from the fist to a red paper rocket with a gold cap, a twist of fuse at its foot."""
    root = empty("firework")
    bm = bmesh.new()
    tube(bm, [Vector((-0.1, 0.0, 0.0)), Vector((0.5, 0.0, 0.0))], [0.014, 0.012], sides=6, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("wood_stick", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.42, 0.0, 0.04)), Vector((0.74, 0.0, 0.04))], [0.06, 0.06], sides=12, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("band_tube", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.74, 0.0, 0.04)), Vector((0.8, 0.0, 0.04)), Vector((0.86, 0.0, 0.04))], [0.064, 0.04, 0.008], sides=12,
         axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("rim_cap", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.42, 0.0, 0.04)), Vector((0.38, 0.0, 0.07)), Vector((0.36, 0.0, 0.03))], [0.008] * 3, sides=5,
         axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("dark_fuse", bm).parent = root
    return root


def turnips():
    """The up and down airs' turnips: three pale bulbs with purple shoulders hung by their green tops from the fist."""
    root = empty("turnips")
    for k, (dz, dy, s) in enumerate(((0.0, 0.0, 1.0), (0.09, 0.03, 0.82), (-0.09, -0.02, 0.88))):
        c = Vector((0.3, dy, dz))
        bm = bmesh.new()
        vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.1 * s, 0.09 * s, 0.09 * s), u=12, v=8)
        # A pointed root end, away from the fist.
        for v in vs:
            if v.co.x > 0.0:
                v.co.x *= 1.0 + 0.8 * (v.co.x / (0.1 * s)) ** 2
        transform(vs, Matrix.Translation(c))
        to_object("pith_turnip%d" % k, bm).parent = root
        bm = bmesh.new()
        vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.05 * s, 0.092 * s, 0.092 * s), u=12, v=8)
        transform(vs, Matrix.Translation(c - Vector((0.05 * s, 0.0, 0.0))))
        to_object("petal_shoulder%d" % k, bm).parent = root
        bm = bmesh.new()
        tube(bm, [c - Vector((0.08 * s, 0.0, 0.0)), Vector((0.02, dy * 0.3, dz * 0.4))], [0.018, 0.014], sides=6,
             axis_hint=Vector((0.0, 1.0, 0.0)))
        for side in (-1.0, 1.0):
            petal(bm, Vector((0.02, 0.0, dz * 0.4)), Vector((-1.0, 0.0, side * 0.8 + dz)), 0.14, 0.06, 0.012)
        to_object("leaf_top%d" % k, bm).parent = root
    return root


def balloons():
    """The up special's balloons, worn on the back (hung on the spine, not the hand): two round balloons, red and blue, on strings from
    a little strap, floating above and behind the head. Built with the strap at the origin, up as +Z and behind the back as +Y."""
    root = empty("balloons")
    for k, (x, z, colour) in enumerate(((-0.22, 1.25, "band"), (0.2, 1.38, "glass"))):
        bm = bmesh.new()
        vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.24, 0.24, 0.28), u=16, v=10)
        for v in vs:
            if v.co.z < 0.0:
                v.co.x *= 1.0 + 0.25 * v.co.z / 0.28
                v.co.y *= 1.0 + 0.25 * v.co.z / 0.28
        transform(vs, Matrix.Translation(Vector((x, 0.12, z))))
        to_object("%s_balloon%d" % (colour, k), bm).parent = root
        bm = bmesh.new()
        tube(bm, [Vector((x, 0.12, z - 0.3)), Vector((x, 0.12, z - 0.36))], [0.03, 0.012], sides=6)
        tube(bm, [Vector((x, 0.12, z - 0.36)), Vector((x * 0.6, 0.08, z * 0.5)), Vector((0.0, 0.04, 0.05))], [0.008] * 3, sides=5,
             axis_hint=Vector((0.0, 1.0, 0.0)))
        to_object("dark_string%d" % k, bm).parent = root
    bm = bmesh.new()
    rounded_box(bm, Vector((0.0, 0.03, 0.0)), (0.16, 0.06, 0.1), bevel=0.3, segments=2)
    to_object("leather_strap", bm).parent = root
    return root


PROPS = {"umbrella": umbrella, "shovel": shovel, "axe": axe, "watering_can": watering_can, "slingshot": slingshot,
         "firework": firework, "turnips": turnips, "balloons": balloons}


# ---- Worn and held kit for other classes -------------------------------------------------------------------------------------------

def bow():
    """The Wanderer's bow, held in the free hand: a wooden bow standing across the fist (its limbs along the thumb's line, Blender
    +-Y), curving back to the tips, with a leather grip and a pale string from tip to tip behind it."""
    root = empty("bow")
    for sgn in (-1.0, 1.0):
        bm = bmesh.new()
        pts = [Vector((0.0, 0.0, 0.0)), Vector((-0.03, sgn * 0.18, 0.0)), Vector((-0.08, sgn * 0.36, 0.0)),
               Vector((-0.15, sgn * 0.5, 0.0)), Vector((-0.2, sgn * 0.58, 0.0)), Vector((-0.18, sgn * 0.64, 0.0))]
        tube(bm, pts, [0.034, 0.03, 0.026, 0.022, 0.018, 0.014], sides=8, axis_hint=Vector((0.0, 0.0, 1.0)))
        to_object("wood_limb%s" % ("u" if sgn > 0 else "d"), bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.0, -0.08, 0.0)), Vector((0.0, 0.08, 0.0))], [0.042, 0.042], sides=8, axis_hint=Vector((0.0, 0.0, 1.0)))
    to_object("leather_grip", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((-0.2, -0.6, 0.0)), Vector((-0.2, 0.6, 0.0))], [0.007, 0.007], sides=5, axis_hint=Vector((0.0, 0.0, 1.0)))
    to_object("pith_string", bm).parent = root
    return root


def water_tank():
    """Embers' water pack, worn on the back for the pump: a rounded tank with two red bands, a brass cap, and a short nozzle reaching
    forward over the shoulder. Built at the spine: up is +Z, forward (the way the fighter faces at rest) -Y, the back +Y."""
    root = empty("water_tank")
    bm = bmesh.new()
    lathe(bm, [(0.0, -0.22), (0.12, -0.22), (0.17, -0.16), (0.18, 0.12), (0.14, 0.2), (0.0, 0.22)], sides=16)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    transform(bm.verts, Matrix.Translation(Vector((0.0, 0.0, 0.0))))
    to_object("glass_tank", bm).parent = root
    bm = bmesh.new()
    for z in (-0.1, 0.08):
        tube(bm, [Vector((0.0, 0.0, z - 0.025)), Vector((0.0, 0.0, z + 0.025))], [0.186, 0.186], sides=16, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("band_hoops", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.0, 0.2), (0.06, 0.2), (0.06, 0.27), (0.0, 0.28)], sides=10)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    to_object("rim_cap", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.14, 0.0, 0.1)), Vector((0.24, -0.12, 0.16)), Vector((0.26, -0.32, 0.12)), Vector((0.26, -0.44, 0.06))],
         [0.04, 0.035, 0.03, 0.026], sides=8)
    to_object("dark_hose", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.26, -0.44, 0.06)), Vector((0.26, -0.56, 0.04))], [0.04, 0.03], sides=8)
    to_object("rim_nozzle", bm).parent = root
    return root


PROPS.update({"bow": bow, "water_tank": water_tank})
def export(root, path):
    bpy.ops.object.select_all(action="DESELECT")
    root.select_set(True)
    for c in root.children_recursive:
        c.select_set(True)
    bpy.ops.export_scene.gltf(filepath=os.path.abspath(path), use_selection=True, export_format="GLB", export_yup=True,
                              export_apply=True)


def preview(root, path):
    """A look from the match camera's side (slightly above, from -Y) and a three-quarter look, side by side in one picture."""
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "RANDOM"
    scene.display.shading.show_object_outline = True
    scene.render.resolution_x = 480
    scene.render.resolution_y = 480
    lo, hi = Vector((1e9,) * 3), Vector((-1e9,) * 3)
    for c in root.children_recursive:
        if c.type != "MESH":
            continue
        for v in c.data.vertices:
            w = c.matrix_world @ v.co
            lo = Vector(map(min, lo, w))
            hi = Vector(map(max, hi, w))
    target = (lo + hi) * 0.5
    size = max(0.6, (hi - lo).length * 1.1)
    for tag, offset in (("side", Vector((0.0, -3.0, 0.5))), ("three", Vector((1.6, -2.4, 1.2)))):
        cam_data = bpy.data.cameras.new("cam")
        cam_data.type = "ORTHO"
        cam_data.ortho_scale = size
        cam = bpy.data.objects.new("cam", cam_data)
        scene.collection.objects.link(cam)
        scene.camera = cam
        cam.location = target + offset * max(1.0, size)
        cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = path.replace(".png", "_%s.png" % tag)
        bpy.ops.render.render(write_still=True)
        bpy.data.objects.remove(cam)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    only = opts["only"].split(",") if "only" in opts else list(PROPS)
    os.makedirs(OUT, exist_ok=True)
    for name in only:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        root = PROPS[name]()
        export(root, os.path.join(OUT, name + ".glb"))
        tris = sum(sum(len(p.vertices) - 2 for p in c.data.polygons) for c in root.children_recursive if c.type == "MESH")
        print("prop %s: %d triangles" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
