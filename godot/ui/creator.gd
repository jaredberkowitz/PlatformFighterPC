extends Control
## The character creator: choose a class, a look and four stats, name the fighter, and save it. Stats change how the
## fighter moves (see sim-content recipe.rs): bigger means heavier, slower, and a lower jump. Saved fighters show up in
## character select. Up/Down choose a row, Left/Right change it, Enter on the name (or Finish) saves, Esc goes back.

const UI := preload("res://ui/ui_kit.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")

const LOOK_TEXT := "How your fighter looks. Looks are only for show: they never change how you play."
const DESCRIPTIONS := {
	"fighter": "Pick one of your saved fighters to edit, or start a brand new one.",
	"class": "",
	"color": "Your body colour. Only for show.",
	"accent": "The colour of your hat band, scarf and neckerchief. Only for show.",
	"size": "Bigger fighters are heavier and harder to launch, but run slower, jump lower and fall faster. Smaller ones are quick and light.",
	"speed": "How fast and responsive you are on the ground and in the air. Quick fighters are a little lighter.",
	"jump": "How high your jumps go.",
	"weight": "Heavier fighters are harder to launch and fall a bit faster.",
	"name": "Name your fighter. It is saved on this computer and shows up in character select.",
}
const BAR_KEYS := ["Size", "Run speed", "Jump height", "Weight", "Fall speed"]

var rows: Array = []          # [{id, node}]
var focus := 0
var selectors := {}
var stat_rows := {}
var name_edit: LineEdit
var preview: Control
var bubble: Control
var bars: Control
var status: Control
var editor: RefCounted
var editing_slug := ""
var saved_list: Array = []
var loading := false
var delete_button: Control


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	add_child(UI.Backdrop.new(false))

	var ribbon := UI.Tag.new("Character Creation", Vector2(620, 58))
	ribbon.fill = UI.SKY
	ribbon.font_size = 32
	ribbon.edge = UI.INK
	ribbon.position = Vector2(0, 14)
	add_child(ribbon)

	var column := VBoxContainer.new()
	column.position = Vector2(40, 84)
	column.add_theme_constant_override("separation", 4)
	add_child(column)

	saved_list = Roster.saved()
	var names := ["New fighter"]
	for s in saved_list:
		names.append(s.name)
	_add_selector(column, "fighter", "Fighter", names)
	_add_selector(column, "class", "Class", Roster.CLASS_NAMES)
	for slot in ["face", "hat", "glasses", "neck", "color", "accent"]:
		_add_selector(column, slot, slot.capitalize(), Loadout.slot_names(slot))
	for stat in ["size", "speed", "jump", "weight"]:
		var r := UI.StatRow.new(stat.capitalize())
		r.changed.connect(func(_v): _changed())
		var id: String = stat
		r.focused.connect(func(): _focus_id(id))
		column.add_child(r)
		stat_rows[stat] = r
		rows.append({"id": stat, "node": r})

	var name_row := HBoxContainer.new()
	name_row.add_theme_constant_override("separation", 6)
	var name_tag := UI.Tag.new("Name", Vector2(150, 44))
	name_tag.fill = UI.CREAM_DARK
	name_tag.font_size = 22
	name_row.add_child(name_tag)
	name_edit = LineEdit.new()
	name_edit.custom_minimum_size = Vector2(300, 44)
	name_edit.max_length = 24
	name_edit.placeholder_text = "type a name"
	name_edit.add_theme_font_override("font", UI.font())
	name_edit.add_theme_font_size_override("font_size", 26)
	name_edit.add_theme_color_override("font_color", UI.INK)
	name_edit.add_theme_color_override("font_placeholder_color", Color(0.4, 0.4, 0.45))
	var sb := StyleBoxFlat.new()
	sb.bg_color = UI.CREAM
	sb.border_color = UI.GOLD
	sb.set_border_width_all(4)
	sb.set_corner_radius_all(6)
	sb.content_margin_left = 14
	name_edit.add_theme_stylebox_override("normal", sb)
	name_edit.add_theme_stylebox_override("focus", sb)
	name_edit.text_submitted.connect(func(_t): _finish())
	name_edit.focus_entered.connect(func(): _focus_id("name", false))
	name_edit.gui_input.connect(_name_key)
	name_row.add_child(name_edit)
	column.add_child(name_row)
	rows.append({"id": "name", "node": name_row})

	preview = Preview.new(Vector2i(420, 480), true)
	preview.position = Vector2(560, 150)
	add_child(preview)

	bubble = UI.Bubble.new(Vector2(600, 120))
	bubble.position = Vector2(660, 16)
	add_child(bubble)

	var how := UI.Tag.new("How it plays", Vector2(260, 40))
	how.fill = UI.CREAM_DARK
	how.font_size = 22
	how.position = Vector2(1000, 160)
	add_child(how)
	bars = UI.Bars.new(Vector2(270, 220))
	bars.position = Vector2(990, 212)
	add_child(bars)

	status = UI.Tag.new("", Vector2(440, 40))
	status.fill = Color(1, 1, 1, 0.0)
	status.ink = UI.GOLD
	status.shadow = false
	status.font_size = 22
	status.position = Vector2(780, 574)
	add_child(status)

	var finish := UI.Btn.new("Finish", Vector2(230, 64))
	finish.position = Vector2(1030, 640)
	finish.activated.connect(_finish)
	add_child(finish)
	var back := UI.Btn.new("Back", Vector2(190, 64))
	back.font_size = 30
	back.position = Vector2(820, 640)
	back.activated.connect(_back)
	add_child(back)
	delete_button = UI.Btn.new("Delete", Vector2(190, 56))
	delete_button.font_size = 26
	delete_button.position = Vector2(620, 646)
	delete_button.activated.connect(_delete)
	add_child(delete_button)

	if Roster.edit_slug != "":
		for i in saved_list.size():
			if saved_list[i].slug == Roster.edit_slug:
				selectors["fighter"].set_index(i + 1, false)
		Roster.edit_slug = ""
	_load_selected()
	_set_focus(0)
	Shot.attach(self)


func _add_selector(parent: Control, id: String, label: String, options: Array) -> void:
	var s := UI.Selector.new(label, options)
	s.changed.connect(func(_i): _on_selector(id))
	s.focused.connect(func(): _focus_id(id))
	parent.add_child(s)
	selectors[id] = s
	rows.append({"id": id, "node": s})


func _on_selector(id: String) -> void:
	if id == "fighter":
		_load_selected()
	else:
		_changed()


## Fills every row from the fighter chosen in the first selector (or from the defaults for "New fighter").
func _load_selected() -> void:
	loading = true
	var i: int = selectors["fighter"].index
	var look: RefCounted
	if i == 0:
		editing_slug = ""
		look = Loadout.default_for(0)
		selectors["class"].set_index(0, false)
		for stat in stat_rows:
			stat_rows[stat].set_value(5, false)
		name_edit.text = ""
	else:
		var e: Dictionary = saved_list[i - 1]
		editing_slug = e.slug
		look = e.look
		selectors["class"].set_index(e["class"], false)
		for stat in stat_rows:
			stat_rows[stat].set_value(e[stat], false)
		name_edit.text = e.name
	for slot in Loadout.SLOTS:
		selectors[slot].set_index(look.get_slot(slot), false)
	delete_button.visible = editing_slug != ""
	loading = false
	_changed()


func current_entry() -> Dictionary:
	var look: RefCounted = Loadout.default_for(0)
	for slot in Loadout.SLOTS:
		look.set_slot(slot, selectors[slot].index)
	var e := Roster.neutral_entry(name_edit.text.strip_edges(), selectors["class"].index, look)
	for stat in stat_rows:
		e[stat] = stat_rows[stat].value
	return e


func _changed() -> void:
	if loading:
		return
	var e := current_entry()
	preview.set_loadout(e.look)
	var r: Dictionary = editor.recipe_readout(e["class"], e.size, e.speed, e.jump, e.weight)
	preview.set_size_percent(float(r.size_percent))
	bars.set_values(Roster.readout_bars(editor, e), BAR_KEYS)
	_describe()
	status.set_text("")


func _describe() -> void:
	var id: String = rows[focus].id
	var text: String = DESCRIPTIONS.get(id, LOOK_TEXT)
	if id == "class":
		var c: int = selectors["class"].index
		text = "%s: %s" % [Roster.CLASS_NAMES[c], Roster.CLASS_BLURBS[c]]
	bubble.set_text(text)


# ---- Focus and keys ------------------------------------------------------------------------------------------------------------

func _focus_id(id: String, release_name := true) -> void:
	for i in rows.size():
		if rows[i].id == id:
			_set_focus(i, release_name)


func _set_focus(i: int, release_name := true) -> void:
	focus = (i + rows.size()) % rows.size()
	for k in rows.size():
		var node = rows[k].node
		if node.has_method("set_focus_mark"):
			node.set_focus_mark(k == focus)
	if rows[focus].id == "name":
		name_edit.grab_focus()
	elif release_name:
		name_edit.release_focus()
	_describe()


func _name_key(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed:
		match event.keycode:
			KEY_UP:
				_set_focus(focus - 1)
				name_edit.accept_event()
			KEY_DOWN:
				_set_focus(focus + 1)
				name_edit.accept_event()
			KEY_ESCAPE:
				name_edit.release_focus()
				_set_focus(focus - 1)
				name_edit.accept_event()


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	var id: String = rows[focus].id
	match event.keycode:
		KEY_UP, KEY_W:
			_set_focus(focus - 1)
		KEY_DOWN, KEY_S:
			_set_focus(focus + 1)
		KEY_LEFT, KEY_A:
			if selectors.has(id):
				selectors[id].step(-1)
			elif stat_rows.has(id):
				stat_rows[id].step(-1)
		KEY_RIGHT, KEY_D:
			if selectors.has(id):
				selectors[id].step(1)
			elif stat_rows.has(id):
				stat_rows[id].step(1)
		KEY_ENTER, KEY_KP_ENTER:
			_finish()
		KEY_ESCAPE:
			_back()


func _back() -> void:
	get_tree().change_scene_to_file("res://menu.tscn")


func _delete() -> void:
	if editing_slug == "":
		return
	Roster.delete(editing_slug)
	get_tree().reload_current_scene()


func _finish() -> void:
	var e := current_entry()
	var problem := Roster.name_problem(e.name, editing_slug)
	if problem != "":
		status.set_text(problem)
		_set_focus(rows.size() - 1)
		return
	var err := Roster.save(e)
	if err != "":
		status.set_text(err)
		return
	# Renaming saves a new fighter and removes the old file.
	if editing_slug != "" and editing_slug != e.slug:
		Roster.delete(editing_slug)
	Roster.session = {"preselect": e.slug}
	get_tree().change_scene_to_file("res://select.tscn")
