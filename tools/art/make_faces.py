"""Draws the fighters' faces (godot/art/faces/*.svg) in the concept art's style: big round eyes (a cream ring round a large dark pupil
with a small highlight), round pink cheeks and a small simple mouth, on a 512x512 sheet that the game lays on the front of the head.

Every expression is the same face with its eyes and mouth changed, so the fighters stay recognisable whatever they are doing. Eyelids
are drawn in the placeholder colour #fe00ff, which the game turns into the fighter's own colour (a shade darker). The drawings look to
the right (the game mirrors them when the fighter looks left), so the pupils sit a little toward the right.

Run: python tools/art/make_faces.py
"""
import math

OUT = __file__.replace('\\', '/').rsplit('/tools/', 1)[0] + '/godot/art/faces/'
INK = '#2a1a33'
PUPIL = '#2d1a3c'
CREAM = '#fff4dc'
CHEEK = '#f59393'
MOUTH_IN = '#6b2234'
TONGUE = '#ef7f8e'
LID = '#fe00ff'

EYES = [(160, 236), (352, 236)]
R_EYE = 74       # the cream ring's radius
R_PUPIL = 52
LOOK = 10        # how far the pupils sit toward the right


def svg(body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512">\n{body}</svg>\n'


def cheeks(op=0.85):
    return ''.join(f'  <ellipse cx="{x}" cy="352" rx="40" ry="27" fill="{CHEEK}" opacity="{op}"/>\n' for x in (104, 408))


def eye(cx, cy, scale=1.0, pupil=1.0, lid_top=None, lid_tilt=0.0, lower=None, look=LOOK, uid='e'):
    """A round eye. `lid_top`: how far down the upper lid comes (0 open .. 1 shut, None no lid), `lid_tilt` slants it (degrees,
    positive lowers its inner end), `lower`: how far up a lower lid comes (a squint)."""
    r = R_EYE * scale
    rp = R_PUPIL * scale * pupil
    out = f'  <clipPath id="{uid}"><circle cx="{cx}" cy="{cy}" r="{r:.1f}"/></clipPath>\n'
    out += f'  <circle cx="{cx}" cy="{cy}" r="{r:.1f}" fill="{CREAM}"/>\n'
    px = cx + look * scale
    out += f'  <circle cx="{px:.1f}" cy="{cy + 2:.1f}" r="{rp:.1f}" fill="{PUPIL}"/>\n'
    out += f'  <circle cx="{px - rp * 0.38:.1f}" cy="{cy - rp * 0.4:.1f}" r="{max(5.0, rp * 0.2):.1f}" fill="#ffffff" opacity="0.9"/>\n'
    if lid_top is not None:
        y = cy - r + 2 * r * lid_top
        inner = 1 if cx < 256 else -1
        out += (f'  <g clip-path="url(#{uid})"><rect x="{cx - r - 4:.1f}" y="{cy - r - 60:.1f}" width="{2 * r + 8:.1f}" '
                f'height="{y - (cy - r - 60):.1f}" fill="{LID}" transform="rotate({lid_tilt * inner:.1f} {cx} {y:.1f})"/>'
                f'<line x1="{cx - r - 4:.1f}" y1="{y:.1f}" x2="{cx + r + 4:.1f}" y2="{y:.1f}" stroke="{INK}" stroke-width="10" '
                f'transform="rotate({lid_tilt * inner:.1f} {cx} {y:.1f})"/></g>\n')
    if lower is not None:
        y = cy + r - 2 * r * lower
        out += (f'  <g clip-path="url(#{uid})"><path d="M{cx - r - 4:.1f},{cy + r + 4:.1f} L{cx - r - 4:.1f},{y + 10:.1f} '
                f'Q{cx},{y - 18:.1f} {cx + r + 4:.1f},{y + 10:.1f} L{cx + r + 4:.1f},{cy + r + 4:.1f} Z" fill="{LID}" '
                f'stroke="{INK}" stroke-width="9"/></g>\n')
    out += f'  <circle cx="{cx}" cy="{cy}" r="{r:.1f}" fill="none" stroke="{INK}" stroke-width="9"/>\n'
    return out


def eyes(**kw):
    return eye(*EYES[0], uid='eL', **kw) + eye(*EYES[1], uid='eR', **kw)


def line(d, w=10):
    return f'  <path d="{d}" fill="none" stroke="{INK}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>\n'


def smile(width=34, depth=22, y=356, w=10):
    return line(f'M{256 - width},{y} Q256,{y + depth} {256 + width},{y}', w)


def open_mouth(path, tongue=True):
    out = f'  <clipPath id="m"><path d="{path}"/></clipPath>\n  <path d="{path}" fill="{MOUTH_IN}"/>\n'
    if tongue:
        out += f'  <ellipse cx="256" cy="404" rx="30" ry="16" fill="{TONGUE}" clip-path="url(#m)"/>\n'
    return out + f'  <path d="{path}" fill="none" stroke="{INK}" stroke-width="9" stroke-linejoin="round"/>\n'


def closed_eye(cx, cy, happy=False):
    # A shut eye: a soft arc (down for asleep or blinking, up for a happy squint).
    if happy:
        return line(f'M{cx - 50},{cy + 14} Q{cx},{cy - 40} {cx + 50},{cy + 14}', 12)
    return line(f'M{cx - 52},{cy - 4} Q{cx},{cy + 36} {cx + 52},{cy - 4}', 12)


FACES = {
    # The concept art's face: open, round, a small smile.
    'deadpan': eyes() + cheeks() + smile(),
    'sleepy': eyes(lid_top=0.55) + cheeks(0.7) + smile(26, 10),
    'grumpy': eyes(lid_top=0.36, lid_tilt=16) + cheeks(0.6) + line('M222,370 Q256,354 290,370', 10),
    'smug': eyes(lower=0.32, lid_top=0.22) + cheeks() + line('M224,360 Q262,380 296,350', 10),
    'blink': closed_eye(*EYES[0]) + closed_eye(*EYES[1]) + cheeks() + smile(),
    'happy': closed_eye(*EYES[0], happy=True) + closed_eye(*EYES[1], happy=True) + cheeks(1.0)
    + open_mouth('M206,350 Q256,352 306,350 Q300,420 256,424 Q212,420 206,350 Z'),
    'focus': eyes(lid_top=0.24, lid_tilt=10) + cheeks(0.7) + line('M232,364 L280,364', 10),
    'attack': eyes(lid_top=0.3, lid_tilt=22, pupil=0.85) + cheeks(0.7)
    + open_mouth('M214,348 L298,348 Q294,410 256,416 Q218,410 214,348 Z'),
    'effort': line('M108,206 L196,236 L108,262', 12) + line('M404,206 L316,236 L404,262', 12) + cheeks(0.8)
    + f'  <rect x="206" y="344" width="100" height="44" rx="14" fill="#ffffff" stroke="{INK}" stroke-width="9"/>\n'
    + line('M206,366 L306,366', 6) + line('M240,344 L240,388', 5) + line('M272,344 L272,388', 5),
    'hurt': line('M100,206 L188,236 L100,266', 13) + line('M412,206 L324,236 L412,266', 13) + cheeks(0.6)
    + open_mouth('M216,382 Q232,348 256,362 Q280,348 296,382 Q256,398 216,382 Z', tongue=False),
    'shock': eyes(scale=1.08, pupil=0.55, look=0) + cheeks(0.6)
    + f'  <ellipse cx="256" cy="378" rx="22" ry="27" fill="{MOUTH_IN}" stroke="{INK}" stroke-width="9"/>\n',
}


def spiral(cx, cy):
    pts = []
    for i in range(60):
        a = i / 60 * math.tau * 2.6
        r = 6 + i * 1.05
        pts.append(f'{cx + math.cos(a) * r:.1f},{cy + math.sin(a) * r:.1f}')
    return (f'  <circle cx="{cx}" cy="{cy}" r="{R_EYE}" fill="{CREAM}" stroke="{INK}" stroke-width="9"/>\n'
            + line('M' + ' L'.join(pts), 9))


FACES['dazed'] = spiral(*EYES[0]) + spiral(*EYES[1]) + cheeks(0.6) + line('M220,372 Q238,356 256,372 Q274,388 292,372', 10)

for name, body in FACES.items():
    open(OUT + name + '.svg', 'w', newline='\n').write(svg(body))
    print(name)
