"""Generates the bark, foliage, stone and grass tiles (godot/art/stage/*.svg): 256x256, tileable, on the placeholder colour #ff00ff (which the
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
    """Painted bark (art review 6 round 27: evenly spaced copies of one wavy line read as a Flash-game texture): broad plates of
    different widths, each a tone of its own with a lit left edge, divided by furrows that wander and now and then split, a few short
    cross-cracks. Every curve repeats with the tile (whole periods in y), so it tiles both ways."""
    rng = random.Random(11)
    s = HEAD
    n = 7
    widths = [rng.uniform(0.6, 1.5) for _ in range(n)]
    total = sum(widths)
    bases = []
    x = rng.uniform(0, 20)
    for w in widths:
        bases.append(x)
        x += w * 256 / total
    waves = [(rng.uniform(3, 7), rng.uniform(0, math.tau), rng.uniform(1.5, 3.5), rng.uniform(0, math.tau)) for _ in range(n)]

    def edge(k, y):
        a1, p1, a2, p2 = waves[k % n]
        return bases[k % n] + (256 if k >= n else 0) + a1 * math.sin(y / 256 * math.tau + p1) + a2 * math.sin(y / 256 * math.tau * 3 + p2)

    ys = [i * 8 for i in range(33)]
    for k in range(n):
        left = [(edge(k, y), y) for y in ys]
        right = [(edge(k + 1, y), y) for y in reversed(ys)]
        tone = rng.choice([('#ffffff', rng.uniform(0.06, 0.14)), ("#000000", rng.uniform(0.05, 0.12))])
        pts = ' '.join(f'{px:.1f},{py:.1f}' for px, py in left + right)
        lit = 'M' + ' L'.join(f'{px + 3.5:.1f},{py:.1f}' for px, py in left)
        for ox in (-256, 0, 256):
            s += f'  <polygon points="{pts}" transform="translate({ox},0)" fill="{tone[0]}" opacity="{tone[1]:.2f}"/>\n'
            s += f'  <path d="{lit}" transform="translate({ox},0)" fill="none" stroke="#ffffff" stroke-width="3" opacity="0.1"/>\n'
    # The furrows: a soft wide shade and a thin dark core along each plate boundary.
    for k in range(n):
        d = 'M' + ' L'.join(f'{edge(k, y):.1f},{y}' for y in ys)
        for ox in (-256, 0, 256):
            s += f'  <path d="{d}" transform="translate({ox},0)" fill="none" stroke="#000000" stroke-width="8" stroke-linecap="round" opacity="0.2"/>\n'
            s += f'  <path d="{d}" transform="translate({ox},0)" fill="none" stroke="#000000" stroke-width="2.8" stroke-linecap="round" opacity="0.45"/>\n'
    # A few furrows split: a short branch leaving one boundary and fading into the plate.
    for _ in range(4):
        k = rng.randrange(n)
        y0 = rng.uniform(0, 256)
        x0 = edge(k, y0)
        dx = rng.uniform(8, 18)
        dy = rng.uniform(30, 60)
        s += wrap(lambda a, b, dx=dx, dy=dy: f'  <path d="M{a:.1f},{b:.1f} q{dx * 0.3:.1f},{dy * 0.5:.1f} {dx:.1f},{dy:.1f}" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" opacity="0.24"/>\n', x0, y0, 70)
    # Short cross-cracks on the plates.
    for _ in range(9):
        k = rng.randrange(n)
        y0 = rng.uniform(0, 256)
        a, b = edge(k, y0), edge(k + 1, y0)
        x0 = a + (b - a) * rng.uniform(0.25, 0.6)
        ln = (b - a) * rng.uniform(0.2, 0.4)
        s += wrap(lambda p, q, ln=ln: f'  <path d="M{p:.1f},{q:.1f} l{ln:.1f},{rng.uniform(-3, 3):.1f}" fill="none" stroke="#000000" stroke-width="1.8" stroke-linecap="round" opacity="0.2"/>\n', x0, y0, 40)
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


def grass():
    """A lawn as the reference paints it: one clean, vivid sheet of green, its variation in long soft sweeps (as wind or a mower lays
    grass), a few broad cooler hollows and warmer rises, no blades or speckle. Low contrast and large in scale, so at play distance it is
    a calm, saturated surface the fighters stand out on."""
    rng = random.Random(11)
    s = HEAD
    # Broad cool hollows and warm rises.
    for _ in range(9):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        rx, ry = rng.uniform(50, 90), rng.uniform(26, 44)
        col, op = ('#fff8d0', 0.07) if rng.random() < 0.5 else ('#002a30', 0.07)
        s += wrap(lambda a, b: f'  <ellipse cx="{a:.1f}" cy="{b:.1f}" rx="{rx:.1f}" ry="{ry:.1f}" fill="{col}" opacity="{op}"/>\n', x, y, rx)
    # Long soft sweeps across the lawn, a little curved, lighter and darker in turn.
    for k in range(26):
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        length = rng.uniform(70, 150)
        bend = rng.uniform(-10, 10)
        width = rng.uniform(5, 12)
        col, op = ('#ffffff', 0.06) if k % 2 == 0 else ('#00262a', 0.06)

        def sweep(a, b, length=length, bend=bend, width=width, col=col, op=op):
            return (f'  <path d="M{a - length / 2:.1f},{b:.1f} q{length / 2:.1f},{bend:.1f} {length:.1f},0" fill="none" stroke="{col}" '
                    f'stroke-width="{width:.1f}" stroke-linecap="round" opacity="{op}"/>\n')
        s += wrap(sweep, x, y, length / 2 + width)
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


for name, make in [('bark', bark), ('foliage', foliage), ('stone', stone), ('grass', grass)]:
    open(OUT + name + '.svg', 'w', newline='\n').write(make())
    open(OUT + name + '.svg.import', 'w', newline='\n').write('[remap]\n\nimporter="keep"\n')
    print(name, 'ok')
