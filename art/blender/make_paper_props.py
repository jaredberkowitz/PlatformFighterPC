"""The meadow's props as paper (fidelity review 3's idea 1, the meadow pilot): the hay bale, barrel, signpost, rock, fence post and rail,
and a flower, in three paper styles for the owner to choose from (`--prop-mock=<glb>` on the stage demo):
  A  folded card: faceted, crisp folds, flat planes (the bale an eight-sided drum, the rock a few folded planes);
  B  layered cut-outs: flat card standees with a card's thickness, stacked in layers toward the camera (a darker back card, the body,
     then the bands, strands and staves as cut strips in front), as a paper theatre is built;
  C  rolled and scored: rolled paper (the bale's ends a coiled strip, the posts rolled tubes, the rock a crumpled ball), the barrel's
     staves scored lines, the flower quilled coils.
One file per style; each prop a child `prop_<name>` standing on the origin at its in-game size (the fence rail one unit long, scaled to
its bay; the post one unit tall). Every part names its family (`straw`, `straw_end`, `band`, `wood`, `wood_light`, `wood_dark`, `stone`,
`stone_light`, `petal`, `heart`, `stem`, and `<family>_back` for a darker back card), painted by the game (`StageArt` paper props), with
its own occlusion baked into its vertex colours.

Run (Blender 5.2):
  blender --background --python art/blender/make_paper_props.py -- --style A --out <file.glb>
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
STYLE = opts.get("style", "A").upper()
# (The other stages' props, rolled and scored after the meadow pilot, in two designs each: `--variant` for all, `--b <names>` for the
# props that take B's design instead, as the owner picked them on 2026-10-21: `--variant A --b stump,crate,cactus`.)
VARIANT = opts.get("variant", "A").upper()
B_PROPS = [n for n in opts.get("b", "").split(",") if n]

# (Blender: x right, z up, -y toward the camera.)
TOWARD = -1.0

PREVIEW = {"straw": (0.95, 0.8, 0.42), "straw_end": (0.86, 0.7, 0.36), "band": (0.62, 0.44, 0.28), "wood": (0.74, 0.52, 0.32),
           "wood_light": (0.9, 0.72, 0.48), "wood_dark": (0.55, 0.36, 0.22), "stone": (0.74, 0.72, 0.8), "stone_light": (0.84, 0.83, 0.9),
           "petal": (1.0, 0.6, 0.65), "heart": (1.0, 0.85, 0.3), "stem": (0.4, 0.7, 0.35)}


def material(fam):
    m = bpy.data.materials.get(fam)
    if m is None:
        m = bpy.data.materials.new(fam)
        c = PREVIEW.get(fam.replace("_back", ""), (0.7, 0.7, 0.7))
        m.diffuse_color = (*c, 1.0)
    return m


def to_object(name, bm, fam, parent, smooth=False, face=None):
    """`face`: the way an open sheet's faces should point (a vector); closed solids are wound outward."""
    bm.normal_update()
    if face is None:
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    else:
        for f in bm.faces:
            if f.normal.dot(Vector(face)) < 0.0:
                f.normal_flip()
    bm.normal_update()
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = smooth
    me.materials.append(material(fam))
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    return ob


def empty(name, x):
    e = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(e)
    e.location = (x, 0.0, 0.0)
    return e


def lathe(bm, prof, segs, axis="z", wobble=None):
    """Rings of `prof` [(radius, height)] round the axis, `segs` around; no caps."""
    rings = []
    for k, (r, h) in enumerate(prof):
        row = []
        for i in range(segs):
            a = math.tau * i / segs
            rr = r * (wobble(a, k) if wobble else 1.0)
            if axis == "z":
                row.append(bm.verts.new((rr * math.cos(a), rr * math.sin(a), h)))
            else:
                row.append(bm.verts.new((h, rr * math.cos(a), rr * math.sin(a))))
        rings.append(row)
    for k in range(len(rings) - 1):
        for i in range(segs):
            j = (i + 1) % segs
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    return rings


def cap(bm, ring, centre, flip=False):
    c = bm.verts.new(centre)
    n = len(ring)
    for i in range(n):
        f = (ring[i], ring[(i + 1) % n], c)
        bm.faces.new(tuple(reversed(f)) if flip else f)


def card(poly, y, thick, fam, parent, name):
    """A flat card cut to `poly` [(x, z)] facing the camera at depth `y`, `thick` deep (toward the back)."""
    bm = bmesh.new()
    front = [bm.verts.new((x, y, z)) for (x, z) in poly]
    f = bm.faces.new(front)
    f.normal_update()
    if f.normal.y > 0.0:
        f.normal_flip()
    r = bmesh.ops.extrude_face_region(bm, geom=[f])
    moved = [v for v in r["geom"] if isinstance(v, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, verts=moved, vec=(0.0, thick, 0.0))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return to_object(name, bm, fam, parent)


def rounded_rect(x0, x1, z0, z1, rad, steps=4):
    pts = []
    for (cx, cz, a0) in ((x1 - rad, z0 + rad, -90), (x1 - rad, z1 - rad, 0), (x0 + rad, z1 - rad, 90), (x0 + rad, z0 + rad, 180)):
        for s in range(steps + 1):
            a = math.radians(a0 + 90.0 * s / steps)
            pts.append((cx + rad * math.cos(a), cz + rad * math.sin(a)))
    return pts


def box(x0, x1, y0, y1, z0, z1, fam, parent, name, taper=0.0):
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        t = (v.co.z + 0.5)
        v.co = Vector((x0 + (v.co.x + 0.5) * (x1 - x0), y0 + (v.co.y + 0.5) * (y1 - y0), z0 + t * (z1 - z0)))
        if taper:
            mx, my = (x0 + x1) / 2.0, (y0 + y1) / 2.0
            v.co.x = mx + (v.co.x - mx) * (1.0 - taper * t)
            v.co.y = my + (v.co.y - my) * (1.0 - taper * t)
    return to_object(name, bm, fam, parent)


def tube(bm, a, b, r, segs=10, caps=True):
    """A rolled tube from `a` to `b`."""
    d = (b - a)
    q = d.to_track_quat("Z", "Y")
    m = Matrix.Translation((a + b) / 2.0) @ q.to_matrix().to_4x4()
    bmesh.ops.create_cone(bm, cap_ends=caps, cap_tris=False, segments=segs, radius1=r, radius2=r, depth=d.length, matrix=m)


def ring(bm, centre, axis, r, width, thick, segs=24):
    """A flat band round `axis` (a rolled strip), `width` along the axis, `thick` outward."""
    q = Vector(axis).to_track_quat("Z", "Y").to_matrix().to_4x4()
    m = Matrix.Translation(centre) @ q
    prof = [(r, -width / 2), (r + thick, -width / 2), (r + thick, width / 2), (r, width / 2), (r, -width / 2)]
    rings = []
    for (rr, h) in prof:
        row = [bm.verts.new(m @ Vector((rr * math.cos(math.tau * i / segs), rr * math.sin(math.tau * i / segs), h))) for i in range(segs)]
        rings.append(row)
    for k in range(len(rings) - 1):
        for i in range(segs):
            j = (i + 1) % segs
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))


def spiral(bm, centre, axis_x_sign, r_out, turns, width, gap, lift):
    """A coiled paper strip on a bale's end (in the y-z plane at x = centre.x), standing `lift` proud."""
    pts_in, pts_out = [], []
    n = int(turns * 40)
    for i in range(n + 1):
        t = i / n
        a = t * turns * math.tau
        r = r_out * (0.12 + 0.88 * t)
        pts_in.append((r - width / 2, a))
        pts_out.append((r + width / 2, a))
    vs = []
    for (ri, a), (ro, _) in zip(pts_in, pts_out):
        for (rr, x) in ((ri, centre.x), (ro, centre.x), (ro, centre.x + axis_x_sign * lift), (ri, centre.x + axis_x_sign * lift)):
            vs.append(bm.verts.new((x, centre.y + rr * math.cos(a), centre.z + rr * math.sin(a))))
    for i in range(n):
        a = vs[i * 4:i * 4 + 4]
        b = vs[i * 4 + 4:i * 4 + 8]
        for k in range(4):
            bm.faces.new((a[k], a[(k + 1) % 4], b[(k + 1) % 4], b[k]))


# ---- The props, per style ------------------------------------------------------------------------------------------------------------

BALE_R = 0.57
BALE_L = 1.3


def hay_bale(p):
    if STYLE == "A":
        bm = bmesh.new()
        prof = [(BALE_R * 0.92, -BALE_L / 2), (BALE_R, -BALE_L / 2 + 0.08), (BALE_R, BALE_L / 2 - 0.08), (BALE_R * 0.92, BALE_L / 2)]
        rings = lathe(bm, prof, 8, axis="x")
        for v in bm.verts:
            v.co.z += BALE_R
        ends = [[v.co.copy() for v in rings[0]], [v.co.copy() for v in rings[-1]]]
        to_object("straw", bm, "straw", p)
        for side, ring_ in ((-1, ends[0]), (1, ends[1])):
            bm = bmesh.new()
            r2 = [bm.verts.new(co + Vector((side * -0.02, 0, 0))) for co in ring_]
            cap(bm, r2, (side * (BALE_L / 2 - 0.05), 0.0, BALE_R), flip=side > 0)
            # (A folded inner octagon: the roll's turn.)
            inner = [bm.verts.new((side * (BALE_L / 2 - 0.035), BALE_R * 0.45 * math.cos(math.tau * i / 8 + 0.4), BALE_R + BALE_R * 0.45 * math.sin(math.tau * i / 8 + 0.4))) for i in range(8)]
            cap(bm, inner, (side * (BALE_L / 2 - 0.03), 0.0, BALE_R), flip=side > 0)
            to_object("end%d" % side, bm, "straw_end", p, face=(side, 0, 0))
        for x in (-0.3, 0.3):
            bm = bmesh.new()
            ring(bm, Vector((x, 0, BALE_R)), (1, 0, 0), BALE_R + 0.004, 0.09, 0.02, segs=8)
            to_object("band", bm, "band", p)
    elif STYLE == "B":
        # (End on: a round standee facing -x, as the game turns a bale a quarter round to show its end to the camera.)
        def disc(r, x, thick, fam, name, n=28):
            pts = [(r * math.cos(math.tau * i / n), BALE_R + r * math.sin(math.tau * i / n)) for i in range(n)]
            ob = card(pts, 0.0, thick, fam, p, name)
            # (Cut in the x-z plane facing -y, then turned to face -x, `x` toward the camera after the game's turn.)
            for v in ob.data.vertices:
                v.co = Vector((-v.co.y + x, v.co.x, v.co.z))
            return ob
        disc(BALE_R + 0.02, 0.35, 0.3, "straw_back", "back")
        disc(BALE_R, 0.0, 0.32, "straw", "body")
        # (The body's face toward the camera is at x -0.32: the rings and bands stand on it.)
        for k, r in enumerate((0.44, 0.31, 0.18)):
            disc(r, -0.33 - 0.02 * k, 0.02, "straw_end" if k % 2 == 0 else "straw", "ring%d" % k)
        for zz in (BALE_R - 0.25, BALE_R + 0.25):
            ob = card([(-BALE_R * 0.95, zz - 0.04), (BALE_R * 0.95, zz - 0.04), (BALE_R * 0.95, zz + 0.04), (-BALE_R * 0.95, zz + 0.04)], 0.0, 0.02, "band", p, "band")
            for v in ob.data.vertices:
                v.co = Vector((-v.co.y - 0.4, v.co.x, v.co.z))
    else:
        bm = bmesh.new()
        prof = [(BALE_R * 0.96, -BALE_L / 2), (BALE_R, -BALE_L / 2 + 0.04), (BALE_R, BALE_L / 2 - 0.04), (BALE_R * 0.96, BALE_L / 2)]
        rings = lathe(bm, prof, 28, axis="x")
        for v in bm.verts:
            v.co.z += BALE_R
        ends = [[v.co.copy() for v in rings[0]], [v.co.copy() for v in rings[-1]]]
        to_object("straw", bm, "straw", p, smooth=True)
        for side, ring_ in ((-1, ends[0]), (1, ends[1])):
            bm = bmesh.new()
            cap(bm, [bm.verts.new(co) for co in ring_], (side * BALE_L / 2, 0.0, BALE_R), flip=side > 0)
            to_object("end%d" % side, bm, "straw_back", p, face=(side, 0, 0))
            bm = bmesh.new()
            spiral(bm, Vector((side * BALE_L / 2, 0.0, BALE_R)), side, BALE_R * 0.92, 4.5, 0.05, 0.02, 0.025)
            to_object("coil%d" % side, bm, "straw_end", p)
        for x in (-0.3, 0.3):
            bm = bmesh.new()
            ring(bm, Vector((x, 0, BALE_R)), (1, 0, 0), BALE_R + 0.003, 0.07, 0.015, segs=28)
            to_object("band", bm, "band", p, smooth=True)


BARREL = [(0.33, 0.0), (0.38, 0.12), (0.42, 0.32), (0.43, 0.55), (0.42, 0.78), (0.38, 0.98), (0.35, 1.1)]


def barrel(p):
    if STYLE == "B":
        sil = [(r, h) for (r, h) in BARREL] + [(-r, h) for (r, h) in reversed(BARREL)]
        card([(x * 1.04, z) for (x, z) in sil], 0.08, 0.16, "wood_back", p, "back")
        card(sil, 0.0, 0.08, "wood", p, "body")
        for x in (-0.2, 0.0, 0.2):
            card([(x - 0.015, 0.06), (x + 0.015, 0.06), (x + 0.015, 1.04), (x - 0.015, 1.04)], -0.015, 0.015, "wood_dark", p, "stave")
        for h in (0.2, 0.86):
            w = 0.43 * (1.0 - abs(h - 0.55) * 0.25)
            card([(-w, h - 0.05), (w, h - 0.05), (w, h + 0.05), (-w, h + 0.05)], -0.03, 0.02, "band", p, "hoop")
        return
    segs = 10 if STYLE == "A" else 24
    bm = bmesh.new()
    rings = lathe(bm, BARREL, segs)
    cap(bm, [bm.verts.new((v.co.x * 0.9, v.co.y * 0.9, 1.06)) for v in rings[-1]], (0, 0, 1.06))
    to_object("body", bm, "wood", p, smooth=STYLE == "C")
    for h in (0.2, 0.86):
        r = 0.43 - abs(h - 0.55) * 0.12
        bm = bmesh.new()
        ring(bm, Vector((0, 0, h)), (0, 0, 1), r + 0.005, 0.08, 0.022, segs=segs)
        to_object("hoop", bm, "band", p, smooth=STYLE == "C")
    if STYLE == "C":
        # (Scored staves: a thin pressed line down the card every thirtieth of a turn, in the darker wood.)
        bm = bmesh.new()
        for i in range(12):
            a = math.tau * (i + 0.5) / 12
            for k in range(len(BARREL) - 1):
                (r0, h0), (r1, h1) = BARREL[k], BARREL[k + 1]
                vs = [bm.verts.new(((r + 0.003) * math.cos(a + da), (r + 0.003) * math.sin(a + da), h)) for (r, h, da) in
                      ((r0, h0, -0.012), (r0, h0, 0.012), (r1, h1, 0.012), (r1, h1, -0.012))]
                bm.faces.new(vs)
        to_object("score", bm, "wood_dark", p)


def signpost(p):
    def arrow(x0, x1, z, h, point):
        tip = x1 + 0.12 if point > 0 else x0 - 0.12
        if point > 0:
            return [(x0, z - h / 2), (x1, z - h / 2), (tip, z), (x1, z + h / 2), (x0, z + h / 2)]
        return [(tip, z), (x0, z - h / 2), (x1, z - h / 2), (x1, z + h / 2), (x0, z + h / 2)]
    boards = [arrow(-0.25, 0.75, 1.4, 0.34, 1), arrow(-0.7, 0.2, 1.0, 0.3, -1)]
    if STYLE == "B":
        card([(-0.08, 0.0), (0.08, 0.0), (0.08, 1.62), (0.0, 1.7), (-0.08, 1.62)], 0.0, 0.06, "wood", p, "post")
        for k, b in enumerate(boards):
            card(b, -0.05 - k * 0.03, 0.04, "wood_light", p, "board%d" % k)
        return
    if STYLE == "A":
        box(-0.08, 0.08, -0.08, 0.08, 0.0, 1.62, "wood", p, "post")
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, segments=4, radius1=0.115, radius2=0.0, depth=0.1,
                              matrix=Matrix.Translation((0, 0, 1.67)) @ Matrix.Rotation(math.pi / 4, 4, "Z"))
        to_object("cap", bm, "wood_dark", p)
    else:
        bm = bmesh.new()
        tube(bm, Vector((0, 0, 0)), Vector((0, 0, 1.66)), 0.075, segs=12)
        to_object("post", bm, "wood", p, smooth=True)
        bm = bmesh.new()
        spiral(bm, Vector((0.0, 0.0, 0.0)), 1, 0.07, 2.0, 0.025, 0.01, 0.02)
        for v in bm.verts:
            v.co = Vector((v.co.y, v.co.z, 1.66 + (v.co.x)))
        to_object("coil", bm, "wood_dark", p)
    for k, b in enumerate(boards):
        if STYLE == "A":
            card(b, -0.09 - k * 0.0, 0.07, "wood_light", p, "board%d" % k)
        else:
            # (A scored board: folded a little along its middle.)
            obj = card(b, -0.09, 0.05, "wood_light", p, "board%d" % k)
            zc = 1.4 if k == 0 else 1.0
            for v in obj.data.vertices:
                v.co.y -= 0.03 * (1.0 - min(1.0, abs(v.co.z - zc) / 0.17))


def rock(p):
    rng = random.Random(11)
    if STYLE == "B":
        pts = []
        for i in range(9):
            a = math.pi * i / 8
            r = rng.uniform(0.85, 1.05)
            pts.append((1.25 * r * math.cos(a), 1.0 * r * math.sin(a) * 0.95))
        card(pts, 0.1, 0.3, "stone", p, "body")
        card([(-0.7, 0.35), (0.1, 0.85), (0.6, 0.6), (0.2, 0.2)], -0.02, 0.02, "stone_light", p, "facet")
        return
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=1 if STYLE == "A" else 3, radius=1.0)
    for v in bm.verts:
        n = v.co.normalized()
        d = rng.uniform(0.88, 1.08) if STYLE == "A" else 1.0 + 0.09 * math.sin(n.x * 9.0 + n.y * 5.0) * math.cos(n.z * 7.0 + n.x * 3.0) + rng.uniform(-0.04, 0.04)
        v.co = Vector((n.x * 1.25 * d, n.y * d, max(0.0, n.z * 0.95 * d + 0.35)))
    to_object("body", bm, "stone", p)


def fence_post(p):
    if STYLE == "B":
        card([(-0.11, 0.0), (0.11, 0.0), (0.11, 0.9), (0.0, 1.0), (-0.11, 0.9)], 0.0, 0.08, "wood_light", p, "post")
        return
    if STYLE == "A":
        box(-0.11, 0.11, -0.11, 0.11, 0.0, 0.92, "wood_light", p, "post")
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, segments=4, radius1=0.155, radius2=0.0, depth=0.12,
                              matrix=Matrix.Translation((0, 0, 0.98)) @ Matrix.Rotation(math.pi / 4, 4, "Z"))
        to_object("cap", bm, "wood", p)
        return
    bm = bmesh.new()
    tube(bm, Vector((0, 0, 0)), Vector((0, 0, 0.96)), 0.1, segs=12)
    to_object("post", bm, "wood_light", p, smooth=True)
    bm = bmesh.new()
    spiral(bm, Vector((0.0, 0.0, 0.0)), 1, 0.095, 2.5, 0.03, 0.01, 0.02)
    for v in bm.verts:
        v.co = Vector((v.co.y, v.co.z, 0.96 + v.co.x))
    to_object("coil", bm, "wood", p)


def fence_rail(p):
    """One unit long along x, centred; the game scales it to its bay."""
    if STYLE == "B":
        card([(-0.5, -0.075), (0.5, -0.07), (0.5, 0.075), (-0.5, 0.07)], 0.0, 0.03, "wood", p, "rail")
    elif STYLE == "A":
        box(-0.5, 0.5, -0.045, 0.045, -0.075, 0.075, "wood", p, "rail")
    else:
        bm = bmesh.new()
        tube(bm, Vector((-0.5, 0, 0)), Vector((0.5, 0, 0)), 0.06, segs=10)
        to_object("rail", bm, "wood", p, smooth=True)


def flower(p):
    head = 0.2
    big = 1.6
    if STYLE == "B":
        pts = []
        for i in range(10):
            a = math.pi / 2 + math.tau * i / 10
            r = 0.15 if i % 2 == 0 else 0.07
            pts.append((r * math.cos(a), head + r * math.sin(a)))
        card(pts, 0.0, 0.015, "petal", p, "petals")
        card([(0.045 * math.cos(math.tau * i / 8), head + 0.045 * math.sin(math.tau * i / 8)) for i in range(8)], -0.012, 0.01, "heart", p, "heart")
        card([(-0.012, 0.0), (0.012, 0.0), (0.012, head - 0.05), (-0.012, head - 0.05)], 0.005, 0.01, "stem", p, "stem")
        return
    if STYLE == "A":
        bm = bmesh.new()
        for i in range(5):
            a = math.tau * i / 5
            d = Vector((math.cos(a), math.sin(a), 0.0))
            s = Vector((-d.y, d.x, 0.0))
            c0 = Vector((0, 0, head))
            tip = c0 + d * 0.16 + Vector((0, 0, 0.05))
            mid = c0 + d * 0.08 + Vector((0, 0, 0.035))
            vs = [bm.verts.new(c0), bm.verts.new(mid + s * 0.06), bm.verts.new(tip), bm.verts.new(mid - s * 0.06)]
            bm.faces.new((vs[0], vs[1], vs[2]))
            bm.faces.new((vs[0], vs[2], vs[3]))
        to_object("petals", bm, "petal", p, face=(0, 0, 1))
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=True, segments=5, radius1=0.045, radius2=0.02, depth=0.04, matrix=Matrix.Translation((0, 0, head + 0.03)))
        to_object("heart", bm, "heart", p)
    else:
        bm = bmesh.new()
        for i in range(5):
            a = math.tau * i / 5
            ring(bm, Vector((0.085 * math.cos(a), 0.085 * math.sin(a), head + 0.02)), (0, 0, 1), 0.045, 0.035, 0.012, segs=12)
        to_object("petals", bm, "petal", p)
        bm = bmesh.new()
        spiral(bm, Vector((0.0, 0.0, 0.0)), 1, 0.045, 2.0, 0.018, 0.006, 0.03)
        for v in bm.verts:
            v.co = Vector((v.co.y, v.co.z, head + v.co.x))
        to_object("heart", bm, "heart", p)
    bm = bmesh.new()
    tube(bm, Vector((0, 0, 0)), Vector((0, 0, head)), 0.012, segs=6)
    to_object("stem", bm, "stem", p)
    # (Half again as large as the built dot: at the match camera a paper flower the dot's size read as a speck.)
    # (The cut-out flower stands facing the camera, so it grows across and up; the others lie open, so across and back.)
    for o in p.children:
        for v in o.data.vertices:
            v.co.x *= big
            if STYLE == "B":
                v.co.z = head + (v.co.z - head) * big if v.co.z > head - 0.06 else v.co.z
            else:
                v.co.y *= big


# ---- The other stages' props, rolled and scored (the rollout after the meadow pilot), two designs each (`--variant`) -------------------


def torus(bm, centre, axis, R, r, segs=24, rsegs=8, arc=math.tau):
    """A rolled paper tube bent round into a ring (or an `arc` of one) round `axis`."""
    q = Vector(axis).to_track_quat("Z", "Y").to_matrix().to_4x4()
    m = Matrix.Translation(centre) @ q
    full = arc >= math.tau - 1e-6
    n = segs if full else segs + 1
    rings = []
    for i in range(n):
        a = arc * i / segs
        c = Vector((R * math.cos(a), R * math.sin(a), 0.0))
        out = Vector((math.cos(a), math.sin(a), 0.0))
        rings.append([bm.verts.new(m @ (c + out * (r * math.cos(math.tau * j / rsegs)) + Vector((0, 0, r * math.sin(math.tau * j / rsegs)))))
                      for j in range(rsegs)])
    for i in range(segs):
        i2 = (i + 1) % n
        for j in range(rsegs):
            j2 = (j + 1) % rsegs
            bm.faces.new((rings[i][j], rings[i2][j], rings[i2][j2], rings[i][j2]))


def scores(bm, prof, n, lift=0.004, half=0.012, a0=0.0):
    """`n` pressed lines down a lathe's profile [(radius, height)], standing `lift` proud, each `half` (radians) either side."""
    for i in range(n):
        a = a0 + math.tau * (i + 0.5) / n
        for k in range(len(prof) - 1):
            (r0, h0), (r1, h1) = prof[k], prof[k + 1]
            vs = [bm.verts.new(((r + lift) * math.cos(a + da), (r + lift) * math.sin(a + da), h)) for (r, h, da) in
                  ((r0, h0, -half), (r0, h0, half), (r1, h1, half), (r1, h1, -half))]
            bm.faces.new(vs)


def coil_stack(bm, z0, z1, r0, r1, n, thick, segs=24):
    """A column of rolled rings stacked from `z0` to `z1`, their radius going from `r0` to `r1` (a quilled dome, a coiled roof)."""
    for k in range(n):
        t = k / max(1, n - 1)
        torus(bm, Vector((0, 0, z0 + (z1 - z0) * t)), (0, 0, 1), r0 + (r1 - r0) * t, thick, segs=segs, rsegs=6)


def mushroom(p):
    """About 1 tall; the cap in the game's colour for this one."""
    bm = bmesh.new()
    tube(bm, Vector((0, 0, 0)), Vector((0, 0, 0.66)), 0.12, segs=12)
    to_object("stalk", bm, "stalk", p, smooth=True)
    if VARIANT == "A":
        prof = [(0.47, 0.6), (0.46, 0.7), (0.41, 0.82), (0.29, 0.93), (0.12, 0.98), (0.0, 0.99)]
        bm = bmesh.new()
        lathe(bm, prof, 24)
        to_object("cap", bm, "cap", p, smooth=True, face=(0, 0, 1))
        bm = bmesh.new()
        scores(bm, prof[:-1], 12, half=0.02)
        to_object("score", bm, "cap_back", p, face=(0, 0, 1))
        bm = bmesh.new()
        cap(bm, [bm.verts.new((0.47 * math.cos(math.tau * i / 24), 0.47 * math.sin(math.tau * i / 24), 0.6)) for i in range(24)],
            (0, 0, 0.64), flip=True)
        to_object("gill", bm, "gill", p, face=(0, 0, -1))
        bm = bmesh.new()
        for (a, rr, hh) in ((0.3, 0.3, 0.88), (2.0, 0.34, 0.84), (3.9, 0.24, 0.92), (5.1, 0.36, 0.83), (1.1, 0.1, 0.985)):
            c = Vector((rr * math.cos(a), rr * math.sin(a), hh))
            n = (c - Vector((0, 0, 0.45))).normalized()
            bmesh.ops.create_circle(bm, cap_ends=True, segments=10, radius=0.055,
                                    matrix=Matrix.Translation(c + n * 0.01) @ n.to_track_quat("Z", "Y").to_matrix().to_4x4())
        to_object("spots", bm, "spot", p, face=(0, 0, 1))
    else:
        bm = bmesh.new()
        coil_stack(bm, 0.62, 0.95, 0.44, 0.08, 6, 0.055)
        to_object("cap", bm, "cap", p, smooth=True)
        bm = bmesh.new()
        for (a, hh) in ((0.4, 0.7), (2.4, 0.7), (4.4, 0.7), (1.4, 0.83), (3.4, 0.83), (5.4, 0.83)):
            rr = 0.48 if hh < 0.75 else 0.33
            torus(bm, Vector((rr * math.cos(a), rr * math.sin(a), hh)), (math.cos(a), math.sin(a), 0.3), 0.04, 0.012, segs=10, rsegs=5)
        to_object("spots", bm, "spot", p, smooth=True)


def stump(p):
    """About 1.2 across its roots and 0.7 tall."""
    if VARIANT == "A":
        prof = [(0.6, 0.0), (0.52, 0.12), (0.5, 0.4), (0.49, 0.7)]
        bm = bmesh.new()
        rings = lathe(bm, prof, 24)
        top = [v.co.copy() for v in rings[-1]]
        to_object("bark", bm, "bark", p, smooth=True)
        bm = bmesh.new()
        scores(bm, prof, 16, half=0.025)
        to_object("furrows", bm, "bark_back", p)
        bm = bmesh.new()
        cap(bm, [bm.verts.new(c) for c in top], (0, 0, 0.71))
        to_object("top", bm, "ring_top", p, face=(0, 0, 1))
        bm = bmesh.new()
        for r in (0.12, 0.24, 0.36):
            torus(bm, Vector((0, 0, 0.705)), (0, 0, 1), r, 0.008, segs=24, rsegs=4)
        to_object("rings", bm, "bark_back", p)
        bm = bmesh.new()
        for a in (0.5, 2.1, 3.7, 5.2):
            d = Vector((math.cos(a), math.sin(a), 0.0))
            tube(bm, d * 0.4 + Vector((0, 0, 0.12)), d * 0.78 + Vector((0, 0, 0.02)), 0.07, segs=8)
        to_object("roots", bm, "bark", p, smooth=True)
    else:
        bm = bmesh.new()
        coil_stack(bm, 0.055, 0.62, 0.56, 0.48, 7, 0.06)
        to_object("bark", bm, "bark", p, smooth=True)
        bm = bmesh.new()
        spiral(bm, Vector((0.0, 0.0, 0.0)), 1, 0.46, 5.0, 0.06, 0.02, 0.03)
        for v in bm.verts:
            v.co = Vector((v.co.y, v.co.z, 0.66 + v.co.x))
        to_object("top", bm, "ring_top", p)
        bm = bmesh.new()
        for a in (0.5, 2.1, 3.7, 5.2):
            torus(bm, Vector((0.6 * math.cos(a), 0.6 * math.sin(a), 0.07)), (-math.sin(a), math.cos(a), 0), 0.12, 0.05, segs=12, rsegs=6,
                  arc=math.pi)
        to_object("roots", bm, "bark", p, smooth=True)


def crate(p):
    """1 x 1 x 1, standing on the origin."""
    if VARIANT == "A":
        box(-0.5, 0.5, -0.5, 0.5, 0.0, 1.0, "crate", p, "box")
        bm = bmesh.new()
        for (x, y) in ((-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)):
            tube(bm, Vector((x, y, 0.0)), Vector((x, y, 1.0)), 0.06, segs=8)
        for z in (0.03, 0.97):
            for (a, b) in (((-0.5, -0.5), (0.5, -0.5)), ((0.5, -0.5), (0.5, 0.5)), ((0.5, 0.5), (-0.5, 0.5)), ((-0.5, 0.5), (-0.5, -0.5))):
                tube(bm, Vector((a[0], a[1], z)), Vector((b[0], b[1], z)), 0.045, segs=8)
        to_object("battens", bm, "crate_dark", p, smooth=True)
        bm = bmesh.new()
        for side in range(4):
            rot = Matrix.Rotation(side * math.pi / 2, 4, "Z")
            for z in (0.36, 0.66):
                vs = [bm.verts.new(rot @ Vector((x, -0.505, zz))) for (x, zz) in ((-0.44, z - 0.012), (0.44, z - 0.012), (0.44, z + 0.012), (-0.44, z + 0.012))]
                f = bm.faces.new(vs)
        to_object("scores", bm, "crate_dark", p)
        bm = bmesh.new()
        tube(bm, Vector((-0.42, -0.52, 0.08)), Vector((0.42, -0.52, 0.92)), 0.035, segs=8)
        to_object("brace", bm, "crate_dark", p, smooth=True)
    else:
        bm = bmesh.new()
        for side in range(4):
            rot = Matrix.Rotation(side * math.pi / 2, 4, "Z")
            for k in range(4):
                z = 0.13 + k * 0.245
                tube(bm, rot @ Vector((-0.44, -0.44, z)), rot @ Vector((0.44, -0.44, z)), 0.1, segs=10)
        to_object("slats", bm, "crate", p, smooth=True)
        bm = bmesh.new()
        for (x, y) in ((-0.46, -0.46), (0.46, -0.46), (-0.46, 0.46), (0.46, 0.46)):
            tube(bm, Vector((x, y, 0.0)), Vector((x, y, 1.0)), 0.075, segs=10)
        to_object("posts", bm, "crate_dark", p, smooth=True)
        box(-0.42, 0.42, -0.42, 0.42, 0.9, 0.98, "crate_dark", p, "lid")


def lamp_post(p):
    """2.4 tall, its lantern at (0.58, 1.86), where the game's halo sits; the lantern `glow` is painted the stage's lamp colour."""
    bm = bmesh.new()
    tube(bm, Vector((0, 0, 0)), Vector((0, 0, 2.25)), 0.07, segs=10)
    to_object("post", bm, "wood", p, smooth=True)
    bm = bmesh.new()
    spiral(bm, Vector((0.0, 0.0, 0.0)), 1, 0.16, 2.5, 0.04, 0.01, 0.06)
    for v in bm.verts:
        v.co = Vector((v.co.y, v.co.z, v.co.x))
    tube(bm, Vector((0, 0, 2.15)), Vector((0.64, 0, 2.15)), 0.03, segs=8)
    tube(bm, Vector((0.58, 0, 2.15)), Vector((0.58, 0, 2.08)), 0.012, segs=6)
    to_object("arm", bm, "iron", p, smooth=True)
    if VARIANT == "A":
        prof = [(0.15, 1.68), (0.17, 1.76), (0.17, 1.96), (0.15, 2.04)]
        bm = bmesh.new()
        lathe(bm, prof, 16)
        for v in bm.verts:
            v.co.x += 0.58
        to_object("lantern", bm, "glow", p, smooth=True)
        bm = bmesh.new()
        scores(bm, prof, 8, half=0.05)
        for v in bm.verts:
            v.co.x += 0.58
        torus(bm, Vector((0.58, 0, 1.68)), (0, 0, 1), 0.15, 0.025, segs=16, rsegs=6)
        torus(bm, Vector((0.58, 0, 2.04)), (0, 0, 1), 0.15, 0.025, segs=16, rsegs=6)
        to_object("frame", bm, "iron", p, smooth=True)
    else:
        prof = [(0.03, 1.62), (0.13, 1.66), (0.2, 1.76), (0.22, 1.86), (0.2, 1.96), (0.13, 2.04), (0.03, 2.08)]
        bm = bmesh.new()
        lathe(bm, prof, 16, wobble=lambda a, k: 1.0 if int(round(a / (math.tau / 16))) % 2 == 0 else 0.9)
        for v in bm.verts:
            v.co.x += 0.58
        to_object("lantern", bm, "glow", p, smooth=True)
        bm = bmesh.new()
        torus(bm, Vector((0.58, 0, 1.64)), (0, 0, 1), 0.08, 0.02, segs=12, rsegs=6)
        torus(bm, Vector((0.58, 0, 2.06)), (0, 0, 1), 0.08, 0.02, segs=12, rsegs=6)
        to_object("frame", bm, "iron", p, smooth=True)


def life_ring(p):
    """A post at the back with a ring hanging on its front at 1.2 high, facing the camera (-y)."""
    bm = bmesh.new()
    tube(bm, Vector((0, 0.1, 0)), Vector((0, 0.1, 1.6)), 0.06, segs=10)
    to_object("post", bm, "wood_light", p, smooth=True)
    c = Vector((0, -0.05, 1.2))
    if VARIANT == "A":
        bm = bmesh.new()
        torus(bm, c, (0, -1, 0), 0.39, 0.11, segs=28, rsegs=10)
        to_object("ring", bm, "red", p, smooth=True)
        bm = bmesh.new()
        # (Four white bands wrapped round the tube, at the diagonals.)
        for k in range(4):
            a = math.tau * k / 4 + math.pi / 4
            centre = c + Vector((0.39 * math.cos(a), 0.0, 0.39 * math.sin(a)))
            torus(bm, centre, (-math.sin(a), 0.0, math.cos(a)), 0.118, 0.04, segs=14, rsegs=4)
        to_object("bands", bm, "white", p, smooth=True)
    else:
        for k in range(8):
            a0, a1 = math.tau * k / 8, math.tau * (k + 1) / 8
            pts = [(0.5 * math.cos(a0 + (a1 - a0) * i / 3), 1.2 + 0.5 * math.sin(a0 + (a1 - a0) * i / 3)) for i in range(4)]
            pts += [(0.28 * math.cos(a0 + (a1 - a0) * i / 3), 1.2 + 0.28 * math.sin(a0 + (a1 - a0) * i / 3)) for i in range(3, -1, -1)]
            card(pts, -0.12, 0.06, "red" if k % 2 == 0 else "white", p, "seg%d" % k)
        bm = bmesh.new()
        torus(bm, Vector((0, -0.15, 1.2)), (0, -1, 0), 0.39, 0.018, segs=32, rsegs=5)
        to_object("rope", bm, "band", p, smooth=True)


def potted_plant(p):
    prof = [(0.26, 0.0), (0.3, 0.2), (0.34, 0.45), (0.36, 0.6)]
    bm = bmesh.new()
    lathe(bm, prof, 20)
    to_object("pot", bm, "pot", p, smooth=True)
    bm = bmesh.new()
    cap(bm, [bm.verts.new((0.33 * math.cos(math.tau * i / 20), 0.33 * math.sin(math.tau * i / 20), 0.56)) for i in range(20)], (0, 0, 0.56))
    to_object("soil", bm, "pot_back", p, face=(0, 0, 1))
    bm = bmesh.new()
    torus(bm, Vector((0, 0, 0.6)), (0, 0, 1), 0.36, 0.035, segs=20, rsegs=6)
    scores(bm, prof, 10, half=0.03)
    to_object("rim", bm, "pot_back", p, smooth=True)
    bm = bmesh.new()
    for k in range(6):
        a = math.tau * k / 6 + 0.3
        d = Vector((math.cos(a), math.sin(a), 0.0))
        if VARIANT == "A":
            # (A rolled leaf: a strip curling up and out from the pot.)
            pts = []
            for i in range(9):
                t = i / 8
                pts.append(d * (0.05 + 0.42 * t) + Vector((0, 0, 0.58 + 0.55 * math.sin(t * 2.6) * (1.0 - 0.3 * t))))
            side = Vector((-d.y, d.x, 0.0)) * 0.06
            vs = [(bm.verts.new(q - side * (1 - i / 9)), bm.verts.new(q + side * (1 - i / 9))) for i, q in enumerate(pts)]
            for i in range(len(vs) - 1):
                bm.faces.new((vs[i][0], vs[i][1], vs[i + 1][1], vs[i + 1][0]))
        else:
            torus(bm, d * 0.22 + Vector((0, 0, 0.82 + 0.08 * (k % 2))), (d.x, d.y, 0.6), 0.09, 0.018, segs=12, rsegs=5)
    to_object("leaves", bm, "leaf", p, face=(0, -0.3, 1) if VARIANT == "A" else None, smooth=VARIANT == "B")


def cactus(p):
    """About 4 tall at size 1 (the game sets the size)."""
    if VARIANT == "A":
        prof = [(0.36, 0.0), (0.38, 0.4), (0.38, 3.2), (0.32, 3.55), (0.18, 3.75), (0.0, 3.82)]
        bm = bmesh.new()
        lathe(bm, prof, 20)
        to_object("trunk", bm, "cactus", p, smooth=True, face=(0, 0, 1))
        bm = bmesh.new()
        scores(bm, prof[:-1], 8, half=0.03)
        to_object("ribs", bm, "cactus_back", p, face=(0, 0, 1))
        bm = bmesh.new()
        for side, h, up in ((1, 1.5, 1.1), (-1, 2.2, 0.8)):
            tube(bm, Vector((0, 0, h)), Vector((side * 0.85, 0, h)), 0.2, segs=12)
            tube(bm, Vector((side * 0.85, 0, h - 0.1)), Vector((side * 0.85, 0, h + up)), 0.2, segs=12)
        to_object("arms", bm, "cactus", p, smooth=True)
    else:
        bm = bmesh.new()
        coil_stack(bm, 0.13, 3.5, 0.34, 0.3, 14, 0.13, segs=16)
        torus(bm, Vector((0, 0, 3.66)), (0, 0, 1), 0.14, 0.12, segs=12, rsegs=8)
        for side, h, up in ((1, 1.5, 1.1), (-1, 2.2, 0.8)):
            for k in range(4):
                torus(bm, Vector((side * (0.4 + k * 0.15), 0, h)), (1, 0, 0), 0.16, 0.08, segs=10, rsegs=6)
            for k in range(int(up / 0.2)):
                torus(bm, Vector((side * 0.85, 0, h + 0.2 + k * 0.2)), (0, 0, 1), 0.15, 0.09, segs=10, rsegs=6)
        to_object("trunk", bm, "cactus", p, smooth=True)


def water_tower(p):
    """The city's tank on legs: legs 2.4 tall at +-0.9, the tank 2.6 across from 2.4 to 4.6, a roof to 5.6, a ladder on its front right."""
    bm = bmesh.new()
    for (x, y) in ((-0.9, -0.9), (0.9, -0.9), (-0.9, 0.9), (0.9, 0.9)):
        tube(bm, Vector((x, y, 0)), Vector((x * 0.85, y * 0.85, 2.45)), 0.1, segs=10)
    tube(bm, Vector((-0.85, -0.85, 1.2)), Vector((0.85, -0.85, 1.2)), 0.05, segs=8)
    tube(bm, Vector((-0.85, 0.85, 1.2)), Vector((0.85, 0.85, 1.2)), 0.05, segs=8)
    to_object("legs", bm, "wood_dark", p, smooth=True)
    prof = [(1.25, 2.4), (1.3, 2.5), (1.3, 4.5), (1.26, 4.6)]
    if VARIANT == "A":
        bm = bmesh.new()
        lathe(bm, prof, 32)
        to_object("tank", bm, "tank", p, smooth=True)
        bm = bmesh.new()
        scores(bm, prof, 18, half=0.012)
        to_object("staves", bm, "tank_dark", p)
        bm = bmesh.new()
        coil_stack(bm, 4.62, 5.5, 1.42, 0.12, 9, 0.07, segs=32)
        to_object("roof", bm, "tank_dark", p, smooth=True)
    else:
        bm = bmesh.new()
        coil_stack(bm, 2.5, 4.5, 1.25, 1.25, 9, 0.13, segs=32)
        to_object("tank", bm, "tank", p, smooth=True)
        bm = bmesh.new()
        rp = [(1.45, 4.6), (0.9, 5.0), (0.3, 5.45), (0.0, 5.55)]
        lathe(bm, rp, 32)
        to_object("roof", bm, "tank_dark", p, smooth=True, face=(0, 0, 1))
        bm = bmesh.new()
        scores(bm, rp[:-1], 16, half=0.02)
        to_object("seams", bm, "tank", p, face=(0, 0, 1))
    bm = bmesh.new()
    for h in (2.6, 3.5, 4.4):
        torus(bm, Vector((0, 0, h)), (0, 0, 1), 1.33, 0.035, segs=32, rsegs=5)
    for lx in (-0.18, 0.18):
        tube(bm, Vector((1.0 + lx, -0.95, 0.5)), Vector((1.0 + lx, -0.95, 3.9)), 0.03, segs=6)
    for k in range(5):
        tube(bm, Vector((0.8, -0.95, 0.8 + k * 0.7)), Vector((1.2, -0.95, 0.8 + k * 0.7)), 0.022, segs=6)
    to_object("iron", bm, "iron", p, smooth=True)


def ac_unit(p):
    """1.6 wide, 1.1 tall, 1.2 deep, its front -y."""
    box(-0.8, 0.8, -0.6, 0.6, 0.0, 1.05, "metal", p, "body")
    bm = bmesh.new()
    if VARIANT == "A":
        for k in range(6):
            z = 0.2 + k * 0.13
            tube(bm, Vector((-0.66, -0.62, z)), Vector((0.66, -0.62, z)), 0.045, segs=8)
    else:
        for k in range(9):
            x = -0.6 + k * 0.15
            bm.faces.new([bm.verts.new((x + dx, -0.605, z)) for (dx, z) in ((-0.02, 0.15), (0.02, 0.15), (0.02, 0.9), (-0.02, 0.9))])
    to_object("grille", bm, "metal_dark", p, smooth=VARIANT == "A", face=None if VARIANT == "A" else (0, -1, 0))
    bm = bmesh.new()
    spiral(bm, Vector((0.0, 0.0, 0.0)), 1, 0.4, 3.0, 0.04, 0.02, 0.03)
    for v in bm.verts:
        v.co = Vector((v.co.y + 0.2, v.co.z, 1.05 + v.co.x))
    to_object("fan", bm, "metal_dark", p)


def roof_vent(p):
    """About 1.2 tall: a stack bending over into a hood facing down and to the side."""
    bm = bmesh.new()
    if VARIANT == "A":
        tube(bm, Vector((0, 0, 0)), Vector((0, 0, 0.8)), 0.16, segs=14)
    else:
        coil_stack(bm, 0.05, 0.8, 0.14, 0.14, 7, 0.06, segs=14)
    torus(bm, Vector((0.25, 0, 0.8)), (0, 1, 0), 0.25, 0.16, segs=8, rsegs=14, arc=math.pi)
    to_object("stack", bm, "metal", p, smooth=True)
    bm = bmesh.new()
    torus(bm, Vector((0, 0, 0.05)), (0, 0, 1), 0.2, 0.05, segs=16, rsegs=6)
    torus(bm, Vector((0, 0, 0.5)), (0, 0, 1), 0.17, 0.025, segs=16, rsegs=5)
    to_object("bands", bm, "metal_dark", p, smooth=True)


def telegraph_pole(p):
    """The canyon's telegraph pole at its true size (fidelity review 7: the last code-built pieces): a rolled pole 9 tall tapering up, its
    rolls scored, a crossarm 2.6 wide at 8.4 with a brace each side, four glass insulators on top of it (the wire hooks at 8.6)."""
    bm = bmesh.new()
    prof = [(0.3, 0.0), (0.27, 1.0), (0.22, 9.0)]
    lathe(bm, prof, 12)
    to_object("pole", bm, "wood", p, smooth=True)
    bm = bmesh.new()
    scores(bm, prof, 6, half=0.03)
    to_object("pole_scores", bm, "wood_dark", p, smooth=True)
    bm = bmesh.new()
    tube(bm, Vector((-1.3, 0, 8.4)), Vector((1.3, 0, 8.4)), 0.11, segs=8)
    for sx in (-1.0, 1.0):
        tube(bm, Vector((0, 0, 7.6)), Vector((sx * 0.7, 0, 8.36)), 0.045, segs=6)
    to_object("arm", bm, "wood_dark", p, smooth=True)
    bm = bmesh.new()
    for x in (-1.1, -0.45, 0.45, 1.1):
        prof_i = [(0.0, 8.5), (0.07, 8.5), (0.08, 8.58), (0.06, 8.66), (0.03, 8.72), (0.0, 8.72)]
        lathe(bm, [(r, z) for (r, z) in prof_i], 8)
        for v in bm.verts[-8 * len(prof_i):]:
            v.co.x += x
    to_object("insulators", bm, "glass", p, smooth=True)


def net_float(p):
    """A fisherman's float bobbing on the water (fidelity review 7: the stilt hut's floats were two spheres): a rolled body, red, with a
    white band round its middle and a rope loop on top, its foot in the water."""
    prof = [(0.0, -0.32), (0.3, -0.28), (0.46, -0.12), (0.5, 0.05), (0.42, 0.24), (0.22, 0.36), (0.0, 0.38)]
    bm = bmesh.new()
    lathe(bm, prof, 16)
    to_object("body", bm, "red", p, smooth=True)
    bm = bmesh.new()
    torus(bm, Vector((0, 0, 0.05)), (0, 0, 1), 0.5, 0.06, segs=20, rsegs=6)
    to_object("band", bm, "white", p, smooth=True)
    bm = bmesh.new()
    torus(bm, Vector((0, 0, 0.5)), (0, 1, 0), 0.12, 0.025, segs=12, rsegs=5)
    to_object("loop", bm, "band", p, smooth=True)


def leaf_tuft(p):
    """A low tuft of rolled leaves for the grove's flower clusters (fidelity review 7: the clusters' leaves were green spheres): seven
    curled leaf strips fanning from the middle, longer at the back."""
    bm = bmesh.new()
    for k in range(7):
        a = math.tau * k / 7 + 0.3
        ln = 0.32 + 0.12 * (0.5 + 0.5 * math.sin(a + 1.2))
        d = Vector((math.cos(a), math.sin(a), 0.0))
        s = Vector((-d.y, d.x, 0.0)) * 0.05
        pts = []
        for i in range(5):
            t = i / 4
            c = d * ln * t + Vector((0, 0, 0.24 * math.sin(t * math.pi * 0.8)))
            w = (1.0 - t) * 0.9 + 0.1
            pts.append((c + s * w, c - s * w))
        for i in range(4):
            a0, b0 = pts[i]
            a1, b1 = pts[i + 1]
            bm.faces.new([bm.verts.new(a0), bm.verts.new(a1), bm.verts.new(b1), bm.verts.new(b0)])
    to_object("leaves", bm, "leaf", p, smooth=True)


PROPS = [("hay_bale", hay_bale), ("barrel", barrel), ("signpost", signpost), ("rock", rock), ("fence_post", fence_post),
         ("fence_rail", fence_rail), ("flower", flower), ("mushroom", mushroom), ("stump", stump), ("crate", crate),
         ("lamp_post", lamp_post), ("life_ring", life_ring), ("potted_plant", potted_plant), ("cactus", cactus),
         ("water_tower", water_tower), ("ac_unit", ac_unit), ("roof_vent", roof_vent),
         ("telegraph_pole", telegraph_pole), ("net_float", net_float), ("leaf_tuft", leaf_tuft)]


def bake_occlusion(distance, floor_z=0.0, floor_at=(80.0, 0.0), floor_size=320.0):
    """Occlusion into each part's vertex colours, a floor at `floor_z` (Blender z) under them all."""
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
    # (A floor under them all, so each prop takes the ground's shade at its foot.)
    bpy.ops.mesh.primitive_plane_add(size=floor_size, location=(floor_at[0], floor_at[1], floor_z))
    floor = bpy.context.active_object
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        a = o.data.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
        o.data.color_attributes.active_color = a
        o.data.color_attributes.render_color_index = o.data.color_attributes.active_color_index
        o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.bake(type="AO", target="VERTEX_COLORS")
    bpy.data.objects.remove(floor, do_unlink=True)
    for o in parts:
        for c in o.data.color_attributes["Col"].data:
            ao = 0.62 + 0.38 * c.color[0]
            c.color = (ao, ao, ao, 1.0)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    roots = []
    global VARIANT
    chosen = VARIANT
    for k, (name, build) in enumerate(PROPS):
        # (Apart for the bake, so one prop does not shade the next; back on the origin for export.)
        root = empty("prop_" + name, k * 9.0)
        VARIANT = "B" if name in B_PROPS else chosen
        build(root)
        roots.append(root)
    VARIANT = chosen
    # (Subdivided for the vertex bake where a face is long, so the shade has vertices to land on.)
    for o in [o for o in bpy.context.scene.objects if o.type == "MESH"]:
        bm = bmesh.new()
        bm.from_mesh(o.data)
        for _ in range(3):
            long_edges = [e for e in bm.edges if e.calc_length() > 0.35]
            if not long_edges:
                break
            bmesh.ops.subdivide_edges(bm, edges=long_edges, cuts=1, use_grid_fill=True)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        bm.to_mesh(o.data)
        bm.free()
    bpy.context.view_layer.update()
    bake_occlusion(0.5)
    for r in roots:
        r.location = (0.0, 0.0, 0.0)
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("paper props %s, new designs %s: %d triangles -> %s" % (STYLE, VARIANT, tris, out))


# (Run as a script; art/blender/make_world.py imports its helpers.)
if __name__ == "__main__":
    main()
