"""Turns a generated or downloaded model into a game asset in the cel style (docs/ART_WORKFLOW.md, step 3).

Run with Blender in the background:

    blender --background --python art/blender/import_generated.py -- --in art/generated/barrel/raw.glb --name barrel --kind prop

What it does, in order:
  1. Imports the model (.glb, .gltf, .fbx or .obj) into an empty scene and joins its meshes into one object.
  2. Turns it (--turn, degrees about the vertical) and scales it to the kind's height (--height overrides), standing on the ground at the
     middle.
  3. Welds split vertices, then reduces it to the kind's triangle budget (--tris overrides) if it is over.
  4. Flattens its colours for cel shading: the model's texture (or materials) is sampled all over, the colours are grouped into a few
     flat ones (--colors; grouped by hue and strength more than lightness, near twins merged), and each flat colour is taken from the
     lighter part of its group, so the light and shadow painted into generated textures go and the game's own lighting does the shading.
     --detail texture (the default when the model has a texture) keeps the texture but repaints every texel in the nearest flat colour,
     so details such as spots, badges or stripes stay crisp, at --texture-size pixels (256); --detail flat (the default otherwise) gives
     each face the nearest flat colour, one plain material per colour (only clean on simple shapes: colours follow the faces).
  5. Smooths the shading of rounded parts while keeping hard edges (by angle), and exports a .glb with plain colour materials to
     godot/models/props/<name>.glb (--out overrides), with a report beside it (<name>.json: triangles, colours, size).

In the game, `Toonify.apply` swaps those plain materials for the cel material of the same colour.
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
import numpy as np

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

# The budgets for each kind of asset (see docs/ART_WORKFLOW.md): height in world units (a fighter is about 2.2 tall), triangles, colours.
KINDS = {
    "accessory": {"height": 0.6, "tris": 1500, "colors": 4},
    "prop": {"height": 1.2, "tris": 3000, "colors": 6},
    "scenery": {"height": 6.0, "tris": 2500, "colors": 6},
    "fighter": {"height": 2.2, "tris": 8000, "colors": 8},
}


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    opts = {"kind": "prop", "turn": 0.0, "detail": "auto", "texture_size": "256"}
    i = 0
    while i < len(argv):
        k = argv[i].lstrip("-").replace("-", "_")
        opts[k] = argv[i + 1]
        i += 2
    if "in" not in opts or "name" not in opts:
        raise SystemExit("usage: ... -- --in <model> --name <name> [--kind accessory|prop|scenery|fighter] [--height H] [--tris N] "
                         "[--colors K] [--detail auto|texture|flat] [--texture-size PX] [--turn DEG] [--out PATH]")
    kind = KINDS[opts["kind"]]
    opts["height"] = float(opts.get("height", kind["height"]))
    opts["tris"] = int(opts.get("tris", kind["tris"]))
    opts["colors"] = int(opts.get("colors", kind["colors"]))
    opts["turn"] = float(opts["turn"])
    opts["texture_size"] = int(opts["texture_size"])
    opts.setdefault("out", os.path.join(ROOT, "godot", "models", "props", opts["name"] + ".glb"))
    return opts


# ---- 1. Import --------------------------------------------------------------------------------------------------------------------------

def import_model(path):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    ext = os.path.splitext(path)[1].lower()
    if ext in (".glb", ".gltf"):
        bpy.ops.import_scene.gltf(filepath=path)
    elif ext == ".fbx":
        bpy.ops.import_scene.fbx(filepath=path)
    elif ext == ".obj":
        bpy.ops.wm.obj_import(filepath=path)
    else:
        raise SystemExit("cannot import %s files" % ext)
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    if not meshes:
        raise SystemExit("no meshes in %s" % path)
    # Free the meshes from any armature or empty they hang under, keeping where they are, then join them.
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
    for o in meshes:
        for m in list(o.modifiers):
            if m.type == "ARMATURE":
                o.modifiers.remove(m)
    if len(meshes) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    for o in list(bpy.context.scene.objects):
        if o != obj:
            bpy.data.objects.remove(o, do_unlink=True)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    return obj


# ---- 2. Size and place ------------------------------------------------------------------------------------------------------------------

def normalize(obj, height, turn):
    if turn:
        obj.rotation_euler = (0.0, 0.0, math.radians(turn))
        bpy.ops.object.transform_apply(rotation=True)
    co = np.empty(len(obj.data.vertices) * 3, dtype=np.float64)
    obj.data.vertices.foreach_get("co", co)
    co = co.reshape(-1, 3)
    lo, hi = co.min(axis=0), co.max(axis=0)
    scale = height / max(hi[2] - lo[2], 1e-6)
    centre = np.array([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2]])
    co = (co - centre) * scale
    obj.data.vertices.foreach_set("co", co.ravel())
    obj.data.update()


# ---- 3. Weld and reduce -----------------------------------------------------------------------------------------------------------------

def triangles(obj):
    return sum(len(p.vertices) - 2 for p in obj.data.polygons)


def weld_and_reduce(obj, height, budget):
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=height * 0.0005)
    bm.to_mesh(obj.data)
    bm.free()
    tris = triangles(obj)
    if tris > budget:
        mod = obj.modifiers.new("budget", "DECIMATE")
        mod.ratio = budget / tris
        mod.use_collapse_triangulate = True
        bpy.ops.object.modifier_apply(modifier=mod.name)


# ---- 4. Flat colours --------------------------------------------------------------------------------------------------------------------

def _srgb_to_linear(c):
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def _linear_to_srgb(c):
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * np.power(np.maximum(c, 0.0), 1.0 / 2.4) - 0.055)


def _base_colour_source(mat):
    """(image, None) when the material's base colour comes from a texture, else (None, the base colour in sRGB)."""
    if mat is None or not mat.use_nodes:
        c = mat.diffuse_color if mat is not None else (0.8, 0.8, 0.8, 1.0)
        return None, np.array(_linear_to_srgb(np.array(c[:3])))
    for node in mat.node_tree.nodes:
        if node.type == "BSDF_PRINCIPLED":
            socket = node.inputs["Base Color"]
            if socket.is_linked:
                src = socket.links[0].from_node
                # Follow through a simple chain (mix, colour ramp...) to the image, if there is one.
                seen = 0
                while src is not None and src.type != "TEX_IMAGE" and seen < 6:
                    nxt = None
                    for inp in src.inputs:
                        if inp.is_linked:
                            nxt = inp.links[0].from_node
                            break
                    src = nxt
                    seen += 1
                if src is not None and src.type == "TEX_IMAGE" and src.image is not None:
                    return src.image, None
            return None, np.array(_linear_to_srgb(np.array(socket.default_value[:3])))
    return None, np.array([0.8, 0.8, 0.8])


def _oklab(srgb):
    """sRGB (0..1) to Oklab, a space where equal steps look equally different."""
    lin = _srgb_to_linear(np.clip(srgb, 0.0, 1.0))
    m1 = np.array([[0.4122214708, 0.5363325363, 0.0514459929], [0.2119034982, 0.6806995451, 0.1073969566],
                   [0.0883024619, 0.2817188376, 0.6299787005]])
    m2 = np.array([[0.2104542553, 0.7936177850, -0.0040720468], [1.9779984951, -2.4285922050, 0.4505937099],
                   [0.0259040371, 0.7827717662, -0.8086757660]])
    return np.cbrt(lin @ m1.T) @ m2.T


# Baked-in light and shade changes mostly a colour's lightness, so the grouping counts lightness for less than hue and strength.
LIGHTNESS_WEIGHT = 0.3
# Groups closer than this (in that space) are one colour.
MERGE_DISTANCE = 0.07


def kmeans(colours, weights, k, iters=20, seed=7):
    """Groups the colours into `k` (k-means++ start, weighted by face area). Returns the group of each colour."""
    rng = random.Random(seed)
    n = len(colours)
    k = max(1, min(k, n))
    centres = [colours[rng.randrange(n)]]
    while len(centres) < k:
        d = np.min([np.sum((colours - c) ** 2, axis=1) for c in centres], axis=0) * weights
        total = d.sum()
        if total <= 0:
            break
        r = rng.random() * total
        centres.append(colours[int(np.searchsorted(np.cumsum(d), r))])
    centres = np.array(centres)
    labels = np.zeros(n, dtype=int)
    for _ in range(iters):
        dist = np.stack([np.sum((colours - c) ** 2, axis=1) for c in centres], axis=1)
        labels = np.argmin(dist, axis=1)
        for j in range(len(centres)):
            m = labels == j
            if m.any():
                centres[j] = np.average(colours[m], axis=0, weights=weights[m] + 1e-9)
    return labels, len(centres)


def _textures(obj):
    """For each material slot: (pixels as an sRGB array h x w x 3, None) when its colour comes from a texture, else (None, its sRGB
    colour)."""
    out = []
    cache = {}
    for slot in obj.material_slots or [None]:
        image, flat = _base_colour_source(slot.material if slot is not None else None)
        if image is None:
            out.append((None, flat))
            continue
        if image.name not in cache:
            w, h = image.size
            arr = np.empty(w * h * 4, dtype=np.float32)
            image.pixels.foreach_get(arr)
            arr = arr.reshape(h, w, 4)[:, :, :3]
            # A texture marked as colour holds sRGB values already; anything else is linear.
            if image.colorspace_settings.name != "sRGB":
                arr = _linear_to_srgb(arr)
            cache[image.name] = arr
        out.append((cache[image.name], None))
    return out


def _sample(arr, u, v):
    h, w = arr.shape[0], arr.shape[1]
    return arr[int((v % 1.0) * (h - 1)), int((u % 1.0) * (w - 1))]


def face_colours(obj, per_face=6, seed=11):
    """Each face's average colour (sRGB, 0..1) and area, and a set of colour samples over the model (spread over each face, weighted by
    its area) for finding the palette: small details such as spots get samples of their own instead of being averaged away."""
    rng = np.random.default_rng(seed)
    me = obj.data
    uv = me.uv_layers.active.data if me.uv_layers.active is not None else None
    sources = _textures(obj)
    colours = np.zeros((len(me.polygons), 3))
    areas = np.zeros(len(me.polygons))
    samples = []
    weights = []
    for i, p in enumerate(me.polygons):
        areas[i] = p.area
        arr, flat = sources[min(p.material_index, len(sources) - 1)]
        if arr is None or uv is None:
            colours[i] = flat
            samples.append(flat)
            weights.append(p.area)
            continue
        loops = list(p.loop_indices)
        corners = np.array([uv[li].uv for li in loops])
        got = [_sample(arr, c[0], c[1]) for c in corners]
        # Random points inside the face (on the fan of triangles from its first corner).
        for _ in range(per_face):
            t = rng.integers(1, len(loops) - 1) if len(loops) > 3 else 1
            a, b = rng.random(), rng.random()
            if a + b > 1.0:
                a, b = 1.0 - a, 1.0 - b
            q = corners[0] + a * (corners[t] - corners[0]) + b * (corners[t + 1] - corners[0])
            got.append(_sample(arr, q[0], q[1]))
        got = np.array(got)
        colours[i] = got.mean(axis=0)
        samples.extend(got)
        weights.extend([p.area / len(got)] * len(got))
    return colours, areas, np.array(samples), np.array(weights), sources, uv


def palette(samples, weights, k):
    """Finds up to `k` flat colours for the samples: groups them by hue and strength more than lightness, merges groups that are nearly
    the same, and takes each group's colour from its lighter part (so baked-in shadow does not darken it). Returns (flat colours in
    sRGB, the group centres in the grouping space)."""
    space = _oklab(samples) * np.array([LIGHTNESS_WEIGHT, 1.0, 1.0])
    labels, k = kmeans(space, weights, k)
    centres = []
    members = []
    for j in range(k):
        m = labels == j
        if m.any():
            centres.append(np.average(space[m], axis=0, weights=weights[m] + 1e-12))
            members.append(m)
    # Merge near twins.
    merged = True
    while merged:
        merged = False
        for a in range(len(centres)):
            for b in range(a):
                if np.linalg.norm(centres[a] - centres[b]) < MERGE_DISTANCE:
                    members[b] = members[b] | members[a]
                    centres[b] = np.average(space[members[b]], axis=0, weights=weights[members[b]] + 1e-12)
                    del centres[a], members[a]
                    merged = True
                    break
            if merged:
                break
    flats = []
    for m in members:
        group = samples[m]
        brightness = group.mean(axis=1)
        keep = brightness >= np.percentile(brightness, 40)
        flats.append(np.average(group[keep], axis=0, weights=weights[m][keep] + 1e-12))
    return np.array(flats), np.array(centres)


def _nearest(colours, centres):
    space = _oklab(colours) * np.array([LIGHTNESS_WEIGHT, 1.0, 1.0])
    return np.argmin(np.stack([np.sum((space - c) ** 2, axis=1) for c in centres], axis=1), axis=1)


def _material(name, srgb, image=None):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    lin = _srgb_to_linear(np.clip(srgb, 0.0, 1.0))
    bsdf.inputs["Base Color"].default_value = (float(lin[0]), float(lin[1]), float(lin[2]), 1.0)
    bsdf.inputs["Roughness"].default_value = 1.0
    if image is not None:
        tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
        tex.image = image
        mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    return mat


def _hex(c):
    return "#%02x%02x%02x" % tuple(int(round(float(x) * 255)) for x in np.clip(c, 0.0, 1.0))


def flatten(obj, k, detail, texture_size):
    """Repaints the model in a few flat colours. `detail` "flat": one plain material per colour, each face the colour nearest its own
    (the cleanest look, for simple shapes). "texture": the texture is kept, every texel repainted in the nearest palette colour, so
    details (spots, badges, stripes) stay but the baked shading goes; it is resized to `texture_size`."""
    colours, areas, samples, weights, sources, uv = face_colours(obj)
    flats, centres = palette(samples, weights, k)
    textured = [s for s in sources if s[0] is not None]
    if detail in ("texture", "auto") and textured and uv is not None:
        arr = textured[0][0]
        h, w = arr.shape[0], arr.shape[1]
        nearest = _nearest(arr.reshape(-1, 3), centres).reshape(h, w)
        painted = flats[nearest]
        img = bpy.data.images.new("palette", w, h)
        rgba = np.concatenate([painted, np.ones((h, w, 1))], axis=2).astype(np.float32)
        img.pixels.foreach_set(rgba.ravel())
        if max(w, h) > texture_size:
            # (Nearest-colour picture, so it is scaled without blending colours together.)
            img.scale(texture_size, texture_size)
        img.pack()
        obj.data.materials.clear()
        obj.data.materials.append(_material("palette", np.ones(3), img))
        for p in obj.data.polygons:
            p.material_index = 0
        return [_hex(c) for c in flats]
    labels = _nearest(colours, centres)
    obj.data.materials.clear()
    for j, c in enumerate(flats):
        obj.data.materials.append(_material("flat_%d" % j, c))
    for i, p in enumerate(obj.data.polygons):
        p.material_index = int(labels[i])
    return [_hex(c) for c in flats]


# ---- 5. Shade and export ----------------------------------------------------------------------------------------------------------------

def shade(obj):
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    try:
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(40.0))
    except (AttributeError, RuntimeError):
        bpy.ops.object.shade_smooth()


def export(obj, path, keep_uv):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if not keep_uv:
        for uv in list(obj.data.uv_layers):
            obj.data.uv_layers.remove(uv)
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True, export_apply=True,
                              export_materials="EXPORT", export_yup=True)


def main():
    opts = parse_args()
    obj = import_model(os.path.abspath(opts["in"]))
    obj.name = opts["name"]
    before = triangles(obj)
    normalize(obj, opts["height"], opts["turn"])
    weld_and_reduce(obj, opts["height"], opts["tris"])
    hexes = flatten(obj, opts["colors"], opts["detail"], opts["texture_size"])
    shade(obj)
    export(obj, opts["out"], len(obj.data.materials) == 1 and obj.data.materials[0].name.startswith("palette"))
    report = {"name": opts["name"], "kind": opts["kind"], "source": os.path.relpath(os.path.abspath(opts["in"]), ROOT).replace("\\", "/"),
              "triangles_in": before, "triangles": triangles(obj), "height": opts["height"], "colours": hexes}
    with open(os.path.splitext(opts["out"])[0] + ".json", "w") as f:
        json.dump(report, f, indent=2)
    print("IMPORTED %s: %d -> %d triangles, %d colours %s -> %s" % (opts["name"], before, report["triangles"], len(hexes),
                                                                    " ".join(hexes), opts["out"]))


main()
