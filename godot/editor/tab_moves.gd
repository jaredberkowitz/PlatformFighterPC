extends RefCounted
## The move editor: pick a moveset and a move slot, edit its timing, hitboxes, motion, projectile and script as plain
## fields, and see the timeline and the hitboxes over the fighter's body as you go. A moveset that `inherit`s another
## shows the moves it takes from its parent; "Edit a copy here" makes it its own.

const MoveView := preload("res://editor/move_view.gd")

var ctx
var page: VBoxContainer
var weapon_pick: OptionButton
var key_list: ItemList
var view: Control
var frame_slider: HSlider
var detail: VBoxContainer
var new_name: LineEdit
var weapon := ""
var key := "jab"
var frame := 0
var building := false

const MOVE_FIELDS := [
	["total_frames", "length (frames)"], ["landing_lag", "landing lag"], ["autocancel_before", "autocancel before"],
	["autocancel_after", "autocancel after"], ["intangible", "intangible frames"], ["helpless_after", "helpless after"],
	["turns_around", "turns around"], ["grabs_ledge", "grabs ledge"], ["counter_strike", "counter strike"],
	["charge_at", "charge at frame"], ["charge_bonus", "charge bonus %"], ["next", "next move (jab chain)"],
	["next_window", "next window"], ["rehit_start", "rehit start"], ["rehit_every", "rehit every"],
]
const HITBOX_FIELDS := ["start", "end", "x", "y", "radius", "damage", "angle", "bkb", "kbg", "priority", "group", "kind", "shield_damage"]
const HITBOX_DEFAULTS := {"start": "5", "end": "7", "x": "2", "y": "1.1", "radius": "0.8", "damage": "5", "angle": "361", "bkb": "30", "kbg": "50"}
const MOTION_FIELDS := ["start", "end", "vx", "vy"]
const OTHER_BLOCKS := {
	"projectile": ["frame", "x", "y", "speed", "life", "end_damage"],
	"reflector": ["start", "end", "x", "y", "radius", "damage_percent", "speed_percent"],
	"counter": ["start", "end", "then", "percent", "min_damage"],
}


func build(tabs: TabContainer, context) -> void:
	ctx = context
	page = VBoxContainer.new()
	page.name = "Moves"
	tabs.add_child(page)

	var top := HBoxContainer.new()
	var l := Label.new()
	l.text = "Moveset"
	top.add_child(l)
	weapon_pick = OptionButton.new()
	weapon_pick.item_selected.connect(func(_i): _choose_weapon(weapon_pick.get_item_text(weapon_pick.selected)))
	top.add_child(weapon_pick)
	new_name = LineEdit.new()
	new_name.placeholder_text = "name for a copy"
	new_name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	top.add_child(new_name)
	var copy := Button.new()
	copy.text = "New moveset (copy)"
	copy.pressed.connect(_copy_weapon)
	top.add_child(copy)
	page.add_child(top)

	var split := HSplitContainer.new()
	split.size_flags_vertical = Control.SIZE_EXPAND_FILL
	split.split_offset = 150
	page.add_child(split)
	key_list = ItemList.new()
	key_list.custom_minimum_size = Vector2(140, 0)
	key_list.item_selected.connect(func(i): _choose_key(key_list.get_item_metadata(i)))
	split.add_child(key_list)

	var right := VBoxContainer.new()
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	split.add_child(right)
	view = MoveView.new()
	view.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	view.frame_changed.connect(func(f):
		frame = f
		if not frame_slider.has_focus():
			frame_slider.set_value_no_signal(f))
	right.add_child(view)
	frame_slider = HSlider.new()
	frame_slider.step = 1
	frame_slider.value_changed.connect(func(v): view.set_frame(int(v)))
	right.add_child(frame_slider)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	right.add_child(scroll)
	detail = VBoxContainer.new()
	detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(detail)
	on_changed()


func weapons() -> Array:
	var out := []
	for s in ctx.editor.sections():
		if s.kind == "weapon":
			out.append(s.name)
	return out


func pick(parts: PackedStringArray) -> void:
	if parts.size() > 0:
		weapon = parts[0]
	if parts.size() > 1:
		key = parts[1]
	on_changed()


func set_frame(f: int) -> void:
	view.set_frame(f)
	frame_slider.set_value_no_signal(f)


func _choose_weapon(name: String) -> void:
	weapon = name
	on_changed()


func _choose_key(k: String) -> void:
	key = k
	on_changed()


func _field(tree: Dictionary, name: String) -> String:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


## Finds `key` in `weapon_name` or in the moveset it inherits from. Returns [move tree or {}, the weapon it was found in].
func _find_move(weapon_name: String, k: String, depth := 0) -> Array:
	var wt: Dictionary = ctx.editor.get_section("weapon", weapon_name)
	if wt.size() == 0 or depth > 8:
		return [{}, ""]
	for it in wt.items:
		if it.t == "block" and it.kind == "move" and it.name == k:
			return [it, weapon_name]
	var parent := _field(wt, "inherit")
	if parent != "":
		return _find_move(parent, k, depth + 1)
	return [{}, ""]


func _fighter_for(weapon_name: String) -> Dictionary:
	for s in ctx.editor.sections():
		if s.kind == "fighter":
			var ft: Dictionary = ctx.editor.get_section("fighter", s.name)
			var w := _field(ft, "weapon")
			if w == "":
				var parent := _field(ft, "inherit")
				if parent != "":
					w = _field(ctx.editor.get_section("fighter", parent), "weapon")
			if w == weapon_name:
				return ft
	return {}


func _param(ft: Dictionary, name: String, fallback: float) -> float:
	if ft.size() == 0:
		return fallback
	var v := _field(ft, name)
	if v == "":
		var parent := _field(ft, "inherit")
		if parent != "":
			return _param(ctx.editor.get_section("fighter", parent), name, fallback)
		return fallback
	return v.to_float()


func on_changed() -> void:
	if building:
		return
	building = true
	var ws := weapons()
	if not ws.has(weapon):
		weapon = ws[0] if ws.size() > 0 else ""
	weapon_pick.clear()
	for w in ws:
		weapon_pick.add_item(w)
	weapon_pick.select(maxi(ws.find(weapon), 0))
	key_list.clear()
	for k in ctx.editor.move_keys():
		var found: Array = _find_move(weapon, k)
		var own: bool = found[1] == weapon
		var text: String = k
		if found[0].size() == 0:
			text += "  (empty)"
		elif not own:
			text += "  (inherited)"
		var i := key_list.add_item(text)
		key_list.set_item_metadata(i, k)
		if found[0].size() == 0:
			key_list.set_item_custom_fg_color(i, Color(1, 1, 1, 0.4))
		if k == key:
			key_list.select(i)
	_rebuild_detail()
	building = false


func _rebuild_detail() -> void:
	for c in detail.get_children():
		detail.remove_child(c)
		c.queue_free()
	var found: Array = _find_move(weapon, key)
	var move: Dictionary = found[0]
	var owner_weapon: String = found[1]
	var ft := _fighter_for(weapon)
	view.set_body(_param(ft, "ecb_half_width", 0.6), _param(ft, "ecb_height", 2.2))
	if move.size() == 0:
		view.set_move({"total": 30, "title": "%s: not defined" % key})
		var b := Button.new()
		b.text = "Define %s" % key
		b.pressed.connect(_define)
		detail.add_child(b)
		return
	var editable: bool = owner_weapon == weapon
	if not editable:
		var note := Label.new()
		note.text = "This move comes from %s." % owner_weapon
		note.modulate = Color(0.7, 0.8, 1.0)
		detail.add_child(note)
		var b := Button.new()
		b.text = "Edit a copy here"
		b.pressed.connect(_copy_move.bind(move))
		detail.add_child(b)
	_feed_view(move)
	frame_slider.max_value = maxi(int(_field(move, "total_frames")), 1)
	frame_slider.set_value_no_signal(frame)

	_section_label("Timing and flags")
	var cols := GridContainer.new()
	cols.columns = 4
	detail.add_child(cols)
	for spec in MOVE_FIELDS:
		var lab := Label.new()
		lab.text = spec[1]
		lab.custom_minimum_size = Vector2(120, 0)
		cols.add_child(lab)
		var le := LineEdit.new()
		le.text = _field(move, spec[0])
		le.custom_minimum_size = Vector2(60, 0)
		le.editable = editable
		le.placeholder_text = "-"
		var name: String = spec[0]
		le.text_submitted.connect(func(t): _set_field(name, t))
		le.focus_exited.connect(func(): _set_field(name, le.text))
		cols.add_child(le)

	_rows("Hitboxes", "hitbox", HITBOX_FIELDS, move, editable, true)
	_rows("Motion", "motion", MOTION_FIELDS, move, editable, false)
	for kind in OTHER_BLOCKS:
		_single_block(kind, OTHER_BLOCKS[kind], move, editable)
	_scripts(move, editable)


func _section_label(text: String) -> void:
	var l := Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", 15)
	l.modulate = Color(1, 0.9, 0.5)
	detail.add_child(l)


func _feed_view(move: Dictionary) -> void:
	var boxes := []
	var motions := []
	var projectile_frame := -1
	for it in move.items:
		if it.t != "block":
			continue
		if it.kind == "hitbox":
			boxes.append(_hitbox_data(it))
		elif it.kind == "motion":
			motions.append({"start": _field(it, "start").to_int(), "end": _field(it, "end").to_int(), "vx": _field(it, "vx").to_float(), "vy": _field(it, "vy").to_float()})
		elif it.kind == "projectile":
			projectile_frame = _field(it, "frame").to_int()
			# The projectile's own hit is shown at its muzzle.
			for sub in it.items:
				if sub.t == "block" and sub.kind == "hitbox":
					var d := _hitbox_data(sub)
					d.start = projectile_frame
					d.end = projectile_frame
					d.x = _field(it, "x").to_float()
					d.y = _field(it, "y").to_float()
					boxes.append(d)
	view.set_move({"total": maxi(_field(move, "total_frames").to_int(), 1), "boxes": boxes, "motions": motions, "projectile_frame": projectile_frame, "title": key})


func _hitbox_data(b: Dictionary) -> Dictionary:
	var kind := _field(b, "kind")
	return {"start": _field(b, "start").to_int(), "end": _field(b, "end").to_int(), "x": _field(b, "x").to_float(), "y": _field(b, "y").to_float(),
		"radius": _field(b, "radius").to_float(), "kind": kind if kind != "" else "normal", "priority": _field(b, "priority").to_int()}


## A table with one row per child block of `kind`: its fields as small text boxes, a delete button, and an add button.
func _rows(title: String, kind: String, fields: Array, move: Dictionary, editable: bool, copy_last: bool) -> void:
	_section_label(title)
	var grid := GridContainer.new()
	grid.columns = fields.size() + 2
	detail.add_child(grid)
	grid.add_child(Label.new())
	for f in fields:
		var h := Label.new()
		h.text = f
		h.add_theme_font_size_override("font_size", 10)
		h.modulate = Color(1, 1, 1, 0.6)
		grid.add_child(h)
	grid.add_child(Label.new())
	var n := 0
	for it in move.items:
		if it.t != "block" or it.kind != kind:
			continue
		var idx := n
		n += 1
		var num := Label.new()
		num.text = "%d" % n
		grid.add_child(num)
		for f in fields:
			var le := LineEdit.new()
			le.text = _field(it, f)
			le.custom_minimum_size = Vector2(38, 0)
			le.add_theme_font_size_override("font_size", 11)
			le.editable = editable
			var fname: String = f
			le.text_submitted.connect(func(t): _set_row(kind, idx, fname, t))
			le.focus_exited.connect(func(): _set_row(kind, idx, fname, le.text))
			grid.add_child(le)
		var del := Button.new()
		del.text = "x"
		del.disabled = not editable
		del.pressed.connect(_delete_row.bind(kind, idx))
		grid.add_child(del)
	var add := Button.new()
	add.text = "Add %s" % kind
	add.disabled = not editable
	add.pressed.connect(_add_row.bind(kind, copy_last))
	detail.add_child(add)


func _single_block(kind: String, fields: Array, move: Dictionary, editable: bool) -> void:
	var existing := {}
	for it in move.items:
		if it.t == "block" and it.kind == kind:
			existing = it
	_section_label(kind.capitalize() + ("" if existing.size() > 0 else " (none)"))
	if existing.size() == 0:
		var add := Button.new()
		add.text = "Add %s" % kind
		add.disabled = not editable
		add.pressed.connect(_add_single.bind(kind, fields))
		detail.add_child(add)
		return
	var cols := GridContainer.new()
	cols.columns = 4
	detail.add_child(cols)
	for f in fields:
		var lab := Label.new()
		lab.text = f
		lab.custom_minimum_size = Vector2(110, 0)
		cols.add_child(lab)
		var le := LineEdit.new()
		le.text = _field(existing, f)
		le.custom_minimum_size = Vector2(60, 0)
		le.editable = editable
		var fname: String = f
		le.text_submitted.connect(func(t): _set_single(kind, fname, t))
		le.focus_exited.connect(func(): _set_single(kind, fname, le.text))
		cols.add_child(le)
	var del := Button.new()
	del.text = "Remove %s" % kind
	del.disabled = not editable
	del.pressed.connect(_remove_single.bind(kind))
	detail.add_child(del)


func _scripts(move: Dictionary, editable: bool) -> void:
	for kind in ["script", "projectile_script"]:
		var text := ""
		var has := false
		for it in move.items:
			if it.t == "raw" and it.kind == kind:
				text = it.text
				has = true
		_section_label(kind.replace("_", " ").capitalize() + ("" if has else " (none)"))
		var te := TextEdit.new()
		te.custom_minimum_size = Vector2(0, 90 if has else 40)
		te.text = text
		te.editable = editable
		te.placeholder_text = "// scripts: see docs/CONTENT.md"
		detail.add_child(te)
		var row := HBoxContainer.new()
		var apply := Button.new()
		apply.text = "Apply script"
		apply.disabled = not editable
		apply.pressed.connect(func(): _set_script(kind, te.text))
		row.add_child(apply)
		detail.add_child(row)


# ---- Editing helpers: all of them rewrite the weapon section and put it back ---------------------------------

## Runs `change` on this move's tree inside the weapon section and puts the weapon back.
func _edit_move(change: Callable) -> void:
	if building:
		return
	var wt: Dictionary = ctx.editor.get_section("weapon", weapon)
	for it in wt.items:
		if it.t == "block" and it.kind == "move" and it.name == key:
			change.call(it)
			ctx.edit(wt)
			return


func _put_field(tree: Dictionary, name: String, value: String) -> void:
	var v := value.strip_edges()
	for i in tree.items.size():
		var it: Dictionary = tree.items[i]
		if it.t == "field" and it.name == name:
			if v == "":
				tree.items.remove_at(i)
			else:
				it.value = v
			return
	if v != "":
		tree.items.insert(0, {"t": "field", "name": name, "value": v})


func _set_field(name: String, text: String) -> void:
	var found: Array = _find_move(weapon, key)
	if found[0].size() > 0 and _field(found[0], name) == text.strip_edges():
		return
	_edit_move(func(m): _put_field(m, name, text))


func _nth_block(move: Dictionary, kind: String, idx: int) -> Dictionary:
	var n := 0
	for it in move.items:
		if it.t == "block" and it.kind == kind:
			if n == idx:
				return it
			n += 1
	return {}


func _set_row(kind: String, idx: int, name: String, text: String) -> void:
	var found: Array = _find_move(weapon, key)
	var cur := _nth_block(found[0], kind, idx)
	if cur.size() > 0 and _field(cur, name) == text.strip_edges():
		return
	_edit_move(func(m):
		var b := _nth_block(m, kind, idx)
		if b.size() > 0:
			_put_field(b, name, text))


func _delete_row(kind: String, idx: int) -> void:
	_edit_move(func(m):
		var n := 0
		for i in m.items.size():
			var it: Dictionary = m.items[i]
			if it.t == "block" and it.kind == kind:
				if n == idx:
					m.items.remove_at(i)
					return
				n += 1)


func _add_row(kind: String, copy_last: bool) -> void:
	_edit_move(func(m):
		var last := {}
		for it in m.items:
			if it.t == "block" and it.kind == kind:
				last = it
		var fresh := {"t": "block", "kind": kind, "name": "", "items": []}
		if last.size() > 0 and copy_last:
			fresh = last.duplicate(true)
		elif kind == "hitbox":
			for f in HITBOX_FIELDS:
				if HITBOX_DEFAULTS.has(f):
					fresh.items.append({"t": "field", "name": f, "value": HITBOX_DEFAULTS[f]})
		else:
			for f in MOTION_FIELDS:
				fresh.items.append({"t": "field", "name": f, "value": "0"})
		# New blocks go after the existing blocks of the same kind.
		var at: int = m.items.size()
		for i in m.items.size():
			if m.items[i].t == "block" and m.items[i].kind == kind:
				at = i + 1
		m.items.insert(at, fresh))


func _add_single(kind: String, fields: Array) -> void:
	_edit_move(func(m):
		var fresh := {"t": "block", "kind": kind, "name": "", "items": []}
		for f in fields:
			fresh.items.append({"t": "field", "name": f, "value": "0"})
		if kind == "projectile":
			fresh.items.append({"t": "block", "kind": "hitbox", "name": "", "items": [
				{"t": "field", "name": "start", "value": "0"}, {"t": "field", "name": "end", "value": "0"},
				{"t": "field", "name": "x", "value": "0"}, {"t": "field", "name": "y", "value": "0"},
				{"t": "field", "name": "radius", "value": "0.5"}, {"t": "field", "name": "damage", "value": "6"},
				{"t": "field", "name": "angle", "value": "361"}, {"t": "field", "name": "bkb", "value": "20"},
				{"t": "field", "name": "kbg", "value": "0"}]})
			_put_in(fresh, "life", "30")
			_put_in(fresh, "speed", "0.3")
		m.items.append(fresh))


func _put_in(tree: Dictionary, name: String, value: String) -> void:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			it.value = value
			return


func _set_single(kind: String, name: String, text: String) -> void:
	var found: Array = _find_move(weapon, key)
	for it in found[0].items:
		if it.t == "block" and it.kind == kind and _field(it, name) == text.strip_edges():
			return
	_edit_move(func(m):
		for it in m.items:
			if it.t == "block" and it.kind == kind:
				_put_field(it, name, text))


func _remove_single(kind: String) -> void:
	_edit_move(func(m):
		for i in m.items.size():
			if m.items[i].t == "block" and m.items[i].kind == kind:
				m.items.remove_at(i)
				return)


func _set_script(kind: String, text: String) -> void:
	_edit_move(func(m):
		for i in m.items.size():
			if m.items[i].t == "raw" and m.items[i].kind == kind:
				if text.strip_edges() == "":
					m.items.remove_at(i)
				else:
					m.items[i].text = text
				return
		if text.strip_edges() != "":
			m.items.append({"t": "raw", "kind": kind, "text": text}))


func _define() -> void:
	var wt: Dictionary = ctx.editor.get_section("weapon", weapon)
	var move := {"t": "block", "kind": "move", "name": key, "items": [
		{"t": "field", "name": "total_frames", "value": "24"},
		{"t": "block", "kind": "hitbox", "name": "", "items": []}]}
	for f in HITBOX_FIELDS:
		if HITBOX_DEFAULTS.has(f):
			move.items[1].items.append({"t": "field", "name": f, "value": HITBOX_DEFAULTS[f]})
	wt.items.append(move)
	ctx.edit(wt)


func _copy_move(move: Dictionary) -> void:
	var wt: Dictionary = ctx.editor.get_section("weapon", weapon)
	wt.items.append(move.duplicate(true))
	ctx.edit(wt)


func _copy_weapon() -> void:
	var name := new_name.text.strip_edges()
	if name == "" or weapon == "":
		ctx.set_status("Give the copy a name first.")
		return
	var tree := {"kind": "weapon", "name": name, "items": [{"t": "field", "name": "inherit", "value": weapon}]}
	ctx.edit(tree)
	weapon = name
	new_name.text = ""
	on_changed()
	ctx.set_status("Made moveset %s, which inherits every move from the one it was copied from. Edit a move to change it." % name)
