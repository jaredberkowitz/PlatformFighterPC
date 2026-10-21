"""The meadow's background land and pines as modelled pieces (the stage refinement, after the grove's ground and trees;
docs/art/stages/BAKE_PILOT.md). Built in world space from the layout the game exports (`ground.json` beside the bake manifest: the hill
rows' and the rise's bumps, where the pine clusters stand), so the crests stay where the windmill, the cottages and the pines stand.
The far mountain ranges, the windmill and the cottages stay code-placed. Three looks for the owner to pick from (`--ground-mock=`,
`--bush-mock=` on the stage demo; the "bush" is the pine cluster, placed at each cluster's spot and size):
  A  deep folds: the hill rows and the rise pleated from under the crest down in irregular ridges and creases, the fields cut into the
     rise's folds with raised hedgerows between them; pines of six crisp pleated tiers;
  B  paper layers: the land as stacked sheets of card, each a contour a step lower and nearer, its cut edge catching the light, the rise's
     sheets in the fields' colours; pines of stacked paper discs;
  C  faceted: the land in big irregular flat triangles, the fields as patches of facets; pines as faceted cones with jagged skirts.
After the papercraft references (docs/art/stages/BACKGROUNDS.md, 2026-10-21), three more on the land, with B's pines and hill:
  PA patchwork: the land a smooth sheet laid over with fields of card, each an organic cut shape in its own green or wheat, lifted
     over the one under it with its cut edge showing (no more stripes edge to edge);
  PB PA with lanes (pale sheets winding down the land) and hedgerows (rolled along some fields' edges);
  PC PB with small things dotted over the fields (bushes, stones, flowers) and each casting a soft shade on the sheet under it.
Each band is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `hill_far`, `hill_near`, `rise`, `field_0`..`field_4`, `hedge`, `card` (the paper's cut edge), `pine_0`..`pine_3`,
`pine_under`, `trunk`.

Run (Blender 5.2):
  blender --background --python art/blender/make_meadow_ground.py -- --dir godot/art/baked/<key> --variant A --ground <g.glb> --bush <p.glb>
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
LAYOUT = json.load(open(os.path.join(opts["dir"], "ground.json")))
X0, X1 = -200.0, 200.0

# The cross-sections (StageArt.ROLL_PROFILE, VALLEY_PROFILE): [drop below the crest, depth toward the camera].
ROLL = [[3.0, -6.0], [1.0, -3.6], [0.2, -1.6], [0.0, 0.0], [0.25, 1.3], [0.9, 2.5], [1.9, 3.4], [3.3, 4.0], [5.2, 4.35], [9.0, 4.5],
        [18.0, 4.55], [40.0, 4.6], [90.0, 4.6]]
VALLEY = [[3.0, -6.0], [1.0, -3.6], [0.2, -1.6], [0.0, 0.0], [0.25, 1.3], [0.9, 2.5], [1.9, 3.6], [3.3, 5.0], [5.2, 7.2], [9.0, 11.5],
          [18.0, 21.0], [32.0, 33.0], [50.0, 40.0], [90.0, 42.0]]
# The rise's hedgerows, as drops under its crest (StageArt._fields).
HEDGES = [2.2, 5.4, 9.6, 15.0, 22.0, 31.0, 43.0]


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


def land_top(bumps, floor_y, x):
    """StageArt._land_top: elliptical hilltops blended into the floor and each other by a smooth maximum."""
    y = floor_y
    for b in bumps:
        dx = (x - b[0]) / b[1]
        if abs(dx) < 1.0:
            top = b[3] + b[2] * math.sqrt(1.0 - dx * dx)
            k = 3.0
            m = min(max(0.5 + 0.5 * (top - y) / k, 0.0), 1.0)
            y = y + (top - y) * m + k * m * (1.0 - m)
    return y


def depth_at(d, profile):
    """StageArt.roll_z: how far in front of the band's z its face is, `d` under the crest."""
    for k in range(1, len(profile)):
        a, b = profile[k - 1], profile[k]
        if d <= b[0] and b[0] > a[0]:
            return a[1] + (b[1] - a[1]) * (d - a[0]) / (b[0] - a[0])
    return profile[-1][1]


PARTS = {}


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
    return PARTS[key]


def face(bm, pts):
    vs = [bm.verts.new(p) for p in pts]
    f = bm.faces.new(vs)
    return f


def field_of(x, d, seed):
    """Which field colour a point of the rise's face falls in: bands between the hedgerows, cut across into fields that fan out as they
    come down (as rows of fields do in perspective)."""
    band = sum(1 for h in HEDGES if d >= h)
    if band == 0 or d > HEDGES[-1]:
        return None
    u = x / (1.0 + d * 0.03)
    col = math.floor((u + 300.0) / (10.0 + 3.0 * (band % 3)))
    return "field_%d" % ((col * 7 + band * 3 + seed) % 5)


def ridge_widths(rng, lo=4.0, hi=8.5):
    xs = [X0]
    while xs[-1] < X1:
        xs.append(xs[-1] + rng.uniform(lo, hi))
    return xs


FARM = LAYOUT.get("farm", [])


def farm_calm(x):
    """0 under the farm (the land kept level, so the barn and farmhouse stand on it), rising to 1 four units out."""
    if len(FARM) < 2:
        return 1.0
    out = max(FARM[0] - x, x - FARM[1], 0.0)
    return min(out / 4.0, 1.0)


# ---- A: deep folds ------------------------------------------------------------------------------------------------------------------


def folded_band(name, fam, bumps, floor_y, z, profile, rng, fields=False):
    """A band of land whose face is pleated in irregular ridges and creases from a little under the crest down, like the ranges."""
    # (Narrower on the rise, under its fields: at the hill rows' widths a fold took a whole field.)
    xs = ridge_widths(rng, 2.6, 5.0) if fields else ridge_widths(rng)
    drops = sorted({r[0] for r in profile} | ({h for h in HEDGES} | {h + 0.6 for h in HEDGES} if fields else set()))
    rows = [d for d in drops if d <= 60.0]
    # Behind the crest, the rows with negative depth come first (back to front).
    back = [r for r in profile if r[1] < 0.0]
    order = [(r[0], r[1]) for r in back] + [(d, depth_at(d, profile)) for d in rows if d >= 0.0 and not any(r[0] == d and r[1] < 0 for r in back)]
    order = [o for o in order if not (o[1] < 0.0 and o[0] == 0.0)]
    amp = []
    for j, x in enumerate(xs):
        calm = farm_calm(x) if fields else 1.0
        amp.append((1.0 if j % 2 else -0.45) * (xs[min(j + 1, len(xs) - 1)] - xs[max(j - 1, 0)]) * 0.22 * rng.uniform(0.8, 1.2) * calm)

    def point(j, d, dz):
        x = xs[j]
        top = land_top(bumps, floor_y, x)
        # (Folds from a few units under the crest, so whatever stands on it keeps its feet; out again where the face turns into a wall.)
        f = min(max((d - 0.8) / 3.0, 0.0), 1.0) * (1.0 - min(max((d - 30.0) / 30.0, 0.0), 1.0)) if dz >= 0.0 else 0.0
        a = depth_at(max(d - 0.5, 0.0), profile)
        b = depth_at(d + 0.5, profile)
        n = Vector((0.0, b - a, 1.0)).normalized()
        push = n * amp[j] * f
        return P(x, top - d + push.y, z + dz + push.z)

    grid = [[point(j, d, dz) for (d, dz) in order] for j in range(len(xs))]
    for j in range(len(xs) - 1):
        for i in range(len(order) - 1):
            d_mid = (order[i][0] + order[i + 1][0]) * 0.5 if order[i][1] >= 0.0 else -1.0
            mat = fam
            if fields and order[i][1] >= 0.0:
                mat = field_of((xs[j] + xs[j + 1]) * 0.5, d_mid, 2) or fam
            bm = part(name, mat)
            a, b, c, e = grid[j][i], grid[j + 1][i], grid[j + 1][i + 1], grid[j][i + 1]
            face(bm, (a, e, c, b))
    if fields:
        # Hedgerows: a low folded ridge along each hedge line, across the folds.
        for h in HEDGES:
            bm = part(name, "hedge")
            for j in range(len(xs) - 1):
                p0, p1 = point(j, h, depth_at(h, profile)), point(j + 1, h, depth_at(h, profile))
                q0, q1 = point(j, h + 0.6, depth_at(h + 0.6, profile)), point(j + 1, h + 0.6, depth_at(h + 0.6, profile))
                up = Vector((0.0, -0.35, 0.55))
                m0, m1 = (p0 + q0) * 0.5 + up, (p1 + q1) * 0.5 + up
                face(bm, (p0, p1, m1, m0))
                face(bm, (m0, m1, q1, q0))


def pine_pleated(lit, under, trunk, base, sc, rng):
    tiers = 6
    for k in range(tiers):
        t = k / (tiers - 1)
        r = 0.8 * (1.0 - 0.74 * t) * sc
        z0 = base.z + (0.3 + 0.33 * k) * sc
        h = (0.74 - 0.07 * k if k < tiers - 1 else 0.85) * sc
        pleated_tier(lit, under, (base.x, base.y), z0, r, h, 8 if k < 3 else 6, rng.uniform(0, math.tau), rng, flare=0.48,
                     droop=(0.22 - 0.025 * k) * sc)
    res = bmesh.ops.create_cone(trunk, cap_ends=True, segments=6, radius1=0.1 * sc, radius2=0.06 * sc, depth=0.6 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 0.3 * sc))


# ---- B: paper layers ----------------------------------------------------------------------------------------------------------------


def layered_band(name, fam, bumps, floor_y, z, profile, rng, fields=False):
    """The land as stacked sheets of card: each sheet a contour a step under the one above and nearer the camera, its top flat and its cut
    edge standing as a lit lip, so the hill reads as a paper model."""
    steps = [0.0, 1.4, 3.2, 5.6, 8.6, 12.4, 17.0, 23.0, 31.0, 43.0, 60.0]
    xs = []
    x = X0
    while x <= X1:
        xs.append(x)
        x += 3.0
    edge = 0.0
    phase = rng.uniform(0, 6)
    for k in range(len(steps) - 1):
        d0, d1 = steps[k], steps[k + 1]
        z_back = z - 6.0 if k == 0 else z + depth_at(d0, profile) + edge
        z_front = z + depth_at(d1, profile)
        mats = []
        for j in range(len(xs) - 1):
            xm = (xs[j] + xs[j + 1]) * 0.5
            mat = fam
            if fields and k >= 1:
                mat = field_of(xm, (d0 + d1) * 0.5 + 0.01, 1) or fam
            mats.append(mat)

        def wav(x, kk=k):
            return 0.5 * math.sin(x * 0.11 + kk * 1.7 + phase) + 0.25 * math.sin(x * 0.29 + kk)

        for j in range(len(xs) - 1):
            xa, xb = xs[j], xs[j + 1]
            ya, yb = land_top(bumps, floor_y, xa) - d0, land_top(bumps, floor_y, xb) - d0
            za, zb = z_front + wav(xa), z_front + wav(xb)
            zba = z_back + (wav(xa, k - 1) if k else 0.0)
            zbb = z_back + (wav(xb, k - 1) if k else 0.0)
            # The sheet's top.
            face(part(name, mats[j]), (P(xa, ya, zba), P(xa, ya, za), P(xb, yb, zb), P(xb, yb, zbb)))
            # Its cut edge, down to the next sheet.
            ya1, yb1 = land_top(bumps, floor_y, xa) - d1, land_top(bumps, floor_y, xb) - d1
            face(part(name, "card_" + name), (P(xa, ya, za), P(xa, ya1, za), P(xb, yb1, zb), P(xb, yb, zb)))
        if k == 0:
            # Behind the crest the top sheet falls away as one back edge.
            for j in range(len(xs) - 1):
                xa, xb = xs[j], xs[j + 1]
                ya, yb = land_top(bumps, floor_y, xa), land_top(bumps, floor_y, xb)
                face(part(name, fam), (P(xa, ya, z - 6.0), P(xb, yb, z - 6.0), P(xb, yb - 6.0, z - 6.0), P(xa, ya - 6.0, z - 6.0)))
    # The last sheet's face carried on down out of view.
    for j in range(len(xs) - 1):
        xa, xb = xs[j], xs[j + 1]
        zf = z + depth_at(steps[-1], profile)
        ya, yb = land_top(bumps, floor_y, xa) - steps[-1], land_top(bumps, floor_y, xb) - steps[-1]
        face(part(name, fam), (P(xa, ya, zf), P(xa, ya - 40.0, zf), P(xb, yb - 40.0, zf), P(xb, yb, zf)))


def pine_discs(lit, under, trunk, base, sc, rng):
    """A pine of stacked paper discs: shallow cones, each a little smaller and turned, the cut edges catching the light."""
    layers = 7
    for k in range(layers):
        t = k / (layers - 1)
        # (As narrow as the code-built pine: wider, the clusters crowded the platforms and hid the cottage.)
        r = 0.6 * (1.0 - 0.75 * t) * sc
        z0 = base.z + (0.35 + 0.3 * k) * sc
        n = 12
        turn = rng.uniform(0, math.tau)
        rim = [Vector((base.x + math.cos(turn + math.tau * i / n) * r * (1.0 if i % 2 else 0.9),
                       base.y + math.sin(turn + math.tau * i / n) * r * (1.0 if i % 2 else 0.9), z0)) for i in range(n)]
        tip = Vector((base.x, base.y, z0 + 0.32 * sc))
        lip = [v - Vector((0, 0, 0.07 * sc)) for v in rim]
        for i in range(n):
            j = (i + 1) % n
            face(lit, (rim[i], rim[j], tip))
            face(under, (lip[i], lip[j], rim[j], rim[i]))
        cen = Vector((base.x, base.y, z0 - 0.07 * sc))
        for i in range(n):
            face(under, (lip[(i + 1) % n], lip[i], cen))
    res = bmesh.ops.create_cone(trunk, cap_ends=True, segments=6, radius1=0.1 * sc, radius2=0.07 * sc, depth=0.6 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 0.3 * sc))


def hill_slope(bottom, d):
    """StageArt._meadow_hill's fall from the wall's foot, `d` out from it."""
    return bottom + 0.05 - 0.55 * d - 0.006 * d * d


def hill_ring(l, r, front, back, d, n_side=8, n_arc=8, n_front=12):
    """The outline `d` out from the wall (left side, front-left corner, front, front-right corner, right side), back to front and round;
    the same count of points at every `d`, so two rings make a strip."""
    cx, hw = (l + r) * 0.5, (r - l) * 0.5
    pts = []
    for i in range(n_side):
        pts.append((cx - hw - d, back + (front - back) * i / n_side))
    for i in range(n_arc):
        a = math.pi + (math.pi * 0.5) * i / n_arc
        pts.append((cx - hw + math.cos(a) * d, front - math.sin(a) * d))
    for i in range(n_front + 1):
        pts.append((cx - hw + 2.0 * hw * i / n_front, front + d))
    for i in range(1, n_arc + 1):
        a = math.pi * 1.5 + (math.pi * 0.5) * i / n_arc
        pts.append((cx + hw + math.cos(a) * d, front - math.sin(a) * d))
    for i in range(1, n_side + 1):
        pts.append((cx + hw + d, front + (back - front) * i / n_side))
    return pts


def hill_layers(rng):
    """The stage's hill as stacked sheets of card, contours of the code-built hill's fall: each sheet flat at its contour's height, its cut
    edge standing at the next contour, thin near the wall and thicker further out."""
    if not LAYOUT.get("hill"):
        return
    l, r, bottom, front = LAYOUT["hill"]
    back = front - 14.0
    levels = [0.0]
    k = 0
    while levels[-1] < 22.0:
        drop = 0.5 * (1.0 + k * 0.1)
        y = hill_slope(bottom, levels[-1]) - drop
        # The distance where the slope reaches `y`.
        a, b, c = 0.006, 0.55, y - bottom - 0.05
        levels.append((-b + math.sqrt(b * b - 4 * a * c)) / (2 * a))
        k += 1
    phase = rng.uniform(0, 6)

    def ring(d, kk):
        pts = hill_ring(l, r, front, back, d)
        out = []
        for i, (x, z) in enumerate(pts):
            w = 0.0 if d == 0.0 else 0.18 * math.sin(i * 0.9 + kk * 1.3 + phase) + 0.1 * math.sin(i * 2.3 + kk)
            out.append((x, z, w))
        return out

    rings = [ring(d, kk) for kk, d in enumerate(levels)]
    cx = (l + r) * 0.5

    def at(pt, d):
        """A ring point pushed out from the wall by its wobble."""
        x, z, w = pt
        if d == 0.0:
            return x, z
        hx = x - cx
        out_x = (1.0 if hx > 0 else -1.0) if abs(hx) > (r - l) * 0.5 + 0.01 else 0.0
        out_z = 1.0 if z > front + 0.01 else 0.0
        return x + out_x * w, z + out_z * w

    for kk in range(len(levels) - 1):
        y = hill_slope(bottom, levels[kk])
        y_next = hill_slope(bottom, levels[kk + 1])
        inner, outer = rings[kk], rings[kk + 1]
        for i in range(len(inner) - 1):
            ax, az = at(inner[i], levels[kk])
            bx, bz = at(inner[i + 1], levels[kk])
            cx2, cz = at(outer[i + 1], levels[kk + 1])
            dx, dz = at(outer[i], levels[kk + 1])
            face(part("hill", "hill"), (P(ax, y, az), P(bx, y, bz), P(cx2, y, cz), P(dx, y, dz)))
            bottom_y = y_next if kk < len(levels) - 2 else y_next - 30.0
            face(part("hill", "card_hill"), (P(dx, y, dz), P(cx2, y, cz), P(cx2, bottom_y, cz), P(dx, bottom_y, dz)))


def hill_field():
    """The stage's hill as the code builds it (its slope in plain faceted planes), for the looks without paper layers."""
    if not LAYOUT.get("hill"):
        return
    l, r, bottom, front = LAYOUT["hill"]
    cx, w = (l + r) * 0.5, r - l
    xs = [cx - w * 1.6 - 20.0 + 2.6 * i for i in range(int((w * 3.2 + 40.0) / 2.6) + 1)]
    zs = [front - 14.0 + 2.6 * j for j in range(int(36.0 / 2.6) + 1)]

    def y(x, z):
        dx = max(0.0, abs(x - cx) - w * 0.5)
        dz = max(0.0, z - front)
        return hill_slope(bottom, math.sqrt(dx * dx + dz * dz))

    for j in range(len(zs) - 1):
        for i in range(len(xs) - 1):
            a, b, c, d = (xs[i], zs[j]), (xs[i + 1], zs[j]), (xs[i + 1], zs[j + 1]), (xs[i], zs[j + 1])
            face(part("hill", "hill"), [P(x, y(x, z), z) for (x, z) in (a, b, c, d)])


# ---- C: faceted ---------------------------------------------------------------------------------------------------------------------


def faceted_band(name, fam, bumps, floor_y, z, profile, rng, fields=False):
    """The land in big irregular flat triangles, each vertex nudged by its own amount; the rise's fields as patches of facets."""
    xs = []
    x = X0
    while x <= X1:
        xs.append(x)
        x += rng.uniform(5.0, 9.0)
    rows = [r for r in profile if r[0] <= 60.0]
    grid = []
    for j, x in enumerate(xs):
        col = []
        for i, (d, dz) in enumerate(rows):
            jit = (0.0 if d <= 0.25 else min(d * 0.25, 1.6)) * (farm_calm(x) if fields else 1.0)
            xx = x + (rng.uniform(-2.0, 2.0) if 0 < j < len(xs) - 1 else 0.0)
            top = land_top(bumps, floor_y, xx)
            col.append(P(xx, top - d + rng.uniform(-jit, jit), z + dz + rng.uniform(-jit, jit) * 0.8))
        grid.append(col)
    for j in range(len(xs) - 1):
        for i in range(len(rows) - 1):
            a, b, c, e = grid[j][i], grid[j + 1][i], grid[j + 1][i + 1], grid[j][i + 1]
            tris = ((a, e, c), (a, c, b)) if rng.random() < 0.5 else ((a, e, b), (e, c, b))
            for tri in tris:
                mat = fam
                if fields and rows[i][1] >= 0.0:
                    mat = field_of((xs[j] + xs[j + 1]) * 0.5 + rng.uniform(-3, 3), (rows[i][0] + rows[i + 1][0]) * 0.5, 3) or fam
                face(part(name, mat), tri)


def pine_faceted(lit, under, trunk, base, sc, rng):
    """A faceted pine: three stacked six-sided cones, each skirt's rim jagged and its points hanging."""
    for k, (z0, r, h) in enumerate(((0.35, 0.8, 1.0), (0.95, 0.6, 0.9), (1.5, 0.4, 1.0))):
        n = 7
        turn = rng.uniform(0, math.tau)
        rim = []
        for i in range(n):
            a = turn + math.tau * i / n
            rr = r * sc * rng.uniform(0.85, 1.1)
            rim.append(Vector((base.x + math.cos(a) * rr, base.y + math.sin(a) * rr, base.z + z0 * sc - rng.uniform(0.0, 0.18) * sc)))
        tip = Vector((base.x, base.y, base.z + (z0 + h) * sc))
        for i in range(n):
            face(lit, (rim[i], rim[(i + 1) % n], tip))
        cen = Vector((base.x, base.y, base.z + (z0 + 0.12) * sc))
        for i in range(n):
            face(under, (rim[(i + 1) % n], rim[i], cen))
    res = bmesh.ops.create_cone(trunk, cap_ends=True, segments=5, radius1=0.1 * sc, radius2=0.07 * sc, depth=0.6 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 0.3 * sc))


# ---- PA, PB, PC: patchwork ---------------------------------------------------------------------------------------------------------


def surface(bumps, floor_y, z, profile):
    """The band's land as a function of (x, the drop under its crest), and its facing (toward the camera and the sky)."""
    def at(x, d):
        return Vector((x, land_top(bumps, floor_y, x) - d, z + depth_at(max(d, 0.0), profile)))

    def normal(x, d):
        t1 = at(x + 0.5, d) - at(x - 0.5, d)
        t2 = at(x, d + 0.4) - at(x, max(d - 0.4, 0.0))
        n = t1.cross(t2)
        if n.z + n.y < 0.0:
            n = -n
        return n.normalized()
    return at, normal


def blob(cx, cd, rx, rd, rng, n=28):
    """A field's cut outline in (x, drop): a rounded oblong laid along the land's contour, its edges a little hand-cut. (Round blobs of
    every size overlapped into camouflage, not fields.)"""
    p1 = rng.uniform(0, math.tau)
    out = []
    for i in range(n):
        a = math.tau * i / n
        c, s_ = math.cos(a), math.sin(a)
        w = 1.0 + 0.04 * math.sin(3 * a + p1)
        out.append((cx + rx * w * math.copysign(abs(c) ** 0.45, c), max(0.15, cd + rd * w * math.copysign(abs(s_) ** 0.6, s_))))
    return out


def tube_seg(bm, a, b, rad, segs=6):
    """A rolled tube between two Blender points."""
    d = b - a
    if d.length < 1e-4:
        return
    res = bmesh.ops.create_cone(bm, cap_ends=True, segments=segs, radius1=rad, radius2=rad, depth=d.length)
    q = Vector((0.0, 0.0, 1.0)).rotation_difference(d.normalized())
    for v in res["verts"]:
        v.co = q @ v.co + (a + b) * 0.5


def sheet(bm_top, bm_edge, ring, centre, lifted):
    """A cut sheet laid on the land: a fan from its centre over two rings, and its cut edge down under the land."""
    mid = [centre.lerp(q, 0.5) for q in ring]
    n = len(ring)
    c = lifted(centre)
    m = [lifted(q) for q in mid]
    o = [lifted(q) for q in ring]
    for i in range(n):
        j = (i + 1) % n
        face(bm_top, (P(*c), P(*m[i]), P(*m[j])))
        face(bm_top, (P(*m[i]), P(*o[i]), P(*o[j]), P(*m[j])))
    for i in range(n):
        j = (i + 1) % n
        face(bm_edge, (P(*o[i]), P(*o[j]), P(*(ring[j] - Vector((0.0, 0.05, 0.0)))), P(*(ring[i] - Vector((0.0, 0.05, 0.0))))))


def patchwork_band(name, fam, bumps, floor_y, z, profile, rng, fields=False):
    """The band's land as a smooth sheet, laid over with fields of card in organic cut shapes, each lifted over the one under it."""
    level = {"PA": 1, "PB": 2, "PC": 3}[VARIANT]
    at, normal = surface(bumps, floor_y, z, profile)
    # The smooth land under the fields (back of the crest first, as the other looks).
    # (Only as wide as any camera sees: the land past it cost triangles and nothing else.)
    lo, hi = -150.0, 150.0
    xs = [lo + 2.5 * i for i in range(int((hi - lo) / 2.5) + 1)]
    back = [(r[0], r[1]) for r in profile if r[1] < 0.0]
    drops = [0.0, 0.3, 0.7, 1.2, 1.8, 2.6, 3.6, 4.8, 6.4, 8.4, 11.0, 14.0, 18.0, 23.0, 29.0, 36.0, 44.0, 52.0, 60.0]
    order = back + [(d, depth_at(d, profile)) for d in drops]
    grid = [[P(x, land_top(bumps, floor_y, x) - d, z + dz) for (d, dz) in order] for x in xs]
    base = part(name, fam)
    for j in range(len(xs) - 1):
        for i in range(len(order) - 1):
            face(base, (grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]))
    # The fields: rows of organic shapes down the land, larger further down (nearer the camera), stacked in a random order.
    rows = [2.2, 5.4, 9.6, 15.0, 22.0, 31.0, 42.0] if fields else [0.9, 2.6, 5.0, 8.4, 12.6]
    gap = {r: (rows[k + 1] - rows[k]) if k + 1 < len(rows) else rows[k] * 0.3 for k, r in enumerate(rows)}
    patches = []
    for cd in rows:
        x = lo + rng.uniform(0, 6)
        while x < hi:
            w = rng.uniform(8.0, 12.0) * (1.0 + 0.03 * cd)
            if not (fields and farm_calm(x) < 0.6):
                # (A gap of the land's own green between neighbours, as the edges between fields.)
                patches.append((x + w * 0.5, cd, w * 0.44, gap[cd] * 0.4, rng.randrange(2)))
            x += w
    patches.sort(key=lambda q: q[4])
    for (cx, cd, rx, rd, layer) in patches:
        lift = 0.05 + 0.05 * layer
        outline = blob(cx, cd, rx, rd, rng)
        ring = [at(x, d) for (x, d) in outline]
        centre = at(cx, cd)
        mat = "field_%d" % rng.choice((0, 2, 4, 0, 2, 4, 1, 0, 2, 3))

        def lifted(q, lift=lift):
            x, d = q.x, land_top(bumps, floor_y, q.x) - q.y
            return q + normal(x, d) * lift
        sheet(part(name, mat), part(name, "card_" + name), ring, centre, lifted)
        if level >= 2 and rng.random() < 0.45:
            # A hedgerow rolled along the field's lower edge.
            bm = part(name, "hedge")
            rad = 0.18 + 0.006 * cd
            pts = [lifted(q) + normal(q.x, land_top(bumps, floor_y, q.x) - q.y) * rad for q in ring]
            n = len(pts)
            for i in range(n // 2 - 1, n):
                tube_seg(bm, P(*pts[i]), P(*pts[(i + 1) % n]), rad)
        if level >= 3:
            for _ in range(rng.randrange(1, 4)):
                a = rng.uniform(0, math.tau)
                u = rng.uniform(0.0, 0.7)
                x, d = cx + rx * u * math.cos(a), max(0.3, cd + rd * u * math.sin(a))
                q = lifted(at(x, d))
                nn = normal(x, d)
                kind = rng.random()
                sc = 0.45 + 0.02 * d
                # The soft shade it casts on the sheet, offset away from the light.
                sh = [q + nn * 0.02 + Vector((0.35 * sc + sc * 0.9 * math.cos(t), 0.0, sc * 0.45 * math.sin(t))) for t in [math.tau * i / 10 for i in range(10)]]
                face(part(name, "shade"), [P(*v) for v in sh])
                bm = part(name, "hedge" if kind < 0.6 else ("stone" if kind < 0.8 else rng.choice(("flower_a", "flower_b"))))
                if kind < 0.6:
                    res = bmesh.ops.create_cone(bm, cap_ends=True, segments=7, radius1=sc, radius2=0.0, depth=sc * 1.6)
                    off = P(*(q + nn * sc * 0.75))
                elif kind < 0.8:
                    res = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=sc * 0.45)
                    for v in res["verts"]:
                        v.co.z *= 0.55
                    off = P(*(q + nn * sc * 0.2))
                else:
                    res = bmesh.ops.create_cone(bm, cap_ends=True, segments=6, radius1=sc * 0.3, radius2=sc * 0.12, depth=sc * 0.25)
                    off = P(*(q + nn * sc * 0.2))
                for v in res["verts"]:
                    v.co += off
    if level >= 2 and fields:
        # Lanes: pale sheets winding down the rise either side of the stage, widening as they come nearer.
        for x0, ph in ((-36.0, 0.0), (34.0, 1.7)):
            bm, edge = part(name, "lane"), part(name, "card_lane")
            ds = [0.4 + 1.0 * k for k in range(30)]
            left, right = [], []
            for d in ds:
                x = x0 + 3.0 * math.sin(d * 0.2 + ph)
                w = 0.7 + d * 0.035
                nn = normal(x, d)
                left.append(at(x - w, d) + nn * 0.3)
                right.append(at(x + w, d) + nn * 0.3)
            for k in range(len(ds) - 1):
                face(bm, (P(*left[k]), P(*left[k + 1]), P(*right[k + 1]), P(*right[k])))
                for side in (left, right):
                    face(edge, (P(*side[k]), P(*side[k + 1]), P(*(side[k + 1] - Vector((0, 0.35, 0)))), P(*(side[k] - Vector((0, 0.35, 0))))))


# ---- Build --------------------------------------------------------------------------------------------------------------------------


def build_ground():
    rng = random.Random(61)
    band = {"A": folded_band, "B": layered_band, "C": faceted_band, "PA": patchwork_band, "PB": patchwork_band, "PC": patchwork_band}[VARIANT]
    band("far", "hill_far", LAYOUT["hills_far"], -17.0, -100.0, ROLL, rng)
    band("near", "hill_near", LAYOUT["hills_near"], -17.0, -72.0, ROLL, rng)
    band("rise", "rise", LAYOUT["fore"], -11.0, -46.0, VALLEY, rng, fields=True)
    if VARIANT in ("B", "PA", "PB", "PC"):
        hill_layers(rng)
    else:
        hill_field()


PINE_SPOTS = [(0.0, 0.0, 1.0), (-0.55, -0.12, 0.72), (0.5, -0.2, 0.8), (0.22, 0.12, 0.6), (-0.3, -0.3, 0.66)]


def build_pines():
    """One cluster at size 1 (the game scales it by each spot's size): the tallest pine 2.5 tall, three or four smaller round it, as
    `StageArt._pine_cluster` sets them, each in its own green."""
    rng = random.Random(83)
    pine = {"A": pine_pleated, "C": pine_faceted}.get(VARIANT, pine_discs)
    # (Each look's pine at the code-built pine's height, 2.5 at size 1.)
    sc = {"A": 0.9}.get(VARIANT, 1.0)
    for k, (ox, oz, s) in enumerate(PINE_SPOTS):
        lit = part("pine", "pine_%d" % (k % 4))
        under = part("pine", "pine_under")
        trunk = part("pine", "trunk")
        base = P(ox, -0.12 * s, oz)
        pine(lit, under, trunk, base, s * sc, rng)


def make_objects():
    for (name, mat), bm in PARTS.items():
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
        bm.normal_update()
        for f in bm.faces:
            c = f.calc_center_median()
            if name == "pine":
                # (Away from its own pine's trunk, sideways: undersides face out and down, skirts out and up.)
                sx, sz = min(PINE_SPOTS, key=lambda p: (c.x - p[0]) ** 2 + (c.y + p[1]) ** 2)[:2]
                away = Vector((c.x - sx, c.y + sz, 0.0))
            elif name == "hill" and LAYOUT.get("hill"):
                # (The hill's sheets face up, their cut edges out from the wall.)
                hl, hr, _, hf = LAYOUT["hill"]
                hx = c.x - (hl + hr) * 0.5
                hx = (hx - math.copysign((hr - hl) * 0.5, hx)) if abs(hx) > (hr - hl) * 0.5 else 0.0
                hz = max(-c.y - hf, 0.0)
                away = Vector((hx, -hz, 0.0)) + Vector((0.0, 0.0, 0.3))
            else:
                # (The land faces the camera and the sky.)
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
    # (Softened: the land keeps most of its own light, the creases a soft shade.)
    for o in parts:
        col = o.data.color_attributes["Col"].data
        for c in col:
            ao = 0.45 + 0.55 * c.color[0]
            c.color = (ao, ao, ao, 1.0)


def export(path, what):
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    path = os.path.abspath(path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("meadow %s %s: %d triangles -> %s" % (what, VARIANT, tris, path))


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_ground()
    make_objects()
    # (Wider on the patchwork's last look, its soft shade: the creases under the hedgerows and the dotted things.)
    bake_occlusion(5.0 if VARIANT == "PC" else 3.0)
    export(opts["ground"], "ground")
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_pines()
    make_objects()
    bpy.ops.mesh.primitive_plane_add(size=6.0)
    ground = bpy.context.active_object
    bake_occlusion(0.6)
    bpy.data.objects.remove(ground, do_unlink=True)
    export(opts["bush"], "pines")


main()
