"""The things fighters throw, shoot and leave lying in a match, modelled by script in the chunky, rounded style of the weapons and gear
(make_weapons.py, make_gear.py). The art director's props pass, 2026-11-04: until now every shot without its own look was drawn as one
generic purple bolt, and four item kinds (the Scamp's banana peel, the Settler's sapling, tree and wood chip) were not drawn at all.
Each piece is a few shaped parts named by what they are made of, so the game colours them (godot/scripts/match_props.gd `PALETTES`):
  cork*  wood*  bark*  dark*  band*  leather*  metal*  peel*  pith*  soil*  leaf*  clay*  petal*  centre*  ball*  hole*  stone*
  paint*  glass*  rim*
Built in the game's space: Blender X is the game's X (a shot's forward, the way it flies), Blender Z the game's up, Blender -Y toward the
camera. Shots are centred on the origin (the thrown flowerpot too); things that stand or lie on the floor (the peel, the sapling, the tree,
the chip) have their base on Z = 0.

Run:  blender --background --python art/blender/make_match_props.py [-- --only cork,tree] [--preview <folder>]
Writes godot/models/match_props/<name>.glb (then `Godot --headless --path godot --import`). Keep each within the prop budget
(docs/ART_WORKFLOW.md: 3,000 triangles).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

OUT = "godot/models/match_props"


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


# ---- The Scamp (the Popgun class) --------------------------------------------------------------------------------------------------

def cork():
    """The popgun's shot: a fat cork flying point first, a red band round its back (the gun's barrel red), a little wider at the back
    as a cork is. About 0.46 long, so it reads at the match camera beside the hitbox's 0.3 radius."""
    root = empty("cork")
    bm = bmesh.new()
    prof = [(0.0, -0.23), (0.11, -0.23), (0.15, -0.205), (0.158, -0.16), (0.152, 0.0), (0.142, 0.16), (0.13, 0.2), (0.09, 0.23), (0.0, 0.23)]
    rings = lathe(bm, prof, sides=16, cap_bottom=False, cap_top=False)
    transform(bm.verts, Matrix.Rotation(math.radians(90.0), 4, "Y"))
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    to_object("cork_body", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((-0.13, 0.0, 0.0)), Vector((-0.07, 0.0, 0.0))], [0.166, 0.163], sides=16)
    to_object("band_ring", bm).parent = root
    return root


def barrels():
    """The Scamp's rocket barrels after they are knocked off: two short wooden barrels side by side, each with two dark hoops and a
    brass nozzle under it, joined by a leather strap across their middles."""
    root = empty("barrels")
    for side in (-1.0, 1.0):
        x = 0.21 * side
        bm = bmesh.new()
        prof = [(0.0, -0.28), (0.13, -0.28), (0.16, -0.2), (0.18, 0.0), (0.16, 0.2), (0.13, 0.28), (0.0, 0.28)]
        lathe(bm, prof, sides=14)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
        bmesh.ops.translate(bm, vec=Vector((x, 0.0, 0.0)), verts=bm.verts)
        to_object("wood_barrel_%s" % ("l" if side < 0 else "r"), bm).parent = root
        bm = bmesh.new()
        for z, r in ((-0.17, 0.17), (0.17, 0.17)):
            tube(bm, [Vector((x, 0.0, z - 0.025)), Vector((x, 0.0, z + 0.025))], [r + 0.012, r + 0.012], sides=14)
        to_object("dark_hoops_%s" % ("l" if side < 0 else "r"), bm).parent = root
        bm = bmesh.new()
        lathe(bm, [(0.0, -0.4), (0.075, -0.4), (0.06, -0.33), (0.07, -0.28), (0.0, -0.28)], sides=10)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
        bmesh.ops.translate(bm, vec=Vector((x, 0.0, 0.0)), verts=bm.verts)
        to_object("metal_nozzle_%s" % ("l" if side < 0 else "r"), bm).parent = root
    # The harness: a strap round the front of both barrels at their middles.
    bm = bmesh.new()
    ribbon(bm, [Vector((x, -0.17 - 0.03 * (1.0 - abs(x) / 0.4), 0.0)) for x in (-0.36, -0.2, 0.0, 0.2, 0.36)],
           [0.09] * 5, Vector((0.0, 0.0, 1.0)), 0.03)
    to_object("leather_strap", bm).parent = root
    return root


def banana_peel():
    """A peel to slip on, as a cartoon peel stands: the banana's top end upright with a brown stalk tip, and three broad flaps split
    from it hanging down to the floor and curling out at their ends, yellow outside and pale inside. About 0.8 across and 0.47 tall.
    The flaps are turned for the side-on match camera: the two in front show their faces as they fall to either side, the one behind
    shows its pale inside past the top."""
    root = empty("banana_peel")
    bm = bmesh.new()
    tube(bm, [Vector((0.0, 0.0, 0.12)), Vector((0.0, 0.0, 0.24)), Vector((0.02, 0.0, 0.34)), Vector((0.05, 0.0, 0.42))],
         [0.085, 0.08, 0.065, 0.045], sides=14, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("peel_top", bm).parent = root
    for k, (ang, tilt) in enumerate(((-155.0, 0.55), (-25.0, 0.55), (90.0, 0.0))):
        a = math.radians(ang)
        d = Vector((math.cos(a), math.sin(a) * 0.8, 0.0)).normalized()
        across = Vector((-d.y, d.x, 0.0))
        # (Tipped toward upright by `tilt`, so a flap falling to the side still shows its face to the camera.)
        across = (across + Vector((0.0, 0.0, tilt * (1.0 if d.x > 0 else -1.0)))).normalized()
        path = [(0.07, 0.26), (0.14, 0.22), (0.21, 0.14), (0.28, 0.06), (0.34, 0.035), (0.39, 0.055), (0.42, 0.1)]
        pts = [d * r + Vector((0.0, 0.0, z)) for r, z in path]
        widths = [0.15, 0.19, 0.2, 0.19, 0.16, 0.11, 0.05]
        bm = bmesh.new()
        ribbon(bm, pts, widths, across, 0.04)
        to_object("peel_flap%d" % k, bm).parent = root
        bm = bmesh.new()
        inner = [p + Vector((0.0, 0.0, 0.026)) - d * 0.014 for p in pts[:-1]]
        ribbon(bm, inner, [w * 0.76 for w in widths[:-1]], across, 0.012)
        to_object("pith_flap%d" % k, bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.045, 0.0, 0.41)), Vector((0.07, 0.0, 0.47))], [0.034, 0.026], sides=8)
    to_object("dark_tip", bm).parent = root
    return root


# ---- The Settler (the Pockets class) -----------------------------------------------------------------------------------------------

def sapling():
    """A planted sapling: a low mound of dark soil with a thin stem and two round leaves turned to the camera."""
    root = empty("sapling")
    bm = bmesh.new()
    vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.26, 0.2, 0.1), u=16, v=8)
    bmesh.ops.delete(bm, geom=[v for v in vs if v.co.z < -0.001], context="VERTS")
    bmesh.ops.holes_fill(bm, edges=bm.edges, sides=0)
    to_object("soil_mound", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.0, 0.0, 0.04)), Vector((0.01, 0.0, 0.16)), Vector((-0.005, 0.0, 0.28)), Vector((0.0, 0.0, 0.34))],
         [0.025, 0.022, 0.019, 0.016], sides=8)
    to_object("leaf_stem", bm).parent = root
    bm = bmesh.new()
    for side in (-1.0, 1.0):
        d = Vector((side, 0.0, 0.55))
        vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.11, 0.025, 0.065), u=12, v=8)
        m = Matrix.Translation(Vector((side * 0.085, -0.01, 0.37))) @ Matrix.Rotation(math.radians(-side * 28.0), 4, "Y")
        transform(vs, m)
    to_object("leaf_pair", bm).parent = root
    return root


TREE_HEIGHT = 3.3


def tree():
    """The Settler's grown tree, about 3.4 tall (the falling tree's capsule is 3.2 long): a chunky, nearly straight trunk with a root
    flare, under one big round crown scalloped by leafy lobes round its edge, darker behind and lighter in front."""
    root = empty("tree")
    bm = bmesh.new()
    pts = [Vector((0.0, 0.0, 0.0)), Vector((0.0, 0.0, 0.3)), Vector((0.02, 0.0, 0.8)), Vector((0.03, 0.0, 1.3)), Vector((0.01, 0.0, 1.8)),
           Vector((0.0, 0.0, 2.2))]
    tube(bm, pts, [0.36, 0.27, 0.23, 0.21, 0.2, 0.19], sides=14)
    # The root flare: three low roots running out into the ground.
    for ang in (-20.0, 110.0, 220.0):
        a = math.radians(ang)
        d = Vector((math.cos(a), math.sin(a) * 0.6, 0.0))
        tube(bm, [Vector((0.0, 0.0, 0.22)) + d * 0.12, d * 0.32 + Vector((0.0, 0.0, 0.06)), d * 0.44 + Vector((0.0, 0.0, 0.0))],
             [0.11, 0.07, 0.03], sides=8)
    to_object("bark_trunk", bm).parent = root
    bm = bmesh.new()
    blob(bm, Vector((0.0, 0.25, 2.62)), (1.02, 0.6, 0.8), u=20, v=12)
    to_object("leafdark_crown", bm).parent = root
    bm = bmesh.new()
    blob(bm, Vector((0.0, -0.05, 2.6)), (0.95, 0.62, 0.76), u=20, v=12)
    for ang, rad in ((0.0, 0.42), (40.0, 0.44), (80.0, 0.46), (120.0, 0.44), (160.0, 0.42), (205.0, 0.36), (335.0, 0.36)):
        a = math.radians(ang)
        c = Vector((math.cos(a) * 0.9, -0.12, 2.6 + math.sin(a) * 0.7))
        blob(bm, c, (rad, rad * 0.8, rad * 0.9), u=14, v=9)
    to_object("leaf_crown", bm).parent = root
    return root


def wood_chip():
    """A chopped length of the tree, lying on its side: bark round it and pale cut wood at both ends."""
    root = empty("wood_chip")
    bm = bmesh.new()
    tube(bm, [Vector((-0.19, 0.0, 0.12)), Vector((0.19, 0.0, 0.12))], [0.12, 0.115], sides=12)
    to_object("bark_log", bm).parent = root
    bm = bmesh.new()
    for x0, x1, r in ((-0.205, -0.185, 0.1), (0.185, 0.205, 0.097)):
        tube(bm, [Vector((x0, 0.0, 0.12)), Vector((x1, 0.0, 0.12))], [r, r], sides=12)
    to_object("wood_ends", bm).parent = root
    return root


FLOWERPOT_MIDDLE = 0.34


def flowerpot():
    """The dash attack's flowerpot: a terracotta pot with a rolled rim, dark soil, and a pink flower on a short stem facing the camera."""
    root = empty("flowerpot")
    bm = bmesh.new()
    prof = [(0.0, 0.0), (0.15, 0.0), (0.17, 0.02), (0.2, 0.24), (0.215, 0.26), (0.24, 0.27), (0.245, 0.32), (0.235, 0.34), (0.2, 0.34),
            (0.19, 0.3)]
    lathe(bm, prof, sides=16, cap_top=False)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    to_object("clay_pot", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.0, 0.27), (0.195, 0.27), (0.19, 0.3), (0.0, 0.31)], sides=16, cap_bottom=False, cap_top=False)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    to_object("soil_top", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.0, 0.0, 0.3)), Vector((0.02, 0.0, 0.45)), Vector((0.0, 0.0, 0.58))], [0.022, 0.02, 0.018], sides=8)
    for side in (-1.0, 1.0):
        petal(bm, Vector((0.01, 0.0, 0.4)), Vector((side, -0.2, 0.45)), 0.16, 0.08, 0.016)
    to_object("leaf_stem", bm).parent = root
    bm = bmesh.new()
    for k in range(5):
        a = math.tau * k / 5 + math.pi / 2
        vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.075, 0.02, 0.05), u=10, v=6)
        m = Matrix.Translation(Vector((0.0, -0.03, 0.62))) @ Matrix.Rotation(a, 4, "Y") @ Matrix.Translation(Vector((0.07, 0.0, 0.0)))
        transform(vs, m)
    to_object("petal_flower", bm).parent = root
    bm = bmesh.new()
    blob(bm, Vector((0.0, -0.05, 0.62)), (0.045, 0.03, 0.045), u=10, v=6)
    to_object("centre_flower", bm).parent = root
    # (Thrown, it turns about its middle: the whole pot is lowered so its middle is on the origin, the shot's point.)
    for c in root.children:
        c.location.z -= FLOWERPOT_MIDDLE
    return root


def bowling_ball():
    """The forward smash's bowling ball: deep blue with three finger holes up toward its back, radius 0.42 (its hitbox grows from
    0.5)."""
    root = empty("bowling_ball")
    bm = bmesh.new()
    blob(bm, Vector((0.0, 0.0, 0.0)), (0.42, 0.42, 0.42), u=24, v=16)
    to_object("ball_body", bm).parent = root
    bm = bmesh.new()
    # (Up and round toward the back, seen foreshortened from the side-on camera: three holes square to the camera under a highlight
    # read as a face.)
    for ax, az, r in ((0.55, 1.02, 0.05), (0.78, 1.12, 0.045), (0.8, 0.86, 0.058)):
        d = Vector((math.sin(ax), -math.cos(ax) * math.cos(az), math.sin(az))).normalized()
        c = d * 0.405
        vs = blob(bm, Vector((0.0, 0.0, 0.0)), (r, r, 0.035), u=10, v=6)
        rot = Vector((0.0, 0.0, 1.0)).rotation_difference(d).to_matrix().to_4x4()
        transform(vs, Matrix.Translation(c) @ rot)
    to_object("hole_fingers", bm).parent = root
    return root


def pellet():
    """A slingshot pellet: a small, slightly flattened round stone."""
    root = empty("pellet")
    bm = bmesh.new()
    blob(bm, Vector((0.0, 0.0, 0.0)), (0.12, 0.11, 0.1), u=12, v=8)
    to_object("stone_pellet", bm).parent = root
    return root


def rocket():
    """The side special's ridden rocket, as a toy: a cream body with a red nose and fins, a round window on the camera's side with a
    brass rim, and a dark nozzle at the back. About 1.05 long, flying along +X."""
    root = empty("rocket")
    bm = bmesh.new()
    tube(bm, [Vector((-0.38, 0.0, 0.0)), Vector((-0.3, 0.0, 0.0)), Vector((0.0, 0.0, 0.0)), Vector((0.24, 0.0, 0.0))],
         [0.16, 0.185, 0.19, 0.18], sides=16, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("paint_body", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((0.235, 0.0, 0.0)), Vector((0.36, 0.0, 0.0)), Vector((0.47, 0.0, 0.0)), Vector((0.55, 0.0, 0.0)), Vector((0.6, 0.0, 0.0))]
    tube(bm, pts, [0.182, 0.16, 0.11, 0.06, 0.012], sides=16, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("band_nose", bm).parent = root
    for k in range(3):
        # Three fins round the back, one straight up, two down and to either side (one toward the camera).
        a = math.radians(90.0 + 120.0 * k)
        d = Vector((0.0, -math.sin(a), math.cos(a)))
        m = Matrix.Translation(Vector((-0.27, 0.0, 0.0)) + d * 0.25) @ Vector((0.0, 0.0, 1.0)).rotation_difference(d).to_matrix().to_4x4()
        box_part("band_fin%d" % k, (0.26, 0.045, 0.2), m, parent=root)
    bm = bmesh.new()
    vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.075, 0.03, 0.075), u=12, v=8)
    transform(vs, Matrix.Translation(Vector((0.05, -0.185, 0.02))))
    to_object("glass_window", bm).parent = root
    bm = bmesh.new()
    rim = []
    for i in range(16):
        a = math.tau * i / 16
        rim.append(Vector((0.05 + 0.088 * math.cos(a), -0.178, 0.02 + 0.088 * math.sin(a))))
    rim.append(rim[0])
    # (A closed ring of short tubes round the window.)
    for p0, p1 in zip(rim, rim[1:]):
        tube(bm, [p0, p1], [0.018, 0.018], sides=6, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("rim_window", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.0, 0.0), (0.12, 0.0), (0.14, 0.08), (0.11, 0.1), (0.0, 0.1)], sides=12)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    transform(bm.verts, Matrix.Translation(Vector((-0.48, 0.0, 0.0))) @ Matrix.Rotation(math.radians(90.0), 4, "Y"))
    to_object("dark_nozzle", bm).parent = root
    return root


# ---- The Wanderer (the Buckler class) ----------------------------------------------------------------------------------------------

def arrow():
    """An arrow, 1.12 long, pointing +X and centred on the origin: a wooden shaft, a leaf-shaped steel head flat to the camera, two red
    paper vanes above and below the shaft and a dark nock. Shared by the flying shot and the arrow item lodged in the floor."""
    root = empty("arrow")
    bm = bmesh.new()
    tube(bm, [Vector((-0.54, 0.0, 0.0)), Vector((0.43, 0.0, 0.0))], [0.028, 0.026], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("wood_shaft", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((0.4, 0.0, 0.0)), Vector((0.45, 0.0, 0.0)), Vector((0.5, 0.0, 0.0)), Vector((0.56, 0.0, 0.0)), Vector((0.6, 0.0, 0.0))]
    # Flat across Y (to the camera): the hint axis is Y, so `flat` thins it toward the camera.
    tube(bm, pts, [0.036, 0.066, 0.06, 0.034, 0.006], sides=8, flat=0.35, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("metal_head", bm).parent = root
    for side in (-1.0, 1.0):
        m = Matrix.Translation(Vector((-0.42, 0.0, side * 0.055))) @ Matrix.Rotation(math.radians(side * -14.0), 4, "Y")
        box_part("band_vane%s" % ("u" if side > 0 else "d"), (0.2, 0.014, 0.075), m, bevel=0.25, segments=1, parent=root)
    bm = bmesh.new()
    tube(bm, [Vector((-0.58, 0.0, 0.0)), Vector((-0.53, 0.0, 0.0))], [0.034, 0.034], sides=8, axis_hint=Vector((0.0, 1.0, 0.0)))
    to_object("dark_nock", bm).parent = root
    return root


def boomerang():
    """A flat wooden boomerang square to the camera (it spins about the camera's axis in flight): one bent blade, broad at the elbow and
    rounded at both tips, with a red band painted across the elbow."""
    root = empty("boomerang")
    across_y = Vector((0.0, 1.0, 0.0))
    n = 12
    pts, widths = [], []
    for i in range(n + 1):
        t = i / n * 2.0 - 1.0
        # A V of about 100 degrees with a rounded elbow: x across, z up.
        pts.append(Vector((t * 0.42, 0.0, -0.3 * (1.0 - abs(t) ** 1.6) + 0.12)))
        widths.append(0.07 + 0.07 * (1.0 - abs(t)) ** 0.8)
    # The strip's width runs in the X-Z plane, square to its path; built as quads across a per-point side vector.
    left, right = [], []
    for i, p in enumerate(pts):
        a = pts[min(i + 1, n)] - pts[max(i - 1, 0)]
        a.normalize()
        side = Vector((-a.z, 0.0, a.x))
        left.append(p - side * widths[i] * 0.5)
        right.append(p + side * widths[i] * 0.5)
    bm = bmesh.new()
    lv = [bm.verts.new(v) for v in left]
    rv = [bm.verts.new(v) for v in right]
    faces = [bm.faces.new((lv[i], rv[i], rv[i + 1], lv[i + 1])) for i in range(n)]
    for v in bm.verts:
        v.co.y -= 0.03
    solidify(bm, faces, 0.06)
    to_object("wood_boomerang", bm, sharp=70.0).parent = root
    bm = bmesh.new()
    i0, i1 = n // 2 - 1, n // 2 + 1
    lv = [bm.verts.new(left[i] + (left[i] - pts[i]) * 0.06) for i in range(i0, i1 + 1)]
    rv = [bm.verts.new(right[i] + (right[i] - pts[i]) * 0.06) for i in range(i0, i1 + 1)]
    faces = [bm.faces.new((lv[i], rv[i], rv[i + 1], lv[i + 1])) for i in range(i1 - i0)]
    for v in bm.verts:
        v.co.y -= 0.036
    solidify(bm, faces, 0.072)
    to_object("band_elbow", bm, sharp=70.0).parent = root
    return root


def bomb():
    """The Wanderer's bomb (item kind 2), in place of the code-built ball: a round dark body with a steel collar on top and a rope fuse
    curling out of it, its tip where the game's spark sits (0.08 across, 0.64 up from the middle). Centred on the ball's middle, so it
    spins about it when thrown."""
    root = empty("bomb")
    bm = bmesh.new()
    vs = blob(bm, Vector((0.0, 0.0, 0.0)), (0.36, 0.36, 0.35), u=20, v=14)
    to_object("ball_body", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.0, 0.3), (0.12, 0.3), (0.13, 0.34), (0.11, 0.39), (0.08, 0.4), (0.0, 0.4)], sides=14)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    to_object("metal_cap", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((0.0, 0.0, 0.38)), Vector((0.02, 0.0, 0.45)), Vector((0.06, 0.0, 0.5)), Vector((0.08, 0.0, 0.56)),
              Vector((0.08, 0.0, 0.62))], [0.026, 0.024, 0.022, 0.02, 0.018], sides=7)
    to_object("pith_fuse", bm).parent = root
    return root


PROPS = {"cork": cork, "barrels": barrels, "banana_peel": banana_peel, "sapling": sapling, "tree": tree, "wood_chip": wood_chip,
         "flowerpot": flowerpot, "bowling_ball": bowling_ball, "pellet": pellet, "rocket": rocket, "arrow": arrow, "boomerang": boomerang, "bomb": bomb}


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
