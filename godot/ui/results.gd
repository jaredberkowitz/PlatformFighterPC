extends CanvasLayer
## The results screen shown when a match ends: the winner standing large in their colour, a card for every fighter in finishing order
## with how their match went (knock-outs, falls, damage dealt and taken, stocks left), and what to do next.
## Left and right choose, Enter (or J, Space) picks, the mouse works too. It reports the choice through `chosen`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Preview := preload("res://ui/preview.gd")

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
var winner := -1
var order: Array = []   # player indices, first place first
var preview: Control
var t := 0.0


class Board extends Control:
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var r = owner_screen
		var w := size.x
		var h := size.y
		var tint: Color = PLAYER_COLORS[r.winner % 4] if r.winner >= 0 else Color(0.5, 0.5, 0.6)
		# The winner's colour sweeping across a dark field, with light bands.
		var dark := Color(0.05, 0.06, 0.12, 0.96)
		draw_rect(Rect2(Vector2.ZERO, size), dark)
		draw_colored_polygon(PackedVector2Array([Vector2(0, 0), Vector2(w * 0.55, 0), Vector2(w * 0.4, h), Vector2(0, h)]), Color(tint.r, tint.g, tint.b, 0.85))
		for k in 6:
			var x := w * 0.05 + k * w * 0.08 + fmod(r.t * 30.0, w * 0.08)
			draw_line(Vector2(x, h), Vector2(x + h * 0.3, 0), Color(1, 1, 1, 0.07), 22.0)
		draw_colored_polygon(PackedVector2Array([Vector2(w * 0.55, 0), Vector2(w * 0.58, 0), Vector2(w * 0.43, h), Vector2(w * 0.4, h)]), D.GOLD)
		# The banner.
		var banner := Rect2(Vector2(36, 34), Vector2(640, 96))
		D.draw_slant(self, banner, D.INK, D.GOLD, 4.0, true)
		var f := D.font()
		var fs := 64
		while fs > 30 and f.get_string_size(r.title, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > banner.size.x - 60.0:
			fs -= 2
		D.draw_text_centered(self, r.title, banner, fs, Color(1, 1, 1))
		# A card for each fighter, in finishing order.
		var count: int = r.order.size()
		var top := 70.0
		var card_h := minf(118.0, (h - 220.0) / maxf(1.0, count))
		for place in count:
			var i: int = r.order[place]
			var c: Dictionary = r.card_data[i]
			var rect := Rect2(Vector2(w * 0.5 + 30.0 - place * 6.0, top + place * (card_h + 12.0)), Vector2(w * 0.5 - 60.0, card_h))
			var first: bool = c.winner
			D.draw_slant(self, rect, Color(0.95, 0.94, 0.9) if first else Color(0.78, 0.79, 0.84), D.GOLD if first else D.INK, 5.0 if first else 3.0, true, 0.12)
			# Place number and player colour.
			var num := Rect2(rect.position + Vector2(8, 8), Vector2(74, rect.size.y - 16))
			D.draw_slant(self, num, PLAYER_COLORS[i % 4], D.INK, 3.0, false, 0.12)
			D.draw_text_centered(self, str(place + 1), num, int(card_h * 0.5), Color(1, 1, 1))
			draw_string(f, rect.position + Vector2(98, 34), "P%d  %s" % [i + 1, str(c.name).left(18)], HORIZONTAL_ALIGNMENT_LEFT, -1, 26, D.INK)
			var stats := [["KOs", str(c.get("kos", 0))], ["Falls", str(c.get("falls", 0))], ["Dealt", "%d%%" % int(c.get("dealt", 0.0))],
				["Taken", "%d%%" % int(c.get("taken", 0.0))], ["Stocks", str(c.stocks)]]
			var sx := rect.position.x + 98.0
			for s in stats:
				draw_string(f, Vector2(sx, rect.position.y + card_h - 40.0), s[0], HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.3, 0.3, 0.36))
				draw_string(f, Vector2(sx, rect.position.y + card_h - 14.0), s[1], HORIZONTAL_ALIGNMENT_LEFT, -1, 26, D.INK)
				sx += (rect.size.x - 120.0) / stats.size()
		if r.note != "":
			D.draw_text_centered(self, r.note, Rect2(Vector2(w * 0.45, h - 150), Vector2(w * 0.55, 40)), 24, Color(1, 1, 1))


func _init() -> void:
	layer = 10


## `cards`: per player {name, stocks, percent, winner, and optionally kos, falls, dealt, taken, look}. `choices`: [[label, action], ...]
## (the first is selected).
func build(heading: String, cards: Array, choices: Array) -> void:
	title = heading
	card_data = cards
	winner = -1
	for i in cards.size():
		if cards[i].winner:
			winner = i
	# Finishing order: the winner, then by stocks left, then by fewer falls, then by less damage.
	order = range(cards.size())
	order.sort_custom(func(a, b):
		var ca: Dictionary = cards[a]
		var cb: Dictionary = cards[b]
		if ca.winner != cb.winner:
			return ca.winner
		if ca.stocks != cb.stocks:
			return ca.stocks > cb.stocks
		if ca.get("falls", 0) != cb.get("falls", 0):
			return ca.get("falls", 0) < cb.get("falls", 0)
		return ca.percent < cb.percent)
	board = Board.new()
	board.owner_screen = self
	add_child(board)
	# The winner, standing large on the coloured side.
	if winner >= 0 and cards[winner].has("look"):
		preview = Preview.new(Vector2i(460, 560), false, 9.4)
		preview.position = Vector2(80, 160)
		preview.spin = false
		preview.facing_bias = -0.5
		preview.set_loadout(cards[winner].look)
		add_child(preview)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 16)
	row.position = Vector2(560, 610)
	add_child(row)
	for c in choices:
		var b := UI.Btn.new(c[0], Vector2(230, 64))
		b.font_size = 26
		var action: String = c[1]
		b.activated.connect(func(): chosen.emit(action))
		row.add_child(b)
		buttons.append(b)
		actions.append(action)
	_select(0)
	add_child(UI.Reveal.new())


func _process(delta: float) -> void:
	t += delta
	if board != null:
		board.queue_redraw()
	# The winner's victory pose (a cheering hop first, then their own pose).
	if preview != null and preview.view != null:
		var cls: int = card_data[winner].get("class", 0) if winner >= 0 else 0
		preview.view.play_victory(cls, delta, t < 1.0)


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
