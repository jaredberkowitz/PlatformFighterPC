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

Shipped since fidelity review 7 (the owner's picks, 2026-10-22): `--variant B --plat timber --wear 2` into meadow_play.glb, then lit.

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
# The platforms after fidelity review 6 (`--plat timber|timber_moss`; on the shipped B's lawn).
PLAT = opts.get("plat", "")
WEAR = int(opts.get("wear", "0"))
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


def solid(side, bottom, top4, bot4, top=None):
    """A closed frustum from a top quad to a bottom quad (corners in the same order round), its faces turned outward from its middle."""
    pts = top4 + bot4
    c = sum(pts, Vector()) / len(pts)
    faces = [(top or side, list(reversed(top4))), (bottom, bot4)]
    for i in range(4):
        j = (i + 1) % 4
        faces.append((side, [top4[i], top4[j], bot4[j], bot4[i]]))
    for mat, q in faces:
        f = face(mat, q)
        f.normal_update()
        if f.normal.dot(f.calc_center_median() - c) < 0.0:
            f.normal_flip()


def log_x(x0, x1, yc, zc, r, side, end, n=9):
    """A round log lying along x, its cut ends showing."""
    import math as _m
    ring = [(_m.cos(_m.tau * k / n) * r, _m.sin(_m.tau * k / n) * r) for k in range(n)]
    a = [P(x0, yc + dy, zc + dz) for (dy, dz) in ring]
    b = [P(x1, yc + dy, zc + dz) for (dy, dz) in ring]
    for k in range(n):
        j = (k + 1) % n
        f = face(side, [a[k], a[j], b[j], b[k]])
        f.normal_update()
        mid = P((x0 + x1) * 0.5, yc, zc)
        if f.normal.dot(f.calc_center_median() - mid) < 0.0:
            f.normal_flip()
    for pts, sgn in ((a, -1.0), (b, 1.0)):
        f = face(end, list(pts))
        f.normal_update()
        if f.normal.x * sgn < 0.0:
            f.normal_flip()


def underside(x0, x1, y_top, zb, zf, depth, rng, side, bottom):
    """A rocky underside hanging under a platform: cells along it, deepest toward the middle, each set in a little and faceted."""
    import math as _m
    n = max(3, int((x1 - x0) / 0.9))
    w = (x1 - x0) / n
    for i in range(n):
        xa, xb = x0 + w * i, x0 + w * (i + 1)
        t = (i + 0.5) / n
        d = depth * (0.3 + 0.7 * _m.sin(_m.pi * t)) * rng.uniform(0.75, 1.15)
        ix = w * rng.uniform(0.08, 0.2)
        izf, izb = rng.uniform(0.35, 0.7), rng.uniform(0.4, 0.8)
        top4 = [P(xa, y_top, zb), P(xb, y_top, zb), P(xb, y_top, zf), P(xa, y_top, zf)]
        yb = y_top - d
        bot4 = [P(xa + ix, yb, zb + izb), P(xb - ix, yb, zb + izb), P(xb - ix, yb + rng.uniform(0.0, 0.12), zf - izf),
                P(xa + ix, yb + rng.uniform(0.0, 0.12), zf - izf)]
        solid(side, bottom, top4, bot4)


def hanging(x0, x1, y_top, z, mat, rng, count, lo, hi, wide=(0.15, 0.35)):
    """Strands hanging from y_top in front of the face at z (moss, grass, roots): tapering, a little crooked."""
    for _ in range(count):
        x = rng.uniform(x0, x1)
        ln = rng.uniform(lo, hi)
        wd = rng.uniform(*wide)
        lean = rng.uniform(-0.15, 0.15)
        face(mat, (P(x - wd, y_top, z), P(x + wd, y_top, z), P(x + wd * 0.35 + lean, y_top - ln * 0.7, z), P(x + lean * 1.4, y_top - ln, z),
                   P(x - wd * 0.4 + lean, y_top - ln * 0.6, z)))


# ---- The play tops' wear (fidelity review 7: the most-seen surfaces were plain; `--wear 1|2`) -----------------------------------------
# Flat pieces a hair over the walkable top (never more than 0.02: stage_front_test lets nothing poke up where fighters stand), and with
# `--wear 2` tufts or pebbles along the front edge, which the test leaves out (the front 0.3).

def patch_flat(fam, cx, cz, rx, rz, y, rng, n=14):
    """A flat, hand-cut blob lying on the top at height y, facing up."""
    import math as _m
    ph = rng.uniform(0, _m.tau)
    pts = []
    for i in range(n):
        a = _m.tau * i / n
        k = 1.0 + 0.22 * _m.sin(a * 3.0 + ph) + rng.uniform(-0.1, 0.1)
        pts.append(P(cx + _m.cos(a) * rx * k, y, cz + _m.sin(a) * rz * k))
    f = face(fam, list(reversed(pts)))
    f.normal_update()
    if f.normal.z < 0.0:
        f.normal_flip()


def wear_top(x0, x1, top, zb, zf, rng, kinds, path_z=None, path_half=0.7):
    """Patches over one top: `kinds` [(family, count, size range)], kept clear of the very edges; with a path, worn earth along its sides."""
    for fam, count, (lo, hi) in kinds:
        for _ in range(count):
            rx = rng.uniform(lo, hi)
            rz = rx * rng.uniform(0.35, 0.6)
            cx = rng.uniform(x0 + rx, x1 - rx)
            cz = rng.uniform(zb + 0.6 + rz, zf - 0.6 - rz)
            # (Off the path: laid over it, they broke it into pieces.)
            if path_z is not None and abs(cz - path_z) < path_half + rz * 1.3 + 0.1:
                continue
            patch_flat(fam, cx, cz, rx, rz, top + 0.008 + 0.002 * rng.random(), rng)
    if path_z is not None:
        # Trodden earth where the path frays: short worn tongues along both its sides.
        x = x0 + rng.uniform(0.3, 1.2)
        while x < x1 - 0.6:
            side = rng.choice((-1.0, 1.0))
            rx = rng.uniform(0.35, 0.9)
            patch_flat("top_worn", x, path_z + side * (path_half + 0.12), rx, rng.uniform(0.12, 0.22), top + 0.012, rng, n=10)
            x += rng.uniform(1.2, 2.6)


def edge_tufts(x0, x1, top, zf, rng, fam="fringe", every=(0.35, 0.8), tall=(0.18, 0.34)):
    """Grass tufts along the front edge (in the strip the stage test leaves out), a few blades each, leaning out over the edge."""
    x = x0 + rng.uniform(0.1, 0.4)
    while x < x1 - 0.1:
        z = zf - rng.uniform(0.04, 0.22)
        for b in range(rng.randrange(2, 5)):
            bx = x + rng.uniform(-0.12, 0.12)
            h = rng.uniform(*tall)
            lean = rng.uniform(-0.1, 0.1)
            face(fam, (P(bx - 0.05, top, z), P(bx + 0.05, top, z), P(bx + lean, top + h, z + 0.08)))
        x += rng.uniform(*every)


def edge_pebbles(x0, x1, top, zf, rng, fam="top_deep"):
    """Pebbles along the front edge (in the strip the stage test leaves out), low and flattened."""
    import bmesh as _b
    x = x0 + rng.uniform(0.2, 0.6)
    while x < x1 - 0.2:
        r = rng.uniform(0.07, 0.16)
        res = _b.ops.create_icosphere(part(fam), subdivisions=1, radius=r)
        for v in res["verts"]:
            v.co.z *= 0.55
            v.co += P(x, top + r * 0.3, zf - rng.uniform(0.05, 0.2))
        x += rng.uniform(0.4, 1.3)


def board_marks(x0, x1, top, zb, zf, n, rng):
    """Nail heads in pairs where the boards cross the joists, flush with the deck."""
    gap = 0.06
    bw = (zf - zb - gap * (n - 1)) / n
    for k in range(n):
        za = zb + k * (bw + gap)
        x = x0 + 0.35
        while x < x1 - 0.2:
            for dz in (0.28, bw - 0.28):
                face("nail", (P(x - 0.035, top + 0.006, za + dz - 0.035), P(x - 0.035, top + 0.006, za + dz + 0.035),
                              P(x + 0.035, top + 0.006, za + dz + 0.035), P(x + 0.035, top + 0.006, za + dz - 0.035)))
            x += 2.4


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
    if WEAR:
        # (Its own random numbers, so the platforms and everything after keep their layout.)
        wr = random.Random(91)
        wear_top(x0 + 0.3, x1 - 0.3, top, zb, zf, wr, [("top_deep", 22, (0.6, 1.8)), ("top_lit", 16, (0.7, 1.9)), ("top_worn", 4, (0.4, 0.9))],
                 path_z=FZ - 1.9, path_half=0.7)
        if WEAR >= 2:
            edge_tufts(x0, x1, top, zf, wr)


def platform(r, rng):
    x0, x1, top = r[0], r[1], r[2]
    zb, zf = FZ - PLAT_DEPTH, FZ
    if PLAT in ("timber", "timber_moss"):
        # (Fidelity review 6: a green slab with a scalloped trim read as a plain slab. A turf mat on two round logs, their cut ends
        # showing and lashed with rope near each end, the farm's own build; `timber_moss` with moss and grass hanging over the logs.)
        box(x0 - 0.05, x1 + 0.05, top - 0.16, top, zb, zf, "lawn", "card", "lawn_under")
        r_log = 0.24
        yc = top - 0.16 - r_log
        for zc in (zf - 0.32, zb + 0.32):
            log_x(x0 - 0.3, x1 + 0.3, yc, zc, r_log, "plank", "plank_card")
            for bx in (x0 + 0.35, x1 - 0.35):
                box(bx - 0.07, bx + 0.07, yc - r_log - 0.03, yc + r_log + 0.03, zc - r_log - 0.03, zc + r_log + 0.03, "rope", "rope")
        # A tufted lip of grass along the front, uneven, in place of the even scallops.
        x = x0 - 0.05
        while x < x1 + 0.05:
            wd = rng.uniform(0.14, 0.32)
            dn = rng.uniform(0.1, 0.32)
            face("fringe", (P(x, top - 0.02, zf + 0.04), P(x + wd, top - 0.02, zf + 0.04), P(x + wd * 0.55, top - 0.16 - dn, zf + 0.04)))
            x += wd * rng.uniform(0.6, 0.9)
        if PLAT == "timber_moss":
            hanging(x0, x1, top - 0.14, zf + 0.06, "fringe", rng, int((x1 - x0) * 1.6), 0.25, 0.6)
            hanging(x0, x1, yc + r_log * 0.6, zf - 0.32 + r_log + 0.02, "lawn_under", rng, int((x1 - x0) * 0.8), 0.15, 0.35, (0.1, 0.22))
        return
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
