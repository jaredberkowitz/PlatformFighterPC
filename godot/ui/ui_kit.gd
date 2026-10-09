extends RefCounted
## Menu pieces: slanted tags and buttons, round arrows, selector rows, stat rows, portraits and the backdrop.
## Colours and drawing helpers live in ui_draw.gd; the same names are repeated here for the screens.

const D := preload("res://ui/ui_draw.gd")
const INK := D.INK
const CREAM_DARK := D.CREAM_DARK
const CREAM := D.CREAM
const SKY := D.SKY
const GOLD := D.GOLD
const TEAL := D.TEAL
const SLATE_TOP := D.SLATE_TOP
const SLATE_BOTTOM := D.SLATE_BOTTOM
const RED := D.RED
const BLUE := D.BLUE
const SKEW := D.SKEW


static func font() -> Font:
	return D.font()


# ---- Backdrop -------------------------------------------------------------------------------------------------

class Backdrop extends Control:
	## A deep indigo-to-navy gradient with slow sweeping light rays, faint diagonal stripes and drifting motes. `use_grid` gives a
	## warmer variant (the character select).
	var warm := false
	var t := 0.0
	var motes: Array = []

	func _init(use_grid := false) -> void:
		warm = use_grid
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		var rng := RandomNumberGenerator.new()
		rng.seed = 5
		for k in 36:
			motes.append([rng.randf(), rng.randf(), rng.randf_range(1.5, 4.0), rng.randf_range(0.01, 0.04), rng.randf() * TAU])

	func _process(delta: float) -> void:
		t += delta
		queue_redraw()

	func _draw() -> void:
		var w := size.x
		var h := size.y
		var top := Color(0.2, 0.12, 0.36) if not warm else Color(0.46, 0.14, 0.3)
		var bottom := Color(0.04, 0.06, 0.15) if not warm else Color(0.1, 0.06, 0.2)
		draw_polygon(PackedVector2Array([Vector2.ZERO, Vector2(w, 0), Vector2(w, h), Vector2(0, h)]),
			PackedColorArray([top, top.lerp(bottom, 0.3), bottom, bottom.lerp(top, 0.25)]))
		# Light rays fanning from the top right, turning slowly.
		var origin := Vector2(w * 0.82, -h * 0.25)
		for k in 9:
			var a := 1.75 + k * 0.17 + sin(t * 0.15 + k) * 0.03 + t * 0.01
			var width := 0.05
			var far := h * 2.2
			var p1 := origin + Vector2(cos(a - width), sin(a - width)) * far
			var p2 := origin + Vector2(cos(a + width), sin(a + width)) * far
			draw_colored_polygon(PackedVector2Array([origin, p1, p2]), Color(1, 0.95, 0.85, 0.035 if k % 2 == 0 else 0.02))
		# Faint diagonal stripes, scrolling.
		var step := 46.0
		var off := fmod(t * 12.0, step)
		var x := -h + off
		while x < w:
			draw_line(Vector2(x, h), Vector2(x + h * 0.6, 0), Color(1, 1, 1, 0.025), 14.0)
			x += step
		# Drifting motes.
		for m in motes:
			var y: float = fmod(m[1] - t * m[3] + 10.0, 1.0)
			var px: float = m[0] + sin(t * 0.6 + m[4]) * 0.01
			draw_circle(Vector2(px * w, y * h), m[2], Color(1, 0.95, 0.8, 0.12 + 0.08 * sin(t * 2.0 + m[4])))


## A wipe that opens a new screen: two slanted bands slide away. `UI.reveal(screen)` from a screen's `_ready`.
class Reveal extends CanvasLayer:
	var t := 0.0
	var board: Control

	func _init() -> void:
		layer = 100
		board = Control.new()
		board.set_anchors_preset(Control.PRESET_FULL_RECT)
		board.mouse_filter = Control.MOUSE_FILTER_IGNORE
		board.draw.connect(_paint)
		add_child(board)

	func _process(delta: float) -> void:
		t += delta
		board.queue_redraw()
		if t > 0.45:
			queue_free()

	func _paint() -> void:
		var w := board.size.x
		var h := board.size.y
		var k := clampf(t / 0.38, 0.0, 1.0)
		var ease := 1.0 - pow(1.0 - k, 3.0)
		var lean := h * 0.35
		var x1 := lerpf(-lean, w + lean, ease)
		board.draw_colored_polygon(PackedVector2Array([Vector2(x1, 0), Vector2(w + lean, 0), Vector2(w + lean, h), Vector2(x1 - lean, h)]), Color(0.06, 0.07, 0.14))
		var x2 := lerpf(-lean - 40.0, w + lean, clampf(ease * 1.12, 0.0, 1.0))
		board.draw_colored_polygon(PackedVector2Array([Vector2(x2, 0), Vector2(x2 + 26.0, 0), Vector2(x2 + 26.0 - lean, h), Vector2(x2 - lean, h)]), D.GOLD)


static func reveal(screen: Node) -> void:
	screen.add_child(Reveal.new())


# ---- Slanted label ----------------------------------------------------------------------------------------------

class Tag extends Control:
	## A slanted label: text on a coloured parallelogram.
	var text := ""
	var fill := Color(0.96, 0.95, 0.88)
	var ink := Color(0.11, 0.12, 0.17)
	var edge := Color(0, 0, 0, 0)
	var font_size := 24
	var shadow := true
	var highlight := false

	func _init(label := "", min_size := Vector2(120, 40)) -> void:
		text = label
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func set_text(t: String) -> void:
		text = t
		queue_redraw()

	func _draw() -> void:
		var rect := Rect2(Vector2.ZERO, size).grow(-4)
		var skew_pad := rect.size.y * D.SKEW
		D.draw_slant(self, rect, fill, D.GOLD if highlight else edge, 4.0 if highlight else 3.0, shadow)
		D.draw_text_centered(self, text, Rect2(rect.position.x + skew_pad * 0.5, rect.position.y, rect.size.x - skew_pad, rect.size.y), font_size, ink)




# ---- Slanted button ---------------------------------------------------------------------------------------------

class Btn extends Control:
	## A big slanted button. It bounces a little when hovered or selected with the keyboard, and emits `activated`.
	signal activated
	var text := ""
	var fill := Color(0.96, 0.95, 0.88)
	var ink := Color(0.11, 0.12, 0.17)
	var accent := Color(1.0, 0.82, 0.22)
	var font_size := 34
	var selected := false
	var disabled := false
	var hover := false
	var pop := 0.0

	func _init(label := "", min_size := Vector2(320, 64)) -> void:
		text = label
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func():
			hover = true)
		mouse_exited.connect(func():
			hover = false)

	func _process(delta: float) -> void:
		var target := 1.0 if (selected or hover) and not disabled else 0.0
		pop = move_toward(pop, target, delta * 7.0)
		queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if disabled:
			return
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()

	func _draw() -> void:
		var grow := pop * 6.0
		var rect := Rect2(Vector2.ZERO, size).grow(-8.0 + grow * 0.3)
		rect.position.x += pop * 10.0
		var base := fill if not disabled else Color(0.6, 0.6, 0.6)
		var shown := base.lerp(accent, pop * 0.55)
		if pop > 0.05:
			D.draw_glow(self, rect, accent, pop)
		D.draw_slant(self, rect, shown, Color(0.11, 0.12, 0.17), 4.0, true)
		if selected:
			var inner := rect.grow(-6.0)
			var pts := D.skew_points(inner)
			pts.append(pts[0])
			draw_polyline(pts, Color(1, 1, 1, 0.9), 2.0, true)
			# A shine that sweeps across now and then.
			var sweep := fmod(Time.get_ticks_msec() / 1000.0, 2.4) / 2.4
			var sx := lerpf(rect.position.x - 40.0, rect.end.x + 40.0, sweep)
			var lean := rect.size.y * D.SKEW
			var band := PackedVector2Array([Vector2(sx, rect.position.y + 2), Vector2(sx + 22, rect.position.y + 2), Vector2(sx + 22 - lean, rect.end.y - 2), Vector2(sx - lean, rect.end.y - 2)])
			var clipped := Geometry2D.intersect_polygons(band, D.skew_points(rect.grow(-3.0)))
			for poly in clipped:
				draw_colored_polygon(poly, Color(1, 1, 1, 0.28))
		var skew_pad := rect.size.y * D.SKEW
		# Long labels shrink to fit the button.
		var fs := font_size
		while fs > 12 and D.font().get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > rect.size.x - skew_pad - 24.0:
			fs -= 2
		D.draw_text_centered(self, text, Rect2(rect.position.x + skew_pad * 0.5, rect.position.y, rect.size.x - skew_pad, rect.size.y), fs, ink if not disabled else Color(0.35, 0.35, 0.35))



# ---- Menu tile ---------------------------------------------------------------------------------------------------------

class Tile extends Control:
	## A big menu tile: a slanted panel in its own colour with a large line-drawn icon, a title and a subtitle. Selected, it lifts,
	## glows and brightens. Emits `activated`.
	signal activated
	var title := ""
	var subtitle := ""
	var icon := ""
	var color := Color(0.9, 0.3, 0.3)
	var selected := false
	var hover := false
	var pop := 0.0
	const TILE_SKEW := 0.08

	func _init(label: String, sub: String, tint: Color, glyph: String) -> void:
		title = label
		subtitle = sub
		color = tint
		icon = glyph
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func(): hover = true)
		mouse_exited.connect(func(): hover = false)

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()

	func _process(delta: float) -> void:
		pop = move_toward(pop, 1.0 if (selected or hover) else 0.0, delta * 6.0)
		queue_redraw()

	func _draw() -> void:
		var rect := Rect2(Vector2.ZERO, size).grow(-6.0 + pop * 4.0)
		rect.position.y -= pop * 4.0
		if pop > 0.02:
			D.draw_glow(self, rect, Color(1, 0.92, 0.6), pop, TILE_SKEW)
		var fill := color.lerp(color.lightened(0.12), pop)
		D.draw_slant(self, rect, fill, D.INK, 5.0, true, TILE_SKEW)
		# A band of darker diagonal stripes across the lower corner, for texture.
		var lean := rect.size.y * TILE_SKEW
		var stripes := PackedVector2Array([rect.position + Vector2(rect.size.x * 0.55, rect.size.y), rect.end, rect.end - Vector2(0, rect.size.y * 0.55)])
		draw_colored_polygon(stripes, Color(0, 0, 0, 0.12))
		# The icon, big and pale, toward the right.
		var box := minf(rect.size.x, rect.size.y) * 0.55
		var ic := Vector2(rect.end.x - box * 0.62 - lean * 0.3, rect.position.y + rect.size.y * 0.42)
		if icon != "":
			_icon(ic, box * 0.5, Color(1, 1, 1, 0.82 + 0.18 * pop))
		# Title and subtitle at the bottom left, shrunk to fit the tile.
		var f := D.font()
		var big := clampi(int(rect.size.y * 0.2), 26, 64)
		var room := rect.size.x - lean - 40.0
		while big > 18 and f.get_string_size(title, HORIZONTAL_ALIGNMENT_LEFT, -1, big).x > room:
			big -= 2
		draw_string(f, rect.position + Vector2(lean + 22.0, rect.size.y - 22.0 - big * 0.55) + Vector2(3, 3), title, HORIZONTAL_ALIGNMENT_LEFT, -1, big, Color(0, 0, 0, 0.35))
		draw_string(f, rect.position + Vector2(lean + 22.0, rect.size.y - 22.0 - big * 0.55), title, HORIZONTAL_ALIGNMENT_LEFT, -1, big, Color(1, 1, 1))
		if subtitle != "" and f.get_string_size(subtitle, HORIZONTAL_ALIGNMENT_LEFT, -1, 18).x <= room:
			draw_string(f, rect.position + Vector2(lean + 24.0, rect.size.y - 18.0), subtitle, HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(1, 1, 1, 0.8))

	## Simple line icons drawn in code (original, no fonts or images).
	func _icon(c: Vector2, r: float, col: Color) -> void:
		var wdt := maxf(3.0, r * 0.09)
		match icon:
			"play":
				# Two round fighters squaring up, with a spark between them.
				draw_arc(c + Vector2(-r * 0.45, r * 0.15), r * 0.38, 0, TAU, 40, col, wdt, true)
				draw_arc(c + Vector2(r * 0.45, r * 0.15), r * 0.38, 0, TAU, 40, col, wdt, true)
				var s := c + Vector2(0, -r * 0.45)
				for k in 8:
					var a := TAU * k / 8.0
					draw_line(s + Vector2(cos(a), sin(a)) * r * 0.1, s + Vector2(cos(a), sin(a)) * r * (0.3 if k % 2 == 0 else 0.2), col, wdt * 0.8, true)
			"online":
				# A globe: the outline, the equator and two lines of latitude, and a meridian ellipse.
				draw_arc(c, r * 0.8, 0, TAU, 48, col, wdt, true)
				draw_line(c + Vector2(-r * 0.8, 0), c + Vector2(r * 0.8, 0), col, wdt * 0.8, true)
				for y in [-0.42, 0.42]:
					var half := sqrt(0.64 - y * y) * r
					draw_line(c + Vector2(-half, y * r), c + Vector2(half, y * r), col, wdt * 0.6, true)
				var ell := PackedVector2Array()
				for k in 33:
					var a := TAU * k / 32.0
					ell.append(c + Vector2(cos(a) * r * 0.36, sin(a) * r * 0.8))
				draw_polyline(ell, col, wdt * 0.8, true)
			"replays":
				var rr := Rect2(c - Vector2(r * 0.85, r * 0.55), Vector2(r * 1.7, r * 1.1))
				draw_rect(rr, col, false, wdt)
				for k in 5:
					draw_rect(Rect2(rr.position + Vector2(r * 0.12 + k * r * 0.32, r * 0.06), Vector2(r * 0.14, r * 0.12)), col)
					draw_rect(Rect2(rr.position + Vector2(r * 0.12 + k * r * 0.32, rr.size.y - r * 0.18), Vector2(r * 0.14, r * 0.12)), col)
				draw_colored_polygon(PackedVector2Array([c + Vector2(-r * 0.2, -r * 0.25), c + Vector2(r * 0.3, 0), c + Vector2(-r * 0.2, r * 0.25)]), col)
			"creator":
				# A pencil over a star.
				var pts := PackedVector2Array()
				for k in 10:
					var a := -PI / 2 + TAU * k / 10.0
					pts.append(c + Vector2(cos(a), sin(a)) * r * (0.75 if k % 2 == 0 else 0.32))
				pts.append(pts[0])
				draw_polyline(pts, col, wdt, true)
			"controls":
				var body := Rect2(c - Vector2(r * 0.85, r * 0.42), Vector2(r * 1.7, r * 0.84))
				draw_rect(body, col, false, wdt)
				draw_line(c + Vector2(-r * 0.55, 0), c + Vector2(-r * 0.25, 0), col, wdt, true)
				draw_line(c + Vector2(-r * 0.4, -r * 0.15), c + Vector2(-r * 0.4, r * 0.15), col, wdt, true)
				draw_circle(c + Vector2(r * 0.35, -r * 0.1), r * 0.09, col)
				draw_circle(c + Vector2(r * 0.55, r * 0.1), r * 0.09, col)
			"editors":
				# A wrench.
				draw_line(c + Vector2(-r * 0.55, r * 0.55), c + Vector2(r * 0.2, -r * 0.2), col, wdt * 1.6, true)
				draw_arc(c + Vector2(r * 0.35, -r * 0.35), r * 0.3, PI * 0.9, PI * 2.6, 24, col, wdt * 1.4, true)
			"quit":
				var door := Rect2(c - Vector2(r * 0.5, r * 0.7), Vector2(r * 0.7, r * 1.4))
				draw_rect(door, col, false, wdt)
				draw_line(c + Vector2(0, 0), c + Vector2(r * 0.8, 0), col, wdt, true)
				draw_colored_polygon(PackedVector2Array([c + Vector2(r * 0.8, -r * 0.2), c + Vector2(r * 1.05, 0), c + Vector2(r * 0.8, r * 0.2)]), col)


# ---- Round arrow button ------------------------------------------------------------------------------------------

class Arrow extends Control:
	signal pressed
	var direction := 1
	var hover := false
	var fill := Color(0.96, 0.95, 0.88)

	func _init(dir := 1) -> void:
		direction = dir
		custom_minimum_size = Vector2(38, 38)
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func():
			hover = true
			queue_redraw())
		mouse_exited.connect(func():
			hover = false
			queue_redraw())

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			pressed.emit()

	func _draw() -> void:
		var c := size * 0.5
		var r := minf(size.x, size.y) * 0.5 - 3.0
		draw_circle(c + Vector2(2, 3), r, Color(0, 0, 0, 0.3))
		draw_circle(c, r, Color(1.0, 0.85, 0.3) if hover else fill)
		draw_arc(c, r, 0, TAU, 32, Color(0.11, 0.12, 0.17), 3.0, true)
		var d := float(direction)
		var tri := PackedVector2Array([c + Vector2(-5 * d, -9), c + Vector2(7 * d, 0), c + Vector2(-5 * d, 9)])
		draw_colored_polygon(tri, Color(0.82, 0.45, 0.2))
		tri.append(tri[0])
		draw_polyline(tri, Color(0.11, 0.12, 0.17), 2.0, true)


# ---- Selector row: [label] <  value  > ----------------------------------------------------------------------------

class Selector extends HBoxContainer:
	## A label tab, a left arrow, a value tab and a right arrow. `changed(index)` fires on every change.
	signal changed(index: int)
	signal focused
	var options: Array = []
	var index := 0
	var tag: Control
	var value_tag: Control
	var swatches := false
	var focus_mark := false

	func _init(label: String, choices: Array) -> void:
		options = choices
		add_theme_constant_override("separation", 6)
		tag = Tag.new(label, Vector2(150, 38))
		tag.fill = D.CREAM_DARK
		tag.font_size = 22
		add_child(tag)
		var left := Arrow.new(-1)
		left.pressed.connect(func(): step(-1))
		add_child(left)
		value_tag = Tag.new("", Vector2(210, 38))
		value_tag.font_size = 24
		add_child(value_tag)
		var right := Arrow.new(1)
		right.pressed.connect(func(): step(1))
		add_child(right)
		set_index(0, false)

	func step(delta: int) -> void:
		set_index((index + delta + options.size()) % options.size())

	func set_index(i: int, emit := true) -> void:
		index = clampi(i, 0, options.size() - 1)
		value_tag.set_text(str(options[index]).capitalize() if options[index] is String else str(options[index]))
		if emit:
			changed.emit(index)

	func set_focus_mark(on: bool) -> void:
		focus_mark = on
		tag.highlight = on
		tag.fill = D.GOLD.lerp(D.CREAM, 0.35) if on else D.CREAM_DARK
		tag.queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			focused.emit()




# ---- Stat row: label, pips, arrows ----------------------------------------------------------------------------------

class StatRow extends HBoxContainer:
	signal changed(value: int)
	signal focused
	## Emitted when a rise was refused because `can_raise` said there is nothing to spend.
	signal blocked
	var can_raise: Callable
	var value := 5
	var tag: Control
	var pips: Control
	var focus_mark := false

	class Pips extends Control:
		var value := 5
		func _init() -> void:
			custom_minimum_size = Vector2(250, 38)
			mouse_filter = Control.MOUSE_FILTER_IGNORE
		func _draw() -> void:
			for i in 9:
				var x := 6.0 + i * 27.0
				var on := i < value
				var rect := Rect2(x, 5.0, 22.0, 28.0)
				D.draw_slant(self, rect, D.GOLD if on else Color(0.2, 0.24, 0.3), Color(0.11, 0.12, 0.17), 2.0, false)

	func _init(label: String, start := 5) -> void:
		add_theme_constant_override("separation", 6)
		tag = Tag.new(label, Vector2(150, 38))
		tag.fill = D.CREAM_DARK
		tag.font_size = 22
		add_child(tag)
		var left := Arrow.new(-1)
		left.pressed.connect(func(): step(-1))
		add_child(left)
		pips = Pips.new()
		add_child(pips)
		var right := Arrow.new(1)
		right.pressed.connect(func(): step(1))
		add_child(right)
		set_value(start, false)

	func step(delta: int) -> void:
		if delta > 0 and value < 9 and can_raise.is_valid() and not can_raise.call():
			blocked.emit()
			return
		set_value(clampi(value + delta, 1, 9))

	func set_value(v: int, emit := true) -> void:
		value = clampi(v, 1, 9)
		pips.value = value
		pips.queue_redraw()
		if emit:
			changed.emit(value)

	func set_focus_mark(on: bool) -> void:
		focus_mark = on
		tag.highlight = on
		tag.fill = D.GOLD.lerp(D.CREAM, 0.35) if on else D.CREAM_DARK
		tag.queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			focused.emit()


# ---- Round portrait -----------------------------------------------------------------------------------------------------

class Portrait extends Control:
	## A round token showing a fighter's colour, face and hat colour: the character-select grid and the saved list use it.
	var body := Color(1.0, 0.85, 0.2)
	var accent := Color(0.25, 0.45, 0.92)
	var hat := 0
	var glasses := 0
	var face := 0
	var label := ""
	var ring := Color(0.11, 0.12, 0.17)
	var ring_width := 4.0
	var hover := false
	var glyph := ""      # "?" for random, "+" for new
	var scale_hint := 1.0
	signal clicked(button: int)

	func _init(diameter := 76.0) -> void:
		custom_minimum_size = Vector2(diameter, diameter)
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func():
			hover = true
			queue_redraw())
		mouse_exited.connect(func():
			hover = false
			queue_redraw())

	func set_look(l: RefCounted) -> void:
		body = l.body_color()
		accent = l.accent_color()
		hat = l.hat
		glasses = l.glasses
		face = l.face
		queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			clicked.emit(event.button_index)

	func _draw() -> void:
		var c := size * 0.5
		var r := minf(size.x, size.y) * 0.5 - ring_width
		draw_circle(c + Vector2(2, 4), r + 2, Color(0, 0, 0, 0.3))
		draw_circle(c, r, Color(1, 0.96, 0.86) if glyph == "" else Color(0.95, 0.9, 0.7))
		if glyph != "":
			D.draw_text_centered(self, glyph, Rect2(Vector2.ZERO, size), int(r * 1.3), Color(0.82, 0.45, 0.2))
		else:
			# Body blob, hat, eyes and mouth: a tiny version of the 3D fighter.
			var s := r / 38.0
			draw_circle(c + Vector2(0, 6 * s), 27 * s * scale_hint, body)
			draw_arc(c + Vector2(0, 6 * s), 27 * s * scale_hint, 0, TAU, 28, Color(0.11, 0.12, 0.17), 2.0, true)
			for sx in [-1.0, 1.0]:
				draw_circle(c + Vector2(sx * 10 * s, 4 * s), 5 * s, Color(1, 1, 1))
				draw_circle(c + Vector2(sx * 10 * s + sx * -0.5, 5 * s), 2.2 * s, Color(0.11, 0.12, 0.17))
			draw_line(c + Vector2(-5 * s, 16 * s), c + Vector2(5 * s, 16 * s), Color(0.11, 0.12, 0.17), 2.0)
			if hat != 0:
				var col := accent if hat in [1, 3, 4] else Color(0.85, 0.7, 0.3)
				draw_rect(Rect2(c + Vector2(-17 * s, -26 * s), Vector2(34 * s, 11 * s)), col)
				draw_rect(Rect2(c + Vector2(-17 * s, -26 * s), Vector2(34 * s, 11 * s)), Color(0.11, 0.12, 0.17), false, 2.0)
			if glasses != 0:
				draw_line(c + Vector2(-17 * s, 3 * s), c + Vector2(17 * s, 3 * s), Color(0.11, 0.12, 0.17), 3.0)
		draw_arc(c, r, 0, TAU, 40, ring if not hover else D.GOLD, ring_width, true)
		if label != "":
			draw_string(D.font(), Vector2(4, size.y - 2), label, HORIZONTAL_ALIGNMENT_LEFT, size.x - 8, 14, Color(1, 1, 1))


# ---- Speech bubble: a slanted panel with wrapped text ---------------------------------------------------------------------

class Bubble extends Control:
	var label: Label
	var fill := Color(0.96, 0.95, 0.88)

	func _init(min_size := Vector2(380, 150)) -> void:
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		label = Label.new()
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		label.add_theme_font_override("font", D.font())
		label.add_theme_font_size_override("font_size", 24)
		label.add_theme_color_override("font_color", D.INK)
		label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		add_child(label)

	func set_text(t: String) -> void:
		label.text = t
		_layout()

	func _layout() -> void:
		var rect := Rect2(Vector2.ZERO, size).grow(-4)
		var pad := rect.size.y * D.SKEW
		label.position = Vector2(rect.position.x + pad + 14.0, rect.position.y + 8.0)
		label.size = Vector2(maxf(rect.size.x - pad * 1.4 - 28.0, 10.0), maxf(rect.size.y - 16.0, 10.0))

	func _notification(what: int) -> void:
		if what == NOTIFICATION_RESIZED:
			_layout()

	func _draw() -> void:
		_layout()
		D.draw_slant(self, Rect2(Vector2.ZERO, size).grow(-4), fill, D.INK, 3.0, true)


# ---- Stat bars: how a character plays -------------------------------------------------------------------------------------------

class Bars extends Control:
	var values := {}
	var order: Array = []

	func _init(min_size := Vector2(380, 200)) -> void:
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func set_values(v: Dictionary, keys: Array) -> void:
		values = v
		order = keys
		queue_redraw()

	func _draw() -> void:
		var f := D.font()
		var row := size.y / maxf(order.size(), 1)
		for i in order.size():
			var y := i * row
			draw_string(f, Vector2(4, y + row * 0.68), order[i], HORIZONTAL_ALIGNMENT_LEFT, 150, 22, Color(1, 1, 1, 0.92))
			var track := Rect2(160, y + row * 0.2, size.x - 170, row * 0.55)
			D.draw_slant(self, track, Color(0.16, 0.19, 0.25), D.INK, 2.0, false)
			var amount: float = clampf(float(values.get(order[i], 0.0)), 0.0, 1.0)
			if amount > 0.02:
				var fill := Rect2(track.position, Vector2(track.size.x * amount, track.size.y)).grow(-2)
				D.draw_slant(self, fill, D.GOLD, Color(0, 0, 0, 0), 0.0, false)
