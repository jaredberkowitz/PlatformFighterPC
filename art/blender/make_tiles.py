"""Painted ground tiles rendered from modelled geometry (the owner, 2026-10-18: new painted materials with variation built in; the
noise tiles read as procedural). Each tile is a small patch of real surface built in Blender (grass blades in clumps; clods of earth
with stones bedded in them), repeated across its borders so the render tiles seamlessly, lit softly the way the game lights that
surface, and rendered straight down (orthographic, Cycles, so the clumps take soft occlusion and their own light and shade). The
render is turned into a greyscale value tile in the format of tools/art/noise_tile.gd: the stage colour is multiplied by it, so one
tile serves every hue.

    blender --background --python art/blender/make_tiles.py -- --only lawn,soil_bank

Writes godot/art/stage/png/<name>.png. Deterministic (seeded).
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

OUT = "godot/art/stage/png"
SIZE = 512
# The tile is 1 x 1 in Blender units, centred on the origin, seen from above; image up is +y.


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    try:
        scene.render.engine = "CYCLES"
    except TypeError:
        pass
    scene.cycles.samples = 48
    scene.cycles.use_denoising = True
    scene.render.resolution_x = SIZE
    scene.render.resolution_y = SIZE
    scene.render.film_transparent = False
    scene.view_settings.view_transform = "Standard"
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
    bg.inputs[0].default_value = (1.0, 1.0, 1.0, 1.0)
    bg.inputs[1].default_value = 0.35
    scene.world = world
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = 1.0
    cam = bpy.data.objects.new("cam", cam_data)
    cam.location = (0.0, 0.0, 5.0)
    scene.collection.objects.link(cam)
    scene.camera = cam
    return scene


def sun(direction, energy):
    """A soft sun shining along `direction` (toward the ground)."""
    data = bpy.data.lights.new("sun", "SUN")
    data.energy = energy
    data.angle = math.radians(18.0)
    obj = bpy.data.objects.new("sun", data)
    obj.rotation_euler = Vector(direction).normalized().to_track_quat("-Z", "Y").to_euler()
    bpy.context.scene.collection.objects.link(obj)


def grey_material(name, value):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = (value, value, value, 1.0)
    bsdf.inputs["Roughness"].default_value = 1.0
    return m


def link(name, bm, mat):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = True
    obj = bpy.data.objects.new(name, me)
    obj.data.materials.append(mat)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def wrapped(x, y, reach):
    """The copies of a point at (x, y) needed for a thing `reach` across to tile: itself and its shifts by a tile where it nears an edge."""
    out = []
    for dx in (-1.0, 0.0, 1.0):
        for dy in (-1.0, 0.0, 1.0):
            px, py = x + dx, y + dy
            if -0.5 - reach < px < 0.5 + reach and -0.5 - reach < py < 0.5 + reach:
                out.append((px, py))
    return out


def blade(bm, base, out, length, width, lean, twist):
    """One curved, tapering grass blade from `base` leaning along `out`."""
    side = Vector((-out.y, out.x, 0.0)).normalized()
    left, right = [], []
    for k in range(6):
        t = k / 5.0
        w = width * (1.0 - t) + width * 0.08
        c = base + out * (lean * length * t * t) + Vector((0.0, 0.0, length * t * (1.0 - 0.3 * t)))
        s = side * math.cos(twist * t) + Vector((0.0, 0.0, 1.0)) * math.sin(twist * t) * 0.3
        left.append(bm.verts.new(c - s * w))
        right.append(bm.verts.new(c + s * w))
    for k in range(5):
        bm.faces.new((left[k], right[k], right[k + 1], left[k + 1]))


def lawn():
    """Turf: about 140 clumps a tile, each 14 to 30 blades fanned from its root, of very different heights, over a darker soil base; the
    tile lies on the ground, and the game's key light comes from the camera's side and the left, so the sun shines from image-down and
    the left."""
    rng = random.Random(11)
    reset()
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=1.5)
    link("soil", bm, grey_material("soil", 0.32))
    shades = [grey_material("blade%d" % k, v) for k, v in enumerate((0.55, 0.68, 0.8, 0.92))]
    bms = [bmesh.new() for _ in shades]
    n = 22
    for j in range(n):
        for i in range(n):
            cx = -0.5 + (i + rng.uniform(0.0, 1.0)) / n
            cy = -0.5 + (j + rng.uniform(0.0, 1.0)) / n
            size = rng.uniform(0.6, 1.4)
            shade = rng.randrange(len(shades))
            blades = []
            for b in range(rng.randint(22, 40)):
                a = rng.uniform(0.0, math.tau)
                blades.append((a, rng.uniform(0.0, 0.012) * size, rng.uniform(0.045, 0.085) * size, rng.uniform(0.005, 0.009),
                               rng.uniform(0.6, 1.3), rng.uniform(-1.0, 1.0)))
            for (px, py) in wrapped(cx, cy, 0.09):
                for (a, r0, length, width, lean, twist) in blades:
                    out = Vector((math.cos(a), math.sin(a), 0.0))
                    base = Vector((px, py, 0.0)) + out * r0
                    blade(bms[shade], base, out, length, width, lean, twist)
    for k, b in enumerate(bms):
        link("blades%d" % k, b, shades[k])
    sun((0.45, 0.75, -0.55), 4.0)
    return bpy.context.scene


def clod(bm, centre, radius, flat, rng):
    """A lump of packed earth: a squashed, lumpy icosphere."""
    ret = bmesh.ops.create_icosphere(bm, subdivisions=2, radius=radius)
    ph = [rng.uniform(0, math.tau) for _ in range(3)]
    for v in ret["verts"]:
        d = v.co.normalized()
        k = 1.0 + 0.16 * math.sin(d.x * 5.0 + ph[0]) * math.sin(d.y * 4.0 + ph[1]) + 0.08 * math.sin(d.z * 7.0 + ph[2])
        v.co = Vector((v.co.x * k * rng.uniform(0.97, 1.03), v.co.y * k, v.co.z * k * flat)) + centre


def soil_bank():
    """A bank of packed earth seen face on: about 50 clods a tile, each its own tone, packed together and overlapping over a dark
    backing, and a few small flat stones bedded among them, a little paler; the game's earth fronts are lit from above and the left, so the sun
    shines from image-up and the left."""
    rng = random.Random(23)
    reset()
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=1.5)
    link("back", bm, grey_material("back", 0.3))
    tones = [grey_material("clod%d" % k, v) for k, v in enumerate((0.5, 0.6, 0.7, 0.8))]
    bms = [bmesh.new() for _ in tones]
    n = 9
    for j in range(n):
        for i in range(n):
            cx = -0.5 + (i + rng.uniform(0.1, 0.9)) / n
            cy = -0.5 + (j + rng.uniform(0.1, 0.9)) / n
            r = rng.uniform(0.06, 0.1)
            tone = rng.randrange(len(tones))
            sub = random.Random(rng.random())
            for (px, py) in wrapped(cx, cy, r * 1.4):
                clod(bms[tone], Vector((px, py, 0.0)), r, 0.35, random.Random(i * 31 + j))
    stone = grey_material("stone", 0.78)
    sbm = bmesh.new()
    for k in range(8):
        cx, cy = rng.uniform(-0.5, 0.5), rng.uniform(-0.5, 0.5)
        r = rng.uniform(0.018, 0.035)
        for (px, py) in wrapped(cx, cy, r * 2.0):
            ret = bmesh.ops.create_icosphere(sbm, subdivisions=2, radius=r)
            for v in ret["verts"]:
                v.co = Vector((v.co.x * 1.5, v.co.y, v.co.z * 0.6)) + Vector((px, py, 0.06))
    link("stones", sbm, stone)
    for k, b in enumerate(bms):
        link("clods%d" % k, b, tones[k])
    sun((0.4, -0.8, -0.45), 4.0)
    return bpy.context.scene


TILES = {"lawn": (lawn, 0.55), "soil_bank": (soil_bank, 0.4)}


def render_tile(name):
    build, low = TILES[name]
    scene = build()
    raw = os.path.abspath(os.path.join(OUT, "_%s_raw.png" % name))
    scene.render.filepath = raw
    scene.render.image_settings.file_format = "PNG"
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(raw)
    px = list(img.pixels)
    lum = [0.2126 * px[i] + 0.7152 * px[i + 1] + 0.0722 * px[i + 2] for i in range(0, len(px), 4)]
    ordered = sorted(lum)
    lo = ordered[int(len(ordered) * 0.02)]
    hi = ordered[int(len(ordered) * 0.98)]
    out = []
    for l in lum:
        t = min(max((l - lo) / max(hi - lo, 1e-4), 0.0), 1.0)
        value = (low + (1.0 - low) * t) ** (1.0 / 2.2)
        out.extend((value, value, value, 1.0))
    tile = bpy.data.images.new(name, SIZE, SIZE)
    tile.pixels = out
    tile.filepath_raw = os.path.abspath(os.path.join(OUT, name + ".png"))
    tile.file_format = "PNG"
    tile.save()
    os.remove(raw)
    print("tile %s: written" % name)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    only = opts["only"].split(",") if "only" in opts else list(TILES)
    os.makedirs(OUT, exist_ok=True)
    for name in only:
        render_tile(name)


main()
