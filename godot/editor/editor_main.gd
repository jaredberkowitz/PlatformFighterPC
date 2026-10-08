extends Control
## The content editors (plan 7.6), built inside Godot on the same simulation the game runs. This is the shell: a toolbar,
## a tab per editor on the left, a live 3D preview on the right, and a problems list that shows what the game's own
## validator says about the content as you edit it.
##
## Every editor works on a `ContentEditor` (the bridge): sections are read as trees, changed, and put back, and Rust
## re-checks the whole document. The editors have no rules of their own, so what they accept is what the game accepts.
##
## Run:  Godot --path godot res://editor.tscn      (play_editor.bat does this)
## Test hooks (after `--`):  --look=<hex code>  --tab=<name>  --hurt  --open=<bundle>  --pick=<kind:name>
##                           --frame=<n>  --shot=<png path> (save a screenshot and quit)

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const LookTab := preload("res://editor/tab_look.gd")
const FighterTab := preload("res://editor/tab_fighter.gd")
const MovesTab := preload("res://editor/tab_moves.gd")
const StageTab := preload("res://editor/tab_stage.gd")
const PackTab := preload("res://editor/tab_package.gd")

var editor: RefCounted      # ContentEditor (bridge)
var sim: Node               # SimRunner (bridge), used to look at the content as the game sees it
var tab_modules: Array = []
var tabs: TabContainer
var status: Label
var problems: RichTextLabel
var view: Node3D
var preview_root: Node3D
var preview_t := 0.0
var spin := true
var shot_path := ""
var shot_frames := 0
var undo_button: Button
var redo_button: Button
var look_tab


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var args := OS.get_cmdline_user_args()
	if not ClassDB.class_exists("ContentEditor"):
		var l := Label.new()
		l.text = "The Rust bridge did not load.\nBuild it first:  cargo build -p godot-bridge"
		add_child(l)
		return
	editor = ClassDB.instantiate("ContentEditor")
	sim = ClassDB.instantiate("SimRunner")
	add_child(sim)
	editor.new_from_builtin("My Pack")

	var split := HSplitContainer.new()
	split.set_anchors_preset(Control.PRESET_FULL_RECT)
	split.split_offset = 700
	add_child(split)

	var left := VBoxContainer.new()
	left.custom_minimum_size = Vector2(680, 0)
	split.add_child(left)
	_toolbar(left)
	tabs = TabContainer.new()
	tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_child(tabs)
	problems = RichTextLabel.new()
	problems.bbcode_enabled = true
	problems.fit_content = false
	problems.scroll_active = true
	problems.custom_minimum_size = Vector2(0, 90)
	left.add_child(problems)
	status = Label.new()
	status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	left.add_child(status)

	var container := SubViewportContainer.new()
	container.stretch = true
	container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	split.add_child(container)
	var vp := SubViewport.new()
	vp.own_world_3d = true
	vp.msaa_3d = Viewport.MSAA_4X
	container.add_child(vp)
	_build_preview(vp)

	look_tab = LookTab.new()
	look_tab.build(tabs, self)
	for module in [FighterTab, MovesTab, StageTab, PackTab]:
		var m = module.new()
		m.build(tabs, self)
		tab_modules.append(m)
	for a in args:
		if a.begins_with("--open="):
			open_bundle(a.substr(7))
		elif a.begins_with("--look="):
			look_tab.apply_code(a.substr(7))
		elif a.begins_with("--tab="):
			for i in tabs.get_tab_count():
				if tabs.get_tab_title(i).to_lower() == a.substr(6).to_lower():
					tabs.current_tab = i
		elif a.begins_with("--pick="):
			var parts := a.substr(7).split(":")
			for m in tab_modules:
				if m.has_method("pick"):
					m.pick(parts)
		elif a.begins_with("--frame="):
			for m in tab_modules:
				if m.has_method("set_frame"):
					m.set_frame(int(a.substr(8)))
		elif a == "--hurt":
			view.set_expression(Loadout.HURT)
		elif a.begins_with("--shot="):
			shot_path = a.substr(7)
	if shot_path != "":
		spin = false
	_show_problems()


func _toolbar(parent: Control) -> void:
	var bar := HBoxContainer.new()
	var title := Label.new()
	title.text = "Editors"
	title.add_theme_font_size_override("font_size", 20)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(title)
	undo_button = Button.new()
	undo_button.text = "Undo"
	undo_button.pressed.connect(func():
		editor.undo()
		changed("Undone."))
	bar.add_child(undo_button)
	redo_button = Button.new()
	redo_button.text = "Redo"
	redo_button.pressed.connect(func():
		editor.redo()
		changed("Redone."))
	bar.add_child(redo_button)
	parent.add_child(bar)


func _build_preview(vp: SubViewport) -> void:
	preview_root = Node3D.new()
	vp.add_child(preview_root)
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.55, 0.78, 0.96)
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(0.75, 0.78, 0.9)
	env.ambient_light_energy = 0.8
	var we := WorldEnvironment.new()
	we.environment = env
	preview_root.add_child(we)
	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-40, 30, 0)
	preview_root.add_child(sun)
	var cam := Camera3D.new()
	cam.fov = 30.0
	cam.position = Vector3(0, 1.45, 8.0)
	cam.look_at_from_position(cam.position, Vector3(0, 1.3, 0))
	preview_root.add_child(cam)
	var floor_mesh := CylinderMesh.new()
	floor_mesh.top_radius = 1.6
	floor_mesh.bottom_radius = 1.6
	floor_mesh.height = 0.1
	var fm := MeshInstance3D.new()
	fm.mesh = floor_mesh
	fm.material_override = FighterView.toon(Color(0.62, 0.9, 0.5), false)
	fm.position = Vector3(0, -0.06, 0)
	preview_root.add_child(fm)
	view = FighterView.new()
	preview_root.add_child(view)
	view.build(0)


## Shows `l` on the preview fighter.
func show_look(l: RefCounted) -> void:
	view.rebuild(l)
	# The weapon is not part of a look; keep it out of the preview.
	view.blade_pivot.visible = false


func set_status(text: String) -> void:
	status.text = text


# ---- Editing: what every tab calls ---------------------------------------------------------------------------

## Puts a changed section back. Returns true if the document is valid afterwards.
func edit(section: Dictionary) -> bool:
	var errors: PackedStringArray = editor.put_section(section)
	changed("")
	return errors.is_empty()


func remove(kind: String, name: String) -> bool:
	var errors: PackedStringArray = editor.remove_section(kind, name)
	changed("")
	return errors.is_empty()


## Everything that depends on the document is refreshed: the problems list, the game's view of the content, the tabs.
func changed(message: String) -> void:
	_show_problems()
	if editor.is_valid():
		sim.load_content_text(editor.text(false))
	if message != "":
		set_status(message)
	for m in tab_modules:
		if m.has_method("on_changed"):
			m.on_changed()


func open_bundle(path: String) -> String:
	var err: String = editor.open_file(path)
	if err == "":
		changed("Opened %s" % path)
	else:
		set_status(err)
	return err


func new_document() -> void:
	editor.new_from_builtin("My Pack")
	changed("Started from the built-in roster.")


func _show_problems() -> void:
	undo_button.disabled = not editor.can_undo()
	redo_button.disabled = not editor.can_redo()
	if editor.is_valid():
		problems.text = "[color=#7fd67f]The content is valid.[/color]"
		return
	var lines := PackedStringArray()
	for e in editor.errors():
		lines.append("[color=#ff8a7a]%s[/color]" % e.replace("[", "(").replace("]", ")"))
	problems.text = "\n".join(lines)


func _process(delta: float) -> void:
	preview_t += delta
	if spin and view != null and view.model != null:
		view.model.rotation.y = sin(preview_t * 0.9) * 0.9
	elif view != null and view.model != null and shot_path != "":
		view.model.rotation.y = 0.5
	if shot_path != "":
		shot_frames += 1
		if shot_frames == 20:
			var img := get_viewport().get_texture().get_image()
			img.save_png(shot_path)
			get_tree().quit()
