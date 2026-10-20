"""The canyon's background in paper (the owner's paper picks on the meadow, the beach and the grove, 2026-10-20): the buttes, the rock arch
and the dunes, built in world space on the canyon's layout (`ground.json`: each butte's and the arch's bounds, each dune range's crest).
The desert floor, the cacti, the telegraph poles and the sun stay code-built. Three looks for the owner to pick from (`--ground-mock=` on
the stage demo):
  A  card stacks: every butte and the arch a stack of thick card plates, each its own lobed outline a little off the one below, in the
     canyon's strata colours band by band, on a low skirt of wide plates for the rubble;
  B  pop-up: every butte a few standing cut-outs one behind another, stepped like the strata; the arch a cut-out with its opening cut;
  C  both: the near buttes and the arch as card stacks, the far row as pop-up cut-outs.
Every look carries the dunes as standing cut-out flats along the crests of the dune ranges they stand in for.
Each piece is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `rock_a`, `rock_b`, `rock_c` (the strata), `dune_far`, `dune_near`, and their cut edges `card_<family>`.

Run (Blender 5.2):
  blender --background --python art/blender/make_canyon_paper.py -- --dir godot/art/baked/<key> --variant A --ground <g.glb>
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
FLOOR = -16.0
STRATA = ["rock_a", "rock_b", "rock_a", "rock_c"]


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}
AXES = {}


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
    return PARTS[key]


def face(bm, pts):
    return bm.faces.new([bm.verts.new(p) for p in pts])


# ---- Card stacks --------------------------------------------------------------------------------------------------------------------


def plate(name, fam, cx, cz, rx, rz, y0, y1, lobes, k, n=20, hole=None):
    """One thick card plate: a lobed outline (its own wobble by `k`) extruded from y0 to y1, its top and its cut edge."""
    ring = []
    for i in range(n):
        a = math.tau * i / n
        w = 1.0 + 0.09 * sum(amp * math.sin(a * m + ph + k * 0.9) for (m, amp, ph) in lobes)
        ring.append((cx + math.cos(a) * rx * w, cz + math.sin(a) * rz * w))
    # (Its top facing up and its underside down: the ring's winding turns over between the game's axes and Blender's.)
    for y, up in ((y1, 1.0), (y0, -1.0)):
        f = face(part(name, fam), [P(x, y, z) for (x, z) in ring])
        f.normal_update()
        if f.normal.z * up < 0.0:
            f.normal_flip()
    for i in range(n):
        j = (i + 1) % n
        (xa, za), (xb, zb) = ring[i], ring[j]
        face(part(name, fam), (P(xa, y1, za), P(xa, y0, za), P(xb, y0, zb), P(xb, y1, zb)))


def butte_stack(name, b, rng):
    """A butte as stacked plates on a skirt: its core about half its bounds (the rest is the rubble slope), narrowing a little to the cap."""
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    rx, rz = (x1 - x0) * 0.5, (z1 - z0) * 0.5
    AXES[name] = (cx, -cz)
    lobes = [(m, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for m in (2, 3, 5)]
    h = y1 - FLOOR
    # The rubble skirt: two wide, low plates.
    # (Not much wider than the body, and taller: at the bounds' full width they read as hat brims.)
    plate(name, "rock_b", cx, cz, rx * 0.66, rz * 0.66, FLOOR - 0.5, FLOOR + h * 0.1, lobes, 0)
    plate(name, "rock_a", cx, cz, rx * 0.58, rz * 0.58, FLOOR + h * 0.1, FLOOR + h * 0.18, lobes, 1)
    # The body: plates a band each.
    n = 7
    y = FLOOR + h * 0.18
    for k in range(n):
        t = k / (n - 1)
        th = (y1 - y) / (n - k)
        r = 0.52 - 0.05 * t
        ox, oz = rng.uniform(-0.6, 0.6), rng.uniform(-0.4, 0.4)
        plate(name, STRATA[k % len(STRATA)], cx + ox, cz + oz, rx * r, rz * r, y, y + th, lobes, k + 2)
        y += th


def arch_stack(name, b, rng):
    """The arch as plates: two legs of stacked plates and a span of three long plates bowing across their tops."""
    x0, y0, z0, x1, y1, z1 = b
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    w, d, h = x1 - x0, z1 - z0, y1 - FLOOR
    AXES[name] = (cx, -cz)
    lobes = [(m, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for m in (2, 3)]
    leg_h = h * 0.72
    for side in (-1, 1):
        lx = cx + side * (w * 0.5 - 3.0)
        y = FLOOR - 0.3
        k = 0
        while y < FLOOR + leg_h:
            th = rng.uniform(1.8, 2.6)
            r = 3.0 - 0.6 * (y - FLOOR) / leg_h
            plate(name, STRATA[k % len(STRATA)], lx + rng.uniform(-0.3, 0.3), cz, r, d * 0.45, y, y + th, lobes, k + (0 if side < 0 else 7))
            y += th
            k += 1
    # The span: three bands, each a row of short plates following a shallow bow.
    for band, (lift, th, fam) in enumerate(((0.0, 2.2, "rock_b"), (2.2, 0.9, "rock_c"), (3.1, 1.6, "rock_a"))):
        segs = 6
        for s in range(segs):
            t0, t1 = s / segs, (s + 1) / segs
            xa = x0 + 0.8 + (w - 1.6) * t0
            xb = x0 + 0.8 + (w - 1.6) * t1
            ya = FLOOR + leg_h - 1.0 + lift + 1.8 * math.sin(t0 * math.pi)
            yb = FLOOR + leg_h - 1.0 + lift + 1.8 * math.sin(t1 * math.pi)
            ym = (ya + yb) * 0.5
            plate(name, fam, (xa + xb) * 0.5, cz, (xb - xa) * 0.66, d * (0.42 - band * 0.04), ym, ym + th, lobes, s + band * 3, n=16)


# ---- Pop-up cut-outs ----------------------------------------------------------------------------------------------------------------


def cutout(name, fam, profile, z, bottom, thick=0.6, hole=None):
    """A standing card: its top edge `profile` [(x, y)], down to `bottom`, at depth `z`, with the cut edge along the top showing its
    thickness. `hole` (x0, x1, y_top): an opening cut out of it from the bottom up to y_top."""
    for i in range(len(profile) - 1):
        (xa, ya), (xb, yb) = profile[i], profile[i + 1]
        lo_a, lo_b = bottom, bottom
        if hole and xa >= hole[0] and xb <= hole[1]:
            lo_a = lo_b = hole[2]
            face(part(name, "card_" + fam), (P(xa, lo_a, z), P(xa, lo_a, z - thick), P(xb, lo_b, z - thick), P(xb, lo_b, z)))
        face(part(name, fam), (P(xa, ya, z), P(xa, lo_a, z), P(xb, lo_b, z), P(xb, ya, z)))
        face(part(name, "card_" + fam), (P(xa, ya, z - thick), P(xa, ya, z), P(xb, yb, z), P(xb, yb, z - thick)))


def stepped_profile(cx, half, top, rng, tiers=4, skirt=None):
    """A butte's outline as steps: each band a little narrower, its corners squared off, over a sloping skirt."""
    pts = []
    h = top - FLOOR
    ys = [FLOOR + h * (0.16 + 0.84 * k / tiers) for k in range(tiers + 1)]
    widths = [half * (1.0 - 0.06 * k) * rng.uniform(0.96, 1.04) for k in range(tiers)]
    skirt = skirt or half * 1.7
    pts.append((cx - skirt, FLOOR))
    pts.append((cx - widths[0] - 0.4, ys[0]))
    for k in range(tiers):
        pts.append((cx - widths[k], ys[k]))
        pts.append((cx - widths[k], ys[k + 1]))
    for k in reversed(range(tiers)):
        pts.append((cx + widths[k], ys[k + 1]))
        pts.append((cx + widths[k], ys[k]))
    pts.append((cx + widths[0] + 0.4, ys[0]))
    pts.append((cx + skirt, FLOOR))
    # (Sorted along x for the columns of the card: equal x at a step is a vertical edge.)
    return pts


def butte_cutouts(name, b, rng):
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    half = (x1 - x0) * 0.5 * 0.52
    # (The palest card at the back, the deepest in front as the base: the pale one in front washed the buttes out.)
    for k, (dz, sc, fam) in enumerate(((-(z1 - z0) * 0.2, 1.0, "rock_c"), (0.0, 0.9, "rock_a"), ((z1 - z0) * 0.2, 0.62, "rock_b"))):
        top = FLOOR + (y1 - FLOOR) * sc
        prof = stepped_profile(cx + rng.uniform(-1.5, 1.5), half * (0.8 + 0.2 * sc), top, rng)
        prof = monotone(prof)
        cutout("%s_%d" % (name, k), fam, prof, cz + dz, FLOOR - 1.0)


def monotone(pts):
    """Columns for a card from an outline that may step straight up: nudge equal xs apart so each column has width."""
    out = []
    for x, y in pts:
        if out and x <= out[-1][0]:
            x = out[-1][0] + 0.02
        out.append((x, y))
    return out


def arch_cutout(name, b, rng):
    x0, y0, z0, x1, y1, z1 = b
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    w = x1 - x0
    prof = []
    n = 24
    for i in range(n + 1):
        u = i / n
        x = x0 + w * u
        y = y1 - 0.8 * abs(math.sin(u * 7.0)) - (2.0 if u < 0.08 or u > 0.92 else 0.0)
        prof.append((x, y))
    opening = (x0 + 5.0, x1 - 5.0, FLOOR + (y1 - FLOOR) * 0.72)
    for k, (dz, fam) in enumerate(((-1.2, "rock_b"), (0.6, "rock_a"))):
        cutout("%s_%d" % (name, k), fam, prof, cz + dz, FLOOR - 1.0, hole=opening)


# ---- Dunes --------------------------------------------------------------------------------------------------------------------------


def dune_flats():
    for r_i, spec in enumerate(LAYOUT.get("ranges", [])):
        fam = "dune_far" if r_i == 0 else "dune_near"
        raw = spec["top"]
        peak = [max(raw[j][1] for j in range(max(0, i - 1), min(len(raw), i + 2))) for i in range(len(raw))]
        top = []
        for i in range(len(raw)):
            near = peak[max(0, i - 2): i + 3]
            top.append((raw[i][0], sum(near) / len(near)))
        z = spec["z"]
        base = min(t[1] for t in top) - 6.0
        for k, (dz, scale) in enumerate(((-8.0, 1.08), (0.0, 1.0), (8.0, 0.9))):
            pts = [(x, base + (y - base) * scale) for (x, y) in top]
            cutout("range%d_%d" % (r_i, k), fam, pts, z + dz, base - 20.0, thick=0.8)


# ---- Build --------------------------------------------------------------------------------------------------------------------------


def build_ground():
    rng = random.Random(53)
    for k, m in enumerate(LAYOUT.get("mesas", [])):
        far = m[6] == 1
        name = "butte%d" % k
        if VARIANT == "B" or (VARIANT == "C" and far):
            butte_cutouts(name, m, rng)
        else:
            butte_stack(name, m, rng)
    if LAYOUT.get("arch"):
        if VARIANT == "B":
            arch_cutout("arch", LAYOUT["arch"], rng)
        else:
            arch_stack("arch", LAYOUT["arch"], rng)
    dune_flats()


def make_objects():
    for (name, mat), bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        ax = AXES.get(name)
        for f in bm.faces:
            c = f.calc_center_median()
            if ax is not None:
                # (A stack's plates: tops up, cut edges out from its middle.)
                if abs(f.normal.z) > 0.9:
                    continue
                away = Vector((c.x - ax[0], c.y - ax[1], 0.0))
            else:
                # (A cut-out faces the camera; its top edge faces up.)
                away = Vector((0.0, -1.0, 0.4))
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
    print("canyon paper %s: %d triangles -> %s" % (VARIANT, tris, path))


main()
