"""The paper house kit, version 2 (docs/art/stages/HOUSES.md, the owner's direction 2026-10-21): the houses at the characters' proportions,
as the papercraft adventure builds its towns, in the layered card the owner picked (make_paper_houses.py style B). Its houses are as chibi
as its people: a big arched door (about 1.15 fighters) under walls only a little taller, a roof as tall as the walls with a deep, thick
overhang, few and big windows with thick frames, every detail a separate thick piece (stones round the arch, timber braces, a stone base
course, roof rows with their edges showing).
  Shapes (the options):
    A  chunky: those proportions and details, straight roofs;
    B  bell: A with flared bell roofs on the houses and gambrel roofs on the long house and the barn;
    C  hand-made: B with sagging ridges, walls leaning a touch and uneven roof rows, nothing quite straight.
The same four houses as version 1 (`prop_house_<type>`: cottage, townhouse, longhouse, barn), so the game places them the same way.

`--detail 1` (the owner's detail pass, 2026-10-22: more detail on the background buildings): shutters beside the front windows, a
window in each side wall, a lantern by every door, a picket fence and flower pots before the houses; on the barn hay in the loft, a side
window and a weathervane on the ridge.

The beach's harbour buildings (the owner, 2026-10-22: Blender models wherever possible), in the same card: the bell tower, the
warehouse, a market stall and the fisherman's stilt hut with its jetty (`prop_house_belltower`, `_warehouse`, `_stall`, `_stilthut`).

Run (Blender 5.2):
  blender --background --python art/blender/make_paper_houses_v2.py -- --shape A --out <file.glb> [--render <sheet.png>]
Shipped (the owner's pick, 2026-10-21): godot/models/props/paper_houses.glb, `--shape B` (with the detail, the default since 2026-10-22).
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import make_paper_houses as hk  # noqa: E402
import make_paper_props as kit  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
SHAPE = opts.get("shape", "A").upper()
hk.STYLE = "B"
# (On by default since the owner shipped it, 2026-10-22; `--detail 0` builds the kit without.)
DETAIL = opts.get("detail", "1") == "1"


def prism(fam, name, ring0, ring1):
    """A solid between two rings of game points (a polygon and its copy moved along)."""
    n = len(ring0)
    faces = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    faces += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return hk.solid(name, fam, list(ring0) + list(ring1), faces)


def front(fam, name, pts, z0, z1):
    """A polygon in the front plane (game x, y) standing from z0 to z1."""
    return prism(fam, name, [(x, y, z0) for (x, y) in pts], [(x, y, z1) for (x, y) in pts])


def across(f, fam, name, pts, a0, a1):
    """A polygon in the house's across-up plane ((c, y) points) run along the ridge from a0 to a1."""
    return prism(fam, name, [f.p(a0, y, c) for (c, y) in pts], [f.p(a1, y, c) for (c, y) in pts])


def profile(kind, half, H, rise, ov, n=10):
    """One side of the roof, from the ridge (c = 0) out past the eave (c = half + ov): straight, a flared bell, or a gambrel."""
    pts = []
    for k in range(n + 1):
        t = k / n
        if kind == "bell":
            y = H + rise * (1.0 - t) ** 1.35
        elif kind == "gambrel":
            y = H + rise * (1.0 - 0.38 * t / 0.55) if t <= 0.55 else H + rise * 0.62 * (1.0 - (t - 0.55) / 0.45)
        else:
            y = H + rise * (1.0 - t)
        pts.append((t * half, y))
    # The overhang kicks out shallow, whatever the roof's pitch, as the references' eaves do: carried on a steep roof's own slope it
    # dropped below the wall top and hid the windows and the door.
    (c1, y1), (c0, y0) = pts[-1], pts[-2]
    slope = max((y1 - y0) / max(c1 - c0, 1e-6), -0.45)
    slope = min(slope, -0.2)
    for k in (1, 2):
        c = half + ov * k / 2.0
        pts.append((c, H + slope * (c - half)))
    return pts


def at(pts, u):
    """The point a share `u` of the way along a polyline."""
    lens = [0.0]
    for (a, b) in zip(pts, pts[1:]):
        lens.append(lens[-1] + math.dist(a, b))
    target = u * lens[-1]
    for i in range(len(pts) - 1):
        if lens[i + 1] >= target:
            k = (target - lens[i]) / max(lens[i + 1] - lens[i], 1e-6)
            return (pts[i][0] + (pts[i + 1][0] - pts[i][0]) * k, pts[i][1] + (pts[i + 1][1] - pts[i][1]) * k)
    return pts[-1]


def sub(pts, u0, u1, n=6):
    return [at(pts, u0 + (u1 - u0) * k / n) for k in range(n + 1)]


def roof(f, along, across_w, H, rise, kind, ov=0.75, th=0.26, rows=4):
    """A thick card roof over the walls, `ov` over every edge, laid in overlapping rows with their lower edges showing; its gables
    filled with wall. Returns the ridge's height."""
    la = along / 2.0 + ov
    half = across_w / 2.0
    for s in (-1.0, 1.0):
        top = profile(kind, half, H, rise, ov)
        under = [(c, y - th) for (c, y) in top]
        across(f, "roof", "roof", [(s * c, y) for (c, y) in top] + [(s * c, y) for (c, y) in reversed(under)], -la, la)
        # The rows: each a strip of card on the one below it, a little proud, its lower edge a lit lip.
        for k in range(rows):
            u0 = k / rows
            u1 = min(1.0, (k + 1) / rows + 0.05)
            strip = sub(top, u0, u1)
            lift = 0.07
            up = [(c, y + lift + 0.06) for (c, y) in strip]
            lo = [(c, y + 0.01) for (c, y) in strip]
            across(f, "roof", "roof_row", [(s * c, y) for (c, y) in up] + [(s * c, y) for (c, y) in reversed(lo)], -la - 0.02, la + 0.02)
            (ce, ye) = strip[-1]
            across(f, "roof_edge", "roof_lip", [(s * (ce - 0.02), ye - 0.06), (s * (ce + 0.1), ye - 0.06), (s * (ce + 0.1), ye + 0.16),
                                                 (s * (ce - 0.02), ye + 0.16)], -la - 0.03, la + 0.03)
    # The ridge: a rolled cap along the top.
    hk.tube("roof_edge", f.p(-la, H + rise + 0.12, 0.0), f.p(la, H + rise + 0.12, 0.0), 0.2, segs=10, name="roof_ridge")
    # The gables, filled up to the roof's underside.
    side = [(c, max(H + 0.001, y - th)) for (c, y) in profile(kind, half, H, rise, 0.0)[:11]]
    gable = [(-c, y) for (c, y) in reversed(side)] + side[1:]
    for a in (-along / 2.0, along / 2.0):
        inset = -0.08 if a > 0 else 0.08
        across(f, "wall", "wall_gable", gable, a + inset, a)
    return H + rise


def walls(W, D, H):
    hk.box("stone", -W / 2.0 - 0.18, W / 2.0 + 0.18, -0.5, 0.35, -D / 2.0 - 0.18, D / 2.0 + 0.18, name="footing")
    hk.box("wall", -W / 2.0, W / 2.0, 0.35, H, -D / 2.0, D / 2.0, name="wall")
    fz = D / 2.0
    # Corner posts and the beams at the floor and the eaves, thick and proud.
    for sx in (-1.0, 1.0):
        for sz in (-1.0, 1.0):
            hk.box("timber", sx * W / 2.0 - 0.2, sx * W / 2.0 + 0.2, 0.35, H, sz * D / 2.0 - 0.2, sz * D / 2.0 + 0.2, name="timber_post")
    for y in (0.35, H - 0.26):
        hk.box("timber", -W / 2.0, W / 2.0, y, y + 0.26, fz, fz + 0.12, name="timber_beam")
    return fz


def base_course(W, fz, skip, rng):
    """A course of raised stones along the foot of the front wall (left out where a door stands)."""
    y0 = 0.62
    for row in range(2):
        x = -W / 2.0 + 0.25 - (0.32 if row else 0.0)
        while x < W / 2.0 - 0.3:
            w = rng.uniform(0.55, 0.8)
            x1 = min(x + w, W / 2.0 - 0.25)
            if not any(a - 0.1 < (x + x1) / 2.0 < b + 0.1 for (a, b) in skip) and x1 - x > 0.25:
                hk.box("stone", x + 0.04, x1 - 0.04, y0 + row * 0.34, y0 + row * 0.34 + 0.29, fz, fz + 0.07, name="stone_course")
            x = x1


def brace(x0, x1, y0, y1, fz):
    """A timber X in a wall panel."""
    t = 0.13
    for (ax, ay, bx, by) in ((x0, y0, x1, y1), (x0, y1, x1, y0)):
        dx, dy = bx - ax, by - ay
        ln = math.hypot(dx, dy)
        nx, ny = -dy / ln * t, dx / ln * t
        front("timber", "timber_brace", [(ax - nx, ay - ny), (bx - nx, by - ny), (bx + nx, by + ny), (ax + nx, ay + ny)], fz, fz + 0.1)


def arched_door(x, w, h, fz):
    """A big arched plank door, a round knob, chunky stones round its arch and up its sides, a stone step."""
    hk.box("stone", x - w / 2.0 - 0.45, x + w / 2.0 + 0.45, -0.2, 0.35, fz - 0.05, fz + 0.6, name="stone_step")
    r = w / 2.0
    spring = 0.35 + h - r
    arch = [(x - r, 0.35), (x + r, 0.35)] + [(x + r * math.cos(a), spring + r * math.sin(a)) for a in [math.pi * k / 10 for k in range(11)]]
    front("door", "door", arch[:1] + arch[1:], fz - 0.02, fz + 0.06)
    for k in (-1, 1):
        hk.box("door_seam", x + k * w / 6.0 - 0.025, x + k * w / 6.0 + 0.025, 0.4, spring + r * 0.85, fz + 0.06, fz + 0.08, name="door_seam")
    hk.box("knob", x + r * 0.55 - 0.09, x + r * 0.55 + 0.09, 1.45, 1.63, fz + 0.06, fz + 0.18, name="knob")
    # The arch's stones (its voussoirs), seven, with gaps; the jambs' stones below them.
    n = 7
    r0, r1 = r + 0.04, r + 0.36
    for k in range(n):
        a0 = math.pi * k / n + 0.03
        a1 = math.pi * (k + 1) / n - 0.03
        pts = [(x + r0 * math.cos(a0), spring + r0 * math.sin(a0)), (x + r1 * math.cos(a0), spring + r1 * math.sin(a0)),
               (x + r1 * math.cos(a1), spring + r1 * math.sin(a1)), (x + r0 * math.cos(a1), spring + r0 * math.sin(a1))]
        front("stone", "stone_arch", pts, fz, fz + 0.16)
    for sx in (-1.0, 1.0):
        y = 0.35
        k = 0
        while y < spring - 0.1:
            hgt = min(0.55, spring - y)
            wide = 0.36 if k % 2 == 0 else 0.26
            xa = x + sx * (r + 0.04)
            xb = x + sx * (r + 0.04 + wide)
            hk.box("stone", min(xa, xb), max(xa, xb), y + 0.03, y + hgt - 0.03, fz, fz + 0.14, name="stone_jamb")
            y += hgt
            k += 1


def window(x, y, w, h, fz, box_under=True):
    """A big window: a thick frame, a cross of glazing bars, blue glass (daylight), a sill and a box of flowers."""
    hk.box("glass", x - w / 2.0, x + w / 2.0, y - h / 2.0, y + h / 2.0, fz - 0.02, fz + 0.03, name="glass")
    t = 0.2
    for (x0, x1, y0, y1) in ((x - w / 2.0 - t, x + w / 2.0 + t, y - h / 2.0 - t, y - h / 2.0), (x - w / 2.0 - t, x + w / 2.0 + t, y + h / 2.0, y + h / 2.0 + t),
                             (x - w / 2.0 - t, x - w / 2.0, y - h / 2.0, y + h / 2.0), (x + w / 2.0, x + w / 2.0 + t, y - h / 2.0, y + h / 2.0)):
        hk.box("trim", x0, x1, y0, y1, fz, fz + 0.12, name="trim_frame")
    hk.box("trim", x - 0.05, x + 0.05, y - h / 2.0, y + h / 2.0, fz, fz + 0.08, name="trim_bar")
    hk.box("trim", x - w / 2.0, x + w / 2.0, y - 0.05, y + 0.05, fz, fz + 0.08, name="trim_bar")
    hk.box("timber", x - w / 2.0 - 0.28, x + w / 2.0 + 0.28, y - h / 2.0 - t - 0.14, y - h / 2.0 - t + 0.02, fz, fz + 0.3, name="timber_sill")
    if box_under:
        hk.box("timber", x - w / 2.0 - 0.05, x + w / 2.0 + 0.05, y - h / 2.0 - t - 0.6, y - h / 2.0 - t - 0.16, fz, fz + 0.4, name="timber_box")
        for k in range(4):
            px = x - w / 2.0 + (k + 0.5) * w / 4.0
            hk.box("petal", px - 0.15, px + 0.15, y - h / 2.0 - t - 0.2, y - h / 2.0 - t + 0.06, fz + 0.1, fz + 0.38, name="petal")


def shutters(x, y, w, h, fz):
    """A plank shutter folded back each side of a window, a little proud of the wall, with a cross board."""
    for sx in (-1.0, 1.0):
        x0 = x + sx * (w / 2.0 + 0.22)
        x1 = x0 + sx * w * 0.48
        hk.box("shutter", min(x0, x1), max(x0, x1), y - h / 2.0 - 0.1, y + h / 2.0 + 0.1, fz + 0.02, fz + 0.12, name="shutter")
        hk.box("timber", min(x0, x1) + 0.06, max(x0, x1) - 0.06, y - 0.06, y + 0.06, fz + 0.12, fz + 0.16, name="timber_shutter")


def side_window(sx, W, y, z, w, h):
    """A window in a side wall (`sx` the side), framed and glazed, with a sill."""
    fx = sx * W / 2.0
    o = sx * 0.03
    hk.box("glass", fx - 0.02 if sx > 0 else fx - 0.03, fx + 0.03 if sx > 0 else fx + 0.02, y - h / 2.0, y + h / 2.0, z - w / 2.0, z + w / 2.0, name="glass")
    t = 0.18
    for (y0, y1, z0, z1) in ((y - h / 2.0 - t, y - h / 2.0, z - w / 2.0 - t, z + w / 2.0 + t), (y + h / 2.0, y + h / 2.0 + t, z - w / 2.0 - t, z + w / 2.0 + t),
                             (y - h / 2.0, y + h / 2.0, z - w / 2.0 - t, z - w / 2.0), (y - h / 2.0, y + h / 2.0, z + w / 2.0, z + w / 2.0 + t)):
        a, b = sorted((fx, fx + sx * 0.12))
        hk.box("trim", a, b, y0, y1, z0, z1, name="trim_frame")
    a, b = sorted((fx, fx + sx * 0.08))
    hk.box("trim", a, b, y - h / 2.0, y + h / 2.0, z - 0.05, z + 0.05, name="trim_bar")
    a, b = sorted((fx, fx + sx * 0.28))
    hk.box("timber", a, b, y - h / 2.0 - t - 0.14, y - h / 2.0 - t + 0.02, z - w / 2.0 - 0.25, z + w / 2.0 + 0.25, name="timber_sill")
    _ = o


def lantern(x, y, fz):
    """A lantern on a bracket beside a door: an iron arm, a glowing glass box under a little roof."""
    hk.box("iron", x - 0.05, x + 0.05, y - 0.05, y + 0.05, fz, fz + 0.45, name="iron_arm")
    # (Warm paper, not the lit "glow" family: drawn in its own light, a lantern on every house bloomed the town and its edges.)
    hk.box("lamp", x - 0.17, x + 0.17, y - 0.5, y - 0.05, fz + 0.28, fz + 0.62, name="lamp_lantern")
    hk.box("iron", x - 0.23, x + 0.23, y - 0.05, y + 0.06, fz + 0.22, fz + 0.68, name="iron_cap")


def picket_fence(x0, x1, fz, gate=None):
    """A low picket fence a step before the front, two rails and pointed pickets, open where the path to the door runs."""
    # (Close before the front, where the house's own footing levels the ground, and the pickets reaching well down: 1.6 out, on a slope
    # the fence ran into the hill on one side.)
    z = fz + 0.85
    hk.box("wood_light", x0, x1, 0.3, 0.42, z - 0.04, z + 0.04, name="fence_rail")
    hk.box("wood_light", x0, x1, 0.75, 0.87, z - 0.04, z + 0.04, name="fence_rail")
    x = x0
    while x <= x1 + 1e-3:
        if gate is None or not (gate[0] < x < gate[1]):
            hk.box("white", x - 0.07, x + 0.07, -0.6, 1.05, z + 0.04, z + 0.1, name="fence_picket")
            front("white", "fence_tip", [(x - 0.07, 1.05), (x + 0.07, 1.05), (x, 1.22)], z + 0.04, z + 0.1)
        x += 0.42
    if gate is not None:
        # The path to the door: flat stones.
        for k in range(2):
            zz = fz + 0.35 + k * 0.5
            hk.box("stone_light", (gate[0] + gate[1]) / 2.0 - 0.32, (gate[0] + gate[1]) / 2.0 + 0.32, -0.05, 0.02, zz - 0.2, zz + 0.2, name="path_stone")


def flower_pot(x, fz):
    hk.box("pot", x - 0.22, x + 0.22, 0.0, 0.42, fz + 0.3, fz + 0.72, name="pot")
    hk.box("leaf", x - 0.28, x + 0.28, 0.42, 0.72, fz + 0.26, fz + 0.76, name="leaf")
    hk.box("petal", x - 0.12, x + 0.12, 0.66, 0.86, fz + 0.4, fz + 0.62, name="petal")


def chimney(x, z, y0, top):
    hk.box("stone", x - 0.42, x + 0.42, y0, top, z - 0.42, z + 0.42, name="chimney")
    hk.box("stone", x - 0.56, x + 0.56, top, top + 0.24, z - 0.56, z + 0.56, name="chimney_cap")


def turned(fam, name, pts, z0, z1, q):
    """`front` on the face a quarter-turn `q` round the up axis (0 the front, 1 the right side, 2 the back, 3 the left)."""
    a = q * math.pi / 2.0
    c, sn = math.cos(a), math.sin(a)

    def rot(x, y, z):
        return (x * c + z * sn, y, -x * sn + z * c)

    return prism(fam, name, [rot(x, y, z0) for (x, y) in pts], [rot(x, y, z1) for (x, y) in pts])


def lathe(fam, name, prof, centre, segs=12, squash=1.0, jitter=None):
    """A solid turned round the up axis from `prof` ((radius, height) from the bottom up) at `centre`, capped; `squash` scales it in
    depth, `jitter(i, k)` an (dr, dy) for ring k's point i (a ragged hem)."""
    bm = bmesh.new()
    rings = []
    for k, (rad, y) in enumerate(prof):
        row = []
        for i in range(segs):
            a = math.tau * i / segs
            dr, dy = jitter(i, k) if jitter else (0.0, 0.0)
            row.append(bm.verts.new(hk.P(centre[0] + (rad + dr) * math.cos(a), centre[1] + y + dy, centre[2] + (rad + dr) * math.sin(a) * squash)))
        rings.append(row)
    for ra, rb in zip(rings, rings[1:]):
        for i in range(segs):
            j = (i + 1) % segs
            bm.faces.new((ra[i], ra[j], rb[j], rb[i]))
    bm.faces.new(rings[0])
    bm.faces.new(rings[-1])
    return kit.to_object(name, bm, fam, hk.PARENT)


def ball(fam, name, centre, rad):
    bm = bmesh.new()
    ret = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=rad)
    for v in ret["verts"]:
        v.co += hk.P(*centre)
    return kit.to_object(name, bm, fam, hk.PARENT)


def barrel(x, z, h=1.0, rad=0.42, y0=0.0):
    lathe("wood", "barrel", [(rad * 0.86, 0.0), (rad, h * 0.5), (rad * 0.86, h)], (x, y0, z), segs=12)
    for y in (0.18 * h, 0.82 * h):
        lathe("iron", "iron_hoop", [(rad * 0.95, y - 0.05), (rad * 0.97, y + 0.05)], (x, y0, z), segs=12)


def bell_tower(rng):
    """The harbour town's bell tower: a tall square whitewashed shaft with stone quoins up its corners and a stone footing, an arched door,
    a slit window and a clock; an open belfry (an arch on each face between corner piers, the bell hung inside on a beam); a steep
    four-sided spire with rows, hips and a lit eave, a finial and a vane. The town's one landmark."""
    W = 3.0
    h2 = W / 2.0
    belfry0, belfry1, top = 9.15, 11.3, 11.6
    fz = h2
    hk.box("stone", -h2 - 0.3, h2 + 0.3, -0.5, 0.35, -h2 - 0.3, h2 + 0.3, name="footing")
    hk.box("wall", -h2, h2, 0.35, 8.85, -h2, h2, name="wall")
    hk.box("wall", -h2, h2, belfry1, top, -h2, h2, name="wall")
    # The belfry's corner piers, and the arches' heads between them on every face.
    pier = 0.75
    for sx in (-1.0, 1.0):
        for sz in (-1.0, 1.0):
            x0, x1 = sorted((sx * h2, sx * (h2 - pier)))
            z0, z1 = sorted((sz * h2, sz * (h2 - pier)))
            hk.box("wall", x0, x1, belfry0, belfry1, z0, z1, name="wall_pier")
    rr = 0.75
    spring = belfry1 - 0.8
    for q in range(4):
        for side in (-1.0, 1.0):
            pts = [(side * rr, spring)] + [(side * rr * math.cos(math.pi / 2.0 * k / 6), spring + rr * math.sin(math.pi / 2.0 * k / 6)) for k in range(7)][1:]
            pts += [(0.0, belfry1), (side * rr, belfry1)]
            turned("wall", "wall_spandrel", pts, h2 - 0.3, h2, q)
        # The arch's stones, and a sill under the opening.
        for k in range(7):
            a0 = math.pi * k / 7 + 0.03
            a1 = math.pi * (k + 1) / 7 - 0.03
            r0, r1 = rr + 0.02, rr + 0.3
            turned("stone", "stone_arch", [(r0 * math.cos(a0), spring + r0 * math.sin(a0)), (r1 * math.cos(a0), spring + r1 * math.sin(a0)),
                                           (r1 * math.cos(a1), spring + r1 * math.sin(a1)), (r0 * math.cos(a1), spring + r0 * math.sin(a1))], h2, h2 + 0.12, q)
    # The bell on its beam.
    hk.box("timber", -h2 + 0.1, h2 - 0.1, belfry1 - 0.32, belfry1 - 0.12, -0.12, 0.12, name="timber_beam")
    lathe("bell", "bell", [(0.62, 0.0), (0.6, 0.12), (0.46, 0.4), (0.36, 0.8), (0.3, 1.05), (0.12, 1.2)], (0.0, belfry1 - 1.55, 0.0), segs=14)
    # Quoins: stone blocks up every corner, long and short by turns.
    y = 0.35
    k = 0
    while y < top - 0.3:
        for sx in (-1.0, 1.0):
            for sz in (-1.0, 1.0):
                if belfry0 - 0.3 < y < belfry1:
                    continue
                ex, ez = (0.62, 0.3) if k % 2 == 0 else (0.3, 0.62)
                x0, x1 = sorted((sx * (h2 + 0.06), sx * (h2 - ex)))
                z0, z1 = sorted((sz * (h2 + 0.06), sz * (h2 - ez)))
                hk.box("stone", x0, x1, y + 0.03, y + 0.5, z0, z1, name="stone_quoin")
        y += 0.55
        k += 1
    # Bands: a string course, the belfry's sill, the cornice.
    for (y0, y1, out) in ((5.2, 5.45, 0.12), (8.85, belfry0, 0.2), (top, top + 0.35, 0.3)):
        hk.box("trim", -h2 - out, h2 + out, y0, y1, -h2 - out, h2 + out, name="trim_band")
    arched_door(0.0, 1.2, 2.4, fz)
    window(0.0, 4.15, 0.55, 1.0, fz, box_under=False)
    for sx in (-1.0, 1.0):
        side_window(sx, W, 4.15, 0.0, 0.5, 1.0)
    # The clock: a white face in a ring of trim, four marks and two hands.
    cy = 7.1
    face = [(0.72 * math.cos(math.tau * k / 16), cy + 0.72 * math.sin(math.tau * k / 16)) for k in range(16)]
    front("white", "clock", face, fz, fz + 0.08)
    for k in range(16):
        a0, a1 = math.tau * k / 16, math.tau * (k + 1) / 16
        front("trim", "trim_clock", [(0.72 * math.cos(a0), cy + 0.72 * math.sin(a0)), (0.9 * math.cos(a0), cy + 0.9 * math.sin(a0)),
                                     (0.9 * math.cos(a1), cy + 0.9 * math.sin(a1)), (0.72 * math.cos(a1), cy + 0.72 * math.sin(a1))], fz, fz + 0.14)
    for k in range(4):
        a = math.tau * k / 4
        hk.box("timber", 0.56 * math.cos(a) - 0.05, 0.56 * math.cos(a) + 0.05, cy + 0.56 * math.sin(a) - 0.05, cy + 0.56 * math.sin(a) + 0.05,
               fz + 0.08, fz + 0.12, name="timber_mark")
    front("timber", "timber_hand", [(-0.04, cy), (0.04, cy), (0.04, cy + 0.55), (-0.04, cy + 0.55)], fz + 0.08, fz + 0.14)
    front("timber", "timber_hand", [(0.0, cy - 0.04), (0.0, cy + 0.04), (0.38, cy + 0.22), (0.4, cy + 0.15)], fz + 0.08, fz + 0.14)
    # The spire: four thick card faces up to a point, rows across them, rolled hips, a lit eave, a finial and a vane.
    yb = top + 0.35
    ya = yb + 4.4
    e = h2 + 0.55
    for q in range(4):
        tri = [(-e, yb), (e, yb), (0.0, ya)]
        a = q * math.pi / 2.0
        c, sn = math.cos(a), math.sin(a)
        ring0 = [(x * c + (e if y == yb else 0.0) * sn, y, -x * sn + (e if y == yb else 0.0) * c) for (x, y) in tri]
        ring1 = [(px, py - 0.26, pz) for (px, py, pz) in ring0]
        prism("roof", "roof", ring0, ring1)
        for t in (0.22, 0.44, 0.66):
            w = e * (1.0 - t)
            yy = yb + t * (ya - yb)
            p0 = (-w * c + w * sn, yy + 0.04, w * sn + w * c)
            p1 = (w * c + w * sn, yy + 0.04, -w * sn + w * c)
            hk.tube("roof_edge", p0, p1, 0.09, segs=6, name="roof_row")
        corner = (-e * c + e * sn, yb, e * sn + e * c)
        nxt = (e * c + e * sn, yb, -e * sn + e * c)
        hk.tube("roof_edge", corner, (0.0, ya, 0.0), 0.13, segs=6, name="roof_hip")
        hk.tube("roof_edge", corner, nxt, 0.15, segs=6, name="roof_lip")
    lathe("bell", "bell_finial", [(0.05, 0.0), (0.24, 0.12), (0.24, 0.3), (0.05, 0.44)], (0.0, ya - 0.1, 0.0), segs=10)
    hk.box("iron", -0.04, 0.04, ya + 0.3, ya + 1.5, -0.04, 0.04, name="iron_vane")
    front("iron", "iron_vane_tip", [(-0.75, ya + 1.25), (-0.5, ya + 1.15), (-0.5, ya + 1.35)], -0.02, 0.02)
    hk.box("iron", -0.5, 0.45, ya + 1.22, ya + 1.28, -0.03, 0.03, name="iron_vane")
    front("iron", "iron_vane_tail", [(0.25, ya + 1.15), (0.55, ya + 1.15), (0.55, ya + 1.45), (0.25, ya + 1.33)], -0.02, 0.02)
    return Spec(False, e, top, ya - top)


def warehouse(rng):
    """The harbour's warehouse, its gable to the sea: long and low, plank walls in a timber frame on a stone footing, a broad straight roof
    in rows, big braced double doors under a heavy lintel, a loft door in the gable with the hoist beam, rope and hook over it, windows
    each side and along the flanks, a lantern, and crates and a barrel by the doors."""
    W, D, H, rise = 7.0, 11.0, 4.6, 2.6
    f = hk.Frame(True)
    fz = walls(W, D, H)
    roof(f, D, W, H, rise, "straight", rows=5)
    hk.box("trim", -1.85, 1.85, 0.35, 3.9, fz, fz + 0.1, name="trim_doors")
    hk.box("door", -1.65, 1.65, 0.35, 3.65, fz + 0.06, fz + 0.16, name="door")
    for sgn in (-1.0, 1.0):
        for k in range(2):
            sx = sgn * (0.55 + k * 0.55)
            hk.box("door_seam", sx - 0.025, sx + 0.025, 0.4, 3.6, fz + 0.16, fz + 0.18, name="door_seam")
        t = 0.1
        ax, ay, bx, by = sgn * 0.12, 0.6, sgn * 1.5, 3.4
        dx, dy = bx - ax, by - ay
        ln = math.hypot(dx, dy)
        nx, ny = -dy / ln * t, dx / ln * t
        front("trim", "trim_brace", [(ax - nx, ay - ny), (bx - nx, by - ny), (bx + nx, by + ny), (ax + nx, ay + ny)], fz + 0.16, fz + 0.22)
        hk.box("knob", sgn * 0.25 - 0.08, sgn * 0.25 + 0.08, 1.7, 1.86, fz + 0.16, fz + 0.26, name="knob")
    hk.box("timber", -2.05, 2.05, 3.9, 4.2, fz, fz + 0.3, name="timber_lintel")
    # The loft: a door in the gable, the hoist beam over it with a rope and a hook.
    hk.box("trim", -0.85, 0.85, H + 0.25, H + 1.75, fz, fz + 0.1, name="trim_loft")
    hk.box("door", -0.65, 0.65, H + 0.35, H + 1.6, fz + 0.06, fz + 0.14, name="door_loft")
    hk.box("timber", -0.18, 0.18, H + 1.95, H + 2.3, fz - 0.5, fz + 1.7, name="timber_hoist")
    hk.tube("wood_light", (0.0, H + 1.95, fz + 1.45), (0.0, H + 0.55, fz + 1.45), 0.04, segs=6, name="rope")
    hk.box("iron", -0.13, 0.13, H + 0.35, H + 0.55, fz + 1.37, fz + 1.53, name="iron_hook")
    for x in (-2.7, 2.7):
        window(x, 2.3, 0.6, 1.0, fz, box_under=False)
    for sx in (-1.0, 1.0):
        for z in (-3.4, 0.0, 3.0):
            side_window(sx, W, 2.5, z, 0.9, 1.0)
    lantern(2.05, 3.75, fz)
    # Goods by the doors.
    hk.box("crate", -3.3, -2.35, 0.0, 0.95, fz + 0.5, fz + 1.45, name="crate")
    hk.box("crate_dark", -3.33, -2.32, 0.4, 0.55, fz + 0.47, fz + 1.48, name="crate_band")
    hk.box("crate", -3.05, -2.4, 0.95, 1.6, fz + 0.65, fz + 1.3, name="crate")
    barrel(2.85, fz + 1.0, 1.05)
    return Spec(True, D / 2.0 + 0.8, H, rise)


def stall(rng):
    """A market stall on the quay (its foot's middle the origin): four posts, a plank counter with crates of fruit on it, and a striped
    canvas awning sloping to the front with a scalloped valance (`awning` and `awning_b`, each stall its own colours)."""
    zb, zf = -1.1, 1.1
    for px in (-1.6, 1.6):
        hk.box("wood_dark", px - 0.09, px + 0.09, 0.0, 2.3, zf - 0.09, zf + 0.09, name="post")
        hk.box("wood_dark", px - 0.09, px + 0.09, 0.0, 2.8, zb - 0.09, zb + 0.09, name="post")
    hk.box("wood", -1.5, 1.5, 0.0, 0.85, -0.05, 1.0, name="counter")
    hk.box("wood_light", -1.62, 1.62, 0.85, 0.97, -0.1, 1.1, name="counter_top")
    for k in range(5):
        x = -1.0 + 0.5 * k
        hk.box("door_seam", x - 0.025, x + 0.025, 0.08, 0.8, 1.0, 1.03, name="door_seam")
    for (x0, x1, fam) in ((-1.35, -0.45, "red"), (0.3, 1.2, "heart")):
        hk.box("crate", x0, x1, 0.97, 1.32, 0.15, 0.9, name="crate")
        hk.box("crate_dark", x0 - 0.02, x1 + 0.02, 1.12, 1.2, 0.13, 0.92, name="crate_band")
        for j in range(7):
            ball(fam, "fruit", (rng.uniform(x0 + 0.2, x1 - 0.2), 1.42 + rng.uniform(0.0, 0.08), rng.uniform(0.35, 0.7)), 0.16)
    # The awning: stripes from the back posts down to the front, and the valance's scallops.
    x0, x1 = -1.85, 1.85
    n = 6
    sw = (x1 - x0) / n
    yb, yf = 2.95, 2.45
    za, zz = zb - 0.2, zf + 0.25
    for j in range(n):
        fam = "awning" if j % 2 == 0 else "awning_b"
        xa, xb = x0 + j * sw, x0 + (j + 1) * sw
        top = [(xa, yb, za), (xb, yb, za), (xb, yf, zz), (xa, yf, zz)]
        prism(fam, "awning", top, [(px, py - 0.08, pz) for (px, py, pz) in top])
        hem = [(xa + (xb - xa) * u / 6.0, yf - 0.3 - 0.12 * math.sin(math.pi * u / 6.0)) for u in range(7)]
        front(fam, "awning_valance", [(xa, yf), (xb, yf)] + list(reversed(hem))[0:1] + list(reversed(hem))[1:], zz - 0.06, zz)
    return Spec(False, 2.0, 2.4, 0.5)


def stilt_hut(rng):
    """The fisherman's hut on stilts over the sea (the origin on the water under its middle): posts down into the sea with a brace, a deck
    of planks on a joist, a board hut with battens, a plank door, a shuttered window, a shaggy thatched roof in three ragged skirts, a
    rail along the deck's front with a net hung over it, a barrel; and a jetty of planks off to the right on pairs of posts."""
    deck = 4.0
    for px in (-2.7, 2.7):
        for pz in (-1.9, 1.9):
            hk.tube("wood_dark", (px, -0.8, pz), (px + rng.uniform(-0.08, 0.08), deck - 0.3, pz), 0.22, segs=7, name="post")
    brace(-2.5, 2.5, 0.7, 3.4, 2.1)
    hk.box("wood_dark", -3.25, 3.25, deck - 0.52, deck - 0.28, -2.2, 2.2, name="joist")
    x = -3.2
    while x < 3.2:
        j = rng.uniform(-0.03, 0.03)
        hk.box("wood", x + 0.03, x + 0.55, deck - 0.28, deck + j, -2.35, 2.35, name="plank")
        x += 0.58
    x0, x1, z0, z1 = -1.9, 2.7, -1.9, 1.5
    top = deck + 2.9
    hk.box("wall", x0, x1, deck, top, z0, z1, name="wall")
    for sx in (x0, x1):
        for sz in (z0, z1):
            hk.box("timber", sx - 0.18, sx + 0.18, deck, top, sz - 0.18, sz + 0.18, name="timber_post")
    xb = x0 + 0.5
    while xb < x1 - 0.3:
        if not (-1.5 < xb < -0.3) and not (0.6 < xb < 1.9):
            hk.box("door_seam", xb - 0.03, xb + 0.03, deck + 0.1, top - 0.1, z1, z1 + 0.05, name="door_seam")
        xb += 0.5
    hk.box("trim", -1.5, -0.3, deck, deck + 2.15, z1, z1 + 0.08, name="trim_door")
    hk.box("door", -1.35, -0.45, deck, deck + 2.0, z1 + 0.04, z1 + 0.12, name="door")
    hk.box("knob", -0.67, -0.55, deck + 1.0, deck + 1.12, z1 + 0.12, z1 + 0.2, name="knob")
    window(1.25, deck + 1.75, 0.8, 0.75, z1, box_under=False)
    shutters(1.25, deck + 1.75, 0.8, 0.75, z1)
    # The thatch: three skirts, each overlapping the one below, their hems ragged.
    cx, cz = 0.4, -0.2
    for k, (rb, rt, y0, y1) in enumerate(((4.5, 3.0, top - 0.3, top + 0.9), (3.5, 1.9, top + 0.75, top + 1.95), (2.4, 0.12, top + 1.8, top + 3.0))):
        jit = [(rng.uniform(-0.12, 0.12), rng.uniform(-0.22, 0.06)) for _ in range(16)]
        lathe("straw", "thatch", [(rb - 0.4, 0.22), (rb, 0.0), (rt, y1 - y0), (max(0.05, rt - 0.4), y1 - y0 + 0.05)], (cx, y0, cz), segs=16,
              squash=0.85, jitter=lambda i, kk, jit=jit: jit[i] if kk < 2 else (0.0, 0.0))
        lathe("straw_end", "thatch_hem", [(rb - 0.05, -0.02), (rb + 0.06, 0.18)], (cx, y0, cz), segs=16, squash=0.85,
              jitter=lambda i, kk, jit=jit: jit[i])
    # The rail along the deck's front with the net over it, its corks along the hem.
    for xr in (-3.0, -1.8, -0.6, 0.6, 1.8, 3.0):
        hk.box("wood_dark", xr - 0.07, xr + 0.07, deck, deck + 1.0, 2.15, 2.29, name="rail_post")
    hk.box("wood_dark", -3.1, 3.1, deck + 0.9, deck + 1.04, 2.14, 2.3, name="rail")
    hem = [(-2.8 + 2.4 * u / 8.0, deck + 0.05 + 0.12 * math.sin(math.pi * u / 2.0)) for u in range(9)]
    front("net", "net", [(-2.8, deck + 1.04), (-0.4, deck + 1.04)] + list(reversed(hem)), 2.31, 2.36)
    for (hx, hy) in hem[1::2]:
        hk.box("red", hx - 0.1, hx + 0.1, hy - 0.08, hy + 0.08, 2.32, 2.42, name="cork")
    barrel(-2.4, 0.2, 0.9, 0.36, deck)
    # The jetty: planks off to the right, two stringers, pairs of posts.
    x = 3.25
    while x < 25.0:
        j = rng.uniform(-0.04, 0.03)
        hk.box("wood", x, x + 0.5, deck - 0.24 + j, deck - 0.04 + j, 0.4, 2.0, name="plank")
        x += 0.56
    for (za, zb) in ((0.45, 0.62), (1.78, 1.95)):
        hk.box("wood_dark", 3.2, 25.0, deck - 0.46, deck - 0.24, za, zb, name="stringer")
    for px in (5.0, 9.0, 13.0, 17.0, 21.0, 24.6):
        for pz in (0.5, 1.9):
            hk.tube("wood_dark", (px, -0.8, pz), (px, deck - 0.3, pz), 0.15, segs=6, name="post")
    return Spec(False, 3.2, top, 3.0)


class Spec:
    """What `hand_made` needs to bend a house: its ridge's axis, half its length, the wall top and the roof's rise."""

    def __init__(self, gable_front, half_along, H, rise):
        self.g, self.la, self.H, self.rise = gable_front, half_along, H, rise


def hand_made(spec, parent, rng):
    """Nothing quite straight (option C): the ridge sags toward its middle, the walls lean a touch, the roof's rows ripple."""
    lean = rng.uniform(0.1, 0.16) * rng.choice((-1.0, 1.0))
    for o in [o for o in bpy.data.objects if o.parent == parent and o.type == "MESH"]:
        for v in o.data.vertices:
            x, y, z = v.co.x, v.co.z, -v.co.y
            a = z if spec.g else x
            u = max(0.0, min(1.0, abs(a) / max(spec.la, 0.01)))
            if y > spec.H:
                k = min(1.0, (y - spec.H) / max(spec.rise, 0.01))
                y -= 0.38 * (1.0 - u * u) * k
                y += 0.06 * math.sin(a * 2.7 + y * 1.3)
            x += lean * max(0.0, y) / (spec.H + spec.rise)
            v.co.x, v.co.y, v.co.z = x, -z, y


def cottage(rng):
    W, D, H, rise = 5.4, 4.2, 3.3, 3.0
    f = hk.Frame(False)
    fz = walls(W, D, H)
    base_course(W, fz, [(-1.25 - 0.95, -1.25 + 0.95)], rng)
    roof(f, W, D, H, rise, "bell" if SHAPE in "BC" else "straight")
    arched_door(-1.25, 1.3, 2.6, fz)
    window(1.35, 1.95, 1.3, 1.3, fz)
    chimney(1.7, -0.9, H, H + rise + 1.2)
    if DETAIL:
        shutters(1.35, 1.95, 1.3, 1.3, fz)
        for sx in (-1.0, 1.0):
            side_window(sx, W, 1.95, -0.2, 1.0, 1.1)
        lantern(-0.25, 2.65, fz)
        picket_fence(-W / 2.0 - 0.3, W / 2.0 + 0.3, fz, gate=(-2.0, -0.5))
        flower_pot(0.4, fz)
    return Spec(False, W / 2.0 + 0.8, H, rise)


def townhouse(rng):
    W, D, H, rise = 4.4, 4.8, 6.0, 3.2
    f = hk.Frame(True)
    fz = walls(W, D, H)
    hk.box("timber", -W / 2.0, W / 2.0, 3.0, 3.26, fz, fz + 0.12, name="timber_beam")
    base_course(W, fz, [(-0.9 - 0.95, -0.9 + 0.95)], rng)
    roof(f, D, W, H, rise, "bell" if SHAPE in "BC" else "straight")
    arched_door(-0.9, 1.2, 2.6, fz)
    window(1.2, 1.95, 1.0, 1.2, fz)
    for x in (-1.05, 1.1):
        window(x, 4.55, 1.0, 1.25, fz)
    brace(-0.35, 0.4, 3.35, 5.8, fz)
    chimney(-1.2, -1.5, H, H + rise + 1.4)
    if DETAIL:
        shutters(1.2, 1.95, 1.0, 1.2, fz)
        for sx in (-1.0, 1.0):
            side_window(sx, W, 4.55, 0.0, 0.9, 1.1)
            side_window(sx, W, 1.95, 0.6, 0.9, 1.1)
        lantern(0.05, 2.65, fz)
        flower_pot(1.75, fz)
    return Spec(True, D / 2.0 + 0.8, H, rise)


def longhouse(rng):
    W, D, H, rise = 8.4, 4.4, 3.2, 2.8
    f = hk.Frame(False)
    fz = walls(W, D, H)
    base_course(W, fz, [(-1.0, 1.0)], rng)
    roof(f, W, D, H, rise, "gambrel" if SHAPE in "BC" else "straight")
    arched_door(0.0, 1.4, 2.6, fz)
    for x in (-2.75, 2.75):
        window(x, 1.95, 1.4, 1.3, fz)
    for sx in (-1.0, 1.0):
        brace(sx * 1.05, sx * 1.75, 0.95, 2.85, fz)
    chimney(-3.1, -0.8, H, H + rise + 1.0)
    if DETAIL:
        for x in (-2.75, 2.75):
            shutters(x, 1.95, 1.4, 1.3, fz)
        for sx in (-1.0, 1.0):
            side_window(sx, W, 1.95, 0.0, 1.0, 1.1)
        lantern(1.05, 2.65, fz)
        picket_fence(-W / 2.0 - 0.3, W / 2.0 + 0.3, fz, gate=(-0.8, 0.8))
    return Spec(False, W / 2.0 + 0.8, H, rise)


def barn(rng):
    W, D, H, rise = 6.2, 6.0, 3.4, 3.2
    f = hk.Frame(True)
    hk.box("stone", -W / 2.0 - 0.18, W / 2.0 + 0.18, -0.5, 0.35, -D / 2.0 - 0.18, D / 2.0 + 0.18, name="footing")
    hk.box("barn", -W / 2.0, W / 2.0, 0.35, H, -D / 2.0, D / 2.0, name="wall_barn")
    fz = D / 2.0
    for sx in (-1.0, 1.0):
        hk.box("barn_trim", sx * W / 2.0 - 0.18, sx * W / 2.0 + 0.18, 0.35, H, fz - 0.18, fz + 0.1, name="trim_corner")
    roof(f, D, W, H, rise, "gambrel" if SHAPE in "BC" else "straight")
    hk.box("barn_trim", -1.55, 1.55, 0.35, 3.25, fz, fz + 0.06, name="trim_doors")
    hk.box("barn_door", -1.35, 1.35, 0.35, 3.05, fz + 0.04, fz + 0.1, name="door_barn")
    for s in (-1.0, 1.0):
        t = 0.1
        ax, ay, bx, by = -1.25 * s, 0.5, 1.25 * s, 2.9
        dx, dy = bx - ax, by - ay
        ln = math.hypot(dx, dy)
        nx, ny = -dy / ln * t, dx / ln * t
        front("barn_trim", "trim_brace", [(ax - nx, ay - ny), (bx - nx, by - ny), (bx + nx, by + ny), (ax + nx, ay + ny)], fz + 0.1, fz + 0.17)
    hk.box("barn_trim", -0.75, 0.75, H + 0.3, H + 1.7, fz + 0.05, fz + 0.12, name="trim_loft")
    hk.box("shadow", -0.58, 0.58, H + 0.45, H + 1.55, fz + 0.1, fz + 0.15, name="shadow_loft")
    if DETAIL:
        # Hay spilling from the loft, a window in each side, a weathervane on the ridge.
        hk.box("straw", -0.55, 0.55, H + 0.45, H + 0.85, fz + 0.12, fz + 0.4, name="straw_loft")
        front("straw", "straw_wisp", [(-0.5, H + 0.85), (0.5, H + 0.85), (0.15, H + 1.15), (-0.2, H + 1.05)], fz + 0.15, fz + 0.3)
        for sx in (-1.0, 1.0):
            side_window(sx, W, 2.0, 0.0, 1.0, 1.0)
        top = H + rise
        hk.box("iron", -0.04, 0.04, top, top + 1.2, -0.04, 0.04, name="iron_vane")
        hk.box("iron", -0.5, 0.5, top + 0.95, top + 1.02, -0.03, 0.03, name="iron_vane")
        front("iron", "iron_vane_tail", [(0.3, top + 0.9), (0.6, top + 0.9), (0.6, top + 1.2), (0.3, top + 1.08)], -0.02, 0.02)
        front("iron", "iron_vane_tip", [(-0.6, top + 0.98), (-0.4, top + 0.9), (-0.4, top + 1.06)], -0.02, 0.02)
    return Spec(True, D / 2.0 + 0.8, H, rise)


HOUSES = [("cottage", cottage), ("townhouse", townhouse), ("longhouse", longhouse), ("barn", barn), ("belltower", bell_tower),
          ("warehouse", warehouse), ("stall", stall), ("stilthut", stilt_hut)]


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    roots = []
    rng = random.Random(31)
    for k, (name, build) in enumerate(HOUSES):
        # (Apart for the bake, so one house does not shade the next; back on the origin for export.)
        root = kit.empty("prop_house_" + name, k * 16.0)
        hk.PARENT = root
        spec = build(rng)
        if SHAPE == "C":
            hand_made(spec, root, rng)
        roots.append(root)
    for o in [o for o in bpy.context.scene.objects if o.type == "MESH"]:
        bm = bmesh.new()
        bm.from_mesh(o.data)
        for _ in range(3):
            long_edges = [e for e in bm.edges if e.calc_length() > 0.7]
            if not long_edges:
                break
            bmesh.ops.subdivide_edges(bm, edges=long_edges, cuts=1, use_grid_fill=True)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        bm.to_mesh(o.data)
        bm.free()
    # (Each house's pieces joined by material: with the detail pass every shutter, picket and lantern was its own mesh, and a town of
    # them overflowed the game's per-instance shader buffer, so other scenery took stray glows and edges.)
    for root in roots:
        groups = {}
        for o in [o for o in bpy.data.objects if o.parent == root and o.type == "MESH"]:
            mat = o.data.materials[0].name if o.data.materials else ""
            groups.setdefault(mat, []).append(o)
        for mat, objs in groups.items():
            if len(objs) < 2:
                continue
            bpy.ops.object.select_all(action="DESELECT")
            for o in objs:
                o.select_set(True)
            bpy.context.view_layer.objects.active = objs[0]
            bpy.ops.object.join()
            objs[0].name = mat
    bpy.context.view_layer.update()
    # (Under every house, the stilt hut's jetty included.)
    kit.bake_occlusion(0.7, floor_z=0.0, floor_at=(len(HOUSES) * 8.0 + 4.0, 0.0), floor_size=len(HOUSES) * 16.0 + 80.0)
    if "render" in opts:
        hk.PREVIEW.update({"glass": (0.36, 0.6, 0.86), "door_seam": (0.36, 0.24, 0.16), "knob": (0.92, 0.9, 0.86), "bell": (0.86, 0.66, 0.3),
                           "awning": (0.92, 0.3, 0.28), "awning_b": (0.98, 0.95, 0.9), "net": (0.36, 0.52, 0.5), "straw": (0.86, 0.7, 0.42),
                           "straw_end": (0.76, 0.6, 0.34), "white": (0.98, 0.96, 0.92), "wood": (0.74, 0.52, 0.32), "wood_dark": (0.46, 0.34, 0.26),
                           "wood_light": (0.9, 0.72, 0.48), "crate": (0.86, 0.66, 0.42), "crate_dark": (0.68, 0.48, 0.3), "iron": (0.36, 0.34, 0.42),
                           "red": (1.0, 0.45, 0.42), "heart": (1.0, 0.85, 0.3), "lamp": (1.0, 0.86, 0.5)})
        hk.render(opts["render"], cx=len(HOUSES) * 8.0 + 2.0, scale=len(HOUSES) * 16.0 + 30.0, width=len(HOUSES) * 400)
    for r in roots:
        r.location = (0.0, 0.0, 0.0)
    bpy.context.view_layer.update()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("paper houses v2 %s: %d triangles -> %s" % (SHAPE, tris, out))


main()
