extends RefCounted
## The fighter editor: every physics number of a fighter, grouped, with the moveset it uses. Edit a value and the
## game's validator checks it straight away. "New fighter" copies the one on screen (the copy `inherit`s, so it only
## stores what you change).

var ctx
var page: VBoxContainer
var fighter_pick: OptionButton
var weapon_pick: OptionButton
var grid: VBoxContainer
var new_name: LineEdit
var current := ""
var inputs := {}   # param name -> LineEdit
var building := false

const GROUPS := [
	["Body", ["ecb_", "weight"]],
	["Ground", ["walk_", "run_", "dash_", "turn_", "ground_", "landing_"]],
	["Jumps", ["jump_", "hop_", "full_hop", "short_hop", "air_jump"]],
	["Air", ["air_speed", "air_accel", "air_friction", "gravity", "max_fall", "fast_fall"]],
	["Air dodge and wavedash", ["air_dodge", "ground_assist", "wavedash", "waveland"]],
	["Shield, rolls, dodges", ["shield_drop", "roll_", "spot_", "platform_ignore"]],
	["Ledges", ["ledge_"]],
	["Other", []],
]


func build(tabs: TabContainer, context) -> void:
	ctx = context
	page = VBoxContainer.new()
	page.name = "Fighter"
	tabs.add_child(page)

	var top := HBoxContainer.new()
	var l1 := Label.new()
	l1.text = "Fighter"
	top.add_child(l1)
	fighter_pick = OptionButton.new()
	fighter_pick.item_selected.connect(func(_i): _select(fighter_pick.get_item_text(fighter_pick.selected)))
	top.add_child(fighter_pick)
	var l2 := Label.new()
	l2.text = "  Moveset"
	top.add_child(l2)
	weapon_pick = OptionButton.new()
	weapon_pick.item_selected.connect(_weapon_chosen)
	top.add_child(weapon_pick)
	page.add_child(top)

	var make := HBoxContainer.new()
	new_name = LineEdit.new()
	new_name.placeholder_text = "name for a copy"
	new_name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	make.add_child(new_name)
	var b := Button.new()
	b.text = "New fighter (copy)"
	b.pressed.connect(_copy)
	make.add_child(b)
	var del := Button.new()
	del.text = "Delete"
	del.pressed.connect(_delete)
	make.add_child(del)
	page.add_child(make)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	page.add_child(scroll)
	grid = VBoxContainer.new()
	grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(grid)
	on_changed()


func fighters() -> Array:
	var out := []
	for s in ctx.editor.sections():
		if s.kind == "fighter":
			out.append(s.name)
	return out


func weapons() -> Array:
	var out := []
	for s in ctx.editor.sections():
		if s.kind == "weapon":
			out.append(s.name)
	return out


func pick(parts: PackedStringArray) -> void:
	if parts.size() > 0:
		_select(parts[0])


func on_changed() -> void:
	if building:
		return
	building = true
	var names := fighters()
	if not names.has(current):
		current = names[0] if names.size() > 0 else ""
	fighter_pick.clear()
	for n in names:
		fighter_pick.add_item(n)
	fighter_pick.select(maxi(names.find(current), 0))
	var tree: Dictionary = ctx.editor.get_section("fighter", current)
	weapon_pick.clear()
	var ws := weapons()
	for w in ws:
		weapon_pick.add_item(w)
	if tree.size() > 0:
		var w := _field(tree, "weapon")
		if w == "":
			w = _inherited_weapon(tree)
		weapon_pick.select(maxi(ws.find(w), 0))
	_rebuild_grid(tree)
	building = false


func _field(tree: Dictionary, name: String) -> String:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


func _inherited_weapon(tree: Dictionary) -> String:
	var parent := _field(tree, "inherit")
	if parent == "":
		return ""
	var pt: Dictionary = ctx.editor.get_section("fighter", parent)
	return _field(pt, "weapon") if pt.size() > 0 else ""


## The value shown for a parameter: the fighter's own, or what it inherits (shown dimmed).
func _value(tree: Dictionary, name: String) -> Array:
	var own := _field(tree, name)
	if own != "":
		return [own, true]
	var parent := _field(tree, "inherit")
	if parent != "":
		var pt: Dictionary = ctx.editor.get_section("fighter", parent)
		if pt.size() > 0:
			var v: Array = _value(pt, name)
			return [v[0], false]
	return ["", false]


func _group_of(name: String) -> int:
	for gi in GROUPS.size() - 1:
		for prefix in GROUPS[gi][1]:
			if name.begins_with(prefix):
				return gi
	return GROUPS.size() - 1


func _rebuild_grid(tree: Dictionary) -> void:
	for c in grid.get_children():
		grid.remove_child(c)
		c.queue_free()
	inputs.clear()
	if tree.size() == 0:
		return
	var by_group := []
	for g in GROUPS:
		by_group.append([])
	for n in ctx.editor.param_names():
		by_group[_group_of(n)].append(n)
	for gi in GROUPS.size():
		if by_group[gi].is_empty():
			continue
		var head := Label.new()
		head.text = GROUPS[gi][0]
		head.add_theme_font_size_override("font_size", 16)
		head.modulate = Color(1, 0.9, 0.5)
		grid.add_child(head)
		var cols := GridContainer.new()
		cols.columns = 4
		grid.add_child(cols)
		for n in by_group[gi]:
			var lab := Label.new()
			lab.text = n
			lab.custom_minimum_size = Vector2(150, 0)
			lab.tooltip_text = n
			cols.add_child(lab)
			var le := LineEdit.new()
			var v: Array = _value(tree, n)
			le.text = v[0]
			le.custom_minimum_size = Vector2(70, 0)
			le.modulate = Color(1, 1, 1) if v[1] else Color(0.7, 0.8, 1.0)
			le.tooltip_text = "" if v[1] else "inherited from %s" % _field(tree, "inherit")
			le.text_submitted.connect(func(t): _commit(n, t))
			le.focus_exited.connect(func(): _commit(n, le.text))
			cols.add_child(le)
			inputs[n] = le


func _commit(name: String, text: String) -> void:
	if building or current == "":
		return
	var tree: Dictionary = ctx.editor.get_section("fighter", current)
	var old := _field(tree, name)
	if text.strip_edges() == old or (old == "" and text == _value(tree, name)[0]):
		return
	_set_value(tree, name, text.strip_edges())
	ctx.edit(tree)


func _set_value(tree: Dictionary, name: String, value: String) -> void:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			it.value = value
			return
	tree.items.append({"t": "field", "name": name, "value": value})


func _select(name: String) -> void:
	current = name
	on_changed()


func _weapon_chosen(_i: int) -> void:
	if building or current == "":
		return
	var tree: Dictionary = ctx.editor.get_section("fighter", current)
	_set_value(tree, "weapon", weapon_pick.get_item_text(weapon_pick.selected))
	ctx.edit(tree)


func _copy() -> void:
	var name := new_name.text.strip_edges()
	if name == "" or current == "":
		ctx.set_status("Give the copy a name first.")
		return
	var tree := {"kind": "fighter", "name": name, "items": []}
	_set_value(tree, "inherit", current)
	ctx.edit(tree)
	current = name
	new_name.text = ""
	on_changed()
	ctx.set_status("Made %s, which inherits from the fighter it was copied from." % name)


func _delete() -> void:
	if current == "" or fighters().size() <= 1:
		ctx.set_status("A roster needs at least one fighter.")
		return
	var gone := current
	ctx.remove("fighter", gone)
	ctx.set_status("Deleted %s." % gone)
