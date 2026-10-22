"""World-building mocks (docs/art/stages/WORLD.md, the owner's go-ahead 2026-10-21): what makes a stage a place, in the paper language
(rolled tubes, scored card, cut-outs), built in world space for the exact stage and placed by the stage demo's `--world-mock=<glb>`.
  ocean (the beach, a fishing harbour)
    A  a working pier: a fishing boat moored along the far boardwalk with lines to its posts, nets hung on its rail, fish crates;
    B  harbour life: a rowboat tied off the dock's left end (its gulls taken out: no animals in the backgrounds, the owner);
    C  the town at work: market awnings and townsfolk on the quay, bunting, chimney smoke.
  sunset (the canyon, a desert crossing at dusk)
    A  a railroad: a timber trestle across the floor behind the buttes with a little train on it and a water tower by the track;
    B  a wagon trail: a pale trail winding across the floor to the horizon, telegraph poles along it, a covered wagon on it;
    C  a ranch: a windpump and its tank, a corral, an adobe house with chimney smoke.
  meadow  A a farm track and hay cart, B the fields at work (haystacks), C village life (smoke, washing, a scarecrow)
  grove   A a far tree village (great trees with lit huts and rope bridges), B festival gates and streamers, C floating islets
  city    A street traffic (a bus, people), B a billboard and a rooftop party, C an elevated train
  L  (every stage that has them) new living things: the grove's fireflies.
No animals in the backgrounds (the owner, 2026-10-21: at the sizes that read they never looked believable beside the rest); what is far is
built at its true size, as the cottages are, and takes the stage's air in the game.
Every part names its family (painted by the game, `StageBake.place_world`), with its occlusion baked; what moves is grouped under a
pivot empty `move_<kind>_<n>` (scripts/world_motion.gd).

Run (Blender 5.2):
  blender --background --python art/blender/make_world.py -- --theme ocean --variant A --out <file.glb>   (`--variant AB` for both)
Shipped (the owner's picks, 2026-10-21), in godot/models/stages/: beach_world.glb (`--theme ocean --variant AB`), canyon_world.glb
(`--theme sunset --variant A --trestle back`, the trestle set back since fidelity review 5), meadow_world.glb (`--theme meadow --variant BC`), grove_world.glb (`--theme grove --variant ABL`),
city_world.glb (`--theme city --variant AB`).
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import make_paper_props as kit  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
THEME = opts.get("theme", "ocean")
VARIANT = opts.get("variant", "A").upper()
SEA = -9.0
FLOOR = -16.0
ROOT = None


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


def ob(name, bm, fam, smooth=False, face=None):
    return kit.to_object(name, bm, fam, ROOT, smooth=smooth, face=face)


def tube(bm, a, b, r, segs=8):
    kit.tube(bm, P(*a), P(*b), r, segs=segs)


def boxg(x0, x1, y0, y1, z0, z1, fam, name):
    """A box in game coordinates."""
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        g = (x0 + (v.co.x + 0.5) * (x1 - x0), y0 + (v.co.z + 0.5) * (y1 - y0), z0 + (v.co.y + 0.5) * (z1 - z0))
        v.co = P(*g)
    return ob(name, bm, fam)


def card(pts, fam, name, thick=0.04, normal=(0.0, 0.0, 1.0)):
    """A flat card through game points `pts`, `thick` deep along -`normal` (game coordinates)."""
    bm = bmesh.new()
    vs = [bm.verts.new(P(*p)) for p in pts]
    f = bm.faces.new(vs)
    r = bmesh.ops.extrude_face_region(bm, geom=[f])
    moved = [v for v in r["geom"] if isinstance(v, bmesh.types.BMVert)]
    n = P(*normal)
    bmesh.ops.translate(bm, verts=moved, vec=-n * thick)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return ob(name, bm, fam)


# What moves (scripts/world_motion.gd, round 2 of the sheet): [kind, pivot (game coordinates), the parts it made], innermost first.
MOVERS = []


def moving(kind, pivot, build, *args):
    """Runs `build` and marks what it made as one thing that moves (`kind`, about `pivot`); `group_movers` puts it under its pivot."""
    before = set(bpy.data.objects)
    build(*args)
    MOVERS.append([kind, pivot, set(o for o in bpy.data.objects if o not in before and o.type == "MESH")])


def group_movers():
    """Each mover becomes an empty `move_<kind>_<n>` at its pivot, its parts under it with their points about it, so the game turns and
    moves it there; one riding on another (the smoke on the train) goes under that one."""
    empties = []
    for n, (kind, pivot, parts) in enumerate(MOVERS):
        e = bpy.data.objects.new("move_%s_%d" % (kind, n), None)
        bpy.context.scene.collection.objects.link(e)
        empties.append(e)
    def owner(parts):
        best = None
        for m, (_k, _p, other) in enumerate(MOVERS):
            if parts < other and (best is None or len(other) < len(MOVERS[best][2])):
                best = m
        return best
    for n, (kind, pivot, parts) in enumerate(MOVERS):
        up = owner(parts)
        e = empties[n]
        e.parent = ROOT if up is None else empties[up]
        e.location = P(*pivot) - (Vector() if up is None else P(*MOVERS[up][1]))
    for o in bpy.context.scene.objects:
        if o.type != "MESH":
            continue
        home = None
        for n, (_k, _p, parts) in enumerate(MOVERS):
            if o in parts and (home is None or len(parts) < len(MOVERS[home][2])):
                home = n
        if home is None:
            continue
        c = P(*MOVERS[home][1])
        for v in o.data.vertices:
            v.co -= c
        o.parent = empties[home]
    # (Or the exporter reads the new pivots' world places as the origin.)
    bpy.context.view_layer.update()


def turned(pts, centre, angle):
    """`pts` [(x, y, z) about the origin] turned `angle` radians about the vertical and moved to `centre`."""
    c, s = math.cos(angle), math.sin(angle)
    return [(centre[0] + x * c + z * s, centre[1] + y, centre[2] - x * s + z * c) for (x, y, z) in pts]



def loft(sections, fam, name, smooth=True, cap=True):
    """A skin through `sections` (each a list of game points, all the same count), its ends capped: a lofted hull or body."""
    bm = bmesh.new()
    rings = [[bm.verts.new(P(*q)) for q in sec] for sec in sections]
    n = len(sections[0])
    for r0, r1 in zip(rings, rings[1:]):
        for i in range(n - 1):
            bm.faces.new((r0[i], r0[i + 1], r1[i + 1], r1[i]))
    if cap:
        for r in (rings[0], rings[-1]):
            if len(set(v.co.to_tuple(4) for v in r)) > 2:
                bm.faces.new(r)
    return ob(name, bm, fam, smooth=smooth)


def scallop(cx, cy, z, rx, ry, lobes, phase, depth=0.16, n=28):
    """The outline of a rounded, scalloped cut-out (a puff of smoke, a leafy edge) in the front plane."""
    out = []
    for k in range(n):
        a = math.tau * k / n
        w = 1.0 + depth * math.sin(lobes * a + phase)
        out.append((cx + rx * w * math.cos(a), cy + ry * w * math.sin(a), z))
    return out

# ---- Shared pieces -------------------------------------------------------------------------------------------------------------------


def smoke(at, n=5, size=0.6, rng=None):
    """Chimney smoke as cut-card puffs, scalloped and thick, rising and drifting, larger up the column (each puff its own part, so the
    column can curl: `move_smoke`). (Faceted balls read as low-poly rock beside the paper clouds.)"""
    rng = rng or random.Random(5)
    for k in range(n):
        t = k / max(1, n - 1)
        c = Vector(at) + Vector((t * 1.6 + rng.uniform(-0.2, 0.2), k * size * 1.3, 0.02 * k))
        r = size * (0.7 + 0.6 * t)
        card(scallop(c.x, c.y, c.z, r, r * 0.86, rng.choice((3, 4)), rng.uniform(0.0, math.tau)), "smoke", "smoke", thick=0.3 * r)


def smoke_column(at, n=5, size=0.6, rng=None):
    """Smoke that curls (`move_smoke`, about the chimney's top)."""
    moving("smoke", at, smoke, at, n, size, rng)


def figure(at, height, fam, rng, turn=0.0):
    """A townsperson as a layered paper doll, as chibi as the fighters: legs, a coat with its arms, a big round head with a cap of hair,
    each its own thin card one layer in front of the last."""
    x, y, z = at
    h = height
    for lx in (-0.08, 0.08):
        card([(x + lx - 0.055 * h, y, z), (x + lx + 0.055 * h, y, z), (x + lx + 0.05 * h, y + 0.24 * h, z), (x + lx - 0.05 * h, y + 0.24 * h, z)],
             "trousers", "figure_legs", thick=0.04)
    coat = [(x - 0.2 * h, y + 0.2 * h, z + 0.01), (x + 0.2 * h, y + 0.2 * h, z + 0.01), (x + 0.24 * h, y + 0.34 * h, z + 0.01),
            (x + 0.17 * h, y + 0.36 * h, z + 0.01), (x + 0.15 * h, y + 0.56 * h, z + 0.01), (x - 0.15 * h, y + 0.56 * h, z + 0.01),
            (x - 0.17 * h, y + 0.36 * h, z + 0.01), (x - 0.24 * h, y + 0.34 * h, z + 0.01)]
    card(coat, fam, "figure", thick=0.05)
    card(scallop(x, y + 0.76 * h, z + 0.02, 0.2 * h, 0.19 * h, 0, 0.0, depth=0.0, n=18), "skin", "head", thick=0.05)
    hair = [(x + 0.205 * h * math.cos(a), y + 0.79 * h + 0.2 * h * math.sin(a), z + 0.03) for a in [math.pi * k / 12 for k in range(13)]]
    card(hair, "hair", "hair", thick=0.03)


# ---- The beach -----------------------------------------------------------------------------------------------------------------------


# The far boardwalk, from the back of the dock out to the fisherman's jetty (StageArt._dock_harbour): its deck.
WALK_A = Vector((9.0, -0.1, -6.2))
WALK_B = Vector((28.4, SEA + 4.1, -45.0))


def walk_at(t):
    return WALK_A.lerp(WALK_B, t)


def fishing_boat(at, angle):
    """A small fishing boat of card: a lofted hull with a raked bow and a sheer rising to it, a rolled gunwale, a deck, a wheelhouse with
    windows and an overhanging roof, a life ring, fenders on the side, a mast with a furled sail and a boom."""
    y0 = at[1] - 0.4

    def hull_points(scale_w, top_lift):
        secs = []
        for k in range(9):
            u = -2.8 + 5.8 * k / 8
            bow = max(0.0, (u - 1.0) / 2.0)
            w = 0.9 * scale_w * (1.0 - bow ** 1.6 * 0.92)
            top = 1.15 + top_lift + 0.4 * max(0.0, u) / 3.0
            keel = 0.05 + 0.35 * bow ** 2
            pts = [(u, y0 + top, -w), (u, y0 + 0.55, -w * 0.97), (u, y0 + keel + 0.12, -w * 0.62), (u, y0 + keel, 0.0),
                   (u, y0 + keel + 0.12, w * 0.62), (u, y0 + 0.55, w * 0.97), (u, y0 + top, w)]
            secs.append(turned(pts, (at[0], 0.0, at[2]), angle))
        return secs

    secs = hull_points(1.0, 0.0)
    loft(secs, "hull", "hull")
    # The rolled gunwale along each side, and a coloured stripe below it.
    for i in (0, 6):
        bm = bmesh.new()
        for a, b in zip(secs, secs[1:]):
            kit.tube(bm, P(*a[i]), P(*b[i]), 0.09, segs=6)
        ob("gunwale", bm, "hull_stripe", smooth=True)
    stripe = hull_points(1.015, -0.45)
    loft([[sec[0], sec[1]] for sec in stripe], "hull_stripe", "stripe", cap=False)
    loft([[sec[5], sec[6]] for sec in stripe], "hull_stripe", "stripe", cap=False)
    deck = [sec[0] for sec in secs] + [sec[6] for sec in reversed(secs)]
    card([(q[0], q[1] - 0.12, q[2]) for q in deck], "deck", "deck", thick=0.1, normal=(0, 1, 0))
    # The wheelhouse: walls, windows, a roof that overhangs, a life ring on its side.
    c = turned([(-0.9, 0.0, 0.0)], (at[0], 0.0, at[2]), angle)[0]
    wx, wz = c[0], c[2]
    boxg(wx - 0.65, wx + 0.65, y0 + 1.0, y0 + 2.45, wz - 0.6, wz + 0.6, "cabin", "wheelhouse")
    boxg(wx - 0.85, wx + 0.85, y0 + 2.45, y0 + 2.65, wz - 0.8, wz + 0.8, "cabin_roof", "roof")
    for dx in (-0.32, 0.32):
        boxg(wx + dx - 0.2, wx + dx + 0.2, y0 + 1.75, y0 + 2.2, wz + 0.6, wz + 0.64, "window", "wheelhouse_window")
    bm = bmesh.new()
    kit.torus(bm, P(wx + 0.0, y0 + 1.5, wz + 0.66), Vector((0.0, -1.0, 0.0)), 0.26, 0.07, segs=14, rsegs=5)
    ob("life_ring", bm, "flag_a", smooth=True)
    # Fenders hung over the side.
    bm = bmesh.new()
    for u in (-1.6, 0.0, 1.4):
        q = turned([(u, 0.0, 0.95)], (at[0], 0.0, at[2]), angle)[0]
        kit.tube(bm, P(q[0], y0 + 0.75, q[2]), P(q[0], y0 + 0.35, q[2]), 0.12, segs=8)
    ob("fenders", bm, "rope", smooth=True)
    mx, mz = at[0] + math.cos(angle) * 0.9, at[2] - math.sin(angle) * 0.9
    bm = bmesh.new()
    tube(bm, (mx, at[1] + 0.5, mz), (mx, at[1] + 5.2, mz), 0.08, segs=8)
    tube(bm, (mx, at[1] + 1.4, mz), (mx - math.cos(angle) * 2.4, at[1] + 1.5, mz + math.sin(angle) * 2.4), 0.06, segs=8)
    ob("mast", bm, "wood_dark", smooth=True)
    bm = bmesh.new()
    tube(bm, (mx - math.cos(angle) * 0.2, at[1] + 1.65, mz + math.sin(angle) * 0.2), (mx - math.cos(angle) * 2.3, at[1] + 1.75, mz + math.sin(angle) * 2.3), 0.16, segs=10)
    ob("furled_sail", bm, "sail", smooth=True)


def scaled(pivot, s, build, *args):
    """Runs `build` and scales what it made by `s` about `pivot` (game coordinates): the match camera sees the water and the floor only
    far out, where a thing must be large to read."""
    before = set(bpy.data.objects)
    movers = len(MOVERS)
    build(*args)
    c = P(*pivot)
    for o in [o for o in bpy.data.objects if o not in before and o.type == "MESH"]:
        for v in o.data.vertices:
            v.co = c + (v.co - c) * s
    # (And the pivots of what moves among them: the smoke rides on the train's stack wherever it ends up.)
    for m in MOVERS[movers:]:
        m[1] = tuple(pivot[i] + (m[1][i] - pivot[i]) * s for i in range(3))


def beach_a():
    # (Out where the match camera sees the water over the dock: anything nearer stands behind the deck.) A fishing boat moored to a
    # buoy off the fisherman's hut, nets over its side and crates on its deck.
    bx, bz, angle = 15.0, -66.0, 0.35
    moving("bob", (bx, SEA, bz), boat_moored, bx, bz, angle)
    beach_walk()


def crate(x, y, z, w, h, d):
    """A slatted crate of the catch: its box, slats across its front, posts at its corners."""
    boxg(x - w / 2.0, x + w / 2.0, y, y + h, z - d / 2.0, z + d / 2.0, "fish_crate", "crate")
    for k in (0.25, 0.7):
        boxg(x - w / 2.0, x + w / 2.0, y + h * k, y + h * k + h * 0.16, z + d / 2.0, z + d / 2.0 + 0.04, "timber_dark", "crate_slat")
    for sx in (-1.0, 1.0):
        boxg(x + sx * w / 2.0 - 0.06 * sx - 0.03, x + sx * w / 2.0 - 0.06 * sx + 0.03, y, y + h, z + d / 2.0, z + d / 2.0 + 0.05, "timber_dark", "crate_post")


def boat_moored(bx, bz, angle):
    scaled((bx, SEA, bz), 1.6, fishing_boat, (bx, SEA + 0.1, bz), angle)
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=0.5, matrix=Matrix.Translation(P(bx + 7.0, SEA + 0.2, bz + 2.5)))
    ob("buoy", bm, "flag_a", smooth=True)
    bm = bmesh.new()
    tube(bm, (bx + 4.0, SEA + 1.5, bz + 1.0), (bx + 7.0, SEA + 0.5, bz + 2.5), 0.05, segs=5)
    ob("mooring", bm, "rope", smooth=True)
    card([(bx - 2.0, SEA + 1.6, bz + 1.35), (bx + 1.2, SEA + 1.6, bz + 1.4), (bx + 0.8, SEA + 0.2, bz + 1.45), (bx - 1.6, SEA + 0.4, bz + 1.4)],
         "net", "boat_net", thick=0.03)
    for k, dx in enumerate((-3.0, -2.2)):
        crate(bx + dx, SEA + 1.55, bz, 0.9, 0.7, 0.8)


def beach_walk():
    d = (WALK_B - WALK_A).normalized()
    side = Vector((-d.z, 0.0, d.x))
    # Nets hung over the boardwalk's rail and fish crates stacked by them.
    for k, tt in enumerate((0.22, 0.3)):
        p = walk_at(tt) + side * 0.86
        q = walk_at(tt + 0.06) + side * 0.86
        net = [(p.x, p.y + 1.0, p.z), (q.x, q.y + 1.0, q.z), (q.x, q.y - 0.3, q.z), ((p.x + q.x) / 2, p.y - 0.6, (p.z + q.z) / 2), (p.x, p.y - 0.25, p.z)]
        card(net, "net", "net%d" % k, thick=0.02, normal=(side.x, 0, side.z))
    for k, (tt, h) in enumerate(((0.12, 0.0), (0.135, 0.0), (0.125, 0.45))):
        p = walk_at(tt) - side * 0.3
        crate(p.x, p.y + 0.1 + h, p.z, 0.6, 0.45, 0.5)


def beach_b():
    # A rowboat by the sailboat, bobbing. (Its gulls are gone: no animals in the backgrounds.)
    moving("bob", (-20.0, SEA, -62.0), scaled, (-20.0, SEA, -62.0), 1.6, rowboat, -20.0, -62.0)


def rowboat(x0, z0):
    """A rowboat beside the sailboat: a lofted hull, pointed at the bow, a rolled gunwale, two thwarts, its oars shipped."""
    secs = []
    for k in range(7):
        u = -1.5 + 3.1 * k / 6
        bow = max(0.0, (u - 0.3) / 1.3)
        w = 0.55 * (1.0 - bow ** 1.5 * 0.9)
        top = 0.55 + 0.15 * max(0.0, u) / 1.6
        secs.append([(x0 + u, SEA - 0.1 + top, z0 - w), (x0 + u, SEA - 0.1 + 0.15, z0 - w * 0.6), (x0 + u, SEA - 0.1, z0),
                     (x0 + u, SEA - 0.1 + 0.15, z0 + w * 0.6), (x0 + u, SEA - 0.1 + top, z0 + w)])
    loft(secs, "hull", "row_hull")
    for i in (0, 4):
        bm = bmesh.new()
        for a, b in zip(secs, secs[1:]):
            kit.tube(bm, P(*a[i]), P(*b[i]), 0.05, segs=6)
        ob("row_gunwale", bm, "hull_stripe", smooth=True)
    for u in (-0.6, 0.4):
        boxg(x0 + u - 0.14, x0 + u + 0.14, SEA + 0.3, SEA + 0.36, z0 - 0.45, z0 + 0.45, "deck", "thwart")
    bm = bmesh.new()
    tube(bm, (x0 - 0.2, SEA + 0.6, z0 - 0.4), (x0 + 1.0, SEA + 0.6, z0 + 0.6), 0.04, segs=6)
    tube(bm, (x0 - 0.6, SEA + 0.6, z0 + 0.4), (x0 + 0.6, SEA + 0.6, z0 - 0.6), 0.04, segs=6)
    ob("oars", bm, "wood_dark", smooth=True)


def beach_c():
    # (Twice life size: at the quay's distance, life size read as specks.)
    scaled((-66.0, SEA + 1.4, -95.5), 2.0, market)


def market():
    # The town's quay (StageArt._harbour_town, its base at (-66, water, -100)): awnings, people, bunting, smoke.
    rng = random.Random(9)
    bx, by, bz = -66.0, SEA + 1.4, -95.5
    for k, x in enumerate((-12.0, -4.0, 4.0)):
        cx = bx + x
        bm = bmesh.new()
        for dx in (-1.3, 1.3):
            for dz in (-0.8, 0.6):
                tube(bm, (cx + dx, by, bz + dz), (cx + dx, by + 2.0, bz + dz), 0.06, segs=6)
        ob("stall_poles", bm, "wood_dark", smooth=True)
        for s in range(6):
            x0 = cx - 1.5 + s * 0.5
            card([(x0, by + 2.1, bz + 0.9), (x0 + 0.5, by + 2.1, bz + 0.9), (x0 + 0.5, by + 2.6, bz - 0.9), (x0, by + 2.6, bz - 0.9)],
                 "awning_a" if (s + k) % 2 == 0 else "awning_b", "awning", thick=0.03, normal=(0, 1, 0.4))
        boxg(cx - 1.2, cx + 1.2, by, by + 0.9, bz - 0.6, bz + 0.5, "stall", "counter")
    for k in range(9):
        x = bx + rng.uniform(-18.0, 14.0)
        figure((x, by, bz + rng.uniform(0.8, 2.5)), rng.uniform(1.3, 1.7), rng.choice(("coat_a", "coat_b", "coat_c")), rng)
    # Bunting along the quay's front, sagging between poles.
    bm = bmesh.new()
    pts = []
    for k in range(4):
        x = bx - 18.0 + k * 11.0
        tube(bm, (x, by, bz + 3.0), (x, by + 3.4, bz + 3.0), 0.05, segs=6)
    ob("bunting_poles", bm, "wood_dark", smooth=True)
    for seg in range(3):
        xa = bx - 18.0 + seg * 11.0
        for k in range(9):
            t0, t1 = k / 9, (k + 1) / 9
            sag0 = math.sin(t0 * math.pi) * 0.9
            sag1 = math.sin(t1 * math.pi) * 0.9
            a = (xa + 11.0 * t0, by + 3.3 - sag0, bz + 3.0)
            b = (xa + 11.0 * t1, by + 3.3 - sag1, bz + 3.0)
            m = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2 - 0.55, bz + 3.0)
            card([a, b, m], ("flag_a", "flag_b", "flag_c")[(k + seg) % 3], "flag", thick=0.01)
    smoke_column((bx + 3.0, SEA + 10.0, -108.0), 6, 0.9, rng)
    smoke_column((bx - 9.0, SEA + 9.0, -105.0), 5, 0.8, rng)


# ---- The canyon ----------------------------------------------------------------------------------------------------------------------


def canyon_a():
    rng = random.Random(4)
    # (Nearer and higher than first tried, and bolder: at -110 the trestle read as a hairline behind the buttes.)
    # (`--trestle back|short`, fidelity review 5: as one line just over the platforms it ran the eye across the whole frame. `back` sets
    # it behind the far row, thinner and in the air; `short` runs it only from behind one near butte to behind another, so it has ends.
    # Set lower, it stood right behind the fighters at the match camera.)
    trestle = opts.get("trestle", "high")
    z = -124.0 if trestle == "back" else -92.0
    top = -6.0 if trestle == "back" else -2.5
    x0, x1 = (-52.0, 68.0) if trestle == "short" else (-110.0, 110.0)
    # The trestle: bents of rolled timber every 7 units, cross-braced, carrying the deck.
    bm = bmesh.new()
    x = x0
    while x <= x1:
        for dz in (-1.1, 1.1):
            tube(bm, (x, FLOOR, z + dz * 1.6), (x, top, z + dz), 0.32, segs=8)
        tube(bm, (x, FLOOR + 1.5, z - 1.9), (x, top - 1.0, z + 1.1), 0.18, segs=6)
        tube(bm, (x, FLOOR + 1.5, z + 1.9), (x, top - 1.0, z - 1.1), 0.18, segs=6)
        tube(bm, (x - 3.5, top - 1.2, z + 1.2), (x + 3.5, top - 1.2, z + 1.2), 0.14, segs=6)
        x += 7.0
    ob("trestle", bm, "timber", smooth=True)
    boxg(x0, x1, top - 0.2, top + 0.35, z - 1.4, z + 1.4, "timber_dark", "deck")
    for dz in (-0.55, 0.55):
        boxg(x0, x1, top + 0.35, top + 0.5, z + dz - 0.08, z + dz + 0.08, "iron", "rail")
    # The train: an engine and two cars, half again as large, in the gap between the left platform and the butte behind the fight.
    tx = 4.0
    y0 = top + 0.5
    moving("train", (tx, y0, z), scaled, (tx, y0, z), 1.6, train, tx, y0, z, rng)
    scaled((-30.0, FLOOR, z + 7.0), 1.4, water_tower, -30.0, z + 7.0)


def train(tx, y0, z, rng):
    """A little steam train of card: a banded boiler with its smokebox door, a sand dome and a steam dome, a flared stack, a cab with
    windows and an overhanging roof, a cowcatcher, spoked drive wheels with their rod; two boxcars, plank-scored, roofed, on bogies."""
    def lathe_x(prof, fam, name, x, y, segs=18):
        bm = bmesh.new()
        kit.lathe(bm, prof, segs, axis="x")
        for v in bm.verts:
            v.co = P(x + v.co.x, y + v.co.z, z - v.co.y)
        return ob(name, bm, fam, smooth=True)

    def lathe_up(prof, fam, name, x, y, segs=14):
        bm = bmesh.new()
        kit.lathe(bm, prof, segs)
        for v in bm.verts:
            v.co = P(x + v.co.x, y + v.co.z, z - v.co.y)
        return ob(name, bm, fam, smooth=True)

    cy = y0 + 1.35
    lathe_x([(0.75, 0.0), (0.75, 2.9)], "engine", "boiler", tx - 0.1, cy)
    lathe_x([(0.0, 0.0), (0.8, 0.0), (0.8, 0.35), (0.0, 0.35)], "engine_dark", "smokebox", tx + 2.8, cy)
    bm = bmesh.new()
    for xb in (tx + 0.5, tx + 1.5, tx + 2.5):
        kit.torus(bm, P(xb, cy, z), Vector((1.0, 0.0, 0.0)), 0.77, 0.06, segs=20, rsegs=4)
    kit.torus(bm, P(tx + 3.17, cy, z), Vector((1.0, 0.0, 0.0)), 0.45, 0.05, segs=18, rsegs=4)
    ob("boiler_bands", bm, "engine_band", smooth=True)
    lathe_up([(0.32, 0.0), (0.32, 0.22), (0.2, 0.42), (0.0, 0.48)], "engine_band", "steam_dome", tx + 1.2, cy + 0.68)
    lathe_up([(0.26, 0.0), (0.26, 0.16), (0.16, 0.32), (0.0, 0.36)], "engine_band", "sand_dome", tx + 0.3, cy + 0.68)
    lathe_up([(0.2, 0.0), (0.18, 0.45), (0.34, 0.9), (0.4, 1.02), (0.0, 1.02)], "engine_dark", "stack", tx + 2.6, cy + 0.6)
    boxg(tx - 1.7, tx - 0.1, y0 + 0.5, y0 + 2.7, z - 0.95, z + 0.95, "engine_cab", "cab")
    boxg(tx - 1.95, tx + 0.15, y0 + 2.7, y0 + 2.9, z - 1.15, z + 1.15, "engine_dark", "cab_roof")
    boxg(tx - 1.35, tx - 0.45, y0 + 1.7, y0 + 2.35, z + 0.95, z + 0.99, "window", "cab_window")
    boxg(tx - 0.1, tx + 3.15, y0 + 0.55, y0 + 0.72, z - 0.95, z + 0.95, "engine_dark", "running_board")
    card([(tx + 3.15, y0 + 0.62, z + 0.8), (tx + 3.95, y0 + 0.1, z + 0.8), (tx + 3.15, y0 + 0.1, z + 0.8)], "engine_cab", "cowcatcher", thick=1.6)
    bm = bmesh.new()
    for wx in (tx + 0.4, tx + 1.45, tx + 2.5):
        kit.torus(bm, P(wx, y0 + 0.55, z + 0.98), Vector((0.0, -1.0, 0.0)), 0.48, 0.08, segs=16, rsegs=4)
        for k in range(6):
            a = math.tau * k / 6
            kit.tube(bm, P(wx, y0 + 0.55, z + 0.98), P(wx + 0.46 * math.cos(a), y0 + 0.55 + 0.46 * math.sin(a), z + 0.98), 0.035, segs=4)
    kit.torus(bm, P(tx - 0.95, y0 + 0.45, z + 0.98), Vector((0.0, -1.0, 0.0)), 0.32, 0.07, segs=14, rsegs=4)
    ob("wheels", bm, "iron", smooth=True)
    bm = bmesh.new()
    kit.tube(bm, P(tx + 0.4, y0 + 0.55, z + 1.06), P(tx + 2.5, y0 + 0.55, z + 1.06), 0.05, segs=5)
    ob("rod", bm, "engine_band", smooth=True)
    for k in range(2):
        cx = tx - 2.2 - k * 4.0
        fam = "car_a" if k == 0 else "car_b"
        boxg(cx - 3.6, cx - 0.2, y0 + 0.65, y0 + 2.3, z - 0.85, z + 0.85, fam, "car%d" % k)
        boxg(cx - 3.75, cx - 0.05, y0 + 2.3, y0 + 2.48, z - 1.0, z + 1.0, "car_roof", "car_roof%d" % k)
        for j in range(1, 7):
            px = cx - 3.6 + j * 0.486
            boxg(px - 0.025, px + 0.025, y0 + 0.7, y0 + 2.25, z + 0.85, z + 0.88, "car_seam", "car_seam")
        boxg(cx - 2.35, cx - 1.45, y0 + 0.75, y0 + 2.15, z + 0.86, z + 0.92, "car_seam", "car_door")
        bm = bmesh.new()
        for wx in (cx - 3.1, cx - 2.5, cx - 1.3, cx - 0.7):
            kit.torus(bm, P(wx, y0 + 0.36, z + 0.9), Vector((0.0, -1.0, 0.0)), 0.26, 0.07, segs=12, rsegs=4)
        ob("car_wheels", bm, "iron", smooth=True)
    smoke_column((tx + 2.6, y0 + 3.2, z), 6, 0.55, rng)


def water_tower(wx, wz):
    """A water tower by the track: a rolled tank bound in iron hoops, a coiled cap, four legs braced in an X, a ladder up one side."""
    bm = bmesh.new()
    for (dx, dz) in ((-1.2, -1.2), (1.2, -1.2), (-1.2, 1.2), (1.2, 1.2)):
        tube(bm, (wx + dx, FLOOR, wz + dz), (wx + dx * 0.85, FLOOR + 6.0, wz + dz * 0.85), 0.16, segs=8)
    for (a, b) in (((-1.2, 1.2), (1.2, 1.2)), ((-1.2, -1.2), (-1.2, 1.2)), ((1.2, -1.2), (1.2, 1.2))):
        tube(bm, (wx + a[0], FLOOR + 0.6, wz + a[1]), (wx + b[0] * 0.87, FLOOR + 5.6, wz + b[1] * 0.87), 0.07, segs=5)
        tube(bm, (wx + b[0], FLOOR + 0.6, wz + b[1]), (wx + a[0] * 0.87, FLOOR + 5.6, wz + a[1] * 0.87), 0.07, segs=5)
    ob("tower_legs", bm, "timber", smooth=True)
    bm = bmesh.new()
    kit.lathe(bm, [(1.6, 6.0), (1.7, 6.2), (1.7, 9.0), (1.6, 9.1)], 22)
    for v in bm.verts:
        v.co = P(wx + v.co.x, FLOOR + v.co.z, wz - v.co.y)
    ob("tank", bm, "tank", smooth=True)
    bm = bmesh.new()
    for hy in (6.5, 7.6, 8.7):
        kit.torus(bm, P(wx, FLOOR + hy, wz), Vector((0.0, 0.0, 1.0)), 1.73, 0.06, segs=24, rsegs=4)
    ob("tank_hoops", bm, "iron", smooth=True)
    bm = bmesh.new()
    kit.coil_stack(bm, 0.0, 1.1, 1.9, 0.2, 6, 0.09, segs=20)
    for v in bm.verts:
        v.co = P(wx + v.co.x, FLOOR + 9.15 + v.co.z, wz - v.co.y)
    ob("tank_roof", bm, "tank_dark", smooth=True)
    bm = bmesh.new()
    lx = wx + 1.0
    for dz in (-0.25, 0.25):
        tube(bm, (lx, FLOOR + 0.2, wz + 1.85 + dz), (lx, FLOOR + 8.8, wz + 1.85 + dz), 0.04, segs=4)
    for k in range(17):
        hy = FLOOR + 0.6 + k * 0.5
        tube(bm, (lx, hy, wz + 1.6), (lx, hy, wz + 2.1), 0.03, segs=4)
    ob("ladder", bm, "iron", smooth=True)


def canyon_b():
    rng = random.Random(6)
    # The trail: a pale ribbon winding from the left foreground out to the horizon.
    pts = []
    n = 40
    for k in range(n + 1):
        t = k / n
        x = -70.0 + 110.0 * t + 18.0 * math.sin(t * 5.0)
        z = -85.0 - 120.0 * t
        pts.append(Vector((x, FLOOR + 0.06, z)))
    bm = bmesh.new()
    prev = None
    for k, p in enumerate(pts):
        d = (pts[min(k + 1, n)] - pts[max(k - 1, 0)]).normalized()
        side = Vector((-d.z, 0.0, d.x)) * (5.0 - 3.0 * k / n)
        a, b = bm.verts.new(P(*(p - side))), bm.verts.new(P(*(p + side)))
        if prev:
            bm.faces.new((prev[0], prev[1], b, a))
        prev = (a, b)
    ob("trail", bm, "trail", face=(0, 0, 1))
    # Telegraph poles along it, wires sagging between.
    bm = bmesh.new()
    wires = bmesh.new()
    poles = []
    for k in range(0, n + 1, 3):
        p = pts[k]
        d = (pts[min(k + 1, n)] - pts[max(k - 1, 0)]).normalized()
        q = p + Vector((-d.z, 0.0, d.x)) * 7.0
        tube(bm, (q.x, FLOOR, q.z), (q.x, FLOOR + 10.0, q.z), 0.24, segs=6)
        tube(bm, (q.x - 1.6, FLOOR + 9.4, q.z), (q.x + 1.6, FLOOR + 9.4, q.z), 0.14, segs=5)
        poles.append(q)
    ob("poles", bm, "timber", smooth=True)
    for a, b in zip(poles, poles[1:]):
        for dx in (-1.3, 1.3):
            last = None
            for s in range(7):
                t = s / 6
                p = a.lerp(b, t)
                y = FLOOR + 9.45 - math.sin(t * math.pi) * 1.0
                cur = (p.x + dx, y, p.z)
                if last:
                    kit.tube(wires, P(*last), P(*cur), 0.05, segs=4)
                last = cur
    kit.to_object("wires", wires, "iron", ROOT, smooth=True)
    # A covered wagon on the trail, twice and more life size to read at the floor's distance.
    p = pts[6]
    d = (pts[7] - pts[5]).normalized()
    scaled((p.x, FLOOR, p.z), 2.4, wagon, p, math.atan2(-d.z, d.x))


def wagon(p, ang):
    bed = turned([(-1.6, 0.6, -0.8), (1.6, 0.6, -0.8), (1.6, 1.3, -0.8), (-1.6, 1.3, -0.8)], (p.x, FLOOR, p.z), ang)
    card(bed, "timber", "wagon_side", thick=0.08)
    bed2 = turned([(-1.6, 0.6, 0.8), (1.6, 0.6, 0.8), (1.6, 1.3, 0.8), (-1.6, 1.3, 0.8)], (p.x, FLOOR, p.z), ang)
    card(bed2, "timber", "wagon_side", thick=0.08)
    bm = bmesh.new()
    rings = kit.lathe(bm, [(0.95, -1.4), (0.95, 1.4)], 16, axis="x")
    for v in list(bm.verts):
        if v.co.z < -0.05:
            bm.verts.remove(v)
    for v in bm.verts:
        g = turned([(v.co.x, 1.3 + v.co.z, -v.co.y)], (p.x, FLOOR, p.z), ang)[0]
        v.co = P(*g)
    ob("canvas", bm, "canvas", smooth=True)
    bm = bmesh.new()
    for (wx, wz) in ((-1.1, -0.9), (1.1, -0.9), (-1.1, 0.9), (1.1, 0.9)):
        g = turned([(wx, 0.55, wz)], (p.x, FLOOR, p.z), ang)[0]
        kit.torus(bm, P(*g), (math.sin(ang), math.cos(ang), 0.0), 0.5, 0.07, segs=12, rsegs=5)
    ob("wheels", bm, "timber_dark", smooth=True)


def canyon_c():
    rng = random.Random(8)
    # The windpump: a lattice tower, a bladed wheel and its vane, the tank at its foot.
    scaled((24.0, FLOOR, -92.0), 1.6, windpump, 24.0, -92.0)
    scaled((42.0, FLOOR, -98.0), 1.5, corral, 42.0, -98.0, rng)
    scaled((8.0, FLOOR, -100.0), 1.4, adobe, 8.0, -100.0, rng)


def windpump(wx, wz):
    bm = bmesh.new()
    for (dx, dz) in ((-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)):
        tube(bm, (wx + dx, FLOOR, wz + dz), (wx + dx * 0.2, FLOOR + 9.0, wz + dz * 0.2), 0.08, segs=6)
    for h in (2.5, 5.0, 7.2):
        f = 1.0 - h / 11.0
        for (a, b) in (((-1, -1), (1, -1)), ((1, -1), (1, 1)), ((1, 1), (-1, 1)), ((-1, 1), (-1, -1))):
            tube(bm, (wx + a[0] * f, FLOOR + h, wz + a[1] * f), (wx + b[0] * f, FLOOR + h, wz + b[1] * f), 0.05, segs=5)
    ob("pump_tower", bm, "iron", smooth=True)
    for k in range(12):
        a0 = math.tau * k / 12
        a1 = a0 + 0.2
        hub = (wx, FLOOR + 9.3, wz + 0.4)
        blade = [(hub[0] + 0.3 * math.cos(a0), hub[1] + 0.3 * math.sin(a0), hub[2]), (hub[0] + 1.8 * math.cos(a0), hub[1] + 1.8 * math.sin(a0), hub[2]),
                 (hub[0] + 1.8 * math.cos(a1), hub[1] + 1.8 * math.sin(a1), hub[2]), (hub[0] + 0.3 * math.cos(a1), hub[1] + 0.3 * math.sin(a1), hub[2])]
        card(blade, "blade", "blade", thick=0.03)
    card([(wx - 0.2, FLOOR + 9.0, wz - 0.2), (wx - 0.2, FLOOR + 9.6, wz - 0.2), (wx - 0.4, FLOOR + 9.9, wz - 2.6), (wx - 0.4, FLOOR + 8.8, wz - 2.6)],
         "blade", "vane", thick=0.03, normal=(1, 0, 0))
    bm = bmesh.new()
    kit.lathe(bm, [(1.6, 0.0), (1.6, 0.9)], 20)
    for v in bm.verts:
        v.co = P(wx - 3.0 + v.co.x, FLOOR + v.co.z, wz + 1.0 - v.co.y)
    ob("stock_tank", bm, "iron", smooth=True)


def corral(cx, cz, rng):
    # A corral of rolled rails and its trough.
    r = 6.0
    bm = bmesh.new()
    posts = [(cx + r * math.cos(math.tau * k / 12), cz + r * math.sin(math.tau * k / 12)) for k in range(12)]
    for (px, pz) in posts:
        tube(bm, (px, FLOOR, pz), (px, FLOOR + 1.6, pz), 0.08, segs=6)
    for a, b in zip(posts, posts[1:] + posts[:1]):
        for h in (0.6, 1.3):
            tube(bm, (a[0], FLOOR + h, a[1]), (b[0], FLOOR + h, b[1]), 0.05, segs=5)
    ob("corral", bm, "timber", smooth=True)
    boxg(cx - 1.6, cx + 1.6, FLOOR, FLOOR + 0.7, cz - 0.5, cz + 0.5, "timber_dark", "trough")


def adobe(hx, hz, rng):
    # An adobe house with a chimney's smoke.
    boxg(hx - 3.0, hx + 3.0, FLOOR, FLOOR + 3.0, hz - 2.2, hz + 2.2, "adobe", "house")
    boxg(hx - 3.2, hx + 3.2, FLOOR + 3.0, FLOOR + 3.3, hz - 2.4, hz + 2.4, "adobe_dark", "roof")
    boxg(hx - 0.5, hx + 0.5, FLOOR, FLOOR + 1.9, hz + 2.2, hz + 2.3, "timber_dark", "door")
    boxg(hx + 1.3, hx + 2.2, FLOOR + 1.3, FLOOR + 2.1, hz + 2.2, hz + 2.3, "window", "window")
    boxg(hx + 1.6, hx + 2.3, FLOOR + 3.3, FLOOR + 4.4, hz - 1.0, hz - 0.3, "adobe_dark", "chimney")
    smoke_column((hx + 1.95, FLOOR + 4.9, hz - 0.65), 5, 0.5, rng)


# ---- The meadow, the grove and the city (positions read from the match camera's depth, so things sit on what is really there) ----------


# (The near slope only: past (-25.8, -13.7, -33.4) the land dips behind a crest, and a track laid across it floated in the air.)
MEADOW_TRACK = [(-31.5, -19.2, -28.8), (-30.4, -18.3, -29.4), (-28.3, -16.3, -29.1), (-27.6, -15.5, -33.0), (-25.8, -13.7, -33.4)]


def hay_cart(at, angle):
    x, y, z = at
    bed = turned([(-1.2, 0.5, -0.6), (1.2, 0.5, -0.6), (1.2, 0.5, 0.6), (-1.2, 0.5, 0.6)], at, angle)
    card(bed, "timber", "cart_bed", thick=0.12, normal=(0, 1, 0))
    for side in (-0.6, 0.6):
        card(turned([(-1.2, 0.5, side), (1.2, 0.5, side), (1.2, 0.95, side), (-1.2, 0.95, side)], at, angle), "timber_dark", "cart_side", thick=0.06)
    bm = bmesh.new()
    for side in (-0.7, 0.7):
        g = turned([(0.0, 0.45, side)], at, angle)[0]
        kit.torus(bm, P(*g), (math.sin(angle), math.cos(angle), 0.0), 0.42, 0.07, segs=14, rsegs=5)
    ob("cart_wheels", bm, "timber_dark", smooth=True)
    # Its load: three rolled bales lying across the bed.
    bm = bmesh.new()
    for k in range(3):
        g0 = turned([(-0.75 + k * 0.75, 1.3, -0.55)], at, angle)[0]
        g1 = turned([(-0.75 + k * 0.75, 1.3, 0.55)], at, angle)[0]
        tube(bm, g0, g1, 0.38, segs=12)
    ob("hay", bm, "straw", smooth=True)
    tube_bm = bmesh.new()
    g0 = turned([(1.2, 0.6, 0.0)], at, angle)[0]
    g1 = turned([(2.4, 0.5, 0.0)], at, angle)[0]
    tube(tube_bm, g0, g1, 0.05, segs=5)
    ob("cart_shaft", tube_bm, "timber_dark", smooth=True)


def meadow_a():
    # A farm track worn up the slope from the valley, past the farm, toward the terrace; a hay cart on it.
    bm = bmesh.new()
    prev = None
    pts = [Vector(p) for p in MEADOW_TRACK]
    for k, p in enumerate(pts):
        d = (pts[min(k + 1, len(pts) - 1)] - pts[max(k - 1, 0)])
        d.y = 0.0
        d.normalize()
        side = Vector((-d.z, 0.0, d.x)) * 0.9
        a, b = bm.verts.new(P(*(p - side + Vector((0, 0.2, 0))))), bm.verts.new(P(*(p + side + Vector((0, 0.2, 0)))))
        if prev:
            bm.faces.new((prev[0], prev[1], b, a))
        prev = (a, b)
    ob("track", bm, "trail", face=(0, 0, 1))
    scaled((-28.0, -16.0, -31.0), 1.4, hay_cart, (-28.0, -16.0, -31.0), 0.9)


# A row on the far field left of the windmill, read from the match camera's depth: the field is level there, and in frame nothing stands
# over it (nearer, the stooks crowded the cottages and the frame's edge; to the right they showed under the platforms, by the fight).
MEADOW_FIELDS = [(-61.5, -13.2, -97.8), (-57.1, -13.5, -98.4), (-52.6, -13.8, -98.9)]


def haystack(at, r=1.6, h=3.2):
    """A rolled haystack, domed and a little uneven, bound by two straw ropes, a twist of straw on top; set into the ground a little so
    it sits on a slope. (A plain dome read as a mushroom cap.)"""
    x, y, z = at
    bm = bmesh.new()
    kit.lathe(bm, [(r * 0.95, -0.5), (r, 0.0), (r * 1.03, 0.28 * h), (r * 0.94, 0.55 * h), (r * 0.66, 0.8 * h), (r * 0.3, 0.95 * h),
                   (0.0, h)], 22, wobble=lambda a, k: 1.0 + 0.05 * math.sin(3.0 * a + k * 1.7))
    for v in bm.verts:
        v.co = P(x + v.co.x, y + v.co.z, z - v.co.y)
    ob("haystack", bm, "straw", smooth=True)
    bm = bmesh.new()
    for k, (hy, rr) in enumerate(((0.3, 1.02), (0.58, 0.92))):
        kit.torus(bm, P(x, y + hy * h, z), Vector((0.0, 0.0, 1.0)), r * rr, 0.09, segs=22, rsegs=5)
    ob("haystack_band", bm, "rope", smooth=True)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=7, radius1=0.28, radius2=0.0, depth=0.6, matrix=Matrix.Translation(P(x, y + h + 0.15, z)))
    ob("haystack_top", bm, "straw")
    # (The detail, on by default since the owner shipped it, 2026-10-22; `--detail 0` without.)
    if opts.get("detail", "1") == "1":
        # (The detail pass, 2026-10-22: straw strands combed down the stack as raised strips in a deeper straw, a frayed hem of wisps at
        # its foot and a few loose wisps on the crown.)
        rng = random.Random(int(abs(x) * 10 + abs(z)))
        bm = bmesh.new()
        for k in range(18):
            a = math.tau * k / 18 + rng.uniform(-0.08, 0.08)
            pts = []
            for t in (0.05, 0.3, 0.55, 0.78, 0.9):
                rr = {0.05: r * 1.0, 0.3: r * 1.03, 0.55: r * 0.95, 0.78: r * 0.7, 0.9: r * 0.42}[t] + 0.03
                pts.append((x + math.cos(a) * rr, y + t * h, z + math.sin(a) * rr))
            for i in range(len(pts) - 1):
                tube(bm, pts[i], pts[i + 1], 0.045, segs=4)
        ob("haystack_strands", bm, "straw_end", smooth=True)
        bm = bmesh.new()
        for k in range(26):
            a = math.tau * k / 26 + rng.uniform(-0.1, 0.1)
            ca, sa = math.cos(a), math.sin(a)
            w = 0.18
            base0 = (x + ca * r * 0.98 - sa * w, y + 0.18, z + sa * r * 0.98 + ca * w)
            base1 = (x + ca * r * 0.98 + sa * w, y + 0.18, z + sa * r * 0.98 - ca * w)
            tipp = (x + ca * (r + 0.35), y - 0.25 - rng.uniform(0, 0.2), z + sa * (r + 0.35))
            bm.faces.new([bm.verts.new(P(*q)) for q in (base0, base1, tipp)])
        for k in range(5):
            a = rng.uniform(0, math.tau)
            p0 = (x + math.cos(a) * 0.2, y + h * 0.97, z + math.sin(a) * 0.2)
            p1 = (x + math.cos(a) * 0.75, y + h * 1.05 + rng.uniform(0, 0.25), z + math.sin(a) * 0.75)
            bm.faces.new([bm.verts.new(P(*q)) for q in (p0, (p0[0] + 0.1, p0[1], p0[2]), p1)])
        ob("haystack_wisps", bm, "straw")


def meadow_b():
    # The fields at work: haystacks dotted over the far fields, life size (as the cottages are), so they shrink with the distance.
    for p in MEADOW_FIELDS:
        haystack((p[0], p[1] - 0.2, p[2]))


def scarecrow(at):
    x, y, z = at
    bm = bmesh.new()
    tube(bm, (x, y, z), (x, y + 2.4, z), 0.06, segs=6)
    tube(bm, (x - 0.9, y + 1.8, z), (x + 0.9, y + 1.8, z), 0.05, segs=6)
    ob("scarecrow_pole", bm, "timber_dark", smooth=True)
    card([(x - 0.45, y + 1.0, z + 0.1), (x + 0.45, y + 1.0, z + 0.1), (x + 0.55, y + 1.95, z + 0.1), (x - 0.55, y + 1.95, z + 0.1)], "coat_a", "shirt", thick=0.05)
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=1, radius=0.25, matrix=Matrix.Translation(P(x, y + 2.25, z + 0.1)))
    ob("scarecrow_head", bm, "canvas")
    bm = bmesh.new()
    kit.lathe(bm, [(0.45, 0.0), (0.2, 0.05), (0.15, 0.3), (0.0, 0.32)], 12)
    for v in bm.verts:
        v.co = P(x + v.co.x, y + 2.42 + v.co.z, z + 0.1 - v.co.y)
    ob("scarecrow_hat", bm, "straw")


_GROUND = {}


def ground_y(glb, x, z):
    """The height of the shipped ground model `glb` (godot/models/stages/) at (x, z), the game's axes, by a ray cast down onto it."""
    if glb not in _GROUND:
        from mathutils.bvhtree import BVHTree
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "godot", "models", "stages", glb))
        new = [o for o in bpy.data.objects if o not in before]
        verts, polys = [], []
        for o in new:
            if o.type == "MESH":
                base = len(verts)
                verts.extend(o.matrix_world @ v.co for v in o.data.vertices)
                polys.extend([base + i for i in p.vertices] for p in o.data.polygons)
        for o in new:
            bpy.data.objects.remove(o, do_unlink=True)
        _GROUND[glb] = BVHTree.FromPolygons(verts, polys)
    hit = _GROUND[glb].ray_cast(P(x, 200.0, z), Vector((0.0, 0.0, -1.0)), 400.0)
    return hit[0].z if hit[0] is not None else None


def meadow_c():
    rng = random.Random(12)
    # Chimney smoke over the farmhouse and the far cottage; washing on a line by the farm; a scarecrow in the near field.
    moving("smoke", (-28.0, -8.0, -36.5), scaled, (-28.0, -8.0, -36.5), 1.6, smoke, (-28.0, -8.0, -36.5), 6, 0.5, rng)
    moving("smoke", (42.5, -4.0, -72.5), scaled, (42.5, -4.0, -72.5), 2.4, smoke, (42.5, -4.0, -72.5), 6, 0.5, rng)
    # (The line and the scarecrow in the field right of the farmhouse, each on the land under it, the
    # believability pass, 2026-10-22: the line stood across the lane and the barn's front, the scarecrow before the farmhouse's door.)
    def on_land(x, z, old):
        y = ground_y("meadow_ground.glb", x, z)
        return (x, y - 0.1 if y is not None else old, z)
    a, b = on_land(-25.4, -36.0, -11.6), on_land(-23.0, -34.0, -13.6)
    bm = bmesh.new()
    for p in (a, b):
        tube(bm, p, (p[0], p[1] + 2.4, p[2]), 0.05, segs=6)
    tube(bm, (a[0], a[1] + 2.3, a[2]), (b[0], b[1] + 2.3, b[2]), 0.02, segs=4)
    ob("line", bm, "timber_dark", smooth=True)
    for k in range(4):
        t = 0.15 + k * 0.22
        x = a[0] + (b[0] - a[0]) * t
        y = a[1] + (b[1] - a[1]) * t + 2.3 - math.sin(t * math.pi) * 0.2
        zz = a[2] + (b[2] - a[2]) * t
        moving("flutter", (x, y, zz), card, [(x - 0.3, y, zz), (x + 0.3, y, zz), (x + 0.28, y - 0.7, zz), (x - 0.28, y - 0.7, zz)],
               ("flag_a", "canvas", "flag_c", "coat_c")[k], "washing", 0.02)
    sc = on_land(-17.5, -32.5, -15.5)
    moving("sway", sc, scaled, sc, 1.5, scarecrow, sc)


# Where the far terraces are, read from the match camera's depth: about a unit under the stage's base level, 100 to 120 units back.
GROVE_FAR = [(-49.0, -1.0, -106.0, 30.0), (-31.0, -1.1, -116.0, 26.0), (47.0, -1.0, -106.0, 31.0), (30.0, -1.0, -112.0, 25.0)]


def pleated_cone(cx, y_base, cz, R, h, n=11, turn=0.0, fam="crown"):
    """A pleated paper cone (a tier of a tree's crown): its rim folded in and out, the out-folds drooping a little."""
    bm = bmesh.new()
    apex = bm.verts.new(P(cx, y_base + h, cz))
    centre = bm.verts.new(P(cx, y_base + h * 0.12, cz))
    rim = []
    for i in range(2 * n):
        a = turn + math.pi * i / n
        r = R if i % 2 == 0 else R * 0.78
        y = y_base - (0.12 * R if i % 2 == 0 else 0.0)
        rim.append(bm.verts.new(P(cx + r * math.cos(a), y, cz + r * math.sin(a))))
    for i in range(2 * n):
        j = (i + 1) % (2 * n)
        bm.faces.new((apex, rim[i], rim[j]))
        bm.faces.new((centre, rim[j], rim[i]))
    return ob("far_crown", bm, fam)


# The detail pass (the owner, 2026-10-22: more detail on the background pieces, stage by stage), on by default; `--detail 0` without.
DETAIL = opts.get("detail", "1") == "1"


def far_tree(x, y0, z, height, r=2.0):
    """A great tree far off, as the grove's own: a trunk of folded paper (seven faces, flared at its foot) and a crown of pleated tiers."""
    bm = bmesh.new()
    kit.lathe(bm, [(r * 1.7, -3.0), (r * 1.25, 0.6), (r, height * 0.35), (r * 0.8, height)], 7)
    for v in bm.verts:
        v.co = P(x + v.co.x, y0 + v.co.z, z - v.co.y)
    ob("far_trunk", bm, "bark")
    if DETAIL:
        # Roots flaring into the terrace and a few broken branch stubs up the trunk.
        bm = bmesh.new()
        for k in range(6):
            a = math.tau * k / 6 + 0.3
            ca, sa = math.cos(a), math.sin(a)
            side = (-sa * 0.7, ca * 0.7)
            top = (x + ca * r * 1.2, y0 + 2.6, z + sa * r * 1.2)
            foot = (x + ca * r * 2.6, y0 - 0.6, z + sa * r * 2.6)
            for q in ((top, foot, (foot[0] + side[0], foot[1], foot[2] + side[1])), (top, (foot[0] + side[0], foot[1], foot[2] + side[1]), (top[0] + side[0] * 0.5, top[1], top[2] + side[1] * 0.5))):
                bm.faces.new([bm.verts.new(P(*v)) for v in q])
        for k, (hy, a) in enumerate(((0.38, 0.6), (0.5, 2.9), (0.62, 4.6))):
            p0 = (x + math.cos(a) * r * 0.8, y0 + height * hy, z + math.sin(a) * r * 0.8)
            p1 = (x + math.cos(a) * r * 2.0, y0 + height * hy + 1.4, z + math.sin(a) * r * 2.0)
            tube(bm, p0, p1, 0.35, segs=6)
        ob("far_roots", bm, "bark", smooth=False)
    for k, (base, rad, tall) in enumerate(((height * 0.5, r * 4.4, 5.0), (height * 0.5 + 3.8, r * 3.6, 4.6), (height * 0.5 + 7.2, r * 2.6, 4.4),
                                           (height * 0.5 + 10.2, r * 1.6, 4.0))):
        pleated_cone(x, y0 + base, z, rad, tall, turn=k * 0.3, fam="crown" if k % 2 == 0 else "crown_lit")


def far_hut(x, y, z, r, side, ground=None):
    """A hut of dark wood on a ring of planks round a far trunk: a lit window, a cone of thatch."""
    bm = bmesh.new()
    kit.lathe(bm, [(r + 1.8, 0.0), (r + 1.8, 0.3)], 18)
    for v in bm.verts:
        v.co = P(x + v.co.x, y + v.co.z, z - v.co.y)
    ob("far_deck", bm, "timber")
    hx = x + side * (r + 0.6)
    hz = z + 1.4
    # (Dark wood, as the reference's huts: on pale walls the warm window had nothing to glow against.)
    boxg(hx - 1.1, hx + 1.1, y + 0.3, y + 2.1, hz - 0.9, hz + 0.9, "timber_dark", "far_hut")
    bm = bmesh.new()
    kit.lathe(bm, [(1.6, 0.0), (1.0, 0.5), (0.0, 1.4)], 10)
    for v in bm.verts:
        v.co = P(hx + v.co.x, y + 2.1 + v.co.z, hz - v.co.y)
    ob("far_thatch", bm, "straw")
    boxg(hx - 0.5, hx + 0.5, y + 0.8, y + 1.7, hz + 0.9, hz + 0.96, "window", "far_window")
    if DETAIL:
        f = hz + 0.9
        # Plank battens up the front, a framed window with shutters, a door, a second fringe of thatch.
        for bx in (hx - 1.0, hx - 0.65, hx + 0.65, hx + 1.0):
            boxg(bx - 0.05, bx + 0.05, y + 0.3, y + 2.1, f, f + 0.06, "timber", "far_batten")
        for (x0, x1, y0_, y1_) in ((hx - 0.6, hx + 0.6, y + 0.72, y + 0.8), (hx - 0.6, hx + 0.6, y + 1.7, y + 1.78)):
            boxg(x0, x1, y0_, y1_, f, f + 0.1, "timber", "far_frame")
        for sx in (-1.0, 1.0):
            boxg(hx + sx * 0.55, hx + sx * 0.85, y + 0.78, y + 1.72, f + 0.02, f + 0.08, "gate_red", "far_shutter")
        bm = bmesh.new()
        kit.lathe(bm, [(1.85, -0.15), (1.55, 0.25)], 14)
        for v in bm.verts:
            v.co = P(hx + v.co.x, y + 2.1 + v.co.z, hz - v.co.y)
        ob("far_fringe", bm, "straw_end")
        # A rail of posts and rope round the deck, a lantern on it, a ladder down the trunk to the terrace.
        bm = bmesh.new()
        n = 14
        rail = []
        for k in range(n):
            a = math.tau * k / n
            px, pz = x + math.cos(a) * (r + 1.7), z + math.sin(a) * (r + 1.7)
            tube(bm, (px, y + 0.3, pz), (px, y + 1.2, pz), 0.06, segs=4)
            rail.append((px, y + 1.15, pz))
        for k in range(n):
            tube(bm, rail[k], rail[(k + 1) % n], 0.035, segs=4)
        ob("far_rail", bm, "rope", smooth=True)
        lx, lz = x - side * (r + 1.7), z + 0.8
        boxg(lx - 0.2, lx + 0.2, y + 1.2, y + 1.6, lz - 0.2, lz + 0.2, "window", "far_lamp")
        if ground is not None:
            ax = x + side * 0.2
            az = z + r + 0.35
            bm = bmesh.new()
            for dx in (-0.35, 0.35):
                tube(bm, (ax + dx, ground, az + 0.3), (ax + dx, y + 0.3, az), 0.06, segs=4)
            k = 0.0
            while k < (y - ground):
                t = k / max(y - ground, 0.01)
                tube(bm, (ax - 0.35, ground + k, az + 0.3 * (1 - t)), (ax + 0.35, ground + k, az + 0.3 * (1 - t)), 0.045, segs=4)
                k += 0.55
            ob("far_ladder", bm, "timber", smooth=True)


def rope_bridge(a, b, sag=1.2):
    """A rope bridge between two decks (game points): plank cards along the sag, two hand ropes over them."""
    bm = bmesh.new()
    n = 14
    for k in range(n):
        t = (k + 0.5) / n
        p = a.lerp(b, t) + Vector((0.0, -sag * math.sin(t * math.pi), 0.0))
        d = (b - a).normalized()
        side = Vector((-d.z, 0.0, d.x)) * 0.6
        q0, q1 = p - side, p + side
        boxg(min(q0.x, q1.x) - 0.25, max(q0.x, q1.x) + 0.25, p.y - 0.08, p.y + 0.08, min(q0.z, q1.z), max(q0.z, q1.z), "timber", "plank")
    for h in (0.0, 1.0):
        last = None
        for k in range(n + 1):
            t = k / n
            p = a.lerp(b, t) + Vector((0.0, 1.0 - sag * 0.8 * math.sin(t * math.pi), 0.0))
            cur = (p.x, p.y, p.z + (0.6 if h else -0.6))
            if last:
                kit.tube(bm, P(*last), P(*cur), 0.05, segs=4)
            last = cur
    kit.to_object("bridge_ropes", bm, "rope", ROOT, smooth=True)


def grove_a():
    # A tree village far off, as in the reference's violet forest: great trees on the far terraces either side of the island, small
    # huts with lit windows on rings of planks round their trunks, rope bridges between them. True size, so the distance reads.
    huts = []
    for k, (x, y0, z, height) in enumerate(GROVE_FAR):
        far_tree(x, y0, z, height)
        side = -1.0 if x < 0 else 1.0
        # (Low on the trunks, against the violet hills where the lit windows show: higher, they were lost among the lantern strings.)
        y = y0 + (4.0 if k % 2 == 0 else 5.5)
        far_hut(x, y, z, 2.0, side if k < 2 else -side, ground=y0)
        huts.append(Vector((x, y + 0.3, z)))
    for i, j in ((0, 1), (2, 3)):
        a, b = huts[i], huts[j]
        d = (b - a)
        d.y = 0.0
        d.normalize()
        rope_bridge(a + d * 3.8, b - d * 3.8)


def grove_b():
    # A festival gate on the left terrace, red posts and two crossbars, and paper streamers hanging from the lantern strings.
    def gate(x, y, z):
        bm = bmesh.new()
        for dx in (-1.6, 1.6):
            tube(bm, (x + dx, y, z), (x + dx * 0.9, y + 4.2, z), 0.18, segs=12)
        tube(bm, (x - 2.3, y + 3.35, z), (x + 2.3, y + 3.35, z), 0.13, segs=10)
        ob("gate_posts", bm, "gate_red", smooth=True)
        for dx in (-1.6, 1.6):
            boxg(x + dx - 0.3, x + dx + 0.3, y - 0.2, y + 0.35, z - 0.3, z + 0.3, "gate_dark", "gate_base")
        top = [(x - 2.9, y + 4.1, z), (x + 2.9, y + 4.1, z), (x + 3.35, y + 4.55, z), (x + 3.0, y + 4.48, z), (x + 2.6, y + 4.42, z),
               (x - 2.6, y + 4.42, z), (x - 3.0, y + 4.48, z), (x - 3.35, y + 4.55, z)]
        card([(p[0], p[1], p[2] + 0.25) for p in top], "gate_dark", "gate_top", thick=0.5)
        card([(x - 2.6, y + 3.95, z + 0.2), (x + 2.6, y + 3.95, z + 0.2), (x + 2.6, y + 4.12, z + 0.2), (x - 2.6, y + 4.12, z + 0.2)], "gate_red", "gate_lintel", thick=0.4)
        card([(x - 0.35, y + 3.45, z + 0.22), (x + 0.35, y + 3.45, z + 0.22), (x + 0.35, y + 3.98, z + 0.22), (x - 0.35, y + 3.98, z + 0.22)], "gate_dark", "gate_plaque", thick=0.1)
        if DETAIL:
            # A twisted rope sagging under the crossbar with paper zigzags hanging from it, and stone plinths under the posts.
            bm = bmesh.new()
            last = None
            for k in range(13):
                t = k / 12.0
                cur = (x - 1.45 + 2.9 * t, y + 3.05 - 0.35 * math.sin(t * math.pi), z + 0.25)
                if last:
                    tube(bm, last, cur, 0.09, segs=6)
                last = cur
            ob("gate_rope", bm, "rope", smooth=True)
            for t in (0.25, 0.5, 0.75):
                cx = x - 1.45 + 2.9 * t
                cy = y + 3.05 - 0.35 * math.sin(t * math.pi)
                zig = [(cx - 0.12, cy), (cx + 0.12, cy), (cx + 0.12, cy - 0.25), (cx - 0.04, cy - 0.25), (cx + 0.1, cy - 0.5), (cx - 0.08, cy - 0.5),
                       (cx + 0.06, cy - 0.78), (cx - 0.14, cy - 0.78), (cx - 0.02, cy - 0.5), (cx - 0.16, cy - 0.5), (cx - 0.02, cy - 0.25), (cx - 0.12, cy - 0.25)]
                card([(px, py, z + 0.32) for (px, py) in zig], "canvas", "gate_shide", thick=0.02)
            for dx in (-1.6, 1.6):
                boxg(x + dx - 0.45, x + dx + 0.45, y - 0.35, y + 0.15, z - 0.45, z + 0.45, "rock", "gate_plinth")
    # (Beside the side trees, toward the stage, the believability pass, 2026-10-22: set where the trees stand, each trunk grew through its
    # gate.)
    scaled((-19.0, -9.1, -53.0), 2.0, gate, -19.0, -9.1, -53.0)
    scaled((21.0, -9.1, -56.0), 2.0, gate, 21.0, -9.1, -56.0)
    for sgn in (-1.0, 1.0):
        a = Vector((4.0 * sgn, 12.0, -17.0))
        b = Vector((22.0 * sgn, 8.0, -30.0))
        for k in range(1, 9):
            f = k / 9.0
            p = a.lerp(b, f) + Vector((0.0, -1.2 * math.sin(f * math.pi), 0.0))
            ln = 1.2 + 0.5 * (k % 3)
            moving("flutter", tuple(p), card, [(p.x - 0.12, p.y, p.z), (p.x + 0.12, p.y, p.z), (p.x + 0.1, p.y - ln, p.z), (p.x, p.y - ln - 0.2, p.z),
                   (p.x - 0.1, p.y - ln, p.z)], ("flag_a", "flag_b", "flag_c")[k % 3], "streamer", 0.01)


def islet(x, y, z, s):
    bm = bmesh.new()
    kit.lathe(bm, [(0.0, -2.2), (0.8, -1.6), (1.8, -0.6), (2.2, 0.0)], 10)
    for v in bm.verts:
        v.co = P(x + v.co.x * s, y + v.co.z * s, z - v.co.y * s)
    ob("islet_rock", bm, "rock")
    card([(x + 2.2 * s * math.cos(math.tau * i / 10), y, z + 2.2 * s * math.sin(math.tau * i / 10)) for i in range(10)], "turf", "islet_top",
         thick=0.15, normal=(0, 1, 0))
    bm = bmesh.new()
    tube(bm, (x, y, z), (x, y + 2.2 * s, z), 0.12 * s, segs=6)
    ob("islet_trunk", bm, "bark", smooth=True)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=7, radius1=1.2 * s, radius2=0.0, depth=2.6 * s, matrix=Matrix.Translation(P(x, y + 3.2 * s, z)))
    ob("islet_crown", bm, "turf")
    bm = bmesh.new()
    kit.lathe(bm, [(0.05, 0.0), (0.22, 0.12), (0.26, 0.3), (0.22, 0.48), (0.05, 0.6)], 10)
    for v in bm.verts:
        v.co = P(x + 1.4 * s + v.co.x * s, y + 1.2 * s + v.co.z * s, z - v.co.y * s)
    ob("islet_lantern", bm, "window")


def grove_c():
    # Small floating islets drifting off either side, each with a little tree and a lantern: the great tree's island is not alone.
    islet(30.0, 9.0, -62.0, 2.0)
    islet(-36.0, 13.0, -78.0, 2.4)
    islet(48.0, 16.0, -95.0, 2.6)


def bus(x, y, z, d):
    """A town bus of card: a body rounded at its edges, a stripe, a band of lit windows, a windscreen and a lit sign, bumpers, wheels with
    hubs."""
    r = 0.38
    y0, y1, z0, z1 = y + 0.3, y + 2.6, z - 0.95, z + 0.95
    prof = []
    for (cy, cz, a0) in ((y1 - r, z1 - r, 0.0), (y1 - r, z0 + r, 90.0), (y0 + r, z0 + r, 180.0), (y0 + r, z1 - r, 270.0)):
        for k in range(5):
            a = math.radians(a0 + 90.0 * k / 4)
            prof.append((x + 3.0, cy + r * math.sin(a), cz + r * math.cos(a)))
    card(prof, "bus", "bus", thick=6.0, normal=(1.0, 0.0, 0.0))
    boxg(x - 2.95, x + 2.95, y + 0.95, y + 1.15, z1 - 0.01, z1 + 0.03, "bus_stripe", "bus_stripe")
    for k in range(5):
        wx = x - 2.35 + k * 1.1
        boxg(wx - 0.42, wx + 0.42, y + 1.45, y + 2.2, z1 - 0.01, z1 + 0.04, "window", "bus_window")
    boxg(x + 2.99, x + 3.04, y + 1.4, y + 2.25, z0 + 0.25, z1 - 0.25, "window", "windscreen")
    boxg(x + 2.99, x + 3.05, y + 2.3, y + 2.5, z0 + 0.4, z1 - 0.4, "window", "bus_sign")
    for bx in (x - 3.08, x + 3.0):
        boxg(bx, bx + 0.08, y + 0.35, y + 0.6, z0 + 0.05, z1 - 0.05, "engine_dark", "bumper")
    bm = bmesh.new()
    for wx in (x - 2.0, x + 2.0):
        kit.torus(bm, P(wx, y + 0.4, z1 + 0.02), (0.0, -1.0, 0.0), 0.32, 0.13, segs=14, rsegs=5)
    ob("bus_wheels", bm, "engine", smooth=True)
    bm = bmesh.new()
    for wx in (x - 2.0, x + 2.0):
        kit.torus(bm, P(wx, y + 0.4, z1 + 0.06), (0.0, -1.0, 0.0), 0.09, 0.08, segs=10, rsegs=4)
    ob("bus_hubs", bm, "iron", smooth=True)


def city_a():
    # The street busier: a bus on the right, people crossing on the left.
    moving("drive", (20.5, -13.5, -25.0), scaled, (20.5, -13.5, -25.0), 1.2, bus, 20.5, -13.5, -25.0, 1)
    rng = random.Random(14)
    for k in range(5):
        at = (-24.0 + k * 1.2 + rng.uniform(-0.2, 0.2), -13.5, -22.0 - rng.uniform(0, 2.5))
        moving("stroll", at, figure, at, 1.7, rng.choice(("coat_a", "coat_b", "coat_c")), rng)


def city_b():
    # A lit billboard on the near tower at the right, and a party on the rooftop at the left: string lights and people.
    def billboard(x, y, z):
        # A framed board, its poster cut-paper shapes (a sun over waves, a line of lettering) lit for the night.
        boxg(x - 4.0, x + 4.0, y, y + 4.0, z - 0.35, z - 0.1, "board_back", "board")
        for (x0, x1, y0, y1) in ((x - 4.2, x + 4.2, y - 0.2, y + 0.2), (x - 4.2, x + 4.2, y + 3.8, y + 4.2), (x - 4.2, x - 3.8, y, y + 4.0),
                                 (x + 3.8, x + 4.2, y, y + 4.0)):
            boxg(x0, x1, y0, y1, z - 0.15, z + 0.2, "board_frame", "board_frame")
        card([(x - 3.8, y + 0.2, z - 0.08), (x + 3.8, y + 0.2, z - 0.08), (x + 3.8, y + 3.8, z - 0.08), (x - 3.8, y + 3.8, z - 0.08)], "poster", "poster", thick=0.02)
        card(scallop(x - 1.9, y + 2.3, z - 0.05, 1.05, 1.05, 0, 0.0, depth=0.0, n=24), "poster_a", "poster_sun", thick=0.02)
        wave = [(x - 3.8 + 4.2 * k / 16, y + 1.2 + 0.25 * math.sin(k * 0.9), z - 0.03) for k in range(17)]
        card(wave + [(x + 0.4, y + 0.2, z - 0.03), (x - 3.8, y + 0.2, z - 0.03)], "poster_b", "poster_wave", thick=0.02)
        for (yy, w) in ((y + 2.7, 2.6), (y + 2.05, 2.1), (y + 1.4, 2.4)):
            card([(x + 0.7, yy, z - 0.03), (x + 0.7 + w, yy, z - 0.03), (x + 0.7 + w, yy + 0.38, z - 0.03), (x + 0.7, yy + 0.38, z - 0.03)],
                 "poster_c", "poster_text", thick=0.02)
        # The scaffold: two legs braced in an X, a catwalk with its rail along the board's foot, three hood lamps leaning over it.
        bm = bmesh.new()
        for dx in (-2.5, 2.5):
            tube(bm, (x + dx, y - 3.0, z - 0.4), (x + dx, y, z - 0.4), 0.12, segs=6)
        tube(bm, (x - 2.5, y - 2.8, z - 0.4), (x + 2.5, y - 0.2, z - 0.4), 0.06, segs=5)
        tube(bm, (x + 2.5, y - 2.8, z - 0.4), (x - 2.5, y - 0.2, z - 0.4), 0.06, segs=5)
        for k in range(7):
            px = x - 4.0 + k * 8.0 / 6
            tube(bm, (px, y - 0.22, z + 0.85), (px, y + 0.5, z + 0.85), 0.035, segs=4)
        tube(bm, (x - 4.0, y + 0.5, z + 0.85), (x + 4.0, y + 0.5, z + 0.85), 0.035, segs=4)
        for lx in (x - 2.6, x, x + 2.6):
            tube(bm, (lx, y + 4.2, z - 0.25), (lx, y + 4.65, z + 0.85), 0.05, segs=5)
        ob("board_legs", bm, "iron", smooth=True)
        boxg(x - 4.2, x + 4.2, y - 0.32, y - 0.2, z - 0.3, z + 0.95, "steel", "catwalk")
        for lx in (x - 2.6, x, x + 2.6):
            boxg(lx - 0.35, lx + 0.35, y + 4.55, y + 4.75, z + 0.65, z + 1.05, "engine_dark", "lamp_hood")
            boxg(lx - 0.28, lx + 0.28, y + 4.5, y + 4.56, z + 0.7, z + 1.0, "window", "lamp_light")
    scaled((28.0, 4.0, -50.0), 1.4, billboard, 28.0, 4.0, -50.0)
    rng = random.Random(16)
    x0, y0, z0 = -24.0, 2.5, -50.0
    bm = bmesh.new()
    for dx in (-4.0, 4.0):
        tube(bm, (x0 + dx, y0, z0), (x0 + dx, y0 + 3.0, z0), 0.07, segs=5)
    ob("party_poles", bm, "iron", smooth=True)
    for k in range(10):
        t = k / 9.0
        x = x0 - 4.0 + 8.0 * t
        y = y0 + 2.9 - math.sin(t * math.pi) * 0.6
        moving("twinkle", (x, y - 0.15, z0), bulb, (x, y - 0.15, z0))
    for k in range(5):
        at = (x0 - 3.0 + k * 1.5 + rng.uniform(-0.2, 0.2), y0, z0 + 0.3)
        moving("dance", at, figure, at, 1.6, rng.choice(("coat_a", "coat_b", "coat_c")), rng)


def bulb(at, r=0.16, fam="window"):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=1, radius=r, matrix=Matrix.Translation(P(*at)))
    ob("bulb", bm, fam)


def city_c():
    # An elevated railway across the city behind the stage: a steel viaduct over the street, a lit train on it.
    z = -38.0
    deck = -6.0
    bm = bmesh.new()
    x = -110.0
    while x <= 110.0:
        for dz in (-1.2, 1.2):
            tube(bm, (x, -13.5, z + dz), (x, deck, z + dz), 0.22, segs=8)
        x += 9.0
    ob("viaduct_piers", bm, "iron", smooth=True)
    boxg(-110.0, 110.0, deck - 0.8, deck, z - 1.6, z + 1.6, "steel", "viaduct_deck")
    tx = -12.0
    for k in range(3):
        cx = tx + k * 5.2
        boxg(cx - 2.4, cx + 2.4, deck + 0.1, deck + 2.3, z - 1.0, z + 1.0, "train", "car%d" % k)
        boxg(cx - 2.1, cx + 2.1, deck + 1.0, deck + 1.7, z + 1.0, z + 1.05, "window", "car_windows%d" % k)


def grove_l():
    # Fireflies round the treehouse and under the lantern strings, high over the fight and behind it.
    rng = random.Random(21)
    for k in range(16):
        side = -1.0 if k % 2 else 1.0
        at = (side * rng.uniform(3.0, 20.0), rng.uniform(7.0, 13.0), rng.uniform(-28.0, -12.0))
        moving("firefly", at, bulb, at, 0.16, "firefly")


BUILDS = {("ocean", "A"): beach_a, ("ocean", "B"): beach_b, ("ocean", "C"): beach_c,
          ("sunset", "A"): canyon_a, ("sunset", "B"): canyon_b, ("sunset", "C"): canyon_c,
          ("meadow", "A"): meadow_a, ("meadow", "B"): meadow_b, ("meadow", "C"): meadow_c,
          ("grove", "A"): grove_a, ("grove", "B"): grove_b, ("grove", "C"): grove_c,
          ("city", "A"): city_a, ("city", "B"): city_b, ("city", "C"): city_c,
          ("grove", "L"): grove_l}
# The ground the occlusion bake lays under each stage's pieces.
FLOORS = {"ocean": SEA, "sunset": FLOOR, "meadow": -20.0, "grove": -12.0, "city": -13.5}


def main():
    global ROOT
    bpy.ops.wm.read_factory_settings(use_empty=True)
    ROOT = bpy.data.objects.new("World", None)
    bpy.context.scene.collection.objects.link(ROOT)
    # (Options combine: `--variant AB` builds both.)
    for v in VARIANT:
        if (THEME, v) in BUILDS:
            BUILDS[(THEME, v)]()
    group_movers()
    # (The still pieces joined by material: the detail pass's battens, rails and frames, each its own mesh, could overflow the game's
    # per-instance shader buffer, as the house kit's did.)
    groups = {}
    for o in [o for o in bpy.data.objects if o.parent == ROOT and o.type == "MESH"]:
        # (And by depth, in bands 25 deep: the game lays the air of its depth over each mesh.)
        depth = sum((o.matrix_world @ Vector(c)).y for c in o.bound_box) / 8.0
        groups.setdefault((o.data.materials[0].name if o.data.materials else "", round(depth / 25.0)), []).append(o)
    for mat, objs in groups.items():
        if len(objs) > 1:
            bpy.ops.object.select_all(action="DESELECT")
            for o in objs:
                o.select_set(True)
            bpy.context.view_layer.objects.active = objs[0]
            bpy.ops.object.join()
    # (The water or the desert floor under them, for the ground's shade at their feet.)
    kit.bake_occlusion(0.6, floor_z=FLOORS[THEME], floor_at=(0.0, 100.0), floor_size=600.0)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("world %s %s: %d triangles -> %s" % (THEME, VARIANT, tris, out))


main()
