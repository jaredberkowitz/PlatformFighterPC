"""The city's stage front as one modelled piece: the building the fighters stand on (the stage refinement, after the grove, meadow, beach
and canyon; docs/art/stages/BAKE_PILOT.md).

Built on the city's block from the bake manifest: from under the roof's coping and brass rail (which stay code-built, with the roof) down
to the street (`StageArt.CITY_STREET_DROP` under the block's foot), where the code-built shopfront and awning stay. The windows are real
openings with the glass set back in a reveal, lit rooms warm with curtains, dark rooms a deep blue, as the code-built facade's are. Three
looks for the owner to pick from (`--front-mock=` on the stage demo):
  A  brownstone: brick pilasters between pairs of windows, arched stone heads with a keystone, a stone band at every floor and a
     bracketed cornice under the roof;
  B  fire escape: plain lintels and sills, and a black iron fire escape down two bays, landings, rails and ladders;
  C  deco piers: tall stone piers running the full height, the windows in narrow columns between them with dark panels between floors.
Families: `brick` and `brick_back` (the city's own brick, laid in world space; the body behind the face is not subdivided), `trim`, `frame`, `glass_lit`, `glass_dark`, `curtain_a/b/c`, `neon`,
`bulb`, `pipe`, `iron`, `pot`, `leaf`, `ac`, `spandrel`; each piece bakes its occlusion into linear vertex colours with the built city
round it.

Run (Blender 5.2):
  blender --background --python art/blender/make_city_front.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
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
BLOCK = [r for r in manifest["rects"] if r[4] < 0.5][0]
LEFT, RIGHT, TOP, BOTTOM = BLOCK[0], BLOCK[1], BLOCK[2], BLOCK[3]
FZ = float(manifest["front_z"])
STREET = BOTTOM - 6.0          # StageArt.CITY_STREET_DROP
Y_TOP = TOP - 0.5              # under the coping and the brass rail
DEPTH = 6.0
REVEAL = 0.28                  # how far the glass sits back in the wall
FLOOR_H = (TOP - 0.5 - BOTTOM) / 4.0
UNDER_RECESS = 0.9            # StageArt.UNDER_RECESS: under the block's foot the building steps back
WZ = FZ                       # the wall face the window helpers build on (the block's, or the recessed one under it)


def P(x, y, z):
    return Vector((x, -z, y))


PARTS = {}


def part(mat):
    if mat not in PARTS:
        PARTS[mat] = bmesh.new()
        PARTS[mat].verts.layers.float.new("tone")
    return PARTS[mat]


def tone(bm, verts, t):
    layer = bm.verts.layers.float.get("tone")
    for v in verts:
        v[layer] = t


def box(mat, x0, x1, y0, y1, z0, z1, bevel=0.0, t=1.0):
    """A box from (x0, y0, z0) to (x1, y1, z1) in game coordinates, its edges bevelled by `bevel` (soft, chunky trim)."""
    bm = bmesh.new()
    res = bmesh.ops.create_cube(bm, size=1.0)
    for v in res["verts"]:
        v.co = P(x0 + (v.co.x + 0.5) * (x1 - x0), y0 + (v.co.z + 0.5) * (y1 - y0), z0 + (-v.co.y + 0.5) * (z1 - z0))
    if bevel > 0.0:
        bmesh.ops.bevel(bm, geom=bm.edges[:] + bm.verts[:], offset=min(bevel, (x1 - x0) * 0.45, (y1 - y0) * 0.45, (z1 - z0) * 0.45),
                        segments=1, affect="EDGES", profile=0.5)
    _merge(part(mat), bm, t)


def quad(mat, x0, x1, y0, y1, z, t=1.0):
    """A flat face toward the camera at depth `z`."""
    bm = bmesh.new()
    vs = [bm.verts.new(P(x, y, z)) for (x, y) in ((x0, y0), (x1, y0), (x1, y1), (x0, y1))]
    f = bm.faces.new(vs)
    f.normal_update()
    if f.normal.dot(P(0, 0, 1) - P(0, 0, 0)) < 0:
        f.normal_flip()
    _merge(part(mat), bm, t)


def cylinder(mat, x, y0, y1, z, r, seg=10, t=1.0):
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=seg, radius1=r, radius2=r, depth=y1 - y0)
    for v in bm.verts:
        v.co = Vector((v.co.x, v.co.y, v.co.z)) + P(x, (y0 + y1) * 0.5, z)
    _merge(part(mat), bm, t)


def sphere(mat, c, r, t=1.0):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=1, radius=r)
    for v in bm.verts:
        v.co = v.co + c
    _merge(part(mat), bm, t)


def _merge(dst, src, t):
    layer = dst.verts.layers.float.get("tone")
    m = {}
    for v in src.verts:
        nv = dst.verts.new(v.co)
        nv[layer] = t
        m[v] = nv
    for f in src.faces:
        try:
            dst.faces.new([m[v] for v in f.verts])
        except ValueError:
            pass
    src.free()


def wall(x0, x1, y0, y1, z=FZ, openings=()):
    """A brick wall face from x0..x1, y0..y1 at depth `z`, cut by window `openings` ([x0, x1, y0, y1], all in one row band each), with
    the reveals' inner faces in brick and a back face behind the glass. Built as strips so the openings are real holes."""
    rows = {}
    for o in openings:
        rows.setdefault((round(o[2], 3), round(o[3], 3)), []).append(o)
    edges = sorted({y0, y1} | {k[0] for k in rows} | {k[1] for k in rows}, reverse=True)
    for a, b in zip(edges, edges[1:]):
        band = rows.get((round(b, 3), round(a, 3)), [])
        xs = [x0]
        for o in sorted(band):
            xs += [o[0], o[1]]
        xs.append(x1)
        for k in range(0, len(xs), 2):
            if xs[k + 1] - xs[k] > 0.001:
                quad("brick", xs[k], xs[k + 1], b, a, z, rng_tone())
    for o in openings:
        ox0, ox1, oy0, oy1 = o
        bm = bmesh.new()
        f, b = z, z - REVEAL
        corners = [(ox0, oy0), (ox1, oy0), (ox1, oy1), (ox0, oy1)]
        for i in range(4):
            (ax, ay), (bx, by) = corners[i], corners[(i + 1) % 4]
            vs = [bm.verts.new(P(ax, ay, f)), bm.verts.new(P(bx, by, f)), bm.verts.new(P(bx, by, b)), bm.verts.new(P(ax, ay, b))]
            face = bm.faces.new(vs)
            face.normal_update()
            # (Facing into the opening.)
            centre = P((ox0 + ox1) * 0.5, (oy0 + oy1) * 0.5, (f + b) * 0.5)
            if face.normal.dot(centre - face.calc_center_median()) < 0:
                face.normal_flip()
        _merge(part("brick"), bm, 0.8)


TONE_RNG = random.Random(7)


def rng_tone():
    return TONE_RNG.uniform(0.9, 1.04)


def window(x0, x1, y0, y1, rng, bars=True):
    """The glass in an opening, set back by REVEAL: a lit room (warm, curtains drawn to the sides) about a third of the time, else dark;
    a frame round it and glazing bars."""
    z = WZ - REVEAL
    lit = rng.random() < 0.34
    quad("glass_lit" if lit else "glass_dark", x0, x1, y0, y1, z + 0.005, 1.0)
    w = x1 - x0
    if lit:
        cur = rng.choice(("curtain_a", "curtain_b", "curtain_c"))
        for s in (0, 1):
            cx0 = x0 + 0.04 if s == 0 else x1 - w * 0.24
            cx1 = x0 + w * 0.24 if s == 0 else x1 - 0.04
            # A curtain as three soft folds.
            for k in range(3):
                fx0 = cx0 + (cx1 - cx0) * k / 3.0
                fx1 = cx0 + (cx1 - cx0) * (k + 1) / 3.0
                box(cur, fx0, fx1, y0 + 0.04, y1 - 0.06, z + 0.01, z + 0.05 + 0.03 * (k % 2), 0.0, 0.85 + 0.12 * (k % 2))
    fw = 0.07
    box("frame", x0, x1, y1 - fw, y1, z, z + 0.08)
    box("frame", x0, x1, y0, y0 + fw, z, z + 0.08)
    box("frame", x0, x0 + fw, y0, y1, z, z + 0.08)
    box("frame", x1 - fw, x1, y0, y1, z, z + 0.08)
    if bars:
        mx = (x0 + x1) * 0.5
        box("frame", mx - 0.03, mx + 0.03, y0, y1, z, z + 0.06)
        hy = y1 - (y1 - y0) * 0.36
        box("frame", x0, x1, hy - 0.03, hy + 0.03, z, z + 0.06)


def sill(x0, x1, y, out=0.22):
    box("trim", x0 - 0.14, x1 + 0.14, y - 0.12, y, WZ - 0.05, WZ + out, 0.03)


def lintel(x0, x1, y, out=0.14):
    box("trim", x0 - 0.1, x1 + 0.1, y, y + 0.16, WZ - 0.05, WZ + out, 0.03)


def arch_head(x0, x1, y, rng):
    """A segmental stone arch over an opening: voussoirs fanned over the top, a keystone standing proud in the middle."""
    cx, w = (x0 + x1) * 0.5, x1 - x0
    rise = w * 0.22
    n = 7
    for k in range(n):
        a0 = math.pi * (0.15 + 0.7 * k / n)
        a1 = math.pi * (0.15 + 0.7 * (k + 1) / n)
        bm = bmesh.new()
        r_in, r_out = w * 0.62, w * 0.62 + 0.26
        oy = y + rise - r_in
        key = k == n // 2
        out = 0.2 if key else 0.12
        pts = []
        for (r, a) in ((r_in, a0), (r_in, a1), (r_out + (0.08 if key else 0.0), a1), (r_out + (0.08 if key else 0.0), a0)):
            pts.append((cx - math.cos(a) * r, oy + math.sin(a) * r))
        fr = [bm.verts.new(P(px, py, WZ + out)) for (px, py) in pts]
        bk = [bm.verts.new(P(px, py, WZ - 0.05)) for (px, py) in pts]
        faces = [bm.faces.new(fr), bm.faces.new(list(reversed(bk)))]
        for i in range(4):
            j = (i + 1) % 4
            faces.append(bm.faces.new((fr[i], bk[i], bk[j], fr[j])))
        bm.normal_update()
        centre = P(cx, oy + (r_in + r_out) * 0.5, WZ)
        c3 = sum((v.co for v in fr + bk), Vector()) / 8.0
        for f in faces:
            if f.normal.dot(f.calc_center_median() - c3) < 0:
                f.normal_flip()
        _merge(part("trim"), bm, (1.06 if key else rng.uniform(0.9, 1.0)))


def blade_sign(x, y, h):
    """The neon blade sign on a pier, as the code-built facade's: a round tube standing off the wall with bulb lights."""
    for k in range(4):
        cylinder("neon", x, y - h * 0.5 + h * k / 4.0 + 0.02, y - h * 0.5 + h * (k + 1) / 4.0 - 0.02, FZ + 0.32, 0.17, 12)
    box("iron", x - 0.04, x + 0.04, y + h * 0.5 - 0.1, y + h * 0.5, FZ - 0.05, FZ + 0.34)
    box("iron", x - 0.04, x + 0.04, y - h * 0.5, y - h * 0.5 + 0.1, FZ - 0.05, FZ + 0.34)
    for g in range(5):
        sphere("bulb", P(x, y + h * 0.5 - 0.28 - g * (h - 0.56) / 4.0, FZ + 0.5), 0.07)


def pot(x, y):
    box("pot", x - 0.12, x + 0.12, y, y + 0.17, WZ + 0.05, WZ + 0.25, 0.02)
    sphere("leaf", P(x, y + 0.27, WZ + 0.15), 0.17)
    sphere("leaf", P(x + 0.1, y + 0.33, WZ + 0.17), 0.1, 0.9)


def ac(x, y):
    box("ac", x - 0.26, x + 0.26, y - 0.34, y, WZ - 0.02, WZ + 0.3, 0.03)
    box("iron", x - 0.2, x + 0.2, y - 0.28, y - 0.06, WZ + 0.3, WZ + 0.32)


def pipe(x, y0, y1):
    cylinder("pipe", x, y0, y1, FZ + 0.1, 0.07, 8)
    for y in range(int(y0) + 1, int(y1)):
        box("pipe", x - 0.1, x + 0.1, y - 0.05, y + 0.05, FZ - 0.02, FZ + 0.2)


def floors():
    """The floor lines down from the roof: four floors to the block's foot."""
    return [Y_TOP - FLOOR_H * k for k in range(4)], BOTTOM


def cornice(y, rng, brackets=True):
    """A deep cornice under the roof: a crown that stands well out, a frieze, brackets under the crown."""
    box("trim", LEFT - 0.2, RIGHT + 0.2, y - 0.22, y, FZ - 0.1, FZ + 0.42, 0.05, 1.04)
    box("trim", LEFT - 0.1, RIGHT + 0.1, y - 0.36, y - 0.22, FZ - 0.1, FZ + 0.26, 0.03, 0.92)
    if brackets:
        x = LEFT + 0.35
        while x < RIGHT - 0.2:
            box("trim", x - 0.09, x + 0.09, y - 0.5, y - 0.36, FZ - 0.05, FZ + 0.24, 0.03, rng.uniform(0.86, 0.96))
            x += 0.72


def backing():
    """The building's body behind the face (its sides and back), so nothing behind shows through the openings' back faces: the block's
    part, and under its foot the part that steps back."""
    box("brick_back", LEFT, RIGHT, BOTTOM, Y_TOP, FZ - DEPTH, FZ - 0.4, 0.0, 0.7)
    quad("glass_dark", LEFT + 0.1, RIGHT - 0.1, BOTTOM, Y_TOP, FZ - REVEAL - 0.02, 0.6)
    lz = FZ - UNDER_RECESS
    box("brick_back", LEFT + 0.2, RIGHT - 0.2, STREET, BOTTOM, FZ - DEPTH, lz - 0.4, 0.0, 0.7)
    quad("glass_dark", LEFT + 0.3, RIGHT - 0.3, STREET, BOTTOM, lz - REVEAL - 0.02, 0.6)
    # The returns at each end, between the face and the body.
    for x0, x1, y0, y1, zf in ((LEFT, LEFT + 0.15, BOTTOM, Y_TOP, FZ), (RIGHT - 0.15, RIGHT, BOTTOM, Y_TOP, FZ),
                               (LEFT + 0.2, LEFT + 0.35, STREET, BOTTOM, lz), (RIGHT - 0.35, RIGHT - 0.2, STREET, BOTTOM, lz)):
        box("brick", x0, x1, y0, y1, zf - 0.41, zf - 0.01, 0.0, 0.8)


def lower(rng):
    """Under the block's foot, the building steps back (as the code-built tower does): a stone belt at the foot, a row of windows, and
    plain wall down to the street behind the code-built shopfront."""
    global WZ
    WZ = FZ - UNDER_RECESS
    box("trim", LEFT - 0.05, RIGHT + 0.05, BOTTOM - 0.32, BOTTOM, WZ - 0.1, FZ + 0.24, 0.04, 0.95)
    x0, x1 = LEFT + 0.2, RIGHT - 0.2
    y1, y0 = BOTTOM - 0.55, BOTTOM - 1.2
    n = 8
    openings = []
    for k in range(n):
        cx = x0 + (x1 - x0) * (k + 0.5) / n
        openings.append([cx - 0.38, cx + 0.38, y0, y1])
    wall(x0, x1, STREET, BOTTOM - 0.32, WZ, openings)
    for o in openings:
        window(o[0], o[1], o[2], o[3], rng)
        sill(o[0], o[1], o[2])
        lintel(o[0], o[1], o[3])
    WZ = FZ


def build():
    rng = random.Random(23)
    backing()
    tops, last = floors()
    lower(rng)
    if VARIANT in ("A", "B"):
        # Four bays, each a pair of windows; pilasters between the bays (A), a plain face (B).
        bays = 4
        bw = (RIGHT - LEFT) / bays
        openings = []
        heads = []
        for n, ft in enumerate(tops):
            wy1 = ft - (0.95 if n == 0 else 0.5)
            wy0 = wy1 - (0.85 if n == 0 else 1.0)
            for b in range(bays):
                bx = LEFT + bw * b
                for s in (0, 1):
                    cx = bx + bw * (0.3 if s == 0 else 0.7)
                    hw = 0.42
                    openings.append([cx - hw, cx + hw, wy0, wy1])
                    heads.append((cx - hw, cx + hw, wy0, wy1, n, b, s))
        wall(LEFT, RIGHT, last, Y_TOP, FZ, openings)
        for (x0, x1, y0, y1, n, b, s) in heads:
            window(x0, x1, y0, y1, rng)
            sill(x0, x1, y0)
            if VARIANT == "A":
                arch_head(x0, x1, y1, rng)
            else:
                lintel(x0, x1, y1)
            r = rng.random()
            if r < 0.18:
                pot(x0 + 0.2, y0)
            elif r < 0.28 and not (VARIANT == "B" and b in (1, 2)):
                ac(x0 + (x1 - x0) * 0.6, y0 - 0.14)
        if VARIANT == "A":
            cornice(Y_TOP, rng)
            for b in range(bays + 1):
                px = LEFT + bw * b
                hw = 0.32 if 0 < b < bays else 0.38
                ox = 0.0 if 0 < b < bays else (hw if b == 0 else -hw)
                box("brick", px + ox - hw, px + ox + hw, last, Y_TOP - 0.36, FZ, FZ + 0.16, 0.03, 0.97)
                # A stone cap and base on each pilaster.
                box("trim", px + ox - hw - 0.06, px + ox + hw + 0.06, Y_TOP - 0.82, Y_TOP - 0.62, FZ, FZ + 0.24, 0.03)
            for n, ft in enumerate(tops[1:], 1):
                box("trim", LEFT, RIGHT, ft - 0.08, ft + 0.08, FZ - 0.05, FZ + 0.12, 0.03, 0.94)
            blade_sign(LEFT + bw * 2, tops[1] - 1.0 if len(tops) > 1 else tops[0] - 1.2, 2.2)
        else:
            cornice(Y_TOP, rng, brackets=False)
            fire_escape(LEFT + bw * 1, LEFT + bw * 3, tops)
            blade_sign(RIGHT - 0.6, tops[0] - 1.6, 2.2)
        pipe(LEFT + 0.12 if VARIANT == "B" else LEFT + bw * 3 + 0.5, last, Y_TOP - 0.3)
    else:
        # Deco: full-height piers, the windows in columns between them with dark panels between floors; a stepped crown.
        cols = 6
        cw = (RIGHT - LEFT) / cols
        openings = []
        heads = []
        for n, ft in enumerate(tops):
            wy1 = ft - (0.55 if n == 0 else 0.35)
            wy0 = ft - FLOOR_H + 0.55
            for c in range(cols):
                cx = LEFT + cw * (c + 0.5)
                hw = cw * 0.3
                openings.append([cx - hw, cx + hw, wy0, wy1])
                heads.append((cx - hw, cx + hw, wy0, wy1, n, c))
        wall(LEFT, RIGHT, last, Y_TOP, FZ, openings)
        for (x0, x1, y0, y1, n, c) in heads:
            window(x0, x1, y0, y1, rng, bars=False)
            mx = (x0 + x1) * 0.5
            box("frame", mx - 0.03, mx + 0.03, y0, y1, FZ - REVEAL, FZ - REVEAL + 0.06)
            # The spandrel under each window: a dark panel with a raised chevron.
            box("spandrel", x0, x1, y0 - 0.5, y0 - 0.05, FZ - 0.12, FZ - 0.02, 0.02)
            for k in range(2):
                yy = y0 - 0.2 - k * 0.14
                box("trim", mx - 0.22 + k * 0.06, mx - 0.01, yy - 0.04, yy + 0.04, FZ - 0.02, FZ + 0.03)
                box("trim", mx + 0.01, mx + 0.22 - k * 0.06, yy - 0.04, yy + 0.04, FZ - 0.02, FZ + 0.03)
            if rng.random() < 0.15:
                pot(mx - 0.1, y0)
        for c in range(cols + 1):
            px = LEFT + cw * c
            hw = cw * 0.2 if 0 < c < cols else cw * 0.24
            ox = 0.0 if 0 < c < cols else (hw if c == 0 else -hw)
            box("trim", px + ox - hw, px + ox + hw, last, Y_TOP - 0.1, FZ - 0.05, FZ + 0.22, 0.05, 0.96 if c % 2 else 1.02)
            # A stepped top on each pier, standing a little proud under the coping.
            box("trim", px + ox - hw * 0.7, px + ox + hw * 0.7, Y_TOP - 0.7, Y_TOP - 0.1, FZ + 0.2, FZ + 0.32, 0.03, 1.06)
        box("trim", LEFT - 0.1, RIGHT + 0.1, Y_TOP - 0.24, Y_TOP, FZ - 0.1, FZ + 0.36, 0.04, 1.02)
        blade_sign(LEFT + cw * 3, tops[0] - 1.6, 2.2)


def fire_escape(x0, x1, tops):
    """A black iron fire escape across two bays: a landing under each floor's windows with a rail, a ladder down between landings."""
    for n, ft in enumerate(tops):
        y = ft - 1.9 if n == 0 else ft - 1.6
        if y < BOTTOM + 0.3:
            break
        box("iron", x0, x1, y - 0.06, y, FZ, FZ + 1.0)
        # Rail: posts, a top rail and a mid rail.
        for x in [x0 + (x1 - x0) * k / 6.0 for k in range(7)]:
            box("iron", x - 0.03, x + 0.03, y, y + 0.9, FZ + 0.94, FZ + 1.0)
        for ry in (y + 0.45, y + 0.88):
            box("iron", x0, x1, ry - 0.025, ry + 0.025, FZ + 0.94, FZ + 1.0)
        for x in (x0, x1):
            box("iron", x - 0.03, x + 0.03, y, y + 0.9, FZ, FZ + 1.0)
        # Brackets under the landing back to the wall.
        for x in (x0 + 0.3, (x0 + x1) * 0.5, x1 - 0.3):
            box("iron", x - 0.03, x + 0.03, y - 0.5, y - 0.06, FZ, FZ + 0.06)
            for k in range(5):
                f = k / 5.0
                box("iron", x - 0.03, x + 0.03, y - 0.5 + 0.44 * f, y - 0.5 + 0.44 * (f + 0.2), FZ + 0.9 * f, FZ + 0.9 * (f + 0.2))
        # The stair down to the next landing, alternating ends.
        if n + 1 < len(tops):
            yn = tops[n + 1] - 1.6
            if yn < BOTTOM + 0.3:
                break
            sx = x1 - 1.4 if n % 2 == 0 else x0 + 0.2
            ex = sx + 1.2
            steps = 7
            for k in range(steps):
                f = (k + 0.5) / steps
                sy = y - (y - yn) * f
                box("iron", sx + (ex - sx) * f - 0.18, sx + (ex - sx) * f + 0.18, sy - 0.03, sy, FZ + 0.2, FZ + 0.8)
            # Its stringers, a short bar from step to step on each side.
            for k in range(steps + 1):
                f0, f1 = k / (steps + 1.0), (k + 1) / (steps + 1.0)
                xa, xb = sx + (ex - sx) * f0, sx + (ex - sx) * f1
                ya, yb = y - (y - yn) * f0, y - (y - yn) * f1
                for z in (FZ + 0.2, FZ + 0.8):
                    box("iron", min(xa, xb) - 0.02, max(xa, xb) + 0.02, min(ya, yb) - 0.08, max(ya, yb), z - 0.03, z + 0.03)


def make_objects():
    for mat, bm in PARTS.items():
        for v in bm.verts:
            v.co.x = min(max(v.co.x, LEFT - 0.6), RIGHT + 0.6)
            v.co.z = min(v.co.z, TOP - 0.45)
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        # (Finer on the wall, so the occlusion baked into the vertices follows the shapes: on big faces it samples only the corners.)
        if mat == "brick":
            for _ in range(5):
                long_edges = [e for e in bm.edges if e.calc_length() > 0.7]
                if not long_edges:
                    break
                bmesh.ops.subdivide_edges(bm, edges=long_edges, cuts=1, use_grid_fill=True)
                bmesh.ops.triangulate(bm, faces=bm.faces[:])
        me = bpy.data.meshes.new("Facade_" + mat)
        bm.normal_update()
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new("Facade_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def bake_occlusion():
    mock = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    gone = {e["name"] for e in manifest["meshes"] if e["path"].split("/")[-1].startswith(("block", "Underside"))}
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.join(opts["dir"], "scene_in.glb"), merge_vertices=True)
    for o in [o for o in bpy.context.scene.objects if o not in before]:
        # (Nor the code-built facade's pieces, exported as occluders lying on the wall's own face: they blacked out its occlusion.)
        cs = [o.matrix_world @ Vector(c) for c in o.bound_box] if o.type == "MESH" else []
        on_face = bool(cs) and min(c.x for c in cs) > LEFT - 0.7 and max(c.x for c in cs) < RIGHT + 0.7 and \
            min(c.z for c in cs) > STREET - 0.1 and max(c.z for c in cs) < TOP - 0.38 and -min(c.y for c in cs) < FZ + 0.6
        if o.name in gone or on_face:
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
    scene.world.light_settings.distance = 0.9
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
        # (The glass and the lights keep most of their own brightness: a lit room is not shaded by its reveal as much as a wall is.)
        # (And everything keeps some: at full strength the frames and curtains deep in their reveals baked black, and the wall went muddy
        # beside the code-built facade's.)
        fam = o.name.split("_", 1)[1]
        keep = 0.6 if fam in ("glass_lit", "glass_dark", "bulb", "neon") else 0.55 if fam in ("frame", "curtain_a", "curtain_b", "curtain_c") \
            else 0.3
        for v in o.data.vertices:
            k = (t.data[v.index].value or 1.0) if t is not None else 1.0
            c = col[v.index].color
            ao = keep + (1.0 - keep) * c[0]
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
    print("city front %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
