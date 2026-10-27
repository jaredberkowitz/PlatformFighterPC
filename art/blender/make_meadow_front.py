"""The meadow's stage front as one modelled piece (the stage refinement, after the grove pilot; docs/art/stages/BAKE_PILOT.md).

Built on the stage's own collision, read from the bake manifest (the block's rectangle and the front plane), following the block's
batter (it narrows 0.13 a unit down, `meadow.cfg` `taper`), starting under the lawn's slab and ending at the block's foot, where the
meadow's hill takes over. Three looks for the owner to pick from (`--front-mock=` on the stage demo):
  A  a folded rock cliff: the face pleated into ridges and creases running down, each plane turned well toward or away from the light,
     in the owner's light moss, as the meadow's mountains are folded;
  B  a terrace wall of logs and stones: courses of round logs with their cut ends showing, on courses of bevelled stones, moss in the
     joints, the farm's own build (fence, bales, barrel);
  C  an earth bank: folded soil with bevelled stones set in it, the hillside cut back.
All three: a grass overhang along the top.
After fidelity review 6 (B's stones, in even courses of one size and one grey, read as a grid of bricks), on B:
  `--stones field`  fieldstone: courses of uneven height, big stones with small ones packed round them, each a rough lump of its own
                    tone and tint (ochre, blue-grey, rosy, lichen), set in or out a little;
  `--moss 1`        with it, moss on the stones' tops and dripping down a few faces, thickest under the logs, and grass at the foot. Families `ledge`, `stone`, `log`, `log_end`, `soil`, `grass`, `moss`, painted from the
stage's palette in the game; each piece bakes its occlusion into linear vertex colours with the built meadow round it.

Shipped (fidelity review 9, the owner's pick B, 2026-10-25): godot/models/stages/meadow_front.glb, `--variant B --stones field --moss 1
--warm B --calm B`, then lit (review 8's had no `--calm`, review 6's no `--warm`).

Run (Blender 5.2):
  blender --background --python art/blender/make_meadow_front.py -- --dir godot/art/baked/<key> --variant A --out <file.glb>
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold_kit import folded_dome  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
VARIANT = opts.get("variant", "A").upper()
FIELD = opts.get("stones", "") == "field"
MOSS = opts.get("moss", "0") == "1"
# Fieldstone tints (multiplied into the baked occlusion with the stone's own value): warm ochre, blue-grey, plain, rosy, lichen.
TINTS = [(1.0, 0.86, 0.68), (0.8, 0.86, 1.0), (0.96, 0.96, 0.96), (1.0, 0.82, 0.76), (0.86, 0.95, 0.74)]
# Fidelity review 8 (the wall read as grey cubes with neon moss drips): `--warm A|B`. Both paint the stones in their own warm family
# (`fieldstone`, not the old wall's near-grey `stone`, which the front's calm and shade then took to grey) and the moss in a deep olive
# (`wall_moss`, not the lawn's lime), as low caps with no drips. B also gives every stone a wider warm-to-cool spread and keeps the moss to
# the courses under the logs and the wall's foot.
WARM = opts.get("warm", "")
# Fidelity review 9 (at the wide camera the wall was a grid of small pale stones, each outlined, the busiest and brightest mass under the
# fight): `--calm A|B`. A: fewer, larger stones in taller courses, rarely split, their values closer; B: A, a step lower in value, so the
# wall sits back under the lawn.
CALM = opts.get("calm", "").upper()
# Fidelity review 13 (the wall still read as near-identical grey boxes in a grid): `--laid A|B`, on the calm fieldstone. A: hand-laid,
# each stone its own height within its course and set at its own height in it, rounder (corners pulled in further, a wider bevel),
# widths ranging further, and small chinking stones packed into the gaps, set back; B: A with the stones' values and set spread wider.
LAID = opts.get("laid", "").upper()
if WARM:
    TINTS = [(1.0, 0.84, 0.66), (0.84, 0.86, 0.94), (1.0, 0.92, 0.8), (0.98, 0.78, 0.68), (0.9, 0.9, 0.78)]
    if WARM == "B":
        TINTS = [(1.0, 0.8, 0.6), (0.78, 0.82, 0.94), (1.0, 0.94, 0.82), (0.96, 0.72, 0.62), (0.86, 0.88, 0.72), (0.7, 0.66, 0.62)]
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
BLOCK = [r for r in manifest["rects"] if r[4] < 0.5][0]
LEFT, RIGHT, TOP, BOTTOM = BLOCK[0], BLOCK[1], BLOCK[2], BLOCK[3]
FZ = float(manifest["front_z"])
TAPER = 0.13
Y_TOP = TOP - 0.55
Y_BOT = BOTTOM


def P(x, y, z):
    return Vector((x, -z, y))


def inset(y):
    return min(max(TOP - y, 0.0) * TAPER, (RIGHT - LEFT) / 3.0)


def edges(y):
    i = inset(y)
    return LEFT + i, RIGHT - i


PARTS = {}


def part(name, mat):
    key = (name, mat)
    if key not in PARTS:
        PARTS[key] = bmesh.new()
        PARTS[key].verts.layers.float.new("tone")
        for c in ("tr", "tg", "tb"):
            PARTS[key].verts.layers.float.new(c)
    return PARTS[key]


def stone(bm, c, size, rng, bevel=0.18):
    res = bmesh.ops.create_cube(bm, size=2.0)
    verts = res["verts"]
    rot = Matrix.Rotation(rng.uniform(-0.15, 0.15), 3, "Y")
    for v in verts:
        v.co = rot @ Vector((v.co.x * size.x * rng.uniform(0.9, 1.1), v.co.y * size.y, v.co.z * size.z * rng.uniform(0.9, 1.1))) + c
    edges_ = list({e for v in verts for e in v.link_edges})
    out = bmesh.ops.bevel(bm, geom=edges_, offset=min(size.x, size.z) * bevel, segments=1, affect="EDGES")
    # (Each stone its own tone, multiplied into its baked occlusion: one grey for all read as pasted cubes.)
    layer = bm.verts.layers.float.get("tone")
    tone = random.Random(round(c.x * 10.0) * 7919 + round(c.z * 10.0)).uniform(0.7, 1.05)
    for v in out["verts"]:
        if layer is not None and v.is_valid:
            v[layer] = tone


def rough_stone(bm, c, size, rng, tint, value, pull=0.14, bevel=0.16):
    """A fieldstone: a box pushed out of square at every corner, bevelled, in its own value and tint."""
    res = bmesh.ops.create_cube(bm, size=2.0)
    verts = res["verts"]
    rot = Matrix.Rotation(rng.uniform(-0.08, 0.08), 3, "Y")
    deep = size.y * rng.uniform(0.85, 1.0)
    for v in verts:
        # (Each corner pulled in, never out: pushed out, a stone ran into its neighbours and the corners buried in them baked black. Its
        # depth one for the whole stone: a front twisted out of plane folded under the bevel and baked black too.)
        kx, kz = 1.0 - rng.uniform(0.0, pull), 1.0 - rng.uniform(0.0, pull)
        v.co = rot @ Vector((v.co.x * kx * size.x, v.co.y * deep, v.co.z * kz * size.z)) + c
    edges_ = list({e for v in verts for e in v.link_edges})
    # (Clamped: on a lump with corners pulled in unevenly, a wider bevel overlapped itself, folded faces inward and baked them black.)
    out = bmesh.ops.bevel(bm, geom=edges_, offset=min(size.x, size.z) * bevel, segments=1, affect="EDGES", clamp_overlap=True)
    layers = [bm.verts.layers.float.get(k) for k in ("tone", "tr", "tg", "tb")]
    for v in out["verts"]:
        if v.is_valid:
            v[layers[0]] = value
            for k in range(3):
                v[layers[k + 1]] = tint[k]


def moss_on(bm, c, size, rng, drip):
    """Moss over a stone's top, a little proud of it, and now and then a tongue of it hanging down its face."""
    top = c.z + size.z * 1.02 + 0.03
    front = c.y - size.y * 0.95
    sx = size.x * rng.uniform(0.55, 0.9)
    ox = rng.uniform(-0.2, 0.2) * size.x
    cap = bm.faces.new([bm.verts.new(Vector(q)) for q in ((c.x + ox - sx, front - 0.04, top), (c.x + ox + sx, front - 0.04, top),
                                                           (c.x + ox + sx * 0.8, c.y + size.y * 0.3, top), (c.x + ox - sx * 0.8, c.y + size.y * 0.3, top))])
    cap.normal_update()
    if cap.normal.z < 0:
        cap.normal_flip()
    if drip:
        w = sx * rng.uniform(0.3, 0.6)
        dx = c.x + ox + rng.uniform(-0.4, 0.4) * sx
        hang = size.z * rng.uniform(0.4, 0.9)
        # (Five-sided with an uneven foot: as a triangle it read as a sticker.)
        tongue = bm.faces.new([bm.verts.new(Vector(q)) for q in ((dx - w, front - 0.05, top), (dx + w, front - 0.05, top),
                                                                  (dx + w * 0.7, front - 0.05, top - hang * 0.45),
                                                                  (dx + w * 0.15, front - 0.05, top - hang),
                                                                  (dx - w * 0.6, front - 0.05, top - hang * 0.6))])
        tongue.normal_update()
        if tongue.normal.y > 0:
            tongue.normal_flip()


def field_wall(y, rng):
    """The wall under the logs in fieldstone (fidelity review 6): courses of uneven height, a big stone now and then with smaller ones
    packed above and below it, every stone a rough lump of its own value and tint, set in or out a little."""
    bm = part("Stones", "fieldstone" if WARM else "stone")
    moss = part("Moss", "wall_moss" if WARM else "moss") if MOSS else None
    first = True
    course = 0
    while y > Y_BOT + 0.25:
        lo, hi = edges(y)
        row_h = rng.uniform(0.85, 1.25) if CALM else rng.uniform(0.5, 0.85)
        x = lo - rng.uniform(0.0, 0.5)
        while x < hi - 0.15:
            big = rng.random() < 0.3
            if LAID:
                w = rng.uniform(1.9, 3.1) if big else rng.uniform(0.8, 1.9)
                stack = [row_h] if big or rng.random() < 0.65 else [row_h * 0.45, row_h * 0.55]
            elif CALM:
                w = rng.uniform(2.0, 3.0) if big else rng.uniform(1.2, 2.0)
                stack = [row_h] if big or rng.random() < 0.8 else [row_h * 0.45, row_h * 0.55]
            else:
                w = rng.uniform(1.0, 1.8) if big else rng.uniform(0.45, 1.0)
                stack = [row_h] if big or rng.random() < 0.55 else [row_h * 0.45, row_h * 0.55]
            yy = y
            for h in stack:
                hh = h - 0.05
                ww = w * (rng.uniform(0.8, 1.0) if len(stack) > 1 else 1.0)
                cx = min(x + ww * 0.5, hi - ww * 0.4)
                drop = 0.0
                if LAID:
                    # (Its own height in the course, and sat at its own height in it.)
                    short = hh * rng.uniform(0.0, 0.28)
                    drop = rng.uniform(0.0, short)
                    hh -= short
                set_in = rng.uniform(-0.08, 0.1) * (1.8 if LAID == "B" else 1.0)
                c = P(cx, yy - drop - hh * 0.5, FZ + 0.32 + set_in)
                size = Vector((ww * 0.47, 0.32, hh * 0.47))
                value = rng.uniform(0.62, 1.0) if WARM == "B" else rng.uniform(0.72, 1.0)
                if CALM:
                    value = 0.8 + (value - 0.8) * (0.85 if LAID == "B" else 0.5) - (0.12 if CALM == "B" else 0.0)
                if LAID:
                    rough_stone(bm, c, size, rng, rng.choice(TINTS), value, pull=0.26, bevel=0.24)
                    if rng.random() < 0.45:
                        # A chinking stone packed into the gap at the stone's side, set back and a step darker.
                        cw, ch = rng.uniform(0.18, 0.32), rng.uniform(0.14, 0.26)
                        cc = P(min(x + ww + 0.03, hi - 0.1), yy - rng.uniform(0.15, max(0.2, h - 0.2)), FZ + 0.42)
                        rough_stone(bm, cc, Vector((cw, 0.2, ch)), rng, rng.choice(TINTS), value * 0.75, pull=0.3, bevel=0.3)
                else:
                    rough_stone(bm, c, size, rng, rng.choice(TINTS), value)
                if WARM:
                    # (Caps only, no drips; with B only under the logs.)
                    share = (0.7 if first else (0.0 if WARM == "B" else 0.15))
                    if moss is not None and rng.random() < share:
                        moss_on(moss, c, size, rng, drip=False)
                elif moss is not None and rng.random() < (0.75 if first else 0.22):
                    moss_on(moss, c, size, rng, drip=rng.random() < (0.5 if first else 0.25))
                yy -= h
            x += w + 0.06
        y -= row_h + 0.05
        first = False
    if moss is not None:
        # Grass at the wall's foot, where it meets the hill.
        g = part("Foot", "grass")
        lo, hi = edges(Y_BOT + 0.3)
        x = lo
        while x < hi:
            for _ in range(3):
                bx = x + rng.uniform(-0.2, 0.2)
                ht = rng.uniform(0.3, 0.6)
                f = g.faces.new([g.verts.new(q) for q in (P(bx - 0.18, Y_BOT, FZ + 0.7), P(bx + 0.18, Y_BOT, FZ + 0.7),
                                                          P(bx + rng.uniform(-0.15, 0.15), Y_BOT + ht, FZ + 0.75))])
                f.normal_update()
                if f.normal.y > 0:
                    f.normal_flip()
            x += rng.uniform(0.5, 1.1)


def folded_face(bm, rng, depth, width, step=0.5):
    """The face pleated into ridges running down: columns of their own widths, every other one standing `depth` proud, leaning a little,
    so each plane turns well toward or away from the light (a shallow fold lit both its sides alike and vanished, pass 2)."""
    xs = []
    lo, hi = edges(Y_TOP)
    x = lo
    while x < hi:
        xs.append(x)
        x += width * rng.uniform(0.7, 1.3)
    xs.append(hi)
    rows = int(math.ceil((Y_TOP - Y_BOT) / step))
    grid = []
    for j in range(rows + 1):
        y = Y_TOP + (Y_BOT - Y_TOP) * j / rows
        lo_y, hi_y = edges(y)
        lo_t, hi_t = edges(Y_TOP)
        row = []
        for i, x0 in enumerate(xs):
            x = lo_y + (x0 - lo_t) / (hi_t - lo_t) * (hi_y - lo_y)
            ridge = (i % 2 == 1) and 0 < i < len(xs) - 1
            lean = 0.25 * math.sin(i * 1.7) * (TOP - y) / 8.0
            out = (depth * rng.uniform(0.8, 1.15) if ridge else 0.0) * min(1.0, (Y_TOP - y + 0.3) / 0.8)
            row.append(bm.verts.new(P(x + lean, y, FZ + 0.05 + out)))
        grid.append(row)
    for j in range(rows):
        for i in range(len(xs) - 1):
            a, b, c, d = grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]
            bm.faces.new((a, d, c, b))


def plain_face(bm, rng, bulge=0.0, step=0.5, cols=40):
    rows = int(math.ceil((Y_TOP - Y_BOT) / step))
    grid = []
    for j in range(rows + 1):
        y = Y_TOP + (Y_BOT - Y_TOP) * j / rows
        lo, hi = edges(y)
        row = []
        for i in range(cols + 1):
            x = lo + (hi - lo) * i / cols
            out = bulge * (0.5 + 0.5 * math.sin(x * 0.9 + y * 1.3)) * (1 if 0 < i < cols and 0 < j else 0)
            row.append(bm.verts.new(P(x, y, FZ + 0.03 + out)))
        grid.append(row)
    for j in range(rows):
        for i in range(cols):
            a, b, c, d = grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]
            bm.faces.new((a, d, c, b))


def log(bm_side, bm_end, x0, x1, y, r, rng):
    """A round log lying along the face from `x0` to `x1` at height `y`, its cut ends showing."""
    sides = 10
    zc = FZ + r * 0.75
    rings = []
    for x in (x0, x1):
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            rr = r * rng.uniform(0.95, 1.05)
            ring.append(Vector((x, -(zc + math.cos(a) * rr), y + math.sin(a) * rr)))
        rings.append(ring)
    sv = [[bm_side.verts.new(p) for p in ring] for ring in rings]
    for i in range(sides):
        j = (i + 1) % sides
        f = bm_side.faces.new((sv[0][i], sv[0][j], sv[1][j], sv[1][i]))
        f.normal_update()
        mid = Vector(((x0 + x1) * 0.5, -zc, y))
        if f.normal.dot(f.calc_center_median() - Vector((f.calc_center_median().x, mid.y, mid.z))) < 0:
            f.normal_flip()
    for k, ring in enumerate(rings):
        ev = [bm_end.verts.new(p) for p in ring]
        f = bm_end.faces.new(ev)
        f.normal_update()
        want = Vector((-1, 0, 0)) if k == 0 else Vector((1, 0, 0))
        if f.normal.dot(want) < 0:
            f.normal_flip()


def grass_overhang(rng):
    lit, shade = part("Grass", "grass"), part("Grass", "moss")
    lo, hi = edges(Y_TOP)
    x = lo + 0.3
    while x < hi - 0.2:
        r = rng.uniform(0.35, 0.6)
        # (Hanging over the face's top, in front of the lawn's slab: tucked under it, the overhang showed as a thin dark strip.)
        c = P(x, Y_TOP - 0.35 + rng.uniform(-0.1, 0.05), FZ + 0.55)
        n_l, n_s = len(lit.verts), len(shade.verts)
        folded_dome(lit, shade, tuple(c), r, 6, rng)
        # (Upright and squashed, not turned over: turned over, their undersides faced the sky inside the lawn's slab and baked a dark
        # purple band along the edge.)
        flip = Matrix.Rotation(rng.uniform(-0.25, 0.25), 3, "X") @ Matrix.Diagonal((1.3, 0.9, rng.uniform(0.45, 0.7)))
        for bm, n in ((lit, n_l), (shade, n_s)):
            bm.verts.ensure_lookup_table()
            for v in bm.verts[n:]:
                v.co = flip @ (v.co - c) + c
        x += r * rng.uniform(0.9, 1.3)


def build():
    rng = random.Random(17)
    if VARIANT == "A":
        folded_face(part("Cliff", "ledge"), rng, depth=0.95, width=1.25)
        # A few boulders where the cliff meets the hill.
        bm = part("Boulders", "ledge")
        lo, hi = edges(Y_BOT + 0.6)
        for k in range(7):
            x = rng.uniform(lo + 0.6, hi - 0.6)
            s = rng.uniform(0.4, 0.75)
            stone(bm, P(x, Y_BOT + s * 0.5, FZ + 1.0 + s * 0.3), Vector((s * 1.3, s * 0.8, s)), rng, bevel=0.3)
    elif VARIANT == "B":
        plain_face(part("Backing", "soil"), rng)
        side, end = part("Logs", "log"), part("Logs", "log_end")
        y = Y_TOP - 0.45
        course = 0
        # (Two courses of logs over courses of stone: logs to the foot read as a cabin wall.)
        while y > TOP - 2.2:
            r = 0.42
            lo, hi = edges(y)
            x = lo - (rng.uniform(0.2, 1.6) if course % 2 else 0.0)
            while x < hi:
                length = rng.uniform(3.0, 5.0)
                x1 = min(x + length, hi + 0.1)
                log(side, end, max(x, lo - 0.1), x1, y + rng.uniform(-0.04, 0.04), r * rng.uniform(0.92, 1.08), rng)
                x = x1 + 0.06
            y -= r * 1.9
            course += 1
        if FIELD:
            field_wall(y, rng)
            y = Y_BOT
        bm = part("Stones", "stone")
        while y > Y_BOT + 0.3:
            lo, hi = edges(y)
            x = lo + rng.uniform(0.0, 0.5)
            h = rng.uniform(0.38, 0.55)
            while x < hi - 0.2:
                w = rng.uniform(0.6, 1.2)
                stone(bm, P(min(x + w * 0.5, hi - w * 0.4), y - h * 0.5, FZ + 0.32), Vector((w * 0.48, 0.32, h * 0.48)), rng)
                x += w + 0.05
            y -= h + 0.06
    else:
        plain_face(part("Bank", "soil"), rng, bulge=0.5, step=0.4, cols=60)
        bm = part("Stones", "stone")
        for k in range(30):
            y = rng.uniform(Y_BOT + 0.4, Y_TOP - 0.9)
            lo, hi = edges(y)
            x = rng.uniform(lo + 0.4, hi - 0.4)
            s = rng.uniform(0.25, 0.75)
            # (Set into the bank, a third showing: standing proud they read as cubes pasted on.)
            stone(bm, P(x, y, FZ + 0.08), Vector((s * 1.3, s * 0.55, s * 0.9)), rng, bevel=0.35)
    grass_overhang(rng)


def clamp(bm, margin=0.9):
    for v in bm.verts:
        y = min(max(v.co.z, BOTTOM), TOP)
        lo, hi = edges(y)
        v.co.x = min(max(v.co.x, lo - margin), hi + margin)
        v.co.z = min(v.co.z, TOP - 0.3)


def make_objects():
    for (name, mat), bm in PARTS.items():
        # (A part made and left empty, the old wall's stones with the fieldstone laid: nothing to bake.)
        if len(bm.faces) == 0:
            bm.free()
            continue
        clamp(bm)
        me = bpy.data.meshes.new(name + "_" + mat)
        bm.normal_update()
        bm.to_mesh(me)
        bm.free()
        for p in me.polygons:
            p.use_smooth = False
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        ob = bpy.data.objects.new(name + "_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def bake_occlusion():
    mock = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    gone = {e["name"] for e in manifest["meshes"] if e["path"].split("/")[-1].startswith(("faceted_ledge", "block", "lip"))}
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.join(opts["dir"], "scene_in.glb"), merge_vertices=True)
    for o in [o for o in bpy.context.scene.objects if o not in before]:
        if o.name in gone:
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
    scene.world.light_settings.distance = 2.0
    bpy.ops.object.select_all(action="DESELECT")
    for o in mock:
        a = o.data.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
        o.data.color_attributes.active_color = a
        o.data.color_attributes.render_color_index = o.data.color_attributes.active_color_index
        o.select_set(True)
    bpy.context.view_layer.objects.active = mock[0]
    bpy.ops.object.bake(type="AO", target="VERTEX_COLORS")
    for o in mock:
        tone = o.data.attributes.get("tone")
        if tone is None:
            continue
        tint = [o.data.attributes.get(k) for k in ("tr", "tg", "tb")]
        col = o.data.color_attributes["Col"].data
        for v in o.data.vertices:
            k = tone.data[v.index].value or 1.0
            # (A fieldstone's tint, 0 where none was set.)
            t = [(a.data[v.index].value or 1.0) if a is not None else 1.0 for a in tint]
            c = list(col[v.index].color)
            if tint[0] is not None and tint[0].data[v.index].value > 0.0:
                # (A fieldstone's shade floored: packed close, the corners in its joints baked to nothing and whole faces drew black.)
                c = [0.25 + 0.75 * q for q in c[:3]]
            col[v.index].color = (c[0] * k * t[0], c[1] * k * t[1], c[2] * k * t[2], 1.0)
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
    print("meadow front %s: %d triangles -> %s" % (VARIANT, tris, out))


main()
