extends Control
## A 2D side view of a stage that can be edited with the mouse: drag a platform to move it, drag its ends to resize it,
## drag a spawn point or a ledge. The canvas edits a working copy of the stage section and hands it back when you let go.

signal edited(tree: Dictionary)
signal selected(kind: String, index: int)

var tree: Dictionary = {}
var sel_kind := ""
var sel_index := -1
var drag_kind := ""      # what is being dragged: "move", "left", "right", "point"
var drag_start_world := Vector2.ZERO
var drag_original: Dictionary = {}
const SNAP := 0.25
const GRAB_PIXELS := 9.0


func _init() -> void:
	custom_minimum_size = Vector2(420, 330)
	clip_contents = true
	mouse_filter = Control.MOUSE_FILTER_STOP


func set_tree(t: Dictionary) -> void:
	tree = t
	queue_redraw()


func select(kind: String, index: int) -> void:
	sel_kind = kind
	sel_index = index
	queue_redraw()


func _num(b: Dictionary, name: String) -> float:
	for it in b.items:
		if it.t == "field" and it.name == name:
			return it.value.to_float()
	return 0.0


func _text(b: Dictionary, name: String) -> String:
	for it in b.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


func _blocks(kind: String) -> Array:
	var out := []
	if tree.size() == 0:
		return out
	for it in tree.items:
		if it.t == "block" and it.kind == kind:
			out.append(it)
	return out


func _put(b: Dictionary, name: String, value: float) -> void:
	var text := "%s" % snappedf(value, SNAP)
	for it in b.items:
		if it.t == "field" and it.name == name:
			it.value = text
			return
	b.items.append({"t": "field", "name": name, "value": text})


# ---- Coordinates ----

var zoomed := false


func set_zoom(on: bool) -> void:
	zoomed = on
	queue_redraw()


func _bounds() -> Rect2:
	if zoomed and tree.size() > 0:
		# Frame the platforms, ledges and spawns with some room around them.
		var lo := Vector2(1e9, 1e9)
		var hi := Vector2(-1e9, -1e9)
		for it in tree.items:
			if it.t != "block":
				continue
			for pair in [["left", "bottom"], ["right", "y"], ["x", "y"]]:
				if _text(it, pair[0]) != "" and _text(it, pair[1]) != "":
					var p := Vector2(_num(it, pair[0]), _num(it, pair[1]))
					lo = lo.min(p)
					hi = hi.max(p)
		if lo.x < hi.x:
			return Rect2(lo - Vector2(6, 6), (hi - lo) + Vector2(12, 12 + 6))
	var l := _text_num("blast_left", -28.0)
	var r := _text_num("blast_right", 28.0)
	var b := _text_num("blast_bottom", -17.0)
	var t := _text_num("blast_top", 24.0)
	return Rect2(l, b, r - l, t - b)


func _text_num(name: String, fallback: float) -> float:
	var v := _text(tree, name) if tree.size() > 0 else ""
	return v.to_float() if v != "" else fallback


func _scale() -> float:
	var bb := _bounds()
	return minf((size.x - 20.0) / maxf(bb.size.x, 1.0), (size.y - 20.0) / maxf(bb.size.y, 1.0))


func to_screen(p: Vector2) -> Vector2:
	var bb := _bounds()
	var s := _scale()
	return Vector2(10.0 + (p.x - bb.position.x) * s, size.y - 10.0 - (p.y - bb.position.y) * s)


func to_world(p: Vector2) -> Vector2:
	var bb := _bounds()
	var s := _scale()
	return Vector2((p.x - 10.0) / s + bb.position.x, (size.y - 10.0 - p.y) / s + bb.position.y)


# ---- Drawing ----

func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.12, 0.13, 0.17))
	if tree.size() == 0:
		return
	var bb := _bounds()
	var tl := to_screen(Vector2(bb.position.x, bb.end.y))
	var br := to_screen(Vector2(bb.end.x, bb.position.y))
	draw_rect(Rect2(tl, br - tl), Color(1, 0.4, 0.4, 0.8), false, 2.0)
	draw_string(ThemeDB.fallback_font, tl + Vector2(6, 14), "blast zone: leaving it costs a stock", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(1, 0.5, 0.5, 0.7))
	var platforms := _blocks("platform")
	for i in platforms.size():
		var p: Dictionary = platforms[i]
		var left := _num(p, "left")
		var right := _num(p, "right")
		var y := _num(p, "y")
		var bottom := _num(p, "bottom")
		var passthrough := _text(p, "pass_through") == "true"
		var chosen := sel_kind == "platform" and sel_index == i
		if passthrough:
			var a := to_screen(Vector2(left, y))
			var b := to_screen(Vector2(right, y))
			draw_line(a, b, Color(1.0, 0.72, 0.4), 6.0)
			if chosen:
				draw_line(a, b, Color(1, 1, 1), 2.0)
		else:
			var a := to_screen(Vector2(left, y))
			var b := to_screen(Vector2(right, bottom))
			draw_rect(Rect2(a, b - a), Color(0.6, 0.45, 0.6, 0.9))
			draw_rect(Rect2(a, Vector2(b.x - a.x, 4)), Color(0.5, 0.85, 0.4))
			if chosen:
				draw_rect(Rect2(a, b - a), Color(1, 1, 1), false, 2.0)
		if chosen:
			for x in [left, right]:
				draw_circle(to_screen(Vector2(x, y)), 5.0, Color(1, 1, 0.4))
	var ledges := _blocks("ledge")
	for i in ledges.size():
		var l: Dictionary = ledges[i]
		var pos := to_screen(Vector2(_num(l, "x"), _num(l, "y")))
		var chosen := sel_kind == "ledge" and sel_index == i
		draw_circle(pos, 6.0 if chosen else 4.5, Color(0.5, 0.9, 1.0))
		var side := _text(l, "side").to_int()
		draw_line(pos, pos + Vector2(-9.0 * side, 0), Color(0.5, 0.9, 1.0), 2.0)
	var spawns := _blocks("spawn")
	for i in spawns.size():
		var s: Dictionary = spawns[i]
		var pos := to_screen(Vector2(_num(s, "x"), _num(s, "y")))
		var chosen := sel_kind == "spawn" and sel_index == i
		draw_circle(pos + Vector2(0, -7), 6.0 if chosen else 5.0, Color(1.0, 0.85, 0.2))
		draw_line(pos, pos + Vector2(0, -7), Color(1, 0.85, 0.2), 2.0)
		draw_string(ThemeDB.fallback_font, pos + Vector2(-3, -4), "%d" % (i + 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 9, Color(0, 0, 0))


# ---- Mouse ----

func _hit(pos: Vector2) -> Array:
	# Points first (spawns, ledges), then platform ends, then platform bodies.
	for kind in ["spawn", "ledge"]:
		var list := _blocks(kind)
		for i in list.size():
			var o: Dictionary = list[i]
			var sp := to_screen(Vector2(_num(o, "x"), _num(o, "y")))
			if kind == "spawn":
				sp += Vector2(0, -7)
			if sp.distance_to(pos) <= GRAB_PIXELS:
				return [kind, i, "point"]
	var platforms := _blocks("platform")
	for i in platforms.size():
		var p: Dictionary = platforms[i]
		var a := to_screen(Vector2(_num(p, "left"), _num(p, "y")))
		var b := to_screen(Vector2(_num(p, "right"), _num(p, "y")))
		if a.distance_to(pos) <= GRAB_PIXELS:
			return ["platform", i, "left"]
		if b.distance_to(pos) <= GRAB_PIXELS:
			return ["platform", i, "right"]
	for i in platforms.size():
		var p: Dictionary = platforms[i]
		var top := to_screen(Vector2(_num(p, "left"), _num(p, "y")))
		var bot := to_screen(Vector2(_num(p, "right"), _num(p, "bottom")))
		var rect := Rect2(top, bot - top).abs().grow(5.0)
		if rect.has_point(pos):
			return ["platform", i, "move"]
	return []


func _gui_input(event: InputEvent) -> void:
	if tree.size() == 0:
		return
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			var h := _hit(event.position)
			if h.is_empty():
				sel_kind = ""
				sel_index = -1
				drag_kind = ""
			else:
				sel_kind = h[0]
				sel_index = h[1]
				drag_kind = h[2]
				drag_start_world = to_world(event.position)
				drag_original = tree.duplicate(true)
				selected.emit(sel_kind, sel_index)
			queue_redraw()
		elif drag_kind != "":
			drag_kind = ""
			edited.emit(tree)
	elif event is InputEventMouseMotion and drag_kind != "":
		var d := to_world(event.position) - drag_start_world
		var work := drag_original.duplicate(true)
		var list := []
		for it in work.items:
			if it.t == "block" and it.kind == sel_kind:
				list.append(it)
		var o: Dictionary = list[sel_index]
		match [sel_kind, drag_kind]:
			["platform", "move"]:
				for f in ["left", "right"]:
					_put(o, f, _num(drag_original_block(), f) + d.x)
				for f in ["y", "bottom"]:
					_put(o, f, _num(drag_original_block(), f) + d.y)
			["platform", "left"]:
				_put(o, "left", minf(_num(drag_original_block(), "left") + d.x, _num(o, "right") - SNAP))
			["platform", "right"]:
				_put(o, "right", maxf(_num(drag_original_block(), "right") + d.x, _num(o, "left") + SNAP))
			[_, "point"]:
				_put(o, "x", _num(drag_original_block(), "x") + d.x)
				_put(o, "y", _num(drag_original_block(), "y") + d.y)
		tree = work
		queue_redraw()


func drag_original_block() -> Dictionary:
	var n := 0
	for it in drag_original.items:
		if it.t == "block" and it.kind == sel_kind:
			if n == sel_index:
				return it
			n += 1
	return {}
