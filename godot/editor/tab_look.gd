extends RefCounted
## The look editor: pick a body colour, face, hat, glasses, neckwear and accent colour for each player, see it on
## the preview, and save it. A look is a few bytes (loadout.gd); the game loads it at start-up and sends it to the
## other player in the network handshake.

const Loadout := preload("res://scripts/loadout.gd")

var ctx
var loadout: RefCounted
var player_slot := 0
var options := {}   # slot name -> OptionButton
var swatches := {}  # slot name -> Array of Buttons
var code_edit: LineEdit
var slot_buttons: Array[Button] = []


func build(tabs: TabContainer, context) -> void:
	ctx = context
	loadout = Loadout.load_saved(0)
	var page := VBoxContainer.new()
	page.name = "Look"
	page.add_theme_constant_override("separation", 8)
	tabs.add_child(page)

	var who := HBoxContainer.new()
	var who_label := Label.new()
	who_label.text = "Editing"
	who.add_child(who_label)
	var group := ButtonGroup.new()
	for i in 2:
		var b := Button.new()
		b.text = "Player %d" % (i + 1)
		b.toggle_mode = true
		b.button_group = group
		b.button_pressed = i == 0
		b.pressed.connect(_choose_player.bind(i))
		who.add_child(b)
		slot_buttons.append(b)
	page.add_child(who)

	for slot in Loadout.SLOTS:
		var row := HBoxContainer.new()
		var label := Label.new()
		label.text = slot.capitalize()
		label.custom_minimum_size = Vector2(80, 0)
		row.add_child(label)
		if slot == "color" or slot == "accent":
			var grid := GridContainer.new()
			grid.columns = 6
			swatches[slot] = []
			for i in Loadout.PALETTE.size():
				var b := Button.new()
				b.custom_minimum_size = Vector2(30, 30)
				b.toggle_mode = true
				b.tooltip_text = Loadout.PALETTE_NAMES[i]
				b.add_theme_stylebox_override("normal", _swatch_style(Loadout.PALETTE[i], false))
				b.add_theme_stylebox_override("hover", _swatch_style(Loadout.PALETTE[i], false))
				b.add_theme_stylebox_override("pressed", _swatch_style(Loadout.PALETTE[i], true))
				b.pressed.connect(_pick.bind(slot, i))
				grid.add_child(b)
				swatches[slot].append(b)
			row.add_child(grid)
		else:
			var opt := OptionButton.new()
			for n in Loadout.slot_names(slot):
				opt.add_item(n)
			opt.item_selected.connect(_pick.bind(slot))
			opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			row.add_child(opt)
			options[slot] = opt
		page.add_child(row)

	var expr := HBoxContainer.new()
	var expr_label := Label.new()
	expr_label.text = "Preview face"
	expr_label.custom_minimum_size = Vector2(80, 0)
	expr.add_child(expr_label)
	var idle := Button.new()
	idle.text = "Normal"
	idle.pressed.connect(func(): ctx.view.set_expression(Loadout.FACES[loadout.face]))
	expr.add_child(idle)
	var hurt := Button.new()
	hurt.text = "Hurt"
	hurt.pressed.connect(func(): ctx.view.set_expression(Loadout.HURT))
	expr.add_child(hurt)
	page.add_child(expr)

	var actions := HBoxContainer.new()
	for spec in [["Save", _save], ["Random", _random], ["Reset", _reset]]:
		var b := Button.new()
		b.text = spec[0]
		b.pressed.connect(spec[1])
		actions.add_child(b)
	page.add_child(actions)

	var share := HBoxContainer.new()
	code_edit = LineEdit.new()
	code_edit.placeholder_text = "share code"
	code_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	share.add_child(code_edit)
	var copy := Button.new()
	copy.text = "Copy"
	copy.pressed.connect(func():
		DisplayServer.clipboard_set(loadout.to_code())
		ctx.set_status("Share code copied: " + loadout.to_code()))
	share.add_child(copy)
	var use := Button.new()
	use.text = "Use code"
	use.pressed.connect(func(): apply_code(code_edit.text))
	share.add_child(use)
	page.add_child(share)

	var note := Label.new()
	note.text = "Looks are cosmetic: they never affect how the game plays, and a missing part falls back to a default."
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	note.modulate = Color(1, 1, 1, 0.65)
	page.add_child(note)
	_refresh()


func _swatch_style(c: Color, selected: bool) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = c
	sb.set_corner_radius_all(15)
	sb.set_border_width_all(4 if selected else 1)
	sb.border_color = Color(1, 1, 1) if selected else Color(0, 0, 0, 0.5)
	return sb


func _choose_player(i: int) -> void:
	player_slot = i
	loadout = Loadout.load_saved(i)
	_refresh()
	ctx.set_status("Editing player %d's look." % (i + 1))


func _pick(a, b = null) -> void:
	# Bound as (slot, index) for swatches and (index, slot) for option buttons: normalise.
	var slot: String = a if a is String else b
	var index: int = b if a is String else a
	loadout.set_slot(slot, index)
	_refresh()
	ctx.set_status("")


func _refresh() -> void:
	for slot in options:
		(options[slot] as OptionButton).select(loadout.get_slot(slot))
	for slot in swatches:
		for i in swatches[slot].size():
			swatches[slot][i].set_pressed_no_signal(i == loadout.get_slot(slot))
	if code_edit != null:
		code_edit.text = loadout.to_code()
	ctx.show_look(loadout)


func _save() -> void:
	var ok: bool = loadout.save(player_slot)
	ctx.set_status("Saved player %d's look." % (player_slot + 1) if ok else "Could not save the look.")


func _random() -> void:
	for slot in Loadout.SLOTS:
		loadout.set_slot(slot, randi() % Loadout.slot_size(slot))
	_refresh()


func _reset() -> void:
	loadout = Loadout.default_for(player_slot)
	_refresh()


## Applies a share code (or any text: a bad one gives the default look rather than an error).
func apply_code(code: String) -> void:
	loadout = Loadout.from_code(code, player_slot)
	_refresh()
