extends RefCounted
## Menu pieces: slanted tags and buttons, round arrows, selector rows, stat rows, portraits and the backdrop.
## Colours and drawing helpers live in ui_draw.gd; the same names are repeated here for the screens.

const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
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
	## The menus' sky (menu_art.gd): a sunny fantasy sky with floating islands and castles, halo rings, clouds, birds and drifting motes,
	## leaves framing the bottom corners. It moves slowly (the islands bob, the clouds drift), and never takes the mouse.
	func _init(_warm := false) -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		var sky := Art.SkyScene.new()
		sky.foliage = true
		sky.set_anchors_preset(Control.PRESET_FULL_RECT)
		add_child(sky)


# ---- Scenery ---------------------------------------------------------------------------------------------------

class Scene extends Control:
	## A painted sky (menu_art.gd) with a grassy ledge across the lower middle for a fighter to stand on: the scene behind the fighters in
	## the menus (the Play tile, the players' panels), warm or cool by `mood`. Put it in a `Mask` to give it a shape.
	var mood := 0.0:
		set(v):
			mood = v
			if sky != null:
				sky.mood = v
	var seed := 3:
		set(v):
			seed = v
			if sky != null:
				sky.seed = v
	var sky: Control
	var ledge: Control

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		sky = Art.SkyScene.new()
		sky.detail = 0.6
		add_child(sky)
		ledge = Art.Piece.new(_paint_ledge)
		add_child(ledge)

	func _notification(what: int) -> void:
		if what == NOTIFICATION_RESIZED:
			sky.size = size
			ledge.size = size
			ledge.queue_redraw()

	func _paint_ledge(c: CanvasItem) -> void:
		# A grassy top with a rocky face, where the fighter stands.
		var w := size.x
		var h := size.y
		var face := PackedVector2Array([Vector2(w * 0.12, h * 0.76), Vector2(w * 0.3, h * 0.7), Vector2(w * 0.72, h * 0.7), Vector2(w * 0.9, h * 0.76),
			Vector2(w * 0.86, h * 1.05), Vector2(w * 0.16, h * 1.05)])
		c.draw_colored_polygon(Art.rough(face, seed, 3.0, 24.0), Color("6a5a7e").lerp(Color("8a5a4e"), clampf(mood, 0.0, 1.0)))
		var grass := PackedVector2Array([Vector2(w * 0.11, h * 0.76), Vector2(w * 0.3, h * 0.69), Vector2(w * 0.72, h * 0.69), Vector2(w * 0.91, h * 0.76),
			Vector2(w * 0.88, h * 0.79), Vector2(w * 0.14, h * 0.79)])
		c.draw_colored_polygon(Art.rough(grass, seed + 1, 2.5, 18.0), Color("5cc46a"))
		for k in 14:
			var x := w * (0.16 + 0.68 * k / 13.0)
			var y := h * (0.72 if k > 2 and k < 11 else 0.76)
			c.draw_line(Vector2(x, y), Vector2(x + Art.noise(seed + k) * 5.0, y - 9.0 - 5.0 * absf(Art.noise(seed + 30 + k))), Color("8be07a"), 2.5, true)


class Mask extends Control:
	## A shape used only as a mask: its children (a `Scene`, say) show inside `shape` (in its own coordinates) and nowhere else.
	var shape := PackedVector2Array()

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		clip_children = CanvasItem.CLIP_CHILDREN_ONLY

	func set_shape(points: PackedVector2Array) -> void:
		shape = points
		queue_redraw()

	func _draw() -> void:
		if shape.size() >= 3:
			draw_colored_polygon(shape, Color.WHITE)


# ---- Key glyphs --------------------------------------------------------------------------------------------------

class KeyBar extends Control:
	## The controls along the bottom of a screen: a dark translucent ribbon of key glyphs and what they do. `items` is a list of
	## [key, what it does].
	var items: Array = []

	func _init(list: Array) -> void:
		items = list
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var band := PackedVector2Array([Vector2(12, 0), Vector2(size.x, 0), Vector2(size.x - 12, size.y), Vector2(0, size.y)])
		draw_colored_polygon(band, Color(Art.INK, 0.72))
		var f := D.font()
		var widths := []
		var total := 0.0
		for item in items:
			var kw := maxf(28.0, f.get_string_size(item[0], HORIZONTAL_ALIGNMENT_LEFT, -1, 15).x + 12.0)
			var tw := f.get_string_size(item[1], HORIZONTAL_ALIGNMENT_LEFT, -1, 14).x
			widths.append([kw, tw])
			total += kw + 6.0 + tw + 18.0
		var x := (size.x - total) * 0.5
		for k in items.size():
			var kw: float = widths[k][0]
			Art.key_cap(self, Rect2(x, size.y * 0.5 - 12.0, kw, 24.0), items[k][0])
			x += kw + 6.0
			draw_string(f, Vector2(x, size.y * 0.5 + 6.0), items[k][1], HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(1, 1, 1, 0.9))
			x += float(widths[k][1]) + 18.0


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
	## A label: text on a painted card (menu_art.gd), or on a plain slanted strip when its colour is see-through.
	var text := ""
	var seed := 1
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
		seed = absi(label.hash()) % 997

	func set_text(t: String) -> void:
		text = t
		queue_redraw()

	func _draw() -> void:
		var rect := Rect2(Vector2.ZERO, size).grow(-4)
		if fill.a < 0.9:
			var skew_pad := rect.size.y * D.SKEW
			D.draw_slant(self, rect, fill, D.GOLD if highlight else edge, 4.0 if highlight else 3.0, shadow)
			D.draw_text_centered(self, text, Rect2(rect.position.x + skew_pad * 0.5, rect.position.y, rect.size.x - skew_pad, rect.size.y), font_size, ink)
			return
		rect.size -= Vector2(3, 3)
		Art.card(self, rect, fill, seed, minf(10.0, rect.size.y * 0.3), 3.0, D.GOLD if highlight else Color(0, 0, 0, 0))
		D.draw_text_centered(self, text, rect, font_size, ink)




# ---- Slanted button ---------------------------------------------------------------------------------------------

class Btn extends Control:
	## A big button on a painted card. It lifts a little when hovered or selected with the keyboard, and emits `activated`. It repaints
	## only when something changes (and while selected, for its shine).
	signal activated
	var text := "":
		set(v):
			text = v
			queue_redraw()
	var fill := Color(0.96, 0.95, 0.88):
		set(v):
			fill = v
			queue_redraw()
	var ink := Color(0.11, 0.12, 0.17):
		set(v):
			ink = v
			queue_redraw()
	var accent := Color(1.0, 0.82, 0.22)
	var font_size := 34:
		set(v):
			font_size = v
			queue_redraw()
	var selected := false:
		set(v):
			selected = v
			queue_redraw()
	var disabled := false:
		set(v):
			disabled = v
			queue_redraw()
	var hover := false
	var pop := 0.0
	var seed := 1

	func _init(label := "", min_size := Vector2(320, 64)) -> void:
		text = label
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_STOP
		seed = absi(label.hash()) % 997
		mouse_entered.connect(func():
			hover = true)
		mouse_exited.connect(func():
			hover = false)

	func _process(delta: float) -> void:
		var target := 1.0 if (selected or hover) and not disabled else 0.0
		var was := pop
		pop = move_toward(pop, target, delta * 7.0)
		if pop != was or selected:
			queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if disabled:
			return
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()

	func _draw() -> void:
		# A painted card (menu_art.gd) that lifts and warms toward the accent colour when selected, with a shine sweeping across it.
		var rect := Rect2(Vector2.ZERO, size).grow(-6.0)
		rect.size -= Vector2(4, 6)
		var base := fill if not disabled else Color(0.6, 0.6, 0.6)
		var shown := base.lerp(accent, pop * 0.55)
		var lift := pop * 4.0
		var r := Rect2(rect.position - Vector2(0, lift), rect.size)
		if pop > 0.05:
			D.draw_glow(self, r, accent, pop, 0.0)
		var cut := minf(14.0, rect.size.y * 0.3)
		var shape := Art.card(self, rect, shown, seed, cut, 4.0, Color(1, 1, 1, 0.85) if selected else Color(0, 0, 0, 0), lift)
		if selected:
			# A shine that sweeps across now and then.
			var sweep := fmod(Time.get_ticks_msec() / 1000.0, 2.4) / 2.4
			var sx := lerpf(r.position.x - 40.0, r.end.x + 40.0, sweep)
			var lean := r.size.y * D.SKEW
			var band := PackedVector2Array([Vector2(sx, r.position.y + 2), Vector2(sx + 22, r.position.y + 2), Vector2(sx + 22 - lean, r.end.y - 2), Vector2(sx - lean, r.end.y - 2)])
			for poly in Geometry2D.intersect_polygons(band, shape):
				draw_colored_polygon(poly, Color(1, 1, 1, 0.28))
		# Long labels shrink to fit the button.
		var fs := font_size
		while fs > 12 and D.font().get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > r.size.x - 28.0:
			fs -= 2
		D.draw_text_centered(self, text, r, fs, ink if not disabled else Color(0.35, 0.35, 0.35))



# ---- Menu tile ---------------------------------------------------------------------------------------------------------

class Tile extends Control:
	## A big menu tile: a painted card (menu_art.gd) in its own colour with a large line-drawn icon, a title and a subtitle. Selected, it
	## lifts, glows and brightens. `show_scene` paints a scene into it instead (the Play tile). Emits `activated`.
	signal activated
	var title := ""
	var subtitle := ""
	var icon := ""
	var color := Color(0.9, 0.3, 0.3)
	var selected := false:
		set(v):
			selected = v
			queue_redraw()
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
		var was := pop
		pop = move_toward(pop, 1.0 if (selected or hover) else 0.0, delta * 6.0)
		if pop == was and _laid == size:
			return
		_laid = size
		if mask != null:
			var r := _card_rect()
			mask.position = r.position + Vector2(10, 10 - _lift())
			mask.size = r.size - Vector2(20, 20)
			scene.size = mask.size
			var c := CUT - 6.0
			mask.set_shape(PackedVector2Array([Vector2(c, 0), Vector2(mask.size.x, 0), Vector2(mask.size.x, mask.size.y - c),
				Vector2(mask.size.x - c, mask.size.y), Vector2(0, mask.size.y), Vector2(0, c)]))
		for ch in get_children():
			if ch is Top:
				ch.queue_redraw()
		queue_redraw()

	const CUT := 22.0
	var _laid := Vector2(-1, -1)
	## A painted scene inside the card (the Play tile's), made by `show_scene`.
	var mask: Control
	var scene: Control
	var top: Control

	class Top extends Control:
		## The tile's text, drawn above anything placed in the tile (its scene, a fighter).
		var tile: Control

		func _init(t: Control) -> void:
			tile = t
			set_anchors_preset(Control.PRESET_FULL_RECT)
			mouse_filter = Control.MOUSE_FILTER_IGNORE

		func _draw() -> void:
			tile.paint_text(self)

	## Fills the card with a painted scene (sky, islands, a ledge), under whatever is added with `add_layer`.
	func show_scene(mood := 0.0, seed := 5) -> void:
		mask = Mask.new()
		add_child(mask)
		scene = Scene.new()
		scene.mood = mood
		scene.seed = seed
		mask.add_child(scene)
		_ensure_top()

	## Adds a child above the tile's scene but below its text.
	func add_layer(node: Node) -> void:
		add_child(node)
		_ensure_top()

	func _ensure_top() -> void:
		if top == null:
			top = Top.new(self)
			add_child(top)
		move_child(top, -1)

	func _lift() -> float:
		return pop * 6.0

	func _card_rect() -> Rect2:
		return Rect2(Vector2(6, 6), size - Vector2(16, 18))

	func _draw() -> void:
		var r := _card_rect()
		if pop > 0.02:
			D.draw_glow(self, Rect2(r.position - Vector2(0, _lift()), r.size), Color(1, 0.92, 0.6), pop, 0.0)
		var fill := color.lerp(color.lightened(0.12), pop)
		Art.card(self, r, fill, absi(title.hash()) % 997, CUT, 5.0, color.lightened(0.4), _lift())
		if top == null:
			paint_text(self)

	## The icon, the title and the subtitle (on `c`, the tile itself or its top layer).
	func paint_text(c: CanvasItem) -> void:
		var rect := _card_rect()
		rect.position.y -= _lift()
		# The icon, big and pale, toward the right.
		var box := minf(rect.size.x, rect.size.y) * 0.55
		var ic := Vector2(rect.end.x - box * 0.62 - 6.0, rect.position.y + rect.size.y * 0.42)
		if icon != "":
			_icon(c, ic + Vector2(3, 4), box * 0.5, Color(D.INK, 0.35))
			_icon(c, ic, box * 0.5, Color(1, 1, 1, 0.85 + 0.15 * pop))
		# The title (outlined) and the subtitle at the bottom left, shrunk to fit the tile.
		var f := D.font()
		var big := clampi(int(rect.size.y * 0.2), 24, 64)
		var room := rect.size.x - 44.0
		while big > 16 and f.get_string_size(title, HORIZONTAL_ALIGNMENT_LEFT, -1, big).x > room:
			big -= 2
		var has_sub := subtitle != "" and f.get_string_size(subtitle, HORIZONTAL_ALIGNMENT_LEFT, -1, 17).x <= room
		var base := rect.end.y - (34.0 if has_sub else 18.0) - (6.0 if rect.size.y > 200.0 else 0.0)
		Art.title(c, title, Vector2(rect.position.x + 20.0, base), big, Color.WHITE, 0.14)
		if has_sub:
			Art.title(c, subtitle, Vector2(rect.position.x + 22.0, rect.end.y - 14.0), 17, Color(1, 1, 1, 0.92), 0.0)

	## Simple line icons drawn in code (original, no fonts or images).
	func _icon(canvas: CanvasItem, c: Vector2, r: float, col: Color) -> void:
		var wdt := maxf(3.0, r * 0.09)
		match icon:
			"play":
				# Two round fighters squaring up, with a spark between them.
				canvas.draw_arc(c + Vector2(-r * 0.45, r * 0.15), r * 0.38, 0, TAU, 40, col, wdt, true)
				canvas.draw_arc(c + Vector2(r * 0.45, r * 0.15), r * 0.38, 0, TAU, 40, col, wdt, true)
				var s := c + Vector2(0, -r * 0.45)
				for k in 8:
					var a := TAU * k / 8.0
					canvas.draw_line(s + Vector2(cos(a), sin(a)) * r * 0.1, s + Vector2(cos(a), sin(a)) * r * (0.3 if k % 2 == 0 else 0.2), col, wdt * 0.8, true)
			"online":
				# A globe: the outline, the equator and two lines of latitude, and a meridian ellipse.
				canvas.draw_arc(c, r * 0.8, 0, TAU, 48, col, wdt, true)
				canvas.draw_line(c + Vector2(-r * 0.8, 0), c + Vector2(r * 0.8, 0), col, wdt * 0.8, true)
				for y in [-0.42, 0.42]:
					var half := sqrt(0.64 - y * y) * r
					canvas.draw_line(c + Vector2(-half, y * r), c + Vector2(half, y * r), col, wdt * 0.6, true)
				var ell := PackedVector2Array()
				for k in 33:
					var a := TAU * k / 32.0
					ell.append(c + Vector2(cos(a) * r * 0.36, sin(a) * r * 0.8))
				canvas.draw_polyline(ell, col, wdt * 0.8, true)
			"replays":
				var rr := Rect2(c - Vector2(r * 0.85, r * 0.55), Vector2(r * 1.7, r * 1.1))
				canvas.draw_rect(rr, col, false, wdt)
				for k in 5:
					canvas.draw_rect(Rect2(rr.position + Vector2(r * 0.12 + k * r * 0.32, r * 0.06), Vector2(r * 0.14, r * 0.12)), col)
					canvas.draw_rect(Rect2(rr.position + Vector2(r * 0.12 + k * r * 0.32, rr.size.y - r * 0.18), Vector2(r * 0.14, r * 0.12)), col)
				canvas.draw_colored_polygon(PackedVector2Array([c + Vector2(-r * 0.2, -r * 0.25), c + Vector2(r * 0.3, 0), c + Vector2(-r * 0.2, r * 0.25)]), col)
			"creator":
				# A pencil over a star.
				var pts := PackedVector2Array()
				for k in 10:
					var a := -PI / 2 + TAU * k / 10.0
					pts.append(c + Vector2(cos(a), sin(a)) * r * (0.75 if k % 2 == 0 else 0.32))
				pts.append(pts[0])
				canvas.draw_polyline(pts, col, wdt, true)
			"controls":
				var body := Rect2(c - Vector2(r * 0.85, r * 0.42), Vector2(r * 1.7, r * 0.84))
				canvas.draw_rect(body, col, false, wdt)
				canvas.draw_line(c + Vector2(-r * 0.55, 0), c + Vector2(-r * 0.25, 0), col, wdt, true)
				canvas.draw_line(c + Vector2(-r * 0.4, -r * 0.15), c + Vector2(-r * 0.4, r * 0.15), col, wdt, true)
				canvas.draw_circle(c + Vector2(r * 0.35, -r * 0.1), r * 0.09, col)
				canvas.draw_circle(c + Vector2(r * 0.55, r * 0.1), r * 0.09, col)
			"editors":
				# A wrench.
				canvas.draw_line(c + Vector2(-r * 0.55, r * 0.55), c + Vector2(r * 0.2, -r * 0.2), col, wdt * 1.6, true)
				canvas.draw_arc(c + Vector2(r * 0.35, -r * 0.35), r * 0.3, PI * 0.9, PI * 2.6, 24, col, wdt * 1.4, true)
			"quit":
				var door := Rect2(c - Vector2(r * 0.5, r * 0.7), Vector2(r * 0.7, r * 1.4))
				canvas.draw_rect(door, col, false, wdt)
				canvas.draw_line(c + Vector2(0, 0), c + Vector2(r * 0.8, 0), col, wdt, true)
				canvas.draw_colored_polygon(PackedVector2Array([c + Vector2(r * 0.8, -r * 0.2), c + Vector2(r * 1.05, 0), c + Vector2(r * 0.8, r * 0.2)]), col)


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
