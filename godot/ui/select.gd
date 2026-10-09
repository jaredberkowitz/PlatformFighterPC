extends Control
## Character select: a grid of roster cards (each with the fighter's own 3D portrait; the built-in fighters, every fighter saved from
## the creator, a random pick and a "new fighter" door), a big card for each player along the bottom, and a "ready to fight" band
## once everyone has locked in.
##
## Player 1: W A S D move, J locks in, K takes it back.   Player 2: arrow keys move, Enter locks in, Backspace takes it back.
## Mouse: left click picks for player 1, right click for player 2.   E edits the saved fighter player 1 is on.
## When everyone is locked in, Space (or Enter, or the band) starts the match. Esc goes back to the menu.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

const COLUMNS := 7
const TOKEN := 124
const P_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
const P_DISC := [Color(0.98, 0.72, 0.72), Color(0.74, 0.8, 0.98), Color(0.99, 0.9, 0.62), Color(0.76, 0.94, 0.8)]

var entries: Array = []     # every roster entry, then two specials
var specials := ["random", "new"]
var tokens: Array = []
var cursor := [0, 1, 0, 1]
var locked := [false, false, false, false]
var picked: Array = [{}, {}, {}, {}]   # the entry each player ends up with (random is resolved when locking)
## How many players are in this match (2 to 4). Players 3 and 4 play on controllers 3 and 4.
var count := 2
var players_button: Control
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
var stage_button: Control
var thumbs_frozen := false
var frames := 0


class Card extends Control:
	## A roster card: a slanted panel with the fighter's 3D portrait (or a "?" or "+" glyph) and its name; rings in the colours of the
	## players whose cursor is on it are drawn over the top by `Overlay`.
	var name_text := ""
	var glyph := ""
	var rings: Array = []   # [colour, locked] for each player on it
	var casual := false
	var hover := false
	signal clicked(button: int)

	func _init() -> void:
		custom_minimum_size = Vector2(TOKEN, TOKEN + 8)
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func():
			hover = true
			queue_redraw())
		mouse_exited.connect(func():
			hover = false
			queue_redraw())

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			clicked.emit(event.button_index)

	func _draw() -> void:
		var rect := Rect2(Vector2(4, 4), size - Vector2(8, 8))
		if not rings.is_empty() or hover:
			var c: Color = rings[0][0] if not rings.is_empty() else UI.GOLD
			UI.D.draw_glow(self, rect, c, 1.0, 0.12)
		var fill := Color(0.94, 0.92, 0.86) if glyph == "" else Color(0.3, 0.32, 0.42)
		UI.D.draw_slant(self, rect, fill, UI.INK, 3.0, true, 0.12)
		if glyph != "":
			UI.D.draw_text_centered(self, glyph, Rect2(rect.position, rect.size - Vector2(0, 20)), 58, UI.GOLD)

	## Drawn over the portrait: the name strip and the player rings.
	func paint_over(over: Control) -> void:
		var rect := Rect2(Vector2(4, 4), size - Vector2(8, 8))
		var strip := Rect2(Vector2(rect.position.x + 2, rect.end.y - 24), Vector2(rect.size.x - 6, 22))
		over.draw_rect(strip, Color(0.08, 0.09, 0.14, 0.85))
		if name_text != "":
			UI.D.draw_text_centered(over, name_text.left(14), strip, 17, Color(1, 1, 1))
		for k in rings.size():
			var r := rect.grow(-2.0 - k * 5.0)
			var pts := UI.D.skew_points(r, 0.12)
			pts.append(pts[0])
			over.draw_polyline(pts, rings[k][0], 7.0 if rings[k][1] else 4.0, true)
		if casual:
			var tag := Rect2(Vector2(rect.position.x + 8, rect.position.y + 4), Vector2(56, 17))
			over.draw_rect(tag, UI.SKY)
			UI.D.draw_text_centered(over, "CASUAL", tag, 12, UI.INK)


class Overlay extends Control:
	## Paints a card's top layer (above its 3D portrait).
	var card: Control

	func _init(c: Control) -> void:
		card = c
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		card.paint_over(self)


class PlayerPanel extends Control:
	## One player's card along the bottom: their colour, the fighter standing large, the fighter's name, and READY when locked in.
	var player := 0
	var disc_color := Color.WHITE
	var flag_color := Color.WHITE
	var preview: Control
	var name_text := ""
	var is_ready := false
	var empty := true
	var t := 0.0

	func _init(p: int, disc: Color, flag: Color) -> void:
		player = p
		disc_color = disc
		flag_color = flag
		custom_minimum_size = Vector2(300, 290)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		preview = Preview.new(Vector2i(260, 260), false, 6.8)
		add_child(preview)

	func lay_out(w: float) -> void:
		size = Vector2(w, 290)
		var pw := minf(w * 0.8, 300.0)
		preview.size = Vector2(pw, 260)
		preview.position = Vector2(w * 0.5 - pw * 0.5 + 10.0, -40)

	func _process(delta: float) -> void:
		t += delta
		if is_ready:
			queue_redraw()

	func _draw() -> void:
		var rect := Rect2(Vector2(6, 6), size - Vector2(12, 12))
		if is_ready:
			UI.D.draw_glow(self, rect, Color(1, 0.9, 0.5), 0.6 + 0.4 * sin(t * 4.0), 0.1)
		UI.D.draw_slant(self, rect, flag_color.darkened(0.15), UI.INK, 5.0, true, 0.1)
		# A lighter field behind the fighter, with diagonal light bands.
		var inner := rect.grow(-12.0)
		inner.size.y -= 58.0
		UI.D.draw_slant(self, inner, disc_color, Color(0, 0, 0, 0), 0.0, false, 0.1)
		for i in 6:
			var x := inner.position.x + 20.0 + i * inner.size.x / 6.0
			draw_line(Vector2(x, inner.end.y - 4.0), Vector2(x + 40.0, inner.position.y + 4.0), Color(1, 1, 1, 0.16), 12.0)
		if empty:
			UI.D.draw_text_centered(self, "?", inner, 110, Color(1, 1, 1, 0.6))
		var tag := Rect2(rect.position + Vector2(14, 12), Vector2(64, 30))
		UI.D.draw_slant(self, tag, UI.INK, Color(0, 0, 0, 0), 0.0, false)
		UI.D.draw_text_centered(self, "P%d" % (player + 1), tag, 22, flag_color.lightened(0.3))
		var plate := Rect2(Vector2(rect.position.x + 10, rect.end.y - 56), Vector2(rect.size.x - 20, 46))
		UI.D.draw_slant(self, plate, UI.INK, Color(0, 0, 0, 0), 0.0, false, 0.1)
		var f := UI.D.font()
		var shown := name_text if not empty else "Choose a fighter"
		var fs := 34
		while fs > 16 and f.get_string_size(shown, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > plate.size.x - 30.0:
			fs -= 2
		UI.D.draw_text_centered(self, shown, plate, fs, Color(1, 1, 1))
		if is_ready:
			var badge := Rect2(Vector2(rect.end.x - 150, rect.position.y + 10), Vector2(136, 40))
			UI.D.draw_slant(self, badge, UI.GOLD, UI.INK, 3.0, true)
			UI.D.draw_text_centered(self, "READY!", badge, 26, UI.INK)

	func show_entry(e: Dictionary, size_percent: float, ready: bool) -> void:
		empty = e.is_empty()
		preview.visible = not empty
		if not empty:
			preview.set_loadout(e.look)
			preview.set_size_percent(size_percent)
			name_text = e.name
		is_ready = ready
		queue_redraw()


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	rng.randomize()
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	add_child(UI.Backdrop.new(true))

	var mode := UI.Tag.new("Versus", Vector2(220, 48))
	mode.fill = UI.GOLD
	mode.edge = UI.INK
	mode.font_size = 28
	mode.position = Vector2(16, 10)
	add_child(mode)
	var ribbon := UI.Tag.new("Choose your fighter", Vector2(420, 56))
	ribbon.fill = UI.INK
	ribbon.ink = Color(1, 1, 1)
	ribbon.font_size = 34
	ribbon.position = Vector2(232, 6)
	add_child(ribbon)

	entries = Roster.all()
	scroll = ScrollContainer.new()
	var columns := mini(COLUMNS, entries.size() + specials.size())
	var grid_w := columns * (TOKEN + 10) + 10
	scroll.position = Vector2(640 - grid_w * 0.5, 74)
	scroll.size = Vector2(grid_w + 16, 252)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var grid := GridContainer.new()
	grid.columns = COLUMNS
	grid.add_theme_constant_override("h_separation", 10)
	grid.add_theme_constant_override("v_separation", 12)
	scroll.add_child(grid)
	for i in entries.size() + specials.size():
		var card := Card.new()
		if i < entries.size():
			card.name_text = entries[i].name
			card.casual = not Roster.ranked_legal(entries[i])
			# The fighter's own 3D portrait, rendered once (see `_process`).
			var thumb := Preview.new(Vector2i(TOKEN - 12, TOKEN - 16), false, 4.6)
			thumb.position = Vector2(6, 4)
			thumb.spin = false
			thumb.facing_bias = 0.35
			thumb.camera.position = Vector3(0, 1.75, 4.6)
			thumb.camera.look_at_from_position(thumb.camera.position, Vector3(0, 1.4, 0))
			thumb.set_loadout(entries[i].look)
			card.add_child(thumb)
		else:
			card.glyph = "?" if specials[i - entries.size()] == "random" else "+"
			card.name_text = "Random" if card.glyph == "?" else "New fighter"
		var over := Overlay.new(card)
		card.add_child(over)
		var index := i
		card.clicked.connect(func(button): _clicked(index, button))
		grid.add_child(card)
		var tags := []
		for p in 4:
			var flag := UI.Tag.new("P%d" % (p + 1), Vector2(46, 28))
			flag.fill = P_COLORS[p]
			flag.ink = Color(1, 1, 1)
			flag.edge = UI.INK
			flag.font_size = 16
			flag.position = [Vector2(-8, -10), Vector2(TOKEN - 40, -10), Vector2(-8, TOKEN - 22), Vector2(TOKEN - 40, TOKEN - 22)][p]
			flag.visible = false
			card.add_child(flag)
			tags.append(flag)
		tokens.append({"node": card, "card": card, "over": over, "flags": tags})

	for p in 4:
		var panel := PlayerPanel.new(p, P_DISC[p], P_COLORS[p])
		add_child(panel)
		panels.append(panel)

	var hint := UI.Tag.new("P1: WASD, J lock, K back     P2: arrows, Enter lock, Backspace back     E edit     T stocks  Y time  G stage     Esc menu", Vector2(1000, 28))
	hint.fill = Color(0, 0, 0, 0.35)
	hint.ink = Color(1, 1, 1, 0.85)
	hint.font_size = 16
	hint.shadow = false
	hint.position = Vector2(140, 690)
	add_child(hint)

	status = UI.Tag.new("", Vector2(760, 30))
	status.fill = Color(1, 1, 1, 0)
	status.ink = Color(1, 1, 1)
	status.shadow = false
	status.font_size = 20
	status.position = Vector2(260, 372)
	add_child(status)

	rules_button = UI.Btn.new("Rules: Casual", Vector2(260, 50))
	rules_button.font_size = 22
	rules_button.position = Vector2(1006, 8)
	rules_button.activated.connect(_toggle_ranked)
	add_child(rules_button)

	players_button = UI.Btn.new("Players: 2", Vector2(220, 50))
	players_button.font_size = 22
	players_button.position = Vector2(780, 8)
	players_button.activated.connect(_cycle_players)
	add_child(players_button)
	Roster.load_match_rules()
	stocks_button = UI.Btn.new("", Vector2(220, 44))
	stocks_button.font_size = 20
	stocks_button.position = Vector2(250, 330)
	stocks_button.activated.connect(func(): _cycle_rules(true))
	add_child(stocks_button)
	time_button = UI.Btn.new("", Vector2(220, 44))
	time_button.font_size = 20
	time_button.position = Vector2(480, 330)
	time_button.activated.connect(func(): _cycle_rules(false))
	add_child(time_button)
	stage_button = UI.Btn.new("", Vector2(320, 44))
	stage_button.font_size = 20
	stage_button.position = Vector2(710, 330)
	stage_button.activated.connect(_cycle_stage)
	add_child(stage_button)
	_show_match_rules()

	# The band across the screen once everyone has locked in.
	start_button = UI.Btn.new("READY TO FIGHT!   Press Start", Vector2(1300, 84))
	start_button.position = Vector2(-10, 296)
	start_button.font_size = 46
	start_button.fill = Color(0.95, 0.3, 0.24)
	start_button.ink = Color(1, 1, 1)
	start_button.accent = Color(1.0, 0.45, 0.3)
	start_button.selected = true
	start_button.activated.connect(_start)
	start_button.visible = false
	add_child(start_button)

	# Where the cursors begin: on the fighter just made in the creator if there is one.
	var pre: String = Roster.session.get("preselect", "")
	cursor = [0, mini(1, entries.size() - 1), 0, mini(1, entries.size() - 1)]
	if pre != "":
		for i in entries.size():
			if entries[i].slug == pre:
				cursor[0] = i
				_lock(0)
	_layout_panels()
	_refresh()
	PadNav.attach(self)
	Music.of(self).play("menu")
	UI.reveal(self)
	Shot.attach(self)


## The roster portraits are rendered for a few frames when the screen opens, then left as still pictures (they cost nothing after).
func _process(_delta: float) -> void:
	frames += 1
	if not thumbs_frozen and frames > 4:
		thumbs_frozen = true
		for t in tokens:
			for c in t.card.get_children():
				if c is SubViewportContainer:
					c.viewport.render_target_update_mode = SubViewport.UPDATE_DISABLED
					c.set_process(false)


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
			if _all_locked():
				_start()
			else:
				_lock(1)
		KEY_BACKSPACE, KEY_DELETE:
			_unlock(1)
		KEY_SPACE:
			if _all_locked():
				_start()
		KEY_N:
			_cycle_players()
		KEY_E:
			_edit()
		KEY_R:
			_toggle_ranked()
		KEY_T:
			_cycle_rules(true)
		KEY_Y:
			_cycle_rules(false)
		KEY_G:
			_cycle_stage()
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
	if p >= count or locked[p]:
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


func _cycle_stage() -> void:
	Roster.match_stage = (Roster.match_stage + 1) % Roster.stage_names().size()
	Roster.save_match_rules()
	_show_match_rules()
	status.set_text(Roster.stage_blurb(Roster.match_stage))


func _show_match_rules() -> void:
	stocks_button.text = "Stocks: " + Roster.stocks_text(Roster.match_stocks)
	time_button.text = "Time: " + Roster.time_text(Roster.match_time)
	stocks_button.queue_redraw()
	time_button.queue_redraw()
	stage_button.text = "Stage: " + Roster.stage_names()[Roster.match_stage]
	stage_button.queue_redraw()


func _toggle_ranked() -> void:
	ranked = not ranked
	rules_button.text = "Rules: Ranked" if ranked else "Rules: Casual"
	rules_button.queue_redraw()
	# Anyone already locked in with a fighter the new rules forbid is sent back to choose again.
	for p in count:
		if locked[p] and ranked and not Roster.ranked_legal(picked[p]):
			_unlock(p)
	_refresh()
	status.set_text("Ranked rules: fighters over %d points are not allowed." % Roster.budget() if ranked else "Casual rules: anything goes.")


func _lock(p: int) -> void:
	if p >= count:
		return
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
		status.set_text("Only fighters you made can be edited. Make a new one with the + card.")


# ---- Showing it -----------------------------------------------------------------------------------------------------------------

func _refresh() -> void:
	for t in tokens.size():
		var rings := []
		for p in 4:
			tokens[t].flags[p].visible = p < count and cursor[p] == t
			if p < count and cursor[p] == t:
				rings.append([P_COLORS[p], locked[p]])
		tokens[t].card.rings = rings
		tokens[t].card.queue_redraw()
		tokens[t].over.queue_redraw()
	for p in 4:
		panels[p].visible = p < count
		if p >= count:
			continue
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
	var both := _all_locked()
	start_button.visible = both
	status.set_text("")


func _all_locked() -> bool:
	for p in count:
		if not locked[p]:
			return false
	return true


## The players' cards share the bottom of the screen.
func _layout_panels() -> void:
	var gap := 16.0
	var w := (1280.0 - 40.0 - gap * (count - 1)) / count
	for p in 4:
		var panel: Control = panels[p]
		panel.position = Vector2(20.0 + p * (w + gap), 396)
		panel.lay_out(w)
	players_button.text = "Players: %d" % count
	players_button.queue_redraw()


func _cycle_players() -> void:
	count = 2 if count >= 4 else count + 1
	for p in range(count, 4):
		locked[p] = false
		picked[p] = {}
	_layout_panels()
	_refresh()
	if count > 2:
		status.set_text("Players 3 and 4 use controllers 3 and 4.")


## A controller button, from `pad_nav.gd`: controller `pad` plays player `pad` + 1.
func pad_action(pad: int, action: String) -> void:
	match action:
		"up":
			_move(pad, 0, -1)
		"down":
			_move(pad, 0, 1)
		"left":
			_move(pad, -1, 0)
		"right":
			_move(pad, 1, 0)
		"confirm":
			if _all_locked():
				_start()
			else:
				_lock(pad)
		"back":
			_unlock(pad)
		"start":
			if _all_locked():
				_start()
		"menu":
			get_tree().change_scene_to_file("res://menu.tscn")


func _start() -> void:
	if not _all_locked():
		return
	var chosen := []
	for p in count:
		chosen.append(picked[p])
	var built: Dictionary = Roster.build_content(chosen)
	if built.error != "":
		status.set_text(built.error)
		return
	Roster.save_last(picked[0].slug)
	Roster.session = {
		"content_text": built.text, "chars": built.chars, "entries": chosen, "ranked": ranked, "from_menu": true,
		"stocks": Roster.match_stocks, "time": Roster.match_time, "stage": Roster.match_stage,
	}
	get_tree().change_scene_to_file("res://main.tscn")
