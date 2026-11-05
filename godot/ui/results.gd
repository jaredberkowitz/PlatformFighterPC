extends CanvasLayer
## The results screen shown when a match ends, over the softened stage. Laid out as a podium (the owner's pick of the results mocks,
## 2026-11-05, "Podium"; docs/art/ui/UI_ANALYSIS.md): the winner tall at the left in its player's colour, cut out of paper and doing its
## victory pose, its fighter's name on a plate and a gold WINNER! card; at the right the heading large, one score sheet with a column
## for every fighter in finishing order (knock-outs, falls, damage dealt and taken, stocks left), the runner-up's snapshot at the
## sheet's corner in a one-on-one, and what to do next under it. Left and right choose, Enter (or J, Space) picks, the mouse works too.
## It reports the choice through `chosen`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Preview := preload("res://ui/preview.gd")
const Art := preload("res://ui/menu_art.gd")
const Roster := preload("res://scripts/roster.gd")

signal chosen(action: String)

const PLAYER_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
## The winner's panel, the score sheet, the runner-up's snapshot and the row of choices.
const PANEL := Rect2(28, 86, 450, 540)
const SHEET := Rect2(520, 184, 730, 330)
const SNAP := Rect2(1104, 384, 150, 150)
const CHOICES_AT := Vector2(520, 586)
## The sheet's rows: what each says, and the card's key.
const ROWS := [["KOs", "kos"], ["FALLS", "falls"], ["DAMAGE DEALT", "dealt"], ["DAMAGE TAKEN", "taken"], ["STOCKS LEFT", "stocks"]]
## The panel camera (as fighter select's face-off: the fighter fills the panel's height, its feet on the floor of the light).
const HERO_CAM := Vector3(0, 1.3, 7.4)
const HERO_AIM_Y := 1.2
## How far the winner turns toward the sheet, and the runner-up toward the winner.
const WINNER_TURN := 0.45
const RUNNER_UP_TURN := -0.5
## How far the runner-up's snapshot is dimmed (it lost; the winner's panel is the bright one).
const RUNNER_UP_DIM := Color(0.86, 0.86, 0.9)

var root: Control
var buttons: Array = []
var actions: Array = []
var index := 0
var card_data: Array = []
var title := ""
var board: Control
var over: Control
var note := ""
var winner := -1
var order: Array = []   # player indices, first place first
var preview: Control
var runner_up: Control
var t := 0.0


class Board extends Control:
	## Under everything: a paper veil over the blurred stage, the winner's panel (a cream mount and its sheet of colour come from the
	## spotlight child), the heading, the score sheet and the note. Drawn when something changes, not every frame.
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	static func _ease(k: float) -> float:
		return 1.0 - pow(1.0 - k, 3.0)

	func _draw() -> void:
		var r = owner_screen
		var w := size.x
		var h := size.y
		# A light paper veil, so the stage behind stays a place without competing.
		var top := Color(D.SHEET, 0.8)
		var low := Color(D.SHEET.darkened(0.06), 0.9)
		draw_polygon(PackedVector2Array([Vector2.ZERO, Vector2(w, 0), Vector2(w, h), Vector2(0, h)]), PackedColorArray([top, top, low, low]))
		var t: float = r.t
		var land := _ease(clampf(t / 0.35, 0.0, 1.0))
		# The winner's panel: a cream mount on its paper shadow (its colour is the spotlight over it).
		var mount := PANEL.grow(8.0)
		var pts := PackedVector2Array([mount.position, Vector2(mount.end.x, mount.position.y), mount.end, Vector2(mount.position.x, mount.end.y)])
		Art._paper_shadow(self, pts, 1.0, 4.0)
		draw_colored_polygon(pts, Art.PAPER_EDGE)
		# The heading, large, with a gold shadow; it slides in.
		var slide := (1.0 - land) * 40.0
		var at := Vector2(SHEET.position.x + slide, 152)
		var size_px := 72
		var f := D.display_font()
		while size_px > 40 and f.get_string_size(r.title.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, size_px).x > SHEET.size.x:
			size_px -= 4
		Art.title(self, r.title.to_upper(), at + Vector2(3, 4), size_px, Color(Art.SUN, 0.95 * land), 0.0)
		Art.title(self, r.title.to_upper(), at, size_px, Color(Art.INK, land), 0.0)
		_draw_sheet(r, land)
		if r.note != "":
			var band := Rect2(Vector2(SHEET.position.x, SHEET.end.y + 16.0), Vector2(560, 36))
			draw_rect(band, D.PAPER_INDIGO)
			D.draw_text_centered(self, r.note, band, UI.TYPE_BODY, Art.SUN)

	## One cardstock sheet: THE MATCH, a chip per fighter in its player's colour (finishing order, left to right), then a line for each
	## row with every fighter's number; the winner's numbers in ink, the others quieter.
	func _draw_sheet(r, land: float) -> void:
		draw_set_transform(SHEET.get_center(), deg_to_rad(0.4), Vector2.ONE)
		var o := -SHEET.size * 0.5
		Art.card(self, Rect2(o, SHEET.size), D.SHEET, 57, 16.0)
		var lf := D.label_font()
		var big := D.display_font()
		draw_string(lf, o + Vector2(28, 38), "THE MATCH", HORIZONTAL_ALIGNMENT_LEFT, -1, UI.TYPE_CAPTION, D.LABEL)
		var n: int = r.order.size()
		var label_w := 290.0
		var col_w := minf(170.0, (SHEET.size.x - label_w - 24.0) / maxf(1.0, n))
		var x0 := o.x + label_w
		for place in n:
			var i: int = r.order[place]
			var c: Dictionary = r.card_data[i]
			var chip := Rect2(Vector2(x0 + place * col_w + 6.0, o.y + 14.0), Vector2(col_w - 12.0, 40.0))
			draw_rect(chip, PLAYER_COLORS[i % 4].darkened(0.12))
			var words := "P%d · %s" % [i + 1, str(c.get("fighter", c.name)).to_upper()]
			var fs := 22
			while fs > 13 and big.get_string_size(words, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > chip.size.x - 12.0:
				fs -= 1
			D.draw_text_centered(self, words, chip, fs, Color.WHITE)
		var y := o.y + 72.0
		for row in ROWS:
			draw_line(Vector2(o.x + 24.0, y - 6.0), Vector2(o.x + SHEET.size.x - 24.0, y - 6.0), Color(D.LABEL, 0.25), 1.0)
			draw_string(lf, Vector2(o.x + 28.0, y + 24.0), row[0], HORIZONTAL_ALIGNMENT_LEFT, -1, UI.TYPE_CAPTION, D.LABEL)
			for place in n:
				var i: int = r.order[place]
				var c: Dictionary = r.card_data[i]
				var v = c.get(row[1], 0)
				var text := ("%d%%" % int(v)) if row[1] in ["dealt", "taken"] else str(int(v))
				var tw := big.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, 30).x
				var ink: Color = Art.INK if c.winner else Color(D.LABEL.darkened(0.2), 1.0)
				draw_string(big, Vector2(x0 + place * col_w + (col_w - tw) * 0.5, y + 30.0), text, HORIZONTAL_ALIGNMENT_LEFT, -1, 30,
					Color(ink, land))
			y += 50.0
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)


class Over extends Control:
	## Above the fighters: the winner's tab, its name plate and the WINNER! card (DRAW for a draw), each laid a little turned.
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var r = owner_screen
		if r.order.is_empty():
			return
		var i: int = r.order[0]
		var c: Dictionary = r.card_data[i]
		var colour: Color = PLAYER_COLORS[i % 4]
		var tab := Rect2(PANEL.position + Vector2(-6, -20), Vector2(96, 48))
		draw_set_transform(tab.get_center(), deg_to_rad(-3.0), Vector2.ONE)
		Art.card(self, Rect2(-tab.size * 0.5, tab.size), colour, 51 + i, 8.0)
		Art.title_centered(self, "P%d" % (i + 1), Rect2(-tab.size * 0.5, tab.size), 32, Color.WHITE, 0.0)
		var plate := Rect2(Vector2(PANEL.position.x + 16, PANEL.end.y - 70), Vector2(330, 82))
		draw_set_transform(plate.get_center(), deg_to_rad(-1.2), Vector2.ONE)
		var local := Rect2(-plate.size * 0.5, plate.size)
		Art.card(self, local, D.SHEET.lightened(0.3), 31 + i, 10.0)
		var cls := int(c.get("class", -1))
		if cls >= 0 and cls < Roster.CLASS_NAMES.size():
			draw_string(D.label_font(), local.position + Vector2(20, 22), Roster.CLASS_NAMES[cls].to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1,
				UI.TYPE_CAPTION, colour.darkened(0.35))
		var shown := str(c.get("fighter", c.name)).to_upper()
		var fs := 52
		var nf := D.display_font()
		while fs > 24 and nf.get_string_size(shown, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > local.size.x - 36.0:
			fs -= 2
		draw_string(nf, Vector2(local.position.x + 18, local.end.y - 14), shown, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Art.INK)
		var badge := Rect2(Vector2(PANEL.end.x - 186, PANEL.end.y - 112), Vector2(176, 54))
		draw_set_transform(badge.get_center(), deg_to_rad(4.0), Vector2.ONE)
		Art.card(self, Rect2(-badge.size * 0.5, badge.size), Art.SUN, 71 + i, 8.0)
		Art.title_centered(self, "WINNER!" if c.winner else "DRAW", Rect2(-badge.size * 0.5, badge.size), 36, Art.INK, 0.0)
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)


class Snap extends Control:
	## The runner-up's snapshot: a cream mount, its sheet of colour, a little turned (the fighter is a child, clipped to it).
	var colour := Color.WHITE

	func _draw() -> void:
		draw_rect(Rect2(Vector2(-6, -6), size + Vector2(12, 12)), Art.PAPER_EDGE)
		draw_rect(Rect2(Vector2.ZERO, size), colour)


func _init() -> void:
	layer = 10


## `cards`: per player {name, stocks, percent, winner, and optionally fighter, kos, falls, dealt, taken, look, class}. `choices`:
## [[label, action], ...] (the first is selected).
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
	add_child(UI.TopBar.new("RESULTS"))
	# The first place (the winner, or the leader of a draw) in its panel.
	if not order.is_empty() and cards[order[0]].has("look"):
		_panel(cards[order[0]], order[0])
	over = Over.new()
	over.owner_screen = self
	add_child(over)
	_runner_up(cards)
	# The choices under the sheet.
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	row.position = CHOICES_AT
	add_child(row)
	var button_w := minf(230.0, (SHEET.size.x - 14.0 * (choices.size() - 1)) / maxf(1.0, choices.size()))
	for c in choices:
		var b := UI.Btn.new(c[0], Vector2(button_w, 56))
		b.font_size = UI.TYPE_LABEL
		# The first choice (Rematch) is the primary action, in gold; the rest stay the quiet cream.
		if buttons.is_empty():
			b.fill = Art.SUN
		var action: String = c[1]
		b.activated.connect(func(): chosen.emit(action))
		row.add_child(b)
		buttons.append(b)
		actions.append(action)
	var keys := UI.KeyBar.new([["A D", "Choose"], ["Enter", "Pick"]])
	keys.position = Vector2(CHOICES_AT.x, 660)
	keys.size = Vector2(SHEET.size.x, 30)
	add_child(keys)
	_select(0)
	add_child(UI.Reveal.new())


## The first place's panel: its player's colour as a plain pool of light (fighter select's look), the fighter filling the panel's height,
## cut out of paper and clipped only at the panel's top edge (open at the sides and the foot, so it breaks out of the frame).
func _panel(c: Dictionary, i: int) -> void:
	var mask := UI.Mask.new()
	mask.position = PANEL.position
	mask.size = PANEL.size
	mask.set_shape(PackedVector2Array([Vector2.ZERO, Vector2(PANEL.size.x, 0), PANEL.size, Vector2(0, PANEL.size.y)]))
	add_child(mask)
	var spot := UI.Spotlight.new()
	var pc: Color = PLAYER_COLORS[i % 4]
	spot.colour = Color.from_hsv(pc.h, 0.62, 0.72)
	spot.plain = true
	spot.feet = [[0.5, 0.8]]
	spot.seed = 13 + i
	spot.size = PANEL.size
	mask.add_child(spot)
	var clip := Control.new()
	clip.clip_contents = true
	clip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	clip.position = Vector2(PANEL.position.x - 300.0, PANEL.position.y)
	clip.size = Vector2(PANEL.size.x + 600.0, PANEL.size.y + 300.0)
	add_child(clip)
	var pw := PANEL.size.x * 1.2
	var ph := PANEL.size.y + 10.0
	preview = Preview.new(Vector2i(int(pw), int(ph)), false, HERO_CAM.z)
	preview.spin = false
	UI.die_cut(preview)
	preview.size = Vector2(pw, ph)
	preview.position = Vector2(300.0 + PANEL.size.x * 0.5 - pw * 0.5, -30.0)
	clip.add_child(preview)
	preview.camera.position = HERO_CAM
	preview.camera.look_at_from_position(HERO_CAM, Vector3(0, HERO_AIM_Y, 0))
	preview.facing_bias = WINNER_TURN
	preview.set_loadout(c.look)
	preview.set_weapon(int(c.get("class", -1)))


## In a one-on-one, the runner-up's snapshot at the sheet's corner, dimmed and in its own colour, turned toward the winner (with three
## or four players the sheet's columns tell the rest).
func _runner_up(cards: Array) -> void:
	if order.size() != 2:
		return
	var i: int = order[1]
	if not cards[i].has("look"):
		return
	var snap := Snap.new()
	snap.colour = Color.from_hsv((PLAYER_COLORS[i % 4] as Color).h, 0.55, 0.66)
	snap.position = SNAP.position
	snap.size = SNAP.size
	snap.pivot_offset = SNAP.size * 0.5
	snap.rotation = deg_to_rad(5.0)
	snap.clip_contents = true
	snap.modulate = RUNNER_UP_DIM
	add_child(snap)
	runner_up = Preview.new(Vector2i(220, 260), false, 5.2)
	runner_up.spin = false
	UI.die_cut(runner_up)
	runner_up.size = Vector2(220, 260)
	runner_up.position = Vector2(-35, -8)
	snap.add_child(runner_up)
	runner_up.camera.position = Vector3(0, 1.45, 5.2)
	runner_up.camera.look_at_from_position(runner_up.camera.position, Vector3(0, 1.25, 0))
	runner_up.facing_bias = RUNNER_UP_TURN
	runner_up.set_loadout(cards[i].look)
	runner_up.set_weapon(int(cards[i].get("class", -1)))


func _process(delta: float) -> void:
	t += delta
	if t < 1.4 and board != null:
		board.queue_redraw()
	# The winner's victory pose (a cheering hop first, then their own pose).
	if preview != null and preview.view != null and winner >= 0:
		var cls: int = card_data[winner].get("class", 0)
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
