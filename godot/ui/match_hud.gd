extends CanvasLayer
## The in-match display: a card for each fighter (name, damage percent, stocks left), the clock, and big banners
## ("GO!", "GAME!"). It only draws what it is given; the match decides what that is.

const D := preload("res://ui/ui_draw.gd")

const PLAYER_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]

var board: Control


class Board extends Control:
	var data := {}

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

	func _card(i: int, count: int) -> void:
		var card := Vector2(310, 104)
		var gap := (size.x - 80.0 - card.x * count) / maxf(1.0, count - 1.0) if count > 1 else 0.0
		var x := 40.0 + i * (card.x + gap) if count > 1 else (size.x - card.x) * 0.5
		var rect := Rect2(Vector2(x, size.y - card.y - 26.0), card)
		var out: bool = not data.alive[i]
		var color: Color = PLAYER_COLORS[i]
		var fill := Color(0.96, 0.95, 0.88, 0.94) if not out else Color(0.6, 0.6, 0.62, 0.75)
		D.draw_slant(self, rect, fill, D.INK, 4.0, true)
		# A coloured flag on the left edge names the player.
		D.draw_slant(self, Rect2(rect.position + Vector2(8, 8), Vector2(54, 26)), color, D.INK, 3.0, false)
		D.draw_text_centered(self, "P%d" % (i + 1), Rect2(rect.position + Vector2(8, 8), Vector2(54, 26)), 18, Color(1, 1, 1))
		var name_text: String = data.names[i]
		draw_string(D.font(), rect.position + Vector2(76, 30), name_text.left(14), HORIZONTAL_ALIGNMENT_LEFT, -1, 22, D.INK)
		if out:
			D.draw_text_centered(self, "OUT", Rect2(rect.position + Vector2(40, 36), Vector2(rect.size.x - 60, 64)), 54, Color(0.35, 0.35, 0.4))
			return
		var percent: float = data.percent[i]
		var hot := clampf(percent / 180.0, 0.0, 1.0)
		var number_color := Color(0.11, 0.12, 0.17).lerp(Color(0.85, 0.1, 0.1), hot)
		var number := "%d" % int(percent)
		var f := D.font()
		var w := f.get_string_size(number, HORIZONTAL_ALIGNMENT_LEFT, -1, 56).x
		var tx := rect.end.x - 62.0 - w
		draw_string(f, Vector2(tx + 2, rect.position.y + 80), number, HORIZONTAL_ALIGNMENT_LEFT, -1, 56, Color(1, 1, 1, 0.8))
		draw_string(f, Vector2(tx, rect.position.y + 78), number, HORIZONTAL_ALIGNMENT_LEFT, -1, 56, number_color)
		draw_string(f, Vector2(rect.end.x - 56, rect.position.y + 78), "%", HORIZONTAL_ALIGNMENT_LEFT, -1, 28, number_color)
		# Stocks as little discs in the player's colour; "x N" if there are too many to draw.
		var stocks: int = data.stocks[i]
		if data.unlimited:
			draw_string(f, rect.position + Vector2(76, 88), "free play", HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(0.35, 0.35, 0.4))
		elif stocks <= 5:
			for s in stocks:
				var c := rect.position + Vector2(86 + s * 26, 76)
				draw_circle(c, 10, color)
				draw_arc(c, 10, 0, TAU, 20, D.INK, 3.0, true)
				draw_circle(c + Vector2(-3, -3), 3, Color(1, 1, 1, 0.6))
		else:
			draw_string(f, rect.position + Vector2(76, 90), "x %d" % stocks, HORIZONTAL_ALIGNMENT_LEFT, -1, 28, color)

	func _clock() -> void:
		var text: String = data.clock
		if text == "":
			return
		var rect := Rect2(Vector2(size.x * 0.5 - 90, 10), Vector2(180, 52))
		var urgent: bool = data.urgent
		D.draw_slant(self, rect, Color(0.8, 0.15, 0.15, 0.95) if urgent else Color(0.11, 0.12, 0.17, 0.9), Color(1, 1, 1, 0.9), 3.0, true)
		D.draw_text_centered(self, text, rect, 36, Color(1, 1, 1))

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


func _init() -> void:
	layer = 5


func build() -> void:
	board = Board.new()
	add_child(board)


## `d`: names, percent, stocks, alive, in_match (arrays per player), unlimited, clock, urgent, banner, banner_alpha.
func show_state(d: Dictionary) -> void:
	board.data = d
	board.queue_redraw()


func set_hud_visible(on: bool) -> void:
	visible = on
