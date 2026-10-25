"""Mood light in paper (review 2's idea 5, the owner's go-ahead 2026-10-20: it adds elements): light as cut paper, for three stages.
  city   searchlight beams rising from behind the skyline: A two soft beams crossing; B four narrower beams in pairs, cooler; C three
         cut-out paper beams, flat pale card with a deeper edge.
  grove  paper lanterns glowing warm: A six big lanterns hanging from the platforms' back edges; B small lanterns along the two strings
         from the great tree; C both, with soft warm pools on the lawn under the platform lanterns.
  ocean  ripple rings round the dock's posts on the water: A two pale rings at each post; B rings with a bright crescent on the near
         side and a third faint ring; C rings with paper sparkles (four-pointed stars) scattered over the water.
Built in world space for the exact stage (the grove's strings and platforms from StageArt and the manifest, the dock's posts from
beach_front.glb). Every piece carries its own brightness and fade in its vertex colours (rgb a shade, alpha the fade), which the game's
unshaded glow materials use (`StageBake.place_mood`). Families `beam`, `beam_edge`, `glow`, `halo`, `pool`, `cap`, `cord`, `ring`,
`spark`.

Calmer rings (fidelity review 9: every post the same set of white rings, a row of targets), `--rings A|B` on the ocean's B: A each post
its own, some left bare, the rings a different size and strength at each and the outer one broken into arcs; B only the soft near-side
crescent, with a broken outer arc at some posts.
Shipped since fidelity review 9 (the owner's pick A): beach_mood.glb `--theme ocean --variant B --rings A`.

Run (Blender 5.2):
  blender --background --python art/blender/make_mood.py -- --dir godot/art/baked/<key> --theme grove --variant A --out <file.glb>
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
RINGS = opts.get("rings", "").upper()
THEME = opts.get("theme", "grove")
manifest = json.load(open(os.path.join(opts["dir"], "manifest.json")))
FZ = float(manifest["front_z"])
PLATFORMS = [r for r in manifest["rects"] if r[4] > 0.5]
BLOCKS = [r for r in manifest["rects"] if r[4] <= 0.5]
SEA = -8.62          # over the beach's foam line and its swell, so the rings stay on top of the water


def P(x, y, z):
    """Godot (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


PARTS = {}


def part(mat):
    if mat not in PARTS:
        bm = bmesh.new()
        bm.verts.layers.float_color.new("Col")
        PARTS[mat] = bm
    return PARTS[mat]


def face(mat, pts, cols):
    """A face whose vertices carry their own colour: a shade times its fade, so additive light fades with it (the exporter drops a
    vertex colour's alpha: kept there, every fade came out solid)."""
    bm = part(mat)
    layer = bm.verts.layers.float_color["Col"]
    vs = []
    for p, c in zip(pts, cols):
        v = bm.verts.new(p)
        v[layer] = (c[0] * c[3], c[1] * c[3], c[2] * c[3], 1.0)
        vs.append(v)
    return bm.faces.new(vs)


def card(mat, centre, rx, ry, alpha_c, alpha_e=0.0, n=20, shade=1.0):
    """A soft disc facing the camera (a halo): bright in the middle, fading out to its rim."""
    c = P(*centre)
    rim = [c + Vector((math.cos(math.tau * i / n) * rx, 0.0, math.sin(math.tau * i / n) * ry)) for i in range(n)]
    for i in range(n):
        j = (i + 1) % n
        face(mat, (c, rim[i], rim[j]), ((shade, shade, shade, alpha_c), (shade, shade, shade, alpha_e), (shade, shade, shade, alpha_e)))


def flat_disc(mat, x, y, z, rx, rz, alpha_c, alpha_e=0.0, n=24):
    """A soft disc lying flat (a light pool)."""
    c = P(x, y, z)
    rim = [P(x + math.cos(math.tau * i / n) * rx, y, z + math.sin(math.tau * i / n) * rz) for i in range(n)]
    for i in range(n):
        j = (i + 1) % n
        face(mat, (c, rim[j], rim[i]), ((1, 1, 1, alpha_c), (1, 1, 1, alpha_e), (1, 1, 1, alpha_e)))


def annulus(mat, x, z, r0, r1, alpha, a0=0.0, a1=math.tau, n=32, y=SEA, rz=0.55):
    """A flat ring on the water from radius r0 to r1 (squashed in depth by `rz`, as a ring seen low reads), between angles a0 and a1."""
    for i in range(n):
        t0 = a0 + (a1 - a0) * i / n
        t1 = a0 + (a1 - a0) * (i + 1) / n
        pts = [P(x + math.cos(t) * r, y, z + math.sin(t) * r * rz) for (t, r) in ((t0, r0), (t1, r0), (t1, r1), (t0, r1))]
        face(mat, (pts[0], pts[3], pts[2], pts[1]), ((1, 1, 1, alpha),) * 4)


# ---- City: searchlight beams --------------------------------------------------------------------------------------------------------


def beam(x, z, angle, length, w0, w1, alpha, edge=False):
    """A searchlight beam rising from (x, street) at `angle` degrees off upright: soft at its sides and fading to nothing at its end; or
    (`edge`) a flat paper beam, its core even and a deeper strip down each side."""
    base_y = -14.0
    a = math.radians(angle)
    d = Vector((math.sin(a), math.cos(a), 0.0))
    side = Vector((math.cos(a), -math.sin(a), 0.0))
    b0 = Vector((x, base_y, 0.0))
    t0 = b0 + d * length
    cols = 6
    for i in range(cols):
        u0, u1 = -1.0 + 2.0 * i / cols, -1.0 + 2.0 * (i + 1) / cols
        if edge:
            ea0 = ea1 = 1.0
        else:
            ea0 = alpha * (1.0 - abs(u0) ** 1.5)
            ea1 = alpha * (1.0 - abs(u1) ** 1.5)
        q = [b0 + side * u0 * w0, b0 + side * u1 * w0, t0 + side * u1 * w1, t0 + side * u0 * w1]
        top_a = 0.0 if not edge else 1.0
        mat = ("beam_edge" if abs(u0) > 0.67 or (abs(u1) > 0.67 and u0 > 0.3) else "beam_card") if edge else "beam"
        face(mat, [P(v.x, v.y, z) for v in q], ((1, 1, 1, ea0), (1, 1, 1, ea1), (1, 1, 1, ea1 * top_a), (1, 1, 1, ea0 * top_a)))


def city():
    if VARIANT == "A":
        beam(-30.0, -95.0, 22.0, 95.0, 0.8, 9.0, 0.24)
        beam(28.0, -95.0, -24.0, 95.0, 0.8, 9.0, 0.24)
    elif VARIANT == "B":
        for x, angs in ((-46.0, (14.0, -6.0)), (42.0, (6.0, -16.0))):
            for a in angs:
                beam(x, -100.0, a, 100.0, 0.5, 5.0, 0.17)
    else:
        for x, a in ((-38.0, 16.0), (4.0, -7.0), (44.0, -19.0)):
            beam(x, -95.0, a, 85.0, 1.2, 5.5, 0.42, edge=True)


# ---- Grove: paper lanterns ----------------------------------------------------------------------------------------------------------


def lantern(x, y, z, r, cord=0.5, halo=2.4):
    """A pleated paper lantern hanging from a cord: eight ribs bulging out, brightest at its middle; caps top and bottom; a halo."""
    ribs = 16
    rows = [(-1.0, 0.45), (-0.55, 0.85), (0.0, 1.0), (0.55, 0.85), (1.0, 0.45)]
    ring_pts = []
    for (v, s) in rows:
        row = []
        for i in range(ribs):
            a = math.tau * i / ribs
            rr = r * s * (1.0 if i % 2 == 0 else 0.9)
            row.append(P(x + math.cos(a) * rr, y + v * r * 0.75, z + math.sin(a) * rr))
        ring_pts.append(row)
    shade = [0.72, 0.92, 1.0, 0.92, 0.72]
    for k in range(len(rows) - 1):
        for i in range(ribs):
            j = (i + 1) % ribs
            c0, c1 = shade[k], shade[k + 1]
            face("glow", (ring_pts[k][i], ring_pts[k][j], ring_pts[k + 1][j], ring_pts[k + 1][i]),
                 ((c0, c0, c0, 1), (c0, c0, c0, 1), (c1, c1, c1, 1), (c1, c1, c1, 1)))
    for (yy, h) in ((y + r * 0.75, 0.12), (y - r * 0.75 - 0.12, 0.12)):
        cap_r = r * 0.5
        for i in range(8):
            a0, a1 = math.tau * i / 8, math.tau * (i + 1) / 8
            q = [P(x + math.cos(a0) * cap_r, yy, z + math.sin(a0) * cap_r), P(x + math.cos(a1) * cap_r, yy, z + math.sin(a1) * cap_r),
                 P(x + math.cos(a1) * cap_r, yy + h, z + math.sin(a1) * cap_r), P(x + math.cos(a0) * cap_r, yy + h, z + math.sin(a0) * cap_r)]
            face("cap", q, ((1, 1, 1, 1),) * 4)
    top = y + r * 0.75 + 0.12
    face("cord", (P(x - 0.02, top, z), P(x + 0.02, top, z), P(x + 0.02, top + cord, z), P(x - 0.02, top + cord, z)), ((1, 1, 1, 1),) * 4)
    if halo > 0:
        card("halo", (x, y, z + r + 0.05), r * halo, r * halo, 0.42)


def grove():
    plat = []
    for r in PLATFORMS:
        for x in (r[0] + 0.8, r[1] - 0.8):
            plat.append((x, r[2] - 1.15, FZ - 3.8))
    strings = []
    for sgn in (-1.0, 1.0):
        a = Vector((4.0 * sgn, 12.0, -17.0))
        b = Vector((22.0 * sgn, 8.0, -30.0))
        for k in range(2, 18, 2):
            f = k / 18.0
            p = a.lerp(b, f) + Vector((0.0, -1.2 * math.sin(f * math.pi), 0.0))
            strings.append((p.x, p.y - 0.75, p.z))
    if VARIANT in ("A", "C"):
        for (x, y, z) in plat:
            lantern(x, y, z, 0.34, cord=0.55)
    if VARIANT in ("B", "C"):
        for (x, y, z) in strings:
            lantern(x, y, z, 0.3, cord=0.3, halo=3.0)
    if VARIANT == "C":
        for (x, y, z) in plat:
            if y < 4.0:
                flat_disc("pool", x, 0.012, z + 0.6, 1.3, 0.7, 0.28)


# ---- Beach: ripple rings ------------------------------------------------------------------------------------------------------------


def posts():
    """The dock's posts where they meet the water (beach_front.glb, its `post` family's lowest vertices, merged into posts)."""
    bpy.ops.import_scene.gltf(filepath=opts["front"])
    pts = []
    for o in list(bpy.context.scene.objects):
        if o.type == "MESH" and any("post" in m.name for m in o.data.materials):
            for v in o.data.vertices:
                w = o.matrix_world @ v.co
                pts.append((w.x, w.z, -w.y))
        bpy.data.objects.remove(o, do_unlink=True)
    low = min(p[1] for p in pts)
    feet = [(x, z) for (x, y, z) in pts if y < low + 0.5]
    found = []
    for (x, z) in feet:
        for f in found:
            if abs(f[0] - x) < 0.9 and abs(f[1] - z) < 0.9:
                f[2].append((x, z))
                break
        else:
            found.append([x, z, [(x, z)]])
    return [(sum(a for a, _ in f[2]) / len(f[2]), sum(b for _, b in f[2]) / len(f[2])) for f in found]


def far_posts():
    """The posts of the boardwalk behind the dock and of the fisherman's hut (`StageArt._dock_harbour` and `_stilt_hut`, from the dock's
    rect): the match camera crops the dock's own waterline, so these carry the rings into its shot."""
    l, r, top, bottom = BLOCKS[0][:4]
    water = bottom - 1.0
    a = Vector((r - 5.0, top - 0.1, FZ - 8.2))
    b = Vector((28.4, water + 4.1, -45.0))
    d = b - a
    length = d.length
    d.normalize()
    side = Vector((0.0, 1.0, 0.0)).cross(-d).normalized()
    out = []
    k = 0.0
    while k < length:
        q = a + d * k
        out += [(q.x + side.x * sv, q.z + side.z * sv) for sv in (-0.85, 0.85)]
        k += 3.5
    out += [(30.0 + px, -46.0 + pz) for px in (-2.6, 2.6) for pz in (-1.8, 1.8)]
    return out


def post_rings(x, z, r_in, r_mid, r_out, rng):
    """One post's rings, calmer (`--rings`): A its own size and strength, the outer ring broken into arcs, a post in four bare; B only the
    near-side crescent, and at some posts a broken outer arc."""
    if RINGS == "A" and rng.random() < 0.25:
        return
    k = rng.uniform(0.75, 1.25)
    a = rng.uniform(0.45, 0.8)
    if RINGS == "A":
        annulus("ring", x, z, r_in * k, (r_in + 0.24) * k, 0.9 * a, n=20, rz=0.8)
    a0 = rng.uniform(0.1, 0.25) * math.pi
    annulus("ring", x, z, r_in * 0.7 * k, r_mid * k, 0.7 * a, a0=a0, a1=math.pi - a0 + rng.uniform(-0.1, 0.1), n=14, y=SEA + 0.01, rz=0.8)
    if rng.random() < (0.7 if RINGS == "A" else 0.4):
        start = rng.uniform(0.0, math.tau)
        annulus("ring", x, z, r_out * k, (r_out + 0.12) * k, 0.4 * a, a0=start, a1=start + rng.uniform(0.6, 1.4) * math.pi, n=18, rz=0.8)


def ocean():
    rng = random.Random(9)
    if RINGS:
        # (Their own random numbers, so the sparkles keep their places.)
        rr = random.Random(29)
        for (x, z) in far_posts():
            post_rings(x, z, 0.3, 0.7, 0.9, rr)
        for (x, z) in posts():
            post_rings(x, z, 0.5, 1.1, 1.3, rr)
        return
    for (x, z) in far_posts():
        annulus("ring", x, z, 0.3, 0.5, 0.9, n=20, rz=0.8)
        annulus("ring", x, z, 0.75, 0.88, 0.5, n=24, rz=0.8)
        if VARIANT == "B":
            annulus("ring", x, z, 0.24, 0.7, 0.7, a0=0.15 * math.pi, a1=0.85 * math.pi, n=12, y=SEA + 0.01, rz=0.8)
            annulus("ring", x, z, 1.1, 1.2, 0.3, n=24, rz=0.8)
    for (x, z) in posts():
        # (Wide and bright enough to read past the post from the match camera: at 0.4 to 0.6 they hid behind it, a faint white line.)
        annulus("ring", x, z, 0.5, 0.78, 0.9, rz=0.8)
        annulus("ring", x, z, 1.05, 1.22, 0.55, rz=0.8)
        if VARIANT == "B":
            annulus("ring", x, z, 0.36, 1.1, 0.7, a0=0.15 * math.pi, a1=0.85 * math.pi, n=16, y=SEA + 0.01, rz=0.8)
            annulus("ring", x, z, 1.5, 1.62, 0.3, rz=0.8)
    if VARIANT == "C":
        for k in range(36):
            x = rng.uniform(-40.0, 40.0)
            z = rng.uniform(-30.0, 10.0)
            if abs(x) < 15.0 and -6.0 < z < 3.0:
                continue
            s = rng.uniform(0.25, 0.55) * (1.0 + max(0.0, -z) * 0.02)
            c = P(x, SEA + 0.25 * s, z)
            pts = [c + Vector((0, 0, s)), c + Vector((s * 0.22, 0, s * 0.22)), c + Vector((s, 0, 0)), c + Vector((s * 0.22, 0, -s * 0.22)),
                   c + Vector((0, 0, -s)), c + Vector((-s * 0.22, 0, -s * 0.22)), c + Vector((-s, 0, 0)), c + Vector((-s * 0.22, 0, s * 0.22))]
            for i in range(8):
                face("spark", (c, pts[i], pts[(i + 1) % 8]), ((1, 1, 1, 0.95), (1, 1, 1, 0.9), (1, 1, 1, 0.9)))


def make_objects():
    for mat, bm in PARTS.items():
        bm.normal_update()
        for f in bm.faces:
            # (Cards face the camera; what lies flat faces the sky.)
            n = f.normal
            if (abs(n.z) > 0.7 and n.z < 0.0) or (abs(n.z) <= 0.7 and n.dot(Vector((0.0, -1.0, 0.0))) < -0.2 and mat != "glow" and mat != "cap"):
                f.normal_flip()
        me = bpy.data.meshes.new("Mood_" + mat)
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.get(mat) or bpy.data.materials.new(mat)
        me.materials.append(m)
        # (Only `Col`, active and for rendering: with another set beside it, the export wrote a white COLOR_0, which the game reads.)
        for ca in [c for c in me.color_attributes if c.name != "Col"]:
            me.color_attributes.remove(ca)
        col = me.color_attributes["Col"]
        me.color_attributes.active_color = col
        me.color_attributes.render_color_index = list(me.color_attributes).index(col)
        ob = bpy.data.objects.new("Mood_" + mat, me)
        bpy.context.scene.collection.objects.link(ob)
    PARTS.clear()


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    {"city": city, "grove": grove, "ocean": ocean}[THEME]()
    make_objects()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("%s mood %s: %d triangles -> %s" % (THEME, VARIANT, tris, out))


main()
