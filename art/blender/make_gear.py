"""The fighters' accessories modelled by script (2026-10-18, the owner: the hats built from game primitives read cheap beside the
reference cast's; modelled like the stage props, over time). Each piece is a few lathed and shaped parts with real profiles (a crown that
domes and sags, a brim that rolls, seams pressed in), named by what they are made of, so the game colours each fighter's piece:
  fabric*  the outfit's accent cloth          canvas*  white cloth
  leather* brown leather                      metal*   brass or steel
  glass*   lens glass                         straw*   woven straw (the game lays the straw weave on it)
  hair*    the fighter's hair (brow) colour
Built in the head's design space (the game's hat pivot space before it subtracts the pivot): a head of radius 0.8 centred 1.42 up, so
its crown is at 2.22. Blender z up (exported y up); the face looks along -Y in Blender (+Z in the game).

Run:  blender --background --python art/blender/make_gear.py [-- --only sailor_cap,aviator_cap,straw_hat] [--preview <folder>]
Writes godot/models/gear/<name>.glb (then `Godot --headless --path godot --import`).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

OUT = "godot/models/gear"
HEAD_C = Vector((0.0, 0.0, 1.42))
HEAD_R = 0.8


def material(name):
    m = bpy.data.materials.get(name)
    if m is None:
        m = bpy.data.materials.new(name)
    return m


def to_object(name, bm, smooth=True, sharp=40.0):
    me = bpy.data.meshes.new(name)
    # (Faces turned outward whichever way a part was built: the aviator's shell, built from the crown down, faced in and was culled.)
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


def lathe(bm, profile, sides=40, wobble=None):
    """A surface of revolution round z from `profile` [(radius, z), ...] (bottom to top or in any order along the outline), each ring
    optionally pushed by `wobble(angle, k)` (a radius multiplier, k the profile index). Returns the rings of verts."""
    rings = []
    for k, (r, z) in enumerate(profile):
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            rr = r * (wobble(a, k) if wobble else 1.0)
            ring.append(bm.verts.new((rr * math.cos(a), rr * math.sin(a), z)))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    return rings


def cap_ring(bm, ring, z):
    """Closes a ring with a fan to a centre point at height z."""
    c = bm.verts.new((0.0, 0.0, z))
    n = len(ring)
    for i in range(n):
        bm.faces.new((ring[i], ring[(i + 1) % n], c))


def head_r(z):
    """The head's radius at height z (0 above its top)."""
    d = z - HEAD_C.z
    return math.sqrt(max(0.0, HEAD_R * HEAD_R - d * d))


def empty(name):
    e = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(e)
    return e


# ---- Sailor cap -------------------------------------------------------------------------------------------------------------------------

def sailor_cap():
    """A soft white sailor's cap: a crown that rises from the head, swells a little and domes over with a soft sag in the middle (cloth
    over no frame), a brim rolled up all round as a thick soft lip, a pressed seam round the crown's top, and a band of the outfit's
    colour just above the brim."""
    root = empty("sailor_cap")
    base = 2.02
    # The crown: from just inside the head, out and up, swelling, then over the top with a sag.
    prof = [(head_r(base) - 0.02, base - 0.04), (0.54, base + 0.02), (0.56, base + 0.12), (0.57, base + 0.24), (0.55, base + 0.31),
            (0.5, base + 0.355), (0.4, base + 0.37), (0.26, base + 0.36), (0.12, base + 0.345)]
    bm = bmesh.new()
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + (0.015 * math.sin(a * 3.0) if 2 <= k <= 4 else 0.0))
    cap_ring(bm, rings[-1], base + 0.34)
    # The seam round the top: pressed in at the shoulder.
    for v in rings[5]:
        v.co.x *= 0.985
        v.co.y *= 0.985
        v.co.z -= 0.006
    o = to_object("canvas_crown", bm)
    o.parent = root
    # The rolled brim: a thick soft lip turned up, its outline a little uneven.
    prof = [(0.5, base - 0.02), (0.6, base - 0.04), (0.68, base - 0.01), (0.7, base + 0.05), (0.66, base + 0.1), (0.58, base + 0.08),
            (0.55, base + 0.04)]
    bm = bmesh.new()
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + 0.012 * math.sin(a * 2.0 + 0.7))
    n = len(rings[0])
    for i in range(n):
        bm.faces.new((rings[-1][i], rings[-1][(i + 1) % n], rings[0][(i + 1) % n], rings[0][i]))
    o = to_object("canvas_brim", bm, sharp=60.0)
    o.parent = root
    # The band, just above the brim.
    prof = [(0.575, base + 0.09), (0.585, base + 0.11), (0.585, base + 0.17), (0.575, base + 0.19)]
    bm = bmesh.new()
    lathe(bm, prof)
    o = to_object("fabric_band", bm, sharp=60.0)
    o.parent = root
    return root


# ---- Aviator cap -----------------------------------------------------------------------------------------------------------------------

def aviator_cap():
    """A leather flying cap: a close shell over the crown and back of the head with a seam down its middle and a stitched rim, two ear
    flaps hanging at the sides, a strap under them, and goggles pushed up on the forehead (two brass rims with glass, on a leather band)."""
    root = empty("aviator_cap")
    # The shell: the head's own sphere, a little out, from the brow round the back.
    bm = bmesh.new()
    seg_u, seg_v = 40, 16
    rows = []
    for j in range(seg_v + 1):
        phi = math.radians(10.0 + 78.0 * j / seg_v)   # from the crown (10 degrees) down
        row = []
        for i in range(seg_u):
            a = math.tau * i / seg_u
            front = max(0.0, -math.sin(a))             # 1 at the face (-y)
            # Lower at the back and sides, up over the brow in front.
            low = math.radians(100.0) - math.radians(28.0) * front ** 1.5
            ph = min(phi, low) if j == seg_v else phi * (low / math.radians(88.0))
            # (Well clear of the head: the fighter's head is an oval the design sphere only approximates, and 0.045 off it sank in.)
            # (On the built cap's own ellipsoid, centred high on the head, so it clears the face shell and the oval head everywhere.)
            r = 0.87
            x = r * math.sin(ph) * math.cos(a)
            y = r * math.sin(ph) * math.sin(a)
            z = 1.74 + 0.6 * math.cos(ph)
            # The seam down the middle of the crown.
            if abs(x) < 0.03:
                x *= 0.6
                z -= 0.008
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.74 + 0.6))
    for i in range(seg_u):
        bm.faces.new((rows[0][i], rows[0][(i + 1) % seg_u], top))
    for j in range(seg_v):
        for i in range(seg_u):
            k = (i + 1) % seg_u
            bm.faces.new((rows[j][i], rows[j][k], rows[j + 1][k], rows[j + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=-0.03)
    o = to_object("leather_shell", bm)
    o.parent = root
    # Ear flaps: rounded tabs at the sides, curving with the head.
    for sx in (-1.0, 1.0):
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=18, v_segments=12, radius=1.0)
        for v in ret["verts"]:
            v.co = Vector((v.co.x * 0.09, v.co.y * 0.24, v.co.z * 0.3))
        bmesh.ops.translate(bm, vec=Vector((sx * 0.83, 0.04, 1.48)), verts=ret["verts"])
        o = to_object("leather_flap" + ("L" if sx < 0 else "R"), bm)
        o.parent = root
    # The goggles' band round the brow, and the goggles on it.
    bm = bmesh.new()
    lathe(bm, [(head_r(1.86) + 0.14, 1.86), (head_r(1.95) + 0.14, 1.95)])
    o = to_object("leather_band", bm, sharp=60.0)
    o.parent = root
    for sx in (-1.0, 1.0):
        centre = Vector((sx * 0.25, -(head_r(1.92) + 0.16), 1.92))
        bm = bmesh.new()
        tor = []
        sides, minor = 28, 10
        rows = []
        for i in range(sides):
            a = math.tau * i / sides
            row = []
            for j in range(minor):
                b = math.tau * j / minor
                rr = 0.15 + 0.045 * math.cos(b)
                row.append(bm.verts.new(centre + Vector((rr * math.cos(a), -0.045 * math.sin(b) - 0.02, rr * math.sin(a)))))
            rows.append(row)
        for i in range(sides):
            for j in range(minor):
                bm.faces.new((rows[i][j], rows[(i + 1) % sides][j], rows[(i + 1) % sides][(j + 1) % minor], rows[i][(j + 1) % minor]))
        o = to_object("metal_rim" + ("L" if sx < 0 else "R"), bm)
        o.parent = root
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=20, v_segments=10, radius=1.0)
        for v in ret["verts"]:
            v.co = centre + Vector((v.co.x * 0.15, v.co.y * 0.03 - 0.03, v.co.z * 0.15))
        o = to_object("glass_lens" + ("L" if sx < 0 else "R"), bm)
        o.parent = root
    return root


# ---- Straw hat --------------------------------------------------------------------------------------------------------------------------

def straw_hat():
    """A boater-ish straw hat: a wide brim that waves gently and dips a little at front and back, rolled at its edge; a crown that rises
    straight with a soft pinch at the top; a band of the outfit's colour round its foot with a little bow at the side."""
    root = empty("straw_hat")
    base = 2.0
    wave = lambda a: 0.025 * math.cos(2.0 * a) + 0.012 * math.sin(5.0 * a + 0.4)
    bm = bmesh.new()
    sides = 48
    prof_r = [0.5, 0.8, 1.05, 1.17, 1.2, 1.16, 1.06]
    prof_z = [0.04, 0.035, 0.02, 0.0, 0.03, 0.06, 0.05]
    rings = []
    for k, (r, z) in enumerate(zip(prof_r, prof_z)):
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            dz = z + wave(a) * min(1.0, r / 1.0) - (0.04 if k >= 3 else 0.0) * abs(math.sin(a))
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), base + dz)))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.03)
    o = to_object("straw_brim", bm, sharp=70.0)
    o.parent = root
    bm = bmesh.new()
    prof = [(0.5, base + 0.03), (0.6, base + 0.05), (0.6, base + 0.3), (0.58, base + 0.37), (0.48, base + 0.4), (0.3, base + 0.37),
            (0.12, base + 0.385)]
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + (0.02 * math.cos(2.0 * a) if k >= 3 else 0.0))
    cap_ring(bm, rings[-1], base + 0.39)
    o = to_object("straw_crown", bm)
    o.parent = root
    bm = bmesh.new()
    lathe(bm, [(0.612, base + 0.05), (0.618, base + 0.07), (0.618, base + 0.15), (0.612, base + 0.17)])
    o = to_object("fabric_band", bm, sharp=60.0)
    o.parent = root
    # The bow at the side: two small loops.
    for side in (-1.0, 1.0):
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=14, v_segments=8, radius=1.0)
        for v in ret["verts"]:
            v.co = Vector((0.63 + v.co.x * 0.02, side * 0.07 + v.co.y * 0.075, base + 0.11 + v.co.z * 0.045))
        o = to_object("fabric_bow" + ("L" if side < 0 else "R"), bm)
        o.parent = root
    return root


# ---- Plumed helm -----------------------------------------------------------------------------------------------------------------------

def _bowl_phi_low(a, deg_back=100.0, deg_front=30.0):
    front = max(0.0, -math.sin(a))
    return math.radians(deg_back) - math.radians(deg_front) * front ** 1.4


# (Taller than the caps' shell, 2026-10-18: on the wide head a low bowl read as a pot lid.)
HELM_H = 0.82


def plumed_helm():
    """A knight's open helm: a steel bowl over the crown with a raised ridge from brow to nape, a brass brow band with rivets, short
    cheek guards curving down beside the face, and a tall plume of feather blades in the outfit's colour sweeping back off a brass
    socket on the crest."""
    root = empty("plumed_helm")
    bm = bmesh.new()
    seg_u, seg_v = 40, 14
    rows = []
    for j in range(seg_v + 1):
        phi = math.radians(8.0 + 92.0 * j / seg_v)
        row = []
        for i in range(seg_u):
            a = math.tau * i / seg_u
            ph = phi * (_bowl_phi_low(a) / math.radians(100.0))
            x = 0.89 * math.sin(ph) * math.cos(a)
            y = 0.89 * math.sin(ph) * math.sin(a)
            z = 1.74 + HELM_H * math.cos(ph)
            if abs(x) < 0.06:
                z += 0.035 * (1.0 - abs(x) / 0.06) * math.sin(ph)
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.74 + HELM_H + 0.03))
    for i in range(seg_u):
        bm.faces.new((rows[0][i], rows[0][(i + 1) % seg_u], top))
    for j in range(seg_v):
        for i in range(seg_u):
            k = (i + 1) % seg_u
            bm.faces.new((rows[j][i], rows[j][k], rows[j + 1][k], rows[j + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=-0.035)
    o = to_object("metal_bowl", bm, sharp=50.0)
    o.parent = root
    # The brow band along the bowl's lower edge.
    bm = bmesh.new()
    sides = 40
    rings = []
    for dr, dz in [(0.02, -0.06), (0.035, -0.03), (0.035, 0.03), (0.02, 0.06)]:
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            ph = _bowl_phi_low(a) - math.radians(8.0)
            r = 0.89 * math.sin(ph) + dr
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), 1.74 + HELM_H * math.cos(ph) + dz)))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    o = to_object("brass_band", bm, sharp=60.0)
    o.parent = root
    for i in range(10):
        a = math.tau * (i + 0.5) / 10.0
        ph = _bowl_phi_low(a) - math.radians(8.0)
        p = Vector((0.93 * math.sin(ph) * math.cos(a), 0.93 * math.sin(ph) * math.sin(a), 1.74 + HELM_H * math.cos(ph)))
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=10, v_segments=6, radius=0.028)
        bmesh.ops.translate(bm, vec=p, verts=bm.verts)
        o = to_object("brass_rivet%d" % i, bm)
        o.parent = root
    # Cheek guards beside the face.
    for sx in (-1.0, 1.0):
        bm = bmesh.new()
        rows = []
        for j in range(5):
            t = j / 4.0
            row = []
            for i in range(6):
                u = i / 5.0
                a = math.radians(-90.0 + sx * (52.0 + 30.0 * u))
                r = 0.9 - 0.04 * t
                row.append(bm.verts.new((r * math.cos(a), r * math.sin(a), 1.62 - 0.36 * t - 0.06 * u * t)))
            rows.append(row)
        for j in range(4):
            for i in range(5):
                bm.faces.new((rows[j][i], rows[j][i + 1], rows[j + 1][i + 1], rows[j + 1][i]))
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.03)
        o = to_object("metal_cheek" + ("L" if sx < 0 else "R"), bm, sharp=50.0)
        o.parent = root
    # The plume: a socket on the crest and feather blades fanned along an arc sweeping up and back.
    bm = bmesh.new()
    cv = bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=14, radius1=0.09, radius2=0.06, depth=0.14)["verts"]
    bmesh.ops.translate(bm, vec=Vector((0.0, 0.05, 1.74 + HELM_H + 0.05)), verts=cv)
    o = to_object("brass_socket", bm)
    o.parent = root
    blades = 9
    for b in range(blades):
        f = b / (blades - 1)
        # (Taller and more upright, 2026-10-18: swept far back it was hidden behind the bowl.)
        base = Vector((0.0, 0.05 + 0.45 * f, 1.74 + HELM_H + 0.09 + 0.18 * math.sin(f * math.pi) - 0.08 * f))
        length = 0.85 - 0.35 * f
        lean = math.radians(8.0 + 55.0 * f)
        bm = bmesh.new()
        left, right, mid = [], [], []
        for k in range(7):
            t = k / 6.0
            w = 0.12 * math.sin(math.pi * min(1.0, t * 1.15)) + 0.014
            p = base + Vector((0.0, math.sin(lean) * length * t, math.cos(lean) * length * t + 0.06 * math.sin(t * math.pi)))
            left.append(bm.verts.new(p + Vector((-w, 0.0, 0.0))))
            right.append(bm.verts.new(p + Vector((w, 0.0, 0.0))))
            mid.append(bm.verts.new(p + Vector((0.0, -0.02, 0.02))))
        for k in range(6):
            bm.faces.new((left[k], mid[k], mid[k + 1], left[k + 1]))
            bm.faces.new((mid[k], right[k], right[k + 1], mid[k + 1]))
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.025)
        o = to_object("fabric_feather%d" % b, bm, sharp=70.0)
        o.parent = root
    return root


# ---- Crown ------------------------------------------------------------------------------------------------------------------------------

def crown():
    """A cast gold crown: a band flaring a little to a raised rim top and bottom, five pointed flat-faced leaves each with a ball on its
    tip, and a set stone in the outfit colour under each point."""
    root = empty("crown")
    base = 2.04
    bm = bmesh.new()
    prof = [(0.555, base), (0.585, base + 0.005), (0.59, base + 0.04), (0.575, base + 0.05), (0.585, base + 0.17), (0.61, base + 0.2),
            (0.6, base + 0.225), (0.575, base + 0.22)]
    rings = lathe(bm, prof)
    n = len(rings[0])
    for i in range(n):
        bm.faces.new((rings[-1][i], rings[-1][(i + 1) % n], rings[0][(i + 1) % n], rings[0][i]))
    o = to_object("gold_band", bm, sharp=50.0)
    o.parent = root
    for i in range(5):
        a = math.tau * i / 5.0 - math.pi / 2.0
        out = Vector((math.cos(a), math.sin(a), 0.0))
        side = Vector((-math.sin(a), math.cos(a), 0.0))
        bm = bmesh.new()
        left, right = [], []
        for k in range(9):
            t = k / 8.0
            w = 0.13 * (1.0 - t) ** 0.8 + 0.004
            left.append(bm.verts.new(out * 0.6 - side * w + Vector((0, 0, base + 0.2 + 0.32 * t))))
            right.append(bm.verts.new(out * 0.6 + side * w + Vector((0, 0, base + 0.2 + 0.32 * t))))
        for k in range(8):
            bm.faces.new((left[k], right[k], right[k + 1], left[k + 1]))
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.035)
        o = to_object("gold_point%d" % i, bm, sharp=40.0)
        o.parent = root
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.045)
        bmesh.ops.translate(bm, vec=out * 0.6 + Vector((0, 0, base + 0.54)), verts=bm.verts)
        o = to_object("gold_tip%d" % i, bm)
        o.parent = root
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=1.0)
        for v in ret["verts"]:
            v.co = out * 0.62 + Vector((v.co.x * 0.05, v.co.y * 0.05, v.co.z * 0.06)) + Vector((0, 0, base + 0.11))
        o = to_object("fabric_stone%d" % i, bm)
        o.parent = root
    return root


# ---- Hair ------------------------------------------------------------------------------------------------------------------------------

def _hair_cap(name, front_deg, back_deg, height, bumps=0.0, seed=0.0):
    """A cap of hair over the crown on the caps' ellipsoid (clear of the oval head): down to `front_deg` from the top at the face and
    `back_deg` behind, its hairline in front notched into a few soft points, the surface swelling in `bumps` of locks."""
    bm = bmesh.new()
    seg_u, seg_v = 48, 12
    rows = []
    for j in range(seg_v + 1):
        t = j / seg_v
        row = []
        for i in range(seg_u):
            a = math.tau * i / seg_u
            front = max(0.0, -math.sin(a))
            low = math.radians(back_deg) - math.radians(back_deg - front_deg) * front ** 1.2
            # The hairline's points across the brow.
            low += math.radians(7.0) * front * (0.5 + 0.5 * math.cos(a * 7.0 + seed)) * (t ** 4)
            ph = math.radians(4.0) + (low - math.radians(4.0)) * t
            r = 0.88 * (1.0 + bumps * math.sin(a * 5.0 + seed) * math.sin(ph * 3.0))
            row.append(bm.verts.new((r * math.sin(ph) * math.cos(a), r * math.sin(ph) * math.sin(a), 1.74 + height * math.cos(ph))))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.74 + height))
    for i in range(seg_u):
        bm.faces.new((rows[0][i], rows[0][(i + 1) % seg_u], top))
    for j in range(seg_v):
        for i in range(seg_u):
            k = (i + 1) % seg_u
            bm.faces.new((rows[j][i], rows[j][k], rows[j + 1][k], rows[j + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=-0.04)
    return to_object(name, bm, sharp=60.0)


def _lock(name, base, direction, length, width, curl, sides=7, rings=8):
    """A lock of hair: a thick tapered cone from `base` along `direction`, curving up by `curl`, its cross-section a soft leaf (wider than
    thick), ending in a blunt point."""
    bm = bmesh.new()
    d = Vector(direction).normalized()
    side = d.cross(Vector((0, 0, 1)))
    if side.length < 0.01:
        side = Vector((1, 0, 0))
    side.normalize()
    up = side.cross(d).normalized()
    ring_vs = []
    for k in range(rings):
        t = k / (rings - 1)
        c = Vector(base) + d * length * t + up * curl * t * t
        w = width * (1.0 - t) ** 0.85 + 0.01
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            ring.append(bm.verts.new(c + side * math.cos(a) * w + up * math.sin(a) * w * 0.6))
        ring_vs.append(ring)
    for k in range(rings - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((ring_vs[k][i], ring_vs[k][j], ring_vs[k + 1][j], ring_vs[k + 1][i]))
    bm.faces.new(ring_vs[0])
    tip = bm.verts.new(Vector(base) + d * length * 1.04 + up * curl)
    for i in range(sides):
        bm.faces.new((ring_vs[-1][i], ring_vs[-1][(i + 1) % sides], tip))
    return to_object(name, bm, sharp=55.0)


def swept_spikes():
    """Spiky hair swept up and back: a cap of hair with a pointed hairline, and seven thick locks rising from it, fanned across the crown and
    swept back, each a tapered curving blade with a blunt tip (not straight cones)."""
    root = empty("swept_spikes")
    _hair_cap("hair_cap", 62.0, 118.0, 0.62, bumps=0.03, seed=1.3).parent = root
    # (Thick swept blades lying back over the head, 2026-10-18: thin upright cones read as a crown of thorns.)
    locks = [(-0.38, -0.3, 2.12, (-0.45, 0.55, 0.7), 0.7, 0.3, 0.16), (0.0, -0.38, 2.22, (0.0, 0.6, 0.8), 0.8, 0.32, 0.18),
             (0.38, -0.3, 2.12, (0.45, 0.55, 0.7), 0.7, 0.3, 0.16), (-0.42, 0.15, 2.18, (-0.4, 0.9, 0.35), 0.72, 0.28, 0.12),
             (0.42, 0.15, 2.18, (0.4, 0.9, 0.35), 0.72, 0.28, 0.12), (0.0, 0.2, 2.3, (0.0, 1.0, 0.3), 0.75, 0.3, 0.1)]
    for k, (x, y, z, d, ln, w, c) in enumerate(locks):
        _lock("hair_lock%d" % k, (x, y, z), d, ln, w, c).parent = root
    return root


def topknot():
    """Close-cropped hair with a round knot of hair tied on top: the cap with a straight hairline, a bun of three twisted lobes, and a band
    of the outfit's colour where the knot is tied."""
    root = empty("topknot")
    _hair_cap("hair_cap", 66.0, 118.0, 0.6, bumps=0.015, seed=0.4).parent = root
    for k in range(3):
        a = math.tau * k / 3.0
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=18, v_segments=12, radius=1.0)
        for v in ret["verts"]:
            v.co = Vector((v.co.x * 0.22, v.co.y * 0.22, v.co.z * 0.17)) + Vector((0.08 * math.cos(a), 0.12 + 0.08 * math.sin(a), 2.5 + 0.04 * k))
        to_object("hair_knot%d" % k, bm).parent = root
    bm = bmesh.new()
    rows = lathe(bm, [(0.17, 2.34), (0.19, 2.36), (0.19, 2.41), (0.17, 2.43)], sides=24)
    for ring in rows:
        for v in ring:
            v.co.y += 0.12
    to_object("fabric_band", bm, sharp=60.0).parent = root
    return root


def beanie():
    """A knit beanie: a ribbed crown pulled over the head (vertical ribs pressed into it), a thick folded cuff with its own ribs, and a
    pompom of soft lumps on top."""
    root = empty("beanie")
    bm = bmesh.new()
    seg_u, seg_v = 56, 14
    rows = []
    for j in range(seg_v + 1):
        t = j / seg_v
        row = []
        for i in range(seg_u):
            a = math.tau * i / seg_u
            front = max(0.0, -math.sin(a))
            low = math.radians(98.0) - math.radians(26.0) * front ** 1.3
            ph = math.radians(3.0) + (low - math.radians(3.0)) * t
            rib = 1.0 + 0.012 * math.cos(a * 28.0) * min(1.0, t * 3.0)
            r = 0.9 * rib
            row.append(bm.verts.new((r * math.sin(ph) * math.cos(a), r * math.sin(ph) * math.sin(a), 1.74 + 0.68 * math.cos(ph))))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.74 + 0.68))
    for i in range(seg_u):
        bm.faces.new((rows[0][i], rows[0][(i + 1) % seg_u], top))
    for j in range(seg_v):
        for i in range(seg_u):
            k = (i + 1) % seg_u
            bm.faces.new((rows[j][i], rows[j][k], rows[j + 1][k], rows[j + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=-0.04)
    to_object("fabric_crown", bm).parent = root
    # The folded cuff: a thick band round the lower edge, a little proud.
    bm = bmesh.new()
    sides = 56
    rings = []
    for dr, dph in [(0.0, -2.0), (0.06, -1.0), (0.07, 8.0), (0.06, 16.0), (0.0, 17.0)]:
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            front = max(0.0, -math.sin(a))
            ph = math.radians(98.0 - dph) - math.radians(26.0) * front ** 1.3
            r = 0.9 * math.sin(ph) + dr * (1.0 + 0.15 * math.cos(a * 28.0))
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), 1.74 + 0.68 * math.cos(ph))))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    to_object("fabric_cuff", bm).parent = root
    # The pompom: a cluster of soft lumps.
    for k in range(7):
        a = math.tau * k / 6.0
        off = Vector((0.0, 0.0, 0.0)) if k == 6 else Vector((0.09 * math.cos(a), 0.09 * math.sin(a), -0.02))
        bm = bmesh.new()
        bmesh.ops.create_icosphere(bm, subdivisions=2, radius=0.12)
        bmesh.ops.translate(bm, vec=Vector((0.0, 0.0, 2.48)) + off, verts=bm.verts)
        to_object("canvas_pom%d" % k, bm).parent = root
    return root


GEAR = {"sailor_cap": sailor_cap, "aviator_cap": aviator_cap, "straw_hat": straw_hat, "plumed_helm": plumed_helm, "crown": crown,
        "swept_spikes": swept_spikes, "topknot": topknot, "beanie": beanie}


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
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=32, v_segments=20, radius=HEAD_R)
    bmesh.ops.translate(bm, vec=HEAD_C, verts=bm.verts)
    head = to_object("head", bm)
    cam_data = bpy.data.cameras.new("cam")
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    target = Vector((0.0, 0.0, 1.85))
    cam.location = target + Vector((1.6, -4.2, 1.2))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)
    bpy.data.objects.remove(head)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    only = opts["only"].split(",") if "only" in opts else list(GEAR)
    os.makedirs(OUT, exist_ok=True)
    for name in only:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        root = GEAR[name]()
        export(root, os.path.join(OUT, name + ".glb"))
        tris = sum(sum(len(p.vertices) - 2 for p in c.data.polygons) for c in root.children_recursive if c.type == "MESH")
        print("gear %s: %d triangles" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
