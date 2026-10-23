"""The city's near skyline as a model (the owner, 2026-10-22: Blender models wherever possible). The game lays out the near layer's towers
(StageArt._skyline_layer) and paints their windows with shaders/facade.gdshader; this builds the same boxes as one mesh carrying the
shader's own coordinates (UV across a face from its left edge and up from its foot, UV2 the face's width and its box's height, colour
red the building's seed, green 1 for a plain face, blue 0.3 for a glass tower's walls), so the shader paints them exactly as before, and
adds what the shader can only paint: a frame round every window, a sill under it and a lintel over it (an arch of stones over the
arched ones), a ledge along every fourth floor, pilasters between the bays of the rendered buildings, a ledge under each row of a strip
building's bands. The relief is plain (the shader's lighter roof tone, its sides darker by the moon) and stands proud by its true
distance after the paper look squashes the layer into cards (`thin` in the export).

The windows are found as the shader finds them: its hash and choices are repeated here in 32-bit floats.

Export the layout from the game, then build (Blender 5.2):
  Godot --path godot res://main.tscn -- --demo=stage --stage=3 --noui --shots=<dir> --city-export=<file.json>
  blender --background --python art/blender/make_city_towers.py -- --layout <file.json> [--out godot/models/city/skyline_near.glb]
"""

import json
import math
import os
import sys

import bpy
import numpy as np

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
F = np.float32


def fract(x):
    return F(x - np.floor(x))


def shader_hash(a, b):
    """facade.gdshader's hash(vec2), in 32-bit floats."""
    p = np.array([a, b], dtype=np.float32)
    p = fract(p * np.array([123.34, 456.21], dtype=np.float32))
    d = F(p[0] * (p[0] + F(45.32)) + p[1] * (p[1] + F(45.32)))
    p = (p + d).astype(np.float32)
    return float(fract(F(p[0] * p[1])))


def mix(a, b, t):
    return a + (b - a) * t


class Mesh:
    """Faces gathered with the shader's corner data: each a quad (or triangle) of points with its UV, UV2 and colour."""

    def __init__(self):
        self.verts, self.faces, self.uv, self.uv2, self.col = [], [], [], [], []

    def quad(self, pts, uvs, uv2, col):
        base = len(self.verts)
        for p, u in zip(pts, uvs):
            self.verts.append(p)
            self.uv.append(u)
            self.uv2.append(uv2)
            self.col.append(col)
        self.faces.append(tuple(range(base, base + len(pts))))

    def face_quad(self, c, size, col):
        """`_facade_quad`: corners bottom-left, bottom-right, top-right, top-left seen from outside, UV across and up the face."""
        self.quad(c, [(0.0, 0.0), (size[0], 0.0), (size[0], size[1]), (0.0, size[1])], size, col)

    def facade_box(self, lo, hi, seed, plain, h, glass):
        """`_facade_box`: its front, sides and top."""
        col = (seed / 255.0, 1.0 if plain else 0.0, 0.3 if glass and not plain else 0.0, 1.0)
        (x0, y0, z0), (x1, y1, z1) = lo, hi
        w, d = x1 - x0, z1 - z0
        self.face_quad([(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)], (w, h), col)
        self.face_quad([(x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)], (d, h), col)
        self.face_quad([(x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1)], (d, h), col)
        self.face_quad([(x0, y1, z1), (x1, y1, z1), (x1, y1, z0), (x0, y1, z0)], (w, d), (seed / 255.0, 1.0, 0.0, 1.0))

    def relief(self, seed, foot, x0, x1, y0, y1, z0, z1, top=True, bottom=True):
        """A plain block on a facade: its front, sides, top and underside, UV.y its height above its building's foot (the shader darkens
        toward the street by it)."""
        col = (seed / 255.0, 1.0, 0.0, 1.0)

        def uvs(pts):
            return [(p[0], p[1] - foot) for p in pts]

        sides = [[(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)],
                 [(x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)],
                 [(x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1)]]
        if top:
            sides.append([(x0, y1, z1), (x1, y1, z1), (x1, y1, z0), (x0, y1, z0)])
        if bottom:
            sides.append([(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)])
        for pts in sides:
            self.quad(pts, uvs(pts), (1.0, 1.0), col)

    def wedge(self, seed, foot, pts2, z0, z1):
        """A plain block on a facade from a polygon in the front plane (an arch's stone)."""
        col = (seed / 255.0, 1.0, 0.0, 1.0)
        front = [(x, y, z1) for (x, y) in pts2]
        self.quad(front, [(p[0], p[1] - foot) for p in front], (1.0, 1.0), col)
        n = len(pts2)
        for i in range(n):
            a, b = pts2[i], pts2[(i + 1) % n]
            q = [(a[0], a[1], z1), (a[0], a[1], z0), (b[0], b[1], z0), (b[0], b[1], z1)]
            self.quad(q, [(p[0], p[1] - foot) for p in q], (1.0, 1.0), col)


def windows(m, box, doc):
    """The relief on a wall box's front where the shader paints its windows."""
    lo, hi, seed, plain, h, glass = box[0:3], box[3:6], box[6], box[7], box[8], box[9]
    if plain:
        return 0
    thin = doc["thin"]
    out = lambda d: d / thin  # noqa: E731  (a true distance proud of the card, before the paper look squashes the layer)
    cell = doc["cell"]
    pane = doc["pane"]
    first_row = doc["first_row"]
    parapet = doc["parapet"]
    s = float(seed)
    x0, y0, fz = lo[0], lo[1], hi[2]
    width = hi[0] - lo[0]
    height = h
    bay = (cell[0] * mix(0.8, 1.3, shader_hash(s, 1.1)), cell[1] * mix(0.88, 1.16, shader_hash(s, 2.2)))
    strips = glass or shader_hash(s, 3.3) < 0.12
    material = shader_hash(s, 11.1)
    cols = max(1.0, math.floor(width / bay[0]))
    bay_w = width / cols
    ps = (pane[0] * mix(0.75, 1.35, shader_hash(s, 4.4)), pane[1] * mix(0.8, 1.2, shader_hash(s, 7.7)))
    if strips:
        ps = (bay_w - 0.22, pane[1] * 0.62)
    ps = (min(ps[0], bay_w - 0.06), min(ps[1], bay[1] - 0.4))
    rows = math.floor((height - parapet - first_row) / bay[1])
    if rows < 1:
        return 0
    arched = material < 0.4 and shader_hash(s, 12.2) < 0.5 and not strips
    c0 = ((bay_w - ps[0]) * 0.5, (bay[1] - ps[1]) * 0.45)
    n = 0
    top_wall = y0 + height - parapet
    if material >= 0.7 and not strips:
        # Pilasters between the bays, up from the first row to the cap.
        for k in range(1, int(cols)):
            x = x0 + k * bay_w
            m.relief(seed, y0, x - 0.16, x + 0.16, y0 + first_row - 0.2, top_wall, fz, fz + out(0.07), bottom=False)
            n += 1
    for r in range(int(rows)):
        yb = y0 + first_row + r * bay[1]
        if r % 4 == 3:
            # A ledge along every fourth floor, where the shader paints its band.
            m.relief(seed, y0, x0 - 0.05, x0 + width + 0.05, yb + bay[1] - 0.18, yb + bay[1], fz, fz + out(0.14))
            n += 1
        if strips:
            m.relief(seed, y0, x0, x0 + width, yb + c0[1] - 0.1, yb + c0[1], fz, fz + out(0.1))
            n += 1
            continue
        for c in range(int(cols)):
            xa = x0 + c * bay_w + c0[0]
            xb = xa + ps[0]
            ya = yb + c0[1]
            yt = ya + ps[1]
            d = out(0.07)
            # The frame: four strips round the pane.
            m.relief(seed, y0, xa - 0.08, xa, ya - 0.08, yt + 0.06, fz, fz + d)
            m.relief(seed, y0, xb, xb + 0.08, ya - 0.08, yt + 0.06, fz, fz + d)
            m.relief(seed, y0, xa, xb, ya - 0.08, ya, fz, fz + d)
            if not arched:
                m.relief(seed, y0, xa, xb, yt, yt + 0.06, fz, fz + d)
                # The lintel over it.
                m.relief(seed, y0, xa - 0.1, xb + 0.1, yt + 0.06, yt + 0.18, fz, fz + out(0.11))
            else:
                # An arch of stones round the rounded head.
                rr = ps[0] * 0.5
                cx, cy = (xa + xb) * 0.5, yt - rr
                for k in range(5):
                    a0 = math.pi * k / 5 + 0.04
                    a1 = math.pi * (k + 1) / 5 - 0.04
                    r0, r1 = rr, rr + 0.12
                    m.wedge(seed, y0, [(cx + r0 * math.cos(a0), cy + r0 * math.sin(a0)), (cx + r1 * math.cos(a0), cy + r1 * math.sin(a0)),
                                       (cx + r1 * math.cos(a1), cy + r1 * math.sin(a1)), (cx + r0 * math.cos(a1), cy + r0 * math.sin(a1))],
                            fz, fz + out(0.1))
            # The sill, standing out further.
            m.relief(seed, y0, xa - 0.12, xb + 0.12, ya - 0.16, ya - 0.06, fz, fz + out(0.16))
            n += 1
    return n


def main():
    doc = json.load(open(opts["layout"], encoding="utf-8"))
    boxes = doc["boxes"]
    m = Mesh()
    lit = 0
    for b in boxes:
        m.facade_box(tuple(b[0:3]), tuple(b[3:6]), b[6], b[7], b[8], b[9])
        if opts.get("relief", "1") == "1":
            lit += windows(m, b, doc)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    # Blender's z is the game's up: (x, y, z) game to (x, -z, y).
    # (Named for the layout, as StageArt._city_sig folds it, so the game knows a stale model.)
    sig = len(boxes)
    for b in boxes:
        sig = (sig * 31 + round(b[0] * 100.0) + b[6] * 7) % 1000003
    me = bpy.data.meshes.new("skyline_near_%d" % sig)
    me.from_pydata([(x, -z, y) for (x, y, z) in m.verts], [], m.faces)
    uv = me.uv_layers.new(name="UV")
    uv2 = me.uv_layers.new(name="UV2")
    # (glTF's v runs down where Blender's runs up, and the exporter flips it: given as 1 - v, the game reads v.)
    for poly in me.polygons:
        for li in poly.loop_indices:
            vi = me.loops[li].vertex_index
            uv.data[li].uv = (m.uv[vi][0], 1.0 - m.uv[vi][1])
            uv2.data[li].uv = (m.uv2[vi][0], 1.0 - m.uv2[vi][1])
    col = me.color_attributes.new("Col", "FLOAT_COLOR", "CORNER")
    me.color_attributes.active_color = col
    for poly in me.polygons:
        for li in poly.loop_indices:
            col.data[li].color = m.col[me.loops[li].vertex_index]
    me.validate()
    for p in me.polygons:
        p.use_smooth = False
    ob = bpy.data.objects.new("skyline_near_%d" % sig, me)
    bpy.context.scene.collection.objects.link(ob)
    out = os.path.abspath(opts.get("out", "godot/models/city/skyline_near.glb"))
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.ops.export_scene.gltf(filepath=out, use_selection=True, export_format="GLB", export_yup=True, export_normals=True,
                              export_materials="NONE", export_vertex_color="ACTIVE", export_all_vertex_colors=False)
    tris = sum(len(p.vertices) - 2 for p in me.polygons)
    print("city towers: %d boxes, %d windowed bays and ledges, %d triangles -> %s" % (len(boxes), lit, tris, out))


main()
