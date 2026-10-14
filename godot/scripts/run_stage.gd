extends Node3D
## The run's streamed stage, drawn (presentation only; the sim streams it, `sim_core::run`): each room in play is built once, from the
## stage art's blocks and platforms in the city's look, when it comes into play and freed when it leaves; a shop room gets a
## storefront at its door and a rest stop a bench sign. The left wall (the closed gate behind the oldest room kept) and an arena's
## exit gate are drawn where the sim's walls stand. The city's backdrop rides along behind the view.
## Placeholder until the art director's city kit (see docs/modes/RUNNER_PLAN.md, the art list).

const StageArt := preload("res://scripts/stage_art.gd")
const StageView := preload("res://scripts/stage_view.gd")

const ROOM_SHOP := 2
const ROOM_REST := 3
## How tall the walls are drawn (the sim's reach far above the view).
const WALL_TOP := 30.0
const WALL_BOTTOM := -8.0

var theme := {}
## Built rooms by their left edge (a string key): the Node3D holding their pieces.
var pieces := {}
var layout_seen := -1
var backdrop: Node3D
## The walls: the left gate and the arena's exit gate.
var wall_node: Node3D
var gate_node: Node3D
## The arena gate's bars (for the opening slide) and where it stands.
var gate_x := 0.0
var gate_shut := false
var gate_open_t := 1.0


func setup(t: Dictionary) -> void:
	theme = t
	for c in get_children():
		c.queue_free()
	pieces.clear()
	layout_seen = -1
	backdrop = Node3D.new()
	backdrop.name = "Backdrop"
	add_child(backdrop)
	var scenery := Node3D.new()
	backdrop.add_child(scenery)
	# (The city's skyline, following the camera across. Its own stage is given just below the street, so the props that stand on a
	# stage sit hidden behind the run's buildings: the run's rooms are the stage.)
	StageArt.backdrop(scenery, theme, [-15.0, 15.0, -9.0, -24.0, -15.0, 15.0])
	wall_node = _gate(Color(0.32, 0.3, 0.36), true)
	add_child(wall_node)
	gate_node = _gate(Color(0.62, 0.22, 0.2), false)
	gate_node.visible = false
	add_child(gate_node)


## A gate: a dark pillar of brick with iron bars across the street (placeholder).
func _gate(colour: Color, solid: bool) -> Node3D:
	var root := Node3D.new()
	root.name = "Gate"
	var mat := StandardMaterial3D.new()
	mat.albedo_color = colour
	mat.roughness = 0.9
	var pillar := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(1.0, WALL_TOP - WALL_BOTTOM, 3.4)
	pillar.mesh = box
	pillar.material_override = mat
	pillar.position = Vector3(-0.5, (WALL_TOP + WALL_BOTTOM) * 0.5, -0.6)
	if solid:
		root.add_child(pillar)
	var bar_mat := StandardMaterial3D.new()
	bar_mat.albedo_color = Color(0.14, 0.13, 0.16)
	bar_mat.metallic = 0.6
	bar_mat.roughness = 0.4
	var bars := Node3D.new()
	bars.name = "Bars"
	root.add_child(bars)
	for n in 5:
		var bar := MeshInstance3D.new()
		var c := CylinderMesh.new()
		c.top_radius = 0.09
		c.bottom_radius = 0.09
		c.height = 9.0
		bar.mesh = c
		bar.material_override = bar_mat
		bar.position = Vector3(-0.5 if solid else 0.5, 4.5, 1.1 - n * 0.55)
		bars.add_child(bar)
	for y in [1.0, 4.5, 8.0]:
		var rail := MeshInstance3D.new()
		var rb := BoxMesh.new()
		rb.size = Vector3(0.14, 0.18, 2.4)
		rail.mesh = rb
		rail.material_override = bar_mat
		rail.position = Vector3(-0.5 if solid else 0.5, y, 0.0)
		bars.add_child(rail)
	if not solid:
		# (An arena's gate: a red frame over the bars, so it reads as the way out.)
		var frame := MeshInstance3D.new()
		var fb := BoxMesh.new()
		fb.size = Vector3(0.5, 0.5, 3.0)
		frame.mesh = fb
		frame.material_override = mat
		frame.position = Vector3(0.5, 9.2, 0.0)
		root.add_child(frame)
	return root


## A storefront at a shop's door (placeholder): a lit window box, a door and an awning colour band, with a sign.
func _storefront(parent: Node3D, door_x: float, door_y: float) -> void:
	var front := Node3D.new()
	front.name = "Storefront"
	front.position = Vector3(door_x, door_y, -0.9)
	parent.add_child(front)
	var wall_mat := StandardMaterial3D.new()
	wall_mat.albedo_color = Color(0.86, 0.74, 0.55)
	var shop := MeshInstance3D.new()
	var sb := BoxMesh.new()
	sb.size = Vector3(7.0, 4.6, 0.6)
	shop.mesh = sb
	shop.material_override = wall_mat
	shop.position = Vector3(0, 2.3, -0.4)
	front.add_child(shop)
	var door_mat := StandardMaterial3D.new()
	door_mat.albedo_color = Color(0.36, 0.2, 0.14)
	var door := MeshInstance3D.new()
	var db := BoxMesh.new()
	db.size = Vector3(1.4, 2.6, 0.2)
	door.mesh = db
	door.material_override = door_mat
	door.position = Vector3(0, 1.3, 0.0)
	front.add_child(door)
	var glow := StandardMaterial3D.new()
	glow.albedo_color = Color(1.0, 0.86, 0.45)
	glow.emission_enabled = true
	glow.emission = Color(1.0, 0.8, 0.4)
	glow.emission_energy_multiplier = 0.8
	for side in [-1.0, 1.0]:
		var window := MeshInstance3D.new()
		var wb := BoxMesh.new()
		wb.size = Vector3(1.8, 1.4, 0.2)
		window.mesh = wb
		window.material_override = glow
		window.position = Vector3(side * 2.4, 1.8, 0.0)
		front.add_child(window)
	var label := Label3D.new()
	label.text = "SHOP"
	label.font_size = 96
	label.pixel_size = 0.01
	label.modulate = Color(0.25, 0.12, 0.08)
	label.outline_size = 0
	label.position = Vector3(0, 3.7, 0.0)
	front.add_child(label)


## A rest stop's sign (placeholder).
func _rest_sign(parent: Node3D, x: float) -> void:
	var label := Label3D.new()
	label.text = "REST"
	label.font_size = 80
	label.pixel_size = 0.01
	label.modulate = Color(0.25, 0.55, 0.3)
	label.position = Vector3(x, 5.2, -1.2)
	parent.add_child(label)


## Builds what has come into play and frees what has gone, and places the walls, when the sim's stage has changed (`layout`).
func sync(sim, layout: int) -> void:
	if layout == layout_seen:
		return
	layout_seen = layout
	var wanted := {}
	var rows: Array = sim.run_pieces()
	for r in rows:
		var key := "%.3f" % r[0]
		if not wanted.has(key):
			wanted[key] = []
		wanted[key].append(r)
	var rooms := {}
	for r in sim.run_rooms():
		rooms["%.3f" % r[0]] = r
	for key in pieces.keys():
		if not wanted.has(key):
			pieces[key].queue_free()
			pieces.erase(key)
	for key in wanted.keys():
		if pieces.has(key):
			continue
		var node := Node3D.new()
		node.name = "Room"
		add_child(node)
		pieces[key] = node
		var list: Array = wanted[key]
		for r in list:
			var left: float = r[2]
			var right: float = r[3]
			var top: float = r[4]
			var bottom: float = r[5]
			if r[6] > 0.5:
				StageArt.platform(node, theme, left, right, top, StageView.FRONT_Z)
			else:
				StageArt.solid_block(node, theme, left, right, top, bottom, StageView.FRONT_Z, true)
		if rooms.has(key):
			var room: PackedFloat32Array = rooms[key]
			if int(room[2]) == ROOM_SHOP:
				_storefront(node, room[3], room[4])
			elif int(room[2]) == ROOM_REST:
				_rest_sign(node, (room[0] + room[1]) * 0.5)
	var view: PackedFloat32Array = sim.run_view()
	var info: PackedInt32Array = sim.run_info()
	if view.size() >= 9 and wall_node != null:
		wall_node.position = Vector3(view[6], 0.0, 0.0)
		var shut := info.size() >= 12 and info[11] == 1
		if shut:
			gate_x = view[7]
			gate_node.position = Vector3(gate_x, 0.0, 0.0)
			gate_node.visible = true
			gate_open_t = 0.0
		elif gate_shut:
			# Just cleared: the bars slide up and away (`animate`).
			gate_open_t = 0.0001
		gate_shut = shut


## The arena gate's opening, over about half a second.
func animate(delta: float) -> void:
	if gate_node == null or gate_shut or not gate_node.visible:
		return
	gate_open_t = minf(gate_open_t + delta * 2.0, 1.0)
	var bars: Node3D = gate_node.get_node("Bars")
	bars.position.y = gate_open_t * 9.5
	if gate_open_t >= 1.0:
		gate_node.visible = false
		bars.position.y = 0.0


## Keeps the backdrop behind the view.
func follow(view_centre_x: float) -> void:
	if backdrop != null:
		backdrop.position.x = view_centre_x
