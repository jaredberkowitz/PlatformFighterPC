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
import random
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

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

def add_box(bm, centre, size):
    """An axis-aligned box into `bm`, `size` its full extents."""
    ret = bmesh.ops.create_cube(bm, size=1.0)
    for v in ret["verts"]:
        v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2])) + Vector(centre)


def _tilt(root, deg, pivot_z):
    """Tips every part of a hat back by `deg` round the x axis at height `pivot_z` (worn pushed back on the head)."""
    m = Matrix.Translation((0, 0, pivot_z)) @ Matrix.Rotation(math.radians(deg), 4, "X") @ Matrix.Translation((0, 0, -pivot_z))
    for c in root.children:
        c.data.transform(m)


def sailor_cap():
    """The yellow sailor's soft white cap, after the owner's reference cast (2026-10-19): a tall puffy crown of cloth rising from a wide
    band of the outfit's colour at its foot, swelling out and over at the top in a soft overhang with a sag in its middle, worn tipped
    back on the head. (A flat boater-like cap before.)"""
    root = empty("sailor_cap")
    base = 2.0
    prof = [(head_r(base) - 0.02, base - 0.04), (0.56, base + 0.0), (0.58, base + 0.2), (0.62, base + 0.36), (0.66, base + 0.48),
            (0.64, base + 0.56), (0.54, base + 0.62), (0.36, base + 0.6), (0.16, base + 0.57)]
    bm = bmesh.new()
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + (0.025 * math.sin(a * 3.0 + 0.4) if k >= 3 else 0.0))
    cap_ring(bm, rings[-1], base + 0.56)
    to_object("canvas_crown", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.575, base - 0.02), (0.595, base + 0.0), (0.6, base + 0.16), (0.585, base + 0.19)])
    to_object("fabric_band", bm, sharp=60.0).parent = root
    _tilt(root, -12.0, base)
    return root


def dixie_cap():
    """The blue fighter's little white sailor's hat, after the owner's reference cast (2026-10-19): a small round crown sitting high on
    the head, its brim turned straight up all round as a stiff wall with a navy stripe along its top edge, and a navy anchor on the front
    of the brim."""
    root = empty("dixie_cap")
    base = 2.1
    bm = bmesh.new()
    rings = lathe(bm, [(head_r(base) - 0.02, base - 0.04), (0.44, base + 0.02), (0.46, base + 0.22), (0.45, base + 0.36), (0.38, base + 0.46),
                       (0.2, base + 0.5)])
    cap_ring(bm, rings[-1], base + 0.505)
    to_object("canvas_crown", bm).parent = root
    bm = bmesh.new()
    rings = lathe(bm, [(0.44, base + 0.0), (0.52, base + 0.02), (0.55, base + 0.1), (0.55, base + 0.18), (0.52, base + 0.2), (0.47, base + 0.17)])
    n = len(rings[0])
    for i in range(n):
        bm.faces.new((rings[-1][i], rings[-1][(i + 1) % n], rings[0][(i + 1) % n], rings[0][i]))
    to_object("canvas_brim", bm, sharp=60.0).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.553, base + 0.13), (0.558, base + 0.15), (0.558, base + 0.175), (0.553, base + 0.19)])
    to_object("navy_stripe", bm, sharp=60.0).parent = root
    # The anchor on the front of the brim (the front is -y): a shank, a stock across its top, a ring, and the arms curving up at its foot.
    bm = bmesh.new()
    y = -0.565
    for (cx, cz, w, h) in ((0.0, base + 0.085, 0.014, 0.1), (0.0, base + 0.125, 0.06, 0.012)):
        add_box(bm, (cx, y, cz), (w, 0.012, h))
    for k in range(7):
        t = (k / 6.0) * 2.0 - 1.0
        add_box(bm, (t * 0.04, y, base + 0.045 + 0.022 * t * t), (0.016, 0.012, 0.012))
    to_object("navy_anchor", bm, smooth=False).parent = root
    _tilt(root, -6.0, base)
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
            # (Lower and fuller, the aviator scout pilot: on the head's crown it sat high as a beret; the reference's hat comes down to the
            # brow and round over the ears.)
            # (A big round dome, the aviator scout pilot round 2: the reference's hat is a wide padded bowl well over the head.)
            r = 1.0
            x = r * math.sin(ph) * math.cos(a)
            y = r * math.sin(ph) * math.sin(a)
            # (Domed up, the character judge's round 9: a thin cap over the fleece in pose.)
            z = 1.52 + 0.98 * math.cos(ph)
            # The seam down the middle of the crown.
            if abs(x) < 0.03:
                x *= 0.6
                z -= 0.008
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.52 + 0.98))
    for i in range(seg_u):
        bm.faces.new((rows[0][i], rows[0][(i + 1) % seg_u], top))
    for j in range(seg_v):
        for i in range(seg_u):
            k = (i + 1) % seg_u
            bm.faces.new((rows[j][i], rows[j][k], rows[j + 1][k], rows[j + 1][i]))
    rim = [v.co.copy() for v in rows[-1]]
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=-0.03)
    o = to_object("leather_shell", bm)
    o.parent = root
    # (The aviator scout pilot, 2026-10-19, after the owner's reference: a fleece-lined flying hat, its fleece rolled out round the rim and
    # lining big ear flaps, and hair spilling out under it; the close leather cap alone read as a bare skull cap.)
    # The fleece roll round the rim: a fat lumpy tube following the shell's lower edge, a little out from it.
    rng = random.Random(23)
    bm = bmesh.new()
    n = len(rim)
    minor = 10
    loops = []
    for i in range(n):
        c = rim[i]
        out = Vector((c.x, c.y, 0.0)).normalized()
        tangent = (rim[(i + 1) % n] - rim[i - 1]).normalized()
        up = tangent.cross(out).normalized()
        if up.z < 0.0:
            up = -up
        lump = 1.0 + 0.12 * math.sin(i * 1.7) + rng.uniform(-0.06, 0.06)
        loop = []
        for j in range(minor):
            b_ = math.tau * j / minor
            # (Fat, the character judge's round 1: a thin ring read as a brim.)
            # (A fifth smaller and drawn in to the head's width, the character judge's round 2: wider than the head it read as a brim.)
            r = 0.16 * lump * (1.0 + 0.12 * math.sin(b_ * 3.0 + i))
            # (Dipping lower over the ears, so it follows the head, the character judge's round 3: level, it read as a plate.)
            side_k = abs(c.x) / max(Vector((c.x, c.y, 0.0)).length, 1e-4)
            # (Taller in section than wide: the game widens the head, and the hat with it, 1.32 times across, which flattened a round tube
            # into a plate, the character judge's round 4.)
            loop.append(bm.verts.new(c - out * 0.09 + Vector((0.0, 0.0, 0.26 - 0.1 * side_k)) + out * math.cos(b_) * r + up * math.sin(b_) * r * 1.35))
        loops.append(loop)
    # (UVs round and along the roll, so the game's curly fleece tile lies on it.)
    uv = bm.loops.layers.uv.new("UVMap")
    for i in range(n):
        for j in range(minor):
            f = bm.faces.new((loops[i][j], loops[(i + 1) % n][j], loops[(i + 1) % n][(j + 1) % minor], loops[i][(j + 1) % minor]))
            for lp, (a_, b_) in zip(f.loops, ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1))):
                lp[uv].uv = (a_ / n * 7.0, b_ / minor * 0.6)
    to_object("fleece_trim", bm).parent = root
    # Ear flaps: long leather tabs hanging past the jaw, each lined with fleece that shows round its edge.
    for sx in (-1.0, 1.0):
        nm = "L" if sx < 0 else "R"
        # (Hugging the cheeks, the leather outside and only a rim of fleece showing, the character judge's round 1: pale paddles hanging
        # free read as bunny ears.)
        for part, k, inset in (("leather_flap", 1.0, 0.0), ("fleece_flap", 1.08, -0.025)):
            bm = bmesh.new()
            ret = bmesh.ops.create_uvsphere(bm, u_segments=20, v_segments=14, radius=1.0, calc_uvs=True)
            for v in ret["verts"]:
                # (A quarter smaller and padded thicker, folded in at its lower edge, the character judge's round 2.)
                # (Hanging below the fleece in front of the hair as a padded tongue, the character judge's round 3: hidden, its top read as a
                # dark disc on the cheek.)
                # (Thin, hugging the cheek, the character judge's round 4: thick, it cupped out as a headphone.)
                # (Narrow front to back, so side-on it is a strap down the cheek, not a disc, the character judge's round 6.)
                v.co = Vector((v.co.x * 0.06, v.co.y * 0.17 * k, v.co.z * 0.44 * k))
                # (Curving in under the jaw, following the cheek.)
                v.co.x -= sx * 0.12 * max(0.0, -v.co.z / 0.44) ** 2
            # (Forward and down beside the cheeks, so they show at three-quarter, the character judge's round 9.)
            bmesh.ops.translate(bm, vec=Vector((sx * (0.84 + inset), -0.22, 0.98)), verts=ret["verts"])
            to_object(part + nm, bm).parent = root
    # Hair spilling out under the hat round the sides and back: soft thick locks hanging down and out.
    # (Six thick rounded locks with soft tips, the character judge's round 10: thirteen flat shards read as cut paper or flames.)
    locks = 6
    for k in range(locks):
        a = math.radians(-20.0 + 220.0 * k / (locks - 1))   # from the right side round the back to the left (front is -y)
        a = math.radians(-10.0) + (math.pi + math.radians(20.0)) * k / (locks - 1)
        base = Vector((0.82 * math.cos(a), 0.82 * math.sin(a), 1.4))
        out = Vector((math.cos(a), math.sin(a), 0.0))
        # (Falling down, only a little out, the character judge's round 10.)
        d = out * 0.02 + Vector((0.0, 0.0, -1.0))
        _lock("hair_spill%d" % k, base, d, rng.uniform(0.5, 0.66), rng.uniform(0.22, 0.27), 0.05, sides=12, rings=9, depth=0.75,
              across=(-math.sin(a), math.cos(a), 0.0)).parent = root
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
    """The pink ranger's straw hat, after the owner's reference cast (2026-10-19): a fedora-like crown pinched in at the front and creased
    along its top, a brim flat at the front and back and curled up at the sides, and a band of the outfit's colour round the crown's foot.
    (A wide flat boater before.)"""
    root = empty("straw_hat")
    base = 2.0
    bm = bmesh.new()
    sides = 48
    rings = []
    radii = [0.5, 0.6, 0.7, 0.8, 0.87, 0.9]
    for k, r in enumerate(radii):
        ring = []
        u = (r - 0.5) / 0.4
        for i in range(sides):
            a = math.tau * i / sides
            curl = 0.26 * abs(math.cos(a)) ** 2 * u ** 2
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), base + 0.03 + curl - 0.03 * u)))
        rings.append(ring)
    # (UVs round and across the brim, so the game's woven straw tile lies on it, the character judge's round 5: smooth, it read as plastic.)
    uv = bm.loops.layers.uv.new("UVMap")
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            f = bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
            for lp, (a_, b_) in zip(f.loops, ((i, k), (i + 1, k), (i + 1, k + 1), (i, k + 1))):
                lp[uv].uv = (a_ / sides * 6.0, b_ / (len(rings) - 1) * 0.6)
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.03)
    to_object("straw_brim", bm, sharp=70.0).parent = root
    bm = bmesh.new()
    # (Tall enough to stand clear of the game's head, which is drawn taller and wider than the design head.)
    prof = [(0.5, base + 0.03), (0.58, base + 0.05), (0.57, base + 0.48), (0.52, base + 0.64), (0.38, base + 0.7), (0.18, base + 0.69)]

    def pinch(a, k):
        # The pinch: two dents in the front of the crown's upper half (the front is -y, angle 270 degrees).
        front = max(0.0, -math.sin(a))
        side = abs(math.cos(a))
        return 1.0 - (0.3 * front ** 2 * side * 2.0 if k >= 2 else 0.0)
    rings = lathe(bm, prof, wobble=pinch)
    cap_ring(bm, rings[-1], base + 0.69)
    uv = bm.loops.layers.uv.new("UVMap")
    for f in bm.faces:
        for lp in f.loops:
            co = lp.vert.co
            lp[uv].uv = ((math.atan2(co.y, co.x) / math.tau + 0.5) * 4.0, (co.z - base) * 1.6)
    # The crease along the top, front to back.
    for ring in rings[3:]:
        for v in ring:
            v.co.z -= 0.07 * max(0.0, 1.0 - abs(v.co.x) / 0.25)
    to_object("straw_crown", bm).parent = root
    bm = bmesh.new()
    lathe(bm, [(0.585, base + 0.05), (0.592, base + 0.07), (0.588, base + 0.14), (0.58, base + 0.16)])
    # (A dark leather band, as the reference's.)
    to_object("leather_band", bm, sharp=60.0).parent = root
    _tilt(root, -5.0, base)
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


def _lock(name, base, direction, length, width, curl, sides=7, rings=8, depth=0.6, across=None):
    """A lock of hair: a thick tapered cone from `base` along `direction`, curving up by `curl`, its cross-section a soft leaf (wider than
    thick), ending in a blunt point."""
    bm = bmesh.new()
    d = Vector(direction).normalized()
    # (`across`: the way the lock's width runs; by default level and square to it. A lock hanging straight down takes it round the head,
    # or its width stands out from the head as a wing.)
    side = Vector(across) if across is not None else d.cross(Vector((0, 0, 1)))
    if side.length < 0.01:
        side = Vector((1, 0, 0))
    side = (side - d * side.dot(d)).normalized()
    up = side.cross(d).normalized()
    ring_vs = []
    for k in range(rings):
        t = k / (rings - 1)
        c = Vector(base) + d * length * t + up * curl * t * t
        w = width * (1.0 - t) ** 0.85 + 0.01
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            ring.append(bm.verts.new(c + side * math.cos(a) * w + up * math.sin(a) * w * depth))
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


# ---- Glasses (one file of parts, each at its size round its own centre, facing -Y in Blender, +Z in the game) ----------------------

def _frame_loop(bm, outline, width, depth, top_extra=0.0):
    """A frame round a closed `outline` [(x, z), ...] (its centre at the origin): a flat-faced band `width` wide inward-outward and `depth`
    deep, a little wider along the top (`top_extra`, a brow bar), its edges softened by a mid ring."""
    n = len(outline)
    rings = []
    for depth_k, out_k in [(-0.5, 0.0), (-0.5, 1.0), (0.0, 1.08), (0.5, 1.0), (0.5, 0.0), (0.0, -0.06)]:
        ring = []
        for (x, z) in outline:
            r = math.hypot(x, z)
            ux, uz = (x / r, z / r) if r > 1e-6 else (0.0, 1.0)
            w = width * (1.0 + top_extra * max(0.0, uz))
            ring.append(bm.verts.new((x + ux * w * out_k, -depth * depth_k, z + uz * w * out_k)))
        rings.append(ring)
    for k in range(len(rings)):
        a, b = rings[k], rings[(k + 1) % len(rings)]
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((a[i], a[j], b[j], b[i]))


def _superellipse(a, b, power, n=48):
    return [(a * math.copysign(abs(math.cos(t)) ** (2.0 / power), math.cos(t)), b * math.copysign(abs(math.sin(t)) ** (2.0 / power), math.sin(t)))
            for t in [math.tau * i / n for i in range(n)]]


def glasses():
    """The glasses' parts: round specs' rim (a thin flat-faced wire ring), shades' rim (a rounded-square frame with a heavier brow bar)
    and lens (a filled, slightly domed pane), goggles' cup (a deep padded ring), and a bridge (a small arc)."""
    root = empty("glasses")
    bm = bmesh.new()
    _frame_loop(bm, [(0.18 * math.cos(t), 0.18 * math.sin(t)) for t in [math.tau * i / 48 for i in range(48)]], 0.05, 0.035)
    to_object("metal_specs_rim", bm).parent = root
    bm = bmesh.new()
    # (Thin wire drawn wider than the eyes, the aviator scout pilot, after the owner's reference: a heavy plastic frame sat on the eyes.)
    # (A tenth narrower, the character judge's round 9: at three-quarter the far lens broke out past the head.)
    _frame_loop(bm, _superellipse(0.225, 0.17, 3.0), 0.016, 0.025, top_extra=0.35)
    to_object("frame_shades_rim", bm).parent = root
    bm = bmesh.new()
    outline = _superellipse(0.225, 0.17, 3.0)
    c = bm.verts.new((0.0, -0.012, 0.0))
    ring = [bm.verts.new((x, 0.0, z)) for (x, z) in outline]
    for i in range(len(ring)):
        bm.faces.new((ring[i], ring[(i + 1) % len(ring)], c))
    to_object("glass_shades_lens", bm, smooth=True).parent = root
    bm = bmesh.new()
    _frame_loop(bm, [(0.16 * math.cos(t), 0.16 * math.sin(t)) for t in [math.tau * i / 48 for i in range(48)]], 0.08, 0.1)
    to_object("leather_goggle_cup", bm).parent = root
    bm = bmesh.new()
    pts = [(0.09 * (2.0 * k / 8.0 - 1.0), 0.025 * math.sin(math.pi * k / 8.0)) for k in range(9)]
    rings = []
    for (x, z) in pts:
        rr = []
        for i in range(6):
            a = math.tau * i / 6
            rr.append(bm.verts.new((x, 0.014 * math.cos(a), z + 0.014 * math.sin(a))))
        rings.append(rr)
    for k in range(len(rings) - 1):
        for i in range(6):
            j = (i + 1) % 6
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    to_object("frame_bridge", bm).parent = root
    return root


# ---- Neckwear (one file of parts, each in the game's own space for where it is placed, see fighter_view.gd) ---------------------------

def _g(x, y, z):
    """A point given in the game's axes (y up, z toward the camera) as Blender's (z up, -y toward the camera)."""
    return Vector((x, -z, y))


def neckwear():
    """The neckwear's cloth: a rolled collar (a soft tube of cloth round the neck, gathered in shallow folds), a knot for the neckerchief,
    the neckerchief's point (a draped triangle with a fold down its middle and soft creases), and the sash (a flat strap with turned
    edges, worn on the diagonal)."""
    root = empty("neckwear")
    # The collar, round the torso rig's neck at its origin (the game places it at the collar's height): ellipse radii 0.54 by 0.49.
    bm = bmesh.new()
    seg, minor = 64, 10
    rows = []
    for i in range(seg):
        a = math.tau * i / seg
        fold = 1.0 + 0.03 * math.sin(a * 9.0)
        cx, cz = 0.54 * math.cos(a), 0.49 * math.sin(a)
        n = Vector((math.cos(a) / 0.54, 0.0, math.sin(a) / 0.49)).normalized()
        row = []
        for j in range(minor):
            b = math.tau * j / minor
            off = n * (0.065 * fold * math.cos(b))
            row.append(bm.verts.new(_g(cx + off.x, 0.05 * fold * math.sin(b), cz + off.z)))
        rows.append(row)
    for i in range(seg):
        for j in range(minor):
            bm.faces.new((rows[i][j], rows[(i + 1) % seg][j], rows[(i + 1) % seg][(j + 1) % minor], rows[i][(j + 1) % minor]))
    to_object("fabric_collar", bm).parent = root
    # The neckerchief's knot and point, round the knot at the origin (the game's neck pivot).
    bm = bmesh.new()
    ret = bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=10, radius=1.0)
    for v in ret["verts"]:
        g = Vector((v.co.x * 0.08, v.co.z * 0.065, v.co.y * 0.05))
        g.x *= 1.0 + 0.15 * math.sin(v.co.z * 4.0)
        v.co = _g(g.x, g.y, g.z + 0.02)
    to_object("fabric_knot", bm).parent = root
    bm = bmesh.new()
    rows_n, cols_n = 8, 9
    grid = []
    for r in range(rows_n + 1):
        t = r / rows_n
        half = 0.23 * (1.0 - t) + 0.006
        row = []
        for c in range(cols_n + 1):
            u = c / cols_n * 2.0 - 1.0
            x = half * u
            y = -0.02 - 0.33 * t
            # A fold down the middle standing out, the sides curling back a little, soft creases from the knot.
            z = 0.035 * (1.0 - abs(u)) - 0.03 * u * u + 0.01 * math.sin(u * 6.0 + t * 3.0) * (1.0 - t)
            row.append(bm.verts.new(_g(x, y, z)))
        grid.append(row)
    for r in range(rows_n):
        for c in range(cols_n):
            bm.faces.new((grid[r][c], grid[r][c + 1], grid[r + 1][c + 1], grid[r + 1][c]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.025)
    to_object("fabric_kerchief", bm).parent = root
    # The sash: a flat strap round the sash node's origin (the game turns it onto the diagonal), ellipse radii 0.5 by 0.57 in its x and z,
    # 0.22 wide along its y, its edges turned.
    bm = bmesh.new()
    seg = 72
    profile = [(-0.11, -0.012), (-0.1, 0.012), (0.0, 0.02), (0.1, 0.012), (0.11, -0.012), (0.0, -0.02)]
    rows = []
    for i in range(seg):
        a = math.tau * i / seg
        cx, cz = 0.5 * math.cos(a), 0.57 * math.sin(a)
        n = Vector((math.cos(a) / 0.5, 0.0, math.sin(a) / 0.57)).normalized()
        row = []
        for (w, o) in profile:
            q = Vector((cx, 0.0, cz)) + n * (0.012 + o)
            row.append(bm.verts.new(_g(q.x, w, q.z)))
        rows.append(row)
    m = len(profile)
    for i in range(seg):
        for j in range(m):
            bm.faces.new((rows[i][j], rows[(i + 1) % seg][j], rows[(i + 1) % seg][(j + 1) % m], rows[i][(j + 1) % m]))
    to_object("fabric_sash", bm, sharp=50.0).parent = root
    return root


GEAR = {"neckwear": neckwear, "glasses": glasses, "sailor_cap": sailor_cap, "dixie_cap": dixie_cap, "aviator_cap": aviator_cap, "straw_hat": straw_hat, "plumed_helm": plumed_helm, "crown": crown,
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
