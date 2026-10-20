"""The meadow's stage front as one modelled piece (the stage refinement, after the grove pilot; docs/art/stages/BAKE_PILOT.md).

Built on the stage's own collision, read from the bake manifest (the block's rectangle and the front plane), following the block's
batter (it narrows 0.13 a unit down, `meadow.cfg` `taper`), starting under the lawn's slab and ending at the block's foot, where the
meadow's hill takes over. Three looks for the owner to pick from (`--front-mock=` on the stage demo):
  A  a folded rock cliff: the face pleated into ridges and creases running down, each plane turned well toward or away from the light,
     in the owner's light moss, as the meadow's mountains are folded;
  B  a terrace wall of logs and stones: courses of round logs with their cut ends showing, on courses of bevelled stones, moss in the
     joints, the farm's own build (fence, bales, barrel);
  C  an earth bank: folded soil with bevelled stones set in it, the hillside cut back.
All three: a grass overhang along the top. Families `ledge`, `stone`, `log`, `log_end`, `soil`, `grass`, `moss`, painted from the
stage's palette in the game; each piece bakes its occlusion into linear vertex colours with the built meadow round it.

Run (Blender 5.2):
  blender --background --python art/blender/make_meadow_front.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold_kit import folded_dome  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
BLOCK = [r for r in manifest["rects"] if r[4] < 0.5][0]
LEFT, RIGHT, TOP, BOTTOM = BLOCK[0], BLOCK[1], BLOCK[2], BLOCK[3]
FZ = float(manifest["front_z"])
TAPER = 0.13
Y_TOP = TOP - 0.55
Y_BOT = BOTTOM


def P(x, y, z):
    return Vector((x, -z, y))


def inset(y):
    return min(max(TOP - y, 0.0) * TAPER, (RIGHT - LEFT) / 3.0)


def edges(y):
    i = inset(y)
    return LEFT + i, RIGHT - i


PARTS = {}


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
        PARTS[key].verts.layers.float.new("tone")
    return PARTS[key]


def stone(bm, c, size, rng, bevel=0.18):
    res = bmesh.ops.create_cube(bm, size=2.0)
    verts = res["verts"]
    rot = Matrix.Rotation(rng.uniform(-0.15, 0.15), 3, "Y")
    for v in verts:
        v.co = rot @ Vector((v.co.x * size.x * rng.uniform(0.9, 1.1), v.co.y * size.y, v.co.z * size.z * rng.uniform(0.9, 1.1))) + c
    edges_ = list({e for v in verts for e in v.link_edges})
    out = bmesh.ops.bevel(bm, geom=edges_, offset=min(size.x, size.z) * bevel, segments=1, affect="EDGES")
    # (Each stone its own tone, multiplied into its baked occlusion: one grey for all read as pasted cubes.)
    layer = bm.verts.layers.float.get("tone")
    tone = random.Random(round(c.x * 10.0) * 7919 + round(c.z * 10.0)).uniform(0.7, 1.05)
    for v in out["verts"]:
        if layer is not None and v.is_valid:
            v[layer] = tone


def folded_face(bm, rng, depth, width, step=0.5):
    """The face pleated into ridges running down: columns of their own widths, every other one standing `depth` proud, leaning a little,
    so each plane turns well toward or away from the light (a shallow fold lit both its sides alike and vanished, pass 2)."""
    xs = []
    lo, hi = edges(Y_TOP)
    x = lo
    while x < hi:
        xs.append(x)
        x += width * rng.uniform(0.7, 1.3)
    xs.append(hi)
    rows = int(math.ceil((Y_TOP - Y_BOT) / step))
    grid = []
    for j in range(rows + 1):
        y = Y_TOP + (Y_BOT - Y_TOP) * j / rows
        lo_y, hi_y = edges(y)
        lo_t, hi_t = edges(Y_TOP)
        row = []
        for i, x0 in enumerate(xs):
            x = lo_y + (x0 - lo_t) / (hi_t - lo_t) * (hi_y - lo_y)
            ridge = (i % 2 == 1) and 0 < i < len(xs) - 1
            lean = 0.25 * math.sin(i * 1.7) * (TOP - y) / 8.0
            out = (depth * rng.uniform(0.8, 1.15) if ridge else 0.0) * min(1.0, (Y_TOP - y + 0.3) / 0.8)
            row.append(bm.verts.new(P(x + lean, y, FZ + 0.05 + out)))
        grid.append(row)
    for j in range(rows):
        for i in range(len(xs) - 1):
            a, b, c, d = grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]
            bm.faces.new((a, d, c, b))


def plain_face(bm, rng, bulge=0.0, step=0.5, cols=40):
    rows = int(math.ceil((Y_TOP - Y_BOT) / step))
    grid = []
    for j in range(rows + 1):
        y = Y_TOP + (Y_BOT - Y_TOP) * j / rows
        lo, hi = edges(y)
        row = []
        for i in range(cols + 1):
            x = lo + (hi - lo) * i / cols
            out = bulge * (0.5 + 0.5 * math.sin(x * 0.9 + y * 1.3)) * (1 if 0 < i < cols and 0 < j else 0)
            row.append(bm.verts.new(P(x, y, FZ + 0.03 + out)))
        grid.append(row)
    for j in range(rows):
        for i in range(cols):
            a, b, c, d = grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]
            bm.faces.new((a, d, c, b))


def log(bm_side, bm_end, x0, x1, y, r, rng):
    """A round log lying along the face from `x0` to `x1` at height `y`, its cut ends showing."""
    sides = 10
    zc = FZ + r * 0.75
    rings = []
    for x in (x0, x1):
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            rr = r * rng.uniform(0.95, 1.05)
            ring.append(Vector((x, -(zc + math.cos(a) * rr), y + math.sin(a) * rr)))
        rings.append(ring)
    sv = [[bm_side.verts.new(p) for p in ring] for ring in rings]
    for i in range(sides):
        j = (i + 1) % sides
        f = bm_side.faces.new((sv[0][i], sv[0][j], sv[1][j], sv[1][i]))
        f.normal_update()
        mid = Vector(((x0 + x1) * 0.5, -zc, y))
        if f.normal.dot(f.calc_center_median() - Vector((f.calc_center_median().x, mid.y, mid.z))) < 0:
            f.normal_flip()
    for k, ring in enumerate(rings):
        ev = [bm_end.verts.new(p) for p in ring]
        f = bm_end.faces.new(ev)
        f.normal_update()
        want = Vector((-1, 0, 0)) if k == 0 else Vector((1, 0, 0))
        if f.normal.dot(want) < 0:
            f.normal_flip()


def grass_overhang(rng):
    lit, shade = part("Grass", "grass"), part("Grass", "moss")
    lo, hi = edges(Y_TOP)
    x = lo + 0.3
    while x < hi - 0.2:
        r = rng.uniform(0.35, 0.6)
        # (Hanging over the face's top, in front of the lawn's slab: tucked under it, the overhang showed as a thin dark strip.)
        c = P(x, Y_TOP - 0.35 + rng.uniform(-0.1, 0.05), FZ + 0.55)
        n_l, n_s = len(lit.verts), len(shade.verts)
        folded_dome(lit, shade, tuple(c), r, 6, rng)
        # (Upright and squashed, not turned over: turned over, their undersides faced the sky inside the lawn's slab and baked a dark
        # purple band along the edge.)
        flip = Matrix.Rotation(rng.uniform(-0.25, 0.25), 3, "X") @ Matrix.Diagonal((1.3, 0.9, rng.uniform(0.45, 0.7)))
        for bm, n in ((lit, n_l), (shade, n_s)):
            bm.verts.ensure_lookup_table()
            for v in bm.verts[n:]:
                v.co = flip @ (v.co - c) + c
        x += r * rng.uniform(0.9, 1.3)


def build():
    rng = random.Random(17)
    if VARIANT == "A":
        folded_face(part("Cliff", "ledge"), rng, depth=0.95, width=1.25)
        # A few boulders where the cliff meets the hill.
        bm = part("Boulders", "ledge")
        lo, hi = edges(Y_BOT + 0.6)
        for k in range(7):
            x = rng.uniform(lo + 0.6, hi - 0.6)
            s = rng.uniform(0.4, 0.75)
            stone(bm, P(x, Y_BOT + s * 0.5, FZ + 1.0 + s * 0.3), Vector((s * 1.3, s * 0.8, s)), rng, bevel=0.3)
    elif VARIANT == "B":
        plain_face(part("Backing", "soil"), rng)
        side, end = part("Logs", "log"), part("Logs", "log_end")
        y = Y_TOP - 0.45
        course = 0
        # (Two courses of logs over courses of stone: logs to the foot read as a cabin wall.)
        while y > TOP - 2.2:
            r = 0.42
            lo, hi = edges(y)
            x = lo - (rng.uniform(0.2, 1.6) if course % 2 else 0.0)
            while x < hi:
                length = rng.uniform(3.0, 5.0)
                x1 = min(x + length, hi + 0.1)
                log(side, end, max(x, lo - 0.1), x1, y + rng.uniform(-0.04, 0.04), r * rng.uniform(0.92, 1.08), rng)
                x = x1 + 0.06
            y -= r * 1.9
            course += 1
        bm = part("Stones", "stone")
        while y > Y_BOT + 0.3:
            lo, hi = edges(y)
            x = lo + rng.uniform(0.0, 0.5)
            h = rng.uniform(0.38, 0.55)
            while x < hi - 0.2:
                w = rng.uniform(0.6, 1.2)
                stone(bm, P(min(x + w * 0.5, hi - w * 0.4), y - h * 0.5, FZ + 0.32), Vector((w * 0.48, 0.32, h * 0.48)), rng)
                x += w + 0.05
            y -= h + 0.06
    else:
        plain_face(part("Bank", "soil"), rng, bulge=0.5, step=0.4, cols=60)
        bm = part("Stones", "stone")
        for k in range(30):
            y = rng.uniform(Y_BOT + 0.4, Y_TOP - 0.9)
            lo, hi = edges(y)
            x = rng.uniform(lo + 0.4, hi - 0.4)
            s = rng.uniform(0.25, 0.75)
            # (Set into the bank, a third showing: standing proud they read as cubes pasted on.)
            stone(bm, P(x, y, FZ + 0.08), Vector((s * 1.3, s * 0.55, s * 0.9)), rng, bevel=0.35)
    grass_overhang(rng)


def clamp(bm, margin=0.9):
    for v in bm.verts:
        y = min(max(v.co.z, BOTTOM), TOP)
        lo, hi = edges(y)
        v.co.x = min(max(v.co.x, lo - margin), hi + margin)
        v.co.z = min(v.co.z, TOP - 0.3)


def make_objects():
    for (name, mat), bm in PARTS.items():
        clamp(bm)
        me = bpy.data.meshes.new(name + "_" + mat)
        bm.normal_update()
        bm.to_mesh(me)
        bm.free()
        for p in me.polygons:
            p.use_smooth = False
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new(name + "_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def bake_occlusion():
    mock = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    gone = {e["name"] for e in manifest["meshes"] if e["path"].split("/")[-1].startswith(("faceted_ledge", "block", "lip"))}
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.join(opts["dir"], "scene_in.glb"), merge_vertices=True)
    for o in [o for o in bpy.context.scene.objects if o not in before]:
        if o.name in gone:
            bpy.data.objects.remove(o, do_unlink=True)
    context = [o for o in bpy.context.scene.objects if o not in before]
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    try:
        prefs = bpy.context.preferences.addons["cycles"].preferences
        prefs.compute_device_type = "OPTIX"
        prefs.get_devices()
        for d in prefs.devices:
            d.use = d.type == "OPTIX"
        scene.cycles.device = "GPU"
    except (KeyError, TypeError):
        pass
    scene.cycles.samples = 128
    if scene.world is None:
        scene.world = bpy.data.worlds.new("Bake")
    scene.world.light_settings.distance = 2.0
    bpy.ops.object.select_all(action="DESELECT")
    for o in mock:
        a = o.data.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
        o.data.color_attributes.active_color = a
        o.data.color_attributes.render_color_index = o.data.color_attributes.active_color_index
        o.select_set(True)
    bpy.context.view_layer.objects.active = mock[0]
    bpy.ops.object.bake(type="AO", target="VERTEX_COLORS")
    for o in mock:
        tone = o.data.attributes.get("tone")
        if tone is None:
            continue
        col = o.data.color_attributes["Col"].data
        for v in o.data.vertices:
            k = tone.data[v.index].value or 1.0
            c = col[v.index].color
            col[v.index].color = (c[0] * k, c[1] * k, c[2] * k, 1.0)
    for o in context:
        bpy.data.objects.remove(o, do_unlink=True)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build()
    make_objects()
    bake_occlusion()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("meadow front %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
