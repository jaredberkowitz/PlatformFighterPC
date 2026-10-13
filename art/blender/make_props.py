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

# The palette (linear-ish sRGB as the exporter writes base colours; the game's cel material and stage light do the rest).
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


def material(name, colour):
    m = bpy.data.materials.get(name)
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


PROPS = {
    "lighthouse": lighthouse,
    "sailboat": sailboat,
    "mesa_a": lambda: mesa("mesa_a", 3, 24.0, 15.0, 4),
    "mesa_b": lambda: mesa("mesa_b", 8, 13.0, 22.0, 5),
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
