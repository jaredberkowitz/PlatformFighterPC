"""Printed paper surfaces (fidelity review 3's idea 2, the meadow pilot): one tiling texture per material family, grey around the middle
(the toon shader's `print_tex` darkens and lightens the surface's own colour by it), in three looks for the owner to choose from:
  A  fibre and mottle: soft watercolour mottling over long paper fibres, the family telling only in the grain's direction and scale;
  B  hatched print: each family drawn in a printed illustration's marks: grass in short upward strokes, wood in long wavering grain,
     stone in patches of crossed hatching, soil and path in stipple, straw in fine parallel strokes;
  C  halftone: a screen of dots at 45 degrees, their size following each family's own pattern (as B, softened), as a printed card.
Families: grass, soil, path, wood, stone, straw. Each 512 pixels square and seamless.
`sandstone` (fidelity review 9: the canyon's long straight beds took `strata`, its thin lines running the bed's length, and read as wood
grain): sand speckle, soft mottled patches and short sets of cross-bedding arcs, no long lines; `--sandstone B` adds faint weathering
streaks down the face and a few pits. Appended last, so the other families' random numbers, and so their prints, stay as they were.

Run with Blender's Python (it carries numpy):
  blender --background --python tools/art/make_prints.py -- --style A --out <folder>
"""

import math
import os
import sys

import bpy
import numpy as np

argv = sys.argv[sys.argv.index("--") + 1:]
opts = {argv[i].lstrip("-"): argv[i + 1] for i in range(0, len(argv) - 1, 2)}
STYLE = opts.get("style", "A").upper()
SANDSTONE = opts.get("sandstone", "A").upper()
N = 512
rng = np.random.default_rng(23)


def smooth_noise(scale, aniso=(1.0, 1.0)):
    """Seamless noise: white noise filtered in frequency (so it wraps), features about `scale` pixels, stretched by `aniso` (x, y)."""
    f = np.fft.fft2(rng.standard_normal((N, N)))
    fy = np.fft.fftfreq(N)[:, None] * aniso[1]
    fx = np.fft.fftfreq(N)[None, :] * aniso[0]
    k = np.sqrt(fx * fx + fy * fy) * scale
    out = np.real(np.fft.ifft2(f * np.exp(-(k ** 2) * 4.0)))
    return (out - out.mean()) / (out.std() + 1e-9)


def strokes(count, length, width, angle, jitter, curve=0.0):
    """Short marks drawn on a wrapping canvas: `count` strokes `length` pixels long at `angle` degrees (+- `jitter`)."""
    img = np.zeros((N, N), np.float32)
    for _ in range(count):
        x, y = rng.uniform(0, N, 2)
        a = math.radians(angle + rng.uniform(-jitter, jitter))
        ln = length * rng.uniform(0.6, 1.2)
        steps = int(ln * 1.5) + 2
        bend = rng.uniform(-curve, curve)
        for s in range(steps):
            t = s / (steps - 1)
            aa = a + bend * (t - 0.5)
            px = x + math.cos(aa) * ln * t
            py = y + math.sin(aa) * ln * t
            w = width * (1.0 - abs(t - 0.5) * 1.2)
            r = max(1, int(math.ceil(w)))
            for dy in range(-r, r + 1):
                for dx in range(-r, r + 1):
                    if dx * dx + dy * dy <= w * w:
                        img[int(py + dy) % N, int(px + dx) % N] = 1.0
    return img


def blur(a, r):
    out = a
    for _ in range(2):
        for axis in (0, 1):
            acc = np.zeros_like(out)
            for d in range(-r, r + 1):
                acc += np.roll(out, d, axis=axis)
            out = acc / (2 * r + 1)
    return out


def norm(a, lo=0.5, amp=0.5):
    """To grey around `lo`, the darkest marks `amp` below it."""
    a = a - a.mean()
    return np.clip(lo + a / (np.abs(a).max() + 1e-9) * amp, 0.0, 1.0)


def fibre(aniso):
    return smooth_noise(2.0, aniso) * 0.5 + smooth_noise(6.0, aniso) * 0.5


def motif(fam):
    """The family's printed marks (1 where ink is), for B and C."""
    if fam == "grass":
        return strokes(900, 16, 1.2, -90, 22, 0.4)
    if fam == "wood":
        img = np.zeros((N, N), np.float32)
        yy = np.arange(N)[:, None]
        xx = np.arange(N)[None, :]
        wave = np.sin(xx * math.tau / N * 2 + 0.7) * 6 + smooth_noise(40, (1.0, 0.2))[0:1, :] * 0
        for k in range(22):
            y0 = k * N / 22 + rng.uniform(-4, 4)
            d = np.abs(((yy - y0 - wave - np.sin(xx * math.tau / N * 3 + k) * 3) + N / 2) % N - N / 2)
            img = np.maximum(img, (d < rng.uniform(0.8, 1.6)).astype(np.float32) * (np.sin(xx * math.tau / N * rng.integers(1, 4) + k) > -0.6))
        return img
    if fam == "stone":
        # A light hatch everywhere, crossed in patches (shade on the stone's own facets).
        patch = smooth_noise(30) > 0.6
        yy, xx = np.mgrid[0:N, 0:N]
        h1 = ((xx + yy) % 12) < 1.1
        h2 = ((xx - yy) % 12) < 1.1
        return np.maximum(h1, h2 & patch).astype(np.float32)
    if fam in ("soil", "path"):
        img = (rng.random((N, N)) < (0.05 if fam == "soil" else 0.03)).astype(np.float32)
        img = np.maximum(img, np.roll(img, 1, 0))
        return img * (0.5 + 0.5 * (smooth_noise(24) > -0.3))
    if fam == "straw":
        return strokes(700, 30, 0.9, 0, 6, 0.2)
    if fam == "sand":
        # Fine stipple with a few long, gentle ripple marks.
        img = (rng.random((N, N)) < 0.035).astype(np.float32)
        return np.maximum(img, strokes(60, 70, 0.8, 0, 8, 0.6) * 0.8)
    if fam == "strata":
        # Layers: broken horizontal lines at uneven spacing, as a rock's bedding.
        img = np.zeros((N, N), np.float32)
        yy, xx = np.mgrid[0:N, 0:N]
        y = 0.0
        while y < N:
            y += rng.uniform(10, 26)
            wav = np.sin(xx * math.tau / N * rng.integers(1, 3) + rng.uniform(0, 6)) * 3
            line = np.abs(((yy - y - wav) + N / 2) % N - N / 2) < 0.9
            gaps = np.sin(xx * math.tau / N * rng.integers(3, 7) + rng.uniform(0, 6)) > -0.5
            img = np.maximum(img, (line & gaps).astype(np.float32))
        return img
    if fam == "sandstone":
        # Sand grains, broad soft patches a little deeper, and cross-bedding: short low arcs, a few to a set, no line longer than a
        # stone's height.
        img = (rng.random((N, N)) < 0.03).astype(np.float32) * 0.8
        img = np.maximum(img, np.clip((smooth_noise(40) - 0.3) * 0.12, 0.0, 0.14))
        img = np.maximum(img, strokes(70, 46, 0.9, 12, 12, 0.6) * 0.9)
        if SANDSTONE == "B":
            # Weathering: faint streaks down the face, and pits.
            img = np.maximum(img, blur(strokes(14, 150, 1.3, 90, 4, 0.1), 2) * 0.5)
            img = np.maximum(img, np.clip(blur((rng.random((N, N)) < 0.004).astype(np.float32), 1) * 4.0, 0.0, 0.9))
        return img
    if fam == "roof":
        # Tar paper: a seam every course, overlapping, and a sparse grit.
        yy, xx = np.mgrid[0:N, 0:N]
        seams = ((yy % 64) < 1.4) | (((yy % 64) < 4) & ((xx % 128) < 1.4))
        grit = rng.random((N, N)) < 0.02
        return np.maximum(seams, grit).astype(np.float32)
    if fam == "steel":
        # Fine, even diagonal hatching, as engraved metal.
        yy, xx = np.mgrid[0:N, 0:N]
        return (((xx - yy * 0.5) % 8) < 0.9).astype(np.float32) * 0.8
    return np.zeros((N, N), np.float32)


FAMS = {"grass": (1.0, 3.0), "soil": (1.0, 1.0), "path": (1.0, 1.0), "wood": (4.0, 1.0), "stone": (1.0, 1.0), "straw": (4.0, 1.0),
        "sand": (2.0, 1.0), "strata": (4.0, 1.0), "roof": (3.0, 1.0), "steel": (1.0, 1.0), "sandstone": (2.0, 1.0)}


def make(fam):
    aniso = FAMS[fam]
    if STYLE == "A":
        mottle = smooth_noise(60) * 0.6 + smooth_noise(18) * 0.4
        return norm(mottle * 0.7 + fibre(aniso) * 0.5, 0.5, 0.5)
    m = motif(fam)
    if STYLE == "B":
        ink = blur(m, 1)
        return norm(-ink * 1.0 + fibre(aniso) * 0.08, 0.55, 0.55)
    # C: halftone dots, their size from the softened motif and a broad mottle.
    tone = blur(m, 6)
    tone = (tone - tone.min()) / (tone.max() - tone.min() + 1e-9)
    tone = np.clip(tone * 0.8 + (smooth_noise(50) * 0.15 + 0.15), 0.0, 1.0)
    yy, xx = np.mgrid[0:N, 0:N].astype(np.float32)
    cell = 8.0
    u = (xx + yy) / math.sqrt(2.0)
    v = (xx - yy) / math.sqrt(2.0)
    du = (u % cell) - cell / 2
    dv = (v % cell) - cell / 2
    dist = np.sqrt(du * du + dv * dv)
    radius = tone * cell * 0.62
    dots = np.clip(radius - dist + 0.5, 0.0, 1.0)
    return norm(-dots, 0.55, 0.5)


def save(a, path):
    img = bpy.data.images.new("print", N, N, alpha=False, float_buffer=False)
    img.colorspace_settings.name = "Non-Color"
    g = a.astype(np.float32)[::-1]
    rgba = np.stack([g, g, g, np.ones_like(g)], axis=2)
    img.pixels.foreach_set(rgba.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()


out = os.path.abspath(opts["out"])
os.makedirs(out, exist_ok=True)
for fam in FAMS:
    save(make(fam), os.path.join(out, fam + ".png"))
print("prints %s -> %s" % (STYLE, out))
