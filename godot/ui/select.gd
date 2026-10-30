extends Control
## Character select, "fighters forward" (the owner's pick of 2026-10-19, docs/art/ui/mocks/select_options.png layout 3): the chosen
## fighters large in a tall panel each (facing across a VS with two players), the roster as a strip of cards along the bottom (each with
## the fighter's own 3D portrait; the built-in fighters, every fighter saved from the creator, a random pick and a "new fighter" door),
## the match rules in a bar along the top, and a "ready to fight" band over the strip once everyone has locked in.
##
## Player 1: W A S D move, J locks in, K takes it back.   Player 2: arrow keys move, Enter locks in, Backspace takes it back.
## Mouse: left click picks for player 1, right click for player 2.   E edits the saved fighter player 1 is on.
## C makes every player but player 1 a computer opponent (the basic CPU), or people again; its fighter is still chosen with the
## player 2 keys (a computer slot that is not locked in when C is pressed takes the fighter its cursor is on).
## When everyone is locked in, Space (or Enter, or the band) starts the match. Esc goes back to the menu.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

## The roster grid fits its cards into the strip along the bottom (GRID_AREA, from GRID_TOP): as many fighters are saved, it starts new
## rows and makes every card smaller (from TOKEN_MAX down to TOKEN_MIN wide), so a long roster still shows whole at once; past what fits
## at the smallest size (the roster holds up to Roster.MAX_FIGHTERS) it scrolls. `grid_layout` picks the columns and the card width; a
## card is CARD_H of its width tall (wider than tall: head and shoulders), plus 12 for its lift and body.
const GRID_AREA := Vector2(1192, 214)
const GRID_TOP := 452.0
const CARD_H := 0.78
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
		var by_height := floori((GRID_AREA.y / float(rows) - GRID_GAP.y - 12.0) / CARD_H)
		var t := mini(TOKEN_MAX, mini(by_width, by_height))
		if t > int(best[1]):
			best = [cols, t]
	if int(best[1]) < TOKEN_MIN:
		return [floori((GRID_AREA.x - GRID_GAP.x) / (TOKEN_MIN + GRID_GAP.x)), TOKEN_MIN]
	return best
## A card's face height for a card `t` wide.
static func card_height(t: int) -> int:
	return int(t * CARD_H)


## The roster cards' colours, in turn (the built-in fighters first), and the random and new-fighter cards'.
const CARD_COLORS := [Art.CORAL, Art.BLUE, Art.SUN, Art.VIOLET, Art.GREEN, Art.TEAL]
const RANDOM_COLOR := Color("252a52")
const NEW_COLOR := Color("1f7a5a")
## The keys along the bottom.
const KEYS := [["WASD", "P1 move   J lock   K back"], ["\u2190\u2192", "P2 move   Enter lock   Bksp back"], ["C", "Computer"],
	["E", "Edit"], ["T", "Stocks"], ["Y", "Time"], ["Esc", "Menu"]]
const RUN_KEYS := [["WASD", "Move   J lock   K back"], ["Enter", "Run"], ["E", "Edit"], ["Esc", "Menu"]]
## The players' panels' top and height (between the top bar and the news line over the roster).
const PANEL_TOP := 64.0
const PANEL_H := 348.0
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
## Choosing the fighter for the single-player run (`Roster.session.mode == "run"`, from the main menu's Run tile): one player, no
## match rules, and the choice starts a run (`runner.tscn`).
var run_mode := false
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
var thumbs_frozen := false
var panels_overlay: Control
var frames := 0
var header: Control


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
		custom_minimum_size = Vector2(token, int(token * CARD_H) + 12)
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
			Art.title_centered(self, glyph, Rect2(r.position, r.size - Vector2(0, 26)), int(58.0 * token / 118.0), Art.SUN, 0.12)

	## Drawn over the portrait: the name on its brush stroke, the player outlines and the casual tag.
	func paint_over(over: Control) -> void:
		var r := _rect()
		r.position.y -= lift
		var scale_k := clampf(token / 118.0, 0.55, 1.1)
		var band := Rect2(Vector2(r.position.x + 2, r.end.y - 30.0 * scale_k), Vector2(r.size.x - 6, 28.0 * scale_k))
		Art.brush_band(over, band, Color(Art.INK, 0.92), seed + 5, 8.0 * scale_k, false, 0.4)
		if name_text != "":
			# (Bigger since the select rebuild, 2026-10-19: at 1.0% of the screen's height the names were unreadable from a couch.)
			Art.title_centered(over, name_text.to_upper(), band.grow_individual(-6, 0, -10, 0), maxi(11, int(19.0 * scale_k)), Color.WHITE, 0.14)
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
		Art.title_centered(self, text, Rect2(Vector2(0, 0), size - Vector2(6, 0)), clampi(int(size.y * 0.72), 12, 20), Color.WHITE, 0.14)


class PlayerPanel extends Control:
	## One player's panel along the bottom: a spotlit card in the player's colour (`UI.Spotlight`), the fighter standing in its light, the fighter's name on a brush stroke in the player's colour, the player tab, and a gold READY badge
	## when locked in.
	var player := 0
	var disc_color := Color.WHITE
	var flag_color := Color.WHITE
	var preview: Control
	var mask: Control
	var scenery: Control
	var name_text := ""
	## The fighter's class, a quieter caption over its name.
	var class_text := ""
	var is_ready := false
	var empty := true
	var t := 0.0
	## Played by the computer: the tag says CPU instead of the player number.
	var cpu := false
	const LEAN := 30.0
	## The fighter's resting height in the panel, and how far it rises from on a new pick and hops on locking in.
	# (Lower since the select rebuild, 2026-10-19: in the tall panels the fighter's feet floated over its name and its hat left the frame.)
	const BASE_Y := 4.0
	const PICK_RISE := 30.0
	const LOCK_HOP := 18.0
	var pick_t := 1.0
	var lock_t := 1.0

	func _process(delta: float) -> void:
		if pick_t >= 1.0 and lock_t >= 1.0:
			return
		pick_t = minf(1.0, pick_t + delta / UI.MOTION_BASE)
		lock_t = minf(1.0, lock_t + delta / (UI.MOTION_BASE * 1.5))
		var k := 1.0 - pow(1.0 - pick_t, 3.0)
		preview.position.y = BASE_Y + (1.0 - k) * PICK_RISE - sin(lock_t * PI) * LOCK_HOP
		preview.modulate.a = lerpf(0.25, 1.0, k)
	const MOODS := [0.85, -0.7, 0.35, 0.0]

	func _init(p: int, disc: Color, flag: Color) -> void:
		player = p
		disc_color = disc
		flag_color = flag
		custom_minimum_size = Vector2(200, 200)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		mask = UI.Mask.new()
		add_child(mask)
		# (A spotlit card in the player's colour, as the main menu's tiles: the world itself is the live backdrop behind the screen.)
		var spot := UI.Spotlight.new()
		# (A clean pool of light in a softer take on the player's colour, no rays or hatching: the slots were the heaviest thing on the
		# screen; the design system's lessons from the owner's results plan, 2026-10-17.)
		# (Richer since the UI pass: the pale pool washed the panels out against the menus' deep backdrop.)
		spot.colour = Color.from_hsv(flag.h, 0.62, 0.72 if Art.paper_style() > 0 else 0.5)
		spot.plain = true
		spot.feet = [[0.5, 0.72]]
		spot.seed = 11 + p * 17
		scenery = spot
		mask.add_child(scenery)
		preview = Preview.new(Vector2i(260, 260), false, 6.8)
		add_child(preview)

	func _frame() -> Rect2:
		return Rect2(Vector2(10, 8), size - Vector2(20, 22))

	func lay_out(w: float, h := 290.0) -> void:
		size = Vector2(w, h)
		var f := _frame()
		mask.position = f.position + Vector2(8, 8)
		mask.size = f.size - Vector2(16, 16)
		mask.set_shape(PackedVector2Array([Vector2(LEAN, 0), Vector2(mask.size.x, 0), Vector2(mask.size.x - LEAN, mask.size.y), Vector2(0, mask.size.y)]))
		scenery.size = mask.size
		# The fighter fills the panel's height (the camera keeps the height, so a taller view is a bigger fighter).
		var ph := h - 30.0
		var pw := minf(w * 0.86, ph * 1.1)
		preview.size = Vector2(pw, ph)
		preview.position = Vector2(w * 0.5 - pw * 0.5 + 10.0, BASE_Y)


	func _draw() -> void:
		var f := _frame()
		var outer := PackedVector2Array([f.position + Vector2(LEAN, 0), Vector2(f.end.x, f.position.y), Vector2(f.end.x - LEAN, f.end.y),
			Vector2(f.position.x, f.end.y)])
		# (Crisp since the UI pass, docs/UI_REVIEW.md: the hand-wobbled frame beside the crisp cards read as two styles.)
		var rough := outer
		if Art.paper_style() > 0:
			_draw_paper_frame(rough, f)
			return
		var sh := PackedVector2Array()
		for q in rough:
			sh.append(q + Art.SHADOW)
		draw_colored_polygon(sh, Color(Art.INK, 0.6))
		if is_ready:
			var g := PackedVector2Array()
			for q in rough:
				g.append(f.get_center() + (q - f.get_center()) * 1.025)
			g.append(g[0])
			draw_polyline(g, Art.SUN, 6.0, true)
		# The player's colour, deep and lighter toward the top, as the cards are.
		var top := flag_color.darkened(0.05)
		var bottom := flag_color.darkened(0.45)
		var cols := PackedColorArray()
		for q in rough:
			cols.append(top.lerp(bottom, clampf((q.y - f.position.y) / maxf(f.size.y, 1.0), 0.0, 1.0)))
		draw_polygon(rough, cols)
		draw_line(f.position + Vector2(LEAN + 6.0, 6.0), Vector2(f.end.x - 8.0, f.position.y + 6.0), Color(1, 1, 1, 0.4), 2.5, true)
		var closed := rough.duplicate()
		closed.append(rough[0])
		draw_polyline(closed, Art.INK, 5.0, true)

	## The paper look's frame: a cream mount (gold once locked in) on a soft lifted shadow, under the panel's own sheet of colour.
	func _draw_paper_frame(shape: PackedVector2Array, f: Rect2) -> void:
		var centre := f.get_center()
		var mount := PackedVector2Array()
		for q in shape:
			mount.append(centre + (q - centre) * 1.0)
		Art._paper_shadow(self, mount, 1.0, 4.0)
		draw_colored_polygon(mount, Art.SUN if is_ready else Art.PAPER_EDGE)
		Art._paper_edges(self, mount, Art.SUN if is_ready else Art.PAPER_EDGE, 2.0)

	## Drawn above the fighter by the screen's overlay: the name stroke, the tab and the badge (and "?" with nobody chosen).
	func paint_over(over: Control) -> void:
		var f := _frame()
		f.position += position
		if empty:
			Art.title_centered(over, "?", Rect2(f.position, f.size - Vector2(0, 60)), 110, Color(1, 1, 1, 0.75))
		var band := Rect2(Vector2(f.position.x + 24, f.end.y - 60), Vector2(minf(f.size.x * 0.7, 440.0), 66))
		Art.brush_band(over, band, flag_color.darkened(0.1), 70 + player, 16.0, true, 1.0)
		var shown := name_text.to_upper() if not empty else "CHOOSE A FIGHTER"
		Art.title_centered(over, shown, band.grow_individual(-10, 0, -24, 0), 48 if f.size.x > 400.0 else 36, Color.WHITE, 0.12)
		if not empty and class_text != "":
			var f_cap := D.font()
			var cap_w := f_cap.get_string_size(class_text, HORIZONTAL_ALIGNMENT_LEFT, -1, UI.TYPE_CAPTION).x + 24.0
			var cap := Rect2(Vector2(band.position.x + 18.0, band.position.y - 24.0), Vector2(cap_w, 24.0))
			over.draw_rect(cap, Color(Art.INK, 0.85))
			D.draw_text_centered(over, class_text, cap, UI.TYPE_CAPTION, D.CREAM)
		var tab := Rect2(f.position + Vector2(-6, -14), Vector2(86, 48))
		Art.brush_band(over, tab, flag_color.darkened(0.05), 90 + player, 10.0, true, 0.5)
		Art.title_centered(over, "CPU" if cpu else "P%d" % (player + 1), tab, 30, Color.WHITE, 0.14)
		if is_ready:
			# (Narrower in a narrow panel, the select rebuild: with four players it ran into the P tab.)
			var bw := minf(170.0, f.size.x - 140.0)
			var badge := Rect2(Vector2(f.end.x - bw - 26.0, f.position.y + 6), Vector2(bw, 48))
			Art.brush_band(over, badge, Art.SUN, 110 + player, 12.0, true, 0.6)
			Art.title_centered(over, "READY!", badge, 30, Art.INK, 0.0)

	func show_entry(e: Dictionary, size_percent: float, ready: bool) -> void:
		# A new fighter slides up into the light; locking in gives a hop (`_process`).
		if not e.is_empty() and str(e.get("name", "")) != name_text:
			pick_t = 0.0
		if ready and not is_ready:
			lock_t = 0.0
		empty = e.is_empty()
		preview.visible = not empty
		if not empty:
			preview.set_loadout(e.look)
			preview.set_size_percent(size_percent)
			name_text = e.name
			var c := int(e.get("class", -1))
			preview.set_weapon(c)
			class_text = Roster.CLASS_NAMES[c].to_upper() if c >= 0 and c < Roster.CLASS_NAMES.size() else ""
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
	## The bar along the top: an ink band across the screen with a gold line under it, the mode ("VERSUS" or "RUN") at its left; the
	## match rules sit on it at the right (the screen places them). With two players, the gold "VS" between their panels.
	var mode_text := "VERSUS"
	var show_vs := true:
		set(v):
			show_vs = v
			queue_redraw()

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		draw_rect(Rect2(0, 0, size.x, TOP_BAR), Color(Art.INK, 0.94))
		draw_line(Vector2(0, TOP_BAR), Vector2(size.x, TOP_BAR), D.GOLD, 3.0)
		Art.title(self, mode_text, Vector2(40, 43), UI.TYPE_HEADING, Color.WHITE, 0.0)
		if show_vs:
			Art.title_centered(self, "VS", Rect2(Vector2(580, 180), Vector2(120, 110)), 80, D.GOLD, 0.12)


## The top bar's height.
const TOP_BAR := 58.0


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
		var size_big := UI.TYPE_TITLE
		var tw := f.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, size_big).x
		var x := r.position.x + (r.size.x - tw) * 0.5 - 90.0
		var y := r.position.y + r.size.y * 0.5 + f.get_ascent(size_big) * 0.36
		Art.title(self, text, Vector2(x, y), size_big, Color.WHITE, 0.12)
		Art.title(self, "Press Start", Vector2(x + tw + 30.0, y - 6.0), UI.TYPE_LABEL, Color(1, 0.86, 0.8), 0.0)


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	rng.randomize()
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	add_child(UI.Backdrop.new(true))

	run_mode = Roster.session.get("mode", "") == "run"
	header = Header.new()
	if run_mode:
		header.mode_text = "RUN"
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
	scroll.position = Vector2(640 - grid_w * 0.5, GRID_TOP)
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
			# (Down to the card's foot, under the name band, since the cards went wide in the select rebuild: the face had only half the card.)
			var thumb := Preview.new(Vector2i(maxi(24, token - 24), maxi(24, card_height(token) - 14)), false, 4.6)
			thumb.position = Vector2(10, 12)
			thumb.spin = false
			thumb.facing_bias = 0.35
			# Head and shoulders, as the reference fighters' portraits are (art review 6 round 2: the whole body at this size left the face a
			# few pixels across and the roster a row of near-identical dolls; the face, hair and hat are what tell fighters apart).
			# (Up and back a little for the rebuilt body's bigger head, 2026-10-18: framed for the old one, it cut the hats and crowns off.)
			# (Closer since the select rebuild: a wide card shows the head small at the old distance.)
			thumb.camera.position = Vector3(0, 1.86, 3.95)
			thumb.camera.look_at_from_position(thumb.camera.position, Vector3(0, 1.56, 0))
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
			# Sized with the card and kept inside it, two to a row (players 3 and 4 under 1 and 2, clear of the name): they hung over the
			# card's edges, so two players on neighbouring cards stacked their tabs (the UI pass, docs/art/ui/UI_ANALYSIS.md).
			var tab := Vector2(clampf(token * 0.4, 28.0, 46.0), clampf(token * 0.22, 16.0, 26.0))
			flag.size = tab
			flag.position = Vector2(0.0 if p % 2 == 0 else token - tab.x, -1.0 + (p / 2) * (tab.y + 1.0))
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

	var footer := UI.KeyBar.new(RUN_KEYS if run_mode else KEYS)
	# (Up from 686, 2026-10-18: the bar's keys were cut by the bottom of the screen.)
	footer.position = Vector2(60, 674)
	footer.size = Vector2(1160, 30)
	add_child(footer)

	# News (the rules' meaning, players 3 and 4) on a dark band between the fighters and the roster.
	status = UI.notice(Vector2(340, 410), 600, UI.TYPE_BODY)
	add_child(status)

	# The match rules, on the top bar at the right (they floated between the grid and the panels before the rebuild).
	Roster.load_match_rules()
	players_button = _rule_button("Players: 2", 150.0, _cycle_players)
	rules_button = _rule_button("Rules: Casual", 170.0, _toggle_ranked)
	stocks_button = _rule_button("", 160.0, func(): _cycle_rules(true))
	time_button = _rule_button("", 140.0, func(): _cycle_rules(false))
	# (The stage is chosen on its own screen after this one since 2026-10-20, `stage_select.gd`.)
	var x := 1250.0
	for b in [time_button, stocks_button, rules_button, players_button]:
		x -= b.custom_minimum_size.x
		b.position = Vector2(x, 5)
		x -= 4.0
	_show_match_rules()

	# The band across the roster once everyone has locked in (the roster is done with by then).
	start_button = Ribbon.new()
	start_button.position = Vector2(-20, 500)
	start_button.size = Vector2(1320, 84)
	start_button.activated.connect(_start)
	start_button.visible = false
	add_child(start_button)

	if run_mode:
		count = 1
		for b in [players_button, rules_button, stocks_button, time_button]:
			b.visible = false
		status.set_text("Single player run: choose your fighter.")

	# Where the cursors begin: on the fighter just made in the creator if there is one.
	var pre: String = Roster.session.get("preselect", "")
	cursor = [0, mini(1, entries.size() - 1), 0, mini(1, entries.size() - 1)]
	if pre != "":
		for i in entries.size():
			if entries[i].slug == pre:
				cursor[0] = i
				_lock(0)
	# Back from stage select: everyone as they were, locked in on the same fighters.
	if Roster.session.get("reselect", false):
		_reselect()
	_layout_panels()
	_refresh()
	PadNav.attach(self)
	Music.of(self).play("menu")
	UI.reveal(self)
	Shot.attach(self)


## A match rule on the top bar: a dark setting button, its value in gold.
func _rule_button(text: String, w: float, action: Callable) -> Control:
	var b := UI.Btn.new(text, Vector2(w, 48))
	b.setting = true
	b.font_size = 20
	b.activated.connect(action)
	add_child(b)
	return b


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
				_lock(0 if run_mode else 1)
		KEY_BACKSPACE, KEY_DELETE:
			_unlock(1)
		KEY_SPACE:
			if _all_locked():
				_start()
		KEY_N:
			if not run_mode:
				_cycle_players()
		KEY_C:
			if not run_mode:
				_toggle_cpu()
		KEY_E:
			_edit()
		KEY_R:
			if not run_mode:
				_toggle_ranked()
		KEY_T:
			if not run_mode:
				_cycle_rules(true)
		KEY_Y:
			if not run_mode:
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
## The players' panels between the top bar and the roster: two face each other across the VS; three or four stand side by side; the run's
## one stands in the middle.
func _layout_panels() -> void:
	var gap := 80.0 if count == 2 else 12.0
	var w := (1220.0 - gap * (count - 1)) / count if count > 1 else 600.0
	var x0 := 30.0 if count > 1 else 640.0 - w * 0.5
	for p in 4:
		var panel: Control = panels[p]
		panel.position = Vector2(x0 + p * (w + gap), PANEL_TOP)
		panel.lay_out(w, PANEL_H)
	if header != null:
		header.show_vs = count == 2
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
	if run_mode:
		Roster.save_last(picked[0].slug)
		Roster.session = {"run_entry": picked[0], "from_menu": true}
		get_tree().change_scene_to_file("res://runner.tscn")
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
	# The stage next, on its own screen (it starts the match).
	get_tree().change_scene_to_file("res://stage.tscn")


## Puts everyone back as they were when the match was set up (stage select's Back): the same number of players, the computer
## slots, the rules, and each player locked in on the fighter they had.
func _reselect() -> void:
	var chosen: Array = Roster.session.get("entries", [])
	count = clampi(chosen.size(), 1, 4)
	ranked = bool(Roster.session.get("ranked", false))
	rules_button.text = "Rules: Ranked" if ranked else "Rules: Casual"
	var levels: Array = Roster.session.get("cpu", [])
	for p in count:
		cpu[p] = p < levels.size() and int(levels[p]) > 0
		for i in entries.size():
			if entries[i].slug == chosen[p].slug:
				cursor[p] = i
		picked[p] = chosen[p]
		locked[p] = true
