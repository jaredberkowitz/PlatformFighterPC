"""The beach's background land as paper (the owner's paper-layer pick on the meadow, 2026-10-20: "this type of paper aesthetic looks the
best"): the harbour headland's hills behind the town and the three islands' mounds, modelled on the layout the game exports
(`ground.json`: each island's sand and green mounds and each headland hill, as the half-axes of their domes), so the houses, palms,
rocks and huts on them keep their feet. The quay, the town, the palms, the lighthouse and the stilt hut stay code-built. Three looks
for the owner to pick from (`--ground-mock=` on the stage demo):
  A  card contours: each dome as a stack of thin card sheets, a contour every step, the cut edges catching the light (the meadow's look);
  B  foam board: fewer, thicker layers with a pale bevelled lip, softer and chunkier;
  C  pop-up: the far hills as standing cut-out flats one behind another, like a pop-up book; the near hills and islands as card contours.
Each dome is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `sand`, `isle_green`, `head_near`, `head_far` and each one's cut edge `card_<family>`.

Run (Blender 5.2):
  blender --background --python art/blender/make_beach_ground.py -- --dir godot/art/baked/<key> --variant A --ground <g.glb>
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold_kit import folded_dome  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
# Rounder islands (fidelity review 14 item 6: the islands read as stacked green slabs, the near hills as one pale mass): `--isles A|B`.
# A: the contours finer (the islands' every 0.32, not 0.7; the near hills' every 0.7, not 1.5), and each sand mound wider, so a rim of
# sand shows round the green at the waterline. B: the islands' green and the near hills as folded paper domes (the grove's and the
# meadow's fold, fold_kit.folded_dome), each fold lit on one side, on A's sand. The far hills stay pop-up flats.
ISLES = opts.get("isles", "").upper()
LAYOUT = json.load(open(os.path.join(opts["dir"], "ground.json")))
WATER = -10.6          # a little under the sea's surface, so every dome's foot is in the water
RING = 56


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}
CENTRES = {}
FOLDED = set()


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
    return PARTS[key]


def face(bm, pts):
    return bm.faces.new([bm.verts.new(p) for p in pts])


def contour(c, r, h, k, shape, wobble):
    """The dome's outline at height `h` (an ellipse of its half-axes), as a ring of (x, z): lobed by the dome's own `shape` (a few
    sines of its own phases) and each layer `k` drifting off-centre a little, as the contours of a real hill do; concentric ellipses
    read as a wedding cake."""
    cx, cy, cz = c
    rx, ry, rz = r
    t = 1.0 - ((h - cy) / ry) ** 2
    s = math.sqrt(max(t, 0.0))
    lobes, drift = shape
    ox = drift[0] * k * rx * 0.035
    oz = drift[1] * k * rz * 0.035
    out = []
    for i in range(RING):
        a = math.tau * i / RING
        w = 1.0 + wobble * sum(amp * math.sin(a * n + ph + k * 0.35 * n) for (n, amp, ph) in lobes)
        out.append((cx + ox + math.cos(a) * rx * s * w, cz + oz + math.sin(a) * rz * s * w))
    return out


def dome_layers(name, fam, c, r, step, rng, wobble=0.07, lip=False):
    """A dome as stacked sheets: each flat at its contour's height, its cut edge standing down to the next contour; the bottom one carried
    into the water."""
    cx, cy, cz = c
    top = cy + r[1]
    heights = []
    h = top - step * 0.6
    while h > WATER:
        heights.append(h)
        h -= step
    heights.append(WATER - 0.4)
    shape = ([(n, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for n in (2, 3, 5)], (rng.uniform(-1, 1), rng.uniform(-1, 1)))
    CENTRES[name] = (cx, cz)
    rings = [contour(c, r, max(h, cy), k, shape, wobble) for k, h in enumerate(heights)]
    # The top sheet: a flat disc.
    face(part(name, fam), [P(x, heights[0], z) for (x, z) in rings[0]])
    for k in range(len(heights) - 1):
        y, y_next = heights[k], heights[k + 1]
        inner, outer = rings[k], rings[k + 1]
        for i in range(RING):
            j = (i + 1) % RING
            # The cut edge of sheet k, down to sheet k + 1.
            if lip:
                # (Foam board: a pale bevelled lip along the top of each edge.)
                mid = y - min(0.35, (y - y_next) * 0.3)
                face(part(name, "card_" + fam), (P(inner[i][0], y, inner[i][1]), P(inner[j][0], y, inner[j][1]),
                                                 P(inner[j][0], mid, inner[j][1]), P(inner[i][0], mid, inner[i][1])))
                face(part(name, fam), (P(inner[i][0], mid, inner[i][1]), P(inner[j][0], mid, inner[j][1]),
                                       P(inner[j][0], y_next, inner[j][1]), P(inner[i][0], y_next, inner[i][1])))
            else:
                face(part(name, "card_" + fam), (P(inner[i][0], y, inner[i][1]), P(inner[j][0], y, inner[j][1]),
                                                 P(inner[j][0], y_next, inner[j][1]), P(inner[i][0], y_next, inner[i][1])))
            # Sheet k + 1's top, from under sheet k's edge out to its own.
            face(part(name, fam), (P(inner[i][0], y_next, inner[i][1]), P(inner[j][0], y_next, inner[j][1]),
                                   P(outer[j][0], y_next, outer[j][1]), P(outer[i][0], y_next, outer[i][1])))


def flats(name, fam, c, r, rng, count=3):
    """A hill as standing cut-out flats, one behind another, each a sheet with the dome's silhouette at its depth and a thickness."""
    cx, cy, cz = c
    rx, ry, rz = r
    CENTRES[name] = None
    for k in range(count):
        t = -0.6 + 1.2 * k / max(count - 1, 1)          # from the front of the dome to its back
        z = cz + rz * (0.55 - 0.55 * k / max(count - 1, 1))
        s = math.sqrt(max(1.0 - t * t * 0.5, 0.2))
        top_pts = []
        n = 28
        for i in range(n + 1):
            u = -1.0 + 2.0 * i / n
            wob = 1.0 + 0.04 * math.sin(i * 1.3 + k * 2.1)
            yy = cy + ry * s * math.sqrt(max(1.0 - u * u, 0.0)) * wob
            top_pts.append((cx + u * rx * s * (1.0 - 0.12 * k), max(yy, WATER)))
        th = 0.6
        for i in range(n):
            (xa, ya), (xb, yb) = top_pts[i], top_pts[i + 1]
            face(part(name, fam), (P(xa, ya, z), P(xa, WATER - 0.4, z), P(xb, WATER - 0.4, z), P(xb, yb, z)))
            face(part(name, "card_" + fam), (P(xa, ya, z - th), P(xa, ya, z), P(xb, yb, z), P(xb, yb, z - th)))


def folded_hill(name, fam, c, r, rng):
    """`--isles B`: a dome of half-axes `r` (Godot's x, y, z) on the centre `c` as a folded paper dome, its pleats lit on one side; the
    folds into `fam`, the skirt and underside into its cut edge's family (a shade under the tops, as the contours' edges are)."""
    cx, cy, cz = c
    rx, ry, rz = r
    lit, shade = part(name, fam), part(name, "card_" + fam)
    counts = (len(lit.verts), len(shade.verts))
    rr = rx / 1.08
    base = P(cx, cy, cz)
    folded_dome(lit, shade, (base.x, base.y, base.z), rr, 7, rng, rows=[(-0.12, 0.3), (0.32, 0.2), (0.68, 0.1)])
    # (Out of the round dome the kit folds into the hill's own ellipsoid: deep as it is, as tall as it is.)
    for bm, n in ((lit, counts[0]), (shade, counts[1])):
        bm.verts.ensure_lookup_table()
        for v in bm.verts[n:]:
            v.co.y = base.y + (v.co.y - base.y) * (rz / rr)
            v.co.z = base.z + (v.co.z - base.z) * (ry / (0.8 * rr))
    FOLDED.add(name)


def build_ground():
    rng = random.Random(73)
    thick = VARIANT == "B"
    for k, i in enumerate(LAYOUT["islands"]):
        c = (i[0], i[1], i[2])
        if ISLES:
            dome_layers("isle%d_sand" % k, "sand", c, (i[3] * 1.15, i[4], i[5] * 1.15), 0.32, rng)
            if ISLES == "B":
                folded_hill("isle%d_green" % k, "isle_green", (i[0], i[1] + 2.0, i[2]), (i[6], i[7], i[8]), rng)
            else:
                dome_layers("isle%d_green" % k, "isle_green", (i[0], i[1] + 2.0, i[2]), (i[6], i[7], i[8]), 0.32, rng)
            continue
        dome_layers("isle%d_sand" % k, "sand", c, (i[3], i[4], i[5]), 1.4 if thick else 0.7, rng, lip=thick)
        dome_layers("isle%d_green" % k, "isle_green", (i[0], i[1] + 2.0, i[2]), (i[6], i[7], i[8]), 1.4 if thick else 0.7, rng,
                    lip=thick)
    for k, h in enumerate(LAYOUT["headland"]):
        row, c, r = h[0], (h[1], h[2], h[3]), (h[4], h[5], h[6])
        fam = "head_near" if row == "near" else "head_far"
        if VARIANT == "C" and row == "far":
            flats("head%d" % k, fam, c, r, rng)
        elif ISLES == "B" and row == "near":
            folded_hill("head%d" % k, fam, c, r, rng)
        elif ISLES == "A" and row == "near":
            dome_layers("head%d" % k, fam, c, r, 0.7, rng)
        else:
            dome_layers("head%d" % k, fam, c, r, (3.0 if thick else 1.5) * (1.6 if row == "far" else 1.0), rng, lip=thick)


def make_objects():
    for (name, mat), bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        centre = CENTRES.get(name)
        if name in FOLDED:
            # (The folded domes face their own way, fold_kit.folded_dome.)
            bm.to_mesh(me := bpy.data.meshes.new(name + "_" + mat))
            bm.free()
            m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
            me.materials.append(m)
            bpy.context.scene.collection.objects.link(bpy.data.objects.new(name + "_" + mat, me))
            continue
        for f in bm.faces:
            c = f.calc_center_median()
            if centre is None:
                # (A flat faces the camera; its top edge faces up.)
                away = Vector((0.0, -1.0, 0.4))
            else:
                # (A sheet's top faces up, its cut edge out from the dome's middle.)
                away = Vector((c.x - centre[0], c.y + centre[1], 0.0)).normalized() + Vector((0.0, 0.0, 0.3))
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
    # (Softened: the land keeps most of its own light, the steps a soft shade.)
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
    print("beach ground %s: %d triangles -> %s" % (VARIANT, tris, path))


main()
