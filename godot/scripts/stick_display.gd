extends Control
## Stick and button display. Shows the dead zone, the dash/tap threshold, and the wavedash cone
## (the green wedge: air dodge directions in it count as a wavedash).

const DEADZONE := 22.0 / 127.0
const THRESHOLD := 76.0 / 127.0
const RADIUS := 44.0

var stick := Vector2.ZERO  # -1..1, +y is up
var buttons := 0
var masks := {}
var min_down := 0.2
var color := Color.WHITE


func _init() -> void:
	custom_minimum_size = Vector2(190, 100)


func _draw() -> void:
	var c := Vector2(54, 50)
	# Wavedash cone: normalised dodge directions with y <= -min_down.
	var theta := asin(clampf(min_down, 0.0, 1.0))
	var pts := PackedVector2Array([c])
	var steps := 24
	var a0 := theta
	var a1 := PI - theta
	for i in steps + 1:
		var a := lerpf(a0, a1, float(i) / steps)
		pts.append(c + Vector2(cos(a), sin(a)) * RADIUS)
	draw_colored_polygon(pts, Color(0.3, 1.0, 0.5, 0.22))
	draw_arc(c, RADIUS, 0, TAU, 48, Color(1, 1, 1, 0.5), 1.5)
	draw_arc(c, RADIUS * THRESHOLD, 0, TAU, 40, Color(1, 1, 1, 0.25), 1.0)
	draw_arc(c, RADIUS * DEADZONE, 0, TAU, 24, Color(1, 1, 1, 0.25), 1.0)
	var dot := c + Vector2(stick.x, -stick.y) * RADIUS
	draw_line(c, dot, color, 2.0)
	draw_circle(dot, 5.0, color)

	var font := ThemeDB.fallback_font
	var names := [["jump", "JMP"], ["attack", "ATK"], ["special", "SPC"], ["shield", "SHD"], ["grab", "GRB"]]
	for i in names.size():
		var on: bool = (buttons & int(masks.get(names[i][0], 0))) != 0
		var r := Rect2(112 + (i % 2) * 40, 8 + (i / 2) * 30, 36, 24)
		draw_rect(r, color if on else Color(1, 1, 1, 0.12), true)
		draw_rect(r, Color(1, 1, 1, 0.5), false, 1.0)
		draw_string(font, r.position + Vector2(5, 17), names[i][1], HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color.BLACK if on else Color.WHITE)
