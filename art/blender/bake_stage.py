"""Bakes ambient occlusion for a code-built stage's scenery (the stage refinement's grove pilot, docs/art/stages/BAKE_PILOT.md).

Input: the glTF that `StageBake.export_scene` (godot/scripts/stage_bake.gd) wrote with `--bake-export`: meshes to bake named `b0007`,
occluders `o0003`. Every `b` mesh larger than `--min_area` gets a second UV set (`LM`, smart-projected, one object at a time) and its
own occlusion texture `b0007.png`, sized by its surface area (meshes wholly behind `--far` are skipped); everything in the scene casts occlusion on it. The `b` meshes are written
back to `scene_baked.glb` with that UV set, which the game swaps in (`StageBake.apply`).

Run (Blender 5.2):
  blender --background --python art/blender/bake_stage.py -- --dir godot/art/baked/grove_14d06e65 [--px 10] [--max 1024] [--samples 256]
"""

import math
import os
import sys

import bpy
import numpy as np

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
DIR = os.path.abspath(opts["dir"])
PX = float(opts.get("px", 10.0))          # texels per world unit
MAX = int(opts.get("max", 1024))
MIN = int(opts.get("min", 16))
MIN_AREA = float(opts.get("min_area", 0.4))
SAMPLES = int(opts.get("samples", 256))
DIST = float(opts.get("dist", 3.0))       # how far occlusion reaches, world units
BLUR = int(opts.get("blur", 1))           # passes of a soft blur over each texture (bake noise)
FAR = float(opts.get("far", -40.0))       # meshes wholly behind this depth (Godot z) are not baked: too large for useful texels


def use_gpu():
    try:
        prefs = bpy.context.preferences.addons["cycles"].preferences
    except KeyError:
        return "CPU"
    for kind in ("OPTIX", "CUDA", "HIP", "ONEAPI", "METAL"):
        try:
            prefs.compute_device_type = kind
        except TypeError:
            continue
        prefs.get_devices()
        devs = [d for d in prefs.devices if d.type == kind]
        if devs:
            for d in prefs.devices:
                d.use = d.type == kind
            bpy.context.scene.cycles.device = "GPU"
            return kind
    return "CPU"


def world_area(ob):
    m = ob.matrix_world.to_3x3()
    k = abs(m.determinant()) ** (2.0 / 3.0)
    return sum(p.area for p in ob.data.polygons) * k


def blur(img):
    if BLUR <= 0:
        return
    w, h = img.size
    px = np.array(img.pixels[:], dtype=np.float32).reshape(h, w, 4)
    filled = px[:, :, 3] > 0.0
    for _ in range(BLUR):
        acc = np.zeros_like(px[:, :, 0])
        cnt = np.zeros_like(acc)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                acc += np.roll(np.roll(px[:, :, 0] * filled, dy, 0), dx, 1)
                cnt += np.roll(np.roll(filled.astype(np.float32), dy, 0), dx, 1)
        v = np.where(cnt > 0, acc / np.maximum(cnt, 1), px[:, :, 0])
        px[:, :, 0] = np.where(filled, v, px[:, :, 0])
    px[:, :, 1] = px[:, :, 0]
    px[:, :, 2] = px[:, :, 0]
    img.pixels[:] = px.ravel()


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=os.path.join(DIR, "scene_in.glb"), merge_vertices=True, import_merge_material_slots=False,
                              import_shading="NORMALS")
    scene = bpy.context.scene
    try:
        scene.render.engine = "CYCLES"
    except TypeError as e:
        print("bake: no Cycles", e)
        return
    device = use_gpu()
    scene.cycles.samples = SAMPLES
    if scene.world is None:
        scene.world = bpy.data.worlds.new("Bake")
    scene.world.light_settings.distance = DIST
    bpy.ops.object.select_all(action="DESELECT")
    bpy.ops.object.make_single_user(type="ALL", object=True, obdata=True)

    import time
    t0 = time.time()
    bake, skipped = [], 0
    names = sorted(o.name for o in scene.objects if o.type == "MESH" and o.name.startswith("b"))
    for idx, name in enumerate(names):
        ob = scene.objects[name]
        area = world_area(ob)
        # (Blender is z-up: Godot's depth z is Blender's -y.)
        corners = [ob.matrix_world @ v.co for v in ob.data.vertices]
        if area < MIN_AREA or not corners or max(-c.y for c in corners) < FAR:
            skipped += 1
            continue
        if idx % 50 == 0:
            print("bake: unwrapping %d of %d (%.0f s)" % (idx, len(names), time.time() - t0), flush=True)
        me = ob.data
        if len(me.uv_layers) == 0:
            me.uv_layers.new(name="UVMap")
        lm = me.uv_layers.new(name="LM")
        me.uv_layers.active = lm
        bpy.context.view_layer.objects.active = ob
        ob.select_set(True)
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.uv.smart_project(angle_limit=math.radians(60.0), island_margin=0.03, area_weight=0.0, scale_to_bounds=False)
        bpy.ops.object.mode_set(mode="OBJECT")
        ob.select_set(False)
        size = 2 ** math.ceil(math.log2(max(MIN, min(MAX, math.sqrt(area) * PX * 1.4))))
        img = bpy.data.images.new(ob.name, size, size, alpha=True)
        img.generated_color = (0.0, 0.0, 0.0, 0.0)
        for slot in ob.material_slots:
            mat = slot.material
            if mat is None:
                continue
            mat.use_nodes = True
            nodes = mat.node_tree.nodes
            node = nodes.new("ShaderNodeTexImage")
            node.image = img
            for n in nodes:
                n.select = False
            node.select = True
            nodes.active = node
        bake.append((ob, img))
    texels = sum(i.size[0] * i.size[1] for _, i in bake)
    print("bake: %d meshes to bake (%d too small or far), %.1f M texels, on %s (%.0f s)" % (len(bake), skipped, texels / 1e6, device,
          time.time() - t0), flush=True)

    for ob, _ in bake:
        ob.select_set(True)
    bpy.context.view_layer.objects.active = bake[0][0]
    bpy.ops.object.bake(type="AO", margin=6, use_clear=False, target="IMAGE_TEXTURES")
    print("bake: baked (%.0f s)" % (time.time() - t0), flush=True)
    for ob, img in bake:
        blur(img)
        img.filepath_raw = os.path.join(DIR, ob.name + ".png")
        img.file_format = "PNG"
        img.save()

    baked_names = {ob.name for ob, _ in bake}
    for ob in list(scene.objects):
        if ob.name not in baked_names:
            bpy.data.objects.remove(ob, do_unlink=True)
    bpy.ops.export_scene.gltf(filepath=os.path.join(DIR, "scene_baked.glb"), export_format="GLB", export_yup=True,
                              export_texcoords=True, export_normals=True, export_materials="EXPORT", export_vertex_color="ACTIVE")
    print("bake: wrote %d textures and scene_baked.glb to %s" % (len(bake), DIR))


main()
