"""The grove's trees as modelled pieces (the stage refinement's grove pilot, after the modelled front; docs/art/stages/BAKE_PILOT.md).

One tree at the code-built tree's own size 1 (an 18-unit trunk, its crown spreading about 9 either side), standing on the origin; the
game places it where each code-built tree stood, scaled to that tree's size (`--tree-mock=` on the stage demo, StageBake.place_trees).
Three looks for the owner to pick from:
  A  a twisted trunk of ropey ridges spiralling up from buttress roots, under a crown of big deep-pleated folded domes;
  B  the same trunk with thick limbs, each carrying a heap of smaller leaf clumps, the grove reference's leafy canopy;
  C  a trunk of broad folded planes under stacked pleated tiers, a papercraft canopy matching the folded pines and hills.
After fidelity review 6 (one tree at one size all round read as a clone), C in more shapes of the same tree (`--shape`):
  `tall`   six narrower tiers climbing higher up a slimmer trunk;
  `broad`  four wide, low tiers on a shorter trunk, the crown set a little off its axis.
Every look has a mossy foot whose roots splay into the ground, and bakes its own occlusion into linear vertex colours. Material
families `bark`, `leaf_lit`, `leaf_shade`, `moss`, painted from the stage's palette in the game.

Shipped since fidelity review 6 (the owner's pick B, 2026-10-22): grove_tree.glb (C), grove_tree_tall.glb (`--variant C --shape
tall`) and grove_tree_broad.glb (`--variant C --shape broad`), StageBake.TREES.

Run (Blender 5.2):
  blender --background --python art/blender/make_grove_trees.py -- --variant A --out <file.glb>
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
SHAPE = opts.get("shape", "")
TRUNK_H = 18.0
R_BOT, R_TOP = 2.9, 2.0


def trunk(bm, ridges, twist, depth, sharp, sides=56, rings=64, flare=1.6, bend=0.7):
    """A trunk lofted ring by ring: `ridges` lobes round it (each `depth` of the radius, `sharp` how ropey) spiralling `twist` turns
    up its height, tapering and bowing a little, and at its foot the lobes splaying out into buttress roots that dip into the ground."""
    grid = []
    for j in range(rings + 1):
        v = j / rings
        h = -1.2 + (TRUNK_H + 1.2) * v
        r = R_BOT + (R_TOP - R_BOT) * v
        foot = math.exp(-max(h, 0.0) * 0.55)
        cx = math.sin(v * math.pi * 0.9) * bend
        row = []
        for i in range(sides):
            a = math.tau * i / sides
            lobe = max(0.0, math.cos(ridges * (a + twist * math.tau * v)))
            lobe = lobe ** sharp
            rr = r * (1.0 - depth * 0.5 + depth * lobe) + flare * foot * lobe * 2.2
            z = h - 0.6 * foot * lobe if h < 0.5 else h
            row.append(bm.verts.new((cx + math.cos(a) * rr, math.sin(a) * rr, z)))
        grid.append(row)
    for j in range(rings):
        for i in range(sides):
            k = (i + 1) % sides
            bm.faces.new((grid[j][i], grid[j][k], grid[j + 1][k], grid[j + 1][i]))
    cap = bm.verts.new((math.sin(math.pi * 0.9) * bend, 0, TRUNK_H))
    for i in range(sides):
        bm.faces.new((grid[-1][i], grid[-1][(i + 1) % sides], cap))
    bm.normal_update()


def limb(bm, a, b, r0, r1, sides=10):
    """A thick limb from `a` to `b`, tapering, bowing upward in the middle."""
    pts = []
    for k in range(9):
        t = k / 8
        p = a.lerp(b, t) + Vector((0, 0, math.sin(t * math.pi) * (b - a).length * 0.12))
        pts.append(p)
    rings = []
    for k, p in enumerate(pts):
        fwd = (pts[min(k + 1, 8)] - pts[max(k - 1, 0)]).normalized()
        side = fwd.cross(Vector((0, 0, 1)))
        if side.length < 1e-3:
            side = Vector((1, 0, 0))
        side.normalize()
        up = side.cross(fwd).normalized()
        rad = r0 + (r1 - r0) * (k / 8)
        rings.append([bm.verts.new(p + (side * math.cos(math.tau * i / sides) + up * math.sin(math.tau * i / sides)) * rad)
                      for i in range(sides)])
    for k in range(8):
        for i in range(sides):
            j = (i + 1) % sides
            f = bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
            f.normal_update()
            if f.normal.dot(f.calc_center_median() - pts[k]) < 0:
                f.normal_flip()


def dome(lit, shade, c, r, pleats, rng, squash=1.0):
    n_l, n_s = len(lit.verts), len(shade.verts)
    folded_dome(lit, shade, tuple(c), r, pleats, rng, rows=[(-0.12, 0.44), (0.32, 0.3), (0.68, 0.14)], stagger=False)
    if squash != 1.0:
        m = Matrix.Diagonal((1.0, 1.0, squash))
        for bm, n in ((lit, n_l), (shade, n_s)):
            bm.verts.ensure_lookup_table()
            for v in bm.verts[n:]:
                v.co = m @ (v.co - c) + c


def moss_foot(lit, shade, rng):
    """Moss heaped round the foot, between the roots."""
    for k in range(9):
        a = math.tau * k / 9 + rng.uniform(-0.2, 0.2)
        rr = R_BOT * rng.uniform(1.0, 1.6)
        dome(lit, shade, Vector((math.cos(a) * rr, math.sin(a) * rr, 0.2)), rng.uniform(0.9, 1.5), 6, rng, squash=0.45)


def obj(name, bm, mat_name, smooth):
    me = bpy.data.meshes.new(name)
    bm.normal_update()
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = smooth
    mat = bpy.data.materials.get(mat_name) or bpy.data.materials.new(mat_name)
    me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def build():
    rng = random.Random(11)
    bark = bmesh.new()
    lit, shade = bmesh.new(), bmesh.new()
    if VARIANT == "A":
        trunk(bark, ridges=9, twist=0.55, depth=0.32, sharp=2.2)
        # (At the code-built crown's footprint, about 12 either side and 9 tall: at two thirds of it the side trees read as bonsai.)
        for c, r in [((0, 0.3, 22.4), 6.2), ((-8.4, 0.8, 19.2), 5.0), ((8.5, 0.6, 19.6), 5.0), ((-4.2, -0.6, 25.2), 4.6),
                     ((4.4, -0.5, 25.0), 4.6), ((-2.8, -2.8, 18.6), 3.8), ((3.2, -2.6, 18.8), 3.6)]:
            dome(lit, shade, Vector(c), r, 9 if r > 4.8 else 8, rng)
    elif VARIANT == "B":
        trunk(bark, ridges=9, twist=0.55, depth=0.32, sharp=2.2)
        top = Vector((math.sin(math.pi * 0.9) * 0.7, 0, TRUNK_H - 1.0))
        clusters = [Vector((-9.2, 0.5, 20.4)), Vector((9.4, 0.3, 20.8)), Vector((-3.8, -0.4, 25.6)), Vector((4.2, -0.6, 25.2)),
                    Vector((0.2, 0.8, 22.6))]
        for cc in clusters[:4]:
            limb(bark, top + Vector((0, 0, rng.uniform(-2.5, 0.5))), cc - Vector((0, 0, 1.2)), 1.1, 0.45)
        for cc in clusters:
            # A heap of clumps: a big middle one, smaller ones round and over it, darker underneath.
            dome(lit, shade, cc, 3.4, 8, rng)
            for k in range(9):
                a = math.tau * k / 9 + rng.uniform(-0.3, 0.3)
                el = rng.uniform(-0.2, 0.9)
                d = rng.uniform(2.6, 3.9)
                p = cc + Vector((math.cos(a) * d * math.cos(el) * 1.3, -abs(math.sin(a)) * d * 0.6 - 0.5,
                                 math.sin(el) * d * 0.9))
                dome(lit, shade, p, rng.uniform(1.6, 2.5), 7, rng)
    else:
        trunk(bark, ridges=6, twist=0.18, depth=0.4, sharp=1.0, sides=36, rings=24)
        # (Five tiers from two thirds up the trunk: starting at its top, the side trees read as pines on stilts.)
        tiers = [(12.6, 11.5, 3.8, 13), (15.6, 10.4, 3.6, 13), (18.6, 8.6, 3.4, 12), (21.4, 6.2, 3.2, 10), (23.8, 3.6, 2.8, 8)]
        axis = (0.6, 0.0)
        if SHAPE == "tall":
            tiers = [(12.0, 9.2, 3.4, 12), (14.8, 8.4, 3.3, 12), (17.6, 7.4, 3.2, 11), (20.4, 6.0, 3.0, 10), (23.0, 4.4, 2.8, 9),
                     (25.4, 2.6, 2.6, 7)]
        elif SHAPE == "broad":
            tiers = [(11.4, 13.2, 3.6, 14), (14.2, 11.6, 3.4, 13), (17.0, 8.8, 3.2, 12), (19.6, 5.0, 2.8, 9)]
            axis = (1.6, 0.4)
        for z, r, h, pleats in tiers:
            pleated_tier(lit, shade, axis, z, r, h, pleats, rng.uniform(0, math.tau), rng, flare=0.55, droop=0.35)
    moss_foot(lit, shade, rng)
    obj("Bark", bark, "bark", smooth=VARIANT != "C")
    obj("LeafLit", lit, "leaf_lit", smooth=False)
    obj("LeafShade", shade, "leaf_shade", smooth=False)


def bake_occlusion():
    """Each piece's occlusion into a float colour attribute (linear, as the game reads COLOR), the ground under the tree casting it too."""
    scene = bpy.context.scene
    parts = [o for o in scene.objects if o.type == "MESH"]
    bpy.ops.mesh.primitive_plane_add(size=60.0, location=(0, 0, 0))
    ground = bpy.context.active_object
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
    scene.world.light_settings.distance = 4.0
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        a = o.data.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
        o.data.color_attributes.active_color = a
        o.data.color_attributes.render_color_index = o.data.color_attributes.active_color_index
        o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.bake(type="AO", target="VERTEX_COLORS")
    bpy.data.objects.remove(ground, do_unlink=True)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build()
    bake_occlusion()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("grove tree %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
