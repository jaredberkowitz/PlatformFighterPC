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

Run (Blender 5.2):
  blender --background --python art/blender/make_paper_houses_v2.py -- --shape A --out <file.glb> [--render <sheet.png>]
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


def chimney(x, z, y0, top):
    hk.box("stone", x - 0.42, x + 0.42, y0, top, z - 0.42, z + 0.42, name="chimney")
    hk.box("stone", x - 0.56, x + 0.56, top, top + 0.24, z - 0.56, z + 0.56, name="chimney_cap")


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
    return Spec(True, D / 2.0 + 0.8, H, rise)


HOUSES = [("cottage", cottage), ("townhouse", townhouse), ("longhouse", longhouse), ("barn", barn)]


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
    bpy.context.view_layer.update()
    kit.bake_occlusion(0.7, floor_z=0.0, floor_at=(24.0, 0.0), floor_size=160.0)
    if "render" in opts:
        hk.PREVIEW.update({"glass": (0.36, 0.6, 0.86), "door_seam": (0.36, 0.24, 0.16), "knob": (0.92, 0.9, 0.86)})
        hk.render(opts["render"])
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
