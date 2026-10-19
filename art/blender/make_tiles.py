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
    # (Lighter joints, round 26: the clods' outlines read a touch dark and even.)
    link("back", bm, grey_material("back", 0.42))
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


def slab(bm, centre, size, tilt=0.0, turn=0.0):
    """A thin box at `centre`, `size` across, tipped by `tilt` about x and turned by `turn` about z (radians)."""
    ret = bmesh.ops.create_cube(bm, size=1.0)
    for v in ret["verts"]:
        x, y, z = v.co.x * size[0], v.co.y * size[1], v.co.z * size[2]
        y, z = y * math.cos(tilt) - z * math.sin(tilt), y * math.sin(tilt) + z * math.cos(tilt)
        x, y = x * math.cos(turn) - y * math.sin(turn), x * math.sin(turn) + y * math.cos(turn)
        v.co = Vector((x, y, z)) + Vector(centre)


def roof():
    """A flat roof's covering (the city's fighting surface; the tile is 4 units of roof): two rows of roofing sheet a tile, each a
    little tipped so its lower edge laps over the row in front and catches the light, each row its own tone; a few patched repairs laid
    over them at their own angles; grit scattered over everything; and a few soft puddle stains where water stood. Lit as the lawn is."""
    rng = random.Random(37)
    reset()
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=1.5)
    link("deck", bm, grey_material("deck", 0.4))
    rows = 2
    for j in range(rows + 2):
        y = -0.5 + (j - 0.5) / rows
        mat = grey_material("sheet%d" % j, (0.66, 0.6, 0.7, 0.63)[j % 4])
        b = bmesh.new()
        slab(b, (0.0, y, 0.004), (1.6, 1.0 / rows + 0.03, 0.006), tilt=math.radians(-1.2))
        link("sheet%d" % j, b, mat)
    patch = [grey_material("patch_a", 0.8), grey_material("patch_b", 0.86)]
    for k in range(3):
        cx, cy = rng.uniform(-0.5, 0.5), rng.uniform(-0.5, 0.5)
        # (Larger, so each repair reads as a shape at match distance, the gauntlet's round 27.)
        w, h = rng.uniform(0.16, 0.3), rng.uniform(0.1, 0.2)
        turn = rng.uniform(-0.3, 0.3)
        m = patch[k % 2]
        for (px, py) in wrapped(cx, cy, 0.15):
            b = bmesh.new()
            slab(b, (px, py, 0.012), (w, h, 0.003), turn=turn)
            link("patch%d" % k, b, m)
    stain = grey_material("stain", 0.45)
    for k in range(4):
        cx, cy = rng.uniform(-0.5, 0.5), rng.uniform(-0.5, 0.5)
        r = rng.uniform(0.05, 0.1)
        ph = [rng.uniform(0, math.tau) for _ in range(3)]
        for (px, py) in wrapped(cx, cy, r * 1.4):
            b = bmesh.new()
            ring = []
            for i in range(24):
                a = math.tau * i / 24
                rr = r * (1.0 + 0.25 * math.sin(a * 3 + ph[0]) + 0.12 * math.sin(a * 5 + ph[1]))
                ring.append(b.verts.new((px + math.cos(a) * rr * 1.4, py + math.sin(a) * rr, 0.0105)))
            b.faces.new(ring)
            link("stain%d" % k, b, stain)
    grit = [grey_material("grit_a", 0.9), grey_material("grit_b", 0.25)]
    gb = [bmesh.new(), bmesh.new()]
    for k in range(1600):
        x, y = rng.uniform(-0.5, 0.5), rng.uniform(-0.5, 0.5)
        r = rng.uniform(0.0015, 0.0035)
        for (px, py) in wrapped(x, y, r):
            ret = bmesh.ops.create_icosphere(gb[k % 2], subdivisions=1, radius=r)
            for v in ret["verts"]:
                v.co += Vector((px, py, 0.012))
    for k in range(2):
        link("grit%d" % k, gb[k], grit[k])
    sun((0.45, 0.75, -0.55), 4.0)
    return bpy.context.scene



def furrows():
    """Ploughed farmland for the meadow's valley fields: eight ridges a tile running across it, each a rounded mound of earth with a
    row of small crop tufts along its crown, darker troughs between them; laid by the fields' own coordinates in the game so the rows
    follow (and converge with) the fields. Lit from image-up, as the land faces the sky."""
    rng = random.Random(53)
    reset()
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=1.5)
    link("trough", bm, grey_material("trough", 0.35))
    ridge = grey_material("ridge", 0.62)
    crop = [grey_material("crop_a", 0.82), grey_material("crop_b", 0.95)]
    rows = 8
    rb = bmesh.new()
    cb = [bmesh.new(), bmesh.new()]
    for j in range(rows + 2):
        y = -0.5 + (j - 0.5) / rows
        # The ridge: a long squashed tube along x, a hair wavy.
        ret = bmesh.ops.create_cone(rb, cap_ends=False, cap_tris=False, segments=12, radius1=0.045, radius2=0.045, depth=1.8)
        for v in ret["verts"]:
            x, yy, z = v.co.z, v.co.x, v.co.y
            v.co = Vector((x, y + yy + 0.004 * math.sin(x * 9.0 + j), z * 0.45))
        # Crop tufts along its crown.
        x = -0.6
        while x < 0.6:
            for (px, py) in wrapped(x, y, 0.03) if abs(y) < 0.5 else [(x, y)]:
                for b in range(rng.randint(5, 9)):
                    a = rng.uniform(0, math.tau)
                    out = Vector((math.cos(a), math.sin(a), 0.0))
                    blade(cb[j % 2], Vector((px, py, 0.016)) + out * 0.004, out, rng.uniform(0.018, 0.03), rng.uniform(0.003, 0.005),
                          rng.uniform(0.4, 0.9), rng.uniform(-1, 1))
            x += rng.uniform(0.022, 0.036)
    rb.normal_update()
    link("ridges", rb, ridge)
    for k in range(2):
        link("crop%d" % k, cb[k], crop[k])
    sun((0.3, -0.85, -0.6), 4.0)
    return bpy.context.scene


# (The lawn's range narrower and brighter, the gauntlet's round 26: at 0.55 the meadow's lawn read as a dark speckled felt.)
TILES = {"lawn": (lawn, 0.68), "soil_bank": (soil_bank, 0.4), "roof_render": (roof, 0.6), "furrows": (furrows, 0.62)}


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
