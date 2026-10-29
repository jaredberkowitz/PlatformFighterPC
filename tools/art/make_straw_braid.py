"""The straw hat's braid (godot/art/cloth/straw_braid.svg, the straw pass, 2026-10-29, the owner's pick of mocks): rows of plaited straw,
each a herringbone of slanted strands with a fine seam between rows, as a straw hat is coiled from one long braid. The rows run along the
tile's width, which the modelled straw hat (art/blender/make_gear.py `straw_hat`) lays round the crown and the brim. #ff00ff is the
straw's colour, swapped in by the game. (A basketweave of square blocks before: close up it read as a checkerboard.)

  python -I tools/art/make_straw_braid.py
"""

import os

SIZE = 64
ROWS = 2            # braided rows per tile
STRANDS = 8         # strands along each row per tile
HIGHLIGHT = 0.2     # the upper strands' light (white over the straw)
SHADE = 0.06        # the lower strands' shade
EDGE = 0.12         # each strand's dark edge
SEAM = 0.16         # the seam between rows
OUT = os.path.join(os.path.dirname(__file__), "..", "..", "godot", "art", "cloth", "straw_braid.svg")


def main():
    h = SIZE / ROWS
    w = SIZE / STRANDS
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 {SIZE} {SIZE}">',
             f'  <rect width="{SIZE}" height="{SIZE}" fill="#ff00ff"/>']
    for r in range(ROWS):
        top, mid, bottom = r * h + 0.8, r * h + h / 2, (r + 1) * h - 0.8
        # (One strand more than fits, so the last runs off the edge and the tile repeats without a seam.)
        for k in range(STRANDS + 1):
            x = k * w
            # The upper half's strands slant one way, the lower half's the other: a plait.
            parts.append(f'  <path d="M{x:.1f},{top:.1f} L{x + w * 0.55:.1f},{top:.1f} L{x + w * 0.95:.1f},{mid:.1f} '
                         f'L{x + w * 0.4:.1f},{mid:.1f} Z" fill="#ffffff" opacity="{HIGHLIGHT}"/>')
            parts.append(f'  <path d="M{x + w * 0.4:.1f},{mid:.1f} L{x + w * 0.95:.1f},{mid:.1f} L{x + w * 0.55:.1f},{bottom:.1f} '
                         f'L{x:.1f},{bottom:.1f} Z" fill="#000000" opacity="{SHADE}"/>')
            parts.append(f'  <path d="M{x + w * 0.55:.1f},{top:.1f} L{x + w * 0.95:.1f},{mid:.1f} L{x + w * 0.55:.1f},{bottom:.1f}" '
                         f'fill="none" stroke="#000000" stroke-opacity="{EDGE}" stroke-width="0.8"/>')
        parts.append(f'  <rect x="0" y="{(r + 1) * h - 0.9:.1f}" width="{SIZE}" height="1.6" fill="#000000" opacity="{SEAM}"/>')
    parts.append('</svg>')
    with open(OUT, "w", newline="\n") as f:
        f.write("\n".join(parts) + "\n")


if __name__ == "__main__":
    main()
