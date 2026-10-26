"""The canyon's stage front as one modelled piece: the mesa the fighters stand on (the stage refinement, after the grove, meadow and
beach; docs/art/stages/BAKE_PILOT.md).

Built on the canyon's block from the bake manifest: from under the sand top (which stays code-built) down past the block's foot to the
desert floor (-16, `StageArt.DESERT_FLOOR`), the column widening a little as it falls, as the code-built mesa did. Three looks for the
owner to pick from (`--front-mock=` on the stage demo), each in the canyon's own strata colours (sandstone, deep brick, a pale band):
  A  stacked slabs: the mesa as tiers of chunky rock, each its own irregular footprint jutting in and out, as the far mesas are built;
  B  fluted cliff: deep vertical erosion columns down the face, cut across by the strata bands;
  C  stepped ledges: thin shelves stepping out and back, each with a lit top edge, boulders fallen at the foot.
After fidelity review 6 (at the wide camera the face read soft and smudged: big flat slabs whose occlusion, sampled at few vertices,
smeared into blotches), on A:
  `--cut 1`      crisp cut edges: each slab's top front edge chamfered back so it catches the light as a line, and a finer mesh so the
                 baked shade follows the shapes;
  `--courses 1`  with it, each slab laid in thin courses of sediment, each a little in or out and a step lighter or darker.
Families `strata_a`, `strata_b`, `strata_c`; each piece bakes its occlusion into linear vertex colours with the built canyon round it.

Shipped (fidelity review 9, the owner's pick C, 2026-10-25): godot/models/stages/canyon_front.glb, `--variant A --cut 1 --courses 1
--straight C` (review 8's `--straight A`, review 6's without `--straight`); with `--mass A` since fidelity review 11 (2026-10-26),
then lit (the palette's `front_lift` raised to 0.35 to keep the face at 0.33).

Run (Blender 5.2):
  blender --background --python art/blender/make_canyon_front.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
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
CUT = opts.get("cut", "0") == "1"
COURSES = opts.get("courses", "0") == "1"
# Straight courses (fidelity review 8: at the wide camera every course bulged with its slab's jutting front, wobbly plywood): `--straight
# A|B`. A: each course a run of flat-fronted blocks of uneven length, clean vertical joints, each block a little in or out. B: A with
# every third course set further back, so the face reads in thick and thin beds with shadow lines between. C (fidelity review 9: even
# beds, lighter and darker by turns, stacked like planks at the wide camera): beds of uneven thickness, thin partings between thick
# beds, longer, the tone stepping less.
STRAIGHT = opts.get("straight", "")
# Jointed rock (fidelity review 11: even with uneven beds the face read as stacked boards, each block its own length): `--mass A|B`. A: on
# the straight courses, vertical joints shared by every bed of a tier (a few beds let one through), each column a little in or out as a
# whole, and the face darkening toward its foot. B: A with some columns stained darker down their length, as weathered cliffs streak.
MASS = opts.get("mass", "").upper()
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
BLOCK = [r for r in manifest["rects"] if r[4] < 0.5][0]
LEFT, RIGHT, TOP, BOTTOM = BLOCK[0], BLOCK[1], BLOCK[2], BLOCK[3]
FZ = float(manifest["front_z"])
FLOOR = -16.0
Y_TOP = TOP - 0.45          # under the sand slab
DEPTH = 4.0                 # how far back the modelled faces reach (the camera sees the front and a little of the sides)
BANDS = ["strata_a", "strata_b", "strata_a", "strata_c"]


def P(x, y, z):
    return Vector((x, -z, y))


def half_width(y):
    """The column's half-width at `y`: the block's, narrowing a touch under it and then widening toward the floor."""
    w = (RIGHT - LEFT) * 0.5
    if y >= BOTTOM:
        return w
    t = (BOTTOM - y) / (BOTTOM - FLOOR)
    return w * (0.92 + 0.12 * t)


PARTS = {}


def part(mat):
    if mat not in PARTS:
        PARTS[mat] = bmesh.new()
        PARTS[mat].verts.layers.float.new("tone")
    return PARTS[mat]


def band_of(y, height=1.6):
    k = int(math.floor((TOP - y) / height))
    return BANDS[k % len(BANDS)]


def tone(bm, verts, t):
    layer = bm.verts.layers.float.get("tone")
    for v in verts:
        v[layer] = t


def front_profile(x0, x1, out, rng, cols):
    pts = []
    for i in range(cols + 1):
        x = x0 + (x1 - x0) * i / cols
        z = FZ + (rng.uniform(-0.25, 1.0) * out if 0 < i < cols else rng.uniform(-0.4, 0.1))
        pts.append((x, z))
    return pts


def slab(bm, x0, x1, y0, y1, out, rng, cols=10, taper=0.08, back=DEPTH, pts=None, chamfer=0.0, shade=None):
    """A tier of rock from `y0` up to `y1`: an irregular footprint whose front edge juts in and out (`out` at most) across x0..x1, its top
    a touch smaller; flat faces. `pts` its front edge, given; `chamfer` its top front edge cut back that far, a face catching the light."""
    if pts is None:
        pts = front_profile(x0, x1, out, rng, cols)
    lo, hi, tops = [], [], []
    ring = pts + [(x1, FZ - back), (x0, FZ - back)]
    cx = (x0 + x1) * 0.5
    ch = min(chamfer, (y1 - y0) * 0.4)
    for x, z in ring:
        lo.append(bm.verts.new(P(x, y0, z)))
        tx = cx + (x - cx) * (1.0 - taper * 0.3)
        front = z > FZ - back + 0.01
        tz = z - taper * (1.0 if front else 0.0)
        if ch > 0.0:
            tops.append(bm.verts.new(P(tx, y1 - ch, tz)))
            hi.append(bm.verts.new(P(tx, y1, tz - (ch * 1.3 if front else 0.0))))
        else:
            hi.append(bm.verts.new(P(tx, y1, tz)))
    n = len(ring)
    faces = []
    mid = tops if ch > 0.0 else hi
    for i in range(n):
        j = (i + 1) % n
        faces.append(bm.faces.new((lo[i], lo[j], mid[j], mid[i])))
        if ch > 0.0:
            faces.append(bm.faces.new((tops[i], tops[j], hi[j], hi[i])))
    faces.append(bm.faces.new(list(reversed(lo))))
    faces.append(bm.faces.new(hi))
    bm.normal_update()
    centre = P(cx, (y0 + y1) * 0.5, FZ - back * 0.5)
    for f in faces:
        if f.normal.dot(f.calc_center_median() - centre) < 0:
            f.normal_flip()
    tone(bm, lo + hi + tops, shade if shade is not None else rng.uniform(0.82, 1.05))


def courses_straight(bm, x0, x1, y0, y1, out, rng):
    """`courses` with straight, flat-fronted blocks (fidelity review 8): long beds with few joints, as sandstone lies (blocks of 1.4 to 3.6
    read as a brick wall)."""
    n = max(1, int(round((y1 - y0) / 0.75)))
    base = rng.uniform(0.84, 1.02)
    face = FZ + rng.uniform(0.15, 0.45) * out
    # (C, fidelity review 9: even beds 0.75 thick, a step lighter and darker by turns, read as stacked planks at the wide camera. Beds of
    # uneven thickness, thin partings between thick ones, longer, the tone stepping less.)
    cuts = [y1 - (y1 - y0) * k / n for k in range(n + 1)]
    if STRAIGHT == "C":
        hs = []
        while sum(hs) < y1 - y0:
            hs.append(rng.choice([rng.uniform(0.3, 0.5), rng.uniform(0.9, 1.7), rng.uniform(1.0, 2.2)]))
        scale = (y1 - y0) / sum(hs)
        cuts = [y1]
        for h in hs:
            cuts.append(cuts[-1] - h * scale)
        cuts[-1] = y0
        n = len(hs)
    if MASS:
        # The tier's joints, the same through every bed, and each column's own set and stain.
        # (The face's own joints, top to bottom, `JOINTS`: drawn per tier and 2.4 to 5 apart they read as a brick wall.)
        inner = [(jx, js, jt) for (jx, js, jt) in JOINTS if x0 + 1.2 < jx < x1 - 1.2]
        joints = [x0] + [jx for (jx, _, _) in inner] + [x1]
        left = max([j for j in JOINTS if j[0] <= x0 + 1.2], default=JOINTS[0])
        sets = [left[1]] + [js for (_, js, _) in inner] + [0.0]
        stains = [left[2]] + [jt for (_, _, jt) in inner] + [1.0]
        for k in range(n):
            ya, yb = cuts[k + 1], cuts[k]
            step = rng.uniform(0.95, 1.05)
            # (One bed in four lets a joint through, so the columns read as rock, not a grid.)
            skip = rng.randrange(1, len(joints) - 1) if (len(joints) > 2 and rng.random() < 0.15) else -1
            j = 0
            while j < len(joints) - 1:
                xa = joints[j]
                jn = j + 2 if j + 1 == skip else j + 1
                xb = joints[jn]
                dz = sets[j] + rng.uniform(-0.03, 0.03)
                slab(bm, xa + 0.06, xb - 0.06, ya + 0.015, yb, out, rng, pts=[(xa + 0.06, face + dz), (xb - 0.06, face + dz)], taper=0.02,
                     chamfer=0.07, shade=base * step * stains[j] * rng.uniform(0.97, 1.03))
                j = jn
        return
    for k in range(n):
        ya, yb = cuts[k + 1], cuts[k]
        recess = -0.32 if (STRAIGHT == "B" and k % 3 == 2) else 0.0
        step = (1.06 if k % 2 == 0 else 0.94) if STRAIGHT != "C" else rng.uniform(0.95, 1.05)
        x = x0
        while x < x1 - 0.05:
            xe = min(x1, x + (rng.uniform(6.0, 14.0) if STRAIGHT == "C" else rng.uniform(4.0, 9.0)))
            if x1 - xe < 2.0:
                xe = x1
            dz = rng.uniform(-0.12, 0.08) + recess
            pts = [(x, face + dz), (xe, face + dz)]
            slab(bm, x + 0.02, xe - 0.02, ya + 0.015, yb, out, rng, pts=[(x + 0.02, face + dz), (xe - 0.02, face + dz)], taper=0.02,
                 chamfer=0.07, shade=base * step * rng.uniform(0.95, 1.05) * (0.86 if recess else 1.0))
            x = xe


def courses(bm, x0, x1, y0, y1, out, rng, cols=6):
    """A slab laid in thin courses of sediment (fidelity review 6): one front edge for the slab, each course a little in or out of it,
    alternately a step lighter and darker round the slab's own tone (darker only, the face fell from 0.33 to 0.30, undoing review 5's
    lift), each with its own cut edge."""
    pts = front_profile(x0, x1, out, rng, cols)
    n = max(2, int(round((y1 - y0) / 0.5)))
    base = rng.uniform(0.84, 1.02)
    for k in range(n):
        ya = y1 - (y1 - y0) * (k + 1) / n
        yb = y1 - (y1 - y0) * k / n
        dz = rng.uniform(-0.18, 0.1)
        own = [(x, z + dz * (1.0 if 0 < i < len(pts) - 1 else 0.3)) for i, (x, z) in enumerate(pts)]
        slab(bm, x0, x1, ya + 0.015, yb, out, rng, pts=own, taper=0.02, chamfer=0.07,
             shade=base * (1.06 if k % 2 == 0 else 0.94) * rng.uniform(0.97, 1.03))


def core():
    """A plain core behind the face, so no gap between tiers shows the sky (the code-built block it replaces is hidden)."""
    bm = part("strata_b")
    y0, y1 = FLOOR - 0.5, Y_TOP - 0.05
    w0, w1 = half_width(y0) - 0.2, half_width(y1) - 0.2
    zf, zb = FZ - 0.5, FZ - DEPTH + 0.2
    vs = [bm.verts.new(P(x, y, z)) for (x, y, z) in ((-w0, y0, zf), (w0, y0, zf), (w1, y1, zf), (-w1, y1, zf),
                                                     (-w0, y0, zb), (w0, y0, zb), (w1, y1, zb), (-w1, y1, zb))]
    centre = P(0.0, (y0 + y1) * 0.5, (zf + zb) * 0.5)
    for f in ((0, 1, 2, 3), (1, 5, 6, 2), (4, 0, 3, 7)):
        face = bm.faces.new([vs[i] for i in f])
        face.normal_update()
        # (Facing out, or it is culled and the sky shows through the gap it fills.)
        if face.normal.dot(face.calc_center_median() - centre) < 0:
            face.normal_flip()
    tone(bm, vs, 0.7)


JOINTS = []


def build():
    rng = random.Random(41)
    core()
    if MASS:
        # The vertical joints of the whole face, 5 to 11 apart, each column's set (in or out) and its stain.
        jr = random.Random(57)
        x = LEFT - 4.0
        while x < RIGHT + 4.0:
            JOINTS.append((x, jr.uniform(-0.32, 0.1), jr.uniform(0.8, 0.9) if (MASS == "B" and jr.random() < 0.4) else 1.0))
            x += jr.uniform(5.0, 11.0)
    if VARIANT == "A":
        y = Y_TOP
        while y > FLOOR - 0.5:
            h = rng.uniform(1.1, 2.0)
            w = half_width(y - h * 0.5)
            # Two or three slabs side by side in each tier, so the face breaks into blocks, not one ribbon.
            cuts = sorted(rng.uniform(-w * 0.6, w * 0.6) for _ in range(rng.choice((1, 2))))
            xs = [-w] + cuts + [w]
            if MASS:
                # (One run per tier: the joints break it, not the tier's own cuts.)
                xs = [-w, w]
            for k in range(len(xs) - 1):
                if COURSES and STRAIGHT:
                    courses_straight(part(band_of(y - h * 0.5)), xs[k] + 0.04, xs[k + 1] - 0.04, y - h, y, rng.uniform(0.35, 0.9), rng)
                elif COURSES:
                    courses(part(band_of(y - h * 0.5)), xs[k] + 0.04, xs[k + 1] - 0.04, y - h, y, rng.uniform(0.35, 0.9), rng)
                else:
                    slab(part(band_of(y - h * 0.5)), xs[k] + 0.04, xs[k + 1] - 0.04, y - h, y, rng.uniform(0.35, 0.9), rng, cols=6,
                         chamfer=0.22 if CUT else 0.0)
            y -= h + 0.02
    elif VARIANT == "B":
        # Columns: a face of vertical flutes, cut across into band-coloured rows.
        y = Y_TOP
        while y > FLOOR - 0.5:
            h = 1.6
            w = half_width(y - h * 0.5)
            flutes = int(w * 2 / 1.1)
            for k in range(flutes):
                x0 = -w + (2 * w) * k / flutes
                x1 = -w + (2 * w) * (k + 1) / flutes
                own = random.Random(k * 31 + 7)
                depth = 0.25 + 0.65 * own.random()
                slab(part(band_of(y - h * 0.5)), x0 + 0.02, x1 - 0.02, y - h, y, depth, rng, cols=2, taper=0.02)
            y -= h
    else:
        y = Y_TOP
        k = 0
        while y > FLOOR - 0.5:
            h = rng.uniform(0.7, 1.2)
            w = half_width(y - h * 0.5)
            jut = 0.9 if k % 2 == 0 else 0.15
            slab(part(band_of(y - h * 0.5, 1.0)), -w, w, y - h, y, jut, rng, cols=14, taper=0.25)
            y -= h
            k += 1
        bm = part("strata_b")
        for k in range(14):
            x = rng.uniform(LEFT - 1.0, RIGHT + 1.0)
            s = rng.uniform(0.6, 1.4)
            res = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=s)
            for v in res["verts"]:
                v.co = Vector((v.co.x * 1.3, v.co.y, v.co.z * 0.75)) + P(x, FLOOR + s * 0.35, FZ + rng.uniform(0.3, 2.0))
            tone(bm, res["verts"], rng.uniform(0.85, 1.05))


def darken_down():
    """`--mass`: the face a step deeper toward its foot, as a cliff darkens down into its own shade (each vertex's tone)."""
    for bm in PARTS.values():
        layer = bm.verts.layers.float.get("tone")
        for v in bm.verts:
            t = min(1.0, max(0.0, (Y_TOP - v.co.z) / (Y_TOP - FLOOR)))
            v[layer] = v[layer] * (1.04 - 0.26 * t ** 1.2)


def clamp(bm, margin=0.9):
    for v in bm.verts:
        y = min(max(v.co.z, BOTTOM), TOP)
        v.co.x = min(max(v.co.x, LEFT - margin), RIGHT + margin)
        v.co.z = min(v.co.z, TOP - 0.3)
        v.co.y = max(v.co.y, -(FZ + 2.5))


def make_objects():
    for mat, bm in PARTS.items():
        clamp(bm)
        # (Finer, so the occlusion baked into the vertices follows the shapes: on big faces it sampled only their corners and smeared.)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        for _ in range(2):
            # (Finer with the cut edges, review 6: at 0.8 the shade still smeared across the slabs into blotches.)
            long_edges = [e for e in bm.edges if e.calc_length() > (0.6 if CUT else 0.8)]
            if long_edges:
                bmesh.ops.subdivide_edges(bm, edges=long_edges, cuts=1, use_grid_fill=True)
            bmesh.ops.triangulate(bm, faces=bm.faces[:])
        me = bpy.data.meshes.new("Mesa_" + mat)
        bm.normal_update()
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new("Mesa_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def bake_occlusion():
    mock = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    gone = {e["name"] for e in manifest["meshes"] if e["path"].split("/")[-1].startswith(("block", "strata_"))}
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
    scene.world.light_settings.distance = 1.6
    bpy.ops.object.select_all(action="DESELECT")
    for o in mock:
        a = o.data.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
        o.data.color_attributes.active_color = a
        o.data.color_attributes.render_color_index = o.data.color_attributes.active_color_index
        o.select_set(True)
    bpy.context.view_layer.objects.active = mock[0]
    bpy.ops.object.bake(type="AO", target="VERTEX_COLORS")
    for o in mock:
        t = o.data.attributes.get("tone")
        col = o.data.color_attributes["Col"].data
        for v in o.data.vertices:
            k = (t.data[v.index].value or 1.0) if t is not None else 1.0
            c = col[v.index].color
            col[v.index].color = (c[0] * k, c[1] * k, c[2] * k, 1.0)
    for o in context:
        bpy.data.objects.remove(o, do_unlink=True)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build()
    if MASS:
        darken_down()
    make_objects()
    bake_occlusion()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("canyon front %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
