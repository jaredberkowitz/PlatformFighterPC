extends RefCounted
## The menus' painted look, drawn entirely in code (no images): a sunny fantasy sky with floating islands, castles, clouds and halo rings;
## brush-stroke banners and ribbons with ragged ends; ink-outlined cards; and small key glyphs. Every shape is deterministic (seeded),
## so the screens look the same on every run, and everything here only draws: the screens own the layout, the text and the input.
##
## Palette (`INK`, `INDIGO`, ...) is the menus' own, after docs/MENU_STYLE.md: warm highlights, cool shadows, deep indigo ink rather than
## black. The text stays live (drawn with the menu font, outlined), never baked into a texture.

const D := preload("res://ui/ui_draw.gd")

const INK := D.INK
const INDIGO := Color("25214d")
const SKY_TOP := Color("3b86dd")
const SKY := Color("67b8f7")
const HORIZON := Color("cdeffc")
const CLOUD := Color("fff0d2")
const CLOUD_SHADE := Color("c9c3ea")
const SUN := Color("ffc943")
const CORAL := Color("f05d5e")
const BLUE := Color("2778e8")
const VIOLET := Color("9256d8")
const GREEN := Color("27a879")
const TEAL := Color("25a7b3")
const GLOW := Color("b5f4ff")
const CREAM := D.CREAM


# ---- Shape helpers -------------------------------------------------------------------------------------------------------------------

## A repeatable pseudo-random number in -1..1 for an integer seed.
static func noise(seed: int) -> float:
	var x := (seed * 1103515245 + 12345) & 0x7fffffff
	x = (x ^ (x >> 13)) * 1274126177 & 0x7fffffff
	return float(x % 20001) / 10000.0 - 1.0


## `points` (a closed outline) with every edge broken into pieces about `step` long and each new point pushed in or out by up to `amp`:
## a hand-painted, slightly uneven edge. Deterministic for a seed.
static func rough(points: PackedVector2Array, seed: int, amp: float, step := 18.0) -> PackedVector2Array:
	var out := PackedVector2Array()
	var n := points.size()
	for i in n:
		var a := points[i]
		var b := points[(i + 1) % n]
		var along := b - a
		var pieces := maxi(1, int(along.length() / step))
		var across := along.orthogonal().normalized()
		for k in pieces:
			var u := float(k) / pieces
			var off := 0.0 if k == 0 else noise(seed + i * 131 + k * 17) * amp
			out.append(a + along * u + across * off)
	return out


## A painted brush band across `rect`: a slanted stroke whose ends break into ragged bristle points, with a darker under-shadow, a few
## lighter streaks along it (the brush's grain) and a pale inner edge on top. `lean` slants it (pixels the top runs ahead of the bottom).
static func brush_band(c: CanvasItem, rect: Rect2, fill: Color, seed: int, lean := 18.0, shadow := true, ends := 1.0) -> void:
	# (Since the UI pass, docs/UI_REVIEW.md: a crisp slanted banner with chevron-cut ends, a gradient, a lit top edge and an accent
	# stripe under it, in the same construction as the cards; the ragged bristle stroke beside crisp panels read as two styles at once.)
	_banner(c, rect, fill, lean, shadow, ends)


## A banner (`brush_band`'s shape): a slanted band, its ends cut into a shallow chevron, lighter at the top, a lit top edge, an ink
## outline, a hard shadow and a thin gold stripe along its foot.
static func _banner(c: CanvasItem, rect: Rect2, fill: Color, lean: float, shadow: bool, ends: float) -> void:
	if paper_style() > 0:
		_paper_banner(c, rect, fill, lean, shadow, ends)
		return
	var h := rect.size.y
	var x0 := rect.position.x
	var x1 := rect.end.x
	var y0 := rect.position.y
	var tip := 14.0 * ends
	var pts := PackedVector2Array([Vector2(x0 + lean, y0), Vector2(x1 + lean * 0.5, y0), Vector2(x1 + tip, y0 + h * 0.5),
		Vector2(x1 - lean * 0.5, y0 + h), Vector2(x0, y0 + h), Vector2(x0 + lean * 0.5 - tip, y0 + h * 0.5)])
	if shadow:
		var sh := PackedVector2Array()
		for p in pts:
			sh.append(p + SHADOW)
		c.draw_colored_polygon(sh, Color(INK, 0.5 * fill.a))
	var top := fill.lightened(0.14)
	var bottom := fill.darkened(0.16)
	var cols := PackedColorArray()
	for p in pts:
		cols.append(top.lerp(bottom, clampf((p.y - y0) / maxf(h, 1.0), 0.0, 1.0)))
	c.draw_polygon(pts, cols)
	c.draw_line(Vector2(x0 + lean + 4.0, y0 + 3.0), Vector2(x1 + lean * 0.5 - 4.0, y0 + 3.0), Color(1, 1, 1, 0.35 * fill.a), 2.0, true)
	c.draw_line(Vector2(x0 + 6.0, y0 + h - 4.0), Vector2(x1 - lean * 0.5 - 6.0, y0 + h - 4.0), Color(SUN, 0.9 * fill.a), 3.0, true)
	var closed := pts.duplicate()
	closed.append(pts[0])
	c.draw_polyline(closed, Color(fill.darkened(0.55), fill.a).lerp(INK, 0.4), 3.0, true)


## The light: everything in the menus is lit from the upper left (the sun's glow is behind the logo), so shadows fall down and to the right
## by this much, highlights sit on upper-left edges, and islands are lit on their left.
const SHADOW := Vector2(6, 8)


# ---- Paper look mock (UI paper pass, 2026-10-29: the owner wants the menus in the stages' cut-paper language) -------------------------
## The owner picked A (2026-10-29), so it is the default; `--ui-paper=0` (or PF_UI_PAPER=0) shows the painted tiles from before.
## 0 the painted tiles; 1 (A) cut card: flat paper sheets with pale cut edges lifted on soft shadows, no ink outline; 2 (B) layered
## sheets: A over a second, darker sheet showing beneath it, a little turned; 3 (C) inked paper: today's ink outline kept, with the paper
## fibre, cut edges and lifted shadow. Chosen with PF_UI_PAPER=A|B|C or `--ui-paper=`.
const PAPER_EDGE := Color(1.0, 0.97, 0.9)
static var _paper := -1


static func paper_style() -> int:
	if _paper < 0:
		var pick := OS.get_environment("PF_UI_PAPER")
		for a in OS.get_cmdline_user_args():
			if a.begins_with("--ui-paper="):
				pick = a.substr(11)
		_paper = maxi(0, "-ABC".find(pick.to_upper().left(1))) if pick != "" else 1
	return _paper


## A paper sheet's soft lifted shadow: two offset layers, nearer and darker, then farther and fainter (`lift` raises the sheet).
static func _paper_shadow(c: CanvasItem, shape: PackedVector2Array, alpha: float, lift := 0.0) -> void:
	for layer in [[Vector2(2, 3), 0.2], [Vector2(5, 7) + Vector2(lift * 0.4, lift * 0.8), 0.12]]:
		var sh := PackedVector2Array()
		for q in shape:
			sh.append(q + (layer[0] as Vector2))
		c.draw_colored_polygon(sh, Color(INK, float(layer[1]) * alpha))


## The paper's fibre over a sheet: short faint strokes, light and dark, placed by the seed (the stages' faint fibre, at menu scale).
static func _paper_fibre(c: CanvasItem, r: Rect2, seed: int, alpha: float) -> void:
	var n := clampi(int(r.size.x * r.size.y / 700.0), 6, 70)
	for i in n:
		var at := r.position + Vector2((noise(seed * 7 + i * 13) * 0.5 + 0.5) * r.size.x, (noise(seed * 11 + i * 29) * 0.5 + 0.5) * r.size.y)
		var dir := Vector2.from_angle(noise(seed + i * 41) * PI) * (4.0 + 5.0 * (noise(seed + i * 53) * 0.5 + 0.5))
		var col := Color(1, 1, 1, 0.09 * alpha) if i % 2 == 0 else Color(INK, 0.05 * alpha)
		c.draw_line(at, at + dir, col, 1.2, true)


## A sheet's cut edge: pale along the edges that face the light (top and left), darker along the others, so it reads as card with a
## thickness, as the stages' cut edges do.
static func _paper_edges(c: CanvasItem, shape: PackedVector2Array, fill: Color, width := 2.0) -> void:
	var n := shape.size()
	var centre := Vector2.ZERO
	for q in shape:
		centre += q
	centre /= n
	for i in n:
		var a := shape[i]
		var b := shape[(i + 1) % n]
		var out := (b - a).orthogonal().normalized()
		if out.dot((a + b) * 0.5 - centre) < 0.0:
			out = -out
		var lit := out.dot(Vector2(-0.6, -0.8)) > 0.0
		var inset := -out * width * 0.5
		c.draw_line(a + inset, b + inset, Color(PAPER_EDGE, 0.9 * fill.a) if lit else Color(fill.darkened(0.45), 0.8 * fill.a), width, true)


## A card in the paper look (see `card` for the arguments; `style` from `paper_style`).
static func _paper_card(c: CanvasItem, rect: Rect2, fill: Color, seed: int, cut: float, outline: float, border: Color, lift: float,
		line: Color, slab: float, shadow: bool, style: int) -> PackedVector2Array:
	var r := rect
	r.position.y -= lift
	var short := minf(r.size.x, r.size.y)
	if cut < 0.0:
		cut = clampf(short * 0.13, 8.0, 20.0)
	if outline < 0.0:
		outline = 3.0 if short < 50.0 else 4.0
	var shape := card_shape(r, cut, seed)
	if style == 2:
		# The sheet under it: the same cut, offset down and right and turned a touch, in a deeper shade.
		var under := PackedVector2Array()
		var centre := r.get_center()
		var turn := deg_to_rad(noise(seed * 3 + 5) * 1.6)
		for q in shape:
			under.append(centre + (q - centre).rotated(turn) + Vector2(5, 6))
		if shadow:
			_paper_shadow(c, under, fill.a, lift)
		c.draw_colored_polygon(under, Color(fill.darkened(0.32).lerp(PAPER_EDGE, 0.12), fill.a))
		_paper_edges(c, under, fill.darkened(0.32), 1.5)
	elif shadow:
		_paper_shadow(c, shape, fill.a, lift)
	if slab > 0.0:
		var drop := slab + lift * 0.5
		var body := PackedVector2Array()
		for q in shape:
			body.append(q + Vector2(0, drop))
		c.draw_colored_polygon(body, fill.darkened(0.38))
	# A flat sheet, barely lighter at the top (paper, not a glossy tile).
	var cols := PackedColorArray()
	for q in shape:
		cols.append(fill.lightened(0.05).lerp(fill.darkened(0.05), clampf((q.y - r.position.y) / maxf(r.size.y, 1.0), 0.0, 1.0)))
	c.draw_polygon(shape, cols)
	_paper_fibre(c, r.grow(-4), seed, fill.a)
	if border.a > 0.0:
		var inner := PackedVector2Array()
		var centre2 := r.get_center()
		for q in shape:
			inner.append(centre2 + (q - centre2) * Vector2(1.0 - 14.0 / r.size.x, 1.0 - 14.0 / r.size.y))
		inner.append(inner[0])
		c.draw_polyline(inner, border, 4.0, true)
	_paper_edges(c, shape, fill, 2.5 if short >= 50.0 else 2.0)
	var closed := shape.duplicate()
	closed.append(shape[0])
	if style == 3:
		var edge := line if line.a > 0.0 else Color(fill.darkened(0.62), 1.0).lerp(INK, 0.25)
		c.draw_polyline(closed, edge, outline, true)
	else:
		# (A hairline where the sheet's edge meets the ground, so it stays crisp without an ink outline.)
		c.draw_polyline(closed, Color(fill.darkened(0.55), 0.55 * fill.a), 1.0, true)
	return shape


## The banner in the paper look: the same chevron-ended band as a strip of paper with a cut gold strip under it.
static func _paper_banner(c: CanvasItem, rect: Rect2, fill: Color, lean: float, shadow: bool, ends: float) -> void:
	var h := rect.size.y
	var x0 := rect.position.x
	var x1 := rect.end.x
	var y0 := rect.position.y
	var tip := 14.0 * ends
	var pts := PackedVector2Array([Vector2(x0 + lean, y0), Vector2(x1 + lean * 0.5, y0), Vector2(x1 + tip, y0 + h * 0.5),
		Vector2(x1 - lean * 0.5, y0 + h), Vector2(x0, y0 + h), Vector2(x0 + lean * 0.5 - tip, y0 + h * 0.5)])
	# The gold strip, a separate sheet showing under the band's foot.
	var strip := PackedVector2Array()
	for p in pts:
		strip.append(p + Vector2(4, 6))
	if shadow:
		_paper_shadow(c, strip, fill.a)
	c.draw_colored_polygon(strip, Color(SUN, fill.a))
	_paper_edges(c, strip, SUN, 1.5)
	c.draw_colored_polygon(pts, fill)
	_paper_fibre(c, Rect2(x0, y0, x1 - x0, h).grow(-4), int(x1) + int(y0), fill.a)
	_paper_edges(c, pts, fill, 2.5)
	if paper_style() == 3:
		var closed := pts.duplicate()
		closed.append(pts[0])
		c.draw_polyline(closed, Color(fill.darkened(0.55), fill.a).lerp(INK, 0.4), 3.0, true)


## A card: an angled panel (corners cut on the top left and bottom right) with one recipe everywhere: a soft offset shadow, the fill
## lighter toward the top, a broad pale shape across its upper part (the light falling on it), a highlight along the top edge, and an
## ink outline. `cut` and `outline` below zero pick the standard ones for the card's size (corners 13% of its short side, 8 to 20; a 4
## outline, 3 on small cards), so every card shares one shape language. `border` adds a painted rim inside the outline: kept for the
## few cards that should lead (the Play tile, the roster cards).
## `line` colours the outline (clear: the ink); `slab` draws the card's thickness under it, that many pixels of a darker shade of its
## own colour (a tile with a body, rather than a flat sticker).
## `shadow` false leaves out its short shadow (a focused card stands on the long one from `focus_frame` instead).
static func card(c: CanvasItem, rect: Rect2, fill: Color, seed: int, cut := -1.0, outline := -1.0, border := Color(0, 0, 0, 0), lift := 0.0,
		line := Color(0, 0, 0, 0), slab := 0.0, shadow := true) -> PackedVector2Array:
	if paper_style() > 0:
		return _paper_card(c, rect, fill, seed, cut, outline, border, lift, line, slab, shadow, paper_style())
	var r := rect
	r.position.y -= lift
	var short := minf(r.size.x, r.size.y)
	if cut < 0.0:
		cut = clampf(short * 0.13, 8.0, 20.0)
	if outline < 0.0:
		outline = 3.0 if short < 50.0 else 4.0
	var shape := card_shape(r, cut, seed)
	var sh := PackedVector2Array()
	for q in shape:
		sh.append(q + SHADOW + Vector2(0, lift * 0.6))
	# (A hard offset shadow, the UI pass: the soft one read as a default drop shadow.)
	if shadow:
		c.draw_colored_polygon(sh, Color(INK, (0.3 if slab > 0.0 else 0.45) * fill.a))
	# (By default a deep shade of the card's own colour, not black: the menus' one construction language, so cards read as painted tiles
	# rather than stickers. Dark cards come out near the ink anyway.)
	var edge := line if line.a > 0.0 else Color(fill.darkened(0.62), 1.0).lerp(INK, 0.25)
	if slab > 0.0:
		# The body under the face: the same shape dropped by `slab` (more as the card lifts toward the viewer), in a deep shade of the fill.
		var drop := slab + lift * 0.5
		var body := PackedVector2Array()
		for q in shape:
			body.append(q + Vector2(0, drop))
		c.draw_colored_polygon(body, fill.darkened(0.42))
		var body_closed := body.duplicate()
		body_closed.append(body[0])
		c.draw_polyline(body_closed, edge, outline, true)
	var top := fill.lightened(0.12)
	var bottom := fill.darkened(0.14)
	var cols := PackedColorArray()
	for q in shape:
		cols.append(top.lerp(bottom, clampf((q.y - r.position.y) / maxf(r.size.y, 1.0), 0.0, 1.0)))
	c.draw_polygon(shape, cols)
	# The light falling across the upper part: one broad pale shape with a slanted lower edge.
	var band := PackedVector2Array([r.position + Vector2(cut, 0), Vector2(r.end.x, r.position.y), Vector2(r.end.x, r.position.y + r.size.y * 0.22),
		Vector2(r.position.x, r.position.y + r.size.y * 0.46), r.position + Vector2(0, cut)])
	c.draw_colored_polygon(band, Color(1, 1, 1, 0.07 * fill.a))
	# A bevel: a darker inner band along the bottom edge, as if the card's face turns away there (the UI pass: brushed grain strokes
	# across every card read as smudges, not material).
	var bevel := clampf(short * 0.07, 3.0, 8.0)
	c.draw_colored_polygon(PackedVector2Array([Vector2(r.position.x, r.end.y - bevel), Vector2(r.end.x - cut - bevel * 0.4, r.end.y - bevel),
		Vector2(r.end.x - cut, r.end.y), Vector2(r.position.x, r.end.y)]), Color(fill.darkened(0.3), 0.55 * fill.a))
	if border.a > 0.0:
		var inner := PackedVector2Array()
		var centre := r.get_center()
		for q in shape:
			inner.append(centre + (q - centre) * Vector2(1.0 - 14.0 / r.size.x, 1.0 - 14.0 / r.size.y))
		inner.append(inner[0])
		c.draw_polyline(inner, border, 5.0, true)
	c.draw_line(r.position + Vector2(cut + 3.0, outline * 0.5 + 2.5), Vector2(r.end.x - 5.0, r.position.y + outline * 0.5 + 2.5),
		Color(1, 1, 1, 0.45 * fill.a), 2.5, true)
	var closed := shape.duplicate()
	closed.append(shape[0])
	c.draw_polyline(closed, edge, outline, true)
	return shape


## The card's outline (cut corners, its edges very slightly uneven, as if inked by hand).
static func card_shape(r: Rect2, cut: float, seed: int) -> PackedVector2Array:
	var p := PackedVector2Array([
		r.position + Vector2(cut, 0), Vector2(r.end.x, r.position.y), Vector2(r.end.x, r.end.y - cut),
		Vector2(r.end.x - cut, r.end.y), Vector2(r.position.x, r.end.y), r.position + Vector2(0, cut)])
	# (Crisp since the UI pass, docs/UI_REVIEW.md: the hand-inked wobble beside crisp panels was two edge languages at once.)
	return p


## Focus feedback for a card: `focus_frame` (kept under this name for its callers; `colour` is no longer used: focus is shape and
## depth, not a colour, docs/UI_DESIGN_SYSTEM.md, daylight).
static func card_glow(c: CanvasItem, rect: Rect2, _colour: Color, strength: float, cut := -1.0) -> void:
	focus_frame(c, rect, strength, cut)


## How far the focused card's long hard shadow falls (down and right, with the light), against `SHADOW` at rest.
const FOCUS_SHADOW := Vector2(14, 16)


## The focused card's frame (the owner's daylight pick, 2026-10-19): a long hard ink shadow, then a thick frame, ink outside and white
## inside, round the card's own cut shape. Drawn before the card, so the card sits in it. `below` extends it down over the card's body
## (a tile's slab). Its weights follow the card's size so a small button is not drowned. `strength` 0..1 (the focus easing in).
static func focus_frame(c: CanvasItem, rect: Rect2, strength: float, cut := -1.0, below := 0.0) -> void:
	if strength <= 0.01:
		return
	var short := minf(rect.size.x, rect.size.y)
	if cut < 0.0:
		cut = clampf(short * 0.13, 8.0, 20.0)
	if paper_style() == 1 or paper_style() == 2:
		# (Paper: a pale mount behind the focused card, a sheet it sits on, on a long soft shadow; C keeps the ink frame.)
		var m := clampf(short * 0.06, 4.0, 8.0)
		var mount := card_shape(Rect2(rect.position, rect.size + Vector2(0, below)).grow(m), cut + m * 0.5, 0)
		var fall2 := FOCUS_SHADOW * clampf(short / 120.0, 0.45, 1.0) * strength
		for layer in [[fall2 * 0.5, 0.22], [fall2, 0.16]]:
			var sh2 := PackedVector2Array()
			for q in mount:
				sh2.append(q + (layer[0] as Vector2))
			c.draw_colored_polygon(sh2, Color(INK, float(layer[1]) * strength))
		c.draw_colored_polygon(mount, Color(PAPER_EDGE, strength))
		_paper_edges(c, mount, Color(PAPER_EDGE, strength), 2.0)
		var ring := mount.duplicate()
		ring.append(mount[0])
		c.draw_polyline(ring, Color(INK, 0.7 * strength), 2.0, true)
		return
	var white := clampf(short * 0.05, 3.0, 6.0)
	var ink := clampf(short * 0.045, 3.0, 6.0)
	var body := Rect2(rect.position, rect.size + Vector2(0, below))
	var outer := body.grow(white + ink)
	var outer_shape := card_shape(outer, cut + (white + ink) * 0.5, 0)
	var sh := PackedVector2Array()
	var fall := FOCUS_SHADOW * clampf(short / 120.0, 0.45, 1.0) * strength
	for q in outer_shape:
		sh.append(q + fall)
	c.draw_colored_polygon(sh, Color(INK, 0.38 * strength))
	c.draw_colored_polygon(outer_shape, Color(INK, strength))
	c.draw_colored_polygon(card_shape(body.grow(white), cut + white * 0.5, 0), Color(1, 1, 1, strength))


static func title(c: CanvasItem, text: String, at: Vector2, size: int, fill := Color.WHITE, outline_scale := 0.16) -> void:
	var f := D.display_font()
	var o := maxi(4, int(size * outline_scale))
	c.draw_string_outline(f, at + Vector2(3, 4), text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, o, Color(INK, 0.4))
	c.draw_string_outline(f, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, o, INK)
	c.draw_string(f, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, fill)


## The same, centred in `rect` (shrunk to fit its width).
static func title_centered(c: CanvasItem, text: String, rect: Rect2, size: int, fill := Color.WHITE, outline_scale := 0.16) -> void:
	var f := D.display_font()
	var s := size
	while s > 10 and f.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, s).x > rect.size.x - 8.0:
		s -= 2
	var w := f.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, s).x
	var y := rect.position.y + (rect.size.y + f.get_ascent(s) - f.get_descent(s)) * 0.5
	title(c, text, Vector2(rect.position.x + (rect.size.x - w) * 0.5, y), s, fill, outline_scale)


## A key glyph: a small cream keycap with an ink edge and its label.
static func key_cap(c: CanvasItem, rect: Rect2, label: String) -> void:
	c.draw_rect(Rect2(rect.position + Vector2(1, 2), rect.size), Color(INK, 0.6))
	var body := rect
	c.draw_rect(body, CREAM)
	c.draw_rect(Rect2(body.position, Vector2(body.size.x, body.size.y * 0.3)), Color(1, 1, 1, 0.5))
	c.draw_rect(body, INK, false, 2.0)
	D.draw_text_centered(c, label, body, int(body.size.y * 0.62), INK)


# ---- The fantasy sky -------------------------------------------------------------------------------------------------------------------

## The sky's colours for a `mood`: 0 the clear day, 1 warm (the concept art's evening: a violet top over pink and a peach horizon), -1
## cool (a deeper blue). Warm goes toward violet rather than straight to coral: blue mixed with coral passes through grey on the way.
static func palette(mood: float) -> Dictionary:
	var warm := clampf(mood, 0.0, 1.0)
	var cool := clampf(-mood, 0.0, 1.0)
	# (Warm earth under vivid turf, as the stages' floating islands are.)
	var rock := Color("a07a5a").lerp(Color("b06a5c"), warm * 0.5).lerp(Color("6a6ea0"), cool * 0.4)
	return {
		"top": SKY_TOP.lerp(Color("6a46d8"), warm * 0.9).lerp(Color("1f5bbf"), cool * 0.6),
		"mid": SKY.lerp(Color("b46fe0"), warm * 0.55).lerp(Color("f28aa6"), warm * warm * 0.5).lerp(Color("3f8fe8"), cool * 0.5),
		"low": HORIZON.lerp(Color("ffcf9a"), warm * 0.85).lerp(Color("a9dcf6"), cool * 0.5),
		"far": Color("a9bfe8").lerp(Color("c99ad8"), warm * 0.7),
		"grass": Color("6fcc3a").lerp(Color("c9a64b"), warm * 0.4),
		"rock": rock,
		"rock_light": rock.lightened(0.2),
		"cream": CLOUD.lerp(Color("ffd6b0"), warm * 0.6),
		"shade": CLOUD_SHADE.lerp(Color("e8a0a8"), warm * 0.6).lerp(Color("9fb8e8"), cool * 0.4),
	}


## The sky's backdrop into `rect`: a gradient from deep sky blue to a pale horizon and the sun's glow.
static func sky_base(c: CanvasItem, rect: Rect2, mood: float) -> void:
	var pal := palette(mood)
	var w := rect.size.x
	var h := rect.size.y
	var o := rect.position
	var low: Color = pal.low
	var bands := [[0.0, pal.top], [0.45, pal.mid], [0.78, low], [1.0, low.lerp(Color.WHITE, 0.2)]]
	for k in bands.size() - 1:
		var y0: float = o.y + h * float(bands[k][0])
		var y1: float = o.y + h * float(bands[k + 1][0])
		c.draw_polygon(PackedVector2Array([Vector2(o.x, y0), Vector2(o.x + w, y0), Vector2(o.x + w, y1), Vector2(o.x, y1)]),
			PackedColorArray([bands[k][1], bands[k][1], bands[k + 1][1], bands[k + 1][1]]))
	# The sun's glow, upper left (where the light comes from), behind the title.
	var sun_at := o + Vector2(w * 0.2, h * 0.04)
	for k in 6:
		c.draw_circle(sun_at, h * (0.08 + k * 0.07), Color(SUN.lerp(Color.WHITE, 0.55), 0.05))
	# Haze lying along the horizon, where the far islands and clouds sit: depth without detail.
	var hz := o.y + h * 0.66
	var hc := low.lerp(Color.WHITE, 0.35)
	c.draw_polygon(PackedVector2Array([Vector2(o.x, hz - h * 0.12), Vector2(o.x + w, hz - h * 0.12), Vector2(o.x + w, hz), Vector2(o.x, hz)]),
		PackedColorArray([Color(hc, 0.0), Color(hc, 0.0), Color(hc, 0.2), Color(hc, 0.2)]))
	c.draw_polygon(PackedVector2Array([Vector2(o.x, hz), Vector2(o.x + w, hz), Vector2(o.x + w, hz + h * 0.14), Vector2(o.x, hz + h * 0.14)]),
		PackedColorArray([Color(hc, 0.2), Color(hc, 0.2), Color(hc, 0.0), Color(hc, 0.0)]))


class Piece extends Control:
	## One piece of a painted scene, drawn once by `paint` (given this node to draw on) and afterwards only moved.
	var paint: Callable

	func _init(p: Callable) -> void:
		paint = p
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		paint.call(self)


class SkyScene extends Control:
	## The fantasy sky, live: a gradient with the sun's glow, a halo of rings, far islands in haze, near islands with grass and castles,
	## banks of cloud, birds, drifting motes and sparkles, and (with `foliage`) leaves framing the bottom corners. The islands bob and the
	## clouds drift. Each piece is drawn once and then only moved (repainting it all every frame is far too slow); only the birds, motes
	## and sparkles, a handful of tiny shapes, are redrawn. `mood` tints it (see `palette`), `detail` (0..1) thins the decoration out
	## for small panels, `seed` lays it out.
	const Me := preload("res://ui/menu_art.gd")
	var mood := 0.0:
		set(v):
			mood = v
			_build()
	var detail := 1.0
	var seed := 7
	var foliage := false
	var t := 0.0
	var movers: Array = []
	var live: Control
	var built := Vector2.ZERO

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _notification(what: int) -> void:
		if (what == NOTIFICATION_RESIZED or what == NOTIFICATION_READY) and size != built:
			_build()

	func _process(delta: float) -> void:
		if not is_visible_in_tree() or live == null:
			return
		t += delta
		var w := size.x
		for m in movers:
			var n: Control = m.node
			match m.kind:
				0:  # bob
					n.position = m.base + Vector2(0, sin(t * m.speed + m.phase) * m.amount)
				1:  # drift across, wrapping round
					n.position = Vector2(fmod(t * m.speed + m.phase, w * 1.3) - w * 0.15, m.base.y)
				2:  # sway
					n.position = m.base + Vector2(sin(t * m.speed + m.phase) * m.amount, 0)
		live.queue_redraw()

	func _add(paint: Callable, at: Vector2, kind := -1, speed := 0.0, phase := 0.0, amount := 0.0) -> void:
		var p := Piece.new(paint)
		p.position = at
		add_child(p)
		if kind >= 0:
			movers.append({"node": p, "base": at, "kind": kind, "speed": speed, "phase": phase, "amount": amount})

	func _build() -> void:
		for ch in get_children():
			remove_child(ch)
			ch.queue_free()
		movers.clear()
		live = null
		built = size
		var w := size.x
		var h := size.y
		if w < 4.0 or h < 4.0:
			return
		var pal: Dictionary = Me.palette(mood)
		var full := Rect2(Vector2.ZERO, size)
		var m := mood
		_add(func(c): Me.sky_base(c, full, m), Vector2.ZERO)
		var sun_at := Vector2(w * 0.66, h * 0.1)
		var halo_r := w * 0.11
		# The halo is a quiet landmark, not a focal point: the UI leads. Its few sparkles stay put.
		_add(func(c): Me._halo_rings(c, Vector2.ZERO, halo_r, 0.5), sun_at + Vector2(0, h * 0.02))
		_add(func(c):
			for k in 3:
				var a := TAU * (0.1 + k * 0.31)
				Me.sparkle(c, Vector2(cos(a) * halo_r * 1.2, sin(a) * halo_r * 0.34), 5.0, Color(1, 1, 1, 0.55)), sun_at + Vector2(0, h * 0.02))
		# Far islands in the haze: a few, spread along a band at the horizon (fixed places, so the picture is always the same).
		var far: Color = pal.far
		var far_spots := [[0.16, 0.47, 0.05, false], [0.44, 0.52, 0.035, true], [0.6, 0.44, 0.045, false], [0.86, 0.5, 0.04, true]]
		for k in far_spots.size() if detail > 0.5 else 2:
			var f: Array = far_spots[k]
			var s: float = w * float(f[2])
			var castle: bool = f[3]
			var sd := seed * 31 + k
			_add(func(c): Me.island(c, Vector2.ZERO, s, far, far.darkened(0.06), far.lightened(0.08), sd, castle, 0.0),
				Vector2(w * float(f[0]), h * float(f[1])), 0, 0.35, k, 2.0)
		# Near islands: the castle island under the halo and one at each edge, half off screen, framing the middle.
		var grass: Color = pal.grass
		var rock: Color = pal.rock
		var light: Color = pal.rock_light
		var near := [[sun_at + Vector2(0, h * 0.22), 0.07, 1, true, 1.0, true, 0.3, 0.0, 3.0],
			[Vector2(w * 0.03, h * 0.3), 0.12, 2, false, 1.0, false, 0.25, 1.0, 3.0],
			[Vector2(w * 0.97, h * 0.36), 0.1, 3, false, 1.0, false, 0.28, 2.0, 3.0]]
		for n in near:
			var s: float = w * float(n[1])
			var sd: int = seed * 7 + int(n[2])
			var castle: bool = n[3]
			var shade: float = n[4]
			var tall: bool = n[5]
			_add(func(c): Me.island(c, Vector2.ZERO, s, grass, rock, light, sd, castle, shade, tall), n[0], 0, n[6], n[7], n[8])
		# Clouds in two layers: smaller banks far off at the horizon, faded into the haze, and a few big ones near, low and half off the
		# bottom; between them some thin wisps. Each bank has its own silhouette; they drift slowly, the near ones a little faster.
		var cream: Color = pal.cream
		var shade_c: Color = pal.shade
		var haze: Color = pal.low
		var far_banks := [[0.08, 0.7, 0.16], [0.36, 0.73, 0.12], [0.62, 0.69, 0.18], [0.9, 0.72, 0.13]]
		for k in far_banks.size() if detail > 0.5 else 2:
			var b: Array = far_banks[k]
			var s: float = w * float(b[2])
			var sd := seed + 40 + k
			_add(func(c): Me.cloud(c, Vector2.ZERO, s, cream.lerp(haze, 0.45), shade_c.lerp(haze, 0.55), sd), Vector2(0, h * float(b[1])), 1,
				2.0 + 0.5 * k, float(b[0]) * w * 1.3 + w * 0.15)
		# Two faint banks high up, almost lost in the sky's colour: depth in the open middle without drawing the eye.
		var mid: Color = pal.mid
		if detail > 0.5:
			for k in 2:
				var s := w * (0.2 + 0.05 * k)
				var sd := seed + 60 + k
				_add(func(c): Me.cloud(c, Vector2.ZERO, s, cream.lerp(mid, 0.72), shade_c.lerp(mid, 0.8), sd), Vector2(0, h * (0.4 + 0.05 * k)), 1,
					1.5, w * (0.45 + 0.6 * k))
		for k in 3:
			var wy := h * (0.22 + 0.09 * k)
			var ws := w * (0.16 + 0.05 * absf(Me.noise(seed + 70 + k)))
			var sd := seed + 70 + k
			_add(func(c): Me.wisp(c, Vector2.ZERO, ws, Color(1, 1, 1, 0.18), sd), Vector2(0, wy), 1, 3.0, w * (0.25 + 0.4 * k))
		var near_banks := [[0.14, 1.0, 0.3], [0.55, 1.02, 0.26], [0.92, 0.99, 0.32]]
		for k in near_banks.size() if detail > 0.5 else 1:
			var b: Array = near_banks[k]
			var s: float = w * float(b[2])
			var sd := seed + 80 + k
			_add(func(c): Me.cloud(c, Vector2.ZERO, s, cream, shade_c, sd), Vector2(0, h * float(b[1])), 1, 4.0 + k, float(b[0]) * w * 1.3 + w * 0.15)
		if foliage:
			_add(func(c): Me.leaves(c, full, -1.0, 3), Vector2.ZERO)
			_add(func(c): Me.leaves(c, full, 1.0, 9), Vector2.ZERO)
		live = Piece.new(_paint_live)
		add_child(live)

	## The small moving things: birds, motes and sparkles, and the sparkles circling the halo.
	func _paint_live(c: CanvasItem) -> void:
		var w := size.x
		var h := size.y
		# A few birds far off, and a handful of motes rising slowly (quiet ambient movement; nothing is built per frame).
		var bird := Color(1, 1, 1, 0.7)
		for k in int(3 * detail):
			var bx := fmod(w * (0.2 + 0.27 * k) + t * 9.0, w)
			var by := h * (0.18 + 0.05 * Me.noise(seed + 200 + k))
			var flap := sin(t * 5.0 + k) * 2.0
			c.draw_line(Vector2(bx - 6, by - 2 + flap), Vector2(bx, by + 1), bird, 1.6, true)
			c.draw_line(Vector2(bx, by + 1), Vector2(bx + 6, by - 2 + flap), bird, 1.6, true)
		for k in int(12 * detail):
			var mx := w * absf(Me.noise(seed + 400 + k))
			var my := fmod(h * absf(Me.noise(seed + 500 + k)) - t * (4.0 + 3.0 * absf(Me.noise(seed + 600 + k))) + h * 4.0, h)
			c.draw_circle(Vector2(mx, my), 1.6, Color(GLOW, 0.3 + 0.12 * sin(t * 1.2 + k)))


## A floating island centred at `at`, `size` wide: a grassy slab with a lip that overhangs and drips a little, over a body of rock
## that tapers to a main point with smaller ones beside it. Lit from the upper left: the left facets of the rock are lighter, the lip
## casts a band of shadow on the rock, and the grass catches light along its top. `shade` (0..1) is how much of that light and shadow it
## gets (0 for a pale one far off in the haze); `castle` puts a castle on it, `tall` makes its rock longer.
static func island(c: CanvasItem, at: Vector2, size: float, top: Color, rock: Color, rock_light: Color, seed: int, castle := false, shade := 1.0,
		tall := false, bare := false) -> void:
	var hw := size * 0.5
	var depth := size * (1.05 if tall else 0.82)
	var j := noise(seed) * size * 0.05
	var body := PackedVector2Array([at + Vector2(-hw * 0.98, 0), at + Vector2(-hw * 0.9, size * 0.12), at + Vector2(-hw * 0.66, size * 0.2),
		at + Vector2(-hw * 0.52, depth * 0.5), at + Vector2(-hw * 0.34, size * 0.28), at + Vector2(-hw * 0.06 + j, depth),
		at + Vector2(hw * 0.2, size * 0.34), at + Vector2(hw * 0.4, depth * 0.56), at + Vector2(hw * 0.58, size * 0.22),
		at + Vector2(hw * 0.88, size * 0.12), at + Vector2(hw * 0.98, 0)])
	body = rough(body, seed, size * 0.012, size * 0.1)
	var rock_shadow := rock.darkened(0.18 * shade)
	c.draw_colored_polygon(body, rock_shadow)
	# The lit facets: the left of the rock down to the main point.
	var lit := PackedVector2Array([at + Vector2(-hw * 0.98, 0), at + Vector2(-hw * 0.9, size * 0.12), at + Vector2(-hw * 0.66, size * 0.2),
		at + Vector2(-hw * 0.52, depth * 0.5), at + Vector2(-hw * 0.34, size * 0.28), at + Vector2(-hw * 0.06 + j, depth),
		at + Vector2(-hw * 0.02 + j * 0.5, size * 0.3), at + Vector2(hw * 0.12, 0)])
	c.draw_colored_polygon(rough(lit, seed + 3, size * 0.01, size * 0.1), rock.lerp(rock_light, 0.6 * shade))
	if shade > 0.5 and not bare:
		# A pale band of strata just under the turf, stones bedded in the earth (lit on their upper left), and roots trailing from under
		# the lip, as on the stages' islands.
		var band_top := PackedVector2Array()
		for k in 9:
			var u := float(k) / 8.0 * 2.0 - 1.0
			band_top.append(at + Vector2(u * hw * 0.9, size * (0.08 + 0.01 * sin(u * 5.0 + seed))))
		var band_low := PackedVector2Array()
		for k in 9:
			var u := 1.0 - float(k) / 8.0 * 2.0
			band_low.append(at + Vector2(u * hw * 0.82, size * (0.13 + 0.012 * sin(u * 4.0 + seed * 2))))
		var strata := band_top.duplicate()
		strata.append_array(band_low)
		c.draw_colored_polygon(strata, Color(rock_light.lightened(0.15), 0.75))
		for k in 3:
			var u := -0.45 + 0.42 * k + noise(seed + k) * 0.12
			var sp := at + Vector2(u * hw, size * (0.2 + 0.05 * k * (1.0 - absf(u))))
			var sr := size * (0.035 - 0.006 * k)
			c.draw_circle(sp, sr, rock.lerp(Color(0.62, 0.6, 0.66), 0.5))
			c.draw_circle(sp + Vector2(-sr * 0.3, -sr * 0.3), sr * 0.5, rock.lerp(Color(0.8, 0.78, 0.82), 0.5))
		for k in 4:
			var rx := at.x + hw * (-0.7 + 0.45 * k + noise(seed * 3 + k) * 0.1)
			var ry := at.y + size * 0.06
			c.draw_polyline(PackedVector2Array([Vector2(rx, ry), Vector2(rx + size * 0.01, ry + size * 0.05), Vector2(rx - size * 0.006, ry + size * (0.09 + 0.02 * float(k % 2)))]),
				Color(rock_shadow.darkened(0.4), 0.85), maxf(1.0, size * 0.008), true)
		# A seam of strata across the rock, and a thin waterfall from under the lip.
		c.draw_line(at + Vector2(-hw * 0.7, size * 0.16), at + Vector2(hw * 0.6, size * 0.19), Color(rock_shadow.darkened(0.15), 0.6), maxf(1.0, size * 0.012), true)
		c.draw_line(at + Vector2(hw * 0.5, size * 0.05), at + Vector2(hw * 0.5, size * 0.75), Color(1, 1, 1, 0.3), maxf(1.2, size * 0.018), true)
	# The grass slab: a gentle dome on top, a lip with a few drips, and the lip's shadow on the rock.
	var slab := PackedVector2Array()
	var steps := 14
	for k in steps + 1:
		var u := float(k) / steps * 2.0 - 1.0
		slab.append(at + Vector2(u * hw * 1.06, -size * (0.05 + 0.03 * (1.0 - u * u))))
	var lip := PackedVector2Array()
	for k in steps + 1:
		var u := 1.0 - float(k) / steps * 2.0
		var drip := maxf(0.0, sin(u * 9.0 + float(seed))) * size * 0.035
		lip.append(at + Vector2(u * hw * 1.04, size * 0.045 + drip))
	var lip_shadow := PackedVector2Array()
	for q in lip:
		lip_shadow.append(q + Vector2(size * 0.01, size * 0.035))
	if shade > 0.0:
		var band := lip.duplicate()
		lip_shadow.reverse()
		band.append_array(lip_shadow)
		c.draw_colored_polygon(band, Color(rock_shadow.darkened(0.3), 0.55 * shade))
	var grass := slab.duplicate()
	grass.append_array(lip)
	c.draw_colored_polygon(grass, top.darkened(0.12 * shade))
	if bare:
		# A platform to stand on: its grass top seen a little from above, an oval lit from the upper left, with a pale rim along the front
		# edge; the slab below it is the front of the turf.
		var face := PackedVector2Array()
		var rim := PackedVector2Array()
		var centre := at + Vector2(0, -size * 0.055)
		for k in 49:
			var a := TAU * k / 48.0
			var q := centre + Vector2(cos(a) * hw * 1.04, sin(a) * size * 0.05)
			face.append(q)
			if a > 0.15 and a < PI - 0.15:
				rim.append(q)
		c.draw_colored_polygon(face, top.lightened(0.1))
		var sunlit := PackedVector2Array()
		for k in 25:
			var a := PI * 0.55 + PI * 1.1 * k / 24.0
			sunlit.append(centre + Vector2(cos(a) * hw * 0.98, sin(a) * size * 0.042) + Vector2(-hw * 0.06, 0))
		c.draw_colored_polygon(sunlit, top.lightened(0.2))
		c.draw_polyline(rim, top.lightened(0.32), maxf(1.5, size * 0.006), true)
		return
	# The lit top of the grass (the left two thirds), as one broad lighter shape.
	var light := PackedVector2Array()
	for k in 10:
		var u := float(k) / 9.0 * 1.5 - 1.0
		light.append(at + Vector2(u * hw * 1.0, -size * (0.05 + 0.03 * (1.0 - u * u))))
	light.append(at + Vector2(hw * 0.3, size * 0.0))
	light.append(at + Vector2(-hw * 0.95, size * 0.02))
	c.draw_colored_polygon(light, top.lerp(top.lightened(0.25), shade))
	if castle:
		_castle(c, at + Vector2(-size * 0.05, -size * 0.06), size * (0.5 if tall else 0.35), Color(rock_light.lerp(Color.WHITE, 0.3), 1.0), Color(rock.darkened(0.1), 1.0))
	elif shade > 0.5 and not bare:
		# A couple of round trees, lit on their left.
		for k in 2:
			var tx := at.x + size * (-0.25 + 0.42 * k)
			var tr := size * (0.075 - 0.015 * k)
			c.draw_circle(Vector2(tx, at.y - size * 0.07 - tr), tr, top.darkened(0.2))
			c.draw_circle(Vector2(tx - tr * 0.25, at.y - size * 0.07 - tr * 1.2), tr * 0.62, top.lightened(0.12))


static func _castle(c: CanvasItem, base: Vector2, size: float, light: Color, dark: Color) -> void:
	var towers := [[-0.32, 0.55, 0.14], [-0.1, 0.95, 0.16], [0.14, 0.7, 0.13], [0.33, 0.45, 0.11]]
	for tw in towers:
		var x := base.x + size * float(tw[0])
		var tall: float = size * float(tw[1])
		var half: float = size * float(tw[2]) * 0.5
		c.draw_rect(Rect2(x - half, base.y - tall, half * 2.0, tall), light)
		c.draw_rect(Rect2(x, base.y - tall, half, tall), Color(dark, 0.35))
		c.draw_colored_polygon(PackedVector2Array([Vector2(x - half * 1.3, base.y - tall), Vector2(x, base.y - tall - half * 2.6),
			Vector2(x + half * 1.3, base.y - tall)]), dark)
	c.draw_rect(Rect2(base.x - size * 0.35, base.y - size * 0.25, size * 0.7, size * 0.25), light)


## A bank of cloud at `at` (the middle of its base), about `size` across: billows of different sizes on a flat base, the tallest a little
## left of the middle. Lit from the upper left: a cool shadow along the base, the body, and light on the billows' upper-left edges.
## Each `seed` gives its own silhouette.
static func cloud(c: CanvasItem, at: Vector2, size: float, cream: Color, shade: Color, seed: int) -> void:
	var n := 4 + int(absf(noise(seed)) * 4.0)
	var centres := PackedVector2Array()
	var radii := PackedFloat32Array()
	var x := -size * 0.5
	for k in n:
		var u := float(k) / float(n - 1)
		var r := size * (0.1 + 0.11 * absf(noise(seed + 10 + k))) * (0.6 + 0.55 * sin(PI * clampf(u * 1.1, 0.0, 1.0)))
		centres.append(at + Vector2(x + r * 0.6, -r * 0.62))
		radii.append(r)
		x += r * (1.0 + 0.3 * absf(noise(seed + 30 + k)))
	# Centre the bank on `at`.
	var shift := Vector2((-size * 0.5 - (x - size * 0.5)) * 0.5 + size * 0.0, 0) + Vector2((size * 0.5 - x) * 0.5, 0)
	var light := cream.lerp(Color.WHITE, 0.6)
	var left := centres[0].x - radii[0] + shift.x
	var right := centres[n - 1].x + radii[n - 1] + shift.x
	var base_h := size * 0.07
	# The shadow along the base, then the light rims, then the body inset from them (so only the upper-left edges stay light).
	c.draw_rect(Rect2(Vector2(left + base_h, at.y - base_h), Vector2(right - left - base_h * 2.0, base_h * 1.6)), shade)
	c.draw_circle(Vector2(left + base_h, at.y - base_h * 0.2), base_h, shade)
	c.draw_circle(Vector2(right - base_h, at.y - base_h * 0.2), base_h, shade)
	for k in n:
		c.draw_circle(centres[k] + shift + Vector2(0, radii[k] * 0.1), radii[k], shade)
	for k in n:
		c.draw_circle(centres[k] + shift + Vector2(-radii[k] * 0.06, -radii[k] * 0.08), radii[k] * 0.96, light)
	for k in n:
		c.draw_circle(centres[k] + shift + Vector2(radii[k] * 0.08, radii[k] * 0.06), radii[k] * 0.86, cream)
	c.draw_rect(Rect2(Vector2(left + base_h * 1.2, at.y - base_h * 1.9), Vector2(right - left - base_h * 2.4, base_h * 1.1)), cream)


## A thin streak of high cloud, for variety between the banks.
static func wisp(c: CanvasItem, at: Vector2, size: float, colour: Color, seed: int) -> void:
	var pts := PackedVector2Array()
	for k in 25:
		var a := TAU * k / 24.0
		var bulge := 1.0 + 0.25 * sin(a * 3.0 + float(seed))
		pts.append(at + Vector2(cos(a) * size * 0.5, sin(a) * size * 0.06 * bulge))
	c.draw_colored_polygon(pts, colour)


## A four-pointed sparkle.
static func sparkle(c: CanvasItem, at: Vector2, r: float, col: Color) -> void:
	c.draw_colored_polygon(PackedVector2Array([at + Vector2(0, -r), at + Vector2(r * 0.25, -r * 0.25), at + Vector2(r, 0), at + Vector2(r * 0.25, r * 0.25),
		at + Vector2(0, r), at + Vector2(-r * 0.25, r * 0.25), at + Vector2(-r, 0), at + Vector2(-r * 0.25, -r * 0.25)]), col)


## The halo: three flat rings of light (the sparkles circling it are drawn with the sky's moving things).
static func _halo_rings(c: CanvasItem, centre: Vector2, r: float, strength: float) -> void:
	for k in 3:
		var pts := PackedVector2Array()
		var rr := r * (1.0 + k * 0.22)
		for i in 49:
			var a := TAU * i / 48.0
			pts.append(centre + Vector2(cos(a) * rr, sin(a) * rr * 0.28))
		c.draw_polyline(pts, Color(GLOW, (0.55 - k * 0.15) * strength), 3.0 - k * 0.6, true)


## Dark leaves fanning in from a bottom corner (`side` -1 the left, 1 the right) of `rect`: frames the screen without blocking it.
static func leaves(c: CanvasItem, rect: Rect2, side: float, seed: int) -> void:
	var corner := Vector2(rect.position.x if side < 0.0 else rect.end.x, rect.end.y)
	var dark := Color("173a2c")
	var mid := Color("23573c")
	for k in 7:
		var a := deg_to_rad(-90.0 + side * (12.0 + k * 11.0))
		var length := rect.size.y * (0.16 + 0.08 * absf(noise(seed + k)))
		var tip := corner + Vector2(cos(a), sin(a)) * length
		var across := Vector2(cos(a), sin(a)).orthogonal() * length * 0.16
		var mid_pt := corner + (tip - corner) * 0.5
		c.draw_colored_polygon(PackedVector2Array([corner, mid_pt + across, tip, mid_pt - across]), dark if k % 2 == 0 else mid)
