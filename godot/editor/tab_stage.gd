extends RefCounted
## The stage editor: a side view you can drag things around in, plus the same numbers as text boxes. Platforms can be
## solid blocks (walls, a ceiling) or pass-through; ledges are the edges fighters can hang from; spawn points are
## where the four fighters start; the blast zone is the box a fighter must leave to lose a stock.

const Canvas := preload("res://editor/stage_canvas.gd")
const StageArt := preload("res://scripts/stage_art.gd")

var ctx
var page: VBoxContainer
var canvas: Control
var list: ItemList
var props: GridContainer
var name_edit: LineEdit
var blast_inputs := {}
## How the stage looks (presentation only): its backdrop and the sky's colours.
var backdrop_pick: OptionButton
var sky_pickers := {}
var sel_kind := ""
var sel_index := -1
var building := false

const BLAST := ["blast_left", "blast_right", "blast_bottom", "blast_top"]
const FIELDS := {
	"platform": ["left", "right", "y", "bottom", "pass_through"],
	"ledge": ["x", "y", "side"],
	"spawn": ["x", "y"],
}


func build(tabs: TabContainer, context) -> void:
	ctx = context
	page = VBoxContainer.new()
	page.name = "Stage"
	tabs.add_child(page)

	var top := HBoxContainer.new()
	var l := Label.new()
	l.text = "Stage name"
	top.add_child(l)
	name_edit = LineEdit.new()
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_edit.text_submitted.connect(_rename)
	name_edit.focus_exited.connect(func(): _rename(name_edit.text))
	top.add_child(name_edit)
	page.add_child(top)

	# The look: which backdrop is drawn behind the stage, and the sky's colours (the backdrop's own unless changed).
	var look := HBoxContainer.new()
	var bl := Label.new()
	bl.text = "Backdrop"
	look.add_child(bl)
	backdrop_pick = OptionButton.new()
	for n in ctx.editor.stage_backdrops():
		backdrop_pick.add_item(n)
	backdrop_pick.item_selected.connect(func(i): _set_look("backdrop", backdrop_pick.get_item_text(i)))
	look.add_child(backdrop_pick)
	for spec in [["sky_top", "Sky top"], ["sky_bottom", "Horizon"]]:
		var key: String = spec[0]
		var sl := Label.new()
		sl.text = "  " + spec[1]
		look.add_child(sl)
		var cp := ColorPickerButton.new()
		cp.custom_minimum_size = Vector2(48, 0)
		cp.edit_alpha = false
		cp.popup_closed.connect(func(): _set_look(key, cp.color.to_html(false)))
		look.add_child(cp)
		sky_pickers[key] = cp
	var reset := Button.new()
	reset.text = "Backdrop's own sky"
	reset.pressed.connect(func():
		_set_look("sky_top", "")
		_set_look("sky_bottom", ""))
	look.add_child(reset)
	page.add_child(look)

	canvas = Canvas.new()
	canvas.size_flags_vertical = Control.SIZE_EXPAND_FILL
	canvas.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	canvas.edited.connect(func(tree): ctx.edit(tree))
	canvas.selected.connect(func(k, i):
		sel_kind = k
		sel_index = i
		_select_in_list()
		_rebuild_props())
	page.add_child(canvas)

	var zoom := CheckBox.new()
	zoom.text = "Zoom to the stage"
	zoom.toggled.connect(func(on): canvas.set_zoom(on))
	page.add_child(zoom)
	var adders := HBoxContainer.new()
	for spec in [["Add platform", "platform"], ["Add solid block", "block"], ["Add ledge", "ledge"]]:
		var b := Button.new()
		b.text = spec[0]
		b.pressed.connect(_add.bind(spec[1]))
		adders.add_child(b)
	var del := Button.new()
	del.text = "Delete selected"
	del.pressed.connect(_delete)
	adders.add_child(del)
	page.add_child(adders)

	var row := HBoxContainer.new()
	row.custom_minimum_size = Vector2(0, 150)
	page.add_child(row)
	list = ItemList.new()
	list.custom_minimum_size = Vector2(160, 0)
	list.item_selected.connect(func(i):
		var m: Array = list.get_item_metadata(i)
		sel_kind = m[0]
		sel_index = m[1]
		canvas.select(sel_kind, sel_index)
		_rebuild_props())
	row.add_child(list)
	var right := VBoxContainer.new()
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(right)
	props = GridContainer.new()
	props.columns = 4
	right.add_child(props)
	var blast_label := Label.new()
	blast_label.text = "Blast zone"
	blast_label.modulate = Color(1, 0.9, 0.5)
	right.add_child(blast_label)
	var bg := GridContainer.new()
	bg.columns = 4
	right.add_child(bg)
	for n in BLAST:
		var lab := Label.new()
		lab.text = n.replace("blast_", "")
		bg.add_child(lab)
		var le := LineEdit.new()
		le.custom_minimum_size = Vector2(60, 0)
		var key: String = n
		le.text_submitted.connect(func(t): _set_blast(key, t))
		le.focus_exited.connect(func(): _set_blast(key, le.text))
		bg.add_child(le)
		blast_inputs[n] = le
	on_changed()


func _tree() -> Dictionary:
	return ctx.editor.get_section("stage", "")


func _field(b: Dictionary, name: String) -> String:
	for it in b.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


func _blocks(tree: Dictionary, kind: String) -> Array:
	var out := []
	for it in tree.items:
		if it.t == "block" and it.kind == kind:
			out.append(it)
	return out


func pick(parts: PackedStringArray) -> void:
	if parts.size() >= 2:
		sel_kind = parts[0]
		sel_index = int(parts[1])
		canvas.select(sel_kind, sel_index)
		_rebuild_props()


func on_changed() -> void:
	if building:
		return
	building = true
	var tree := _tree()
	canvas.set_tree(tree.duplicate(true))
	canvas.select(sel_kind, sel_index)
	if tree.size() > 0:
		name_edit.text = tree.name
		for n in BLAST:
			blast_inputs[n].text = _field(tree, n)
		var backdrop := _field(tree, "backdrop")
		for i in backdrop_pick.item_count:
			if backdrop_pick.get_item_text(i) == (backdrop if backdrop != "" else "meadow"):
				backdrop_pick.select(i)
		var t := StageArt.theme({"backdrop": backdrop, "sky_top": _field(tree, "sky_top"), "sky_bottom": _field(tree, "sky_bottom")})
		sky_pickers["sky_top"].color = t.sky_top
		sky_pickers["sky_bottom"].color = t.sky_horizon
	list.clear()
	if tree.size() > 0:
		for kind in ["platform", "ledge", "spawn"]:
			var blocks := _blocks(tree, kind)
			for i in blocks.size():
				var label := "%s %d" % [kind, i + 1]
				if kind == "platform":
					label += " (pass-through)" if _field(blocks[i], "pass_through") == "true" else " (solid)"
				var idx := list.add_item(label)
				list.set_item_metadata(idx, [kind, i])
	_select_in_list()
	_rebuild_props()
	building = false


func _select_in_list() -> void:
	for i in list.item_count:
		var m: Array = list.get_item_metadata(i)
		if m[0] == sel_kind and m[1] == sel_index:
			list.select(i)
			return


func _rebuild_props() -> void:
	for c in props.get_children():
		props.remove_child(c)
		c.queue_free()
	if sel_kind == "":
		return
	var blocks := _blocks(_tree(), sel_kind)
	if sel_index >= blocks.size():
		return
	var b: Dictionary = blocks[sel_index]
	for f in FIELDS[sel_kind]:
		var lab := Label.new()
		lab.text = f
		props.add_child(lab)
		var le := LineEdit.new()
		le.text = _field(b, f)
		le.custom_minimum_size = Vector2(70, 0)
		var fname: String = f
		le.text_submitted.connect(func(t): _set_prop(fname, t))
		le.focus_exited.connect(func(): _set_prop(fname, le.text))
		props.add_child(le)


func _set_value(tree: Dictionary, parent: Dictionary, name: String, value: String) -> void:
	for it in parent.items:
		if it.t == "field" and it.name == name:
			it.value = value
			return
	parent.items.append({"t": "field", "name": name, "value": value})


func _set_prop(name: String, text: String) -> void:
	if building or sel_kind == "":
		return
	var tree := _tree()
	var blocks := _blocks(tree, sel_kind)
	if sel_index >= blocks.size() or _field(blocks[sel_index], name) == text.strip_edges():
		return
	_set_value(tree, blocks[sel_index], name, text.strip_edges())
	ctx.edit(tree)


func _set_blast(name: String, text: String) -> void:
	if building:
		return
	var tree := _tree()
	if _field(tree, name) == text.strip_edges():
		return
	_set_value(tree, tree, name, text.strip_edges())
	ctx.edit(tree)


## Sets (or, with an empty value, removes) one of the stage's look fields.
func _set_look(name: String, value: String) -> void:
	if building:
		return
	var tree := _tree()
	if tree.size() == 0 or _field(tree, name) == value:
		return
	if value == "":
		for i in tree.items.size():
			if tree.items[i].t == "field" and tree.items[i].name == name:
				tree.items.remove_at(i)
				break
	else:
		_set_value(tree, tree, name, value)
	ctx.edit(tree)
	ctx.set_status("The stage's look changed (it never affects play).")


func _rename(text: String) -> void:
	var tree := _tree()
	var n := text.strip_edges()
	if building or n == "" or tree.size() == 0 or tree.name == n:
		return
	tree.name = n
	ctx.edit(tree)


func _add(kind: String) -> void:
	var tree := _tree()
	var bb := Rect2(-28, -17, 56, 41)
	var block := {"t": "block", "kind": "platform", "name": "", "items": []}
	match kind:
		"platform":
			block.items = [_f("left", "-3"), _f("right", "3"), _f("y", "6"), _f("bottom", "6"), _f("pass_through", "true")]
		"block":
			block.items = [_f("left", "-3"), _f("right", "3"), _f("y", "4"), _f("bottom", "2"), _f("pass_through", "false")]
		"ledge":
			block = {"t": "block", "kind": "ledge", "name": "", "items": [_f("x", "0"), _f("y", "0"), _f("side", "1")]}
	var at: int = tree.items.size()
	for i in tree.items.size():
		if tree.items[i].t == "block" and tree.items[i].kind == block.kind:
			at = i + 1
	tree.items.insert(at, block)
	sel_kind = block.kind
	sel_index = _blocks(tree, block.kind).size() - 1
	ctx.edit(tree)
	ctx.set_status("Added a %s. Drag it into place or type its numbers." % kind)


func _f(name: String, value: String) -> Dictionary:
	return {"t": "field", "name": name, "value": value}


func _delete() -> void:
	if sel_kind == "" or sel_kind == "spawn":
		ctx.set_status("Select a platform or ledge to delete (a stage needs its four spawn points).")
		return
	var tree := _tree()
	var n := 0
	for i in tree.items.size():
		if tree.items[i].t == "block" and tree.items[i].kind == sel_kind:
			if n == sel_index:
				tree.items.remove_at(i)
				break
			n += 1
	sel_kind = ""
	sel_index = -1
	ctx.edit(tree)
