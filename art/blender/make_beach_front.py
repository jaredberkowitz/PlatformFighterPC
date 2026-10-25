"""The beach's dock front as one modelled piece (the stage refinement, after the grove and the meadow; docs/art/stages/BAKE_PILOT.md).

Built on the dock's collision from the bake manifest: under the walkable deck (which stays code-built, its boards and joists) down
into the sea one unit under the block's foot (`StageArt._dock_block`'s water), the front row of supports at the front plane and a back
row behind it. Three looks for the owner to pick from (`--front-mock=` on the stage demo), after the harbour reference's short, thick
posts with a pale waterline band and soft ripple rings round them:
  A  timber pier: thick chamfered square posts, a heavy beam along the front with iron bolts, X bracing in every other bay;
  B  stone quay: courses of big bevelled stone blocks down to the water, wooden fender posts against it, rope lashings;
  C  bundled piles: rough round posts lashed in threes with rope, wide gaps of sea between, a heavy beam along the front.
Families `post`, `beam`, `waterline`, `iron`, `rope`, `harbour_stone`, `foam`, `weed`; each piece bakes its own occlusion into linear
vertex colours with the built beach round it.

Run (Blender 5.2):
  blender --background --python art/blender/make_beach_front.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
BLOCK = [r for r in manifest["rects"] if r[4] < 0.5][0]
LEFT, RIGHT, TOP, BOTTOM = BLOCK[0], BLOCK[1], BLOCK[2], BLOCK[3]
FZ = float(manifest["front_z"])
WATER = BOTTOM - 1.0
DECK_UNDER = TOP - 0.26          # the boards' underside (`_dock_block` thick)
DEPTH = 8.0
BACK_Z = FZ - DEPTH + 1.2


def P(x, y, z):
    return Vector((x, -z, y))


PARTS = {}


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
        PARTS[key].verts.layers.float.new("tone")
    return PARTS[key]


def tone_verts(bm, verts, seed):
    layer = bm.verts.layers.float.get("tone")
    t = random.Random(seed).uniform(0.78, 1.06)
    for v in verts:
        if v.is_valid:
            v[layer] = t


def box(bm, c, size, rng, bevel=0.12, rot=None, seed=None):
    res = bmesh.ops.create_cube(bm, size=2.0)
    verts = res["verts"]
    m = rot or Matrix.Identity(3)
    for v in verts:
        v.co = m @ Vector((v.co.x * size.x, v.co.y * size.y, v.co.z * size.z)) + c
    edges = list({e for v in verts for e in v.link_edges})
    out = bmesh.ops.bevel(bm, geom=edges, offset=min(size.x, size.y, size.z) * bevel * 2.0, segments=1, affect="EDGES")
    tone_verts(bm, out["verts"], seed if seed is not None else rng.randrange(1 << 30))


def column(bm, x, z, y0, y1, r, sides, rng, lean=(0.0, 0.0), seed=None):
    """A post from `y0` up to `y1` (Godot heights) at (x, z): `sides` faces (4 a chamfered square, more a round log), leaning a little."""
    res = bmesh.ops.create_cone(bm, cap_ends=True, segments=sides, radius1=r, radius2=r * 0.96, depth=y1 - y0)
    for v in res["verts"]:
        h = v.co.z + (y1 - y0) * 0.5
        v.co = Matrix.Rotation(math.pi / sides, 3, "Z") @ v.co
        v.co += P(x + lean[0] * h, (y0 + y1) * 0.5, z + lean[1] * h)
    if sides == 4:
        edges = [e for e in {e for v in res["verts"] for e in v.link_edges} if abs((e.verts[0].co - e.verts[1].co).normalized().z) > 0.9]
        bmesh.ops.bevel(bm, geom=edges, offset=r * 0.28, segments=1, affect="EDGES")
    tone_verts(bm, res["verts"], seed if seed is not None else rng.randrange(1 << 30))


def ring(bm, x, z, y, r0, r1, sides=16):
    """A flat ring on the water round a post: a soft ripple."""
    inner = [bm.verts.new(P(x + math.cos(math.tau * i / sides) * r0, y, z + math.sin(math.tau * i / sides) * r0)) for i in range(sides)]
    outer = [bm.verts.new(P(x + math.cos(math.tau * i / sides) * r1, y, z + math.sin(math.tau * i / sides) * r1)) for i in range(sides)]
    for i in range(sides):
        j = (i + 1) % sides
        bm.faces.new((inner[i], outer[i], outer[j], inner[j]))


def strut(bm, a, b, w, d, rng):
    """A flat brace from `a` to `b` (Godot points) on the front face."""
    mid = (a + b) * 0.5
    ang = math.atan2(b.y - a.y, b.x - a.x)
    length = (b - a).length
    box(bm, P(mid.x, mid.y, mid.z), Vector((length * 0.5, d * 0.5, w * 0.5)), rng, bevel=0.15, rot=Matrix.Rotation(-ang, 3, "Y"))


def lashing(bm, x, z, y, r):
    """Three turns of rope round a post at height `y`."""
    for k in range(3):
        res = bmesh.ops.create_cone(bm, cap_ends=False, segments=10, radius1=r * 1.08, radius2=r * 1.08, depth=0.07)
        for v in res["verts"]:
            v.co += P(x, y + (k - 1) * 0.09, z)


def bolts(bm, x0, x1, y, z, every=1.2):
    x = x0 + 0.3
    while x < x1 - 0.2:
        for dy in (-0.09, 0.09):
            res = bmesh.ops.create_uvsphere(bm, u_segments=6, v_segments=4, radius=0.055)
            for v in res["verts"]:
                v.co = Vector((v.co.x, v.co.y * 0.5, v.co.z)) + P(x, y + dy, z)
        x += every


def waterline(x, z, r, sides, rng):
    column(part("Water", "waterline"), x, z, WATER - 0.2, WATER + 0.55, r * 1.04, sides, rng)
    column(part("Weed", "weed"), x, z, WATER + 0.55, WATER + 0.7, r * 1.06, sides, rng)
    ring(part("Foam", "foam"), x, z, WATER + 0.06, r * 1.15, r * 1.75)


def build():
    rng = random.Random(29)
    span = RIGHT - LEFT
    if VARIANT == "A":
        n = max(3, int(round(span / 3.2)))
        xs = [LEFT + 0.35 + (span - 0.7) * k / n for k in range(n + 1)]
        posts = part("Posts", "post")
        for row, z in enumerate((FZ - 0.15, BACK_Z)):
            for k, x in enumerate(xs):
                r = rng.uniform(0.36, 0.44)
                column(posts, x, z, WATER - 1.5, DECK_UNDER - 0.02, r, 4, rng, lean=(rng.uniform(-0.01, 0.01), 0.0))
                waterline(x, z, r * 1.25, 4, rng)
        box(part("Beam", "beam"), P((LEFT + RIGHT) * 0.5, DECK_UNDER - 0.24, FZ + 0.1), Vector(((span + 0.3) * 0.5, 0.17, 0.22)), rng,
            bevel=0.12)
        bolts(part("Bolts", "iron"), LEFT, RIGHT, DECK_UNDER - 0.24, FZ + 0.27)
        # X bracing in every other bay, and a low walkway rail along the posts.
        br = part("Braces", "beam")
        y_hi, y_lo = DECK_UNDER - 0.6, WATER + 1.2
        for k in range(0, len(xs) - 1, 2):
            a, b = xs[k], xs[k + 1]
            strut(br, Vector((a, y_hi, FZ + 0.18)), Vector((b, y_lo, FZ + 0.18)), 0.26, 0.14, rng)
            strut(br, Vector((a, y_lo, FZ + 0.22)), Vector((b, y_hi, FZ + 0.22)), 0.26, 0.14, rng)
        box(br, P((LEFT + RIGHT) * 0.5, WATER + 1.0, FZ + 0.15), Vector(((span) * 0.5, 0.1, 0.13)), rng, bevel=0.15)
    elif VARIANT == "B":
        st = part("Quay", "harbour_stone")
        y = DECK_UNDER - 0.05
        course = 0
        while y > WATER - 0.6:
            h = rng.uniform(0.55, 0.75)
            x = LEFT + 0.1 - (rng.uniform(0.3, 0.9) if course % 2 else 0.0)
            while x < RIGHT - 0.1:
                w = rng.uniform(1.0, 1.8)
                x1 = min(x + w, RIGHT - 0.05)
                cx = (max(x, LEFT + 0.05) + x1) * 0.5
                box(st, P(cx, y - h * 0.5, FZ - 0.25), Vector(((x1 - max(x, LEFT + 0.05)) * 0.5 - 0.03, 0.4, h * 0.5 - 0.03)), rng, bevel=0.14)
                x = x1 + 0.04
            y -= h + 0.05
            course += 1
        n = max(3, int(round(span / 3.6)))
        posts = part("Fenders", "post")
        for k in range(n + 1):
            x = LEFT + 0.6 + (span - 1.2) * k / n
            r = 0.26
            column(posts, x, FZ + 0.25, WATER - 0.8, DECK_UNDER - 0.02, r, 8, rng)
            lashing(part("Rope", "rope"), x, FZ + 0.25, DECK_UNDER - 0.6, r)
            lashing(part("Rope", "rope"), x, FZ + 0.25, WATER + 1.4, r)
            ring(part("Foam", "foam"), x, FZ + 0.25, WATER + 0.06, r * 1.2, r * 2.0)
        box(part("Beam", "beam"), P((LEFT + RIGHT) * 0.5, DECK_UNDER - 0.16, FZ + 0.12), Vector(((span + 0.3) * 0.5, 0.16, 0.16)), rng,
            bevel=0.12)
    else:
        n = max(3, int(round(span / 4.4)))
        xs = [LEFT + 0.6 + (span - 1.2) * k / n for k in range(n + 1)]
        posts = part("Piles", "post")
        for row, z in enumerate((FZ - 0.2, BACK_Z)):
            for x in xs:
                for j, (dx, dz) in enumerate(((-0.42, 0.0), (0.42, 0.0), (0.0, -0.4))):
                    r = rng.uniform(0.26, 0.32)
                    lean = (-dx * 0.03, dz * 0.03)
                    column(posts, x + dx, z + dz, WATER - 1.5, DECK_UNDER - 0.02 - rng.uniform(0.0, 0.1), r, 9, rng, lean=lean)
                    waterline(x + dx, z + dz, r, 9, rng)
                for yy in (DECK_UNDER - 0.7, WATER + 1.5):
                    res = bmesh.ops.create_cone(part("Rope", "rope"), cap_ends=False, segments=12, radius1=0.78, radius2=0.78, depth=0.22)
                    for v in res["verts"]:
                        v.co = Vector((v.co.x, v.co.y * 0.75, v.co.z)) + P(x, yy, z - 0.12)
        box(part("Beam", "beam"), P((LEFT + RIGHT) * 0.5, DECK_UNDER - 0.26, FZ + 0.12), Vector(((span + 0.3) * 0.5, 0.2, 0.26)), rng,
            bevel=0.15)


def clamp(bm, margin=0.9):
    for v in bm.verts:
        v.co.x = min(max(v.co.x, LEFT - margin), RIGHT + margin)
        v.co.z = min(v.co.z, TOP - 0.3)


def make_objects():
    for (name, mat), bm in PARTS.items():
        if not bm.faces:
            bm.free()
            continue
        clamp(bm)
        me = bpy.data.meshes.new(name + "_" + mat)
        bm.normal_update()
        bm.to_mesh(me)
        bm.free()
        for p in me.polygons:
            p.use_smooth = mat in ("rope",)
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new(name + "_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def bake_occlusion():
    mock = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.join(opts["dir"], "scene_in.glb"), merge_vertices=True)
    # (Only the deck and its joists shade the front: the old piles, fascia and braces it replaces would bake into it.)
    keep = {e["name"] for e in manifest["meshes"] if e["aabb"][0] > 20.0 and e["aabb"][1] < 0.5 and e["aabb"][2] > 5.0}
    context = [o for o in bpy.context.scene.objects if o not in before]
    for o in context:
        if o.name not in keep:
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
        col = o.data.color_attributes["Col"].data
        for v in o.data.vertices:
            k = (tone.data[v.index].value or 1.0) if tone is not None else 1.0
            c = col[v.index].color
            # (The foam stays white: occlusion under the deck would grey it.)
            if o.name.endswith("_foam"):
                c = (1.0, 1.0, 1.0, 1.0)
            # (Half strength: under the closed deck every post baked nearly black; the reference's posts read in open air.)
            ao = 1.0 - (1.0 - c[0]) * 0.5
            col[v.index].color = (ao * k, ao * k, ao * k, 1.0)
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
    print("beach front %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
