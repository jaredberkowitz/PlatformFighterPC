extends Node3D
## Builds the stage from sim data: solid blocks, pass-through platforms, ledges and blast zones, dressed in the stage's theme
## (`StageArt`, from the stage's look), with the theme's backdrop behind.

const FighterView := preload("res://scripts/fighter_view.gd")
const StageArt := preload("res://scripts/stage_art.gd")
const StageLight := preload("res://scripts/stage_light.gd")

## Fighters live in the z = 0 plane. Stage geometry is pushed BACK so its front face sits just in front of
## that plane. If a block were centred on z = 0, perspective would make its front face look wider than the
## fighters standing in front of it, and it would cover a fighter hanging from a ledge whenever the camera
## sat toward the middle of the stage.
const FRONT_Z := 0.3

var ledge_markers: Array[MeshInstance3D] = []
var ledge_mats: Array[StandardMaterial3D] = []
## The theme the stage was built with (sky colours and so on; see `StageArt.theme`).
var theme := {}
## Whether the blast zone outline and ledge markers are drawn (the training view; matches from the menus hide them).
var show_guides := true
var guides: Array[Node3D] = []


## Throws the old stage away (a different stage was chosen, or the match content changed).
func clear() -> void:
	for c in get_children():
		c.queue_free()
	ledge_markers.clear()
	ledge_mats.clear()
	guides.clear()


func build(sim) -> void:
	var look: Dictionary = sim.stage_look() if sim.has_method("stage_look") else {}
	theme = StageArt.theme(look)
	var rects := []
	for i in sim.platform_count():
		var r: PackedFloat32Array = sim.platform_rect(i)
		rects.append(r)
	# The extent of the solid ground, for placing decorations.
	var extent := []
	for r in rects:
		if r[4] > 0.5:
			continue
		if extent.is_empty():
			extent = [r[0], r[1], r[2], r[3]]
		else:
			extent = [minf(extent[0], r[0]), maxf(extent[1], r[1]), maxf(extent[2], r[2]), minf(extent[3], r[3])]
	for i in rects.size():
		var r: PackedFloat32Array = rects[i]
		var left := r[0]
		var right := r[1]
		var top := r[2]
		var bottom := r[3]
		if r[4] > 0.5:
			StageArt.platform(self, theme, left, right, top, FRONT_Z)
		else:
			# A block's top is grassed unless another solid block sits right on it and covers it.
			var exposed := true
			for o in rects:
				if o[4] < 0.5 and o != r and is_equal_approx(o[3], top) and o[0] <= left + 0.01 and o[1] >= right - 0.01:
					exposed = false
			StageArt.solid_block(self, theme, left, right, top, bottom, FRONT_Z, exposed)

	# The soft shading under platforms and at the foot of walls, baked once for this stage.
	StageLight.bake(rects)

	var scenery := Node3D.new()
	scenery.name = "Backdrop"
	add_child(scenery)
	StageArt.backdrop(scenery, theme, extent)

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
		guides.append(mi)

	_blast_zone(sim.blast_zone())
	set_guides(show_guides)


## Shows or hides the training guides (ledge markers and the blast zone outline).
func set_guides(on: bool) -> void:
	show_guides = on
	for g in guides:
		if is_instance_valid(g):
			g.visible = on


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
	guides.append(mi)
