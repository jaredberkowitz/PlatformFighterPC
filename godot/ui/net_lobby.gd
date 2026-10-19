extends CanvasLayer
## The online lobby (the owner, 19 October 2026): once two players are connected, each chooses a fighter and then a stage, and both
## see the other's choices as they change. Left / Right change the choice, Enter locks it in (the fighter, then the stage), Esc takes
## the last lock back, or leaves when nothing is locked. When both have locked both, the match starts on their common stage, or on one
## of the two drawn at random (the host draws: `netplay::peer`). A rematch comes back here with the last choices selected.
## The scene sends the choice to the bridge (`net_lobby_set`) when `changed` fires and feeds the other player's in `show_view`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const Preview := preload("res://ui/preview.gd")
const Roster := preload("res://scripts/roster.gd")

## The choice changed: `fighter_index` into `entries`, the stage, and the two locks.
signal changed(fighter_index: int, stage: int, fighter_locked: bool, stage_locked: bool)
## The player asked to leave (Esc with nothing locked).
signal leave

const FIGHTER := 0
const STAGE := 1
const READY := 2

var entries: Array = []
var stage_names: Array = []
var fighter_index := 0
var stage := 0
## Where the player is: choosing the fighter, the stage, or locked in.
var phase := FIGHTER
var friend_name := "Your friend"

var board: Control
var my_preview: Control
var their_preview: Control
var my_rows: VBoxContainer
var fighter_row: Control
var stage_row: Control
var my_state: Control
var their_name: Control
var their_fighter: Control
var their_stage: Control
var their_state: Control
var hint: Control
var their_look_bytes := PackedByteArray()


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


## `start_index` / `start_stage`: what is selected at first (the last match's, or the online screen's).
func build(all_entries: Array, start_index: int, start_stage: int) -> void:
	entries = all_entries
	stage_names = Roster.stage_names()
	fighter_index = clampi(start_index, 0, entries.size() - 1)
	stage = clampi(start_stage, 0, stage_names.size() - 1)
	board = Board.new()
	add_child(board)

	# This player, on the left.
	var mine := _player_card(Vector2(60, 120), "YOU")
	my_preview = mine.preview
	my_rows = VBoxContainer.new()
	my_rows.position = Vector2(60, 540)
	my_rows.add_theme_constant_override("separation", 6)
	add_child(my_rows)
	var names := entries.map(func(e): return e.name)
	fighter_row = UI.Selector.new("Fighter", names)
	stage_row = UI.Selector.new("Stage", stage_names)
	fighter_row.changed.connect(func(i): _pick_fighter(i))
	stage_row.changed.connect(func(i): _pick_stage(i))
	my_rows.add_child(fighter_row)
	my_rows.add_child(stage_row)
	my_state = UI.notice(Vector2(60, 640), 520)
	add_child(my_state)

	# The friend, on the right: what they have chosen so far.
	var theirs := _player_card(Vector2(700, 120), "FRIEND")
	their_preview = theirs.preview
	their_name = theirs.label
	their_fighter = UI.Tag.new("...", Vector2(400, 40))
	their_fighter.position = Vector2(700, 540)
	add_child(their_fighter)
	their_stage = UI.Tag.new("...", Vector2(400, 40))
	their_stage.position = Vector2(700, 586)
	add_child(their_stage)
	their_state = UI.notice(Vector2(700, 640), 520)
	add_child(their_state)

	hint = UI.KeyBar.new([["A D", "Change"], ["Enter", "Lock in"], ["Esc", "Back"]])
	hint.position = Vector2(60, 686)
	hint.size = Vector2(700, 30)
	add_child(hint)

	fighter_row.set_index(fighter_index, false)
	stage_row.set_index(stage, false)
	_show_mine()
	_refresh()


## A card with a fighter standing on a little stage and a name tag over it.
func _player_card(at: Vector2, heading: String) -> Dictionary:
	var stand := UI.Stage.new(Vector2(520, 380), 0.25, 61 if heading == "YOU" else 67)
	stand.position = at + Vector2(0, 40)
	add_child(stand)
	var preview := Preview.new(Vector2i(500, 360), false)
	stand.add_fighter(preview)
	var label := UI.Tag.new(heading, Vector2(260, 44))
	label.position = at
	label.font_size = 26
	add_child(label)
	return {"preview": preview, "label": label}


func _pick_fighter(i: int) -> void:
	if phase != FIGHTER:
		fighter_row.set_index(fighter_index, false)
		return
	fighter_index = i
	_show_mine()
	_emit()


func _pick_stage(i: int) -> void:
	if phase != STAGE:
		stage_row.set_index(stage, false)
		return
	stage = i
	_emit()


func _show_mine() -> void:
	var e: Dictionary = entries[fighter_index]
	my_preview.set_loadout(e.look)


func _emit() -> void:
	changed.emit(fighter_index, stage, phase >= STAGE, phase >= READY)
	_refresh()


func _refresh() -> void:
	fighter_row.set_focus_mark(phase == FIGHTER)
	stage_row.set_focus_mark(phase == STAGE)
	match phase:
		FIGHTER:
			my_state.set_text("Choose your fighter, then press Enter.")
		STAGE:
			my_state.set_text("Fighter locked in. Choose a stage, then press Enter.")
		READY:
			my_state.set_text("Ready! (Esc to change)")


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
	var st := int(pick.stage)
	their_stage.set_text("Stage: " + (str(stage_names[st]) if st < stage_names.size() else "?"))
	if not pick.fighter_locked:
		their_state.set_text("Choosing a fighter...")
	elif not pick.stage_locked:
		their_state.set_text("Fighter locked in. Choosing a stage...")
	elif phase == READY:
		their_state.set_text("Ready! Starting...")
	else:
		their_state.set_text("Ready! Waiting for you.")


## The other player left: nothing more can happen here but going back.
func show_left() -> void:
	their_state.set_text("Your friend left.")
	my_state.set_text("Press Esc to go back.")
	phase = FIGHTER
	_refresh_marks_only()


func _refresh_marks_only() -> void:
	fighter_row.set_focus_mark(false)
	stage_row.set_focus_mark(false)


func set_friend_name(n: String) -> void:
	if n != "":
		friend_name = n
		their_name.set_text(n.to_upper().left(16))


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
		_:
			return
	get_viewport().set_input_as_handled()


func _step(delta: int) -> void:
	if phase == FIGHTER:
		fighter_row.step(delta)
	elif phase == STAGE:
		stage_row.step(delta)
