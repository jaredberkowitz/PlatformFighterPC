"""The grove's stage front as one modelled piece (the stage refinement's grove pilot, step 2; docs/art/stages/BAKE_PILOT.md).

Built on the stage's own collision, read from the bake manifest (`StageBake.export_scene` writes the blocks' rectangles and the front
plane), so the island's outline stays within the blocks' and the walkable top is untouched (the front starts 0.3 under it). Three
looks for the owner to pick from, rendered in the game from the match camera (`--front-mock=` on the stage demo):
  A  the quilted bank modelled: deep puffed pillows with pressed creases and buttons, a rolled lip of moss tufts, a quilted keel;
  B  roots and rocks after the grove reference: packed faceted stones on dark earth, ribbed roots wrapping down from under the lip,
     moss on the stones and the lip, roots dangling under the island;
  C  both: the quilt above, roots wrapping it lower down and gathering into a rocky keel.
Material names are families (`quilt`, `button`, `earth`, `rock`, `root`, `moss`) that the game paints from the stage's palette.

Run (Blender 5.2):
  blender --background --python art/blender/make_grove_front.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
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
SOLIDS = [r for r in manifest["rects"] if r[4] < 0.5]
FZ = float(manifest["front_z"])
TOP = max(r[2] for r in SOLIDS)
LOW = min(r[3] for r in SOLIDS)
ROUND = 4.0          # how far under the lowest block the island rounds off to its keel
Y_TOP = TOP - 0.3
Y_BOT = LOW - ROUND
BEND = 1.3           # how far in from each side the front curves back
# Where the front meets the lawn: just under the lawn's slab (`solid_block`, 0.55 deep). (The thick rolled turf lip that hung 2.1 down
# is left out over a modelled front, the owner, 2026-10-20.)
CURL = TOP - 0.6


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


def span(y):
    lo, hi = math.inf, -math.inf
    for r in SOLIDS:
        if r[3] - 0.001 <= y <= r[2] + 0.001:
            lo, hi = min(lo, r[0]), max(hi, r[1])
    return lo, hi


def edges(y):
    """The island's left and right at height `y`: the blocks' spans eased over a window (as the code-built quilt), rounded under."""
    sl, sr, n = 0.0, 0.0, 0
    for k in range(9):
        yy = min(max(y + (-1.1 + 2.2 * k / 8.0), LOW + 0.01), TOP - 0.01)
        lo, hi = span(yy)
        if not math.isinf(lo):
            sl, sr, n = sl + lo, sr + hi, n + 1
    lo, hi = sl / max(n, 1), sr / max(n, 1)
    if y < LOW:
        f = math.sqrt(max(0.0, 1.0 - ((LOW - y) / ROUND) ** 2))
        mid = (lo + hi) * 0.5
        lo, hi = mid - (mid - lo) * f, mid + (hi - mid) * f
    return lo, hi


def base_z(x, y):
    lo, hi = edges(y)
    to_edge = min(x - lo, hi - x)
    z = FZ + 0.02
    if to_edge < BEND:
        q = (BEND - to_edge) / BEND
        z -= BEND * (1.0 - math.sqrt(max(0.0, 1.0 - q * q)))
    return z


QUILT = 1.9


def quilt_d(x, y):
    u, v = (x + y) / QUILT, (x - y) / QUILT
    return max(abs(u - round(u)), abs(v - round(v))) * 2.0


def quilt_puff(x, y):
    lo, hi = edges(y)
    to_edge = min(x - lo, hi - x)
    soft = min(1.0, max(0.0, to_edge / 0.6)) * min(1.0, max(0.0, (Y_TOP - y) / 0.25))
    d = quilt_d(x, y)
    # (Deeper than the code-built quilt, 0.55 against 0.32, and creased: the pillow rounds over and drops sharply into the seam.)
    return 0.55 * max(0.0, 1.0 - d ** 2.4) ** 0.7 * soft


def sheet(bm, displace, y_top=Y_TOP, y_bot=Y_BOT, step=0.13, cols=150):
    rows = int(math.ceil((y_top - y_bot) / step))
    grid = []
    for j in range(rows + 1):
        y = y_top + (y_bot - y_top) * j / rows
        lo, hi = edges(y)
        row = []
        for i in range(cols + 1):
            x = lo + (hi - lo) * i / cols
            row.append(bm.verts.new(P(x, y, base_z(x, y) + displace(x, y))))
        grid.append(row)
    for j in range(rows):
        for i in range(cols):
            a, b, c, d = grid[j][i], grid[j][i + 1], grid[j + 1][i], grid[j + 1][i + 1]
            bm.faces.new((a, c, d, b))


def rock(bm, c, r, rng):
    """A blocky stone (the grove reference's packed stones: squared, chamfered, a few big planes each): a box, its corners pushed in
    or out, its edges bevelled, turned a little, flattened against the island."""
    res = bmesh.ops.create_cube(bm, size=2.0)
    verts = res["verts"]
    s = Vector((rng.uniform(0.9, 1.35), rng.uniform(0.5, 0.7), rng.uniform(0.65, 0.95))) * r
    rot = Matrix.Rotation(rng.uniform(-0.35, 0.35), 3, "Y") @ Matrix.Rotation(rng.uniform(-0.2, 0.2), 3, "X")
    for v in verts:
        v.co = rot @ Vector((v.co.x * s.x * rng.uniform(0.85, 1.1), v.co.y * s.y, v.co.z * s.z * rng.uniform(0.85, 1.1))) + c
    edges = list({e for v in verts for e in v.link_edges})
    out = bmesh.ops.bevel(bm, geom=edges, offset=r * 0.16, segments=1, affect="EDGES")
    # (Each stone its own tone, as packed field stones are; multiplied into its baked occlusion, `tone`.)
    layer = bm.verts.layers.float.get("tone")
    # (Its own random numbers from where it sits, so the stones' layout does not move with it.)
    tone = random.Random(round(c.x * 10.0) * 7919 + round(c.z * 10.0)).uniform(0.72, 1.05)
    for v in out["verts"]:
        if layer is not None and v.is_valid:
            v[layer] = tone


def tube(bm, pts, r0, r1, rng, sides=10, ribs=6, rib=0.2):
    """A ribbed root along `pts` (Blender points), its radius tapering from `r0` to `r1`, its ridges twisting a little as it goes."""
    rings = []
    twist = rng.uniform(0, math.tau)
    for k, p in enumerate(pts):
        t = k / (len(pts) - 1)
        fwd = (pts[min(k + 1, len(pts) - 1)] - pts[max(k - 1, 0)]).normalized()
        side = fwd.cross(Vector((0, -1, 0)))
        if side.length < 1e-4:
            side = Vector((1, 0, 0))
        side.normalize()
        up = side.cross(fwd).normalized()
        # (Thick most of the way and thinning toward the tip, a little swell where it bends.)
        rad = (r0 + (r1 - r0) * t ** 1.6) * (1.0 + 0.08 * math.sin(t * 9.0 + twist))
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            rr = rad * (1.0 + rib * math.cos(ribs * a + twist + t * 7.0))
            ring.append(bm.verts.new(p + (side * math.cos(a) + up * math.sin(a)) * rr))
        rings.append((p, ring))
    for k in range(len(rings) - 1):
        p0, a = rings[k]
        _, b = rings[k + 1]
        for i in range(sides):
            j = (i + 1) % sides
            f = bm.faces.new((a[i], a[j], b[j], b[i]))
            f.normal_update()
            if f.normal.dot(f.calc_center_median() - p0) < 0.0:
                f.normal_flip()
    tip = bm.verts.new(pts[-1] + (pts[-1] - pts[-2]).normalized() * r1 * 2.0)
    for i in range(sides):
        f = bm.faces.new((rings[-1][1][i], rings[-1][1][(i + 1) % sides], tip))
        f.normal_update()
        if f.normal.dot(f.calc_center_median() - rings[-1][0]) < 0.0:
            f.normal_flip()


def root_path(x0, y_end, rng, lift, follow, x1=None, sway=0.35, y0=None):
    """A root from `y0` (under the lip's curl) at `x0` down to `y_end`: as a share of the island's width it sweeps from `x0` toward
    `x1` (default: drifting toward the middle as the island narrows) with a slow S of `sway`, kept on the island and wrapping round its
    curved sides, `lift` in front of the surface under it (`follow(x, y)`)."""
    pts = []
    y0 = CURL + 0.2 if y0 is None else y0
    lo0, hi0 = edges(min(y0, Y_TOP))
    mid0, half0 = (lo0 + hi0) * 0.5, (hi0 - lo0) * 0.5
    k0 = (x0 - mid0) / half0
    k1 = k0 * 0.92 if x1 is None else (x1 - mid0) / half0
    phase = rng.uniform(0, math.tau)
    y = y0
    while y > y_end:
        t = (y0 - y) / max(y0 - y_end, 0.01)
        e = t * t * (3.0 - 2.0 * t)
        lo, hi = edges(max(y, Y_BOT + 0.2))
        mid, half = (lo + hi) * 0.5, (hi - lo) * 0.5
        x = mid + (k0 + (k1 - k0) * e) * half + sway * math.sin(t * 5.0 + phase)
        x = min(max(x, lo + 0.25), hi - 0.25)
        pts.append(P(x, y, base_z(x, y) + follow(x, y) + lift))
        y -= 0.25
    return pts


def lip_tufts(bm_lit, bm_shade, rng):
    """Moss tufts hanging over the lip where the lawn meets the front: folded domes turned over."""
    lo, hi = edges(Y_TOP)
    x = lo + 0.5
    while x < hi - 0.4:
        r = rng.uniform(0.5, 0.85)
        c = P(x, CURL - 0.05 + rng.uniform(-0.15, 0.1), base_z(x, CURL) + 0.7 + rng.uniform(-0.1, 0.15))
        n_lit, n_shade = len(bm_lit.verts), len(bm_shade.verts)
        folded_dome(bm_lit, bm_shade, tuple(c), r, 6, rng)
        flip = Matrix.Rotation(math.pi + rng.uniform(-0.25, 0.25), 3, "X") @ Matrix.Diagonal((1.15, 1.0, rng.uniform(0.5, 0.95)))
        for bm, n in ((bm_lit, n_lit), (bm_shade, n_shade)):
            bm.verts.ensure_lookup_table()
            for v in bm.verts[n:]:
                v.co = flip @ (v.co - c) + c
        x += r * rng.uniform(0.9, 1.3)


def clamp_outline(bm, margin=0.9):
    """Keeps every point under the walkable top and within `margin` of the blocks' exact outline at its height (the eased outline the sheet follows runs up to a
    unit wider at each step, and stones and roots near the sides went further), so no fighter passes in front of or behind what is not
    solid (tests/stage_front_test.gd)."""
    for v in bm.verts:
        y = min(max(v.co.z, LOW), TOP)
        lo, hi = span(y)
        v.co.x = min(max(v.co.x, lo - margin), hi + margin)
        # (And under the walkable top: with the turf lip gone the top stones rose over the lawn's front edge.)
        v.co.z = min(v.co.z, TOP - 0.3)


def obj(name, bm, mat_name, colour, flat=True):
    clamp_outline(bm)
    me = bpy.data.meshes.new(name)
    bm.normal_update()
    bm.to_mesh(me)
    bm.free()
    if not flat:
        for p in me.polygons:
            p.use_smooth = True
    mat = bpy.data.materials.get(mat_name) or bpy.data.materials.new(mat_name)
    mat.diffuse_color = (*colour, 1.0)
    me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def build():
    rng = random.Random(7)
    quilted = VARIANT in ("A", "C")
    if quilted:
        bm = bmesh.new()
        sheet(bm, quilt_puff)
        obj("Quilt", bm, "quilt", (0.88, 0.66, 0.42), flat=False)
        # Buttons pressed in where the seams cross.
        bm = bmesh.new()
        for a in range(-12, 13):
            for b in range(-12, 13):
                x, y = (a + b + 1) * QUILT * 0.5, (a - b) * QUILT * 0.5
                if not (Y_BOT + 1.0 < y < Y_TOP - 0.4):
                    continue
                lo, hi = edges(y)
                if min(x - lo, hi - x) < 0.6:
                    continue
                if VARIANT == "C" and y < LOW + 1.0:
                    continue
                res = bmesh.ops.create_uvsphere(bm, u_segments=8, v_segments=5, radius=0.12)
                for v in res["verts"]:
                    v.co = Vector((v.co.x, v.co.y * 0.5, v.co.z)) + P(x, y, base_z(x, y) + 0.04)
        obj("Buttons", bm, "button", (0.6, 0.42, 0.28))
    else:
        bm = bmesh.new()
        sheet(bm, lambda x, y: 0.0, step=0.4, cols=60)
        obj("Earth", bm, "earth", (0.32, 0.22, 0.16))
    lit, shade = bmesh.new(), bmesh.new()
    lip_tufts(lit, shade, rng)
    if VARIANT in ("B", "C"):
        # The stones: packed across the face (B) or gathered in the keel under the quilt (C).
        bm = bmesh.new()
        bm.verts.layers.float.new("tone")
        y = CURL - 0.4 if VARIANT == "B" else LOW + 0.6
        tops = []
        while y > Y_BOT + 0.4:
            lo, hi = edges(y)
            x = lo + rng.uniform(0.3, 0.9)
            while x < hi - 0.3:
                r = (rng.uniform(0.8, 1.6) if VARIANT == "B" else rng.uniform(0.45, 1.3)) * (0.8 if y < LOW else 1.0)
                c = P(x, y, base_z(x, y) + r * 0.3)
                rock(bm, c, r, rng)
                if rng.random() < 0.55:
                    tops.append((c, r))
                x += r * rng.uniform(1.55, 1.85)
            y -= rng.uniform(1.25, 1.6) if VARIANT == "B" else rng.uniform(0.95, 1.2)
        obj("Rocks", bm, "rock", (0.6, 0.56, 0.66))
        for c, r in tops:
            # (Moss on the stones' tops, the reference's soft green caps: wide and low.)
            n_lit, n_shade = len(lit.verts), len(shade.verts)
            top_c = c + Vector((rng.uniform(-0.2, 0.2) * r, -r * 0.25, r * 0.6))
            folded_dome(lit, shade, tuple(top_c), r * rng.uniform(0.65, 0.9), 7, rng)
            squash = Matrix.Diagonal((1.25, 1.0, 0.55))
            for bm_, n in ((lit, n_lit), (shade, n_shade)):
                bm_.verts.ensure_lookup_table()
                for v in bm_.verts[n:]:
                    v.co = squash @ (v.co - top_c) + top_c
        # The roots: from under the lip, wrapping down over the face, a few dangling under the island.
        bm = bmesh.new()
        if VARIANT == "B":
            # (A few thick roots sweeping across the stones and crossing, after the grove reference, in place of a row of thin parallel
            # ones; branches split off them; the side roots wrap round the island's curved flanks; some dangle under the keel.)
            def follow(x, y):
                return 0.75
            mains = [(-10.2, -2.0, 0.75, Y_BOT - 2.5), (-5.5, 3.0, 0.68, LOW - 1.0), (-0.8, -6.0, 0.62, Y_BOT - 1.5),
                     (4.2, 1.5, 0.72, Y_BOT - 3.0), (9.8, 3.5, 0.7, LOW + 0.5)]
            for x0, x1, r0, y_end in mains:
                pts = root_path(x0, y_end, rng, 0.25, follow, x1=x1, sway=0.9)
                if len(pts) > 2:
                    tube(bm, pts, r0, 0.12, rng)
                    # A branch from a third of the way down, sweeping the other way.
                    k = len(pts) // 3
                    bx, by = pts[k].x, pts[k].z
                    bend = 3.5 if x1 < x0 else -3.5
                    bpts = root_path(bx, by - rng.uniform(2.5, 4.0), rng, 0.15, follow, x1=bx + bend, sway=0.4, y0=by)
                    if len(bpts) > 2:
                        tube(bm, bpts, r0 * 0.45, 0.06, rng, sides=8)
        else:
            follow = quilt_puff
            for x0 in [-8.0, -4.2, 0.4, 4.6, 8.2]:
                y_end = Y_BOT - rng.uniform(0.5, 2.5) if rng.random() < 0.5 else LOW - rng.uniform(0.0, 2.0)
                pts = root_path(x0 + rng.uniform(-0.4, 0.4), y_end, rng, 0.32, follow)
                if len(pts) > 2:
                    tube(bm, pts, rng.uniform(0.32, 0.42), 0.07, rng)
        obj("Roots", bm, "root", (0.5, 0.36, 0.28))
    if len(lit.verts):
        obj("MossLit", lit, "moss", (0.42, 0.66, 0.24))
        obj("MossShade", shade, "moss_shade", (0.26, 0.44, 0.2))
    else:
        lit.free()
        shade.free()


def bake_occlusion():
    """Each piece's occlusion into a float colour attribute (linear, as the game reads COLOR), with the rest of the built grove
    imported round it so the turf lip and the lawn shade the front too (less what the front replaces and the hidden block and ledge
    inside the island, which would read as covering it); the quilt's seams pressed darker into it, as the code-built
    quilt paints them."""
    mock = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    gone = {e["name"] for e in manifest["meshes"] if e["path"].split("/")[-1].startswith(("quilted_island", "Underside", "block", "faceted_ledge"))}
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.join(opts["dir"], "scene_in.glb"), merge_vertices=True)
    context = [o for o in bpy.context.scene.objects if o not in before]
    for o in context:
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
    scene.world.light_settings.distance = 2.5
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
        if tone is not None:
            col = o.data.color_attributes["Col"].data
            for v in o.data.vertices:
                k = tone.data[v.index].value or 1.0
                c = col[v.index].color
                col[v.index].color = (c[0] * k, c[1] * k, c[2] * k, 1.0)
        if o.name != "Quilt":
            continue
        col = o.data.color_attributes["Col"].data
        for v in o.data.vertices:
            seam = 1.0 - 0.38 * min(1.0, max(0.0, (quilt_d(v.co.x, v.co.z) - 0.82) / 0.18))
            c = col[v.index].color
            col[v.index].color = (c[0] * seam, c[1] * seam, c[2] * seam, 1.0)
    for o in context:
        bpy.data.objects.remove(o, do_unlink=True)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build()
    bake_occlusion()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("grove front %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
