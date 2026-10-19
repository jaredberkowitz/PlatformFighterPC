"""The fighters' base body as a clean quad cage (body remodel, 2026-10-15), replacing the generated figure's voxel skin.

The generated figure (make_figure_body.py) gave the proportions but a scan-like surface: dents, a belly bulge, a mirror seam, pinched
shoulders and a neck stalk, and no edge loops where the joints bend, so skinning creased it. This builds the same proportions as a
low-poly cage of quads, laid out by hand in numbers below, then subdivides it:
  - a 16-sided torso from the crotch to the shoulders, a soft waist, the front a little fuller than the back;
  - the arms leave through an 8-edge hole in each side of the chest (a 2x2 patch of torso faces), so the shoulder is a round cap of
    the torso, not a tube glued on; 8-sided arms with three loops round each elbow;
  - the legs leave the bottom ring split in two by a crotch vertex: 10-sided legs, three loops round each knee, the feet bent forward
    out of the same tubes, flat soled;
  - the head is the torso carried on through a short wide neck into a sphere, closed by a 4x4 grid (no pole on top);
  - small mitten fists closed by a 2x2 grid, the wrists straight in line with the forearms.
Every ring is placed by its centre, its axis and its radii; a tube's rings keep the angles of the loop they start from, eased towards an
even spacing, so the quads stay square. The body is symmetric by construction (no seam). Then, as before: fit_body.py,
make_rigged_blob.py --body.

Run:
    blender --background --python art/blender/make_cage_body.py -- --out art/models/base_body/base_body.glb
Live (Blender MCP): exec this file with __name__ != "__main__", then build() returns the cage object with its subdivision modifier.
"""
import math
import sys

import bmesh
import bpy
from mathutils import Quaternion, Vector

X, Y, Z = Vector((1, 0, 0)), Vector((0, 1, 0)), Vector((0, 0, 1))
SUBDIV = 1   # subdivision levels baked into the exported body (1: ~2.7k quads, inside the ~6.5k triangle budget)

# ---- Torso: rings from the crotch up. (z, half width, half depth at the front, half depth at the back, depth centre) -----------------
# Angle 0 is the right side (+X), 90 the back (+Y), 180 the left side, 270 the front (-Y, Blender's front).
TSEG = 16
# Proportions (body rebuild, 2026-10-18, the owner's proportion study, docs/BODY_PROPORTIONS.md): after the reference cast, about 2.34
# tall: the head a third of the height, the torso a wide sturdy block (hips 0.84 of the head's width) from the crotch at 0.80 to the
# neck, the legs over a third of the height, thick and set well apart. (The old body was toddler-built: a head 43% of the height once
# scaled up, short close-set legs, a narrow body.)
TORSO = [
    (0.88, 0.375, 0.270, 0.250, 0.000),   # a sturdy block front-on, an egg side-on: as wide at the hips as across the shoulders
    (0.98, 0.395, 0.300, 0.270, 0.000),
    (1.08, 0.402, 0.310, 0.275, 0.000),
    (1.18, 0.398, 0.302, 0.270, 0.000),
    (1.28, 0.388, 0.288, 0.258, 0.000),   # shoulder hole: rings 4..6
    (1.37, 0.365, 0.262, 0.238, 0.000),
    # (Fuller to the top, the character pass, 2026-10-19, the owner's reference: the shoulders ran up into the head instead of narrowing
    # to a stalk under it, which read as a lightbulb.)
    (1.45, 0.335, 0.240, 0.218, 0.000),
    (1.52, 0.300, 0.215, 0.198, 0.000),
]
HOLE_RINGS = (4, 5, 6)
CROTCH = (0.0, 0.80)            # crotch vertex (y, z)

# ---- Neck and head: rings of the same 16 angles. (z, half width, half depth) then the sphere ------------------------------------------
# (Wider still and the head a little lower, its lowest ring as wide as the neck, the character pass, 2026-10-19: the head sits straight on
# the shoulders, a squat oval with no neck showing, as the reference cast's.)
NECK = [(1.57, 0.305, 0.245)]
HEAD_C, HEAD_R = Vector((0.0, 0.0, 1.89)), 0.41
HEAD_PHI = (50, 66, 82, 98, 118, 138, 156)   # degrees from the bottom pole; the 4x4 grid closes the last ring
HEAD_TOP = 0.03                              # how far the cap's middle rises above a flat lid

# ---- Limbs, right side (the left is mirrored). Each ring: (centre, axis, radius across u, radius across w below, w above, squareness) -
# For the arm u starts as +Y (front/back) and w as +Z (the top of the hole becomes the outside of the arm); for the leg u starts as +X
# (outside) and w as +Y (back), which the foot's bend turns into the sole.
ARM = [
    # (One thick smooth tube from the shoulder to the hand, the owner's second reference, 2026-10-19: the reference's arm has no visible
    # elbow, shoulder or wrist; it is a soft sausage about as wide as the hand, tapering only gently, the hand growing straight out of it.)
    ((0.435, 0.000, 1.370), (1.0, 0.0, -0.9), 0.172, 0.172, 0.172, 2.0),   # shoulder cap
    ((0.505, 0.000, 1.295), (0.5, 0.0, -1.0), 0.17, 0.17, 0.17, 2.0),
    ((0.555, 0.000, 1.230), (0.4, 0.0, -1.0), 0.166, 0.166, 0.166, 2.0),
    ((0.580, -0.002, 1.170), (0.4, -0.02, -1.0), 0.162, 0.162, 0.162, 2.0),  # elbow loops (no narrowing: the joint does not show)
    ((0.595, -0.003, 1.130), (0.4, -0.02, -1.0), 0.16, 0.16, 0.16, 2.0),
    ((0.610, -0.004, 1.090), (0.4, -0.02, -1.0), 0.157, 0.157, 0.157, 2.0),
    ((0.640, -0.007, 1.020), (0.4, -0.03, -1.0), 0.15, 0.15, 0.15, 2.0),
    ((0.665, -0.011, 0.955), (0.4, -0.03, -1.0), 0.143, 0.143, 0.143, 2.0),
    ((0.680, -0.015, 0.905), (0.4, -0.03, -1.0), 0.136, 0.136, 0.136, 2.0),  # wrist, as thick as the hand's base
    ((0.698, -0.018, 0.845), (0.4, -0.03, -1.0), 0.130, 0.114, 0.114, 2.6),  # fist
    ((0.720, -0.020, 0.780), (0.4, -0.03, -1.0), 0.130, 0.114, 0.114, 2.6),
]
ARM_CAP = 0.045
LEG_C0 = Vector((0.235, 0.0))   # the leg's middle where it leaves the torso (x, y)
LEG = [
    # (Thighs fuller, tapering to the knee, the character pass: even tubes read as sticks.)
    ((0.235, 0.000, 0.780), (-0.02, 0.0, -1.0), 0.190, 0.190, 0.190, 2.0),
    ((0.240, 0.000, 0.660), (0.0, 0.0, -1.0), 0.184, 0.184, 0.184, 2.0),
    ((0.245, 0.000, 0.560), (0.0, 0.0, -1.0), 0.170, 0.170, 0.170, 2.0),
    ((0.248, -0.004, 0.490), (0.0, 0.0, -1.0), 0.157, 0.157, 0.157, 2.0),  # knee loops
    ((0.249, -0.006, 0.445), (0.0, 0.0, -1.0), 0.155, 0.155, 0.155, 2.0),
    ((0.250, -0.004, 0.400), (0.0, 0.0, -1.0), 0.155, 0.155, 0.155, 2.0),
    ((0.251, 0.000, 0.300), (0.0, 0.0, -1.0), 0.152, 0.152, 0.152, 2.0),
    ((0.252, 0.010, 0.185), (0.01, 0.0, -1.0), 0.148, 0.148, 0.148, 2.0),  # ankle
    ((0.258, 0.035, 0.118), (0.03, -0.25, -1.0), 0.200, 0.205, 0.220, 2.2),  # heel and instep
    ((0.265, -0.105, 0.108), (0.05, -1.0, -0.05), 0.212, 0.135, 0.124, 2.4),  # foot
    ((0.272, -0.215, 0.105), (0.06, -1.0, 0.05), 0.198, 0.128, 0.118, 2.4),
]
FOOT_CAP = 0.05


class Cage:
    def __init__(self):
        self.verts, self.faces = [], []

    def v(self, co):
        self.verts.append(Vector(co))
        return len(self.verts) - 1

    def bridge(self, a, b):
        n = len(a)
        for k in range(n):
            self.faces.append((a[k], a[(k + 1) % n], b[(k + 1) % n], b[k]))

    def cap(self, loop, na, nb, axis, rise):
        """Closes a loop of 2(na+nb) edges with an na x nb grid of quads (a Coons patch of its sides) domed `rise` along `axis`."""
        assert len(loop) == 2 * (na + nb)

        # The grid's corners go where they span the most area (the loop's diagonals, not its top, side, bottom and side, which
        # would fold the grid into a diamond).
        def corner_area(r):
            c = [self.verts[loop[(r + k) % len(loop)]] for k in (0, na, na + nb, 2 * na + nb)]
            return ((c[2] - c[0]).cross(c[3] - c[1])).length

        r = max(range(len(loop)), key=corner_area)
        loop = loop[r:] + loop[:r]
        P = [self.verts[i] for i in loop]
        bottom = [P[s] for s in range(na + 1)]
        right = [P[na + t] for t in range(nb + 1)]
        top = [P[(2 * na + nb - s) % len(P)] for s in range(na + 1)]
        left = [P[(2 * na + 2 * nb - t) % len(P)] for t in range(nb + 1)]
        grid = {}
        for s in range(na + 1):
            for t in range(nb + 1):
                if s in (0, na) or t in (0, nb):
                    if t == 0:
                        idx = loop[s]
                    elif s == na:
                        idx = loop[na + t]
                    elif t == nb:
                        idx = loop[(2 * na + nb - s) % len(loop)]
                    else:
                        idx = loop[(2 * na + 2 * nb - t) % len(loop)]
                    grid[s, t] = idx
                    continue
                u, w = s / na, t / nb
                p = ((1 - w) * bottom[s] + w * top[s] + (1 - u) * left[t] + u * right[t]
                     - ((1 - u) * (1 - w) * P[0] + u * (1 - w) * P[na] + u * w * P[na + nb] + (1 - u) * w * P[2 * na + nb]))
                p = p + axis.normalized() * rise * math.sin(math.pi * u) * math.sin(math.pi * w)
                grid[s, t] = self.v(p)
        for s in range(na):
            for t in range(nb):
                self.faces.append((grid[s, t], grid[s + 1, t], grid[s + 1, t + 1], grid[s, t + 1]))


def superellipse(c, s, e):
    """cos/sin pushed towards a rounded square as e grows past 2."""
    k = 2.0 / e
    return math.copysign(abs(c) ** k, c), math.copysign(abs(s) ** k, s)


def tube(cage, start, rings, n0, u0, w0, centre0, side):
    """Rings after the loop `start` (vertex indices). Each ring keeps the loop's angles (about centre0 in the u0/w0 plane), eased to an
    even spacing over the first two rings. side = -1 mirrors the ring specs across X."""
    psi0 = []
    for i in start:
        d = cage.verts[i] - centre0
        psi0.append(math.atan2(d.dot(w0), d.dot(u0)))
    n = len(start)
    # The even spacing that starts where the loop's first angle does and runs the same way round.
    turn = 1.0 if sum(math.sin(psi0[(k + 1) % n] - psi0[k]) for k in range(n)) > 0 else -1.0
    even = [psi0[0] + turn * 2 * math.pi * k / n for k in range(n)]
    prev = list(start)
    loops = [prev]
    for j, (c, ax, ru, rwn, rwp, e) in enumerate(rings):
        c = Vector((c[0] * side, c[1], c[2]))
        ax = Vector((ax[0] * side, ax[1], ax[2])).normalized()
        q = n0.rotation_difference(ax)
        u, w = q @ u0, q @ w0
        mix = min(1.0, (j + 1) / 2.0)
        ring = []
        for k in range(n):
            a0, a1 = psi0[k], even[k]
            a1 = a0 + math.atan2(math.sin(a1 - a0), math.cos(a1 - a0))
            a = a0 + (a1 - a0) * mix
            cu, sw = superellipse(math.cos(a), math.sin(a), e)
            rw = rwp if sw > 0 else rwn
            ring.append(cage.v(c + u * ru * cu + w * rw * sw))
        cage.bridge(prev, ring)
        prev = ring
        loops.append(ring)
    return loops


def build_cage():
    cage = Cage()
    T = []
    for (z, rx, ryf, ryb, cy) in TORSO:
        row = []
        for i in range(TSEG):
            th = 2 * math.pi * i / TSEG
            s = math.sin(th)
            row.append(cage.v((rx * math.cos(th), cy + (ryb if s > 0 else ryf) * s, z)))
        T.append(row)
    # Torso faces, leaving a 2x2 patch open on each side for the arms.
    a, b = HOLE_RINGS[0], HOLE_RINGS[2]
    open_segs = {(TSEG - 1) % TSEG, 0, TSEG // 2 - 1, TSEG // 2}
    for k in range(len(T) - 1):
        for i in range(TSEG):
            if a <= k < b and i in open_segs:
                continue
            j = (i + 1) % TSEG
            cage.faces.append((T[k][i], T[k][j], T[k + 1][j], T[k + 1][i]))
    m = HOLE_RINGS[1]
    holes = {
        1: [T[a][TSEG - 1], T[a][0], T[a][1], T[m][1], T[b][1], T[b][0], T[b][TSEG - 1], T[m][TSEG - 1]],
        -1: [T[a][TSEG // 2 + 1], T[a][TSEG // 2], T[a][TSEG // 2 - 1], T[m][TSEG // 2 - 1], T[b][TSEG // 2 - 1], T[b][TSEG // 2],
             T[b][TSEG // 2 + 1], T[m][TSEG // 2 + 1]],
    }
    # Neck and head.
    prev = T[-1]
    for (z, rx, ry) in NECK:
        row = [cage.v((rx * math.cos(2 * math.pi * i / TSEG), ry * math.sin(2 * math.pi * i / TSEG), z)) for i in range(TSEG)]
        cage.bridge(prev, row)
        prev = row
    for phi in HEAD_PHI:
        p = math.radians(phi)
        r, z = HEAD_R * math.sin(p), HEAD_C.z - HEAD_R * math.cos(p)
        row = [cage.v((HEAD_C.x + r * math.cos(2 * math.pi * i / TSEG), HEAD_C.y + r * math.sin(2 * math.pi * i / TSEG), z))
               for i in range(TSEG)]
        cage.bridge(prev, row)
        prev = row
    cage.cap(prev, 4, 4, Z, HEAD_TOP)
    # Arms and fists.
    for side in (1, -1):
        hole = holes[side]
        c0 = sum((cage.verts[i] for i in hole), Vector()) / len(hole)
        loops = tube(cage, hole, ARM, X * side, Y, Z, c0, side)
        last = loops[-1]
        ax = Vector((ARM[-1][1][0] * side, ARM[-1][1][1], ARM[-1][1][2]))
        cage.cap(last, 2, 2, ax, ARM_CAP)
    # Legs and feet: the bottom ring split by the crotch vertex.
    crotch = cage.v((0.0, CROTCH[0], CROTCH[1]))
    q = TSEG // 4
    legs = {
        1: [T[0][(3 * q + k) % TSEG] for k in range(2 * q + 1)] + [crotch],
        -1: [T[0][(3 * q - k) % TSEG] for k in range(2 * q + 1)] + [crotch],
    }
    for side in (1, -1):
        c0 = Vector((LEG_C0.x * side, LEG_C0.y, TORSO[0][0]))
        loops = tube(cage, legs[side], LEG, -Z, X * side, Y, c0, side)
        ax = Vector((LEG[-1][1][0] * side, LEG[-1][1][1], LEG[-1][1][2]))
        cage.cap(loops[-1], 2, 3, ax, FOOT_CAP)
    return cage


# The character judge's round 7 (2026-10-19), after the owner's reference cast: thighs shorter (the reference's scouts stand on stubby legs,
# shorts nearly meeting the socks), the upper body a broader block (wide sloping shoulders, nearly as wide as the head) and the hips a
# little narrower, so the shirts stop reading pear-shaped. Applied to the cage as a whole, so every ring and loop keeps its place.
KNEE_Z, CROTCH_Z = 0.445, 0.80
THIGH_DROP = 0.05          # how much shorter the thighs are; everything above them comes down by it
SHOULDER_WIDEN = 1.1       # across the chest and shoulders
HIP_NARROW = 0.95          # across the hips
TORSO_CORE = 0.36          # inside this distance from the middle a point is scaled; outside it (the sides, the arms) shifted


def _reshape(co):
    x, y, z = co
    if z > KNEE_Z:
        z -= THIGH_DROP * min(1.0, (z - KNEE_Z) / (CROTCH_Z - KNEE_Z))
    # Width by height (in the original heights): the hips narrowed, ramping to the shoulders widened, the neck and head as they were.
    oz = co[2]

    def ramp(a, b, t):
        return a + (b - a) * max(0.0, min(1.0, t))
    if oz < 0.82:
        k = 1.0
    elif oz < 0.95:
        k = ramp(1.0, HIP_NARROW, (oz - 0.82) / 0.13)
    elif oz < 1.25:
        k = ramp(HIP_NARROW, SHOULDER_WIDEN, (oz - 0.95) / 0.3)
    elif oz < 1.5:
        k = SHOULDER_WIDEN
    else:
        k = ramp(SHOULDER_WIDEN, 1.0, (oz - 1.5) / 0.08)
    if abs(x) <= TORSO_CORE:
        x *= k
    else:
        x += math.copysign(TORSO_CORE * (k - 1.0), x)
    return (x, y, z)


def build(name="BaseBody", levels=SUBDIV):
    cage = build_cage()
    cage.verts = [Vector(_reshape(v)) for v in cage.verts]
    used = sorted({i for f in cage.faces for i in f})
    remap = {old: new for new, old in enumerate(used)}
    me = bpy.data.meshes.new(name)
    me.from_pydata([cage.verts[i][:] for i in used], [], [tuple(remap[i] for i in f) for f in cage.faces])
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = True
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    sd = obj.modifiers.new("subdiv", "SUBSURF")
    sd.levels = levels
    sd.render_levels = levels
    return obj


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv), 2)}
    bpy.ops.wm.read_factory_settings(use_empty=True)
    obj = build()
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.modifier_apply(modifier="subdiv")
    print("CAGE BODY", len(obj.data.polygons), "quads")
    bpy.ops.export_scene.gltf(filepath=opts["out"], use_selection=True, export_format="GLB", export_yup=True)


if __name__ == "__main__":
    main()
