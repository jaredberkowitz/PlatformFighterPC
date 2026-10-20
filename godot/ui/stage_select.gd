extends Control
## Stage select, between fighter select and the match (the owner, 2026-10-20: docs/art/ui/mocks/stage_options.png). Two views of the
## same choice, kept while testers' opinions come in (V, Tab or a controller's Y switches; the choice is remembered):
## - "preview": the focused stage large, its name and what it plays like beside it, every stage as a card in a row under it;
## - "full": the focused stage fills the screen, its name large over it, the stages as a strip of cards on an ink band at the foot.
## The pictures are stills of each stage as a match shows it (art/ui/stages, made by tests/stage_stills.gd); a stage without one gets a
## plain card. A Random card picks any stage.
## Left and right (A D, the arrows) choose, Enter (J, Space) fights there, R picks at random, Esc (K, Backspace) goes back to fighter
## select with everyone still locked in. The mouse: hover chooses, click fights.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const Sfx := preload("res://scripts/sfx.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

const VIEWS := ["preview", "full"]
const VIEW_PATH := "user://stage_view.txt"
const STILLS := "res://art/ui/stages/stage_%d.webp"
## The players' colours, as in a match, for the match line on the top bar.
const P_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
const KEYS := [["←→", "Choose"], ["Enter", "Fight here"], ["R", "Random"], ["V", "View"], ["Esc", "Back"]]

var view := "preview"
var names: Array = []
var blurbs: Array = []
var stills: Array = []
var index := 0
var cards: Array = []
## The view's own pieces (rebuilt when the view changes), above the backdrop and under the bar.
var layer: Control
var big: Control
var words: Control
var full: TextureRect
var rng := RandomNumberGenerator.new()
var leaving := false


## A stage card: the stage's picture in the card's cut shape, its name on an ink band; focused, the kit's frame, lifted and tilted
## (docs/UI_DESIGN_SYSTEM.md, focus). With `show_name` off and never focused it is the large preview.
class StageCard extends Control:
	signal activated
	signal hovered
	var tex: Texture2D
	var name_text := ""
	var name_size := 18
	var show_name := true
	var selected := false
	var pop := 0.0

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func(): hovered.emit())

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()

	func _process(delta: float) -> void:
		var was := pop
		pop = move_toward(pop, 1.0 if selected else 0.0, delta / UI.MOTION_FAST)
		if pop == was:
			return
		pivot_offset = size * 0.5
		rotation = deg_to_rad(UI.FOCUS_TILT) * pop
		scale = Vector2.ONE * lerpf(1.0, UI.FOCUS_SCALE, pop)
		z_index = 1 if pop > 0.01 else 0
		queue_redraw()

	func _draw() -> void:
		var r := Rect2(Vector2.ZERO, size)
		var cut := clampf(minf(size.x, size.y) * 0.13, 8.0, 20.0)
		var shape := Art.card_shape(r, cut, 0)
		Art.focus_frame(self, r, pop, cut)
		if pop < 0.5:
			var sh := PackedVector2Array()
			for q in shape:
				sh.append(q + Art.SHADOW)
			draw_colored_polygon(sh, Color(D.INK, 0.35))
		if tex != null:
			# Cover: the 16:9 picture cropped to the card's shape around its centre.
			var ta := float(tex.get_width()) / maxf(1.0, tex.get_height())
			var ca := size.x / size.y
			var uw := 1.0 if ca >= ta else ca / ta
			var vh := 1.0 if ca <= ta else ta / ca
			var uvs := PackedVector2Array()
			var cols := PackedColorArray()
			for q in shape:
				uvs.append(Vector2(0.5 - uw * 0.5 + uw * q.x / size.x, 0.5 - vh * 0.5 + vh * q.y / size.y))
				cols.append(Color.WHITE)
			draw_polygon(shape, cols, uvs, tex)
		else:
			# No picture (Random, or a stage without its still): the menus' deep indigo, and a "?" for Random.
			var cols := PackedColorArray()
			for q in shape:
				cols.append(Art.INDIGO.lightened(0.12).lerp(Art.INDIGO.darkened(0.2), q.y / size.y))
			draw_polygon(shape, cols)
			if name_text == "Random":
				Art.title_centered(self, "?", Rect2(Vector2.ZERO, size - Vector2(0, name_size + 10)), int(size.y * 0.42), Art.SUN, 0.12)
		if show_name:
			var bh := name_size + 14.0
			var band := PackedVector2Array([Vector2(0, size.y - bh), Vector2(size.x, size.y - bh), Vector2(size.x, size.y - cut),
				Vector2(size.x - cut, size.y), Vector2(0, size.y)])
			draw_colored_polygon(band, Color(D.INK, 0.8))
			var fs := name_size
			while fs > 12 and D.display_font().get_string_size(name_text.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > size.x - 24.0:
				fs -= 1
			draw_string(D.display_font(), Vector2(12, size.y - 10), name_text.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color.WHITE)
		if pop < 0.5:
			var closed := shape.duplicate()
			closed.append(shape[0])
			draw_polyline(closed, D.INK, 3.0, true)


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	rng.randomize()
	add_child(UI.Backdrop.new())
	names = Roster.stage_names()
	for i in names.size():
		blurbs.append(Roster.stage_blurb(i))
		var path := STILLS % i
		stills.append(load(path) if ResourceLoader.exists(path) else null)
	index = clampi(Roster.match_stage, 0, names.size() - 1)
	view = _load_view()

	# The match this is for: the fighters' names in their players' colours.
	var items := []
	var chosen: Array = Roster.session.get("entries", [])
	for p in chosen.size():
		if p > 0:
			items.append(["VS", D.GOLD])
		items.append([str(chosen[p].get("name", "")).to_upper(), P_COLORS[p % 4]])
	add_child(UI.TopBar.new("STAGE", items))
	var keys := UI.KeyBar.new(KEYS)
	keys.position = Vector2(300, 680)
	keys.size = Vector2(680, 30)
	# (Over the cards: the focused card's long shadow fell across the keys.)
	keys.z_index = 2
	add_child(keys)
	_build_view()
	PadNav.attach(self)
	Music.of(self).play("menu")
	UI.reveal(self)
	Shot.attach(self)


static func _load_view() -> String:
	if FileAccess.file_exists(VIEW_PATH):
		var f := FileAccess.open(VIEW_PATH, FileAccess.READ)
		if f != null:
			var v := f.get_as_text().strip_edges()
			if v in VIEWS:
				return v
	return VIEWS[0]


func _save_view() -> void:
	var f := FileAccess.open(VIEW_PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(view)


## The cards and the large picture for the current view.
func _build_view() -> void:
	if layer != null:
		layer.queue_free()
	cards.clear()
	big = null
	full = null
	layer = Control.new()
	layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	layer.size = Vector2(1280, 720)
	layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(layer)
	move_child(layer, 1)
	var n := names.size() + 1
	if view == "full":
		full = TextureRect.new()
		full.size = Vector2(1280, 720)
		full.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		full.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED
		full.mouse_filter = Control.MOUSE_FILTER_IGNORE
		layer.add_child(full)
		words = _painter(_paint_full_words)
		var cw := (1200.0 - 14.0 * (n - 1)) / n
		for i in n:
			_add_card(i, Rect2(Vector2(40 + i * (cw + 14), 574), Vector2(cw, 96)), 16)
	else:
		big = StageCard.new()
		big.show_name = false
		big.mouse_filter = Control.MOUSE_FILTER_IGNORE
		big.position = Vector2(40, 82)
		big.size = Vector2(768, 432)
		layer.add_child(big)
		words = _painter(_paint_preview_words)
		var cw := (1200.0 - 16.0 * (n - 1)) / n
		for i in n:
			_add_card(i, Rect2(Vector2(40 + i * (cw + 16), 548), Vector2(cw, cw * 9.0 / 16.0 + 4.0)), 17)
	_refresh()


func _painter(paint: Callable) -> Control:
	var c := Control.new()
	c.set_anchors_preset(Control.PRESET_FULL_RECT)
	c.size = Vector2(1280, 720)
	c.mouse_filter = Control.MOUSE_FILTER_IGNORE
	c.draw.connect(func(): paint.call(c))
	layer.add_child(c)
	return c


func _add_card(i: int, rect: Rect2, name_size: int) -> void:
	var card := StageCard.new()
	card.position = rect.position
	card.size = rect.size
	card.name_size = name_size
	if i < names.size():
		card.tex = stills[i]
		card.name_text = names[i]
	else:
		card.name_text = "Random"
	card.hovered.connect(func(): _focus(i))
	card.activated.connect(func(): _pick(i))
	layer.add_child(card)
	cards.append(card)


func _focus(i: int) -> void:
	if i == index or leaving:
		return
	index = i
	Sfx.of(self).play("blip")
	_refresh()


func _refresh() -> void:
	for k in cards.size():
		cards[k].selected = k == index
	var tex: Texture2D = stills[index] if index < stills.size() else null
	if big != null:
		big.tex = tex
		big.name_text = _name()
		big.queue_redraw()
	if full != null:
		full.texture = tex
	if words != null:
		words.queue_redraw()


func _name() -> String:
	return names[index] if index < names.size() else "Random"


func _blurb() -> String:
	return blurbs[index] if index < blurbs.size() else "Any of the %d stages, chosen when the match starts." % names.size()


func _paint_preview_words(c: Control) -> void:
	Art.title(c, _name().to_upper(), Vector2(840, 150), UI.TYPE_TITLE, D.INK, 0.0)
	c.draw_line(Vector2(842, 168), Vector2(1100, 168), D.GOLD, 4.0)
	var y := 210.0
	for line in _wrap(_blurb(), 390.0, UI.TYPE_LABEL):
		c.draw_string(D.font(), Vector2(842, y), line, HORIZONTAL_ALIGNMENT_LEFT, -1, UI.TYPE_LABEL, D.INK)
		y += 36.0


func _paint_full_words(c: Control) -> void:
	if full.texture == null:
		# Random (or a stage without its still): the deep indigo of the Random card, and its sign large.
		var top := Art.INDIGO.lightened(0.12)
		var bottom := Art.INDIGO.darkened(0.2)
		c.draw_polygon(PackedVector2Array([Vector2(0, 0), Vector2(1280, 0), Vector2(1280, 720), Vector2(0, 720)]),
			PackedColorArray([top, top, bottom, bottom]))
		if index >= names.size():
			Art.title_centered(c, "?", Rect2(Vector2(0, 40), Vector2(1280, 400)), 300, Color(Art.SUN, 0.85), 0.06)
	# A veil deepening toward the foot, where the words and the strip sit, so the picture stays a place above them.
	var clear := Color(D.INK, 0.0)
	var deep := Color(D.INK, 0.82)
	c.draw_polygon(PackedVector2Array([Vector2(0, 330), Vector2(1280, 330), Vector2(1280, 720), Vector2(0, 720)]),
		PackedColorArray([clear, clear, deep, deep]))
	c.draw_colored_polygon(PackedVector2Array([Vector2(0, 556), Vector2(1280, 556), Vector2(1280, 720), Vector2(0, 720)]), Color(D.INK, 0.6))
	c.draw_line(Vector2(0, 556), Vector2(1280, 556), D.GOLD, 3.0)
	Art.title(c, _name().to_upper(), Vector2(44, 478), UI.TYPE_DISPLAY, Color.WHITE, 0.1)
	c.draw_string(D.font(), Vector2(48, 526), _blurb(), HORIZONTAL_ALIGNMENT_LEFT, 1180, UI.TYPE_LABEL, Color(1, 1, 1, 0.92))


func _wrap(text: String, width: float, fs: int) -> Array:
	var f := D.font()
	var lines := []
	var line := ""
	for word in text.split(" "):
		var trial := word if line == "" else line + " " + word
		if f.get_string_size(trial, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > width and line != "":
			lines.append(line)
			line = word
		else:
			line = trial
	if line != "":
		lines.append(line)
	return lines


# ---- Input -----------------------------------------------------------------------------------------------------------------

func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo or leaving:
		return
	# (Handled first: Back and Fight leave the tree, and then there is no viewport to tell.)
	get_viewport().set_input_as_handled()
	var n := cards.size()
	match event.keycode:
		KEY_LEFT, KEY_A:
			_focus((index - 1 + n) % n)
		KEY_RIGHT, KEY_D:
			_focus((index + 1) % n)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			_pick(index)
		KEY_R:
			_pick(names.size())
		KEY_V, KEY_TAB:
			toggle_view()
		KEY_ESCAPE, KEY_K, KEY_BACKSPACE:
			_back()


## A controller's Y switches the view (pad_nav.gd's "alt").
func pad_scheme(_pad: int) -> Dictionary:
	var scheme: Dictionary = PadNav.DEFAULT.duplicate()
	scheme["alt"] = KEY_V
	return scheme


func toggle_view() -> void:
	view = VIEWS[(VIEWS.find(view) + 1) % VIEWS.size()]
	_save_view()
	Sfx.of(self).play("blip")
	_build_view()


## Fights on stage `i` (the last card picks one at random), starting the match fighter select set up.
func _pick(i: int) -> void:
	if leaving:
		return
	if not Roster.session.has("content_text"):
		# Opened without a match to start (a capture, a test): back to fighter select.
		get_tree().change_scene_to_file("res://select.tscn")
		return
	leaving = true
	index = i
	_refresh()
	Sfx.of(self).play("confirm")
	var stage := i if i < names.size() else rng.randi_range(0, names.size() - 1)
	# (A chosen stage is remembered as the next default; a random pick leaves the last chosen one.)
	if i < names.size():
		Roster.match_stage = stage
		Roster.save_match_rules()
	Roster.session["stage"] = stage
	await get_tree().create_timer(0.12).timeout
	get_tree().change_scene_to_file("res://main.tscn")


func _back() -> void:
	leaving = true
	Roster.session["reselect"] = true
	get_tree().change_scene_to_file("res://select.tscn")
