extends Control
## The Controls screen: shows both players' keyboard keys and lets you rebind them. Up/Down choose an action, Left/Right switch between
## player 1 and player 2, Enter then a key rebinds, Delete resets the player's keys, Esc goes back. Controllers need no setup: the
## left stick or D-pad moves, A jumps, X attacks, B special, bumpers shield, right stick click grabs, the right stick smashes.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Bindings := preload("res://scripts/bindings.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

var column := 0
var row := 0
var capturing := false
var cells: Array = []    # cells[player][row] = {"label": Tag, "key": Tag}
var status: Control
var info: Control


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(UI.Backdrop.new(false))
	var ribbon := UI.Tag.new("Controls", Vector2(520, 58))
	ribbon.fill = UI.SKY
	ribbon.font_size = 32
	ribbon.edge = UI.INK
	ribbon.position = Vector2(0, 14)
	add_child(ribbon)

	for p in 2:
		var head := UI.Tag.new("Player %d" % (p + 1), Vector2(420, 44))
		head.fill = UI.RED if p == 0 else UI.BLUE
		head.ink = Color(1, 1, 1)
		head.edge = UI.INK
		head.font_size = 28
		head.position = Vector2(70 + p * 600, 90)
		add_child(head)
		var box := VBoxContainer.new()
		box.position = Vector2(70 + p * 600, 142)
		box.add_theme_constant_override("separation", 4)
		add_child(box)
		var col := []
		for a in Bindings.ACTIONS:
			var line := HBoxContainer.new()
			line.add_theme_constant_override("separation", 6)
			var label := UI.Tag.new(a[1], Vector2(250, 32))
			label.fill = UI.CREAM_DARK
			label.font_size = 19
			line.add_child(label)
			var key := UI.Tag.new("", Vector2(210, 32))
			key.font_size = 21
			line.add_child(key)
			box.add_child(line)
			col.append({"label": label, "key": key})
		cells.append(col)

	info = UI.Tag.new("Controller: stick or D-pad move, A jump, X attack, B special, bumpers shield, stick click grab, right stick smash", Vector2(1160, 36))
	info.fill = Color(1, 1, 1, 0.12)
	info.ink = Color(1, 1, 1, 0.85)
	info.font_size = 18
	info.shadow = false
	info.position = Vector2(60, 592)
	add_child(info)
	status = UI.Tag.new("", Vector2(1100, 40))
	status.fill = Color(1, 1, 1, 0)
	status.ink = UI.GOLD
	status.shadow = false
	status.font_size = 24
	status.position = Vector2(60, 632)
	add_child(status)
	var hint := UI.Tag.new("Up / Down choose     Left / Right player     Enter rebind     Delete reset     Esc back", Vector2(900, 36))
	hint.fill = Color(1, 1, 1, 0.12)
	hint.ink = Color(1, 1, 1, 0.8)
	hint.font_size = 20
	hint.shadow = false
	hint.position = Vector2(60, 676)
	add_child(hint)
	_show()
	PadNav.attach(self)
	Music.of(self).play("menu")
	Shot.attach(self)


func _show() -> void:
	for p in 2:
		for r in cells[p].size():
			var action: String = Bindings.ACTIONS[r][0]
			var here: bool = p == column and r == row
			cells[p][r].key.set_text("press a key..." if (here and capturing) else Bindings.key_name(Bindings.key(p, action)))
			cells[p][r].key.fill = UI.GOLD if here else UI.CREAM
			cells[p][r].key.highlight = here
			cells[p][r].label.highlight = here
			cells[p][r].key.queue_redraw()
			cells[p][r].label.queue_redraw()


func _input(event: InputEvent) -> void:
	if not capturing or not (event is InputEventKey) or not event.pressed or event.echo:
		return
	# The next key press is the new binding (Esc cancels).
	get_viewport().set_input_as_handled()
	capturing = false
	if event.physical_keycode == KEY_ESCAPE:
		status.set_text("Cancelled.")
	else:
		var action: String = Bindings.ACTIONS[row][0]
		if Bindings.set_key(column, action, event.physical_keycode):
			status.set_text("%s is now %s." % [Bindings.ACTIONS[row][1], Bindings.key_name(event.physical_keycode)])
		else:
			status.set_text("%s is kept for the game's own keys. Pick another." % Bindings.key_name(event.physical_keycode))
	_show()


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_UP, KEY_W:
			row = (row - 1 + Bindings.ACTIONS.size()) % Bindings.ACTIONS.size()
		KEY_DOWN, KEY_S:
			row = (row + 1) % Bindings.ACTIONS.size()
		KEY_LEFT, KEY_A, KEY_RIGHT, KEY_D:
			column = 1 - column
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE:
			capturing = true
			status.set_text("Press the new key for %s (Esc cancels)." % Bindings.ACTIONS[row][1])
		KEY_DELETE:
			Bindings.reset(column)
			status.set_text("Player %d's keys are back to the defaults." % (column + 1))
		KEY_ESCAPE:
			get_tree().change_scene_to_file("res://menu.tscn")
			return
	_show()
