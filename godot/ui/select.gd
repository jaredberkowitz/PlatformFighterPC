extends Control
## Character select: a grid of roster cards (each with the fighter's own 3D portrait; the built-in fighters, every fighter saved from
## the creator, a random pick and a "new fighter" door), a big card for each player along the bottom, and a "ready to fight" band
## once everyone has locked in.
##
## Player 1: W A S D move, J locks in, K takes it back.   Player 2: arrow keys move, Enter locks in, Backspace takes it back.
## Mouse: left click picks for player 1, right click for player 2.   E edits the saved fighter player 1 is on.
## C makes every player but player 1 a computer opponent (the basic CPU), or people again; its fighter is still chosen with the
## player 2 keys (a computer slot that is not locked in when C is pressed takes the fighter its cursor is on).
## When everyone is locked in, Space (or Enter, or the band) starts the match. Esc goes back to the menu.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Art := preload("res://ui/menu_art.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

## The roster grid fits its cards into the band under the header (GRID_AREA): as many fighters are saved, it starts new rows and makes
## every card smaller (from TOKEN_MAX down to TOKEN_MIN), so a long roster still shows whole at once; past what fits at the smallest
## size (the roster holds up to Roster.MAX_FIGHTERS) it scrolls. `grid_layout` picks the columns and the card size.
const GRID_AREA := Vector2(1180, 236)
const TOKEN_MAX := 132
const TOKEN_MIN := 52
const GRID_GAP := Vector2(10, 12)
## The card size and columns in use (set by `grid_layout` when the screen is built).
var token := 118
var columns := 9


## [columns, card size] for `n` cards: the largest card that fits them all in GRID_AREA (cards are square, plus 12 for the lift), or the
## smallest size filling the width when even that does not fit (then the grid scrolls).
static func grid_layout(n: int) -> Array:
	var best := [maxi(1, n), 0]
	for cols in range(1, maxi(1, n) + 1):
		var rows := ceili(float(n) / float(cols))
		var by_width := floori((GRID_AREA.x - GRID_GAP.x) / float(cols) - GRID_GAP.x)
		var by_height := floori(GRID_AREA.y / float(rows) - GRID_GAP.y - 12.0)
		var t := mini(TOKEN_MAX, mini(by_width, by_height))
		if t > int(best[1]):
			best = [cols, t]
	if int(best[1]) < TOKEN_MIN:
		return [floori((GRID_AREA.x - GRID_GAP.x) / (TOKEN_MIN + GRID_GAP.x)), TOKEN_MIN]
	return best
## The roster cards' colours, in turn (the built-in fighters first), and the random and new-fighter cards'.
const CARD_COLORS := [Art.CORAL, Art.BLUE, Art.SUN, Art.VIOLET, Art.GREEN, Art.TEAL]
const RANDOM_COLOR := Color("252a52")
const NEW_COLOR := Color("1f7a5a")
## The keys along the bottom.
const KEYS := [["WASD", "P1 move   J lock   K back"], ["\u2190\u2192", "P2 move   Enter lock   Bksp back"], ["C", "Computer"],
	["E", "Edit"], ["T", "Stocks"], ["Y", "Time"], ["G", "Stage"], ["Esc", "Menu"]]
const P_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
const P_DISC := [Color(0.98, 0.72, 0.72), Color(0.74, 0.8, 0.98), Color(0.99, 0.9, 0.62), Color(0.76, 0.94, 0.8)]

var entries: Array = []     # every roster entry, then two specials
var specials := ["random", "new"]
var tokens: Array = []
var cursor := [0, 1, 0, 1]
var locked := [false, false, false, false]
var picked: Array = [{}, {}, {}, {}]   # the entry each player ends up with (random is resolved when locking)
## Which players the computer plays (never player 1).
var cpu := [false, false, false, false]
const CPU_LEVEL := 5
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
var panels_overlay: Control
var frames := 0


class Card extends Control:
	## A roster card, like a collectible card: an angled panel in its own colour with a painted rim, the fighter's 3D portrait (or a "?" or
	## "+" glyph), and its name on an ink brush stroke along the bottom. The players whose cursor is on it outline it in their colours
	## (thicker once locked in) and it lifts a little; their P tabs sit on its top corners. The top layer is drawn by `Overlay`.
	var name_text := ""
	var glyph := ""
	var fill := Art.CORAL
	var rings: Array = []   # [colour, locked] for each player on it
	var casual := false
	var hover := false
	var seed := 1
	var lift := 0.0
	## Its size (the grid's card size): text and glyph scale with it.
	var token := 118
	signal clicked(button: int)

	func _init(t := 118) -> void:
		token = t
		custom_minimum_size = Vector2(token, token + 12)
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

	func _process(delta: float) -> void:
		var target := 5.0 if (not rings.is_empty() or hover) else 0.0
		if absf(lift - target) > 0.01:
			lift = move_toward(lift, target, delta * 40.0)
			queue_redraw()
			for ch in get_children():
				if ch is SubViewportContainer:
					ch.position.y = 12.0 - lift
				ch.queue_redraw()

	## The card's body under it (its thickness, as the main menu's tiles have), smaller on a small card.
	func _slab() -> float:
		return clampf(token * 0.04, 2.5, 5.0)

	func _rect() -> Rect2:
		return Rect2(Vector2(6, 8), size - Vector2(14, 18 + _slab()))

	func _draw() -> void:
		Art.card(self, _rect(), fill, seed, 12.0, 3.0, fill.lightened(0.35), lift, Color(0, 0, 0, 0), _slab())
		# A soft burst behind the portrait, and the glyph cards' sign.
		var r := _rect()
		r.position.y -= lift
		draw_circle(r.get_center() + Vector2(0, -10), r.size.x * 0.34, Color(1, 1, 1, 0.14))
		if glyph != "":
			Art.title_centered(self, glyph, Rect2(r.position, r.size - Vector2(0, 26)), int(70.0 * token / 118.0), Art.SUN, 0.12)

	## Drawn over the portrait: the name on its brush stroke, the player outlines and the casual tag.
	func paint_over(over: Control) -> void:
		var r := _rect()
		r.position.y -= lift
		var scale_k := clampf(token / 118.0, 0.55, 1.1)
		var band := Rect2(Vector2(r.position.x + 2, r.end.y - 30.0 * scale_k), Vector2(r.size.x - 6, 28.0 * scale_k))
		Art.brush_band(over, band, Color(Art.INK, 0.92), seed + 5, 8.0 * scale_k, false, 0.4)
		if name_text != "":
			Art.title_centered(over, name_text.to_upper(), band.grow_individual(-6, 0, -10, 0), maxi(9, int(17.0 * scale_k)), Color.WHITE, 0.14)
		for k in rings.size():
			var g := r.grow(3.0 + k * 6.0)
			var pts := PackedVector2Array([g.position + Vector2(12, 0), Vector2(g.end.x, g.position.y), Vector2(g.end.x, g.end.y - 12),
				Vector2(g.end.x - 12, g.end.y), Vector2(g.position.x, g.end.y), g.position + Vector2(0, 12), g.position + Vector2(12, 0)])
			over.draw_polyline(pts, Art.INK, (9.0 if rings[k][1] else 6.0) + 3.0, true)
			over.draw_polyline(pts, rings[k][0], 7.0 if rings[k][1] else 4.0, true)
		if casual:
			var tag := Rect2(Vector2(r.position.x + 10, r.position.y + 8), Vector2(58, 18))
			over.draw_rect(tag, Art.GLOW)
			over.draw_rect(tag, Art.INK, false, 2.0)
			UI.D.draw_text_centered(over, "CASUAL", tag, 12, Art.INK)


class Overlay extends Control:
	## Paints a card's top layer (above its 3D portrait).
	var card: Control

	func _init(c: Control) -> void:
		card = c
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		card.paint_over(self)


class PTab extends Control:
	## A player's tab on a roster card: "P1" on a small brush stroke in the player's colour.
	var text := ""
	var fill := Color.WHITE
	var seed := 1

	func _init(label: String, colour: Color, s := 1) -> void:
		text = label
		fill = colour
		seed = s
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		Art.brush_band(self, Rect2(Vector2.ZERO, size), fill, seed, 8.0, true, 0.4)
		Art.title_centered(self, text, Rect2(Vector2(0, 0), size - Vector2(6, 0)), 20, Color.WHITE, 0.14)


class PlayerPanel extends Control:
	## One player's panel along the bottom: a painted scene (warm for player 1, cool for player 2) framed in the player's colour, the
	## fighter standing on its ledge, the fighter's name on a brush stroke in the player's colour, the player tab, and a gold READY badge
	## when locked in.
	var player := 0
	var disc_color := Color.WHITE
	var flag_color := Color.WHITE
	var preview: Control
	var mask: Control
	var scenery: Control
	var name_text := ""
	var is_ready := false
	var empty := true
	var t := 0.0
	## Played by the computer: the tag says CPU instead of the player number.
	var cpu := false
	const LEAN := 30.0
	const MOODS := [0.85, -0.7, 0.35, 0.0]

	func _init(p: int, disc: Color, flag: Color) -> void:
		player = p
		disc_color = disc
		flag_color = flag
		custom_minimum_size = Vector2(300, 290)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		mask = UI.Mask.new()
		add_child(mask)
		scenery = UI.Scene.new()
		scenery.mood = MOODS[p]
		scenery.seed = 11 + p * 17
		mask.add_child(scenery)
		preview = Preview.new(Vector2i(260, 260), false, 6.8)
		add_child(preview)

	func _frame() -> Rect2:
		return Rect2(Vector2(10, 8), size - Vector2(20, 22))

	func lay_out(w: float) -> void:
		size = Vector2(w, 290)
		var f := _frame()
		mask.position = f.position + Vector2(8, 8)
		mask.size = f.size - Vector2(16, 16)
		mask.set_shape(PackedVector2Array([Vector2(LEAN, 0), Vector2(mask.size.x, 0), Vector2(mask.size.x - LEAN, mask.size.y), Vector2(0, mask.size.y)]))
		scenery.size = mask.size
		var pw := minf(w * 0.8, 300.0)
		preview.size = Vector2(pw, 260)
		preview.position = Vector2(w * 0.5 - pw * 0.5 + 10.0, -26)


	func _draw() -> void:
		var f := _frame()
		var outer := PackedVector2Array([f.position + Vector2(LEAN, 0), Vector2(f.end.x, f.position.y), Vector2(f.end.x - LEAN, f.end.y),
			Vector2(f.position.x, f.end.y)])
		var rough := Art.rough(outer, 40 + player, 2.5, 30.0)
		var sh := PackedVector2Array()
		for q in rough:
			sh.append(q + Vector2(7, 9))
		draw_colored_polygon(sh, Color(Art.INK, 0.55))
		if is_ready:
			var glow := 0.32
			for k in 3:
				var g := PackedVector2Array()
				for q in rough:
					g.append(f.get_center() + (q - f.get_center()) * (1.0 + 0.012 * (k + 1)))
				g.append(g[0])
				draw_polyline(g, Color(Art.SUN, glow * (1.0 - k * 0.3)), 5.0, true)
		draw_colored_polygon(rough, flag_color)
		var closed := rough.duplicate()
		closed.append(rough[0])
		draw_polyline(closed, Art.INK, 5.0, true)

	## Drawn above the fighter by the screen's overlay: the name stroke, the tab and the badge (and "?" with nobody chosen).
	func paint_over(over: Control) -> void:
		var f := _frame()
		f.position += position
		if empty:
			Art.title_centered(over, "?", Rect2(f.position, f.size - Vector2(0, 60)), 110, Color(1, 1, 1, 0.75))
		var band := Rect2(Vector2(f.position.x + 24, f.end.y - 52), Vector2(minf(f.size.x * 0.62, 420.0), 58))
		Art.brush_band(over, band, flag_color.darkened(0.1), 70 + player, 16.0, true, 1.0)
		var shown := name_text.to_upper() if not empty else "CHOOSE A FIGHTER"
		Art.title_centered(over, shown, band.grow_individual(-10, 0, -24, 0), 40, Color.WHITE, 0.12)
		var tab := Rect2(f.position + Vector2(-6, -14), Vector2(86, 48))
		Art.brush_band(over, tab, flag_color.darkened(0.05), 90 + player, 10.0, true, 0.5)
		Art.title_centered(over, "CPU" if cpu else "P%d" % (player + 1), tab, 30, Color.WHITE, 0.14)
		if is_ready:
			var badge := Rect2(Vector2(f.end.x - 196, f.position.y + 6), Vector2(170, 48))
			Art.brush_band(over, badge, Art.SUN, 110 + player, 12.0, true, 0.6)
			Art.title_centered(over, "READY!", badge, 30, Art.INK, 0.0)

	func show_entry(e: Dictionary, size_percent: float, ready: bool) -> void:
		empty = e.is_empty()
		preview.visible = not empty
		if not empty:
			preview.set_loadout(e.look)
			preview.set_size_percent(size_percent)
			name_text = e.name
		is_ready = ready
		queue_redraw()


class PanelsOverlay extends Control:
	## The players' panels' top layer (their names, tabs and badges), above the fighters.
	var panels: Array = []

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		for p in panels:
			if p.visible:
				p.paint_over(self)


class Header extends Control:
	## The screen's title: "VERSUS" on a gold brush stroke and "CHOOSE YOUR FIGHTER" on a dark one.
	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var big := Rect2(Vector2(250, 8), Vector2(490, 74))
		Art.brush_band(self, big, Art.INK, 3, 22.0, true, 1.0)
		Art.title(self, "CHOOSE YOUR FIGHTER", Vector2(big.position.x + 32, big.end.y - 18), 54, Color.WHITE, 0.0)
		var small := Rect2(Vector2(22, 14), Vector2(210, 62))
		Art.brush_band(self, small, Art.SUN, 5, 18.0, true, 0.8)
		Art.title_centered(self, "VERSUS", small.grow_individual(-8, 0, -14, 0), 40, Art.INK, 0.0)


class Ribbon extends Control:
	## The band across the screen once everyone has locked in: "READY TO FIGHT!" on a broad coral brush stroke. Click (or Start) begins.
	signal activated
	var t := 0.0

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()


	func _draw() -> void:
		var r := Rect2(Vector2(40, 6), size - Vector2(80, 12))
		Art.brush_band(self, r, Color("e8424a"), 21, 28.0, true, 1.6)
		Art.brush_band(self, Rect2(r.position + Vector2(80, r.size.y * 0.7), Vector2(r.size.x - 220, r.size.y * 0.16)), Color(1, 1, 1, 0.12), 27, 6.0, false, 0.6)
		var f := UI.D.display_font()
		var text := "READY TO FIGHT!"
		var size_big := 64
		var tw := f.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, size_big).x
		var x := r.position.x + (r.size.x - tw) * 0.5 - 90.0
		var y := r.position.y + r.size.y * 0.5 + f.get_ascent(size_big) * 0.36
		Art.title(self, text, Vector2(x, y), size_big, Color.WHITE, 0.12)
		Art.title(self, "Press Start", Vector2(x + tw + 30.0, y - 8.0), 28, Color(1, 0.86, 0.8), 0.0)


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	rng.randomize()
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	add_child(UI.Backdrop.new(true))

	var header := Header.new()
	header.size = Vector2(800, 100)
	add_child(header)

	entries = Roster.all()
	scroll = ScrollContainer.new()
	# (No "new fighter" card once the roster is full.)
	if Roster.saved().size() >= Roster.MAX_FIGHTERS:
		specials = ["random"]
	var fit := grid_layout(entries.size() + specials.size())
	columns = fit[0]
	token = fit[1]
	var shown := mini(columns, entries.size() + specials.size())
	var grid_w := shown * (token + int(GRID_GAP.x)) + int(GRID_GAP.x)
	scroll.position = Vector2(640 - grid_w * 0.5, 92)
	scroll.size = Vector2(grid_w + 16, GRID_AREA.y)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var grid := GridContainer.new()
	grid.columns = columns
	grid.add_theme_constant_override("h_separation", 10)
	grid.add_theme_constant_override("v_separation", 12)
	scroll.add_child(grid)
	for i in entries.size() + specials.size():
		var card := Card.new(token)
		card.seed = 100 + i * 13
		if i < entries.size():
			card.fill = CARD_COLORS[i % CARD_COLORS.size()]
			card.name_text = entries[i].name
			card.casual = not Roster.ranked_legal(entries[i])
			# The fighter's own 3D portrait, rendered once (see `_process`).
			var thumb := Preview.new(Vector2i(maxi(24, token - 24), maxi(24, token - 30)), false, 4.6)
			thumb.position = Vector2(10, 12)
			thumb.spin = false
			thumb.facing_bias = 0.35
			thumb.camera.position = Vector3(0, 1.75, 4.6)
			thumb.camera.look_at_from_position(thumb.camera.position, Vector3(0, 1.4, 0))
			thumb.set_loadout(entries[i].look)
			card.add_child(thumb)
		else:
			card.glyph = "?" if specials[i - entries.size()] == "random" else "+"
			card.name_text = "Random" if card.glyph == "?" else "New fighter"
			card.fill = RANDOM_COLOR if card.glyph == "?" else NEW_COLOR
		var over := Overlay.new(card)
		card.add_child(over)
		var index := i
		card.clicked.connect(func(button): _clicked(index, button))
		grid.add_child(card)
		var tags := []
		for p in 4:
			var flag := PTab.new("P%d" % (p + 1), P_COLORS[p], 60 + p)
			flag.size = Vector2(50, 28)
			flag.position = [Vector2(-6, -1), Vector2(token - 46, -1), Vector2(-6, token - 24), Vector2(token - 46, token - 24)][p]
			flag.visible = false
			card.add_child(flag)
			tags.append(flag)
		tokens.append({"node": card, "card": card, "over": over, "flags": tags})

	for p in 4:
		var panel := PlayerPanel.new(p, P_DISC[p], P_COLORS[p])
		add_child(panel)
		panels.append(panel)
	panels_overlay = PanelsOverlay.new()
	panels_overlay.panels = panels
	add_child(panels_overlay)

	var footer := UI.KeyBar.new(KEYS)
	footer.position = Vector2(60, 686)
	footer.size = Vector2(1160, 30)
	add_child(footer)

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
	start_button = Ribbon.new()
	start_button.position = Vector2(-20, 296)
	start_button.size = Vector2(1320, 96)
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
		KEY_C:
			_toggle_cpu()
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
	i = clampi(i + dx + dy * columns, 0, n - 1)
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


## Every player but player 1 played by the computer, or by people again. A computer slot that is not locked in takes the fighter its
## cursor is on (a random one if that is the random or new card).
func _toggle_cpu() -> void:
	var on: bool = not cpu[1]
	for p in range(1, 4):
		cpu[p] = on
	if on:
		for p in range(1, count):
			if not locked[p]:
				if cursor[p] >= entries.size():
					cursor[p] = entries.size() + specials.find("random")
				_lock(p)
	_refresh()
	status.set_text("Players 2 to %d: computer" % count if on else "Every player is a person")


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
		panels[p].cpu = cpu[p]
		panels[p].show_entry(shown, size_percent, locked[p])
	panels_overlay.queue_redraw()
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
		panel.position = Vector2(20.0 + p * (w + gap), 392)
		panel.lay_out(w)
	if panels_overlay != null:
		panels_overlay.queue_redraw()
	players_button.text = "Players: %d" % count
	players_button.queue_redraw()


func _cycle_players() -> void:
	count = 2 if count >= 4 else count + 1
	for p in range(count, 4):
		locked[p] = false
		picked[p] = {}
	# A new slot is played by the computer when the others are.
	if cpu[1]:
		for p in range(1, count):
			if not locked[p]:
				cursor[p] = entries.size() + specials.find("random")
				_lock(p)
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
		"cpu": range(4).map(func(p): return CPU_LEVEL if cpu[p] and p < count else 0),
	}
	get_tree().change_scene_to_file("res://main.tscn")
