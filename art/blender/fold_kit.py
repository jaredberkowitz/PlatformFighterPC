"""The folded construction kit (docs/art/STYLE_SHEET.md, section 9, rule 9 "Folded, not crumpled"): the shapes every natural prop is
built from, so a new prop starts in the game's language instead of reinventing it. Pines and bushes stack `pleated_tier`s; leafy crowns,
bushes and moss heap `folded_dome`s; ranges and hills are `folded_peak`s. Imported by make_props.py.
"""

import math

from mathutils import Vector


def pleated_tier(bm_out, bm_under, at, z_bot, r, h, pleats, turn, rng, flare=0.42, droop=0.1):
    """One tier of a papercraft conifer or a bush (the owner's blue-mountain reference, 2026-10-19): a skirt folded in and out like a
    paper fan, `pleats` points round its rim, so the key light falls on one side of every fold and not the other and each tier reads as
    crisp stripes of light and shade. Steep near its tip and flaring at the foot (`flare`, the waist's radius as a share of the rim's),
    the points hanging lower than the folds between them (`droop`), so the rim is a deep sawtooth; its underside a shallow dark cone
    in `bm_under`."""
    ax, ay = at
    tip = bm_out.verts.new((ax, ay, z_bot + h))
    waist, rim = [], []
    n = pleats * 2
    for i in range(n):
        a = turn + math.tau * i / n
        point = i % 2 == 0
        jit = rng.uniform(0.92, 1.08) if point else 1.0
        rr = r * (1.0 if point else 0.8) * jit
        zr = z_bot - (droop * jit if point else -0.6 * droop)
        rw = r * flare * (1.0 if point else 0.8)
        zw = z_bot + h * 0.45
        rim.append((ax + math.cos(a) * rr, ay + math.sin(a) * rr, zr))
        waist.append((ax + math.cos(a) * rw, ay + math.sin(a) * rw, zw))
    wv = [bm_out.verts.new(p) for p in waist]
    rv = [bm_out.verts.new(p) for p in rim]
    for i in range(n):
        j = (i + 1) % n
        bm_out.faces.new((tip, wv[i], wv[j]))
        # Two triangles per panel, each its own plane: the folds stay crisp.
        bm_out.faces.new((wv[i], rv[i], rv[j]))
        bm_out.faces.new((wv[i], rv[j], wv[j]))
    under = bm_under.verts.new((ax, ay, z_bot + h * 0.18))
    uv = [bm_under.verts.new(p) for p in rim]
    for i in range(n):
        bm_under.faces.new((under, uv[(i + 1) % n], uv[i]))


def outward(bm, axis):
    """Turns every face of an open shell to face away from `axis` (a point under its middle): recalc_face_normals guesses on open
    meshes."""
    bm.normal_update()
    for f in bm.faces:
        if f.normal.dot(f.calc_center_median() - Vector(axis)) < 0.0:
            f.normal_flip()


def folded_dome(bm_lit, bm_shade, centre, r, pleats, rng, rows=None):
    """A round leaf clump folded like the pines (the owner's blue-mountain reference, 2026-10-19): a dome wider than tall whose surface is
    pleated in and out from its crown to its rim, the folds deepest at the rim and fading toward the top, the rim a skirt of leaf points
    hanging a little; so the key light falls on one side of every fold and the clump reads as crisp stripes over a round mass. The skirt
    and the underside go to `bm_shade` (the deep green), the dome above to `bm_lit`."""
    cx, cy, cz = centre
    turn = rng.uniform(0, math.tau)
    n = pleats * 2
    # (elevation as a share of a quarter turn, how deep the folds are there)
    # (Two bands and a cap, the judge's round 39: four bands of many pleats read as small evenly lit triangles, crumpled noise; a few
    # big folds read as planes of light and shade.)
    if rows is None:
        rows = [(-0.12, 0.34), (0.32, 0.22), (0.68, 0.1)]
    made = []
    rings = []
    for (el, depth) in rows:
        e = el * math.pi / 2.0
        ring_v = []
        for i in range(n):
            a = turn + math.tau * i / n
            point = i % 2 == 0
            rr = math.cos(e) * (1.0 if point else 1.0 - depth) * r
            z = math.sin(e) * 0.78 * r
            if el < 0.0 and point:
                z -= 0.24 * r * rng.uniform(0.8, 1.2)
            ring_v.append((cx + math.cos(a) * rr * 1.08, cy + math.sin(a) * rr, cz + z))
        rings.append(ring_v)
    top = (cx, cy, cz + 0.8 * r)
    for k in range(len(rings) - 1):
        bm = bm_shade if k == 0 else bm_lit
        lo = [bm.verts.new(p) for p in rings[k]]
        hi = [bm.verts.new(p) for p in rings[k + 1]]
        for i in range(n):
            j = (i + 1) % n
            made.append(bm.faces.new((lo[i], lo[j], hi[j])))
            made.append(bm.faces.new((lo[i], hi[j], hi[i])))
    hi = [bm_lit.verts.new(p) for p in rings[-1]]
    t = bm_lit.verts.new(top)
    for i in range(n):
        made.append(bm_lit.faces.new((hi[i], hi[(i + 1) % n], t)))
    # The underside: a shallow dark cone under the rim.
    rim = [bm_shade.verts.new(p) for p in rings[0]]
    u = bm_shade.verts.new((cx, cy, cz - 0.1 * r))
    for i in range(n):
        made.append(bm_shade.faces.new((u, rim[(i + 1) % n], rim[i])))
    for f in made:
        f.normal_update()
        if f.normal.dot(f.calc_center_median() - Vector((cx, cy, cz - 0.3 * r))) < 0.0:
            f.normal_flip()


def folded_peak(bm, px, py, h, reach, foot, rng, rounded=False):
    """One summit folded like paper (the owner's blue-mountain reference, 2026-10-19), into `bm`: an apex at (px, py, h) with ridges
    running straight down from it to `foot` about `reach` away, a crease between each pair, so its flanks are a few big planes each
    wholly lit or in shade. `rounded`: a blunt dome of a hill rather than a sharp peak. Faces are left for the caller to orient."""
    ridges = rng.randint(6, 9)
    base_turn = rng.uniform(0, math.tau / ridges)
    # (Each summit leans off its footprint's middle and is wider than deep, and some ridges hold a shoulder partway down, so no two
    # peaks are the same pyramid: an even row of symmetric cones read as a diagram.)
    lean = rng.uniform(-0.3, 0.3) * reach * (1.0 if rounded else 1.8)
    apex = bm.verts.new((px, py, h))
    lines = []
    for k in range(ridges * 2):
        ridge = k % 2 == 0
        a = base_turn + math.tau * k / (ridges * 2) + rng.uniform(-0.12, 0.12)
        L = reach * (rng.uniform(0.8, 1.2) if rounded else rng.uniform(0.55, 1.45)) * (1.0 if ridge else (0.85 if rounded else 0.72))
        shoulder = rng.uniform(0.06, 0.16) * h if ridge and not rounded and rng.random() < 0.4 else 0.0
        pts = []
        for j, t in enumerate((0.2, 0.42, 0.68, 1.0)):
            # A ridge drops straight then eases out at its foot; a crease sinks a little deeper between its ridges.
            tt = t + rng.uniform(-0.04, 0.04) * (t < 1.0)
            z = h - (h - foot) * (tt ** (0.85 if not rounded else 1.5))
            z += shoulder * (j in (1, 2))
            side = rng.uniform(-0.1, 0.1) * L * (0.3 if rounded else 1.0)
            d = L * tt
            pts.append(bm.verts.new((px - lean * tt + (math.cos(a) * d - math.sin(a) * side) * 1.25,
                                     py + math.sin(a) * d + math.cos(a) * side,
                                     z - (0.0 if ridge else (0.03 if rounded else 0.07) * h * math.sin(math.pi * tt)))))
        lines.append(pts)
    m = len(lines)
    for k in range(m):
        a_line, b_line = lines[k], lines[(k + 1) % m]
        bm.faces.new((apex, a_line[0], b_line[0]))
        for j in range(3):
            bm.faces.new((a_line[j], a_line[j + 1], b_line[j + 1]))
            bm.faces.new((a_line[j], b_line[j + 1], b_line[j]))


def block_tier(bm, at, z0, h, radii, turn, taper=0.94, lean=(0.0, 0.0)):
    """One chunky ledge of cut rock (the owner's canyon reference, 2026-10-19: blocky sculpted cliffs, a few planes per tier): a prism on
    an irregular footprint of `len(radii)` corners round `at` (each corner its own reach, so the sides are uneven planes), `h` tall from
    `z0`, its top a little smaller (`taper`) and shifted by `lean`; capped top and bottom. Faces are built outward."""
    n = len(radii)
    ax, ay = at
    lo, hi = [], []
    for i, r in enumerate(radii):
        a = turn + math.tau * i / n
        lo.append(bm.verts.new((ax + math.cos(a) * r, ay + math.sin(a) * r, z0)))
        hi.append(bm.verts.new((ax + lean[0] + math.cos(a) * r * taper, ay + lean[1] + math.sin(a) * r * taper, z0 + h)))
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((lo[i], lo[j], hi[j], hi[i]))
    bm.faces.new(list(reversed(lo)))
    bm.faces.new(hi)
