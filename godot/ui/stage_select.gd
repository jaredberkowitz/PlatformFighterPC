extends Control
## Stage select, between fighter select and the match (the owner, 2026-10-20: docs/art/ui/mocks/stage_options.png). Two views of the
## same choice, kept while testers' opinions come in (V, Tab or a controller's Y switches; the choice is remembered):
## - "preview": the focused stage large, its name and what it plays like beside it, every stage as a card in a row under it;
## - "full": the focused stage fills the screen, its name large over it, the stages as a strip of cards on an ink band at the foot.
## The pictures are stills of each stage as a match shows it (art/ui/stages, made by tests/stage_stills.gd); a stage without one gets a
## plain card. A Random card picks any stage. The cards and both views are the kit's `StagePicker` (the online lobby's stage step uses
## the same one, and the same remembered view).
## Left and right (A D, the arrows) choose, Enter (J, Space) fights there, R picks at random, Esc (K, Backspace) goes back to fighter
## select with everyone still locked in. The mouse: hover chooses, click fights.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const Sfx := preload("res://scripts/sfx.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

## The players' colours, as in a match, for the match line on the top bar.
const P_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
const KEYS := [["←→", "Choose"], ["Enter", "Fight here"], ["R", "Random"], ["V", "View"], ["Esc", "Back"]]

var picker: UI.StagePicker
var names: Array = []
## The view ("preview" or "full") and the focused card, as the picker has them.
var view: String:
	get:
		return picker.view
var index: int:
	get:
		return picker.index
var rng := RandomNumberGenerator.new()
var leaving := false


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	rng.randomize()
	add_child(UI.Backdrop.new())
	names = Roster.stage_names()
	var blurbs := []
	for i in names.size():
		blurbs.append(Roster.stage_blurb(i))
	# (Above the backdrop and under the bar.)
	picker = UI.StagePicker.new(names, blurbs, true, Roster.match_stage)
	picker.hovered.connect(_focus)
	picker.activated.connect(_pick)
	add_child(picker)

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
	PadNav.attach(self)
	Music.of(self).play("menu")
	UI.reveal(self)
	Shot.attach(self)


# ---- Input -----------------------------------------------------------------------------------------------------------------

func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo or leaving:
		return
	# (Handled first: Back and Fight leave the tree, and then there is no viewport to tell.)
	get_viewport().set_input_as_handled()
	var n := picker.count()
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
	picker.toggle_view()
	Sfx.of(self).play("blip")


func _focus(i: int) -> void:
	if i == index or leaving:
		return
	picker.focus(i)
	Sfx.of(self).play("blip")


## Fights on stage `i` (the last card picks one at random), starting the match fighter select set up.
func _pick(i: int) -> void:
	if leaving:
		return
	if not Roster.session.has("content_text"):
		# Opened without a match to start (a capture, a test): back to fighter select.
		get_tree().change_scene_to_file("res://select.tscn")
		return
	leaving = true
	picker.focus(i)
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
