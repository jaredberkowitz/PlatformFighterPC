"""Concept paint-over of a stage (review 2's idea 6, docs/art/stages/PAINTOVERS.md): the match camera's frame repainted toward the paper
target, so the target is a picture in our own style that the next passes can be measured against, not other games' looks.

It works from two captures of the same view (the stage demo with `--depth`: the frame and its distance from the camera) and paints the
paper language over them, each move a separate layer that can be judged on its own:
  layers   each paper layer casts a soft shadow onto the layer behind it, away from the light (where a nearer surface stands between this
           pixel and the light, a step along the screen), so the scene reads as cut sheets standing apart;
  edges    a pale die-cut edge along each near layer's outline, against what is behind it;
  crevice  shade where a surface sits deeper than its surroundings (the gaps between stones, under the lip of the top);
  light    the painted key: a warm wash from the light's side, a cool one away from it, and a soft lift round the fight;
  air      aerial perspective by depth: the distance toward the stage's air colour and lower in contrast;
  grain    a faint paper fibre over everything.

Run with Blender's Python (it carries numpy):
  blender --background --python tools/art/paintover.py -- <frame.png> <depth.png> <out.png> <stage> [--only layers,edges,...]
The fighters (the frame against `<frame>_stage.png`, the same frame without them) are left as they were: the target is the stage's.
"""

import sys

import bpy
import numpy as np

args = sys.argv[sys.argv.index("--") + 1:]
FRAME, DEPTH, OUT, STAGE = args[:4]
ONLY = None
if "--only" in args:
    ONLY = set(args[args.index("--only") + 1].split(","))

# Per stage: the light's direction on screen (towards it), its warm and the shade's cool colour, the air colour and how far it starts,
# and each move's strength.
LOOKS = {
    "meadow": dict(light=(-0.8, -0.6), warm=(1.0, 0.93, 0.8), cool=(0.78, 0.86, 1.0), air=(0.82, 0.9, 0.95), air_from=0.25,
                   layers=0.35, edges=0.5, crevice=0.45, key=0.18, lift=0.1, grain=0.03),
    "grove": dict(light=(0.7, -0.7), warm=(1.0, 0.9, 0.78), cool=(0.8, 0.8, 1.0), air=(0.86, 0.82, 0.94), air_from=0.25,
                  layers=0.4, edges=0.5, crevice=0.5, key=0.2, lift=0.12, grain=0.03),
    "beach": dict(light=(0.6, -0.8), warm=(1.0, 0.95, 0.84), cool=(0.8, 0.9, 1.0), air=(0.84, 0.92, 0.98), air_from=0.3,
                  layers=0.3, edges=0.45, crevice=0.4, key=0.15, lift=0.1, grain=0.03),
    "canyon": dict(light=(-0.9, -0.4), warm=(1.0, 0.86, 0.7), cool=(0.84, 0.78, 0.98), air=(0.9, 0.8, 0.9), air_from=0.2,
                   layers=0.4, edges=0.5, crevice=0.45, key=0.22, lift=0.1, grain=0.03),
    "city": dict(light=(0.0, -1.0), warm=(1.0, 0.82, 0.6), cool=(0.7, 0.68, 1.0), air=(0.5, 0.36, 0.62), air_from=0.3,
                 layers=0.45, edges=0.55, crevice=0.5, key=0.22, lift=0.16, grain=0.03),
}
L = LOOKS[STAGE]


def load(path):
    img = bpy.data.images.load(path)
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    # (Blender's pixels are bottom row first and linear for a colour image: flip to top first, back to display values.)
    a = px.reshape(h, w, 4)[::-1, :, :3].copy()
    if img.colorspace_settings.name != "Non-Color":
        a = np.where(a <= 0.0031308, a * 12.92, 1.055 * np.power(np.clip(a, 0, None), 1 / 2.4) - 0.055)
    return a


def save(a, path):
    h, w, _ = a.shape
    img = bpy.data.images.new("out", w, h, alpha=False)
    lin = np.where(a <= 0.04045, a / 12.92, np.power((np.clip(a, 0, 1) + 0.055) / 1.055, 2.4))
    rgba = np.concatenate([lin, np.ones((h, w, 1), np.float32)], axis=2)[::-1].astype(np.float32)
    img.pixels.foreach_set(rgba.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()


def blur(a, r):
    """A soft blur: three passes of a box `r` pixels each way (close to a Gaussian), edges held."""
    out = a.astype(np.float32)
    for _ in range(3):
        for axis in (0, 1):
            pad = [(0, 0)] * out.ndim
            pad[axis] = (r + 1, r)
            c = np.cumsum(np.pad(out, pad, mode="edge"), axis=axis)
            n = out.shape[axis]
            hi = np.take(c, np.arange(2 * r + 1, 2 * r + 1 + n), axis=axis)
            lo = np.take(c, np.arange(0, n), axis=axis)
            out = (hi - lo) / (2 * r + 1)
    return out


def shift(a, dx, dy):
    """`a` moved by (dx, dy) pixels, edges held."""
    h, w = a.shape[:2]
    ys = np.clip(np.arange(h) - dy, 0, h - 1)
    xs = np.clip(np.arange(w) - dx, 0, w - 1)
    return a[ys][:, xs]


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def on(name):
    return ONLY is None or name in ONLY


import json
import os

col = load(FRAME)
near = load(DEPTH)[:, :, 0]             # 1 at the camera, 0 at 160 units
h, w = near.shape
dist = 1.0 - near                       # 0 near, 1 far
lum = col @ np.array([0.299, 0.587, 0.114], np.float32)
out = col.copy()
ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
u = xs / w - 0.5
v = ys / h - 0.5

if on("air"):
    # The distance toward the air colour, its contrast lowered round its own blurred value.
    k = smoothstep(L["air_from"], 0.9, dist)[..., None]
    soft = blur(out, 6)
    out = out + (soft - out) * 0.4 * k
    out = out + (np.array(L["air"], np.float32) - out) * 0.35 * k

if on("light"):
    lx, ly = L["light"]
    s = np.clip(0.5 + (u * lx + v * ly) * 1.1, 0.0, 1.0)[..., None]   # 1 toward the light
    warm = np.array(L["warm"], np.float32)
    cool = np.array(L["cool"], np.float32)
    tint = cool + (warm - cool) * s
    out = out * (1.0 + (tint - 1.0) * L["key"] * 1.2)
    # A soft lift round the fight (the frame's middle, a little below centre) and a gentle fall-off to the corners.
    r = np.sqrt((u / 0.62) ** 2 + ((v - 0.08) / 0.5) ** 2)
    out = out * (1.0 + L["lift"] * (1.0 - smoothstep(0.0, 1.0, r)) - 0.08 * smoothstep(0.8, 1.4, r))[..., None]

if on("crevice"):
    # Deeper than the surroundings at a few scales: crevices and contact.
    occl = np.zeros_like(near)
    for r_, wgt in ((3, 0.5), (8, 0.35), (18, 0.15)):
        occl += wgt * np.clip((blur(near, r_) - near) * 160.0 / (r_ * 0.6), 0.0, 1.0)
    occl *= 1.0 - smoothstep(0.15, 0.3, dist)
    out = out * (1.0 - L["crevice"] * occl)[..., None]

if on("layers"):
    # Each nearer sheet's shadow on the one behind it, cast away from the light by a few pixels and softened.
    lx, ly = L["light"]
    shade = np.zeros_like(near)
    for step, wgt in ((5, 0.45), (11, 0.35), (20, 0.2)):
        toward = shift(near, int(round(lx * step)), int(round(ly * step)))
        gap = (toward - near) * 160.0          # units the surface toward the light stands in front of this one
        shade += wgt * smoothstep(0.4, 3.0, gap)
    shade = blur(shade, 3) * (1.0 - smoothstep(0.2, 0.45, dist))
    out = out * (1.0 - L["layers"] * shade)[..., None]

if on("edges"):
    # A pale cut edge on the near side of each outline where what is behind is a step further back.
    back = np.maximum.reduce([near - shift(near, dx, dy) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))])
    edge = smoothstep(0.004, 0.03, back) * (1.0 - smoothstep(0.25, 0.5, dist))
    edge = np.maximum(edge, 0.5 * blur(edge, 1))
    out = out + (np.array([1.0, 0.97, 0.9], np.float32) - out) * (L["edges"] * edge)[..., None]

if on("grain"):
    rng = np.random.default_rng(7)
    fibre = blur(rng.standard_normal((h, w)).astype(np.float32), 1)
    streak = blur(rng.standard_normal((h, w)).astype(np.float32), 4)
    g = fibre + streak * 0.25
    g = g / (np.abs(g).max() + 1e-6)
    out = out * (1.0 + L["grain"] * g * (1.0 - smoothstep(0.85, 0.98, dist)))[..., None]

# The fighters as they were: where the frame differs from the same frame without them (`<frame>_stage.png` beside it; their bodies,
# shadows and markers), with a soft edge.
bare = FRAME.replace(".png", "_stage.png")
if os.path.exists(bare):
    keep = smoothstep(0.02, 0.08, np.abs(col - load(bare)).max(axis=2))
    keep = np.clip(blur(np.maximum(keep, shift(keep, 1, 0)), 1) * 1.6, 0.0, 1.0)
    out = out + (col - out) * keep[..., None]

save(np.clip(out, 0.0, 1.0), OUT)
print("paintover: %s -> %s" % (STAGE, OUT))
