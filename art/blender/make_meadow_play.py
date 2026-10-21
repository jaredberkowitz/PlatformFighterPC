"""The meadow's play area as paper card (the paper pilot after the second fidelity review, docs/art/stages/FIDELITY_REVIEW_2.md): the lawn
the fighters stand on and the two platforms, on the collision from the bake manifest. The top of every walkable surface stays exactly at
the sim's height; the modelled front under the lawn stays. Three looks for the owner to pick from (`--play-mock=` on the stage demo):
  A  card slab: the lawn one thick card, its cut edge pale, the cream path an inlaid strip; the platforms three card boards side by side
     on two battens, a rope band round each end;
  B  layered sheets: the lawn a thin green sheet over a soil sheet and a stone sheet, each a step wider, so the edge shows the layers; the
     platforms a green sheet over a plank card with a scalloped grass fringe hanging from the front;
  C  pop-up: the lawn a thin sheet with a standing ground-row card in front of its edge, its top cut in grass scallops; the platforms
     plank-shaped cut-outs with notched ends over a soft cast-shadow card.
Families `lawn`, `lawn_under`, `path`, `card`, `soil`, `stone`, `fringe`, `plank`, `plank_b`, `plank_card`, `rope`, `shadow`.

Run (Blender 5.2):
  blender --background --python art/blender/make_meadow_play.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
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
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
FZ = float(manifest["front_z"])
BLOCK = [r for r in manifest["rects"] if r[4] < 0.5][0]
PLATFORMS = [r for r in manifest["rects"] if r[4] > 0.5]
DEPTH = 8.0          # the block's depth back from its front
PLAT_DEPTH = 4.0     # StageArt.PLATFORM_DEPTH


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}


def part(mat):
    if mat not in PARTS:
        PARTS[mat] = bmesh.new()
    return PARTS[mat]


def face(mat, pts):
    bm = part(mat)
    return bm.faces.new([bm.verts.new(p) for p in pts])


def box(x0, x1, y0, y1, z0, z1, top, side, bottom=None):
    """A card box: its top in `top`, its four sides in `side` (the cut edge), its underside in `bottom` (or `side`)."""
    c = [P(x, y, z) for x in (x0, x1) for y in (y0, y1) for z in (z0, z1)]
    # index = xi*4 + yi*2 + zi
    def v(xi, yi, zi):
        return c[xi * 4 + yi * 2 + zi]
    face(top, (v(0, 1, 1), v(1, 1, 1), v(1, 1, 0), v(0, 1, 0)))
    face(bottom or side, (v(0, 0, 0), v(1, 0, 0), v(1, 0, 1), v(0, 0, 1)))
    face(side, (v(0, 0, 1), v(1, 0, 1), v(1, 1, 1), v(0, 1, 1)))
    face(side, (v(1, 0, 0), v(0, 0, 0), v(0, 1, 0), v(1, 1, 0)))
    face(side, (v(0, 0, 0), v(0, 0, 1), v(0, 1, 1), v(0, 1, 0)))
    face(side, (v(1, 0, 1), v(1, 0, 0), v(1, 1, 0), v(1, 1, 1)))


def path_strip(x0, x1, zc, half, rng):
    """The cream path, an inlaid strip along the play line, its edges wavering."""
    xs = [x0 + (x1 - x0) * i / 60 for i in range(61)]
    ph = rng.uniform(0, 6)
    for i in range(60):
        xa, xb = xs[i], xs[i + 1]

        def edge(x, s):
            return zc + s * (half + 0.12 * math.sin(x * 0.9 + ph) + 0.07 * math.sin(x * 2.3 + ph * 2.0))
        face("path", (P(xa, 0.006, edge(xa, -1)), P(xa, 0.006, edge(xa, 1)), P(xb, 0.006, edge(xb, 1)), P(xb, 0.006, edge(xb, -1))))


def scallops(x0, x1, y_top, y_low, z, mat, size=0.45, up=False):
    """A cut card edge of grass scallops along x at depth z: from y_low up to y_top, the top edge (or the bottom, `up` False hangs them)
    in rounded points."""
    n = max(2, int((x1 - x0) / size))
    w = (x1 - x0) / n
    for i in range(n):
        xa, xb = x0 + w * i, x0 + w * (i + 1)
        xm = (xa + xb) * 0.5
        if up:
            face(mat, (P(xa, y_low, z), P(xb, y_low, z), P(xb, y_top - w * 0.45, z), P(xm, y_top, z), P(xa, y_top - w * 0.45, z)))
        else:
            face(mat, (P(xa, y_top, z), P(xa, y_low + w * 0.45, z), P(xm, y_low, z), P(xb, y_low + w * 0.45, z), P(xb, y_top, z)))


def lawn(rng):
    x0, x1, top = BLOCK[0] - 0.12, BLOCK[1] + 0.12, BLOCK[2]
    zb, zf = FZ - DEPTH - 0.05, FZ + 0.06
    if VARIANT == "A":
        box(x0, x1, top - 0.5, top, zb, zf, "lawn", "card", "lawn_under")
    elif VARIANT == "B":
        box(x0, x1, top - 0.2, top, zb, zf, "lawn", "card", "lawn_under")
        box(x0 - 0.12, x1 + 0.12, top - 0.55, top - 0.2, zb, zf + 0.12, "soil", "soil")
        box(x0 - 0.24, x1 + 0.24, top - 0.95, top - 0.55, zb, zf + 0.24, "stone", "stone")
    else:
        box(x0, x1, top - 0.15, top, zb, zf, "lawn", "card", "lawn_under")
        # The ground row: a standing card in front of the lawn's edge, cut in grass scallops, never above the walking surface.
        scallops(x0 - 0.2, x1 + 0.2, top - 0.02, top - 1.4, zf + 0.35, "fringe", 0.55, up=True)
    # (Nearer the painted path's width: at 1.1 either side, in the pilot's build, it took a third of the lawn and paled it.)
    path_strip(BLOCK[0] + 0.2, BLOCK[1] - 0.2, FZ - 1.9, 0.7, rng)


def platform(r, rng):
    x0, x1, top = r[0], r[1], r[2]
    zb, zf = FZ - PLAT_DEPTH, FZ
    if VARIANT == "A":
        gap = 0.1
        bw = (zf - zb - 2 * gap) / 3.0
        for k in range(3):
            za = zb + k * (bw + gap)
            box(x0 - 0.1, x1 + 0.1, top - 0.28, top, za, za + bw, "plank", "plank_card", "plank_b")
        for bx in (x0 + 0.4, x1 - 0.4):
            box(bx - 0.25, bx + 0.25, top - 0.55, top - 0.28, zb + 0.1, zf - 0.1, "plank_b", "plank_b")
            # The rope band: a strip round the front and the back of the boards.
            box(bx - 0.12, bx + 0.12, top - 0.3, top - 0.02, zf - 0.02, zf + 0.04, "rope", "rope")
            box(bx - 0.12, bx + 0.12, top - 0.3, top - 0.02, zb - 0.04, zb + 0.02, "rope", "rope")
    elif VARIANT == "B":
        box(x0 - 0.1, x1 + 0.1, top - 0.12, top, zb, zf, "lawn", "card", "lawn_under")
        box(x0 - 0.04, x1 + 0.04, top - 0.45, top - 0.12, zb + 0.05, zf - 0.05, "plank", "plank_card", "plank_b")
        scallops(x0 - 0.1, x1 + 0.1, top - 0.08, top - 0.55, zf + 0.04, "fringe", 0.4)
    else:
        box(x0 - 0.1, x1 + 0.1, top - 0.1, top, zb, zf, "plank", "plank_card", "plank_b")
        # The cut-out plank front with notched ends, and its soft shadow card behind and below.
        prof = [(x0 - 0.1, top - 0.05), (x0 + 0.25, top - 0.05), (x0 + 0.25, top - 0.15), (x1 - 0.25, top - 0.15), (x1 - 0.25, top - 0.05),
                (x1 + 0.1, top - 0.05), (x1 + 0.1, top - 0.6), (x1 - 0.25, top - 0.6), (x1 - 0.25, top - 0.5), (x0 + 0.25, top - 0.5),
                (x0 + 0.25, top - 0.6), (x0 - 0.1, top - 0.6)]
        face("plank_card", [P(x, y, zf + 0.03) for (x, y) in prof])
        face("shadow", [P(x + 0.25, y - 0.25, zf - 0.04) for (x, y) in prof])


def make_objects():
    for mat, bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        for f in bm.faces:
            # (Boxes are wound outward already; flat cards face the camera and the sky.)
            if len(f.verts) != 4 or mat in ("path", "fringe", "shadow"):
                if f.normal.dot(Vector((0.0, -0.6, 1.0))) < 0.0:
                    f.normal_flip()
        me = bpy.data.meshes.new("Play_" + mat)
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new("Play_" + mat, me)
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
            ao = 0.6 + 0.4 * c.color[0]
            c.color = (ao, ao, ao, 1.0)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    rng = random.Random(17)
    lawn(rng)
    for r in PLATFORMS:
        platform(r, rng)
    make_objects()
    # (Subdivided for the vertex bake: the lawn's two triangles would take one shade.)
    for o in [o for o in bpy.context.scene.objects if o.type == "MESH"]:
        bm = bmesh.new()
        bm.from_mesh(o.data)
        for _ in range(4):
            long_edges = [e for e in bm.edges if e.calc_length() > 0.9]
            if not long_edges:
                break
            bmesh.ops.subdivide_edges(bm, edges=long_edges, cuts=1, use_grid_fill=True)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        bm.to_mesh(o.data)
        bm.free()
    bake_occlusion(0.6)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("meadow play %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
