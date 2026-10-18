"""Stage props modelled by script, in the game's style: chunky, slightly hand-made shapes (a little taper, a little wobble, soft bevels),
one flat colour per part, so `Toonify.load_prop` turns each part into the cel material of its colour with the ink outline. Original
designs; no reference geometry.

Run:  blender --background --python art/blender/make_props.py [-- --only lighthouse,sailboat,mesa_a,mesa_b] [--preview <folder>]
Writes godot/models/props/<name>.glb for each prop (then `Godot --headless --path godot --import`).

Units: one Blender unit is one game unit; z up in Blender (exported y up). Each prop stands on the origin.
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

OUT = "godot/models/props"

# The palette, as the colours should look in the game (sRGB); `material` turns them into the linear values Blender and glTF keep, so the
# game reads back exactly these. The game's cel material and stage light do the rest.
WHITE = (0.95, 0.93, 0.88)
RED = (0.82, 0.26, 0.22)
SLATE = (0.28, 0.3, 0.38)
GLASS = (1.0, 0.92, 0.62)
ROCK = (0.56, 0.54, 0.6)
ROCK_DARK = (0.42, 0.4, 0.48)
WOOD = (0.62, 0.4, 0.24)
WOOD_LIGHT = (0.78, 0.56, 0.34)
SAIL = (0.97, 0.95, 0.9)
SAND_A = (0.78, 0.4, 0.3)
SAND_B = (0.86, 0.52, 0.36)
SAND_C = (0.7, 0.33, 0.28)
CACTUS = (0.3, 0.6, 0.36)
PINK = (0.98, 0.55, 0.7)
PALM = (0.22, 0.6, 0.3)
COCONUT = (0.45, 0.3, 0.2)


def material(name, colour):
    m = bpy.data.materials.get(name)
    colour = tuple(c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in colour)
    if m is None:
        m = bpy.data.materials.new(name)
        m.use_nodes = True
        bsdf = m.node_tree.nodes.get("Principled BSDF")
        bsdf.inputs["Base Color"].default_value = (*colour, 1.0)
        bsdf.inputs["Roughness"].default_value = 1.0
        m.diffuse_color = (*colour, 1.0)
    return m


def to_object(name, bm, colour, smooth=True, bevel=0.0, parent=None):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    obj.data.materials.append(material("m_%s_%.2f_%.2f_%.2f" % ((name,) + tuple(colour)), colour))
    if bevel > 0.0:
        mod = obj.modifiers.new("bevel", "BEVEL")
        mod.width = bevel
        mod.segments = 2
        mod.limit_method = "ANGLE"
    for p in obj.data.polygons:
        p.use_smooth = smooth
    if smooth:
        # Smooth, but rims and caps keep a crisp edge.
        obj.data.set_sharp_from_angle(angle=math.radians(40))
    if parent is not None:
        obj.parent = parent
    return obj


def cone(bm, r_bottom, r_top, height, z0=0.0, sides=16, at=(0.0, 0.0), wobble=0.0, rng=None):
    """A tapered cylinder standing at height z0, its rim points pushed in or out by up to `wobble` (hand-made, not machined)."""
    ret = bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=sides, radius1=r_bottom, radius2=r_top, depth=height)
    verts = ret["verts"]
    for v in verts:
        if wobble > 0.0 and rng is not None:
            k = 1.0 + rng.uniform(-wobble, wobble)
            v.co.x *= k
            v.co.y *= k
        v.co.z += z0 + height / 2.0
        v.co.x += at[0]
        v.co.y += at[1]
    return verts


def blob(bm, radius, centre, scale=(1.0, 1.0, 1.0), subdiv=2, jitter=0.0, rng=None):
    """A lumpy rock: an icosphere, squashed by `scale`, each point pushed in or out by up to `jitter`."""
    ret = bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=radius)
    for v in ret["verts"]:
        if jitter > 0.0 and rng is not None:
            v.co *= 1.0 + rng.uniform(-jitter, jitter)
        v.co = Vector((v.co.x * scale[0], v.co.y * scale[1], v.co.z * scale[2])) + Vector(centre)
    return ret["verts"]


def ring(bm, radius, thickness, z, sides=24, minor_sides=8):
    """A rounded hoop (a torus) at height z."""
    verts = []
    rows = []
    for i in range(sides):
        a = math.tau * i / sides
        row = []
        for j in range(minor_sides):
            b = math.tau * j / minor_sides
            r = radius + thickness * math.cos(b)
            row.append(bm.verts.new((r * math.cos(a), r * math.sin(a), z + thickness * math.sin(b))))
        rows.append(row)
    for i in range(sides):
        for j in range(minor_sides):
            a, b = rows[i][j], rows[(i + 1) % sides][j]
            c, d = rows[(i + 1) % sides][(j + 1) % minor_sides], rows[i][(j + 1) % minor_sides]
            bm.faces.new((a, b, c, d))
    return verts


def empty(name):
    e = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(e)
    return e


def ribbed_tube(bm, points, radius, ribs=8, rib_depth=0.12, sides=16, per_span=4):
    """A ribbed stem along `points` (a cactus trunk or arm): its cross-section a circle with `ribs` soft ridges, the radius easing to a
    rounded dome at the top end. Returns nothing; the faces are added to `bm`."""
    pts = [Vector(q) for q in points]
    path = []
    for i in range(len(pts) - 1):
        for k in range(per_span):
            t = k / per_span
            # A gentle curve through the points (Catmull-Rom).
            p0 = pts[max(i - 1, 0)]
            p1, p2 = pts[i], pts[i + 1]
            p3 = pts[min(i + 2, len(pts) - 1)]
            path.append(0.5 * ((2 * p1) + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t + (-p0 + 3 * p1 - 3 * p2 + p3) * t * t * t))
    path.append(pts[-1])
    n = len(path)
    dome = 4
    rings = []
    # The cross-section's frame is carried along the path (each step's side turned only as much as the path turns), so the ribs run
    # straight round a bend instead of twisting.
    side = (path[1] - path[0]).normalized().orthogonal().normalized()
    for i in range(n + dome):
        if i < n:
            c = path[i]
            d = (path[min(i + 1, n - 1)] - path[max(i - 1, 0)]).normalized()
            r = radius
        else:
            k = (i - n + 1) / dome
            d = (path[-1] - path[-2]).normalized()
            c = path[-1] + d * radius * math.sin(k * math.pi / 2) * 0.9
            r = radius * math.cos(k * math.pi / 2 * 0.92)
        side = (side - d * side.dot(d)).normalized()
        up = d.cross(side).normalized()
        ring_v = []
        for j in range(sides):
            a = math.tau * j / sides
            rr = r * (1.0 + rib_depth * math.cos(a * ribs))
            ring_v.append(bm.verts.new(c + (side * math.cos(a) + up * math.sin(a)) * rr))
        rings.append(ring_v)
    for i in range(len(rings) - 1):
        for j in range(sides):
            bm.faces.new((rings[i][j], rings[i][(j + 1) % sides], rings[i + 1][(j + 1) % sides], rings[i + 1][j]))
    bm.faces.new(list(reversed(rings[0])))
    tip = bm.verts.new(path[-1] + (path[-1] - path[-2]).normalized() * radius * 0.95)
    for j in range(sides):
        bm.faces.new((rings[-1][j], rings[-1][(j + 1) % sides], tip))


def frond(bm, angle, droop, length, width, rng):
    """One palm frond from the crown: a midrib arching out and down at `angle` about the vertical, its leaf a strip on either side cut
    into leaflets (a zigzag edge), widest a third of the way out, slightly folded along the rib."""
    steps = 12
    out = Vector((math.cos(angle), math.sin(angle), 0.0))
    side = Vector((-out.y, out.x, 0.0))
    rib = []
    for i in range(steps + 1):
        t = i / steps
        rib.append(out * length * t + Vector((0, 0, length * (0.75 * t - droop * t * t))))
    for sgn in (-1.0, 1.0):
        prev_in = None
        prev_out = None
        for i in range(steps + 1):
            t = i / steps
            w = width * math.sin(math.pi * min(1.0, t * 1.3)) * (1.0 - 0.3 * t)
            notch = 0.72 if i % 2 else 1.0
            vin = bm.verts.new(rib[i])
            vout = bm.verts.new(rib[i] + side * sgn * w * notch + Vector((0, 0, -0.12 * w)))
            if prev_in is not None:
                f = (prev_in, vin, vout, prev_out) if sgn > 0 else (prev_in, prev_out, vout, vin)
                bm.faces.new(f)
            prev_in, prev_out = vin, vout


def cactus():
    """A saguaro: a ribbed trunk with two arms curving up from it at different heights, rounded tops, a pink flower on the crown.
    About 4.5 tall."""
    root = empty("cactus")
    bm = bmesh.new()
    ribbed_tube(bm, [(0, 0, -0.2), (0, 0, 1.6), (0.02, 0, 3.2), (0, 0, 4.1)], 0.48)
    ribbed_tube(bm, [(-0.3, 0, 1.4), (-0.95, 0, 1.55), (-1.2, 0, 2.1), (-1.2, 0, 2.9)], 0.3)
    ribbed_tube(bm, [(0.3, 0, 2.1), (0.85, 0, 2.2), (1.05, 0, 2.7), (1.08, 0, 3.4)], 0.27)
    to_object("Cactus", bm, CACTUS, parent=root)
    bm = bmesh.new()
    for k in range(5):
        a = math.tau * k / 5
        blob(bm, 0.11, (math.cos(a) * 0.13, math.sin(a) * 0.13, 4.6), (1.0, 1.0, 0.55), 1)
    to_object("Flower", bm, PINK, parent=root)
    return root


def palm_crown():
    """A palm's crown (the trunk is built in the game, so the crown can sway on it): eight arching fronds cut into leaflets round a few
    coconuts. About 5 across."""
    rng = random.Random(4)
    root = empty("palm_crown")
    bm = bmesh.new()
    for k in range(8):
        a = math.tau * k / 8 + rng.uniform(-0.15, 0.15)
        frond(bm, a, rng.uniform(1.0, 1.25), rng.uniform(2.4, 2.9), rng.uniform(0.8, 0.95), rng)
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.03)
    to_object("Fronds", bm, PALM, parent=root)
    bm = bmesh.new()
    for k in range(3):
        a = math.tau * k / 3 + 0.4
        blob(bm, 0.2, (math.cos(a) * 0.22, math.sin(a) * 0.22, -0.22), (1.0, 1.0, 1.1), 2)
    to_object("Coconuts", bm, COCONUT, parent=root)
    return root


# ---- The props ----------------------------------------------------------------------------------------------------------------------

def lighthouse():
    """A stout lighthouse on a heap of rocks: a white tower tapering a little, two red bands, a dark gallery with a railing, a glowing
    lantern room with mullions, a red cap and a ball on top, a door and two little windows. About 11 units tall."""
    rng = random.Random(5)
    root = empty("lighthouse")
    bm = bmesh.new()
    for k in range(9):
        a = math.tau * k / 9 + rng.uniform(-0.2, 0.2)
        r = rng.uniform(1.5, 2.1)
        blob(bm, rng.uniform(0.55, 0.9), (math.cos(a) * r, math.sin(a) * r, 0.15), (1.2, 1.0, 0.7), 1, 0.18, rng)
    blob(bm, 1.9, (0, 0, -0.5), (1.15, 1.15, 0.55), 2, 0.1, rng)
    to_object("Rocks", bm, ROCK, smooth=False, parent=root)
    # The tower, in three bands (white, red, white, red, white), so each band is its own colour; slightly wobbly.
    bands = [(0.4, 2.2, WHITE), (2.2, 3.1, RED), (3.1, 5.3, WHITE), (5.3, 6.2, RED), (6.2, 7.6, WHITE)]
    r_at = lambda z: 1.25 - 0.045 * z
    for i, (z0, z1, colour) in enumerate(bands):
        bm = bmesh.new()
        grow = 0.03 if colour == RED else 0.0
        cone(bm, r_at(z0) + grow, r_at(z1) + grow, z1 - z0, z0, sides=18, wobble=0.012, rng=random.Random(11 + i))
        to_object("Tower%d" % i, bm, colour, bevel=0.04, parent=root)
    # The door and windows: dark insets on the front (-y).
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((v.co.x * 0.55, v.co.y * 0.3, v.co.z * 1.0 + 0.5 + 0.4)) + Vector((0, -r_at(0.9) + 0.05, 0))
    for z in (3.9, 6.75):
        ret = bmesh.ops.create_cube(bm, size=1.0)
        for v in ret["verts"]:
            v.co = Vector((v.co.x * 0.32, v.co.y * 0.3, v.co.z * 0.46 + z)) + Vector((0, -r_at(z) + 0.06, 0))
    to_object("Openings", bm, SLATE, smooth=False, bevel=0.03, parent=root)
    # The gallery: a wide dark deck, a railing of posts and a hoop.
    bm = bmesh.new()
    cone(bm, 1.25, 1.35, 0.22, 7.6, sides=20)
    to_object("Gallery", bm, SLATE, bevel=0.04, parent=root)
    bm = bmesh.new()
    for k in range(16):
        a = math.tau * k / 16
        cone(bm, 0.035, 0.035, 0.5, 7.82, sides=6, at=(math.cos(a) * 1.25, math.sin(a) * 1.25))
    ring(bm, 1.25, 0.045, 8.32)
    to_object("Railing", bm, SLATE, parent=root)
    # The lantern room: glowing glass behind dark mullions, a red cap and a ball.
    bm = bmesh.new()
    cone(bm, 0.72, 0.72, 1.0, 7.82, sides=16)
    to_object("Lantern", bm, GLASS, parent=root)
    bm = bmesh.new()
    for k in range(8):
        a = math.tau * k / 8
        cone(bm, 0.05, 0.05, 1.0, 7.82, sides=6, at=(math.cos(a) * 0.74, math.sin(a) * 0.74))
    to_object("Mullions", bm, SLATE, parent=root)
    bm = bmesh.new()
    cone(bm, 0.95, 0.12, 0.95, 8.82, sides=18)
    blob(bm, 0.2, (0, 0, 9.85), subdiv=2)
    to_object("Cap", bm, RED, bevel=0.03, parent=root)
    return root


def sailboat():
    """A little sailing dinghy: a round-bellied wooden hull with a lighter rail, a mast, a full white sail with a red band and a pennant.
    About 6 long and 7 tall; bow toward +x."""
    root = empty("sailboat")
    bm = bmesh.new()
    # The hull: the lower half of a stretched ball, its bow drawn to a soft point.
    ret = bmesh.ops.create_uvsphere(bm, u_segments=24, v_segments=12, radius=1.0)
    geom = bm.verts[:] + bm.edges[:] + bm.faces[:]
    bmesh.ops.bisect_plane(bm, geom=geom, plane_co=(0, 0, 0.0), plane_no=(0, 0, 1), clear_outer=True)
    bmesh.ops.contextual_create(bm, geom=[e for e in bm.edges if e.is_boundary])
    for v in bm.verts:
        x = v.co.x
        v.co.x *= 3.0
        v.co.y *= 1.05 * (1.0 - 0.45 * max(0.0, x) ** 2)
        v.co.z = v.co.z * 1.0 + 0.85
    to_object("Hull", bm, WOOD, parent=root)
    bm = bmesh.new()
    ret = bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((v.co.x * 5.6, v.co.y * 1.9, v.co.z * 0.16 + 0.88))
    to_object("Rail", bm, WOOD_LIGHT, smooth=False, bevel=0.06, parent=root)
    bm = bmesh.new()
    cone(bm, 0.09, 0.07, 6.2, 0.8, sides=8, at=(0.3, 0.0))
    to_object("Mast", bm, WOOD, parent=root)
    # The sail: a triangle from the masthead down the mast and out along a boom, bellied out a little, a red band across its foot.
    for name, colour, lo, hi in (("Sail", SAIL, 0.28, 1.0), ("SailBand", RED, 0.12, 0.28)):
        bm = bmesh.new()
        rows = 10
        cols = 8
        grid = []
        for i in range(rows + 1):
            t = lo + (hi - lo) * i / rows
            row = []
            for j in range(cols + 1):
                u = j / cols
                z = 1.5 + 5.2 * t
                x = 0.25 - u * 2.9 * (1.0 - t)
                belly = math.sin(math.pi * u) * 0.35 * (1.0 - t)
                row.append(bm.verts.new((x, belly, z)))
            grid.append(row)
        for i in range(rows):
            for j in range(cols):
                bm.faces.new((grid[i][j], grid[i][j + 1], grid[i + 1][j + 1], grid[i + 1][j]))
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.04)
        to_object(name, bm, colour, parent=root)
    bm = bmesh.new()
    cone(bm, 0.06, 0.06, 3.0, 0.0, sides=6)
    for v in bm.verts:
        v.co = Vector((v.co.z * -1.0 + 0.3, 0.0, 1.55 + v.co.x * 0.0))
    to_object("Boom", bm, WOOD, parent=root)
    bm = bmesh.new()
    a, b, c = bm.verts.new((0.35, 0, 7.0)), bm.verts.new((0.35, 0, 6.55)), bm.verts.new((1.25, 0, 6.85))
    bm.faces.new((a, b, c))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.04)
    to_object("Pennant", bm, RED, smooth=False, parent=root)
    return root


def mesa(name, seed, width, height, bands):
    """A sandstone butte: a slope of rubble at its foot, then near-vertical cliffs in strata, each band wobbly at the rim and nudged a
    little sideways, alternating three warm shades, under a flat, slightly overhanging cap; fallen boulders round the foot. `width`
    across the cliffs, `height` tall."""
    rng = random.Random(seed)
    root = empty(name)
    shades = [SAND_A, SAND_B, SAND_C]
    r = width * 0.5
    # The rubble slope.
    slope = height * 0.22
    bm = bmesh.new()
    cone(bm, r * 1.45, r * 1.02, slope, 0.0, sides=12, wobble=0.08, rng=rng)
    to_object("%s_slope" % name, bm, SAND_C, smooth=False, bevel=0.1, parent=root)
    z = slope
    x = 0.0
    cliff = (height - slope) * 0.94
    for i in range(bands):
        h = cliff / bands * rng.uniform(0.75, 1.25)
        r_top = r * rng.uniform(0.96, 1.0)
        bm = bmesh.new()
        cone(bm, r, r_top, h, z, sides=11, wobble=0.07, rng=rng, at=(x, 0.0))
        to_object("%s_band%d" % (name, i), bm, shades[i % 3], smooth=False, bevel=0.1, parent=root)
        z += h
        r = r_top
        x += rng.uniform(-0.04, 0.04) * width
    bm = bmesh.new()
    cone(bm, r * 1.07, r * 1.03, height * 0.06, z, sides=11, wobble=0.05, rng=rng, at=(x, 0.0))
    to_object("%s_cap" % name, bm, SAND_B, smooth=False, bevel=0.08, parent=root)
    bm = bmesh.new()
    for k in range(9):
        a = rng.uniform(0, math.tau)
        d = width * 0.5 * rng.uniform(1.35, 1.7)
        s = width * rng.uniform(0.04, 0.09)
        blob(bm, s, (math.cos(a) * d, math.sin(a) * d, s * 0.4), (1.2, 1.0, 0.75), 1, 0.2, rng)
    to_object("%s_boulders" % name, bm, SAND_C, smooth=False, parent=root)
    return root


LEAF_LIT = (0.55, 0.78, 0.3)
LEAF_SHADE = (0.2, 0.42, 0.26)


def leaf_clump(bm_lit, bm_shade, centre, radius, seed, teeth=9):
    """One leaf clump of a crown, after the owner's Wind Waker HD reference: a leafy dome, lumpy over its top (a hash of soft bumps),
    flattened underneath, with a skirt of pointed leaf tips round its lower rim (`teeth` of them, each its own length), so its outline
    is toothed and leafy, not a smooth ball. Its upper faces go to `bm_lit`, its underside and leaf tips to `bm_shade` (the game paints
    them in the tree's sunlit and deep greens)."""
    rng = random.Random(seed)
    tmp = bmesh.new()
    bmesh.ops.create_icosphere(tmp, subdivisions=3, radius=1.0)
    phase = rng.uniform(0, math.tau)
    lens = [rng.uniform(0.75, 1.25) for _ in range(teeth)]
    for v in tmp.verts:
        d = v.co.normalized()
        az = math.atan2(d.y, d.x) + phase
        # Soft leafy lumps over the dome.
        # (Faint: strong lumps broke the flat facets into noise; the reference's tiers are calm planes.)
        lump = 0.025 * math.sin(az * 5.0 + d.z * 4.0)
        r = 1.0 + lump
        z = d.z
        # Flattened underneath.
        if z < 0.0:
            z *= 0.62
        else:
            z *= 0.88
        # The leaf tips: round the lower rim, every tooth pulled out and down to a point.
        # (Two tiers of leaf tips, a lower skirt and a smaller one halfway up offset half a tooth, so a clump reads as layers of leaves,
        # after the owner's second reference, a papercraft forest whose trees are stacked tiers with crisp zigzag edges.)
        # (A wide, drooping skirt of a few long panels per tier, after the owner's papercraft references; a faint second row above.)
        for centre_z, width, reach, drop, shift in ((-0.12, 0.42, 0.72, 0.5, 0.0), (0.45, 0.18, 0.16, 0.1, 0.5)):
            band = max(0.0, 1.0 - abs(d.z - centre_z) / width)
            if band <= 0.0:
                continue
            a2 = az + shift * math.tau / teeth
            k = (a2 % (math.tau / teeth)) / (math.tau / teeth)
            tip = max(0.0, 1.0 - abs(k - 0.5) * 2.0) ** 1.6
            t = int((a2 % math.tau) / (math.tau / teeth)) % teeth
            r += reach * tip * band * lens[t]
            z -= drop * tip * band * lens[t]
        v.co = Vector((d.x * r, d.y * r, z * r)) * radius + Vector(centre)
    tmp.normal_update()
    for f in tmp.faces:
        fc = f.calc_center_median()
        up = (fc.z - centre[2]) / radius
        target = bm_lit if up > -0.05 else bm_shade
        vs = [target.verts.new(v.co) for v in f.verts]
        target.faces.new(vs)
    tmp.free()


def tree_crown(name, shape):
    """A broadleaf tree's crown in clumps (the trunk and its sway are the game's, `StageArt._round_tree`): one of the game's three crown
    shapes, a big body clump with smaller ones round it, each a `leaf_clump`. Two parts, `leaf_lit` and `leaf_shade`, centred on the
    crown's middle, sized for a tree of size 1."""
    root = empty(name)
    lit = bmesh.new()
    shade = bmesh.new()
    # Stacked tiers, each smaller than the one under it and overlapping it by about half, so the crown is a rounded shape taller than it
    # is wide that reads as layers of leaves (both of the owner's references); small clumps tucked in at the sides break the outline.
    # ((x, depth, height), radius); the crown's middle is the game's crown origin.
    # (Upper tiers a little toward the camera (Blender -y), so each tier's leaf skirt lies over the tier below and the layers read; set
    # behind, the skirts sank inside and the crown read as one lumpy cone.)
    if shape == 0:
        masses = [((0, 0.4, 0.4), 2.0), ((-1.35, 0.3, 0.7), 1.0), ((1.4, 0.3, 0.55), 0.95), ((0.1, -0.35, 2.05), 1.55), ((-0.05, -0.7, 3.3), 1.0)]
    elif shape == 1:
        masses = [((0, 0.4, 0.3), 1.75), ((1.2, 0.3, 0.8), 0.8), ((0, -0.3, 1.75), 1.45), ((0.1, -0.65, 2.95), 1.15), ((0, -0.95, 3.95), 0.75)]
    else:
        masses = [((0, 0.4, 0.5), 2.1), ((-1.5, 0.2, 0.75), 1.25), ((1.55, 0.2, 0.65), 1.15), ((0.1, -0.35, 2.15), 1.5), ((0.25, -0.7, 3.2), 0.85)]
    for k, (c, r) in enumerate(masses):
        leaf_clump(lit, shade, c, r, shape * 100 + k, teeth=8 if r > 1.6 else 6)
    for bm in (lit, shade):
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.0001)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    # (Flat-shaded planes, as a papercraft forest's trees are (the owner's third reference): each facet one tone, so the light reads
    # in crisp planes rather than a smooth plastic gradient.)
    for nm, bm, col in (("leaf_lit", lit, LEAF_LIT), ("leaf_shade", shade, LEAF_SHADE)):
        to_object(nm, bm, col, smooth=False, parent=root)
    return root


BARK_LIT = (0.5, 0.36, 0.28)


def tree_trunk():
    """A trunk to go with `tree_crown` (the papercraft references): seven flat sides tapering up, a little twist and bow, and five roots
    flaring out at its foot as folded planes, so it reads as a solid faceted trunk standing in the ground, not a smooth tube. Height 3.2
    and foot radius 0.42 for a tree of size 1 (the game's `_round_tree` proportions); flat-shaded."""
    root = empty("tree_trunk")
    bm = bmesh.new()
    rng = random.Random(5)
    sides = 7
    rings = [(0.0, 0.46), (0.35, 0.36), (1.2, 0.3), (2.2, 0.26), (3.2, 0.22)]
    loops = []
    for k, (z, r) in enumerate(rings):
        twist = k * 0.12
        bow = 0.06 * math.sin(z / 3.2 * math.pi)
        loops.append([bm.verts.new((math.cos(twist + j * math.tau / sides) * r * rng.uniform(0.92, 1.06) + bow,
                                    math.sin(twist + j * math.tau / sides) * r * rng.uniform(0.92, 1.06), z)) for j in range(sides)])
    for a_, b_ in zip(loops, loops[1:]):
        for j in range(sides):
            bm.faces.new((a_[j], a_[(j + 1) % sides], b_[(j + 1) % sides], b_[j]))
    bm.faces.new(loops[-1])
    # Roots: a folded wedge from low on the trunk out along the ground.
    for j in range(5):
        ang = j * math.tau / 5 + 0.3
        d = Vector((math.cos(ang), math.sin(ang), 0))
        side = Vector((-d.y, d.x, 0))
        out = rng.uniform(0.75, 1.0)
        base = d * 0.36
        p_top = bm.verts.new(base + Vector((0, 0, 0.55)))
        p_l = bm.verts.new(base + side * 0.16)
        p_r = bm.verts.new(base - side * 0.16)
        p_tip = bm.verts.new(d * out + Vector((0, 0, 0.02)))
        bm.faces.new((p_l, p_tip, p_top))
        bm.faces.new((p_tip, p_r, p_top))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    to_object("bark", bm, BARK_LIT, smooth=False, parent=root)
    return root


PROPS = {
    "lighthouse": lighthouse,
    "sailboat": sailboat,
    "mesa_a": lambda: mesa("mesa_a", 3, 24.0, 15.0, 4),
    "mesa_b": lambda: mesa("mesa_b", 8, 13.0, 22.0, 5),
    "cactus": lambda: cactus(),
    "palm_crown": lambda: palm_crown(),
    "tree_trunk": tree_trunk,
    "tree_crown_a": lambda: tree_crown("tree_crown_a", 0),
    "tree_crown_b": lambda: tree_crown("tree_crown_b", 1),
    "tree_crown_c": lambda: tree_crown("tree_crown_c", 2),
}


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
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_object_outline = True
    scene.render.resolution_x = 480
    scene.render.resolution_y = 480
    lo = Vector((1e9, 1e9, 1e9))
    hi = Vector((-1e9, -1e9, -1e9))
    for c in root.children_recursive:
        for corner in c.bound_box:
            p = c.matrix_world @ Vector(corner)
            lo = Vector(map(min, lo, p))
            hi = Vector(map(max, hi, p))
    size = max(hi - lo)
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = size * 1.2
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    mid = (lo + hi) / 2.0
    cam.location = mid + Vector((size * 0.6, -size * 2.0, size * 0.35))
    cam.rotation_euler = (Vector((0, 0, 0)) - Vector((0.6, -2.0, 0.35))).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = path
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
        print("prop %s: %d triangles (before bevels)" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
