extends Control
## The main menu: where the game starts. Play goes to character select, Character Creator makes and saves fighters,
## Editors opens the content editors, Quit leaves. Up and down (or W and S) choose, Enter (or J, Space) picks, Esc quits.

const UI := preload("res://ui/ui_kit.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

var buttons: Array = []
var index := 0
var title_a: Control
var title_b: Control
var t := 0.0
var preview: Control

const ITEMS := [
	["Play", "res://select.tscn"],
	["Online", "res://online.tscn"],
	["Replays", "res://replays.tscn"],
	["Character Creator", "res://creator.tscn"],
	["Controls", "res://controls.tscn"],
	["Editors", "res://editor.tscn"],
	["Quit", ""],
]


## Launch arguments that mean "play right now": demos, network matches, a chosen content bundle or fighters. Launchers and
## test scripts use these, so they go straight to the game instead of the menu.
const DIRECT_ARGS := ["--demo", "--host", "--join", "--relay", "--content", "--chars", "--shots", "--replay"]


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		for d in DIRECT_ARGS:
			if a.begins_with(d):
				get_tree().change_scene_to_file.call_deferred("res://main.tscn")
				return
	set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(UI.Backdrop.new(false))

	var column := VBoxContainer.new()
	column.position = Vector2(70, 40)
	column.add_theme_constant_override("separation", 8)
	add_child(column)
	title_a = UI.Tag.new("PLATFORM", Vector2(470, 80))
	title_a.fill = UI.SKY
	title_a.font_size = 60
	title_a.edge = UI.INK
	column.add_child(title_a)
	title_b = UI.Tag.new("FIGHTER", Vector2(400, 80))
	title_b.fill = UI.GOLD
	title_b.font_size = 60
	title_b.edge = UI.INK
	title_b.position.x = 40
	var wrap := Control.new()
	wrap.custom_minimum_size = Vector2(0, 88)
	wrap.add_child(title_b)
	column.add_child(wrap)
	var spacer := Control.new()
	spacer.custom_minimum_size = Vector2(0, 12)
	column.add_child(spacer)

	for item in ITEMS:
		var b := UI.Btn.new(item[0], Vector2(400, 50))
		b.activated.connect(_choose.bind(item[1]))
		b.mouse_entered.connect(func(): _select(buttons.find(b)))
		column.add_child(b)
		buttons.append(b)
	_select(0)

	preview = Preview.new(Vector2i(560, 600), true)
	preview.position = Vector2(700, 60)
	preview.set_loadout(Loadout.load_saved(0))
	add_child(preview)

	var hint := UI.Tag.new("Up / Down choose     Enter pick     Esc quit", Vector2(560, 40))
	hint.fill = Color(1, 1, 1, 0.12)
	hint.ink = Color(1, 1, 1, 0.8)
	hint.font_size = 20
	hint.shadow = false
	hint.position = Vector2(60, 676)
	add_child(hint)
	PadNav.attach(self)
	Shot.attach(self)


func _select(i: int) -> void:
	if i < 0:
		return
	index = i
	for k in buttons.size():
		buttons[k].selected = k == index


func _choose(target: String) -> void:
	if target == "":
		get_tree().quit()
	else:
		get_tree().change_scene_to_file(target)


func _process(delta: float) -> void:
	t += delta
	title_a.position.y = sin(t * 1.6) * 3.0
	title_b.position.y = sin(t * 1.6 + 1.2) * 3.0


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_UP, KEY_W:
			_select((index - 1 + buttons.size()) % buttons.size())
		KEY_DOWN, KEY_S:
			_select((index + 1) % buttons.size())
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			_choose(ITEMS[index][1])
		KEY_ESCAPE:
			get_tree().quit()
