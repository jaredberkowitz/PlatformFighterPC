extends Control
## The Controls screen: shows both players' keyboard keys and lets you rebind them. Up/Down choose an action, Left/Right switch between
## player 1 and player 2, Enter then a key rebinds, Delete resets the player's keys, Esc goes back.
## The Controller page (scripts/pad_bindings.gd): each controller's buttons and options. Up/Down choose a row, Left/Right choose the
## controller (1 to 4), Enter then a button or trigger on that controller rebinds an action, or changes an option (tap jump, the right
## stick, rumble); Delete resets the controller. Defaults: A/Y jump, B attack, X special, triggers shield, bumpers grab.
## The Display page: screen shake, flashes, effects and quality (scripts/visual_settings.gd); there Up/Down choose and Left/Right (or
## Enter) change the setting. Q and E switch pages.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Bindings := preload("res://scripts/bindings.gd")
const PadBindings := preload("res://scripts/pad_bindings.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")
const VisualSettings := preload("res://scripts/visual_settings.gd")

var column := 0
var row := 0
var capturing := false
var cells: Array = []    # cells[player][row] = {"label": Tag, "key": Tag}
var status: Control
var info: Control
## 0 the keys, 1 the controllers, 2 the display settings.
var page := 0
const PAGES := 3
var pad_page: Control
var pad_rows: Array = []   # [{"label", "value"}] per action, then per option
var pad_head: Control
var pad := 0               # the controller being set up (0 to 3)
var pad_row := 0
var pad_capturing := false
## After a button was bound, the menus ignore that controller until all its buttons are let go (so the button just bound does not
## also press something on the screen).
var pad_quiet := false
var key_page: Array = []      # the keys page's pieces (hidden on the display page)
var display_rows: Array = []  # [{"label", "value"}] per setting
var display_page: Control
var setting := 0
var tabs: Array = []
var keybar: Control


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(UI.Backdrop.new(false, 0.38))
	add_child(UI.ScreenTitle.new("Controls"))
	# The two pages' tabs.
	for k in PAGES:
		var tab := UI.Tag.new(["Keys", "Controller", "Display"][k], Vector2(170, 44))
		tab.font_size = 24
		tab.position = Vector2(430 + k * 180, 18)
		add_child(tab)
		tabs.append(tab)

	for p in 2:
		var head := UI.Tag.new("Player %d" % (p + 1), Vector2(420, 44))
		head.fill = UI.RED if p == 0 else UI.BLUE
		head.ink = Color(1, 1, 1)
		head.edge = UI.INK
		head.font_size = 28
		head.position = Vector2(70 + p * 600, 90)
		add_child(head)
		key_page.append(head)
		var box := VBoxContainer.new()
		box.position = Vector2(70 + p * 600, 142)
		box.add_theme_constant_override("separation", 4)
		add_child(box)
		key_page.append(box)
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

	# Where the controllers are set up, on a dark band so it reads over the sky.
	info = UI.Tag.new("Controllers: press Q or E for the Controller page (buttons, tap jump, right stick, rumble)", Vector2(1160, 36))
	info.fill = Color(UI.INK, 0.72)
	info.ink = Color(1, 1, 1, 0.92)
	info.font_size = 18
	info.shadow = false
	info.position = Vector2(60, 592)
	add_child(info)
	key_page.append(info)

	# The controller page: the controller's actions and their buttons, then its options.
	pad_page = Control.new()
	pad_page.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pad_page.visible = false
	add_child(pad_page)
	pad_head = UI.Tag.new("", Vector2(520, 44))
	pad_head.font_size = 28
	pad_head.ink = Color(1, 1, 1)
	pad_head.edge = UI.INK
	pad_head.position = Vector2(380, 84)
	pad_page.add_child(pad_head)
	var rows := []
	for a in PadBindings.ACTIONS:
		rows.append(a[1])
	for o in PadBindings.OPTIONS:
		rows.append(o[1])
	for k in rows.size():
		var label := UI.Tag.new(rows[k], Vector2(260, 34))
		label.fill = UI.CREAM_DARK
		label.font_size = 20
		label.position = Vector2(300, 138 + k * 40 + (14 if k >= PadBindings.ACTIONS.size() else 0))
		pad_page.add_child(label)
		var value := UI.Tag.new("", Vector2(380, 34))
		value.font_size = 21
		value.position = Vector2(570, label.position.y)
		pad_page.add_child(value)
		pad_rows.append({"label": label, "value": value})

	# The display page: each setting's name and its value, and what the chosen one does.
	display_page = Control.new()
	display_page.mouse_filter = Control.MOUSE_FILTER_IGNORE
	display_page.visible = false
	add_child(display_page)
	for k in VisualSettings.ORDER.size():
		var key: String = VisualSettings.ORDER[k]
		var label := UI.Tag.new(VisualSettings.OPTIONS[key].label, Vector2(320, 52))
		label.fill = UI.CREAM_DARK
		label.font_size = 26
		label.position = Vector2(250, 120 + k * 72)
		display_page.add_child(label)
		var value := UI.Tag.new("", Vector2(420, 52))
		value.font_size = 28
		value.position = Vector2(590, 120 + k * 72)
		display_page.add_child(value)
		display_rows.append({"label": label, "value": value})
	var about := UI.Tag.new("", Vector2(900, 44))
	about.fill = Color(UI.INK, 0.72)
	about.ink = Color(1, 1, 1, 0.92)
	about.font_size = 19
	about.shadow = false
	about.position = Vector2(190, 430)
	about.name = "About"
	display_page.add_child(about)
	status = UI.notice(Vector2(60, 632), 1100, 22)
	add_child(status)
	keybar = UI.KeyBar.new([])
	keybar.position = Vector2(60, 676)
	keybar.size = Vector2(860, 30)
	add_child(keybar)
	_show()
	PadNav.attach(self)
	UI.reveal(self)
	Music.of(self).play("menu")
	Shot.attach(self)


## Switches between the keys, controller and display pages.
func _set_page(p: int) -> void:
	page = posmod(p, PAGES)
	capturing = false
	pad_capturing = false
	status.set_text("")
	_show()


func _show() -> void:
	for k in tabs.size():
		tabs[k].fill = UI.GOLD if k == page else UI.CREAM
		tabs[k].highlight = k == page
		tabs[k].queue_redraw()
	for n in key_page:
		n.visible = page == 0
	pad_page.visible = page == 1
	display_page.visible = page == 2
	if page == 0:
		keybar.items = [["W S", "Choose"], ["A D", "Player"], ["Enter", "Rebind"], ["Del", "Reset"], ["Q E", "Page"], ["Esc", "Back"]]
	elif page == 1:
		keybar.items = [["W S", "Choose"], ["A D", "Controller"], ["Enter", "Rebind / change"], ["Del", "Reset"], ["Q E", "Page"], ["Esc", "Back"]]
	else:
		keybar.items = [["W S", "Choose"], ["A D", "Change"], ["Q E", "Page"], ["Esc", "Back"]]
	keybar.queue_redraw()
	_show_pad()
	for k in display_rows.size():
		var key: String = VisualSettings.ORDER[k]
		var here := k == setting
		display_rows[k].value.set_text("◀  %s  ▶" % VisualSettings.text(key))
		display_rows[k].value.fill = UI.GOLD if here else UI.CREAM
		display_rows[k].value.highlight = here
		display_rows[k].label.highlight = here
		display_rows[k].value.queue_redraw()
		display_rows[k].label.queue_redraw()
	(display_page.get_node("About") as Control).set_text(VisualSettings.OPTIONS[VisualSettings.ORDER[setting]].about)
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


## The controller page's rows: each action's buttons, then each option's choice.
func _show_pad() -> void:
	var connected: bool = pad in Input.get_connected_joypads()
	pad_head.set_text("Controller %d%s" % [pad + 1, "" if connected else " (not connected)"])
	pad_head.fill = [UI.RED, UI.BLUE, UI.GOLD, UI.CREAM_DARK][pad]
	pad_head.queue_redraw()
	var n_actions := PadBindings.ACTIONS.size()
	for k in pad_rows.size():
		var here := k == pad_row
		var text := ""
		if k < n_actions:
			var action: String = PadBindings.ACTIONS[k][0]
			text = "press a button..." if (here and pad_capturing) else PadBindings.inputs_text(pad, action)
		else:
			var o: Array = PadBindings.OPTIONS[k - n_actions]
			text = "◀  %s  ▶" % o[2][PadBindings.option(pad, o[0])]
		pad_rows[k].value.set_text(text)
		pad_rows[k].value.fill = UI.GOLD if here else UI.CREAM
		pad_rows[k].value.highlight = here
		pad_rows[k].label.highlight = here
		pad_rows[k].value.queue_redraw()
		pad_rows[k].label.queue_redraw()


## While a controller button is being bound (or was just bound and is still held) the menus leave that controller alone.
func pad_scheme(which: int) -> Dictionary:
	if page == 1 and pad_capturing and which == pad:
		return {}
	if pad_quiet and which == pad:
		var any := false
		for c in JOY_BUTTON_MAX:
			any = any or Input.is_joy_button_pressed(which, c)
		any = any or Input.get_joy_axis(which, JOY_AXIS_TRIGGER_LEFT) > 0.3 or Input.get_joy_axis(which, JOY_AXIS_TRIGGER_RIGHT) > 0.3
		if any:
			return {}
		pad_quiet = false
	return PadNav.DEFAULT


func _bind_pad(code: int) -> void:
	pad_capturing = false
	pad_quiet = true
	var action: String = PadBindings.ACTIONS[pad_row][0]
	if PadBindings.set_input(pad, action, code):
		status.set_text("%s is now %s on controller %d." % [PadBindings.ACTIONS[pad_row][1], PadBindings.input_name(code), pad + 1])
	else:
		status.set_text("%s is kept for the menus. Pick another." % PadBindings.input_name(code))
	_show()


func _input(event: InputEvent) -> void:
	if page == 1 and pad_capturing:
		if event is InputEventJoypadButton and event.pressed and event.device == pad:
			get_viewport().set_input_as_handled()
			_bind_pad(event.button_index)
		elif event is InputEventJoypadMotion and event.device == pad and event.axis_value > 0.6 \
				and (event.axis == JOY_AXIS_TRIGGER_LEFT or event.axis == JOY_AXIS_TRIGGER_RIGHT):
			get_viewport().set_input_as_handled()
			_bind_pad(PadBindings.TRIGGER_LEFT if event.axis == JOY_AXIS_TRIGGER_LEFT else PadBindings.TRIGGER_RIGHT)
		elif event is InputEventKey and event.pressed and not event.echo and event.physical_keycode == KEY_ESCAPE:
			get_viewport().set_input_as_handled()
			pad_capturing = false
			status.set_text("Cancelled.")
			_show()
		return
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
	if event.keycode == KEY_Q:
		_set_page(page - 1)
		return
	if event.keycode == KEY_E or event.keycode == KEY_TAB:
		_set_page(page + 1)
		return
	if page == 1:
		_pad_key(event.keycode)
		return
	if page == 2:
		_display_key(event.keycode)
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


## Keys on the controller page: Up/Down choose a row, Left/Right the controller, Enter rebinds an action (the next button or trigger
## on that controller) or moves an option on, Delete resets the controller, Esc goes back.
func _pad_key(code: int) -> void:
	var n := pad_rows.size()
	var n_actions := PadBindings.ACTIONS.size()
	match code:
		KEY_UP, KEY_W:
			pad_row = (pad_row - 1 + n) % n
		KEY_DOWN, KEY_S:
			pad_row = (pad_row + 1) % n
		KEY_LEFT, KEY_A:
			pad = (pad + PadBindings.PADS - 1) % PadBindings.PADS
		KEY_RIGHT, KEY_D:
			pad = (pad + 1) % PadBindings.PADS
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE:
			if pad_row < n_actions:
				if pad in Input.get_connected_joypads():
					pad_capturing = true
					status.set_text("Press the new button or trigger for %s on controller %d (Esc cancels)." % [PadBindings.ACTIONS[pad_row][1], pad + 1])
				else:
					status.set_text("Controller %d is not connected." % (pad + 1))
			else:
				var o: Array = PadBindings.OPTIONS[pad_row - n_actions]
				PadBindings.set_option(pad, o[0], PadBindings.option(pad, o[0]) + 1)
		KEY_DELETE:
			PadBindings.reset(pad)
			status.set_text("Controller %d is back to the defaults." % (pad + 1))
		KEY_ESCAPE:
			get_tree().change_scene_to_file("res://menu.tscn")
			return
	_show()


## Keys on the display page: Up/Down choose a setting, Left/Right (or Enter, which wraps round) change it, Esc goes back.
func _display_key(code: int) -> void:
	var n := VisualSettings.ORDER.size()
	var key: String = VisualSettings.ORDER[setting]
	var count: int = VisualSettings.OPTIONS[key].choices.size()
	match code:
		KEY_UP, KEY_W:
			setting = (setting - 1 + n) % n
		KEY_DOWN, KEY_S:
			setting = (setting + 1) % n
		KEY_LEFT, KEY_A:
			VisualSettings.set_index(key, VisualSettings.get_index(key) - 1)
		KEY_RIGHT, KEY_D:
			VisualSettings.set_index(key, VisualSettings.get_index(key) + 1)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE:
			VisualSettings.set_index(key, (VisualSettings.get_index(key) + 1) % count)
		KEY_ESCAPE:
			get_tree().change_scene_to_file("res://menu.tscn")
			return
	_show()
