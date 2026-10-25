"""The canyon's background in paper (the owner's paper picks on the meadow, the beach and the grove, 2026-10-20): the buttes, the rock arch
and the dunes, built in world space on the canyon's layout (`ground.json`: each butte's and the arch's bounds, each dune range's crest).
The desert floor, the cacti, the telegraph poles and the sun stay code-built. Three looks for the owner to pick from (`--ground-mock=` on
the stage demo):
  A  card stacks: every butte and the arch a stack of thick card plates, each its own lobed outline a little off the one below, in the
     canyon's strata colours band by band, on a low skirt of wide plates for the rubble;
  B  pop-up: every butte a few standing cut-outs one behind another, stepped like the strata; the arch a cut-out with its opening cut;
  C  both: the near buttes and the arch as card stacks, the far row as pop-up cut-outs.
After the backgrounds audit (docs/art/stages/BACKGROUNDS.md, 2026-10-21; the floor one plain sand sheet), three more, each with A's
buttes and the desert floor laid in paper over the code-built sand:
  DA dune sheets: the floor laid over with long cut sheets of sand in three tones, each a little lifted with its cut edge showing, wind
     ripples drawn on some as pale strips;
  DB DA with a dry wash winding in from the far right to the front, a pale sheet with darker banks and pebbles along them;
  DC DB with paper scrub, stones and the soft shade under them in place of the code-built scrub (which is hidden under it).
After fidelity review 4 (the buttes and the arch read as stacks of pancakes, rounded discs unlike the slab front and the references'
chunky cardboard rock, docs/art/stages/LOOK.md lesson 4), three more, each on DC's floor:
  KA cardboard blocks: every butte two or three tiers of angular blocks, a few big flat planes each with a bevelled top edge, stepping
     in as they rise, each tier banded in the strata; the arch's legs blocks and its span one bevelled beam in three pieces;
  KB slab courses: built as the stage's own front is, courses of squared slabs of the strata laid in rows, stepping in near the top;
  KC KA with notched crowns (the top tier split into two or three blocks of different heights) and fallen blocks round the feet.
Every look carries the dunes as standing cut-out flats along the crests of the dune ranges they stand in for.
Each piece is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `rock_a`, `rock_b`, `rock_c` (the strata), `dune_far`, `dune_near`, and their cut edges `card_<family>`.

After fidelity review 6 (the butte right behind the fight still read as a stack of round discs, every tier cut into bands a block each;
the plain empty at the wide camera; the arch boxy), on KC's floor with `--arch solid` (`--arch chipped` breaks its silhouette):
  `--buttes solid`  every butte as rock, as the arch: two or three massive tiers of few sides, a pale band between them, a dark cap;
  `--fight wide`    with it, the butte behind the fight a broad flat-topped mesa, a calm table behind the fighters, not a pillar;
  `--plain outcrops` low rock ledges and boulders across the plain, off the wash, the buttes' feet and the strip behind the fight.

New art toward 5 (docs/art/stages/NEW_ART_PLAN.md, C3 and C2, the owner's approval 2026-10-25), on KC:
  `--foot talus`  the stage mesa's foot: a fan of fallen sandstone blocks and scree spreading from its sides onto the floor, the biggest
                  nearest the wall, so the mesa stands on the ground;
  `--hoodoos N`   N clusters of wind-carved hoodoos on the floor either side of the mesa, at true size (columns of stacked strata under a
                  wider cap stone), with boulders round them and their shade.

Shipped (new art toward 5, the owner's pick B, 2026-10-25): godot/models/stages/canyon_paper.glb, `--variant KC --arch chipped
--buttes solid --fight broken --plain outcrops --foot talus --hoodoos 5`, then lit (review 7's without `--foot` and `--hoodoos`) (review 6's had `--fight wide`, review 5's `--variant KC --arch solid`).

Run (Blender 5.2):
  blender --background --python art/blender/make_canyon_paper.py -- --dir godot/art/baked/<key> --variant A --ground <g.glb>
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
CHIP = opts.get("arch", "") == "chipped"
LAYOUT = json.load(open(os.path.join(opts["dir"], "ground.json")))
FLOOR = -16.0
STRATA = ["rock_a", "rock_b", "rock_a", "rock_c"]
FOOT = opts.get("foot", "")
HOODOOS = int(opts.get("hoodoos", "0"))
# The stage mesa (the play block from the bake manifest): its sides and its front, for the talus at its foot.
MANIFEST = json.load(open(os.path.join(opts["dir"], "manifest.json")))
STAGE = [r for r in MANIFEST["rects"] if r[4] < 0.5][0]
STAGE_FZ = float(MANIFEST["front_z"])


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}
AXES = {}


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
    return PARTS[key]


def face(bm, pts):
    return bm.faces.new([bm.verts.new(p) for p in pts])


# ---- Card stacks --------------------------------------------------------------------------------------------------------------------


def plate(name, fam, cx, cz, rx, rz, y0, y1, lobes, k, n=20, hole=None):
    """One thick card plate: a lobed outline (its own wobble by `k`) extruded from y0 to y1, its top and its cut edge."""
    ring = []
    for i in range(n):
        a = math.tau * i / n
        w = 1.0 + 0.09 * sum(amp * math.sin(a * m + ph + k * 0.9) for (m, amp, ph) in lobes)
        ring.append((cx + math.cos(a) * rx * w, cz + math.sin(a) * rz * w))
    # (Its top facing up and its underside down: the ring's winding turns over between the game's axes and Blender's.)
    for y, up in ((y1, 1.0), (y0, -1.0)):
        f = face(part(name, fam), [P(x, y, z) for (x, z) in ring])
        f.normal_update()
        if f.normal.z * up < 0.0:
            f.normal_flip()
    for i in range(n):
        j = (i + 1) % n
        (xa, za), (xb, zb) = ring[i], ring[j]
        face(part(name, fam), (P(xa, y1, za), P(xa, y0, za), P(xb, y0, zb), P(xb, y1, zb)))


def butte_stack(name, b, rng):
    """A butte as stacked plates on a skirt: its core about half its bounds (the rest is the rubble slope), narrowing a little to the cap."""
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    rx, rz = (x1 - x0) * 0.5, (z1 - z0) * 0.5
    AXES[name] = (cx, -cz)
    lobes = [(m, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for m in (2, 3, 5)]
    h = y1 - FLOOR
    # The rubble skirt: two wide, low plates.
    # (Not much wider than the body, and taller: at the bounds' full width they read as hat brims.)
    plate(name, "rock_b", cx, cz, rx * 0.66, rz * 0.66, FLOOR - 0.5, FLOOR + h * 0.1, lobes, 0)
    plate(name, "rock_a", cx, cz, rx * 0.58, rz * 0.58, FLOOR + h * 0.1, FLOOR + h * 0.18, lobes, 1)
    # The body: plates a band each.
    n = 7
    y = FLOOR + h * 0.18
    for k in range(n):
        t = k / (n - 1)
        th = (y1 - y) / (n - k)
        r = 0.52 - 0.05 * t
        ox, oz = rng.uniform(-0.6, 0.6), rng.uniform(-0.4, 0.4)
        plate(name, STRATA[k % len(STRATA)], cx + ox, cz + oz, rx * r, rz * r, y, y + th, lobes, k + 2)
        y += th


def arch_stack(name, b, rng):
    """The arch as plates: two legs of stacked plates and a span of three long plates bowing across their tops."""
    x0, y0, z0, x1, y1, z1 = b
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    w, d, h = x1 - x0, z1 - z0, y1 - FLOOR
    AXES[name] = (cx, -cz)
    lobes = [(m, rng.uniform(0.5, 1.0), rng.uniform(0, math.tau)) for m in (2, 3)]
    leg_h = h * 0.72
    for side in (-1, 1):
        lx = cx + side * (w * 0.5 - 3.0)
        y = FLOOR - 0.3
        k = 0
        while y < FLOOR + leg_h:
            th = rng.uniform(1.8, 2.6)
            r = 3.0 - 0.6 * (y - FLOOR) / leg_h
            plate(name, STRATA[k % len(STRATA)], lx + rng.uniform(-0.3, 0.3), cz, r, d * 0.45, y, y + th, lobes, k + (0 if side < 0 else 7))
            y += th
            k += 1
    # The span: three bands, each a row of short plates following a shallow bow.
    for band, (lift, th, fam) in enumerate(((0.0, 2.2, "rock_b"), (2.2, 0.9, "rock_c"), (3.1, 1.6, "rock_a"))):
        segs = 6
        for s in range(segs):
            t0, t1 = s / segs, (s + 1) / segs
            xa = x0 + 0.8 + (w - 1.6) * t0
            xb = x0 + 0.8 + (w - 1.6) * t1
            ya = FLOOR + leg_h - 1.0 + lift + 1.8 * math.sin(t0 * math.pi)
            yb = FLOOR + leg_h - 1.0 + lift + 1.8 * math.sin(t1 * math.pi)
            ym = (ya + yb) * 0.5
            plate(name, fam, (xa + xb) * 0.5, cz, (xb - xa) * 0.66, d * (0.42 - band * 0.04), ym, ym + th, lobes, s + band * 3, n=16)


# ---- KA, KB, KC: cardboard rock ---------------------------------------------------------------------------------------------------


OWN = set()        # pieces whose faces are turned outward as they are built (each block's own middle), not from one axis


def facet_ring(cx, cz, rx, rz, corners, rng, rot=None):
    """A block's outline: a few straight sides at uneven angles, so it reads as cut card, not a disc."""
    rot = rng.uniform(0, math.tau) if rot is None else rot
    angs = sorted(rot + math.tau * (i + rng.uniform(-0.28, 0.28)) / corners for i in range(corners))
    return [(cx + rx * math.cos(a) * rng.uniform(0.9, 1.05), cz + rz * math.sin(a) * rng.uniform(0.9, 1.05)) for a in angs]


def oriented(bm, pts, centre):
    f = face(bm, pts)
    f.normal_update()
    if f.normal.dot(f.calc_center_median() - centre) < 0.0:
        f.normal_flip()


def block(name, fam, ring, y0, y1, bevel):
    """A bevelled block: straight sides from y0 up to the bevel, a chamfer, and a flat top inset by the bevel."""
    OWN.add(name)
    bm = part(name, fam)
    cx = sum(q[0] for q in ring) / len(ring)
    cz = sum(q[1] for q in ring) / len(ring)
    centre = P(cx, (y0 + y1) * 0.5, cz)
    bevel = min(bevel, (y1 - y0) * 0.45)
    inset = []
    for (x, z) in ring:
        d = math.hypot(x - cx, z - cz) or 1.0
        f = max(0.2, 1.0 - bevel / d)
        inset.append((cx + (x - cx) * f, cz + (z - cz) * f))
    n = len(ring)
    yb = y1 - bevel
    for i in range(n):
        j = (i + 1) % n
        (xa, za), (xb, zb) = ring[i], ring[j]
        oriented(bm, (P(xa, y0, za), P(xb, y0, zb), P(xb, yb, zb), P(xa, yb, za)), centre)
        (ia, ka), (ib, kb) = inset[i], inset[j]
        oriented(bm, (P(xa, yb, za), P(xb, yb, zb), P(ib, y1, kb), P(ia, y1, ka)), centre)
    oriented(bm, [P(x, y1, z) for (x, z) in inset], centre)


def butte_blocks(name, b, rng, notched=False):
    """A butte as two or three tiers of bevelled blocks stepping in, each tier banded in the strata, on a low skirt block."""
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    rx, rz = (x1 - x0) * 0.5, (z1 - z0) * 0.5
    h = y1 - FLOOR
    corners = rng.randrange(5, 8)
    rot = rng.uniform(0, math.tau)
    block(name, "rock_b", facet_ring(cx, cz, rx * 0.66, rz * 0.62, corners + 1, rng, rot), FLOOR - 0.5, FLOOR + h * 0.12, 1.4)
    tiers = 3 if h > 14 else 2
    y = FLOOR + h * 0.12
    k = 0
    for t in range(tiers):
        sc = 0.54 - 0.09 * t
        top = FLOOR + h * (0.12 + 0.88 * (t + 1) / tiers)
        ox, oz = rng.uniform(-1.2, 1.2), rng.uniform(-0.6, 0.6)
        ring = facet_ring(cx + ox, cz + oz, rx * sc, rz * sc, corners, rng, rot + rng.uniform(-0.2, 0.2))
        last = t == tiers - 1
        if last and notched:
            # The crown split into blocks of different heights, as weathered rock breaks.
            parts = rng.randrange(2, 4)
            for q in range(parts):
                u0, u1 = q / parts, (q + 1) / parts
                bx = cx + ox + rx * sc * (u0 + u1 - 1.0) * 0.9
                ring_q = facet_ring(bx, cz + oz, rx * sc / parts * 1.05, rz * sc * 0.9, 5, rng)
                yq = top - (y1 - FLOOR) * rng.uniform(0.0, 0.12)
                block("%s_c%d" % (name, q), STRATA[(k + q) % len(STRATA)], ring_q, y - 0.2, yq, 0.9)
            break
        # (Banded: the tier's height in two or three strata, each a block on the last with its own small bevel.)
        bands = rng.randrange(2, 4)
        for q in range(bands):
            ya = y + (top - y) * q / bands
            yb_ = y + (top - y) * (q + 1) / bands
            shrink = 1.0 - 0.03 * q
            ring_q = [(cx + ox + (x - cx - ox) * shrink, cz + oz + (z - cz - oz) * shrink) for (x, z) in ring]
            block(name, STRATA[k % len(STRATA)], ring_q, ya, yb_, 0.8 if q == bands - 1 else 0.35)
            k += 1
        y = top
    if notched:
        fallen(name, cx, cz, rx * 0.62, rz * 0.6, h, rng)


def fallen(name, cx, cz, rx, rz, h, rng):
    """Blocks fallen from the butte, lying round its foot, the biggest nearest it."""
    for q in range(rng.randrange(3, 6)):
        a = rng.uniform(0, math.tau)
        d = rng.uniform(1.0, 1.35)
        bx, bz = cx + rx * d * math.cos(a), cz + rz * d * math.sin(a)
        sz = h * rng.uniform(0.06, 0.12)
        block("%s_f%d" % (name, q), rng.choice(STRATA), facet_ring(bx, bz, sz, sz * 0.8, rng.randrange(4, 6), rng), FLOOR - 0.3, FLOOR + sz * rng.uniform(0.9, 1.3), sz * 0.25)


def slab(name, fam, x0, x1, z0, z1, y0, y1, bevel=0.35):
    block(name, fam, [(x0, z0), (x1, z0), (x1, z1), (x0, z1)], y0, y1, bevel)


def butte_courses(name, b, rng):
    """A butte built as the stage's front is: courses of squared slabs of the strata, a course each band, stepping in near the top."""
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    rx, rz = (x1 - x0) * 0.5, (z1 - z0) * 0.5
    h = y1 - FLOOR
    y = FLOOR - 0.4
    k = 0
    courses = max(4, int(h / 2.6))
    for c in range(courses):
        t = c / (courses - 1)
        th = (y1 - y) / (courses - c)
        half = rx * (0.6 if c == 0 else (0.52 - 0.12 * max(0.0, t - 0.55) / 0.45)) * rng.uniform(0.96, 1.04)
        dep = rz * (0.58 if c == 0 else 0.48 - 0.08 * t)
        xa = cx - half + rng.uniform(-0.6, 0.6)
        xe = cx + half + rng.uniform(-0.6, 0.6)
        # (The course in two to four slabs of uneven length, a hairline gap between them, each a little in or out.)
        cuts = sorted(rng.uniform(0.2, 0.8) for _ in range(rng.randrange(1, 4)))
        xs = [xa] + [xa + (xe - xa) * u for u in cuts] + [xe]
        fam = STRATA[k % len(STRATA)] if c else "rock_b"
        for q in range(len(xs) - 1):
            dz = rng.uniform(-0.5, 0.5)
            slab("%s_s%d" % (name, c), fam, xs[q] + 0.08, xs[q + 1] - 0.08, cz - dep + dz, cz + dep + dz, y, y + th * rng.uniform(0.9, 1.0))
        y += th
        k += 1


def arch_rock(name, b, rng):
    """The arch as rock, not a stack (fidelity review 5: its legs of strata blocks read as stacked discs): each leg two massive blocks,
    the upper set a little in and turned, a thin pale band between them; the span one solid piece across their tops, its underside a
    true arch, its top uneven, a darker cap course along it."""
    x0, y0, z0, x1, y1, z1 = b
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    w, d, h = x1 - x0, z1 - z0, y1 - FLOOR
    leg_h = h * 0.72
    legs = []
    for side in (-1, 1):
        lx = cx + side * (w * 0.5 - 3.0)
        legs.append(lx)
        nm = "%s_l%d" % (name, side + 1)
        rot = rng.uniform(0, math.tau)
        lower = FLOOR + leg_h * 0.52
        block(nm, "rock_a", facet_ring(lx, cz, 3.5, d * 0.5, 7, rng, rot), FLOOR - 0.4, lower, 0.7)
        block(nm, "rock_c", facet_ring(lx + side * 0.2, cz, 3.25, d * 0.47, 7, rng, rot + 0.15), lower, lower + 0.7, 0.2)
        block(nm, "rock_a", facet_ring(lx + side * 0.35, cz + rng.uniform(-0.3, 0.3), 3.0, d * 0.44, 6, rng, rot + 0.4), lower + 0.7, FLOOR + leg_h, 0.6)
    # The span: a front outline (x, y) extruded through the arch's depth; its underside an arch between the legs' inner faces.
    xi_l, xi_r = legs[0] + 2.6, legs[1] - 2.6
    yb = FLOOR + leg_h - 0.6
    apex = yb + 2.2
    yt = FLOOR + leg_h + 3.6
    under = [(xi_l + (xi_r - xi_l) * k / 10.0, yb + (apex - yb) * math.sin(math.pi * k / 10.0)) for k in range(11)]
    top = []
    for k in range(9):
        x = x1 - 0.6 - (w - 1.2) * k / 8.0
        top.append((x, yt + rng.uniform(-0.45, 0.25)))
    if CHIP:
        # (Fidelity review 6, the arch read boxy: one end of the span broken off at a slant, and a notch stepped into its top.)
        top[-1] = (top[-1][0], yt - 1.6)
        top.insert(len(top) - 1, (top[-2][0] - (w - 1.2) / 16.0, yt - 0.3))
        n_at = 3
        xa_, xb_ = top[n_at][0], top[n_at + 1][0]
        top[n_at + 1:n_at + 1] = [(xa_ - (xa_ - xb_) * 0.3, yt - 0.9), (xa_ - (xa_ - xb_) * 0.7, yt - 0.9)]
    outline = [(x0 + 0.6, yb)] + under + [(x1 - 0.6, yb)] + top
    dd = d * 0.4
    OWN.add(name + "_span")
    bm = part(name + "_span", "rock_a")
    zf, zb = cz + dd, cz - dd
    front = face(bm, [P(x, y, zf) for (x, y) in outline])
    front.normal_update()
    if front.normal.y > 0.0:
        front.normal_flip()
    back = face(bm, [P(x, y, zb) for (x, y) in reversed(outline)])
    back.normal_update()
    if back.normal.y < 0.0:
        back.normal_flip()
    n = len(outline)
    for i in range(n):
        (xa, ya), (xb, yb_) = outline[i], outline[(i + 1) % n]
        f = face(bm, (P(xa, ya, zf), P(xb, yb_, zf), P(xb, yb_, zb), P(xa, ya, zb)))
        f.normal_update()
        # (Outward: to the right of the outline's direction, the outline running counter-clockwise.)
        mid = P((xa + xb) * 0.5, (ya + yb_) * 0.5, cz)
        if f.normal.dot(P((xa + xb) * 0.5 + (yb_ - ya), (ya + yb_) * 0.5 - (xb - xa), cz) - mid) < 0.0:
            f.normal_flip()
    if CHIP:
        # The broken-off end, fallen at the left leg's foot.
        block(name + "_f", "rock_a", facet_ring(x0 - 1.5, cz + d * 0.6, 2.2, 1.6, 5, rng), FLOOR - 0.3, FLOOR + 2.4, 0.5)
    # The cap course along the top.
    for k in range(4 if not CHIP else 3):
        xa = x0 + 0.8 + (w - 1.6) * k / 4.0
        xb = x0 + 0.8 + (w - 1.6) * (k + 1) / 4.0
        ytop = yt + rng.uniform(-0.2, 0.3)
        slab("%s_cap%d" % (name, k), "rock_b", xa + 0.1, xb - 0.1, cz - dd * 0.9, cz + dd * 0.9, ytop - 0.2, ytop + 0.9, 0.3)


def butte_rock(name, b, rng, wide=False):
    """A butte as rock, as the arch is (fidelity review 6: banded a block each, every butte read as a stack of discs): two or three
    massive faceted tiers of few sides, each set off the one under it and stepping well in, a thin pale band between them and a dark cap
    course on top, on a low skirt; `wide` a broad flat-topped mesa, one tier with a pale band through it (the butte behind the fight)."""
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    rx, rz = (x1 - x0) * 0.5, (z1 - z0) * 0.5
    if wide:
        # (Wider than deep, a flat face toward the camera: a five-sided ring at a random turn read as a round drum.)
        # (And lower, seven tenths: at the butte's full height it stood taller than wide, a tower, not a table.)
        rx *= 1.6
        rz *= 0.75
    h = (y1 - FLOOR) * (0.7 if wide else 1.0)
    rot = math.pi * 0.25 if wide else rng.uniform(0, math.tau)
    block(name, "rock_b", facet_ring(cx, cz, rx * 0.7, rz * 0.64, 6, rng, rot), FLOOR - 0.5, FLOOR + h * 0.1, 1.4)
    scales = [0.6] if wide else ([0.56, 0.42, 0.3] if h > 14 else [0.56, 0.38])
    tiers = len(scales)
    y = FLOOR + h * 0.1
    cap_h = 1.0
    ring = None
    for t, sc in enumerate(scales):
        top = FLOOR + h * (0.1 + 0.9 * (t + 1) / tiers) - (cap_h if t == tiers - 1 else 0.0)
        ox = rng.uniform(-0.14, 0.14) * rx if t else 0.0
        ring = facet_ring(cx + ox, cz + rng.uniform(-0.4, 0.4), rx * sc, rz * sc, 4 if wide else rng.randrange(4, 6), rng,
                          rot + (0.0 if wide else rng.uniform(-0.35, 0.35)))
        if wide:
            # (One tier, a pale band through it a little over halfway, as the arch's legs.)
            mid = y + (top - y) * 0.56
            block(name, "rock_a", ring, y, mid, 0.5)
            block(name, "rock_c", scaled(ring, 0.97), mid, mid + 0.8, 0.2)
            block(name, "rock_a", scaled(ring, 0.95), mid + 0.8, top, 0.6)
        else:
            block(name, "rock_a", ring, y, top, 0.8)
            if t < tiers - 1:
                # The pale band the next tier stands on.
                block(name, "rock_c", scaled(ring, 0.86), top, top + 0.6, 0.15)
                top += 0.6
        y = top
    # The dark cap course, and on a tall one a lump of the cap left standing to one side.
    cap = scaled(ring, 0.96)
    block(name, "rock_b", cap, y, y + cap_h, 0.3)
    if not wide and h > 12 and rng.random() < 0.6:
        ccx = sum(q[0] for q in cap) / len(cap)
        ccz = sum(q[1] for q in cap) / len(cap)
        side = rng.choice((-1, 1))
        lump = facet_ring(ccx + side * rx * scales[-1] * 0.45, ccz, rx * scales[-1] * 0.4, rz * scales[-1] * 0.5, 5, rng)
        block(name + "_c", "rock_b", lump, y + cap_h - 0.2, y + cap_h + h * 0.06, 0.4)
    fallen(name, cx, cz, rx * 0.66, rz * 0.62, h, rng)


def butte_broken(name, b, rng):
    """The butte behind the fight as a broken table of rock (fidelity review 7: the wide mesa read as a round pink drum behind the
    fighters): three blocks of uneven height side by side with a crevice between each, the middle one a step lower so the skyline is
    notched, each in the strata (a pale band through it, a dark cap), on a low skirt, fallen blocks at its foot; wider than tall."""
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    # (Wide and low: at 1.7 the width and 0.62 the height each block stood taller than wide, three chimneys.)
    rx, rz = (x1 - x0) * 0.5 * 2.5, (z1 - z0) * 0.5 * 0.7
    h = (y1 - FLOOR) * 0.42
    block(name, "rock_b", facet_ring(cx, cz, rx * 0.75, rz * 0.75, 6, rng, 0.0), FLOOR - 0.5, FLOOR + h * 0.08, 1.2)
    shares = [0.36, 0.3, 0.34]
    heights = [1.0, 0.8, 0.92]
    left = cx - rx * 0.62
    span = rx * 1.24
    x = left
    for k, (sh, hk) in enumerate(zip(shares, heights)):
        w = span * sh
        bx = x + w * 0.5
        top = FLOOR + h * hk * rng.uniform(0.95, 1.05)
        ring = facet_ring(bx, cz + rng.uniform(-0.6, 0.6), w * 0.5 - 0.35, rz * rng.uniform(0.6, 0.75), rng.randrange(4, 6), rng, rng.uniform(-0.25, 0.25))
        y = FLOOR + h * 0.08
        mid = y + (top - y) * rng.uniform(0.45, 0.6)
        block(name, "rock_a", ring, y, mid, 0.4)
        block(name, "rock_c", scaled(ring, 0.97), mid, mid + 0.7, 0.15)
        block(name, "rock_a", scaled(ring, 0.94), mid + 0.7, top - 0.9, 0.5)
        block(name, "rock_b", scaled(ring, 0.9), top - 0.9, top, 0.3)
        x += w
    fallen(name, cx, cz, rx * 0.7, rz * 0.9, h, rng)


def scaled(ring, f):
    cx = sum(q[0] for q in ring) / len(ring)
    cz = sum(q[1] for q in ring) / len(ring)
    return [(cx + (x - cx) * f, cz + (z - cz) * f) for (x, z) in ring]


def outcrops(rng):
    """Low rock ledges and boulders across the plain (fidelity review 6: at the wide camera the plain was empty): a ledge of one or two
    flat tiers with a pale band, a boulder or two by it; off the buttes' feet, the dry wash and the strip behind the fight."""
    feet = [((m[0] + m[3]) * 0.5, (m[2] + m[5]) * 0.5, (m[3] - m[0]) * 0.55, (m[5] - m[2]) * 0.55) for m in LAYOUT.get("mesas", [])]
    a = LAYOUT.get("arch")
    if a:
        feet.append(((a[0] + a[3]) * 0.5, (a[2] + a[5]) * 0.5, (a[3] - a[0]) * 0.7, (a[5] - a[2]) * 1.2))
    line = wash_line()
    placed = []
    tries = 0
    # (Within the wide camera's view: spread across the whole plain, most fell out of frame.)
    while len(placed) < 14 and tries < 3000:
        tries += 1
        x, z = rng.uniform(-95.0, 95.0), rng.uniform(-95.0, -14.0)
        if abs(x) < 32.0:
            continue
        if any(((x - fx) / fr) ** 2 + ((z - fz) / fd) ** 2 < 1.0 for (fx, fz, fr, fd) in feet):
            continue
        if near_wash(line, x, z, 9.0) or any(math.hypot(x - px, z - pz) < 16.0 for (px, pz) in placed):
            continue
        placed.append((x, z))
        k = len(placed) - 1
        far = (-z - 14.0) / 80.0
        rx, rz = rng.uniform(4.0, 8.0) * (1.0 + 0.6 * far), rng.uniform(2.0, 3.6) * (1.0 + 0.4 * far)
        hh = rng.uniform(1.4, 2.6) * (1.0 + 0.5 * far)
        y = seat(x, z) - 0.3
        nm = "outcrop%d" % k
        rot = rng.uniform(-0.4, 0.4)
        ring = facet_ring(x, z, rx, rz, rng.randrange(5, 7), rng, rot)
        block(nm, "rock_a", ring, y, y + hh, 0.4)
        if rng.random() < 0.6:
            block(nm, "rock_c", scaled(ring, 0.8), y + hh, y + hh + 0.35, 0.1)
            block(nm, "rock_a", facet_ring(x + rng.uniform(-0.3, 0.3) * rx, z, rx * 0.5, rz * 0.6, 5, rng), y + hh + 0.35, y + hh * 1.7, 0.3)
        for q in range(rng.randrange(1, 3)):
            ang = rng.uniform(0, math.tau)
            bx, bz = x + rx * 1.15 * math.cos(ang), z + rz * 1.3 * math.sin(ang)
            sz = hh * rng.uniform(0.35, 0.6)
            block(nm, rng.choice(("rock_a", "rock_b")), facet_ring(bx, bz, sz, sz * 0.8, rng.randrange(4, 6), rng), seat(bx, bz) - 0.2,
                  seat(bx, bz) + sz * 1.1, sz * 0.25)
        shade(nm + "_shade", x, seat(x, z), z, max(rx, rz) * 1.1)


def mesa_foot(rng):
    """The stage mesa's foot (C3): either side, a fan of scree a step darker than the sand, and on it fallen blocks of the mesa's strata,
    the biggest against the wall and smaller out across the fan."""
    cx = (STAGE[0] + STAGE[1]) * 0.5
    half = (STAGE[1] - STAGE[0]) * 0.5 * 1.04
    for side in (-1.0, 1.0):
        fx = cx + side * (half + 3.5)
        floor_sheet("scree%d" % (side > 0), "scree", oblong(fx, STAGE_FZ - 2.5, 7.5, 5.5, rng), FLOOR + 0.42)
        for q in range(12):
            d = rng.uniform(0.0, 1.0) ** 1.4
            x = cx + side * (half + 0.4 + d * 8.5)
            z = STAGE_FZ - rng.uniform(-1.5, 6.5) + d * 1.5
            sz = (2.4 - 1.9 * d) * rng.uniform(0.7, 1.15)
            y = seat(x, z) - 0.2
            block("talus%d_%d" % (side > 0, q), rng.choice(STRATA), facet_ring(x, z, sz, sz * 0.75, rng.randrange(4, 7), rng,
                  rng.uniform(0, math.tau)), y, y + sz * rng.uniform(0.7, 1.2), sz * 0.22)
        shade("talus%d_shade" % (side > 0), cx + side * (half + 2.0), FLOOR + 0.45, STAGE_FZ - 1.5, 4.5)


def hoodoos(rng):
    """Clusters of hoodoos either side of the mesa (C2), at true size: each column a stack of strata blocks narrowing upward under a
    wider pale-banded cap stone, two to four to a cluster, boulders round them and their shade; off the wash, the arch and the buttes'
    feet, and clear of the strip under the fight."""
    feet = [((m[0] + m[3]) * 0.5, (m[2] + m[5]) * 0.5, (m[3] - m[0]) * 0.6, (m[5] - m[2]) * 0.6) for m in LAYOUT.get("mesas", [])]
    a = LAYOUT.get("arch")
    if a:
        feet.append(((a[0] + a[3]) * 0.5, (a[2] + a[5]) * 0.5, (a[3] - a[0]) * 0.8, (a[5] - a[2]) * 1.4))
    line = wash_line()
    placed = []
    tries = 0
    while len(placed) < HOODOOS and tries < 4000:
        tries += 1
        side = -1.0 if len(placed) % 2 == 0 else 1.0
        x, z = side * rng.uniform(22.0, 58.0), rng.uniform(-34.0, -4.0)
        if any(((x - fx) / fr) ** 2 + ((z - fz) / fd) ** 2 < 1.0 for (fx, fz, fr, fd) in feet):
            continue
        if near_wash(line, x, z, 7.0) or any(math.hypot(x - px, z - pz) < 18.0 for (px, pz) in placed):
            continue
        placed.append((x, z))
        k = len(placed) - 1
        for c in range(rng.randrange(2, 5)):
            hx, hz = x + rng.uniform(-3.5, 3.5), z + rng.uniform(-2.0, 2.0)
            h = rng.uniform(4.0, 9.0) * (1.0 - 0.25 * c / 4.0)
            r = rng.uniform(0.9, 1.4)
            y = seat(hx, hz) - 0.2
            nm = "hoodoo%d_%d" % (k, c)
            rot = rng.uniform(0, math.tau)
            bands = rng.randrange(3, 5)
            for q in range(bands):
                ya, yb = y + h * q / bands, y + h * (q + 1) / bands
                # (Narrowing up the column, a waist below the cap where the soft rock wore away.)
                rr = r * (1.0 - 0.18 * q / bands) * (0.85 if q == bands - 1 else 1.0)
                block(nm, STRATA[(q + k) % len(STRATA)], facet_ring(hx, hz, rr, rr * 0.85, 6, rng, rot), ya, yb, 0.15)
            block(nm, "rock_c", facet_ring(hx, hz, r * 1.35, r * 1.15, 5, rng, rot), y + h, y + h + 0.35, 0.08)
            block(nm, "rock_b", facet_ring(hx + rng.uniform(-0.2, 0.2), hz, r * 1.25, r * 1.05, 5, rng, rot), y + h + 0.35, y + h + 1.1, 0.3)
        for q in range(rng.randrange(3, 6)):
            ang = rng.uniform(0, math.tau)
            bx, bz = x + 5.0 * math.cos(ang), z + 3.0 * math.sin(ang)
            sz = rng.uniform(0.6, 1.5)
            block("hoodoo%d_b" % k, rng.choice(STRATA), facet_ring(bx, bz, sz, sz * 0.8, rng.randrange(4, 6), rng), seat(bx, bz) - 0.2,
                  seat(bx, bz) + sz * 1.1, sz * 0.25)
        shade("hoodoo%d_shade" % k, x, seat(x, z), z, 6.0)


def arch_blocks(name, b, rng, courses=False):
    """The arch: two legs of blocks (or slab courses) and a span of three bevelled beams bowing across their tops."""
    x0, y0, z0, x1, y1, z1 = b
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    w, d, h = x1 - x0, z1 - z0, y1 - FLOOR
    leg_h = h * 0.72
    for side in (-1, 1):
        lx = cx + side * (w * 0.5 - 3.0)
        y = FLOOR - 0.3
        k = 0
        while y < FLOOR + leg_h - 0.5:
            th = rng.uniform(2.0, 2.8)
            r_ = 3.1 - 0.5 * (y - FLOOR) / leg_h
            nm = "%s_l%d" % (name, side + 1)
            if courses:
                slab(nm, STRATA[k % len(STRATA)], lx - r_, lx + r_, cz - d * 0.42, cz + d * 0.42, y, y + th)
            else:
                block(nm, STRATA[k % len(STRATA)], facet_ring(lx + rng.uniform(-0.3, 0.3), cz, r_, d * 0.45, 6, rng), y, y + th, 0.6)
            y += th
            k += 1
    for band, (lift, th, fam) in enumerate(((0.0, 2.2, "rock_b"), (2.2, 1.0, "rock_c"), (3.2, 1.6, "rock_a"))):
        segs = 3
        for sgm in range(segs):
            t0, t1 = sgm / segs, (sgm + 1) / segs
            xa = x0 + 0.6 + (w - 1.2) * t0
            xb = x0 + 0.6 + (w - 1.2) * t1
            ym = FLOOR + leg_h - 1.0 + lift + 1.6 * math.sin((t0 + t1) * 0.5 * math.pi)
            dd = d * (0.42 - band * 0.04)
            slab("%s_span%d" % (name, band), fam, xa + 0.05, xb - 0.05, cz - dd, cz + dd, ym, ym + th, 0.5)


# ---- Pop-up cut-outs ----------------------------------------------------------------------------------------------------------------


def cutout(name, fam, profile, z, bottom, thick=0.6, hole=None):
    """A standing card: its top edge `profile` [(x, y)], down to `bottom`, at depth `z`, with the cut edge along the top showing its
    thickness. `hole` (x0, x1, y_top): an opening cut out of it from the bottom up to y_top."""
    for i in range(len(profile) - 1):
        (xa, ya), (xb, yb) = profile[i], profile[i + 1]
        lo_a, lo_b = bottom, bottom
        if hole and xa >= hole[0] and xb <= hole[1]:
            lo_a = lo_b = hole[2]
            face(part(name, "card_" + fam), (P(xa, lo_a, z), P(xa, lo_a, z - thick), P(xb, lo_b, z - thick), P(xb, lo_b, z)))
        face(part(name, fam), (P(xa, ya, z), P(xa, lo_a, z), P(xb, lo_b, z), P(xb, ya, z)))
        face(part(name, "card_" + fam), (P(xa, ya, z - thick), P(xa, ya, z), P(xb, yb, z), P(xb, yb, z - thick)))


def stepped_profile(cx, half, top, rng, tiers=4, skirt=None):
    """A butte's outline as steps: each band a little narrower, its corners squared off, over a sloping skirt."""
    pts = []
    h = top - FLOOR
    ys = [FLOOR + h * (0.16 + 0.84 * k / tiers) for k in range(tiers + 1)]
    widths = [half * (1.0 - 0.06 * k) * rng.uniform(0.96, 1.04) for k in range(tiers)]
    skirt = skirt or half * 1.7
    pts.append((cx - skirt, FLOOR))
    pts.append((cx - widths[0] - 0.4, ys[0]))
    for k in range(tiers):
        pts.append((cx - widths[k], ys[k]))
        pts.append((cx - widths[k], ys[k + 1]))
    for k in reversed(range(tiers)):
        pts.append((cx + widths[k], ys[k + 1]))
        pts.append((cx + widths[k], ys[k]))
    pts.append((cx + widths[0] + 0.4, ys[0]))
    pts.append((cx + skirt, FLOOR))
    # (Sorted along x for the columns of the card: equal x at a step is a vertical edge.)
    return pts


def butte_cutouts(name, b, rng):
    x0, y0, z0, x1, y1, z1 = b[:6]
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    half = (x1 - x0) * 0.5 * 0.52
    # (The palest card at the back, the deepest in front as the base: the pale one in front washed the buttes out.)
    for k, (dz, sc, fam) in enumerate(((-(z1 - z0) * 0.2, 1.0, "rock_c"), (0.0, 0.9, "rock_a"), ((z1 - z0) * 0.2, 0.62, "rock_b"))):
        top = FLOOR + (y1 - FLOOR) * sc
        prof = stepped_profile(cx + rng.uniform(-1.5, 1.5), half * (0.8 + 0.2 * sc), top, rng)
        prof = monotone(prof)
        cutout("%s_%d" % (name, k), fam, prof, cz + dz, FLOOR - 1.0)


def monotone(pts):
    """Columns for a card from an outline that may step straight up: nudge equal xs apart so each column has width."""
    out = []
    for x, y in pts:
        if out and x <= out[-1][0]:
            x = out[-1][0] + 0.02
        out.append((x, y))
    return out


def arch_cutout(name, b, rng):
    x0, y0, z0, x1, y1, z1 = b
    cx, cz = (x0 + x1) * 0.5, (z0 + z1) * 0.5
    w = x1 - x0
    prof = []
    n = 24
    for i in range(n + 1):
        u = i / n
        x = x0 + w * u
        y = y1 - 0.8 * abs(math.sin(u * 7.0)) - (2.0 if u < 0.08 or u > 0.92 else 0.0)
        prof.append((x, y))
    opening = (x0 + 5.0, x1 - 5.0, FLOOR + (y1 - FLOOR) * 0.72)
    for k, (dz, fam) in enumerate(((-1.2, "rock_b"), (0.6, "rock_a"))):
        cutout("%s_%d" % (name, k), fam, prof, cz + dz, FLOOR - 1.0, hole=opening)


# ---- Dunes --------------------------------------------------------------------------------------------------------------------------


def dune_flats():
    for r_i, spec in enumerate(LAYOUT.get("ranges", [])):
        fam = "dune_far" if r_i == 0 else "dune_near"
        raw = spec["top"]
        peak = [max(raw[j][1] for j in range(max(0, i - 1), min(len(raw), i + 2))) for i in range(len(raw))]
        top = []
        for i in range(len(raw)):
            near = peak[max(0, i - 2): i + 3]
            top.append((raw[i][0], sum(near) / len(near)))
        z = spec["z"]
        base = min(t[1] for t in top) - 6.0
        for k, (dz, scale) in enumerate(((-8.0, 1.08), (0.0, 1.0), (8.0, 0.9))):
            pts = [(x, base + (y - base) * scale) for (x, y) in top]
            cutout("range%d_%d" % (r_i, k), fam, pts, z + dz, base - 20.0, thick=0.8)


# ---- DA, DB, DC: the desert floor in paper ------------------------------------------------------------------------------------------


SHEETS = []        # (ring, y): the floor's sheets, for seating things on the topmost under them


def oblong(cx, cz, rx, rz, rng, n=22):
    """A sheet's cut outline in (x, z): a long rounded oblong along the floor, its edges a little hand-cut."""
    p1, p2 = rng.uniform(0, math.tau), rng.uniform(0, math.tau)
    out = []
    for i in range(n):
        a = math.tau * i / n
        c, s_ = math.cos(a), math.sin(a)
        w = 1.0 + 0.05 * math.sin(3 * a + p1) + 0.03 * math.sin(7 * a + p2)
        out.append((cx + rx * w * math.copysign(abs(c) ** 0.5, c), cz + rz * w * math.copysign(abs(s_) ** 0.7, s_)))
    return out


def floor_sheet(name, fam, ring, y):
    """A cut sheet lying on the floor: its top a fan from the middle, its cut edge down to the sand."""
    cx = sum(q[0] for q in ring) / len(ring)
    cz = sum(q[1] for q in ring) / len(ring)
    n = len(ring)
    for i in range(n):
        j = (i + 1) % n
        face(part(name, fam), (P(cx, y, cz), P(ring[i][0], y, ring[i][1]), P(ring[j][0], y, ring[j][1])))
        face(part(name, "card_" + fam), (P(ring[i][0], y, ring[i][1]), P(ring[i][0], FLOOR - 0.05, ring[i][1]),
                                          P(ring[j][0], FLOOR - 0.05, ring[j][1]), P(ring[j][0], y, ring[j][1])))
    SHEETS.append((ring, y))


def inside(ring, x, z):
    hit = False
    for i in range(len(ring)):
        (xa, za), (xb, zb) = ring[i], ring[i - 1]
        if (za > z) != (zb > z) and x < xa + (z - za) * (xb - xa) / (zb - za):
            hit = not hit
    return hit


def seat(x, z):
    """The height of the topmost sheet under (x, z), or the sand's."""
    return max([y for (ring, y) in SHEETS if inside(ring, x, z)], default=FLOOR)


def ripples(name, ring, y, rng):
    """Wind ripples on a sheet: two or three pale strips following its front edge, inset."""
    cx = sum(q[0] for q in ring) / len(ring)
    cz = sum(q[1] for q in ring) / len(ring)
    front = [q for q in ring if q[1] > cz]
    front.sort()
    for k, f in enumerate((0.75, 0.5, 0.28)[:rng.randrange(2, 4)]):
        pts = [(cx + (x - cx) * f, cz + (z - cz) * f) for (x, z) in front]
        for i in range(len(pts) - 1):
            (xa, za), (xb, zb) = pts[i], pts[i + 1]
            face(part(name, "ripple"), (P(xa, y + 0.02, za), P(xb, y + 0.02, zb), P(xb, y + 0.02, zb - 0.6), P(xa, y + 0.02, za - 0.6)))


# (Through the gap between the near buttes right of the stage, where the cameras see it: past x 60 is off the frame.)
# (Through the gaps between the buttes, the believability pass, 2026-10-22: from (72, -150) it ran under the far butte at (44, -112).)
WASH = [(14.0, -150.0), (18.0, -125.0), (22.0, -100.0), (30.0, -80.0), (38.0, -58.0), (44.0, -32.0), (50.0, -10.0), (58.0, 14.0)]


def wash_line(step=2.0):
    """The dry wash's centre line, smoothed (Catmull-Rom through WASH) and resampled every `step`."""
    pts = []
    w = [WASH[0]] + WASH + [WASH[-1]]
    for i in range(1, len(w) - 2):
        p0, p1, p2, p3 = w[i - 1], w[i], w[i + 1], w[i + 2]
        seg = math.dist(p1, p2)
        for k in range(max(2, int(seg / step))):
            t = k / max(2, int(seg / step))
            pts.append(tuple(0.5 * ((2 * p1[c]) + (-p0[c] + p2[c]) * t + (2 * p0[c] - 5 * p1[c] + 4 * p2[c] - p3[c]) * t * t
                                    + (-p0[c] + 3 * p1[c] - 3 * p2[c] + p3[c]) * t ** 3) for c in (0, 1)))
    pts.append(WASH[-1])
    return pts


def dry_wash(rng):
    """A pale wash winding in from the far right to the front: its bed a sheet over the dunes, darker banks either side, pebbles on them."""
    line = wash_line()
    y = 0.75 + FLOOR
    left, right, bl, br = [], [], [], []
    for i, (x, z) in enumerate(line):
        a, b = line[max(0, i - 1)], line[min(len(line) - 1, i + 1)]
        tx, tz = b[0] - a[0], b[1] - a[1]
        ln = math.hypot(tx, tz) or 1.0
        nx, nz = -tz / ln, tx / ln
        t = (z + 150.0) / 166.0
        half = 2.0 + 2.6 * t + 0.4 * math.sin(i * 0.7)
        left.append((x + nx * half, z + nz * half))
        right.append((x - nx * half, z - nz * half))
        bl.append((x + nx * (half + 0.9), z + nz * (half + 0.9)))
        br.append((x - nx * (half + 0.9), z - nz * (half + 0.9)))
        if rng.random() < 0.5:
            side = rng.choice((1.0, -1.0))
            sc = rng.uniform(0.3, 0.6) * (1.0 + t)
            res = bmesh.ops.create_icosphere(part("wash_pebbles", "pebble"), subdivisions=1, radius=sc)
            for v in res["verts"]:
                v.co.z *= 0.55
                v.co += P(x + side * nx * (half + 0.4), y + sc * 0.2, z + side * nz * (half + 0.4))
    for i in range(len(line) - 1):
        face(part("wash", "wash"), (P(left[i][0], y, left[i][1]), P(left[i + 1][0], y, left[i + 1][1]),
                                    P(right[i + 1][0], y, right[i + 1][1]), P(right[i][0], y, right[i][1])))
        for (o, n_) in ((bl, left), (br, right)):
            face(part("wash", "card_wash"), (P(o[i][0], y - 0.15, o[i][1]), P(o[i + 1][0], y - 0.15, o[i + 1][1]),
                                             P(n_[i + 1][0], y, n_[i + 1][1]), P(n_[i][0], y, n_[i][1])))
            face(part("wash", "card_wash"), (P(o[i][0], y - 0.15, o[i][1]), P(o[i + 1][0], y - 0.15, o[i + 1][1]),
                                             P(o[i + 1][0], FLOOR - 0.05, o[i + 1][1]), P(o[i][0], FLOOR - 0.05, o[i][1])))
    return line


def near_wash(line, x, z, pad):
    return any(math.hypot(x - a, z - b) < pad for (a, b) in line[::2])


def scrub(name, x, y, z, sc, rng):
    """A clump of desert scrub in paper: five or six upright card leaves round a point, and the soft shade under it."""
    for k in range(rng.randrange(5, 7)):
        a = math.tau * k / 6 + rng.uniform(-0.3, 0.3)
        lean = rng.uniform(0.25, 0.5)
        dx, dz = math.cos(a), math.sin(a)
        w = 0.32 * sc
        h = sc * rng.uniform(0.8, 1.1)
        face(part(name, "scrub"), (P(x - dz * w, y, z + dx * w), P(x + dz * w, y, z - dx * w),
                                   P(x + dx * h * lean + dz * w * 0.4, y + h * 0.7, z + dz * h * lean - dx * w * 0.4),
                                   P(x + dx * h * lean * 1.4, y + h, z + dz * h * lean * 1.4),
                                   P(x + dx * h * lean - dz * w * 0.4, y + h * 0.7, z + dz * h * lean + dx * w * 0.4)))
    shade(name, x, y, z, sc * 0.9)


def shade(name, x, y, z, rad):
    for i in range(12):
        a, b = math.tau * i / 12, math.tau * (i + 1) / 12
        face(part(name, "sand_shade"), (P(x + rad * 0.15, y + 0.03, z - rad * 0.1), P(x + rad * 0.15 + rad * 1.3 * math.cos(a), y + 0.03, z - rad * 0.1 + rad * 0.8 * math.sin(a)),
                                        P(x + rad * 0.15 + rad * 1.3 * math.cos(b), y + 0.03, z - rad * 0.1 + rad * 0.8 * math.sin(b))))


def desert_floor(rng, level=None):
    level = level or {"DA": 1, "DB": 2, "DC": 3}[VARIANT]
    tones = ["sand_a", "sand_b", "sand_c"]
    k = 0
    z = 10.0
    row = 0
    while z > -150.0:
        far = (10.0 - z) / 160.0
        rz = rng.uniform(4.0, 6.5) * (1.0 + 0.8 * far)
        x = -215.0 + rng.uniform(0, 14)
        while x < 215.0:
            rx = rng.uniform(14.0, 28.0) * (1.0 + 0.5 * far)
            if rng.random() < 0.78:
                ring = oblong(x + rx, z + rng.uniform(-1.5, 1.5), rx, rz, rng)
                y = FLOOR + 0.12 + 0.12 * ((k + row) % 3)
                floor_sheet("sheet%d" % (k // 24), tones[(k + row) % 3], ring, y)
                if rng.random() < 0.45:
                    ripples("ripple%d" % (k // 24), ring, y, rng)
                k += 1
            x += rx * 2.0 * rng.uniform(0.75, 1.05)
        z -= rz * 1.25
        row += 1
    line = dry_wash(rng) if level >= 2 else []
    if level >= 3:
        # Scrub and stones over the floor, bigger with distance as the code-built scatter was, kept off the wash and the buttes' feet.
        feet = [((m[0] + m[3]) * 0.5, (m[2] + m[5]) * 0.5, (m[3] - m[0]) * 0.35, (m[5] - m[2]) * 0.35) for m in LAYOUT.get("mesas", [])]
        placed = 0
        tries = 0
        while placed < 70 and tries < 2000:
            tries += 1
            x, z = rng.uniform(-150.0, 150.0), rng.uniform(-135.0, 12.0)
            if abs(x) < 30 and z > -24.0:
                continue
            if any(((x - fx) / fr) ** 2 + ((z - fz) / fd) ** 2 < 1.0 for (fx, fz, fr, fd) in feet):
                continue
            if line and near_wash(line, x, z, 7.0):
                continue
            sc = rng.uniform(1.0, 1.8) * (1.0 + 0.8 * (-z / 135.0 if z < 0 else 0.0))
            y = seat(x, z)
            if placed % 3 == 0:
                res = bmesh.ops.create_icosphere(part("scatter_stones", "pebble"), subdivisions=1, radius=sc * 0.7)
                for v in res["verts"]:
                    v.co.z *= 0.6
                    v.co += P(x, y + sc * 0.2, z)
                shade("scatter_stones", x, y, z, sc * 0.8)
            else:
                scrub("scatter_scrub", x, y, z, sc, rng)
            placed += 1


# ---- Build --------------------------------------------------------------------------------------------------------------------------


def build_ground():
    rng = random.Random(53)
    if VARIANT in ("DA", "DB", "DC"):
        desert_floor(random.Random(61))
    if VARIANT in ("KA", "KB", "KC"):
        # (On the floor as shipped, DC's.)
        desert_floor(random.Random(61), level=3)
        for k, m in enumerate(LAYOUT.get("mesas", [])):
            if VARIANT == "KB":
                butte_courses("butte%d" % k, m, rng)
            elif opts.get("buttes", "") == "solid":
                # (The one behind the fight: in the front row, across the middle.)
                behind = m[6] == 0 and m[0] < 9.0 < m[3]
                if behind and opts.get("fight", "") == "broken":
                    # (Drawn and thrown away, so the random numbers of every butte after it stay as they were.)
                    butte_rock("discard%d" % k, m, rng, wide=True)
                    for key in [key for key in PARTS if key[0].startswith("discard")]:
                        PARTS.pop(key).free()
                    OWN.difference_update({n for n in OWN if n.startswith("discard")})
                    butte_broken("butte%d" % k, m, random.Random(97))
                else:
                    butte_rock("butte%d" % k, m, rng, wide=behind and opts.get("fight", "") == "wide")
            else:
                butte_blocks("butte%d" % k, m, rng, notched=VARIANT == "KC")
        if opts.get("plain", "") == "outcrops":
            outcrops(random.Random(71))
        # (New art toward 5: their own random numbers, so the rest keeps its layout.)
        if FOOT == "talus":
            mesa_foot(random.Random(301))
        if HOODOOS:
            hoodoos(random.Random(307))
        if LAYOUT.get("arch"):
            # (`--arch solid`: fidelity review 5's rock arch, `arch_rock`; `--arch chipped` it broken, review 6.)
            if opts.get("arch", "") in ("solid", "chipped"):
                arch_rock("arch", LAYOUT["arch"], rng)
            else:
                arch_blocks("arch", LAYOUT["arch"], rng, courses=VARIANT == "KB")
        dune_flats()
        return
    for k, m in enumerate(LAYOUT.get("mesas", [])):
        far = m[6] == 1
        name = "butte%d" % k
        if VARIANT == "B" or (VARIANT == "C" and far):
            butte_cutouts(name, m, rng)
        else:
            butte_stack(name, m, rng)
    if LAYOUT.get("arch"):
        if VARIANT == "B":
            arch_cutout("arch", LAYOUT["arch"], rng)
        else:
            arch_stack("arch", LAYOUT["arch"], rng)
    dune_flats()


def make_objects():
    for (name, mat), bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        ax = AXES.get(name)
        for f in ([] if name in OWN else bm.faces):
            c = f.calc_center_median()
            if ax is not None:
                # (A stack's plates: tops up, cut edges out from its middle.)
                if abs(f.normal.z) > 0.9:
                    continue
                away = Vector((c.x - ax[0], c.y - ax[1], 0.0))
            else:
                # (A cut-out faces the camera; its top edge faces up.)
                away = Vector((0.0, -1.0, 0.4))
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
        for c in o.data.color_attributes["Col"].data:
            ao = 0.45 + 0.55 * c.color[0]
            c.color = (ao, ao, ao, 1.0)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_ground()
    make_objects()
    bake_occlusion(2.0)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    path = os.path.abspath(opts["ground"])
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("canyon paper %s: %d triangles -> %s" % (VARIANT, tris, path))


main()
