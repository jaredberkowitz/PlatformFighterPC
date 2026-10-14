extends CanvasLayer
## The run's display (the single-player run, `scripts/runner.gd`), in the menus' painted style: the distance on an ink brush stroke at
## the top left with the coins under it, the combo and its coin multiplier at the top right (a bar fills toward the next step), and the
## fighter's damage and lives at the bottom left. A banner in the middle for the start, a lost life and the end.
## Everything it shows comes from `show_state`; nothing here decides anything.

const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")

var board: Control


class Board extends Control:
	## {metres, coins, kills, combo, best, mult, step, lives, max_lives, percent, name, banner, banner_alpha, combo_flash}
	var data := {}

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
