extends CanvasLayer
## The results screen shown when a match ends: who won (or a draw), how each fighter finished, and what to do next.
## Left and right choose, Enter (or J, Space) picks, the mouse works too. It reports the choice through `chosen`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")

signal chosen(action: String)

const PLAYER_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]

var root: Control
var buttons: Array = []
var actions: Array = []
var index := 0
var card_data: Array = []
var title := ""
var board: Control
var note := ""


class Board extends Control:
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var r = owner_screen
		draw_rect(Rect2(Vector2.ZERO, size), Color(0.05, 0.06, 0.1, 0.72))
		var banner := Rect2(Vector2(size.x * 0.5 - 380, 70), Vector2(760, 110))
		D.draw_slant(self, banner, D.GOLD, D.INK, 5.0, true)
		D.draw_text_centered(self, r.title, banner, 66, D.INK)
		var count: int = r.card_data.size()
		var card := Vector2(300, 250)
		var total := count * card.x + (count - 1) * 30.0
		for i in count:
			var c: Dictionary = r.card_data[i]
			var rect := Rect2(Vector2(size.x * 0.5 - total * 0.5 + i * (card.x + 30.0), 220), card)
			var winner: bool = c.winner
			D.draw_slant(self, rect, D.CREAM if winner else Color(0.78, 0.78, 0.8), D.GOLD if winner else D.INK, 6.0 if winner else 4.0, true)
			D.draw_slant(self, Rect2(rect.position + Vector2(14, 14), Vector2(70, 30)), PLAYER_COLORS[i], D.INK, 3.0, false)
			D.draw_text_centered(self, "P%d" % (i + 1), Rect2(rect.position + Vector2(14, 14), Vector2(70, 30)), 20, Color(1, 1, 1))
			draw_string(D.font(), rect.position + Vector2(30, 90), str(c.name).left(16), HORIZONTAL_ALIGNMENT_LEFT, -1, 32, D.INK)
			draw_string(D.font(), rect.position + Vector2(30, 140), "Stocks left", HORIZONTAL_ALIGNMENT_LEFT, -1, 22, Color(0.3, 0.3, 0.36))
			draw_string(D.font(), rect.position + Vector2(190, 140), str(c.stocks), HORIZONTAL_ALIGNMENT_LEFT, -1, 30, D.INK)
			draw_string(D.font(), rect.position + Vector2(30, 185), "Damage", HORIZONTAL_ALIGNMENT_LEFT, -1, 22, Color(0.3, 0.3, 0.36))
			draw_string(D.font(), rect.position + Vector2(190, 185), "%d%%" % int(c.percent), HORIZONTAL_ALIGNMENT_LEFT, -1, 30, D.INK)
			if winner:
				draw_string(D.font(), rect.position + Vector2(30, 232), "WINNER", HORIZONTAL_ALIGNMENT_LEFT, -1, 30, Color(0.85, 0.55, 0.0))
		if r.note != "":
			D.draw_text_centered(self, r.note, Rect2(Vector2(0, 630), Vector2(size.x, 50)), 30, Color(1, 1, 1))


func _init() -> void:
	layer = 10


## `cards`: per player {name, stocks, percent, winner}. `choices`: [[label, action], ...] (the first is selected).
func build(heading: String, cards: Array, choices: Array) -> void:
	title = heading
	card_data = cards
	board = Board.new()
	board.owner_screen = self
	add_child(board)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 16)
	row.position = Vector2(150, 540)
	add_child(row)
	for c in choices:
		var b := UI.Btn.new(c[0], Vector2(340, 70))
		b.font_size = 32
		var action: String = c[1]
		b.activated.connect(func(): chosen.emit(action))
		row.add_child(b)
		buttons.append(b)
		actions.append(action)
	_select(0)


func set_note(text: String) -> void:
	note = text
	board.queue_redraw()


func _select(i: int) -> void:
	index = posmod(i, buttons.size())
	for n in buttons.size():
		buttons[n].selected = n == index


func handle_key(event: InputEventKey) -> bool:
	match event.keycode:
		KEY_LEFT, KEY_A:
			_select(index - 1)
		KEY_RIGHT, KEY_D:
			_select(index + 1)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			chosen.emit(actions[index])
		_:
			return false
	return true
