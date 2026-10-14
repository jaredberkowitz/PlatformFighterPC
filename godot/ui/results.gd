extends CanvasLayer
## The results screen shown when a match ends, in the menus' painted style over the softened stage: the winner on a painted stage of its
## own, doing its victory pose, the heading on a brush stroke, a card for every fighter in finishing order
## with how their match went (knock-outs, falls, damage dealt and taken, stocks left), and what to do next.
## Left and right choose, Enter (or J, Space) picks, the mouse works too. It reports the choice through `chosen`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Preview := preload("res://ui/preview.gd")
const Art := preload("res://ui/menu_art.gd")

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
	## The results' painted layer: a soft veil over the (blurred) stage, the heading on an ink brush stroke with the winner's colour under
	## it, a card for each fighter in finishing order, and the note. Drawn when something changes, not every frame.
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var r = owner_screen
		var w := size.x
		var h := size.y
		var tint: Color = PLAYER_COLORS[r.winner % 4] if r.winner >= 0 else Color(0.55, 0.55, 0.65)
		draw_rect(Rect2(Vector2.ZERO, size), Color(Art.INK, 0.42))
		# The heading: the winner's colour as a brush stroke, the title on ink over it.
		Art.brush_band(self, Rect2(Vector2(46, 92), Vector2(560, 34)), tint, 81, 14.0, false, 0.8)
		var banner := Rect2(Vector2(24, 28), Vector2(600, 82))
		Art.brush_band(self, banner, Art.INK, 83, 22.0, true, 1.0)
		Art.title_centered(self, r.title.to_upper(), banner.grow_individual(-12, 0, -26, 0), 56, Color.WHITE, 0.0)
		# A card for each fighter, in finishing order.
		var f := D.font()
		var big := D.display_font()
		var count: int = r.order.size()
		var top := 40.0
		var card_h := minf(120.0, (h - 230.0) / maxf(1.0, count))
		for place in count:
			var i: int = r.order[place]
			var c: Dictionary = r.card_data[i]
			var first: bool = c.winner
			var rect := Rect2(Vector2(w * 0.5 + 10.0 - place * 8.0, top + place * (card_h + 14.0)), Vector2(w * 0.5 - 40.0, card_h - 6.0))
			# (Cards with a body, as the main menu's tiles: the winner's deeper, and rimmed in gold.)
			Art.card(self, rect, Art.CREAM if first else Art.CREAM.darkened(0.12), 90 + i, -1.0, -1.0,
				Art.SUN if first else Color(0, 0, 0, 0), 0.0, Color(0, 0, 0, 0), 7.0 if first else 4.0)
			# The place on a brush tab in the player's colour.
			var tab := Rect2(rect.position + Vector2(-14, 10), Vector2(92, rect.size.y - 30))
			Art.brush_band(self, tab, PLAYER_COLORS[i % 4], 100 + i, 10.0, true, 0.4)
			Art.title_centered(self, str(place + 1), tab.grow_individual(0, 0, -10, 0), int(card_h * 0.55), Color.WHITE, 0.12)
			draw_string(big, rect.position + Vector2(100, 38), "P%d  %s" % [i + 1, str(c.name).left(18).to_upper()], HORIZONTAL_ALIGNMENT_LEFT, -1, 26, Art.INK)
			var stats := [["KOs", str(c.get("kos", 0))], ["Falls", str(c.get("falls", 0))], ["Dealt", "%d%%" % int(c.get("dealt", 0.0))],
				["Taken", "%d%%" % int(c.get("taken", 0.0))], ["Stocks", str(c.stocks)]]
			var sx := rect.position.x + 100.0
			for st in stats:
				draw_string(f, Vector2(sx, rect.position.y + card_h - 42.0), st[0], HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(Art.INK, 0.6))
				draw_string(big, Vector2(sx, rect.position.y + card_h - 16.0), st[1], HORIZONTAL_ALIGNMENT_LEFT, -1, 28, Art.INK)
				sx += (rect.size.x - 124.0) / stats.size()
		if r.note != "":
			var band := Rect2(Vector2(w * 0.47, h - 152), Vector2(w * 0.5, 38))
			draw_rect(band, Color(Art.INK, 0.78))
			D.draw_text_centered(self, r.note, band, 20, Art.SUN)


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
	# The winner on a painted stage of its own (a warm one), standing on its platform.
	if winner >= 0 and cards[winner].has("look"):
		var stage := UI.Stage.new(Vector2(560, 470), 0.6, 13 + winner)
		# (Its frame rimmed in the winner's colour.)
		stage.rim = Color(PLAYER_COLORS[winner % 4], 0.9)
		stage.position = Vector2(40, 140)
		add_child(stage)
		preview = Preview.new(Vector2i(530, 438), false, 9.4)
		stage.add_fighter(preview)
		# The stage's framing scaled up about the feet (so they stay on the painted platform): room over the head for the cheering hop
		# (a sword held straight up may still break the top edge, as a hero shot does).
		preview.body_from *= RESULTS_FRAME
		preview.body_at *= RESULTS_FRAME
		preview.facing_bias = -0.5
		preview.set_loadout(cards[winner].look)
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


const RESULTS_FRAME := 1.15


func _process(delta: float) -> void:
	t += delta
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
