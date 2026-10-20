"""The grove's background in paper (the owner's paper pick on the meadow and the beach, 2026-10-20): the ledges behind the island, the
hills on them and the far violet ranges, built in world space on the grove's layout (`ground.json`: each painted range's crest). The side
trees (`grove_tree.glb`) keep their footing on the nearest ledge, at -9.3 round x -30 and 32, depth -55. Three looks for the owner to pick
from (`--ground-mock=` on the stage demo):
  A  card terraces: each ledge a stack of card sheets stepping down and forward, a green top over teal cut edges; paper disc pines;
  B  pop-up: each ledge a shelf with standing flats in front of it, a scalloped grass fringe over a jagged teal cliff; trees as two cut-out
     pine silhouettes slotted together crosswise, as in a pop-up book;
  C  contour hills: rolling hills of stacked card in place of the ledges, a flat top under each side tree; paper disc pines.
Every look carries the far ranges as standing cut-out flats, three to a range, their crests from the painted ranges they stand in for.
Each piece is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `grass`, `grass_far`, `cliff`, `range_far`, `range_near`, their cut edges `card_<family>`, `leaf_lit`, `leaf_shade`, `bark`.

Run (Blender 5.2):
  blender --background --python art/blender/make_grove_paper.py -- --dir godot/art/baked/<key> --variant A --ground <g.glb>
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
LAYOUT = json.load(open(os.path.join(opts["dir"], "ground.json")))
SIDE_TREES = [(-30.0, -55.0), (32.0, -55.0)]
# The ledges of the shipped ground (make_grove_ground.py A): name, x0, x1, front z, back z, top, top family.
LEDGES = [("T0", -150, 150, -46, -64, -9.3, "grass"), ("T1L", -150, -16, -70, -90, -5.0, "grass_far"),
          ("T1R", 18, 150, -72, -92, -4.5, "grass_far"), ("T2", -170, 170, -98, -122, -1.0, "grass_far")]


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}
AXES = {}          # an object's dome centre or tree axes, for turning its faces outward
KEEP = set()       # objects whose faces are built facing the way they should (two-sided cut-outs)


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
    return PARTS[key]


def face(bm, pts):
    return bm.faces.new([bm.verts.new(p) for p in pts])


def xs_across(x0, x1, step):
    out = [x0]
    while out[-1] < x1:
        out.append(min(out[-1] + step, x1))
    return out


# ---- Trees --------------------------------------------------------------------------------------------------------------------------


def disc_pine(name, x, y, z, sc, rng):
    """A pine of stacked paper discs (the meadow's paper pines): shallow cones, each a little smaller and turned, over a trunk."""
    base = P(x, y, z)
    AXES.setdefault(name, []).append((base.x, base.y))
    lit, under, trunk = part(name, "leaf_lit"), part(name, "leaf_shade"), part(name, "bark")
    for k in range(6):
        t = k / 5.0
        r = 0.62 * (1.0 - 0.75 * t) * sc
        z0 = base.z + (0.35 + 0.32 * k) * sc
        n = 12
        turn = rng.uniform(0, math.tau)
        rim = [Vector((base.x + math.cos(turn + math.tau * i / n) * r * (1.0 if i % 2 else 0.9),
                       base.y + math.sin(turn + math.tau * i / n) * r * (1.0 if i % 2 else 0.9), z0)) for i in range(n)]
        tip = Vector((base.x, base.y, z0 + 0.34 * sc))
        lip = [v - Vector((0, 0, 0.07 * sc)) for v in rim]
        cen = Vector((base.x, base.y, z0 - 0.07 * sc))
        for i in range(n):
            j = (i + 1) % n
            face(lit, (rim[i], rim[j], tip))
            face(under, (lip[i], lip[j], rim[j], rim[i]))
            face(under, (lip[j], lip[i], cen))
    res = bmesh.ops.create_cone(trunk, cap_ends=True, segments=6, radius1=0.1 * sc, radius2=0.07 * sc, depth=0.6 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 0.3 * sc))


def cutout_tree(name, x, y, z, sc, rng):
    """A pop-up tree: two cut-out pine silhouettes slotted together crosswise, each card two-sided, over a cut-out trunk."""
    KEEP.add(name)
    turn = rng.uniform(0, math.pi * 0.5)
    for k, fam in enumerate(("leaf_lit", "leaf_shade")):
        a = turn + k * math.pi * 0.5
        d = Vector((math.cos(a), math.sin(a), 0.0))
        base = P(x, y, z)
        outline = [(-0.12, 0.0), (-0.12, 0.5)]
        tiers = 4
        for i in range(tiers):
            w = 0.95 * (1.0 - 0.2 * i) * (1.0 if i < tiers - 1 else 0.8)
            y0 = 0.5 + 0.62 * i
            outline += [(-w, y0), (-w * 0.35, y0 + 0.5)]
        outline += [(0.0, 0.5 + 0.62 * tiers + 0.25)]
        right = [(-u, v) for (u, v) in reversed(outline[:-1])]
        pts = [base + d * u * sc + Vector((0, 0, v * sc)) for (u, v) in outline + right]
        # Fan from the trunk's middle, both windings so the card shows from either side.
        c = base + Vector((0, 0, 1.4 * sc))
        bm = part(name, fam if len(pts) > 0 else "bark")
        for i in range(len(pts) - 1):
            face(bm, (c, pts[i], pts[i + 1]))
            face(bm, (c, pts[i + 1], pts[i]))
    tb = part(name, "bark")
    base = P(x, y, z)
    for a in (turn, turn + math.pi * 0.5):
        d = Vector((math.cos(a), math.sin(a), 0.0)) * 0.12 * sc
        quad = (base - d, base + d, base + d + Vector((0, 0, 0.55 * sc)), base - d + Vector((0, 0, 0.55 * sc)))
        face(tb, quad)
        face(tb, tuple(reversed(quad)))


def plant_trees(rng, tree):
    """Small trees on the ledges, as the shipped ground has them: clear of the side trees and of the space behind the fight."""
    for name, x0, x1, zf, zb, top, _ in LEDGES:
        n = {"T0": 34, "T2": 30}.get(name, 20)
        for k in range(n):
            x = rng.uniform(x0 + 6, x1 - 6)
            if abs(x + 30) < 8 or abs(x - 32) < 8 or abs(x) < (24 if name != "T2" else 14):
                continue
            tree(name + "_trees", x, top, rng.uniform(zb + 2, zf - 2), rng.uniform(1.8, 3.0), rng)


# ---- A: card terraces ---------------------------------------------------------------------------------------------------------------


def terrace(name, x0, x1, zf, zb, top, fam, rng, sheets=8):
    """A ledge as card: its top sheet, then sheets stepping down and forward, each cut edge standing over the next."""
    xs = xs_across(x0, x1, 3.0)
    ph = [rng.uniform(0, 6) for _ in range(sheets + 1)]

    def edge(k, x):
        return zf + k * 0.8 + 0.6 * math.sin(x * 0.13 + ph[k]) + 0.3 * math.sin(x * 0.37 + ph[k] * 2.0)

    for i in range(len(xs) - 1):
        xa, xb = xs[i], xs[i + 1]
        face(part(name, fam), (P(xa, top, zb), P(xa, top, edge(0, xa)), P(xb, top, edge(0, xb)), P(xb, top, zb)))
        for k in range(sheets):
            y, yn = top - k * 1.4, top - (k + 1) * 1.4
            card = "card_" + (fam if k == 0 else "cliff")
            bottom = yn if k < sheets - 1 else yn - 45.0
            face(part(name, card), (P(xa, y, edge(k, xa)), P(xa, bottom, edge(k, xa)), P(xb, bottom, edge(k, xb)), P(xb, y, edge(k, xb))))
            if k < sheets - 1:
                face(part(name, "cliff"), (P(xa, yn, edge(k, xa)), P(xa, yn, edge(k + 1, xa)), P(xb, yn, edge(k + 1, xb)),
                                           P(xb, yn, edge(k, xb))))


# ---- B: pop-up ----------------------------------------------------------------------------------------------------------------------


def popup_ledge(name, x0, x1, zf, zb, top, fam, rng):
    """A ledge as a pop-up: a shelf for the trees, a scalloped grass fringe standing at its front, and a jagged teal cliff card in front
    of the fringe's foot, with a second, lower cliff card nearer still; each card's top edge shows its thickness."""
    xs = xs_across(x0, x1, 2.0)
    for i in range(len(xs) - 1):
        xa, xb = xs[i], xs[i + 1]
        face(part(name, fam), (P(xa, top, zb), P(xa, top, zf), P(xb, top, zf), P(xb, top, zb)))
    # The fringe: scallops over the shelf's front edge.
    for i in range(len(xs) - 1):
        xa, xb = xs[i], xs[i + 1]
        xm = (xa + xb) * 0.5
        hm = top + 0.45 + 0.15 * math.sin(xm * 0.7)
        z = zf + 0.05
        face(part(name, fam), (P(xa, top - 1.2, z), P(xb, top - 1.2, z), P(xb, top + 0.1, z), P(xm, hm, z), P(xa, top + 0.1, z)))
    # The cliff cards.
    for k, (dz, drop, fam_c) in enumerate(((0.6, 0.9, "cliff"), (1.6, 3.4, "cliff"))):
        z = zf + dz
        pts = []
        x = x0
        while x < x1:
            pts.append((x, top - drop - rng.uniform(0.0, 1.1)))
            x += rng.uniform(1.6, 3.4)
        pts.append((x1, top - drop))
        for i in range(len(pts) - 1):
            (xa, ya), (xb, yb) = pts[i], pts[i + 1]
            face(part(name, fam_c), (P(xa, ya, z), P(xa, top - 50.0, z), P(xb, top - 50.0, z), P(xb, yb, z)))
            face(part(name, "card_" + fam_c), (P(xa, ya, z - 0.3), P(xa, ya, z), P(xb, yb, z), P(xb, yb, z - 0.3)))


# ---- C: contour hills ---------------------------------------------------------------------------------------------------------------


def dome(name, fam, cx, top, cz, rx, rz, rng, step=1.3, floor=-28.0):
    """A hill as stacked card contours (the beach's islands): a flat top at `top`, each sheet below a little wider, lobed and drifting."""
    AXES[name] = [(cx, -cz)]
    lobes = [(n, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for n in (2, 3, 5)]
    drift = (rng.uniform(-1, 1), rng.uniform(-1, 1))
    ring_n = 28
    heights = [top]
    while heights[-1] > floor:
        heights.append(heights[-1] - step * (1.0 + 0.04 * len(heights)))
    rings = []
    for k, h in enumerate(heights):
        grow = 1.0 + 0.22 * k
        ring = []
        for i in range(ring_n):
            a = math.tau * i / ring_n
            w = 1.0 + 0.08 * sum(amp * math.sin(a * n + ph + k * 0.3 * n) for (n, amp, ph) in lobes)
            ring.append((cx + drift[0] * k * 0.3 + math.cos(a) * rx * grow * w, cz + drift[1] * k * 0.2 + math.sin(a) * rz * grow * w))
        rings.append(ring)
    face(part(name, fam), [P(x, heights[0], z) for (x, z) in rings[0]])
    for k in range(len(heights) - 1):
        y, yn = heights[k], heights[k + 1]
        for i in range(ring_n):
            j = (i + 1) % ring_n
            a, b = rings[k][i], rings[k][j]
            c, d = rings[k + 1][j], rings[k + 1][i]
            face(part(name, "card_" + (fam if k == 0 else "cliff")), (P(a[0], y, a[1]), P(b[0], y, b[1]), P(b[0], yn, b[1]), P(a[0], yn, a[1])))
            face(part(name, fam if k < 2 else "cliff"), (P(a[0], yn, a[1]), P(b[0], yn, b[1]), P(c[0], yn, c[1]), P(d[0], yn, d[1])))


def contour_hills(rng):
    k = 0
    # A flat-topped hill under each side tree.
    for (x, z) in SIDE_TREES:
        dome("H%d" % k, "grass", x, -9.3 if x < 0 else -9.0, z, 7.0, 5.0, rng)
        k += 1
    for name, x0, x1, zf, zb, top, fam in LEDGES:
        x = x0 + rng.uniform(0, 8)
        while x < x1:
            rx = rng.uniform(9.0, 15.0)
            if not any(abs(x - tx) < rx + 4 for (tx, _) in SIDE_TREES) or name != "T0":
                dome("H%d" % k, fam, x, top + rng.uniform(-1.5, 1.0), (zf + zb) * 0.5 + rng.uniform(-3, 3), rx, rng.uniform(6.0, 9.0), rng,
                     step=1.3 if name == "T0" else 2.2)
                k += 1
            x += rx * rng.uniform(2.0, 2.6)


# ---- Far land: the ranges as cut-out flats ------------------------------------------------------------------------------------------


def range_flats():
    for r_i, rng_spec in enumerate(LAYOUT.get("ranges", [])):
        fam = "range_far" if r_i == 0 else "range_near"
        raw = rng_spec["top"]
        # (Smoothed: the highest vertex in each bin follows every fold's tip, and the crest came out a sawtooth.)
        top = []
        peak = [max(raw[j][1] for j in range(max(0, i - 1), min(len(raw), i + 2))) for i in range(len(raw))]
        for i in range(len(raw)):
            near = peak[max(0, i - 2): i + 3]
            top.append((raw[i][0], sum(near) / len(near)))
        z = rng_spec["z"]
        base = min(t[1] for t in top) - 10.0
        # (Close to the painted range's own height, a step taller behind and lower in front: much taller walled off the sky.)
        for k, (dz, scale) in enumerate(((-8.0, 1.08), (0.0, 1.0), (8.0, 0.9))):
            name = "range%d_%d" % (r_i, k)
            zz = z + dz
            pts = [(x, base + (y - base) * scale) for (x, y) in top]
            for i in range(len(pts) - 1):
                (xa, ya), (xb, yb) = pts[i], pts[i + 1]
                face(part(name, fam), (P(xa, ya, zz), P(xa, base - 30.0, zz), P(xb, base - 30.0, zz), P(xb, yb, zz)))
                face(part(name, "card_" + fam), (P(xa, ya, zz - 0.8), P(xa, ya, zz), P(xb, yb, zz), P(xb, yb, zz - 0.8)))


# ---- Build --------------------------------------------------------------------------------------------------------------------------


def build_ground():
    rng = random.Random(29)
    if VARIANT == "A":
        for name, x0, x1, zf, zb, top, fam in LEDGES:
            terrace(name, x0, x1, zf, zb, top, fam, rng)
        plant_trees(rng, disc_pine)
    elif VARIANT == "B":
        for name, x0, x1, zf, zb, top, fam in LEDGES:
            popup_ledge(name, x0, x1, zf, zb, top, fam, rng)
        plant_trees(rng, cutout_tree)
    else:
        contour_hills(rng)
        for name, x0, x1, zf, zb, top, _ in LEDGES:
            for k in range({"T0": 14, "T2": 14}.get(name, 9)):
                x = rng.uniform(x0 + 6, x1 - 6)
                if abs(x + 30) < 9 or abs(x - 32) < 9 or abs(x) < (24 if name != "T2" else 14):
                    continue
                disc_pine(name + "_trees", x, top - 0.3, rng.uniform(zb + 3, zf - 3), rng.uniform(1.8, 3.0), rng)
    range_flats()


def make_objects():
    for (name, mat), bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        if name not in KEEP:
            axes = AXES.get(name)
            for f in bm.faces:
                c = f.calc_center_median()
                if axes:
                    ax = min(axes, key=lambda a: (c.x - a[0]) ** 2 + (c.y - a[1]) ** 2)
                    away = Vector((c.x - ax[0], c.y - ax[1], 0.0))
                    away = (away.normalized() if away.length > 1e-4 else away) + Vector((0.0, 0.0, 0.3))
                else:
                    # (Ledges and flats face the camera and the sky.)
                    away = Vector((0.0, -0.7, 1.0))
                if f.normal.dot(away) < 0.0:
                    f.normal_flip()
        me = bpy.data.meshes.new(name + "_" + mat)
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new(name + "_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def bake_occlusion(distance):
    scene = bpy.context.scene
    parts = [o for o in scene.objects if o.type == "MESH"]
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
    scene.cycles.samples = 96
    if scene.world is None:
        scene.world = bpy.data.worlds.new("Bake")
    scene.world.light_settings.distance = distance
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        a = o.data.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
        o.data.color_attributes.active_color = a
        o.data.color_attributes.render_color_index = o.data.color_attributes.active_color_index
        o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.bake(type="AO", target="VERTEX_COLORS")
    for o in parts:
        for c in o.data.color_attributes["Col"].data:
            ao = 0.45 + 0.55 * c.color[0]
            c.color = (ao, ao, ao, 1.0)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_ground()
    make_objects()
    bake_occlusion(2.0)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    path = os.path.abspath(opts["ground"])
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("grove paper %s: %d triangles -> %s" % (VARIANT, tris, path))


main()
