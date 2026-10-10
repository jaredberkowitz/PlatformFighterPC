extends CanvasLayer
## The in-match display, after the reference game's: for each fighter a tilted portrait of its own face (its real model and outfit,
## rendered once), its damage beside it in big italic numbers that heat from white through orange to deep red, its stocks as little heads
## and its name on a dark bar; then the clock and big banners ("GO!", "GAME!"). Drawn in the menus' style: flat cel colours, thick ink
## edges, a little slant. It only draws what it is given; the match decides what that is.

const D := preload("res://ui/ui_draw.gd")
const FighterView := preload("res://scripts/fighter_view.gd")
const Lighting := preload("res://scripts/lighting.gd")

const PLAYER_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]

var board: Control
var portraits: Array = []


class Board extends Control:
	var data := {}
	## The damage shown a moment ago, and how hard each number is shaking from the last hit (it settles over a third of a second).
	var last_percent := [0.0, 0.0, 0.0, 0.0]
	var shake := [0.0, 0.0, 0.0, 0.0]
	var last_ms := 0
	## Each fighter's portrait (the texture of its `Portrait`).
	var portraits: Array = []

	## Notices new damage: the number shakes, harder for a bigger hit.
	func take(d: Dictionary) -> void:
		var now := Time.get_ticks_msec()
		var dt := clampf((now - last_ms) / 1000.0, 0.0, 0.1) if last_ms > 0 else 0.0
		last_ms = now
		var percent: Array = d.get("percent", [])
		for i in mini(percent.size(), 4):
			var p: float = percent[i]
			if p > last_percent[i] + 0.01:
				shake[i] = clampf(shake[i] + (p - last_percent[i]) / 12.0, 0.0, 1.5)
			last_percent[i] = p
			shake[i] = move_toward(shake[i], 0.0, dt * 3.5)

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		if data.is_empty():
			return
		var count: int = data.names.size()
		for i in count:
			if data.in_match[i]:
				_card(i, count)
		_clock()
		_banner()
		_status()
		_offscreen()

	## A bubble on the screen edge for each fighter the camera cannot see, pointing at them, with their damage.
	func _offscreen() -> void:
		for m in data.get("offscreen", []):
			var at: Vector2 = m.at
			var dir: Vector2 = m.dir
			var color: Color = m.color
			var tip := at + dir * 52.0
			var side := dir.orthogonal() * 16.0
			draw_colored_polygon(PackedVector2Array([at + dir * 30.0 + side, tip, at + dir * 30.0 - side]), D.INK)
			draw_circle(at, 40.0, D.INK)
			draw_circle(at, 35.0, color)
			draw_circle(at, 27.0, Color(0.97, 0.96, 0.9))
			D.draw_text_centered(self, m.label, Rect2(at + Vector2(-30, -26), Vector2(60, 24)), 18, D.INK)
			D.draw_text_centered(self, "%d%%" % m.percent, Rect2(at + Vector2(-30, -4), Vector2(60, 26)), 20, color.darkened(0.35))

	## One fighter's display: the portrait, the damage, the stocks and the name bar, with the weapon's emblem faint behind.
	func _card(i: int, count: int) -> void:
		var block := Vector2(330, 126)
		var gap := (size.x - 80.0 - block.x * count) / maxf(1.0, count - 1.0) if count > 1 else 0.0
		var x := 40.0 + i * (block.x + gap) if count > 1 else (size.x - block.x) * 0.5
		var base := Vector2(x, size.y - block.y - 20.0)
		var out: bool = not data.alive[i]
		var color: Color = PLAYER_COLORS[i]
		var grey := Color(0.5, 0.5, 0.55)
		var classes: Array = data.get("classes", [])
		var cls: int = int(classes[i]) if i < classes.size() else 0
		var f := D.font()
		# The weapon's emblem, big and faint behind the number (where the reference shows a series symbol).
		_emblem_mark(cls, base + Vector2(250, 50), 46.0, Color(color, 0.3 if not out else 0.12), 9.0)
		# The name on a dark slanted bar with a stripe of the player's colour, and the player's tag (so players are told apart by more
		# than colour).
		var bar := Rect2(base + Vector2(92, 96), Vector2(232, 28))
		D.draw_slant(self, bar, Color(0.1, 0.11, 0.16, 0.94), D.INK, 3.0, true)
		draw_colored_polygon(D.skew_points(Rect2(bar.position + Vector2(4, bar.size.y - 6), Vector2(bar.size.x - 8, 4)), D.SKEW),
			color if not out else grey)
		var tag := Rect2(bar.position + Vector2(16, 4), Vector2(40, 18))
		D.draw_slant(self, tag, color if not out else grey, Color(0, 0, 0, 0), 0.0, false)
		D.draw_text_centered(self, "P%d" % (i + 1), tag, 16, Color(1, 1, 1))
		var name_text: String = String(data.names[i]).to_upper().left(14)
		var nw := f.get_string_size(name_text, HORIZONTAL_ALIGNMENT_LEFT, -1, 19).x
		draw_string(f, Vector2(bar.end.x - 18.0 - nw, bar.position.y + 20), name_text, HORIZONTAL_ALIGNMENT_LEFT, -1, 19, Color(1, 1, 1))
		_portrait(i, base + Vector2(54, 62), color, out)
		if out:
			_outlined(f, Vector2(base.x + 150, base.y + 84), "OUT", 60, Color(0.66, 0.66, 0.7))
			return
		# Stocks above the number.
		var stocks: int = data.stocks[i]
		var sy := base.y + 12.0
		if data.unlimited:
			_outlined(f, Vector2(base.x + 124, sy + 6), "FREE PLAY", 16, Color(1, 1, 1))
		elif stocks <= 6:
			for k in stocks:
				_head(Vector2(base.x + 136 + k * 23, sy), color)
		else:
			_head(Vector2(base.x + 136, sy), color)
			_outlined(f, Vector2(base.x + 152, sy + 8), "x%d" % stocks, 22, Color(1, 1, 1))
		# The damage: a big whole number, a small tenth and "%", right-aligned, heating with the damage. A hit makes it jump and shake
		# (and grow a little) for a moment.
		var percent: float = data.percent[i]
		var hot := heat(percent)
		var whole := "%d" % int(percent)
		var tenth := ".%d%%" % (int(percent * 10.0) % 10)
		var k: float = shake[i] if i < shake.size() else 0.0
		var t := Time.get_ticks_msec() / 1000.0
		var jolt := Vector2(sin(t * 61.0), cos(t * 47.0)) * 7.0 * k
		var big := 72 + int(10.0 * minf(k, 1.0))
		var small := 32
		var right := base.x + 318.0
		var baseline := base.y + 88.0
		var tw := f.get_string_size(tenth, HORIZONTAL_ALIGNMENT_LEFT, -1, small).x
		var ww := f.get_string_size(whole, HORIZONTAL_ALIGNMENT_LEFT, -1, big).x
		_outlined(f, Vector2(right - tw, baseline) + jolt, tenth, small, hot)
		_outlined(f, Vector2(right - tw - ww - 2.0, baseline) + jolt, whole, big, hot)

	## Text in the HUD's style: a soft drop shadow, a thick ink outline, then the fill.
	func _outlined(f: Font, at: Vector2, text: String, font_size: int, fill: Color) -> void:
		var o := maxi(6, font_size / 7)
		draw_string_outline(f, at + Vector2(3, 4), text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, o, Color(0, 0, 0, 0.3))
		draw_string_outline(f, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, o, D.INK)
		draw_string(f, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, fill)

	## The damage colour: white when fresh, through cream, orange and red to a deep red at high damage (the reference's scale).
	static func heat(percent: float) -> Color:
		var stops := [[0.0, Color(1, 1, 1)], [40.0, Color(1.0, 0.94, 0.66)], [75.0, Color(1.0, 0.68, 0.3)], [110.0, Color(0.98, 0.34, 0.24)],
			[150.0, Color(0.84, 0.1, 0.14)], [200.0, Color(0.58, 0.04, 0.1)]]
		for n in stops.size() - 1:
			var a: Array = stops[n]
			var b: Array = stops[n + 1]
			if percent <= float(b[0]):
				return (a[1] as Color).lerp(b[1], clampf((percent - float(a[0])) / (float(b[0]) - float(a[0])), 0.0, 1.0))
		return stops[stops.size() - 1][1]

	## The fighter's face in a tilted square: a backing in the player's colour, the portrait, a pale inner line and an ink edge.
	func _portrait(i: int, centre: Vector2, color: Color, out: bool) -> void:
		draw_set_transform(centre, deg_to_rad(-7.0), Vector2.ONE)
		var r := Rect2(-50, -50, 100, 100)
		draw_rect(Rect2(r.position + Vector2(5, 7), r.size), Color(0, 0, 0, 0.28))
		var back := color if not out else Color(0.55, 0.55, 0.58)
		draw_polygon(PackedVector2Array([r.position, Vector2(r.end.x, r.position.y), r.end, Vector2(r.position.x, r.end.y)]),
			PackedColorArray([back.lightened(0.45), back.lightened(0.3), back.darkened(0.1), back]))
		var tex: Texture2D = portraits[i] if i < portraits.size() else null
		if tex != null:
			draw_texture_rect(tex, r.grow(-3.0), false, Color(1, 1, 1) if not out else Color(0.55, 0.55, 0.6))
		draw_rect(r.grow(-4.0), Color(1, 1, 1, 0.55), false, 1.5)
		draw_rect(r, D.INK, false, 4.0)
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)

	## A stock: a little round head in the player's colour with two eyes, in ink.
	func _head(c: Vector2, color: Color) -> void:
		draw_circle(c + Vector2(1.5, 2.5), 10.0, Color(0, 0, 0, 0.25))
		draw_circle(c, 10.0, color.lightened(0.1))
		draw_arc(c, 10.0, 0, TAU, 24, D.INK, 2.5, true)
		draw_circle(c + Vector2(-3.4, -3.6), 2.6, Color(1, 1, 1, 0.55))
		draw_circle(c + Vector2(1.5, 0.5), 1.8, D.INK)
		draw_circle(c + Vector2(5.5, 0.5), 1.8, D.INK)

	## The fighter's weapon as a mark of radius `r` in one colour: a sword (class 0) or three claw marks (class 1).
	func _emblem_mark(cls: int, c: Vector2, r: float, color: Color, width: float) -> void:
		if cls == 1:
			for k in 3:
				var off := Vector2((k - 1) * r * 0.42, 0)
				var pts := PackedVector2Array()
				for t in 9:
					var u := float(t) / 8.0
					pts.append(c + off + Vector2(r * (0.3 - 0.6 * u + sin(u * PI) * 0.16), r * (-0.8 + 1.6 * u)))
				draw_polyline(pts, color, width * (1.0 - 0.2 * absf(k - 1)), true)
		else:
			draw_line(c + Vector2(-0.7, 0.75) * r, c + Vector2(0.75, -0.8) * r, color, width, true)
			draw_line(c + Vector2(-0.75, 0.1) * r, c + Vector2(-0.1, 0.8) * r, color, width * 0.85, true)
			draw_circle(c + Vector2(-0.8, 0.85) * r, width * 0.7, color)

	## The clock, in the same style (slant, ink edge, shadow): minutes and seconds as 00:12, with a word saying whether it is the time
	## left (a timed match) or the time played (a stock match with no time limit).
	func _clock() -> void:
		var text: String = data.clock
		if text == "":
			return
		var rect := Rect2(Vector2(size.x * 0.5 - 95, 10), Vector2(190, 64))
		var urgent: bool = data.urgent
		var down: bool = data.get("counting_down", true)
		D.draw_slant(self, rect, Color(0.8, 0.15, 0.15, 0.95) if urgent else Color(0.11, 0.12, 0.17, 0.92), D.INK, 4.0, true)
		D.draw_text_centered(self, "TIME LEFT" if down else "TIME PLAYED", Rect2(rect.position + Vector2(0, 4), Vector2(rect.size.x, 16)), 14,
			Color(1, 1, 1, 0.7))
		D.draw_text_centered(self, text, Rect2(rect.position + Vector2(0, 16), Vector2(rect.size.x, rect.size.y - 16)), 36, Color(1, 1, 1))

	func _status() -> void:
		var text: String = data.get("status", "")
		if text == "":
			return
		var lines := text.split("\n")
		var f := D.font()
		for i in lines.size():
			var w := f.get_string_size(lines[i], HORIZONTAL_ALIGNMENT_LEFT, -1, 22).x + 60.0
			var rect := Rect2(Vector2(14, 70 + i * 40), Vector2(minf(w, size.x - 28.0), 36))
			D.draw_slant(self, rect, Color(0.11, 0.12, 0.17, 0.85), Color(1, 1, 1, 0.7), 2.0, false)
			draw_string(f, rect.position + Vector2(28, 26), lines[i], HORIZONTAL_ALIGNMENT_LEFT, -1, 22, Color(1, 1, 1))

	func _banner() -> void:
		var text: String = data.banner
		if text == "":
			return
		var alpha: float = data.banner_alpha
		var rect := Rect2(Vector2(0, size.y * 0.28), Vector2(size.x, 150))
		var shade := Color(0, 0, 0, 0.35 * alpha)
		draw_rect(Rect2(0, rect.position.y + 20, size.x, 110), shade)
		D.draw_text_centered(self, text, Rect2(rect.position + Vector2(5, 6), rect.size), 120, Color(0, 0, 0, 0.5 * alpha))
		D.draw_text_centered(self, text, rect, 120, Color(1.0, 0.86, 0.25, alpha))


## A fighter's face, rendered once into a texture: its own model and outfit in a small private 3D world, the camera on its head turned a
## little toward the viewer. It renders for a few frames after a change (so the pose and the materials settle) and then stops, so the
## portraits cost nothing during play.
class Portrait extends SubViewport:
	var view: Node3D
	var frames_left := 0
	var shown: RefCounted

	func _init() -> void:
		size = Vector2i(256, 256)
		own_world_3d = true
		transparent_bg = true
		msaa_3d = Viewport.MSAA_4X
		render_target_update_mode = SubViewport.UPDATE_DISABLED
		var root := Node3D.new()
		add_child(root)
		var env := Environment.new()
		env.background_mode = Environment.BG_CLEAR_COLOR
		var we := WorldEnvironment.new()
		we.environment = env
		root.add_child(we)
		Lighting.apply(env, root, true)
		var cam := Camera3D.new()
		cam.fov = 30.0
		root.add_child(cam)
		cam.look_at_from_position(Vector3(0.3, 2.02, 2.5), Vector3(0.06, 1.9, 0))
		view = FighterView.new()
		root.add_child(view)
		view.build(0)

	func show_look(look: RefCounted) -> void:
		if look == null or look == shown:
			return
		shown = look
		view.rebuild(look)
		view.blade_pivot.visible = false
		view.percent_label.visible = false
		view.name_label.visible = false
		view.model.rotation.y = 0.45
		view.set_gaze(1)
		frames_left = 4
		render_target_update_mode = SubViewport.UPDATE_ALWAYS

	func _process(_delta: float) -> void:
		if frames_left > 0:
			frames_left -= 1
			if frames_left == 0:
				render_target_update_mode = SubViewport.UPDATE_DISABLED


func _init() -> void:
	layer = 5


func build() -> void:
	board = Board.new()
	add_child(board)
	for i in 4:
		var p := Portrait.new()
		add_child(p)
		portraits.append(p)
		board.portraits.append(p.get_texture())


## `d`: names, percent, stocks, alive, in_match, classes, looks (loadouts, for the portraits) (arrays per player), unlimited, clock,
## counting_down, urgent, banner, banner_alpha.
func show_state(d: Dictionary) -> void:
	var looks: Array = d.get("looks", [])
	for i in mini(looks.size(), portraits.size()):
		if d.in_match[i]:
			portraits[i].show_look(looks[i])
	board.take(d)
	board.data = d
	board.queue_redraw()


func set_hud_visible(on: bool) -> void:
	visible = on
