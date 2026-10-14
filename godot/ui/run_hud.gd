extends CanvasLayer
## The run's display (the single-player run, `scripts/runner.gd`), in the menus' painted style: the distance on an ink brush stroke at
## the top left with the coins under it, the combo and its coin multiplier at the top right (a bar fills toward the next step), and the
## fighter's damage and lives at the bottom left. A banner in the middle for the start, a lost life and the end. At the top in the
## middle, the map: a strip of the rooms behind (dimmed; those dropped behind the wall shown shut), the current one and the next one
## ahead, each with its type's icon. A prompt at a shop's door, a hint while an arena's gate is shut, and the shop itself when open.
## Everything it shows comes from `show_state`; nothing here decides anything.

const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")

var board: Control


class Board extends Control:
	## {metres, coins, kills, combo, best, mult, step, lives, max_lives, percent, name, banner, banner_alpha, combo_flash, map, door,
	## gate, shop, shop_names}
	var data := {}

	const ROOM_STREET := 0
	const ROOM_ARENA := 1
	const ROOM_SHOP := 2
	const ROOM_REST := 3
	const NOTES := ["", "Bought!", "Not enough coins", "Sold out", "Not needed"]

	## One room's icon on the map (placeholder shapes for the art director): a street is a little building, an arena crossed
	## swords, a shop a coin, a rest stop a green cross.
	func room_icon(at: Vector2, kind: int, r: float, alpha: float) -> void:
		var ink := Color(Art.INK, alpha)
		match kind:
			ROOM_ARENA:
				draw_line(at + Vector2(-r, -r) * 0.6, at + Vector2(r, r) * 0.6, Color(Art.CORAL, alpha), 4.0)
				draw_line(at + Vector2(r, -r) * 0.6, at + Vector2(-r, r) * 0.6, Color(Art.CORAL, alpha), 4.0)
			ROOM_SHOP:
				draw_circle(at, r * 0.62, ink)
				draw_circle(at, r * 0.5, Color(Art.SUN, alpha))
			ROOM_REST:
				draw_rect(Rect2(at - Vector2(r * 0.18, r * 0.6), Vector2(r * 0.36, r * 1.2)), Color(Art.GREEN, alpha))
				draw_rect(Rect2(at - Vector2(r * 0.6, r * 0.18), Vector2(r * 1.2, r * 0.36)), Color(Art.GREEN, alpha))
			_:
				draw_rect(Rect2(at + Vector2(-r * 0.45, -r * 0.55), Vector2(r * 0.9, r * 1.1)), Color(Art.BLUE, alpha * 0.85))
				for n in 2:
					draw_rect(Rect2(at + Vector2(-r * 0.3 + n * r * 0.35, -r * 0.35), Vector2(r * 0.22, r * 0.22)), Color(1, 1, 1, alpha))

	## The map: a short strip, oldest room on the left. Rows of `map` are [number, type, status (0 shut behind, 1 behind, 2 here,
	## 3 ahead)].
	func draw_map(centre_x: float, top: float) -> void:
		var map: PackedInt32Array = data.get("map", PackedInt32Array())
		var count := map.size() / 3
		if count == 0:
			return
		var gap := 58.0
		var start := centre_x - (count - 1) * gap * 0.5
		var y := top + 30.0
		draw_line(Vector2(start, y), Vector2(start + (count - 1) * gap, y), Color(Art.INK, 0.6), 6.0)
		for n in count:
			var kind := map[n * 3 + 1]
			var status := map[n * 3 + 2]
			var at := Vector2(start + n * gap, y)
			var r := 24.0 if status == 2 else 19.0
			var alpha := 1.0 if status >= 2 else 0.55
			draw_circle(at, r + 3.0, Art.INK)
			draw_circle(at, r, Color(Art.CREAM, 0.35 if status == 0 else alpha))
			if status == 2:
				draw_arc(at, r + 6.0, 0.0, TAU, 32, Art.SUN, 4.0)
			if status == 0:
				# Shut behind the wall: bars across it.
				for k in 3:
					var bx := at.x - r * 0.5 + k * r * 0.5
					draw_line(Vector2(bx, at.y - r * 0.8), Vector2(bx, at.y + r * 0.8), Color(Art.INK, 0.8), 3.0)
			else:
				room_icon(at, kind, r, alpha)
			if status == 3:
				draw_string(D.font(), at + Vector2(-14, r + 22), "next", HORIZONTAL_ALIGNMENT_LEFT, -1, 15, Color(1, 1, 1, 0.8))

	## The shop: its wares in a list with prices, the row picked, what the last press did, and how to leave.
	func draw_shop(w: float, h: float) -> void:
		var shop: PackedInt32Array = data.get("shop", PackedInt32Array())
		if shop.size() < 6 or shop[0] == 0:
			return
		var names: PackedStringArray = data.get("shop_names", PackedStringArray())
		var big := D.display_font()
		var f := D.font()
		var wares := shop[5]
		var panel := Rect2(Vector2(w * 0.5 - 300, h * 0.2), Vector2(600, 170 + (wares + 1) * 64))
		draw_rect(Rect2(Vector2.ZERO, Vector2(w, h)), Color(Art.INK, 0.35))
		Art.card(self, panel, Art.CREAM, 401, -1.0, -1.0, Art.SUN, 0.0, Color(0, 0, 0, 0), 6.0)
		var head := Rect2(panel.position + Vector2(150, -30), Vector2(300, 70))
		Art.brush_band(self, head, Art.INK, 403, 16.0, true, 1.0)
		Art.title_centered(self, "SHOP", head.grow_individual(-8, 0, -16, 0), 46, Color.WHITE, 0.0)
		draw_string(big, panel.position + Vector2(panel.size.x - 150, 74), "%d" % shop[4], HORIZONTAL_ALIGNMENT_LEFT, -1, 30, Art.INK)
		draw_circle(panel.position + Vector2(panel.size.x - 170, 63), 12.0, Art.SUN)
		for n in wares + 1:
			var row := Rect2(panel.position + Vector2(30, 96 + n * 64), Vector2(panel.size.x - 60, 54))
			var picked := shop[1] == n
			if picked:
				Art.brush_band(self, row, Art.SUN, 411 + n, 10.0, false, 0.7)
			if n == wares:
				draw_string(big, row.position + Vector2(18, 38), "Leave", HORIZONTAL_ALIGNMENT_LEFT, -1, 30, Art.INK)
				continue
			var o := 6 + n * 4
			var label := names[n] if n < names.size() else "?"
			draw_string(big, row.position + Vector2(18, 38), label, HORIZONTAL_ALIGNMENT_LEFT, -1, 30, Art.INK)
			var limit := shop[o + 2]
			var sold := shop[o + 3]
			var tag := "%d" % shop[o + 1]
			if limit > 0 and sold >= limit:
				tag = "sold out"
			draw_string(big, row.position + Vector2(row.size.x - 150, 38), tag, HORIZONTAL_ALIGNMENT_LEFT, -1, 28,
				Color(Art.INK, 0.45 if tag == "sold out" else 1.0))
			if limit > 0:
				draw_string(f, row.position + Vector2(row.size.x - 270, 36), "%d per shop" % limit, HORIZONTAL_ALIGNMENT_LEFT, -1, 16,
					Color(Art.INK, 0.55))
		var note := shop[2]
		if note > 0 and note < NOTES.size():
			draw_string(big, panel.position + Vector2(36, panel.size.y - 52), NOTES[note], HORIZONTAL_ALIGNMENT_LEFT, -1, 24,
				Art.GREEN if note == 1 else Art.CORAL)
		draw_string(f, panel.position + Vector2(36, panel.size.y - 20), "Up / down: choose    Attack or jump: buy    Shield or special: leave",
			HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(Art.INK, 0.6))

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	## The damage's colour: white, warming through yellow and orange to red as it climbs (as the match display's).
	static func percent_colour(p: float) -> Color:
		var t := clampf(p / 150.0, 0.0, 1.0)
		if t < 0.5:
			return Color.WHITE.lerp(Color(1.0, 0.78, 0.25), t * 2.0)
		return Color(1.0, 0.78, 0.25).lerp(Color(0.9, 0.2, 0.15), (t - 0.5) * 2.0)

	func _draw() -> void:
		if data.is_empty():
			return
		var w := size.x
		var h := size.y
		var big := D.display_font()
		var f := D.font()
		# The distance.
		var band := Rect2(Vector2(22, 18), Vector2(260, 64))
		Art.brush_band(self, band, Art.INK, 211, 16.0, true, 1.0)
		Art.title_centered(self, "%d m" % int(data.get("metres", 0)), band.grow_individual(-10, 0, -18, 0), 46, Color.WHITE, 0.0)
		# The coins: a gold coin and the count.
		var coin_at := Vector2(52, 112)
		draw_circle(coin_at, 17.0, Art.INK)
		draw_circle(coin_at, 14.0, Art.SUN)
		draw_circle(coin_at + Vector2(-3, -3), 5.0, Color(1, 1, 1, 0.55))
		draw_string(big, Vector2(78, 124), str(int(data.get("coins", 0))), HORIZONTAL_ALIGNMENT_LEFT, -1, 34, Color.WHITE)
		draw_string(f, Vector2(80, 150), "%d beaten" % int(data.get("kills", 0)), HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(1, 1, 1, 0.8))
		# The combo, when there is one: the count, the multiplier on a gold stroke, a bar toward the next step.
		var combo := int(data.get("combo", 0))
		var mult := int(data.get("mult", 1))
		var right := Rect2(Vector2(w - 300, 18), Vector2(278, 64))
		var flash := float(data.get("combo_flash", 0.0))
		if combo > 0:
			Art.brush_band(self, right, Art.INK, 223, 16.0, true, 0.92)
			draw_string(big, right.position + Vector2(18, 46), "COMBO %d" % combo, HORIZONTAL_ALIGNMENT_LEFT, -1, 34, Color.WHITE.lerp(Art.SUN, flash))
			var tag := Rect2(right.position + Vector2(right.size.x - 92, 6), Vector2(84, 52))
			Art.brush_band(self, tag, Art.SUN, 227, 10.0, true, 0.6)
			Art.title_centered(self, "x%d" % mult, tag.grow_individual(0, 0, -8, 0), 38, Art.INK, 0.0)
			var step := maxi(1, int(data.get("step", 5)))
			var fill := 1.0 if mult >= int(data.get("max_mult", 5)) else float(combo % step) / float(step)
			var bar := Rect2(right.position + Vector2(18, right.size.y + 8), Vector2(right.size.x - 36, 10))
			draw_rect(bar, Color(Art.INK, 0.7))
			draw_rect(Rect2(bar.position, Vector2(bar.size.x * fill, bar.size.y)), Art.SUN)
		draw_string(f, Vector2(w - 290, right.position.y + right.size.y + 46), "best combo %d" % int(data.get("best", 0)), HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(1, 1, 1, 0.75))
		# The fighter: name, damage, lives.
		var plate := Rect2(Vector2(22, h - 118), Vector2(300, 96))
		Art.card(self, plate, Art.CREAM, 241, -1.0, -1.0, Color(0, 0, 0, 0), 0.0, Color(0, 0, 0, 0), 4.0)
		draw_string(big, plate.position + Vector2(16, 30), str(data.get("name", "")).left(16).to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 22, Art.INK)
		var p := float(data.get("percent", 0.0))
		var pc := percent_colour(p)
		var pct := "%d%%" % int(p)
		draw_string(big, plate.position + Vector2(18, 84), pct, HORIZONTAL_ALIGNMENT_LEFT, -1, 54, Art.INK)
		draw_string(big, plate.position + Vector2(15, 81), pct, HORIZONTAL_ALIGNMENT_LEFT, -1, 54, pc if p > 0.5 else Color(1, 1, 1))
		var lives := int(data.get("lives", 0))
		for n in int(data.get("max_lives", 3)):
			var at := plate.position + Vector2(plate.size.x - 34 - n * 32, 26)
			draw_circle(at, 12.0, Art.INK)
			draw_circle(at, 9.0, Color(0.95, 0.35, 0.4) if n < lives else Color(1, 1, 1, 0.25))
		# The map, and what this spot offers.
		draw_map(w * 0.5, 14.0)
		var hint := ""
		if bool(data.get("door", false)):
			hint = "UP: ENTER THE SHOP"
		elif bool(data.get("gate", false)):
			hint = "THE GATE IS SHUT: BEAT THEM ALL"
		if hint != "":
			var hr := Rect2(Vector2(w * 0.5 - 230, h - 104), Vector2(460, 56))
			Art.brush_band(self, hr, Art.INK, 261, 12.0, true, 0.9)
			Art.title_centered(self, hint, hr.grow_individual(-8, 0, -12, 0), 26, Art.SUN if hint.begins_with("UP") else Color.WHITE, 0.0)
		draw_shop(w, h)
		# The banner.
		var banner := str(data.get("banner", ""))
		if banner != "":
			var a := float(data.get("banner_alpha", 1.0))
			var r := Rect2(Vector2(w * 0.5 - 300, h * 0.36), Vector2(600, 110))
			Art.brush_band(self, r, Color(Art.INK, a), 251, 22.0, true, 1.0)
			Art.title_centered(self, banner, r.grow_individual(-12, 0, -26, 0), 72, Color(1, 1, 1, a), 0.0)


func _init() -> void:
	layer = 5


func build() -> void:
	board = Board.new()
	add_child(board)


func show_state(d: Dictionary) -> void:
	if board == null:
		build()
	if d != board.data:
		board.data = d
		board.queue_redraw()
