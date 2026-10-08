extends Control
## The main menu: where the game starts. A mosaic of big tiles (Play is the largest, with the last fighter played standing in it).
## Play goes to character select, Online to online play, Replays to saved matches, Character Creator makes fighters, Controls rebinds
## keys, Editors opens the content editors, Quit leaves.
## Up and down (W and S) step through the tiles in order, left and right (A and D) move across, Enter (J, Space) picks, Esc quits.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const Sfx := preload("res://scripts/sfx.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

var buttons: Array = []
var index := 0
var t := 0.0
var preview: Control
var logo: Control

## [title, subtitle, scene, colour, icon, rect (x, y, w, h) on a 1280x720 screen]
const ITEMS := [
	["Play", "Battle on the couch", "res://select.tscn", Color(0.9, 0.3, 0.26), "play", Rect2(50, 150, 520, 500)],
	["Online", "Duels, groups, quick match", "res://online.tscn", Color(0.22, 0.48, 0.92), "online", Rect2(590, 150, 330, 240)],
	["Replays", "Watch saved matches", "res://replays.tscn", Color(0.55, 0.32, 0.82), "replays", Rect2(940, 150, 290, 240)],
	["Character Creator", "Make your own fighter", "res://creator.tscn", Color(0.24, 0.66, 0.38), "creator", Rect2(590, 410, 330, 240)],
	["Controls", "Keys, pads", "res://controls.tscn", Color(0.16, 0.6, 0.64), "controls", Rect2(940, 410, 140, 140)],
	["Editors", "Content", "res://editor.tscn", Color(0.4, 0.44, 0.54), "editors", Rect2(1090, 410, 140, 140)],
	["Quit", "", "", Color(0.32, 0.22, 0.3), "quit", Rect2(940, 565, 290, 85)],
]

## Where left and right go from each tile (index into ITEMS).
const ACROSS := {0: [0, 1], 1: [0, 2], 2: [1, 2], 3: [0, 4], 4: [3, 5], 5: [4, 5], 6: [3, 6]}


## Launch arguments that mean "play right now": demos, network matches, a chosen content bundle or fighters. Launchers and
## test scripts use these, so they go straight to the game instead of the menu.
const DIRECT_ARGS := ["--demo", "--host", "--join", "--relay", "--content", "--chars", "--shots", "--replay"]


class Logo extends Control:
	## The game's name, set in two slanted bars.
	func _draw() -> void:
		var a := Rect2(Vector2(0, 0), Vector2(420, 62))
		D.draw_slant(self, a, Color(0.97, 0.96, 0.92), D.INK, 4.0, true)
		D.draw_text_centered(self, "PLATFORM", a, 50, D.INK)
		var b := Rect2(Vector2(230, 52), Vector2(300, 62))
		D.draw_slant(self, b, D.GOLD, D.INK, 4.0, true)
		D.draw_text_centered(self, "FIGHTER", b, 50, D.INK)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		for d in DIRECT_ARGS:
			if a.begins_with(d):
				get_tree().change_scene_to_file.call_deferred("res://main.tscn")
				return
	set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(UI.Backdrop.new(false))

	logo = Logo.new()
	logo.position = Vector2(56, 16)
	logo.size = Vector2(600, 120)
	logo.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(logo)

	for item in ITEMS:
		var tile := UI.Tile.new(item[0], item[1], item[3], item[4])
		var r: Rect2 = item[5]
		tile.position = r.position
		tile.size = r.size
		var target: String = item[2]
		tile.activated.connect(_choose.bind(target))
		tile.mouse_entered.connect(func(): _select(buttons.find(tile)))
		add_child(tile)
		buttons.append(tile)
		if item[4] == "play":
			# The last fighter played stands in the Play tile (in place of its icon).
			tile.icon = ""
			preview = Preview.new(Vector2i(300, 420), false, 9.0)
			preview.position = Vector2(r.size.x - 330, 10)
			preview.set_loadout(Roster.net_entry().look)
			tile.add_child(preview)
	_select(0)

	var hint := UI.Tag.new("Up / Down: next tile     Left / Right: across     Enter: pick     Esc: quit", Vector2(700, 36))
	hint.fill = Color(1, 1, 1, 0.1)
	hint.ink = Color(1, 1, 1, 0.75)
	hint.font_size = 18
	hint.shadow = false
	hint.position = Vector2(50, 672)
	add_child(hint)
	PadNav.attach(self)
	Music.of(self).play("menu")
	UI.reveal(self)
	Shot.attach(self)


func _select(i: int) -> void:
	if i < 0:
		return
	if i != index:
		Sfx.of(self).play("blip")
	index = i
	for k in buttons.size():
		buttons[k].selected = k == index


func _choose(target: String) -> void:
	Sfx.of(self).play("confirm")
	if target == "":
		get_tree().quit()
	else:
		get_tree().change_scene_to_file(target)


func _process(delta: float) -> void:
	t += delta
	logo.position.y = 16.0 + sin(t * 1.4) * 2.5


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_UP, KEY_W:
			_select((index - 1 + buttons.size()) % buttons.size())
		KEY_DOWN, KEY_S:
			_select((index + 1) % buttons.size())
		KEY_LEFT, KEY_A:
			_select(ACROSS[index][0])
		KEY_RIGHT, KEY_D:
			_select(ACROSS[index][1])
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			_choose(ITEMS[index][2])
		KEY_ESCAPE:
			get_tree().quit()
