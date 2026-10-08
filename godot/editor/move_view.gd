extends Control
## Draws one move: a timeline of its hitboxes (a bar per hitbox over the move's frames) and, below it, a side view of
## the fighter's body with the hitboxes that are active on the selected frame, at their real size and position. This is the
## "draw the real hitboxes over the model" view from plan 7.3: the numbers shown are the numbers the simulation uses.

signal frame_changed(frame: int)

var total := 30
var boxes: Array = []     # each {start,end,x,y,radius,kind,priority,group}
var motions: Array = []   # each {start,end,vx,vy}
var half_width := 0.6
var body_height := 2.2
var frame := 0
var projectile_frame := -1
var title := ""

const KIND_COLORS := {"normal": Color(1.0, 0.35, 0.3), "grab": Color(0.35, 0.9, 0.45), "throw": Color(0.4, 0.6, 1.0), "pummel": Color(1.0, 0.85, 0.3)}
const ROW_H := 14.0
const TIMELINE_TOP := 8.0


func _init() -> void:
	custom_minimum_size = Vector2(420, 330)
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true


func set_move(data: Dictionary) -> void:
	total = maxi(int(data.get("total", 30)), 1)
	boxes = data.get("boxes", [])
	motions = data.get("motions", [])
	projectile_frame = int(data.get("projectile_frame", -1))
	title = String(data.get("title", ""))
	frame = clampi(frame, 0, total)
	queue_redraw()


func set_body(half_w: float, height: float) -> void:
	half_width = half_w
	body_height = height
	queue_redraw()


func set_frame(f: int) -> void:
	frame = clampi(f, 0, total)
	queue_redraw()
	frame_changed.emit(frame)


func _timeline_height() -> float:
	return TIMELINE_TOP + ROW_H * (maxi(boxes.size(), 1) + 1) + 6.0


func _frame_x(f: float) -> float:
	return 8.0 + (size.x - 16.0) * f / float(total)


func _gui_input(event: InputEvent) -> void:
	var pressed: bool = (event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT) or (event is InputEventMouseMotion and (event.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0)
	if pressed and event.position.y < _timeline_height():
		set_frame(roundi((event.position.x - 8.0) / maxf(size.x - 16.0, 1.0) * total))


func _draw() -> void:
	var font := ThemeDB.fallback_font
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.12, 0.13, 0.17))
	# ---- Timeline ----
	var th := _timeline_height()
	draw_rect(Rect2(0, 0, size.x, th), Color(0.16, 0.17, 0.22))
	var step := 1 if total <= 40 else (5 if total <= 120 else 10)
	for f in range(0, total + 1, step):
		var x := _frame_x(f)
		draw_line(Vector2(x, TIMELINE_TOP), Vector2(x, th - 4.0), Color(1, 1, 1, 0.08))
		if f % (step * (2 if total > 40 else 5)) == 0:
			draw_string(font, Vector2(x + 2, th - 6.0), str(f), HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(1, 1, 1, 0.5))
	for m in motions:
		var y := TIMELINE_TOP
		draw_rect(Rect2(_frame_x(float(m.start)), y, maxf(_frame_x(float(m.end) + 1.0) - _frame_x(float(m.start)), 2.0), 5.0), Color(0.6, 0.8, 1.0, 0.7))
	for i in boxes.size():
		var b: Dictionary = boxes[i]
		var y := TIMELINE_TOP + 8.0 + ROW_H * i
		var c: Color = KIND_COLORS.get(b.kind, KIND_COLORS.normal)
		var active := frame >= int(b.start) and frame <= int(b.end)
		var r := Rect2(_frame_x(float(b.start)), y, maxf(_frame_x(float(b.end) + 1.0) - _frame_x(float(b.start)), 3.0), ROW_H - 3.0)
		draw_rect(r, Color(c.r, c.g, c.b, 0.95 if active else 0.5))
		draw_string(font, Vector2(r.position.x + 2, y + 9), "%d" % (i + 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 9, Color(0, 0, 0, 0.8))
	if projectile_frame >= 0:
		draw_circle(Vector2(_frame_x(float(projectile_frame)), TIMELINE_TOP + 2.0), 3.5, Color(0.8, 0.5, 1.0))
	var cx := _frame_x(float(frame))
	draw_line(Vector2(cx, 0), Vector2(cx, th), Color(1, 1, 0.4), 2.0)
	draw_string(font, Vector2(6, 12), "frame %d of %d%s" % [frame, total, ("   " + title) if title != "" else ""], HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(1, 1, 0.6))

	# ---- Side view ----
	var area := Rect2(0, th, size.x, size.y - th)
	var scale := minf(area.size.y / 5.2, area.size.x / 11.0)
	var origin := Vector2(area.position.x + area.size.x * 0.34, area.end.y - 22.0)
	draw_line(Vector2(area.position.x, origin.y), Vector2(area.end.x, origin.y), Color(0.5, 0.8, 0.4), 2.0)
	# Body: the ECB outline and a round-body stand-in, both from the fighter's numbers.
	var body := Rect2(origin.x - half_width * scale, origin.y - body_height * scale, half_width * 2.0 * scale, body_height * scale)
	draw_rect(body, Color(1.0, 0.85, 0.2, 0.25))
	draw_rect(body, Color(1.0, 0.85, 0.2, 0.9), false, 1.5)
	draw_string(font, Vector2(origin.x - 28, origin.y + 14), "feet", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(1, 1, 1, 0.45))
	draw_line(origin + Vector2(0, -6), origin + Vector2(0, 6), Color(1, 1, 1, 0.5))
	for i in boxes.size():
		var b: Dictionary = boxes[i]
		var active := frame >= int(b.start) and frame <= int(b.end)
		var c: Color = KIND_COLORS.get(b.kind, KIND_COLORS.normal)
		var centre := origin + Vector2(float(b.x), -float(b.y)) * scale
		var rad := float(b.radius) * scale
		if active:
			draw_circle(centre, rad, Color(c.r, c.g, c.b, 0.4))
			draw_arc(centre, rad, 0, TAU, 40, Color(c.r, c.g, c.b, 1.0), 2.0)
			draw_string(font, centre - Vector2(3, -4), "%d" % (i + 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(1, 1, 1))
		else:
			draw_arc(centre, rad, 0, TAU, 40, Color(c.r, c.g, c.b, 0.25), 1.0)
	draw_string(font, Vector2(area.end.x - 150, area.position.y + 16), "side view, facing right", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(1, 1, 1, 0.4))
