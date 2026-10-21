"""Draws the parts the fighters' faces are put together from (godot/art/face_parts/), in the concept art's style: big round eyes, round
pink cheeks, small simple mouths, and (new) brows and noses to choose from. The game places, sizes, turns and colours each part on the
head (shaders/face_common.gdshaderinc, scripts/face_look.gd), so a player can build a face the way they like, and every expression is
the player's own face with its lids, brows and mouth changed.

Each part is a 256x256 drawing centred in its square:

  eyes/<style>_fill.svg   the white of the eye (a white mask: the game colours it cream and draws the pupil and the lids inside it)
  eyes/<style>_line.svg   its ink outline and lashes (a white mask, drawn in ink)
  eyes/closed_<name>.svg  shut and squeezed eyes for expressions (ink masks): blink, happy, effort, hurt, dazed (a spiral)
  brows/<style>.svg       a brow (a mask in the brow colour)
  noses/<style>.svg       a nose (a mask, drawn a shade of ink)
  cheeks/<style>.svg      a cheek (a mask in the cheek colour)
  mouths/<style>.svg      a mouth, in colour (resting mouths and the expressions' mouths)

Eyes, brows and cheeks are drawn for the part on the right of the face (screen right when the fighter faces the camera); the game
mirrors them for the left. Inner ends (toward the nose) are on the left of each drawing.

Run: python tools/art/make_face_parts.py
"""
import math
import os

ROOT = __file__.replace('\\', '/').rsplit('/tools/', 1)[0] + '/godot/art/face_parts/'
# (A near-black since the face finish pass, 2026-10-20, as the face shader's ink: the dark plum read soft beside the references' black.)
INK = '#0b0b38'   # (a deep navy since the reference pass, 2026-10-20, as the salute reference's lines)
MOUTH_IN = '#3a1830'
TONGUE = '#ef7f8e'
TEETH = '#f1ecd6'   # (ivory, as the eyes' whites)
MASK = '#ffffff'
LINE = 11          # the eye outline's width: a fine line, as the concept's
MOUTH_LINE = 18   # (21 until the face pass, 2026-10-19: the reference's mouths are bold strokes; thin, they vanished in a match; 30 until the
                  # reference pass, 2026-10-20: the salute's line is about a twentieth of its mouth's width)


def svg(body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">\n{body}</svg>\n'


def stroke(d, w=LINE, colour=MASK):
    return f'  <path d="{d}" fill="none" stroke="{colour}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>\n'


def fill(d, colour=MASK):
    return f'  <path d="{d}" fill="{colour}"/>\n'


def ellipse_d(cx, cy, rx, ry, rot=0.0):
    """An ellipse as a path (so it can be both filled and stroked the same way), turned `rot` degrees (clockwise on screen)."""
    pts = []
    for i in range(64):
        a = math.tau * i / 64
        x, y = rx * math.cos(a), ry * math.sin(a)
        c, s = math.cos(math.radians(rot)), math.sin(math.radians(rot))
        pts.append(f'{cx + x * c - y * s:.1f},{cy + x * s + y * c:.1f}')
    return 'M' + ' L'.join(pts) + ' Z'


# ---- Eyes --------------------------------------------------------------------------------------------------------------------------
# The concept's eye is a round one about 92 across the 256 square (radius), so the pupils (drawn by the game, 0.26 of the square) fill
# most of it. Others vary the shape; "dot" is small enough that the pupil fills it (a beady eye).
EYES = {
    'round': ellipse_d(128, 128, 92, 92),
    'tall': ellipse_d(128, 128, 76, 100),
    'wide': ellipse_d(128, 128, 104, 72),
    'almond': 'M24,132 Q128,6 232,124 Q128,246 24,132 Z',
    'droopy': ellipse_d(128, 128, 100, 80, 16),
    'cat': ellipse_d(128, 128, 102, 74, -18),
    'dot': ellipse_d(128, 128, 50, 54),
    'lash': ellipse_d(128, 128, 90, 90),
}
# Lashes on the outer top of the "lash" eye.
LASHES = ''.join(stroke(f'M{128 + 90 * math.cos(a):.1f},{128 + 90 * math.sin(a):.1f} L{128 + 124 * math.cos(a - 0.12):.1f},'
                        f'{128 + 124 * math.sin(a - 0.12):.1f}', 13)
                 for a in (math.radians(-80), math.radians(-52), math.radians(-24)))

CLOSED = {
    # A soft downward arc (asleep, blinking), an upward one (a happy squint).
    'blink': stroke('M56,120 Q128,184 200,120', 17),
    'happy': stroke('M58,150 Q128,70 198,150', 17),
    # Squeezed shut: chevrons pointing in toward the nose (gritted effort, a wince when hurt).
    'effort': stroke('M196,90 L80,128 L196,166', 17),
    'hurt': stroke('M206,86 L90,128 L206,170', 18),
}


def spiral():
    pts = []
    for i in range(70):
        a = i / 70 * math.tau * 2.6
        r = 8 + i * 1.2
        pts.append(f'{128 + math.cos(a) * r:.1f},{128 + math.sin(a) * r:.1f}')
    return stroke('M' + ' L'.join(pts), 13)


CLOSED['dazed'] = spiral()

# ---- Brows ---------------------------------------------------------------------------------------------------------------------------
BROWS = {
    # (The arch, bar and slant soft rounded strokes since the reference pass, 2026-10-20: cornered blocks read sharp, the owner.)
    'arch': stroke('M48,150 Q124,84 206,126', 26),
    'thin': stroke('M44,146 Q124,92 212,134', 18),
    'bar': stroke('M50,134 Q128,100 206,124', 34),
    'slant': stroke('M48,154 Q130,112 208,104', 32),
    'round': fill(ellipse_d(128, 126, 44, 28)),
    'bushy': fill('M30,150 Q44,108 70,116 Q80,88 110,98 Q128,74 152,92 Q176,78 192,100 Q220,96 226,126 Q200,118 176,126 '
                  'Q150,112 128,128 Q104,116 84,134 Q60,128 30,150 Z'),
}

# ---- Noses -----------------------------------------------------------------------------------------------------------------------------
NOSES = {
    'dot': fill(ellipse_d(128, 132, 22, 16)),
    'curve': stroke('M136,88 Q104,132 128,156 Q140,164 152,154', 14),
    'button': stroke(ellipse_d(128, 128, 34, 28), 12),
    'hook': stroke('M120,80 L104,150 Q120,162 148,152', 14),
    'nostrils': fill(ellipse_d(106, 132, 11, 9)) + fill(ellipse_d(150, 132, 11, 9)),
    'wide': stroke('M88,140 Q104,118 128,138 Q152,118 168,140', 13),
}

# ---- Cheeks ----------------------------------------------------------------------------------------------------------------------------
CHEEKS = {
    'round': fill(ellipse_d(128, 128, 90, 56)),
    'oval': fill(ellipse_d(128, 128, 104, 40)),
    'freckles': ''.join(fill(ellipse_d(x, y, 13, 13)) for x, y in ((96, 112), (140, 104), (118, 146), (164, 140))),
    'lines': ''.join(stroke(f'M{x},{150} L{x + 26},{100}', 14) for x in (70, 112, 154)),
    'swirl': stroke('M128,128 m-6,0 a6,6 0 1,1 12,0 a18,18 0 1,1 -30,6 a32,32 0 1,1 54,-18 a46,46 0 1,1 -78,34', 12),
    'heart': fill('M128,176 C64,132 72,84 104,84 Q122,84 128,104 Q134,84 152,84 C184,84 192,132 128,176 Z'),
}

# ---- Mouths ---------------------------------------------------------------------------------------------------------------------------


# (Redrawn since the reference pass, 2026-10-20, after the salute reference: a thinner navy line, small rounded shapes, ivory teeth in one
# band with plain gaps; the owner: ours read sharp, striking and sometimes scary, theirs soft and cute. The corners of the grins rounded.)


def open_mouth(path, tongue=True, tongue_y=160):
    out = f'  <clipPath id="m"><path d="{path}"/></clipPath>\n' + fill(path, MOUTH_IN)
    if tongue:
        out += f'  <ellipse cx="128" cy="{tongue_y}" rx="52" ry="26" fill="{TONGUE}" clip-path="url(#m)"/>\n'
    return out + f'  <path d="{path}" fill="none" stroke="{INK}" stroke-width="13" stroke-linejoin="round"/>\n'


def teeth(path, gaps):
    out = f'  <clipPath id="t"><path d="{path}"/></clipPath>\n' + fill(path, TEETH)
    for x in gaps:
        out += stroke(f'M{x},40 L{x},220', 9, INK).replace('/>', ' clip-path="url(#t)"/>')
    return out + f'  <path d="{path}" fill="none" stroke="{INK}" stroke-width="13" stroke-linejoin="round"/>\n'


MOUTHS = {
    # Resting mouths a player can choose (the concept's small smile first).
    'smile': stroke('M44,112 Q128,170 212,112', MOUTH_LINE, INK),
    'grin': open_mouth('M58,106 Q128,124 198,106 Q192,176 128,180 Q64,176 58,106 Z'),
    'flat': stroke('M88,130 L168,130', MOUTH_LINE, INK),
    'oh': open_mouth(ellipse_d(128, 130, 28, 32), tongue=False),
    'cat': stroke('M60,118 Q94,158 128,124 Q162,158 196,118', MOUTH_LINE, INK),
    'smirk': stroke('M76,138 Q140,154 194,108', MOUTH_LINE, INK),
    # The salute's grin: its top following the smile, tall teeth, three gaps, rounded at the corners.
    'toothy': teeth('M58,104 C90,120 166,120 198,104 C214,96 224,110 216,128 C202,166 168,184 128,184 C88,184 54,166 40,128 '
                    'C32,110 42,96 58,104 Z', (84, 128, 172)),
    'frown': stroke('M74,150 Q128,110 182,150', MOUTH_LINE, INK),
    'blep': f'  <path d="M136,126 L170,126 L170,146 Q153,168 136,146 Z" fill="{TONGUE}" stroke="{INK}" stroke-width="10" '
            f'stroke-linejoin="round"/>\n' + stroke('M70,126 L186,126', MOUTH_LINE, INK),
    # The expressions' mouths (the same for everyone).
    # (A rounded open D for the yell, a clenched grin for the strain, a small open frown for the ouch, a small o for the gasp: a dark
    # trapezoid, a box of teeth in a grid, a lipstick bean and a big dark oval before.)
    'x_attack': open_mouth('M62,100 Q128,90 194,100 Q188,170 128,176 Q68,170 62,100 Z', tongue_y=166),
    'x_effort': teeth('M54,112 Q128,98 202,112 Q198,158 128,164 Q58,158 54,112 Z', (100, 128, 156)),
    'x_hurt': open_mouth('M74,156 Q78,112 128,110 Q178,112 182,156 Q128,146 74,156 Z', tongue=False),
    'x_shock': open_mouth(ellipse_d(128, 132, 26, 32), tongue=False),
    'x_happy': open_mouth('M50,100 Q128,112 206,100 Q198,176 128,182 Q58,176 50,100 Z', tongue_y=166),
    'x_focus': stroke('M92,126 L164,126', MOUTH_LINE, INK),
    'x_dazed': stroke('M62,128 Q95,100 128,128 Q161,156 194,128', MOUTH_LINE, INK),
}

def write(folder, name, body):
    os.makedirs(ROOT + folder, exist_ok=True)
    path = f'{ROOT}{folder}/{name}.svg'
    open(path, 'w', newline='\n').write(svg(body))
    # Kept as files (not imported as textures) so the game can draw them at the size it needs (scripts/svg_art.gd).
    open(path + '.import', 'w', newline='\n').write('[remap]\n\nimporter="keep"\n')


for name, d in EYES.items():
    write('eyes', name + '_fill', fill(d))
    write('eyes', name + '_line', stroke(d) + (LASHES if name == 'lash' else ''))
for name, body in CLOSED.items():
    write('eyes', 'closed_' + name, body)
# ---- Freckles (their own slot since the face pass, 2026-10-19, so they go with any cheeks: bold dots in a loose cluster over the cheek,
# drawn in a dark brown by the game, as the owner's salute reference) ---------------------------------------------------------------------
FRECKLES = {
    'dots': ''.join(fill(ellipse_d(x, y, r, r)) for x, y, r in ((84, 104, 15), (128, 92, 15), (170, 106, 14), (102, 146, 14),
                                                               (150, 150, 15))),
    'sprinkle': ''.join(fill(ellipse_d(x, y, 10, 10)) for x, y in ((74, 110), (106, 92), (140, 98), (176, 112), (92, 140), (126, 132),
                                                                    (160, 142), (116, 168))),
    'few': ''.join(fill(ellipse_d(x, y, 16, 16)) for x, y in ((96, 110), (148, 104), (124, 150))),
}

for folder, table in (('brows', BROWS), ('noses', NOSES), ('cheeks', CHEEKS), ('mouths', MOUTHS), ('freckles', FRECKLES)):
    for name, body in table.items():
        write(folder, name, body)
print('face parts written to', ROOT)
