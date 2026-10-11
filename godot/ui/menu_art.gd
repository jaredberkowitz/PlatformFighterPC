extends RefCounted
## The menus' painted look, drawn entirely in code (no images): a sunny fantasy sky with floating islands, castles, clouds and halo rings;
## brush-stroke banners and ribbons with ragged ends; ink-outlined cards; and small key glyphs. Every shape is deterministic (seeded),
## so the screens look the same on every run, and everything here only draws: the screens own the layout, the text and the input.
##
## Palette (`INK`, `INDIGO`, ...) is the menus' own, after docs/MENU_STYLE.md: warm highlights, cool shadows, deep indigo ink rather than
## black. The text stays live (drawn with the menu font, outlined), never baked into a texture.

const D := preload("res://ui/ui_draw.gd")

const INK := Color("171b35")
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
const CREAM := Color("fbf3df")


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
	var pts := PackedVector2Array()
	var h := rect.size.y
	var x0 := rect.position.x
	var x1 := rect.end.x
	var y0 := rect.position.y
	# The top edge (left to right), the ragged right end, the bottom edge (right to left), the ragged left end.
	pts.append(Vector2(x0 + lean, y0))
	pts.append(Vector2(x1 + lean * 0.5, y0 + noise(seed) * 3.0))
	var spikes := 5
	for k in range(1, spikes):
		var u := float(k) / spikes
		var reach := (16.0 + 14.0 * absf(noise(seed + k * 7))) * ends * (1.0 if k % 2 == 1 else 0.35)
		pts.append(Vector2(x1 + lerpf(lean * 0.5, -lean * 0.5, u) + reach, y0 + h * u))
	pts.append(Vector2(x1 - lean * 0.5, y0 + h))
	pts.append(Vector2(x0, y0 + h))
	for k in range(1, spikes):
		var u := 1.0 - float(k) / spikes
		var reach := (16.0 + 14.0 * absf(noise(seed + 50 + k * 7))) * ends * (1.0 if k % 2 == 1 else 0.35)
		pts.append(Vector2(x0 + lean * u - reach, y0 + h * u))
	pts = rough(pts, seed, 2.2, 22.0)
	if shadow:
		var sh := PackedVector2Array()
		for p in pts:
			sh.append(p + Vector2(5, 7))
		c.draw_colored_polygon(sh, Color(INK, 0.45 * fill.a))
	c.draw_colored_polygon(pts, fill)
	# Streaks of the brush along the band.
	for k in 4:
		var y := y0 + h * (0.2 + 0.2 * k) + noise(seed + 300 + k) * 2.0
		var sx := x0 + lean * (1.0 - (y - y0) / h) + 12.0 + 40.0 * absf(noise(seed + 310 + k))
		var ex := x1 - 20.0 - 60.0 * absf(noise(seed + 320 + k))
		var tint := Color(1, 1, 1, 0.08) if k % 2 == 0 else Color(0, 0, 0, 0.07)
		c.draw_line(Vector2(sx, y), Vector2(ex, y), tint, maxf(2.0, h * 0.06), true)
	# A pale edge along the top.
	c.draw_line(Vector2(x0 + lean + 6.0, y0 + 3.0), Vector2(x1 + lean * 0.5 - 8.0, y0 + 3.0), Color(1, 1, 1, 0.3 * fill.a), 2.0, true)


## The light: everything in the menus is lit from the upper left (the sun's glow is behind the logo), so shadows fall down and to the right
## by this much, highlights sit on upper-left edges, and islands are lit on their left.
const SHADOW := Vector2(6, 8)


## A card: an angled panel (corners cut on the top left and bottom right) with one recipe everywhere: a soft offset shadow, the fill
## lighter toward the top, a broad pale shape across its upper part (the light falling on it), a highlight along the top edge, and an
## ink outline. `cut` and `outline` below zero pick the standard ones for the card's size (corners 13% of its short side, 8 to 20; a 4
## outline, 3 on small cards), so every card shares one shape language. `border` adds a painted rim inside the outline: kept for the
## few cards that should lead (the Play tile, the roster cards).
static func card(c: CanvasItem, rect: Rect2, fill: Color, seed: int, cut := -1.0, outline := -1.0, border := Color(0, 0, 0, 0), lift := 0.0) -> PackedVector2Array:
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
	c.draw_colored_polygon(sh, Color(INK, 0.32 * fill.a))
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
	if border.a > 0.0:
		var inner := PackedVector2Array()
		var centre := r.get_center()
		for q in shape:
			inner.append(centre + (q - centre) * Vector2(1.0 - 14.0 / r.size.x, 1.0 - 14.0 / r.size.y))
		inner.append(inner[0])
		c.draw_polyline(inner, border, 5.0, true)
	c.draw_line(r.position + Vector2(cut + 3.0, outline * 0.5 + 2.5), Vector2(r.end.x - 5.0, r.position.y + outline * 0.5 + 2.5),
		Color(1, 1, 1, 0.3 * fill.a), 2.5, true)
	var closed := shape.duplicate()
	closed.append(shape[0])
	c.draw_polyline(closed, INK, outline, true)
	return shape


## The card's outline (cut corners, its edges very slightly uneven, as if inked by hand).
static func card_shape(r: Rect2, cut: float, seed: int) -> PackedVector2Array:
	var p := PackedVector2Array([
		r.position + Vector2(cut, 0), Vector2(r.end.x, r.position.y), Vector2(r.end.x, r.end.y - cut),
		Vector2(r.end.x - cut, r.end.y), Vector2(r.position.x, r.end.y), r.position + Vector2(0, cut)])
	return rough(p, seed, 1.0, 30.0)


## Focus feedback for a card: a few soft rings of `colour` that follow its shape (not a box), `strength` 0..1.
static func card_glow(c: CanvasItem, rect: Rect2, colour: Color, strength: float, cut := -1.0) -> void:
	if strength <= 0.01:
		return
	if cut < 0.0:
		cut = clampf(minf(rect.size.x, rect.size.y) * 0.13, 8.0, 20.0)
	for k in 3:
		var g := rect.grow(3.0 + k * 4.0)
		var shape := PackedVector2Array([g.position + Vector2(cut + k * 2.0, 0), Vector2(g.end.x, g.position.y), Vector2(g.end.x, g.end.y - cut - k * 2.0),
			Vector2(g.end.x - cut - k * 2.0, g.end.y), Vector2(g.position.x, g.end.y), g.position + Vector2(0, cut + k * 2.0)])
		shape.append(shape[0])
		c.draw_polyline(shape, Color(colour, (0.55 - k * 0.17) * strength), 4.0, true)


## Text in the menus' display style: live text, outlined in ink, with a soft drop shadow.
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

## The sky's colours for a `mood`: 0 the clear day, 1 warm (a coral and gold evening), -1 cool (a deeper blue).
static func palette(mood: float) -> Dictionary:
	var warm := clampf(mood, 0.0, 1.0)
	var cool := clampf(-mood, 0.0, 1.0)
	var rock := Color("8a7ea6").lerp(Color("b06a5c"), warm * 0.5).lerp(Color("5f6ea8"), cool * 0.4)
	return {
		"top": SKY_TOP.lerp(Color("e8604f"), warm * 0.85).lerp(Color("1f5bbf"), cool * 0.6),
		"mid": SKY.lerp(Color("f6a35f"), warm * 0.8).lerp(Color("3f8fe8"), cool * 0.5),
		"low": HORIZON.lerp(Color("ffe0a8"), warm * 0.85).lerp(Color("a9dcf6"), cool * 0.5),
		"far": Color("a9bfe8").lerp(Color("e9a99a"), warm * 0.7),
		"grass": Color("57c26a").lerp(Color("c9a64b"), warm * 0.4),
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
		PackedColorArray([Color(hc, 0.0), Color(hc, 0.0), Color(hc, 0.35), Color(hc, 0.35)]))
	c.draw_polygon(PackedVector2Array([Vector2(o.x, hz), Vector2(o.x + w, hz), Vector2(o.x + w, hz + h * 0.14), Vector2(o.x, hz + h * 0.14)]),
		PackedColorArray([Color(hc, 0.35), Color(hc, 0.35), Color(hc, 0.0), Color(hc, 0.0)]))


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
