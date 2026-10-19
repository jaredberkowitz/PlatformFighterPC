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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold_kit import block_tier, folded_dome, folded_peak, outward, pleated_tier  # noqa: E402

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
    """One palm frond from the crown: a midrib arching out and down at `angle` about the vertical, lined on either side with separate
    leaflets, each a long pointed blade swept toward the tip and drooping from the rib, longest a third of the way out (2026-10-18, art
    review 7: a continuous strip read as a flat green card; separate drooping leaflets catch the light one by one, flat-shaded)."""
    steps = 20
    out = Vector((math.cos(angle), math.sin(angle), 0.0))
    side = Vector((-out.y, out.x, 0.0))
    rib = []
    for i in range(steps + 1):
        t = i / steps
        rib.append(out * length * t + Vector((0, 0, length * (0.75 * t - droop * t * t))))
    # The rib itself, a thin strip, so the leaflets hang from something.
    for i in range(steps):
        a, b = rib[i], rib[i + 1]
        n = Vector((0, 0, 0.025))
        bm.faces.new((bm.verts.new(a - side * 0.03), bm.verts.new(b - side * 0.03), bm.verts.new(b + side * 0.03), bm.verts.new(a + side * 0.03)))
    for sgn in (-1.0, 1.0):
        for i in range(1, steps):
            t = i / steps
            w = width * 1.15 * min(1.0, t * 4.0) * (1.0 - 0.72 * t) * rng.uniform(0.85, 1.1)
            if w < 0.05:
                continue
            base0 = rib[i]
            base1 = rib[min(steps, i + 1)]
            tip = (base0 + base1) * 0.5 + side * sgn * w + out * w * 0.45 + Vector((0, 0, -w * rng.uniform(0.45, 0.7)))
            mid = (base0 + base1) * 0.5 + side * sgn * w * 0.5 + out * w * 0.2 + Vector((0, 0, -w * 0.12))
            v0, v1, vt, vm = bm.verts.new(base0), bm.verts.new(base1), bm.verts.new(tip), bm.verts.new(mid)
            # Two facets folded along the leaflet's own spine (base to tip through `mid`), so each leaflet has a lit and a shaded half.
            if sgn > 0:
                bm.faces.new((v0, vm, vt))
                bm.faces.new((vm, v1, vt))
            else:
                bm.faces.new((v0, vt, vm))
                bm.faces.new((vm, vt, v1))


def cactus():
    """A saguaro in chunky pleated columns (the judge's round 40 and the owner's canyon reference: blocky, bevelled; ribbed tubes read as
    smooth capsules): a trunk of three stacked fourteen-cornered prisms whose corners alternate in and out, so it is seven deep flat
    ribs, each a lit plane and a shaded one, tapering to a blunt cut top; two arms the same, each a short elbow block out from the trunk
    and a column up from it, at different heights; a pink flower on the crown. About 4.5 tall, standing on the origin."""
    rng = random.Random(97)
    root = empty("cactus")
    bm = bmesh.new()

    def column(at, z0, r, h, segs, top_taper):
        z = z0
        for k in range(segs):
            last = k == segs - 1
            rr = r * (1.0 - 0.04 * k)
            block_tier(bm, at, z, h / segs, [rr * (1.0 if i % 2 == 0 else 0.78) for i in range(14)], 0.0,
                       taper=top_taper if last else 0.98)
            z += h / segs
        return z
    column((0.0, 0.0), -0.15, 0.5, 4.25, 3, 0.6)
    for (side, z_out, reach, r, up) in ((-1.0, 1.4, 0.75, 0.32, 1.5), (1.0, 2.1, 0.62, 0.29, 1.3)):
        # The elbow: a short block out from the trunk.
        block_tier(bm, (side * (0.42 + reach * 0.5), 0.0), z_out, r * 1.6, [math.hypot(reach * 0.5 + 0.2, r) * rng.uniform(0.95, 1.05) for _ in range(4)],
                   math.atan2(r, reach * 0.5 + 0.2), taper=0.96)
        column((side * (0.42 + reach), 0.0), z_out + r * 0.6, r, up, 2, 0.6)
    to_object("Cactus", bm, CACTUS, smooth=False, parent=root)
    bm = bmesh.new()
    for k in range(5):
        a = math.tau * k / 5
        blob(bm, 0.11, (math.cos(a) * 0.13, math.sin(a) * 0.13, 4.18), (1.0, 1.0, 0.55), 1)
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
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.02)
    to_object("Fronds", bm, PALM, smooth=False, parent=root)
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
    """A sandstone butte in chunky cut ledges (the judge's round 40 and the owner's canyon reference, 2026-10-19: blocky sculpted
    cliffs, each ledge a few big planes, in place of stacked smooth wobbly discs that read as pancakes): a slope of fallen blocks at its
    foot, then `bands` ledges, each an irregular six- or seven-sided prism of its own height, stepping in or jutting out from the one
    under it so the cliff has shelves and overhangs, under a wider flat cap; boulders round the foot. `width` across the cliffs, `height`
    tall. The game paints every part in the stage's strata, so the parts keep their old names and colours."""
    rng = random.Random(seed)
    root = empty(name)
    shades = [SAND_A, SAND_B, SAND_C]
    r = width * 0.5
    slope = height * 0.22
    bm = bmesh.new()
    block_tier(bm, (0.0, 0.0), 0.0, slope, [r * rng.uniform(1.3, 1.55) for _ in range(7)], rng.uniform(0, math.tau), taper=0.72)
    to_object("%s_slope" % name, bm, SAND_C, smooth=False, parent=root)
    z = slope
    x = 0.0
    cliff = (height - slope) * 0.94
    sides = rng.choice((6, 7))
    for i in range(bands):
        h = cliff / bands * rng.uniform(0.75, 1.25)
        # A shelf or an overhang now and then: the ledge steps in, or juts out past the one below.
        step = rng.choice((0.9, 0.94, 1.0, 1.06)) if i > 0 else 1.0
        r *= step
        bm = bmesh.new()
        block_tier(bm, (x, 0.0), z, h, [r * rng.uniform(0.82, 1.08) for _ in range(sides)], rng.uniform(-0.25, 0.25), taper=rng.uniform(0.9, 0.97),
                   lean=(rng.uniform(-0.03, 0.03) * width, 0.0))
        to_object("%s_band%d" % (name, i), bm, shades[i % 3], smooth=False, parent=root)
        z += h
        r *= 0.95
        x += rng.uniform(-0.04, 0.04) * width
    bm = bmesh.new()
    block_tier(bm, (x, 0.0), z, height * 0.06, [r * rng.uniform(1.04, 1.12) for _ in range(sides)], rng.uniform(-0.2, 0.2), taper=0.97)
    to_object("%s_cap" % name, bm, SAND_B, smooth=False, parent=root)
    bm = bmesh.new()
    for k in range(9):
        a = rng.uniform(0, math.tau)
        d = width * 0.5 * rng.uniform(1.35, 1.7)
        s_ = width * rng.uniform(0.04, 0.09)
        block_tier(bm, (math.cos(a) * d, math.sin(a) * d), -s_ * 0.2, s_ * 1.1, [s_ * rng.uniform(0.8, 1.2) for _ in range(5)], rng.uniform(0, math.tau), taper=0.7)
    to_object("%s_boulders" % name, bm, SAND_C, smooth=False, parent=root)
    return root


def rock_arch():
    """The canyon's rock arch in the buttes' chunky cut ledges (the judge's round 40: smooth discs and boxes in four inked stone colours):
    two legs of stacked irregular prisms narrowing as they rise, leaning a little toward each other, joined by a thick span of blocks
    along a shallow bow, with fallen blocks at the feet. 12 across between the legs' middles, 18.5 tall, standing on the origin, front -y.
    Parts `band0` to `band2` and `blocks`, painted by the game in the stage's strata."""
    rng = random.Random(171)
    root = empty("rock_arch")
    parts = [bmesh.new() for _ in range(3)]
    span, height = 12.0, 18.5
    for side in (-1.0, 1.0):
        z, k = 0.0, 0
        while z < height - 2.0:
            u = z / height
            r = (3.2 + (2.1 - 3.2) * u) * rng.uniform(0.92, 1.08)
            h = 2.4 * rng.uniform(0.8, 1.2)
            x = side * span * 0.5 - side * 0.6 * u
            block_tier(parts[k % 3], (x, 0.0), z, h, [r * rng.uniform(0.82, 1.08) * (0.8 if i % 3 == 1 else 1.0) for i in range(6)],
                       rng.uniform(0, math.tau), taper=rng.uniform(0.9, 0.98))
            z += h
            k += 1
    # The span: chunky blocks along a shallow bow from leg to leg, two courses.
    for course, (lift, thick) in enumerate(((0.0, 2.4), (2.4, 1.6))):
        n = 7
        for j in range(n):
            t = (j + 0.5) / n
            x = -span * 0.5 - 1.6 + (span + 3.2) * t
            zc = height - 2.2 + lift + 1.6 * math.sin(t * math.pi)
            w = (span + 3.2) / n * rng.uniform(1.05, 1.25)
            d = (3.2 - lift * 0.3) * rng.uniform(0.9, 1.1)
            block_tier(parts[(j + course) % 3], (x, 0.0), zc, thick * rng.uniform(0.9, 1.1),
                       [math.hypot(w, d) * 0.5 * rng.uniform(0.9, 1.05) for _ in range(4)], math.atan2(d, w), taper=0.96)
    blocks = bmesh.new()
    for (bx, s_) in ((-span * 0.5 - 3.2, 1.1), (-span * 0.5 + 2.6, 0.7), (span * 0.5 + 3.0, 1.3), (span * 0.5 - 2.4, 0.6)):
        block_tier(blocks, (bx, -1.0), 0.0, 1.2 * s_, [0.9 * s_ * rng.uniform(0.8, 1.2) for _ in range(5)], rng.uniform(0, math.tau), taper=0.8)
    for i, bm in enumerate(parts):
        to_object("band%d" % i, bm, [SAND_A, SAND_B, SAND_C][i], smooth=False, parent=root)
    to_object("blocks", blocks, SAND_C, smooth=False, parent=root)
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
    # (Round, 2026-10-19, the owner: a broadleaf's crown as a cloud of folded domes, a big one in the middle, shoulders at its sides and a
    # top one a little forward, wider than tall, in place of stacked tiers that read as a fir.)
    # ((x, depth, height), radius); the crown's middle is the game's crown origin.
    if shape == 0:
        masses = [((0, 0.3, 0.6), 2.1), ((-1.75, 0.2, 0.15), 1.35), ((1.8, 0.2, 0.25), 1.3), ((-0.6, -0.3, 1.9), 1.45), ((0.9, -0.2, 1.7), 1.25)]
    elif shape == 1:
        masses = [((0, 0.3, 0.5), 1.9), ((1.5, 0.1, 0.1), 1.2), ((-1.4, 0.2, 0.3), 1.1), ((0.2, -0.35, 1.75), 1.4)]
    else:
        masses = [((0, 0.3, 0.6), 2.2), ((-1.9, 0.2, 0.2), 1.3), ((1.85, 0.1, 0.3), 1.35), ((-0.4, -0.3, 1.95), 1.3), ((1.0, -0.4, 1.6), 1.05)]
    for k, (c, r) in enumerate(masses):
        folded_dome(lit, shade, c, r, 7 if r > 1.6 else 6, random.Random(shape * 100 + k))
    for bm in (lit, shade):
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.0001)
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


# How far a box's corners wander (a share of its smallest side), so pieces read hand-made.
HAND_JITTER = 0.12
_JRNG = random.Random(99)


def add_box(bm, centre, size, rot=(0.0, 0.0, 0.0), taper=0.0):
    """A box into `bm` at `centre`, `size` (x, y, z), turned by `rot` degrees (x, y, z); `taper` narrows its top in x."""
    m = (Matrix.Translation(Vector(centre)) @ Matrix.Rotation(math.radians(rot[2]), 4, "Z") @ Matrix.Rotation(math.radians(rot[1]), 4, "Y")
         @ Matrix.Rotation(math.radians(rot[0]), 4, "X"))
    sx, sy, sz = size[0] / 2.0, size[1] / 2.0, size[2] / 2.0
    j = min(size) * HAND_JITTER
    vs = []
    for z in (-sz, sz):
        k = 1.0 - taper if z > 0 else 1.0
        for x, y in ((-sx, -sy), (sx, -sy), (sx, sy), (-sx, sy)):
            vs.append(bm.verts.new(m @ Vector((x * k + _JRNG.uniform(-j, j), y + _JRNG.uniform(-j, j), z + _JRNG.uniform(-j, j)))))
    for f in ((0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)):
        bm.faces.new([vs[i] for i in f])


def cottage():
    """A half-timbered cottage after the owner's papercraft village reference: cream plaster panels between thick dark timbers standing
    proud of the wall (posts, sill and wall plates, diagonal braces), a stone footing laid stone by stone, a deep roof of overlapping
    shingle rows (each row a slab with a lit lower edge) over generous eaves, a dormer, a stone chimney with a cap, an arched plank door
    with iron straps on a step, a framed window with shutters and a flower box. Parts are named by role (the game recolours `roof` and
    `roof_edge` per cottage). Front faces -y. 5 wide, 3.6 deep, walls 3.2 high, on the origin (the game's `_cottage` size)."""
    root = empty("cottage")
    parts = {}

    def part(name):
        # (Stones, timbers and shingle rows each go to one of three tones at random, so no two neighbours are the same flat colour.)
        if name in ("stone", "timber", "roof"):
            name = "%s_%d" % (name, rng.randint(0, 2))
        if name not in parts:
            parts[name] = bmesh.new()
        return parts[name]
    rng = random.Random(11)
    W, D, H = 5.0, 3.6, 3.8
    fy = -D / 2.0
    # Footing: a course of stones of their own sizes along the front, on a plinth.
    x = -W / 2.0 - 0.15
    while x < W / 2.0 + 0.15:
        w = rng.uniform(0.45, 0.8)
        add_box(part("stone"), (x + w / 2.0, fy - 0.08, 0.22), (w - 0.05, 0.3, rng.uniform(0.36, 0.46)), (0, 0, rng.uniform(-2, 2)))
        x += w
    add_box(part("stone"), (0, 0, 0.2), (W + 0.2, D + 0.2, 0.4))
    add_box(part("wall"), (0, 0, 0.4 + (H - 0.4) / 2.0), (W, D, H - 0.4))
    # The timber frame, proud of the front.
    t = 0.22
    front = fy - 0.06
    for px in (-W / 2.0 + t / 2.0, W / 2.0 - t / 2.0, -0.85, 0.45):
        add_box(part("timber"), (px, front, 0.4 + (H - 0.4) / 2.0), (t, 0.14, H - 0.4))
    for z in (0.45, H - 0.08):
        add_box(part("timber"), (0, front, z), (W + 0.04, 0.15, t))
    add_box(part("timber"), (W / 2.0 - 1.1, front, 1.55), (2.2, 0.15, t))
    for x0, x1, z0, z1 in ((-W / 2.0 + 0.2, -0.95, H - 0.15, 2.65), (-0.75, 0.35, 2.65, H - 0.15)):
        L = math.hypot(x1 - x0, z1 - z0)
        ang = math.degrees(math.atan2(z1 - z0, x1 - x0))
        add_box(part("timber"), ((x0 + x1) / 2.0, front, (z0 + z1) / 2.0), (L, 0.13, 0.17), (0, -ang, 0))
    for sx in (-1, 1):
        for py in (-D / 2.0 + t / 2.0, D / 2.0 - t / 2.0, 0.0):
            add_box(part("timber"), (sx * (W / 2.0 + 0.05), py, 0.4 + (H - 0.4) / 2.0), (0.14, t, H - 0.4))
    # Gables.
    pitch = 42.0
    run = D / 2.0 + 0.75
    rise = (D / 2.0) * math.tan(math.radians(pitch))
    for sx in (-1, 1):
        g = part("wall")
        a_ = g.verts.new((sx * W / 2.0, -D / 2.0, H))
        b_ = g.verts.new((sx * W / 2.0, D / 2.0, H))
        c_ = g.verts.new((sx * W / 2.0, 0, H + rise))
        g.faces.new((a_, b_, c_))
        # (The other side too, so whichever way the gable faces it is drawn.)
        g.faces.new([g.verts.new(v.co + Vector((sx * 0.01, 0, 0))) for v in (c_, b_, a_)])
    # Roof: rows of shingle slabs from the eave up, each overlapping the next, its lower edge a lighter lip.
    tanp = math.tan(math.radians(pitch))
    ez = H - (run - D / 2.0) * tanp
    slope = run / math.cos(math.radians(pitch))
    rows = 5
    for sgn in (-1, 1):
        for r in range(rows):
            u0 = r / rows
            u1 = min(1.0, (r + 1.25) / rows)
            um = (u0 + u1) / 2.0
            length = slope * (u1 - u0)
            lift = 0.14 + 0.08 * r
            sag = rng.uniform(-0.06, 0.06)
            cy = sgn * run * (1.0 - um)
            cz = ez + (H + rise - ez) * um + lift + sag
            add_box(part("roof"), (0, cy, cz), (W + 1.3 - r * 0.04, length, 0.18), (-sgn * pitch, rng.uniform(-1.2, 1.2), 0))
            ly = sgn * run * (1.0 - u0)
            lz = ez + (H + rise - ez) * u0 + lift + sag - 0.04
            add_box(part("roof_edge"), (0, ly, lz - 0.02), (W + 1.36 - r * 0.04, 0.16, 0.26), (-sgn * pitch, 0, 0))
    add_box(part("roof_edge"), (0, 0, H + rise + 0.32), (W + 1.4, 0.36, 0.26))
    # A dormer on the front slope.
    dz = H + rise * 0.3
    dy = -D / 2.0 * 0.55
    add_box(part("wall"), (-1.2, dy, dz + 0.35), (1.0, 0.9, 0.8))
    add_box(part("frame"), (-1.2, dy - 0.46, dz + 0.38), (0.62, 0.06, 0.52))
    add_box(part("glow"), (-1.2, dy - 0.48, dz + 0.38), (0.44, 0.04, 0.36))
    for sg in (-1, 1):
        add_box(part("roof"), (-1.2 + sg * 0.32, dy, dz + 0.95), (0.85, 1.2, 0.12), (0, sg * 40, 0))
    add_box(part("roof_edge"), (-1.2, dy - 0.6, dz + 0.92), (1.35, 0.1, 0.14))
    # A stone chimney with a cap.
    cx, cyy = 1.4, 0.6
    for k in range(7):
        add_box(part("stone"), (cx + rng.uniform(-0.02, 0.02), cyy, H + rise * 0.4 + k * 0.32), (0.62, 0.62, 0.3))
    add_box(part("stone"), (cx, cyy, H + rise * 0.4 + 7 * 0.32 - 0.05), (0.8, 0.8, 0.14))
    # The door (after the owner: windows and doors carry believability): a timber surround (posts and a heavy lintel), planks with dark
    # gaps between them and an arched head, two long strap hinges with curled ends, a ring handle, a little barred window, a stone step
    # and a lantern on a bracket beside it.
    dxc = -1.4
    dfront = front - 0.02
    add_box(part("gap"), (dxc, dfront + 0.02, 1.5), (0.9, 0.04, 2.3))
    for k in range(4):
        px = dxc - 0.33 + k * 0.22
        top = math.sqrt(max(0.0, 0.45 ** 2 - (px - dxc) ** 2))
        add_box(part("door"), (px, dfront - 0.03, 0.4 + (1.9 + top) / 2.0), (0.19, 0.08, 1.9 + top))
    for z in (0.85, 1.85):
        add_box(part("iron"), (dxc - 0.05, dfront - 0.09, z), (0.78, 0.03, 0.09))
        add_box(part("iron"), (dxc + 0.36, dfront - 0.09, z), (0.12, 0.03, 0.16))
    add_box(part("iron"), (dxc + 0.27, dfront - 0.11, 1.35), (0.05, 0.05, 0.14))
    add_box(part("iron"), (dxc + 0.27, dfront - 0.11, 1.27), (0.12, 0.04, 0.04))
    add_box(part("frame"), (dxc, dfront - 0.08, 2.05), (0.34, 0.04, 0.3))
    add_box(part("glow"), (dxc, dfront - 0.09, 2.05), (0.24, 0.03, 0.2))
    for k in range(2):
        add_box(part("iron"), (dxc - 0.05 + k * 0.1, dfront - 0.11, 2.05), (0.025, 0.03, 0.22))
    for sx in (-1, 1):
        add_box(part("timber"), (dxc + sx * 0.55, front - 0.02, 1.5), (0.16, 0.16, 2.3))
    add_box(part("timber"), (dxc, front - 0.04, 2.75), (1.4, 0.2, 0.22))
    add_box(part("stone"), (dxc, fy - 0.45, 0.12), (1.4, 0.6, 0.24))
    add_box(part("stone"), (dxc + 0.05, fy - 0.8, 0.06), (1.1, 0.4, 0.12))
    lx = dxc + 0.85
    add_box(part("iron"), (lx, front - 0.2, 2.35), (0.04, 0.36, 0.04))
    add_box(part("iron"), (lx, front - 0.36, 2.22), (0.24, 0.24, 0.04))
    add_box(part("glow"), (lx, front - 0.36, 2.05), (0.16, 0.16, 0.24))
    add_box(part("iron"), (lx, front - 0.36, 1.92), (0.22, 0.22, 0.04))
    # The window: a deep reveal (sill and lintel standing out), six panes in a mullion grid, curtains drawn to the sides inside, shutters
    # of three boards with a Z brace, and a planter of leaf clumps and flowers under it.
    wxc, wz = 1.15, 2.2
    ww, wh = 0.86, 0.76
    add_box(part("gap"), (wxc, front + 0.02, wz), (ww, 0.04, wh))
    add_box(part("glow"), (wxc, front - 0.0, wz), (ww - 0.04, 0.03, wh - 0.04))
    for sx in (-1, 1):
        add_box(part("curtain"), (wxc + sx * (ww / 2.0 - 0.12), front - 0.02, wz + 0.02), (0.2, 0.03, wh - 0.06), (0, sx * 8, 0))
    for k in range(1, 3):
        add_box(part("frame"), (wxc - ww / 2.0 + k * ww / 3.0, front - 0.05, wz), (0.05, 0.04, wh))
    add_box(part("frame"), (wxc, front - 0.05, wz), (ww, 0.04, 0.05))
    for sx in (-1, 1):
        add_box(part("frame"), (wxc + sx * (ww / 2.0 + 0.04), front - 0.06, wz), (0.09, 0.12, wh + 0.1))
    add_box(part("frame"), (wxc, front - 0.08, wz + wh / 2.0 + 0.07), (ww + 0.3, 0.18, 0.13))
    add_box(part("frame"), (wxc, front - 0.12, wz - wh / 2.0 - 0.05), (ww + 0.34, 0.26, 0.1))
    for sx in (-1, 1):
        sc = wxc + sx * (ww / 2.0 + 0.32)
        for k in range(3):
            add_box(part("shutter"), (sc + (k - 1) * 0.15, front - 0.07, wz), (0.14, 0.05, wh + 0.08))
        for z in (wz - wh / 2.0 + 0.12, wz + wh / 2.0 - 0.12):
            add_box(part("shutter_brace"), (sc, front - 0.1, z), (0.46, 0.03, 0.07))
        add_box(part("shutter_brace"), (sc, front - 0.1, wz), (0.6, 0.03, 0.06), (0, sx * 52, 0))
    add_box(part("frame"), (wxc, front - 0.26, wz - wh / 2.0 - 0.2), (ww + 0.3, 0.3, 0.22))
    prng = random.Random(4)
    for k in range(7):
        x = wxc - ww / 2.0 - 0.08 + k * (ww + 0.16) / 6.0
        add_box(part("leaves"), (x, front - 0.28, wz - wh / 2.0 - 0.05), (0.2, 0.22, 0.18), (prng.uniform(0, 40), 45, prng.uniform(0, 40)))
        add_box(part("flowers" if k % 2 else "flowers_b"), (x + 0.02, front - 0.36, wz - wh / 2.0 + 0.05), (0.11, 0.11, 0.11), (0, 45, 45))
    cols = {"wall": (0.97, 0.93, 0.82), "timber": (0.42, 0.28, 0.2), "stone": (0.72, 0.7, 0.72), "roof": RED, "roof_edge": (0.95, 0.5, 0.42),
            "door": (0.86, 0.5, 0.26), "iron": (0.22, 0.2, 0.24), "frame": (0.58, 0.4, 0.28), "glow": GLASS, "shutter": (0.36, 0.56, 0.74),
            "flowers": PINK, "leaves": (0.32, 0.6, 0.3), "gap": (0.16, 0.12, 0.12), "curtain": (0.95, 0.66, 0.6),
            "shutter_brace": (0.26, 0.42, 0.6), "flowers_b": (1.0, 0.86, 0.36)}
    tone = (0.9, 1.0, 1.1)
    for name, bm in parts.items():
        base = name.rsplit("_", 1)
        if len(base) == 2 and base[1].isdigit():
            col = tuple(min(1.0, c * tone[int(base[1])]) for c in cols[base[0]])
        else:
            col = cols[name]
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        # (Soft bevels on the chunky pieces: their edges catch a lit rim, the opposite of a flat cut-out.)
        chunky = base[0] in ("stone", "timber", "roof", "roof_edge", "door", "frame")
        to_object(name, bm, col, smooth=False, bevel=0.025 if chunky else 0.0, parent=root)
    return root


def windmill_tower():
    """The meadow windmill's tower and cap (its door, windows, gallery and turning sails stay the game's, `StageArt._windmill`), in the
    cottage's language: eight plastered faces tapering up (apothem 3.0 at the foot to 1.8 at the top, as the round tower it replaces, so
    the game's windows sit on it), a little off true, proud timber posts on the corners and two timber bands, a course of stones at the
    foot, and a cap of three lapped shingle tiers with lit edges under a knob. Base at z 1 (the stone drum below is the game's)."""
    root = empty("windmill_tower")
    parts = {}
    rng = random.Random(21)

    def part(name):
        if name in ("stone", "timber", "roof"):
            name = "%s_%d" % (name, rng.randint(0, 2))
        if name not in parts:
            parts[name] = bmesh.new()
        return parts[name]
    sides = 8
    k8 = 1.0 / math.cos(math.pi / sides)
    z0, z1 = 1.0, 13.0

    def apo(z):
        return 3.0 + (1.8 - 3.0) * (z - z0) / (z1 - z0)
    # The plastered faces.
    bm = part("wall")
    rings = []
    for z in (z0, (z0 + z1) / 2.0, z1):
        R = apo(z) * k8
        rings.append([bm.verts.new((math.cos(j * math.tau / sides + math.pi / sides) * R * rng.uniform(0.98, 1.02),
                                    math.sin(j * math.tau / sides + math.pi / sides) * R * rng.uniform(0.98, 1.02), z)) for j in range(sides)])
    for ra, rb in zip(rings, rings[1:]):
        for j in range(sides):
            bm.faces.new((ra[j], ra[(j + 1) % sides], rb[(j + 1) % sides], rb[j]))
    # Corner posts, standing a little proud, following the taper.
    for j in range(sides):
        ang = j * math.tau / sides + math.pi / sides
        p0 = Vector((math.cos(ang) * apo(z0) * k8, math.sin(ang) * apo(z0) * k8, z0))
        p1 = Vector((math.cos(ang) * apo(z1) * k8, math.sin(ang) * apo(z1) * k8, z1))
        mid = (p0 + p1) / 2.0
        L = (p1 - p0).length
        tilt = math.degrees(math.atan2((p0 - p1).xy.length, z1 - z0))
        add_box(part("timber"), mid, (0.3, 0.3, L), (0, -tilt, math.degrees(ang)))
    # Timber bands round the tower, and a course of stones at its foot.
    for zb in (4.6, 11.6):
        R = apo(zb) * k8 + 0.06
        for j in range(sides):
            a0 = j * math.tau / sides + math.pi / sides
            a1 = a0 + math.tau / sides
            c = Vector((math.cos(a0) + math.cos(a1), math.sin(a0) + math.sin(a1), 0)) * R / 2.0
            side_len = 2.0 * R * math.sin(math.pi / sides)
            add_box(part("timber"), (c.x, c.y, zb), (side_len + 0.1, 0.22, 0.3), (0, 0, math.degrees((a0 + a1) / 2.0) + 90.0))
    for j in range(16):
        a0 = j * math.tau / 16
        R = apo(z0) * k8 + 0.12
        add_box(part("stone"), (math.cos(a0) * R, math.sin(a0) * R, z0 + 0.3), (1.25, 0.45, rng.uniform(0.5, 0.7)), (0, 0, math.degrees(a0) + 90.0))
    # The cap: a trim ring and three lapped shingle tiers, each a cone with a lit lower lip, under a knob.
    zc = z1
    add_box(part("timber"), (0, 0, zc + 0.2), (4.4, 4.4, 0.4))
    tiers = [(2.9, 1.4), (2.2, 1.3), (1.4, 1.2)]
    zt = zc + 0.35
    for r_bottom, h in tiers:
        cone(part("roof"), r_bottom, r_bottom * 0.45, h, z0=zt, sides=12, wobble=0.06, rng=rng)
        ring(part("roof_edge"), r_bottom - 0.05, 0.12, zt + 0.05, sides=16, minor_sides=6)
        zt += h * 0.72
    cone(part("roof"), 0.75, 0.0, 1.0, z0=zt, sides=12)
    blob(part("timber"), 0.3, (0, 0, zt + 1.05), subdiv=1)
    cols = {"wall": (0.97, 0.94, 0.86), "timber": (0.44, 0.3, 0.22), "stone": (0.68, 0.66, 0.66), "roof": (0.76, 0.34, 0.28),
            "roof_edge": (0.95, 0.56, 0.46)}
    tone = (0.9, 1.0, 1.1)
    for name, bm in parts.items():
        base = name.rsplit("_", 1)
        if len(base) == 2 and base[1].isdigit():
            col = tuple(min(1.0, c * tone[int(base[1])]) for c in cols[base[0]])
        else:
            col = cols[name]
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        to_object(name, bm, col, smooth=False, bevel=0.03 if base[0] in ("stone", "timber") else 0.0, parent=root)
    return root


def crate():
    """A slatted shipping crate (1 x 1 x 1, standing on the origin; the gauntlet's round 4 after the owner's harbour reference, in place of
    the built box): corner posts, three slats a side with dark gaps between them showing the inside, top and bottom rails, and one
    diagonal brace across the front and back, every board bevelled and a hair off true, a little varied in shade."""
    rng = random.Random(41)
    root = empty("crate")
    bm_dark = bmesh.new()
    add_box(bm_dark, (0.0, 0.0, 0.5), (0.86, 0.86, 0.9))
    to_object("inside", bm_dark, (0.3, 0.2, 0.14), smooth=False).parent = root
    shades = [(0.8, 0.6, 0.38), (0.74, 0.54, 0.34), (0.84, 0.64, 0.42)]
    for i, (sx, sy) in enumerate([(-1, -1), (1, -1), (1, 1), (-1, 1)]):
        bm = bmesh.new()
        add_box(bm, (sx * 0.44, sy * 0.44, 0.5), (0.13, 0.13, 1.0))
        to_object("post%d" % i, bm, WOOD, smooth=False, bevel=0.015).parent = root
    for face, (nx, ny) in enumerate([(0, -1), (1, 0), (0, 1), (-1, 0)]):
        for k in range(3):
            z = 0.2 + 0.3 * k
            w = (0.84, 0.05) if nx == 0 else (0.05, 0.84)
            bm = bmesh.new()
            add_box(bm, (nx * 0.465, ny * 0.465, z + rng.uniform(-0.01, 0.01)), (w[0], w[1], 0.24), rot=(0.0, 0.0, rng.uniform(-0.6, 0.6)))
            to_object("slat%d_%d" % (face, k), bm, rng.choice(shades), smooth=False, bevel=0.012).parent = root
        for z in (0.04, 0.96):
            w = (0.86, 0.07) if nx == 0 else (0.07, 0.86)
            bm = bmesh.new()
            add_box(bm, (nx * 0.475, ny * 0.475, z), (w[0], w[1], 0.08))
            to_object("rail%d_%d" % (face, int(z * 10)), bm, WOOD, smooth=False, bevel=0.012).parent = root
        if face in (0, 2):
            bm = bmesh.new()
            add_box(bm, (0.0, ny * 0.49, 0.5), (1.1, 0.05, 0.09), rot=(0.0, 45.0 if face == 0 else -45.0, 0.0))
            to_object("brace%d" % face, bm, WOOD, smooth=False, bevel=0.012).parent = root
    bm = bmesh.new()
    for k in range(4):
        add_box(bm, (-0.33 + 0.22 * k, 0.0, 0.985), (0.2, 0.86, 0.04))
    to_object("lid", bm, (0.78, 0.58, 0.36), smooth=False, bevel=0.01).parent = root
    return root


def barrel():
    """A cooper's barrel (1.1 tall, standing on the origin): twelve bulging staves each its own shade with a hairline gap, two iron hoops
    sitting on the bulge, the staves standing as a rim above a lid of boards."""
    rng = random.Random(7)
    root = empty("barrel")
    prof = [(0.33, 0.0), (0.37, 0.06), (0.405, 0.22), (0.425, 0.4), (0.43, 0.55), (0.425, 0.7), (0.405, 0.88), (0.37, 1.04), (0.36, 1.1)]
    staves = 12
    for st in range(staves):
        a0 = math.tau * (st + 0.04) / staves
        a1 = math.tau * (st + 0.96) / staves
        bm = bmesh.new()
        cols = 4
        rings = []
        for (r, z) in prof:
            row = []
            for c in range(cols + 1):
                a = a0 + (a1 - a0) * c / cols
                rr = r * (1.0 + 0.01 * math.sin(c / cols * math.pi))
                row.append(bm.verts.new((rr * math.cos(a), rr * math.sin(a), z)))
            rings.append(row)
        for k in range(len(rings) - 1):
            for c in range(cols):
                bm.faces.new((rings[k][c], rings[k][c + 1], rings[k + 1][c + 1], rings[k + 1][c]))
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.04)
        shade = tuple(v * rng.uniform(0.9, 1.08) for v in (0.66, 0.44, 0.28))
        to_object("stave%d" % st, bm, shade, smooth=True).parent = root
    for h in (0.22, 0.88):
        r = 0.405 + 0.012
        bm = bmesh.new()
        ring(bm, r, 0.03, h, sides=32, minor_sides=6)
        to_object("hoop%d" % int(h * 100), bm, (0.32, 0.31, 0.38), smooth=True).parent = root
    bm = bmesh.new()
    cone(bm, 0.34, 0.34, 0.04, z0=1.0, sides=24)
    to_object("lid", bm, (0.52, 0.34, 0.22), smooth=False).parent = root
    return root


def grass_tuft():
    """A clump of grass for the lawns' front lip (the gauntlet's round 6: round turf tongues read as blobs and the lip as a ruled line):
    eleven tapered, curving blades fanned from one root, leaning out and up, each its own length and twist, standing on the origin."""
    rng = random.Random(19)
    root = empty("grass_tuft")
    bm = bmesh.new()
    for b in range(11):
        a = math.tau * b / 11.0 + rng.uniform(-0.2, 0.2)
        lean = rng.uniform(0.25, 0.75)
        length = rng.uniform(0.35, 0.62)
        base = Vector((math.cos(a) * 0.05, math.sin(a) * 0.05, 0.0))
        out = Vector((math.cos(a), math.sin(a), 0.0))
        side = Vector((-math.sin(a), math.cos(a), 0.0))
        left, right = [], []
        for k in range(6):
            t = k / 5.0
            w = 0.045 * (1.0 - t) + 0.004
            c = base + out * (lean * length * t * t) + Vector((0, 0, length * t * (1.0 - 0.25 * t)))
            left.append(bm.verts.new(c - side * w))
            right.append(bm.verts.new(c + side * w))
        for k in range(5):
            bm.faces.new((left[k], right[k], right[k + 1], left[k + 1]))
    to_object("blades", bm, (0.4, 0.66, 0.3), smooth=True).parent = root
    return root


def truss_bay():
    """One bay of a riveted steel scaffold tower (the gauntlet's round 6: the city's thin primitive struts): 1.0 wide (x), 1.8 tall (z,
    standing on the origin and reaching up), flat in the frame's plane (y): two square posts with flanges, a cross rail of angle iron at
    the top, an X of flat bars, gusset plates at every joint and rivet heads on them, every edge bevelled. One mesh, so the game stacks
    and merges it."""
    bm = bmesh.new()
    for x in (-0.5, 0.5):
        add_box(bm, (x, 0.0, 0.9), (0.11, 0.11, 1.8))
        add_box(bm, (x, 0.0, 0.9), (0.16, 0.03, 1.8))
    add_box(bm, (0.0, 0.0, 1.74), (1.1, 0.09, 0.09))
    add_box(bm, (0.0, 0.03, 1.69), (1.1, 0.04, 0.05))
    length = math.hypot(1.0, 1.55)
    ang = math.degrees(math.atan2(1.0, 1.55))
    for sgn in (-1.0, 1.0):
        add_box(bm, (0.0, 0.04, 0.9), (0.06, 0.025, length), rot=(0.0, sgn * ang, 0.0))
    for (x, z) in ((-0.5, 0.12), (0.5, 0.12), (-0.5, 1.68), (0.5, 1.68), (0.0, 0.9)):
        add_box(bm, (x, 0.055, z), (0.2, 0.02, 0.2), rot=(0.0, 45.0, 0.0))
        for (dx, dz) in ((-0.05, 0.0), (0.05, 0.0), (0.0, -0.05), (0.0, 0.05)):
            ret = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=0.018)
            for v in ret["verts"]:
                v.co += Vector((x + dx, 0.07, z + dz))
    root = empty("truss_bay")
    to_object("truss", bm, (0.5, 0.52, 0.6), smooth=False, bevel=0.01).parent = root
    return root


def moss_clump(name, seed):
    """A clump of moss (about 1 across, its base on the origin; the gauntlet's round 9: identical round moss balls under the grove's
    platforms), folded like the grove's crowns (2026-10-19): a few small `folded_dome`s of their own sizes heaped together and squashed,
    the whole a little lopsided, so it reads as leafy folds in the stage's light rather than a smooth lump."""
    rng = random.Random(seed)
    root = empty(name)
    bm = bmesh.new()
    lumps = [((0.0, 0.0, 0.05), 0.34)]
    for k in range(rng.randint(2, 4)):
        a = math.tau * k / 3.5 + rng.uniform(-0.4, 0.4)
        r = rng.uniform(0.18, 0.28)
        d = rng.uniform(0.22, 0.34)
        lumps.append(((math.cos(a) * d, math.sin(a) * d * 0.7, 0.0), r))
    for k, (c, r) in enumerate(lumps):
        folded_dome(bm, bm, c, r, 5, random.Random(seed * 10 + k))
    for v in bm.verts:
        v.co.z = max(v.co.z, -0.02) * 0.7
    to_object("moss", bm, (0.36, 0.64, 0.36), smooth=False).parent = root
    return root


def _lathe_bm(bm, prof, sides=24, wobble=None):
    rings = []
    for k, (r, z) in enumerate(prof):
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
    # (Faces outward whichever way the profile runs: the toadstool's cap, drawn top down, faced in and shaded dark.)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return rings


def mushroom():
    """A toadstool (size 1: about 1 tall, standing on the origin; the gauntlet's round 10, in place of a cylinder under a half ball): a
    stem that swells at its foot and bends a little, a domed cap with a rolled lip, gills under it, and pale warts on top."""
    rng = random.Random(23)
    root = empty("mushroom")
    bm = bmesh.new()
    prof = [(0.17, 0.0), (0.15, 0.08), (0.12, 0.25), (0.11, 0.45), (0.115, 0.62), (0.1, 0.7)]
    rings = _lathe_bm(bm, prof, 18)
    for k, ring in enumerate(rings):
        for v in ring:
            v.co.x += 0.04 * math.sin(k / 5.0 * math.pi * 0.8)
    to_object("stem", bm, (0.97, 0.93, 0.84), smooth=True).parent = root
    bm = bmesh.new()
    prof = [(0.0, 1.02), (0.18, 1.0), (0.32, 0.94), (0.42, 0.84), (0.46, 0.74), (0.44, 0.68), (0.36, 0.68)]
    rings = _lathe_bm(bm, prof, 28, wobble=lambda a, k: 1.0 + 0.03 * math.sin(a * 3.0 + 1.0) * (k / 6.0))
    to_object("cap", bm, (0.9, 0.5, 0.58), smooth=True).parent = root
    bm = bmesh.new()
    prof = [(0.36, 0.69), (0.12, 0.72), (0.1, 0.73)]
    rings = _lathe_bm(bm, prof, 28, wobble=lambda a, k: 1.0 + (0.06 * math.cos(a * 14.0) if k == 0 else 0.0))
    to_object("gills", bm, (0.86, 0.76, 0.66), smooth=False).parent = root
    for k in range(5):
        a = math.tau * k / 5.0 + rng.uniform(-0.3, 0.3)
        d = rng.uniform(0.12, 0.3)
        bm = bmesh.new()
        ret = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=rng.uniform(0.04, 0.065))
        z = 1.025 - 0.75 * d * d
        for v in ret["verts"]:
            v.co = Vector((v.co.x, v.co.y, v.co.z * 0.6)) + Vector((math.cos(a) * d, math.sin(a) * d, z))
        to_object("spot%d" % k, bm, (1.0, 0.97, 0.93), smooth=True).parent = root
    return root


def stump():
    """A tree stump (about 1.2 across at its foot and 0.75 tall, on the origin): a trunk that flares into four roots gripping the ground,
    bark with deep vertical furrows, a bark lip standing a little above the cut top, and the top with growth rings and a crack."""
    rng = random.Random(31)
    root = empty("stump")
    bm = bmesh.new()
    prof = [(0.66, 0.0), (0.56, 0.08), (0.5, 0.2), (0.47, 0.45), (0.46, 0.66), (0.47, 0.74)]
    def wob(a, k):
        roots = 0.35 * max(0.0, math.cos(2.0 * (a + 0.4))) ** 3 * max(0.0, 1.0 - k / 3.0)
        furrow = 0.035 * math.cos(a * 16.0 + rng.uniform(0, 0.01))
        return 1.0 + roots + furrow
    rings = _lathe_bm(bm, prof, 48, wobble=wob)
    to_object("bark", bm, (0.5, 0.36, 0.26), smooth=True).parent = root
    bm = bmesh.new()
    rings = _lathe_bm(bm, [(0.47, 0.74), (0.42, 0.73), (0.0, 0.72)], 48, wobble=lambda a, k: 1.0 + 0.035 * math.cos(a * 16.0))
    to_object("top", bm, (0.9, 0.76, 0.54), smooth=False).parent = root
    for k, r in enumerate((0.12, 0.22, 0.32)):
        bm = bmesh.new()
        ring(bm, r, 0.008, 0.726 + 0.001 * k, sides=36, minor_sides=4)
        to_object("ring%d" % k, bm, (0.72, 0.56, 0.38), smooth=True).parent = root
    return root


def lamp_post():
    """A harbour lamp post (2.4 tall, on the origin): a square timber post on a stone footing, an iron bracket arm with a curl, and a glass
    lantern hanging from it: an iron cap and base, four glass panes in an iron frame, a ring on top."""
    root = empty("lamp_post")
    bm = bmesh.new()
    add_box(bm, (0.0, 0.0, 0.15), (0.34, 0.34, 0.3))
    to_object("footing", bm, (0.62, 0.6, 0.58), smooth=False, bevel=0.03).parent = root
    bm = bmesh.new()
    add_box(bm, (0.0, 0.0, 1.25), (0.14, 0.14, 2.2), taper=0.12)
    to_object("post", bm, (0.48, 0.34, 0.26), smooth=False, bevel=0.02).parent = root
    bm = bmesh.new()
    add_box(bm, (0.3, 0.0, 2.25), (0.66, 0.05, 0.06))
    add_box(bm, (0.12, 0.0, 2.1), (0.3, 0.04, 0.04), rot=(0.0, 40.0, 0.0))
    ring(bm, 0.05, 0.012, 0.0, sides=12, minor_sides=4)
    to_object("iron_arm", bm, (0.22, 0.22, 0.26), smooth=False).parent = root
    for v in root.children[-1].data.vertices:
        pass
    cx, cz = 0.58, 1.88
    bm = bmesh.new()
    for z, w in ((cz + 0.2, 0.17), (cz - 0.2, 0.15)):
        add_box(bm, (cx, 0.0, z), (w, w, 0.05))
    for dx in (-0.075, 0.075):
        for dy in (-0.075, 0.075):
            add_box(bm, (cx + dx, dy, cz), (0.02, 0.02, 0.4))
    cone(bm, 0.12, 0.02, 0.12, z0=cz + 0.22, sides=8, at=(cx, 0.0))
    add_box(bm, (cx, 0.0, cz + 0.3), (0.02, 0.02, 0.14))
    to_object("iron_lantern", bm, (0.22, 0.22, 0.26), smooth=False).parent = root
    bm = bmesh.new()
    add_box(bm, (cx, 0.0, cz), (0.14, 0.14, 0.36))
    to_object("glow", bm, (1.0, 0.86, 0.56), smooth=False).parent = root
    return root


def _on_side(bm, lift):
    """Lay a part built round the z axis on its side (its axis along y, toward the game's camera: a quarter turn about x) and lift it by
    `lift`."""
    for v in bm.verts:
        x, y, z = v.co
        v.co = Vector((x, -z, y + lift))


def _wrap_uv(bm, u_repeat, z0, z1):
    """UVs for a part built round the z axis: u round it `u_repeat` times, v along it from z0 to z1 (the game's painted straw)."""
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        us = [math.atan2(l.vert.co.y, l.vert.co.x) / math.tau % 1.0 for l in f.loops]
        seam = max(us) - min(us) > 0.5
        for l, u in zip(f.loops, us):
            if seam and u < 0.5:
                u += 1.0
            l[uv].uv = (u * u_repeat, (l.vert.co.z - z0) / (z1 - z0))


def _flat_uv(bm, radius):
    """UVs laid flat across a part built round the z axis: the middle at (0.5, 0.5), `radius` at the edge."""
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.5 + 0.5 * l.vert.co.x / radius, 0.5 + 0.5 * l.vert.co.y / radius)


def hay_bale():
    """A round bale lying on its side (1.3 long, 0.57 in radius, resting on the origin; the gauntlet's round 12, in place of the built
    drum): a body rolled in uneven layers, lumpy and a touch fuller in the middle, its rim rounding over into ends dished a little
    inward with the roll's spiral as raised grooves, two twine bands sunk into the straw, and loose wisps lying across the body."""
    rng = random.Random(53)
    root = empty("hay_bale")
    lift = 0.55
    bm = bmesh.new()
    prof = [(0.44, -0.65), (0.5, -0.635), (0.535, -0.6), (0.552, -0.54), (0.56, -0.42), (0.565, -0.2), (0.567, 0.0),
            (0.565, 0.2), (0.56, 0.42), (0.552, 0.54), (0.535, 0.6), (0.5, 0.635), (0.44, 0.65)]
    phase = [rng.uniform(0, math.tau) for _ in range(4)]
    def lumps(a, k):
        t = k / (len(prof) - 1.0)
        return (1.0 + 0.018 * math.sin(a * 5.0 + phase[0] + t * 3.0) + 0.012 * math.sin(a * 11.0 + phase[1])
                + 0.01 * math.sin(t * 19.0 + phase[2]))
    _lathe_bm(bm, prof, 40, wobble=lumps)
    _wrap_uv(bm, 3.0, -0.65, 0.65)
    _on_side(bm, lift)
    to_object("straw", bm, (0.86, 0.68, 0.36), smooth=True).parent = root
    for side in (-1.0, 1.0):
        bm = bmesh.new()
        end = [(0.44, 0.65), (0.3, 0.635), (0.15, 0.625), (0.0, 0.62)]
        _lathe_bm(bm, [(r, z * side) for r, z in end], 40)
        if side < 0.0:
            bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
        _flat_uv(bm, 0.47)
        _on_side(bm, lift)
        to_object("end%d" % int(side), bm, (0.9, 0.74, 0.44), smooth=True).parent = root
        bm = bmesh.new()
        # The roll's spiral: a raised groove winding out from the core, drawn as a chain of short tubes.
        turns, steps = 4.0, 120
        prev = None
        for i in range(steps + 1):
            t = i / steps
            a = t * turns * math.tau
            r = 0.05 + 0.36 * t
            z = side * (0.628 + 0.012 * t)
            c = Vector((r * math.cos(a), r * math.sin(a), z))
            if prev is not None:
                d = c - prev
                mid = (c + prev) / 2.0
                ret = bmesh.ops.create_cone(bm, cap_ends=False, cap_tris=False, segments=5, radius1=0.012, radius2=0.012,
                                            depth=d.length * 1.15)
                q = d.normalized().to_track_quat("Z", "X")
                for v in ret["verts"]:
                    v.co = q @ v.co + mid
            prev = c
        _on_side(bm, lift)
        to_object("groove%d" % int(side), bm, (0.7, 0.52, 0.28), smooth=True).parent = root
    for dz in (-0.3, 0.3):
        bm = bmesh.new()
        ring(bm, 0.562, 0.016, dz, sides=40, minor_sides=6)
        _on_side(bm, lift)
        to_object("twine%d" % int(dz * 10), bm, (0.62, 0.4, 0.24), smooth=True).parent = root
    bm = bmesh.new()
    for w in range(16):
        a = rng.uniform(0.0, math.tau)
        z0 = rng.uniform(-0.55, 0.55)
        length = rng.uniform(0.18, 0.34)
        tilt = rng.uniform(-0.9, 0.9)
        left, right = [], []
        for k in range(5):
            t = k / 4.0
            aa = a + math.cos(tilt) * length * t / 0.57
            z = z0 + math.sin(tilt) * length * t
            r = 0.575 + 0.05 * t * t
            c = Vector((r * math.cos(aa), r * math.sin(aa), z))
            n = Vector((math.cos(aa), math.sin(aa), 0.0))
            side_v = n.cross(Vector((0.0, 0.0, 1.0))).normalized() * math.sin(tilt) + Vector((0, 0, 1)) * -math.cos(tilt) * 0.0
            wv = Vector((0.0, 0.0, 0.012 * (1.0 - t) + 0.003))
            left.append(bm.verts.new(c - wv))
            right.append(bm.verts.new(c + wv))
        for k in range(4):
            bm.faces.new((left[k], right[k], right[k + 1], left[k + 1]))
    _on_side(bm, lift)
    to_object("wisps", bm, (0.94, 0.8, 0.5), smooth=True).parent = root
    return root


def mountain_range(name, seed, height, peaks, base, amplitude, rounded, res=(260, 40)):
    """A far range of mountains or hills as a landform (the gauntlet's round 13, in place of a painted cut card: flat outlined shapes in
    the distance were the strongest cheap cue): 520 across (x, centred), `height` at its tallest, about as deep as it is tall (y, away
    from the camera, its front foot on the origin). A crest line that wanders in depth, `peaks` main summits with smaller ones between
    (sharp, or `rounded` hills), and spurs and gullies running down its front, so the stage's light shades every flank. One smooth mesh,
    its foot carried below the horizon."""
    rng = random.Random(seed)
    root = empty(name)
    W, D = 520.0, height * 1.1
    tops = []
    n = int(peaks)
    for i in range(n):
        x = -W / 2 + W * (i + 0.5) / n + rng.uniform(-0.25, 0.25) * W / n
        tops.append((x, base + amplitude * rng.uniform(0.6, 1.0), W / n * rng.uniform(0.75, 1.15)))
    for i in range(2 * n):
        x = rng.uniform(-W / 2, W / 2)
        tops.append((x, base + amplitude * rng.uniform(0.25, 0.55), W / n * rng.uniform(0.35, 0.6)))

    def crest(x):
        h = base * 0.8
        for (px, ph, pw) in tops:
            t = min(abs(x - px) / pw, 1.0)
            f = (0.5 + 0.5 * math.cos(math.pi * t)) if rounded else (1.0 - t) ** 1.15
            h = max(h, ph * f + base * 0.8 * (1.0 - f))
        return h

    ph = rng.uniform(0, math.tau)
    # (`res`: a coarse grid for faceted mountains, big clean planes as the papercraft camp's, 2026-10-19.)
    nx, nd = res
    bm = bmesh.new()
    grid = []
    for j in range(nd + 2):
        row = []
        for i in range(nx + 1):
            x = -W / 2 + W * i / nx
            if j == 0:
                row.append(bm.verts.new((x, -D * 0.05, -height * 0.6)))
                continue
            d = D * (j - 1) / nd
            ridge_d = D * (0.5 + 0.12 * math.sin(x * 0.017 + ph))
            span = ridge_d if d < ridge_d else (D - ridge_d)
            t = min(abs(d - ridge_d) / span, 1.0)
            prof = 1.0 - t ** (2.0 if rounded else 1.4)
            # Spurs and gullies down the front: a ripple across x that grows toward the foot.
            ripple = math.sin(x * (0.06 if rounded else 0.11) + 2.5 * math.sin(x * 0.021 + ph)) * (0.12 if rounded else 0.06)
            z = height * crest(x) * prof * (1.0 + ripple * t)
            row.append(bm.verts.new((x, d, z)))
        grid.append(row)
    for j in range(nd + 1):
        for i in range(nx):
            bm.faces.new((grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    to_object("land", bm, (0.5, 0.6, 0.6), smooth=True).parent = root
    return root


def hill_dome(name, seed):
    """A rounded hill at unit size (radius 1 across, 1 tall, centred on the origin, its foot carried below it), folded like the ranges
    (the owner's blue-mountain reference, 2026-10-19, in place of a lumpy faceted sphere that read as a pale glassy blob): ridges running
    down from its crown with a crease between each pair, so its flanks are big planes wholly lit or in shade. Its ridges follow the
    ellipsoid the game seats the town's houses on (`_harbour_town`'s `ground`), its creases sink a little under it."""
    rng = random.Random(seed)
    root = empty(name)
    bm = bmesh.new()
    # (Six, three of them strong spurs standing proud, the judge's round 40: nine even ridges read as one faceted dome at its size.)
    ridges = 6
    turn = rng.uniform(0, math.tau)
    apex = bm.verts.new((rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05), 1.0))
    lines = []
    for k in range(ridges * 2):
        ridge = k % 2 == 0
        a = turn + math.tau * k / (ridges * 2) + rng.uniform(-0.1, 0.1)
        spur = ridge and (k // 2) % 2 == 0
        reach = (rng.uniform(1.02, 1.08) if spur else rng.uniform(0.95, 1.0)) if ridge else rng.uniform(0.78, 0.84)
        pts = []
        for t in (0.32, 0.58, 0.8, 0.94, 1.0):
            r = t * reach
            z = math.sqrt(max(0.0, 1.0 - t * t)) * (1.0 if ridge else 0.86)
            pts.append(bm.verts.new((math.cos(a) * r, math.sin(a) * r, z)))
        # The foot carried under the waterline.
        pts.append(bm.verts.new((math.cos(a) * reach, math.sin(a) * reach, -0.4)))
        lines.append(pts)
    m = len(lines)
    for k in range(m):
        p, q = lines[k], lines[(k + 1) % m]
        bm.faces.new((apex, p[0], q[0]))
        for j in range(len(p) - 1):
            bm.faces.new((p[j], p[j + 1], q[j + 1]))
            bm.faces.new((p[j], q[j + 1], q[j]))
    bm.normal_update()
    for f in bm.faces:
        if f.normal.dot(f.calc_center_median() - Vector((0.0, 0.0, -0.2))) < 0.0:
            f.normal_flip()
    to_object("land", bm, (0.4, 0.56, 0.46), smooth=False).parent = root
    return root


def log_pole():
    """A round log for the meadow's platforms to rest on (unit size: radius 1, 1 long along y, centred; the game scales it to each
    platform): bark in long rounded ridges that wander and split a little, the log a hair bowed and uneven in girth, each end sawn flat
    with growth rings and a lighter rim of sapwood, a chip off one edge. Parts `bark` and `end`, coloured by the game."""
    rng = random.Random(113)
    root = empty("log_pole")
    bm = bmesh.new()
    sides, rows = 28, 24
    ph = [rng.uniform(0, math.tau) for _ in range(4)]
    rings = []
    for k in range(rows + 1):
        t = k / rows
        y = t - 0.5
        loop = []
        for i in range(sides):
            a = math.tau * i / sides
            ridge = 0.06 * abs(math.sin(a * 5.0 + 0.6 * math.sin(t * 7.0 + ph[0]) + ph[1]))
            r = 1.0 + ridge + 0.04 * math.sin(t * 9.0 + ph[2]) * math.cos(a * 2.0)
            loop.append(bm.verts.new((math.cos(a) * r + 0.05 * math.sin(t * math.pi), y, math.sin(a) * r)))
        rings.append(loop)
    for k in range(rows):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    ends = [[v.co.copy() for v in rings[0]], [v.co.copy() for v in rings[-1]]]
    to_object("bark", bm, (0.42, 0.3, 0.22), smooth=True).parent = root
    for side in (-1.0, 1.0):
        bm = bmesh.new()
        src = ends[0] if side < 0 else ends[1]
        centre = bm.verts.new((0.05 * math.sin(0.0 if side < 0 else math.pi), 0.5 * side, 0.0))
        rim = [bm.verts.new((v.x * 0.97, v.y, v.z * 0.97)) for v in src]
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((centre, rim[j], rim[i]) if side > 0 else (centre, rim[i], rim[j]))
        to_object("end%d" % int(side), bm, (0.86, 0.72, 0.52), smooth=False).parent = root
        for r in (0.3, 0.55, 0.8):
            b = bmesh.new()
            ring(b, r, 0.025, 0.0, sides=28, minor_sides=4)
            for v in b.verts:
                x, y, z = v.co
                v.co = Vector((x, 0.5 * side + 0.004 * side, y))
            to_object("ring%d_%d" % (int(side), int(r * 10)), b, (0.66, 0.5, 0.34), smooth=True).parent = root
    return root


def rope_lash():
    """A rope lashing (unit size: 1 tall along z, about 0.3 wide, its loops lying in the x-z plane): three turns of rope, each a tall
    rounded loop of two twisted strands, side by side; the game sets it down the front of a platform's board and round its log."""
    root = empty("rope_lash")
    bm = bmesh.new()
    for turn in range(3):
        x0 = (turn - 1) * 0.1
        path = []
        n = 72
        for i in range(n):
            u = i / n
            # A stadium: up one side, over the top, down the other, under the bottom.
            a = u * math.tau
            px = x0 + 0.04 * math.cos(a)
            pz = 0.5 * max(-1.0, min(1.0, math.sin(a) * 1.6))
            path.append(Vector((px, 0.06 * math.cos(a), pz)))
        for strand in range(2):
            loops = []
            for i in range(n):
                c = path[i]
                d = (path[(i + 1) % n] - path[i - 1]).normalized()
                side = d.cross(Vector((0.0, 1.0, 0.0))).normalized()
                up = d.cross(side).normalized()
                tw = i / n * math.tau * 14.0 + strand * math.pi
                cc = c + (side * math.cos(tw) + up * math.sin(tw)) * 0.012
                ring_v = []
                for j in range(6):
                    b = math.tau * j / 6
                    ring_v.append(bm.verts.new(cc + (side * math.cos(b) + up * math.sin(b)) * 0.014))
                loops.append(ring_v)
            for i in range(n):
                for j in range(6):
                    a0, b0 = loops[i][j], loops[(i + 1) % n][j]
                    c0, d0 = loops[(i + 1) % n][(j + 1) % 6], loops[i][(j + 1) % 6]
                    bm.faces.new((a0, b0, c0, d0))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    to_object("rope", bm, (0.86, 0.74, 0.52), smooth=True).parent = root
    return root


def pine():
    """A pine in the papercraft camp's language (the owner's blue-mountain reference, 2026-10-19): five stacked pleated tiers
    (`pleated_tier`), each overlapping the one under it and a hair turned, narrowing to a spire, over a short trunk. About 1.6 wide and
    2.5 tall, standing on the origin. Parts `needles_lit` (the folded skirts, flat-shaded), `under` (their dark undersides) and
    `trunk`."""
    rng = random.Random(127)
    root = empty("pine")
    out, under = bmesh.new(), bmesh.new()
    tiers = 5
    for k in range(tiers):
        t = k / (tiers - 1)
        r = 0.8 * (1.0 - 0.72 * t)
        z0 = 0.32 + 0.36 * k
        h = 0.78 - 0.08 * k if k < tiers - 1 else 0.9
        pleated_tier(out, under, (0.0, 0.0), z0, r, h, 7 if k < 3 else 6, rng.uniform(0, math.tau), rng, flare=0.5, droop=0.2 - 0.025 * k)
    for bm in (out, under):
        outward(bm, (0.0, 0.0, 0.0))
    to_object("needles_lit", out, (0.3, 0.5, 0.38), smooth=False).parent = root
    to_object("under", under, (0.12, 0.22, 0.2), smooth=False).parent = root
    bm = bmesh.new()
    cone(bm, 0.09, 0.06, 0.55, 0.0, sides=6)
    to_object("trunk", bm, (0.32, 0.24, 0.2), smooth=False).parent = root
    return root


def bush():
    """A bush in the grove crowns' folded language (the owner, 2026-10-19: round, as the trees are): a heap of four `folded_dome`s, a
    big one in the middle, two lower at its sides and one behind, about 2.4 wide, 1.3 tall and 1.6 deep, standing on the origin. Parts
    `leaf_lit` (the domes, flat-shaded, so the light falls in folds) and `leaf_shade` (their leafy skirts and undersides)."""
    root = empty("bush")
    lit, shade = bmesh.new(), bmesh.new()
    for k, (c, r) in enumerate((((0.0, 0.0, 0.3), 0.72), ((-0.72, 0.05, 0.16), 0.5), ((0.74, 0.0, 0.18), 0.52), ((0.15, 0.35, 0.5), 0.48))):
        folded_dome(lit, shade, c, r, 6 if r > 0.6 else 5, random.Random(131 + k))
    to_object("leaf_shade", shade, (0.2, 0.46, 0.3), smooth=False).parent = root
    to_object("leaf_lit", lit, (0.46, 0.74, 0.36), smooth=False).parent = root
    return root


def folded_range(name, seed, height, peaks, rounded, plateau=0.0):
    """A range of mountains folded like paper (the owner's blue-mountain reference, 2026-10-19, in place of `mountain_range`'s faceted
    terrain grid, which read as random low-poly noise): every peak an apex with ridges running straight down from it to the foot and a
    crease between each pair, so its flanks are a few big planes, each wholly lit or wholly in shade. `peaks` main summits across 520
    (x, centred), `height` at the tallest, smaller summits and foothills in front of and between them, all one open shell whose foot is
    carried below the horizon; about as deep as it is tall (y, away from the camera, its front foot on the origin). `rounded`: lower,
    broader summits with blunt tops (the grove's hills)."""
    rng = random.Random(seed)
    root = empty(name)
    W = 520.0
    bm = bmesh.new()
    spots = []
    n = int(peaks)
    for i in range(n):
        x = -W / 2 + W * (i + 0.5) / n + rng.uniform(-0.2, 0.2) * W / n
        spots.append((x, height * rng.uniform(0.5, 0.65), height * rng.uniform(0.72, 1.0), W / n * rng.uniform(1.1, 1.4)))
    # (One summit dominant and the rest clearly lower, the judge's round 39: summits of near-equal height read as a row of pyramids.
    # The dominant one is the main summit nearest a third of the way across the frame (x about -40), off the middle of the fight.)
    if not rounded:
        lead = min(range(n), key=lambda i: abs(spots[i][0] + 40.0))
        for i in range(n):
            x, y, h, reach = spots[i]
            spots[i] = (x, y, height if i == lead else h * 0.72, reach * (1.15 if i == lead else 1.0))
    # (Spur summits on the main peaks' flanks, lower and a little nearer, so each mountain is a crumpled mass of folds with secondary
    # tops, as the reference's are, not one clean pyramid.)
    for (x, y, h, reach) in list(spots):
        for side in (-1.0, 1.0):
            if rng.random() < 0.75:
                spots.append((x + side * reach * rng.uniform(0.35, 0.6), y - height * rng.uniform(0.05, 0.15), h * rng.uniform(0.55, 0.78),
                              reach * rng.uniform(0.5, 0.7)))
    # (A back row between the summits and foothills in front: one continuous massif with shoulders, not a row of separate cones.)
    for i in range(n + 1):
        x = -W / 2 + W * i / n + rng.uniform(-0.15, 0.15) * W / n
        spots.append((x, height * rng.uniform(0.75, 0.9), height * rng.uniform(0.5, 0.7), W / n * rng.uniform(1.0, 1.3)))
    for i in range(n + 1):
        x = -W / 2 + W * i / n + rng.uniform(-0.3, 0.3) * W / n
        spots.append((x, height * rng.uniform(0.15, 0.28), height * rng.uniform(0.26, 0.4), W / n * rng.uniform(0.8, 1.0)))
    for (px, py, h, reach) in spots:
        folded_peak(bm, px, py, h, reach, -0.35 * height, rng, rounded)
    # (`plateau`: every summit cut off flat at that share of the height, a range of mesas for the canyon, the judge's round 40.)
    if plateau > 0.0:
        for v in bm.verts:
            v.co.z = min(v.co.z, height * plateau)
    bm.normal_update()
    for f in bm.faces:
        if f.normal.z < 0.0:
            f.normal_flip()
    to_object("land", bm, (0.5, 0.6, 0.6), smooth=False).parent = root
    return root


def ac_unit():
    """A rooftop air conditioner (1.6 wide, 1.1 tall, 1.2 deep, on the origin, its front -y): a bevelled cabinet on a plinth, a louvred
    grille across the front, a round fan grille on top with a hub, and a pipe running down its side. Parts: `body`, `grille`, `fan`, `pipe`."""
    root = empty("ac_unit")
    bm = bmesh.new()
    add_box(bm, (0.0, 0.0, 0.06), (1.8, 1.4, 0.12))
    add_box(bm, (0.0, 0.0, 0.62), (1.6, 1.2, 1.0))
    to_object("body", bm, (0.66, 0.68, 0.76), smooth=False, bevel=0.04).parent = root
    bm = bmesh.new()
    for k in range(6):
        add_box(bm, (0.0, -0.61, 0.28 + k * 0.12), (1.3, 0.05, 0.05), rot=(math.radians(-25), 0.0, 0.0))
    to_object("grille", bm, (0.42, 0.42, 0.52), smooth=False).parent = root
    bm = bmesh.new()
    ring(bm, 0.42, 0.04, 1.14, sides=20, minor_sides=6)
    cone(bm, 0.36, 0.36, 0.03, z0=1.1, sides=20)
    cone(bm, 0.12, 0.1, 0.08, z0=1.12, sides=10)
    to_object("fan", bm, (0.38, 0.38, 0.48), smooth=True).parent = root
    bm = bmesh.new()
    cone(bm, 0.06, 0.06, 0.9, z0=0.15, sides=8, at=(0.86, 0.3))
    to_object("pipe", bm, (0.56, 0.58, 0.66), smooth=True).parent = root
    return root


def roof_vent():
    """A gooseneck roof vent (about 1.2 tall, on the origin): a flanged foot, a round stack bending over into a hooded mouth facing
    down and to the side, a band round the stack."""
    root = empty("roof_vent")
    bm = bmesh.new()
    cone(bm, 0.42, 0.36, 0.08, z0=0.0, sides=14)
    cone(bm, 0.22, 0.22, 0.7, z0=0.08, sides=14)
    to_object("stack", bm, (0.62, 0.64, 0.72), smooth=True).parent = root
    bm = bmesh.new()
    # (One continuous bend from the top of the stack, up, over and down, ending in the hood, so nothing floats: the owner, 2026-10-19.)
    # (Swept as one tube with shared rings, the owner, 2026-10-19: overlapping short pieces left gaps and read as detached.)
    n, sides = 16, 14
    path = [Vector((0.3 - 0.3 * math.cos(math.pi * t / n), 0.0, 0.78 + 0.3 * math.sin(math.pi * t / n))) for t in range(n + 1)]
    rings = []
    for t in range(n + 1):
        a = math.pi * t / n
        tangent = Vector((math.sin(a), 0.0, math.cos(a))).normalized()
        side = Vector((0.0, 1.0, 0.0))
        up = tangent.cross(side).normalized()
        rings.append([bm.verts.new(path[t] + (side * math.cos(math.tau * k / sides) + up * math.sin(math.tau * k / sides)) * 0.22)
                      for k in range(sides)])
    for t in range(n):
        for k in range(sides):
            bm.faces.new((rings[t][k], rings[t][(k + 1) % sides], rings[t + 1][(k + 1) % sides], rings[t + 1][k]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    cone(bm, 0.32, 0.23, 0.18, z0=0.62, sides=14, at=(0.6, 0.0))
    to_object("hood", bm, (0.56, 0.58, 0.66), smooth=True).parent = root
    bm = bmesh.new()
    ring(bm, 0.24, 0.025, 0.5, sides=14, minor_sides=4)
    to_object("band", bm, (0.42, 0.42, 0.52), smooth=True).parent = root
    return root


def street_lamp():
    """A city street lamp (4.2 tall, on the origin): a fluted iron post on a stepped base, a curved arm and a lantern head with a
    glowing panel underneath. Parts: `iron`, `glow`."""
    root = empty("street_lamp")
    bm = bmesh.new()
    cone(bm, 0.24, 0.2, 0.3, z0=0.0, sides=8)
    cone(bm, 0.12, 0.08, 3.7, z0=0.3, sides=8)
    add_box(bm, (0.35, 0.0, 4.0), (0.8, 0.08, 0.08))
    cone(bm, 0.12, 0.28, 0.24, z0=3.82, sides=8, at=(0.72, 0.0))
    to_object("iron", bm, (0.24, 0.22, 0.3), smooth=False).parent = root
    bm = bmesh.new()
    cone(bm, 0.22, 0.22, 0.06, z0=3.78, sides=8, at=(0.72, 0.0))
    to_object("glow", bm, (1.0, 0.86, 0.56), smooth=False).parent = root
    return root


def car():
    """A chunky toy-like car (3.4 long along x, 1.6 wide, 1.4 tall, on the origin): a rounded body, a smaller cabin with glass, four
    wheels, headlights at +x and tail lights at -x. Parts: `body` (the game colours it), `glass`, `wheels`, `head`, `tail`."""
    root = empty("car")
    bm = bmesh.new()
    add_box(bm, (0.0, 0.0, 0.6), (3.4, 1.5, 0.6))
    to_object("body", bm, (0.8, 0.3, 0.3), smooth=False, bevel=0.12).parent = root
    bm = bmesh.new()
    add_box(bm, (-0.2, 0.0, 1.12), (1.8, 1.3, 0.5), taper=0.15)
    to_object("glass", bm, (0.36, 0.42, 0.56), smooth=False, bevel=0.08).parent = root
    bm = bmesh.new()
    for x in (-1.05, 1.05):
        for y in (-0.72, 0.72):
            ret = bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=12, radius1=0.34, radius2=0.34, depth=0.28)
            for v in ret["verts"]:
                v.co = Vector((v.co.x, v.co.z, v.co.y)) + Vector((x, y, 0.34))
    to_object("wheels", bm, (0.12, 0.1, 0.14), smooth=True).parent = root
    for name, x, col in (("head", 1.71, (1.0, 0.95, 0.8)), ("tail", -1.71, (1.0, 0.25, 0.25))):
        bm = bmesh.new()
        for y in (-0.5, 0.5):
            add_box(bm, (x, y, 0.68), (0.04, 0.3, 0.16))
        to_object(name, bm, col, smooth=False).parent = root
    return root


PROPS = {
    "rock_arch": rock_arch,
    "ac_unit": ac_unit,
    "roof_vent": roof_vent,
    "street_lamp": street_lamp,
    "car": car,
    "bush": bush,
    "pine": pine,
    "log_pole": log_pole,
    "rope_lash": rope_lash,
    "hill_dome_a": lambda: hill_dome("hill_dome_a", 5),
    "hill_dome_b": lambda: hill_dome("hill_dome_b", 12),
    "range_meadow_far": lambda: folded_range("range_meadow_far", 3, 88.0, 6, False),
    "range_meadow_near": lambda: folded_range("range_meadow_near", 9, 62.0, 8, False),
    "range_grove_far": lambda: folded_range("range_grove_far", 8, 52.0, 4, True),
    "range_grove_near": lambda: folded_range("range_grove_near", 4, 40.0, 6, True),
    "range_canyon_far": lambda: folded_range("range_canyon_far", 5, 58.0, 4, False, 0.48),
    "range_canyon_near": lambda: folded_range("range_canyon_near", 11, 46.0, 5, False, 0.44),
    "hay_bale": hay_bale,
    "mushroom": mushroom,
    "stump": stump,
    "lamp_post": lamp_post,
    "moss_a": lambda: moss_clump("moss_a", 3),
    "moss_b": lambda: moss_clump("moss_b", 8),
    "moss_c": lambda: moss_clump("moss_c", 13),
    "truss_bay": truss_bay,
    "grass_tuft": grass_tuft,
    "crate": crate,
    "barrel": barrel,
    "lighthouse": lighthouse,
    "sailboat": sailboat,
    "mesa_a": lambda: mesa("mesa_a", 3, 24.0, 15.0, 4),
    "mesa_b": lambda: mesa("mesa_b", 8, 13.0, 22.0, 5),
    "cactus": lambda: cactus(),
    "palm_crown": lambda: palm_crown(),
    "tree_trunk": tree_trunk,
    "cottage": cottage,
    "windmill_tower": windmill_tower,
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
    cam_data.clip_end = size * 10.0
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
