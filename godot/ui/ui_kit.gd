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
	## A slate gradient with soft diagonal beams of light, like a stage spotlight; or, in `grid` mode, a pastel graph-paper
	## sheet with a diagonal sheen.
	var grid := false
	var t := 0.0

	func _init(use_grid := false) -> void:
		grid = use_grid
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _process(delta: float) -> void:
		t += delta
		queue_redraw()

	func _draw() -> void:
		var w := size.x
		var h := size.y
		if grid:
			draw_rect(Rect2(0, 0, w, h), Color(0.97, 0.78, 0.82))
			var step := 38.0
			var x := fmod(t * 6.0, step)
			while x < w:
				draw_line(Vector2(x, 0), Vector2(x, h), Color(1, 1, 1, 0.35), 1.5)
				x += step
			var y := fmod(t * 6.0, step)
			while y < h:
				draw_line(Vector2(0, y), Vector2(w, y), Color(1, 1, 1, 0.35), 1.5)
				y += step
			draw_colored_polygon(PackedVector2Array([Vector2(w * 0.45, 0), Vector2(w * 0.75, 0), Vector2(w * 0.5, h), Vector2(w * 0.2, h)]), Color(1, 1, 1, 0.12))
			draw_colored_polygon(PackedVector2Array([Vector2(0, 0), Vector2(w * 0.28, 0), Vector2(w * 0.05, h), Vector2(0, h)]), Color(0.1, 0.1, 0.15, 0.12))
		else:
			var steps := 24
			for i in steps:
				var k := float(i) / steps
				draw_rect(Rect2(0, h * k, w, h / steps + 1.0), D.SLATE_TOP.lerp(D.SLATE_BOTTOM, k))
			var drift := sin(t * 0.35) * 40.0
			draw_colored_polygon(PackedVector2Array([Vector2(w * 0.5 + drift, 0), Vector2(w * 0.62 + drift, 0), Vector2(w * 0.38 + drift, h), Vector2(w * 0.24 + drift, h)]), Color(1, 1, 0.8, 0.09))
			draw_colored_polygon(PackedVector2Array([Vector2(w * 0.66 - drift, 0), Vector2(w * 0.7 - drift, 0), Vector2(w * 0.5 - drift, h), Vector2(w * 0.46 - drift, h)]), Color(1, 1, 0.8, 0.06))


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
		D.draw_slant(self, rect, shown, Color(0.11, 0.12, 0.17), 4.0, true)
		if selected:
			var inner := rect.grow(-6.0)
			var pts := D.skew_points(inner)
			pts.append(pts[0])
			draw_polyline(pts, Color(1, 1, 1, 0.9), 2.0, true)
		var skew_pad := rect.size.y * D.SKEW
		D.draw_text_centered(self, text, Rect2(rect.position.x + skew_pad * 0.5, rect.position.y, rect.size.x - skew_pad, rect.size.y), font_size, ink if not disabled else Color(0.35, 0.35, 0.35))



# ---- Round arrow button ------------------------------------------------------------------------------------------

class Arrow extends Control:
	signal pressed
	var direction := 1
	var hover := false
	var fill := Color(0.96, 0.95, 0.88)

	func _init(dir := 1) -> void:
		direction = dir
		custom_minimum_size = Vector2(44, 44)
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
		tag = Tag.new(label, Vector2(150, 44))
		tag.fill = D.CREAM_DARK
		tag.font_size = 22
		add_child(tag)
		var left := Arrow.new(-1)
		left.pressed.connect(func(): step(-1))
		add_child(left)
		value_tag = Tag.new("", Vector2(210, 44))
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
	var value := 5
	var tag: Control
	var pips: Control
	var focus_mark := false

	class Pips extends Control:
		var value := 5
		func _init() -> void:
			custom_minimum_size = Vector2(250, 44)
			mouse_filter = Control.MOUSE_FILTER_IGNORE
		func _draw() -> void:
			for i in 9:
				var x := 6.0 + i * 27.0
				var on := i < value
				var rect := Rect2(x, 8.0, 22.0, 28.0)
				D.draw_slant(self, rect, D.GOLD if on else Color(0.2, 0.24, 0.3), Color(0.11, 0.12, 0.17), 2.0, false)

	func _init(label: String, start := 5) -> void:
		add_theme_constant_override("separation", 6)
		tag = Tag.new(label, Vector2(150, 44))
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
