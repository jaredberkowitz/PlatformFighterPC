"""The grove's background ground and its bushes as modelled pieces (the stage refinement's grove pilot, after its front and trees;
docs/art/stages/BAKE_PILOT.md). Built in world space round the stage, in the game's units; the side trees stand at x -30 and 32, depth
-55, on ground at about -9.3. Three looks for the owner to pick from (`--ground-mock=`, `--bush-mock=` on the stage demo):
  A  stepped ledges: grassy terraces with faceted rock cliffs, the nearest holding the side trees, the far ones split behind the island,
     small pleated trees on them; bushes as heaps of folded leaf clumps;
  B  a forest: a folded grassy rise holding the side trees, and below and behind it a canopy of pleated trees falling away down the
     valley, smaller and airier with distance; bushes as pleated fern rosettes;
  C  floating islets: no ground, only rock islets of stones and hanging roots at several depths, each capped in grass, two holding the
     side trees, the others small pleated trees; bushes as heaps of folded leaf clumps.
Each piece is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `grass`, `grass_far`, `cliff`, `earth`, `rock`, `root`, `bark`, `leaf_lit`, `leaf_shade`, `moss`, `fern`, `fern_shade`.

Run (Blender 5.2):
  blender --background --python art/blender/make_grove_ground.py -- --variant A --ground <ground.glb> --bush <bush.glb>
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold_kit import folded_dome, pleated_tier  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
TREES = [(-30.0, -9.38, -55.0, 0.6), (32.0, -9.02, -55.0, 0.55)]


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}


def part(name, mat):
    """The bmesh collecting one object's faces (one object per piece and material, so the game can give each the air of its depth)."""
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
    return PARTS[key]


def slab(name, x0, x1, z_front, z_back, top, rng, drop=60.0, wobble=2.0, cols=48, grass="grass", cliff="cliff"):
    """A terrace: a grassy top from `z_front` back to `z_back` at `top`, its front edge wavering in and out, folded in shallow ridges, and
    a faceted cliff falling from that edge `drop` units, its planes jutting in and out (the front's language, far simpler)."""
    tb = part(name, grass)
    cb = part(name + "_cliff", cliff)
    edge = []
    for i in range(cols + 1):
        x = x0 + (x1 - x0) * i / cols
        zf = z_front + wobble * math.sin(x * 0.17 + rng.uniform(-0.3, 0.3)) + rng.uniform(-0.6, 0.6)
        edge.append((x, zf))
    top_rows = []
    for k, t in enumerate((0.0, 0.35, 1.0)):
        row = []
        for x, zf in edge:
            z = zf + (z_back - zf) * t
            y = top + (0.25 * math.sin(x * 0.5) if k == 1 else 0.0)
            row.append(tb.verts.new(P(x, y, z)))
        top_rows.append(row)
    for k in range(2):
        for i in range(cols):
            a, b, c, d = top_rows[k][i], top_rows[k][i + 1], top_rows[k + 1][i + 1], top_rows[k + 1][i]
            tb.faces.new((a, b, c, d))
    rows = []
    for k, dy in enumerate((0.0, 1.2, 4.0, 9.0, 18.0, drop)):
        row = []
        for x, zf in edge:
            out = 0.0 if k == 0 else rng.uniform(-1.6, 1.6) + (0.8 if k == 1 else 0.0)
            row.append(cb.verts.new(P(x + (rng.uniform(-0.6, 0.6) if k else 0.0), top - dy, zf + out)))
        rows.append(row)
    for k in range(len(rows) - 1):
        for i in range(cols):
            a, b, c, d = rows[k][i], rows[k][i + 1], rows[k + 1][i + 1], rows[k + 1][i]
            cb.faces.new((a, d, c, b))


def mini_tree(name, base, sc, rng):
    """A small pleated tree, the grove tree's tiered canopy in miniature: a tapered trunk and three or four pleated tiers."""
    bk = part(name, "bark")
    res = bmesh.ops.create_cone(bk, cap_ends=True, segments=6, radius1=0.3 * sc, radius2=0.14 * sc, depth=3.2 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 1.6 * sc))
    lit, shade = part(name, "leaf_lit"), part(name, "leaf_shade")
    tiers = [(1.4, 1.7, 1.3), (2.3, 1.3, 1.2), (3.1, 0.9, 1.1), (3.8, 0.5, 0.9)][: rng.choice((3, 4))]
    for z, r, h in tiers:
        pleated_tier(lit, shade, (base.x, base.y), base.z + z * sc, r * sc, h * sc, 8, rng.uniform(0, math.tau), rng, flare=0.5,
                     droop=0.18 * sc)


def dome_heap(lit, shade, c, r, rng, n=4):
    folded_dome(lit, shade, tuple(c), r, 7, rng, rows=[(-0.12, 0.44), (0.32, 0.3), (0.68, 0.14)], stagger=False)
    for k in range(n):
        a = math.tau * k / n + rng.uniform(-0.3, 0.3)
        p = c + Vector((math.cos(a) * r * 0.9, math.sin(a) * r * 0.6, -r * 0.25 + rng.uniform(-0.1, 0.15) * r))
        folded_dome(lit, shade, tuple(p), r * rng.uniform(0.55, 0.75), 6, rng, rows=[(-0.12, 0.44), (0.32, 0.3), (0.68, 0.14)],
                    stagger=False)


def islet(name, cx, top, cz, r, rng, trees=()):
    """A floating islet: a grass cap and a cone of faceted stones under it, a few roots hanging."""
    g = part(name, "grass")
    cap_c = P(cx, top - 0.4, cz)
    n0 = len(g.verts)
    res = bmesh.ops.create_cone(g, cap_ends=True, segments=14, radius1=r, radius2=r * 0.9, depth=0.8)
    for v in res["verts"]:
        v.co = Vector((v.co.x * rng.uniform(0.95, 1.05), v.co.y * rng.uniform(0.95, 1.05), v.co.z)) + cap_c
    rk = part(name, "rock")
    depth = r * 1.6
    for k in range(int(r * 4)):
        t = rng.random()
        y = top - 0.8 - t * depth
        rr = (1.0 - t) * r * 0.85 + 0.6
        a = rng.uniform(0, math.tau)
        c = P(cx + math.cos(a) * rr * rng.uniform(0.3, 0.9), y, cz + math.sin(a) * rr * rng.uniform(0.3, 0.9))
        s = rng.uniform(0.7, 1.4) * (1.0 - t * 0.5) * max(0.6, r / 5.0)
        res = bmesh.ops.create_icosphere(rk, subdivisions=1, radius=s)
        for v in res["verts"]:
            v.co = Vector((v.co.x * 1.2, v.co.y, v.co.z * 0.85)) * rng.uniform(0.85, 1.1) + c
    rt = part(name, "root")
    for k in range(3):
        a = rng.uniform(0, math.tau)
        x, z = cx + math.cos(a) * r * 0.6, cz + math.sin(a) * r * 0.4
        length = rng.uniform(depth * 0.8, depth * 1.4)
        res = bmesh.ops.create_cone(rt, cap_ends=True, segments=6, radius1=0.08, radius2=0.35, depth=length)
        for v in res["verts"]:
            v.co += P(x, top - 0.6 - length * 0.5, z)
    for tx, ty, tz, sc in trees:
        mini_tree(name + "_trees", P(tx, ty, tz), sc, rng)


def build_ground():
    rng = random.Random(23)
    if VARIANT == "A":
        slab("T0", -150, 150, -46, -64, -9.3, rng)
        slab("T1L", -150, -16, -70, -90, -5.0, rng, grass="grass_far")
        slab("T1R", 18, 150, -72, -92, -4.5, rng, grass="grass_far")
        slab("T2", -170, 170, -98, -122, -1.0, rng, grass="grass_far")
        for name, x0, x1, z0, z1, y, n in (("T0", -140, 140, -48, -62, -9.3, 40), ("T1L", -140, -22, -72, -88, -5.0, 24),
                                           ("T1R", 24, 140, -74, -90, -4.5, 24), ("T2", -160, 160, -100, -120, -1.0, 36)):
            for k in range(n):
                x = rng.uniform(x0, x1)
                # (Clear of the side trees, and of the space behind the fight on the near ledges: a small tree behind the left
                # platform drew the eye at the match camera.)
                if abs(x + 30) < 8 or abs(x - 32) < 8 or abs(x) < (24 if name != "T2" else 14):
                    continue
                mini_tree(name + "_trees", P(x, y, rng.uniform(z0, z1)), rng.uniform(1.6, 2.8), rng)
    elif VARIANT == "B":
        slab("T0", -150, 150, -47, -60, -9.3, rng, drop=6.0)
        # The forest falling away below and behind the rise: rows of pleated trees, each row lower, further and smaller.
        for row in range(7):
            z = -50 - row * 6.5
            y = -12.0 - row * 2.6
            name = "F%d" % row
            slab(name + "_floor", -170, 170, z + 3.0, z - 4.0, y - 0.4, rng, drop=40.0, wobble=1.0, grass="grass_far")
            x = -165.0 + rng.uniform(0, 4)
            while x < 165.0:
                sc = rng.uniform(2.2, 3.2) * (1.0 - row * 0.06)
                mini_tree(name, P(x, y, z + rng.uniform(-2.0, 2.0)), sc, rng)
                x += sc * rng.uniform(1.6, 2.4)
        slab("T2", -170, 170, -100, -124, -4.0, rng, grass="grass_far")
    else:
        sides = [("I0", TREES[0][0], TREES[0][1], TREES[0][2], 6.5, ()), ("I1", TREES[1][0], TREES[1][1], TREES[1][2], 6.5, ())]
        more = [("I2", -66, -4.0, -82, 9.0), ("I3", 62, -2.5, -86, 8.0), ("I4", -16, -20.0, -66, 4.5), ("I5", 22, -23.0, -78, 5.5),
                ("I6", -2, 3.0, -112, 11.0), ("I7", -110, 1.0, -104, 8.0), ("I8", 108, 0.0, -100, 8.5), ("I9", -48, -16.0, -58, 4.0)]
        for name, x, y, z, r, tr in sides:
            islet(name, x, y + 0.4, z, r, rng)
        for name, x, y, z, r in more:
            trees = []
            for k in range(max(1, int(r / 3.5))):
                a = rng.uniform(0, math.tau)
                trees.append((x + math.cos(a) * r * 0.45, y, z + math.sin(a) * r * 0.3, rng.uniform(1.4, 2.4)))
            islet(name, x, y, z, r, rng, trees)


def build_bush():
    """One bush at size 1 on the origin (the game sets each at the code-built bush's place, size and turn)."""
    rng = random.Random(5)
    if VARIANT == "B":
        lit, shade = part("Bush", "fern"), part("Bush", "fern_shade")
        pleated_tier(lit, shade, (0.0, 0.0), 0.05, 0.95, 0.55, 9, 0.0, rng, flare=0.28, droop=0.22)
        pleated_tier(lit, shade, (0.05, 0.0), 0.32, 0.62, 0.5, 8, 0.4, rng, flare=0.3, droop=0.18)
        pleated_tier(lit, shade, (0.0, 0.03), 0.55, 0.32, 0.4, 7, 0.9, rng, flare=0.3, droop=0.1)
    else:
        lit, shade = part("Bush", "leaf_lit"), part("Bush", "leaf_shade")
        dome_heap(lit, shade, Vector((0, 0, 0.45)), 0.55, rng)


def make_objects():
    for (name, mat), bm in PARTS.items():
        me = bpy.data.meshes.new(name + "_" + mat)
        bm.normal_update()
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


def export(path):
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    path = os.path.abspath(path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("grove ground %s: %d triangles -> %s" % (VARIANT, tris, path))


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_ground()
    make_objects()
    bake_occlusion(4.0)
    export(opts["ground"])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_bush()
    make_objects()
    bpy.ops.mesh.primitive_plane_add(size=6.0)
    ground = bpy.context.active_object
    bake_occlusion(0.8)
    bpy.data.objects.remove(ground, do_unlink=True)
    export(opts["bush"])


main()
