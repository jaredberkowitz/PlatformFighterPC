"""World-building mocks (docs/art/stages/WORLD.md, the owner's go-ahead 2026-10-21): what makes a stage a place, in the paper language
(rolled tubes, scored card, cut-outs), built in world space for the exact stage and placed by the stage demo's `--world-mock=<glb>`.
  ocean (the beach, a fishing harbour)
    A  a working pier: a fishing boat moored along the far boardwalk with lines to its posts, nets hung on its rail, fish crates;
    B  harbour life: gulls perched on posts and the lamp and gliding over the water, a rowboat tied off the dock's left end;
    C  the town at work: market awnings and townsfolk on the quay, bunting, chimney smoke.
  sunset (the canyon, a desert crossing at dusk)
    A  a railroad: a timber trestle across the floor behind the buttes with a little train on it and a water tower by the track;
    B  a wagon trail: a pale trail winding across the floor to the horizon, telegraph poles along it, a covered wagon on it;
    C  a ranch: a windpump and its tank, a corral with cattle, an adobe house with chimney smoke.
Every part names its family (painted by the game, `StageBake.place_world`), with its occlusion baked.

Run (Blender 5.2):
  blender --background --python art/blender/make_world.py -- --theme ocean --variant A --out <file.glb>   (`--variant AB` for both)
Shipped (the owner's picks, 2026-10-21): godot/models/stages/beach_world.glb (`--theme ocean --variant AB`), canyon_world.glb
(`--theme sunset --variant A`).
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


def turned(pts, centre, angle):
    """`pts` [(x, y, z) about the origin] turned `angle` radians about the vertical and moved to `centre`."""
    c, s = math.cos(angle), math.sin(angle)
    return [(centre[0] + x * c + z * s, centre[1] + y, centre[2] - x * s + z * c) for (x, y, z) in pts]


# ---- Shared pieces -------------------------------------------------------------------------------------------------------------------


def smoke(at, n=5, size=0.6, rng=None):
    """Chimney smoke as rolled paper puffs rising and drifting, smaller and paler up the column."""
    rng = rng or random.Random(5)
    bm = bmesh.new()
    for k in range(n):
        t = k / max(1, n - 1)
        c = Vector(at) + Vector((t * 1.6 + rng.uniform(-0.2, 0.2), k * size * 1.3, 0.0))
        r = size * (0.7 + 0.6 * t)
        bmesh.ops.create_icosphere(bm, subdivisions=1, radius=r, matrix=Matrix.Translation(P(*c)))
    ob("smoke", bm, "smoke")


def figure(at, height, fam, rng, turn=0.0):
    """A townsperson as a paper standee: a rounded body card, a head disc, facing the camera."""
    x, y, z = at
    h = height
    body = [(x - 0.22 * h, y, z), (x + 0.22 * h, y, z), (x + 0.18 * h, y + 0.62 * h, z), (x - 0.18 * h, y + 0.62 * h, z)]
    card(body, fam, "figure")
    head = [(x + 0.13 * h * math.cos(math.tau * i / 10), y + 0.8 * h + 0.13 * h * math.sin(math.tau * i / 10), z + 0.01) for i in range(10)]
    card(head, "skin", "head")


def gull(at, span, glide=True, turn=0.0):
    """A gull cut from card: a white body, wings in a shallow V (gliding) or folded (perched), grey tips, a yellow beak."""
    x, y, z = at
    s = span
    d = 1.0 if math.cos(turn) >= 0 else -1.0
    body = [(x - 0.3 * s * d, y, z), (x + 0.25 * s * d, y + 0.03 * s, z), (x + 0.32 * s * d, y + 0.08 * s, z),
            (x + 0.2 * s * d, y + 0.12 * s, z), (x - 0.25 * s * d, y + 0.08 * s, z)]
    card(body, "gull", "gull_body", thick=0.02)
    card([(x + 0.32 * s * d, y + 0.08 * s, z + 0.01), (x + 0.42 * s * d, y + 0.07 * s, z + 0.01), (x + 0.32 * s * d, y + 0.05 * s, z + 0.01)],
         "beak", "gull_beak", thick=0.01)
    if glide:
        for side in (-1.0, 1.0):
            w = [(x - 0.05 * s, y + 0.08 * s, z), (x + 0.12 * s, y + 0.1 * s, z), (x + 0.02 * s, y + 0.28 * s, z + side * 0.5 * s),
                 (x - 0.12 * s, y + 0.3 * s, z + side * 0.5 * s)]
            card(w, "gull", "gull_wing", thick=0.015, normal=(0, 1, 0))
            tip = [(x + 0.02 * s, y + 0.28 * s, z + side * 0.5 * s), (x - 0.12 * s, y + 0.3 * s, z + side * 0.5 * s),
                   (x - 0.08 * s, y + 0.33 * s, z + side * 0.62 * s)]
            card(tip, "gull_tip", "gull_tip", thick=0.015, normal=(0, 1, 0))
    else:
        card([(x - 0.22 * s * d, y + 0.05 * s, z + 0.02), (x + 0.15 * s * d, y + 0.1 * s, z + 0.02), (x - 0.28 * s * d, y + 0.12 * s, z + 0.02)],
             "gull_tip", "gull_wing", thick=0.01)
        bm = bmesh.new()
        tube(bm, (x, y - 0.12 * s, z), (x, y, z), 0.012 * s, segs=5)
        ob("gull_leg", bm, "beak")


# ---- The beach -----------------------------------------------------------------------------------------------------------------------


# The far boardwalk, from the back of the dock out to the fisherman's jetty (StageArt._dock_harbour): its deck.
WALK_A = Vector((9.0, -0.1, -6.2))
WALK_B = Vector((28.4, SEA + 4.1, -45.0))


def walk_at(t):
    return WALK_A.lerp(WALK_B, t)


def fishing_boat(at, angle):
    """A small fishing boat of scored card: a hull with a raked bow, a wheelhouse, a mast with a furled sail and a boom."""
    hull = [(-2.6, 0.0, 0.0), (2.2, 0.0, 0.0), (3.0, 1.2, 0.0), (-2.8, 1.1, 0.0)]
    for side in (-0.8, 0.8):
        card(turned([(x, y + at[1] - 0.4, side) for (x, y, _z) in hull], (at[0], 0.0, at[2]), angle), "hull", "hull_side", thick=0.06,
             normal=(math.sin(angle) * side, 0, math.cos(angle) * side))
    deck = [(-2.7, 0.9, -0.8), (2.6, 0.9, -0.8), (2.95, 0.9, 0.0), (2.6, 0.9, 0.8), (-2.7, 0.9, 0.8)]
    card(turned([(x, y + at[1] - 0.4, z) for (x, y, z) in deck], (at[0], 0.0, at[2]), angle), "deck", "deck", thick=0.08, normal=(0, 1, 0))
    stripe = [(-2.75, 0.55, 0.0), (2.6, 0.55, 0.0), (2.8, 0.8, 0.0), (-2.78, 0.8, 0.0)]
    for side in (-0.82, 0.82):
        card(turned([(x, y + at[1] - 0.4, side) for (x, y, _z) in stripe], (at[0], 0.0, at[2]), angle), "hull_stripe", "stripe", thick=0.01,
             normal=(math.sin(angle) * side, 0, math.cos(angle) * side))
    wx, wz = at[0] + math.cos(angle) * -0.9, at[2] - math.sin(angle) * -0.9
    boxg(wx - 0.6, wx + 0.6, at[1] + 0.5, at[1] + 1.8, wz - 0.55, wz + 0.55, "cabin", "wheelhouse")
    boxg(wx - 0.75, wx + 0.75, at[1] + 1.8, at[1] + 1.95, wz - 0.7, wz + 0.7, "cabin_roof", "roof")
    mx, mz = at[0] + math.cos(angle) * 0.9, at[2] - math.sin(angle) * 0.9
    bm = bmesh.new()
    tube(bm, (mx, at[1] + 0.5, mz), (mx, at[1] + 5.2, mz), 0.07, segs=8)
    tube(bm, (mx, at[1] + 1.4, mz), (mx - math.cos(angle) * 2.4, at[1] + 1.5, mz + math.sin(angle) * 2.4), 0.05, segs=8)
    ob("mast", bm, "wood_dark", smooth=True)
    bm = bmesh.new()
    tube(bm, (mx - math.cos(angle) * 0.2, at[1] + 1.6, mz + math.sin(angle) * 0.2), (mx - math.cos(angle) * 2.3, at[1] + 1.7, mz + math.sin(angle) * 2.3), 0.14, segs=10)
    ob("furled_sail", bm, "sail", smooth=True)


def scaled(pivot, s, build, *args):
    """Runs `build` and scales what it made by `s` about `pivot` (game coordinates): the match camera sees the water and the floor only
    far out, where a thing must be large to read."""
    before = set(bpy.data.objects)
    build(*args)
    c = P(*pivot)
    for o in [o for o in bpy.data.objects if o not in before and o.type == "MESH"]:
        for v in o.data.vertices:
            v.co = c + (v.co - c) * s


def beach_a():
    # (Out where the match camera sees the water over the dock: anything nearer stands behind the deck.) A fishing boat moored to a
    # buoy off the fisherman's hut, nets over its side and crates on its deck.
    bx, bz, angle = 15.0, -66.0, 0.35
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
        boxg(bx + dx - 0.45, bx + dx + 0.45, SEA + 1.55, SEA + 2.25, bz - 0.4, bz + 0.4, "fish_crate", "boat_crate%d" % k)
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
        boxg(p.x - 0.3, p.x + 0.3, p.y + 0.1 + h, p.y + 0.55 + h, p.z - 0.25, p.z + 0.25, "fish_crate", "crate%d" % k)


def beach_b():
    # Perched on what stands on the dock (StageArt._props on the beach): the lamp post's top, the stacked crates, the life ring's post.
    for (x, y, z) in ((-6.5, 2.27, -5.2), (-11.3, 1.92, -4.8), (9.6, 1.62, -4.7)):
        gull((x, y + 0.12, z), 1.4, glide=False)
    # Gliding over the water to either side of the fight, high enough to stay clear of it.
    rng = random.Random(3)
    for (x, y, z, s) in ((-14, 8, -26, 2.6), (15, 10, -32, 2.8), (-28, 12, -46, 3.0), (30, 13, -52, 3.0), (2, 17, -72, 3.4)):
        gull((x, y, z), s, glide=True, turn=rng.uniform(-0.5, 3.6))
    scaled((-20.0, SEA, -62.0), 1.6, rowboat, -20.0, -62.0)


def rowboat(x0, z0):
    """A rowboat beside the sailboat, its oars shipped."""
    hull = [(-1.4, 0.0, 0.0), (1.2, 0.0, 0.0), (1.6, 0.55, 0.0), (-1.5, 0.5, 0.0)]
    for side in (-0.5, 0.5):
        card([(x0 + x, SEA - 0.1 + y, z0 + side) for (x, y, _z) in hull], "hull", "row_side", thick=0.05, normal=(0, 0, side))
    card([(x0 - 1.45, SEA + 0.35, z0 - 0.5), (x0 + 1.4, SEA + 0.35, z0 - 0.5), (x0 + 1.4, SEA + 0.35, z0 + 0.5), (x0 - 1.45, SEA + 0.35, z0 + 0.5)],
         "deck", "row_floor", thick=0.05, normal=(0, 1, 0))
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
    smoke((bx + 3.0, SEA + 10.0, -108.0), 6, 0.9, rng)
    smoke((bx - 9.0, SEA + 9.0, -105.0), 5, 0.8, rng)


# ---- The canyon ----------------------------------------------------------------------------------------------------------------------


def canyon_a():
    rng = random.Random(4)
    # (Nearer and higher than first tried, and bolder: at -110 the trestle read as a hairline behind the buttes.)
    z = -92.0
    top = -2.5
    x0, x1 = -110.0, 110.0
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
    scaled((tx, y0, z), 1.6, train, tx, y0, z, rng)
    scaled((-30.0, FLOOR, z + 7.0), 1.4, water_tower, -30.0, z + 7.0)


def train(tx, y0, z, rng):
    bm = bmesh.new()
    kit.lathe(bm, [(0.75, 0.0), (0.75, 3.2)], 14, axis="x")
    for v in bm.verts:
        v.co = P(tx + v.co.x, y0 + 1.3 + v.co.z, z + -v.co.y)
    ob("boiler", bm, "engine", smooth=True)
    boxg(tx - 1.6, tx, y0 + 0.4, y0 + 2.7, z - 0.9, z + 0.9, "engine_cab", "cab")
    boxg(tx - 1.8, tx + 0.2, y0 + 2.7, y0 + 2.9, z - 1.05, z + 1.05, "engine", "cab_roof")
    bm = bmesh.new()
    tube(bm, (tx + 2.6, y0 + 2.0, z), (tx + 2.6, y0 + 3.0, z), 0.22, segs=10)
    ob("stack", bm, "engine", smooth=True)
    for k in range(2):
        cx = tx - 2.2 - k * 4.0
        boxg(cx - 3.6, cx - 0.2, y0 + 0.4, y0 + 2.2, z - 0.85, z + 0.85, "car_a" if k == 0 else "car_b", "car%d" % k)
    bm = bmesh.new()
    for wx in (tx + 2.4, tx + 0.8, tx - 2.6, tx - 5.0, tx - 6.6, tx - 9.0):
        kit.torus(bm, P(wx, y0 + 0.4, z + 0.95), P(0, 0, 1) - P(0, 0, 0), 0.35, 0.08, segs=12, rsegs=5)
    ob("wheels", bm, "iron", smooth=True)
    smoke((tx + 2.6, y0 + 3.4, z), 6, 0.55, rng)


def water_tower(wx, wz):
    # A water tower by the track.
    bm = bmesh.new()
    for (dx, dz) in ((-1.2, -1.2), (1.2, -1.2), (-1.2, 1.2), (1.2, 1.2)):
        tube(bm, (wx + dx, FLOOR, wz + dz), (wx + dx * 0.85, FLOOR + 6.0, wz + dz * 0.85), 0.16, segs=8)
    ob("tower_legs", bm, "timber", smooth=True)
    bm = bmesh.new()
    kit.lathe(bm, [(1.6, 6.0), (1.7, 6.2), (1.7, 9.0), (1.6, 9.1)], 20)
    for v in bm.verts:
        v.co = P(wx + v.co.x, FLOOR + v.co.z, wz - v.co.y)
    ob("tank", bm, "tank", smooth=True)
    bm = bmesh.new()
    kit.coil_stack(bm, 0.0, 1.1, 1.9, 0.2, 6, 0.09, segs=20)
    for v in bm.verts:
        v.co = P(wx + v.co.x, FLOOR + 9.15 + v.co.z, wz - v.co.y)
    ob("tank_roof", bm, "tank_dark", smooth=True)


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
    # A covered wagon on the trail, its oxen ahead, twice and more life size to read at the floor's distance.
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
    for k in range(2):
        g = turned([(3.2 + k * 0.1, 0.0, -0.5 + k * 1.0)], (p.x, FLOOR, p.z), ang)[0]
        body = turned([(-0.8, 0.6, 0.0), (0.7, 0.6, 0.0), (1.0, 1.2, 0.0), (0.8, 1.5, 0.0), (-0.8, 1.4, 0.0)], g, ang)
        card(body, "ox", "ox", thick=0.5)


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
    # A corral of rolled rails with cattle in it.
    r = 6.0
    bm = bmesh.new()
    posts = [(cx + r * math.cos(math.tau * k / 12), cz + r * math.sin(math.tau * k / 12)) for k in range(12)]
    for (px, pz) in posts:
        tube(bm, (px, FLOOR, pz), (px, FLOOR + 1.6, pz), 0.08, segs=6)
    for a, b in zip(posts, posts[1:] + posts[:1]):
        for h in (0.6, 1.3):
            tube(bm, (a[0], FLOOR + h, a[1]), (b[0], FLOOR + h, b[1]), 0.05, segs=5)
    ob("corral", bm, "timber", smooth=True)
    for k in range(4):
        x = cx + rng.uniform(-3.5, 3.5)
        z = cz + rng.uniform(-3.0, 3.0)
        d = rng.choice((-1.0, 1.0))
        body = [(x - 1.0 * d, FLOOR + 0.7, z), (x + 0.8 * d, FLOOR + 0.7, z), (x + 1.3 * d, FLOOR + 1.3, z), (x + 1.1 * d, FLOOR + 1.6, z),
                (x - 1.0 * d, FLOOR + 1.5, z)]
        card(body, "cattle_" + ("a" if k % 2 == 0 else "b"), "cow", thick=0.7)
        bm = bmesh.new()
        for lx in (-0.7, -0.4, 0.4, 0.7):
            tube(bm, (x + lx * d, FLOOR, z - 0.25), (x + lx * d, FLOOR + 0.75, z - 0.25), 0.06, segs=5)
        ob("cow_legs", bm, "cattle_b", smooth=True)


def adobe(hx, hz, rng):
    # An adobe house with a chimney's smoke.
    boxg(hx - 3.0, hx + 3.0, FLOOR, FLOOR + 3.0, hz - 2.2, hz + 2.2, "adobe", "house")
    boxg(hx - 3.2, hx + 3.2, FLOOR + 3.0, FLOOR + 3.3, hz - 2.4, hz + 2.4, "adobe_dark", "roof")
    boxg(hx - 0.5, hx + 0.5, FLOOR, FLOOR + 1.9, hz + 2.2, hz + 2.3, "timber_dark", "door")
    boxg(hx + 1.3, hx + 2.2, FLOOR + 1.3, FLOOR + 2.1, hz + 2.2, hz + 2.3, "window", "window")
    boxg(hx + 1.6, hx + 2.3, FLOOR + 3.3, FLOOR + 4.4, hz - 1.0, hz - 0.3, "adobe_dark", "chimney")
    smoke((hx + 1.95, FLOOR + 4.9, hz - 0.65), 5, 0.5, rng)


BUILDS = {("ocean", "A"): beach_a, ("ocean", "B"): beach_b, ("ocean", "C"): beach_c,
          ("sunset", "A"): canyon_a, ("sunset", "B"): canyon_b, ("sunset", "C"): canyon_c}


def main():
    global ROOT
    bpy.ops.wm.read_factory_settings(use_empty=True)
    ROOT = bpy.data.objects.new("World", None)
    bpy.context.scene.collection.objects.link(ROOT)
    # (Options combine: `--variant AB` builds both.)
    for v in VARIANT:
        BUILDS[(THEME, v)]()
    # (The water or the desert floor under them, for the ground's shade at their feet.)
    kit.bake_occlusion(0.6, floor_z=SEA if THEME == "ocean" else FLOOR, floor_at=(0.0, 100.0), floor_size=600.0)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("world %s %s: %d triangles -> %s" % (THEME, VARIANT, tris, out))


main()
