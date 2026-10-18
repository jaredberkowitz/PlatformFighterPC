"""The fighters' accessories modelled by script (2026-10-18, the owner: the hats built from game primitives read cheap beside the
reference cast's; modelled like the stage props, over time). Each piece is a few lathed and shaped parts with real profiles (a crown that
domes and sags, a brim that rolls, seams pressed in), named by what they are made of, so the game colours each fighter's piece:
  fabric*  the outfit's accent cloth          canvas*  white cloth
  leather* brown leather                      metal*   brass or steel
  glass*   lens glass                         straw*   woven straw (the game lays the straw weave on it)
Built in the head's design space (the game's hat pivot space before it subtracts the pivot): a head of radius 0.8 centred 1.42 up, so
its crown is at 2.22. Blender z up (exported y up); the face looks along -Y in Blender (+Z in the game).

Run:  blender --background --python art/blender/make_gear.py [-- --only sailor_cap,aviator_cap,straw_hat] [--preview <folder>]
Writes godot/models/gear/<name>.glb (then `Godot --headless --path godot --import`).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

OUT = "godot/models/gear"
HEAD_C = Vector((0.0, 0.0, 1.42))
HEAD_R = 0.8


def material(name):
    m = bpy.data.materials.get(name)
    if m is None:
        m = bpy.data.materials.new(name)
    return m


def to_object(name, bm, smooth=True, sharp=40.0):
    me = bpy.data.meshes.new(name)
    # (Faces turned outward whichever way a part was built: the aviator's shell, built from the crown down, faced in and was culled.)
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


def lathe(bm, profile, sides=40, wobble=None):
    """A surface of revolution round z from `profile` [(radius, z), ...] (bottom to top or in any order along the outline), each ring
    optionally pushed by `wobble(angle, k)` (a radius multiplier, k the profile index). Returns the rings of verts."""
    rings = []
    for k, (r, z) in enumerate(profile):
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            rr = r * (wobble(a, k) if wobble else 1.0)
            ring.append(bm.verts.new((rr * math.cos(a), rr * math.sin(a), z)))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    return rings


def cap_ring(bm, ring, z):
    """Closes a ring with a fan to a centre point at height z."""
    c = bm.verts.new((0.0, 0.0, z))
    n = len(ring)
    for i in range(n):
        bm.faces.new((ring[i], ring[(i + 1) % n], c))


def head_r(z):
    """The head's radius at height z (0 above its top)."""
    d = z - HEAD_C.z
    return math.sqrt(max(0.0, HEAD_R * HEAD_R - d * d))


def empty(name):
    e = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(e)
    return e


# ---- Sailor cap -------------------------------------------------------------------------------------------------------------------------

def sailor_cap():
    """A soft white sailor's cap: a crown that rises from the head, swells a little and domes over with a soft sag in the middle (cloth
    over no frame), a brim rolled up all round as a thick soft lip, a pressed seam round the crown's top, and a band of the outfit's
    colour just above the brim."""
    root = empty("sailor_cap")
    base = 2.02
    # The crown: from just inside the head, out and up, swelling, then over the top with a sag.
    prof = [(head_r(base) - 0.02, base - 0.04), (0.54, base + 0.02), (0.56, base + 0.12), (0.57, base + 0.24), (0.55, base + 0.31),
            (0.5, base + 0.355), (0.4, base + 0.37), (0.26, base + 0.36), (0.12, base + 0.345)]
    bm = bmesh.new()
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + (0.015 * math.sin(a * 3.0) if 2 <= k <= 4 else 0.0))
    cap_ring(bm, rings[-1], base + 0.34)
    # The seam round the top: pressed in at the shoulder.
    for v in rings[5]:
        v.co.x *= 0.985
        v.co.y *= 0.985
        v.co.z -= 0.006
    o = to_object("canvas_crown", bm)
    o.parent = root
    # The rolled brim: a thick soft lip turned up, its outline a little uneven.
    prof = [(0.5, base - 0.02), (0.6, base - 0.04), (0.68, base - 0.01), (0.7, base + 0.05), (0.66, base + 0.1), (0.58, base + 0.08),
            (0.55, base + 0.04)]
    bm = bmesh.new()
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + 0.012 * math.sin(a * 2.0 + 0.7))
    n = len(rings[0])
    for i in range(n):
        bm.faces.new((rings[-1][i], rings[-1][(i + 1) % n], rings[0][(i + 1) % n], rings[0][i]))
    o = to_object("canvas_brim", bm, sharp=60.0)
    o.parent = root
    # The band, just above the brim.
    prof = [(0.575, base + 0.09), (0.585, base + 0.11), (0.585, base + 0.17), (0.575, base + 0.19)]
    bm = bmesh.new()
    lathe(bm, prof)
    o = to_object("fabric_band", bm, sharp=60.0)
    o.parent = root
    return root


# ---- Aviator cap -----------------------------------------------------------------------------------------------------------------------

def aviator_cap():
    """A leather flying cap: a close shell over the crown and back of the head with a seam down its middle and a stitched rim, two ear
    flaps hanging at the sides, a strap under them, and goggles pushed up on the forehead (two brass rims with glass, on a leather band)."""
    root = empty("aviator_cap")
    # The shell: the head's own sphere, a little out, from the brow round the back.
    bm = bmesh.new()
    seg_u, seg_v = 40, 16
    rows = []
    for j in range(seg_v + 1):
        phi = math.radians(10.0 + 78.0 * j / seg_v)   # from the crown (10 degrees) down
        row = []
        for i in range(seg_u):
            a = math.tau * i / seg_u
            front = max(0.0, -math.sin(a))             # 1 at the face (-y)
            # Lower at the back and sides, up over the brow in front.
            low = math.radians(100.0) - math.radians(28.0) * front ** 1.5
            ph = min(phi, low) if j == seg_v else phi * (low / math.radians(88.0))
            # (Well clear of the head: the fighter's head is an oval the design sphere only approximates, and 0.045 off it sank in.)
            # (On the built cap's own ellipsoid, centred high on the head, so it clears the face shell and the oval head everywhere.)
            r = 0.87
            x = r * math.sin(ph) * math.cos(a)
            y = r * math.sin(ph) * math.sin(a)
            z = 1.74 + 0.6 * math.cos(ph)
            # The seam down the middle of the crown.
            if abs(x) < 0.03:
                x *= 0.6
                z -= 0.008
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    top = bm.verts.new((0.0, 0.0, 1.74 + 0.6))
    for i in range(seg_u):
        bm.faces.new((rows[0][i], rows[0][(i + 1) % seg_u], top))
    for j in range(seg_v):
        for i in range(seg_u):
            k = (i + 1) % seg_u
            bm.faces.new((rows[j][i], rows[j][k], rows[j + 1][k], rows[j + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=-0.03)
    o = to_object("leather_shell", bm)
    o.parent = root
    # Ear flaps: rounded tabs at the sides, curving with the head.
    for sx in (-1.0, 1.0):
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=18, v_segments=12, radius=1.0)
        for v in ret["verts"]:
            v.co = Vector((v.co.x * 0.09, v.co.y * 0.24, v.co.z * 0.3))
        bmesh.ops.translate(bm, vec=Vector((sx * 0.83, 0.04, 1.48)), verts=ret["verts"])
        o = to_object("leather_flap" + ("L" if sx < 0 else "R"), bm)
        o.parent = root
    # The goggles' band round the brow, and the goggles on it.
    bm = bmesh.new()
    lathe(bm, [(head_r(1.86) + 0.14, 1.86), (head_r(1.95) + 0.14, 1.95)])
    o = to_object("leather_band", bm, sharp=60.0)
    o.parent = root
    for sx in (-1.0, 1.0):
        centre = Vector((sx * 0.25, -(head_r(1.92) + 0.16), 1.92))
        bm = bmesh.new()
        tor = []
        sides, minor = 28, 10
        rows = []
        for i in range(sides):
            a = math.tau * i / sides
            row = []
            for j in range(minor):
                b = math.tau * j / minor
                rr = 0.15 + 0.045 * math.cos(b)
                row.append(bm.verts.new(centre + Vector((rr * math.cos(a), -0.045 * math.sin(b) - 0.02, rr * math.sin(a)))))
            rows.append(row)
        for i in range(sides):
            for j in range(minor):
                bm.faces.new((rows[i][j], rows[(i + 1) % sides][j], rows[(i + 1) % sides][(j + 1) % minor], rows[i][(j + 1) % minor]))
        o = to_object("metal_rim" + ("L" if sx < 0 else "R"), bm)
        o.parent = root
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=20, v_segments=10, radius=1.0)
        for v in ret["verts"]:
            v.co = centre + Vector((v.co.x * 0.15, v.co.y * 0.03 - 0.03, v.co.z * 0.15))
        o = to_object("glass_lens" + ("L" if sx < 0 else "R"), bm)
        o.parent = root
    return root


# ---- Straw hat --------------------------------------------------------------------------------------------------------------------------

def straw_hat():
    """A boater-ish straw hat: a wide brim that waves gently and dips a little at front and back, rolled at its edge; a crown that rises
    straight with a soft pinch at the top; a band of the outfit's colour round its foot with a little bow at the side."""
    root = empty("straw_hat")
    base = 2.0
    wave = lambda a: 0.025 * math.cos(2.0 * a) + 0.012 * math.sin(5.0 * a + 0.4)
    bm = bmesh.new()
    sides = 48
    prof_r = [0.5, 0.8, 1.05, 1.17, 1.2, 1.16, 1.06]
    prof_z = [0.04, 0.035, 0.02, 0.0, 0.03, 0.06, 0.05]
    rings = []
    for k, (r, z) in enumerate(zip(prof_r, prof_z)):
        ring = []
        for i in range(sides):
            a = math.tau * i / sides
            dz = z + wave(a) * min(1.0, r / 1.0) - (0.04 if k >= 3 else 0.0) * abs(math.sin(a))
            ring.append(bm.verts.new((r * math.cos(a), r * math.sin(a), base + dz)))
        rings.append(ring)
    for k in range(len(rings) - 1):
        for i in range(sides):
            j = (i + 1) % sides
            bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.03)
    o = to_object("straw_brim", bm, sharp=70.0)
    o.parent = root
    bm = bmesh.new()
    prof = [(0.5, base + 0.03), (0.6, base + 0.05), (0.6, base + 0.3), (0.58, base + 0.37), (0.48, base + 0.4), (0.3, base + 0.37),
            (0.12, base + 0.385)]
    rings = lathe(bm, prof, wobble=lambda a, k: 1.0 + (0.02 * math.cos(2.0 * a) if k >= 3 else 0.0))
    cap_ring(bm, rings[-1], base + 0.39)
    o = to_object("straw_crown", bm)
    o.parent = root
    bm = bmesh.new()
    lathe(bm, [(0.612, base + 0.05), (0.618, base + 0.07), (0.618, base + 0.15), (0.612, base + 0.17)])
    o = to_object("fabric_band", bm, sharp=60.0)
    o.parent = root
    # The bow at the side: two small loops.
    for side in (-1.0, 1.0):
        bm = bmesh.new()
        ret = bmesh.ops.create_uvsphere(bm, u_segments=14, v_segments=8, radius=1.0)
        for v in ret["verts"]:
            v.co = Vector((0.63 + v.co.x * 0.02, side * 0.07 + v.co.y * 0.075, base + 0.11 + v.co.z * 0.045))
        o = to_object("fabric_bow" + ("L" if side < 0 else "R"), bm)
        o.parent = root
    return root


GEAR = {"sailor_cap": sailor_cap, "aviator_cap": aviator_cap, "straw_hat": straw_hat}


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
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=32, v_segments=20, radius=HEAD_R)
    bmesh.ops.translate(bm, vec=HEAD_C, verts=bm.verts)
    head = to_object("head", bm)
    cam_data = bpy.data.cameras.new("cam")
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    target = Vector((0.0, 0.0, 1.85))
    cam.location = target + Vector((1.6, -4.2, 1.2))
    cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)
    bpy.data.objects.remove(head)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
    only = opts["only"].split(",") if "only" in opts else list(GEAR)
    os.makedirs(OUT, exist_ok=True)
    for name in only:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        root = GEAR[name]()
        export(root, os.path.join(OUT, name + ".glb"))
        tris = sum(sum(len(p.vertices) - 2 for p in c.data.polygons) for c in root.children_recursive if c.type == "MESH")
        print("gear %s: %d triangles" % (name, tris))
        if "preview" in opts:
            os.makedirs(opts["preview"], exist_ok=True)
            preview(root, os.path.join(os.path.abspath(opts["preview"]), name + ".png"))


main()
