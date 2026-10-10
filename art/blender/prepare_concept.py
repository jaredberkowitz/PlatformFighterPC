"""Cuts the views out of a concept turnaround sheet for image-to-3D (docs/ART_WORKFLOW.md, step 2), using Blender only for its image
reading and writing (no other tools needed):

    blender --background --python art/blender/prepare_concept.py -- --in art/concept/base_body_turnaround.jpg --views art/concept/base_body_views.json

The views file lists each view to cut: {"front": {"box": [x0, y0, x1, y1], "blank_face": {"centre": [x, y], "radius": r}}, ...} in the
sheet's pixels (from its top-left). Each view is cropped, put on a clean white square (the sheet's grey ground, guide lines and labels
turned white: anything nearly colourless), and saved beside the sheet as <sheet>_<view>.png. With `blank_face`, the face inside that
circle is painted over in the skin colour (the colour most common there), for a head that will carry the game's own 2D face drawings
(godot/art/faces); `<view>_blank` is then written as well.
"""

import json
import os
import sys

import bpy
import numpy as np


def args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    out = {}
    for i in range(0, len(argv), 2):
        out[argv[i].lstrip("-")] = argv[i + 1]
    return out


def load(path):
    img = bpy.data.images.load(os.path.abspath(path))
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    # Blender's rows start at the bottom; turn it so row 0 is the top, as the sheet is measured.
    return px.reshape(h, w, 4)[::-1, :, :3].copy()


def save(rgb, path):
    h, w = rgb.shape[0], rgb.shape[1]
    img = bpy.data.images.new(os.path.basename(path), w, h)
    rgba = np.concatenate([rgb[::-1], np.ones((h, w, 1), dtype=np.float32)], axis=2)
    img.pixels.foreach_set(rgba.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()


def saturation(rgb):
    mx = rgb.max(axis=2)
    mn = rgb.min(axis=2)
    return np.where(mx > 1e-4, (mx - mn) / np.maximum(mx, 1e-4), 0.0)


def _grow(region, passable):
    """Floods `region` through `passable` pixels (4-neighbours) until it stops growing."""
    while True:
        bigger = region.copy()
        bigger[1:, :] |= region[:-1, :]
        bigger[:-1, :] |= region[1:, :]
        bigger[:, 1:] |= region[:, :-1]
        bigger[:, :-1] |= region[:, 1:]
        bigger &= passable
        if (bigger == region).all():
            return region
        region = bigger


def clean(view):
    """Only the figure is kept: the coloured or inked pixels joined to the middle of the view are the figure, and everything that can be
    reached from the edge of the view without crossing it (the grey ground, guide lines, labels, bits of the next view) becomes white.
    What the figure encloses (eyes, highlights) is kept as it is."""
    sat = saturation(view)
    value = view.max(axis=2)
    ink = (sat > 0.25) | ((value < 0.4) & (sat > 0.08))
    h, w = ink.shape
    # The figure: the inked region holding the inked pixel nearest the middle.
    ys, xs = np.nonzero(ink)
    k = np.argmin((ys - h / 2) ** 2 + (xs - w / 2) ** 2)
    seed = np.zeros_like(ink)
    seed[ys[k], xs[k]] = True
    figure = _grow(seed, ink)
    # Outside: reached from the border without crossing the figure.
    border = np.zeros_like(ink)
    border[0, :] = border[-1, :] = True
    border[:, 0] = border[:, -1] = True
    outside = _grow(border & ~figure, ~figure)
    out = view.copy()
    out[outside] = 1.0
    return out


def blank_face(view, centre, radius):
    """Paints the face out: inside the circle, every pixel unlike the skin becomes the skin colour."""
    h, w = view.shape[0], view.shape[1]
    yy, xx = np.mgrid[0:h, 0:w]
    inside = (xx - centre[0]) ** 2 + (yy - centre[1]) ** 2 < radius ** 2
    region = view[inside]
    # The skin: the most common colour in the circle (rounded, to group near shades).
    keys = np.round(region * 24).astype(int)
    uniq, counts = np.unique(keys, axis=0, return_counts=True)
    skin_key = uniq[np.argmax(counts)]
    near = np.all(np.abs(keys - skin_key) <= 2, axis=1)
    skin = region[near].mean(axis=0)
    # (Never the white ground round the figure, where the circle runs past the head.)
    ground = view.min(axis=2) > 0.97
    differs = inside & ~ground & (np.linalg.norm(view - skin, axis=2) > 0.06)
    # A little wider than the features, so their soft edges go too.
    for _ in range(3):
        grown = differs.copy()
        grown[1:, :] |= differs[:-1, :]
        grown[:-1, :] |= differs[1:, :]
        grown[:, 1:] |= differs[:, :-1]
        grown[:, :-1] |= differs[:, 1:]
        differs = grown & inside & ~ground
    out = view.copy()
    out[differs] = skin
    return out


def square(view, margin=0.08):
    """The view centred on a white square with a margin round it."""
    h, w = view.shape[0], view.shape[1]
    side = int(max(h, w) * (1.0 + 2.0 * margin))
    out = np.ones((side, side, 3), dtype=np.float32)
    y0 = (side - h) // 2
    x0 = (side - w) // 2
    out[y0:y0 + h, x0:x0 + w] = view
    return out


def main():
    a = args()
    sheet = load(a["in"])
    with open(a["views"]) as f:
        views = json.load(f)
    base = os.path.splitext(os.path.abspath(a["in"]))[0]
    for name, spec in views.items():
        x0, y0, x1, y1 = spec["box"]
        view = clean(sheet[y0:y1, x0:x1])
        save(square(view), "%s_%s.png" % (base, name))
        print("VIEW %s -> %s_%s.png" % (name, base, name))
        if "blank_face" in spec:
            c = spec["blank_face"]["centre"]
            blank = blank_face(view, (c[0] - x0, c[1] - y0), spec["blank_face"]["radius"])
            save(square(blank), "%s_%s_blank.png" % (base, name))
            print("VIEW %s_blank -> %s_%s_blank.png" % (name, base, name))


main()
