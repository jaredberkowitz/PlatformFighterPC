extends Node3D
## Builds the stage from sim data: solid blocks, pass-through platforms, ledges and blast zones.

const FighterView := preload("res://scripts/fighter_view.gd")

## Fighters live in the z = 0 plane. Stage geometry is pushed BACK so its front face sits just in front of
## that plane. If a block were centred on z = 0, perspective would make its front face look wider than the
## fighters standing in front of it, and it would cover a fighter hanging from a ledge whenever the camera
## sat toward the middle of the stage.
const FRONT_Z := 0.3

var ledge_markers: Array[MeshInstance3D] = []
var ledge_mats: Array[StandardMaterial3D] = []


func build(sim) -> void:
	for i in sim.platform_count():
		var r: PackedFloat32Array = sim.platform_rect(i)
		var left := r[0]
		var right := r[1]
		var top := r[2]
		var bottom := r[3]
		var pass_through := r[4] > 0.5
		var mi := MeshInstance3D.new()
		var box := BoxMesh.new()
		if pass_through:
			var h := 0.4
			box.size = Vector3(right - left, h, 5.0)
			mi.position = Vector3((left + right) / 2.0, top - h / 2.0, FRONT_Z - 2.5)
			mi.material_override = FighterView.toon(Color(0.84, 0.6, 0.34))
		else:
			box.size = Vector3(right - left, top - bottom, 8.0)
			mi.position = Vector3((left + right) / 2.0, (top + bottom) / 2.0, FRONT_Z - 4.0)
			mi.material_override = FighterView.toon(Color(0.5, 0.42, 0.55))
			# Grassy top strip.
			var cap := MeshInstance3D.new()
			var cb := BoxMesh.new()
			cb.size = Vector3(right - left + 0.1, 0.35, 8.1)
			cap.mesh = cb
			cap.position = Vector3(0, (top - bottom) / 2.0 - 0.17, 0)
			cap.material_override = FighterView.toon(Color(0.5, 0.78, 0.4))
			mi.add_child(cap)
		mi.mesh = box
		add_child(mi)

	for i in sim.ledge_count():
		var l: Vector3 = sim.ledge_pos(i)
		var mi := MeshInstance3D.new()
		var s := SphereMesh.new()
		s.radius = 0.28
		s.height = 0.56
		mi.mesh = s
		var m := StandardMaterial3D.new()
		m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		m.albedo_color = Color(1, 1, 1, 0.9)
		m.no_depth_test = true
		mi.material_override = m
		mi.position = Vector3(l.x, l.y, 0.6)
		add_child(mi)
		ledge_markers.append(mi)
		ledge_mats.append(m)

	_blast_zone(sim.blast_zone())


func update_ledges(sim) -> void:
	for i in ledge_markers.size():
		var owner: int = sim.ledge_owner(i)
		ledge_mats[i].albedo_color = Color(1, 1, 1, 0.9) if owner < 0 else FighterView.COLORS[owner % 4]


func _blast_zone(b: PackedFloat32Array) -> void:
	var im := ImmediateMesh.new()
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = Color(1.0, 0.3, 0.3, 0.5)
	im.surface_begin(Mesh.PRIMITIVE_LINE_STRIP, mat)
	for p in [Vector2(b[0], b[2]), Vector2(b[1], b[2]), Vector2(b[1], b[3]), Vector2(b[0], b[3]), Vector2(b[0], b[2])]:
		im.surface_add_vertex(Vector3(p.x, p.y, 0))
	im.surface_end()
	var mi := MeshInstance3D.new()
	mi.mesh = im
	add_child(mi)
