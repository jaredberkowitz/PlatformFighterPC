extends CanvasLayer
## The online lobby (the owner, 19 October 2026): once two players are connected, each chooses a fighter and then a stage, and both
## see the other's choices as they change. Left / Right change the choice, Enter locks it in (the fighter, then the stage), Esc takes
## the last lock back, or leaves when nothing is locked. When both have locked both, the match starts on their common stage, or on one
## of the two drawn at random (the host draws: `netplay::peer`). A rematch comes back here with the last choices selected.
## The scene sends the choice to the bridge (`net_lobby_set`) when `changed` fires and feeds the other player's in `show_view`.
## Two steps: the fighters, laid out as fighter select's face-off (the owner's pick of the lobby mocks, 2026-11-05): this player's panel
## at the left, the friend's at the right, both hero-posed toward the VS, the roster's cards between them with this player's cursor and
## the friend's pick badged with their name, the three steps (fighter, stage, ready) on the top bar; then the stage, as stage select
## shows it (the kit's `StagePicker`, in the view the player chose there: V, Tab or a controller's Y switches), the friend's stage badged
## with their name.
## No Random card here: the friend sees the very stage this player would fight on.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const Preview := preload("res://ui/preview.gd")
const Roster := preload("res://scripts/roster.gd")
const Sfx := preload("res://scripts/sfx.gd")
const Select := preload("res://ui/select.gd")

## The choice changed: `fighter_index` into `entries`, the stage, and the two locks.
signal changed(fighter_index: int, stage: int, fighter_locked: bool, stage_locked: bool)
## The player asked to leave (Esc with nothing locked).
signal leave

const FIGHTER := 0
const STAGE := 1
const READY := 2
## The players' colours, as in a match.
const P_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
const STAGE_KEYS := [["A D", "Change"], ["Enter", "Lock in"], ["V", "View"], ["Esc", "Back"]]
const FIGHTER_KEYS := [["WASD", "Choose"], ["Enter", "Lock in"], ["Esc", "Back"]]
## The players' panels' height in the fighter step (fighter select's, less the room for the two players' news under them).
const PANEL_H := 556.0

var entries: Array = []
var stage_names: Array = []
var fighter_index := 0
var stage := 0
## Where the player is: choosing the fighter, the stage, or locked in.
var phase := FIGHTER
var friend_name := "Your friend"
## This player's place in the match (0 hosts), for the colours.
var slot := 0

## The fighter step: the two players' panels (fighter select's `PlayerPanel`), the roster's cards, the steps on the bar.
var fighter_step: Control
var header: Control
var steps: Control
var my_panel: Control
var their_panel: Control
var panels_overlay: Control
var cards: Array = []
var scroll: ScrollContainer
var columns := 4
var hint: Control
## The friend's pick among this player's cards (-1 when it is a fighter of their own), and whether they have locked it in.
var their_index := -1
var their_locked := false
var editor: RefCounted
var frames := 0
var thumbs_frozen := false
## The stage step.
var stage_step: Control
var picker: UI.StagePicker
var bar: UI.TopBar
var my_chip: Control
var their_chip: Control
## The two players' news, under their cards in the fighter step and beside the stages in the stage step.
var my_state: Control
var their_state: Control
var their_look_bytes := PackedByteArray()
var their_fighter_name := "..."
var their_stage_index := -1


class Steps extends Control:
	## On the top bar: who this lobby is with, after the title, and the three steps at the right (fighter, stage, ready), the one this
	## player is on a gold card, those done cream with a tick, those to come dim.
	const NAMES := ["Fighter", "Stage", "Ready"]
	var friend := ""
	var phase := 0

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var lf := D.font()
		var df := D.display_font()
		var title_w := D.display_font().get_string_size("LOBBY", HORIZONTAL_ALIGNMENT_LEFT, -1, UI.TYPE_HEADING).x
		if friend != "":
			var x0 := 40.0 + title_w + 22.0
			draw_string(lf, Vector2(x0, 37), "online with", HORIZONTAL_ALIGNMENT_LEFT, -1, 20, Color(D.SHEET, 0.7))
			var w0 := lf.get_string_size("online with ", HORIZONTAL_ALIGNMENT_LEFT, -1, 20).x
			draw_string(D.display_font(), Vector2(x0 + w0, 38), friend, HORIZONTAL_ALIGNMENT_LEFT, -1, 24, Color.WHITE)
		var x := size.x - 40.0
		for k in range(NAMES.size() - 1, -1, -1):
			var done := k < phase
			var words: String = ("\u2713 " if done else "%d  " % (k + 1)) + NAMES[k]
			var w := df.get_string_size(words, HORIZONTAL_ALIGNMENT_LEFT, -1, 22).x + 28.0
			x -= w
			var r := Rect2(Vector2(x, 12), Vector2(w, 34))
			var fill := D.GOLD if k == phase else (D.SHEET if done else Color(1, 1, 1, 0.12))
			var ink := Art.INK if (k == phase or done) else Color(D.SHEET, 0.6)
			draw_set_transform(r.get_center(), 0.0, Vector2.ONE)
			draw_colored_polygon(PackedVector2Array([Vector2(-w * 0.5 + 6, -17), Vector2(w * 0.5, -17), Vector2(w * 0.5 - 6, 17),
				Vector2(-w * 0.5, 17)]), fill)
			draw_string(df, Vector2(-w * 0.5 + 14, 8), words, HORIZONTAL_ALIGNMENT_LEFT, -1, 22, ink)
			draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
			x -= 12.0


func _init() -> void:
	layer = 11


func _step_group() -> Control:
	var c := Control.new()
	c.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	c.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(c)
	return c


## `start_index` / `start_stage`: what is selected at first (the last match's, or the online screen's).
func build(all_entries: Array, start_index: int, start_stage: int) -> void:
	entries = all_entries
	stage_names = Roster.stage_names()
	fighter_index = clampi(start_index, 0, entries.size() - 1)
	stage = clampi(start_stage, 0, stage_names.size() - 1)

	# The fighters, as fighter select's face-off: this player at the left, the friend at the right, the roster between them.
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	fighter_step = _step_group()
	fighter_step.add_child(UI.Backdrop.new(true))
	header = Select.Header.new()
	header.mode_text = "LOBBY"
	fighter_step.add_child(header)
	steps = Steps.new()
	fighter_step.add_child(steps)
	_build_cards()
	my_panel = Select.PlayerPanel.new(0, P_COLORS[slot % 4].lightened(0.4), P_COLORS[slot % 4])
	my_panel.tab_text = "YOU"
	their_panel = Select.PlayerPanel.new(1, P_COLORS[(1 - slot) % 4].lightened(0.4), P_COLORS[(1 - slot) % 4])
	their_panel.tab_text = "FRIEND"
	for k in 2:
		var panel: Control = [my_panel, their_panel][k]
		panel.facing = 1 if k == 0 else -1
		fighter_step.add_child(panel)
		panel.position = Vector2(Select.SIDE_MARGIN if k == 0 else 1280.0 - Select.SIDE_MARGIN - Select.SIDE_W, Select.PANEL_TOP)
		panel.lay_out(Select.SIDE_W, PANEL_H)
	their_panel.show_entry({}, 100.0, false)
	panels_overlay = Select.PanelsOverlay.new()
	panels_overlay.panels = [my_panel, their_panel]
	fighter_step.add_child(panels_overlay)
	hint = UI.KeyBar.new(FIGHTER_KEYS)
	hint.position = Vector2(60, 680)
	hint.size = Vector2(1160, 30)
	fighter_step.add_child(hint)

	# The stage: stage select's ground, cards and view, the match on the top bar.
	stage_step = _step_group()
	stage_step.add_child(UI.Backdrop.new())
	var blurbs := []
	for i in stage_names.size():
		blurbs.append(Roster.stage_blurb(i))
	picker = UI.StagePicker.new(stage_names, blurbs, false, stage)
	picker.hovered.connect(_hover_stage)
	picker.activated.connect(_click_stage)
	stage_step.add_child(picker)
	bar = UI.TopBar.new("LOBBY")
	stage_step.add_child(bar)
	var keys := UI.KeyBar.new(STAGE_KEYS)
	keys.band = true
	keys.position = Vector2(340, 680)
	keys.size = Vector2(600, 30)
	# (Over the cards, as on stage select: the focused card's long shadow fell across the keys.)
	keys.z_index = 2
	stage_step.add_child(keys)
	my_chip = _chip("YOU", P_COLORS[slot % 4])
	their_chip = _chip("FRIEND", P_COLORS[(1 - slot) % 4])

	my_state = UI.notice(Vector2.ZERO, 520)
	add_child(my_state)
	their_state = UI.notice(Vector2.ZERO, 520)
	add_child(their_state)

	_show_mine()
	_refresh()


## The roster's cards between the panels, as fighter select's (its `Card`, sized by its `grid_layout`), without the random and new-fighter
## cards (the friend sees the very fighter this player would play). Each carries a YOU tab and the friend's tab, shown on the cards the
## two players are on.
func _build_cards() -> void:
	var fit := Select.grid_layout(entries.size(), Select.GRID_AREA)
	columns = fit[0]
	var token: int = fit[1]
	var shown := mini(columns, entries.size())
	var grid_w := shown * (token + int(Select.GRID_GAP.x)) + int(Select.GRID_GAP.x)
	scroll = ScrollContainer.new()
	scroll.position = Vector2(Select.GRID_LEFT + Select.GRID_AREA.x * 0.5 - grid_w * 0.5, Select.GRID_TOP)
	scroll.size = Vector2(grid_w + 16, Select.GRID_AREA.y)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	fighter_step.add_child(scroll)
	var grid := GridContainer.new()
	grid.columns = columns
	grid.add_theme_constant_override("h_separation", 10)
	grid.add_theme_constant_override("v_separation", 12)
	scroll.add_child(grid)
	for i in entries.size():
		var card: Control = Select.Card.new(token)
		card.seed = 100 + i * 13
		card.fill = Select.CARD_COLORS[i % Select.CARD_COLORS.size()]
		card.name_text = entries[i].name
		var thumb := Preview.new(Vector2i(maxi(24, token - 24), maxi(24, Select.card_height(token) - 14)), false, 4.6)
		thumb.position = Vector2(10, 12)
		thumb.spin = false
		thumb.facing_bias = 0.35
		thumb.camera.position = Vector3(0, 1.86, 3.95)
		thumb.camera.look_at_from_position(thumb.camera.position, Vector3(0, 1.56, 0))
		thumb.set_loadout(entries[i].look)
		card.add_child(thumb)
		var over: Control = Select.Overlay.new(card)
		card.add_child(over)
		var index := i
		card.clicked.connect(func(button): _clicked(index, button))
		grid.add_child(card)
		var tags := []
		for k in 2:
			var colour: Color = P_COLORS[slot % 4] if k == 0 else P_COLORS[(1 - slot) % 4]
			var tag: Control = Select.PTab.new("YOU" if k == 0 else "FRIEND", colour, 60 + k)
			var tab := Vector2(clampf(token * 0.5, 34.0, 60.0), clampf(token * 0.22, 16.0, 26.0))
			tag.size = tab
			tag.position = Vector2(0.0 if k == 0 else token - tab.x, -1.0)
			tag.visible = false
			card.add_child(tag)
			tags.append(tag)
		cards.append({"card": card, "over": over, "tags": tags})


## The roster portraits are rendered for a few frames when the step opens, then left as still pictures (as on fighter select).
func _process(_delta: float) -> void:
	frames += 1
	if not thumbs_frozen and frames > 4:
		thumbs_frozen = true
		for c in cards:
			for ch in c.card.get_children():
				if ch is SubViewportContainer:
					ch.viewport.render_target_update_mode = SubViewport.UPDATE_DISABLED
					ch.set_process(false)


## The cards' rings and tabs: this player's cursor, and the friend's pick (heavier once they have locked it in).
func _mark_cards() -> void:
	for i in cards.size():
		var rings := []
		var mine := i == fighter_index
		var theirs := i == their_index
		if mine:
			rings.append([P_COLORS[slot % 4], false])
		if theirs:
			rings.append([P_COLORS[(1 - slot) % 4], their_locked])
		cards[i].tags[0].visible = mine
		cards[i].tags[1].visible = theirs
		cards[i].card.rings = rings
		cards[i].card.queue_redraw()
		cards[i].over.queue_redraw()


func _clicked(i: int, button: int) -> void:
	if button != MOUSE_BUTTON_LEFT or phase != FIGHTER:
		return
	_pick_fighter(i)
	phase = STAGE
	_emit()


## The size a roster entry's stats give its fighter (as on fighter select), or 100.
func _size_of(e: Dictionary) -> float:
	if editor == null or not e.has("class"):
		return 100.0
	var r: Dictionary = editor.recipe_readout(e["class"], e.size, e.speed, e.jump, e.weight)
	return float(r.size_percent)


## Whose news a line is, beside it in the stage step: a slanted chip in the player's colour.
func _chip(text: String, colour: Color) -> Control:
	var t := UI.Tag.new(text, Vector2(104, 38))
	t.fill = Color(colour, 0.85)
	t.ink = Color.WHITE
	t.edge = D.INK
	t.font_size = 20
	stage_step.add_child(t)
	return t


func _pick_fighter(i: int) -> void:
	if phase != FIGHTER or i == fighter_index:
		return
	fighter_index = i
	Sfx.of(self).play("blip")
	_show_mine()
	scroll.ensure_control_visible(cards[i].card)
	_emit()


func _set_stage(i: int) -> void:
	if phase != STAGE or i == stage:
		return
	stage = i
	picker.focus(i)
	Sfx.of(self).play("blip")
	_emit()


func _hover_stage(i: int) -> void:
	_set_stage(i)


func _click_stage(i: int) -> void:
	if phase != STAGE:
		return
	_set_stage(i)
	phase = READY
	_emit()


func _show_mine() -> void:
	var e: Dictionary = entries[fighter_index]
	my_panel.show_entry(e, _size_of(e), false)
	panels_overlay.queue_redraw()
	_mark_cards()


## The class a fighter spec names (`Roster.spec_bytes`: [0, n] a built-in fighter, [1, class, ...] a made one), or 0.
func _spec_class(spec: PackedByteArray) -> int:
	if spec.size() >= 2 and spec[0] == 1:
		return spec[1]
	if spec.size() >= 2:
		for e in entries:
			if Roster.spec_bytes(e) == spec:
				return int(e.get("class", 0))
	return 0


func _emit() -> void:
	changed.emit(fighter_index, stage, phase >= STAGE, phase >= READY)
	_refresh()


func _refresh() -> void:
	steps.phase = phase
	steps.queue_redraw()
	fighter_step.visible = phase == FIGHTER
	stage_step.visible = phase != FIGHTER
	match phase:
		FIGHTER:
			my_state.set_text("Choose your fighter, then press Enter.")
		STAGE:
			my_state.set_text("Choose a stage, then press Enter.")
		READY:
			my_state.set_text("Ready! Esc to change.")
	_layout()


## Puts the two lines of news where the step has room: under each player's card, or stacked beside the stages with their chips.
func _layout() -> void:
	if phase == FIGHTER:
		_place(my_state, Vector2(Select.SIDE_MARGIN + 8.0, 626), Select.SIDE_W - 16.0)
		_place(their_state, Vector2(1280.0 - Select.SIDE_MARGIN - Select.SIDE_W + 8.0, 626), Select.SIDE_W - 16.0)
		return
	var free: Rect2 = UI.StagePicker.FREE[picker.view]
	var w := free.size.x - 112.0
	my_chip.position = free.position
	_place(my_state, free.position + Vector2(112, 0), w)
	their_chip.position = free.position + Vector2(0, 48)
	_place(their_state, free.position + Vector2(112, 48), w)
	# The match this is for, on the top bar: the two fighters in their players' colours.
	var me := str(entries[fighter_index].name).to_upper()
	var them := their_fighter_name.to_upper()
	var items := [[me, P_COLORS[slot % 4]], ["VS", D.GOLD], [them, P_COLORS[(1 - slot) % 4]]]
	if slot == 1:
		items = [[them, P_COLORS[0]], ["VS", D.GOLD], [me, P_COLORS[1]]]
	bar.items = items
	bar.queue_redraw()


func _place(t: Control, at: Vector2, width: float) -> void:
	t.position = at
	t.custom_minimum_size = Vector2(width, 38)
	t.size = Vector2(width, 38)


## The other player's choice as last heard (`pick` from the bridge's `net_lobby_view`, or an empty dictionary before it arrives).
func show_view(pick: Dictionary) -> void:
	if pick.is_empty():
		their_state.set_text("Waiting to hear from your friend...")
		return
	var cosmetics: PackedByteArray = pick.cosmetics
	var locked := bool(pick.fighter_locked)
	if cosmetics != their_look_bytes or locked != their_locked:
		their_look_bytes = cosmetics
		their_locked = locked
		var profile := Roster.parse_profile(cosmetics, 1)
		var spec: PackedByteArray = pick.get("fighter", PackedByteArray())
		their_index = -1
		for i in entries.size():
			if Roster.spec_bytes(entries[i]) == spec:
				their_index = i
		var shown := {"name": profile.name if profile.name != "" else "?", "look": profile.look, "class": _spec_class(spec)}
		their_panel.show_entry(shown, _size_of(entries[their_index]) if their_index >= 0 else 100.0, locked)
		panels_overlay.queue_redraw()
		their_fighter_name = profile.name if profile.name != "" else "?"
		_mark_cards()
		_layout()
	var st := int(pick.stage)
	var stage_text: String = str(stage_names[st]) if st >= 0 and st < stage_names.size() else "?"
	if st != their_stage_index:
		their_stage_index = st
		picker.mark(st if st < stage_names.size() else -1, friend_name.to_upper().left(12), P_COLORS[(1 - slot) % 4])
	if not pick.fighter_locked:
		their_state.set_text("Choosing a fighter...")
	elif not pick.stage_locked:
		their_state.set_text("Choosing a stage: " + stage_text)
	elif phase == READY:
		their_state.set_text("Ready! Starting...")
	else:
		# (Short enough for the line beside the stages; the stage is badged on its card too.)
		their_state.set_text("Ready on %s!" % stage_text)


## The other player left: nothing more can happen here but going back.
func show_left() -> void:
	their_state.set_text("Your friend left.")
	phase = FIGHTER
	_refresh()
	my_state.set_text("Press Esc to go back.")


func set_friend_name(n: String) -> void:
	if n != "":
		friend_name = n
		their_panel.tab_text = n.to_upper().left(8)
		panels_overlay.queue_redraw()
		steps.friend = n.left(16)
		steps.queue_redraw()
		for c in cards:
			c.tags[1].text = n.to_upper().left(6)
			c.tags[1].queue_redraw()
		their_chip.set_text(n.to_upper().left(8))
		if their_stage_index >= 0:
			picker.mark(their_stage_index, n.to_upper().left(12), P_COLORS[(1 - slot) % 4])


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_LEFT, KEY_A:
			_step(-1)
		KEY_RIGHT, KEY_D:
			_step(1)
		KEY_UP, KEY_W:
			if phase == FIGHTER:
				_step(-columns)
		KEY_DOWN, KEY_S:
			if phase == FIGHTER:
				_step(columns)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			if phase < READY:
				phase += 1
				_emit()
		KEY_ESCAPE:
			if phase > FIGHTER:
				phase -= 1
				_emit()
			else:
				leave.emit()
		KEY_V, KEY_TAB:
			if phase == FIGHTER:
				return
			picker.toggle_view()
			Sfx.of(self).play("blip")
			_layout()
		_:
			return
	get_viewport().set_input_as_handled()


## Left (-1) or right (1) on the step's choice: the fighter (a row up or down by `columns`), or the stage (round the ends, as stage
## select). Left and right go round the roster's ends; up and down stop at its top and bottom rows.
func _step(delta: int) -> void:
	if phase == FIGHTER:
		var n := entries.size()
		var i := fighter_index + delta
		if absi(delta) == 1:
			i = (i + n) % n
		elif i < 0 or i >= n:
			return
		_pick_fighter(i)
	elif phase == STAGE:
		_set_stage((stage + delta + stage_names.size()) % stage_names.size())
