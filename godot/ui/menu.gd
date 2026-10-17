extends Control
## The main menu: where the game starts. A mosaic of big tiles (Play is the largest, with the last fighter played standing in it).
## Play goes to character select, Run to the single-player run (choosing a fighter first), Online to online play, Replays to saved matches, Character Creator makes fighters, Controls rebinds
## keys, Editors opens the content editors, Quit leaves.
## Up and down (W and S) step through the tiles in order, left and right (A and D) move across, Enter (J, Space) picks, Esc quits.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
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
	["Play", "Battle on the couch", "res://select.tscn", Color(0.98, 0.74, 0.26), "play", Rect2(50, 150, 520, 380)],
	["Run", "Single player: how far can you go?", "run", Color(0.86, 0.36, 0.3), "run", Rect2(50, 550, 520, 100)],
	["Online", "Duels, groups, quick match", "res://online.tscn", Color(0.24, 0.5, 0.86), "online", Rect2(590, 150, 330, 240)],
	["Replays", "Watch saved matches", "res://replays.tscn", Color(0.5, 0.36, 0.76), "replays", Rect2(940, 150, 290, 240)],
	["Character Creator", "Make your own fighter", "res://creator.tscn", Color(0.3, 0.64, 0.36), "creator", Rect2(590, 410, 330, 240)],
	["Controls", "Keys, pads", "res://controls.tscn", Color(0.25, 0.27, 0.46), "controls", Rect2(940, 410, 135, 135)],
	["Editors", "Content", "res://editor.tscn", Color(0.25, 0.27, 0.46), "editors", Rect2(1095, 410, 135, 135)],
	["Quit", "", "", Color(0.21, 0.22, 0.38), "quit", Rect2(940, 565, 290, 85)],
]

## The destination tiles' painted scenes, in place of a line icon (a template's look): fighters from the roster standing in them, each
## tile in its own light. [sky mood, scene seed, [[built-in fighter, x across the tile 0..1, facing (-1 left .. 1 right)], ...]].
const VIGNETTES := {
	# Two fighters squaring up across a gap: playing someone far away.
	"online": [0.75, 31, [[3, 0.5, 0.7], [6, 0.8, -0.7]]],
	# A fighter turning in the light: one being looked over.
	"replays": [0.55, 17, [[8, 0.72, 0.0]]],
	"creator": [0.15, 9, [[2, 0.74, 0.0]]],
}

## Where left and right go from each tile (index into ITEMS).
const ACROSS := {0: [0, 2], 1: [1, 4], 2: [0, 3], 3: [2, 3], 4: [1, 5], 5: [4, 6], 6: [5, 6], 7: [4, 7]}


## Launch arguments that mean "play right now": demos, network matches, a chosen content bundle or fighters. Launchers and
## test scripts use these, so they go straight to the game instead of the menu.
const DIRECT_ARGS := ["--run", "--demo", "--host", "--join", "--relay", "--content", "--chars", "--shots", "--replay"]


class Logo extends Control:
	## The game's name on two brush strokes: "PLATFORM" in white on ink, "FIGHTER" in ink on gold, with a spark where they cross.
	func _draw() -> void:
		var a := Rect2(Vector2(6, 2), Vector2(380, 64))
		Art.brush_band(self, a, Art.INK, 31, 20.0, true, 1.0)
		Art.title_centered(self, "PLATFORM", a.grow_individual(-10, 0, -20, 0), 62, Color.WHITE, 0.0)
		var b := Rect2(Vector2(220, 56), Vector2(300, 62))
		Art.brush_band(self, b, Art.SUN, 37, 18.0, true, 1.0)
		Art.title_centered(self, "FIGHTER", b.grow_individual(-8, 0, -18, 0), 62, Art.INK, 0.0)
		Art.sparkle(self, Vector2(206, 66), 16.0, Color.WHITE)


func _ready() -> void:
	# The game cannot run without its simulation library (pf_bridge.dll beside the game). Say so plainly instead of failing
	# silently: it is nearly always a zip run without extracting it, or a file left behind.
	if not ClassDB.class_exists("SimRunner"):
		OS.alert("The game's files are incomplete: pf_bridge.dll was not found next to PlatformFighter.exe.\n\n"
			+ "Right-click the downloaded zip, choose \"Extract All\", and run PlatformFighter.exe from the extracted folder.\n"
			+ "(From the source code: run setup.bat first.)", "Platform Fighter")
		get_tree().quit(1)
		return
	for a in OS.get_cmdline_user_args():
		for d in DIRECT_ARGS:
			if a.begins_with(d):
				get_tree().change_scene_to_file.call_deferred("res://runner.tscn" if d == "--run" else "res://main.tscn")
				return
	set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(UI.Backdrop.new(false))

	logo = Logo.new()
	logo.position = Vector2(50, 16)
	logo.size = Vector2(600, 120)
	logo.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(logo)

	for item in ITEMS:
		var tile := UI.Tile.new(item[0], item[1], item[3], item[4])
		# (The utility tiles are quieter: a thinner line, a shallower body and smaller type.)
		tile.tier = 1 if item[4] in ["controls", "editors", "quit"] or item[2] == "run" else 0
		var r: Rect2 = item[5]
		tile.position = r.position
		tile.size = r.size
		var target: String = item[2]
		tile.activated.connect(_choose.bind(target, tile))
		tile.mouse_entered.connect(func(): _select(buttons.find(tile)))
		add_child(tile)
		buttons.append(tile)
		if item[4] == "play":
			# The last fighter played stands in the Play tile (in place of its icon).
			tile.icon = ""
			tile.hero = true
			tile.show_spotlight([[0.5, 0.8]])
			# (Sized to the tile: it shares its column with the Run tile.)
			preview = Preview.new(Vector2i(300, int(r.size.y) - 70), false, 7.2)
			preview.position = Vector2(r.size.x * 0.5 - 150.0, 16)
			preview.set_loadout(Roster.net_entry().look)
			tile.add_layer(preview)
		elif VIGNETTES.has(item[4]):
			_vignette(tile, r, VIGNETTES[item[4]])
	_select(0)

	var keys := UI.KeyBar.new([["W A S D", "Navigate"], ["Enter", "Select"], ["Esc", "Quit"]])
	keys.position = Vector2(50, 668)
	keys.size = Vector2(520, 32)
	add_child(keys)
	PadNav.attach(self)
	Music.of(self).play("menu")
	UI.reveal(self)
	Shot.attach(self)


## Fills a destination tile with its painted scene and its fighters (see VIGNETTES).
func _vignette(tile: Control, r: Rect2, spec: Array) -> void:
	tile.icon = ""
	var people: Array = spec[2]
	var feet := []
	for p in people:
		feet.append([float(p[1]), 0.7])
	tile.show_spotlight(feet)
	var built := Roster.builtins()
	var h := r.size.y * 0.8
	for p in people:
		var w := h * 0.8
		var pv := Preview.new(Vector2i(int(w), int(h)), false, 6.5)
		pv.position = Vector2(r.size.x * float(p[1]) - w * 0.5, 0)
		var k: int = int(p[0]) % maxi(built.size(), 1)
		if built.size() > 0:
			pv.set_loadout(built[k].look)
		if float(p[2]) != 0.0:
			pv.spin = false
			pv.facing_bias = float(p[2])
		tile.add_layer(pv)


func _select(i: int) -> void:
	if i < 0:
		return
	if i != index:
		Sfx.of(self).play("blip")
	index = i
	for k in buttons.size():
		buttons[k].selected = k == index


func _choose(target: String, tile: Control = null) -> void:
	Sfx.of(self).play("confirm")
	if tile == null and index < buttons.size():
		tile = buttons[index]
	# The card sinks into its body for a moment before the screen changes (short: it never holds up navigation).
	if tile != null and tile.has_method("pressed"):
		tile.pressed()
		await get_tree().create_timer(0.09).timeout
	if target == "":
		get_tree().quit()
	elif target == "run":
		# The single-player run: the fighter choice in its run mode, then the run.
		Roster.session = {"mode": "run"}
		get_tree().change_scene_to_file("res://select.tscn")
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
