"""The fighters' held weapons modelled by script (the animation overhaul, 2026-10-25: the owner asked for a modelled claw on the Claws
class's fists and for the Dagger's and Nodachi's weapons, in the same chunky, rounded style as the modelled gear, make_gear.py, and the
stage props). Each piece is a few shaped parts named by what they are made of, so the game colours them:
  metal*    the blade or claw steel           band*    the class's accent band (wrap, cuff)
  leather*  brown leather                     wood*    a wooden grip
Built in the hand's space as the game holds it: the weapon points along +X from the wrist (Blender X), the back of the hand is up
(Blender +Z, the game's +Y), and the hand's middle is at the origin. The game hangs it on the hand bone and turns +X along the forearm
(fighter_view.gd `_mount_weapon_models`).

Run:  blender --background --python art/blender/make_weapons.py [-- --only claw] [--preview <folder>]
Writes godot/models/weapons/<name>.glb (then `Godot --headless --path godot --import`). Keep each within the prop budget
(docs/ART_WORKFLOW.md: 3,000 triangles).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

OUT = "godot/models/weapons"


def material(name):
    m = bpy.data.materials.get(name)
    if m is None:
        m = bpy.data.materials.new(name)
    return m


def to_object(name, bm, smooth=True, sharp=50.0):
    me = bpy.data.meshes.new(name)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.normal_update()
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    obj.data.materials.append(material("m_" + name.split("_")[0]))
    for p in obj.data.polygons:
        p.use_smooth = smooth
    if smooth:
        obj.data.set_sharp_from_angle(angle=math.radians(sharp))
    return obj


def empty(name):
    e = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(e)
    return e


def tube(bm, points, radii, sides=10, flat=1.0, cap_end=True):
    """A tube along `points` (Vectors) with a radius per point, its cross-section squashed to `flat` across Z (a blade's edge), closed
    at the start and pointed or closed at the end."""
    rings = []
    for i, p in enumerate(points):
        a = points[min(i + 1, len(points) - 1)] - points[max(i - 1, 0)]
        a.normalize()
        side = a.cross(Vector((0.0, 0.0, 1.0)))
        if side.length < 1e-4:
            side = Vector((0.0, 1.0, 0.0))
        side.normalize()
        up = side.cross(a)
        ring = []
        for k in range(sides):
            t = math.tau * k / sides
            ring.append(bm.verts.new(p + side * math.cos(t) * radii[i] + up * math.sin(t) * radii[i] * flat))
        rings.append(ring)
    for r0, r1 in zip(rings, rings[1:]):
        for k in range(sides):
            bm.faces.new((r0[k], r0[(k + 1) % sides], r1[(k + 1) % sides], r1[k]))
    bm.faces.new(list(reversed(rings[0])))
    if cap_end:
        bm.faces.new(rings[-1])
    return rings


def rounded_box(bm, centre, size, bevel=0.35, segments=3):
    """A box with well rounded edges (a chunky plate)."""
    res = bmesh.ops.create_cube(bm, size=1.0)
    verts = res["verts"]
    for v in verts:
        v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2])) + centre
    bmesh.ops.bevel(bm, geom=list(bm.edges), offset=min(size) * bevel, segments=segments, affect="EDGES", profile=0.5)


def claw():
    """The Claws class's claw, one on each fist: a rounded steel plate over the knuckles, three thick curved claws out past the fist
    (stubby and rounded at the root, tapering to a blunt hook), and a wrap band round the wrist in the class colour. About 0.33 long
    past the knuckles, so the claw tips, not the fist, meet the hit."""
    root = empty("claw")
    bm = bmesh.new()
    # The knuckle plate, on the back of the hand.
    rounded_box(bm, Vector((0.07, 0.0, 0.095)), (0.2, 0.2, 0.06), bevel=0.45)
    to_object("metal_plate", bm).parent = root
    bm = bmesh.new()
    for y, lift, length in ((-0.072, 0.0, 0.9), (0.0, 0.012, 1.0), (0.072, 0.0, 0.9)):
        pts, radii = [], []
        n = 9
        for i in range(n):
            k = i / (n - 1)
            # Out along X, rising a little, then hooking down at the tip.
            x = 0.1 + 0.34 * length * k
            z = 0.075 + lift + 0.055 * math.sin(k * math.pi * 0.8) - 0.075 * k ** 3
            pts.append(Vector((x, y * (1.0 + 0.2 * k), z)))
            # (Chunky: thick and round at the root, tapering to a blunt rounded hook, not a needle.)
            radii.append(0.05 * (1.0 - 0.72 * k ** 1.5))
        tube(bm, pts, radii, sides=12, flat=0.8)
    to_object("metal_claws", bm).parent = root
    # The wrap at the wrist.
    bm = bmesh.new()
    pts = [Vector((-0.1, 0.0, 0.0)), Vector((-0.05, 0.0, 0.0)), Vector((0.0, 0.0, 0.0))]
    tube(bm, pts, [0.135, 0.145, 0.135], sides=20, flat=1.0)
    to_object("band_wrap", bm).parent = root
    return root


def _blade(bm, x0, x1, width, thick, curve=0.0, tip=0.25, steps=14):
    """A chunky blade along X from x0 to x1: a lens cross-section (thick spine, rounded edge) `width` tall in Z and `thick` across Y,
    bowed up by `curve` at the middle, narrowing to a rounded point over its last `tip` share."""
    rings = []
    sides = 10
    for i in range(steps + 1):
        k = i / steps
        x = x0 + (x1 - x0) * k
        z = curve * 4.0 * k * (1.0 - k) + curve * 0.6 * k
        w = width * (1.0 if k < 1.0 - tip else max(0.06, ((1.0 - k) / tip) ** 0.7))
        ring = []
        for j in range(sides):
            t = math.tau * j / sides
            ring.append(bm.verts.new((x, math.sin(t) * thick * 0.5 * (0.35 + 0.65 * abs(math.cos(t))), z + math.cos(t) * w * 0.5)))
        rings.append(ring)
    for r0, r1 in zip(rings, rings[1:]):
        for j in range(sides):
            bm.faces.new((r0[j], r0[(j + 1) % sides], r1[(j + 1) % sides], r1[j]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])


def _grip(bm, x0, x1, r, wraps=0):
    pts = [Vector((x0 + (x1 - x0) * k / 6.0, 0.0, 0.0)) for k in range(7)]
    radii = [r * (1.0 + (0.12 * math.sin(k * math.pi) if wraps else 0.0)) for k in range(7)]
    tube(bm, pts, radii, sides=12)


def dagger():
    """The Dagger class's knife, held in the weapon hand (hung on the hand bone like the claw, along the forearm's line): a short broad
    leaf blade with a thick spine and a rounded point, a round guard, a wrapped grip in the fist and a ball pommel. About 0.5 of blade
    past the fist, so the knife's point meets the jab's far hitboxes."""
    root = empty("dagger")
    bm = bmesh.new()
    _blade(bm, 0.1, 0.62, 0.16, 0.05, curve=0.0, tip=0.38)
    to_object("metal_blade", bm).parent = root
    bm = bmesh.new()
    rounded_box(bm, Vector((0.08, 0.0, 0.0)), (0.05, 0.12, 0.26), bevel=0.45)
    to_object("metal_guard", bm, sharp=60.0).parent = root
    bm = bmesh.new()
    _grip(bm, -0.16, 0.06, 0.045, wraps=1)
    to_object("band_grip", bm).parent = root
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=14, v_segments=10, radius=0.06)
    bmesh.ops.translate(bm, vec=Vector((-0.19, 0.0, 0.0)), verts=bm.verts)
    to_object("metal_pommel", bm).parent = root
    return root


def nodachi():
    """The Nodachi class's very long sword, laid out like the game's longsword so it is posed and stretched on the same weapon pivot
    (fighter_view.gd `blade_pivot`: the blade from 0.2 to 2.9 along X, the hand at 0): a long slim blade with a gentle upward curve and a
    chunky rounded spine, a round guard, and a long two-handed grip wrapped in the class colour with a cap."""
    root = empty("nodachi")
    bm = bmesh.new()
    _blade(bm, 0.18, 2.9, 0.15, 0.06, curve=0.08, tip=0.12, steps=24)
    to_object("metal_blade", bm).parent = root
    bm = bmesh.new()
    pts = [Vector((0.1, 0.0, 0.0)), Vector((0.16, 0.0, 0.0))]
    tube(bm, pts, [0.16, 0.16], sides=20)
    to_object("metal_guard", bm).parent = root
    bm = bmesh.new()
    _grip(bm, -0.62, 0.1, 0.06, wraps=1)
    to_object("band_grip", bm).parent = root
    bm = bmesh.new()
    tube(bm, [Vector((-0.7, 0.0, 0.0)), Vector((-0.6, 0.0, 0.0))], [0.07, 0.075], sides=14)
    to_object("metal_cap", bm).parent = root
    return root


WEAPONS = {"claw": claw, "dagger": dagger, "nodachi": nodachi}


def export(root, path):
    bpy.ops.object.select_all(action="DESELECT")
    root.select_set(True)
    for c in root.children_recursive:
        c.select_set(True)
    bpy.ops.export_scene.gltf(filepath=os.path.abspath(path), use_selection=True, export_format="GLB", export_yup=True,
                              export_apply=True)


def preview(root, path):
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.show_object_outline = True
    scene.render.resolution_x = 480
    scene.render.resolution_y = 480
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = 1.1
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    target = Vector((0.2, 0.0, 0.05))
    cam.location = target + Vector((0.9, -2.2, 1.3))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    only = opts["only"].split(",") if "only" in opts else list(WEAPONS)
    os.makedirs(OUT, exist_ok=True)
    for name in only:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        root = WEAPONS[name]()
        export(root, os.path.join(OUT, name + ".glb"))
        tris = sum(sum(len(p.vertices) - 2 for p in c.data.polygons) for c in root.children_recursive if c.type == "MESH")
        print("weapon %s: %d triangles" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
