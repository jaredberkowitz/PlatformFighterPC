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
After the grove's terraces (the owner, 2026-10-21: refine each stage on its own, from its own reference, the meadow's the papercraft camp):
  MH shelves: the rise as a few broad shelves of bright lime grass, each ending in a crisp dark ledge under a scalloped grass lip, the
     ledges wandering out into promontories; two lanes cross the shelves and step down each ledge on stairs cut into it; belts of big
     pines along the near hills, all at one large size, as the reference's forest; the far hills plain card.
After the owner's note on PB as shipped (2026-10-22: "reads a bit muddy, unprofessional, not consistently shaped"), three tidier looks:
  QA tidy quilt: every field the same rounded-rectangle shape and size along its row (shrinking with distance), even gaps of the land's
     own green between them, three clean greens and now and then wheat, in a steady pattern; continuous hedgerows along the rows instead
     of rolls round some fields; cut edges a deeper green instead of cream; a light occlusion bake;
  QB QA with fewer, larger fields (half as many rows, twice as wide);
  QC the land a clean lawn, with tidy fields only round the farm and along the lanes.
After the owner's note on QA (2026-10-22: "the plots feel sloppy, the road does not feel connected, the grove reads way better"), three
looks of farmland that holds together:
  FA hedged fields: the rise cut edge to edge into fields (no land between them), rows along the contours and boundaries running down
     the slope, every boundary a bushy hedgerow; pasture in three greens, wheat, ploughed earth and young crops, the cropped fields
     drawn in rows; one network of lanes, each going somewhere: from over the crest past the farm (a spur into its yard) and from the
     right-hand cottage, down to a lane across the slope, and on down toward the stage; hedges along the lanes, open where they cross;
     the near hills' fields tiled the same way in pasture greens;
  FB FA laid as shallow terraces: each row of fields a flatter shelf ending in a short steep bank of earth under a grass lip, as the
     grove's terraces (lit top, dark side); the lanes ramp down the banks;
  FC FB with pines along the hedgerows, in ones to threes where boundaries start, and a copse behind the farm.
Each band is its own object so the game can lay the air of its depth over it, and bakes its occlusion into linear vertex colours.
Families `hill_far`, `hill_near`, `rise`, `field_0`..`field_4`, `hedge`, `card` (the paper's cut edge), `pine_0`..`pine_3`,
`pine_under`, `trunk`.

Shipped (the owner's pick, 2026-10-22): FC with `--pine D` as godot/models/stages/meadow_ground.glb and meadow_pines.glb, then lit
(light_stage.py on the front, the play area and the ground).

Shipped since fidelity review 6 (the owner's pick B, 2026-10-22): `--variant FC --pine D --hill banks`, then lit.
Shipped since the new art toward 5 (the owner's pick B, 2026-10-25): `--variant FC --pine D --hill fields_lane`, then lit.

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
HILL = opts.get("hill", "")
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


def pine_detailed(lit, under, trunk, base, sc, rng):
    """A pine with more detail at background size (the owner, 2026-10-22: more detail on the background trees), in the shipped pines'
    clean stacked-skirt shape: eight skirts, each with a scalloped hem (every other point hanging lower, a little uneven) and a slight
    turn, so each tier's edge catches light in steps; the skirts a little off the axis up the tree; a visible trunk with three roots."""
    layers = 8
    lean = Vector((rng.uniform(-0.03, 0.03), rng.uniform(-0.03, 0.03), 0.0)) * sc
    for k in range(layers):
        t = k / (layers - 1)
        r = 0.66 * (1.0 - 0.78 * t) * sc
        c = base + lean * k
        z0 = base.z + (0.42 + 0.27 * k) * sc
        n = 16 if k < 4 else 12
        turn = rng.uniform(0, math.tau)
        rim = []
        for i in range(n):
            a = turn + math.tau * i / n
            drop = (0.13 if i % 2 == 0 else 0.0) * (1.0 - 0.5 * t) * sc * rng.uniform(0.8, 1.2)
            rr = r * (1.0 if i % 2 == 0 else 0.86)
            rim.append(Vector((c.x + math.cos(a) * rr, c.y + math.sin(a) * rr, z0 - drop)))
        tip = Vector((c.x, c.y, z0 + 0.36 * sc))
        lip = [v - Vector((0, 0, 0.07 * sc)) for v in rim]
        cen = Vector((c.x, c.y, z0 - 0.04 * sc))
        for i in range(n):
            j = (i + 1) % n
            face(lit, (rim[i], rim[j], tip))
            face(under, (lip[i], lip[j], rim[j], rim[i]))
            face(under, (lip[j], lip[i], cen))
    res = bmesh.ops.create_cone(trunk, cap_ends=True, segments=7, radius1=0.13 * sc, radius2=0.07 * sc, depth=0.7 * sc)
    for v in res["verts"]:
        v.co += base + Vector((0, 0, 0.3 * sc))
    for k in range(3):
        a = rng.uniform(0, 1.0) + k * math.tau / 3
        d = Vector((math.cos(a), math.sin(a), 0.0))
        side = d.cross(Vector((0, 0, 1)))
        p0 = base + Vector((0, 0, 0.16 * sc))
        face(trunk, (p0 + side * 0.05 * sc, p0 - side * 0.05 * sc, base + d * 0.26 * sc))


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
    # (Fidelity review 6: at a sheet every half unit the hill read as a regular stepped cake, the one machine-made shape beside the
    # terraced farmland. `--hill big` fewer, bigger sheets of uneven height and a broader wobble; `--hill banks` those as terraces,
    # each shelf falling to the next on a grass bank, as the farmland's do.)
    big = HILL in ("big", "banks", "fields", "fields_lane")
    hrng = random.Random(29)
    while levels[-1] < 22.0:
        drop = 0.5 * (1.0 + k * 0.1)
        if big:
            drop = 1.3 * (1.0 + k * 0.12) * hrng.uniform(0.7, 1.35)
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
            if big and d > 0.0:
                w = 0.7 * math.sin(i * 0.45 + kk * 1.7 + phase) + 0.3 * math.sin(i * 1.3 + kk * 0.6)
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
        if HILL in ("fields", "fields_lane"):
            terrace_fields(kk, inner, outer, levels, y, y_next, at, len(levels) - 2)
            continue
        if HILL == "banks":
            # A shelf out to a little past halfway, then a grass bank down to the next shelf.
            dm = levels[kk] + (levels[kk + 1] - levels[kk]) * 0.55
            mid = ring(dm, kk + 0.5)
            for i in range(len(inner) - 1):
                ax, az = at(inner[i], levels[kk])
                bx, bz = at(inner[i + 1], levels[kk])
                mx2, mz2 = at(mid[i + 1], dm)
                mx1, mz1 = at(mid[i], dm)
                face(part("hill", "hill"), (P(ax, y, az), P(bx, y, bz), P(mx2, y, mz2), P(mx1, y, mz1)))
                cx2, cz = at(outer[i + 1], levels[kk + 1])
                dx, dz = at(outer[i], levels[kk + 1])
                face(part("hill", "bank"), (P(mx1, y, mz1), P(mx2, y, mz2), P(cx2, y_next, cz), P(dx, y_next, dz)))
                if kk == len(levels) - 2:
                    face(part("hill", "card_hill"), (P(dx, y_next, dz), P(cx2, y_next, cz), P(cx2, y_next - 30.0, cz), P(dx, y_next - 30.0, dz)))
            continue
        for i in range(len(inner) - 1):
            ax, az = at(inner[i], levels[kk])
            bx, bz = at(inner[i + 1], levels[kk])
            cx2, cz = at(outer[i + 1], levels[kk + 1])
            dx, dz = at(outer[i], levels[kk + 1])
            face(part("hill", "hill"), (P(ax, y, az), P(bx, y, bz), P(cx2, y, cz), P(dx, y, dz)))
            bottom_y = y_next if kk < len(levels) - 2 else y_next - 30.0
            face(part("hill", "card_hill"), (P(dx, y, dz), P(cx2, y, cz), P(cx2, bottom_y, cz), P(dx, bottom_y, dz)))


def terrace_fields(kk, inner, outer, levels, y, y_next, at, last):
    """One terrace of the hill as farm fields (new art toward 5, M1, the owner's approval 2026-10-25; `--hill fields`): the shelf laid in
    fields of the valley's own kinds, rows on the crops, held by a low fieldstone wall at its outer edge down to the next shelf; with
    `--hill fields_lane` a lane along each shelf, switching back from side to side and stepping down a ramp to the next."""
    frng = random.Random(41 + kk)
    kinds = [k for k in FIELD_KINDS for _ in range(k[2])]
    n = len(inner)
    i = 0
    while i < n - 1:
        run = frng.randrange(3, 7)
        fam, row, _ = frng.choice(kinds)
        for j in range(i, min(i + run, n - 1)):
            ax, az = at(inner[j], levels[kk])
            bx, bz = at(inner[j + 1], levels[kk])
            cx2, cz = at(outer[j + 1], levels[kk + 1])
            dx, dz = at(outer[j], levels[kk + 1])
            face(part("hill", fam), (P(ax, y, az), P(bx, y, bz), P(cx2, y, cz), P(dx, y, dz)))
            if row:
                for f in (0.3, 0.55, 0.8):
                    def lp(p, q, t):
                        return p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t
                    a0, a1 = lp((ax, az), (dx, dz), f - 0.05), lp((ax, az), (dx, dz), f + 0.05)
                    b0, b1 = lp((bx, bz), (cx2, cz), f - 0.05), lp((bx, bz), (cx2, cz), f + 0.05)
                    face(part("hill", row), (P(a0[0], y + 0.02, a0[1]), P(b0[0], y + 0.02, b0[1]), P(b1[0], y + 0.02, b1[1]),
                                             P(a1[0], y + 0.02, a1[1])))
        i += run
    # The fieldstone wall at the shelf's outer edge, in two courses of blocks a little in and out, two tones.
    bottom = y_next if kk < last else y_next - 30.0
    wall_h = y - y_next
    for j in range(n - 1):
        cx2, cz = at(outer[j + 1], levels[kk + 1])
        dx, dz = at(outer[j], levels[kk + 1])
        if kk == last:
            face(part("hill", "card_hill"), (P(dx, y_next, dz), P(cx2, y_next, cz), P(cx2, bottom, cz), P(dx, bottom, dz)))
        ln = math.hypot(cx2 - dx, cz - dz)
        if ln < 0.05:
            continue
        nx, nz = (cz - dz) / ln, -(cx2 - dx) / ln
        blocks = max(1, int(ln / 1.3))
        for c in range(2):
            ya, yb = y - wall_h * c / 2.0, y - wall_h * (c + 1) / 2.0
            # (The second course offset half a block, its first and last blocks halves, so the courses close; flush, as a block set in
            # or out drew a cut edge round itself and the wall read as a grid.)
            cuts = [q / blocks for q in range(blocks + 1)] if c == 0 else [0.0] + [(q + 0.5) / blocks for q in range(blocks)] + [1.0]
            for q in range(len(cuts) - 1):
                t0, t1 = cuts[q], cuts[q + 1]
                pa = (dx + (cx2 - dx) * t0, dz + (cz - dz) * t0)
                pb = (dx + (cx2 - dx) * t1, dz + (cz - dz) * t1)
                fam = "terrace_wall" if (q + c + frng.randrange(2)) % 2 == 0 else "terrace_wall_b"
                face(part("hill", fam), (P(pa[0], ya, pa[1]), P(pb[0], ya, pb[1]), P(pb[0], yb, pb[1]), P(pa[0], yb, pa[1])))
    if HILL == "fields_lane" and kk < last:
        # The lane along the shelf near its inner edge, from the front's middle out to one side, then a ramp down to the next shelf.
        mid = n // 2
        end = min(n - 2, mid + 12) if kk % 2 == 0 else max(1, mid - 12)
        step = 1 if end > mid else -1
        for j in range(mid, end, step):
            j2 = j + step
            a = at(inner[j], levels[kk]); b = at(inner[j2], levels[kk])
            c2 = at(outer[j2], levels[kk + 1]); d = at(outer[j], levels[kk + 1])
            def lp2(p, q, t):
                return p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t
            a0, a1 = lp2(a, d, 0.12), lp2(a, d, 0.36)
            b0, b1 = lp2(b, c2, 0.12), lp2(b, c2, 0.36)
            face(part("hill", "lane"), (P(a0[0], y + 0.04, a0[1]), P(b0[0], y + 0.04, b0[1]), P(b1[0], y + 0.04, b1[1]), P(a1[0], y + 0.04, a1[1])))
        # The ramp: from the lane's end over the wall, down onto the next shelf.
        a = at(inner[end], levels[kk]); d = at(outer[end], levels[kk + 1])
        e = at(outer[end - step], levels[kk + 1])
        top0, top1 = (a[0] + (d[0] - a[0]) * 0.12, a[1] + (d[1] - a[1]) * 0.12), (a[0] + (d[0] - a[0]) * 0.36, a[1] + (d[1] - a[1]) * 0.36)
        out = (d[0] + (d[0] - a[0]) * 0.25, d[1] + (d[1] - a[1]) * 0.25)
        face(part("hill", "lane"), (P(top0[0], y + 0.04, top0[1]), P(top1[0], y + 0.04, top1[1]), P(out[0], y_next + 0.04, out[1]),
                                    P(e[0], y_next + 0.04, e[1])))


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


def blob(cx, cd, rx, rd, rng, n=20):
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
            # (Every other point: at this distance a hedgerow of half the segments reads the same, for half the triangles.)
            for i in range(n // 2 - 1, n, 2):
                tube_seg(bm, P(*pts[i]), P(*pts[(i + 2) % n]), rad, segs=5)
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
        # (The left lane beside the farm, not through it: at -36 it ran under the barn, the believability pass, 2026-10-22.)
        for x0, ph in ((-47.0, 0.0), (34.0, 1.7)):
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


# ---- MH: shelves ---------------------------------------------------------------------------------------------------------------------


SHELVES = [0.0, 4.0, 11.0, 22.0, 36.0, 52.0]


def ledge_offsets(rng, k):
    """How far the ledge under shelf k wanders out from its line, along x: a slow wave and two or three promontories."""
    spots = [(rng.uniform(-140, 140), rng.uniform(4.0, 8.0), rng.uniform(10.0, 18.0)) for _ in range(rng.randrange(2, 4))]
    ph = rng.uniform(0, math.tau)

    def off(x):
        return 1.6 * math.sin(x * 0.045 + ph) + 0.8 * math.sin(x * 0.12 + 2 * ph) + sum(o * math.exp(-((x - c) / w) ** 2) for (c, o, w) in spots)
    return off


def shelved_band(name, bumps, floor_y, z, profile, rng, lanes=False):
    """The rise as broad flat shelves stepping down toward the camera, each ending in a crisp dark ledge under a scalloped grass lip."""
    lo, hi = -150.0, 150.0
    xs = [lo + 2.5 * i for i in range(int((hi - lo) / 2.5) + 1)]

    def crest(x):
        return land_top(bumps, floor_y, x)

    # The land behind the crest, as the other looks have it.
    back = [(rr[0], rr[1]) for rr in profile if rr[1] < 0.0] + [(0.0, 0.0)]
    for j in range(len(xs) - 1):
        for i in range(len(back) - 1):
            (da, za), (db, zb_) = back[i], back[i + 1]
            face(part(name, "shelf"), (P(xs[j], crest(xs[j]) - da, z + za), P(xs[j], crest(xs[j]) - db, z + zb_),
                                       P(xs[j + 1], crest(xs[j + 1]) - db, z + zb_), P(xs[j + 1], crest(xs[j + 1]) - da, z + za)))
    offs = [ledge_offsets(rng, k) for k in range(len(SHELVES))]
    # (How much of each step a ledge takes along x, 0 to 1: where it falls to nothing two shelves run together, so no ledge crosses the
    # whole frame as a stripe; the last ledge takes up what the others leave.)
    phases = [(rng.uniform(0, math.tau), rng.uniform(0, math.tau)) for _ in SHELVES]

    def share(k, x):
        if k >= len(SHELVES) - 2:
            return 1.0
        a, b = phases[k]
        # (Slow: changing faster, the shelves between pulled into steep folded ramps.)
        return min(1.0, max(0.0, 0.55 + 1.1 * math.sin(x * 0.017 + a) + 0.4 * math.sin(x * 0.041 + b)))

    def drop(k, x):
        # The drop under the crest of shelf k's top at x.
        if k >= len(SHELVES) - 1:
            return SHELVES[-1]
        return sum((SHELVES[i + 1] - SHELVES[i]) * share(i, x) for i in range(k))

    ledges = []
    for k in range(len(SHELVES) - 1):
        d0, d1 = SHELVES[k], SHELVES[k + 1]
        # (One green: alternating, the shelves read as panels.)
        fam = "shelf"
        top = part("%s_s%d" % (name, k), fam)
        face_bm = part("%s_s%d" % (name, k), "ledge")
        lip = part("%s_s%d" % (name, k), "shelf_lip")
        hang = 0.5 + 0.03 * d1
        for j in range(len(xs) - 1):
            pts = []
            for x in (xs[j], xs[j + 1]):
                y0, y1 = crest(x) - drop(k, x), crest(x) - drop(k + 1, x)
                zb = z + depth_at(d0, profile) + (offs[k - 1](x) if k > 0 else 0.0)
                zf = z + depth_at(d1, profile) + offs[k](x)
                pts.append((x, y0, y1, zb, zf))
            (xa, ya0, ya1, zba, zfa), (xb, yb0, yb1, zbb, zfb) = pts
            face(top, (P(xa, ya0, zba), P(xa, ya0, zfa), P(xb, yb0, zfb), P(xb, yb0, zbb)))
            if ya0 - ya1 < 0.25 and yb0 - yb1 < 0.25:
                # (Two shelves run together here: no ledge, no lip.)
                continue
            face(face_bm, (P(xa, ya0, zfa), P(xa, ya1, zfa), P(xb, yb1, zfb), P(xb, yb0, zfb)))
            # The lip: the grass's cut edge a little proud of the ledge, and a scalloped tab hanging over it.
            la, lb = zfa + 0.3, zfb + 0.3
            face(lip, (P(xa, ya0, la), P(xa, ya0 - 0.45, la), P(xb, yb0 - 0.45, lb), P(xb, yb0, lb)))
            h = hang * rng.uniform(0.7, 1.3)
            face(lip, (P(xa, ya0 - 0.4, la), P(xb, yb0 - 0.4, lb), P(xb - 0.3, yb0 - 0.4 - h * 0.7, lb + 0.04),
                       P((xa + xb) * 0.5, (ya0 + yb0) * 0.5 - 0.4 - h, (la + lb) * 0.5 + 0.06), P(xa + 0.3, ya0 - 0.4 - h * 0.7, la + 0.04)))
        ledges.append((k, offs[k - 1] if k > 0 else (lambda _x: 0.0), offs[k]))
        # Tufts and flowers over the shelf, as the reference's lawns have them, bigger further down (nearer the camera).
        sz = 0.5 + 0.02 * d1
        for _ in range(int((hi - lo) * 0.6)):
            x = rng.uniform(lo, hi)
            zb = z + depth_at(d0, profile) + (offs[k - 1](x) if k > 0 else 0.0) + 0.6
            zf = z + depth_at(d1, profile) + offs[k](x) - 0.6
            if zf <= zb:
                continue
            zz = rng.uniform(zb, zf)
            y = crest(x) - drop(k, x)
            if rng.random() < 0.2:
                res = bmesh.ops.create_cone(part("%s_s%d" % (name, k), rng.choice(("flower_a", "flower_b"))), cap_ends=True, segments=6,
                                            radius1=0.35 * sz, radius2=0.12 * sz, depth=0.25 * sz)
                for v in res["verts"]:
                    v.co += P(x, y + 0.45 * sz, zz)
                continue
            for _t in range(3):
                ang = rng.uniform(0, math.pi)
                dx, dz = math.cos(ang) * 0.45 * sz, math.sin(ang) * 0.14 * sz
                face(part("%s_s%d" % (name, k), "tuft"), (P(x - dx, y + 0.01, zz - dz), P(x + dx, y + 0.01, zz + dz), P(x + dx * 0.2, y + 1.1 * sz, zz + dz * 0.2)))
    if lanes:
        # Two lanes down the shelves either side of the stage, each crossing a shelf and stepping down its ledge on stairs cut into it.
        for x0, ph in ((-36.0, 0.0), (34.0, 1.7)):
            lane, card = part("%s_lanes" % name, "lane"), part("%s_lanes" % name, "card_lane")
            for (k, off_b, off_f) in ledges:
                d0, d1 = SHELVES[k], SHELVES[k + 1]
                x = x0 + 3.0 * math.sin(k * 0.9 + ph)
                w = 0.55 + 0.015 * d1
                y0, y1 = crest(x) - drop(k, x), crest(x) - drop(k + 1, x)
                zb = z + depth_at(d0, profile) + off_b(x) + (0.0 if k == 0 else 1.6)
                zf = z + depth_at(d1, profile) + off_f(x)
                face(lane, (P(x - w, y0 + 0.04, zb), P(x - w, y0 + 0.04, zf), P(x + w, y0 + 0.04, zf), P(x + w, y0 + 0.04, zb)))
                # The stairs: treads stepping out from the ledge's foot back up to its top.
                if y0 - y1 < 0.6:
                    continue
                n = max(2, int((y0 - y1) / 0.9))
                for q in range(n):
                    ty = y0 - (y0 - y1) * (q + 1) / n
                    tz0 = zf + 0.0 + 0.8 * q
                    tz1 = tz0 + 0.8
                    face(lane, (P(x - w, ty, tz0), P(x - w, ty, tz1), P(x + w, ty, tz1), P(x + w, ty, tz0)))
                    face(card, (P(x - w, ty, tz1), P(x - w, ty - (y0 - y1) / n, tz1), P(x + w, ty - (y0 - y1) / n, tz1), P(x + w, ty, tz1)))
                    for sx in (x - w, x + w):
                        face(card, (P(sx, ty, tz0), P(sx, y1, tz0), P(sx, y1, tz1), P(sx, ty, tz1)))


FOREST = []


# Where the meadow's buildings stand (StageArt._meadow, make_world.py): the windmill, the hill cottages, the haystacks and the farm.
KEEP_CLEAR = [(-34.0, -73.0, 9.0), (44.0, -74.0, 8.0), (-66.0, -74.0, 8.0), (-57.0, -98.4, 9.0), (-32.0, -40.0, 12.0)]
BELTS = opts.get("belts", "1") == "1"


def pine_belts(rng):
    """The meadow's pine belts (the owner, 2026-10-22: the forest was a few sporadic trees; the reference's camp sits under dense belts
    of pines): clumps of the detailed pine along the near and far hills, five to ten trees a clump with gaps between, the far ones a
    little larger (the distance makes them smaller), clear of the buildings, the haystacks and the sky behind the fight."""
    k = 0
    for band_z, bumps, size, depth in ((-72.0, LAYOUT["hills_near"], (9.0, 13.0), (-1.0, 4.0)), (-100.0, LAYOUT["hills_far"], (10.0, 15.0), (-1.0, 5.0))):
        gx = -160.0 + rng.uniform(0, 8)
        while gx < 160.0:
            width = rng.uniform(12.0, 26.0)
            if abs(gx) > 20.0 + width * 0.5:
                for _ in range(int(width / 2.6)):
                    x = gx + rng.uniform(-width / 2, width / 2)
                    d = rng.uniform(*depth)
                    zz = band_z + depth_at(max(d, 0.0), ROLL) + rng.uniform(-2.5, 1.5)
                    if abs(x) < 20.0 or any(math.hypot(x - cx, zz - cz) < rad for (cx, cz, rad) in KEEP_CLEAR):
                        continue
                    base = P(x, land_top(bumps, -17.0, x) - max(d, 0.0) - 0.3, zz)
                    FOREST.append((base.x, base.y))
                    nm = "forest%d" % (k % 6)
                    pine_detailed(part(nm, "pine_%d" % rng.randrange(4)), part(nm, "pine_under"), part(nm, "trunk"), base,
                                  rng.uniform(*size) / 2.9, rng)
                    k += 1
            gx += width + rng.uniform(6.0, 16.0)
    print("meadow pine belts: %d pines" % k)


def forest_belts(rng):
    """Belts of pines along the near hills' crest, as the reference's mid-ground forest: dense groups of one kind of pine at one large
    size (about 15 tall), clear of the space behind the fight."""
    bumps = LAYOUT["hills_near"]
    gx = -150.0 + rng.uniform(0, 10)
    k = 0
    while gx < 150.0:
        if abs(gx) > 30.0:
            for _ in range(rng.randrange(5, 9)):
                x = gx + rng.uniform(-8.0, 8.0)
                if abs(x) < 26.0:
                    continue
                d = rng.uniform(0.0, 3.5)
                zz = -72.0 + depth_at(d, ROLL) + rng.uniform(-3.0, 1.0)
                base = P(x, land_top(bumps, -17.0, x) - d - 0.3, zz)
                FOREST.append((base.x, base.y))
                pine_discs(part("forest%d" % (k % 4), "pine_%d" % rng.randrange(4)), part("forest%d" % (k % 4), "pine_under"),
                           part("forest%d" % (k % 4), "trunk"), base, rng.uniform(5.2, 6.4), rng)
            k += 1
        gx += rng.uniform(28.0, 40.0)


# ---- QA, QB, QC: tidy quilt ------------------------------------------------------------------------------------------------------------


QUILT = ["qf_a", "qf_b", "qf_c", "qf_b", "qf_a", "wheat", "qf_c", "qf_b"]
LANES_X = (-47.0, 34.0)


def quilt_outline(cx, cd, rx, rd, n=24):
    """A field's outline in (x, drop): a rounded rectangle, every field the same shape."""
    out = []
    for i in range(n):
        a = math.tau * i / n
        c, s_ = math.cos(a), math.sin(a)
        out.append((cx + rx * math.copysign(abs(c) ** 0.3, c), max(0.15, cd + rd * math.copysign(abs(s_) ** 0.3, s_))))
    return out


def quilt_band(name, fam, bumps, floor_y, z, profile, rng, fields=False):
    at, normal = surface(bumps, floor_y, z, profile)
    lo, hi = -150.0, 150.0
    xs = [lo + 2.5 * i for i in range(int((hi - lo) / 2.5) + 1)]
    back = [(rr[0], rr[1]) for rr in profile if rr[1] < 0.0]
    drops = [0.0, 0.3, 0.7, 1.2, 1.8, 2.6, 3.6, 4.8, 6.4, 8.4, 11.0, 14.0, 18.0, 23.0, 29.0, 36.0, 44.0, 52.0, 60.0]
    order = back + [(d, depth_at(d, profile)) for d in drops]
    grid = [[P(x, land_top(bumps, floor_y, x) - d, z + dz) for (d, dz) in order] for x in xs]
    # (The land between the fields a deeper green than any field, so the gaps draw the quilt's lines; the same green as the fields, the
    # fields vanished. QC's lawn keeps the land's own green.)
    base = part(name, "qgap" if fields and VARIANT != "QC" else fam)
    for j in range(len(xs) - 1):
        for i in range(len(order) - 1):
            face(base, (grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]))
    if VARIANT == "QB":
        rows = [3.0, 9.0, 18.0, 30.0, 44.0] if fields else [1.4, 4.6, 9.6]
        width = 20.0
    else:
        rows = [2.2, 5.4, 9.6, 15.0, 22.0, 31.0, 42.0] if fields else [0.9, 2.6, 5.0, 8.4, 12.6]
        width = 10.5
    gaps = [(rows[k + 1] - rows[k]) if k + 1 < len(rows) else rows[k] * 0.3 for k in range(len(rows))]

    def lifted(q, lift):
        x, d = q.x, land_top(bumps, floor_y, q.x) - q.y
        return q + normal(x, d) * lift

    def on_lane(x, pad):
        return fields and any(abs(x - lx) < pad + 3.0 for lx in LANES_X)

    for k, cd in enumerate(rows):
        w = width * (1.0 + 0.03 * cd)
        n_cols = int((hi - lo) / w)
        for c in range(n_cols):
            cx = lo + (c + 0.5) * w + (w * 0.5 if k % 2 else 0.0)
            if cx > hi:
                continue
            # (Clear of the farm's own ground, and not with a lane down its middle; a lane over a field's edge is drawn over it, as on PB.)
            # (Only on the rows the farm stands on, about 5 under the crest.)
            if fields and cd < 10.0 and FARM and cx + w * 0.45 > FARM[0] - 1.5 and cx - w * 0.45 < FARM[1] + 1.5:
                continue
            if fields and any(abs(cx - lx) < w * 0.22 for lx in LANES_X):
                continue
            if VARIANT == "QC":
                # (Only round the farm and along the lanes, on the near rows.)
                near_farm = FARM and FARM[0] - 30.0 < cx < FARM[1] + 12.0
                near_lane = any(abs(cx - lx) < 18.0 for lx in LANES_X)
                if not fields or k > 3 or not (near_farm or near_lane):
                    continue
            fam_k = QUILT[(k * 3 + c) % len(QUILT)] if fields else fam + ("_l" if (k + c) % 2 else "_d")
            outline = quilt_outline(cx, cd, w * 0.42, gaps[k] * 0.34)
            ring = [at(x, d) for (x, d) in outline]
            sheet(part(name, fam_k), part(name, "qcard"), ring, at(cx, cd), lambda q: lifted(q, 0.08))
    if fields and VARIANT != "QC":
        # Hedgerows along the rows, between one row of fields and the next, broken at the lanes and the farm.
        bm = part(name, "qhedge")
        for k in range(len(rows) - 1):
            d = rows[k] + gaps[k] * 0.5
            rad = 0.2 + 0.006 * d
            hx = [lo + 3.0 * i for i in range(int((hi - lo) / 3.0) + 1)]
            for i in range(len(hx) - 1):
                xa, xb = hx[i], hx[i + 1]
                if on_lane(xa, 0.0) or on_lane(xb, 0.0) or farm_calm(xa) < 0.9 or farm_calm(xb) < 0.9:
                    continue
                pa = lifted(at(xa, d), rad)
                pb = lifted(at(xb, d), rad)
                tube_seg(bm, P(*pa), P(*pb), rad, segs=5)
    if fields:
        # The lanes, as PB has them.
        for x0, ph in zip(LANES_X, (0.0, 1.7)):
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


# ---- FA, FB, FC: hedged farmland ---------------------------------------------------------------------------------------------------


# (The farm's row wide, so the barn and the farmhouse stand on one shelf, not at a bank's edge.)
FARM_ROWS = [0.0, 2.0, 9.0, 13.5, 18.0, 23.5, 30.0, 38.0, 47.0, 56.0]
TERRACES = VARIANT in ("FB", "FC")
# Field kinds: [top family, row family or None, how often].
FIELD_KINDS = [("qf_a", None, 3), ("qf_b", None, 3), ("qf_c", None, 2), ("wheat", "wheat_row", 2), ("plough", "plough_row", 1),
               ("plough", "crop_row", 2)]
# The lanes as (x, drop) points, each from somewhere to somewhere: over the crest past the farm (and a spur into its yard), from the
# right-hand cottage, the lane across the slope that joins them, and on down toward the stage.
LANES = [
    [(-49.0, 0.0), (-48.6, 3.0), (-47.5, 7.0), (-45.0, 11.5), (-40.0, 16.5), (-33.0, 21.0)],
    [(-47.8, 5.0), (-43.0, 5.4), (-38.5, 5.8)],
    [(34.0, 0.0), (33.6, 3.5), (32.5, 8.0), (30.0, 13.0), (27.0, 18.0), (25.0, 21.0)],
    [(-170.0, 21.6), (-120.0, 20.8), (-75.0, 21.4), (-33.0, 21.0), (0.0, 21.2), (25.0, 21.0), (70.0, 20.6), (120.0, 21.4), (170.0, 21.0)],
    [(-33.0, 21.0), (-36.0, 27.0), (-41.0, 34.0), (-47.0, 42.0), (-54.0, 52.0)],
    [(25.0, 21.0), (28.5, 27.0), (33.0, 34.0), (38.0, 42.0), (44.0, 52.0)],
]
LANE_W = [1.1, 0.75, 1.1, 1.0, 1.0, 1.0]


def terraced(d, terr=True):
    """FB: a drop under the crest as the terraces lay it: across most of each row the land falls at a third of the slope, then drops to
    the next row in a short steep bank (the last fifth of the row)."""
    if not terr or not TERRACES or d <= 0.0 or d >= FARM_ROWS[-1]:
        return d
    for k in range(len(FARM_ROWS) - 1):
        d0, d1 = FARM_ROWS[k], FARM_ROWS[k + 1]
        if d0 <= d < d1:
            t = (d - d0) / (d1 - d0)
            if t < 0.8:
                return d0 + 0.3 * (d - d0)
            top = d0 + 0.3 * 0.8 * (d1 - d0)
            return top + (d1 - top) * (t - 0.8) / 0.2
    return d


def bank_of(d):
    """FB: whether a drop is on a row's bank (its last fifth)."""
    if not TERRACES:
        return False
    for k in range(len(FARM_ROWS) - 1):
        d0, d1 = FARM_ROWS[k], FARM_ROWS[k + 1]
        if d0 <= d < d1:
            return (d - d0) / (d1 - d0) >= 0.8
    return False


def farm_surface(bumps, floor_y, z, profile, terr=True):
    """`surface` with FB's terraces (`terr`): the land's height takes the terraced drop, its depth the true one."""
    def at(x, d):
        return Vector((x, land_top(bumps, floor_y, x) - terraced(d, terr), z + depth_at(max(d, 0.0), profile)))

    def normal(x, d):
        t1 = at(x + 0.5, d) - at(x - 0.5, d)
        t2 = at(x, d + 0.3) - at(x, max(d - 0.3, 0.0))
        n = t1.cross(t2)
        if n.z + n.y < 0.0:
            n = -n
        return n.normalized()
    return at, normal


def poly_len(pts):
    return sum((pts[i + 1] - pts[i]).length for i in range(len(pts) - 1))


def resample(pts2, step):
    """A lane's (x, drop) points every `step` along it."""
    out = [pts2[0]]
    for i in range(len(pts2) - 1):
        a, b = pts2[i], pts2[i + 1]
        n = max(1, int(math.hypot(b[0] - a[0], b[1] - a[1]) / step))
        for k in range(1, n + 1):
            t = k / n
            out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t))
    return out


def hedge_run(bm, pts, rng, rad):
    """A bushy hedgerow along world points: a rolled tube whose girth swells and shrinks, with a round bush now and then."""
    for i in range(len(pts) - 1):
        a, b = pts[i], pts[i + 1]
        rr = rad * rng.uniform(0.8, 1.15)
        tube_seg(bm, P(*a), P(*b), rr, segs=5)
        # (Low bushes: at this distance a twenty-faced one reads as round, and four times as many cost a third of the band.)
        if rng.random() < 0.3:
            ret = bmesh.ops.create_icosphere(bm, subdivisions=0, radius=rad * rng.uniform(1.3, 1.8))
            for v in ret["verts"]:
                v.co = Vector((v.co.x, v.co.y, v.co.z * 0.85)) + P(*a)


def farm_band(name, fam, bumps, floor_y, z, profile, rng, fields=True):
    """The band cut edge to edge into hedged fields (see FA above). `fields` False: the near hills, pasture only, no lanes."""
    at, normal = farm_surface(bumps, floor_y, z, profile, terr=fields)
    lo, hi = -160.0, 160.0
    back = [(rr[0], rr[1]) for rr in profile if rr[1] < 0.0]
    rows = FARM_ROWS if fields else [0.0, 1.6, 3.6, 6.0, 9.0, 12.6, 17.0]
    # The land under it all (shows only under the hedges, the lanes and the farmyard), back of the crest first.
    ds = sorted(set([0.0] + [rows[k] + (rows[k + 1] - rows[k]) * f for k in range(len(rows) - 1) for f in (0.0, 0.4, 0.8, 0.9)]
                    + [rows[-1], rows[-1] + 8.0, 60.0, 90.0]))
    order = [(d, None) for (d, _) in back] + [(d, 1) for d in ds]
    xs = [lo + 2.5 * i for i in range(int((hi - lo) / 2.5) + 1)]

    def ground(x, d, flag):
        if flag is None:
            return P(x, land_top(bumps, floor_y, x) - d, z + depth_at(0.0, profile) + [b for (dd, b) in back if dd == d][0])
        return P(*at(x, d))

    for j in range(len(xs) - 1):
        for i in range(len(order) - 1):
            (da, fa), (db, fb) = order[i], order[i + 1]
            mid = (da + db) * 0.5
            fam_c = "bank" if (fields and fa and fb and bank_of(mid)) else fam
            face(part(name, fam_c), (ground(xs[j], da, fa), ground(xs[j], db, fb), ground(xs[j + 1], db, fb), ground(xs[j + 1], da, fa)))
    # The lanes in world space, each with its sides, and how near a point of land is to one.
    lanes = []
    if fields:
        for k, lane in enumerate(LANES):
            pts = resample(lane, 0.8)
            world = [at(x, d) for (x, d) in pts]
            lanes.append((pts, world, LANE_W[k]))

    def near_lane(q, pad):
        for (_, world, w) in lanes:
            for i in range(len(world) - 1):
                a, b = world[i], world[i + 1]
                ab = b - a
                t = max(0.0, min(1.0, (q - a).dot(ab) / max(ab.length_squared, 1e-6)))
                if (a + ab * t - q).length < w + pad:
                    return True
        return False

    def in_yard(x, d):
        return fields and FARM and FARM[0] - 2.0 < x < FARM[1] + 2.0 and d < 8.0

    # The fields: each row along the contours, cut across by boundaries running down the slope (some run through several rows).
    long_bounds = []
    x = lo
    while x < hi:
        long_bounds.append(x)
        x += rng.uniform(22.0, 34.0) if fields else rng.uniform(16.0, 26.0)
    long_bounds.append(hi)
    kinds = [k for k in FIELD_KINDS if fields or k[1] is None for _ in range(k[2])]
    hedge_lines = []
    for r in range(len(rows) - 1):
        d0, d1 = rows[r], rows[r + 1]
        top_d1 = d0 + (d1 - d0) * (0.8 if TERRACES and fields else 1.0)
        bounds = []
        for i in range(len(long_bounds) - 1):
            a, b = long_bounds[i], long_bounds[i + 1]
            bounds.append(a)
            if rng.random() < 0.6:
                bounds.append(a + (b - a) * rng.uniform(0.4, 0.6))
        bounds.append(hi)
        wave = [rng.uniform(0, math.tau) for _ in bounds]

        def bx(i, d):
            # (A boundary wanders a little as it comes down; the outer ones stay put.)
            return bounds[i] + (1.6 * math.sin(d * 0.21 + wave[i]) if 0 < i < len(bounds) - 1 else 0.0)

        for i in range(len(bounds) - 1):
            kind = kinds[rng.randrange(len(kinds))]
            # (Round the farm mostly crops: a farm works the fields nearest it.)
            if fields and FARM and abs((bounds[i] + bounds[i + 1]) * 0.5 - (FARM[0] + FARM[1]) * 0.5) < 45.0 and d0 < 25.0 and rng.random() < 0.6:
                kind = [k for k in FIELD_KINDS if k[1]][rng.randrange(3)]
            top_f, row_f = kind[0] if fields else ("qf_b", "qf_a", "qf_c")[rng.randrange(3)], kind[1]
            nu = max(2, int((bounds[i + 1] - bounds[i]) / 2.0))
            nv = 4
            grid = []
            for v in range(nv + 1):
                d = d0 + (top_d1 - d0) * v / nv
                row_pts = []
                for u in range(nu + 1):
                    x = bx(i, d) + (bx(i + 1, d) - bx(i, d)) * u / nu
                    row_pts.append((x, d))
                grid.append(row_pts)
            cx = (bounds[i] + bounds[i + 1]) * 0.5
            if in_yard(cx, (d0 + d1) * 0.5):
                continue
            lift = 0.05
            for v in range(nv):
                for u in range(nu):
                    quad = [grid[v][u], grid[v][u + 1], grid[v + 1][u + 1], grid[v + 1][u]]
                    face(part(name, top_f), [P(*(at(x, d) + normal(x, d) * lift)) for (x, d) in quad])
            if row_f:
                # Rows across the field: along the contours, or down the slope, by turns.
                along = rng.random() < 0.5
                n_rows = max(3, int(((bx(i + 1, d0) - bx(i, d0)) if not along else (top_d1 - d0) * 2.2) / 0.9))
                for k in range(n_rows):
                    f0, f1 = (k + 0.25) / n_rows, (k + 0.65) / n_rows
                    strip = []
                    steps = 8
                    for sidx in range(steps + 1):
                        g = 0.04 + 0.92 * sidx / steps
                        if along:
                            dd0, dd1 = d0 + (top_d1 - d0) * f0, d0 + (top_d1 - d0) * f1
                            xa = bx(i, dd0) + (bx(i + 1, dd0) - bx(i, dd0)) * g
                            strip.append(((xa, dd0), (xa, dd1)))
                        else:
                            dd = d0 + (top_d1 - d0) * g
                            xa = bx(i, dd) + (bx(i + 1, dd) - bx(i, dd)) * f0
                            xb = bx(i, dd) + (bx(i + 1, dd) - bx(i, dd)) * f1
                            strip.append(((xa, dd), (xb, dd)))
                    for sidx in range(steps):
                        (p0, p1), (p2, p3) = strip[sidx], strip[sidx + 1]
                        q = [p0, p2, p3, p1]
                        face(part(name, row_f), [P(*(at(x, d) + normal(x, d) * 0.1)) for (x, d) in q])
            # This field's boundary down its right side (not at the band's ends).
            if i + 1 < len(bounds) - 1:
                hedge_lines.append([(bx(i + 1, d0 + (top_d1 - d0) * t / 6.0), d0 + (top_d1 - d0) * t / 6.0) for t in range(7)])
        # The row's lower boundary.
        if r + 1 < len(rows) - 1 or not fields:
            hd = top_d1
            hedge_lines.append([(lo + 4.0 * t, hd) for t in range(int((hi - lo) / 4.0) + 1)])
    # The hedgerows, open where a lane passes and round the farmyard.
    hedge_bm = part(name, "qhedge")
    rad = 0.32 if fields else 0.24
    for line in hedge_lines:
        run = []
        for (x, d) in line:
            q = at(x, d)
            if near_lane(q, 0.5) or in_yard(x, d):
                if len(run) > 1:
                    hedge_run(hedge_bm, [p + normal(*xd) * rad for p, xd in run], rng, rad)
                run = []
                continue
            run.append((q, (x, d)))
        if len(run) > 1:
            hedge_run(hedge_bm, [p + normal(*xd) * rad for p, xd in run], rng, rad)
        CORNERS.append(at(*line[0]))
    # The lanes: a pale ribbon with a grass crown down its middle, its cut edges, and hedges along both sides.
    for (pts, world, w) in lanes:
        left, right, crown_l, crown_r = [], [], [], []
        for k, (x, d) in enumerate(pts):
            p = world[k]
            t = world[min(k + 1, len(world) - 1)] - world[max(k - 1, 0)]
            n = normal(x, d)
            side = t.cross(n).normalized()
            lift = n * 0.22
            left.append(p - side * w + lift)
            right.append(p + side * w + lift)
            crown_l.append(p - side * w * 0.12 + n * 0.25)
            crown_r.append(p + side * w * 0.12 + n * 0.25)
        for k in range(len(pts) - 1):
            face(part(name, "lane"), (P(*left[k]), P(*left[k + 1]), P(*right[k + 1]), P(*right[k])))
            face(part(name, "verge"), (P(*crown_l[k]), P(*crown_l[k + 1]), P(*crown_r[k + 1]), P(*crown_r[k])))
            for sd in (left, right):
                face(part(name, "card_lane"), (P(*sd[k]), P(*sd[k + 1]), P(*(sd[k + 1] - Vector((0, 0.3, 0)))), P(*(sd[k] - Vector((0, 0.3, 0))))))
        for sgn in (-1.0, 1.0):
            run = []
            for k, (x, d) in enumerate(pts):
                # (Every third of the lane's points: at its own spacing the lanes' hedges were half the band's triangles.)
                if k % 3 and k != len(pts) - 1:
                    continue
                p = world[k]
                t = world[min(k + 1, len(world) - 1)] - world[max(k - 1, 0)]
                n = normal(x, d)
                q = p + t.cross(n).normalized() * sgn * (w + 0.75) + n * 0.28
                # (Open where another lane joins, and round the farmyard.)
                if in_yard(x, d) or any(near_lane_other(q, other) for other in lanes if other[1] is not world):
                    if len(run) > 1:
                        hedge_run(hedge_bm, run, rng, 0.28)
                    run = []
                    continue
                run.append(q)
            if len(run) > 1:
                hedge_run(hedge_bm, run, rng, 0.28)
    if fields and VARIANT == "FC":
        copse(rng, at)


def near_lane_other(q, lane):
    (_, world, w) = lane
    for i in range(len(world) - 1):
        a, b = world[i], world[i + 1]
        ab = b - a
        t = max(0.0, min(1.0, (q - a).dot(ab) / max(ab.length_squared, 1e-6)))
        if (a + ab * t - q).length < w + 1.6:
            return True
    return False


CORNERS = []


def copse(rng, at):
    """FC: pines along the hedgerows, in twos and threes where boundaries start, and a copse behind the farm."""
    k = 0
    spots = list(CORNERS)
    rng.shuffle(spots)
    for c in spots:
        if abs(c.x) < 22.0 or abs(c.x) > 140.0 or rng.random() < 0.75 or any(math.hypot(c.x - cx, c.z - cz) < rad for (cx, cz, rad) in KEEP_CLEAR):
            continue
        for _ in range(rng.randrange(1, 4)):
            q = c + Vector((rng.uniform(-2.0, 2.0), 0.0, rng.uniform(-1.0, 1.0)))
            base = P(q.x, q.y - 0.2, q.z)
            FOREST.append((base.x, base.y))
            nm = "forest%d" % (k % 4)
            pine_detailed(part(nm, "pine_%d" % rng.randrange(4)), part(nm, "pine_under"), part(nm, "trunk"), base, rng.uniform(5.0, 7.5) / 2.9, rng)
            k += 1
    if FARM:
        for _ in range(9):
            x = rng.uniform(FARM[0] - 9.0, FARM[0] - 3.0)
            d = rng.uniform(0.3, 2.5)
            q = at(x, d)
            base = P(q.x, q.y - 0.2, q.z)
            FOREST.append((base.x, base.y))
            nm = "forest%d" % (k % 4)
            pine_detailed(part(nm, "pine_%d" % rng.randrange(4)), part(nm, "pine_under"), part(nm, "trunk"), base, rng.uniform(6.0, 8.5) / 2.9, rng)
            k += 1
    print("meadow hedgerow pines: %d" % k)


# ---- Build --------------------------------------------------------------------------------------------------------------------------


def build_ground():
    rng = random.Random(61)
    if VARIANT == "MH":
        layered_band("far", "hill_far", LAYOUT["hills_far"], -17.0, -100.0, ROLL, rng)
        layered_band("near", "hill_near", LAYOUT["hills_near"], -17.0, -72.0, ROLL, rng)
        shelved_band("rise", LAYOUT["fore"], -11.0, -46.0, VALLEY, rng, lanes=True)
        forest_belts(rng)
        hill_layers(rng)
        return
    if VARIANT in ("FA", "FB", "FC"):
        quilt_band("far", "hill_far", LAYOUT["hills_far"], -17.0, -100.0, ROLL, rng)
        farm_band("near", "hill_near", LAYOUT["hills_near"], -17.0, -72.0, ROLL, rng, fields=False)
        farm_band("rise", "rise", LAYOUT["fore"], -11.0, -46.0, VALLEY, rng, fields=True)
        if BELTS:
            pine_belts(random.Random(97))
        hill_layers(rng)
        return
    band = {"A": folded_band, "B": layered_band, "C": faceted_band, "PA": patchwork_band, "PB": patchwork_band, "PC": patchwork_band,
            "QA": quilt_band, "QB": quilt_band, "QC": quilt_band}[VARIANT]
    band("far", "hill_far", LAYOUT["hills_far"], -17.0, -100.0, ROLL, rng)
    band("near", "hill_near", LAYOUT["hills_near"], -17.0, -72.0, ROLL, rng)
    band("rise", "rise", LAYOUT["fore"], -11.0, -46.0, VALLEY, rng, fields=True)
    if VARIANT in ("QA", "QB", "QC") and BELTS:
        pine_belts(random.Random(97))
    if VARIANT in ("B", "PA", "PB", "PC", "QA", "QB", "QC"):
        hill_layers(rng)
    else:
        hill_field()


PINE_SPOTS = [(0.0, 0.0, 1.0), (-0.55, -0.12, 0.72), (0.5, -0.2, 0.8), (0.22, 0.12, 0.6), (-0.3, -0.3, 0.66)]


def build_pines():
    """One cluster at size 1 (the game scales it by each spot's size): the tallest pine 2.5 tall, three or four smaller round it, as
    `StageArt._pine_cluster` sets them, each in its own green."""
    rng = random.Random(83)
    pine = {"A": pine_pleated, "C": pine_faceted}.get(VARIANT, pine_discs)
    if opts.get("pine", "") == "D":
        pine = pine_detailed
    # (Each look's pine at the code-built pine's height, 2.5 at size 1; MH's half as tall again, as the forest belts behind them.)
    sc = {"A": 0.9, "MH": 1.2}.get(VARIANT, 1.0)
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
            elif name.startswith("forest"):
                # (Away from the nearest pine's trunk, as the clusters' pines.)
                fx, fy = min(FOREST, key=lambda q: (c.x - q[0]) ** 2 + (c.y - q[1]) ** 2)
                away = Vector((c.x - fx, c.y - fy, 0.0))
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
    # (Lighter on the tidy quilt: the deep crease shade was much of PB's muddiness.)
    bake_occlusion({"PC": 5.0, "QA": 1.5, "QB": 1.5, "QC": 1.5, "FA": 1.5, "FB": 2.0, "FC": 1.5}.get(VARIANT, 3.0))
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
