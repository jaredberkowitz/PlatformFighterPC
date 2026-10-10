"""Measures a body model and places the game's skeleton in it (docs/ART_WORKFLOW.md, Characters).

    blender --background --python art/blender/fit_body.py -- --in art/generated/base_body/base_body.glb --out art/generated/base_body/joints.json

The body must stand upright on the ground, facing -Y (Blender's front), arms hanging at its sides, as `import_generated.py --kind fighter`
leaves it. It is cut into thin horizontal slices; in each slice the points are grouped by gaps across X (two arms and the trunk, or two
legs). From those:
  - the neck is where the trunk is narrowest between the shoulders and the top of the head; the head's middle and radius come from the
    part above it;
  - the armpits are the highest slice where the arms stand apart from the trunk; each arm's line runs down through its slices to the
    fingertips; the shoulder sits a little above the armpit on that line, the elbow halfway to the wrist, the wrist at 78% of the arm;
  - the crotch is the highest slice where the legs stand apart; hips just above it, knees halfway to the ankles, ankles a little above
    the ground, each leg on its own line.
The result is the same joints the game's skeleton has (make_rigged_blob.py JOINTS), plus the head's middle and radius and the arm and leg
lines, written as JSON for make_rigged_blob.py --body.
"""

import json
import os
import sys

import bpy
import numpy as np


def args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    return {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv), 2)}


def load_points(path, count=200000, seed=5):
    """Points spread evenly over the body's surface (a low-poly model has too few corners for thin slices to see its shape)."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    tris = []
    for o in bpy.context.scene.objects:
        if o.type != "MESH":
            continue
        me = o.data
        me.calc_loop_triangles()
        co = np.array([(o.matrix_world @ v.co)[:] for v in me.vertices])
        idx = np.array([t.vertices[:] for t in me.loop_triangles])
        tris.append(co[idx])
    tris = np.concatenate(tris)
    a, b, c = tris[:, 0], tris[:, 1], tris[:, 2]
    area = np.linalg.norm(np.cross(b - a, c - a), axis=1) / 2.0
    rng = np.random.default_rng(seed)
    pick = rng.choice(len(tris), size=count, p=area / area.sum())
    u = rng.random(count)
    v = rng.random(count)
    flip = u + v > 1.0
    u[flip], v[flip] = 1.0 - u[flip], 1.0 - v[flip]
    return a[pick] + (b[pick] - a[pick]) * u[:, None] + (c[pick] - a[pick]) * v[:, None]


def clusters(xs, gap):
    """Groups sorted X values where they are more than `gap` apart: [(min, max, mean), ...] left to right."""
    xs = np.sort(xs)
    if len(xs) == 0:
        return []
    cuts = np.flatnonzero(np.diff(xs) > gap)
    out = []
    start = 0
    for c in list(cuts) + [len(xs) - 1]:
        part = xs[start:c + 1]
        out.append((part.min(), part.max(), part.mean()))
        start = c + 1
    return out


def fit(pts):
    height = pts[:, 2].max()
    step = height / 120.0
    gap = height * 0.02
    slices = []
    z = step / 2
    while z < height:
        band = pts[np.abs(pts[:, 2] - z) < step / 2]
        slices.append((z, band, clusters(band[:, 0], gap)))
        z += step

    def central(cl):
        # The group holding the middle (x = 0), or the nearest one.
        return min(cl, key=lambda c: 0.0 if c[0] <= 0.0 <= c[1] else min(abs(c[0]), abs(c[1])))

    # The neck: the narrowest trunk between 55% and 85% of the height.
    upper = [(z, central(cl)) for z, band, cl in slices if 0.55 * height < z < 0.85 * height and cl]
    neck_z, neck_c = min(upper, key=lambda t: t[1][1] - t[1][0])
    head_band = pts[pts[:, 2] > neck_z]
    top = head_band[:, 2].max()
    head_half_width = (head_band[:, 0].max() - head_band[:, 0].min()) / 2.0
    head_radius = max((top - neck_z) / 2.0, head_half_width)
    head_centre = np.array([0.0, float(np.median(head_band[:, 1])), top - head_radius])

    def overlaps(c, prev):
        return c[0] <= prev[1] + gap and c[1] >= prev[0] - gap

    def follow(order, start_cl, stop):
        """Follows one group through the slices in `order`, slice to slice by overlap, until it is gone or `stop(cluster, cl)`."""
        line = []
        prev = start_cl
        for z, band, cl in order:
            match = [c for c in cl if overlaps(c, prev)]
            if not match:
                break
            c = max(match, key=lambda c: c[1] - c[0]) if len(match) > 1 else match[0]
            if stop(c, cl):
                return line, z
            sel = band[(band[:, 0] >= c[0]) & (band[:, 0] <= c[1])]
            line.append((z, c[2], float(sel[:, 1].mean()), c[0], c[1]))
            prev = c
        return line, None

    # The legs: the two groups at the ground, followed up until they join (the crotch).
    bottom = next(i for i, (z, band, cl) in enumerate(slices) if len(cl) >= 2)
    up = slices[bottom:]
    feet = up[0][2]
    legs = {}
    crotch_z = None
    for side, start_cl in (("L", feet[0]), ("R", feet[-1])):
        other = feet[-1] if side == "L" else feet[0]

        def joined(c, cl, other=other, side=side):
            # The leg's group now spans the middle: both legs are in it.
            return c[0] < -gap * 0.5 and c[1] > gap * 0.5
        line, z_join = follow(up, start_cl, joined)
        legs[side] = sorted([(t[0], t[1], t[2]) for t in line], key=lambda t: -t[0])
        crotch_z = z_join if crotch_z is None else min(crotch_z, z_join if z_join is not None else crotch_z)
    # The armpits: going down from the neck, the first slice where the trunk has an arm clear of it on either side.
    down = [t for t in reversed(slices) if t[0] < neck_z]
    armpit_z = None
    for z, band, cl in down:
        if len(cl) >= 3 and cl[0][2] < -gap and cl[-1][2] > gap:
            armpit_z = z
            first = cl
            break
    arms = {}
    below = [t for t in down if t[0] <= armpit_z]
    for side, start_cl in (("L", first[0]), ("R", first[-1])):

        def merged(c, cl, side=side):
            # The arm has reached the trunk or a leg (its group now holds the middle, or it is the innermost group).
            return c[0] < 0.0 < c[1] or len(cl) < 2
        line, _ = follow(below, start_cl, merged)
        arms[side] = [(t[0], t[1], t[2]) for t in line]

    def at(line, frac):
        """The point `frac` of the way down a line of slices (0 its top, 1 its bottom)."""
        top_z, bottom_z = line[0][0], line[-1][0]
        zz = top_z + (bottom_z - top_z) * frac
        best = min(line, key=lambda t: abs(t[0] - zz))
        return [best[1], best[2], zz]

    joints = {"root": [0.0, 0.0, 0.0]}
    hips_z = crotch_z + height * 0.04
    joints["hips"] = [0.0, 0.0, hips_z]
    joints["spine"] = [0.0, 0.0, hips_z + (neck_z - hips_z) * 0.3]
    joints["head"] = [0.0, float(head_centre[1]) * 0.5, neck_z]
    for side in ("L", "R"):
        arm = arms[side]
        # The arm's top end is at the armpit; the shoulder (the ball the arm turns on) sits above it, a little in toward the trunk. The
        # wrist is a hand's length above the fingertips.
        sx, sy, sz = arm[0][1], arm[0][2], arm[0][0]
        shoulder = [sx * 0.92, sy, sz + height * 0.07]
        tip = arm[-1]
        wrist_z = tip[0] + height * 0.075
        near = min(arm, key=lambda t: abs(t[0] - wrist_z))
        wrist = [near[1], near[2], wrist_z]
        elbow = [(shoulder[0] + wrist[0]) / 2.0, (shoulder[1] + wrist[1]) / 2.0, (shoulder[2] + wrist[2]) / 2.0]
        joints["armU." + side] = shoulder
        joints["armL." + side] = elbow
        joints["hand." + side] = wrist
        joints["tip." + side] = [tip[1], tip[2], tip[0]]
        leg = legs[side]
        thigh = [leg[0][1], 0.0, hips_z]
        ankle_z = height * 0.075
        ankle = at(leg, 1.0)
        ankle[2] = ankle_z
        knee = [(thigh[0] + ankle[0]) / 2.0, 0.0, (thigh[2] + ankle_z) / 2.0]
        joints["thigh." + side] = thigh
        joints["shin." + side] = knee
        joints["foot." + side] = [ankle[0], ankle[1], ankle_z]
    return {
        "height": float(height),
        "neck": float(neck_z), "armpit": float(armpit_z), "crotch": float(crotch_z),
        "head_centre": head_centre.tolist(), "head_radius": float(head_radius),
        "joints": {k: [float(x) for x in v] for k, v in joints.items()},
        "arm_lines": {k: [[float(x) for x in p] for p in v] for k, v in arms.items()},
        "leg_lines": {k: [[float(x) for x in p] for p in v] for k, v in legs.items()},
    }


def main():
    a = args()
    result = fit(load_points(os.path.abspath(a["in"])))
    with open(os.path.abspath(a["out"]), "w") as f:
        json.dump(result, f, indent=1)
    print("FIT neck %.3f armpit %.3f crotch %.3f head centre %s radius %.3f" % (result["neck"], result["armpit"], result["crotch"],
          [round(x, 3) for x in result["head_centre"]], result["head_radius"]))
    for k, v in result["joints"].items():
        print("JOINT %-8s %s" % (k, [round(x, 3) for x in v]))


main()
