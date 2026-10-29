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
            # (Close to the head, the character judge's round 17: wider, it read as an acorn cap in the run.)
            r = 0.9
            x = r * math.sin(ph) * math.cos(a)
            y = r * math.sin(ph) * math.sin(a)
            # (Domed up, the character judge's round 9: a thin cap over the fleece in pose.)
            # (A tall dome, the character judge's round 14: it read flat from above.)
            z = 1.52 + 1.1 * math.cos(ph)
            # The seam down the middle of the crown.
            if abs(x) < 0.03:
                x *= 0.6
                z -= 0.008
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.52 + 1.1))
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
            r = 0.12 * lump * (1.0 + 0.12 * math.sin(b_ * 3.0 + i))
            # (Dipping lower over the ears, so it follows the head, the character judge's round 3: level, it read as a plate.)
            side_k = abs(c.x) / max(Vector((c.x, c.y, 0.0)).length, 1e-4)
            # (Taller in section than wide: the game widens the head, and the hat with it, 1.32 times across, which flattened a round tube
            # into a plate, the character judge's round 4.)
            # (Lower at the back, over the shell's edge, the cap pass, 2026-10-19: seen from behind the leather's rim showed under the roll
            # as a ragged dark band.)
            back_k = max(0.0, c.y) / max(Vector((c.x, c.y, 0.0)).length, 1e-4)
            loop.append(bm.verts.new(c - out * 0.07 + Vector((0.0, 0.0, 0.1 - 0.08 * side_k - 0.08 * back_k)) + out * math.cos(b_) * r + up * math.sin(b_) * r * 1.35))
        loops.append(loop)
    # (UVs round and along the roll, so the game's curly fleece tile lies on it.)
    uv = bm.loops.layers.uv.new("UVMap")
    for i in range(n):
        for j in range(minor):
            f = bm.faces.new((loops[i][j], loops[(i + 1) % n][j], loops[(i + 1) % n][(j + 1) % minor], loops[i][(j + 1) % minor]))
            for lp, (a_, b_) in zip(f.loops, ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1))):
                lp[uv].uv = (a_ / n * 7.0, b_ / minor * 0.6)
    to_object("fleece_trim", bm).parent = root
    # Ear flaps: fleece pads hanging from the fleece roll down beside each cheek.
    # (Fleece all through since the cap pass, 2026-10-19, after the owner's style reference, whose flaps are the same fleece as the rim:
    # the leather tongue read side-on as a flat brown disc pasted on the cheek, and the fleece edge down its front, five separate balls,
    # as a string of beads. Now one soft lumpy pad, wider at the top where it joins the roll, tapering down past the jaw.)
    rng_f = random.Random(7)
    for sx in (-1.0, 1.0):
        nm = "L" if sx < 0 else "R"
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=20, v_segments=14, radius=1.0, calc_uvs=True)
        bumps = [(Vector((rng_f.uniform(-1, 1), rng_f.uniform(-1, 1), rng_f.uniform(-1, 1))).normalized(), rng_f.uniform(0.04, 0.08))
                 for _ in range(9)]
        for v in ret["verts"]:
            n0 = v.co.normalized()
            # (Soft lumps, as the rim's roll: fleece, not a smooth egg.)
            k_l = 1.0 + sum(h * max(0.0, n0.dot(d)) ** 6 for d, h in bumps)
            down = max(0.0, -v.co.z)
            # Wider at the top, tapering toward its foot.
            w = 1.0 - 0.4 * down
            v.co = Vector((v.co.x * 0.15 * k_l, v.co.y * 0.19 * w * k_l, v.co.z * 0.5 * k_l))
            # (Curving in under the jaw, following the cheek.)
            v.co.x -= sx * 0.12 * max(0.0, -v.co.z / 0.5) ** 2
        # (Its top tucked up under the fleece roll, so it hangs from the rim rather than sitting on the cheek as a separate pad.)
        bmesh.ops.translate(bm, vec=Vector((sx * 0.87, -0.16, 1.2)), verts=ret["verts"])
        to_object("fleece_flap" + nm, bm).parent = root
    # (No hair: the hat covers it all, the owner, 2026-10-19; hair pieces are to be their own items.)
    # The goggles pushed up on the front of the crown, oversized (the character judge's round 14: the aviator's signature from every angle,
    # lost with the hair): two brass rims with glass on a leather strap round the crown.
    bm = bmesh.new()
    # (On the shell, a strap's thickness off it, the cap pass, 2026-10-19: at a fixed 0.93 it stood 0.12 off the dome as a brim round the
    # back of the hat.)
    def shell_r(z):
        return 0.9 * math.sqrt(max(0.0, 1.0 - ((z - 1.52) / 1.1) ** 2))
    lathe(bm, [(shell_r(1.98) + 0.045, 1.98), (shell_r(2.08) + 0.045, 2.08)])   # (clear of the shell's dents, _lumpy)
    to_object("leather_band", bm, sharp=60.0).parent = root
    for sx in (-1.0, 1.0):
        centre = Vector((sx * 0.3, -0.86, 2.06))
        tilt = Matrix.Rotation(math.radians(-28.0), 3, "X")
        bm = bmesh.new()
        sides, minor = 28, 10
        rows = []
        for i in range(sides):
            a_ = math.tau * i / sides
            row = []
            for j in range(minor):
                b_ = math.tau * j / minor
                rr = 0.22 + 0.06 * math.cos(b_)
                row.append(bm.verts.new(centre + tilt @ Vector((rr * math.cos(a_), -0.06 * math.sin(b_) - 0.03, rr * math.sin(a_)))))
            rows.append(row)
        for i in range(sides):
            for j in range(minor):
                bm.faces.new((rows[i][j], rows[(i + 1) % sides][j], rows[(i + 1) % sides][(j + 1) % minor], rows[i][(j + 1) % minor]))
        to_object("brass_rim" + ("L" if sx < 0 else "R"), bm).parent = root
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=20, v_segments=10, radius=1.0)
        for v in ret["verts"]:
            v.co = centre + tilt @ Vector((v.co.x * 0.22, v.co.y * 0.04 - 0.04, v.co.z * 0.22))
        to_object("glass_lens" + ("L" if sx < 0 else "R"), bm).parent = root
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
    # (Wider, drooping at the front and back, the character judge's round 14: a neat cowboy hat, not the reference's floppy straw.)
    radii = [0.5, 0.62, 0.74, 0.86, 0.96, 1.02]
    for k, r in enumerate(radii):
        ring = []
        u = (r - 0.5) / 0.52
        for i in range(sides):
            a = math.tau * i / sides
            curl = 0.26 * abs(math.cos(a)) ** 2 * u ** 2 - 0.16 * abs(math.sin(a)) ** 2 * u ** 2
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
HELM_H = 1.12
# (The helm pass, 2026-10-19: a tall dome on the aviator cap's shell, centred low, so it comes down over the head like a kettle helm;
# centred at 1.74 with a 0.82 rise it sat on top as a wide dish, a pan lid. Its sides come down past the ears, so the separate cheek
# guards, flat slabs that crossed the eyes at three-quarters, are gone.)
HELM_C = 1.52
HELM_R = 0.9
HELM_BACK, HELM_FRONT = 104.0, 46.0   # (the front cut up over the brows, so the face and brows show)


def plumed_helm():
    """A knight's open helm: a steel bowl over the head with a raised ridge from brow to nape and a brass brow band with rivets, and a tall plume of feather blades in the outfit's colour sweeping back off a brass
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
            ph = phi * (_bowl_phi_low(a, HELM_BACK, HELM_FRONT) / math.radians(100.0))
            x = HELM_R * math.sin(ph) * math.cos(a)
            y = HELM_R * math.sin(ph) * math.sin(a)
            z = HELM_C + HELM_H * math.cos(ph)
            if abs(x) < 0.06:
                z += 0.035 * (1.0 - abs(x) / 0.06) * math.sin(ph)
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, HELM_C + HELM_H + 0.03))
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
            ph = _bowl_phi_low(a, HELM_BACK, HELM_FRONT) - math.radians(8.0)
            r = HELM_R * math.sin(ph) + dr
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), HELM_C + HELM_H * math.cos(ph) + dz)))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    o = to_object("brass_band", bm, sharp=60.0)
    o.parent = root
    for i in range(10):
        a = math.tau * (i + 0.5) / 10.0
        ph = _bowl_phi_low(a, HELM_BACK, HELM_FRONT) - math.radians(8.0)
        p = Vector(((HELM_R + 0.04) * math.sin(ph) * math.cos(a), (HELM_R + 0.04) * math.sin(ph) * math.sin(a), HELM_C + HELM_H * math.cos(ph)))
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=10, v_segments=6, radius=0.028)
        bmesh.ops.translate(bm, vec=p, verts=bm.verts)
        o = to_object("brass_rivet%d" % i, bm)
        o.parent = root
    # The plume: a socket on the crest and feather blades fanned along an arc sweeping up and back.
    bm = bmesh.new()
    cv = bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=14, radius1=0.09, radius2=0.06, depth=0.14)["verts"]
    bmesh.ops.translate(bm, vec=Vector((0.0, 0.05, HELM_C + HELM_H + 0.05)), verts=cv)
    o = to_object("brass_socket", bm)
    o.parent = root
    blades = 9
    for b in range(blades):
        f = b / (blades - 1)
        # (Taller and more upright, 2026-10-18: swept far back it was hidden behind the bowl.)
        base = Vector((0.0, 0.05 + 0.45 * f, HELM_C + HELM_H + 0.09 + 0.18 * math.sin(f * math.pi) - 0.08 * f))
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
    # (Lower and snug, the crown pass, 2026-10-19: at 2.04 with a 0.555 band it stood off the head with a gap all round, and floated.)
    base = 1.97
    fit = (head_r(base) + 0.01) / 0.555
    bm = bmesh.new()
    prof = [(0.555 * fit, base), (0.585 * fit, base + 0.005), (0.59 * fit, base + 0.04), (0.575 * fit, base + 0.05), (0.585 * fit, base + 0.17),
            (0.61 * fit, base + 0.2), (0.6 * fit, base + 0.225), (0.575 * fit, base + 0.22)]
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
            left.append(bm.verts.new(out * 0.6 * fit - side * w + Vector((0, 0, base + 0.2 + 0.32 * t))))
            right.append(bm.verts.new(out * 0.6 * fit + side * w + Vector((0, 0, base + 0.2 + 0.32 * t))))
        for k in range(8):
            bm.faces.new((left[k], right[k], right[k + 1], left[k + 1]))
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.035)
        o = to_object("gold_point%d" % i, bm, sharp=40.0)
        o.parent = root
        bm = bmesh.new()
        bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.045)
        bmesh.ops.translate(bm, vec=out * 0.6 * fit + Vector((0, 0, base + 0.54)), verts=bm.verts)
        o = to_object("gold_tip%d" % i, bm)
        o.parent = root
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=1.0)
        for v in ret["verts"]:
            v.co = out * 0.62 * fit + Vector((v.co.x * 0.05, v.co.y * 0.05, v.co.z * 0.06)) + Vector((0, 0, base + 0.11))
        o = to_object("fabric_stone%d" % i, bm)
        o.parent = root
    return root


# ---- Hair ------------------------------------------------------------------------------------------------------------------------------
# (Rebuilt in the hair pass, 2026-10-24, the owner: "the hair currently reads as a spiky hat". It was a rigid cap standing off the head
# with a hard rim and separate cones stuck on. Now each style is one soft mass hugging the head down to the nape, its locks growing out of
# it: the mass from metaballs laid just under the measured head's surface, the locks as tapered curved cones, all fused into one surface
# by a voxel remesh, its inside (within the head) cut away, then brought down to the budget.)

# The fighter's head in this design space (polar angle from the top in degrees, radius), measured off the rig: a ball of 0.8 above its
# middle, widening below it into the jaw and neck.
HEAD_PROFILE = [(0, 0.8), (30, 0.79), (50, 0.77), (70, 0.775), (90, 0.79), (100, 0.82), (110, 0.87), (120, 0.9), (130, 0.95), (140, 1.0)]
# How far under the head's surface the mass's metaballs are centred, how far its surface stands off the head, the remesh's voxel size,
# and the triangle budget.
HAIR_SINK = 0.13
HAIR_DEPTH = 0.055
HAIR_VOXEL = 0.022
HAIR_BUDGET = 3000


def head_surface(pol):
    for (p0, r0), (p1, r1) in zip(HEAD_PROFILE, HEAD_PROFILE[1:]):
        if p0 <= pol <= p1:
            return r0 + (r1 - r0) * (pol - p0) / (p1 - p0)
    return HEAD_PROFILE[-1][1]


def hair_dir(polar_deg, azimuth_deg):
    """A direction from the head's middle: `polar` from the top, `azimuth` round from the face (0 front, 90 the right side, 180 back)."""
    p = math.radians(polar_deg)
    a = math.radians(azimuth_deg)
    return Vector((math.sin(p) * math.sin(a), -math.sin(p) * math.cos(a), math.cos(p)))


def hair_root(polar_deg, azimuth_deg, sink=0.02):
    """A point on the hair's surface, `sink` under it (where a lock is rooted, so it grows out of the mass)."""
    return HEAD_C + hair_dir(polar_deg, azimuth_deg) * (head_surface(polar_deg) + HAIR_DEPTH - sink)


def _hairline(az, front, side, back):
    a = abs(((az + 180.0) % 360.0) - 180.0)   # 0 front .. 180 back
    t = a / 90.0 if a <= 90.0 else (a - 90.0) / 90.0
    s = t * t * (3 - 2 * t)
    return front + (side - front) * s if a <= 90.0 else side + (back - side) * s


def hair_mass(front, side, back, r=0.24, step=8.0):
    """The hair's mass: metaballs over its area (down to `front` degrees from the top at the face, `side` over the ears, `back` at the
    nape), close enough to merge into one smooth shell HAIR_DEPTH off the head, a little thicker toward the hairline and below the head's
    middle (thin there, the skin broke through its lower edge)."""
    mb = bpy.data.metaballs.new("hair_mass")
    mb.resolution = 0.045
    mb.threshold = 0.6
    obj = bpy.data.objects.new("hair_mass", mb)
    bpy.context.scene.collection.objects.link(obj)
    pol = 2.0
    while pol <= 125.0:
        n = max(1, int(360.0 * math.sin(math.radians(pol)) / step + 0.5))
        for i in range(n):
            az = 360.0 * (i + 0.5 * (int(pol / step) % 2)) / n
            lim = _hairline(az, front, side, back)
            if pol > lim:
                continue
            k = max(0.7, min(1.0, (lim - pol) / 16.0))
            low = max(0.0, min(1.0, (pol - 80.0) / 40.0))
            e = mb.elements.new()
            e.co = HEAD_C + hair_dir(pol, az) * (head_surface(pol) - HAIR_SINK + HAIR_DEPTH + (1.0 - k) * 0.06 + 0.06 * low)
            e.radius = r * k
            e.stiffness = 2.0
        pol += step
    return mb, obj


def hair_balls(mb, centres, radius, stiffness=2.4):
    for c in centres:
        e = mb.elements.new()
        e.co = c
        e.radius = radius
        e.stiffness = stiffness


def spike(bm, root, direction, length, width, bend=(0.0, 0.0, 0.0), flat=0.55, rings=10, sides=9, tip=0.008, across=None):
    """A lock of hair as a tapered cone: from `root` out along `direction` for `length`, curving by `bend` (an offset reached at the tip,
    growing with the square of the distance), `width` across at the root and `flat` of that through its thickness, narrowing to a point.
    Its width runs along `across` (by default level, square to the lock)."""
    d = Vector(direction).normalized()
    side = Vector(across) if across is not None else d.cross(Vector((0, 0, 1)))
    if side.length < 0.01:
        side = Vector((1, 0, 0))
    side = (side - d * side.dot(d)).normalized()
    up = side.cross(d).normalized()
    bend = Vector(bend)
    rings_v = []
    for k in range(rings):
        t = k / (rings - 1)
        c = root + d * length * t + bend * t * t
        # (Full through its first quarter, then narrowing to the point: a chunky lock, not a needle.)
        w = width if t <= 0.25 else max(tip, width * (1.0 - (t - 0.25) / 0.75) ** 1.25)
        ring = [bm.verts.new(c + side * math.cos(a) * w + up * math.sin(a) * w * flat)
                for a in (math.tau * i / sides for i in range(sides))]
        rings_v.append(ring)
    for k in range(rings - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings_v[k][i], rings_v[k][j], rings_v[k + 1][j], rings_v[k + 1][i]))
    bm.faces.new(rings_v[0])
    tip_v = bm.verts.new(root + d * length * 1.03 + bend)
    for i in range(sides):
        bm.faces.new((rings_v[-1][i], rings_v[-1][(i + 1) % sides], tip_v))
    return bm


def _mesh_obj(name, bm):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def hair_finish(name, objs, budget=None):
    """Fuses the mass and the locks into one surface (a voxel remesh, smoothed), cuts away its inside (within the head), and brings it
    down to the budget, smooth-shaded."""
    dg = bpy.context.evaluated_depsgraph_get()
    bm = bmesh.new()
    for o in objs:
        me = bpy.data.meshes.new_from_object(o.evaluated_get(dg))
        me.transform(o.matrix_world)
        bm.from_mesh(me)
        bpy.data.meshes.remove(me)
    for o in objs:
        bpy.data.objects.remove(o)
    obj = _mesh_obj(name, bm)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    rm = obj.modifiers.new("remesh", "REMESH")
    rm.mode = "VOXEL"
    rm.voxel_size = HAIR_VOXEL
    bpy.ops.object.modifier_apply(modifier="remesh")
    sm = obj.modifiers.new("smooth", "SMOOTH")
    sm.factor = 0.5
    sm.iterations = 2
    bpy.ops.object.modifier_apply(modifier="smooth")
    bm = bmesh.new()
    bm.from_mesh(obj.data)

    def inside(v):
        rel = v.co - HEAD_C
        pol = math.degrees(math.acos(max(-1.0, min(1.0, rel.normalized().z))))
        return rel.length < head_surface(pol) - 0.03
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if all(inside(v) for v in f.verts)], context="FACES")
    bm.to_mesh(obj.data)
    bm.free()
    tris = sum(len(p.vertices) - 2 for p in obj.data.polygons)
    budget = budget or HAIR_BUDGET
    if tris > budget:
        dec = obj.modifiers.new("dec", "DECIMATE")
        dec.ratio = budget / tris
        bpy.ops.object.modifier_apply(modifier="dec")
    obj.data.materials.append(material("m_hair"))
    for p in obj.data.polygons:
        p.use_smooth = True
    return obj


def swept_spikes():
    """Big anime-style spiky hair: long chunky spikes sweeping up and back from the crown, a heavy fringe of long spikes falling over the
    brow (stopping above the eyes), and spikes flaring out over the ears and down the back, all out of one mass."""
    root = empty("swept_spikes")
    # (The mass down to 112 degrees at the nape, 124 until the nape pass, 2026-10-29: see the tufts below.)
    mb, mass = hair_mass(front=50.0, side=96.0, back=112.0)
    bm = bmesh.new()
    # The crown: four big spikes swept up, back and out, the middle two tallest (about a head's radius and a half).
    for az, ln in ((-48, 1.0), (-16, 1.18), (16, 1.18), (48, 1.0)):
        sx = math.sin(math.radians(az))
        spike(bm, hair_root(16.0, az, sink=0.14), Vector((sx * 0.55, 0.45, 1.0)), ln, 0.3, bend=(sx * 0.2, 0.22, -0.06))
    # Behind them, a row fanning back and up, a lower row swept out and up over the back of the head, and the nape in tapered tufts
    # lying down the head. (The lower row hung down, and the mass ended in a level hem, until the nape pass, 2026-10-29, the owner's pick
    # of mocks: from behind it read as a helmet's rim with teeth hanging from it.)
    for az in (140, 180, 220):
        spike(bm, hair_root(42.0, az, sink=0.14), hair_dir(55.0, az) + Vector((0, 0, 0.5)), 0.95, 0.29, bend=(0.0, 0.0, -0.1))
    for az in (120, 160, 200, 240):
        spike(bm, hair_root(70.0, az, sink=0.14), hair_dir(80.0, az) + Vector((0, 0, 0.25)), 0.7, 0.26, bend=(0.0, 0.0, 0.08))
    for az, ln in ((135, 0.32), (158, 0.4), (180, 0.44), (202, 0.4), (225, 0.32)):
        spike(bm, hair_root(110.0, float(az), sink=0.1), hair_dir(130.0, az) * 0.35 + Vector((0, 0, -1.0)), ln, 0.2, flat=0.4)
    for sx in (-1.0, 1.0):
        # Over the ears: a big spike each side flaring out, up a little and back.
        spike(bm, hair_root(56.0, 96.0 * sx, sink=0.14), Vector((sx * 1.0, 0.3, 0.45)), 0.85, 0.27, bend=(0.0, 0.12, -0.12))
        # And a long lock down the side of the face, in front of the ear.
        spike(bm, hair_root(70.0, 70.0 * sx, sink=0.1), Vector((sx * 0.35, -0.25, -1.0)), 0.55, 0.17, bend=(sx * 0.06, -0.04, 0.0),
              across=(0, 1, 0))
    # The fringe: three long bangs of unequal length, the middle one falling to a point between the eyes and curling aside, the outer two
    # swept out over the brows, and two short locks between them so no edge of the mass shows.
    # (Five equal spikes ending level above the eyes until the fringe pass, 2026-10-28, the owner's pick of mocks: from the front they read
    # as a jagged band across the brow, a cut-paper crown.)
    spike(bm, hair_root(36.0, 4.0, sink=0.06), Vector((0.1, -0.75, -0.85)), 0.6, 0.25, bend=(0.1, -0.07, 0.0), flat=0.45)
    spike(bm, hair_root(40.0, -30.0, sink=0.06), Vector((-0.75, -0.7, -0.6)), 0.55, 0.23, bend=(-0.08, -0.04, 0.03), flat=0.45)
    spike(bm, hair_root(40.0, 32.0, sink=0.06), Vector((0.8, -0.65, -0.5)), 0.48, 0.21, bend=(0.06, -0.03, 0.04), flat=0.45)
    for az in (-14.0, 18.0):
        spike(bm, hair_root(46.0, az, sink=0.06), Vector((0.0, -0.9, -0.6)), 0.3, 0.17, flat=0.5)
    locks = _mesh_obj("hair_locks", bm)
    hair_finish("hair", [mass, locks]).parent = root
    return root


def swoop():
    """A short soft cut down to the nape, one big forelock swept up and over to the side (a quiff), a soft tuft over each ear."""
    root = empty("swoop")
    mb, mass = hair_mass(front=58.0, side=96.0, back=126.0)

    def lock(start_pol, start_az, c1, c2, r0, r1, n):
        base = HEAD_C + hair_dir(start_pol, start_az) * (head_surface(start_pol) + 0.02)
        for k in range(n):
            t = k / (n - 1)
            hair_balls(mb, [base + Vector(c1) * (2 * t * (1 - t)) + Vector(c2) * (t * t)], r0 + (r1 - r0) * t)
    lock(48.0, -10.0, (0.1, -0.22, 0.22), (0.34, -0.12, 0.3), 0.22, 0.1, 7)
    lock(44.0, 15.0, (0.16, -0.16, 0.18), (0.42, -0.04, 0.2), 0.18, 0.08, 6)
    for az in (-94.0, 94.0):
        lock(88.0, az, (0.0, 0.0, -0.06), (0.0, 0.04, -0.14), 0.12, 0.07, 3)
    hair_finish("hair", [mass]).parent = root
    return root


def topknot():
    """Hair drawn back smooth from the hairline into a round knot on top, tied with a band of the outfit's colour, a loose lock at each
    temple."""
    root = empty("topknot")
    mb, mass = hair_mass(front=54.0, side=96.0, back=126.0)
    knot = HEAD_C + hair_dir(12.0, 180.0) * (head_surface(12.0) + 0.16)
    hair_balls(mb, [knot + Vector((0.08 * math.cos(a), 0.08 * math.sin(a), 0.03 * k))
                    for k, a in enumerate(math.tau * i / 3.0 for i in range(3))], 0.24)
    for az in (-62.0, 62.0):
        base = HEAD_C + hair_dir(60.0, az) * (head_surface(60.0) + 0.02)
        hair_balls(mb, [base + Vector((0.0, -0.03, -0.12)) * t + Vector((0.04 * (1 if az > 0 else -1), -0.05, -0.26)) * t * t
                        for t in (0.0, 0.33, 0.66, 1.0)], 0.07)
    hair_finish("hair", [mass]).parent = root
    # The band where the knot is tied.
    bm = bmesh.new()
    kz = knot.z - 0.12
    rows = lathe(bm, [(0.17, kz - 0.03), (0.19, kz - 0.01), (0.19, kz + 0.03), (0.17, kz + 0.05)], sides=24)
    for ring in rows:
        for v in ring:
            v.co.y += knot.y
    to_object("fabric_band", bm, sharp=60.0).parent = root
    return root


# (Pulled down over the head, the beanie pass, 2026-10-19, as the helm: centred at 1.74 with a 0.68 rise it sat on top of the head as a
# flat bowl.)
BEANIE_C = 1.52
BEANIE_H = 0.98


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
            # (Deeper ribs, the beanie pass, 2026-10-19: at 0.012 the knit did not show and it read as a smooth plastic bowl.)
            rib = 1.0 + 0.028 * math.cos(a * 28.0) * min(1.0, t * 3.0)
            r = 0.9 * rib
            row.append(bm.verts.new((r * math.sin(ph) * math.cos(a), r * math.sin(ph) * math.sin(a), BEANIE_C + BEANIE_H * math.cos(ph))))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, BEANIE_C + BEANIE_H))
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
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), BEANIE_C + BEANIE_H * math.cos(ph))))
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
        bmesh.ops.translate(bm, vec=Vector((0.0, 0.0, BEANIE_C + BEANIE_H + 0.08)) + off, verts=bm.verts)
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
    # (Fine wire, the helm pass, 2026-10-19: at 0.05 wide and 0.035 deep the rims read as plastic donuts, and the far one, edge-on at
    # three-quarters, as a bar sticking out past the head.)
    _frame_loop(bm, [(0.19 * math.cos(t), 0.19 * math.sin(t)) for t in [math.tau * i / 48 for i in range(48)]], 0.022, 0.018)
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


# ---- Hood and cape ---------------------------------------------------------------------------------------------------------------------
# (Modelled in the cape and hood pass, 2026-10-25, the owner's picks of mocked options: the hood was a stiff open shell with a flat rim and
# a cone for a point, the cape a flat panel hanging behind the back that barely showed.)

def hood():
    """A soft hood (in the head's design space, as the hair): a loose cloth shell over the head round an open face, a soft peak over the
    brow, a drooping point behind, a cowl falling round the neck, and a thick rolled rim of a lighter lining framing the face. The shell
    and the cowl are one surface (metaballs fused by a voxel remesh, its inside cut away); the rim its own part."""
    root = empty("hood")
    loose, face_w, face_h, face_c, point = 0.07, 70.0, 50.0, 92.0 + 2.0, 0.8

    def in_face(pol, az, grow):
        a = ((az + 180.0) % 360.0) - 180.0
        return (a / (face_w + grow)) ** 2 + ((pol - face_c) / (face_h + grow)) ** 2 < 1.0
    mb = bpy.data.metaballs.new("hoodshell")
    mb.resolution = 0.045
    mb.threshold = 0.6
    shell = bpy.data.objects.new("hoodshell", mb)
    bpy.context.scene.collection.objects.link(shell)
    pol = 2.0
    while pol <= 132.0:
        n = max(1, int(360.0 * math.sin(math.radians(pol)) / 8.0 + 0.5))
        for i in range(n):
            az = 360.0 * (i + 0.5 * (int(pol / 8.0) % 2)) / n
            if not in_face(pol, az, 6.0):
                hair_balls(mb, [HEAD_C + hair_dir(pol, az) * (head_surface(pol) - 0.13 + loose)], 0.24, 2.0)
        pol += 8.0
    # The cowl: close round the neck, only a little wider at its foot (a wide ring read as a rubber float), lower behind.
    for z, r in ((0.9, 0.6), (0.82, 0.64), (0.74, 0.69), (0.66, 0.74)):
        for i in range(36):
            a = math.tau * i / 36
            hair_balls(mb, [Vector((r * math.sin(a), -r * math.cos(a) * 0.92, z - 0.05 * (0.5 - 0.5 * math.cos(a))))], 0.16, 2.0)
    # The point: out back and drooping, tapering (what keeps the hood from reading as a helmet).
    for k in range(15):
        t = k / 14.0
        p = HEAD_C + hair_dir(34.0, 180.0) * (head_surface(34.0) + loose - 0.04) + Vector((0.0, point * t, -point * 0.8 * t * t))
        hair_balls(mb, [p], 0.22 * (1.0 - 0.42 * t), 2.0)
    # A soft peak over the brow, the cloth gathered forward at the top of the opening.
    for k in range(4):
        t = k / 3.0
        p = HEAD_C + hair_dir(face_c - face_h - 2.0, 0.0) * (head_surface(40.0) + loose + 0.04 * t) + Vector((0.0, -0.05 * t, 0.03 * t))
        hair_balls(mb, [p], 0.2 - 0.03 * t, 2.0)
    # The rolled rim on the shell's edge round the face.
    rmb = bpy.data.metaballs.new("hoodrim")
    rmb.resolution = 0.04
    rmb.threshold = 0.6
    rim = bpy.data.objects.new("hoodrim", rmb)
    bpy.context.scene.collection.objects.link(rim)
    for i in range(72):
        t = math.tau * i / 72
        az = (face_w + 5.0) * math.sin(t)
        pol = face_c + (face_h + 5.0) * math.cos(t)
        hair_balls(rmb, [HEAD_C + hair_dir(pol, az) * (head_surface(pol) + loose)], 0.17, 2.0)
    hair_finish("hood_cloth", [shell]).parent = root
    hair_finish("lining_rim", [rim], budget=1000).parent = root
    for o in root.children:
        o.data.materials.clear()
        o.data.materials.append(material("m_" + o.name.split("_")[0]))
    return root


# The cape's shape: from the collar to the calves (heights in the rig's space), how far round the shoulders it wraps at the top and lower
# down (degrees from straight back), its half widths there, its folds, how much shorter its sides are than its middle, how far it stands
# off the body, and how far it flares back toward the hem.
CAPE = {"top": 1.53, "hem": 0.42, "wrap_top": 118.0, "wrap_low": 62.0, "w_top": 0.3, "w_low": 0.66, "folds": 4, "fold_amp": 0.03,
        "hem_curve": 0.08, "gap": 0.04, "flare": 0.16, "rows": 26, "cols": 28, "thick": 0.028}
RIG = "godot/models/base_rig.glb"


def cape():
    """A long cape (in the rig's own space, not the head's: it hangs from the shoulders): laid round the back of the neck and over the
    shoulders, kept off the body by pushing it out of the fighter's own mesh there, then falling straight from the shoulder blades and
    flaring back to the calves, with soft folds and a rounded hem. The game hangs it on a chain of bones down its back that swings
    (fighter_view.gd `CapeRig`)."""
    from mathutils import bvhtree
    bpy.ops.import_scene.gltf(filepath=os.path.abspath(RIG))
    body = bmesh.new()
    for o in bpy.data.objects:
        if o.type != "MESH" or not o.name.startswith(("Skin", "Shirt", "Shorts", "Belt")) or o.name.startswith("ShirtCampCollar"):
            continue
        me = o.data.copy()
        me.transform(o.matrix_world)
        part = bmesh.new()
        part.from_mesh(me)
        # (No arms: their faces stand out to the sides below the shoulders.)
        bmesh.ops.delete(part, geom=[f for f in part.faces if any(abs(v.co.x) > 0.47 and v.co.z < 1.4 for v in f.verts)],
                         context="FACES")
        tmp = bpy.data.meshes.new("tmp")
        part.to_mesh(tmp)
        part.free()
        body.from_mesh(tmp)
        bpy.data.meshes.remove(tmp)
        bpy.data.meshes.remove(me)
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    body.normal_update()
    tree = bvhtree.BVHTree.FromBMesh(body)
    st = CAPE

    def back_depth(z, half=0.35):
        ys = [v.co.y for v in body.verts if abs(v.co.z - z) < 0.035 and abs(v.co.x) < half]
        return max(ys) if ys else 0.3

    def smooth(a, b, t):
        t = max(0.0, min(1.0, (t - a) / (b - a)))
        return t * t * (3 - 2 * t)
    neck = [v.co for v in body.verts if abs(v.co.z - st["top"]) < 0.03 and abs(v.co.x) < 0.4]
    yc_top = (min(c.y for c in neck) + max(c.y for c in neck)) / 2.0
    blades = max(back_depth(z) for z in (1.3, 1.25, 1.2))
    rows, cols = st["rows"], st["cols"]
    grid = []
    for j in range(rows + 1):
        v = j / rows
        row = []
        for i in range(cols + 1):
            u = -1.0 + 2.0 * i / cols
            vv = v * (1.0 - st["hem_curve"] * u * u)
            z = st["top"] - (st["top"] - st["hem"]) * vv
            s = smooth(0.0, 0.35, vv)
            wrap = math.radians(st["wrap_top"] + (st["wrap_low"] - st["wrap_top"]) * s)
            th = u * wrap
            back = blades + st["gap"] + st["flare"] * smooth(0.25, 1.0, vv) ** 1.5
            w = st["w_low"] * (0.8 + 0.2 * vv)
            rx = st["w_top"] + (w / max(0.3, math.sin(wrap)) - st["w_top"]) * s
            ry = 0.27 + (0.34 - 0.27) * s
            yc = yc_top + ((back - 0.34) - yc_top) * s
            p = Vector((rx * math.sin(th), yc + ry * math.cos(th), z))
            p += Vector((math.sin(th), math.cos(th), 0.0)) * st["fold_amp"] * smooth(0.15, 0.9, vv) * math.sin(
                st["folds"] * math.pi * (u + 1.0) - math.pi / 2)
            row.append(p)
        grid.append(row)
    # Off the body round the shoulders and upper back (below them it hangs free: pushed off everywhere it clung in crumples).
    for it in range(6):
        for j in range(rows + 1):
            for i in range(cols + 1):
                p = grid[j][i]
                if p.z < 1.18:
                    continue
                loc, nrm, _, _ = tree.find_nearest(p)
                if loc is not None and (p - loc).dot(nrm) < st["gap"]:
                    grid[j][i] = loc + nrm * st["gap"]
        if it < 5:
            new = [r[:] for r in grid]
            for j in range(1, rows):
                for i in range(1, cols):
                    new[j][i] = (grid[j][i] * 4 + grid[j - 1][i] + grid[j + 1][i] + grid[j][i - 1] + grid[j][i + 1]) / 8.0
            grid = new
    body.free()
    bm = bmesh.new()
    vs = [[bm.verts.new(p) for p in r] for r in grid]
    for j in range(rows):
        for i in range(cols):
            bm.faces.new((vs[j][i], vs[j][i + 1], vs[j + 1][i + 1], vs[j + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=st["thick"])
    root = empty("cape")
    to_object("fabric_cape", bm, sharp=80.0).parent = root
    return root


GEAR = {"neckwear": neckwear, "glasses": glasses, "sailor_cap": sailor_cap, "dixie_cap": dixie_cap, "aviator_cap": aviator_cap, "straw_hat": straw_hat, "plumed_helm": plumed_helm, "crown": crown,
        "swept_spikes": swept_spikes, "topknot": topknot, "swoop": swoop, "beanie": beanie, "hood": hood,
        "cape": cape}


def _lumpy(root, amount=0.045, seed=1.0):
    """The hand-made pass (the character judge's round 14, after the reference's soft lumpy shapes): every crown and shell of a hat
    pushed in and out along its normals by a slow smooth noise, soft dents rather than a machined surface."""
    for c in root.children_recursive:
        if c.type != "MESH" or not any(k in c.name for k in ("crown", "shell")):
            continue
        me = c.data
        lo = min(v.co.z for v in me.vertices)
        hi = max(v.co.z for v in me.vertices)
        for v in me.vertices:
            p = v.co
            n = (math.sin(p.x * 3.1 + seed) * math.sin(p.y * 2.7 + 1.3 * seed) + 0.6 * math.sin(p.z * 4.3 + p.x * 1.7 + 2.0 * seed)) / 1.6
            # (None at the rim, so the band, trim or fleece seated there stays seated.)
            up = max(0.0, min(1.0, ((p.z - lo) / max(hi - lo, 1e-4) - 0.35) / 0.3))
            v.co = p + v.normal * amount * n * up


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
        # (Shallow on the aviator's leather, round 17: deep dents read as blotches on its dark dome.)
        _lumpy(root, amount=0.015 if name == "aviator_cap" else 0.045, seed=float(len(name)))
        export(root, os.path.join(OUT, name + ".glb"))
        tris = sum(sum(len(p.vertices) - 2 for p in c.data.polygons) for c in root.children_recursive if c.type == "MESH")
        print("gear %s: %d triangles" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
