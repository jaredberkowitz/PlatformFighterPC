extends CanvasLayer
## The online lobby (the owner, 19 October 2026): once two players are connected, each chooses a fighter and then a stage, and both
## see the other's choices as they change. Left / Right change the choice, Enter locks it in (the fighter, then the stage), Esc takes
## the last lock back, or leaves when nothing is locked. When both have locked both, the match starts on their common stage, or on one
## of the two drawn at random (the host draws: `netplay::peer`). A rematch comes back here with the last choices selected.
## The scene sends the choice to the bridge (`net_lobby_set`) when `changed` fires and feeds the other player's in `show_view`.
## Two steps: the fighters, the two players' cards side by side over the match; then the stage, as stage select shows it (the kit's
## `StagePicker`, in the view the player chose there: V, Tab or a controller's Y switches), the friend's stage badged with their name.
## No Random card here: the friend sees the very stage this player would fight on.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const Preview := preload("res://ui/preview.gd")
const Roster := preload("res://scripts/roster.gd")
const Sfx := preload("res://scripts/sfx.gd")

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

var entries: Array = []
var stage_names: Array = []
var fighter_index := 0
var stage := 0
## Where the player is: choosing the fighter, the stage, or locked in.
var phase := FIGHTER
var friend_name := "Your friend"
## This player's place in the match (0 hosts), for the colours.
var slot := 0

## The fighter step.
var fighter_step: Control
var board: Control
var my_preview: Control
var their_preview: Control
var fighter_row: Control
var their_name: Control
var their_fighter: Control
var their_stage: Control
var hint: Control
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


class Board extends Control:
	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var w := size.x
		draw_rect(Rect2(Vector2.ZERO, size), Color(Art.INK, 0.9))
		var banner := Rect2(Vector2(w * 0.5 - 200, 26), Vector2(400, 76))
		Art.brush_band(self, banner, Art.INK.lightened(0.12), 511, 22.0, true, 1.0)
		Art.title_centered(self, "LOBBY", banner.grow_individual(-12, 0, -26, 0), 54, Color.WHITE, 0.0)


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

	# The fighters: this player on the left, the friend on the right with what they have chosen so far.
	fighter_step = _step_group()
	board = Board.new()
	fighter_step.add_child(board)
	var mine := _player_card(Vector2(60, 120), "YOU")
	my_preview = mine.preview
	var names := entries.map(func(e): return e.name)
	fighter_row = UI.Selector.new("Fighter", names)
	fighter_row.position = Vector2(60, 540)
	fighter_row.changed.connect(func(i): _pick_fighter(i))
	fighter_step.add_child(fighter_row)
	var theirs := _player_card(Vector2(700, 120), "FRIEND")
	their_preview = theirs.preview
	their_name = theirs.label
	their_fighter = UI.Tag.new("...", Vector2(400, 40))
	their_fighter.position = Vector2(700, 540)
	fighter_step.add_child(their_fighter)
	their_stage = UI.Tag.new("...", Vector2(400, 40))
	their_stage.position = Vector2(700, 586)
	fighter_step.add_child(their_stage)
	hint = UI.KeyBar.new([["A D", "Change"], ["Enter", "Lock in"], ["Esc", "Back"]])
	hint.position = Vector2(60, 686)
	hint.size = Vector2(700, 30)
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

	fighter_row.set_index(fighter_index, false)
	_show_mine()
	_refresh()


## A card with a fighter standing on a little stage and a name tag over it.
func _player_card(at: Vector2, heading: String) -> Dictionary:
	var stand := UI.Stage.new(Vector2(520, 380), 0.25, 61 if heading == "YOU" else 67)
	stand.position = at + Vector2(0, 40)
	fighter_step.add_child(stand)
	var preview := Preview.new(Vector2i(500, 360), false)
	stand.add_fighter(preview)
	var label := UI.Tag.new(heading, Vector2(260, 44))
	label.position = at
	label.font_size = 26
	fighter_step.add_child(label)
	return {"preview": preview, "label": label}


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
	if phase != FIGHTER:
		fighter_row.set_index(fighter_index, false)
		return
	fighter_index = i
	_show_mine()
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
	my_preview.set_loadout(e.look)


func _emit() -> void:
	changed.emit(fighter_index, stage, phase >= STAGE, phase >= READY)
	_refresh()


func _refresh() -> void:
	fighter_row.set_focus_mark(phase == FIGHTER)
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
		_place(my_state, Vector2(60, 640), 520)
		_place(their_state, Vector2(700, 640), 520)
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
	if cosmetics != their_look_bytes:
		their_look_bytes = cosmetics
		var profile := Roster.parse_profile(cosmetics, 1)
		their_preview.set_loadout(profile.look)
		var n: String = profile.name if profile.name != "" else "Your friend's fighter"
		their_fighter.set_text(n)
		their_fighter_name = profile.name if profile.name != "" else "?"
		_layout()
	var st := int(pick.stage)
	var stage_text: String = str(stage_names[st]) if st >= 0 and st < stage_names.size() else "?"
	their_stage.set_text("Stage: " + stage_text)
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
	fighter_row.set_focus_mark(false)


func set_friend_name(n: String) -> void:
	if n != "":
		friend_name = n
		their_name.set_text(n.to_upper().left(16))
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


## Left (-1) or right (1) on the step's choice: the fighter, or the stage (round the ends, as stage select).
func _step(delta: int) -> void:
	if phase == FIGHTER:
		fighter_row.step(delta)
	elif phase == STAGE:
		_set_stage((stage + delta + stage_names.size()) % stage_names.size())
