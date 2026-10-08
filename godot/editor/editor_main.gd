extends Control
## The content editors (plan 7.6), built inside Godot on the same simulation the game runs. This is the shell: a live
## 3D preview on the right and a tab per editor on the left. The look (loadout) editor comes first because it is the
## easiest and safest end of the toolchain: it only changes presentation and can never affect the simulation.
##
## Run:  Godot --path godot res://editor.tscn      (play_editor.bat does this)
## Test hooks (after `--`):  --look=<hex code>  --tab=<n>  --hurt  --shot=<png path> (save a screenshot and quit)

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const LookTab := preload("res://editor/tab_look.gd")

var tabs: TabContainer
var status: Label
var view: Node3D
var preview_root: Node3D
var preview_t := 0.0
var spin := true
var shot_path := ""
var shot_frames := 0
var look_tab


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var args := OS.get_cmdline_user_args()

	var split := HSplitContainer.new()
	split.set_anchors_preset(Control.PRESET_FULL_RECT)
	split.split_offset = 430
	add_child(split)

	var left := VBoxContainer.new()
	left.custom_minimum_size = Vector2(420, 0)
	split.add_child(left)
	var title := Label.new()
	title.text = "Editors"
	title.add_theme_font_size_override("font_size", 22)
	left.add_child(title)
	tabs = TabContainer.new()
	tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_child(tabs)
	status = Label.new()
	status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	status.custom_minimum_size = Vector2(400, 40)
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
	for a in args:
		if a.begins_with("--look="):
			look_tab.apply_code(a.substr(7))
		elif a.begins_with("--tab="):
			tabs.current_tab = int(a.substr(6))
		elif a == "--hurt":
			view.set_expression(Loadout.HURT)
		elif a.begins_with("--shot="):
			shot_path = a.substr(7)
	if shot_path != "":
		spin = false


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
	# A floor disc so the fighter has something to stand on.
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
