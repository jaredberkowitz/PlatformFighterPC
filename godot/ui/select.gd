extends Control
## Character select: a grid of round portraits (the two built-in fighters, every fighter saved from the creator, a
## random pick and a "new fighter" door), a big panel for each player, and a start button once both have locked in.
##
## Player 1: W A S D move, J locks in, K takes it back.   Player 2: arrow keys move, Enter locks in, Backspace takes it back.
## Mouse: left click picks for player 1, right click for player 2.   E edits the saved fighter player 1 is on.
## When both are locked in, Space (or Enter, or the Start button) starts the match. Esc goes back to the menu.

const UI := preload("res://ui/ui_kit.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

const COLUMNS := 5
const TOKEN := 92
const P_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95)]
const P_DISC := [Color(0.98, 0.72, 0.72), Color(0.74, 0.8, 0.98)]

var entries: Array = []     # every roster entry, then two specials
var specials := ["random", "new"]
var tokens: Array = []
var cursor := [0, 1]
var locked := [false, false]
var picked: Array = [{}, {}]   # the entry each player ends up with (random is resolved when locking)
var panels: Array = []
var status: Control
var start_button: Control
var editor: RefCounted
var scroll: ScrollContainer
var rng := RandomNumberGenerator.new()
var ranked := false
var rules_button: Control
var stocks_button: Control
var time_button: Control


class PlayerPanel extends Control:
	## One player's big panel: a coloured disc, the fighter on it, and a name tag with a few stat pips below.
	var player := 0
	var disc_color := Color.WHITE
	var preview: Control
	var name_tag: Control
	var info: Control
	var player_tag: Control
	var ready_tag: Control
	var empty := true

	func _init(p: int, disc: Color, flag: Color) -> void:
		player = p
		disc_color = disc
		custom_minimum_size = Vector2(430, 300)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		preview = Preview.new(Vector2i(250, 290), false, 10.5)
		preview.position = Vector2(60, -33)
		add_child(preview)
		player_tag = UI.Tag.new("Player %d" % (p + 1), Vector2(220, 50))
		player_tag.fill = flag
		player_tag.ink = Color(1, 1, 1)
		player_tag.edge = UI.INK
		player_tag.font_size = 28
		player_tag.position = Vector2(150, 232)
		add_child(player_tag)
		name_tag = UI.Tag.new("", Vector2(300, 46))
		name_tag.fill = UI.CREAM
		name_tag.font_size = 30
		name_tag.position = Vector2(110, 186)
		add_child(name_tag)
		ready_tag = UI.Tag.new("READY!", Vector2(150, 44))
		ready_tag.fill = UI.GOLD
		ready_tag.edge = UI.INK
		ready_tag.font_size = 28
		ready_tag.position = Vector2(300, 8)
		ready_tag.visible = false
		add_child(ready_tag)

	func _draw() -> void:
		var c := Vector2(185, 112)
		draw_circle(c + Vector2(4, 6), 108, Color(0, 0, 0, 0.25))
		draw_circle(c, 108, disc_color)
		draw_arc(c, 108, 0, TAU, 56, UI.INK, 5.0, true)
		# A few light stripes across the disc, like the reference's patterned portraits.
		for i in 5:
			var x := c.x - 90.0 + i * 45.0
			draw_line(Vector2(x, c.y - 50), Vector2(x + 28, c.y + 70), Color(1, 1, 1, 0.18), 10.0)
		if empty:
			UI.D.draw_text_centered(self, "?", Rect2(c - Vector2(60, 60), Vector2(120, 120)), 110, Color(1, 1, 1, 0.55))

	func show_entry(e: Dictionary, size_percent: float, is_ready: bool) -> void:
		empty = e.is_empty()
		preview.visible = not empty
		if not empty:
			preview.set_loadout(e.look)
			preview.set_size_percent(size_percent)
			name_tag.set_text(e.name)
		else:
			name_tag.set_text("Choose a fighter")
		ready_tag.visible = is_ready
		queue_redraw()


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	rng.randomize()
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	add_child(UI.Backdrop.new(true))

	var mode := UI.Tag.new("Versus", Vector2(300, 48))
	mode.fill = UI.GOLD
	mode.edge = UI.INK
	mode.font_size = 28
	mode.position = Vector2(16, 12)
	add_child(mode)
	var ribbon := UI.Tag.new("Select your fighter", Vector2(560, 64))
	ribbon.fill = UI.INK
	ribbon.ink = Color(1, 1, 1)
	ribbon.font_size = 40
	ribbon.position = Vector2(420, 4)
	add_child(ribbon)

	for p in 2:
		var panel := PlayerPanel.new(p, P_DISC[p], P_COLORS[p])
		panel.position = Vector2(20, 76 + p * 296)
		add_child(panel)
		panels.append(panel)

	entries = Roster.all()
	scroll = ScrollContainer.new()
	scroll.position = Vector2(500, 90)
	scroll.size = Vector2(COLUMNS * (TOKEN + 14) + 20, 480)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var grid := GridContainer.new()
	grid.columns = COLUMNS
	grid.add_theme_constant_override("h_separation", 10)
	grid.add_theme_constant_override("v_separation", 10)
	scroll.add_child(grid)
	for i in entries.size() + specials.size():
		var token := Control.new()
		token.custom_minimum_size = Vector2(TOKEN + 8, TOKEN + 8)
		var portrait := UI.Portrait.new(TOKEN)
		portrait.position = Vector2(4, 4)
		if i < entries.size():
			portrait.set_look(entries[i].look)
			var r: Dictionary = editor.recipe_readout(entries[i]["class"], entries[i].size, entries[i].speed, entries[i].jump, entries[i].weight)
			portrait.scale_hint = clampf(float(r.size_percent) / 100.0, 0.8, 1.2)
		else:
			portrait.glyph = "?" if specials[i - entries.size()] == "random" else "+"
		var index := i
		portrait.clicked.connect(func(button): _clicked(index, button))
		token.add_child(portrait)
		var tags := []
		for p in 2:
			var flag := UI.Tag.new("P%d" % (p + 1), Vector2(46, 28))
			flag.fill = P_COLORS[p]
			flag.ink = Color(1, 1, 1)
			flag.edge = UI.INK
			flag.font_size = 16
			flag.position = Vector2(-4 + p * 54, -6)
			flag.visible = false
			token.add_child(flag)
			tags.append(flag)
		if i < entries.size() and not Roster.ranked_legal(entries[i]):
			var mark := UI.Tag.new("CASUAL", Vector2(76, 24))
			mark.fill = UI.SKY
			mark.edge = UI.INK
			mark.font_size = 14
			mark.position = Vector2(10, TOKEN - 14)
			token.add_child(mark)
		grid.add_child(token)
		tokens.append({"node": token, "portrait": portrait, "flags": tags})

	var hint := UI.Tag.new("P1: WASD, J lock, K back        P2: arrows, Enter lock, Backspace back        E edit        T stocks   Y time        Esc menu", Vector2(1000, 36))
	hint.fill = Color(1, 1, 1, 0.5)
	hint.ink = UI.INK
	hint.font_size = 18
	hint.shadow = false
	hint.position = Vector2(150, 678)
	add_child(hint)

	status = UI.Tag.new("", Vector2(560, 44))
	status.fill = Color(1, 1, 1, 0)
	status.ink = UI.INK
	status.shadow = false
	status.font_size = 24
	status.position = Vector2(500, 580)
	add_child(status)

	rules_button = UI.Btn.new("Rules: Casual", Vector2(280, 52))
	rules_button.font_size = 24
	rules_button.position = Vector2(990, 8)
	rules_button.activated.connect(_toggle_ranked)
	add_child(rules_button)

	Roster.load_match_rules()
	stocks_button = UI.Btn.new("", Vector2(260, 46))
	stocks_button.font_size = 22
	stocks_button.position = Vector2(500, 622)
	stocks_button.activated.connect(func(): _cycle_rules(true))
	add_child(stocks_button)
	time_button = UI.Btn.new("", Vector2(260, 46))
	time_button.font_size = 22
	time_button.position = Vector2(780, 622)
	time_button.activated.connect(func(): _cycle_rules(false))
	add_child(time_button)
	_show_match_rules()

	start_button = UI.Btn.new("Start Battle!", Vector2(330, 74))
	start_button.position = Vector2(930, 590)
	start_button.font_size = 38
	start_button.activated.connect(_start)
	start_button.visible = false
	add_child(start_button)

	# Where the cursors begin: on the fighter just made in the creator if there is one.
	var pre: String = Roster.session.get("preselect", "")
	cursor = [0, mini(1, entries.size() - 1)]
	if pre != "":
		for i in entries.size():
			if entries[i].slug == pre:
				cursor[0] = i
				_lock(0)
	_refresh()
	PadNav.attach(self)
	Shot.attach(self)


## Controller 1 plays the player 1 keys on this screen, controller 2 the player 2 keys.
func pad_scheme(pad: int) -> Dictionary:
	if pad == 0:
		return {"up": KEY_W, "down": KEY_S, "left": KEY_A, "right": KEY_D, "confirm": KEY_J, "back": KEY_K, "start": KEY_SPACE, "menu": KEY_ESCAPE}
	return {"up": KEY_UP, "down": KEY_DOWN, "left": KEY_LEFT, "right": KEY_RIGHT, "confirm": KEY_ENTER, "back": KEY_BACKSPACE, "start": KEY_SPACE, "menu": KEY_ESCAPE}


# ---- Input -----------------------------------------------------------------------------------------------------------------

func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_A:
			_move(0, -1, 0)
		KEY_D:
			_move(0, 1, 0)
		KEY_W:
			_move(0, 0, -1)
		KEY_S:
			_move(0, 0, 1)
		KEY_J:
			_lock(0)
		KEY_K:
			_unlock(0)
		KEY_LEFT:
			_move(1, -1, 0)
		KEY_RIGHT:
			_move(1, 1, 0)
		KEY_UP:
			_move(1, 0, -1)
		KEY_DOWN:
			_move(1, 0, 1)
		KEY_ENTER, KEY_KP_ENTER:
			if locked[0] and locked[1]:
				_start()
			else:
				_lock(1)
		KEY_BACKSPACE, KEY_DELETE:
			_unlock(1)
		KEY_SPACE:
			if locked[0] and locked[1]:
				_start()
		KEY_E:
			_edit()
		KEY_R:
			_toggle_ranked()
		KEY_T:
			_cycle_rules(true)
		KEY_Y:
			_cycle_rules(false)
		KEY_ESCAPE:
			get_tree().change_scene_to_file("res://menu.tscn")


func _clicked(index: int, button: int) -> void:
	var p := 1 if button == MOUSE_BUTTON_RIGHT else 0
	if button != MOUSE_BUTTON_LEFT and button != MOUSE_BUTTON_RIGHT:
		return
	cursor[p] = index
	locked[p] = false
	_lock(p)


func _move(p: int, dx: int, dy: int) -> void:
	if locked[p]:
		return
	var n := tokens.size()
	var i: int = cursor[p]
	i = clampi(i + dx + dy * COLUMNS, 0, n - 1)
	cursor[p] = i
	_refresh()
	scroll.ensure_control_visible(tokens[i].node)


func _cycle_rules(stocks: bool) -> void:
	if stocks:
		var i: int = Roster.STOCK_CHOICES.find(Roster.match_stocks)
		Roster.match_stocks = Roster.STOCK_CHOICES[(i + 1) % Roster.STOCK_CHOICES.size()]
	else:
		var j: int = Roster.TIME_CHOICES.find(Roster.match_time)
		Roster.match_time = Roster.TIME_CHOICES[(j + 1) % Roster.TIME_CHOICES.size()]
	Roster.save_match_rules()
	_show_match_rules()


func _show_match_rules() -> void:
	stocks_button.text = "Stocks: " + Roster.stocks_text(Roster.match_stocks)
	time_button.text = "Time: " + Roster.time_text(Roster.match_time)
	stocks_button.queue_redraw()
	time_button.queue_redraw()


func _toggle_ranked() -> void:
	ranked = not ranked
	rules_button.text = "Rules: Ranked" if ranked else "Rules: Casual"
	rules_button.queue_redraw()
	# Anyone already locked in with a fighter the new rules forbid is sent back to choose again.
	for p in 2:
		if locked[p] and ranked and not Roster.ranked_legal(picked[p]):
			_unlock(p)
	status.set_text("Ranked rules: fighters over %d points are not allowed." % Roster.budget() if ranked else "Casual rules: anything goes.")
	_refresh()
	if ranked:
		status.set_text("Ranked rules: fighters over %d points are not allowed." % Roster.budget())


func _lock(p: int) -> void:
	var i: int = cursor[p]
	if i >= entries.size():
		if specials[i - entries.size()] == "new":
			Roster.edit_slug = ""
			get_tree().change_scene_to_file("res://creator.tscn")
			return
		var pool := entries.filter(func(e): return not ranked or Roster.ranked_legal(e))
		picked[p] = pool[rng.randi_range(0, pool.size() - 1)]
	else:
		if ranked and not Roster.ranked_legal(entries[i]):
			status.set_text("%s is over the %d point budget: not allowed under ranked rules." % [entries[i].name, Roster.budget()])
			return
		picked[p] = entries[i]
	locked[p] = true
	_refresh()


func _unlock(p: int) -> void:
	locked[p] = false
	picked[p] = {}
	_refresh()


func _edit() -> void:
	var i: int = cursor[0]
	if i < entries.size() and not entries[i].get("builtin", false):
		Roster.edit_slug = entries[i].slug
		get_tree().change_scene_to_file("res://creator.tscn")
	else:
		status.set_text("Only fighters you made can be edited. Make a new one with the + circle.")


# ---- Showing it -----------------------------------------------------------------------------------------------------------------

func _refresh() -> void:
	for t in tokens.size():
		for p in 2:
			tokens[t].flags[p].visible = cursor[p] == t
		tokens[t].portrait.ring = UI.INK
		tokens[t].portrait.ring_width = 4.0
		for p in [1, 0]:
			if cursor[p] == t:
				tokens[t].portrait.ring = P_COLORS[p]
				tokens[t].portrait.ring_width = 9.0 if locked[p] else 6.0
		tokens[t].portrait.queue_redraw()
	for p in 2:
		var e: Dictionary = picked[p]
		var shown: Dictionary = e
		if not locked[p]:
			# Not locked in yet: show whatever the cursor is on.
			var i: int = cursor[p]
			shown = entries[i] if i < entries.size() else {}
		var size_percent := 100.0
		if not shown.is_empty():
			var r: Dictionary = editor.recipe_readout(shown["class"], shown.size, shown.speed, shown.jump, shown.weight)
			size_percent = float(r.size_percent)
		panels[p].show_entry(shown, size_percent, locked[p])
	var both: bool = locked[0] and locked[1]
	start_button.visible = both
	status.set_text("Press Space to start!" if both else "")


func _start() -> void:
	if not (locked[0] and locked[1]):
		return
	var built: Dictionary = Roster.build_content([picked[0], picked[1]])
	if built.error != "":
		status.set_text(built.error)
		return
	Roster.save_last(picked[0].slug)
	Roster.session = {
		"content_text": built.text, "chars": built.chars, "entries": [picked[0], picked[1]], "ranked": ranked, "from_menu": true,
		"stocks": Roster.match_stocks, "time": Roster.match_time,
	}
	get_tree().change_scene_to_file("res://main.tscn")
