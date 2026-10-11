"""Generates the bark, foliage and stone tiles (godot/art/stage/*.svg): 256x256, tileable, on the placeholder colour #ff00ff (which the
game swaps for the surface's own colour), with only translucent black and white over it, so each stays low-contrast and takes any hue."""
import math
import random

OUT = 'C:/Users/jared/OneDrive/Desktop/Platform Fighter PC/godot/art/stage/'
HEAD = '<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">\n  <rect width="256" height="256" fill="#ff00ff"/>\n'


def wrap(draw, x, y, r):
    """Draws a shape at (x, y) and again across each edge it reaches, so the tile repeats seamlessly."""
    out = []
    for ox in (-256, 0, 256):
        for oy in (-256, 0, 256):
            if -r - 4 <= x + ox <= 256 + r + 4 and -r - 4 <= y + oy <= 256 + r + 4:
                out.append(draw(x + ox, y + oy))
    return ''.join(out)


def bark():
    rng = random.Random(4)
    s = HEAD
    # Broad vertical bands of light and shade (the ridges and furrows of the bark).
    for k in range(8):
        x = k * 32 + rng.uniform(-4, 4)
        s += f'  <rect x="{x:.1f}" y="0" width="{rng.uniform(10, 18):.1f}" height="256" fill="#ffffff" opacity="0.07"/>\n'
    # Grooves: long wavering vertical lines, a few deeper ones, each one wrapping top to bottom.
    for k in range(11):
        x0 = k * 256 / 11 + rng.uniform(-6, 6)
        amp = rng.uniform(2.5, 6.0)
        ph = rng.uniform(0, math.tau)
        pts = []
        for i in range(17):
            y = i * 16
            pts.append(f'{x0 + math.sin(y / 256 * math.tau * 2 + ph) * amp:.1f},{y}')
        deep = k % 4 == 0
        w = 5.5 if deep else 3.0
        op = 0.34 if deep else 0.2
        d = 'M' + ' L'.join(pts)
        for ox in (-256, 0, 256):
            s += f'  <path d="{d}" transform="translate({ox},0)" fill="none" stroke="#000000" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round" opacity="{op}"/>\n'
    # A few darker recesses (knots and splits) and lighter flecks.
    for _ in range(4):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        rx, ry = rng.uniform(5, 9), rng.uniform(10, 18)
        s += wrap(lambda a, b: f'  <ellipse cx="{a:.1f}" cy="{b:.1f}" rx="{rx:.1f}" ry="{ry:.1f}" fill="#000000" opacity="0.18"/>\n'
                  f'  <ellipse cx="{a:.1f}" cy="{b:.1f}" rx="{rx * 0.45:.1f}" ry="{ry * 0.5:.1f}" fill="#000000" opacity="0.16"/>\n', x, y, ry)
    for _ in range(14):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        s += wrap(lambda a, b: f'  <rect x="{a:.1f}" y="{b:.1f}" width="2.2" height="{rng.uniform(6, 14):.1f}" rx="1.1" fill="#ffffff" opacity="0.12"/>\n', x, y, 14)
    return s + '</svg>\n'


def foliage():
    rng = random.Random(7)
    s = HEAD
    # Soft clumps of leaves: rounded scalloped blobs, the lower edge of each in shade, a light catch on the upper edge.
    for layer, (count, rmin, rmax) in enumerate([(14, 22, 34), (18, 14, 22)]):
        for _ in range(count):
            x, y = rng.uniform(0, 256), rng.uniform(0, 256)
            r = rng.uniform(rmin, rmax)

            def clump(a, b, r=r):
                out = f'  <circle cx="{a:.1f}" cy="{b + r * 0.18:.1f}" r="{r:.1f}" fill="#000000" opacity="0.07"/>\n'
                out += f'  <circle cx="{a:.1f}" cy="{b:.1f}" r="{r * 0.92:.1f}" fill="#ff00ff"/>\n'
                out += f'  <path d="M{a - r * 0.6:.1f},{b - r * 0.35:.1f} Q{a:.1f},{b - r * 0.95:.1f} {a + r * 0.6:.1f},{b - r * 0.35:.1f}" fill="none" stroke="#ffffff" stroke-width="{r * 0.12:.1f}" stroke-linecap="round" opacity="0.07"/>\n'
                return out
            s += wrap(clump, x, y, r)
    # Tiny leaf flecks.
    for _ in range(40):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        s += wrap(lambda a, b: f'  <ellipse cx="{a:.1f}" cy="{b:.1f}" rx="3.2" ry="1.8" transform="rotate({rng.uniform(0, 180):.0f} {a:.1f} {b:.1f})" fill="#ffffff" opacity="0.1"/>\n', x, y, 4)
    return s + '</svg>\n'


def stone():
    rng = random.Random(11)
    s = HEAD
    # Broad, faint facets of light and shade.
    for _ in range(9):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        r = rng.uniform(28, 52)
        col, op = ('#ffffff', 0.06) if rng.random() < 0.5 else ('#000000', 0.06)
        pts = []
        n = rng.randint(5, 7)
        for i in range(n):
            a = i / n * math.tau + rng.uniform(-0.3, 0.3)
            rr = r * rng.uniform(0.7, 1.0)
            pts.append((math.cos(a) * rr, math.sin(a) * rr))
        s += wrap(lambda a, b: '  <polygon points="' + ' '.join(f'{a + px:.1f},{b + py:.1f}' for px, py in pts) + f'" fill="{col}" opacity="{op}"/>\n', x, y, r)
    # A couple of hairline cracks.
    for _ in range(3):
        x, y = rng.uniform(20, 236), rng.uniform(20, 236)
        d = f'M{x:.1f},{y:.1f}'
        for _ in range(4):
            x += rng.uniform(-14, 14)
            y += rng.uniform(6, 16)
            d += f' L{x:.1f},{y:.1f}'
        s += f'  <path d="{d}" fill="none" stroke="#000000" stroke-width="1.6" stroke-linecap="round" opacity="0.18"/>\n'
    # Speckles.
    for _ in range(70):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        col = '#000000' if rng.random() < 0.6 else '#ffffff'
        s += wrap(lambda a, b: f'  <circle cx="{a:.1f}" cy="{b:.1f}" r="{rng.uniform(0.8, 2.0):.1f}" fill="{col}" opacity="0.14"/>\n', x, y, 2)
    return s + '</svg>\n'


for name, make in [('bark', bark), ('foliage', foliage), ('stone', stone)]:
    open(OUT + name + '.svg', 'w', newline='\n').write(make())
    open(OUT + name + '.svg.import', 'w', newline='\n').write('[remap]\n\nimporter="keep"\n')
    print(name, 'ok')
