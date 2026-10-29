"""The cloth's fleck print (godot/art/cloth/fleck.svg, the fleck pass, 2026-10-28, the owner's pick of mocks): short grey dashes and dots
scattered on white, as the reference cast's shirts and shorts carry a soft printed speckle. The toon shader multiplies the cloth's
colour by it, so it darkens each colour a little in flecks. 256 units square and seamless (marks near an edge are drawn again across it).

  python -I tools/art/make_fleck.py
"""

import math
import os
import random

SIZE = 256
MARKS = 150
DOTS = 0.4                  # share of the marks that are dots
DASH = (6.8, 15.3)          # a dash's length
DASH_WIDTH = 3.7
DOT_RADIUS = 2.7
DEPTH = (0.12, 0.2)         # how dark a mark is (its opacity on white)
OUT = os.path.join(os.path.dirname(__file__), "..", "..", "godot", "art", "cloth", "fleck.svg")


def main():
    rng = random.Random(7)
    marks = []
    for _ in range(MARKS):
        x, y = rng.uniform(0, SIZE), rng.uniform(0, SIZE)
        a = rng.uniform(0, math.tau)
        dot = rng.random() < DOTS
        depth = rng.uniform(*DEPTH)
        reach = DOT_RADIUS if dot else DASH[1] / 2 + DASH_WIDTH
        length = 0.0 if dot else rng.uniform(*DASH)
        # (Drawn again one tile over wherever it crosses an edge, so the tile repeats without a seam.)
        for ox in (-SIZE, 0, SIZE):
            for oy in (-SIZE, 0, SIZE):
                cx, cy = x + ox, y + oy
                if cx + reach < 0 or cx - reach > SIZE or cy + reach < 0 or cy - reach > SIZE:
                    continue
                if dot:
                    marks.append(f'  <circle cx="{cx:.1f}" cy="{cy:.1f}" r="{DOT_RADIUS}" stroke="none" opacity="{depth:.3f}"/>')
                else:
                    dx, dy = math.cos(a) * length / 2, math.sin(a) * length / 2
                    marks.append(f'  <line x1="{cx - dx:.1f}" y1="{cy - dy:.1f}" x2="{cx + dx:.1f}" y2="{cy + dy:.1f}" '
                                 f'opacity="{depth:.3f}"/>')
    svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 {SIZE} {SIZE}">',
           f'  <rect width="{SIZE}" height="{SIZE}" fill="#ffffff"/>',
           f'  <g fill="#000000" stroke="#000000" stroke-width="{DASH_WIDTH}" stroke-linecap="round">',
           *marks, '  </g>', '</svg>', '']
    with open(OUT, "w", newline="\n") as f:
        f.write("\n".join(svg))


if __name__ == "__main__":
    main()
