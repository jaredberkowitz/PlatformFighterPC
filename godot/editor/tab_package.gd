extends RefCounted
## Packaging: the bundle's name and author, saving it, packing it (adds the content hash and sim version so a changed
## file is detected), opening an existing bundle, and playtesting it in the game with both players on one keyboard.

var ctx
var page: VBoxContainer
var name_edit: LineEdit
var author_edit: LineEdit
var desc_edit: LineEdit
var path_edit: LineEdit
var summary: Label
var building := false


func build(tabs: TabContainer, context) -> void:
	ctx = context
	page = VBoxContainer.new()
	page.name = "Package"
	page.add_theme_constant_override("separation", 8)
	tabs.add_child(page)

	var grid := GridContainer.new()
	grid.columns = 2
	page.add_child(grid)
	name_edit = _row(grid, "Name", "name")
	author_edit = _row(grid, "Author", "author")
	desc_edit = _row(grid, "Description", "description")

	summary = Label.new()
	summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	page.add_child(summary)

	var file_row := HBoxContainer.new()
	var l := Label.new()
	l.text = "File"
	file_row.add_child(l)
	path_edit = LineEdit.new()
	path_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	path_edit.text = ProjectSettings.globalize_path("user://my_pack.pfc")
	file_row.add_child(path_edit)
	page.add_child(file_row)

	var buttons := HBoxContainer.new()
	for spec in [["Open", _open], ["Save", _save.bind(false)], ["Save packed", _save.bind(true)], ["New from built-in", _new], ["Playtest in game", _playtest]]:
		var b := Button.new()
		b.text = spec[0]
		b.pressed.connect(spec[1])
		buttons.add_child(b)
	page.add_child(buttons)

	var help := Label.new()
	help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	help.modulate = Color(1, 1, 1, 0.7)
	help.text = "Save writes a loose bundle you can keep editing (by hand too). Save packed adds a content hash and the sim version: a packed bundle that is edited afterwards is refused, and online both players must have the same one. Playtest opens the game with this content; player 1 uses WASD, player 2 the arrow keys."
	page.add_child(help)
	on_changed()


func _row(grid: GridContainer, label: String, field: String) -> LineEdit:
	var l := Label.new()
	l.text = label
	grid.add_child(l)
	var le := LineEdit.new()
	le.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	le.custom_minimum_size = Vector2(300, 0)
	le.text_submitted.connect(func(t): _set_manifest(field, t))
	le.focus_exited.connect(func(): _set_manifest(field, le.text))
	grid.add_child(le)
	return le


func _field(tree: Dictionary, name: String) -> String:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


func on_changed() -> void:
	building = true
	var m: Dictionary = ctx.editor.manifest()
	if m.size() > 0:
		name_edit.text = _field(m, "name")
		author_edit.text = _field(m, "author")
		desc_edit.text = _field(m, "description")
	var fighters := 0
	var weapons := 0
	for s in ctx.editor.sections():
		if s.kind == "fighter":
			fighters += 1
		elif s.kind == "weapon":
			weapons += 1
	var state := "valid" if ctx.editor.is_valid() else "has problems (see the list below)"
	summary.text = "%d fighters, %d movesets. The content is %s." % [fighters, weapons, state]
	if ctx.editor.path() != "":
		path_edit.text = ctx.editor.path()
	building = false


func _set_manifest(name: String, text: String) -> void:
	if building:
		return
	var m: Dictionary = ctx.editor.manifest()
	if m.size() == 0 or _field(m, name) == text.strip_edges():
		return
	var found := false
	for i in m.items.size():
		var it: Dictionary = m.items[i]
		if it.t == "field" and it.name == name:
			found = true
			if text.strip_edges() == "":
				m.items.remove_at(i)
			else:
				it.value = text.strip_edges()
			break
	if not found and text.strip_edges() != "":
		m.items.append({"t": "field", "name": name, "value": text.strip_edges()})
	ctx.edit(m)


func _open() -> void:
	ctx.open_bundle(path_edit.text)


func _save(pack: bool) -> void:
	var err: String = ctx.editor.save(path_edit.text, pack)
	ctx.set_status(("Saved%s to %s" % [" (packed)" if pack else "", path_edit.text]) if err == "" else err)


func _new() -> void:
	ctx.new_document()


func _playtest() -> void:
	if not ctx.editor.is_valid():
		ctx.set_status("Fix the problems first: the game would not load this content.")
		return
	var path := ProjectSettings.globalize_path("user://playtest.pfc")
	var err: String = ctx.editor.save(path, false)
	if err != "":
		ctx.set_status(err)
		return
	# Player 1 plays the fighter selected in the Fighter tab, against the first other fighter in the roster.
	var fighter_tab = ctx.tab_modules[0]
	var names: Array = fighter_tab.fighters()
	var p1: int = maxi(names.find(fighter_tab.current), 0)
	var p2 := 0 if p1 != 0 else mini(1, names.size() - 1)
	var args := ["--path", ProjectSettings.globalize_path("res://"), "--", "--content=" + path, "--chars=%d,%d" % [p1, p2]]
	var pid := OS.create_process(OS.get_executable_path(), args)
	ctx.set_status(("Started the game: player 1 is %s." % names[p1]) if pid > 0 else "Could not start the game.")
