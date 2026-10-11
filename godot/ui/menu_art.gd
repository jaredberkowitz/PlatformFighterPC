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


## A card: an angled panel (corners cut on the top left and bottom right), an offset shadow, the fill lighter toward the top, a broad
## translucent plane across it, a pale inner highlight along the top and left, and a thick ink outline. `border` adds a painted rim
## in that colour inside the outline (the roster cards).
static func card(c: CanvasItem, rect: Rect2, fill: Color, seed: int, cut := 14.0, outline := 4.0, border := Color(0, 0, 0, 0), lift := 0.0) -> PackedVector2Array:
	var r := rect
	r.position.y -= lift
	var p := PackedVector2Array([
		r.position + Vector2(cut, 0), Vector2(r.end.x, r.position.y), Vector2(r.end.x, r.end.y - cut),
		Vector2(r.end.x - cut, r.end.y), Vector2(r.position.x, r.end.y), r.position + Vector2(0, cut)])
	var shape := rough(p, seed, 1.2, 26.0)
	var sh := PackedVector2Array()
	for q in shape:
		sh.append(q + Vector2(6, 8 + lift * 0.6))
	c.draw_colored_polygon(sh, Color(INK, 0.5))
	var top := fill.lightened(0.14)
	var bottom := fill.darkened(0.12)
	var cols := PackedColorArray()
	for q in shape:
		cols.append(top.lerp(bottom, clampf((q.y - r.position.y) / maxf(r.size.y, 1.0), 0.0, 1.0)))
	c.draw_polygon(shape, cols)
	# A broad translucent plane across the lower right.
	var plane := PackedVector2Array([Vector2(r.position.x + r.size.x * 0.45, r.end.y), Vector2(r.end.x, r.position.y + r.size.y * 0.25),
		Vector2(r.end.x, r.end.y - cut), Vector2(r.end.x - cut, r.end.y)])
	c.draw_colored_polygon(plane, Color(0, 0, 0, 0.1))
	if border.a > 0.0:
		var inner := PackedVector2Array()
		var centre := r.get_center()
		for q in shape:
			inner.append(centre + (q - centre) * Vector2(1.0 - 14.0 / r.size.x, 1.0 - 14.0 / r.size.y))
		inner.append(inner[0])
		c.draw_polyline(inner, border, 6.0, true)
	# The highlight along the top and the left.
	c.draw_line(r.position + Vector2(cut + 4.0, 4.0), Vector2(r.end.x - 6.0, r.position.y + 4.0), Color(1, 1, 1, 0.32), 2.5, true)
	c.draw_line(r.position + Vector2(4.0, cut + 4.0), Vector2(r.position.x + 4.0, r.end.y - 8.0), Color(1, 1, 1, 0.18), 2.0, true)
	var closed := shape.duplicate()
	closed.append(shape[0])
	c.draw_polyline(closed, INK, outline, true)
	return shape


## Text in the menus' display style: live text, outlined in ink, with a soft drop shadow.
static func title(c: CanvasItem, text: String, at: Vector2, size: int, fill := Color.WHITE, outline_scale := 0.16) -> void:
	var f := D.font()
	var o := maxi(4, int(size * outline_scale))
	c.draw_string_outline(f, at + Vector2(3, 4), text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, o, Color(INK, 0.4))
	c.draw_string_outline(f, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, o, INK)
	c.draw_string(f, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, fill)


## The same, centred in `rect` (shrunk to fit its width).
static func title_centered(c: CanvasItem, text: String, rect: Rect2, size: int, fill := Color.WHITE, outline_scale := 0.16) -> void:
	var f := D.font()
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
	var sun_at := o + Vector2(w * 0.66, h * 0.1)
	for k in 6:
		c.draw_circle(sun_at, h * (0.08 + k * 0.06), Color(SUN.lerp(Color.WHITE, 0.5), 0.05))


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
		if what == NOTIFICATION_RESIZED and size != built:
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
		_add(func(c): Me._halo_rings(c, Vector2.ZERO, halo_r, 1.0), sun_at + Vector2(0, h * 0.02))
		var rng := RandomNumberGenerator.new()
		rng.seed = seed
		# Far islands in the haze, small and pale.
		var far: Color = pal.far
		for k in int(6 * detail) + 1:
			var x := w * (0.05 + 0.9 * rng.randf())
			var y := h * rng.randf_range(0.3, 0.55)
			var s := w * rng.randf_range(0.035, 0.07)
			var castle := rng.randf() < 0.4
			var sd := seed * 31 + k
			_add(func(c): Me.island(c, Vector2.ZERO, s, far, far.darkened(0.08), far.lightened(0.1), sd, castle, 0.0), Vector2(x, y), 0, 0.35, k, 2.0)
		# Near islands: the castle island under the halo, one large at each side, two small high up.
		var grass: Color = pal.grass
		var rock: Color = pal.rock
		var light: Color = pal.rock_light
		var near := [[sun_at + Vector2(0, h * 0.22), 0.075, 1, true, 1.0, true, 0.3, 0.0, 3.0],
			[Vector2(w * 0.06, h * 0.28), 0.12, 2, false, 1.0, false, 0.25, 1.0, 3.0],
			[Vector2(w * 0.95, h * 0.33), 0.11, 3, true, 1.0, false, 0.28, 2.0, 3.0]]
		if detail > 0.5:
			near.append([Vector2(w * 0.3, h * 0.12), 0.045, 4, false, 0.7, false, 0.33, 3.0, 2.0])
			near.append([Vector2(w * 0.83, h * 0.12), 0.04, 5, false, 0.7, false, 0.31, 4.0, 2.0])
		for n in near:
			var s: float = w * float(n[1])
			var sd: int = seed * 7 + int(n[2])
			var castle: bool = n[3]
			var shade: float = n[4]
			var tall: bool = n[5]
			_add(func(c): Me.island(c, Vector2.ZERO, s, grass, rock, light, sd, castle, shade, tall), n[0], 0, n[6], n[7], n[8])
		# Banks of cloud along the bottom, drifting slowly, and a few at the sides.
		var cream: Color = pal.cream
		var shade_c: Color = pal.shade
		var count := int(9 * detail) + 3
		for k in count:
			var u := float(k) / (count - 1)
			var y := h * (0.8 + 0.12 * Me.noise(seed + 40 + k))
			var s := w * (0.06 + 0.03 * absf(Me.noise(seed + 60 + k)))
			var sd := seed + k
			_add(func(c): Me.cloud(c, Vector2.ZERO, s, cream, shade_c, sd), Vector2(0, y), 1, 4.0 + 2.0 * Me.noise(seed + k), u * w * 1.3)
		for k in int(4 * detail):
			var side := -1.0 if k % 2 == 0 else 1.0
			var x := w * 0.02 if side < 0 else w * 0.98
			var s := w * 0.06
			var sd := seed + 90 + k
			_add(func(c): Me.cloud(c, Vector2.ZERO, s, cream, shade_c, sd), Vector2(x, h * (0.5 + 0.1 * k)), 2, 0.2, k, 6.0)
		if foliage:
			_add(func(c): Me.leaves(c, full, -1.0, 3), Vector2.ZERO)
			_add(func(c): Me.leaves(c, full, 1.0, 9), Vector2.ZERO)
		live = Piece.new(_paint_live)
		add_child(live)

	## The small moving things: birds, motes and sparkles, and the sparkles circling the halo.
	func _paint_live(c: CanvasItem) -> void:
		var w := size.x
		var h := size.y
		for k in int(5 * detail):
			var bx := fmod(w * (0.2 + 0.15 * k) + t * 10.0, w)
			var by := h * (0.18 + 0.05 * Me.noise(seed + 200 + k))
			var flap := sin(t * 6.0 + k) * 2.0
			c.draw_polyline(PackedVector2Array([Vector2(bx - 6, by - 2 + flap), Vector2(bx, by + 1), Vector2(bx + 6, by - 2 + flap)]), Color(1, 1, 1, 0.75), 1.6, true)
		for k in int(26 * detail):
			var mx := w * absf(Me.noise(seed + 400 + k))
			var my := fmod(h * absf(Me.noise(seed + 500 + k)) - t * (6.0 + 4.0 * absf(Me.noise(seed + 600 + k))) + h * 4.0, h)
			var a := 0.25 + 0.25 * sin(t * 2.0 + k)
			if k % 5 == 0:
				Me.sparkle(c, Vector2(mx, my), 4.0 + 3.0 * absf(Me.noise(seed + 700 + k)), Color(1, 1, 1, a + 0.2))
			else:
				c.draw_circle(Vector2(mx, my), 1.6, Color(GLOW, a))
		var centre := Vector2(w * 0.66, h * 0.12)
		var r := w * 0.11
		for k in 4:
			var a := t * 0.4 + TAU * k / 4.0
			Me.sparkle(c, centre + Vector2(cos(a) * r * 1.2, sin(a) * r * 0.34), 5.0, Color(1, 1, 1, 0.7))


## A floating island centred at `at`, `size` wide: a grassy top, a jagged rocky underside narrowing to a point, a few lighter strata, a
## thin waterfall, and optionally a castle (`castle`), drawn with `shade` 0..1 of the near islands' contrast (0 for a pale far one).
static func island(c: CanvasItem, at: Vector2, size: float, top: Color, rock: Color, rock_light: Color, seed: int, castle := false, shade := 1.0,
		tall := false) -> void:
	var hw := size * 0.5
	var under := PackedVector2Array([at + Vector2(-hw, 0)])
	var n := 7
	for k in range(1, n):
		var u := float(k) / n
		var depth := size * (0.55 + 0.25 * absf(noise(seed + k))) * sin(u * PI) * (1.3 if tall else 1.0)
		under.append(at + Vector2(-hw + size * u + noise(seed + 20 + k) * size * 0.04, depth))
	under.append(at + Vector2(hw, 0))
	c.draw_colored_polygon(rough(under, seed, size * 0.02, size * 0.08), rock)
	# Lighter strata on the left of the underside.
	var strata := PackedVector2Array([at + Vector2(-hw * 0.9, 2), at + Vector2(-hw * 0.1, 2), at + Vector2(-hw * 0.25, size * 0.35)])
	c.draw_colored_polygon(strata, Color(rock_light, 0.6 * shade + 0.3))
	if shade > 0.5:
		c.draw_line(at + Vector2(hw * 0.35, size * 0.05), at + Vector2(hw * 0.35, size * 0.9), Color(1, 1, 1, 0.35), maxf(1.5, size * 0.025), true)
	# The grassy top, a rounded cap with a lighter rim.
	var cap := PackedVector2Array()
	for k in 13:
		var a := PI + PI * k / 12.0
		cap.append(at + Vector2(cos(a) * hw * 1.05, sin(a) * size * 0.07 - size * 0.02))
	cap.append(at + Vector2(hw * 1.02, size * 0.04))
	cap.append(at + Vector2(-hw * 1.02, size * 0.04))
	c.draw_colored_polygon(cap, top)
	c.draw_line(at + Vector2(-hw * 0.8, -size * 0.07), at + Vector2(hw * 0.6, -size * 0.08), Color(top.lightened(0.3), 0.8), maxf(1.0, size * 0.02), true)
	if castle:
		_castle(c, at + Vector2(-size * 0.05, -size * 0.05), size * (0.5 if tall else 0.35), Color(rock_light.lerp(Color.WHITE, 0.25), 1.0), Color(rock.darkened(0.1), 1.0))
	elif shade > 0.5:
		# A couple of round trees.
		for k in 2:
			var tx := at.x + size * (-0.25 + 0.4 * k)
			c.draw_circle(Vector2(tx, at.y - size * 0.13), size * 0.08, top.darkened(0.15))
			c.draw_circle(Vector2(tx - size * 0.02, at.y - size * 0.15), size * 0.05, top.lightened(0.12))


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


## A cloud of round lobes at `at`, `size` across: a lavender shadow below, the cream body, and a bright top.
static func cloud(c: CanvasItem, at: Vector2, size: float, cream: Color, shade: Color, seed: int) -> void:
	var lobes := 6
	for pass_i in 3:
		var col: Color = [shade, cream, cream.lerp(Color.WHITE, 0.55)][pass_i]
		var lift: float = [0.0, -size * 0.06, -size * 0.13][pass_i]
		var shrink: float = [1.0, 0.92, 0.55][pass_i]
		for k in lobes:
			var u := float(k) / (lobes - 1) - 0.5
			var r := size * (0.22 + 0.12 * (1.0 - absf(u) * 2.0) + 0.04 * noise(seed + k)) * shrink
			var p := at + Vector2(u * size * 1.4, -absf(noise(seed + 30 + k)) * size * 0.12 + lift)
			c.draw_circle(p, r, col)


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
