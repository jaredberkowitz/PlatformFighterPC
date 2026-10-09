extends SubViewportContainer
## A small 3D stage with one fighter on it, for the menus: the creator's centre piece and the character-select panels.
## It never touches the simulation; it just shows a loadout at a body size.

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Lighting := preload("res://scripts/lighting.gd")

var view: Node3D
var viewport: SubViewport
var spin := true
var t := 0.0
var size_percent := 100.0
var facing_bias := 0.5
var show_floor := true
var floor_node: Node3D
var camera: Camera3D


func _init(vp_size := Vector2i(400, 460), floor := true, distance := 8.2) -> void:
	stretch = true
	custom_minimum_size = Vector2(vp_size)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	show_floor = floor
	viewport = SubViewport.new()
	viewport.own_world_3d = true
	viewport.transparent_bg = true
	viewport.msaa_3d = Viewport.MSAA_4X
	add_child(viewport)
	var root := Node3D.new()
	viewport.add_child(root)
	var env := Environment.new()
	env.background_mode = Environment.BG_CLEAR_COLOR
	var we := WorldEnvironment.new()
	we.environment = env
	root.add_child(we)
	Lighting.apply(env, root, true)
	camera = Camera3D.new()
	camera.fov = 30.0
	camera.position = Vector3(0, 1.45, distance)
	camera.look_at_from_position(camera.position, Vector3(0, 1.25, 0))
	root.add_child(camera)
	if floor:
		var tile := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(3.0, 0.7, 3.0)
		tile.mesh = box
		tile.material_override = FighterView.toon(Color(0.72, 0.58, 0.42), true)
		tile.position = Vector3(0, -0.4, 0)
		root.add_child(tile)
		var grass := MeshInstance3D.new()
		var gbox := BoxMesh.new()
		gbox.size = Vector3(3.0, 0.12, 3.0)
		grass.mesh = gbox
		grass.material_override = FighterView.toon(Color(0.55, 0.82, 0.4), false)
		grass.position = Vector3(0, -0.02, 0)
		root.add_child(grass)
	view = FighterView.new()
	root.add_child(view)
	view.build(0)
	_tidy()


## A menu preview shows only the body: no weapon, damage readout or name.
func _tidy() -> void:
	view.blade_pivot.visible = false
	view.percent_label.visible = false
	view.name_label.visible = false


func set_loadout(l: RefCounted) -> void:
	view.rebuild(l)
	_tidy()


func set_expression(e: Dictionary) -> void:
	view.set_expression(e)


## Scales the fighter to a body size (percent of normal). The feet stay on the floor.
func set_size_percent(p: float) -> void:
	size_percent = p
	view.scale = Vector3.ONE * (p / 100.0)


func _process(delta: float) -> void:
	t += delta
	if view.model != null:
		view.model.rotation.y = (sin(t * 0.9) * 0.8 if spin else 0.0) + facing_bias * (0.0 if spin else 1.0)
		# The eyes look the way the fighter is turned.
		view.set_gaze(1 if view.model.rotation.y >= 0.0 else -1)
		# A gentle idle breath so the preview is never dead still.
		view.model.scale = Vector3(1.0 + 0.012 * sin(t * 2.4), 1.0 - 0.02 * sin(t * 2.4), 1.0 + 0.012 * sin(t * 2.4))
