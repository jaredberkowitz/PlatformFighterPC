extends Node3D
## Builds the stage from sim data: solid blocks, pass-through platforms, ledges and blast zones, dressed in the stage's theme
## (`StageArt`, from the stage's look), with the theme's backdrop behind.

const FighterView := preload("res://scripts/fighter_view.gd")
const StageArt := preload("res://scripts/stage_art.gd")
const StageLight := preload("res://scripts/stage_light.gd")
const StageBake := preload("res://scripts/stage_bake.gd")

## Fighters live in the z = 0 plane. Stage geometry is pushed BACK so its front face sits just in front of
## that plane. If a block were centred on z = 0, perspective would make its front face look wider than the
## fighters standing in front of it, and it would cover a fighter hanging from a ledge whenever the camera
## sat toward the middle of the stage.
## Where the stage's front edge is, toward the camera from the plane the fighters stand in. (2.0, was 0.3, art review 6, the owner after
## the reference fighter: fighters stand in the middle of the stage's top, with floor in front of them, not on its front lip.)
const FRONT_Z := StageArt.STAGE_FRONT

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
	# ...and the left and right of the lowest block (what hangs under the stage is fitted to it).
	var lowest := INF
	for r in rects:
		if r[4] < 0.5 and r[3] < lowest:
			lowest = r[3]
			if extent.size() == 4:
				extent.append_array([r[0], r[1]])
			else:
				extent[4] = r[0]
				extent[5] = r[1]
	# Baked light and a modelled front belong to this exact stage (scripts/stage_bake.gd).
	var bake_key := StageBake.key(String(theme.get("name", "")), rects)
	var front := StageBake.front_path(bake_key)
	StageArt.front_modelled = front != "" or StageBake.front_mock_path() != ""
	StageArt.island_mode = theme.has("quilt")
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
			StageArt.solid_block(self, theme, left, right, top, bottom, FRONT_Z, exposed, float(extent[2]))
	if front != "":
		StageBake.place_front(self, theme, (load(front) as PackedScene).instantiate())
	elif StageArt.island_mode:
		StageArt.quilted_island(self, theme, rects, FRONT_Z)
	StageArt.island_mode = false

	# The soft shading under platforms and at the foot of walls, baked once for this stage.
	StageLight.bake(rects, Vector2(FRONT_Z - StageArt.PLATFORM_DEPTH, FRONT_Z))

	var scenery := Node3D.new()
	scenery.name = "Backdrop"
	add_child(scenery)
	# Modelled trees for the theme, if it has them (scripts/stage_bake.gd TREES).
	var trees = StageBake.tree_source(String(theme.get("name", "")))
	StageArt.tree_mock = trees != null
	StageArt.tree_slots.clear()
	# A modelled ground and bushes for this exact stage, if it has them.
	var ground := StageBake.ground_source(bake_key)
	StageArt.ground_mock = not ground.is_empty()
	StageArt.bush_slots.clear()
	StageArt.backdrop(scenery, theme, extent)
	if StageArt.ground_mock:
		StageBake.place_ground(scenery, theme, ground)
		StageArt.ground_mock = false
	if StageArt.tree_mock:
		StageBake.place_trees(scenery, theme, StageArt.tree_slots, trees)
		StageArt.tree_mock = false
	# `--props=a,b,c` (art preview, docs/ART_WORKFLOW.md): the named props lined up along the stage's top under its own light.
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--props="):
			prop_lineup(self, arg.substr(8).split(","), extent)
	# Baked light for this exact stage, if it has one; `--bake-export` writes the scenery out for Blender instead.
	if OS.get_cmdline_user_args().has("--bake-export"):
		StageBake.export_scene(self, bake_key, rects, FRONT_Z)
	else:
		StageBake.apply(self, bake_key)
	StageBake.mock_front(self, theme)
	StageArt.front_modelled = false

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


## A preview row of authored props (art/blender/make_props.py) on the stage's top, each scaled to `height` (at most `width` wide) and
## standing on the top a little in front of the fighters' line, so a prop is judged in the stage's real light, haze and ink rather than in
## Blender's studio light. Its foot is where its model's lowest point is, so a prop that floats or sinks in its own file shows here.
static func prop_lineup(root: Node3D, names: PackedStringArray, extent: Array, height := 3.0, width := 5.0) -> void:
	if extent.size() < 3:
		return
	var left: float = extent[0] + 2.0
	var right: float = extent[1] - 2.0
	for i in names.size():
		var prop: Node3D = StageArt.Toonify.load_prop(names[i].strip_edges())
		if prop == null:
			push_warning("prop_lineup: no prop %s" % names[i])
			continue
		var box := AABB()
		var first := true
		for mi: MeshInstance3D in prop.find_children("*", "MeshInstance3D", true, false):
			var xf := Transform3D.IDENTITY
			var up: Node = mi
			while up != null and up != prop:
				xf = (up as Node3D).transform * xf
				up = up.get_parent()
			var b := xf * mi.get_aabb()
			box = b if first else box.merge(b)
			first = false
		var k := minf(height / maxf(box.size.y, 0.01), width / maxf(box.size.x, 0.01))
		var x := lerpf(left, right, (i + 0.5) / float(names.size()))
		prop.scale = Vector3.ONE * k
		# (The prop's own origin on the top: its model decides whether it stands, floats or sinks.)
		prop.position = Vector3(x, float(extent[2]), FRONT_Z - 1.2)
		root.add_child(prop)
