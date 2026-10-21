"""Contact shade for a stage's modelled pieces (review 2's idea 3): each piece (the front, the play area, the background ground) was
modelled and its occlusion baked on its own, so nothing shades it where it meets the rest of the stage: the play area's slab over the
front, the trees and props standing on the lawn, the terraces behind the island. This bakes ambient occlusion twice per piece, once with
the whole assembled stage casting it (the pieces themselves and `light_in.glb`, everything else the game builds, written by
`--light-export` on the stage demo) and once with the piece alone, and keeps the ratio: the shade the rest of the stage adds. It is multiplied into the piece's vertex colours (which the game already
multiplies in, `vertex_shade`), and their alpha keeps how much, so running this again divides the old shade out before putting the new
one in instead of stacking them; the piece's own colours and occlusion stay as they were modelled. (Not a second UV set: with any UV
set the exporter stops sharing vertices and the files grew two to three times. Not the alpha alone: the game never sees it.)

Run (Blender 5.2), after the pieces are built and placed, with the game's export beside the bake manifest:
  godot --path godot res://main.tscn -- --demo=stage --stage=<n> --noui --light-export --shots=<tmp>
  blender --background --python art/blender/light_stage.py -- --dir godot/art/baked/<key> --pieces <a.glb>,<b.glb>[,...]
Options: --reach 2.0 (how far shade reaches, units), --samples 96, --curve 1.0 (above 1 deepens it), --floor 0.45 (the darkest it
gets), --out <dir> (write the pieces
there instead of over their files, for comparisons).
"""

import math
import os
import sys

import bpy

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
REACH = float(opts.get("reach", "2.0"))
SAMPLES = int(opts.get("samples", "96"))
FLOOR = float(opts.get("floor", "0.45"))
CURVE = float(opts.get("curve", "1.0"))
PIECES = [p for p in opts["pieces"].split(",") if p]
EPS = 0.03
MARK = 254.0 / 255.0


def setup():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    prefs = bpy.context.preferences.addons["cycles"].preferences
    for kind in ("OPTIX", "CUDA"):
        try:
            prefs.compute_device_type = kind
            prefs.get_devices()
            if any(d.type == kind for d in prefs.devices):
                for d in prefs.devices:
                    d.use = d.type == kind
                sc.cycles.device = "GPU"
                break
        except TypeError:
            continue
    sc.cycles.samples = SAMPLES
    sc.render.bake.target = "VERTEX_COLORS"
    sc.render.bake.margin = 0
    if sc.world is None:
        sc.world = bpy.data.worlds.new("World")
    sc.world.light_settings.distance = REACH


def load(path):
    """Imports a glTF file; returns its mesh objects."""
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    return [o for o in bpy.data.objects if o not in before and o.type == "MESH"]


def bake(objs, name):
    """Bakes ambient occlusion into a new colour attribute `name` on each of `objs` (with whatever renders now casting it)."""
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        me = o.data
        if name in me.color_attributes:
            me.color_attributes.remove(me.color_attributes[name])
        ca = me.color_attributes.new(name, "FLOAT_COLOR", "POINT")
        me.color_attributes.active_color = ca
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.bake(type="AO")


def grey(o, name):
    ca = o.data.color_attributes[name]
    n = len(o.data.vertices)
    buf = [0.0] * (n * 4)
    ca.data.foreach_get("color", buf)
    return [buf[i * 4] for i in range(n)]


def smoothstep(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3.0 - 2.0 * t)


def main():
    setup()
    occ = load(os.path.join(opts["dir"], "light_in.glb"))
    pieces = [(p, load(p)) for p in PIECES]
    everything = occ + [o for _, objs in pieces for o in objs]
    report = []
    for path, objs in pieces:
        own = {}
        # (Each piece's original colour set, kept active for the export.)
        for o in objs:
            own[o.name] = o.data.color_attributes.active_color.name if o.data.color_attributes.active_color else None
        bake(objs, "ao_all")
        for o in everything:
            o.hide_render = o not in objs
        bake(objs, "ao_self")
        for o in everything:
            o.hide_render = False
        shaded = 0
        total = 0
        for o in objs:
            me = o.data
            a = grey(o, "ao_all")
            s = grey(o, "ao_self")
            mw = o.matrix_world
            shade = []
            for i, v in enumerate(me.vertices):
                c = min(1.0, (a[i] + EPS) / (s[i] + EPS)) ** CURVE
                # (Full near the fight, fading back: the distance keeps its clear air, as the grove pilot's bake found.)
                depth = (mw @ v.co).y
                c = 1.0 - (1.0 - c) * (1.0 - 0.7 * smoothstep(15.0, 60.0, depth))
                # (Never quite 1: the alpha also marks the piece as lit, for the test, where nothing shades it.)
                shade.append(min(MARK, max(FLOOR, c)))
                shaded += c < 0.9
            total += len(shade)
            # Into the piece's own colours (made white if it had none), per vertex or per corner as they are stored: the colour
            # without the last run's shade (its alpha), times this one, which the alpha keeps.
            ca = me.color_attributes.get(own[o.name] or "")
            fresh = ca is None
            if fresh:
                ca = me.color_attributes.new("Col", "BYTE_COLOR", "CORNER")
                own[o.name] = ca.name
            if ca.domain == "CORNER":
                idx = [0] * len(me.loops)
                me.loops.foreach_get("vertex_index", idx)
                per = [shade[vi] for vi in idx]
            else:
                per = shade
            buf = [1.0] * (len(per) * 4)
            if not fresh:
                ca.data.foreach_get("color", buf)
            for i, c in enumerate(per):
                old = max(buf[i * 4 + 3], 0.05)
                for k in range(3):
                    buf[i * 4 + k] = min(1.0, buf[i * 4 + k] / old) * c
                buf[i * 4 + 3] = c
            ca.data.foreach_set("color", buf)
            for nm in ("ao_all", "ao_self"):
                me.color_attributes.remove(me.color_attributes[nm])
            if own[o.name]:
                col = me.color_attributes[own[o.name]]
                me.color_attributes.active_color = col
                me.color_attributes.render_color_index = list(me.color_attributes).index(col)
        report.append("%s: %d of %d vertices shaded" % (os.path.basename(path), shaded, total))
        # Write the piece back alone, its materials under their own names: imported beside the rest, a name another file also used
        # came in as `top.001`, and the game paints each material by its name.
        mine = {s.material for o in objs for s in o.material_slots if s.material}
        names = {m: m.name.rsplit(".", 1)[0] if m.name[-4:-3] == "." and m.name[-3:].isdigit() else m.name for m in mine}
        for m in bpy.data.materials:
            if m not in mine and m.name in names.values():
                m.name = "other_" + m.name
        for m, nm in names.items():
            m.name = nm
        bpy.ops.object.select_all(action="DESELECT")
        for o in objs:
            o.select_set(True)
        out = os.path.join(opts["out"], os.path.basename(path)) if "out" in opts else path
        os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
        bpy.ops.export_scene.gltf(filepath=os.path.abspath(out), export_format="GLB", export_yup=True, export_normals=True,
                                  export_materials="EXPORT", export_vertex_color="ACTIVE", use_selection=True)
    for line in report:
        print("light stage: " + line)


main()
