"""The grove's background in paper (the owner's paper pick on the meadow and the beach, 2026-10-20): the ledges behind the island, the
hills on them and the far violet ranges, built in world space on the grove's layout (`ground.json`: each painted range's crest). The side
trees (`grove_tree.glb`) keep their footing on the nearest ledge, at -9.3 round x -30 and 32, depth -55. Three looks for the owner to pick
from (`--ground-mock=` on the stage demo):
  A  card terraces: each ledge a stack of card sheets stepping down and forward, a green top over teal cut edges; paper disc pines;
  B  pop-up: each ledge a shelf with standing flats in front of it, a scalloped grass fringe over a jagged teal cliff; trees as two cut-out
     pine silhouettes slotted together crosswise, as in a pop-up book;
  C  contour hills: rolling hills of stacked card in place of the ledges, a flat top under each side tree; paper disc pines.
After the backgrounds audit (docs/art/stages/BACKGROUNDS.md, 2026-10-21; the ledges read as teal stripes over one dark band), three more:
  FA floating ledges: the ledges broken into islands at varied heights, as the grove's reference floats them, each a grass sheet whose
     lip drapes over its edge above a rock underside tapering to a point, over a sea of pale violet mist in place of the dark band;
  FB FA with moss hanging down the cliffs and tufts, flowers and stones on the tops;
  FC FB with the reference's far forest: rows of giant trunks fading into the violet under a canopy band.
After the owner's note that the floating islands made little progress (2026-10-21), one bolder look built from the grove reference:
  H  hero terraces: connected land round the chasm the island floats in, three tiers in a half-circle round the stage, each tier's
     front rising from the one before it; their edges wander out into promontories and turn back into the chasm, so the faces look
     every way; under a thick grass top that overhangs in a scalloped lip, dark faceted cardboard rock steps back into a violet abyss;
     ferns and vines hang where the edges face the camera; pines seated on the tops; deep shade baked under every lip.
After fidelity review 6 (H's rock read as flat dark-teal blocks over a flat violet floor), on H:
  `--rock shaped`  each block its own tone, its bevel a lighter lit rim (`cliff_lit`), roots hanging from the lips facing the camera;
  `--abyss mist`   with it, a misty valley in the abyss: two layers of violet mist with cut edges at different depths, the tops of a
                   forest of the grove's trees rising through them.
After fidelity review 9 (the pale pine tops near the chasm read as trees standing in water): `--valley none|far` with `--abyss mist`,
  none  the mist without the trees; far  only the trees well back in the valley (behind 80), fewer.
New art toward 5 (docs/art/stages/NEW_ART_PLAN.md, G1 and G3, the owner's approval 2026-10-25), with `--abyss mist`:
  `--falls N`     N waterfalls pouring off the terraces' lips into the abyss, away from the fight: cut paper strips in two blues
                  arcing off the edge and falling to the mist, spray heaped where they meet it;
  `--mist 3`      the mist in three layers that pale and thicken with depth, in place of two;
  `--roots long`  the roots hanging from the lips grown long into the abyss.
Every look carries the far ranges as standing cut-out flats, three to a range, their crests from the painted ranges they stand in for.
Each piece is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `grass`, `grass_far`, `cliff`, `range_far`, `range_near`, their cut edges `card_<family>`, `leaf_lit`, `leaf_shade`, `bark`.

Shipped (new art toward 5, the owner's pick B, 2026-10-25): godot/models/stages/grove_paper.glb, `--variant H --rock shaped
--abyss mist --valley none --falls 3 --mist 3 --roots long`, then lit (review 9's without the falls, mist and roots); with `--deep A`
since fidelity review 11 (2026-10-26).

Run (Blender 5.2):
  blender --background --python art/blender/make_grove_paper.py -- --dir godot/art/baked/<key> --variant A --ground <g.glb>
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold_kit import pleated_tier  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
SHAPED = opts.get("rock", "") == "shaped"
MISTY = opts.get("abyss", "") == "mist"
VALLEY = opts.get("valley", "")
FALLS = int(opts.get("falls", "0"))
MIST3 = opts.get("mist", "") == "3"
LONG_ROOTS = opts.get("roots", "") == "long"
# Depth in the abyss (fidelity review 11: flat purple with soft blotches, the mist's front one long wavy line): `--deep A|B`. A: the
# near abyss sheet shaded deeper under the island and toward the frame's foot, and the nearest mist's front broken into sheets at their
# own depths, each with its own cut edge. B: A with each mist sheet lighter at its cut edge and deeper behind it.
DEEP = opts.get("deep", "").upper()
# Edges facing the camera, for the falls: (x, z, top, drop, tx, tz, ox, oz).
FALL_SPOTS = []
LAYOUT = json.load(open(os.path.join(opts["dir"], "ground.json")))
SIDE_TREES = [(-30.0, -55.0), (32.0, -55.0)]
# The ledges of the shipped ground (make_grove_ground.py A): name, x0, x1, front z, back z, top, top family.
LEDGES = [("T0", -150, 150, -46, -64, -9.3, "grass"), ("T1L", -150, -16, -70, -90, -5.0, "grass_far"),
          ("T1R", 18, 150, -72, -92, -4.5, "grass_far"), ("T2", -170, 170, -98, -122, -1.0, "grass_far")]


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}
AXES = {}          # an object's dome centre or tree axes, for turning its faces outward
KEEP = set()       # objects whose faces are built facing the way they should (two-sided cut-outs)


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
        PARTS[key].verts.layers.float.new("tone")
    return PARTS[key]


def face(bm, pts):
    return bm.faces.new([bm.verts.new(p) for p in pts])


def xs_across(x0, x1, step):
    out = [x0]
    while out[-1] < x1:
        out.append(min(out[-1] + step, x1))
    return out


# ---- Trees --------------------------------------------------------------------------------------------------------------------------


def disc_pine(name, x, y, z, sc, rng):
    """A pine of stacked paper discs (the meadow's paper pines): shallow cones, each a little smaller and turned, over a trunk."""
    base = P(x, y, z)
    AXES.setdefault(name, []).append((base.x, base.y))
    lit, under, trunk = part(name, "leaf_lit"), part(name, "leaf_shade"), part(name, "bark")
    # (Five discs of eight points, built: six of twelve made the ledges' trees most of the model's triangles.)
    for k in range(5):
        t = k / 4.0
        r = 0.62 * (1.0 - 0.75 * t) * sc
        z0 = base.z + (0.35 + 0.4 * k) * sc
        n = 8
        turn = rng.uniform(0, math.tau)
        rim = [Vector((base.x + math.cos(turn + math.tau * i / n) * r * (1.0 if i % 2 else 0.9),
                       base.y + math.sin(turn + math.tau * i / n) * r * (1.0 if i % 2 else 0.9), z0)) for i in range(n)]
        tip = Vector((base.x, base.y, z0 + 0.34 * sc))
        lip = [v - Vector((0, 0, 0.07 * sc)) for v in rim]
        cen = Vector((base.x, base.y, z0 - 0.07 * sc))
        for i in range(n):
            j = (i + 1) % n
            face(lit, (rim[i], rim[j], tip))
            face(under, (lip[i], lip[j], rim[j], rim[i]))
            face(under, (lip[j], lip[i], cen))
    res = bmesh.ops.create_cone(trunk, cap_ends=True, segments=6, radius1=0.1 * sc, radius2=0.07 * sc, depth=0.6 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 0.3 * sc))


def cutout_tree(name, x, y, z, sc, rng):
    """A pop-up tree: two cut-out pine silhouettes slotted together crosswise, each card two-sided, over a cut-out trunk."""
    KEEP.add(name)
    turn = rng.uniform(0, math.pi * 0.5)
    for k, fam in enumerate(("leaf_lit", "leaf_shade")):
        a = turn + k * math.pi * 0.5
        d = Vector((math.cos(a), math.sin(a), 0.0))
        base = P(x, y, z)
        outline = [(-0.12, 0.0), (-0.12, 0.5)]
        tiers = 4
        for i in range(tiers):
            w = 0.95 * (1.0 - 0.2 * i) * (1.0 if i < tiers - 1 else 0.8)
            y0 = 0.5 + 0.62 * i
            outline += [(-w, y0), (-w * 0.35, y0 + 0.5)]
        outline += [(0.0, 0.5 + 0.62 * tiers + 0.25)]
        right = [(-u, v) for (u, v) in reversed(outline[:-1])]
        pts = [base + d * u * sc + Vector((0, 0, v * sc)) for (u, v) in outline + right]
        # Fan from the trunk's middle, both windings so the card shows from either side.
        c = base + Vector((0, 0, 1.4 * sc))
        bm = part(name, fam if len(pts) > 0 else "bark")
        for i in range(len(pts) - 1):
            face(bm, (c, pts[i], pts[i + 1]))
            face(bm, (c, pts[i + 1], pts[i]))
    tb = part(name, "bark")
    base = P(x, y, z)
    for a in (turn, turn + math.pi * 0.5):
        d = Vector((math.cos(a), math.sin(a), 0.0)) * 0.12 * sc
        quad = (base - d, base + d, base + d + Vector((0, 0, 0.55 * sc)), base - d + Vector((0, 0, 0.55 * sc)))
        face(tb, quad)
        face(tb, tuple(reversed(quad)))


def plant_trees_floating(rng):
    """Small paper pines on the floating ledges' tops (`AXES` holds each island's middle; the trees stand within the inner half)."""
    for name, axes in list(AXES.items()):
        if not name.startswith("F"):
            continue
        cx, cz_neg = axes[0]
        # (The top's height is the island's own: read back from its top sheet's middle vertex.)
        for bm_key in [key for key in PARTS if key[0] == name and key[1] in ("grass", "grass_far")]:
            PARTS[bm_key].verts.ensure_lookup_table()
            top = PARTS[bm_key].verts[0].co.z
            for _ in range(rng.randrange(0, 3)):
                x = cx + rng.uniform(-3.5, 3.5)
                if abs(x + 30) < 8 or abs(x - 32) < 8:
                    continue
                disc_pine(name + "_trees", x, top, -cz_neg + rng.uniform(-2, 2), rng.uniform(1.8, 2.8), rng)
            break


def plant_trees(rng, tree):
    """Small trees on the ledges, as the shipped ground has them: clear of the side trees and of the space behind the fight."""
    for name, x0, x1, zf, zb, top, _ in LEDGES:
        n = {"T0": 34, "T2": 30}.get(name, 20)
        for k in range(n):
            x = rng.uniform(x0 + 6, x1 - 6)
            if abs(x + 30) < 8 or abs(x - 32) < 8 or abs(x) < (24 if name != "T2" else 14):
                continue
            tree(name + "_trees", x, top, rng.uniform(zb + 2, zf - 2), rng.uniform(1.8, 3.0), rng)


# ---- A: card terraces ---------------------------------------------------------------------------------------------------------------


def terrace(name, x0, x1, zf, zb, top, fam, rng, sheets=8):
    """A ledge as card: its top sheet, then sheets stepping down and forward, each cut edge standing over the next."""
    xs = xs_across(x0, x1, 3.0)
    ph = [rng.uniform(0, 6) for _ in range(sheets + 1)]

    def edge(k, x):
        return zf + k * 0.8 + 0.6 * math.sin(x * 0.13 + ph[k]) + 0.3 * math.sin(x * 0.37 + ph[k] * 2.0)

    for i in range(len(xs) - 1):
        xa, xb = xs[i], xs[i + 1]
        face(part(name, fam), (P(xa, top, zb), P(xa, top, edge(0, xa)), P(xb, top, edge(0, xb)), P(xb, top, zb)))
        for k in range(sheets):
            y, yn = top - k * 1.4, top - (k + 1) * 1.4
            card = "card_" + (fam if k == 0 else "cliff")
            bottom = yn if k < sheets - 1 else yn - 45.0
            face(part(name, card), (P(xa, y, edge(k, xa)), P(xa, bottom, edge(k, xa)), P(xb, bottom, edge(k, xb)), P(xb, y, edge(k, xb))))
            if k < sheets - 1:
                face(part(name, "cliff"), (P(xa, yn, edge(k, xa)), P(xa, yn, edge(k + 1, xa)), P(xb, yn, edge(k + 1, xb)),
                                           P(xb, yn, edge(k, xb))))


# ---- B: pop-up ----------------------------------------------------------------------------------------------------------------------


def popup_ledge(name, x0, x1, zf, zb, top, fam, rng):
    """A ledge as a pop-up: a shelf for the trees, a scalloped grass fringe standing at its front, and a jagged teal cliff card in front
    of the fringe's foot, with a second, lower cliff card nearer still; each card's top edge shows its thickness."""
    xs = xs_across(x0, x1, 2.0)
    for i in range(len(xs) - 1):
        xa, xb = xs[i], xs[i + 1]
        face(part(name, fam), (P(xa, top, zb), P(xa, top, zf), P(xb, top, zf), P(xb, top, zb)))
    # The fringe: scallops over the shelf's front edge.
    for i in range(len(xs) - 1):
        xa, xb = xs[i], xs[i + 1]
        xm = (xa + xb) * 0.5
        hm = top + 0.45 + 0.15 * math.sin(xm * 0.7)
        z = zf + 0.05
        face(part(name, fam), (P(xa, top - 1.2, z), P(xb, top - 1.2, z), P(xb, top + 0.1, z), P(xm, hm, z), P(xa, top + 0.1, z)))
    # The cliff cards.
    for k, (dz, drop, fam_c) in enumerate(((0.6, 0.9, "cliff"), (1.6, 3.4, "cliff"))):
        z = zf + dz
        pts = []
        x = x0
        while x < x1:
            pts.append((x, top - drop - rng.uniform(0.0, 1.1)))
            x += rng.uniform(1.6, 3.4)
        pts.append((x1, top - drop))
        for i in range(len(pts) - 1):
            (xa, ya), (xb, yb) = pts[i], pts[i + 1]
            face(part(name, fam_c), (P(xa, ya, z), P(xa, top - 50.0, z), P(xb, top - 50.0, z), P(xb, yb, z)))
            face(part(name, "card_" + fam_c), (P(xa, ya, z - 0.3), P(xa, ya, z), P(xb, yb, z), P(xb, yb, z - 0.3)))


# ---- C: contour hills ---------------------------------------------------------------------------------------------------------------


def dome(name, fam, cx, top, cz, rx, rz, rng, step=1.3, floor=-28.0):
    """A hill as stacked card contours (the beach's islands): a flat top at `top`, each sheet below a little wider, lobed and drifting."""
    AXES[name] = [(cx, -cz)]
    lobes = [(n, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for n in (2, 3, 5)]
    drift = (rng.uniform(-1, 1), rng.uniform(-1, 1))
    ring_n = 28
    heights = [top]
    while heights[-1] > floor:
        heights.append(heights[-1] - step * (1.0 + 0.04 * len(heights)))
    rings = []
    for k, h in enumerate(heights):
        grow = 1.0 + 0.22 * k
        ring = []
        for i in range(ring_n):
            a = math.tau * i / ring_n
            w = 1.0 + 0.08 * sum(amp * math.sin(a * n + ph + k * 0.3 * n) for (n, amp, ph) in lobes)
            ring.append((cx + drift[0] * k * 0.3 + math.cos(a) * rx * grow * w, cz + drift[1] * k * 0.2 + math.sin(a) * rz * grow * w))
        rings.append(ring)
    face(part(name, fam), [P(x, heights[0], z) for (x, z) in rings[0]])
    for k in range(len(heights) - 1):
        y, yn = heights[k], heights[k + 1]
        for i in range(ring_n):
            j = (i + 1) % ring_n
            a, b = rings[k][i], rings[k][j]
            c, d = rings[k + 1][j], rings[k + 1][i]
            face(part(name, "card_" + (fam if k == 0 else "cliff")), (P(a[0], y, a[1]), P(b[0], y, b[1]), P(b[0], yn, b[1]), P(a[0], yn, a[1])))
            face(part(name, fam if k < 2 else "cliff"), (P(a[0], yn, a[1]), P(b[0], yn, b[1]), P(c[0], yn, c[1]), P(d[0], yn, d[1])))


def contour_hills(rng):
    k = 0
    # A flat-topped hill under each side tree.
    for (x, z) in SIDE_TREES:
        dome("H%d" % k, "grass", x, -9.3 if x < 0 else -9.0, z, 7.0, 5.0, rng)
        k += 1
    for name, x0, x1, zf, zb, top, fam in LEDGES:
        x = x0 + rng.uniform(0, 8)
        while x < x1:
            rx = rng.uniform(9.0, 15.0)
            if not any(abs(x - tx) < rx + 4 for (tx, _) in SIDE_TREES) or name != "T0":
                dome("H%d" % k, fam, x, top + rng.uniform(-1.5, 1.0), (zf + zb) * 0.5 + rng.uniform(-3, 3), rx, rng.uniform(6.0, 9.0), rng,
                     step=1.3 if name == "T0" else 2.2)
                k += 1
            x += rx * rng.uniform(2.0, 2.6)


# ---- FA, FB, FC: floating ledges -----------------------------------------------------------------------------------------------------


def isle_ring(cx, cz, rx, rz, rng, n=24):
    p1, p2 = rng.uniform(0, math.tau), rng.uniform(0, math.tau)
    return [(cx + rx * (1.0 + 0.12 * math.sin(2 * a + p1) + 0.06 * math.sin(5 * a + p2)) * math.cos(a),
             cz + rz * (1.0 + 0.12 * math.sin(2 * a + p1) + 0.06 * math.sin(5 * a + p2)) * math.sin(a)) for a in [math.tau * i / n for i in range(n)]]


def floating_isle(name, cx, cz, rx, rz, top, fam, rng, depth, detail):
    """A floating ledge: a grass sheet on top, its lip draped over the edge in scallops, and under it a rock body of stacked card plates
    narrowing to a point; `detail` 2 hangs moss down the rock and dots the top with tufts, flowers and stones."""
    AXES[name] = [(cx, -cz)]
    ring = isle_ring(cx, cz, rx, rz, rng)
    n = len(ring)
    # The top sheet, a fan from its middle.
    c = P(cx, top, cz)
    for i in range(n):
        j = (i + 1) % n
        face(part(name, fam), (c, P(ring[i][0], top, ring[i][1]), P(ring[j][0], top, ring[j][1])))
    # The lip: a scalloped strip of the grass hanging over the edge, a little outward.
    for i in range(n):
        j = (i + 1) % n
        (xa, za), (xb, zb) = ring[i], ring[j]
        ox, oz = (xa - cx) / max(rx, 1e-3) * 0.25, (za - cz) / max(rz, 1e-3) * 0.25
        hang = 0.7 + 0.35 * rng.random()
        xm, zm = (xa + xb) * 0.5 + ox, (za + zb) * 0.5 + oz
        face(part(name, "card_" + fam), (P(xa, top, za), P(xb, top, zb), P(xb + ox, top - 0.45, zb + oz), P(xm, top - hang, zm), P(xa + ox, top - 0.45, za + oz)))
    # The rock: plates stepping in and down to a point.
    plates = 5
    rings = [ring]
    ys = [top - 0.4]
    for k in range(1, plates + 1):
        f = 1.0 - k / (plates + 0.6)
        ph = rng.uniform(0, 1)
        rings.append([(cx + (x - cx) * f * (0.92 + 0.08 * math.sin(i * 1.3 + ph * 6)), cz + (z - cz) * f) for i, (x, z) in enumerate(ring)])
        ys.append(top - 0.4 - depth * (k / plates) ** 0.85)
    for k in range(plates):
        a_ring, b_ring = rings[k], rings[k + 1]
        fam_r = "isle_rock"
        for i in range(n):
            j = (i + 1) % n
            face(part(name, fam_r), (P(a_ring[i][0], ys[k], a_ring[i][1]), P(a_ring[j][0], ys[k], a_ring[j][1]),
                                     P(b_ring[j][0], ys[k + 1], b_ring[j][1]), P(b_ring[i][0], ys[k + 1], b_ring[i][1])))
    tip = P(cx + rng.uniform(-1, 1), top - 0.4 - depth * 1.18, cz)
    for i in range(n):
        j = (i + 1) % n
        face(part(name, "isle_rock"), (P(rings[-1][i][0], ys[-1], rings[-1][i][1]), P(rings[-1][j][0], ys[-1], rings[-1][j][1]), tip))
    if detail >= 2:
        # Moss hanging down the rock from the lip, in strips.
        for i in range(0, n, 3):
            (xa, za) = ring[i]
            if za < cz:
                continue
            ln = depth * rng.uniform(0.18, 0.4)
            w = 0.5 + 0.4 * rng.random()
            dx = (xa - cx) / max(rx, 1e-3) * 0.3
            dz = (za - cz) / max(rz, 1e-3) * 0.3
            pts = [(xa - w + dx, top - 0.3, za + dz), (xa + w + dx, top - 0.3, za + dz), (xa + w * 0.6 + dx * 0.6, top - ln * 0.7, za + dz * 0.6),
                   (xa + dx * 0.5, top - ln, za + dz * 0.5), (xa - w * 0.6 + dx * 0.6, top - ln * 0.8, za + dz * 0.6)]
            face(part(name, "isle_moss"), [P(*q) for q in pts])
        # Tufts, flowers and stones on the top.
        for _ in range(int(rx * rz * 0.08) + 2):
            a = rng.uniform(0, math.tau)
            u = rng.uniform(0.1, 0.8)
            x, z = cx + rx * u * math.cos(a), cz + rz * u * math.sin(a)
            kind = rng.random()
            if kind < 0.55:
                for t in range(3):
                    ang = rng.uniform(0, math.pi)
                    dx, dz = math.cos(ang) * 0.35, math.sin(ang) * 0.1
                    face(part(name, "tuft"), (P(x - dx, top + 0.01, z - dz), P(x + dx, top + 0.01, z + dz), P(x + dx * 0.2, top + 0.9, z + dz * 0.2)))
            elif kind < 0.8:
                res = bmesh.ops.create_cone(part(name, rng.choice(("flower_a", "flower_b"))), cap_ends=True, segments=6, radius1=0.25, radius2=0.1, depth=0.2)
                for v in res["verts"]:
                    v.co += P(x, top + 0.3, z)
            else:
                res = bmesh.ops.create_icosphere(part(name, "stone"), subdivisions=1, radius=0.5)
                for v in res["verts"]:
                    v.co.z *= 0.5
                    v.co += P(x, top + 0.15, z)


def mist_sea():
    """A sea of pale violet mist under the islands: two sheets with scalloped fronts, the nearer a step lower."""
    for k, (y, zf) in enumerate(((-24.0, -38.0), (-31.0, -30.0))):
        name = "mist%d" % k
        xs = xs_across(-220, 220, 4.0)
        for i in range(len(xs) - 1):
            xa, xb = xs[i], xs[i + 1]
            fa = zf + 1.6 * math.sin(xa * 0.11 + k) + 0.8 * math.sin(xa * 0.31 + 2 * k)
            fb = zf + 1.6 * math.sin(xb * 0.11 + k) + 0.8 * math.sin(xb * 0.31 + 2 * k)
            face(part(name, "mist"), (P(xa, y, -200.0), P(xa, y, fa), P(xb, y, fb), P(xb, y, -200.0)))
            face(part(name, "card_mist"), (P(xa, y, fa), P(xa, y - 1.2, fa), P(xb, y - 1.2, fb), P(xb, y, fb)))


def far_forest(rng):
    """The reference's far forest: rows of giant trunks rising from the mist past the frame, fading into the violet, under a canopy band."""
    for row, (z, r0, fam) in enumerate(((-150.0, 3.2, "range_near"), (-175.0, 3.8, "range_far"))):
        x = -230.0 + rng.uniform(0, 10)
        while x < 230.0:
            if abs(x) > 18:
                name = "forest%d" % row
                bm = part(name, fam)
                res = bmesh.ops.create_cone(bm, cap_ends=False, segments=7, radius1=r0 * 1.25, radius2=r0, depth=90.0)
                for v in res["verts"]:
                    v.co += P(x, 15.0, z + rng.uniform(-6, 6))
            x += rng.uniform(16.0, 26.0)
        # The canopy: a scalloped band across the top.
        name = "canopy%d" % row
        xs = xs_across(-240, 240, 6.0)
        y0 = 34.0 + row * 4.0
        for i in range(len(xs) - 1):
            xa, xb = xs[i], xs[i + 1]
            ya = y0 - 3.0 * abs(math.sin(xa * 0.09 + row))
            yb = y0 - 3.0 * abs(math.sin(xb * 0.09 + row))
            face(part(name, "canopy"), (P(xa, ya, z - 2), P(xb, yb, z - 2), P(xb, 90.0, z - 2), P(xa, 90.0, z - 2)))


def floating_ledges(rng):
    detail = {"FA": 1, "FB": 2, "FC": 2}[VARIANT]
    k = 0
    for name, x0, x1, zf, zb, top, fam in LEDGES:
        depth = {"T0": 10.0}.get(name, 7.0)
        cz = (zf + zb) * 0.5
        # (The side trees keep their islands: one centred under each.)
        starts = [tx for (tx, tz) in SIDE_TREES] if name == "T0" else []
        for tx in starts:
            floating_isle("F%d" % k, tx, -55.0, 11.0, 7.5, -9.3 if tx < 0 else -9.0, fam, rng, depth, detail)
            k += 1
        x = x0 + rng.uniform(0, 8)
        while x < x1:
            rx = rng.uniform(7.0, 13.0)
            if all(abs(x - tx) > rx + 13.0 for tx in starts) and abs(x) > (14 if name != "T2" else 6):
                floating_isle("F%d" % k, x, cz + rng.uniform(-3, 3), rx, rng.uniform(5.0, 8.0), top + rng.uniform(-2.0, 1.5), fam, rng, depth, detail)
                k += 1
            x += rx * 2.0 + rng.uniform(4.0, 10.0)
    mist_sea()
    if VARIANT == "FC":
        far_forest(rng)


# ---- Far land: the ranges as cut-out flats ------------------------------------------------------------------------------------------


# ---- H: hero ledges -----------------------------------------------------------------------------------------------------------------


def oriented(bm, pts, centre):
    f = face(bm, pts)
    f.normal_update()
    if f.normal.dot(f.calc_center_median() - centre) < 0.0:
        f.normal_flip()
    return f


def rock_block(name, fam, corners, y0, y1, bevel):
    """A cardboard rock block: straight sides (`corners` in (x, z), its own outline), a bevel all round its top, a flat top."""
    KEEP.add(name)
    bm = part(name, fam)
    # (Shaped rock, review 6: the bevel in a lighter lit rim, and each block a tone of its own.)
    rim = part(name, "cliff_lit") if SHAPED and fam.startswith("cliff") else bm
    first = len(bm.verts)
    first_rim = len(rim.verts)
    cx = sum(q[0] for q in corners) / len(corners)
    cz = sum(q[1] for q in corners) / len(corners)
    centre = P(cx, (y0 + y1) * 0.5, cz)
    if SHAPED and fam.startswith("cliff"):
        # (A deeper bevel: at 0.45 on blocks three to five tall the lit rim was a hairline.)
        bevel = bevel * 2.0
    bevel = min(bevel, (y1 - y0) * 0.4)
    inset = []
    for (x, z) in corners:
        d = math.hypot(x - cx, z - cz) or 1.0
        f = max(0.3, 1.0 - bevel / d)
        inset.append((cx + (x - cx) * f, cz + (z - cz) * f))
    n = len(corners)
    yb = y1 - bevel
    for i in range(n):
        j = (i + 1) % n
        (xa, za), (xb, zb) = corners[i], corners[j]
        oriented(bm, (P(xa, y0, za), P(xb, y0, zb), P(xb, yb, zb), P(xa, yb, za)), centre)
        (ia, ka), (ib, kb) = inset[i], inset[j]
        oriented(rim, (P(xa, yb, za), P(xb, yb, zb), P(ib, y1, kb), P(ia, y1, ka)), centre)
    oriented(rim, [P(x, y1, z) for (x, z) in inset], centre)
    if SHAPED:
        t = random.Random(round(cx * 7.0) * 7919 + round(cz * 7.0) + round(y0 * 3.0)).uniform(0.7, 1.4)
        for b, k0 in ((bm, first), (rim, first_rim)):
            layer = b.verts.layers.float.get("tone")
            b.verts.ensure_lookup_table()
            for v in b.verts[k0:]:
                v[layer] = t
    return inset


# (The terraces as plan outlines, Godot x and z, each with its top: connected land, every tier's back running under the face of the tier
# behind it, so nothing floats. The side trees stand at -55 on T0, x -30 and 32 (the grove's tree slots); the far tree village's great
# trees stand on T2 at -1, x 31 to 49 either side, z -106 to -116 (make_world.py GROVE_FAR).)
ABYSS_FLOOR = -26.0


def bumps(x, spots):
    """A front edge's promontories: each spot (x, how far out, how wide) pushes the edge toward the camera."""
    return sum(out * math.exp(-((x - cx) / w) ** 2) for (cx, out, w) in spots)


def arc_band(name, r_in, r_out, a0, a1, top, grass_far, rng, promontories):
    """A terrace as a band of a half-circle round the stage (angles in degrees, 270 straight behind it): its front the inner arc,
    wandering in toward the stage in promontories, its back the outer arc, which runs under the face of the tier behind."""
    span = math.radians(a1 - a0)
    length = r_in * span
    spots = [(rng.uniform(0.08, 0.92) * length, rng.uniform(4.0, 8.0), rng.uniform(6.0, 10.0)) for _ in range(promontories)]
    n = int(length / 2.0)
    inner = []
    for i in range(n + 1):
        sv = length * i / n
        a = math.radians(a0) + span * i / n
        r = r_in - bumps(sv, spots) + 0.8 * math.sin(sv * 0.31)
        inner.append((r * math.cos(a), r * math.sin(a)))
    m = max(8, int(r_out * span / 6.0))
    outer = [(r_out * math.cos(math.radians(a1) - span * i / m), r_out * math.sin(math.radians(a1) - span * i / m)) for i in range(m + 1)]
    return (name, inner + outer, top, grass_far)


def arc_tiers(rng):
    """The terraces in a half-circle round the stage, owner's ask 2026-10-21: T0 and T1 in halves (left and right at their own
    heights, meeting straight behind the great tree), T2 whole, each a little longer than the last so its ends wrap past the one in
    front. The side trees' gates stand on T0 at about 61 and 65 from the middle; the far tree village on T2 at 110 to 121."""
    return [arc_band("T0L", 46.0, 72.0, 166.0, 270.0, -9.3, False, rng, 2), arc_band("T0R", 46.0, 72.0, 270.0, 374.0, -9.0, False, rng, 2),
            arc_band("T1L", 72.0, 100.0, 162.0, 270.0, -5.0, True, rng, 2), arc_band("T1R", 72.0, 100.0, 270.0, 378.0, -4.5, True, rng, 2),
            arc_band("T2", 100.0, 136.0, 156.0, 384.0, -1.0, True, rng, 4)]


def inside(poly, x, z):
    hit = False
    for i in range(len(poly)):
        (xa, za), (xb, zb) = poly[i], poly[i - 1]
        if (za > z) != (zb > z) and x < xa + (z - za) * (xb - xa) / (zb - za):
            hit = not hit
    return hit


def ground_under(tiers, x, z, below):
    """The highest terrace top under (x, z) lower than `below`, or the abyss floor."""
    tops = [t for (_, poly, t, _) in tiers if t < below - 0.01 and inside(poly, x, z)]
    return max(tops, default=ABYSS_FLOOR)


def edge_rock(name, a, b, out, top, drop, rng, far):
    """The rock under one stretch of a terrace's edge: blocks of uneven width along it, each column stepping back from the edge as it
    goes down, its front faceted; `out` is the edge's outward direction in plan (x, z)."""
    rocks = ["cliff", "cliff_b", "cliff", "cliff_b", "cliff"]
    ln = math.hypot(b[0] - a[0], b[1] - a[1])
    if ln < 0.05:
        return
    tx, tz = (b[0] - a[0]) / ln, (b[1] - a[1]) / ln
    ox, oz = out
    s0 = 0.0
    k = 0
    while s0 < ln - 0.05:
        w = min(rng.uniform(4.0, 7.5), ln - s0)
        s1 = s0 + w
        y = top - 0.4
        step = 0.0
        while y > drop + 0.05:
            h = rng.uniform(3.0, 5.0) if not far else rng.uniform(3.5, 5.5)
            y_lo = max(drop - 0.3, y - h)
            if y_lo - (drop - 0.3) < 1.2:
                y_lo = drop - 0.3
            # (Set back under the lip: in front of it, the rock hid the grass's edge.)
            back = 0.9 + step
            pa = (a[0] + tx * s0 - ox * back, a[1] + tz * s0 - oz * back)
            pb = (a[0] + tx * s1 - ox * back, a[1] + tz * s1 - oz * back)
            u = rng.uniform(0.3, 0.7)
            ridge = rng.uniform(0.2, 0.6)
            pm = (pa[0] + (pb[0] - pa[0]) * u + ox * ridge, pa[1] + (pb[1] - pa[1]) * u + oz * ridge)
            depth = 3.0
            corners = [(pa[0] + tx * 0.05 - ox * 0.3, pa[1] + tz * 0.05 - oz * 0.3), pm,
                       (pb[0] - tx * 0.05 - ox * 0.3, pb[1] - tz * 0.05 - oz * 0.3),
                       (pb[0] - ox * depth, pb[1] - oz * depth), (pa[0] - ox * depth, pa[1] - oz * depth)]
            # (Counter-clockwise seen from above, whichever way the edge runs.)
            area = sum(corners[i][0] * corners[i - 1][1] - corners[i - 1][0] * corners[i][1] for i in range(len(corners)))
            if area > 0:
                corners.reverse()
            fam = rocks[(k + rng.randrange(3)) % len(rocks)]
            inset = rock_block("%s_r%d" % (name, k % 6), fam, corners, y_lo, y, 0.45)
            if step > 0.0 and not far and rng.random() < 0.55:
                # Moss on the ledge the block above leaves where it steps back (the reference's mossy rock).
                ym = y + 0.03
                # (The block's three front corners, whichever way its outline was wound, in order along the edge.)
                front3 = sorted(sorted(inset, key=lambda q: -(q[0] * ox + q[1] * oz))[:3], key=lambda q: q[0] * tx + q[1] * tz)
                fa_, fm_, fb_ = front3
                face(part("%s_moss" % name, "moss"), (P(fa_[0], ym, fa_[1]), P(fm_[0], ym, fm_[1]), P(fb_[0], ym, fb_[1]),
                                                       P(fb_[0] - ox * 1.1, ym, fb_[1] - oz * 1.1), P(fa_[0] - ox * 1.1, ym, fa_[1] - oz * 1.1)))
            y = y_lo
            step += rng.uniform(1.0, 2.0)
            k += 1
        s0 = s1


def terrace(name, poly, top, exposed, tiers, rng, far, fam):
    """One terrace: its grass top, and along every edge that shows, the rock under it, the lip over it, and ferns and vines."""
    bm = part("%s_top" % name, fam)
    f = bm.faces.new([bm.verts.new(P(x, top, z)) for (x, z) in poly])
    f.normal_update()
    if f.normal.z < 0.0:
        f.normal_flip()
    bmesh.ops.triangulate(bm, faces=[f])
    n = len(poly)
    # (Which way is out: seen from above in Godot's x and -z, out is the edge turned right on a counter-clockwise outline, left on a
    # clockwise one; the shoelace sum says which.)
    area = sum(poly[i - 1][0] * -poly[i][1] - poly[i][0] * -poly[i - 1][1] for i in range(n))
    turn = 1.0 if area > 0 else -1.0
    runs = []
    cur = None
    for i in range(n):
        a, b = poly[i], poly[(i + 1) % n]
        ln = math.hypot(b[0] - a[0], b[1] - a[1])
        if ln < 0.05:
            continue
        tx, tz = (b[0] - a[0]) / ln, (b[1] - a[1]) / ln
        # Turned right in (x, -z) and back: the outward direction in (x, z).
        ox, oz = -turn * tz, turn * tx
        mx, mz = (a[0] + b[0]) * 0.5 + ox * 0.6, (a[1] + b[1]) * 0.5 + oz * 0.6
        if oz < -0.6 or not exposed(mx, mz):
            # (Facing away from the camera, or lying against the tier behind: nothing shows there.)
            cur = None
            continue
        drop = ground_under(tiers, mx, mz, top)
        # (The rock is laid along whole runs of showing edge, not per outline segment: those are two units long, and blocks no wider
        # than them read as a wall of little cubes.)
        if cur is not None and cur[0] == drop and cur[1][-1] == a:
            cur[1].append(b)
        else:
            cur = (drop, [a, b])
            runs.append(cur)
        # The lip over this stretch: the grass's thick cut edge and a hanging tab, a little past the rock.
        ax, az = a[0] + ox * 0.5, a[1] + oz * 0.5
        bx, bz = b[0] + ox * 0.5, b[1] + oz * 0.5
        lip = part("%s_top" % name, "lip")
        oriented(lip, (P(ax, top, az), P(ax, top - 0.55, az), P(bx, top - 0.55, bz), P(bx, top, bz)), P((a[0] + b[0]) * 0.5, top - 0.3, (a[1] + b[1]) * 0.5))
        hang = rng.uniform(0.9, 2.2) * (0.7 if far else 1.0)
        cx, cz = (ax + bx) * 0.5 + ox * 0.08, (az + bz) * 0.5 + oz * 0.08
        if not far and oz >= 0.6:
            FALL_SPOTS.append((cx, cz, top, drop, tx, tz, ox, oz))
        oriented(lip, (P(ax, top - 0.5, az), P(bx, top - 0.5, bz), P(bx - tx * 0.25, top - 0.5 - hang * 0.7, bz + oz * 0.05),
                       P(cx, top - 0.5 - hang, cz), P(ax + tx * 0.25, top - 0.5 - hang * 0.7, az + oz * 0.05)),
                 P((a[0] + b[0]) * 0.5 - ox, top - 0.8, (a[1] + b[1]) * 0.5 - oz))
        if far or oz < 0.2:
            continue
        # Vines and ferns where the edge faces the camera.
        if rng.random() < 0.18:
            length = min(rng.uniform(3.0, 9.0), top - drop - 1.0)
            if length > 1.5:
                vx, vz = cx + ox * 0.2, cz + oz * 0.2
                q_n = int(length / 0.8)
                for q in range(q_n):
                    ya, yb = top - 0.6 - length * q / q_n, top - 0.6 - length * (q + 1) / q_n
                    wob = 0.3 * math.sin(q * 1.3 + vx)
                    face(part("%s_vines" % name, "vine"), (P(vx + wob - 0.18, ya, vz), P(vx + wob + 0.18, ya, vz), P(vx + wob + 0.14, yb, vz), P(vx + wob - 0.14, yb, vz)))
                    if q % 2 == 1:
                        sd = 1 if q % 4 == 1 else -1
                        face(part("%s_vines" % name, "fern"), (P(vx + wob, ya, vz + 0.02), P(vx + wob + sd * 0.7, ya + 0.25, vz + 0.02),
                                                               P(vx + wob + sd * 1.0, ya - 0.15, vz + 0.02), P(vx + wob + sd * 0.3, ya - 0.35, vz + 0.02)))
        if SHAPED and rng.random() < 0.3:
            # Roots hanging from the lip, tapering, one forking (review 6: the reference's rock is hung with roots).
            # (Long, `--roots long`: the same draws, so the layout after them stays as it was.)
            length = min(rng.uniform(2.0, 5.5) * (2.4 if LONG_ROOTS else 1.0), top - drop - 1.0)
            if length > 1.2:
                rx_, rz_ = cx + ox * 0.15 + tx * rng.uniform(-0.8, 0.8), cz + oz * 0.15
                for strand in range(rng.randrange(1, 3)):
                    sx = rx_ + tx * strand * 0.5
                    q_n = max(2, int(length / 0.7))
                    for q in range(q_n):
                        ya, yb = top - 0.5 - length * q / q_n, top - 0.5 - length * (q + 1) / q_n
                        wa, wb = 0.16 * (1.0 - q / q_n) + 0.03, 0.16 * (1.0 - (q + 1) / q_n) + 0.03
                        xa = sx + 0.25 * math.sin(q * 1.1 + sx)
                        xb = sx + 0.25 * math.sin((q + 1) * 1.1 + sx)
                        face(part("%s_roots" % name, "root"), (P(xa - wa, ya, rz_), P(xa + wa, ya, rz_), P(xb + wb, yb, rz_), P(xb - wb, yb, rz_)))
        if rng.random() < 0.3:
            for q in range(5):
                ang = math.radians(-160 + q * 35 + rng.uniform(-8, 8))
                lnf = rng.uniform(1.4, 2.4)
                dx, dy = math.cos(ang) * lnf, math.sin(ang) * lnf * 0.8
                face(part("%s_ferns" % name, "fern"), (P(cx, top - 0.3, cz), P(cx + dx * 0.5 - 0.25, top - 0.3 + dy * 0.5 + 0.2, cz + 0.1),
                                                       P(cx + dx, top - 0.3 + dy, cz + 0.15), P(cx + dx * 0.5 + 0.25, top - 0.3 + dy * 0.5 - 0.2, cz + 0.1)))
    for drop, pts in runs:
        rock_run(name, pts, turn, top, drop, rng, far)


def rock_run(name, pts, turn, top, drop, rng, far):
    """The rock under a run of showing edge: the run cut into chords three to six units long, each with its blocks."""
    lens = [math.hypot(pts[i + 1][0] - pts[i][0], pts[i + 1][1] - pts[i][1]) for i in range(len(pts) - 1)]
    total = sum(lens)

    def at(sv):
        for i, ln in enumerate(lens):
            if sv <= ln or i == len(lens) - 1:
                t = min(1.0, sv / ln) if ln > 0 else 0.0
                return (pts[i][0] + (pts[i + 1][0] - pts[i][0]) * t, pts[i][1] + (pts[i + 1][1] - pts[i][1]) * t)
            sv -= ln
        return pts[-1]

    s0 = 0.0
    while s0 < total - 0.3:
        s1 = min(total, s0 + rng.uniform(3.0, 6.0))
        a, b = at(s0), at(s1)
        ln = math.hypot(b[0] - a[0], b[1] - a[1])
        if ln > 0.2:
            tx, tz = (b[0] - a[0]) / ln, (b[1] - a[1]) / ln
            edge_rock(name, a, b, (-turn * tz, turn * tx), top, drop, rng, far)
        s0 = s1


def hero_ledges(rng):
    """The grove's terraces round the chasm the island floats in, in a half-circle round the stage."""
    tiers = arc_tiers(rng)
    for name, poly, top, grass_far in tiers:
        far = name == "T2"

        def exposed(x, z, t=top):
            # (An edge shows unless a higher terrace stands right against it.)
            return not any(inside(q, x, z) for (_, q, tt, _) in tiers if tt > t + 0.01)

        terrace(name, poly, top, exposed, tiers, rng, far, "grass_far" if grass_far else "grass")
    HERO_TIERS.extend(tiers)
    abyss()
    if MISTY:
        misty_valley(random.Random(83))
    if FALLS:
        falls(random.Random(131))


HERO_TIERS = []


# Where the far tree village's great trees stand (make_world.py GROVE_FAR), kept clear like the side trees.
VILLAGE = [(-49.0, -106.0), (-31.0, -116.0), (47.0, -106.0), (30.0, -112.0)]
SLOTS = []


def tree_slot(x, y, z, size):
    """An empty where the game stands the grove's own modelled tree (`treeslot`, StageBake.place_ground), as it stands the side trees:
    one kind of tree at one size all round, as in the reference's forest of giants; the paper pines read as shrubs beside them."""
    ob = bpy.data.objects.new("treeslot_%d" % len(SLOTS), None)
    ob.location = P(x, y, z)
    ob.scale = (size, size, size)
    bpy.context.scene.collection.objects.link(ob)
    SLOTS.append((x, z))


def plant_trees_hero(rng):
    """The grove's tree on the terraces, at the side trees' size (0.55 to 0.7 of the great tree), seated on the top it stands on, its
    canopy clear of every edge, of the side trees, the village's trees and one another, and of the space behind the fight."""
    for name, poly, top, _ in HERO_TIERS:
        xs = [q[0] for q in poly]
        zs = [q[1] for q in poly]
        n = {"T2": 6}.get(name, 3)
        placed = 0
        tries = 0
        while placed < n and tries < 2000:
            tries += 1
            x, z = rng.uniform(min(xs), max(xs)), rng.uniform(min(zs), max(zs))
            # (Clear of the space behind the fight, and of the near ends beside the stage, where tall pines would crowd the frame.)
            if not inside(poly, x, z) or abs(x) < (26 if name != "T2" else 16) or z > -15.0:
                continue
            if any(math.hypot(x - tx, z - tz) < 15.0 for (tx, tz) in SIDE_TREES + VILLAGE + SLOTS):
                continue
            if any(not inside(poly, x + dx, z + dz) for (dx, dz) in ((6, 0), (-6, 0), (0, 6), (0, -6), (4, 4), (-4, 4))):
                continue
            if any(inside(q, x + dx, z + dz) for (_, q, t, _) in HERO_TIERS if t > top + 0.01 for (dx, dz) in ((0, 0), (0, -6))):
                continue
            # (Sunk 0.6, as the side trees are, so the roots sit in the grass.)
            tree_slot(x, top - 0.6, z, rng.uniform(0.55, 0.7))
            placed += 1
        # Tufts over the top, thicker toward its front, so it isn't one plain sheet.
        for _ in range(int((max(xs) - min(xs)) * (max(zs) - min(zs)) * (0.02 if name != "T2" else 0.008))):
            x, z = rng.uniform(min(xs), max(xs)), rng.uniform(min(zs), max(zs))
            if not inside(poly, x, z) or any(inside(q, x, z) for (_, q, t, _) in HERO_TIERS if t > top + 0.01):
                continue
            if rng.random() < 0.15:
                res = bmesh.ops.create_cone(part(name + "_flowers", rng.choice(("flower_a", "flower_b"))), cap_ends=True, segments=6,
                                            radius1=0.35, radius2=0.12, depth=0.25)
                for v in res["verts"]:
                    v.co += P(x, top + 0.45, z)
                continue
            for _t in range(3):
                ang = rng.uniform(0, math.pi)
                dx, dz = math.cos(ang) * 0.45, math.sin(ang) * 0.14
                face(part(name + "_tufts", "tuft"), (P(x - dx, top + 0.01, z - dz), P(x + dx, top + 0.01, z + dz), P(x + dx * 0.2, top + 1.1, z + dz * 0.2)))


FOREST = opts.get("forest", "1") == "1"


def forest_tree(name, x, y, z, h, rng, fams=("bark", "leaf_lit", "leaf_shade")):
    """A tree of the grove's forest, in the grove tree's manner at a fraction of its cost: a fluted, tapering trunk flared at its foot and
    four pleated tiers of crown, the lowest widest, each a little turned. `h` its height; `fams` its bark and leaves' families."""
    bark = part(name, fams[0])
    n = 7
    rings = [(0.0, 0.11 * h), (0.06 * h, 0.075 * h), (0.5 * h, 0.05 * h), (0.85 * h, 0.03 * h)]
    turn = rng.uniform(0, math.tau)
    pts = []
    for (yy, rr) in rings:
        pts.append([P(x + math.cos(turn + math.tau * i / n) * rr * (1.0 if i % 2 else 0.86), y + yy,
                      z + math.sin(turn + math.tau * i / n) * rr * (1.0 if i % 2 else 0.86)) for i in range(n)])
    for k in range(len(pts) - 1):
        for i in range(n):
            j = (i + 1) % n
            face(bark, (pts[k][i], pts[k][j], pts[k + 1][j], pts[k + 1][i]))
    lit, under = part(name, fams[1]), part(name, fams[2])
    base = P(x, y, z)
    for k in range(4):
        t = k / 3.0
        rad = (0.34 - 0.2 * t) * h * rng.uniform(0.92, 1.08)
        z0 = base.z + (0.36 + 0.17 * k) * h
        tall = (0.24 - 0.02 * k) * h
        pleated_tier(lit, under, (base.x, base.y), z0, rad, tall, 9 if k < 2 else 7, rng.uniform(0, math.tau), rng, flare=0.46,
                     droop=0.05 * h * (1.0 - 0.4 * t))


def forest(rng):
    """The grove's forest (the owner, 2026-10-22: "our forest is a few sporadically placed trees"; the reference's violet forest is dense):
    the middle and back terraces filled with trees on a jittered grid, the back ones the tallest so they rise over the middle row, clear
    of every edge, the village's great trees and their huts, the space right behind the fight, and the near terrace's own big trees."""
    placed = []
    for name, poly, top, _ in HERO_TIERS:
        if name.startswith("T0"):
            continue
        back = name == "T2"
        step = 6.5 if back else 7.5
        xs = [q[0] for q in poly]
        zs = [q[1] for q in poly]
        gx = min(xs)
        while gx < max(xs):
            gz = min(zs)
            while gz < max(zs):
                x = gx + rng.uniform(-2.2, 2.2)
                z = gz + rng.uniform(-2.2, 2.2)
                gz += step * rng.uniform(0.85, 1.15)
                if not inside(poly, x, z) or abs(x) < (22 if not back else 14):
                    continue
                if any(not inside(poly, x + dx, z + dz) for (dx, dz) in ((3.5, 0), (-3.5, 0), (0, 3.5), (0, -3.5))):
                    continue
                if any(inside(q, x, z) for (_, q, t, _) in HERO_TIERS if t > top + 0.01):
                    continue
                if any(math.hypot(x - tx, z - tz) < 12.0 for (tx, tz) in VILLAGE) or any(math.hypot(x - tx, z - tz) < 9.0 for (tx, tz) in SIDE_TREES + SLOTS):
                    continue
                if rng.random() < 0.12:
                    # (A clearing now and then, so it reads as a wood, not a plantation.)
                    continue
                h = rng.uniform(13.0, 19.0) if back else rng.uniform(10.0, 14.0)
                forest_tree("forest_%s_%d" % (name, len(placed) // 40), x, top - 0.3, z, h, rng)
                placed.append((x, z))
            gx += step * rng.uniform(0.85, 1.15)
    print("grove forest: %d trees" % len(placed))


def abyss():
    """The abyss under the ledges: a deep violet sheet low down, and a paler veil of mist over its far half."""
    # (High enough to swallow the ledges' rock feet: the rock reads as going down into the dark, not as a wall standing on a floor.)
    for k, (y, zf, fam) in enumerate(((-24.0, -20.0, "abyss"), (-20.0, -84.0, "abyss_far"))):
        name = "abyss%d" % k
        xs = xs_across(-220, 220, 5.0)
        if DEEP and k == 0:
            # (In rows, so its tone can change toward the camera and round the island: one sheet per column had no vertices to carry it.)
            bm = part(name, fam)
            layer = bm.verts.layers.float.get("tone")
            zs = [-220.0, -60.0] + [-20.0 + 4.0 * j for j in range(21)]
            for i in range(len(xs) - 1):
                xa, xb = xs[i], xs[i + 1]
                for j in range(len(zs) - 1):
                    za, zb = zs[j], zs[j + 1]
                    f = face(bm, (P(xa, y, za), P(xa, y, zb), P(xb, y, zb), P(xb, y, za)))
                    for v in f.verts:
                        gx, gz = v.co.x, -v.co.y
                        under = math.exp(-(gx / 20.0) ** 2 - ((gz + 2.0) / 16.0) ** 2)
                        foot = min(1.0, max(0.0, (gz - 10.0) / 40.0))
                        v[layer] = (1.0 - 0.4 * under) * (1.0 - 0.28 * foot)
            continue
        for i in range(len(xs) - 1):
            xa, xb = xs[i], xs[i + 1]
            fa = zf + 1.8 * math.sin(xa * 0.09 + k)
            fb = zf + 1.8 * math.sin(xb * 0.09 + k)
            # (The nearer sheet runs on past the camera, so no edge of it shows.)
            near = 60.0 if k == 0 else fa
            face(part(name, fam), (P(xa, y, -220.0), P(xa, y, near if k == 0 else fa), P(xb, y, near if k == 0 else fb), P(xb, y, -220.0)))


def misty_valley(rng):
    """The abyss as a misty valley (review 6): two layers of violet mist at different depths, each a sheet with a wavy cut front edge,
    and the tops of a forest of the grove's trees rising through them, smaller and fainter further back."""
    layers = ((-21.5, -24.0, -70.0), (-17.5, -62.0, -130.0))
    if MIST3:
        # (Three, each paler and higher than the one before it, so the mist thickens with depth.)
        layers = ((-22.0, -22.0, -58.0), (-19.6, -50.0, -96.0), (-17.0, -88.0, -140.0))
    for k, (y, zf, zb) in enumerate(layers):
        name = "mist%d" % k
        fam = ("abyss_mist", "abyss_mist2", "abyss_mist3")[k] if MIST3 else "abyss_mist"
        xs = xs_across(-220, 220, 4.0)
        # (`--deep`: the nearest layer's front in sheets of their own, 30 to 70 across, each further in or out and a little higher or lower.)
        sheets = [(-220.0, 0.0, 0.0)]
        if DEEP and k == 0:
            sr = random.Random(91)
            x = -220.0
            sheets = []
            while x < 220.0:
                sheets.append((x, sr.uniform(-5.0, 4.0), sr.uniform(-0.5, 0.4)))
                x += sr.uniform(30.0, 70.0)
        for i in range(len(xs) - 1):
            xa, xb = xs[i], xs[i + 1]
            _, dz, dy = [sh for sh in sheets if sh[0] <= xa + 0.01][-1]
            yy = y + dy
            fa = zf + dz + 2.5 * math.sin(xa * 0.11 + k * 2.0) + 1.2 * math.sin(xa * 0.37 + k)
            fb = zf + dz + 2.5 * math.sin(xb * 0.11 + k * 2.0) + 1.2 * math.sin(xb * 0.37 + k)
            top = face(part(name, fam), (P(xa, yy, zb), P(xa, yy, fa), P(xb, yy, fb), P(xb, yy, zb)))
            face(part(name, "card_abyss_mist"), (P(xa, yy, fa), P(xa, yy - 0.5, fa), P(xb, yy - 0.5, fb), P(xb, yy, fb)))
            if DEEP == "B":
                # Lighter toward its cut edge, deeper behind it, as a sheet of mist with a lit lip.
                layer = part(name, fam).verts.layers.float.get("tone")
                for v in top.verts:
                    v[layer] = 1.1 if abs(-v.co.y - zb) > 1.0 else 0.82
        if DEEP and k == 0:
            # The ends of each sheet where the next steps in or out: a side cut so no gap shows the abyss through.
            for (sx, dz, dy) in sheets[1:]:
                prev = [sh for sh in sheets if sh[0] < sx][-1]
                for (pdz, pdy) in ((prev[1], prev[2]), (dz, dy)):
                    f0 = zf + pdz + 2.5 * math.sin(sx * 0.11) + 1.2 * math.sin(sx * 0.37)
                    face(part(name, "card_abyss_mist"), (P(sx, y + pdy, zb), P(sx, y + pdy, f0), P(sx, y + pdy - 0.5, f0), P(sx, y + pdy - 0.5, zb)))
    placed = []
    tries = 0
    if VALLEY == "none":
        return
    while len(placed) < (30 if VALLEY == "far" else 70) and tries < 4000:
        tries += 1
        x, z = rng.uniform(-140.0, 140.0), rng.uniform(-110.0, -80.0 if VALLEY == "far" else -26.0)
        if any(math.hypot(x - px, z - pz) < 5.5 for (px, pz) in placed):
            continue
        # (Under the terraces' tops nothing shows; keep to the open chasm and the far valley.)
        if any(inside(poly, x, z) for (_, poly, _, _) in HERO_TIERS):
            continue
        placed.append((x, z))
        # (Only their tops through the mist: taller, the nearest stood as whole pines in the chasm.)
        h = rng.uniform(6.5, 9.5)
        # (In the valley's own hazed violet-greens: in the forest's, spread across the chasm in pieces of no one depth, they took no air
        # and stood out lime against the violet.)
        forest_tree("valley%d" % (len(placed) // 10), x, ABYSS_FLOOR - 1.0, z, h, rng, ("valley_bark", "valley_lit", "valley_shade"))


def falls(rng):
    """Waterfalls off the terraces' lips into the abyss (G1): chosen where an edge faces the camera over the chasm, away from the fight
    and from one another; each a sheet of cut strips in two blues arcing out from the lip and falling to the mist, ragged at its foot,
    with spray heaped where it meets the mist."""
    mist_y = -22.0 if MIST3 else -21.5
    # (Beside the island, inside the wide camera's frame: further out than 32 they fell off its edges.)
    spots = [s for s in FALL_SPOTS if s[3] <= ABYSS_FLOOR + 1.0 and 15.0 < abs(s[0]) < 32.0 and s[1] > -70.0]
    spots.sort(key=lambda s: (abs(abs(s[0]) - 24.0), -s[1]))
    chosen = []
    for s in spots:
        if all(math.hypot(s[0] - c[0], s[1] - c[1]) > 12.0 for c in chosen):
            chosen.append(s)
        if len(chosen) == FALLS:
            break
    print("grove falls at %s" % [(round(c[0], 1), round(c[1], 1)) for c in chosen])
    for k, (x, z, top, drop, tx, tz, ox, oz) in enumerate(chosen):
        name = "fall%d" % k
        KEEP.add(name)
        w = rng.uniform(3.0, 4.4)
        y0 = top - 0.35
        height = y0 - mist_y
        strips = rng.randrange(5, 8)
        steps = 12
        for q in range(strips):
            u0 = -w * 0.5 + w * q / strips + 0.04
            u1 = -w * 0.5 + w * (q + 1) / strips - 0.04
            fam = "fall" if q % 2 == 0 else "fall_shade"
            end = 1.0 - rng.uniform(0.0, 0.06)
            lift = 0.03 * (q % 2)
            bm = part(name, fam)
            for i in range(steps):
                ta, tb = end * i / steps, end * (i + 1) / steps
                # (Out from the lip, then falling: the arc of water leaving an edge.)
                oa, ob = 0.5 + 1.8 * math.sqrt(ta) + lift, 0.5 + 1.8 * math.sqrt(tb) + lift
                ya, yb = y0 - height * ta, y0 - height * tb
                pts = (P(x + tx * u0 + ox * oa, ya, z + tz * u0 + oz * oa), P(x + tx * u1 + ox * oa, ya, z + tz * u1 + oz * oa),
                       P(x + tx * u1 + ox * ob, yb, z + tz * u1 + oz * ob), P(x + tx * u0 + ox * ob, yb, z + tz * u0 + oz * ob))
                oriented(bm, pts, P(x - ox * 2.0, (ya + yb) * 0.5, z - oz * 2.0))
        # Spray where it meets the mist: low heaped puffs.
        bx, bz = x + ox * 2.3, z + oz * 2.3
        bm = part(name + "_spray", "spray")
        KEEP.add(name + "_spray")
        for q in range(rng.randrange(4, 7)):
            r = rng.uniform(0.6, 1.2)
            u = rng.uniform(-w * 0.7, w * 0.7)
            res = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=r)
            c = P(bx + tx * u, mist_y + r * 0.2, bz + tz * u + rng.uniform(-0.3, 0.3))
            for v in res["verts"]:
                v.co = Vector((v.co.x * 1.3, v.co.y, v.co.z * 0.7)) + c


def range_flats():
    for r_i, rng_spec in enumerate(LAYOUT.get("ranges", [])):
        fam = "range_far" if r_i == 0 else "range_near"
        raw = rng_spec["top"]
        # (Smoothed: the highest vertex in each bin follows every fold's tip, and the crest came out a sawtooth.)
        top = []
        peak = [max(raw[j][1] for j in range(max(0, i - 1), min(len(raw), i + 2))) for i in range(len(raw))]
        for i in range(len(raw)):
            near = peak[max(0, i - 2): i + 3]
            top.append((raw[i][0], sum(near) / len(near)))
        z = rng_spec["z"]
        base = min(t[1] for t in top) - 10.0
        # (Close to the painted range's own height, a step taller behind and lower in front: much taller walled off the sky.)
        for k, (dz, scale) in enumerate(((-8.0, 1.08), (0.0, 1.0), (8.0, 0.9))):
            name = "range%d_%d" % (r_i, k)
            zz = z + dz
            pts = [(x, base + (y - base) * scale) for (x, y) in top]
            for i in range(len(pts) - 1):
                (xa, ya), (xb, yb) = pts[i], pts[i + 1]
                face(part(name, fam), (P(xa, ya, zz), P(xa, base - 30.0, zz), P(xb, base - 30.0, zz), P(xb, yb, zz)))
                face(part(name, "card_" + fam), (P(xa, ya, zz - 0.8), P(xa, ya, zz), P(xb, yb, zz), P(xb, yb, zz - 0.8)))


# ---- Build --------------------------------------------------------------------------------------------------------------------------


def build_ground():
    rng = random.Random(29)
    if VARIANT == "H":
        hero_ledges(rng)
        plant_trees_hero(rng)
        if FOREST:
            forest(rng)
    elif VARIANT in ("FA", "FB", "FC"):
        floating_ledges(rng)
        plant_trees_floating(rng)
    elif VARIANT == "A":
        for name, x0, x1, zf, zb, top, fam in LEDGES:
            terrace(name, x0, x1, zf, zb, top, fam, rng)
        plant_trees(rng, disc_pine)
    elif VARIANT == "B":
        for name, x0, x1, zf, zb, top, fam in LEDGES:
            popup_ledge(name, x0, x1, zf, zb, top, fam, rng)
        plant_trees(rng, cutout_tree)
    else:
        contour_hills(rng)
        for name, x0, x1, zf, zb, top, _ in LEDGES:
            for k in range({"T0": 14, "T2": 14}.get(name, 9)):
                x = rng.uniform(x0 + 6, x1 - 6)
                if abs(x + 30) < 9 or abs(x - 32) < 9 or abs(x) < (24 if name != "T2" else 14):
                    continue
                disc_pine(name + "_trees", x, top - 0.3, rng.uniform(zb + 3, zf - 3), rng.uniform(1.8, 3.0), rng)
    range_flats()


def make_objects():
    for (name, mat), bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        if name not in KEEP:
            axes = AXES.get(name)
            for f in bm.faces:
                c = f.calc_center_median()
                if axes:
                    ax = min(axes, key=lambda a: (c.x - a[0]) ** 2 + (c.y - a[1]) ** 2)
                    away = Vector((c.x - ax[0], c.y - ax[1], 0.0))
                    away = (away.normalized() if away.length > 1e-4 else away) + Vector((0.0, 0.0, 0.3))
                else:
                    # (Ledges and flats face the camera and the sky.)
                    away = Vector((0.0, -0.7, 1.0))
                if f.normal.dot(away) < 0.0:
                    f.normal_flip()
        me = bpy.data.meshes.new(name + "_" + mat)
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new(name + "_" + mat, me)
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
        tone = o.data.attributes.get("tone")
        col = o.data.color_attributes["Col"].data
        for v in o.data.vertices:
            k = (tone.data[v.index].value or 1.0) if tone is not None else 1.0
            ao = (0.45 + 0.55 * col[v.index].color[0]) * k
            col[v.index].color = (min(ao, 1.0), min(ao, 1.0), min(ao, 1.0), 1.0)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_ground()
    make_objects()
    # (The hero ledges deeper: the shade under each lip and between the blocks is most of their form.)
    bake_occlusion(5.0 if VARIANT == "H" else 2.0)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    path = os.path.abspath(opts["ground"])
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("grove paper %s: %d triangles -> %s" % (VARIANT, tris, path))


main()
