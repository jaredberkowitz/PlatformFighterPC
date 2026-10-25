"""A stage's play area as paper card (the paper pilot rolled out after the meadow, `make_meadow_play.py`): the top the fighters stand on and
the platforms, on the collision from the bake manifest. Every walkable top stays exactly at the sim's height (tests/stage_front_test.gd
checks it); the modelled front under the top stays. Three looks per stage, as on the meadow (`--play-mock=` on the stage demo):
  A  card slab: the top one thick card with a pale cut edge; the platforms as card boards (wood) or a card plate (girders);
  B  layered sheets: the top a thin sheet over two more, each a step wider, so the edge shows the stage's own layers; the platforms a
     top sheet over a body card, a cut fringe hanging from the front where the stage has one;
  C  pop-up: the top a thin sheet behind a standing ground-row card cut in the stage's own edge (grass, rock, a board, a cornice); the
     platforms as cut-out fronts over a soft shadow card.
`--theme` picks the stage's materials and shapes: grove (lawn, moss, earth; a path), sunset (sand over strata), ocean (the dock's boards
over joists and a beam), city (roofing with a coping, girders).
Families `top`, `top_card`, `layer1`, `layer2`, `fringe`, `path`, `plank`, `plank_b`, `plank_card`, `rope`, `steel`, `steel_dark`,
`steel_card`, `shadow` (painted per theme by `StageBake.place_play`).

Shipped since fidelity review 7 (the owner's picks, 2026-10-22), each with `--wear 2`, then lit: grove_play.glb `--theme grove --variant B
--plat islet`, canyon_play.glb `--theme sunset --variant B --plat rock`, beach_play.glb `--theme ocean --variant A`, city_play.glb
`--theme city --variant A`.

Run (Blender 5.2):
  blender --background --python art/blender/make_play.py -- --dir godot/art/baked/<key> --theme grove --variant A --out <file.glb>
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
# The platforms after fidelity review 6 (`--plat islet|islet_moss` on the grove, `rock|rock_under` on the canyon) and review 8
# (`deck|boards` on the city).
PLAT = opts.get("plat", "")
WEAR = int(opts.get("wear", "0"))
THEME = opts.get("theme", "grove")
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
FZ = float(manifest["front_z"])
SOLIDS = [r for r in manifest["rects"] if r[4] < 0.5]
TOP_Y = max(r[2] for r in SOLIDS)
TOPS = [r for r in SOLIDS if abs(r[2] - TOP_Y) < 0.01]
PLATFORMS = [r for r in manifest["rects"] if r[4] > 0.5]
DEPTH = 8.0
PLAT_DEPTH = 4.0
GIRDER = THEME == "city"
# The cut edge each stage's ground row and fringes take: grass scallops, rock jags, a straight board, a stepped cornice.
EDGE = {"grove": "scallop", "sunset": "jag", "ocean": "board", "city": "cornice"}[THEME]


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
    """A card box, wound outward: its top in `top`, its sides in `side` (the cut edge), its underside in `bottom` (or `side`)."""
    c = [P(x, y, z) for x in (x0, x1) for y in (y0, y1) for z in (z0, z1)]

    def v(xi, yi, zi):
        return c[xi * 4 + yi * 2 + zi]
    face(top, (v(0, 1, 1), v(1, 1, 1), v(1, 1, 0), v(0, 1, 0)))
    face(bottom or side, (v(0, 0, 0), v(1, 0, 0), v(1, 0, 1), v(0, 0, 1)))
    face(side, (v(0, 0, 1), v(1, 0, 1), v(1, 1, 1), v(0, 1, 1)))
    face(side, (v(1, 0, 0), v(0, 0, 0), v(0, 1, 0), v(1, 1, 0)))
    face(side, (v(0, 0, 0), v(0, 0, 1), v(0, 1, 1), v(0, 1, 0)))
    face(side, (v(1, 0, 1), v(1, 0, 0), v(1, 1, 0), v(1, 1, 1)))


def edge_row(x0, x1, y_top, y_low, z, mat, up=True, size=0.5, rng=None):
    """A cut card edge along x at depth z, from y_low up to y_top (`up`), or hanging from y_top down to y_low: grass scallops, rock jags,
    a straight board with notched ends, or a stepped cornice, as the stage's EDGE."""
    if EDGE == "board":
        prof = [(x0, y_top), (x1, y_top), (x1, y_low + 0.1), (x1 - 0.25, y_low + 0.1), (x1 - 0.25, y_low), (x0 + 0.25, y_low),
                (x0 + 0.25, y_low + 0.1), (x0, y_low + 0.1)]
        face(mat, [P(x, y, z) for (x, y) in prof])
        return
    if EDGE == "cornice":
        steps = [(0.0, 0.0), (0.12, -0.18), (0.24, -0.36)]
        for (inset, dy) in steps:
            face(mat, (P(x0 + inset, y_low, z + 0.02 * steps.index((inset, dy))), P(x1 - inset, y_low, z + 0.02 * steps.index((inset, dy))),
                       P(x1 - inset, y_top + dy, z + 0.02 * steps.index((inset, dy))), P(x0 + inset, y_top + dy, z + 0.02 * steps.index((inset, dy)))))
        return
    n = max(2, int((x1 - x0) / size))
    w = (x1 - x0) / n
    for i in range(n):
        xa, xb = x0 + w * i, x0 + w * (i + 1)
        if EDGE == "jag":
            xm = xa + w * (rng.uniform(0.3, 0.7) if rng else 0.5)
            peak = w * (rng.uniform(0.35, 0.8) if rng else 0.5)
        else:
            xm, peak = (xa + xb) * 0.5, w * 0.45
        if up:
            face(mat, (P(xa, y_low, z), P(xb, y_low, z), P(xb, y_top - peak, z), P(xm, y_top, z), P(xa, y_top - peak, z)))
        else:
            face(mat, (P(xa, y_top, z), P(xa, y_low + peak, z), P(xm, y_low, z), P(xb, y_low + peak, z), P(xb, y_top, z)))


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
    xs = [x0 + (x1 - x0) * i / 60 for i in range(61)]
    ph = rng.uniform(0, 6)
    for i in range(60):
        xa, xb = xs[i], xs[i + 1]

        def edge(x, s):
            return zc + s * (half + 0.12 * math.sin(x * 0.9 + ph) + 0.07 * math.sin(x * 2.3 + ph * 2.0))
        face("path", (P(xa, TOP_Y + 0.006, edge(xa, -1)), P(xa, TOP_Y + 0.006, edge(xa, 1)), P(xb, TOP_Y + 0.006, edge(xb, 1)),
                      P(xb, TOP_Y + 0.006, edge(xb, -1))))


def deck_boards(x0, x1, top, zb, zf, thick):
    """The dock's boards: card planks running along x, a narrow gap between, over two joists."""
    n = 7
    gap = 0.06
    bw = (zf - zb - gap * (n - 1)) / n
    for k in range(n):
        za = zb + k * (bw + gap)
        box(x0, x1, top - thick, top, za, za + bw, "top", "top_card", "layer1")
    for jz in (zb + 0.8, zf - 0.8):
        box(x0 + 0.2, x1 - 0.2, top - thick - 0.4, top - thick, jz - 0.25, jz + 0.25, "layer1", "layer1")


def top_area(rng):
    for r in TOPS:
        x0, x1, top = r[0] - 0.12, r[1] + 0.12, r[2]
        zb, zf = FZ - DEPTH - 0.05, FZ + 0.06
        if VARIANT == "A":
            if THEME == "ocean":
                deck_boards(x0, x1, top, zb, zf, 0.3)
            else:
                box(x0, x1, top - 0.5, top, zb, zf, "top", "top_card", "layer1")
            if THEME == "city":
                # The coping: a pale card strip along the front edge, flush with the roof.
                box(x0 - 0.1, x1 + 0.1, top - 0.32, top, zf - 0.45, zf + 0.12, "top_card", "top_card")
        elif VARIANT == "B":
            if THEME == "ocean":
                deck_boards(x0, x1, top, zb, zf, 0.2)
                box(x0 - 0.12, x1 + 0.12, top - 0.95, top - 0.6, zb, zf + 0.12, "layer2", "layer2")
            else:
                box(x0, x1, top - 0.2, top, zb, zf, "top", "top_card", "layer1")
                box(x0 - 0.12, x1 + 0.12, top - 0.55, top - 0.2, zb, zf + 0.12, "layer1", "layer1")
                box(x0 - 0.24, x1 + 0.24, top - 0.95, top - 0.55, zb, zf + 0.24, "layer2", "layer2")
        else:
            if THEME == "ocean":
                deck_boards(x0, x1, top, zb, zf, 0.15)
            else:
                box(x0, x1, top - 0.15, top, zb, zf, "top", "top_card", "layer1")
            # The ground row in front of the top's edge, never above the walking surface.
            edge_row(x0 - 0.2, x1 + 0.2, top - 0.02, top - 1.4, zf + 0.35, "fringe", up=True, size=0.55, rng=rng)
        if THEME == "grove":
            path_strip(r[0] + 0.2, r[1] - 0.2, FZ - 1.9, 1.0, rng)
        if WEAR:
            # (Its own random numbers, so the platforms and everything after keep their layout.)
            wr = random.Random(91 + int(r[0] * 10))
            if THEME == "ocean":
                # Weathered boards: patches of grey sun-bleach and darker wet wood, a replaced board or two, nail heads.
                wear_top(x0 + 0.3, x1 - 0.3, top, zb, zf, wr, [("top_worn", 8, (0.6, 1.6)), ("top_deep", 6, (0.4, 1.0))])
                for _ in range(2):
                    bx = wr.uniform(x0 + 2.0, x1 - 4.0)
                    k = wr.randrange(1, 6)
                    gap = 0.06
                    bw = (zf - zb - gap * 6) / 7
                    za = zb + k * (bw + gap)
                    face("top_lit", (P(bx, top + 0.006, za + 0.04), P(bx, top + 0.006, za + bw - 0.04), P(bx + 3.0, top + 0.006, za + bw - 0.04),
                                     P(bx + 3.0, top + 0.006, za + 0.04)))
                if WEAR >= 2:
                    board_marks(x0, x1, top, zb, zf, 7, wr)
            elif THEME == "sunset":
                # Wind-laid sand: darker drifts and pale ripples; pebbles along the edge.
                wear_top(x0 + 0.3, x1 - 0.3, top, zb, zf, wr, [("top_deep", 10, (0.6, 1.6)), ("top_lit", 12, (0.5, 1.4))])
                if WEAR >= 2:
                    edge_pebbles(x0, x1, top, zf, wr)
            elif THEME == "city":
                # The roof's patched felt: darker tar patches and paler sun-faded sheets.
                wear_top(x0 + 0.3, x1 - 0.3, top, zb, zf, wr, [("top_deep", 7, (0.6, 1.4)), ("top_lit", 5, (0.8, 1.6))])
            else:
                wear_top(x0 + 0.3, x1 - 0.3, top, zb, zf, wr, [("top_deep", 22, (0.6, 1.8)), ("top_lit", 16, (0.7, 1.9)), ("top_worn", 4, (0.4, 0.9))],
                         path_z=FZ - 1.9, path_half=1.0)
                if WEAR >= 2:
                    edge_tufts(x0, x1, top, zf, wr)


def platform(r, rng):
    x0, x1, top = r[0], r[1], r[2]
    zb, zf = FZ - PLAT_DEPTH, FZ
    if PLAT in ("islet", "islet_moss") and THEME == "grove":
        # (Fidelity review 6: the grove's platforms as small islands of the stage's own island: turf over a band of earth, a rocky
        # underside hanging under it, roots; `islet_moss` deeper, with moss hanging and more roots.)
        box(x0 - 0.1, x1 + 0.1, top - 0.14, top, zb, zf, "top", "top_card", "layer1")
        box(x0 - 0.02, x1 + 0.02, top - 0.42, top - 0.14, zb + 0.05, zf - 0.03, "layer2", "layer2")
        deep = PLAT == "islet_moss"
        # (In the island's dark earth: in the pale stone it read as a paper skirt, not rock.)
        underside(x0 + 0.05, x1 - 0.05, top - 0.42, zb + 0.15, zf - 0.08, 1.1 if deep else 0.75, rng, "layer2", "layer2")
        edge_row(x0 - 0.1, x1 + 0.1, top - 0.08, top - 0.5, zf + 0.04, "fringe", up=False, size=0.4, rng=rng)
        hanging(x0 + 0.3, x1 - 0.3, top - 0.45, zf - 0.2, "plank_b", rng, 5 if deep else 3, 0.5, 1.2, (0.05, 0.1))
        if deep:
            hanging(x0, x1, top - 0.4, zf + 0.05, "fringe", rng, int((x1 - x0) * 1.4), 0.3, 0.75)
        return
    if PLAT in ("rock", "rock_under") and THEME == "sunset":
        # (Fidelity review 6: the canyon's platforms a bare board. A slab of the stage's own sandstone: a sand top over two strata courses,
        # the lower set in and broken short at one end; `rock_under` with a tapered rock hanging under it, a piece of mesa.)
        box(x0 - 0.1, x1 + 0.1, top - 0.1, top, zb, zf, "top", "top_card", "layer1")
        box(x0 - 0.06, x1 + 0.06, top - 0.38, top - 0.1, zb + 0.02, zf + 0.02, "layer1", "layer1")
        cut_l, cut_r = rng.uniform(0.05, 0.45), rng.uniform(0.05, 0.45)
        box(x0 + cut_l, x1 - cut_r, top - 0.66, top - 0.4, zb + 0.1, zf - 0.06, "layer2", "layer2")
        if PLAT == "rock_under":
            underside(x0 + 0.2, x1 - 0.2, top - 0.66, zb + 0.2, zf - 0.12, 1.0, rng, "layer1", "layer2")
        return
    if PLAT in ("deck", "boards") and GIRDER:
        # (Fidelity review 8: the city's platforms a plain dark card on the scaffold. In the city's own materials: `deck` a steel deck of
        # plates with their seams, a painted safety line along the front, a riveted girder in red-oxide primer; `boards` scaffold boards laid on
        # steel tubes, clamped, over a toe board painted with hazard stripes. Nothing rises above the walking surface.)
        if PLAT == "deck":
            box(x0, x1, top - 0.1, top, zb, zf, "steel", "steel_card", "steel_dark")
            n = max(2, round((x1 - x0) / 1.8))
            w = (x1 - x0) / n
            for i in range(1, n):
                sx = x0 + w * i
                face("steel_seam", (P(sx - 0.03, top + 0.004, zf - 0.02), P(sx + 0.03, top + 0.004, zf - 0.02),
                                    P(sx + 0.03, top + 0.004, zb + 0.02), P(sx - 0.03, top + 0.004, zb + 0.02)))
            face("steel_paint", (P(x0 + 0.05, top + 0.005, zf - 0.05), P(x1 - 0.05, top + 0.005, zf - 0.05),
                                 P(x1 - 0.05, top + 0.005, zf - 0.3), P(x0 + 0.05, top + 0.005, zf - 0.3)))
            # The girder under the deck's front edge in red-oxide primer (a deep channel went black in its own shade), its flanges standing
            # a little proud, and a joist under every seam.
            box(x0 + 0.04, x1 - 0.04, top - 0.62, top - 0.1, zf - 0.16, zf, "steel_dark", "steel_primer", "steel_dark")
            box(x0 + 0.04, x1 - 0.04, top - 0.18, top - 0.1, zf - 0.16, zf + 0.04, "steel_primer", "steel_primer", "steel_dark")
            box(x0 + 0.04, x1 - 0.04, top - 0.62, top - 0.54, zf - 0.16, zf + 0.04, "steel_primer", "steel_primer", "steel_dark")
            for i in range(n + 1):
                jx = min(max(x0 + w * i, x0 + 0.2), x1 - 0.2)
                box(jx - 0.08, jx + 0.08, top - 0.5, top - 0.1, zb + 0.2, zf - 0.16, "steel_dark", "steel_dark")
            k = 0.45
            rx = x0 + 0.25
            while rx < x1 - 0.2:
                for ry in (top - 0.27, top - 0.45):
                    box(rx - 0.04, rx + 0.04, ry - 0.04, ry + 0.04, zf - 0.02, zf + 0.025, "rivet", "rivet")
                rx += k
        else:
            gap = 0.07
            nb = 4
            bw = (zf - zb - gap * (nb - 1)) / nb
            for k in range(nb):
                za = zb + k * (bw + gap)
                ea, eb = rng.uniform(-0.25, 0.05), rng.uniform(-0.05, 0.25)
                box(x0 + ea, x1 + eb, top - 0.12, top, za, za + bw, "deck_board", "plank_card", "plank_b")
                # The steel hoops round each board's ends.
                for hx in (x0 + ea + 0.18, x1 + eb - 0.18):
                    box(hx - 0.05, hx + 0.05, top - 0.13, top + 0.006, za - 0.005, za + bw + 0.005, "rivet", "rivet")
            # The transoms under the boards, and the ledger along the front with a clamp where each crosses it.
            for tx in (x0 + 0.45, (x0 + x1) * 0.5, x1 - 0.45):
                box(tx - 0.09, tx + 0.09, top - 0.3, top - 0.12, zb + 0.05, zf + 0.05, "steel", "steel_card", "steel_dark")
                box(tx - 0.14, tx + 0.14, top - 0.36, top - 0.1, zf + 0.06, zf + 0.12, "rivet", "rivet")
            box(x0 - 0.15, x1 + 0.15, top - 0.36, top - 0.18, zf - 0.08, zf + 0.06, "steel", "steel_card", "steel_dark")
            # The toe board, hung below the boards' front, painted in hazard stripes.
            box(x0 + 0.1, x1 - 0.1, top - 0.68, top - 0.36, zf - 0.04, zf + 0.04, "steel_paint", "steel_paint", "plank_b")
            sx = x0 + 0.25
            while sx < x1 - 0.5:
                face("hazard", (P(sx, top - 0.66, zf + 0.045), P(sx + 0.22, top - 0.66, zf + 0.045),
                                P(sx + 0.42, top - 0.38, zf + 0.045), P(sx + 0.2, top - 0.38, zf + 0.045)))
                sx += 0.6
        return
    if GIRDER:
        if VARIANT == "A":
            box(x0, x1, top - 0.18, top, zb, zf, "steel", "steel_card", "steel_dark")
            box(x0 + 0.1, x1 - 0.1, top - 0.68, top - 0.18, zf - 0.38, zf - 0.22, "steel_dark", "steel_dark")
        elif VARIANT == "B":
            box(x0, x1, top - 0.14, top, zb, zf, "steel", "steel_card", "steel_dark")
            box(x0 + 0.06, x1 - 0.06, top - 0.5, top - 0.14, zb + 0.4, zf - 0.1, "steel_dark", "steel_dark")
            box(x0 + 0.12, x1 - 0.12, top - 0.7, top - 0.5, zb + 0.2, zf - 0.2, "steel", "steel_card")
        else:
            box(x0, x1, top - 0.1, top, zb, zf, "steel", "steel_card", "steel_dark")
            # An I-beam's profile cut out at the front, over its shadow.
            prof = [(x0, top - 0.05), (x1, top - 0.05), (x1, top - 0.22), (x1 - 0.2, top - 0.22), (x1 - 0.2, top - 0.55), (x1, top - 0.55),
                    (x1, top - 0.72), (x0, top - 0.72), (x0, top - 0.55), (x0 + 0.2, top - 0.55), (x0 + 0.2, top - 0.22), (x0, top - 0.22)]
            face("steel_card", [P(x, y, zf + 0.03) for (x, y) in prof])
            face("shadow", [P(x + 0.25, y - 0.25, zf - 0.04) for (x, y) in prof])
        return
    if VARIANT == "A":
        gap = 0.1
        bw = (zf - zb - 2 * gap) / 3.0
        for k in range(3):
            za = zb + k * (bw + gap)
            box(x0 - 0.1, x1 + 0.1, top - 0.28, top, za, za + bw, "plank", "plank_card", "plank_b")
        for bx in (x0 + 0.4, x1 - 0.4):
            box(bx - 0.25, bx + 0.25, top - 0.55, top - 0.28, zb + 0.1, zf - 0.1, "plank_b", "plank_b")
            box(bx - 0.12, bx + 0.12, top - 0.3, top - 0.02, zf - 0.02, zf + 0.04, "rope", "rope")
            box(bx - 0.12, bx + 0.12, top - 0.3, top - 0.02, zb - 0.04, zb + 0.02, "rope", "rope")
    elif VARIANT == "B":
        sheet = "top" if THEME in ("grove", "sunset") else "plank"
        box(x0 - 0.1, x1 + 0.1, top - 0.12, top, zb, zf, sheet, "top_card" if sheet == "top" else "plank_card", "plank_b")
        box(x0 - 0.04, x1 + 0.04, top - 0.45, top - 0.12, zb + 0.05, zf - 0.05, "plank", "plank_card", "plank_b")
        if THEME == "grove":
            edge_row(x0 - 0.1, x1 + 0.1, top - 0.08, top - 0.55, zf + 0.04, "fringe", up=False, size=0.4, rng=rng)
    else:
        box(x0 - 0.1, x1 + 0.1, top - 0.1, top, zb, zf, "plank", "plank_card", "plank_b")
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
            # (Boxes are wound outward already; the flat cards face the camera and the sky.)
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
    top_area(rng)
    for r in PLATFORMS:
        platform(r, rng)
    make_objects()
    # (Subdivided for the vertex bake: a big top's two triangles would take one shade.)
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
    print("%s play %s: %d triangles -> %s" % (THEME, VARIANT, tris, out))


main()
