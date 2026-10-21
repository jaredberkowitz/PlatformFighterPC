"""The paper house kit (docs/art/stages/HOUSES.md, the owner's go-ahead 2026-10-21): the far towns' houses in the stages' paper language,
in place of the cel-shaded cottage model (make_props.py `cottage`) that the meadow's village and farm and the beach's harbour town repeat.
Built at true size against a fighter (about 2.3 units tall to the top of the head): doors 2.5 high, sills at 1.1, a storey about 3.2.
  Types:  cottage (one storey, eaves to the front), townhouse (two storeys, gable to the front), longhouse (long and low, three windows),
          barn (gable to the front, big doors, a loft door).
  Styles: A  scored card: crisp folded walls and roof, scored lines, card frames round cut-out windows;
          B  layered card: applied timber strips, shutters, flower boxes, overlapping rows of roof card;
          C  rolled: the same massing, its edges and frames rolled paper tubes (the props' language, make_paper_props.py style C).
Every part names its family (painted by the game, `StageArt._paper` with each house's roof and wall colours as tints), with its occlusion
baked. Each house is `prop_house_<type>`, its foot at the origin, its front toward +z (the camera).

Run (Blender 5.2):
  blender --background --python art/blender/make_paper_houses.py -- --style A --out <file.glb> [--render <sheet.png>]
(`--render`: the four houses side by side in Blender's studio light, for the review page.)
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import make_paper_props as kit  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
STYLE = opts.get("style", "A").upper()
PARENT = None


def P(x, y, z):
    """Game (x right, y up, z toward the camera) to Blender (z up, -y toward the camera)."""
    return Vector((x, -z, y))


def solid(name, fam, verts, faces, smooth=False):
    bm = bmesh.new()
    vs = [bm.verts.new(P(*v)) for v in verts]
    for f in faces:
        bm.faces.new([vs[i] for i in f])
    return kit.to_object(name, bm, fam, PARENT, smooth=smooth)


def box(fam, x0, x1, y0, y1, z0, z1, name=None):
    v = [(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1), (x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)]
    f = [(0, 1, 2, 3), (4, 7, 6, 5), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)]
    return solid(name or fam, fam, v, f)


def tube(fam, a, b, r, segs=8, name=None):
    bm = bmesh.new()
    kit.tube(bm, P(*a), P(*b), r, segs=segs)
    return kit.to_object(name or fam, bm, fam, PARENT, smooth=True)


def ring_arc(fam, centre, R, r, arc, start, name=None):
    """A rolled tube bent into an arc in the front plane (an arched door's head), `start` its first angle."""
    pts = []
    n = 10
    for k in range(n + 1):
        a = start + arc * k / n
        pts.append((centre[0] + R * math.cos(a), centre[1] + R * math.sin(a), centre[2]))
    bm = bmesh.new()
    for p, q in zip(pts, pts[1:]):
        kit.tube(bm, P(*p), P(*q), r, segs=6)
    return kit.to_object(name or fam, bm, fam, PARENT, smooth=True)


class Frame:
    """Maps a house's own (along, up, across) to the game's (x, y, z): along the ridge is x for eaves to the front, z for a gable to the
    front."""

    def __init__(self, gable_front):
        self.g = gable_front

    def p(self, a, y, c):
        return (c, y, a) if self.g else (a, y, c)


def roof(W, D, H, pitch, gable_front, ov=0.55):
    """A folded card roof over the walls, `ov` over every edge, its gables filled with wall; returns the ridge's height."""
    f = Frame(gable_front)
    along = D if gable_front else W
    across = W if gable_front else D
    t = math.tan(math.radians(pitch))
    rise = across / 2.0 * t
    run = across / 2.0 + ov
    th = 0.14
    la = along / 2.0 + ov
    for s in (-1.0, 1.0):
        top = (0.0, H + rise)
        eave = (s * run, H + rise - run * t)
        n = Vector((s * t, 1.0)).normalized()
        v = []
        for a in (-la, la):
            for (c, y) in (top, eave):
                v.append(f.p(a, y, c))
                v.append(f.p(a, y - th / n.y, c))
        # v: [a0 top, a0 top-under, a0 eave, a0 eave-under, a1 top, a1 top-under, a1 eave, a1 eave-under]
        faces = [(0, 2, 6, 4), (1, 5, 7, 3), (0, 4, 5, 1), (2, 3, 7, 6), (0, 1, 3, 2), (4, 6, 7, 5)]
        solid("roof", "roof", v, faces)
        roof_details(f, s, top, eave, la, th, t)
    # The gables, filled to the roof's underside.
    for a in (-along / 2.0, along / 2.0):
        inset = 0.06 if a > 0 else -0.06
        v = [f.p(a, H, -across / 2.0), f.p(a, H, across / 2.0), f.p(a, H + rise - 0.1, 0.0),
             f.p(a - inset, H, -across / 2.0), f.p(a - inset, H, across / 2.0), f.p(a - inset, H + rise - 0.1, 0.0)]
        solid("wall_gable", "wall", v, [(0, 1, 2), (3, 5, 4), (0, 3, 4, 1), (1, 4, 5, 2), (2, 5, 3, 0)])
    if STYLE == "C":
        tube("roof_edge", f.p(-la, H + rise + 0.05, 0.0), f.p(la, H + rise + 0.05, 0.0), 0.16, segs=10, name="roof_ridge")
    else:
        box("roof_edge", *box_args(f, -la, la, H + rise - 0.02, H + rise + 0.16, -0.12, 0.12), name="roof_ridge")
    return H + rise


def box_args(f, a0, a1, y0, y1, c0, c1):
    """The game box of a house box (along a0..a1, up y0..y1, across c0..c1)."""
    p0 = f.p(a0, y0, c0)
    p1 = f.p(a1, y1, c1)
    return (min(p0[0], p1[0]), max(p0[0], p1[0]), y0, y1, min(p0[2], p1[2]), max(p0[2], p1[2]))


def roof_details(f, s, top, eave, la, th, t):
    """Each style's roof surface: A scored rows, B overlapping card rows with a lit lower edge, C a rolled eave."""
    if STYLE == "A":
        for k in (1, 2, 3):
            u = k / 4.0
            c = top[0] + (eave[0] - top[0]) * u
            y = top[1] + (eave[1] - top[1]) * u + 0.01
            v = []
            for a in (-la, la):
                v.append(f.p(a, y, c - s * 0.04))
                v.append(f.p(a, y, c + s * 0.04))
                v.append(f.p(a, y + 0.03, c))
            solid("roof_score", "roof_score", v, [(0, 3, 4, 1), (1, 4, 5, 2), (2, 5, 3, 0), (0, 1, 2), (3, 5, 4)])
    elif STYLE == "B":
        rows = 4
        for k in range(rows):
            u0 = k / rows
            u1 = (k + 1) / rows + 0.04
            c0 = top[0] + (eave[0] - top[0]) * u0
            c1 = top[0] + (eave[0] - top[0]) * min(u1, 1.0)
            y0 = top[1] + (eave[1] - top[1]) * u0 + 0.06
            y1 = top[1] + (eave[1] - top[1]) * min(u1, 1.0) + 0.06
            v = []
            for a in (-la, la):
                v += [f.p(a, y0, c0), f.p(a, y1, c1), f.p(a, y1 - 0.1, c1), f.p(a, y0 - 0.04, c0)]
            solid("roof_row", "roof", v, [(0, 1, 5, 4), (3, 7, 6, 2), (0, 4, 7, 3), (1, 2, 6, 5), (0, 3, 2, 1), (4, 5, 6, 7)])
            # The row's lower edge, lit as a card edge catches the light.
            box("roof_edge", *box_args(f, -la, la, y1 - 0.11, y1 + 0.02, c1 - 0.05, c1 + 0.05), name="roof_lip")
    else:
        tube("roof_edge", f.p(-la, eave[1] - 0.04, eave[0]), f.p(la, eave[1] - 0.04, eave[0]), 0.13, segs=10, name="roof_eave")


def walls(W, D, H, gable_front, storeys=1):
    box("stone", -W / 2.0 - 0.15, W / 2.0 + 0.15, -0.8, 0.35, -D / 2.0 - 0.15, D / 2.0 + 0.15, name="footing")
    box("wall", -W / 2.0, W / 2.0, 0.35, H, -D / 2.0, D / 2.0, name="wall")
    fz = D / 2.0
    if STYLE == "A":
        # Scored card: a crisp strip down each front corner and one along the floor line (and the storey's).
        for sx in (-1.0, 1.0):
            box("trim", sx * W / 2.0 - 0.12, sx * W / 2.0 + 0.12, 0.35, H, fz - 0.12, fz + 0.04, name="trim_corner")
        for y in [0.35] + [0.35 + (H - 0.35) * k / storeys for k in range(1, storeys)]:
            box("trim", -W / 2.0, W / 2.0, y, y + 0.16, fz, fz + 0.04, name="trim_line")
    elif STYLE == "B":
        # Layered card: timber strips applied on the walls, corners, the floor line, the eaves, braces in the end bays.
        for sx in (-1.0, 1.0):
            for sz in (-1.0, 1.0):
                box("timber", sx * W / 2.0 - 0.14, sx * W / 2.0 + 0.14, 0.35, H, sz * D / 2.0 - 0.14, sz * D / 2.0 + 0.14, name="timber_post")
        for y in [0.35, H - 0.2] + [0.35 + (H - 0.35) * k / storeys for k in range(1, storeys)]:
            box("timber", -W / 2.0, W / 2.0, y, y + 0.2, fz, fz + 0.08, name="timber_beam")
    else:
        # Rolled: corner posts and the floor line as rolled tubes.
        for sx in (-1.0, 1.0):
            tube("timber", (sx * W / 2.0, 0.35, fz), (sx * W / 2.0, H, fz), 0.13, name="timber_post")
        for y in [0.42] + [0.35 + (H - 0.35) * k / storeys for k in range(1, storeys)]:
            tube("timber", (-W / 2.0, y, fz + 0.04), (W / 2.0, y, fz + 0.04), 0.1, name="timber_beam")


def window(x, y, w, h, fz):
    """A lit window in the front wall at (x, y) its centre."""
    box("glow", x - w / 2.0, x + w / 2.0, y - h / 2.0, y + h / 2.0, fz - 0.02, fz + 0.02, name="glow_window")
    if STYLE == "C":
        for (a, b) in (((x - w / 2.0, y - h / 2.0), (x + w / 2.0, y - h / 2.0)), ((x - w / 2.0, y + h / 2.0), (x + w / 2.0, y + h / 2.0)),
                       ((x - w / 2.0, y - h / 2.0), (x - w / 2.0, y + h / 2.0)), ((x + w / 2.0, y - h / 2.0), (x + w / 2.0, y + h / 2.0))):
            tube("trim", (a[0], a[1], fz + 0.05), (b[0], b[1], fz + 0.05), 0.07, segs=6, name="trim_frame")
        tube("trim", (x, y - h / 2.0, fz + 0.05), (x, y + h / 2.0, fz + 0.05), 0.035, segs=5, name="trim_bar")
        tube("timber", (x - w / 2.0 - 0.15, y - h / 2.0 - 0.1, fz + 0.12), (x + w / 2.0 + 0.15, y - h / 2.0 - 0.1, fz + 0.12), 0.09, segs=6,
             name="timber_sill")
        return
    t = 0.12
    for (x0, x1, y0, y1) in ((x - w / 2.0 - t, x + w / 2.0 + t, y - h / 2.0 - t, y - h / 2.0), (x - w / 2.0 - t, x + w / 2.0 + t, y + h / 2.0, y + h / 2.0 + t),
                             (x - w / 2.0 - t, x - w / 2.0, y - h / 2.0, y + h / 2.0), (x + w / 2.0, x + w / 2.0 + t, y - h / 2.0, y + h / 2.0)):
        box("trim", x0, x1, y0, y1, fz, fz + 0.06, name="trim_frame")
    box("trim", x - 0.03, x + 0.03, y - h / 2.0, y + h / 2.0, fz, fz + 0.05, name="trim_bar")
    if STYLE == "B":
        for sx in (-1.0, 1.0):
            box("shutter", x + sx * (w / 2.0 + t) - (0.0 if sx > 0 else w * 0.48), x + sx * (w / 2.0 + t) + (w * 0.48 if sx > 0 else 0.0),
                y - h / 2.0, y + h / 2.0, fz + 0.02, fz + 0.08, name="shutter")
        box("timber", x - w / 2.0 - 0.1, x + w / 2.0 + 0.1, y - h / 2.0 - 0.42, y - h / 2.0 - 0.12, fz, fz + 0.32, name="timber_box")
        for k in range(4):
            px = x - w / 2.0 + (k + 0.5) * w / 4.0
            box("petal", px - 0.11, px + 0.11, y - h / 2.0 - 0.14, y - h / 2.0 + 0.06, fz + 0.08, fz + 0.28, name="petal")


def door(x, w, h, fz):
    box("stone", x - w / 2.0 - 0.3, x + w / 2.0 + 0.3, -0.2, 0.35, fz - 0.05, fz + 0.55, name="stone_step")
    box("door", x - w / 2.0, x + w / 2.0, 0.35, 0.35 + h, fz - 0.02, fz + 0.04, name="door")
    if STYLE == "C":
        tube("timber", (x - w / 2.0 - 0.05, 0.35, fz + 0.08), (x - w / 2.0 - 0.05, 0.35 + h - w / 2.0, fz + 0.08), 0.08, segs=6, name="timber_jamb")
        tube("timber", (x + w / 2.0 + 0.05, 0.35, fz + 0.08), (x + w / 2.0 + 0.05, 0.35 + h - w / 2.0, fz + 0.08), 0.08, segs=6, name="timber_jamb")
        ring_arc("timber", (x, 0.35 + h - w / 2.0, fz + 0.08), w / 2.0 + 0.05, 0.08, math.pi, 0.0, name="timber_arch")
        return
    t = 0.14
    box("trim", x - w / 2.0 - t, x - w / 2.0, 0.35, 0.35 + h + t, fz, fz + 0.08, name="trim_jamb")
    box("trim", x + w / 2.0, x + w / 2.0 + t, 0.35, 0.35 + h + t, fz, fz + 0.08, name="trim_jamb")
    box("trim", x - w / 2.0 - t, x + w / 2.0 + t, 0.35 + h, 0.35 + h + t, fz, fz + 0.08, name="trim_head")
    if STYLE == "B":
        box("glow", x + w / 2.0 + 0.3, x + w / 2.0 + 0.55, 2.0, 2.3, fz + 0.04, fz + 0.3, name="glow_lamp")


def chimney(x, z, y0, top):
    if STYLE == "C":
        tube("stone", (x, y0, z), (x, top, z), 0.36, segs=10, name="chimney")
        tube("stone", (x, top - 0.05, z), (x, top + 0.2, z), 0.46, segs=10, name="chimney_rim")
    else:
        box("stone", x - 0.38, x + 0.38, y0, top, z - 0.38, z + 0.38, name="chimney")
        box("stone", x - 0.48, x + 0.48, top, top + 0.18, z - 0.48, z + 0.48, name="chimney_cap")


def cottage():
    W, D, H = 5.0, 3.8, 3.6
    walls(W, D, H, False)
    ridge = roof(W, D, H, 40.0, False)
    door(-1.15, 1.15, 2.5, D / 2.0)
    window(1.1, 1.75, 1.15, 1.1, D / 2.0)
    chimney(1.6, -0.8, H, ridge + 0.9)


def townhouse():
    W, D, H = 3.8, 4.6, 6.6
    walls(W, D, H, True, storeys=2)
    roof(W, D, H, 50.0, True)
    door(-0.75, 1.1, 2.5, D / 2.0)
    window(0.95, 1.75, 0.9, 1.1, D / 2.0)
    for x in (-0.85, 0.95):
        window(x, 4.7, 0.9, 1.2, D / 2.0)
    chimney(-1.0, -1.4, H, H + 2.6)


def longhouse():
    W, D, H = 7.6, 4.0, 3.4
    walls(W, D, H, False)
    ridge = roof(W, D, H, 34.0, False)
    door(0.0, 1.2, 2.5, D / 2.0)
    for x in (-2.5, 2.5):
        window(x, 1.75, 1.2, 1.1, D / 2.0)
    chimney(-2.9, -0.7, H, ridge + 0.8)


def barn():
    # (Low, as the barn it replaces: taller, with its gable to the front, it crowded the frame's corner beside the farmhouse.)
    W, D, H = 5.6, 6.0, 3.4
    box("stone", -W / 2.0 - 0.15, W / 2.0 + 0.15, -0.8, 0.35, -D / 2.0 - 0.15, D / 2.0 + 0.15, name="footing")
    box("barn", -W / 2.0, W / 2.0, 0.35, H, -D / 2.0, D / 2.0, name="wall_barn")
    roof(W, D, H, 40.0, True)
    fz = D / 2.0
    # The big doors under a white surround, braced in a white X; a loft door in the gable.
    box("barn_trim", -1.35, 1.35, 0.35, 3.15, fz, fz + 0.05, name="trim_doors")
    box("barn_door", -1.15, 1.15, 0.35, 2.95, fz + 0.03, fz + 0.09, name="door_barn")
    for s in (-1.0, 1.0):
        if STYLE == "C":
            tube("barn_trim", (-1.05 * s, 0.5, fz + 0.12), (1.05 * s, 3.0, fz + 0.12), 0.07, segs=6, name="trim_brace")
        else:
            v = [(-1.05 * s - 0.08, 0.5, fz + 0.09), (-1.05 * s + 0.08, 0.5, fz + 0.09), (1.05 * s + 0.08, 3.0, fz + 0.09), (1.05 * s - 0.08, 3.0, fz + 0.09)]
            v += [(p[0], p[1], p[2] + 0.05) for p in v]
            solid("trim_brace", "barn_trim", v, [(0, 1, 2, 3), (4, 7, 6, 5), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)])
    box("barn_trim", -0.6, 0.6, H + 0.25, H + 1.35, fz + 0.05, fz + 0.1, name="trim_loft")
    box("shadow", -0.45, 0.45, H + 0.37, H + 1.23, fz + 0.08, fz + 0.12, name="shadow_loft")
    if STYLE != "C":
        for sx in (-1.0, 1.0):
            box("barn_trim", sx * W / 2.0 - 0.12, sx * W / 2.0 + 0.12, 0.35, H, fz - 0.12, fz + 0.06, name="trim_corner")
    else:
        for sx in (-1.0, 1.0):
            tube("barn_trim", (sx * W / 2.0, 0.35, fz), (sx * W / 2.0, H, fz), 0.13, name="trim_corner")


# The game's colours (StageArt.PAPER_COLOURS), for the studio sheet only.
PREVIEW = {"wall": (0.98, 0.93, 0.82), "roof": (0.8, 0.36, 0.3), "roof_edge": (0.92, 0.58, 0.5), "roof_score": (0.6, 0.26, 0.22),
           "trim": (0.56, 0.38, 0.26), "timber": (0.42, 0.28, 0.2), "door": (0.5, 0.32, 0.22), "shutter": (0.36, 0.52, 0.6),
           "barn": (0.78, 0.26, 0.22), "barn_door": (0.62, 0.2, 0.18), "barn_trim": (0.98, 0.95, 0.9), "shadow": (0.24, 0.16, 0.14),
           "stone": (0.74, 0.72, 0.8), "glow": (1.0, 0.8, 0.5), "petal": (1.0, 0.6, 0.65)}


def render(path):
    """The four houses in a row from the front, in the studio light, with a figure of a fighter's height (2.3) for scale."""
    scene = bpy.context.scene
    for m in bpy.data.materials:
        m.diffuse_color = (*PREVIEW.get(m.name.split(".")[0], (0.7, 0.7, 0.7)), 1.0)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=12, radius1=0.45, radius2=0.45, depth=1.4, matrix=Matrix.Translation((-5.5, 2.6, 0.7)))
    bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.55, matrix=Matrix.Translation((-5.5, 2.6, 1.75)))
    me = bpy.data.meshes.new("ruler")
    bm.to_mesh(me)
    ruler = bpy.data.objects.new("ruler", me)
    scene.collection.objects.link(ruler)
    mat = bpy.data.materials.new("ruler")
    mat.diffuse_color = (0.98, 0.78, 0.3, 1.0)
    me.materials.append(mat)
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = 62.0
    co = bpy.data.objects.new("cam", cam)
    scene.collection.objects.link(co)
    co.location = (21.0, -60.0, 14.0)
    co.rotation_euler = (math.radians(80.0), 0.0, 0.0)
    scene.camera = co
    try:
        scene.render.engine = "BLENDER_WORKBENCH"
    except TypeError:
        pass
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_cavity = True
    scene.display.shading.show_object_outline = True
    scene.render.resolution_x = 1600
    scene.render.resolution_y = 560
    scene.render.filepath = os.path.abspath(path)
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(ruler, do_unlink=True)
    bpy.data.objects.remove(co, do_unlink=True)


HOUSES = [("cottage", cottage), ("townhouse", townhouse), ("longhouse", longhouse), ("barn", barn)]


def main():
    global PARENT
    bpy.ops.wm.read_factory_settings(use_empty=True)
    roots = []
    for k, (name, build) in enumerate(HOUSES):
        # (Apart for the bake, so one house does not shade the next; back on the origin for export.)
        PARENT = kit.empty("prop_house_" + name, k * 16.0)
        build()
        roots.append(PARENT)
    # (Subdivided for the vertex bake where a face is long, so the shade has vertices to land on.)
    for o in [o for o in bpy.context.scene.objects if o.type == "MESH"]:
        bm = bmesh.new()
        bm.from_mesh(o.data)
        for _ in range(3):
            long_edges = [e for e in bm.edges if e.calc_length() > 0.6]
            if not long_edges:
                break
            bmesh.ops.subdivide_edges(bm, edges=long_edges, cuts=1, use_grid_fill=True)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        bm.to_mesh(o.data)
        bm.free()
    bpy.context.view_layer.update()
    kit.bake_occlusion(0.7, floor_z=0.0, floor_at=(24.0, 0.0), floor_size=160.0)
    if "render" in opts:
        render(opts["render"])
    for r in roots:
        r.location = (0.0, 0.0, 0.0)
    bpy.context.view_layer.update()
    tris = sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH")
    out = os.path.abspath(opts["out"])
    os.makedirs(os.path.dirname(out), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", export_yup=True, export_normals=True, export_materials="EXPORT",
                              export_vertex_color="ACTIVE")
    print("paper houses %s: %d triangles -> %s" % (STYLE, tris, out))


main()
