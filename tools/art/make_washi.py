"""Washi paper (the style pass's step 6, the grove's craft; docs/art/stages/STYLE_PASS.md): one seamless 512-pixel texture for paper
held to the light, in three channels:
  R  long fibres: thin strands wandering across the sheet, as the kozo fibres that show in washi against a lamp;
  G  formation: the sheet's thickness in soft clouds, thicker and thinner patches as in a handmade sheet (grey around the middle);
  B  fine fibres: shorter, finer strands between the long ones.
Read by shaders/washi.gdshader (the grove's lanterns) and toon_common's `washi_paper` and tissue (the grove's mist).

Run with Blender's Python (it carries numpy):
  blender --background --python tools/art/make_washi.py -- --out godot/art/stage/png/washi.png
"""

import math
import os
import sys

import bpy
import numpy as np

argv = sys.argv[sys.argv.index("--") + 1:]
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
N = 512
rng = np.random.default_rng(1971)


def smooth_noise(scale):
    """Seamless noise: white noise filtered in frequency (so it wraps), features about `scale` pixels."""
    f = np.fft.fft2(rng.standard_normal((N, N)))
    fy = np.fft.fftfreq(N)[:, None]
    fx = np.fft.fftfreq(N)[None, :]
    k = np.sqrt(fx * fx + fy * fy) * scale
    out = np.real(np.fft.ifft2(f * np.exp(-(k ** 2) * 4.0)))
    return (out - out.mean()) / (out.std() + 1e-9)


def fibres(count, length, width, bend, jitter):
    """`count` strands about `length` pixels long and `width` across, each bending gently as it goes (up to `bend` radians over its
    length, with `jitter` radians of wobble a step), tapered at both ends, each at its own strength, drawn soft-edged on a wrapping
    canvas. (Long and mostly straight: turning freely they curled into rings.)"""
    img = np.zeros((N, N), np.float32)
    for _ in range(count):
        x, y = rng.uniform(0, N, 2)
        a = rng.uniform(0.0, math.tau)
        ln = length * rng.uniform(0.5, 1.4)
        turn = rng.uniform(-bend, bend) / (ln * 2)
        k = rng.uniform(0.45, 1.0)
        w = width * rng.uniform(0.7, 1.3)
        steps = int(ln * 2)
        r = int(math.ceil(w * 0.5 + 1.0))
        for s in range(steps):
            t = s / steps
            a += turn + rng.normal(0.0, jitter)
            x += math.cos(a) * 0.5
            y += math.sin(a) * 0.5
            taper = min(1.0, 5.0 * t, 5.0 * (1.0 - t))
            ix, iy = int(math.floor(x)), int(math.floor(y))
            for dy in range(-r, r + 1):
                for dx in range(-r, r + 1):
                    d = math.hypot(ix + dx + 0.5 - x, iy + dy + 0.5 - y)
                    v = (1.0 - min(1.0, max(0.0, d - w * 0.5))) * k * taper
                    if v > 0.0:
                        py, px = (iy + dy) % N, (ix + dx) % N
                        if v > img[py, px]:
                            img[py, px] = v
    return img


def blur(a, r):
    out = a
    for axis in (0, 1):
        acc = np.zeros_like(out)
        for d in range(-r, r + 1):
            acc += np.roll(out, d, axis=axis)
        out = acc / (2 * r + 1)
    return out


long_f = blur(fibres(380, 170, 1.3, 1.1, 0.012), 1)
long_f = np.clip(long_f / (long_f.max() + 1e-9) * 1.25, 0.0, 1.0)
form = smooth_noise(22.0) * 0.6 + smooth_noise(60.0) * 0.4
form = np.clip(0.5 + form / (np.abs(form).max() + 1e-9) * 0.5, 0.0, 1.0)
fine = blur(fibres(1300, 50, 0.9, 0.8, 0.02), 1)
fine = np.clip(fine / (fine.max() + 1e-9) * 1.3, 0.0, 1.0)

img = bpy.data.images.new("washi", N, N, alpha=False, float_buffer=False)
img.colorspace_settings.name = "Non-Color"
rgba = np.stack([long_f[::-1], form[::-1], fine[::-1], np.ones((N, N), np.float32)], axis=2)
img.pixels.foreach_set(rgba.astype(np.float32).ravel())
out = os.path.abspath(opts.get("out", "washi.png"))
os.makedirs(os.path.dirname(out), exist_ok=True)
img.filepath_raw = out
img.file_format = "PNG"
img.save()
print("washi -> %s" % out)
